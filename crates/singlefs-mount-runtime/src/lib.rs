//! 挂载就起的后台运行库（D26（后台整理与放置回收） 已定项 13 第 5、6 条）：核心层不起线程，这一层在挂载入口起一条周期线程，
//! 调用方挂载之后不调任何后台入口，水位越线的池上 GC 自己跑。
//!
//! 一把锁：攒批的会话连同盘表都在一把锁里（[`MountedRuntime`]），前台入口与后台批在它上面排队。每次发布之间放锁
//! （`MountedSessionWithPendingChanges::run_compaction_in_the_background_for_at_most_one_publish`）；一批之内 GPU 仍在锁内等（D24（后台重活能不能卸给 GPU） 已定项 2 的射程，里程碑六拆）。
//! 「前台有 fsync 在等」是锁外的原子计数：核心层问它的那一刻后台线程正拿着锁，答案不能在锁里。
//!
//! 线程体之外有同步的 [`MountedRuntime::run_one_maintenance_pass`]：确定性测试不起线程，一拍一拍推，录制盘（不能跨线程）也走得通。
//! 一个维护线程、两个到期时刻：`publish_poll_period` 到期调 `publish_when_due`（D16（发布语义） 丢失窗口里的 P），`compaction_period` 到期推后台整理。
//! 每一拍（两个到期时刻都算）先重发冻结的发布：写盘报错之后前台的写都被拒，等不到整理到期。前台 fsync（[`MountedRuntime::publish_pending_now`]）持锁时冻结着也先重发一次：
//! 重发没成就交回错误，不答成功（D16（发布语义） 已定项 7）。
//!
//! 卸载等线程、重发冻结的发布、排空待发、走核心层的卸载；重发还失败就交回错误，盘交回，等价于崩溃（[`UnmountRefused::FrozenPublishResendFailed`]）。
//! 丢弃（Drop）只停线程，不写盘，等价于崩溃（下一次挂载走恢复）。线程或前台在锁里 panic 等价于崩溃：
//! 会话可能停在半个更新里，不接着用，之后每个入口报 [`RuntimeFailed`]，只能丢掉内存重挂。
#![forbid(unsafe_code)]

use std::any::Any;
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::channel;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use singlefs_core::address::DeviceIdentity;
use singlefs_core::background_compute::{
    AlarmOfABackgroundTaskStoppedAfterAComputeError, BackgroundComputeBackend,
    BackgroundComputeSettings, BackgroundComputeSettingsRefused,
    BackgroundGcCpuCompensationThreshold, BackgroundGpuComputeHandle, GpuDeviceIdentifier,
    GpuMemoryInBytes, BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
};
use singlefs_core::block_device::BlockDevice;
use singlefs_core::compaction_plan::{
    AnswerOfTheCallerBeforeTheNextBatch, CallerMonotonicClockReading,
    CallerOfTheBackgroundCompaction, CompactionMountParameters, CompactionPacing,
    CompactionRunStop, CompactionTimerFallback, CompactionWriteBandwidthLimit,
    COMPACTION_WRITE_BANDWIDTH_LIMIT_BY_DEFAULT,
};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{
    mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_parameters,
    unmount, MountError, MountOutput, RaisedFloor, Unmounted,
    WritableMountWithBackgroundComputeRefused,
};
use singlefs_core::mounted_session::{
    AcceptRefused, BackgroundCompactionAfterThePendingChangesRefused, BackgroundCompactionCall,
    BackgroundCompactionNotStarted, ChangeAccepted, CompactionRunOutcome, MountedSession,
    MountedSessionWithPendingChanges, PublishOfThePendingChangesRefused,
    PublishReportedByTheSession, UserChange, UserChangeAfterThePendingChangesRefused,
    UserChangeRefused,
};
use singlefs_core::transaction::PublishError;

/// 后台整理的周期默认：30 秒（D26（后台整理与放置回收） 已定项 13 第 5 条，挂载参数）。
pub const COMPACTION_PERIOD_BY_DEFAULT: Duration = Duration::from_secs(30);
/// `publish_when_due` 的周期默认：1 秒（数是拍的，相对 `T_time` 5 秒加 20% 的丢失窗口；挂载参数）。
pub const PUBLISH_POLL_PERIOD_BY_DEFAULT: Duration = Duration::from_secs(1);
/// 一拍里至多调几次放锁的后台入口：一趟至多 `3S − 4` 次发布（S 至多 16），另加几趟的收尾；用户改动不停时不让一拍一直不返回（卸载与 `publish_when_due` 要等它）。
pub const COMPACTION_CALLS_PER_MAINTENANCE_PASS_AT_MOST: u32 = 64;

/// 运行库的挂载参数（都不存盘）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeParameters {
    pub background_compute: BackgroundComputeSettings,
    pub gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
    /// 批大小与三条水位线（默认 16、全空聚簇段线 5% 下限 2、另两条关）。
    pub compaction: CompactionMountParameters,
    /// 后台整理的周期，大于 0。
    pub compaction_period: Duration,
    /// `publish_when_due` 的周期（D16（发布语义） 已定项 2、5 的 P），大于 0。
    pub publish_poll_period: Duration,
    pub write_bandwidth_limit: CompactionWriteBandwidthLimit,
}

impl RuntimeParameters {
    /// 全取默认。
    #[must_use]
    pub fn by_default() -> Self {
        RuntimeParameters {
            background_compute: BackgroundComputeSettings::PRODUCT,
            gc_cpu_compensation_threshold: BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
            compaction: CompactionMountParameters::BY_DEFAULT,
            compaction_period: COMPACTION_PERIOD_BY_DEFAULT,
            publish_poll_period: PUBLISH_POLL_PERIOD_BY_DEFAULT,
            write_bandwidth_limit: COMPACTION_WRITE_BANDWIDTH_LIMIT_BY_DEFAULT,
        }
    }

    /// 交给核心层后台入口的节奏：定时兜底的间隔取周期的一半。周期由运行库的睡眠定，核心层那一道闸只兜底：
    /// 间隔等于周期时，某次醒来比上一次早到几毫秒，核心层就答「间隔没到」，实际周期隔一次才放行。
    fn compaction_pacing(&self) -> CompactionPacing {
        let half_period_in_milliseconds =
            u64::try_from(self.compaction_period.as_millis() / 2).unwrap_or(u64::MAX);
        CompactionPacing {
            timer_fallback: match std::num::NonZeroU64::new(half_period_in_milliseconds) {
                Some(interval) => CompactionTimerFallback::EveryMilliseconds(interval),
                None => CompactionTimerFallback::EveryMilliseconds(std::num::NonZeroU64::MIN),
            },
            write_bandwidth_limit: self.write_bandwidth_limit,
        }
    }
}

/// 运行库的钟：核心层不读钟，定时兜底、写带宽与发布触发都读它。两个实现：产品的单调钟（[`MonotonicClockOfThisProcess`]）与测试摆的假钟。
pub trait ClockOfTheRuntime: Send + Sync {
    fn reading(&self) -> CallerMonotonicClockReading;
}

