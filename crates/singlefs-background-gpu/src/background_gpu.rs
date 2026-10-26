//! 起 GPU 的外壳（D26（后台整理与放置回收） 已定项 13 第 1–4、8 条）：挂载那一刻枚举 wgpu 适配器、判过校验、在本机每一张独立显卡上起设备与各段两份核的管线、
//! 量一次提交的主机 CPU 时间；一张卡没开成时，它是参与的卡就拒绝挂载，给了 `background_gpu_devices` 而没点它就跳过、报出来
//! （[`BackgroundGpu::gpus_not_opened_at_the_mount`]；判法在 `crate::gpus_of_this_host::gpus_opened_at_the_mount`，`records/2026-10-19-里程碑四GPU垃圾回收开工.md`
//! 第 45 批交回那一行的定法）。参与的卡由设置定（[`BackgroundGpu::participating_gpus`] 报出来），挂着切换时核心层交来新的一组
//! （[`BackgroundGpu::use_the_settings_from_the_next_batch`]，两批之间），从下一批起只在它点的卡上、照它给的显存上限与等的上限跑，不重开设备
//! （`records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 43 批交回那一行的定法；分活、切段、等的纯逻辑在 `crate::gpus_of_this_host`）。
//! 挂着的时候每一批按量出的代价分给参与的卡、按显存上限切段、两份核各算一次、逐条比对。三段都接上了：扫描里的校验和核对（[`BackgroundGpu::verify_unit_checksums`]）、区域扫描里的单元头解析与映射反查
//! （[`BackgroundGpu::classify_region_records`]）与选段打分的每段活槽数（[`BackgroundGpu::count_live_slots_per_cluster_segment`]）。
//! 本机的卡清单（核心层 `GpuInventory`）答的是本机全部独显与各自的显存总量（按总线地址排，挂载那一刻没开成的照列），连同其中没开成的那几张与原因：
//! 挂载与切换照它校验设置，切换可以点开包时没参与而开成了的卡；点名不在本机的卡就是点名的卡不存在，参与的卡里有没开成的照第 4 条拒。
//!
//! GPU 出错（设备丢了、wgpu 报了没接住的错、等不成、等超过上限、回读映射不成、核没写到某一条）时这一批交回出错：不重试、不换 CPU（第 4 条）；
//! 这时不动连续不确定的计数（这一批放弃了，不算一轮）。只有一张卡时两份核都在这张卡上跑；几张卡时每张卡在自己分到的那几条上跑两份核。
//! 每一处阻塞等 GPU 都在此刻设置给的上限之内等（开包时是 `BackgroundGpuSelection::background_gpu_wait_limit`，等法在 `crate::waiting_for_the_gpu`）：
//! 等 wgpu 的 future 用 std 写的最小等法（[`wait_for_the_wgpu_future`]），等提交用 wgpu 自己带限时的 `poll`，不依赖 `pollster`
//! （门禁 checker-implementation-disjoint 格第 ⑧ 条）。
//! 每一段切批之前先扣掉这张卡上本包别的段此刻占着的显存（`ChunkBoundsOfAGpu::with_the_room_left_after`），几段合在一张卡上占的不超过它的上限（第 3 条）。
//! 一段的提交等超过上限之后，它的缓冲在卡上可能还占着：这一段的账留到那次提交真在 GPU 上做完才放，下一批在这张卡上切段交活之前先在上限之内
//! 等它做完，等不到就交回出错、这张卡上一段都不交（`crate::memory_limit_and_chunks` 文件头）。

use std::future::Future;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::time::Duration;

use singlefs_core::allocator::AllocationRecord;
use singlefs_core::background_compute::{
    AdmittedGpuSettings, BackgroundComputeSettingsRefused, ClusterSegmentEntryKey,
    GpuDeviceIdentifier, GpuDeviceNotOpenedAtTheMount, GpuInventory, GpuMemoryInBytes,
};
use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::region_scan::{AllocationInRegion, CentralMappingEntry};
use wgpu::util::DeviceExt;

