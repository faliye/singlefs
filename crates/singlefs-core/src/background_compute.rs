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
//! 每块盘都不超过就照旧停着（改第 4 条「不换回 CPU」，同日弹窗定「任一块盘超过就补跑」；[`WhetherTheTaskRunsTheNextBatch`]）。
//! 打包搬迁不补跑。
//! 挂载那一刻选 `gpu` 而实现照挂载那一组一张卡都用不上（参与的卡的显存上限连最小的一片都装不下、一张都不剩）时不拒挂载
//! 有 GPU 路径的后台任务当场记成出错停着（[`BackgroundGpuCompute::failure_under_the_settings_handed_in`]）；
//! 挂着切到这样一组同样不拒切换，切换生效的那一批开头照挂载那一刻一样读、一样当场停着，报同一个成员。

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use crate::address::{DeviceIdentity, SlotNumber};
use crate::admission::BytesOnOneDevice;
use crate::allocator::{AllocationRecord, DeviceFreeMap, PoolAllocator};
use crate::compaction_plan::UnitAreaOfOneDevice;
use crate::logged_operations::CompactionRegion;
use crate::region_scan::{AllocationInRegion, CentralMappingEntry};

/// 后台任务在哪算（挂载参数 `background_compute`，第 1 条）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundComputeBackend {
    Cpu,
    Gpu,
}

crate::enum_listing_every_member! {
/// 计算可以放在别处的后台任务种类（第 1 条「凡有 GPU 路径的后台任务」）。封闭集合：里程碑七的 GPU 打包搬迁
/// （`.claude/kb/milestone/07-seventh-txn.md`「三　GPU 打包搬迁」）加一个成员，挂载参数、校验与切换都不改。
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

/// 一张显卡的标识（`background_gpu_devices` 列表里的一项）：不透明的字节，只比相等。卡用什么认、怎么排是里程碑四设计轮（七·2）
/// 还挂着的岔路，这里不定格式：卡清单（[`GpuInventory`]）交出什么，调用方就点什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuDeviceIdentifier(Vec<u8>);

impl GpuDeviceIdentifier {
    #[must_use]
    pub fn of_opaque_bytes(opaque_bytes: Vec<u8>) -> Self {
        Self(opaque_bytes)
    }
}

/// 显存的字节数：一张卡的显存总量，或 `background_gpu_memory_limit` 里给一张卡的上限（第 3 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GpuMemoryInBytes(pub u64);

/// 本机有哪些显卡、每张多少显存，其中哪几张挂载那一刻没开成：只读。GPU 那一侧（住哪是七·2 的岔路）实现它，测试里一份假清单实现它。
/// 挂载与切换的校验只读它；没有这一份清单（`None`）就是 GPU 不在。
pub trait GpuInventory {
    /// 本机全部显卡与各自的显存总量，按「本机全部显卡的列表」的次序：没给 `background_gpu_devices` 时，`background_gpu_memory_limit`
    /// 按这个次序一张卡一个（第 3 条）。怎么排由实现这份清单的一方定（七·2 的岔路）。同一张卡只出现一次。挂载那一刻没开成的卡照列。
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)>;

    /// [`Self::devices_with_total_memory`] 里挂载那一刻没开成的那几张与原因（
    /// 给了 `background_gpu_devices` 而没点名的卡开不了，挂载跳过它、照挂）。参与的卡里有它就拒（第 4 条，挂载与切换同一判：
    /// [`BackgroundComputeSettingsRefused::ParticipatingGpuDeviceNotOpenedAtTheMount`]）。
    /// 不开卡的清单（测试里的假清单、开卡之前那一刻的枚举）没有这一样，交回空。
    /// 挂载那一刻定下、之后不变：答它不现查本机此刻在的卡（一次校验里只有 [`Self::devices_with_total_memory`] 现查一次，判与取参与的卡读的是同一份
    /// ），所以可以列着此刻已不在 [`Self::devices_with_total_memory`] 里的卡（挂载之后掉了的）；
    /// 校验只拿参与的卡去比，掉了的卡不参与。
    fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
        Vec::new()
    }
}

/// 卡清单里挂载那一刻没开成的一张卡（[`GpuInventory::devices_not_opened_at_the_mount`]）：原因同挂着时这张卡出错的报法。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuDeviceNotOpenedAtTheMount {
    pub device: GpuDeviceIdentifier,
    pub failure_at_the_mount: BackgroundGpuComputeFailed,
}

/// GC 搬迁放上 GPU 的三段（第 1 条）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuComputedSegment {
    /// 甲：扫描里的校验和核对——区域扫描读到的、判成可搬的单元，整单元 CRC-32C 与指着它的位置项里的比。
    UnitChecksumVerification,
    /// 乙：区域扫描里逐条记录的单元头解析与映射反查（钉住、隔离两判之后的那一段）。
    RegionRecordClassification,
    /// 丙：选段打分——每个聚簇段的活槽数。
    LiveSlotsPerClusterSegment,
}

/// 区域扫描里一条记录（连同它读到的单元）的条目键：甲、乙两段的两份核逐条比对时的条目身份，实现按它数连续不确定的轮数（第 8 条）。
/// R 在意图期内不作分配目的地（D26（后台整理与放置回收） 已定项 1 第 3 条），同一个键在两批之间指的是同一条记录。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RegionRecordEntryKey {
    pub device: DeviceIdentity,
    pub slot: SlotNumber,
    pub span_slots: u16,
}

impl RegionRecordEntryKey {
    #[must_use]
    pub fn of_the_record(record: &AllocationRecord) -> Self {
        Self {
            device: record.device,
            slot: record.slot,
            span_slots: record.span_slots,
        }
    }
}

/// 一个聚簇段的条目键：丙段按它数连续不确定的轮数（第 8 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClusterSegmentEntryKey {
    pub device: DeviceIdentity,
    pub start_slot: SlotNumber,
}

/// 甲段要核的一个单元：区域扫描读到的整单元字节，与指着这个落点的那条位置项里记的整单元 CRC-32C。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitChecksumToVerifyOnTheGpu<'unit> {
    pub entry_key: RegionRecordEntryKey,
    pub unit_bytes: &'unit [u8],
    pub expected_crc32c: u32,
}

/// 甲段一条的结局：算出的整单元 CRC-32C，与它对不对得上位置项。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitChecksumOutcome {
    pub computed_crc32c: u32,
    pub matches_the_expected_crc32c: bool,
}

/// 乙段要分类的一条记录：区域扫描会去读的那一种（R 那块盘上、未释放、起点在 R 里、不罩出 R、不钉住、不隔离、跨度是 1 或 2），
/// 与读单元那一步的结局（跨度 1 读 16 KiB、跨度 2 读 32 KiB；`None` 是读不到）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionRecordToClassifyOnTheGpu<'unit> {
    pub entry_key: RegionRecordEntryKey,
    pub record: AllocationRecord,
    pub unit_bytes: Option<&'unit [u8]>,
}

/// 一个条目这一轮的判定（第 8 条）。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GpuEntryDecision<Outcome> {
    /// 两份核结果相同：确认，交 CPU 照发布路径逐条核。
    ConfirmedByBothKernels(Outcome),
    /// 两份核结果不同：状态不确定，这一批不回收、不搬，留到下一轮重算。
    UncertainThisRound,
    /// 同一个条目连续 2 轮不确定：CPU 那一份实现单独判了一次，按它的判定处理。
    JudgedByTheCpuReference(Outcome),
}

impl<Outcome> GpuEntryDecision<Outcome> {
    /// 确认的与交 CPU 判的那一份结局；不确定的是 `None`。
    #[must_use]
    pub fn outcome(&self) -> Option<&Outcome> {
        match self {
            GpuEntryDecision::ConfirmedByBothKernels(outcome)
            | GpuEntryDecision::JudgedByTheCpuReference(outcome) => Some(outcome),
            GpuEntryDecision::UncertainThisRound => None,
        }
    }
}

/// 实现算一段时出的错（第 4 条「停下并报错」）。核心层对三种做的是同一件事：这一批放弃、意图留在盘上、那个后台任务停下并报这个错，不换回 CPU；
/// 成员按停下之后该怎么处置分（修好或不再点那张卡、调大等 GPU 的上限、切回 `cpu`），出在哪张卡、什么原因带在成员里。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackgroundGpuComputeFailed {
    /// 这张卡出错（起不了、设备丢了、等不成、回读坏了、核留下认不出的判定字）：修好它或切换时不再点它，或切回 `cpu`。
    GpuFailed {
        device: GpuDeviceIdentifier,
        cause: GpuFailureOnTheDevice,
    },
    /// 等这张卡超过了 `background_gpu_wait_limit`：调大上限，或查这张卡是不是卡死了。
    WaitForTheGpuExceededTheLimit {
        device: GpuDeviceIdentifier,
        wait_limit: Duration,
        submission_waited_for: SubmissionWaitedForPastTheLimit,
    },
    /// 这一批的输入 GPU 那一侧第一版不支持，一张卡都没交活：切回 `cpu`（装不进一段的那一种也可以调大显存上限）。
    InputNotSupportedOnTheGpu(InputNotSupportedOnTheGpu),
    /// 这张卡上一次提交等超过 `wait_limit` 之后，在同一张卡上重交了 `resubmissions` 次、次次等超过上限（「在间隔内重试三次，
    /// 如果gpu无法用就报警」「重试是把那一批重新交一次」）：这张卡用不了。修好它或切换时不再点它，或切回 `cpu`。
    /// `last_submission_waited_for` 是最后那一次等超过上限的是哪一次提交（重交之前先等前一次，那一次没等到也算一次）。
    GpuUnusableAfterResubmissions {
        device: GpuDeviceIdentifier,
        resubmissions: u32,
        wait_limit: Duration,
        last_submission_waited_for: SubmissionWaitedForPastTheLimit,
    },
    /// 这一组设置点的卡的显存上限（扣掉卡上已占的）连最小的一片都装不下，参与的卡一张都不剩（
    /// 「不能因为装不下就拒绝」）：开卡、换设置时按与输入无关的最小的一片判，每一批切段时按这一批的最小的一片判（一张卡这一批切不出就这一批不分给它、
    /// 活分给别的卡，全都切不出才是这一种）。调大显存上限、点别的卡，或切回 `cpu`。按参与的次序列出每张卡、给它的上限、至少要多少与已占的，至少一张。
    EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
        gpus_left_out: Vec<GpuLeftOutUnderItsMemoryLimit>,
    },
    /// 这一批参与的卡一张都不剩，其中至少一张是因为卡上此刻空闲的显存连这一批最小的一片都建不成（
    /// 给的上限之内建缓冲显存不足就一路切小重交，切到最小的一片都建不成，这一批就不用这张卡、它还没交成的活转给这一批别的参与的卡重交；
    /// 下一批照设置重新起）。腾出卡上的显存、点别的卡，或切回 `cpu`。`gpus_short_of_free_memory` 按不用的先后列出因空闲显存不用的卡，至少一张；
    /// `gpus_left_out_under_their_memory_limit` 是这一段切的时候因显存上限（扣掉已占的）不用的卡，可以是空。
    EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
        gpus_short_of_free_memory: Vec<GpuShortOfFreeMemoryForTheSmallestPiece>,
        gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
    },
}

