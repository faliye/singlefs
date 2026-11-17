# c996-code-r3：这一轮被判的产品代码相对上一轮开工快照的改动全文

左边是 `c996-code-r2` 与 `bggpu-audit-fix-code-r1` 两轮共用的开工快照那一刻的文件，右边是这一轮开工快照（`research/prompts/c996-code-r3-snapshot/crates-sha256.txt`）那一刻的文件；`diff -u` 原样，一份文件一段。

## crates/singlefs-background-gpu/src/background_gpu.rs

````diff
--- a/crates/singlefs-background-gpu/src/background_gpu.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu.rs
@@ -1,16 +1,18 @@
 //! 起 GPU 的外壳（D26（后台整理与放置回收） 已定项 13 第 1–4、8 条）：挂载那一刻枚举 wgpu 适配器、判过校验、只在参与的卡上起设备与各段两份核的管线、
-//! 量一次提交的主机 CPU 时间（欠账 C996 的定案「只开参与的卡」：给了 `background_gpu_devices` 是点名的卡，没给是本机全部独立显卡）；
-//! 参与的卡开不成就隔 1 分钟重开、至多重开 3 次，都不成才拒绝挂载（用户定案；判法在 `crate::gpus_of_this_host::gpus_opened_at_the_mount`）。给了 `background_gpu_devices` 时只判点名的卡：没点名的独显缺总线地址、缺设备本地堆、与别的独显报了同一个地址都不拒，
+//! 量一次提交的主机 CPU 时间（第 2 条「挂载时只开参与的卡」：给了 `background_gpu_devices` 是点名的卡，没给是本机全部独立显卡）；
+//! 参与的卡开不成就隔 1 分钟重开、至多重开 3 次，都不成才拒绝挂载（第 4 条；判法在 `crate::gpus_of_this_host::gpus_opened_at_the_mount`）。给了 `background_gpu_devices` 时只判点名的卡：没点名的独显缺总线地址、缺设备本地堆、与别的独显报了同一个地址都不拒，
 //! 不收进卡清单、报出来（[`BackgroundGpuOverTheCards::unnamed_discrete_gpus_left_off_the_list`]，判法 `crate::adapter_inventory::discrete_gpus_of_the_enumeration_under_the_named`）。
 //! 参与的卡由设置定（[`BackgroundGpuOverTheCards::participating_gpus`] 报出来），挂着切换时核心层交来新的一组
 //! （[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]，两批之间），从下一批起只在它点的卡上、照它给的显存上限与等的上限跑：
-//! 开着的卡不重开；它点到没开的卡，照用户定案「切换请求照收，在会话锁外开新卡」在另一条线程上开（开卡同挂载那一刻，[`OpeningOfAGpuAtASwitch`]；
+//! 开着的卡不重开；它点到没开的卡，照第 4 条「切换请求照收，在会话锁外开新卡」在另一条线程上开（开卡同挂载那一刻，[`OpeningOfAGpuAtASwitch`]；
 //! 开不成同样隔 1 分钟重开、至多重开 3 次；起不了开卡线程、开卡里 panic 都算那一次没开成），开着的这段时间照旧照交来之前那一组跑，开成了在之后的两批之间换上
 //! （[`BackgroundGpuOverTheCards::put_the_gpus_opened_since_the_switch_in_effect`]，答最近交来的那一组还在开卡、换上了、开不成三种）；
 //! 开卡的结局按交设置的次序认，为更早一次交来的开的、开不成的不报，照最近那一组重开；重开到上限都没开成就报出那张卡与每一次的原因，
-//! 此刻生效那一组之外开着的卡全部关掉，之后每一批报成它的 GPU 出错（核心层当场停 GC、报警），不自动再试，直到下一次换设置。
-//! 同一张卡上前一次开卡到点之后还没回来就不再起下一次（算那一次又没开成）；卸载与切到 `cpu` 时叫停切换开卡线程。
-//! 换上一组时不再点名的卡关掉（用户定案「切换时关掉」），切到 `cpu` 时开着的卡全部关掉（用户定案「全部关掉」），有提交还在跑的那张同卸载整张泄漏、
+//! 此刻生效那一组之外开着的卡全部关掉，照旧照交来之前那一组跑（核心层生效的也是它，当场停 GC、报警），不自动再试，直到下一次换设置。
+//! 一张卡在一次挂载里连着开了 4 次都没开成（跨切换接着数，中间开成一次就清零），之后再有切换点到它不再开，当场答开不成（[`GpuFailureCause::OpeningsInThisMountUsedUp`]），卸载重挂清零。
+//! 同一张卡上前一次开卡到点之后还没回来就不再起下一次（算那一次又没开成；这一张表是进程级的，各次挂载与切换共用，[`OpeningsNotReturnedInThisProcess`]）；
+//! 卸载与切到 `cpu` 时叫停切换开卡线程。
+//! 换上一组时不再点名的卡关掉（第 2 条「切换之后不再参与的卡，切换生效时关掉」），切到 `cpu` 时开着的卡全部关掉，有提交还在跑的那张同卸载整张泄漏、
 //! 累计进对外读得到的一份（[`BackgroundGpuOverTheCards::gpus_leaked_when_closed_at_a_switch`]）；之后再点它就重开（分活、切段、等的纯逻辑在 `crate::gpus_of_this_host`）。
 //! 挂着的时候每一批按卡的顺序填满参与的卡（前面的卡填到它照显存上限一次放得下的量，装不下的才给后面的卡；挂载时量的代价不用来分活）、按显存上限切段、两份核各算一次、逐条比对。三段都接上了：扫描里的校验和核对（[`BackgroundGpuOverTheCards::verify_unit_checksums`]）、
 //! 区域扫描里的单元头解析与映射反查（[`BackgroundGpuOverTheCards::classify_region_records`]）与选段打分的每段活槽数（[`BackgroundGpuOverTheCards::count_live_slots_per_cluster_segment`]）。
@@ -91,8 +93,8 @@
     decoded_live_slot_outcomes, live_slot_chunk_footprint,
     live_slot_outcome_buffer_before_the_kernel_writes, live_slot_parameter_bytes,
     live_slot_plan_on_a_gpu, live_slots_summed_over_the_record_slices,
-    reconciled_live_slots_of_the_segments, segment_bound_words,
-    AllocationRecordTablesForTheKernels, RecordSliceWithItsChunks,
+    reconciled_live_slots_of_the_segments, segment_bound_words, AllocationRecordCountOfATable,
+    AllocationRecordTablesForTheKernels, ClusterSegmentCountOfAGpu, RecordSliceWithItsChunks,
     LIVE_SLOT_INVOCATIONS_PER_WORKGROUP, LIVE_SLOT_OUTCOME_BYTES,
     SEARCHING_SORTED_RECORDS_KERNEL_ENTRY_POINT, SEARCHING_SORTED_RECORDS_KERNEL_SOURCE,
     SEGMENT_BOUND_BYTES, WALKING_EVERY_RECORD_KERNEL_ENTRY_POINT,
@@ -100,8 +102,8 @@
 };
 use crate::measured_host_cpu_time_per_submission::HostCpuNanosecondsPerSubmission;
 use crate::memory_limit_and_chunks::{
-    input_buffer_bytes, pieces_cut_to_fit_on_the_gpu, ChunkBoundsOfAGpu,
-    SmallestPieceLargerThanTheRoom, SubmissionOfTheChunk,
+    input_buffer_bytes, pieces_cut_to_fit_on_the_gpu, ChunkBoundsOfAGpu, EntryCountOfAChunk,
+    InputLengthOnTheHostInBytes, SmallestPieceLargerThanTheRoom, SubmissionOfTheChunk,
     COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS,
     COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION,
 };
@@ -130,12 +132,13 @@
 };
 
 /// 各段的着色器按字节偏移用 `u32` 寻址、整字那一段按「剩下的字节 ≥ 4」走：一个绑定不超过 2^31 字节，偏移加 4 也不回绕。
-const LARGEST_STORAGE_BINDING_THE_KERNELS_ADDRESS_IN_BYTES: u64 = 1 << 31;
+const LARGEST_STORAGE_BINDING_THE_KERNELS_ADDRESS_IN_BYTES: GpuMemoryInBytes =
+    GpuMemoryInBytes(1 << 31);
 /// 挂载那一刻量代价交的那一段：一条 4096 字节全 0 的单元（大小取一个常见的单元块；它只用来量一次提交的主机 CPU，结果不用），走校验和核对那一段的缓冲。
 /// 量的时候照这张卡开卡时的界切成几片逐段交（[`CostProbeChunk::pieces_under_the_bounds`]，与挂着时校验和那一段同一个切法），代价取各片之和
 /// （[`measured_cost_summed_over_the_pieces`]）；真建的缓冲与 `CHECKSUM_CHUNK_FOOTPRINT` 一致（[`CardOfTheBackgroundGpu::both_kernel_outcomes_of_a_checksum_chunk`] 核过）。
 pub const COST_PROBE_CHUNK: CostProbeChunk = CostProbeChunk {
-    input_bytes: 4096,
+    input_bytes: InputLengthOnTheHostInBytes(4096),
     footprint: CHECKSUM_CHUNK_FOOTPRINT,
 };
 /// 量几次取中位数：先空跑一次（管线刚编好时驱动的懒初始化不算进去），再量这么多次。
@@ -144,11 +147,25 @@
 const OUTCOME_BUFFER_USAGE: wgpu::BufferUsages =
     wgpu::BufferUsages::STORAGE.union(wgpu::BufferUsages::COPY_SRC);
 
-/// 一次 GPU 出错是什么（第 4 条）：调用方的决定只有一个——这一批放弃、后台任务停下并报错，成员只为报得清。
+/// 一次 GPU 出错是什么（第 4 条）：调用方的决定只有一个——这一批放弃、后台任务停下并报错，成员只为报得清；开卡那几种
+/// （起不了设备或编不了管线、起不了开卡线程、开卡 panic、前一次开卡没回来、叫停、连着试满）报给人时出路各不相同，各是一个成员。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum GpuFailureCause {
-    /// 起不了设备或编不了管线（驱动不对、显存不够）。
+    /// wgpu 起不了这张卡的设备或编不了管线（驱动不对、显存不够；量代价那一段最小的一片装不进绑定上限也报这一种）。
     DeviceOrPipelineDidNotStart { reason: String },
+    /// 系统起不了开卡的那条线程（`reason` 是起线程报的原话）：算这张卡这一次没开成，照重开。
+    OpeningThreadNotStarted { reason: String },
+    /// 开卡那一步里 panic 了（`said_by_the_panic` 是 panic 带的原话；驱动里的与本包自己的断言都算）：算这一次没开成，照重开。
+    OpeningPanicked { said_by_the_panic: String },
+    /// 同一张卡上前一次开卡到点之后还没回来（那条线程还挂在驱动里），这一次不再起：算这一次又没开成，照间隔往下数；出路只有重启进程。
+    EarlierOpeningOfTheDeviceNotReturned,
+    /// 切换开卡叫停了（卸载或切到 `cpu`），不再开：这一次的结局没人收，原因不报到哪里。
+    OpeningCalledOff,
+    /// 这张卡在这一次挂载里已连着开了 `failed_openings` 次都没开成（跨切换接着数，开成一次清零；[`ReopeningsOfAGpuNotOpened::openings_at_most`]），切换再点到它不再开：要卸载重挂。
+    OpeningsInThisMountUsedUp { failed_openings: u32 },
+    /// 一段建缓冲出错之后冲掉队列待写的那一次空提交，等满上限还没做完（已建的那几份缓冲连同暂存还占着卡上的显存）：这一段没交到卡上
+    /// （不算进重交数），算这一批这张卡一次等超过上限。
+    FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { wait_limit: Duration },
     /// wgpu 报了没接住的错（校验错）或设备丢了。
     UncapturedErrorOrDeviceLost { reason: String },
     /// 等这一次提交没等成（不是超时）。
@@ -166,19 +183,20 @@
     /// 已建的那几份连同暂存冲掉、做完了，它的账已经放掉（没等到做完的不报这一种，报等超过上限）。
     /// 挂着时外壳按更小的片重切重交，切到最小的一片都建不成就这一批不用这张卡、活转给别的卡（[`BackgroundGpuOverTheCards`]），不报出来。
     OutOfMemoryCreatingTheBuffers {
-        bytes_held_by_the_chunk: u64,
+        bytes_held_by_the_chunk: GpuMemoryInBytes,
         reason: String,
     },
     /// 一批在一张卡上第 4 次等超过上限（`waiting_for_the_gpu::RESUBMISSIONS_AFTER_A_TIMEOUT` 加 1；一批切成几片时各片的超时合起来算，
-    /// 交活之前、重交之前等前一次那一次没等到也算一次），`resubmissions` 是这期间真交到卡上的重交次数（重交之前那一等没等到、
-    /// 重交那一次建缓冲显存不足没交出去的不算，所以可以比 3 少）：这张卡用不了，
-    /// 交核心层当报警。`last_cause` 是最后那一次的原因（等超过上限的那两种之一）。
+    /// 交活之前、重交之前等前一次那一次没等到、冲掉待写的那一次空提交没等到也算一次），`resubmissions` 是这期间真交到卡上的重交次数（重交之前那一等没等到、
+    /// 重交那一次建缓冲出错没交出去的不算，所以可以比 3 少）：这张卡用不了，
+    /// 交核心层当报警。`last_cause` 是最后那一次的原因（等超过上限的那三种之一）。
     UnusableAfterResubmissions {
         resubmissions: u32,
         last_cause: Box<GpuFailureCause>,
     },
-    /// 挂着切换点到的这张卡开不成，隔 1 分钟重开 3 次也都没开成（用户定案；`waiting_for_the_gpu::gpu_opened_with_the_reopenings`）：
-    /// `causes_of_each_opening` 是每一次开不成的原因，按开的次序（头一次的在前，1 + 重开次数 个）。交核心层当报警，不自动再试。
+    /// 挂着切换点到的这张卡开不成，隔 1 分钟重开 3 次也都没开成（第 4 条；`waiting_for_the_gpu::gpu_opened_with_the_reopenings`）：
+    /// `causes_of_each_opening` 是每一次开不成的原因，按开的次序（头一次的在前，至多 1 + 重开次数 个；这一次挂载里早先已开不成过的扣掉，
+    /// 连着试满、这一次不再开的只一个 [`Self::OpeningsInThisMountUsedUp`]）。交核心层当报警，不自动再试。
     NotOpenedAfterTheReopenings {
         causes_of_each_opening: Vec<GpuFailureCause>,
     },
@@ -245,7 +263,7 @@
     pub device: GpuDeviceIdentifier,
     pub entry_position: usize,
     pub memory_limit: Option<GpuMemoryInBytes>,
-    pub smallest_piece_bytes: u64,
+    pub smallest_piece_bytes: GpuMemoryInBytes,
 }
 
 /// 一批校验和核对交不成。
@@ -387,8 +405,8 @@
     RefusedForTests { refusals_left: u32 },
 }
 
-/// 挂着切换点到的一张没开的卡，隔一段时间重开到上限也都没开成（用户定案「3 次都不成：GC 停下，报警并报出哪张卡、每次开不成的原因」）：
-/// 哪张卡、每一次开不成的原因（按开的次序，头一次的在前，1 + 重开次数 个）。
+/// 挂着切换点到的一张没开的卡，隔一段时间重开到上限也都没开成（第 4 条「3 次都不成，GC 停下，报警里报出哪张卡、每次开不成的原因」），
+/// 或它在这一次挂载里已连着试满、这一次不再开：哪张卡、每一次开不成的原因（按开的次序，头一次的在前，见 [`GpuFailureCause::NotOpenedAfterTheReopenings`]）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct GpuNotOpenedAfterTheReopenings {
     pub device: GpuDeviceIdentifier,
@@ -517,6 +535,8 @@
     opening_at_a_switch: Option<OpeningAtASwitch<Card>>,
     /// 起切换开卡那条线程的办法：产品恒是照 std 起。
     start_of_the_opening_thread: StartOfTheOpeningThread,
+    /// 这一次挂载里每张卡切换开卡连着开不成了几次（开卡线程也记，开成一次清零）：试满的卡再被点到就不再开、当场答开不成。
+    failed_openings_in_this_mount: FailedOpeningsOfEachGpuInThisMount,
     /// 切换关卡时泄漏的卡，按关的先后累计（同卸载的规矩：有提交还在跑的那张整张泄漏、不析构）。
     gpus_leaked_when_closed_at_a_switch: GpusLeakedWhenClosedAtASwitch,
     /// 上一次交回以来三段各自在卡上是怎么交的（[`Self::take_the_submissions_of_the_batch`]）。
@@ -767,10 +787,10 @@
     fn open_the_out_of_memory_scope(&self) -> Self::OutOfMemoryScope;
 
     /// 一份核的回读暂存：`outcome_bytes` 字节，映射来读、接结果缓冲的拷贝。
-    fn readback_of_one_kernel(&self, outcome_bytes: u64) -> Self::Buffer;
+    fn readback_of_one_kernel(&self, outcome_bytes: GpuMemoryInBytes) -> Self::Buffer;
 
     /// 这份缓冲建的时候要的字节数（建没建成都照要的报：建不成的那一份，显存不足由范围接住）。
-    fn bytes_of_the_buffer(&self, buffer: &Self::Buffer) -> u64;
+    fn bytes_of_the_buffer(&self, buffer: &Self::Buffer) -> GpuMemoryInBytes;
 
     /// 关上 `scope`，交回它接住的显存不足（一句原因；没接住的是 `None`）。
     ///
@@ -791,17 +811,17 @@
         self.push_error_scope(wgpu::ErrorFilter::OutOfMemory)
     }
 
-    fn readback_of_one_kernel(&self, outcome_bytes: u64) -> wgpu::Buffer {
+    fn readback_of_one_kernel(&self, outcome_bytes: GpuMemoryInBytes) -> wgpu::Buffer {
         self.create_buffer(&wgpu::BufferDescriptor {
             label: None,
-            size: outcome_bytes,
+            size: outcome_bytes.0,
             usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
             mapped_at_creation: false,
         })
     }
 
-    fn bytes_of_the_buffer(&self, buffer: &wgpu::Buffer) -> u64 {
-        buffer.size()
+    fn bytes_of_the_buffer(&self, buffer: &wgpu::Buffer) -> GpuMemoryInBytes {
+        GpuMemoryInBytes(buffer.size())
     }
 
     fn out_of_memory_captured_by(
@@ -834,8 +854,8 @@
 fn buffers_and_both_readbacks_created_within_one_out_of_memory_scope<Device, Buffers>(
     device: &Device,
     wait_limit: Duration,
-    bytes_held_by_this_chunk: u64,
-    outcome_bytes: u64,
+    bytes_held_by_this_chunk: GpuMemoryInBytes,
+    outcome_bytes: GpuMemoryInBytes,
     create_the_buffers: impl FnOnce() -> Buffers,
 ) -> Result<BuffersOfAChunk<Buffers, Device::Buffer>, GpuFailureCause>
 where
@@ -847,12 +867,12 @@
     let readbacks: [Device::Buffer; 2] =
         std::array::from_fn(|_kernel| device.readback_of_one_kernel(outcome_bytes));
     let captured = device.out_of_memory_captured_by(out_of_memory_scope, wait_limit);
-    let bytes_of_the_buffers_created: u64 = buffers
+    let bytes_of_the_buffers_created: GpuMemoryInBytes = buffers
         .as_ref()
         .iter()
         .map(|buffer_mapped_at_creation| {
             COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
-                * device.bytes_of_the_buffer(buffer_mapped_at_creation)
+                .bytes_of_the_copies(device.bytes_of_the_buffer(buffer_mapped_at_creation))
         })
         .chain(
             readbacks
@@ -882,13 +902,13 @@
 /// （[`COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION`]，切批与记账都算这一份）。
 fn buffer_mapped_at_creation_within_the_scope_of_the_caller(
     device: &wgpu::Device,
-    bytes: u64,
+    bytes: GpuMemoryInBytes,
     usage: wgpu::BufferUsages,
     write_the_contents: impl FnOnce(&mut wgpu::BufferViewMut),
 ) -> wgpu::Buffer {
     let buffer = device.create_buffer(&wgpu::BufferDescriptor {
         label: None,
-        size: bytes,
+        size: bytes.0,
         usage,
         mapped_at_creation: true,
     });
@@ -1003,7 +1023,7 @@
 }
 
 /// 挂载那一刻：枚举适配器、判卡清单与校验、在参与的卡上起设备与各段两份核、量一次提交的主机 CPU 时间（同
-/// [`open_background_gpu_with_the_memory_leaked_in_this_process`]，进程里还一张卡都没泄漏过；实例是调用方留着的 `vulkan_instance`）。
+/// [`open_background_gpu_with_the_memory_leaked_in_this_process`]，进程里还一张卡都没泄漏过、没有一次开卡挂住；实例是调用方留着的 `vulkan_instance`）。
 ///
 /// # Errors
 /// [`BackgroundGpuOpenRefused`] 的五种。
@@ -1014,6 +1034,7 @@
     open_background_gpu_with_the_memory_leaked_in_this_process(
         selection,
         &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        &OpeningsNotReturnedInThisProcess::with_none_not_returned(),
         vulkan_instance,
         |_gpus_to_open| {},
     )
@@ -1031,19 +1052,21 @@
 /// 卡数乘一张卡连同重开的最长等待）；枚举不成、卡清单或校验拒了的不报。
 /// 量代价出错时账上记着还在跑的提交的那张卡泄漏、不析构，这一次占着的字节当场记进 `memory_leaked_in_this_process`
 /// （`failure_of_the_measurement_dropping_or_leaking_the_card`）。
-/// 没参与的卡不开、不量代价（欠账 C996 的定案「只开参与的卡」）：卡清单照列它，之后的切换点到它时在会话锁外开（[`OpeningOfAGpuAtASwitch`]，
+/// 没参与的卡不开、不量代价（第 2 条「挂载时只开参与的卡」）：卡清单照列它，之后的切换点到它时在会话锁外开（[`OpeningOfAGpuAtASwitch`]，
 /// 同一份适配器与泄漏账）。参与的卡重开到上限都没开成就拒绝挂载（[`gpus_opened_at_the_mount`]）。参与的卡照开卡那一组给它的显存上限
 /// （先扣掉这个进程里之前几次挂载泄漏在它上面的字节，`memory_leaked_in_this_process`）把量代价那一段（[`COST_PROBE_CHUNK`]）切成几片逐片量、代价取各片之和；
 /// 显存上限连最小的一片都装不下的卡照开、不量代价、不参与、报出来（[`BackgroundGpuOverTheCards::gpus_left_out_under_their_memory_limit`]），挂载不因此拒绝；
 /// 参与的卡因此一张都不剩时照样交回，三段每一批都报「参与的卡一张都不剩」（[`BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut`]），
 /// 核心层挂载那一刻就读到、当场报警停着。丢掉它时泄漏的卡记进 `memory_leaked_in_this_process`。
 /// 枚举与之后现查本机在的独显都用 `vulkan_instance_handed_over` 交来的那一个实例（调用方在进程里留着的那一份，[`VulkanInstanceOfThisProcess`]），不另建。
+/// 一张卡上前一次开卡到点之后还没回来就不再起下一次，那一张表是 `openings_not_returned_in_this_process`（调用方在进程里留着的那一份，挂载与之后的切换开卡共用）。
 ///
 /// # Errors
 /// [`BackgroundGpuOpenRefused`] 的五种。
 pub fn open_background_gpu_with_the_memory_leaked_in_this_process(
     selection: &BackgroundGpuSelection,
     memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
+    openings_not_returned_in_this_process: &OpeningsNotReturnedInThisProcess,
     vulkan_instance_handed_over: &impl VulkanInstanceHandedToTheOpening,
     report_the_gpus_to_open: impl FnOnce(usize),
 ) -> Result<BackgroundGpu, BackgroundGpuOpenRefused> {
@@ -1065,7 +1088,11 @@
             &descriptions,
         )?;
     report_the_gpus_to_open(participating.len());
-    let mut open_a_gpu = opening_of_a_gpu_in_use(adapters, memory_leaked_in_this_process.clone());
+    let mut open_a_gpu = opening_of_a_gpu_in_use(
+        adapters,
+        memory_leaked_in_this_process.clone(),
+        openings_not_returned_in_this_process.clone(),
+    );
     let gpus_opened = gpus_opened_at_the_mount(
         discrete_gpus.listed,
         &participating,
@@ -1104,6 +1131,7 @@
 fn opening_of_a_gpu_in_use(
     adapters: Vec<wgpu::Adapter>,
     memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
+    openings_not_returned_in_this_process: OpeningsNotReturnedInThisProcess,
 ) -> impl FnMut(
     &DiscreteGpuOnThisHost,
     Option<GpuMemoryInBytes>,
@@ -1131,6 +1159,7 @@
             }
         },
         memory_leaked_in_this_process,
+        openings_not_returned_in_this_process,
     )
 }
 
@@ -1141,13 +1170,20 @@
 /// 到点当那张卡没开成，`waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit`）。处置量代价出错的卡在那条线程上：
 /// 到点之后那条线程跑完时交回的只是那个错，没有要析构的卡。
 /// 同一张卡上前一次开卡到点之后还没回来（那条线程还挂在驱动里），就不再起下一次：这一次当场算没开成（[`opening_not_started_while_an_earlier_one_has_not_returned`]），
-/// 照重开的间隔往下数（挂载与切换同一个规矩，同一张卡上同时在开的线程至多一条）。
+/// 照重开的间隔往下数（挂载与切换同一个规矩）。记着哪几张卡上有还没回来的开卡的那一张表是 `openings_not_returned_in_this_process`
+/// （进程开卡器持有、各次挂载与切换共用：前一次挂载里挂住的那张卡，下一次挂载同样不再起），同一张卡上同时在开的线程整个进程至多一条。
 fn opening_of_a_card_within_the_limit<Card, StartTheCard>(
     opening_limit: Duration,
-    mut start_of_the_card: impl FnMut(&DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>, Duration, u64) -> StartTheCard
+    mut start_of_the_card: impl FnMut(
+            &DiscreteGpuOnThisHost,
+            Option<GpuMemoryInBytes>,
+            Duration,
+            GpuMemoryInBytes,
+        ) -> StartTheCard
         + Send
         + 'static,
     memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
+    openings_not_returned_in_this_process: OpeningsNotReturnedInThisProcess,
 ) -> impl FnMut(
     &DiscreteGpuOnThisHost,
     Option<GpuMemoryInBytes>,
@@ -1160,10 +1196,9 @@
     Card: CardOfTheBackgroundGpu + Send + 'static,
     StartTheCard: FnOnce() -> Result<Card, GpuFailureCause> + Send + 'static,
 {
-    let devices_with_an_opening_not_returned = Arc::new(Mutex::new(Vec::new()));
     move |discrete_gpu, memory_limit_at_the_opening, wait_limit, cost_at_the_opening| {
         let Some(opening_not_returned) = OpeningOfTheDeviceNotReturned::noted_unless_one_already_is(
-            &devices_with_an_opening_not_returned,
+            &openings_not_returned_in_this_process,
             &discrete_gpu.identifier,
         ) else {
             return Err(opening_not_started_while_an_earlier_one_has_not_returned());
@@ -1188,19 +1223,55 @@
     }
 }
 
-/// 一张卡上一次还没回来的开卡：在 `devices_with_an_opening_not_returned` 里记着这张卡，丢它（那一次开卡跑完、或起不了那条线程）时划掉。
-struct OpeningOfTheDeviceNotReturned {
+/// 这个进程里哪几张卡上有一次到点之后还没回来的开卡（那条线程还挂在驱动里）：进程开卡器持有一份，各次挂载与挂着切换开卡共用
+/// （[`opening_of_a_card_within_the_limit`]）。几份把手共用同一张表。
+#[derive(Clone, Debug, Default)]
+pub struct OpeningsNotReturnedInThisProcess {
     devices_with_an_opening_not_returned: Arc<Mutex<Vec<GpuDeviceIdentifier>>>,
+}
+
+impl OpeningsNotReturnedInThisProcess {
+    /// 这个进程里还没有一次开卡挂住。
+    #[must_use]
+    pub fn with_none_not_returned() -> Self {
+        Self {
+            devices_with_an_opening_not_returned: Arc::new(Mutex::new(Vec::new())),
+        }
+    }
+
+    /// 两份把手是不是同一张表（进程开卡器交给各次开卡的是同一张）。
+    #[must_use]
+    pub fn is_the_same_table_as(&self, other: &Self) -> bool {
+        Arc::ptr_eq(
+            &self.devices_with_an_opening_not_returned,
+            &other.devices_with_an_opening_not_returned,
+        )
+    }
+
+    /// 此刻有一次开卡还没回来的卡。
+    #[must_use]
+    pub fn devices_with_an_opening_not_returned(&self) -> Vec<GpuDeviceIdentifier> {
+        self.devices_with_an_opening_not_returned
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner)
+            .clone()
+    }
+}
+
+/// 一张卡上一次还没回来的开卡：在进程的那一张表里记着这张卡，丢它（那一次开卡跑完、或起不了那条线程）时划掉。
+struct OpeningOfTheDeviceNotReturned {
+    openings_not_returned_in_this_process: OpeningsNotReturnedInThisProcess,
     device: GpuDeviceIdentifier,
 }
 
 impl OpeningOfTheDeviceNotReturned {
     /// 这张卡上没有还没回来的开卡，就记下这一次、交回它；有就交回 `None`（这一次不起）。
     fn noted_unless_one_already_is(
-        devices_with_an_opening_not_returned: &Arc<Mutex<Vec<GpuDeviceIdentifier>>>,
+        openings_not_returned_in_this_process: &OpeningsNotReturnedInThisProcess,
         device: &GpuDeviceIdentifier,
     ) -> Option<Self> {
-        let mut devices = devices_with_an_opening_not_returned
+        let mut devices = openings_not_returned_in_this_process
+            .devices_with_an_opening_not_returned
             .lock()
             .unwrap_or_else(std::sync::PoisonError::into_inner);
         if devices.contains(device) {
@@ -1208,7 +1279,7 @@
         }
         devices.push(device.clone());
         Some(Self {
-            devices_with_an_opening_not_returned: Arc::clone(devices_with_an_opening_not_returned),
+            openings_not_returned_in_this_process: openings_not_returned_in_this_process.clone(),
             device: device.clone(),
         })
     }
@@ -1216,7 +1287,8 @@
 
 impl Drop for OpeningOfTheDeviceNotReturned {
     fn drop(&mut self) {
-        self.devices_with_an_opening_not_returned
+        self.openings_not_returned_in_this_process
+            .devices_with_an_opening_not_returned
             .lock()
             .unwrap_or_else(std::sync::PoisonError::into_inner)
             .retain(|device| *device != self.device);
@@ -1225,14 +1297,13 @@
 
 /// 同一张卡上前一次开卡还没回来、这一次不起时报的原因（算这一次没开成，照重开的间隔往下数）。
 fn opening_not_started_while_an_earlier_one_has_not_returned() -> GpuFailureCause {
-    GpuFailureCause::DeviceOrPipelineDidNotStart {
-        reason: "这张卡上前一次开卡到点之后还没回来（还挂在驱动里），这一次不再起".to_string(),
-    }
+    GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned
 }
 
-/// 开一次卡（`open`），开卡里 panic 了就接住、算这一次没开成（原因带着 panic 的原话），不往上传：切换开卡线程照重开的规矩往下走，
-/// 不在两批之间持着会话锁时再 panic。
-fn opening_with_its_panic_caught<Opened>(
+/// 开一次卡（`open`），开卡里 panic 了就接住、算这一次没开成（[`GpuFailureCause::OpeningPanicked`]，带着 panic 的原话），不往上传：
+/// 挂载那一刻与切换开卡线程都照重开的规矩往下走，不在两批之间持着会话锁时再 panic（挂载与切换共用的开一张卡那一步
+/// `gpus_of_this_host::gpu_opened_under_its_memory_limit` 包着它）。
+pub(crate) fn opening_with_its_panic_caught<Opened>(
     open: impl FnOnce() -> Result<Opened, GpuFailureCause>,
 ) -> Result<Opened, GpuFailureCause> {
     std::panic::catch_unwind(std::panic::AssertUnwindSafe(open)).unwrap_or_else(|payload| {
@@ -1241,26 +1312,108 @@
             .map(|said| (*said).to_string())
             .or_else(|| payload.downcast_ref::<String>().cloned())
             .unwrap_or_else(|| "（panic 带的不是字符串）".to_string());
-        Err(GpuFailureCause::DeviceOrPipelineDidNotStart {
-            reason: format!("开卡 panic 了：{said_by_the_panic}"),
-        })
+        Err(GpuFailureCause::OpeningPanicked { said_by_the_panic })
     })
 }
 
 /// 切换开卡叫停之后（卸载、切到 `cpu`）不再开的那几次报的原因：这一次开卡的结局没人收，原因不报到哪里。
 fn opening_called_off() -> GpuFailureCause {
-    GpuFailureCause::DeviceOrPipelineDidNotStart {
-        reason: "切换开卡叫停了（卸载或切到 cpu），不再开".to_string(),
+    GpuFailureCause::OpeningCalledOff
+}
+
+/// 一次挂载里每张卡切换开卡连着开不成了几次（第 4 条：一张卡至多开 [`ReopeningsOfAGpuNotOpened::openings_at_most`] 次，跨切换接着数、卸载重挂清零）：
+/// 外壳与切换开卡线程共用一份，开不成一次就记一次（叫停的那几次没开，不记），开成一次就把这张卡的清零；起不了开卡线程那一回记在头一张要开的卡上。
+#[derive(Clone, Debug, Default)]
+struct FailedOpeningsOfEachGpuInThisMount {
+    failed_openings_of_each_device: Arc<Mutex<Vec<(GpuDeviceIdentifier, u32)>>>,
+}
+
+impl FailedOpeningsOfEachGpuInThisMount {
+    /// 这张卡这一次挂载里到此刻连着开不成了几次。
+    fn failed_openings_of(&self, device: &GpuDeviceIdentifier) -> u32 {
+        self.failed_openings_of_each_device
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner)
+            .iter()
+            .find(|(counted_device, _failed_openings)| counted_device == device)
+            .map_or(0, |(_device, failed_openings)| *failed_openings)
+    }
+
+    /// 记下这张卡又开不成一次。
+    fn note_a_failed_opening(&self, device: &GpuDeviceIdentifier) {
+        let mut failed_openings_of_each_device = self
+            .failed_openings_of_each_device
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner);
+        match failed_openings_of_each_device
+            .iter_mut()
+            .find(|(counted_device, _failed_openings)| counted_device == device)
+        {
+            Some((_device, failed_openings)) => *failed_openings += 1,
+            None => failed_openings_of_each_device.push((device.clone(), 1)),
+        }
+    }
+
+    /// 这张卡开成了：它连着开不成的次数清零，别的卡的不动。
+    fn forget_the_failed_openings_of(&self, device: &GpuDeviceIdentifier) {
+        self.failed_openings_of_each_device
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner)
+            .retain(|(counted_device, _failed_openings)| counted_device != device);
+    }
+
+    /// 照切换开卡线程开一次的结局记这张卡：开成清零；开不成照 [`opening_counts_toward_the_failed_openings`] 记一次或不记。
+    fn note_the_outcome_of_an_opening<Opened>(
+        &self,
+        device: &GpuDeviceIdentifier,
+        outcome: &Result<Opened, GpuFailureCause>,
+    ) {
+        match outcome {
+            Ok(_opened) => self.forget_the_failed_openings_of(device),
+            Err(cause) => {
+                if opening_counts_toward_the_failed_openings(cause) {
+                    self.note_a_failed_opening(device);
+                }
+            }
+        }
+    }
+}
+
+/// 这一次开不成算不算进一张卡连着开不成的次数：叫停的那一次没开，不算；别的都算。
+fn opening_counts_toward_the_failed_openings(cause: &GpuFailureCause) -> bool {
+    match cause {
+        GpuFailureCause::OpeningCalledOff => false,
+        GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
+        | GpuFailureCause::OpeningThreadNotStarted { .. }
+        | GpuFailureCause::OpeningPanicked { .. }
+        | GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned
+        | GpuFailureCause::OpeningsInThisMountUsedUp { .. }
+        | GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { .. }
+        | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
+        | GpuFailureCause::WaitForTheSubmissionFailed { .. }
+        | GpuFailureCause::WaitForTheGpuExceededTheLimit { .. }
+        | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
+        | GpuFailureCause::ReadbackMappingFailed { .. }
+        | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
+        | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
+        | GpuFailureCause::UnusableAfterResubmissions { .. }
+        | GpuFailureCause::NotOpenedAfterTheReopenings { .. } => true,
     }
 }
 
 /// 切换开卡那条线程上做的事：按给的次序逐张开 `discrete_gpus_to_open`（各带这一组给它的显存上限），开一张同挂载那一刻开卡那一步
-/// （[`gpu_opened_under_its_memory_limit`]，开一次交 `opening_of_a_gpu_at_a_switch`、等 GPU 的上限是这一组的 `wait_limit`），开不成就照 `reopenings`
-/// 重开、每次重开之前在这条线程上睡一个间隔（[`gpu_opened_with_the_reopenings`]）。有一张重开到上限都没开成就停在它，后面的不再开。
-/// 装不下最小的一片的照开、不量代价（换上那一组时照这张卡的账与开着的卡同一判，再判一次）。
-/// 开一次卡里 panic 了照这一次没开成算（[`opening_with_its_panic_caught`]）。`causes_of_each_start` 是起这条线程之前起不了线程的那几次
-/// （各算头一张卡一次没开成）：头一张卡的重开次数扣掉它们、原因排在前面，连同它们至多开 1 + 重开次数 次。
+/// （[`gpu_opened_under_its_memory_limit`]，开一次交 `opening_of_a_gpu_at_a_switch`、等 GPU 的上限是这一组的 `wait_limit`；开卡里 panic 了在那一步就接住、
+/// 算这一次没开成），开不成就照 `reopenings` 的间隔重开、每次重开之前在这条线程上睡一个间隔（[`gpu_opened_with_the_reopenings`]）。
+/// 一张卡这一次至多开它在这一次挂载里剩下的次数（[`ReopeningsOfAGpuNotOpened::openings_at_most`] 扣掉 `failed_openings_in_this_mount` 里它连着开不成的；
+/// 连着试满的卡不交到这里，[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`] 当场答开不成），
+/// 每开不成一次记进 `failed_openings_in_this_mount`、开成一次把它清零（叫停没开的那几次照 [`opening_counts_toward_the_failed_openings`] 不记）。有一张重开到上限都没开成就停在它，后面的不再开。
+/// 装不下最小的一片的照开、不量代价（换上那一组时照这张卡的账与开着的卡同一判，再判一次）。`causes_of_each_start` 是起这条线程之前起不了线程的那几次
+/// （各算头一张卡一次没开成、已记进它连着开不成的次数）：原因排在头一张卡的原因前面。
 /// 每一次开卡之前看 `called_off`：置上了就不再开（卸载、切到 `cpu`），也不再睡重开的间隔。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：开一张卡那一步、要开的卡、等的上限、重开的规矩、起线程之前的原因、叫停、泄漏账与连着开不成的次数"
+)]
 fn gpus_opened_at_a_switch<Card>(
     opening_of_a_gpu_at_a_switch: &mut OpeningOfAGpuAtASwitch<Card>,
     discrete_gpus_to_open: Vec<(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)>,
@@ -1269,17 +1422,23 @@
     causes_of_each_start: Vec<GpuFailureCause>,
     called_off: &AtomicBool,
     memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
+    failed_openings_in_this_mount: &FailedOpeningsOfEachGpuInThisMount,
 ) -> OutcomeOfTheOpeningAtASwitch<Card> {
     let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
     let mut opened = Vec::with_capacity(discrete_gpus_to_open.len());
     let mut causes_before_the_thread = causes_of_each_start;
     for (discrete_gpu, memory_limit) in discrete_gpus_to_open {
         let causes_before_this_card = std::mem::take(&mut causes_before_the_thread);
+        let openings_left_in_this_mount = reopenings.openings_at_most().saturating_sub(
+            failed_openings_in_this_mount.failed_openings_of(&discrete_gpu.identifier),
+        );
+        debug_assert!(
+            openings_left_in_this_mount > 0,
+            "连着试满的卡在交设置那一刻、为在等的那一组起开卡之前就答开不成，不为它起开卡线程"
+        );
         let reopenings_of_this_card = ReopeningsOfAGpuNotOpened {
             interval: reopenings.interval,
-            reopenings_at_most: reopenings
-                .reopenings_at_most
-                .saturating_sub(u32::try_from(causes_before_this_card.len()).unwrap_or(u32::MAX)),
+            reopenings_at_most: openings_left_in_this_mount.saturating_sub(1),
         };
         let opened_with_the_reopenings = gpu_opened_with_the_reopenings(
             reopenings_of_this_card,
@@ -1289,10 +1448,9 @@
                 }
             },
             || {
-                if called_off.load(Ordering::SeqCst) {
-                    return Err(opening_called_off());
-                }
-                opening_with_its_panic_caught(|| {
+                let opened_once = if called_off.load(Ordering::SeqCst) {
+                    Err(opening_called_off())
+                } else {
                     gpu_opened_under_its_memory_limit(
                         &discrete_gpu,
                         memory_limit,
@@ -1307,7 +1465,10 @@
                             )
                         },
                     )
-                })
+                };
+                failed_openings_in_this_mount
+                    .note_the_outcome_of_an_opening(&discrete_gpu.identifier, &opened_once);
+                opened_once
             },
         );
         match opened_with_the_reopenings {
@@ -1333,6 +1494,16 @@
     }
 }
 
+/// 一张卡在这一次挂载里已试满、不再开时报的原因（带它连着开不成的次数）。
+fn openings_in_this_mount_used_up(
+    failed_openings_in_this_mount: &FailedOpeningsOfEachGpuInThisMount,
+    device: &GpuDeviceIdentifier,
+) -> GpuFailureCause {
+    GpuFailureCause::OpeningsInThisMountUsedUp {
+        failed_openings: failed_openings_in_this_mount.failed_openings_of(device),
+    }
+}
+
 /// 起一张卡（`start_the_card`）、照开卡那一组给它的界量一次提交的代价（`cost_at_the_opening`）。量代价出错时这张卡照
 /// [`failure_of_the_measurement_dropping_or_leaking_the_card`] 处置：有提交还在跑就泄漏、这一次占着的字节记进 `memory_leaked_in_this_process`。
 ///
@@ -1494,7 +1665,7 @@
     discrete_gpu: DiscreteGpuOnThisHost,
     memory_limit: Option<GpuMemoryInBytes>,
     wait_limit: Duration,
-    bytes_leaked_by_earlier_mounts: u64,
+    bytes_leaked_by_earlier_mounts: GpuMemoryInBytes,
 ) -> Result<GpuInUse, GpuFailureCause> {
     let adapter_limits = adapter.limits();
     let (device, queue) = wait_for_the_wgpu_future(
@@ -1555,18 +1726,21 @@
     if let Some(reason) = fault_taken_from_the_record(&fault_reported_by_wgpu) {
         return Err(GpuFailureCause::DeviceOrPipelineDidNotStart { reason });
     }
-    let largest_entry_count_per_dispatch =
+    let largest_entry_count_per_dispatch = EntryCountOfAChunk(
         u64::from(adapter_limits.max_compute_workgroups_per_dimension)
             * u64::from(
                 INVOCATIONS_PER_WORKGROUP
                     .min(REGION_RECORD_INVOCATIONS_PER_WORKGROUP)
                     .min(LIVE_SLOT_INVOCATIONS_PER_WORKGROUP),
-            );
+            ),
+    );
     let budget = GpuBudgetUnderTheSettings::with_the_bytes_leaked_by_earlier_mounts(
-        adapter_limits
-            .max_storage_buffer_binding_size
-            .min(adapter_limits.max_buffer_size)
-            .min(LARGEST_STORAGE_BINDING_THE_KERNELS_ADDRESS_IN_BYTES),
+        GpuMemoryInBytes(
+            adapter_limits
+                .max_storage_buffer_binding_size
+                .min(adapter_limits.max_buffer_size)
+                .min(LARGEST_STORAGE_BINDING_THE_KERNELS_ADDRESS_IN_BYTES.0),
+        ),
         largest_entry_count_per_dispatch,
         memory_limit,
         wait_limit,
@@ -1637,8 +1811,14 @@
     Ok(read_back)
 }
 
-fn bytes_in_a_buffer(bytes: &[u8]) -> u64 {
-    u64::try_from(bytes.len()).expect("装得进 u64")
+/// 主机上要装进一个缓冲的内容有多长（对齐前）。
+fn bytes_in_a_buffer(bytes: &[u8]) -> InputLengthOnTheHostInBytes {
+    InputLengthOnTheHostInBytes::of_this_many_bytes(bytes.len())
+}
+
+/// 装着 `contents` 建的缓冲在卡上占的字节数：同 [`buffer_with_contents_created_within_the_scope_of_the_caller`] 的建法，按整字向上补、至少一个整字。
+fn bytes_on_the_gpu_of_a_buffer_with(contents: &[u8]) -> GpuMemoryInBytes {
+    input_buffer_bytes(bytes_in_a_buffer(contents))
 }
 
 /// 看一次这张卡上的这次提交在 GPU 上做没做完，最多等 `time_left`（wgpu 自己带限时的 `poll`）。
@@ -1663,6 +1843,7 @@
 
 impl GpuInUse {
     /// 量一次提交的主机 CPU 时间（第 2 条；挂载那一刻，或换设置让开卡时没量的卡参与时）：这条线程的 CPU 钟（不是挂钟）。
+    /// 账上不记着还在跑的提交（挂载那一刻刚开的卡没有；换设置时外壳先等它做完，[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]）。
     /// 交的那一段是 [`COST_PROBE_CHUNK`]，照这张卡此刻剩下的界（上限扣掉已占的，含这个进程里之前泄漏在它上面的）切成几片、几段
     /// （装得下就一段一片），每一次是各段各交一遍：建缓冲、编码两份核、交、等、回读、解码，与挂着时交一段走同一条路
     /// （[`CardOfTheBackgroundGpu::both_kernel_outcomes_of_a_checksum_chunk`]）；先空跑一遍，再量 [`COST_MEASUREMENT_SUBMISSIONS`] 遍，
@@ -1675,7 +1856,7 @@
     ) -> Result<HostCpuNanosecondsPerSubmission, GpuFailureCause> {
         let probe_unit_bytes = vec![
             0_u8;
-            usize::try_from(COST_PROBE_CHUNK.input_bytes)
+            usize::try_from(COST_PROBE_CHUNK.input_bytes.0)
                 .expect("量代价那一条 4096 字节，装得进 usize")
         ];
         let probe = [UnitChecksumToVerify {
@@ -1689,7 +1870,7 @@
                 |smallest_piece| GpuFailureCause::DeviceOrPipelineDidNotStart {
                     reason: format!(
                         "量代价那一段最小的一片 {} 字节装不进这张卡的绑定上限",
-                        smallest_piece.smallest_piece_bytes
+                        smallest_piece.smallest_piece_bytes.0
                     ),
                 },
             )?;
@@ -1718,8 +1899,8 @@
     /// （`GpuMemoryAccount::settle_the_chunk_whose_buffers_were_not_created`）。
     fn buffers_created_within_the_free_memory<Buffers: AsRef<[wgpu::Buffer]>>(
         &mut self,
-        bytes_held_by_this_chunk: u64,
-        outcome_bytes: u64,
+        bytes_held_by_this_chunk: GpuMemoryInBytes,
+        outcome_bytes: GpuMemoryInBytes,
         create_the_buffers: impl FnOnce(&Self) -> Buffers,
     ) -> Result<BuffersOfAChunk<Buffers, wgpu::Buffer>, GpuFailureCause> {
         let gpu_in_use: &Self = self;
@@ -1762,7 +1943,7 @@
         kernel_dispatches: [KernelDispatch<'_>; 2],
         readbacks: &[wgpu::Buffer; 2],
         workgroups: u32,
-        outcome_bytes: u64,
+        outcome_bytes: GpuMemoryInBytes,
     ) -> (
         Result<[Vec<u8>; 2], GpuFailureCause>,
         SubmissionOfTheChunk<wgpu::SubmissionIndex>,
@@ -1805,7 +1986,7 @@
                 0,
                 readback,
                 0,
-                outcome_bytes,
+                outcome_bytes.0,
             );
         }
         let submission = self.queue.submit(std::iter::once(encoder.finish()));
@@ -1941,29 +2122,33 @@
         let entry_count = packed.entry_count();
         let entry_count_in_words =
             u32::try_from(entry_count).expect("一段的条数不超过一次派发的上限，装得进 u32");
-        let outcome_bytes = u64::from(entry_count_in_words) * OUTCOME_BYTES_PER_ENTRY;
+        let entry_count_of_the_chunk = EntryCountOfAChunk(u64::from(entry_count_in_words));
+        let outcome_bytes = entry_count_of_the_chunk.bytes_of_the_entries(OUTCOME_BYTES_PER_ENTRY);
         let mut parameter_bytes =
-            vec![0_u8; usize::try_from(PARAMETER_BUFFER_BYTES).expect("16 字节")];
+            vec![0_u8; usize::try_from(PARAMETER_BUFFER_BYTES.0).expect("16 字节")];
         parameter_bytes[..4].copy_from_slice(&entry_count_in_words.to_le_bytes());
         let descriptor_bytes = packed.descriptor_bytes();
         let outcome_fill = outcome_buffer_before_the_kernel_writes(entry_count);
 
         let bytes_held_by_this_chunk = COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
-            * (PARAMETER_BUFFER_BYTES
-                + input_buffer_bytes(u64::try_from(packed.input_bytes.len()).expect("装得进 u64"))
-                + u64::try_from(descriptor_bytes.len()).expect("装得进 u64"))
-            + COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS * outcome_bytes;
+            .bytes_of_the_copies(
+                PARAMETER_BUFFER_BYTES
+                    .plus(bytes_on_the_gpu_of_a_buffer_with(&packed.input_bytes))
+                    .plus(bytes_on_the_gpu_of_a_buffer_with(&descriptor_bytes)),
+            )
+            .plus(
+                COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS.bytes_of_the_copies(outcome_bytes),
+            );
         assert_eq!(
             bytes_held_by_this_chunk,
             CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
-                u64::try_from(
+                InputLengthOnTheHostInBytes::of_this_many_bytes(
                     units
                         .iter()
                         .map(|unit| unit.unit_bytes.len())
                         .sum::<usize>()
-                )
-                .expect("装得进 u64"),
-                u64::from(entry_count_in_words),
+                ),
+                entry_count_of_the_chunk,
             ),
             "这一段真建的缓冲与切批按的占用一致"
         );
@@ -2060,7 +2245,9 @@
         let packed = packed_region_record_chunk(pieces);
         let record_count = u32::try_from(packed.record_count())
             .expect("一段的条数不超过一次派发的上限，装得进 u32");
-        let outcome_bytes = u64::from(record_count) * REGION_RECORD_OUTCOME_BYTES;
+        let record_count_of_the_chunk = EntryCountOfAChunk(u64::from(record_count));
+        let outcome_bytes =
+            record_count_of_the_chunk.bytes_of_the_entries(REGION_RECORD_OUTCOME_BYTES);
         let parameter_bytes = region_record_parameter_bytes(record_count, mapping_tables);
         let descriptor_bytes = little_endian_bytes_of_the_words(&packed.descriptor_words);
         let entry_bytes = little_endian_bytes_of_the_words(&mapping_tables.entry_words);
@@ -2072,13 +2259,19 @@
             region_record_outcome_buffer_before_the_kernel_writes(packed.record_count());
 
         let bytes_held_by_this_chunk = COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
-            * (bytes_in_a_buffer(&parameter_bytes)
-                + bytes_in_a_buffer(&packed.input_bytes)
-                + bytes_in_a_buffer(&descriptor_bytes)
-                + bytes_in_a_buffer(&entry_bytes)
-                + bytes_in_a_buffer(&location_item_bytes)
-                + bytes_in_a_buffer(&key_sorted_position_bytes))
-            + COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS * outcome_bytes;
+            .bytes_of_the_copies(
+                bytes_on_the_gpu_of_a_buffer_with(&parameter_bytes)
+                    .plus(bytes_on_the_gpu_of_a_buffer_with(&packed.input_bytes))
+                    .plus(bytes_on_the_gpu_of_a_buffer_with(&descriptor_bytes))
+                    .plus(bytes_on_the_gpu_of_a_buffer_with(&entry_bytes))
+                    .plus(bytes_on_the_gpu_of_a_buffer_with(&location_item_bytes))
+                    .plus(bytes_on_the_gpu_of_a_buffer_with(
+                        &key_sorted_position_bytes,
+                    )),
+            )
+            .plus(
+                COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS.bytes_of_the_copies(outcome_bytes),
+            );
         let total_unit_bytes: usize = pieces
             .iter()
             .map(|piece| piece.bytes().map_or(0, <[u8]>::len))
@@ -2086,8 +2279,8 @@
         assert_eq!(
             bytes_held_by_this_chunk,
             region_record_chunk_footprint(mapping_tables).bytes_held_by_a_chunk(
-                u64::try_from(total_unit_bytes).expect("装得进 u64"),
-                u64::from(record_count),
+                InputLengthOnTheHostInBytes::of_this_many_bytes(total_unit_bytes),
+                record_count_of_the_chunk,
             ),
             "这一段真建的缓冲与切批按的占用一致"
         );
@@ -2198,7 +2391,9 @@
         assert!(!segments.is_empty(), "切批不交空段");
         let segment_count =
             u32::try_from(segments.len()).expect("一段的段数不超过一次派发的上限，装得进 u32");
-        let outcome_bytes = u64::from(segment_count) * LIVE_SLOT_OUTCOME_BYTES;
+        let segment_count_of_the_chunk = EntryCountOfAChunk(u64::from(segment_count));
+        let outcome_bytes =
+            segment_count_of_the_chunk.bytes_of_the_entries(LIVE_SLOT_OUTCOME_BYTES);
         let parameter_bytes = live_slot_parameter_bytes(segment_count, record_tables, unit_area);
         let segment_bound_bytes = little_endian_bytes_of_the_words(&segment_bound_words(segments));
         let record_bytes = little_endian_bytes_of_the_words(&record_tables.record_words);
@@ -2207,16 +2402,20 @@
         let outcome_fill = live_slot_outcome_buffer_before_the_kernel_writes(segments.len());
 
         let bytes_held_by_this_chunk = COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
-            * (bytes_in_a_buffer(&parameter_bytes)
-                + bytes_in_a_buffer(&segment_bound_bytes)
-                + bytes_in_a_buffer(&record_bytes)
-                + bytes_in_a_buffer(&sorted_position_bytes))
-            + COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS * outcome_bytes;
+            .bytes_of_the_copies(
+                bytes_on_the_gpu_of_a_buffer_with(&parameter_bytes)
+                    .plus(bytes_on_the_gpu_of_a_buffer_with(&segment_bound_bytes))
+                    .plus(bytes_on_the_gpu_of_a_buffer_with(&record_bytes))
+                    .plus(bytes_on_the_gpu_of_a_buffer_with(&sorted_position_bytes)),
+            )
+            .plus(
+                COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS.bytes_of_the_copies(outcome_bytes),
+            );
         assert_eq!(
             bytes_held_by_this_chunk,
             live_slot_chunk_footprint(record_tables).bytes_held_by_a_chunk(
-                u64::from(segment_count) * SEGMENT_BOUND_BYTES,
-                u64::from(segment_count),
+                segment_count_of_the_chunk.input_bytes_of_the_entries(SEGMENT_BOUND_BYTES),
+                segment_count_of_the_chunk,
             ),
             "这一段真建的缓冲与切批按的占用一致"
         );
@@ -2336,6 +2535,7 @@
             settings_waiting_for_the_opening: None,
             opening_at_a_switch: None,
             start_of_the_opening_thread: StartOfTheOpeningThread::ByTheStandardLibrary,
+            failed_openings_in_this_mount: FailedOpeningsOfEachGpuInThisMount::default(),
             gpus_leaked_when_closed_at_a_switch: GpusLeakedWhenClosedAtASwitch::with_none_leaked(),
             submissions_since_the_last_take: GpuSubmissionsOfTheBatch::default(),
             gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped::not_yet_dropped(),
@@ -2417,9 +2617,11 @@
     /// 从下一批起照核心层交来的这一组跑（D26 已定项 13 第 1–3 条；核心层只在两批之间交）：参与的卡换成它点的那几张、按它的次序，
     /// 每张卡的显存上限与等的上限换成它给的。开着的卡不重开，各卡的账照旧记着。这一次交来记一个新的次序号。
     /// 它点的卡都开着，就当场换上（[`Self::put_the_settings_in_effect`]：不再点名的卡关掉）。它点到没开的卡（挂载时没开的，或之前切换、切到 `cpu` 关掉的），
-    /// 照用户定案「切换请求照收，在会话锁外开新卡」：这一刻不开卡，在另一条线程上按点的次序逐张开（开不成就隔一段时间重开，
+    /// 照第 4 条「切换请求照收，在会话锁外开新卡」：这一刻不开卡，在另一条线程上按点的次序逐张开（开不成就隔一段时间重开，
     /// [`Self::of_the_cards_opened_at_the_mount`] 给的重开规矩），开着的这段时间照旧照交来之前那一组跑；开完了在之后的两批之间换上
-    /// （[`Self::put_the_gpus_opened_since_the_switch_in_effect`]）。上一次切换的卡还在开时又交来一组，记下最近这一组，等那一次开完再看。
+    /// （[`Self::put_the_gpus_opened_since_the_switch_in_effect`]）。它点到的没开的卡里有一张在这一次挂载里已连着试满
+    /// （[`ReopeningsOfAGpuNotOpened::openings_at_most`] 次都没开成）：不再开，当场记成开不成（[`GpuFailureCause::OpeningsInThisMountUsedUp`]），
+    /// 此刻生效那一组之外开着的卡关掉、照旧照它跑。上一次切换的卡还在开时又交来一组，记下最近这一组，等那一次开完再看。
     /// 上一次切换留下的「重开到上限都没开成」在这一刻清掉（下一次换设置解除它）。
     pub fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
         self.latest_handing_in = self.latest_handing_in.next();
@@ -2430,6 +2632,14 @@
             self.put_the_settings_in_effect(settings);
             return;
         }
+        if let Some(used_up) =
+            self.gpu_whose_openings_in_this_mount_are_used_up(&discrete_gpus_to_open)
+        {
+            self.settings_waiting_for_the_opening = None;
+            self.close_the_cards_outside_the_settings_in_effect();
+            self.not_opened_after_the_reopenings_at_the_last_switch = Some(used_up);
+            return;
+        }
         self.settings_waiting_for_the_opening = Some(SettingsWaitingForTheOpening {
             handing_in_number: self.latest_handing_in,
             settings: settings.clone(),
@@ -2444,7 +2654,7 @@
         }
     }
 
-    /// 从下一批起一张卡都不参与（核心层切到 `cpu` 时交来；用户定案「切到 cpu 时开着的卡全部关掉」）：这一次也记一个新的次序号，
+    /// 从下一批起一张卡都不参与（核心层切到 `cpu` 时交来；第 2 条「从 gpu 切到 cpu……关掉全部开着的卡、丢掉还在开的那一组」）：这一次也记一个新的次序号，
     /// 在会话锁外开着的那一次叫停（开卡线程不再开下一次，开出来的没人收、在那条线程上照常丢），已送回还没取走的开卡结局里开成的卡收下再关，
     /// 还在等开卡的那一组丢掉，开着的卡全部关掉（有提交还在跑的那张整张泄漏，[`Self::close_the_cards`]），「重开到上限都没开成」与「换设置时量代价出错」清掉。
     /// 之后再交来一组，它点的卡都当没开的卡照重开的规矩开。
@@ -2472,7 +2682,8 @@
     }
 
     /// 两批之间（核心层每一批开头与整理两个入口判停没停之前，经 `BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`）：
-    /// 会话锁外为切换开卡的那一次开完了就在这一刻收下它的结局（[`Self::take_in_the_outcome_of_the_opening`]），起不了开卡线程的到了间隔就再起；
+    /// 会话锁外为切换开卡的那一次开完了就在这一刻收下它的结局（[`Self::take_in_the_outcome_of_the_opening`]），起不了开卡线程的到了间隔就再起
+    /// （那一次之后又交来了别的一组的，为最近那一组起，不为被盖掉的那一组起；最近没有在等开卡的一组就不起）；
     /// 然后交回最近交来的那一组此刻怎样了（[`LatestSettingsHandedIn`]）。`wait` 是 [`WaitForTheOpeningAtASwitch::NotAtAll`] 时还在开就不等，
     /// [`WaitForTheOpeningAtASwitch::UntilItEnds`] 时一直等到为最近交来的那一组开卡开完。
     ///
@@ -2517,12 +2728,20 @@
                             }
                         }
                     }
-                    self.start_the_opening_at_a_switch(
-                        handing_in_number,
-                        discrete_gpus_to_open,
-                        wait_limit,
-                        causes_of_each_start,
-                    );
+                    let latest_waiting_handing_in_number = self
+                        .settings_waiting_for_the_opening
+                        .as_ref()
+                        .map(|waiting| waiting.handing_in_number);
+                    if latest_waiting_handing_in_number == Some(handing_in_number) {
+                        self.start_the_opening_at_a_switch(
+                            handing_in_number,
+                            discrete_gpus_to_open,
+                            wait_limit,
+                            causes_of_each_start,
+                        );
+                    } else {
+                        self.start_the_opening_for_the_settings_waiting();
+                    }
                 }
                 OpeningAtASwitch::UnderWay {
                     handing_in_number,
@@ -2586,31 +2805,66 @@
 
     /// 收下为第 `handing_in_number` 次交来的设置开卡的结局：开成的卡先收下（[`Self::take_in_the_cards_opened_at_a_switch`]），再照最近交来、在等的那一组办——
     /// 开卡为的就是它（次序号相同）、有一张重开到上限都没开成：此刻生效那一组之外开着的卡全部关掉（连同为更早几次交来的设置开成的），记下那一张，
-    /// 之后每一批报成它的 GPU 出错，直到下一次换设置；它点的卡都开着了：换上它（不再点名的卡关掉）；它还点着没开的卡（开卡为的是更早一次交来的，
-    /// 那一次开不成的不报给它）：为它再开一次。没有在等的那一组（之后交来的一组不用开卡、已换上，或告知过一张卡都不参与）：此刻生效那一组之外开着的卡全部关掉。
+    /// 照旧照此刻生效那一组跑，直到下一次换设置；它点的卡都开着了：换上它（不再点名的卡关掉）；它还点着没开的卡（开卡为的是更早一次交来的，
+    /// 那一次开不成的不报给它）：此刻生效那一组与它都没点的开着的卡当场关掉（为被盖掉的那一组开成的），再为它开一次
+    /// （[`Self::start_the_opening_for_the_settings_waiting`]）。没有在等的那一组（之后交来的一组不用开卡、已换上，或告知过一张卡都不参与）：
+    /// 此刻生效那一组之外开着的卡全部关掉。
     fn take_in_the_outcome_of_the_opening(
         &mut self,
         handing_in_number: HandingInNumber,
         outcome: OutcomeOfTheOpeningAtASwitch<Card>,
     ) {
         self.take_in_the_cards_opened_at_a_switch(outcome.opened);
-        let Some(settings_waiting) = self.settings_waiting_for_the_opening.take() else {
+        let Some(settings_waiting) = self.settings_waiting_for_the_opening.as_ref() else {
             self.close_the_cards_outside_the_settings_in_effect();
             return;
         };
         if settings_waiting.handing_in_number == handing_in_number {
             if let Some(not_opened) = outcome.not_opened {
+                self.settings_waiting_for_the_opening = None;
                 self.close_the_cards_outside_the_settings_in_effect();
                 self.not_opened_after_the_reopenings_at_the_last_switch = Some(not_opened);
                 return;
             }
         }
+        let devices_named_by_the_settings_in_effect_or_waiting: Vec<GpuDeviceIdentifier> = self
+            .devices_named_by_the_settings_in_effect
+            .iter()
+            .cloned()
+            .chain(
+                settings_waiting
+                    .settings
+                    .participating_devices()
+                    .iter()
+                    .map(|participating| participating.device.clone()),
+            )
+            .collect();
+        self.close_the_cards(|device| {
+            !devices_named_by_the_settings_in_effect_or_waiting.contains(device)
+        });
+        self.start_the_opening_for_the_settings_waiting();
+    }
+
+    /// 为最近交来、在等开卡的那一组（有的话）接着办：它点的卡都开着了就换上；还点着没开的卡，其中有一张这一次挂载里已连着试满就记成开不成
+    /// （不再开，此刻生效那一组之外开着的卡关掉），不然为它起开卡（起线程之前没有起不了的原因）。没有在等的那一组就什么都不做。
+    /// 起不了开卡线程、到了间隔而之后又交来了别的一组时，也走这一步为最近那一组起。
+    fn start_the_opening_for_the_settings_waiting(&mut self) {
+        let Some(settings_waiting) = self.settings_waiting_for_the_opening.take() else {
+            return;
+        };
         let discrete_gpus_to_open =
             self.discrete_gpus_named_and_not_opened(&settings_waiting.settings);
         if discrete_gpus_to_open.is_empty() {
             self.put_the_settings_in_effect(&settings_waiting.settings);
             return;
         }
+        if let Some(used_up) =
+            self.gpu_whose_openings_in_this_mount_are_used_up(&discrete_gpus_to_open)
+        {
+            self.close_the_cards_outside_the_settings_in_effect();
+            self.not_opened_after_the_reopenings_at_the_last_switch = Some(used_up);
+            return;
+        }
         let waiting_handing_in_number = settings_waiting.handing_in_number;
         let wait_limit = settings_waiting.settings.wait_limit();
         self.settings_waiting_for_the_opening = Some(settings_waiting);
@@ -2622,6 +2876,31 @@
         );
     }
 
+    /// `discrete_gpus_to_open` 里头一张在这一次挂载里已连着试满（开不成的次数到了 [`ReopeningsOfAGpuNotOpened::openings_at_most`]）的卡，
+    /// 记成它开不成（[`GpuFailureCause::OpeningsInThisMountUsedUp`]，只一个原因）；都没试满交回 `None`。
+    fn gpu_whose_openings_in_this_mount_are_used_up(
+        &self,
+        discrete_gpus_to_open: &[(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)],
+    ) -> Option<GpuNotOpenedAfterTheReopenings> {
+        let openings_at_most = self.reopenings_of_a_gpu_not_opened.openings_at_most();
+        discrete_gpus_to_open
+            .iter()
+            .find(|(discrete_gpu, _memory_limit)| {
+                self.failed_openings_in_this_mount
+                    .failed_openings_of(&discrete_gpu.identifier)
+                    >= openings_at_most
+            })
+            .map(
+                |(discrete_gpu, _memory_limit)| GpuNotOpenedAfterTheReopenings {
+                    device: discrete_gpu.identifier.clone(),
+                    causes_of_each_opening: vec![openings_in_this_mount_used_up(
+                        &self.failed_openings_in_this_mount,
+                        &discrete_gpu.identifier,
+                    )],
+                },
+            )
+    }
+
     /// 关掉此刻生效那一组没点的开着的卡（[`Self::close_the_cards`]）。
     fn close_the_cards_outside_the_settings_in_effect(&mut self) {
         let devices_named_by_the_settings_in_effect =
@@ -2668,8 +2947,9 @@
 
     /// 在另一条线程上为第 `handing_in_number` 次交来的设置开 `discrete_gpus_to_open` 那几张（按给的次序逐张开，开一张同挂载那一刻开卡那一步，
     /// [`gpus_opened_at_a_switch`]；开不成就照重开的规矩重开，每次重开之前在那条线程上睡一个间隔；有一张重开到上限都没开成，后面的就不再开）。
-    /// 不等它：调用即返回。`causes_of_each_start` 是之前几次起不了这条线程的原因。起不了那条线程，就算头一张要开的卡这一次没开成（原因是起不了线程的那个错）：
-    /// 没到重开的上限就隔重开的间隔、在之后的两批之间再起（[`OpeningAtASwitch::ThreadNotStarted`]），到了上限结局就定成头一张开不成。
+    /// 不等它：调用即返回。`causes_of_each_start` 是之前几次起不了这条线程的原因。起不了那条线程，就算头一张要开的卡这一次没开成
+    /// （[`GpuFailureCause::OpeningThreadNotStarted`]，记进它这一次挂载里连着开不成的次数）：它还没试满就隔重开的间隔、在之后的两批之间再起
+    /// （[`OpeningAtASwitch::ThreadNotStarted`]），试满了结局就定成头一张开不成。
     fn start_the_opening_at_a_switch(
         &mut self,
         handing_in_number: HandingInNumber,
@@ -2679,6 +2959,7 @@
     ) {
         let opening_of_a_gpu_at_a_switch = Arc::clone(&self.opening_of_a_gpu_at_a_switch);
         let memory_leaked_in_this_process = self.memory_leaked_in_this_process.clone();
+        let failed_openings_in_this_mount = self.failed_openings_in_this_mount.clone();
         let reopenings = self.reopenings_of_a_gpu_not_opened;
         let called_off = Arc::new(AtomicBool::new(false));
         let called_off_seen_by_the_thread = Arc::clone(&called_off);
@@ -2697,6 +2978,7 @@
                 causes_of_each_start_seen_by_the_thread,
                 &called_off_seen_by_the_thread,
                 &memory_leaked_in_this_process,
+                &failed_openings_in_this_mount,
             );
             drop(opening_of_a_gpu);
             // 送不出去只有一种：后台 GPU 已经丢了（卸载）或切到了 `cpu`，开出来的卡没人要，在这里照常丢。
@@ -2714,15 +2996,21 @@
             }
             Err(error) => error,
         };
+        let first_device_to_open = discrete_gpus_to_open
+            .first()
+            .map(|(discrete_gpu, _memory_limit)| discrete_gpu.identifier.clone())
+            .expect("只为点到没开的卡的那一组开卡：至少一张");
         let mut causes_of_each_start = causes_of_each_start;
-        causes_of_each_start.push(GpuFailureCause::DeviceOrPipelineDidNotStart {
+        causes_of_each_start.push(GpuFailureCause::OpeningThreadNotStarted {
             reason: format!("起不了切换开卡的那条线程：{error}"),
         });
-        let starts_at_most = usize::try_from(reopenings.reopenings_at_most)
-            .map_or(usize::MAX, |reopenings_at_most| {
-                reopenings_at_most.saturating_add(1)
-            });
-        if causes_of_each_start.len() < starts_at_most {
+        self.failed_openings_in_this_mount
+            .note_a_failed_opening(&first_device_to_open);
+        if self
+            .failed_openings_in_this_mount
+            .failed_openings_of(&first_device_to_open)
+            < reopenings.openings_at_most()
+        {
             self.opening_at_a_switch = Some(OpeningAtASwitch::ThreadNotStarted {
                 handing_in_number,
                 discrete_gpus_to_open,
@@ -2732,10 +3020,6 @@
             });
             return;
         }
-        let first_device_to_open = discrete_gpus_to_open
-            .first()
-            .map(|(discrete_gpu, _memory_limit)| discrete_gpu.identifier.clone())
-            .expect("只为点到没开的卡的那一组开卡：至少一张");
         self.opening_at_a_switch = Some(OpeningAtASwitch::EndedWithoutAThread {
             handing_in_number,
             outcome: OutcomeOfTheOpeningAtASwitch {
@@ -2748,7 +3032,7 @@
         });
     }
 
-    /// 关掉 `is_closed` 判成要关的开着的卡（用户定案「切换之后不再参与的卡关不关」=「切换时关掉」）：它从开着的那几张挪回没开的那几张（卡清单照列，
+    /// 关掉 `is_closed` 判成要关的开着的卡（第 2 条「切换之后不再参与的卡，切换生效时关掉」）：它从开着的那几张挪回没开的那几张（卡清单照列，
     /// 之后再点它就重开），设备丢掉；上面还记着一次等超过上限、没在 GPU 上做完的提交的那张不丢、整张泄漏，这一刻占着的字节记进这个进程的泄漏账
     /// （同卸载，`gpus_of_this_host::gpus_dropped_leaking_those_still_running`），并累计进对外读得到的那一份
     /// （[`Self::gpus_leaked_when_closed_at_a_switch`]）。参与的卡照它们的标识在剩下的卡里重找位置，次序不变。
@@ -2800,8 +3084,9 @@
     /// 换上 `settings`（它点的卡都开着）：不再点名的开着的卡关掉（[`Self::close_the_cards`]），参与的卡换成它点的那几张、按它的次序，
     /// 每张卡的显存上限与等的上限换成它给的（不参与的卡不设显存上限，它分不到活）。
     /// 显存上限（扣掉卡上已占的）连最小的一片都装不下的卡不用、报出来（[`Self::gpus_left_out_under_their_memory_limit`]）；开卡时因显存上限没量代价、
-    /// 或上一组设置因显存上限晾下而这一组给了装得下的上限的卡，照新上限量一次（量不量照旧、量出的代价不用来分活），量出错的这张卡不参与，
-    /// 之后每一批报成它的 GPU 出错，直到下一次换设置。请回来的卡照参与的次序重新参与分活。
+    /// 或上一组设置因显存上限晾下而这一组给了装得下的上限的卡，先等账上还在跑的那次提交做完、放账，再照新上限量一次（量不量照旧、量出的代价不用来分活），
+    /// 等不到或量出错的这张卡不参与，之后每一批报成它的 GPU 出错（[`BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch`]），直到下一次换设置。
+    /// 请回来的卡照参与的次序重新参与分活。
     fn put_the_settings_in_effect(&mut self, settings: &AdmittedGpuSettings) {
         let devices_named_by_the_settings: Vec<GpuDeviceIdentifier> = settings
             .participating_devices()
@@ -2843,11 +3128,15 @@
                 continue;
             }
             // 第 3 条「挂着切换给了它装得下的上限，从下一批起照新上限重新量代价」：上一组设置因显存上限晾下的卡请回来时照新上限再量一次
-            // 量出的代价不用来分活，量不成照旧不参与。
+            // 量出的代价不用来分活，量不成照旧不参与。量之前先在这一组给的等的上限之内等账上还在跑的那次提交（前一次等超过上限、或前一次量代价超时留下的）
+            // 做完、放账（同交活之前那一等）：量代价那一段要记账，账上记着还在跑的提交时不交；等不到就照量代价出错办。
             let is_brought_back_after_being_left_out = devices_left_out_under_the_settings_before
                 .contains(&card.discrete_gpu().identifier);
             if card.measured_cost().is_none() || is_brought_back_after_being_left_out {
-                if let Err(cause) = card.measure_the_cost_of_one_submission() {
+                let measured = card
+                    .room_once_the_submission_running_past_its_wait_completed()
+                    .and_then(|_room_left| card.measure_the_cost_of_one_submission());
+                if let Err(cause) = measured {
                     positions_left_out.push(*position);
                     cost_not_measured
                         .get_or_insert((card.discrete_gpu().identifier.clone(), cause));
@@ -2866,9 +3155,11 @@
     /// 告知一张卡都不参与之后（切到 `cpu`）没有生效的那一组，核心层不问它。
     ///
     /// # Panics
-    /// 参与的卡一张都不剩，却既没有量代价出错，也没有因显存上限不用的卡：核心层校验过的一组至少点一张卡，换上的那一组点的卡都开着，
-    /// 没参与的只有这两种原因；告知一张卡都不参与（[`Self::use_no_gpu_from_the_next_batch`]）之后、再交来一组换上之前调它（核心层这时生效的选 `cpu`，
-    /// 不问它、不交活）。
+    /// 参与的卡一张都不剩，却既没有开不成、量代价出错，也没有因显存上限不用的卡：核心层校验过的一组至少点一张卡，换上的那一组点的卡都开着，
+    /// 换上之后没参与的只有后两种原因；参与的卡为空而这三样都没有的只有一种——告知一张卡都不参与（[`Self::use_no_gpu_from_the_next_batch`]）之后、
+    /// 再交来的一组换上之前。这期间核心层生效的是 `cpu`：交来的一组还在开卡时两批之间那一问在走到这里之前答还在开卡，答开不成时这一组不生效、
+    /// 核心层生效的照旧是 `cpu`（`singlefs_core::background_compute::BackgroundComputeOfThisMount` 只在实现答换上了那一刻换上 `gpu` 那一组），
+    /// 所以核心层不为生效的那一组问它、不走 GPU 那一路交活。
     #[must_use]
     pub fn batch_not_split_over_the_gpus(&self) -> Option<BatchNotSplitOverTheGpus> {
         if let Some(not_opened) = &self.not_opened_after_the_reopenings_at_the_last_switch {
@@ -3111,9 +3402,9 @@
                             reason: _,
                         }) => {
                             recuts_after_running_out_of_memory += 1;
-                            room = room.with_the_memory_limit_at_most(
-                                singlefs_replay_prefix::gpu_gc_capacity_proofs::gpu_memory_cap_after_allocation_failure(bytes_held_by_the_chunk),
-                            );
+                            room = room.with_the_memory_limit_at_most(GpuMemoryInBytes(
+                                singlefs_replay_prefix::gpu_gc_capacity_proofs::gpu_memory_cap_after_allocation_failure(bytes_held_by_the_chunk.0),
+                            ));
                             // 重切不留已做完的片：这张卡分到的那几条照新切法从头整段重交。
                             match cut(planned.entries.clone(), room) {
                                 Ok(recut) => plan = recut,
@@ -3217,9 +3508,9 @@
         &mut self,
         units: &[UnitChecksumToVerify<'_, EntryKey>],
     ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, ChecksumBatchFailed> {
-        let unit_lengths: Vec<u64> = units
+        let unit_lengths: Vec<InputLengthOnTheHostInBytes> = units
             .iter()
-            .map(|unit| u64::try_from(unit.unit_bytes.len()).expect("装得进 u64"))
+            .map(|unit| InputLengthOnTheHostInBytes::of_this_many_bytes(unit.unit_bytes.len()))
             .collect();
         self.outcomes_of_a_batch_over_the_cards(
             GpuComputedSegment::UnitChecksumVerification,
@@ -3377,7 +3668,13 @@
         self.outcomes_of_a_batch_over_the_cards(
             GpuComputedSegment::LiveSlotsPerClusterSegment,
             segments.len(),
-            |entries, room| live_slot_plan_on_a_gpu(allocation_records.len(), entries.len(), room),
+            |entries, room| {
+                live_slot_plan_on_a_gpu(
+                    AllocationRecordCountOfATable::of_this_many_records(allocation_records.len()),
+                    ClusterSegmentCountOfAGpu::of_this_many_segments(entries.len()),
+                    room,
+                )
+            },
             |plan: &Vec<RecordSliceWithItsChunks>| submissions_of_a_live_slot_plan(plan),
             |card, entries, plan, submissions_of_the_card| {
                 live_slots_summed_over_the_record_slices(
@@ -3795,7 +4092,8 @@
     /// 交了几次），账与产品那张卡同一份（`GpuBudgetUnderTheSettings`）。按脚本造三样：交活之前、重交之前等前一次超时的那次提交等满上限没等到
     /// （`earlier_submission_waits_timing_out`，按次序，`true` 是没等到；卡上记着一次还在跑的提交时才等、才取脚本），每一次提交等超过上限
     /// （`submissions_timing_out`，按次序，`true` 是超时，超时的那一段的账照产品记成还在跑的那次提交），卡上空闲的显存（`free_bytes`：一段合占的比它多就建不成缓冲）。
-    /// 脚本用完之后都等到、都做完。另记量了几次代价（`cost_measurements`；`cost_measurement_failing_with` 给了时每次量都交回那个错、不记代价），可点名第几次交成的校验和那一段交回翻过的判定
+    /// 脚本用完之后都等到、都做完。另记量了几次代价（`cost_measurements`；`cost_measurement_failing_with` 给了时每次量都交回那个错、不记代价；
+    /// `cost_measured_through_the_checksum_chunk` 时照产品那样把量代价那一段照此刻剩下的界切片、逐段当一次提交走账：先记账、超时就记成还在跑），可点名第几次交成的校验和那一段交回翻过的判定
     /// （`checksum_submission_with_flipped_verdicts`，从 1 数：那一次的结果是坏的，用了它判定就与 CPU 参照不同）。
     /// 析构一次就在 `drops_counted` 上加一（泄漏的卡不析构，计数不动）：用例在卡交出去之前取一份它的克隆。
     struct CardComputedOnTheCpuForTests {
@@ -3810,6 +4108,7 @@
         checksum_submission_with_flipped_verdicts: Option<u32>,
         cost_measurement_failing_with: Option<GpuFailureCause>,
         drops_counted: Arc<std::sync::atomic::AtomicU32>,
+        cost_measured_through_the_checksum_chunk: bool,
     }
 
     impl Drop for CardComputedOnTheCpuForTests {
@@ -3835,11 +4134,11 @@
                     total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
                 },
                 budget: GpuBudgetUnderTheSettings::with_the_bytes_leaked_by_earlier_mounts(
-                    1 << 30,
-                    1 << 20,
+                    GpuMemoryInBytes(1 << 30),
+                    EntryCountOfAChunk(1 << 20),
                     memory_limit_in_bytes.map(GpuMemoryInBytes),
                     WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
-                    bytes_leaked,
+                    GpuMemoryInBytes(bytes_leaked),
                 ),
                 measured_cost: Some(HostCpuNanosecondsPerSubmission(1_000)),
                 earlier_submission_waits_timing_out: VecDeque::new(),
@@ -3850,16 +4149,19 @@
                 checksum_submission_with_flipped_verdicts: None,
                 cost_measurement_failing_with: None,
                 drops_counted: Arc::new(std::sync::atomic::AtomicU32::new(0)),
+                cost_measured_through_the_checksum_chunk: false,
             }
         }
 
         /// 卡上记着一次等超过上限、还在跑的提交（占 `bytes` 字节），交活之前要先等它。
         fn with_a_submission_running_past_its_wait(mut self, bytes: u64) -> Self {
-            self.budget.memory_account_mut().hold(bytes);
+            self.budget
+                .memory_account_mut()
+                .hold(GpuMemoryInBytes(bytes));
             self.budget
                 .memory_account_mut()
                 .settle_the_chunk_after_its_submission(
-                    bytes,
+                    GpuMemoryInBytes(bytes),
                     SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(0),
                 );
             self
@@ -3869,7 +4171,7 @@
         /// 做完了放账、交回 `outcome`。
         fn chunk_submitted_on_the_cpu<Outcome>(
             &mut self,
-            bytes_held_by_the_chunk: u64,
+            bytes_held_by_the_chunk: GpuMemoryInBytes,
             outcome: impl FnOnce() -> Outcome,
         ) -> Result<Outcome, GpuFailureCause> {
             self.budget
@@ -3877,7 +4179,7 @@
                 .hold(bytes_held_by_the_chunk);
             if self
                 .free_bytes
-                .is_some_and(|free_bytes| bytes_held_by_the_chunk > free_bytes)
+                .is_some_and(|free_bytes| bytes_held_by_the_chunk.0 > free_bytes)
             {
                 self.budget
                     .memory_account_mut()
@@ -3944,6 +4246,29 @@
             if let Some(cause) = &self.cost_measurement_failing_with {
                 return Err(cause.clone());
             }
+            if self.cost_measured_through_the_checksum_chunk {
+                let chunks_of_pieces = COST_PROBE_CHUNK
+                    .pieces_under_the_bounds(self.budget.room_left_now())
+                    .map_err(
+                        |smallest_piece| GpuFailureCause::DeviceOrPipelineDidNotStart {
+                            reason: format!(
+                                "替身：量代价那一段最小的一片 {} 字节装不下",
+                                smallest_piece.smallest_piece_bytes.0
+                            ),
+                        },
+                    )?;
+                for pieces in &chunks_of_pieces {
+                    let input_bytes: InputLengthOnTheHostInBytes = pieces
+                        .iter()
+                        .map(crate::memory_limit_and_chunks::PieceOfAnEntry::input_byte_count)
+                        .sum();
+                    let bytes_held_by_the_chunk = CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
+                        input_bytes,
+                        EntryCountOfAChunk::of_this_many_entries(pieces.len()),
+                    );
+                    self.chunk_submitted_on_the_cpu(bytes_held_by_the_chunk, || ())?;
+                }
+            }
             self.measured_cost = Some(HostCpuNanosecondsPerSubmission(1_000));
             Ok(())
         }
@@ -3985,8 +4310,8 @@
         ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, GpuFailureCause> {
             let input_bytes: usize = units.iter().map(|unit| unit.unit_bytes.len()).sum();
             let bytes_held_by_the_chunk = CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
-                u64::try_from(input_bytes).expect("装得进 u64"),
-                u64::try_from(units.len()).expect("装得进 u64"),
+                InputLengthOnTheHostInBytes::of_this_many_bytes(input_bytes),
+                EntryCountOfAChunk::of_this_many_entries(units.len()),
             );
             let submission_with_flipped_verdicts = self.checksum_submission_with_flipped_verdicts;
             let submission_number = self.submissions_made + 1;
@@ -4021,8 +4346,8 @@
             let bytes_held_by_the_chunk =
                 region_record_chunk_footprint(mapping_tables.unwrap_or(&empty_mapping_tables))
                     .bytes_held_by_a_chunk(
-                        u64::try_from(input_bytes).expect("装得进 u64"),
-                        u64::try_from(pieces.len()).expect("装得进 u64"),
+                        InputLengthOnTheHostInBytes::of_this_many_bytes(input_bytes),
+                        EntryCountOfAChunk::of_this_many_entries(pieces.len()),
                     );
             self.chunk_submitted_on_the_cpu(bytes_held_by_the_chunk, || match mapping_tables {
                 None => BothKernelOutcomesOfARegionRecordChunk::RegistersAfterThePieces(vec![
@@ -4042,9 +4367,12 @@
             record_tables: &AllocationRecordTablesForTheKernels,
             _unit_area: UnitAreaOfOneDevice,
         ) -> Result<Vec<(u64, u64)>, GpuFailureCause> {
-            let segment_count = u64::try_from(segments.len()).expect("装得进 u64");
+            let segment_count = EntryCountOfAChunk::of_this_many_entries(segments.len());
             let bytes_held_by_the_chunk = live_slot_chunk_footprint(record_tables)
-                .bytes_held_by_a_chunk(segment_count * SEGMENT_BOUND_BYTES, segment_count);
+                .bytes_held_by_a_chunk(
+                    segment_count.input_bytes_of_the_entries(SEGMENT_BOUND_BYTES),
+                    segment_count,
+                );
             self.chunk_submitted_on_the_cpu(bytes_held_by_the_chunk, || {
                 vec![(0, 0); segments.len()]
             })
@@ -4401,8 +4729,8 @@
     ) -> GpuShortOfFreeMemoryForTheSmallestPiece {
         GpuShortOfFreeMemoryForTheSmallestPiece {
             device: card_identifier(pci_bus_address),
-            bytes_of_the_last_chunk_not_created: 128,
-            smallest_piece_bytes: 112,
+            bytes_of_the_last_chunk_not_created: GpuMemoryInBytes(128),
+            smallest_piece_bytes: GpuMemoryInBytes(112),
         }
     }
 
@@ -4595,8 +4923,8 @@
             gpus_leaked.gpus_leaked(),
             Some(vec![GpuLeakedWhenDropped {
                 device: card_identifier("0000:01:00.0"),
-                bytes_held_on_the_device: 600,
-                bytes_leaked_by_earlier_mounts: 0,
+                bytes_held_on_the_device: GpuMemoryInBytes(600),
+                bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
             }]),
             "只泄漏还在跑的那一张"
         );
@@ -4605,7 +4933,7 @@
                 memory_leaked.bytes_leaked_on(&card_identifier("0000:01:00.0")),
                 memory_leaked.bytes_leaked_on(&card_identifier("0000:41:00.0"))
             ),
-            (600, 0)
+            (GpuMemoryInBytes(600), GpuMemoryInBytes(0))
         );
     }
 
@@ -4621,8 +4949,8 @@
         let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
         memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
             device: card_identifier("0000:01:00.0"),
-            bytes_held_on_the_device: 100,
-            bytes_leaked_by_earlier_mounts: 0,
+            bytes_held_on_the_device: GpuMemoryInBytes(100),
+            bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
         }]);
         memory_leaked
     }
@@ -4653,7 +4981,7 @@
         );
         assert_eq!(
             memory_leaked.bytes_leaked_on(&card_identifier("0000:01:00.0")),
-            100 + 600,
+            GpuMemoryInBytes(100 + 600),
             "泄漏账加的恰好是这一次占着的 600 字节"
         );
     }
@@ -4677,7 +5005,7 @@
         );
         assert_eq!(
             memory_leaked.bytes_leaked_on(&card_identifier("0000:01:00.0")),
-            100,
+            GpuMemoryInBytes(100),
             "泄漏账不变"
         );
     }
@@ -4950,7 +5278,7 @@
             .collect()
     }
 
-    /// 欠账 C996 的定案「切换到新卡时再开」：挂载点「0000:01:00.0」、切换点它与「0000:41:00.0」——核心层照卡清单收下这一次切换、校验那一刻不开卡；
+    /// 第 2 条「挂着时切换点到还没开的卡，那一刻再开那张卡」：挂载点「0000:01:00.0」、切换点它与「0000:41:00.0」——核心层照卡清单收下这一次切换、校验那一刻不开卡；
     /// 交设置之后开「0000:41:00.0」（只开这一张、只开一次），开完了在两批之间换上，两张都参与、按点名的次序；再交同一组不重开。
     #[test]
     fn the_switch_naming_a_card_not_opened_at_the_mount_opens_that_card_when_the_switch_takes_effect(
@@ -5012,7 +5340,7 @@
         assert_eq!(openings_so_far().len(), 1, "开着的卡不重开");
     }
 
-    /// 用户定案「切换请求照收，在会话锁外开新卡……这期间照旧用切换前那组卡。开成了，下一批起换成新设置」：切换只点「0000:41:00.0」，
+    /// 第 4 条「切换请求照收，在会话锁外开新卡；重开期间照旧用切换前那组卡，开成了，下一批起换成新设置」：切换只点「0000:41:00.0」，
     /// 开它那一步等用例放开才交回——交设置照常返回、不等开卡（核心层在会话锁里交设置），两批之间不等它的那一问交回 `None`，
     /// 其间那一批照切换之前那一组只分给「0000:01:00.0」；放开之后换上，下一批只分给「0000:41:00.0」，不再点名的那张关掉。
     #[test]
@@ -5089,7 +5417,7 @@
         );
     }
 
-    /// 用户定案「开不成就间隔 1 分钟重开，最多重开 3 次」：切换点到的「0000:41:00.0」头两次开不成、第三次开成——照常换上、两张都参与，
+    /// 第 4 条「间隔 1 分钟重开，最多重开 3 次」：切换点到的「0000:41:00.0」头两次开不成、第三次开成——照常换上、两张都参与，
     /// 开卡那一步一共只开它 3 次。
     #[test]
     fn the_card_not_opened_twice_at_the_switch_opens_the_third_time_and_the_switch_takes_effect() {
@@ -5142,7 +5470,7 @@
         assert_eq!(background_gpu.batch_not_split_over_the_gpus(), None);
     }
 
-    /// 用户定案「3 次都不成：GC 停下，报警并报出哪张卡、每次开不成的原因，等人处理，不自动再试」：切换只点「0000:41:00.0」，它头一次与 3 次重开都开不成——
+    /// 第 4 条「重开 3 次都不成，GC 停下，报警里报出哪张卡、每次开不成的原因，等人处理，不自动再试」：切换只点「0000:41:00.0」，它头一次与 3 次重开都开不成——
     /// 两批之间那一问答开不成、带这张卡与 4 次的原因（再问照答，之后不再开），之后那一批报成这张卡的 GPU 出错（带每一次的原因）、一条活都不分出去；
     /// 照旧只开着切换之前那一张。下一次换设置（切回「0000:01:00.0」）解除它，那一批照常只分给它。
     #[test]
@@ -5229,7 +5557,7 @@
         );
     }
 
-    /// 用户定案「切换之后不再参与的卡关不关」=「切换时关掉」：挂载开着「0000:01:00.0」「0000:41:00.0」，切换只点「0000:01:00.0」——
+    /// 第 2 条「切换之后不再参与的卡，切换生效时关掉」：挂载开着「0000:01:00.0」「0000:41:00.0」，切换只点「0000:01:00.0」——
     /// 换上那一刻「0000:41:00.0」关掉（替身卡析构一次、不再开着），卡清单照列它；再切到两张都点，它重开（开卡那一步走一次），两张都参与。
     #[test]
     fn the_card_no_longer_named_by_a_switch_is_closed_and_reopened_when_named_again() {
@@ -5311,7 +5639,7 @@
         );
         assert_eq!(
             memory_leaked.bytes_leaked_on(&card_identifier("0000:41:00.0")),
-            600,
+            GpuMemoryInBytes(600),
             "泄漏账加的恰好是这一次占着的 600 字节"
         );
         assert_eq!(
@@ -5339,7 +5667,7 @@
                 let mut card = CardComputedOnTheCpuForTests::of_the_address(
                     &discrete_gpu.pci_bus_address,
                     memory_limit.map(|memory_limit| memory_limit.0),
-                    bytes_leaked_by_earlier_mounts,
+                    bytes_leaked_by_earlier_mounts.0,
                 )
                 .with_a_submission_running_past_its_wait(600);
                 card.measured_cost = None;
@@ -5349,6 +5677,7 @@
                 move || Ok(card)
             },
             memory_leaked.clone(),
+            OpeningsNotReturnedInThisProcess::with_none_not_returned(),
         );
         let (mut background_gpu, _cards_handed_to_the_opening) =
             background_gpu_with_one_card_not_opened_at_the_mount(
@@ -5375,7 +5704,7 @@
         );
         assert_eq!(
             memory_leaked.bytes_leaked_on(&card_identifier("0000:41:00.0")),
-            4 * 600,
+            GpuMemoryInBytes(4 * 600),
             "泄漏账每一次加的恰好是那一次占着的 600 字节"
         );
         assert_eq!(
@@ -5425,7 +5754,7 @@
                     memory_limit: GpuMemoryInBytes(memory_limit_of_the_card_at_one.unwrap_or(0)),
                     smallest_piece_bytes: bytes_of_the_smallest_piece_every_participating_gpu_holds(
                     ),
-                    bytes_already_held_on_the_device: 0,
+                    bytes_already_held_on_the_device: GpuMemoryInBytes(0),
                 }],
             )
         } else {
@@ -5652,11 +5981,11 @@
         );
     }
 
-    /// 开卡的结局按交设置的次序认，不按内容比：切换只点「0000:41:00.0」，头一次与 3 次重开都开不成；开卡线程开完 4 次之后、两批之间那一问还没问到时，
-    /// 同一组设置又交了一次——那一问不报上一次的 4 个原因，照最近那一次重开（一共开 8 次），再报这一次的 4 个原因。
+    /// 第 4 条「一张卡至多开 4 次」在一次挂载里连着数（开成一次清零）：切换只点「0000:41:00.0」，头一次与 3 次重开都开不成；开卡线程开完 4 次之后、两批之间那一问还没问到时，
+    /// 同一组设置又交了一次——不再开它（开卡那一步一共只被调 4 次），那一问答开不成，原因只一个：这一次挂载里连着试满了 4 次。
     #[test]
-    fn the_same_settings_handed_in_again_before_the_poll_reopen_the_card_instead_of_taking_the_earlier_failure(
-    ) {
+    fn the_same_settings_handed_in_again_once_the_openings_of_the_card_are_used_up_open_it_no_more()
+    {
         let (opened_four_times, every_opening_made) = std::sync::mpsc::channel::<()>();
         let openings_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
         let openings_counted = Arc::clone(&openings_made);
@@ -5686,16 +6015,17 @@
         );
         assert_eq!(
             openings_made_so_far(&cards_handed_to_the_opening),
-            8,
-            "又交的那一次照它重开：一共开 8 次"
+            4,
+            "连着试满了：又交的那一次不再开"
         );
-        assert!(
-            matches!(
-                &answered,
-                LatestSettingsHandedIn::NotOpened(not_opened)
-                    if not_opened.causes_of_each_opening.len() == 4
-            ),
-            "报的是又交那一次的 4 个原因：{answered:?}"
+        assert_eq!(
+            answered,
+            LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
+                device: card_identifier("0000:41:00.0"),
+                causes_of_each_opening: vec![GpuFailureCause::OpeningsInThisMountUsedUp {
+                    failed_openings: 4
+                }],
+            })
         );
     }
 
@@ -5810,7 +6140,7 @@
         );
     }
 
-    /// 用户定案「切到 cpu 时开着的卡全部关掉」：挂载开着「0000:01:00.0」「0000:41:00.0」，告知一张卡都不参与——开着的卡为零（「0000:41:00.0」那张替身卡析构一次），
+    /// 第 2 条「从 gpu 切到 cpu……关掉全部开着的卡」：挂载开着「0000:01:00.0」「0000:41:00.0」，告知一张卡都不参与——开着的卡为零（「0000:41:00.0」那张替身卡析构一次），
     /// 卡清单照列两张；再交来只点「0000:41:00.0」的一组，它当没开的卡开（开卡那一步走一次），开完换上。
     #[test]
     fn no_gpu_taking_part_closes_every_open_card_and_the_next_settings_open_their_cards_again() {
@@ -5877,7 +6207,7 @@
         }
     }
 
-    /// 用户定案「切到 cpu 时……丢掉在等的那一组与还没取走的开卡结局」：切换点没开的「0000:41:00.0」，头一次开卡在开（等用例放开）时告知一张卡都不参与——
+    /// 第 2 条「从 gpu 切到 cpu……丢掉还在开的那一组」：切换点没开的「0000:41:00.0」，头一次开卡在开（等用例放开）时告知一张卡都不参与——
     /// 开卡线程开完这一次就不再开（重开规矩本来还许 3 次），开卡那一步一共只被调一次；开着的卡为零。
     #[test]
     fn no_gpu_taking_part_while_a_card_opens_calls_off_the_reopenings() {
@@ -6070,8 +6400,8 @@
             LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
                 device: card_identifier("0000:41:00.0"),
                 causes_of_each_opening: vec![
-                    GpuFailureCause::DeviceOrPipelineDidNotStart {
-                        reason: "开卡 panic 了：替身：开卡次次 panic".to_string(),
+                    GpuFailureCause::OpeningPanicked {
+                        said_by_the_panic: "替身：开卡次次 panic".to_string(),
                     };
                     4
                 ],
@@ -6099,7 +6429,7 @@
                 let card = CardComputedOnTheCpuForTests::of_the_address(
                     &discrete_gpu.pci_bus_address,
                     memory_limit.map(|memory_limit| memory_limit.0),
-                    bytes_leaked_by_earlier_mounts,
+                    bytes_leaked_by_earlier_mounts.0,
                 );
                 let start_released_seen_by_the_start = Arc::clone(&start_released);
                 let starts_counted_by_the_start = Arc::clone(&starts_counted);
@@ -6113,6 +6443,7 @@
                 }
             },
             GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            OpeningsNotReturnedInThisProcess::with_none_not_returned(),
         );
         let (mut background_gpu, _cards_handed_to_the_opening) =
             background_gpu_with_the_card_at_one_opened_and_two_not_opened(
@@ -6169,8 +6500,8 @@
             leaks_of_the_switches.gpus_leaked(),
             vec![GpuLeakedWhenDropped {
                 device: card_identifier("0000:41:00.0"),
-                bytes_held_on_the_device: 600,
-                bytes_leaked_by_earlier_mounts: 0,
+                bytes_held_on_the_device: GpuMemoryInBytes(600),
+                bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
             }]
         );
     }
@@ -6248,7 +6579,7 @@
                 vec![CardComputedOnTheCpuForTests::of_the_address(
                     "0000:01:00.0",
                     None,
-                    memory_leaked.bytes_leaked_on(&card),
+                    memory_leaked.bytes_leaked_on(&card).0,
                 )
                 .with_a_submission_running_past_its_wait(600)],
                 &memory_leaked,
@@ -6258,7 +6589,7 @@
                 vec![CardComputedOnTheCpuForTests::of_the_address(
                     "0000:01:00.0",
                     None,
-                    memory_leaked.bytes_leaked_on(&card),
+                    memory_leaked.bytes_leaked_on(&card).0,
                 )
                 .with_a_submission_running_past_its_wait(300)],
                 &memory_leaked,
@@ -6267,19 +6598,19 @@
         drop(mount_leaking_three_hundred_bytes);
         assert_eq!(
             memory_leaked.bytes_leaked_on(&card),
-            900,
+            GpuMemoryInBytes(900),
             "两次各泄漏 600、300 字节：记 900，不让后丢的盖掉先丢的"
         );
         assert_eq!(
             CardComputedOnTheCpuForTests::of_the_address(
                 "0000:01:00.0",
                 Some(1 << 20),
-                memory_leaked.bytes_leaked_on(&card),
+                memory_leaked.bytes_leaked_on(&card).0,
             )
             .budget
             .memory_account()
             .bytes_held(),
-            900,
+            GpuMemoryInBytes(900),
             "新一次挂载这张卡的账从 900 起"
         );
     }
@@ -6298,12 +6629,12 @@
             &memory_leaked,
         );
         drop(earlier_mount);
-        assert_eq!(memory_leaked.bytes_leaked_on(&card), 600);
+        assert_eq!(memory_leaked.bytes_leaked_on(&card), GpuMemoryInBytes(600));
         let (later_mount, _later_present) = background_gpu_over_cards_on_the_cpu(
             vec![CardComputedOnTheCpuForTests::of_the_address(
                 "0000:01:00.0",
                 None,
-                memory_leaked.bytes_leaked_on(&card),
+                memory_leaked.bytes_leaked_on(&card).0,
             )
             .with_a_submission_running_past_its_wait(300)],
             &memory_leaked,
@@ -6314,14 +6645,14 @@
             gpus_leaked_by_the_later_mount.gpus_leaked(),
             Some(vec![GpuLeakedWhenDropped {
                 device: card.clone(),
-                bytes_held_on_the_device: 900,
-                bytes_leaked_by_earlier_mounts: 600,
+                bytes_held_on_the_device: GpuMemoryInBytes(900),
+                bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(600),
             }]),
             "后一次挂载丢掉那一刻占着 900，其中 600 是开卡时已算上的"
         );
         assert_eq!(
             memory_leaked.bytes_leaked_on(&card),
-            900,
+            GpuMemoryInBytes(900),
             "进程里只加这一次自己的 300"
         );
     }
@@ -6332,7 +6663,13 @@
     /// 校验和那一段一次交得下 `unit_count` 个 [`BYTES_OF_A_UNIT_IN_THE_FILL`] 字节的单元、多一个就交不下的显存上限（`CHECKSUM_CHUNK_FOOTPRINT` 算的）。
     fn memory_limit_holding_units_in_one_submission(unit_count: u64) -> u64 {
         let bytes_of_a_unit = u64::try_from(BYTES_OF_A_UNIT_IN_THE_FILL).expect("小数");
-        CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(bytes_of_a_unit * unit_count, unit_count)
+        CHECKSUM_CHUNK_FOOTPRINT
+            .bytes_held_by_a_chunk(
+                EntryCountOfAChunk(unit_count)
+                    .input_bytes_of_the_entries(InputLengthOnTheHostInBytes(bytes_of_a_unit)),
+                EntryCountOfAChunk(unit_count),
+            )
+            .0
     }
 
     /// 本机 `card_count` 张替身卡的总线地址，按本机的次序：「0000:01:00.0」「0000:41:00.0」「0000:81:00.0」……
@@ -6522,7 +6859,7 @@
                 .collect(),
             &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
         );
-        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
+        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
         let leaving_out_the_card_in_front = settings_admitted_on_the_cards(
             &background_gpu,
             None,
@@ -6700,12 +7037,15 @@
             }
         }
 
-        fn readback_of_one_kernel(&self, outcome_bytes: u64) -> BufferInMemoryForTests {
-            self.buffer_of(outcome_bytes)
+        fn readback_of_one_kernel(
+            &self,
+            outcome_bytes: GpuMemoryInBytes,
+        ) -> BufferInMemoryForTests {
+            self.buffer_of(outcome_bytes.0)
         }
 
-        fn bytes_of_the_buffer(&self, buffer: &BufferInMemoryForTests) -> u64 {
-            buffer.bytes_requested
+        fn bytes_of_the_buffer(&self, buffer: &BufferInMemoryForTests) -> GpuMemoryInBytes {
+            GpuMemoryInBytes(buffer.bytes_requested)
         }
 
         fn out_of_memory_captured_by(
@@ -6727,9 +7067,10 @@
     const BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK: u64 = 1000;
     const OUTCOME_BYTES_OF_THE_CHUNK: u64 = 64;
     /// 这一段记在账上的字节：自己那几样映射着建、连同暂存，加两份回读暂存。
-    const BYTES_HELD_BY_THE_CHUNK_ON_THE_ACCOUNT: u64 =
-        COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION * BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK
-            + 2 * OUTCOME_BYTES_OF_THE_CHUNK;
+    const BYTES_HELD_BY_THE_CHUNK_ON_THE_ACCOUNT: GpuMemoryInBytes =
+        COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
+            .bytes_of_the_copies(GpuMemoryInBytes(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK))
+            .plus(GpuMemoryInBytes(2 * OUTCOME_BYTES_OF_THE_CHUNK));
 
     /// 卡上空闲的装得下这一段自己那几样缓冲，
     /// 装不下两份回读暂存——回读暂存建在同一个只接显存不足的范围里，那一条显存不足被范围接住：交回 `OutOfMemoryCreatingTheBuffers`
@@ -6743,7 +7084,7 @@
             &device,
             Duration::from_secs(1),
             BYTES_HELD_BY_THE_CHUNK_ON_THE_ACCOUNT,
-            OUTCOME_BYTES_OF_THE_CHUNK,
+            GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
             || [device.buffer_of(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK)],
         );
         assert!(
@@ -6775,7 +7116,7 @@
             &device,
             Duration::from_secs(1),
             BYTES_HELD_BY_THE_CHUNK_ON_THE_ACCOUNT,
-            OUTCOME_BYTES_OF_THE_CHUNK,
+            GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
             || [device.buffer_of(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK)],
         )
         .expect("空闲的正好装得下：建得成");
@@ -6827,7 +7168,7 @@
             &device,
             Duration::from_secs(1),
             bytes_held_by_the_chunk,
-            OUTCOME_BYTES_OF_THE_CHUNK,
+            GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
             || {
                 [
                     device.buffer_of(BYTES_OF_THE_OTHER_BUFFERS_OF_THE_CHUNK),
@@ -6837,10 +7178,10 @@
         );
         // 走到这里就是账没核真建的缓冲：卡上真占的峰值比账上的多，用例照「峰值不超账」红在这里。
         assert!(
-            device.peak_bytes_held.get() <= account.bytes_held(),
+            device.peak_bytes_held.get() <= account.bytes_held().0,
             "卡上真占的峰值 {} 字节超过了账上的 {} 字节（{:?}）",
             device.peak_bytes_held.get(),
-            account.bytes_held(),
+            account.bytes_held().0,
             created.as_ref().map(|_buffers| "建成了")
         );
     }
@@ -6965,8 +7306,10 @@
         let contents_written = Cell::new(false);
         let bytes_held_by_the_chunk_on_the_account =
             COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
-                * BYTES_OF_THE_BUFFER_BEYOND_THE_CAPPED_ADDRESS_SPACE
-                + 2 * OUTCOME_BYTES_OF_THE_CHUNK;
+                .bytes_of_the_copies(GpuMemoryInBytes(
+                    BYTES_OF_THE_BUFFER_BEYOND_THE_CAPPED_ADDRESS_SPACE,
+                ))
+                .plus(GpuMemoryInBytes(2 * OUTCOME_BYTES_OF_THE_CHUNK));
         let created = with_the_address_space_capped_beyond_the_present_size(
             ROOM_LEFT_IN_THE_CAPPED_ADDRESS_SPACE,
             || {
@@ -6974,11 +7317,11 @@
                     &device,
                     Duration::from_secs(10),
                     bytes_held_by_the_chunk_on_the_account,
-                    OUTCOME_BYTES_OF_THE_CHUNK,
+                    GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
                     || {
                         [buffer_mapped_at_creation_within_the_scope_of_the_caller(
                             &device,
-                            BYTES_OF_THE_BUFFER_BEYOND_THE_CAPPED_ADDRESS_SPACE,
+                            GpuMemoryInBytes(BYTES_OF_THE_BUFFER_BEYOND_THE_CAPPED_ADDRESS_SPACE),
                             wgpu::BufferUsages::STORAGE,
                             |_mapped| contents_written.set(true),
                         )]
@@ -7053,8 +7396,10 @@
         let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
             &device,
             Duration::from_secs(10),
-            COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION * 8 + 2 * OUTCOME_BYTES_OF_THE_CHUNK,
-            OUTCOME_BYTES_OF_THE_CHUNK,
+            COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
+                .bytes_of_the_copies(GpuMemoryInBytes(8))
+                .plus(GpuMemoryInBytes(2 * OUTCOME_BYTES_OF_THE_CHUNK)),
+            GpuMemoryInBytes(OUTCOME_BYTES_OF_THE_CHUNK),
             || {
                 [buffer_with_contents_created_within_the_scope_of_the_caller(
                     &device,
@@ -7111,6 +7456,7 @@
             let refused = open_background_gpu_with_the_memory_leaked_in_this_process(
                 &selection,
                 &memory_leaked,
+                &OpeningsNotReturnedInThisProcess::with_none_not_returned(),
                 &vulkan_instance,
                 |_gpus_to_open| {},
             )
@@ -7279,4 +7625,1394 @@
             );
         }
     }
+
+    // ── 开不成之后生效的照旧是切换之前那一组、挂载那一刻开卡 panic 照重开、一张卡一次挂载里连着试满、还没回来的开卡管到整个进程、
+    // 读得到切换关卡的泄漏、被盖掉的切换（D26（后台整理与放置回收） 已定项 13 第 2、4 条）──
+
+    /// 切换开卡那一步：「0000:01:00.0」等用例放开（至多 20 秒）再开成，别的卡每一次都开不成。
+    fn opening_the_card_at_one_once_released_and_no_other(
+        opening_released: std::sync::mpsc::Receiver<()>,
+    ) -> impl FnMut(
+        &DiscreteGpuOnThisHost,
+        Option<GpuMemoryInBytes>,
+        Duration,
+        CostAtTheOpening,
+    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
+           + Send
+           + 'static {
+        move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
+            if discrete_gpu.pci_bus_address != "0000:01:00.0" {
+                return Err(reason_the_card_does_not_open_at_the_switch());
+            }
+            opening_released
+                .recv_timeout(Duration::from_secs(20))
+                .map_err(|_not_released| reason_the_card_does_not_open_at_the_switch())?;
+            opening_that_always_opens(discrete_gpu, memory_limit, wait_limit, cost_at_the_opening)
+        }
+    }
+
+    /// 一段历史在切换点的卡开不成之后的读数：开不成之后开着的卡、参与的卡，与再交同一组那一刻的答。
+    struct ReadingsOfTheSwitchNotOpenedAndHandedInAgain {
+        answered_once_not_opened: LatestSettingsHandedIn,
+        gpus_opened_once_not_opened: Vec<GpuDeviceIdentifier>,
+        participating_once_not_opened: Vec<GpuDeviceIdentifier>,
+        answered_when_handed_in_again: LatestSettingsHandedIn,
+        openings: usize,
+    }
+
+    /// 挂载开着「0000:01:00.0」，核心层交来点它的那一组；`switch_to_the_cpu_first` 时先告知一张卡都不参与；切换只点没开的「0000:41:00.0」，
+    /// 头一次与 3 次重开都开不成；再交同一组。
+    fn readings_of_the_switch_not_opened_and_handed_in_again(
+        switch_to_the_cpu_first: bool,
+    ) -> ReadingsOfTheSwitchNotOpenedAndHandedInAgain {
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
+                    Err(reason_the_card_does_not_open_at_the_switch())
+                },
+                None,
+                false,
+            );
+        let at_the_mount =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:01:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&at_the_mount);
+        if switch_to_the_cpu_first {
+            background_gpu.use_no_gpu_from_the_next_batch();
+        }
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        let answered_once_not_opened = background_gpu
+            .put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds,
+            );
+        let gpus_opened_once_not_opened = background_gpu.gpus_opened();
+        let participating_once_not_opened = participating_devices_of(&background_gpu);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        let answered_when_handed_in_again = background_gpu
+            .put_the_gpus_opened_since_the_switch_in_effect(WaitForTheOpeningAtASwitch::NotAtAll);
+        ReadingsOfTheSwitchNotOpenedAndHandedInAgain {
+            answered_once_not_opened,
+            gpus_opened_once_not_opened,
+            participating_once_not_opened,
+            answered_when_handed_in_again,
+            openings: openings_made_so_far(&cards_handed_to_the_opening),
+        }
+    }
+
+    fn not_opened_with_the_openings_of_the_card_at_forty_one_used_up() -> LatestSettingsHandedIn {
+        LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
+            device: card_identifier("0000:41:00.0"),
+            causes_of_each_opening: vec![GpuFailureCause::OpeningsInThisMountUsedUp {
+                failed_openings: 4,
+            }],
+        })
+    }
+
+    /// 先切到 `cpu` 的那一臂：开不成之后一张卡都没开、没参与（核心层生效的照旧是 `cpu`，不交活）；再交同一组，这张卡已连着试满，当场答开不成、不再开。
+    #[test]
+    fn a_switch_not_opened_after_the_cpu_leaves_no_card_open_and_the_same_settings_again_are_answered_not_opened_at_once(
+    ) {
+        let readings = readings_of_the_switch_not_opened_and_handed_in_again(true);
+        assert!(matches!(
+            readings.answered_once_not_opened,
+            LatestSettingsHandedIn::NotOpened(_)
+        ));
+        assert!(readings.gpus_opened_once_not_opened.is_empty());
+        assert!(readings.participating_once_not_opened.is_empty());
+        assert_eq!(
+            readings.answered_when_handed_in_again,
+            not_opened_with_the_openings_of_the_card_at_forty_one_used_up()
+        );
+        assert_eq!(readings.openings, 4);
+    }
+
+    /// 不先切到 `cpu` 的那一臂：开不成之后参与的照旧是切换之前那张「0000:01:00.0」，与核心层照旧生效的那一组是同一张；再交同一组当场答开不成、不再开。
+    #[test]
+    fn a_switch_not_opened_on_the_gpu_keeps_the_card_before_the_switch_taking_part() {
+        let readings = readings_of_the_switch_not_opened_and_handed_in_again(false);
+        assert!(matches!(
+            readings.answered_once_not_opened,
+            LatestSettingsHandedIn::NotOpened(_)
+        ));
+        assert_eq!(
+            readings.gpus_opened_once_not_opened,
+            vec![card_identifier("0000:01:00.0")]
+        );
+        assert_eq!(
+            readings.participating_once_not_opened,
+            vec![card_identifier("0000:01:00.0")],
+            "参与的照旧是切换之前那一组"
+        );
+        assert_eq!(
+            readings.answered_when_handed_in_again,
+            not_opened_with_the_openings_of_the_card_at_forty_one_used_up()
+        );
+        assert_eq!(readings.openings, 4);
+    }
+
+    /// 一张替身独显（没开）。
+    fn discrete_gpu_not_opened_at(pci_bus_address: &str) -> DiscreteGpuOnThisHost {
+        DiscreteGpuOnThisHost {
+            position_in_the_enumeration: 0,
+            identifier: card_identifier(pci_bus_address),
+            pci_bus_address: pci_bus_address.to_string(),
+            name: format!("cpu card at {pci_bus_address} not opened yet"),
+            total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
+        }
+    }
+
+    /// 产品那一份开一张卡那一步（`opening_of_a_card_within_the_limit`，一次至多等 20 秒），起卡那一件头一次 panic、之后开成替身卡；
+    /// 起卡那一件每被起一次往 `starts_made` 加一。
+    fn opening_panicking_at_its_first_start(
+        starts_made: &Arc<std::sync::atomic::AtomicU32>,
+    ) -> impl FnMut(
+        &DiscreteGpuOnThisHost,
+        Option<GpuMemoryInBytes>,
+        Duration,
+        CostAtTheOpening,
+    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
+           + Send
+           + 'static {
+        let starts_counted = Arc::clone(starts_made);
+        opening_of_a_card_within_the_limit(
+            Duration::from_secs(20),
+            move |discrete_gpu: &DiscreteGpuOnThisHost,
+                  memory_limit,
+                  _wait_limit,
+                  bytes_leaked_by_earlier_mounts| {
+                let card = CardComputedOnTheCpuForTests::of_the_address(
+                    &discrete_gpu.pci_bus_address,
+                    memory_limit.map(|memory_limit| memory_limit.0),
+                    bytes_leaked_by_earlier_mounts.0,
+                );
+                let starts_counted_by_the_start = Arc::clone(&starts_counted);
+                move || {
+                    let starts_before = starts_counted_by_the_start
+                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
+                    assert!(starts_before > 0, "替身：起卡那一件头一次 panic");
+                    Ok(card)
+                }
+            },
+            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            OpeningsNotReturnedInThisProcess::with_none_not_returned(),
+        )
+    }
+
+    /// 第 4 条「开卡线程 panic 都算这张卡这一次没开成，照重开」用于挂载那一刻：开一张卡那一步（产品那一份）头一次 panic——挂载照重开、第二次开成，
+    /// 不拒挂载，起卡那一件一共被起 2 次。
+    #[test]
+    fn an_opening_panicking_at_the_mount_is_reopened_and_the_mount_is_not_refused() {
+        let discrete_gpu = discrete_gpu_not_opened_at("0000:41:00.0");
+        let starts_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
+        let mut opening_at_the_mount = opening_panicking_at_its_first_start(&starts_made);
+        let mount = gpus_opened_at_the_mount(
+            vec![discrete_gpu.clone()],
+            &[ParticipatingGpu {
+                discrete_gpu,
+                memory_limit: None,
+            }],
+            bytes_of_the_smallest_piece_every_participating_gpu_holds(),
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+            |_interval: Duration| {},
+            |gpu_to_open, memory_limit_at_the_opening, cost_at_the_opening| {
+                opening_at_the_mount(
+                    gpu_to_open,
+                    memory_limit_at_the_opening,
+                    BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+                    cost_at_the_opening,
+                )
+            },
+        );
+        assert!(
+            mount.is_ok(),
+            "头一次 panic 照重开、第二次开成：{:?}",
+            mount.err()
+        );
+        assert_eq!(starts_made.load(std::sync::atomic::Ordering::SeqCst), 2);
+    }
+
+    /// 开卡那一步：「0000:41:00.0」第 3 次与第 6 次开成、别的次次开不成；别的卡每一次都开成。各次交来的卡记在交回的那一份里（另由外壳那一层记）。
+    fn opening_the_card_at_forty_one_only_the_third_and_sixth_times() -> impl FnMut(
+        &DiscreteGpuOnThisHost,
+        Option<GpuMemoryInBytes>,
+        Duration,
+        CostAtTheOpening,
+    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
+           + Send
+           + 'static {
+        let mut openings_of_the_card_at_forty_one: u32 = 0;
+        move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
+            if discrete_gpu.pci_bus_address == "0000:41:00.0" {
+                openings_of_the_card_at_forty_one += 1;
+                if openings_of_the_card_at_forty_one != 3 && openings_of_the_card_at_forty_one != 6
+                {
+                    return Err(reason_the_card_does_not_open_at_the_switch());
+                }
+            }
+            opening_that_always_opens(discrete_gpu, memory_limit, wait_limit, cost_at_the_opening)
+        }
+    }
+
+    /// 开卡那一步：「0000:41:00.0」次次开不成，对它的第 2 次先往 `second_opening_begun` 送一声、等用例放开（至多 20 秒）；别的卡每一次都开成。
+    fn opening_the_card_at_forty_one_never_and_holding_its_second_opening(
+        second_opening_begun: std::sync::mpsc::Sender<()>,
+        second_opening_released: std::sync::mpsc::Receiver<()>,
+    ) -> impl FnMut(
+        &DiscreteGpuOnThisHost,
+        Option<GpuMemoryInBytes>,
+        Duration,
+        CostAtTheOpening,
+    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
+           + Send
+           + 'static {
+        let mut openings_of_the_card_at_forty_one: u32 = 0;
+        move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
+            if discrete_gpu.pci_bus_address == "0000:41:00.0" {
+                openings_of_the_card_at_forty_one += 1;
+                if openings_of_the_card_at_forty_one == 2 {
+                    // 收的那一头在用例里一直拿着。
+                    let _signal_sent = second_opening_begun.send(());
+                    let _released_or_not =
+                        second_opening_released.recv_timeout(Duration::from_secs(20));
+                }
+                return Err(reason_the_card_does_not_open_at_the_switch());
+            }
+            opening_that_always_opens(discrete_gpu, memory_limit, wait_limit, cost_at_the_opening)
+        }
+    }
+
+    /// 开卡那一步对「0000:41:00.0」被调了几次。
+    fn openings_of_the_card_at_forty_one(
+        cards_handed_to_the_opening: &Arc<Mutex<Vec<String>>>,
+    ) -> usize {
+        cards_handed_to_the_opening
+            .lock()
+            .expect("用例没在别处 panic 过")
+            .iter()
+            .filter(|address| address.as_str() == "0000:41:00.0")
+            .count()
+    }
+
+    /// 第 4 条「一张卡至多开 4 次」数的是一次挂载里这张卡连着开不成的次数，开成一次就清零：「0000:41:00.0」头一次切换开不成 2 次、第 3 次开成；
+    /// 切走关掉之后再点它，又开不成 2 次、这一次的第 3 次（对它是第 6 次）开成——前后两段的开不成不并成 4 次，不当试满，换上。
+    #[test]
+    fn a_card_opened_between_its_failed_openings_counts_its_failed_openings_afresh() {
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                opening_the_card_at_forty_one_only_the_third_and_sixth_times(),
+                None,
+                false,
+            );
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        let back_to_the_card_at_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:01:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            },
+            "第 3 次开成"
+        );
+        background_gpu.use_the_settings_from_the_next_batch(&back_to_the_card_at_one);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            },
+            "切回「0000:01:00.0」：「0000:41:00.0」关掉"
+        );
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            },
+            "开成过一次就清零：又开不成 2 次不算试满，第 3 次开成"
+        );
+        assert_eq!(
+            participating_devices_of(&background_gpu),
+            vec![card_identifier("0000:41:00.0")]
+        );
+        assert_eq!(
+            openings_of_the_card_at_forty_one(&cards_handed_to_the_opening),
+            6
+        );
+    }
+
+    /// 连着开不成的次数按卡数、跨切换接着数：「0000:41:00.0」头一次切换开不成 2 次（第 2 次在开时告知一张卡都不参与，余下的叫停没开）；
+    /// 之后「0000:81:00.0」开成，不给「0000:41:00.0」清零；再点「0000:41:00.0」只剩 2 次可开、都开不成——答开不成、报这 2 次的原因；
+    /// 再点它不再开，答试满了 4 次，开卡那一步对它一共 4 次。
+    #[test]
+    fn the_failed_openings_of_a_card_in_a_row_add_up_across_the_switches_and_another_card_opening_does_not_clear_them(
+    ) {
+        let (second_opening_begun, every_second_opening_begun) = std::sync::mpsc::channel::<()>();
+        let (release_the_second_opening, second_opening_released) =
+            std::sync::mpsc::channel::<()>();
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                opening_the_card_at_forty_one_never_and_holding_its_second_opening(
+                    second_opening_begun,
+                    second_opening_released,
+                ),
+                None,
+                false,
+            );
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        let to_the_card_at_eighty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        every_second_opening_begun
+            .recv_timeout(Duration::from_secs(20))
+            .expect("开卡线程 20 秒内开始对「0000:41:00.0」的第 2 次开卡");
+        background_gpu.use_no_gpu_from_the_next_batch();
+        release_the_second_opening
+            .send(())
+            .expect("开卡那一步还在等放开：收的那一头还在");
+        drop(
+            background_gpu
+                .opening_of_a_gpu_at_a_switch
+                .lock()
+                .expect("开一次卡里的 panic 在那一步就接住了：锁没被毒化"),
+        );
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            },
+            "「0000:81:00.0」开成"
+        );
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
+                device: card_identifier("0000:41:00.0"),
+                causes_of_each_opening: vec![reason_the_card_does_not_open_at_the_switch(); 2],
+            }),
+            "连着开不成 2 次之后只剩 2 次可开"
+        );
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            not_opened_with_the_openings_of_the_card_at_forty_one_used_up()
+        );
+        assert_eq!(
+            openings_of_the_card_at_forty_one(&cards_handed_to_the_opening),
+            4,
+            "试满之后不再开"
+        );
+    }
+
+    /// 产品那一份开一张卡那一步（`opening_of_a_card_within_the_limit`，一次至多等 50 毫秒）、还没回来的开卡那一张表是 `openings_not_returned`：
+    /// 起卡那一件头一次挂着直到用例放开（至多 20 秒），之后当场开成；每被起一次往 `starts_made` 加一、往 `start_begun` 送一声。
+    fn opening_held_at_its_first_start_until_released(
+        start_released: &Arc<Mutex<std::sync::mpsc::Receiver<()>>>,
+        starts_made: &Arc<std::sync::atomic::AtomicU32>,
+        start_begun: &std::sync::mpsc::Sender<()>,
+        openings_not_returned: &OpeningsNotReturnedInThisProcess,
+    ) -> impl FnMut(
+        &DiscreteGpuOnThisHost,
+        Option<GpuMemoryInBytes>,
+        Duration,
+        CostAtTheOpening,
+    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
+           + Send
+           + 'static {
+        let start_released = Arc::clone(start_released);
+        let starts_counted = Arc::clone(starts_made);
+        let start_begun = start_begun.clone();
+        opening_of_a_card_within_the_limit(
+            Duration::from_millis(50),
+            move |discrete_gpu: &DiscreteGpuOnThisHost,
+                  memory_limit,
+                  _wait_limit,
+                  bytes_leaked_by_earlier_mounts| {
+                let card = CardComputedOnTheCpuForTests::of_the_address(
+                    &discrete_gpu.pci_bus_address,
+                    memory_limit.map(|memory_limit| memory_limit.0),
+                    bytes_leaked_by_earlier_mounts.0,
+                );
+                let start_released_seen_by_the_start = Arc::clone(&start_released);
+                let starts_counted_by_the_start = Arc::clone(&starts_counted);
+                let start_begun_by_the_start = start_begun.clone();
+                move || {
+                    let starts_before = starts_counted_by_the_start
+                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
+                    // 收的那一头在用例里一直拿着。
+                    let _signal_sent = start_begun_by_the_start.send(());
+                    if starts_before == 0 {
+                        let _released_or_not = start_released_seen_by_the_start
+                            .lock()
+                            .expect("用例没在别处 panic 过")
+                            .recv_timeout(Duration::from_secs(20));
+                    }
+                    Ok(card)
+                }
+            },
+            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            openings_not_returned.clone(),
+        )
+    }
+
+    /// 第 4 条「同一张卡前一次开卡没回来就不再起下一次」管到整个进程：前一次挂载在「0000:41:00.0」上的一次开卡到点之后还挂着，
+    /// 下一次挂载（另起一份开卡那一步，共用进程开卡器那一张表）开它不再起，算没开成；起卡那一件一共只被起 1 次。
+    #[test]
+    fn an_opening_hung_in_an_earlier_mount_keeps_a_later_mount_from_starting_another_on_that_card()
+    {
+        let discrete_gpu = discrete_gpu_not_opened_at("0000:41:00.0");
+        let openings_not_returned = OpeningsNotReturnedInThisProcess::with_none_not_returned();
+        let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
+        let start_released = Arc::new(Mutex::new(start_released));
+        let (start_begun, every_start_begun) = std::sync::mpsc::channel::<()>();
+        let starts_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
+        let mut opening_of_the_earlier_mount = opening_held_at_its_first_start_until_released(
+            &start_released,
+            &starts_made,
+            &start_begun,
+            &openings_not_returned,
+        );
+        let mut opening_of_the_later_mount = opening_held_at_its_first_start_until_released(
+            &start_released,
+            &starts_made,
+            &start_begun,
+            &openings_not_returned,
+        );
+        let earlier_mount = opening_of_the_earlier_mount(
+            &discrete_gpu,
+            None,
+            BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+            CostAtTheOpening::MeasuredUnderTheMemoryLimit,
+        )
+        .err();
+        every_start_begun
+            .recv_timeout(Duration::from_secs(20))
+            .expect("前一次挂载的那一次开卡 20 秒内起了");
+        let later_mount = opening_of_the_later_mount(
+            &discrete_gpu,
+            None,
+            BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+            CostAtTheOpening::MeasuredUnderTheMemoryLimit,
+        )
+        .err();
+        let starts_after_the_later_mount = starts_made.load(std::sync::atomic::Ordering::SeqCst);
+        release_the_start
+            .send(())
+            .expect("起卡那一件还在等放开：收的那一头还在");
+        assert!(matches!(
+            earlier_mount,
+            Some(GpuFailureCause::WaitForTheGpuExceededTheLimit { .. })
+        ));
+        assert_eq!(
+            later_mount,
+            Some(GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned),
+            "下一次挂载在这张卡上不再起"
+        );
+        assert_eq!(starts_after_the_later_mount, 1);
+    }
+
+    /// 前一次开卡回来了（那条线程跑完）就从那一张表里划掉：同一张卡下一次照起、开成。
+    #[test]
+    fn an_opening_that_returns_after_its_limit_lets_the_next_opening_of_that_card_start() {
+        let discrete_gpu = discrete_gpu_not_opened_at("0000:41:00.0");
+        let openings_not_returned = OpeningsNotReturnedInThisProcess::with_none_not_returned();
+        let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
+        let start_released = Arc::new(Mutex::new(start_released));
+        let (start_begun, every_start_begun) = std::sync::mpsc::channel::<()>();
+        let starts_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
+        let mut opening = opening_held_at_its_first_start_until_released(
+            &start_released,
+            &starts_made,
+            &start_begun,
+            &openings_not_returned,
+        );
+        let hung = opening(
+            &discrete_gpu,
+            None,
+            BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+            CostAtTheOpening::MeasuredUnderTheMemoryLimit,
+        );
+        every_start_begun
+            .recv_timeout(Duration::from_secs(20))
+            .expect("那一次开卡 20 秒内起了");
+        release_the_start
+            .send(())
+            .expect("起卡那一件还在等放开：收的那一头还在");
+        let returned_within_twenty_seconds = (0..2000).any(|_poll| {
+            if openings_not_returned
+                .devices_with_an_opening_not_returned()
+                .is_empty()
+            {
+                return true;
+            }
+            std::thread::sleep(Duration::from_millis(10));
+            false
+        });
+        let again = opening(
+            &discrete_gpu,
+            None,
+            BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+            CostAtTheOpening::MeasuredUnderTheMemoryLimit,
+        );
+        assert!(matches!(
+            hung.err(),
+            Some(GpuFailureCause::WaitForTheGpuExceededTheLimit { .. })
+        ));
+        assert!(
+            returned_within_twenty_seconds,
+            "放开之后那一次开卡 20 秒内回来了"
+        );
+        assert!(again.is_ok(), "回来之后照起：{:?}", again.err());
+        assert_eq!(starts_made.load(std::sync::atomic::Ordering::SeqCst), 2);
+    }
+
+    /// 运行库收的那一份读得到切换关卡泄漏了哪几张：产品开卡器开成之后换成运行库收的样子（`opened_background_gpu_for_the_mount_runtime`），
+    /// 「0000:41:00.0」上记着一次还在跑的提交（600 字节），核心层挂载那一组只点「0000:01:00.0」——那一刻关掉「0000:41:00.0」、整张泄漏，
+    /// 运行库那一份读出它与 600 字节；卸载（丢掉把手）之后读得到丢掉那一刻泄漏的（一张都没有）。
+    #[test]
+    fn the_leaks_at_a_switch_are_read_through_what_the_product_opener_hands_to_the_mount_runtime() {
+        let (background_gpu, _cards_handed_to_the_opening, _drops_of_the_card_at_forty_one) =
+            background_gpu_with_both_cards_opened_at_the_mount(
+                opening_that_always_opens,
+                |card| card.with_a_submission_running_past_its_wait(600),
+                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            );
+        let only_the_card_at_one = BackgroundComputeSettings {
+            background_compute: BackgroundComputeBackend::Gpu,
+            background_gpu_devices: Some(vec![card_identifier("0000:01:00.0")]),
+            background_gpu_memory_limit: None,
+            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+        };
+        let opened = crate::gpu_opener_of_this_process::opened_background_gpu_for_the_mount_runtime(
+            crate::background_gpu_compute::background_gpu_compute_of_the_opened_gpu(background_gpu),
+        );
+        let gpus_leaked = opened
+            .gpus_leaked
+            .clone()
+            .expect("产品开卡器恒给读泄漏的那一份");
+        let background_compute =
+            singlefs_core::background_compute::BackgroundComputeOfThisMount::of_a_mount(
+                singlefs_core::background_compute::admit_background_compute_settings_with_the_gpu_compute(
+                    only_the_card_at_one,
+                    Some(opened.gpu_compute),
+                )
+                .expect("卡清单列着「0000:01:00.0」"),
+            );
+        assert_eq!(
+            gpus_leaked.gpus_leaked_when_closed_at_a_switch(),
+            vec![singlefs_mount_runtime::GpuLeakedByTheBackgroundGpu {
+                device: card_identifier("0000:41:00.0"),
+                memory_held_on_the_device: GpuMemoryInBytes(600),
+            }]
+        );
+        assert_eq!(gpus_leaked.gpus_leaked_when_dropped(), None, "还没丢");
+        drop(background_compute);
+        assert_eq!(
+            gpus_leaked.gpus_leaked_when_dropped(),
+            Some(Vec::new()),
+            "丢掉之后读得到：开着的那张没有还在跑的提交，一张都没泄漏"
+        );
+    }
+
+    /// 起不了开卡线程、到了间隔而之后又交来了别的一组：照最近那一组起，不为被盖掉的那一组起——开卡那一步只被交「0000:81:00.0」，换上的是它。
+    #[test]
+    fn the_opening_thread_not_started_for_a_superseded_switch_is_started_for_the_latest_settings() {
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                opening_that_always_opens,
+                None,
+                false,
+            );
+        background_gpu.refuse_to_start_the_opening_thread_for_tests(1);
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        let to_the_card_at_eighty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            }
+        );
+        assert_eq!(
+            *cards_handed_to_the_opening
+                .lock()
+                .expect("用例没在别处 panic 过"),
+            vec!["0000:81:00.0".to_string()],
+            "只为最近那一组开"
+        );
+        assert_eq!(
+            participating_devices_of(&background_gpu),
+            vec![card_identifier("0000:81:00.0")]
+        );
+    }
+
+    /// 收下为被盖掉的切换开成的卡、最近那一组还要开卡而没点它：收下那一刻就关掉，不开着等最近那一组开完——切换点「0000:41:00.0」（等用例放开），
+    /// 又来一次只点「0000:81:00.0」；放开之后「0000:41:00.0」开成、收下，为「0000:81:00.0」开卡那一刻「0000:41:00.0」已析构一次。
+    #[test]
+    fn the_card_opened_for_a_superseded_switch_is_closed_as_it_is_taken_in_while_the_latest_switch_still_opens(
+    ) {
+        let (release_the_card_at_forty_one, card_at_forty_one_released) =
+            std::sync::mpsc::channel::<()>();
+        let drops_of_the_card_at_forty_one = Arc::new(std::sync::atomic::AtomicU32::new(0));
+        let drops_noted_by_the_opening = Arc::clone(&drops_of_the_card_at_forty_one);
+        let drops_seen_when_the_latest_card_opens = Arc::new(Mutex::new(None));
+        let drops_seen_by_the_opening = Arc::clone(&drops_seen_when_the_latest_card_opens);
+        let (mut background_gpu, _cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
+                    if discrete_gpu.pci_bus_address == "0000:41:00.0" {
+                        card_at_forty_one_released
+                            .recv_timeout(Duration::from_secs(20))
+                            .map_err(|_not_released| {
+                                reason_the_card_does_not_open_at_the_switch()
+                            })?;
+                        let mut card = opening_that_always_opens(
+                            discrete_gpu,
+                            memory_limit,
+                            wait_limit,
+                            cost_at_the_opening,
+                        )?;
+                        card.drops_counted = Arc::clone(&drops_noted_by_the_opening);
+                        return Ok(card);
+                    }
+                    *drops_seen_by_the_opening
+                        .lock()
+                        .expect("用例没在别处 panic 过") =
+                        Some(drops_noted_by_the_opening.load(std::sync::atomic::Ordering::SeqCst));
+                    opening_that_always_opens(
+                        discrete_gpu,
+                        memory_limit,
+                        wait_limit,
+                        cost_at_the_opening,
+                    )
+                },
+                None,
+                false,
+            );
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        let to_the_card_at_eighty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
+        release_the_card_at_forty_one
+            .send(())
+            .expect("开「0000:41:00.0」那一步还在等放开");
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            }
+        );
+        assert_eq!(
+            *drops_seen_when_the_latest_card_opens
+                .lock()
+                .expect("用例没在别处 panic 过"),
+            Some(1),
+            "为「0000:81:00.0」开卡那一刻，为被盖掉的那一组开成的卡已经关掉"
+        );
+    }
+
+    /// 最近那一组点了两张没开的卡，前一张开成、后一张开不成：这一组没生效，为它开成的那一张关掉，开着的只剩此刻生效那一组的「0000:01:00.0」。
+    #[test]
+    fn the_card_opened_for_the_latest_switch_is_closed_when_another_card_of_it_does_not_open() {
+        let (mut background_gpu, _cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
+                    if discrete_gpu.pci_bus_address == "0000:41:00.0" {
+                        return opening_that_always_opens(
+                            discrete_gpu,
+                            memory_limit,
+                            wait_limit,
+                            cost_at_the_opening,
+                        );
+                    }
+                    Err(reason_the_card_does_not_open_at_the_switch())
+                },
+                None,
+                false,
+            );
+        let to_both_cards_not_opened_at_the_mount = switch_naming_the_cards_admitted_against(
+            &background_gpu,
+            &["0000:41:00.0", "0000:81:00.0"],
+            None,
+        );
+        background_gpu.use_the_settings_from_the_next_batch(&to_both_cards_not_opened_at_the_mount);
+        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+            WaitForTheOpeningAtASwitch::UntilItEnds,
+        );
+        assert!(
+            matches!(
+                &answered,
+                LatestSettingsHandedIn::NotOpened(not_opened)
+                    if not_opened.device == card_identifier("0000:81:00.0")
+            ),
+            "{answered:?}"
+        );
+        assert_eq!(
+            background_gpu.gpus_opened(),
+            vec![card_identifier("0000:01:00.0")],
+            "为没生效的那一组开成的「0000:41:00.0」关掉"
+        );
+    }
+
+    /// 重开的间隔照产品是 1 分钟（用例不真等它：只用来看「该等的没等、不该等的等了」）。
+    const REOPENINGS_ONE_MINUTE_APART: ReopeningsOfAGpuNotOpened = ReopeningsOfAGpuNotOpened {
+        interval: Duration::from_secs(60),
+        reopenings_at_most: 3,
+    };
+
+    /// 起不了开卡线程那一回记在头一张要开的卡上：头一回起不了，之后这张卡次次开不成——连同那一回至多 4 次，开卡那一步只被调 3 次，
+    /// 那一问答开不成，原因是起不了线程那一个在前、3 次开不成在后。
+    #[test]
+    fn the_opening_thread_not_started_once_counts_toward_the_openings_of_the_first_card() {
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_one_card_not_opened_at_the_mount(
+                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
+                    Err(reason_the_card_does_not_open_at_the_switch())
+                },
+                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            );
+        background_gpu.refuse_to_start_the_opening_thread_for_tests(1);
+        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
+        background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+            WaitForTheOpeningAtASwitch::UntilItEnds,
+        );
+        assert!(
+            matches!(
+                &answered,
+                LatestSettingsHandedIn::NotOpened(not_opened)
+                    if not_opened.causes_of_each_opening.len() == 4
+                        && matches!(
+                            not_opened.causes_of_each_opening[0],
+                            GpuFailureCause::OpeningThreadNotStarted { .. }
+                        )
+            ),
+            "{answered:?}"
+        );
+        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 3);
+    }
+
+    /// 叫停之后不再睡重开的间隔：间隔是 1 分钟，头一次开卡在开（等用例放开）时告知一张卡都不参与——放开之后开卡线程不睡那 1 分钟，20 秒之内就开完、放开那一步。
+    #[test]
+    fn no_gpu_taking_part_while_a_card_opens_does_not_sleep_out_the_interval_before_a_reopening() {
+        let (opening_begun, every_opening_begun) = std::sync::mpsc::channel::<()>();
+        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_one_card_not_opened_at_the_mount(
+                opening_that_does_not_open_once_released(opening_begun, opening_released),
+                REOPENINGS_ONE_MINUTE_APART,
+                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            );
+        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
+        background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        every_opening_begun
+            .recv_timeout(Duration::from_secs(20))
+            .expect("开卡线程 20 秒内开始头一次开卡");
+        background_gpu.use_no_gpu_from_the_next_batch();
+        release_the_opening
+            .send(())
+            .expect("开卡那一步还在等放开：收的那一头还在");
+        drop(release_the_opening);
+        let opening_step = Arc::clone(&background_gpu.opening_of_a_gpu_at_a_switch);
+        let opening_thread_done_within_twenty_seconds = (0..2000).any(|_poll| {
+            if opening_step.try_lock().is_ok() {
+                return true;
+            }
+            std::thread::sleep(Duration::from_millis(10));
+            false
+        });
+        assert!(
+            opening_thread_done_within_twenty_seconds,
+            "叫停之后不睡 1 分钟的间隔"
+        );
+        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 1);
+    }
+
+    /// 起不了开卡线程之后到了间隔才再起：间隔是 1 分钟，头一回起不了，两批之间那一问当场问——不再起，照旧是那一次起不了，开卡那一步一次都没被调。
+    #[test]
+    fn the_opening_thread_not_started_is_not_started_again_before_the_interval() {
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_one_card_not_opened_at_the_mount(
+                opening_that_always_opens,
+                REOPENINGS_ONE_MINUTE_APART,
+                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            );
+        background_gpu.refuse_to_start_the_opening_thread_for_tests(1);
+        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
+        background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::NotAtAll
+            ),
+            LatestSettingsHandedIn::StillOpeningItsCards
+        );
+        assert!(
+            matches!(
+                background_gpu.opening_at_a_switch,
+                Some(OpeningAtASwitch::ThreadNotStarted { .. })
+            ),
+            "没到间隔：照旧是那一次起不了"
+        );
+        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 0);
+    }
+
+    /// 告知一张卡都不参与时生效那一组点的卡清掉：之后为被盖掉的一组开成的卡（「0000:01:00.0」），在最近那一组（只点「0000:81:00.0」）开不成时关掉——
+    /// 开着的卡为零。
+    #[test]
+    fn no_gpu_taking_part_forgets_the_cards_of_the_settings_in_effect_so_a_card_opened_later_is_closed(
+    ) {
+        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
+        let (mut background_gpu, _cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                opening_the_card_at_one_once_released_and_no_other(opening_released),
+                None,
+                false,
+            );
+        background_gpu.use_no_gpu_from_the_next_batch();
+        let to_the_card_at_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:01:00.0"], None);
+        let to_the_card_at_eighty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_one);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
+        release_the_opening
+            .send(())
+            .expect("开卡那一步还在等放开：收的那一头还在");
+        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+            WaitForTheOpeningAtASwitch::UntilItEnds,
+        );
+        assert!(
+            matches!(&answered, LatestSettingsHandedIn::NotOpened(_)),
+            "{answered:?}"
+        );
+        assert!(
+            background_gpu.gpus_opened().is_empty(),
+            "开着的卡为零：{:?}",
+            background_gpu.gpus_opened()
+        );
+    }
+
+    /// 告知一张卡都不参与时因显存上限不用的卡一并清掉：挂载那一刻「0000:01:00.0」因显存上限不参与，切到 `cpu` 之后不再报它。
+    #[test]
+    fn no_gpu_taking_part_forgets_the_cards_left_out_under_their_memory_limit() {
+        let (mut background_gpu, _cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                opening_that_always_opens,
+                Some(64),
+                true,
+            );
+        assert!(!background_gpu
+            .gpus_left_out_under_their_memory_limit()
+            .is_empty());
+        background_gpu.use_no_gpu_from_the_next_batch();
+        assert!(background_gpu
+            .gpus_left_out_under_their_memory_limit()
+            .is_empty());
+    }
+
+    // ── 切换时量代价先等账上还在跑的那次提交（替身卡的量代价照产品那一份走账）──
+
+    /// 两张替身卡：「0000:01:00.0」量代价照产品走账（`measured_at_the_mount` 为假时开卡时没量过代价）、「0000:41:00.0」照旧。
+    fn two_cards_measuring_the_cost_through_the_account(
+        measured_at_the_mount: bool,
+    ) -> BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests> {
+        let mut card_at_one = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
+        card_at_one.cost_measured_through_the_checksum_chunk = true;
+        if !measured_at_the_mount {
+            card_at_one.measured_cost = None;
+        }
+        let card_at_forty_one =
+            CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0);
+        let (background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
+            vec![card_at_one, card_at_forty_one],
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        );
+        background_gpu
+    }
+
+    /// 历史甲：「0000:01:00.0」开卡时没量过代价、被晾下；切换请回来，量代价那一次提交等超过上限（账上记成还在跑）。
+    fn card_at_one_whose_measurement_at_a_switch_timed_out(
+    ) -> BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests> {
+        let mut background_gpu = two_cards_measuring_the_cost_through_the_account(false);
+        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
+        let leaving_it_out = settings_admitted_on_the_cards(
+            &background_gpu,
+            None,
+            vec![smallest_piece_bytes - 1, 1 << 30],
+        );
+        background_gpu.use_the_settings_from_the_next_batch(&leaving_it_out);
+        background_gpu.cards_on_this_host[0]
+            .submissions_timing_out
+            .push_back(true);
+        let bringing_it_back =
+            settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
+        background_gpu.use_the_settings_from_the_next_batch(&bringing_it_back);
+        assert!(
+            background_gpu.cards_on_this_host[0]
+                .budget
+                .memory_account()
+                .has_a_submission_running_past_its_wait(),
+            "前缀：量代价那一次等超过上限，账上记着还在跑"
+        );
+        background_gpu
+    }
+
+    /// 历史乙：「0000:01:00.0」量过代价、参与；一批校验和头一次交等超过上限、之后三回等前一次都没等到（报这张卡用不了，账上照旧记着还在跑），
+    /// 切换给它装不下最小一片的上限（晾下）。
+    fn card_at_one_left_out_while_its_timed_out_submission_runs(
+    ) -> BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests> {
+        let mut background_gpu = two_cards_measuring_the_cost_through_the_account(true);
+        {
+            let card_at_one = &mut background_gpu.cards_on_this_host[0];
+            card_at_one.submissions_timing_out = VecDeque::from([true]);
+            card_at_one.earlier_submission_waits_timing_out = VecDeque::from([true, true, true]);
+        }
+        let unit_bytes = unit_bytes_of(1, 1000);
+        let batch = background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes));
+        assert!(batch.is_err(), "前缀：这一批报这张卡用不了");
+        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
+        let leaving_it_out = settings_admitted_on_the_cards(
+            &background_gpu,
+            None,
+            vec![smallest_piece_bytes - 1, 1 << 30],
+        );
+        background_gpu.use_the_settings_from_the_next_batch(&leaving_it_out);
+        assert!(
+            background_gpu.cards_on_this_host[0]
+                .budget
+                .memory_account()
+                .has_a_submission_running_past_its_wait(),
+            "前缀：晾下之后账上照旧记着还在跑"
+        );
+        background_gpu
+    }
+
+    /// 之后用户的那一次切换。
+    #[derive(Clone, Copy, Debug)]
+    enum SwitchAfterTheHistory {
+        BothNamedUnderALargeLimit,
+        BothNamedUnderALimitJustHoldingTheSmallestPiece,
+        BothNamedUnderALimitOneByteShort,
+        OnlyTheOtherCardNamed,
+        ToTheCpu,
+    }
+
+    /// 在前缀之后交 `switch`，接住 panic：交回 panic 带的原话（没 panic 是 `None`）与「0000:01:00.0」此刻开没开着、账上还记不记着在跑。
+    fn outcome_of_the_switch_after_the_history(
+        mut background_gpu: BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
+        switch: SwitchAfterTheHistory,
+    ) -> (Option<String>, bool, bool) {
+        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
+        let held_on_the_card_at_one = background_gpu.cards_on_this_host[0]
+            .budget
+            .memory_account()
+            .bytes_held()
+            .0;
+        let settings = match switch {
+            SwitchAfterTheHistory::BothNamedUnderALargeLimit => Some(
+                settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]),
+            ),
+            SwitchAfterTheHistory::BothNamedUnderALimitJustHoldingTheSmallestPiece => {
+                Some(settings_admitted_on_the_cards(
+                    &background_gpu,
+                    None,
+                    vec![held_on_the_card_at_one + smallest_piece_bytes, 1 << 30],
+                ))
+            }
+            SwitchAfterTheHistory::BothNamedUnderALimitOneByteShort => {
+                Some(settings_admitted_on_the_cards(
+                    &background_gpu,
+                    None,
+                    vec![held_on_the_card_at_one + smallest_piece_bytes - 1, 1 << 30],
+                ))
+            }
+            SwitchAfterTheHistory::OnlyTheOtherCardNamed => Some(settings_admitted_on_the_cards(
+                &background_gpu,
+                Some(vec![card_identifier("0000:41:00.0")]),
+                vec![1 << 30],
+            )),
+            SwitchAfterTheHistory::ToTheCpu => None,
+        };
+        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match &settings {
+            Some(settings) => background_gpu.use_the_settings_from_the_next_batch(settings),
+            None => background_gpu.use_no_gpu_from_the_next_batch(),
+        }));
+        let said_by_the_panic = caught.err().map(|payload| {
+            payload
+                .downcast_ref::<&str>()
+                .map(|said| (*said).to_string())
+                .or_else(|| payload.downcast_ref::<String>().cloned())
+                .unwrap_or_default()
+        });
+        let card_at_one = background_gpu
+            .cards_on_this_host
+            .iter()
+            .find(|card| card.discrete_gpu.identifier == card_identifier("0000:01:00.0"));
+        (
+            said_by_the_panic,
+            card_at_one.is_some(),
+            card_at_one.is_some_and(|card| {
+                card.budget
+                    .memory_account()
+                    .has_a_submission_running_past_its_wait()
+            }),
+        )
+    }
+
+    /// 切换让账上还记着在跑的那张卡量代价：先在等的上限之内等它做完、放账，再量——不 panic，量完账上不再记着在跑。
+    #[test]
+    fn a_measurement_at_a_switch_after_the_measurement_timed_out_waits_for_it_under_a_large_limit()
+    {
+        assert_eq!(
+            outcome_of_the_switch_after_the_history(
+                card_at_one_whose_measurement_at_a_switch_timed_out(),
+                SwitchAfterTheHistory::BothNamedUnderALargeLimit
+            ),
+            (None, true, false)
+        );
+    }
+
+    #[test]
+    fn a_measurement_at_a_switch_after_the_measurement_timed_out_waits_for_it_under_a_limit_just_holding_the_smallest_piece(
+    ) {
+        assert_eq!(
+            outcome_of_the_switch_after_the_history(
+                card_at_one_whose_measurement_at_a_switch_timed_out(),
+                SwitchAfterTheHistory::BothNamedUnderALimitJustHoldingTheSmallestPiece
+            ),
+            (None, true, false)
+        );
+    }
+
+    /// 对照：上限少一个字节，这张卡照旧晾下、不量代价——账上照旧记着在跑。
+    #[test]
+    fn a_switch_after_the_measurement_timed_out_leaves_the_card_out_under_a_limit_one_byte_short() {
+        assert_eq!(
+            outcome_of_the_switch_after_the_history(
+                card_at_one_whose_measurement_at_a_switch_timed_out(),
+                SwitchAfterTheHistory::BothNamedUnderALimitOneByteShort
+            ),
+            (None, true, true)
+        );
+    }
+
+    #[test]
+    fn a_switch_after_the_measurement_timed_out_naming_only_the_other_card_closes_it() {
+        assert_eq!(
+            outcome_of_the_switch_after_the_history(
+                card_at_one_whose_measurement_at_a_switch_timed_out(),
+                SwitchAfterTheHistory::OnlyTheOtherCardNamed
+            ),
+            (None, false, false)
+        );
+    }
+
+    #[test]
+    fn a_switch_to_the_cpu_after_the_measurement_timed_out_closes_every_card() {
+        assert_eq!(
+            outcome_of_the_switch_after_the_history(
+                card_at_one_whose_measurement_at_a_switch_timed_out(),
+                SwitchAfterTheHistory::ToTheCpu
+            ),
+            (None, false, false)
+        );
+    }
+
+    #[test]
+    fn a_measurement_at_a_switch_after_a_batch_timed_out_waits_for_it_under_a_large_limit() {
+        assert_eq!(
+            outcome_of_the_switch_after_the_history(
+                card_at_one_left_out_while_its_timed_out_submission_runs(),
+                SwitchAfterTheHistory::BothNamedUnderALargeLimit
+            ),
+            (None, true, false)
+        );
+    }
+
+    #[test]
+    fn a_measurement_at_a_switch_after_a_batch_timed_out_waits_for_it_under_a_limit_just_holding_the_smallest_piece(
+    ) {
+        assert_eq!(
+            outcome_of_the_switch_after_the_history(
+                card_at_one_left_out_while_its_timed_out_submission_runs(),
+                SwitchAfterTheHistory::BothNamedUnderALimitJustHoldingTheSmallestPiece
+            ),
+            (None, true, false)
+        );
+    }
+
+    #[test]
+    fn a_switch_after_a_batch_timed_out_leaves_the_card_out_under_a_limit_one_byte_short() {
+        assert_eq!(
+            outcome_of_the_switch_after_the_history(
+                card_at_one_left_out_while_its_timed_out_submission_runs(),
+                SwitchAfterTheHistory::BothNamedUnderALimitOneByteShort
+            ),
+            (None, true, true)
+        );
+    }
+
+    #[test]
+    fn a_switch_after_a_batch_timed_out_naming_only_the_other_card_closes_it() {
+        assert_eq!(
+            outcome_of_the_switch_after_the_history(
+                card_at_one_left_out_while_its_timed_out_submission_runs(),
+                SwitchAfterTheHistory::OnlyTheOtherCardNamed
+            ),
+            (None, false, false)
+        );
+    }
+
+    #[test]
+    fn a_switch_to_the_cpu_after_a_batch_timed_out_closes_every_card() {
+        assert_eq!(
+            outcome_of_the_switch_after_the_history(
+                card_at_one_left_out_while_its_timed_out_submission_runs(),
+                SwitchAfterTheHistory::ToTheCpu
+            ),
+            (None, false, false)
+        );
+    }
+
+    /// 等不到账上还在跑的那次提交：量代价照出错交回，这张卡不参与，下一批报成换设置时量代价出错（等前一次没等到的那一种）。
+    #[test]
+    fn a_measurement_at_a_switch_whose_wait_for_the_running_submission_times_out_leaves_the_card_out(
+    ) {
+        let mut background_gpu = card_at_one_whose_measurement_at_a_switch_timed_out();
+        background_gpu.cards_on_this_host[0]
+            .earlier_submission_waits_timing_out
+            .push_back(true);
+        let bringing_it_back =
+            settings_admitted_on_the_cards(&background_gpu, None, vec![1 << 30, 1 << 30]);
+        background_gpu.use_the_settings_from_the_next_batch(&bringing_it_back);
+        assert_eq!(
+            background_gpu.batch_not_split_over_the_gpus(),
+            Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
+                device: card_identifier("0000:01:00.0"),
+                cause: GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
+                    wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
+                },
+            })
+        );
+        assert_eq!(
+            participating_devices_of(&background_gpu),
+            vec![card_identifier("0000:41:00.0")]
+        );
+    }
+
+    /// 连着试满的卡在交设置那一刻就答开不成，不为它起开卡线程：「0000:41:00.0」开了 4 次都没开成之后，再交只点它的一组（这一次起线程会被拒）——
+    /// 那一问当场答连着试满、没有在开的那一次，开卡那一步照旧只被调 4 次。
+    #[test]
+    fn the_card_whose_openings_are_used_up_is_answered_not_opened_at_the_handing_in_without_starting_a_thread(
+    ) {
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_one_card_not_opened_at_the_mount(
+                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
+                    Err(reason_the_card_does_not_open_at_the_switch())
+                },
+                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            );
+        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
+        background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        let _not_opened_after_four_openings = background_gpu
+            .put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds,
+            );
+        background_gpu.refuse_to_start_the_opening_thread_for_tests(1);
+        background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        assert!(
+            background_gpu.opening_at_a_switch.is_none(),
+            "不为连着试满的卡起开卡"
+        );
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::NotAtAll
+            ),
+            not_opened_with_the_openings_of_the_card_at_forty_one_used_up()
+        );
+        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 4);
+    }
+
+    /// 叫停之后没开的那几次不算进连着开不成的次数：「0000:41:00.0」头一次开卡在开（等用例放开）时告知一张卡都不参与，放开之后头一次开不成、
+    /// 余下 3 次叫停没开（这 3 次照样交记账那一步，算进去就连同头一次满 4 次）；再交只点它的一组，它照开（这一次开成）——不当试满。
+    #[test]
+    fn openings_called_off_do_not_count_toward_the_openings_of_the_card() {
+        let (opening_begun, every_opening_begun) = std::sync::mpsc::channel::<()>();
+        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
+        let mut is_the_first_opening = true;
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_one_card_not_opened_at_the_mount(
+                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
+                    if is_the_first_opening {
+                        is_the_first_opening = false;
+                        // 收的那一头在用例里一直拿着。
+                        let _signal_sent = opening_begun.send(());
+                        let _released_or_not =
+                            opening_released.recv_timeout(Duration::from_secs(20));
+                        return Err(reason_the_card_does_not_open_at_the_switch());
+                    }
+                    opening_that_always_opens(
+                        discrete_gpu,
+                        memory_limit,
+                        wait_limit,
+                        cost_at_the_opening,
+                    )
+                },
+                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            );
+        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
+        background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        every_opening_begun
+            .recv_timeout(Duration::from_secs(20))
+            .expect("开卡线程 20 秒内开始头一次开卡");
+        background_gpu.use_no_gpu_from_the_next_batch();
+        release_the_opening
+            .send(())
+            .expect("开卡那一步还在等放开：收的那一头还在");
+        drop(
+            background_gpu
+                .opening_of_a_gpu_at_a_switch
+                .lock()
+                .expect("开一次卡里的 panic 在那一步就接住了：锁没被毒化"),
+        );
+        background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            },
+            "叫停的那几次不算：照开、开成"
+        );
+        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 2);
+    }
+
+    /// 开卡的结局按交设置的次序认：切换点「0000:41:00.0」（头一次开卡等用例放开，次次开不成），它在开时又来一次只点「0000:81:00.0」——
+    /// 「0000:41:00.0」重开到上限都没开成的结局不报给最近那一组，为最近那一组开「0000:81:00.0」、开成、换上。
+    #[test]
+    fn the_failure_of_an_opening_for_a_superseded_switch_is_not_reported_to_the_latest_switch() {
+        let (release_the_card_at_forty_one, card_at_forty_one_released) =
+            std::sync::mpsc::channel::<()>();
+        let (mut background_gpu, _cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
+                    if discrete_gpu.pci_bus_address == "0000:41:00.0" {
+                        let _released_or_not =
+                            card_at_forty_one_released.recv_timeout(Duration::from_secs(20));
+                        return Err(reason_the_card_does_not_open_at_the_switch());
+                    }
+                    opening_that_always_opens(
+                        discrete_gpu,
+                        memory_limit,
+                        wait_limit,
+                        cost_at_the_opening,
+                    )
+                },
+                None,
+                false,
+            );
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        let to_the_card_at_eighty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:81:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_eighty_one);
+        release_the_card_at_forty_one
+            .send(())
+            .expect("开「0000:41:00.0」那一步还在等放开");
+        drop(release_the_card_at_forty_one);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            },
+            "为被盖掉的那一组开不成的不报给最近那一组"
+        );
+        assert_eq!(
+            participating_devices_of(&background_gpu),
+            vec![card_identifier("0000:81:00.0")]
+        );
+    }
+
+    /// 最近那一组点的卡被被盖掉的那一次开卡连着试满：切换点「0000:41:00.0」（头一次开卡等用例放开，次次开不成），它在开时又来一次点它与「0000:81:00.0」的一组——
+    /// 为被盖掉的那一组开了 4 次之后，为最近那一组不再开「0000:41:00.0」，当场答连着试满；开卡那一步对它一共只被调 4 次，「0000:81:00.0」一次都没开。
+    #[test]
+    fn the_latest_switch_naming_a_card_used_up_by_a_superseded_switch_is_answered_not_opened() {
+        let (release_the_card_at_forty_one, card_at_forty_one_released) =
+            std::sync::mpsc::channel::<()>();
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
+                    if discrete_gpu.pci_bus_address == "0000:41:00.0" {
+                        let _released_or_not =
+                            card_at_forty_one_released.recv_timeout(Duration::from_secs(20));
+                        return Err(reason_the_card_does_not_open_at_the_switch());
+                    }
+                    opening_that_always_opens(
+                        discrete_gpu,
+                        memory_limit,
+                        wait_limit,
+                        cost_at_the_opening,
+                    )
+                },
+                None,
+                false,
+            );
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        let to_both_cards_not_opened_at_the_mount = switch_naming_the_cards_admitted_against(
+            &background_gpu,
+            &["0000:41:00.0", "0000:81:00.0"],
+            None,
+        );
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        background_gpu.use_the_settings_from_the_next_batch(&to_both_cards_not_opened_at_the_mount);
+        release_the_card_at_forty_one
+            .send(())
+            .expect("开「0000:41:00.0」那一步还在等放开");
+        drop(release_the_card_at_forty_one);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            not_opened_with_the_openings_of_the_card_at_forty_one_used_up()
+        );
+        assert_eq!(
+            *cards_handed_to_the_opening
+                .lock()
+                .expect("用例没在别处 panic 过"),
+            vec!["0000:41:00.0".to_string(); 4]
+        );
+    }
 }
````

## crates/singlefs-background-gpu/src/background_gpu_compute.rs

````diff
--- a/crates/singlefs-background-gpu/src/background_gpu_compute.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
@@ -38,8 +38,9 @@
     open_background_gpu_with_the_memory_leaked_in_this_process, BackgroundGpu,
     BackgroundGpuOpenRefused, BackgroundGpuOverTheCards, BatchNotSplitOverTheGpus,
     CardOfTheBackgroundGpu, ChecksumBatchFailed, GpuFailureCause, GpuNotOpenedAfterTheReopenings,
-    LatestSettingsHandedIn, LiveSlotBatchFailed, RegionRecordBatchFailed,
-    SmallestPieceNotHeldOnTheGpu, VulkanInstanceOfThisProcess, WaitForTheOpeningAtASwitch,
+    LatestSettingsHandedIn, LiveSlotBatchFailed, OpeningsNotReturnedInThisProcess,
+    RegionRecordBatchFailed, SmallestPieceNotHeldOnTheGpu, VulkanInstanceOfThisProcess,
+    WaitForTheOpeningAtASwitch,
 };
 use crate::checksum_kernels::{
     reconciled_unit_checksums, ChecksumKernelOutcome, UnitChecksumToVerify,
@@ -224,6 +225,7 @@
 /// （`use_the_settings_from_the_next_batch`），之后每次切换生效时另交，参与的卡、显存上限、等的上限都照核心层交来的那一组。
 /// `memory_leaked_in_this_process` 是调用方在进程里留着的那一份：之前几次挂载泄漏在一张卡上的字节先从它的余量里扣掉，这一次卸载泄漏的记进去。
 /// `vulkan_instance` 同样是调用方在进程里留着的那一份（[`VulkanInstanceOfThisProcess`]）：开卡的枚举与之后现查本机在的独显都用它，不另建。
+/// `openings_not_returned_in_this_process` 也是调用方在进程里留着的那一份：一张卡上前一次开卡到点之后还没回来，这一次挂载与之后的切换都不在它上面再起。
 ///
 /// # Errors
 /// [`BackgroundGpuOpenRefused`] 的五种：设置被拒（GPU 不在、上限个数不对、上限超过显存总量……）、点名的卡不在这一次枚举的清单里
@@ -232,11 +234,13 @@
 pub fn open_background_gpu_compute(
     selection: &BackgroundGpuSelection,
     memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
+    openings_not_returned_in_this_process: &OpeningsNotReturnedInThisProcess,
     vulkan_instance: &VulkanInstanceOfThisProcess,
 ) -> Result<OpenedBackgroundGpuCompute, BackgroundGpuOpenRefused> {
     let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
         selection,
         memory_leaked_in_this_process,
+        openings_not_returned_in_this_process,
         vulkan_instance,
         |_gpus_to_open| {},
     )?;
@@ -245,9 +249,10 @@
 
 /// 开成的后台 GPU 交成核心层挂载与切换收的把手，连同开着的卡、因显存上限不用、没收进清单的卡与丢掉之后泄漏了哪几张的那一份
 /// （[`OpenedBackgroundGpuCompute`]）。进程开卡器（`crate::gpu_opener_of_this_process`）自己开卡（要枚举完报回要开几张卡）之后也走这一步。
+/// 对任何一种卡都一样（产品是 [`BackgroundGpu`]，测试换上在 CPU 上算的替身卡）。
 #[must_use]
-pub fn background_gpu_compute_of_the_opened_gpu(
-    background_gpu: BackgroundGpu,
+pub fn background_gpu_compute_of_the_opened_gpu<Card: CardOfTheBackgroundGpu>(
+    background_gpu: BackgroundGpuOverTheCards<Card>,
 ) -> OpenedBackgroundGpuCompute {
     let gpus_opened_at_the_mount = background_gpu.gpus_opened();
     let gpus_left_out_under_their_memory_limit = background_gpu
@@ -260,7 +265,7 @@
     let gpus_leaked_when_closed_at_a_switch = background_gpu.gpus_leaked_when_closed_at_a_switch();
     OpenedBackgroundGpuCompute {
         gpu_compute: BackgroundGpuComputeHandle::of_an_implementation(
-            BackgroundGpuComputeOnTheParticipatingGpus::with_no_entry_uncertain(background_gpu),
+            BackgroundGpuComputeThroughBothKernels::with_no_entry_uncertain(background_gpu),
         ),
         gpus_opened_at_the_mount,
         gpus_left_out_under_their_memory_limit,
@@ -287,9 +292,9 @@
     }
 }
 
-/// 一张卡上的 GPU 出错报给核心层：等超过上限的两种（这一批的那一次、前一次超时的那一次没做完）是 `WaitForTheGpuExceededTheLimit`，
-/// 带上限本身；重交 3 次都等超过上限的（这张卡用不了，第 4 条「都超时才报警」）是 `GpuUnusableAfterResubmissions`，带重交了几次、
-/// 最后那一次等的上限与是哪一次提交；别的是 `GpuFailed`，原因一一对过去、原话照带。卡照带。
+/// 一张卡上的 GPU 出错报给核心层：等超过上限的三种（这一批的那一次、前一次超时的那一次没做完、冲掉待写的那一次空提交没做完）是
+/// `WaitForTheGpuExceededTheLimit`，带上限本身与是哪一次提交；重交 3 次都等超过上限的（这张卡用不了，第 4 条「都超时才报警」）是
+/// `GpuUnusableAfterResubmissions`，带重交了几次、最后那一次等的上限与是哪一次提交；别的是 `GpuFailed`，原因一一对过去（开卡那几种各对各的成员）、原话照带。卡照带。
 #[must_use]
 pub fn core_failure_of_the_gpu(
     device: &GpuDeviceIdentifier,
@@ -316,11 +321,40 @@
                     SubmissionWaitedForPastTheLimit::EarlierTimedOutSubmissionStillRunning,
             }
         }
+        GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { wait_limit } => {
+            BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit {
+                device: device.clone(),
+                wait_limit: *wait_limit,
+                submission_waited_for:
+                    SubmissionWaitedForPastTheLimit::EmptySubmissionFlushingThePendingWrites,
+            }
+        }
         GpuFailureCause::DeviceOrPipelineDidNotStart { reason } => {
             failed_on_the_device(GpuFailureOnTheDevice::DeviceOrPipelineDidNotStart {
                 reason: reason.clone(),
             })
         }
+        GpuFailureCause::OpeningThreadNotStarted { reason } => {
+            failed_on_the_device(GpuFailureOnTheDevice::OpeningThreadNotStarted {
+                reason: reason.clone(),
+            })
+        }
+        GpuFailureCause::OpeningPanicked { said_by_the_panic } => {
+            failed_on_the_device(GpuFailureOnTheDevice::OpeningPanicked {
+                said_by_the_panic: said_by_the_panic.clone(),
+            })
+        }
+        GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned => {
+            failed_on_the_device(GpuFailureOnTheDevice::EarlierOpeningOfTheDeviceNotReturned)
+        }
+        GpuFailureCause::OpeningCalledOff => {
+            failed_on_the_device(GpuFailureOnTheDevice::OpeningCalledOff)
+        }
+        GpuFailureCause::OpeningsInThisMountUsedUp { failed_openings } => {
+            failed_on_the_device(GpuFailureOnTheDevice::OpeningsInThisMountUsedUp {
+                failed_openings: *failed_openings,
+            })
+        }
         GpuFailureCause::UncapturedErrorOrDeviceLost { reason } => {
             failed_on_the_device(GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost {
                 reason: reason.clone(),
@@ -371,11 +405,25 @@
                 last_submission_waited_for:
                     SubmissionWaitedForPastTheLimit::EarlierTimedOutSubmissionStillRunning,
             },
-            // 只有等超过上限的两种才重交（`crate::waiting_for_the_gpu::outcome_resubmitted_after_timeouts`），别的原因当场交回、不包进这一种；
+            GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { wait_limit } => {
+                BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
+                    device: device.clone(),
+                    resubmissions: *resubmissions,
+                    wait_limit: *wait_limit,
+                    last_submission_waited_for:
+                        SubmissionWaitedForPastTheLimit::EmptySubmissionFlushingThePendingWrites,
+                }
+            }
+            // 只有等超过上限的三种才重交（`crate::waiting_for_the_gpu::outcome_resubmitted_after_timeouts`），别的原因当场交回、不包进这一种；
             // 包进来了也照最后那一次的原因如实报。
             last_cause_other_than_a_wait_past_the_limit @ (GpuFailureCause::DeviceOrPipelineDidNotStart {
                 ..
             }
+            | GpuFailureCause::OpeningThreadNotStarted { .. }
+            | GpuFailureCause::OpeningPanicked { .. }
+            | GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned
+            | GpuFailureCause::OpeningCalledOff
+            | GpuFailureCause::OpeningsInThisMountUsedUp { .. }
             | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
             | GpuFailureCause::WaitForTheSubmissionFailed { .. }
             | GpuFailureCause::ReadbackMappingFailed { .. }
@@ -454,7 +502,7 @@
     )
 }
 
-/// 一张参与的卡连这一批最小的一片都装不下报给核心层：`InputNotSupportedOnTheGpu` 的「一条装不进一段」，`input_bytes` 是最小的一片合占的字节数
+/// 一张参与的卡连这一批最小的一片都装不下报给核心层：`InputNotSupportedOnTheGpu` 的「一条装不进一段」，`smallest_piece_bytes` 是最小的一片合占的字节数
 /// （这张卡至少要剩这么多）。
 #[must_use]
 pub fn core_failure_of_the_smallest_piece_not_held(
@@ -464,7 +512,7 @@
         InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice {
             device: smallest_piece_not_held.device.clone(),
             entry_position: smallest_piece_not_held.entry_position,
-            input_bytes: smallest_piece_not_held.smallest_piece_bytes,
+            smallest_piece_bytes: smallest_piece_not_held.smallest_piece_bytes,
         },
     )
 }
@@ -803,15 +851,56 @@
             ),
             (
                 GpuFailureCause::OutOfMemoryCreatingTheBuffers {
-                    bytes_held_by_the_chunk: 4096,
+                    bytes_held_by_the_chunk: GpuMemoryInBytes(4096),
                     reason: "假的：显存不足".to_string(),
                 },
                 failed_on_the_card(GpuFailureOnTheDevice::OutOfMemoryCreatingTheBuffers {
-                    bytes_held_by_the_chunk: 4096,
+                    bytes_held_by_the_chunk: GpuMemoryInBytes(4096),
                     reason: "假的：显存不足".to_string(),
                 }),
             ),
             (
+                GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit {
+                    wait_limit,
+                },
+                BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit {
+                    device: card_named_in_the_failures(),
+                    wait_limit,
+                    submission_waited_for:
+                        SubmissionWaitedForPastTheLimit::EmptySubmissionFlushingThePendingWrites,
+                },
+            ),
+            (
+                GpuFailureCause::OpeningThreadNotStarted {
+                    reason: "假的：起不了开卡线程".to_string(),
+                },
+                failed_on_the_card(GpuFailureOnTheDevice::OpeningThreadNotStarted {
+                    reason: "假的：起不了开卡线程".to_string(),
+                }),
+            ),
+            (
+                GpuFailureCause::OpeningPanicked {
+                    said_by_the_panic: "假的：开卡 panic".to_string(),
+                },
+                failed_on_the_card(GpuFailureOnTheDevice::OpeningPanicked {
+                    said_by_the_panic: "假的：开卡 panic".to_string(),
+                }),
+            ),
+            (
+                GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned,
+                failed_on_the_card(GpuFailureOnTheDevice::EarlierOpeningOfTheDeviceNotReturned),
+            ),
+            (
+                GpuFailureCause::OpeningCalledOff,
+                failed_on_the_card(GpuFailureOnTheDevice::OpeningCalledOff),
+            ),
+            (
+                GpuFailureCause::OpeningsInThisMountUsedUp { failed_openings: 4 },
+                failed_on_the_card(GpuFailureOnTheDevice::OpeningsInThisMountUsedUp {
+                    failed_openings: 4,
+                }),
+            ),
+            (
                 GpuFailureCause::NotOpenedAfterTheReopenings {
                     causes_of_each_opening: vec![
                         GpuFailureCause::DeviceOrPipelineDidNotStart {
@@ -838,7 +927,8 @@
         ]
     }
 
-    /// 第 4 条「停下并报错」：三段的每一种 GPU 出错报给核心层时都带着出在哪张卡、什么原因，等超过上限的两种报成等超过上限、分得清是哪一次提交。
+    /// 第 4 条「停下并报错」：三段的每一种 GPU 出错报给核心层时都带着出在哪张卡、什么原因，等超过上限的三种报成等超过上限、分得清是哪一次提交；
+    /// 开卡那几种（起不了线程、panic、前一次没回来、叫停、连着试满）各报各的成员。
     #[test]
     fn every_gpu_failure_is_reported_to_the_core_with_its_card_and_cause() {
         for (cause, expected) in every_gpu_failure_cause_and_its_core_failure() {
@@ -876,13 +966,13 @@
             device: card_named_in_the_failures(),
             entry_position: 3,
             memory_limit: Some(GpuMemoryInBytes(50)),
-            smallest_piece_bytes: 104,
+            smallest_piece_bytes: GpuMemoryInBytes(104),
         };
         let reported = BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
             InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice {
                 device: card_named_in_the_failures(),
                 entry_position: 3,
-                input_bytes: 104,
+                smallest_piece_bytes: GpuMemoryInBytes(104),
             },
         );
         assert_eq!(
@@ -946,6 +1036,12 @@
                 },
                 SubmissionWaitedForPastTheLimit::EarlierTimedOutSubmissionStillRunning,
             ),
+            (
+                GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit {
+                    wait_limit,
+                },
+                SubmissionWaitedForPastTheLimit::EmptySubmissionFlushingThePendingWrites,
+            ),
         ] {
             let unusable = GpuFailureCause::UnusableAfterResubmissions {
                 resubmissions: 3,
@@ -994,14 +1090,14 @@
             GpuLeftOutUnderItsMemoryLimit {
                 device: card_named_in_the_failures(),
                 memory_limit: GpuMemoryInBytes(50),
-                smallest_piece_bytes: 976,
-                bytes_already_held_on_the_device: 0,
+                smallest_piece_bytes: GpuMemoryInBytes(976),
+                bytes_already_held_on_the_device: GpuMemoryInBytes(0),
             },
             GpuLeftOutUnderItsMemoryLimit {
                 device: GpuDeviceIdentifier::of_opaque_bytes(b"0000:02:00.0".to_vec()),
                 memory_limit: GpuMemoryInBytes(900),
-                smallest_piece_bytes: 976,
-                bytes_already_held_on_the_device: 600,
+                smallest_piece_bytes: GpuMemoryInBytes(976),
+                bytes_already_held_on_the_device: GpuMemoryInBytes(600),
             },
         ];
         let expected =
@@ -1065,14 +1161,14 @@
     ) {
         let gpus_short_of_free_memory = vec![GpuShortOfFreeMemoryForTheSmallestPiece {
             device: card_named_in_the_failures(),
-            bytes_of_the_last_chunk_not_created: 64,
-            smallest_piece_bytes: 64,
+            bytes_of_the_last_chunk_not_created: GpuMemoryInBytes(64),
+            smallest_piece_bytes: GpuMemoryInBytes(64),
         }];
         let gpus_left_out_under_their_memory_limit = vec![GpuLeftOutUnderItsMemoryLimit {
             device: GpuDeviceIdentifier::of_opaque_bytes(b"0000:02:00.0".to_vec()),
             memory_limit: GpuMemoryInBytes(50),
-            smallest_piece_bytes: 64,
-            bytes_already_held_on_the_device: 0,
+            smallest_piece_bytes: GpuMemoryInBytes(64),
+            bytes_already_held_on_the_device: GpuMemoryInBytes(0),
         }];
         let expected =
             BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
````

## crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs

````diff
--- a/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs
+++ b/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs
@@ -46,7 +46,8 @@
 };
 use singlefs_background_gpu::background_gpu::{
     host_cpu_time_of_this_thread, monotonic_raw_time_now,
-    open_background_gpu_with_the_memory_leaked_in_this_process, VulkanInstanceOfThisProcess,
+    open_background_gpu_with_the_memory_leaked_in_this_process, OpeningsNotReturnedInThisProcess,
+    VulkanInstanceOfThisProcess,
 };
 use singlefs_background_gpu::background_gpu_compute::{
     open_background_gpu_compute, BackgroundGpuComputeOnTheParticipatingGpus,
@@ -4259,8 +4260,13 @@
     let selection = BackgroundGpuSelection::of_the_settings(settings);
     match arm.rejudging {
         RejudgingOfTheArm::NotRejudged => {
-            let opened = open_background_gpu_compute(&selection, memory_leaked, vulkan_instance)
-                .map_err(|refused| format!("开后台 GPU：{refused:?}"))?;
+            let opened = open_background_gpu_compute(
+                &selection,
+                memory_leaked,
+                &OpeningsNotReturnedInThisProcess::with_none_not_returned(),
+                vulkan_instance,
+            )
+            .map_err(|refused| format!("开后台 GPU：{refused:?}"))?;
             Ok(GpuOfTheRun {
                 gpu_compute: Some(opened.gpu_compute),
                 rejudge_log: None,
@@ -4271,6 +4277,7 @@
             let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
                 &selection,
                 memory_leaked,
+                &OpeningsNotReturnedInThisProcess::with_none_not_returned(),
                 vulkan_instance,
                 |_gpus_to_open| {},
             )
@@ -7863,6 +7870,7 @@
         let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
             &BackgroundGpuSelection::of_the_settings(&settings),
             memory_leaked,
+            &OpeningsNotReturnedInThisProcess::with_none_not_returned(),
             vulkan_instance_for_the_gpu_arm(vulkan_instance)?,
             |_gpus_to_open| {},
         )
@@ -7964,6 +7972,7 @@
             let background_gpu = open_background_gpu_with_the_memory_leaked_in_this_process(
                 &BackgroundGpuSelection::of_the_settings(&settings),
                 memory_leaked,
+                &OpeningsNotReturnedInThisProcess::with_none_not_returned(),
                 vulkan_instance_for_the_gpu_arm(vulkan_instance)?,
                 |_gpus_to_open| {},
             )
````

## crates/singlefs-background-gpu/src/checksum_kernels.rs

````diff
--- a/crates/singlefs-background-gpu/src/checksum_kernels.rs
+++ b/crates/singlefs-background-gpu/src/checksum_kernels.rs
@@ -14,8 +14,11 @@
 //! （[`crc32c_of_two_pieces_by_the_zero_operator_matrices`]），逐位移那一份核的各片按「乘 x 的 8n 次方再取模」合
 //! （[`crc32c_of_two_pieces_by_multiplying_a_power_of_the_polynomial_variable`]）；CPU 参照照旧对整单元一次算（核心层的 CRC-32C），与两种合法都不共用代码。
 
+use singlefs_core::background_compute::GpuMemoryInBytes;
+
 use crate::memory_limit_and_chunks::{
-    ChunkFootprint, PieceOfAnEntry, COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS,
+    ChunkFootprint, EntryCountOfAChunk, InputLengthOnTheHostInBytes, PieceOfAnEntry,
+    COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS,
     COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION,
 };
 use crate::two_kernel_reconciliation::{
@@ -33,13 +36,13 @@
 /// 两份核的工作组都是 64 个调用（两份 WGSL 里的 `@workgroup_size(64)`），每条一个调用。
 pub const INVOCATIONS_PER_WORKGROUP: u32 = 64;
 /// 参数缓冲占的字节数：着色器只读开头一个 `u32`，缓冲按 16 字节建（uniform 缓冲的常见最小对齐）。
-pub const PARAMETER_BUFFER_BYTES: u64 = 16;
+pub const PARAMETER_BUFFER_BYTES: GpuMemoryInBytes = GpuMemoryInBytes(16);
 /// 每条描述三个 `u32`。
 pub const DESCRIPTOR_WORDS_PER_ENTRY: usize = 3;
-pub const DESCRIPTOR_BYTES_PER_ENTRY: u64 = 12;
+pub const DESCRIPTOR_BYTES_PER_ENTRY: GpuMemoryInBytes = GpuMemoryInBytes(12);
 /// 每条结果两个 `u32`。
 pub const OUTCOME_WORDS_PER_ENTRY: usize = 2;
-pub const OUTCOME_BYTES_PER_ENTRY: u64 = 8;
+pub const OUTCOME_BYTES_PER_ENTRY: GpuMemoryInBytes = GpuMemoryInBytes(8);
 /// 结果缓冲开算之前的填充字：判定字只会被核写成 0 或 1。
 pub const OUTCOME_NOT_WRITTEN_FILL_WORD: u32 = 0xFFFF_FFFF;
 const VERDICT_WORD_MATCHES: u32 = 1;
@@ -50,17 +53,21 @@
 /// 按条排的最大缓冲是描述那一个。
 pub const CHECKSUM_CHUNK_FOOTPRINT: ChunkFootprint = ChunkFootprint {
     bytes_per_chunk_besides_the_entries: COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
-        * PARAMETER_BUFFER_BYTES,
+        .bytes_of_the_copies(PARAMETER_BUFFER_BYTES),
     bytes_per_entry_besides_its_input: COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
-        * DESCRIPTOR_BYTES_PER_ENTRY
-        + COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS * OUTCOME_BYTES_PER_ENTRY,
+        .bytes_of_the_copies(DESCRIPTOR_BYTES_PER_ENTRY)
+        .plus(
+            COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS
+                .bytes_of_the_copies(OUTCOME_BYTES_PER_ENTRY),
+        ),
     bytes_per_entry_in_the_largest_entry_buffer: DESCRIPTOR_BYTES_PER_ENTRY,
 };
 
 /// 校验和核对一段里最小的一片在卡上合占的字节数：一个字节的单元（补到一个整字）连同一段的固定占用。
 #[must_use]
-pub fn bytes_of_the_smallest_checksum_piece() -> u64 {
-    CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(1, 1)
+pub fn bytes_of_the_smallest_checksum_piece() -> GpuMemoryInBytes {
+    CHECKSUM_CHUNK_FOOTPRINT
+        .bytes_held_by_a_chunk(InputLengthOnTheHostInBytes(1), EntryCountOfAChunk(1))
 }
 
 /// 要核的一个单元：调用方给的条目键（同一单元在下一轮还用同一个键，连续不确定按它数）、单元字节、期望的 CRC-32C。
@@ -111,10 +118,10 @@
 ) -> PackedChecksumChunk {
     let total_unit_bytes: usize = units.iter().map(|unit| unit.unit_bytes.len()).sum();
     let padded_bytes = crate::memory_limit_and_chunks::input_buffer_bytes(
-        u64::try_from(total_unit_bytes).expect("usize 装得进 u64"),
+        InputLengthOnTheHostInBytes::of_this_many_bytes(total_unit_bytes),
     );
     let mut input_bytes =
-        Vec::with_capacity(usize::try_from(padded_bytes).expect("一段的输入装得进内存"));
+        Vec::with_capacity(usize::try_from(padded_bytes.0).expect("一段的输入装得进内存"));
     let mut descriptor_words = Vec::with_capacity(units.len() * DESCRIPTOR_WORDS_PER_ENTRY);
     for unit in units {
         let byte_offset = u32::try_from(input_bytes.len())
@@ -125,7 +132,7 @@
         descriptor_words.extend_from_slice(&[byte_offset, byte_length, unit.expected_crc32c]);
     }
     input_bytes.resize(
-        usize::try_from(padded_bytes).expect("一段的输入装得进内存"),
+        usize::try_from(padded_bytes.0).expect("一段的输入装得进内存"),
         0,
     );
     PackedChecksumChunk {
@@ -381,9 +388,9 @@
         .map(|piece| {
             let unit = &units[piece.entry_position];
             let piece_start =
-                usize::try_from(piece.input_bytes.start).expect("片的起点在单元里，装得进 usize");
+                usize::try_from(piece.input_bytes.start.0).expect("片的起点在单元里，装得进 usize");
             let piece_end =
-                usize::try_from(piece.input_bytes.end).expect("片的终点在单元里，装得进 usize");
+                usize::try_from(piece.input_bytes.end.0).expect("片的终点在单元里，装得进 usize");
             UnitChecksumToVerify {
                 entry_key: (),
                 unit_bytes: &unit.unit_bytes[piece_start..piece_end],
@@ -425,9 +432,9 @@
     let mut unit_outcomes = Vec::with_capacity(units.len());
     let mut next_piece = 0;
     for (entry_position, unit) in units.iter().enumerate() {
-        let unit_length = u64::try_from(unit.unit_bytes.len()).expect("装得进 u64");
+        let unit_length = InputLengthOnTheHostInBytes::of_this_many_bytes(unit.unit_bytes.len());
         let first_piece = next_piece;
-        let mut covered_up_to = 0;
+        let mut covered_up_to = InputLengthOnTheHostInBytes(0);
         while next_piece < piece_outcomes.len()
             && piece_outcomes[next_piece].0.entry_position == entry_position
         {
@@ -453,12 +460,12 @@
             table_driven_crc32c = crc32c_of_two_pieces_by_the_zero_operator_matrices(
                 table_driven_crc32c,
                 table_driven.computed_crc32c,
-                piece.input_byte_count(),
+                piece.input_byte_count().0,
             );
             bitwise_crc32c = crc32c_of_two_pieces_by_multiplying_a_power_of_the_polynomial_variable(
                 bitwise_crc32c,
                 bitwise.computed_crc32c,
-                piece.input_byte_count(),
+                piece.input_byte_count().0,
             );
         }
         unit_outcomes.push((
@@ -681,17 +688,17 @@
                 }
             })
             .collect();
-        let lengths: Vec<u64> = unit_bytes
+        let lengths: Vec<InputLengthOnTheHostInBytes> = unit_bytes
             .iter()
-            .map(|bytes| u64::try_from(bytes.len()).expect("装得进 u64"))
+            .map(|bytes| InputLengthOnTheHostInBytes::of_this_many_bytes(bytes.len()))
             .collect();
         let chunks_of_pieces = crate::memory_limit_and_chunks::pieces_cut_to_fit_on_the_gpu(
             &lengths,
             CHECKSUM_CHUNK_FOOTPRINT,
             crate::memory_limit_and_chunks::ChunkBoundsOfAGpu {
-                memory_limit: Some(singlefs_core::background_compute::GpuMemoryInBytes(1024)),
-                largest_storage_binding_in_bytes: 1 << 30,
-                largest_entry_count_per_dispatch: 1 << 20,
+                memory_limit: Some(GpuMemoryInBytes(1024)),
+                largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
+                largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
             },
         )
         .expect("1 KiB 装得下最小的一片");
````

## crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs

````diff
--- a/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
+++ b/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
@@ -1,15 +1,17 @@
 //! 开卡器（D26（后台整理与放置回收） 已定项 13 第 4 条「挂载时由调用方把它的实现递进去」）：运行库的特质
-//! [`singlefs_mount_runtime::OpensTheBackgroundGpu`] 在产品 GPU 包这一侧的实现。进程级的三样由它持有：Vulkan 实例（第一次要开卡才建，之后反复用）、
-//! 泄漏账（这个进程里之前几次挂载泄漏在一张卡上的字节，[`GpuMemoryLeakedInThisProcess`]）、建过几个实例的计数。
+//! [`singlefs_mount_runtime::OpensTheBackgroundGpu`] 在产品 GPU 包这一侧的实现。进程级的四样由它持有：Vulkan 实例（第一次要开卡才建，之后反复用）、
+//! 泄漏账（这个进程里之前几次挂载泄漏在一张卡上的字节，[`GpuMemoryLeakedInThisProcess`]）、哪几张卡上有一次到点之后还没回来的开卡
+//! （[`OpeningsNotReturnedInThisProcess`]，各次挂载与挂着切换开卡共用：前一次挂载里挂住的那张卡，下一次挂载同样不再起）、建过几个实例的计数。
+//! 开成交给运行库的那一份带着读这一次挂载泄漏了哪几张卡的读法（切换关卡时与卸载时，[`opened_background_gpu_for_the_mount_runtime`]）。
 //!
 //! 开卡（枚举适配器在内：wgpu 在造 future 那一步同步做完枚举，等待上限管不到它）放工作线程，调用线程的期限分两步（[`OpeningDeadlinesOfTheOpener`]）：
 //! 建实例、枚举、判卡清单与校验那一步至多等枚举期限；工作线程枚举完报回要开几张卡，期限延到「枚举期限 + 卡数 × 一张卡的期限」
 //! （开卡按卡串行、只开参与的卡，卡数是参与的卡数；每一次开卡至多等单卡上限，到点当那一次没开成，由 `waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit` 管；
 //! 开不成隔 1 分钟重开、至多重开 3 次，所以一张卡的期限是 4 个单卡上限加 3 个间隔，`waiting_for_the_gpu::ReopeningsOfAGpuNotOpened::longest_wait_of_one_gpu`）。
 //! 到点挂载拒绝（[`BackgroundGpuNotOpened::TimedOut`] 带到点那一刻的期限），工作线程留在驱动里不杀（没有安全的杀法）；它回来之前闸住新开卡，不再多起一条枚举线程
-//! （[`BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung`]；同一个驱动加载器里再起一条会不会也挂没量过）。同一个理由管到一次挂载之内：一张卡的一次开卡到点之后
-//! 还没回来，重开（挂载那一刻与挂着切换同一个规矩）不在这张卡上再起一条，算那一次又没开成（`background_gpu::opening_of_a_card_within_the_limit`），
-//! 所以工作线程拒了挂载放开闸时，这一次挂载挂在驱动里的开卡线程每张卡至多一条。它回来之后闸放开、开出来的东西没人要：
+//! （[`BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung`]；同一个驱动加载器里再起一条会不会也挂没量过）。同一个理由管到每一张卡：一张卡的一次开卡到点之后
+//! 还没回来，重开（挂载那一刻与挂着切换同一个规矩）与之后的挂载都不在这张卡上再起一条，算那一次又没开成（`background_gpu::opening_of_a_card_within_the_limit`），
+//! 所以整个进程里挂在驱动里的开卡线程每张卡至多一条；出路只有重启进程。它回来之后闸放开、开出来的东西没人要：
 //! 开成的把手照卸载丢（还记着等超过上限的提交的卡泄漏、记进泄漏账；开成时量代价的提交都已做完，没有这种卡），开卡途中量代价出错而提交还在跑的卡
 //! 在开那张卡的线程上已经泄漏、记进泄漏账（`background_gpu::started_gpu_with_its_cost_measured`）。它 panic 了闸同样放开。
 //! 实例对象在超时之后的状态读不到：造不出真挂住的驱动，这一格核不了。
@@ -25,14 +27,23 @@
 use std::time::{Duration, Instant};
 
 use singlefs_core::background_compute::BackgroundComputeSettings;
-use singlefs_mount_runtime::{BackgroundGpuNotOpened, OpenedBackgroundGpu, OpensTheBackgroundGpu};
+use singlefs_mount_runtime::{
+    BackgroundGpuNotOpened, GpuLeakedByTheBackgroundGpu, OpenedBackgroundGpu,
+    OpensTheBackgroundGpu, ReadsTheGpusLeakedByTheBackgroundGpu,
+};
 
 use crate::adapter_inventory::BackgroundGpuSelection;
 use crate::background_gpu::{
-    open_background_gpu_with_the_memory_leaked_in_this_process, VulkanInstanceOfThisProcess,
+    open_background_gpu_with_the_memory_leaked_in_this_process, OpeningsNotReturnedInThisProcess,
+    VulkanInstanceOfThisProcess,
+};
+use crate::background_gpu_compute::{
+    background_gpu_compute_of_the_opened_gpu, OpenedBackgroundGpuCompute,
+};
+use crate::gpus_of_this_host::{
+    GpuLeakedWhenDropped, GpuMemoryLeakedInThisProcess, GpusLeakedWhenClosedAtASwitch,
+    GpusLeakedWhenTheBackgroundGpuIsDropped,
 };
-use crate::background_gpu_compute::background_gpu_compute_of_the_opened_gpu;
-use crate::gpus_of_this_host::GpuMemoryLeakedInThisProcess;
 use crate::waiting_for_the_gpu::{
     ReopeningsOfAGpuNotOpened, WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
 };
@@ -66,13 +77,14 @@
     }
 }
 
-/// 工作线程上这一次开卡要做的事：拿到这个进程的实例、设置与泄漏账，枚举完把要开几张卡交给最后那个参数（调用线程照它延期限），
+/// 工作线程上这一次开卡要做的事：拿到这个进程的实例、设置、泄漏账与还没回来的开卡那一张表，枚举完把要开几张卡交给最后那个参数（调用线程照它延期限），
 /// 交回开成的把手或开卡器报的原样。
 type OpeningOnTheWorkerThread<Instance> = Arc<
     dyn Fn(
             &Instance,
             &BackgroundGpuSelection,
             &GpuMemoryLeakedInThisProcess,
+            &OpeningsNotReturnedInThisProcess,
             &dyn Fn(usize),
         ) -> Result<OpenedBackgroundGpu, String>
         + Send
@@ -133,6 +145,7 @@
     an_opening_is_hung: Arc<AtomicBool>,
     instance_of_this_process: Arc<Mutex<InstanceOfThisProcess<Instance>>>,
     memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
+    openings_not_returned_in_this_process: OpeningsNotReturnedInThisProcess,
     make_an_instance: InstanceMaker<Instance>,
     open_on_the_worker_thread: OpeningOnTheWorkerThread<Instance>,
 }
@@ -160,30 +173,80 @@
 }
 
 /// 工作线程上的产品开卡（[`open_background_gpu_with_the_memory_leaked_in_this_process`]，枚举完把要开几张卡交给 `report_the_gpus_to_open`），
-/// 开成的交成核心层收的把手（[`background_gpu_compute_of_the_opened_gpu`]），结局换成运行库收的样子。
+/// 开成的交成核心层收的把手（[`background_gpu_compute_of_the_opened_gpu`]），结局换成运行库收的样子（[`opened_background_gpu_for_the_mount_runtime`]）。
 fn the_background_gpu_opened_by_the_product_path(
     vulkan_instance: &VulkanInstanceOfThisProcess,
     selection: &BackgroundGpuSelection,
     memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
+    openings_not_returned_in_this_process: &OpeningsNotReturnedInThisProcess,
     report_the_gpus_to_open: &dyn Fn(usize),
 ) -> Result<OpenedBackgroundGpu, String> {
     open_background_gpu_with_the_memory_leaked_in_this_process(
         selection,
         memory_leaked_in_this_process,
+        openings_not_returned_in_this_process,
         vulkan_instance,
         report_the_gpus_to_open,
     )
     .map(background_gpu_compute_of_the_opened_gpu)
-    .map(|opened| OpenedBackgroundGpu {
+    .map(opened_background_gpu_for_the_mount_runtime)
+    .map_err(|refused| format!("{refused:?}"))
+}
+
+/// 开成的后台 GPU 换成运行库收的样子：核心层收的把手、开卡的报告（开着的、因显存上限不用的、没收进清单的卡），与读这一次挂载泄漏了哪几张卡的那一份
+/// （切换关卡时累计的、卸载时丢掉的，[`GpusLeakedByTheOpenedBackgroundGpu`]）。
+#[must_use]
+pub fn opened_background_gpu_for_the_mount_runtime(
+    opened: OpenedBackgroundGpuCompute,
+) -> OpenedBackgroundGpu {
+    OpenedBackgroundGpu {
         report_of_the_opening: format!(
             "gpus_opened={:?} gpus_left_out={:?} unnamed_gpus_left_off={:?}",
             opened.gpus_opened_at_the_mount,
             opened.gpus_left_out_under_their_memory_limit,
             opened.unnamed_discrete_gpus_left_off_the_list
         ),
+        gpus_leaked: Some(Arc::new(GpusLeakedByTheOpenedBackgroundGpu {
+            when_closed_at_a_switch: opened.gpus_leaked_when_closed_at_a_switch,
+            when_dropped: opened.gpus_leaked_when_dropped,
+        })),
         gpu_compute: opened.gpu_compute,
-    })
-    .map_err(|refused| format!("{refused:?}"))
+    }
+}
+
+/// 读产品开卡器开成的后台 GPU 泄漏了哪几张卡（运行库的 [`ReadsTheGpusLeakedByTheBackgroundGpu`]）：切换关卡时累计的那一份与卸载时丢掉的那一份，
+/// 各带泄漏那一刻本包在那张卡上占着的显存。
+#[derive(Debug)]
+struct GpusLeakedByTheOpenedBackgroundGpu {
+    when_closed_at_a_switch: GpusLeakedWhenClosedAtASwitch,
+    when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped,
+}
+
+/// 本包记的一张泄漏的卡换成运行库报的样子。
+fn gpu_leaked_for_the_mount_runtime(leaked: GpuLeakedWhenDropped) -> GpuLeakedByTheBackgroundGpu {
+    GpuLeakedByTheBackgroundGpu {
+        device: leaked.device,
+        memory_held_on_the_device: leaked.bytes_held_on_the_device,
+    }
+}
+
+impl ReadsTheGpusLeakedByTheBackgroundGpu for GpusLeakedByTheOpenedBackgroundGpu {
+    fn gpus_leaked_when_closed_at_a_switch(&self) -> Vec<GpuLeakedByTheBackgroundGpu> {
+        self.when_closed_at_a_switch
+            .gpus_leaked()
+            .into_iter()
+            .map(gpu_leaked_for_the_mount_runtime)
+            .collect()
+    }
+
+    fn gpus_leaked_when_dropped(&self) -> Option<Vec<GpuLeakedByTheBackgroundGpu>> {
+        self.when_dropped.gpus_leaked().map(|gpus_leaked| {
+            gpus_leaked
+                .into_iter()
+                .map(gpu_leaked_for_the_mount_runtime)
+                .collect()
+        })
+    }
 }
 
 impl<Instance: Clone + Send + 'static> GpuOpenerOfThisProcess<Instance> {
@@ -204,6 +267,8 @@
                 instances_forgotten: 0,
             })),
             memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            openings_not_returned_in_this_process:
+                OpeningsNotReturnedInThisProcess::with_none_not_returned(),
             make_an_instance,
             open_on_the_worker_thread,
         }
@@ -218,6 +283,12 @@
             .instances_created
     }
 
+    /// 这个进程里哪几张卡上有一次到点之后还没回来的开卡：交给每一次开卡的都是这一张（[`OpeningsNotReturnedInThisProcess`]）。
+    #[must_use]
+    pub fn openings_not_returned_in_this_process(&self) -> OpeningsNotReturnedInThisProcess {
+        self.openings_not_returned_in_this_process.clone()
+    }
+
     /// 上一次开卡的工作线程还在驱动里没回来。
     #[must_use]
     pub fn is_an_opening_hung(&self) -> bool {
@@ -290,6 +361,8 @@
         let make_an_instance = Arc::clone(&self.make_an_instance);
         let open_on_the_worker_thread = Arc::clone(&self.open_on_the_worker_thread);
         let memory_leaked_in_this_process = self.memory_leaked_in_this_process.clone();
+        let openings_not_returned_in_this_process =
+            self.openings_not_returned_in_this_process.clone();
         let selection = BackgroundGpuSelection::of_the_settings(settings);
         let started = std::thread::Builder::new()
             .name("singlefs-gpu-opener".to_string())
@@ -311,6 +384,7 @@
                         &instance,
                         &selection,
                         &memory_leaked_in_this_process,
+                        &openings_not_returned_in_this_process,
                         &report_the_gpus_to_open,
                     )
                     .map_err(|description| {
````

## crates/singlefs-background-gpu/src/gpus_of_this_host.rs

````diff
--- a/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
+++ b/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
@@ -1,28 +1,27 @@
 //! 开着的本机独显里，此刻照哪一组设置用哪几张、每张怎么切、等多久（D26（后台整理与放置回收） 已定项 13 第 1–3 条）。
-//! 只开参与的卡（欠账 C996 的定案）：挂载那一刻只在参与的卡上起设备、量代价（给了 `background_gpu_devices` 是点名的那几张，没给是本机全部独显），
+//! 只开参与的卡（第 2 条「挂载时只开参与的卡」）：挂载那一刻只在参与的卡上起设备、量代价（给了 `background_gpu_devices` 是点名的那几张，没给是本机全部独显），
 //! 没点名的卡不开、不量，卡清单照列它（[`DiscreteGpusAfterTheOpening::not_opened_for_not_taking_part`]）；挂着切换点到它，外壳在会话锁外开它、开成了再换上那一组
-//! （外壳 `crate::background_gpu::BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`）。换上一组时不再点名的开着的卡关掉（用户定案「切换时关掉」）：
+//! （外壳 `crate::background_gpu::BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`）。换上一组时不再点名的开着的卡关掉（第 2 条「切换之后不再参与的卡，切换生效时关掉」）：
 //! 挪回没开的那几张（[`DiscreteGpusAfterTheOpening::note_the_gpu_closed_at_a_switch`]），之后再点它就重开；上面还有等超过上限、没在 GPU 上做完的提交的那张
 //! 不等它、整张泄漏，同卸载（[`gpus_dropped_leaking_those_still_running`]）。
 //! 纯逻辑，不碰 GPU：外壳每张开着的卡一份 [`GpuBudgetUnderTheSettings`]，参与的卡由 [`ParticipatingGpusOnThisHost`] 定；
 //! 核心层交来一组新设置（`BackgroundGpuCompute::use_the_settings_from_the_next_batch`，只在两批之间）就换这两样，从下一批起照它分活、切段、等。
 //!
-//! 挂载那一刻参与的卡开不成就隔一段时间重开，重开到上限都没开成才拒绝挂载（第 4 条与用户定案「挂载那一刻点名的卡开不成：也照这样重开 3 次」）：
+//! 挂载那一刻参与的卡开不成就隔一段时间重开，重开到上限都没开成才拒绝挂载（第 4 条「挂载那一刻参与的卡开不成……间隔 1 分钟重开，最多重开 3 次」）：
 //! [`gpus_opened_at_the_mount`]。
 //!
-//! 一批的活按卡的次序填满（用户 2026-11-02 定「按卡的顺序填满」）：参与的卡排成参与的次序，前面的卡先填到它照显存上限切片之后一次能放下的量，
+//! 一批的活按卡的次序填满（D26（后台整理与放置回收） 已定项 13 第 2 条「按卡的顺序填满」）：参与的卡排成参与的次序，前面的卡先填到它照显存上限切片之后一次能放下的量，
 //! 装不下的才给后面的卡，排在最后的那张拿剩下的全部（[`plans_of_a_batch_filling_the_gpus_in_order`]）；挂载时量的代价照量、不用来分活。
 //!
 //! 换设置不清账：一张卡上等超过上限、还在 GPU 上跑的那次提交照旧记着，下一批在这张卡上交活之前先在新的等的上限之内等它做完，
 //! 再按新的显存上限扣掉还占着的切（第 3 条）；连续不确定的计数不在这里，换设置不碰它（第 8 条）。
 //!
-//! 显存上限连最小的一片都装不下的卡不用（用户 2026-10-28 定「连一片都装不下就不用这张卡」，原话在 `records/2026-10-19-里程碑四GPU垃圾回收开工.md`
-//! 末尾「用户定案（弹窗两次）」那一行）：挂载那一刻与每次换设置都照 [`bytes_of_the_smallest_piece_every_participating_gpu_holds`] 判，
+//! 显存上限连最小的一片都装不下的卡不用（第 3 条「参与的卡的上限连最小的一片都装不下，这张卡就不用」）：挂载那一刻与每次换设置都照 [`bytes_of_the_smallest_piece_every_participating_gpu_holds`] 判，
 //! 装不下的卡不参与、报出来（[`GpuLeftOutUnderItsMemoryLimit`]，核心层定的那一份），挂载不因此拒绝；参与的卡因此一张都不剩时，三段每一批都交回
 //! 「参与的卡一张都不剩」，核心层挂载那一刻就读到、当场报警停着（`singlefs_core::background_compute::BackgroundGpuCompute::failure_under_the_settings_handed_in`）。量代价那一段按开卡时的上限切成几片逐片交，代价取各片之和
 //! （[`measured_cost_summed_over_the_pieces`]）。
 //!
-//! 丢后台 GPU 时不等还在 GPU 上跑的提交（同一定案「卸载时同样不等、泄漏设备」）：[`gpus_dropped_leaking_those_still_running`]。
+//! 丢后台 GPU 时不等还在 GPU 上跑的提交（第 4 条「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构」）：[`gpus_dropped_leaking_those_still_running`]。
 
 use std::ops::Range;
 use std::sync::{Arc, Mutex};
@@ -34,14 +33,16 @@
 
 use crate::adapter_inventory::{DiscreteGpuOnThisHost, ParticipatingGpu};
 use crate::background_gpu::{
-    BackgroundGpuOpenRefused, GpuFailureCause, SmallestPieceNotHeldOnTheGpu,
+    opening_with_its_panic_caught, BackgroundGpuOpenRefused, GpuFailureCause,
+    SmallestPieceNotHeldOnTheGpu,
 };
 use crate::checksum_kernels::bytes_of_the_smallest_checksum_piece;
 use crate::live_slot_kernels::bytes_of_the_smallest_live_slot_piece;
 use crate::measured_host_cpu_time_per_submission::HostCpuNanosecondsPerSubmission;
 use crate::memory_limit_and_chunks::{
     pieces_cut_to_fit_on_the_gpu, ChunkBoundsOfAGpu, ChunkFootprint, ChunksNotCutOnTheGpu,
-    GpuMemoryAccount, PieceOfAnEntry, SmallestPieceLargerThanTheRoom,
+    EntryCountOfAChunk, GpuMemoryAccount, InputLengthOnTheHostInBytes, PieceOfAnEntry,
+    SmallestPieceLargerThanTheRoom,
 };
 use crate::region_record_kernels::bytes_of_the_smallest_region_record_piece;
 use crate::waiting_for_the_gpu::{
@@ -52,20 +53,20 @@
 /// 一张卡此刻照的设置管的那几样：显存上限（记在账上）、等 GPU 的上限，连同与设置无关的绑定上限与派发上限、本包在它上面的账。
 #[derive(Debug)]
 pub struct GpuBudgetUnderTheSettings<Submission> {
-    largest_storage_binding_in_bytes: u64,
-    largest_entry_count_per_dispatch: u64,
+    largest_storage_binding_in_bytes: GpuMemoryInBytes,
+    largest_entry_count_per_dispatch: EntryCountOfAChunk,
     memory_account: GpuMemoryAccount<Submission>,
     wait_limit: Duration,
-    /// 开卡那一刻这个进程里此前几次挂载泄漏在这张卡上的字节（账从它起）：丢的时候扣掉它，算这一次挂载自己泄漏的（[`GpuLeakedWhenDropped`]）。
-    bytes_leaked_by_earlier_mounts: u64,
+    /// 开卡那一刻这个进程里更早几次挂载泄漏在这张卡上的字节（账从它起）：丢的时候扣掉它，算这一次挂载自己泄漏的（[`GpuLeakedWhenDropped`]）。
+    bytes_leaked_by_earlier_mounts: GpuMemoryInBytes,
 }
 
 impl<Submission> GpuBudgetUnderTheSettings<Submission> {
     /// 开卡那一刻：一个字节都没占，照开卡时给的上限。
     #[must_use]
     pub fn with_nothing_held(
-        largest_storage_binding_in_bytes: u64,
-        largest_entry_count_per_dispatch: u64,
+        largest_storage_binding_in_bytes: GpuMemoryInBytes,
+        largest_entry_count_per_dispatch: EntryCountOfAChunk,
         memory_limit: Option<GpuMemoryInBytes>,
         wait_limit: Duration,
     ) -> Self {
@@ -74,19 +75,19 @@
             largest_entry_count_per_dispatch,
             memory_limit,
             wait_limit,
-            0,
+            GpuMemoryInBytes(0),
         )
     }
 
-    /// 开卡那一刻，这个进程里此前几次挂载在这张卡上泄漏了 `bytes_leaked_by_earlier_mounts` 字节（[`GpuMemoryLeakedInThisProcess`]）：
+    /// 开卡那一刻，这个进程里更早几次挂载在这张卡上泄漏了 `bytes_leaked_by_earlier_mounts` 字节（[`GpuMemoryLeakedInThisProcess`]）：
     /// 账从它起、一直记着，这张卡的可用余量先扣掉它（第 3 条：泄漏的那一片仍占着这张卡）。
     #[must_use]
     pub fn with_the_bytes_leaked_by_earlier_mounts(
-        largest_storage_binding_in_bytes: u64,
-        largest_entry_count_per_dispatch: u64,
+        largest_storage_binding_in_bytes: GpuMemoryInBytes,
+        largest_entry_count_per_dispatch: EntryCountOfAChunk,
         memory_limit: Option<GpuMemoryInBytes>,
         wait_limit: Duration,
-        bytes_leaked_by_earlier_mounts: u64,
+        bytes_leaked_by_earlier_mounts: GpuMemoryInBytes,
     ) -> Self {
         Self {
             largest_storage_binding_in_bytes,
@@ -133,9 +134,9 @@
         self.wait_limit
     }
 
-    /// 开卡那一刻已经算上的、这个进程里此前几次挂载泄漏在这张卡上的字节。
+    /// 开卡那一刻已经算上的、这个进程里更早几次挂载泄漏在这张卡上的字节。
     #[must_use]
-    pub fn bytes_leaked_by_earlier_mounts(&self) -> u64 {
+    pub fn bytes_leaked_by_earlier_mounts(&self) -> GpuMemoryInBytes {
         self.bytes_leaked_by_earlier_mounts
     }
 
@@ -196,7 +197,7 @@
         &mut self,
         clock: &impl MonotonicClock,
         check_once_waiting_at_most: impl FnMut(&Submission, Duration) -> CompletionCheck<()>,
-        entry_input_lengths_in_bytes: &[u64],
+        entry_input_lengths_in_bytes: &[InputLengthOnTheHostInBytes],
         footprint: ChunkFootprint,
     ) -> Result<Vec<Range<usize>>, ChunksNotCutOnTheGpu> {
         let deadline = GpuWaitDeadline::starting_now(self.wait_limit, clock);
@@ -295,7 +296,7 @@
         }
     }
 
-    /// 为切换开成了一张原先没开的卡：它从没开的那几张挪进开成的那几张（排在开成的那几张末尾，与外壳开着的卡同序）。
+    /// 为切换开成了一张没开着的卡：它从没开的那几张挪进开成的那几张（排在开成的那几张末尾，与外壳开着的卡同序）。
     ///
     /// # Panics
     /// 它不在没开的那几张里：外壳只开没开的卡，开着的时候它一直在没开的那几张里（开卡至多一次在开，关卡只关开着的）。
@@ -357,15 +358,16 @@
 /// 挂载那一刻量代价交的那一段（外壳的那一段是 `crate::background_gpu::COST_PROBE_CHUNK`）：一条 `input_bytes` 字节的单元，`footprint` 是它那一段在卡上的占用。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct CostProbeChunk {
-    pub input_bytes: u64,
+    pub input_bytes: InputLengthOnTheHostInBytes,
     pub footprint: ChunkFootprint,
 }
 
 impl CostProbeChunk {
     /// 整条一段在卡上合占的字节数。
     #[must_use]
-    pub fn bytes_held(&self) -> u64 {
-        self.footprint.bytes_held_by_a_chunk(self.input_bytes, 1)
+    pub fn bytes_held(&self) -> GpuMemoryInBytes {
+        self.footprint
+            .bytes_held_by_a_chunk(self.input_bytes, EntryCountOfAChunk(1))
     }
 
     /// 照这张卡此刻的界切这一条：装得下就一段一片，装不下就切成几片、几段（切法与挂着时校验和那一段同一个，[`pieces_cut_to_fit_on_the_gpu`]）。
@@ -419,10 +421,10 @@
 
 /// 参与的卡显存上限至少要装得下的那一片：三段各自与输入无关、定得下来的最小的一片取最大的那个——校验和那一段（量代价那一段同它）一个字节的单元，
 /// 选段打分那一段一个段连同一条记录的记录表，区域扫描那一段切开的单元头一片连同只放一条条目的映射（单元能切开之后它不再随最大那条记录变，
-/// 用户 2026-10-28 定「装不下应该自适应的分片，缩小逐次进行」，`crate::region_record_kernels::bytes_of_the_smallest_region_record_piece`）。
+/// D26（后台整理与放置回收） 已定项 13 第 3 条「按上限自适应切成几片、缩小逐次交」，`crate::region_record_kernels::bytes_of_the_smallest_region_record_piece`）。
 /// 区域扫描一个落点上不止一条条目的那一片随这一批的映射变，挂着时逐批判（[`plans_of_a_batch_filling_the_gpus_in_order`]）。
 #[must_use]
-pub fn bytes_of_the_smallest_piece_every_participating_gpu_holds() -> u64 {
+pub fn bytes_of_the_smallest_piece_every_participating_gpu_holds() -> GpuMemoryInBytes {
     bytes_of_the_smallest_checksum_piece()
         .max(bytes_of_the_smallest_live_slot_piece())
         .max(bytes_of_the_smallest_region_record_piece())
@@ -431,21 +433,21 @@
 /// 一张卡因显存上限装不下最小的一片而不用：核心层定的那一份（参与的卡一张都不剩时原样报给核心层），本包照它报。
 pub use singlefs_core::background_compute::GpuLeftOutUnderItsMemoryLimit;
 
-/// 给这张卡的显存上限扣掉卡上已经占着的 `bytes_already_held_on_the_device`（这个进程里此前几次挂载泄漏的、等超过上限还在跑的那一次提交的）
+/// 给这张卡的显存上限扣掉卡上已经占着的 `bytes_already_held_on_the_device`（这个进程里更早几次挂载泄漏的、等超过上限还在跑的那一次提交的）
 /// 装不下 `smallest_piece_bytes` 时交回它不用的那一条；不给上限、装得下都交回 `None`。
 #[must_use]
 pub fn gpu_left_out_under_its_memory_limit(
     device: &GpuDeviceIdentifier,
     memory_limit: Option<GpuMemoryInBytes>,
-    smallest_piece_bytes: u64,
-    bytes_already_held_on_the_device: u64,
+    smallest_piece_bytes: GpuMemoryInBytes,
+    bytes_already_held_on_the_device: GpuMemoryInBytes,
 ) -> Option<GpuLeftOutUnderItsMemoryLimit> {
     memory_limit
         .filter(|memory_limit| {
             memory_limit
                 .0
-                .saturating_sub(bytes_already_held_on_the_device)
-                < smallest_piece_bytes
+                .saturating_sub(bytes_already_held_on_the_device.0)
+                < smallest_piece_bytes.0
         })
         .map(|memory_limit| GpuLeftOutUnderItsMemoryLimit {
             device: device.clone(),
@@ -462,7 +464,7 @@
     pub device: GpuDeviceIdentifier,
     pub room: ChunkBoundsOfAGpu,
     pub memory_limit: Option<GpuMemoryInBytes>,
-    pub bytes_already_held_on_the_device: u64,
+    pub bytes_already_held_on_the_device: GpuMemoryInBytes,
 }
 
 /// 一张参与的卡这一批分到的活与切法：它在本机里的位置、分到的那一段相连的条目、切的时候照的界、切法。
@@ -488,7 +490,7 @@
     GpuCannotHoldTheSmallestPiece(SmallestPieceNotHeldOnTheGpu),
 }
 
-/// 一张卡从 `entries_not_handed_out` 的头一条起填得满的条数（D26（后台整理与放置回收） 已定项 13 第 2 条，用户 2026-11-02 定「按卡的顺序填满」：
+/// 一张卡从 `entries_not_handed_out` 的头一条起填得满的条数（D26（后台整理与放置回收） 已定项 13 第 2 条「按卡的顺序填满」：
 /// 「填到它的上限（照显存上限切片之后能放下的量）」的读法）：照这张卡的地方 `room` 切（`cut`），交的次数（`submissions_of_the_plan`）不比只拿头一条时多的
 /// 最多条数。头一条一次交得下的，就是一次交得下的最多条数；头一条自己就要切成几片、交几次的（单元比这张卡的地方大，或区域扫描的映射要切成几片），
 /// 是跟着它那几次交、不多添一次的最多条数。至少交回 1：头一条连最小的一片都切不出时也交回 1（调用方切它时照切不出办）。
@@ -553,7 +555,7 @@
     entry_count_held
 }
 
-/// 一批（`entry_count` 条）按卡的次序填满（D26（后台整理与放置回收） 已定项 13 第 2 条，用户 2026-11-02 定「按卡的顺序填满」）：参与的卡按参与的次序
+/// 一批（`entry_count` 条）按卡的次序填满（D26（后台整理与放置回收） 已定项 13 第 2 条「按卡的顺序填满」）：参与的卡按参与的次序
 /// （点名了是 `background_gpu_devices` 的次序，没点名是总线地址的次序，[`ParticipatingGpusOnThisHost`]），每张卡照交活之前那一刻的地方
 /// （`room_of_the_position`，先等前一次超时的那次提交）从还没分出去的头一条起拿它填得满的那么多条（[`entry_count_filling_the_gpu`]），剩下的才给后面的卡；
 /// 排在最后的那张卡拿剩下的全部、照它的地方切成几次交（活多到每张卡都填满时）。各卡照分到的那几条切好（`cut`）。活少时后面的卡一条都不分、不等、
@@ -640,7 +642,7 @@
                         room_of_the_gpu.room.memory_limit,
                     ) {
                         (Some(memory_limit), Some(room_left))
-                            if room_left.0 < smallest_piece.smallest_piece_bytes =>
+                            if room_left < smallest_piece.smallest_piece_bytes =>
                         {
                             Some(memory_limit)
                         }
@@ -679,22 +681,22 @@
 }
 
 /// 一张卡丢后台 GPU 时泄漏了（还有一次等超过上限、没在 GPU 上做完的提交，设备不析构）：哪张卡、它上面那一刻本包占着的字节（泄漏的那一片），
-/// 其中开卡那一刻已经算上的、这个进程里此前几次挂载泄漏在它上面的字节（[`GpuBudgetUnderTheSettings::bytes_leaked_by_earlier_mounts`]，
+/// 其中开卡那一刻已经算上的、这个进程里更早几次挂载泄漏在它上面的字节（[`GpuBudgetUnderTheSettings::bytes_leaked_by_earlier_mounts`]，
 /// 恒不超过占着的：账从它起、这一次挂载里不放）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct GpuLeakedWhenDropped {
     pub device: GpuDeviceIdentifier,
-    pub bytes_held_on_the_device: u64,
-    pub bytes_leaked_by_earlier_mounts: u64,
+    pub bytes_held_on_the_device: GpuMemoryInBytes,
+    pub bytes_leaked_by_earlier_mounts: GpuMemoryInBytes,
 }
 
 impl GpuLeakedWhenDropped {
-    /// 这一次挂载自己泄漏的字节：那一刻占着的扣掉开卡时已算上的此前泄漏的那一片。
+    /// 这一次挂载自己泄漏的字节：那一刻占着的扣掉开卡时已算上的更早泄漏的那一片。
     ///
     /// # Panics
-    /// 此前泄漏的比那一刻占着的多：账从此前泄漏的那一片起，这一次挂载里一直记着、不放（`GpuMemoryAccount::with_the_bytes_leaked_by_earlier_mounts`）。
+    /// 更早泄漏的比那一刻占着的多：账从更早泄漏的那一片起，这一次挂载里一直记着、不放（`GpuMemoryAccount::with_the_bytes_leaked_by_earlier_mounts`）。
     #[must_use]
-    pub fn bytes_leaked_by_this_mount(&self) -> u64 {
+    pub fn bytes_leaked_by_this_mount(&self) -> GpuMemoryInBytes {
         self.bytes_held_on_the_device
             .checked_sub(self.bytes_leaked_by_earlier_mounts)
             .expect("账从此前泄漏的那一片起、这一次挂载里不放：占着的不比它少")
@@ -771,10 +773,10 @@
 /// 这个进程里每张卡泄漏着的字节（几次挂载的后台 GPU 丢掉时泄漏的设备不析构，那一片显存这个进程活着就一直占着）：调用方在进程里留一份、
 /// 每次开卡都交同一份（`background_gpu_compute::open_background_gpu_compute`），新一次挂载这张卡的可用余量先扣掉它（第 3 条）。
 /// 同一张卡上每丢一次泄漏一次就按次累加这一次挂载自己泄漏的字节（[`GpuLeakedWhenDropped::bytes_leaked_by_this_mount`]），不覆盖：
-/// 两次挂载同时开着时各自开卡那一刻都没算上对方的那一片，后丢的那一次也不含先丢的（判决 m4-gpugc-code-r2 第三节第 4 条）。几份把手共用同一份。
+/// 两次挂载同时开着时各自开卡那一刻都没算上对方的那一片，后丢的那一次也不含先丢的。几份把手共用同一份。
 #[derive(Clone, Debug, Default)]
 pub struct GpuMemoryLeakedInThisProcess {
-    bytes_leaked_on_each_device: Arc<Mutex<Vec<(GpuDeviceIdentifier, u64)>>>,
+    bytes_leaked_on_each_device: Arc<Mutex<Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)>>>,
 }
 
 impl GpuMemoryLeakedInThisProcess {
@@ -788,16 +790,16 @@
 
     /// 这张卡上泄漏着的字节；没泄漏过是 0。
     #[must_use]
-    pub fn bytes_leaked_on(&self, device: &GpuDeviceIdentifier) -> u64 {
+    pub fn bytes_leaked_on(&self, device: &GpuDeviceIdentifier) -> GpuMemoryInBytes {
         self.bytes_leaked_on_each_device
             .lock()
             .unwrap_or_else(std::sync::PoisonError::into_inner)
             .iter()
             .find(|(leaked_device, _bytes)| leaked_device == device)
-            .map_or(0, |(_device, bytes)| *bytes)
+            .map_or(GpuMemoryInBytes(0), |(_device, bytes)| *bytes)
     }
 
-    /// 记下一次丢后台 GPU 时泄漏的卡：每张加上这一次挂载自己泄漏的字节（那一刻占着的扣掉开卡时已算上的此前泄漏的那一片）。
+    /// 记下一次丢后台 GPU 时泄漏的卡：每张加上这一次挂载自己泄漏的字节（那一刻占着的扣掉开卡时已算上的更早泄漏的那一片）。
     pub fn note_the_gpus_leaked_when_dropped(&self, gpus_leaked: &[GpuLeakedWhenDropped]) {
         let mut bytes_leaked_on_each_device = self
             .bytes_leaked_on_each_device
@@ -808,7 +810,7 @@
                 .iter_mut()
                 .find(|(leaked_device, _bytes)| *leaked_device == leaked.device)
             {
-                Some((_device, bytes)) => *bytes += leaked.bytes_leaked_by_this_mount(),
+                Some((_device, bytes)) => *bytes = bytes.plus(leaked.bytes_leaked_by_this_mount()),
                 None => bytes_leaked_on_each_device
                     .push((leaked.device.clone(), leaked.bytes_leaked_by_this_mount())),
             }
@@ -835,14 +837,15 @@
 
 /// 开一张卡（挂载那一刻开参与的卡、切换生效那一刻开点到的没开的卡，同一步）：先照给它的显存上限（扣掉这个进程里之前几次挂载泄漏在它上面的字节）
 /// 判装不装得下最小的一片（`smallest_piece_bytes`），装得下的交 `open_the_gpu` 起设备、编管线、照上限量代价；装不下的照开、不量代价，
-/// 连同报它不用的那一条交回（用户 2026-10-28 定「连一片都装不下就不用这张卡」）。
+/// 连同报它不用的那一条交回（第 3 条「参与的卡的上限连最小的一片都装不下，这张卡就不用」）。`open_the_gpu` 里 panic 了在这一步接住、
+/// 算这一次没开成（第 4 条「开卡线程 panic 都算这张卡这一次没开成，照重开」，挂载与切换同一个规矩）。
 ///
 /// # Errors
-/// `open_the_gpu` 交回的这张卡没开成的原因，原样。
+/// `open_the_gpu` 交回的这张卡没开成的原因，原样；它 panic 了是 [`GpuFailureCause::OpeningPanicked`]，带 panic 的原话。
 pub fn gpu_opened_under_its_memory_limit<Opened>(
     discrete_gpu: &DiscreteGpuOnThisHost,
     memory_limit_at_the_opening: Option<GpuMemoryInBytes>,
-    smallest_piece_bytes: u64,
+    smallest_piece_bytes: GpuMemoryInBytes,
     memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
     open_the_gpu: impl FnOnce(
         &DiscreteGpuOnThisHost,
@@ -860,19 +863,21 @@
         Some(_left_out) => CostAtTheOpening::NotMeasuredForTheGpuIsLeftOut,
         None => CostAtTheOpening::MeasuredUnderTheMemoryLimit,
     };
-    let opened = open_the_gpu(
-        discrete_gpu,
-        memory_limit_at_the_opening,
-        cost_at_the_opening,
-    )?;
+    let opened = opening_with_its_panic_caught(|| {
+        open_the_gpu(
+            discrete_gpu,
+            memory_limit_at_the_opening,
+            cost_at_the_opening,
+        )
+    })?;
     Ok((opened, left_out))
 }
 
-/// 挂载那一刻只在参与的卡上开卡（欠账 C996 的定案；按 `discrete_gpus` 的次序，即总线地址的次序）：`participating` 里有它（给了 `background_gpu_devices` 时是点名的卡，
+/// 挂载那一刻只在参与的卡上开卡（第 2 条「挂载时只开参与的卡」；按 `discrete_gpus` 的次序，即总线地址的次序）：`participating` 里有它（给了 `background_gpu_devices` 时是点名的卡，
 /// 没给时本机全部独显都参与）就照开卡那一组给它的显存上限开（[`gpu_opened_under_its_memory_limit`]，`open_the_gpu` 是外壳起设备、编管线、量代价那一步，
 /// 测试里交造出来的结局）；不参与的不开、不量代价，记进 [`DiscreteGpusAfterTheOpening::not_opened_for_not_taking_part`]，卡清单照列它，挂着切换点到它时再开。
 /// 显存上限装不下最小的一片的参与的卡照开、不量代价、不进参与的卡、报出来（[`GpuLeftOutUnderItsMemoryLimit`]），挂载不因此拒绝。
-/// 一张参与的卡开不成就照 `reopenings` 重开（用户定案「挂载那一刻点名的卡开不成：也照这样重开 3 次，都不成才拒绝挂载」，每次重开之前把间隔交给
+/// 一张参与的卡开不成就照 `reopenings` 重开（第 4 条「挂载那一刻参与的卡重开 3 次都不成，才拒绝挂载」，开卡里 panic 了同样算一次没开成，每次重开之前把间隔交给
 /// `wait_before_a_reopening` 等，[`gpu_opened_with_the_reopenings`]）；重开到上限都没开成就拒绝挂载、报出它与每一次的原因，后面的卡不再开。
 /// `participating` 是开卡那一组照同一份 `discrete_gpus` 校验过、取出来的参与的卡（`adapter_inventory::participating_gpus_admitted_by_the_mount_checks`）。
 ///
@@ -884,7 +889,7 @@
 pub fn gpus_opened_at_the_mount<Opened>(
     discrete_gpus: Vec<DiscreteGpuOnThisHost>,
     participating: &[ParticipatingGpu],
-    smallest_piece_bytes: u64,
+    smallest_piece_bytes: GpuMemoryInBytes,
     memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
     reopenings: ReopeningsOfAGpuNotOpened,
     mut wait_before_a_reopening: impl FnMut(Duration),
@@ -972,7 +977,7 @@
 
 /// 丢后台 GPU 时（卸载、丢掉核心层收的把手）逐张丢开着的卡：还有一次等超过上限、没在 GPU 上做完的提交的那几张不丢、泄漏
 /// （它的设备与队列不析构：wgpu-core 30.0.1 的 `Queue` 析构时调 `wait_for_idle`，即 Vulkan 的 `vkQueueWaitIdle`，不带上限），
-/// 别的照常丢（用户 2026-10-28 定「卸载时同样不等、泄漏设备」）。交回泄漏的那几张（按交进来的次序，各带它上面那一刻占着的字节，
+/// 别的照常丢（D26（后台整理与放置回收） 已定项 13 第 4 条「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构」）。交回泄漏的那几张（按交进来的次序，各带它上面那一刻占着的字节，
 /// `leaked_when_dropped` 报）。不等任何一次提交：调用即返回。
 pub fn gpus_dropped_leaking_those_still_running<Gpu>(
     gpus: Vec<Gpu>,
@@ -1102,16 +1107,16 @@
     #[test]
     fn after_a_switch_to_a_smaller_memory_limit_the_next_batch_is_cut_by_the_smaller_limit() {
         let footprint = ChunkFootprint {
-            bytes_per_chunk_besides_the_entries: 16,
-            bytes_per_entry_besides_its_input: 40,
-            bytes_per_entry_in_the_largest_entry_buffer: 16,
+            bytes_per_chunk_besides_the_entries: GpuMemoryInBytes(16),
+            bytes_per_entry_besides_its_input: GpuMemoryInBytes(40),
+            bytes_per_entry_in_the_largest_entry_buffer: GpuMemoryInBytes(16),
         };
-        let lengths = [50_u64; 6];
+        let lengths = [InputLengthOnTheHostInBytes(50); 6];
         let clock = ClockThatNeverMoves;
         let mut budget: GpuBudgetUnderTheSettings<u32> =
             GpuBudgetUnderTheSettings::with_nothing_held(
-                1 << 30,
-                1 << 20,
+                GpuMemoryInBytes(1 << 30),
+                EntryCountOfAChunk(1 << 20),
                 Some(GpuMemoryInBytes(1000)),
                 Duration::from_secs(10),
             );
@@ -1146,7 +1151,7 @@
         );
         let held_past_the_smaller_limit =
             std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
-                budget.memory_account_mut().hold(301);
+                budget.memory_account_mut().hold(GpuMemoryInBytes(301));
             }));
         assert!(
             held_past_the_smaller_limit.is_err(),
@@ -1159,9 +1164,9 @@
     #[test]
     fn after_a_switch_of_the_wait_limit_the_next_batch_waits_by_the_new_limit() {
         let footprint = ChunkFootprint {
-            bytes_per_chunk_besides_the_entries: 16,
-            bytes_per_entry_besides_its_input: 40,
-            bytes_per_entry_in_the_largest_entry_buffer: 16,
+            bytes_per_chunk_besides_the_entries: GpuMemoryInBytes(16),
+            bytes_per_entry_besides_its_input: GpuMemoryInBytes(40),
+            bytes_per_entry_in_the_largest_entry_buffer: GpuMemoryInBytes(16),
         };
         let (mut budget_kept_at_the_old_limit, clock_of_the_old_limit) =
             budget_with_a_timed_out_submission_completing_after(
@@ -1174,7 +1179,7 @@
                     &clock_of_the_old_limit,
                     |_submission, time_left| clock_of_the_old_limit
                         .wait_for_the_submission(time_left),
-                    &[100],
+                    &[InputLengthOnTheHostInBytes(100)],
                     footprint,
                 ),
             Err(ChunksNotCutOnTheGpu::EarlierSubmissionStillRunning(
@@ -1201,7 +1206,7 @@
                     &clock_of_the_new_limit,
                     |_submission, time_left| clock_of_the_new_limit
                         .wait_for_the_submission(time_left),
-                    &[100],
+                    &[InputLengthOnTheHostInBytes(100)],
                     footprint,
                 ),
             Ok(vec![Range { start: 0, end: 1 }]),
@@ -1211,7 +1216,7 @@
             budget_switched_to_the_new_limit
                 .memory_account()
                 .bytes_held(),
-            0,
+            GpuMemoryInBytes(0),
             "那次提交的账放了"
         );
     }
@@ -1259,13 +1264,17 @@
         GpuBudgetUnderTheSettings<u32>,
         ClockWithASubmissionCompletingAt,
     ) {
-        let mut budget =
-            GpuBudgetUnderTheSettings::with_nothing_held(1 << 30, 1 << 20, None, wait_limit);
-        budget.memory_account_mut().hold(200);
+        let mut budget = GpuBudgetUnderTheSettings::with_nothing_held(
+            GpuMemoryInBytes(1 << 30),
+            EntryCountOfAChunk(1 << 20),
+            None,
+            wait_limit,
+        );
+        budget.memory_account_mut().hold(GpuMemoryInBytes(200));
         budget
             .memory_account_mut()
             .settle_the_chunk_after_its_submission(
-                200,
+                GpuMemoryInBytes(200),
                 SubmissionOfTheChunk::MayStillBeRunningOnTheGpu(7),
             );
         (
@@ -1327,7 +1336,7 @@
         )
     }
 
-    /// 欠账 C996 的定案「只开参与的卡」：点名一张时，挂载那一刻 `open_the_gpu` 只被调一次、调的是点名的那张；
+    /// 第 2 条「挂载时只开参与的卡」：点名一张时，挂载那一刻 `open_the_gpu` 只被调一次、调的是点名的那张；
     /// 没点名的两张不开、不量代价，卡清单照列它们（按总线地址排），不报没开成。
     #[test]
     fn naming_one_card_opens_only_the_named_card_at_the_mount() {
@@ -1460,7 +1469,7 @@
         );
     }
 
-    /// 用户定案「挂载那一刻点名的卡开不成：也照这样重开」：点名的「0000:41:00.0」头两次开不成、第三次开成——挂载照常，它参与；
+    /// 第 4 条「挂载那一刻参与的卡开不成……间隔 1 分钟重开」：点名的「0000:41:00.0」头两次开不成、第三次开成——挂载照常，它参与；
     /// 两次重开之前各把间隔交去等一次（产品是 1 分钟，用例给 9 秒、不真等），开卡一共只开它 3 次、没点名的不开。
     #[test]
     fn the_named_card_not_opened_twice_at_the_mount_opens_the_third_time_and_takes_part() {
@@ -1541,16 +1550,16 @@
                     let chunks = COST_PROBE_CHUNK
                         .pieces_under_the_bounds(ChunkBoundsOfAGpu {
                             memory_limit,
-                            largest_storage_binding_in_bytes: 1 << 30,
-                            largest_entry_count_per_dispatch: 1 << 20,
+                            largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
+                            largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
                         })
                         .expect("量代价的卡装得下最小的一片");
                     for chunk in &chunks {
-                        let input_bytes: u64 =
+                        let input_bytes: InputLengthOnTheHostInBytes =
                             chunk.iter().map(PieceOfAnEntry::input_byte_count).sum();
                         let held = COST_PROBE_CHUNK.footprint.bytes_held_by_a_chunk(
                             input_bytes,
-                            u64::try_from(chunk.len()).expect("小数"),
+                            EntryCountOfAChunk::of_this_many_entries(chunk.len()),
                         );
                         account.hold(held);
                         account.release(held);
@@ -1587,8 +1596,8 @@
         let chunks = COST_PROBE_CHUNK
             .pieces_under_the_bounds(ChunkBoundsOfAGpu {
                 memory_limit: Some(GpuMemoryInBytes(2000)),
-                largest_storage_binding_in_bytes: 1 << 30,
-                largest_entry_count_per_dispatch: 1 << 20,
+                largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
+                largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
             })
             .expect("2000 字节装得下最小的一片");
         assert_eq!(
@@ -1596,7 +1605,7 @@
                 .iter()
                 .map(|chunk| chunk
                     .iter()
-                    .map(|piece| piece.input_bytes.clone())
+                    .map(|piece| piece.input_bytes.start.0..piece.input_bytes.end.0)
                     .collect::<Vec<_>>())
                 .collect::<Vec<_>>(),
             vec![
@@ -1610,7 +1619,7 @@
         );
     }
 
-    /// 用户 2026-10-28 定「代价取各片之和」：量代价那一段切成几段时，每一轮把各段的 CPU 时间加起来，代价是各轮之和的中位数；
+    /// D26（后台整理与放置回收） 已定项 13 第 3 条「代价取各片之和」：量代价那一段切成几段时，每一轮把各段的 CPU 时间加起来，代价是各轮之和的中位数；
     /// 头一遍各段空跑、不算进去。
     #[test]
     fn the_cost_of_a_cost_probe_cut_into_pieces_is_the_sum_over_the_pieces() {
@@ -1628,11 +1637,11 @@
         assert_eq!(rounds_seen.get(), 6 * 5, "空跑一遍、再量五轮，每轮五段");
     }
 
-    /// 用户 2026-10-28 定「连一片都装不下就不用这张卡」：一张参与的卡的显存上限连最小的一片都装不下——照开、不量代价，不参与、报出那张卡、
+    /// D26（后台整理与放置回收） 已定项 13 第 3 条「参与的卡的上限连最小的一片都装不下，这张卡就不用」：一张参与的卡的显存上限连最小的一片都装不下——照开、不量代价，不参与、报出那张卡、
     /// 它的上限与至少要多少字节，另一张参与的卡照常；挂载不拒、不 panic。
     #[test]
     fn the_memory_limit_below_the_smallest_piece_leaves_the_gpu_out_without_refusing_the_mount() {
-        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
+        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds().0;
         let opening = opening_that_holds_the_cost_probe_in_pieces([smallest_piece_bytes - 1, 2000])
             .expect("一张卡的上限装不下最小的一片：不用它，不拒挂载");
         assert_eq!(
@@ -1640,8 +1649,8 @@
             vec![GpuLeftOutUnderItsMemoryLimit {
                 device: card("0000:01:00.0"),
                 memory_limit: GpuMemoryInBytes(smallest_piece_bytes - 1),
-                smallest_piece_bytes,
-                bytes_already_held_on_the_device: 0,
+                smallest_piece_bytes: GpuMemoryInBytes(smallest_piece_bytes),
+                bytes_already_held_on_the_device: GpuMemoryInBytes(0),
             }],
             "报出不用的那张卡、它的上限与至少要多少"
         );
@@ -1679,19 +1688,19 @@
     fn the_smallest_piece_every_participating_gpu_holds_is_the_largest_of_the_three_segments() {
         assert_eq!(
             bytes_of_the_smallest_checksum_piece(),
-            2 * (4 + 12 + 2 * 8 + 16) + 2 * 8
+            GpuMemoryInBytes(2 * (4 + 12 + 2 * 8 + 16) + 2 * 8)
         );
         assert_eq!(
             bytes_of_the_smallest_live_slot_piece(),
-            2 * (16 + 2 * 12 + 16 + 20 + 4) + 2 * 12
+            GpuMemoryInBytes(2 * (16 + 2 * 12 + 16 + 20 + 4) + 2 * 12)
         );
         assert_eq!(
             bytes_of_the_smallest_region_record_piece(),
-            2 * (628 + 44 + 2 * 52 + 16 + 56 + 16 + 4) + 2 * 52
+            GpuMemoryInBytes(2 * (628 + 44 + 2 * 52 + 16 + 56 + 16 + 4) + 2 * 52)
         );
         assert_eq!(
             bytes_of_the_smallest_piece_every_participating_gpu_holds(),
-            1840
+            GpuMemoryInBytes(1840)
         );
     }
 
@@ -1713,7 +1722,7 @@
         }
     }
 
-    /// 用户 2026-10-28 定「卸载时同样不等、泄漏设备」：丢后台 GPU 时还有没做完的提交的那张卡不丢（泄漏），别的照常丢，调用即返回。
+    /// D26（后台整理与放置回收） 已定项 13 第 4 条「卸载时不等还在跑的提交」：丢后台 GPU 时还有没做完的提交的那张卡不丢（泄漏），别的照常丢，调用即返回。
     #[test]
     fn dropping_the_gpus_leaks_the_one_with_a_running_submission_and_drops_the_others() {
         let dropped = std::cell::RefCell::new(Vec::new());
@@ -1739,16 +1748,16 @@
             |gpu| gpu.waits_for_its_queue,
             |gpu| GpuLeakedWhenDropped {
                 device: card(gpu.name),
-                bytes_held_on_the_device: 600,
-                bytes_leaked_by_earlier_mounts: 0,
+                bytes_held_on_the_device: GpuMemoryInBytes(600),
+                bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
             },
         );
         assert_eq!(
             leaked,
             vec![GpuLeakedWhenDropped {
                 device: card("还在跑的卡"),
-                bytes_held_on_the_device: 600,
-                bytes_leaked_by_earlier_mounts: 0,
+                bytes_held_on_the_device: GpuMemoryInBytes(600),
+                bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
             }],
             "泄漏还在跑的那一张"
         );
@@ -1761,9 +1770,9 @@
 
     /// 测试自己定的占用：每段另占 16 字节、每条另占 40 字节、按条排的最大缓冲每条 16 字节；最小的一片（输入连同暂存两份）2 × 4 + 40 + 16 = 64 字节。
     const FOOTPRINT_OF_THE_PLANNING_CASES: ChunkFootprint = ChunkFootprint {
-        bytes_per_chunk_besides_the_entries: 16,
-        bytes_per_entry_besides_its_input: 40,
-        bytes_per_entry_in_the_largest_entry_buffer: 16,
+        bytes_per_chunk_besides_the_entries: GpuMemoryInBytes(16),
+        bytes_per_entry_besides_its_input: GpuMemoryInBytes(40),
+        bytes_per_entry_in_the_largest_entry_buffer: GpuMemoryInBytes(16),
     };
 
     /// 切法是 `pieces_cut_to_fit_on_the_gpu` 那一种（几段，每段几片）的一批的结局。
@@ -1791,16 +1800,20 @@
                     device: card(ADDRESSES_OF_THE_PLANNING_CASES[position]),
                     room: ChunkBoundsOfAGpu {
                         memory_limit: Some(GpuMemoryInBytes(memory_limits_in_bytes[position])),
-                        largest_storage_binding_in_bytes: 1 << 30,
-                        largest_entry_count_per_dispatch: 1 << 20,
+                        largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
+                        largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
                     },
                     memory_limit: Some(GpuMemoryInBytes(memory_limits_in_bytes[position])),
-                    bytes_already_held_on_the_device: 0,
+                    bytes_already_held_on_the_device: GpuMemoryInBytes(0),
                 })
             },
             |entries, room| {
                 pieces_cut_to_fit_on_the_gpu(
-                    &lengths[entries],
+                    &lengths
+                        .iter()
+                        .copied()
+                        .map(InputLengthOnTheHostInBytes)
+                        .collect::<Vec<_>>()[entries],
                     FOOTPRINT_OF_THE_PLANNING_CASES,
                     room,
                 )
@@ -1837,7 +1850,7 @@
             .collect()
     }
 
-    /// 用户 2026-11-02 定「按卡的顺序填满」：两张卡各 1000 字节、四条各 50 字节一次交得下（输入连同暂存 400、4 × 40 + 16，合 576）——头一张全拿，
+    /// D26（后台整理与放置回收） 已定项 13 第 2 条「按卡的顺序填满」：两张卡各 1000 字节、四条各 50 字节一次交得下（输入连同暂存 400、4 × 40 + 16，合 576）——头一张全拿，
     /// 第二张一条都不分、不问它的地方（活少时只有头一张卡分到活，不要求每张卡都有活）。
     #[test]
     fn the_batch_that_one_gpu_holds_goes_wholly_to_the_first_gpu_in_the_order() {
@@ -1903,8 +1916,8 @@
         assert_eq!(rooms_asked, [1, 1, 0]);
     }
 
-    /// 判决 m4-gpugc-code-r1 第三节第 4 条：两张参与的卡，一张这一批连最小的一片（64 字节）都切不出（上限 50）——这一批不分给它，四条全分给另一张、
-    /// 照它的上限切，照常交回，不停整理（攻方探针 T3-1 那一种，改前这一批交回切不出、整理停下）。
+    /// 两张参与的卡，一张这一批连最小的一片（64 字节）都切不出（上限 50）——这一批不分给它，四条全分给另一张、
+    /// 照它的上限切，照常交回，不停整理。
     #[test]
     fn one_gpu_short_of_the_smallest_piece_of_the_batch_is_left_out_and_the_other_gpu_takes_the_whole_batch(
     ) {
@@ -1934,42 +1947,42 @@
                     GpuLeftOutUnderItsMemoryLimit {
                         device: card("0000:01:00.0"),
                         memory_limit: GpuMemoryInBytes(50),
-                        smallest_piece_bytes: 64,
-                        bytes_already_held_on_the_device: 0,
+                        smallest_piece_bytes: GpuMemoryInBytes(64),
+                        bytes_already_held_on_the_device: GpuMemoryInBytes(0),
                     },
                     GpuLeftOutUnderItsMemoryLimit {
                         device: card("0000:41:00.0"),
                         memory_limit: GpuMemoryInBytes(63),
-                        smallest_piece_bytes: 64,
-                        bytes_already_held_on_the_device: 0,
+                        smallest_piece_bytes: GpuMemoryInBytes(64),
+                        bytes_already_held_on_the_device: GpuMemoryInBytes(0),
                     },
                 ]
             ))
         );
     }
 
-    /// 判决第三节第 6 条：一次丢后台 GPU 时泄漏了一张卡 600 字节，进程里记下；同一张卡新一次挂载、上限 1000 字节，账从 600 起、
+    /// 一次丢后台 GPU 时泄漏了一张卡 600 字节，进程里记下；同一张卡新一次挂载、上限 1000 字节，账从 600 起、
     /// 可用余量只剩 400：五条各 50 字节切成两条一段（连同暂存 296 字节）的三段，与只剩 400 字节时相同。同一张卡这一次挂载再泄漏（那一刻占着 900，其中 600 是开卡时已算上的），加上这一次自己的 300，记 900、不重算已含的那一片。
     #[test]
     fn memory_leaked_by_an_earlier_mount_is_taken_off_the_room_of_the_next_mount_on_that_gpu() {
         let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
         memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
             device: card("0000:01:00.0"),
-            bytes_held_on_the_device: 600,
-            bytes_leaked_by_earlier_mounts: 0,
+            bytes_held_on_the_device: GpuMemoryInBytes(600),
+            bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
         }]);
         assert_eq!(
             (
                 memory_leaked.bytes_leaked_on(&card("0000:01:00.0")),
                 memory_leaked.bytes_leaked_on(&card("0000:41:00.0"))
             ),
-            (600, 0),
+            (GpuMemoryInBytes(600), GpuMemoryInBytes(0)),
             "只记泄漏的那一张"
         );
         let mut budget: GpuBudgetUnderTheSettings<u32> =
             GpuBudgetUnderTheSettings::with_the_bytes_leaked_by_earlier_mounts(
-                1 << 30,
-                1 << 20,
+                GpuMemoryInBytes(1 << 30),
+                EntryCountOfAChunk(1 << 20),
                 Some(GpuMemoryInBytes(1000)),
                 Duration::from_secs(10),
                 memory_leaked.bytes_leaked_on(&card("0000:01:00.0")),
@@ -1983,7 +1996,7 @@
             budget.chunks_cut_once_the_submission_running_past_its_wait_completed(
                 &ClockThatNeverMoves,
                 |_submission, _time_left| panic!("没有还在跑的提交，不该等"),
-                &[50, 50, 50, 50, 50],
+                &[InputLengthOnTheHostInBytes(50); 5],
                 FOOTPRINT_OF_THE_PLANNING_CASES,
             ),
             Ok(vec![0..2, 2..4, 4..5]),
@@ -1991,13 +2004,16 @@
         );
         memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
             device: card("0000:01:00.0"),
-            bytes_held_on_the_device: 900,
-            bytes_leaked_by_earlier_mounts: 600,
+            bytes_held_on_the_device: GpuMemoryInBytes(900),
+            bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(600),
         }]);
-        assert_eq!(memory_leaked.bytes_leaked_on(&card("0000:01:00.0")), 900);
+        assert_eq!(
+            memory_leaked.bytes_leaked_on(&card("0000:01:00.0")),
+            GpuMemoryInBytes(900)
+        );
     }
 
-    /// 判决 m4-gpugc-code-r2 第三节第 4 条：同一进程里两次挂载同时开着、都在同一张卡上泄漏（开卡那一刻都没算上对方的），先丢的那一次 300 字节、
+    /// 同一进程里两次挂载同时开着、都在同一张卡上泄漏（开卡那一刻都没算上对方的），先丢的那一次 300 字节、
     /// 后丢的那一次 600 字节——进程里这张卡按次累加记 900，不让后丢的那一次盖掉先丢的。
     #[test]
     fn two_mounts_alive_at_once_leaking_on_one_gpu_add_up_in_the_process() {
@@ -2005,13 +2021,13 @@
         for bytes_leaked_by_the_mount in [300, 600] {
             memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
                 device: card("0000:01:00.0"),
-                bytes_held_on_the_device: bytes_leaked_by_the_mount,
-                bytes_leaked_by_earlier_mounts: 0,
+                bytes_held_on_the_device: GpuMemoryInBytes(bytes_leaked_by_the_mount),
+                bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
             }]);
         }
         assert_eq!(
             memory_leaked.bytes_leaked_on(&card("0000:01:00.0")),
-            900,
+            GpuMemoryInBytes(900),
             "两次各 300、600 字节：记 900"
         );
     }
@@ -2033,8 +2049,8 @@
         let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
         memory_leaked.note_the_gpus_leaked_when_dropped(&[GpuLeakedWhenDropped {
             device: card("0000:01:00.0"),
-            bytes_held_on_the_device: 600,
-            bytes_leaked_by_earlier_mounts: 0,
+            bytes_held_on_the_device: GpuMemoryInBytes(600),
+            bytes_leaked_by_earlier_mounts: GpuMemoryInBytes(0),
         }]);
         let opening = gpus_opened_at_the_mount(
             discrete_gpus,
@@ -2053,8 +2069,8 @@
             vec![GpuLeftOutUnderItsMemoryLimit {
                 device: card("0000:01:00.0"),
                 memory_limit: GpuMemoryInBytes(2000),
-                smallest_piece_bytes: 1840,
-                bytes_already_held_on_the_device: 600,
+                smallest_piece_bytes: GpuMemoryInBytes(1840),
+                bytes_already_held_on_the_device: GpuMemoryInBytes(600),
             }]
         );
         assert_eq!(
````

## crates/singlefs-background-gpu/src/lib.rs

与上一轮快照相同，没有改动。

## crates/singlefs-background-gpu/src/live_slot_kernels.rs

````diff
--- a/crates/singlefs-background-gpu/src/live_slot_kernels.rs
+++ b/crates/singlefs-background-gpu/src/live_slot_kernels.rs
@@ -33,14 +33,17 @@
 use std::ops::Range;
 
 use crate::checksum_kernels::UnrecognizedVerdictWord;
+use singlefs_core::background_compute::GpuMemoryInBytes;
+
 use crate::memory_limit_and_chunks::{
-    chunks_that_fit_on_the_gpu, ChunkBoundsOfAGpu, ChunkFootprint, SmallestPieceLargerThanTheRoom,
+    chunks_that_fit_on_the_gpu, ChunkBoundsOfAGpu, ChunkFootprint, EntryCountOfAChunk,
+    InputLengthOnTheHostInBytes, SmallestPieceLargerThanTheRoom,
     COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS,
     COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION,
 };
 use crate::two_kernel_reconciliation::{
-    ConsecutiveUncertaintyCounts, ReconciledBatch, SameEntryKeyTwiceInOneBatch,
-    TwoKernelOutcomesOfAnEntry,
+    first_entry_key_repeated_among, ConsecutiveUncertaintyCounts, ReconciledBatch,
+    SameEntryKeyTwiceInOneBatch, TwoKernelOutcomesOfAnEntry,
 };
 
 /// 逐条走遍记录、按区间相交算长度的那一份核。
@@ -56,12 +59,13 @@
 /// 两份核的工作组都是 64 个调用（两份 WGSL 里的 `@workgroup_size(64)`），每个段一个调用。
 pub const LIVE_SLOT_INVOCATIONS_PER_WORKGROUP: u32 = 64;
 /// 参数缓冲：四个 `u32`。
-pub const LIVE_SLOT_PARAMETER_BUFFER_BYTES: u64 = 16;
+pub const LIVE_SLOT_PARAMETER_BUFFER_BYTES: GpuMemoryInBytes = GpuMemoryInBytes(16);
 pub const SEGMENT_BOUND_WORDS: usize = 4;
-pub const SEGMENT_BOUND_BYTES: u64 = 16;
+/// 每个段一项上下界在主机上打包的字节数（[`SEGMENT_BOUND_WORDS`] 个 `u32`）：选段打分把它当这一段的「输入」交给切批。
+pub const SEGMENT_BOUND_BYTES: InputLengthOnTheHostInBytes = InputLengthOnTheHostInBytes(16);
 pub const ALLOCATION_RECORD_WORDS: usize = 5;
 pub const LIVE_SLOT_OUTCOME_WORDS: usize = 3;
-pub const LIVE_SLOT_OUTCOME_BYTES: u64 = 12;
+pub const LIVE_SLOT_OUTCOME_BYTES: GpuMemoryInBytes = GpuMemoryInBytes(12);
 /// 结果缓冲开算之前的填充字：有效判定字见 [`singlefs_replay_prefix::gpu_gc_proofs::GPU_LIVE_SLOTS_COUNTED`]。
 pub const LIVE_SLOT_OUTCOME_NOT_WRITTEN_FILL_WORD: u32 = 0xFFFF_FFFF;
 /// 释放了没有那一个字（两份 WGSL 里各写一份同值的常量）。
@@ -130,19 +134,19 @@
 
     /// 两个缓冲在卡上合占的字节数。
     #[must_use]
-    pub fn bytes_on_the_gpu(&self) -> u64 {
+    pub fn bytes_on_the_gpu(&self) -> GpuMemoryInBytes {
         let words = self.record_words.len() + self.record_positions_by_device_and_slot.len();
-        4 * u64::try_from(words).expect("装得进 u64")
+        GpuMemoryInBytes(4 * u64::try_from(words).expect("装得进 u64"))
     }
 
     /// 两个缓冲里大的那一个的字节数：它也要装得进这张卡的绑定上限。
     #[must_use]
-    pub fn largest_buffer_bytes(&self) -> u64 {
+    pub fn largest_buffer_bytes(&self) -> GpuMemoryInBytes {
         let words = self
             .record_words
             .len()
             .max(self.record_positions_by_device_and_slot.len());
-        4 * u64::try_from(words).expect("装得进 u64")
+        GpuMemoryInBytes(4 * u64::try_from(words).expect("装得进 u64"))
     }
 }
 
@@ -158,25 +162,60 @@
 
 /// 同 [`live_slot_chunk_footprint`]，记录表的两个缓冲按合占的字节数给。
 #[must_use]
-pub fn live_slot_chunk_footprint_of_record_table_bytes(record_table_bytes: u64) -> ChunkFootprint {
+pub fn live_slot_chunk_footprint_of_record_table_bytes(
+    record_table_bytes: GpuMemoryInBytes,
+) -> ChunkFootprint {
     ChunkFootprint {
         bytes_per_chunk_besides_the_entries: COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
-            * (LIVE_SLOT_PARAMETER_BUFFER_BYTES + record_table_bytes),
+            .bytes_of_the_copies(LIVE_SLOT_PARAMETER_BUFFER_BYTES.plus(record_table_bytes)),
         bytes_per_entry_besides_its_input: COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS
-            * LIVE_SLOT_OUTCOME_BYTES,
+            .bytes_of_the_copies(LIVE_SLOT_OUTCOME_BYTES),
         bytes_per_entry_in_the_largest_entry_buffer: LIVE_SLOT_OUTCOME_BYTES,
     }
 }
 
+/// 记录表（整份或一片）里几条分配记录：条数，与一段里几个段（[`EntryCountOfAChunk`]）、一张卡分到几个段（[`ClusterSegmentCountOfAGpu`]）不是一回事。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct AllocationRecordCountOfATable(pub u64);
+
+impl AllocationRecordCountOfATable {
+    /// 记录表里 `record_count` 条（调用方手里是切片长度、`usize`）。
+    ///
+    /// # Panics
+    /// `record_count` 装不进 `u64`（`usize` 不比 `u64` 宽的平台上走不到）。
+    #[must_use]
+    pub fn of_this_many_records(record_count: usize) -> Self {
+        Self(u64::try_from(record_count).expect("记录表的条数装得进 u64"))
+    }
+}
+
+/// 这一批分到一张卡的聚簇段有几个：段数，与一段里几条（[`EntryCountOfAChunk`]）、记录表里几条分配记录（[`AllocationRecordCountOfATable`]）
+/// 不是一回事。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct ClusterSegmentCountOfAGpu(pub u64);
+
+impl ClusterSegmentCountOfAGpu {
+    /// 一张卡分到 `segment_count` 个段（调用方手里是切片长度、`usize`）。
+    ///
+    /// # Panics
+    /// `segment_count` 装不进 `u64`（`usize` 不比 `u64` 宽的平台上走不到）。
+    #[must_use]
+    pub fn of_this_many_segments(segment_count: usize) -> Self {
+        Self(u64::try_from(segment_count).expect("一张卡分到的段数装得进 u64"))
+    }
+}
+
 /// `record_count` 条记录的记录表交给核时两个缓冲合占的字节数与大的那一个的字节数（按 [`AllocationRecordTablesForTheKernels`] 的建法：
 /// 没有记录时各放一项）。
 #[must_use]
-pub fn record_table_bytes_of(record_count: u64) -> (u64, u64) {
-    let record_words = ALLOCATION_RECORD_WORDS_AS_U64 * record_count.max(1);
-    let sorted_position_words = record_count.max(1);
+pub fn record_table_bytes_of(
+    record_count: AllocationRecordCountOfATable,
+) -> (GpuMemoryInBytes, GpuMemoryInBytes) {
+    let record_words = ALLOCATION_RECORD_WORDS_AS_U64 * record_count.0.max(1);
+    let sorted_position_words = record_count.0.max(1);
     (
-        4 * (record_words + sorted_position_words),
-        4 * record_words.max(sorted_position_words),
+        GpuMemoryInBytes(4 * (record_words + sorted_position_words)),
+        GpuMemoryInBytes(4 * record_words.max(sorted_position_words)),
     )
 }
 
@@ -184,9 +223,11 @@
 
 /// 选段打分一段里最小的一片在卡上合占的字节数：一个段连同只有一条记录的记录表。
 #[must_use]
-pub fn bytes_of_the_smallest_live_slot_piece() -> u64 {
-    live_slot_chunk_footprint_of_record_table_bytes(record_table_bytes_of(1).0)
-        .bytes_held_by_a_chunk(SEGMENT_BOUND_BYTES, 1)
+pub fn bytes_of_the_smallest_live_slot_piece() -> GpuMemoryInBytes {
+    live_slot_chunk_footprint_of_record_table_bytes(
+        record_table_bytes_of(AllocationRecordCountOfATable(1)).0,
+    )
+    .bytes_held_by_a_chunk(SEGMENT_BOUND_BYTES, EntryCountOfAChunk(1))
 }
 
 /// 记录表的一片（按记录的次序相连的几条）与这张卡上这一片之下段切成的几段（按这张卡分到的段里的位置）。
@@ -203,15 +244,14 @@
 /// # Errors
 /// 连只有一条记录的那一片连同一个段都装不下（[`SmallestPieceLargerThanTheRoom`]，`entry_position` 是 0）。
 pub fn live_slot_plan_on_a_gpu(
-    record_count: usize,
-    segment_count: usize,
+    record_count: AllocationRecordCountOfATable,
+    segment_count: ClusterSegmentCountOfAGpu,
     room: ChunkBoundsOfAGpu,
 ) -> Result<Vec<RecordSliceWithItsChunks>, SmallestPieceLargerThanTheRoom> {
-    if segment_count == 0 {
+    if segment_count.0 == 0 {
         return Ok(Vec::new());
     }
-    let record_count_wide = u64::try_from(record_count).expect("装得进 u64");
-    let fits = |records_in_the_slice: u64| -> bool {
+    let fits = |records_in_the_slice: AllocationRecordCountOfATable| -> bool {
         let (table_bytes, largest_table_bytes) = record_table_bytes_of(records_in_the_slice);
         largest_table_bytes <= room.largest_storage_binding_in_bytes
             && chunks_that_fit_on_the_gpu(
@@ -221,22 +261,27 @@
             )
             .is_ok()
     };
-    let records_per_slice = if fits(record_count_wide) {
-        record_count_wide.max(1)
+    let records_per_slice = if fits(record_count) {
+        record_count.0.max(1)
     } else {
         // 记录表每多一条，两个缓冲合占多 24 字节（连同暂存多 48 字节）、大的那一个多 20 字节：按显存上限与绑定上限各自的余量现算最多几条，再核一遍。
         let (table_bytes_of_one_record, largest_table_bytes_of_one_record) =
-            record_table_bytes_of(1);
-        let one_segment_without_the_table = live_slot_chunk_footprint_of_record_table_bytes(0)
-            .bytes_held_by_a_chunk(SEGMENT_BOUND_BYTES, 1);
+            record_table_bytes_of(AllocationRecordCountOfATable(1));
+        let one_segment_without_the_table =
+            live_slot_chunk_footprint_of_record_table_bytes(GpuMemoryInBytes(0))
+                .bytes_held_by_a_chunk(SEGMENT_BOUND_BYTES, EntryCountOfAChunk(1));
         let records_under_the_memory_limit = room.memory_limit.map_or(u64::MAX, |memory_limit| {
-            memory_limit.0.saturating_sub(one_segment_without_the_table)
-                / (COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION * table_bytes_of_one_record)
+            memory_limit
+                .0
+                .saturating_sub(one_segment_without_the_table.0)
+                / COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
+                    .bytes_of_the_copies(table_bytes_of_one_record)
+                    .0
         });
         let records_under_the_binding =
-            room.largest_storage_binding_in_bytes / largest_table_bytes_of_one_record;
+            room.largest_storage_binding_in_bytes.0 / largest_table_bytes_of_one_record.0;
         let records_per_slice = records_under_the_memory_limit.min(records_under_the_binding);
-        if records_per_slice == 0 || !fits(records_per_slice) {
+        if records_per_slice == 0 || !fits(AllocationRecordCountOfATable(records_per_slice)) {
             return Err(SmallestPieceLargerThanTheRoom {
                 entry_position: 0,
                 smallest_piece_bytes: bytes_of_the_smallest_live_slot_piece(),
@@ -246,20 +291,26 @@
     };
     let records_per_slice =
         usize::try_from(records_per_slice).expect("一片的条数不超过记录数，装得进 usize");
-    let segment_bound_lengths = vec![SEGMENT_BOUND_BYTES; segment_count];
+    let segment_bound_lengths = vec![
+        SEGMENT_BOUND_BYTES;
+        usize::try_from(segment_count.0)
+            .expect("一张卡分到的段数是切片长度，装得进 usize")
+    ];
     let mut slices = Vec::new();
     let mut slice_start = 0;
     loop {
         let slice_end = usize::try_from(
             singlefs_replay_prefix::gpu_gc_proofs::next_gpu_record_slice_end(
                 u64::try_from(slice_start).expect("记录位置装得进 u64"),
-                record_count_wide,
+                record_count.0,
                 u64::try_from(records_per_slice).expect("片容量装得进 u64"),
             ),
         )
         .expect("片终点不超过记录数，装得进 usize");
         let (table_bytes, _largest_table_bytes) =
-            record_table_bytes_of(u64::try_from(slice_end - slice_start).expect("装得进 u64"));
+            record_table_bytes_of(AllocationRecordCountOfATable(
+                u64::try_from(slice_end - slice_start).expect("装得进 u64"),
+            ));
         let segment_chunks = chunks_that_fit_on_the_gpu(
             &segment_bound_lengths,
             live_slot_chunk_footprint_of_record_table_bytes(table_bytes),
@@ -271,7 +322,7 @@
             segment_chunks,
         });
         slice_start = slice_end;
-        if slice_start >= record_count {
+        if u64::try_from(slice_start).expect("记录位置装得进 u64") >= record_count.0 {
             break;
         }
     }
@@ -433,13 +484,13 @@
 }
 
 /// 选段打分一批的比对（第 8 条）：两份核数出的活槽数交进来，逐段比、更新计数；连续不确定到 N 轮的段交 CPU 参照现判
-/// （整块单元区的 CPU 参照只在头一次要它时数一遍）。这一批出错、或同一个段在一批里出现两次，就交回出错、计数不动、CPU 参照不调。
+/// （整块单元区的 CPU 参照只在头一次要它时数一遍）。同一个段在一批里出现两次（先于整段清单那一判查），或这一批出错，就交回出错、计数不动、CPU 参照不调。
 ///
 /// # Errors
-/// `both_kernel_live_slots` 是 `Err` 时交回它；同一个段在一批里出现两次时交回 [`SameEntryKeyTwiceInOneBatch`] 转成的那一种。
+/// 同一个段在一批里出现两次时交回 [`SameEntryKeyTwiceInOneBatch`] 转成的那一种（两份核交成了还是出错了都先报它）；`both_kernel_live_slots` 是 `Err` 时交回它。
 ///
 /// # Panics
-/// `segments` 不是 `unit_area` 的整段按起点升序一个不漏（核心层 `BackgroundGpuCompute::count_live_slots_per_cluster_segment` 的约定）；
+/// `segments` 不重复而不是 `unit_area` 的整段按起点升序一个不漏（核心层 `BackgroundGpuCompute::count_live_slots_per_cluster_segment` 的约定）；
 /// 两份核交回的段数与 `segments` 不等。
 pub fn reconciled_live_slots_of_the_segments<BatchFailure: From<SameEntryKeyTwiceInOneBatch>>(
     allocation_records: &[AllocationRecord],
@@ -448,6 +499,9 @@
     both_kernel_live_slots: Result<Vec<(u64, u64)>, BatchFailure>,
     uncertainty: &mut ConsecutiveUncertaintyCounts<ClusterSegmentEntryKey>,
 ) -> Result<ReconciledBatch<u64>, BatchFailure> {
+    if let Some(repeated) = first_entry_key_repeated_among(segments.iter()) {
+        return Err(BatchFailure::from(repeated));
+    }
     assert_eq!(
         segments,
         cluster_segment_entry_keys_of_the_unit_area(unit_area).as_slice(),
@@ -566,7 +620,7 @@
         assert_eq!(tables.record_count, 0);
         assert_eq!(
             tables.bytes_on_the_gpu(),
-            4 * u64::try_from(ALLOCATION_RECORD_WORDS + 1).expect("小数")
+            GpuMemoryInBytes(4 * u64::try_from(ALLOCATION_RECORD_WORDS + 1).expect("小数"))
         );
     }
 
@@ -787,13 +841,31 @@
                 .collect();
             let tables = AllocationRecordTablesForTheKernels::of_the_allocation_records(&records);
             assert_eq!(
-                record_table_bytes_of(u64::try_from(record_count).expect("小数")),
+                record_table_bytes_of(AllocationRecordCountOfATable(
+                    u64::try_from(record_count).expect("小数")
+                )),
                 (tables.bytes_on_the_gpu(), tables.largest_buffer_bytes()),
                 "{record_count} 条"
             );
         }
     }
 
+    #[test]
+    fn allocation_record_count_of_this_many_records_is_the_slice_length() {
+        assert_eq!(
+            AllocationRecordCountOfATable::of_this_many_records(7),
+            AllocationRecordCountOfATable(7)
+        );
+    }
+
+    #[test]
+    fn cluster_segment_count_of_this_many_segments_is_the_slice_length() {
+        assert_eq!(
+            ClusterSegmentCountOfAGpu::of_this_many_segments(3),
+            ClusterSegmentCountOfAGpu(3)
+        );
+    }
+
     /// 第 3 条「按上限自适应切成几片」：上限 232 字节（一个段连同暂存 136 字节加两条记录连同暂存 96 字节）装不下整份记录表加一个段，
     /// 记录表按记录切成每片两条、各片逐段数（两份核在 CPU 上的替身照核心层数这一片记录）再相加，与 CPU 参照对整份记录逐段相同。
     #[test]
@@ -812,12 +884,16 @@
             record(UNIT_AREA_DEVICE, start + 2 * segment_slots + 1, 200, false),
         ];
         let room = ChunkBoundsOfAGpu {
-            memory_limit: Some(singlefs_core::background_compute::GpuMemoryInBytes(232)),
-            largest_storage_binding_in_bytes: 1 << 30,
-            largest_entry_count_per_dispatch: 1 << 20,
+            memory_limit: Some(GpuMemoryInBytes(232)),
+            largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
+            largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
         };
-        let plan = live_slot_plan_on_a_gpu(records.len(), segments.len(), room)
-            .expect("232 字节装得下最小的一片");
+        let plan = live_slot_plan_on_a_gpu(
+            AllocationRecordCountOfATable::of_this_many_records(records.len()),
+            ClusterSegmentCountOfAGpu::of_this_many_segments(segments.len()),
+            room,
+        )
+        .expect("232 字节装得下最小的一片");
         assert_eq!(
             plan.iter()
                 .map(|slice| slice.records.clone())
@@ -873,18 +949,124 @@
     #[test]
     fn the_memory_limit_below_the_smallest_live_slot_piece_reports_how_much_it_needs() {
         let room = ChunkBoundsOfAGpu {
-            memory_limit: Some(singlefs_core::background_compute::GpuMemoryInBytes(
-                bytes_of_the_smallest_live_slot_piece() - 1,
+            memory_limit: Some(GpuMemoryInBytes(
+                bytes_of_the_smallest_live_slot_piece().0 - 1,
             )),
-            largest_storage_binding_in_bytes: 1 << 30,
-            largest_entry_count_per_dispatch: 1 << 20,
+            largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
+            largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
         };
         assert_eq!(
-            live_slot_plan_on_a_gpu(7, 3, room),
+            live_slot_plan_on_a_gpu(
+                AllocationRecordCountOfATable(7),
+                ClusterSegmentCountOfAGpu(3),
+                room
+            ),
             Err(SmallestPieceLargerThanTheRoom {
                 entry_position: 0,
                 smallest_piece_bytes: bytes_of_the_smallest_live_slot_piece(),
             })
         );
     }
+
+    /// 选段打分一批里同一个段出现两次（`segments_of_the_batch` 照整段清单的位置摆），两份核交成了（`kernels_failed` 为假）或出错了都一样：
+    /// 交回同一个段两次出现的位置，不走到整段清单那一判，计数一条不动。
+    fn live_slots_of_a_batch_with_a_segment_repeated(
+        positions_in_the_full_list: &[usize],
+        kernels_failed: bool,
+    ) -> (
+        Result<ReconciledBatch<u64>, SameEntryKeyTwiceInOneBatch>,
+        u32,
+    ) {
+        let unit_area = unit_area_of_three_segments();
+        let full_list = cluster_segment_entry_keys_of_the_unit_area(unit_area);
+        let segments_of_the_batch: Vec<ClusterSegmentEntryKey> = positions_in_the_full_list
+            .iter()
+            .map(|position| full_list[*position])
+            .collect();
+        let both_kernel_live_slots = if kernels_failed {
+            Err(SameEntryKeyTwiceInOneBatch {
+                position_of_the_first_occurrence: usize::MAX,
+                position_of_the_second_occurrence: usize::MAX,
+            })
+        } else {
+            Ok(vec![(0, 1); segments_of_the_batch.len()])
+        };
+        let mut uncertainty = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
+        let reconciled = reconciled_live_slots_of_the_segments(
+            &[],
+            unit_area,
+            &segments_of_the_batch,
+            both_kernel_live_slots,
+            &mut uncertainty,
+        );
+        let rounds_of_the_first_segment =
+            uncertainty.consecutive_uncertain_rounds_of(&full_list[0]);
+        (reconciled, rounds_of_the_first_segment)
+    }
+
+    fn repeated_at(first: usize, second: usize) -> SameEntryKeyTwiceInOneBatch {
+        SameEntryKeyTwiceInOneBatch {
+            position_of_the_first_occurrence: first,
+            position_of_the_second_occurrence: second,
+        }
+    }
+
+    #[test]
+    fn the_middle_segment_twice_in_place_of_the_last_is_returned_as_a_repeated_entry_key() {
+        assert_eq!(
+            live_slots_of_a_batch_with_a_segment_repeated(&[0, 1, 1], false),
+            (Err(repeated_at(1, 2)), 0)
+        );
+    }
+
+    #[test]
+    fn the_middle_segment_twice_with_the_kernels_failed_is_returned_as_a_repeated_entry_key() {
+        assert_eq!(
+            live_slots_of_a_batch_with_a_segment_repeated(&[0, 1, 1], true),
+            (Err(repeated_at(1, 2)), 0)
+        );
+    }
+
+    #[test]
+    fn the_last_segment_appended_again_is_returned_as_a_repeated_entry_key() {
+        assert_eq!(
+            live_slots_of_a_batch_with_a_segment_repeated(&[0, 1, 2, 2], false),
+            (Err(repeated_at(2, 3)), 0)
+        );
+    }
+
+    #[test]
+    fn the_last_segment_appended_again_with_the_kernels_failed_is_returned_as_a_repeated_entry_key()
+    {
+        assert_eq!(
+            live_slots_of_a_batch_with_a_segment_repeated(&[0, 1, 2, 2], true),
+            (Err(repeated_at(2, 3)), 0)
+        );
+    }
+
+    #[test]
+    fn the_first_segment_appended_again_is_returned_as_a_repeated_entry_key() {
+        assert_eq!(
+            live_slots_of_a_batch_with_a_segment_repeated(&[0, 1, 2, 0], false),
+            (Err(repeated_at(0, 3)), 0)
+        );
+    }
+
+    #[test]
+    fn the_first_segment_appended_again_with_the_kernels_failed_is_returned_as_a_repeated_entry_key(
+    ) {
+        assert_eq!(
+            live_slots_of_a_batch_with_a_segment_repeated(&[0, 1, 2, 0], true),
+            (Err(repeated_at(0, 3)), 0)
+        );
+    }
+
+    /// 对照：整段清单一个不漏、两份核数得不一样——照常比，头一个段这一轮不确定（计数到 1）。
+    #[test]
+    fn the_full_list_of_segments_without_a_repeat_is_reconciled() {
+        let (reconciled, rounds_of_the_first_segment) =
+            live_slots_of_a_batch_with_a_segment_repeated(&[0, 1, 2], false);
+        assert!(reconciled.is_ok(), "{reconciled:?}");
+        assert_eq!(rounds_of_the_first_segment, 1);
+    }
 }
````

## crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs

````diff
--- a/crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs
+++ b/crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs
@@ -37,40 +37,257 @@
 pub const GPU_BUFFER_ALIGNMENT_IN_BYTES: u64 =
     singlefs_replay_prefix::gpu_gc_proofs::GPU_INPUT_ALIGNMENT;
 
+/// 主机上一条输入的字节长度，对齐前（切批按它切、按它取片）；一片在它那一条里的字节偏移、一段几条合起来的输入长度也用它。
+/// 量的是主机内存里那几个字节有多长，不是卡上的显存：卡上那个输入缓冲占的显存由 [`input_buffer_bytes`] 换成 [`GpuMemoryInBytes`]，
+/// 两者交反编不过。交给被验函数、切 `&[u8]` 的地方取内值。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct InputLengthOnTheHostInBytes(pub u64);
+
+impl InputLengthOnTheHostInBytes {
+    /// 主机上 `byte_count` 个字节（调用方手里是切片长度、`usize`）。
+    ///
+    /// # Panics
+    /// `byte_count` 装不进 `u64`（`usize` 不比 `u64` 宽的平台上走不到）。
+    #[must_use]
+    pub fn of_this_many_bytes(byte_count: usize) -> Self {
+        Self(u64::try_from(byte_count).expect("主机上的输入长度装得进 u64"))
+    }
+
+    /// 两段输入长度之和；超出 `u64` 交回 `None`。
+    #[must_use]
+    pub const fn checked_add(self, more: Self) -> Option<Self> {
+        match self.0.checked_add(more.0) {
+            Some(sum_in_bytes) => Some(Self(sum_in_bytes)),
+            None => None,
+        }
+    }
+
+    /// 两段输入长度之和，用在和不超过一条输入长度的地方（一片在它那一条里的偏移挪过头一片）。
+    ///
+    /// # Panics
+    /// 和超出 `u64`。
+    #[must_use]
+    pub const fn plus(self, more: Self) -> Self {
+        match self.checked_add(more) {
+            Some(sum) => sum,
+            None => panic!("主机上的输入长度之和超出 u64"),
+        }
+    }
+
+    /// 扣掉 `less` 之后剩下的输入长度（一片的终点扣起点、一条扣掉头一片）。
+    ///
+    /// # Panics
+    /// `less` 比它长。
+    #[must_use]
+    pub const fn minus(self, less: Self) -> Self {
+        match self.0.checked_sub(less.0) {
+            Some(difference_in_bytes) => Self(difference_in_bytes),
+            None => panic!("扣掉的输入长度比剩下的长"),
+        }
+    }
+}
+
+/// 几段输入长度合起来（一段里几片的输入加起来）。
+///
+/// # Panics
+/// 和超出 `u64`。
+impl std::iter::Sum for InputLengthOnTheHostInBytes {
+    fn sum<Lengths: Iterator<Item = Self>>(lengths: Lengths) -> Self {
+        lengths.fold(Self(0), Self::plus)
+    }
+}
+
 /// 输入缓冲的字节数：输入字节按 4 字节向上取整；一个字节都没有时也占 4 字节（wgpu 不许绑定长度为 0 的缓冲）。
+/// `input_bytes` 是对齐前的输入数据长度，交回的是卡上那个缓冲占的显存。
 #[must_use]
-pub fn input_buffer_bytes(input_bytes: u64) -> u64 {
-    singlefs_replay_prefix::gpu_gc_proofs::checked_gpu_input_bytes(input_bytes)
-        .expect("四字节对齐后的输入长度装得进 u64")
+pub fn input_buffer_bytes(input_bytes: InputLengthOnTheHostInBytes) -> GpuMemoryInBytes {
+    GpuMemoryInBytes(
+        singlefs_replay_prefix::gpu_gc_proofs::checked_gpu_input_bytes(input_bytes.0)
+            .expect("四字节对齐后的输入长度装得进 u64"),
+    )
+}
+
+/// 一份缓冲在卡上占几份它的字节（缓冲本身、另带的暂存、回读暂存各算一份）：份数，不是字节，也不是条数。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct CopiesOnTheGpu(pub u64);
+
+impl CopiesOnTheGpu {
+    /// 这么多份、每份 `bytes_of_one_copy` 字节合占的显存；超出 `u64` 交回 `None`。
+    #[must_use]
+    pub const fn checked_bytes_of_the_copies(
+        self,
+        bytes_of_one_copy: GpuMemoryInBytes,
+    ) -> Option<GpuMemoryInBytes> {
+        match bytes_of_one_copy.0.checked_mul(self.0) {
+            Some(bytes_of_the_copies) => Some(GpuMemoryInBytes(bytes_of_the_copies)),
+            None => None,
+        }
+    }
+
+    /// 这么多份、每份 `bytes_of_one_copy` 字节合占的显存，用在乘出来装得进 `u64` 的地方（各段固定占用的常量、一段真建的缓冲）。
+    ///
+    /// # Panics
+    /// 乘出来超出 `u64`。
+    #[must_use]
+    pub const fn bytes_of_the_copies(
+        self,
+        bytes_of_one_copy: GpuMemoryInBytes,
+    ) -> GpuMemoryInBytes {
+        match self.checked_bytes_of_the_copies(bytes_of_one_copy) {
+            Some(bytes_of_the_copies) => bytes_of_the_copies,
+            None => panic!("份数乘每份的显存字节数超出 u64"),
+        }
+    }
+}
+
+/// 一段里有几条（一次交给一张卡的那几条输入；一次派发最多几条也按它数）：条数，不是字节，也不是份数。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct EntryCountOfAChunk(pub u64);
+
+impl EntryCountOfAChunk {
+    /// 一段里 `entry_count` 条（调用方手里是切片长度、`usize`）。
+    ///
+    /// # Panics
+    /// `entry_count` 装不进 `u64`（`usize` 不比 `u64` 宽的平台上走不到）。
+    #[must_use]
+    pub fn of_this_many_entries(entry_count: usize) -> Self {
+        Self(u64::try_from(entry_count).expect("一段的条数装得进 u64"))
+    }
+
+    /// 这么多条、每条 `bytes_of_one_entry` 字节合占的显存（每条一项的缓冲：两份核每一份的结果缓冲按它建）。
+    ///
+    /// # Panics
+    /// 乘出来超出 `u64`。
+    #[must_use]
+    pub const fn bytes_of_the_entries(
+        self,
+        bytes_of_one_entry: GpuMemoryInBytes,
+    ) -> GpuMemoryInBytes {
+        match bytes_of_one_entry.0.checked_mul(self.0) {
+            Some(bytes_of_the_entries) => GpuMemoryInBytes(bytes_of_the_entries),
+            None => panic!("条数乘每条的显存字节数超出 u64"),
+        }
+    }
+
+    /// 这么多条、每条的输入都是 `input_bytes_of_one_entry` 字节时，这一段对齐前的输入合起来多长（选段打分每个段一项上下界）。
+    ///
+    /// # Panics
+    /// 乘出来超出 `u64`。
+    #[must_use]
+    pub const fn input_bytes_of_the_entries(
+        self,
+        input_bytes_of_one_entry: InputLengthOnTheHostInBytes,
+    ) -> InputLengthOnTheHostInBytes {
+        match input_bytes_of_one_entry.0.checked_mul(self.0) {
+            Some(input_bytes_of_the_entries) => {
+                InputLengthOnTheHostInBytes(input_bytes_of_the_entries)
+            }
+            None => panic!("条数乘每条的输入长度超出 u64"),
+        }
+    }
 }
 
 /// 一份映射着建的缓冲（建的时候映射着写内容，用法里没有 `MAP_WRITE`）在卡上占几份它的字节：缓冲本身，加 wgpu-core 30.0.1 为它另建的
 /// 一份同尺寸暂存（`src/device/resource.rs` 里 `mapped_at_creation` 那一支建的 `StagingBuffer`，尺寸是对齐后的缓冲尺寸；本包建的缓冲尺寸都是
 /// 4 的整数倍，对齐后不变）。解开映射时暂存挂进队列的待写，冲掉它的那一次提交做完才放。各段的参数、输入、描述、表与两份结果缓冲都这样建。
-pub const COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION: u64 = 2;
+pub const COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION: CopiesOnTheGpu = CopiesOnTheGpu(2);
+
+/// 映射着建的一份缓冲另带的暂存是几份：[`COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION`] 扣掉缓冲本身那一份。
+const STAGING_COPIES_OF_A_BUFFER_MAPPED_AT_CREATION: CopiesOnTheGpu =
+    CopiesOnTheGpu(COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION.0 - 1);
 
 /// 两份核每一项结果在卡上合占几份它的字节：两份核各一个结果缓冲（映射着建，各带暂存）与一个回读暂存（`MAP_READ`、不映射着建，不另带暂存）。
-pub const COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS: u64 =
-    2 * COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION + 2;
+pub const COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS: CopiesOnTheGpu =
+    CopiesOnTheGpu(2 * COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION.0 + 2);
 
 /// 一段（这一次交给一张卡的那几条）在卡上占多少：输入缓冲（映射着建，连同它的暂存占 [`COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION`] 份）之外，
 /// 每一条占的固定字节（描述、两份核的输出与回读暂存，映射着建的连同暂存），加上与条数无关的那几样（参数、每段都交一份的表，连同暂存）。
 /// 由每一段的核自己给（校验和核对那一段在 `checksum_kernels` 里）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct ChunkFootprint {
-    pub bytes_per_chunk_besides_the_entries: u64,
-    pub bytes_per_entry_besides_its_input: u64,
+    pub bytes_per_chunk_besides_the_entries: GpuMemoryInBytes,
+    pub bytes_per_entry_besides_its_input: GpuMemoryInBytes,
     /// 每一条在它最大的那一个按条排的缓冲里占几字节：那个缓冲也要装得进这张卡的绑定上限。
-    pub bytes_per_entry_in_the_largest_entry_buffer: u64,
+    pub bytes_per_entry_in_the_largest_entry_buffer: GpuMemoryInBytes,
 }
 
 impl ChunkFootprint {
-    /// 这一段在卡上合占的字节数：对齐后的输入连同它的暂存，加上每条与每段的固定占用。
+    /// 这一段在卡上合占的字节数：对齐后的输入连同它的暂存，加上每条与每段的固定占用。`input_bytes` 是这一段对齐前的输入数据长度
+    /// （同 [`input_buffer_bytes`]）。
+    ///
+    /// 条数与输入长度各是各的类型，交反了编不过；照对的次序交编得过（下面两段只差交的次序）：
+    ///
+    /// ```compile_fail
+    /// use singlefs_background_gpu::checksum_kernels::CHECKSUM_CHUNK_FOOTPRINT;
+    /// use singlefs_background_gpu::memory_limit_and_chunks::{
+    ///     EntryCountOfAChunk, InputLengthOnTheHostInBytes,
+    /// };
+    /// let _held = CHECKSUM_CHUNK_FOOTPRINT
+    ///     .bytes_held_by_a_chunk(EntryCountOfAChunk(1), InputLengthOnTheHostInBytes(4096));
+    /// ```
+    ///
+    /// ```
+    /// use singlefs_background_gpu::checksum_kernels::CHECKSUM_CHUNK_FOOTPRINT;
+    /// use singlefs_background_gpu::memory_limit_and_chunks::{
+    ///     EntryCountOfAChunk, InputLengthOnTheHostInBytes,
+    /// };
+    /// let _held = CHECKSUM_CHUNK_FOOTPRINT
+    ///     .bytes_held_by_a_chunk(InputLengthOnTheHostInBytes(4096), EntryCountOfAChunk(1));
+    /// ```
+    ///
+    /// 主机上的输入长度与卡上的显存字节数各是各的类型：显存字节数交到输入长度那一格编不过，
+    /// 输入长度交到显存字节数那一格（剩下的地方扣掉已占的）也编不过：
+    ///
+    /// ```compile_fail
+    /// use singlefs_background_gpu::checksum_kernels::CHECKSUM_CHUNK_FOOTPRINT;
+    /// use singlefs_background_gpu::memory_limit_and_chunks::EntryCountOfAChunk;
+    /// use singlefs_core::background_compute::GpuMemoryInBytes;
+    /// let _held =
+    ///     CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(GpuMemoryInBytes(4096), EntryCountOfAChunk(1));
+    /// ```
+    ///
+    /// ```compile_fail
+    /// use singlefs_background_gpu::memory_limit_and_chunks::{
+    ///     ChunkBoundsOfAGpu, EntryCountOfAChunk, InputLengthOnTheHostInBytes,
+    /// };
+    /// use singlefs_core::background_compute::GpuMemoryInBytes;
+    /// let bounds = ChunkBoundsOfAGpu {
+    ///     memory_limit: Some(GpuMemoryInBytes(1 << 20)),
+    ///     largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
+    ///     largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
+    /// };
+    /// let _room = bounds.with_the_room_left_after(InputLengthOnTheHostInBytes(4096));
+    /// ```
+    ///
+    /// ```
+    /// use singlefs_background_gpu::memory_limit_and_chunks::{ChunkBoundsOfAGpu, EntryCountOfAChunk};
+    /// use singlefs_core::background_compute::GpuMemoryInBytes;
+    /// let bounds = ChunkBoundsOfAGpu {
+    ///     memory_limit: Some(GpuMemoryInBytes(1 << 20)),
+    ///     largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
+    ///     largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
+    /// };
+    /// let _room = bounds.with_the_room_left_after(GpuMemoryInBytes(4096));
+    /// ```
+    ///
+    /// 份数交到每份字节数那一格同样编不过：
+    ///
+    /// ```compile_fail
+    /// use singlefs_background_gpu::memory_limit_and_chunks::{
+    ///     COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS, COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION,
+    /// };
+    /// let _bytes = COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
+    ///     .bytes_of_the_copies(COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS);
+    /// ```
     ///
     /// # Panics
     /// 乘加超出 `u64`：调用方交的是已经切过、装得下这张卡的一段，或最小的一片。
     #[must_use]
-    pub fn bytes_held_by_a_chunk(&self, input_bytes: u64, entry_count: u64) -> u64 {
+    pub fn bytes_held_by_a_chunk(
+        &self,
+        input_bytes: InputLengthOnTheHostInBytes,
+        entry_count: EntryCountOfAChunk,
+    ) -> GpuMemoryInBytes {
         checked_bytes_held_by_a_chunk(*self, input_bytes, entry_count)
             .expect("GPU 缓冲占用的乘加装得进 u64")
     }
@@ -80,19 +297,19 @@
 /// 再加输入的暂存那一份（被验的 `checked_gpu_input_bytes`）。
 fn checked_bytes_held_by_a_chunk(
     footprint: ChunkFootprint,
-    input_bytes: u64,
-    entry_count: u64,
-) -> Option<u64> {
-    let bytes_besides_the_staging_of_the_input =
+    input_bytes: InputLengthOnTheHostInBytes,
+    entry_count: EntryCountOfAChunk,
+) -> Option<GpuMemoryInBytes> {
+    let bytes_besides_the_staging_of_the_input = GpuMemoryInBytes(
         singlefs_replay_prefix::gpu_gc_proofs::checked_gpu_chunk_bytes(
-            input_bytes,
-            entry_count,
-            footprint.bytes_per_entry_besides_its_input,
-            footprint.bytes_per_chunk_besides_the_entries,
-        )?;
-    let staging_copies_of_the_input = COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION - 1;
-    checked_gpu_input_bytes(input_bytes)?
-        .checked_mul(staging_copies_of_the_input)?
+            input_bytes.0,
+            entry_count.0,
+            footprint.bytes_per_entry_besides_its_input.0,
+            footprint.bytes_per_chunk_besides_the_entries.0,
+        )?,
+    );
+    STAGING_COPIES_OF_A_BUFFER_MAPPED_AT_CREATION
+        .checked_bytes_of_the_copies(GpuMemoryInBytes(checked_gpu_input_bytes(input_bytes.0)?))?
         .checked_add(bytes_besides_the_staging_of_the_input)
 }
 
@@ -100,20 +317,20 @@
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct ChunkBoundsOfAGpu {
     pub memory_limit: Option<GpuMemoryInBytes>,
-    pub largest_storage_binding_in_bytes: u64,
-    pub largest_entry_count_per_dispatch: u64,
+    pub largest_storage_binding_in_bytes: GpuMemoryInBytes,
+    pub largest_entry_count_per_dispatch: EntryCountOfAChunk,
 }
 
 impl ChunkBoundsOfAGpu {
     /// 这张卡上已经有 `bytes_already_held_on_the_gpu` 字节被本包别的段占着时，再切一段能用的界：显存上限扣掉已占的（第 3 条：全部后台任务
     /// 在那张卡上合占的显存不超过上限，三段同时在一张卡上时一起算），绑定上限与派发上限不变。不给上限的照旧不设限；已占的不比上限少时一字节都不剩。
     #[must_use]
-    pub fn with_the_room_left_after(self, bytes_already_held_on_the_gpu: u64) -> Self {
+    pub fn with_the_room_left_after(self, bytes_already_held_on_the_gpu: GpuMemoryInBytes) -> Self {
         Self {
             memory_limit: self.memory_limit.map(|memory_limit| {
                 GpuMemoryInBytes(singlefs_replay_prefix::gpu_gc_proofs::remaining_gpu_memory(
                     memory_limit.0,
-                    bytes_already_held_on_the_gpu,
+                    bytes_already_held_on_the_gpu.0,
                 ))
             }),
             largest_storage_binding_in_bytes: self.largest_storage_binding_in_bytes,
@@ -124,11 +341,11 @@
     /// 卡上此刻空闲的显存比剩下的地方少（建缓冲显存不足，第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」）时，显存那一项压到至多
     /// `bytes_at_most`：给了上限的取两者小的，不给上限的取它。绑定上限与派发上限不变。
     #[must_use]
-    pub fn with_the_memory_limit_at_most(self, bytes_at_most: u64) -> Self {
+    pub fn with_the_memory_limit_at_most(self, bytes_at_most: GpuMemoryInBytes) -> Self {
         Self {
             memory_limit: Some(GpuMemoryInBytes(tightened_gpu_memory_limit(
                 self.memory_limit.map(|limit| limit.0),
-                bytes_at_most,
+                bytes_at_most.0,
             ))),
             largest_storage_binding_in_bytes: self.largest_storage_binding_in_bytes,
             largest_entry_count_per_dispatch: self.largest_entry_count_per_dispatch,
@@ -143,11 +360,11 @@
     memory_limit_handed_to_the_verified_function: Option<u64>,
 ) -> GpuChunkCapacity {
     GpuChunkCapacity {
-        fixed_bytes: footprint.bytes_per_chunk_besides_the_entries,
-        bytes_per_entry: footprint.bytes_per_entry_besides_its_input,
-        entry_buffer_bytes: footprint.bytes_per_entry_in_the_largest_entry_buffer,
-        binding_bytes: bounds.largest_storage_binding_in_bytes,
-        dispatch_entries: bounds.largest_entry_count_per_dispatch,
+        fixed_bytes: footprint.bytes_per_chunk_besides_the_entries.0,
+        bytes_per_entry: footprint.bytes_per_entry_besides_its_input.0,
+        entry_buffer_bytes: footprint.bytes_per_entry_in_the_largest_entry_buffer.0,
+        binding_bytes: bounds.largest_storage_binding_in_bytes.0,
+        dispatch_entries: bounds.largest_entry_count_per_dispatch.0,
         memory_limit: memory_limit_handed_to_the_verified_function,
     }
 }
@@ -156,31 +373,33 @@
 /// 交给它的显存上限先扣掉输入的暂存那几份（被验函数里 `对齐后的输入 + 条数 × 每条 + 每段 ≤ 上限 − 暂存`，即 [`ChunkFootprint::bytes_held_by_a_chunk`] ≤ 上限）。
 /// 输入的暂存比上限还大、或对齐后的输入超出 `u64`，装不下。不给上限的只判绑定与派发。
 fn chunk_fits_with_the_staging_of_its_input(
-    input_bytes: u64,
-    entry_count: u64,
+    input_bytes: InputLengthOnTheHostInBytes,
+    entry_count: EntryCountOfAChunk,
     footprint: ChunkFootprint,
     bounds: ChunkBoundsOfAGpu,
 ) -> bool {
-    let staging_copies_of_the_input = COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION - 1;
     let memory_limit_handed_to_the_verified_function = match bounds.memory_limit {
         None => None,
         Some(memory_limit) => {
-            let Some(staging_of_the_input) = checked_gpu_input_bytes(input_bytes)
-                .and_then(|aligned_input| aligned_input.checked_mul(staging_copies_of_the_input))
+            let Some(staging_of_the_input) =
+                checked_gpu_input_bytes(input_bytes.0).and_then(|aligned_input| {
+                    STAGING_COPIES_OF_A_BUFFER_MAPPED_AT_CREATION
+                        .checked_bytes_of_the_copies(GpuMemoryInBytes(aligned_input))
+                })
             else {
                 return false;
             };
             let Some(limit_left_besides_the_staging) =
-                memory_limit.0.checked_sub(staging_of_the_input)
+                memory_limit.checked_sub(staging_of_the_input)
             else {
                 return false;
             };
-            Some(limit_left_besides_the_staging)
+            Some(limit_left_besides_the_staging.0)
         }
     };
     gpu_chunk_fits(
-        input_bytes,
-        entry_count,
+        input_bytes.0,
+        entry_count.0,
         &verified_chunk_capacity_under_the_memory_limit(
             footprint,
             bounds,
@@ -193,7 +412,7 @@
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct EntryLargerThanAChunkOnTheGpu {
     pub entry_position: usize,
-    pub input_bytes: u64,
+    pub input_bytes: InputLengthOnTheHostInBytes,
 }
 
 /// 把一批（每条的输入字节数按批里的次序给）切成相连的几段，每段在卡上合占的字节不超过上限、输入与按条排的缓冲都装得进绑定上限、
@@ -202,16 +421,16 @@
 /// # Errors
 /// 某一条单独成一段也装不下（[`EntryLargerThanAChunkOnTheGpu`]），一段都不交。
 pub fn chunks_that_fit_on_the_gpu(
-    entry_input_lengths_in_bytes: &[u64],
+    entry_input_lengths_in_bytes: &[InputLengthOnTheHostInBytes],
     footprint: ChunkFootprint,
     bounds: ChunkBoundsOfAGpu,
 ) -> Result<Vec<std::ops::Range<usize>>, EntryLargerThanAChunkOnTheGpu> {
     let mut chunks = Vec::new();
     let mut chunk_start = 0;
-    let mut chunk_input_bytes: u64 = 0;
+    let mut chunk_input_bytes = InputLengthOnTheHostInBytes(0);
     for (entry_position, input_bytes) in entry_input_lengths_in_bytes.iter().copied().enumerate() {
         let entry_count_with_this_entry =
-            u64::try_from(entry_position - chunk_start + 1).expect("条数装得进 u64");
+            EntryCountOfAChunk::of_this_many_entries(entry_position - chunk_start + 1);
         if let Some(combined) = chunk_input_bytes
             .checked_add(input_bytes)
             .filter(|combined| {
@@ -226,7 +445,12 @@
             chunk_input_bytes = combined;
             continue;
         }
-        if !chunk_fits_with_the_staging_of_its_input(input_bytes, 1, footprint, bounds) {
+        if !chunk_fits_with_the_staging_of_its_input(
+            input_bytes,
+            EntryCountOfAChunk(1),
+            footprint,
+            bounds,
+        ) {
             return Err(EntryLargerThanAChunkOnTheGpu {
                 entry_position,
                 input_bytes,
@@ -246,13 +470,13 @@
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct PieceOfAnEntry {
     pub entry_position: usize,
-    pub input_bytes: std::ops::Range<u64>,
+    pub input_bytes: std::ops::Range<InputLengthOnTheHostInBytes>,
 }
 
 impl PieceOfAnEntry {
     #[must_use]
-    pub fn input_byte_count(&self) -> u64 {
-        self.input_bytes.end - self.input_bytes.start
+    pub fn input_byte_count(&self) -> InputLengthOnTheHostInBytes {
+        self.input_bytes.end.minus(self.input_bytes.start)
     }
 }
 
@@ -261,7 +485,7 @@
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct SmallestPieceLargerThanTheRoom {
     pub entry_position: usize,
-    pub smallest_piece_bytes: u64,
+    pub smallest_piece_bytes: GpuMemoryInBytes,
 }
 
 /// 一片单独成一段时，它的输入最多几个字节装得下 `bounds`：按显存上限扣掉一段的固定占用与一条的固定占用、剩下的分给输入连同它的暂存
@@ -270,20 +494,22 @@
 ///
 /// 被验的 `largest_gpu_piece_input` 只把输入算一遍：交给它的显存上限是「一段与一条的固定占用」加「剩下的除以份数」，它交回的最大 4 的整数倍
 /// 照份数乘回去不超过剩下的；比它大的 4 的整数倍乘份数都超过剩下的。显存上限连一段与一条的固定占用都装不下就交回 `None`。
-fn largest_piece_input_bytes(footprint: ChunkFootprint, bounds: ChunkBoundsOfAGpu) -> Option<u64> {
+fn largest_piece_input_bytes(
+    footprint: ChunkFootprint,
+    bounds: ChunkBoundsOfAGpu,
+) -> Option<InputLengthOnTheHostInBytes> {
     let memory_limit_handed_to_the_verified_function = match bounds.memory_limit {
         None => None,
         Some(memory_limit) => {
             let bytes_of_a_chunk_and_an_entry_besides_the_input = footprint
                 .bytes_per_chunk_besides_the_entries
                 .checked_add(footprint.bytes_per_entry_besides_its_input)?;
-            let room_for_the_input_and_its_staging = memory_limit
-                .0
-                .checked_sub(bytes_of_a_chunk_and_an_entry_besides_the_input)?;
+            let room_for_the_input_and_its_staging =
+                memory_limit.checked_sub(bytes_of_a_chunk_and_an_entry_besides_the_input)?;
             Some(
-                bytes_of_a_chunk_and_an_entry_besides_the_input
-                    + room_for_the_input_and_its_staging
-                        / COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION,
+                bytes_of_a_chunk_and_an_entry_besides_the_input.0
+                    + room_for_the_input_and_its_staging.0
+                        / COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION.0,
             )
         }
     };
@@ -292,6 +518,7 @@
         bounds,
         memory_limit_handed_to_the_verified_function,
     ))
+    .map(InputLengthOnTheHostInBytes)
 }
 
 /// 把一批切成相连的几段，每段是几片；装得下的一条整条是一片，单独成一段都装不下的一条从头切成几片，每片取单独成一段装得下的最大字节数
@@ -301,7 +528,7 @@
 /// # Errors
 /// 某一条要切而连一个字节的一片都装不下（[`SmallestPieceLargerThanTheRoom`]），一段都不交。
 pub fn pieces_cut_to_fit_on_the_gpu(
-    entry_input_lengths_in_bytes: &[u64],
+    entry_input_lengths_in_bytes: &[InputLengthOnTheHostInBytes],
     footprint: ChunkFootprint,
     bounds: ChunkBoundsOfAGpu,
 ) -> Result<Vec<Vec<PieceOfAnEntry>>, SmallestPieceLargerThanTheRoom> {
@@ -313,26 +540,29 @@
         if whole_entry_fits {
             pieces.push(PieceOfAnEntry {
                 entry_position,
-                input_bytes: 0..input_bytes,
+                input_bytes: InputLengthOnTheHostInBytes(0)..input_bytes,
             });
             continue;
         }
         let Some(largest_piece_bytes) = largest_piece else {
             return Err(SmallestPieceLargerThanTheRoom {
                 entry_position,
-                smallest_piece_bytes: footprint.bytes_held_by_a_chunk(1, 1),
+                smallest_piece_bytes: footprint
+                    .bytes_held_by_a_chunk(InputLengthOnTheHostInBytes(1), EntryCountOfAChunk(1)),
             });
         };
         assert!(
             largest_piece_bytes < input_bytes,
             "最大的一片装得下而整条装不下：整条比最大的一片长"
         );
-        let mut piece_start = 0;
+        let mut piece_start = InputLengthOnTheHostInBytes(0);
         while piece_start < input_bytes {
-            let piece_end = singlefs_replay_prefix::gpu_gc_proofs::next_gpu_record_slice_end(
-                piece_start,
-                input_bytes,
-                largest_piece_bytes,
+            let piece_end = InputLengthOnTheHostInBytes(
+                singlefs_replay_prefix::gpu_gc_proofs::next_gpu_record_slice_end(
+                    piece_start.0,
+                    input_bytes.0,
+                    largest_piece_bytes.0,
+                ),
             );
             pieces.push(PieceOfAnEntry {
                 entry_position,
@@ -341,7 +571,7 @@
             piece_start = piece_end;
         }
     }
-    let piece_lengths: Vec<u64> = pieces
+    let piece_lengths: Vec<InputLengthOnTheHostInBytes> = pieces
         .iter()
         .map(PieceOfAnEntry::input_byte_count)
         .collect();
@@ -380,7 +610,7 @@
 #[derive(Debug)]
 struct SubmissionRunningPastItsWait<Submission> {
     submission: Submission,
-    bytes_held_by_its_chunk: u64,
+    bytes_held_by_its_chunk: GpuMemoryInBytes,
 }
 
 /// 一张卡上切不出这一批分到它的几条（这张卡上一段都不交）。
@@ -398,7 +628,7 @@
 pub struct GpuMemoryAccount<Submission> {
     memory_limit: Option<GpuMemoryInBytes>,
     /// 合占的字节数，含还在跑的那次提交那一段。
-    bytes_held: u64,
+    bytes_held: GpuMemoryInBytes,
     /// 至多一次：它没做完之前这张卡不再交活（[`Self::hold`] 断言）。
     submission_running_past_its_wait: Option<SubmissionRunningPastItsWait<Submission>>,
 }
@@ -406,7 +636,7 @@
 impl<Submission> GpuMemoryAccount<Submission> {
     #[must_use]
     pub fn with_nothing_held(memory_limit: Option<GpuMemoryInBytes>) -> Self {
-        Self::with_the_bytes_leaked_by_earlier_mounts(memory_limit, 0)
+        Self::with_the_bytes_leaked_by_earlier_mounts(memory_limit, GpuMemoryInBytes(0))
     }
 
     /// 这个进程里之前几次挂载在这张卡上泄漏了 `bytes_leaked_by_earlier_mounts` 字节（丢后台 GPU 时还在跑的提交，设备不析构，显存不还）：
@@ -414,7 +644,7 @@
     #[must_use]
     pub fn with_the_bytes_leaked_by_earlier_mounts(
         memory_limit: Option<GpuMemoryInBytes>,
-        bytes_leaked_by_earlier_mounts: u64,
+        bytes_leaked_by_earlier_mounts: GpuMemoryInBytes,
     ) -> Self {
         Self {
             memory_limit,
@@ -424,7 +654,7 @@
     }
 
     #[must_use]
-    pub fn bytes_held(&self) -> u64 {
+    pub fn bytes_held(&self) -> GpuMemoryInBytes {
         self.bytes_held
     }
 
@@ -454,27 +684,30 @@
     /// 这张卡上还有一次等超过上限、没做完的提交：交活之前先等它（[`Self::chunks_cut_once_the_submission_running_past_its_wait_completed`]），
     /// 走到这里是外壳跳过了那一步。
     /// 记下之后超过上限：切批（[`chunks_that_fit_on_the_gpu`]）按同一个上限切过，超了是切批与建缓冲对不上的 bug。
-    pub fn hold(&mut self, bytes: u64) {
+    pub fn hold(&mut self, bytes: GpuMemoryInBytes) {
         assert!(
             self.submission_running_past_its_wait.is_none(),
             "这张卡上还有一次等超过上限、没做完的提交：交下一段之前先等它做完"
         );
-        self.bytes_held = checked_gpu_memory_hold(
-            self.bytes_held,
-            bytes,
-            self.submission_running_past_its_wait.is_some(),
-            self.memory_limit.map(|limit| limit.0),
-        )
-        .expect("本包在这张卡上占的缓冲超过了上限或 u64：切批与记账必须一致");
+        self.bytes_held = GpuMemoryInBytes(
+            checked_gpu_memory_hold(
+                self.bytes_held.0,
+                bytes.0,
+                self.submission_running_past_its_wait.is_some(),
+                self.memory_limit.map(|limit| limit.0),
+            )
+            .expect("本包在这张卡上占的缓冲超过了上限或 u64：切批与记账必须一致"),
+        );
     }
 
     /// 放掉丢掉的那几个缓冲。
     ///
     /// # Panics
     /// 放的比记的多。
-    pub fn release(&mut self, bytes: u64) {
-        self.bytes_held =
-            checked_gpu_memory_release(self.bytes_held, bytes).expect("放掉的缓冲比记下的多");
+    pub fn release(&mut self, bytes: GpuMemoryInBytes) {
+        self.bytes_held = GpuMemoryInBytes(
+            checked_gpu_memory_release(self.bytes_held.0, bytes.0).expect("放掉的缓冲比记下的多"),
+        );
     }
 
     /// 一段的提交等完之后结它的账：做完了就放掉这一段；没等到它做完的，账照旧记着，记成这张卡上还在跑的那次提交，
@@ -484,12 +717,12 @@
     /// 放的比记的多；或这张卡上已经记着一次还在跑的提交（交这一段之前 [`Self::hold`] 断言过没有）。
     pub fn settle_the_chunk_after_its_submission(
         &mut self,
-        bytes_held_by_the_chunk: u64,
+        bytes_held_by_the_chunk: GpuMemoryInBytes,
         submission_of_the_chunk: SubmissionOfTheChunk<Submission>,
     ) {
         let bytes_after_wait = gpu_bytes_after_submission_wait(
-            self.bytes_held,
-            bytes_held_by_the_chunk,
+            self.bytes_held.0,
+            bytes_held_by_the_chunk.0,
             matches!(
                 &submission_of_the_chunk,
                 SubmissionOfTheChunk::CompletedOnTheGpu
@@ -509,20 +742,21 @@
                 });
             }
         }
-        self.bytes_held = bytes_after_wait;
+        self.bytes_held = GpuMemoryInBytes(bytes_after_wait);
     }
 
     /// 一段建缓冲出错（显存不足，或等设备交回错误范围超过上限）之后结它的账：这一段已经建好、解开映射的那几份缓冲连同暂存挂在队列的待写里，
     /// 冲掉待写的那一次提交（`flushing_submission`，调用方交的一次空提交）做完才放。等到它做完（`waited_for_the_flush` 是 `Ok`）就放掉这一段、
     /// 交回建缓冲那一次的错；没等到就把这一段记成这张卡上还在跑的那次提交（同 [`Self::settle_the_chunk_after_its_submission`]），
-    /// 交回等的那一次的错：这张卡下一次交之前先等它做完（等超过上限的那两种照重交的规矩数）。
+    /// 这张卡下一次交之前先等它做完：等满了上限交回 [`GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit`]（算这一批这张卡一次超时，
+    /// 这一段没交到卡上、不算重交），等的时候报的别的出错照原样交回。
     ///
     /// # Panics
     /// 同 [`Self::settle_the_chunk_after_its_submission`]。
     #[must_use]
     pub fn settle_the_chunk_whose_buffers_were_not_created(
         &mut self,
-        bytes_held_by_the_chunk: u64,
+        bytes_held_by_the_chunk: GpuMemoryInBytes,
         failure_creating_the_buffers: GpuFailureCause,
         flushing_submission: Submission,
         waited_for_the_flush: Result<(), GpuFailureCause>,
@@ -537,7 +771,11 @@
         );
         match waited_for_the_flush {
             Ok(()) => failure_creating_the_buffers,
-            Err(failure_waiting_for_the_flush) => failure_waiting_for_the_flush,
+            Err(failure_waiting_for_the_flush) => {
+                failure_of_waiting_for_the_flush_of_the_pending_writes(
+                    failure_waiting_for_the_flush,
+                )
+            }
         }
     }
 
@@ -601,7 +839,7 @@
         deadline: &GpuWaitDeadline,
         clock: &impl MonotonicClock,
         check_once_waiting_at_most: impl FnMut(&Submission, Duration) -> CompletionCheck<()>,
-        entry_input_lengths_in_bytes: &[u64],
+        entry_input_lengths_in_bytes: &[InputLengthOnTheHostInBytes],
         footprint: ChunkFootprint,
         bounds: ChunkBoundsOfAGpu,
     ) -> Result<Vec<std::ops::Range<usize>>, ChunksNotCutOnTheGpu> {
@@ -621,6 +859,32 @@
     }
 }
 
+/// 等冲掉待写的那一次空提交等满了上限：报成它自己的那一种（这一段没交到卡上）；别的出错照原样。
+fn failure_of_waiting_for_the_flush_of_the_pending_writes(
+    cause: GpuFailureCause,
+) -> GpuFailureCause {
+    match cause {
+        GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit } => {
+            GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { wait_limit }
+        }
+        reported_by_the_wait @ (GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
+        | GpuFailureCause::OpeningThreadNotStarted { .. }
+        | GpuFailureCause::OpeningPanicked { .. }
+        | GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned
+        | GpuFailureCause::OpeningCalledOff
+        | GpuFailureCause::OpeningsInThisMountUsedUp { .. }
+        | GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { .. }
+        | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
+        | GpuFailureCause::WaitForTheSubmissionFailed { .. }
+        | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
+        | GpuFailureCause::ReadbackMappingFailed { .. }
+        | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
+        | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
+        | GpuFailureCause::UnusableAfterResubmissions { .. }
+        | GpuFailureCause::NotOpenedAfterTheReopenings { .. }) => reported_by_the_wait,
+    }
+}
+
 /// 等前一次超时的那次提交又等满了上限：报成它自己的那一种，与这一次交的段自己等超时分得开；别的出错照原样。
 fn failure_of_waiting_for_the_earlier_submission(cause: GpuFailureCause) -> GpuFailureCause {
     match cause {
@@ -628,6 +892,12 @@
             GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { wait_limit }
         }
         reported_by_the_wait @ (GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
+        | GpuFailureCause::OpeningThreadNotStarted { .. }
+        | GpuFailureCause::OpeningPanicked { .. }
+        | GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned
+        | GpuFailureCause::OpeningCalledOff
+        | GpuFailureCause::OpeningsInThisMountUsedUp { .. }
+        | GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { .. }
         | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
         | GpuFailureCause::WaitForTheSubmissionFailed { .. }
         | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
@@ -646,19 +916,28 @@
 
     /// 每条另占 40 字节、每段另占 16 字节、按条排的最大缓冲每条 16 字节：校验和核对那一段的量级，数是测试自己定的。
     const FOOTPRINT_UNDER_TEST: ChunkFootprint = ChunkFootprint {
-        bytes_per_chunk_besides_the_entries: 16,
-        bytes_per_entry_besides_its_input: 40,
-        bytes_per_entry_in_the_largest_entry_buffer: 16,
+        bytes_per_chunk_besides_the_entries: GpuMemoryInBytes(16),
+        bytes_per_entry_besides_its_input: GpuMemoryInBytes(40),
+        bytes_per_entry_in_the_largest_entry_buffer: GpuMemoryInBytes(16),
     };
 
     fn bounds(memory_limit_in_bytes: Option<u64>) -> ChunkBoundsOfAGpu {
         ChunkBoundsOfAGpu {
             memory_limit: memory_limit_in_bytes.map(GpuMemoryInBytes),
-            largest_storage_binding_in_bytes: 1 << 30,
-            largest_entry_count_per_dispatch: 1 << 20,
+            largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
+            largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
         }
     }
 
+    /// 用例写的每条输入长度（主机上的字节）。
+    fn lengths_on_the_host(lengths_in_bytes: &[u64]) -> Vec<InputLengthOnTheHostInBytes> {
+        lengths_in_bytes
+            .iter()
+            .copied()
+            .map(InputLengthOnTheHostInBytes)
+            .collect()
+    }
+
     /// 用例里交给 GPU 的一次提交：只是个编号。
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     struct SubmissionNamedByTheTest(u32);
@@ -670,7 +949,7 @@
     /// 这张卡的显存上限：用例自己定的数。
     const MEMORY_LIMIT_UNDER_TEST_IN_BYTES: u64 = 1000;
     /// 等超时的那一段占的字节。
-    const BYTES_HELD_BY_THE_TIMED_OUT_CHUNK: u64 = 600;
+    const BYTES_HELD_BY_THE_TIMED_OUT_CHUNK: GpuMemoryInBytes = GpuMemoryInBytes(600);
 
     /// 假钟：只在假完成信号让它走时才走。
     struct ClockMovedByTheCompletionSignal {
@@ -766,7 +1045,7 @@
         let bounds_of_the_gpu = bounds(Some(MEMORY_LIMIT_UNDER_TEST_IN_BYTES));
         assert_eq!(
             chunks_that_fit_on_the_gpu(
-                &lengths,
+                &lengths_on_the_host(&lengths),
                 FOOTPRINT_UNDER_TEST,
                 bounds_of_the_gpu.with_the_room_left_after(account.bytes_held()),
             ),
@@ -778,7 +1057,7 @@
             &GpuWaitDeadline::starting_now(WAIT_LIMIT_UNDER_TEST, &clock),
             &clock,
             completion_signal_of(&clock, TIMED_OUT_SUBMISSION, Duration::from_secs(107)),
-            &lengths,
+            &lengths_on_the_host(&lengths),
             FOOTPRINT_UNDER_TEST,
             bounds_of_the_gpu,
         );
@@ -792,7 +1071,11 @@
             Duration::from_secs(107),
             "等到它做完那一刻"
         );
-        assert_eq!(account.bytes_held(), 0, "完成信号来了才放");
+        assert_eq!(
+            account.bytes_held(),
+            GpuMemoryInBytes(0),
+            "完成信号来了才放"
+        );
     }
 
     /// 第 3、4 条：前一次等超时的那次提交还没在 GPU 上做完时，下一批不在这张卡上交活：切段之前在上限之内等它，等满上限还没做完就交回出错，
@@ -805,7 +1088,7 @@
             &GpuWaitDeadline::starting_now(WAIT_LIMIT_UNDER_TEST, &clock),
             &clock,
             completion_signal_of(&clock, TIMED_OUT_SUBMISSION, Duration::from_secs(200)),
-            &[100, 100, 100, 100, 100],
+            &lengths_on_the_host(&[100, 100, 100, 100, 100]),
             FOOTPRINT_UNDER_TEST,
             bounds(Some(MEMORY_LIMIT_UNDER_TEST_IN_BYTES)),
         );
@@ -836,7 +1119,7 @@
     fn holding_a_chunk_while_a_timed_out_submission_is_still_running_panics() {
         let (mut account, _clock) =
             account_after_a_chunk_whose_wait_timed_out(Duration::from_secs(107));
-        account.hold(100);
+        account.hold(GpuMemoryInBytes(100));
     }
 
     /// 这一批没分到这张卡的活：不在这张卡上交活，就不等它前一次超时的那次提交；一段不切，账照旧记着。
@@ -848,7 +1131,7 @@
             &GpuWaitDeadline::starting_now(WAIT_LIMIT_UNDER_TEST, &clock),
             &clock,
             |_submission, _time_left| panic!("没分到活的卡不该等"),
-            &[],
+            &lengths_on_the_host(&[]),
             FOOTPRINT_UNDER_TEST,
             bounds(Some(MEMORY_LIMIT_UNDER_TEST_IN_BYTES)),
         );
@@ -861,18 +1144,173 @@
     }
 
     #[test]
+    fn copies_on_the_gpu_times_the_bytes_of_one_copy_give_the_bytes_of_the_copies() {
+        assert_eq!(
+            CopiesOnTheGpu(3).bytes_of_the_copies(GpuMemoryInBytes(16)),
+            GpuMemoryInBytes(48)
+        );
+        assert_eq!(
+            COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS
+                .bytes_of_the_copies(GpuMemoryInBytes(8)),
+            GpuMemoryInBytes(48),
+            "两份核每一项结果：两个结果缓冲各连同暂存两份、两个回读暂存各一份，合六份"
+        );
+        assert_eq!(
+            STAGING_COPIES_OF_A_BUFFER_MAPPED_AT_CREATION
+                .bytes_of_the_copies(GpuMemoryInBytes(4100)),
+            GpuMemoryInBytes(4100),
+            "映射着建的缓冲另带一份暂存"
+        );
+    }
+
+    #[test]
+    fn copies_on_the_gpu_checked_bytes_of_the_copies_refuse_past_u64() {
+        assert_eq!(
+            CopiesOnTheGpu(2).checked_bytes_of_the_copies(GpuMemoryInBytes(7)),
+            Some(GpuMemoryInBytes(14))
+        );
+        assert_eq!(
+            CopiesOnTheGpu(2).checked_bytes_of_the_copies(GpuMemoryInBytes(u64::MAX / 2 + 1)),
+            None
+        );
+    }
+
+    #[test]
+    #[should_panic(expected = "份数乘每份的显存字节数超出 u64")]
+    fn copies_on_the_gpu_bytes_of_the_copies_past_u64_panics() {
+        let _bytes = CopiesOnTheGpu(2).bytes_of_the_copies(GpuMemoryInBytes(u64::MAX));
+    }
+
+    #[test]
+    fn entry_count_of_a_chunk_times_the_bytes_of_one_entry_gives_the_bytes_of_the_entries() {
+        assert_eq!(
+            EntryCountOfAChunk(5).bytes_of_the_entries(GpuMemoryInBytes(12)),
+            GpuMemoryInBytes(60)
+        );
+        assert_eq!(
+            EntryCountOfAChunk::of_this_many_entries(7),
+            EntryCountOfAChunk(7)
+        );
+    }
+
+    #[test]
+    #[should_panic(expected = "条数乘每条的显存字节数超出 u64")]
+    fn entry_count_of_a_chunk_bytes_of_the_entries_past_u64_panics() {
+        let _bytes = EntryCountOfAChunk(2).bytes_of_the_entries(GpuMemoryInBytes(u64::MAX));
+    }
+
+    #[test]
+    fn entry_count_of_a_chunk_times_the_input_bytes_of_one_entry_gives_the_input_bytes_of_the_entries(
+    ) {
+        assert_eq!(
+            EntryCountOfAChunk(3).input_bytes_of_the_entries(InputLengthOnTheHostInBytes(16)),
+            InputLengthOnTheHostInBytes(48)
+        );
+    }
+
+    #[test]
+    #[should_panic(expected = "条数乘每条的输入长度超出 u64")]
+    fn entry_count_of_a_chunk_input_bytes_of_the_entries_past_u64_panics() {
+        let _input_bytes =
+            EntryCountOfAChunk(2).input_bytes_of_the_entries(InputLengthOnTheHostInBytes(u64::MAX));
+    }
+
+    #[test]
+    fn input_length_on_the_host_of_this_many_bytes_is_the_slice_length() {
+        assert_eq!(
+            InputLengthOnTheHostInBytes::of_this_many_bytes(4097),
+            InputLengthOnTheHostInBytes(4097)
+        );
+    }
+
+    #[test]
+    fn input_length_on_the_host_checked_add_gives_the_sum_and_refuses_past_u64() {
+        assert_eq!(
+            InputLengthOnTheHostInBytes(628).checked_add(InputLengthOnTheHostInBytes(100)),
+            Some(InputLengthOnTheHostInBytes(728))
+        );
+        assert_eq!(
+            InputLengthOnTheHostInBytes(u64::MAX).checked_add(InputLengthOnTheHostInBytes(1)),
+            None
+        );
+    }
+
+    #[test]
+    fn input_length_on_the_host_plus_gives_the_sum() {
+        assert_eq!(
+            InputLengthOnTheHostInBytes(628).plus(InputLengthOnTheHostInBytes(484)),
+            InputLengthOnTheHostInBytes(1112)
+        );
+    }
+
+    #[test]
+    #[should_panic(expected = "主机上的输入长度之和超出 u64")]
+    fn input_length_on_the_host_plus_past_u64_panics() {
+        let _sum = InputLengthOnTheHostInBytes(u64::MAX).plus(InputLengthOnTheHostInBytes(1));
+    }
+
+    #[test]
+    fn input_length_on_the_host_minus_gives_what_is_left() {
+        assert_eq!(
+            InputLengthOnTheHostInBytes(968).minus(InputLengthOnTheHostInBytes(484)),
+            InputLengthOnTheHostInBytes(484)
+        );
+        assert_eq!(
+            PieceOfAnEntry {
+                entry_position: 0,
+                input_bytes: InputLengthOnTheHostInBytes(3872)..InputLengthOnTheHostInBytes(4096),
+            }
+            .input_byte_count(),
+            InputLengthOnTheHostInBytes(224),
+            "一片罩住的字节数是终点扣起点"
+        );
+    }
+
+    #[test]
+    #[should_panic(expected = "扣掉的输入长度比剩下的长")]
+    fn input_length_on_the_host_minus_more_than_there_is_panics() {
+        let _left = InputLengthOnTheHostInBytes(484).minus(InputLengthOnTheHostInBytes(968));
+    }
+
+    #[test]
+    fn input_lengths_on_the_host_summed_over_the_pieces_are_their_total() {
+        let total: InputLengthOnTheHostInBytes =
+            lengths_on_the_host(&[484, 484, 224]).into_iter().sum();
+        assert_eq!(total, InputLengthOnTheHostInBytes(1192));
+    }
+
+    #[test]
     fn input_buffers_round_up_to_whole_words_and_never_have_zero_bytes() {
-        assert_eq!(input_buffer_bytes(0), 4);
-        assert_eq!(input_buffer_bytes(1), 4);
-        assert_eq!(input_buffer_bytes(4), 4);
-        assert_eq!(input_buffer_bytes(5), 8);
-        assert_eq!(input_buffer_bytes(4097), 4100);
+        assert_eq!(
+            input_buffer_bytes(InputLengthOnTheHostInBytes(0)),
+            GpuMemoryInBytes(4)
+        );
+        assert_eq!(
+            input_buffer_bytes(InputLengthOnTheHostInBytes(1)),
+            GpuMemoryInBytes(4)
+        );
+        assert_eq!(
+            input_buffer_bytes(InputLengthOnTheHostInBytes(4)),
+            GpuMemoryInBytes(4)
+        );
+        assert_eq!(
+            input_buffer_bytes(InputLengthOnTheHostInBytes(5)),
+            GpuMemoryInBytes(8)
+        );
+        assert_eq!(
+            input_buffer_bytes(InputLengthOnTheHostInBytes(4097)),
+            GpuMemoryInBytes(4100)
+        );
     }
 
     #[test]
     fn without_a_memory_limit_a_batch_is_one_chunk() {
         assert_eq!(
-            chunks_that_fit_on_the_gpu(&[4096, 4096, 7, 0], FOOTPRINT_UNDER_TEST, bounds(None)),
+            chunks_that_fit_on_the_gpu(
+                &lengths_on_the_host(&[4096, 4096, 7, 0]),
+                FOOTPRINT_UNDER_TEST,
+                bounds(None)
+            ),
             Ok(std::iter::once(0..4).collect::<Vec<_>>()),
             "不给上限就不设限：整批一段"
         );
@@ -884,21 +1322,21 @@
         // 两条一段：输入 200 字节连同暂存 400、条目 80、参数 16，合 496；三条要 736
         let memory_limit_in_bytes = 500;
         let chunks = chunks_that_fit_on_the_gpu(
-            &lengths,
+            &lengths_on_the_host(&lengths),
             FOOTPRINT_UNDER_TEST,
             bounds(Some(memory_limit_in_bytes)),
         )
         .expect("单条 256 字节装得下");
         assert_eq!(chunks, vec![0..2, 2..4, 4..5], "两条一段，最后剩一条");
         for chunk in &chunks {
-            let input_bytes: u64 = lengths[chunk.clone()].iter().sum();
+            let input_bytes = InputLengthOnTheHostInBytes(lengths[chunk.clone()].iter().sum());
             let held = FOOTPRINT_UNDER_TEST.bytes_held_by_a_chunk(
                 input_bytes,
-                u64::try_from(chunk.len()).expect("装得进 u64"),
+                EntryCountOfAChunk::of_this_many_entries(chunk.len()),
             );
             assert!(
-                held <= memory_limit_in_bytes,
-                "每一段合占 {held} 字节不超过上限 {memory_limit_in_bytes}"
+                held <= GpuMemoryInBytes(memory_limit_in_bytes),
+                "每一段合占 {held:?} 字节不超过上限 {memory_limit_in_bytes}"
             );
         }
     }
@@ -907,11 +1345,19 @@
     fn the_chunk_exactly_at_the_memory_limit_is_kept_whole() {
         // 两条 100 字节：2 × 200 + 80 + 16 = 496，正好等于上限也算装得下
         assert_eq!(
-            chunks_that_fit_on_the_gpu(&[100, 100], FOOTPRINT_UNDER_TEST, bounds(Some(496))),
+            chunks_that_fit_on_the_gpu(
+                &lengths_on_the_host(&[100, 100]),
+                FOOTPRINT_UNDER_TEST,
+                bounds(Some(496))
+            ),
             Ok(std::iter::once(0..2).collect::<Vec<_>>())
         );
         assert_eq!(
-            chunks_that_fit_on_the_gpu(&[100, 100], FOOTPRINT_UNDER_TEST, bounds(Some(495))),
+            chunks_that_fit_on_the_gpu(
+                &lengths_on_the_host(&[100, 100]),
+                FOOTPRINT_UNDER_TEST,
+                bounds(Some(495))
+            ),
             Ok(vec![0..1, 1..2]),
             "少一个字节就切开"
         );
@@ -921,24 +1367,32 @@
     fn the_dispatch_and_binding_limits_also_cut_chunks() {
         let dispatch_bound = ChunkBoundsOfAGpu {
             memory_limit: None,
-            largest_storage_binding_in_bytes: 1 << 30,
-            largest_entry_count_per_dispatch: 3,
+            largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
+            largest_entry_count_per_dispatch: EntryCountOfAChunk(3),
         };
         assert_eq!(
-            chunks_that_fit_on_the_gpu(&[1; 7], FOOTPRINT_UNDER_TEST, dispatch_bound),
+            chunks_that_fit_on_the_gpu(
+                &lengths_on_the_host(&[1; 7]),
+                FOOTPRINT_UNDER_TEST,
+                dispatch_bound
+            ),
             Ok(vec![0..3, 3..6, 6..7]),
             "一次派发最多 3 条"
         );
         let binding_bound = ChunkBoundsOfAGpu {
             memory_limit: None,
-            largest_storage_binding_in_bytes: 8,
-            largest_entry_count_per_dispatch: 1 << 20,
+            largest_storage_binding_in_bytes: GpuMemoryInBytes(8),
+            largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
         };
         assert_eq!(
-            chunks_that_fit_on_the_gpu(&[4, 4, 4], FOOTPRINT_UNDER_TEST, binding_bound),
+            chunks_that_fit_on_the_gpu(
+                &lengths_on_the_host(&[4, 4, 4]),
+                FOOTPRINT_UNDER_TEST,
+                binding_bound
+            ),
             Err(EntryLargerThanAChunkOnTheGpu {
                 entry_position: 0,
-                input_bytes: 4
+                input_bytes: InputLengthOnTheHostInBytes(4)
             }),
             "一条在按条排的缓冲里就占 16 字节，超过 8 字节的绑定上限"
         );
@@ -955,31 +1409,35 @@
         ));
         // 先占着的那一段：每段另占 200 字节、每条另占 50 字节的四条 25 字节，合 2 × 100 + 200 + 200 = 600
         let footprint_of_the_other_segment = ChunkFootprint {
-            bytes_per_chunk_besides_the_entries: 200,
-            bytes_per_entry_besides_its_input: 50,
-            bytes_per_entry_in_the_largest_entry_buffer: 50,
+            bytes_per_chunk_besides_the_entries: GpuMemoryInBytes(200),
+            bytes_per_entry_besides_its_input: GpuMemoryInBytes(50),
+            bytes_per_entry_in_the_largest_entry_buffer: GpuMemoryInBytes(50),
         };
         assert_eq!(
             chunks_that_fit_on_the_gpu(
-                &[25, 25, 25, 25],
+                &lengths_on_the_host(&[25, 25, 25, 25]),
                 footprint_of_the_other_segment,
                 bounds_of_the_gpu.with_the_room_left_after(account.bytes_held()),
             ),
             Ok(std::iter::once(0..4).collect::<Vec<_>>())
         );
-        let held_by_the_other_segment =
-            footprint_of_the_other_segment.bytes_held_by_a_chunk(100, 4);
-        assert_eq!(held_by_the_other_segment, 600);
+        let held_by_the_other_segment = footprint_of_the_other_segment
+            .bytes_held_by_a_chunk(InputLengthOnTheHostInBytes(100), EntryCountOfAChunk(4));
+        assert_eq!(held_by_the_other_segment, GpuMemoryInBytes(600));
         account.hold(held_by_the_other_segment);
 
         let lengths = [50_u64, 50, 50, 50, 50];
         assert_eq!(
-            chunks_that_fit_on_the_gpu(&lengths, FOOTPRINT_UNDER_TEST, bounds_of_the_gpu),
+            chunks_that_fit_on_the_gpu(
+                &lengths_on_the_host(&lengths),
+                FOOTPRINT_UNDER_TEST,
+                bounds_of_the_gpu
+            ),
             Ok(std::iter::once(0..5).collect::<Vec<_>>()),
             "单看这一段，五条合 720 字节装得进 1000：不扣另一段占的就整批一段，两段合占 1320 字节"
         );
         let chunks = chunks_that_fit_on_the_gpu(
-            &lengths,
+            &lengths_on_the_host(&lengths),
             FOOTPRINT_UNDER_TEST,
             bounds_of_the_gpu.with_the_room_left_after(account.bytes_held()),
         )
@@ -990,15 +1448,15 @@
             "只用剩下的 400 字节：两条 296 一段，三条 440 装不下"
         );
         for chunk in &chunks {
-            let input_bytes: u64 = lengths[chunk.clone()].iter().sum();
+            let input_bytes = InputLengthOnTheHostInBytes(lengths[chunk.clone()].iter().sum());
             let held_by_this_chunk = FOOTPRINT_UNDER_TEST.bytes_held_by_a_chunk(
                 input_bytes,
-                u64::try_from(chunk.len()).expect("装得进 u64"),
+                EntryCountOfAChunk::of_this_many_entries(chunk.len()),
             );
             account.hold(held_by_this_chunk);
             assert!(
-                account.bytes_held() <= memory_limit_in_bytes,
-                "两段合占 {} 字节不超过上限 {memory_limit_in_bytes}",
+                account.bytes_held() <= GpuMemoryInBytes(memory_limit_in_bytes),
+                "两段合占 {:?} 字节不超过上限 {memory_limit_in_bytes}",
                 account.bytes_held()
             );
             account.release(held_by_this_chunk);
@@ -1013,12 +1471,14 @@
     #[test]
     fn the_room_left_after_other_segments_without_a_memory_limit_is_still_unlimited() {
         assert_eq!(
-            bounds(None).with_the_room_left_after(1 << 40),
+            bounds(None).with_the_room_left_after(GpuMemoryInBytes(1 << 40)),
             bounds(None),
             "不给上限就不设限，别的段占多少都一样"
         );
         assert_eq!(
-            bounds(Some(100)).with_the_room_left_after(150).memory_limit,
+            bounds(Some(100))
+                .with_the_room_left_after(GpuMemoryInBytes(150))
+                .memory_limit,
             Some(GpuMemoryInBytes(0)),
             "已占的比上限多：一字节都不剩"
         );
@@ -1027,10 +1487,14 @@
     #[test]
     fn an_entry_that_alone_exceeds_the_memory_limit_refuses_the_batch() {
         assert_eq!(
-            chunks_that_fit_on_the_gpu(&[10, 4096, 10], FOOTPRINT_UNDER_TEST, bounds(Some(1024))),
+            chunks_that_fit_on_the_gpu(
+                &lengths_on_the_host(&[10, 4096, 10]),
+                FOOTPRINT_UNDER_TEST,
+                bounds(Some(1024))
+            ),
             Err(EntryLargerThanAChunkOnTheGpu {
                 entry_position: 1,
-                input_bytes: 4096
+                input_bytes: InputLengthOnTheHostInBytes(4096)
             })
         );
     }
@@ -1038,7 +1502,11 @@
     #[test]
     fn an_empty_batch_has_no_chunk() {
         assert_eq!(
-            chunks_that_fit_on_the_gpu(&[], FOOTPRINT_UNDER_TEST, bounds(Some(64))),
+            chunks_that_fit_on_the_gpu(
+                &lengths_on_the_host(&[]),
+                FOOTPRINT_UNDER_TEST,
+                bounds(Some(64))
+            ),
             Ok(Vec::new())
         );
     }
@@ -1048,12 +1516,16 @@
         let mut account = GpuMemoryAccount::<SubmissionNamedByTheTest>::with_nothing_held(Some(
             GpuMemoryInBytes(100),
         ));
-        account.hold(60);
-        account.hold(40);
-        assert_eq!(account.bytes_held(), 100, "正好到上限");
-        account.release(100);
-        account.hold(100);
-        assert_eq!(account.bytes_held(), 100, "放掉之后又能占满");
+        account.hold(GpuMemoryInBytes(60));
+        account.hold(GpuMemoryInBytes(40));
+        assert_eq!(account.bytes_held(), GpuMemoryInBytes(100), "正好到上限");
+        account.release(GpuMemoryInBytes(100));
+        account.hold(GpuMemoryInBytes(100));
+        assert_eq!(
+            account.bytes_held(),
+            GpuMemoryInBytes(100),
+            "放掉之后又能占满"
+        );
     }
 
     #[test]
@@ -1062,50 +1534,57 @@
         let mut account = GpuMemoryAccount::<SubmissionNamedByTheTest>::with_nothing_held(Some(
             GpuMemoryInBytes(100),
         ));
-        account.hold(60);
-        account.hold(41);
+        account.hold(GpuMemoryInBytes(60));
+        account.hold(GpuMemoryInBytes(41));
     }
 
     #[test]
     fn without_a_memory_limit_holding_is_never_refused() {
         let mut account = GpuMemoryAccount::<SubmissionNamedByTheTest>::with_nothing_held(None);
-        account.hold(u64::MAX / 2);
-        assert_eq!(account.bytes_held(), u64::MAX / 2);
+        account.hold(GpuMemoryInBytes(u64::MAX / 2));
+        assert_eq!(account.bytes_held(), GpuMemoryInBytes(u64::MAX / 2));
     }
 
     /// 整条都装得下的批：每条一片，切出的段与不切单条的切法相同。
     #[test]
     fn the_batch_whose_entries_each_fit_is_cut_like_the_chunks_with_one_whole_piece_per_entry() {
         let lengths = [100_u64, 100, 100, 100, 100];
-        let pieces =
-            pieces_cut_to_fit_on_the_gpu(&lengths, FOOTPRINT_UNDER_TEST, bounds(Some(500)))
-                .expect("单条 256 字节装得下");
+        let pieces = pieces_cut_to_fit_on_the_gpu(
+            &lengths_on_the_host(&lengths),
+            FOOTPRINT_UNDER_TEST,
+            bounds(Some(500)),
+        )
+        .expect("单条 256 字节装得下");
         assert_eq!(
             pieces,
             vec![
                 vec![
                     PieceOfAnEntry {
                         entry_position: 0,
-                        input_bytes: 0..100
+                        input_bytes: InputLengthOnTheHostInBytes(0)
+                            ..InputLengthOnTheHostInBytes(100)
                     },
                     PieceOfAnEntry {
                         entry_position: 1,
-                        input_bytes: 0..100
+                        input_bytes: InputLengthOnTheHostInBytes(0)
+                            ..InputLengthOnTheHostInBytes(100)
                     }
                 ],
                 vec![
                     PieceOfAnEntry {
                         entry_position: 2,
-                        input_bytes: 0..100
+                        input_bytes: InputLengthOnTheHostInBytes(0)
+                            ..InputLengthOnTheHostInBytes(100)
                     },
                     PieceOfAnEntry {
                         entry_position: 3,
-                        input_bytes: 0..100
+                        input_bytes: InputLengthOnTheHostInBytes(0)
+                            ..InputLengthOnTheHostInBytes(100)
                     }
                 ],
                 vec![PieceOfAnEntry {
                     entry_position: 4,
-                    input_bytes: 0..100
+                    input_bytes: InputLengthOnTheHostInBytes(0)..InputLengthOnTheHostInBytes(100)
                 }],
             ]
         );
@@ -1119,7 +1598,7 @@
         let lengths = [10_u64, 4096, 10];
         let memory_limit_in_bytes = 1024;
         let chunks = pieces_cut_to_fit_on_the_gpu(
-            &lengths,
+            &lengths_on_the_host(&lengths),
             FOOTPRINT_UNDER_TEST,
             bounds(Some(memory_limit_in_bytes)),
         )
@@ -1127,7 +1606,12 @@
         let pieces_in_order: Vec<(usize, std::ops::Range<u64>)> = chunks
             .iter()
             .flatten()
-            .map(|piece| (piece.entry_position, piece.input_bytes.clone()))
+            .map(|piece| {
+                (
+                    piece.entry_position,
+                    piece.input_bytes.start.0..piece.input_bytes.end.0,
+                )
+            })
             .collect();
         assert_eq!(
             pieces_in_order,
@@ -1147,12 +1631,15 @@
             "4096 字节切成八片 484 与一片 224"
         );
         for chunk in &chunks {
-            let input_bytes: u64 = chunk.iter().map(PieceOfAnEntry::input_byte_count).sum();
-            let held = FOOTPRINT_UNDER_TEST
-                .bytes_held_by_a_chunk(input_bytes, u64::try_from(chunk.len()).expect("小数"));
+            let input_bytes: InputLengthOnTheHostInBytes =
+                chunk.iter().map(PieceOfAnEntry::input_byte_count).sum();
+            let held = FOOTPRINT_UNDER_TEST.bytes_held_by_a_chunk(
+                input_bytes,
+                EntryCountOfAChunk::of_this_many_entries(chunk.len()),
+            );
             assert!(
-                held <= memory_limit_in_bytes,
-                "每一段合占 {held} 字节不超过上限"
+                held <= GpuMemoryInBytes(memory_limit_in_bytes),
+                "每一段合占 {held:?} 字节不超过上限"
             );
         }
     }
@@ -1160,13 +1647,14 @@
     /// 连一个字节的一片都装不下：交回要切的那一条与最小的一片合占多少（这张卡至少要剩这么多）。
     #[test]
     fn the_memory_limit_below_the_smallest_piece_reports_how_much_the_smallest_piece_needs() {
-        let smallest_piece_bytes = FOOTPRINT_UNDER_TEST.bytes_held_by_a_chunk(1, 1);
-        assert_eq!(smallest_piece_bytes, 2 * 4 + 40 + 16);
+        let smallest_piece_bytes = FOOTPRINT_UNDER_TEST
+            .bytes_held_by_a_chunk(InputLengthOnTheHostInBytes(1), EntryCountOfAChunk(1));
+        assert_eq!(smallest_piece_bytes, GpuMemoryInBytes(2 * 4 + 40 + 16));
         assert_eq!(
             pieces_cut_to_fit_on_the_gpu(
-                &[0, 4096],
+                &lengths_on_the_host(&[0, 4096]),
                 FOOTPRINT_UNDER_TEST,
-                bounds(Some(smallest_piece_bytes - 1))
+                bounds(Some(smallest_piece_bytes.0 - 1))
             ),
             Err(SmallestPieceLargerThanTheRoom {
                 entry_position: 0,
@@ -1176,9 +1664,9 @@
         );
         assert!(
             pieces_cut_to_fit_on_the_gpu(
-                &[4096],
+                &lengths_on_the_host(&[4096]),
                 FOOTPRINT_UNDER_TEST,
-                bounds(Some(smallest_piece_bytes))
+                bounds(Some(smallest_piece_bytes.0))
             )
             .is_ok(),
             "正好装得下最小的一片：切成 1024 片"
@@ -1192,48 +1680,49 @@
     fn checksum_chunk_fitting_without_the_staging_but_not_with_it_is_cut_into_pieces_that_fit_with_the_staging(
     ) {
         use crate::checksum_kernels::CHECKSUM_CHUNK_FOOTPRINT;
-        let bytes_held_with_the_staging = CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(4096, 1);
+        let bytes_held_with_the_staging = CHECKSUM_CHUNK_FOOTPRINT
+            .bytes_held_by_a_chunk(InputLengthOnTheHostInBytes(4096), EntryCountOfAChunk(1));
         assert_eq!(
             bytes_held_with_the_staging,
-            2 * (16 + 4096 + 12 + 2 * 8) + 2 * 8,
+            GpuMemoryInBytes(2 * (16 + 4096 + 12 + 2 * 8) + 2 * 8),
             "参数、输入、描述、两份结果各两份，两份回读各一份"
         );
         let bytes_held_without_the_staging = 4096 + 12 + 4 * 8 + 16;
         assert_eq!(
             chunks_that_fit_on_the_gpu(
-                &[4096],
+                &lengths_on_the_host(&[4096]),
                 CHECKSUM_CHUNK_FOOTPRINT,
                 bounds(Some(bytes_held_without_the_staging))
             ),
             Err(EntryLargerThanAChunkOnTheGpu {
                 entry_position: 0,
-                input_bytes: 4096
+                input_bytes: InputLengthOnTheHostInBytes(4096)
             }),
             "只算缓冲本身正好装得下的上限，连同暂存装不下"
         );
         assert_eq!(
             chunks_that_fit_on_the_gpu(
-                &[4096],
+                &lengths_on_the_host(&[4096]),
                 CHECKSUM_CHUNK_FOOTPRINT,
-                bounds(Some(bytes_held_with_the_staging))
+                bounds(Some(bytes_held_with_the_staging.0))
             ),
             Ok(std::iter::once(0..1).collect::<Vec<_>>()),
             "连同暂存正好装得下：整条一段"
         );
         assert_eq!(
             chunks_that_fit_on_the_gpu(
-                &[4096],
+                &lengths_on_the_host(&[4096]),
                 CHECKSUM_CHUNK_FOOTPRINT,
-                bounds(Some(bytes_held_with_the_staging - 1))
+                bounds(Some(bytes_held_with_the_staging.0 - 1))
             ),
             Err(EntryLargerThanAChunkOnTheGpu {
                 entry_position: 0,
-                input_bytes: 4096
+                input_bytes: InputLengthOnTheHostInBytes(4096)
             }),
             "少一个字节就装不下"
         );
         let chunks = pieces_cut_to_fit_on_the_gpu(
-            &[4096],
+            &lengths_on_the_host(&[4096]),
             CHECKSUM_CHUNK_FOOTPRINT,
             bounds(Some(bytes_held_without_the_staging)),
         )
@@ -1241,7 +1730,7 @@
         let pieces_in_order: Vec<std::ops::Range<u64>> = chunks
             .iter()
             .flatten()
-            .map(|piece| piece.input_bytes.clone())
+            .map(|piece| piece.input_bytes.start.0..piece.input_bytes.end.0)
             .collect();
         assert_eq!(
             pieces_in_order,
@@ -1249,12 +1738,15 @@
             "4096 字节切成两片 2024 与一片 48"
         );
         for chunk in &chunks {
-            let input_bytes: u64 = chunk.iter().map(PieceOfAnEntry::input_byte_count).sum();
-            let held = CHECKSUM_CHUNK_FOOTPRINT
-                .bytes_held_by_a_chunk(input_bytes, u64::try_from(chunk.len()).expect("小数"));
+            let input_bytes: InputLengthOnTheHostInBytes =
+                chunk.iter().map(PieceOfAnEntry::input_byte_count).sum();
+            let held = CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
+                input_bytes,
+                EntryCountOfAChunk::of_this_many_entries(chunk.len()),
+            );
             assert!(
-                held <= bytes_held_without_the_staging,
-                "每一段连同暂存合占 {held} 字节不超过上限"
+                held <= GpuMemoryInBytes(bytes_held_without_the_staging),
+                "每一段连同暂存合占 {held:?} 字节不超过上限"
             );
         }
     }
@@ -1288,12 +1780,16 @@
             out_of_memory_creating_the_buffers_of_the_chunk(),
             "交回建缓冲那一次的错"
         );
-        assert_eq!(account.bytes_held(), 0, "冲掉待写的提交做完了才放");
+        assert_eq!(
+            account.bytes_held(),
+            GpuMemoryInBytes(0),
+            "冲掉待写的提交做完了才放"
+        );
         assert!(!account.has_a_submission_running_past_its_wait());
     }
 
     /// 冲掉待写的那一次空提交在上限之内没做完：已建的缓冲与暂存还挂在待写里占着卡上的显存，这一段的账照旧记着、记成还在跑的那次提交，
-    /// 交回等超过上限（外壳照它先等那次提交、再重交），不放账也不交回显存不足。
+    /// 交回冲掉待写等满了上限的那一种（外壳照它先等那次提交、再重交；算超时、不算交到了卡上），不放账也不交回显存不足。
     #[test]
     fn the_chunk_whose_buffers_were_not_created_stays_held_while_the_flush_of_the_pending_writes_is_running(
     ) {
@@ -1308,9 +1804,15 @@
             BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
             out_of_memory_creating_the_buffers_of_the_chunk(),
             FLUSHING_SUBMISSION,
-            Err(waited_past_the_limit.clone()),
+            Err(waited_past_the_limit),
+        );
+        assert_eq!(
+            reported,
+            GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit {
+                wait_limit: WAIT_LIMIT_UNDER_TEST,
+            },
+            "交回冲掉待写等满了上限的那一种"
         );
-        assert_eq!(reported, waited_past_the_limit, "交回等的那一次的错");
         assert_eq!(
             account.bytes_held(),
             BYTES_HELD_BY_THE_TIMED_OUT_CHUNK,
````

## crates/singlefs-background-gpu/src/region_record_kernels.rs

````diff
--- a/crates/singlefs-background-gpu/src/region_record_kernels.rs
+++ b/crates/singlefs-background-gpu/src/region_record_kernels.rs
@@ -52,6 +52,7 @@
 
 use singlefs_core::address::{CheckpointTxg, SlotNumber};
 use singlefs_core::allocator::AllocationRecord;
+use singlefs_core::background_compute::GpuMemoryInBytes;
 use singlefs_core::logged_operations::CompactionRegion;
 use singlefs_core::region_scan::{
     scan_region_over_the_identity_index, AllocationInRegion, CentralMappingEntry,
@@ -66,7 +67,8 @@
 use crate::live_slot_kernels::low_and_high_words_of_the_slot;
 use crate::memory_limit_and_chunks::{
     chunks_that_fit_on_the_gpu, input_buffer_bytes, pieces_cut_to_fit_on_the_gpu,
-    ChunkBoundsOfAGpu, ChunkFootprint, PieceOfAnEntry, SmallestPieceLargerThanTheRoom,
+    ChunkBoundsOfAGpu, ChunkFootprint, EntryCountOfAChunk, InputLengthOnTheHostInBytes,
+    PieceOfAnEntry, SmallestPieceLargerThanTheRoom,
     COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS,
     COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION,
 };
@@ -88,19 +90,20 @@
 /// 两份核的工作组都是 64 个调用（两份 WGSL 里的 `@workgroup_size(64)`），每条记录一个调用。
 pub const REGION_RECORD_INVOCATIONS_PER_WORKGROUP: u32 = 64;
 /// 参数缓冲：四个 `u32`。
-pub const REGION_RECORD_PARAMETER_BUFFER_BYTES: u64 = 16;
+pub const REGION_RECORD_PARAMETER_BUFFER_BYTES: GpuMemoryInBytes = GpuMemoryInBytes(16);
 pub const RECORD_DESCRIPTOR_WORDS: usize = 11;
-pub const RECORD_DESCRIPTOR_BYTES: u64 = 44;
+pub const RECORD_DESCRIPTOR_BYTES: GpuMemoryInBytes = GpuMemoryInBytes(44);
 /// 切开的单元头一片的输入字节数：两份核在头一片里读到的最远那个字节是码 2 索引节点最宽的头（86 + 2 × 255 字节，key 宽一个字节最大 255）
 /// 连同其后 29 字节预留位，共 625 字节，补到整字是 628。头一片之后的字节只进载荷 CRC，随便切。
-pub const UNIT_HEAD_PIECE_INPUT_BYTES: u64 = 628;
+pub const UNIT_HEAD_PIECE_INPUT_BYTES: InputLengthOnTheHostInBytes =
+    InputLengthOnTheHostInBytes(628);
 /// 映射 key 在核里占几个字：key 不超过这么多字的字节（[`KEY_BYTES_THE_KERNELS_HOLD`]），长了的这一批交不成。
 pub const KEY_WORDS: usize = 7;
 pub const KEY_BYTES_THE_KERNELS_HOLD: usize = 4 * KEY_WORDS;
 pub const MAPPING_ENTRY_WORDS: usize = 1 + KEY_WORDS + 6;
 pub const LOCATION_ITEM_WORDS: usize = 4;
 pub const REGION_RECORD_OUTCOME_WORDS: usize = 6 + KEY_WORDS;
-pub const REGION_RECORD_OUTCOME_BYTES: u64 = 52;
+pub const REGION_RECORD_OUTCOME_BYTES: GpuMemoryInBytes = GpuMemoryInBytes(52);
 /// 结果缓冲开算之前的填充字：结果的类只会被核写成认得的那几个。
 pub const REGION_RECORD_OUTCOME_NOT_WRITTEN_FILL_WORD: u32 = 0xFFFF_FFFF;
 
@@ -250,10 +253,11 @@
         .iter()
         .map(|piece| piece.bytes().map_or(0, <[u8]>::len))
         .sum();
-    let padded_bytes =
-        input_buffer_bytes(u64::try_from(total_unit_bytes).expect("usize 装得进 u64"));
+    let padded_bytes = input_buffer_bytes(InputLengthOnTheHostInBytes::of_this_many_bytes(
+        total_unit_bytes,
+    ));
     let mut input_bytes =
-        Vec::with_capacity(usize::try_from(padded_bytes).expect("一段的输入装得进内存"));
+        Vec::with_capacity(usize::try_from(padded_bytes.0).expect("一段的输入装得进内存"));
     let mut descriptor_words = Vec::with_capacity(pieces.len() * RECORD_DESCRIPTOR_WORDS);
     for piece in pieces {
         let byte_offset = u32::try_from(input_bytes.len())
@@ -294,7 +298,7 @@
         ]);
     }
     input_bytes.resize(
-        usize::try_from(padded_bytes).expect("一段的输入装得进内存"),
+        usize::try_from(padded_bytes.0).expect("一段的输入装得进内存"),
         0,
     );
     PackedRegionRecordChunk {
@@ -419,22 +423,22 @@
 
     /// 三个缓冲在卡上合占的字节数。
     #[must_use]
-    pub fn bytes_on_the_gpu(&self) -> u64 {
+    pub fn bytes_on_the_gpu(&self) -> GpuMemoryInBytes {
         let words = self.entry_words.len()
             + self.location_item_words.len()
             + self.key_sorted_entry_positions.len();
-        4 * u64::try_from(words).expect("装得进 u64")
+        GpuMemoryInBytes(4 * u64::try_from(words).expect("装得进 u64"))
     }
 
     /// 三个缓冲里最大的那一个的字节数：它也要装得进这张卡的绑定上限。
     #[must_use]
-    pub fn largest_buffer_bytes(&self) -> u64 {
+    pub fn largest_buffer_bytes(&self) -> GpuMemoryInBytes {
         let words = self
             .entry_words
             .len()
             .max(self.location_item_words.len())
             .max(self.key_sorted_entry_positions.len());
-        4 * u64::try_from(words).expect("装得进 u64")
+        GpuMemoryInBytes(4 * u64::try_from(words).expect("装得进 u64"))
     }
 }
 
@@ -451,15 +455,20 @@
 /// 同 [`region_record_chunk_footprint`]，映射的三个缓冲按合占的字节数给。
 #[must_use]
 pub fn region_record_chunk_footprint_of_mapping_table_bytes(
-    mapping_table_bytes: u64,
+    mapping_table_bytes: GpuMemoryInBytes,
 ) -> ChunkFootprint {
     ChunkFootprint {
         bytes_per_chunk_besides_the_entries: COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
-            * REGION_RECORD_PARAMETER_BUFFER_BYTES
-            + mapping_table_bytes_held_with_their_staging(mapping_table_bytes),
+            .bytes_of_the_copies(REGION_RECORD_PARAMETER_BUFFER_BYTES)
+            .plus(mapping_table_bytes_held_with_their_staging(
+                mapping_table_bytes,
+            )),
         bytes_per_entry_besides_its_input: COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
-            * RECORD_DESCRIPTOR_BYTES
-            + COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS * REGION_RECORD_OUTCOME_BYTES,
+            .bytes_of_the_copies(RECORD_DESCRIPTOR_BYTES)
+            .plus(
+                COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS
+                    .bytes_of_the_copies(REGION_RECORD_OUTCOME_BYTES),
+            ),
         bytes_per_entry_in_the_largest_entry_buffer: REGION_RECORD_OUTCOME_BYTES
             .max(RECORD_DESCRIPTOR_BYTES),
     }
@@ -470,9 +479,11 @@
 /// # Panics
 /// 乘出来超出 `u64`：映射的表照内存里的条目数算，字节数不过 `isize::MAX`，乘 2 装得进 `u64`。
 #[must_use]
-pub fn mapping_table_bytes_held_with_their_staging(mapping_table_bytes: u64) -> u64 {
+pub fn mapping_table_bytes_held_with_their_staging(
+    mapping_table_bytes: GpuMemoryInBytes,
+) -> GpuMemoryInBytes {
     COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
-        .checked_mul(mapping_table_bytes)
+        .checked_bytes_of_the_copies(mapping_table_bytes)
         .expect("映射的表照内存里的条目数算，字节数不过 isize::MAX，乘 2 装得进 u64")
 }
 
@@ -523,19 +534,37 @@
     }
 }
 
+/// 一片映射里几条条目（条目表的项数）：条数，与落点项数（[`MappingLocationItemCount`]）、一段里几条记录（[`EntryCountOfAChunk`]）不是一回事。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+struct MappingEntryCount(u64);
+
+/// 一片映射里几项落点（落点表的项数）：条数，与条目数（[`MappingEntryCount`]）不是一回事。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+struct MappingLocationItemCount(u64);
+
 /// 一片映射交给核时三张表合占的字节数与最大那一张的字节数：`entry_count` 条条目、`location_item_count` 项落点（都按 [`MappingTablesForTheKernels`]
 /// 的建法：一项都没有时照样放一项全 0 的）。
-fn mapping_table_bytes_of(entry_count: u64, location_item_count: u64) -> (u64, u64) {
-    singlefs_replay_prefix::gpu_gc_capacity_proofs::checked_gpu_mapping_table_bytes(
-        entry_count,
-        location_item_count,
+fn mapping_table_bytes_of(
+    entry_count: MappingEntryCount,
+    location_item_count: MappingLocationItemCount,
+) -> (GpuMemoryInBytes, GpuMemoryInBytes) {
+    let (table_bytes, largest_table_bytes) =
+        singlefs_replay_prefix::gpu_gc_capacity_proofs::checked_gpu_mapping_table_bytes(
+            entry_count.0,
+            location_item_count.0,
+        )
+        .expect("映射表的总字节数与最大绑定量装得进 u64");
+    (
+        GpuMemoryInBytes(table_bytes),
+        GpuMemoryInBytes(largest_table_bytes),
     )
-    .expect("映射表的总字节数与最大绑定量装得进 u64")
 }
 
 /// 一条记录交给核的输入字节数（读不到的是 0）。
-fn input_bytes_of_the_record<EntryKey>(record: &RegionRecordToClassify<'_, EntryKey>) -> u64 {
-    u64::try_from(record.unit_bytes.map_or(0, <[u8]>::len)).expect("装得进 u64")
+fn input_bytes_of_the_record<EntryKey>(
+    record: &RegionRecordToClassify<'_, EntryKey>,
+) -> InputLengthOnTheHostInBytes {
+    InputLengthOnTheHostInBytes::of_this_many_bytes(record.unit_bytes.map_or(0, <[u8]>::len))
 }
 
 /// 一片映射：位置项指进 `[first_location, location_past_the_slice)` 的条目按条目表的次序，位置项指到段外的换成指段里的那一条。
@@ -574,7 +603,7 @@
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct RegionRecordPlanOnAGpu {
     /// 每条记录从单元头起交给核的那一片的字节数，按记录的次序：没切开的是整个单元（读不到的是 0），切开的是 [`UNIT_HEAD_PIECE_INPUT_BYTES`]。
-    pub bytes_from_the_start_of_each_record: Vec<u64>,
+    pub bytes_from_the_start_of_each_record: Vec<InputLengthOnTheHostInBytes>,
     /// 切开的单元头一片之后的那几片，按交的次序切成的几段：片的 `entry_position` 是那条记录在这一批里的位置，`input_bytes` 从单元的头一个字节数起；
     /// 同一个单元的几片按字节的次序、各在前后不同的段里（后一片接着前一片算完的寄存器算）。没有切开的单元时是空。
     pub chunks_of_the_pieces_after_the_heads: Vec<Vec<PieceOfAnEntry>>,
@@ -600,7 +629,8 @@
     mapping_tables: &MappingTablesForTheKernels,
     room: ChunkBoundsOfAGpu,
 ) -> Result<RegionRecordPlanOnAGpu, SmallestPieceLargerThanTheRoom> {
-    let input_lengths: Vec<u64> = records.iter().map(input_bytes_of_the_record).collect();
+    let input_lengths: Vec<InputLengthOnTheHostInBytes> =
+        records.iter().map(input_bytes_of_the_record).collect();
     let whole_units_not_held = match mapping_slices_under_the_input_lengths(
         &input_lengths,
         mapping_entries,
@@ -625,7 +655,7 @@
     if positions_of_the_units_cut.is_empty() {
         return Err(whole_units_not_held);
     }
-    let bytes_from_the_start_of_each_record: Vec<u64> = input_lengths
+    let bytes_from_the_start_of_each_record: Vec<InputLengthOnTheHostInBytes> = input_lengths
         .iter()
         .map(|input_bytes| (*input_bytes).min(UNIT_HEAD_PIECE_INPUT_BYTES))
         .collect();
@@ -635,13 +665,15 @@
         mapping_tables,
         room,
     )?;
-    let bytes_after_the_heads: Vec<u64> = positions_of_the_units_cut
+    let bytes_after_the_heads: Vec<InputLengthOnTheHostInBytes> = positions_of_the_units_cut
         .iter()
-        .map(|position| input_lengths[*position] - UNIT_HEAD_PIECE_INPUT_BYTES)
+        .map(|position| input_lengths[*position].minus(UNIT_HEAD_PIECE_INPUT_BYTES))
         .collect();
     let chunks_of_the_pieces_after_the_heads = pieces_cut_to_fit_on_the_gpu(
         &bytes_after_the_heads,
-        region_record_chunk_footprint_of_mapping_table_bytes(mapping_table_bytes_of(0, 0).0),
+        region_record_chunk_footprint_of_mapping_table_bytes(
+            mapping_table_bytes_of(MappingEntryCount(0), MappingLocationItemCount(0)).0,
+        ),
         room,
     )
     .map_err(|smallest_piece| SmallestPieceLargerThanTheRoom {
@@ -654,8 +686,8 @@
             .into_iter()
             .map(|piece| PieceOfAnEntry {
                 entry_position: positions_of_the_units_cut[piece.entry_position],
-                input_bytes: UNIT_HEAD_PIECE_INPUT_BYTES + piece.input_bytes.start
-                    ..UNIT_HEAD_PIECE_INPUT_BYTES + piece.input_bytes.end,
+                input_bytes: UNIT_HEAD_PIECE_INPUT_BYTES.plus(piece.input_bytes.start)
+                    ..UNIT_HEAD_PIECE_INPUT_BYTES.plus(piece.input_bytes.end),
             })
             .collect()
     })
@@ -671,13 +703,15 @@
 /// 取大的。一个落点上不止一条条目的那一片随这一批的映射变，挂着时逐批判（[`region_record_plan_on_a_gpu`]）：一张卡这一批切不出就这一批不分给它、
 /// 活照卡的顺序填给别的卡（`crate::gpus_of_this_host::plans_of_a_batch_filling_the_gpus_in_order`）。
 #[must_use]
-pub fn bytes_of_the_smallest_region_record_piece() -> u64 {
-    let head_piece_with_one_entry =
-        region_record_chunk_footprint_of_mapping_table_bytes(mapping_table_bytes_of(1, 1).0)
-            .bytes_held_by_a_chunk(UNIT_HEAD_PIECE_INPUT_BYTES, 1);
-    let piece_after_the_head_of_one_byte =
-        region_record_chunk_footprint_of_mapping_table_bytes(mapping_table_bytes_of(0, 0).0)
-            .bytes_held_by_a_chunk(1, 1);
+pub fn bytes_of_the_smallest_region_record_piece() -> GpuMemoryInBytes {
+    let head_piece_with_one_entry = region_record_chunk_footprint_of_mapping_table_bytes(
+        mapping_table_bytes_of(MappingEntryCount(1), MappingLocationItemCount(1)).0,
+    )
+    .bytes_held_by_a_chunk(UNIT_HEAD_PIECE_INPUT_BYTES, EntryCountOfAChunk(1));
+    let piece_after_the_head_of_one_byte = region_record_chunk_footprint_of_mapping_table_bytes(
+        mapping_table_bytes_of(MappingEntryCount(0), MappingLocationItemCount(0)).0,
+    )
+    .bytes_held_by_a_chunk(InputLengthOnTheHostInBytes(1), EntryCountOfAChunk(1));
     head_piece_with_one_entry.max(piece_after_the_head_of_one_byte)
 }
 
@@ -694,7 +728,7 @@
 /// # Panics
 /// `mapping_tables` 不是 `mapping_entries` 建出来的。
 pub fn mapping_slices_under_the_input_lengths(
-    input_lengths: &[u64],
+    input_lengths: &[InputLengthOnTheHostInBytes],
     mapping_entries: &[CentralMappingEntry],
     mapping_tables: &MappingTablesForTheKernels,
     room: ChunkBoundsOfAGpu,
@@ -717,23 +751,28 @@
             }]);
         }
     }
-    let footprint_without_the_mapping = region_record_chunk_footprint_of_mapping_table_bytes(0);
+    let footprint_without_the_mapping =
+        region_record_chunk_footprint_of_mapping_table_bytes(GpuMemoryInBytes(0));
     let (largest_record_position, largest_record_input_bytes) = input_lengths
         .iter()
         .copied()
         .enumerate()
         .max_by_key(|(_position, input_bytes)| *input_bytes)
         .expect("上面判过有记录");
-    let largest_record_piece_bytes =
-        footprint_without_the_mapping.bytes_held_by_a_chunk(largest_record_input_bytes, 1);
-    let smallest_piece_with = |mapping_table_bytes: u64| SmallestPieceLargerThanTheRoom {
-        entry_position: largest_record_position,
-        smallest_piece_bytes: largest_record_piece_bytes
-            + mapping_table_bytes_held_with_their_staging(mapping_table_bytes),
-    };
-    let room_for_the_mapping_tables = room.memory_limit.map_or(u64::MAX, |memory_limit| {
-        memory_limit.0.saturating_sub(largest_record_piece_bytes)
-    });
+    let largest_record_piece_bytes = footprint_without_the_mapping
+        .bytes_held_by_a_chunk(largest_record_input_bytes, EntryCountOfAChunk(1));
+    let smallest_piece_with =
+        |mapping_table_bytes: GpuMemoryInBytes| SmallestPieceLargerThanTheRoom {
+            entry_position: largest_record_position,
+            smallest_piece_bytes: largest_record_piece_bytes.plus(
+                mapping_table_bytes_held_with_their_staging(mapping_table_bytes),
+            ),
+        };
+    let room_for_the_mapping_tables =
+        room.memory_limit
+            .map_or(GpuMemoryInBytes(u64::MAX), |memory_limit| {
+                GpuMemoryInBytes(memory_limit.0.saturating_sub(largest_record_piece_bytes.0))
+            });
     let record_fits_the_binding = input_buffer_bytes(largest_record_input_bytes)
         <= room.largest_storage_binding_in_bytes
         && footprint_without_the_mapping.bytes_per_entry_in_the_largest_entry_buffer
@@ -756,8 +795,10 @@
     let mut entries_in_the_slice: u64 = 0;
     let mut location_items_in_the_slice: u64 = 0;
     let fits = |entry_count: u64, location_item_count: u64| -> bool {
-        let (table_bytes, largest_table_bytes) =
-            mapping_table_bytes_of(entry_count, location_item_count);
+        let (table_bytes, largest_table_bytes) = mapping_table_bytes_of(
+            MappingEntryCount(entry_count),
+            MappingLocationItemCount(location_item_count),
+        );
         record_fits_the_binding
             && mapping_table_bytes_held_with_their_staging(table_bytes)
                 <= room_for_the_mapping_tables
@@ -789,7 +830,11 @@
         ) {
             if !fits(group_items, group_items) {
                 return Err(smallest_piece_with(
-                    mapping_table_bytes_of(group_items, group_items).0,
+                    mapping_table_bytes_of(
+                        MappingEntryCount(group_items),
+                        MappingLocationItemCount(group_items),
+                    )
+                    .0,
                 ));
             }
             if entries_in_the_slice != 0 {
@@ -809,7 +854,9 @@
         group_start = group_end;
     }
     if !fits(entries_in_the_slice, location_items_in_the_slice) {
-        return Err(smallest_piece_with(mapping_table_bytes_of(0, 0).0));
+        return Err(smallest_piece_with(
+            mapping_table_bytes_of(MappingEntryCount(0), MappingLocationItemCount(0)).0,
+        ));
     }
     let mut slices = Vec::with_capacity(slice_starts.len());
     for (slice_position, first_location) in slice_starts.iter().enumerate() {
@@ -940,9 +987,9 @@
                     .unit_bytes
                     .expect("切开的单元读到了：读不到的那一条交 0 字节，不比头一片长、不切");
                 let piece_start =
-                    usize::try_from(piece.input_bytes.start).expect("片在单元里，装得进 usize");
+                    usize::try_from(piece.input_bytes.start.0).expect("片在单元里，装得进 usize");
                 let piece_end =
-                    usize::try_from(piece.input_bytes.end).expect("片在单元里，装得进 usize");
+                    usize::try_from(piece.input_bytes.end.0).expect("片在单元里，装得进 usize");
                 RegionRecordPieceForTheKernels::PieceAfterTheHead {
                     record: records[piece.entry_position].record,
                     bytes: &unit_bytes[piece_start..piece_end],
@@ -982,12 +1029,13 @@
         .zip(&registers_of_the_bytes_after_the_head)
         .map(|((to_classify, bytes_from_the_start), registers)| {
             let bytes_after_this_piece = input_bytes_of_the_record(to_classify)
-                .checked_sub(*bytes_from_the_start)
+                .0
+                .checked_sub(bytes_from_the_start.0)
                 .expect("从单元头起的那一片不比单元长");
             RegionRecordPieceForTheKernels::UnitFromItsStart {
                 record: to_classify.record,
                 bytes: to_classify.unit_bytes.map(|unit_bytes| {
-                    &unit_bytes[..usize::try_from(*bytes_from_the_start)
+                    &unit_bytes[..usize::try_from(bytes_from_the_start.0)
                         .expect("不比单元长，装得进 usize")]
                 }),
                 bytes_of_the_unit_after_this_piece: u32::try_from(bytes_after_this_piece)
@@ -1739,7 +1787,9 @@
         assert_eq!((tables.entry_count, tables.location_item_count), (0, 0));
         assert_eq!(
             tables.bytes_on_the_gpu(),
-            4 * u64::try_from(MAPPING_ENTRY_WORDS + LOCATION_ITEM_WORDS + 1).expect("小数")
+            GpuMemoryInBytes(
+                4 * u64::try_from(MAPPING_ENTRY_WORDS + LOCATION_ITEM_WORDS + 1).expect("小数")
+            )
         );
     }
 
@@ -2146,16 +2196,15 @@
             .collect();
         let mapping_tables = MappingTablesForTheKernels::of_the_mapping_entries(&mapping_entries)
             .expect("key 都是 27 字节");
-        let one_entry_with_one_location_bytes = mapping_table_bytes_of(1, 1).0;
+        let one_entry_with_one_location_bytes =
+            mapping_table_bytes_of(MappingEntryCount(1), MappingLocationItemCount(1)).0;
         let memory_limit =
             region_record_chunk_footprint_of_mapping_table_bytes(one_entry_with_one_location_bytes)
-                .bytes_held_by_a_chunk(32768, 1);
+                .bytes_held_by_a_chunk(InputLengthOnTheHostInBytes(32768), EntryCountOfAChunk(1));
         let room = ChunkBoundsOfAGpu {
-            memory_limit: Some(singlefs_core::background_compute::GpuMemoryInBytes(
-                memory_limit,
-            )),
-            largest_storage_binding_in_bytes: 1 << 30,
-            largest_entry_count_per_dispatch: 1 << 20,
+            memory_limit: Some(memory_limit),
+            largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
+            largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
         };
         let plan = region_record_plan_on_a_gpu(&records, &mapping_entries, &mapping_tables, room)
             .expect("最大那条记录连同一个落点上的一条条目装得下");
@@ -2373,9 +2422,9 @@
         let mapping_tables = MappingTablesForTheKernels::of_the_mapping_entries(&mapping_entries)
             .expect("key 都是 27 字节");
         let room = ChunkBoundsOfAGpu {
-            memory_limit: Some(singlefs_core::background_compute::GpuMemoryInBytes(2048)),
-            largest_storage_binding_in_bytes: 1 << 30,
-            largest_entry_count_per_dispatch: 1 << 20,
+            memory_limit: Some(GpuMemoryInBytes(2048)),
+            largest_storage_binding_in_bytes: GpuMemoryInBytes(1 << 30),
+            largest_entry_count_per_dispatch: EntryCountOfAChunk(1 << 20),
         };
         let plan = region_record_plan_on_a_gpu(&records, &mapping_entries, &mapping_tables, room)
             .expect("切开之后头一片连同一个落点上的条目、之后的每一片都装得下 2 KiB");
@@ -2386,13 +2435,13 @@
                 UNIT_HEAD_PIECE_INPUT_BYTES,
                 UNIT_HEAD_PIECE_INPUT_BYTES,
                 UNIT_HEAD_PIECE_INPUT_BYTES,
-                0,
-                0,
+                InputLengthOnTheHostInBytes(0),
+                InputLengthOnTheHostInBytes(0),
                 UNIT_HEAD_PIECE_INPUT_BYTES,
             ],
             "读到的单元都切成头一片，读不到的交 0 字节"
         );
-        let pieces_after_the_head_of_the_index_node: u64 = plan
+        let pieces_after_the_head_of_the_index_node: InputLengthOnTheHostInBytes = plan
             .chunks_of_the_pieces_after_the_heads
             .iter()
             .flatten()
@@ -2401,7 +2450,7 @@
             .sum();
         assert_eq!(
             pieces_after_the_head_of_the_index_node,
-            16384 - UNIT_HEAD_PIECE_INPUT_BYTES,
+            InputLengthOnTheHostInBytes(16384).minus(UNIT_HEAD_PIECE_INPUT_BYTES),
             "索引节点头一片之后的几片一个字节不漏"
         );
         assert!(
@@ -2502,8 +2551,8 @@
                 .expect("key 都是 27 字节");
             assert_eq!(
                 mapping_table_bytes_of(
-                    u64::try_from(entries.len()).expect("小数"),
-                    expected_location_items
+                    MappingEntryCount(u64::try_from(entries.len()).expect("小数")),
+                    MappingLocationItemCount(expected_location_items)
                 ),
                 (tables.bytes_on_the_gpu(), tables.largest_buffer_bytes()),
                 "{} 条条目",
````

## crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs

````diff
--- a/crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs
+++ b/crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs
@@ -53,11 +53,18 @@
 fn first_entry_key_repeated_in_the_batch<EntryKey: Eq + Hash, KernelOutcome>(
     entries: &[TwoKernelOutcomesOfAnEntry<EntryKey, KernelOutcome>],
 ) -> Option<SameEntryKeyTwiceInOneBatch> {
+    first_entry_key_repeated_among(entries.iter().map(|entry| &entry.entry_key))
+}
+
+/// 一串条目键里头一个重复的（按次序，第二次出现的位置最小的那一个）；都不重复交回 `None`。交给两份核之前就查得了（选段打分那一段在核对整段清单之前查）。
+pub fn first_entry_key_repeated_among<'key, EntryKey: Eq + Hash + 'key>(
+    entry_keys: impl ExactSizeIterator<Item = &'key EntryKey>,
+) -> Option<SameEntryKeyTwiceInOneBatch> {
     let mut first_position_of_each_key: HashMap<&EntryKey, usize> =
-        HashMap::with_capacity(entries.len());
-    for (entry_position, entry) in entries.iter().enumerate() {
+        HashMap::with_capacity(entry_keys.len());
+    for (entry_position, entry_key) in entry_keys.enumerate() {
         if let Some(position_of_the_first_occurrence) =
-            first_position_of_each_key.insert(&entry.entry_key, entry_position)
+            first_position_of_each_key.insert(entry_key, entry_position)
         {
             return Some(SameEntryKeyTwiceInOneBatch {
                 position_of_the_first_occurrence,
````

## crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs

````diff
--- a/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs
+++ b/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs
@@ -9,7 +9,7 @@
 //! 连续不确定的计数不动（`two_kernel_reconciliation::ConsecutiveUncertaintyCounts::reconcile_unless_the_batch_failed`）。别的出错不重交。
 //!
 //! 开卡（起设备、编管线、量代价）每一次至多等 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]（[`outcome_of_opening_a_gpu_within_the_limit`]）；
-//! 一张卡开不成就隔 1 分钟重开、至多重开 3 次，都不成才算它开不成（用户定案，挂载那一刻点名的卡与挂着切换点到的新卡同一种：[`gpu_opened_with_the_reopenings`]）。
+//! 一张卡开不成就隔 1 分钟重开、至多重开 3 次，都不成才算它开不成（第 4 条，挂载那一刻点名的卡与挂着切换点到的新卡同一种：[`gpu_opened_with_the_reopenings`]）。
 //!
 //! 等法与钟、完成信号拆开：钟是 [`MonotonicClock`]（产品用这个进程的单调钟 [`SteadyClockOfThisProcess`]），完成信号是调用方给的
 //! 「看一次做没做完、最多等这么久」的闭包。不要 GPU 的用例换上假钟与假完成信号，钉住「上限内完成照常、超过上限交回超时出错」。
@@ -119,12 +119,19 @@
 /// 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次等超过上限就报这张卡用不了。
 pub const RESUBMISSIONS_AFTER_A_TIMEOUT: u32 = 3;
 
-/// 是不是等超过上限的那两种：这一次提交自己等超过上限，或交活之前、重交之前等前一次那一次又等满了上限。只有这两种重交、算进一批的超时次数。
+/// 是不是等超过上限的那三种：这一次提交自己等超过上限，交活之前、重交之前等前一次那一次又等满了上限，或建缓冲出错之后冲掉待写的那一次空提交等满了上限。
+/// 只有这三种重交、算进一批的超时次数。
 fn waited_past_the_limit(cause: &GpuFailureCause) -> bool {
     match cause {
         GpuFailureCause::WaitForTheGpuExceededTheLimit { .. }
-        | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. } => true,
+        | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
+        | GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { .. } => true,
         GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
+        | GpuFailureCause::OpeningThreadNotStarted { .. }
+        | GpuFailureCause::OpeningPanicked { .. }
+        | GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned
+        | GpuFailureCause::OpeningCalledOff
+        | GpuFailureCause::OpeningsInThisMountUsedUp { .. }
         | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
         | GpuFailureCause::WaitForTheSubmissionFailed { .. }
         | GpuFailureCause::ReadbackMappingFailed { .. }
@@ -136,7 +143,7 @@
 }
 
 /// 一批里一张卡上到此刻等超过上限了几次、重交了几次（重交只数真交出去的：重交之前那一等又等满了上限、没交的那一回只算超时，
-/// 重交那一次建缓冲显存不足、没交出去的那一回不算）（第 4 条「放弃的那一批在同一张卡上重交，至多 3 次，都超时才报警」按批、按卡数：
+/// 重交那一次建缓冲显存不足、或之后冲掉待写的那一次空提交等满了上限而没交出去的那一回不算）（第 4 条「放弃的那一批在同一张卡上重交，至多 3 次，都超时才报警」按批、按卡数：
 /// 一批切成几片、几段时各片的超时合起来算，交活之前等前一次超时的那次提交那一等也算）。一批开头每张卡各一份，从 0 起。
 /// 外壳的「一批」是三段各自交给 GPU 的那一次（`BackgroundGpu` 的三段方法各调一次是一批）。
 #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
@@ -186,14 +193,23 @@
     }
 }
 
-/// 一次交的结局说不说明那一次真交到了卡上：建缓冲显存不足（[`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]）是在交之前判出的、那一段没交；
+/// 一次交的结局说不说明那一次真交到了卡上：建缓冲出错（显存不足 [`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]，或之后冲掉待写的那一次空提交等满了上限
+/// [`GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit`]）是在交之前判出的、那一段没交；
 /// 别的出错都是交了之后才判出的（等提交、回读、读回的判定字），做完了的当然交了。只给交一段那一次的结局用（`submit_once` 交回的）。
 fn the_attempt_reached_the_gpu<Outcome>(attempt: &Result<Outcome, GpuFailureCause>) -> bool {
     match attempt {
         Ok(_outcome) => true,
-        Err(GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }) => false,
+        Err(
+            GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
+            | GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { .. },
+        ) => false,
         Err(
             GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
+            | GpuFailureCause::OpeningThreadNotStarted { .. }
+            | GpuFailureCause::OpeningPanicked { .. }
+            | GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned
+            | GpuFailureCause::OpeningCalledOff
+            | GpuFailureCause::OpeningsInThisMountUsedUp { .. }
             | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
             | GpuFailureCause::WaitForTheSubmissionFailed { .. }
             | GpuFailureCause::WaitForTheGpuExceededTheLimit { .. }
@@ -282,10 +298,11 @@
 /// 量代价出错而账上还记着在跑的提交的那张卡，在 `open` 里已经泄漏、这一次占着的字节已记进这个进程的泄漏账，交回的只是那个错。
 ///
 /// # Errors
-/// `open` 交回的出错；到点 ⇒ `WaitForTheGpuExceededTheLimit`；起不了那条线程 ⇒ [`GpuFailureCause::DeviceOrPipelineDidNotStart`]。
+/// `open` 交回的出错；到点 ⇒ `WaitForTheGpuExceededTheLimit`；起不了那条线程 ⇒ [`GpuFailureCause::OpeningThreadNotStarted`]。
 ///
 /// # Panics
-/// `open` panic 了：在这条线程上照原样再 panic（与不隔一条线程时相同）。
+/// `open` panic 了：在这条线程上照原样再 panic（与不隔一条线程时相同；挂载与切换共用的开一张卡那一步接住它，
+/// `gpus_of_this_host::gpu_opened_under_its_memory_limit`）。
 pub fn outcome_of_opening_a_gpu_within_the_limit<Opened: Send + 'static>(
     opening_limit: Duration,
     open: impl FnOnce() -> Result<Opened, GpuFailureCause> + Send + 'static,
@@ -297,7 +314,7 @@
             // 送不出去只有一种：等它的那一头到点不等了、已经丢了，这次开卡的结局没人要。
             let _nobody_waits_for_this_opening_any_more = outcome_sender.send(open());
         })
-        .map_err(|error| GpuFailureCause::DeviceOrPipelineDidNotStart {
+        .map_err(|error| GpuFailureCause::OpeningThreadNotStarted {
             reason: format!("起不了开卡的那条线程：{error}"),
         })?;
     match outcome_receiver.recv_timeout(opening_limit) {
@@ -315,10 +332,10 @@
     }
 }
 
-/// 一张卡开不成之后隔多久重开（用户定案「间隔 1 分钟重开」，挂载那一刻点名的卡与挂着切换点到的新卡同一个数）。
+/// 一张卡开不成之后隔多久重开（第 4 条「间隔 1 分钟重开」，挂载那一刻点名的卡与挂着切换点到的新卡同一个数）。
 pub const INTERVAL_BEFORE_REOPENING_A_GPU_NOT_OPENED: Duration = Duration::from_secs(60);
 
-/// 一张卡开不成之后至多重开几次（用户定案「最多重开 3 次」）：头一次开不成之后再开这么多次，都不成才算这张卡开不成。
+/// 一张卡开不成之后至多重开几次（第 4 条「最多重开 3 次」）：头一次开不成之后再开这么多次，都不成才算这张卡开不成。
 pub const REOPENINGS_OF_A_GPU_NOT_OPENED_AT_MOST: u32 = 3;
 
 /// 一张卡开不成时怎么重开：头一次开不成就等 `interval` 再开，至多再开 `reopenings_at_most` 次，都不成才算这张卡开不成
@@ -336,6 +353,13 @@
         reopenings_at_most: REOPENINGS_OF_A_GPU_NOT_OPENED_AT_MOST,
     };
 
+    /// 一张卡至多开几次：头一次加至多重开的次数（第 4 条「头一次开卡不算重开，一张卡至多开 4 次」）。挂着切换时一张卡在一次挂载里连着开不成到这个数（开成一次清零），
+    /// 再被点到就不再开（`background_gpu::BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`）。
+    #[must_use]
+    pub const fn openings_at_most(self) -> u32 {
+        self.reopenings_at_most.saturating_add(1)
+    }
+
     /// 连同重开，一张卡至多要等多久才有结局：每一次开卡至多 `each_opening`，开 1 + 重开次数 次，中间隔重开次数个间隔；超出 `Duration` 取它的最大值。
     #[must_use]
     pub const fn longest_wait_of_one_gpu(self, each_opening: Duration) -> Duration {
@@ -376,7 +400,7 @@
     use crate::checksum_kernels::{
         checksum_outcomes_of_the_units_cut_into_pieces, ChecksumKernelOutcome, UnitChecksumToVerify,
     };
-    use crate::memory_limit_and_chunks::PieceOfAnEntry;
+    use crate::memory_limit_and_chunks::{InputLengthOnTheHostInBytes, PieceOfAnEntry};
     use crate::two_kernel_reconciliation::{
         ConsecutiveUncertaintyCounts, SameEntryKeyTwiceInOneBatch, TwoKernelOutcomesOfAnEntry,
     };
@@ -641,7 +665,7 @@
             .map(|entry_position| {
                 vec![PieceOfAnEntry {
                     entry_position,
-                    input_bytes: 0..8,
+                    input_bytes: InputLengthOnTheHostInBytes(0)..InputLengthOnTheHostInBytes(8),
                 }]
             })
             .collect();
@@ -940,7 +964,7 @@
     #[test]
     fn the_resubmission_short_of_memory_for_its_buffers_is_not_counted_as_a_resubmission() {
         let short_of_memory = GpuFailureCause::OutOfMemoryCreatingTheBuffers {
-            bytes_held_by_the_chunk: 2000,
+            bytes_held_by_the_chunk: singlefs_core::background_compute::GpuMemoryInBytes(2000),
             reason: "用例造的：重交那一次卡上空闲的不够".to_string(),
         };
         let mut gpu = GpuWithScriptedSubmissions::scripted(
@@ -964,6 +988,39 @@
         );
     }
 
+    /// 头一次交等超过上限，重交之前那一等等到了，重交那一次建缓冲出错、冲掉待写的那一次空提交等满了上限（这一段没交到卡上），
+    /// 再一回等到、重交做完——超时 2 次（冲掉待写那一回算一次），「重交了几次」只记真交到卡上的那 1 次。
+    #[test]
+    fn the_flush_of_the_pending_writes_timing_out_counts_as_a_timeout_and_not_as_a_resubmission() {
+        let flush_timed_out =
+            GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit {
+                wait_limit: WAIT_LIMIT_OF_THE_RESUBMISSIONS,
+            };
+        let mut gpu = GpuWithScriptedSubmissions::scripted(
+            vec![
+                Err(this_submission_timed_out()),
+                Err(flush_timed_out),
+                Ok(9),
+            ],
+            vec![Ok(()), Ok(())],
+        );
+        let (outcome, timeouts_of_the_batch) = outcome_and_timeouts_of_one_submission(&mut gpu);
+        assert_eq!(
+            outcome,
+            Ok(9),
+            "冲掉待写等不到照重交的规矩往下走，第三回做完"
+        );
+        assert_eq!(
+            (
+                gpu.submissions_made,
+                timeouts_of_the_batch.timeouts(),
+                timeouts_of_the_batch.resubmissions()
+            ),
+            (3, 2, 1),
+            "交了 3 回，超时 2 次，冲掉待写等不到的那一回没交到卡上：重交记 1"
+        );
+    }
+
     /// 编管线那一步卡住的替身：卡 `stuck_for` 那么久之后才交回开成了（`OPENED_AFTER_BEING_STUCK`）。
     const OPENED_AFTER_BEING_STUCK: u32 = 7;
 
````

## crates/singlefs-core/src/background_compute.rs

````diff
--- a/crates/singlefs-core/src/background_compute.rs
+++ b/crates/singlefs-core/src/background_compute.rs
@@ -70,9 +70,53 @@
 }
 
 /// 显存的字节数：一张卡的显存总量，或 `background_gpu_memory_limit` 里给一张卡的上限（第 3 条）。
+/// 后台 GPU 包切批与记账的字节数也用它（一段的占用、每条与每段的固定占用、卡上占着的、最小的一片）：都是卡上显存的字节，与上限直接比。
+/// 份数、条数各是各的类型（后台 GPU 包的 `memory_limit_and_chunks`），乘份数在那边算。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
 pub struct GpuMemoryInBytes(pub u64);
 
+impl GpuMemoryInBytes {
+    /// 两份显存字节数之和；超出 `u64` 交回 `None`。
+    #[must_use]
+    pub const fn checked_add(self, more: Self) -> Option<Self> {
+        match self.0.checked_add(more.0) {
+            Some(sum_in_bytes) => Some(Self(sum_in_bytes)),
+            None => None,
+        }
+    }
+
+    /// 扣掉 `less` 之后剩下的；`less` 比它多交回 `None`。
+    #[must_use]
+    pub const fn checked_sub(self, less: Self) -> Option<Self> {
+        match self.0.checked_sub(less.0) {
+            Some(difference_in_bytes) => Some(Self(difference_in_bytes)),
+            None => None,
+        }
+    }
+
+    /// 两份显存字节数之和，用在和装得进 `u64` 的地方（各段固定占用的常量、一段真建的缓冲合起来）。
+    ///
+    /// # Panics
+    /// 和超出 `u64`。
+    #[must_use]
+    pub const fn plus(self, more: Self) -> Self {
+        match self.checked_add(more) {
+            Some(sum) => sum,
+            None => panic!("显存字节数之和超出 u64"),
+        }
+    }
+}
+
+/// 几份显存字节数合起来（一段真建的每一份缓冲加起来）。
+///
+/// # Panics
+/// 和超出 `u64`。
+impl std::iter::Sum for GpuMemoryInBytes {
+    fn sum<Parts: Iterator<Item = Self>>(parts: Parts) -> Self {
+        parts.fold(Self(0), Self::plus)
+    }
+}
+
 /// 本机有哪些显卡、每张多少显存：只读。GPU 那一侧（住哪是七·2 的岔路）实现它，测试里一份假清单实现它。
 /// 挂载与切换的校验只读它；没有这一份清单（`None`）就是 GPU 不在。
 pub trait GpuInventory {
@@ -166,7 +210,8 @@
     }
 }
 
-/// 实现算一段时出的错（第 4 条「停下并报错」）。核心层对三种做的是同一件事：这一批放弃、意图留在盘上、那个后台任务停下并报这个错，不换回 CPU；
+/// 实现算一段时出的错（第 4 条「停下并报错」）。核心层对归卡、归输入的那几种做的是同一件事：这一批放弃、意图留在盘上、那个后台任务停下并报这个错，不换回 CPU；
+/// 一批里同一个条目键出现两次（[`Self::SameEntryKeyTwiceInOneBatch`]）只放弃这一批，后台任务不停、不报警，下一批照常算（第 8 条）。
 /// 成员按停下之后该怎么处置分（修好或不再点那张卡、调大等 GPU 的上限、切回 `cpu`），出在哪张卡、什么原因带在成员里。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum BackgroundGpuComputeFailed {
@@ -183,10 +228,11 @@
     },
     /// 这一批的输入 GPU 那一侧第一版不支持，一张卡都没交活：切回 `cpu`（装不进一段的那一种也可以调大显存上限）。
     InputNotSupportedOnTheGpu(InputNotSupportedOnTheGpu),
-    /// 挂着切换点到的这张卡开不成（起不了设备、编不了管线、量代价出错或超时），隔一段时间重开到上限也都没开成（用户定案「开不成就间隔 1 分钟重开，
-    /// 最多重开 3 次……3 次都不成：GC 停下，报警并报出哪张卡、每次开不成的原因，等人处理，不自动再试」）：修好它或切换时不再点它，或切回 `cpu`。
-    /// `failures_of_each_opening` 是每一次开不成照这张卡出错报成的那一个（[`Self::GpuFailed`] 或 [`Self::WaitForTheGpuExceededTheLimit`]），
-    /// 按开的次序，头一次的在前。
+    /// 挂着切换点到的这张卡开不成（起不了设备、编不了管线、量代价出错或超时、起不了开卡线程、开卡 panic、前一次开卡没回来），隔一段时间重开到上限也都没开成
+    /// （第 4 条「间隔 1 分钟重开，最多重开 3 次……3 次都不成，GC 停下，报警里报出哪张卡、每次开不成的原因，等人处理，不自动再试」），
+    /// 或这张卡在这一次挂载里已连着开了 4 次都没开成（开成一次清零）、这一次不再开（[`GpuFailureOnTheDevice::OpeningsInThisMountUsedUp`]）：修好它或切换时不再点它，或切回 `cpu`；
+    /// 连着试满的那一种要卸载重挂。`failures_of_each_opening` 是每一次开不成照这张卡出错报成的那一个（[`Self::GpuFailed`] 或
+    /// [`Self::WaitForTheGpuExceededTheLimit`]），按开的次序，头一次的在前；连着试满、这一次不再开的只有一个。
     GpuNotOpenedAfterTheReopenings {
         device: GpuDeviceIdentifier,
         failures_of_each_opening: Vec<BackgroundGpuComputeFailed>,
@@ -215,7 +261,8 @@
         gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
     },
     /// 交进这一批的条目里同一个条目键出现了两次（两份核比对之前查出，连续不确定的计数一条不动、CPU 参照一次都不调）：是调用方组批的 bug，
-    /// 不归哪张卡。两个位置是它在这一批里先后两次出现的位置。
+    /// 不归哪张卡。两个位置是它在这一批里先后两次出现的位置。核心层收到它只放弃这一批：后台任务不停、不报警，下一批照常算
+    /// （[`BackgroundComputeError::reach_of_the_error`]）。
     SameEntryKeyTwiceInOneBatch {
         position_of_the_first_occurrence: usize,
         position_of_the_second_occurrence: usize,
@@ -266,8 +313,8 @@
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct GpuShortOfFreeMemoryForTheSmallestPiece {
     pub device: GpuDeviceIdentifier,
-    pub bytes_of_the_last_chunk_not_created: u64,
-    pub smallest_piece_bytes: u64,
+    pub bytes_of_the_last_chunk_not_created: GpuMemoryInBytes,
+    pub smallest_piece_bytes: GpuMemoryInBytes,
 }
 
 /// 一张卡因显存上限装不下最小的一片而不用：哪张卡、给它的上限、最小的一片要多少字节，
@@ -278,15 +325,27 @@
 pub struct GpuLeftOutUnderItsMemoryLimit {
     pub device: GpuDeviceIdentifier,
     pub memory_limit: GpuMemoryInBytes,
-    pub smallest_piece_bytes: u64,
-    pub bytes_already_held_on_the_device: u64,
+    pub smallest_piece_bytes: GpuMemoryInBytes,
+    pub bytes_already_held_on_the_device: GpuMemoryInBytes,
 }
 
 /// 一张卡出错的原因（[`BackgroundGpuComputeFailed::GpuFailed`] 带着）。`reason` 是 GPU 那一侧报的原话。封闭集合。
+/// 开卡那几种（起不了设备或编不了管线、起不了开卡线程、开卡 panic、前一次开卡没回来、叫停、连着试满）只在一张卡开不成的原因里出现
+/// （[`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`] 的 `failures_of_each_opening`），出路各不相同。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum GpuFailureOnTheDevice {
-    /// 起不了设备或编不了管线（驱动不对、显存不够）。
+    /// wgpu 起不了这张卡的设备或编不了管线（驱动不对、显存不够；量代价那一段最小的一片装不进绑定上限也报这一种）。
     DeviceOrPipelineDidNotStart { reason: String },
+    /// 系统起不了开卡的那条线程（`reason` 是起线程报的原话）：卡本身没碰过，查进程的线程与内存额度。
+    OpeningThreadNotStarted { reason: String },
+    /// 开卡那一步里 panic 了（`said_by_the_panic` 是 panic 带的原话；驱动里的与本包自己的断言都算）：照这一次没开成重开。
+    OpeningPanicked { said_by_the_panic: String },
+    /// 同一张卡上前一次开卡到点之后还没回来（那条线程还挂在驱动里），这一次不再起：出路只有重启进程。
+    EarlierOpeningOfTheDeviceNotReturned,
+    /// 开卡在开完之前叫停了（卸载、切到 `cpu`）：这一次没开，结局没人收。
+    OpeningCalledOff,
+    /// 这张卡在这一次挂载里已连着开了 `failed_openings` 次都没开成（开成一次清零），切换再点到它不再开（第 4 条：一张卡至多开 4 次）：要卸载重挂。
+    OpeningsInThisMountUsedUp { failed_openings: u32 },
     /// GPU 那一侧报了没接住的错（校验错、显存不够）或设备丢了。
     UncapturedErrorOrDeviceLost { reason: String },
     /// 等一次提交没等成（不是超时）。
@@ -302,7 +361,7 @@
     /// 「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」），切到最小的一片都建不成就这一批不用这张卡
     /// （[`BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`]），不报出来；报出来的是挂载那一刻量代价时遇到的。
     OutOfMemoryCreatingTheBuffers {
-        bytes_held_by_the_chunk: u64,
+        bytes_held_by_the_chunk: GpuMemoryInBytes,
         reason: String,
     },
 }
@@ -314,6 +373,8 @@
     SubmissionOfThisBatch,
     /// 这张卡上前一次等超过上限的那一次：这一批在这张卡上交活之前又等满了上限它还没做完（它的缓冲还占着卡上的显存，第 3 条）。
     EarlierTimedOutSubmissionStillRunning,
+    /// 建一段的缓冲出错之后冲掉队列待写的那一次空提交：等满上限它还没做完（已建的那几份缓冲连同暂存还占着卡上的显存，第 3 条）。这一段没交到卡上。
+    EmptySubmissionFlushingThePendingWrites,
 }
 
 /// 第一版不支持的输入（[`BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu`] 带着）：条款只写了「一批放不下就切成几次交」，
@@ -322,12 +383,12 @@
 pub enum InputNotSupportedOnTheGpu {
     /// 分到活的那张卡这一批连最小的一片都装不下，而装不下的不是显存上限（给的上限扣掉卡上已占的连最小的一片都装不下的卡，这一批不分给它，
     /// 第 3 条；参与的卡全都这样才报 [`BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit`]）：是单个缓冲的绑定上限。
-    /// `entry_position` 是要切的那一条在这一批里的位置；`input_bytes` 是那最小的一片（一个字节的输入、或乙段一个落点上的全部条目那一片，补到整字）
-    /// 连同一段的固定占用合占的字节数，即这张卡至少要剩这么多（产品 GPU 包按这个数填）。
+    /// `entry_position` 是要切的那一条在这一批里的位置；`smallest_piece_bytes` 是那最小的一片（一个字节的输入、或乙段一个落点上的全部条目那一片，补到整字）
+    /// 连同一段的固定占用合占的字节数，即这张卡至少要剩这么多（产品 GPU 包按这个数填）：是卡上显存的字节，不是主机上那一条输入的长度。
     EntryLargerThanAChunkOnTheDevice {
         device: GpuDeviceIdentifier,
         entry_position: usize,
-        input_bytes: u64,
+        smallest_piece_bytes: GpuMemoryInBytes,
     },
     /// 映射里有一条 key 比核装得下的长：一张卡都还没分活就判出，不归哪张卡。`entry_position` 是它在映射条目里的位置。
     MappingKeyLongerThanTheKernelsHold {
@@ -429,8 +490,9 @@
     PutInEffect {
         failure_under_the_settings: Option<BackgroundGpuComputeFailed>,
     },
-    /// 它点到的一张卡重开到上限都没开成（[`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]）：核心层把有 GPU 路径的后台任务当场记成出错停着、
-    /// 报警，不自动再试，直到下一次通过校验的切换。
+    /// 它点到的一张卡重开到上限都没开成，或这张卡在这一次挂载里已连着试满、不再开（[`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]）：
+    /// 这一组不生效，实现照旧照交来之前那一组（核心层生效的那一组）；核心层把有 GPU 路径的后台任务当场记成出错停着、报警，不自动再试，
+    /// 直到下一次通过校验的切换。
     NotOpened { failure: BackgroundGpuComputeFailed },
 }
 
@@ -842,7 +904,7 @@
 
     /// 把这一组交给它带着的实现：选 `gpu` 时交这一组（[`BackgroundGpuCompute::use_the_settings_from_the_next_batch`]），从下一批起照它跑
     /// （点到没开的卡时实现开完才换上）；选 `cpu` 而带着实现时告诉它一张卡都不参与（[`BackgroundGpuCompute::use_no_gpu_from_the_next_batch`]，
-    /// 用户定案「切到 cpu 时开着的卡全部关掉」）；选 `cpu` 而没带实现时什么都不做。
+    /// 第 2 条「从 gpu 切到 cpu……关掉全部开着的卡」）；选 `cpu` 而没带实现时什么都不做。
     ///
     /// # Panics
     /// 选 `gpu` 而没带实现：选 `gpu` 的那一组在校验那一刻就带上实现。
@@ -1190,12 +1252,45 @@
     },
     /// 挂载那一刻、或挂着切换时选 `gpu`，实现答交给它的那一组换上了而照它一张卡都用不上（[`LatestGpuSettingsHandedIn::PutInEffect`] 带着的出错），
     /// 或那一组点到的卡在会话锁外重开到上限都没开成（[`LatestGpuSettingsHandedIn::NotOpened`]，报的是
-    /// [`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]）：
+    /// [`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]；那一组不生效，生效的照旧是切换之前那一组）：
     /// 挂载照常做成、切换照收，有 GPU 路径的后台任务当场停着、报这个错，不等那一组下的头一批交回出错。
     GpuUnusableUnderTheSettingsHandedIn { failure: BackgroundGpuComputeFailed },
 }
 
+/// 一批算着出的错管到哪（[`BackgroundComputeError::reach_of_the_error`]）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ReachOfTheComputeError {
+    /// 这一批放弃，那个后台任务停下、每次叫起报警（第 4 条）。
+    StopsTheTask,
+    /// 只放弃这一批：后台任务不停、不报警，下一批照常算（一批里同一个条目键出现两次，调用方组批的 bug，不归哪张卡）。
+    AbandonsOnlyTheBatch,
+}
+
 impl BackgroundComputeError {
+    /// 这个错管到哪：实现交回一批里同一个条目键出现两次的只放弃这一批，别的都停下那个后台任务。
+    #[must_use]
+    pub fn reach_of_the_error(&self) -> ReachOfTheComputeError {
+        match self {
+            BackgroundComputeError::GpuSegmentFailed {
+                failure: BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch { .. },
+                ..
+            } => ReachOfTheComputeError::AbandonsOnlyTheBatch,
+            BackgroundComputeError::InjectedByTheTestOnlyHook
+            | BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn { .. }
+            | BackgroundComputeError::GpuSegmentFailed {
+                failure:
+                    BackgroundGpuComputeFailed::GpuFailed { .. }
+                    | BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit { .. }
+                    | BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(..)
+                    | BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings { .. }
+                    | BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions { .. }
+                    | BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit { .. }
+                    | BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece { .. },
+                ..
+            } => ReachOfTheComputeError::StopsTheTask,
+        }
+    }
+
     /// 这个错点名的卡（[`BackgroundGpuComputeFailed::gpu_devices_named`]）；只供测试的钩子注入的不归哪张卡，是空。
     #[must_use]
     pub fn gpu_devices_named(&self) -> Vec<GpuDeviceIdentifier> {
@@ -1215,7 +1310,7 @@
 /// 都是停下之后头一次叫起报 1。
 /// 挂载那一刻就停着的、挂着切换生效时因一张卡都用不上停着的，头一次叫起（切换生效的那一次）算第 1 次。
 /// 这一次叫起在 CPU 上补跑了一批还是照旧停着、此刻 GPU 用不用得上都照报，直到一次通过校验的切换解除停下。
-#[must_use = "后台任务因 GPU 出错停着：每次叫起都要把这份报警告诉文件系统的用户（用户 2026-10-30 定）"]
+#[must_use = "后台任务因 GPU 出错停着：每次叫起都要把这份报警告诉文件系统的用户（第 4 条：通道是返回值里必须处理的报警）"]
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct AlarmOfABackgroundTaskStoppedAfterAComputeError {
     pub task: BackgroundTask,
@@ -1316,20 +1411,30 @@
 pub enum BackgroundTaskAfterTheComputeError {
     /// 停着，等一次通过校验的切换请求解除。
     LeftStopped,
-    /// 出错之前就挂着一次待生效的切换：它在停下那一刻也算停着时收到的切换请求，当场解除停下，从下一批起用它。
+    /// 停着：出错时交给了实现的那一组还在开卡。每次叫起报警、照阈值在 CPU 上补跑；实现答那一组换上了的那一刻解除，答开不成就照开不成停着；
+    /// 一次通过校验的切换请求照样解除。
+    LeftStoppedWhileTheSwitchOpensItsCards,
+    /// 出错之前就挂着一次待生效、还没交给实现的切换：它在停下那一刻也算停着时收到的切换请求，当场解除停下，从下一批起用它。
     StopLiftedByTheSwitchRequestedBeforeTheError,
+    /// 一批里同一个条目键出现两次（[`ReachOfTheComputeError::AbandonsOnlyTheBatch`]）：只放弃这一批，那个后台任务不停、不报警，下一批照常算。
+    NotStoppedForOnlyTheBatchIsAbandoned,
 }
 
 /// 后台计算在这一次挂载里的状态：此刻生效的设置；已交给实现、实现还在会话锁外开卡而没生效的那一组；挂着时请求、从下一批开头起交给实现的那一组；
 /// 因一批出错停着的后台任务。生命周期一次挂载，只住内存。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct BackgroundComputeOfThisMount {
-    /// 此刻生效的那一组：一批照它定走哪条路。选 `gpu` 的一组在实现答换上了（[`LatestGpuSettingsHandedIn::PutInEffect`]）或开不成
-    /// （[`LatestGpuSettingsHandedIn::NotOpened`]，停着）那一刻才成为生效的；选 `cpu` 的一组交给实现那一刻就生效。
+    /// 此刻生效的那一组：一批照它定走哪条路。选 `gpu` 的一组在实现答换上了（[`LatestGpuSettingsHandedIn::PutInEffect`]）那一刻才成为生效的；
+    /// 实现答开不成（[`LatestGpuSettingsHandedIn::NotOpened`]）的那一组不生效，生效的照旧是切换之前那一组（实现照旧用的就是它）；
+    /// 选 `cpu` 的一组交给实现那一刻就生效。
     in_effect: AdmittedBackgroundComputeSettings,
-    /// 已交给实现、实现答还在为它开卡的那一组（选 `gpu`）：这期间照旧照 `in_effect` 跑，它算还没生效的切换请求
-    /// （第 4 条「出错之前已经收到、还没生效的切换请求，在停下那一刻也算数」）。
+    /// 已交给实现、实现答还在为它开卡的那一组（选 `gpu`）：这期间照旧照 `in_effect` 跑；这期间一批出错，后台任务停下、报警，
+    /// 实现答它换上了的那一刻解除（[`Self::stop_the_task_after_a_compute_error`]）。
     handed_in_while_the_implementation_opens_its_cards: Option<AdmittedBackgroundComputeSettings>,
+    /// 实现答开不成的那一组请求给的补跑阈值：那一组不生效而阈值随切换请求从下一批起生效（第 4 条），补跑那一判照它；
+    /// 生效的那一组换掉时清掉。没有是 `None`（照 `in_effect` 带的）。
+    gc_cpu_compensation_threshold_of_the_switch_not_opened:
+        Option<BackgroundGcCpuCompensationThreshold>,
     requested_from_the_next_batch: Option<AdmittedBackgroundComputeSettings>,
     stopped_after_a_compute_error: Vec<BackgroundTaskStoppedAfterAComputeError>,
     /// 停着的后台任务停下以来叫起了几次：头一次叫起时记下（一次叫起里一批出错停下的，停下那一刻就记下 0），停着的那几条换掉或清掉时一起清掉
@@ -1354,6 +1459,7 @@
         let mut background_compute = Self {
             in_effect: admitted_at_the_mount.clone(),
             handed_in_while_the_implementation_opens_its_cards: None,
+            gc_cpu_compensation_threshold_of_the_switch_not_opened: None,
             requested_from_the_next_batch: None,
             stopped_after_a_compute_error: Vec::new(),
             wakes_of_the_tasks_stopped: Vec::new(),
@@ -1402,21 +1508,29 @@
         self.compute_error_of_the_batch_under_way.take()
     }
 
-    /// 一批出错之后（第 4 条）：出错之前就挂着还没生效的切换（请求了还没交给实现，或交给了实现而实现还在为它开卡），就当场解除——
-    /// 那一次切换算停着时收到的请求，`task` 不记成停着，之后照常等它生效；没有就把 `task` 记成停着，等下一次通过校验的切换请求。停在一次叫起里的（[`Self::begin_a_wake`] 开过、还没结束），
+    /// 一批出错之后（第 4 条）：一批里同一个条目键出现两次的只放弃这一批（[`ReachOfTheComputeError::AbandonsOnlyTheBatch`]），`task` 不停、不报警；
+    /// 出错之前就挂着请求了、还没交给实现的切换，就当场解除——那一次切换算停着时收到的请求，`task` 不记成停着，之后照常等它生效；
+    /// 别的把 `task` 记成停着：交给了实现而实现还在为它开卡的，那一组换上的那一刻解除（[`Self::take_the_answer_on_the_settings_handed_in`]），
+    /// 都等下一次通过校验的切换请求。停在一次叫起里的（[`Self::begin_a_wake`] 开过、还没结束），
     /// 那一次叫起结束时报警不数这一趟（[`Self::alarm_at_the_end_of_a_wake`] 报 0）；停在可写挂载续做里的不是叫起，停下之后头一次叫起照数 1。不读写盘。
     pub(crate) fn stop_the_task_after_a_compute_error(
         &mut self,
         task: BackgroundTask,
         error: BackgroundComputeError,
     ) -> BackgroundTaskAfterTheComputeError {
-        if self.requested_from_the_next_batch.is_some()
-            || self
-                .handed_in_while_the_implementation_opens_its_cards
-                .is_some()
-        {
+        match error.reach_of_the_error() {
+            ReachOfTheComputeError::AbandonsOnlyTheBatch => {
+                return BackgroundTaskAfterTheComputeError::NotStoppedForOnlyTheBatchIsAbandoned;
+            }
+            ReachOfTheComputeError::StopsTheTask => {}
+        }
+        if self.requested_from_the_next_batch.is_some() {
             return BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError;
         }
+        let task_after_the_error = match self.handed_in_while_the_implementation_opens_its_cards {
+            Some(_) => BackgroundTaskAfterTheComputeError::LeftStoppedWhileTheSwitchOpensItsCards,
+            None => BackgroundTaskAfterTheComputeError::LeftStopped,
+        };
         if !self.is_stopped_after_a_compute_error(task) {
             assert!(
                 self.wakes_of_the_tasks_stopped
@@ -1441,7 +1555,7 @@
                 WakeOfTheBackgroundTasks::NoneUnderWay => {}
             }
         }
-        BackgroundTaskAfterTheComputeError::LeftStopped
+        task_after_the_error
     }
 
     /// 一次叫起开头（整理的显式入口每一趟、后台入口每一次判起跑，入口在任何读写之前就拒的那几判之后）：记下此刻有一次叫起在做，
@@ -1518,8 +1632,8 @@
         }
     }
 
-    /// 下一批读的那一组的补跑阈值：有请求了还没交给实现的取它，再看交给了实现而实现还在开卡的那一组，都没有取生效的
-    /// （第 4 条：阈值随切换请求一起换、从下一批起生效，不等开卡）。
+    /// 下一批读的那一组的补跑阈值：有请求了还没交给实现的取它，再看交给了实现而实现还在开卡的那一组，再看实现答开不成的那一组请求给的，
+    /// 都没有取生效的（第 4 条：阈值随切换请求一起换、从下一批起生效，不等开卡，开不成也照它）。
     fn gc_cpu_compensation_threshold_of_the_next_batch(
         &self,
     ) -> BackgroundGcCpuCompensationThreshold {
@@ -1528,8 +1642,9 @@
             .or(self
                 .handed_in_while_the_implementation_opens_its_cards
                 .as_ref())
-            .unwrap_or(&self.in_effect)
-            .gc_cpu_compensation_threshold
+            .map(|requested| requested.gc_cpu_compensation_threshold)
+            .or(self.gc_cpu_compensation_threshold_of_the_switch_not_opened)
+            .unwrap_or(self.in_effect.gc_cpu_compensation_threshold)
     }
 
     /// `task` 下一批跑不跑、怎么跑（第 4 条与）：没停着照设置走；GC 停着时池子最满的那一块盘的已分配占它的容量严格大于阈值
@@ -1635,10 +1750,11 @@
     }
 
     /// 两批之间：挂着请求、待生效的那一组有的话交给实现（[`Self::hand_the_settings_in`]），再问实现最近交给它的那一组此刻怎样了
-    /// （[`Self::take_the_answer_on_the_settings_handed_in`]）：选 `gpu` 的一组要等实现答换上了才生效，实现还在会话锁外开卡的这段时间照旧照切换之前那一组跑，
-    /// 这一组算还没生效的切换请求（这期间一批出错照它解除停下，[`Self::stop_the_task_after_a_compute_error`]）；答换上了就照挂载那一刻的停法判
-    /// （照它一张卡都用不上就把有 GPU 路径的后台任务当场记成出错停着，[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；用得上就不停）；
-    /// 答开不成就照「一张卡都用不上」一样停着、报那个错（用户定案「3 次都不成：GC 停下，报警……不自动再试」），直到下一次通过校验的切换。
+    /// （[`Self::take_the_answer_on_the_settings_handed_in`]）：选 `gpu` 的一组要等实现答换上了才生效，实现还在会话锁外开卡的这段时间照旧照切换之前那一组跑
+    /// （这期间一批出错，后台任务停下、报警，[`Self::stop_the_task_after_a_compute_error`]）；答换上了就照挂载那一刻的停法判
+    /// （照它一张卡都用不上就把有 GPU 路径的后台任务当场记成出错停着，[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；用得上就不停，
+    /// 开卡期间出错停着的也在这一刻解除）；答开不成，那一组不生效、生效的照旧是切换之前那一组，有 GPU 路径的后台任务照「一张卡都用不上」一样停着、报那个错
+    /// （第 4 条「3 次都不成，GC 停下，报警……不自动再试」），直到下一次通过校验的切换。
     /// 选 `cpu` 的一组交给实现那一刻就生效。
     /// 一批开头（[`Self::begin_a_batch`]）调；整理的两个入口在判「整理停没停」之前也调（显式入口一趟开头、后台入口判起不起跑之前），
     /// 切到一张卡都用不上的那一组之后，下一次进来在任何一个写之前就判出停着：不走删除续做、不取走「该跑了」，与挂载那一刻同。
@@ -1656,7 +1772,7 @@
         settings.hand_the_settings_to_the_implementation();
         match settings.gpu_settings() {
             None => {
-                self.in_effect = settings;
+                self.put_the_settings_in_effect(settings);
                 self.handed_in_while_the_implementation_opens_its_cards = None;
             }
             Some(_) => {
@@ -1665,9 +1781,16 @@
         }
     }
 
+    /// 换上生效的那一组（实现答换上了、或选 `cpu` 的一组交给实现那一刻）：开不成那一组留下的补跑阈值跟着清掉。
+    fn put_the_settings_in_effect(&mut self, settings: AdmittedBackgroundComputeSettings) {
+        self.in_effect = settings;
+        self.gc_cpu_compensation_threshold_of_the_switch_not_opened = None;
+    }
+
     /// 问实现最近交给它的那一组此刻怎样了（[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]）：
     /// 有交给了实现、还没生效的那一组就问它带着的实现，照答案办——还在开卡：不动；换上了：它生效，停着的那几条换成照它判出的
-    /// （[`tasks_stopped_when_no_gpu_is_usable_under_the_settings_put_in_effect`]）；开不成：它生效，有 GPU 路径的后台任务记成出错停着、报那个错。
+    /// （[`tasks_stopped_when_no_gpu_is_usable_under_the_settings_put_in_effect`]；开卡期间出错停着的就此解除）；开不成：它不生效、清掉，
+    /// 生效的照旧是切换之前那一组，有 GPU 路径的后台任务记成出错停着、报那个错，补跑阈值照它请求给的（[`Self::gc_cpu_compensation_threshold_of_the_next_batch`]）。
     /// 没有还没生效的那一组而生效的选 `gpu` 时也问生效那一组带着的实现（让它收下为更早几次交设置开完的卡），答案不用。
     fn take_the_answer_on_the_settings_handed_in(&mut self) {
         let Some(handed_in) = self
@@ -1686,7 +1809,7 @@
             Some(LatestGpuSettingsHandedIn::PutInEffect {
                 failure_under_the_settings,
             }) => {
-                self.in_effect = handed_in;
+                self.put_the_settings_in_effect(handed_in);
                 self.replace_the_tasks_stopped(
                     tasks_stopped_when_no_gpu_is_usable_under_the_settings_put_in_effect(
                         failure_under_the_settings.as_ref(),
@@ -1694,7 +1817,8 @@
                 );
             }
             Some(LatestGpuSettingsHandedIn::NotOpened { failure }) => {
-                self.in_effect = handed_in;
+                self.gc_cpu_compensation_threshold_of_the_switch_not_opened =
+                    Some(handed_in.gc_cpu_compensation_threshold);
                 self.replace_the_tasks_stopped(tasks_with_a_gpu_path_stopped_by(&failure));
             }
         }
@@ -1705,12 +1829,12 @@
     /// 实现答换上了而照它一张卡都用不上、或答开不成，就照挂载那一刻一样，有 GPU 路径的后台任务当场记成出错停着
     /// （[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；切换照收）；再判 `task` 这一批跑不跑（[`Self::whether_the_task_runs_the_next_batch`]，`fullest_device`
     /// 是调用方这一刻从分配器取的池子最满的那一块盘）——停着而不补跑就不开这一批；最后定这一批走哪条路：补跑的走 CPU、停下以来补跑的批数加一，
-    /// 别的按生效的那一组定（实现还在为交给它的那一组开卡时，生效的仍是切换之前那一组）。有还没生效的切换时没有停着的后台任务
-    /// （请求切换那一刻解除全部停下，停下那一刻挂着切换的不记成停着），所以先问再判与先判再问，只差实现答的那一组用不上 GPU 这一种。
+    /// 别的按生效的那一组定（实现还在为交给它的那一组开卡、或答了开不成时，生效的仍是切换之前那一组）。先问再判：停着的那几条照这一刻的答案换过
+    /// （换上了的那一刻解除开卡期间出错的停下，开不成的那一刻照开不成停着）。
     ///
     /// # Errors
-    /// `task` 停着、这一批不补跑 ⇒ [`BatchNotBegunWhileTheTaskStaysStopped`]：没有还没生效的切换时状态一样都没动；有的话那一组已交给了实现、
-    /// 实现照它答过，停着的是这一批开头照答案记下的那几条。
+    /// `task` 停着、这一批不补跑 ⇒ [`BatchNotBegunWhileTheTaskStaysStopped`]：没有还没生效的切换、实现也没答出新结局时状态一样都没动；
+    /// 有的话那一组已交给了实现、实现照它答过，停着的是这一批开头照答案记下的那几条。
     ///
     /// # Panics
     /// 上一批注入的错没被读走：注入只在一批的钩子里，那一批读完设置之后就读走。
@@ -1814,6 +1938,50 @@
 mod tests {
     use super::*;
 
+    #[test]
+    fn gpu_memory_in_bytes_checked_add_gives_the_sum_and_refuses_past_u64() {
+        assert_eq!(
+            GpuMemoryInBytes(600).checked_add(GpuMemoryInBytes(400)),
+            Some(GpuMemoryInBytes(1000))
+        );
+        assert_eq!(
+            GpuMemoryInBytes(u64::MAX).checked_add(GpuMemoryInBytes(1)),
+            None
+        );
+    }
+
+    #[test]
+    fn gpu_memory_in_bytes_checked_sub_gives_what_is_left_and_refuses_taking_more_than_there_is() {
+        assert_eq!(
+            GpuMemoryInBytes(1000).checked_sub(GpuMemoryInBytes(600)),
+            Some(GpuMemoryInBytes(400))
+        );
+        assert_eq!(
+            GpuMemoryInBytes(600).checked_sub(GpuMemoryInBytes(1000)),
+            None
+        );
+    }
+
+    #[test]
+    fn gpu_memory_in_bytes_plus_gives_the_sum() {
+        assert_eq!(
+            GpuMemoryInBytes(16).plus(GpuMemoryInBytes(40)),
+            GpuMemoryInBytes(56)
+        );
+    }
+
+    #[test]
+    #[should_panic(expected = "显存字节数之和超出 u64")]
+    fn gpu_memory_in_bytes_plus_past_u64_panics() {
+        let _sum = GpuMemoryInBytes(u64::MAX).plus(GpuMemoryInBytes(1));
+    }
+
+    #[test]
+    fn gpu_memory_in_bytes_summed_over_the_parts_is_their_total() {
+        let total: GpuMemoryInBytes = [8_u64, 16, 24].into_iter().map(GpuMemoryInBytes).sum();
+        assert_eq!(total, GpuMemoryInBytes(48));
+    }
+
     /// 第 1 条「凡有 GPU 路径的后台任务」：GPU 一张卡都用不上时停着的，正好是每一个有 GPU 路径的成员各一条，报同一个错、补跑的批数 0。
     /// 逐个成员穷举：加成员时这里的 match 编不过，逼着写它停不停。
     #[test]
@@ -1822,8 +1990,8 @@
             gpus_left_out: vec![GpuLeftOutUnderItsMemoryLimit {
                 device: GpuDeviceIdentifier::of_opaque_bytes(b"0000:01:00.0".to_vec()),
                 memory_limit: GpuMemoryInBytes(64),
-                smallest_piece_bytes: 972,
-                bytes_already_held_on_the_device: 0,
+                smallest_piece_bytes: GpuMemoryInBytes(972),
+                bytes_already_held_on_the_device: GpuMemoryInBytes(0),
             }],
         };
         let stopped = tasks_with_a_gpu_path_stopped_by(&failure);
@@ -1974,7 +2142,7 @@
     }
 
     // ── 切换点到没开的卡：交给实现之后、实现开完卡之前，切换还没生效（第 4 条「开成了，下一批起换成新设置」「出错之前已经收到、还没生效的切换请求，
-    // 在停下那一刻也算数」「报的是挂载那一刻同一个成员」；用户定案「切到 cpu 时开着的卡全部关掉」）──
+    // 在停下那一刻也算数」「报的是挂载那一刻同一个成员」；第 2 条「从 gpu 切到 cpu……关掉全部开着的卡」）──
 
     fn card_for_the_switch_tests(address: &[u8]) -> GpuDeviceIdentifier {
         GpuDeviceIdentifier::of_opaque_bytes(address.to_vec())
@@ -2108,8 +2276,8 @@
             gpus_left_out: vec![GpuLeftOutUnderItsMemoryLimit {
                 device: card_for_the_switch_tests(address),
                 memory_limit: GpuMemoryInBytes(64),
-                smallest_piece_bytes: 972,
-                bytes_already_held_on_the_device: 0,
+                smallest_piece_bytes: GpuMemoryInBytes(972),
+                bytes_already_held_on_the_device: GpuMemoryInBytes(0),
             }],
         }
     }
@@ -2165,11 +2333,11 @@
         batch.as_ref().ok().map(|under_way| under_way.route)
     }
 
-    /// 第 4 条「出错之前已经收到、还没生效的切换请求，在停下那一刻也算数，照它解除停下」：挂载点「0000:01:00.0」，请求切到没开的「0000:41:00.0」；
-    /// 下一批开头交给实现（实现开卡、照切换之前那一组跑），这一批在「0000:01:00.0」上出错——切换还没生效，照它解除停下；卡开成、换上之后那一批照常在 GPU 上开，
+    /// 交给了实现、实现还在开卡的期间一批出错：挂载点「0000:01:00.0」，请求切到没开的「0000:41:00.0」；下一批开头交给实现（实现开卡、照切换之前那一组跑），
+    /// 这一批在「0000:01:00.0」上出错——停下（不照还没生效的那一组解除）；卡开成、实现答换上了的那一批开头解除，那一批照常在 GPU 上开，
     /// 生效的是点「0000:41:00.0」的那一组。
     #[test]
-    fn a_batch_failing_on_the_card_before_the_switch_while_the_new_card_opens_is_lifted_by_that_switch(
+    fn a_batch_failing_on_the_card_before_the_switch_while_the_new_card_opens_stops_until_that_switch_is_put_in_effect(
     ) {
         let state = state_with_the_card_at_one_opened(None, None);
         let mut background_compute = mount_on_the_card_at_one(&state);
@@ -2199,10 +2367,10 @@
                 BackgroundTask::CompactionRelocation,
                 failure_on_the_card_at_one(),
             ),
-            BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError,
-            "出错之前已经收到、还没生效的切换照它解除停下"
+            BackgroundTaskAfterTheComputeError::LeftStoppedWhileTheSwitchOpensItsCards,
+            "开卡期间旧卡出错：停下"
         );
-        assert!(!background_compute
+        assert!(background_compute
             .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
         state.lock().expect("用例没在别处 panic 过").opening_ends = true;
         let batch_once_opened = background_compute.begin_a_batch(
@@ -2212,8 +2380,10 @@
         assert_eq!(
             route_of_the_batch(&batch_once_opened),
             Some(BackgroundComputeRoute::Gpu),
-            "换上之后那一批照常开"
+            "换上的那一刻解除：那一批照常开"
         );
+        assert!(!background_compute
+            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
         assert_eq!(
             background_compute
                 .in_effect()
@@ -2369,7 +2539,7 @@
             )));
     }
 
-    /// 用户定案「切到 cpu 时开着的卡全部关掉」：挂载点「0000:01:00.0」，切到 `cpu`——那一批开头告诉实现一张卡都不参与（替身关掉开着的卡），那一批在 CPU 上算。
+    /// 第 2 条「从 gpu 切到 cpu……关掉全部开着的卡」：挂载点「0000:01:00.0」，切到 `cpu`——那一批开头告诉实现一张卡都不参与（替身关掉开着的卡），那一批在 CPU 上算。
     #[test]
     fn the_switch_to_the_cpu_tells_the_implementation_that_no_gpu_takes_part() {
         let state = state_with_the_card_at_one_opened(None, None);
@@ -2395,7 +2565,7 @@
         );
     }
 
-    /// 用户定案「切到 cpu 时开着的卡全部关掉」之后切回 `gpu`：点的卡都当没开的卡开——开卡的这段时间生效的仍是 `cpu`，那一批在 CPU 上算；
+    /// 切到 `cpu`、开着的卡全部关掉之后切回 `gpu`：点的卡都当没开的卡开——开卡的这段时间生效的仍是 `cpu`，那一批在 CPU 上算；
     /// 开成、实现答换上了之后，下一批起在 GPU 上算。
     #[test]
     fn the_switch_back_to_the_gpu_runs_on_the_cpu_while_the_cards_open_and_on_the_gpu_once_put_in_effect(
@@ -2437,4 +2607,587 @@
             "开成之后下一批起在 GPU 上算"
         );
     }
+
+    // ── 实现照用例逐批摆的答法（两批之间那一问答什么由用例定），钉核心层在开不成、开卡期间出错、重复键三处的结局 ──
+
+    /// 替身的脚本：两批之间那一问答 `answer`；告知一张卡都不参与、被问那一问各记一次。
+    struct ScriptOfTheImplementation {
+        answer: LatestGpuSettingsHandedIn,
+        times_told_no_gpu_takes_part: u32,
+        times_asked_between_two_batches: u32,
+    }
+
+    /// 卡清单同 [`GpuComputeOpeningCardsWhenTheTestSays`]（「0000:01:00.0」「0000:41:00.0」），三段交回空，两批之间那一问照脚本答。
+    struct GpuComputeAnsweringAsTheScriptSays {
+        script: Arc<Mutex<ScriptOfTheImplementation>>,
+    }
+
+    impl GpuInventory for GpuComputeAnsweringAsTheScriptSays {
+        fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
+            vec![
+                (
+                    card_for_the_switch_tests(b"0000:01:00.0"),
+                    GpuMemoryInBytes(8 << 30),
+                ),
+                (
+                    card_for_the_switch_tests(b"0000:41:00.0"),
+                    GpuMemoryInBytes(8 << 30),
+                ),
+            ]
+        }
+    }
+
+    impl BackgroundGpuCompute for GpuComputeAnsweringAsTheScriptSays {
+        fn verify_unit_checksums(
+            &mut self,
+            _units: &[UnitChecksumToVerifyOnTheGpu<'_>],
+        ) -> Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed>
+        {
+            Ok(Vec::new())
+        }
+
+        fn classify_region_records(
+            &mut self,
+            _region: CompactionRegion,
+            _records: &[RegionRecordToClassifyOnTheGpu<'_>],
+            _mapping_entries: &[CentralMappingEntry],
+        ) -> Result<Vec<GpuEntryDecision<AllocationInRegion>>, BackgroundGpuComputeFailed> {
+            Ok(Vec::new())
+        }
+
+        fn count_live_slots_per_cluster_segment(
+            &mut self,
+            _allocation_records: &[AllocationRecord],
+            _unit_area: UnitAreaOfOneDevice,
+            _segments: &[ClusterSegmentEntryKey],
+        ) -> Result<Vec<GpuEntryDecision<u64>>, BackgroundGpuComputeFailed> {
+            Ok(Vec::new())
+        }
+
+        fn use_the_settings_from_the_next_batch(&mut self, _settings: &AdmittedGpuSettings) {}
+
+        fn use_no_gpu_from_the_next_batch(&mut self) {
+            self.script
+                .lock()
+                .expect("用例没在别处 panic 过")
+                .times_told_no_gpu_takes_part += 1;
+        }
+
+        fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
+            &mut self,
+        ) -> LatestGpuSettingsHandedIn {
+            let mut script = self.script.lock().expect("用例没在别处 panic 过");
+            script.times_asked_between_two_batches += 1;
+            script.answer.clone()
+        }
+    }
+
+    fn script_answering(
+        answer: LatestGpuSettingsHandedIn,
+    ) -> Arc<Mutex<ScriptOfTheImplementation>> {
+        Arc::new(Mutex::new(ScriptOfTheImplementation {
+            answer,
+            times_told_no_gpu_takes_part: 0,
+            times_asked_between_two_batches: 0,
+        }))
+    }
+
+    fn answer_from_now_on(
+        script: &Arc<Mutex<ScriptOfTheImplementation>>,
+        answer: LatestGpuSettingsHandedIn,
+    ) {
+        script.lock().expect("用例没在别处 panic 过").answer = answer;
+    }
+
+    fn put_in_effect_and_usable() -> LatestGpuSettingsHandedIn {
+        LatestGpuSettingsHandedIn::PutInEffect {
+            failure_under_the_settings: None,
+        }
+    }
+
+    /// 选 `gpu`、点「0000:01:00.0」的一次挂载，替身照 `script` 答。
+    fn mount_on_the_card_at_one_answering_as_the_script_says(
+        script: &Arc<Mutex<ScriptOfTheImplementation>>,
+    ) -> BackgroundComputeOfThisMount {
+        let handle =
+            BackgroundGpuComputeHandle::of_an_implementation(GpuComputeAnsweringAsTheScriptSays {
+                script: Arc::clone(script),
+            });
+        BackgroundComputeOfThisMount::of_a_mount(
+            admit_background_compute_settings_with_the_gpu_compute(
+                gpu_settings_naming_the_card(b"0000:01:00.0", None),
+                Some(handle),
+            )
+            .expect("卡清单列着「0000:01:00.0」"),
+        )
+    }
+
+    fn not_opened_on_the_card_at_forty_one() -> BackgroundGpuComputeFailed {
+        BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings {
+            device: card_for_the_switch_tests(b"0000:41:00.0"),
+            failures_of_each_opening: Vec::new(),
+        }
+    }
+
+    /// 池子最满的那一块盘 95%，过了默认 85% 的补跑阈值：停着就在 CPU 上补跑。
+    fn fullest_device_above_the_default_threshold() -> AllocatedBytesAndCapacityOfADevice {
+        AllocatedBytesAndCapacityOfADevice {
+            device: DeviceIdentity(0),
+            allocated_bytes: 95,
+            capacity_bytes: 100,
+        }
+    }
+
+    fn route_of_the_next_batch(
+        background_compute: &mut BackgroundComputeOfThisMount,
+        fullest_device: AllocatedBytesAndCapacityOfADevice,
+    ) -> Option<BackgroundComputeRoute> {
+        route_of_the_batch(
+            &background_compute.begin_a_batch(BackgroundTask::CompactionRelocation, fullest_device),
+        )
+    }
+
+    /// 一段历史在开不成之后的读数：开不成那一批走哪条路、开不成之后生效的那一组点哪张卡、再请求同一组之后那一批走哪条路。
+    struct ReadingsOfTheRetryAfterNotOpened {
+        times_told_no_gpu_takes_part: u32,
+        route_once_not_opened: Option<BackgroundComputeRoute>,
+        devices_in_effect_once_not_opened: Option<Vec<GpuDeviceIdentifier>>,
+        route_at_the_retry: Option<BackgroundComputeRoute>,
+        handed_in_at_the_retry: bool,
+    }
+
+    /// 挂载点「0000:01:00.0」；`switch_to_the_cpu_first` 时先切到 `cpu`；请求切到没开的「0000:41:00.0」，实现答还在开卡、之后答开不成；
+    /// 再请求同一组（停着时的切换：当场解除），实现答还在开卡。
+    fn readings_of_the_retry_after_not_opened(
+        switch_to_the_cpu_first: bool,
+    ) -> ReadingsOfTheRetryAfterNotOpened {
+        let script = script_answering(put_in_effect_and_usable());
+        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
+        if switch_to_the_cpu_first {
+            background_compute
+                .request_switch_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
+                .expect("选 cpu 照收");
+            let _route_on_the_cpu = route_of_the_next_batch(
+                &mut background_compute,
+                fullest_device_far_below_the_threshold(),
+            );
+        }
+        background_compute
+            .request_switch_from_the_next_batch(
+                gpu_settings_naming_the_card(b"0000:41:00.0", None),
+                None,
+            )
+            .expect("卡清单列着「0000:41:00.0」");
+        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
+        let _route_while_opening = route_of_the_next_batch(
+            &mut background_compute,
+            fullest_device_far_below_the_threshold(),
+        );
+        answer_from_now_on(
+            &script,
+            LatestGpuSettingsHandedIn::NotOpened {
+                failure: not_opened_on_the_card_at_forty_one(),
+            },
+        );
+        let route_once_not_opened = route_of_the_next_batch(
+            &mut background_compute,
+            fullest_device_far_below_the_threshold(),
+        );
+        let devices_in_effect_once_not_opened = background_compute
+            .in_effect()
+            .settings()
+            .background_gpu_devices
+            .clone();
+        background_compute
+            .request_switch_from_the_next_batch(
+                gpu_settings_naming_the_card(b"0000:41:00.0", None),
+                None,
+            )
+            .expect("卡清单列着「0000:41:00.0」");
+        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
+        let route_at_the_retry = route_of_the_next_batch(
+            &mut background_compute,
+            fullest_device_far_below_the_threshold(),
+        );
+        let times_told_no_gpu_takes_part = script
+            .lock()
+            .expect("用例没在别处 panic 过")
+            .times_told_no_gpu_takes_part;
+        ReadingsOfTheRetryAfterNotOpened {
+            times_told_no_gpu_takes_part,
+            route_once_not_opened,
+            devices_in_effect_once_not_opened,
+            route_at_the_retry,
+            handed_in_at_the_retry: background_compute
+                .handed_in_while_the_implementation_opens_its_cards()
+                .is_some(),
+        }
+    }
+
+    /// 实现答开不成时核心层不把那一组记成生效：先切到 `cpu`、切回的卡开不成、再请求同一组——开不成那一批停着、生效的仍是 `cpu`，
+    /// 再请求之后实现还在开卡，那一批在 CPU 上算（不交给一张卡都没开的实现）。
+    #[test]
+    fn a_switch_not_opened_after_the_cpu_leaves_the_cpu_in_effect_and_the_retry_runs_on_the_cpu() {
+        let readings = readings_of_the_retry_after_not_opened(true);
+        assert_eq!(readings.times_told_no_gpu_takes_part, 1);
+        assert_eq!(readings.route_once_not_opened, None, "开不成：停着、不开批");
+        assert_eq!(
+            readings.devices_in_effect_once_not_opened, None,
+            "开不成的那一组不生效：生效的仍是 cpu"
+        );
+        assert_eq!(
+            readings.route_at_the_retry,
+            Some(BackgroundComputeRoute::Cpu(
+                ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings
+            )),
+            "再请求之后实现还在开卡：照生效的 cpu 算"
+        );
+        assert!(readings.handed_in_at_the_retry);
+    }
+
+    /// 同一段历史不先切到 `cpu`：开不成之后核心层生效的仍是切换之前那张「0000:01:00.0」——与实现照旧参与的是同一组；再请求之后那一批照它在 GPU 上算。
+    #[test]
+    fn a_switch_not_opened_on_the_gpu_leaves_the_card_before_the_switch_in_effect() {
+        let readings = readings_of_the_retry_after_not_opened(false);
+        assert_eq!(readings.times_told_no_gpu_takes_part, 0);
+        assert_eq!(readings.route_once_not_opened, None, "开不成：停着、不开批");
+        assert_eq!(
+            readings.devices_in_effect_once_not_opened,
+            Some(vec![card_for_the_switch_tests(b"0000:01:00.0")]),
+            "开不成的那一组不生效：生效的仍是切换之前那一组"
+        );
+        assert_eq!(
+            readings.route_at_the_retry,
+            Some(BackgroundComputeRoute::Gpu)
+        );
+        assert!(readings.handed_in_at_the_retry);
+    }
+
+    /// 实现答开不成之后，补跑那一判照那一次切换请求给的阈值：挂载取默认 85%，请求切到「0000:41:00.0」给 50%，开不成——
+    /// 池子最满的那一块盘 60%，下一批在 CPU 上补跑。
+    #[test]
+    fn the_gc_cpu_compensation_threshold_of_a_switch_not_opened_is_the_one_the_request_gave() {
+        let script = script_answering(put_in_effect_and_usable());
+        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
+        background_compute
+            .request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
+                gpu_settings_naming_the_card(b"0000:41:00.0", None),
+                None,
+                BackgroundGcCpuCompensationThreshold {
+                    allocated_percent_of_the_capacity: 50,
+                },
+            )
+            .expect("卡清单列着「0000:41:00.0」、阈值不超过 100");
+        answer_from_now_on(
+            &script,
+            LatestGpuSettingsHandedIn::NotOpened {
+                failure: not_opened_on_the_card_at_forty_one(),
+            },
+        );
+        let fullest_device_at_sixty_percent = AllocatedBytesAndCapacityOfADevice {
+            device: DeviceIdentity(0),
+            allocated_bytes: 60,
+            capacity_bytes: 100,
+        };
+        let route =
+            route_of_the_next_batch(&mut background_compute, fullest_device_at_sixty_percent);
+        assert!(
+            matches!(
+                route,
+                Some(BackgroundComputeRoute::Cpu(
+                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
+                ))
+            ),
+            "照请求给的 50% 判：60% 超过，补跑：{route:?}"
+        );
+    }
+
+    /// 交给了实现、实现还在开卡的期间每一批都在切换之前那张卡上出错，池子最满的那一块盘过了补跑阈值：头一批出错停下，之后每次叫起报警、
+    /// 每一批在 CPU 上补跑；实现答换上了的那一刻解除，那一批在 GPU 上算。
+    #[test]
+    fn batches_failing_while_the_switched_card_opens_stop_alarm_and_compensate_until_it_is_put_in_effect(
+    ) {
+        let script = script_answering(put_in_effect_and_usable());
+        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
+        background_compute
+            .request_switch_from_the_next_batch(
+                gpu_settings_naming_the_card(b"0000:41:00.0", None),
+                None,
+            )
+            .expect("卡清单列着「0000:41:00.0」");
+        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
+        background_compute.begin_a_wake();
+        let route_of_the_failing_batch = route_of_the_next_batch(
+            &mut background_compute,
+            fullest_device_above_the_default_threshold(),
+        );
+        let after_the_error = background_compute.stop_the_task_after_a_compute_error(
+            BackgroundTask::CompactionRelocation,
+            failure_on_the_card_at_one(),
+        );
+        let _alarm_of_the_failing_wake =
+            background_compute.alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation);
+        background_compute.begin_a_wake();
+        let route_while_stopped = route_of_the_next_batch(
+            &mut background_compute,
+            fullest_device_above_the_default_threshold(),
+        );
+        let alarm_while_stopped =
+            background_compute.alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation);
+        assert_eq!(
+            route_of_the_failing_batch,
+            Some(BackgroundComputeRoute::Gpu)
+        );
+        assert_eq!(
+            after_the_error,
+            BackgroundTaskAfterTheComputeError::LeftStoppedWhileTheSwitchOpensItsCards
+        );
+        assert!(
+            matches!(
+                route_while_stopped,
+                Some(BackgroundComputeRoute::Cpu(
+                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
+                ))
+            ),
+            "停着、过了阈值：在 CPU 上补跑：{route_while_stopped:?}"
+        );
+        assert_eq!(
+            alarm_while_stopped
+                .map(|alarm| (alarm.wakes_while_stopped, alarm.error_that_stopped_the_task)),
+            Some((1, failure_on_the_card_at_one())),
+            "停下之后的下一次叫起报警"
+        );
+        answer_from_now_on(&script, put_in_effect_and_usable());
+        let route_once_put_in_effect = route_of_the_next_batch(
+            &mut background_compute,
+            fullest_device_above_the_default_threshold(),
+        );
+        assert_eq!(
+            route_once_put_in_effect,
+            Some(BackgroundComputeRoute::Gpu),
+            "实现答换上了：解除，照新的那一组在 GPU 上算"
+        );
+        assert!(!background_compute
+            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
+    }
+
+    /// 开卡期间出错停着、之后实现答开不成：照开不成停着，报的是开不成那一个错。
+    #[test]
+    fn a_batch_failing_while_the_switched_card_opens_stays_stopped_by_the_not_opened_failure_once_not_opened(
+    ) {
+        let script = script_answering(put_in_effect_and_usable());
+        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
+        background_compute
+            .request_switch_from_the_next_batch(
+                gpu_settings_naming_the_card(b"0000:41:00.0", None),
+                None,
+            )
+            .expect("卡清单列着「0000:41:00.0」");
+        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
+        let _route_of_the_failing_batch = route_of_the_next_batch(
+            &mut background_compute,
+            fullest_device_far_below_the_threshold(),
+        );
+        let _after_the_error = background_compute.stop_the_task_after_a_compute_error(
+            BackgroundTask::CompactionRelocation,
+            failure_on_the_card_at_one(),
+        );
+        answer_from_now_on(
+            &script,
+            LatestGpuSettingsHandedIn::NotOpened {
+                failure: not_opened_on_the_card_at_forty_one(),
+            },
+        );
+        let route_once_not_opened = route_of_the_next_batch(
+            &mut background_compute,
+            fullest_device_far_below_the_threshold(),
+        );
+        assert_eq!(route_once_not_opened, None, "照开不成停着");
+        assert_eq!(
+            background_compute
+                .stopped_after_a_compute_error()
+                .iter()
+                .map(|stopped| stopped.error.clone())
+                .collect::<Vec<_>>(),
+            vec![
+                BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
+                    failure: not_opened_on_the_card_at_forty_one(),
+                }
+            ]
+        );
+    }
+
+    /// 对照：同一个错、同样过了补跑阈值，没有切换——停下（`LeftStopped`），下一批在 CPU 上补跑。
+    #[test]
+    fn the_same_error_without_a_switch_is_left_stopped_and_compensates_on_the_cpu() {
+        let script = script_answering(put_in_effect_and_usable());
+        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
+        let _route_of_the_failing_batch = route_of_the_next_batch(
+            &mut background_compute,
+            fullest_device_above_the_default_threshold(),
+        );
+        let after_the_error = background_compute.stop_the_task_after_a_compute_error(
+            BackgroundTask::CompactionRelocation,
+            failure_on_the_card_at_one(),
+        );
+        let route_of_the_next_batch_after_the_error = route_of_the_next_batch(
+            &mut background_compute,
+            fullest_device_above_the_default_threshold(),
+        );
+        assert_eq!(
+            after_the_error,
+            BackgroundTaskAfterTheComputeError::LeftStopped
+        );
+        assert!(matches!(
+            route_of_the_next_batch_after_the_error,
+            Some(BackgroundComputeRoute::Cpu(
+                ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
+            ))
+        ));
+    }
+
+    /// 一批里同一个条目键出现两次只放弃这一批：那个后台任务不停、叫起不报警，下一批照常在 GPU 上算。
+    #[test]
+    fn the_same_entry_key_twice_in_one_batch_abandons_only_the_batch_without_stopping_or_alarming()
+    {
+        let script = script_answering(put_in_effect_and_usable());
+        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
+        background_compute.begin_a_wake();
+        let _route_of_the_abandoned_batch = route_of_the_next_batch(
+            &mut background_compute,
+            fullest_device_far_below_the_threshold(),
+        );
+        let after_the_error = background_compute.stop_the_task_after_a_compute_error(
+            BackgroundTask::CompactionRelocation,
+            BackgroundComputeError::GpuSegmentFailed {
+                segment: GpuComputedSegment::LiveSlotsPerClusterSegment,
+                failure: BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch {
+                    position_of_the_first_occurrence: 1,
+                    position_of_the_second_occurrence: 3,
+                },
+            },
+        );
+        let alarm =
+            background_compute.alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation);
+        let route_of_the_next_batch_after_the_error = route_of_the_next_batch(
+            &mut background_compute,
+            fullest_device_far_below_the_threshold(),
+        );
+        assert_eq!(
+            after_the_error,
+            BackgroundTaskAfterTheComputeError::NotStoppedForOnlyTheBatchIsAbandoned
+        );
+        assert!(!background_compute
+            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
+        assert_eq!(alarm, None, "不报警");
+        assert_eq!(
+            route_of_the_next_batch_after_the_error,
+            Some(BackgroundComputeRoute::Gpu),
+            "下一批照常算"
+        );
+    }
+
+    /// 交给了实现、还在开卡的那一组被一次 `cpu` 的切换盖掉：交 `cpu` 那一刻作废还在开卡的那一格，之后不再为它问实现。
+    #[test]
+    fn a_switch_to_the_cpu_drops_the_settings_whose_cards_are_still_opening() {
+        let script = script_answering(put_in_effect_and_usable());
+        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
+        background_compute
+            .request_switch_from_the_next_batch(
+                gpu_settings_naming_the_card(b"0000:41:00.0", None),
+                None,
+            )
+            .expect("卡清单列着「0000:41:00.0」");
+        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
+        let _route_while_opening = route_of_the_next_batch(
+            &mut background_compute,
+            fullest_device_far_below_the_threshold(),
+        );
+        background_compute
+            .request_switch_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
+            .expect("选 cpu 照收");
+        let route_on_the_cpu = route_of_the_next_batch(
+            &mut background_compute,
+            fullest_device_far_below_the_threshold(),
+        );
+        assert_eq!(
+            route_on_the_cpu,
+            Some(BackgroundComputeRoute::Cpu(
+                ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings
+            ))
+        );
+        assert!(
+            background_compute
+                .handed_in_while_the_implementation_opens_its_cards()
+                .is_none(),
+            "交 cpu 那一刻作废还在开卡的那一组"
+        );
+    }
+
+    /// 开卡期间出错停着时，补跑那一判照交给了实现、还在开卡的那一组请求给的阈值：挂载取默认 85%，请求切到「0000:41:00.0」给 50%，开卡期间出错——
+    /// 池子最满的那一块盘 60%，下一批在 CPU 上补跑。
+    #[test]
+    fn the_gc_cpu_compensation_threshold_while_the_switch_opens_its_cards_is_the_one_the_request_gave(
+    ) {
+        let script = script_answering(put_in_effect_and_usable());
+        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
+        background_compute
+            .request_switch_with_the_gc_cpu_compensation_threshold_from_the_next_batch(
+                gpu_settings_naming_the_card(b"0000:41:00.0", None),
+                None,
+                BackgroundGcCpuCompensationThreshold {
+                    allocated_percent_of_the_capacity: 50,
+                },
+            )
+            .expect("卡清单列着「0000:41:00.0」、阈值不超过 100");
+        answer_from_now_on(&script, LatestGpuSettingsHandedIn::StillOpeningTheirCards);
+        let fullest_device_at_sixty_percent = AllocatedBytesAndCapacityOfADevice {
+            device: DeviceIdentity(0),
+            allocated_bytes: 60,
+            capacity_bytes: 100,
+        };
+        let _route_of_the_failing_batch =
+            route_of_the_next_batch(&mut background_compute, fullest_device_at_sixty_percent);
+        let _after_the_error = background_compute.stop_the_task_after_a_compute_error(
+            BackgroundTask::CompactionRelocation,
+            failure_on_the_card_at_one(),
+        );
+        let route =
+            route_of_the_next_batch(&mut background_compute, fullest_device_at_sixty_percent);
+        assert!(
+            matches!(
+                route,
+                Some(BackgroundComputeRoute::Cpu(
+                    ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(_)
+                ))
+            ),
+            "照还在开卡的那一组给的 50% 判：60% 超过，补跑：{route:?}"
+        );
+    }
+
+    /// 没有交给了实现、还没生效的那一组时，一批开头照样问一次生效那一组带着的实现（让它收下为更早几次交设置开完的卡），答案不用：
+    /// 挂载那一刻问 1 次，下一批开头再问 1 次。
+    #[test]
+    fn a_batch_begun_with_no_settings_waiting_still_asks_the_implementation_once() {
+        let script = script_answering(put_in_effect_and_usable());
+        let mut background_compute = mount_on_the_card_at_one_answering_as_the_script_says(&script);
+        let asked_at_the_mount = script
+            .lock()
+            .expect("用例没在别处 panic 过")
+            .times_asked_between_two_batches;
+        let _route = route_of_the_next_batch(
+            &mut background_compute,
+            fullest_device_far_below_the_threshold(),
+        );
+        assert_eq!(
+            (
+                asked_at_the_mount,
+                script
+                    .lock()
+                    .expect("用例没在别处 panic 过")
+                    .times_asked_between_two_batches
+            ),
+            (1, 2)
+        );
+    }
 }
````

## crates/singlefs-core/src/compaction_plan.rs

````diff
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -629,18 +629,19 @@
         limit_in_bytes_per_second: u64,
     },
     /// 这一批算着出了错（D26 已定项 13 第 4 条）：这一批放弃、一个单元都没搬；现行那一版里有活意图就单发一次只删意图的发布丢掉它
-    /// （R 排他跟着撤，同 [`Self::PausedShortOfSpace`]），没有就一个写都没发。整理这个后台任务停下或当场解除停下见 `task_after_the_error`；
-    /// 下一次整理从头签发。
+    /// （R 排他跟着撤，同 [`Self::PausedShortOfSpace`]），没有就一个写都没发。整理这个后台任务怎样了见 `task_after_the_error`：
+    /// 一批里同一个条目键出现两次（`background_compute::BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch`）只作废这一批、任务不停
+    /// （[`BackgroundTaskAfterTheComputeError::NotStoppedForOnlyTheBatchIsAbandoned`]）；别的错停下，或当场解除停下。下一次整理从头签发。
     BatchAbandonedAfterABackgroundComputeError {
         task_after_the_error: BackgroundTaskAfterTheComputeError,
     },
-    /// 整理这个后台任务先前因一批出错停着（D26 已定项 13 第 4 条；挂载那一刻或切换生效的那一批开头，实现照交给它的那一组一张卡都用不上而当场停着的同算），
+    /// 整理这个后台任务先前因一批出错停着（D26 已定项 13 第 4 条；挂载那一刻或两批之间实现答交给它的那一组换上了而一张卡都用不上、答开不成而当场停着的同算），
     /// 而这一刻池子每块盘的已分配占它的容量都不超过补跑阈值：
     /// 停在这一刻：现行那一版里有活意图就单发一次只删意图的发布丢掉它（R 排他跟着撤，同 [`Self::PausedShortOfSpace`]），之后一个写都不发；
     /// 没有活意图就从这一刻起一个写都不发（删除续做也不走）。显式入口每一趟开头判这一条；后台入口判出停着而有活意图时起跑一趟只为删它
-    /// （`mounted_session::BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`），没有活意图就不起跑。挂着切到一张卡都用不上的那一组、
-    /// 手里没有活意图时，两个入口都在任何写之前停在这一条。一趟之中在 CPU 上补跑过几批之后、或一趟之中切换生效的那一批开头才判出的，
-    /// 前面那几批照常发过。等占用超过阈值（下一趟照常补跑、从头签发），或一次通过校验的切换请求解除停下。
+    /// （`mounted_session::BackgroundCompactionTrigger::LiveIntentWhileStoppedAfterAComputeError`），没有活意图就不起跑。挂着切换之后实现答那一组一张卡都用不上或开不成、
+    /// 手里没有活意图时，两个入口都在任何写之前停在这一条。一趟之中在 CPU 上补跑过几批之后、或一趟之中两批之间实现答出那一组的结局才判出的，
+    /// 前面那几批照常发过。等占用超过阈值（下一趟照常补跑、从头签发），或一次通过校验的切换请求解除停下；开卡期间出错停着的，实现答那一组换上了也解除。
     BackgroundTaskStoppedAfterAnEarlierComputeError,
     /// GPU 路径上 R 里除了这一轮两份核判得不一样的记录，再没有判定了可搬的（D26 已定项 13 第 8 条「这一批不回收、不搬，留到下一轮重算」）：
     /// 这一批不发，意图照留（不删也不丢），下一批重扫照判。
````

## crates/singlefs-mount-runtime/src/lib.rs

````diff
--- a/crates/singlefs-mount-runtime/src/lib.rs
+++ b/crates/singlefs-mount-runtime/src/lib.rs
@@ -28,8 +28,8 @@
 use singlefs_core::background_compute::{
     AlarmOfABackgroundTaskStoppedAfterAComputeError, BackgroundComputeBackend,
     BackgroundComputeSettings, BackgroundComputeSettingsRefused,
-    BackgroundGcCpuCompensationThreshold, BackgroundGpuComputeHandle,
-    BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
+    BackgroundGcCpuCompensationThreshold, BackgroundGpuComputeHandle, GpuDeviceIdentifier,
+    GpuMemoryInBytes, BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
 };
 use singlefs_core::block_device::BlockDevice;
 use singlefs_core::compaction_plan::{
@@ -143,11 +143,31 @@
     RefusedByTheOpening { description: String },
 }
 
-/// 开成的后台 GPU：交给核心层挂载的把手，与开卡时跳过了哪几张卡的报告（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
+/// 开成的后台 GPU：交给核心层挂载的把手，开卡时跳过了哪几张卡的报告，与读它这一次挂载泄漏了哪几张卡的那一份
+/// （`.claude/rules/fs-design.md` 五条硬要求第 4 条；开卡器给不出这一份的是 `None`，产品开卡器恒给）。运行库留着后两样：
+/// [`MountedRuntime::report_of_the_opening`]、[`MountedRuntime::gpus_leaked_by_the_background_gpu`]，卸载之后在 [`UnmountedPool`] 里。
 #[derive(Debug)]
 pub struct OpenedBackgroundGpu {
     pub gpu_compute: BackgroundGpuComputeHandle,
     pub report_of_the_opening: String,
+    pub gpus_leaked: Option<Arc<dyn ReadsTheGpusLeakedByTheBackgroundGpu>>,
+}
+
+/// 后台 GPU 在这一次挂载里泄漏的一张卡（D26（后台整理与放置回收） 已定项 13 第 4 条：切换关卡、卸载时有提交还在跑的那张整张泄漏、不析构，并报出来）：
+/// 哪张卡、泄漏那一刻本包在它上面占着的显存。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct GpuLeakedByTheBackgroundGpu {
+    pub device: GpuDeviceIdentifier,
+    pub memory_held_on_the_device: GpuMemoryInBytes,
+}
+
+/// 读开成的后台 GPU 在这一次挂载里泄漏了哪几张卡（实现在产品 GPU 包，开卡器开成时交来）：随时读，后台 GPU 丢掉之后照样读得到。
+pub trait ReadsTheGpusLeakedByTheBackgroundGpu: fmt::Debug + Send + Sync {
+    /// 挂着切换关卡时泄漏的卡，按关的先后累计；没有是空。
+    fn gpus_leaked_when_closed_at_a_switch(&self) -> Vec<GpuLeakedByTheBackgroundGpu>;
+
+    /// 后台 GPU 丢掉（卸载、丢掉核心层收的把手）那一刻泄漏的卡；还没丢是 `None`，丢了而一张都没泄漏是空。
+    fn gpus_leaked_when_dropped(&self) -> Option<Vec<GpuLeakedByTheBackgroundGpu>>;
 }
 
 /// 开卡器（D26（后台整理与放置回收） 已定项 13 第 4 条「挂载时由调用方把它的实现递进去」）：运行库不依赖产品 GPU 包，实现在那边
@@ -298,6 +318,9 @@
     /// 卸载之前排空待发那一次（待发空时 `None`）。
     pub pending_changes_published: Option<PublishReportedByTheSession>,
     pub unmounted: Unmounted,
+    /// 挂载那一刻开卡器开成的后台 GPU 泄漏了哪几张卡（[`OpenedBackgroundGpu::gpus_leaked`]；卸载时丢掉的那一份已记下）。
+    /// 没开卡、或开卡器给不出这一份的是 `None`。
+    pub gpus_leaked_by_the_background_gpu: Option<Arc<dyn ReadsTheGpusLeakedByTheBackgroundGpu>>,
 }
 
 impl<Device> fmt::Debug for UnmountedPool<Device> {
@@ -305,6 +328,10 @@
         formatter
             .debug_struct("UnmountedPool")
             .field("pending_changes_published", &self.pending_changes_published)
+            .field(
+                "gpus_leaked_by_the_background_gpu",
+                &self.gpus_leaked_by_the_background_gpu,
+            )
             .finish_non_exhaustive()
     }
 }
@@ -375,6 +402,15 @@
     observations: Mutex<RuntimeObservations>,
     clock: Arc<dyn ClockOfTheRuntime>,
     receiver: Arc<dyn ReceiverOfTheMaintenanceEvents>,
+    /// 挂载那一刻开卡器开成的后台 GPU：开卡的报告与读它泄漏了哪几张卡的那一份。没开卡是 `None`。
+    background_gpu_opened_at_the_mount: Option<BackgroundGpuOpenedAtTheMount>,
+}
+
+/// 运行库留着的、挂载那一刻开卡器开成的后台 GPU 的两样（[`OpenedBackgroundGpu`] 里除把手之外的）。
+#[derive(Clone, Debug)]
+struct BackgroundGpuOpenedAtTheMount {
+    report_of_the_opening: String,
+    gpus_leaked: Option<Arc<dyn ReadsTheGpusLeakedByTheBackgroundGpu>>,
 }
 
 /// 后台入口每一批之前问的两样（核心层不起线程，钟与前台 fsync 都在这一层）。停线程时答「在停线程」：整理在下一批之前就返回，线程很快 join 得上。
@@ -806,13 +842,19 @@
                 reason: RuntimeMountRefusal::PeriodOfZero,
             });
         }
-        let gpu_compute = match (
+        let (gpu_compute, background_gpu_opened_at_the_mount) = match (
             &runtime_parameters.background_compute.background_compute,
             gpu_opener,
         ) {
             (BackgroundComputeBackend::Gpu, Some(opener)) => {
                 match opener.open_the_background_gpu(&runtime_parameters.background_compute) {
-                    Ok(opened) => Some(opened.gpu_compute),
+                    Ok(opened) => (
+                        Some(opened.gpu_compute),
+                        Some(BackgroundGpuOpenedAtTheMount {
+                            report_of_the_opening: opened.report_of_the_opening,
+                            gpus_leaked: opened.gpus_leaked,
+                        }),
+                    ),
                     Err(not_opened) => {
                         return Err(RuntimeMountRefused {
                             devices,
@@ -821,7 +863,9 @@
                     }
                 }
             }
-            (BackgroundComputeBackend::Gpu, None) | (BackgroundComputeBackend::Cpu, _) => None,
+            (BackgroundComputeBackend::Gpu, None) | (BackgroundComputeBackend::Cpu, _) => {
+                (None, None)
+            }
         };
         let mounted = match mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_parameters(
             parameters,
@@ -849,6 +893,7 @@
             },
             clock,
             receiver,
+            background_gpu_opened_at_the_mount,
         );
         Ok(RuntimeMounted { output, runtime })
     }
@@ -859,6 +904,7 @@
         locked: SessionAndDevices<Device>,
         clock: Arc<dyn ClockOfTheRuntime>,
         receiver: Arc<dyn ReceiverOfTheMaintenanceEvents>,
+        background_gpu_opened_at_the_mount: Option<BackgroundGpuOpenedAtTheMount>,
     ) -> Self {
         MountedRuntime {
             shared: Arc::new(Shared {
@@ -873,11 +919,33 @@
                 observations: Mutex::new(RuntimeObservations::default()),
                 clock,
                 receiver,
+                background_gpu_opened_at_the_mount,
             }),
             maintenance_thread: None,
         }
     }
 
+    /// 挂载那一刻开卡器开成的后台 GPU 开卡时的报告（跳过了哪几张卡）；没开卡是 `None`。
+    #[must_use]
+    pub fn report_of_the_opening(&self) -> Option<String> {
+        self.shared
+            .background_gpu_opened_at_the_mount
+            .as_ref()
+            .map(|opened| opened.report_of_the_opening.clone())
+    }
+
+    /// 读挂载那一刻开卡器开成的后台 GPU 泄漏了哪几张卡的那一份（[`ReadsTheGpusLeakedByTheBackgroundGpu`]：切换关卡时泄漏的随时读得到，
+    /// 卸载时丢掉的那一份卸载之后读得到，[`UnmountedPool::gpus_leaked_by_the_background_gpu`] 是同一份）。没开卡、或开卡器给不出这一份的是 `None`。
+    #[must_use]
+    pub fn gpus_leaked_by_the_background_gpu(
+        &self,
+    ) -> Option<Arc<dyn ReadsTheGpusLeakedByTheBackgroundGpu>> {
+        self.shared
+            .background_gpu_opened_at_the_mount
+            .as_ref()
+            .and_then(|opened| opened.gpus_leaked.clone())
+    }
+
     /// 前台收下一次用户改动（`MountedSessionWithPendingChanges::accept`）。被空间拒时这一次调用当场推抬 F 再判（D16（发布语义） 已定项 1「准入」那一行）：
     /// 收下之前可能发抬 F 的空发布，一次准入至多 B − 1 次，全程持着会话那把锁（后台批与别的前台入口等到它返回；发布之间不放锁，推的那一串与再判之间不插别的发布），
     /// 写盘的时长算进这一次调用。推成的报在 [`ChangeAccepted`] 的 `floor_raises_pushed_for_space`（不进 `publishes`：空发布不改用户可见状态），
@@ -1062,6 +1130,7 @@
             locked,
             clock,
             receiver,
+            background_gpu_opened_at_the_mount,
             ..
         } = Arc::try_unwrap(shared).unwrap_or_else(|_still_shared| {
             panic!("线程 join 过了、前台入口都借着 self：运行库是这一份的唯一持有者")
@@ -1102,6 +1171,7 @@
                             },
                             clock,
                             receiver,
+                            background_gpu_opened_at_the_mount,
                         ),
                         refusal: refused.refusal,
                     }));
@@ -1119,6 +1189,8 @@
                 devices,
                 pending_changes_published,
                 unmounted,
+                gpus_leaked_by_the_background_gpu: background_gpu_opened_at_the_mount
+                    .and_then(|opened| opened.gpus_leaked),
             }),
             Err(refusal) => Err(Box::new(UnmountRefused::UnmountFailed { devices, refusal })),
         }
````
