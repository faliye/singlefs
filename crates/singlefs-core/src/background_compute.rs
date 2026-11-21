//! 后台任务的计算放在哪（D26（后台整理与放置回收） 已定项 13 第 1–4 条）：一组挂载参数（`background_compute`、`background_gpu_devices`、
//! `background_gpu_memory_limit`、`background_gpu_wait_limit`）管全部后台任务，不按任务分设；挂载与挂着切换那一刻用同一套校验，配错、GPU 用不上就拒，不降级。
//! 它只活在内存里：不写进系统配置、不占 feature bit，核心层不带 GPU 也能编（D24（后台重活能不能卸给 GPU） 已定项 2 第 1 条）。
//!
//! GPU 那一份实现住单开的产品 GPU 包（`singlefs-background-gpu`），核心层不依赖它：核心层只定它要实现的 [`BackgroundGpuCompute`]
//! （GC 搬迁放上 GPU 的三段各一个方法，入出类型都住这里），挂载与挂着切换时由调用方把实现递进来（[`BackgroundGpuComputeHandle`]；
//! D26（后台整理与放置回收） 已定项 13「GPU 路径与 CPU 路径是两份实现」那一段「挂载时由调用方把它的实现递进去」）。选 `gpu` 而没给实现就是 GPU 不在，拒。
//! 选了 `gpu` 时整理搬迁的每一批三段经这份实现算，每批记下请求的后端、实际走的路与三段各自确认、不确定、交 CPU 判的条目数
//! （[`BackgroundComputeOfOneBatch`]；`.claude/rules/fs-design.md` 五条硬要求第 4 条：运行时看得出走了哪一条）。
//! 设置在每一批开头读一次，挂着切换从下一批开始生效、一批之内不换（五条硬要求第 5 条）。
//! 一批算着出错（第 4 条：实现交回出错，或只供测试的钩子注入）时那个后台任务停下，停着时收到一次通过校验的切换请求就当场解除停下
//! GC 停着时池子里有一块盘的已分配字节占它的容量超过挂载参数 `background_gc_cpu_compensation_threshold`（默认 85%），GC 照常起跑、每一批在 CPU 上补跑；
//! 每块盘都不超过就照旧停着（第 4 条：GC 报警停着之后任一块盘超过阈值就在 CPU 上补跑，[`WhetherTheTaskRunsTheNextBatch`]）。
//! 打包搬迁不补跑。
//! 挂载那一刻选 `gpu` 而实现照挂载那一组一张卡都用不上（参与的卡的显存上限连最小的一片都装不下、一张都不剩）时不拒挂载
//! 有 GPU 路径的后台任务当场记成出错停着（实现答换上了时带着的那个出错，[`LatestGpuSettingsHandedIn::PutInEffect`]）；
//! 挂着切到这样一组同样不拒切换，实现答换上了那一刻照挂载那一刻一样当场停着，报同一个成员。挂着切换点到没开的卡时实现在会话锁外开卡，
//! 开卡的这段时间切换还没生效：照旧照切换之前那一组跑（切换之前选 `cpu` 的就在 CPU 上算），实现答换上了才生效（[`BackgroundComputeOfThisMount`]）。
//! 切到 `cpu` 时告诉实现一张卡都不参与（[`BackgroundGpuCompute::use_no_gpu_from_the_next_batch`]），开着的卡全部关掉。
//! 调用方在停（运行库卸载、丢弃）时经把手置一个原子标记（[`BackgroundGpuComputeHandle::tell_the_implementation_the_caller_is_stopping`]），
//! 不取实现那把锁：锁外那一段此刻可能正借着它在等卡。实现建把手时收下这个标记（[`BackgroundGpuCompute::give_up_waiting_for_the_cards_once_the_caller_is_stopping`]），
//! 等卡的那几处读到它就放弃那一等（第 4 条「卸载时不等还在跑的提交」）；卸载没做成、运行库交回调用方接着用时经把手撤掉它
//! （[`BackgroundGpuComputeHandle::tell_the_implementation_the_caller_is_no_longer_stopping`]）。

