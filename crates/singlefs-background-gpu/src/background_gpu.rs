//! 起 GPU 的外壳（D26（后台整理与放置回收） 已定项 13 第 1–4、8 条）：挂载那一刻枚举 wgpu 适配器、判过校验、在本机每一张独立显卡上起设备与各段两份核的管线、
//! 量一次提交的主机 CPU 时间；一张卡没开成时，它是参与的卡就拒绝挂载，给了 `background_gpu_devices` 而没点它就跳过、报出来
//! （[`BackgroundGpuOverTheCards::gpus_not_opened_at_the_mount`]；判法在 `crate::gpus_of_this_host::gpus_opened_at_the_mount`，`records/2026-10-19-里程碑四GPU垃圾回收开工.md`
//! 第 45 批交回那一行的定法）。给了 `background_gpu_devices` 时只判点名的卡：没点名的独显缺总线地址、缺设备本地堆、与别的独显报了同一个地址都不拒，
//! 不收进卡清单、报出来（[`BackgroundGpuOverTheCards::unnamed_discrete_gpus_left_off_the_list`]，判法 `crate::adapter_inventory::discrete_gpus_of_the_enumeration_under_the_named`）。
//! 参与的卡由设置定（[`BackgroundGpuOverTheCards::participating_gpus`] 报出来），挂着切换时核心层交来新的一组
//! （[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]，两批之间），从下一批起只在它点的卡上、照它给的显存上限与等的上限跑，不重开设备
//! （`records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 43 批交回那一行的定法；分活、切段、等的纯逻辑在 `crate::gpus_of_this_host`）。
//! 挂着的时候每一批按卡的顺序填满参与的卡（前面的卡填到它照显存上限一次放得下的量，装不下的才给后面的卡；挂载时量的代价不用来分活）、按显存上限切段、两份核各算一次、逐条比对。三段都接上了：扫描里的校验和核对（[`BackgroundGpuOverTheCards::verify_unit_checksums`]）、
//! 区域扫描里的单元头解析与映射反查（[`BackgroundGpuOverTheCards::classify_region_records`]）与选段打分的每段活槽数（[`BackgroundGpuOverTheCards::count_live_slots_per_cluster_segment`]）。
//! 本机的卡清单（核心层 `GpuInventory`）答的是此刻本机在的独显与各自的显存总量（按总线地址排，挂载那一刻没开成的照列；每次答之前现枚举一次，
//! 挂载之后掉了的卡不列，切换点它照第 4 条「GPU 不在」拒），连同挂载那一刻没开成的那几张与原因（挂载那一刻定下，答它不现枚举：一次切换只现枚举一次）：
//! 挂载与切换照它校验设置，切换可以点开包时没参与而开成了的卡；
//! 点名不在本机的卡就是点名的卡不存在，参与的卡里有没开成的照第 4 条拒。
//!
//! 外壳分两层：一张开着的卡要做的几样（[`CardOfTheBackgroundGpu`]：交活之前先等前一次超时的那次提交、重交之前等、交一段两份核）由 [`GpuInUse`]
//! （wgpu 设备与六条管线）实现；几张卡上怎么分活、切段、重交、超时按批按卡计数、建缓冲显存不足时重切、丢的时候泄漏谁，在 [`BackgroundGpuOverTheCards`]
//! 里对任何一种卡都一样，测试换上在 CPU 上算的替身卡就钉得住，不起 GPU。产品那一份是 [`BackgroundGpu`]。
//!
//! GPU 出错（设备丢了、wgpu 报了没接住的错、等不成、回读映射不成、核没写到某一条）时这一批交回出错、不换 CPU（第 4 条）；一次提交等超过上限的，
//! 在同一张卡上重交，这一批在这张卡上（一批切成几片、几段时合起来算，交活之前等前一次超时的那次提交那一等也算）第 4 次等超过上限才交回「这张卡用不了」
//! （`crate::waiting_for_the_gpu::TimeoutsOfTheBatchOnOneGpu`，用户 2026-10-28 定；重交的单位照旧只是超时的那一片，用户 2026-10-30 定）。
//! 出错时不动连续不确定的计数（这一批放弃了，不算一轮）。只有一张卡时两份核都在这张卡上跑；几张卡时每张卡在自己分到的那几条上跑两份核。
//! 每一处阻塞等 GPU 都在此刻设置给的上限之内等（开包时是 `BackgroundGpuSelection::background_gpu_wait_limit`，等法在 `crate::waiting_for_the_gpu`）：
//! 等 wgpu 的 future 用 std 写的最小等法（[`wait_for_the_wgpu_future`]），等提交用 wgpu 自己带限时的 `poll`，不依赖 `pollster`
//! （门禁 checker-implementation-disjoint 格第 ⑧ 条）。
//! 每一段切批之前先扣掉这张卡上此刻占着的显存（`ChunkBoundsOfAGpu::with_the_room_left_after`），几段合在一张卡上占的不超过它的上限（第 3 条）。
//! 一段的提交等超过上限之后，它的缓冲在卡上可能还占着：这一段的账留到那次提交真在 GPU 上做完才放，下一批在这张卡上切段交活之前先在上限之内
//! 等它做完（`crate::memory_limit_and_chunks` 文件头）。
//! 装不下就切小：单元切成几片（校验和那一段）、映射按落点切成几片（区域扫描那一段）、记录表按记录切成几片（选段打分那一段），逐片交；
//! 一张卡这一批连最小的一片都切不出（显存上限扣掉已占的）就这一批不分给它、活分给别的卡，参与的卡全都切不出才报
//! （`crate::gpus_of_this_host::plans_of_a_batch_filling_the_gpus_in_order`；用户 2026-10-28 定）。
//! 卡上建缓冲时显存不足（空闲的比上限少，第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」）：这张卡这一批的地方压到建不成的那一段的一半、
//! 重切、从头再交它分到的那几条，不报出错；一路压到连最小的一片都建不成才报（条款没写，第一版不支持）。
//! 挂载那一刻量代价那一段也按开卡时的上限切片，代价取各片之和。
//! 丢后台 GPU 时不等还在 GPU 上跑的提交：那张卡的设备与队列泄漏，别的卡照常丢；泄漏了哪几张、各占着多少报给调用方
//! （[`BackgroundGpuOverTheCards::gpus_leaked_when_dropped`]），并记进进程里的那一份（`crate::gpus_of_this_host::GpuMemoryLeakedInThisProcess`），
//! 新一次挂载这张卡的可用余量先扣掉它。

use std::future::Future;
use std::ops::Range;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::time::Duration;

use singlefs_core::allocator::AllocationRecord;
use singlefs_core::background_compute::{
    AdmittedGpuSettings, BackgroundComputeSettingsRefused, ClusterSegmentEntryKey,
    GpuComputedSegment, GpuDeviceIdentifier, GpuDeviceNotOpenedAtTheMount, GpuInventory,
    GpuMemoryInBytes, GpuShortOfFreeMemoryForTheSmallestPiece, GpuSubmissionsOfOneSegment,
    GpuSubmissionsOfTheBatch,
};
use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};

use crate::adapter_inventory::{
    discrete_gpus_of_the_enumeration_under_the_named,
    identifiers_of_the_discrete_gpus_in_the_enumeration,
    participating_gpus_admitted_by_the_mount_checks, AdapterDeviceKind, BackgroundGpuSelection,
    DiscreteGpuListRefused, DiscreteGpuOnThisHost, DiscreteGpusPresentNow,
    EnumeratedAdapterDescription,
};
use crate::checksum_kernels::{
    checksum_outcomes_of_the_units_cut_into_pieces, decoded_kernel_outcomes,
    outcome_buffer_before_the_kernel_writes, packed_checksum_chunk, reconciled_unit_checksums,
    units_of_the_pieces, ChecksumKernelOutcome, UnitChecksumToVerify, UnrecognizedVerdictWord,
    BITWISE_KERNEL_ENTRY_POINT, BITWISE_KERNEL_SOURCE, CHECKSUM_CHUNK_FOOTPRINT,
    INVOCATIONS_PER_WORKGROUP, OUTCOME_BYTES_PER_ENTRY, PARAMETER_BUFFER_BYTES,
    TABLE_DRIVEN_KERNEL_ENTRY_POINT, TABLE_DRIVEN_KERNEL_SOURCE,
};
use crate::gpus_of_this_host::{
    bytes_of_the_smallest_piece_every_participating_gpu_holds, gpu_left_out_under_its_memory_limit,
    gpus_dropped_leaking_those_still_running, gpus_opened_at_the_mount,
    measured_cost_summed_over_the_pieces, memory_limit_of_the_device_under_the_settings,
    plans_of_a_batch_filling_the_gpus_in_order, BatchNotPlannedOverTheGpus, CostAtTheOpening,
    CostProbeChunk, DiscreteGpuNotOpenedAtTheMount, DiscreteGpusAfterTheOpening,
    GpuBudgetUnderTheSettings, GpuLeakedWhenDropped, GpuLeftOutUnderItsMemoryLimit,
    GpuMemoryLeakedInThisProcess, GpusLeakedWhenTheBackgroundGpuIsDropped,
    ParticipatingGpusOnThisHost, PlanOfAGpuForTheBatch, RoomOfAGpuForTheBatch,
};
use crate::live_slot_kernels::{
    decoded_live_slot_outcomes, live_slot_chunk_footprint,
    live_slot_outcome_buffer_before_the_kernel_writes, live_slot_parameter_bytes,
    live_slot_plan_on_a_gpu, live_slots_summed_over_the_record_slices,
    reconciled_live_slots_of_the_segments, segment_bound_words,
    AllocationRecordTablesForTheKernels, RecordSliceWithItsChunks,
    LIVE_SLOT_INVOCATIONS_PER_WORKGROUP, LIVE_SLOT_OUTCOME_BYTES,
    SEARCHING_SORTED_RECORDS_KERNEL_ENTRY_POINT, SEARCHING_SORTED_RECORDS_KERNEL_SOURCE,
    SEGMENT_BOUND_BYTES, WALKING_EVERY_RECORD_KERNEL_ENTRY_POINT,
    WALKING_EVERY_RECORD_KERNEL_SOURCE,
};
use crate::measured_host_cpu_time_per_submission::HostCpuNanosecondsPerSubmission;
use crate::memory_limit_and_chunks::{
    input_buffer_bytes, pieces_cut_to_fit_on_the_gpu, ChunkBoundsOfAGpu,
    SmallestPieceLargerThanTheRoom, SubmissionOfTheChunk,
};
use crate::region_record_kernels::{
    decoded_region_record_outcomes, decoded_registers_after_the_pieces,
    little_endian_bytes_of_the_words, packed_region_record_chunk,
    reconciled_region_record_classifications, record_is_one_the_region_scan_reads,
    region_record_chunk_footprint, region_record_classifications_of_the_plan,
    region_record_outcome_buffer_before_the_kernel_writes, region_record_parameter_bytes,
    region_record_plan_on_a_gpu, BothKernelOutcomesOfARegionRecordChunk,
    CrcRegistersOfTheTwoKernels, MappingKeyLongerThanTheKernelsHold, MappingTablesForTheKernels,
    RegionRecordPieceForTheKernels, RegionRecordPlanOnAGpu, RegionRecordToClassify,
    REGION_RECORD_INVOCATIONS_PER_WORKGROUP, REGION_RECORD_OUTCOME_BYTES,
    SEARCHING_KERNEL_ENTRY_POINT, SEARCHING_KERNEL_SOURCE, WALKING_KERNEL_ENTRY_POINT,
    WALKING_KERNEL_SOURCE,
};
use crate::two_kernel_reconciliation::{ConsecutiveUncertaintyCounts, ReconciledBatch};
use crate::waiting_for_the_gpu::{
    outcome_of_opening_a_gpu_within_the_limit, outcome_resubmitted_after_timeouts,
    outcome_waited_for_again_after_timeouts, wait_for_completion_before_the_deadline,
    CompletionCheck, GpuWaitDeadline, MonotonicClock, SteadyClockOfThisProcess,
    TimeoutsOfTheBatchOnOneGpu, WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
};

/// 各段的着色器按字节偏移用 `u32` 寻址、整字那一段按「剩下的字节 ≥ 4」走：一个绑定不超过 2^31 字节，偏移加 4 也不回绕。
const LARGEST_STORAGE_BINDING_THE_KERNELS_ADDRESS_IN_BYTES: u64 = 1 << 31;
/// 挂载那一刻量代价交的那一段：一条 4096 字节全 0 的单元（大小取一个常见的单元块；它只用来量一次提交的主机 CPU，结果不用），走校验和核对那一段的缓冲。
/// 量的时候照这张卡开卡时的界切成几片逐段交（[`CostProbeChunk::pieces_under_the_bounds`]，与挂着时校验和那一段同一个切法），代价取各片之和
/// （[`measured_cost_summed_over_the_pieces`]）；真建的缓冲与 `CHECKSUM_CHUNK_FOOTPRINT` 一致（[`CardOfTheBackgroundGpu::both_kernel_outcomes_of_a_checksum_chunk`] 核过）。
pub const COST_PROBE_CHUNK: CostProbeChunk = CostProbeChunk {
    input_bytes: 4096,
    footprint: CHECKSUM_CHUNK_FOOTPRINT,
};
/// 量几次取中位数：先空跑一次（管线刚编好时驱动的懒初始化不算进去），再量这么多次。
const COST_MEASUREMENT_SUBMISSIONS: usize = 5;
/// 两份核的结果缓冲：核写、拷给回读暂存。
const OUTCOME_BUFFER_USAGE: wgpu::BufferUsages =
    wgpu::BufferUsages::STORAGE.union(wgpu::BufferUsages::COPY_SRC);

/// 一次 GPU 出错是什么（第 4 条）：调用方的决定只有一个——这一批放弃、后台任务停下并报错，成员只为报得清。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GpuFailureCause {
    /// 起不了设备或编不了管线（驱动不对、显存不够）。
    DeviceOrPipelineDidNotStart { reason: String },
    /// wgpu 报了没接住的错（校验错）或设备丢了。
    UncapturedErrorOrDeviceLost { reason: String },
    /// 等这一次提交没等成（不是超时）。
    WaitForTheSubmissionFailed { reason: String },
    /// 等 GPU（等 wgpu 的 future、等一次提交做完、等回读映射好）超过了开包时给的上限（用户 2026-10-26 定「设超时，超时算出错」）。
    WaitForTheGpuExceededTheLimit { wait_limit: Duration },
    /// 这张卡上前一次等超过上限的那次提交，下一批在这张卡上交活之前又等满了上限还没在 GPU 上做完：它的缓冲还占着卡上的显存，
    /// 不越过上限去交（第 3、4 条）。
    EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { wait_limit: Duration },
    /// 回读缓冲映射不成。
    ReadbackMappingFailed { reason: String },
    /// 读回的判定字认不得：核没写到那一条或写坏了。
    KernelLeftAnUnrecognizedVerdictWord(UnrecognizedVerdictWord),
    /// 在卡上建一段的缓冲时显存不足（wgpu 在显存不足的错误范围里报的），`bytes_held_by_the_chunk` 是那一段合占的字节数；这一段没交，它的账已经放掉。
    /// 挂着时外壳按更小的片重切重交，切到最小的一片都建不成就这一批不用这张卡、活转给别的卡（[`BackgroundGpuOverTheCards`]），不报出来。
    OutOfMemoryCreatingTheBuffers {
        bytes_held_by_the_chunk: u64,
        reason: String,
    },
    /// 一批在一张卡上第 4 次等超过上限（`waiting_for_the_gpu::RESUBMISSIONS_AFTER_A_TIMEOUT` 加 1；一批切成几片时各片的超时合起来算，
    /// 交活之前、重交之前等前一次那一次没等到也算一次），`resubmissions` 是这期间真交到卡上的重交次数（重交之前那一等没等到、
    /// 重交那一次建缓冲显存不足没交出去的不算，所以可以比 3 少）：这张卡用不了，
    /// 交核心层当报警（用户 2026-10-28 定「都超时才报警」）。`last_cause` 是最后那一次的原因（等超过上限的那两种之一）。
    UnusableAfterResubmissions {
        resubmissions: u32,
        last_cause: Box<GpuFailureCause>,
    },
}

/// 挂载那一刻开不了后台 GPU（第 4 条：拒绝挂载并报错，不降级、不退回 CPU）。成员按调用方要改哪一样分。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackgroundGpuOpenRefused {
    /// 挂载参数配错或 GPU 不在：核心层那一套校验拒的（改参数，或装上卡）。
    SettingsRefused(BackgroundComputeSettingsRefused),
    /// 枚举到的独立显卡交不成一份卡清单（条款没写的形态，第一版不支持）。
    DiscreteGpuListNotSupported(DiscreteGpuListRefused),
    /// 枚举适配器这一步就没成（等 wgpu 交回适配器列表超过了上限）：还没有哪一张卡可点名。
    AdapterEnumerationFailed { cause: GpuFailureCause },
    /// 一张参与的卡用不上（起不了设备、编不了管线、量代价那几次提交出错或超时）：给了 `background_gpu_devices` 时是点名的卡，没给时本机任何一张独显。
    /// 给了列表而没点名的卡用不上不拒，跳过（[`BackgroundGpu::gpus_not_opened_at_the_mount`]）。显存上限连最小的一片都装不下的卡也不拒，
    /// 不用它、报出来（[`BackgroundGpu::gpus_left_out_under_their_memory_limit`]）。
    GpuUnusable {
        device: GpuDeviceIdentifier,
        cause: GpuFailureCause,
    },
}

/// 一张参与的卡连这一批最小的一片都装不下（用户 2026-10-28 定「连一片都装不下就不用这张卡」）：哪张卡、
/// 那一片要切的条目在这一批里的位置、给它的显存上限（不给是 `None`：最小的一片装不进绑定上限）、
/// 最小的一片合占的字节数（这张卡至少要剩这么多）。这一批一张卡都还没交活。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmallestPieceNotHeldOnTheGpu {
    pub device: GpuDeviceIdentifier,
    pub entry_position: usize,
    pub memory_limit: Option<GpuMemoryInBytes>,
    pub smallest_piece_bytes: u64,
}

/// 一批校验和核对交不成。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChecksumBatchFailed {
    /// GPU 出错（含一次提交在同一张卡上重交 3 次都等超过上限、前一次超时的提交在交这一批之前又没等到做完、换设置时量代价出错）：
    /// 这一批放弃、不换 CPU（第 4 条）。
    GpuFailed {
        device: GpuDeviceIdentifier,
        cause: GpuFailureCause,
    },
    /// 最小的一片（一个字节的单元）都装不下分到它的那张卡。
    GpuCannotHoldTheSmallestPiece(SmallestPieceNotHeldOnTheGpu),
    /// 这组设置点的卡的显存上限连最小的一片都装不下，参与的卡一张都不剩：按参与的次序列出每张卡、上限与至少要多少。一张卡都没交活。
    EveryParticipatingGpuLeftOutUnderItsMemoryLimit(Vec<GpuLeftOutUnderItsMemoryLimit>),
    /// 这一批参与的卡一张都不剩，其中至少一张是卡上空闲的显存连最小的一片都建不成（用户 2026-10-28 定「连一片都装不下就不用这张卡」：
    /// 这一批不用它，它还没交成的活转给这一批别的参与的卡重交，全都不剩才是这一种）：因空闲显存不用的按先后列出，连同这一段因显存上限不用的。
    EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
        gpus_short_of_free_memory: Vec<GpuShortOfFreeMemoryForTheSmallestPiece>,
        gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
    },
}

/// 一批区域扫描的单元头解析与映射反查交不成。后三种在一张卡都还没交活之前判出。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegionRecordBatchFailed {
    /// GPU 出错（同 [`ChecksumBatchFailed::GpuFailed`]）：这一批放弃、不换 CPU（第 4 条）。
    GpuFailed {
        device: GpuDeviceIdentifier,
        cause: GpuFailureCause,
    },
    /// 映射里有一条 key 长过核装得下的字节数（第一版不支持：核心层写的 key 都是 27 字节）。
    MappingKeyLongerThanTheKernelsHold(MappingKeyLongerThanTheKernelsHold),
    /// 最小的一片都装不下分到它的那张卡（切开的单元：头一片连同一个落点上的全部条目，或头一片之后最小的一片；没切开的：最大那条记录连同一个落点上的全部条目）。
    GpuCannotHoldTheSmallestPiece(SmallestPieceNotHeldOnTheGpu),
    /// 同 [`ChecksumBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit`]。
    EveryParticipatingGpuLeftOutUnderItsMemoryLimit(Vec<GpuLeftOutUnderItsMemoryLimit>),
    /// 同 [`ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`]。
    EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
        gpus_short_of_free_memory: Vec<GpuShortOfFreeMemoryForTheSmallestPiece>,
        gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
    },
}

/// 一批选段打分交不成。后两种在一张卡都还没交活之前判出。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveSlotBatchFailed {
    /// GPU 出错（同 [`ChecksumBatchFailed::GpuFailed`]）：这一批放弃、不换 CPU（第 4 条）。
    GpuFailed {
        device: GpuDeviceIdentifier,
        cause: GpuFailureCause,
    },
    /// 最小的一片（一个段连同一条记录的记录表）都装不下分到它的那张卡。
    GpuCannotHoldTheSmallestPiece(SmallestPieceNotHeldOnTheGpu),
    /// 同 [`ChecksumBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit`]。
    EveryParticipatingGpuLeftOutUnderItsMemoryLimit(Vec<GpuLeftOutUnderItsMemoryLimit>),
    /// 同 [`ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`]。
    EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
        gpus_short_of_free_memory: Vec<GpuShortOfFreeMemoryForTheSmallestPiece>,
        gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
    },
}

/// 此刻这组设置下一批分不到卡上（[`BackgroundGpu::batch_not_split_over_the_gpus`]）：换设置时量一张卡的代价出错了（这张卡没参与，报成它的 GPU 出错），
/// 或参与的卡因显存上限一张都不剩。这组设置下每一批三段都交回它，直到下一次换设置。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BatchNotSplitOverTheGpus {
    CostNotMeasuredAtTheSwitch {
        device: GpuDeviceIdentifier,
        cause: GpuFailureCause,
    },
    /// 按参与的次序列出每张被晾下的卡、给它的上限与至少要多少，至少一张。
    EveryParticipatingGpuLeftOut(Vec<GpuLeftOutUnderItsMemoryLimit>),
}

/// 一张参与的卡对外报的样子（第 2 条：参与的卡都用，并报出来）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticipatingGpuReport {
    pub identifier: GpuDeviceIdentifier,
    pub name: String,
    pub memory_limit: Option<GpuMemoryInBytes>,
    pub measured_cost: HostCpuNanosecondsPerSubmission,
}

/// 外壳里一张开着的卡要做的几样（产品是 [`GpuInUse`]，测试换上在 CPU 上算的替身卡）：报它是哪张卡、此刻设置给它的界与它上面的账、量一次提交的代价，
/// 交活之前与重交之前先等它上面前一次等超过上限的那次提交，交一段两份核（三段各一种）。每一样的约定同 [`GpuInUse`] 上对应的那一个。
/// 一段交成了就照那一段的账放掉；等超过上限的那一段的账留着、记成还在跑的那次提交（`GpuMemoryAccount::settle_the_chunk_after_its_submission`）；
/// 建缓冲显存不足的那一段不交、账放掉、交回 [`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]。
pub trait CardOfTheBackgroundGpu {
    /// 一次提交的把手（等超过上限之后还在卡上跑的那一次记在账上）。
    type Submission;

    fn discrete_gpu(&self) -> &DiscreteGpuOnThisHost;

    fn budget(&self) -> &GpuBudgetUnderTheSettings<Self::Submission>;

    fn budget_mut(&mut self) -> &mut GpuBudgetUnderTheSettings<Self::Submission>;

    /// 量过的一次提交的主机 CPU 时间；开卡时因显存上限不用、还没量过的是 `None`。
    fn measured_cost(&self) -> Option<HostCpuNanosecondsPerSubmission>;

    /// 照此刻的界量一次提交的主机 CPU 时间并记下（之后 [`Self::measured_cost`] 交回它）。
    ///
    /// # Errors
    /// 交出错照原样交回，不记。
    fn measure_the_cost_of_one_submission(&mut self) -> Result<(), GpuFailureCause>;

    /// 交活之前：前一次等超过上限、还在跑的那次提交在此刻设置给的等的上限之内等一回、做完就放掉它的账，交回此刻剩下的界。
    ///
    /// # Errors
    /// 等不到它做完（等满上限是 [`GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit`]），账照旧记着。
    fn room_once_the_submission_running_past_its_wait_completed(
        &mut self,
    ) -> Result<ChunkBoundsOfAGpu, GpuFailureCause>;

    /// 重交之前：同 [`Self::room_once_the_submission_running_past_its_wait_completed`]，不交回界。
    ///
    /// # Errors
    /// 同上。
    fn wait_for_the_timed_out_submission(&mut self) -> Result<(), GpuFailureCause>;