/// 从建它那一刻算的单调钟。
#[derive(Clone, Copy, Debug)]
pub struct MonotonicClockOfThisProcess {
    started_at: Instant,
}

impl MonotonicClockOfThisProcess {
    #[must_use]
    pub fn starting_now() -> Self {
        MonotonicClockOfThisProcess {
            started_at: Instant::now(),
        }
    }
}

impl ClockOfTheRuntime for MonotonicClockOfThisProcess {
    fn reading(&self) -> CallerMonotonicClockReading {
        CallerMonotonicClockReading(self.started_at.elapsed())
    }
}

/// 开卡没开成。成员按调用方要做的决定分。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackgroundGpuNotOpened {
    /// 这一次开卡过了期限还没回来：工作线程留在驱动里不杀（没有安全的杀法），挂载拒绝，调用线程不再等。`deadline` 是到点那一刻的期限
    /// （从开卡起算；开卡器的期限怎么定在实现里，产品的是枚举期限，枚举完延到枚举期限加卡数乘单卡上限）。
    TimedOut { deadline: Duration },
    /// 上一次开卡过了期限、到现在还没回来：闸住新开卡，不再多起一条枚举线程（同一个驱动加载器里再起一条会不会也挂没量过）。
    AnEarlierOpeningIsStillHung,
    /// 这个进程建实例的次数到了上限（新建到一定数量 Vulkan 驱动不再认得出独显，额度按进程生命期累计）：要重启进程。
    InstanceLimitOfThisProcessReached { instances_created: u32 },
    /// 开卡器自己拒的（设置被拒、点名的卡不在、参与的卡用不上这一类）：`description` 是开卡器报的原样。
    RefusedByTheOpening { description: String },
}

/// 开成的后台 GPU：交给核心层挂载的把手，开卡时跳过了哪几张卡的报告，与读它这一次挂载泄漏了哪几张卡的那一份
/// （`.claude/rules/fs-design.md` 五条硬要求第 4 条；开卡器给不出这一份的是 `None`，产品开卡器恒给）。运行库留着后两样：
/// [`MountedRuntime::report_of_the_opening`]、[`MountedRuntime::gpus_leaked_by_the_background_gpu`]，卸载之后在 [`UnmountedPool`] 里。
#[derive(Debug)]
pub struct OpenedBackgroundGpu {
    pub gpu_compute: BackgroundGpuComputeHandle,
    pub report_of_the_opening: String,
    pub gpus_leaked: Option<Arc<dyn ReadsTheGpusLeakedByTheBackgroundGpu>>,
}

/// 后台 GPU 在这一次挂载里泄漏的一张卡（D26（后台整理与放置回收） 已定项 13 第 4 条：切换关卡、卸载时有提交还在跑的那张整张泄漏、不析构，并报出来）：
/// 哪张卡、泄漏那一刻本包在它上面占着的显存。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuLeakedByTheBackgroundGpu {
    pub device: GpuDeviceIdentifier,
    pub memory_held_on_the_device: GpuMemoryInBytes,
}

/// 读开成的后台 GPU 在这一次挂载里泄漏了哪几张卡（实现在产品 GPU 包，开卡器开成时交来）：随时读，后台 GPU 丢掉之后照样读得到。
pub trait ReadsTheGpusLeakedByTheBackgroundGpu: fmt::Debug + Send + Sync {
    /// 挂着切换关卡时泄漏的卡，按关的先后累计；没有是空。
    fn gpus_leaked_when_closed_at_a_switch(&self) -> Vec<GpuLeakedByTheBackgroundGpu>;

    /// 后台 GPU 丢掉（卸载、丢掉核心层收的把手）那一刻泄漏的卡；还没丢是 `None`，丢了而一张都没泄漏是空。
    fn gpus_leaked_when_dropped(&self) -> Option<Vec<GpuLeakedByTheBackgroundGpu>>;
}

/// 开卡器（D26（后台整理与放置回收） 已定项 13 第 4 条「挂载时由调用方把它的实现递进去」）：运行库不依赖产品 GPU 包，实现在那边
/// （`singlefs_background_gpu::gpu_opener_of_this_process`）。开卡在挂载之前：选 `gpu` 而开不成就在读任何一块盘之前拒绝挂载。
/// 期限、闸、进程级实例与泄漏账都在实现里；这里只收到一个结局，所以这一次调用要在实现自己的期限之内返回。
pub trait OpensTheBackgroundGpu {
    /// # Errors
    /// [`BackgroundGpuNotOpened`] 的几种。
    fn open_the_background_gpu(
        &self,
        settings: &BackgroundComputeSettings,
    ) -> Result<OpenedBackgroundGpu, BackgroundGpuNotOpened>;
}