use std::fmt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};
use std::time::Duration;

use crate::address::{DeviceIdentity, SlotNumber};
use crate::admission::BytesOnOneDevice;
use crate::allocator::{AllocationRecord, DeviceFreeMap, PoolAllocator};
use crate::compaction_plan::UnitAreaOfOneDevice;
use crate::logged_operations::CompactionRegion;
use crate::region_scan::{AllocationInRegion, CentralMappingEntry};

mod compute_of_this_mount;
mod gpu_compute_interface;
mod settings_admission;

pub use compute_of_this_mount::*;
pub use gpu_compute_interface::*;
pub use settings_admission::*;
#[cfg(test)]
use tests::pci_state_of_the_card_at_one;

/// 后台任务在哪算（挂载参数 `background_compute`，第 1 条）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundComputeBackend {
    Cpu,
    Gpu,
}

crate::enum_listing_every_member! {
/// 计算可以放在别处的后台任务种类（第 1 条「凡有 GPU 路径的后台任务」）。封闭集合：里程碑六的 GPU 打包搬迁
/// （`.claude/kb/milestone/06-sixth-txn.md`「六　GPU 打包搬迁」）加一个成员，挂载参数、校验与切换都不改。
/// 每一个成员的表（`BackgroundTask::ALL`，按声明的次序）由 `enum_listing_every_member!` 从这一张成员表生成：加成员不会漏进表
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundTask {
    /// 整理（GC）的搬迁：`mounted_session` 里整理每推一次发布算这一种任务的一批。
    CompactionRelocation,
}
}

impl BackgroundTask {
    /// 有没有 GPU 路径（第 1 条「凡有 GPU 路径的后台任务」）：有的那几种，挂载那一刻或切换生效时 GPU 一张卡都用不上就当场停着。
    #[must_use]
    pub const fn has_a_gpu_path(self) -> bool {
        match self {
            BackgroundTask::CompactionRelocation => true,
        }
    }
}

/// 一批实际在哪算、算了什么。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::large_enum_variant,
    reason = "一批一份的记账，不在热路径上：GPU 那一臂带着三段各自交了几片、用了哪几张卡，装箱只多一次分配，换不来什么"
)]
pub enum BackgroundComputePathTaken {
    Cpu(ReasonTheBatchRanOnTheCpu),
    /// 三段经递进来的 GPU 实现算（第 1 条），各段的条目数在里面（第 8 条）。
    Gpu(GpuWorkOfTheBatch),
}

/// 一批为什么在 CPU 上算。封闭集合：以后哪一种后台任务还没有 GPU 路径时，加「这种任务还没有 GPU 路径」那一个成员（第 1 条「没有 GPU 路径的照旧在 CPU 上算」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReasonTheBatchRanOnTheCpu {
    /// 设置选的就是 `cpu`。
    CpuRequestedByTheSettings,
    /// GC 因一批计算出错停着，这一批开头池子最满的那一块盘的已分配占它的容量超过阈值：在 CPU 上补跑。带那一判的读数
    /// （哪块盘、它的已分配字节与容量、阈值），设置照旧是停下时生效的那一组（请求的后端可以是 `gpu`）。
    CompensatingWhileStoppedAfterAComputeError(GcCpuCompensationReading),
}

/// 一批开头定下的走法（第 1 条）：判据只有任务种类与请求的后端，都在一批开头定下、一批之内不变。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundComputeRoute {
    Cpu(ReasonTheBatchRanOnTheCpu),
    Gpu,
}

/// 一种后台任务在请求的后端下走哪条路（第 1 条）。
#[must_use]
pub fn route_of_the_task(
    task: BackgroundTask,
    requested_backend: BackgroundComputeBackend,
) -> BackgroundComputeRoute {
    match (task, requested_backend) {
        (BackgroundTask::CompactionRelocation, BackgroundComputeBackend::Cpu) => {
            BackgroundComputeRoute::Cpu(ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings)
        }
        (BackgroundTask::CompactionRelocation, BackgroundComputeBackend::Gpu) => {
            BackgroundComputeRoute::Gpu
        }
    }
}