use crate::adapter_inventory::{
    discrete_gpus_of_the_enumeration, participating_gpus_admitted_by_the_mount_checks,
    AdapterDeviceKind, BackgroundGpuSelection, DiscreteGpuListRefused, DiscreteGpuOnThisHost,
    EnumeratedAdapterDescription,
};
use crate::checksum_kernels::{
    decoded_kernel_outcomes, outcome_buffer_before_the_kernel_writes, packed_checksum_chunk,
    reconciled_unit_checksums, ChecksumKernelOutcome, UnitChecksumToVerify,
    UnrecognizedVerdictWord, BITWISE_KERNEL_ENTRY_POINT, BITWISE_KERNEL_SOURCE,
    CHECKSUM_CHUNK_FOOTPRINT, INVOCATIONS_PER_WORKGROUP, OUTCOME_BYTES_PER_ENTRY,
    PARAMETER_BUFFER_BYTES, TABLE_DRIVEN_KERNEL_ENTRY_POINT, TABLE_DRIVEN_KERNEL_SOURCE,
};
use crate::gpus_of_this_host::{
    gpus_opened_at_the_mount, memory_limit_of_the_device_under_the_settings,
    DiscreteGpuNotOpenedAtTheMount, DiscreteGpusAfterTheOpening, GpuBudgetUnderTheSettings,
    ParticipatingGpusOnThisHost,
};
use crate::live_slot_kernels::{
    decoded_live_slot_outcomes, live_slot_chunk_footprint,
    live_slot_outcome_buffer_before_the_kernel_writes, live_slot_parameter_bytes,
    reconciled_live_slots_of_the_segments, segment_bound_words,
    AllocationRecordTablesForTheKernels, LIVE_SLOT_INVOCATIONS_PER_WORKGROUP,
    LIVE_SLOT_OUTCOME_BYTES, SEARCHING_SORTED_RECORDS_KERNEL_ENTRY_POINT,
    SEARCHING_SORTED_RECORDS_KERNEL_SOURCE, SEGMENT_BOUND_BYTES,
    WALKING_EVERY_RECORD_KERNEL_ENTRY_POINT, WALKING_EVERY_RECORD_KERNEL_SOURCE,
};
use crate::memory_limit_and_chunks::{
    input_buffer_bytes, ChunkFootprint, ChunksNotCutOnTheGpu, EntryLargerThanAChunkOnTheGpu,
    SubmissionOfTheChunk,
};
use crate::region_record_kernels::{
    decoded_region_record_outcomes, little_endian_bytes_of_the_words, packed_region_record_chunk,
    reconciled_region_record_classifications, record_is_one_the_region_scan_reads,
    region_record_chunk_footprint, region_record_outcome_buffer_before_the_kernel_writes,
    region_record_parameter_bytes, MappingKeyLongerThanTheKernelsHold, MappingTablesForTheKernels,
    RegionRecordToClassify, REGION_RECORD_INVOCATIONS_PER_WORKGROUP, REGION_RECORD_OUTCOME_BYTES,
    SEARCHING_KERNEL_ENTRY_POINT, SEARCHING_KERNEL_SOURCE, WALKING_KERNEL_ENTRY_POINT,
    WALKING_KERNEL_SOURCE,
};
use crate::two_kernel_reconciliation::{ConsecutiveUncertaintyCounts, ReconciledBatch};
use crate::waiting_for_the_gpu::{
    wait_for_completion_before_the_deadline, CompletionCheck, GpuWaitDeadline, MonotonicClock,
    SteadyClockOfThisProcess,
};
use crate::work_split_by_measured_cost::HostCpuNanosecondsPerSubmission;

/// 各段的着色器按字节偏移用 `u32` 寻址、整字那一段按「剩下的字节 ≥ 4」走：一个绑定不超过 2^31 字节，偏移加 4 也不回绕。
const LARGEST_STORAGE_BINDING_THE_KERNELS_ADDRESS_IN_BYTES: u64 = 1 << 31;
/// 量代价用的那一条：4096 字节全 0（大小取一个常见的单元块；它只用来量一次提交的主机 CPU，结果不用）。
const COST_PROBE_UNIT_BYTES: usize = 4096;
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
    /// wgpu 报了没接住的错（校验错、显存不够）或设备丢了。
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
    /// 给了列表而没点名的卡用不上不拒，跳过（[`BackgroundGpu::gpus_not_opened_at_the_mount`]）。
    GpuUnusable {
        device: GpuDeviceIdentifier,
        cause: GpuFailureCause,
    },
}

/// 一批校验和核对交不成。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChecksumBatchFailed {
    /// GPU 出错（含等 GPU 超过上限、前一次超时的提交在交这一批之前又没等到做完）：这一批放弃，不重试、不换 CPU（第 4 条）。
    GpuFailed {
        device: GpuDeviceIdentifier,
        cause: GpuFailureCause,
    },
    /// 一个单元单独占的缓冲就超过分到它的那张卡此刻剩下的显存或绑定上限：条款只写了一批放不下就切成几次交，
    /// 单个条目放不下怎么办没写，第一版不支持。这时一张卡都还没交活。`entry_position` 是它在这一批里的位置。
    UnitLargerThanAChunkOnTheGpu {
        device: GpuDeviceIdentifier,
        entry_position: usize,
        unit_bytes: u64,
    },
}

/// 一批区域扫描的单元头解析与映射反查交不成。后三种在一张卡都还没交活之前判出（第一版不支持的形态）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegionRecordBatchFailed {
    /// GPU 出错（含等 GPU 超过上限、前一次超时的提交在交这一批之前又没等到做完）：这一批放弃，不重试、不换 CPU（第 4 条）。
    GpuFailed {
        device: GpuDeviceIdentifier,
        cause: GpuFailureCause,
    },
    /// 映射里有一条 key 长过核装得下的字节数。
    MappingKeyLongerThanTheKernelsHold(MappingKeyLongerThanTheKernelsHold),
    /// 映射交给核的三个缓冲里最大的那个装不进分到记录的那张卡的绑定上限：整份映射每段都交一份，切批切不小它；条款没写，第一版不支持。
    MappingTablesLargerThanABindingOnTheGpu {
        device: GpuDeviceIdentifier,
        largest_buffer_bytes: u64,
    },
    /// 一条记录连同整份映射单独成一段就超过分到它的那张卡此刻剩下的显存或绑定上限：条款没写，第一版不支持。
    /// `entry_position` 是它在这一批里的位置，`unit_bytes` 是它读到的单元字节数。
    RecordLargerThanAChunkOnTheGpu {
        device: GpuDeviceIdentifier,
        entry_position: usize,
        unit_bytes: u64,
    },
}