    /// 校验和核对的一段（切批切出来的几片，各当一条单元）交两份核，交回每片两份的结果（查表那一份在前），按段里的次序。
    ///
    /// # Errors
    /// GPU 出错、等超过上限、建缓冲显存不足。
    fn both_kernel_outcomes_of_a_checksum_chunk(
        &mut self,
        units: &[UnitChecksumToVerify<'_, ()>],
    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, GpuFailureCause>;

    /// 区域扫描的一段（几片）交两份核（`mapping_tables` 是这一段交的那一片映射，头一片之后那几片的段是 `None`），约定同
    /// `region_record_kernels::region_record_classifications_of_the_plan` 交给它的那一个。
    ///
    /// # Errors
    /// 同 [`Self::both_kernel_outcomes_of_a_checksum_chunk`]。
    fn both_kernel_outcomes_of_a_region_record_chunk(
        &mut self,
        pieces: &[RegionRecordPieceForTheKernels<'_>],
        mapping_tables: Option<&MappingTablesForTheKernels>,
    ) -> Result<BothKernelOutcomesOfARegionRecordChunk, GpuFailureCause>;

    /// 选段打分的一段（几个段，连同这一片记录的两张表）交两份核，交回每段两份的活槽数（逐条走遍记录那一份在前），按段里的次序。
    ///
    /// # Errors
    /// 同 [`Self::both_kernel_outcomes_of_a_checksum_chunk`]。
    fn both_kernel_live_slots_of_a_chunk(
        &mut self,
        segments: &[ClusterSegmentEntryKey],
        record_tables: &AllocationRecordTablesForTheKernels,
        unit_area: UnitAreaOfOneDevice,
    ) -> Result<Vec<(u64, u64)>, GpuFailureCause>;
}

/// 这一次挂载的后台 GPU 在几张开着的卡上（`Card` 是一张开着的卡，产品是 [`GpuInUse`]）：开包时开成的本机独显（按总线地址排），没开成而跳过的那几张，
/// 给了点名时没点名而交不成一张卡、没收进清单的那几张，与此刻参与的那几张（设置定的，按参与的次序：没点名是总线地址的次序，点了名是点名的次序；
/// 显存上限连最小的一片都装不下的不在里面）。生命周期一次挂载，只住内存。丢它时不等还在 GPU 上跑的提交（`Drop`）。
pub struct BackgroundGpuOverTheCards<Card: CardOfTheBackgroundGpu> {
    /// 开成的那几张，与 `discrete_gpus_after_the_opening.opened` 一一对应、同序。
    cards_on_this_host: Vec<Card>,
    discrete_gpus_after_the_opening: DiscreteGpusAfterTheOpening,
    unnamed_discrete_gpus_left_off_the_list: Vec<DiscreteGpuListRefused>,
    participating: ParticipatingGpusOnThisHost,
    /// 此刻这组设置点了而因显存上限装不下最小的一片不用的卡，按参与的次序。
    left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
    /// 最近一次换设置时量一张新参与的卡的代价出错（这张卡没参与）：之后每一批都报成它的 GPU 出错，直到下一次换设置。
    cost_not_measured_at_the_last_switch: Option<(GpuDeviceIdentifier, GpuFailureCause)>,
    /// 上一次交回以来三段各自在卡上是怎么交的（[`Self::take_the_submissions_of_the_batch`]）。
    submissions_since_the_last_take: GpuSubmissionsOfTheBatch,
    gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped,
    memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
    /// 此刻本机在的独显（卡清单每次答之前现查）。产品里它带着枚举出卡的那个 wgpu 实例：排在卡后面，卡都丢了才丢它。
    discrete_gpus_present_now: Box<dyn DiscreteGpusPresentNow + Send>,
}

/// 产品那一份：开着的卡是 wgpu 设备与六条管线。
pub type BackgroundGpu = BackgroundGpuOverTheCards<GpuInUse>;

/// 一张开着的 wgpu 卡（产品的 [`CardOfTheBackgroundGpu`]）。
pub struct GpuInUse {
    discrete_gpu: DiscreteGpuOnThisHost,
    device: wgpu::Device,
    queue: wgpu::Queue,
    table_driven_pipeline: wgpu::ComputePipeline,
    bitwise_pipeline: wgpu::ComputePipeline,
    walking_pipeline: wgpu::ComputePipeline,
    searching_pipeline: wgpu::ComputePipeline,
    live_slot_walking_pipeline: wgpu::ComputePipeline,
    live_slot_searching_pipeline: wgpu::ComputePipeline,
    /// 此刻设置给这张卡的显存上限与等的上限（一次提交里的几处等合起来不超过它），连同本包在它上面占的显存与等超过上限之后还在 GPU 上跑的那次提交。
    budget: GpuBudgetUnderTheSettings<wgpu::SubmissionIndex>,
    /// wgpu 的回调（没接住的错、设备丢了）记在这里；每次等完提交先看它。
    fault_reported_by_wgpu: Arc<Mutex<Option<String>>>,
    /// 一次提交的主机 CPU 时间；开卡时因显存上限不用、还没量过的是 `None`（换设置让它参与时再量）。
    measured_cost: Option<HostCpuNanosecondsPerSubmission>,
}

/// 一批里一张卡上交成了几次、合起来几片，与这一批这张卡的超时次数（按批、按卡数：`TimeoutsOfTheBatchOnOneGpu`）。
pub struct SubmissionsOfACardInTheBatch {
    pub timeouts: TimeoutsOfTheBatchOnOneGpu,
    pub submissions_completed: u64,
    pub pieces_submitted: u64,
}

impl SubmissionsOfACardInTheBatch {
    /// 一批开头：一次都还没交。
    #[must_use]
    pub fn none_yet() -> Self {
        Self {
            timeouts: TimeoutsOfTheBatchOnOneGpu::none_yet(),
            submissions_completed: 0,
            pieces_submitted: 0,
        }
    }

    /// 在这张卡上交一段（`pieces` 片）：等超过上限在同一张卡上重交，超时记进这一批这张卡的那一份
    /// （`waiting_for_the_gpu::outcome_resubmitted_after_timeouts`）；交成了记一次、记片数。
    ///
    /// # Errors
    /// 同 `outcome_resubmitted_after_timeouts`。
    pub fn outcome_of_a_chunk<Card: CardOfTheBackgroundGpu, Outcome>(
        &mut self,
        card: &mut Card,
        pieces: usize,
        submit_once: impl FnMut(&mut Card) -> Result<Outcome, GpuFailureCause>,
    ) -> Result<Outcome, GpuFailureCause> {
        let outcome = outcome_resubmitted_after_timeouts(
            card,
            &mut self.timeouts,
            Card::wait_for_the_timed_out_submission,
            submit_once,
        )?;
        self.submissions_completed += 1;
        self.pieces_submitted += u64::try_from(pieces).expect("一段的片数装得进 u64");
        Ok(outcome)
    }
}

/// 一段的一次调用里一张卡交它分到的那几条的结局（[`BackgroundGpuOverTheCards`] 的三段共用的那一套）。封闭集合。
enum OutcomesOfACardInTheBatch<Outcome> {
    Computed(Vec<Outcome>),
    /// 卡上空闲的显存一路切小到连最小的一片都建不成：这一批不用这张卡，它分到的那几条重新分给别的卡。
    ShortOfFreeMemory(GpuShortOfFreeMemoryForTheSmallestPiece),
    Failed(GpuFailureCause),
}

/// 一批没算成（三段各自的出错照它报：`checksum_batch_failed_of` 这三个）。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
enum OutcomesOfTheBatchNotComputed {
    GpuFailed {
        device: GpuDeviceIdentifier,
        cause: GpuFailureCause,
    },
    EveryParticipatingGpuLeftOut(Vec<GpuLeftOutUnderItsMemoryLimit>),
    EveryParticipatingGpuShortOfFreeMemory {
        gpus_short_of_free_memory: Vec<GpuShortOfFreeMemoryForTheSmallestPiece>,
        gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
    },
    GpuCannotHoldTheSmallestPiece(SmallestPieceNotHeldOnTheGpu),
}

fn checksum_batch_failed_of(not_computed: OutcomesOfTheBatchNotComputed) -> ChecksumBatchFailed {
    {
        match not_computed {
            OutcomesOfTheBatchNotComputed::GpuFailed { device, cause } => {
                ChecksumBatchFailed::GpuFailed { device, cause }
            }
            OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(gpus_left_out) => {
                ChecksumBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(gpus_left_out)
            }
            OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory {
                gpus_short_of_free_memory,
                gpus_left_out_under_their_memory_limit,
            } => ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
                gpus_short_of_free_memory,
                gpus_left_out_under_their_memory_limit,
            },
            OutcomesOfTheBatchNotComputed::GpuCannotHoldTheSmallestPiece(smallest_piece) => {
                ChecksumBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece)
            }
        }
    }
}

fn region_record_batch_failed_of(
    not_computed: OutcomesOfTheBatchNotComputed,
) -> RegionRecordBatchFailed {
    {
        match not_computed {
            OutcomesOfTheBatchNotComputed::GpuFailed { device, cause } => {
                RegionRecordBatchFailed::GpuFailed { device, cause }
            }
            OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(gpus_left_out) => {
                RegionRecordBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(
                    gpus_left_out,
                )
            }
            OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory {
                gpus_short_of_free_memory,
                gpus_left_out_under_their_memory_limit,
            } => {
                RegionRecordBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
                    gpus_short_of_free_memory,
                    gpus_left_out_under_their_memory_limit,
                }
            }
            OutcomesOfTheBatchNotComputed::GpuCannotHoldTheSmallestPiece(smallest_piece) => {
                RegionRecordBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece)
            }
        }
    }
}

fn live_slot_batch_failed_of(not_computed: OutcomesOfTheBatchNotComputed) -> LiveSlotBatchFailed {
    {
        match not_computed {
            OutcomesOfTheBatchNotComputed::GpuFailed { device, cause } => {
                LiveSlotBatchFailed::GpuFailed { device, cause }
            }
            OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(gpus_left_out) => {
                LiveSlotBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(gpus_left_out)
            }
            OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory {
                gpus_short_of_free_memory,
                gpus_left_out_under_their_memory_limit,
            } => LiveSlotBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
                gpus_short_of_free_memory,
                gpus_left_out_under_their_memory_limit,
            },
            OutcomesOfTheBatchNotComputed::GpuCannotHoldTheSmallestPiece(smallest_piece) => {
                LiveSlotBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece)
            }
        }
    }
}

/// 一份核这一次的派发：管线、按绑定号交给它的缓冲（它的结果缓冲也在里面）、它写的那个结果缓冲。
struct KernelDispatch<'buffers> {
    pipeline: &'buffers wgpu::ComputePipeline,
    bound_buffers: Vec<(u32, &'buffers wgpu::Buffer)>,
    outcome_buffer: &'buffers wgpu::Buffer,
}

/// 一段在同一个只接显存不足的错误范围里建成的缓冲（[`buffers_and_both_readbacks_created_within_one_out_of_memory_scope`]）：
/// 各段自己的那几样与两份核各自的回读暂存（按派发的次序）。
struct BuffersOfAChunk<Buffers, Buffer> {
    buffers: Buffers,
    readbacks: [Buffer; 2],
}

/// 建一段缓冲那一刻要设备做的几样：开一个只接显存不足的错误范围、建一份回读暂存、报一份缓冲建的时候要了多少字节、关上那个范围并在等的上限之内
/// 取它接住的显存不足。产品是 `wgpu::Device`；单测换上在内存里记账的替身（`tests::DeviceOfAChunkInMemoryForTests`）：它照 wgpu 错误范围的规矩
/// （范围开着时建不成的错归最里层那个范围，没有范围开着时报成没接住的错）判得出回读暂存是不是建在范围里（判决 m4-gpugc-code-r2 第三节第 5 条、
/// 四之一「造不出就换方法继续造」：真卡上造不出只让回读暂存建不成的条件），也数得出这一段真建了几份、各多少字节（判决 m4-gpugc-code-r3 第三节第 4 条）。
trait DeviceCreatingTheBuffersOfAChunk {
    type Buffer;
    type OutOfMemoryScope;

    fn open_the_out_of_memory_scope(&self) -> Self::OutOfMemoryScope;

    /// 一份核的回读暂存：`outcome_bytes` 字节，映射来读、接结果缓冲的拷贝。
    fn readback_of_one_kernel(&self, outcome_bytes: u64) -> Self::Buffer;

    /// 这份缓冲建的时候要的字节数（建没建成都照要的报：建不成的那一份，显存不足由范围接住）。
    fn bytes_of_the_buffer(&self, buffer: &Self::Buffer) -> u64;

    /// 关上 `scope`，交回它接住的显存不足（一句原因；没接住的是 `None`）。
    ///
    /// # Errors
    /// 等设备交回那个范围超过 `wait_limit`。
    fn out_of_memory_captured_by(
        &self,
        scope: Self::OutOfMemoryScope,
        wait_limit: Duration,
    ) -> Result<Option<String>, GpuFailureCause>;
}

impl DeviceCreatingTheBuffersOfAChunk for wgpu::Device {
    type Buffer = wgpu::Buffer;
    type OutOfMemoryScope = wgpu::ErrorScopeGuard;

    fn open_the_out_of_memory_scope(&self) -> wgpu::ErrorScopeGuard {
        self.push_error_scope(wgpu::ErrorFilter::OutOfMemory)
    }

    fn readback_of_one_kernel(&self, outcome_bytes: u64) -> wgpu::Buffer {
        self.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: outcome_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn bytes_of_the_buffer(&self, buffer: &wgpu::Buffer) -> u64 {
        buffer.size()
    }

    fn out_of_memory_captured_by(
        &self,
        scope: wgpu::ErrorScopeGuard,
        wait_limit: Duration,
    ) -> Result<Option<String>, GpuFailureCause> {
        let captured = wait_for_the_wgpu_future(
            scope.pop(),
            wait_limit,
            &SteadyClockOfThisProcess::starting_now(),
        )?;
        Ok(captured.map(|out_of_memory| format!("{out_of_memory}")))
    }
}

/// 在同一个只接显存不足的错误范围里建这一段的缓冲（`create_the_buffers`）与两份核各自的回读暂存（`outcome_bytes` 字节）：回读暂存算在这一段的占用里
/// （`bytes_held_by_this_chunk`），建在范围之外时它显存不足报成没接住的错、这一批出错停下，不走「这一批转给别的卡」（判决 m4-gpugc-code-r2 第三节第 5 条）。
/// 账核真建的缓冲（判决 m4-gpugc-code-r3 第三节第 4 条）：关上范围之后把这一段真建的每一份缓冲要的字节加起来，与记在这张卡账上的 `bytes_held_by_this_chunk`
/// 比（账在建之前按切批的占用记下、记的时候断言过不超上限）；多建一块、少记一块都在这里断言，不带着比账多的显存往下交。
/// 建得成交回缓冲与回读暂存；范围接住了显存不足就丢掉建出来的、交回 [`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]；
/// 等设备交回那个范围超过 `wait_limit` 照它的错交回。不动这一段的账（调用方放）。
///
/// # Errors
/// 上面那两种。
///
/// # Panics
/// 真建的缓冲合起来的字节数与 `bytes_held_by_this_chunk` 不等：切批按的占用与建缓冲对不上，是 bug。
fn buffers_and_both_readbacks_created_within_one_out_of_memory_scope<Device, Buffers>(
    device: &Device,
    wait_limit: Duration,
    bytes_held_by_this_chunk: u64,
    outcome_bytes: u64,
    create_the_buffers: impl FnOnce() -> Buffers,
) -> Result<BuffersOfAChunk<Buffers, Device::Buffer>, GpuFailureCause>
where
    Device: DeviceCreatingTheBuffersOfAChunk,
    Buffers: AsRef<[Device::Buffer]>,
{
    let out_of_memory_scope = device.open_the_out_of_memory_scope();
    let buffers = create_the_buffers();
    let readbacks: [Device::Buffer; 2] =
        std::array::from_fn(|_kernel| device.readback_of_one_kernel(outcome_bytes));
    let captured = device.out_of_memory_captured_by(out_of_memory_scope, wait_limit);
    let bytes_of_the_buffers_created: u64 = buffers
        .as_ref()
        .iter()
        .chain(&readbacks)
        .map(|buffer| device.bytes_of_the_buffer(buffer))
        .sum();
    assert_eq!(
        bytes_of_the_buffers_created, bytes_held_by_this_chunk,
        "这一段真建的缓冲合起来的字节数与记在这张卡账上的不等：切批按的占用与建缓冲对不上，账上的峰值管不住卡上真占的"
    );
    match captured {
        Ok(None) => Ok(BuffersOfAChunk { buffers, readbacks }),
        Ok(Some(reason)) => Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
            bytes_held_by_the_chunk: bytes_held_by_this_chunk,
            reason,
        }),
        Err(cause) => Err(cause),
    }
}

/// 在卡上建一份 `bytes` 字节的缓冲（`usage`），建的时候映射着：映射得上才交给 `write_the_contents` 写、再解开映射。
/// 不用 wgpu 的 `create_buffer_init`：它建完就对映射区 `expect`，缓冲因显存不足建不成时那一句 panic（判决 m4-gpugc-code-r3 第三节第 2 条，wgpu 30.0.1
/// `src/util/device.rs`），调用方开着的显存不足的错误范围没机会关、走不到「切小重切、这一批转给别的卡」。映射不上只有一种：缓冲没建成（显存不足），
/// 那一条错已归调用方开着的那个范围，这里不写、不解开映射（对建不成的缓冲解开映射会另报一条没接住的错）。
fn buffer_mapped_at_creation_within_the_scope_of_the_caller(
    device: &wgpu::Device,
    bytes: u64,
    usage: wgpu::BufferUsages,
    write_the_contents: impl FnOnce(&mut wgpu::BufferViewMut),
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: bytes,
        usage,
        mapped_at_creation: true,
    });
    if let Ok(mut mapped) = buffer.slice(..).get_mapped_range_mut() {
        write_the_contents(&mut mapped);
        drop(mapped);
        buffer.unmap();
    }
    buffer
}

/// 在卡上建一份装着 `contents` 的缓冲（`usage`），字节数按整字向上补、至少一个整字（`memory_limit_and_chunks::input_buffer_bytes`，与切批记账同一种算法），
/// 建法见 [`buffer_mapped_at_creation_within_the_scope_of_the_caller`]。
fn buffer_with_contents_created_within_the_scope_of_the_caller(
    device: &wgpu::Device,
    contents: &[u8],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    buffer_mapped_at_creation_within_the_scope_of_the_caller(
        device,
        input_buffer_bytes(bytes_in_a_buffer(contents)),
        usage,
        |mapped| mapped.slice(..contents.len()).copy_from_slice(contents),
    )
}

/// 区域扫描那一段一种切法要交几次：头一片之后那几片的段，加映射每一片之下的段。
fn submissions_of_a_region_record_plan(plan: &RegionRecordPlanOnAGpu) -> usize {
    plan.chunks_of_the_pieces_after_the_heads.len()
        + plan
            .mapping_slices
            .iter()
            .map(|mapping_slice| mapping_slice.record_chunks.len())
            .sum::<usize>()
}

/// 选段打分那一段一种切法要交几次：记录表每一片之下的段。
fn submissions_of_a_live_slot_plan(plan: &[RecordSliceWithItsChunks]) -> usize {
    plan.iter()
        .map(|record_slice| record_slice.segment_chunks.len())
        .sum()
}

/// 挂载那一刻：枚举适配器、判卡清单与校验、在本机每一张独立显卡上起设备与各段两份核、量一次提交的主机 CPU 时间（同
/// [`open_background_gpu_with_the_memory_leaked_in_this_process`]，进程里还一张卡都没泄漏过）。
///
/// # Errors
/// [`BackgroundGpuOpenRefused`] 的四种。
pub fn open_background_gpu(
    selection: &BackgroundGpuSelection,
) -> Result<BackgroundGpu, BackgroundGpuOpenRefused> {
    open_background_gpu_with_the_memory_leaked_in_this_process(
        selection,
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
    )
}

/// 挂载那一刻：枚举适配器、判卡清单与校验、在本机每一张独立显卡上起设备与各段两份核、量一次提交的主机 CPU 时间；参与的卡照 `selection`
/// （之后切换时核心层另交，[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]）。每一处等 wgpu 都在 `selection.background_gpu_wait_limit` 之内。
/// 给了 `background_gpu_devices` 时只判点名的卡（没点名的独显交不成一张卡就不收、报出来）。
/// 每张卡开卡（起设备、编各段两份核的管线、量代价）至多等 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]（用户 2026-11-02 定五分钟），到点当那张卡没开成
/// （`waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit`，报等超过上限、带这个上限），挂载等开卡成功或到点才往下走。
/// 没参与的卡也开（之后的切换可能点它，切换不重开设备）；一张卡没开成，参与的就拒，给了列表而没点名的跳过、报出来
/// （[`gpus_opened_at_the_mount`]、[`BackgroundGpuOverTheCards::gpus_not_opened_at_the_mount`]）。参与的卡照开卡那一组给它的显存上限
/// （先扣掉这个进程里此前几次挂载泄漏在它上面的字节，`memory_leaked_in_this_process`）把量代价那一段（[`COST_PROBE_CHUNK`]）切成几片逐片量、代价取各片之和；
/// 显存上限连最小的一片都装不下的卡照开、不量代价、不参与、报出来（[`BackgroundGpuOverTheCards::gpus_left_out_under_their_memory_limit`]），挂载不因此拒绝；
/// 参与的卡因此一张都不剩时照样交回，三段每一批都报「参与的卡一张都不剩」（[`BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut`]），
/// 核心层挂载那一刻就读到、当场报警停着。丢掉它时泄漏的卡记进 `memory_leaked_in_this_process`。
///
/// # Errors
/// [`BackgroundGpuOpenRefused`] 的四种。
pub fn open_background_gpu_with_the_memory_leaked_in_this_process(
    selection: &BackgroundGpuSelection,
    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
) -> Result<BackgroundGpu, BackgroundGpuOpenRefused> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapters = wait_for_the_wgpu_future(
        instance.enumerate_adapters(wgpu::Backends::VULKAN),
        selection.background_gpu_wait_limit,
        &SteadyClockOfThisProcess::starting_now(),
    )
    .map_err(|cause| BackgroundGpuOpenRefused::AdapterEnumerationFailed { cause })?;
    let descriptions: Vec<EnumeratedAdapterDescription> =
        adapters.iter().map(description_of_the_adapter).collect();
    let discrete_gpus = discrete_gpus_of_the_enumeration_under_the_named(
        &descriptions,
        selection.background_gpu_devices.as_deref(),
    )
    .map_err(BackgroundGpuOpenRefused::DiscreteGpuListNotSupported)?;
    let participating =
        participating_gpus_admitted_by_the_mount_checks(selection, &discrete_gpus.listed)
            .map_err(BackgroundGpuOpenRefused::SettingsRefused)?;
    let gpus_opened = gpus_opened_at_the_mount(
        discrete_gpus.listed,
        &participating,
        bytes_of_the_smallest_piece_every_participating_gpu_holds(),
        memory_leaked_in_this_process,
        |discrete_gpu, memory_limit_at_the_opening, cost_at_the_opening| {
            let adapter = adapters[discrete_gpu.position_in_the_enumeration].clone();
            let discrete_gpu_to_open = discrete_gpu.clone();
            let wait_limit = selection.background_gpu_wait_limit;
            let bytes_leaked_by_earlier_mounts =
                memory_leaked_in_this_process.bytes_leaked_on(&discrete_gpu.identifier);
            outcome_of_opening_a_gpu_within_the_limit(
                WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
                move || {
                    started_gpu_with_its_cost_measured(
                        &adapter,
                        discrete_gpu_to_open,
                        memory_limit_at_the_opening,
                        wait_limit,
                        cost_at_the_opening,
                        bytes_leaked_by_earlier_mounts,
                    )
                },
            )
        },
    )?;
    Ok(BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
        gpus_opened.opened,
        gpus_opened.discrete_gpus,
        discrete_gpus.unnamed_left_off_the_list,
        gpus_opened.participating,
        gpus_opened.left_out_under_their_memory_limit,
        memory_leaked_in_this_process.clone(),
        Box::new(AdaptersEnumeratedByTheInstance {
            instance,
            wait_limit: selection.background_gpu_wait_limit,
        }),
    ))
}

/// 挂着切换那一刻现查本机在的独显（[`DiscreteGpusPresentNow`]）：拿开卡的那个 wgpu 实例再枚举一次，在开卡时那个等的上限之内等；
/// 它也是卡的设备都丢了之后才丢的那个实例。
struct AdaptersEnumeratedByTheInstance {
    instance: wgpu::Instance,
    wait_limit: Duration,
}

impl DiscreteGpusPresentNow for AdaptersEnumeratedByTheInstance {
    fn identifiers_of_the_discrete_gpus_present_now(&self) -> Vec<GpuDeviceIdentifier> {
        let Ok(adapters) = wait_for_the_wgpu_future(
            self.instance.enumerate_adapters(wgpu::Backends::VULKAN),
            self.wait_limit,
            &SteadyClockOfThisProcess::starting_now(),
        ) else {
            // 枚举不成（等 wgpu 超过上限）：一张都不算在，切换照「GPU 不在」拒（DiscreteGpusPresentNow 的约定）。
            return Vec::new();
        };
        let descriptions: Vec<EnumeratedAdapterDescription> = adapters
            .iter()
            .map(|adapter| {
                let adapter_information = adapter.get_info();
                EnumeratedAdapterDescription {
                    name: adapter_information.name,
                    device_kind: AdapterDeviceKind::of_the_wgpu_device_type(
                        adapter_information.device_type,
                    ),
                    pci_bus_address: adapter_information.device_pci_bus_id,
                    device_local_memory: None,
                }
            })
            .collect();
        identifiers_of_the_discrete_gpus_in_the_enumeration(&descriptions)
    }
}

fn description_of_the_adapter(adapter: &wgpu::Adapter) -> EnumeratedAdapterDescription {
    let adapter_information = adapter.get_info();
    let device_kind = AdapterDeviceKind::of_the_wgpu_device_type(adapter_information.device_type);
    let device_local_memory = if device_kind.counts_as_a_background_gpu() {
        largest_device_local_memory_heap(adapter)
    } else {
        None
    };
    EnumeratedAdapterDescription {
        name: adapter_information.name,
        device_kind,
        pci_bus_address: adapter_information.device_pci_bus_id,
        device_local_memory,
    }
}

/// 显存总量：这张卡 Vulkan 物理设备最大的那一块设备本地内存堆。独显上显存是一整块设备本地堆；另一块小的设备本地堆
/// （主机可见的那一窗，例如 256 MiB）是同一片显存的一部分，加起来会重算，所以取最大的一块、不求和。不是 Vulkan 后端的适配器交回 `None`。
#[allow(
    unsafe_code,
    reason = "wgpu 不报显存，只能经 hal 层拿 Vulkan 实例与物理设备句柄去读内存堆；wgpu 与 ash 都没有安全的封装"
)]
fn largest_device_local_memory_heap(adapter: &wgpu::Adapter) -> Option<GpuMemoryInBytes> {
    // SAFETY: `as_hal` 只借出 hal 层的适配器，这里不销毁也不改它指向的任何 Vulkan 对象；守卫在这个函数里丢掉。
    let hal_adapter = unsafe { adapter.as_hal::<wgpu::hal::api::Vulkan>() }?;
    let raw_instance = hal_adapter.shared_instance().raw_instance();
    // SAFETY: 物理设备句柄取自同一个 hal 适配器，那个适配器就是这个实例枚举出来的；守卫 `hal_adapter` 到函数末尾才丢，
    // 这一刻还没丢，实例与物理设备都还在；`vkGetPhysicalDeviceMemoryProperties` 只读。
    let memory_properties = unsafe {
        raw_instance.get_physical_device_memory_properties(hal_adapter.raw_physical_device())
    };
    memory_properties
        .memory_heaps_as_slice()
        .iter()
        .filter(|memory_heap| {
            memory_heap
                .flags
                .contains(ash::vk::MemoryHeapFlags::DEVICE_LOCAL)
        })
        .map(|memory_heap| memory_heap.size)
        .max()
        .map(GpuMemoryInBytes)
}