impl BackgroundGpuComputeFailed {
    /// 这个错点名的卡，按错里的次序：出在一张卡上的是那一张；参与的卡一张都不剩的是每一张被晾下的（因空闲显存不用的在前）；
    /// 不归哪张卡的（映射 key 比核装得下的长）是空。
    #[must_use]
    pub fn gpu_devices_named(&self) -> Vec<GpuDeviceIdentifier> {
        match self {
            BackgroundGpuComputeFailed::GpuFailed { device, .. }
            | BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit { device, .. }
            | BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions { device, .. }
            | BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
                InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice { device, .. },
            ) => vec![device.clone()],
            BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
                InputNotSupportedOnTheGpu::MappingKeyLongerThanTheKernelsHold { .. },
            ) => Vec::new(),
            BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
                gpus_left_out,
            } => gpus_left_out
                .iter()
                .map(|left_out| left_out.device.clone())
                .collect(),
            BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
                gpus_short_of_free_memory,
                gpus_left_out_under_their_memory_limit,
            } => gpus_short_of_free_memory
                .iter()
                .map(|short| short.device.clone())
                .chain(
                    gpus_left_out_under_their_memory_limit
                        .iter()
                        .map(|left_out| left_out.device.clone()),
                )
                .collect(),
        }
    }
}

/// 一张卡这一批因空闲显存连最小的一片都建不成而不用（[`BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`] 带着）：
/// 哪张卡、它最后那一段建不成时那一段合占的字节数（再压到它的一半就切不出最小的一片了）、这一段最小的一片要多少字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuShortOfFreeMemoryForTheSmallestPiece {
    pub device: GpuDeviceIdentifier,
    pub bytes_of_the_last_chunk_not_created: u64,
    pub smallest_piece_bytes: u64,
}

/// 一张卡因显存上限装不下最小的一片而不用：哪张卡、给它的上限、最小的一片要多少字节，
/// 与判的那一刻卡上已经占着的字节（这个进程里之前几次挂载泄漏在这张卡上的，或等超过上限之后还在跑的那一次提交的；不用的判据是
/// 「给的上限扣掉已占的不到最小的一片」）（[`BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit`] 带着；
/// 产品 GPU 包开卡、换设置与每一批切段时报的也是它）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuLeftOutUnderItsMemoryLimit {
    pub device: GpuDeviceIdentifier,
    pub memory_limit: GpuMemoryInBytes,
    pub smallest_piece_bytes: u64,
    pub bytes_already_held_on_the_device: u64,
}

/// 一张卡出错的原因（[`BackgroundGpuComputeFailed::GpuFailed`] 带着）。`reason` 是 GPU 那一侧报的原话。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GpuFailureOnTheDevice {
    /// 起不了设备或编不了管线（驱动不对、显存不够）。
    DeviceOrPipelineDidNotStart { reason: String },
    /// GPU 那一侧报了没接住的错（校验错、显存不够）或设备丢了。
    UncapturedErrorOrDeviceLost { reason: String },
    /// 等一次提交没等成（不是超时）。
    WaitForTheSubmissionFailed { reason: String },
    /// 回读缓冲映射不成。
    ReadbackMappingFailed { reason: String },
    /// 读回的判定字认不得：核没写到那一条或写坏了。`entry_position` 是那一条在它那一次提交里的位置。
    KernelLeftAnUnrecognizedVerdictWord {
        entry_position: usize,
        verdict_word: u32,
    },
    /// 在卡上建一段的缓冲时显存不足（`bytes_held_by_the_chunk` 是那一段合占的字节数）。挂着时一批里遇到它按更小的片重切重交（第 3 条
    /// 「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」），切到最小的一片都建不成就这一批不用这张卡
    /// （[`BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`]），不报出来；报出来的是挂载那一刻量代价时遇到的。
    OutOfMemoryCreatingTheBuffers {
        bytes_held_by_the_chunk: u64,
        reason: String,
    },
}

/// 等超过上限的是哪一次提交（[`BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit`] 带着）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmissionWaitedForPastTheLimit {
    /// 这一批交给这张卡的那一次。
    SubmissionOfThisBatch,
    /// 这张卡上前一次等超过上限的那一次：这一批在这张卡上交活之前又等满了上限它还没做完（它的缓冲还占着卡上的显存，第 3 条）。
    EarlierTimedOutSubmissionStillRunning,
}

/// 第一版不支持的输入（[`BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu`] 带着）：条款只写了「一批放不下就切成几次交」，
/// 下面这几种切不小的怎么办没写。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputNotSupportedOnTheGpu {
    /// 分到活的那张卡这一批连最小的一片都装不下，而装不下的不是显存上限（给的上限扣掉卡上已占的连最小的一片都装不下的卡，这一批不分给它，
    /// 第 3 条；参与的卡全都这样才报 [`BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit`]）：是单个缓冲的绑定上限。
    /// `entry_position` 是要切的那一条在这一批里的位置；`input_bytes` 是那最小的一片（一个字节的输入、或乙段一个落点上的全部条目那一片，补到整字）
    /// 连同一段的固定占用合占的字节数，即这张卡至少要剩这么多（产品 GPU 包按这个数填）。
    EntryLargerThanAChunkOnTheDevice {
        device: GpuDeviceIdentifier,
        entry_position: usize,
        input_bytes: u64,
    },
    /// 映射里有一条 key 比核装得下的长：一张卡都还没分活就判出，不归哪张卡。`entry_position` 是它在映射条目里的位置。
    MappingKeyLongerThanTheKernelsHold {
        entry_position: usize,
        key_bytes: usize,
    },
}

/// 产品 GPU 包要实现的那一份：GC 搬迁放上 GPU 的三段各一个方法（第 1 条），两份独立的 GPU 核各算一次、逐条比，连续 2 轮不确定的条目交
/// CPU 那一份实现单独判（第 8 条）。连续不确定的计数随这份实现活：只在内存里，重挂换一份实现就清零。
/// 本机的卡清单也由它答（[`GpuInventory`]）：挂载与切换照它校验设置，没给实现就是 GPU 不在。
///
/// 三个方法的约定：交回的判定一条输入一条、按输入的次序；交 CPU 判的那一份就是核心层的 CPU 写法（甲 `checksum::crc32_castagnoli`，
/// 乙 `region_scan::scan_region_over_the_identity_index` 对单条记录，丙 `compaction_plan::live_slots_per_cluster_segment_by_the_records`）。
/// 出错时这一批的判定一条都不交、计数不动。条数对不上是实现的 bug，核心层断言。
pub trait BackgroundGpuCompute: GpuInventory {
    /// 甲：每个单元的整单元 CRC-32C 与 `expected_crc32c` 比。
    ///
    /// # Errors
    /// [`BackgroundGpuComputeFailed`]。
    fn verify_unit_checksums(
        &mut self,
        units: &[UnitChecksumToVerifyOnTheGpu<'_>],
    ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed>;

    /// 乙：`region` 里每条记录分一类，判法同核心层区域扫描在钉住、隔离两判之后那一段（读头 → 类标签 → key 与位置）。
    /// `mapping_entries` 是这一版的中央映射条目（区域扫描建按身份索引的那一份）。
    ///
    /// # Errors
    /// [`BackgroundGpuComputeFailed`]。
    fn classify_region_records(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassifyOnTheGpu<'_>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed>;

    /// 丙：`unit_area` 那块盘上每个整段的活槽数（未释放记录罩住的槽数），`segments` 是这几个整段按起点升序的条目键，一段一条。
    ///
    /// # Errors
    /// [`BackgroundGpuComputeFailed`]。
    fn count_live_slots_per_cluster_segment(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed>;

    /// 从下一批起照这一组设置跑（第 1–3 条）：只在 `settings` 点的那几张卡上、按卡的顺序填满分活，每张卡上合占的显存不超过
    /// 给它的上限，每一处等 GPU 不超过等的上限。核心层在挂载那一刻（选 `gpu` 时）与每次通过校验的切换生效的那一批开头、开算之前调，
    /// 一批里三段都算完之前不再调（一批之内不换）。实现不为它重开设备，三段的连续不确定计数照旧留着（第 8 条：只在重挂时清零）。
    /// `settings` 的卡都在这份实现答的卡清单里、都不是它报的挂载那一刻没开成的（核心层照这份清单校验过）。
    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings);

    /// 照最近一次交来的那一组设置（[`Self::use_the_settings_from_the_next_batch`]）一张卡都用不上时，那一组下每一批三段都会交回的那个出错
    /// （例如参与的卡的显存上限连最小的一片都装不下、一张都不剩）；用得上交回 `None`。核心层在挂载那一刻与每次切换生效的那一批开头把那一组交给实现之后
    /// 各读一次：不是 `None` 就不拒挂载、不拒切换，有 GPU 路径的后台任务当场记成出错停着
    /// （[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]），不等那一组下的头一批交回出错。只读，不交活。
    /// 不开卡的实现（测试里的假实现）没有这一样，交回 `None`。
    fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
        None
    }

    /// 上一次交回以来三段各自在卡上是怎么交的（交了几片、重交了几次、用了哪几张卡），交回之后清零。核心层每一批三段算完（含出错放弃）之后读一次，
    /// 记进这一批的记账（[`GpuWorkOfTheBatch::submissions_on_the_gpus`]）。不开卡的实现（测试里的假实现）没有这一样，交回什么都没交的那一份。
    fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
        GpuSubmissionsOfTheBatch::default()
    }
}

/// 一张参与的卡：它的标识、显存总量（卡清单答的），与给它的显存上限（`None` 是不设上限，第 3 条）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticipatingGpuDevice {
    pub device: GpuDeviceIdentifier,
    pub total_memory: GpuMemoryInBytes,
    pub memory_limit: Option<GpuMemoryInBytes>,
}

