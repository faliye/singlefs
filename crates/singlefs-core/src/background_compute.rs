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

use std::fmt;
use std::path::PathBuf;
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

/// 一张显卡的标识（`background_gpu_devices` 列表里的一项）：不透明的字节，只比相等。卡用什么认、怎么排是里程碑四设计轮（七·2）
/// 还挂着的岔路，这里不定格式：卡清单（[`GpuInventory`]）交出什么，调用方就点什么。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuDeviceIdentifier(Vec<u8>);

impl GpuDeviceIdentifier {
    #[must_use]
    pub fn of_opaque_bytes(opaque_bytes: Vec<u8>) -> Self {
        Self(opaque_bytes)
    }

    /// 标识的字节原样（产品 GPU 包照它当 PCI 地址读那张卡的 PCI 状态）。
    #[must_use]
    pub fn opaque_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// 显存的字节数：一张卡的显存总量，或 `background_gpu_memory_limit` 里给一张卡的上限（第 3 条）。
/// 后台 GPU 包切批与记账的字节数也用它（一段的占用、每条与每段的固定占用、卡上占着的、最小的一片）：都是卡上显存的字节，与上限直接比。
/// 份数、条数各是各的类型（后台 GPU 包的 `memory_limit_and_chunks`），乘份数在那边算。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GpuMemoryInBytes(pub u64);

impl GpuMemoryInBytes {
    /// 两份显存字节数之和；超出 `u64` 交回 `None`。
    #[must_use]
    pub const fn checked_add(self, more: Self) -> Option<Self> {
        match self.0.checked_add(more.0) {
            Some(sum_in_bytes) => Some(Self(sum_in_bytes)),
            None => None,
        }
    }

    /// 扣掉 `less` 之后剩下的；`less` 比它多交回 `None`。
    #[must_use]
    pub const fn checked_sub(self, less: Self) -> Option<Self> {
        match self.0.checked_sub(less.0) {
            Some(difference_in_bytes) => Some(Self(difference_in_bytes)),
            None => None,
        }
    }

    /// 两份显存字节数之和，用在和装得进 `u64` 的地方（各段固定占用的常量、一段真建的缓冲合起来）。
    ///
    /// # Panics
    /// 和超出 `u64`。
    #[must_use]
    pub const fn plus(self, more: Self) -> Self {
        match self.checked_add(more) {
            Some(sum) => sum,
            None => panic!("显存字节数之和超出 u64"),
        }
    }
}

/// 几份显存字节数合起来（一段真建的每一份缓冲加起来）。
///
/// # Panics
/// 和超出 `u64`。
impl std::iter::Sum for GpuMemoryInBytes {
    fn sum<Parts: Iterator<Item = Self>>(parts: Parts) -> Self {
        parts.fold(Self(0), Self::plus)
    }
}

/// 本机有哪些显卡、每张多少显存：只读。GPU 那一侧（住哪是七·2 的岔路）实现它，测试里一份假清单实现它。
/// 挂载与切换的校验只读它；没有这一份清单（`None`）就是 GPU 不在。
pub trait GpuInventory {
    /// 本机全部显卡与各自的显存总量，按「本机全部显卡的列表」的次序：没给 `background_gpu_devices` 时，`background_gpu_memory_limit`
    /// 按这个次序一张卡一个（第 3 条）。怎么排由实现这份清单的一方定（七·2 的岔路）。同一张卡只出现一次。没开着的卡（不参与而没开的）照列。
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)>;
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

/// 实现算一段时出的错（第 4 条「停下并报错」）。核心层对归卡、归输入的那几种做的是同一件事：这一批放弃、意图删掉（有活意图就单发一次只删意图的发布，
/// D26（后台整理与放置回收） 已定项 1 第 3 条）、那个后台任务停下并报这个错，不换回 CPU；
/// 一批里同一个条目键出现两次（[`Self::SameEntryKeyTwiceInOneBatch`]）只放弃这一批，后台任务不停、不报警，下一批照常算（第 8 条）。
/// 成员按停下之后该怎么处置分（修好或不再点那张卡、调大等 GPU 的上限、切回 `cpu`），出在哪张卡、什么原因带在成员里。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackgroundGpuComputeFailed {
    /// 这张卡出错（起不了、设备丢了、等不成、回读坏了、核留下认不出的判定字）：修好它或切换时不再点它，或切回 `cpu`。
    /// `pci_state_when_judged_unusable` 是挂着时一批里卡当场报出错（[`GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost`]）那一刻读的 PCI 状态；
    /// 别的原因、与开卡每一次没开成的原因（[`Self::GpuNotOpenedAfterTheReopenings`] 的 `failures_of_each_opening`）都是没读。
    /// 带读数的三个成员都把读数装箱：读数只在出错时有，装箱之后这个出错类型只多一个指针，装着它的
    /// [`BackgroundComputeError`]、报警与返回它的 `Result` 不过 clippy `result_large_err` 与 `large_enum_variant` 的线。
    GpuFailed {
        device: GpuDeviceIdentifier,
        cause: GpuFailureOnTheDevice,
        pci_state_when_judged_unusable: Box<PciStateOfTheGpuReadWhenJudgedUnusable>,
    },
    /// 等这张卡超过了 `background_gpu_wait_limit`：调大上限，或查这张卡是不是卡死了。
    WaitForTheGpuExceededTheLimit {
        device: GpuDeviceIdentifier,
        wait_limit: Duration,
        submission_waited_for: SubmissionWaitedForPastTheLimit,
    },
    /// 这一批的输入 GPU 那一侧第一版不支持，一张卡都没交活：切回 `cpu`（装不进一段的那一种也可以调大显存上限）。
    InputNotSupportedOnTheGpu(InputNotSupportedOnTheGpu),
    /// 挂着切换点到的这张卡开不成（起不了设备、编不了管线、量代价出错或超时、起不了开卡线程、开卡 panic、前一次开卡没回来），隔一段时间重开到上限也都没开成
    /// （第 4 条「间隔 1 分钟重开，最多重开 3 次……3 次都不成，GC 停下，报警里报出哪张卡、每次开不成的原因，等人处理，不自动再试」），
    /// 或这张卡在这一次挂载里已连着开了 4 次都没开成（开成一次清零）、这一次不再开（[`GpuFailureOnTheDevice::OpeningsInThisMountUsedUp`]）：修好它或切换时不再点它，或切回 `cpu`；
    /// 连着试满的那一种要卸载重挂。`failures_of_each_opening` 是每一次开不成照这张卡出错报成的那一个（[`Self::GpuFailed`] 或
    /// [`Self::WaitForTheGpuExceededTheLimit`]），按开的次序，头一次的在前；连着试满、这一次不再开的只有一个。
    /// `pci_state_when_judged_unusable` 是判它开不成那一刻读的 PCI 状态（每一次没开成而还要重开时不读）。
    GpuNotOpenedAfterTheReopenings {
        device: GpuDeviceIdentifier,
        failures_of_each_opening: Vec<BackgroundGpuComputeFailed>,
        pci_state_when_judged_unusable: Box<PciStateOfTheGpuReadWhenJudgedUnusable>,
    },
    /// 这张卡上一次提交等超过 `wait_limit` 之后，在同一张卡上重交了 `resubmissions` 次、次次等超过上限（「在间隔内重试三次，
    /// 如果gpu无法用就报警」「重试是把那一批重新交一次」）：这张卡用不了。修好它或切换时不再点它，或切回 `cpu`。
    /// `last_submission_waited_for` 是最后那一次等超过上限的是哪一次提交（重交之前先等前一次，那一次没等到也算一次）。
    /// `pci_state_when_judged_unusable` 是判它用不了那一刻读的 PCI 状态（每次超时重交之前不读）。
    GpuUnusableAfterResubmissions {
        device: GpuDeviceIdentifier,
        resubmissions: u32,
        wait_limit: Duration,
        last_submission_waited_for: SubmissionWaitedForPastTheLimit,
        pci_state_when_judged_unusable: Box<PciStateOfTheGpuReadWhenJudgedUnusable>,
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
    /// 交进这一批的条目里同一个条目键出现了两次（两份核比对之前查出，连续不确定的计数一条不动、CPU 参照一次都不调）：是调用方组批的 bug，
    /// 不归哪张卡。两个位置是它在这一批里先后两次出现的位置。核心层收到它只放弃这一批：后台任务不停、不报警，下一批照常算
    /// （[`BackgroundComputeError::reach_of_the_error`]）。
    SameEntryKeyTwiceInOneBatch {
        position_of_the_first_occurrence: usize,
        position_of_the_second_occurrence: usize,
    },
}

impl BackgroundGpuComputeFailed {
    /// 这个错点名的卡，按错里的次序：出在一张卡上的是那一张；参与的卡一张都不剩的是每一张被晾下的（因空闲显存不用的在前）；
    /// 不归哪张卡的（映射 key 比核装得下的长、一批里同一个条目键出现两次）是空。
    #[must_use]
    pub fn gpu_devices_named(&self) -> Vec<GpuDeviceIdentifier> {
        match self {
            BackgroundGpuComputeFailed::GpuFailed { device, .. }
            | BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit { device, .. }
            | BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings { device, .. }
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
            BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch { .. } => Vec::new(),
        }
    }

    /// 判那张卡用不了那一刻读的 PCI 状态：出在一张卡上、带着读数的三种交回它带的那一份（照样可以是没读）；
    /// 别的（等超过上限还没用完重交、输入不支持、参与的卡一张都不剩、一批里同一个条目键出现两次）是没读。
    #[must_use]
    pub fn pci_state_when_judged_unusable(&self) -> PciStateOfTheGpuReadWhenJudgedUnusable {
        match self {
            BackgroundGpuComputeFailed::GpuFailed {
                pci_state_when_judged_unusable,
                ..
            }
            | BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings {
                pci_state_when_judged_unusable,
                ..
            }
            | BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
                pci_state_when_judged_unusable,
                ..
            } => PciStateOfTheGpuReadWhenJudgedUnusable::clone(pci_state_when_judged_unusable),
            BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit { .. }
            | BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(..)
            | BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit { .. }
            | BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
                ..
            }
            | BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch { .. } => {
                PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError
            }
        }
    }
}

/// 一张卡这一批因空闲显存连最小的一片都建不成而不用（[`BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`] 带着）：
/// 哪张卡、它最后那一段建不成时那一段合占的字节数（再压到它的一半就切不出最小的一片了）、这一段最小的一片要多少字节。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuShortOfFreeMemoryForTheSmallestPiece {
    pub device: GpuDeviceIdentifier,
    pub bytes_of_the_last_chunk_not_created: GpuMemoryInBytes,
    pub smallest_piece_bytes: GpuMemoryInBytes,
}

/// 一张卡因显存上限装不下最小的一片而不用：哪张卡、给它的上限、最小的一片要多少字节，
/// 与判的那一刻卡上已经占着的字节（这个进程里之前几次挂载泄漏在这张卡上的，或等超过上限之后还在跑的那一次提交的；不用的判据是
/// 「给的上限扣掉已占的不到最小的一片」）（[`BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit`] 带着；
/// 产品 GPU 包开卡、换设置与每一批切段时报的也是它）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuLeftOutUnderItsMemoryLimit {
    pub device: GpuDeviceIdentifier,
    pub memory_limit: GpuMemoryInBytes,
    pub smallest_piece_bytes: GpuMemoryInBytes,
    pub bytes_already_held_on_the_device: GpuMemoryInBytes,
}