fn started_gpu_with_its_cost_measured(
    adapter: &wgpu::Adapter,
    discrete_gpu: DiscreteGpuOnThisHost,
    memory_limit: Option<GpuMemoryInBytes>,
    wait_limit: Duration,
    cost_at_the_opening: CostAtTheOpening,
    bytes_leaked_by_earlier_mounts: u64,
) -> Result<GpuInUse, GpuFailureCause> {
    let adapter_limits = adapter.limits();
    let (device, queue) = wait_for_the_wgpu_future(
        adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("singlefs-background-gpu"),
            required_features: wgpu::Features::empty(),
            // 一段的输入与描述各绑成一张存储缓冲：按这张卡实际支持的上限要，默认的 128 MiB 绑定上限会让一批切得太碎。
            required_limits: adapter_limits.clone(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }),
        wait_limit,
        &SteadyClockOfThisProcess::starting_now(),
    )?
    .map_err(|error| GpuFailureCause::DeviceOrPipelineDidNotStart {
        reason: format!("起不了设备：{error}"),
    })?;
    let fault_reported_by_wgpu: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let fault_record_for_uncaptured_errors = Arc::clone(&fault_reported_by_wgpu);
    device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
        record_the_fault_reported_by_wgpu(
            &fault_record_for_uncaptured_errors,
            format!("wgpu 报了没接住的错：{error}"),
        );
    }));
    let fault_record_for_the_loss = Arc::clone(&fault_reported_by_wgpu);
    device.set_device_lost_callback(move |reason, message| {
        record_the_fault_reported_by_wgpu(
            &fault_record_for_the_loss,
            format!("设备丢了（{reason:?}）：{message}"),
        );
    });
    let table_driven_pipeline = compute_pipeline_of_the_kernel(
        &device,
        TABLE_DRIVEN_KERNEL_SOURCE,
        TABLE_DRIVEN_KERNEL_ENTRY_POINT,
    );
    let bitwise_pipeline =
        compute_pipeline_of_the_kernel(&device, BITWISE_KERNEL_SOURCE, BITWISE_KERNEL_ENTRY_POINT);
    let walking_pipeline =
        compute_pipeline_of_the_kernel(&device, WALKING_KERNEL_SOURCE, WALKING_KERNEL_ENTRY_POINT);
    let searching_pipeline = compute_pipeline_of_the_kernel(
        &device,
        SEARCHING_KERNEL_SOURCE,
        SEARCHING_KERNEL_ENTRY_POINT,
    );
    let live_slot_walking_pipeline = compute_pipeline_of_the_kernel(
        &device,
        WALKING_EVERY_RECORD_KERNEL_SOURCE,
        WALKING_EVERY_RECORD_KERNEL_ENTRY_POINT,
    );
    let live_slot_searching_pipeline = compute_pipeline_of_the_kernel(
        &device,
        SEARCHING_SORTED_RECORDS_KERNEL_SOURCE,
        SEARCHING_SORTED_RECORDS_KERNEL_ENTRY_POINT,
    );
    if let Some(reason) = fault_taken_from_the_record(&fault_reported_by_wgpu) {
        return Err(GpuFailureCause::DeviceOrPipelineDidNotStart { reason });
    }
    let largest_entry_count_per_dispatch =
        u64::from(adapter_limits.max_compute_workgroups_per_dimension)
            * u64::from(
                INVOCATIONS_PER_WORKGROUP
                    .min(REGION_RECORD_INVOCATIONS_PER_WORKGROUP)
                    .min(LIVE_SLOT_INVOCATIONS_PER_WORKGROUP),
            );
    let budget = GpuBudgetUnderTheSettings::with_the_bytes_leaked_by_earlier_mounts(
        adapter_limits
            .max_storage_buffer_binding_size
            .min(adapter_limits.max_buffer_size)
            .min(LARGEST_STORAGE_BINDING_THE_KERNELS_ADDRESS_IN_BYTES),
        largest_entry_count_per_dispatch,
        memory_limit,
        wait_limit,
        bytes_leaked_by_earlier_mounts,
    );
    let mut gpu_in_use = GpuInUse {
        discrete_gpu,
        device,
        queue,
        table_driven_pipeline,
        bitwise_pipeline,
        walking_pipeline,
        searching_pipeline,
        live_slot_walking_pipeline,
        live_slot_searching_pipeline,
        budget,
        fault_reported_by_wgpu,
        measured_cost: None,
    };
    match cost_at_the_opening {
        CostAtTheOpening::MeasuredUnderTheMemoryLimit => {
            gpu_in_use.measure_the_cost_of_one_submission()?;
        }
        CostAtTheOpening::NotMeasuredForTheGpuIsLeftOut => {}
    }
    Ok(gpu_in_use)
}

fn compute_pipeline_of_the_kernel(
    device: &wgpu::Device,
    kernel_source: &str,
    entry_point: &str,
) -> wgpu::ComputePipeline {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(entry_point),
        source: wgpu::ShaderSource::Wgsl(kernel_source.into()),
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(entry_point),
        layout: None,
        module: &module,
        entry_point: Some(entry_point),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn record_the_fault_reported_by_wgpu(record: &Mutex<Option<String>>, reason: String) {
    let mut recorded = record
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // 先记下的那一个原因留着：后面的多半是它的连锁反应
    if recorded.is_none() {
        *recorded = Some(reason);
    }
}

fn fault_taken_from_the_record(record: &Mutex<Option<String>>) -> Option<String> {
    record
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}

fn bytes_read_back_from(readback: &wgpu::Buffer) -> Result<Vec<u8>, GpuFailureCause> {
    let read_back = {
        let mapped = readback.slice(..).get_mapped_range().map_err(|error| {
            GpuFailureCause::ReadbackMappingFailed {
                reason: format!("{error}"),
            }
        })?;
        mapped.to_vec()
    };
    readback.unmap();
    Ok(read_back)
}

fn bytes_in_a_buffer(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).expect("装得进 u64")
}

/// 看一次这张卡上的这次提交在 GPU 上做没做完，最多等 `time_left`（wgpu 自己带限时的 `poll`）。
fn completion_check_of_the_submission(
    device: &wgpu::Device,
    submission: &wgpu::SubmissionIndex,
    time_left: Duration,
) -> CompletionCheck<()> {
    match device.poll(wgpu::PollType::Wait {
        submission_index: Some(submission.clone()),
        timeout: Some(time_left),
    }) {
        Ok(_poll_status) => CompletionCheck::Completed(()),
        Err(wgpu::PollError::Timeout) => CompletionCheck::NotYetComplete,
        Err(wrong_index @ wgpu::PollError::WrongSubmissionIndex(..)) => {
            CompletionCheck::Failed(GpuFailureCause::WaitForTheSubmissionFailed {
                reason: format!("{wrong_index}"),
            })
        }
    }
}

impl GpuInUse {
    /// 量一次提交的主机 CPU 时间（第 2 条；挂载那一刻，或换设置让开卡时没量的卡参与时）：这条线程的 CPU 钟（不是挂钟）。
    /// 交的那一段是 [`COST_PROBE_CHUNK`]，照这张卡此刻剩下的界（上限扣掉已占的，含这个进程里此前泄漏在它上面的）切成几片、几段
    /// （装得下就一段一片），每一次是各段各交一遍：建缓冲、编码两份核、交、等、回读、解码，与挂着时交一段走同一条路
    /// （[`CardOfTheBackgroundGpu::both_kernel_outcomes_of_a_checksum_chunk`]）；先空跑一遍，再量 [`COST_MEASUREMENT_SUBMISSIONS`] 遍，
    /// 每遍取各段之和，取中位数（[`measured_cost_summed_over_the_pieces`]）。
    ///
    /// # Errors
    /// 交出错照原样交回；最小的一片装不进这张卡的绑定上限报成起不了（参与的卡显存上限装得下最小的一片，这里只剩绑定上限一样）。
    fn measured_cost_of_one_submission(
        &mut self,
    ) -> Result<HostCpuNanosecondsPerSubmission, GpuFailureCause> {
        let probe_unit_bytes = vec![
            0_u8;
            usize::try_from(COST_PROBE_CHUNK.input_bytes)
                .expect("量代价那一条 4096 字节，装得进 usize")
        ];
        let probe = [UnitChecksumToVerify {
            entry_key: (),
            unit_bytes: probe_unit_bytes.as_slice(),
            expected_crc32c: 0,
        }];
        let chunks_of_pieces = COST_PROBE_CHUNK
            .pieces_under_the_bounds(self.budget.room_left_now())
            .map_err(
                |smallest_piece| GpuFailureCause::DeviceOrPipelineDidNotStart {
                    reason: format!(
                        "量代价那一段最小的一片 {} 字节装不进这张卡的绑定上限",
                        smallest_piece.smallest_piece_bytes
                    ),
                },
            )?;
        measured_cost_summed_over_the_pieces(
            &chunks_of_pieces,
            COST_MEASUREMENT_SUBMISSIONS,
            |pieces| {
                let piece_units = units_of_the_pieces(&probe, pieces);
                let cpu_time_before = host_cpu_time_of_this_thread();
                self.both_kernel_outcomes_of_a_checksum_chunk(&piece_units)?;
                Ok(host_cpu_time_of_this_thread().saturating_sub(cpu_time_before))
            },
        )
    }

    /// 这张卡上建一份装着 `contents` 的缓冲（[`buffer_with_contents_created_within_the_scope_of_the_caller`]）：只在建一段缓冲的那个显存不足的错误范围里调。
    fn buffer_with_contents(&self, contents: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
        buffer_with_contents_created_within_the_scope_of_the_caller(&self.device, contents, usage)
    }

    /// 在只接显存不足的错误范围里建这一段的缓冲（`create_the_buffers`）与两份核各自的回读暂存
    /// （[`buffers_and_both_readbacks_created_within_one_out_of_memory_scope`]），这一段的账（`bytes_held_by_this_chunk`）调用方已经记下：
    /// 建得成交回缓冲与回读暂存；建不成（显存不足，第 3 条「卡上此刻空闲的不到这个数」，或等 wgpu 交回那个错误范围超过上限）
    /// 就放掉这一段的账、交回那个错，一次都不交。
    fn buffers_created_within_the_free_memory<Buffers: AsRef<[wgpu::Buffer]>>(
        &mut self,
        bytes_held_by_this_chunk: u64,
        outcome_bytes: u64,
        create_the_buffers: impl FnOnce(&Self) -> Buffers,
    ) -> Result<BuffersOfAChunk<Buffers, wgpu::Buffer>, GpuFailureCause> {
        let gpu_in_use: &Self = self;
        let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
            &gpu_in_use.device,
            gpu_in_use.budget.wait_limit(),
            bytes_held_by_this_chunk,
            outcome_bytes,
            || create_the_buffers(gpu_in_use),
        );
        if created.is_err() {
            self.budget
                .memory_account_mut()
                .release(bytes_held_by_this_chunk);
        }
        created
    }

    /// 两份核在同一次提交里各算一遍、各自的结果缓冲拷进各自的回读暂存（`readbacks`，按派发的次序，与别的缓冲在同一个错误范围里建好的），
    /// 等做完、等两份映射好，交回两份读回的字节（按派发的次序），
    /// 连同这次提交等完之后在卡上还占不占显存（没等到它做完的，调用方照它把这一段的账留着）。
    /// 一次提交里的几处等共用一个期限，合起来不超过此刻设置给这张卡的等的上限。
    fn dispatch_both_kernels_and_read_back(
        &self,
        kernel_dispatches: [KernelDispatch<'_>; 2],
        readbacks: &[wgpu::Buffer; 2],
        workgroups: u32,
        outcome_bytes: u64,
    ) -> (
        Result<[Vec<u8>; 2], GpuFailureCause>,
        SubmissionOfTheChunk<wgpu::SubmissionIndex>,
    ) {
        let device = &self.device;
        let bind_groups: Vec<wgpu::BindGroup> = kernel_dispatches
            .iter()
            .map(|kernel_dispatch| {
                let entries: Vec<wgpu::BindGroupEntry<'_>> = kernel_dispatch
                    .bound_buffers
                    .iter()
                    .map(|(binding, buffer)| wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: buffer.as_entire_binding(),
                    })
                    .collect();
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &kernel_dispatch.pipeline.get_bind_group_layout(0),
                    entries: &entries,
                })
            })
            .collect();
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            for (kernel_dispatch, bind_group) in kernel_dispatches.iter().zip(&bind_groups) {
                pass.set_pipeline(kernel_dispatch.pipeline);
                pass.set_bind_group(0, bind_group, &[]);
                pass.dispatch_workgroups(workgroups, 1, 1);
            }
        }
        for (kernel_dispatch, readback) in kernel_dispatches.iter().zip(readbacks) {
            encoder.copy_buffer_to_buffer(
                kernel_dispatch.outcome_buffer,
                0,
                readback,
                0,
                outcome_bytes,
            );
        }
        let submission = self.queue.submit(std::iter::once(encoder.finish()));
        let (mapping_sender, mapping_receiver) = std::sync::mpsc::channel();
        for readback in readbacks {
            let sender_for_this_readback = mapping_sender.clone();
            readback
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |mapping_outcome| {
                    // 送不出去只有一种：收的那一头已经丢了（这一批已经按出错交回），这次映射的结局没人要
                    let _nobody_waits_for_this_mapping_any_more =
                        sender_for_this_readback.send(mapping_outcome);
                });
        }
        drop(mapping_sender);
        let clock = SteadyClockOfThisProcess::starting_now();
        let deadline = GpuWaitDeadline::starting_now(self.budget.wait_limit(), &clock);
        let polled = wait_for_completion_before_the_deadline(&deadline, &clock, |time_left| {
            completion_check_of_the_submission(device, &submission, time_left)
        });
        let submission_of_the_chunk =
            SubmissionOfTheChunk::after_waiting_for_its_completion(submission, &polled);
        let read_back = self.both_readbacks_once_the_submission_completed(
            polled,
            readbacks,
            &mapping_receiver,
            &deadline,
            &clock,
        );
        (read_back, submission_of_the_chunk)
    }

    /// 等完提交之后：先看 wgpu 的回调记没记下错，再看提交等没等到，再在同一个期限之内等两份回读映射好、读出来。
    fn both_readbacks_once_the_submission_completed(
        &self,
        polled: Result<(), GpuFailureCause>,
        readbacks: &[wgpu::Buffer; 2],
        mapping_receiver: &std::sync::mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>,
        deadline: &GpuWaitDeadline,
        clock: &SteadyClockOfThisProcess,
    ) -> Result<[Vec<u8>; 2], GpuFailureCause> {
        if let Some(reason) = fault_taken_from_the_record(&self.fault_reported_by_wgpu) {
            return Err(GpuFailureCause::UncapturedErrorOrDeviceLost { reason });
        }
        polled?;
        for _readback in readbacks {
            wait_for_completion_before_the_deadline(deadline, clock, |time_left| {
                match mapping_receiver.recv_timeout(time_left) {
                    Ok(Ok(())) => CompletionCheck::Completed(()),
                    Ok(Err(error)) => {
                        CompletionCheck::Failed(GpuFailureCause::ReadbackMappingFailed {
                            reason: format!("{error}"),
                        })
                    }
                    Err(RecvTimeoutError::Timeout) => CompletionCheck::NotYetComplete,
                    Err(RecvTimeoutError::Disconnected) => {
                        CompletionCheck::Failed(GpuFailureCause::ReadbackMappingFailed {
                            reason: "等完了提交，映射的回调还没送到就都丢了".to_string(),
                        })
                    }
                }
            })?;
        }
        let [first_readback, second_readback] = readbacks;
        Ok([
            bytes_read_back_from(first_readback)?,
            bytes_read_back_from(second_readback)?,
        ])
    }
}

impl CardOfTheBackgroundGpu for GpuInUse {
    type Submission = wgpu::SubmissionIndex;

    fn discrete_gpu(&self) -> &DiscreteGpuOnThisHost {
        &self.discrete_gpu
    }

    fn budget(&self) -> &GpuBudgetUnderTheSettings<wgpu::SubmissionIndex> {
        &self.budget
    }

    fn budget_mut(&mut self) -> &mut GpuBudgetUnderTheSettings<wgpu::SubmissionIndex> {
        &mut self.budget
    }

    fn measured_cost(&self) -> Option<HostCpuNanosecondsPerSubmission> {
        self.measured_cost
    }

    fn measure_the_cost_of_one_submission(&mut self) -> Result<(), GpuFailureCause> {
        self.measured_cost = Some(self.measured_cost_of_one_submission()?);
        Ok(())
    }

    /// 在这张卡上交这一批之前调：前一次等超过上限、还在跑的那次提交先在此刻设置给的等的上限之内等它做完、放掉它的账（第 3 条；等不到交回出错，
    /// 不越过上限去交），交回此刻剩下的界（`GpuBudgetUnderTheSettings`）。
    fn room_once_the_submission_running_past_its_wait_completed(
        &mut self,
    ) -> Result<ChunkBoundsOfAGpu, GpuFailureCause> {
        let clock = SteadyClockOfThisProcess::starting_now();
        let device = &self.device;
        self.budget
            .room_once_the_submission_running_past_its_wait_completed(
                &clock,
                |submission, time_left| {
                    completion_check_of_the_submission(device, submission, time_left)
                },
            )
    }

    /// 重交之前调：前一次等超过上限、还在跑的那次提交在此刻设置给的等的上限之内等它做完、放掉它的账。
    fn wait_for_the_timed_out_submission(&mut self) -> Result<(), GpuFailureCause> {
        let clock = SteadyClockOfThisProcess::starting_now();
        let device = &self.device;
        self.budget.wait_for_the_submission_running_past_its_wait(
            &clock,
            |submission, time_left| {
                completion_check_of_the_submission(device, submission, time_left)
            },
        )
    }