/// 校验过、选 `gpu` 的一组设置交给实现的样子（[`BackgroundGpuCompute::use_the_settings_from_the_next_batch`]）：参与的卡按上限对应的次序
/// （没点名是卡清单的次序，点了名是点名的次序）、各自的显存上限，与等 GPU 的上限。只由 [`admit_gpu_settings_against_the_inventory`] 造出：
/// 至少一张卡、同一张卡只出现一次、每张都在校验时的卡清单里且不是清单报的挂载那一刻没开成的、上限不超过那张卡的显存总量、等的上限不是 0。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedGpuSettings {
    participating_devices: Vec<ParticipatingGpuDevice>,
    wait_limit: Duration,
}

impl AdmittedGpuSettings {
    #[must_use]
    pub fn participating_devices(&self) -> &[ParticipatingGpuDevice] {
        &self.participating_devices
    }

    #[must_use]
    pub fn wait_limit(&self) -> Duration {
        self.wait_limit
    }
}

/// 调用方递进来的那一份 GPU 实现的把手：几处共用同一份（克隆的是把手，实现与它的连续不确定计数只有一份）。
/// 两份把手相等当且仅当指着同一份实现。
#[derive(Clone)]
pub struct BackgroundGpuComputeHandle {
    shared: Arc<Mutex<dyn BackgroundGpuCompute + Send>>,
}

impl BackgroundGpuComputeHandle {
    #[must_use]
    pub fn of_an_implementation(
        implementation: impl BackgroundGpuCompute + Send + 'static,
    ) -> Self {
        Self {
            shared: Arc::new(Mutex::new(implementation)),
        }
    }

    /// 调用方自己也留着一份的实现（测试读它记下的调用、改注入的故障）。
    #[must_use]
    pub fn of_a_shared_implementation(shared: Arc<Mutex<dyn BackgroundGpuCompute + Send>>) -> Self {
        Self { shared }
    }

    /// 借出实现：一批里三段都经这一次借出算。
    ///
    /// # Panics
    /// 实现上一次被借出时 panic 过（那一次的 panic 已经往上传过）：不带着可能半改的计数再用它。
    pub(crate) fn lock(&self) -> MutexGuard<'_, dyn BackgroundGpuCompute + Send + 'static> {
        self.shared
            .lock()
            .expect("GPU 实现上一次借出时 panic 过：那一次已经往上报过，不带着可能半改的连续不确定计数再用它")
    }
}

impl fmt::Debug for BackgroundGpuComputeHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BackgroundGpuComputeHandle")
            .finish_non_exhaustive()
    }
}

impl PartialEq for BackgroundGpuComputeHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.shared, &other.shared)
    }
}

impl Eq for BackgroundGpuComputeHandle {}

/// 一段里确认、不确定、交 CPU 判的条目各多少（第 8 条「每一批报确认、不确定、交 CPU 判的条目各多少」）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GpuEntryCounts {
    pub confirmed_by_both_kernels: u64,
    pub uncertain_this_round: u64,
    pub judged_by_the_cpu_reference: u64,
}

impl GpuEntryCounts {
    /// 把这一段交回的判定记进来。
    pub(crate) fn note_the_decisions<Outcome>(&mut self, decisions: &[GpuEntryDecision<Outcome>]) {
        for decision in decisions {
            match decision {
                GpuEntryDecision::ConfirmedByBothKernels(_) => self.confirmed_by_both_kernels += 1,
                GpuEntryDecision::UncertainThisRound => self.uncertain_this_round += 1,
                GpuEntryDecision::JudgedByTheCpuReference(_) => {
                    self.judged_by_the_cpu_reference += 1;
                }
            }
        }
    }
}

/// 一段在卡上是怎么交的（`.claude/rules/fs-design.md` 五条硬要求第 4 条：运行时看得出切没切片、重交了几次、用了哪几张卡）：
/// 交成的提交几次、这几次合起来交了几片（整条一片、切开的一条几片），等超过上限之后在同一张卡上重交了几次，建缓冲显存不足之后按更小的片重切了几次，
/// 分到活的是哪几张卡（按分活的次序，同一张卡只列一次）。一批里同一段算了几次（选段时逐段先扫）的，各次加起来。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GpuSubmissionsOfOneSegment {
    pub submissions_completed: u64,
    pub pieces_submitted: u64,
    pub resubmissions_after_timeouts: u64,
    pub recuts_after_running_out_of_memory: u64,
    pub devices_given_work: Vec<GpuDeviceIdentifier>,
}

impl GpuSubmissionsOfOneSegment {
    /// 把同一段后来算的那一次加进来：各数相加，卡照先后并起来、不重列。
    pub fn add(&mut self, later: &GpuSubmissionsOfOneSegment) {
        self.submissions_completed += later.submissions_completed;
        self.pieces_submitted += later.pieces_submitted;
        self.resubmissions_after_timeouts += later.resubmissions_after_timeouts;
        self.recuts_after_running_out_of_memory += later.recuts_after_running_out_of_memory;
        for device in &later.devices_given_work {
            if !self.devices_given_work.contains(device) {
                self.devices_given_work.push(device.clone());
            }
        }
    }
}

/// 一批三段各自在卡上是怎么交的（[`GpuSubmissionsOfOneSegment`]），由实现在一批算完之后交回（[`BackgroundGpuCompute::take_the_submissions_of_the_batch`]）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GpuSubmissionsOfTheBatch {
    pub unit_checksum_verification: GpuSubmissionsOfOneSegment,
    pub region_record_classification: GpuSubmissionsOfOneSegment,
    pub live_slots_per_cluster_segment: GpuSubmissionsOfOneSegment,
}

impl GpuSubmissionsOfTheBatch {
    /// `segment` 那一段的那一份。
    pub fn of_the_segment_mut(
        &mut self,
        segment: GpuComputedSegment,
    ) -> &mut GpuSubmissionsOfOneSegment {
        match segment {
            GpuComputedSegment::UnitChecksumVerification => &mut self.unit_checksum_verification,
            GpuComputedSegment::RegionRecordClassification => {
                &mut self.region_record_classification
            }
            GpuComputedSegment::LiveSlotsPerClusterSegment => {
                &mut self.live_slots_per_cluster_segment
            }
        }
    }
}

/// 一批在 GPU 上算了什么：三段各自的条目数，甲段判成整单元 CRC-32C 对不上的单元数，与三段各自在卡上是怎么交的。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GpuWorkOfTheBatch {
    pub unit_checksum_verification: GpuEntryCounts,
    pub region_record_classification: GpuEntryCounts,
    pub live_slots_per_cluster_segment: GpuEntryCounts,
    /// 甲段确认或交 CPU 判出整单元 CRC-32C 与位置项对不上的可搬单元数。它们照第 8 条交 CPU：发布路径照常逐条核计划、释放之前读盘核
    /// （那一核读出对不上的那一份隔离，D19（块指针的结构与宽度预算） 已定项 5），区域扫描不因它改分类，结局与 CPU 路径相同；这里只报。
    pub units_whose_checksum_mismatched_on_the_gpu: u64,
    /// 三段各自交了几片、重交了几次、用了哪几张卡（实现在这一批算完之后交回的，出错放弃的那一批是出错之前与出错那一段交的）。
    pub submissions_on_the_gpus: GpuSubmissionsOfTheBatch,
}

impl GpuWorkOfTheBatch {
    /// 三段合起来的条目数。
    #[must_use]
    pub fn entries_of_the_three_segments(&self) -> GpuEntryCounts {
        let segments = [
            self.unit_checksum_verification,
            self.region_record_classification,
            self.live_slots_per_cluster_segment,
        ];
        GpuEntryCounts {
            confirmed_by_both_kernels: segments
                .iter()
                .map(|counts| counts.confirmed_by_both_kernels)
                .sum(),
            uncertain_this_round: segments
                .iter()
                .map(|counts| counts.uncertain_this_round)
                .sum(),
            judged_by_the_cpu_reference: segments
                .iter()
                .map(|counts| counts.judged_by_the_cpu_reference)
                .sum(),
        }
    }
}

/// 一段交回的判定：出错就转成这一批的错（[`BackgroundComputeError::GpuSegmentFailed`]），否则记进 `counts` 再交回。
///
/// # Panics
/// 判定的条数与输入的条数不等：违反 [`BackgroundGpuCompute`] 的约定，是实现的 bug。
pub(crate) fn decisions_of_one_segment<Outcome>(
    segment: GpuComputedSegment,
    entries_handed_in: usize,
    returned: Result<Vec<GpuEntryDecision<Outcome>>, BackgroundGpuComputeFailed>,
    counts: &mut GpuEntryCounts,
) -> Result<Vec<GpuEntryDecision<Outcome>>, BackgroundComputeError> {
    let decisions = returned
        .map_err(|failure| BackgroundComputeError::GpuSegmentFailed { segment, failure })?;
    assert_eq!(
        decisions.len(),
        entries_handed_in,
        "GPU 实现一条输入交一条判定（BackgroundGpuCompute 的约定）：{segment:?} 段交进 {entries_handed_in} 条"
    );
    counts.note_the_decisions(&decisions);
    Ok(decisions)
}