/// 维护线程上每一件事的结局（`BackgroundCompactionCall` 是 `#[must_use]`：报警通道就是这里）。排空待发那几次发布是前台调用方不知道的发布点，一并送出。
#[derive(Debug)]
pub enum MaintenanceEvent<'event> {
    /// `publish_when_due` 或整理之前排空发成的一次发布。
    PendingChangesPublished(&'event PublishReportedByTheSession),
    /// `publish_when_due` 那一次没发成。
    PublishWhenDueRefused(&'event PublishOfThePendingChangesRefused),
    /// 后台入口这一次调用的结局，整理停着时带着报警（`BackgroundCompactionCall::alarm_of_the_compaction_stopped_after_a_compute_error`）。
    CompactionCallReturned(&'event BackgroundCompactionCall),
    /// 后台入口这一次调用没跑（盘表不对、现行那一版没有树、排空被拒）。
    CompactionCallRefused(&'event BackgroundCompactionAfterThePendingChangesRefused),
    /// 写盘报错把发布路径冻结了，这一次重发成了（前台 fsync 那一次也算）：前台的写不再因冻结被拒；重发的是 unlink 的意图时还要等删除队列走空（后台入口因队列不空起跑）。
    FrozenPublishResent,
    /// 冻结着的那一次这一拍重发又报错：照旧冻结，前台的写照旧被拒（`AcceptRefused::PublishFrozenAfterAWriteFailureIsNotResentYet`），整理这一拍不跑。
    FrozenPublishResendFailed(&'event PublishError),
    /// 前台 [`MountedRuntime::accept`] 这一次调用为空间推成了这几串抬 F 的空发布（收没收下都报）：调用方看不到发布点的接收方靠它知道根环走了几格。
    /// 这几种串不报，根环照样走了：屏障进门排空待发与屏障自己发布时推的串（在 `AcceptRefused` 的屏障那两个成员或那几次发布的报里）、
    /// 收下之后触发的那一次发布自己推的串、抬 F 半路报错那一串里已落盘的那几次（`FloorRaiseFailedWhilePushingForSpace` 的 `floor_raises` 不含它们）。
    FloorRaisesPushedByAnAccept(&'event [RaisedFloor]),
    /// 维护线程 panic 了：等价于崩溃，运行库从此报 [`RuntimeFailed`]。
    MaintenanceThreadPanicked { description: &'event str },
}

/// 维护线程的事件送到哪。至少两个实现：不记的（[`IgnoringTheMaintenanceEvents`]）与调用方自己的（首个真正的入口是 FUSE 守护进程，它把报警告诉文件系统的用户）。
pub trait ReceiverOfTheMaintenanceEvents: Send + Sync {
    fn maintenance_event(&self, event: &MaintenanceEvent<'_>);
}

/// 不记事件的接收方：只看 [`MountedRuntime::observations`] 的调用方用它。
#[derive(Clone, Copy, Debug, Default)]
pub struct IgnoringTheMaintenanceEvents;

impl ReceiverOfTheMaintenanceEvents for IgnoringTheMaintenanceEvents {
    fn maintenance_event(&self, _event: &MaintenanceEvent<'_>) {}
}

/// 运行库累计的观测（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。只住内存。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeObservations {
    pub maintenance_passes: u64,
    pub compaction_calls: u64,
    /// 后台入口发成的整理发布次数（签发、搬一批、删意图、丢意图），不含删除续做。
    pub compaction_publishes: u64,
    /// 后台入口发成的删除续做步数。
    pub deletion_continuation_publishes: u64,
    /// 维护线程发成的排空待发次数。
    pub pending_publishes: u64,
    pub alarms_received: u64,
    pub last_alarm: Option<AlarmOfABackgroundTaskStoppedAfterAComputeError>,
    pub last_not_started: Option<BackgroundCompactionNotStarted>,
    pub last_stop: Option<CompactionRunStop>,
    pub frozen_publishes_resent: u64,
    pub frozen_publish_resends_failed: u64,
    /// 前台 `accept` 为空间推成的抬 F 串数。
    pub floor_raises_pushed_by_accepts: u64,
    /// 那几串一共发了几次空发布（占根环的格数）。
    pub empty_publishes_pushed_by_accepts: u64,
}

/// 同步的一拍做成了什么（[`MountedRuntime::run_one_maintenance_pass`]）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MaintenancePass {
    pub compaction_calls: u32,
    /// 这一拍的整理发布次数，不含删除续做。
    pub compaction_publishes: u64,
    /// 这一拍走的删除续做步数。
    pub deletion_continuation_publishes: u64,
    /// 这一趟被节奏截住、或到了一拍的调用次数上限而没完：下一拍不等一个周期。
    pub is_a_trip_under_way: bool,
}

/// 线程或前台在锁里 panic 过：会话可能停在半个更新里，只能丢掉内存、从盘重挂。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFailed {
    pub description: String,
}

/// 前台入口没做成。
#[derive(Debug)]
pub enum RuntimeRefused<Refusal> {
    /// 核心层拒的，原样。
    Refused(Refusal),
    /// 运行库已经坏了。
    Failed(RuntimeFailed),
}

/// 挂载没做成的原因。
#[derive(Debug)]
pub enum RuntimeMountRefusal {
    /// 周期是 0，在读任何一块盘之前拒。
    PeriodOfZero,
    /// 选 `gpu` 而开卡没成，在读任何一块盘之前拒。
    BackgroundGpuNotOpened(BackgroundGpuNotOpened),
    /// 核心层拒的，原样。
    Mount(WritableMountWithBackgroundComputeRefused),
    /// 维护线程起不来。挂载已经写过盘，会话丢掉，等价于崩溃：下一次挂载走恢复。
    MaintenanceThreadNotStarted { description: String },
}

/// 挂载没做成：盘原样交回。
pub struct RuntimeMountRefused<Device> {
    pub devices: Vec<(DeviceIdentity, Device)>,
    pub reason: RuntimeMountRefusal,
}

impl<Device> fmt::Debug for RuntimeMountRefused<Device> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeMountRefused")
            .field("reason", &self.reason)
            .finish_non_exhaustive()
    }
}

/// 挂载做成了：挂载写出的东西（挂载续做那一趟的结局在里面）与运行库。
#[must_use = "丢掉运行库只停线程、不排空待发，等价于崩溃：要 unmount"]
pub struct RuntimeMounted<Device> {
    pub output: MountOutput,
    pub runtime: MountedRuntime<Device>,
}

impl<Device> fmt::Debug for RuntimeMounted<Device> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RuntimeMounted")
            .field("runtime", &self.runtime)
            .finish_non_exhaustive()
    }
}

/// 卸载做成了：盘交回。
pub struct UnmountedPool<Device> {
    pub devices: Vec<(DeviceIdentity, Device)>,
    /// 卸载之前排空待发那一次（待发空时 `None`）。
    pub pending_changes_published: Option<PublishReportedByTheSession>,
    pub unmounted: Unmounted,
    /// 挂载那一刻开卡器开成的后台 GPU 泄漏了哪几张卡（[`OpenedBackgroundGpu::gpus_leaked`]；卸载时丢掉的那一份已记下）。
    /// 没开卡、或开卡器给不出这一份的是 `None`。
    pub gpus_leaked_by_the_background_gpu: Option<Arc<dyn ReadsTheGpusLeakedByTheBackgroundGpu>>,
}

impl<Device> fmt::Debug for UnmountedPool<Device> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UnmountedPool")
            .field("pending_changes_published", &self.pending_changes_published)
            .field(
                "gpus_leaked_by_the_background_gpu",
                &self.gpus_leaked_by_the_background_gpu,
            )
            .finish_non_exhaustive()
    }
}

/// 卸载没做成。
pub enum UnmountRefused<Device> {
    /// 运行库已经坏了：内存不接着用，盘交回，等价于崩溃。
    Failed {
        devices: Vec<(DeviceIdentity, Device)>,
        failure: RuntimeFailed,
    },
    /// 卸载先重发冻结的那一次，重发又报错：盘交回，内存不接着用，等价于崩溃。那一次可能成了也可能没成（D16（发布语义） 已定项 17）：报错可能在根落盘之后，下一次挂载走恢复，读现行根判。
    FrozenPublishResendFailed {
        devices: Vec<(DeviceIdentity, Device)>,
        refusal: Box<PublishError>,
    },
    /// 排空待发没发成：运行库原样交回，待发没冻结时还在里面。维护线程已经停了，它不再自己发布与整理：腾出空间之后再调一次卸载（卸载先重发冻结的那一次）；丢掉它就是崩溃。
    PendingChangesNotPublished {
        runtime: MountedRuntime<Device>,
        refusal: Box<UserChangeRefused>,
    },
    /// 核心层的卸载没做成：盘交回，内存不接着用，等价于崩溃。
    UnmountFailed {
        devices: Vec<(DeviceIdentity, Device)>,
        refusal: MountError,
    },
}