    /// 校验和核对的一段（切批切出来的、装得下这张卡的那几片）交给这张卡：两份核在同一次提交里各算一遍，交回每片两份的结果，按段里的次序。
    /// 本包这一段占的缓冲先记进这张卡的账（超上限断言），那次提交做完放掉；没等到它做完的留着（`GpuMemoryAccount::settle_the_chunk_after_its_submission`）；
    /// 建缓冲显存不足的放掉、不交（[`GpuInUse::buffers_created_within_the_free_memory`]）。
    fn both_kernel_outcomes_of_a_checksum_chunk(
        &mut self,
        units: &[UnitChecksumToVerify<'_, ()>],
    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, GpuFailureCause> {
        assert!(!units.is_empty(), "切批不交空段");
        let packed = packed_checksum_chunk(units);
        let entry_count = packed.entry_count();
        let entry_count_in_words =
            u32::try_from(entry_count).expect("一段的条数不超过一次派发的上限，装得进 u32");
        let outcome_bytes = u64::from(entry_count_in_words) * OUTCOME_BYTES_PER_ENTRY;
        let mut parameter_bytes =
            vec![0_u8; usize::try_from(PARAMETER_BUFFER_BYTES).expect("16 字节")];
        parameter_bytes[..4].copy_from_slice(&entry_count_in_words.to_le_bytes());
        let descriptor_bytes = packed.descriptor_bytes();
        let outcome_fill = outcome_buffer_before_the_kernel_writes(entry_count);

        let bytes_held_by_this_chunk = PARAMETER_BUFFER_BYTES
            + input_buffer_bytes(u64::try_from(packed.input_bytes.len()).expect("装得进 u64"))
            + u64::try_from(descriptor_bytes.len()).expect("装得进 u64")
            + 4 * outcome_bytes;
        assert_eq!(
            bytes_held_by_this_chunk,
            CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
                u64::try_from(
                    units
                        .iter()
                        .map(|unit| unit.unit_bytes.len())
                        .sum::<usize>()
                )
                .expect("装得进 u64"),
                u64::from(entry_count_in_words),
            ),
            "这一段真建的缓冲与切批按的占用一致"
        );
        self.budget
            .memory_account_mut()
            .hold(bytes_held_by_this_chunk);
        let BuffersOfAChunk {
            buffers:
                [parameter_buffer, input_buffer, descriptor_buffer, table_driven_outcomes, bitwise_outcomes],
            readbacks,
        } = self.buffers_created_within_the_free_memory(
            bytes_held_by_this_chunk,
            outcome_bytes,
            |gpu| {
                [
                    gpu.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM),
                    gpu.buffer_with_contents(&packed.input_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&descriptor_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                ]
            },
        )?;
        let (read_back, submission_of_the_chunk) = self.dispatch_both_kernels_and_read_back(
            [
                KernelDispatch {
                    pipeline: &self.table_driven_pipeline,
                    bound_buffers: vec![
                        (0, &parameter_buffer),
                        (1, &input_buffer),
                        (2, &descriptor_buffer),
                        (3, &table_driven_outcomes),
                    ],
                    outcome_buffer: &table_driven_outcomes,
                },
                KernelDispatch {
                    pipeline: &self.bitwise_pipeline,
                    bound_buffers: vec![
                        (0, &parameter_buffer),
                        (1, &input_buffer),
                        (2, &descriptor_buffer),
                        (3, &bitwise_outcomes),
                    ],
                    outcome_buffer: &bitwise_outcomes,
                },
            ],
            &readbacks,
            entry_count_in_words.div_ceil(INVOCATIONS_PER_WORKGROUP),
            outcome_bytes,
        );
        self.budget
            .memory_account_mut()
            .settle_the_chunk_after_its_submission(
                bytes_held_by_this_chunk,
                submission_of_the_chunk,
            );
        let [table_driven_bytes, bitwise_bytes] = read_back?;
        let table_driven = decoded_kernel_outcomes(&table_driven_bytes)
            .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
        let bitwise = decoded_kernel_outcomes(&bitwise_bytes)
            .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
        Ok(table_driven.into_iter().zip(bitwise).collect())
    }

    /// 区域扫描的一段（几片）交给这张卡：两份核在同一次提交里各算一遍。`mapping_tables` 是这一段交的那一片映射：从单元头起的那几片的段交它、
    /// 交回每条两份的分类；单元头一片之后那几片的段是 `None`，交一份空映射、交回每片两份算完的寄存器（`region_record_kernels` 文件头「一个单元切成几片」）。
    /// 按段里的次序。账同 [`CardOfTheBackgroundGpu::both_kernel_outcomes_of_a_checksum_chunk`]。
    fn both_kernel_outcomes_of_a_region_record_chunk(
        &mut self,
        pieces: &[RegionRecordPieceForTheKernels<'_>],
        mapping_tables: Option<&MappingTablesForTheKernels>,
    ) -> Result<BothKernelOutcomesOfARegionRecordChunk, GpuFailureCause> {
        assert!(!pieces.is_empty(), "切批不交空段");
        let is_a_chunk_of_pieces_after_the_heads = mapping_tables.is_none();
        assert!(
            pieces.iter().all(|piece| matches!(
                piece,
                RegionRecordPieceForTheKernels::PieceAfterTheHead { .. }
            ) == is_a_chunk_of_pieces_after_the_heads),
            "一段里的片同一种：交映射片的全是从单元头起的，不交的全是头一片之后的"
        );
        let empty_mapping_tables;
        let mapping_tables = match mapping_tables {
            Some(mapping_tables) => mapping_tables,
            None => {
                empty_mapping_tables = MappingTablesForTheKernels::of_the_mapping_entries(&[])
                    .expect("空映射没有长过核装得下的 key");
                &empty_mapping_tables
            }
        };
        let packed = packed_region_record_chunk(pieces);
        let record_count = u32::try_from(packed.record_count())
            .expect("一段的条数不超过一次派发的上限，装得进 u32");
        let outcome_bytes = u64::from(record_count) * REGION_RECORD_OUTCOME_BYTES;
        let parameter_bytes = region_record_parameter_bytes(record_count, mapping_tables);
        let descriptor_bytes = little_endian_bytes_of_the_words(&packed.descriptor_words);
        let entry_bytes = little_endian_bytes_of_the_words(&mapping_tables.entry_words);
        let location_item_bytes =
            little_endian_bytes_of_the_words(&mapping_tables.location_item_words);
        let key_sorted_position_bytes =
            little_endian_bytes_of_the_words(&mapping_tables.key_sorted_entry_positions);
        let outcome_fill =
            region_record_outcome_buffer_before_the_kernel_writes(packed.record_count());

        let bytes_held_by_this_chunk = bytes_in_a_buffer(&parameter_bytes)
            + bytes_in_a_buffer(&packed.input_bytes)
            + bytes_in_a_buffer(&descriptor_bytes)
            + bytes_in_a_buffer(&entry_bytes)
            + bytes_in_a_buffer(&location_item_bytes)
            + bytes_in_a_buffer(&key_sorted_position_bytes)
            + 4 * outcome_bytes;
        let total_unit_bytes: usize = pieces
            .iter()
            .map(|piece| piece.bytes().map_or(0, <[u8]>::len))
            .sum();
        assert_eq!(
            bytes_held_by_this_chunk,
            region_record_chunk_footprint(mapping_tables).bytes_held_by_a_chunk(
                u64::try_from(total_unit_bytes).expect("装得进 u64"),
                u64::from(record_count),
            ),
            "这一段真建的缓冲与切批按的占用一致"
        );
        self.budget
            .memory_account_mut()
            .hold(bytes_held_by_this_chunk);
        let BuffersOfAChunk {
            buffers:
                [parameter_buffer, input_buffer, descriptor_buffer, entry_buffer, location_item_buffer, key_sorted_position_buffer, walking_outcomes, searching_outcomes],
            readbacks,
        } = self.buffers_created_within_the_free_memory(
            bytes_held_by_this_chunk,
            outcome_bytes,
            |gpu| {
                [
                    gpu.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM),
                    gpu.buffer_with_contents(&packed.input_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&descriptor_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&entry_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&location_item_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(
                        &key_sorted_position_bytes,
                        wgpu::BufferUsages::STORAGE,
                    ),
                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                ]
            },
        )?;
        let (read_back, submission_of_the_chunk) = self.dispatch_both_kernels_and_read_back(
            [
                KernelDispatch {
                    pipeline: &self.walking_pipeline,
                    bound_buffers: vec![
                        (0, &parameter_buffer),
                        (1, &input_buffer),
                        (2, &descriptor_buffer),
                        (3, &entry_buffer),
                        (6, &walking_outcomes),
                    ],
                    outcome_buffer: &walking_outcomes,
                },
                KernelDispatch {
                    pipeline: &self.searching_pipeline,
                    bound_buffers: vec![
                        (0, &parameter_buffer),
                        (1, &input_buffer),
                        (2, &descriptor_buffer),
                        (3, &entry_buffer),
                        (4, &location_item_buffer),
                        (5, &key_sorted_position_buffer),
                        (6, &searching_outcomes),
                    ],
                    outcome_buffer: &searching_outcomes,
                },
            ],
            &readbacks,
            record_count.div_ceil(REGION_RECORD_INVOCATIONS_PER_WORKGROUP),
            outcome_bytes,
        );
        self.budget
            .memory_account_mut()
            .settle_the_chunk_after_its_submission(
                bytes_held_by_this_chunk,
                submission_of_the_chunk,
            );
        let [walking_bytes, searching_bytes] = read_back?;
        if is_a_chunk_of_pieces_after_the_heads {
            let walking = decoded_registers_after_the_pieces(&walking_bytes)
                .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
            let searching = decoded_registers_after_the_pieces(&searching_bytes)
                .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
            return Ok(
                BothKernelOutcomesOfARegionRecordChunk::RegistersAfterThePieces(
                    walking
                        .into_iter()
                        .zip(searching)
                        .map(
                            |(walking_kernel, searching_kernel)| CrcRegistersOfTheTwoKernels {
                                walking_kernel,
                                searching_kernel,
                            },
                        )
                        .collect(),
                ),
            );
        }
        let walking = decoded_region_record_outcomes(&walking_bytes)
            .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
        let searching = decoded_region_record_outcomes(&searching_bytes)
            .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
        Ok(BothKernelOutcomesOfARegionRecordChunk::Classifications(
            walking.into_iter().zip(searching).collect(),
        ))
    }

    /// 选段打分的一段（切批切出来的、装得下这张卡的那几个段）交给这张卡：两份核在同一次提交里各数一遍，交回每段两份的活槽数，按段里的次序。
    /// 这一片的记录表随每一段交一份。账同 [`CardOfTheBackgroundGpu::both_kernel_outcomes_of_a_checksum_chunk`]。
    fn both_kernel_live_slots_of_a_chunk(
        &mut self,
        segments: &[ClusterSegmentEntryKey],
        record_tables: &AllocationRecordTablesForTheKernels,
        unit_area: UnitAreaOfOneDevice,
    ) -> Result<Vec<(u64, u64)>, GpuFailureCause> {
        assert!(!segments.is_empty(), "切批不交空段");
        let segment_count =
            u32::try_from(segments.len()).expect("一段的段数不超过一次派发的上限，装得进 u32");
        let outcome_bytes = u64::from(segment_count) * LIVE_SLOT_OUTCOME_BYTES;
        let parameter_bytes = live_slot_parameter_bytes(segment_count, record_tables, unit_area);
        let segment_bound_bytes = little_endian_bytes_of_the_words(&segment_bound_words(segments));
        let record_bytes = little_endian_bytes_of_the_words(&record_tables.record_words);
        let sorted_position_bytes =
            little_endian_bytes_of_the_words(&record_tables.record_positions_by_device_and_slot);
        let outcome_fill = live_slot_outcome_buffer_before_the_kernel_writes(segments.len());

        let bytes_held_by_this_chunk = bytes_in_a_buffer(&parameter_bytes)
            + bytes_in_a_buffer(&segment_bound_bytes)
            + bytes_in_a_buffer(&record_bytes)
            + bytes_in_a_buffer(&sorted_position_bytes)
            + 4 * outcome_bytes;
        assert_eq!(
            bytes_held_by_this_chunk,
            live_slot_chunk_footprint(record_tables).bytes_held_by_a_chunk(
                u64::from(segment_count) * SEGMENT_BOUND_BYTES,
                u64::from(segment_count),
            ),
            "这一段真建的缓冲与切批按的占用一致"
        );
        self.budget
            .memory_account_mut()
            .hold(bytes_held_by_this_chunk);
        let BuffersOfAChunk {
            buffers:
                [parameter_buffer, segment_bound_buffer, record_buffer, sorted_position_buffer, walking_outcomes, searching_outcomes],
            readbacks,
        } = self.buffers_created_within_the_free_memory(
            bytes_held_by_this_chunk,
            outcome_bytes,
            |gpu| {
                [
                    gpu.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM),
                    gpu.buffer_with_contents(&segment_bound_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&record_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&sorted_position_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                ]
            },
        )?;
        let (read_back, submission_of_the_chunk) = self.dispatch_both_kernels_and_read_back(
            [
                KernelDispatch {
                    pipeline: &self.live_slot_walking_pipeline,
                    bound_buffers: vec![
                        (0, &parameter_buffer),
                        (1, &segment_bound_buffer),
                        (2, &record_buffer),
                        (4, &walking_outcomes),
                    ],
                    outcome_buffer: &walking_outcomes,
                },
                KernelDispatch {
                    pipeline: &self.live_slot_searching_pipeline,
                    bound_buffers: vec![
                        (0, &parameter_buffer),
                        (1, &segment_bound_buffer),
                        (2, &record_buffer),
                        (3, &sorted_position_buffer),
                        (4, &searching_outcomes),
                    ],
                    outcome_buffer: &searching_outcomes,
                },
            ],
            &readbacks,
            segment_count.div_ceil(LIVE_SLOT_INVOCATIONS_PER_WORKGROUP),
            outcome_bytes,
        );
        self.budget
            .memory_account_mut()
            .settle_the_chunk_after_its_submission(
                bytes_held_by_this_chunk,
                submission_of_the_chunk,
            );
        let [walking_bytes, searching_bytes] = read_back?;
        let walking = decoded_live_slot_outcomes(&walking_bytes)
            .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
        let searching = decoded_live_slot_outcomes(&searching_bytes)
            .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
        Ok(walking.into_iter().zip(searching).collect())
    }
}

impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
    /// 挂载那一刻开卡之后：开成的卡（与 `discrete_gpus_after_the_opening.opened` 一一对应、同序）、开卡之后的本机独显、没点名而没收进清单的那几张、
    /// 开卡那一组参与的卡与因显存上限不用的卡（`gpus_of_this_host::gpus_opened_at_the_mount` 交回的），进程里泄漏的那一份（丢它时记进去），
    /// 与卡清单每次答之前现查本机在的独显的那一样（产品带着开卡的 wgpu 实例）。测试拿它把替身卡装进来。
    ///
    /// # Panics
    /// 开成的卡与 `discrete_gpus_after_the_opening.opened` 张数不等。
    #[must_use]
    pub fn of_the_cards_opened_at_the_mount(
        cards_on_this_host: Vec<Card>,
        discrete_gpus_after_the_opening: DiscreteGpusAfterTheOpening,
        unnamed_discrete_gpus_left_off_the_list: Vec<DiscreteGpuListRefused>,
        participating: ParticipatingGpusOnThisHost,
        left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
        memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
        discrete_gpus_present_now: Box<dyn DiscreteGpusPresentNow + Send>,
    ) -> Self {
        assert_eq!(
            cards_on_this_host.len(),
            discrete_gpus_after_the_opening.opened.len(),
            "开成的卡与开卡之后的独显清单一一对应"
        );
        Self {
            cards_on_this_host,
            discrete_gpus_after_the_opening,
            unnamed_discrete_gpus_left_off_the_list,
            participating,
            left_out_under_their_memory_limit,
            cost_not_measured_at_the_last_switch: None,
            submissions_since_the_last_take: GpuSubmissionsOfTheBatch::default(),
            gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped::not_yet_dropped(),
            memory_leaked_in_this_process,
            discrete_gpus_present_now,
        }
    }

    /// 挂载那一刻没开成而跳过的卡（给了 `background_gpu_devices` 而没点名的），按总线地址排，各带原因；没跳过一张是空。
    #[must_use]
    pub fn gpus_not_opened_at_the_mount(&self) -> &[DiscreteGpuNotOpenedAtTheMount] {
        &self.discrete_gpus_after_the_opening.not_opened
    }

    /// 给了 `background_gpu_devices` 时没点名、交不成一张卡（缺总线地址、缺设备本地堆、地址重复）而没收进卡清单的独显，各带原因；没有是空。
    #[must_use]
    pub fn unnamed_discrete_gpus_left_off_the_list(&self) -> &[DiscreteGpuListRefused] {
        &self.unnamed_discrete_gpus_left_off_the_list
    }

    /// 此刻这组设置点了而因显存上限连最小的一片都装不下、不用的卡，按参与的次序，各带上限与至少要多少；没有是空。
    /// 与 [`Self::participating_gpus`] 一起看：参与的卡一张都不剩时三段每一批都交回「参与的卡一张都不剩」（[`Self::batch_not_split_over_the_gpus`]）。
    #[must_use]
    pub fn gpus_left_out_under_their_memory_limit(&self) -> &[GpuLeftOutUnderItsMemoryLimit] {
        &self.left_out_under_their_memory_limit
    }

    /// 丢掉它那一刻泄漏了哪几张卡（还没丢是 `None`）：调用方拿着这一份，丢了之后读（[`GpusLeakedWhenTheBackgroundGpuIsDropped`]）。
    #[must_use]
    pub fn gpus_leaked_when_dropped(&self) -> GpusLeakedWhenTheBackgroundGpuIsDropped {
        self.gpus_leaked_when_dropped.clone()
    }

    /// 上一次交回以来三段各自在卡上是怎么交的（交成了几次、几片、重交了几次、显存不足重切了几次、分到活的是哪几张卡），交回之后清零。
    pub fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
        std::mem::take(&mut self.submissions_since_the_last_take)
    }

    /// 此刻参与的卡，按参与的次序，带此刻设置给的显存上限与量出的一次提交的主机 CPU 时间。
    ///
    /// # Panics
    /// 一张参与的卡没量过代价：开卡或换设置让它参与的那一刻量过，没量成的不参与。
    #[must_use]
    pub fn participating_gpus(&self) -> Vec<ParticipatingGpuReport> {
        self.participating
            .positions_on_this_host()
            .iter()
            .map(|position| {
                let card = &self.cards_on_this_host[*position];
                ParticipatingGpuReport {
                    identifier: card.discrete_gpu().identifier.clone(),
                    name: card.discrete_gpu().name.clone(),
                    memory_limit: card.budget().memory_account().memory_limit(),
                    measured_cost: card
                        .measured_cost()
                        .expect("参与的卡都量过代价：开卡或换设置让它参与的那一刻量的"),
                }
            })
            .collect()
    }

