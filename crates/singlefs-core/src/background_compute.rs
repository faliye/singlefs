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
//! （用户 2026-10-26 定；[`BackgroundComputeError`]）。

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use crate::address::{DeviceIdentity, SlotNumber};
use crate::allocator::AllocationRecord;
use crate::compaction_plan::UnitAreaOfOneDevice;
use crate::logged_operations::CompactionRegion;
use crate::region_scan::{AllocationInRegion, CentralMappingEntry};

/// 后台任务在哪算（挂载参数 `background_compute`，第 1 条）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundComputeBackend {
    Cpu,
    Gpu,
}

/// 计算可以放在别处的后台任务种类（第 1 条「凡有 GPU 路径的后台任务」）。封闭集合：里程碑七的 GPU 打包搬迁
/// （`.claude/kb/milestone/07-seventh-txn.md`「三　GPU 打包搬迁」）加一个成员，挂载参数、校验与切换都不改。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundTask {
    /// 整理（GC）的搬迁：`mounted_session` 里整理每推一次发布算这一种任务的一批。
    CompactionRelocation,
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

    /// [`Self::devices_with_total_memory`] 里挂载那一刻没开成的那几张与原因（`records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 45 批交回那一行的定法：
    /// 给了 `background_gpu_devices` 而没点名的卡开不了，挂载跳过它、照挂）。参与的卡里有它就拒（第 4 条，挂载与切换同一判：
    /// [`BackgroundComputeSettingsRefused::ParticipatingGpuDeviceNotOpenedAtTheMount`]）。
    /// 不开卡的清单（测试里的假清单、开卡之前那一刻的枚举）没有这一样，交回空。
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
/// 成员按停下之后用户要做的决定分（修好或不再点那张卡、调大等 GPU 的上限、切回 `cpu`），出在哪张卡、什么原因带在成员里。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackgroundGpuComputeFailed {
    /// 这张卡出错（起不了、设备丢了、等不成、回读坏了、核留下认不出的判定字）：修好它或切换时不再点它，或切回 `cpu`。
    GpuFailed {
        device: GpuDeviceIdentifier,
        cause: GpuFailureOnTheDevice,
    },
    /// 等这张卡超过了 `background_gpu_wait_limit`（用户 2026-10-26 定「设超时，超时算出错」）：调大上限，或查这张卡是不是卡死了。
    WaitForTheGpuExceededTheLimit {
        device: GpuDeviceIdentifier,
        wait_limit: Duration,
        submission_waited_for: SubmissionWaitedForPastTheLimit,
    },
    /// 这一批的输入 GPU 那一侧第一版不支持，一张卡都没交活：切回 `cpu`（装不进一段的那一种也可以调大显存上限）。
    InputNotSupportedOnTheGpu(InputNotSupportedOnTheGpu),
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
    /// 一条（甲段的单元、乙段的记录、丙段的段）单独成一段就超过分到它的那张卡此刻剩下的显存或绑定上限。`entry_position` 是它在这一批里的位置，
    /// `input_bytes` 是它自己的输入字节数。
    EntryLargerThanAChunkOnTheDevice {
        device: GpuDeviceIdentifier,
        entry_position: usize,
        input_bytes: u64,
    },
    /// 每一段都要交一份的表（乙段的映射、丙段的分配记录表）最大的那个缓冲装不进分到活的那张卡的绑定上限。
    TablesLargerThanABindingOnTheDevice {
        device: GpuDeviceIdentifier,
        largest_buffer_bytes: u64,
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

    /// 从下一批起照这一组设置跑（第 1–3 条）：只在 `settings` 点的那几张卡上、按挂载时在每张卡上量出的代价分活，每张卡上合占的显存不超过
    /// 给它的上限，每一处等 GPU 不超过等的上限。核心层在挂载那一刻（选 `gpu` 时）与每次通过校验的切换生效的那一批开头、开算之前调，
    /// 一批里三段都算完之前不再调（一批之内不换）。实现不为它重开设备，三段的连续不确定计数照旧留着（第 8 条：只在重挂时清零）。
    /// `settings` 的卡都在这份实现答的卡清单里、都不是它报的挂载那一刻没开成的（核心层照这份清单校验过）。
    fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings);
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

/// 一批在 GPU 上算了什么：三段各自的条目数，与甲段判成整单元 CRC-32C 对不上的单元数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GpuWorkOfTheBatch {
    pub unit_checksum_verification: GpuEntryCounts,
    pub region_record_classification: GpuEntryCounts,
    pub live_slots_per_cluster_segment: GpuEntryCounts,
    /// 甲段确认或交 CPU 判出整单元 CRC-32C 与位置项对不上的可搬单元数。它们照第 8 条交 CPU：发布路径照常逐条核计划、释放之前读盘核
    /// （那一核读出对不上的那一份隔离，D19（块指针的结构与宽度预算） 已定项 5），区域扫描不因它改分类，结局与 CPU 路径相同；这里只报。
    pub units_whose_checksum_mismatched_on_the_gpu: u64,
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