impl<Device> fmt::Debug for UnmountRefused<Device> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnmountRefused::Failed { failure, .. } => formatter
                .debug_struct("Failed")
                .field("failure", failure)
                .finish_non_exhaustive(),
            UnmountRefused::FrozenPublishResendFailed { refusal, .. } => formatter
                .debug_struct("FrozenPublishResendFailed")
                .field("refusal", refusal)
                .finish_non_exhaustive(),
            UnmountRefused::PendingChangesNotPublished { refusal, .. } => formatter
                .debug_struct("PendingChangesNotPublished")
                .field("refusal", refusal)
                .finish_non_exhaustive(),
            UnmountRefused::UnmountFailed { refusal, .. } => formatter
                .debug_struct("UnmountFailed")
                .field("refusal", refusal)
                .finish_non_exhaustive(),
        }
    }
}

/// 一把锁里的东西：攒批的会话与盘表。
pub struct SessionAndDevices<Device> {
    pub session: MountedSessionWithPendingChanges,
    pub devices: Vec<(DeviceIdentity, Device)>,
}

/// 运行库各处共用的那一份。`fsync_waiters`、`stopping` 不在会话锁里：核心层问它们的那一刻后台线程正拿着锁。
struct Shared<Device> {
    parameters: MakeFilesystemParameters,
    runtime_parameters: RuntimeParameters,
    locked: Mutex<SessionAndDevices<Device>>,
    fsync_waiters: AtomicU32,
    stopping: AtomicBool,
    sleep_lock: Mutex<()>,
    wake_the_maintenance_thread: Condvar,
    failed: Mutex<Option<RuntimeFailed>>,
    observations: Mutex<RuntimeObservations>,
    clock: Arc<dyn ClockOfTheRuntime>,
    receiver: Arc<dyn ReceiverOfTheMaintenanceEvents>,
    /// 挂载那一刻开卡器开成的后台 GPU：开卡的报告与读它泄漏了哪几张卡的那一份。没开卡是 `None`。
    background_gpu_opened_at_the_mount: Option<BackgroundGpuOpenedAtTheMount>,
}

/// 运行库留着的、挂载那一刻开卡器开成的后台 GPU 的两样（[`OpenedBackgroundGpu`] 里除把手之外的）。
#[derive(Clone, Debug)]
struct BackgroundGpuOpenedAtTheMount {
    report_of_the_opening: String,
    gpus_leaked: Option<Arc<dyn ReadsTheGpusLeakedByTheBackgroundGpu>>,
}

/// 后台入口每一批之前问的两样（核心层不起线程，钟与前台 fsync 都在这一层）。停线程时答「在停线程」：整理在下一批之前就返回，线程很快 join 得上。
struct CallerOfTheRuntime<'shared> {
    clock: &'shared dyn ClockOfTheRuntime,
    fsync_waiters: &'shared AtomicU32,
    stopping: &'shared AtomicBool,
}

impl CallerOfTheBackgroundCompaction for CallerOfTheRuntime<'_> {
    fn clock_reading(&self) -> CallerMonotonicClockReading {
        self.clock.reading()
    }

    fn answer_before_the_next_batch(&self) -> AnswerOfTheCallerBeforeTheNextBatch {
        if self.stopping.load(Ordering::SeqCst) {
            AnswerOfTheCallerBeforeTheNextBatch::CallerIsStoppingItsThread
        } else if self.fsync_waiters.load(Ordering::SeqCst) > 0 {
            AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting
        } else {
            AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
        }
    }
}

/// 前台有一个 fsync 在等：取锁之前登记，丢弃时减（错误返回与 panic 展开都走到）。
#[must_use = "丢掉就不算在等了"]
pub struct ForegroundFsyncWaiting<'runtime> {
    fsync_waiters: &'runtime AtomicU32,
}

impl Drop for ForegroundFsyncWaiting<'_> {
    fn drop(&mut self) {
        self.fsync_waiters.fetch_sub(1, Ordering::SeqCst);
    }
}

impl<Device> Shared<Device> {
    /// 叫维护线程停：拿着睡眠锁通知，线程在睡之前查 `stopping`，通知丢不了。
    fn ask_the_maintenance_thread_to_stop(&self) {
        self.stopping.store(true, Ordering::SeqCst);
        let _sleep_guard = self
            .sleep_lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        self.wake_the_maintenance_thread.notify_all();
    }
}

fn description_of_a_panic(payload: &(dyn Any + Send)) -> String {
    match (
        payload.downcast_ref::<&str>(),
        payload.downcast_ref::<String>(),
    ) {
        (Some(text), _) => (*text).to_string(),
        (None, Some(text)) => text.clone(),
        (None, None) => "panic 带的不是字符串".to_string(),
    }
}

impl<Device: BlockDevice> Shared<Device> {
    /// 记下坏了（只记第一次）。观测与坏的记号在各自的小锁里，那几把锁里不跑会 panic 的代码，中毒照用。
    fn note_the_failure(&self, description: String) {
        let mut failed = self.failed.lock().unwrap_or_else(PoisonError::into_inner);
        if failed.is_none() {
            *failed = Some(RuntimeFailed { description });
        }
    }

    fn failure_so_far(&self) -> Option<RuntimeFailed> {
        self.failed
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// 取会话锁。坏过或锁中毒都报 [`RuntimeFailed`]：不用 `into_inner` 接着用半个更新里的会话。
    fn lock_the_session(&self) -> Result<MutexGuard<'_, SessionAndDevices<Device>>, RuntimeFailed> {
        if let Some(failure) = self.failure_so_far() {
            return Err(failure);
        }
        self.locked.lock().map_err(|_poisoned| {
            let failure = RuntimeFailed {
                description: "持着会话锁的线程 panic 过".to_string(),
            };
            self.note_the_failure(failure.description.clone());
            failure
        })
    }

    fn update_the_observations(&self, update: impl FnOnce(&mut RuntimeObservations)) {
        let mut observations = self
            .observations
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        update(&mut observations);
    }

    fn deliver(&self, event: &MaintenanceEvent<'_>) {
        self.receiver.maintenance_event(event);
    }

    /// 记下一次 `accept` 推成的几串抬 F，一串都没推不记（锁已放开：事件不在锁里送）。
    fn note_the_floor_raises_pushed_by_an_accept(&self, floor_raises: &[RaisedFloor]) {
        if floor_raises.is_empty() {
            return;
        }
        let empty_publishes: usize = floor_raises
            .iter()
            .map(|raised| raised.publishes.len())
            .sum();
        self.update_the_observations(|observations| {
            observations.floor_raises_pushed_by_accepts +=
                u64::try_from(floor_raises.len()).expect("一次准入至多推几串");
            observations.empty_publishes_pushed_by_accepts +=
                u64::try_from(empty_publishes).expect("一次准入至多推几次空发布");
        });
        self.deliver(&MaintenanceEvent::FloorRaisesPushedByAnAccept(floor_raises));
    }

    /// 记下一次重发成了（锁已放开：事件不在锁里送）。
    fn note_a_frozen_publish_resent(&self) {
        self.update_the_observations(|observations| {
            observations.frozen_publishes_resent += 1;
        });
        self.deliver(&MaintenanceEvent::FrozenPublishResent);
    }