/// 一张卡出错的原因（[`BackgroundGpuComputeFailed::GpuFailed`] 带着）。`reason` 是 GPU 那一侧报的原话。封闭集合。
/// 开卡那几种（起不了设备或编不了管线、起不了开卡线程、开卡 panic、前一次开卡没回来、叫停、连着试满）只在一张卡开不成的原因里出现
/// （[`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`] 的 `failures_of_each_opening`），出路各不相同。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GpuFailureOnTheDevice {
    /// wgpu 起不了这张卡的设备或编不了管线（驱动不对、显存不够；量代价那一段最小的一片装不进绑定上限也报这一种）。
    DeviceOrPipelineDidNotStart { reason: String },
    /// 系统起不了开卡的那条线程（`reason` 是起线程报的原话）：卡本身没碰过，查进程的线程与内存额度。
    OpeningThreadNotStarted { reason: String },
    /// 开卡那一步里 panic 了（`said_by_the_panic` 是 panic 带的原话；驱动里的与本包自己的断言都算）：照这一次没开成重开。
    OpeningPanicked { said_by_the_panic: String },
    /// 同一张卡上前一次开卡到点之后还没回来（那条线程还挂在驱动里），这一次不再起：出路只有重启进程。
    EarlierOpeningOfTheDeviceNotReturned,
    /// 开卡在开完之前叫停了（卸载、切到 `cpu`）：这一次没开，结局没人收。
    OpeningCalledOff,
    /// 这张卡在这一次挂载里已连着开了 `failed_openings` 次都没开成（开成一次清零），切换再点到它不再开（第 4 条：一张卡至多开 4 次）：要卸载重挂。
    OpeningsInThisMountUsedUp { failed_openings: u32 },
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
        bytes_held_by_the_chunk: GpuMemoryInBytes,
        reason: String,
    },
}

/// 判一张卡用不了、要报警的那一刻照它的 PCI 地址读的本机 PCI 状态（D26（后台整理与放置回收） 已定项 13 第 4 条「判一张卡用不了、要报警的那一刻，查一次这张卡的硬件与链路」）。
/// 读在产品 GPU 包里做（`singlefs-background-gpu` 的 `pci_state_of_the_gpu` 模块）；核心层不读，只带着它进停下记录与报警，停着时每次叫起交回的报警带的是停下那一刻的同一份。
/// 读的时刻只有三种：挂着时一批在同一张卡上等超时用完、判它用不了（[`BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions`]）；
/// 开卡重开用完都开不成（[`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]，挂载那一刻拒绝挂载的那一份在产品 GPU 包的挂载拒绝里）；
/// 卡当场报出错（设备丢了、wgpu 报了没接住的错）使后台任务停下（[`BackgroundGpuComputeFailed::GpuFailed`] 带 [`GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost`]）。
/// 封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PciStateOfTheGpuReadWhenJudgedUnusable {
    /// 这个错不是那三种时刻之一，或不归哪张卡：没读。
    NotReadForThisError,
    /// 卡的标识当不了 PCI 设备目录下的一个目录名（不是 UTF-8、是空的、是 `.` 或 `..`、带 `/` 或 NUL 字节）：没读，免得读到那个目录之外去。
    PciBusAddressNotADirectoryName { device: GpuDeviceIdentifier },
    /// 读了：九样各自是读到、文件不在、读不通之一。装箱：九样连同地址与目录比另两个成员大三百多字节（clippy `large_enum_variant`），
    /// 这个枚举又放在出错成员里，不装箱就带得装着它的出错类型、报警与 `Result` 一起变大。
    Read(Box<PciStateOfTheGpu>),
}

impl PciStateOfTheGpuReadWhenJudgedUnusable {
    /// 照读数判这张卡在不在总线上（[`hardware_judgement_of_the_pci_state`]）；没读的交回 `None`。
    #[must_use]
    pub fn hardware_judgement(&self) -> Option<HardwareJudgementOfTheGpu> {
        match self {
            PciStateOfTheGpuReadWhenJudgedUnusable::Read(pci_state) => {
                Some(hardware_judgement_of_the_pci_state(pci_state))
            }
            PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError
            | PciStateOfTheGpuReadWhenJudgedUnusable::PciBusAddressNotADirectoryName { .. } => None,
        }
    }
}

/// 照一张卡的 PCI 地址读的那一份：照哪个地址、哪个目录读的，与九样各自的读数。读不到的那一样报读不到（[`PciFileReading::FileNotPresent`]、
/// [`PciFileReading::NotReadable`]），不当成 0，也不当成正常。链路四样（`current_link_speed`、`current_link_width`、`max_link_speed`、`max_link_width`）
/// 与高级错误报告的三类出错计数（`aer_dev_correctable`、`aer_dev_fatal`、`aer_dev_nonfatal`，每份是多行「名字 计数」）是文件原文、去掉末尾的换行，只报不判；
/// 判硬件坏只看目录在不在与使能位（[`hardware_judgement_of_the_pci_state`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PciStateOfTheGpu {
    /// 照这个地址读的：卡的标识原样（wgpu 适配器信息的 `device_pci_bus_id`，形如 `0000:01:00.0`）。
    pub pci_bus_address: String,
    /// 读的那个目录：PCI 设备目录（本机是 `/sys/bus/pci/devices`）下以地址为名的那一个。
    pub device_directory: PathBuf,
    /// 那个目录在不在：卡在不在总线上。
    pub device_directory_reading: PciDeviceDirectoryReading,
    /// `enable`：使能位。
    pub device_enable: PciFileReading<PciDeviceEnable>,
    /// `current_link_speed`：链路此刻的速度。
    pub current_link_speed: PciFileReading<String>,
    /// `current_link_width`：链路此刻的宽度。
    pub current_link_width: PciFileReading<String>,
    /// `max_link_speed`：链路最大的速度。
    pub maximum_link_speed: PciFileReading<String>,
    /// `max_link_width`：链路最大的宽度。
    pub maximum_link_width: PciFileReading<String>,
    /// `aer_dev_correctable`：可纠正的出错计数。
    pub advanced_error_reporting_correctable: PciFileReading<String>,
    /// `aer_dev_fatal`：致命的出错计数。
    pub advanced_error_reporting_fatal: PciFileReading<String>,
    /// `aer_dev_nonfatal`：不致命的出错计数。
    pub advanced_error_reporting_nonfatal: PciFileReading<String>,
}

/// 一张卡的 PCI 设备目录在不在（[`PciStateOfTheGpu::device_directory_reading`]）。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PciDeviceDirectoryReading {
    Present,
    /// 目录不在：卡不在总线上。
    NotPresent,
    /// 打开那个目录报了别的错（`system_error` 是系统报的原话）：读不通，不当成在、也不当成不在。
    NotReadable {
        system_error: String,
    },
}

/// PCI 设备目录下一个文件的读数。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PciFileReading<Content> {
    Read(Content),
    /// 文件不在（例：不报高级错误报告的设备没有那三份）。
    FileNotPresent,
    /// 读那个文件报了别的错（`system_error` 是系统报的原话）。
    NotReadable {
        system_error: String,
    },
}

/// `enable` 读到的：内核记的这个设备的使能计数（文件原文是一个十进制数），0 是关的。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PciDeviceEnable {
    Enabled {
        enable_count: u64,
    },
    Disabled,
    /// 原文不是一个十进制数（`text` 是去掉首尾空白的原文）：不判开关。
    UnrecognizedText {
        text: String,
    },
}

/// 照一张卡的 PCI 读数判的硬件好坏（[`hardware_judgement_of_the_pci_state`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareJudgementOfTheGpu {
    /// 卡不在总线上：判硬件坏。
    OffTheBus(ReasonTheGpuIsOffTheBus),
    /// 没判出硬件坏（读到的链路与出错计数只报不判；使能位读不到也落在这里）。
    NotJudgedOffTheBus,
}

/// 判卡不在总线上的依据。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReasonTheGpuIsOffTheBus {
    /// 那个 PCI 地址的目录不在了。
    DeviceDirectoryNotPresent,
    /// 使能位读到了、是关的。
    DeviceDisabled,
}

/// 照一张卡的 PCI 读数判硬件坏没坏：只有那个地址的目录不在、或使能位读到了而是关的，才判卡不在总线上；
/// 目录读不通、使能位文件不在或读不通、原文认不得都不判坏；链路速度与宽度、出错计数不看。不读盘、不读内核导出的设备文件，只看交进来的读数。
#[must_use]
pub fn hardware_judgement_of_the_pci_state(
    pci_state: &PciStateOfTheGpu,
) -> HardwareJudgementOfTheGpu {
    match &pci_state.device_directory_reading {
        PciDeviceDirectoryReading::NotPresent => {
            return HardwareJudgementOfTheGpu::OffTheBus(
                ReasonTheGpuIsOffTheBus::DeviceDirectoryNotPresent,
            );
        }
        PciDeviceDirectoryReading::Present | PciDeviceDirectoryReading::NotReadable { .. } => {}
    }
    match &pci_state.device_enable {
        PciFileReading::Read(PciDeviceEnable::Disabled) => {
            HardwareJudgementOfTheGpu::OffTheBus(ReasonTheGpuIsOffTheBus::DeviceDisabled)
        }
        PciFileReading::Read(
            PciDeviceEnable::Enabled { .. } | PciDeviceEnable::UnrecognizedText { .. },
        )
        | PciFileReading::FileNotPresent
        | PciFileReading::NotReadable { .. } => HardwareJudgementOfTheGpu::NotJudgedOffTheBus,
    }
}

/// 等超过上限的是哪一次提交（[`BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit`] 带着）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubmissionWaitedForPastTheLimit {
    /// 这一批交给这张卡的那一次。
    SubmissionOfThisBatch,
    /// 这张卡上前一次等超过上限的那一次：这一批在这张卡上交活之前又等满了上限它还没做完（它的缓冲还占着卡上的显存，第 3 条）。
    EarlierTimedOutSubmissionStillRunning,
    /// 建一段的缓冲出错之后冲掉队列待写的那一次空提交：等满上限它还没做完（已建的那几份缓冲连同暂存还占着卡上的显存，第 3 条）。这一段没交到卡上。
    EmptySubmissionFlushingThePendingWrites,
}