/// 等 GPU 交回一批结果的上限的默认：10 秒。默认值是推的，待量：岔路单 `research/prompts/m4-gpugc-design-r1-forks.md` 引的 E185（GC一批搬迁的挂钟拆分）
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
    /// 一批交给 GPU 之后至多等它多久（用户 2026-10-26 定「设超时，超时算出错」：过了这个上限还没交回，照第 4 条「挂着的时候 GPU 出错」办，
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

/// 一组设置在挂载或切换那一刻被拒（第 4 条：拒，不降级到 cpu）。成员按调用方要改哪一样分：第 4 条点名的四种配错
/// （GPU 不在、点名的卡不存在、上限个数与卡数不等、上限超过显存总量），条款没写、第一版不支持的两种点法（空卡列表、同一张卡点两次），
/// 等 GPU 结果的上限给了 0，与参与的卡里有一张挂载那一刻没开成。
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
}

/// 校验过的一组设置，连同它校验时用的那一份 GPU 实现：只由 [`admit_background_compute_settings_with_the_gpu_compute`] 造出，
/// 产品默认另有 [`Self::PRODUCT`]。选 `gpu` 的那一组恒带着实现与交给实现的那一组（没给实现那一刻就拒了），一批照它算。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedBackgroundComputeSettings {
    settings: BackgroundComputeSettings,
    gpu_compute: Option<BackgroundGpuComputeHandle>,
    /// 选 `gpu` 时恒有：照实现答的卡清单校验出的参与的卡、上限与等的上限；选 `cpu` 时恒无。
    gpu_settings: Option<AdmittedGpuSettings>,
}

impl AdmittedBackgroundComputeSettings {
    /// [`BackgroundComputeSettings::PRODUCT`]：选 `cpu` 照收、不校验，不带实现。
    pub const PRODUCT: AdmittedBackgroundComputeSettings = AdmittedBackgroundComputeSettings {
        settings: BackgroundComputeSettings::PRODUCT,
        gpu_compute: None,
        gpu_settings: None,
    };

    #[must_use]
    pub fn settings(&self) -> &BackgroundComputeSettings {
        &self.settings
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

    /// 这一组带着的 GPU 实现：选 `gpu` 时恒有；选 `cpu` 时是挂载或切换时递进来的那一份（以后切到 `gpu` 没另给实现时用它），没给是 `None`。
    #[must_use]
    pub fn gpu_compute(&self) -> Option<&BackgroundGpuComputeHandle> {
        self.gpu_compute.as_ref()
    }
}

/// 挂载与挂着切换那一刻的校验（第 4 条）：选 `gpu` 而没给实现就是 GPU 不在，拒；给了就照实现答的卡清单走 [`admit_background_compute_settings`]。
/// 选 `cpu` 时照收、不校验，给了实现也带着。只读卡清单，不碰盘。
///
/// # Errors
/// [`BackgroundComputeSettingsRefused`] 的八种。
pub fn admit_background_compute_settings_with_the_gpu_compute(
    settings: BackgroundComputeSettings,
    gpu_compute: Option<BackgroundGpuComputeHandle>,
) -> Result<AdmittedBackgroundComputeSettings, BackgroundComputeSettingsRefused> {
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
    })
}

/// 第 4 条选 `gpu` 那一支的校验（`settings.background_compute` 不看，照选了 `gpu` 判，判法就是 [`admit_background_compute_settings`] 选 `gpu` 那一支），
/// 过了交回交给实现的那一组：参与的卡按上限对应的次序、各带上限，与等的上限。卡清单只读一次，判与取参与的卡读的是同一份。只读卡清单，不碰盘。
///
/// # Errors
/// [`BackgroundComputeSettingsRefused`] 的八种。
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
/// [`BackgroundComputeSettingsRefused`] 的八种。
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

/// 一种后台任务因一批出错停着（第 4 条：正在算的那一批放弃、意图留在盘上，那个后台任务停下并报错，不换回 CPU）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackgroundTaskStoppedAfterAComputeError {
    pub task: BackgroundTask,
    pub error: BackgroundComputeError,
}

/// 一批出错之后那个后台任务怎样了（[`BackgroundComputeOfThisMount::stop_the_task_after_a_compute_error`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundTaskAfterTheComputeError {
    /// 停着，等一次通过校验的切换请求解除（用户 2026-10-26 定「停着时收到切换请求就当场解除停下」）。
    LeftStopped,
    /// 出错之前就挂着一次待生效的切换：它在停下那一刻也算停着时收到的切换请求，当场解除停下，从下一批起用它（用户同日定）。
    StopLiftedByTheSwitchRequestedBeforeTheError,
}