/// 一批选段打分交不成。后两种在一张卡都还没交活之前判出（第一版不支持的形态）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveSlotBatchFailed {
    /// GPU 出错（含等 GPU 超过上限、前一次超时的提交在交这一批之前又没等到做完）：这一批放弃，不重试、不换 CPU（第 4 条）。
    GpuFailed {
        device: GpuDeviceIdentifier,
        cause: GpuFailureCause,
    },
    /// 分配记录交给核的两个缓冲里大的那个装不进分到段的那张卡的绑定上限：整份记录表每段都交一份，切批切不小它；条款没写，第一版不支持。
    RecordTablesLargerThanABindingOnTheGpu {
        device: GpuDeviceIdentifier,
        largest_buffer_bytes: u64,
    },
    /// 一个段连同整份记录表单独成一段就超过分到它的那张卡此刻剩下的显存或绑定上限：条款没写，第一版不支持。
    /// `entry_position` 是它在这一批里的位置。
    SegmentLargerThanAChunkOnTheGpu {
        device: GpuDeviceIdentifier,
        entry_position: usize,
    },
}

/// 一张参与的卡对外报的样子（第 2 条：参与的卡都用，并报出来）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticipatingGpuReport {
    pub identifier: GpuDeviceIdentifier,
    pub name: String,
    pub memory_limit: Option<GpuMemoryInBytes>,
    pub measured_cost: HostCpuNanosecondsPerSubmission,
}

/// 这一次挂载的后台 GPU：开包时开成的本机独显（按总线地址排），没开成而跳过的那几张，与此刻参与的那几张（设置定的，按参与的次序：
/// 没点名是总线地址的次序，点了名是点名的次序）。生命周期一次挂载，只住内存。
pub struct BackgroundGpu {
    /// 开成的那几张，与 `discrete_gpus_after_the_opening.opened` 一一对应、同序。
    gpus_on_this_host: Vec<GpuInUse>,
    discrete_gpus_after_the_opening: DiscreteGpusAfterTheOpening,
    participating: ParticipatingGpusOnThisHost,
    /// 设备由这个实例枚举出来，随后台 GPU 一起活着。
    #[allow(dead_code, reason = "只为让实例与设备同寿，不读它")]
    instance_the_devices_came_from: wgpu::Instance,
}

struct GpuInUse {
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
    measured_cost: HostCpuNanosecondsPerSubmission,
}

/// 一份核这一次的派发：管线、按绑定号交给它的缓冲（它的结果缓冲也在里面）、它写的那个结果缓冲。
struct KernelDispatch<'buffers> {
    pipeline: &'buffers wgpu::ComputePipeline,
    bound_buffers: Vec<(u32, &'buffers wgpu::Buffer)>,
    outcome_buffer: &'buffers wgpu::Buffer,
}