/// 一批后台任务的计算记账：哪种任务、请求的后端（这一批开头读到的设置）、实际走的路，与这一批算着出错放弃时报的错。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackgroundComputeOfOneBatch {
    pub task: BackgroundTask,
    pub requested_backend: BackgroundComputeBackend,
    pub path_taken: BackgroundComputePathTaken,
    /// 这一批算着出了错、放弃了（第 4 条「停下并报错」）：出在哪一段、哪张卡、什么原因。出错之前就挂着待生效的切换、停下当场解除时，
    /// 那个后台任务不记成停着（[`BackgroundComputeOfThisMount::stopped_after_a_compute_error`] 里没有它），错照样报在这里。没出错是 `None`。
    pub compute_error_that_abandoned_the_batch: Option<BackgroundComputeError>,
}

/// 一批开头读到的设置与定下的走法：一批结束时拿它核设置一批之内没换（[`BackgroundComputeOfThisMount::end_the_batch`]）。
pub(crate) struct BackgroundBatchUnderWay {
    settings_read_at_the_start_of_the_batch: AdmittedBackgroundComputeSettings,
    task: BackgroundTask,
    requested_backend: BackgroundComputeBackend,
    route: BackgroundComputeRoute,
}

impl BackgroundBatchUnderWay {
    /// 这一批走 GPU 时交回它用的那一份实现，走 CPU 时 `None`。
    ///
    /// # Panics
    /// 走 GPU 而这一组没带实现：选 `gpu` 的那一组在校验时就带上实现（[`admit_background_compute_settings_with_the_gpu_compute`]）。
    pub(crate) fn gpu_compute_of_the_route(&self) -> Option<&BackgroundGpuComputeHandle> {
        match self.route {
            BackgroundComputeRoute::Cpu(_) => None,
            BackgroundComputeRoute::Gpu => Some(
                self.settings_read_at_the_start_of_the_batch
                    .gpu_compute()
                    .expect("选 gpu 的那一组在校验那一刻就带上实现：没给实现照 GPU 不在拒了"),
            ),
        }
    }
}

/// 一批后台任务算着出的错（第 4 条「挂着的时候 GPU 出错」）。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackgroundComputeError {
    /// 只供测试的钩子注入的（[`BackgroundComputeOfThisMount::report_a_compute_error_in_the_batch_under_way_for_tests`]）。
    InjectedByTheTestOnlyHook,
    /// GPU 实现算 `segment` 那一段时交回出错。
    GpuSegmentFailed {
        segment: GpuComputedSegment,
        failure: BackgroundGpuComputeFailed,
    },
    /// GPU 实现那把锁中毒（实现上一次被借出时 panic 过，[`GpuImplementationPanickedWhileLent`]）：一批里借它算、两批之间交设置或问结局时借不出，
    /// 有 GPU 路径的后台任务照出错停下，等一次通过校验的切换（换一份实现）。不归哪张卡。
    GpuImplementationPanickedWhileLent,
    /// 挂载那一刻、或挂着切换时选 `gpu`，实现答交给它的那一组换上了而照它一张卡都用不上（[`LatestGpuSettingsHandedIn::PutInEffect`] 带着的出错），
    /// 或那一组点到的卡在会话锁外重开到上限都没开成（[`LatestGpuSettingsHandedIn::NotOpened`]，报的是
    /// [`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]；那一组不生效，生效的照旧是切换之前那一组）：
    /// 挂载照常做成、切换照收，有 GPU 路径的后台任务当场停着、报这个错，不等那一组下的头一批交回出错。
    GpuUnusableUnderTheSettingsHandedIn { failure: BackgroundGpuComputeFailed },
}