/// 等 GPU 交回一批结果的上限的默认：10 秒。默认值是推的，待量：E185（GC一批搬迁的挂钟拆分）
/// 读数里，一批 CPU 18–338 ms（第 14 行）、GPU 往返让前台多等 5.50–14.76 ms（第 11 行），取比最慢的一批大一个半数量级的数：
/// 卡死的驱动等得出来，慢而活着的卡不被当成出错。
pub const BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT: Duration = Duration::from_secs(10);

/// 一组后台计算的挂载参数（第 1–3 条，加等 GPU 结果的上限）。每次挂载各给各的；挂着的时候整组换（[`BackgroundComputeOfThisMount::request_switch_from_the_next_batch`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackgroundComputeSettings {
    pub background_compute: BackgroundComputeBackend,
    /// `None` 是不给：用本机全部显卡（第 2 条）。
    pub background_gpu_devices: Option<Vec<GpuDeviceIdentifier>>,
    /// `None` 是不给：不设上限（第 3 条）。给了就按次序与参与的卡一张一个，不补齐。
    pub background_gpu_memory_limit: Option<Vec<GpuMemoryInBytes>>,
    /// 一批交给 GPU 之后至多等它多久（过了这个上限还没交回，照第 4 条「挂着的时候 GPU 出错」办，
    /// 这一批放弃、那个后台任务停下并报错）。只在选 `gpu` 时起作用；选 `cpu` 时照收、不用。不许是 0
    /// （[`BackgroundComputeSettingsRefused::GpuWaitLimitOfZero`]）。默认 [`BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT`]。
    pub background_gpu_wait_limit: Duration,
}

impl BackgroundComputeSettings {
    /// 产品默认：`cpu`，卡与上限都不给（第 1 条「默认 `cpu`」），等 GPU 的上限取默认。现有的可写挂载入口都等价于挂载时给这一组。
    pub const PRODUCT: BackgroundComputeSettings = BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Cpu,
        background_gpu_devices: None,
        background_gpu_memory_limit: None,
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    };
}

/// 挂载参数 `background_gc_cpu_compensation_threshold`（
/// 「对 挂载参数默认，但是提供给可调节的参数」）：GC 因一批计算出错停着时，池子里有一块盘的已分配字节占它的容量的百分比严格大于它
/// （同日弹窗定「任一块盘超过就补跑」），GC 照常起跑、这一批在 CPU 上补跑。
/// 与 [`BackgroundComputeSettings`] 同属一组挂载参数：挂载与挂着切换时一起给、一起校验、从下一批起一起生效（第 1、4 条）。
/// 取值 0 到 100（含两端，整百分点）：100 是停着就不补跑，0 是池子里有一个字节已分配就补跑；超过 100 挂载或切换那一刻拒
/// （[`BackgroundComputeSettingsRefused::GcCpuCompensationThresholdAboveTheWholeCapacity`]），选 `cpu` 时也判。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackgroundGcCpuCompensationThreshold {
    pub allocated_percent_of_the_capacity: u32,
}

/// [`BackgroundGcCpuCompensationThreshold`] 的默认：85%。
pub const BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT:
    BackgroundGcCpuCompensationThreshold = BackgroundGcCpuCompensationThreshold {
    allocated_percent_of_the_capacity: 85,
};

/// 百分比的满值：已分配不超过总容量，阈值超过它就永远越不过。
const WHOLE_CAPACITY_IN_PERCENT: u32 = 100;

/// 一块盘此刻的已分配字节与容量（补跑那一判读的两个量）：分配器给这块盘增量维护的两个计数——占着的槽数
/// （`allocator::DeviceFreeMap::allocated_slots`，含已释放、还在 defer 窗口里的，同空间准入式子里的「已分配」）与单元区槽数
/// （`allocator::DeviceFreeMap::unit_area_slots`，同式子里的「容量」）——各换成字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocatedBytesAndCapacityOfADevice {
    pub device: DeviceIdentity,
    pub allocated_bytes: u64,
    pub capacity_bytes: u64,
}

impl AllocatedBytesAndCapacityOfADevice {
    /// 池子里已分配占容量最满的那一块盘（任一块盘超过就补跑：最满的那一块超过阈值，就是有一块超过；
    /// 每块都不超过，最满的那一块也不超过）。逐盘比 `已分配 ÷ 容量`（交叉相乘在 u128 里，不取整），一样满的取分配器里排在前面的那一块；
    /// 容量是 0 的盘不参加比（它装不下一个槽，占比无从谈起），全是 0 的取头一块。不遍历槽，代价随盘数、不随盘容量
    /// （`.claude/rules/fs-design.md`「记账是事务的副产品」：起不起跑是运行时决策路径）。
    ///
    /// # Panics
    /// 分配器里一块盘都没有：可写挂载的池至少有一块盘。
    #[must_use]
    pub fn of_the_fullest_device_of_the_allocator(allocator: &PoolAllocator) -> Self {
        let of_the_device = |device_map: &DeviceFreeMap| AllocatedBytesAndCapacityOfADevice {
            device: device_map.device,
            allocated_bytes: BytesOnOneDevice::of_slots(device_map.allocated_slots()).0,
            capacity_bytes: BytesOnOneDevice::of_slots(device_map.unit_area_slots()).0,
        };
        let first_device = of_the_device(
            allocator
                .devices
                .first()
                .expect("可写挂载的池至少有一块盘：分配器照盘表一块盘一张空闲图"),
        );
        allocator
            .devices
            .iter()
            .map(of_the_device)
            .filter(|device| device.capacity_bytes != 0)
            .fold(
                None,
                |fullest_so_far: Option<Self>, device| match fullest_so_far {
                    Some(fullest) if !device.is_fuller_than(&fullest) => Some(fullest),
                    Some(_) | None => Some(device),
                },
            )
            .unwrap_or(first_device)
    }

    /// 这一块盘的已分配占容量严格大于 `other` 的：`已分配 × 对方容量 > 对方已分配 × 容量`。两块盘的容量都不是 0 时才有意义。
    fn is_fuller_than(&self, other: &Self) -> bool {
        u128::from(self.allocated_bytes) * u128::from(other.capacity_bytes)
            > u128::from(other.allocated_bytes) * u128::from(self.capacity_bytes)
    }
}

/// 补跑那一判的读数：池子此刻最满的那一块盘（哪块盘、它的已分配字节与容量），与那一刻生效的阈值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GcCpuCompensationReading {
    pub fullest_device: AllocatedBytesAndCapacityOfADevice,
    pub threshold: BackgroundGcCpuCompensationThreshold,
}

impl GcCpuCompensationReading {
    /// 最满的那一块盘的已分配占它的容量严格大于阈值：`已分配 × 100 > 容量 × 阈值百分比`，乘在 u128 里，不溢出、不取整。
    #[must_use]
    pub fn is_the_allocated_share_above_the_threshold(&self) -> bool {
        u128::from(self.fullest_device.allocated_bytes) * u128::from(WHOLE_CAPACITY_IN_PERCENT)
            > u128::from(self.fullest_device.capacity_bytes)
                * u128::from(self.threshold.allocated_percent_of_the_capacity)
    }
}

/// 一组设置在挂载或切换那一刻被拒（第 4 条：拒，不降级到 cpu）。成员按调用方要改哪一样分：第 4 条点名的四种配错
/// （GPU 不在、点名的卡不存在、上限个数与卡数不等、上限超过显存总量），条款没写、第一版不支持的两种点法（空卡列表、同一张卡点两次），
/// 等 GPU 结果的上限给了 0，与参与的卡里有一张挂载那一刻没开成；GC 在 CPU 上补跑的阈值超过 100%。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackgroundComputeSettingsRefused {
    /// 选了 `gpu` 而 GPU 不在：没有卡清单，或没点名卡而清单里一张卡都没有。改选 `cpu`，或装上 GPU。
    GpuNotPresent,
    /// `background_gpu_devices` 点名的这张卡不在卡清单里（点名的卡不存在）。
    NamedGpuDeviceNotInTheInventory { device: GpuDeviceIdentifier },
    /// `background_gpu_memory_limit` 给了而个数与参与的卡数不等（给了卡列表对它的列表、没给对本机全部显卡）。
    MemoryLimitCountDifferentFromTheParticipatingDevices {
        memory_limits_given: usize,
        participating_devices: usize,
    },
    /// 给这张卡的上限超过它的显存总量。
    MemoryLimitAboveTheTotalMemoryOfTheDevice {
        device: GpuDeviceIdentifier,
        memory_limit: GpuMemoryInBytes,
        total_memory: GpuMemoryInBytes,
    },
    /// `background_gpu_devices` 给了而一张卡都没点：第 2 条只写了「不给就用本机全部显卡」，给了空列表算什么条款没写，第一版不支持。
    EmptyGpuDeviceListWhoseMeaningIsUndecided,
    /// 同一张卡在 `background_gpu_devices` 里点了不止一次：第 3 条按次序一张卡一个上限，同一张卡两个上限怎么算条款没写，第一版不支持。
    GpuDeviceNamedMoreThanOnceWhoseMeaningIsUndecided { device: GpuDeviceIdentifier },
    /// 选了 `gpu` 而 `background_gpu_wait_limit` 是 0：一批交出去就算超时，一批都算不成。改成大于 0 的上限。
    GpuWaitLimitOfZero,
    /// 参与的卡里有一张挂载那一刻没开成（[`GpuInventory::devices_not_opened_at_the_mount`]）：给了卡列表时它被点了名，没给时本机全部显卡都参与。
    /// 第 4 条拒（同「GPU 不在、驱动不对」）：切换时不点它，或切回 `cpu`，或修好它重挂。
    ParticipatingGpuDeviceNotOpenedAtTheMount(GpuDeviceNotOpenedAtTheMount),
    /// `background_gc_cpu_compensation_threshold` 超过 100%：已分配不会超过总容量，这个阈值永远越不过。改成 0 到 100 之间。
    GcCpuCompensationThresholdAboveTheWholeCapacity {
        threshold: BackgroundGcCpuCompensationThreshold,
    },
}