    /// 记下一次重发又报错（锁已放开：事件不在锁里送）。
    fn note_a_frozen_publish_resend_failed(&self, refusal: &PublishError) {
        self.update_the_observations(|observations| {
            observations.frozen_publish_resends_failed += 1;
        });
        self.deliver(&MaintenanceEvent::FrozenPublishResendFailed(refusal));
    }

    fn caller(&self) -> CallerOfTheRuntime<'_> {
        CallerOfTheRuntime {
            clock: &*self.clock,
            fsync_waiters: &self.fsync_waiters,
            stopping: &self.stopping,
        }
    }

    /// `publish_when_due` 一次（持锁一次发布），结局送接收方。
    fn publish_when_due_once(&self) -> Result<(), RuntimeFailed> {
        let now = self.clock.reading();
        let outcome = {
            let mut locked = self.lock_the_session()?;
            let SessionAndDevices { session, devices } = &mut *locked;
            session.publish_when_due(devices, now)
        };
        match &outcome {
            Ok(Some(published)) => {
                self.update_the_observations(|observations| observations.pending_publishes += 1);
                self.deliver(&MaintenanceEvent::PendingChangesPublished(published));
            }
            Ok(None) => {}
            Err(refusal) => self.deliver(&MaintenanceEvent::PublishWhenDueRefused(refusal)),
        }
        Ok(())
    }

    /// 上一次写盘报错冻结了发布就重发一次。交回这一拍还冻不冻着：冻着时 `publish_when_due` 与整理都不跑（每次都是被拒一次）。
    fn resend_the_frozen_publish_if_any(&self) -> Result<bool, RuntimeFailed> {
        let mut locked = self.lock_the_session()?;
        let SessionAndDevices { session, devices } = &mut *locked;
        if session.session().allocator.frozen_publish().is_none() {
            return Ok(false);
        }
        let outcome = session.resend_the_frozen_publish(devices);
        drop(locked);
        match outcome {
            Ok(_resent) => {
                self.note_a_frozen_publish_resent();
                Ok(false)
            }
            Err(refusal) => {
                self.note_a_frozen_publish_resend_failed(&refusal);
                Ok(true)
            }
        }
    }

    /// 一拍：重发冻结的那一次、`publish_when_due`，再一次次调放锁的后台入口，每次之间放锁。没起跑、被节奏截住、出错或到了调用次数上限就停。
    /// 用户改动之后那一趟（至多一批）停了接着调：定时兜底那一支在同一拍里轮得到，用户改动不断时整理不会只剩每拍一批。
    fn run_one_maintenance_pass(&self) -> Result<MaintenancePass, RuntimeFailed> {
        self.update_the_observations(|observations| observations.maintenance_passes += 1);
        let mut pass = MaintenancePass::default();
        if self.resend_the_frozen_publish_if_any()? {
            return Ok(pass);
        }
        self.publish_when_due_once()?;
        let pacing = self.runtime_parameters.compaction_pacing();
        let caller = self.caller();
        while pass.compaction_calls < COMPACTION_CALLS_PER_MAINTENANCE_PASS_AT_MOST
            && !self.stopping.load(Ordering::SeqCst)
        {
            let (outcome, is_a_trip_under_way) = {
                let mut locked = self.lock_the_session()?;
                let SessionAndDevices { session, devices } = &mut *locked;
                let outcome = session.run_compaction_in_the_background_for_at_most_one_publish(
                    devices, pacing, &caller,
                );
                let is_a_trip_under_way = session
                    .session()
                    .compaction
                    .has_a_background_trip_under_way();
                (outcome, is_a_trip_under_way)
            };
            pass.compaction_calls += 1;
            pass.is_a_trip_under_way = is_a_trip_under_way;
            self.update_the_observations(|observations| observations.compaction_calls += 1);
            let completed = match &outcome {
                Ok(completed) => completed,
                Err(refusal) => {
                    self.deliver(&MaintenanceEvent::CompactionCallRefused(refusal));
                    break;
                }
            };
            if let Some(published) = &completed.pending_changes_published_before_the_run {
                self.update_the_observations(|observations| observations.pending_publishes += 1);
                self.deliver(&MaintenanceEvent::PendingChangesPublished(published));
            }
            self.deliver(&MaintenanceEvent::CompactionCallReturned(&completed.call));
            if let Some(alarm) = completed
                .call
                .alarm_of_the_compaction_stopped_after_a_compute_error()
            {
                self.update_the_observations(|observations| {
                    observations.alarms_received += 1;
                    observations.last_alarm = Some(alarm.clone());
                });
            }
            let keeps_going = match &completed.call {
                BackgroundCompactionCall::NotStarted { reason, .. } => {
                    self.update_the_observations(|observations| {
                        observations.last_not_started = Some(*reason);
                    });
                    false
                }
                BackgroundCompactionCall::Ran { run, .. } => {
                    let deletion_steps = run.deletion_steps_published;
                    let publishes = u64::try_from(run.versions_published.len())
                        .expect("一次调用的发布数")
                        - deletion_steps;
                    pass.compaction_publishes += publishes;
                    pass.deletion_continuation_publishes += deletion_steps;
                    self.update_the_observations(|observations| {
                        observations.compaction_publishes += publishes;
                        observations.deletion_continuation_publishes += deletion_steps;
                    });
                    match &run.outcome {
                        CompactionRunOutcome::Stopped(stop) => {
                            self.update_the_observations(|observations| {
                                observations.last_stop = Some(*stop);
                            });
                            !matches!(
                                stop,
                                CompactionRunStop::YieldedToAForegroundFsync
                                    | CompactionRunStop::YieldedToTheCallerStoppingItsThread
                                    | CompactionRunStop::WriteBandwidthLimitReached { .. }
                            )
                        }
                        CompactionRunOutcome::Failed(_) => false,
                    }
                }
            };
            if !keeps_going {
                break;
            }
            self.publish_when_due_once()?;
        }
        Ok(pass)
    }

    /// 线程体：两个到期时刻。`stopping` 在睡之前查、卸载在拿着睡眠锁时通知，通知丢不了。
    fn maintenance_thread_body(&self) -> Result<(), RuntimeFailed> {
        let compaction_period = self.runtime_parameters.compaction_period;
        let publish_poll_period = self.runtime_parameters.publish_poll_period;
        let mut next_compaction_at = self.clock.reading().0;
        while !self.stopping.load(Ordering::SeqCst) {
            let now = self.clock.reading().0;
            if now >= next_compaction_at {
                let pass = self.run_one_maintenance_pass()?;
                next_compaction_at = now
                    + if pass.is_a_trip_under_way {
                        publish_poll_period
                    } else {
                        compaction_period
                    };
            } else if !self.resend_the_frozen_publish_if_any()? {
                self.publish_when_due_once()?;
            }
            let until_the_next_compaction =
                next_compaction_at.saturating_sub(self.clock.reading().0);
            let sleep_for = publish_poll_period.min(until_the_next_compaction);
            let sleep_guard = self
                .sleep_lock
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            if !self.stopping.load(Ordering::SeqCst) {
                let _woken_or_timed_out = self
                    .wake_the_maintenance_thread
                    .wait_timeout(sleep_guard, sleep_for)
                    .unwrap_or_else(PoisonError::into_inner);
            }
        }
        Ok(())
    }
}