/// 一批算着出的错管到哪（[`BackgroundComputeError::reach_of_the_error`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReachOfTheComputeError {
    /// 这一批放弃，那个后台任务停下、每次叫起报警（第 4 条）。
    StopsTheTask,
    /// 只放弃这一批：后台任务不停、不报警，下一批照常算（一批里同一个条目键出现两次，调用方组批的 bug，不归哪张卡）。
    AbandonsOnlyTheBatch,
}

impl BackgroundComputeError {
    /// 借 GPU 实现时锁中毒那一个错（[`BackgroundGpuComputeHandle::lock`] 交回的转成这一批的错）。
    pub(crate) fn of_the_gpu_implementation_poisoned(
        _poisoned: GpuImplementationPanickedWhileLent,
    ) -> Self {
        BackgroundComputeError::GpuImplementationPanickedWhileLent
    }

    /// 这个错管到哪：实现交回一批里同一个条目键出现两次的只放弃这一批，别的都停下那个后台任务。
    #[must_use]
    pub fn reach_of_the_error(&self) -> ReachOfTheComputeError {
        match self {
            BackgroundComputeError::GpuSegmentFailed {
                failure: BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch { .. },
                ..
            } => ReachOfTheComputeError::AbandonsOnlyTheBatch,
            BackgroundComputeError::InjectedByTheTestOnlyHook
            | BackgroundComputeError::GpuImplementationPanickedWhileLent
            | BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn { .. }
            | BackgroundComputeError::GpuSegmentFailed {
                failure:
                    BackgroundGpuComputeFailed::GpuFailed { .. }
                    | BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit { .. }
                    | BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(..)
                    | BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings { .. }
                    | BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions { .. }
                    | BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit { .. }
                    | BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece { .. }
                    | BackgroundGpuComputeFailed::GaveUpWaitingForTheCallerIsStopping,
                ..
            } => ReachOfTheComputeError::StopsTheTask,
        }
    }

    /// 这个错点名的卡（[`BackgroundGpuComputeFailed::gpu_devices_named`]）；只供测试的钩子注入的不归哪张卡，是空。
    #[must_use]
    pub fn gpu_devices_named(&self) -> Vec<GpuDeviceIdentifier> {
        match self {
            BackgroundComputeError::InjectedByTheTestOnlyHook
            | BackgroundComputeError::GpuImplementationPanickedWhileLent => Vec::new(),
            BackgroundComputeError::GpuSegmentFailed { failure, .. }
            | BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn { failure } => {
                failure.gpu_devices_named()
            }
        }
    }

    /// 这个错带的、判那张卡用不了那一刻读的 PCI 状态（[`BackgroundGpuComputeFailed::pci_state_when_judged_unusable`]）；
    /// 只供测试的钩子注入的不归哪张卡，是没读。
    #[must_use]
    pub fn pci_state_when_judged_unusable(&self) -> PciStateOfTheGpuReadWhenJudgedUnusable {
        match self {
            BackgroundComputeError::InjectedByTheTestOnlyHook
            | BackgroundComputeError::GpuImplementationPanickedWhileLent => {
                PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError
            }
            BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn { failure }
            | BackgroundComputeError::GpuSegmentFailed { failure, .. } => {
                failure.pci_state_when_judged_unusable()
            }
        }
    }
}