/// 校验过的一组设置，连同它校验时用的那一份 GPU 实现与 GC 在 CPU 上补跑的阈值：只由
/// [`admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold`] 造出，
/// 产品默认另有 [`Self::PRODUCT`]。选 `gpu` 的那一组恒带着实现与交给实现的那一组（没给实现那一刻就拒了），一批照它算。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedBackgroundComputeSettings {
    settings: BackgroundComputeSettings,
    gpu_compute: Option<BackgroundGpuComputeHandle>,
    /// 选 `gpu` 时恒有：照实现答的卡清单校验出的参与的卡、上限与等的上限；选 `cpu` 时恒无。
    gpu_settings: Option<AdmittedGpuSettings>,
    /// 恒不超过 100%（校验那一刻判过）。
    gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
}

impl AdmittedBackgroundComputeSettings {
    /// [`BackgroundComputeSettings::PRODUCT`]：选 `cpu` 照收、不校验，不带实现；补跑的阈值取默认。
    pub const PRODUCT: AdmittedBackgroundComputeSettings = AdmittedBackgroundComputeSettings {
        settings: BackgroundComputeSettings::PRODUCT,
        gpu_compute: None,
        gpu_settings: None,
        gc_cpu_compensation_threshold: BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
    };

    #[must_use]
    pub fn settings(&self) -> &BackgroundComputeSettings {
        &self.settings
    }

    /// GC 停着时在 CPU 上补跑的阈值（挂载参数 `background_gc_cpu_compensation_threshold`）。
    #[must_use]
    pub fn gc_cpu_compensation_threshold(&self) -> BackgroundGcCpuCompensationThreshold {
        self.gc_cpu_compensation_threshold
    }

    /// 选 `gpu` 时交给实现的那一组（[`BackgroundGpuCompute::use_the_settings_from_the_next_batch`]）；选 `cpu` 时 `None`。
    #[must_use]
    pub fn gpu_settings(&self) -> Option<&AdmittedGpuSettings> {
        self.gpu_settings.as_ref()
    }

    /// 选 `gpu` 时把这一组交给它带着的实现，从下一批起照它跑；选 `cpu` 时什么都不做（实现这时不算活，切到 `gpu` 那一批再交）。
    ///
    /// # Panics
    /// 选 `gpu` 而没带实现：选 `gpu` 的那一组在校验那一刻就带上实现。
    fn hand_the_gpu_settings_to_the_implementation(&self) {
        let Some(gpu_settings) = &self.gpu_settings else {
            return;
        };
        self.gpu_compute
            .as_ref()
            .expect("选 gpu 的那一组在校验那一刻就带上实现：没给实现照 GPU 不在拒了")
            .lock()
            .use_the_settings_from_the_next_batch(gpu_settings);
    }

    /// 选 `gpu` 时问它带着的实现：照交给它的这一组是不是一张卡都用不上（[`BackgroundGpuCompute::failure_under_the_settings_handed_in`]），
    /// 在 [`Self::hand_the_gpu_settings_to_the_implementation`] 之后调；选 `cpu` 时 `None`（实现这时不算活）。
    ///
    /// # Panics
    /// 选 `gpu` 而没带实现：选 `gpu` 的那一组在校验那一刻就带上实现。
    fn failure_of_the_implementation_under_the_gpu_settings(
        &self,
    ) -> Option<BackgroundGpuComputeFailed> {
        self.gpu_settings.as_ref()?;
        self.gpu_compute
            .as_ref()
            .expect("选 gpu 的那一组在校验那一刻就带上实现：没给实现照 GPU 不在拒了")
            .lock()
            .failure_under_the_settings_handed_in()
    }

    /// 这一组带着的 GPU 实现：选 `gpu` 时恒有；选 `cpu` 时是挂载或切换时递进来的那一份（以后切到 `gpu` 没另给实现时用它），没给是 `None`。
    #[must_use]
    pub fn gpu_compute(&self) -> Option<&BackgroundGpuComputeHandle> {
        self.gpu_compute.as_ref()
    }
}

/// 同 [`admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold`]，GC 在 CPU 上补跑的阈值取默认
/// [`BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT`]（不给就是默认）。
///
/// # Errors
/// [`BackgroundComputeSettingsRefused`] 除阈值越界之外的八种。
pub fn admit_background_compute_settings_with_the_gpu_compute(
    settings: BackgroundComputeSettings,
    gpu_compute: Option<BackgroundGpuComputeHandle>,
) -> Result<AdmittedBackgroundComputeSettings, BackgroundComputeSettingsRefused> {
    admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold(
        settings,
        gpu_compute,
        BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
    )
}

/// 挂载与挂着切换那一刻的校验（第 4 条）：先判 GC 在 CPU 上补跑的阈值不超过 100%（选 `cpu` 时也判：它与卡、上限不同，停着那一判不看后端），
/// 再判选 `gpu` 而没给实现就是 GPU 不在，拒；给了就照实现答的卡清单走 [`admit_background_compute_settings`]。
/// 选 `cpu` 时卡与上限照收、不校验，给了实现也带着。只读卡清单，不碰盘。
///
/// # Errors
/// [`BackgroundComputeSettingsRefused`] 的九种。
pub fn admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold(
    settings: BackgroundComputeSettings,
    gpu_compute: Option<BackgroundGpuComputeHandle>,
    gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
) -> Result<AdmittedBackgroundComputeSettings, BackgroundComputeSettingsRefused> {
    if gc_cpu_compensation_threshold.allocated_percent_of_the_capacity > WHOLE_CAPACITY_IN_PERCENT {
        return Err(
            BackgroundComputeSettingsRefused::GcCpuCompensationThresholdAboveTheWholeCapacity {
                threshold: gc_cpu_compensation_threshold,
            },
        );
    }
    let gpu_settings = match settings.background_compute {
        BackgroundComputeBackend::Cpu => None,
        BackgroundComputeBackend::Gpu => {
            let Some(gpu_compute) = &gpu_compute else {
                return Err(BackgroundComputeSettingsRefused::GpuNotPresent);
            };
            let implementation = gpu_compute.lock();
            let gpu_inventory: &dyn GpuInventory = &*implementation;
            Some(admit_gpu_settings_against_the_inventory(
                &settings,
                gpu_inventory,
            )?)
        }
    };
    Ok(AdmittedBackgroundComputeSettings {
        settings,
        gpu_compute,
        gpu_settings,
        gc_cpu_compensation_threshold,
    })
}

/// 第 4 条选 `gpu` 那一支的校验（`settings.background_compute` 不看，照选了 `gpu` 判，判法就是 [`admit_background_compute_settings`] 选 `gpu` 那一支），
/// 过了交回交给实现的那一组：参与的卡按上限对应的次序、各带上限，与等的上限。卡清单只读一次，判与取参与的卡读的是同一份。只读卡清单，不碰盘。
///
/// # Errors
/// [`BackgroundComputeSettingsRefused`] 除阈值越界之外的八种。
pub fn admit_gpu_settings_against_the_inventory(
    settings: &BackgroundComputeSettings,
    gpu_inventory: &dyn GpuInventory,
) -> Result<AdmittedGpuSettings, BackgroundComputeSettingsRefused> {
    let inventory_read_once = GpuInventoryReadOnce {
        devices_with_total_memory: gpu_inventory.devices_with_total_memory(),
        devices_not_opened_at_the_mount: gpu_inventory.devices_not_opened_at_the_mount(),
    };
    admit_background_compute_settings(
        BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: settings.background_gpu_devices.clone(),
            background_gpu_memory_limit: settings.background_gpu_memory_limit.clone(),
            background_gpu_wait_limit: settings.background_gpu_wait_limit,
        },
        Some(&inventory_read_once),
    )?;
    let participating_devices = participating_gpu_devices(
        settings.background_gpu_devices.as_deref(),
        &inventory_read_once.devices_with_total_memory,
    )
    .expect("上面的校验照同一份清单判过点名的卡：不空、不重、都在清单里");
    let memory_limits: Vec<Option<GpuMemoryInBytes>> = match &settings.background_gpu_memory_limit {
        None => vec![None; participating_devices.len()],
        Some(memory_limits) => memory_limits.iter().copied().map(Some).collect(),
    };
    assert_eq!(
        memory_limits.len(),
        participating_devices.len(),
        "上面的校验判过上限个数与参与的卡数相等"
    );
    Ok(AdmittedGpuSettings {
        participating_devices: participating_devices
            .into_iter()
            .zip(memory_limits)
            .map(
                |((device, total_memory), memory_limit)| ParticipatingGpuDevice {
                    device,
                    total_memory,
                    memory_limit,
                },
            )
            .collect(),
        wait_limit: settings.background_gpu_wait_limit,
    })
}

/// 读过一次的卡清单：判与取参与的卡都读这一份，不读第二次（实现答的清单两次之间可能变）。
struct GpuInventoryReadOnce {
    devices_with_total_memory: Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)>,
    devices_not_opened_at_the_mount: Vec<GpuDeviceNotOpenedAtTheMount>,
}

impl GpuInventory for GpuInventoryReadOnce {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        self.devices_with_total_memory.clone()
    }

    fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
        self.devices_not_opened_at_the_mount.clone()
    }
}