/// 挂载那一刻：枚举适配器、判卡清单与校验、在本机每一张独立显卡上起设备与各段两份核、量一次提交的主机 CPU 时间；参与的卡照 `selection`
/// （之后切换时核心层另交，[`BackgroundGpu::use_the_settings_from_the_next_batch`]）。每一处等 wgpu 都在 `selection.background_gpu_wait_limit` 之内。
/// 没参与的卡也开（之后的切换可能点它，切换不重开设备）；一张卡没开成，参与的就拒，给了列表而没点名的跳过、报出来
/// （[`gpus_opened_at_the_mount`]、[`BackgroundGpu::gpus_not_opened_at_the_mount`]）。
///
/// # Errors
/// [`BackgroundGpuOpenRefused`] 的四种。
pub fn open_background_gpu(
    selection: &BackgroundGpuSelection,
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
    let discrete_gpus = discrete_gpus_of_the_enumeration(&descriptions)
        .map_err(BackgroundGpuOpenRefused::DiscreteGpuListNotSupported)?;
    let participating = participating_gpus_admitted_by_the_mount_checks(selection, &discrete_gpus)
        .map_err(BackgroundGpuOpenRefused::SettingsRefused)?;
    let gpus_opened = gpus_opened_at_the_mount(
        discrete_gpus,
        &participating,
        |discrete_gpu, memory_limit_at_the_opening| {
            started_gpu_with_its_cost_measured(
                &adapters[discrete_gpu.position_in_the_enumeration],
                discrete_gpu.clone(),
                memory_limit_at_the_opening,
                selection.background_gpu_wait_limit,
            )
        },
    )?;
    Ok(BackgroundGpu {
        gpus_on_this_host: gpus_opened.opened,
        discrete_gpus_after_the_opening: gpus_opened.discrete_gpus,
        participating: gpus_opened.participating,
        instance_the_devices_came_from: instance,
    })
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
    // SAFETY: 物理设备句柄取自同一个 hal 适配器，那个适配器就是这个实例枚举出来的；`vkGetPhysicalDeviceMemoryProperties` 只读。
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
    let budget = GpuBudgetUnderTheSettings::with_nothing_held(
        adapter_limits
            .max_storage_buffer_binding_size
            .min(adapter_limits.max_buffer_size)
            .min(LARGEST_STORAGE_BINDING_THE_KERNELS_ADDRESS_IN_BYTES),
        largest_entry_count_per_dispatch,
        memory_limit,
        wait_limit,
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
        measured_cost: HostCpuNanosecondsPerSubmission(0),
    };
    gpu_in_use.measured_cost = gpu_in_use.measured_cost_of_one_submission()?;
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
    /// 这张卡分到的几条（每条的输入字节数）切段，交这几段之前调：前一次等超过上限、还在跑的那次提交先在此刻设置给的等的上限之内等它做完、
    /// 放掉它的账（第 3 条；等不到交回出错，不越过上限去交），再按此刻设置给的显存上限扣掉还占着的切（`GpuBudgetUnderTheSettings`）。
    fn chunks_once_the_submission_running_past_its_wait_completed(
        &mut self,
        entry_input_lengths_in_bytes: &[u64],
        footprint: ChunkFootprint,
    ) -> Result<Vec<std::ops::Range<usize>>, ChunksNotCutOnTheGpu> {
        let clock = SteadyClockOfThisProcess::starting_now();
        let device = &self.device;
        self.budget
            .chunks_cut_once_the_submission_running_past_its_wait_completed(
                &clock,
                |submission, time_left| {
                    completion_check_of_the_submission(device, submission, time_left)
                },
                entry_input_lengths_in_bytes,
                footprint,
            )
    }

    /// 挂载时量一次提交的主机 CPU 时间（第 2 条）：这条线程的 CPU 钟（不是挂钟），先空跑一次，再量 [`COST_MEASUREMENT_SUBMISSIONS`] 次取中位数。
    /// 每一次是一整段校验和核对的提交：建缓冲、编码两份核、交、等、回读、解码，与挂着时交一段走同一条路（[`Self::both_kernel_outcomes_of_a_chunk`]）。
    fn measured_cost_of_one_submission(
        &mut self,
    ) -> Result<HostCpuNanosecondsPerSubmission, GpuFailureCause> {
        let probe_unit_bytes = vec![0_u8; COST_PROBE_UNIT_BYTES];
        let probe = [UnitChecksumToVerify {
            entry_key: (),
            unit_bytes: probe_unit_bytes.as_slice(),
            expected_crc32c: 0,
        }];
        self.both_kernel_outcomes_of_a_chunk(&probe)?;
        let mut costs_in_nanoseconds = Vec::with_capacity(COST_MEASUREMENT_SUBMISSIONS);
        for _measurement in 0..COST_MEASUREMENT_SUBMISSIONS {
            let cpu_time_before = host_cpu_time_of_this_thread();
            self.both_kernel_outcomes_of_a_chunk(&probe)?;
            let cpu_time_spent = host_cpu_time_of_this_thread().saturating_sub(cpu_time_before);
            costs_in_nanoseconds.push(
                u64::try_from(cpu_time_spent.as_nanos())
                    .expect("一次提交的 CPU 时间装得进 u64 纳秒"),
            );
        }
        costs_in_nanoseconds.sort_unstable();
        Ok(HostCpuNanosecondsPerSubmission(
            costs_in_nanoseconds[COST_MEASUREMENT_SUBMISSIONS / 2],
        ))
    }

    fn buffer_with_contents(&self, contents: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents,
                usage,
            })
    }

    /// 两份核在同一次提交里各算一遍、各自的结果缓冲拷进各自的回读暂存，等做完、等两份映射好，交回两份读回的字节（按派发的次序），
    /// 连同这次提交等完之后在卡上还占不占显存（没等到它做完的，调用方照它把这一段的账留着）。
    /// 一次提交里的几处等共用一个期限，合起来不超过此刻设置给这张卡的等的上限。
    fn dispatch_both_kernels_and_read_back(
        &self,
        kernel_dispatches: [KernelDispatch<'_>; 2],
        workgroups: u32,
        outcome_bytes: u64,
    ) -> (
        Result<[Vec<u8>; 2], GpuFailureCause>,
        SubmissionOfTheChunk<wgpu::SubmissionIndex>,
    ) {
        let device = &self.device;
        let readbacks: [wgpu::Buffer; 2] = std::array::from_fn(|_kernel| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: outcome_bytes,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });
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
        for (kernel_dispatch, readback) in kernel_dispatches.iter().zip(&readbacks) {
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
        for readback in &readbacks {
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
            &readbacks,
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

    /// 校验和核对的一段（切批切出来的、装得下这张卡的那几条）交给这张卡：两份核在同一次提交里各算一遍，交回每条两份的结果，按段里的次序。
    /// 本包这一段占的缓冲先记进这张卡的账（超上限断言），那次提交做完放掉；没等到它做完的留着（`GpuMemoryAccount::settle_the_chunk_after_its_submission`）。
    fn both_kernel_outcomes_of_a_chunk<EntryKey>(
        &mut self,
        units: &[UnitChecksumToVerify<'_, EntryKey>],
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
        let (read_back, submission_of_the_chunk) = {
            let parameter_buffer =
                self.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM);
            let input_buffer =
                self.buffer_with_contents(&packed.input_bytes, wgpu::BufferUsages::STORAGE);
            let descriptor_buffer =
                self.buffer_with_contents(&descriptor_bytes, wgpu::BufferUsages::STORAGE);
            let table_driven_outcomes =
                self.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE);
            let bitwise_outcomes = self.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE);
            self.dispatch_both_kernels_and_read_back(
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
                entry_count_in_words.div_ceil(INVOCATIONS_PER_WORKGROUP),
                outcome_bytes,
            )
        };
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

    /// 区域扫描的一段记录交给这张卡：两份核在同一次提交里各算一遍，交回每条两份的分类，按段里的次序。整份映射随每一段交一份。
    /// 本包这一段占的缓冲先记进这张卡的账（超上限断言），那次提交做完放掉；没等到它做完的留着（`GpuMemoryAccount::settle_the_chunk_after_its_submission`）。
    fn both_kernel_classifications_of_a_chunk<EntryKey>(
        &mut self,
        records: &[RegionRecordToClassify<'_, EntryKey>],
        mapping_tables: &MappingTablesForTheKernels,
    ) -> Result<Vec<(AllocationInRegion, AllocationInRegion)>, GpuFailureCause> {
        assert!(!records.is_empty(), "切批不交空段");
        let packed = packed_region_record_chunk(records);
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
        let total_unit_bytes: usize = records
            .iter()
            .map(|record| record.unit_bytes.map_or(0, <[u8]>::len))
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
        let (read_back, submission_of_the_chunk) = {
            let parameter_buffer =
                self.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM);
            let input_buffer =
                self.buffer_with_contents(&packed.input_bytes, wgpu::BufferUsages::STORAGE);
            let descriptor_buffer =
                self.buffer_with_contents(&descriptor_bytes, wgpu::BufferUsages::STORAGE);
            let entry_buffer = self.buffer_with_contents(&entry_bytes, wgpu::BufferUsages::STORAGE);
            let location_item_buffer =
                self.buffer_with_contents(&location_item_bytes, wgpu::BufferUsages::STORAGE);
            let key_sorted_position_buffer =
                self.buffer_with_contents(&key_sorted_position_bytes, wgpu::BufferUsages::STORAGE);
            let walking_outcomes = self.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE);
            let searching_outcomes = self.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE);
            self.dispatch_both_kernels_and_read_back(
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
                record_count.div_ceil(REGION_RECORD_INVOCATIONS_PER_WORKGROUP),
                outcome_bytes,
            )
        };
        self.budget
            .memory_account_mut()
            .settle_the_chunk_after_its_submission(
                bytes_held_by_this_chunk,
                submission_of_the_chunk,
            );
        let [walking_bytes, searching_bytes] = read_back?;
        let walking = decoded_region_record_outcomes(&walking_bytes)
            .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
        let searching = decoded_region_record_outcomes(&searching_bytes)
            .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
        Ok(walking.into_iter().zip(searching).collect())
    }

    /// 选段打分的一段（切批切出来的、装得下这张卡的那几个段）交给这张卡：两份核在同一次提交里各数一遍，交回每段两份的活槽数，按段里的次序。
    /// 整份记录表随每一段交一份。本包这一段占的缓冲先记进这张卡的账（超上限断言），那次提交做完放掉；没等到它做完的留着
    /// （`GpuMemoryAccount::settle_the_chunk_after_its_submission`）。
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
        let (read_back, submission_of_the_chunk) = {
            let parameter_buffer =
                self.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM);
            let segment_bound_buffer =
                self.buffer_with_contents(&segment_bound_bytes, wgpu::BufferUsages::STORAGE);
            let record_buffer =
                self.buffer_with_contents(&record_bytes, wgpu::BufferUsages::STORAGE);
            let sorted_position_buffer =
                self.buffer_with_contents(&sorted_position_bytes, wgpu::BufferUsages::STORAGE);
            let walking_outcomes = self.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE);
            let searching_outcomes = self.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE);
            self.dispatch_both_kernels_and_read_back(
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
                segment_count.div_ceil(LIVE_SLOT_INVOCATIONS_PER_WORKGROUP),
                outcome_bytes,
            )
        };
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