/// 后台任务因一批出错停着时，每次叫起都交回的报警（D26（后台整理与放置回收） 已定项 13 第 4 条：「每次」是每次后台整理被叫起，通道是返回值里必须处理的报警）：
/// 哪个后台任务、停下的那一个错与它点名的卡、停了多久、停着期间在 CPU 上补跑了几批。停了多久不读挂钟，数叫起的次数：停下之后每次叫起结束时停着就加一。
/// 一批出错停下的那一趟叫起照报、不数这一趟（报 0），下一次叫起才是第 1 次；在叫起里停下与在可写挂载续做里停下（那里不是叫起）数法相同，
/// 都是停下之后头一次叫起报 1。
/// 挂载那一刻就停着的、挂着切换生效时因一张卡都用不上停着的，头一次叫起（切换生效的那一次）算第 1 次。
/// 这一次叫起在 CPU 上补跑了一批还是照旧停着、此刻 GPU 用不用得上都照报，直到一次通过校验的切换解除停下。
#[must_use = "后台任务因 GPU 出错停着：每次叫起都要把这份报警告诉文件系统的用户（第 4 条：通道是返回值里必须处理的报警）"]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlarmOfABackgroundTaskStoppedAfterAComputeError {
    pub task: BackgroundTask,
    /// 停下的那一个错点名的卡（[`BackgroundComputeError::gpu_devices_named`]），按错里的次序；不归哪张卡的是空。
    pub gpu_devices_named_by_the_error: Vec<GpuDeviceIdentifier>,
    pub error_that_stopped_the_task: BackgroundComputeError,
    /// 停下以来叫起了几次（含这一次；一批出错停下的那一趟不算，那一趟报 0）。
    pub wakes_while_stopped: u64,
    /// 停下以来在 CPU 上补跑的批数（[`BackgroundTaskStoppedAfterAComputeError::batches_compensated_on_the_cpu`]，含这一次叫起补跑的）。
    pub batches_compensated_on_the_cpu: u64,
}

impl AlarmOfABackgroundTaskStoppedAfterAComputeError {
    /// 停下的那一个错带的、判那张卡用不了那一刻读的 PCI 状态（[`BackgroundComputeError::pci_state_when_judged_unusable`]）：
    /// 每次叫起都是停下那一刻的同一份，不重读；不是那三种时刻、不归哪张卡的是没读。
    /// 从 `error_that_stopped_the_task` 现取、不另存一份：报警放在 `Option` 里、又放进挂载会话那几个拒绝成员，多存一份读数
    /// 就让装着它的 `Result` 过 clippy `result_large_err` 的线。
    #[must_use]
    pub fn pci_state_of_the_gpu_judged_unusable(&self) -> PciStateOfTheGpuReadWhenJudgedUnusable {
        self.error_that_stopped_the_task
            .pci_state_when_judged_unusable()
    }
}

/// 一种停着的后台任务停下以来叫起了几次（[`BackgroundComputeOfThisMount::alarm_at_the_end_of_a_wake`] 数的）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct WakesOfAStoppedTask {
    task: BackgroundTask,
    wakes_while_stopped: u64,
    /// 它是在此刻还没结束的那一次叫起里一批出错停下的：那一次叫起结束时不数（报 0），之后的叫起照数。
    is_stopped_in_the_wake_under_way: bool,
}

/// 此刻有没有一次叫起在做（[`BackgroundComputeOfThisMount::begin_a_wake`] 开、[`BackgroundComputeOfThisMount::alarm_at_the_end_of_a_wake`] 关）：
/// 一批出错停下时据它分「停在一次叫起里」与「停在可写挂载续做里」（续做不是叫起）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WakeOfTheBackgroundTasks {
    NoneUnderWay,
    UnderWay,
}

/// 有 GPU 路径的后台任务各一条停着（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]，补跑的批数 0），按 [`BackgroundTask::ALL`]
/// 的次序、由 [`BackgroundTask::has_a_gpu_path`] 现挑：加成员时不用改这里。
fn tasks_with_a_gpu_path_stopped_by(
    failure: &BackgroundGpuComputeFailed,
) -> Vec<BackgroundTaskStoppedAfterAComputeError> {
    tasks_with_a_gpu_path_stopped_by_the_error(
        &BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
            failure: failure.clone(),
        },
    )
}