/// 第 4 条的校验：选 `cpu` 时卡、显存上限与等 GPU 的上限都照收、不校验（切到 `gpu` 那一刻再校验）；选 `gpu` 时，依次判 GPU 在不在、
/// 点名的卡在不在清单里、上限个数与参与的卡数等不等、每个上限超没超那张卡的显存总量、等 GPU 的上限是不是 0、参与的卡里有没有挂载那一刻没开成的，
/// 有一样不对就拒。最后那一判排在最后，同产品 GPU 包挂载那一刻的次序：先判设置、再开卡，点名的卡开不了才拒。
/// 只读卡清单，不碰盘。只校验：交回的是交进来的那一组，不带实现、装不进挂载；挂载与切换经
/// [`admit_background_compute_settings_with_the_gpu_compute`]（卡清单由实现答），产品 GPU 包开包时用这一份。
///
/// # Errors
/// [`BackgroundComputeSettingsRefused`] 除阈值越界之外的八种。
pub fn admit_background_compute_settings(
    settings: BackgroundComputeSettings,
    gpu_inventory: Option<&dyn GpuInventory>,
) -> Result<BackgroundComputeSettings, BackgroundComputeSettingsRefused> {
    match settings.background_compute {
        BackgroundComputeBackend::Cpu => {
            return Ok(settings);
        }
        BackgroundComputeBackend::Gpu => {}
    }
    let Some(gpu_inventory) = gpu_inventory else {
        return Err(BackgroundComputeSettingsRefused::GpuNotPresent);
    };
    let devices_on_this_host = gpu_inventory.devices_with_total_memory();
    let participating_devices = participating_gpu_devices(
        settings.background_gpu_devices.as_deref(),
        &devices_on_this_host,
    )?;
    if participating_devices.is_empty() {
        return Err(BackgroundComputeSettingsRefused::GpuNotPresent);
    }
    if let Some(memory_limits) = &settings.background_gpu_memory_limit {
        if memory_limits.len() != participating_devices.len() {
            return Err(
                BackgroundComputeSettingsRefused::MemoryLimitCountDifferentFromTheParticipatingDevices {
                    memory_limits_given: memory_limits.len(),
                    participating_devices: participating_devices.len(),
                },
            );
        }
        for ((device, total_memory), memory_limit) in
            participating_devices.iter().zip(memory_limits)
        {
            if memory_limit > total_memory {
                return Err(
                    BackgroundComputeSettingsRefused::MemoryLimitAboveTheTotalMemoryOfTheDevice {
                        device: device.clone(),
                        memory_limit: *memory_limit,
                        total_memory: *total_memory,
                    },
                );
            }
        }
    }
    if settings.background_gpu_wait_limit.is_zero() {
        return Err(BackgroundComputeSettingsRefused::GpuWaitLimitOfZero);
    }
    let devices_not_opened_at_the_mount = gpu_inventory.devices_not_opened_at_the_mount();
    for (participating_device, _total_memory) in &participating_devices {
        if let Some(not_opened) = devices_not_opened_at_the_mount
            .iter()
            .find(|not_opened| not_opened.device == *participating_device)
        {
            return Err(
                BackgroundComputeSettingsRefused::ParticipatingGpuDeviceNotOpenedAtTheMount(
                    not_opened.clone(),
                ),
            );
        }
    }
    Ok(settings)
}

/// 参与的卡与各自的显存总量，按上限对应的次序：没点名就是清单里的全部，点了名就是点名的那几张、按点名的次序。
///
/// # Errors
/// 点名的空列表、同一张卡点了两次、点名的卡不在清单里。
fn participating_gpu_devices(
    named_devices: Option<&[GpuDeviceIdentifier]>,
    devices_on_this_host: &[(GpuDeviceIdentifier, GpuMemoryInBytes)],
) -> Result<Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)>, BackgroundComputeSettingsRefused> {
    let Some(named_devices) = named_devices else {
        return Ok(devices_on_this_host.to_vec());
    };
    if named_devices.is_empty() {
        return Err(BackgroundComputeSettingsRefused::EmptyGpuDeviceListWhoseMeaningIsUndecided);
    }
    let mut participating_devices = Vec::with_capacity(named_devices.len());
    for (position_in_the_list, named_device) in named_devices.iter().enumerate() {
        if named_devices[..position_in_the_list].contains(named_device) {
            return Err(
                BackgroundComputeSettingsRefused::GpuDeviceNamedMoreThanOnceWhoseMeaningIsUndecided {
                    device: named_device.clone(),
                },
            );
        }
        let Some(device_on_this_host) = devices_on_this_host
            .iter()
            .find(|(device, _)| device == named_device)
        else {
            return Err(
                BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory {
                    device: named_device.clone(),
                },
            );
        };
        participating_devices.push(device_on_this_host.clone());
    }
    Ok(participating_devices)
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
    /// 挂载那一刻、或挂着切换生效的那一批开头选 `gpu`，实现照交给它的那一组一张卡都用不上（[`BackgroundGpuCompute::failure_under_the_settings_handed_in`]）：
    /// 挂载照常做成、切换照收，有 GPU 路径的后台任务当场停着、报这个错，不等那一组下的头一批交回出错。
    GpuUnusableUnderTheSettingsHandedIn { failure: BackgroundGpuComputeFailed },
}

