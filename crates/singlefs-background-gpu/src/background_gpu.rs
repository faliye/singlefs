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
//! 卸载也不等锁外正在等的那一批：核心层建把手时交来调用方在停的标记（[`BackgroundGpuOverTheCards::give_up_waiting_for_the_cards_once_the_caller_is_stopping`]），
//! 每张开着的卡等一批的结果拆成短等、醒来读它，置了就放弃那一等、交回 [`GpuFailureCause::GaveUpWaitingForTheCallerIsStopping`]，
//! 那一次提交照没等到它做完留在账上，丢的时候这张卡照上面那一条整张泄漏（等法在 `crate::waiting_for_the_gpu`）。开卡那几等不读它。

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
    AdmittedGpuSettings, BackgroundComputeSettingsRefused, CallerIsStoppingTheBackgroundGpu,
    ClusterSegmentEntryKey, GpuComputedSegment, GpuDeviceIdentifier, GpuInventory,
    GpuMemoryInBytes, GpuShortOfFreeMemoryForTheSmallestPiece, GpuSubmissionsOfOneSegment,
    GpuSubmissionsOfTheBatch, PciStateOfTheGpuReadWhenJudgedUnusable,
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
    wait_for_completion_before_the_deadline, CallerStoppingSeenByTheWait, CompletionCheck,
    GpuWaitDeadline, MonotonicClock, ReopeningsOfAGpuNotOpened, SteadyClockOfThisProcess,
    TimeoutsOfTheBatchOnOneGpu, WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
};

mod batches_over_the_cards;
mod buffers_of_a_chunk_in_one_out_of_memory_scope;
mod cards_and_their_settings_between_batches;
mod gpu_in_use;
mod opening_of_the_gpus;
mod waiting_for_wgpu_futures_and_thread_clocks;

use buffers_of_a_chunk_in_one_out_of_memory_scope::*;
use gpu_in_use::*;
pub use opening_of_the_gpus::*;
pub use waiting_for_wgpu_futures_and_thread_clocks::*;

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
    /// 等一批的结果时调用方在停（核心层把手置了调用方在停的标记，运行库卸载、丢弃；`crate::waiting_for_the_gpu::CallerStoppingSeenByTheWait`），
    /// 这一等放弃了：那一次提交照没等到它做完留在这张卡的账上（卸载与切换关卡时这张卡整张泄漏、不析构，第 4 条「卸载时不等还在跑的提交」）。
    /// 不是等超过上限：不重交、不算进一批的超时次数，不判这张卡用不了、不读 PCI 状态；报给核心层是不归哪张卡的那一种。
    GaveUpWaitingForTheCallerIsStopping,
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
    /// 等一批的结果时读不读调用方在停的标记：建核心层的把手之前不读，把手交来标记之后读
    /// （[`Self::give_up_waiting_for_the_cards_once_the_caller_is_stopping`]）。开着的卡每张的账上都记着同一份，切换开成、收进来的卡收进来那一刻记上。
    caller_stopping_seen_by_the_waits_of_the_batches: CallerStoppingSeenByTheWait,
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

/// 建一段缓冲那一刻要设备做的几样：开一个只接显存不足的错误范围、建一份回读暂存、报一份缓冲建的时候要了多少字节、关上那个范围并交回等它接住的
/// 显存不足的那一个 future（等由调用方在一批的等的期限之内做，[`buffers_and_both_readbacks_created_within_one_out_of_memory_scope`]）。
/// 产品是 `wgpu::Device`；单测换上在内存里记账的替身（`tests::DeviceOfAChunkInMemoryForTests`）：它照 wgpu 错误范围的规矩
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

    /// 当场关上 `scope`，交回等它接住的显存不足的 future（交出一句原因；没接住的是 `None`）。不等：等多久、等的时候读不读调用方在停的标记归调用方。
    fn out_of_memory_captured_by(
        &self,
        scope: Self::OutOfMemoryScope,
    ) -> impl Future<Output = Option<String>>;
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

/// 一张卡上在开的一次开卡：在进程的那一张表里记着，丢它（那一次开卡跑完、开卡里 panic、或起不了那条线程）时划掉、叫醒排队等这张卡的。
struct OpeningUnderWayOnTheDevice {
    openings_in_this_process: OpeningsNotReturnedInThisProcess,
    opening_number: u64,
}

/// 一次挂载里每张卡切换开卡连着开不成了几次（第 4 条：一张卡至多开 [`ReopeningsOfAGpuNotOpened::openings_at_most`] 次，跨切换接着数、卸载重挂清零）：
/// 外壳与切换开卡线程共用一份，开不成一次就记一次（叫停的那几次没开、前一次开卡没回来而这一次没起的，都不记，[`opening_counts_toward_the_failed_openings`]），
/// 开成一次就把这张卡的清零；起不了开卡线程那一回记在头一张要开的卡上。
#[derive(Clone, Debug, Default)]
struct FailedOpeningsOfEachGpuInThisMount {
    failed_openings_of_each_device: Arc<Mutex<Vec<(GpuDeviceIdentifier, u32)>>>,
}

/// 挂着切换那一刻现查本机在的独显（[`DiscreteGpusPresentNow`]）：拿开卡用的那一份进程级 Vulkan 实例再枚举一次，在开卡时那个等的上限之内等。
struct AdaptersEnumeratedByTheInstance {
    vulkan_instance: VulkanInstanceOfThisProcess,
    wait_limit: Duration,
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

#[cfg(test)]
mod tests;