    /// 从下一批起照核心层交来的这一组跑（D26 已定项 13 第 1–3 条；核心层只在两批之间交）：参与的卡换成它点的那几张、按它的次序，
    /// 每张卡的显存上限与等的上限换成它给的（不参与的卡不设显存上限，它分不到活）。不重开设备，各卡的账照旧记着。
    /// 显存上限（扣掉卡上已占的）连最小的一片都装不下的卡不用、报出来（[`Self::gpus_left_out_under_their_memory_limit`]）；开卡时因显存上限没量代价、
    /// 或上一组设置因显存上限晾下而这一组给了装得下的上限的卡，照新上限量一次（量不量照旧、量出的代价不用来分活），量出错的这张卡不参与，
    /// 之后每一批报成它的 GPU 出错，直到下一次换设置。请回来的卡照参与的次序重新参与分活。
    ///
    /// # Panics
    /// 它点的卡有一张不在开成的独显里：核心层照这份实现答的卡清单（此刻本机在的独显，连同没开成的那几张）校验过，参与的卡里有没开成的就拒了。
    pub fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
        let identifiers_on_this_host: Vec<GpuDeviceIdentifier> = self
            .cards_on_this_host
            .iter()
            .map(|card| card.discrete_gpu().identifier.clone())
            .collect();
        let participating_by_the_settings =
            ParticipatingGpusOnThisHost::of_the_settings(settings, &identifiers_on_this_host);
        for card in &mut self.cards_on_this_host {
            let memory_limit = memory_limit_of_the_device_under_the_settings(
                settings,
                &card.discrete_gpu().identifier,
            );
            card.budget_mut()
                .use_the_settings_from_the_next_batch(memory_limit, settings.wait_limit());
        }
        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
        let devices_left_out_under_the_settings_before: Vec<GpuDeviceIdentifier> = self
            .left_out_under_their_memory_limit
            .iter()
            .map(|left_out| left_out.device.clone())
            .collect();
        let mut left_out_under_their_memory_limit = Vec::new();
        let mut cost_not_measured = None;
        let mut positions_left_out = Vec::new();
        for position in participating_by_the_settings.positions_on_this_host() {
            let card = &mut self.cards_on_this_host[*position];
            if let Some(left_out) = gpu_left_out_under_its_memory_limit(
                &card.discrete_gpu().identifier,
                card.budget().memory_account().memory_limit(),
                smallest_piece_bytes,
                card.budget().memory_account().bytes_held(),
            ) {
                left_out_under_their_memory_limit.push(left_out);
                positions_left_out.push(*position);
                continue;
            }
            // 第 3 条「挂着切换给了它装得下的上限，从下一批起照新上限重新量代价」：上一组设置因显存上限晾下的卡请回来时照新上限再量一次
            // （判决 m4-gpugc-code-r3 第三节第 5 条）；量出的代价不用来分活（用户 2026-11-02 定「按卡的顺序填满」），量不成照旧不参与。
            let is_brought_back_after_being_left_out = devices_left_out_under_the_settings_before
                .contains(&card.discrete_gpu().identifier);
            if card.measured_cost().is_none() || is_brought_back_after_being_left_out {
                if let Err(cause) = card.measure_the_cost_of_one_submission() {
                    positions_left_out.push(*position);
                    cost_not_measured
                        .get_or_insert((card.discrete_gpu().identifier.clone(), cause));
                }
            }
        }
        self.participating = participating_by_the_settings
            .leaving_out(|position| positions_left_out.contains(&position));
        self.left_out_under_their_memory_limit = left_out_under_their_memory_limit;
        self.cost_not_measured_at_the_last_switch = cost_not_measured;
    }

    /// 此刻这组设置下一批分不分得到卡上（与这一批有几条无关）：换设置时量一张卡的代价出错了，或参与的卡因显存上限一张都不剩，就交回是哪一种；
    /// 分得到交回 `None`。挂载那一刻核心层经 `BackgroundGpuCompute::failure_under_the_settings_handed_in` 读它，不等头一批。
    ///
    /// # Panics
    /// 参与的卡一张都不剩，却既没有量代价出错、也没有因显存上限不用的卡：核心层校验过的一组至少点一张卡，没参与的只有这两种原因。
    #[must_use]
    pub fn batch_not_split_over_the_gpus(&self) -> Option<BatchNotSplitOverTheGpus> {
        if let Some((device, cause)) = &self.cost_not_measured_at_the_last_switch {
            return Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
                device: device.clone(),
                cause: cause.clone(),
            });
        }
        if !self.participating.is_empty() {
            return None;
        }
        assert!(
            !self.left_out_under_their_memory_limit.is_empty(),
            "参与的卡一张都不剩只有两种：换设置时量代价出错（上面交回了），或这组设置点的卡都因显存上限不用"
        );
        Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(
            self.left_out_under_their_memory_limit.clone(),
        ))
    }

    /// 一批里 `entries` 那几条按卡的顺序填满这一批还用的参与的卡、各卡切好（`cut` 切一张卡分到的那几条，照这张卡交活之前那一刻的地方；
    /// `submissions_of_the_plan` 数一种切法要交几次，填满照它判）：
    /// 这一批因空闲显存不用的卡（`gpus_short_of_free_memory_in_this_batch`，它在本机里的位置连同报的那一条，按不用的先后）不分；交活之前先等这张卡上前一次超时的那次提交
    /// （等满上限算这一批这张卡超时一次、再等，`waiting_for_the_gpu::outcome_waited_for_again_after_timeouts`），切不出最小的一片（显存上限扣掉已占的）的卡
    /// 这一批不分给它（`gpus_of_this_host::plans_of_a_batch_filling_the_gpus_in_order`）。交回的各卡分到的条目在 `entries` 里、
    /// 按卡的次序相连。没有条目交回空。
    ///
    /// # Errors
    /// 换设置时量代价出错、参与的卡因显存上限一张都不剩、这一批还用的卡都因空闲显存不用了（[`OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory`]，
    /// 连同这一次因显存上限不用的）、交活之前那一等出错、一张卡切不出而装不下的是绑定上限。
    fn plans_of_a_batch<Plan>(
        &mut self,
        entries: Range<usize>,
        gpus_short_of_free_memory_in_this_batch: &[(
            usize,
            GpuShortOfFreeMemoryForTheSmallestPiece,
        )],
        submissions_of_each_card: &mut [SubmissionsOfACardInTheBatch],
        cut: impl Fn(Range<usize>, ChunkBoundsOfAGpu) -> Result<Plan, SmallestPieceLargerThanTheRoom>,
        submissions_of_the_plan: impl Fn(&Plan) -> usize,
    ) -> Result<Vec<PlanOfAGpuForTheBatch<Plan>>, OutcomesOfTheBatchNotComputed> {
        match self.batch_not_split_over_the_gpus() {
            Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause }) => {
                return Err(OutcomesOfTheBatchNotComputed::GpuFailed { device, cause });
            }
            Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)) => {
                if entries.is_empty() {
                    return Ok(Vec::new());
                }
                return Err(OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(
                    gpus_left_out,
                ));
            }
            None => {}
        }
        if entries.is_empty() {
            return Ok(Vec::new());
        }
        let gpus_short_of_free_memory: Vec<GpuShortOfFreeMemoryForTheSmallestPiece> =
            gpus_short_of_free_memory_in_this_batch
                .iter()
                .map(|(_position, short)| short.clone())
                .collect();
        let positions_short_of_free_memory: Vec<usize> = gpus_short_of_free_memory_in_this_batch
            .iter()
            .map(|(position, _short)| *position)
            .collect();
        let participating_in_this_batch = self
            .participating
            .clone()
            .leaving_out(|position| positions_short_of_free_memory.contains(&position));
        if participating_in_this_batch.is_empty() {
            return Err(
                OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory {
                    gpus_short_of_free_memory,
                    gpus_left_out_under_their_memory_limit: Vec::new(),
                },
            );
        }
        let cards_on_this_host = &mut self.cards_on_this_host;
        let first_entry = entries.start;
        let shifted_by_the_first_entry =
            |relative: Range<usize>| (relative.start + first_entry)..(relative.end + first_entry);
        plans_of_a_batch_filling_the_gpus_in_order(
            &participating_in_this_batch,
            entries.len(),
            |position| {
                let card = &mut cards_on_this_host[position];
                let device = card.discrete_gpu().identifier.clone();
                let room = outcome_waited_for_again_after_timeouts(
                    &mut *card,
                    &mut submissions_of_each_card[position].timeouts,
                    Card::room_once_the_submission_running_past_its_wait_completed,
                )
                .map_err(|cause| BatchNotPlannedOverTheGpus::GpuFailed {
                    device: device.clone(),
                    cause,
                })?;
                let memory_account = card.budget().memory_account();
                Ok(RoomOfAGpuForTheBatch {
                    device,
                    room,
                    memory_limit: memory_account.memory_limit(),
                    bytes_already_held_on_the_device: memory_account.bytes_held(),
                })
            },
            |relative, room| cut(shifted_by_the_first_entry(relative), room),
            submissions_of_the_plan,
        )
        .map(|plans| {
            plans
                .into_iter()
                .map(|plan| PlanOfAGpuForTheBatch {
                    position_on_this_host: plan.position_on_this_host,
                    entries: shifted_by_the_first_entry(plan.entries),
                    room: plan.room,
                    plan: plan.plan,
                })
                .collect()
        })
        .map_err(|not_planned| match not_planned {
            BatchNotPlannedOverTheGpus::GpuFailed { device, cause } => {
                OutcomesOfTheBatchNotComputed::GpuFailed { device, cause }
            }
            BatchNotPlannedOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)
                if gpus_short_of_free_memory.is_empty() =>
            {
                OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(gpus_left_out)
            }
            BatchNotPlannedOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out) => {
                OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory {
                    gpus_short_of_free_memory,
                    gpus_left_out_under_their_memory_limit: gpus_left_out,
                }
            }
            BatchNotPlannedOverTheGpus::GpuCannotHoldTheSmallestPiece(smallest_piece) => {
                OutcomesOfTheBatchNotComputed::GpuCannotHoldTheSmallestPiece(
                    SmallestPieceNotHeldOnTheGpu {
                        device: smallest_piece.device,
                        entry_position: smallest_piece.entry_position + first_entry,
                        memory_limit: smallest_piece.memory_limit,
                        smallest_piece_bytes: smallest_piece.smallest_piece_bytes,
                    },
                )
            }
        })
    }

    /// 一段（`segment`）的一次调用照三段共用的那一套交给参与的卡：按卡的顺序填满、切（[`Self::plans_of_a_batch`]，`submissions_of_the_plan` 数一种切法交几次），
    /// 再逐张卡照切法交
    /// （`run_the_plan_on_a_card`：这一段自己的交法，交它这张卡、分到的那几条、切法与这一批这张卡的那一份，每一段套重交）。
    /// 建缓冲显存不足（[`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]）就把这张卡这一次的地方压到建不成的那一段的一半、照新切法从头整段重交它分到的那几条
    /// （已交成的片的结果不用，第 4 条「这一段在这张卡上已做完的片不留」）
    /// （第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」）；压到连最小的一片都切不出，这一批就不用这张卡（用户 2026-10-28 定
    /// 「连一片都装不下就不用这张卡」；下一批照设置重新起），它分到的那几条（一条都没交回）重新分给这一批还用的参与的卡、照它们的地方切、交；
    /// 都不用了交回 [`OutcomesOfTheBatchNotComputed::EveryParticipatingGpuShortOfFreeMemory`]。各卡交回的按条目的次序接起来。
    /// 交成了几次、几片、重交了几次、重切了几次、哪几张卡分到了活，记进这一段的那一份（出错也记，[`Self::take_the_submissions_of_the_batch`]）。
    ///
    /// 重切那一圈：每一轮要么交回、要么这张卡的显存那一项严格变小（建不成的那一段是按此刻的界切的，不比界大，压到它的一半），压到切不出就停，所以有界。
    /// 重分那一圈：每次把一段条目放回去，就多一张这一批不用的卡（不再分给它），参与的卡有限，所以至多比参与的卡数多一轮。
    fn outcomes_of_a_batch_over_the_cards<Plan, Outcome>(
        &mut self,
        segment: GpuComputedSegment,
        entry_count: usize,
        cut: impl Fn(Range<usize>, ChunkBoundsOfAGpu) -> Result<Plan, SmallestPieceLargerThanTheRoom>,
        submissions_of_the_plan: impl Fn(&Plan) -> usize,
        mut run_the_plan_on_a_card: impl FnMut(
            &mut Card,
            Range<usize>,
            &Plan,
            &mut SubmissionsOfACardInTheBatch,
        ) -> Result<Vec<Outcome>, GpuFailureCause>,
    ) -> Result<Vec<Outcome>, OutcomesOfTheBatchNotComputed> {
        let mut submissions_of_each_card: Vec<SubmissionsOfACardInTheBatch> = self
            .cards_on_this_host
            .iter()
            .map(|_card| SubmissionsOfACardInTheBatch::none_yet())
            .collect();
        let mut gpus_short_of_free_memory_in_this_batch: Vec<(
            usize,
            GpuShortOfFreeMemoryForTheSmallestPiece,
        )> = Vec::new();
        let mut outcomes_from_the_first_entry: Vec<(usize, Vec<Outcome>)> = Vec::new();
        let mut devices_given_work: Vec<GpuDeviceIdentifier> = Vec::new();
        let mut recuts_after_running_out_of_memory: u64 = 0;
        // 一张待分的条目段的栈，起头只有整批那一段（不是 0 到 entry_count 这几个数）：装不下最小一片的卡交回的那几条再压回来。
        let mut entries_to_hand_out: Vec<Range<usize>> = std::iter::once(0..entry_count).collect();
        let mut failure = None;
        'handing_out: while let Some(entries) = entries_to_hand_out.pop() {
            let plans = match self.plans_of_a_batch(
                entries,
                &gpus_short_of_free_memory_in_this_batch,
                &mut submissions_of_each_card,
                &cut,
                &submissions_of_the_plan,
            ) {
                Ok(plans) => plans,
                Err(not_planned) => {
                    failure = Some(not_planned);
                    break 'handing_out;
                }
            };
            for planned in plans {
                let card = &mut self.cards_on_this_host[planned.position_on_this_host];
                let device = card.discrete_gpu().identifier.clone();
                devices_given_work.push(device.clone());
                let submissions_of_the_card =
                    &mut submissions_of_each_card[planned.position_on_this_host];
                let mut room = planned.room;
                let mut plan = planned.plan;
                let outcomes_of_the_card = loop {
                    match run_the_plan_on_a_card(
                        &mut *card,
                        planned.entries.clone(),
                        &plan,
                        &mut *submissions_of_the_card,
                    ) {
                        Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
                            bytes_held_by_the_chunk,
                            reason: _,
                        }) => {
                            recuts_after_running_out_of_memory += 1;
                            room = room.with_the_memory_limit_at_most(bytes_held_by_the_chunk / 2);
                            // 重切不留已做完的片：这张卡分到的那几条照新切法从头整段重交（判决 m4-gpugc-code-r3 第三节第 3 条）。
                            match cut(planned.entries.clone(), room) {
                                Ok(recut) => plan = recut,
                                Err(smallest_piece) => {
                                    break OutcomesOfACardInTheBatch::ShortOfFreeMemory(
                                        GpuShortOfFreeMemoryForTheSmallestPiece {
                                            device: device.clone(),
                                            bytes_of_the_last_chunk_not_created:
                                                bytes_held_by_the_chunk,
                                            smallest_piece_bytes: smallest_piece
                                                .smallest_piece_bytes,
                                        },
                                    )
                                }
                            }
                        }
                        Ok(of_the_card) => break OutcomesOfACardInTheBatch::Computed(of_the_card),
                        Err(cause) => break OutcomesOfACardInTheBatch::Failed(cause),
                    }
                };
                match outcomes_of_the_card {
                    OutcomesOfACardInTheBatch::Computed(of_the_card) => {
                        outcomes_from_the_first_entry.push((planned.entries.start, of_the_card));
                    }
                    OutcomesOfACardInTheBatch::ShortOfFreeMemory(short) => {
                        gpus_short_of_free_memory_in_this_batch
                            .push((planned.position_on_this_host, short));
                        entries_to_hand_out.push(planned.entries);
                    }
                    OutcomesOfACardInTheBatch::Failed(cause) => {
                        failure = Some(OutcomesOfTheBatchNotComputed::GpuFailed { device, cause });
                        break 'handing_out;
                    }
                }
            }
        }
        self.note_the_submissions_of_the_segment(
            segment,
            &submissions_of_each_card,
            devices_given_work,
            recuts_after_running_out_of_memory,
        );
        if let Some(failure) = failure {
            return Err(failure);
        }
        outcomes_from_the_first_entry.sort_by_key(|(first_entry, _outcomes)| *first_entry);
        let mut outcomes = Vec::with_capacity(entry_count);
        for (first_entry, of_the_card) in outcomes_from_the_first_entry {
            assert_eq!(
                first_entry,
                outcomes.len(),
                "各卡交回的条目按次序相连、不重不漏：重分的那几条接在原来的位置上"
            );
            outcomes.extend(of_the_card);
        }
        assert_eq!(
            outcomes.len(),
            entry_count,
            "每条都有两份结果，按条目的次序接回"
        );
        Ok(outcomes)
    }

    /// 把一段这一批在各卡上交的记进这一段的那一份（同一段一批里算了几次的加起来）。
    fn note_the_submissions_of_the_segment(
        &mut self,
        segment: GpuComputedSegment,
        submissions_of_each_card: &[SubmissionsOfACardInTheBatch],
        devices_given_work: Vec<GpuDeviceIdentifier>,
        recuts_after_running_out_of_memory: u64,
    ) {
        let submissions_of_the_segment = GpuSubmissionsOfOneSegment {
            submissions_completed: submissions_of_each_card
                .iter()
                .map(|of_the_card| of_the_card.submissions_completed)
                .sum(),
            pieces_submitted: submissions_of_each_card
                .iter()
                .map(|of_the_card| of_the_card.pieces_submitted)
                .sum(),
            resubmissions_after_timeouts: submissions_of_each_card
                .iter()
                .map(|of_the_card| u64::from(of_the_card.timeouts.resubmissions()))
                .sum(),
            recuts_after_running_out_of_memory,
            devices_given_work,
        };
        self.submissions_since_the_last_take
            .of_the_segment_mut(segment)
            .add(&submissions_of_the_segment);
    }

    /// 一批单元交两份核各算一遍，交回每条两份的结果（按交进来的次序），不比对、不动计数：对拍与 [`Self::verify_unit_checksums`] 都用它。
    /// 按卡的顺序填满各卡（每张卡一段相连的条目），每张卡上再按此刻剩下的显存切段，单独成一段都装不下的单元切成几片、逐片交、
    /// 各片的值两份各按自己的合法合成整单元的（`checksum_kernels::checksum_outcomes_of_the_units_cut_into_pieces`）；分活、切段、超时、重切照
    /// [`Self::outcomes_of_a_batch_over_the_cards`]。
    ///
    /// # Errors
    /// [`ChecksumBatchFailed`] 的三种。
    pub fn both_kernel_outcomes_of_the_batch<EntryKey>(
        &mut self,
        units: &[UnitChecksumToVerify<'_, EntryKey>],
    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, ChecksumBatchFailed> {
        let unit_lengths: Vec<u64> = units
            .iter()
            .map(|unit| u64::try_from(unit.unit_bytes.len()).expect("装得进 u64"))
            .collect();
        self.outcomes_of_a_batch_over_the_cards(
            GpuComputedSegment::UnitChecksumVerification,
            units.len(),
            |entries, room| {
                pieces_cut_to_fit_on_the_gpu(&unit_lengths[entries], CHECKSUM_CHUNK_FOOTPRINT, room)
            },
            Vec::len,
            |card, entries, chunks_of_pieces, submissions_of_the_card| {
                checksum_outcomes_of_the_units_cut_into_pieces(
                    &units[entries],
                    chunks_of_pieces,
                    |piece_units| {
                        submissions_of_the_card.outcome_of_a_chunk(
                            &mut *card,
                            piece_units.len(),
                            |card_handed_the_chunk| {
                                card_handed_the_chunk
                                    .both_kernel_outcomes_of_a_checksum_chunk(piece_units)
                            },
                        )
                    },
                )
            },
        )
        .map_err(checksum_batch_failed_of)
    }

    /// 扫描里的校验和核对一批（第 8 条）：两份核各算一遍、逐条比对；连续不确定到 N 轮的条目交核心层的 CRC-32C 现判。
    /// 交回逐条的判定与确认、不确定、交 CPU 判三个计数。GPU 出错（含这一批这张卡第 4 次等超过上限）时交回出错，`uncertainty` 不动。
    ///
    /// # Errors
    /// [`ChecksumBatchFailed`] 的三种。
    pub fn verify_unit_checksums<EntryKey: Eq + std::hash::Hash + Clone>(
        &mut self,
        units: &[UnitChecksumToVerify<'_, EntryKey>],
        uncertainty: &mut ConsecutiveUncertaintyCounts<EntryKey>,
    ) -> Result<ReconciledBatch<ChecksumKernelOutcome>, ChecksumBatchFailed> {
        let both_kernel_outcomes = self.both_kernel_outcomes_of_the_batch(units);
        reconciled_unit_checksums(units, both_kernel_outcomes, uncertainty)
    }

    /// 区域扫描里一批记录交两份核各分一次类，交回每条两份的分类（按交进来的次序），不比对、不动计数：对拍与
    /// [`Self::classify_region_records`] 都用它。`records` 都是区域扫描会去读的那一种（`region_record_kernels` 文件头），
    /// `mapping_entries` 是这一版的中央映射条目（与核心层区域扫描建按身份索引的是同一份）。
    /// 按卡的顺序填满各卡，每张卡上再按此刻剩下的显存切段（整份映射每段都占一份；连同一条记录装不下时映射按落点切成几片逐片交，
    /// 最大那条记录连同一个落点上的条目还装不下时单元切成头一片与之后的几片，`region_record_kernels::region_record_plan_on_a_gpu`；
    /// 用户 2026-10-28 定「装不下应该自适应的分片，缩小逐次进行」）；分活、切段、超时、重切照 [`Self::outcomes_of_a_batch_over_the_cards`]。
    ///
    /// # Errors
    /// [`RegionRecordBatchFailed`] 的四种。
    ///
    /// # Panics
    /// 有一条记录不是区域扫描会去读的那一种（`region_record_kernels::record_is_one_the_region_scan_reads`）：调用方先按 R 筛过。
    pub fn both_kernel_classifications_of_the_region_records<EntryKey>(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassify<'_, EntryKey>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<(AllocationInRegion, AllocationInRegion)>, RegionRecordBatchFailed> {
        assert!(
            records
                .iter()
                .all(|record| record_is_one_the_region_scan_reads(region, &record.record)),
            "交进来的记录要在 R 那块盘上、未释放、起点在 R 里、不罩出 R"
        );
        let mapping_tables = MappingTablesForTheKernels::of_the_mapping_entries(mapping_entries)
            .map_err(RegionRecordBatchFailed::MappingKeyLongerThanTheKernelsHold)?;
        self.outcomes_of_a_batch_over_the_cards(
            GpuComputedSegment::RegionRecordClassification,
            records.len(),
            |entries, room| {
                region_record_plan_on_a_gpu(
                    &records[entries],
                    mapping_entries,
                    &mapping_tables,
                    room,
                )
            },
            submissions_of_a_region_record_plan,
            |card, entries, plan, submissions_of_the_card| {
                region_record_classifications_of_the_plan(
                    &records[entries],
                    plan,
                    |chunk_pieces, mapping_slice| {
                        submissions_of_the_card.outcome_of_a_chunk(
                            &mut *card,
                            chunk_pieces.len(),
                            |card_handed_the_chunk| {
                                card_handed_the_chunk.both_kernel_outcomes_of_a_region_record_chunk(
                                    chunk_pieces,
                                    mapping_slice.map(|slice| &slice.tables),
                                )
                            },
                        )
                    },
                )
            },
        )
        .map_err(region_record_batch_failed_of)
    }

    /// 区域扫描里的单元头解析与映射反查一批（第 1、8 条）：两份核各分一次类、逐条比对；连续不确定到 N 轮的记录交核心层的区域扫描现判
    /// （`region_record_kernels::cpu_reference_classification`，按身份的索引只在头一次要它时建）。交回逐条的判定与三个计数。
    /// GPU 出错或这一批交不成时交回出错，`uncertainty` 不动。
    ///
    /// # Errors
    /// [`RegionRecordBatchFailed`] 的四种。
    ///
    /// # Panics
    /// 同 [`Self::both_kernel_classifications_of_the_region_records`]。
    pub fn classify_region_records<EntryKey: Eq + std::hash::Hash + Clone>(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassify<'_, EntryKey>],
        mapping_entries: &[CentralMappingEntry],
        uncertainty: &mut ConsecutiveUncertaintyCounts<EntryKey>,
    ) -> Result<ReconciledBatch<AllocationInRegion>, RegionRecordBatchFailed> {
        let both_kernel_classifications = self.both_kernel_classifications_of_the_region_records(
            region,
            records,
            mapping_entries,
        );
        reconciled_region_record_classifications(
            region,
            records,
            mapping_entries,
            both_kernel_classifications,
            uncertainty,
        )
    }

    /// 选段打分一批交两份核各数一遍，交回每段两份的活槽数（按 `segments` 的次序，逐条走遍记录那一份在前），不比对、不动计数：对拍与
    /// [`Self::count_live_slots_per_cluster_segment`] 都用它。`segments` 是 `unit_area` 那块盘上的整段（核心层按单元区给的条目键）。
    /// 按卡的顺序填满各卡，每张卡上再按此刻剩下的显存切段（整份记录表每段都占一份；连同一个段装不下时记录表按记录切成几片、各片数出的逐段相加，
    /// `live_slot_kernels::live_slot_plan_on_a_gpu`）；分活、切段、超时、重切照 [`Self::outcomes_of_a_batch_over_the_cards`]。
    ///
    /// # Errors
    /// [`LiveSlotBatchFailed`] 的三种。
    ///
    /// # Panics
    /// 有一个段不在 `unit_area` 那块盘上。
    pub fn both_kernel_live_slots_of_the_cluster_segments(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
    ) -> Result<Vec<(u64, u64)>, LiveSlotBatchFailed> {
        assert!(
            segments
                .iter()
                .all(|segment| segment.device == unit_area.device),
            "交进来的段都在单元区那块盘上"
        );
        self.outcomes_of_a_batch_over_the_cards(
            GpuComputedSegment::LiveSlotsPerClusterSegment,
            segments.len(),
            |entries, room| live_slot_plan_on_a_gpu(allocation_records.len(), entries.len(), room),
            |plan: &Vec<RecordSliceWithItsChunks>| submissions_of_a_live_slot_plan(plan),
            |card, entries, plan, submissions_of_the_card| {
                live_slots_summed_over_the_record_slices(
                    allocation_records,
                    &segments[entries],
                    plan,
                    |chunk_segments, _records_of_the_slice, record_tables| {
                        submissions_of_the_card.outcome_of_a_chunk(
                            &mut *card,
                            chunk_segments.len(),
                            |card_handed_the_chunk| {
                                card_handed_the_chunk.both_kernel_live_slots_of_a_chunk(
                                    chunk_segments,
                                    record_tables,
                                    unit_area,
                                )
                            },
                        )
                    },
                )
            },
        )
        .map_err(live_slot_batch_failed_of)
    }

    /// 选段打分一批（第 1、8 条）：两份核各数一遍每段的活槽数、逐段比对；连续不确定到 N 轮的段交核心层的
    /// `live_slots_per_cluster_segment_by_the_records` 现判（`live_slot_kernels::cpu_reference_live_slots`）。交回逐段的判定与三个计数。
    /// GPU 出错或这一批交不成时交回出错，`uncertainty` 不动。
    ///
    /// # Errors
    /// [`LiveSlotBatchFailed`] 的三种。
    ///
    /// # Panics
    /// `segments` 不是 `unit_area` 的整段按起点升序一个不漏（`live_slot_kernels::reconciled_live_slots_of_the_segments`）。
    pub fn count_live_slots_per_cluster_segment(
        &mut self,
        allocation_records: &[AllocationRecord],
        unit_area: UnitAreaOfOneDevice,
        segments: &[ClusterSegmentEntryKey],
        uncertainty: &mut ConsecutiveUncertaintyCounts<ClusterSegmentEntryKey>,
    ) -> Result<ReconciledBatch<u64>, LiveSlotBatchFailed> {
        let both_kernel_live_slots = self.both_kernel_live_slots_of_the_cluster_segments(
            allocation_records,
            unit_area,
            segments,
        );
        reconciled_live_slots_of_the_segments(
            allocation_records,
            unit_area,
            segments,
            both_kernel_live_slots,
            uncertainty,
        )
    }
}

/// 丢后台 GPU（卸载、丢掉核心层收的把手、测试里展开）：不等还在 GPU 上跑的提交（用户 2026-10-28 定「卸载时同样不等、泄漏设备」）——
/// 还记着一次等超过上限、没做完的提交的卡整张泄漏（设备、队列、管线都不析构：wgpu-core 30.0.1 的 `Queue` 析构调 `wait_for_idle`，
/// 即 Vulkan 的 `vkQueueWaitIdle`，不带上限），别的卡照常丢（`gpus_of_this_host::gpus_dropped_leaking_those_still_running`）。
/// 泄漏了哪几张、各占着多少字节报给调用方（[`BackgroundGpuOverTheCards::gpus_leaked_when_dropped`]），并记进进程里的那一份，新一次挂载先扣掉。
impl<Card: CardOfTheBackgroundGpu> Drop for BackgroundGpuOverTheCards<Card> {
    fn drop(&mut self) {
        let cards_on_this_host = std::mem::take(&mut self.cards_on_this_host);
        let gpus_leaked = gpus_dropped_leaking_those_still_running(
            cards_on_this_host,
            |card| {
                card.budget()
                    .memory_account()
                    .has_a_submission_running_past_its_wait()
            },
            |card| GpuLeakedWhenDropped {
                device: card.discrete_gpu().identifier.clone(),
                bytes_held_on_the_device: card.budget().memory_account().bytes_held(),
                bytes_leaked_by_earlier_mounts: card.budget().bytes_leaked_by_earlier_mounts(),
            },
        );
        self.memory_leaked_in_this_process
            .note_the_gpus_leaked_when_dropped(&gpus_leaked);
        self.gpus_leaked_when_dropped.note_the_drop(gpus_leaked);
    }
}

/// 卡清单（核心层挂载与切换照它校验设置，D26（后台整理与放置回收） 已定项 13 第 2、4 条）：此刻本机在的独显与各自的显存总量，按总线地址排
/// （挂载那一刻没开成的照列；没给 `background_gpu_devices` 时上限按这个次序一张卡一个），连同没开成的那几张与原因（[`DiscreteGpusAfterTheOpening`]）。
/// 答卡清单之前现查本机在的独显（[`DiscreteGpusPresentNow`]）：挂载之后掉了的卡不列，切换点它照「点名的卡不存在」拒、不点名时它不参与、一张都不剩照「GPU 不在」拒。
/// 没开成的那几张是挂载那一刻定下的，答它不现查（判决 m4-gpugc-code-r2 第三节第 1 条：一次切换只现枚举一次，判与取参与的卡读同一份；
/// 两样各现查一次时，第二次枚举答空就把点名一张没开成的卡的切换收下了），掉了的没开成的卡照列，它不在卡清单里、不参与。
/// 切换可以点开包时没参与而开成了的卡；参与的卡里有没开成的照第 4 条拒。
impl<Card: CardOfTheBackgroundGpu> GpuInventory for BackgroundGpuOverTheCards<Card> {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        self.discrete_gpus_after_the_opening
            .still_present(
                &self
                    .discrete_gpus_present_now
                    .identifiers_of_the_discrete_gpus_present_now(),
            )
            .devices_with_total_memory()
    }

    fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
        self.discrete_gpus_after_the_opening
            .devices_not_opened_at_the_mount()
    }
}

/// 这条线程到此刻用掉的 CPU 时间（`CLOCK_THREAD_CPUTIME_ID`）：线程睡着、等 GPU 时不走。
///
/// # Panics
/// `clock_gettime` 报错：Linux 上这个钟总在。
#[must_use]
#[allow(unsafe_code, reason = "std 没有线程 CPU 钟，只能调 clock_gettime(2)")]
pub fn host_cpu_time_of_this_thread() -> std::time::Duration {
    let mut thread_cpu_time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `thread_cpu_time` 是这个栈帧里一个初始化过的 `timespec`，`clock_gettime` 只往它里面写，调用期间没有别的引用。
    let status =
        unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &raw mut thread_cpu_time) };
    assert_eq!(
        status, 0,
        "clock_gettime(CLOCK_THREAD_CPUTIME_ID) 在 Linux 上总读得到"
    );
    std::time::Duration::new(
        u64::try_from(thread_cpu_time.tv_sec).expect("CPU 时间的秒数不为负"),
        u32::try_from(thread_cpu_time.tv_nsec).expect("纳秒部分在 0 到 10^9 之间"),
    )
}

/// 本机开机以来的 `CLOCK_MONOTONIC_RAW`：单调不减，不受 NTP 调频（用户 2026-11-05 定）。
/// `std::time::Instant` 读的 `CLOCK_MONOTONIC` 跟着 NTP 调频走，线程 CPU 钟（[`host_cpu_time_of_this_thread`]）不跟，
/// 两样成对相减时差出几十 ppm；要拿挂钟与线程 CPU 钟比大小的读这一个。
///
/// # Panics
/// `clock_gettime` 报错：Linux 上这个钟总在。
#[must_use]
#[allow(
    unsafe_code,
    reason = "std 的 Instant 读 CLOCK_MONOTONIC，没有不受 NTP 调频的单调钟，只能调 clock_gettime(2)"
)]
pub fn monotonic_raw_time_now() -> std::time::Duration {
    let mut monotonic_raw_time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `monotonic_raw_time` 是这个栈帧里一个初始化过的 `timespec`，`clock_gettime` 只往它里面写，调用期间没有别的引用。
    let status =
        unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC_RAW, &raw mut monotonic_raw_time) };
    assert_eq!(
        status, 0,
        "clock_gettime(CLOCK_MONOTONIC_RAW) 在 Linux 上总读得到"
    );
    std::time::Duration::new(
        u64::try_from(monotonic_raw_time.tv_sec).expect("开机以来的秒数不为负"),
        u32::try_from(monotonic_raw_time.tv_nsec).expect("纳秒部分在 0 到 10^9 之间"),
    )
}