/// 有 GPU 路径的后台任务各一条停着、报 `error`（补跑的批数 0），按 [`BackgroundTask::ALL`] 的次序：交设置、问结局时 GPU 实现那把锁中毒的那一刻用它
/// （[`BackgroundComputeError::GpuImplementationPanickedWhileLent`]）。
fn tasks_with_a_gpu_path_stopped_by_the_error(
    error: &BackgroundComputeError,
) -> Vec<BackgroundTaskStoppedAfterAComputeError> {
    BackgroundTask::ALL
        .into_iter()
        .filter(|task| task.has_a_gpu_path())
        .map(|task| BackgroundTaskStoppedAfterAComputeError {
            task,
            error: error.clone(),
            batches_compensated_on_the_cpu: 0,
        })
        .collect()
}

/// 实现答一组设置换上了（[`LatestGpuSettingsHandedIn::PutInEffect`]）之后停着的后台任务：照它一张卡都用不上，就是有 GPU 路径的每一种后台任务各一条停着
/// （[`tasks_with_a_gpu_path_stopped_by`]）；用得上是空。挂载那一刻与切换生效时共用这一份，两处停着报的是同一个成员。
fn tasks_stopped_when_no_gpu_is_usable_under_the_settings_put_in_effect(
    failure_under_the_settings: Option<&BackgroundGpuComputeFailed>,
) -> Vec<BackgroundTaskStoppedAfterAComputeError> {
    match failure_under_the_settings {
        None => Vec::new(),
        Some(failure) => tasks_with_a_gpu_path_stopped_by(failure),
    }
}

/// GC 停着时补跑那一判照什么判（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支都能被测试强制进入）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GcCpuCompensationJudgement {
    /// 产品：照最满的那一块盘的占用与阈值判。
    ByTheReading,
    /// 只供测试：从调这一刻起，停着的后台任务每一批开头都当占用不超过阈值判（读数照常读、照常报），
    /// 强制进入「一趟之中补跑过的一批之后，下一批开头不补跑了」那一格（[`BackgroundComputeOfThisMount::judge_the_gc_cpu_compensation_as_not_above_the_threshold_for_tests`]）。
    NotAboveTheThresholdForcedForTests,
}

/// 一种后台任务因一批出错停着（第 4 条：正在算的那一批放弃、意图留在盘上，那个后台任务停下并报错；GC 停着时占用超过阈值就在 CPU 上补跑）。
/// 运行时读得出它停着、报的是哪个错、停下以来在 CPU 上补跑了几批（每批的读数在那一批的记账里，
/// [`ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackgroundTaskStoppedAfterAComputeError {
    pub task: BackgroundTask,
    pub error: BackgroundComputeError,
    /// 停下以来在 CPU 上补跑的批数（每开一批补跑加一，不论那一批发没发成）；停下那一刻是 0，解除停下时连同这一条一起清掉。
    pub batches_compensated_on_the_cpu: u64,
}

/// 一种后台任务下一批跑不跑、怎么跑（第 4 条与）。判据在一批开头读一次、一批之内不变
/// （`.claude/rules/fs-design.md` 五条硬要求第 5 条）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WhetherTheTaskRunsTheNextBatch {
    /// 没停着：照生效的设置走（[`route_of_the_task`]）。
    AsTheSettingsSay,
    /// GC 停着、池子最满的那一块盘的已分配占它的容量超过阈值：这一批在 CPU 上补跑。
    OnTheCpuToCompensateWhileStopped(GcCpuCompensationReading),
    /// GC 停着、不超过阈值：不跑，照旧等一次通过校验的切换。
    StaysStopped(GcCpuCompensationReading),
}

/// 一批没开成：那个后台任务停着、这一批不补跑（[`WhetherTheTaskRunsTheNextBatch::StaysStopped`]）。状态一样都没动。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BatchNotBegunWhileTheTaskStaysStopped;