/// 第一版不支持的输入（[`BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu`] 带着）：条款只写了「一批放不下就切成几次交」，
/// 下面这几种切不小的怎么办没写。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputNotSupportedOnTheGpu {
    /// 分到活的那张卡这一批连最小的一片都装不下，而装不下的不是显存上限（给的上限扣掉卡上已占的连最小的一片都装不下的卡，这一批不分给它，
    /// 第 3 条；参与的卡全都这样才报 [`BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit`]）：是单个缓冲的绑定上限。
    /// `entry_position` 是要切的那一条在这一批里的位置；`smallest_piece_bytes` 是那最小的一片（一个字节的输入、或乙段一个落点上的全部条目那一片，补到整字）
    /// 连同一段的固定占用合占的字节数，即这张卡至少要剩这么多（产品 GPU 包按这个数填）：是卡上显存的字节，不是主机上那一条输入的长度。
    EntryLargerThanAChunkOnTheDevice {
        device: GpuDeviceIdentifier,
        entry_position: usize,
        smallest_piece_bytes: GpuMemoryInBytes,
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
    /// 一批里三段都算完之前不再调（一批之内不换）。开着的卡实现不为它重开设备，三段的连续不确定计数照旧留着（第 8 条：只在重挂时清零）。
    /// `settings` 点到没开着的卡时，实现可以不在这一刻换上：在会话锁外开卡、开完了在之后的两批之间换上
    /// （[`Self::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`] 交回 [`LatestGpuSettingsHandedIn::PutInEffect`] 那一问），
    /// 开着的这段时间照旧照交来之前那一组跑，核心层把这一组记成还没生效。
    /// `settings` 的卡都在这份实现答的卡清单里（核心层照这份清单校验过）。
    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings);

    /// 从下一批起一张卡都不参与（核心层在挂载那一刻与切换生效的那一批开头，生效的那一组选 `cpu` 时调；第 2 条「切换之后不再参与的卡，切换生效时关掉」
    /// 用于切到 `cpu`：一张卡都不参与）：开着的卡全部关掉，还在等开卡的那一组与没取走的开卡结局丢掉、开卡叫停。之后再交来一组 `gpu` 的设置，
    /// 它点的卡都当没开的卡照重开的规矩开。不开卡的实现（测试里的假实现）没有这一样，什么都不做。
    fn use_no_gpu_from_the_next_batch(&mut self) {}

    /// 两批之间（每一批开头与整理两个入口判停没停之前，`BackgroundComputeOfThisMount::put_the_switch_requested_from_the_next_batch_in_effect` 里；
    /// 交设置之后当场也问一次）：最近交来的那一组此刻怎样了（[`LatestGpuSettingsHandedIn`]）。在会话锁外开的卡开完了就在这一刻换上；
    /// 还在开的不等。核心层只在交给实现的那一组还没生效时照答案办，别的时候问它只为让实现收下已开完的卡。
    /// 不开卡的实现（测试里的假实现）交来那一刻就换上：答换上了，连同 [`Self::failure_under_the_settings_handed_in`]。
    fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
        &mut self,
    ) -> LatestGpuSettingsHandedIn {
        LatestGpuSettingsHandedIn::PutInEffect {
            failure_under_the_settings: self.failure_under_the_settings_handed_in(),
        }
    }

    /// 照最近一次换上的那一组设置一张卡都用不上时，那一组下每一批三段都会交回的那个出错
    /// （例如参与的卡的显存上限连最小的一片都装不下、一张都不剩）；用得上交回 `None`。核心层不直接读它：
    /// [`Self::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`] 的缺省答法把它带进「换上了」那一种，
    /// 核心层照它判停（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]），不等那一组下的头一批交回出错。只读，不交活。
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

/// 两批之间问实现的那一问（[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]）交回的：
/// 最近交给它的那一组此刻怎样了。核心层照它定这一组什么时候生效（第 4 条「开成了，下一批起换成新设置」）。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LatestGpuSettingsHandedIn {
    /// 实现还在会话锁外为它开卡：照旧照交来之前那一组跑，这一组还没生效。
    StillOpeningTheirCards,
    /// 换上了：从下一批起照它跑。`failure_under_the_settings` 是照它一张卡都用不上时每一批三段都会交回的那个出错（用得上是 `None`），
    /// 核心层照挂载那一刻的停法判（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]）。
    PutInEffect {
        failure_under_the_settings: Option<BackgroundGpuComputeFailed>,
    },
    /// 它点到的一张卡重开到上限都没开成，或这张卡在这一次挂载里已连着试满、不再开（[`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]）：
    /// 这一组不生效，实现照旧照交来之前那一组（核心层生效的那一组）；核心层把有 GPU 路径的后台任务当场记成出错停着、报警，不自动再试，
    /// 直到下一次通过校验的切换。
    NotOpened { failure: BackgroundGpuComputeFailed },
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
/// 至少一张卡、同一张卡只出现一次、每张都在校验时的卡清单里、上限不超过那张卡的显存总量、等的上限不是 0。
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