/// 后台计算在这一次挂载里的状态：此刻生效的设置，与挂着时请求、从下一批开头起用的那一组；因一批出错停着的后台任务。
/// 生命周期一次挂载，只住内存。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackgroundComputeOfThisMount {
    in_effect: AdmittedBackgroundComputeSettings,
    requested_from_the_next_batch: Option<AdmittedBackgroundComputeSettings>,
    stopped_after_a_compute_error: Vec<BackgroundTaskStoppedAfterAComputeError>,
    /// 只供测试的钩子在这一批里注入的错，这一批读完它就清掉（[`Self::take_the_compute_error_of_the_batch_under_way`]）。
    compute_error_of_the_batch_under_way: Option<BackgroundComputeError>,
}

impl BackgroundComputeOfThisMount {
    /// 挂载那一刻：挂载给的那一组（校验过）生效，没有待生效的，没有停着的后台任务。选 `gpu` 时当场把这一组交给它带着的实现
    /// （[`BackgroundGpuCompute::use_the_settings_from_the_next_batch`]），头一批起照它跑。
    #[must_use]
    pub fn of_a_mount(admitted_at_the_mount: AdmittedBackgroundComputeSettings) -> Self {
        admitted_at_the_mount.hand_the_gpu_settings_to_the_implementation();
        Self {
            in_effect: admitted_at_the_mount,
            requested_from_the_next_batch: None,
            stopped_after_a_compute_error: Vec::new(),
            compute_error_of_the_batch_under_way: None,
        }
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
    /// 下一批开头照常换上它；没有就把 `task` 记成停着，等下一次通过校验的切换请求。不读写盘。
    pub(crate) fn stop_the_task_after_a_compute_error(
        &mut self,
        task: BackgroundTask,
        error: BackgroundComputeError,
    ) -> BackgroundTaskAfterTheComputeError {
        if self.requested_from_the_next_batch.is_some() {
            return BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError;
        }
        if !self.is_stopped_after_a_compute_error(task) {
            self.stopped_after_a_compute_error
                .push(BackgroundTaskStoppedAfterAComputeError { task, error });
        }
        BackgroundTaskAfterTheComputeError::LeftStopped
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

    /// 挂着切换（第 1 条，一次切全部后台任务）：与挂载同一套校验（[`admit_background_compute_settings_with_the_gpu_compute`]）；过了就记成待生效，
    /// 下一批开头起用，正在算的这一批照旧；同一批开头之前请求了几次，用最后一次。切到 `gpu` 要有实现：这一次递进来的，没递就沿用这次挂载
    /// 已有的那一份（待生效的那一组带的，没有再看生效的那一组带的）；都没有就是 GPU 不在，拒。有后台任务因一批出错停着时，过了校验的这一次当场解除
    /// 全部停下（用户 2026-10-26 定「停着时收到切换请求就当场解除停下」；第 4 条「恢复要人切回 `cpu` 或修好 GPU 再切」），从下一批起用新设置。
    ///
    /// # Errors
    /// 校验不过（第 4 条）：拒绝切换，生效的与待生效的都留着原样，停着的照旧停着。
    pub fn request_switch_from_the_next_batch(
        &mut self,
        settings: BackgroundComputeSettings,
        gpu_compute: Option<BackgroundGpuComputeHandle>,
    ) -> Result<(), BackgroundComputeSettingsRefused> {
        let gpu_compute_of_this_mount = gpu_compute
            .or_else(|| {
                self.requested_from_the_next_batch
                    .as_ref()
                    .and_then(|requested| requested.gpu_compute.clone())
            })
            .or_else(|| self.in_effect.gpu_compute.clone());
        let admitted = admit_background_compute_settings_with_the_gpu_compute(
            settings,
            gpu_compute_of_this_mount,
        )?;
        self.requested_from_the_next_batch = Some(admitted);
        self.stopped_after_a_compute_error.clear();
        Ok(())
    }

    /// 一批开头（每批读一次设置的那一处）：有待生效的就换上，换上的是 `gpu` 的就在开算之前把它交给带着的实现（从这一批起照它跑，
    /// 一批之内不再交）；再按生效的那一组定这一批走哪条路。
    ///
    /// # Panics
    /// 上一批注入的错没被读走：注入只在一批的钩子里，那一批读完设置之后就读走。
    pub(crate) fn begin_a_batch(&mut self, task: BackgroundTask) -> BackgroundBatchUnderWay {
        assert!(
            self.compute_error_of_the_batch_under_way.is_none(),
            "只供测试注入的错只在一批的钩子里注入、那一批就读走：一批开头不该还挂着上一批的"
        );
        if let Some(requested) = self.requested_from_the_next_batch.take() {
            self.in_effect = requested;
            self.in_effect.hand_the_gpu_settings_to_the_implementation();
        }
        let requested_backend = self.in_effect.settings.background_compute;
        BackgroundBatchUnderWay {
            settings_read_at_the_start_of_the_batch: self.in_effect.clone(),
            task,
            requested_backend,
            route: route_of_the_task(task, requested_backend),
        }
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