impl BackgroundGpu {
    /// 挂载那一刻没开成而跳过的卡（给了 `background_gpu_devices` 而没点名的），按总线地址排，各带原因；没跳过一张是空。
    #[must_use]
    pub fn gpus_not_opened_at_the_mount(&self) -> &[DiscreteGpuNotOpenedAtTheMount] {
        &self.discrete_gpus_after_the_opening.not_opened
    }

    /// 此刻参与的卡，按参与的次序，带此刻设置给的显存上限与挂载时量出的一次提交的主机 CPU 时间。
    #[must_use]
    pub fn participating_gpus(&self) -> Vec<ParticipatingGpuReport> {
        self.participating
            .positions_on_this_host()
            .iter()
            .map(|position| {
                let gpu_in_use = &self.gpus_on_this_host[*position];
                ParticipatingGpuReport {
                    identifier: gpu_in_use.discrete_gpu.identifier.clone(),
                    name: gpu_in_use.discrete_gpu.name.clone(),
                    memory_limit: gpu_in_use.budget.memory_account().memory_limit(),
                    measured_cost: gpu_in_use.measured_cost,
                }
            })
            .collect()
    }

    /// 从下一批起照核心层交来的这一组跑（D26 已定项 13 第 1–3 条；核心层只在两批之间交）：参与的卡换成它点的那几张、按它的次序，
    /// 每张卡的显存上限与等的上限换成它给的（不参与的卡不设显存上限，它分不到活）。不重开设备、不重量代价，各卡的账照旧记着。
    ///
    /// # Panics
    /// 它点的卡有一张不在开成的独显里：核心层照这份实现答的卡清单（本机全部独显，连同没开成的那几张）校验过，参与的卡里有没开成的就拒了。
    pub fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
        let identifiers_on_this_host: Vec<GpuDeviceIdentifier> = self
            .gpus_on_this_host
            .iter()
            .map(|gpu_in_use| gpu_in_use.discrete_gpu.identifier.clone())
            .collect();
        self.participating =
            ParticipatingGpusOnThisHost::of_the_settings(settings, &identifiers_on_this_host);
        for gpu_in_use in &mut self.gpus_on_this_host {
            let memory_limit = memory_limit_of_the_device_under_the_settings(
                settings,
                &gpu_in_use.discrete_gpu.identifier,
            );
            gpu_in_use
                .budget
                .use_the_settings_from_the_next_batch(memory_limit, settings.wait_limit());
        }
    }

    /// 一批的条目按挂载时量出的代价分给参与的卡：（它在本机里的位置，分到的那一段相连的条目），按参与的次序。
    fn entry_ranges_by_measured_cost(
        &self,
        entry_count: usize,
    ) -> Vec<(usize, std::ops::Range<usize>)> {
        let measured_costs_on_this_host: Vec<HostCpuNanosecondsPerSubmission> = self
            .gpus_on_this_host
            .iter()
            .map(|gpu_in_use| gpu_in_use.measured_cost)
            .collect();
        self.participating
            .entry_ranges_by_measured_cost(&measured_costs_on_this_host, entry_count)
    }

    /// 一批单元交两份核各算一遍，交回每条两份的结果（按交进来的次序），不比对、不动计数：对拍与 [`Self::verify_unit_checksums`] 都用它。
    /// 按量出的代价分给各卡（每张卡一段相连的条目），每张卡上再按此刻剩下的显存切段：分到活的卡上有前一次等超过上限、还在跑的提交，
    /// 切段之前先等它做完（等不到交回 [`ChecksumBatchFailed::GpuFailed`]）；先把各卡的切法都算完，有单元放不下或等不到就一张卡都不交。
    ///
    /// # Errors
    /// [`ChecksumBatchFailed`] 的两种。
    pub fn both_kernel_outcomes_of_the_batch<EntryKey>(
        &mut self,
        units: &[UnitChecksumToVerify<'_, EntryKey>],
    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, ChecksumBatchFailed> {
        let ranges_per_gpu = self.entry_ranges_by_measured_cost(units.len());
        let mut chunks_per_gpu = Vec::with_capacity(ranges_per_gpu.len());
        for (position_on_this_host, range_of_the_gpu) in &ranges_per_gpu {
            let gpu_in_use = &mut self.gpus_on_this_host[*position_on_this_host];
            let device = gpu_in_use.discrete_gpu.identifier.clone();
            let unit_lengths: Vec<u64> = units[range_of_the_gpu.clone()]
                .iter()
                .map(|unit| u64::try_from(unit.unit_bytes.len()).expect("装得进 u64"))
                .collect();
            let chunks = gpu_in_use
                .chunks_once_the_submission_running_past_its_wait_completed(
                    &unit_lengths,
                    CHECKSUM_CHUNK_FOOTPRINT,
                )
                .map_err(|chunks_not_cut| match chunks_not_cut {
                    ChunksNotCutOnTheGpu::EarlierSubmissionStillRunning(cause) => {
                        ChecksumBatchFailed::GpuFailed {
                            device: device.clone(),
                            cause,
                        }
                    }
                    ChunksNotCutOnTheGpu::EntryLargerThanAChunk(
                        EntryLargerThanAChunkOnTheGpu {
                            entry_position,
                            input_bytes,
                        },
                    ) => ChecksumBatchFailed::UnitLargerThanAChunkOnTheGpu {
                        device: device.clone(),
                        entry_position: range_of_the_gpu.start + entry_position,
                        unit_bytes: input_bytes,
                    },
                })?;
            chunks_per_gpu.push(chunks);
        }
        let mut outcomes = Vec::with_capacity(units.len());
        for ((position_on_this_host, range_of_the_gpu), chunks) in
            ranges_per_gpu.iter().zip(chunks_per_gpu)
        {
            let gpu_in_use = &mut self.gpus_on_this_host[*position_on_this_host];
            let units_of_the_gpu = &units[range_of_the_gpu.clone()];
            for chunk in chunks {
                let chunk_outcomes = gpu_in_use
                    .both_kernel_outcomes_of_a_chunk(&units_of_the_gpu[chunk])
                    .map_err(|cause| ChecksumBatchFailed::GpuFailed {
                        device: gpu_in_use.discrete_gpu.identifier.clone(),
                        cause,
                    })?;
                outcomes.extend(chunk_outcomes);
            }
        }
        assert_eq!(
            outcomes.len(),
            units.len(),
            "每条都有两份结果，按卡的次序接回"
        );
        Ok(outcomes)
    }

    /// 扫描里的校验和核对一批（第 8 条）：两份核各算一遍、逐条比对；连续不确定到 N 轮的条目交核心层的 CRC-32C 现判。
    /// 交回逐条的判定与确认、不确定、交 CPU 判三个计数。GPU 出错（含等 GPU 超过上限）时交回出错，`uncertainty` 不动。
    ///
    /// # Errors
    /// [`ChecksumBatchFailed`] 的两种。
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
    /// 按量出的代价分给各卡，每张卡上再按此刻剩下的显存切段（整份映射每段都占一份）：分到活的卡上有前一次等超过上限、还在跑的提交，
    /// 切段之前先等它做完（等不到交回 [`RegionRecordBatchFailed::GpuFailed`]）；先把各卡的切法都算完，放不下或等不到就一张卡都不交。
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
        let footprint = region_record_chunk_footprint(&mapping_tables);
        let ranges_per_gpu = self.entry_ranges_by_measured_cost(records.len());
        let mut chunks_per_gpu = Vec::with_capacity(ranges_per_gpu.len());
        for (position_on_this_host, range_of_the_gpu) in &ranges_per_gpu {
            let gpu_in_use = &mut self.gpus_on_this_host[*position_on_this_host];
            let device = gpu_in_use.discrete_gpu.identifier.clone();
            if !range_of_the_gpu.is_empty()
                && mapping_tables.largest_buffer_bytes()
                    > gpu_in_use
                        .budget
                        .chunk_bounds()
                        .largest_storage_binding_in_bytes
            {
                return Err(
                    RegionRecordBatchFailed::MappingTablesLargerThanABindingOnTheGpu {
                        device: device.clone(),
                        largest_buffer_bytes: mapping_tables.largest_buffer_bytes(),
                    },
                );
            }
            let unit_lengths: Vec<u64> = records[range_of_the_gpu.clone()]
                .iter()
                .map(|record| {
                    u64::try_from(record.unit_bytes.map_or(0, <[u8]>::len)).expect("装得进 u64")
                })
                .collect();
            let chunks = gpu_in_use
                .chunks_once_the_submission_running_past_its_wait_completed(
                    &unit_lengths,
                    footprint,
                )
                .map_err(|chunks_not_cut| match chunks_not_cut {
                    ChunksNotCutOnTheGpu::EarlierSubmissionStillRunning(cause) => {
                        RegionRecordBatchFailed::GpuFailed {
                            device: device.clone(),
                            cause,
                        }
                    }
                    ChunksNotCutOnTheGpu::EntryLargerThanAChunk(
                        EntryLargerThanAChunkOnTheGpu {
                            entry_position,
                            input_bytes,
                        },
                    ) => RegionRecordBatchFailed::RecordLargerThanAChunkOnTheGpu {
                        device: device.clone(),
                        entry_position: range_of_the_gpu.start + entry_position,
                        unit_bytes: input_bytes,
                    },
                })?;
            chunks_per_gpu.push(chunks);
        }
        let mut classifications = Vec::with_capacity(records.len());
        for ((position_on_this_host, range_of_the_gpu), chunks) in
            ranges_per_gpu.iter().zip(chunks_per_gpu)
        {
            let gpu_in_use = &mut self.gpus_on_this_host[*position_on_this_host];
            let records_of_the_gpu = &records[range_of_the_gpu.clone()];
            for chunk in chunks {
                let chunk_classifications = gpu_in_use
                    .both_kernel_classifications_of_a_chunk(
                        &records_of_the_gpu[chunk],
                        &mapping_tables,
                    )
                    .map_err(|cause| RegionRecordBatchFailed::GpuFailed {
                        device: gpu_in_use.discrete_gpu.identifier.clone(),
                        cause,
                    })?;
                classifications.extend(chunk_classifications);
            }
        }
        assert_eq!(
            classifications.len(),
            records.len(),
            "每条都有两份分类，按卡的次序接回"
        );
        Ok(classifications)
    }

    /// 区域扫描里的单元头解析与映射反查一批（第 1、8 条）：两份核各分一次类、逐条比对；连续不确定到 N 轮的记录交核心层的区域扫描现判
    /// （`region_record_kernels::cpu_reference_classification`，按身份的索引只在头一次要它时建）。交回逐条的判定与三个计数。
    /// GPU 出错（含等 GPU 超过上限）或这一批交不成时交回出错，`uncertainty` 不动。
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
    /// 按量出的代价分给各卡，每张卡上再按此刻剩下的显存切段（整份记录表每段都占一份）：分到活的卡上有前一次等超过上限、还在跑的提交，
    /// 切段之前先等它做完（等不到交回 [`LiveSlotBatchFailed::GpuFailed`]）；先把各卡的切法都算完，放不下或等不到就一张卡都不交。
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
        let record_tables =
            AllocationRecordTablesForTheKernels::of_the_allocation_records(allocation_records);
        let footprint = live_slot_chunk_footprint(&record_tables);
        let ranges_per_gpu = self.entry_ranges_by_measured_cost(segments.len());
        let mut chunks_per_gpu = Vec::with_capacity(ranges_per_gpu.len());
        for (position_on_this_host, range_of_the_gpu) in &ranges_per_gpu {
            let gpu_in_use = &mut self.gpus_on_this_host[*position_on_this_host];
            let device = gpu_in_use.discrete_gpu.identifier.clone();
            if !range_of_the_gpu.is_empty()
                && record_tables.largest_buffer_bytes()
                    > gpu_in_use
                        .budget
                        .chunk_bounds()
                        .largest_storage_binding_in_bytes
            {
                return Err(
                    LiveSlotBatchFailed::RecordTablesLargerThanABindingOnTheGpu {
                        device: device.clone(),
                        largest_buffer_bytes: record_tables.largest_buffer_bytes(),
                    },
                );
            }
            let segment_bound_lengths: Vec<u64> = vec![SEGMENT_BOUND_BYTES; range_of_the_gpu.len()];
            let chunks = gpu_in_use
                .chunks_once_the_submission_running_past_its_wait_completed(
                    &segment_bound_lengths,
                    footprint,
                )
                .map_err(|chunks_not_cut| match chunks_not_cut {
                    ChunksNotCutOnTheGpu::EarlierSubmissionStillRunning(cause) => {
                        LiveSlotBatchFailed::GpuFailed {
                            device: device.clone(),
                            cause,
                        }
                    }
                    ChunksNotCutOnTheGpu::EntryLargerThanAChunk(
                        EntryLargerThanAChunkOnTheGpu {
                            entry_position,
                            input_bytes: _segment_bound_bytes,
                        },
                    ) => LiveSlotBatchFailed::SegmentLargerThanAChunkOnTheGpu {
                        device: device.clone(),
                        entry_position: range_of_the_gpu.start + entry_position,
                    },
                })?;
            chunks_per_gpu.push(chunks);
        }
        let mut live_slots = Vec::with_capacity(segments.len());
        for ((position_on_this_host, range_of_the_gpu), chunks) in
            ranges_per_gpu.iter().zip(chunks_per_gpu)
        {
            let gpu_in_use = &mut self.gpus_on_this_host[*position_on_this_host];
            let segments_of_the_gpu = &segments[range_of_the_gpu.clone()];
            for chunk in chunks {
                let chunk_live_slots = gpu_in_use
                    .both_kernel_live_slots_of_a_chunk(
                        &segments_of_the_gpu[chunk],
                        &record_tables,
                        unit_area,
                    )
                    .map_err(|cause| LiveSlotBatchFailed::GpuFailed {
                        device: gpu_in_use.discrete_gpu.identifier.clone(),
                        cause,
                    })?;
                live_slots.extend(chunk_live_slots);
            }
        }
        assert_eq!(
            live_slots.len(),
            segments.len(),
            "每段都有两份活槽数，按卡的次序接回"
        );
        Ok(live_slots)
    }

    /// 选段打分一批（第 1、8 条）：两份核各数一遍每段的活槽数、逐段比对；连续不确定到 N 轮的段交核心层的
    /// `live_slots_per_cluster_segment_by_the_records` 现判（`live_slot_kernels::cpu_reference_live_slots`）。交回逐段的判定与三个计数。
    /// GPU 出错（含等 GPU 超过上限）或这一批交不成时交回出错，`uncertainty` 不动。
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

/// 卡清单（核心层挂载与切换照它校验设置，D26（后台整理与放置回收） 已定项 13 第 2、4 条）：本机全部独显与各自的显存总量，按总线地址排
/// （挂载那一刻没开成的照列；没给 `background_gpu_devices` 时上限按这个次序一张卡一个），连同没开成的那几张与原因（[`DiscreteGpusAfterTheOpening`]）。
/// 切换可以点开包时没参与而开成了的卡；点名不在本机的卡照「点名的卡不存在」拒，参与的卡里有没开成的照第 4 条拒。
impl GpuInventory for BackgroundGpu {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        self.discrete_gpus_after_the_opening
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
    fn a_wgpu_future_still_pending_at_the_wait_limit_is_a_timeout_failure() {
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
}