/// 一次可写挂载之后的运行库。`Device` 要 `Send` 才起得了线程；录制盘走不起线程的那一种（[`Self::mount_without_the_maintenance_thread`]）。
#[must_use = "丢掉运行库只停线程、不排空待发，等价于崩溃：要 unmount"]
pub struct MountedRuntime<Device> {
    shared: Arc<Shared<Device>>,
    maintenance_thread: Option<JoinHandle<()>>,
}

impl<Device> fmt::Debug for MountedRuntime<Device> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MountedRuntime")
            .field(
                "has_a_maintenance_thread",
                &self.maintenance_thread.is_some(),
            )
            .finish_non_exhaustive()
    }
}

impl<Device: BlockDevice + Send + 'static> MountedRuntime<Device> {
    /// 挂载：校验周期 → 开卡（只在选 `gpu` 时碰开卡器，读任何一块盘之前）→ 可写挂载（挂载续做在这里跑完，线程还没起，没有交错）→ 接会话 → 起维护线程。
    /// 调用方之后不调任何后台入口。
    ///
    /// # Errors
    /// [`RuntimeMountRefused`]：盘原样交回。
    pub fn mount(
        parameters: &MakeFilesystemParameters,
        devices: Vec<(DeviceIdentity, Device)>,
        runtime_parameters: RuntimeParameters,
        gpu_opener: Option<&dyn OpensTheBackgroundGpu>,
        clock: Arc<dyn ClockOfTheRuntime>,
        receiver: Arc<dyn ReceiverOfTheMaintenanceEvents>,
    ) -> Result<RuntimeMounted<Device>, RuntimeMountRefused<Device>> {
        let RuntimeMounted {
            output,
            mut runtime,
        } = MountedRuntime::mount_without_the_maintenance_thread(
            parameters,
            devices,
            runtime_parameters,
            gpu_opener,
            clock,
            receiver,
        )?;
        // 线程起来之后才把这一份交给它：起不来时没有第二个持有者，盘交得回去。
        let (hand_over_the_shared, shared_handed_over) = channel::<Arc<Shared<Device>>>();
        let started = std::thread::Builder::new()
            .name("singlefs-maintenance".to_string())
            .spawn(move || {
                let Ok(shared_of_the_thread) = shared_handed_over.recv() else {
                    return;
                };
                let body = catch_unwind(AssertUnwindSafe(|| {
                    shared_of_the_thread.maintenance_thread_body()
                }));
                match body {
                    Ok(Ok(())) | Ok(Err(_)) => {}
                    Err(payload) => {
                        let description = description_of_a_panic(&*payload);
                        shared_of_the_thread.note_the_failure(description.clone());
                        shared_of_the_thread.deliver(
                            &MaintenanceEvent::MaintenanceThreadPanicked {
                                description: &description,
                            },
                        );
                    }
                }
            });
        match started {
            Ok(thread) => {
                // 送不出去只有一种：线程在收之前就结束了，这一份留在这里随运行库走。
                let _thread_gone_before_taking_it =
                    hand_over_the_shared.send(Arc::clone(&runtime.shared));
                runtime.maintenance_thread = Some(thread);
                Ok(RuntimeMounted { output, runtime })
            }
            Err(error) => {
                let shared = Arc::clone(&runtime.shared);
                drop(runtime);
                let devices_handed_back = Arc::try_unwrap(shared)
                    .unwrap_or_else(|_still_shared| {
                        panic!("维护线程没起来，从没拿到过这一份；运行库已丢：这一份是唯一持有者")
                    })
                    .locked
                    .into_inner()
                    .unwrap_or_else(PoisonError::into_inner)
                    .devices;
                Err(RuntimeMountRefused {
                    devices: devices_handed_back,
                    reason: RuntimeMountRefusal::MaintenanceThreadNotStarted {
                        description: error.to_string(),
                    },
                })
            }
        }
    }
}

impl<Device: BlockDevice> MountedRuntime<Device> {
    /// 同 [`MountedRuntime::mount`]，不起线程：调用方一拍一拍调 [`Self::run_one_maintenance_pass`]（确定性测试走它，录制盘过不了线程）。
    ///
    /// # Errors
    /// [`RuntimeMountRefused`]：盘原样交回。
    pub fn mount_without_the_maintenance_thread(
        parameters: &MakeFilesystemParameters,
        mut devices: Vec<(DeviceIdentity, Device)>,
        runtime_parameters: RuntimeParameters,
        gpu_opener: Option<&dyn OpensTheBackgroundGpu>,
        clock: Arc<dyn ClockOfTheRuntime>,
        receiver: Arc<dyn ReceiverOfTheMaintenanceEvents>,
    ) -> Result<RuntimeMounted<Device>, RuntimeMountRefused<Device>> {
        if runtime_parameters.compaction_period.is_zero()
            || runtime_parameters.publish_poll_period.is_zero()
        {
            return Err(RuntimeMountRefused {
                devices,
                reason: RuntimeMountRefusal::PeriodOfZero,
            });
        }
        let (gpu_compute, background_gpu_opened_at_the_mount) = match (
            &runtime_parameters.background_compute.background_compute,
            gpu_opener,
        ) {
            (BackgroundComputeBackend::Gpu, Some(opener)) => {
                match opener.open_the_background_gpu(&runtime_parameters.background_compute) {
                    Ok(opened) => (
                        Some(opened.gpu_compute),
                        Some(BackgroundGpuOpenedAtTheMount {
                            report_of_the_opening: opened.report_of_the_opening,
                            gpus_leaked: opened.gpus_leaked,
                        }),
                    ),
                    Err(not_opened) => {
                        return Err(RuntimeMountRefused {
                            devices,
                            reason: RuntimeMountRefusal::BackgroundGpuNotOpened(not_opened),
                        })
                    }
                }
            }
            (BackgroundComputeBackend::Gpu, None) | (BackgroundComputeBackend::Cpu, _) => {
                (None, None)
            }
        };
        let mounted = match mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_parameters(
            parameters,
            &mut devices,
            runtime_parameters.background_compute.clone(),
            gpu_compute,
            runtime_parameters.gc_cpu_compensation_threshold,
            runtime_parameters.compaction,
        ) {
            Ok(mounted) => mounted,
            Err(refused) => {
                return Err(RuntimeMountRefused {
                    devices,
                    reason: RuntimeMountRefusal::Mount(refused),
                })
            }
        };
        let (output, session) = MountedSession::of_the_mount(mounted);
        let runtime = MountedRuntime::of_the_session(
            parameters.clone(),
            runtime_parameters,
            SessionAndDevices {
                session: MountedSessionWithPendingChanges::of_the_session(session),
                devices,
            },
            clock,
            receiver,
            background_gpu_opened_at_the_mount,
        );
        Ok(RuntimeMounted { output, runtime })
    }