/// 挂载参数 `background_gc_cpu_compensation_threshold`（有默认值、可调）：GC 因一批计算出错停着时，池子里有一块盘的已分配字节占它的容量的百分比严格大于它
/// （第 4 条：任一块盘超过就补跑），GC 照常起跑、这一批在 CPU 上补跑。
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
/// 等 GPU 结果的上限给了 0；GC 在 CPU 上补跑的阈值超过 100%。切换点到的卡开不开得成不在这里判：切换照收，开不成照
/// [`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`] 停 GC、报警。
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

    /// 把这一组交给它带着的实现：选 `gpu` 时交这一组（[`BackgroundGpuCompute::use_the_settings_from_the_next_batch`]），从下一批起照它跑
    /// （点到没开的卡时实现开完才换上）；选 `cpu` 而带着实现时告诉它一张卡都不参与（[`BackgroundGpuCompute::use_no_gpu_from_the_next_batch`]，
    /// 第 2 条「从 gpu 切到 cpu……关掉全部开着的卡」）；选 `cpu` 而没带实现时什么都不做。
    ///
    /// # Panics
    /// 选 `gpu` 而没带实现：选 `gpu` 的那一组在校验那一刻就带上实现。
    fn hand_the_settings_to_the_implementation(&self) {
        match (&self.gpu_settings, &self.gpu_compute) {
            (Some(gpu_settings), gpu_compute) => gpu_compute
                .as_ref()
                .expect("选 gpu 的那一组在校验那一刻就带上实现：没给实现照 GPU 不在拒了")
                .lock()
                .use_the_settings_from_the_next_batch(gpu_settings),
            (None, Some(gpu_compute)) => gpu_compute.lock().use_no_gpu_from_the_next_batch(),
            (None, None) => {}
        }
    }

    /// 选 `gpu` 时问它带着的实现最近交给它的那一组此刻怎样了
    /// （[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]，开完的卡在这一刻换上）；
    /// 选 `cpu` 时 `None`（实现这时不算活）。
    ///
    /// # Panics
    /// 选 `gpu` 而没带实现：选 `gpu` 的那一组在校验那一刻就带上实现。
    fn latest_gpu_settings_handed_in_asked_of_the_implementation(
        &self,
    ) -> Option<LatestGpuSettingsHandedIn> {
        self.gpu_settings.as_ref()?;
        Some(
            self.gpu_compute
                .as_ref()
                .expect("选 gpu 的那一组在校验那一刻就带上实现：没给实现照 GPU 不在拒了")
                .lock()
                .put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(),
        )
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
/// [`BackgroundComputeSettingsRefused`] 除阈值越界之外的七种。
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
/// [`BackgroundComputeSettingsRefused`] 的八种。
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
/// [`BackgroundComputeSettingsRefused`] 除阈值越界之外的七种。
pub fn admit_gpu_settings_against_the_inventory(
    settings: &BackgroundComputeSettings,
    gpu_inventory: &dyn GpuInventory,
) -> Result<AdmittedGpuSettings, BackgroundComputeSettingsRefused> {
    let inventory_read_once = GpuInventoryReadOnce {
        devices_with_total_memory: gpu_inventory.devices_with_total_memory(),
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
}

impl GpuInventory for GpuInventoryReadOnce {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        self.devices_with_total_memory.clone()
    }
}

/// 第 4 条的校验：选 `cpu` 时卡、显存上限与等 GPU 的上限都照收、不校验（切到 `gpu` 那一刻再校验）；选 `gpu` 时，依次判 GPU 在不在、
/// 点名的卡在不在清单里、上限个数与参与的卡数等不等、每个上限超没超那张卡的显存总量、等 GPU 的上限是不是 0，有一样不对就拒。
/// 只读卡清单，不碰盘。只校验：交回的是交进来的那一组，不带实现、装不进挂载；挂载与切换经
/// [`admit_background_compute_settings_with_the_gpu_compute`]（卡清单由实现答），产品 GPU 包开包时用这一份。
///
/// # Errors
/// [`BackgroundComputeSettingsRefused`] 除阈值越界之外的七种。
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
    /// 这个错管到哪：实现交回一批里同一个条目键出现两次的只放弃这一批，别的都停下那个后台任务。
    #[must_use]
    pub fn reach_of_the_error(&self) -> ReachOfTheComputeError {
        match self {
            BackgroundComputeError::GpuSegmentFailed {
                failure: BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch { .. },
                ..
            } => ReachOfTheComputeError::AbandonsOnlyTheBatch,
            BackgroundComputeError::InjectedByTheTestOnlyHook
            | BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn { .. }
            | BackgroundComputeError::GpuSegmentFailed {
                failure:
                    BackgroundGpuComputeFailed::GpuFailed { .. }
                    | BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit { .. }
                    | BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(..)
                    | BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings { .. }
                    | BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions { .. }
                    | BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit { .. }
                    | BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece { .. },
                ..
            } => ReachOfTheComputeError::StopsTheTask,
        }
    }

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

    /// 这个错带的、判那张卡用不了那一刻读的 PCI 状态（[`BackgroundGpuComputeFailed::pci_state_when_judged_unusable`]）；
    /// 只供测试的钩子注入的不归哪张卡，是没读。
    #[must_use]
    pub fn pci_state_when_judged_unusable(&self) -> PciStateOfTheGpuReadWhenJudgedUnusable {
        match self {
            BackgroundComputeError::InjectedByTheTestOnlyHook => {
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

/// 后台计算在这一次挂载里的状态：此刻生效的设置；已交给实现、实现还在会话锁外开卡而没生效的那一组；挂着时请求、从下一批开头起交给实现的那一组；
/// 因一批出错停着的后台任务。生命周期一次挂载，只住内存。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackgroundComputeOfThisMount {
    /// 此刻生效的那一组：一批照它定走哪条路。选 `gpu` 的一组在实现答换上了（[`LatestGpuSettingsHandedIn::PutInEffect`]）那一刻才成为生效的；
    /// 实现答开不成（[`LatestGpuSettingsHandedIn::NotOpened`]）的那一组不生效，生效的照旧是切换之前那一组（实现照旧用的就是它）；
    /// 选 `cpu` 的一组交给实现那一刻就生效。
    in_effect: AdmittedBackgroundComputeSettings,
    /// 已交给实现、实现答还在为它开卡的那一组（选 `gpu`）：这期间照旧照 `in_effect` 跑；这期间一批出错，后台任务停下、报警，
    /// 实现答它换上了的那一刻解除（[`Self::stop_the_task_after_a_compute_error`]）。
    handed_in_while_the_implementation_opens_its_cards: Option<AdmittedBackgroundComputeSettings>,
    /// 实现答开不成的那一组请求给的补跑阈值：那一组不生效而阈值随切换请求从下一批起生效（第 4 条），补跑那一判照它；
    /// 生效的那一组换掉时清掉。没有是 `None`（照 `in_effect` 带的）。
    gc_cpu_compensation_threshold_of_the_switch_not_opened:
        Option<BackgroundGcCpuCompensationThreshold>,
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
    /// 挂载那一刻：挂载给的那一组（校验过）生效，没有待生效的。当场把这一组交给它带着的实现（[`AdmittedBackgroundComputeSettings`] 选 `gpu`
    /// 时交这一组，选 `cpu` 而带着实现时告诉它一张卡都不参与）、选 `gpu` 时当场问一次（[`Self::put_the_switch_requested_from_the_next_batch_in_effect`]
    /// 里的同一判）：实现答换上了而照这一组一张卡都用不上时，有 GPU 路径的后台任务当场记成出错停着
    /// （[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]，补跑的批数 0），挂载照常做成；别的情形没有停着的后台任务。
    #[must_use]
    pub fn of_a_mount(admitted_at_the_mount: AdmittedBackgroundComputeSettings) -> Self {
        let mut background_compute = Self {
            in_effect: admitted_at_the_mount.clone(),
            handed_in_while_the_implementation_opens_its_cards: None,
            gc_cpu_compensation_threshold_of_the_switch_not_opened: None,
            requested_from_the_next_batch: None,
            stopped_after_a_compute_error: Vec::new(),
            wakes_of_the_tasks_stopped: Vec::new(),
            wake_under_way: WakeOfTheBackgroundTasks::NoneUnderWay,
            compute_error_of_the_batch_under_way: None,
            gc_cpu_compensation_judgement: GcCpuCompensationJudgement::ByTheReading,
        };
        background_compute.hand_the_settings_in(admitted_at_the_mount);
        background_compute.take_the_answer_on_the_settings_handed_in();
        background_compute
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

    /// 一批出错之后（第 4 条）：一批里同一个条目键出现两次的只放弃这一批（[`ReachOfTheComputeError::AbandonsOnlyTheBatch`]），`task` 不停、不报警；
    /// 出错之前就挂着请求了、还没交给实现的切换，就当场解除——那一次切换算停着时收到的请求，`task` 不记成停着，之后照常等它生效；
    /// 别的把 `task` 记成停着：交给了实现而实现还在为它开卡的，那一组换上的那一刻解除（[`Self::take_the_answer_on_the_settings_handed_in`]），
    /// 都等下一次通过校验的切换请求。停在一次叫起里的（[`Self::begin_a_wake`] 开过、还没结束），
    /// 那一次叫起结束时报警不数这一趟（[`Self::alarm_at_the_end_of_a_wake`] 报 0）；停在可写挂载续做里的不是叫起，停下之后头一次叫起照数 1。不读写盘。
    pub(crate) fn stop_the_task_after_a_compute_error(
        &mut self,
        task: BackgroundTask,
        error: BackgroundComputeError,
    ) -> BackgroundTaskAfterTheComputeError {
        match error.reach_of_the_error() {
            ReachOfTheComputeError::AbandonsOnlyTheBatch => {
                return BackgroundTaskAfterTheComputeError::NotStoppedForOnlyTheBatchIsAbandoned;
            }
            ReachOfTheComputeError::StopsTheTask => {}
        }
        if self.requested_from_the_next_batch.is_some() {
            return BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError;
        }
        let task_after_the_error = match self.handed_in_while_the_implementation_opens_its_cards {
            Some(_) => BackgroundTaskAfterTheComputeError::LeftStoppedWhileTheSwitchOpensItsCards,
            None => BackgroundTaskAfterTheComputeError::LeftStopped,
        };
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
        task_after_the_error
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
    /// （[`AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator`]），阈值取下一批读的那一组
    /// （[`Self::gc_cpu_compensation_threshold_of_the_next_batch`]）。
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

    /// 下一批读的那一组的补跑阈值：有请求了还没交给实现的取它，再看交给了实现而实现还在开卡的那一组，再看实现答开不成的那一组请求给的，
    /// 都没有取生效的（第 4 条：阈值随切换请求一起换、从下一批起生效，不等开卡，开不成也照它）。
    fn gc_cpu_compensation_threshold_of_the_next_batch(
        &self,
    ) -> BackgroundGcCpuCompensationThreshold {
        self.requested_from_the_next_batch
            .as_ref()
            .or(self
                .handed_in_while_the_implementation_opens_its_cards
                .as_ref())
            .map(|requested| requested.gc_cpu_compensation_threshold)
            .or(self.gc_cpu_compensation_threshold_of_the_switch_not_opened)
            .unwrap_or(self.in_effect.gc_cpu_compensation_threshold)
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
            // 只管 GC：里程碑六的打包搬迁加成员时这里加一臂，照旧停着。
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

    /// 已交给实现、实现还在会话锁外为它开卡、还没生效的那一组（选 `gpu`）；没有是 `None`（`.claude/rules/fs-design.md` 五条硬要求第 4 条：
    /// 运行时看得出这一刻跑的是切换之前那一组）。
    #[must_use]
    pub fn handed_in_while_the_implementation_opens_its_cards(
        &self,
    ) -> Option<&AdmittedBackgroundComputeSettings> {
        self.handed_in_while_the_implementation_opens_its_cards
            .as_ref()
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

    /// 两批之间：挂着请求、待生效的那一组有的话交给实现（[`Self::hand_the_settings_in`]），再问实现最近交给它的那一组此刻怎样了
    /// （[`Self::take_the_answer_on_the_settings_handed_in`]）：选 `gpu` 的一组要等实现答换上了才生效，实现还在会话锁外开卡的这段时间照旧照切换之前那一组跑
    /// （这期间一批出错，后台任务停下、报警，[`Self::stop_the_task_after_a_compute_error`]）；答换上了就照挂载那一刻的停法判
    /// （照它一张卡都用不上就把有 GPU 路径的后台任务当场记成出错停着，[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；用得上就不停，
    /// 开卡期间出错停着的也在这一刻解除）；答开不成，那一组不生效、生效的照旧是切换之前那一组，有 GPU 路径的后台任务照「一张卡都用不上」一样停着、报那个错
    /// （第 4 条「3 次都不成，GC 停下，报警……不自动再试」），直到下一次通过校验的切换。
    /// 选 `cpu` 的一组交给实现那一刻就生效。
    /// 一批开头（[`Self::begin_a_batch`]）调；整理的两个入口在判「整理停没停」之前也调（显式入口一趟开头、后台入口判起不起跑之前），
    /// 切到一张卡都用不上的那一组之后，下一次进来在任何一个写之前就判出停着：不走删除续做、不取走「该跑了」，与挂载那一刻同。
    /// 只在两批之间调：一批之内的切换只记成待生效。不读写盘。
    pub(crate) fn put_the_switch_requested_from_the_next_batch_in_effect(&mut self) {
        if let Some(requested) = self.requested_from_the_next_batch.take() {
            self.hand_the_settings_in(requested);
        }
        self.take_the_answer_on_the_settings_handed_in();
    }

    /// 把 `settings` 交给它带着的实现（[`AdmittedBackgroundComputeSettings::hand_the_settings_to_the_implementation`]）：选 `cpu` 的当场生效
    /// （之前交给实现、还在开卡的那一组作废）；选 `gpu` 的记成交给了实现、还没生效，等实现答（[`Self::take_the_answer_on_the_settings_handed_in`]）。
    fn hand_the_settings_in(&mut self, settings: AdmittedBackgroundComputeSettings) {
        settings.hand_the_settings_to_the_implementation();
        match settings.gpu_settings() {
            None => {
                self.put_the_settings_in_effect(settings);
                self.handed_in_while_the_implementation_opens_its_cards = None;
            }
            Some(_) => {
                self.handed_in_while_the_implementation_opens_its_cards = Some(settings);
            }
        }
    }

    /// 换上生效的那一组（实现答换上了、或选 `cpu` 的一组交给实现那一刻）：开不成那一组留下的补跑阈值跟着清掉。
    fn put_the_settings_in_effect(&mut self, settings: AdmittedBackgroundComputeSettings) {
        self.in_effect = settings;
        self.gc_cpu_compensation_threshold_of_the_switch_not_opened = None;
    }

    /// 问实现最近交给它的那一组此刻怎样了（[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]）：
    /// 有交给了实现、还没生效的那一组就问它带着的实现，照答案办——还在开卡：不动；换上了：它生效，停着的那几条换成照它判出的
    /// （[`tasks_stopped_when_no_gpu_is_usable_under_the_settings_put_in_effect`]；开卡期间出错停着的就此解除）；开不成：它不生效、清掉，
    /// 生效的照旧是切换之前那一组，有 GPU 路径的后台任务记成出错停着、报那个错，补跑阈值照它请求给的（[`Self::gc_cpu_compensation_threshold_of_the_next_batch`]）。
    /// 没有还没生效的那一组而生效的选 `gpu` 时也问生效那一组带着的实现（让它收下为更早几次交设置开完的卡），答案不用。
    fn take_the_answer_on_the_settings_handed_in(&mut self) {
        let Some(handed_in) = self
            .handed_in_while_the_implementation_opens_its_cards
            .take()
        else {
            let _answer_with_no_settings_waiting = self
                .in_effect
                .latest_gpu_settings_handed_in_asked_of_the_implementation();
            return;
        };
        match handed_in.latest_gpu_settings_handed_in_asked_of_the_implementation() {
            None | Some(LatestGpuSettingsHandedIn::StillOpeningTheirCards) => {
                self.handed_in_while_the_implementation_opens_its_cards = Some(handed_in);
            }
            Some(LatestGpuSettingsHandedIn::PutInEffect {
                failure_under_the_settings,
            }) => {
                self.put_the_settings_in_effect(handed_in);
                self.replace_the_tasks_stopped(
                    tasks_stopped_when_no_gpu_is_usable_under_the_settings_put_in_effect(
                        failure_under_the_settings.as_ref(),
                    ),
                );
            }
            Some(LatestGpuSettingsHandedIn::NotOpened { failure }) => {
                self.gc_cpu_compensation_threshold_of_the_switch_not_opened =
                    Some(handed_in.gc_cpu_compensation_threshold);
                self.replace_the_tasks_stopped(tasks_with_a_gpu_path_stopped_by(&failure));
            }
        }
    }

    /// 一批开头（每批读一次设置的那一处）：先把待生效的交给实现、问实现最近交给它的那一组此刻怎样了
    /// （[`Self::put_the_switch_requested_from_the_next_batch_in_effect`]；一批之内不再交、不再问），
    /// 实现答换上了而照它一张卡都用不上、或答开不成，就照挂载那一刻一样，有 GPU 路径的后台任务当场记成出错停着
    /// （[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；切换照收）；再判 `task` 这一批跑不跑（[`Self::whether_the_task_runs_the_next_batch`]，`fullest_device`
    /// 是调用方这一刻从分配器取的池子最满的那一块盘）——停着而不补跑就不开这一批；最后定这一批走哪条路：补跑的走 CPU、停下以来补跑的批数加一，
    /// 别的按生效的那一组定（实现还在为交给它的那一组开卡、或答了开不成时，生效的仍是切换之前那一组）。先问再判：停着的那几条照这一刻的答案换过
    /// （换上了的那一刻解除开卡期间出错的停下，开不成的那一刻照开不成停着）。
    ///
    /// # Errors
    /// `task` 停着、这一批不补跑 ⇒ [`BatchNotBegunWhileTheTaskStaysStopped`]：没有还没生效的切换、实现也没答出新结局时状态一样都没动；
    /// 有的话那一组已交给了实现、实现照它答过，停着的是这一批开头照答案记下的那几条。
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

    #[test]
    fn gpu_memory_in_bytes_checked_add_gives_the_sum_and_refuses_past_u64() {
        assert_eq!(
            GpuMemoryInBytes(600).checked_add(GpuMemoryInBytes(400)),
            Some(GpuMemoryInBytes(1000))
        );
        assert_eq!(
            GpuMemoryInBytes(u64::MAX).checked_add(GpuMemoryInBytes(1)),
            None
        );
    }

    #[test]
    fn gpu_memory_in_bytes_checked_sub_gives_what_is_left_and_refuses_taking_more_than_there_is() {
        assert_eq!(
            GpuMemoryInBytes(1000).checked_sub(GpuMemoryInBytes(600)),
            Some(GpuMemoryInBytes(400))
        );
        assert_eq!(
            GpuMemoryInBytes(600).checked_sub(GpuMemoryInBytes(1000)),
            None
        );
    }

    #[test]
    fn gpu_memory_in_bytes_plus_gives_the_sum() {
        assert_eq!(
            GpuMemoryInBytes(16).plus(GpuMemoryInBytes(40)),
            GpuMemoryInBytes(56)
        );
    }

    #[test]
    #[should_panic(expected = "显存字节数之和超出 u64")]
    fn gpu_memory_in_bytes_plus_past_u64_panics() {
        let _sum = GpuMemoryInBytes(u64::MAX).plus(GpuMemoryInBytes(1));
    }

    #[test]
    fn gpu_memory_in_bytes_summed_over_the_parts_is_their_total() {
        let total: GpuMemoryInBytes = [8_u64, 16, 24].into_iter().map(GpuMemoryInBytes).sum();
        assert_eq!(total, GpuMemoryInBytes(48));
    }

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

    // ── 切换点到没开的卡：交给实现之后、实现开完卡之前，切换还没生效（第 4 条「开成了，下一批起换成新设置」「出错之前已经收到、还没生效的切换请求，
    // 在停下那一刻也算数」「报的是挂载那一刻同一个成员」；第 2 条「从 gpu 切到 cpu……关掉全部开着的卡」）──

    fn card_for_the_switch_tests(address: &[u8]) -> GpuDeviceIdentifier {
        GpuDeviceIdentifier::of_opaque_bytes(address.to_vec())
    }

    /// 替身照 [`BackgroundGpuCompute`] 文档写的约定：交来的一组点的卡都开着就当场换上；点到没开的卡就在会话锁外开（用例把 `opening_ends` 摆成 `true` 就算开完了），
    /// 开着的这段时间两批之间那一问答还在开卡，开完了换上、答换上了连同这一组用不用得上（`failure_once_put_in_effect`）。告诉它一张卡都不参与时开着的卡全部关掉。
    struct CardsOpenedByTheTestDoubleState {
        cards_opened: Vec<GpuDeviceIdentifier>,
        cards_waiting_for_the_opening: Option<Vec<GpuDeviceIdentifier>>,
        opening_ends: bool,
        failure_under_the_settings_in_use: Option<BackgroundGpuComputeFailed>,
        failure_once_put_in_effect: Option<BackgroundGpuComputeFailed>,
        times_told_no_gpu_takes_part: u32,
    }

    struct GpuComputeOpeningCardsWhenTheTestSays {
        state: Arc<Mutex<CardsOpenedByTheTestDoubleState>>,
    }

    impl GpuInventory for GpuComputeOpeningCardsWhenTheTestSays {
        fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
            vec![
                (
                    card_for_the_switch_tests(b"0000:01:00.0"),
                    GpuMemoryInBytes(8 << 30),
                ),
                (
                    card_for_the_switch_tests(b"0000:41:00.0"),
                    GpuMemoryInBytes(8 << 30),
                ),
            ]
        }
    }

    impl BackgroundGpuCompute for GpuComputeOpeningCardsWhenTheTestSays {
        fn verify_unit_checksums(
            &mut self,
            _units: &[UnitChecksumToVerifyOnTheGpu<'_>],
        ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed>
        {
            Ok(Vec::new())
        }

        fn classify_region_records(
            &mut self,
            _region: CompactionRegion,
            _records: &[RegionRecordToClassifyOnTheGpu<'_>],
            _mapping_entries: &[CentralMappingEntry],
        ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
            Ok(Vec::new())
        }

        fn count_live_slots_per_cluster_segment(
            &mut self,
            _allocation_records: &[AllocationRecord],
            _unit_area: UnitAreaOfOneDevice,
            _segments: &[ClusterSegmentEntryKey],
        ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
            Ok(Vec::new())
        }

        fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
            let mut state = self.state.lock().expect("用例没在别处 panic 过");
            let cards_named: Vec<GpuDeviceIdentifier> = settings
                .participating_devices()
                .iter()
                .map(|participating| participating.device.clone())
                .collect();
            if cards_named
                .iter()
                .all(|card| state.cards_opened.contains(card))
            {
                state.cards_waiting_for_the_opening = None;
                state.failure_under_the_settings_in_use = state.failure_once_put_in_effect.clone();
            } else {
                state.cards_waiting_for_the_opening = Some(cards_named);
            }
        }

        fn use_no_gpu_from_the_next_batch(&mut self) {
            let mut state = self.state.lock().expect("用例没在别处 panic 过");
            state.cards_opened.clear();
            state.cards_waiting_for_the_opening = None;
            state.times_told_no_gpu_takes_part += 1;
        }

        fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
            &mut self,
        ) -> LatestGpuSettingsHandedIn {
            let mut state = self.state.lock().expect("用例没在别处 panic 过");
            if state.cards_waiting_for_the_opening.is_some() {
                if !state.opening_ends {
                    return LatestGpuSettingsHandedIn::StillOpeningTheirCards;
                }
                state.cards_opened = state
                    .cards_waiting_for_the_opening
                    .take()
                    .expect("上面判过在等");
                state.failure_under_the_settings_in_use = state.failure_once_put_in_effect.clone();
            }
            LatestGpuSettingsHandedIn::PutInEffect {
                failure_under_the_settings: state.failure_under_the_settings_in_use.clone(),
            }
        }
    }

    fn gpu_settings_naming_the_card(
        address: &[u8],
        memory_limit: Option<u64>,
    ) -> BackgroundComputeSettings {
        BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(vec![card_for_the_switch_tests(address)]),
            background_gpu_memory_limit: memory_limit.map(|limit| vec![GpuMemoryInBytes(limit)]),
            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        }
    }

    /// 池子最满的那一块盘远没到补跑阈值：停着就照旧停着。
    fn fullest_device_far_below_the_threshold() -> AllocatedBytesAndCapacityOfADevice {
        AllocatedBytesAndCapacityOfADevice {
            device: DeviceIdentity(0),
            allocated_bytes: 0,
            capacity_bytes: 1 << 30,
        }
    }

    fn every_gpu_left_out_on_the_card(address: &[u8]) -> BackgroundGpuComputeFailed {
        BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
            gpus_left_out: vec![GpuLeftOutUnderItsMemoryLimit {
                device: card_for_the_switch_tests(address),
                memory_limit: GpuMemoryInBytes(64),
                smallest_piece_bytes: GpuMemoryInBytes(972),
                bytes_already_held_on_the_device: GpuMemoryInBytes(0),
            }],
        }
    }

    /// 替身开着「0000:01:00.0」，两个出错照 `failure_under_the_settings_in_use`、`failure_once_put_in_effect` 摆。
    fn state_with_the_card_at_one_opened(
        failure_under_the_settings_in_use: Option<BackgroundGpuComputeFailed>,
        failure_once_put_in_effect: Option<BackgroundGpuComputeFailed>,
    ) -> Arc<Mutex<CardsOpenedByTheTestDoubleState>> {
        Arc::new(Mutex::new(CardsOpenedByTheTestDoubleState {
            cards_opened: vec![card_for_the_switch_tests(b"0000:01:00.0")],
            cards_waiting_for_the_opening: None,
            opening_ends: false,
            failure_under_the_settings_in_use,
            failure_once_put_in_effect,
            times_told_no_gpu_takes_part: 0,
        }))
    }

    /// 选 `gpu`、点「0000:01:00.0」的一次挂载，替身状态照 `state`。
    fn mount_on_the_card_at_one(
        state: &Arc<Mutex<CardsOpenedByTheTestDoubleState>>,
    ) -> BackgroundComputeOfThisMount {
        let handle = BackgroundGpuComputeHandle::of_an_implementation(
            GpuComputeOpeningCardsWhenTheTestSays {
                state: Arc::clone(state),
            },
        );
        BackgroundComputeOfThisMount::of_a_mount(
            admit_background_compute_settings_with_the_gpu_compute(
                gpu_settings_naming_the_card(b"0000:01:00.0", None),
                Some(handle),
            )
            .expect("卡清单列着「0000:01:00.0」"),
        )
    }

    fn failure_on_the_card_at_one() -> BackgroundComputeError {
        BackgroundComputeError::GpuSegmentFailed {
            segment: GpuComputedSegment::UnitChecksumVerification,
            failure: BackgroundGpuComputeFailed::GpuFailed {
                device: card_for_the_switch_tests(b"0000:01:00.0"),
                cause: GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost {
                    reason: "替身：切换之前那张卡上设备丢了".to_string(),
                },
                pci_state_when_judged_unusable: Box::new(
                    PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
                ),
            },
        }
    }

    fn route_of_the_batch(
        batch: &Result<BackgroundBatchUnderWay, BatchNotBegunWhileTheTaskStaysStopped>,
    ) -> Option<BackgroundComputeRoute> {
        batch.as_ref().ok().map(|under_way| under_way.route)
    }

    /// 交给了实现、实现还在开卡的期间一批出错：挂载点「0000:01:00.0」，请求切到没开的「0000:41:00.0」；下一批开头交给实现（实现开卡、照切换之前那一组跑），
    /// 这一批在「0000:01:00.0」上出错——停下（不照还没生效的那一组解除）；卡开成、实现答换上了的那一批开头解除，那一批照常在 GPU 上开，
    /// 生效的是点「0000:41:00.0」的那一组。
    #[test]
    fn a_batch_failing_on_the_card_before_the_switch_while_the_new_card_opens_stops_until_that_switch_is_put_in_effect(
    ) {
        let state = state_with_the_card_at_one_opened(None, None);
        let mut background_compute = mount_on_the_card_at_one(&state);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        let batch_while_opening = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            route_of_the_batch(&batch_while_opening),
            Some(BackgroundComputeRoute::Gpu),
            "开卡的这段时间照切换之前那一组在 GPU 上算"
        );
        assert!(
            background_compute
                .handed_in_while_the_implementation_opens_its_cards()
                .is_some(),
            "交给了实现、还在开卡：切换还没生效"
        );
        assert_eq!(
            background_compute.stop_the_task_after_a_compute_error(
                BackgroundTask::CompactionRelocation,
                failure_on_the_card_at_one(),
            ),
            BackgroundTaskAfterTheComputeError::LeftStoppedWhileTheSwitchOpensItsCards,
            "开卡期间旧卡出错：停下"
        );
        assert!(background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
        state.lock().expect("用例没在别处 panic 过").opening_ends = true;
        let batch_once_opened = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            route_of_the_batch(&batch_once_opened),
            Some(BackgroundComputeRoute::Gpu),
            "换上的那一刻解除：那一批照常开"
        );
        assert!(!background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
        assert_eq!(
            background_compute
                .in_effect()
                .settings()
                .background_gpu_devices,
            Some(vec![card_for_the_switch_tests(b"0000:41:00.0")]),
            "实现答换上了：生效的是切换点的那一组"
        );
    }

    /// 同一段历史的对照：同一个错落在核心层把切换交给实现之前（请求还挂着）——照那一次请求当场解除。
    #[test]
    fn the_same_error_before_the_handing_in_is_lifted_by_the_switch_request() {
        let state = state_with_the_card_at_one_opened(None, None);
        let mut background_compute = mount_on_the_card_at_one(&state);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        assert_eq!(
            background_compute.stop_the_task_after_a_compute_error(
                BackgroundTask::CompactionRelocation,
                failure_on_the_card_at_one(),
            ),
            BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError
        );
    }

    /// 第 4 条「停着的时候收到任何一次通过校验的切换请求……当场解除停下，从下一批起用新设置」：挂载那一刻照挂载那一组一张卡都用不上、停着；
    /// 请求切到没开的「0000:41:00.0」（装得下）——请求那一刻解除；交给实现那一批开头实现还在开卡，不读切换之前那一组的「用不上」，不再停；
    /// 卡开成、换上之后照它判，用得上，照常开批。
    #[test]
    fn a_mount_stopped_under_its_settings_is_not_stopped_again_while_the_switched_card_opens_nor_once_it_is_put_in_effect(
    ) {
        let state = state_with_the_card_at_one_opened(
            None,
            Some(every_gpu_left_out_on_the_card(b"0000:01:00.0")),
        );
        let mut background_compute = mount_on_the_card_at_one(&state);
        assert!(
            background_compute
                .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation),
            "挂载那一组一张卡都用不上：挂载就停着"
        );
        state
            .lock()
            .expect("用例没在别处 panic 过")
            .failure_once_put_in_effect = None;
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        assert!(!background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
        let batch_while_opening = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert!(
            batch_while_opening.is_ok(),
            "交给实现那一刻实现还在开卡：不读切换之前那一组的用不上、不再停"
        );
        state.lock().expect("用例没在别处 panic 过").opening_ends = true;
        let batch_once_opened = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert!(batch_once_opened.is_ok(), "换上之后用得上：照常开批");
        assert!(!background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
    }

    /// 第 4 条「挂着切换到参与的卡一张都不剩的设置……报的是挂载那一刻同一个成员（GpuUnusableUnderTheSettingsHandedIn）」用于点没开的卡：
    /// 请求切到没开的「0000:41:00.0」、给它装不下最小的一片的上限——开卡的这段时间照旧开批；卡开成、实现答换上了而一张卡都用不上，
    /// 那一批开头按这个成员停着、不开批。
    #[test]
    fn the_switch_put_in_effect_after_the_opening_with_every_card_left_out_stops_as_unusable_and_begins_no_batch(
    ) {
        let state = state_with_the_card_at_one_opened(None, None);
        let mut background_compute = mount_on_the_card_at_one(&state);
        state
            .lock()
            .expect("用例没在别处 panic 过")
            .failure_once_put_in_effect = Some(every_gpu_left_out_on_the_card(b"0000:41:00.0"));
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", Some(64)),
                None,
            )
            .expect("上限不超过显存总量：照收");
        let batch_while_opening = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert!(
            batch_while_opening.is_ok(),
            "开卡的这段时间照切换之前那一组开批"
        );
        state.lock().expect("用例没在别处 panic 过").opening_ends = true;
        let batch_once_opened = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert!(
            batch_once_opened.is_err(),
            "换上的那一组一张卡都用不上：不开批"
        );
        assert_eq!(
            background_compute
                .stopped_after_a_compute_error()
                .iter()
                .map(|stopped| stopped.error.clone())
                .collect::<Vec<_>>(),
            vec![
                BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
                    failure: every_gpu_left_out_on_the_card(b"0000:41:00.0"),
                }
            ],
            "报的是挂载那一刻同一个成员"
        );
    }

    /// 对照：同样一张卡都用不上的设置，点的是挂载那一刻开着的「0000:01:00.0」——交给实现那一刻就换上、当场按同一个成员停着、不开批。
    #[test]
    fn the_switch_to_every_card_left_out_on_a_card_already_open_stops_at_the_handing_in() {
        let state = state_with_the_card_at_one_opened(None, None);
        let mut background_compute = mount_on_the_card_at_one(&state);
        state
            .lock()
            .expect("用例没在别处 panic 过")
            .failure_once_put_in_effect = Some(every_gpu_left_out_on_the_card(b"0000:01:00.0"));
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:01:00.0", Some(64)),
                None,
            )
            .expect("上限不超过显存总量：照收");
        let batch_at_the_handing_in = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert!(batch_at_the_handing_in.is_err());
        assert!(background_compute
            .stopped_after_a_compute_error()
            .iter()
            .any(|stopped| matches!(
                stopped.error,
                BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn { .. }
            )));
    }

    /// 第 2 条「从 gpu 切到 cpu……关掉全部开着的卡」：挂载点「0000:01:00.0」，切到 `cpu`——那一批开头告诉实现一张卡都不参与（替身关掉开着的卡），那一批在 CPU 上算。
    #[test]
    fn the_switch_to_the_cpu_tells_the_implementation_that_no_gpu_takes_part() {
        let state = state_with_the_card_at_one_opened(None, None);
        let mut background_compute = mount_on_the_card_at_one(&state);
        background_compute
            .request_switch_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
            .expect("选 cpu 照收");
        let batch_on_the_cpu = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            route_of_the_batch(&batch_on_the_cpu),
            Some(BackgroundComputeRoute::Cpu(
                ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings
            ))
        );
        let state_after_the_switch = state.lock().expect("用例没在别处 panic 过");
        assert_eq!(state_after_the_switch.times_told_no_gpu_takes_part, 1);
        assert!(
            state_after_the_switch.cards_opened.is_empty(),
            "开着的卡全部关掉"
        );
    }

    /// 切到 `cpu`、开着的卡全部关掉之后切回 `gpu`：点的卡都当没开的卡开——开卡的这段时间生效的仍是 `cpu`，那一批在 CPU 上算；
    /// 开成、实现答换上了之后，下一批起在 GPU 上算。
    #[test]
    fn the_switch_back_to_the_gpu_runs_on_the_cpu_while_the_cards_open_and_on_the_gpu_once_put_in_effect(
    ) {
        let state = state_with_the_card_at_one_opened(None, None);
        let mut background_compute = mount_on_the_card_at_one(&state);
        background_compute
            .request_switch_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
            .expect("选 cpu 照收");
        let _batch_on_the_cpu = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:01:00.0", None),
                None,
            )
            .expect("挂载时递进来的实现还在：切回 gpu 照收");
        let batch_while_opening = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            route_of_the_batch(&batch_while_opening),
            Some(BackgroundComputeRoute::Cpu(
                ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings
            )),
            "开卡的这段时间生效的仍是 cpu：在 CPU 上算"
        );
        state.lock().expect("用例没在别处 panic 过").opening_ends = true;
        let batch_once_opened = background_compute.begin_a_batch(
            BackgroundTask::CompactionRelocation,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            route_of_the_batch(&batch_once_opened),
            Some(BackgroundComputeRoute::Gpu),
            "开成之后下一批起在 GPU 上算"
        );
    }

    // ── 实现照用例逐批摆的答法（两批之间那一问答什么由用例定），钉核心层在开不成、开卡期间出错、重复键三处的结局 ──

    /// 替身的脚本：两批之间那一问答 `answer`；告知一张卡都不参与、被问那一问各记一次。
    struct ScriptOfTheImplementation {
        answer: LatestGpuSettingsHandedIn,
        times_told_no_gpu_takes_part: u32,
        times_asked_between_two_batches: u32,
    }

    /// 卡清单同 [`GpuComputeOpeningCardsWhenTheTestSays`]（「0000:01:00.0」「0000:41:00.0」），三段交回空，两批之间那一问照脚本答。
    struct GpuComputeAnsweringAsTheScriptSays {
        script: Arc<Mutex<ScriptOfTheImplementation>>,
    }

    impl GpuInventory for GpuComputeAnsweringAsTheScriptSays {
        fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
            vec![
                (
                    card_for_the_switch_tests(b"0000:01:00.0"),
                    GpuMemoryInBytes(8 << 30),
                ),
                (
                    card_for_the_switch_tests(b"0000:41:00.0"),
                    GpuMemoryInBytes(8 << 30),
                ),
            ]
        }
    }

    impl BackgroundGpuCompute for GpuComputeAnsweringAsTheScriptSays {
        fn verify_unit_checksums(
            &mut self,
            _units: &[UnitChecksumToVerifyOnTheGpu<'_>],
        ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed>
        {
            Ok(Vec::new())
        }

        fn classify_region_records(
            &mut self,
            _region: CompactionRegion,
            _records: &[RegionRecordToClassifyOnTheGpu<'_>],
            _mapping_entries: &[CentralMappingEntry],
        ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
            Ok(Vec::new())
        }

        fn count_live_slots_per_cluster_segment(
            &mut self,
            _allocation_records: &[AllocationRecord],
            _unit_area: UnitAreaOfOneDevice,
            _segments: &[ClusterSegmentEntryKey],
        ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
            Ok(Vec::new())
        }

        fn use_the_settings_from_the_next_batch(&mut self, _settings: &AdmittedGpuSettings) {}

        fn use_no_gpu_from_the_next_batch(&mut self) {
            self.script
                .lock()
                .expect("用例没在别处 panic 过")
                .times_told_no_gpu_takes_part += 1;
        }

        fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
            &mut self,
        ) -> LatestGpuSettingsHandedIn {
            let mut script = self.script.lock().expect("用例没在别处 panic 过");
            script.times_asked_between_two_batches += 1;
            script.answer.clone()
        }
    }

    fn script_answering(
        answer: LatestGpuSettingsHandedIn,
    ) -> Arc<Mutex<ScriptOfTheImplementation>> {
        Arc::new(Mutex::new(ScriptOfTheImplementation {
            answer,
            times_told_no_gpu_takes_part: 0,
            times_asked_between_two_batches: 0,
        }))
    }

    fn answer_from_now_on(
        script: &Arc<Mutex<ScriptOfTheImplementation>>,
        answer: LatestGpuSettingsHandedIn,
    ) {
        script.lock().expect("用例没在别处 panic 过").answer = answer;
    }

    fn put_in_effect_and_usable() -> LatestGpuSettingsHandedIn {
        LatestGpuSettingsHandedIn::PutInEffect {
            failure_under_the_settings: None,
        }
    }

    /// 选 `gpu`、点「0000:01:00.0」的一次挂载，替身照 `script` 答。
    fn mount_on_the_card_at_one_answering_as_the_script_says(
        script: &Arc<Mutex<ScriptOfTheImplementation>>,
    ) -> BackgroundComputeOfThisMount {
        let handle =
            BackgroundGpuComputeHandle::of_an_implementation(GpuComputeAnsweringAsTheScriptSays {
                script: Arc::clone(script),
            });
        BackgroundComputeOfThisMount::of_a_mount(
            admit_background_compute_settings_with_the_gpu_compute(
                gpu_settings_naming_the_card(b"0000:01:00.0", None),
                Some(handle),
            )
            .expect("卡清单列着「0000:01:00.0」"),
        )
    }

    fn not_opened_on_the_card_at_forty_one() -> BackgroundGpuComputeFailed {
        BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings {
            device: card_for_the_switch_tests(b"0000:41:00.0"),
            failures_of_each_opening: Vec::new(),
            pci_state_when_judged_unusable: Box::new(
                PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
            ),
        }
    }

    /// 池子最满的那一块盘 95%，过了默认 85% 的补跑阈值：停着就在 CPU 上补跑。
    fn fullest_device_above_the_default_threshold() -> AllocatedBytesAndCapacityOfADevice {
        AllocatedBytesAndCapacityOfADevice {
            device: DeviceIdentity(0),
            allocated_bytes: 95,
            capacity_bytes: 100,
        }
    }

    fn route_of_the_next_batch(
        background_compute: &mut BackgroundComputeOfThisMount,
        fullest_device: AllocatedBytesAndCapacityOfADevice,
    ) -> Option<BackgroundComputeRoute> {
        route_of_the_batch(
            &background_compute.begin_a_batch(BackgroundTask::CompactionRelocation, fullest_device),
        )
    }

    /// 一段历史在开不成之后的读数：开不成那一批走哪条路、开不成之后生效的那一组点哪张卡、再请求同一组之后那一批走哪条路。
    struct ReadingsOfTheRetryAfterNotOpened {
        times_told_no_gpu_takes_part: u32,
        route_once_not_opened: Option<BackgroundComputeRoute>,
        devices_in_effect_once_not_opened: Option<Vec<GpuDeviceIdentifier>>,
        route_at_the_retry: Option<BackgroundComputeRoute>,
        handed_in_at_the_retry: bool,
    }

    /// 挂载点「0000:01:00.0」；`switch_to_the_cpu_first` 时先切到 `cpu`；请求切到没开的「0000:41:00.0」，实现答还在开卡、之后答开不成；
    /// 再请求同一组（停着时的切换：当场解除），实现答还在开卡。
    fn readings_of_the_retry_after_not_opened(
        switch_to_the_cpu_first: bool,
    ) -> ReadingsOfTheRetryAfterNotOpened {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        if switch_to_the_cpu_first {
            background_compute
                .request_switch_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
                .expect("选 cpu 照收");
            let _route_on_the_cpu = route_of_the_next_batch(
                &mut background_compute,
                fullest_device_far_below_the_threshold(),
            );
        }
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let _route_while_opening = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        answer_from_now_on(
            &script,
            LatestGpuSettingsHandedIn::NotOpened {
                failure: not_opened_on_the_card_at_forty_one(),
            },
        );
        let route_once_not_opened = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        let devices_in_effect_once_not_opened = background_compute
            .in_effect()
            .settings()
            .background_gpu_devices
            .clone();
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let route_at_the_retry = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        let times_told_no_gpu_takes_part = script
            .lock()
            .expect("用例没在别处 panic 过")
            .times_told_no_gpu_takes_part;
        ReadingsOfTheRetryAfterNotOpened {
            times_told_no_gpu_takes_part,
            route_once_not_opened,
            devices_in_effect_once_not_opened,
            route_at_the_retry,
            handed_in_at_the_retry: background_compute
                .handed_in_while_the_implementation_opens_its_cards()
                .is_some(),
        }
    }

    /// 实现答开不成时核心层不把那一组记成生效：先切到 `cpu`、切回的卡开不成、再请求同一组——开不成那一批停着、生效的仍是 `cpu`，
    /// 再请求之后实现还在开卡，那一批在 CPU 上算（不交给一张卡都没开的实现）。
    #[test]
    fn a_switch_not_opened_after_the_cpu_leaves_the_cpu_in_effect_and_the_retry_runs_on_the_cpu() {
        let readings = readings_of_the_retry_after_not_opened(true);
        assert_eq!(readings.times_told_no_gpu_takes_part, 1);
        assert_eq!(readings.route_once_not_opened, None, "开不成：停着、不开批");
        assert_eq!(
            readings.devices_in_effect_once_not_opened, None,
            "开不成的那一组不生效：生效的仍是 cpu"
        );
        assert_eq!(
            readings.route_at_the_retry,
            Some(BackgroundComputeRoute::Cpu(
                ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings
            )),
            "再请求之后实现还在开卡：照生效的 cpu 算"
        );
        assert!(readings.handed_in_at_the_retry);
    }

    /// 同一段历史不先切到 `cpu`：开不成之后核心层生效的仍是切换之前那张「0000:01:00.0」——与实现照旧参与的是同一组；再请求之后那一批照它在 GPU 上算。
    #[test]
    fn a_switch_not_opened_on_the_gpu_leaves_the_card_before_the_switch_in_effect() {
        let readings = readings_of_the_retry_after_not_opened(false);
        assert_eq!(readings.times_told_no_gpu_takes_part, 0);
        assert_eq!(readings.route_once_not_opened, None, "开不成：停着、不开批");
        assert_eq!(
            readings.devices_in_effect_once_not_opened,
            Some(vec![card_for_the_switch_tests(b"0000:01:00.0")]),
            "开不成的那一组不生效：生效的仍是切换之前那一组"
        );
        assert_eq!(
            readings.route_at_the_retry,
            Some(BackgroundComputeRoute::Gpu)
        );
        assert!(readings.handed_in_at_the_retry);
    }

    /// 实现答开不成之后，补跑那一判照那一次切换请求给的阈值：挂载取默认 85%，请求切到「0000:41:00.0」给 50%，开不成——
    /// 池子最满的那一块盘 60%，下一批在 CPU 上补跑。
    #[test]
    fn the_gc_cpu_compensation_threshold_of_a_switch_not_opened_is_the_one_the_request_gave() {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        background_compute
            .request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
                BackgroundGcCpuCompensationThreshold {
                    allocated_percent_of_the_capacity: 50,
                },
            )
            .expect("卡清单列着「0000:41:00.0」、阈值不超过 100");
        answer_from_now_on(
            &script,
            LatestGpuSettingsHandedIn::NotOpened {
                failure: not_opened_on_the_card_at_forty_one(),
            },
        );
        let fullest_device_at_sixty_percent = AllocatedBytesAndCapacityOfADevice {
            device: DeviceIdentity(0),
            allocated_bytes: 60,
            capacity_bytes: 100,
        };
        let route =
            route_of_the_next_batch(&mut background_compute, fullest_device_at_sixty_percent);
        assert!(
            matches!(
                route,
                Some(BackgroundComputeRoute::Cpu(
                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
                ))
            ),
            "照请求给的 50% 判：60% 超过，补跑：{route:?}"
        );
    }

    /// 开不成那一组请求给的阈值只管到生效的那一组换掉为止：请求切到「0000:41:00.0」给 50%、开不成；再请求切到 `cpu` 给 90%，它交给实现那一刻生效——
    /// 之后补跑那一判照 90%，不再照开不成那一组留下的 50%。
    #[test]
    fn the_gc_cpu_compensation_threshold_left_by_a_switch_not_opened_is_dropped_once_another_switch_is_put_in_effect(
    ) {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        background_compute
            .request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
                BackgroundGcCpuCompensationThreshold {
                    allocated_percent_of_the_capacity: 50,
                },
            )
            .expect("卡清单列着「0000:41:00.0」、阈值不超过 100");
        answer_from_now_on(
            &script,
            LatestGpuSettingsHandedIn::NotOpened {
                failure: not_opened_on_the_card_at_forty_one(),
            },
        );
        let _route_once_not_opened = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        background_compute
            .request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
                BackgroundComputeSettings::PRODUCT,
                None,
                BackgroundGcCpuCompensationThreshold {
                    allocated_percent_of_the_capacity: 90,
                },
            )
            .expect("选 cpu、阈值不超过 100：照收");
        let _route_on_the_cpu = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            background_compute
                .gc_cpu_compensation_reading(fullest_device_far_below_the_threshold())
                .threshold,
            BackgroundGcCpuCompensationThreshold {
                allocated_percent_of_the_capacity: 90,
            },
            "此刻生效的是 cpu 那一组：照它的 90%"
        );
    }

    /// 交给了实现、实现还在开卡的期间每一批都在切换之前那张卡上出错，池子最满的那一块盘过了补跑阈值：头一批出错停下，之后每次叫起报警、
    /// 每一批在 CPU 上补跑；实现答换上了的那一刻解除，那一批在 GPU 上算。
    #[test]
    fn batches_failing_while_the_switched_card_opens_stop_alarm_and_compensate_until_it_is_put_in_effect(
    ) {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        background_compute.begin_a_wake();
        let route_of_the_failing_batch = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_above_the_default_threshold(),
        );
        let after_the_error = background_compute.stop_the_task_after_a_compute_error(
            BackgroundTask::CompactionRelocation,
            failure_on_the_card_at_one(),
        );
        let _alarm_of_the_failing_wake =
            background_compute.alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation);
        background_compute.begin_a_wake();
        let route_while_stopped = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_above_the_default_threshold(),
        );
        let alarm_while_stopped =
            background_compute.alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation);
        assert_eq!(
            route_of_the_failing_batch,
            Some(BackgroundComputeRoute::Gpu)
        );
        assert_eq!(
            after_the_error,
            BackgroundTaskAfterTheComputeError::LeftStoppedWhileTheSwitchOpensItsCards
        );
        assert!(
            matches!(
                route_while_stopped,
                Some(BackgroundComputeRoute::Cpu(
                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
                ))
            ),
            "停着、过了阈值：在 CPU 上补跑：{route_while_stopped:?}"
        );
        assert_eq!(
            alarm_while_stopped
                .map(|alarm| (alarm.wakes_while_stopped, alarm.error_that_stopped_the_task)),
            Some((1, failure_on_the_card_at_one())),
            "停下之后的下一次叫起报警"
        );
        answer_from_now_on(&script, put_in_effect_and_usable());
        let route_once_put_in_effect = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_above_the_default_threshold(),
        );
        assert_eq!(
            route_once_put_in_effect,
            Some(BackgroundComputeRoute::Gpu),
            "实现答换上了：解除，照新的那一组在 GPU 上算"
        );
        assert!(!background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
    }

    /// 开卡期间出错停着、之后实现答开不成：照开不成停着，报的是开不成那一个错。
    #[test]
    fn a_batch_failing_while_the_switched_card_opens_stays_stopped_by_the_not_opened_failure_once_not_opened(
    ) {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let _route_of_the_failing_batch = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        let _after_the_error = background_compute.stop_the_task_after_a_compute_error(
            BackgroundTask::CompactionRelocation,
            failure_on_the_card_at_one(),
        );
        answer_from_now_on(
            &script,
            LatestGpuSettingsHandedIn::NotOpened {
                failure: not_opened_on_the_card_at_forty_one(),
            },
        );
        let route_once_not_opened = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(route_once_not_opened, None, "照开不成停着");
        assert_eq!(
            background_compute
                .stopped_after_a_compute_error()
                .iter()
                .map(|stopped| stopped.error.clone())
                .collect::<Vec<_>>(),
            vec![
                BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
                    failure: not_opened_on_the_card_at_forty_one(),
                }
            ]
        );
    }

    /// 对照：同一个错、同样过了补跑阈值，没有切换——停下（`LeftStopped`），下一批在 CPU 上补跑。
    #[test]
    fn the_same_error_without_a_switch_is_left_stopped_and_compensates_on_the_cpu() {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        let _route_of_the_failing_batch = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_above_the_default_threshold(),
        );
        let after_the_error = background_compute.stop_the_task_after_a_compute_error(
            BackgroundTask::CompactionRelocation,
            failure_on_the_card_at_one(),
        );
        let route_of_the_next_batch_after_the_error = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_above_the_default_threshold(),
        );
        assert_eq!(
            after_the_error,
            BackgroundTaskAfterTheComputeError::LeftStopped
        );
        assert!(matches!(
            route_of_the_next_batch_after_the_error,
            Some(BackgroundComputeRoute::Cpu(
                ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
            ))
        ));
    }

    /// 一批里同一个条目键出现两次只放弃这一批：那个后台任务不停、叫起不报警，下一批照常在 GPU 上算。
    #[test]
    fn the_same_entry_key_twice_in_one_batch_abandons_only_the_batch_without_stopping_or_alarming()
    {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        background_compute.begin_a_wake();
        let _route_of_the_abandoned_batch = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        let after_the_error = background_compute.stop_the_task_after_a_compute_error(
            BackgroundTask::CompactionRelocation,
            BackgroundComputeError::GpuSegmentFailed {
                segment: GpuComputedSegment::LiveSlotsPerClusterSegment,
                failure: BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch {
                    position_of_the_first_occurrence: 1,
                    position_of_the_second_occurrence: 3,
                },
            },
        );
        let alarm =
            background_compute.alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation);
        let route_of_the_next_batch_after_the_error = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            after_the_error,
            BackgroundTaskAfterTheComputeError::NotStoppedForOnlyTheBatchIsAbandoned
        );
        assert!(!background_compute
            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
        assert_eq!(alarm, None, "不报警");
        assert_eq!(
            route_of_the_next_batch_after_the_error,
            Some(BackgroundComputeRoute::Gpu),
            "下一批照常算"
        );
    }

    /// 交给了实现、还在开卡的那一组被一次 `cpu` 的切换盖掉：交 `cpu` 那一刻作废还在开卡的那一格，之后不再为它问实现。
    #[test]
    fn a_switch_to_the_cpu_drops_the_settings_whose_cards_are_still_opening() {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        background_compute
            .request_switch_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
            )
            .expect("卡清单列着「0000:41:00.0」");
        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let _route_while_opening = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        background_compute
            .request_switch_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
            .expect("选 cpu 照收");
        let route_on_the_cpu = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            route_on_the_cpu,
            Some(BackgroundComputeRoute::Cpu(
                ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings
            ))
        );
        assert!(
            background_compute
                .handed_in_while_the_implementation_opens_its_cards()
                .is_none(),
            "交 cpu 那一刻作废还在开卡的那一组"
        );
    }

    /// 开卡期间出错停着时，补跑那一判照交给了实现、还在开卡的那一组请求给的阈值：挂载取默认 85%，请求切到「0000:41:00.0」给 50%，开卡期间出错——
    /// 池子最满的那一块盘 60%，下一批在 CPU 上补跑。
    #[test]
    fn the_gc_cpu_compensation_threshold_while_the_switch_opens_its_cards_is_the_one_the_request_gave(
    ) {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        background_compute
            .request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
                gpu_settings_naming_the_card(b"0000:41:00.0", None),
                None,
                BackgroundGcCpuCompensationThreshold {
                    allocated_percent_of_the_capacity: 50,
                },
            )
            .expect("卡清单列着「0000:41:00.0」、阈值不超过 100");
        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
        let fullest_device_at_sixty_percent = AllocatedBytesAndCapacityOfADevice {
            device: DeviceIdentity(0),
            allocated_bytes: 60,
            capacity_bytes: 100,
        };
        let _route_of_the_failing_batch =
            route_of_the_next_batch(&mut background_compute, fullest_device_at_sixty_percent);
        let _after_the_error = background_compute.stop_the_task_after_a_compute_error(
            BackgroundTask::CompactionRelocation,
            failure_on_the_card_at_one(),
        );
        let route =
            route_of_the_next_batch(&mut background_compute, fullest_device_at_sixty_percent);
        assert!(
            matches!(
                route,
                Some(BackgroundComputeRoute::Cpu(
                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
                ))
            ),
            "照还在开卡的那一组给的 50% 判：60% 超过，补跑：{route:?}"
        );
    }

    /// 没有交给了实现、还没生效的那一组时，一批开头照样问一次生效那一组带着的实现（让它收下为更早几次交设置开完的卡），答案不用：
    /// 挂载那一刻问 1 次，下一批开头再问 1 次。
    #[test]
    fn a_batch_begun_with_no_settings_waiting_still_asks_the_implementation_once() {
        let script = script_answering(put_in_effect_and_usable());
        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
        let asked_at_the_mount = script
            .lock()
            .expect("用例没在别处 panic 过")
            .times_asked_between_two_batches;
        let _route = route_of_the_next_batch(
            &mut background_compute,
            fullest_device_far_below_the_threshold(),
        );
        assert_eq!(
            (
                asked_at_the_mount,
                script
                    .lock()
                    .expect("用例没在别处 panic 过")
                    .times_asked_between_two_batches
            ),
            (1, 2)
        );
    }

    /// 一份读到的 PCI 状态：目录照 `device_directory_reading`、使能位照 `device_enable`；链路此刻降到 5.0 GT/s ×4（最大 32.0 GT/s ×16），
    /// 可纠正的出错计数不是 0，致命的那份不在、不致命的那份读不通。
    fn pci_state_of_the_card_at_one(
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

    /// 第 4 条「只有卡不在总线上（那个 PCI 地址的目录不在了，或使能位是关的）才判硬件坏」：目录不在就判不在总线上，使能位读没读到都一样。
    #[test]
    fn the_gpu_whose_device_directory_is_not_present_is_judged_off_the_bus() {
        assert_eq!(
            hardware_judgement_of_the_pci_state(&pci_state_of_the_card_at_one(
                PciDeviceDirectoryReading::NotPresent,
                PciFileReading::FileNotPresent,
            )),
            HardwareJudgementOfTheGpu::OffTheBus(
                ReasonTheGpuIsOffTheBus::DeviceDirectoryNotPresent
            )
        );
    }

    /// 同一条：目录在、使能位读到 0（关的）判不在总线上。
    #[test]
    fn the_gpu_whose_device_enable_reads_disabled_is_judged_off_the_bus() {
        assert_eq!(
            hardware_judgement_of_the_pci_state(&pci_state_of_the_card_at_one(
                PciDeviceDirectoryReading::Present,
                PciFileReading::Read(PciDeviceEnable::Disabled),
            )),
            HardwareJudgementOfTheGpu::OffTheBus(ReasonTheGpuIsOffTheBus::DeviceDisabled)
        );
    }

    /// 同一条「链路速度与宽度、出错计数只报不判」：目录在、使能位开着，链路降速降宽、出错计数不是 0，也不判坏。
    #[test]
    fn the_gpu_enabled_with_a_degraded_link_and_errors_counted_is_not_judged_off_the_bus() {
        assert_eq!(
            hardware_judgement_of_the_pci_state(&pci_state_of_the_card_at_one(
                PciDeviceDirectoryReading::Present,
                PciFileReading::Read(PciDeviceEnable::Enabled { enable_count: 1 }),
            )),
            HardwareJudgementOfTheGpu::NotJudgedOffTheBus
        );
    }

    /// 同一条「读不到的那一样报读不到，不当成 0」：目录在，使能位文件不在、读不通、原文认不得，都不判坏（不当成读到了关的）。
    #[test]
    fn the_gpu_whose_device_enable_is_not_read_is_not_judged_off_the_bus() {
        let judgements: Vec<HardwareJudgementOfTheGpu> = [
            PciFileReading::FileNotPresent,
            PciFileReading::NotReadable {
                system_error: "替身：读不通".to_string(),
            },
            PciFileReading::Read(PciDeviceEnable::UnrecognizedText {
                text: "替身：认不得".to_string(),
            }),
        ]
        .into_iter()
        .map(|device_enable| {
            hardware_judgement_of_the_pci_state(&pci_state_of_the_card_at_one(
                PciDeviceDirectoryReading::Present,
                device_enable,
            ))
        })
        .collect();
        assert_eq!(
            judgements,
            vec![HardwareJudgementOfTheGpu::NotJudgedOffTheBus; 3]
        );
    }

    /// 同一条：目录读不通（不是不在）不判坏，照使能位判。
    #[test]
    fn the_gpu_whose_device_directory_is_not_readable_is_judged_by_its_device_enable() {
        assert_eq!(
            hardware_judgement_of_the_pci_state(&pci_state_of_the_card_at_one(
                PciDeviceDirectoryReading::NotReadable {
                    system_error: "替身：打不开".to_string(),
                },
                PciFileReading::Read(PciDeviceEnable::Enabled { enable_count: 2 }),
            )),
            HardwareJudgementOfTheGpu::NotJudgedOffTheBus
        );
    }

    /// 第 4 条「停着之后每次叫起交回的报警带停下那一刻读的同一份读数，不重读」：一批因「0000:01:00.0」重交用完停下（错里带判它用不了那一刻读的一份），
    /// 停下那一趟与之后三次叫起的报警都带那一份，与停下记录里那个错带的相同。
    // harness-test-granularity:one-scenario 同一份停着的状态上按次序叫起几次，钉的是每一次报警带的都是停下那一刻的同一份
    #[test]
    fn the_alarm_at_every_wake_while_stopped_carries_the_pci_state_read_when_the_gpu_was_judged_unusable(
    ) {
        let pci_state_read =
            PciStateOfTheGpuReadWhenJudgedUnusable::Read(Box::new(pci_state_of_the_card_at_one(
                PciDeviceDirectoryReading::Present,
                PciFileReading::Read(PciDeviceEnable::Enabled { enable_count: 1 }),
            )));
        let error = BackgroundComputeError::GpuSegmentFailed {
            segment: GpuComputedSegment::UnitChecksumVerification,
            failure: BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
                device: card_for_the_switch_tests(b"0000:01:00.0"),
                resubmissions: 3,
                wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                last_submission_waited_for: SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
                pci_state_when_judged_unusable: Box::new(pci_state_read.clone()),
            },
        };
        let mut background_compute =
            BackgroundComputeOfThisMount::of_a_mount(AdmittedBackgroundComputeSettings::PRODUCT);
        background_compute.begin_a_wake();
        assert_eq!(
            background_compute
                .stop_the_task_after_a_compute_error(BackgroundTask::CompactionRelocation, error),
            BackgroundTaskAfterTheComputeError::LeftStopped
        );
        let pci_states_of_the_alarms: Vec<Option<PciStateOfTheGpuReadWhenJudgedUnusable>> = (0..4)
            .map(|wake| {
                if wake > 0 {
                    background_compute.begin_a_wake();
                }
                background_compute
                    .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
                    .map(|alarm| alarm.pci_state_of_the_gpu_judged_unusable())
            })
            .collect();
        assert_eq!(
            pci_states_of_the_alarms,
            vec![Some(pci_state_read.clone()); 4],
            "停下那一趟与之后每次叫起的报警都带停下那一刻读的那一份"
        );
        assert_eq!(
            background_compute.stopped_after_a_compute_error()[0]
                .error
                .pci_state_when_judged_unusable(),
            pci_state_read,
            "停下记录里的那个错带的是同一份"
        );
    }

    /// 第 4 条：不是判一张卡用不了的那三种时刻、或不归哪张卡的错不读，报的是没读那个显式成员：只供测试的钩子注入的停下之后报警照报、读数那一格是没读；
    /// 一批里同一个条目键出现两次、一次等超过上限（还没用完重交）、参与的卡一张都不剩的错带的也是没读。
    #[test]
    fn errors_not_judging_a_gpu_unusable_carry_no_pci_state_read() {
        let mut background_compute =
            BackgroundComputeOfThisMount::of_a_mount(AdmittedBackgroundComputeSettings::PRODUCT);
        assert_eq!(
            background_compute.stop_the_task_after_a_compute_error(
                BackgroundTask::CompactionRelocation,
                BackgroundComputeError::InjectedByTheTestOnlyHook,
            ),
            BackgroundTaskAfterTheComputeError::LeftStopped
        );
        background_compute.begin_a_wake();
        assert_eq!(
            background_compute
                .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
                .map(|alarm| alarm.pci_state_of_the_gpu_judged_unusable()),
            Some(PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError)
        );
        let failures_not_judging_a_gpu_unusable = [
            BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch {
                position_of_the_first_occurrence: 0,
                position_of_the_second_occurrence: 1,
            },
            BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit {
                device: card_for_the_switch_tests(b"0000:01:00.0"),
                wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                submission_waited_for: SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
            },
            every_gpu_left_out_on_the_card(b"0000:01:00.0"),
        ];
        assert_eq!(
            failures_not_judging_a_gpu_unusable
                .iter()
                .map(BackgroundGpuComputeFailed::pci_state_when_judged_unusable)
                .collect::<Vec<_>>(),
            vec![PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError; 3]
        );
    }
}