impl BackgroundComputeError {
    /// 这个错点名的卡（[`BackgroundGpuComputeFailed::gpu_devices_named`]）；只供测试的钩子注入的不归哪张卡，是空。
    #[must_use]
    pub fn gpu_devices_named(&self) -> Vec<GpuDeviceIdentifier> {
        match self {
            BackgroundComputeError::InjectedByTheTestOnlyHook => Vec::new(),
            BackgroundComputeError::GpuSegmentFailed { failure, .. }
            | BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn { failure } => {
                failure.gpu_devices_named()
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
#[must_use = "后台任务因 GPU 出错停着：每次叫起都要把这份报警告诉文件系统的用户（用户 2026-10-30 定）"]
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
    BackgroundTask::ALL
        .into_iter()
        .filter(|task| task.has_a_gpu_path())
        .map(|task| BackgroundTaskStoppedAfterAComputeError {
            task,
            error: BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
                failure: failure.clone(),
            },
            batches_compensated_on_the_cpu: 0,
        })
        .collect()
}

/// 把 `settings_handed_in` 交给它带着的实现之后读一次（[`BackgroundGpuCompute::failure_under_the_settings_handed_in`]）：实现照这一组一张卡都用不上，
/// 就交回有 GPU 路径的每一种后台任务各一条停着（[`tasks_with_a_gpu_path_stopped_by`]）；用得上、或选 `cpu`
/// 交回空。挂载那一刻与切换生效时共用这一份，两处停着报的是同一个成员。
fn tasks_stopped_when_no_gpu_is_usable_under_the_settings_handed_in(
    settings_handed_in: &AdmittedBackgroundComputeSettings,
) -> Vec<BackgroundTaskStoppedAfterAComputeError> {
    match settings_handed_in.failure_of_the_implementation_under_the_gpu_settings() {
        None => Vec::new(),
        Some(failure) => tasks_with_a_gpu_path_stopped_by(&failure),
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
    /// 出错之前就挂着一次待生效的切换：它在停下那一刻也算停着时收到的切换请求，当场解除停下，从下一批起用它。
    StopLiftedByTheSwitchRequestedBeforeTheError,
}

/// 后台计算在这一次挂载里的状态：此刻生效的设置，与挂着时请求、从下一批开头起用的那一组；因一批出错停着的后台任务。
/// 生命周期一次挂载，只住内存。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackgroundComputeOfThisMount {
    in_effect: AdmittedBackgroundComputeSettings,
    requested_from_the_next_batch: Option<AdmittedBackgroundComputeSettings>,
    stopped_after_a_compute_error: Vec<BackgroundTaskStoppedAfterAComputeError>,
    /// 停着的后台任务停下以来叫起了几次：头一次叫起时记下（一次叫起里一批出错停下的，停下那一刻就记下 0），停着的那几条换掉或清掉时一起清掉
    /// （[`Self::replace_the_tasks_stopped`]）。
    /// 一种后台任务只经换掉、清掉不再停着，所以这里没有不停着的任务的那一条。
    wakes_of_the_tasks_stopped: Vec<WakesOfAStoppedTask>,
    /// 此刻有没有一次叫起在做：挂载那一刻与可写挂载续做时没有。
    wake_under_way: WakeOfTheBackgroundTasks,
    /// 只供测试的钩子在这一批里注入的错，这一批读完它就清掉（[`Self::take_the_compute_error_of_the_batch_under_way`]）。
    compute_error_of_the_batch_under_way: Option<BackgroundComputeError>,
    /// 补跑那一判照什么判：产品恒是照读数判，只供测试的开关能改。
    gc_cpu_compensation_judgement: GcCpuCompensationJudgement,
}

impl BackgroundComputeOfThisMount {
    /// 挂载那一刻：挂载给的那一组（校验过）生效，没有待生效的。选 `gpu` 时当场把这一组交给它带着的实现
    /// （[`BackgroundGpuCompute::use_the_settings_from_the_next_batch`]），头一批起照它跑；实现照这一组一张卡都用不上
    /// （[`BackgroundGpuCompute::failure_under_the_settings_handed_in`]）时，有 GPU 路径的后台任务当场记成出错停着
    /// （[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]，补跑的批数 0），挂载照常做成；别的情形没有停着的后台任务。
    #[must_use]
    pub fn of_a_mount(admitted_at_the_mount: AdmittedBackgroundComputeSettings) -> Self {
        admitted_at_the_mount.hand_the_gpu_settings_to_the_implementation();
        let stopped_after_a_compute_error =
            tasks_stopped_when_no_gpu_is_usable_under_the_settings_handed_in(
                &admitted_at_the_mount,
            );
        Self {
            in_effect: admitted_at_the_mount,
            requested_from_the_next_batch: None,
            stopped_after_a_compute_error,
            wakes_of_the_tasks_stopped: Vec::new(),
            wake_under_way: WakeOfTheBackgroundTasks::NoneUnderWay,
            compute_error_of_the_batch_under_way: None,
            gc_cpu_compensation_judgement: GcCpuCompensationJudgement::ByTheReading,
        }
    }

    /// 只供测试（`.claude/rules/fs-design.md`「五条硬要求」第 2 条）：从这一刻起，停着的后台任务每一批开头的补跑那一判都当占用不超过阈值
    /// （[`WhetherTheTaskRunsTheNextBatch::StaysStopped`]，读数照常读），一直到这次挂载结束。在一批读完设置、开算之前的钩子里调，
    /// 强制进入「这一批在 CPU 上补跑了，下一批开头不补跑」那一格：下一批不开、撤 R 排他，停在先前出错停着上。没停着的后台任务不受影响。
    pub fn judge_the_gc_cpu_compensation_as_not_above_the_threshold_for_tests(&mut self) {
        self.gc_cpu_compensation_judgement =
            GcCpuCompensationJudgement::NotAboveTheThresholdForcedForTests;
    }

    /// 因一批出错停着的后台任务（第 4 条），按停下的先后。
    #[must_use]
    pub fn stopped_after_a_compute_error(&self) -> &[BackgroundTaskStoppedAfterAComputeError] {
        &self.stopped_after_a_compute_error
    }

    /// `task` 此刻因一批出错停着。
    #[must_use]
    pub fn is_stopped_after_a_compute_error(&self, task: BackgroundTask) -> bool {
        self.stopped_after_a_compute_error
            .iter()
            .any(|stopped| stopped.task == task)
    }

    /// 只供测试（`.claude/rules/fs-design.md`「五条硬要求」第 2 条）：在一批读完设置、开算之前的钩子里调，让这一批照「算着出了错」走
    /// （第 4 条那一支；产品路径上 GPU 实现交回出错时也走它，[`BackgroundComputeError::GpuSegmentFailed`]）。
    pub fn report_a_compute_error_in_the_batch_under_way_for_tests(&mut self) {
        self.compute_error_of_the_batch_under_way =
            Some(BackgroundComputeError::InjectedByTheTestOnlyHook);
    }

    /// 这一批出了错没有（取走就清掉）：一批读完设置、钩子调过之后读一次。
    pub(crate) fn take_the_compute_error_of_the_batch_under_way(
        &mut self,
    ) -> Option<BackgroundComputeError> {
        self.compute_error_of_the_batch_under_way.take()
    }

    /// 一批出错之后（第 4 条）：出错之前就挂着待生效的切换，就当场解除——那一次切换算停着时收到的请求，`task` 不记成停着，
    /// 下一批开头照常换上它；没有就把 `task` 记成停着，等下一次通过校验的切换请求。停在一次叫起里的（[`Self::begin_a_wake`] 开过、还没结束），
    /// 那一次叫起结束时报警不数这一趟（[`Self::alarm_at_the_end_of_a_wake`] 报 0）；停在可写挂载续做里的不是叫起，停下之后头一次叫起照数 1。不读写盘。
    pub(crate) fn stop_the_task_after_a_compute_error(
        &mut self,
        task: BackgroundTask,
        error: BackgroundComputeError,
    ) -> BackgroundTaskAfterTheComputeError {
        if self.requested_from_the_next_batch.is_some() {
            return BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError;
        }
        if !self.is_stopped_after_a_compute_error(task) {
            assert!(
                self.wakes_of_the_tasks_stopped
                    .iter()
                    .all(|wakes| wakes.task != task),
                "停下以来叫起的次数只记停着的任务：没停着的这一种不该有一条"
            );
            self.stopped_after_a_compute_error
                .push(BackgroundTaskStoppedAfterAComputeError {
                    task,
                    error,
                    batches_compensated_on_the_cpu: 0,
                });
            match self.wake_under_way {
                WakeOfTheBackgroundTasks::UnderWay => {
                    self.wakes_of_the_tasks_stopped.push(WakesOfAStoppedTask {
                        task,
                        wakes_while_stopped: 0,
                        is_stopped_in_the_wake_under_way: true,
                    });
                }
                WakeOfTheBackgroundTasks::NoneUnderWay => {}
            }
        }
        BackgroundTaskAfterTheComputeError::LeftStopped
    }

    /// 一次叫起开头（整理的显式入口每一趟、后台入口每一次判起跑，入口在任何读写之前就拒的那几判之后）：记下此刻有一次叫起在做，
    /// 这一次叫起里一批出错停下的，到它结束时不数（[`Self::stop_the_task_after_a_compute_error`]）。不读写盘。
    pub(crate) fn begin_a_wake(&mut self) {
        self.wake_under_way = WakeOfTheBackgroundTasks::UnderWay;
    }

    /// 换掉停着的那几条（`tasks_stopped` 可以是空：解除全部停下），停下以来叫起的次数一起清掉：换上的每一条都从这一刻起重新数。
    fn replace_the_tasks_stopped(
        &mut self,
        tasks_stopped: Vec<BackgroundTaskStoppedAfterAComputeError>,
    ) {
        self.stopped_after_a_compute_error = tasks_stopped;
        self.wakes_of_the_tasks_stopped.clear();
    }

    /// 一次叫起结束时（整理的显式入口每一趟、后台入口每一次判起跑，不论起没起跑、补没补跑；入口在任何读写之前就拒的那一次不算）：这一次叫起到此结束；
    /// `task` 此刻停着，就把它停下以来叫起的次数加一（这一次叫起里一批出错停下的不加，报 0）、交回这一次的报警
    /// （[`AlarmOfABackgroundTaskStoppedAfterAComputeError`]）；没停着交回 `None`。不读写盘。
    #[must_use = "停着的后台任务每次叫起都要报警：交回的报警要交到调用方手里"]
    pub(crate) fn alarm_at_the_end_of_a_wake(
        &mut self,
        task: BackgroundTask,
    ) -> Option<AlarmOfABackgroundTaskStoppedAfterAComputeError> {
        self.wake_under_way = WakeOfTheBackgroundTasks::NoneUnderWay;
        let stopped = self
            .stopped_after_a_compute_error
            .iter()
            .find(|stopped| stopped.task == task)?
            .clone();
        let wakes_while_stopped = match self
            .wakes_of_the_tasks_stopped
            .iter_mut()
            .find(|wakes| wakes.task == task)
        {
            Some(wakes) if wakes.is_stopped_in_the_wake_under_way => {
                wakes.is_stopped_in_the_wake_under_way = false;
                wakes.wakes_while_stopped
            }
            Some(wakes) => {
                wakes.wakes_while_stopped += 1;
                wakes.wakes_while_stopped
            }
            None => {
                self.wakes_of_the_tasks_stopped.push(WakesOfAStoppedTask {
                    task,
                    wakes_while_stopped: 1,
                    is_stopped_in_the_wake_under_way: false,
                });
                1
            }
        };
        Some(AlarmOfABackgroundTaskStoppedAfterAComputeError {
            task,
            gpu_devices_named_by_the_error: stopped.error.gpu_devices_named(),
            error_that_stopped_the_task: stopped.error,
            wakes_while_stopped,
            batches_compensated_on_the_cpu: stopped.batches_compensated_on_the_cpu,
        })
    }

    /// 补跑那一判此刻的读数：`fullest_device` 是调用方从分配器取的池子此刻最满的那一块盘
    /// （[`AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator`]），阈值取下一批读的那一组（有待生效的取它，没有取生效的）。
    #[must_use]
    pub fn gc_cpu_compensation_reading(
        &self,
        fullest_device: AllocatedBytesAndCapacityOfADevice,
    ) -> GcCpuCompensationReading {
        GcCpuCompensationReading {
            fullest_device,
            threshold: self.gc_cpu_compensation_threshold_of_the_next_batch(),
        }
    }

    /// 下一批读的那一组的补跑阈值：有待生效的取它，没有取生效的。
    fn gc_cpu_compensation_threshold_of_the_next_batch(
        &self,
    ) -> BackgroundGcCpuCompensationThreshold {
        self.requested_from_the_next_batch
            .as_ref()
            .unwrap_or(&self.in_effect)
            .gc_cpu_compensation_threshold
    }

    /// `task` 下一批跑不跑、怎么跑（第 4 条与）：没停着照设置走；GC 停着时池子最满的那一块盘的已分配占它的容量严格大于阈值
    /// （有一块盘超过）就在 CPU 上补跑，不超过就照旧停着。只读，不改状态。
    #[must_use]
    pub fn whether_the_task_runs_the_next_batch(
        &self,
        task: BackgroundTask,
        fullest_device: AllocatedBytesAndCapacityOfADevice,
    ) -> WhetherTheTaskRunsTheNextBatch {
        if !self.is_stopped_after_a_compute_error(task) {
            return WhetherTheTaskRunsTheNextBatch::AsTheSettingsSay;
        }
        let reading = self.gc_cpu_compensation_reading(fullest_device);
        match self.gc_cpu_compensation_judgement {
            GcCpuCompensationJudgement::ByTheReading => {}
            GcCpuCompensationJudgement::NotAboveTheThresholdForcedForTests => {
                return WhetherTheTaskRunsTheNextBatch::StaysStopped(reading);
            }
        }
        match task {
            // 只管 GC：里程碑七的打包搬迁加成员时这里加一臂，照旧停着。
            BackgroundTask::CompactionRelocation => {
                if reading.is_the_allocated_share_above_the_threshold() {
                    WhetherTheTaskRunsTheNextBatch::OnTheCpuToCompensateWhileStopped(reading)
                } else {
                    WhetherTheTaskRunsTheNextBatch::StaysStopped(reading)
                }
            }
        }
    }

    /// 此刻生效的那一组：正在算的这一批（有的话）用的就是它。
    #[must_use]
    pub fn in_effect(&self) -> &AdmittedBackgroundComputeSettings {
        &self.in_effect
    }

    /// 挂着时请求过、还没到下一批开头的那一组。
    #[must_use]
    pub fn requested_from_the_next_batch(&self) -> Option<&AdmittedBackgroundComputeSettings> {
        self.requested_from_the_next_batch.as_ref()
    }

    /// 同 [`Self::request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch`]，GC 在 CPU 上补跑的阈值沿用下一批读的那一组的
    /// （有待生效的取它，没有取生效的）：不给阈值的切换不动阈值。
    ///
    /// # Errors
    /// 同 [`Self::request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch`]。
    pub fn request_switch_from_the_next_batch(
        &mut self,
        settings: BackgroundComputeSettings,
        gpu_compute: Option<BackgroundGpuComputeHandle>,
    ) -> Result<(), BackgroundComputeSettingsRefused> {
        let threshold_of_the_next_batch = self.gc_cpu_compensation_threshold_of_the_next_batch();
        self.request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
            settings,
            gpu_compute,
            threshold_of_the_next_batch,
        )
    }

    /// 挂着切换（第 1 条，一次切全部后台任务）：与挂载同一套校验
    /// （[`admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold`]）；过了就记成待生效，
    /// 下一批开头起用，正在算的这一批照旧；同一批开头之前请求了几次，用最后一次。切到 `gpu` 要有实现：这一次递进来的，没递就沿用这次挂载
    /// 已有的那一份（待生效的那一组带的，没有再看生效的那一组带的）；都没有就是 GPU 不在，拒。有后台任务因一批出错停着时，过了校验的这一次当场解除
    /// 全部停下，从下一批起用新设置。
    ///
    /// # Errors
    /// 校验不过（第 4 条）：拒绝切换，生效的与待生效的都留着原样，停着的照旧停着。
    pub fn request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
        &mut self,
        settings: BackgroundComputeSettings,
        gpu_compute: Option<BackgroundGpuComputeHandle>,
        gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
    ) -> Result<(), BackgroundComputeSettingsRefused> {
        let gpu_compute_of_this_mount = gpu_compute
            .or_else(|| {
                self.requested_from_the_next_batch
                    .as_ref()
                    .and_then(|requested| requested.gpu_compute.clone())
            })
            .or_else(|| self.in_effect.gpu_compute.clone());
        let admitted =
            admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold(
                settings,
                gpu_compute_of_this_mount,
                gc_cpu_compensation_threshold,
            )?;
        self.requested_from_the_next_batch = Some(admitted);
        self.replace_the_tasks_stopped(Vec::new());
        Ok(())
    }