    fn of_the_session(
        parameters: MakeFilesystemParameters,
        runtime_parameters: RuntimeParameters,
        locked: SessionAndDevices<Device>,
        clock: Arc<dyn ClockOfTheRuntime>,
        receiver: Arc<dyn ReceiverOfTheMaintenanceEvents>,
        background_gpu_opened_at_the_mount: Option<BackgroundGpuOpenedAtTheMount>,
    ) -> Self {
        MountedRuntime {
            shared: Arc::new(Shared {
                parameters,
                runtime_parameters,
                locked: Mutex::new(locked),
                fsync_waiters: AtomicU32::new(0),
                stopping: AtomicBool::new(false),
                sleep_lock: Mutex::new(()),
                wake_the_maintenance_thread: Condvar::new(),
                failed: Mutex::new(None),
                observations: Mutex::new(RuntimeObservations::default()),
                clock,
                receiver,
                background_gpu_opened_at_the_mount,
            }),
            maintenance_thread: None,
        }
    }

    /// 挂载那一刻开卡器开成的后台 GPU 开卡时的报告（跳过了哪几张卡）；没开卡是 `None`。
    #[must_use]
    pub fn report_of_the_opening(&self) -> Option<String> {
        self.shared
            .background_gpu_opened_at_the_mount
            .as_ref()
            .map(|opened| opened.report_of_the_opening.clone())
    }

    /// 读挂载那一刻开卡器开成的后台 GPU 泄漏了哪几张卡的那一份（[`ReadsTheGpusLeakedByTheBackgroundGpu`]：切换关卡时泄漏的随时读得到，
    /// 卸载时丢掉的那一份卸载之后读得到，[`UnmountedPool::gpus_leaked_by_the_background_gpu`] 是同一份）。没开卡、或开卡器给不出这一份的是 `None`。
    #[must_use]
    pub fn gpus_leaked_by_the_background_gpu(
        &self,
    ) -> Option<Arc<dyn ReadsTheGpusLeakedByTheBackgroundGpu>> {
        self.shared
            .background_gpu_opened_at_the_mount
            .as_ref()
            .and_then(|opened| opened.gpus_leaked.clone())
    }

    /// 前台收下一次用户改动（`MountedSessionWithPendingChanges::accept`）。被空间拒时这一次调用当场推抬 F 再判（D16（发布语义） 已定项 1「准入」那一行）：
    /// 收下之前可能发抬 F 的空发布，一次准入至多 B − 1 次，全程持着会话那把锁（后台批与别的前台入口等到它返回；发布之间不放锁，推的那一串与再判之间不插别的发布），
    /// 写盘的时长算进这一次调用。推成的报在 [`ChangeAccepted`] 的 `floor_raises_pushed_for_space`（不进 `publishes`：空发布不改用户可见状态），
    /// 没收下时报在 `AcceptRefused::floor_raises_pushed_by_this_accept`；连同观测 `floor_raises_pushed_by_accepts`、`empty_publishes_pushed_by_accepts` 与事件 [`MaintenanceEvent::FloorRaisesPushedByAnAccept`]。
    ///
    /// # Errors
    /// 核心层拒的原样；运行库坏了报 `Failed`。
    pub fn accept(
        &self,
        change: UserChange<'_>,
    ) -> Result<ChangeAccepted, RuntimeRefused<AcceptRefused>> {
        let now = self.shared.clock.reading();
        let mut locked = self
            .shared
            .lock_the_session()
            .map_err(RuntimeRefused::Failed)?;
        let SessionAndDevices { session, devices } = &mut *locked;
        let accepted = session.accept(devices, change, now);
        drop(locked);
        self.shared
            .note_the_floor_raises_pushed_by_an_accept(match &accepted {
                Ok(accepted) => &accepted.floor_raises_pushed_for_space.floor_raises,
                Err(refused) => refused.floor_raises_pushed_by_this_accept(),
            });
        accepted.map_err(RuntimeRefused::Refused)
    }

    /// fsync 走这里（`MountedSessionWithPendingChanges::publish_pending_now`）：进门先登记「有 fsync 在等」（取锁之前），整理在下一批之前让路，每次发布之间都放锁。
    /// 持锁时冻结着一次发布就先重发一次（D16（发布语义） 已定项 7）：重发成了照常排空待发，答成功（待发空着时答 `None`，那一次已落盘）；重发没成交回错误，照旧冻结着。
    ///
    /// # Errors
    /// 同 [`Self::accept`]；重发又报错时 `Refused` 里是 [`UserChangeRefused::Publish`]，`cause` 是重发报的错：那一次可能成了也可能没成（D16（发布语义） 已定项 17）。
    pub fn publish_pending_now(
        &self,
    ) -> Result<
        Option<PublishReportedByTheSession>,
        RuntimeRefused<PublishOfThePendingChangesRefused>,
    > {
        let _waiting = self.foreground_fsync_is_waiting();
        let now = self.shared.clock.reading();
        let mut locked = self
            .shared
            .lock_the_session()
            .map_err(RuntimeRefused::Failed)?;
        let SessionAndDevices { session, devices } = &mut *locked;
        let was_frozen = session.session().allocator.frozen_publish().is_some();
        if was_frozen {
            if let Err(cause) = session.resend_the_frozen_publish(devices) {
                drop(locked);
                self.shared.note_a_frozen_publish_resend_failed(&cause);
                return Err(RuntimeRefused::Refused(PublishOfThePendingChangesRefused {
                    refusal: Box::new(UserChangeRefused::Publish {
                        cause,
                        floor_raises: Vec::new(),
                        versions_published: Vec::new(),
                    }),
                }));
            }
        }
        let published = session.publish_pending_now(devices, now);
        drop(locked);
        if was_frozen {
            self.shared.note_a_frozen_publish_resent();
        }
        published.map_err(RuntimeRefused::Refused)
    }

    /// 单发一次用户改动（`MountedSessionWithPendingChanges::publish_user_change`）。
    ///
    /// # Errors
    /// 同 [`Self::accept`]。
    pub fn publish_user_change(
        &self,
        change: UserChange<'_>,
    ) -> Result<
        Vec<PublishReportedByTheSession>,
        RuntimeRefused<UserChangeAfterThePendingChangesRefused>,
    > {
        let now = self.shared.clock.reading();
        let mut locked = self
            .shared
            .lock_the_session()
            .map_err(RuntimeRefused::Failed)?;
        let SessionAndDevices { session, devices } = &mut *locked;
        session
            .publish_user_change(devices, change, now)
            .map_err(RuntimeRefused::Refused)
    }

