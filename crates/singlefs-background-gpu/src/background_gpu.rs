//! 起 GPU 的外壳（D26（后台整理与放置回收） 已定项 13 第 1–4、8 条）：挂载那一刻枚举 wgpu 适配器、判过校验、只在参与的卡上起设备与各段两份核的管线、
//! 量一次提交的主机 CPU 时间（第 2 条「挂载时只开参与的卡」：给了 `background_gpu_devices` 是点名的卡，没给是本机全部独立显卡）；
//! 参与的卡开不成就隔 1 分钟重开、至多重开 3 次，都不成才拒绝挂载（第 4 条；判法在 `crate::gpus_of_this_host::gpus_opened_at_the_mount`）。给了 `background_gpu_devices` 时只判点名的卡：没点名的独显缺总线地址、缺设备本地堆、与别的独显报了同一个地址都不拒，
//! 不收进卡清单、报出来（[`BackgroundGpuOverTheCards::unnamed_discrete_gpus_left_off_the_list`]，判法 `crate::adapter_inventory::discrete_gpus_of_the_enumeration_under_the_named`）。
//! 参与的卡由设置定（[`BackgroundGpuOverTheCards::participating_gpus`] 报出来），挂着切换时核心层交来新的一组
//! （[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]，两批之间），从下一批起只在它点的卡上、照它给的显存上限与等的上限跑：
//! 开着的卡不重开；它点到没开的卡，照第 4 条「切换请求照收，在会话锁外开新卡」在另一条线程上开（开卡同挂载那一刻，[`OpeningOfAGpuAtASwitch`]；
//! 开不成同样隔 1 分钟重开、至多重开 3 次；起不了开卡线程、开卡里 panic 都算那一次没开成），开着的这段时间照旧照交来之前那一组跑，开成了在之后的两批之间换上
//! （[`BackgroundGpuOverTheCards::put_the_gpus_opened_since_the_switch_in_effect`]，答最近交来的那一组还在开卡、换上了、开不成三种）；
//! 开卡的结局按交设置的次序认，为更早一次交来的开的、开不成的不报，照最近那一组重开；重开到上限都没开成就报出那张卡与每一次的原因，
//! 此刻生效那一组之外开着的卡全部关掉，照旧照交来之前那一组跑（核心层生效的也是它，当场停 GC、报警），不自动再试，直到下一次换设置。
//! 一张卡在一次挂载里连着开了 4 次都没开成（跨切换接着数，中间开成一次就清零），之后再有切换点到它不再开，当场答开不成（[`GpuFailureCause::OpeningsInThisMountUsedUp`]），卸载重挂清零。
//! 同一张卡上前一次开卡到点之后还没回来就不再起下一次（算那一次又没开成，不记进连着开不成的次数；这一张表是进程级的，各次挂载与切换共用，[`OpeningsNotReturnedInThisProcess`]），
//! 前一次没到点、还在开的，后到的一方在自己的期限之内排队等它开完再开；
//! 卸载与切到 `cpu` 时叫停切换开卡线程。
//! 换上一组时不再点名的卡关掉（第 2 条「切换之后不再参与的卡，切换生效时关掉」），切到 `cpu` 时开着的卡全部关掉，有提交还在跑的那张同卸载整张泄漏、
//! 累计进对外读得到的一份（[`BackgroundGpuOverTheCards::gpus_leaked_when_closed_at_a_switch`]）；之后再点它就重开（分活、切段、等的纯逻辑在 `crate::gpus_of_this_host`）。
//! 挂着的时候每一批按卡的顺序填满参与的卡（前面的卡填到它照显存上限一次放得下的量，装不下的才给后面的卡；挂载时量的代价不用来分活）、按显存上限切段、两份核各算一次、逐条比对。三段都接上了：扫描里的校验和核对（[`BackgroundGpuOverTheCards::verify_unit_checksums`]）、
//! 区域扫描里的单元头解析与映射反查（[`BackgroundGpuOverTheCards::classify_region_records`]）与选段打分的每段活槽数（[`BackgroundGpuOverTheCards::count_live_slots_per_cluster_segment`]）。
//! 本机的卡清单（核心层 `GpuInventory`）答的是此刻本机在的独显与各自的显存总量（按总线地址排，开着的与没参与而没开的都列；每次答之前现枚举一次，
//! 挂载之后掉了的卡不列，切换点它照第 4 条「GPU 不在」拒）：挂载与切换照它校验设置，切换可以点挂载时没开的卡；
//! 点名不在本机的卡就是点名的卡不存在。
//!
//! 外壳分两层：一张开着的卡要做的几样（[`CardOfTheBackgroundGpu`]：交活之前先等前一次超时的那次提交、重交之前等、交一段两份核）由 [`GpuInUse`]
//! （wgpu 设备与六条管线）实现；几张卡上怎么分活、切段、重交、超时按批按卡计数、建缓冲显存不足时重切、丢的时候泄漏谁，在 [`BackgroundGpuOverTheCards`]
//! 里对任何一种卡都一样，测试换上在 CPU 上算的替身卡就钉得住，不起 GPU。产品那一份是 [`BackgroundGpu`]。
//!
//! GPU 出错（设备丢了、wgpu 报了没接住的错、等不成、回读映射不成、核没写到某一条）时这一批交回出错、不换 CPU（第 4 条）；一次提交等超过上限的，
//! 在同一张卡上重交，这一批在这张卡上（一批切成几片、几段时合起来算，交活之前等前一次超时的那次提交那一等也算）第 4 次等超过上限才交回「这张卡用不了」
//! （`crate::waiting_for_the_gpu::TimeoutsOfTheBatchOnOneGpu`；重交的单位照旧只是超时的那一片）。
//! 出错时不动连续不确定的计数（这一批放弃了，不算一轮）。只有一张卡时两份核都在这张卡上跑；几张卡时每张卡在自己分到的那几条上跑两份核。
//! 每一处阻塞等 GPU 都在此刻设置给的上限之内等（开包时是 `BackgroundGpuSelection::background_gpu_wait_limit`，等法在 `crate::waiting_for_the_gpu`）：
//! 等 wgpu 的 future 用 std 写的最小等法（[`wait_for_the_wgpu_future`]），等提交用 wgpu 自己带限时的 `poll`，不依赖 `pollster`
//! （门禁 checker-implementation-disjoint 格第 ⑧ 条）。枚举适配器不在上限之内：wgpu 在造 future 那一步同步做完枚举（[`VulkanInstanceOfThisProcess::vulkan_adapters_enumerated_within`]）。
//! 每一段切批之前先扣掉这张卡上此刻占着的显存（`ChunkBoundsOfAGpu::with_the_room_left_after`），几段合在一张卡上占的不超过它的上限（第 3 条）。
//! 一段的提交等超过上限之后，它的缓冲在卡上可能还占着：这一段的账留到那次提交真在 GPU 上做完才放，下一批在这张卡上切段交活之前先在上限之内
//! 等它做完（`crate::memory_limit_and_chunks` 文件头）。
//! 装不下就切小：单元切成几片（校验和那一段）、映射按落点切成几片（区域扫描那一段）、记录表按记录切成几片（选段打分那一段），逐片交；
//! 一张卡这一批连最小的一片都切不出（显存上限扣掉已占的）就这一批不分给它、活分给别的卡，参与的卡全都切不出才报
//! （`crate::gpus_of_this_host::plans_of_a_batch_filling_the_gpus_in_order`）。
//! 卡上建缓冲时显存不足（空闲的比上限少，第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」）：这张卡这一批的地方压到建不成的那一段的一半、
//! 重切、从头再交它分到的那几条，不报出错；一路压到连最小的一片都建不成才报（条款没写，第一版不支持）。
//! 挂载那一刻量代价那一段也按开卡时的上限切片，代价取各片之和。
//! 丢后台 GPU 时不等还在 GPU 上跑的提交：那张卡的设备与队列泄漏，别的卡照常丢；泄漏了哪几张、各占着多少报给调用方
//! （[`BackgroundGpuOverTheCards::gpus_leaked_when_dropped`]），并记进进程里的那一份（`crate::gpus_of_this_host::GpuMemoryLeakedInThisProcess`），
//! 新一次挂载这张卡的可用余量先扣掉它。

use std::future::Future;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, TryRecvError};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use singlefs_core::allocator::AllocationRecord;
use singlefs_core::background_compute::{
    AdmittedGpuSettings, BackgroundComputeSettingsRefused, ClusterSegmentEntryKey,
    GpuComputedSegment, GpuDeviceIdentifier, GpuInventory, GpuMemoryInBytes,
    GpuShortOfFreeMemoryForTheSmallestPiece, GpuSubmissionsOfOneSegment, GpuSubmissionsOfTheBatch,
    PciStateOfTheGpuReadWhenJudgedUnusable,
};
use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};

use crate::adapter_inventory::{
    discrete_gpus_of_the_enumeration_under_the_named,
    identifiers_of_the_discrete_gpus_in_the_enumeration,
    participating_gpus_admitted_by_the_mount_checks, AdapterDeviceKind, BackgroundGpuSelection,
    DiscreteGpuListRefused, DiscreteGpuOnThisHost, DiscreteGpusOfTheEnumeration,
    DiscreteGpusPresentNow, EnumeratedAdapterDescription, ParticipatingGpu,
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
    gpu_opened_under_its_memory_limit, gpus_dropped_leaking_those_still_running,
    gpus_opened_at_the_mount, measured_cost_summed_over_the_pieces,
    memory_limit_of_the_device_under_the_settings, plans_of_a_batch_filling_the_gpus_in_order,
    BatchNotPlannedOverTheGpus, CostAtTheOpening, CostProbeChunk, DiscreteGpusAfterTheOpening,
    GpuBudgetUnderTheSettings, GpuLeakedWhenDropped, GpuLeftOutUnderItsMemoryLimit,
    GpuMemoryLeakedInThisProcess, GpusLeakedWhenClosedAtASwitch,
    GpusLeakedWhenTheBackgroundGpuIsDropped, ParticipatingGpusOnThisHost, PlanOfAGpuForTheBatch,
    ReopeningsAndPciStateReadingOfAGpuNotOpened, RoomOfAGpuForTheBatch,
};
use crate::live_slot_kernels::{
    decoded_live_slot_outcomes, live_slot_chunk_footprint,
    live_slot_outcome_buffer_before_the_kernel_writes, live_slot_parameter_bytes,
    live_slot_plan_on_a_gpu, live_slots_summed_over_the_record_slices,
    reconciled_live_slots_of_the_segments, segment_bound_words, AllocationRecordCountOfATable,
    AllocationRecordTablesForTheKernels, ClusterSegmentCountOfAGpu, RecordSliceWithItsChunks,
    LIVE_SLOT_INVOCATIONS_PER_WORKGROUP, LIVE_SLOT_OUTCOME_BYTES,
    SEARCHING_SORTED_RECORDS_KERNEL_ENTRY_POINT, SEARCHING_SORTED_RECORDS_KERNEL_SOURCE,
    SEGMENT_BOUND_BYTES, WALKING_EVERY_RECORD_KERNEL_ENTRY_POINT,
    WALKING_EVERY_RECORD_KERNEL_SOURCE,
};
use crate::measured_host_cpu_time_per_submission::HostCpuNanosecondsPerSubmission;
use crate::memory_limit_and_chunks::{
    input_buffer_bytes, pieces_cut_to_fit_on_the_gpu, ChunkBoundsOfAGpu, EntryCountOfAChunk,
    InputLengthOnTheHostInBytes, SmallestPieceLargerThanTheRoom, SubmissionOfTheChunk,
    COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS,
    COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION,
};
use crate::pci_state_of_the_gpu::PciStateReaderOfTheGpus;
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
use crate::two_kernel_reconciliation::{
    first_entry_key_repeated_among, ConsecutiveUncertaintyCounts, ReconciledBatch,
    SameEntryKeyTwiceInOneBatch,
};
use crate::waiting_for_the_gpu::{
    gpu_opened_with_the_reopenings, outcome_of_opening_a_gpu_within_what_is_left_of_the_limit,
    outcome_resubmitted_after_timeouts, outcome_waited_for_again_after_timeouts,
    wait_for_completion_before_the_deadline, CompletionCheck, GpuWaitDeadline, MonotonicClock,
    ReopeningsOfAGpuNotOpened, SteadyClockOfThisProcess, TimeoutsOfTheBatchOnOneGpu,
    WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
};

/// 各段的着色器按字节偏移用 `u32` 寻址、整字那一段按「剩下的字节 ≥ 4」走：一个绑定不超过 2^31 字节，偏移加 4 也不回绕。
const LARGEST_STORAGE_BINDING_THE_KERNELS_ADDRESS_IN_BYTES: GpuMemoryInBytes =
    GpuMemoryInBytes(1 << 31);
/// 挂载那一刻量代价交的那一段：一条 4096 字节全 0 的单元（大小取一个常见的单元块；它只用来量一次提交的主机 CPU，结果不用），走校验和核对那一段的缓冲。
/// 量的时候照这张卡开卡时的界切成几片逐段交（[`CostProbeChunk::pieces_under_the_bounds`]，与挂着时校验和那一段同一个切法），代价取各片之和
/// （[`measured_cost_summed_over_the_pieces`]）；真建的缓冲与 `CHECKSUM_CHUNK_FOOTPRINT` 一致（[`CardOfTheBackgroundGpu::both_kernel_outcomes_of_a_checksum_chunk`] 核过）。
pub const COST_PROBE_CHUNK: CostProbeChunk = CostProbeChunk {
    input_bytes: InputLengthOnTheHostInBytes(4096),
    footprint: CHECKSUM_CHUNK_FOOTPRINT,
};
/// 量几次取中位数：先空跑一次（管线刚编好时驱动的懒初始化不算进去），再量这么多次。
const COST_MEASUREMENT_SUBMISSIONS: usize = 5;
/// 两份核的结果缓冲：核写、拷给回读暂存。
const OUTCOME_BUFFER_USAGE: wgpu::BufferUsages =
    wgpu::BufferUsages::STORAGE.union(wgpu::BufferUsages::COPY_SRC);

/// 一次 GPU 出错是什么（第 4 条）：调用方的决定只有一个——这一批放弃、后台任务停下并报错，成员只为报得清；开卡那几种
/// （起不了设备或编不了管线、起不了开卡线程、开卡 panic、前一次开卡没回来、叫停、连着试满）报给人时出路各不相同，各是一个成员。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GpuFailureCause {
    /// wgpu 起不了这张卡的设备或编不了管线（驱动不对、显存不够；量代价那一段最小的一片装不进绑定上限也报这一种）。
    DeviceOrPipelineDidNotStart { reason: String },
    /// 系统起不了开卡的那条线程（`reason` 是起线程报的原话）：算这张卡这一次没开成，照重开。
    OpeningThreadNotStarted { reason: String },
    /// 开卡那一步里 panic 了（`said_by_the_panic` 是 panic 带的原话；驱动里的与本包自己的断言都算）：算这一次没开成，照重开。
    OpeningPanicked { said_by_the_panic: String },
    /// 同一张卡上前一次开卡到点之后还没回来（开它的那一方到点不等了，那条线程还挂在驱动里），这一次不再起：算这一次又没开成，照间隔往下数；
    /// 不记进这张卡在一次挂载里连着开不成的次数（不因它判成试满，之后的切换照常为它起开卡、照旧报这一种）；出路只有重启进程。
    /// 前一次没到点、还在开的（别的挂载正在开这张卡）不报这一种：这一次在自己的期限之内排队等它开完再开（[`OpeningsNotReturnedInThisProcess`]）。
    EarlierOpeningOfTheDeviceNotReturned,
    /// 切换开卡叫停了（卸载或切到 `cpu`），不再开：这一次的结局没人收，原因不报到哪里。
    OpeningCalledOff,
    /// 这张卡在这一次挂载里已连着开了 `failed_openings` 次都没开成（跨切换接着数，开成一次清零；[`ReopeningsOfAGpuNotOpened::openings_at_most`]），切换再点到它不再开：要卸载重挂。
    OpeningsInThisMountUsedUp { failed_openings: u32 },
    /// 一段建缓冲出错之后冲掉队列待写的那一次空提交，等满上限还没做完（已建的那几份缓冲连同暂存还占着卡上的显存）：这一段没交到卡上
    /// （不算进重交数），算这一批这张卡一次等超过上限。
    FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { wait_limit: Duration },
    /// wgpu 报了没接住的错（校验错）或设备丢了；等提交时 wgpu 的 `Device::poll` panic 了也算这一种（`completion_check_of_one_poll_with_its_panic_caught`）。
    UncapturedErrorOrDeviceLost { reason: String },
    /// 等这一次提交没等成（不是超时）。
    WaitForTheSubmissionFailed { reason: String },
    /// 等 GPU（等 wgpu 的 future、等一次提交做完、等回读映射好）超过了开包时给的上限。
    WaitForTheGpuExceededTheLimit { wait_limit: Duration },
    /// 这张卡上前一次等超过上限的那次提交，下一批在这张卡上交活之前又等满了上限还没在 GPU 上做完：它的缓冲还占着卡上的显存，
    /// 不越过上限去交（第 3、4 条）。
    EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { wait_limit: Duration },
    /// 回读缓冲映射不成。
    ReadbackMappingFailed { reason: String },
    /// 读回的判定字认不得：核没写到那一条或写坏了。
    KernelLeftAnUnrecognizedVerdictWord(UnrecognizedVerdictWord),
    /// 在卡上建一段的缓冲时显存不足（wgpu 在显存不足的错误范围里报的），`bytes_held_by_the_chunk` 是那一段合占的字节数；这一段没交，
    /// 已建的那几份连同暂存冲掉、做完了，它的账已经放掉（没等到做完的不报这一种，报等超过上限）。
    /// 挂着时外壳按更小的片重切重交，切到最小的一片都建不成就这一批不用这张卡、活转给别的卡（[`BackgroundGpuOverTheCards`]），不报出来。
    OutOfMemoryCreatingTheBuffers {
        bytes_held_by_the_chunk: GpuMemoryInBytes,
        reason: String,
    },
    /// 一批在一张卡上第 4 次等超过上限（`waiting_for_the_gpu::RESUBMISSIONS_AFTER_A_TIMEOUT` 加 1；一批切成几片时各片的超时合起来算，
    /// 交活之前、重交之前等前一次那一次没等到、冲掉待写的那一次空提交没等到也算一次），`resubmissions` 是这期间真交到卡上的重交次数（重交之前那一等没等到、
    /// 重交那一次建缓冲出错没交出去的不算，所以可以比 3 少）：这张卡用不了，
    /// 交核心层当报警。`last_cause` 是最后那一次的原因（等超过上限的那三种之一）。
    UnusableAfterResubmissions {
        resubmissions: u32,
        last_cause: Box<GpuFailureCause>,
    },
    /// 挂着切换点到的这张卡开不成，隔 1 分钟重开 3 次也都没开成（第 4 条；`waiting_for_the_gpu::gpu_opened_with_the_reopenings`）：
    /// `causes_of_each_opening` 是每一次开不成的原因，按开的次序（头一次的在前，至多 1 + 重开次数 个；这一次挂载里早先已开不成过的扣掉，
    /// 连着试满、这一次不再开的只一个 [`Self::OpeningsInThisMountUsedUp`]）。交核心层当报警，不自动再试。
    NotOpenedAfterTheReopenings {
        causes_of_each_opening: Vec<GpuFailureCause>,
    },
}

/// 挂载那一刻开不了后台 GPU（第 4 条：拒绝挂载并报错，不降级、不退回 CPU）。成员按调用方要改哪一样分。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackgroundGpuOpenRefused {
    /// 挂载参数配错或 GPU 不在：核心层那一套校验拒的（改参数，或装上卡）。核心层拒成「点名的卡不在卡清单里」的不在这里，在
    /// [`Self::NamedGpuDeviceNotOnTheListOfThisEnumeration`]。
    SettingsRefused(BackgroundComputeSettingsRefused),
    /// 点名的卡不在这一次枚举交成的卡清单里（核心层拒成 `NamedGpuDeviceNotInTheInventory`）：连同这一次枚举到的每个适配器与没收进清单的那几张，
    /// 分得清是这一次枚举里没有这张卡、还是有独显而它交不成一张卡（例：没报总线地址）。
    NamedGpuDeviceNotOnTheListOfThisEnumeration(NamedGpuDeviceNotOnTheListOfThisEnumeration),
    /// 枚举到的独立显卡交不成一份卡清单（条款没写的形态，第一版不支持）。
    DiscreteGpuListNotSupported(DiscreteGpuListRefused),
    /// 枚举适配器这一步交回等超过了上限：还没有哪一张卡可点名。wgpu 在造 future 那一步同步做完枚举，那一步不在上限之内（驱动在枚举里挂住时开卡跟着挂住、
    /// 不拒，走不到这里）；期限从枚举做完才起算，上限小到枚举刚做完头一次看就到期才走到这里。上限给 0 不走这里，在枚举之前就拒成
    /// [`Self::SettingsRefused`] 的 `GpuWaitLimitOfZero`。
    AdapterEnumerationFailed { cause: GpuFailureCause },
    /// 一张参与的卡用不上（起不了设备、编不了管线、量代价那几次提交出错或超时），隔 1 分钟重开 3 次也都没开成
    /// （`waiting_for_the_gpu::gpu_opened_with_the_reopenings`）：给了 `background_gpu_devices` 时是点名的卡，没给时本机任何一张独显。
    /// `causes_of_each_opening` 是每一次开不成的原因，按开的次序（头一次的在前，1 + 重开次数 个）。
    /// 没点名的卡不开，不走这里。显存上限连最小的一片都装不下的卡不拒，
    /// 不用它、报出来（[`BackgroundGpu::gpus_left_out_under_their_memory_limit`]）。
    /// `pci_state_when_judged_unusable` 是重开用完、判它开不成那一刻照它的 PCI 地址读的本机 PCI 状态（`crate::pci_state_of_the_gpu`；每一次没开成而还要重开时不读）。
    GpuUnusable {
        device: GpuDeviceIdentifier,
        causes_of_each_opening: Vec<GpuFailureCause>,
        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
    },
}

/// 点名的卡不在这一次枚举交成的卡清单里：点的是哪张卡、这一次枚举到的每个适配器（按枚举的次序，不论算不算显卡）、
/// 给了 `background_gpu_devices` 而没点名、交不成一张卡而没收进清单的那几张独显（各带原因，按枚举的次序；同
/// [`BackgroundGpuOverTheCards::unnamed_discrete_gpus_left_off_the_list`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedGpuDeviceNotOnTheListOfThisEnumeration {
    pub device: GpuDeviceIdentifier,
    pub adapters_in_the_enumeration: Vec<AdapterSeenInTheEnumeration>,
    pub unnamed_left_off_the_list: Vec<DiscreteGpuListRefused>,
}

/// 枚举到的一个适配器在拒绝里报的两样：哪一类设备与 PCI 总线地址（驱动不报时是空串）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterSeenInTheEnumeration {
    pub device_kind: AdapterDeviceKind,
    pub pci_bus_address: String,
}

impl AdapterSeenInTheEnumeration {
    #[must_use]
    pub fn of_the_description(description: &EnumeratedAdapterDescription) -> Self {
        Self {
            device_kind: description.device_kind,
            pci_bus_address: description.pci_bus_address.clone(),
        }
    }
}

/// 一张参与的卡连这一批最小的一片都装不下：哪张卡、
/// 那一片要切的条目在这一批里的位置、给它的显存上限（不给是 `None`：最小的一片装不进绑定上限）、
/// 最小的一片合占的字节数（这张卡至少要剩这么多）。这一批一张卡都还没交活。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmallestPieceNotHeldOnTheGpu {
    pub device: GpuDeviceIdentifier,
    pub entry_position: usize,
    pub memory_limit: Option<GpuMemoryInBytes>,
    pub smallest_piece_bytes: GpuMemoryInBytes,
}

/// 一批校验和核对交不成。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChecksumBatchFailed {
    /// GPU 出错（含一次提交在同一张卡上重交 3 次都等超过上限、前一次超时的提交在交这一批之前又没等到做完、换设置时量代价出错）：
    /// 这一批放弃、不换 CPU（第 4 条）。`pci_state_when_judged_unusable` 是判这张卡用不了那一刻读的 PCI 状态
    /// （等超时用完、卡当场报出错时现读，`crate::pci_state_of_the_gpu::whether_the_cause_judges_the_gpu_unusable_in_a_batch`；
    /// 切换点到的卡重开到上限都没开成的，带判它开不成那一刻读的那一份；别的没读）。
    GpuFailed {
        device: GpuDeviceIdentifier,
        cause: GpuFailureCause,
        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
    },
    /// 最小的一片（一个字节的单元）都装不下分到它的那张卡。
    GpuCannotHoldTheSmallestPiece(SmallestPieceNotHeldOnTheGpu),
    /// 这组设置点的卡的显存上限连最小的一片都装不下，参与的卡一张都不剩：按参与的次序列出每张卡、上限与至少要多少。一张卡都没交活。
    EveryParticipatingGpuLeftOutUnderItsMemoryLimit(Vec<GpuLeftOutUnderItsMemoryLimit>),
    /// 这一批参与的卡一张都不剩，其中至少一张是卡上空闲的显存连最小的一片都建不成（
    /// 这一批不用它，它还没交成的活转给这一批别的参与的卡重交，全都不剩才是这一种）：因空闲显存不用的按先后列出，连同这一段因显存上限不用的。
    EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
        gpus_short_of_free_memory: Vec<GpuShortOfFreeMemoryForTheSmallestPiece>,
        gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
    },
    /// 调用方交进来的这一批里同一个条目键出现了两次：两份核的结果不比、连续不确定的计数不动、CPU 参照不调，这一批放弃（同 GPU 出错）。
    SameEntryKeyTwiceInOneBatch(SameEntryKeyTwiceInOneBatch),
}

impl From<SameEntryKeyTwiceInOneBatch> for ChecksumBatchFailed {
    fn from(repeated: SameEntryKeyTwiceInOneBatch) -> Self {
        Self::SameEntryKeyTwiceInOneBatch(repeated)
    }
}

/// 一批区域扫描的单元头解析与映射反查交不成。后三种在一张卡都还没交活之前判出。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegionRecordBatchFailed {
    /// GPU 出错（同 [`ChecksumBatchFailed::GpuFailed`]）：这一批放弃、不换 CPU（第 4 条）。
    GpuFailed {
        device: GpuDeviceIdentifier,
        cause: GpuFailureCause,
        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
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
    /// 同 [`ChecksumBatchFailed::SameEntryKeyTwiceInOneBatch`]。
    SameEntryKeyTwiceInOneBatch(SameEntryKeyTwiceInOneBatch),
}

impl From<SameEntryKeyTwiceInOneBatch> for RegionRecordBatchFailed {
    fn from(repeated: SameEntryKeyTwiceInOneBatch) -> Self {
        Self::SameEntryKeyTwiceInOneBatch(repeated)
    }
}

/// 一批选段打分交不成。后两种在一张卡都还没交活之前判出。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveSlotBatchFailed {
    /// GPU 出错（同 [`ChecksumBatchFailed::GpuFailed`]）：这一批放弃、不换 CPU（第 4 条）。
    GpuFailed {
        device: GpuDeviceIdentifier,
        cause: GpuFailureCause,
        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
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
    /// 同 [`ChecksumBatchFailed::SameEntryKeyTwiceInOneBatch`]（键是段）。
    SameEntryKeyTwiceInOneBatch(SameEntryKeyTwiceInOneBatch),
}

impl From<SameEntryKeyTwiceInOneBatch> for LiveSlotBatchFailed {
    fn from(repeated: SameEntryKeyTwiceInOneBatch) -> Self {
        Self::SameEntryKeyTwiceInOneBatch(repeated)
    }
}

/// 此刻这组设置下一批分不到卡上（[`BackgroundGpu::batch_not_split_over_the_gpus`]）：最近交来的那一组点到的一张没开的卡重开到上限都没开成、
/// 或换设置时量一张卡的代价出错了（这张卡没参与，报成它的 GPU 出错），或参与的卡因显存上限一张都不剩。这组设置下每一批三段都交回它，直到下一次换设置。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BatchNotSplitOverTheGpus {
    GpuNotOpenedAfterTheReopeningsAtTheSwitch(GpuNotOpenedAfterTheReopenings),
    /// `pci_state_when_judged_unusable` 是量代价出错那一刻读的这张卡的 PCI 状态：出错的原因是判卡用不了的那一种
    /// （卡当场报出错、等超时用完，`crate::pci_state_of_the_gpu::whether_the_cause_judges_the_gpu_unusable_in_a_batch`）才读，不是的没读；
    /// 之后每一批、两批之间每一问带的都是这一份，不重读，下一次换设置清掉它。
    CostNotMeasuredAtTheSwitch {
        device: GpuDeviceIdentifier,
        cause: GpuFailureCause,
        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
    },
    /// 按参与的次序列出每张被晾下的卡、给它的上限与至少要多少，至少一张。
    EveryParticipatingGpuLeftOut(Vec<GpuLeftOutUnderItsMemoryLimit>),
}

/// 两批之间那一问（[`BackgroundGpuOverTheCards::put_the_gpus_opened_since_the_switch_in_effect`]）交回的：最近交来的那一组此刻怎样了。封闭集合。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LatestSettingsHandedIn {
    /// 还在会话锁外为它开卡：照旧照交来之前那一组跑，这一组还没生效。
    StillOpeningItsCards,
    /// 换上了（交来那一刻点的卡都开着就当场换上，开完了在两批之间换上），连同这组设置下一批分不分得到卡上
    /// （[`BackgroundGpuOverTheCards::batch_not_split_over_the_gpus`]，分得到是 `None`）。
    PutInEffect {
        not_split_over_the_gpus: Option<BatchNotSplitOverTheGpus>,
    },
    /// 它点到的一张没开的卡重开到上限都没开成：之后每一批报成这张卡的 GPU 出错，直到下一次换设置。
    NotOpened(GpuNotOpenedAfterTheReopenings),
}

/// 交来一组设置（或告知一张卡都不参与）的次序号：开卡的结局认它是为哪一次交来的开的，不按设置的内容比
/// （同一组设置先后交两次是两次，为前一次开的、开不成的不报给后一次）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HandingInNumber(u64);

impl HandingInNumber {
    /// 下一次交来的号。
    fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

/// 交来了、点到没开的卡、还没换上的那一组，连同它是哪一次交来的。
struct SettingsWaitingForTheOpening {
    handing_in_number: HandingInNumber,
    settings: AdmittedGpuSettings,
}

/// 换设置时量代价出错的那张卡：哪张卡、出错的原因、出错那一刻读的它的 PCI 状态（原因不是判卡用不了的那一种时没读）。
#[derive(Clone, Debug)]
struct GpuWhoseCostWasNotMeasuredAtTheSwitch {
    device: GpuDeviceIdentifier,
    cause: GpuFailureCause,
    pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
}

/// 起切换开卡那条线程的办法（`.claude/rules/fs-design.md` 五条硬要求第 2 条：起不了线程那一支测得到）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartOfTheOpeningThread {
    /// 产品：照 std 起。
    ByTheStandardLibrary,
    /// 只供测试：接下来这么多次照起不了线程报，之后照 std 起。
    RefusedForTests { refusals_left: u32 },
}

/// 挂着切换点到的一张没开的卡，隔一段时间重开到上限也都没开成（第 4 条「3 次都不成，GC 停下，报警里报出哪张卡、每次开不成的原因」），
/// 或它在这一次挂载里已连着试满、这一次不再开：哪张卡、每一次开不成的原因（按开的次序，头一次的在前，见 [`GpuFailureCause::NotOpenedAfterTheReopenings`]）。
/// `pci_state_when_judged_unusable` 是判它开不成那一刻照它的 PCI 地址读的本机 PCI 状态（`crate::pci_state_of_the_gpu`，读一次）：
/// 之后每一批、两批之间每一问报的都是这一份，不重读；下一次换设置清掉它，那张卡再一次被判开不成时才再读。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GpuNotOpenedAfterTheReopenings {
    pub device: GpuDeviceIdentifier,
    pub causes_of_each_opening: Vec<GpuFailureCause>,
    pub pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
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
/// 建缓冲显存不足的那一段不交：已建的那几份连同暂存冲掉、做完了才放账、交回 [`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]，
/// 没等到做完的账留着、记成还在跑的那次提交，交回等超过上限。
pub trait CardOfTheBackgroundGpu: Send + 'static {
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

/// 挂着切换点到一张没开的卡时开它一次（同挂载那一刻开卡那一步）：交给它这张卡、这一组给它的显存上限、等 GPU 的上限、量不量代价，
/// 交回开着的卡或这一次没开成的原因（开不成重开归调用方，`waiting_for_the_gpu::gpu_opened_with_the_reopenings`）。
/// 产品是在这张卡的 wgpu 适配器上起设备、编管线、量代价（[`opening_of_a_gpu_in_use`]），测试里交造出来的卡。在会话锁外那条开卡线程上调。
pub type OpeningOfAGpuAtASwitch<Card> = Box<
    dyn FnMut(
            &DiscreteGpuOnThisHost,
            Option<GpuMemoryInBytes>,
            Duration,
            CostAtTheOpening,
        ) -> Result<Card, GpuFailureCause>
        + Send,
>;

/// 这一次挂载的后台 GPU 在几张开着的卡上（`Card` 是一张开着的卡，产品是 [`GpuInUse`]）：开着的本机独显（挂载那一刻开的参与的卡按总线地址排，
/// 切换开的依次排在后面；切换之后不再点名的关掉）与没开的那几张，给了点名时没点名而交不成一张卡、没收进清单的那几张，与此刻参与的那几张（设置定的，按参与的次序：
/// 没点名是总线地址的次序，点了名是点名的次序；显存上限连最小的一片都装不下的不在里面）。生命周期一次挂载，只住内存。丢它时不等还在 GPU 上跑的提交（`Drop`）。
pub struct BackgroundGpuOverTheCards<Card: CardOfTheBackgroundGpu> {
    /// 开着的那几张，与 `discrete_gpus_after_the_opening.opened` 一一对应、同序。
    cards_on_this_host: Vec<Card>,
    discrete_gpus_after_the_opening: DiscreteGpusAfterTheOpening,
    unnamed_discrete_gpus_left_off_the_list: Vec<DiscreteGpuListRefused>,
    participating: ParticipatingGpusOnThisHost,
    /// 此刻这组设置点了而因显存上限装不下最小的一片不用的卡，按参与的次序。
    left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
    /// 最近交来的那一组点到的一张没开的卡重开到上限都没开成：之后每一批都报成它的 GPU 出错，直到下一次换设置。
    not_opened_after_the_reopenings_at_the_last_switch: Option<GpuNotOpenedAfterTheReopenings>,
    /// 最近一次换设置时量一张新参与的卡的代价出错（这张卡没参与）：之后每一批都报成它的 GPU 出错，带出错那一刻读的那一份 PCI 状态，直到下一次换设置。
    cost_not_measured_at_the_last_switch: Option<GpuWhoseCostWasNotMeasuredAtTheSwitch>,
    /// 切换点到没开的卡时开它一次的那一步：开卡线程开着时那条线程锁着它，开完了放开。
    opening_of_a_gpu_at_a_switch: Arc<Mutex<OpeningOfAGpuAtASwitch<Card>>>,
    /// 开不成时怎么重开（挂载那一刻与切换同一种；产品是 [`ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT`]）。
    reopenings_of_a_gpu_not_opened: ReopeningsOfAGpuNotOpened,
    /// 最近一次交来设置（或告知一张卡都不参与）的次序号。
    latest_handing_in: HandingInNumber,
    /// 此刻生效的那一组点的卡：挂载那一刻是开着的那几张，换上一组时换成它点的，告知一张卡都不参与之后是空。
    /// 最近交来的那一组开不成时，开着的卡里不在这里的全部关掉。
    devices_named_by_the_settings_in_effect: Vec<GpuDeviceIdentifier>,
    /// 交来了、点到没开的卡、还没换上的那一组（最近交来的那一组；开卡开完了才换上）。没有是 `None`。
    settings_waiting_for_the_opening: Option<SettingsWaitingForTheOpening>,
    /// 会话锁外为切换开卡的那一次（至多一次）；没在开是 `None`。
    opening_at_a_switch: Option<OpeningAtASwitch<Card>>,
    /// 起切换开卡那条线程的办法：产品恒是照 std 起。
    start_of_the_opening_thread: StartOfTheOpeningThread,
    /// 这一次挂载里每张卡切换开卡连着开不成了几次（开卡线程也记，开成一次清零）：试满的卡再被点到就不再开、当场答开不成。
    failed_openings_in_this_mount: FailedOpeningsOfEachGpuInThisMount,
    /// 切换关卡时泄漏的卡，按关的先后累计（同卸载的规矩：有提交还在跑的那张整张泄漏、不析构）。
    gpus_leaked_when_closed_at_a_switch: GpusLeakedWhenClosedAtASwitch,
    /// 上一次交回以来三段各自在卡上是怎么交的（[`Self::take_the_submissions_of_the_batch`]）。
    submissions_since_the_last_take: GpuSubmissionsOfTheBatch,
    gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped,
    memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
    /// 此刻本机在的独显（卡清单每次答之前现查）。产品里它带着这个进程的 Vulkan 实例（[`VulkanInstanceOfThisProcess`] 的一份克隆）现枚举：
    /// 排在卡后面，卡都丢了才丢它（实例本身由调用方留着，活过这一次挂载）。
    discrete_gpus_present_now: Box<dyn DiscreteGpusPresentNow + Send>,
    /// 判一张卡用不了那一刻读它的 PCI 状态的那一份（产品读本机的 PCI 设备目录，测试换成自己的目录）：一批在一张卡上等超时用完或卡当场报出错时、
    /// 切换点到的卡重开到上限都没开成（或这一次挂载里已试满）时各读一次。
    pci_state_reader: PciStateReaderOfTheGpus,
}

/// 产品那一份：开着的卡是 wgpu 设备与六条管线。
pub type BackgroundGpu = BackgroundGpuOverTheCards<GpuInUse>;

/// 会话锁外为切换开卡的那一次：为哪一次交来的设置开（交来那一刻点到的没开的卡），与它的结局从哪来。封闭集合。
enum OpeningAtASwitch<Card> {
    /// 开卡线程起来了、在开：开完了经 `outcome_receiver` 送回（至多一次）。`called_off` 置上之后那条线程不再开下一次（卸载、切到 `cpu` 时置）。
    UnderWay {
        handing_in_number: HandingInNumber,
        outcome_receiver: Receiver<OutcomeOfTheOpeningAtASwitch<Card>>,
        opening_thread: JoinHandle<()>,
        called_off: Arc<AtomicBool>,
    },
    /// 起不了开卡线程：这算头一张要开的卡这一次没开成（`causes_of_each_start` 每次一个原因），隔重开的间隔再起（`not_started_at` 起算），
    /// 连同线程里的开卡至多开 1 + 重开次数 次。
    ThreadNotStarted {
        handing_in_number: HandingInNumber,
        discrete_gpus_to_open: Vec<(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)>,
        wait_limit: Duration,
        causes_of_each_start: Vec<GpuFailureCause>,
        not_started_at: Instant,
    },
    /// 起不了开卡线程的次数到了重开的上限：结局已定（头一张要开的卡开不成），下一次两批之间收下。
    EndedWithoutAThread {
        handing_in_number: HandingInNumber,
        outcome: OutcomeOfTheOpeningAtASwitch<Card>,
    },
}

impl<Card> OpeningAtASwitch<Card> {
    /// 叫停这一次开卡：开卡线程在下一次开卡之前看到就不再开（正在开的那一次照旧开完）。没起线程的不用叫停。
    fn call_off(&self) {
        match self {
            OpeningAtASwitch::UnderWay { called_off, .. } => {
                called_off.store(true, Ordering::SeqCst)
            }
            OpeningAtASwitch::ThreadNotStarted { .. }
            | OpeningAtASwitch::EndedWithoutAThread { .. } => {}
        }
    }
}

/// 开卡线程开完了送回的：开成的卡（按开的次序），与重开到上限都没开成的那一张（有一张开不成，后面的就不再开）。
struct OutcomeOfTheOpeningAtASwitch<Card> {
    opened: Vec<Card>,
    not_opened: Option<GpuNotOpenedAfterTheReopenings>,
}

/// 两批之间取切换开卡的结局时等不等它开完（[`BackgroundGpuOverTheCards::put_the_gpus_opened_since_the_switch_in_effect`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitForTheOpeningAtASwitch {
    /// 产品：不等，还在开就照旧照交设置之前那一组跑（核心层每一批开头在会话锁里问，不能在锁里等开卡）。
    NotAtAll,
    /// 等到开完：测试里不靠钟就走到「开完了换上」那一格。
    UntilItEnds,
}

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
    /// 等这张卡上一次提交时 wgpu 的 `Device::poll` panic 了：记下 panic 的原话，之后这张卡的每一次等都不再调 wgpu、当场照它报
    /// （[`completion_check_of_one_poll_with_its_panic_caught`]）。
    panic_in_a_wait_of_the_device: OnceLock<String>,
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
        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
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
            OutcomesOfTheBatchNotComputed::GpuFailed {
                device,
                cause,
                pci_state_when_judged_unusable,
            } => ChecksumBatchFailed::GpuFailed {
                device,
                cause,
                pci_state_when_judged_unusable,
            },
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
            OutcomesOfTheBatchNotComputed::GpuFailed {
                device,
                cause,
                pci_state_when_judged_unusable,
            } => RegionRecordBatchFailed::GpuFailed {
                device,
                cause,
                pci_state_when_judged_unusable,
            },
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
            OutcomesOfTheBatchNotComputed::GpuFailed {
                device,
                cause,
                pci_state_when_judged_unusable,
            } => LiveSlotBatchFailed::GpuFailed {
                device,
                cause,
                pci_state_when_judged_unusable,
            },
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
/// （范围开着时建不成的错归最里层那个范围，没有范围开着时报成没接住的错）判得出回读暂存是不是建在范围里（
/// 真卡上造不出只让回读暂存建不成的条件），也数得出这一段真建了几份、各多少字节。
trait DeviceCreatingTheBuffersOfAChunk {
    type Buffer;
    type OutOfMemoryScope;

    fn open_the_out_of_memory_scope(&self) -> Self::OutOfMemoryScope;

    /// 一份核的回读暂存：`outcome_bytes` 字节，映射来读、接结果缓冲的拷贝。
    fn readback_of_one_kernel(&self, outcome_bytes: GpuMemoryInBytes) -> Self::Buffer;

    /// 这份缓冲建的时候要的字节数（建没建成都照要的报：建不成的那一份，显存不足由范围接住）。
    fn bytes_of_the_buffer(&self, buffer: &Self::Buffer) -> GpuMemoryInBytes;

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

    fn readback_of_one_kernel(&self, outcome_bytes: GpuMemoryInBytes) -> wgpu::Buffer {
        self.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: outcome_bytes.0,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn bytes_of_the_buffer(&self, buffer: &wgpu::Buffer) -> GpuMemoryInBytes {
        GpuMemoryInBytes(buffer.size())
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
/// （`bytes_held_by_this_chunk`），建在范围之外时它显存不足报成没接住的错、这一批出错停下，不走「这一批转给别的卡」。
/// 账核真建的缓冲：关上范围之后把这一段真建的每一份缓冲要的字节加起来（`create_the_buffers` 建的都映射着建，各连同它的暂存算
/// [`COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION`] 份；两份回读暂存不映射着建，各算一份），与记在这张卡账上的 `bytes_held_by_this_chunk`
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
    bytes_held_by_this_chunk: GpuMemoryInBytes,
    outcome_bytes: GpuMemoryInBytes,
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
    let bytes_of_the_buffers_created: GpuMemoryInBytes = buffers
        .as_ref()
        .iter()
        .map(|buffer_mapped_at_creation| {
            COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
                .bytes_of_the_copies(device.bytes_of_the_buffer(buffer_mapped_at_creation))
        })
        .chain(
            readbacks
                .iter()
                .map(|readback| device.bytes_of_the_buffer(readback)),
        )
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
/// 不用 wgpu 的 `create_buffer_init`：它建完就对映射区 `expect`，缓冲因显存不足建不成时那一句 panic（wgpu 30.0.1
/// `src/util/device.rs`），调用方开着的显存不足的错误范围没机会关、走不到「切小重切、这一批转给别的卡」。映射不上只有一种：缓冲没建成（显存不足），
/// 那一条错已归调用方开着的那个范围，这里不写、不解开映射（对建不成的缓冲解开映射会另报一条没接住的错）。
/// 用法里没有 `MAP_WRITE`：wgpu-core 30.0.1 为它另建一份同尺寸的暂存，解开映射时挂进队列的待写，冲掉它的那一次提交做完才放
/// （[`COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION`]，切批与记账都算这一份）。
fn buffer_mapped_at_creation_within_the_scope_of_the_caller(
    device: &wgpu::Device,
    bytes: GpuMemoryInBytes,
    usage: wgpu::BufferUsages,
    write_the_contents: impl FnOnce(&mut wgpu::BufferViewMut),
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: bytes.0,
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

/// 这个进程的那一个 wgpu 实例（只开 Vulkan 后端、不带显示句柄）：调用方在进程里建一份、跨多次挂载留着，每次开卡都交同一份
/// （传法同 [`GpuMemoryLeakedInThisProcess`]）。开卡时的枚举、挂着切换时现查本机在的独显（[`AdaptersEnumeratedByTheInstance`] 拿它的一份克隆，
/// 克隆指的是同一个实例）、本包测试与实验装置枚举本机适配器都用它，不另建。
/// 与泄漏的那一份并列、不合进去：泄漏的那一份不要卡也建（不要卡的测试与 CPU 臂随处新建），实例只在要开卡、要枚举本机适配器时才建，
/// 不开 `tests-on-a-real-gpu` 的测试不建（`Cargo.toml` 的特性说明「不碰 GPU」）。
///
/// 为什么一个进程只建一个：同一个进程里新建的实例多了，Vulkan 驱动不一定还认得出独显——第二台上同一个进程里新建到第 12 个实例起，
/// 枚举只剩软件 Vulkan、三张独显一张都不报（前面的实例都已丢掉），开卡就拒点名的卡不在清单里；只建一个实例、反复枚举，每一次三张都在。
///
/// 这个实例枚举到的驱动清单定在建它的那一刻：之后修好、装上或藏起驱动，同一个实例枚举到的不变，新建的实例才看得到（只换驱动清单的环境变量
/// `VK_DRIVER_FILES`、`VK_ICD_FILENAMES` 量的，真拔卡、真装驱动没量）。跨多次挂载的长进程遇到修好驱动这种情形，开卡器（[`crate::gpu_opener_of_this_process`]）
/// 显式重建，新建的次数有上限。
#[derive(Clone, Debug)]
pub struct VulkanInstanceOfThisProcess {
    instance: wgpu::Instance,
}

impl VulkanInstanceOfThisProcess {
    /// 建这个进程的那一个实例：调用方一个进程只建这一次、留着它，之后每次开卡都交它。
    /// 驱动清单定在这一刻（[`VulkanInstanceOfThisProcess`]）：此刻驱动不在也交回实例、不报错，这个实例枚举一直是空的，之后修好驱动它也看不到。
    #[must_use]
    pub fn created_for_this_process() -> Self {
        Self::with_the_backends(wgpu::Backends::VULKAN)
    }

    /// 一个后端都不开的实例：不装 Vulkan 加载器、不碰 GPU，枚举交回空列表。只给不要卡的单测走开卡那一条路。
    #[cfg(test)]
    fn without_any_backend() -> Self {
        Self::with_the_backends(wgpu::Backends::empty())
    }

    fn with_the_backends(backends: wgpu::Backends) -> Self {
        Self {
            instance: wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends,
                ..wgpu::InstanceDescriptor::new_without_display_handle()
            }),
        }
    }

    /// 这个实例看得到的 Vulkan 适配器，按枚举的次序。枚举本身不受 `wait_limit` 管：wgpu 在造 future 那一步同步做完枚举，期限从它做完才起算，
    /// 驱动在枚举里挂住时这里跟着挂住、不交回。
    ///
    /// # Errors
    /// 枚举做完之后的等超过 `wait_limit`：上限小到头一次看就到期（上限是 0 同样），或到期限 future 还没好。
    pub fn vulkan_adapters_enumerated_within(
        &self,
        wait_limit: Duration,
    ) -> Result<Vec<wgpu::Adapter>, GpuFailureCause> {
        wait_for_the_wgpu_future(
            self.instance.enumerate_adapters(wgpu::Backends::VULKAN),
            wait_limit,
            &SteadyClockOfThisProcess::starting_now(),
        )
    }
}

/// 开卡从哪里取这个进程的 Vulkan 实例：产品交 [`VulkanInstanceOfThisProcess`] 本身；不要卡的单测换上记下取了几次的替身，
/// 钉住每次开卡都取交进来的那一份、不另建。
pub trait VulkanInstanceHandedToTheOpening {
    /// 这一次开卡用的实例（交回的克隆指的是同一个实例）。
    fn vulkan_instance_for_this_opening(&self) -> VulkanInstanceOfThisProcess;
}

impl VulkanInstanceHandedToTheOpening for VulkanInstanceOfThisProcess {
    fn vulkan_instance_for_this_opening(&self) -> VulkanInstanceOfThisProcess {
        self.clone()
    }
}

/// 挂载那一刻：枚举适配器、判卡清单与校验、在参与的卡上起设备与各段两份核、量一次提交的主机 CPU 时间（同
/// [`open_background_gpu_with_the_memory_leaked_in_this_process`]，进程里还一张卡都没泄漏过、没有一次开卡挂住；实例是调用方留着的 `vulkan_instance`）。
///
/// # Errors
/// [`BackgroundGpuOpenRefused`] 的五种。
pub fn open_background_gpu(
    selection: &BackgroundGpuSelection,
    vulkan_instance: &VulkanInstanceOfThisProcess,
) -> Result<BackgroundGpu, BackgroundGpuOpenRefused> {
    open_background_gpu_with_the_memory_leaked_in_this_process(
        selection,
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        &OpeningsNotReturnedInThisProcess::with_none_not_returned(),
        vulkan_instance,
        |_gpus_to_open| {},
    )
}

/// 挂载那一刻：枚举适配器、判卡清单与校验、只在参与的卡上起设备与各段两份核、量一次提交的主机 CPU 时间；参与的卡照 `selection`
/// （之后切换时核心层另交，[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]）。等 wgpu 的 future 都在 `selection.background_gpu_wait_limit` 之内；
/// 枚举适配器本身不在其内：wgpu 在造 future 那一步同步做完枚举，驱动在枚举里挂住时开卡跟着挂住、不拒（[`VulkanInstanceOfThisProcess::vulkan_adapters_enumerated_within`]）。
/// 上限给 0 在枚举之前就拒成 [`BackgroundGpuOpenRefused::SettingsRefused`] 的 `GpuWaitLimitOfZero`，不看选了哪几张卡。
/// 给了 `background_gpu_devices` 时只判点名的卡（没点名的独显交不成一张卡就不收、报出来）。
/// 每张卡开卡（起设备、编各段两份核的管线、量代价）至多等 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]（五分钟），到点当那张卡没开成
/// （`waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit`，报等超过上限、带这个上限），挂载等开卡成功或到点才往下走。
/// 一张参与的卡开不成就隔 1 分钟重开、至多重开 3 次（[`ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT`]，这条线程上睡着等），都不成才拒绝挂载。
/// 枚举完、判过卡清单与校验，逐张开卡之前把要开几张卡（参与的卡数）交给 `report_the_gpus_to_open`（进程开卡器照它把外层的期限延到枚举期限加
/// 卡数乘一张卡连同重开的最长等待）；枚举不成、卡清单或校验拒了的不报。
/// 量代价出错时账上记着还在跑的提交的那张卡泄漏、不析构，这一次占着的字节当场记进 `memory_leaked_in_this_process`
/// （`failure_of_the_measurement_dropping_or_leaking_the_card`）。
/// 没参与的卡不开、不量代价（第 2 条「挂载时只开参与的卡」）：卡清单照列它，之后的切换点到它时在会话锁外开（[`OpeningOfAGpuAtASwitch`]，
/// 同一份适配器与泄漏账）。参与的卡重开到上限都没开成就拒绝挂载（[`gpus_opened_at_the_mount`]），拒绝里带判它开不成那一刻读的本机 PCI 状态
/// （[`PciStateReaderOfTheGpus::of_this_host`]；挂着之后判卡用不了时也照它读）。参与的卡照开卡那一组给它的显存上限
/// （先扣掉这个进程里之前几次挂载泄漏在它上面的字节，`memory_leaked_in_this_process`）把量代价那一段（[`COST_PROBE_CHUNK`]）切成几片逐片量、代价取各片之和；
/// 显存上限连最小的一片都装不下的卡照开、不量代价、不参与、报出来（[`BackgroundGpuOverTheCards::gpus_left_out_under_their_memory_limit`]），挂载不因此拒绝；
/// 参与的卡因此一张都不剩时照样交回，三段每一批都报「参与的卡一张都不剩」（[`BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut`]），
/// 核心层挂载那一刻就读到、当场报警停着。丢掉它时泄漏的卡记进 `memory_leaked_in_this_process`。
/// 枚举与之后现查本机在的独显都用 `vulkan_instance_handed_over` 交来的那一个实例（调用方在进程里留着的那一份，[`VulkanInstanceOfThisProcess`]），不另建。
/// 一张卡上前一次开卡到点之后还没回来就不再起下一次、没到点还在开的就排队等它开完，那一张表是 `openings_not_returned_in_this_process`（调用方在进程里留着的那一份，挂载与之后的切换开卡共用）。
///
/// # Errors
/// [`BackgroundGpuOpenRefused`] 的五种。
pub fn open_background_gpu_with_the_memory_leaked_in_this_process(
    selection: &BackgroundGpuSelection,
    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
    openings_not_returned_in_this_process: &OpeningsNotReturnedInThisProcess,
    vulkan_instance_handed_over: &impl VulkanInstanceHandedToTheOpening,
    report_the_gpus_to_open: impl FnOnce(usize),
) -> Result<BackgroundGpu, BackgroundGpuOpenRefused> {
    // 上限 0 是配错，出路是改参数；枚举那一步头一次看就算到期，要先判，不然报成枚举没成。
    if selection.background_gpu_wait_limit.is_zero() {
        return Err(BackgroundGpuOpenRefused::SettingsRefused(
            BackgroundComputeSettingsRefused::GpuWaitLimitOfZero,
        ));
    }
    let vulkan_instance = vulkan_instance_handed_over.vulkan_instance_for_this_opening();
    let adapters = vulkan_instance
        .vulkan_adapters_enumerated_within(selection.background_gpu_wait_limit)
        .map_err(|cause| BackgroundGpuOpenRefused::AdapterEnumerationFailed { cause })?;
    let descriptions: Vec<EnumeratedAdapterDescription> =
        adapters.iter().map(description_of_the_adapter).collect();
    let (discrete_gpus, participating) =
        discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
            selection,
            &descriptions,
        )?;
    report_the_gpus_to_open(participating.len());
    let mut open_a_gpu = opening_of_a_gpu_in_use(
        adapters,
        memory_leaked_in_this_process.clone(),
        openings_not_returned_in_this_process.clone(),
    );
    let pci_state_reader = PciStateReaderOfTheGpus::of_this_host();
    let gpus_opened = gpus_opened_at_the_mount(
        discrete_gpus.listed,
        &participating,
        bytes_of_the_smallest_piece_every_participating_gpu_holds(),
        memory_leaked_in_this_process,
        ReopeningsAndPciStateReadingOfAGpuNotOpened {
            reopenings: ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT,
            pci_state_reader: &pci_state_reader,
        },
        std::thread::sleep,
        |discrete_gpu, memory_limit_at_the_opening, cost_at_the_opening| {
            open_a_gpu(
                discrete_gpu,
                memory_limit_at_the_opening,
                selection.background_gpu_wait_limit,
                cost_at_the_opening,
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
            vulkan_instance,
            wait_limit: selection.background_gpu_wait_limit,
        }),
        Box::new(open_a_gpu),
        ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT,
        pci_state_reader,
    ))
}

/// 在这一次枚举到的适配器（`adapters`，按枚举的次序）里的一张独显上开卡：起设备、编各段两份核的管线（[`started_gpu_in_use`]），
/// 再照开卡那一步（[`opening_of_a_card_within_the_limit`]）量代价、出错时处置这张卡、整个开卡至多等 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]。
/// 挂载那一刻开参与的卡与切换生效那一刻开挂载时没开的卡都走它。
fn opening_of_a_gpu_in_use(
    adapters: Vec<wgpu::Adapter>,
    memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
    openings_not_returned_in_this_process: OpeningsNotReturnedInThisProcess,
) -> impl FnMut(
    &DiscreteGpuOnThisHost,
    Option<GpuMemoryInBytes>,
    Duration,
    CostAtTheOpening,
) -> Result<GpuInUse, GpuFailureCause>
       + Send
       + 'static {
    opening_of_a_card_within_the_limit(
        WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
        move |discrete_gpu: &DiscreteGpuOnThisHost,
              memory_limit_at_the_opening,
              wait_limit,
              bytes_leaked_by_earlier_mounts| {
            let adapter = adapters[discrete_gpu.position_in_the_enumeration].clone();
            let discrete_gpu_to_start = discrete_gpu.clone();
            move || {
                started_gpu_in_use(
                    &adapter,
                    discrete_gpu_to_start,
                    memory_limit_at_the_opening,
                    wait_limit,
                    bytes_leaked_by_earlier_mounts,
                )
            }
        },
        memory_leaked_in_this_process,
        openings_not_returned_in_this_process,
    )
}

/// 开一张卡那一步（挂载那一刻开参与的卡、切换生效那一刻开挂载时没开的卡，对任何一种卡都一样；`Card` 产品是 [`GpuInUse`]，测试换上替身卡）：
/// `start_of_the_card` 照这张卡、给它的显存上限、等 GPU 的上限与这个进程里之前几次挂载泄漏在它上面的字节（从 `memory_leaked_in_this_process` 现读）
/// 交回起这张卡的那一件；那一件连同量代价（[`started_gpu_with_its_cost_measured`]，量不量照 `CostAtTheOpening`；出错时有提交还在跑的那张卡泄漏、
/// 这一次占着的字节记进 `memory_leaked_in_this_process`）在另一条线程上做，整个至多等 `opening_limit`（产品是 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]；
/// 到点当那张卡没开成，`waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit`）。处置量代价出错的卡在那条线程上：
/// 到点之后那条线程跑完时交回的只是那个错，没有要析构的卡。
/// 同一张卡上前一次开卡到点之后还没回来（开它的那一方到点不等了，那条线程还挂在驱动里），就不再起下一次：这一次当场算没开成
/// （[`opening_not_started_while_an_earlier_one_has_not_returned`]），照重开的间隔往下数（挂载与切换同一个规矩）。前一次没到点、还在开的（别的挂载正在开这张卡），
/// 这一次在自己的期限之内排队等它开完再开，等的时间算进这一次的期限，等满了照这一次到点办（报等超过上限、带 `opening_limit`）。
/// 那一张表是 `openings_not_returned_in_this_process`（进程开卡器持有、各次挂载与切换共用：前一次挂载里挂住的那张卡，下一次挂载同样不再起），
/// 同一张卡上同时在开的线程整个进程至多一条。
fn opening_of_a_card_within_the_limit<Card, StartTheCard>(
    opening_limit: Duration,
    mut start_of_the_card: impl FnMut(
            &DiscreteGpuOnThisHost,
            Option<GpuMemoryInBytes>,
            Duration,
            GpuMemoryInBytes,
        ) -> StartTheCard
        + Send
        + 'static,
    memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
    openings_not_returned_in_this_process: OpeningsNotReturnedInThisProcess,
) -> impl FnMut(
    &DiscreteGpuOnThisHost,
    Option<GpuMemoryInBytes>,
    Duration,
    CostAtTheOpening,
) -> Result<Card, GpuFailureCause>
       + Send
       + 'static
where
    Card: CardOfTheBackgroundGpu + Send + 'static,
    StartTheCard: FnOnce() -> Result<Card, GpuFailureCause> + Send + 'static,
{
    move |discrete_gpu, memory_limit_at_the_opening, wait_limit, cost_at_the_opening| {
        let opening_began_at = Instant::now();
        let opening_under_way =
            match OpeningUnderWayOnTheDevice::noted_once_no_earlier_one_is_under_way(
                &openings_not_returned_in_this_process,
                &discrete_gpu.identifier,
                opening_began_at,
                opening_limit,
            ) {
                Ok(opening_under_way) => opening_under_way,
                Err(EarlierOpeningOfTheDeviceInTheWay::NotReturnedPastTheLimitOfItsCaller) => {
                    return Err(opening_not_started_while_an_earlier_one_has_not_returned());
                }
                Err(EarlierOpeningOfTheDeviceInTheWay::StillUnderWayAtTheLimitOfThisOpening) => {
                    return Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                        wait_limit: opening_limit,
                    });
                }
            };
        let opening_number = opening_under_way.opening_number;
        let bytes_leaked_by_earlier_mounts =
            memory_leaked_in_this_process.bytes_leaked_on(&discrete_gpu.identifier);
        let start_the_card = start_of_the_card(
            discrete_gpu,
            memory_limit_at_the_opening,
            wait_limit,
            bytes_leaked_by_earlier_mounts,
        );
        let memory_leaked_noted_by_the_opening = memory_leaked_in_this_process.clone();
        let outcome = outcome_of_opening_a_gpu_within_what_is_left_of_the_limit(
            opening_limit,
            opening_limit.saturating_sub(opening_began_at.elapsed()),
            move || {
                let outcome_of_the_start = started_gpu_with_its_cost_measured(
                    start_the_card,
                    cost_at_the_opening,
                    &memory_leaked_noted_by_the_opening,
                );
                // 交回结局之前先从表里划掉：等它的那一头收到结局时，表里已没有这一次。
                drop(opening_under_way);
                outcome_of_the_start
            },
        );
        // 收到了结局的，这一次在表里已经划掉、下面这一步什么都不动；到点没收到的，那条线程还挂着，记成没回来。
        openings_not_returned_in_this_process
            .note_the_opening_not_returned_past_the_limit_of_its_caller(opening_number);
        outcome
    }
}

/// 这个进程里各张卡上此刻在开的那一次开卡（整个进程一张卡上同时至多一次）：开它的那一方还在自己的期限之内等，就算在开，同一张卡上后到的一方
/// 在自己的期限之内排队等它开完再开（等的时间算进后到那一方自己的期限，等到点照它自己的到点办，[`GpuFailureCause::WaitForTheGpuExceededTheLimit`]）；
/// 开它的那一方到点不等了、那条线程还挂在驱动里，就记成没回来，这张卡上不再起，直到那条线程回来（[`GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned`]）。
/// 进程开卡器持有一份，各次挂载与挂着切换开卡共用（[`opening_of_a_card_within_the_limit`]）。几份把手共用同一张表。
#[derive(Clone, Debug)]
pub struct OpeningsNotReturnedInThisProcess {
    openings_on_the_devices: Arc<(Mutex<OpeningsUnderWayInThisProcess>, Condvar)>,
}

/// 那一张表里的东西：每一次在开的开卡一条（带它的号，号在这个进程里不重复），与此刻有几次开卡在排队等同一张卡上前一次开完。
#[derive(Debug)]
struct OpeningsUnderWayInThisProcess {
    openings: Vec<OpeningUnderWayInThisProcess>,
    openings_numbered_so_far: u64,
    openings_waiting_for_an_earlier_one: usize,
}

/// 一张卡上在开的一次开卡。
#[derive(Debug)]
struct OpeningUnderWayInThisProcess {
    device: GpuDeviceIdentifier,
    opening_number: u64,
    state: StateOfAnOpeningUnderWay,
}

/// 一次在开的开卡此刻怎样了。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StateOfAnOpeningUnderWay {
    /// 开它的那一方还在自己的期限之内等：同一张卡上后到的一方等它开完再开。
    WithinTheLimitOfItsCaller,
    /// 开它的那一方到点不等了，那条线程还挂在驱动里：同一张卡上不再起，直到它回来。
    NotReturnedPastTheLimitOfItsCaller,
}

/// 同一张卡上前一次开卡挡着、这一次没起的两种。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EarlierOpeningOfTheDeviceInTheWay {
    /// 前一次到点没回来：不等，算这一次没开成（[`GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned`]）。
    NotReturnedPastTheLimitOfItsCaller,
    /// 前一次还在它那一方的期限之内开着，这一次等满了自己的期限：照这一次到点办。
    StillUnderWayAtTheLimitOfThisOpening,
}

impl OpeningsNotReturnedInThisProcess {
    /// 这个进程里还没有一次开卡在开、没有一次挂住。
    #[must_use]
    pub fn with_none_not_returned() -> Self {
        Self {
            openings_on_the_devices: Arc::new((
                Mutex::new(OpeningsUnderWayInThisProcess {
                    openings: Vec::new(),
                    openings_numbered_so_far: 0,
                    openings_waiting_for_an_earlier_one: 0,
                }),
                Condvar::new(),
            )),
        }
    }

    /// 两份把手是不是同一张表（进程开卡器交给各次开卡的是同一张）。
    #[must_use]
    pub fn is_the_same_table_as(&self, other: &Self) -> bool {
        Arc::ptr_eq(
            &self.openings_on_the_devices,
            &other.openings_on_the_devices,
        )
    }

    /// 此刻有一次开卡到点之后还没回来的卡（还在期限之内开着的不列）。
    #[must_use]
    pub fn devices_with_an_opening_not_returned(&self) -> Vec<GpuDeviceIdentifier> {
        self.openings_locked()
            .openings
            .iter()
            .filter(|opening| {
                opening.state == StateOfAnOpeningUnderWay::NotReturnedPastTheLimitOfItsCaller
            })
            .map(|opening| opening.device.clone())
            .collect()
    }

    /// 此刻有几次开卡在排队等同一张卡上前一次开完（开它的那一方还在期限之内）。
    #[must_use]
    pub fn openings_waiting_for_an_earlier_one(&self) -> usize {
        self.openings_locked().openings_waiting_for_an_earlier_one
    }

    fn openings_locked(&self) -> std::sync::MutexGuard<'_, OpeningsUnderWayInThisProcess> {
        self.openings_on_the_devices
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// 第 `opening_number` 次开卡的那一方到点不等了：它还在表里（那条线程还没回来）就记成没回来，叫醒排队等这张卡的；已经回来、划掉了的不动。
    fn note_the_opening_not_returned_past_the_limit_of_its_caller(&self, opening_number: u64) {
        let mut openings = self.openings_locked();
        if let Some(opening) = openings
            .openings
            .iter_mut()
            .find(|opening| opening.opening_number == opening_number)
        {
            opening.state = StateOfAnOpeningUnderWay::NotReturnedPastTheLimitOfItsCaller;
            self.openings_on_the_devices.1.notify_all();
        }
    }
}

/// 一张卡上在开的一次开卡：在进程的那一张表里记着，丢它（那一次开卡跑完、开卡里 panic、或起不了那条线程）时划掉、叫醒排队等这张卡的。
struct OpeningUnderWayOnTheDevice {
    openings_in_this_process: OpeningsNotReturnedInThisProcess,
    opening_number: u64,
}

impl OpeningUnderWayOnTheDevice {
    /// 这张卡上没有在开的开卡，就记下这一次、交回它。有一次在开而开它的那一方还在期限之内，就排队等它开完（表里划掉）再记，
    /// 等到这一次自己的期限（`opening_began_at` 起 `opening_limit`）还没等到就不记、交回还在开；前一次已记成没回来的不等、交回没回来。
    fn noted_once_no_earlier_one_is_under_way(
        openings_in_this_process: &OpeningsNotReturnedInThisProcess,
        device: &GpuDeviceIdentifier,
        opening_began_at: Instant,
        opening_limit: Duration,
    ) -> Result<Self, EarlierOpeningOfTheDeviceInTheWay> {
        let (_openings, an_opening_ended_or_was_given_up) =
            &*openings_in_this_process.openings_on_the_devices;
        let mut openings = openings_in_this_process.openings_locked();
        let mut is_counted_as_waiting = false;
        let noted = loop {
            let state_of_the_earlier_one = openings
                .openings
                .iter()
                .find(|opening| opening.device == *device)
                .map(|opening| opening.state);
            match state_of_the_earlier_one {
                None => {
                    openings.openings_numbered_so_far += 1;
                    let opening_number = openings.openings_numbered_so_far;
                    openings.openings.push(OpeningUnderWayInThisProcess {
                        device: device.clone(),
                        opening_number,
                        state: StateOfAnOpeningUnderWay::WithinTheLimitOfItsCaller,
                    });
                    break Ok(opening_number);
                }
                Some(StateOfAnOpeningUnderWay::NotReturnedPastTheLimitOfItsCaller) => {
                    break Err(
                        EarlierOpeningOfTheDeviceInTheWay::NotReturnedPastTheLimitOfItsCaller,
                    );
                }
                Some(StateOfAnOpeningUnderWay::WithinTheLimitOfItsCaller) => {
                    let time_left = opening_limit.saturating_sub(opening_began_at.elapsed());
                    if time_left.is_zero() {
                        break Err(
                            EarlierOpeningOfTheDeviceInTheWay::StillUnderWayAtTheLimitOfThisOpening,
                        );
                    }
                    if !is_counted_as_waiting {
                        openings.openings_waiting_for_an_earlier_one += 1;
                        is_counted_as_waiting = true;
                    }
                    openings = an_opening_ended_or_was_given_up
                        .wait_timeout(openings, time_left)
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .0;
                }
            }
        };
        if is_counted_as_waiting {
            openings.openings_waiting_for_an_earlier_one -= 1;
        }
        noted.map(|opening_number| Self {
            openings_in_this_process: openings_in_this_process.clone(),
            opening_number,
        })
    }
}

impl Drop for OpeningUnderWayOnTheDevice {
    fn drop(&mut self) {
        let mut openings = self.openings_in_this_process.openings_locked();
        openings
            .openings
            .retain(|opening| opening.opening_number != self.opening_number);
        self.openings_in_this_process
            .openings_on_the_devices
            .1
            .notify_all();
    }
}

/// 同一张卡上前一次开卡到点没回来、这一次不起时报的原因（算这一次没开成，照重开的间隔往下数；不记进连着开不成的次数）。
fn opening_not_started_while_an_earlier_one_has_not_returned() -> GpuFailureCause {
    GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned
}

/// 开一次卡（`open`），开卡里 panic 了就接住、算这一次没开成（[`GpuFailureCause::OpeningPanicked`]，带着 panic 的原话），不往上传：
/// 挂载那一刻与切换开卡线程都照重开的规矩往下走，不在两批之间持着会话锁时再 panic（挂载与切换共用的开一张卡那一步
/// `gpus_of_this_host::gpu_opened_under_its_memory_limit` 包着它）。
pub(crate) fn opening_with_its_panic_caught<Opened>(
    open: impl FnOnce() -> Result<Opened, GpuFailureCause>,
) -> Result<Opened, GpuFailureCause> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(open)).unwrap_or_else(|payload| {
        let said_by_the_panic = payload
            .downcast_ref::<&str>()
            .map(|said| (*said).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "（panic 带的不是字符串）".to_string());
        Err(GpuFailureCause::OpeningPanicked { said_by_the_panic })
    })
}

/// 切换开卡叫停之后（卸载、切到 `cpu`）不再开的那几次报的原因：这一次开卡的结局没人收，原因不报到哪里。
fn opening_called_off() -> GpuFailureCause {
    GpuFailureCause::OpeningCalledOff
}

/// 一次挂载里每张卡切换开卡连着开不成了几次（第 4 条：一张卡至多开 [`ReopeningsOfAGpuNotOpened::openings_at_most`] 次，跨切换接着数、卸载重挂清零）：
/// 外壳与切换开卡线程共用一份，开不成一次就记一次（叫停的那几次没开、前一次开卡没回来而这一次没起的，都不记，[`opening_counts_toward_the_failed_openings`]），
/// 开成一次就把这张卡的清零；起不了开卡线程那一回记在头一张要开的卡上。
#[derive(Clone, Debug, Default)]
struct FailedOpeningsOfEachGpuInThisMount {
    failed_openings_of_each_device: Arc<Mutex<Vec<(GpuDeviceIdentifier, u32)>>>,
}

impl FailedOpeningsOfEachGpuInThisMount {
    /// 这张卡这一次挂载里到此刻连着开不成了几次。
    fn failed_openings_of(&self, device: &GpuDeviceIdentifier) -> u32 {
        self.failed_openings_of_each_device
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .find(|(counted_device, _failed_openings)| counted_device == device)
            .map_or(0, |(_device, failed_openings)| *failed_openings)
    }

    /// 这张卡这一次挂载里还能开几次：一张卡至多开 [`ReopeningsOfAGpuNotOpened::openings_at_most`] 次，扣掉它此刻连着开不成的；0 是试满了。
    /// 交设置那一刻判试满（[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]）与切换开卡线程开一张卡之前算这一次至多开几次
    /// （[`gpus_opened_at_a_switch`]）都读它。
    fn openings_left_in_this_mount_of(
        &self,
        device: &GpuDeviceIdentifier,
        reopenings: ReopeningsOfAGpuNotOpened,
    ) -> u32 {
        reopenings
            .openings_at_most()
            .saturating_sub(self.failed_openings_of(device))
    }

    /// 记下这张卡又开不成一次。
    fn note_a_failed_opening(&self, device: &GpuDeviceIdentifier) {
        let mut failed_openings_of_each_device = self
            .failed_openings_of_each_device
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match failed_openings_of_each_device
            .iter_mut()
            .find(|(counted_device, _failed_openings)| counted_device == device)
        {
            Some((_device, failed_openings)) => *failed_openings += 1,
            None => failed_openings_of_each_device.push((device.clone(), 1)),
        }
    }

    /// 这张卡开成了：它连着开不成的次数清零，别的卡的不动。
    fn forget_the_failed_openings_of(&self, device: &GpuDeviceIdentifier) {
        self.failed_openings_of_each_device
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|(counted_device, _failed_openings)| counted_device != device);
    }

    /// 照切换开卡线程开一次的结局记这张卡：开成清零；开不成照 [`opening_counts_toward_the_failed_openings`] 记一次或不记。
    fn note_the_outcome_of_an_opening<Opened>(
        &self,
        device: &GpuDeviceIdentifier,
        outcome: &Result<Opened, GpuFailureCause>,
    ) {
        match outcome {
            Ok(_opened) => self.forget_the_failed_openings_of(device),
            Err(cause) => {
                if opening_counts_toward_the_failed_openings(cause) {
                    self.note_a_failed_opening(device);
                }
            }
        }
    }
}

/// 这一次开不成算不算进一张卡连着开不成的次数：叫停的那一次没开，不算；前一次开卡到点没回来、这一次没起的，不算（出路是重启进程，
/// 不是卸载重挂：不把这张卡判成试满，之后的切换照常为它起开卡，照旧报没回来）；别的都算。
fn opening_counts_toward_the_failed_openings(cause: &GpuFailureCause) -> bool {
    match cause {
        GpuFailureCause::OpeningCalledOff
        | GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned => false,
        GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
        | GpuFailureCause::OpeningThreadNotStarted { .. }
        | GpuFailureCause::OpeningPanicked { .. }
        | GpuFailureCause::OpeningsInThisMountUsedUp { .. }
        | GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { .. }
        | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
        | GpuFailureCause::WaitForTheSubmissionFailed { .. }
        | GpuFailureCause::WaitForTheGpuExceededTheLimit { .. }
        | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
        | GpuFailureCause::ReadbackMappingFailed { .. }
        | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
        | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
        | GpuFailureCause::UnusableAfterResubmissions { .. }
        | GpuFailureCause::NotOpenedAfterTheReopenings { .. } => true,
    }
}

/// 切换开卡那条线程上做的事：按给的次序逐张开 `discrete_gpus_to_open`（各带这一组给它的显存上限），开一张同挂载那一刻开卡那一步
/// （[`gpu_opened_under_its_memory_limit`]，开一次交 `opening_of_a_gpu_at_a_switch`、等 GPU 的上限是这一组的 `wait_limit`；开卡里 panic 了在那一步就接住、
/// 算这一次没开成），开不成就照 `reopenings` 的间隔重开、每次重开之前在这条线程上睡一个间隔（[`gpu_opened_with_the_reopenings`]）。
/// 一张卡这一次至多开它在这一次挂载里剩下的次数（[`FailedOpeningsOfEachGpuInThisMount::openings_left_in_this_mount_of`]，与交设置那一刻判试满是同一个判据）；
/// 开到它这一刻已一次不剩（交设置那一刻还剩，之后前一条线程叫停时正在开的那一次开完、记进了次数）就不开，当场记成它开不成（[`GpuFailureCause::OpeningsInThisMountUsedUp`]，
/// 排在起线程之前的原因后面），后面的不再开。每开不成一次记进 `failed_openings_in_this_mount`、开成一次把它清零（不记的几种照 [`opening_counts_toward_the_failed_openings`]）。
/// 有一张重开到上限都没开成就停在它，后面的不再开。
/// 装不下最小的一片的照开、不量代价（换上那一组时照这张卡的账与开着的卡同一判，再判一次）。`causes_of_each_start` 是起这条线程之前起不了线程的那几次
/// （各算头一张卡一次没开成、已记进它连着开不成的次数）：原因排在头一张卡的原因前面。
/// 每一次开卡之前看 `called_off`：置上了就不再开（卸载、切到 `cpu`），也不再睡重开的间隔。
/// 判一张卡开不成（重开到上限都没开成、或开到它这一刻已试满）那一刻照 `pci_state_reader` 读一次它的 PCI 状态，带进开不成的那一份；
/// 每一次没开成而还要重开时不读，`called_off` 置上了（结局没人收）也不读（[`pci_state_of_the_gpu_not_opened_unless_called_off`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：开一张卡那一步、要开的卡、等的上限、重开的规矩、起线程之前的原因、叫停、泄漏账、连着开不成的次数与读 PCI 状态的那一份"
)]
fn gpus_opened_at_a_switch<Card>(
    opening_of_a_gpu_at_a_switch: &mut OpeningOfAGpuAtASwitch<Card>,
    discrete_gpus_to_open: Vec<(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)>,
    wait_limit: Duration,
    reopenings: ReopeningsOfAGpuNotOpened,
    causes_of_each_start: Vec<GpuFailureCause>,
    called_off: &AtomicBool,
    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
    failed_openings_in_this_mount: &FailedOpeningsOfEachGpuInThisMount,
    pci_state_reader: &PciStateReaderOfTheGpus,
) -> OutcomeOfTheOpeningAtASwitch<Card> {
    let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
    let mut opened = Vec::with_capacity(discrete_gpus_to_open.len());
    let mut causes_before_the_thread = causes_of_each_start;
    for (discrete_gpu, memory_limit) in discrete_gpus_to_open {
        let causes_before_this_card = std::mem::take(&mut causes_before_the_thread);
        let openings_left_in_this_mount = failed_openings_in_this_mount
            .openings_left_in_this_mount_of(&discrete_gpu.identifier, reopenings);
        // 交设置那一刻没试满、这一刻试满了：前一条切换开卡线程叫停时正在开的那一次（叫停只拦之后的几次）在这条线程起来之后才开完、记进了次数。
        // 照交设置那一刻同一个判据当场记成开不成，不再开。
        if openings_left_in_this_mount == 0 {
            let mut causes_of_each_opening = causes_before_this_card;
            causes_of_each_opening.push(openings_in_this_mount_used_up(
                failed_openings_in_this_mount,
                &discrete_gpu.identifier,
            ));
            return OutcomeOfTheOpeningAtASwitch {
                opened,
                not_opened: Some(GpuNotOpenedAfterTheReopenings {
                    pci_state_when_judged_unusable:
                        pci_state_of_the_gpu_not_opened_unless_called_off(
                            pci_state_reader,
                            called_off,
                            &discrete_gpu.identifier,
                        ),
                    device: discrete_gpu.identifier,
                    causes_of_each_opening,
                }),
            };
        }
        let reopenings_of_this_card = ReopeningsOfAGpuNotOpened {
            interval: reopenings.interval,
            reopenings_at_most: openings_left_in_this_mount.saturating_sub(1),
        };
        let opened_with_the_reopenings = gpu_opened_with_the_reopenings(
            reopenings_of_this_card,
            |interval| {
                if !called_off.load(Ordering::SeqCst) {
                    std::thread::sleep(interval);
                }
            },
            || {
                let opened_once = if called_off.load(Ordering::SeqCst) {
                    Err(opening_called_off())
                } else {
                    gpu_opened_under_its_memory_limit(
                        &discrete_gpu,
                        memory_limit,
                        smallest_piece_bytes,
                        memory_leaked_in_this_process,
                        |gpu_to_open, memory_limit_at_the_opening, cost_at_the_opening| {
                            opening_of_a_gpu_at_a_switch(
                                gpu_to_open,
                                memory_limit_at_the_opening,
                                wait_limit,
                                cost_at_the_opening,
                            )
                        },
                    )
                };
                failed_openings_in_this_mount
                    .note_the_outcome_of_an_opening(&discrete_gpu.identifier, &opened_once);
                opened_once
            },
        );
        match opened_with_the_reopenings {
            Ok((card, _left_out_judged_again_when_the_settings_are_put_in_effect)) => {
                opened.push(card);
            }
            Err(causes_in_the_thread) => {
                let mut causes_of_each_opening = causes_before_this_card;
                causes_of_each_opening.extend(causes_in_the_thread);
                return OutcomeOfTheOpeningAtASwitch {
                    opened,
                    not_opened: Some(GpuNotOpenedAfterTheReopenings {
                        pci_state_when_judged_unusable:
                            pci_state_of_the_gpu_not_opened_unless_called_off(
                                pci_state_reader,
                                called_off,
                                &discrete_gpu.identifier,
                            ),
                        device: discrete_gpu.identifier,
                        causes_of_each_opening,
                    }),
                };
            }
        }
    }
    OutcomeOfTheOpeningAtASwitch {
        opened,
        not_opened: None,
    }
}

/// 切换开卡线程判 `device` 开不成、交回之前读它的 PCI 状态：`called_off` 置上了（卸载、切到 `cpu`；叫停之后余下的几次没开，开不成的结局送回去也没人收）
/// 就不读、交回没读；没叫停（重开确实用完、或开到它这一刻已试满，结局后来被更新的交设置盖掉的也算）照读一次。
fn pci_state_of_the_gpu_not_opened_unless_called_off(
    pci_state_reader: &PciStateReaderOfTheGpus,
    called_off: &AtomicBool,
    device: &GpuDeviceIdentifier,
) -> PciStateOfTheGpuReadWhenJudgedUnusable {
    if called_off.load(Ordering::SeqCst) {
        return PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError;
    }
    pci_state_reader.pci_state_of_the_gpu(device)
}

/// 一张卡在这一次挂载里已试满、不再开时报的原因（带它连着开不成的次数）。
fn openings_in_this_mount_used_up(
    failed_openings_in_this_mount: &FailedOpeningsOfEachGpuInThisMount,
    device: &GpuDeviceIdentifier,
) -> GpuFailureCause {
    GpuFailureCause::OpeningsInThisMountUsedUp {
        failed_openings: failed_openings_in_this_mount.failed_openings_of(device),
    }
}

/// 起一张卡（`start_the_card`）、照开卡那一组给它的界量一次提交的代价（`cost_at_the_opening`）。量代价出错时这张卡照
/// [`failure_of_the_measurement_dropping_or_leaking_the_card`] 处置：有提交还在跑就泄漏、这一次占着的字节记进 `memory_leaked_in_this_process`。
///
/// # Errors
/// 起不了这张卡（`start_the_card` 交回的，原样）、量代价出错。
fn started_gpu_with_its_cost_measured<Card: CardOfTheBackgroundGpu>(
    start_the_card: impl FnOnce() -> Result<Card, GpuFailureCause>,
    cost_at_the_opening: CostAtTheOpening,
    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
) -> Result<Card, GpuFailureCause> {
    let mut card = start_the_card()?;
    match cost_at_the_opening {
        CostAtTheOpening::MeasuredUnderTheMemoryLimit => {
            if let Err(failure_of_the_measurement) = card.measure_the_cost_of_one_submission() {
                return Err(failure_of_the_measurement_dropping_or_leaking_the_card(
                    card,
                    failure_of_the_measurement,
                    memory_leaked_in_this_process,
                ));
            }
        }
        CostAtTheOpening::NotMeasuredForTheGpuIsLeftOut => {}
    }
    Ok(card)
}

/// 同一次枚举（`descriptions`）交成卡清单（给了 `background_gpu_devices` 时只判点名的卡）、照它判挂载那一刻的校验、取参与的卡：判与取读同一份清单。
/// 核心层拒成「点名的卡不在卡清单里」时，拒绝带上这一次枚举到的每个适配器（类型与总线地址）与没点名、没收进清单的那几张
/// （[`BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration`]）；核心层别的拒照原样交回。
///
/// # Errors
/// [`BackgroundGpuOpenRefused`] 的 `DiscreteGpuListNotSupported`、`NamedGpuDeviceNotOnTheListOfThisEnumeration` 与 `SettingsRefused`。
fn discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
    selection: &BackgroundGpuSelection,
    descriptions: &[EnumeratedAdapterDescription],
) -> Result<(DiscreteGpusOfTheEnumeration, Vec<ParticipatingGpu>), BackgroundGpuOpenRefused> {
    let discrete_gpus = discrete_gpus_of_the_enumeration_under_the_named(
        descriptions,
        selection.background_gpu_devices.as_deref(),
    )
    .map_err(BackgroundGpuOpenRefused::DiscreteGpuListNotSupported)?;
    match participating_gpus_admitted_by_the_mount_checks(selection, &discrete_gpus.listed) {
        Ok(participating) => Ok((discrete_gpus, participating)),
        Err(BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory { device }) => {
            Err(BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(
                NamedGpuDeviceNotOnTheListOfThisEnumeration {
                    device,
                    adapters_in_the_enumeration: descriptions
                        .iter()
                        .map(AdapterSeenInTheEnumeration::of_the_description)
                        .collect(),
                    unnamed_left_off_the_list: discrete_gpus.unnamed_left_off_the_list,
                },
            ))
        }
        Err(
            refused @ (BackgroundComputeSettingsRefused::GpuNotPresent
            | BackgroundComputeSettingsRefused::MemoryLimitCountDifferentFromTheParticipatingDevices {
                ..
            }
            | BackgroundComputeSettingsRefused::MemoryLimitAboveTheTotalMemoryOfTheDevice { .. }
            | BackgroundComputeSettingsRefused::EmptyGpuDeviceListWhoseMeaningIsUndecided
            | BackgroundComputeSettingsRefused::GpuDeviceNamedMoreThanOnceWhoseMeaningIsUndecided {
                ..
            }
            | BackgroundComputeSettingsRefused::GpuWaitLimitOfZero
            | BackgroundComputeSettingsRefused::GcCpuCompensationThresholdAboveTheWholeCapacity {
                ..
            }),
        ) => Err(BackgroundGpuOpenRefused::SettingsRefused(refused)),
    }
}

/// 挂着切换那一刻现查本机在的独显（[`DiscreteGpusPresentNow`]）：拿开卡用的那一份进程级 Vulkan 实例再枚举一次，在开卡时那个等的上限之内等。
struct AdaptersEnumeratedByTheInstance {
    vulkan_instance: VulkanInstanceOfThisProcess,
    wait_limit: Duration,
}

impl DiscreteGpusPresentNow for AdaptersEnumeratedByTheInstance {
    fn identifiers_of_the_discrete_gpus_present_now(&self) -> Vec<GpuDeviceIdentifier> {
        let Ok(adapters) = self
            .vulkan_instance
            .vulkan_adapters_enumerated_within(self.wait_limit)
        else {
            // 枚举不成（等 wgpu 超过上限）：一张都不算在，交回空。切换拒成哪个成员由核心层按点没点名判：
            // 点了名的拒成 `NamedGpuDeviceNotInTheInventory`，没点名的拒成 `GpuNotPresent`。
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

/// 在一张卡上起设备、编六条管线，不量代价（量代价在 [`started_gpu_with_its_cost_measured`]）。账从这个进程里之前泄漏在它上面的
/// `bytes_leaked_by_earlier_mounts` 起。
///
/// # Errors
/// 起不了设备、编不了管线。
fn started_gpu_in_use(
    adapter: &wgpu::Adapter,
    discrete_gpu: DiscreteGpuOnThisHost,
    memory_limit: Option<GpuMemoryInBytes>,
    wait_limit: Duration,
    bytes_leaked_by_earlier_mounts: GpuMemoryInBytes,
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
    let largest_entry_count_per_dispatch = EntryCountOfAChunk(
        u64::from(adapter_limits.max_compute_workgroups_per_dimension)
            * u64::from(
                INVOCATIONS_PER_WORKGROUP
                    .min(REGION_RECORD_INVOCATIONS_PER_WORKGROUP)
                    .min(LIVE_SLOT_INVOCATIONS_PER_WORKGROUP),
            ),
    );
    let budget = GpuBudgetUnderTheSettings::with_the_bytes_leaked_by_earlier_mounts(
        GpuMemoryInBytes(
            adapter_limits
                .max_storage_buffer_binding_size
                .min(adapter_limits.max_buffer_size)
                .min(LARGEST_STORAGE_BINDING_THE_KERNELS_ADDRESS_IN_BYTES.0),
        ),
        largest_entry_count_per_dispatch,
        memory_limit,
        wait_limit,
        bytes_leaked_by_earlier_mounts,
    );
    Ok(GpuInUse {
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
        panic_in_a_wait_of_the_device: OnceLock::new(),
        measured_cost: None,
    })
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

/// 主机上要装进一个缓冲的内容有多长（对齐前）。
fn bytes_in_a_buffer(bytes: &[u8]) -> InputLengthOnTheHostInBytes {
    InputLengthOnTheHostInBytes::of_this_many_bytes(bytes.len())
}

/// 装着 `contents` 建的缓冲在卡上占的字节数：同 [`buffer_with_contents_created_within_the_scope_of_the_caller`] 的建法，按整字向上补、至少一个整字。
fn bytes_on_the_gpu_of_a_buffer_with(contents: &[u8]) -> GpuMemoryInBytes {
    input_buffer_bytes(bytes_in_a_buffer(contents))
}

/// 看一次这张卡上的这次提交在 GPU 上做没做完，最多等 `time_left`（wgpu 自己带限时的 `poll`）。`poll` 里的 panic 接住、记进
/// `panic_in_a_wait_of_the_device`，之后不再调它（[`completion_check_of_one_poll_with_its_panic_caught`]）。本包里进 wgpu 等的只有这一处
/// （等一批的提交、冲掉待写的空提交、交活与重交之前等前一次还在跑的提交都走它）；回读映射的回调由这一处的 `poll` 推着送到，等映射只收通道、不进 wgpu。
fn completion_check_of_the_submission(
    device: &wgpu::Device,
    panic_in_a_wait_of_the_device: &OnceLock<String>,
    submission: &wgpu::SubmissionIndex,
    time_left: Duration,
) -> CompletionCheck<()> {
    completion_check_of_one_poll_with_its_panic_caught(panic_in_a_wait_of_the_device, || {
        device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission.clone()),
            timeout: Some(time_left),
        })
    })
}

/// 照一次 `poll_once`（wgpu 的 `Device::poll`）的结局判这次提交做没做完。wgpu 30.0.1 在等待报设备丢了时在 `Device::poll` 里 panic
/// （`wgpu-30.0.1/src/backend/wgpu_core.rs` 的 `handle_error_fatal(err, "Device::poll")`）：这里接住，算卡当场报出错
/// （[`GpuFailureCause::UncapturedErrorOrDeviceLost`]，原因带 panic 的原话），走一批出错那一路（任务停下、报警、照 D26（后台整理与放置回收） 已定项 13 第 4 条读一次 PCI 状态），
/// 不让 panic 穿出去（后台那一趟持着会话锁）。panic 的原话记进 `panic_in_a_wait_of_the_device`；记过的不再调 `poll_once`，当场照记下的报。
/// 那一次提交照「没等到它做完」记在账上（`SubmissionOfTheChunk::after_waiting_for_its_completion`），这张卡交活之前的那一等因此每次都走到这里、当场报出错，
/// 不再交活；关它时它有提交还在跑，整张泄漏、不析构（`gpus_of_this_host::gpus_dropped_leaking_those_still_running`）。
fn completion_check_of_one_poll_with_its_panic_caught(
    panic_in_a_wait_of_the_device: &OnceLock<String>,
    poll_once: impl FnOnce() -> Result<wgpu::PollStatus, wgpu::PollError>,
) -> CompletionCheck<()> {
    if let Some(said_by_the_panic) = panic_in_a_wait_of_the_device.get() {
        return CompletionCheck::Failed(device_lost_by_a_panic_in_the_wait(said_by_the_panic));
    }
    // AssertUnwindSafe：`poll_once` 只借着这张卡的设备与一个提交号，本包自己的状态（账、记下的错、PCI 读数）一样都不在它里面改，panic 时改了一半的
    // 只可能是 wgpu 里这张卡的设备那一份（设备丢了的标记、在跑的提交与回读映射的登记、映射回调送没送到）。接住之后不再碰它：这一次提交照没等到它做完
    // 留在账上，之后这张卡的每一次等都在上面那一步当场交回、不再调 wgpu；交活与量代价之前都先等这次提交，所以也不再在它上面建缓冲、交提交；
    // 卸载与切换关卡时它有提交还在跑，整张泄漏、不析构（`card_leaked_when_dropped`）。
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(poll_once)) {
        Ok(Ok(_poll_status)) => CompletionCheck::Completed(()),
        Ok(Err(wgpu::PollError::Timeout)) => CompletionCheck::NotYetComplete,
        Ok(Err(wrong_index @ wgpu::PollError::WrongSubmissionIndex(..))) => {
            CompletionCheck::Failed(GpuFailureCause::WaitForTheSubmissionFailed {
                reason: format!("{wrong_index}"),
            })
        }
        Err(payload) => {
            let said_by_the_panic = payload
                .downcast_ref::<&str>()
                .map(|said| (*said).to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "（panic 带的不是字符串）".to_string());
            let recorded = panic_in_a_wait_of_the_device.get_or_init(|| said_by_the_panic);
            CompletionCheck::Failed(device_lost_by_a_panic_in_the_wait(recorded))
        }
    }
}

/// 等提交时 `Device::poll` panic 了报的原因：卡当场报出错的那一种，带 panic 的原话。
fn device_lost_by_a_panic_in_the_wait(said_by_the_panic: &str) -> GpuFailureCause {
    GpuFailureCause::UncapturedErrorOrDeviceLost {
        reason: format!("等提交时 wgpu 的 Device::poll panic 了：{said_by_the_panic}"),
    }
}

impl GpuInUse {
    /// 量一次提交的主机 CPU 时间（第 2 条；挂载那一刻，或换设置让开卡时没量的卡参与时）：这条线程的 CPU 钟（不是挂钟）。
    /// 账上不记着还在跑的提交（挂载那一刻刚开的卡没有；换设置时外壳先等它做完，[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]）。
    /// 交的那一段是 [`COST_PROBE_CHUNK`]，照这张卡此刻剩下的界（上限扣掉已占的，含这个进程里之前泄漏在它上面的）切成几片、几段
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
            usize::try_from(COST_PROBE_CHUNK.input_bytes.0)
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
                        smallest_piece.smallest_piece_bytes.0
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
    /// 建得成交回缓冲与回读暂存；建不成（显存不足，第 3 条「卡上此刻空闲的不到这个数」，或等 wgpu 交回那个错误范围超过上限）就一次都不交：
    /// 建不成之前已经建好、解开映射的那几份缓冲连同暂存挂在队列的待写里，要等冲掉待写的那一次提交做完才放，所以先交一次空提交冲掉它们、
    /// 在等的上限之内等它做完，做完才放这一段的账、交回建不成的那个错；等不到就把这一段的账记成还在跑的那次提交、交回等的那一次的错
    /// （`GpuMemoryAccount::settle_the_chunk_whose_buffers_were_not_created`）。
    fn buffers_created_within_the_free_memory<Buffers: AsRef<[wgpu::Buffer]>>(
        &mut self,
        bytes_held_by_this_chunk: GpuMemoryInBytes,
        outcome_bytes: GpuMemoryInBytes,
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
        let failure_creating_the_buffers = match created {
            Ok(buffers_of_the_chunk) => return Ok(buffers_of_the_chunk),
            Err(failure_creating_the_buffers) => failure_creating_the_buffers,
        };
        let flushing_submission = self.queue.submit(std::iter::empty());
        let clock = SteadyClockOfThisProcess::starting_now();
        let deadline = GpuWaitDeadline::starting_now(self.budget.wait_limit(), &clock);
        let device = &self.device;
        let panic_in_a_wait_of_the_device = &self.panic_in_a_wait_of_the_device;
        let waited_for_the_flush =
            wait_for_completion_before_the_deadline(&deadline, &clock, |time_left| {
                completion_check_of_the_submission(
                    device,
                    panic_in_a_wait_of_the_device,
                    &flushing_submission,
                    time_left,
                )
            });
        Err(self
            .budget
            .memory_account_mut()
            .settle_the_chunk_whose_buffers_were_not_created(
                bytes_held_by_this_chunk,
                failure_creating_the_buffers,
                flushing_submission,
                waited_for_the_flush,
            ))
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
        outcome_bytes: GpuMemoryInBytes,
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
                outcome_bytes.0,
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
            completion_check_of_the_submission(
                device,
                &self.panic_in_a_wait_of_the_device,
                &submission,
                time_left,
            )
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
        let panic_in_a_wait_of_the_device = &self.panic_in_a_wait_of_the_device;
        self.budget
            .room_once_the_submission_running_past_its_wait_completed(
                &clock,
                |submission, time_left| {
                    completion_check_of_the_submission(
                        device,
                        panic_in_a_wait_of_the_device,
                        submission,
                        time_left,
                    )
                },
            )
    }

    /// 重交之前调：前一次等超过上限、还在跑的那次提交在此刻设置给的等的上限之内等它做完、放掉它的账。
    fn wait_for_the_timed_out_submission(&mut self) -> Result<(), GpuFailureCause> {
        let clock = SteadyClockOfThisProcess::starting_now();
        let device = &self.device;
        let panic_in_a_wait_of_the_device = &self.panic_in_a_wait_of_the_device;
        self.budget.wait_for_the_submission_running_past_its_wait(
            &clock,
            |submission, time_left| {
                completion_check_of_the_submission(
                    device,
                    panic_in_a_wait_of_the_device,
                    submission,
                    time_left,
                )
            },
        )
    }

    /// 校验和核对的一段（切批切出来的、装得下这张卡的那几片）交给这张卡：两份核在同一次提交里各算一遍，交回每片两份的结果，按段里的次序。
    /// 本包这一段占的缓冲先记进这张卡的账（超上限断言），那次提交做完放掉；没等到它做完的留着（`GpuMemoryAccount::settle_the_chunk_after_its_submission`）；
    /// 建缓冲显存不足的不交，已建的冲掉、做完了才放（[`GpuInUse::buffers_created_within_the_free_memory`]）。
    fn both_kernel_outcomes_of_a_checksum_chunk(
        &mut self,
        units: &[UnitChecksumToVerify<'_, ()>],
    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, GpuFailureCause> {
        assert!(!units.is_empty(), "切批不交空段");
        let packed = packed_checksum_chunk(units);
        let entry_count = packed.entry_count();
        let entry_count_in_words =
            u32::try_from(entry_count).expect("一段的条数不超过一次派发的上限，装得进 u32");
        let entry_count_of_the_chunk = EntryCountOfAChunk(u64::from(entry_count_in_words));
        let outcome_bytes = entry_count_of_the_chunk.bytes_of_the_entries(OUTCOME_BYTES_PER_ENTRY);
        let mut parameter_bytes =
            vec![0_u8; usize::try_from(PARAMETER_BUFFER_BYTES.0).expect("16 字节")];
        parameter_bytes[..4].copy_from_slice(&entry_count_in_words.to_le_bytes());
        let descriptor_bytes = packed.descriptor_bytes();
        let outcome_fill = outcome_buffer_before_the_kernel_writes(entry_count);

        let bytes_held_by_this_chunk = COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
            .bytes_of_the_copies(
                PARAMETER_BUFFER_BYTES
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&packed.input_bytes))
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&descriptor_bytes)),
            )
            .plus(
                COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS.bytes_of_the_copies(outcome_bytes),
            );
        assert_eq!(
            bytes_held_by_this_chunk,
            CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
                InputLengthOnTheHostInBytes::of_this_many_bytes(
                    units
                        .iter()
                        .map(|unit| unit.unit_bytes.len())
                        .sum::<usize>()
                ),
                entry_count_of_the_chunk,
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
            singlefs_replay_prefix::gpu_gc_capacity_proofs::gpu_workgroups_for_entries(
                entry_count_in_words,
                INVOCATIONS_PER_WORKGROUP,
            ),
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
        let record_count_of_the_chunk = EntryCountOfAChunk(u64::from(record_count));
        let outcome_bytes =
            record_count_of_the_chunk.bytes_of_the_entries(REGION_RECORD_OUTCOME_BYTES);
        let parameter_bytes = region_record_parameter_bytes(record_count, mapping_tables);
        let descriptor_bytes = little_endian_bytes_of_the_words(&packed.descriptor_words);
        let entry_bytes = little_endian_bytes_of_the_words(&mapping_tables.entry_words);
        let location_item_bytes =
            little_endian_bytes_of_the_words(&mapping_tables.location_item_words);
        let key_sorted_position_bytes =
            little_endian_bytes_of_the_words(&mapping_tables.key_sorted_entry_positions);
        let outcome_fill =
            region_record_outcome_buffer_before_the_kernel_writes(packed.record_count());

        let bytes_held_by_this_chunk = COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
            .bytes_of_the_copies(
                bytes_on_the_gpu_of_a_buffer_with(&parameter_bytes)
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&packed.input_bytes))
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&descriptor_bytes))
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&entry_bytes))
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&location_item_bytes))
                    .plus(bytes_on_the_gpu_of_a_buffer_with(
                        &key_sorted_position_bytes,
                    )),
            )
            .plus(
                COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS.bytes_of_the_copies(outcome_bytes),
            );
        let total_unit_bytes: usize = pieces
            .iter()
            .map(|piece| piece.bytes().map_or(0, <[u8]>::len))
            .sum();
        assert_eq!(
            bytes_held_by_this_chunk,
            region_record_chunk_footprint(mapping_tables).bytes_held_by_a_chunk(
                InputLengthOnTheHostInBytes::of_this_many_bytes(total_unit_bytes),
                record_count_of_the_chunk,
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
            singlefs_replay_prefix::gpu_gc_capacity_proofs::gpu_workgroups_for_entries(
                record_count,
                REGION_RECORD_INVOCATIONS_PER_WORKGROUP,
            ),
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
        let segment_count_of_the_chunk = EntryCountOfAChunk(u64::from(segment_count));
        let outcome_bytes =
            segment_count_of_the_chunk.bytes_of_the_entries(LIVE_SLOT_OUTCOME_BYTES);
        let parameter_bytes = live_slot_parameter_bytes(segment_count, record_tables, unit_area);
        let segment_bound_bytes = little_endian_bytes_of_the_words(&segment_bound_words(segments));
        let record_bytes = little_endian_bytes_of_the_words(&record_tables.record_words);
        let sorted_position_bytes =
            little_endian_bytes_of_the_words(&record_tables.record_positions_by_device_and_slot);
        let outcome_fill = live_slot_outcome_buffer_before_the_kernel_writes(segments.len());

        let bytes_held_by_this_chunk = COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
            .bytes_of_the_copies(
                bytes_on_the_gpu_of_a_buffer_with(&parameter_bytes)
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&segment_bound_bytes))
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&record_bytes))
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&sorted_position_bytes)),
            )
            .plus(
                COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS.bytes_of_the_copies(outcome_bytes),
            );
        assert_eq!(
            bytes_held_by_this_chunk,
            live_slot_chunk_footprint(record_tables).bytes_held_by_a_chunk(
                segment_count_of_the_chunk.input_bytes_of_the_entries(SEGMENT_BOUND_BYTES),
                segment_count_of_the_chunk,
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
            singlefs_replay_prefix::gpu_gc_capacity_proofs::gpu_workgroups_for_entries(
                segment_count,
                LIVE_SLOT_INVOCATIONS_PER_WORKGROUP,
            ),
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
    /// 挂载那一刻开卡之后：开成的卡（与 `discrete_gpus_after_the_opening.opened` 一一对应、同序）、开卡之后的本机独显（连同没参与而没开的）、没点名而没收进清单的那几张、
    /// 开卡那一组参与的卡与因显存上限不用的卡（`gpus_of_this_host::gpus_opened_at_the_mount` 交回的），进程里泄漏的那一份（丢它时记进去），
    /// 卡清单每次答之前现查本机在的独显的那一样（产品带着开卡用的那一份进程级 Vulkan 实例），切换点到没开的卡时开它一次的那一步，
    /// 开不成时怎么重开（产品是 [`ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT`]），与判一张卡用不了那一刻读它的 PCI 状态的那一份
    /// （产品是 [`PciStateReaderOfTheGpus::of_this_host`]）。测试拿它把替身卡装进来。
    ///
    /// # Panics
    /// 开成的卡与 `discrete_gpus_after_the_opening.opened` 张数不等。
    #[must_use]
    #[allow(
        clippy::too_many_arguments,
        reason = "挂载那一刻开卡之后的各样各是一个参数，测试照它逐样装替身；捆成一个结构体只是把同样几个字段挪个地方"
    )]
    pub fn of_the_cards_opened_at_the_mount(
        cards_on_this_host: Vec<Card>,
        discrete_gpus_after_the_opening: DiscreteGpusAfterTheOpening,
        unnamed_discrete_gpus_left_off_the_list: Vec<DiscreteGpuListRefused>,
        participating: ParticipatingGpusOnThisHost,
        left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
        memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
        discrete_gpus_present_now: Box<dyn DiscreteGpusPresentNow + Send>,
        opening_of_a_gpu_at_a_switch: OpeningOfAGpuAtASwitch<Card>,
        reopenings_of_a_gpu_not_opened: ReopeningsOfAGpuNotOpened,
        pci_state_reader: PciStateReaderOfTheGpus,
    ) -> Self {
        assert_eq!(
            cards_on_this_host.len(),
            discrete_gpus_after_the_opening.opened.len(),
            "开成的卡与开卡之后的独显清单一一对应"
        );
        let devices_named_by_the_settings_in_effect = discrete_gpus_after_the_opening
            .opened
            .iter()
            .map(|discrete_gpu| discrete_gpu.identifier.clone())
            .collect();
        Self {
            cards_on_this_host,
            discrete_gpus_after_the_opening,
            unnamed_discrete_gpus_left_off_the_list,
            participating,
            left_out_under_their_memory_limit,
            not_opened_after_the_reopenings_at_the_last_switch: None,
            cost_not_measured_at_the_last_switch: None,
            opening_of_a_gpu_at_a_switch: Arc::new(Mutex::new(opening_of_a_gpu_at_a_switch)),
            reopenings_of_a_gpu_not_opened,
            latest_handing_in: HandingInNumber(0),
            devices_named_by_the_settings_in_effect,
            settings_waiting_for_the_opening: None,
            opening_at_a_switch: None,
            start_of_the_opening_thread: StartOfTheOpeningThread::ByTheStandardLibrary,
            failed_openings_in_this_mount: FailedOpeningsOfEachGpuInThisMount::default(),
            gpus_leaked_when_closed_at_a_switch: GpusLeakedWhenClosedAtASwitch::with_none_leaked(),
            submissions_since_the_last_take: GpuSubmissionsOfTheBatch::default(),
            gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped::not_yet_dropped(),
            memory_leaked_in_this_process,
            discrete_gpus_present_now,
            pci_state_reader,
        }
    }

    /// 此刻开着的卡（挂载那一刻开的参与的卡按总线地址排，为切换开的依次排在后面；换上一组时不再点名的已关掉），参与与否都列。
    #[must_use]
    pub fn gpus_opened(&self) -> Vec<GpuDeviceIdentifier> {
        self.cards_on_this_host
            .iter()
            .map(|card| card.discrete_gpu().identifier.clone())
            .collect()
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

    /// 挂着切换关卡时泄漏了哪几张卡（有提交还在跑的那张整张泄漏、不析构，同卸载的规矩），按关的先后累计：调用方拿着这一份随时读
    /// （[`GpusLeakedWhenClosedAtASwitch`]，`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
    #[must_use]
    pub fn gpus_leaked_when_closed_at_a_switch(&self) -> GpusLeakedWhenClosedAtASwitch {
        self.gpus_leaked_when_closed_at_a_switch.clone()
    }

    /// 这一次挂载里判卡用不了时一共读了几次 PCI 状态（切换开卡线程上读的也算；`.claude/rules/fs-design.md` 五条硬要求第 4 条：看得出读了没有）。
    #[must_use]
    pub fn pci_state_readings_taken(&self) -> u64 {
        self.pci_state_reader.readings_taken()
    }

    /// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：接下来 `refusals` 次起切换开卡那条线程都照起不了报，强制进入「起不了开卡线程」那一支。
    pub fn refuse_to_start_the_opening_thread_for_tests(&mut self, refusals: u32) {
        self.start_of_the_opening_thread = StartOfTheOpeningThread::RefusedForTests {
            refusals_left: refusals,
        };
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
    /// 每张卡的显存上限与等的上限换成它给的。开着的卡不重开，各卡的账照旧记着。这一次交来记一个新的次序号。
    /// 它点的卡都开着，就当场换上（[`Self::put_the_settings_in_effect`]：不再点名的卡关掉）。它点到没开的卡（挂载时没开的，或之前切换、切到 `cpu` 关掉的），
    /// 照第 4 条「切换请求照收，在会话锁外开新卡」：这一刻不开卡，在另一条线程上按点的次序逐张开（开不成就隔一段时间重开，
    /// [`Self::of_the_cards_opened_at_the_mount`] 给的重开规矩），开着的这段时间照旧照交来之前那一组跑；开完了在之后的两批之间换上
    /// （[`Self::put_the_gpus_opened_since_the_switch_in_effect`]）。它点到的没开的卡里有一张在这一次挂载里已连着试满
    /// （[`ReopeningsOfAGpuNotOpened::openings_at_most`] 次都没开成）：不再开，当场记成开不成（[`GpuFailureCause::OpeningsInThisMountUsedUp`]），
    /// 此刻生效那一组之外开着的卡关掉、照旧照它跑。上一次切换的卡还在开时又交来一组，记下最近这一组，等那一次开完再看。
    /// 上一次切换留下的「重开到上限都没开成」在这一刻清掉（下一次换设置解除它）。
    pub fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
        self.latest_handing_in = self.latest_handing_in.next();
        self.not_opened_after_the_reopenings_at_the_last_switch = None;
        let discrete_gpus_to_open = self.discrete_gpus_named_and_not_opened(settings);
        if discrete_gpus_to_open.is_empty() {
            self.settings_waiting_for_the_opening = None;
            self.put_the_settings_in_effect(settings);
            return;
        }
        if let Some(used_up) =
            self.gpu_whose_openings_in_this_mount_are_used_up(&discrete_gpus_to_open)
        {
            self.settings_waiting_for_the_opening = None;
            self.close_the_cards_outside_the_settings_in_effect();
            self.not_opened_after_the_reopenings_at_the_last_switch = Some(used_up);
            return;
        }
        self.settings_waiting_for_the_opening = Some(SettingsWaitingForTheOpening {
            handing_in_number: self.latest_handing_in,
            settings: settings.clone(),
        });
        if self.opening_at_a_switch.is_none() {
            self.start_the_opening_at_a_switch(
                self.latest_handing_in,
                discrete_gpus_to_open,
                settings.wait_limit(),
                Vec::new(),
            );
        }
    }

    /// 从下一批起一张卡都不参与（核心层切到 `cpu` 时交来；第 2 条「从 gpu 切到 cpu……关掉全部开着的卡、丢掉还在开的那一组」）：这一次也记一个新的次序号，
    /// 在会话锁外开着的那一次叫停（开卡线程不再开下一次，开出来的没人收、在那条线程上照常丢），已送回还没取走的开卡结局里开成的卡收下再关，
    /// 还在等开卡的那一组丢掉，开着的卡全部关掉（有提交还在跑的那张整张泄漏，[`Self::close_the_cards`]），「重开到上限都没开成」与「换设置时量代价出错」清掉。
    /// 之后再交来一组，它点的卡都当没开的卡照重开的规矩开。
    pub fn use_no_gpu_from_the_next_batch(&mut self) {
        self.latest_handing_in = self.latest_handing_in.next();
        self.settings_waiting_for_the_opening = None;
        self.not_opened_after_the_reopenings_at_the_last_switch = None;
        self.cost_not_measured_at_the_last_switch = None;
        if let Some(opening) = self.opening_at_a_switch.take() {
            opening.call_off();
            let cards_opened_and_not_taken = match opening {
                OpeningAtASwitch::UnderWay {
                    outcome_receiver, ..
                } => outcome_receiver
                    .try_recv()
                    .map_or_else(|_nothing_sent| Vec::new(), |outcome| outcome.opened),
                OpeningAtASwitch::EndedWithoutAThread { outcome, .. } => outcome.opened,
                OpeningAtASwitch::ThreadNotStarted { .. } => Vec::new(),
            };
            self.take_in_the_cards_opened_at_a_switch(cards_opened_and_not_taken);
        }
        self.devices_named_by_the_settings_in_effect.clear();
        self.left_out_under_their_memory_limit.clear();
        self.close_the_cards(|_device| true);
    }

    /// 两批之间（核心层每一批开头与整理两个入口判停没停之前，经 `BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`）：
    /// 会话锁外为切换开卡的那一次开完了就在这一刻收下它的结局（[`Self::take_in_the_outcome_of_the_opening`]），起不了开卡线程的到了间隔就再起
    /// （那一次之后又交来了别的一组的，为最近那一组起，不为被盖掉的那一组起；最近没有在等开卡的一组就不起）；
    /// 然后交回最近交来的那一组此刻怎样了（[`LatestSettingsHandedIn`]）。`wait` 是 [`WaitForTheOpeningAtASwitch::NotAtAll`] 时还在开就不等，
    /// [`WaitForTheOpeningAtASwitch::UntilItEnds`] 时一直等到为最近交来的那一组开卡开完。
    ///
    /// # Panics
    /// 开卡线程没送结局就断开了：开一次卡里的 panic 在那条线程上就接住（[`opening_with_its_panic_caught`]），断开只剩这个包自己的代码 panic 一种，照原样再 panic。
    pub fn put_the_gpus_opened_since_the_switch_in_effect(
        &mut self,
        wait: WaitForTheOpeningAtASwitch,
    ) -> LatestSettingsHandedIn {
        let mut round: u64 = 0;
        while let Some(opening) = self.opening_at_a_switch.take() {
            round += 1;
            // 照进度打一行（第 1、2、4、8… 轮）：等得久的能从日志看出在第几轮、等的是哪一步，空转的轮数一路翻倍也看得出来
            if round.is_power_of_two() {
                eprintln!(
                    "BACKGROUND_GPU_SWITCH_OPENING_PROGRESS round={round} waiting={} step={}",
                    matches!(wait, WaitForTheOpeningAtASwitch::UntilItEnds),
                    match &opening {
                        OpeningAtASwitch::EndedWithoutAThread { .. } => "ended-without-a-thread",
                        OpeningAtASwitch::ThreadNotStarted { .. } => "thread-not-started",
                        OpeningAtASwitch::UnderWay { .. } => "under-way",
                    }
                );
            }
            match opening {
                OpeningAtASwitch::EndedWithoutAThread {
                    handing_in_number,
                    outcome,
                } => self.take_in_the_outcome_of_the_opening(handing_in_number, outcome),
                OpeningAtASwitch::ThreadNotStarted {
                    handing_in_number,
                    discrete_gpus_to_open,
                    wait_limit,
                    causes_of_each_start,
                    not_started_at,
                } => {
                    let interval_left = self
                        .reopenings_of_a_gpu_not_opened
                        .interval
                        .saturating_sub(not_started_at.elapsed());
                    if !interval_left.is_zero() {
                        match wait {
                            WaitForTheOpeningAtASwitch::NotAtAll => {
                                self.opening_at_a_switch =
                                    Some(OpeningAtASwitch::ThreadNotStarted {
                                        handing_in_number,
                                        discrete_gpus_to_open,
                                        wait_limit,
                                        causes_of_each_start,
                                        not_started_at,
                                    });
                                break;
                            }
                            WaitForTheOpeningAtASwitch::UntilItEnds => {
                                std::thread::sleep(interval_left);
                            }
                        }
                    }
                    let latest_waiting_handing_in_number = self
                        .settings_waiting_for_the_opening
                        .as_ref()
                        .map(|waiting| waiting.handing_in_number);
                    if latest_waiting_handing_in_number == Some(handing_in_number) {
                        self.start_the_opening_at_a_switch(
                            handing_in_number,
                            discrete_gpus_to_open,
                            wait_limit,
                            causes_of_each_start,
                        );
                    } else {
                        self.start_the_opening_for_the_settings_waiting();
                    }
                }
                OpeningAtASwitch::UnderWay {
                    handing_in_number,
                    outcome_receiver,
                    opening_thread,
                    called_off,
                } => {
                    let received = match wait {
                        WaitForTheOpeningAtASwitch::NotAtAll => outcome_receiver.try_recv(),
                        WaitForTheOpeningAtASwitch::UntilItEnds => outcome_receiver
                            .recv()
                            .map_err(|_disconnected| TryRecvError::Disconnected),
                    };
                    match received {
                        Ok(outcome) => {
                            self.take_in_the_outcome_of_the_opening(handing_in_number, outcome);
                        }
                        Err(TryRecvError::Empty) => {
                            self.opening_at_a_switch = Some(OpeningAtASwitch::UnderWay {
                                handing_in_number,
                                outcome_receiver,
                                opening_thread,
                                called_off,
                            });
                            break;
                        }
                        Err(TryRecvError::Disconnected) => {
                            std::panic::resume_unwind(opening_thread.join().expect_err(
                                "开卡线程没送结局就断开只有一种：它 panic 了（正常结束一定送了，送的时候这一头还拿着收的那一端）",
                            ))
                        }
                    }
                }
            }
        }
        self.latest_settings_handed_in()
    }

    /// 最近交来的那一组此刻怎样了：还在等开卡是 [`LatestSettingsHandedIn::StillOpeningItsCards`]；为它开卡重开到上限都没开成是
    /// [`LatestSettingsHandedIn::NotOpened`]；别的是换上了，连同这组设置下一批分不分得到卡上。
    fn latest_settings_handed_in(&self) -> LatestSettingsHandedIn {
        if self.settings_waiting_for_the_opening.is_some() {
            return LatestSettingsHandedIn::StillOpeningItsCards;
        }
        if let Some(not_opened) = &self.not_opened_after_the_reopenings_at_the_last_switch {
            return LatestSettingsHandedIn::NotOpened(not_opened.clone());
        }
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: self.batch_not_split_over_the_gpus(),
        }
    }

    /// 收下切换开卡开成的卡：接到开着的卡末尾，卡清单里从没开的那几张挪到开着的那几张。
    fn take_in_the_cards_opened_at_a_switch(&mut self, cards_opened: Vec<Card>) {
        for card in cards_opened {
            let identifier = card.discrete_gpu().identifier.clone();
            self.discrete_gpus_after_the_opening
                .note_the_gpu_opened_at_a_switch(&identifier);
            assert!(
                !self
                    .discrete_gpus_after_the_opening
                    .not_opened_for_not_taking_part
                    .iter()
                    .any(|discrete_gpu| discrete_gpu.identifier == identifier),
                "切换开成的卡记账之后不该还在没开的那几张里（不然再交同一组会为它一遍遍重开、永远换不上）：{identifier:?}"
            );
            self.cards_on_this_host.push(card);
        }
    }

    /// 收下为第 `handing_in_number` 次交来的设置开卡的结局：开成的卡先收下（[`Self::take_in_the_cards_opened_at_a_switch`]），再照最近交来、在等的那一组办——
    /// 开卡为的就是它（次序号相同）、有一张重开到上限都没开成：此刻生效那一组之外开着的卡全部关掉（连同为更早几次交来的设置开成的），记下那一张，
    /// 照旧照此刻生效那一组跑，直到下一次换设置；它点的卡都开着了：换上它（不再点名的卡关掉）；它还点着没开的卡（开卡为的是更早一次交来的，
    /// 那一次开不成的不报给它）：此刻生效那一组与它都没点的开着的卡当场关掉（为被盖掉的那一组开成的），再为它开一次
    /// （[`Self::start_the_opening_for_the_settings_waiting`]）。没有在等的那一组（之后交来的一组不用开卡、已换上，或告知过一张卡都不参与）：
    /// 此刻生效那一组之外开着的卡全部关掉。
    fn take_in_the_outcome_of_the_opening(
        &mut self,
        handing_in_number: HandingInNumber,
        outcome: OutcomeOfTheOpeningAtASwitch<Card>,
    ) {
        self.take_in_the_cards_opened_at_a_switch(outcome.opened);
        let Some(settings_waiting) = self.settings_waiting_for_the_opening.as_ref() else {
            self.close_the_cards_outside_the_settings_in_effect();
            return;
        };
        if settings_waiting.handing_in_number == handing_in_number {
            if let Some(not_opened) = outcome.not_opened {
                self.settings_waiting_for_the_opening = None;
                self.close_the_cards_outside_the_settings_in_effect();
                self.not_opened_after_the_reopenings_at_the_last_switch = Some(not_opened);
                return;
            }
        }
        let devices_named_by_the_settings_in_effect_or_waiting: Vec<GpuDeviceIdentifier> = self
            .devices_named_by_the_settings_in_effect
            .iter()
            .cloned()
            .chain(
                settings_waiting
                    .settings
                    .participating_devices()
                    .iter()
                    .map(|participating| participating.device.clone()),
            )
            .collect();
        self.close_the_cards(|device| {
            !devices_named_by_the_settings_in_effect_or_waiting.contains(device)
        });
        self.start_the_opening_for_the_settings_waiting();
    }

    /// 为最近交来、在等开卡的那一组（有的话）接着办：它点的卡都开着了就换上；还点着没开的卡，其中有一张这一次挂载里已连着试满就记成开不成
    /// （不再开，此刻生效那一组之外开着的卡关掉），不然为它起开卡（起线程之前没有起不了的原因）。没有在等的那一组就什么都不做。
    /// 起不了开卡线程、到了间隔而之后又交来了别的一组时，也走这一步为最近那一组起。
    fn start_the_opening_for_the_settings_waiting(&mut self) {
        let Some(settings_waiting) = self.settings_waiting_for_the_opening.take() else {
            return;
        };
        let discrete_gpus_to_open =
            self.discrete_gpus_named_and_not_opened(&settings_waiting.settings);
        if discrete_gpus_to_open.is_empty() {
            self.put_the_settings_in_effect(&settings_waiting.settings);
            return;
        }
        if let Some(used_up) =
            self.gpu_whose_openings_in_this_mount_are_used_up(&discrete_gpus_to_open)
        {
            self.close_the_cards_outside_the_settings_in_effect();
            self.not_opened_after_the_reopenings_at_the_last_switch = Some(used_up);
            return;
        }
        let waiting_handing_in_number = settings_waiting.handing_in_number;
        let wait_limit = settings_waiting.settings.wait_limit();
        self.settings_waiting_for_the_opening = Some(settings_waiting);
        self.start_the_opening_at_a_switch(
            waiting_handing_in_number,
            discrete_gpus_to_open,
            wait_limit,
            Vec::new(),
        );
    }

    /// `discrete_gpus_to_open` 里头一张在这一次挂载里已连着试满（开不成的次数到了 [`ReopeningsOfAGpuNotOpened::openings_at_most`]）的卡，
    /// 记成它开不成（[`GpuFailureCause::OpeningsInThisMountUsedUp`]，只一个原因），带判它开不成这一刻读的它的 PCI 状态；都没试满交回 `None`（不读）。
    fn gpu_whose_openings_in_this_mount_are_used_up(
        &self,
        discrete_gpus_to_open: &[(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)],
    ) -> Option<GpuNotOpenedAfterTheReopenings> {
        discrete_gpus_to_open
            .iter()
            .find(|(discrete_gpu, _memory_limit)| {
                self.failed_openings_in_this_mount
                    .openings_left_in_this_mount_of(
                        &discrete_gpu.identifier,
                        self.reopenings_of_a_gpu_not_opened,
                    )
                    == 0
            })
            .map(
                |(discrete_gpu, _memory_limit)| GpuNotOpenedAfterTheReopenings {
                    device: discrete_gpu.identifier.clone(),
                    causes_of_each_opening: vec![openings_in_this_mount_used_up(
                        &self.failed_openings_in_this_mount,
                        &discrete_gpu.identifier,
                    )],
                    pci_state_when_judged_unusable: self
                        .pci_state_reader
                        .pci_state_of_the_gpu(&discrete_gpu.identifier),
                },
            )
    }

    /// 关掉此刻生效那一组没点的开着的卡（[`Self::close_the_cards`]）。
    fn close_the_cards_outside_the_settings_in_effect(&mut self) {
        let devices_named_by_the_settings_in_effect =
            self.devices_named_by_the_settings_in_effect.clone();
        self.close_the_cards(|device| !devices_named_by_the_settings_in_effect.contains(device));
    }

    /// 这一组点到、此刻没开的卡（按点的次序），各带这一组给它的显存上限。
    fn discrete_gpus_named_and_not_opened(
        &self,
        settings: &AdmittedGpuSettings,
    ) -> Vec<(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)> {
        settings
            .participating_devices()
            .iter()
            .filter_map(|participating| {
                self.discrete_gpus_after_the_opening
                    .not_opened_for_not_taking_part
                    .iter()
                    .find(|discrete_gpu| discrete_gpu.identifier == participating.device)
                    .map(|discrete_gpu| (discrete_gpu.clone(), participating.memory_limit))
            })
            .collect()
    }

    /// 起切换开卡那条线程（[`StartOfTheOpeningThread`]）：产品照 std 起；只供测试的那一种照剩下的次数报起不了。
    fn opening_thread_started(
        &mut self,
        body_of_the_thread: impl FnOnce() + Send + 'static,
    ) -> std::io::Result<JoinHandle<()>> {
        match &mut self.start_of_the_opening_thread {
            StartOfTheOpeningThread::ByTheStandardLibrary
            | StartOfTheOpeningThread::RefusedForTests { refusals_left: 0 } => {
                std::thread::Builder::new()
                    .name("singlefs-opening-gpus-at-a-switch".to_string())
                    .spawn(body_of_the_thread)
            }
            StartOfTheOpeningThread::RefusedForTests { refusals_left } => {
                *refusals_left -= 1;
                Err(std::io::Error::other("只供测试：照起不了切换开卡线程报"))
            }
        }
    }

    /// 在另一条线程上为第 `handing_in_number` 次交来的设置开 `discrete_gpus_to_open` 那几张（按给的次序逐张开，开一张同挂载那一刻开卡那一步，
    /// [`gpus_opened_at_a_switch`]；开不成就照重开的规矩重开，每次重开之前在那条线程上睡一个间隔；有一张重开到上限都没开成，后面的就不再开）。
    /// 不等它：调用即返回。`causes_of_each_start` 是之前几次起不了这条线程的原因。起不了那条线程，就算头一张要开的卡这一次没开成
    /// （[`GpuFailureCause::OpeningThreadNotStarted`]，记进它这一次挂载里连着开不成的次数）：它还没试满就隔重开的间隔、在之后的两批之间再起
    /// （[`OpeningAtASwitch::ThreadNotStarted`]），试满了结局就定成头一张开不成。
    fn start_the_opening_at_a_switch(
        &mut self,
        handing_in_number: HandingInNumber,
        discrete_gpus_to_open: Vec<(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)>,
        wait_limit: Duration,
        causes_of_each_start: Vec<GpuFailureCause>,
    ) {
        let opening_of_a_gpu_at_a_switch = Arc::clone(&self.opening_of_a_gpu_at_a_switch);
        let memory_leaked_in_this_process = self.memory_leaked_in_this_process.clone();
        let failed_openings_in_this_mount = self.failed_openings_in_this_mount.clone();
        let pci_state_reader = self.pci_state_reader.clone();
        let reopenings = self.reopenings_of_a_gpu_not_opened;
        let called_off = Arc::new(AtomicBool::new(false));
        let called_off_seen_by_the_thread = Arc::clone(&called_off);
        let discrete_gpus_opened_by_the_thread = discrete_gpus_to_open.clone();
        let causes_of_each_start_seen_by_the_thread = causes_of_each_start.clone();
        let (outcome_sender, outcome_receiver) = channel();
        let started = self.opening_thread_started(move || {
            let mut opening_of_a_gpu = opening_of_a_gpu_at_a_switch.lock().expect(
                "开卡那一步只在开卡线程上锁着用，开一次卡里的 panic 在锁里就接住了：锁没被毒化",
            );
            let outcome = gpus_opened_at_a_switch(
                &mut *opening_of_a_gpu,
                discrete_gpus_opened_by_the_thread,
                wait_limit,
                reopenings,
                causes_of_each_start_seen_by_the_thread,
                &called_off_seen_by_the_thread,
                &memory_leaked_in_this_process,
                &failed_openings_in_this_mount,
                &pci_state_reader,
            );
            drop(opening_of_a_gpu);
            // 送不出去只有一种：后台 GPU 已经丢了（卸载）或切到了 `cpu`，开出来的卡没人要，在这里照常丢。
            let _nobody_takes_this_opening_any_more = outcome_sender.send(outcome);
        });
        let error = match started {
            Ok(opening_thread) => {
                self.opening_at_a_switch = Some(OpeningAtASwitch::UnderWay {
                    handing_in_number,
                    outcome_receiver,
                    opening_thread,
                    called_off,
                });
                return;
            }
            Err(error) => error,
        };
        let first_device_to_open = discrete_gpus_to_open
            .first()
            .map(|(discrete_gpu, _memory_limit)| discrete_gpu.identifier.clone())
            .expect("只为点到没开的卡的那一组开卡：至少一张");
        let mut causes_of_each_start = causes_of_each_start;
        causes_of_each_start.push(GpuFailureCause::OpeningThreadNotStarted {
            reason: format!("起不了切换开卡的那条线程：{error}"),
        });
        self.failed_openings_in_this_mount
            .note_a_failed_opening(&first_device_to_open);
        if self
            .failed_openings_in_this_mount
            .failed_openings_of(&first_device_to_open)
            < reopenings.openings_at_most()
        {
            self.opening_at_a_switch = Some(OpeningAtASwitch::ThreadNotStarted {
                handing_in_number,
                discrete_gpus_to_open,
                wait_limit,
                causes_of_each_start,
                not_started_at: Instant::now(),
            });
            return;
        }
        self.opening_at_a_switch = Some(OpeningAtASwitch::EndedWithoutAThread {
            handing_in_number,
            outcome: OutcomeOfTheOpeningAtASwitch {
                opened: Vec::new(),
                not_opened: Some(GpuNotOpenedAfterTheReopenings {
                    pci_state_when_judged_unusable: self
                        .pci_state_reader
                        .pci_state_of_the_gpu(&first_device_to_open),
                    device: first_device_to_open,
                    causes_of_each_opening: causes_of_each_start,
                }),
            },
        });
    }

    /// 关掉 `is_closed` 判成要关的开着的卡（第 2 条「切换之后不再参与的卡，切换生效时关掉」）：它从开着的那几张挪回没开的那几张（卡清单照列，
    /// 之后再点它就重开），设备丢掉；上面还记着一次等超过上限、没在 GPU 上做完的提交的那张不丢、整张泄漏，这一刻占着的字节记进这个进程的泄漏账
    /// （同卸载，`gpus_of_this_host::gpus_dropped_leaking_those_still_running`），并累计进对外读得到的那一份
    /// （[`Self::gpus_leaked_when_closed_at_a_switch`]）。参与的卡照它们的标识在剩下的卡里重找位置，次序不变。
    fn close_the_cards(&mut self, is_closed: impl Fn(&GpuDeviceIdentifier) -> bool) {
        let participating_devices: Vec<GpuDeviceIdentifier> = self
            .participating
            .positions_on_this_host()
            .iter()
            .map(|position| {
                self.cards_on_this_host[*position]
                    .discrete_gpu()
                    .identifier
                    .clone()
            })
            .collect();
        let mut cards_kept_open = Vec::new();
        let mut cards_closed = Vec::new();
        for card in std::mem::take(&mut self.cards_on_this_host) {
            if is_closed(&card.discrete_gpu().identifier) {
                self.discrete_gpus_after_the_opening
                    .note_the_gpu_closed_at_a_switch(&card.discrete_gpu().identifier);
                cards_closed.push(card);
            } else {
                cards_kept_open.push(card);
            }
        }
        self.cards_on_this_host = cards_kept_open;
        let gpus_leaked = gpus_dropped_leaking_those_still_running(
            cards_closed,
            card_has_a_submission_running_past_its_wait,
            card_leaked_when_dropped,
        );
        self.memory_leaked_in_this_process
            .note_the_gpus_leaked_when_dropped(&gpus_leaked);
        self.gpus_leaked_when_closed_at_a_switch
            .note_the_gpus_leaked(&gpus_leaked);
        self.participating = ParticipatingGpusOnThisHost::of_the_positions(
            participating_devices
                .iter()
                .filter_map(|device| {
                    self.cards_on_this_host
                        .iter()
                        .position(|card| card.discrete_gpu().identifier == *device)
                })
                .collect(),
        );
    }

    /// 换上 `settings`（它点的卡都开着）：不再点名的开着的卡关掉（[`Self::close_the_cards`]），参与的卡换成它点的那几张、按它的次序，
    /// 每张卡的显存上限与等的上限换成它给的（不参与的卡不设显存上限，它分不到活）。
    /// 显存上限（扣掉卡上已占的）连最小的一片都装不下的卡不用、报出来（[`Self::gpus_left_out_under_their_memory_limit`]）；开卡时因显存上限没量代价、
    /// 或上一组设置因显存上限晾下而这一组给了装得下的上限的卡，先等账上还在跑的那次提交做完、放账，再照新上限量一次（量不量照旧、量出的代价不用来分活），
    /// 等不到或量出错的这张卡不参与，之后每一批报成它的 GPU 出错（[`BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch`]，出错的原因是判卡用不了的那一种时
    /// 带出错那一刻读的它的 PCI 状态），直到下一次换设置。
    /// 请回来的卡照参与的次序重新参与分活。
    fn put_the_settings_in_effect(&mut self, settings: &AdmittedGpuSettings) {
        let devices_named_by_the_settings: Vec<GpuDeviceIdentifier> = settings
            .participating_devices()
            .iter()
            .map(|participating| participating.device.clone())
            .collect();
        self.close_the_cards(|device| !devices_named_by_the_settings.contains(device));
        self.devices_named_by_the_settings_in_effect = devices_named_by_the_settings;
        let identifiers_on_this_host = self.gpus_opened();
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
            // 第 3 条「挂着切换给了它装得下的上限，从下一批起照新上限按卡的顺序重新参与分活」：上一组设置因显存上限晾下的卡请回来时照新上限再量一次代价
            // （参与的卡都报得出量出的代价，[`Self::participating_gpus`]），量出的代价不用来分活，量不成照旧不参与、报出来。量之前先在这一组给的等的上限之内等账上还在跑的那次提交（前一次等超过上限、或前一次量代价超时留下的）
            // 做完、放账（同交活之前那一等）：量代价那一段要记账，账上记着还在跑的提交时不交；等不到就照量代价出错办。
            let is_brought_back_after_being_left_out = devices_left_out_under_the_settings_before
                .contains(&card.discrete_gpu().identifier);
            if card.measured_cost().is_none() || is_brought_back_after_being_left_out {
                let measured = card
                    .room_once_the_submission_running_past_its_wait_completed()
                    .and_then(|_room_left| card.measure_the_cost_of_one_submission());
                if let Err(cause) = measured {
                    positions_left_out.push(*position);
                    // 量代价那几次提交出的错与一批里的同样判：卡当场报出错、等超时用完是判它用不了的时刻，当场读一次（D26（后台整理与放置回收） 已定项 13 第 4 条）；
                    // 只报头一张出错的卡，后面的不读。
                    if cost_not_measured.is_none() {
                        let device = card.discrete_gpu().identifier.clone();
                        let pci_state_when_judged_unusable = self
                            .pci_state_reader
                            .pci_state_if_the_cause_judges_the_gpu_unusable(&device, &cause);
                        cost_not_measured = Some(GpuWhoseCostWasNotMeasuredAtTheSwitch {
                            device,
                            cause,
                            pci_state_when_judged_unusable,
                        });
                    }
                }
            }
        }
        self.participating = participating_by_the_settings
            .leaving_out(|position| positions_left_out.contains(&position));
        self.left_out_under_their_memory_limit = left_out_under_their_memory_limit;
        self.cost_not_measured_at_the_last_switch = cost_not_measured;
    }

    /// 此刻这组设置下一批分不分得到卡上（与这一批有几条无关）：最近交来的那一组点到的一张没开的卡重开到上限都没开成、换设置时量一张卡的代价出错了，
    /// 或参与的卡因显存上限一张都不剩，就交回是哪一种；分得到交回 `None`（切换点到的卡还在开时照的是交来之前那一组）。
    /// 两批之间那一问答换上了时带着它（[`LatestSettingsHandedIn::PutInEffect`]），核心层照它判停，不等头一批。
    /// 告知一张卡都不参与之后（切到 `cpu`）没有生效的那一组，核心层不问它。
    ///
    /// # Panics
    /// 参与的卡一张都不剩，却既没有开不成、量代价出错，也没有因显存上限不用的卡：核心层校验过的一组至少点一张卡，换上的那一组点的卡都开着，
    /// 换上之后没参与的只有后两种原因；参与的卡为空而这三样都没有的只有一种——告知一张卡都不参与（[`Self::use_no_gpu_from_the_next_batch`]）之后、
    /// 再交来的一组换上之前。这期间核心层生效的是 `cpu`：交来的一组还在开卡时两批之间那一问在走到这里之前答还在开卡，答开不成时这一组不生效、
    /// 核心层生效的照旧是 `cpu`（`singlefs_core::background_compute::BackgroundComputeOfThisMount` 只在实现答换上了那一刻换上 `gpu` 那一组），
    /// 所以核心层不为生效的那一组问它、不走 GPU 那一路交活。
    #[must_use]
    pub fn batch_not_split_over_the_gpus(&self) -> Option<BatchNotSplitOverTheGpus> {
        if let Some(not_opened) = &self.not_opened_after_the_reopenings_at_the_last_switch {
            return Some(
                BatchNotSplitOverTheGpus::GpuNotOpenedAfterTheReopeningsAtTheSwitch(
                    not_opened.clone(),
                ),
            );
        }
        if let Some(not_measured) = &self.cost_not_measured_at_the_last_switch {
            return Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
                device: not_measured.device.clone(),
                cause: not_measured.cause.clone(),
                pci_state_when_judged_unusable: not_measured.pci_state_when_judged_unusable.clone(),
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
            Some(BatchNotSplitOverTheGpus::GpuNotOpenedAfterTheReopeningsAtTheSwitch(
                GpuNotOpenedAfterTheReopenings {
                    device,
                    causes_of_each_opening,
                    pci_state_when_judged_unusable,
                },
            )) => {
                // 判它开不成那一刻读过了：每一批带的都是那一份，不重读。
                return Err(OutcomesOfTheBatchNotComputed::GpuFailed {
                    device,
                    cause: GpuFailureCause::NotOpenedAfterTheReopenings {
                        causes_of_each_opening,
                    },
                    pci_state_when_judged_unusable,
                });
            }
            Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
                device,
                cause,
                pci_state_when_judged_unusable,
            }) => {
                // 量代价出错那一刻读过了（原因不是判卡用不了的那一种时没读）：每一批带的都是那一份，不重读。
                return Err(OutcomesOfTheBatchNotComputed::GpuFailed {
                    device,
                    cause,
                    pci_state_when_judged_unusable,
                });
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
        let pci_state_reader = &self.pci_state_reader;
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
                OutcomesOfTheBatchNotComputed::GpuFailed {
                    pci_state_when_judged_unusable: pci_state_reader
                        .pci_state_if_the_cause_judges_the_gpu_unusable(&device, &cause),
                    device,
                    cause,
                }
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
    /// （第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」）；压到连最小的一片都切不出，这一批就不用这张卡
    /// （下一批照设置重新起），它分到的那几条（一条都没交回）重新分给这一批还用的参与的卡、照它们的地方切、交；
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
                            room = room.with_the_memory_limit_at_most(GpuMemoryInBytes(
                                singlefs_replay_prefix::gpu_gc_capacity_proofs::gpu_memory_cap_after_allocation_failure(bytes_held_by_the_chunk.0),
                            ));
                            // 重切不留已做完的片：这张卡分到的那几条照新切法从头整段重交。
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
                        failure = Some(OutcomesOfTheBatchNotComputed::GpuFailed {
                            pci_state_when_judged_unusable: self
                                .pci_state_reader
                                .pci_state_if_the_cause_judges_the_gpu_unusable(&device, &cause),
                            device,
                            cause,
                        });
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
    /// [`Self::outcomes_of_a_batch_over_the_cards`]。同一个条目键在这一批里出现两次，一片都不交给卡、交回重复的那个键（卡出错与重复键不同时报）。
    ///
    /// # Errors
    /// [`ChecksumBatchFailed`] 的三种。
    pub fn both_kernel_outcomes_of_the_batch<EntryKey: Eq + std::hash::Hash>(
        &mut self,
        units: &[UnitChecksumToVerify<'_, EntryKey>],
    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, ChecksumBatchFailed> {
        if let Some(repeated) =
            first_entry_key_repeated_among(units.iter().map(|unit| &unit.entry_key))
        {
            return Err(ChecksumBatchFailed::SameEntryKeyTwiceInOneBatch(repeated));
        }
        let unit_lengths: Vec<InputLengthOnTheHostInBytes> = units
            .iter()
            .map(|unit| InputLengthOnTheHostInBytes::of_this_many_bytes(unit.unit_bytes.len()))
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
    /// 最大那条记录连同一个落点上的条目还装不下时单元切成头一片与之后的几片，`region_record_kernels::region_record_plan_on_a_gpu`）；
    /// 分活、切段、超时、重切照 [`Self::outcomes_of_a_batch_over_the_cards`]。同一个条目键在这一批里出现两次，一片都不交给卡、交回重复的那个键（先于映射与记录的别的判）。
    ///
    /// # Errors
    /// [`RegionRecordBatchFailed`] 的四种。
    ///
    /// # Panics
    /// 有一条记录不是区域扫描会去读的那一种（`region_record_kernels::record_is_one_the_region_scan_reads`）：调用方先按 R 筛过。
    pub fn both_kernel_classifications_of_the_region_records<EntryKey: Eq + std::hash::Hash>(
        &mut self,
        region: CompactionRegion,
        records: &[RegionRecordToClassify<'_, EntryKey>],
        mapping_entries: &[CentralMappingEntry],
    ) -> Result<Vec<(AllocationInRegion, AllocationInRegion)>, RegionRecordBatchFailed> {
        if let Some(repeated) =
            first_entry_key_repeated_among(records.iter().map(|record| &record.entry_key))
        {
            return Err(RegionRecordBatchFailed::SameEntryKeyTwiceInOneBatch(
                repeated,
            ));
        }
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
    /// 同一个段在这一批里出现两次，一片都不交给卡、交回重复的那个段。
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
        if let Some(repeated) = first_entry_key_repeated_among(segments.iter()) {
            return Err(LiveSlotBatchFailed::SameEntryKeyTwiceInOneBatch(repeated));
        }
        assert!(
            segments
                .iter()
                .all(|segment| segment.device == unit_area.device),
            "交进来的段都在单元区那块盘上"
        );
        self.outcomes_of_a_batch_over_the_cards(
            GpuComputedSegment::LiveSlotsPerClusterSegment,
            segments.len(),
            |entries, room| {
                live_slot_plan_on_a_gpu(
                    AllocationRecordCountOfATable::of_this_many_records(allocation_records.len()),
                    ClusterSegmentCountOfAGpu::of_this_many_segments(entries.len()),
                    room,
                )
            },
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

/// 丢后台 GPU（卸载、丢掉核心层收的把手、测试里展开）：会话锁外为切换开卡的那一次叫停（开卡线程不再开下一次），不等还在 GPU 上跑的提交——
/// 还记着一次等超过上限、没做完的提交的卡整张泄漏（设备、队列、管线都不析构：wgpu-core 30.0.1 的 `Queue` 析构调 `wait_for_idle`，
/// 即 Vulkan 的 `vkQueueWaitIdle`，不带上限），别的卡照常丢（`gpus_of_this_host::gpus_dropped_leaking_those_still_running`）。
/// 泄漏了哪几张、各占着多少字节报给调用方（[`BackgroundGpuOverTheCards::gpus_leaked_when_dropped`]），并记进进程里的那一份，新一次挂载先扣掉。
impl<Card: CardOfTheBackgroundGpu> Drop for BackgroundGpuOverTheCards<Card> {
    fn drop(&mut self) {
        if let Some(opening) = &self.opening_at_a_switch {
            opening.call_off();
        }
        let cards_on_this_host = std::mem::take(&mut self.cards_on_this_host);
        let gpus_leaked = gpus_dropped_leaking_those_still_running(
            cards_on_this_host,
            card_has_a_submission_running_past_its_wait,
            card_leaked_when_dropped,
        );
        self.memory_leaked_in_this_process
            .note_the_gpus_leaked_when_dropped(&gpus_leaked);
        self.gpus_leaked_when_dropped.note_the_drop(gpus_leaked);
    }
}

/// 这张卡上记着一次等超过上限、还没在 GPU 上做完的提交：丢它的时候整张泄漏、不析构（卸载与开卡中途丢卡同一条）。
fn card_has_a_submission_running_past_its_wait<Card: CardOfTheBackgroundGpu>(card: &Card) -> bool {
    card.budget()
        .memory_account()
        .has_a_submission_running_past_its_wait()
}

/// 泄漏这张卡时报的那一条：哪张卡、那一刻本包在它上面占着的字节、开卡时已算上的这个进程里之前泄漏在它上面的字节。
fn card_leaked_when_dropped<Card: CardOfTheBackgroundGpu>(card: &Card) -> GpuLeakedWhenDropped {
    GpuLeakedWhenDropped {
        device: card.discrete_gpu().identifier.clone(),
        bytes_held_on_the_device: card.budget().memory_account().bytes_held(),
        bytes_leaked_by_earlier_mounts: card.budget().bytes_leaked_by_earlier_mounts(),
    }
}

/// 开卡时量代价出错之后处置这张卡（第 4 条「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构」，开卡中途丢卡同一条）：
/// 账上记着一次等超过上限、还在跑的提交的，整张泄漏（设备与队列不析构：wgpu-core 30.0.1 的 `Queue` 析构调不带上限的 `wait_for_idle`），
/// 这一次占着的字节（账上占着的扣掉开卡时已算上的之前泄漏的）当场记进这个进程的泄漏账；没有这种提交的照常丢。交回量代价那一次的错，原样。
fn failure_of_the_measurement_dropping_or_leaking_the_card<Card: CardOfTheBackgroundGpu>(
    card: Card,
    failure_of_the_measurement: GpuFailureCause,
    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
) -> GpuFailureCause {
    let gpus_leaked = gpus_dropped_leaking_those_still_running(
        vec![card],
        card_has_a_submission_running_past_its_wait,
        card_leaked_when_dropped,
    );
    memory_leaked_in_this_process.note_the_gpus_leaked_when_dropped(&gpus_leaked);
    failure_of_the_measurement
}

/// 卡清单（核心层挂载与切换照它校验设置，D26（后台整理与放置回收） 已定项 13 第 2、4 条）：此刻本机在的独显与各自的显存总量，按总线地址排
/// （开着的与没参与而没开的都列，切换可以点没开的卡；没给 `background_gpu_devices` 时上限按这个次序一张卡一个；[`DiscreteGpusAfterTheOpening`]）。
/// 答卡清单之前现查本机在的独显（[`DiscreteGpusPresentNow`]）：挂载之后掉了的卡不列，切换点它照「点名的卡不存在」拒、不点名时它不参与、一张都不剩照「GPU 不在」拒。
/// 切换点到的卡开不开得成不在校验里判：切换照收，开不成照 [`BackgroundGpuOverTheCards::put_the_gpus_opened_since_the_switch_in_effect`] 报。
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

/// 本机开机以来的 `CLOCK_MONOTONIC_RAW`：单调不减，不受 NTP 调频。
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
/// 期限从进这个函数起算：实参求值时做的活（造 future 那一步同步做完的枚举）不在 `wait_limit` 之内。
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
    use crate::pci_state_of_the_gpu::{
        pci_state_read_under_the_root_not_present_for_tests,
        pci_state_reader_under_a_root_not_present_for_tests,
    };
    use crate::two_kernel_reconciliation::EntryDecision;

    const WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU: Duration = Duration::from_secs(10);
    /// 用例里开卡开不成照产品那样至多重开 3 次，间隔为 0：不真等 1 分钟。
    const REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS: ReopeningsOfAGpuNotOpened =
        ReopeningsOfAGpuNotOpened {
            interval: Duration::ZERO,
            reopenings_at_most: 3,
        };
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
    /// 脚本用完之后都等到、都做完；`next_submission_failing_with` 给了时下一次交一段当场交回那个错（只一次，例：设备丢了），放掉那一段的账。另记量了几次代价（`cost_measurements`；`cost_measurement_failing_with` 给了时每次量都交回那个错、不记代价；
    /// `cost_measured_through_the_checksum_chunk` 时照产品那样把量代价那一段照此刻剩下的界切片、逐段当一次提交走账：先记账、超时就记成还在跑），可点名第几次交成的校验和那一段交回翻过的判定
    /// （`checksum_submission_with_flipped_verdicts`，从 1 数：那一次的结果是坏的，用了它判定就与 CPU 参照不同）。
    /// 析构一次就在 `drops_counted` 上加一（泄漏的卡不析构，计数不动）：用例在卡交出去之前取一份它的克隆。
    /// 只供测试的钩子 `submission_waits_panicking`：交成一段之后等它时，那一次等交给产品等提交那一步（`completion_check_of_one_poll_with_its_panic_caught`），
    /// 等的那一下 panic（造真卡掉线时 wgpu 的 `Device::poll` panic）；交活之前、重交之前等前一次还在跑的提交也走产品那一步（没 panic 过的当场等到），
    /// panic 的原话记在 `panic_in_a_wait_of_the_device`。
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
        cost_measurement_failing_with: Option<GpuFailureCause>,
        drops_counted: Arc<std::sync::atomic::AtomicU32>,
        cost_measured_through_the_checksum_chunk: bool,
        next_submission_failing_with: Option<GpuFailureCause>,
        submission_waits_panicking: bool,
        panic_in_a_wait_of_the_device: OnceLock<String>,
    }

    impl Drop for CardComputedOnTheCpuForTests {
        fn drop(&mut self) {
            self.drops_counted
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    impl CardComputedOnTheCpuForTests {
        /// 总线地址 `pci_bus_address` 的一张 8 GiB 替身卡，量过代价，给它的显存上限 `memory_limit_in_bytes`，这个进程里之前泄漏在它上面 `bytes_leaked` 字节。
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
                    GpuMemoryInBytes(1 << 30),
                    EntryCountOfAChunk(1 << 20),
                    memory_limit_in_bytes.map(GpuMemoryInBytes),
                    WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                    GpuMemoryInBytes(bytes_leaked),
                ),
                measured_cost: Some(HostCpuNanosecondsPerSubmission(1_000)),
                earlier_submission_waits_timing_out: VecDeque::new(),
                submissions_timing_out: VecDeque::new(),
                free_bytes: None,
                submissions_made: 0,
                cost_measurements: 0,
                checksum_submission_with_flipped_verdicts: None,
                cost_measurement_failing_with: None,
                drops_counted: Arc::new(std::sync::atomic::AtomicU32::new(0)),
                cost_measured_through_the_checksum_chunk: false,
                next_submission_failing_with: None,
                submission_waits_panicking: false,
                panic_in_a_wait_of_the_device: OnceLock::new(),
            }
        }

        /// 卡上记着一次等超过上限、还在跑的提交（占 `bytes` 字节），交活之前要先等它。
        fn with_a_submission_running_past_its_wait(mut self, bytes: u64) -> Self {
            self.budget
                .memory_account_mut()
                .hold(GpuMemoryInBytes(bytes));
            self.budget
                .memory_account_mut()
                .settle_the_chunk_after_its_submission(
                    GpuMemoryInBytes(bytes),
                    SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(0),
                );
            self
        }

        /// 交一段（合占 `bytes_held_by_the_chunk` 字节）：照产品记账；空闲的不够就放掉、交回显存不足；照脚本超时就把这一段记成还在跑的那次提交、交回超时；
        /// 做完了放账、交回 `outcome`。
        fn chunk_submitted_on_the_cpu<Outcome>(
            &mut self,
            bytes_held_by_the_chunk: GpuMemoryInBytes,
            outcome: impl FnOnce() -> Outcome,
        ) -> Result<Outcome, GpuFailureCause> {
            self.budget
                .memory_account_mut()
                .hold(bytes_held_by_the_chunk);
            if self
                .free_bytes
                .is_some_and(|free_bytes| bytes_held_by_the_chunk.0 > free_bytes)
            {
                self.budget
                    .memory_account_mut()
                    .release(bytes_held_by_the_chunk);
                return Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
                    bytes_held_by_the_chunk,
                    reason: "替身卡上空闲的不够".to_string(),
                });
            }
            if let Some(cause) = self.next_submission_failing_with.take() {
                self.budget
                    .memory_account_mut()
                    .release(bytes_held_by_the_chunk);
                return Err(cause);
            }
            self.submissions_made += 1;
            if self.submission_waits_panicking {
                let waited = completion_check_of_one_poll_with_its_panic_caught(
                    &self.panic_in_a_wait_of_the_device,
                    || panic!("{PANIC_SAID_BY_THE_WAIT_OF_THE_TEST_CARD}"),
                );
                let cause = match waited {
                    CompletionCheck::Failed(cause) => cause,
                    CompletionCheck::Completed(()) | CompletionCheck::NotYetComplete => {
                        panic!("替身：等的那一下 panic 了，只交得回出错")
                    }
                };
                self.budget
                    .memory_account_mut()
                    .settle_the_chunk_after_its_submission(
                        bytes_held_by_the_chunk,
                        SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(self.submissions_made),
                    );
                return Err(cause);
            }
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
            self.cost_measurements += 1;
            if let Some(cause) = &self.cost_measurement_failing_with {
                return Err(cause.clone());
            }
            if self.cost_measured_through_the_checksum_chunk {
                let chunks_of_pieces = COST_PROBE_CHUNK
                    .pieces_under_the_bounds(self.budget.room_left_now())
                    .map_err(
                        |smallest_piece| GpuFailureCause::DeviceOrPipelineDidNotStart {
                            reason: format!(
                                "替身：量代价那一段最小的一片 {} 字节装不下",
                                smallest_piece.smallest_piece_bytes.0
                            ),
                        },
                    )?;
                for pieces in &chunks_of_pieces {
                    let input_bytes: InputLengthOnTheHostInBytes = pieces
                        .iter()
                        .map(crate::memory_limit_and_chunks::PieceOfAnEntry::input_byte_count)
                        .sum();
                    let bytes_held_by_the_chunk = CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
                        input_bytes,
                        EntryCountOfAChunk::of_this_many_entries(pieces.len()),
                    );
                    self.chunk_submitted_on_the_cpu(bytes_held_by_the_chunk, || ())?;
                }
            }
            self.measured_cost = Some(HostCpuNanosecondsPerSubmission(1_000));
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
            let panic_in_a_wait_of_the_device = &self.panic_in_a_wait_of_the_device;
            self.budget
                .room_once_the_submission_running_past_its_wait_completed(
                    &ClockOfTheCardsOnTheCpu,
                    |_submission, _time_left| {
                        completion_check_of_one_poll_with_its_panic_caught(
                            panic_in_a_wait_of_the_device,
                            || Ok(wgpu::PollStatus::WaitSucceeded),
                        )
                    },
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
            let panic_in_a_wait_of_the_device = &self.panic_in_a_wait_of_the_device;
            self.budget.wait_for_the_submission_running_past_its_wait(
                &ClockOfTheCardsOnTheCpu,
                |_submission, _time_left| {
                    completion_check_of_one_poll_with_its_panic_caught(
                        panic_in_a_wait_of_the_device,
                        || Ok(wgpu::PollStatus::WaitSucceeded),
                    )
                },
            )
        }

        fn both_kernel_outcomes_of_a_checksum_chunk(
            &mut self,
            units: &[UnitChecksumToVerify<'_, ()>],
        ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, GpuFailureCause> {
            let input_bytes: usize = units.iter().map(|unit| unit.unit_bytes.len()).sum();
            let bytes_held_by_the_chunk = CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
                InputLengthOnTheHostInBytes::of_this_many_bytes(input_bytes),
                EntryCountOfAChunk::of_this_many_entries(units.len()),
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
                        InputLengthOnTheHostInBytes::of_this_many_bytes(input_bytes),
                        EntryCountOfAChunk::of_this_many_entries(pieces.len()),
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
            let segment_count = EntryCountOfAChunk::of_this_many_entries(segments.len());
            let bytes_held_by_the_chunk = live_slot_chunk_footprint(record_tables)
                .bytes_held_by_a_chunk(
                    segment_count.input_bytes_of_the_entries(SEGMENT_BOUND_BYTES),
                    segment_count,
                );
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

    /// 外壳装上这几张替身卡（按交进来的次序就是本机的次序，全都参与），进程里泄漏的那一份是 `memory_leaked`；交回外壳与「此刻本机在的独显」那一份（一开始全在）。
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
                    not_opened_for_not_taking_part: Vec::new(),
                },
                Vec::new(),
                participating,
                Vec::new(),
                memory_leaked.clone(),
                Box::new(DiscreteGpusPresentForTests(Arc::clone(&present))),
                no_opening_at_a_switch(),
                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                pci_state_reader_under_a_root_not_present_for_tests(),
            ),
            present,
        )
    }

    /// 本机的卡挂载那一刻全开了时交的开卡那一步：切换点不到没开的卡，调到它就是用例错了。
    fn no_opening_at_a_switch() -> OpeningOfAGpuAtASwitch<CardComputedOnTheCpuForTests> {
        Box::new(
            |discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                panic!(
                    "用例的卡挂载那一刻全开了，切换不该开 {}",
                    discrete_gpu.pci_bus_address
                )
            },
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

    /// 校验和那一段一批切成 4 段（上限 2200 字节只装得下一个 1000 字节的单元：连同暂存 2104 字节），每段头一次交都等超过上限、
    /// 重交一次就做完——这一批在这张卡上第 4 次超时报这张卡用不了。这一段记下交成 3 次、3 片、重交 3 次、分到活的是这张卡。
    #[test]
    fn the_checksum_batch_cut_into_four_chunks_each_timing_out_once_reports_the_card_unusable() {
        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", Some(2200), 0);
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
                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                    "0000:01:00.0"
                ),
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

    /// 外壳装上一张「0000:01:00.0」替身卡（照 `card_as_the_test_wants` 改过），交一批一条 1000 字节的单元，交回这一批的结局与这一次挂载一共读了几次 PCI 状态。
    fn checksum_batch_on_the_card_at_one_and_the_pci_state_readings(
        card_as_the_test_wants: impl FnOnce(
            CardComputedOnTheCpuForTests,
        ) -> CardComputedOnTheCpuForTests,
    ) -> (BothChecksumKernelOutcomesOfTheBatch, u64) {
        let card = card_as_the_test_wants(CardComputedOnTheCpuForTests::of_the_address(
            "0000:01:00.0",
            None,
            0,
        ));
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let unit_bytes = unit_bytes_of(1, 1000);
        let outcome =
            background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes));
        (outcome, background_gpu.pci_state_readings_taken())
    }

    /// 第 4 条「挂着时一批在同一张卡上等超时、到判那张卡用不了」那一刻查一次这张卡的硬件与链路：一批 1 片在这张卡上 4 次都等超过上限——
    /// 报这张卡用不了，带判它用不了那一刻照它的地址读的 PCI 状态；3 次超时重交之前都不读，一共只读 1 次。
    #[test]
    fn the_card_unusable_after_its_resubmissions_is_read_once_and_the_batch_failure_carries_the_reading(
    ) {
        let (outcome, pci_state_readings) =
            checksum_batch_on_the_card_at_one_and_the_pci_state_readings(|mut card| {
                card.submissions_timing_out = VecDeque::from([true, true, true, true]);
                card
            });
        assert_eq!(
            outcome,
            Err(ChecksumBatchFailed::GpuFailed {
                device: card_identifier("0000:01:00.0"),
                cause: GpuFailureCause::UnusableAfterResubmissions {
                    resubmissions: 3,
                    last_cause: Box::new(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                        wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                    }),
                },
                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                    "0000:01:00.0"
                ),
            })
        );
        assert_eq!(pci_state_readings, 1);
    }

    /// 同一条「平常的批、每次超时重交之前不查」：一批 1 片超时 3 次、第 4 次交做完——照常交回，一次都不读。
    #[test]
    fn the_batch_completing_after_three_timeouts_reads_no_pci_state() {
        let (outcome, pci_state_readings) =
            checksum_batch_on_the_card_at_one_and_the_pci_state_readings(|mut card| {
                card.submissions_timing_out = VecDeque::from([true, true, true]);
                card
            });
        assert!(outcome.is_ok(), "第 4 次交做完：照常交回（{outcome:?}）");
        assert_eq!(pci_state_readings, 0);
    }

    /// 第 4 条「卡当场报出错（设备丢失、驱动报的未捕获的错）、后台任务因此停下」那一刻查一次：这一批交到卡上时设备丢了——报这张卡的 GPU 出错，
    /// 带那一刻读的 PCI 状态，一共只读 1 次。
    #[test]
    fn the_card_lost_in_a_batch_is_read_once_and_the_batch_failure_carries_the_reading() {
        let (outcome, pci_state_readings) =
            checksum_batch_on_the_card_at_one_and_the_pci_state_readings(|mut card| {
                card.next_submission_failing_with =
                    Some(GpuFailureCause::UncapturedErrorOrDeviceLost {
                        reason: "替身：设备丢了".to_string(),
                    });
                card
            });
        assert_eq!(
            outcome,
            Err(ChecksumBatchFailed::GpuFailed {
                device: card_identifier("0000:01:00.0"),
                cause: GpuFailureCause::UncapturedErrorOrDeviceLost {
                    reason: "替身：设备丢了".to_string(),
                },
                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                    "0000:01:00.0"
                ),
            })
        );
        assert_eq!(pci_state_readings, 1);
    }

    /// 同一条：一批里等提交没等成（不是设备丢了、不是超时用完）不是判卡用不了的那三种时刻——照报这张卡的 GPU 出错，读数那一格是没读，一次都不读。
    #[test]
    fn a_batch_failing_without_judging_the_card_unusable_reads_no_pci_state() {
        let (outcome, pci_state_readings) =
            checksum_batch_on_the_card_at_one_and_the_pci_state_readings(|mut card| {
                card.next_submission_failing_with =
                    Some(GpuFailureCause::WaitForTheSubmissionFailed {
                        reason: "替身：没等成".to_string(),
                    });
                card
            });
        assert_eq!(
            outcome,
            Err(ChecksumBatchFailed::GpuFailed {
                device: card_identifier("0000:01:00.0"),
                cause: GpuFailureCause::WaitForTheSubmissionFailed {
                    reason: "替身：没等成".to_string(),
                },
                pci_state_when_judged_unusable:
                    PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
            })
        );
        assert_eq!(pci_state_readings, 0);
    }

    /// 第 4 条「等超时用完到判那张卡用不了」也算交活之前那一等：卡上记着前一次超时、还在跑的提交，这一批交活之前 4 次都等满上限没等到——
    /// 报这张卡用不了（一次都没重交到卡上），带那一刻读的 PCI 状态，一共只读 1 次。
    #[test]
    fn the_card_unusable_while_waiting_before_the_batch_is_read_once_and_the_failure_carries_the_reading(
    ) {
        let (outcome, pci_state_readings) =
            checksum_batch_on_the_card_at_one_and_the_pci_state_readings(|card| {
                let mut card = card.with_a_submission_running_past_its_wait(100);
                card.earlier_submission_waits_timing_out = VecDeque::from([true, true, true, true]);
                card
            });
        let Err(ChecksumBatchFailed::GpuFailed {
            cause: GpuFailureCause::UnusableAfterResubmissions { .. },
            pci_state_when_judged_unusable,
            ..
        }) = &outcome
        else {
            panic!("交活之前 4 次都没等到：报这张卡用不了，结局是 {outcome:?}");
        };
        assert_eq!(
            pci_state_when_judged_unusable,
            &pci_state_read_under_the_root_not_present_for_tests("0000:01:00.0")
        );
        assert_eq!(pci_state_readings, 1);
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

    /// 外壳上：两张卡，「0000:01:00.0」上限 1000 字节而这个进程里之前泄漏了 950 字节，余量 50 字节装不下校验和那一段最小的一片（连同暂存 112 字节）——
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

    /// 第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」：上限 2 MiB、卡上实际只空闲 512 KiB，八个 64 KiB 的单元
    /// 照上限一段装得下（连同暂存 1 MiB 多）而建缓冲显存不足——压到那一段的一半重切成三、三、二个一段，照常交回，判定与 CPU 参照逐条相同、全都确认；
    /// 这一段记下重切 1 次、交成 3 次、8 片。
    #[test]
    fn the_checksum_batch_on_a_card_with_less_free_memory_than_its_limit_is_recut_and_agrees_with_the_cpu(
    ) {
        let mut card =
            CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", Some(2 << 20), 0);
        card.free_bytes = Some(512 << 10);
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

    /// 只空闲 10 字节的那张替身卡上一个 1000 字节的单元一路压小（连同暂存 2104、1048、520、256、128 字节的一段各建不成一次）到连最小的一片（112 字节）都建不成时，
    /// 它报的那一条：最后建不成的一段 128 字节、最小的一片 112 字节。
    fn short_of_free_memory_after_cutting_down_a_unit_of_one_thousand_bytes(
        pci_bus_address: &str,
    ) -> GpuShortOfFreeMemoryForTheSmallestPiece {
        GpuShortOfFreeMemoryForTheSmallestPiece {
            device: card_identifier(pci_bus_address),
            bytes_of_the_last_chunk_not_created: GpuMemoryInBytes(128),
            smallest_piece_bytes: GpuMemoryInBytes(112),
        }
    }

    /// 「连一片都装不下就不用这张卡」：唯一一张卡上只空闲 10 字节，一路压小到连最小的一片（112 字节）都建不成——这一批不用这张卡，参与的卡一张都不剩，
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
            "2104、1048、520、256、128 字节的一段各建不成一次"
        );
    }

    /// 一张卡的上限一次只交得下一个 1000 字节的单元（连同暂存 2104 字节）：按卡的顺序它填一个，剩下的才给后面的卡。
    const MEMORY_LIMIT_HOLDING_ONE_UNIT_OF_ONE_THOUSAND_BYTES: u64 = 2104;

    /// 「连一片都装不下就不用这张卡」用于空闲显存：两张卡、两个 1000 字节的单元，排在前面的「0000:01:00.0」的上限一次只交得下一个，
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

    /// 丢外壳时，记着一次还在跑的提交（占 600 字节）的那张卡泄漏、另一张照常丢；泄漏了哪一张、占着多少交到调用方拿着的那一份，
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
                bytes_held_on_the_device: GpuMemoryInBytes(600),
                bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
            }]),
            "只泄漏还在跑的那一张"
        );
        assert_eq!(
            (
                memory_leaked.bytes_leaked_on(&card_identifier("0000:01:00.0")),
                memory_leaked.bytes_leaked_on(&card_identifier("0000:41:00.0"))
            ),
            (GpuMemoryInBytes(600), GpuMemoryInBytes(0))
        );
    }

    /// 开卡时量代价交回的错（等超过上限）：两格用同一个。
    fn failure_of_the_cost_measurement_in_the_test() -> GpuFailureCause {
        GpuFailureCause::WaitForTheGpuExceededTheLimit {
            wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
        }
    }

    /// 这个进程里之前几次挂载在「0000:01:00.0」上泄漏了 100 字节的那一份泄漏账。
    fn memory_leaked_with_one_hundred_bytes_on_the_card() -> GpuMemoryLeakedInThisProcess {
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
            device: card_identifier("0000:01:00.0"),
            bytes_held_on_the_device: GpuMemoryInBytes(100),
            bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
        }]);
        memory_leaked
    }

    /// 第 4 条「有这种提交的那张卡整张泄漏、不析构」用于开卡：量代价出错时账上记着一次等超过上限、还在跑的提交（这一段占 600 字节，
    /// 账从之前泄漏的 100 字节起）——这张卡不丢（替身的析构一次都没走），进程里的泄漏账在这张卡上加这一次的 600 字节、到 700，交回量代价那一次的错，原样。
    #[test]
    fn the_card_whose_cost_measurement_failed_with_a_submission_still_running_is_leaked_and_noted_in_the_process(
    ) {
        let card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 100)
            .with_a_submission_running_past_its_wait(600);
        let drops_counted = Arc::clone(&card.drops_counted);
        let memory_leaked = memory_leaked_with_one_hundred_bytes_on_the_card();
        let reported = failure_of_the_measurement_dropping_or_leaking_the_card(
            card,
            failure_of_the_cost_measurement_in_the_test(),
            &memory_leaked,
        );
        assert_eq!(
            reported,
            failure_of_the_cost_measurement_in_the_test(),
            "交回量代价那一次的错"
        );
        assert_eq!(
            drops_counted.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "提交还在跑：这张卡泄漏、不析构"
        );
        assert_eq!(
            memory_leaked.bytes_leaked_on(&card_identifier("0000:01:00.0")),
            GpuMemoryInBytes(100 + 600),
            "泄漏账加的恰好是这一次占着的 600 字节"
        );
    }

    /// 同一条的另一格：量代价出错而账上没有还在跑的提交——照常丢（替身析构一次），泄漏账不变，交回量代价那一次的错。
    #[test]
    fn the_card_whose_cost_measurement_failed_without_a_submission_still_running_is_dropped() {
        let card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 100);
        let drops_counted = Arc::clone(&card.drops_counted);
        let memory_leaked = memory_leaked_with_one_hundred_bytes_on_the_card();
        let reported = failure_of_the_measurement_dropping_or_leaking_the_card(
            card,
            failure_of_the_cost_measurement_in_the_test(),
            &memory_leaked,
        );
        assert_eq!(reported, failure_of_the_cost_measurement_in_the_test());
        assert_eq!(
            drops_counted.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "没有还在跑的提交：照常丢"
        );
        assert_eq!(
            memory_leaked.bytes_leaked_on(&card_identifier("0000:01:00.0")),
            GpuMemoryInBytes(100),
            "泄漏账不变"
        );
    }

    /// 卡清单每次答之前现查本机在的独显——挂载之后那张卡掉了，不点名的切换照「GPU 不在」拒、点名它照「点名的卡不存在」拒。
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

    /// 本机两张独显：「0000:01:00.0」挂载那一刻参与、开着；「0000:41:00.0」没参与、没开。切换点它时开它一次的那一步交 `opening_at_the_switch`，
    /// 开不成照 `reopenings` 重开；开卡那一步被交了哪几张卡（每开一次记一次）记在交回的那一份里。进程里泄漏的那一份是 `memory_leaked`。
    fn background_gpu_with_one_card_not_opened_at_the_mount(
        opening_at_the_switch: impl FnMut(
                &DiscreteGpuOnThisHost,
                Option<GpuMemoryInBytes>,
                Duration,
                CostAtTheOpening,
            ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
            + Send
            + 'static,
        reopenings: ReopeningsOfAGpuNotOpened,
        memory_leaked: &GpuMemoryLeakedInThisProcess,
    ) -> (
        BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
        Arc<Mutex<Vec<String>>>,
    ) {
        background_gpu_with_one_card_not_opened_at_the_mount_read_by(
            opening_at_the_switch,
            reopenings,
            memory_leaked,
            pci_state_reader_under_a_root_not_present_for_tests(),
        )
    }

    /// 同 [`background_gpu_with_one_card_not_opened_at_the_mount`]，判一张卡用不了时照 `pci_state_reader` 读 PCI 状态。
    fn background_gpu_with_one_card_not_opened_at_the_mount_read_by(
        opening_at_the_switch: impl FnMut(
                &DiscreteGpuOnThisHost,
                Option<GpuMemoryInBytes>,
                Duration,
                CostAtTheOpening,
            ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
            + Send
            + 'static,
        reopenings: ReopeningsOfAGpuNotOpened,
        memory_leaked: &GpuMemoryLeakedInThisProcess,
        pci_state_reader: PciStateReaderOfTheGpus,
    ) -> (
        BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
        Arc<Mutex<Vec<String>>>,
    ) {
        let opened_card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        let opened = opened_card.discrete_gpu.clone();
        let not_opened = DiscreteGpuOnThisHost {
            position_in_the_enumeration: 1,
            identifier: card_identifier("0000:41:00.0"),
            pci_bus_address: "0000:41:00.0".to_string(),
            name: "cpu card not opened at the mount".to_string(),
            total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
        };
        let (opening_noting_the_cards, cards_handed_to_the_opening) =
            opening_noting_the_cards_handed_to_it(opening_at_the_switch);
        let background_gpu = BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
            vec![opened_card],
            DiscreteGpusAfterTheOpening {
                opened: vec![opened],
                not_opened_for_not_taking_part: vec![not_opened],
            },
            Vec::new(),
            ParticipatingGpusOnThisHost::of_the_positions(vec![0]),
            Vec::new(),
            memory_leaked.clone(),
            Box::new(DiscreteGpusPresentForTests(Arc::new(Mutex::new(vec![
                card_identifier("0000:01:00.0"),
                card_identifier("0000:41:00.0"),
            ])))),
            opening_noting_the_cards,
            reopenings,
            pci_state_reader,
        );
        (background_gpu, cards_handed_to_the_opening)
    }

    /// 开卡那一步包一层：每开一次把交来的卡（总线地址）记进交回的那一份，再交 `opening_at_the_switch` 开。
    fn opening_noting_the_cards_handed_to_it(
        mut opening_at_the_switch: impl FnMut(
                &DiscreteGpuOnThisHost,
                Option<GpuMemoryInBytes>,
                Duration,
                CostAtTheOpening,
            ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
            + Send
            + 'static,
    ) -> (
        OpeningOfAGpuAtASwitch<CardComputedOnTheCpuForTests>,
        Arc<Mutex<Vec<String>>>,
    ) {
        let cards_handed_to_the_opening = Arc::new(Mutex::new(Vec::new()));
        let cards_noted_by_the_opening = Arc::clone(&cards_handed_to_the_opening);
        let opening_noting_the_cards: OpeningOfAGpuAtASwitch<CardComputedOnTheCpuForTests> =
            Box::new(
                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                    cards_noted_by_the_opening
                        .lock()
                        .expect("用例没在别处 panic 过")
                        .push(discrete_gpu.pci_bus_address.clone());
                    opening_at_the_switch(
                        discrete_gpu,
                        memory_limit,
                        wait_limit,
                        cost_at_the_opening,
                    )
                },
            );
        (opening_noting_the_cards, cards_handed_to_the_opening)
    }

    /// 开卡那一步：每一次都开成交来的那张卡（替身卡，照给它的显存上限）。
    fn opening_that_always_opens(
        discrete_gpu: &DiscreteGpuOnThisHost,
        memory_limit: Option<GpuMemoryInBytes>,
        _wait_limit: Duration,
        _cost_at_the_opening: CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause> {
        Ok(CardComputedOnTheCpuForTests::of_the_address(
            &discrete_gpu.pci_bus_address,
            memory_limit.map(|memory_limit| memory_limit.0),
            0,
        ))
    }

    /// 用例里切换那一刻开不成的原因。
    fn reason_the_card_does_not_open_at_the_switch() -> GpuFailureCause {
        GpuFailureCause::DeviceOrPipelineDidNotStart {
            reason: "替身：切换那一刻起不了设备".to_string(),
        }
    }

    /// 本机两张独显「0000:01:00.0」「0000:41:00.0」挂载那一刻都参与、都开着（按这个次序），切换再点到关掉的卡时开它一次的那一步交 `opening_at_the_switch`；
    /// 交回外壳、开卡那一步被交了哪几张卡，与「0000:41:00.0」那张替身卡析构了几次。「0000:41:00.0」那张交给用例改（例：记上一次还在跑的提交）。
    fn background_gpu_with_both_cards_opened_at_the_mount(
        opening_at_the_switch: impl FnMut(
                &DiscreteGpuOnThisHost,
                Option<GpuMemoryInBytes>,
                Duration,
                CostAtTheOpening,
            ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
            + Send
            + 'static,
        card_at_forty_one_as_the_test_wants: impl FnOnce(
            CardComputedOnTheCpuForTests,
        ) -> CardComputedOnTheCpuForTests,
        memory_leaked: &GpuMemoryLeakedInThisProcess,
    ) -> (
        BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
        Arc<Mutex<Vec<String>>>,
        Arc<std::sync::atomic::AtomicU32>,
    ) {
        let card_at_one = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        let card_at_forty_one = card_at_forty_one_as_the_test_wants(
            CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
        );
        let drops_of_the_card_at_forty_one = Arc::clone(&card_at_forty_one.drops_counted);
        let opened = vec![
            card_at_one.discrete_gpu.clone(),
            card_at_forty_one.discrete_gpu.clone(),
        ];
        let (opening_noting_the_cards, cards_handed_to_the_opening) =
            opening_noting_the_cards_handed_to_it(opening_at_the_switch);
        let background_gpu = BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
            vec![card_at_one, card_at_forty_one],
            DiscreteGpusAfterTheOpening {
                opened,
                not_opened_for_not_taking_part: Vec::new(),
            },
            Vec::new(),
            ParticipatingGpusOnThisHost::of_the_positions(vec![0, 1]),
            Vec::new(),
            memory_leaked.clone(),
            Box::new(DiscreteGpusPresentForTests(Arc::new(Mutex::new(vec![
                card_identifier("0000:01:00.0"),
                card_identifier("0000:41:00.0"),
            ])))),
            opening_noting_the_cards,
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            pci_state_reader_under_a_root_not_present_for_tests(),
        );
        (
            background_gpu,
            cards_handed_to_the_opening,
            drops_of_the_card_at_forty_one,
        )
    }

    /// 核心层照外壳答的卡清单校验点「0000:01:00.0」「0000:41:00.0」两张的切换（照这个次序）。
    fn switch_naming_both_cards_admitted_against(
        background_gpu: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
    ) -> AdmittedGpuSettings {
        admit_gpu_settings_against_the_inventory(
            &BackgroundComputeSettings {
                background_compute: BackgroundComputeBackend::Gpu,
                background_gpu_devices: Some(vec![
                    card_identifier("0000:01:00.0"),
                    card_identifier("0000:41:00.0"),
                ]),
                background_gpu_memory_limit: None,
                background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            },
            background_gpu,
        )
        .expect("卡清单列着没开的那张：切换点它照收")
    }

    /// 核心层照外壳答的卡清单校验只点 `pci_bus_address` 那一张的切换。
    fn switch_naming_one_card_admitted_against(
        background_gpu: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
        pci_bus_address: &str,
    ) -> AdmittedGpuSettings {
        admit_gpu_settings_against_the_inventory(
            &BackgroundComputeSettings {
                background_compute: BackgroundComputeBackend::Gpu,
                background_gpu_devices: Some(vec![card_identifier(pci_bus_address)]),
                background_gpu_memory_limit: None,
                background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            },
            background_gpu,
        )
        .expect("卡清单列着本机两张独显：切换点其中一张照收")
    }

    /// 一批校验和两核各自的结局，按单元的次序。
    type BothChecksumKernelOutcomesOfTheBatch =
        Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, ChecksumBatchFailed>;

    /// 在外壳上交一批校验和那一段（一条 1000 字节的单元），交回这一批的结局与这一段分到活的卡。
    fn checksum_batch_and_the_devices_given_work(
        background_gpu: &mut BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
    ) -> (
        BothChecksumKernelOutcomesOfTheBatch,
        Vec<GpuDeviceIdentifier>,
    ) {
        let unit_bytes = unit_bytes_of(1, 1000);
        let units = units_to_verify(&unit_bytes);
        let outcome = background_gpu.both_kernel_outcomes_of_the_batch(&units);
        let devices_given_work = background_gpu
            .take_the_submissions_of_the_batch()
            .unit_checksum_verification
            .devices_given_work;
        (outcome, devices_given_work)
    }

    /// 此刻参与的卡，按参与的次序。
    fn participating_devices_of(
        background_gpu: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
    ) -> Vec<GpuDeviceIdentifier> {
        background_gpu
            .participating_gpus()
            .into_iter()
            .map(|participating| participating.identifier)
            .collect()
    }

    /// 第 2 条「挂着时切换点到还没开的卡，那一刻再开那张卡」：挂载点「0000:01:00.0」、切换点它与「0000:41:00.0」——核心层照卡清单收下这一次切换、校验那一刻不开卡；
    /// 交设置之后开「0000:41:00.0」（只开这一张、只开一次），开完了在两批之间换上，两张都参与、按点名的次序；再交同一组不重开。
    #[test]
    fn the_switch_naming_a_card_not_opened_at_the_mount_opens_that_card_when_the_switch_takes_effect(
    ) {
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_one_card_not_opened_at_the_mount(
                opening_that_always_opens,
                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        let openings_so_far = || {
            cards_handed_to_the_opening
                .lock()
                .expect("用例没在别处 panic 过")
                .clone()
        };
        let admitted = switch_naming_both_cards_admitted_against(&background_gpu);
        assert!(openings_so_far().is_empty(), "校验那一刻不开卡");
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            },
            "开成了：换上，这一组用得上"
        );
        assert_eq!(
            openings_so_far(),
            vec!["0000:41:00.0".to_string()],
            "只开挂载时没开的那一张"
        );
        assert_eq!(
            background_gpu.gpus_opened(),
            vec![
                card_identifier("0000:01:00.0"),
                card_identifier("0000:41:00.0")
            ]
        );
        assert_eq!(
            participating_devices_of(&background_gpu),
            vec![
                card_identifier("0000:01:00.0"),
                card_identifier("0000:41:00.0")
            ],
            "两张都参与、按点名的次序"
        );
        assert_eq!(background_gpu.batch_not_split_over_the_gpus(), None);
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            }
        );
        assert_eq!(openings_so_far().len(), 1, "开着的卡不重开");
    }

    /// 第 4 条「切换请求照收，在会话锁外开新卡；重开期间照旧用切换前那组卡，开成了，下一批起换成新设置」：切换只点「0000:41:00.0」，
    /// 开它那一步等用例放开才交回——交设置照常返回、不等开卡（核心层在会话锁里交设置），两批之间不等它的那一问交回 `None`，
    /// 其间那一批照切换之前那一组只分给「0000:01:00.0」；放开之后换上，下一批只分给「0000:41:00.0」，不再点名的那张关掉。
    #[test]
    fn the_handing_in_and_the_batches_while_the_card_opens_do_not_wait_for_it_and_run_on_the_cards_before_the_switch(
    ) {
        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_one_card_not_opened_at_the_mount(
                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                    // 等放开有个上限：交设置要是在这一刻等开卡（会话锁里等），开卡到点交回开不成，用例照断言红，不一直挂着。
                    opening_released
                        .recv_timeout(Duration::from_secs(20))
                        .map_err(|_not_released| reason_the_card_does_not_open_at_the_switch())?;
                    opening_that_always_opens(
                        discrete_gpu,
                        memory_limit,
                        wait_limit,
                        cost_at_the_opening,
                    )
                },
                ReopeningsOfAGpuNotOpened {
                    interval: Duration::ZERO,
                    reopenings_at_most: 0,
                },
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::NotAtAll
            ),
            LatestSettingsHandedIn::StillOpeningItsCards,
            "还在开：不等它"
        );
        let (outcome_while_opening, devices_given_work_while_opening) =
            checksum_batch_and_the_devices_given_work(&mut background_gpu);
        assert!(
            outcome_while_opening.is_ok(),
            "开着的这段时间照常算（{outcome_while_opening:?}）"
        );
        assert_eq!(
            devices_given_work_while_opening,
            vec![card_identifier("0000:01:00.0")],
            "开着的这段时间照旧照切换之前那一组跑"
        );
        release_the_opening
            .send(())
            .expect("开卡那一步还在等放开：收的那一头还在");
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            },
            "开成了：换上，这一组用得上"
        );
        let (outcome_after_the_opening, devices_given_work_after_the_opening) =
            checksum_batch_and_the_devices_given_work(&mut background_gpu);
        assert!(
            outcome_after_the_opening.is_ok(),
            "换上之后照常算（{outcome_after_the_opening:?}）"
        );
        assert_eq!(
            devices_given_work_after_the_opening,
            vec![card_identifier("0000:41:00.0")],
            "开成了：下一批起换成新设置"
        );
        assert_eq!(
            background_gpu.gpus_opened(),
            vec![card_identifier("0000:41:00.0")],
            "不再点名的那张关掉"
        );
    }

    /// 第 4 条「间隔 1 分钟重开，最多重开 3 次」：切换点到的「0000:41:00.0」头两次开不成、第三次开成——照常换上、两张都参与，
    /// 开卡那一步一共只开它 3 次。
    #[test]
    fn the_card_not_opened_twice_at_the_switch_opens_the_third_time_and_the_switch_takes_effect() {
        let openings_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let openings_counted = Arc::clone(&openings_made);
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_one_card_not_opened_at_the_mount(
                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                    let opening =
                        openings_counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    if opening < 3 {
                        return Err(reason_the_card_does_not_open_at_the_switch());
                    }
                    opening_that_always_opens(
                        discrete_gpu,
                        memory_limit,
                        wait_limit,
                        cost_at_the_opening,
                    )
                },
                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        let admitted = switch_naming_both_cards_admitted_against(&background_gpu);
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            },
            "第三次开成：换上"
        );
        assert_eq!(
            *cards_handed_to_the_opening
                .lock()
                .expect("用例没在别处 panic 过"),
            vec!["0000:41:00.0".to_string(); 3],
            "开不成两次之后重开、第三次开成就不再开"
        );
        assert_eq!(
            participating_devices_of(&background_gpu),
            vec![
                card_identifier("0000:01:00.0"),
                card_identifier("0000:41:00.0")
            ],
            "换上了：两张都参与"
        );
        assert_eq!(background_gpu.batch_not_split_over_the_gpus(), None);
    }

    /// 第 4 条「重开 3 次都不成，GC 停下，报警里报出哪张卡、每次开不成的原因，等人处理，不自动再试」：切换只点「0000:41:00.0」，它头一次与 3 次重开都开不成——
    /// 两批之间那一问答开不成、带这张卡与 4 次的原因（再问照答，之后不再开），之后那一批报成这张卡的 GPU 出错（带每一次的原因）、一条活都不分出去；
    /// 照旧只开着切换之前那一张。下一次换设置（切回「0000:01:00.0」）解除它，那一批照常只分给它。
    #[test]
    fn the_switch_to_only_a_card_that_does_not_open_is_reported_as_its_gpu_failure_and_gives_it_no_work(
    ) {
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_one_card_not_opened_at_the_mount(
                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                    Err(reason_the_card_does_not_open_at_the_switch())
                },
                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        let not_opened = GpuNotOpenedAfterTheReopenings {
            device: card_identifier("0000:41:00.0"),
            pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                "0000:41:00.0",
            ),
            causes_of_each_opening: vec![reason_the_card_does_not_open_at_the_switch(); 4],
        };
        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::NotOpened(not_opened.clone()),
            "头一次与 3 次重开都开不成：报出这张卡与每一次的原因"
        );
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::NotOpened(not_opened.clone()),
            "再问照答开不成、不再开"
        );
        assert_eq!(
            *cards_handed_to_the_opening
                .lock()
                .expect("用例没在别处 panic 过"),
            vec!["0000:41:00.0".to_string(); 4],
            "开 4 次之后不自动再试"
        );
        assert_eq!(
            background_gpu.batch_not_split_over_the_gpus(),
            Some(
                BatchNotSplitOverTheGpus::GpuNotOpenedAfterTheReopeningsAtTheSwitch(
                    not_opened.clone()
                )
            ),
        );
        assert_eq!(
            background_gpu.gpus_opened(),
            vec![card_identifier("0000:01:00.0")],
            "开不成：照旧只开着切换之前那一张"
        );
        let (outcome, devices_given_work) =
            checksum_batch_and_the_devices_given_work(&mut background_gpu);
        assert_eq!(
            outcome,
            Err(ChecksumBatchFailed::GpuFailed {
                device: card_identifier("0000:41:00.0"),
                cause: GpuFailureCause::NotOpenedAfterTheReopenings {
                    causes_of_each_opening: not_opened.causes_of_each_opening,
                },
                pci_state_when_judged_unusable: not_opened.pci_state_when_judged_unusable,
            }),
            "之后那一批报成这张卡的 GPU 出错"
        );
        assert!(
            devices_given_work.is_empty(),
            "一条活都没分出去（{devices_given_work:?}）"
        );
        let back_to_the_card_opened_at_the_mount =
            switch_naming_one_card_admitted_against(&background_gpu, "0000:01:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&back_to_the_card_opened_at_the_mount);
        assert_eq!(
            background_gpu.batch_not_split_over_the_gpus(),
            None,
            "下一次换设置解除它"
        );
        let (outcome_after_switching_back, devices_given_work_after_switching_back) =
            checksum_batch_and_the_devices_given_work(&mut background_gpu);
        assert!(outcome_after_switching_back.is_ok());
        assert_eq!(
            devices_given_work_after_switching_back,
            vec![card_identifier("0000:01:00.0")]
        );
    }

    /// 第 4 条「开卡重开用完都开不成」那一刻查一次、「这张卡再一次被判用不了时才再读一次」：切换只点「0000:41:00.0」，它头一次与 3 次重开都开不成——
    /// 判它开不成那一刻（会话锁外的开卡线程上）读 1 次；之后两批之间连问 3 次、连交 2 批，报的都是那一份，不再读。
    #[test]
    fn the_switch_not_opened_is_read_once_and_every_later_answer_and_batch_carries_that_reading() {
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_one_card_not_opened_at_the_mount(
                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                    Err(reason_the_card_does_not_open_at_the_switch())
                },
                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        let pci_state_read = pci_state_read_under_the_root_not_present_for_tests("0000:41:00.0");
        let answers: Vec<LatestSettingsHandedIn> = (0..3)
            .map(|_question| {
                background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                    WaitForTheOpeningAtASwitch::UntilItEnds,
                )
            })
            .collect();
        assert!(
            answers.iter().all(|answer| matches!(
                answer,
                LatestSettingsHandedIn::NotOpened(not_opened)
                    if not_opened.pci_state_when_judged_unusable == pci_state_read
            )),
            "每一问都答开不成、带判它开不成那一刻读的那一份：{answers:?}"
        );
        let batch_outcomes: Vec<BothChecksumKernelOutcomesOfTheBatch> = (0..2)
            .map(|_batch| checksum_batch_and_the_devices_given_work(&mut background_gpu).0)
            .collect();
        assert!(
            batch_outcomes.iter().all(|outcome| matches!(
                outcome,
                Err(ChecksumBatchFailed::GpuFailed {
                    pci_state_when_judged_unusable,
                    ..
                }) if *pci_state_when_judged_unusable == pci_state_read
            )),
            "之后每一批都报成这张卡的 GPU 出错、带同一份读数：{batch_outcomes:?}"
        );
        assert_eq!(background_gpu.pci_state_readings_taken(), 1);
    }

    /// 同一条「开卡每一次没开成而还要重开时都不查」：切换点到的「0000:41:00.0」头两次开不成、第三次开成——换上了，一次都不读。
    #[test]
    fn the_card_opened_at_its_third_try_at_a_switch_reads_no_pci_state() {
        let openings_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let openings_counted = Arc::clone(&openings_made);
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_one_card_not_opened_at_the_mount(
                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                    let opening =
                        openings_counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    if opening < 3 {
                        return Err(reason_the_card_does_not_open_at_the_switch());
                    }
                    opening_that_always_opens(
                        discrete_gpu,
                        memory_limit,
                        wait_limit,
                        cost_at_the_opening,
                    )
                },
                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        let admitted = switch_naming_both_cards_admitted_against(&background_gpu);
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            }
        );
        assert_eq!(background_gpu.pci_state_readings_taken(), 0);
    }

    /// 第 2 条「切换之后不再参与的卡，切换生效时关掉」：挂载开着「0000:01:00.0」「0000:41:00.0」，切换只点「0000:01:00.0」——
    /// 换上那一刻「0000:41:00.0」关掉（替身卡析构一次、不再开着），卡清单照列它；再切到两张都点，它重开（开卡那一步走一次），两张都参与。
    #[test]
    fn the_card_no_longer_named_by_a_switch_is_closed_and_reopened_when_named_again() {
        let (mut background_gpu, cards_handed_to_the_opening, drops_of_the_card_at_forty_one) =
            background_gpu_with_both_cards_opened_at_the_mount(
                opening_that_always_opens,
                |card| card,
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        let only_the_card_at_one =
            switch_naming_one_card_admitted_against(&background_gpu, "0000:01:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&only_the_card_at_one);
        assert_eq!(
            drops_of_the_card_at_forty_one.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "不再点名的那张关掉：设备丢掉"
        );
        assert_eq!(
            background_gpu.gpus_opened(),
            vec![card_identifier("0000:01:00.0")]
        );
        assert_eq!(
            background_gpu
                .devices_with_total_memory()
                .into_iter()
                .map(|(device, _total_memory)| device)
                .collect::<Vec<_>>(),
            vec![
                card_identifier("0000:01:00.0"),
                card_identifier("0000:41:00.0")
            ],
            "卡清单照列关掉的那张：切换可以再点它"
        );
        let both_cards = switch_naming_both_cards_admitted_against(&background_gpu);
        background_gpu.use_the_settings_from_the_next_batch(&both_cards);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            }
        );
        assert_eq!(
            *cards_handed_to_the_opening
                .lock()
                .expect("用例没在别处 panic 过"),
            vec!["0000:41:00.0".to_string()],
            "再点它就重开"
        );
        assert_eq!(
            participating_devices_of(&background_gpu),
            vec![
                card_identifier("0000:01:00.0"),
                card_identifier("0000:41:00.0")
            ]
        );
    }

    /// 第 4 条「有这种提交的那张卡整张泄漏、不析构」用于切换时关掉的卡：「0000:41:00.0」上记着一次等超过上限、还在跑的提交（占 600 字节），
    /// 切换不再点它——它不析构，进程里的泄漏账在它上面加这一次的 600 字节；它照样不再开着。
    #[test]
    fn the_card_closed_at_a_switch_with_a_submission_still_running_is_leaked_and_noted_in_the_process(
    ) {
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        let (mut background_gpu, _cards_handed_to_the_opening, drops_of_the_card_at_forty_one) =
            background_gpu_with_both_cards_opened_at_the_mount(
                opening_that_always_opens,
                |card| card.with_a_submission_running_past_its_wait(600),
                &memory_leaked,
            );
        let only_the_card_at_one =
            switch_naming_one_card_admitted_against(&background_gpu, "0000:01:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&only_the_card_at_one);
        assert_eq!(
            drops_of_the_card_at_forty_one.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "提交还在跑：关掉的这张泄漏、不析构"
        );
        assert_eq!(
            memory_leaked.bytes_leaked_on(&card_identifier("0000:41:00.0")),
            GpuMemoryInBytes(600),
            "泄漏账加的恰好是这一次占着的 600 字节"
        );
        assert_eq!(
            background_gpu.gpus_opened(),
            vec![card_identifier("0000:01:00.0")]
        );
    }

    /// 第 4 条「有这种提交的那张卡整张泄漏、不析构」用于切换开的卡：切换点挂载时没开的「0000:41:00.0」，开卡那一步是产品那一份
    /// （`opening_of_a_card_within_the_limit`，起设备那一件由用例交、造一张替身卡），替身卡上记着一次等超过上限、还在跑的提交（占 600 字节），量代价次次出错——
    /// 头一次与 3 次重开造的 4 张卡都不析构（替身的析构一次都没走），进程里的泄漏账在它上面每一次加 600 字节；切换按这张卡重开到上限都没开成报（每一次的原因
    /// 都是量代价那一次的错，原样），它没算开成。
    #[test]
    fn the_card_opened_at_the_switch_whose_cost_measurement_fails_with_a_submission_still_running_is_leaked_and_noted_in_the_process(
    ) {
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        let drops_counted = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let drops_counted_by_the_card = Arc::clone(&drops_counted);
        let opening_at_the_switch = opening_of_a_card_within_the_limit(
            WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
            move |discrete_gpu: &DiscreteGpuOnThisHost,
                  memory_limit,
                  _wait_limit,
                  bytes_leaked_by_earlier_mounts| {
                let mut card = CardComputedOnTheCpuForTests::of_the_address(
                    &discrete_gpu.pci_bus_address,
                    memory_limit.map(|memory_limit| memory_limit.0),
                    bytes_leaked_by_earlier_mounts.0,
                )
                .with_a_submission_running_past_its_wait(600);
                card.measured_cost = None;
                card.cost_measurement_failing_with =
                    Some(failure_of_the_cost_measurement_in_the_test());
                card.drops_counted = Arc::clone(&drops_counted_by_the_card);
                move || Ok(card)
            },
            memory_leaked.clone(),
            OpeningsNotReturnedInThisProcess::with_none_not_returned(),
        );
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_one_card_not_opened_at_the_mount(
                opening_at_the_switch,
                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                &memory_leaked,
            );
        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
                device: card_identifier("0000:41:00.0"),
                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                    "0000:41:00.0"
                ),
                causes_of_each_opening: vec![failure_of_the_cost_measurement_in_the_test(); 4],
            }),
            "切换按这张卡重开到上限都没开成报，每一次的原因是量代价那一次的错"
        );
        assert_eq!(
            drops_counted.load(std::sync::atomic::Ordering::SeqCst),
            0,
            "提交还在跑：切换开的这几张卡泄漏、不析构"
        );
        assert_eq!(
            memory_leaked.bytes_leaked_on(&card_identifier("0000:41:00.0")),
            GpuMemoryInBytes(4 * 600),
            "泄漏账每一次加的恰好是那一次占着的 600 字节"
        );
        assert_eq!(
            background_gpu.gpus_opened(),
            vec![card_identifier("0000:01:00.0")],
            "量代价出错的那张没算开成"
        );
    }

    /// 本机三张独显：「0000:01:00.0」挂载那一刻开着（给它的显存上限 `memory_limit_of_the_card_at_one`；`left_out_at_the_mount` 时它因显存上限不参与、
    /// 参与的卡一张都不剩），「0000:41:00.0」「0000:81:00.0」没开。切换点到没开的卡时开它那一步交 `opening_at_the_switch`，重开照用例的规矩
    /// （间隔 0、至多重开 3 次）；开卡那一步被交了哪几张卡记在交回的那一份里。
    fn background_gpu_with_the_card_at_one_opened_and_two_not_opened(
        opening_at_the_switch: impl FnMut(
                &DiscreteGpuOnThisHost,
                Option<GpuMemoryInBytes>,
                Duration,
                CostAtTheOpening,
            ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
            + Send
            + 'static,
        memory_limit_of_the_card_at_one: Option<u64>,
        left_out_at_the_mount: bool,
    ) -> (
        BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
        Arc<Mutex<Vec<String>>>,
    ) {
        let card_at_one = CardComputedOnTheCpuForTests::of_the_address(
            "0000:01:00.0",
            memory_limit_of_the_card_at_one,
            0,
        );
        let opened = card_at_one.discrete_gpu.clone();
        let not_opened_card =
            |pci_bus_address: &str, position_in_the_enumeration: usize| DiscreteGpuOnThisHost {
                position_in_the_enumeration,
                identifier: card_identifier(pci_bus_address),
                pci_bus_address: pci_bus_address.to_string(),
                name: format!("cpu card at {pci_bus_address} not opened at the mount"),
                total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
            };
        let (participating, left_out) = if left_out_at_the_mount {
            (
                ParticipatingGpusOnThisHost::of_the_positions(Vec::new()),
                vec![GpuLeftOutUnderItsMemoryLimit {
                    device: card_identifier("0000:01:00.0"),
                    memory_limit: GpuMemoryInBytes(memory_limit_of_the_card_at_one.unwrap_or(0)),
                    smallest_piece_bytes: bytes_of_the_smallest_piece_every_participating_gpu_holds(
                    ),
                    bytes_already_held_on_the_device: GpuMemoryInBytes(0),
                }],
            )
        } else {
            (
                ParticipatingGpusOnThisHost::of_the_positions(vec![0]),
                Vec::new(),
            )
        };
        let (opening_noting_the_cards, cards_handed_to_the_opening) =
            opening_noting_the_cards_handed_to_it(opening_at_the_switch);
        let background_gpu = BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
            vec![card_at_one],
            DiscreteGpusAfterTheOpening {
                opened: vec![opened],
                not_opened_for_not_taking_part: vec![
                    not_opened_card("0000:41:00.0", 1),
                    not_opened_card("0000:81:00.0", 2),
                ],
            },
            Vec::new(),
            participating,
            left_out,
            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            Box::new(DiscreteGpusPresentForTests(Arc::new(Mutex::new(vec![
                card_identifier("0000:01:00.0"),
                card_identifier("0000:41:00.0"),
                card_identifier("0000:81:00.0"),
            ])))),
            opening_noting_the_cards,
            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
            pci_state_reader_under_a_root_not_present_for_tests(),
        );
        (background_gpu, cards_handed_to_the_opening)
    }

    /// 核心层照外壳答的卡清单校验点 `addresses` 那几张（按这个次序）、显存上限 `memory_limits` 的切换。
    fn switch_naming_the_cards_admitted_against(
        background_gpu: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
        addresses: &[&str],
        memory_limits: Option<Vec<u64>>,
    ) -> AdmittedGpuSettings {
        admit_gpu_settings_against_the_inventory(
            &BackgroundComputeSettings {
                background_compute: BackgroundComputeBackend::Gpu,
                background_gpu_devices: Some(
                    addresses
                        .iter()
                        .map(|address| card_identifier(address))
                        .collect(),
                ),
                background_gpu_memory_limit: memory_limits
                    .map(|limits| limits.into_iter().map(GpuMemoryInBytes).collect()),
                background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            },
            background_gpu,
        )
        .expect("卡清单列着这几张：切换照收")
    }

    fn openings_made_so_far(cards_handed_to_the_opening: &Arc<Mutex<Vec<String>>>) -> usize {
        cards_handed_to_the_opening
            .lock()
            .expect("用例没在别处 panic 过")
            .len()
    }

    /// 开卡那一步：等用例放开（至多 20 秒，没放开就交回开不成，用例照断言红、不一直挂着）再开成交来的那张卡。
    fn opening_that_opens_once_released(
        opening_released: std::sync::mpsc::Receiver<()>,
    ) -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
            opening_released
                .recv_timeout(Duration::from_secs(20))
                .map_err(|_not_released| reason_the_card_does_not_open_at_the_switch())?;
            opening_that_always_opens(discrete_gpu, memory_limit, wait_limit, cost_at_the_opening)
        }
    }

    /// 第 4 条「停着的时候收到任何一次通过校验的切换请求……当场解除停下，从下一批起用新设置」：挂载那一刻「0000:01:00.0」因显存上限不参与、
    /// 参与的卡一张都不剩；切换只点没开的「0000:41:00.0」（不给上限）——开卡的这段时间那一问答还在开卡（不答切换之前那一组的「一张都不剩」），
    /// 开完换上之后答换上了、这一组用得上，参与的只有「0000:41:00.0」。
    #[test]
    fn the_settings_whose_card_is_still_opening_are_answered_as_still_opening_and_as_usable_once_put_in_effect(
    ) {
        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_that_opens_once_released(opening_released),
                Some(64),
                true,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::NotAtAll
            ),
            LatestSettingsHandedIn::StillOpeningItsCards,
            "还在开卡：这一组还没生效，不答切换之前那一组用不用得上"
        );
        release_the_opening
            .send(())
            .expect("开卡那一步还在等放开：收的那一头还在");
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            },
            "开完换上：这一组用得上"
        );
        assert_eq!(
            participating_devices_of(&background_gpu),
            vec![card_identifier("0000:41:00.0")]
        );
    }

    /// 第 4 条「挂着切换到参与的卡一张都不剩的设置……报的是挂载那一刻同一个成员」用于点没开的卡：切换只点没开的「0000:41:00.0」、给它 64 字节的上限
    /// （连最小的一片都装不下）——开完换上那一问答换上了，连同这一组下一批分不到卡上（参与的卡因显存上限一张都不剩，晾下的是「0000:41:00.0」）。
    #[test]
    fn the_switch_put_in_effect_after_the_opening_with_every_card_left_out_is_answered_with_every_card_left_out(
    ) {
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_that_always_opens,
                None,
                false,
            );
        let to_the_card_at_forty_one_under_a_tiny_limit = switch_naming_the_cards_admitted_against(
            &background_gpu,
            &["0000:41:00.0"],
            Some(vec![64]),
        );
        background_gpu
            .use_the_settings_from_the_next_batch(&to_the_card_at_forty_one_under_a_tiny_limit);
        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        assert!(
            matches!(
                &answered,
                LatestSettingsHandedIn::PutInEffect {
                    not_split_over_the_gpus: Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(left_out)),
                } if left_out.iter().map(|gpu| gpu.device.clone()).collect::<Vec<_>>()
                    == vec![card_identifier("0000:41:00.0")]
            ),
            "换上了，连同这一组一张卡都用不上：{answered:?}"
        );
    }

    /// 第 2 条「切换之后不再参与的卡，切换生效时关掉」：切换点「0000:41:00.0」，它在开（等用例放开）；又来一次切换只点「0000:81:00.0」。放开之后
    /// 「0000:41:00.0」开成，接着为最近那一组开「0000:81:00.0」，它头一次与 3 次重开都开不成——此刻生效那一组（挂载那一组）之外开着的卡全部关掉：
    /// 为被盖掉的那一组开成的「0000:41:00.0」关掉（替身卡析构一次），开着的只剩「0000:01:00.0」。
    #[test]
    fn the_card_opened_for_a_superseded_switch_is_closed_when_the_latest_switch_does_not_open() {
        let (release_the_card_at_forty_one, card_at_forty_one_released) =
            std::sync::mpsc::channel::<()>();
        let drops_of_the_card_at_forty_one = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let drops_noted_by_the_opening = Arc::clone(&drops_of_the_card_at_forty_one);
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                    if discrete_gpu.pci_bus_address == "0000:41:00.0" {
                        card_at_forty_one_released
                            .recv_timeout(Duration::from_secs(20))
                            .map_err(|_not_released| {
                                reason_the_card_does_not_open_at_the_switch()
                            })?;
                        let mut card = opening_that_always_opens(
                            discrete_gpu,
                            memory_limit,
                            wait_limit,
                            cost_at_the_opening,
                        )?;
                        card.drops_counted = Arc::clone(&drops_noted_by_the_opening);
                        return Ok(card);
                    }
                    Err(reason_the_card_does_not_open_at_the_switch())
                },
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        let to_the_card_at_eighty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
        release_the_card_at_forty_one
            .send(())
            .expect("开「0000:41:00.0」那一步还在等放开");
        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        assert!(
            matches!(
                &answered,
                LatestSettingsHandedIn::NotOpened(not_opened)
                    if not_opened.device == card_identifier("0000:81:00.0")
            ),
            "最近那一组点的「0000:81:00.0」开不成：{answered:?}"
        );
        assert_eq!(
            background_gpu.gpus_opened(),
            vec![card_identifier("0000:01:00.0")],
            "此刻生效那一组之外开着的卡全部关掉"
        );
        assert_eq!(
            drops_of_the_card_at_forty_one.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "为被盖掉的那一组开成的卡关掉了"
        );
        assert_eq!(
            participating_devices_of(&background_gpu),
            vec![card_identifier("0000:01:00.0")]
        );
    }

    /// 第 4 条「一张卡至多开 4 次」在一次挂载里连着数（开成一次清零）：切换只点「0000:41:00.0」，头一次与 3 次重开都开不成；开卡线程开完 4 次之后、两批之间那一问还没问到时，
    /// 同一组设置又交了一次——不再开它（开卡那一步一共只被调 4 次），那一问答开不成，原因只一个：这一次挂载里连着试满了 4 次。
    #[test]
    fn the_same_settings_handed_in_again_once_the_openings_of_the_card_are_used_up_open_it_no_more()
    {
        let (opened_four_times, every_opening_made) = std::sync::mpsc::channel::<()>();
        let openings_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let openings_counted = Arc::clone(&openings_made);
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                move |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                    let opening =
                        openings_counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    if opening == 4 {
                        // 收的那一头还在：用例在等这一声。
                        let _signal_sent = opened_four_times.send(());
                    }
                    Err(reason_the_card_does_not_open_at_the_switch())
                },
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        every_opening_made
            .recv_timeout(Duration::from_secs(20))
            .expect("开卡线程 20 秒内开完 4 次");
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        assert_eq!(
            openings_made_so_far(&cards_handed_to_the_opening),
            4,
            "连着试满了：又交的那一次不再开"
        );
        assert_eq!(
            answered,
            LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
                device: card_identifier("0000:41:00.0"),
                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                    "0000:41:00.0"
                ),
                causes_of_each_opening: vec![GpuFailureCause::OpeningsInThisMountUsedUp {
                    failed_openings: 4
                }],
            })
        );
    }

    /// 交来的一组点的卡都开着就当场换上，在开的那一次为更早一次交来的设置开成的卡收下时关掉：切换点没开的「0000:41:00.0」，它在开（等用例放开）；
    /// 又交来只点开着的「0000:01:00.0」的一组——那一问当场答换上了；放开之后「0000:41:00.0」开成、收下，此刻生效那一组没点它，关掉。
    #[test]
    fn the_settings_handed_in_while_an_earlier_opening_runs_with_their_cards_open_take_effect_at_once_and_the_card_opened_later_is_closed(
    ) {
        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_that_opens_once_released(opening_released),
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        let back_to_the_card_at_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:01:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        background_gpu.use_the_settings_from_the_next_batch(&back_to_the_card_at_one);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::NotAtAll
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            },
            "点的卡都开着：当场换上"
        );
        release_the_opening
            .send(())
            .expect("开卡那一步还在等放开：收的那一头还在");
        drop(release_the_opening);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            }
        );
        assert_eq!(
            background_gpu.gpus_opened(),
            vec![card_identifier("0000:01:00.0")],
            "为更早一次交来的设置开成的卡收下时关掉"
        );
    }

    /// 上一次切换的卡还在开时又交来一组，等那一次开完再看，不为它另起一条：切换点没开的「0000:41:00.0」，它在开（等用例放开）；又交来点
    /// 「0000:41:00.0」「0000:81:00.0」的一组——放开之后收下「0000:41:00.0」，再为最近那一组只开还没开的「0000:81:00.0」，开卡那一步按次序只被交这两张、各一次。
    #[test]
    fn the_settings_handed_in_while_an_earlier_opening_runs_wait_for_it_and_open_only_the_cards_still_not_opened(
    ) {
        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
        let mut open_once_released = opening_that_opens_once_released(opening_released);
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                    if discrete_gpu.pci_bus_address == "0000:41:00.0" {
                        return open_once_released(
                            discrete_gpu,
                            memory_limit,
                            wait_limit,
                            cost_at_the_opening,
                        );
                    }
                    opening_that_always_opens(
                        discrete_gpu,
                        memory_limit,
                        wait_limit,
                        cost_at_the_opening,
                    )
                },
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        let to_both_cards_not_opened_at_the_mount = switch_naming_the_cards_admitted_against(
            &background_gpu,
            &["0000:41:00.0", "0000:81:00.0"],
            None,
        );
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        background_gpu.use_the_settings_from_the_next_batch(&to_both_cards_not_opened_at_the_mount);
        release_the_opening
            .send(())
            .expect("开卡那一步还在等放开：收的那一头还在");
        drop(release_the_opening);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            }
        );
        assert_eq!(
            *cards_handed_to_the_opening
                .lock()
                .expect("用例没在别处 panic 过"),
            vec!["0000:41:00.0".to_string(), "0000:81:00.0".to_string()],
            "每张卡只开一次"
        );
        assert_eq!(
            participating_devices_of(&background_gpu),
            vec![
                card_identifier("0000:41:00.0"),
                card_identifier("0000:81:00.0")
            ]
        );
    }

    /// 第 2 条「从 gpu 切到 cpu……关掉全部开着的卡」：挂载开着「0000:01:00.0」「0000:41:00.0」，告知一张卡都不参与——开着的卡为零（「0000:41:00.0」那张替身卡析构一次），
    /// 卡清单照列两张；再交来只点「0000:41:00.0」的一组，它当没开的卡开（开卡那一步走一次），开完换上。
    #[test]
    fn no_gpu_taking_part_closes_every_open_card_and_the_next_settings_open_their_cards_again() {
        let (mut background_gpu, cards_handed_to_the_opening, drops_of_the_card_at_forty_one) =
            background_gpu_with_both_cards_opened_at_the_mount(
                opening_that_always_opens,
                |card| card,
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        background_gpu.use_no_gpu_from_the_next_batch();
        assert!(
            background_gpu.gpus_opened().is_empty(),
            "开着的卡为零：{:?}",
            background_gpu.gpus_opened()
        );
        assert_eq!(
            drops_of_the_card_at_forty_one.load(std::sync::atomic::Ordering::SeqCst),
            1
        );
        assert_eq!(
            background_gpu.devices_with_total_memory().len(),
            2,
            "卡清单照列关掉的卡：切回 gpu 可以再点"
        );
        let only_the_card_at_forty_one =
            switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&only_the_card_at_forty_one);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            }
        );
        assert_eq!(
            openings_made_so_far(&cards_handed_to_the_opening),
            1,
            "切回之后当没开的卡开"
        );
        assert_eq!(
            participating_devices_of(&background_gpu),
            vec![card_identifier("0000:41:00.0")]
        );
    }

    /// 开卡那一步：每被调一次先往 `opening_begun` 送一声，等用例放开（至多 20 秒）之后交回开不成。
    fn opening_that_does_not_open_once_released(
        opening_begun: std::sync::mpsc::Sender<()>,
        opening_released: std::sync::mpsc::Receiver<()>,
    ) -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        move |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
            // 收的那一头在用例里一直拿着。
            let _signal_sent = opening_begun.send(());
            let _released_or_not = opening_released.recv_timeout(Duration::from_secs(20));
            Err(reason_the_card_does_not_open_at_the_switch())
        }
    }

    /// 第 2 条「从 gpu 切到 cpu……丢掉还在开的那一组」：切换点没开的「0000:41:00.0」，头一次开卡在开（等用例放开）时告知一张卡都不参与——
    /// 开卡线程开完这一次就不再开（重开规矩本来还许 3 次），开卡那一步一共只被调一次；开着的卡为零。
    #[test]
    fn no_gpu_taking_part_while_a_card_opens_calls_off_the_reopenings() {
        let (opening_begun, every_opening_begun) = std::sync::mpsc::channel::<()>();
        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_that_does_not_open_once_released(opening_begun, opening_released),
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        every_opening_begun
            .recv_timeout(Duration::from_secs(20))
            .expect("开卡线程 20 秒内开始头一次开卡");
        background_gpu.use_no_gpu_from_the_next_batch();
        release_the_opening
            .send(())
            .expect("开卡那一步还在等放开：收的那一头还在");
        drop(release_the_opening);
        // 开卡线程从头一次开卡之前到开完都锁着这一步：拿到锁就是它开完了。
        drop(
            background_gpu
                .opening_of_a_gpu_at_a_switch
                .lock()
                .expect("开一次卡里的 panic 在锁里就接住了：锁没被毒化"),
        );
        assert_eq!(
            openings_made_so_far(&cards_handed_to_the_opening),
            1,
            "叫停之后不再开"
        );
        assert!(background_gpu.gpus_opened().is_empty(), "开着的卡为零");
    }

    /// 卸载时叫停切换开卡线程：切换点没开的「0000:41:00.0」，头一次开卡在开（等用例放开）时丢掉后台 GPU——开卡线程开完这一次就不再开，开卡那一步一共只被调一次。
    #[test]
    fn the_opening_at_a_switch_under_way_at_the_drop_opens_no_more() {
        let (opening_begun, every_opening_begun) = std::sync::mpsc::channel::<()>();
        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_that_does_not_open_once_released(opening_begun, opening_released),
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        every_opening_begun
            .recv_timeout(Duration::from_secs(20))
            .expect("开卡线程 20 秒内开始头一次开卡");
        let opening_step_kept_by_the_test =
            Arc::clone(&background_gpu.opening_of_a_gpu_at_a_switch);
        drop(background_gpu);
        release_the_opening
            .send(())
            .expect("开卡那一步还在等放开：收的那一头还在");
        drop(release_the_opening);
        // 开卡线程从头一次开卡之前到开完都锁着这一步：拿到锁就是它开完了。
        drop(
            opening_step_kept_by_the_test
                .lock()
                .expect("开一次卡里的 panic 在锁里就接住了：锁没被毒化"),
        );
        assert_eq!(
            openings_made_so_far(&cards_handed_to_the_opening),
            1,
            "卸载之后不再开"
        );
    }

    /// 起不了开卡线程也照重开的规矩（间隔之后再起，至多重开 3 次）：头一次起不了，两批之间那一问到了间隔再起、开成——照常换上，参与的只有「0000:41:00.0」。
    #[test]
    fn the_opening_thread_not_started_once_is_started_again_and_the_switch_takes_effect() {
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_that_always_opens,
                None,
                false,
            );
        background_gpu.refuse_to_start_the_opening_thread_for_tests(1);
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            },
            "再起一次就开成了"
        );
        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 1);
        assert_eq!(
            participating_devices_of(&background_gpu),
            vec![card_identifier("0000:41:00.0")]
        );
    }

    /// 起不了开卡线程次次都起不了：头一次与 3 次再起都不成，那一问答开不成，带头一张要开的卡与 4 个原因（1 + 重开次数，每次一个），开卡那一步一次都没被调。
    #[test]
    fn the_opening_thread_never_started_is_reported_with_one_cause_for_each_start() {
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_that_always_opens,
                None,
                false,
            );
        background_gpu.refuse_to_start_the_opening_thread_for_tests(u32::MAX);
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        assert!(
            matches!(
                &answered,
                LatestSettingsHandedIn::NotOpened(not_opened)
                    if not_opened.device == card_identifier("0000:41:00.0")
                        && not_opened.causes_of_each_opening.len() == 4
            ),
            "1 + 重开次数个原因：{answered:?}"
        );
        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 0);
    }

    /// 开卡里 panic 了算那一次没开成、照重开：切换点到的「0000:41:00.0」头一次开卡 panic、第二次开成——照常换上，开卡那一步一共被调 2 次。
    #[test]
    fn an_opening_panicking_once_is_reopened_and_the_switch_takes_effect() {
        let openings_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let openings_counted = Arc::clone(&openings_made);
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                    let opening =
                        openings_counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    assert!(opening > 1, "替身：头一次开卡 panic");
                    opening_that_always_opens(
                        discrete_gpu,
                        memory_limit,
                        wait_limit,
                        cost_at_the_opening,
                    )
                },
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            }
        );
        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 2);
        assert_eq!(
            participating_devices_of(&background_gpu),
            vec![card_identifier("0000:41:00.0")]
        );
    }

    /// 开卡次次 panic：重开到上限之后那一问答开不成，带 4 个原因，每个都带着 panic 的原话；两批之间那一问不再 panic。
    #[test]
    fn an_opening_panicking_every_time_is_reported_after_the_reopenings_with_every_cause() {
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                    panic!("替身：开卡次次 panic")
                },
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        assert_eq!(
            answered,
            LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
                device: card_identifier("0000:41:00.0"),
                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                    "0000:41:00.0"
                ),
                causes_of_each_opening: vec![
                    GpuFailureCause::OpeningPanicked {
                        said_by_the_panic: "替身：开卡次次 panic".to_string(),
                    };
                    4
                ],
            })
        );
        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 4);
    }

    /// 同一张卡前一次开卡没回来就不再起下一次（算这一次又没开成、照间隔往下数）：切换点到的「0000:41:00.0」开卡那一步是产品那一份
    /// （`opening_of_a_card_within_the_limit`，一次至多等 50 毫秒），起这张卡那一件挂着直到用例放开——头一次到点算没开成，之后 3 次重开都因前一次没回来不起，
    /// 起卡那一件一共只被起一次（同一张卡上同时在开的线程至多一条），那一问答开不成、4 个原因。
    #[test]
    fn an_earlier_opening_of_the_card_not_returned_keeps_the_reopenings_from_starting_another() {
        const OPENING_LIMIT_OF_THE_TEST: Duration = Duration::from_millis(50);
        let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
        let start_released = Arc::new(Mutex::new(start_released));
        let starts_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let starts_counted = Arc::clone(&starts_made);
        let opening_at_the_switch = opening_of_a_card_within_the_limit(
            OPENING_LIMIT_OF_THE_TEST,
            move |discrete_gpu: &DiscreteGpuOnThisHost,
                  memory_limit,
                  _wait_limit,
                  bytes_leaked_by_earlier_mounts| {
                let card = CardComputedOnTheCpuForTests::of_the_address(
                    &discrete_gpu.pci_bus_address,
                    memory_limit.map(|memory_limit| memory_limit.0),
                    bytes_leaked_by_earlier_mounts.0,
                );
                let start_released_seen_by_the_start = Arc::clone(&start_released);
                let starts_counted_by_the_start = Arc::clone(&starts_counted);
                move || {
                    starts_counted_by_the_start.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let _released_or_not = start_released_seen_by_the_start
                        .lock()
                        .expect("用例没在别处 panic 过")
                        .recv_timeout(Duration::from_secs(20));
                    Ok(card)
                }
            },
            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            OpeningsNotReturnedInThisProcess::with_none_not_returned(),
        );
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_at_the_switch,
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        let starts_while_the_first_hangs = starts_made.load(std::sync::atomic::Ordering::SeqCst);
        release_the_start
            .send(())
            .expect("起卡那一件还在等放开：收的那一头还在");
        assert_eq!(
            starts_while_the_first_hangs, 1,
            "前一次没回来：同一张卡上不再起"
        );
        assert_eq!(
            answered,
            LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
                device: card_identifier("0000:41:00.0"),
                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                    "0000:41:00.0"
                ),
                causes_of_each_opening: vec![
                    GpuFailureCause::WaitForTheGpuExceededTheLimit {
                        wait_limit: OPENING_LIMIT_OF_THE_TEST,
                    },
                    opening_not_started_while_an_earlier_one_has_not_returned(),
                    opening_not_started_while_an_earlier_one_has_not_returned(),
                    opening_not_started_while_an_earlier_one_has_not_returned(),
                ],
            })
        );
    }

    /// 切换关卡时有提交还在跑的卡照卸载同一个规矩泄漏，另累计进对外读得到的那一份：「0000:41:00.0」上记着一次等超过上限、还在跑的提交（占 600 字节），
    /// 切换不再点它——切换关卡的泄漏报告里读得到它与 600 字节。
    #[test]
    fn the_card_leaked_when_closed_at_a_switch_is_read_from_the_leaks_of_the_switches() {
        let (mut background_gpu, _cards_handed_to_the_opening, _drops_of_the_card_at_forty_one) =
            background_gpu_with_both_cards_opened_at_the_mount(
                opening_that_always_opens,
                |card| card.with_a_submission_running_past_its_wait(600),
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        let leaks_of_the_switches = background_gpu.gpus_leaked_when_closed_at_a_switch();
        assert!(leaks_of_the_switches.gpus_leaked().is_empty());
        let only_the_card_at_one =
            switch_naming_one_card_admitted_against(&background_gpu, "0000:01:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&only_the_card_at_one);
        assert_eq!(
            leaks_of_the_switches.gpus_leaked(),
            vec![GpuLeakedWhenDropped {
                device: card_identifier("0000:41:00.0"),
                bytes_held_on_the_device: GpuMemoryInBytes(600),
                bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
            }]
        );
    }

    /// 产品那一份实现（两份核那一层是外壳）在两批之间那一问里不等开卡：切换点到的「0000:41:00.0」在开（等用例放开），核心层那一问照常交回、答还在开卡。
    #[test]
    fn the_product_implementation_asked_between_two_batches_does_not_wait_for_the_opening() {
        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_that_opens_once_released(opening_released),
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let mut implementation =
            crate::background_gpu_compute::BackgroundGpuComputeThroughBothKernels::with_no_entry_uncertain(
                background_gpu,
            );
        let answered = singlefs_core::background_compute::BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
            &mut implementation,
        );
        release_the_opening
            .send(())
            .expect("开卡那一步还在等放开：收的那一头还在");
        assert_eq!(
            answered,
            singlefs_core::background_compute::LatestGpuSettingsHandedIn::StillOpeningTheirCards
        );
    }

    /// 外壳上：校验和那一段一条 1000 字节的单元，头一次交等超过上限，重交之前等前一次那一等又超时 2 回、第 3 回等到再交、做完——
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

    /// 外壳上：同一进程里两次挂载同时开着、都在同一张卡上各有一次等超过上限还在跑的提交（先开的 600 字节、后开的 300 字节，
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
                    memory_leaked.bytes_leaked_on(&card).0,
                )
                .with_a_submission_running_past_its_wait(600)],
                &memory_leaked,
            );
        let (mount_leaking_three_hundred_bytes, _present_of_the_mount_leaking_three_hundred_bytes) =
            background_gpu_over_cards_on_the_cpu(
                vec![CardComputedOnTheCpuForTests::of_the_address(
                    "0000:01:00.0",
                    None,
                    memory_leaked.bytes_leaked_on(&card).0,
                )
                .with_a_submission_running_past_its_wait(300)],
                &memory_leaked,
            );
        drop(mount_leaking_six_hundred_bytes);
        drop(mount_leaking_three_hundred_bytes);
        assert_eq!(
            memory_leaked.bytes_leaked_on(&card),
            GpuMemoryInBytes(900),
            "两次各泄漏 600、300 字节：记 900，不让后丢的盖掉先丢的"
        );
        assert_eq!(
            CardComputedOnTheCpuForTests::of_the_address(
                "0000:01:00.0",
                Some(1 << 20),
                memory_leaked.bytes_leaked_on(&card).0,
            )
            .budget
            .memory_account()
            .bytes_held(),
            GpuMemoryInBytes(900),
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
        assert_eq!(memory_leaked.bytes_leaked_on(&card), GpuMemoryInBytes(600));
        let (later_mount, _later_present) = background_gpu_over_cards_on_the_cpu(
            vec![CardComputedOnTheCpuForTests::of_the_address(
                "0000:01:00.0",
                None,
                memory_leaked.bytes_leaked_on(&card).0,
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
                bytes_held_on_the_device: GpuMemoryInBytes(900),
                bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(600),
            }]),
            "后一次挂载丢掉那一刻占着 900，其中 600 是开卡时已算上的"
        );
        assert_eq!(
            memory_leaked.bytes_leaked_on(&card),
            GpuMemoryInBytes(900),
            "进程里只加这一次自己的 300"
        );
    }

    /// 按卡的顺序填满那几条用例里的单元：每个 100 字节。
    const BYTES_OF_A_UNIT_IN_THE_FILL: usize = 100;

    /// 校验和那一段一次交得下 `unit_count` 个 [`BYTES_OF_A_UNIT_IN_THE_FILL`] 字节的单元、多一个就交不下的显存上限（`CHECKSUM_CHUNK_FOOTPRINT` 算的）。
    fn memory_limit_holding_units_in_one_submission(unit_count: u64) -> u64 {
        let bytes_of_a_unit = u64::try_from(BYTES_OF_A_UNIT_IN_THE_FILL).expect("小数");
        CHECKSUM_CHUNK_FOOTPRINT
            .bytes_held_by_a_chunk(
                EntryCountOfAChunk(unit_count)
                    .input_bytes_of_the_entries(InputLengthOnTheHostInBytes(bytes_of_a_unit)),
                EntryCountOfAChunk(unit_count),
            )
            .0
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

    /// 按卡的顺序填满：三张卡各 1 GiB、一批 12 个单元一次交得下——只有排在前面的
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

    /// 同一条的饱和：三张卡的上限各一次只交得下 8 个单元，一批 30 个——前两张各填满 8 个、交一次，
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

    /// 一组设置给「0000:01:00.0」的上限连最小的一片都装不下，它被晾下、
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
        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
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

    /// 第 4 条「建缓冲显存不足、切小重切时，这一段在这张卡上已做完的片不留，照新的切法整段从头重交」：
    /// 一张卡上限 3000 字节、卡上只空闲 700 字节，三个单元 100、600、600 字节一次交得下（连同暂存 2848 字节）而建不成——压到 1424 字节重切成三段
    /// （连同暂存 304、1304、1304），头一段交成（这一次的结果是坏的：替身把它的判定翻了），第二段又建不成——压到 652 字节，照新切法整段从头重交七段
    /// （600 字节的单元切成 272、272、56），
    /// 头一段交成的那一次的结果不用：判定与 CPU 参照逐条相同；这张卡上交成 8 次（1 + 7），这一段记下交成 8 次、8 片、重切 2 次。
    #[test]
    fn the_recut_after_running_out_of_memory_resubmits_the_whole_share_without_keeping_the_pieces_already_done(
    ) {
        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", Some(3000), 0);
        card.free_bytes = Some(700);
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

    /// 外壳上：一个单元头一次交等超过上限，之后重交之前那一等连着 3 回等不到、一次都没重交——
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
                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                    "0000:01:00.0"
                ),
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

        fn readback_of_one_kernel(
            &self,
            outcome_bytes: GpuMemoryInBytes,
        ) -> BufferInMemoryForTests {
            self.buffer_of(outcome_bytes.0)
        }

        fn bytes_of_the_buffer(&self, buffer: &BufferInMemoryForTests) -> GpuMemoryInBytes {
            GpuMemoryInBytes(buffer.bytes_requested)
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
    /// 这一段记在账上的字节：自己那几样映射着建、连同暂存，加两份回读暂存。
    const BYTES_HELD_BY_THE_CHUNK_ON_THE_ACCOUNT: GpuMemoryInBytes =
        COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
            .bytes_of_the_copies(GpuMemoryInBytes(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK))
            .plus(GpuMemoryInBytes(2 * OUTCOME_BYTES_OF_THE_CHUNK));

    /// 卡上空闲的装得下这一段自己那几样缓冲，
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
            BYTES_HELD_BY_THE_CHUNK_ON_THE_ACCOUNT,
            GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
            || [device.buffer_of(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK)],
        );
        assert!(
            matches!(
                created,
                Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers {
                    bytes_held_by_the_chunk,
                    ..
                }) if bytes_held_by_the_chunk == BYTES_HELD_BY_THE_CHUNK_ON_THE_ACCOUNT
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
            BYTES_HELD_BY_THE_CHUNK_ON_THE_ACCOUNT,
            GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
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

    /// 这张卡的上限与卡上空闲的都是 2500 字节，这一段记在账上
    /// 2128 字节（这一段自己那几样 1000 连同暂存 2000、两份回读暂存各 64），建的时候多建了一块 300 字节的缓冲——连同暂存卡上真占的到了 2728 字节，比账多：
    /// 账核真建的缓冲字节，当场断言，不带着比账多的显存往下交。
    #[test]
    #[should_panic(expected = "这一段真建的缓冲合起来的字节数与记在这张卡账上的不等")]
    fn the_chunk_creating_one_buffer_more_than_it_holds_on_the_account_panics() {
        const BYTES_OF_THE_EXTRA_BUFFER: u64 = 300;
        let memory_limit_in_bytes = 2500;
        let device = DeviceOfAChunkInMemoryForTests::with_free_bytes(memory_limit_in_bytes);
        let bytes_held_by_the_chunk = BYTES_HELD_BY_THE_CHUNK_ON_THE_ACCOUNT;
        let mut account = GpuMemoryAccount::<u32>::with_nothing_held(Some(GpuMemoryInBytes(
            memory_limit_in_bytes,
        )));
        account.hold(bytes_held_by_the_chunk);
        let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
            &device,
            Duration::from_secs(1),
            bytes_held_by_the_chunk,
            GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
            || {
                [
                    device.buffer_of(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK),
                    device.buffer_of(BYTES_OF_THE_EXTRA_BUFFER),
                ]
            },
        );
        // 走到这里就是账没核真建的缓冲：卡上真占的峰值比账上的多，用例照「峰值不超账」红在这里。
        assert!(
            device.peak_bytes_held.get() <= account.bytes_held().0,
            "卡上真占的峰值 {} 字节超过了账上的 {} 字节（{:?}）",
            device.peak_bytes_held.get(),
            account.bytes_held().0,
            created.as_ref().map(|_buffers| "建成了")
        );
    }

    /// 地址空间压住之后这一段自己的那份缓冲要的字节（`noop_chunk_buffer_with_contents_short` 取 128 MiB）：比压住之后留的那一截
    /// [`ROOM_LEFT_IN_THE_CAPPED_ADDRESS_SPACE`] 大，noop 后端在主机内存里 `try_reserve_exact` 这么多就失败、wgpu 报显存不足；
    /// 又在 `wgpu::Limits::default()` 的单个缓冲上限（256 MiB）之内，wgpu 不先报校验错。
    const BYTES_OF_THE_BUFFER_BEYOND_THE_CAPPED_ADDRESS_SPACE: u64 = 128 << 20;
    /// 压地址空间上限时比此刻的 VmSize 多留的那一截（取 32 MiB）：两份 64 字节的回读暂存、等错误范围、打印与 libtest 收尾用得下。
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
            COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
                .bytes_of_the_copies(GpuMemoryInBytes(
                    BYTES_OF_THE_BUFFER_BEYOND_THE_CAPPED_ADDRESS_SPACE,
                ))
                .plus(GpuMemoryInBytes(2 * OUTCOME_BYTES_OF_THE_CHUNK));
        let created = with_the_address_space_capped_beyond_the_present_size(
            ROOM_LEFT_IN_THE_CAPPED_ADDRESS_SPACE,
            || {
                buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
                    &device,
                    Duration::from_secs(10),
                    bytes_held_by_the_chunk_on_the_account,
                    GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
                    || {
                        [buffer_mapped_at_creation_within_the_scope_of_the_caller(
                            &device,
                            GpuMemoryInBytes(BYTES_OF_THE_BUFFER_BEYOND_THE_CAPPED_ADDRESS_SPACE),
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

    /// 用 `noop_chunk_buffer_with_contents_short` 的造法：noop 后端在主机内存里建缓冲，要真建不成就得压
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
            COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
                .bytes_of_the_copies(GpuMemoryInBytes(8))
                .plus(GpuMemoryInBytes(2 * OUTCOME_BYTES_OF_THE_CHUNK)),
            GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
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

    /// 开卡取实例的替身：交回一个不开任何后端的实例（枚举交回空列表，不碰 GPU），记下开卡取了它几次。
    struct VulkanInstanceCountingTheOpenings {
        instance_without_any_backend: VulkanInstanceOfThisProcess,
        openings_that_took_it: Cell<usize>,
    }

    impl VulkanInstanceHandedToTheOpening for VulkanInstanceCountingTheOpenings {
        fn vulkan_instance_for_this_opening(&self) -> VulkanInstanceOfThisProcess {
            self.openings_that_took_it
                .set(self.openings_that_took_it.get() + 1);
            self.instance_without_any_backend.clone()
        }
    }

    /// 同一个进程里开卡几次的轮数：开卡每次另建实例时取交来的那一份的次数是 0，与它不等。
    const OPENINGS_ON_THE_HANDED_OVER_INSTANCE: usize = 3;

    fn selection_naming_the_card_at(pci_bus_address: &str) -> BackgroundGpuSelection {
        BackgroundGpuSelection {
            background_gpu_devices: Some(vec![card_identifier(pci_bus_address)]),
            background_gpu_memory_limit: None,
            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        }
    }

    /// 同一个进程里反复开卡，每一次都从调用方交来的那一份取实例、不另建：替身记下的次数等于开卡的次数。替身的实例不开后端、枚举是空的，
    /// 点名的卡每一次都拒成「不在这一次枚举的清单里」、枚举到的适配器一个都没有（不要卡，也不建 Vulkan 实例）。
    #[test]
    fn every_opening_in_one_process_takes_the_vulkan_instance_handed_over_instead_of_creating_one()
    {
        let vulkan_instance = VulkanInstanceCountingTheOpenings {
            instance_without_any_backend: VulkanInstanceOfThisProcess::without_any_backend(),
            openings_that_took_it: Cell::new(0),
        };
        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
        let selection = selection_naming_the_card_at("0000:ff:1f.7");
        for opening_round in 0..OPENINGS_ON_THE_HANDED_OVER_INSTANCE {
            let refused = open_background_gpu_with_the_memory_leaked_in_this_process(
                &selection,
                &memory_leaked,
                &OpeningsNotReturnedInThisProcess::with_none_not_returned(),
                &vulkan_instance,
                |_gpus_to_open| {},
            )
            .err();
            assert_eq!(
                refused,
                Some(
                    BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(
                        NamedGpuDeviceNotOnTheListOfThisEnumeration {
                            device: card_identifier("0000:ff:1f.7"),
                            adapters_in_the_enumeration: Vec::new(),
                            unnamed_left_off_the_list: Vec::new(),
                        }
                    )
                ),
                "第 {opening_round} 轮（从 0 数）开卡在交来的不开后端的实例上枚举，一个适配器都没有，点名的卡不在清单里"
            );
        }
        assert_eq!(
            vulkan_instance.openings_that_took_it.get(),
            OPENINGS_ON_THE_HANDED_OVER_INSTANCE,
            "开了 {OPENINGS_ON_THE_HANDED_OVER_INSTANCE} 次卡，每一次都取交来的那一份实例（不另建）"
        );
    }

    /// 点名一张卡开包时的那一次枚举里一张独显都没有（只有软件 Vulkan）。
    fn enumeration_with_only_the_software_vulkan() -> Vec<EnumeratedAdapterDescription> {
        vec![EnumeratedAdapterDescription {
            name: "llvmpipe software rasterizer".to_string(),
            device_kind: AdapterDeviceKind::SoftwareRenderingOnTheCpu,
            pci_bus_address: String::new(),
            device_local_memory: None,
        }]
    }

    /// 点名的卡这一次枚举里根本没有：拒绝里列出枚举到的那一个软件 Vulkan，没有一张独显被留在清单外。
    #[test]
    fn naming_a_card_absent_from_the_enumeration_is_refused_listing_the_adapters_seen_and_none_left_off(
    ) {
        assert_eq!(
            discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
                &selection_naming_the_card_at("0000:01:00.0"),
                &enumeration_with_only_the_software_vulkan(),
            ),
            Err(
                BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(
                    NamedGpuDeviceNotOnTheListOfThisEnumeration {
                        device: card_identifier("0000:01:00.0"),
                        adapters_in_the_enumeration: vec![AdapterSeenInTheEnumeration {
                            device_kind: AdapterDeviceKind::SoftwareRenderingOnTheCpu,
                            pci_bus_address: String::new(),
                        }],
                        unnamed_left_off_the_list: Vec::new(),
                    }
                )
            ),
            "枚举里没有这张卡：只列出枚举到的软件 Vulkan，清单外一张都没有"
        );
    }

    /// 枚举里有一张独显而它没报总线地址：拒绝里列出这张独显（地址是空串）与它被留在清单外的原因，与「枚举里没有」分得开。
    #[test]
    fn naming_a_card_while_the_discrete_gpu_reports_no_bus_address_is_refused_listing_that_gpu_left_off(
    ) {
        let mut adapters = vec![EnumeratedAdapterDescription {
            name: "discrete graphics without an address".to_string(),
            device_kind: AdapterDeviceKind::DiscreteGpu,
            pci_bus_address: String::new(),
            device_local_memory: Some(GpuMemoryInBytes(EIGHT_GIBIBYTES)),
        }];
        adapters.extend(enumeration_with_only_the_software_vulkan());
        assert_eq!(
            discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
                &selection_naming_the_card_at("0000:01:00.0"),
                &adapters,
            ),
            Err(
                BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(
                    NamedGpuDeviceNotOnTheListOfThisEnumeration {
                        device: card_identifier("0000:01:00.0"),
                        adapters_in_the_enumeration: vec![
                            AdapterSeenInTheEnumeration {
                                device_kind: AdapterDeviceKind::DiscreteGpu,
                                pci_bus_address: String::new(),
                            },
                            AdapterSeenInTheEnumeration {
                                device_kind: AdapterDeviceKind::SoftwareRenderingOnTheCpu,
                                pci_bus_address: String::new(),
                            },
                        ],
                        unnamed_left_off_the_list: vec![
                            DiscreteGpuListRefused::DiscreteGpuWithoutAPciBusAddressIsNotSupported {
                                name: "discrete graphics without an address".to_string(),
                            }
                        ],
                    }
                )
            ),
            "有独显而它没报总线地址：列出这张独显（地址空串）与它没收进清单的原因"
        );
    }

    /// 核心层别的拒照原样交回：一张独显都没有、也没点名时是「GPU 不在」。
    #[test]
    fn no_discrete_gpu_without_named_cards_is_refused_as_gpu_not_present_through_the_settings() {
        assert_eq!(
            discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
                &BackgroundGpuSelection {
                    background_gpu_devices: None,
                    background_gpu_memory_limit: None,
                    background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                },
                &enumeration_with_only_the_software_vulkan(),
            ),
            Err(BackgroundGpuOpenRefused::SettingsRefused(
                BackgroundComputeSettingsRefused::GpuNotPresent
            )),
            "没点名、一张独显都没有：照核心层的「GPU 不在」交回"
        );
    }

    /// 产品对「开卡取实例」的实现交回的是调用方手里的那一个实例，不是另建的一个：两边的 wgpu 实例相等
    /// （wgpu 30.0.1 比较两个实例比的是内部那一份 `Arc` 的地址）。用不开后端的实例，不要卡、也不碰 Vulkan。
    /// 对照：各自另建的两个实例互不相等，比较分得出是不是同一个。
    #[test]
    fn the_product_hands_the_opening_the_instance_it_holds_instead_of_a_new_one() {
        let held = VulkanInstanceOfThisProcess::without_any_backend();
        let handed = held.vulkan_instance_for_this_opening();
        assert!(
            handed.instance == held.instance,
            "交给开卡的是调用方手里的那一个实例"
        );
        let created_separately = VulkanInstanceOfThisProcess::without_any_backend();
        assert!(
            created_separately.instance != held.instance,
            "另建的实例与手里的不是同一个：比较分得出来"
        );
    }

    /// 等的上限给 0 是配错，出路是改参数：开卡在枚举之前就拒成核心层的 `GpuWaitLimitOfZero`，没点名、点名一张、空列表三种选卡形态都一样。
    /// 实例不开后端（不要卡、也不碰 Vulkan）：先枚举的话，上限 0 在枚举那一步就拒成 `AdapterEnumerationFailed`，这条断言红。
    #[test]
    fn wait_limit_of_zero_is_refused_as_the_settings_member_before_the_enumeration_for_every_form_of_the_selection(
    ) {
        let vulkan_instance = VulkanInstanceOfThisProcess::without_any_backend();
        for (form_of_the_selection, background_gpu_devices) in [
            ("没点名", None),
            ("点名一张", Some(vec![card_identifier("0000:ff:1f.7")])),
            ("空列表", Some(Vec::new())),
        ] {
            let refused = open_background_gpu(
                &BackgroundGpuSelection {
                    background_gpu_devices,
                    background_gpu_memory_limit: None,
                    background_gpu_wait_limit: Duration::ZERO,
                },
                &vulkan_instance,
            )
            .err();
            assert_eq!(
                refused,
                Some(BackgroundGpuOpenRefused::SettingsRefused(
                    BackgroundComputeSettingsRefused::GpuWaitLimitOfZero
                )),
                "{form_of_the_selection}：上限 0 拒成设置配错，不是枚举没成"
            );
        }
    }

    // ── 开不成之后生效的照旧是切换之前那一组、挂载那一刻开卡 panic 照重开、一张卡一次挂载里连着试满、还没回来的开卡管到整个进程、
    // 读得到切换关卡的泄漏、被盖掉的切换（D26（后台整理与放置回收） 已定项 13 第 2、4 条）──

    /// 切换开卡那一步：「0000:01:00.0」等用例放开（至多 20 秒）再开成，别的卡每一次都开不成。
    fn opening_the_card_at_one_once_released_and_no_other(
        opening_released: std::sync::mpsc::Receiver<()>,
    ) -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
            if discrete_gpu.pci_bus_address != "0000:01:00.0" {
                return Err(reason_the_card_does_not_open_at_the_switch());
            }
            opening_released
                .recv_timeout(Duration::from_secs(20))
                .map_err(|_not_released| reason_the_card_does_not_open_at_the_switch())?;
            opening_that_always_opens(discrete_gpu, memory_limit, wait_limit, cost_at_the_opening)
        }
    }

    /// 一段历史在切换点的卡开不成之后的读数：开不成之后开着的卡、参与的卡，与再交同一组那一刻的答。
    struct ReadingsOfTheSwitchNotOpenedAndHandedInAgain {
        answered_once_not_opened: LatestSettingsHandedIn,
        gpus_opened_once_not_opened: Vec<GpuDeviceIdentifier>,
        participating_once_not_opened: Vec<GpuDeviceIdentifier>,
        answered_when_handed_in_again: LatestSettingsHandedIn,
        openings: usize,
    }

    /// 挂载开着「0000:01:00.0」，核心层交来点它的那一组；`switch_to_the_cpu_first` 时先告知一张卡都不参与；切换只点没开的「0000:41:00.0」，
    /// 头一次与 3 次重开都开不成；再交同一组。
    fn readings_of_the_switch_not_opened_and_handed_in_again(
        switch_to_the_cpu_first: bool,
    ) -> ReadingsOfTheSwitchNotOpenedAndHandedInAgain {
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                    Err(reason_the_card_does_not_open_at_the_switch())
                },
                None,
                false,
            );
        let at_the_mount =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:01:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&at_the_mount);
        if switch_to_the_cpu_first {
            background_gpu.use_no_gpu_from_the_next_batch();
        }
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered_once_not_opened = background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds,
            );
        let gpus_opened_once_not_opened = background_gpu.gpus_opened();
        let participating_once_not_opened = participating_devices_of(&background_gpu);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered_when_handed_in_again = background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::NotAtAll);
        ReadingsOfTheSwitchNotOpenedAndHandedInAgain {
            answered_once_not_opened,
            gpus_opened_once_not_opened,
            participating_once_not_opened,
            answered_when_handed_in_again,
            openings: openings_made_so_far(&cards_handed_to_the_opening),
        }
    }

    fn not_opened_with_the_openings_of_the_card_at_forty_one_used_up() -> LatestSettingsHandedIn {
        LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
            device: card_identifier("0000:41:00.0"),
            pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                "0000:41:00.0",
            ),
            causes_of_each_opening: vec![GpuFailureCause::OpeningsInThisMountUsedUp {
                failed_openings: 4,
            }],
        })
    }

    /// 先切到 `cpu` 的那一臂：开不成之后一张卡都没开、没参与（核心层生效的照旧是 `cpu`，不交活）；再交同一组，这张卡已连着试满，当场答开不成、不再开。
    #[test]
    fn a_switch_not_opened_after_the_cpu_leaves_no_card_open_and_the_same_settings_again_are_answered_not_opened_at_once(
    ) {
        let readings = readings_of_the_switch_not_opened_and_handed_in_again(true);
        assert!(matches!(
            readings.answered_once_not_opened,
            LatestSettingsHandedIn::NotOpened(_)
        ));
        assert!(readings.gpus_opened_once_not_opened.is_empty());
        assert!(readings.participating_once_not_opened.is_empty());
        assert_eq!(
            readings.answered_when_handed_in_again,
            not_opened_with_the_openings_of_the_card_at_forty_one_used_up()
        );
        assert_eq!(readings.openings, 4);
    }

    /// 不先切到 `cpu` 的那一臂：开不成之后参与的照旧是切换之前那张「0000:01:00.0」，与核心层照旧生效的那一组是同一张；再交同一组当场答开不成、不再开。
    #[test]
    fn a_switch_not_opened_on_the_gpu_keeps_the_card_before_the_switch_taking_part() {
        let readings = readings_of_the_switch_not_opened_and_handed_in_again(false);
        assert!(matches!(
            readings.answered_once_not_opened,
            LatestSettingsHandedIn::NotOpened(_)
        ));
        assert_eq!(
            readings.gpus_opened_once_not_opened,
            vec![card_identifier("0000:01:00.0")]
        );
        assert_eq!(
            readings.participating_once_not_opened,
            vec![card_identifier("0000:01:00.0")],
            "参与的照旧是切换之前那一组"
        );
        assert_eq!(
            readings.answered_when_handed_in_again,
            not_opened_with_the_openings_of_the_card_at_forty_one_used_up()
        );
        assert_eq!(readings.openings, 4);
    }

    /// 一张替身独显（没开）。
    fn discrete_gpu_not_opened_at(pci_bus_address: &str) -> DiscreteGpuOnThisHost {
        DiscreteGpuOnThisHost {
            position_in_the_enumeration: 0,
            identifier: card_identifier(pci_bus_address),
            pci_bus_address: pci_bus_address.to_string(),
            name: format!("cpu card at {pci_bus_address} not opened yet"),
            total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
        }
    }

    /// 产品那一份开一张卡那一步（`opening_of_a_card_within_the_limit`，一次至多等 20 秒），起卡那一件头一次 panic、之后开成替身卡；
    /// 起卡那一件每被起一次往 `starts_made` 加一。
    fn opening_panicking_at_its_first_start(
        starts_made: &Arc<std::sync::atomic::AtomicU32>,
    ) -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        let starts_counted = Arc::clone(starts_made);
        opening_of_a_card_within_the_limit(
            Duration::from_secs(20),
            move |discrete_gpu: &DiscreteGpuOnThisHost,
                  memory_limit,
                  _wait_limit,
                  bytes_leaked_by_earlier_mounts| {
                let card = CardComputedOnTheCpuForTests::of_the_address(
                    &discrete_gpu.pci_bus_address,
                    memory_limit.map(|memory_limit| memory_limit.0),
                    bytes_leaked_by_earlier_mounts.0,
                );
                let starts_counted_by_the_start = Arc::clone(&starts_counted);
                move || {
                    let starts_before = starts_counted_by_the_start
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    assert!(starts_before > 0, "替身：起卡那一件头一次 panic");
                    Ok(card)
                }
            },
            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            OpeningsNotReturnedInThisProcess::with_none_not_returned(),
        )
    }

    /// 第 4 条「开卡线程 panic 都算这张卡这一次没开成，照重开」用于挂载那一刻：开一张卡那一步（产品那一份）头一次 panic——挂载照重开、第二次开成，
    /// 不拒挂载，起卡那一件一共被起 2 次。
    #[test]
    fn an_opening_panicking_at_the_mount_is_reopened_and_the_mount_is_not_refused() {
        let discrete_gpu = discrete_gpu_not_opened_at("0000:41:00.0");
        let starts_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let mut opening_at_the_mount = opening_panicking_at_its_first_start(&starts_made);
        let mount = gpus_opened_at_the_mount(
            vec![discrete_gpu.clone()],
            &[ParticipatingGpu {
                discrete_gpu,
                memory_limit: None,
            }],
            bytes_of_the_smallest_piece_every_participating_gpu_holds(),
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            ReopeningsAndPciStateReadingOfAGpuNotOpened {
                reopenings: REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                pci_state_reader: &pci_state_reader_under_a_root_not_present_for_tests(),
            },
            |_interval: Duration| {},
            |gpu_to_open, memory_limit_at_the_opening, cost_at_the_opening| {
                opening_at_the_mount(
                    gpu_to_open,
                    memory_limit_at_the_opening,
                    BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                    cost_at_the_opening,
                )
            },
        );
        assert!(
            mount.is_ok(),
            "头一次 panic 照重开、第二次开成：{:?}",
            mount.err()
        );
        assert_eq!(starts_made.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    /// 开卡那一步：「0000:41:00.0」第 3 次与第 6 次开成、别的次次开不成；别的卡每一次都开成。各次交来的卡记在交回的那一份里（另由外壳那一层记）。
    fn opening_the_card_at_forty_one_only_the_third_and_sixth_times() -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        let mut openings_of_the_card_at_forty_one: u32 = 0;
        move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
            if discrete_gpu.pci_bus_address == "0000:41:00.0" {
                openings_of_the_card_at_forty_one += 1;
                if openings_of_the_card_at_forty_one != 3 && openings_of_the_card_at_forty_one != 6
                {
                    return Err(reason_the_card_does_not_open_at_the_switch());
                }
            }
            opening_that_always_opens(discrete_gpu, memory_limit, wait_limit, cost_at_the_opening)
        }
    }

    /// 开卡那一步：「0000:41:00.0」次次开不成，对它的第 2 次先往 `second_opening_begun` 送一声、等用例放开（至多 20 秒）；别的卡每一次都开成。
    fn opening_the_card_at_forty_one_never_and_holding_its_second_opening(
        second_opening_begun: std::sync::mpsc::Sender<()>,
        second_opening_released: std::sync::mpsc::Receiver<()>,
    ) -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        let mut openings_of_the_card_at_forty_one: u32 = 0;
        move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
            if discrete_gpu.pci_bus_address == "0000:41:00.0" {
                openings_of_the_card_at_forty_one += 1;
                if openings_of_the_card_at_forty_one == 2 {
                    // 收的那一头在用例里一直拿着。
                    let _signal_sent = second_opening_begun.send(());
                    let _released_or_not =
                        second_opening_released.recv_timeout(Duration::from_secs(20));
                }
                return Err(reason_the_card_does_not_open_at_the_switch());
            }
            opening_that_always_opens(discrete_gpu, memory_limit, wait_limit, cost_at_the_opening)
        }
    }

    /// 开卡那一步对「0000:41:00.0」被调了几次。
    fn openings_of_the_card_at_forty_one(
        cards_handed_to_the_opening: &Arc<Mutex<Vec<String>>>,
    ) -> usize {
        cards_handed_to_the_opening
            .lock()
            .expect("用例没在别处 panic 过")
            .iter()
            .filter(|address| address.as_str() == "0000:41:00.0")
            .count()
    }

    /// 第 4 条「一张卡至多开 4 次」数的是一次挂载里这张卡连着开不成的次数，开成一次就清零：「0000:41:00.0」头一次切换开不成 2 次、第 3 次开成；
    /// 切走关掉之后再点它，又开不成 2 次、这一次的第 3 次（对它是第 6 次）开成——前后两段的开不成不并成 4 次，不当试满，换上。
    #[test]
    fn a_card_opened_between_its_failed_openings_counts_its_failed_openings_afresh() {
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_the_card_at_forty_one_only_the_third_and_sixth_times(),
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        let back_to_the_card_at_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:01:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            },
            "第 3 次开成"
        );
        background_gpu.use_the_settings_from_the_next_batch(&back_to_the_card_at_one);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            },
            "切回「0000:01:00.0」：「0000:41:00.0」关掉"
        );
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            },
            "开成过一次就清零：又开不成 2 次不算试满，第 3 次开成"
        );
        assert_eq!(
            participating_devices_of(&background_gpu),
            vec![card_identifier("0000:41:00.0")]
        );
        assert_eq!(
            openings_of_the_card_at_forty_one(&cards_handed_to_the_opening),
            6
        );
    }

    /// 连着开不成的次数按卡数、跨切换接着数：「0000:41:00.0」头一次切换开不成 2 次（第 2 次在开时告知一张卡都不参与，余下的叫停没开）；
    /// 之后「0000:81:00.0」开成，不给「0000:41:00.0」清零；再点「0000:41:00.0」只剩 2 次可开、都开不成——答开不成、报这 2 次的原因；
    /// 再点它不再开，答试满了 4 次，开卡那一步对它一共 4 次。
    #[test]
    fn the_failed_openings_of_a_card_in_a_row_add_up_across_the_switches_and_another_card_opening_does_not_clear_them(
    ) {
        let (second_opening_begun, every_second_opening_begun) = std::sync::mpsc::channel::<()>();
        let (release_the_second_opening, second_opening_released) =
            std::sync::mpsc::channel::<()>();
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_the_card_at_forty_one_never_and_holding_its_second_opening(
                    second_opening_begun,
                    second_opening_released,
                ),
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        let to_the_card_at_eighty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        every_second_opening_begun
            .recv_timeout(Duration::from_secs(20))
            .expect("开卡线程 20 秒内开始对「0000:41:00.0」的第 2 次开卡");
        background_gpu.use_no_gpu_from_the_next_batch();
        release_the_second_opening
            .send(())
            .expect("开卡那一步还在等放开：收的那一头还在");
        drop(
            background_gpu
                .opening_of_a_gpu_at_a_switch
                .lock()
                .expect("开一次卡里的 panic 在那一步就接住了：锁没被毒化"),
        );
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            },
            "「0000:81:00.0」开成"
        );
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
                device: card_identifier("0000:41:00.0"),
                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                    "0000:41:00.0"
                ),
                causes_of_each_opening: vec![reason_the_card_does_not_open_at_the_switch(); 2],
            }),
            "连着开不成 2 次之后只剩 2 次可开"
        );
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            not_opened_with_the_openings_of_the_card_at_forty_one_used_up()
        );
        assert_eq!(
            openings_of_the_card_at_forty_one(&cards_handed_to_the_opening),
            4,
            "试满之后不再开"
        );
    }

    /// 产品那一份开一张卡那一步（`opening_of_a_card_within_the_limit`，一次至多等 50 毫秒）、还没回来的开卡那一张表是 `openings_not_returned`：
    /// 起卡那一件头一次挂着直到用例放开（至多 20 秒），之后当场开成；每被起一次往 `starts_made` 加一、往 `start_begun` 送一声。
    fn opening_held_at_its_first_start_until_released(
        start_released: &Arc<Mutex<std::sync::mpsc::Receiver<()>>>,
        starts_made: &Arc<std::sync::atomic::AtomicU32>,
        start_begun: &std::sync::mpsc::Sender<()>,
        openings_not_returned: &OpeningsNotReturnedInThisProcess,
    ) -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        let start_released = Arc::clone(start_released);
        let starts_counted = Arc::clone(starts_made);
        let start_begun = start_begun.clone();
        opening_of_a_card_within_the_limit(
            Duration::from_millis(50),
            move |discrete_gpu: &DiscreteGpuOnThisHost,
                  memory_limit,
                  _wait_limit,
                  bytes_leaked_by_earlier_mounts| {
                let card = CardComputedOnTheCpuForTests::of_the_address(
                    &discrete_gpu.pci_bus_address,
                    memory_limit.map(|memory_limit| memory_limit.0),
                    bytes_leaked_by_earlier_mounts.0,
                );
                let start_released_seen_by_the_start = Arc::clone(&start_released);
                let starts_counted_by_the_start = Arc::clone(&starts_counted);
                let start_begun_by_the_start = start_begun.clone();
                move || {
                    let starts_before = starts_counted_by_the_start
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    // 收的那一头在用例里一直拿着。
                    let _signal_sent = start_begun_by_the_start.send(());
                    if starts_before == 0 {
                        let _released_or_not = start_released_seen_by_the_start
                            .lock()
                            .expect("用例没在别处 panic 过")
                            .recv_timeout(Duration::from_secs(20));
                    }
                    Ok(card)
                }
            },
            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            openings_not_returned.clone(),
        )
    }

    /// 第 4 条「同一张卡前一次开卡没回来就不再起下一次」管到整个进程：前一次挂载在「0000:41:00.0」上的一次开卡到点之后还挂着，
    /// 下一次挂载（另起一份开卡那一步，共用进程开卡器那一张表）开它不再起，算没开成；起卡那一件一共只被起 1 次。
    #[test]
    fn an_opening_hung_in_an_earlier_mount_keeps_a_later_mount_from_starting_another_on_that_card()
    {
        let discrete_gpu = discrete_gpu_not_opened_at("0000:41:00.0");
        let openings_not_returned = OpeningsNotReturnedInThisProcess::with_none_not_returned();
        let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
        let start_released = Arc::new(Mutex::new(start_released));
        let (start_begun, every_start_begun) = std::sync::mpsc::channel::<()>();
        let starts_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let mut opening_of_the_earlier_mount = opening_held_at_its_first_start_until_released(
            &start_released,
            &starts_made,
            &start_begun,
            &openings_not_returned,
        );
        let mut opening_of_the_later_mount = opening_held_at_its_first_start_until_released(
            &start_released,
            &starts_made,
            &start_begun,
            &openings_not_returned,
        );
        let earlier_mount = opening_of_the_earlier_mount(
            &discrete_gpu,
            None,
            BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            CostAtTheOpening::MeasuredUnderTheMemoryLimit,
        )
        .err();
        every_start_begun
            .recv_timeout(Duration::from_secs(20))
            .expect("前一次挂载的那一次开卡 20 秒内起了");
        let later_mount = opening_of_the_later_mount(
            &discrete_gpu,
            None,
            BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            CostAtTheOpening::MeasuredUnderTheMemoryLimit,
        )
        .err();
        let starts_after_the_later_mount = starts_made.load(std::sync::atomic::Ordering::SeqCst);
        release_the_start
            .send(())
            .expect("起卡那一件还在等放开：收的那一头还在");
        assert!(matches!(
            earlier_mount,
            Some(GpuFailureCause::WaitForTheGpuExceededTheLimit { .. })
        ));
        assert_eq!(
            later_mount,
            Some(GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned),
            "下一次挂载在这张卡上不再起"
        );
        assert_eq!(starts_after_the_later_mount, 1);
    }

    /// 前一次开卡回来了（那条线程跑完）就从那一张表里划掉：同一张卡下一次照起、开成。
    #[test]
    fn an_opening_that_returns_after_its_limit_lets_the_next_opening_of_that_card_start() {
        let discrete_gpu = discrete_gpu_not_opened_at("0000:41:00.0");
        let openings_not_returned = OpeningsNotReturnedInThisProcess::with_none_not_returned();
        let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
        let start_released = Arc::new(Mutex::new(start_released));
        let (start_begun, every_start_begun) = std::sync::mpsc::channel::<()>();
        let starts_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let mut opening = opening_held_at_its_first_start_until_released(
            &start_released,
            &starts_made,
            &start_begun,
            &openings_not_returned,
        );
        let hung = opening(
            &discrete_gpu,
            None,
            BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            CostAtTheOpening::MeasuredUnderTheMemoryLimit,
        );
        every_start_begun
            .recv_timeout(Duration::from_secs(20))
            .expect("那一次开卡 20 秒内起了");
        release_the_start
            .send(())
            .expect("起卡那一件还在等放开：收的那一头还在");
        let returned_within_twenty_seconds = (0..2000).any(|_poll| {
            if openings_not_returned
                .devices_with_an_opening_not_returned()
                .is_empty()
            {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
            false
        });
        let again = opening(
            &discrete_gpu,
            None,
            BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            CostAtTheOpening::MeasuredUnderTheMemoryLimit,
        );
        assert!(matches!(
            hung.err(),
            Some(GpuFailureCause::WaitForTheGpuExceededTheLimit { .. })
        ));
        assert!(
            returned_within_twenty_seconds,
            "放开之后那一次开卡 20 秒内回来了"
        );
        assert!(again.is_ok(), "回来之后照起：{:?}", again.err());
        assert_eq!(starts_made.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    /// 运行库收的那一份读得到切换关卡泄漏了哪几张：产品开卡器开成之后换成运行库收的样子（`opened_background_gpu_for_the_mount_runtime`），
    /// 「0000:41:00.0」上记着一次还在跑的提交（600 字节），核心层挂载那一组只点「0000:01:00.0」——那一刻关掉「0000:41:00.0」、整张泄漏，
    /// 运行库那一份读出它与 600 字节；卸载（丢掉把手）之后读得到丢掉那一刻泄漏的（一张都没有）。
    #[test]
    fn the_leaks_at_a_switch_are_read_through_what_the_product_opener_hands_to_the_mount_runtime() {
        let (background_gpu, _cards_handed_to_the_opening, _drops_of_the_card_at_forty_one) =
            background_gpu_with_both_cards_opened_at_the_mount(
                opening_that_always_opens,
                |card| card.with_a_submission_running_past_its_wait(600),
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        let only_the_card_at_one = BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: Some(vec![card_identifier("0000:01:00.0")]),
            background_gpu_memory_limit: None,
            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
        };
        let opened = crate::gpu_opener_of_this_process::opened_background_gpu_for_the_mount_runtime(
            crate::background_gpu_compute::background_gpu_compute_of_the_opened_gpu(background_gpu),
        );
        let gpus_leaked = opened
            .gpus_leaked
            .clone()
            .expect("产品开卡器恒给读泄漏的那一份");
        let background_compute =
            singlefs_core::background_compute::BackgroundComputeOfThisMount::of_a_mount(
                singlefs_core::background_compute::admit_background_compute_settings_with_the_gpu_compute(
                    only_the_card_at_one,
                    Some(opened.gpu_compute),
                )
                .expect("卡清单列着「0000:01:00.0」"),
            );
        assert_eq!(
            gpus_leaked.gpus_leaked_when_closed_at_a_switch(),
            vec![singlefs_mount_runtime::GpuLeakedByTheBackgroundGpu {
                device: card_identifier("0000:41:00.0"),
                memory_held_on_the_device: GpuMemoryInBytes(600),
            }]
        );
        assert_eq!(gpus_leaked.gpus_leaked_when_dropped(), None, "还没丢");
        drop(background_compute);
        assert_eq!(
            gpus_leaked.gpus_leaked_when_dropped(),
            Some(Vec::new()),
            "丢掉之后读得到：开着的那张没有还在跑的提交，一张都没泄漏"
        );
    }

    /// 起不了开卡线程、到了间隔而之后又交来了别的一组：照最近那一组起，不为被盖掉的那一组起——开卡那一步只被交「0000:81:00.0」，换上的是它。
    #[test]
    fn the_opening_thread_not_started_for_a_superseded_switch_is_started_for_the_latest_settings() {
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_that_always_opens,
                None,
                false,
            );
        background_gpu.refuse_to_start_the_opening_thread_for_tests(1);
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        let to_the_card_at_eighty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            }
        );
        assert_eq!(
            *cards_handed_to_the_opening
                .lock()
                .expect("用例没在别处 panic 过"),
            vec!["0000:81:00.0".to_string()],
            "只为最近那一组开"
        );
        assert_eq!(
            participating_devices_of(&background_gpu),
            vec![card_identifier("0000:81:00.0")]
        );
    }

    /// 收下为被盖掉的切换开成的卡、最近那一组还要开卡而没点它：收下那一刻就关掉，不开着等最近那一组开完——切换点「0000:41:00.0」（等用例放开），
    /// 又来一次只点「0000:81:00.0」；放开之后「0000:41:00.0」开成、收下，为「0000:81:00.0」开卡那一刻「0000:41:00.0」已析构一次。
    #[test]
    fn the_card_opened_for_a_superseded_switch_is_closed_as_it_is_taken_in_while_the_latest_switch_still_opens(
    ) {
        let (release_the_card_at_forty_one, card_at_forty_one_released) =
            std::sync::mpsc::channel::<()>();
        let drops_of_the_card_at_forty_one = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let drops_noted_by_the_opening = Arc::clone(&drops_of_the_card_at_forty_one);
        let drops_seen_when_the_latest_card_opens = Arc::new(Mutex::new(None));
        let drops_seen_by_the_opening = Arc::clone(&drops_seen_when_the_latest_card_opens);
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                    if discrete_gpu.pci_bus_address == "0000:41:00.0" {
                        card_at_forty_one_released
                            .recv_timeout(Duration::from_secs(20))
                            .map_err(|_not_released| {
                                reason_the_card_does_not_open_at_the_switch()
                            })?;
                        let mut card = opening_that_always_opens(
                            discrete_gpu,
                            memory_limit,
                            wait_limit,
                            cost_at_the_opening,
                        )?;
                        card.drops_counted = Arc::clone(&drops_noted_by_the_opening);
                        return Ok(card);
                    }
                    *drops_seen_by_the_opening
                        .lock()
                        .expect("用例没在别处 panic 过") =
                        Some(drops_noted_by_the_opening.load(std::sync::atomic::Ordering::SeqCst));
                    opening_that_always_opens(
                        discrete_gpu,
                        memory_limit,
                        wait_limit,
                        cost_at_the_opening,
                    )
                },
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        let to_the_card_at_eighty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
        release_the_card_at_forty_one
            .send(())
            .expect("开「0000:41:00.0」那一步还在等放开");
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            }
        );
        assert_eq!(
            *drops_seen_when_the_latest_card_opens
                .lock()
                .expect("用例没在别处 panic 过"),
            Some(1),
            "为「0000:81:00.0」开卡那一刻，为被盖掉的那一组开成的卡已经关掉"
        );
    }

    /// 最近那一组点了两张没开的卡，前一张开成、后一张开不成：这一组没生效，为它开成的那一张关掉，开着的只剩此刻生效那一组的「0000:01:00.0」。
    #[test]
    fn the_card_opened_for_the_latest_switch_is_closed_when_another_card_of_it_does_not_open() {
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                    if discrete_gpu.pci_bus_address == "0000:41:00.0" {
                        return opening_that_always_opens(
                            discrete_gpu,
                            memory_limit,
                            wait_limit,
                            cost_at_the_opening,
                        );
                    }
                    Err(reason_the_card_does_not_open_at_the_switch())
                },
                None,
                false,
            );
        let to_both_cards_not_opened_at_the_mount = switch_naming_the_cards_admitted_against(
            &background_gpu,
            &["0000:41:00.0", "0000:81:00.0"],
            None,
        );
        background_gpu.use_the_settings_from_the_next_batch(&to_both_cards_not_opened_at_the_mount);
        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        assert!(
            matches!(
                &answered,
                LatestSettingsHandedIn::NotOpened(not_opened)
                    if not_opened.device == card_identifier("0000:81:00.0")
            ),
            "{answered:?}"
        );
        assert_eq!(
            background_gpu.gpus_opened(),
            vec![card_identifier("0000:01:00.0")],
            "为没生效的那一组开成的「0000:41:00.0」关掉"
        );
    }

    /// 重开的间隔照产品是 1 分钟（用例不真等它：只用来看「该等的没等、不该等的等了」）。
    const REOPENINGS_ONE_MINUTE_APART: ReopeningsOfAGpuNotOpened = ReopeningsOfAGpuNotOpened {
        interval: Duration::from_secs(60),
        reopenings_at_most: 3,
    };

    /// 起不了开卡线程那一回记在头一张要开的卡上：头一回起不了，之后这张卡次次开不成——连同那一回至多 4 次，开卡那一步只被调 3 次，
    /// 那一问答开不成，原因是起不了线程那一个在前、3 次开不成在后。
    #[test]
    fn the_opening_thread_not_started_once_counts_toward_the_openings_of_the_first_card() {
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_one_card_not_opened_at_the_mount(
                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                    Err(reason_the_card_does_not_open_at_the_switch())
                },
                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        background_gpu.refuse_to_start_the_opening_thread_for_tests(1);
        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        assert!(
            matches!(
                &answered,
                LatestSettingsHandedIn::NotOpened(not_opened)
                    if not_opened.causes_of_each_opening.len() == 4
                        && matches!(
                            not_opened.causes_of_each_opening[0],
                            GpuFailureCause::OpeningThreadNotStarted { .. }
                        )
            ),
            "{answered:?}"
        );
        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 3);
    }

    /// 叫停之后不再睡重开的间隔：间隔是 1 分钟，头一次开卡在开（等用例放开）时告知一张卡都不参与——放开之后开卡线程不睡那 1 分钟，20 秒之内就开完、放开那一步。
    #[test]
    fn no_gpu_taking_part_while_a_card_opens_does_not_sleep_out_the_interval_before_a_reopening() {
        let (opening_begun, every_opening_begun) = std::sync::mpsc::channel::<()>();
        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_one_card_not_opened_at_the_mount(
                opening_that_does_not_open_once_released(opening_begun, opening_released),
                REOPENINGS_ONE_MINUTE_APART,
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        every_opening_begun
            .recv_timeout(Duration::from_secs(20))
            .expect("开卡线程 20 秒内开始头一次开卡");
        background_gpu.use_no_gpu_from_the_next_batch();
        release_the_opening
            .send(())
            .expect("开卡那一步还在等放开：收的那一头还在");
        drop(release_the_opening);
        let opening_step = Arc::clone(&background_gpu.opening_of_a_gpu_at_a_switch);
        let opening_thread_done_within_twenty_seconds = (0..2000).any(|_poll| {
            if opening_step.try_lock().is_ok() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
            false
        });
        assert!(
            opening_thread_done_within_twenty_seconds,
            "叫停之后不睡 1 分钟的间隔"
        );
        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 1);
    }

    /// 起不了开卡线程之后到了间隔才再起：间隔是 1 分钟，头一回起不了，两批之间那一问当场问——不再起，照旧是那一次起不了，开卡那一步一次都没被调。
    #[test]
    fn the_opening_thread_not_started_is_not_started_again_before_the_interval() {
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_one_card_not_opened_at_the_mount(
                opening_that_always_opens,
                REOPENINGS_ONE_MINUTE_APART,
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        background_gpu.refuse_to_start_the_opening_thread_for_tests(1);
        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::NotAtAll
            ),
            LatestSettingsHandedIn::StillOpeningItsCards
        );
        assert!(
            matches!(
                background_gpu.opening_at_a_switch,
                Some(OpeningAtASwitch::ThreadNotStarted { .. })
            ),
            "没到间隔：照旧是那一次起不了"
        );
        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 0);
    }

    /// 告知一张卡都不参与时生效那一组点的卡清掉：之后为被盖掉的一组开成的卡（「0000:01:00.0」），在最近那一组（只点「0000:81:00.0」）开不成时关掉——
    /// 开着的卡为零。
    #[test]
    fn no_gpu_taking_part_forgets_the_cards_of_the_settings_in_effect_so_a_card_opened_later_is_closed(
    ) {
        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_the_card_at_one_once_released_and_no_other(opening_released),
                None,
                false,
            );
        background_gpu.use_no_gpu_from_the_next_batch();
        let to_the_card_at_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:01:00.0"], None);
        let to_the_card_at_eighty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_one);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
        release_the_opening
            .send(())
            .expect("开卡那一步还在等放开：收的那一头还在");
        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        assert!(
            matches!(&answered, LatestSettingsHandedIn::NotOpened(_)),
            "{answered:?}"
        );
        assert!(
            background_gpu.gpus_opened().is_empty(),
            "开着的卡为零：{:?}",
            background_gpu.gpus_opened()
        );
    }

    /// 告知一张卡都不参与时因显存上限不用的卡一并清掉：挂载那一刻「0000:01:00.0」因显存上限不参与，切到 `cpu` 之后不再报它。
    #[test]
    fn no_gpu_taking_part_forgets_the_cards_left_out_under_their_memory_limit() {
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_that_always_opens,
                Some(64),
                true,
            );
        assert!(!background_gpu
            .gpus_left_out_under_their_memory_limit()
            .is_empty());
        background_gpu.use_no_gpu_from_the_next_batch();
        assert!(background_gpu
            .gpus_left_out_under_their_memory_limit()
            .is_empty());
    }

    // ── 切换时量代价先等账上还在跑的那次提交（替身卡的量代价照产品那一份走账）──

    /// 两张替身卡：「0000:01:00.0」量代价照产品走账（`measured_at_the_mount` 为假时开卡时没量过代价）、「0000:41:00.0」照旧。
    fn two_cards_measuring_the_cost_through_the_account(
        measured_at_the_mount: bool,
    ) -> BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests> {
        let mut card_at_one = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        card_at_one.cost_measured_through_the_checksum_chunk = true;
        if !measured_at_the_mount {
            card_at_one.measured_cost = None;
        }
        let card_at_forty_one =
            CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0);
        let (background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card_at_one, card_at_forty_one],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        background_gpu
    }

    /// 历史甲：「0000:01:00.0」开卡时没量过代价、被晾下；切换请回来，量代价那一次提交等超过上限（账上记成还在跑）。
    fn card_at_one_whose_measurement_at_a_switch_timed_out(
    ) -> BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests> {
        let mut background_gpu = two_cards_measuring_the_cost_through_the_account(false);
        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
        let leaving_it_out = settings_admitted_on_the_cards(
            &background_gpu,
            None,
            vec![smallest_piece_bytes - 1, 1 << 30],
        );
        background_gpu.use_the_settings_from_the_next_batch(&leaving_it_out);
        background_gpu.cards_on_this_host[0]
            .submissions_timing_out
            .push_back(true);
        let bringing_it_back =
            settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
        background_gpu.use_the_settings_from_the_next_batch(&bringing_it_back);
        assert!(
            background_gpu.cards_on_this_host[0]
                .budget
                .memory_account()
                .has_a_submission_running_past_its_wait(),
            "前缀：量代价那一次等超过上限，账上记着还在跑"
        );
        background_gpu
    }

    /// 历史乙：「0000:01:00.0」量过代价、参与；一批校验和头一次交等超过上限、之后三回等前一次都没等到（报这张卡用不了，账上照旧记着还在跑），
    /// 切换给它装不下最小一片的上限（晾下）。
    fn card_at_one_left_out_while_its_timed_out_submission_runs(
    ) -> BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests> {
        let mut background_gpu = two_cards_measuring_the_cost_through_the_account(true);
        {
            let card_at_one = &mut background_gpu.cards_on_this_host[0];
            card_at_one.submissions_timing_out = VecDeque::from([true]);
            card_at_one.earlier_submission_waits_timing_out = VecDeque::from([true, true, true]);
        }
        let unit_bytes = unit_bytes_of(1, 1000);
        let batch = background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes));
        assert!(batch.is_err(), "前缀：这一批报这张卡用不了");
        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
        let leaving_it_out = settings_admitted_on_the_cards(
            &background_gpu,
            None,
            vec![smallest_piece_bytes - 1, 1 << 30],
        );
        background_gpu.use_the_settings_from_the_next_batch(&leaving_it_out);
        assert!(
            background_gpu.cards_on_this_host[0]
                .budget
                .memory_account()
                .has_a_submission_running_past_its_wait(),
            "前缀：晾下之后账上照旧记着还在跑"
        );
        background_gpu
    }

    /// 之后用户的那一次切换。
    #[derive(Clone, Copy, Debug)]
    enum SwitchAfterTheHistory {
        BothNamedUnderALargeLimit,
        BothNamedUnderALimitJustHoldingTheSmallestPiece,
        BothNamedUnderALimitOneByteShort,
        OnlyTheOtherCardNamed,
        ToTheCpu,
    }

    /// 在前缀之后交 `switch`，接住 panic：交回 panic 带的原话（没 panic 是 `None`）与「0000:01:00.0」此刻开没开着、账上还记不记着在跑。
    fn outcome_of_the_switch_after_the_history(
        mut background_gpu: BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
        switch: SwitchAfterTheHistory,
    ) -> (Option<String>, bool, bool) {
        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
        let held_on_the_card_at_one = background_gpu.cards_on_this_host[0]
            .budget
            .memory_account()
            .bytes_held()
            .0;
        let settings = match switch {
            SwitchAfterTheHistory::BothNamedUnderALargeLimit => Some(
                settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]),
            ),
            SwitchAfterTheHistory::BothNamedUnderALimitJustHoldingTheSmallestPiece => {
                Some(settings_admitted_on_the_cards(
                    &background_gpu,
                    None,
                    vec![held_on_the_card_at_one + smallest_piece_bytes, 1 << 30],
                ))
            }
            SwitchAfterTheHistory::BothNamedUnderALimitOneByteShort => {
                Some(settings_admitted_on_the_cards(
                    &background_gpu,
                    None,
                    vec![held_on_the_card_at_one + smallest_piece_bytes - 1, 1 << 30],
                ))
            }
            SwitchAfterTheHistory::OnlyTheOtherCardNamed => Some(settings_admitted_on_the_cards(
                &background_gpu,
                Some(vec![card_identifier("0000:41:00.0")]),
                vec![1 << 30],
            )),
            SwitchAfterTheHistory::ToTheCpu => None,
        };
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match &settings {
            Some(settings) => background_gpu.use_the_settings_from_the_next_batch(settings),
            None => background_gpu.use_no_gpu_from_the_next_batch(),
        }));
        let said_by_the_panic = caught.err().map(|payload| {
            payload
                .downcast_ref::<&str>()
                .map(|said| (*said).to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_default()
        });
        let card_at_one = background_gpu
            .cards_on_this_host
            .iter()
            .find(|card| card.discrete_gpu.identifier == card_identifier("0000:01:00.0"));
        (
            said_by_the_panic,
            card_at_one.is_some(),
            card_at_one.is_some_and(|card| {
                card.budget
                    .memory_account()
                    .has_a_submission_running_past_its_wait()
            }),
        )
    }

    /// 切换让账上还记着在跑的那张卡量代价：先在等的上限之内等它做完、放账，再量——不 panic，量完账上不再记着在跑。
    #[test]
    fn a_measurement_at_a_switch_after_the_measurement_timed_out_waits_for_it_under_a_large_limit()
    {
        assert_eq!(
            outcome_of_the_switch_after_the_history(
                card_at_one_whose_measurement_at_a_switch_timed_out(),
                SwitchAfterTheHistory::BothNamedUnderALargeLimit
            ),
            (None, true, false)
        );
    }

    #[test]
    fn a_measurement_at_a_switch_after_the_measurement_timed_out_waits_for_it_under_a_limit_just_holding_the_smallest_piece(
    ) {
        assert_eq!(
            outcome_of_the_switch_after_the_history(
                card_at_one_whose_measurement_at_a_switch_timed_out(),
                SwitchAfterTheHistory::BothNamedUnderALimitJustHoldingTheSmallestPiece
            ),
            (None, true, false)
        );
    }

    /// 对照：上限少一个字节，这张卡照旧晾下、不量代价——账上照旧记着在跑。
    #[test]
    fn a_switch_after_the_measurement_timed_out_leaves_the_card_out_under_a_limit_one_byte_short() {
        assert_eq!(
            outcome_of_the_switch_after_the_history(
                card_at_one_whose_measurement_at_a_switch_timed_out(),
                SwitchAfterTheHistory::BothNamedUnderALimitOneByteShort
            ),
            (None, true, true)
        );
    }

    #[test]
    fn a_switch_after_the_measurement_timed_out_naming_only_the_other_card_closes_it() {
        assert_eq!(
            outcome_of_the_switch_after_the_history(
                card_at_one_whose_measurement_at_a_switch_timed_out(),
                SwitchAfterTheHistory::OnlyTheOtherCardNamed
            ),
            (None, false, false)
        );
    }

    #[test]
    fn a_switch_to_the_cpu_after_the_measurement_timed_out_closes_every_card() {
        assert_eq!(
            outcome_of_the_switch_after_the_history(
                card_at_one_whose_measurement_at_a_switch_timed_out(),
                SwitchAfterTheHistory::ToTheCpu
            ),
            (None, false, false)
        );
    }

    #[test]
    fn a_measurement_at_a_switch_after_a_batch_timed_out_waits_for_it_under_a_large_limit() {
        assert_eq!(
            outcome_of_the_switch_after_the_history(
                card_at_one_left_out_while_its_timed_out_submission_runs(),
                SwitchAfterTheHistory::BothNamedUnderALargeLimit
            ),
            (None, true, false)
        );
    }

    #[test]
    fn a_measurement_at_a_switch_after_a_batch_timed_out_waits_for_it_under_a_limit_just_holding_the_smallest_piece(
    ) {
        assert_eq!(
            outcome_of_the_switch_after_the_history(
                card_at_one_left_out_while_its_timed_out_submission_runs(),
                SwitchAfterTheHistory::BothNamedUnderALimitJustHoldingTheSmallestPiece
            ),
            (None, true, false)
        );
    }

    #[test]
    fn a_switch_after_a_batch_timed_out_leaves_the_card_out_under_a_limit_one_byte_short() {
        assert_eq!(
            outcome_of_the_switch_after_the_history(
                card_at_one_left_out_while_its_timed_out_submission_runs(),
                SwitchAfterTheHistory::BothNamedUnderALimitOneByteShort
            ),
            (None, true, true)
        );
    }

    #[test]
    fn a_switch_after_a_batch_timed_out_naming_only_the_other_card_closes_it() {
        assert_eq!(
            outcome_of_the_switch_after_the_history(
                card_at_one_left_out_while_its_timed_out_submission_runs(),
                SwitchAfterTheHistory::OnlyTheOtherCardNamed
            ),
            (None, false, false)
        );
    }

    #[test]
    fn a_switch_to_the_cpu_after_a_batch_timed_out_closes_every_card() {
        assert_eq!(
            outcome_of_the_switch_after_the_history(
                card_at_one_left_out_while_its_timed_out_submission_runs(),
                SwitchAfterTheHistory::ToTheCpu
            ),
            (None, false, false)
        );
    }

    /// 等不到账上还在跑的那次提交：量代价照出错交回，这张卡不参与，下一批报成换设置时量代价出错（等前一次没等到的那一种）。
    #[test]
    fn a_measurement_at_a_switch_whose_wait_for_the_running_submission_times_out_leaves_the_card_out(
    ) {
        let mut background_gpu = card_at_one_whose_measurement_at_a_switch_timed_out();
        background_gpu.cards_on_this_host[0]
            .earlier_submission_waits_timing_out
            .push_back(true);
        let bringing_it_back =
            settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
        background_gpu.use_the_settings_from_the_next_batch(&bringing_it_back);
        assert_eq!(
            background_gpu.batch_not_split_over_the_gpus(),
            Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
                device: card_identifier("0000:01:00.0"),
                cause: GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
                    wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                },
                pci_state_when_judged_unusable:
                    PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
            })
        );
        assert_eq!(
            participating_devices_of(&background_gpu),
            vec![card_identifier("0000:41:00.0")]
        );
        assert_eq!(
            background_gpu.pci_state_readings_taken(),
            0,
            "等前一次没等到不是判卡用不了的那一种：量代价出错也不读"
        );
    }

    /// 连着试满的卡在交设置那一刻就答开不成，不为它起开卡线程：「0000:41:00.0」开了 4 次都没开成之后，再交只点它的一组（这一次起线程会被拒）——
    /// 那一问当场答连着试满、没有在开的那一次，开卡那一步照旧只被调 4 次。
    #[test]
    fn the_card_whose_openings_are_used_up_is_answered_not_opened_at_the_handing_in_without_starting_a_thread(
    ) {
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_one_card_not_opened_at_the_mount(
                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                    Err(reason_the_card_does_not_open_at_the_switch())
                },
                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        let _not_opened_after_four_openings = background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds,
            );
        background_gpu.refuse_to_start_the_opening_thread_for_tests(1);
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        assert!(
            background_gpu.opening_at_a_switch.is_none(),
            "不为连着试满的卡起开卡"
        );
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::NotAtAll
            ),
            not_opened_with_the_openings_of_the_card_at_forty_one_used_up()
        );
        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 4);
    }

    /// 叫停之后没开的那几次不算进连着开不成的次数：「0000:41:00.0」头一次开卡在开（等用例放开）时告知一张卡都不参与，放开之后头一次开不成、
    /// 余下 3 次叫停没开（这 3 次照样交记账那一步，算进去就连同头一次满 4 次）；再交只点它的一组，它照开（这一次开成）——不当试满。
    #[test]
    fn openings_called_off_do_not_count_toward_the_openings_of_the_card() {
        let (opening_begun, every_opening_begun) = std::sync::mpsc::channel::<()>();
        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
        let mut is_the_first_opening = true;
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_one_card_not_opened_at_the_mount(
                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                    if is_the_first_opening {
                        is_the_first_opening = false;
                        // 收的那一头在用例里一直拿着。
                        let _signal_sent = opening_begun.send(());
                        let _released_or_not =
                            opening_released.recv_timeout(Duration::from_secs(20));
                        return Err(reason_the_card_does_not_open_at_the_switch());
                    }
                    opening_that_always_opens(
                        discrete_gpu,
                        memory_limit,
                        wait_limit,
                        cost_at_the_opening,
                    )
                },
                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        every_opening_begun
            .recv_timeout(Duration::from_secs(20))
            .expect("开卡线程 20 秒内开始头一次开卡");
        background_gpu.use_no_gpu_from_the_next_batch();
        release_the_opening
            .send(())
            .expect("开卡那一步还在等放开：收的那一头还在");
        drop(
            background_gpu
                .opening_of_a_gpu_at_a_switch
                .lock()
                .expect("开一次卡里的 panic 在那一步就接住了：锁没被毒化"),
        );
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            },
            "叫停的那几次不算：照开、开成"
        );
        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 2);
    }

    /// 开卡的结局按交设置的次序认：切换点「0000:41:00.0」（头一次开卡等用例放开，次次开不成），它在开时又来一次只点「0000:81:00.0」——
    /// 「0000:41:00.0」重开到上限都没开成的结局不报给最近那一组，为最近那一组开「0000:81:00.0」、开成、换上。
    #[test]
    fn the_failure_of_an_opening_for_a_superseded_switch_is_not_reported_to_the_latest_switch() {
        let (release_the_card_at_forty_one, card_at_forty_one_released) =
            std::sync::mpsc::channel::<()>();
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                    if discrete_gpu.pci_bus_address == "0000:41:00.0" {
                        let _released_or_not =
                            card_at_forty_one_released.recv_timeout(Duration::from_secs(20));
                        return Err(reason_the_card_does_not_open_at_the_switch());
                    }
                    opening_that_always_opens(
                        discrete_gpu,
                        memory_limit,
                        wait_limit,
                        cost_at_the_opening,
                    )
                },
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        let to_the_card_at_eighty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
        release_the_card_at_forty_one
            .send(())
            .expect("开「0000:41:00.0」那一步还在等放开");
        drop(release_the_card_at_forty_one);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            LatestSettingsHandedIn::PutInEffect {
                not_split_over_the_gpus: None
            },
            "为被盖掉的那一组开不成的不报给最近那一组"
        );
        assert_eq!(
            participating_devices_of(&background_gpu),
            vec![card_identifier("0000:81:00.0")]
        );
        assert_eq!(
            background_gpu.pci_state_readings_taken(),
            1,
            "为被盖掉的那一组重开确实用完了（没叫停）：判它开不成那一刻照读一次"
        );
    }

    /// 最近那一组点的卡被被盖掉的那一次开卡连着试满：切换点「0000:41:00.0」（头一次开卡等用例放开，次次开不成），它在开时又来一次点它与「0000:81:00.0」的一组——
    /// 为被盖掉的那一组开了 4 次之后，为最近那一组不再开「0000:41:00.0」，当场答连着试满；开卡那一步对它一共只被调 4 次，「0000:81:00.0」一次都没开。
    #[test]
    fn the_latest_switch_naming_a_card_used_up_by_a_superseded_switch_is_answered_not_opened() {
        let (release_the_card_at_forty_one, card_at_forty_one_released) =
            std::sync::mpsc::channel::<()>();
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                    if discrete_gpu.pci_bus_address == "0000:41:00.0" {
                        let _released_or_not =
                            card_at_forty_one_released.recv_timeout(Duration::from_secs(20));
                        return Err(reason_the_card_does_not_open_at_the_switch());
                    }
                    opening_that_always_opens(
                        discrete_gpu,
                        memory_limit,
                        wait_limit,
                        cost_at_the_opening,
                    )
                },
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        let to_both_cards_not_opened_at_the_mount = switch_naming_the_cards_admitted_against(
            &background_gpu,
            &["0000:41:00.0", "0000:81:00.0"],
            None,
        );
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        background_gpu.use_the_settings_from_the_next_batch(&to_both_cards_not_opened_at_the_mount);
        release_the_card_at_forty_one
            .send(())
            .expect("开「0000:41:00.0」那一步还在等放开");
        drop(release_the_card_at_forty_one);
        assert_eq!(
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds
            ),
            not_opened_with_the_openings_of_the_card_at_forty_one_used_up()
        );
        assert_eq!(
            *cards_handed_to_the_opening
                .lock()
                .expect("用例没在别处 panic 过"),
            vec!["0000:41:00.0".to_string(); 4]
        );
    }

    // ── 切换开卡线程开一张卡之前再判一次试满：交设置那一刻还剩次数，前一条线程叫停时正在开的那一次开完之后试满了 ──

    /// 切换点到「0000:41:00.0」之后挂着的是它的哪一次开卡：一次挂载里至多开的最后一次的前一次，或最后一次。
    #[derive(Clone, Copy, Debug)]
    enum OpeningHeldAtTheSwitch {
        TheOneBeforeTheLastAllowed,
        TheLastAllowed,
    }

    impl OpeningHeldAtTheSwitch {
        /// 挂着的那一次是这张卡的第几次开卡（从 1 数）。
        fn opening_number_of_the_card(self) -> usize {
            let openings_at_most =
                usize::try_from(REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS.openings_at_most())
                    .expect("一张卡至多开 4 次：放得进 usize");
            match self {
                Self::TheOneBeforeTheLastAllowed => openings_at_most - 1,
                Self::TheLastAllowed => openings_at_most,
            }
        }
    }

    /// 用户切到 `cpu` 之后切回的那一刻：挂着的那一次开卡开完之前，还是开完（那条线程放开开卡那一步的锁）之后。
    #[derive(Clone, Copy, Debug)]
    enum MomentOfTheSwitchBack {
        WhileTheOpeningIsHeld,
        OnceTheHeldOpeningReturned,
    }

    /// 开卡那一步：「0000:41:00.0」每一次都开不成，对它的第 `held_opening_number` 次先往 `held_opening_begun` 送一声、等用例放开（至多 20 秒）；
    /// 别的卡当场开成。
    fn opening_the_card_at_forty_one_never_and_holding_one_opening(
        held_opening_number: usize,
        held_opening_begun: std::sync::mpsc::Sender<()>,
        held_opening_released: std::sync::mpsc::Receiver<()>,
    ) -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        let mut openings_of_the_card_at_forty_one: usize = 0;
        move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
            if discrete_gpu.pci_bus_address != "0000:41:00.0" {
                return opening_that_always_opens(
                    discrete_gpu,
                    memory_limit,
                    wait_limit,
                    cost_at_the_opening,
                );
            }
            openings_of_the_card_at_forty_one += 1;
            if openings_of_the_card_at_forty_one == held_opening_number {
                // 收的那一头在用例里一直拿着。
                let _signal_sent = held_opening_begun.send(());
                let _released_or_not = held_opening_released.recv_timeout(Duration::from_secs(20));
            }
            Err(reason_the_card_does_not_open_at_the_switch())
        }
    }

    /// panic 带的原话（`&str` 或 `String`；别的交回一句说明）。
    fn text_said_by_the_panic(payload: &(dyn std::any::Any + Send)) -> String {
        payload
            .downcast_ref::<&str>()
            .map(|said| (*said).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "（panic 带的不是字符串）".to_string())
    }

    /// 「切到 cpu 再切回」那一段历史的读数：切回之后当场那一问的答案、等开卡开完那一问的答案（panic 了是它的原话）、开卡那一步对「0000:41:00.0」被调了几次、
    /// 之后再切一次只点「0000:81:00.0」等到开完的答案（panic 了是它的原话）。
    struct ReadingsOfASwitchBack {
        answered_right_after_the_switch_back: LatestSettingsHandedIn,
        answered_once_ended: Result<LatestSettingsHandedIn, String>,
        openings_of_the_card_at_forty_one: usize,
        answered_at_a_later_switch: Result<LatestSettingsHandedIn, String>,
    }

    /// 挂载开着「0000:01:00.0」；切换只点没开的「0000:41:00.0」，它每一次都开不成，`held_opening` 那一次挂着；挂着的时候用户切到 `cpu`
    /// （叫停那条线程，正在开的那一次照旧开完），再切回点 `switch_back_to`（`moment` 定在挂着的那一次开完之前还是之后）；之后再切一次只点「0000:81:00.0」。
    fn readings_of_switching_back_while_an_opening_of_the_card_at_forty_one_is_held(
        held_opening: OpeningHeldAtTheSwitch,
        switch_back_to: &[&str],
        moment: MomentOfTheSwitchBack,
    ) -> ReadingsOfASwitchBack {
        let (held_opening_begun, every_held_opening_begun) = std::sync::mpsc::channel::<()>();
        let (release_the_held_opening, held_opening_released) = std::sync::mpsc::channel::<()>();
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_the_card_at_forty_one_never_and_holding_one_opening(
                    held_opening.opening_number_of_the_card(),
                    held_opening_begun,
                    held_opening_released,
                ),
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        every_held_opening_begun
            .recv_timeout(Duration::from_secs(20))
            .expect("挂着的那一次开卡 20 秒内起了");
        background_gpu.use_no_gpu_from_the_next_batch();
        let switching_back =
            switch_naming_the_cards_admitted_against(&background_gpu, switch_back_to, None);
        match moment {
            MomentOfTheSwitchBack::WhileTheOpeningIsHeld => {}
            MomentOfTheSwitchBack::OnceTheHeldOpeningReturned => {
                release_the_held_opening
                    .send(())
                    .expect("挂着的那一次还在等放开");
                // 切换开卡线程从头一次开卡之前到开完都锁着开卡那一步：拿到锁就是它开完了。
                drop(
                    background_gpu
                        .opening_of_a_gpu_at_a_switch
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner),
                );
            }
        }
        background_gpu.use_the_settings_from_the_next_batch(&switching_back);
        let answered_right_after_the_switch_back = background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::NotAtAll);
        match moment {
            MomentOfTheSwitchBack::WhileTheOpeningIsHeld => {
                release_the_held_opening
                    .send(())
                    .expect("挂着的那一次还在等放开");
            }
            MomentOfTheSwitchBack::OnceTheHeldOpeningReturned => {}
        }
        let answered_once_ended = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds,
            )
        }))
        .map_err(|payload| text_said_by_the_panic(&*payload));
        let openings_of_the_card_at_forty_one =
            openings_of_the_card_at_forty_one(&cards_handed_to_the_opening);
        let to_the_card_at_eighty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
        let answered_at_a_later_switch =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
                background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                    WaitForTheOpeningAtASwitch::UntilItEnds,
                )
            }))
            .map_err(|payload| text_said_by_the_panic(&*payload));
        ReadingsOfASwitchBack {
            answered_right_after_the_switch_back,
            answered_once_ended,
            openings_of_the_card_at_forty_one,
            answered_at_a_later_switch,
        }
    }

    /// 「0000:41:00.0」开不成、每一次的原因是 `causes_of_each_opening`。
    fn card_at_forty_one_not_opened_with(
        causes_of_each_opening: Vec<GpuFailureCause>,
    ) -> LatestSettingsHandedIn {
        LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
            device: card_identifier("0000:41:00.0"),
            pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
                "0000:41:00.0",
            ),
            causes_of_each_opening,
        })
    }

    /// 换上了，这组设置下一批分得到卡上。
    fn put_in_effect_on_every_card() -> LatestSettingsHandedIn {
        LatestSettingsHandedIn::PutInEffect {
            not_split_over_the_gpus: None,
        }
    }

    /// 「0000:41:00.0」在这一次挂载里连着开不成 4 次、试满了。
    fn card_at_forty_one_used_up() -> LatestSettingsHandedIn {
        card_at_forty_one_not_opened_with(vec![GpuFailureCause::OpeningsInThisMountUsedUp {
            failed_openings: 4,
        }])
    }

    /// 那一段历史照条款的结局：两问的答案、「0000:41:00.0」一共开了几次（一次挂载里连着开不成至多 4 次）；之后再切一次只点「0000:81:00.0」换上了；
    /// 哪一问都不 panic。
    fn assert_the_switch_back_ends_as(
        held_opening: OpeningHeldAtTheSwitch,
        switch_back_to: &[&str],
        moment: MomentOfTheSwitchBack,
        answered_right_after_the_switch_back: LatestSettingsHandedIn,
        answered_once_ended: LatestSettingsHandedIn,
        openings_of_the_card_at_forty_one: usize,
    ) {
        let readings = readings_of_switching_back_while_an_opening_of_the_card_at_forty_one_is_held(
            held_opening,
            switch_back_to,
            moment,
        );
        match moment {
            MomentOfTheSwitchBack::WhileTheOpeningIsHeld => assert_eq!(
                readings.answered_right_after_the_switch_back,
                answered_right_after_the_switch_back
            ),
            // 挂着的那一次已开完、为切回起的新线程不用等锁：它开得快时，切回之后当场那一问可能已收到结局。还在开、或答的就是开完时那个结局，两样都对。
            MomentOfTheSwitchBack::OnceTheHeldOpeningReturned => assert!(
                readings.answered_right_after_the_switch_back == answered_right_after_the_switch_back
                    || readings.answered_right_after_the_switch_back == answered_once_ended,
                "切回之后当场答 {:?}：不是 {answered_right_after_the_switch_back:?}，也不是开完时的 {answered_once_ended:?}",
                readings.answered_right_after_the_switch_back
            ),
        }
        assert_eq!(readings.answered_once_ended, Ok(answered_once_ended));
        assert_eq!(
            readings.openings_of_the_card_at_forty_one,
            openings_of_the_card_at_forty_one
        );
        assert_eq!(
            readings.answered_at_a_later_switch,
            Ok(put_in_effect_on_every_card())
        );
    }

    /// 挂着的是最后一次，挂着的时候切回同一张卡：交设置那一刻它才开不成 3 次、为它起了新线程；新线程开它之前前一次开完、试满了——
    /// 不再开、当场答试满，它一共开 4 次，不 panic。
    #[test]
    fn switching_back_to_the_card_while_its_last_allowed_opening_is_held_answers_it_used_up_without_opening_it_again(
    ) {
        assert_the_switch_back_ends_as(
            OpeningHeldAtTheSwitch::TheLastAllowed,
            &["0000:41:00.0"],
            MomentOfTheSwitchBack::WhileTheOpeningIsHeld,
            LatestSettingsHandedIn::StillOpeningItsCards,
            card_at_forty_one_used_up(),
            4,
        );
    }

    /// 同上，切回点「0000:81:00.0」再点它：先开成「0000:81:00.0」，轮到它时试满了、当场答试满，它一共开 4 次，不 panic。
    #[test]
    fn switching_back_to_the_other_card_then_the_card_while_its_last_allowed_opening_is_held_answers_it_used_up(
    ) {
        assert_the_switch_back_ends_as(
            OpeningHeldAtTheSwitch::TheLastAllowed,
            &["0000:81:00.0", "0000:41:00.0"],
            MomentOfTheSwitchBack::WhileTheOpeningIsHeld,
            LatestSettingsHandedIn::StillOpeningItsCards,
            card_at_forty_one_used_up(),
            4,
        );
    }

    /// 挂着的是最后一次，它开完之后才切回同一张卡：交设置那一刻就试满，当场答试满。
    #[test]
    fn switching_back_to_the_card_once_its_last_allowed_opening_returned_answers_it_used_up_at_the_handing_in(
    ) {
        assert_the_switch_back_ends_as(
            OpeningHeldAtTheSwitch::TheLastAllowed,
            &["0000:41:00.0"],
            MomentOfTheSwitchBack::OnceTheHeldOpeningReturned,
            card_at_forty_one_used_up(),
            card_at_forty_one_used_up(),
            4,
        );
    }

    /// 同上，切回点「0000:81:00.0」再点它：交设置那一刻就试满，当场答试满。
    #[test]
    fn switching_back_to_the_other_card_then_the_card_once_its_last_allowed_opening_returned_answers_it_used_up(
    ) {
        assert_the_switch_back_ends_as(
            OpeningHeldAtTheSwitch::TheLastAllowed,
            &["0000:81:00.0", "0000:41:00.0"],
            MomentOfTheSwitchBack::OnceTheHeldOpeningReturned,
            card_at_forty_one_used_up(),
            card_at_forty_one_used_up(),
            4,
        );
    }

    /// 挂着的是最后一次，挂着的时候切回只点「0000:81:00.0」：换上它。
    #[test]
    fn switching_back_to_the_other_card_while_the_last_allowed_opening_is_held_puts_it_in_effect() {
        assert_the_switch_back_ends_as(
            OpeningHeldAtTheSwitch::TheLastAllowed,
            &["0000:81:00.0"],
            MomentOfTheSwitchBack::WhileTheOpeningIsHeld,
            LatestSettingsHandedIn::StillOpeningItsCards,
            put_in_effect_on_every_card(),
            4,
        );
    }

    /// 同上，最后一次开完之后才切回：换上它。
    #[test]
    fn switching_back_to_the_other_card_once_the_last_allowed_opening_returned_puts_it_in_effect() {
        assert_the_switch_back_ends_as(
            OpeningHeldAtTheSwitch::TheLastAllowed,
            &["0000:81:00.0"],
            MomentOfTheSwitchBack::OnceTheHeldOpeningReturned,
            LatestSettingsHandedIn::StillOpeningItsCards,
            put_in_effect_on_every_card(),
            4,
        );
    }

    /// 挂着的是最后一次的前一次，挂着的时候切回同一张卡：新线程开它之前它开不成 3 次，还剩一次，开一次又开不成，答开不成、一个原因，一共 4 次。
    #[test]
    fn switching_back_to_the_card_while_the_opening_before_its_last_is_held_opens_it_once_more() {
        assert_the_switch_back_ends_as(
            OpeningHeldAtTheSwitch::TheOneBeforeTheLastAllowed,
            &["0000:41:00.0"],
            MomentOfTheSwitchBack::WhileTheOpeningIsHeld,
            LatestSettingsHandedIn::StillOpeningItsCards,
            card_at_forty_one_not_opened_with(vec![reason_the_card_does_not_open_at_the_switch()]),
            4,
        );
    }

    /// 同上，那一次开完之后才切回：开一次又开不成，一共 4 次。
    #[test]
    fn switching_back_to_the_card_once_the_opening_before_its_last_returned_opens_it_once_more() {
        assert_the_switch_back_ends_as(
            OpeningHeldAtTheSwitch::TheOneBeforeTheLastAllowed,
            &["0000:41:00.0"],
            MomentOfTheSwitchBack::OnceTheHeldOpeningReturned,
            LatestSettingsHandedIn::StillOpeningItsCards,
            card_at_forty_one_not_opened_with(vec![reason_the_card_does_not_open_at_the_switch()]),
            4,
        );
    }

    /// 挂着的是最后一次的前一次，挂着的时候切回点「0000:81:00.0」再点它：先开成「0000:81:00.0」，它再开一次又开不成，一共 4 次。
    #[test]
    fn switching_back_to_the_other_card_then_the_card_while_the_opening_before_its_last_is_held_opens_it_once_more(
    ) {
        assert_the_switch_back_ends_as(
            OpeningHeldAtTheSwitch::TheOneBeforeTheLastAllowed,
            &["0000:81:00.0", "0000:41:00.0"],
            MomentOfTheSwitchBack::WhileTheOpeningIsHeld,
            LatestSettingsHandedIn::StillOpeningItsCards,
            card_at_forty_one_not_opened_with(vec![reason_the_card_does_not_open_at_the_switch()]),
            4,
        );
    }

    /// 同上，那一次开完之后才切回：它再开一次又开不成，一共 4 次。
    #[test]
    fn switching_back_to_the_other_card_then_the_card_once_the_opening_before_its_last_returned_opens_it_once_more(
    ) {
        assert_the_switch_back_ends_as(
            OpeningHeldAtTheSwitch::TheOneBeforeTheLastAllowed,
            &["0000:81:00.0", "0000:41:00.0"],
            MomentOfTheSwitchBack::OnceTheHeldOpeningReturned,
            LatestSettingsHandedIn::StillOpeningItsCards,
            card_at_forty_one_not_opened_with(vec![reason_the_card_does_not_open_at_the_switch()]),
            4,
        );
    }

    /// 挂着的是最后一次的前一次，挂着的时候切回只点「0000:81:00.0」：换上它，「0000:41:00.0」一共 3 次。
    #[test]
    fn switching_back_to_the_other_card_while_the_opening_before_the_last_is_held_puts_it_in_effect(
    ) {
        assert_the_switch_back_ends_as(
            OpeningHeldAtTheSwitch::TheOneBeforeTheLastAllowed,
            &["0000:81:00.0"],
            MomentOfTheSwitchBack::WhileTheOpeningIsHeld,
            LatestSettingsHandedIn::StillOpeningItsCards,
            put_in_effect_on_every_card(),
            3,
        );
    }

    /// 同上，那一次开完之后才切回：换上它，「0000:41:00.0」一共 3 次。
    #[test]
    fn switching_back_to_the_other_card_once_the_opening_before_the_last_returned_puts_it_in_effect(
    ) {
        assert_the_switch_back_ends_as(
            OpeningHeldAtTheSwitch::TheOneBeforeTheLastAllowed,
            &["0000:81:00.0"],
            MomentOfTheSwitchBack::OnceTheHeldOpeningReturned,
            LatestSettingsHandedIn::StillOpeningItsCards,
            put_in_effect_on_every_card(),
            3,
        );
    }

    // ── 前一次开卡到点没回来不记进连着开不成的次数 ──

    /// 「0000:41:00.0」上前一次开卡到点没回来：开卡那一步次次报没回来——切换点它开 4 次都报没回来、答开不成；再交同一组，照常为它起开卡、又开 4 次，
    /// 报的仍是没回来、不是试满（没回来不记进这一次挂载里连着开不成的次数）。
    #[test]
    fn openings_answered_not_returned_four_times_in_a_row_leave_the_card_opened_again_at_the_next_switch(
    ) {
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                    Err(opening_not_started_while_an_earlier_one_has_not_returned())
                },
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered_at_the_switch = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered_at_the_next_switch = background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds,
            );
        let not_returned_four_times = card_at_forty_one_not_opened_with(vec![
            opening_not_started_while_an_earlier_one_has_not_returned();
            4
        ]);
        assert_eq!(answered_at_the_switch, not_returned_four_times);
        assert_eq!(
            answered_at_the_next_switch, not_returned_four_times,
            "再切到它照常起开卡，报的是没回来、不是试满"
        );
        assert_eq!(
            openings_of_the_card_at_forty_one(&cards_handed_to_the_opening),
            8
        );
    }

    // ── 进程级那一张表：没到点、正在开的开卡，后到的一方排队等它开完 ──

    /// 两次挂载拿的那一张表：进程开卡器那一张（产品照这样给），或各拿一张。
    #[derive(Clone, Copy, Debug)]
    enum TablesOfTheTwoMounts {
        OneTableOfTheProcess,
        OneTableForEachMount,
    }

    /// 产品那一份开一张卡那一步（`opening_of_a_card_within_the_limit`，一次至多等 `opening_limit`，在开的开卡那一张表是 `openings_on_the_devices`）：
    /// 起卡那一件先往 `start_begun` 送一声、等用例放开（至多 20 秒）再开成替身卡。
    fn opening_held_until_released_within(
        opening_limit: Duration,
        start_released: std::sync::mpsc::Receiver<()>,
        start_begun: std::sync::mpsc::Sender<()>,
        openings_on_the_devices: &OpeningsNotReturnedInThisProcess,
    ) -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        let start_released = Arc::new(Mutex::new(start_released));
        opening_of_a_card_within_the_limit(
            opening_limit,
            move |discrete_gpu: &DiscreteGpuOnThisHost,
                  memory_limit,
                  _wait_limit,
                  bytes_leaked_by_earlier_mounts| {
                let card = CardComputedOnTheCpuForTests::of_the_address(
                    &discrete_gpu.pci_bus_address,
                    memory_limit.map(|memory_limit| memory_limit.0),
                    bytes_leaked_by_earlier_mounts.0,
                );
                let start_released_seen_by_the_start = Arc::clone(&start_released);
                let start_begun_by_the_start = start_begun.clone();
                move || {
                    // 收的那一头在用例里一直拿着。
                    let _signal_sent = start_begun_by_the_start.send(());
                    let _released_or_not = start_released_seen_by_the_start
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .recv_timeout(Duration::from_secs(20));
                    Ok(card)
                }
            },
            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            openings_on_the_devices.clone(),
        )
    }

    /// 产品那一份开一张卡那一步（一次至多等 20 秒，在开的开卡那一张表是 `openings_on_the_devices`），起卡那一件当场开成替身卡。
    fn opening_that_opens_at_once_within_twenty_seconds(
        openings_on_the_devices: &OpeningsNotReturnedInThisProcess,
    ) -> impl FnMut(
        &DiscreteGpuOnThisHost,
        Option<GpuMemoryInBytes>,
        Duration,
        CostAtTheOpening,
    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
           + Send
           + 'static {
        opening_of_a_card_within_the_limit(
            Duration::from_secs(20),
            move |discrete_gpu: &DiscreteGpuOnThisHost,
                  memory_limit,
                  _wait_limit,
                  bytes_leaked_by_earlier_mounts| {
                let card = CardComputedOnTheCpuForTests::of_the_address(
                    &discrete_gpu.pci_bus_address,
                    memory_limit.map(|memory_limit| memory_limit.0),
                    bytes_leaked_by_earlier_mounts.0,
                );
                move || Ok(card)
            },
            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            openings_on_the_devices.clone(),
        )
    }

    /// 等到 `openings_on_the_devices` 里有一次开卡在排队（至多 20 秒）：等到了交回 `true`。
    fn an_opening_queued_within_twenty_seconds(
        openings_on_the_devices: &OpeningsNotReturnedInThisProcess,
    ) -> bool {
        (0..2000).any(|_poll| {
            if openings_on_the_devices.openings_waiting_for_an_earlier_one() > 0 {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
            false
        })
    }

    /// 「另一次挂载正在开」那一段历史的读数。
    struct ReadingsOfASwitchWhileAnotherMountOpens {
        the_other_mount_failed_with: Option<GpuFailureCause>,
        this_mount_queued_behind_the_other: bool,
        answered_while_the_other_mount_opens: LatestSettingsHandedIn,
        answered_once_handed_in_again: LatestSettingsHandedIn,
        openings_in_this_mount: usize,
    }

    /// 同一进程里另一次挂载正在开「0000:41:00.0」（一次至多等 20 秒，起卡那一件等用例放开，没到点）；这一次挂载（切换开卡那一步用产品那一份）
    /// 切换只点它，等开完；之后再交同一组。`OneTableOfTheProcess` 时这一次挂载的开卡排上队之后才放开另一次挂载，不然那一问答了之后才放开。
    fn readings_of_a_switch_while_another_mount_opens_the_card(
        tables: TablesOfTheTwoMounts,
    ) -> ReadingsOfASwitchWhileAnotherMountOpens {
        let table_of_this_mount = OpeningsNotReturnedInThisProcess::with_none_not_returned();
        let table_of_the_other_mount = match tables {
            TablesOfTheTwoMounts::OneTableOfTheProcess => table_of_this_mount.clone(),
            TablesOfTheTwoMounts::OneTableForEachMount => {
                OpeningsNotReturnedInThisProcess::with_none_not_returned()
            }
        };
        let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
        let (start_begun, every_start_begun) = std::sync::mpsc::channel::<()>();
        let mut opening_of_the_other_mount = opening_held_until_released_within(
            Duration::from_secs(20),
            start_released,
            start_begun,
            &table_of_the_other_mount,
        );
        let the_other_mount = std::thread::spawn(move || {
            opening_of_the_other_mount(
                &discrete_gpu_not_opened_at("0000:41:00.0"),
                None,
                BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                CostAtTheOpening::MeasuredUnderTheMemoryLimit,
            )
            .err()
        });
        every_start_begun
            .recv_timeout(Duration::from_secs(20))
            .expect("另一次挂载的开卡 20 秒内起了");
        let (mut background_gpu, cards_handed_to_the_opening) =
            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
                opening_that_opens_at_once_within_twenty_seconds(&table_of_this_mount),
                None,
                false,
            );
        let to_the_card_at_forty_one =
            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let this_mount_queued_behind_the_other = match tables {
            TablesOfTheTwoMounts::OneTableOfTheProcess => {
                let queued = an_opening_queued_within_twenty_seconds(&table_of_this_mount);
                release_the_start
                    .send(())
                    .expect("另一次挂载的起卡那一件还在等放开");
                queued
            }
            TablesOfTheTwoMounts::OneTableForEachMount => false,
        };
        let answered_while_the_other_mount_opens = background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds,
            );
        match tables {
            TablesOfTheTwoMounts::OneTableOfTheProcess => {}
            TablesOfTheTwoMounts::OneTableForEachMount => {
                release_the_start
                    .send(())
                    .expect("另一次挂载的起卡那一件还在等放开");
            }
        }
        let the_other_mount_failed_with =
            the_other_mount.join().expect("另一次挂载那条线程没 panic");
        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
        let answered_once_handed_in_again = background_gpu
            .put_the_gpus_opened_since_the_switch_in_effect(
                WaitForTheOpeningAtASwitch::UntilItEnds,
            );
        ReadingsOfASwitchWhileAnotherMountOpens {
            the_other_mount_failed_with,
            this_mount_queued_behind_the_other,
            answered_while_the_other_mount_opens,
            answered_once_handed_in_again,
            openings_in_this_mount: openings_made_so_far(&cards_handed_to_the_opening),
        }
    }

    /// 第 4 条（两个池同时开同一张卡，后到的等先到的开完再开）：两次挂载共用进程那一张表，另一次挂载正在开「0000:41:00.0」、没到点——
    /// 这一次挂载的切换开卡排队等它开完再开，开成、换上，不算它「前一次开卡没回来」、不在这一次挂载里试满；另一次挂载照常开成。
    #[test]
    fn a_switch_while_another_mount_opens_the_card_queues_behind_that_opening_and_puts_the_card_in_effect(
    ) {
        let readings = readings_of_a_switch_while_another_mount_opens_the_card(
            TablesOfTheTwoMounts::OneTableOfTheProcess,
        );
        assert_eq!(readings.the_other_mount_failed_with, None);
        assert!(
            readings.this_mount_queued_behind_the_other,
            "这一次挂载的开卡排在另一次挂载那一次后面"
        );
        assert_eq!(
            readings.answered_while_the_other_mount_opens,
            put_in_effect_on_every_card()
        );
        assert_eq!(
            readings.answered_once_handed_in_again,
            put_in_effect_on_every_card()
        );
        assert_eq!(readings.openings_in_this_mount, 1);
    }

    /// 对照：两次挂载各拿一张表，这一次挂载不排队、当场开成换上。
    #[test]
    fn a_switch_while_another_mount_with_its_own_table_opens_the_card_puts_the_card_in_effect_at_once(
    ) {
        let readings = readings_of_a_switch_while_another_mount_opens_the_card(
            TablesOfTheTwoMounts::OneTableForEachMount,
        );
        assert_eq!(readings.the_other_mount_failed_with, None);
        assert_eq!(
            readings.answered_while_the_other_mount_opens,
            put_in_effect_on_every_card()
        );
        assert_eq!(
            readings.answered_once_handed_in_again,
            put_in_effect_on_every_card()
        );
        assert_eq!(readings.openings_in_this_mount, 1);
    }

    /// 排队等的时间算进后到那一方自己的期限：另一次挂载正在开「0000:41:00.0」（一次至多等 20 秒，没到点），这一次开它至多等 50 毫秒——
    /// 等满 50 毫秒还没轮到，照这一次到点办（报等超过上限、带 50 毫秒），这一次的起卡那一件一次都没起。
    #[test]
    fn an_opening_queued_behind_another_mount_past_its_own_limit_is_not_opened_within_that_limit() {
        const OPENING_LIMIT_OF_THIS_OPENING: Duration = Duration::from_millis(50);
        let openings_on_the_devices = OpeningsNotReturnedInThisProcess::with_none_not_returned();
        let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
        let (start_begun, every_start_begun) = std::sync::mpsc::channel::<()>();
        let mut opening_of_the_other_mount = opening_held_until_released_within(
            Duration::from_secs(20),
            start_released,
            start_begun,
            &openings_on_the_devices,
        );
        let the_other_mount = std::thread::spawn(move || {
            opening_of_the_other_mount(
                &discrete_gpu_not_opened_at("0000:41:00.0"),
                None,
                BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                CostAtTheOpening::MeasuredUnderTheMemoryLimit,
            )
            .err()
        });
        every_start_begun
            .recv_timeout(Duration::from_secs(20))
            .expect("另一次挂载的开卡 20 秒内起了");
        let starts_of_this_opening = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let starts_counted = Arc::clone(&starts_of_this_opening);
        let mut opening_of_this_mount = opening_of_a_card_within_the_limit(
            OPENING_LIMIT_OF_THIS_OPENING,
            move |discrete_gpu: &DiscreteGpuOnThisHost,
                  memory_limit,
                  _wait_limit,
                  bytes_leaked_by_earlier_mounts| {
                let card = CardComputedOnTheCpuForTests::of_the_address(
                    &discrete_gpu.pci_bus_address,
                    memory_limit.map(|memory_limit| memory_limit.0),
                    bytes_leaked_by_earlier_mounts.0,
                );
                let starts_counted_by_the_start = Arc::clone(&starts_counted);
                move || {
                    starts_counted_by_the_start.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(card)
                }
            },
            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            openings_on_the_devices.clone(),
        );
        let this_opening = opening_of_this_mount(
            &discrete_gpu_not_opened_at("0000:41:00.0"),
            None,
            BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
            CostAtTheOpening::MeasuredUnderTheMemoryLimit,
        )
        .err();
        release_the_start
            .send(())
            .expect("另一次挂载的起卡那一件还在等放开");
        let the_other_mount_failed_with =
            the_other_mount.join().expect("另一次挂载那条线程没 panic");
        assert_eq!(
            this_opening,
            Some(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                wait_limit: OPENING_LIMIT_OF_THIS_OPENING,
            })
        );
        assert_eq!(
            starts_of_this_opening.load(std::sync::atomic::Ordering::SeqCst),
            0
        );
        assert_eq!(the_other_mount_failed_with, None);
    }

    // ── 三段都在交卡之前查重复键 ──

    /// 一张替身卡，每一次提交都等超过上限（交到它上面这一批就报这张卡出错）。
    fn background_gpu_over_a_card_timing_out_every_submission(
    ) -> BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests> {
        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        card.submissions_timing_out = VecDeque::from([true; 16]);
        let (background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        background_gpu
    }

    /// 一段一片都没交过、一张卡都没分到活时记下的样子。
    fn submissions_of_a_segment_never_handed_to_a_card() -> GpuSubmissionsOfOneSegment {
        GpuSubmissionsOfOneSegment {
            submissions_completed: 0,
            pieces_submitted: 0,
            resubmissions_after_timeouts: 0,
            recuts_after_running_out_of_memory: 0,
            devices_given_work: Vec::new(),
        }
    }

    fn repeated_at_the_first_two_positions() -> SameEntryKeyTwiceInOneBatch {
        SameEntryKeyTwiceInOneBatch {
            position_of_the_first_occurrence: 0,
            position_of_the_second_occurrence: 1,
        }
    }

    /// 校验和那一段一批里同一个条目键两次、交到卡上就会报这张卡出错：交卡之前就查出，交回重复的键，两份核一片都没交、这张卡没分到活。
    #[test]
    fn a_checksum_batch_with_a_repeated_entry_key_is_returned_as_repeated_before_any_piece_is_handed_to_a_card(
    ) {
        let mut background_gpu = background_gpu_over_a_card_timing_out_every_submission();
        let unit_bytes = unit_bytes_of(2, 100);
        let units: Vec<UnitChecksumToVerify<'_, usize>> = unit_bytes
            .iter()
            .map(|bytes| UnitChecksumToVerify {
                entry_key: 7,
                unit_bytes: bytes,
                expected_crc32c: singlefs_core::checksum::crc32_castagnoli(bytes),
            })
            .collect();
        assert_eq!(
            background_gpu.verify_unit_checksums(
                &units,
                &mut ConsecutiveUncertaintyCounts::with_no_entry_uncertain()
            ),
            Err(ChecksumBatchFailed::SameEntryKeyTwiceInOneBatch(
                repeated_at_the_first_two_positions()
            ))
        );
        assert_eq!(
            background_gpu
                .take_the_submissions_of_the_batch()
                .unit_checksum_verification,
            submissions_of_a_segment_never_handed_to_a_card()
        );
    }

    /// 同一条在区域扫描那一段：两条记录同一个条目键。
    #[test]
    fn a_region_record_batch_with_a_repeated_entry_key_is_returned_as_repeated_before_any_piece_is_handed_to_a_card(
    ) {
        let mut background_gpu = background_gpu_over_a_card_timing_out_every_submission();
        let (region, records) = two_region_records_with_an_empty_mapping();
        let unit_bytes = [7_u8; 64];
        let to_classify: Vec<RegionRecordToClassify<'_, usize>> = records
            .iter()
            .map(|record| RegionRecordToClassify {
                entry_key: 7,
                record: *record,
                unit_bytes: Some(&unit_bytes),
            })
            .collect();
        assert_eq!(
            background_gpu.classify_region_records(
                region,
                &to_classify,
                &[],
                &mut ConsecutiveUncertaintyCounts::with_no_entry_uncertain()
            ),
            Err(RegionRecordBatchFailed::SameEntryKeyTwiceInOneBatch(
                repeated_at_the_first_two_positions()
            ))
        );
        assert_eq!(
            background_gpu
                .take_the_submissions_of_the_batch()
                .region_record_classification,
            submissions_of_a_segment_never_handed_to_a_card()
        );
    }

    /// 同一条在选段打分那一段：同一个段两次。
    #[test]
    fn a_live_slot_batch_with_a_repeated_segment_is_returned_as_repeated_before_any_piece_is_handed_to_a_card(
    ) {
        let mut background_gpu = background_gpu_over_a_card_timing_out_every_submission();
        let unit_area = UnitAreaOfOneDevice {
            device: DeviceIdentity(0),
            start: UnitAreaStart::of_the_default_journal_ring(),
            slots: 128,
        };
        let segment = ClusterSegmentEntryKey {
            device: DeviceIdentity(0),
            start_slot: unit_area.start.slot(),
        };
        assert_eq!(
            background_gpu.count_live_slots_per_cluster_segment(
                &[],
                unit_area,
                &[segment, segment],
                &mut ConsecutiveUncertaintyCounts::with_no_entry_uncertain()
            ),
            Err(LiveSlotBatchFailed::SameEntryKeyTwiceInOneBatch(
                repeated_at_the_first_two_positions()
            ))
        );
        assert_eq!(
            background_gpu
                .take_the_submissions_of_the_batch()
                .live_slots_per_cluster_segment,
            submissions_of_a_segment_never_handed_to_a_card()
        );
    }

    /// 替身卡等提交时 panic 带的原话（只供测试的钩子 `submission_waits_panicking`）。
    const PANIC_SAID_BY_THE_WAIT_OF_THE_TEST_CARD: &str = "替身：等提交时设备丢了";

    /// 等卡回话那一下 panic（wgpu 30.0.1 在等待报设备丢了时在 `Device::poll` 里 panic）：接住，这一批报这张卡的 GPU 出错、原因是卡当场报出错
    /// （带 panic 的原话），判它用不了那一刻读 1 次 PCI 状态，panic 不穿出去；这一段的提交照没等到它做完记在账上（关卡时整张泄漏）。
    /// 下一批交活之前那一等当场照记下的报、不再调 wgpu，这张卡一段都不再交（交成的次数停在 1），这一次又判它用不了、再读 1 次。
    #[test]
    fn a_panic_in_the_wait_for_a_submission_is_caught_as_the_card_lost_and_the_card_gets_no_more_work(
    ) {
        let mut card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        card.submission_waits_panicking = true;
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let card_lost = GpuFailureCause::UncapturedErrorOrDeviceLost {
            reason: format!(
                "等提交时 wgpu 的 Device::poll panic 了：{PANIC_SAID_BY_THE_WAIT_OF_THE_TEST_CARD}"
            ),
        };
        let pci_state_read = pci_state_read_under_the_root_not_present_for_tests("0000:01:00.0");
        let (first_batch, _devices_given_work) =
            checksum_batch_and_the_devices_given_work(&mut background_gpu);
        assert_eq!(
            first_batch,
            Err(ChecksumBatchFailed::GpuFailed {
                device: card_identifier("0000:01:00.0"),
                cause: card_lost.clone(),
                pci_state_when_judged_unusable: pci_state_read.clone(),
            })
        );
        assert_eq!(background_gpu.pci_state_readings_taken(), 1);
        assert!(
            background_gpu.cards_on_this_host[0]
                .budget
                .memory_account()
                .has_a_submission_running_past_its_wait(),
            "panic 了的那次提交照没等到它做完记着：关卡时这张卡整张泄漏"
        );
        let (second_batch, _devices_given_work_by_the_second_batch) =
            checksum_batch_and_the_devices_given_work(&mut background_gpu);
        assert_eq!(
            second_batch,
            Err(ChecksumBatchFailed::GpuFailed {
                device: card_identifier("0000:01:00.0"),
                cause: card_lost,
                pci_state_when_judged_unusable: pci_state_read,
            })
        );
        assert_eq!(
            background_gpu.cards_on_this_host[0].submissions_made, 1,
            "接住之后这张卡一段都不再交"
        );
        assert_eq!(background_gpu.pci_state_readings_taken(), 2);
    }

    /// 第 4 条「卡当场报出错（设备丢失、驱动报的未捕获的错）、后台任务因此停下」用于换设置时量代价：「0000:01:00.0」开卡时没量过代价，换设置让它参与时
    /// 量代价那一次设备丢了——那一刻读 1 次 PCI 状态，读数跟着「换设置时量代价出错」存下；之后两批之间连问 3 次、连交 2 批，带的都是那一份，不再读。
    #[test]
    fn the_card_lost_while_its_cost_is_measured_at_a_switch_is_read_once_and_every_later_batch_carries_that_reading(
    ) {
        let card_lost = GpuFailureCause::UncapturedErrorOrDeviceLost {
            reason: "替身：量代价时设备丢了".to_string(),
        };
        let mut card_at_one = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        card_at_one.measured_cost = None;
        card_at_one.cost_measurement_failing_with = Some(card_lost.clone());
        let card_at_forty_one =
            CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0);
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card_at_one, card_at_forty_one],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let both_cards =
            settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
        background_gpu.use_the_settings_from_the_next_batch(&both_cards);
        let pci_state_read = pci_state_read_under_the_root_not_present_for_tests("0000:01:00.0");
        let not_split = BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
            device: card_identifier("0000:01:00.0"),
            cause: card_lost.clone(),
            pci_state_when_judged_unusable: pci_state_read.clone(),
        };
        for _question in 0..3 {
            assert_eq!(
                background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                    WaitForTheOpeningAtASwitch::NotAtAll
                ),
                LatestSettingsHandedIn::PutInEffect {
                    not_split_over_the_gpus: Some(not_split.clone())
                }
            );
        }
        for _batch in 0..2 {
            let (outcome, _devices_given_work) =
                checksum_batch_and_the_devices_given_work(&mut background_gpu);
            assert_eq!(
                outcome,
                Err(ChecksumBatchFailed::GpuFailed {
                    device: card_identifier("0000:01:00.0"),
                    cause: card_lost.clone(),
                    pci_state_when_judged_unusable: pci_state_read.clone(),
                })
            );
        }
        assert_eq!(background_gpu.pci_state_readings_taken(), 1);
    }

    /// 卸载或切到 `cpu` 叫停了开卡线程之后不读：「0000:41:00.0」头一次开卡在开（等用例放开）时告知一张卡都不参与，放开之后头一次开不成、余下 3 次叫停没开——
    /// 线程交回开不成之前看见叫停，不读，读次数是 0。
    #[test]
    fn the_opening_called_off_reads_no_pci_state() {
        let (opening_begun, every_opening_begun) = std::sync::mpsc::channel::<()>();
        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_one_card_not_opened_at_the_mount(
                move |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                    // 收的那一头在用例里一直拿着。
                    let _signal_sent = opening_begun.send(());
                    let _released_or_not = opening_released.recv_timeout(Duration::from_secs(20));
                    Err(reason_the_card_does_not_open_at_the_switch())
                },
                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        every_opening_begun
            .recv_timeout(Duration::from_secs(20))
            .expect("开卡线程 20 秒内开始头一次开卡");
        background_gpu.use_no_gpu_from_the_next_batch();
        release_the_opening
            .send(())
            .expect("开卡那一步还在等放开：收的那一头还在");
        drop(
            background_gpu
                .opening_of_a_gpu_at_a_switch
                .lock()
                .expect("开一次卡里的 panic 在那一步就接住了：锁没被毒化"),
        );
        assert_eq!(background_gpu.pci_state_readings_taken(), 0);
    }

    /// 第 4 条「开卡重开用完都开不成」用于起不了开卡线程：头一次与 3 次再起都起不了，次数用完——判头一张要开的卡开不成那一刻读 1 次，那一问答的开不成带着那一份。
    #[test]
    fn the_opening_thread_never_started_reads_the_pci_state_once_its_openings_are_used_up() {
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_one_card_not_opened_at_the_mount(
                opening_that_always_opens,
                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
            );
        background_gpu.refuse_to_start_the_opening_thread_for_tests(u32::MAX);
        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
            WaitForTheOpeningAtASwitch::UntilItEnds,
        );
        let pci_state_read = pci_state_read_under_the_root_not_present_for_tests("0000:41:00.0");
        assert!(
            matches!(
                &answered,
                LatestSettingsHandedIn::NotOpened(not_opened)
                    if not_opened.pci_state_when_judged_unusable == pci_state_read
                        && not_opened.causes_of_each_opening.len() == 4
            ),
            "{answered:?}"
        );
        assert_eq!(background_gpu.pci_state_readings_taken(), 1);
    }

    /// 开卡线程上开到「0000:41:00.0」这一刻它已试满（连着开不成 4 次）：不开，判它开不成那一刻读 1 次，开不成的那一份带着它；
    /// 同一刻已叫停的，交回开不成之前看见叫停、不读，读数是没读。
    #[test]
    fn the_card_used_up_when_the_opening_thread_reaches_it_is_read_once_unless_called_off() {
        let discrete_gpu = discrete_gpu_not_opened_at("0000:41:00.0");
        let outcome_and_readings = |is_called_off: bool| {
            let failed_openings_in_this_mount = FailedOpeningsOfEachGpuInThisMount::default();
            for _failed_opening in 0..REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS.openings_at_most() {
                failed_openings_in_this_mount.note_a_failed_opening(&discrete_gpu.identifier);
            }
            let pci_state_reader = pci_state_reader_under_a_root_not_present_for_tests();
            let outcome = gpus_opened_at_a_switch(
                &mut no_opening_at_a_switch(),
                vec![(discrete_gpu.clone(), None)],
                WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                Vec::new(),
                &AtomicBool::new(is_called_off),
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
                &failed_openings_in_this_mount,
                &pci_state_reader,
            );
            (
                outcome
                    .not_opened
                    .map(|not_opened| not_opened.pci_state_when_judged_unusable),
                pci_state_reader.readings_taken(),
            )
        };
        assert_eq!(
            outcome_and_readings(false),
            (
                Some(pci_state_read_under_the_root_not_present_for_tests(
                    "0000:41:00.0"
                )),
                1
            )
        );
        assert_eq!(
            outcome_and_readings(true),
            (
                Some(PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError),
                0
            )
        );
    }

    /// 区域扫描与选段打分两段一批出错时，判卡用不了那一刻读的那一份照带进各段的出错（`region_record_batch_failed_of`、`live_slot_batch_failed_of`）。
    #[test]
    fn the_pci_state_read_is_carried_into_the_region_record_and_live_slot_batch_failures() {
        let card_lost = GpuFailureCause::UncapturedErrorOrDeviceLost {
            reason: "替身：设备丢了".to_string(),
        };
        let pci_state_read = pci_state_read_under_the_root_not_present_for_tests("0000:01:00.0");
        let not_computed = || OutcomesOfTheBatchNotComputed::GpuFailed {
            device: card_identifier("0000:01:00.0"),
            cause: card_lost.clone(),
            pci_state_when_judged_unusable: pci_state_read.clone(),
        };
        assert_eq!(
            region_record_batch_failed_of(not_computed()),
            RegionRecordBatchFailed::GpuFailed {
                device: card_identifier("0000:01:00.0"),
                cause: card_lost.clone(),
                pci_state_when_judged_unusable: pci_state_read.clone(),
            }
        );
        assert_eq!(
            live_slot_batch_failed_of(not_computed()),
            LiveSlotBatchFailed::GpuFailed {
                device: card_identifier("0000:01:00.0"),
                cause: card_lost.clone(),
                pci_state_when_judged_unusable: pci_state_read.clone(),
            }
        );
    }

    /// 第 4 条「这张卡再一次被判用不了时才再读一次」：切换点「0000:41:00.0」，4 次都开不成——读 1 次（那一刻它的设备目录不在）；
    /// 之后连问 3 次、连交 2 批都带那一份，读次数不变。之后它的设备目录出现、使能位关着，再交同一组（它已试满，当场再判一次开不成）——
    /// 这一次读出新的那一份（目录在、使能位关），读次数变成 2，之后的问与批带的都是新的那一份，读次数照旧是 2。
    #[test]
    fn the_card_judged_unusable_again_is_read_again_and_no_new_judgement_reads_nothing_more() {
        let directories =
            crate::pci_state_of_the_gpu::PciDeviceDirectoriesMadeByTheTest::named_after_the_test(
                "judged-again",
            );
        let (mut background_gpu, _cards_handed_to_the_opening) =
            background_gpu_with_one_card_not_opened_at_the_mount_read_by(
                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
                    Err(reason_the_card_does_not_open_at_the_switch())
                },
                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
                directories.reader(),
            );
        let reading_of_the_card_now =
            |directories_now: &crate::pci_state_of_the_gpu::PciDeviceDirectoriesMadeByTheTest| {
                PciStateOfTheGpuReadWhenJudgedUnusable::Read(Box::new(
                    crate::pci_state_of_the_gpu::pci_state_under_the_device_directory(
                        "0000:41:00.0",
                        directories_now.device_directory_of("0000:41:00.0"),
                    ),
                ))
            };
        let answers_and_batches_carry =
            |background_gpu_asked: &mut BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
             pci_state_read: &PciStateOfTheGpuReadWhenJudgedUnusable| {
                (0..3).all(|_question| {
                    matches!(
                        background_gpu_asked.put_the_gpus_opened_since_the_switch_in_effect(
                            WaitForTheOpeningAtASwitch::UntilItEnds
                        ),
                        LatestSettingsHandedIn::NotOpened(not_opened)
                            if not_opened.pci_state_when_judged_unusable == *pci_state_read
                    )
                }) && (0..2).all(|_batch| {
                    matches!(
                        checksum_batch_and_the_devices_given_work(background_gpu_asked).0,
                        Err(ChecksumBatchFailed::GpuFailed {
                            pci_state_when_judged_unusable,
                            ..
                        }) if pci_state_when_judged_unusable == *pci_state_read
                    )
                })
            };
        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        let read_with_the_directory_not_present = reading_of_the_card_now(&directories);
        assert!(answers_and_batches_carry(
            &mut background_gpu,
            &read_with_the_directory_not_present
        ));
        assert_eq!(background_gpu.pci_state_readings_taken(), 1);
        let directories = directories.with_the_file("0000:41:00.0", "enable", "0\n");
        let read_with_the_device_disabled = reading_of_the_card_now(&directories);
        assert_ne!(
            read_with_the_device_disabled, read_with_the_directory_not_present,
            "前提：两次读到的不同"
        );
        background_gpu.use_the_settings_from_the_next_batch(&admitted);
        assert!(answers_and_batches_carry(
            &mut background_gpu,
            &read_with_the_device_disabled
        ));
        assert_eq!(background_gpu.pci_state_readings_taken(), 2);
    }

    /// 本机两张替身卡「0000:01:00.0」「0000:41:00.0」都参与，「0000:01:00.0」等提交时 panic（只供测试的钩子）；交一批，交回接住之后的那一份外壳。
    fn background_gpu_after_the_wait_on_the_card_at_one_panicked(
    ) -> BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests> {
        let mut card_at_one = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
        card_at_one.submission_waits_panicking = true;
        let card_at_forty_one =
            CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0);
        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
            vec![card_at_one, card_at_forty_one],
            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        );
        let (outcome, _devices_given_work) =
            checksum_batch_and_the_devices_given_work(&mut background_gpu);
        assert!(
            matches!(
                &outcome,
                Err(ChecksumBatchFailed::GpuFailed {
                    cause: GpuFailureCause::UncapturedErrorOrDeviceLost { .. },
                    ..
                })
            ),
            "前缀：等的那一下 panic 接住、报卡当场报出错：{outcome:?}"
        );
        background_gpu
    }

    /// 接住 panic 之后切换先因显存上限晾下「0000:01:00.0」、再请它回来：请回来要先等账上那次提交，等的那一步照记下的报卡当场报出错——
    /// 照换设置时量代价出错办，它不参与，下一批报成它的 GPU 出错（带那一刻读的 PCI 状态），它一段都不再交。
    #[test]
    fn a_switch_bringing_back_the_card_whose_wait_panicked_leaves_it_out_and_gives_it_no_work() {
        let mut background_gpu = background_gpu_after_the_wait_on_the_card_at_one_panicked();
        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
        let held_on_the_card_at_one = background_gpu.cards_on_this_host[0]
            .budget
            .memory_account()
            .bytes_held()
            .0;
        let leaving_it_out = settings_admitted_on_the_cards(
            &background_gpu,
            None,
            vec![held_on_the_card_at_one + smallest_piece_bytes - 1, 1 << 30],
        );
        background_gpu.use_the_settings_from_the_next_batch(&leaving_it_out);
        let bringing_it_back =
            settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
        background_gpu.use_the_settings_from_the_next_batch(&bringing_it_back);
        assert!(
            matches!(
                background_gpu.batch_not_split_over_the_gpus(),
                Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
                    device,
                    cause: GpuFailureCause::UncapturedErrorOrDeviceLost { .. },
                    ..
                }) if device == card_identifier("0000:01:00.0")
            ),
            "{:?}",
            background_gpu.batch_not_split_over_the_gpus()
        );
        assert_eq!(
            participating_devices_of(&background_gpu),
            vec![card_identifier("0000:41:00.0")]
        );
        let (outcome, _devices_given_work) =
            checksum_batch_and_the_devices_given_work(&mut background_gpu);
        assert!(
            matches!(
                &outcome,
                Err(ChecksumBatchFailed::GpuFailed { device, .. })
                    if *device == card_identifier("0000:01:00.0")
            ),
            "{outcome:?}"
        );
        assert_eq!(
            background_gpu.cards_on_this_host[0].submissions_made, 1,
            "接住之后这张卡一段都不再交"
        );
    }

    /// 接住 panic 之后切换照原样再点「0000:01:00.0」（上限装得下，不重量代价）：下一批交活之前那一等照记下的报卡当场报出错，这一批报成它的 GPU 出错，
    /// 它一段都不再交。
    #[test]
    fn a_switch_naming_again_the_card_whose_wait_panicked_gives_it_no_work() {
        let mut background_gpu = background_gpu_after_the_wait_on_the_card_at_one_panicked();
        let both_cards =
            settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
        background_gpu.use_the_settings_from_the_next_batch(&both_cards);
        let (outcome, _devices_given_work) =
            checksum_batch_and_the_devices_given_work(&mut background_gpu);
        assert!(
            matches!(
                &outcome,
                Err(ChecksumBatchFailed::GpuFailed {
                    device,
                    cause: GpuFailureCause::UncapturedErrorOrDeviceLost { .. },
                    ..
                }) if *device == card_identifier("0000:01:00.0")
            ),
            "{outcome:?}"
        );
        assert_eq!(
            background_gpu.cards_on_this_host[0].submissions_made, 1,
            "接住之后这张卡一段都不再交"
        );
    }
}