    /// 两批之间换上挂着请求、待生效的那一组（没有就什么都不做）：换上的是 `gpu` 的就把它交给带着的实现（从下一批起照它跑），实现照它一张卡都用不上
    /// 就照挂载那一刻一样，有 GPU 路径的后台任务当场记成出错停着（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；切换照收）；
    /// 用得上就没有停着的（请求切换那一刻已解除全部停下）。
    /// 一批开头（[`Self::begin_a_batch`]）调；整理的两个入口在判「整理停没停」之前也调（显式入口一趟开头、后台入口判起不起跑之前），
    /// 切到一张卡都用不上的那一组之后，下一次进来在任何一个写之前就判出停着：不走删除续做、不取走「该跑了」，与挂载那一刻同。
    /// 只在两批之间调：一批之内的切换只记成待生效。不读写盘。
    pub(crate) fn put_the_switch_requested_from_the_next_batch_in_effect(&mut self) {
        if let Some(requested) = self.requested_from_the_next_batch.take() {
            self.in_effect = requested;
            self.in_effect.hand_the_gpu_settings_to_the_implementation();
            let tasks_stopped =
                tasks_stopped_when_no_gpu_is_usable_under_the_settings_handed_in(&self.in_effect);
            self.replace_the_tasks_stopped(tasks_stopped);
        }
    }

    /// 一批开头（每批读一次设置的那一处）：先把待生效的换上（[`Self::put_the_switch_requested_from_the_next_batch_in_effect`]），
    /// 换上的是 `gpu` 的就在开算之前把它交给带着的实现（从这一批起照它跑，一批之内不再交），
    /// 实现照它一张卡都用不上就照挂载那一刻一样，有 GPU 路径的后台任务当场记成出错停着（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；
    /// 切换照收）；再判 `task` 这一批跑不跑（[`Self::whether_the_task_runs_the_next_batch`]，`fullest_device`
    /// 是调用方这一刻从分配器取的池子最满的那一块盘）——停着而不补跑就不开这一批；最后定这一批走哪条路：补跑的走 CPU、停下以来补跑的批数加一，
    /// 别的按生效的那一组定。有待生效的切换时没有停着的后台任务（请求切换那一刻解除全部停下，停下那一刻挂着切换的不记成停着），
    /// 所以先换上再判与先判再换上，只差换上的那一组用不上 GPU 这一种。
    ///
    /// # Errors
    /// `task` 停着、这一批不补跑 ⇒ [`BatchNotBegunWhileTheTaskStaysStopped`]：没有待生效的切换时状态一样都没动；有的话那一组已换上、交给了实现，
    /// 停着的是这一批开头照它记下的那几条。
    ///
    /// # Panics
    /// 上一批注入的错没被读走：注入只在一批的钩子里，那一批读完设置之后就读走。
    pub(crate) fn begin_a_batch(
        &mut self,
        task: BackgroundTask,
        fullest_device: AllocatedBytesAndCapacityOfADevice,
    ) -> Result<BackgroundBatchUnderWay, BatchNotBegunWhileTheTaskStaysStopped> {
        assert!(
            self.compute_error_of_the_batch_under_way.is_none(),
            "只供测试注入的错只在一批的钩子里注入、那一批就读走：一批开头不该还挂着上一批的"
        );
        self.put_the_switch_requested_from_the_next_batch_in_effect();
        let compensation = match self.whether_the_task_runs_the_next_batch(task, fullest_device) {
            WhetherTheTaskRunsTheNextBatch::StaysStopped(_) => {
                return Err(BatchNotBegunWhileTheTaskStaysStopped);
            }
            WhetherTheTaskRunsTheNextBatch::OnTheCpuToCompensateWhileStopped(reading) => {
                Some(reading)
            }
            WhetherTheTaskRunsTheNextBatch::AsTheSettingsSay => None,
        };
        let requested_backend = self.in_effect.settings.background_compute;
        let route = match compensation {
            None => route_of_the_task(task, requested_backend),
            Some(reading) => {
                let stopped = self
                    .stopped_after_a_compute_error
                    .iter_mut()
                    .find(|stopped| stopped.task == task)
                    .expect("补跑只在 task 停着时判出：上面那一判读的就是这张表，其间没改过它");
                stopped.batches_compensated_on_the_cpu += 1;
                BackgroundComputeRoute::Cpu(
                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(reading),
                )
            }
        };
        Ok(BackgroundBatchUnderWay {
            settings_read_at_the_start_of_the_batch: self.in_effect.clone(),
            task,
            requested_backend,
            route,
        })
    }

    /// 一批结束：交回这一批的记账。`gpu_work` 是这一批在 GPU 上算了什么：走 GPU 的一批恒有（出错放弃的那一批是出错之前算完的几段），
    /// 走 CPU 的恒无。`compute_error_that_abandoned_the_batch` 是这一批算着出错放弃时的那个错。
    ///
    /// # Panics
    /// 生效的设置在这一批里换了（分支变量在操作中途改变，`.claude/rules/fs-design.md` 五条硬要求第 5 条）：
    /// 挂着的切换只记成待生效，换上只在 [`Self::begin_a_batch`]。`gpu_work` 有没有与这一批开头定下的走法不符。
    pub(crate) fn end_the_batch(
        &self,
        under_way: BackgroundBatchUnderWay,
        gpu_work: Option<GpuWorkOfTheBatch>,
        compute_error_that_abandoned_the_batch: Option<BackgroundComputeError>,
    ) -> BackgroundComputeOfOneBatch {
        assert_eq!(
            self.in_effect, under_way.settings_read_at_the_start_of_the_batch,
            "后台计算的设置一批之内不换：挂着的切换只记成待生效，下一批开头才换上"
        );
        let path_taken = match (under_way.route, gpu_work) {
            (BackgroundComputeRoute::Cpu(reason), None) => BackgroundComputePathTaken::Cpu(reason),
            (BackgroundComputeRoute::Gpu, Some(work)) => BackgroundComputePathTaken::Gpu(work),
            (BackgroundComputeRoute::Cpu(_), Some(_)) | (BackgroundComputeRoute::Gpu, None) => {
                panic!(
                    "一批在 GPU 上算了什么只在走 GPU 的那一批有：这一批开头定的是 {:?}",
                    under_way.route
                )
            }
        };
        BackgroundComputeOfOneBatch {
            task: under_way.task,
            requested_backend: under_way.requested_backend,
            path_taken,
            compute_error_that_abandoned_the_batch,
        }
    }
}

/// 一批读完设置、开算之前做什么（`.claude/rules/fs-design.md` 五条硬要求第 2 条）。产品路径什么都不做；只供测试的那一臂调用例给的钩子，
/// 钩子拿到这次挂载的后台计算状态，在这一刻请求切换，强制进入「一批跑到一半时请求切换」那一格。
pub enum InTheMiddleOfEachBackgroundBatch<'hook> {
    NothingHappens,
    CallTheTestOnlyHook(&'hook mut dyn FnMut(&mut BackgroundComputeOfThisMount)),
}

impl InTheMiddleOfEachBackgroundBatch<'_> {
    pub(crate) fn in_the_middle_of_the_batch(
        &mut self,
        background_compute: &mut BackgroundComputeOfThisMount,
    ) {
        match self {
            InTheMiddleOfEachBackgroundBatch::NothingHappens => {}
            InTheMiddleOfEachBackgroundBatch::CallTheTestOnlyHook(hook) => hook(background_compute),
        }
    }
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
                smallest_piece_bytes: 972,
                bytes_already_held_on_the_device: 0,
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
    fn a_stop_in_a_wake_is_reported_as_zero_wakes_and_the_next_three_wakes_count_one_two_three() {
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
    fn a_stop_in_the_writable_mount_continuation_counts_the_next_three_wakes_one_two_three() {
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
    fn a_stop_after_a_wake_has_ended_is_not_counted_as_stopped_in_that_wake() {
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

    /// 同一段一批里算了两次（选段时逐段先扫）：各数相加，卡照先后并起来、同一张卡不重列。
    #[test]
    fn submissions_of_a_segment_computed_twice_in_a_batch_add_up_and_list_each_device_once() {
        let card = |address: &[u8]| GpuDeviceIdentifier::of_opaque_bytes(address.to_vec());
        let mut submissions = GpuSubmissionsOfOneSegment {
            submissions_completed: 2,
            pieces_submitted: 5,
            resubmissions_after_timeouts: 1,
            recuts_after_running_out_of_memory: 0,
            devices_given_work: vec![card(b"0000:41:00.0")],
        };
        submissions.add(&GpuSubmissionsOfOneSegment {
            submissions_completed: 3,
            pieces_submitted: 4,
            resubmissions_after_timeouts: 0,
            recuts_after_running_out_of_memory: 2,
            devices_given_work: vec![card(b"0000:01:00.0"), card(b"0000:41:00.0")],
        });
        assert_eq!(
            submissions,
            GpuSubmissionsOfOneSegment {
                submissions_completed: 5,
                pieces_submitted: 9,
                resubmissions_after_timeouts: 1,
                recuts_after_running_out_of_memory: 2,
                devices_given_work: vec![card(b"0000:41:00.0"), card(b"0000:01:00.0")],
            }
        );
    }
}