    /// 挂着切换后台计算的设置，从下一批起生效（不排空待发）。开卡器不在这一条路上：新递一份实现的，`gpu_compute` 由调用方先开好递进来；
    /// 沿用这次挂载那一份而点到它没开着的卡的，切换照收，那一份实现在会话锁外开卡（产品 GPU 包，开不成隔一段时间重开），这一次调用与之后的批都不等它，
    /// 开卡的这段时间切换还没生效、照切换之前那一组跑，开完了在两批之间换上；重开到上限都没开成就在那时停 GC、报警
    /// （`singlefs_core::background_compute::BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`）。
    /// 切到 `cpu` 时那一份实现开着的卡全部关掉（`singlefs_core::background_compute::BackgroundGpuCompute::use_no_gpu_from_the_next_batch`）。
    ///
    /// # Errors
    /// 同 [`Self::accept`]。
    pub fn switch_background_compute(
        &self,
        settings: BackgroundComputeSettings,
        gpu_compute: Option<BackgroundGpuComputeHandle>,
        gc_cpu_compensation_threshold: Option<BackgroundGcCpuCompensationThreshold>,
    ) -> Result<(), RuntimeRefused<BackgroundComputeSettingsRefused>> {
        let mut locked = self
            .shared
            .lock_the_session()
            .map_err(RuntimeRefused::Failed)?;
        let switched = match gc_cpu_compensation_threshold {
            Some(threshold) => locked
                .session
                .switch_background_compute_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
                    settings,
                    gpu_compute,
                    threshold,
                ),
            None => locked
                .session
                .switch_background_compute_from_the_next_batch(settings, gpu_compute),
        };
        switched.map_err(RuntimeRefused::Refused)
    }

    /// 在锁里读或改会话与盘（读路径 `read_at` 要盘表，走这里；用例摆开关也走这里）。闭包里 panic 等于前台在锁里 panic：运行库从此报 `Failed`。
    ///
    /// # Errors
    /// 运行库坏了。
    pub fn with_the_session_and_devices<Answer>(
        &self,
        work: impl FnOnce(&mut SessionAndDevices<Device>) -> Answer,
    ) -> Result<Answer, RuntimeFailed> {
        let mut locked = self.shared.lock_the_session()?;
        Ok(work(&mut *locked))
    }

    /// 同步的一拍：线程体循环里那一次（重发冻结的那一次、`publish_when_due`、放锁的后台入口推到没得推）。不起线程的确定性测试直接调。
    ///
    /// # Errors
    /// 运行库坏了。
    pub fn run_one_maintenance_pass(&self) -> Result<MaintenancePass, RuntimeFailed> {
        self.shared.run_one_maintenance_pass()
    }

    /// 累计的观测。
    #[must_use]
    pub fn observations(&self) -> RuntimeObservations {
        self.shared
            .observations
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// 登记一个前台 fsync 在等，丢弃交回的那一份就减（[`Self::publish_pending_now`] 自己走这里；用例拨它钉整理让路）。
    pub fn foreground_fsync_is_waiting(&self) -> ForegroundFsyncWaiting<'_> {
        self.shared.fsync_waiters.fetch_add(1, Ordering::SeqCst);
        ForegroundFsyncWaiting {
            fsync_waiters: &self.shared.fsync_waiters,
        }
    }

    /// 停线程并等它结束（线程在一拍之间查 `stopping`，整理在下一批之前让路）。一批之内 GPU 还在等时要等到那一批出来。
    fn stop_and_join_the_maintenance_thread(&mut self) {
        if let Some(thread) = self.maintenance_thread.take() {
            self.shared.ask_the_maintenance_thread_to_stop();
            let _panicked_is_noted_by_the_thread_itself = thread.join();
        }
    }

    /// 卸载：停线程 → 重发冻结的那一次（没有冻结着的发布就什么都不写）→ 排空待发 → 走核心层的卸载 → 丢会话（GPU 把手随它丢，有没交完的提交的那张卡整张泄漏、记进开卡器那份账）→ 盘交回。
    /// 活意图不用做完：留在盘上，下次挂载续做。
    ///
    /// # Errors
    /// [`UnmountRefused`] 的四种。重发冻结的那一次又报错（[`UnmountRefused::FrozenPublishResendFailed`]）等价于崩溃：那一次可能成了也可能没成（D16（发布语义） 已定项 17），下一次挂载走恢复，读现行根判。
    pub fn unmount(mut self) -> Result<UnmountedPool<Device>, Box<UnmountRefused<Device>>> {
        self.stop_and_join_the_maintenance_thread();
        let failure = self.shared.failure_so_far();
        let shared = Arc::clone(&self.shared);
        drop(self);
        let Shared {
            parameters,
            runtime_parameters,
            locked,
            clock,
            receiver,
            background_gpu_opened_at_the_mount,
            ..
        } = Arc::try_unwrap(shared).unwrap_or_else(|_still_shared| {
            panic!("线程 join 过了、前台入口都借着 self：运行库是这一份的唯一持有者")
        });
        let locked_or_poisoned = locked.into_inner();
        let is_poisoned = locked_or_poisoned.is_err();
        let SessionAndDevices {
            mut session,
            mut devices,
        } = locked_or_poisoned.unwrap_or_else(PoisonError::into_inner);
        if failure.is_some() || is_poisoned {
            return Err(Box::new(UnmountRefused::Failed {
                devices,
                failure: failure.unwrap_or(RuntimeFailed {
                    description: "持着会话锁的线程 panic 过".to_string(),
                }),
            }));
        }
        if let Err(refusal) = session.resend_the_frozen_publish(&mut devices) {
            return Err(Box::new(UnmountRefused::FrozenPublishResendFailed {
                devices,
                refusal: Box::new(refusal),
            }));
        }
        let now = clock.reading();
        let (pending_changes_published, mut session) =
            match session.into_session_after_publishing_the_pending_changes(&mut devices, now) {
                Ok(drained_and_session) => drained_and_session,
                Err(refused) => {
                    let refused = *refused;
                    return Err(Box::new(UnmountRefused::PendingChangesNotPublished {
                        runtime: MountedRuntime::of_the_session(
                            parameters,
                            runtime_parameters,
                            SessionAndDevices {
                                session: refused.session_with_pending_changes,
                                devices,
                            },
                            clock,
                            receiver,
                            background_gpu_opened_at_the_mount,
                        ),
                        refusal: refused.refusal,
                    }));
                }
            };
        let shadow_ledger = session.shadow_ledger;
        match unmount(
            &parameters,
            &mut devices,
            &mut session.allocator,
            &mut session.current,
            shadow_ledger,
        ) {
            Ok(unmounted) => Ok(UnmountedPool {
                devices,
                pending_changes_published,
                unmounted,
                gpus_leaked_by_the_background_gpu: background_gpu_opened_at_the_mount
                    .and_then(|opened| opened.gpus_leaked),
            }),
            Err(refusal) => Err(Box::new(UnmountRefused::UnmountFailed { devices, refusal })),
        }
    }
}

impl<Device> Drop for MountedRuntime<Device> {
    /// 只停线程，不排空、不写盘（等价于崩溃）：析构里写盘失败无处可报。
    fn drop(&mut self) {
        if let Some(thread) = self.maintenance_thread.take() {
            self.shared.ask_the_maintenance_thread_to_stop();
            let _panicked_is_noted_by_the_thread_itself = thread.join();
        }
    }
}