/// 一批出错之后那个后台任务怎样了（[`BackgroundComputeOfThisMount::stop_the_task_after_a_compute_error`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundTaskAfterTheComputeError {
    /// 停着，等一次通过校验的切换请求解除。
    LeftStopped,
    /// 停着：出错时交给了实现的那一组还在开卡。每次叫起报警、照阈值在 CPU 上补跑；实现答那一组换上了的那一刻解除，答开不成就照开不成停着；
    /// 一次通过校验的切换请求照样解除。
    LeftStoppedWhileTheSwitchOpensItsCards,
    /// 出错之前就挂着一次待生效、还没交给实现的切换：它在停下那一刻也算停着时收到的切换请求，当场解除停下，从下一批起用它。
    StopLiftedByTheSwitchRequestedBeforeTheError,
    /// 一批里同一个条目键出现两次（[`ReachOfTheComputeError::AbandonsOnlyTheBatch`]）：只放弃这一批，那个后台任务不停、不报警，下一批照常算。
    NotStoppedForOnlyTheBatchIsAbandoned,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 第 1 条「凡有 GPU 路径的后台任务」：GPU 一张卡都用不上时停着的，正好是每一个有 GPU 路径的成员各一条，报同一个错、补跑的批数 0。
    /// 逐个成员穷举：加成员时这里的 match 编不过，逼着写它停不停。
    #[test]
    fn every_background_task_with_a_gpu_path_stops_when_no_gpu_is_usable_and_no_other_does() {
        let failure = BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
            gpus_left_out: vec![GpuLeftOutUnderItsMemoryLimit {
                device: GpuDeviceIdentifier::of_opaque_bytes(b"0000:01:00.0".to_vec()),
                memory_limit: GpuMemoryInBytes(64),
                smallest_piece_bytes: GpuMemoryInBytes(972),
                bytes_already_held_on_the_device: GpuMemoryInBytes(0),
            }],
        };
        let stopped = tasks_with_a_gpu_path_stopped_by(&failure);
        for task in BackgroundTask::ALL {
            let stopped_for_the_task: Vec<&BackgroundTaskStoppedAfterAComputeError> = stopped
                .iter()
                .filter(|stopped_task| stopped_task.task == task)
                .collect();
            match task {
                BackgroundTask::CompactionRelocation => assert_eq!(
                    stopped_for_the_task,
                    vec![&BackgroundTaskStoppedAfterAComputeError {
                        task: BackgroundTask::CompactionRelocation,
                        error: BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
                            failure: failure.clone(),
                        },
                        batches_compensated_on_the_cpu: 0,
                    }],
                    "整理有 GPU 路径：停着一条"
                ),
            }
        }
        assert_eq!(
            stopped.len(),
            BackgroundTask::ALL
                .into_iter()
                .filter(|task| task.has_a_gpu_path())
                .count(),
            "没有 GPU 路径的一条都不停"
        );
    }

    /// 停下之后连着叫起三次，交回的报警里停下以来的叫起次数。
    fn wakes_while_stopped_of_three_wakes(
        background_compute: &mut BackgroundComputeOfThisMount,
    ) -> Vec<Option<u64>> {
        (0..3)
            .map(|_wake| {
                background_compute.begin_a_wake();
                background_compute
                    .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
                    .map(|alarm| alarm.wakes_while_stopped)
            })
            .collect()
    }

    /// 一次叫起里一批出错停下，那一趟照报、叫起次数报 0；
    /// 之后连着叫起三次报 1、2、3。
    // harness-test-granularity:one-scenario 同一份状态上按次序叫起几次，数的是停下以来的叫起次数，拆开就数不成
    #[test]
    fn stop_in_a_wake_is_reported_as_zero_wakes_and_the_next_three_wakes_count_one_two_three() {
        let mut background_compute =
            BackgroundComputeOfThisMount::of_a_mount(AdmittedBackgroundComputeSettings::PRODUCT);
        background_compute.begin_a_wake();
        assert_eq!(
            background_compute.stop_the_task_after_a_compute_error(
                BackgroundTask::CompactionRelocation,
                BackgroundComputeError::InjectedByTheTestOnlyHook,
            ),
            BackgroundTaskAfterTheComputeError::LeftStopped
        );
        assert_eq!(
            background_compute
                .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
                .map(|alarm| alarm.wakes_while_stopped),
            Some(0),
            "出错停下的那一趟照报、不数这一趟"
        );
        assert_eq!(
            wakes_while_stopped_of_three_wakes(&mut background_compute),
            vec![Some(1), Some(2), Some(3)],
            "下一次叫起才是第 1 次"
        );
    }

    /// 同一条：停在可写挂载续做里（不是叫起，没开过叫起），停下之后连着叫起三次报 1、2、3，与停在叫起里的数法相同。
    #[test]
    fn stop_in_the_writable_mount_continuation_counts_the_next_three_wakes_one_two_three() {
        let mut background_compute =
            BackgroundComputeOfThisMount::of_a_mount(AdmittedBackgroundComputeSettings::PRODUCT);
        assert_eq!(
            background_compute.stop_the_task_after_a_compute_error(
                BackgroundTask::CompactionRelocation,
                BackgroundComputeError::InjectedByTheTestOnlyHook,
            ),
            BackgroundTaskAfterTheComputeError::LeftStopped
        );
        assert_eq!(
            wakes_while_stopped_of_three_wakes(&mut background_compute),
            vec![Some(1), Some(2), Some(3)],
            "续做里停下之后头一次叫起就是第 1 次"
        );
    }

    /// 同一条：一次叫起结束（交过报警）之后、下一次叫起开头之前停下的不在叫起里（叫起结束时就关了），停下之后头一次叫起报 1，不报 0。
    #[test]
    fn stop_after_a_wake_has_ended_is_not_counted_as_stopped_in_that_wake() {
        let mut background_compute =
            BackgroundComputeOfThisMount::of_a_mount(AdmittedBackgroundComputeSettings::PRODUCT);
        background_compute.begin_a_wake();
        assert_eq!(
            background_compute.alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation),
            None,
            "没停着：不报警"
        );
        assert_eq!(
            background_compute.stop_the_task_after_a_compute_error(
                BackgroundTask::CompactionRelocation,
                BackgroundComputeError::InjectedByTheTestOnlyHook,
            ),
            BackgroundTaskAfterTheComputeError::LeftStopped
        );
        assert_eq!(
            wakes_while_stopped_of_three_wakes(&mut background_compute),
            vec![Some(1), Some(2), Some(3)],
            "叫起结束之后停下的：下一次叫起就是第 1 次"
        );
    }

    /// 一份读到的 PCI 状态：目录照 `device_directory_reading`、使能位照 `device_enable`；链路此刻降到 5.0 GT/s ×4（最大 32.0 GT/s ×16），
    /// 可纠正的出错计数不是 0，致命的那份不在、不致命的那份读不通。
    pub(super) fn pci_state_of_the_card_at_one(
        device_directory_reading: PciDeviceDirectoryReading,
        device_enable: PciFileReading<PciDeviceEnable>,
    ) -> PciStateOfTheGpu {
        PciStateOfTheGpu {
            pci_bus_address: "0000:01:00.0".to_string(),
            device_directory: PathBuf::from("/pci-device-directories-of-the-test/0000:01:00.0"),
            device_directory_reading,
            device_enable,
            current_link_speed: PciFileReading::Read("5.0 GT/s PCIe".to_string()),
            current_link_width: PciFileReading::Read("4".to_string()),
            maximum_link_speed: PciFileReading::Read("32.0 GT/s PCIe".to_string()),
            maximum_link_width: PciFileReading::Read("16".to_string()),
            advanced_error_reporting_correctable: PciFileReading::Read(
                "RxErr 3\nTOTAL_ERR_COR 3".to_string(),
            ),
            advanced_error_reporting_fatal: PciFileReading::FileNotPresent,
            advanced_error_reporting_nonfatal: PciFileReading::NotReadable {
                system_error: "替身：读不通".to_string(),
            },
        }
    }
}