/// 阻塞等一个 wgpu 的 future 交出结果（枚举适配器、起设备），最多等 `wait_limit`（按 `clock` 量）：轮询它，没好就把这条线程挂起至多到期限，
/// 等它的唤醒把线程叫回来再轮询。本机的 wgpu 后端这几个 future 一轮询就好；不依赖 `pollster`。
///
/// # Errors
/// 到期限还没好 ⇒ [`GpuFailureCause::WaitForTheGpuExceededTheLimit`]。
pub fn wait_for_the_wgpu_future<Output>(
    future: impl Future<Output = Output>,
    wait_limit: Duration,
    clock: &impl MonotonicClock,
) -> Result<Output, GpuFailureCause> {
    let waker = Waker::from(Arc::new(UnparkTheWaitingThread(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut pinned_future = std::pin::pin!(future);
    let deadline = GpuWaitDeadline::starting_now(wait_limit, clock);
    wait_for_completion_before_the_deadline(&deadline, clock, |time_left| {
        match pinned_future.as_mut().poll(&mut context) {
            Poll::Ready(output) => CompletionCheck::Completed(output),
            Poll::Pending => {
                std::thread::park_timeout(time_left);
                CompletionCheck::NotYetComplete
            }
        }
    })
}

struct UnparkTheWaitingThread(std::thread::Thread);

impl Wake for UnparkTheWaitingThread {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// 没被叫醒过时交 Pending 并当场叫醒自己，再被轮询时交出结果。
    struct ReadyAfterWakingItselfOnce {
        polled_before: bool,
    }

    impl Future for ReadyAfterWakingItselfOnce {
        type Output = u32;

        fn poll(mut self: std::pin::Pin<&mut Self>, context: &mut Context<'_>) -> Poll<u32> {
            if self.polled_before {
                return Poll::Ready(42);
            }
            self.polled_before = true;
            context.waker().wake_by_ref();
            Poll::Pending
        }
    }

    /// 永远交 Pending、每次都当场叫醒自己（挂起的线程马上回来）。
    struct NeverReadyButWakingItself;

    impl Future for NeverReadyButWakingItself {
        type Output = u32;

        fn poll(self: std::pin::Pin<&mut Self>, context: &mut Context<'_>) -> Poll<u32> {
            context.waker().wake_by_ref();
            Poll::Pending
        }
    }

    /// 假钟：每读一次走一秒。
    struct ClockMovingOneSecondPerReading {
        readings: Cell<u64>,
    }

    impl MonotonicClock for ClockMovingOneSecondPerReading {
        fn time_since_the_origin(&self) -> Duration {
            self.readings.set(self.readings.get() + 1);
            Duration::from_secs(self.readings.get())
        }
    }

    #[test]
    fn waiting_for_a_future_that_is_pending_once_returns_its_output() {
        assert_eq!(
            wait_for_the_wgpu_future(
                ReadyAfterWakingItselfOnce {
                    polled_before: false
                },
                Duration::from_secs(60),
                &SteadyClockOfThisProcess::starting_now(),
            ),
            Ok(42)
        );
    }

    #[test]
    fn the_wgpu_future_still_pending_at_the_wait_limit_is_a_timeout_failure() {
        let clock = ClockMovingOneSecondPerReading {
            readings: Cell::new(0),
        };
        assert_eq!(
            wait_for_the_wgpu_future(NeverReadyButWakingItself, Duration::from_secs(3), &clock),
            Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: Duration::from_secs(3)
            }),
            "到期限还没好：交回超时出错，不一直等"
        );
    }

    #[test]
    fn the_thread_cpu_clock_does_not_advance_while_the_thread_sleeps() {
        let sleep = std::time::Duration::from_millis(300);
        let cpu_time_before = host_cpu_time_of_this_thread();
        std::thread::sleep(sleep);
        let cpu_time_spent = host_cpu_time_of_this_thread().saturating_sub(cpu_time_before);
        assert!(
            cpu_time_spent < sleep / 3,
            "睡 {sleep:?} 只用掉 {cpu_time_spent:?} 的 CPU：量的是线程 CPU 时间，不是挂钟"
        );
    }

    #[test]
    fn the_thread_cpu_clock_advances_while_the_thread_computes() {
        let cpu_time_before = host_cpu_time_of_this_thread();
        let mut accumulator: u64 = 0;
        let mut rounds: u64 = 0;
        while host_cpu_time_of_this_thread().saturating_sub(cpu_time_before)
            < std::time::Duration::from_millis(20)
        {
            for step in 0..10_000_u64 {
                accumulator = accumulator
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(step);
            }
            rounds += 1;
            assert!(
                rounds < 1_000_000,
                "算了这么多轮线程 CPU 钟还没走 20 ms：钟没在走"
            );
        }
        std::hint::black_box(accumulator);
    }

    #[test]
    fn the_monotonic_raw_clock_never_goes_backward_over_successive_readings() {
        const SUCCESSIVE_READINGS: usize = 100_000;
        let mut previous_reading = monotonic_raw_time_now();
        for reading_number in 0..SUCCESSIVE_READINGS {
            let reading = monotonic_raw_time_now();
            assert!(
                reading >= previous_reading,
                "读数号 {reading_number} 读到 {reading:?}，比它前一个读数 {previous_reading:?} 小：钟往回走了"
            );
            previous_reading = reading;
        }
    }

    #[test]
    fn the_monotonic_raw_clock_keeps_running_while_the_thread_sleeps() {
        let sleep = Duration::from_millis(100);
        let monotonic_raw_time_before = monotonic_raw_time_now();
        std::thread::sleep(sleep);
        let elapsed = monotonic_raw_time_now()
            .checked_sub(monotonic_raw_time_before)
            .expect("CLOCK_MONOTONIC_RAW 单调不减");
        assert!(
            elapsed >= sleep / 2,
            "睡 {sleep:?} 这个钟只走了 {elapsed:?}：读的是挂钟，不是线程 CPU 时间"
        );
    }

    /// 读序同包含自检：挂钟在外、线程 CPU 钟在里，读钟那几次的开销只落在挂钟里。一窗只自旋 1.5 ms 的线程 CPU：
    /// 两样钟走速差几十 ppm 在这么短的窗里只差几十纳秒，小于读钟的开销；按滴答走的粗钟在这么短的窗里常常一格都不走，挂钟就短于 CPU 钟。
    #[test]
    fn the_monotonic_raw_clock_runs_at_least_as_far_as_the_thread_cpu_clock_over_a_spin() {
        const SPIN_WINDOWS: usize = 40;
        const THREAD_CPU_OF_ONE_WINDOW: Duration = Duration::from_micros(1500);
        let mut windows_with_the_wall_behind_the_thread_cpu: Vec<(Duration, Duration)> = Vec::new();
        for _spin_window in 0..SPIN_WINDOWS {
            let wall_at_the_opening = monotonic_raw_time_now();
            let thread_cpu_at_the_opening = host_cpu_time_of_this_thread();
            let mut thread_cpu_at_the_closing = thread_cpu_at_the_opening;
            while thread_cpu_at_the_closing.saturating_sub(thread_cpu_at_the_opening)
                < THREAD_CPU_OF_ONE_WINDOW
            {
                std::hint::spin_loop();
                thread_cpu_at_the_closing = host_cpu_time_of_this_thread();
            }
            let wall = monotonic_raw_time_now()
                .checked_sub(wall_at_the_opening)
                .expect("CLOCK_MONOTONIC_RAW 单调不减");
            let thread_cpu = thread_cpu_at_the_closing.saturating_sub(thread_cpu_at_the_opening);
            if wall < thread_cpu {
                windows_with_the_wall_behind_the_thread_cpu.push((wall, thread_cpu));
            }
        }
        assert!(
            windows_with_the_wall_behind_the_thread_cpu.is_empty(),
            "这几窗（挂钟，线程 CPU）里挂钟走得比线程 CPU 钟少：{windows_with_the_wall_behind_the_thread_cpu:?}"
        );
    }

    use std::collections::VecDeque;

    use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
    use singlefs_core::allocator::{AllocationRecordQuarantineMark, UnitAreaStart};
    use singlefs_core::background_compute::admit_gpu_settings_against_the_inventory;
    use singlefs_core::background_compute::{
        BackgroundComputeBackend, BackgroundComputeSettings, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    };

    use crate::checksum_kernels::cpu_reference_outcome;
    use crate::live_slot_kernels::live_slot_chunk_footprint;
    use crate::memory_limit_and_chunks::{GpuMemoryAccount, SubmissionOfTheChunk};
    use crate::two_kernel_reconciliation::EntryDecision;

    const WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU: Duration = Duration::from_secs(10);
    const EIGHT_GIBIBYTES: u64 = 8 << 30;

    /// 替身卡上的钟不走：替身等前一次提交照脚本当场交回，不真等。
    struct ClockOfTheCardsOnTheCpu;

    impl MonotonicClock for ClockOfTheCardsOnTheCpu {
        fn time_since_the_origin(&self) -> Duration {
            Duration::ZERO
        }
    }

    /// 在 CPU 上算的替身卡：两份核都照 CPU 参照算（校验和那一段 `cpu_reference_outcome`；区域扫描与选段打分那两段只交形状对的结果，用例只看交没交成、
    /// 交了几次），账与产品那张卡同一份（`GpuBudgetUnderTheSettings`）。按脚本造三样：交活之前、重交之前等前一次超时的那次提交等满上限没等到
    /// （`earlier_submission_waits_timing_out`，按次序，`true` 是没等到；卡上记着一次还在跑的提交时才等、才取脚本），每一次提交等超过上限
    /// （`submissions_timing_out`，按次序，`true` 是超时，超时的那一段的账照产品记成还在跑的那次提交），卡上空闲的显存（`free_bytes`：一段合占的比它多就建不成缓冲）。
    /// 脚本用完之后都等到、都做完。另记量了几次代价（`cost_measurements`），可点名第几次交成的校验和那一段交回翻过的判定
    /// （`checksum_submission_with_flipped_verdicts`，从 1 数：那一次的结果是坏的，用了它判定就与 CPU 参照不同）。
    struct CardComputedOnTheCpuForTests {
        discrete_gpu: DiscreteGpuOnThisHost,
        budget: GpuBudgetUnderTheSettings<u32>,
        measured_cost: Option<HostCpuNanosecondsPerSubmission>,
        earlier_submission_waits_timing_out: VecDeque<bool>,
        submissions_timing_out: VecDeque<bool>,
        free_bytes: Option<u64>,
        submissions_made: u32,
        cost_measurements: u32,
        checksum_submission_with_flipped_verdicts: Option<u32>,
    }

    impl CardComputedOnTheCpuForTests {
        /// 总线地址 `pci_bus_address` 的一张 8 GiB 替身卡，量过代价，给它的显存上限 `memory_limit_in_bytes`，这个进程里此前泄漏在它上面 `bytes_leaked` 字节。
        fn of_the_address(
            pci_bus_address: &str,
            memory_limit_in_bytes: Option<u64>,
            bytes_leaked: u64,
        ) -> Self {
            Self {
                discrete_gpu: DiscreteGpuOnThisHost {
                    position_in_the_enumeration: 0,
                    identifier: card_identifier(pci_bus_address),
                    pci_bus_address: pci_bus_address.to_string(),
                    name: format!("cpu card at {pci_bus_address}"),
                    total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
                },
                budget: GpuBudgetUnderTheSettings::with_the_bytes_leaked_by_earlier_mounts(
                    1 << 30,
                    1 << 20,
                    memory_limit_in_bytes.map(GpuMemoryInBytes),
                    WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                    bytes_leaked,
                ),
                measured_cost: Some(HostCpuNanosecondsPerSubmission(1_000)),
                earlier_submission_waits_timing_out: VecDeque::new(),
                submissions_timing_out: VecDeque::new(),
                free_bytes: None,
                submissions_made: 0,
                cost_measurements: 0,
                checksum_submission_with_flipped_verdicts: None,
            }
        }

        /// 卡上记着一次等超过上限、还在跑的提交（占 `bytes` 字节），交活之前要先等它。
        fn with_a_submission_running_past_its_wait(mut self, bytes: u64) -> Self {
            self.budget.memory_account_mut().hold(bytes);
            self.budget
                .memory_account_mut()
                .settle_the_chunk_after_its_submission(
                    bytes,
                    SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(0),
                );
            self
        }

        /// 交一段（合占 `bytes_held_by_the_chunk` 字节）：照产品记账；空闲的不够就放掉、交回显存不足；照脚本超时就把这一段记成还在跑的那次提交、交回超时；
        /// 做完了放账、交回 `outcome`。
        fn chunk_submitted_on_the_cpu<Outcome>(
            &mut self,
            bytes_held_by_the_chunk: u64,
            outcome: impl FnOnce() -> Outcome,
        ) -> Result<Outcome, GpuFailureCause> {
            self.budget
                .memory_account_mut()
                .hold(bytes_held_by_the_chunk);
            if self
                .free_bytes
                .is_some_and(|free_bytes| bytes_held_by_the_chunk > free_bytes)
            {
                self.budget
                    .memory_account_mut()
                    .release(bytes_held_by_the_chunk);
                return Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
                    bytes_held_by_the_chunk,
                    reason: "替身卡上空闲的不够".to_string(),
                });
            }
            self.submissions_made += 1;
            if self.submissions_timing_out.pop_front().unwrap_or(false) {
                self.budget
                    .memory_account_mut()
                    .settle_the_chunk_after_its_submission(
                        bytes_held_by_the_chunk,
                        SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(self.submissions_made),
                    );
                return Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                    wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                });
            }
            self.budget
                .memory_account_mut()
                .settle_the_chunk_after_its_submission(
                    bytes_held_by_the_chunk,
                    SubmissionOfTheChunk::CompletedOnTheGpu,
                );
            Ok(outcome())
        }

        /// 卡上记着一次还在跑的提交、脚本这一回说没等到：交回等满上限。
        fn earlier_submission_still_running_this_time(&mut self) -> bool {
            self.budget
                .memory_account()
                .has_a_submission_running_past_its_wait()
                && self
                    .earlier_submission_waits_timing_out
                    .pop_front()
                    .unwrap_or(false)
        }
    }

    impl CardOfTheBackgroundGpu for CardComputedOnTheCpuForTests {
        type Submission = u32;

        fn discrete_gpu(&self) -> &DiscreteGpuOnThisHost {
            &self.discrete_gpu
        }

        fn budget(&self) -> &GpuBudgetUnderTheSettings<u32> {
            &self.budget
        }

        fn budget_mut(&mut self) -> &mut GpuBudgetUnderTheSettings<u32> {
            &mut self.budget
        }

        fn measured_cost(&self) -> Option<HostCpuNanosecondsPerSubmission> {
            self.measured_cost
        }

        fn measure_the_cost_of_one_submission(&mut self) -> Result<(), GpuFailureCause> {
            self.measured_cost = Some(HostCpuNanosecondsPerSubmission(1_000));
            self.cost_measurements += 1;
            Ok(())
        }

        fn room_once_the_submission_running_past_its_wait_completed(
            &mut self,
        ) -> Result<ChunkBoundsOfAGpu, GpuFailureCause> {
            if self.earlier_submission_still_running_this_time() {
                return Err(
                    GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
                        wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                    },
                );
            }
            self.budget
                .room_once_the_submission_running_past_its_wait_completed(
                    &ClockOfTheCardsOnTheCpu,
                    |_submission, _time_left| CompletionCheck::Completed(()),
                )
        }

        fn wait_for_the_timed_out_submission(&mut self) -> Result<(), GpuFailureCause> {
            if self.earlier_submission_still_running_this_time() {
                return Err(
                    GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
                        wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                    },
                );
            }
            self.budget.wait_for_the_submission_running_past_its_wait(
                &ClockOfTheCardsOnTheCpu,
                |_submission, _time_left| CompletionCheck::Completed(()),
            )
        }

        fn both_kernel_outcomes_of_a_checksum_chunk(
            &mut self,
            units: &[UnitChecksumToVerify<'_, ()>],
        ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, GpuFailureCause> {
            let input_bytes: usize = units.iter().map(|unit| unit.unit_bytes.len()).sum();
            let bytes_held_by_the_chunk = CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
                u64::try_from(input_bytes).expect("装得进 u64"),
                u64::try_from(units.len()).expect("装得进 u64"),
            );
            let submission_with_flipped_verdicts = self.checksum_submission_with_flipped_verdicts;
            let submission_number = self.submissions_made + 1;
            self.chunk_submitted_on_the_cpu(bytes_held_by_the_chunk, || {
                units
                    .iter()
                    .map(|unit| {
                        let mut outcome =
                            cpu_reference_outcome(unit.unit_bytes, unit.expected_crc32c);
                        if submission_with_flipped_verdicts == Some(submission_number) {
                            outcome.computed_crc32c ^= 1;
                            outcome.matches_the_expected_crc32c =
                                !outcome.matches_the_expected_crc32c;
                        }
                        (outcome, outcome)
                    })
                    .collect()
            })
        }

        fn both_kernel_outcomes_of_a_region_record_chunk(
            &mut self,
            pieces: &[RegionRecordPieceForTheKernels<'_>],
            mapping_tables: Option<&MappingTablesForTheKernels>,
        ) -> Result<BothKernelOutcomesOfARegionRecordChunk, GpuFailureCause> {
            let empty_mapping_tables = MappingTablesForTheKernels::of_the_mapping_entries(&[])
                .expect("空映射没有长过核装得下的 key");
            let input_bytes: usize = pieces
                .iter()
                .map(|piece| piece.bytes().map_or(0, <[u8]>::len))
                .sum();
            let bytes_held_by_the_chunk =
                region_record_chunk_footprint(mapping_tables.unwrap_or(&empty_mapping_tables))
                    .bytes_held_by_a_chunk(
                        u64::try_from(input_bytes).expect("装得进 u64"),
                        u64::try_from(pieces.len()).expect("装得进 u64"),
                    );
            self.chunk_submitted_on_the_cpu(bytes_held_by_the_chunk, || match mapping_tables {
                None => BothKernelOutcomesOfARegionRecordChunk::RegistersAfterThePieces(vec![
                    CrcRegistersOfTheTwoKernels::BEFORE_ANY_BYTE;
                    pieces.len()
                ]),
                Some(_tables) => BothKernelOutcomesOfARegionRecordChunk::Classifications(vec![
                    (AllocationInRegion::Pinned, AllocationInRegion::Pinned);
                    pieces.len()
                ]),
            })
        }

        fn both_kernel_live_slots_of_a_chunk(
            &mut self,
            segments: &[ClusterSegmentEntryKey],
            record_tables: &AllocationRecordTablesForTheKernels,
            _unit_area: UnitAreaOfOneDevice,
        ) -> Result<Vec<(u64, u64)>, GpuFailureCause> {
            let segment_count = u64::try_from(segments.len()).expect("装得进 u64");
            let bytes_held_by_the_chunk = live_slot_chunk_footprint(record_tables)
                .bytes_held_by_a_chunk(segment_count * SEGMENT_BOUND_BYTES, segment_count);
            self.chunk_submitted_on_the_cpu(bytes_held_by_the_chunk, || {
                vec![(0, 0); segments.len()]
            })
        }
    }

    fn card_identifier(pci_bus_address: &str) -> GpuDeviceIdentifier {
        GpuDeviceIdentifier::of_opaque_bytes(pci_bus_address.as_bytes().to_vec())
    }

    /// 替身的「此刻本机在的独显」：用例改它模拟挂载之后掉了一张卡。
    struct DiscreteGpusPresentForTests(Arc<Mutex<Vec<GpuDeviceIdentifier>>>);

    impl DiscreteGpusPresentNow for DiscreteGpusPresentForTests {
        fn identifiers_of_the_discrete_gpus_present_now(&self) -> Vec<GpuDeviceIdentifier> {
            self.0.lock().expect("用例没在别处 panic 过").clone()
        }
    }

    /// 外壳装上这几张替身卡（按交进来的次序就是本机的次序，全都参与），进程里泄漏的那一份是 `memory_leaked`；交回外壳与「此刻本机在的独显」那一份（起初全在）。
    fn background_gpu_over_cards_on_the_cpu(
        cards: Vec<CardComputedOnTheCpuForTests>,
        memory_leaked: &GpuMemoryLeakedInThisProcess,
    ) -> (
        BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
        Arc<Mutex<Vec<GpuDeviceIdentifier>>>,
    ) {
        let discrete_gpus: Vec<DiscreteGpuOnThisHost> =
            cards.iter().map(|card| card.discrete_gpu.clone()).collect();
        let present = Arc::new(Mutex::new(
            discrete_gpus
                .iter()
                .map(|discrete_gpu| discrete_gpu.identifier.clone())
                .collect(),
        ));
        let participating =
            ParticipatingGpusOnThisHost::of_the_positions((0..cards.len()).collect());
        (
            BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
                cards,
                DiscreteGpusAfterTheOpening {
                    opened: discrete_gpus,
                    not_opened: Vec::new(),
                },
                Vec::new(),
                participating,
                Vec::new(),
                memory_leaked.clone(),
                Box::new(DiscreteGpusPresentForTests(Arc::clone(&present))),
            ),
            present,
        )
    }

    /// `count` 个单元，第 i 个是 `bytes_per_unit` 个字节的 i，期望值照 CPU 参照算对。
    fn unit_bytes_of(count: usize, bytes_per_unit: usize) -> Vec<Vec<u8>> {
        (0..count)
            .map(|unit| vec![u8::try_from(unit % 251).expect("小于 256"); bytes_per_unit])
            .collect()
    }

    fn units_to_verify(unit_bytes: &[Vec<u8>]) -> Vec<UnitChecksumToVerify<'_, usize>> {
        unit_bytes
            .iter()
            .enumerate()
            .map(|(entry_key, bytes)| UnitChecksumToVerify {
                entry_key,
                unit_bytes: bytes,
                expected_crc32c: singlefs_core::checksum::crc32_castagnoli(bytes),
            })
            .collect()
    }

    fn cpu_reference_outcomes_of(
        units: &[UnitChecksumToVerify<'_, usize>],
    ) -> Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)> {
        units
            .iter()
            .map(|unit| {
                let outcome = cpu_reference_outcome(unit.unit_bytes, unit.expected_crc32c);
                (outcome, outcome)
            })
            .collect()
    }

    /// 判决 m4-gpugc-code-r1 第三节第 1 条：校验和那一段一批切成 4 段（上限 1100 字节只装得下一个 1000 字节的单元），每段头一次交都等超过上限、
    /// 重交一次就做完——这一批在这张卡上第 4 次超时报这张卡用不了。这一段记下交成 3 次、3 片、重交 3 次、分到活的是这张卡。
    #[test]
    fn the_checksum_batch_cut_into_four_chunks_each_timing_out_once_reports_the_card_unusable() {
        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", Some(1100), 0);
        card.submissions_timing_out = VecDeque::from([true, false, true, false, true, false, true]);
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let unit_bytes = unit_bytes_of(4, 1000);
        assert_eq!(
            background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes)),
            Err(ChecksumBatchFailed::GpuFailed {
                device: card_identifier("0000:01:00.0"),
                cause: GpuFailureCause::UnusableAfterResubmissions {
                    resubmissions: 3,
                    last_cause: Box::new(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                        wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                    }),
                },
            }),
            "四段各超时一次：这一批第 4 次超时报警"
        );
        assert_eq!(
            background_gpu
                .take_the_submissions_of_the_batch()
                .unit_checksum_verification,
            GpuSubmissionsOfOneSegment {
                submissions_completed: 3,
                pieces_submitted: 3,
                resubmissions_after_timeouts: 3,
                recuts_after_running_out_of_memory: 0,
                devices_given_work: vec![card_identifier("0000:01:00.0")],
            }
        );
    }

    /// 同一条：一批 1 片超时 3 次、第 4 次交做完——照常交回，结果与 CPU 参照逐条相同；这一段记下交成 1 次、1 片、重交 3 次。
    #[test]
    fn the_checksum_batch_of_one_piece_timing_out_three_times_then_completing_is_returned() {
        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        card.submissions_timing_out = VecDeque::from([true, true, true]);
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let unit_bytes = unit_bytes_of(1, 1000);
        let units = units_to_verify(&unit_bytes);
        assert_eq!(
            background_gpu.both_kernel_outcomes_of_the_batch(&units),
            Ok(cpu_reference_outcomes_of(&units))
        );
        assert_eq!(
            background_gpu
                .take_the_submissions_of_the_batch()
                .unit_checksum_verification,
            GpuSubmissionsOfOneSegment {
                submissions_completed: 1,
                pieces_submitted: 1,
                resubmissions_after_timeouts: 3,
                recuts_after_running_out_of_memory: 0,
                devices_given_work: vec![card_identifier("0000:01:00.0")],
            }
        );
    }

    /// 两条区域扫描会去读的记录（R 是第一块盘上槽 57600 起那一段）与空映射。
    fn two_region_records_with_an_empty_mapping() -> (CompactionRegion, Vec<AllocationRecord>) {
        let region =
            CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(64 * 900))
                .expect("64 的倍数");
        let records = (0..2_u64)
            .map(|offset| AllocationRecord {
                device: DeviceIdentity(0),
                slot: SlotNumber(region.start_slot().0 + offset),
                span_slots: 1,
                generation: CheckpointTxg(7),
                is_released: false,
                quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
            })
            .collect();
        (region, records)
    }

    /// 同一条在区域扫描那一段的接线：卡上记着前一次超时、还在跑的提交，交活之前等了两回没等到、第三回等到，再交一段超时一次、重交做完——
    /// 这一批在这张卡上超时 3 次，照常交回；这一段记下交成 1 次、2 片、重交 1 次。交活之前那一等不算进超时、或这一段不套重交，都交回出错。
    #[test]
    fn the_region_record_batch_waiting_twice_before_handing_over_and_timing_out_once_is_returned() {
        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0)
            .with_a_submission_running_past_its_wait(100);
        card.earlier_submission_waits_timing_out = VecDeque::from([true, true]);
        card.submissions_timing_out = VecDeque::from([true]);
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let (region, records) = two_region_records_with_an_empty_mapping();
        let unit_bytes = [7_u8; 64];
        let to_classify: Vec<RegionRecordToClassify<'_, usize>> = records
            .iter()
            .enumerate()
            .map(|(entry_key, record)| RegionRecordToClassify {
                entry_key,
                record: *record,
                unit_bytes: Some(&unit_bytes),
            })
            .collect();
        assert_eq!(
            background_gpu
                .both_kernel_classifications_of_the_region_records(region, &to_classify, &[])
                .map(|classifications| classifications.len()),
            Ok(2),
            "交活之前两回加这一段一回，第 3 次超时之内：照常交回"
        );
        assert_eq!(
            background_gpu
                .take_the_submissions_of_the_batch()
                .region_record_classification,
            GpuSubmissionsOfOneSegment {
                submissions_completed: 1,
                pieces_submitted: 2,
                resubmissions_after_timeouts: 1,
                recuts_after_running_out_of_memory: 0,
                devices_given_work: vec![card_identifier("0000:01:00.0")],
            }
        );
    }

    /// 同一条在选段打分那一段的接线：同上一条的时序，两个段。
    #[test]
    fn the_live_slot_batch_waiting_twice_before_handing_over_and_timing_out_once_is_returned() {
        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0)
            .with_a_submission_running_past_its_wait(100);
        card.earlier_submission_waits_timing_out = VecDeque::from([true, true]);
        card.submissions_timing_out = VecDeque::from([true]);
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let unit_area = UnitAreaOfOneDevice {
            device: DeviceIdentity(0),
            start: UnitAreaStart::of_the_default_journal_ring(),
            slots: 128,
        };
        let segments: Vec<ClusterSegmentEntryKey> = (0..2_u64)
            .map(|segment| ClusterSegmentEntryKey {
                device: DeviceIdentity(0),
                start_slot: SlotNumber(unit_area.start.slot().0 + 64 * segment),
            })
            .collect();
        assert_eq!(
            background_gpu.both_kernel_live_slots_of_the_cluster_segments(
                &[],
                unit_area,
                &segments
            ),
            Ok(vec![(0, 0), (0, 0)]),
            "交活之前两回加这一段一回，第 3 次超时之内：照常交回"
        );
        assert_eq!(
            background_gpu
                .take_the_submissions_of_the_batch()
                .live_slots_per_cluster_segment,
            GpuSubmissionsOfOneSegment {
                submissions_completed: 1,
                pieces_submitted: 2,
                resubmissions_after_timeouts: 1,
                recuts_after_running_out_of_memory: 0,
                devices_given_work: vec![card_identifier("0000:01:00.0")],
            }
        );
    }

    /// 判决第三节第 4 条在外壳上：两张卡，「0000:01:00.0」上限 1000 字节而这个进程里此前泄漏了 950 字节，余量 50 字节装不下校验和那一段最小的一片（64 字节）——
    /// 这一批不分给它，两条全交另一张卡，结果与 CPU 参照逐条相同；这一段记下分到活的只有「0000:41:00.0」。
    #[test]
    fn the_checksum_batch_goes_wholly_to_the_other_card_when_one_card_is_short_of_the_smallest_piece(
    ) {
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![
                CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", Some(1000), 950),
                CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
            ],
            &memory_leaked,
        );
        let unit_bytes = unit_bytes_of(2, 100);
        let units = units_to_verify(&unit_bytes);
        assert_eq!(
            background_gpu.both_kernel_outcomes_of_the_batch(&units),
            Ok(cpu_reference_outcomes_of(&units))
        );
        assert_eq!(
            background_gpu
                .take_the_submissions_of_the_batch()
                .unit_checksum_verification,
            GpuSubmissionsOfOneSegment {
                submissions_completed: 1,
                pieces_submitted: 2,
                resubmissions_after_timeouts: 0,
                recuts_after_running_out_of_memory: 0,
                devices_given_work: vec![card_identifier("0000:41:00.0")],
            }
        );
    }

    /// 判决第三节第 5 条（第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」）：上限 1 MiB、卡上实际只空闲 256 KiB，八个 64 KiB 的单元
    /// 照上限一段装得下（512 KiB 多）而建缓冲显存不足——压到那一段的一半重切成三、三、二个一段，照常交回，判定与 CPU 参照逐条相同、全都确认；
    /// 这一段记下重切 1 次、交成 3 次、8 片。
    #[test]
    fn the_checksum_batch_on_a_card_with_less_free_memory_than_its_limit_is_recut_and_agrees_with_the_cpu(
    ) {
        let mut card =
            CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", Some(1 << 20), 0);
        card.free_bytes = Some(256 << 10);
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let unit_bytes = unit_bytes_of(8, 64 << 10);
        let units = units_to_verify(&unit_bytes);
        let reconciled = background_gpu
            .verify_unit_checksums(
                &units,
                &mut ConsecutiveUncertaintyCounts::with_no_entry_uncertain(),
            )
            .expect("显存不足按更小的片重切重交：不报出错");
        assert_eq!(
            reconciled.decisions,
            cpu_reference_outcomes_of(&units)
                .into_iter()
                .map(|(outcome, _same)| EntryDecision::ConfirmedByBothKernels(outcome))
                .collect::<Vec<_>>(),
            "判定与 CPU 参照逐条相同"
        );
        assert_eq!(
            background_gpu
                .take_the_submissions_of_the_batch()
                .unit_checksum_verification,
            GpuSubmissionsOfOneSegment {
                submissions_completed: 3,
                pieces_submitted: 8,
                resubmissions_after_timeouts: 0,
                recuts_after_running_out_of_memory: 1,
                devices_given_work: vec![card_identifier("0000:01:00.0")],
            }
        );
    }

    /// 只空闲 10 字节的那张替身卡上一个 1000 字节的单元一路压小（1060、528、264、132、64 字节的一段各建不成一次）到连最小的一片（64 字节）都建不成时，
    /// 它报的那一条：最后建不成的一段 64 字节、最小的一片 64 字节。
    fn short_of_free_memory_after_cutting_down_a_unit_of_one_thousand_bytes(
        pci_bus_address: &str,
    ) -> GpuShortOfFreeMemoryForTheSmallestPiece {
        GpuShortOfFreeMemoryForTheSmallestPiece {
            device: card_identifier(pci_bus_address),
            bytes_of_the_last_chunk_not_created: 64,
            smallest_piece_bytes: 64,
        }
    }

    /// 判决第三节第 5 条连同用户 2026-10-28「连一片都装不下就不用这张卡」：唯一一张卡上只空闲 10 字节，一路压小到连最小的一片（64 字节）都建不成——这一批不用这张卡，参与的卡一张都不剩，
    /// 报空闲显存那一种（它是因空闲显存不用的那一张，没有因显存上限不用的），重切了 5 次。
    #[test]
    fn the_single_card_short_of_free_memory_for_the_smallest_piece_leaves_every_participating_gpu_short(
    ) {
        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        card.free_bytes = Some(10);
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let unit_bytes = unit_bytes_of(1, 1000);
        assert_eq!(
            background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes)),
            Err(
                ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
                    gpus_short_of_free_memory: vec![
                        short_of_free_memory_after_cutting_down_a_unit_of_one_thousand_bytes(
                            "0000:01:00.0"
                        )
                    ],
                    gpus_left_out_under_their_memory_limit: Vec::new(),
                }
            )
        );
        assert_eq!(
            background_gpu
                .take_the_submissions_of_the_batch()
                .unit_checksum_verification
                .recuts_after_running_out_of_memory,
            5,
            "1060、528、264、132、64 字节的一段各建不成一次"
        );
    }

    /// 一张卡的上限一次只交得下一个 1000 字节的单元（1060 字节）：按卡的顺序它填一个，剩下的才给后面的卡。
    const MEMORY_LIMIT_HOLDING_ONE_UNIT_OF_ONE_THOUSAND_BYTES: u64 = 1060;

    /// 用户 2026-10-28「连一片都装不下就不用这张卡」用于空闲显存：两张卡、两个 1000 字节的单元，排在前面的「0000:01:00.0」的上限一次只交得下一个，
    /// 按卡的顺序它填一个、另一个给「0000:41:00.0」；「0000:01:00.0」只空闲 10 字节、压到最小的一片都建不成——这一批不用它，它分到的那一条转给
    /// 「0000:41:00.0」重交（接在后面交、按条目的次序接回），照常交回、与 CPU 参照逐条相同；这一段记下交成 2 次、2 片、重切 5 次、分到活的是两张卡（先「0000:01:00.0」）。
    #[test]
    fn the_checksum_batch_goes_to_the_other_card_when_one_card_is_short_of_free_memory_for_the_smallest_piece(
    ) {
        let mut card_short_of_free_memory = CardComputedOnTheCpuForTests::of_the_address(
            "0000:01:00.0",
            Some(MEMORY_LIMIT_HOLDING_ONE_UNIT_OF_ONE_THOUSAND_BYTES),
            0,
        );
        card_short_of_free_memory.free_bytes = Some(10);
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![
                card_short_of_free_memory,
                CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
            ],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let unit_bytes = unit_bytes_of(2, 1000);
        let units = units_to_verify(&unit_bytes);
        assert_eq!(
            background_gpu.both_kernel_outcomes_of_the_batch(&units),
            Ok(cpu_reference_outcomes_of(&units)),
            "一张卡空闲的装不下最小的一片：活转给另一张，照常交回"
        );
        assert_eq!(
            background_gpu
                .take_the_submissions_of_the_batch()
                .unit_checksum_verification,
            GpuSubmissionsOfOneSegment {
                submissions_completed: 2,
                pieces_submitted: 2,
                resubmissions_after_timeouts: 0,
                recuts_after_running_out_of_memory: 5,
                devices_given_work: vec![
                    card_identifier("0000:01:00.0"),
                    card_identifier("0000:41:00.0")
                ],
            }
        );
    }

    /// 同上，两张卡都只空闲 10 字节：两条按卡的顺序先都给排在前面的「0000:01:00.0」、它不用之后都转给「0000:41:00.0」，这一批两张都不用，报空闲显存那一种，按不用的先后列出两张；
    /// 重切了 12 次（一张 6 次）。
    #[test]
    fn the_checksum_batch_whose_cards_are_all_short_of_free_memory_reports_every_participating_gpu_short(
    ) {
        let mut cards = vec![
            CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0),
            CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
        ];
        for card in &mut cards {
            card.free_bytes = Some(10);
        }
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            cards,
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let unit_bytes = unit_bytes_of(2, 1000);
        assert_eq!(
            background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes)),
            Err(
                ChecksumBatchFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
                    gpus_short_of_free_memory: vec![
                        short_of_free_memory_after_cutting_down_a_unit_of_one_thousand_bytes(
                            "0000:01:00.0"
                        ),
                        short_of_free_memory_after_cutting_down_a_unit_of_one_thousand_bytes(
                            "0000:41:00.0"
                        ),
                    ],
                    gpus_left_out_under_their_memory_limit: Vec::new(),
                }
            )
        );
        assert_eq!(
            background_gpu
                .take_the_submissions_of_the_batch()
                .unit_checksum_verification
                .recuts_after_running_out_of_memory,
            12
        );
    }

    /// 「下一批照设置重新起」：两张卡、「0000:01:00.0」的上限一次只交得下一个单元、只空闲 10 字节，两个 1000 字节的单元，这一批不用它；
    /// 下一批（外壳的一批是一段的一次调用）它又照设置排在前面、分到一个（又压小 5 次、又不用它），两条都照常交回。
    #[test]
    fn the_card_short_of_free_memory_is_given_work_again_in_the_next_batch() {
        let mut card_short_of_free_memory = CardComputedOnTheCpuForTests::of_the_address(
            "0000:01:00.0",
            Some(MEMORY_LIMIT_HOLDING_ONE_UNIT_OF_ONE_THOUSAND_BYTES),
            0,
        );
        card_short_of_free_memory.free_bytes = Some(10);
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![
                card_short_of_free_memory,
                CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
            ],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let unit_bytes = unit_bytes_of(2, 1000);
        let units = units_to_verify(&unit_bytes);
        assert_eq!(
            background_gpu.both_kernel_outcomes_of_the_batch(&units),
            Ok(cpu_reference_outcomes_of(&units))
        );
        drop(background_gpu.take_the_submissions_of_the_batch());
        assert_eq!(
            background_gpu.both_kernel_outcomes_of_the_batch(&units),
            Ok(cpu_reference_outcomes_of(&units))
        );
        assert_eq!(
            background_gpu
                .take_the_submissions_of_the_batch()
                .unit_checksum_verification,
            GpuSubmissionsOfOneSegment {
                submissions_completed: 2,
                pieces_submitted: 2,
                resubmissions_after_timeouts: 0,
                recuts_after_running_out_of_memory: 5,
                devices_given_work: vec![
                    card_identifier("0000:01:00.0"),
                    card_identifier("0000:41:00.0")
                ],
            },
            "下一批那张卡又分到活、又压小 5 次"
        );
    }

    /// 判决第三节第 8 条：丢外壳时，记着一次还在跑的提交（占 600 字节）的那张卡泄漏、另一张照常丢；泄漏了哪一张、占着多少交到调用方拿着的那一份，
    /// 并记进进程里的那一份（新一次挂载这张卡先扣掉）。
    #[test]
    fn dropping_the_background_gpu_reports_the_card_leaked_and_notes_it_in_the_process() {
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        let (background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![
                CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0)
                    .with_a_submission_running_past_its_wait(600),
                CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
            ],
            &memory_leaked,
        );
        let gpus_leaked = background_gpu.gpus_leaked_when_dropped();
        assert_eq!(gpus_leaked.gpus_leaked(), None, "还没丢");
        drop(background_gpu);
        assert_eq!(
            gpus_leaked.gpus_leaked(),
            Some(vec![GpuLeakedWhenDropped {
                device: card_identifier("0000:01:00.0"),
                bytes_held_on_the_device: 600,
                bytes_leaked_by_earlier_mounts: 0,
            }]),
            "只泄漏还在跑的那一张"
        );
        assert_eq!(
            (
                memory_leaked.bytes_leaked_on(&card_identifier("0000:01:00.0")),
                memory_leaked.bytes_leaked_on(&card_identifier("0000:41:00.0"))
            ),
            (600, 0)
        );
    }

    /// 判决第三节第 2 条：卡清单每次答之前现查本机在的独显——挂载之后那张卡掉了，不点名的切换照「GPU 不在」拒、点名它照「点名的卡不存在」拒。
    #[test]
    fn the_switch_after_the_card_dropped_off_the_host_is_refused_as_gpu_not_present() {
        let (background_gpu, present) = background_gpu_over_cards_on_the_cpu(
            vec![CardComputedOnTheCpuForTests::of_the_address(
                "0000:01:00.0",
                None,
                0,
            )],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let settings_naming = |named: Option<Vec<GpuDeviceIdentifier>>| BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: named,
            background_gpu_memory_limit: None,
            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        };
        assert!(
            admit_gpu_settings_against_the_inventory(&settings_naming(None), &background_gpu)
                .is_ok(),
            "卡还在：照收"
        );
        present.lock().expect("用例没在别处 panic 过").clear();
        assert_eq!(
            admit_gpu_settings_against_the_inventory(&settings_naming(None), &background_gpu),
            Err(BackgroundComputeSettingsRefused::GpuNotPresent),
            "卡掉了、不点名：GPU 不在"
        );
        assert_eq!(
            admit_gpu_settings_against_the_inventory(
                &settings_naming(Some(vec![card_identifier("0000:01:00.0")])),
                &background_gpu
            ),
            Err(
                BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory {
                    device: card_identifier("0000:01:00.0")
                }
            ),
            "卡掉了、点名它：点名的卡不存在"
        );
    }

    /// 切换那一刻现查的「此刻本机在的独显」：头一次枚举交回 `present_while_nothing_has_enumerated_yet`，之后的枚举一张都没有
    /// （等 wgpu 超过上限当一张都不在，或两次之间掉了卡）；枚举了几次记在 `enumerations`。
    struct DiscreteGpusPresentOnlyUntilTheEnumerationIsRepeated {
        enumerations: Arc<std::sync::atomic::AtomicUsize>,
        present_while_nothing_has_enumerated_yet: Vec<GpuDeviceIdentifier>,
    }

    impl DiscreteGpusPresentNow for DiscreteGpusPresentOnlyUntilTheEnumerationIsRepeated {
        fn identifiers_of_the_discrete_gpus_present_now(&self) -> Vec<GpuDeviceIdentifier> {
            if self
                .enumerations
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                == 0
            {
                self.present_while_nothing_has_enumerated_yet.clone()
            } else {
                Vec::new()
            }
        }
    }

    /// 判决 m4-gpugc-code-r2 第三节第 1 条：本机两张独显，0000:41:00.0 挂载那一刻没开成；切换点名它，现查本机在的独显头一次答两张都在、之后答一张都没有——
    /// 一次切换只现枚举一次（判与取参与的卡读同一份），照第 4 条拒成「参与的卡里有挂载那一刻没开成的」，不收下、不交给实现（交了就在外壳里 panic）。
    #[test]
    fn the_switch_naming_a_card_not_opened_at_the_mount_is_refused_on_a_single_enumeration_though_a_repeated_one_would_come_back_empty(
    ) {
        let opened_card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        let opened = opened_card.discrete_gpu.clone();
        let not_opened = DiscreteGpuNotOpenedAtTheMount {
            discrete_gpu: DiscreteGpuOnThisHost {
                position_in_the_enumeration: 1,
                identifier: card_identifier("0000:41:00.0"),
                pci_bus_address: "0000:41:00.0".to_string(),
                name: "cpu card that did not open at the mount".to_string(),
                total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
            },
            cause: GpuFailureCause::DeviceOrPipelineDidNotStart {
                reason: "替身：挂载那一刻没开成".to_string(),
            },
        };
        let enumerations = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let background_gpu = BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
            vec![opened_card],
            DiscreteGpusAfterTheOpening {
                opened: vec![opened],
                not_opened: vec![not_opened],
            },
            Vec::new(),
            ParticipatingGpusOnThisHost::of_the_positions(vec![0]),
            Vec::new(),
            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            Box::new(DiscreteGpusPresentOnlyUntilTheEnumerationIsRepeated {
                enumerations: Arc::clone(&enumerations),
                present_while_nothing_has_enumerated_yet: vec![
                    card_identifier("0000:01:00.0"),
                    card_identifier("0000:41:00.0"),
                ],
            }),
        );
        let admitted = admit_gpu_settings_against_the_inventory(
            &BackgroundComputeSettings {
                background_compute: BackgroundComputeBackend::Gpu,
                background_gpu_devices: Some(vec![card_identifier("0000:41:00.0")]),
                background_gpu_memory_limit: None,
                background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            },
            &background_gpu,
        );
        assert!(
            matches!(
                &admitted,
                Err(BackgroundComputeSettingsRefused::ParticipatingGpuDeviceNotOpenedAtTheMount(
                    not_opened_at_the_mount
                )) if not_opened_at_the_mount.device == card_identifier("0000:41:00.0")
            ),
            "点名挂载那一刻没开成的卡：拒（{admitted:?}）"
        );
        assert_eq!(
            enumerations.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "一次切换只现枚举一次"
        );
    }

    /// 判决第三节第 2 条在外壳上：校验和那一段一条 1000 字节的单元，头一次交等超过上限，重交之前等前一次那一等又超时 2 回、第 3 回等到再交、做完——
    /// 这一段照常交回、与 CPU 参照相同，这张卡上交了 2 次，这一段记下重交 1 次（只数真交出去的）。
    #[test]
    fn the_checksum_batch_whose_wait_before_the_resubmission_times_out_twice_reports_one_resubmission(
    ) {
        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        card.submissions_timing_out = VecDeque::from([true]);
        card.earlier_submission_waits_timing_out = VecDeque::from([true, true]);
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let unit_bytes = unit_bytes_of(1, 1000);
        let units = units_to_verify(&unit_bytes);
        assert_eq!(
            background_gpu.both_kernel_outcomes_of_the_batch(&units),
            Ok(cpu_reference_outcomes_of(&units)),
            "超时 3 次之后做完：照常交回"
        );
        let submissions = background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification;
        assert_eq!(
            (
                background_gpu.cards_on_this_host[0].submissions_made,
                submissions.resubmissions_after_timeouts
            ),
            (2, 1),
            "这张卡上交了 2 次，重交只数真交出去的那 1 次（{submissions:?}）"
        );
    }

    /// 判决第三节第 4 条在外壳上：同一进程里两次挂载同时开着、都在同一张卡上各有一次等超过上限还在跑的提交（先开的 600 字节、后开的 300 字节，
    /// 开卡那一刻都没算上对方的），两次都丢掉——进程里这张卡记 900（按次累加），新一次挂载这张卡的账从 900 起。
    #[test]
    fn two_mounts_alive_at_once_leaking_on_one_gpu_add_up_through_the_drops_of_the_background_gpu()
    {
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        let card = card_identifier("0000:01:00.0");
        let (mount_leaking_six_hundred_bytes, _present_of_the_mount_leaking_six_hundred_bytes) =
            background_gpu_over_cards_on_the_cpu(
                vec![CardComputedOnTheCpuForTests::of_the_address(
                    "0000:01:00.0",
                    None,
                    memory_leaked.bytes_leaked_on(&card),
                )
                .with_a_submission_running_past_its_wait(600)],
                &memory_leaked,
            );
        let (mount_leaking_three_hundred_bytes, _present_of_the_mount_leaking_three_hundred_bytes) =
            background_gpu_over_cards_on_the_cpu(
                vec![CardComputedOnTheCpuForTests::of_the_address(
                    "0000:01:00.0",
                    None,
                    memory_leaked.bytes_leaked_on(&card),
                )
                .with_a_submission_running_past_its_wait(300)],
                &memory_leaked,
            );
        drop(mount_leaking_six_hundred_bytes);
        drop(mount_leaking_three_hundred_bytes);
        assert_eq!(
            memory_leaked.bytes_leaked_on(&card),
            900,
            "两次各泄漏 600、300 字节：记 900，不让后丢的盖掉先丢的"
        );
        assert_eq!(
            CardComputedOnTheCpuForTests::of_the_address(
                "0000:01:00.0",
                Some(1 << 20),
                memory_leaked.bytes_leaked_on(&card),
            )
            .budget
            .memory_account()
            .bytes_held(),
            900,
            "新一次挂载这张卡的账从 900 起"
        );
    }

    /// 同一条：一次挂载在一张卡上泄漏 600 字节、丢掉；同一张卡新一次挂载开卡那一刻账从 600 起，再有一次等超过上限还在跑的提交（300 字节），
    /// 丢掉时那一刻占着 900——进程里只加这一次自己的 300，记 900，不把开卡时已算上的 600 再加一遍。
    #[test]
    fn the_later_mount_leaking_again_on_the_same_gpu_adds_only_its_own_bytes_in_the_process() {
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        let card = card_identifier("0000:01:00.0");
        let (earlier_mount, _earlier_present) = background_gpu_over_cards_on_the_cpu(
            vec![
                CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0)
                    .with_a_submission_running_past_its_wait(600),
            ],
            &memory_leaked,
        );
        drop(earlier_mount);
        assert_eq!(memory_leaked.bytes_leaked_on(&card), 600);
        let (later_mount, _later_present) = background_gpu_over_cards_on_the_cpu(
            vec![CardComputedOnTheCpuForTests::of_the_address(
                "0000:01:00.0",
                None,
                memory_leaked.bytes_leaked_on(&card),
            )
            .with_a_submission_running_past_its_wait(300)],
            &memory_leaked,
        );
        let gpus_leaked_by_the_later_mount = later_mount.gpus_leaked_when_dropped();
        drop(later_mount);
        assert_eq!(
            gpus_leaked_by_the_later_mount.gpus_leaked(),
            Some(vec![GpuLeakedWhenDropped {
                device: card.clone(),
                bytes_held_on_the_device: 900,
                bytes_leaked_by_earlier_mounts: 600,
            }]),
            "后一次挂载丢掉那一刻占着 900，其中 600 是开卡时已算上的"
        );
        assert_eq!(
            memory_leaked.bytes_leaked_on(&card),
            900,
            "进程里只加这一次自己的 300"
        );
    }

    /// 按卡的顺序填满那几条用例里的单元：每个 100 字节。
    const BYTES_OF_A_UNIT_IN_THE_FILL: usize = 100;

    /// 校验和那一段一次交得下 `unit_count` 个 [`BYTES_OF_A_UNIT_IN_THE_FILL`] 字节的单元、多一个就交不下的显存上限（`CHECKSUM_CHUNK_FOOTPRINT` 算的）。
    fn memory_limit_holding_units_in_one_submission(unit_count: u64) -> u64 {
        let bytes_of_a_unit = u64::try_from(BYTES_OF_A_UNIT_IN_THE_FILL).expect("小数");
        CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(bytes_of_a_unit * unit_count, unit_count)
    }

    /// 本机 `card_count` 张替身卡的总线地址，按本机的次序：「0000:01:00.0」「0000:41:00.0」「0000:81:00.0」……
    fn addresses_of_the_cards(card_count: usize) -> Vec<String> {
        (0..card_count)
            .map(|card| format!("0000:{:02x}:00.0", 0x01 + 0x40 * card))
            .collect()
    }

    /// `card_count` 张替身卡全都参与、按本机的次序，每张给 `memory_limit_in_bytes` 的显存上限，交一批 `unit_count` 个 [`BYTES_OF_A_UNIT_IN_THE_FILL`] 字节的单元：
    /// 结果与 CPU 参照逐条相同（断言在里面）；交回这一段记下的与每张卡交了几次（按本机的次序）。
    fn checksum_batch_filling_cards_on_the_cpu(
        card_count: usize,
        memory_limit_in_bytes: u64,
        unit_count: usize,
    ) -> (GpuSubmissionsOfOneSegment, Vec<u32>) {
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            addresses_of_the_cards(card_count)
                .iter()
                .map(|address| {
                    CardComputedOnTheCpuForTests::of_the_address(
                        address,
                        Some(memory_limit_in_bytes),
                        0,
                    )
                })
                .collect(),
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let unit_bytes = unit_bytes_of(unit_count, BYTES_OF_A_UNIT_IN_THE_FILL);
        let units = units_to_verify(&unit_bytes);
        assert_eq!(
            background_gpu.both_kernel_outcomes_of_the_batch(&units),
            Ok(cpu_reference_outcomes_of(&units)),
            "按卡的顺序填满几张卡，结果照样与 CPU 参照逐条相同"
        );
        let submissions = background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification;
        let submissions_made_by_each_card = background_gpu
            .cards_on_this_host
            .iter()
            .map(|card| card.submissions_made)
            .collect();
        (submissions, submissions_made_by_each_card)
    }

    /// 用户 2026-11-02 定「按卡的顺序填满」（判决 m4-gpugc-code-r3 五之一）：三张卡各 1 GiB、一批 12 个单元一次交得下——只有排在前面的
    /// 「0000:01:00.0」分到活、交一次，后两张一次都不交（不要求每张卡都分到活）。
    #[test]
    fn the_batch_the_first_card_in_the_order_holds_goes_only_to_that_card() {
        let (submissions, submissions_made_by_each_card) =
            checksum_batch_filling_cards_on_the_cpu(3, 1 << 30, 12);
        assert_eq!(
            submissions,
            GpuSubmissionsOfOneSegment {
                submissions_completed: 1,
                pieces_submitted: 12,
                resubmissions_after_timeouts: 0,
                recuts_after_running_out_of_memory: 0,
                devices_given_work: vec![card_identifier("0000:01:00.0")],
            }
        );
        assert_eq!(submissions_made_by_each_card, vec![1, 0, 0]);
    }

    /// 同一条：三张卡的上限各一次只交得下 8 个单元，一批 9 个——「0000:01:00.0」填满 8 个，剩下的 1 个才给「0000:41:00.0」，「0000:81:00.0」不分。
    #[test]
    fn the_batch_overflowing_the_first_card_in_the_order_hands_only_the_rest_to_the_next_card() {
        let (submissions, submissions_made_by_each_card) = checksum_batch_filling_cards_on_the_cpu(
            3,
            memory_limit_holding_units_in_one_submission(8),
            9,
        );
        assert_eq!(
            submissions,
            GpuSubmissionsOfOneSegment {
                submissions_completed: 2,
                pieces_submitted: 9,
                resubmissions_after_timeouts: 0,
                recuts_after_running_out_of_memory: 0,
                devices_given_work: vec![
                    card_identifier("0000:01:00.0"),
                    card_identifier("0000:41:00.0")
                ],
            }
        );
        assert_eq!(submissions_made_by_each_card, vec![1, 1, 0]);
    }

    /// 同一条的饱和（用户 2026-11-02「你要测任务饱和的情况」）：三张卡的上限各一次只交得下 8 个单元，一批 30 个——前两张各填满 8 个、交一次，
    /// 排在最后的那张拿剩下的 14 个、交两次（8 个与 6 个）；每张都分到活、各自都交过，结果与 CPU 参照逐条相同。
    #[test]
    fn the_batch_saturating_every_card_fills_each_card_and_each_card_submits() {
        let (submissions, submissions_made_by_each_card) = checksum_batch_filling_cards_on_the_cpu(
            3,
            memory_limit_holding_units_in_one_submission(8),
            30,
        );
        assert_eq!(
            submissions,
            GpuSubmissionsOfOneSegment {
                submissions_completed: 4,
                pieces_submitted: 30,
                resubmissions_after_timeouts: 0,
                recuts_after_running_out_of_memory: 0,
                devices_given_work: addresses_of_the_cards(3)
                    .iter()
                    .map(|address| card_identifier(address.as_str()))
                    .collect(),
            }
        );
        assert_eq!(submissions_made_by_each_card, vec![1, 1, 2]);
    }

    /// 两张卡上一组设置：点名 `named`（`None` 是不点名）、上限 `memory_limits`（按点名的次序或本机的次序），在外壳上照它从下一批起跑。
    fn settings_admitted_on_the_cards(
        background_gpu: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
        named: Option<Vec<GpuDeviceIdentifier>>,
        memory_limits: Vec<u64>,
    ) -> AdmittedGpuSettings {
        admit_gpu_settings_against_the_inventory(
            &BackgroundComputeSettings {
                background_compute: BackgroundComputeBackend::Gpu,
                background_gpu_devices: named,
                background_gpu_memory_limit: Some(
                    memory_limits.into_iter().map(GpuMemoryInBytes).collect(),
                ),
                background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            },
            background_gpu,
        )
        .expect("用例给的一组照两张替身卡校验都合法")
    }

    /// 一批 4 个单元交给外壳，交回这一段记下的分到活的卡（结果与 CPU 参照逐条相同）。
    fn devices_given_work_by_a_batch_of_four_units(
        background_gpu: &mut BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
    ) -> Vec<GpuDeviceIdentifier> {
        let unit_bytes = unit_bytes_of(4, BYTES_OF_A_UNIT_IN_THE_FILL);
        let units = units_to_verify(&unit_bytes);
        assert_eq!(
            background_gpu.both_kernel_outcomes_of_the_batch(&units),
            Ok(cpu_reference_outcomes_of(&units))
        );
        background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification
            .devices_given_work
    }

    /// 同一条「点名了的按 `background_gpu_devices` 的次序」：切换点名「0000:41:00.0」在前、「0000:01:00.0」在后——活少时只有点名在前的那张分到活。
    #[test]
    fn the_batch_fills_the_cards_in_the_order_they_are_named() {
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            addresses_of_the_cards(2)
                .iter()
                .map(|address| CardComputedOnTheCpuForTests::of_the_address(address, None, 0))
                .collect(),
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let named_in_reverse = settings_admitted_on_the_cards(
            &background_gpu,
            Some(vec![
                card_identifier("0000:41:00.0"),
                card_identifier("0000:01:00.0"),
            ]),
            vec![1 << 30, 1 << 30],
        );
        background_gpu.use_the_settings_from_the_next_batch(&named_in_reverse);
        assert_eq!(
            devices_given_work_by_a_batch_of_four_units(&mut background_gpu),
            vec![card_identifier("0000:41:00.0")],
            "点名在前的那张全拿"
        );
    }

    /// 判决 m4-gpugc-code-r3 第三节第 5 条（照「按卡的顺序填满」改写的验收）：一组设置给「0000:01:00.0」的上限连最小的一片都装不下，它被晾下、
    /// 活全给「0000:41:00.0」；下一组给它装得下的上限，它请回来——照新上限再量一次代价（另一张不重量），从下一批起照参与的次序排在前面、重新分到活。
    #[test]
    fn the_card_left_out_under_its_memory_limit_is_measured_again_and_takes_part_in_order_once_brought_back(
    ) {
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            addresses_of_the_cards(2)
                .iter()
                .map(|address| CardComputedOnTheCpuForTests::of_the_address(address, None, 0))
                .collect(),
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
        let leaving_out_the_card_in_front = settings_admitted_on_the_cards(
            &background_gpu,
            None,
            vec![smallest_piece_bytes - 1, 1 << 30],
        );
        background_gpu.use_the_settings_from_the_next_batch(&leaving_out_the_card_in_front);
        assert_eq!(
            devices_given_work_by_a_batch_of_four_units(&mut background_gpu),
            vec![card_identifier("0000:41:00.0")],
            "晾下的那张不分活"
        );
        let bringing_it_back =
            settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
        background_gpu.use_the_settings_from_the_next_batch(&bringing_it_back);
        assert_eq!(
            background_gpu
                .cards_on_this_host
                .iter()
                .map(|card| card.cost_measurements)
                .collect::<Vec<_>>(),
            vec![1, 0],
            "请回来的那张照新上限再量一次，另一张不重量"
        );
        assert_eq!(
            devices_given_work_by_a_batch_of_four_units(&mut background_gpu),
            vec![card_identifier("0000:01:00.0")],
            "请回来的那张照参与的次序排在前面，活少时它全拿"
        );
    }

    /// 判决 m4-gpugc-code-r3 第三节第 3 条（第 4 条「建缓冲显存不足、切小重切时，这一段在这张卡上已做完的片不留，照新的切法整段从头重交」）：
    /// 一张卡上限 1500 字节、卡上只空闲 500 字节，三个单元 100、600、600 字节一次交得下（1448 字节）而建不成——压到 724 字节重切成三段（160、660、660），
    /// 头一段交成（这一次的结果是坏的：替身把它的判定翻了），第二段又建不成——压到 330 字节，照新切法整段从头重交七段（600 字节的单元切成 268、268、64），
    /// 头一段交成的那一次的结果不用：判定与 CPU 参照逐条相同；这张卡上交成 8 次（1 + 7），这一段记下交成 8 次、8 片、重切 2 次。
    #[test]
    fn the_recut_after_running_out_of_memory_resubmits_the_whole_share_without_keeping_the_pieces_already_done(
    ) {
        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", Some(1500), 0);
        card.free_bytes = Some(500);
        card.checksum_submission_with_flipped_verdicts = Some(1);
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let unit_bytes = vec![vec![3_u8; 100], vec![5_u8; 600], vec![7_u8; 600]];
        let units = units_to_verify(&unit_bytes);
        assert_eq!(
            background_gpu.both_kernel_outcomes_of_the_batch(&units),
            Ok(cpu_reference_outcomes_of(&units)),
            "已交成的那一片的坏结果不用：整段照新切法重交，判定与 CPU 参照逐条相同"
        );
        assert_eq!(
            background_gpu.cards_on_this_host[0].submissions_made, 8,
            "交成 8 次：头一段那一次，加整段重交的 7 段"
        );
        assert_eq!(
            background_gpu
                .take_the_submissions_of_the_batch()
                .unit_checksum_verification,
            GpuSubmissionsOfOneSegment {
                submissions_completed: 8,
                pieces_submitted: 8,
                resubmissions_after_timeouts: 0,
                recuts_after_running_out_of_memory: 2,
                devices_given_work: vec![card_identifier("0000:01:00.0")],
            }
        );
    }

    /// 判决 m4-gpugc-code-r3 第三节第 6 条在外壳上：一个单元头一次交等超过上限，之后重交之前那一等连着 3 回等不到、一次都没重交——
    /// 第 4 次超时报这张卡用不了，报警带的重交次数是真交出去的 0 次（不是常数 3），原因是最后那一等没等到。
    #[test]
    fn the_card_unusable_without_any_resubmission_reaching_it_reports_no_resubmission() {
        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        card.submissions_timing_out = VecDeque::from([true]);
        card.earlier_submission_waits_timing_out = VecDeque::from([true, true, true]);
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let unit_bytes = unit_bytes_of(1, 1000);
        assert_eq!(
            background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes)),
            Err(ChecksumBatchFailed::GpuFailed {
                device: card_identifier("0000:01:00.0"),
                cause: GpuFailureCause::UnusableAfterResubmissions {
                    resubmissions: 0,
                    last_cause: Box::new(
                        GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
                            wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                        }
                    ),
                },
            })
        );
        assert_eq!(
            background_gpu.cards_on_this_host[0].submissions_made, 1,
            "只交了头一次"
        );
    }

    /// 在内存里建的一份缓冲：建的时候要了多少字节，建没建成（显存不足就没建成）。
    #[derive(Debug, PartialEq, Eq)]
    struct BufferInMemoryForTests {
        bytes_requested: u64,
        is_created: bool,
    }

    /// 建一段缓冲的设备替身：卡上此刻空闲 `free_bytes`，每建一份照它扣，不够就建不成、记一条显存不足。记法照 wgpu 的错误范围：
    /// 有范围开着时那条错归最里层那一个范围（每个范围只留它接住的第一条），一个范围都没开着时记成没接住的错（真卡上那条错走设备的没接住错误回调、
    /// 这一批报 `UncapturedErrorOrDeviceLost`）。另记卡上被本包占着的最多那一刻的字节数（`peak_bytes_held`）。
    struct DeviceOfAChunkInMemoryForTests {
        free_bytes: Cell<u64>,
        bytes_held: Cell<u64>,
        peak_bytes_held: Cell<u64>,
        out_of_memory_captured_by_each_open_scope: std::cell::RefCell<Vec<Option<String>>>,
        uncaptured_out_of_memory: std::cell::RefCell<Vec<String>>,
    }

    /// 替身开的一个范围：开时范围栈里的位置。
    struct OutOfMemoryScopeInMemoryForTests {
        position_in_the_open_scopes: usize,
    }

    impl DeviceOfAChunkInMemoryForTests {
        fn with_free_bytes(free_bytes: u64) -> Self {
            Self {
                free_bytes: Cell::new(free_bytes),
                bytes_held: Cell::new(0),
                peak_bytes_held: Cell::new(0),
                out_of_memory_captured_by_each_open_scope: std::cell::RefCell::new(Vec::new()),
                uncaptured_out_of_memory: std::cell::RefCell::new(Vec::new()),
            }
        }

        fn buffer_of(&self, bytes: u64) -> BufferInMemoryForTests {
            let free_bytes = self.free_bytes.get();
            if bytes <= free_bytes {
                self.free_bytes.set(free_bytes - bytes);
                self.bytes_held.set(self.bytes_held.get() + bytes);
                self.peak_bytes_held
                    .set(self.peak_bytes_held.get().max(self.bytes_held.get()));
                return BufferInMemoryForTests {
                    bytes_requested: bytes,
                    is_created: true,
                };
            }
            let reason = format!("建 {bytes} 字节的缓冲：卡上只剩 {free_bytes} 字节");
            let mut open_scopes = self.out_of_memory_captured_by_each_open_scope.borrow_mut();
            match open_scopes.last_mut() {
                Some(innermost_scope) => {
                    if innermost_scope.is_none() {
                        *innermost_scope = Some(reason);
                    }
                }
                None => self.uncaptured_out_of_memory.borrow_mut().push(reason),
            }
            BufferInMemoryForTests {
                bytes_requested: bytes,
                is_created: false,
            }
        }
    }

    impl DeviceCreatingTheBuffersOfAChunk for DeviceOfAChunkInMemoryForTests {
        type Buffer = BufferInMemoryForTests;
        type OutOfMemoryScope = OutOfMemoryScopeInMemoryForTests;

        fn open_the_out_of_memory_scope(&self) -> OutOfMemoryScopeInMemoryForTests {
            let mut open_scopes = self.out_of_memory_captured_by_each_open_scope.borrow_mut();
            open_scopes.push(None);
            OutOfMemoryScopeInMemoryForTests {
                position_in_the_open_scopes: open_scopes.len() - 1,
            }
        }

        fn readback_of_one_kernel(&self, outcome_bytes: u64) -> BufferInMemoryForTests {
            self.buffer_of(outcome_bytes)
        }

        fn bytes_of_the_buffer(&self, buffer: &BufferInMemoryForTests) -> u64 {
            buffer.bytes_requested
        }

        fn out_of_memory_captured_by(
            &self,
            scope: OutOfMemoryScopeInMemoryForTests,
            _wait_limit: Duration,
        ) -> Result<Option<String>, GpuFailureCause> {
            let mut open_scopes = self.out_of_memory_captured_by_each_open_scope.borrow_mut();
            assert_eq!(
                scope.position_in_the_open_scopes + 1,
                open_scopes.len(),
                "关的是最里层那个范围"
            );
            Ok(open_scopes.pop().expect("上面判过：关的那个范围还开着"))
        }
    }

    /// 这一段自己那几样缓冲合起来的字节数与一份核的结果字节数（两份回读暂存各这么大）：只用来造「那几样建得成、回读暂存建不成」。
    const BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK: u64 = 1000;
    const OUTCOME_BYTES_OF_THE_CHUNK: u64 = 64;

    /// 判决 m4-gpugc-code-r2 第三节第 5 条、四之一（用户 2026-11-01 定「造不出就换方法继续造」）：卡上空闲的装得下这一段自己那几样缓冲，
    /// 装不下两份回读暂存——回读暂存建在同一个只接显存不足的范围里，那一条显存不足被范围接住：交回 `OutOfMemoryCreatingTheBuffers`
    /// （外壳照它放掉这一段的账、走「这一批转给别的卡」），一条没接住的错都没有。回读暂存建在范围之外时，那一条显存不足没被接住、这一段照「建成了」交回。
    #[test]
    fn readbacks_short_of_free_memory_are_caught_by_the_out_of_memory_scope_of_the_chunk() {
        let device = DeviceOfAChunkInMemoryForTests::with_free_bytes(
            BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK + OUTCOME_BYTES_OF_THE_CHUNK - 1,
        );
        let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
            &device,
            Duration::from_secs(1),
            BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK + 2 * OUTCOME_BYTES_OF_THE_CHUNK,
            OUTCOME_BYTES_OF_THE_CHUNK,
            || [device.buffer_of(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK)],
        );
        assert!(
            matches!(
                created,
                Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
                    bytes_held_by_the_chunk,
                    ..
                }) if bytes_held_by_the_chunk
                    == BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK + 2 * OUTCOME_BYTES_OF_THE_CHUNK
            ),
            "回读暂存建不成的那一条显存不足被这一段的范围接住、报成建缓冲显存不足（带这一段的账）：{:?}",
            created.as_ref().map(|_buffers| "建成了")
        );
        assert_eq!(
            *device.uncaptured_out_of_memory.borrow(),
            Vec::<String>::new(),
            "没有一条显存不足落到范围之外"
        );
    }

    /// 同一条的阳性对照：空闲的装得下这一段自己那几样与两份回读暂存——交回建成的那几样与两份各 64 字节的回读暂存，没有错、范围都关上了。
    #[test]
    fn the_chunk_and_both_readbacks_within_the_free_memory_are_created_in_one_out_of_memory_scope()
    {
        let device = DeviceOfAChunkInMemoryForTests::with_free_bytes(
            BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK + 2 * OUTCOME_BYTES_OF_THE_CHUNK,
        );
        let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
            &device,
            Duration::from_secs(1),
            BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK + 2 * OUTCOME_BYTES_OF_THE_CHUNK,
            OUTCOME_BYTES_OF_THE_CHUNK,
            || [device.buffer_of(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK)],
        )
        .expect("空闲的正好装得下：建得成");
        assert_eq!(
            (created.buffers, created.readbacks),
            (
                [BufferInMemoryForTests {
                    bytes_requested: BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK,
                    is_created: true,
                }],
                [
                    BufferInMemoryForTests {
                        bytes_requested: OUTCOME_BYTES_OF_THE_CHUNK,
                        is_created: true,
                    },
                    BufferInMemoryForTests {
                        bytes_requested: OUTCOME_BYTES_OF_THE_CHUNK,
                        is_created: true,
                    },
                ]
            )
        );
        assert_eq!(device.free_bytes.get(), 0, "三样都照字节数扣了");
        assert!(
            device.uncaptured_out_of_memory.borrow().is_empty()
                && device
                    .out_of_memory_captured_by_each_open_scope
                    .borrow()
                    .is_empty(),
            "没有错、范围关上了"
        );
    }

    /// 判决 m4-gpugc-code-r3 第三节第 4 条（七·3 验收「塞一个多分配一块缓冲的故障要红」）：这张卡的上限与卡上空闲的都是 1500 字节，这一段记在账上
    /// 1128 字节（这一段自己那几样 1000、两份回读暂存各 64），建的时候多建了一块 300 字节的缓冲——卡上真占的到了 1428 字节，比账多：
    /// 账核真建的缓冲字节，当场断言，不带着比账多的显存往下交。
    #[test]
    #[should_panic(expected = "这一段真建的缓冲合起来的字节数与记在这张卡账上的不等")]
    fn the_chunk_creating_one_buffer_more_than_it_holds_on_the_account_panics() {
        const BYTES_OF_THE_EXTRA_BUFFER: u64 = 300;
        let memory_limit_in_bytes = 1500;
        let device = DeviceOfAChunkInMemoryForTests::with_free_bytes(memory_limit_in_bytes);
        let bytes_held_by_the_chunk =
            BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK + 2 * OUTCOME_BYTES_OF_THE_CHUNK;
        let mut account = GpuMemoryAccount::<u32>::with_nothing_held(Some(GpuMemoryInBytes(
            memory_limit_in_bytes,
        )));
        account.hold(bytes_held_by_the_chunk);
        let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
            &device,
            Duration::from_secs(1),
            bytes_held_by_the_chunk,
            OUTCOME_BYTES_OF_THE_CHUNK,
            || {
                [
                    device.buffer_of(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK),
                    device.buffer_of(BYTES_OF_THE_EXTRA_BUFFER),
                ]
            },
        );
        // 走到这里就是账没核真建的缓冲：卡上真占的峰值比账上的多，用例照「峰值不超账」红在这里。
        assert!(
            device.peak_bytes_held.get() <= account.bytes_held(),
            "卡上真占的峰值 {} 字节超过了账上的 {} 字节（{:?}）",
            device.peak_bytes_held.get(),
            account.bytes_held(),
            created.as_ref().map(|_buffers| "建成了")
        );
    }

    /// 地址空间压住之后这一段自己的那份缓冲要的字节（攻方探针 `noop_chunk_buffer_with_contents_short` 取的 128 MiB）：比压住之后留的那一截
    /// [`ROOM_LEFT_IN_THE_CAPPED_ADDRESS_SPACE`] 大，noop 后端在主机内存里 `try_reserve_exact` 这么多就失败、wgpu 报显存不足；
    /// 又在 `wgpu::Limits::default()` 的单个缓冲上限（256 MiB）之内，wgpu 不先报校验错。
    const BYTES_OF_THE_BUFFER_BEYOND_THE_CAPPED_ADDRESS_SPACE: u64 = 128 << 20;
    /// 压地址空间上限时比此刻的 VmSize 多留的那一截（攻方探针取的 32 MiB）：两份 64 字节的回读暂存、等错误范围、打印与 libtest 收尾用得下。
    const ROOM_LEFT_IN_THE_CAPPED_ADDRESS_SPACE: u64 = 32 << 20;
    /// 标记「这一次是地址空间压住的那个子进程」的环境变量：外面那一次起子进程时设上。
    const VARIABLE_MARKING_THE_RUN_WITH_THE_ADDRESS_SPACE_CAPPED: &str =
        "SINGLEFS_BACKGROUND_GPU_TEST_WITH_THE_ADDRESS_SPACE_CAPPED";
    /// 地址空间压住的那个子进程判过之后打的那一行的开头：外面那一次认它，认不到判红。
    const LINE_OF_THE_RUN_WITH_THE_ADDRESS_SPACE_CAPPED: &str =
        "CHUNK_SHORT_OF_MEMORY_JUDGED_WITH_THE_ADDRESS_SPACE_CAPPED";

    /// 压地址空间时设的上限：软上限取「想压到的字节数」与原来的硬上限里小的那个，硬上限照原样。软上限比硬上限大时 setrlimit 报 EINVAL，
    /// 所以硬上限比想压到的还小时，软上限就压到硬上限。
    fn address_space_limit_capped_at(
        wanted_soft_limit_in_bytes: u64,
        limit_before: &libc::rlimit,
    ) -> libc::rlimit {
        libc::rlimit {
            rlim_cur: wanted_soft_limit_in_bytes.min(limit_before.rlim_max),
            rlim_max: limit_before.rlim_max,
        }
    }

    /// 硬上限比想压到的小：软上限等于硬上限；硬上限是无限：软上限就是想压到的那个数。两种情形硬上限都照原样。
    #[test]
    fn the_capped_soft_limit_of_the_address_space_does_not_exceed_the_hard_limit() {
        let four_gibibytes: u64 = 4 << 30;
        let one_gibibyte: u64 = 1 << 30;
        let limit_with_a_small_hard_limit = libc::rlimit {
            rlim_cur: one_gibibyte,
            rlim_max: one_gibibyte,
        };
        let capped_under_the_small_hard_limit =
            address_space_limit_capped_at(four_gibibytes, &limit_with_a_small_hard_limit);
        assert_eq!(
            capped_under_the_small_hard_limit.rlim_cur, one_gibibyte,
            "硬上限 1 GiB 比想压到的 4 GiB 小，软上限压到硬上限"
        );
        assert_eq!(capped_under_the_small_hard_limit.rlim_max, one_gibibyte);
        let limit_without_a_hard_limit = libc::rlimit {
            rlim_cur: libc::RLIM_INFINITY,
            rlim_max: libc::RLIM_INFINITY,
        };
        let capped_under_no_hard_limit =
            address_space_limit_capped_at(four_gibibytes, &limit_without_a_hard_limit);
        assert_eq!(
            capped_under_no_hard_limit.rlim_cur, four_gibibytes,
            "硬上限无限，软上限就是想压到的那个数"
        );
        assert_eq!(capped_under_no_hard_limit.rlim_max, libc::RLIM_INFINITY);
    }

    /// 本进程的地址空间上限（`RLIMIT_AS`）压到「此刻的 VmSize 加 `room_beyond_the_present_size` 字节」跑 `run`，跑完放回原来的上限，交回 `run` 的结局。
    /// 压的是整个进程：只在地址空间压住的那个子进程里调（那个进程只跑一条用例），不在跑别的用例的进程里调。
    ///
    /// # Panics
    /// 读不到 `/proc/self/status` 的 VmSize 一行，或 `getrlimit` / `setrlimit` 报错：Linux 上这几样总在。
    #[allow(
        unsafe_code,
        reason = "std 没有 getrlimit/setrlimit，只能调 libc；只在用例自己起的子进程里压本进程的地址空间"
    )]
    fn with_the_address_space_capped_beyond_the_present_size<Outcome>(
        room_beyond_the_present_size: u64,
        run: impl FnOnce() -> Outcome,
    ) -> Outcome {
        let status_of_this_process =
            std::fs::read_to_string("/proc/self/status").expect("Linux 上读得到 /proc/self/status");
        let present_size_in_kibibytes: u64 = status_of_this_process
            .lines()
            .find_map(|line| line.strip_prefix("VmSize:"))
            .and_then(|rest_of_the_line| rest_of_the_line.split_whitespace().next())
            .and_then(|kibibytes| kibibytes.parse().ok())
            .expect("/proc/self/status 有 VmSize 一行，单位 kB");
        let mut limit_before = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: `limit_before` 是这个栈帧里初始化过的 `rlimit`，getrlimit 只往它里面写，调用期间没有别的引用。
        let read_status = unsafe { libc::getrlimit(libc::RLIMIT_AS, &raw mut limit_before) };
        assert_eq!(read_status, 0, "getrlimit(RLIMIT_AS) 在 Linux 上总读得到");
        let capped_limit = address_space_limit_capped_at(
            present_size_in_kibibytes * 1024 + room_beyond_the_present_size,
            &limit_before,
        );
        // SAFETY: `capped_limit` 是这个栈帧里初始化过的 `rlimit`，setrlimit 只读它；软上限不超过原来的硬上限（只压软的那一个）。
        let cap_status = unsafe { libc::setrlimit(libc::RLIMIT_AS, &raw const capped_limit) };
        assert_eq!(cap_status, 0, "setrlimit(RLIMIT_AS) 压软上限");
        let outcome = run();
        // SAFETY: `limit_before` 是上面 getrlimit 读回的那一份，setrlimit 只读它。
        let restore_status = unsafe { libc::setrlimit(libc::RLIMIT_AS, &raw const limit_before) };
        assert_eq!(restore_status, 0, "setrlimit(RLIMIT_AS) 放回原来的上限");
        outcome
    }

    /// 地址空间压住的那个子进程里判的那一件：在 wgpu 30.0.1 的 noop 后端上走产品那一份 `impl DeviceCreatingTheBuffersOfAChunk for wgpu::Device`，
    /// 地址空间压到此刻多 [`ROOM_LEFT_IN_THE_CAPPED_ADDRESS_SPACE`]，这一段自己的缓冲照产品的建法（建的时候映射着、映射得上才写）要
    /// [`BYTES_OF_THE_BUFFER_BEYOND_THE_CAPPED_ADDRESS_SPACE`] 字节——建不成，交回 `OutOfMemoryCreatingTheBuffers`（带这一段的账）、不 panic，
    /// 没接住的错一条都没有、写内容的那一步没走到。
    fn the_buffer_of_the_chunk_beyond_the_capped_address_space_is_reported_as_out_of_memory() {
        let (device, _queue) = wgpu::Device::noop(&wgpu::DeviceDescriptor {
            label: Some("singlefs-background-gpu-noop-test"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        });
        let uncaptured_errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let uncaptured_errors_of_the_device = Arc::clone(&uncaptured_errors);
        device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
            uncaptured_errors_of_the_device
                .lock()
                .expect("用例没在别处 panic 过")
                .push(format!("{error}"));
        }));
        let contents_written = Cell::new(false);
        let bytes_held_by_the_chunk_on_the_account =
            BYTES_OF_THE_BUFFER_BEYOND_THE_CAPPED_ADDRESS_SPACE + 2 * OUTCOME_BYTES_OF_THE_CHUNK;
        let created = with_the_address_space_capped_beyond_the_present_size(
            ROOM_LEFT_IN_THE_CAPPED_ADDRESS_SPACE,
            || {
                buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
                    &device,
                    Duration::from_secs(10),
                    bytes_held_by_the_chunk_on_the_account,
                    OUTCOME_BYTES_OF_THE_CHUNK,
                    || {
                        [buffer_mapped_at_creation_within_the_scope_of_the_caller(
                            &device,
                            BYTES_OF_THE_BUFFER_BEYOND_THE_CAPPED_ADDRESS_SPACE,
                            wgpu::BufferUsages::STORAGE,
                            |_mapped| contents_written.set(true),
                        )]
                    },
                )
            },
        );
        assert!(
            matches!(
                created,
                Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
                    bytes_held_by_the_chunk,
                    ..
                }) if bytes_held_by_the_chunk == bytes_held_by_the_chunk_on_the_account
            ),
            "建不成的那一份被这一段的范围接住、报成建缓冲显存不足：{:?}",
            created.as_ref().map(|_buffers| "建成了")
        );
        assert!(!contents_written.get(), "映射不上：不写内容");
        assert_eq!(
            *uncaptured_errors.lock().expect("用例没在别处 panic 过"),
            Vec::<String>::new(),
            "没有一条错落到范围之外（不解开没建成的缓冲的映射）"
        );
    }

    /// 判决 m4-gpugc-code-r3 第三节第 2 条（攻方探针 `noop_chunk_buffer_with_contents_short` 的造法）：noop 后端在主机内存里建缓冲，要真建不成就得压
    /// 进程的地址空间上限；压的是整个进程，所以本进程照原样再起一次自己、只跑这一条，那一次里压上限再建
    /// （[`the_buffer_of_the_chunk_beyond_the_capped_address_space_is_reported_as_out_of_memory`]），跑别的用例的这个进程不压。
    /// 外面那一次认那一次退 0、打了判过的那一行。起的那一次放进它自己的进程组，它跑完才交回。
    /// 改回 `create_buffer_init` 那种建完就对映射区 `expect` 的写法，那一次里 panic、退非 0，这里红。
    #[test]
    fn the_buffer_of_the_chunk_short_of_memory_on_a_noop_device_is_reported_as_out_of_memory_without_panicking(
    ) {
        use std::os::unix::process::CommandExt;
        let test_name = "background_gpu::tests::the_buffer_of_the_chunk_short_of_memory_on_a_noop_device_is_reported_as_out_of_memory_without_panicking";
        if std::env::var_os(VARIABLE_MARKING_THE_RUN_WITH_THE_ADDRESS_SPACE_CAPPED).is_some() {
            the_buffer_of_the_chunk_beyond_the_capped_address_space_is_reported_as_out_of_memory();
            println!("{LINE_OF_THE_RUN_WITH_THE_ADDRESS_SPACE_CAPPED} test={test_name}");
            return;
        }
        let run_with_the_address_space_capped =
            std::process::Command::new(std::env::current_exe().expect("这个测试程序自己的路径"))
                .args([test_name, "--exact", "--nocapture", "--test-threads", "1"])
                .env(VARIABLE_MARKING_THE_RUN_WITH_THE_ADDRESS_SPACE_CAPPED, "1")
                .process_group(0)
                .output()
                .expect("起得来地址空间压住的那一次");
        let standard_output = String::from_utf8_lossy(&run_with_the_address_space_capped.stdout);
        assert!(
            run_with_the_address_space_capped.status.success()
                && standard_output.contains(&format!(
                    "{LINE_OF_THE_RUN_WITH_THE_ADDRESS_SPACE_CAPPED} test={test_name}"
                )),
            "地址空间压住的那一次退 0、判过：{:?}\n{standard_output}\n{}",
            run_with_the_address_space_capped.status,
            String::from_utf8_lossy(&run_with_the_address_space_capped.stderr)
        );
    }

    /// 同一条的阳性对照：noop 后端上照产品的建法建一份装着内容的缓冲，建得成、内容写进去了，没有错。
    #[test]
    fn the_buffer_with_contents_on_a_noop_device_is_created_and_written_within_the_scope() {
        let (device, _queue) = wgpu::Device::noop(&wgpu::DeviceDescriptor {
            label: Some("singlefs-background-gpu-noop-test"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        });
        let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
            &device,
            Duration::from_secs(10),
            8 + 2 * OUTCOME_BYTES_OF_THE_CHUNK,
            OUTCOME_BYTES_OF_THE_CHUNK,
            || {
                [buffer_with_contents_created_within_the_scope_of_the_caller(
                    &device,
                    &[1, 2, 3, 4, 5],
                    wgpu::BufferUsages::STORAGE,
                )]
            },
        )
        .expect("noop 后端上 8 字节建得成");
        assert_eq!(
            created.buffers[0].size(),
            8,
            "5 个字节的内容按整字补成 8 字节，与切批记账同一种算法"
        );
    }
}
