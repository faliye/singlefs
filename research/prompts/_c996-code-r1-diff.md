# c996-code-r1 被判的产品代码改动（2026-11-12）

两份实现员补丁里落在 `crates/*/src/` 下的块，原样。测试与变异表的改动不在这里，在补丁目录里。

## 补丁 `/home/fy5090/.local/share/singlefs/work/c996-open-only-participating/patch/crates.patch`

### crates/singlefs-background-gpu/src/background_gpu.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/background_gpu.rs b/crates/singlefs-background-gpu/src/background_gpu.rs
index 76f431d..8025732 100644
--- a/crates/singlefs-background-gpu/src/background_gpu.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu.rs
@@ -1,16 +1,16 @@
-//! 起 GPU 的外壳（D26（后台整理与放置回收） 已定项 13 第 1–4、8 条）：挂载那一刻枚举 wgpu 适配器、判过校验、在本机每一张独立显卡上起设备与各段两份核的管线、
-//! 量一次提交的主机 CPU 时间；一张卡没开成时，它是参与的卡就拒绝挂载，给了 `background_gpu_devices` 而没点它就跳过、报出来
-//! （[`BackgroundGpuOverTheCards::gpus_not_opened_at_the_mount`]；判法在 `crate::gpus_of_this_host::gpus_opened_at_the_mount`）。给了 `background_gpu_devices` 时只判点名的卡：没点名的独显缺总线地址、缺设备本地堆、与别的独显报了同一个地址都不拒，
+//! 起 GPU 的外壳（D26（后台整理与放置回收） 已定项 13 第 1–4、8 条）：挂载那一刻枚举 wgpu 适配器、判过校验、只在参与的卡上起设备与各段两份核的管线、
+//! 量一次提交的主机 CPU 时间（欠账 C996 的定案「只开参与的卡」：给了 `background_gpu_devices` 是点名的卡，没给是本机全部独立显卡）；
+//! 参与的卡没开成就拒绝挂载（判法在 `crate::gpus_of_this_host::gpus_opened_at_the_mount`）。给了 `background_gpu_devices` 时只判点名的卡：没点名的独显缺总线地址、缺设备本地堆、与别的独显报了同一个地址都不拒，
 //! 不收进卡清单、报出来（[`BackgroundGpuOverTheCards::unnamed_discrete_gpus_left_off_the_list`]，判法 `crate::adapter_inventory::discrete_gpus_of_the_enumeration_under_the_named`）。
 //! 参与的卡由设置定（[`BackgroundGpuOverTheCards::participating_gpus`] 报出来），挂着切换时核心层交来新的一组
-//! （[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]，两批之间），从下一批起只在它点的卡上、照它给的显存上限与等的上限跑，不重开设备
-//! （分活、切段、等的纯逻辑在 `crate::gpus_of_this_host`）。
+//! （[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]，两批之间），从下一批起只在它点的卡上、照它给的显存上限与等的上限跑：
+//! 开着的卡不重开；它点到挂载时没开的卡，在这一刻开那一张（开卡同挂载那一刻，[`OpeningOfAGpuAtASwitch`]），开不成的这张卡不参与，
+//! 之后每一批报成它的 GPU 出错，直到下一次换设置。切换之后不再参与的卡照开着，不关（分活、切段、等的纯逻辑在 `crate::gpus_of_this_host`）。
 //! 挂着的时候每一批按卡的顺序填满参与的卡（前面的卡填到它照显存上限一次放得下的量，装不下的才给后面的卡；挂载时量的代价不用来分活）、按显存上限切段、两份核各算一次、逐条比对。三段都接上了：扫描里的校验和核对（[`BackgroundGpuOverTheCards::verify_unit_checksums`]）、
 //! 区域扫描里的单元头解析与映射反查（[`BackgroundGpuOverTheCards::classify_region_records`]）与选段打分的每段活槽数（[`BackgroundGpuOverTheCards::count_live_slots_per_cluster_segment`]）。
-//! 本机的卡清单（核心层 `GpuInventory`）答的是此刻本机在的独显与各自的显存总量（按总线地址排，挂载那一刻没开成的照列；每次答之前现枚举一次，
-//! 挂载之后掉了的卡不列，切换点它照第 4 条「GPU 不在」拒），连同挂载那一刻没开成的那几张与原因（挂载那一刻定下，答它不现枚举：一次切换只现枚举一次）：
-//! 挂载与切换照它校验设置，切换可以点开包时没参与而开成了的卡；
-//! 点名不在本机的卡就是点名的卡不存在，参与的卡里有没开成的照第 4 条拒。
+//! 本机的卡清单（核心层 `GpuInventory`）答的是此刻本机在的独显与各自的显存总量（按总线地址排，开着的与没参与而没开的都列；每次答之前现枚举一次，
+//! 挂载之后掉了的卡不列，切换点它照第 4 条「GPU 不在」拒）：挂载与切换照它校验设置，切换可以点挂载时没开的卡；
+//! 点名不在本机的卡就是点名的卡不存在。
 //!
 //! 外壳分两层：一张开着的卡要做的几样（[`CardOfTheBackgroundGpu`]：交活之前先等前一次超时的那次提交、重交之前等、交一段两份核）由 [`GpuInUse`]
 //! （wgpu 设备与六条管线）实现；几张卡上怎么分活、切段、重交、超时按批按卡计数、建缓冲显存不足时重切、丢的时候泄漏谁，在 [`BackgroundGpuOverTheCards`]
@@ -46,9 +46,8 @@ use std::time::Duration;
 use singlefs_core::allocator::AllocationRecord;
 use singlefs_core::background_compute::{
     AdmittedGpuSettings, BackgroundComputeSettingsRefused, ClusterSegmentEntryKey,
-    GpuComputedSegment, GpuDeviceIdentifier, GpuDeviceNotOpenedAtTheMount, GpuInventory,
-    GpuMemoryInBytes, GpuShortOfFreeMemoryForTheSmallestPiece, GpuSubmissionsOfOneSegment,
-    GpuSubmissionsOfTheBatch,
+    GpuComputedSegment, GpuDeviceIdentifier, GpuInventory, GpuMemoryInBytes,
+    GpuShortOfFreeMemoryForTheSmallestPiece, GpuSubmissionsOfOneSegment, GpuSubmissionsOfTheBatch,
 };
 use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
 use singlefs_core::logged_operations::CompactionRegion;
@@ -71,10 +70,10 @@ use crate::checksum_kernels::{
 };
 use crate::gpus_of_this_host::{
     bytes_of_the_smallest_piece_every_participating_gpu_holds, gpu_left_out_under_its_memory_limit,
-    gpus_dropped_leaking_those_still_running, gpus_opened_at_the_mount,
-    measured_cost_summed_over_the_pieces, memory_limit_of_the_device_under_the_settings,
-    plans_of_a_batch_filling_the_gpus_in_order, BatchNotPlannedOverTheGpus, CostAtTheOpening,
-    CostProbeChunk, DiscreteGpuNotOpenedAtTheMount, DiscreteGpusAfterTheOpening,
+    gpu_opened_under_its_memory_limit, gpus_dropped_leaking_those_still_running,
+    gpus_opened_at_the_mount, measured_cost_summed_over_the_pieces,
+    memory_limit_of_the_device_under_the_settings, plans_of_a_batch_filling_the_gpus_in_order,
+    BatchNotPlannedOverTheGpus, CostAtTheOpening, CostProbeChunk, DiscreteGpusAfterTheOpening,
     GpuBudgetUnderTheSettings, GpuLeakedWhenDropped, GpuLeftOutUnderItsMemoryLimit,
     GpuMemoryLeakedInThisProcess, GpusLeakedWhenTheBackgroundGpuIsDropped,
     ParticipatingGpusOnThisHost, PlanOfAGpuForTheBatch, RoomOfAGpuForTheBatch,
@@ -186,7 +185,7 @@ pub enum BackgroundGpuOpenRefused {
     /// [`Self::SettingsRefused`] 的 `GpuWaitLimitOfZero`。
     AdapterEnumerationFailed { cause: GpuFailureCause },
     /// 一张参与的卡用不上（起不了设备、编不了管线、量代价那几次提交出错或超时）：给了 `background_gpu_devices` 时是点名的卡，没给时本机任何一张独显。
-    /// 给了列表而没点名的卡用不上不拒，跳过（[`BackgroundGpu::gpus_not_opened_at_the_mount`]）。显存上限连最小的一片都装不下的卡也不拒，
+    /// 没点名的卡不开，不走这里。显存上限连最小的一片都装不下的卡不拒，
     /// 不用它、报出来（[`BackgroundGpu::gpus_left_out_under_their_memory_limit`]）。
     GpuUnusable {
         device: GpuDeviceIdentifier,
@@ -317,10 +316,14 @@ impl From<SameEntryKeyTwiceInOneBatch> for LiveSlotBatchFailed {
     }
 }
 
-/// 此刻这组设置下一批分不到卡上（[`BackgroundGpu::batch_not_split_over_the_gpus`]）：换设置时量一张卡的代价出错了（这张卡没参与，报成它的 GPU 出错），
-/// 或参与的卡因显存上限一张都不剩。这组设置下每一批三段都交回它，直到下一次换设置。封闭集合。
+/// 此刻这组设置下一批分不到卡上（[`BackgroundGpu::batch_not_split_over_the_gpus`]）：换设置时开一张挂载时没开的卡没开成、或量一张卡的代价出错了
+/// （这张卡没参与，报成它的 GPU 出错），或参与的卡因显存上限一张都不剩。这组设置下每一批三段都交回它，直到下一次换设置。封闭集合。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum BatchNotSplitOverTheGpus {
+    GpuNotOpenedAtTheSwitch {
+        device: GpuDeviceIdentifier,
+        cause: GpuFailureCause,
+    },
     CostNotMeasuredAtTheSwitch {
         device: GpuDeviceIdentifier,
         cause: GpuFailureCause,
@@ -408,19 +411,35 @@ pub trait CardOfTheBackgroundGpu {
     ) -> Result<Vec<(u64, u64)>, GpuFailureCause>;
 }
 
-/// 这一次挂载的后台 GPU 在几张开着的卡上（`Card` 是一张开着的卡，产品是 [`GpuInUse`]）：开包时开成的本机独显（按总线地址排），没开成而跳过的那几张，
-/// 给了点名时没点名而交不成一张卡、没收进清单的那几张，与此刻参与的那几张（设置定的，按参与的次序：没点名是总线地址的次序，点了名是点名的次序；
-/// 显存上限连最小的一片都装不下的不在里面）。生命周期一次挂载，只住内存。丢它时不等还在 GPU 上跑的提交（`Drop`）。
+/// 切换生效那一刻开一张挂载时没开的卡（同挂载那一刻开卡那一步）：交给它这张卡、这一组给它的显存上限、等 GPU 的上限、量不量代价，
+/// 交回开着的卡或没开成的原因。产品是在这张卡的 wgpu 适配器上起设备、编管线、量代价（[`opening_of_a_gpu_in_use`]），测试里交造出来的卡。
+pub type OpeningOfAGpuAtASwitch<Card> = Box<
+    dyn FnMut(
+            &DiscreteGpuOnThisHost,
+            Option<GpuMemoryInBytes>,
+            Duration,
+            CostAtTheOpening,
+        ) -> Result<Card, GpuFailureCause>
+        + Send,
+>;
+
+/// 这一次挂载的后台 GPU 在几张开着的卡上（`Card` 是一张开着的卡，产品是 [`GpuInUse`]）：开着的本机独显（挂载那一刻开的参与的卡按总线地址排，
+/// 切换那一刻开的依次排在后面）与没参与而没开的那几张，给了点名时没点名而交不成一张卡、没收进清单的那几张，与此刻参与的那几张（设置定的，按参与的次序：
+/// 没点名是总线地址的次序，点了名是点名的次序；显存上限连最小的一片都装不下的不在里面）。生命周期一次挂载，只住内存。丢它时不等还在 GPU 上跑的提交（`Drop`）。
 pub struct BackgroundGpuOverTheCards<Card: CardOfTheBackgroundGpu> {
-    /// 开成的那几张，与 `discrete_gpus_after_the_opening.opened` 一一对应、同序。
+    /// 开着的那几张，与 `discrete_gpus_after_the_opening.opened` 一一对应、同序。
     cards_on_this_host: Vec<Card>,
     discrete_gpus_after_the_opening: DiscreteGpusAfterTheOpening,
     unnamed_discrete_gpus_left_off_the_list: Vec<DiscreteGpuListRefused>,
     participating: ParticipatingGpusOnThisHost,
     /// 此刻这组设置点了而因显存上限装不下最小的一片不用的卡，按参与的次序。
     left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
+    /// 最近一次换设置时开一张它点到的、挂载时没开的卡没开成（这张卡没参与、照旧没开）：之后每一批都报成它的 GPU 出错，直到下一次换设置。
+    not_opened_at_the_last_switch: Option<(GpuDeviceIdentifier, GpuFailureCause)>,
     /// 最近一次换设置时量一张新参与的卡的代价出错（这张卡没参与）：之后每一批都报成它的 GPU 出错，直到下一次换设置。
     cost_not_measured_at_the_last_switch: Option<(GpuDeviceIdentifier, GpuFailureCause)>,
+    /// 切换点到挂载时没开的卡时开它的那一步。
+    opening_of_a_gpu_at_a_switch: OpeningOfAGpuAtASwitch<Card>,
     /// 上一次交回以来三段各自在卡上是怎么交的（[`Self::take_the_submissions_of_the_batch`]）。
     submissions_since_the_last_take: GpuSubmissionsOfTheBatch,
     gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped,
@@ -851,7 +870,7 @@ impl VulkanInstanceHandedToTheOpening for VulkanInstanceOfThisProcess {
     }
 }
 
-/// 挂载那一刻：枚举适配器、判卡清单与校验、在本机每一张独立显卡上起设备与各段两份核、量一次提交的主机 CPU 时间（同
+/// 挂载那一刻：枚举适配器、判卡清单与校验、在参与的卡上起设备与各段两份核、量一次提交的主机 CPU 时间（同
 /// [`open_background_gpu_with_the_memory_leaked_in_this_process`]，进程里还一张卡都没泄漏过；实例是调用方留着的 `vulkan_instance`）。
 ///
 /// # Errors
@@ -868,19 +887,19 @@ pub fn open_background_gpu(
     )
 }
 
-/// 挂载那一刻：枚举适配器、判卡清单与校验、在本机每一张独立显卡上起设备与各段两份核、量一次提交的主机 CPU 时间；参与的卡照 `selection`
+/// 挂载那一刻：枚举适配器、判卡清单与校验、只在参与的卡上起设备与各段两份核、量一次提交的主机 CPU 时间；参与的卡照 `selection`
 /// （之后切换时核心层另交，[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]）。等 wgpu 的 future 都在 `selection.background_gpu_wait_limit` 之内；
 /// 枚举适配器本身不在其内：wgpu 在造 future 那一步同步做完枚举，驱动在枚举里挂住时开卡跟着挂住、不拒（[`VulkanInstanceOfThisProcess::vulkan_adapters_enumerated_within`]）。
 /// 上限给 0 在枚举之前就拒成 [`BackgroundGpuOpenRefused::SettingsRefused`] 的 `GpuWaitLimitOfZero`，不看选了哪几张卡。
 /// 给了 `background_gpu_devices` 时只判点名的卡（没点名的独显交不成一张卡就不收、报出来）。
 /// 每张卡开卡（起设备、编各段两份核的管线、量代价）至多等 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]（五分钟），到点当那张卡没开成
 /// （`waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit`，报等超过上限、带这个上限），挂载等开卡成功或到点才往下走。
-/// 枚举完、判过卡清单与校验，逐张开卡之前把要开几张卡交给 `report_the_gpus_to_open`（进程开卡器照它把外层的期限延到枚举期限加卡数乘单卡上限）；
+/// 枚举完、判过卡清单与校验，逐张开卡之前把要开几张卡（参与的卡数）交给 `report_the_gpus_to_open`（进程开卡器照它把外层的期限延到枚举期限加卡数乘单卡上限）；
 /// 枚举不成、卡清单或校验拒了的不报。
 /// 量代价出错时账上记着还在跑的提交的那张卡泄漏、不析构，这一次占着的字节当场记进 `memory_leaked_in_this_process`
 /// （`failure_of_the_measurement_dropping_or_leaking_the_card`）。
-/// 没参与的卡也开（之后的切换可能点它，切换不重开设备）；一张卡没开成，参与的就拒，给了列表而没点名的跳过、报出来
-/// （[`gpus_opened_at_the_mount`]、[`BackgroundGpuOverTheCards::gpus_not_opened_at_the_mount`]）。参与的卡照开卡那一组给它的显存上限
+/// 没参与的卡不开、不量代价（欠账 C996 的定案「只开参与的卡」）：卡清单照列它，之后的切换点到它时在切换生效那一刻开（[`OpeningOfAGpuAtASwitch`]，
+/// 同一份适配器与泄漏账）。参与的卡没开成就拒绝挂载（[`gpus_opened_at_the_mount`]）。参与的卡照开卡那一组给它的显存上限
 /// （先扣掉这个进程里之前几次挂载泄漏在它上面的字节，`memory_leaked_in_this_process`）把量代价那一段（[`COST_PROBE_CHUNK`]）切成几片逐片量、代价取各片之和；
 /// 显存上限连最小的一片都装不下的卡照开、不量代价、不参与、报出来（[`BackgroundGpuOverTheCards::gpus_left_out_under_their_memory_limit`]），挂载不因此拒绝；
 /// 参与的卡因此一张都不剩时照样交回，三段每一批都报「参与的卡一张都不剩」（[`BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut`]），
@@ -912,32 +931,19 @@ pub fn open_background_gpu_with_the_memory_leaked_in_this_process(
             selection,
             &descriptions,
         )?;
-    report_the_gpus_to_open(discrete_gpus.listed.len());
+    report_the_gpus_to_open(participating.len());
+    let mut open_a_gpu = opening_of_a_gpu_in_use(adapters, memory_leaked_in_this_process.clone());
     let gpus_opened = gpus_opened_at_the_mount(
         discrete_gpus.listed,
         &participating,
         bytes_of_the_smallest_piece_every_participating_gpu_holds(),
         memory_leaked_in_this_process,
         |discrete_gpu, memory_limit_at_the_opening, cost_at_the_opening| {
-            let adapter = adapters[discrete_gpu.position_in_the_enumeration].clone();
-            let discrete_gpu_to_open = discrete_gpu.clone();
-            let wait_limit = selection.background_gpu_wait_limit;
-            let bytes_leaked_by_earlier_mounts =
-                memory_leaked_in_this_process.bytes_leaked_on(&discrete_gpu.identifier);
-            let memory_leaked_noted_by_the_opening = memory_leaked_in_this_process.clone();
-            outcome_of_opening_a_gpu_within_the_limit(
-                WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
-                move || {
-                    started_gpu_with_its_cost_measured(
-                        &adapter,
-                        discrete_gpu_to_open,
-                        memory_limit_at_the_opening,
-                        wait_limit,
-                        cost_at_the_opening,
-                        bytes_leaked_by_earlier_mounts,
-                        memory_leaked_noted_by_the_opening,
-                    )
-                },
+            open_a_gpu(
+                discrete_gpu,
+                memory_limit_at_the_opening,
+                selection.background_gpu_wait_limit,
+                cost_at_the_opening,
             )
         },
     )?;
@@ -952,9 +958,48 @@ pub fn open_background_gpu_with_the_memory_leaked_in_this_process(
             vulkan_instance,
             wait_limit: selection.background_gpu_wait_limit,
         }),
+        Box::new(open_a_gpu),
     ))
 }
 
+/// 在这一次枚举到的适配器（`adapters`，按枚举的次序）里的一张独显上开卡：起设备、编各段两份核的管线、照给它的显存上限量代价（量不量照 `CostAtTheOpening`），
+/// 至多等 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]（到点当那张卡没开成，`waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit`）。
+/// 挂载那一刻开参与的卡与切换生效那一刻开挂载时没开的卡都走它；这个进程里之前几次挂载泄漏在这张卡上的字节从 `memory_leaked_in_this_process` 现读，
+/// 量代价出错而提交还在跑的卡泄漏、记进去（`started_gpu_with_its_cost_measured`）。
+fn opening_of_a_gpu_in_use(
+    adapters: Vec<wgpu::Adapter>,
+    memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
+) -> impl FnMut(
+    &DiscreteGpuOnThisHost,
+    Option<GpuMemoryInBytes>,
+    Duration,
+    CostAtTheOpening,
+) -> Result<GpuInUse, GpuFailureCause>
+       + Send
+       + 'static {
+    move |discrete_gpu, memory_limit_at_the_opening, wait_limit, cost_at_the_opening| {
+        let adapter = adapters[discrete_gpu.position_in_the_enumeration].clone();
+        let discrete_gpu_to_open = discrete_gpu.clone();
+        let bytes_leaked_by_earlier_mounts =
+            memory_leaked_in_this_process.bytes_leaked_on(&discrete_gpu.identifier);
+        let memory_leaked_noted_by_the_opening = memory_leaked_in_this_process.clone();
+        outcome_of_opening_a_gpu_within_the_limit(
+            WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
+            move || {
+                started_gpu_with_its_cost_measured(
+                    &adapter,
+                    discrete_gpu_to_open,
+                    memory_limit_at_the_opening,
+                    wait_limit,
+                    cost_at_the_opening,
+                    bytes_leaked_by_earlier_mounts,
+                    memory_leaked_noted_by_the_opening,
+                )
+            },
+        )
+    }
+}
+
 /// 同一次枚举（`descriptions`）交成卡清单（给了 `background_gpu_devices` 时只判点名的卡）、照它判挂载那一刻的校验、取参与的卡：判与取读同一份清单。
 /// 核心层拒成「点名的卡不在卡清单里」时，拒绝带上这一次枚举到的每个适配器（类型与总线地址）与没点名、没收进清单的那几张
 /// （[`BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration`]）；核心层别的拒照原样交回。
@@ -1902,9 +1947,9 @@ impl CardOfTheBackgroundGpu for GpuInUse {
 }
 
 impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
-    /// 挂载那一刻开卡之后：开成的卡（与 `discrete_gpus_after_the_opening.opened` 一一对应、同序）、开卡之后的本机独显、没点名而没收进清单的那几张、
+    /// 挂载那一刻开卡之后：开成的卡（与 `discrete_gpus_after_the_opening.opened` 一一对应、同序）、开卡之后的本机独显（连同没参与而没开的）、没点名而没收进清单的那几张、
     /// 开卡那一组参与的卡与因显存上限不用的卡（`gpus_of_this_host::gpus_opened_at_the_mount` 交回的），进程里泄漏的那一份（丢它时记进去），
-    /// 与卡清单每次答之前现查本机在的独显的那一样（产品带着开卡用的那一份进程级 Vulkan 实例）。测试拿它把替身卡装进来。
+    /// 卡清单每次答之前现查本机在的独显的那一样（产品带着开卡用的那一份进程级 Vulkan 实例），与切换点到没开的卡时开它的那一步。测试拿它把替身卡装进来。
     ///
     /// # Panics
     /// 开成的卡与 `discrete_gpus_after_the_opening.opened` 张数不等。
@@ -1917,6 +1962,7 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
         memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
         discrete_gpus_present_now: Box<dyn DiscreteGpusPresentNow + Send>,
+        opening_of_a_gpu_at_a_switch: OpeningOfAGpuAtASwitch<Card>,
     ) -> Self {
         assert_eq!(
             cards_on_this_host.len(),
@@ -1929,7 +1975,9 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
             unnamed_discrete_gpus_left_off_the_list,
             participating,
             left_out_under_their_memory_limit,
+            not_opened_at_the_last_switch: None,
             cost_not_measured_at_the_last_switch: None,
+            opening_of_a_gpu_at_a_switch,
             submissions_since_the_last_take: GpuSubmissionsOfTheBatch::default(),
             gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped::not_yet_dropped(),
             memory_leaked_in_this_process,
@@ -1937,10 +1985,13 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         }
     }
 
-    /// 挂载那一刻没开成而跳过的卡（给了 `background_gpu_devices` 而没点名的），按总线地址排，各带原因；没跳过一张是空。
+    /// 此刻开着的卡（挂载那一刻开的参与的卡按总线地址排，切换那一刻开的依次排在后面），参与与否都列。
     #[must_use]
-    pub fn gpus_not_opened_at_the_mount(&self) -> &[DiscreteGpuNotOpenedAtTheMount] {
-        &self.discrete_gpus_after_the_opening.not_opened
+    pub fn gpus_opened(&self) -> Vec<GpuDeviceIdentifier> {
+        self.cards_on_this_host
+            .iter()
+            .map(|card| card.discrete_gpu().identifier.clone())
+            .collect()
     }
 
     /// 给了 `background_gpu_devices` 时没点名、交不成一张卡（缺总线地址、缺设备本地堆、地址重复）而没收进卡清单的独显，各带原因；没有是空。
@@ -1991,21 +2042,25 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
     }
 
     /// 从下一批起照核心层交来的这一组跑（D26 已定项 13 第 1–3 条；核心层只在两批之间交）：参与的卡换成它点的那几张、按它的次序，
-    /// 每张卡的显存上限与等的上限换成它给的（不参与的卡不设显存上限，它分不到活）。不重开设备，各卡的账照旧记着。
+    /// 每张卡的显存上限与等的上限换成它给的（不参与的卡不设显存上限，它分不到活）。开着的卡不重开，各卡的账照旧记着；不再参与的照开着。
+    /// 它点到挂载时没开的卡（欠账 C996 的定案「切换到新卡时再开」），在这一刻按点的次序逐张开（同挂载那一刻开卡那一步，照这一组给它的显存上限与等的上限；
+    /// 装不下最小的一片的照开、不量代价）；开不成的这张卡不参与、照旧没开，之后每一批报成它的 GPU 出错，直到下一次换设置（下一次点它再开）。
     /// 显存上限（扣掉卡上已占的）连最小的一片都装不下的卡不用、报出来（[`Self::gpus_left_out_under_their_memory_limit`]）；开卡时因显存上限没量代价、
     /// 或上一组设置因显存上限晾下而这一组给了装得下的上限的卡，照新上限量一次（量不量照旧、量出的代价不用来分活），量出错的这张卡不参与，
     /// 之后每一批报成它的 GPU 出错，直到下一次换设置。请回来的卡照参与的次序重新参与分活。
     ///
     /// # Panics
-    /// 它点的卡有一张不在开成的独显里：核心层照这份实现答的卡清单（此刻本机在的独显，连同没开成的那几张）校验过，参与的卡里有没开成的就拒了。
+    /// 它点的卡有一张既不在开着的独显里、也不在没开的那几张里：核心层照这份实现答的卡清单（此刻本机在的独显，开着的与没开的）校验过，
+    /// 点名不在清单里的就拒了。
     pub fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
-        let identifiers_on_this_host: Vec<GpuDeviceIdentifier> = self
-            .cards_on_this_host
-            .iter()
-            .map(|card| card.discrete_gpu().identifier.clone())
-            .collect();
-        let participating_by_the_settings =
-            ParticipatingGpusOnThisHost::of_the_settings(settings, &identifiers_on_this_host);
+        let devices_not_opened_at_the_switch =
+            self.open_the_gpus_named_and_not_yet_opened(settings);
+        let identifiers_on_this_host = self.gpus_opened();
+        let participating_by_the_settings = ParticipatingGpusOnThisHost::of_the_settings(
+            settings,
+            &identifiers_on_this_host,
+            &devices_not_opened_at_the_switch,
+        );
         for card in &mut self.cards_on_this_host {
             let memory_limit = memory_limit_of_the_device_under_the_settings(
                 settings,
@@ -2053,13 +2108,73 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         self.cost_not_measured_at_the_last_switch = cost_not_measured;
     }
 
-    /// 此刻这组设置下一批分不分得到卡上（与这一批有几条无关）：换设置时量一张卡的代价出错了，或参与的卡因显存上限一张都不剩，就交回是哪一种；
-    /// 分得到交回 `None`。挂载那一刻核心层经 `BackgroundGpuCompute::failure_under_the_settings_handed_in` 读它，不等头一批。
+    /// 切换生效那一刻开这一组点到的、挂载时没开的卡，按参与的次序逐张开（[`gpu_opened_under_its_memory_limit`]，开卡那一步是 [`OpeningOfAGpuAtASwitch`]）：
+    /// 开成的排在开着的卡末尾；头一张开不成的记成这一组下每一批都报的 GPU 出错（[`BatchNotSplitOverTheGpus::GpuNotOpenedAtTheSwitch`]）。
+    /// 交回开不成的那几张（照旧没开，不参与）。
+    fn open_the_gpus_named_and_not_yet_opened(
+        &mut self,
+        settings: &AdmittedGpuSettings,
+    ) -> Vec<GpuDeviceIdentifier> {
+        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
+        let mut devices_not_opened_at_the_switch = Vec::new();
+        let mut not_opened_at_the_switch = None;
+        for participating in settings.participating_devices() {
+            let Some(discrete_gpu_not_yet_opened) = self
+                .discrete_gpus_after_the_opening
+                .not_opened_for_not_taking_part
+                .iter()
+                .find(|discrete_gpu| discrete_gpu.identifier == participating.device)
+                .cloned()
+            else {
+                continue;
+            };
+            let opening_of_a_gpu_at_a_switch = &mut self.opening_of_a_gpu_at_a_switch;
+            let opened = gpu_opened_under_its_memory_limit(
+                &discrete_gpu_not_yet_opened,
+                participating.memory_limit,
+                smallest_piece_bytes,
+                &self.memory_leaked_in_this_process,
+                |discrete_gpu, memory_limit, cost_at_the_opening| {
+                    opening_of_a_gpu_at_a_switch(
+                        discrete_gpu,
+                        memory_limit,
+                        settings.wait_limit(),
+                        cost_at_the_opening,
+                    )
+                },
+            );
+            match opened {
+                // 装不装得下最小的一片在 `use_the_settings_from_the_next_batch` 里照这张卡的账再判一次（与开着的卡同一判）。
+                Ok((card, _left_out_judged_again_with_the_other_cards)) => {
+                    self.cards_on_this_host.push(card);
+                    self.discrete_gpus_after_the_opening
+                        .note_the_gpu_opened_at_a_switch(&discrete_gpu_not_yet_opened.identifier);
+                }
+                Err(cause) => {
+                    devices_not_opened_at_the_switch
+                        .push(discrete_gpu_not_yet_opened.identifier.clone());
+                    not_opened_at_the_switch
+                        .get_or_insert((discrete_gpu_not_yet_opened.identifier, cause));
+                }
+            }
+        }
+        self.not_opened_at_the_last_switch = not_opened_at_the_switch;
+        devices_not_opened_at_the_switch
+    }
+
+    /// 此刻这组设置下一批分不分得到卡上（与这一批有几条无关）：换设置时开一张挂载时没开的卡没开成、量一张卡的代价出错了，或参与的卡因显存上限一张都不剩，
+    /// 就交回是哪一种；分得到交回 `None`。挂载那一刻与切换生效那一批开头核心层经 `BackgroundGpuCompute::failure_under_the_settings_handed_in` 读它，不等头一批。
     ///
     /// # Panics
-    /// 参与的卡一张都不剩，却既没有量代价出错、也没有因显存上限不用的卡：核心层校验过的一组至少点一张卡，没参与的只有这两种原因。
+    /// 参与的卡一张都不剩，却既没有开卡没开成、量代价出错，也没有因显存上限不用的卡：核心层校验过的一组至少点一张卡，没参与的只有这三种原因。
     #[must_use]
     pub fn batch_not_split_over_the_gpus(&self) -> Option<BatchNotSplitOverTheGpus> {
+        if let Some((device, cause)) = &self.not_opened_at_the_last_switch {
+            return Some(BatchNotSplitOverTheGpus::GpuNotOpenedAtTheSwitch {
+                device: device.clone(),
+                cause: cause.clone(),
+            });
+        }
         if let Some((device, cause)) = &self.cost_not_measured_at_the_last_switch {
             return Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
                 device: device.clone(),
@@ -2071,7 +2186,7 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         }
         assert!(
             !self.left_out_under_their_memory_limit.is_empty(),
-            "参与的卡一张都不剩只有两种：换设置时量代价出错（上面交回了），或这组设置点的卡都因显存上限不用"
+            "参与的卡一张都不剩只有三种：换设置时开卡没开成、量代价出错（上面交回了），或这组设置点的卡都因显存上限不用"
         );
         Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(
             self.left_out_under_their_memory_limit.clone(),
@@ -2100,7 +2215,10 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         submissions_of_the_plan: impl Fn(&Plan) -> usize,
     ) -> Result<Vec<PlanOfAGpuForTheBatch<Plan>>, OutcomesOfTheBatchNotComputed> {
         match self.batch_not_split_over_the_gpus() {
-            Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause }) => {
+            Some(
+                BatchNotSplitOverTheGpus::GpuNotOpenedAtTheSwitch { device, cause }
+                | BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause },
+            ) => {
                 return Err(OutcomesOfTheBatchNotComputed::GpuFailed { device, cause });
             }
             Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)) => {
@@ -2655,11 +2773,9 @@ fn failure_of_the_measurement_dropping_or_leaking_the_card<Card: CardOfTheBackgr
 }
 
 /// 卡清单（核心层挂载与切换照它校验设置，D26（后台整理与放置回收） 已定项 13 第 2、4 条）：此刻本机在的独显与各自的显存总量，按总线地址排
-/// （挂载那一刻没开成的照列；没给 `background_gpu_devices` 时上限按这个次序一张卡一个），连同没开成的那几张与原因（[`DiscreteGpusAfterTheOpening`]）。
+/// （开着的与没参与而没开的都列，切换可以点没开的卡；没给 `background_gpu_devices` 时上限按这个次序一张卡一个；[`DiscreteGpusAfterTheOpening`]）。
 /// 答卡清单之前现查本机在的独显（[`DiscreteGpusPresentNow`]）：挂载之后掉了的卡不列，切换点它照「点名的卡不存在」拒、不点名时它不参与、一张都不剩照「GPU 不在」拒。
-/// 没开成的那几张是挂载那一刻定下的，答它不现查（一次切换只现枚举一次，判与取参与的卡读同一份；
-/// 两样各现查一次时，第二次枚举答空就把点名一张没开成的卡的切换收下了），掉了的没开成的卡照列，它不在卡清单里、不参与。
-/// 切换可以点开包时没参与而开成了的卡；参与的卡里有没开成的照第 4 条拒。
+/// 挂载那一刻没有没开成而跳过的卡（参与的卡没开成就拒绝挂载，不参与的不开），挂载那一刻没开成的那几张答空（核心层的缺省）。
 impl<Card: CardOfTheBackgroundGpu> GpuInventory for BackgroundGpuOverTheCards<Card> {
     fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
         self.discrete_gpus_after_the_opening
@@ -2670,11 +2786,6 @@ impl<Card: CardOfTheBackgroundGpu> GpuInventory for BackgroundGpuOverTheCards<Ca
             )
             .devices_with_total_memory()
     }
-
-    fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
-        self.discrete_gpus_after_the_opening
-            .devices_not_opened_at_the_mount()
-    }
 }
 
 /// 这条线程到此刻用掉的 CPU 时间（`CLOCK_THREAD_CPUTIME_ID`）：线程睡着、等 GPU 时不走。
@@ -3249,18 +3360,31 @@ mod tests {
                 cards,
                 DiscreteGpusAfterTheOpening {
                     opened: discrete_gpus,
-                    not_opened: Vec::new(),
+                    not_opened_for_not_taking_part: Vec::new(),
                 },
                 Vec::new(),
                 participating,
                 Vec::new(),
                 memory_leaked.clone(),
                 Box::new(DiscreteGpusPresentForTests(Arc::clone(&present))),
+                no_opening_at_a_switch(),
             ),
             present,
         )
     }
 
+    /// 本机的卡挂载那一刻全开了时交的开卡那一步：切换点不到没开的卡，调到它就是用例错了。
+    fn no_opening_at_a_switch() -> OpeningOfAGpuAtASwitch<CardComputedOnTheCpuForTests> {
+        Box::new(
+            |discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
+                panic!(
+                    "用例的卡挂载那一刻全开了，切换不该开 {}",
+                    discrete_gpu.pci_bus_address
+                )
+            },
+        )
+    }
+
     /// `count` 个单元，第 i 个是 `bytes_per_unit` 个字节的 i，期望值照 CPU 参照算对。
     fn unit_bytes_of(count: usize, bytes_per_unit: usize) -> Vec<Vec<u8>> {
         (0..count)
@@ -3872,87 +3996,184 @@ mod tests {
         );
     }
 
-    /// 切换那一刻现查的「此刻本机在的独显」：头一次枚举交回 `present_while_nothing_has_enumerated_yet`，之后的枚举一张都没有
-    /// （等 wgpu 超过上限当一张都不在，或两次之间掉了卡）；枚举了几次记在 `enumerations`。
-    struct DiscreteGpusPresentOnlyUntilTheEnumerationIsRepeated {
-        enumerations: Arc<std::sync::atomic::AtomicUsize>,
-        present_while_nothing_has_enumerated_yet: Vec<GpuDeviceIdentifier>,
-    }
-
-    impl DiscreteGpusPresentNow for DiscreteGpusPresentOnlyUntilTheEnumerationIsRepeated {
-        fn identifiers_of_the_discrete_gpus_present_now(&self) -> Vec<GpuDeviceIdentifier> {
-            if self
-                .enumerations
-                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
-                == 0
-            {
-                self.present_while_nothing_has_enumerated_yet.clone()
-            } else {
-                Vec::new()
-            }
-        }
-    }
-
-    /// 本机两张独显，0000:41:00.0 挂载那一刻没开成；切换点名它，现查本机在的独显头一次答两张都在、之后答一张都没有——
-    /// 一次切换只现枚举一次（判与取参与的卡读同一份），照第 4 条拒成「参与的卡里有挂载那一刻没开成的」，不收下、不交给实现（交了就在外壳里 panic）。
-    #[test]
-    fn the_switch_naming_a_card_not_opened_at_the_mount_is_refused_on_a_single_enumeration_though_a_repeated_one_would_come_back_empty(
+    /// 本机两张独显：「0000:01:00.0」挂载那一刻参与、开着；「0000:41:00.0」没参与、没开。切换点它时开它那一步交 `opening_at_the_switch`；
+    /// 开卡那一步被交了哪几张卡记在交回的那一份里。
+    fn background_gpu_with_one_card_not_opened_at_the_mount(
+        opening_at_the_switch: impl FnMut(
+                &DiscreteGpuOnThisHost,
+                Option<GpuMemoryInBytes>,
+                Duration,
+                CostAtTheOpening,
+            ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
+            + Send
+            + 'static,
+    ) -> (
+        BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
+        Arc<Mutex<Vec<String>>>,
     ) {
         let opened_card = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
         let opened = opened_card.discrete_gpu.clone();
-        let not_opened = DiscreteGpuNotOpenedAtTheMount {
-            discrete_gpu: DiscreteGpuOnThisHost {
-                position_in_the_enumeration: 1,
-                identifier: card_identifier("0000:41:00.0"),
-                pci_bus_address: "0000:41:00.0".to_string(),
-                name: "cpu card that did not open at the mount".to_string(),
-                total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
-            },
-            cause: GpuFailureCause::DeviceOrPipelineDidNotStart {
-                reason: "替身：挂载那一刻没开成".to_string(),
-            },
+        let not_opened = DiscreteGpuOnThisHost {
+            position_in_the_enumeration: 1,
+            identifier: card_identifier("0000:41:00.0"),
+            pci_bus_address: "0000:41:00.0".to_string(),
+            name: "cpu card not opened at the mount".to_string(),
+            total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
         };
-        let enumerations = Arc::new(std::sync::atomic::AtomicUsize::new(0));
+        let cards_handed_to_the_opening = Arc::new(Mutex::new(Vec::new()));
+        let cards_noted_by_the_opening = Arc::clone(&cards_handed_to_the_opening);
+        let mut opening_at_the_switch = opening_at_the_switch;
         let background_gpu = BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
             vec![opened_card],
             DiscreteGpusAfterTheOpening {
                 opened: vec![opened],
-                not_opened: vec![not_opened],
+                not_opened_for_not_taking_part: vec![not_opened],
             },
             Vec::new(),
             ParticipatingGpusOnThisHost::of_the_positions(vec![0]),
             Vec::new(),
             GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
-            Box::new(DiscreteGpusPresentOnlyUntilTheEnumerationIsRepeated {
-                enumerations: Arc::clone(&enumerations),
-                present_while_nothing_has_enumerated_yet: vec![
-                    card_identifier("0000:01:00.0"),
-                    card_identifier("0000:41:00.0"),
-                ],
-            }),
+            Box::new(DiscreteGpusPresentForTests(Arc::new(Mutex::new(vec![
+                card_identifier("0000:01:00.0"),
+                card_identifier("0000:41:00.0"),
+            ])))),
+            Box::new(
+                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
+                    cards_noted_by_the_opening
+                        .lock()
+                        .expect("用例没在别处 panic 过")
+                        .push(discrete_gpu.pci_bus_address.clone());
+                    opening_at_the_switch(
+                        discrete_gpu,
+                        memory_limit,
+                        wait_limit,
+                        cost_at_the_opening,
+                    )
+                },
+            ),
         );
-        let admitted = admit_gpu_settings_against_the_inventory(
+        (background_gpu, cards_handed_to_the_opening)
+    }
+
+    /// 核心层照外壳答的卡清单校验点「0000:01:00.0」「0000:41:00.0」两张的切换（照这个次序）。
+    fn switch_naming_both_cards_admitted_against(
+        background_gpu: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
+    ) -> AdmittedGpuSettings {
+        admit_gpu_settings_against_the_inventory(
             &BackgroundComputeSettings {
                 background_compute: BackgroundComputeBackend::Gpu,
-                background_gpu_devices: Some(vec![card_identifier("0000:41:00.0")]),
+                background_gpu_devices: Some(vec![
+                    card_identifier("0000:01:00.0"),
+                    card_identifier("0000:41:00.0"),
+                ]),
                 background_gpu_memory_limit: None,
                 background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
             },
-            &background_gpu,
-        );
+            background_gpu,
+        )
+        .expect("卡清单列着挂载时没开的那张：切换点它照收")
+    }
+
+    /// 欠账 C996 的定案「切换到新卡时再开」：挂载点「0000:01:00.0」、切换点它与「0000:41:00.0」——核心层照卡清单收下这一次切换；
+    /// 切换生效那一刻（交设置）开「0000:41:00.0」，只开这一张、只开一次，之后两张都参与、按点名的次序；再交同一组不重开。
+    #[test]
+    fn the_switch_naming_a_card_not_opened_at_the_mount_opens_that_card_when_the_switch_takes_effect(
+    ) {
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_one_card_not_opened_at_the_mount(
+                |discrete_gpu, memory_limit, _wait_limit, _cost_at_the_opening| {
+                    Ok(CardComputedOnTheCpuForTests::of_the_address(
+                        &discrete_gpu.pci_bus_address,
+                        memory_limit.map(|memory_limit| memory_limit.0),
+                        0,
+                    ))
+                },
+            );
+        let admitted = switch_naming_both_cards_admitted_against(&background_gpu);
         assert!(
-            matches!(
-                &admitted,
-                Err(BackgroundComputeSettingsRefused::ParticipatingGpuDeviceNotOpenedAtTheMount(
-                    not_opened_at_the_mount
-                )) if not_opened_at_the_mount.device == card_identifier("0000:41:00.0")
-            ),
-            "点名挂载那一刻没开成的卡：拒（{admitted:?}）"
+            cards_handed_to_the_opening
+                .lock()
+                .expect("用例没在别处 panic 过")
+                .is_empty(),
+            "校验那一刻不开卡"
+        );
+        background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        assert_eq!(
+            *cards_handed_to_the_opening
+                .lock()
+                .expect("用例没在别处 panic 过"),
+            vec!["0000:41:00.0".to_string()],
+            "切换生效那一刻只开挂载时没开的那一张"
         );
         assert_eq!(
-            enumerations.load(std::sync::atomic::Ordering::SeqCst),
+            background_gpu.gpus_opened(),
+            vec![
+                card_identifier("0000:01:00.0"),
+                card_identifier("0000:41:00.0")
+            ]
+        );
+        assert_eq!(
+            background_gpu
+                .participating_gpus()
+                .into_iter()
+                .map(|participating| participating.identifier)
+                .collect::<Vec<_>>(),
+            vec![
+                card_identifier("0000:01:00.0"),
+                card_identifier("0000:41:00.0")
+            ],
+            "两张都参与、按点名的次序"
+        );
+        assert_eq!(background_gpu.batch_not_split_over_the_gpus(), None);
+        background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        assert_eq!(
+            cards_handed_to_the_opening
+                .lock()
+                .expect("用例没在别处 panic 过")
+                .len(),
             1,
-            "一次切换只现枚举一次"
+            "开着的卡不重开"
+        );
+    }
+
+    /// 切换生效那一刻开挂载时没开的那张卡没开成：它不参与、照旧没开，另一张照常参与；这一组下每一批都报成它的 GPU 出错（带卡与原因），
+    /// 核心层在切换生效那一批开头读到、当场报警停着（`failure_under_the_settings_handed_in`）。
+    #[test]
+    fn the_card_that_does_not_open_when_the_switch_takes_effect_takes_no_part_and_every_batch_reports_its_failure(
+    ) {
+        let reason_it_did_not_open = GpuFailureCause::DeviceOrPipelineDidNotStart {
+            reason: "替身：切换那一刻起不了设备".to_string(),
+        };
+        let cause_handed_back = reason_it_did_not_open.clone();
+        let (mut background_gpu, _cards_handed_to_the_opening) =
+            background_gpu_with_one_card_not_opened_at_the_mount(
+                move |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
+                    Err(cause_handed_back.clone())
+                },
+            );
+        let admitted = switch_naming_both_cards_admitted_against(&background_gpu);
+        background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        assert_eq!(
+            background_gpu.batch_not_split_over_the_gpus(),
+            Some(BatchNotSplitOverTheGpus::GpuNotOpenedAtTheSwitch {
+                device: card_identifier("0000:41:00.0"),
+                cause: reason_it_did_not_open,
+            }),
+            "开不成的那张报成它的 GPU 出错"
+        );
+        assert_eq!(
+            background_gpu.gpus_opened(),
+            vec![card_identifier("0000:01:00.0")],
+            "开不成的那张照旧没开"
+        );
+        assert_eq!(
+            background_gpu
+                .participating_gpus()
+                .into_iter()
+                .map(|participating| participating.identifier)
+                .collect::<Vec<_>>(),
+            vec![card_identifier("0000:01:00.0")],
+            "开不成的那张不参与"
         );
     }
 
````

### crates/singlefs-background-gpu/src/background_gpu_compute.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/background_gpu_compute.rs b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
index 8526bbc..48c8099 100644
--- a/crates/singlefs-background-gpu/src/background_gpu_compute.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
@@ -14,8 +14,8 @@
 //!
 //! 两份核各算一次的那一层抽成 [`BothKernelsOfTheThreeSegments`]：产品是 `background_gpu::BackgroundGpu`（参与的卡上两份 WGSL 核）；
 //! 测试里换上在 CPU 上算的替身，比对、计数、映射这几样不起卡就钉得住。
-//! 挂载那一刻的入口 [`open_background_gpu_compute`]：开卡（卡清单、显存上限、等 GPU 的上限，校验同核心层 `admit_background_compute_settings`），
-//! 交回核心层挂载与切换收的把手，连同没开成而跳过的卡。
+//! 挂载那一刻的入口 [`open_background_gpu_compute`]：只在参与的卡上开卡（卡清单、显存上限、等 GPU 的上限，校验同核心层 `admit_background_compute_settings`），
+//! 交回核心层挂载与切换收的把手；切换点到挂载时没开的卡，切换生效那一刻再开（`background_gpu::BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`）。
 
 use singlefs_core::allocator::AllocationRecord;
 use singlefs_core::background_compute::{
@@ -41,7 +41,7 @@ use crate::checksum_kernels::{
     reconciled_unit_checksums, ChecksumKernelOutcome, UnitChecksumToVerify,
 };
 use crate::gpus_of_this_host::{
-    DiscreteGpuNotOpenedAtTheMount, GpuLeftOutUnderItsMemoryLimit, GpuMemoryLeakedInThisProcess,
+    GpuLeftOutUnderItsMemoryLimit, GpuMemoryLeakedInThisProcess,
     GpusLeakedWhenTheBackgroundGpuIsDropped,
 };
 use crate::live_slot_kernels::reconciled_live_slots_of_the_segments;
@@ -173,8 +173,7 @@ impl<Kernels> BackgroundGpuComputeThroughBothKernels<Kernels> {
     }
 }
 
-/// 挂载那一刻开后台 GPU 的结局：核心层挂载与切换收的把手，没开成而跳过的卡（给了 `background_gpu_devices` 而没点名的，按总线地址排、各带原因；
-/// 没跳过一张是空），与开卡那一组点了而因显存上限连最小的一片都装不下、不用的卡（按参与的次序，各带上限与至少要多少；第 3 条
+/// 挂载那一刻开后台 GPU 的结局：核心层挂载与切换收的把手，挂载那一刻开着的卡（只有参与的卡，按总线地址排），与开卡那一组点了而因显存上限连最小的一片都装不下、不用的卡（按参与的次序，各带上限与至少要多少；第 3 条
 /// 「连最小的一片都装不下，这张卡就不用」）。参与的卡因此一张都不剩时挂载不拒：核心层挂载那一刻把校验过的那一组交给实现、读
 /// [`BackgroundGpuCompute::failure_under_the_settings_handed_in`]，读到 `EveryParticipatingGpuLeftOutUnderItsMemoryLimit` 就当场报警停着。
 /// 给了 `background_gpu_devices` 时没点名而交不成一张卡、没收进卡清单的独显各带原因（`unnamed_discrete_gpus_left_off_the_list`）；
@@ -182,16 +181,16 @@ impl<Kernels> BackgroundGpuComputeThroughBothKernels<Kernels> {
 #[derive(Debug)]
 pub struct OpenedBackgroundGpuCompute {
     pub gpu_compute: BackgroundGpuComputeHandle,
-    pub gpus_not_opened_at_the_mount: Vec<DiscreteGpuNotOpenedAtTheMount>,
+    pub gpus_opened_at_the_mount: Vec<GpuDeviceIdentifier>,
     pub gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
     pub unnamed_discrete_gpus_left_off_the_list: Vec<DiscreteGpuListRefused>,
     pub gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped,
 }
 
 /// 挂载那一刻开后台 GPU、交回核心层挂载与切换收的把手（D26（后台整理与放置回收） 已定项 13「挂载时由调用方把它的实现递进去」）：
-/// 开本机全部独显、逐张量代价，`selection` 定开卡时参与的卡（`BackgroundGpuSelection::of_the_settings` 照挂载那一组取；本包开卡时照它校验一次，
-/// 同核心层 `admit_background_compute_settings`）。给了列表而没点名的卡没开成就跳过、在交回的结局里报出来，参与的卡没开成就拒。
-/// 核心层挂载时照这份实现答的卡清单（此刻本机在的独显，连同没开成的那几张）再校验一次，并把校验过的那一组交过来
+/// 只开参与的卡、逐张量代价，`selection` 定开卡时参与的卡（`BackgroundGpuSelection::of_the_settings` 照挂载那一组取；本包开卡时照它校验一次，
+/// 同核心层 `admit_background_compute_settings`）。没点名的卡不开，参与的卡没开成就拒。
+/// 核心层挂载时照这份实现答的卡清单（此刻本机在的独显，开着的与没开的）再校验一次，并把校验过的那一组交过来
 /// （`use_the_settings_from_the_next_batch`），之后每次切换生效时另交，参与的卡、显存上限、等的上限都照核心层交来的那一组。
 /// `memory_leaked_in_this_process` 是调用方在进程里留着的那一份：之前几次挂载泄漏在一张卡上的字节先从它的余量里扣掉，这一次卸载泄漏的记进去。
 /// `vulkan_instance` 同样是调用方在进程里留着的那一份（[`VulkanInstanceOfThisProcess`]）：开卡的枚举与之后现查本机在的独显都用它，不另建。
@@ -214,13 +213,13 @@ pub fn open_background_gpu_compute(
     Ok(background_gpu_compute_of_the_opened_gpu(background_gpu))
 }
 
-/// 开成的后台 GPU 交成核心层挂载与切换收的把手，连同开卡时跳过、因显存上限不用、没收进清单的卡与丢掉之后泄漏了哪几张的那一份
+/// 开成的后台 GPU 交成核心层挂载与切换收的把手，连同开着的卡、因显存上限不用、没收进清单的卡与丢掉之后泄漏了哪几张的那一份
 /// （[`OpenedBackgroundGpuCompute`]）。进程开卡器（`crate::gpu_opener_of_this_process`）自己开卡（要枚举完报回要开几张卡）之后也走这一步。
 #[must_use]
 pub fn background_gpu_compute_of_the_opened_gpu(
     background_gpu: BackgroundGpu,
 ) -> OpenedBackgroundGpuCompute {
-    let gpus_not_opened_at_the_mount = background_gpu.gpus_not_opened_at_the_mount().to_vec();
+    let gpus_opened_at_the_mount = background_gpu.gpus_opened();
     let gpus_left_out_under_their_memory_limit = background_gpu
         .gpus_left_out_under_their_memory_limit()
         .to_vec();
@@ -232,7 +231,7 @@ pub fn background_gpu_compute_of_the_opened_gpu(
         gpu_compute: BackgroundGpuComputeHandle::of_an_implementation(
             BackgroundGpuComputeOnTheParticipatingGpus::with_no_entry_uncertain(background_gpu),
         ),
-        gpus_not_opened_at_the_mount,
+        gpus_opened_at_the_mount,
         gpus_left_out_under_their_memory_limit,
         unnamed_discrete_gpus_left_off_the_list,
         gpus_leaked_when_dropped,
@@ -380,14 +379,15 @@ pub fn core_failure_of_every_participating_gpu_short_of_free_memory(
     }
 }
 
-/// 此刻这组设置下一批分不到卡上，报给核心层：换设置时量代价出错照 [`core_failure_of_the_gpu`]，参与的卡一张都不剩照
+/// 此刻这组设置下一批分不到卡上，报给核心层：换设置时开卡没开成、量代价出错照 [`core_failure_of_the_gpu`]，参与的卡一张都不剩照
 /// [`core_failure_of_every_participating_gpu_left_out`]。三段交回的与挂载那一刻核心层读到的都照它报。
 #[must_use]
 pub fn core_failure_of_the_batch_not_split_over_the_gpus(
     not_split: &BatchNotSplitOverTheGpus,
 ) -> BackgroundGpuComputeFailed {
     match not_split {
-        BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause } => {
+        BatchNotSplitOverTheGpus::GpuNotOpenedAtTheSwitch { device, cause }
+        | BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause } => {
             core_failure_of_the_gpu(device, cause)
         }
         BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out) => {
````

### crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs b/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
index 8cbb2e4..ab4c284 100644
--- a/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
+++ b/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
@@ -4,7 +4,7 @@
 //!
 //! 开卡（枚举适配器在内：wgpu 在造 future 那一步同步做完枚举，等待上限管不到它）放工作线程，调用线程的期限分两步（[`OpeningDeadlinesOfTheOpener`]）：
 //! 建实例、枚举、判卡清单与校验那一步至多等枚举期限；工作线程枚举完报回要开几张卡，期限延到「枚举期限 + 卡数 × 单卡上限」
-//! （开卡按卡串行、没参与的卡也开；每张卡自己至多等单卡上限，到点当那张卡没开成，由 `waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit` 管）。
+//! （开卡按卡串行、只开参与的卡，卡数是参与的卡数；每张卡自己至多等单卡上限，到点当那张卡没开成，由 `waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit` 管）。
 //! 到点挂载拒绝（[`BackgroundGpuNotOpened::TimedOut`] 带到点那一刻的期限），工作线程留在驱动里不杀（没有安全的杀法）；它回来之前闸住新开卡，不再多起一条枚举线程
 //! （[`BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung`]；同一个驱动加载器里再起一条会不会也挂没量过）。它回来之后闸放开、开出来的东西没人要：
 //! 开成的把手照卸载丢（还记着等超过上限的提交的卡泄漏、记进泄漏账；开成时量代价的提交都已做完，没有这种卡），开卡途中量代价出错而提交还在跑的卡
@@ -169,8 +169,8 @@ fn the_background_gpu_opened_by_the_product_path(
     .map(background_gpu_compute_of_the_opened_gpu)
     .map(|opened| OpenedBackgroundGpu {
         report_of_the_opening: format!(
-            "gpus_not_opened={:?} gpus_left_out={:?} unnamed_gpus_left_off={:?}",
-            opened.gpus_not_opened_at_the_mount,
+            "gpus_opened={:?} gpus_left_out={:?} unnamed_gpus_left_off={:?}",
+            opened.gpus_opened_at_the_mount,
             opened.gpus_left_out_under_their_memory_limit,
             opened.unnamed_discrete_gpus_left_off_the_list
         ),
````

### crates/singlefs-background-gpu/src/gpus_of_this_host.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/gpus_of_this_host.rs b/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
index 8f86424..0bf1438 100644
--- a/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
+++ b/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
@@ -1,11 +1,12 @@
-//! 挂载那一刻开着的本机全部独显里，此刻照哪一组设置用哪几张、每张怎么切、等多久（D26（后台整理与放置回收） 已定项 13 第 1–3 条；
-//! `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 43 批交回那一行的定法：挂载那一刻开全部独显并量代价，设置决定哪几张参与，切换不重开）。
-//! 纯逻辑，不碰 GPU：外壳（`crate::background_gpu`）每张卡一份 [`GpuBudgetUnderTheSettings`]，参与的卡由 [`ParticipatingGpusOnThisHost`] 定；
+//! 开着的本机独显里，此刻照哪一组设置用哪几张、每张怎么切、等多久（D26（后台整理与放置回收） 已定项 13 第 1–3 条）。
+//! 只开参与的卡（欠账 C996 的定案）：挂载那一刻只在参与的卡上起设备、量代价（给了 `background_gpu_devices` 是点名的那几张，没给是本机全部独显），
+//! 没点名的卡不开、不量，卡清单照列它（[`DiscreteGpusAfterTheOpening::not_opened_for_not_taking_part`]）；挂着切换点到它，切换生效那一刻开它
+//! （外壳 `crate::background_gpu::BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`）。开着的卡切换之后不再参与的照开着、不关：
+//! 它上面可能还有等超过上限、没在 GPU 上做完的提交（关它要等那次提交，第 4 条不等），再点它不用重开。
+//! 纯逻辑，不碰 GPU：外壳每张开着的卡一份 [`GpuBudgetUnderTheSettings`]，参与的卡由 [`ParticipatingGpusOnThisHost`] 定；
 //! 核心层交来一组新设置（`BackgroundGpuCompute::use_the_settings_from_the_next_batch`，只在两批之间）就换这两样，从下一批起照它分活、切段、等。
 //!
-//! 挂载那一刻一张卡没开成挡不挡挂载（第 2、4 条与同一份记录第 45 批交回那一行的定法）在 [`gpus_opened_at_the_mount`]：参与的卡没开成就拒，
-//! 给了 `background_gpu_devices` 而没点名的卡没开成就跳过、报出来（[`DiscreteGpusAfterTheOpening`]），卡清单照列它、并报它没开成，
-//! 核心层挂着切换点到它照第 4 条拒（校验同挂载那一刻）。
+//! 挂载那一刻参与的卡没开成就拒绝挂载（第 4 条）：[`gpus_opened_at_the_mount`]。
 //!
 //! 一批的活按卡的次序填满（用户 2026-11-02 定「按卡的顺序填满」）：参与的卡排成参与的次序，前面的卡先填到它照显存上限切片之后一次能放下的量，
 //! 装不下的才给后面的卡，排在最后的那张拿剩下的全部（[`plans_of_a_batch_filling_the_gpus_in_order`]）；挂载时量的代价照量、不用来分活。
@@ -26,15 +27,13 @@ use std::sync::{Arc, Mutex};
 use std::time::Duration;
 
 use singlefs_core::background_compute::{
-    AdmittedGpuSettings, GpuDeviceIdentifier, GpuDeviceNotOpenedAtTheMount, GpuInventory,
-    GpuMemoryInBytes,
+    AdmittedGpuSettings, GpuDeviceIdentifier, GpuInventory, GpuMemoryInBytes,
 };
 
 use crate::adapter_inventory::{DiscreteGpuOnThisHost, ParticipatingGpu};
 use crate::background_gpu::{
     BackgroundGpuOpenRefused, GpuFailureCause, SmallestPieceNotHeldOnTheGpu,
 };
-use crate::background_gpu_compute::core_failure_of_the_gpu;
 use crate::checksum_kernels::bytes_of_the_smallest_checksum_piece;
 use crate::live_slot_kernels::bytes_of_the_smallest_live_slot_piece;
 use crate::measured_host_cpu_time_per_submission::HostCpuNanosecondsPerSubmission;
@@ -217,30 +216,29 @@ pub struct ParticipatingGpusOnThisHost {
 }
 
 impl ParticipatingGpusOnThisHost {
-    /// 核心层交来的那一组里参与的卡，在 `identifiers_on_this_host`（挂载那一刻开成的本机独显，按总线地址排）里找到各自的位置。
+    /// 核心层交来的那一组里参与的卡，在 `identifiers_on_this_host`（开着的本机独显，与外壳开着的卡同序）里找到各自的位置；
+    /// `devices_not_opened_at_the_switch`（切换生效那一刻要开而没开成的）不在里面、跳过，可以一张都不剩。
     ///
     /// # Panics
-    /// 有一张参与的卡不在开成的独显里：核心层照这份实现答的卡清单（本机全部独显，连同它报的没开成的那几张）校验过这一组，
-    /// 点名不在本机的、参与的卡里有没开成的都拒了。
+    /// 有一张参与的卡既不在开着的独显里、也不是切换那一刻没开成的：核心层照这份实现答的卡清单（开着的与没开的本机独显）校验过这一组，
+    /// 外壳在交这一组之前把点到的没开的卡都开过一遍。
     #[must_use]
     pub fn of_the_settings(
         settings: &AdmittedGpuSettings,
         identifiers_on_this_host: &[GpuDeviceIdentifier],
+        devices_not_opened_at_the_switch: &[GpuDeviceIdentifier],
     ) -> Self {
         let positions_on_this_host: Vec<usize> = settings
             .participating_devices()
             .iter()
+            .filter(|participating| !devices_not_opened_at_the_switch.contains(&participating.device))
             .map(|participating| {
                 identifiers_on_this_host
                     .iter()
                     .position(|on_this_host| *on_this_host == participating.device)
-                    .expect("核心层照卡清单校验过：参与的卡都在本机、都不是挂载那一刻没开成的")
+                    .expect("核心层照卡清单校验过、外壳开过点到的没开的卡：参与的卡都开着，除了切换那一刻没开成的")
             })
             .collect();
-        assert!(
-            !positions_on_this_host.is_empty(),
-            "核心层校验过的一组至少一张卡"
-        );
         Self {
             positions_on_this_host,
         }
@@ -277,44 +275,47 @@ impl ParticipatingGpusOnThisHost {
     }
 }
 
-/// 本机的一张独显挂载那一刻没开成（起不了设备、编不了管线、量代价那几次提交出错或超时）。
-#[derive(Clone, Debug, PartialEq, Eq)]
-pub struct DiscreteGpuNotOpenedAtTheMount {
-    pub discrete_gpu: DiscreteGpuOnThisHost,
-    pub cause: GpuFailureCause,
-}
-
 impl DiscreteGpusAfterTheOpening {
     /// 只留 `identifiers_present_now`（挂着切换那一刻现查的那一次枚举里在的独显）里在的：挂载之后掉了的卡不再算本机的卡，切换点它照「点名的卡不存在」拒，
     /// 不点名时它不再参与（第 4 条「切换那一刻……GPU 不在……就拒绝切换」）。
     #[must_use]
     pub fn still_present(&self, identifiers_present_now: &[GpuDeviceIdentifier]) -> Self {
+        let is_present_now = |discrete_gpu: &&DiscreteGpuOnThisHost| {
+            identifiers_present_now.contains(&discrete_gpu.identifier)
+        };
         Self {
-            opened: self
-                .opened
+            opened: self.opened.iter().filter(is_present_now).cloned().collect(),
+            not_opened_for_not_taking_part: self
+                .not_opened_for_not_taking_part
                 .iter()
-                .filter(|opened| identifiers_present_now.contains(&opened.identifier))
-                .cloned()
-                .collect(),
-            not_opened: self
-                .not_opened
-                .iter()
-                .filter(|not_opened| {
-                    identifiers_present_now.contains(&not_opened.discrete_gpu.identifier)
-                })
+                .filter(is_present_now)
                 .cloned()
                 .collect(),
         }
     }
+
+    /// 切换生效那一刻开成了一张原先没开的卡：它从没开的那几张挪进开成的那几张（排在开成的那几张末尾，与外壳开着的卡同序）。
+    ///
+    /// # Panics
+    /// 它不在没开的那几张里：外壳只开没开的卡。
+    pub fn note_the_gpu_opened_at_a_switch(&mut self, identifier: &GpuDeviceIdentifier) {
+        let position = self
+            .not_opened_for_not_taking_part
+            .iter()
+            .position(|discrete_gpu| discrete_gpu.identifier == *identifier)
+            .expect("外壳只开没开的卡");
+        let opened_at_the_switch = self.not_opened_for_not_taking_part.remove(position);
+        self.opened.push(opened_at_the_switch);
+    }
 }
 
-/// 挂载那一刻开卡之后的本机独显：开成的那几张与没开成而跳过的那几张，各按总线地址排。
-/// 卡清单（核心层 `GpuInventory`）照它答：本机全部独显连同没开成的（两样合起来按总线地址排，没给 `background_gpu_devices` 时上限按这个次序
-/// 一张一个，第 3 条），与其中没开成的那几张、原因照挂着时这张卡出错的报法（`core_failure_of_the_gpu`）。
+/// 开卡之后的本机独显：开着的那几张（挂载那一刻开的按总线地址排，之后切换开的依次排在末尾，与外壳开着的卡一一对应、同序），
+/// 与没参与而没开的那几张（按总线地址排）。卡清单（核心层 `GpuInventory`）照它答：两样合起来按总线地址排（没给 `background_gpu_devices` 时
+/// 上限按这个次序一张一个，第 3 条）；没开的照列，切换可以点它。挂载那一刻没有「没开成而跳过」的卡：参与的卡没开成就拒绝挂载，不参与的卡不开。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct DiscreteGpusAfterTheOpening {
     pub opened: Vec<DiscreteGpuOnThisHost>,
-    pub not_opened: Vec<DiscreteGpuNotOpenedAtTheMount>,
+    pub not_opened_for_not_taking_part: Vec<DiscreteGpuOnThisHost>,
 }
 
 impl GpuInventory for DiscreteGpusAfterTheOpening {
@@ -322,11 +323,7 @@ impl GpuInventory for DiscreteGpusAfterTheOpening {
         let mut every_discrete_gpu: Vec<&DiscreteGpuOnThisHost> = self
             .opened
             .iter()
-            .chain(
-                self.not_opened
-                    .iter()
-                    .map(|not_opened| &not_opened.discrete_gpu),
-            )
+            .chain(self.not_opened_for_not_taking_part.iter())
             .collect();
         every_discrete_gpu.sort_by(|left, right| left.pci_bus_address.cmp(&right.pci_bus_address));
         every_discrete_gpu
@@ -334,19 +331,6 @@ impl GpuInventory for DiscreteGpusAfterTheOpening {
             .map(|discrete_gpu| (discrete_gpu.identifier.clone(), discrete_gpu.total_memory))
             .collect()
     }
-
-    fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
-        self.not_opened
-            .iter()
-            .map(|not_opened| GpuDeviceNotOpenedAtTheMount {
-                device: not_opened.discrete_gpu.identifier.clone(),
-                failure_at_the_mount: core_failure_of_the_gpu(
-                    &not_opened.discrete_gpu.identifier,
-                    &not_opened.cause,
-                ),
-            })
-            .collect()
-    }
 }
 
 /// 挂载那一刻量代价交的那一段（外壳的那一段是 `crate::background_gpu::COST_PROBE_CHUNK`）：一条 `input_bytes` 字节的单元，`footprint` 是它那一段在卡上的占用。
@@ -784,7 +768,7 @@ pub enum CostAtTheOpening {
     NotMeasuredForTheGpuIsLeftOut,
 }
 
-/// 挂载那一刻开卡的结局：开成的每张卡开出来的东西（与 `discrete_gpus.opened` 一一对应、同序）、开卡之后的本机独显、开卡那一组参与的卡
+/// 挂载那一刻开卡的结局：开成的每张卡开出来的东西（与 `discrete_gpus.opened` 一一对应、同序）、开卡之后的本机独显（没参与的没开）、开卡那一组参与的卡
 /// （在开成的那几张里的位置；显存上限装不下最小的一片的不在里面，可以一张都没有）、因显存上限不用的那几张（按参与的次序）。
 #[derive(Debug)]
 pub struct GpusOpenedAtTheMount<Opened> {
@@ -794,15 +778,47 @@ pub struct GpusOpenedAtTheMount<Opened> {
     pub left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
 }
 
-/// 挂载那一刻在本机每一张独显上开卡（按 `discrete_gpus` 的次序，即总线地址的次序）：`open_the_gpu` 是外壳起设备、编管线、量代价那一步，
-/// 交给它这张卡、开卡那一组给它的显存上限、量不量代价；测试里交造出来的结局。参与的卡显存上限装不下 `smallest_piece_bytes`（最小的一片）的照开、
-/// 不量代价，不进参与的卡、报出来（[`GpuLeftOutUnderItsMemoryLimit`]），挂载不因此拒绝（用户 2026-10-28 定「连一片都装不下就不用这张卡」）；
-/// 装得下的照它的上限量代价（外壳切成几片逐片交）。一张卡没开成挡不挡挂载（D26（后台整理与放置回收） 已定项 13 第 2、4 条，
-/// `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 45 批交回那一行的定法）：它参与（`participating` 里有它——给了 `background_gpu_devices` 时是点名的卡，
-/// 没给时本机全部独显都参与）就拒绝挂载、报出它与原因，后面的卡不再开；不参与（给了列表而没点它）就跳过，不进参与的卡，记进
-/// [`DiscreteGpusAfterTheOpening::not_opened`] 报出来，挂载照常。
+/// 开一张卡（挂载那一刻开参与的卡、切换生效那一刻开点到的没开的卡，同一步）：先照给它的显存上限（扣掉这个进程里之前几次挂载泄漏在它上面的字节）
+/// 判装不装得下最小的一片（`smallest_piece_bytes`），装得下的交 `open_the_gpu` 起设备、编管线、照上限量代价；装不下的照开、不量代价，
+/// 连同报它不用的那一条交回（用户 2026-10-28 定「连一片都装不下就不用这张卡」）。
+///
+/// # Errors
+/// `open_the_gpu` 交回的这张卡没开成的原因，原样。
+pub fn gpu_opened_under_its_memory_limit<Opened>(
+    discrete_gpu: &DiscreteGpuOnThisHost,
+    memory_limit_at_the_opening: Option<GpuMemoryInBytes>,
+    smallest_piece_bytes: u64,
+    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
+    open_the_gpu: impl FnOnce(
+        &DiscreteGpuOnThisHost,
+        Option<GpuMemoryInBytes>,
+        CostAtTheOpening,
+    ) -> Result<Opened, GpuFailureCause>,
+) -> Result<(Opened, Option<GpuLeftOutUnderItsMemoryLimit>), GpuFailureCause> {
+    let left_out = gpu_left_out_under_its_memory_limit(
+        &discrete_gpu.identifier,
+        memory_limit_at_the_opening,
+        smallest_piece_bytes,
+        memory_leaked_in_this_process.bytes_leaked_on(&discrete_gpu.identifier),
+    );
+    let cost_at_the_opening = match &left_out {
+        Some(_left_out) => CostAtTheOpening::NotMeasuredForTheGpuIsLeftOut,
+        None => CostAtTheOpening::MeasuredUnderTheMemoryLimit,
+    };
+    let opened = open_the_gpu(
+        discrete_gpu,
+        memory_limit_at_the_opening,
+        cost_at_the_opening,
+    )?;
+    Ok((opened, left_out))
+}
+
+/// 挂载那一刻只在参与的卡上开卡（欠账 C996 的定案；按 `discrete_gpus` 的次序，即总线地址的次序）：`participating` 里有它（给了 `background_gpu_devices` 时是点名的卡，
+/// 没给时本机全部独显都参与）就照开卡那一组给它的显存上限开（[`gpu_opened_under_its_memory_limit`]，`open_the_gpu` 是外壳起设备、编管线、量代价那一步，
+/// 测试里交造出来的结局）；不参与的不开、不量代价，记进 [`DiscreteGpusAfterTheOpening::not_opened_for_not_taking_part`]，卡清单照列它，挂着切换点到它时再开。
+/// 显存上限装不下最小的一片的参与的卡照开、不量代价、不进参与的卡、报出来（[`GpuLeftOutUnderItsMemoryLimit`]），挂载不因此拒绝。
+/// 一张参与的卡没开成就拒绝挂载、报出它与原因，后面的卡不再开（D26（后台整理与放置回收） 已定项 13 第 4 条）。
 /// `participating` 是开卡那一组照同一份 `discrete_gpus` 校验过、取出来的参与的卡（`adapter_inventory::participating_gpus_admitted_by_the_mount_checks`）。
-/// 这个进程里此前几次挂载泄漏在一张卡上的字节（`memory_leaked_in_this_process`）先从它的上限里扣掉再判装不装得下（第 3 条；外壳开这张卡时账也从它起）。
 ///
 /// # Errors
 /// [`BackgroundGpuOpenRefused::GpuUnusable`]：一张参与的卡没开成。
@@ -820,50 +836,36 @@ pub fn gpus_opened_at_the_mount<Opened>(
         CostAtTheOpening,
     ) -> Result<Opened, GpuFailureCause>,
 ) -> Result<GpusOpenedAtTheMount<Opened>, BackgroundGpuOpenRefused> {
-    let mut opened = Vec::with_capacity(discrete_gpus.len());
+    let mut opened = Vec::with_capacity(participating.len());
     let mut opened_discrete_gpus: Vec<DiscreteGpuOnThisHost> =
-        Vec::with_capacity(discrete_gpus.len());
-    let mut not_opened = Vec::new();
+        Vec::with_capacity(participating.len());
+    let mut not_opened_for_not_taking_part = Vec::new();
     let mut left_out_under_their_memory_limit = Vec::new();
     for discrete_gpu in discrete_gpus {
         let participating_gpu = participating.iter().find(|participating_gpu| {
             participating_gpu.discrete_gpu.identifier == discrete_gpu.identifier
         });
-        let memory_limit_at_the_opening =
-            participating_gpu.and_then(|participating_gpu| participating_gpu.memory_limit);
-        let left_out = gpu_left_out_under_its_memory_limit(
-            &discrete_gpu.identifier,
-            memory_limit_at_the_opening,
-            smallest_piece_bytes,
-            memory_leaked_in_this_process.bytes_leaked_on(&discrete_gpu.identifier),
-        );
-        let cost_at_the_opening = match &left_out {
-            Some(_left_out) => CostAtTheOpening::NotMeasuredForTheGpuIsLeftOut,
-            None => CostAtTheOpening::MeasuredUnderTheMemoryLimit,
+        let memory_limit_at_the_opening = match participating_gpu {
+            Some(participating_gpu) => participating_gpu.memory_limit,
+            None => {
+                not_opened_for_not_taking_part.push(discrete_gpu);
+                continue;
+            }
         };
-        match open_the_gpu(
+        let (opened_gpu, left_out) = gpu_opened_under_its_memory_limit(
             &discrete_gpu,
             memory_limit_at_the_opening,
-            cost_at_the_opening,
-        ) {
-            Ok(opened_gpu) => {
-                opened.push(opened_gpu);
-                opened_discrete_gpus.push(discrete_gpu);
-                left_out_under_their_memory_limit.extend(left_out);
-            }
-            Err(cause) => match participating_gpu {
-                Some(_participating) => {
-                    return Err(BackgroundGpuOpenRefused::GpuUnusable {
-                        device: discrete_gpu.identifier,
-                        cause,
-                    });
-                }
-                None => not_opened.push(DiscreteGpuNotOpenedAtTheMount {
-                    discrete_gpu,
-                    cause,
-                }),
-            },
-        }
+            smallest_piece_bytes,
+            memory_leaked_in_this_process,
+            &mut open_the_gpu,
+        )
+        .map_err(|cause| BackgroundGpuOpenRefused::GpuUnusable {
+            device: discrete_gpu.identifier.clone(),
+            cause,
+        })?;
+        opened.push(opened_gpu);
+        opened_discrete_gpus.push(discrete_gpu);
+        left_out_under_their_memory_limit.extend(left_out);
     }
     let participating_positions: Vec<usize> = participating
         .iter()
@@ -873,7 +875,7 @@ pub fn gpus_opened_at_the_mount<Opened>(
                 .position(|opened_gpu| {
                     opened_gpu.identifier == participating_gpu.discrete_gpu.identifier
                 })
-                .expect("参与的卡取自同一份独显清单；没开成的那一张在上面就拒了挂载")
+                .expect("参与的卡取自同一份独显清单，都开了；没开成的那一张在上面就拒了挂载")
         })
         .collect();
     let left_out_positions: Vec<usize> = left_out_under_their_memory_limit
@@ -897,7 +899,7 @@ pub fn gpus_opened_at_the_mount<Opened>(
         opened,
         discrete_gpus: DiscreteGpusAfterTheOpening {
             opened: opened_discrete_gpus,
-            not_opened,
+            not_opened_for_not_taking_part,
         },
         participating: ParticipatingGpusOnThisHost::of_the_positions(participating_positions)
             .leaving_out(|position| left_out_positions.contains(&position)),
@@ -949,8 +951,6 @@ mod tests {
         BackgroundComputeSettings, GpuInventory,
     };
 
-    use singlefs_core::background_compute::{BackgroundGpuComputeFailed, GpuFailureOnTheDevice};
-
     use crate::adapter_inventory::{
         discrete_gpus_of_the_enumeration, participating_gpus_admitted_by_the_mount_checks,
         AdapterDeviceKind, BackgroundGpuSelection, EnumeratedAdapterDescription, GIBIBYTE,
@@ -1007,6 +1007,7 @@ mod tests {
         let at_the_mount = ParticipatingGpusOnThisHost::of_the_settings(
             &admitted(None, None, Duration::from_secs(10)),
             &identifiers_on_this_host(),
+            &[],
         );
         assert_eq!(
             at_the_mount.positions_on_this_host(),
@@ -1020,6 +1021,7 @@ mod tests {
                 Duration::from_secs(10),
             ),
             &identifiers_on_this_host(),
+            &[],
         );
         assert_eq!(
             after_the_switch.positions_on_this_host(),
@@ -1255,47 +1257,47 @@ mod tests {
         )
     }
 
-    /// 第 2 条「点名了就只用点名的卡」：点名的两张都开成、没点名的那张没开成——照挂，没开成的那张跳过、不进参与的卡，卡清单照列它并报出它与原因。
+    /// 欠账 C996 的定案「只开参与的卡」：点名一张时，挂载那一刻 `open_the_gpu` 只被调一次、调的是点名的那张；
+    /// 没点名的两张不开、不量代价，卡清单照列它们（按总线地址排），不报没开成。
     #[test]
-    fn an_unnamed_card_that_did_not_open_is_skipped_and_reported_while_the_named_cards_take_part() {
-        let opening = opening_under_the_selection(
-            &BackgroundGpuSelection {
-                background_gpu_devices: Some(vec![card("0000:81:00.0"), card("0000:01:00.0")]),
-                background_gpu_memory_limit: Some(vec![
-                    GpuMemoryInBytes(2 * GIBIBYTE),
-                    GpuMemoryInBytes(3 * GIBIBYTE),
-                ]),
-                background_gpu_wait_limit: Duration::from_secs(10),
+    fn naming_one_card_opens_only_the_named_card_at_the_mount() {
+        let selection = BackgroundGpuSelection {
+            background_gpu_devices: Some(vec![card("0000:41:00.0")]),
+            background_gpu_memory_limit: None,
+            background_gpu_wait_limit: Duration::from_secs(10),
+        };
+        let discrete_gpus = three_discrete_gpus_of_the_enumeration();
+        let participating =
+            participating_gpus_admitted_by_the_mount_checks(&selection, &discrete_gpus)
+                .expect("点名本机的一张卡：校验合法");
+        let mut cards_handed_to_the_opening: Vec<String> = Vec::new();
+        let opening = gpus_opened_at_the_mount(
+            discrete_gpus,
+            &participating,
+            bytes_of_the_smallest_piece_every_participating_gpu_holds(),
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            |discrete_gpu, _memory_limit, _cost_at_the_opening| {
+                cards_handed_to_the_opening.push(discrete_gpu.pci_bus_address.clone());
+                Ok::<_, GpuFailureCause>(discrete_gpu.pci_bus_address.clone())
             },
-            "0000:41:00.0",
         )
-        .expect("没点名的卡开不了：不挡挂载");
+        .expect("点名的那张开得成：挂载照常");
         assert_eq!(
-            opening.opened,
-            vec![
-                (
-                    "0000:01:00.0".to_string(),
-                    Some(GpuMemoryInBytes(3 * GIBIBYTE))
-                ),
-                (
-                    "0000:81:00.0".to_string(),
-                    Some(GpuMemoryInBytes(2 * GIBIBYTE))
-                ),
-            ],
-            "开成的是点名的两张，按总线地址排，各带点名给的上限"
+            cards_handed_to_the_opening,
+            vec!["0000:41:00.0".to_string()],
+            "开卡只调一次，调的是点名的那张"
         );
+        assert_eq!(opening.opened, vec!["0000:41:00.0".to_string()]);
+        assert_eq!(opening.participating.positions_on_this_host(), &[0]);
         assert_eq!(
-            opening.participating.positions_on_this_host(),
-            &[1, 0],
-            "参与的卡是点名的两张、按点名的次序（在开成的卡里的位置）"
-        );
-        assert_eq!(
-            opening.discrete_gpus.not_opened,
-            vec![DiscreteGpuNotOpenedAtTheMount {
-                discrete_gpu: three_discrete_gpus_of_the_enumeration()[1].clone(),
-                cause: reason_the_card_did_not_open("0000:41:00.0"),
-            }],
-            "跳过的那张报出来：哪张卡、什么原因"
+            opening
+                .discrete_gpus
+                .not_opened_for_not_taking_part
+                .iter()
+                .map(|discrete_gpu| discrete_gpu.pci_bus_address.clone())
+                .collect::<Vec<_>>(),
+            vec!["0000:01:00.0".to_string(), "0000:81:00.0".to_string()],
+            "没点名的两张没开，按总线地址排"
         );
         assert_eq!(
             opening
@@ -1309,21 +1311,46 @@ mod tests {
                 card("0000:41:00.0"),
                 card("0000:81:00.0")
             ],
-            "卡清单照列本机全部独显，连同没开成的那张，按总线地址排"
+            "卡清单照列本机全部独显，连同没开的两张，切换可以点它们"
         );
+        assert!(
+            opening
+                .discrete_gpus
+                .devices_not_opened_at_the_mount()
+                .is_empty(),
+            "没开的卡不是没开成：核心层不因它拒切换"
+        );
+    }
+
+    /// 第 2 条「不给就用本机全部独立显卡」：没给 `background_gpu_devices` 时三张都参与、都开。
+    #[test]
+    fn without_named_cards_every_discrete_gpu_is_opened_at_the_mount() {
+        let opening = opening_under_the_selection(
+            &BackgroundGpuSelection {
+                background_gpu_devices: None,
+                background_gpu_memory_limit: None,
+                background_gpu_wait_limit: Duration::from_secs(10),
+            },
+            "没有开不成的卡",
+        )
+        .expect("三张都开得成");
         assert_eq!(
-            opening.discrete_gpus.devices_not_opened_at_the_mount(),
-            vec![GpuDeviceNotOpenedAtTheMount {
-                device: card("0000:41:00.0"),
-                failure_at_the_mount: BackgroundGpuComputeFailed::GpuFailed {
-                    device: card("0000:41:00.0"),
-                    cause: GpuFailureOnTheDevice::DeviceOrPipelineDidNotStart {
-                        reason: "起不了设备：0000:41:00.0 造出来的坏卡".to_string(),
-                    },
-                },
-            }],
-            "卡清单报给核心层的没开成的那张：卡与原因同挂着时这张卡出错的报法"
+            opening
+                .opened
+                .iter()
+                .map(|(pci_bus_address, _memory_limit)| pci_bus_address.clone())
+                .collect::<Vec<_>>(),
+            vec![
+                "0000:01:00.0".to_string(),
+                "0000:41:00.0".to_string(),
+                "0000:81:00.0".to_string()
+            ]
         );
+        assert!(opening
+            .discrete_gpus
+            .not_opened_for_not_taking_part
+            .is_empty());
+        assert_eq!(opening.participating.positions_on_this_host(), &[0, 1, 2]);
     }
 
     /// 第 4 条：点名的卡开不了，拒绝挂载，报出它与原因。
@@ -1436,12 +1463,8 @@ mod tests {
                     "0000:41:00.0".to_string(),
                     CostAtTheOpening::MeasuredUnderTheMemoryLimit
                 ),
-                (
-                    "0000:81:00.0".to_string(),
-                    CostAtTheOpening::MeasuredUnderTheMemoryLimit
-                ),
             ],
-            "三张卡都开、都量代价"
+            "点名的两张开、都量代价，没点名的那张不开"
         );
         assert_eq!(opening.participating.positions_on_this_host(), &[0, 1]);
         assert!(opening.left_out_under_their_memory_limit.is_empty());
````

### crates/singlefs-background-gpu/src/lib.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/lib.rs b/crates/singlefs-background-gpu/src/lib.rs
index 7888d28..bf6613d 100644
--- a/crates/singlefs-background-gpu/src/lib.rs
+++ b/crates/singlefs-background-gpu/src/lib.rs
@@ -10,7 +10,7 @@
 //!   每张怎么切、等多久、一批按卡的顺序填满（前面的卡填到它照显存上限一次放得下的量，装不下的才给后面的卡；[`gpus_of_this_host`]）。
 //! - 起 GPU 的外壳（[`background_gpu`]）：枚举 wgpu 适配器、在参与的卡上起设备、量一次提交的主机 CPU 时间、交一批校验和核对、一批区域扫描的记录或一批聚簇段。
 //! - 核心层要的那一份实现（[`background_gpu_compute`]）：`singlefs_core::background_compute::BackgroundGpuCompute` 三段各一个方法接到上面三段的两份核，
-//!   连续不确定的计数随它活；挂载那一刻开卡、交回核心层收的把手与没开成而跳过的卡（[`background_gpu_compute::open_background_gpu_compute`]）。
+//!   连续不确定的计数随它活；挂载那一刻只开参与的卡、交回核心层收的把手（[`background_gpu_compute::open_background_gpu_compute`]）。
 //!
 //! 与验证包的 GPU 判器不共用任何代码（`#[path]`、`include!` 也不用；门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格第 ⑧ 条）：
 //! 两边共用一段写错的代码时，判器与被判的产品路径会一起错，GPU 对拍照样全绿。
````

## 补丁 `/home/fy5090/.local/share/singlefs/work/c996-reopen-and-close/patch/crates.patch`

### crates/singlefs-background-gpu/src/background_gpu.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/background_gpu.rs b/crates/singlefs-background-gpu/src/background_gpu.rs
index a739cce..c974bee 100644
--- a/crates/singlefs-background-gpu/src/background_gpu.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu.rs
@@ -1,11 +1,14 @@
 //! 起 GPU 的外壳（D26（后台整理与放置回收） 已定项 13 第 1–4、8 条）：挂载那一刻枚举 wgpu 适配器、判过校验、只在参与的卡上起设备与各段两份核的管线、
 //! 量一次提交的主机 CPU 时间（欠账 C996 的定案「只开参与的卡」：给了 `background_gpu_devices` 是点名的卡，没给是本机全部独立显卡）；
-//! 参与的卡没开成就拒绝挂载（判法在 `crate::gpus_of_this_host::gpus_opened_at_the_mount`）。给了 `background_gpu_devices` 时只判点名的卡：没点名的独显缺总线地址、缺设备本地堆、与别的独显报了同一个地址都不拒，
+//! 参与的卡开不成就隔 1 分钟重开、至多重开 3 次，都不成才拒绝挂载（用户定案；判法在 `crate::gpus_of_this_host::gpus_opened_at_the_mount`）。给了 `background_gpu_devices` 时只判点名的卡：没点名的独显缺总线地址、缺设备本地堆、与别的独显报了同一个地址都不拒，
 //! 不收进卡清单、报出来（[`BackgroundGpuOverTheCards::unnamed_discrete_gpus_left_off_the_list`]，判法 `crate::adapter_inventory::discrete_gpus_of_the_enumeration_under_the_named`）。
 //! 参与的卡由设置定（[`BackgroundGpuOverTheCards::participating_gpus`] 报出来），挂着切换时核心层交来新的一组
 //! （[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]，两批之间），从下一批起只在它点的卡上、照它给的显存上限与等的上限跑：
-//! 开着的卡不重开；它点到挂载时没开的卡，在这一刻开那一张（开卡同挂载那一刻，[`OpeningOfAGpuAtASwitch`]），开不成的这张卡不参与，
-//! 之后每一批报成它的 GPU 出错，直到下一次换设置。切换之后不再参与的卡照开着，不关（分活、切段、等的纯逻辑在 `crate::gpus_of_this_host`）。
+//! 开着的卡不重开；它点到没开的卡，照用户定案「切换请求照收，在会话锁外开新卡」在另一条线程上开（开卡同挂载那一刻，[`OpeningOfAGpuAtASwitch`]；
+//! 开不成同样隔 1 分钟重开、至多重开 3 次），开着的这段时间照旧照交来之前那一组跑，开成了在之后的两批之间换上
+//! （[`BackgroundGpuOverTheCards::put_the_gpus_opened_since_the_switch_in_effect`]）；重开到上限都没开成就报出那张卡与每一次的原因，
+//! 之后每一批报成它的 GPU 出错（核心层当场停 GC、报警），不自动再试，直到下一次换设置。换上一组时不再点名的卡关掉（用户定案「切换时关掉」），
+//! 之后再点它就重开（分活、切段、等的纯逻辑在 `crate::gpus_of_this_host`）。
 //! 挂着的时候每一批按卡的顺序填满参与的卡（前面的卡填到它照显存上限一次放得下的量，装不下的才给后面的卡；挂载时量的代价不用来分活）、按显存上限切段、两份核各算一次、逐条比对。三段都接上了：扫描里的校验和核对（[`BackgroundGpuOverTheCards::verify_unit_checksums`]）、
 //! 区域扫描里的单元头解析与映射反查（[`BackgroundGpuOverTheCards::classify_region_records`]）与选段打分的每段活槽数（[`BackgroundGpuOverTheCards::count_live_slots_per_cluster_segment`]）。
 //! 本机的卡清单（核心层 `GpuInventory`）答的是此刻本机在的独显与各自的显存总量（按总线地址排，开着的与没参与而没开的都列；每次答之前现枚举一次，
@@ -38,9 +41,10 @@
 
 use std::future::Future;
 use std::ops::Range;
-use std::sync::mpsc::RecvTimeoutError;
+use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, TryRecvError};
 use std::sync::{Arc, Mutex};
 use std::task::{Context, Poll, Wake, Waker};
+use std::thread::JoinHandle;
 use std::time::Duration;
 
 use singlefs_core::allocator::AllocationRecord;
@@ -113,10 +117,11 @@ use crate::two_kernel_reconciliation::{
     ConsecutiveUncertaintyCounts, ReconciledBatch, SameEntryKeyTwiceInOneBatch,
 };
 use crate::waiting_for_the_gpu::{
-    outcome_of_opening_a_gpu_within_the_limit, outcome_resubmitted_after_timeouts,
-    outcome_waited_for_again_after_timeouts, wait_for_completion_before_the_deadline,
-    CompletionCheck, GpuWaitDeadline, MonotonicClock, SteadyClockOfThisProcess,
-    TimeoutsOfTheBatchOnOneGpu, WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
+    gpu_opened_with_the_reopenings, outcome_of_opening_a_gpu_within_the_limit,
+    outcome_resubmitted_after_timeouts, outcome_waited_for_again_after_timeouts,
+    wait_for_completion_before_the_deadline, CompletionCheck, GpuWaitDeadline, MonotonicClock,
+    ReopeningsOfAGpuNotOpened, SteadyClockOfThisProcess, TimeoutsOfTheBatchOnOneGpu,
+    WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
 };
 
 /// 各段的着色器按字节偏移用 `u32` 寻址、整字那一段按「剩下的字节 ≥ 4」走：一个绑定不超过 2^31 字节，偏移加 4 也不回绕。
@@ -167,6 +172,11 @@ pub enum GpuFailureCause {
         resubmissions: u32,
         last_cause: Box<GpuFailureCause>,
     },
+    /// 挂着切换点到的这张卡开不成，隔 1 分钟重开 3 次也都没开成（用户定案；`waiting_for_the_gpu::gpu_opened_with_the_reopenings`）：
+    /// `causes_of_each_opening` 是每一次开不成的原因，按开的次序（头一次的在前，1 + 重开次数 个）。交核心层当报警，不自动再试。
+    NotOpenedAfterTheReopenings {
+        causes_of_each_opening: Vec<GpuFailureCause>,
+    },
 }
 
 /// 挂载那一刻开不了后台 GPU（第 4 条：拒绝挂载并报错，不降级、不退回 CPU）。成员按调用方要改哪一样分。
@@ -184,12 +194,14 @@ pub enum BackgroundGpuOpenRefused {
     /// 不拒，走不到这里）；期限从枚举做完才起算，上限小到枚举刚做完头一次看就到期才走到这里。上限给 0 不走这里，在枚举之前就拒成
     /// [`Self::SettingsRefused`] 的 `GpuWaitLimitOfZero`。
     AdapterEnumerationFailed { cause: GpuFailureCause },
-    /// 一张参与的卡用不上（起不了设备、编不了管线、量代价那几次提交出错或超时）：给了 `background_gpu_devices` 时是点名的卡，没给时本机任何一张独显。
+    /// 一张参与的卡用不上（起不了设备、编不了管线、量代价那几次提交出错或超时），隔 1 分钟重开 3 次也都没开成
+    /// （`waiting_for_the_gpu::gpu_opened_with_the_reopenings`）：给了 `background_gpu_devices` 时是点名的卡，没给时本机任何一张独显。
+    /// `causes_of_each_opening` 是每一次开不成的原因，按开的次序（头一次的在前，1 + 重开次数 个）。
     /// 没点名的卡不开，不走这里。显存上限连最小的一片都装不下的卡不拒，
     /// 不用它、报出来（[`BackgroundGpu::gpus_left_out_under_their_memory_limit`]）。
     GpuUnusable {
         device: GpuDeviceIdentifier,
-        cause: GpuFailureCause,
+        causes_of_each_opening: Vec<GpuFailureCause>,
     },
 }
 
@@ -316,14 +328,11 @@ impl From<SameEntryKeyTwiceInOneBatch> for LiveSlotBatchFailed {
     }
 }
 
-/// 此刻这组设置下一批分不到卡上（[`BackgroundGpu::batch_not_split_over_the_gpus`]）：换设置时开一张挂载时没开的卡没开成、或量一张卡的代价出错了
-/// （这张卡没参与，报成它的 GPU 出错），或参与的卡因显存上限一张都不剩。这组设置下每一批三段都交回它，直到下一次换设置。封闭集合。
+/// 此刻这组设置下一批分不到卡上（[`BackgroundGpu::batch_not_split_over_the_gpus`]）：最近交来的那一组点到的一张没开的卡重开到上限都没开成、
+/// 或换设置时量一张卡的代价出错了（这张卡没参与，报成它的 GPU 出错），或参与的卡因显存上限一张都不剩。这组设置下每一批三段都交回它，直到下一次换设置。封闭集合。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum BatchNotSplitOverTheGpus {
-    GpuNotOpenedAtTheSwitch {
-        device: GpuDeviceIdentifier,
-        cause: GpuFailureCause,
-    },
+    GpuNotOpenedAfterTheReopeningsAtTheSwitch(GpuNotOpenedAfterTheReopenings),
     CostNotMeasuredAtTheSwitch {
         device: GpuDeviceIdentifier,
         cause: GpuFailureCause,
@@ -332,6 +341,14 @@ pub enum BatchNotSplitOverTheGpus {
     EveryParticipatingGpuLeftOut(Vec<GpuLeftOutUnderItsMemoryLimit>),
 }
 
+/// 挂着切换点到的一张没开的卡，隔一段时间重开到上限也都没开成（用户定案「3 次都不成：GC 停下，报警并报出哪张卡、每次开不成的原因」）：
+/// 哪张卡、每一次开不成的原因（按开的次序，头一次的在前，1 + 重开次数 个）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct GpuNotOpenedAfterTheReopenings {
+    pub device: GpuDeviceIdentifier,
+    pub causes_of_each_opening: Vec<GpuFailureCause>,
+}
+
 /// 一张参与的卡对外报的样子（第 2 条：参与的卡都用，并报出来）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct ParticipatingGpuReport {
@@ -346,7 +363,7 @@ pub struct ParticipatingGpuReport {
 /// 一段交成了就照那一段的账放掉；等超过上限的那一段的账留着、记成还在跑的那次提交（`GpuMemoryAccount::settle_the_chunk_after_its_submission`）；
 /// 建缓冲显存不足的那一段不交：已建的那几份连同暂存冲掉、做完了才放账、交回 [`GpuFailureCause::OutOfMemoryCreatingTheBuffers`]，
 /// 没等到做完的账留着、记成还在跑的那次提交，交回等超过上限。
-pub trait CardOfTheBackgroundGpu {
+pub trait CardOfTheBackgroundGpu: Send + 'static {
     /// 一次提交的把手（等超过上限之后还在卡上跑的那一次记在账上）。
     type Submission;
 
@@ -411,8 +428,9 @@ pub trait CardOfTheBackgroundGpu {
     ) -> Result<Vec<(u64, u64)>, GpuFailureCause>;
 }
 
-/// 切换生效那一刻开一张挂载时没开的卡（同挂载那一刻开卡那一步）：交给它这张卡、这一组给它的显存上限、等 GPU 的上限、量不量代价，
-/// 交回开着的卡或没开成的原因。产品是在这张卡的 wgpu 适配器上起设备、编管线、量代价（[`opening_of_a_gpu_in_use`]），测试里交造出来的卡。
+/// 挂着切换点到一张没开的卡时开它一次（同挂载那一刻开卡那一步）：交给它这张卡、这一组给它的显存上限、等 GPU 的上限、量不量代价，
+/// 交回开着的卡或这一次没开成的原因（开不成重开归调用方，`waiting_for_the_gpu::gpu_opened_with_the_reopenings`）。
+/// 产品是在这张卡的 wgpu 适配器上起设备、编管线、量代价（[`opening_of_a_gpu_in_use`]），测试里交造出来的卡。在会话锁外那条开卡线程上调。
 pub type OpeningOfAGpuAtASwitch<Card> = Box<
     dyn FnMut(
             &DiscreteGpuOnThisHost,
@@ -424,7 +442,7 @@ pub type OpeningOfAGpuAtASwitch<Card> = Box<
 >;
 
 /// 这一次挂载的后台 GPU 在几张开着的卡上（`Card` 是一张开着的卡，产品是 [`GpuInUse`]）：开着的本机独显（挂载那一刻开的参与的卡按总线地址排，
-/// 切换那一刻开的依次排在后面）与没参与而没开的那几张，给了点名时没点名而交不成一张卡、没收进清单的那几张，与此刻参与的那几张（设置定的，按参与的次序：
+/// 切换开的依次排在后面；切换之后不再点名的关掉）与没开的那几张，给了点名时没点名而交不成一张卡、没收进清单的那几张，与此刻参与的那几张（设置定的，按参与的次序：
 /// 没点名是总线地址的次序，点了名是点名的次序；显存上限连最小的一片都装不下的不在里面）。生命周期一次挂载，只住内存。丢它时不等还在 GPU 上跑的提交（`Drop`）。
 pub struct BackgroundGpuOverTheCards<Card: CardOfTheBackgroundGpu> {
     /// 开着的那几张，与 `discrete_gpus_after_the_opening.opened` 一一对应、同序。
@@ -434,12 +452,18 @@ pub struct BackgroundGpuOverTheCards<Card: CardOfTheBackgroundGpu> {
     participating: ParticipatingGpusOnThisHost,
     /// 此刻这组设置点了而因显存上限装不下最小的一片不用的卡，按参与的次序。
     left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
-    /// 最近一次换设置时开一张它点到的、挂载时没开的卡没开成（这张卡没参与、照旧没开）：之后每一批都报成它的 GPU 出错，直到下一次换设置。
-    not_opened_at_the_last_switch: Option<(GpuDeviceIdentifier, GpuFailureCause)>,
+    /// 最近交来的那一组点到的一张没开的卡重开到上限都没开成：之后每一批都报成它的 GPU 出错，直到下一次换设置。
+    not_opened_after_the_reopenings_at_the_last_switch: Option<GpuNotOpenedAfterTheReopenings>,
     /// 最近一次换设置时量一张新参与的卡的代价出错（这张卡没参与）：之后每一批都报成它的 GPU 出错，直到下一次换设置。
     cost_not_measured_at_the_last_switch: Option<(GpuDeviceIdentifier, GpuFailureCause)>,
-    /// 切换点到挂载时没开的卡时开它的那一步。
-    opening_of_a_gpu_at_a_switch: OpeningOfAGpuAtASwitch<Card>,
+    /// 切换点到没开的卡时开它一次的那一步：开卡线程开着时那条线程锁着它，开完了放开。
+    opening_of_a_gpu_at_a_switch: Arc<Mutex<OpeningOfAGpuAtASwitch<Card>>>,
+    /// 开不成时怎么重开（挂载那一刻与切换同一种；产品是 [`ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT`]）。
+    reopenings_of_a_gpu_not_opened: ReopeningsOfAGpuNotOpened,
+    /// 交来了、点到没开的卡、还没换上的那一组（最近交来的那一组；开卡开完了才换上）。没有是 `None`。
+    settings_waiting_for_the_opening: Option<AdmittedGpuSettings>,
+    /// 会话锁外为切换开卡的那一次（至多一次）；没在开是 `None`。
+    opening_at_a_switch: Option<OpeningAtASwitch<Card>>,
     /// 上一次交回以来三段各自在卡上是怎么交的（[`Self::take_the_submissions_of_the_batch`]）。
     submissions_since_the_last_take: GpuSubmissionsOfTheBatch,
     gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped,
@@ -452,6 +476,36 @@ pub struct BackgroundGpuOverTheCards<Card: CardOfTheBackgroundGpu> {
 /// 产品那一份：开着的卡是 wgpu 设备与六条管线。
 pub type BackgroundGpu = BackgroundGpuOverTheCards<GpuInUse>;
 
+/// 会话锁外为切换开卡的那一次：为哪一组开（交来那一刻点到的没开的卡），与它的结局从哪来。封闭集合。
+enum OpeningAtASwitch<Card> {
+    /// 开卡线程起来了、在开：开完了经 `outcome_receiver` 送回（至多一次）。
+    UnderWay {
+        settings: AdmittedGpuSettings,
+        outcome_receiver: Receiver<OutcomeOfTheOpeningAtASwitch<Card>>,
+        opening_thread: JoinHandle<()>,
+    },
+    /// 起不了开卡线程：头一张要开的卡按开不成报（原因是起不了线程的那个错），下一次两批之间交回。
+    ThreadNotStarted {
+        settings: AdmittedGpuSettings,
+        not_opened: GpuNotOpenedAfterTheReopenings,
+    },
+}
+
+/// 开卡线程开完了送回的：开成的卡（按开的次序），与重开到上限都没开成的那一张（有一张开不成，后面的就不再开）。
+struct OutcomeOfTheOpeningAtASwitch<Card> {
+    opened: Vec<Card>,
+    not_opened: Option<GpuNotOpenedAfterTheReopenings>,
+}
+
+/// 两批之间取切换开卡的结局时等不等它开完（[`BackgroundGpuOverTheCards::put_the_gpus_opened_since_the_switch_in_effect`]）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum WaitForTheOpeningAtASwitch {
+    /// 产品：不等，还在开就照旧照交设置之前那一组跑（核心层每一批开头在会话锁里问，不能在锁里等开卡）。
+    NotAtAll,
+    /// 等到开完：测试里不靠钟就走到「开完了换上」那一格。
+    UntilItEnds,
+}
+
 /// 一张开着的 wgpu 卡（产品的 [`CardOfTheBackgroundGpu`]）。
 pub struct GpuInUse {
     discrete_gpu: DiscreteGpuOnThisHost,
@@ -894,12 +948,13 @@ pub fn open_background_gpu(
 /// 给了 `background_gpu_devices` 时只判点名的卡（没点名的独显交不成一张卡就不收、报出来）。
 /// 每张卡开卡（起设备、编各段两份核的管线、量代价）至多等 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]（五分钟），到点当那张卡没开成
 /// （`waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit`，报等超过上限、带这个上限），挂载等开卡成功或到点才往下走。
-/// 枚举完、判过卡清单与校验，逐张开卡之前把要开几张卡（参与的卡数）交给 `report_the_gpus_to_open`（进程开卡器照它把外层的期限延到枚举期限加卡数乘单卡上限）；
-/// 枚举不成、卡清单或校验拒了的不报。
+/// 一张参与的卡开不成就隔 1 分钟重开、至多重开 3 次（[`ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT`]，这条线程上睡着等），都不成才拒绝挂载。
+/// 枚举完、判过卡清单与校验，逐张开卡之前把要开几张卡（参与的卡数）交给 `report_the_gpus_to_open`（进程开卡器照它把外层的期限延到枚举期限加
+/// 卡数乘一张卡连同重开的最长等待）；枚举不成、卡清单或校验拒了的不报。
 /// 量代价出错时账上记着还在跑的提交的那张卡泄漏、不析构，这一次占着的字节当场记进 `memory_leaked_in_this_process`
 /// （`failure_of_the_measurement_dropping_or_leaking_the_card`）。
-/// 没参与的卡不开、不量代价（欠账 C996 的定案「只开参与的卡」）：卡清单照列它，之后的切换点到它时在切换生效那一刻开（[`OpeningOfAGpuAtASwitch`]，
-/// 同一份适配器与泄漏账）。参与的卡没开成就拒绝挂载（[`gpus_opened_at_the_mount`]）。参与的卡照开卡那一组给它的显存上限
+/// 没参与的卡不开、不量代价（欠账 C996 的定案「只开参与的卡」）：卡清单照列它，之后的切换点到它时在会话锁外开（[`OpeningOfAGpuAtASwitch`]，
+/// 同一份适配器与泄漏账）。参与的卡重开到上限都没开成就拒绝挂载（[`gpus_opened_at_the_mount`]）。参与的卡照开卡那一组给它的显存上限
 /// （先扣掉这个进程里之前几次挂载泄漏在它上面的字节，`memory_leaked_in_this_process`）把量代价那一段（[`COST_PROBE_CHUNK`]）切成几片逐片量、代价取各片之和；
 /// 显存上限连最小的一片都装不下的卡照开、不量代价、不参与、报出来（[`BackgroundGpuOverTheCards::gpus_left_out_under_their_memory_limit`]），挂载不因此拒绝；
 /// 参与的卡因此一张都不剩时照样交回，三段每一批都报「参与的卡一张都不剩」（[`BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut`]），
@@ -938,6 +993,8 @@ pub fn open_background_gpu_with_the_memory_leaked_in_this_process(
         &participating,
         bytes_of_the_smallest_piece_every_participating_gpu_holds(),
         memory_leaked_in_this_process,
+        ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT,
+        std::thread::sleep,
         |discrete_gpu, memory_limit_at_the_opening, cost_at_the_opening| {
             open_a_gpu(
                 discrete_gpu,
@@ -959,6 +1016,7 @@ pub fn open_background_gpu_with_the_memory_leaked_in_this_process(
             wait_limit: selection.background_gpu_wait_limit,
         }),
         Box::new(open_a_gpu),
+        ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT,
     ))
 }
 
@@ -1043,6 +1101,58 @@ where
     }
 }
 
+/// 切换开卡那条线程上做的事：按给的次序逐张开 `discrete_gpus_to_open`（各带这一组给它的显存上限），开一张同挂载那一刻开卡那一步
+/// （[`gpu_opened_under_its_memory_limit`]，开一次交 `opening_of_a_gpu_at_a_switch`、等 GPU 的上限是这一组的 `wait_limit`），开不成就照 `reopenings`
+/// 重开、每次重开之前在这条线程上睡一个间隔（[`gpu_opened_with_the_reopenings`]）。有一张重开到上限都没开成就停在它，后面的不再开。
+/// 装不下最小的一片的照开、不量代价（换上那一组时照这张卡的账与开着的卡同一判，再判一次）。
+fn gpus_opened_at_a_switch<Card>(
+    opening_of_a_gpu_at_a_switch: &mut OpeningOfAGpuAtASwitch<Card>,
+    discrete_gpus_to_open: Vec<(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)>,
+    wait_limit: Duration,
+    reopenings: ReopeningsOfAGpuNotOpened,
+    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
+) -> OutcomeOfTheOpeningAtASwitch<Card> {
+    let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
+    let mut opened = Vec::with_capacity(discrete_gpus_to_open.len());
+    for (discrete_gpu, memory_limit) in discrete_gpus_to_open {
+        let opened_with_the_reopenings =
+            gpu_opened_with_the_reopenings(reopenings, std::thread::sleep, || {
+                gpu_opened_under_its_memory_limit(
+                    &discrete_gpu,
+                    memory_limit,
+                    smallest_piece_bytes,
+                    memory_leaked_in_this_process,
+                    |gpu_to_open, memory_limit_at_the_opening, cost_at_the_opening| {
+                        opening_of_a_gpu_at_a_switch(
+                            gpu_to_open,
+                            memory_limit_at_the_opening,
+                            wait_limit,
+                            cost_at_the_opening,
+                        )
+                    },
+                )
+            });
+        match opened_with_the_reopenings {
+            Ok((card, _left_out_judged_again_when_the_settings_are_put_in_effect)) => {
+                opened.push(card);
+            }
+            Err(causes_of_each_opening) => {
+                return OutcomeOfTheOpeningAtASwitch {
+                    opened,
+                    not_opened: Some(GpuNotOpenedAfterTheReopenings {
+                        device: discrete_gpu.identifier,
+                        causes_of_each_opening,
+                    }),
+                };
+            }
+        }
+    }
+    OutcomeOfTheOpeningAtASwitch {
+        opened,
+        not_opened: None,
+    }
+}
+
 /// 起一张卡（`start_the_card`）、照开卡那一组给它的界量一次提交的代价（`cost_at_the_opening`）。量代价出错时这张卡照
 /// [`failure_of_the_measurement_dropping_or_leaking_the_card`] 处置：有提交还在跑就泄漏、这一次占着的字节记进 `memory_leaked_in_this_process`。
 ///
@@ -1109,7 +1219,6 @@ fn discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
                 ..
             }
             | BackgroundComputeSettingsRefused::GpuWaitLimitOfZero
-            | BackgroundComputeSettingsRefused::ParticipatingGpuDeviceNotOpenedAtTheMount(_)
             | BackgroundComputeSettingsRefused::GcCpuCompensationThresholdAboveTheWholeCapacity {
                 ..
             }),
@@ -2001,11 +2110,16 @@ impl CardOfTheBackgroundGpu for GpuInUse {
 impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
     /// 挂载那一刻开卡之后：开成的卡（与 `discrete_gpus_after_the_opening.opened` 一一对应、同序）、开卡之后的本机独显（连同没参与而没开的）、没点名而没收进清单的那几张、
     /// 开卡那一组参与的卡与因显存上限不用的卡（`gpus_of_this_host::gpus_opened_at_the_mount` 交回的），进程里泄漏的那一份（丢它时记进去），
-    /// 卡清单每次答之前现查本机在的独显的那一样（产品带着开卡用的那一份进程级 Vulkan 实例），与切换点到没开的卡时开它的那一步。测试拿它把替身卡装进来。
+    /// 卡清单每次答之前现查本机在的独显的那一样（产品带着开卡用的那一份进程级 Vulkan 实例），切换点到没开的卡时开它一次的那一步，
+    /// 与开不成时怎么重开（产品是 [`ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT`]）。测试拿它把替身卡装进来。
     ///
     /// # Panics
     /// 开成的卡与 `discrete_gpus_after_the_opening.opened` 张数不等。
     #[must_use]
+    #[allow(
+        clippy::too_many_arguments,
+        reason = "挂载那一刻开卡之后的各样各是一个参数，测试照它逐样装替身；捆成一个结构体只是把同样几个字段挪个地方"
+    )]
     pub fn of_the_cards_opened_at_the_mount(
         cards_on_this_host: Vec<Card>,
         discrete_gpus_after_the_opening: DiscreteGpusAfterTheOpening,
@@ -2015,6 +2129,7 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
         discrete_gpus_present_now: Box<dyn DiscreteGpusPresentNow + Send>,
         opening_of_a_gpu_at_a_switch: OpeningOfAGpuAtASwitch<Card>,
+        reopenings_of_a_gpu_not_opened: ReopeningsOfAGpuNotOpened,
     ) -> Self {
         assert_eq!(
             cards_on_this_host.len(),
@@ -2027,9 +2142,12 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
             unnamed_discrete_gpus_left_off_the_list,
             participating,
             left_out_under_their_memory_limit,
-            not_opened_at_the_last_switch: None,
+            not_opened_after_the_reopenings_at_the_last_switch: None,
             cost_not_measured_at_the_last_switch: None,
-            opening_of_a_gpu_at_a_switch,
+            opening_of_a_gpu_at_a_switch: Arc::new(Mutex::new(opening_of_a_gpu_at_a_switch)),
+            reopenings_of_a_gpu_not_opened,
+            settings_waiting_for_the_opening: None,
+            opening_at_a_switch: None,
             submissions_since_the_last_take: GpuSubmissionsOfTheBatch::default(),
             gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped::not_yet_dropped(),
             memory_leaked_in_this_process,
@@ -2037,7 +2155,7 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         }
     }
 
-    /// 此刻开着的卡（挂载那一刻开的参与的卡按总线地址排，切换那一刻开的依次排在后面），参与与否都列。
+    /// 此刻开着的卡（挂载那一刻开的参与的卡按总线地址排，为切换开的依次排在后面；换上一组时不再点名的已关掉），参与与否都列。
     #[must_use]
     pub fn gpus_opened(&self) -> Vec<GpuDeviceIdentifier> {
         self.cards_on_this_host
@@ -2094,25 +2212,240 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
     }
 
     /// 从下一批起照核心层交来的这一组跑（D26 已定项 13 第 1–3 条；核心层只在两批之间交）：参与的卡换成它点的那几张、按它的次序，
-    /// 每张卡的显存上限与等的上限换成它给的（不参与的卡不设显存上限，它分不到活）。开着的卡不重开，各卡的账照旧记着；不再参与的照开着。
-    /// 它点到挂载时没开的卡（欠账 C996 的定案「切换到新卡时再开」），在这一刻按点的次序逐张开（同挂载那一刻开卡那一步，照这一组给它的显存上限与等的上限；
-    /// 装不下最小的一片的照开、不量代价）；开不成的这张卡不参与、照旧没开，之后每一批报成它的 GPU 出错，直到下一次换设置（下一次点它再开）。
+    /// 每张卡的显存上限与等的上限换成它给的。开着的卡不重开，各卡的账照旧记着。
+    /// 它点的卡都开着，就当场换上（[`Self::put_the_settings_in_effect`]：不再点名的卡关掉）。它点到没开的卡（挂载时没开的，或之前切换关掉的），
+    /// 照用户定案「切换请求照收，在会话锁外开新卡」：这一刻不开卡，在另一条线程上按点的次序逐张开（开不成就隔一段时间重开，
+    /// [`Self::of_the_cards_opened_at_the_mount`] 给的重开规矩），开着的这段时间照旧照交来之前那一组跑；开完了在之后的两批之间换上
+    /// （[`Self::put_the_gpus_opened_since_the_switch_in_effect`]）。上一次切换的卡还在开时又交来一组，记下最近这一组，等那一次开完再看。
+    /// 上一次切换留下的「重开到上限都没开成」在这一刻清掉（下一次换设置解除它）。
+    pub fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
+        self.not_opened_after_the_reopenings_at_the_last_switch = None;
+        let discrete_gpus_to_open = self.discrete_gpus_named_and_not_opened(settings);
+        if discrete_gpus_to_open.is_empty() {
+            self.settings_waiting_for_the_opening = None;
+            self.put_the_settings_in_effect(settings);
+            return;
+        }
+        self.settings_waiting_for_the_opening = Some(settings.clone());
+        if self.opening_at_a_switch.is_none() {
+            self.start_the_opening_at_a_switch(settings.clone(), discrete_gpus_to_open);
+        }
+    }
+
+    /// 两批之间（核心层每一批开头与整理两个入口判停没停之前，经 `BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`）：
+    /// 会话锁外为切换开卡的那一次开完了就在这一刻收下它的结局，开成的卡接到开着的卡末尾，再照最近交来、在等的那一组办——
+    /// 开卡为的就是它、有一张重开到上限都没开成：这一次开成的卡关掉、照旧没参与，记下那一张，之后每一批报成它的 GPU 出错，交回它（只交这一次）；
+    /// 它点的卡都开着了：换上它（不再点名的卡关掉）；它还点着没开的卡（开卡开的是更早的一组）：为它再开一次。没有在等的那一组（之后交来的一组不用开卡、已换上）：
+    /// 这一次开成的卡都关掉。还在开（`wait` 是 [`WaitForTheOpeningAtASwitch::NotAtAll`] 时不等它）、没在开，都交回 `None`、什么都不动。
+    ///
+    /// # Panics
+    /// 开卡线程 panic 了：照原样在这条线程上再 panic（开一张卡的那一步 panic 时同样再 panic，`waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit`）。
+    pub fn put_the_gpus_opened_since_the_switch_in_effect(
+        &mut self,
+        wait: WaitForTheOpeningAtASwitch,
+    ) -> Option<GpuNotOpenedAfterTheReopenings> {
+        let (settings_of_the_opening, outcome) = match self.opening_at_a_switch.take()? {
+            OpeningAtASwitch::ThreadNotStarted {
+                settings,
+                not_opened,
+            } => (
+                settings,
+                OutcomeOfTheOpeningAtASwitch {
+                    opened: Vec::new(),
+                    not_opened: Some(not_opened),
+                },
+            ),
+            OpeningAtASwitch::UnderWay {
+                settings,
+                outcome_receiver,
+                opening_thread,
+            } => {
+                let received = match wait {
+                    WaitForTheOpeningAtASwitch::NotAtAll => outcome_receiver.try_recv(),
+                    WaitForTheOpeningAtASwitch::UntilItEnds => outcome_receiver
+                        .recv()
+                        .map_err(|_disconnected| TryRecvError::Disconnected),
+                };
+                match received {
+                    Ok(outcome) => (settings, outcome),
+                    Err(TryRecvError::Empty) => {
+                        self.opening_at_a_switch = Some(OpeningAtASwitch::UnderWay {
+                            settings,
+                            outcome_receiver,
+                            opening_thread,
+                        });
+                        return None;
+                    }
+                    Err(TryRecvError::Disconnected) => {
+                        std::panic::resume_unwind(opening_thread.join().expect_err(
+                            "开卡线程没送结局就断开只有一种：它 panic 了（正常结束一定送了，送的时候这一头还拿着收的那一端）",
+                        ))
+                    }
+                }
+            }
+        };
+        let devices_opened_by_this_opening: Vec<GpuDeviceIdentifier> = outcome
+            .opened
+            .iter()
+            .map(|card| card.discrete_gpu().identifier.clone())
+            .collect();
+        for card in outcome.opened {
+            self.discrete_gpus_after_the_opening
+                .note_the_gpu_opened_at_a_switch(&card.discrete_gpu().identifier);
+            self.cards_on_this_host.push(card);
+        }
+        let Some(settings_waiting) = self.settings_waiting_for_the_opening.take() else {
+            self.close_the_cards(|device| devices_opened_by_this_opening.contains(device));
+            return None;
+        };
+        if let Some(not_opened) = outcome.not_opened {
+            if settings_of_the_opening == settings_waiting {
+                self.close_the_cards(|device| devices_opened_by_this_opening.contains(device));
+                self.not_opened_after_the_reopenings_at_the_last_switch = Some(not_opened.clone());
+                return Some(not_opened);
+            }
+        }
+        let discrete_gpus_to_open = self.discrete_gpus_named_and_not_opened(&settings_waiting);
+        if discrete_gpus_to_open.is_empty() {
+            self.put_the_settings_in_effect(&settings_waiting);
+        } else {
+            self.settings_waiting_for_the_opening = Some(settings_waiting.clone());
+            self.start_the_opening_at_a_switch(settings_waiting, discrete_gpus_to_open);
+        }
+        None
+    }
+
+    /// 这一组点到、此刻没开的卡（按点的次序），各带这一组给它的显存上限。
+    fn discrete_gpus_named_and_not_opened(
+        &self,
+        settings: &AdmittedGpuSettings,
+    ) -> Vec<(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)> {
+        settings
+            .participating_devices()
+            .iter()
+            .filter_map(|participating| {
+                self.discrete_gpus_after_the_opening
+                    .not_opened_for_not_taking_part
+                    .iter()
+                    .find(|discrete_gpu| discrete_gpu.identifier == participating.device)
+                    .map(|discrete_gpu| (discrete_gpu.clone(), participating.memory_limit))
+            })
+            .collect()
+    }
+
+    /// 在另一条线程上为 `settings` 开 `discrete_gpus_to_open` 那几张（按给的次序逐张开，开一张同挂载那一刻开卡那一步，[`gpu_opened_under_its_memory_limit`]；
+    /// 开不成就照重开的规矩重开，每次重开之前在那条线程上睡一个间隔；有一张重开到上限都没开成，后面的就不再开）。不等它：调用即返回。
+    /// 起不了那条线程就记成头一张开不成（原因是起不了线程），下一次两批之间交回。
+    fn start_the_opening_at_a_switch(
+        &mut self,
+        settings: AdmittedGpuSettings,
+        discrete_gpus_to_open: Vec<(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)>,
+    ) {
+        let first_device_to_open = discrete_gpus_to_open
+            .first()
+            .map(|(discrete_gpu, _memory_limit)| discrete_gpu.identifier.clone())
+            .expect("只为点到没开的卡的那一组开卡：至少一张");
+        let opening_of_a_gpu_at_a_switch = Arc::clone(&self.opening_of_a_gpu_at_a_switch);
+        let memory_leaked_in_this_process = self.memory_leaked_in_this_process.clone();
+        let reopenings = self.reopenings_of_a_gpu_not_opened;
+        let wait_limit = settings.wait_limit();
+        let (outcome_sender, outcome_receiver) = channel();
+        let started = std::thread::Builder::new()
+            .name("singlefs-opening-gpus-at-a-switch".to_string())
+            .spawn(move || {
+                let mut opening_of_a_gpu = opening_of_a_gpu_at_a_switch.lock().expect(
+                    "开卡那一步只在开卡线程上锁着用：上一次开卡线程 panic 时那一次已经往上报过",
+                );
+                let outcome = gpus_opened_at_a_switch(
+                    &mut *opening_of_a_gpu,
+                    discrete_gpus_to_open,
+                    wait_limit,
+                    reopenings,
+                    &memory_leaked_in_this_process,
+                );
+                drop(opening_of_a_gpu);
+                // 送不出去只有一种：后台 GPU 已经丢了（卸载），开出来的卡没人要，在这里照常丢。
+                let _nobody_takes_this_opening_any_more = outcome_sender.send(outcome);
+            });
+        self.opening_at_a_switch = Some(match started {
+            Ok(opening_thread) => OpeningAtASwitch::UnderWay {
+                settings,
+                outcome_receiver,
+                opening_thread,
+            },
+            Err(error) => OpeningAtASwitch::ThreadNotStarted {
+                settings,
+                not_opened: GpuNotOpenedAfterTheReopenings {
+                    device: first_device_to_open,
+                    causes_of_each_opening: vec![GpuFailureCause::DeviceOrPipelineDidNotStart {
+                        reason: format!("起不了切换开卡的那条线程：{error}"),
+                    }],
+                },
+            },
+        });
+    }
+
+    /// 关掉 `is_closed` 判成要关的开着的卡（用户定案「切换之后不再参与的卡关不关」=「切换时关掉」）：它从开着的那几张挪回没开的那几张（卡清单照列，
+    /// 之后再点它就重开），设备丢掉；上面还记着一次等超过上限、没在 GPU 上做完的提交的那张不丢、整张泄漏，这一刻占着的字节记进这个进程的泄漏账
+    /// （同卸载，`gpus_of_this_host::gpus_dropped_leaking_those_still_running`）。参与的卡照它们的标识在剩下的卡里重找位置，次序不变。
+    fn close_the_cards(&mut self, is_closed: impl Fn(&GpuDeviceIdentifier) -> bool) {
+        let participating_devices: Vec<GpuDeviceIdentifier> = self
+            .participating
+            .positions_on_this_host()
+            .iter()
+            .map(|position| {
+                self.cards_on_this_host[*position]
+                    .discrete_gpu()
+                    .identifier
+                    .clone()
+            })
+            .collect();
+        let mut cards_kept_open = Vec::new();
+        let mut cards_closed = Vec::new();
+        for card in std::mem::take(&mut self.cards_on_this_host) {
+            if is_closed(&card.discrete_gpu().identifier) {
+                self.discrete_gpus_after_the_opening
+                    .note_the_gpu_closed_at_a_switch(&card.discrete_gpu().identifier);
+                cards_closed.push(card);
+            } else {
+                cards_kept_open.push(card);
+            }
+        }
+        self.cards_on_this_host = cards_kept_open;
+        let gpus_leaked = gpus_dropped_leaking_those_still_running(
+            cards_closed,
+            card_has_a_submission_running_past_its_wait,
+            card_leaked_when_dropped,
+        );
+        self.memory_leaked_in_this_process
+            .note_the_gpus_leaked_when_dropped(&gpus_leaked);
+        self.participating = ParticipatingGpusOnThisHost::of_the_positions(
+            participating_devices
+                .iter()
+                .filter_map(|device| {
+                    self.cards_on_this_host
+                        .iter()
+                        .position(|card| card.discrete_gpu().identifier == *device)
+                })
+                .collect(),
+        );
+    }
+
+    /// 换上 `settings`（它点的卡都开着）：不再点名的开着的卡关掉（[`Self::close_the_cards`]），参与的卡换成它点的那几张、按它的次序，
+    /// 每张卡的显存上限与等的上限换成它给的（不参与的卡不设显存上限，它分不到活）。
     /// 显存上限（扣掉卡上已占的）连最小的一片都装不下的卡不用、报出来（[`Self::gpus_left_out_under_their_memory_limit`]）；开卡时因显存上限没量代价、
     /// 或上一组设置因显存上限晾下而这一组给了装得下的上限的卡，照新上限量一次（量不量照旧、量出的代价不用来分活），量出错的这张卡不参与，
     /// 之后每一批报成它的 GPU 出错，直到下一次换设置。请回来的卡照参与的次序重新参与分活。
-    ///
-    /// # Panics
-    /// 它点的卡有一张既不在开着的独显里、也不在没开的那几张里：核心层照这份实现答的卡清单（此刻本机在的独显，开着的与没开的）校验过，
-    /// 点名不在清单里的就拒了。
-    pub fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
-        let devices_not_opened_at_the_switch =
-            self.open_the_gpus_named_and_not_yet_opened(settings);
+    fn put_the_settings_in_effect(&mut self, settings: &AdmittedGpuSettings) {
+        let devices_named_by_the_settings: Vec<GpuDeviceIdentifier> = settings
+            .participating_devices()
+            .iter()
+            .map(|participating| participating.device.clone())
+            .collect();
+        self.close_the_cards(|device| !devices_named_by_the_settings.contains(device));
         let identifiers_on_this_host = self.gpus_opened();
-        let participating_by_the_settings = ParticipatingGpusOnThisHost::of_the_settings(
-            settings,
-            &identifiers_on_this_host,
-            &devices_not_opened_at_the_switch,
-        );
+        let participating_by_the_settings =
+            ParticipatingGpusOnThisHost::of_the_settings(settings, &identifiers_on_this_host);
         for card in &mut self.cards_on_this_host {
             let memory_limit = memory_limit_of_the_device_under_the_settings(
                 settings,
@@ -2160,72 +2493,21 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         self.cost_not_measured_at_the_last_switch = cost_not_measured;
     }
 
-    /// 切换生效那一刻开这一组点到的、挂载时没开的卡，按参与的次序逐张开（[`gpu_opened_under_its_memory_limit`]，开卡那一步是 [`OpeningOfAGpuAtASwitch`]）：
-    /// 开成的排在开着的卡末尾；头一张开不成的记成这一组下每一批都报的 GPU 出错（[`BatchNotSplitOverTheGpus::GpuNotOpenedAtTheSwitch`]）。
-    /// 交回开不成的那几张（照旧没开，不参与）。
-    fn open_the_gpus_named_and_not_yet_opened(
-        &mut self,
-        settings: &AdmittedGpuSettings,
-    ) -> Vec<GpuDeviceIdentifier> {
-        let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
-        let mut devices_not_opened_at_the_switch = Vec::new();
-        let mut not_opened_at_the_switch = None;
-        for participating in settings.participating_devices() {
-            let Some(discrete_gpu_not_yet_opened) = self
-                .discrete_gpus_after_the_opening
-                .not_opened_for_not_taking_part
-                .iter()
-                .find(|discrete_gpu| discrete_gpu.identifier == participating.device)
-                .cloned()
-            else {
-                continue;
-            };
-            let opening_of_a_gpu_at_a_switch = &mut self.opening_of_a_gpu_at_a_switch;
-            let opened = gpu_opened_under_its_memory_limit(
-                &discrete_gpu_not_yet_opened,
-                participating.memory_limit,
-                smallest_piece_bytes,
-                &self.memory_leaked_in_this_process,
-                |discrete_gpu, memory_limit, cost_at_the_opening| {
-                    opening_of_a_gpu_at_a_switch(
-                        discrete_gpu,
-                        memory_limit,
-                        settings.wait_limit(),
-                        cost_at_the_opening,
-                    )
-                },
-            );
-            match opened {
-                // 装不装得下最小的一片在 `use_the_settings_from_the_next_batch` 里照这张卡的账再判一次（与开着的卡同一判）。
-                Ok((card, _left_out_judged_again_with_the_other_cards)) => {
-                    self.cards_on_this_host.push(card);
-                    self.discrete_gpus_after_the_opening
-                        .note_the_gpu_opened_at_a_switch(&discrete_gpu_not_yet_opened.identifier);
-                }
-                Err(cause) => {
-                    devices_not_opened_at_the_switch
-                        .push(discrete_gpu_not_yet_opened.identifier.clone());
-                    not_opened_at_the_switch
-                        .get_or_insert((discrete_gpu_not_yet_opened.identifier, cause));
-                }
-            }
-        }
-        self.not_opened_at_the_last_switch = not_opened_at_the_switch;
-        devices_not_opened_at_the_switch
-    }
-
-    /// 此刻这组设置下一批分不分得到卡上（与这一批有几条无关）：换设置时开一张挂载时没开的卡没开成、量一张卡的代价出错了，或参与的卡因显存上限一张都不剩，
-    /// 就交回是哪一种；分得到交回 `None`。挂载那一刻与切换生效那一批开头核心层经 `BackgroundGpuCompute::failure_under_the_settings_handed_in` 读它，不等头一批。
+    /// 此刻这组设置下一批分不分得到卡上（与这一批有几条无关）：最近交来的那一组点到的一张没开的卡重开到上限都没开成、换设置时量一张卡的代价出错了，
+    /// 或参与的卡因显存上限一张都不剩，就交回是哪一种；分得到交回 `None`（切换点到的卡还在开时照的是交来之前那一组）。
+    /// 挂载那一刻与切换生效那一批开头核心层经 `BackgroundGpuCompute::failure_under_the_settings_handed_in` 读它，不等头一批。
     ///
     /// # Panics
-    /// 参与的卡一张都不剩，却既没有开卡没开成、量代价出错，也没有因显存上限不用的卡：核心层校验过的一组至少点一张卡，没参与的只有这三种原因。
+    /// 参与的卡一张都不剩，却既没有量代价出错，也没有因显存上限不用的卡：核心层校验过的一组至少点一张卡，换上的那一组点的卡都开着，
+    /// 没参与的只有这两种原因。
     #[must_use]
     pub fn batch_not_split_over_the_gpus(&self) -> Option<BatchNotSplitOverTheGpus> {
-        if let Some((device, cause)) = &self.not_opened_at_the_last_switch {
-            return Some(BatchNotSplitOverTheGpus::GpuNotOpenedAtTheSwitch {
-                device: device.clone(),
-                cause: cause.clone(),
-            });
+        if let Some(not_opened) = &self.not_opened_after_the_reopenings_at_the_last_switch {
+            return Some(
+                BatchNotSplitOverTheGpus::GpuNotOpenedAfterTheReopeningsAtTheSwitch(
+                    not_opened.clone(),
+                ),
+            );
         }
         if let Some((device, cause)) = &self.cost_not_measured_at_the_last_switch {
             return Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch {
@@ -2238,7 +2520,7 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         }
         assert!(
             !self.left_out_under_their_memory_limit.is_empty(),
-            "参与的卡一张都不剩只有三种：换设置时开卡没开成、量代价出错（上面交回了），或这组设置点的卡都因显存上限不用"
+            "参与的卡一张都不剩只有两种：换设置时量代价出错（上面交回了），或这组设置点的卡都因显存上限不用"
         );
         Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(
             self.left_out_under_their_memory_limit.clone(),
@@ -2267,10 +2549,20 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         submissions_of_the_plan: impl Fn(&Plan) -> usize,
     ) -> Result<Vec<PlanOfAGpuForTheBatch<Plan>>, OutcomesOfTheBatchNotComputed> {
         match self.batch_not_split_over_the_gpus() {
-            Some(
-                BatchNotSplitOverTheGpus::GpuNotOpenedAtTheSwitch { device, cause }
-                | BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause },
-            ) => {
+            Some(BatchNotSplitOverTheGpus::GpuNotOpenedAfterTheReopeningsAtTheSwitch(
+                GpuNotOpenedAfterTheReopenings {
+                    device,
+                    causes_of_each_opening,
+                },
+            )) => {
+                return Err(OutcomesOfTheBatchNotComputed::GpuFailed {
+                    device,
+                    cause: GpuFailureCause::NotOpenedAfterTheReopenings {
+                        causes_of_each_opening,
+                    },
+                });
+            }
+            Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause }) => {
                 return Err(OutcomesOfTheBatchNotComputed::GpuFailed { device, cause });
             }
             Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)) => {
@@ -2827,7 +3119,7 @@ fn failure_of_the_measurement_dropping_or_leaking_the_card<Card: CardOfTheBackgr
 /// 卡清单（核心层挂载与切换照它校验设置，D26（后台整理与放置回收） 已定项 13 第 2、4 条）：此刻本机在的独显与各自的显存总量，按总线地址排
 /// （开着的与没参与而没开的都列，切换可以点没开的卡；没给 `background_gpu_devices` 时上限按这个次序一张卡一个；[`DiscreteGpusAfterTheOpening`]）。
 /// 答卡清单之前现查本机在的独显（[`DiscreteGpusPresentNow`]）：挂载之后掉了的卡不列，切换点它照「点名的卡不存在」拒、不点名时它不参与、一张都不剩照「GPU 不在」拒。
-/// 挂载那一刻没有没开成而跳过的卡（参与的卡没开成就拒绝挂载，不参与的不开），挂载那一刻没开成的那几张答空（核心层的缺省）。
+/// 切换点到的卡开不开得成不在校验里判：切换照收，开不成照 [`BackgroundGpuOverTheCards::put_the_gpus_opened_since_the_switch_in_effect`] 报。
 impl<Card: CardOfTheBackgroundGpu> GpuInventory for BackgroundGpuOverTheCards<Card> {
     fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
         self.discrete_gpus_after_the_opening
@@ -3110,6 +3402,12 @@ mod tests {
     use crate::two_kernel_reconciliation::EntryDecision;
 
     const WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU: Duration = Duration::from_secs(10);
+    /// 用例里开卡开不成照产品那样至多重开 3 次，间隔为 0：不真等 1 分钟。
+    const REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS: ReopeningsOfAGpuNotOpened =
+        ReopeningsOfAGpuNotOpened {
+            interval: Duration::ZERO,
+            reopenings_at_most: 3,
+        };
     const EIGHT_GIBIBYTES: u64 = 8 << 30;
 
     /// 替身卡上的钟不走：替身等前一次提交照脚本当场交回，不真等。
@@ -3425,6 +3723,7 @@ mod tests {
                 memory_leaked.clone(),
                 Box::new(DiscreteGpusPresentForTests(Arc::clone(&present))),
                 no_opening_at_a_switch(),
+                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
             ),
             present,
         )
@@ -4053,8 +4352,8 @@ mod tests {
         );
     }
 
-    /// 本机两张独显：「0000:01:00.0」挂载那一刻参与、开着；「0000:41:00.0」没参与、没开。切换点它时开它那一步交 `opening_at_the_switch`；
-    /// 开卡那一步被交了哪几张卡记在交回的那一份里。进程里泄漏的那一份是 `memory_leaked`。
+    /// 本机两张独显：「0000:01:00.0」挂载那一刻参与、开着；「0000:41:00.0」没参与、没开。切换点它时开它一次的那一步交 `opening_at_the_switch`，
+    /// 开不成照 `reopenings` 重开；开卡那一步被交了哪几张卡（每开一次记一次）记在交回的那一份里。进程里泄漏的那一份是 `memory_leaked`。
     fn background_gpu_with_one_card_not_opened_at_the_mount(
         opening_at_the_switch: impl FnMut(
                 &DiscreteGpuOnThisHost,
@@ -4064,6 +4363,7 @@ mod tests {
             ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
             + Send
             + 'static,
+        reopenings: ReopeningsOfAGpuNotOpened,
         memory_leaked: &GpuMemoryLeakedInThisProcess,
     ) -> (
         BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
@@ -4078,9 +4378,8 @@ mod tests {
             name: "cpu card not opened at the mount".to_string(),
             total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
         };
-        let cards_handed_to_the_opening = Arc::new(Mutex::new(Vec::new()));
-        let cards_noted_by_the_opening = Arc::clone(&cards_handed_to_the_opening);
-        let mut opening_at_the_switch = opening_at_the_switch;
+        let (opening_noting_the_cards, cards_handed_to_the_opening) =
+            opening_noting_the_cards_handed_to_it(opening_at_the_switch);
         let background_gpu = BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
             vec![opened_card],
             DiscreteGpusAfterTheOpening {
@@ -4095,6 +4394,29 @@ mod tests {
                 card_identifier("0000:01:00.0"),
                 card_identifier("0000:41:00.0"),
             ])))),
+            opening_noting_the_cards,
+            reopenings,
+        );
+        (background_gpu, cards_handed_to_the_opening)
+    }
+
+    /// 开卡那一步包一层：每开一次把交来的卡（总线地址）记进交回的那一份，再交 `opening_at_the_switch` 开。
+    fn opening_noting_the_cards_handed_to_it(
+        mut opening_at_the_switch: impl FnMut(
+                &DiscreteGpuOnThisHost,
+                Option<GpuMemoryInBytes>,
+                Duration,
+                CostAtTheOpening,
+            ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
+            + Send
+            + 'static,
+    ) -> (
+        OpeningOfAGpuAtASwitch<CardComputedOnTheCpuForTests>,
+        Arc<Mutex<Vec<String>>>,
+    ) {
+        let cards_handed_to_the_opening = Arc::new(Mutex::new(Vec::new()));
+        let cards_noted_by_the_opening = Arc::clone(&cards_handed_to_the_opening);
+        let opening_noting_the_cards: OpeningOfAGpuAtASwitch<CardComputedOnTheCpuForTests> =
             Box::new(
                 move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
                     cards_noted_by_the_opening
@@ -4108,9 +4430,84 @@ mod tests {
                         cost_at_the_opening,
                     )
                 },
-            ),
+            );
+        (opening_noting_the_cards, cards_handed_to_the_opening)
+    }
+
+    /// 开卡那一步：每一次都开成交来的那张卡（替身卡，照给它的显存上限）。
+    fn opening_that_always_opens(
+        discrete_gpu: &DiscreteGpuOnThisHost,
+        memory_limit: Option<GpuMemoryInBytes>,
+        _wait_limit: Duration,
+        _cost_at_the_opening: CostAtTheOpening,
+    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause> {
+        Ok(CardComputedOnTheCpuForTests::of_the_address(
+            &discrete_gpu.pci_bus_address,
+            memory_limit.map(|memory_limit| memory_limit.0),
+            0,
+        ))
+    }
+
+    /// 用例里切换那一刻开不成的原因。
+    fn reason_the_card_does_not_open_at_the_switch() -> GpuFailureCause {
+        GpuFailureCause::DeviceOrPipelineDidNotStart {
+            reason: "替身：切换那一刻起不了设备".to_string(),
+        }
+    }
+
+    /// 本机两张独显「0000:01:00.0」「0000:41:00.0」挂载那一刻都参与、都开着（按这个次序），切换再点到关掉的卡时开它一次的那一步交 `opening_at_the_switch`；
+    /// 交回外壳、开卡那一步被交了哪几张卡，与「0000:41:00.0」那张替身卡析构了几次。「0000:41:00.0」那张交给用例改（例：记上一次还在跑的提交）。
+    fn background_gpu_with_both_cards_opened_at_the_mount(
+        opening_at_the_switch: impl FnMut(
+                &DiscreteGpuOnThisHost,
+                Option<GpuMemoryInBytes>,
+                Duration,
+                CostAtTheOpening,
+            ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
+            + Send
+            + 'static,
+        card_at_forty_one_as_the_test_wants: impl FnOnce(
+            CardComputedOnTheCpuForTests,
+        ) -> CardComputedOnTheCpuForTests,
+        memory_leaked: &GpuMemoryLeakedInThisProcess,
+    ) -> (
+        BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
+        Arc<Mutex<Vec<String>>>,
+        Arc<std::sync::atomic::AtomicU32>,
+    ) {
+        let card_at_one = CardComputedOnTheCpuForTests::of_the_address("0000:01:00.0", None, 0);
+        let card_at_forty_one = card_at_forty_one_as_the_test_wants(
+            CardComputedOnTheCpuForTests::of_the_address("0000:41:00.0", None, 0),
         );
-        (background_gpu, cards_handed_to_the_opening)
+        let drops_of_the_card_at_forty_one = Arc::clone(&card_at_forty_one.drops_counted);
+        let opened = vec![
+            card_at_one.discrete_gpu.clone(),
+            card_at_forty_one.discrete_gpu.clone(),
+        ];
+        let (opening_noting_the_cards, cards_handed_to_the_opening) =
+            opening_noting_the_cards_handed_to_it(opening_at_the_switch);
+        let background_gpu = BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
+            vec![card_at_one, card_at_forty_one],
+            DiscreteGpusAfterTheOpening {
+                opened,
+                not_opened_for_not_taking_part: Vec::new(),
+            },
+            Vec::new(),
+            ParticipatingGpusOnThisHost::of_the_positions(vec![0, 1]),
+            Vec::new(),
+            memory_leaked.clone(),
+            Box::new(DiscreteGpusPresentForTests(Arc::new(Mutex::new(vec![
+                card_identifier("0000:01:00.0"),
+                card_identifier("0000:41:00.0"),
+            ])))),
+            opening_noting_the_cards,
+            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+        );
+        (
+            background_gpu,
+            cards_handed_to_the_opening,
+            drops_of_the_card_at_forty_one,
+        )
     }
 
     /// 核心层照外壳答的卡清单校验点「0000:01:00.0」「0000:41:00.0」两张的切换（照这个次序）。
@@ -4129,112 +4526,7 @@ mod tests {
             },
             background_gpu,
         )
-        .expect("卡清单列着挂载时没开的那张：切换点它照收")
-    }
-
-    /// 欠账 C996 的定案「切换到新卡时再开」：挂载点「0000:01:00.0」、切换点它与「0000:41:00.0」——核心层照卡清单收下这一次切换；
-    /// 切换生效那一刻（交设置）开「0000:41:00.0」，只开这一张、只开一次，之后两张都参与、按点名的次序；再交同一组不重开。
-    #[test]
-    fn the_switch_naming_a_card_not_opened_at_the_mount_opens_that_card_when_the_switch_takes_effect(
-    ) {
-        let (mut background_gpu, cards_handed_to_the_opening) =
-            background_gpu_with_one_card_not_opened_at_the_mount(
-                |discrete_gpu, memory_limit, _wait_limit, _cost_at_the_opening| {
-                    Ok(CardComputedOnTheCpuForTests::of_the_address(
-                        &discrete_gpu.pci_bus_address,
-                        memory_limit.map(|memory_limit| memory_limit.0),
-                        0,
-                    ))
-                },
-                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
-            );
-        let admitted = switch_naming_both_cards_admitted_against(&background_gpu);
-        assert!(
-            cards_handed_to_the_opening
-                .lock()
-                .expect("用例没在别处 panic 过")
-                .is_empty(),
-            "校验那一刻不开卡"
-        );
-        background_gpu.use_the_settings_from_the_next_batch(&admitted);
-        assert_eq!(
-            *cards_handed_to_the_opening
-                .lock()
-                .expect("用例没在别处 panic 过"),
-            vec!["0000:41:00.0".to_string()],
-            "切换生效那一刻只开挂载时没开的那一张"
-        );
-        assert_eq!(
-            background_gpu.gpus_opened(),
-            vec![
-                card_identifier("0000:01:00.0"),
-                card_identifier("0000:41:00.0")
-            ]
-        );
-        assert_eq!(
-            background_gpu
-                .participating_gpus()
-                .into_iter()
-                .map(|participating| participating.identifier)
-                .collect::<Vec<_>>(),
-            vec![
-                card_identifier("0000:01:00.0"),
-                card_identifier("0000:41:00.0")
-            ],
-            "两张都参与、按点名的次序"
-        );
-        assert_eq!(background_gpu.batch_not_split_over_the_gpus(), None);
-        background_gpu.use_the_settings_from_the_next_batch(&admitted);
-        assert_eq!(
-            cards_handed_to_the_opening
-                .lock()
-                .expect("用例没在别处 panic 过")
-                .len(),
-            1,
-            "开着的卡不重开"
-        );
-    }
-
-    /// 切换生效那一刻开挂载时没开的那张卡没开成：它不参与、照旧没开，另一张照常参与；这一组下每一批都报成它的 GPU 出错（带卡与原因），
-    /// 核心层在切换生效那一批开头读到、当场报警停着（`failure_under_the_settings_handed_in`）。
-    #[test]
-    fn the_card_that_does_not_open_when_the_switch_takes_effect_takes_no_part_and_every_batch_reports_its_failure(
-    ) {
-        let reason_it_did_not_open = GpuFailureCause::DeviceOrPipelineDidNotStart {
-            reason: "替身：切换那一刻起不了设备".to_string(),
-        };
-        let cause_handed_back = reason_it_did_not_open.clone();
-        let (mut background_gpu, _cards_handed_to_the_opening) =
-            background_gpu_with_one_card_not_opened_at_the_mount(
-                move |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
-                    Err(cause_handed_back.clone())
-                },
-                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
-            );
-        let admitted = switch_naming_both_cards_admitted_against(&background_gpu);
-        background_gpu.use_the_settings_from_the_next_batch(&admitted);
-        assert_eq!(
-            background_gpu.batch_not_split_over_the_gpus(),
-            Some(BatchNotSplitOverTheGpus::GpuNotOpenedAtTheSwitch {
-                device: card_identifier("0000:41:00.0"),
-                cause: reason_it_did_not_open,
-            }),
-            "开不成的那张报成它的 GPU 出错"
-        );
-        assert_eq!(
-            background_gpu.gpus_opened(),
-            vec![card_identifier("0000:01:00.0")],
-            "开不成的那张照旧没开"
-        );
-        assert_eq!(
-            background_gpu
-                .participating_gpus()
-                .into_iter()
-                .map(|participating| participating.identifier)
-                .collect::<Vec<_>>(),
-            vec![card_identifier("0000:01:00.0")],
-            "开不成的那张不参与"
-        );
+        .expect("卡清单列着没开的那张：切换点它照收")
     }
 
     /// 核心层照外壳答的卡清单校验只点 `pci_bus_address` 那一张的切换。
@@ -4271,20 +4563,26 @@ mod tests {
         (outcome, devices_given_work)
     }
 
-    /// 欠账 C996 的定案「切换到新卡时再开」：挂载点「0000:01:00.0」，切换只点挂载时没开的「0000:41:00.0」——切换生效那一刻开它一次，
-    /// 下一批只分给它；切回「0000:01:00.0」、再切到「0000:41:00.0」都不重开（开着的卡留着），每次切换之后那一批只分给点的那一张。
+    /// 此刻参与的卡，按参与的次序。
+    fn participating_devices_of(
+        background_gpu: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
+    ) -> Vec<GpuDeviceIdentifier> {
+        background_gpu
+            .participating_gpus()
+            .into_iter()
+            .map(|participating| participating.identifier)
+            .collect()
+    }
+
+    /// 欠账 C996 的定案「切换到新卡时再开」：挂载点「0000:01:00.0」、切换点它与「0000:41:00.0」——核心层照卡清单收下这一次切换、校验那一刻不开卡；
+    /// 交设置之后开「0000:41:00.0」（只开这一张、只开一次），开完了在两批之间换上，两张都参与、按点名的次序；再交同一组不重开。
     #[test]
-    fn the_switch_to_only_the_card_not_opened_at_the_mount_opens_it_once_and_switching_back_and_forth_never_reopens_it(
+    fn the_switch_naming_a_card_not_opened_at_the_mount_opens_that_card_when_the_switch_takes_effect(
     ) {
         let (mut background_gpu, cards_handed_to_the_opening) =
             background_gpu_with_one_card_not_opened_at_the_mount(
-                |discrete_gpu, memory_limit, _wait_limit, _cost_at_the_opening| {
-                    Ok(CardComputedOnTheCpuForTests::of_the_address(
-                        &discrete_gpu.pci_bus_address,
-                        memory_limit.map(|memory_limit| memory_limit.0),
-                        0,
-                    ))
-                },
+                opening_that_always_opens,
+                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                 &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
             );
         let openings_so_far = || {
@@ -4293,94 +4591,226 @@ mod tests {
                 .expect("用例没在别处 panic 过")
                 .clone()
         };
-        let to_the_card_not_opened_at_the_mount =
-            switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
-        let back_to_the_card_opened_at_the_mount =
-            switch_naming_one_card_admitted_against(&background_gpu, "0000:01:00.0");
-        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_not_opened_at_the_mount);
+        let admitted = switch_naming_both_cards_admitted_against(&background_gpu);
+        assert!(openings_so_far().is_empty(), "校验那一刻不开卡");
+        background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            None,
+            "开成了：没有开不成的卡要报"
+        );
         assert_eq!(
             openings_so_far(),
             vec!["0000:41:00.0".to_string()],
-            "切换生效那一刻开挂载时没开的那一张、开一次"
+            "只开挂载时没开的那一张"
         );
-        let (outcome_after_the_switch, devices_given_work_after_the_switch) =
-            checksum_batch_and_the_devices_given_work(&mut background_gpu);
-        assert!(
-            outcome_after_the_switch.is_ok(),
-            "切到开成的那一张：照常算（{outcome_after_the_switch:?}）"
+        assert_eq!(
+            background_gpu.gpus_opened(),
+            vec![
+                card_identifier("0000:01:00.0"),
+                card_identifier("0000:41:00.0")
+            ]
         );
         assert_eq!(
-            devices_given_work_after_the_switch,
-            vec![card_identifier("0000:41:00.0")],
-            "切换之后那一批只分给切到的那一张"
+            participating_devices_of(&background_gpu),
+            vec![
+                card_identifier("0000:01:00.0"),
+                card_identifier("0000:41:00.0")
+            ],
+            "两张都参与、按点名的次序"
         );
-        background_gpu.use_the_settings_from_the_next_batch(&back_to_the_card_opened_at_the_mount);
-        let (outcome_after_switching_back, devices_given_work_after_switching_back) =
+        assert_eq!(background_gpu.batch_not_split_over_the_gpus(), None);
+        background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            None
+        );
+        assert_eq!(openings_so_far().len(), 1, "开着的卡不重开");
+    }
+
+    /// 用户定案「切换请求照收，在会话锁外开新卡……这期间照旧用切换前那组卡。开成了，下一批起换成新设置」：切换只点「0000:41:00.0」，
+    /// 开它那一步等用例放开才交回——交设置照常返回、不等开卡（核心层在会话锁里交设置），两批之间不等它的那一问交回 `None`，
+    /// 其间那一批照切换之前那一组只分给「0000:01:00.0」；放开之后换上，下一批只分给「0000:41:00.0」，不再点名的那张关掉。
+    #[test]
+    fn the_handing_in_and_the_batches_while_the_card_opens_do_not_wait_for_it_and_run_on_the_cards_before_the_switch(
+    ) {
+        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
+        let (mut background_gpu, _cards_handed_to_the_opening) =
+            background_gpu_with_one_card_not_opened_at_the_mount(
+                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
+                    // 等放开有个上限：交设置要是在这一刻等开卡（会话锁里等），开卡到点交回开不成，用例照断言红，不一直挂着。
+                    opening_released
+                        .recv_timeout(Duration::from_secs(20))
+                        .map_err(|_not_released| reason_the_card_does_not_open_at_the_switch())?;
+                    opening_that_always_opens(
+                        discrete_gpu,
+                        memory_limit,
+                        wait_limit,
+                        cost_at_the_opening,
+                    )
+                },
+                ReopeningsOfAGpuNotOpened {
+                    interval: Duration::ZERO,
+                    reopenings_at_most: 0,
+                },
+                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            );
+        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
+        background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::NotAtAll
+            ),
+            None,
+            "还在开：不等它"
+        );
+        let (outcome_while_opening, devices_given_work_while_opening) =
             checksum_batch_and_the_devices_given_work(&mut background_gpu);
         assert!(
-            outcome_after_switching_back.is_ok(),
-            "切回挂载时开的那一张：照常算（{outcome_after_switching_back:?}）"
+            outcome_while_opening.is_ok(),
+            "开着的这段时间照常算（{outcome_while_opening:?}）"
         );
         assert_eq!(
-            devices_given_work_after_switching_back,
+            devices_given_work_while_opening,
             vec![card_identifier("0000:01:00.0")],
-            "切回之后那一批只分给挂载时开的那一张"
+            "开着的这段时间照旧照切换之前那一组跑"
         );
-        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_not_opened_at_the_mount);
-        let (outcome_after_switching_again, devices_given_work_after_switching_again) =
+        release_the_opening
+            .send(())
+            .expect("开卡那一步还在等放开：收的那一头还在");
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            None,
+            "开成了：没有开不成的卡要报"
+        );
+        let (outcome_after_the_opening, devices_given_work_after_the_opening) =
             checksum_batch_and_the_devices_given_work(&mut background_gpu);
         assert!(
-            outcome_after_switching_again.is_ok(),
-            "再切到切换时开的那一张：照常算（{outcome_after_switching_again:?}）"
+            outcome_after_the_opening.is_ok(),
+            "换上之后照常算（{outcome_after_the_opening:?}）"
         );
         assert_eq!(
-            devices_given_work_after_switching_again,
-            vec![card_identifier("0000:41:00.0")]
+            devices_given_work_after_the_opening,
+            vec![card_identifier("0000:41:00.0")],
+            "开成了：下一批起换成新设置"
         );
         assert_eq!(
-            openings_so_far(),
-            vec!["0000:41:00.0".to_string()],
-            "切回、再切到都不重开：开卡那一步一共只走过一次"
+            background_gpu.gpus_opened(),
+            vec![card_identifier("0000:41:00.0")],
+            "不再点名的那张关掉"
+        );
+    }
+
+    /// 用户定案「开不成就间隔 1 分钟重开，最多重开 3 次」：切换点到的「0000:41:00.0」头两次开不成、第三次开成——照常换上、两张都参与，
+    /// 开卡那一步一共只开它 3 次。
+    #[test]
+    fn the_card_not_opened_twice_at_the_switch_opens_the_third_time_and_the_switch_takes_effect() {
+        let openings_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
+        let openings_counted = Arc::clone(&openings_made);
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_one_card_not_opened_at_the_mount(
+                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
+                    let opening =
+                        openings_counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
+                    if opening < 3 {
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
+        let admitted = switch_naming_both_cards_admitted_against(&background_gpu);
+        background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            None,
+            "第三次开成：没有开不成的卡要报"
         );
         assert_eq!(
-            background_gpu.gpus_opened(),
+            *cards_handed_to_the_opening
+                .lock()
+                .expect("用例没在别处 panic 过"),
+            vec!["0000:41:00.0".to_string(); 3],
+            "开不成两次之后重开、第三次开成就不再开"
+        );
+        assert_eq!(
+            participating_devices_of(&background_gpu),
             vec![
                 card_identifier("0000:01:00.0"),
                 card_identifier("0000:41:00.0")
             ],
-            "两张都开着：不再参与的那张照开着"
+            "换上了：两张都参与"
         );
+        assert_eq!(background_gpu.batch_not_split_over_the_gpus(), None);
     }
 
-    /// 切换只点挂载时没开的那一张、切换生效那一刻它开不成：这一次切换按出错报（这张卡的 GPU 出错，带原因），之后那一批交回同一个错、
-    /// 一条活都不分给它（也不分给不再参与的那一张）。
+    /// 用户定案「3 次都不成：GC 停下，报警并报出哪张卡、每次开不成的原因，等人处理，不自动再试」：切换只点「0000:41:00.0」，它头一次与 3 次重开都开不成——
+    /// 两批之间那一问交回这张卡与 4 次的原因（只交这一次，之后不再开），之后那一批报成这张卡的 GPU 出错（带每一次的原因）、一条活都不分出去；
+    /// 照旧只开着切换之前那一张。下一次换设置（切回「0000:01:00.0」）解除它，那一批照常只分给它。
     #[test]
     fn the_switch_to_only_a_card_that_does_not_open_is_reported_as_its_gpu_failure_and_gives_it_no_work(
     ) {
-        let reason_it_did_not_open = GpuFailureCause::DeviceOrPipelineDidNotStart {
-            reason: "替身：切换那一刻起不了设备".to_string(),
-        };
-        let cause_handed_back = reason_it_did_not_open.clone();
-        let (mut background_gpu, _cards_handed_to_the_opening) =
+        let (mut background_gpu, cards_handed_to_the_opening) =
             background_gpu_with_one_card_not_opened_at_the_mount(
-                move |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
-                    Err(cause_handed_back.clone())
+                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
+                    Err(reason_the_card_does_not_open_at_the_switch())
                 },
+                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                 &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
             );
+        let not_opened = GpuNotOpenedAfterTheReopenings {
+            device: card_identifier("0000:41:00.0"),
+            causes_of_each_opening: vec![reason_the_card_does_not_open_at_the_switch(); 4],
+        };
         let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
         background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            Some(not_opened.clone()),
+            "头一次与 3 次重开都开不成：报出这张卡与每一次的原因"
+        );
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            None,
+            "只报这一次"
+        );
+        assert_eq!(
+            *cards_handed_to_the_opening
+                .lock()
+                .expect("用例没在别处 panic 过"),
+            vec!["0000:41:00.0".to_string(); 4],
+            "开 4 次之后不自动再试"
+        );
         assert_eq!(
             background_gpu.batch_not_split_over_the_gpus(),
-            Some(BatchNotSplitOverTheGpus::GpuNotOpenedAtTheSwitch {
-                device: card_identifier("0000:41:00.0"),
-                cause: reason_it_did_not_open.clone(),
-            }),
-            "切换按这张卡的 GPU 出错报"
+            Some(
+                BatchNotSplitOverTheGpus::GpuNotOpenedAfterTheReopeningsAtTheSwitch(
+                    not_opened.clone()
+                )
+            ),
         );
-        assert!(
-            background_gpu.participating_gpus().is_empty(),
-            "开不成的那张不参与、不再点的那张也不参与"
+        assert_eq!(
+            background_gpu.gpus_opened(),
+            vec![card_identifier("0000:01:00.0")],
+            "开不成：照旧只开着切换之前那一张"
         );
         let (outcome, devices_given_work) =
             checksum_batch_and_the_devices_given_work(&mut background_gpu);
@@ -4388,7 +4818,9 @@ mod tests {
             outcome,
             Err(ChecksumBatchFailed::GpuFailed {
                 device: card_identifier("0000:41:00.0"),
-                cause: reason_it_did_not_open,
+                cause: GpuFailureCause::NotOpenedAfterTheReopenings {
+                    causes_of_each_opening: not_opened.causes_of_each_opening,
+                },
             }),
             "之后那一批报成这张卡的 GPU 出错"
         );
@@ -4396,11 +4828,116 @@ mod tests {
             devices_given_work.is_empty(),
             "一条活都没分出去（{devices_given_work:?}）"
         );
+        let back_to_the_card_opened_at_the_mount =
+            switch_naming_one_card_admitted_against(&background_gpu, "0000:01:00.0");
+        background_gpu.use_the_settings_from_the_next_batch(&back_to_the_card_opened_at_the_mount);
+        assert_eq!(
+            background_gpu.batch_not_split_over_the_gpus(),
+            None,
+            "下一次换设置解除它"
+        );
+        let (outcome_after_switching_back, devices_given_work_after_switching_back) =
+            checksum_batch_and_the_devices_given_work(&mut background_gpu);
+        assert!(outcome_after_switching_back.is_ok());
+        assert_eq!(
+            devices_given_work_after_switching_back,
+            vec![card_identifier("0000:01:00.0")]
+        );
+    }
+
+    /// 用户定案「切换之后不再参与的卡关不关」=「切换时关掉」：挂载开着「0000:01:00.0」「0000:41:00.0」，切换只点「0000:01:00.0」——
+    /// 换上那一刻「0000:41:00.0」关掉（替身卡析构一次、不再开着），卡清单照列它；再切到两张都点，它重开（开卡那一步走一次），两张都参与。
+    #[test]
+    fn the_card_no_longer_named_by_a_switch_is_closed_and_reopened_when_named_again() {
+        let (mut background_gpu, cards_handed_to_the_opening, drops_of_the_card_at_forty_one) =
+            background_gpu_with_both_cards_opened_at_the_mount(
+                opening_that_always_opens,
+                |card| card,
+                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            );
+        let only_the_card_at_one =
+            switch_naming_one_card_admitted_against(&background_gpu, "0000:01:00.0");
+        background_gpu.use_the_settings_from_the_next_batch(&only_the_card_at_one);
+        assert_eq!(
+            drops_of_the_card_at_forty_one.load(std::sync::atomic::Ordering::SeqCst),
+            1,
+            "不再点名的那张关掉：设备丢掉"
+        );
+        assert_eq!(
+            background_gpu.gpus_opened(),
+            vec![card_identifier("0000:01:00.0")]
+        );
+        assert_eq!(
+            background_gpu
+                .devices_with_total_memory()
+                .into_iter()
+                .map(|(device, _total_memory)| device)
+                .collect::<Vec<_>>(),
+            vec![
+                card_identifier("0000:01:00.0"),
+                card_identifier("0000:41:00.0")
+            ],
+            "卡清单照列关掉的那张：切换可以再点它"
+        );
+        let both_cards = switch_naming_both_cards_admitted_against(&background_gpu);
+        background_gpu.use_the_settings_from_the_next_batch(&both_cards);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            None
+        );
+        assert_eq!(
+            *cards_handed_to_the_opening
+                .lock()
+                .expect("用例没在别处 panic 过"),
+            vec!["0000:41:00.0".to_string()],
+            "再点它就重开"
+        );
+        assert_eq!(
+            participating_devices_of(&background_gpu),
+            vec![
+                card_identifier("0000:01:00.0"),
+                card_identifier("0000:41:00.0")
+            ]
+        );
     }
 
-    /// 第 4 条「有这种提交的那张卡整张泄漏、不析构」用于切换生效那一刻开的卡：切换生效那一刻开挂载时没开的「0000:41:00.0」，开卡那一步是产品那一份
-    /// （`opening_of_a_card_within_the_limit`，起设备那一件由用例交、造一张替身卡），替身卡上记着一次等超过上限、还在跑的提交（占 600 字节），量代价出错——
-    /// 这张卡不析构（替身的析构一次都没走），进程里的泄漏账在它上面加这一次的 600 字节；切换按这张卡的 GPU 出错报（量代价那一次的错，原样），它不参与。
+    /// 第 4 条「有这种提交的那张卡整张泄漏、不析构」用于切换时关掉的卡：「0000:41:00.0」上记着一次等超过上限、还在跑的提交（占 600 字节），
+    /// 切换不再点它——它不析构，进程里的泄漏账在它上面加这一次的 600 字节；它照样不再开着。
+    #[test]
+    fn the_card_closed_at_a_switch_with_a_submission_still_running_is_leaked_and_noted_in_the_process(
+    ) {
+        let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
+        let (mut background_gpu, _cards_handed_to_the_opening, drops_of_the_card_at_forty_one) =
+            background_gpu_with_both_cards_opened_at_the_mount(
+                opening_that_always_opens,
+                |card| card.with_a_submission_running_past_its_wait(600),
+                &memory_leaked,
+            );
+        let only_the_card_at_one =
+            switch_naming_one_card_admitted_against(&background_gpu, "0000:01:00.0");
+        background_gpu.use_the_settings_from_the_next_batch(&only_the_card_at_one);
+        assert_eq!(
+            drops_of_the_card_at_forty_one.load(std::sync::atomic::Ordering::SeqCst),
+            0,
+            "提交还在跑：关掉的这张泄漏、不析构"
+        );
+        assert_eq!(
+            memory_leaked.bytes_leaked_on(&card_identifier("0000:41:00.0")),
+            600,
+            "泄漏账加的恰好是这一次占着的 600 字节"
+        );
+        assert_eq!(
+            background_gpu.gpus_opened(),
+            vec![card_identifier("0000:01:00.0")]
+        );
+    }
+
+    /// 第 4 条「有这种提交的那张卡整张泄漏、不析构」用于切换开的卡：切换点挂载时没开的「0000:41:00.0」，开卡那一步是产品那一份
+    /// （`opening_of_a_card_within_the_limit`，起设备那一件由用例交、造一张替身卡），替身卡上记着一次等超过上限、还在跑的提交（占 600 字节），量代价次次出错——
+    /// 头一次与 3 次重开造的 4 张卡都不析构（替身的析构一次都没走），进程里的泄漏账在它上面每一次加 600 字节；切换按这张卡重开到上限都没开成报（每一次的原因
+    /// 都是量代价那一次的错，原样），它没算开成。
     #[test]
     fn the_card_opened_at_the_switch_whose_cost_measurement_fails_with_a_submission_still_running_is_leaked_and_noted_in_the_process(
     ) {
@@ -4429,27 +4966,30 @@ mod tests {
         let (mut background_gpu, _cards_handed_to_the_opening) =
             background_gpu_with_one_card_not_opened_at_the_mount(
                 opening_at_the_switch,
+                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
                 &memory_leaked,
             );
         let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
         background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            Some(GpuNotOpenedAfterTheReopenings {
+                device: card_identifier("0000:41:00.0"),
+                causes_of_each_opening: vec![failure_of_the_cost_measurement_in_the_test(); 4],
+            }),
+            "切换按这张卡重开到上限都没开成报，每一次的原因是量代价那一次的错"
+        );
         assert_eq!(
             drops_counted.load(std::sync::atomic::Ordering::SeqCst),
             0,
-            "提交还在跑：切换开的这张卡泄漏、不析构"
+            "提交还在跑：切换开的这几张卡泄漏、不析构"
         );
         assert_eq!(
             memory_leaked.bytes_leaked_on(&card_identifier("0000:41:00.0")),
-            600,
-            "泄漏账加的恰好是这一次占着的 600 字节"
-        );
-        assert_eq!(
-            background_gpu.batch_not_split_over_the_gpus(),
-            Some(BatchNotSplitOverTheGpus::GpuNotOpenedAtTheSwitch {
-                device: card_identifier("0000:41:00.0"),
-                cause: failure_of_the_cost_measurement_in_the_test(),
-            }),
-            "切换按这张卡的 GPU 出错报，原因是量代价那一次的错"
+            4 * 600,
+            "泄漏账每一次加的恰好是那一次占着的 600 字节"
         );
         assert_eq!(
             background_gpu.gpus_opened(),
````

### crates/singlefs-background-gpu/src/background_gpu_compute.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/background_gpu_compute.rs b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
index 48c8099..9b2af18 100644
--- a/crates/singlefs-background-gpu/src/background_gpu_compute.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
@@ -15,16 +15,17 @@
 //! 两份核各算一次的那一层抽成 [`BothKernelsOfTheThreeSegments`]：产品是 `background_gpu::BackgroundGpu`（参与的卡上两份 WGSL 核）；
 //! 测试里换上在 CPU 上算的替身，比对、计数、映射这几样不起卡就钉得住。
 //! 挂载那一刻的入口 [`open_background_gpu_compute`]：只在参与的卡上开卡（卡清单、显存上限、等 GPU 的上限，校验同核心层 `admit_background_compute_settings`），
-//! 交回核心层挂载与切换收的把手；切换点到挂载时没开的卡，切换生效那一刻再开（`background_gpu::BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`）。
+//! 交回核心层挂载与切换收的把手；切换点到没开的卡，在会话锁外开、开完了在两批之间换上（`background_gpu::BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`），
+//! 重开到上限都没开成映射成 `GpuNotOpenedAfterTheReopenings`，带卡与每一次的原因，核心层当场停 GC、报警。新开的卡并进这一份实现：连续不确定的计数不因切换清零。
 
 use singlefs_core::allocator::AllocationRecord;
 use singlefs_core::background_compute::{
     AdmittedGpuSettings, BackgroundGpuCompute, BackgroundGpuComputeFailed,
-    BackgroundGpuComputeHandle, ClusterSegmentEntryKey, GpuDeviceIdentifier,
-    GpuDeviceNotOpenedAtTheMount, GpuEntryDecision, GpuFailureOnTheDevice, GpuInventory,
-    GpuMemoryInBytes, GpuShortOfFreeMemoryForTheSmallestPiece, GpuSubmissionsOfTheBatch,
-    InputNotSupportedOnTheGpu, RegionRecordEntryKey, RegionRecordToClassifyOnTheGpu,
-    SubmissionWaitedForPastTheLimit, UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
+    BackgroundGpuComputeHandle, ClusterSegmentEntryKey, GpuDeviceIdentifier, GpuEntryDecision,
+    GpuFailureOnTheDevice, GpuInventory, GpuMemoryInBytes, GpuShortOfFreeMemoryForTheSmallestPiece,
+    GpuSubmissionsOfTheBatch, InputNotSupportedOnTheGpu, RegionRecordEntryKey,
+    RegionRecordToClassifyOnTheGpu, SubmissionWaitedForPastTheLimit, UnitChecksumOutcome,
+    UnitChecksumToVerifyOnTheGpu,
 };
 use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
 use singlefs_core::logged_operations::CompactionRegion;
@@ -34,8 +35,9 @@ use crate::adapter_inventory::{BackgroundGpuSelection, DiscreteGpuListRefused};
 use crate::background_gpu::{
     open_background_gpu_with_the_memory_leaked_in_this_process, BackgroundGpu,
     BackgroundGpuOpenRefused, BackgroundGpuOverTheCards, BatchNotSplitOverTheGpus,
-    CardOfTheBackgroundGpu, ChecksumBatchFailed, GpuFailureCause, LiveSlotBatchFailed,
-    RegionRecordBatchFailed, SmallestPieceNotHeldOnTheGpu, VulkanInstanceOfThisProcess,
+    CardOfTheBackgroundGpu, ChecksumBatchFailed, GpuFailureCause, GpuNotOpenedAfterTheReopenings,
+    LiveSlotBatchFailed, RegionRecordBatchFailed, SmallestPieceNotHeldOnTheGpu,
+    VulkanInstanceOfThisProcess, WaitForTheOpeningAtASwitch,
 };
 use crate::checksum_kernels::{
     reconciled_unit_checksums, ChecksumKernelOutcome, UnitChecksumToVerify,
@@ -87,10 +89,18 @@ pub trait BothKernelsOfTheThreeSegments: GpuInventory {
     ) -> Result<Vec<(u64, u64)>, LiveSlotBatchFailed>;
 
     /// 从下一批起照核心层交来的这一组跑：只在它点的卡上、按卡的顺序填满（前面的卡填到它照显存上限一次放得下的量，装不下的才给后面的卡；
-    /// 挂载时量出的代价不用来分活），照它给的显存上限切段、照它给的等的上限等；不重开设备
+    /// 挂载时量出的代价不用来分活），照它给的显存上限切段、照它给的等的上限等；开着的卡不重开，点到没开的卡在会话锁外开、开完了才换上
     /// （`singlefs_core::background_compute::BackgroundGpuCompute::use_the_settings_from_the_next_batch` 转来的，只在两批之间）。
     fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings);
 
+    /// 两批之间：会话锁外为切换开卡的那一次开完了就换上（[`BackgroundGpuOverTheCards::put_the_gpus_opened_since_the_switch_in_effect`]，不等它开完），
+    /// 重开到上限都没开成交回那张卡与每一次的原因（只交一次）。不开卡的替身没有这一样，交回 `None`。
+    fn put_the_gpus_opened_since_the_switch_in_effect(
+        &mut self,
+    ) -> Option<GpuNotOpenedAfterTheReopenings> {
+        None
+    }
+
     /// 此刻这组设置下一批分不分得到卡上（与这一批有几条无关）：分不到交回是哪一种（换设置时量代价出错、参与的卡因显存上限一张都不剩），
     /// 分得到交回 `None`。只读，不交活。
     fn batch_not_split_over_the_gpus_under_the_settings_in_use(
@@ -137,6 +147,15 @@ impl<Card: CardOfTheBackgroundGpu> BothKernelsOfTheThreeSegments
         BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch(self, settings);
     }
 
+    fn put_the_gpus_opened_since_the_switch_in_effect(
+        &mut self,
+    ) -> Option<GpuNotOpenedAfterTheReopenings> {
+        BackgroundGpuOverTheCards::put_the_gpus_opened_since_the_switch_in_effect(
+            self,
+            WaitForTheOpeningAtASwitch::NotAtAll,
+        )
+    }
+
     fn batch_not_split_over_the_gpus_under_the_settings_in_use(
         &self,
     ) -> Option<BatchNotSplitOverTheGpus> {
@@ -349,10 +368,20 @@ pub fn core_failure_of_the_gpu(
             | GpuFailureCause::ReadbackMappingFailed { .. }
             | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
             | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
-            | GpuFailureCause::UnusableAfterResubmissions { .. }) => {
+            | GpuFailureCause::UnusableAfterResubmissions { .. }
+            | GpuFailureCause::NotOpenedAfterTheReopenings { .. }) => {
                 core_failure_of_the_gpu(device, last_cause_other_than_a_wait_past_the_limit)
             }
         },
+        GpuFailureCause::NotOpenedAfterTheReopenings {
+            causes_of_each_opening,
+        } => BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings {
+            device: device.clone(),
+            failures_of_each_opening: causes_of_each_opening
+                .iter()
+                .map(|cause_of_the_opening| core_failure_of_the_gpu(device, cause_of_the_opening))
+                .collect(),
+        },
     }
 }
 
@@ -379,15 +408,18 @@ pub fn core_failure_of_every_participating_gpu_short_of_free_memory(
     }
 }
 
-/// 此刻这组设置下一批分不到卡上，报给核心层：换设置时开卡没开成、量代价出错照 [`core_failure_of_the_gpu`]，参与的卡一张都不剩照
-/// [`core_failure_of_every_participating_gpu_left_out`]。三段交回的与挂载那一刻核心层读到的都照它报。
+/// 此刻这组设置下一批分不到卡上，报给核心层：切换点到的卡重开到上限都没开成照 [`core_failure_of_the_gpu_not_opened_after_the_reopenings`]，
+/// 换设置时量代价出错照 [`core_failure_of_the_gpu`]，参与的卡一张都不剩照 [`core_failure_of_every_participating_gpu_left_out`]。
+/// 三段交回的与挂载那一刻核心层读到的都照它报。
 #[must_use]
 pub fn core_failure_of_the_batch_not_split_over_the_gpus(
     not_split: &BatchNotSplitOverTheGpus,
 ) -> BackgroundGpuComputeFailed {
     match not_split {
-        BatchNotSplitOverTheGpus::GpuNotOpenedAtTheSwitch { device, cause }
-        | BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause } => {
+        BatchNotSplitOverTheGpus::GpuNotOpenedAfterTheReopeningsAtTheSwitch(not_opened) => {
+            core_failure_of_the_gpu_not_opened_after_the_reopenings(not_opened)
+        }
+        BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause } => {
             core_failure_of_the_gpu(device, cause)
         }
         BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out) => {
@@ -396,6 +428,19 @@ pub fn core_failure_of_the_batch_not_split_over_the_gpus(
     }
 }
 
+/// 切换点到的一张卡重开到上限都没开成报给核心层：`GpuNotOpenedAfterTheReopenings`，带这张卡与每一次开不成照 [`core_failure_of_the_gpu`] 报成的出错，按开的次序。
+#[must_use]
+pub fn core_failure_of_the_gpu_not_opened_after_the_reopenings(
+    not_opened: &GpuNotOpenedAfterTheReopenings,
+) -> BackgroundGpuComputeFailed {
+    core_failure_of_the_gpu(
+        &not_opened.device,
+        &GpuFailureCause::NotOpenedAfterTheReopenings {
+            causes_of_each_opening: not_opened.causes_of_each_opening.clone(),
+        },
+    )
+}
+
 /// 一张参与的卡连这一批最小的一片都装不下报给核心层：`InputNotSupportedOnTheGpu` 的「一条装不进一段」，`input_bytes` 是最小的一片合占的字节数
 /// （这张卡至少要剩这么多）。
 #[must_use]
@@ -521,10 +566,6 @@ impl<Kernels: GpuInventory> GpuInventory for BackgroundGpuComputeThroughBothKern
     fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
         self.kernels.devices_with_total_memory()
     }
-
-    fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
-        self.kernels.devices_not_opened_at_the_mount()
-    }
 }
 
 impl<Kernels: BothKernelsOfTheThreeSegments> BackgroundGpuCompute
@@ -619,6 +660,16 @@ impl<Kernels: BothKernelsOfTheThreeSegments> BackgroundGpuCompute
         self.kernels.use_the_settings_from_the_next_batch(settings);
     }
 
+    /// 转给两份核那一层（[`BothKernelsOfTheThreeSegments::put_the_gpus_opened_since_the_switch_in_effect`]），重开到上限都没开成的那张照
+    /// [`core_failure_of_the_gpu_not_opened_after_the_reopenings`] 报；三段的连续不确定计数不动（新开的卡并进这一份实现，不整份换掉）。
+    fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
+        &mut self,
+    ) -> Option<BackgroundGpuComputeFailed> {
+        self.kernels
+            .put_the_gpus_opened_since_the_switch_in_effect()
+            .map(|not_opened| core_failure_of_the_gpu_not_opened_after_the_reopenings(&not_opened))
+    }
+
     /// 问两份核那一层此刻这组设置下一批分不分得到卡上，分不到照 [`core_failure_of_the_batch_not_split_over_the_gpus`] 报。
     fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
         self.kernels
@@ -726,6 +777,30 @@ mod tests {
                     reason: "假的：显存不足".to_string(),
                 }),
             ),
+            (
+                GpuFailureCause::NotOpenedAfterTheReopenings {
+                    causes_of_each_opening: vec![
+                        GpuFailureCause::DeviceOrPipelineDidNotStart {
+                            reason: "假的：头一次起不了设备".to_string(),
+                        },
+                        GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit },
+                    ],
+                },
+                BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings {
+                    device: card_named_in_the_failures(),
+                    failures_of_each_opening: vec![
+                        failed_on_the_card(GpuFailureOnTheDevice::DeviceOrPipelineDidNotStart {
+                            reason: "假的：头一次起不了设备".to_string(),
+                        }),
+                        BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit {
+                            device: card_named_in_the_failures(),
+                            wait_limit,
+                            submission_waited_for:
+                                SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
+                        },
+                    ],
+                },
+            ),
         ]
     }
 
````

### crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs b/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs
index 729e4ad..ebebb22 100644
--- a/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs
+++ b/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs
@@ -67,10 +67,10 @@ use singlefs_core::background_compute::{
     admit_gpu_settings_against_the_inventory, AdmittedGpuSettings, BackgroundComputeBackend,
     BackgroundComputeOfOneBatch, BackgroundComputePathTaken, BackgroundComputeSettings,
     BackgroundGpuCompute, BackgroundGpuComputeFailed, BackgroundGpuComputeHandle,
-    ClusterSegmentEntryKey, GpuDeviceIdentifier, GpuDeviceNotOpenedAtTheMount, GpuEntryDecision,
-    GpuInventory, GpuMemoryInBytes, GpuSubmissionsOfOneSegment, GpuSubmissionsOfTheBatch,
-    ReasonTheBatchRanOnTheCpu, RegionRecordToClassifyOnTheGpu, UnitChecksumOutcome,
-    UnitChecksumToVerifyOnTheGpu, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+    ClusterSegmentEntryKey, GpuDeviceIdentifier, GpuEntryDecision, GpuInventory, GpuMemoryInBytes,
+    GpuSubmissionsOfOneSegment, GpuSubmissionsOfTheBatch, ReasonTheBatchRanOnTheCpu,
+    RegionRecordToClassifyOnTheGpu, UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
+    BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
 };
 use singlefs_core::block_device::{
     BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
@@ -2960,10 +2960,6 @@ impl<Inner: GpuInventory> GpuInventory for GpuComputeRejudgedOnTheCpu<Inner> {
     fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
         self.inner.devices_with_total_memory()
     }
-
-    fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
-        self.inner.devices_not_opened_at_the_mount()
-    }
 }
 
 impl<Inner: BackgroundGpuCompute> BackgroundGpuCompute for GpuComputeRejudgedOnTheCpu<Inner> {
@@ -3028,6 +3024,13 @@ impl<Inner: BackgroundGpuCompute> BackgroundGpuCompute for GpuComputeRejudgedOnT
         self.inner.use_the_settings_from_the_next_batch(settings);
     }
 
+    fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
+        &mut self,
+    ) -> Option<BackgroundGpuComputeFailed> {
+        self.inner
+            .put_the_gpus_opened_since_the_settings_were_handed_in_in_effect()
+    }
+
     fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
         self.inner.failure_under_the_settings_handed_in()
     }
@@ -7740,10 +7743,6 @@ impl<Inner: GpuInventory> GpuInventory for GpuComputeTimingTheStageThree<Inner>
     fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
         self.inner.devices_with_total_memory()
     }
-
-    fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
-        self.inner.devices_not_opened_at_the_mount()
-    }
 }
 
 impl<Inner: BackgroundGpuCompute> BackgroundGpuCompute for GpuComputeTimingTheStageThree<Inner> {
@@ -7790,6 +7789,13 @@ impl<Inner: BackgroundGpuCompute> BackgroundGpuCompute for GpuComputeTimingTheSt
         self.inner.use_the_settings_from_the_next_batch(settings);
     }
 
+    fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
+        &mut self,
+    ) -> Option<BackgroundGpuComputeFailed> {
+        self.inner
+            .put_the_gpus_opened_since_the_settings_were_handed_in_in_effect()
+    }
+
     fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
         self.inner.failure_under_the_settings_handed_in()
     }
````

### crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs b/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
index ab4c284..3cc9d49 100644
--- a/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
+++ b/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
@@ -3,8 +3,9 @@
 //! 泄漏账（这个进程里之前几次挂载泄漏在一张卡上的字节，[`GpuMemoryLeakedInThisProcess`]）、建过几个实例的计数。
 //!
 //! 开卡（枚举适配器在内：wgpu 在造 future 那一步同步做完枚举，等待上限管不到它）放工作线程，调用线程的期限分两步（[`OpeningDeadlinesOfTheOpener`]）：
-//! 建实例、枚举、判卡清单与校验那一步至多等枚举期限；工作线程枚举完报回要开几张卡，期限延到「枚举期限 + 卡数 × 单卡上限」
-//! （开卡按卡串行、只开参与的卡，卡数是参与的卡数；每张卡自己至多等单卡上限，到点当那张卡没开成，由 `waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit` 管）。
+//! 建实例、枚举、判卡清单与校验那一步至多等枚举期限；工作线程枚举完报回要开几张卡，期限延到「枚举期限 + 卡数 × 一张卡的期限」
+//! （开卡按卡串行、只开参与的卡，卡数是参与的卡数；每一次开卡至多等单卡上限，到点当那一次没开成，由 `waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit` 管；
+//! 开不成隔 1 分钟重开、至多重开 3 次，所以一张卡的期限是 4 个单卡上限加 3 个间隔，`waiting_for_the_gpu::ReopeningsOfAGpuNotOpened::longest_wait_of_one_gpu`）。
 //! 到点挂载拒绝（[`BackgroundGpuNotOpened::TimedOut`] 带到点那一刻的期限），工作线程留在驱动里不杀（没有安全的杀法）；它回来之前闸住新开卡，不再多起一条枚举线程
 //! （[`BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung`]；同一个驱动加载器里再起一条会不会也挂没量过）。它回来之后闸放开、开出来的东西没人要：
 //! 开成的把手照卸载丢（还记着等超过上限的提交的卡泄漏、记进泄漏账；开成时量代价的提交都已做完，没有这种卡），开卡途中量代价出错而提交还在跑的卡
@@ -30,13 +31,15 @@ use crate::background_gpu::{
 };
 use crate::background_gpu_compute::background_gpu_compute_of_the_opened_gpu;
 use crate::gpus_of_this_host::GpuMemoryLeakedInThisProcess;
-use crate::waiting_for_the_gpu::WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT;
+use crate::waiting_for_the_gpu::{
+    ReopeningsOfAGpuNotOpened, WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
+};
 
 /// 一个进程至多新建几个 Vulkan 实例（数是拍的：比第二台那一套驱动上量到的约 12 个低得多，那个数只对那一套驱动成立）。
 pub const VULKAN_INSTANCES_OF_THIS_PROCESS_AT_MOST: u32 = 4;
 
 /// 调用线程等一次开卡的期限（从开卡起算）：枚举那一步（建实例、枚举适配器、判卡清单与校验）至多等 `enumeration`；工作线程枚举完报回要开 N 张卡，
-/// 期限延到 `enumeration + N × each_gpu`。`each_gpu` 与开每张卡那一层的单卡上限是同一个数（产品的 [`Self::OF_THE_PRODUCT`]）；单测换上短的。
+/// 期限延到 `enumeration + N × each_gpu`。`each_gpu` 是一张卡连同重开至多等多久（产品的 [`Self::OF_THE_PRODUCT`]）；单测换上短的。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct OpeningDeadlinesOfTheOpener {
     pub enumeration: Duration,
@@ -44,10 +47,12 @@ pub struct OpeningDeadlinesOfTheOpener {
 }
 
 impl OpeningDeadlinesOfTheOpener {
-    /// 产品的：枚举期限与每张卡的期限都取单卡上限 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]。
+    /// 产品的：枚举期限取单卡上限 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]；每张卡的期限是它连同产品的重开（[`ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT`]）
+    /// 至多等的那么久：每一次开卡至多单卡上限，开 1 + 重开次数 次，中间隔重开次数个间隔。
     pub const OF_THE_PRODUCT: Self = Self {
         enumeration: WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
-        each_gpu: WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
+        each_gpu: ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT
+            .longest_wait_of_one_gpu(WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT),
     };
 
     /// 工作线程枚举完报回要开 `gpus_to_open` 张卡之后的期限（从开卡起算）：枚举期限加卡数乘每张卡的期限，超出 `Duration` 取它的最大值。
````

### crates/singlefs-background-gpu/src/gpus_of_this_host.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/gpus_of_this_host.rs b/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
index 0bf1438..504979b 100644
--- a/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
+++ b/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
@@ -1,12 +1,14 @@
 //! 开着的本机独显里，此刻照哪一组设置用哪几张、每张怎么切、等多久（D26（后台整理与放置回收） 已定项 13 第 1–3 条）。
 //! 只开参与的卡（欠账 C996 的定案）：挂载那一刻只在参与的卡上起设备、量代价（给了 `background_gpu_devices` 是点名的那几张，没给是本机全部独显），
-//! 没点名的卡不开、不量，卡清单照列它（[`DiscreteGpusAfterTheOpening::not_opened_for_not_taking_part`]）；挂着切换点到它，切换生效那一刻开它
-//! （外壳 `crate::background_gpu::BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`）。开着的卡切换之后不再参与的照开着、不关：
-//! 它上面可能还有等超过上限、没在 GPU 上做完的提交（关它要等那次提交，第 4 条不等），再点它不用重开。
+//! 没点名的卡不开、不量，卡清单照列它（[`DiscreteGpusAfterTheOpening::not_opened_for_not_taking_part`]）；挂着切换点到它，外壳在会话锁外开它、开成了再换上那一组
+//! （外壳 `crate::background_gpu::BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`）。换上一组时不再点名的开着的卡关掉（用户定案「切换时关掉」）：
+//! 挪回没开的那几张（[`DiscreteGpusAfterTheOpening::note_the_gpu_closed_at_a_switch`]），之后再点它就重开；上面还有等超过上限、没在 GPU 上做完的提交的那张
+//! 不等它、整张泄漏，同卸载（[`gpus_dropped_leaking_those_still_running`]）。
 //! 纯逻辑，不碰 GPU：外壳每张开着的卡一份 [`GpuBudgetUnderTheSettings`]，参与的卡由 [`ParticipatingGpusOnThisHost`] 定；
 //! 核心层交来一组新设置（`BackgroundGpuCompute::use_the_settings_from_the_next_batch`，只在两批之间）就换这两样，从下一批起照它分活、切段、等。
 //!
-//! 挂载那一刻参与的卡没开成就拒绝挂载（第 4 条）：[`gpus_opened_at_the_mount`]。
+//! 挂载那一刻参与的卡开不成就隔一段时间重开，重开到上限都没开成才拒绝挂载（第 4 条与用户定案「挂载那一刻点名的卡开不成：也照这样重开 3 次」）：
+//! [`gpus_opened_at_the_mount`]。
 //!
 //! 一批的活按卡的次序填满（用户 2026-11-02 定「按卡的顺序填满」）：参与的卡排成参与的次序，前面的卡先填到它照显存上限切片之后一次能放下的量，
 //! 装不下的才给后面的卡，排在最后的那张拿剩下的全部（[`plans_of_a_batch_filling_the_gpus_in_order`]）；挂载时量的代价照量、不用来分活。
@@ -42,7 +44,10 @@ use crate::memory_limit_and_chunks::{
     GpuMemoryAccount, PieceOfAnEntry, SmallestPieceLargerThanTheRoom,
 };
 use crate::region_record_kernels::bytes_of_the_smallest_region_record_piece;
-use crate::waiting_for_the_gpu::{CompletionCheck, GpuWaitDeadline, MonotonicClock};
+use crate::waiting_for_the_gpu::{
+    gpu_opened_with_the_reopenings, CompletionCheck, GpuWaitDeadline, MonotonicClock,
+    ReopeningsOfAGpuNotOpened,
+};
 
 /// 一张卡此刻照的设置管的那几样：显存上限（记在账上）、等 GPU 的上限，连同与设置无关的绑定上限与派发上限、本包在它上面的账。
 #[derive(Debug)]
@@ -216,27 +221,23 @@ pub struct ParticipatingGpusOnThisHost {
 }
 
 impl ParticipatingGpusOnThisHost {
-    /// 核心层交来的那一组里参与的卡，在 `identifiers_on_this_host`（开着的本机独显，与外壳开着的卡同序）里找到各自的位置；
-    /// `devices_not_opened_at_the_switch`（切换生效那一刻要开而没开成的）不在里面、跳过，可以一张都不剩。
+    /// 核心层交来的那一组里参与的卡，在 `identifiers_on_this_host`（开着的本机独显，与外壳开着的卡同序）里找到各自的位置。
     ///
     /// # Panics
-    /// 有一张参与的卡既不在开着的独显里、也不是切换那一刻没开成的：核心层照这份实现答的卡清单（开着的与没开的本机独显）校验过这一组，
-    /// 外壳在交这一组之前把点到的没开的卡都开过一遍。
+    /// 有一张参与的卡不在开着的独显里：外壳只在这一组点的卡都开着时换上它（点到没开的卡就先开，开完了才换上）。
     #[must_use]
     pub fn of_the_settings(
         settings: &AdmittedGpuSettings,
         identifiers_on_this_host: &[GpuDeviceIdentifier],
-        devices_not_opened_at_the_switch: &[GpuDeviceIdentifier],
     ) -> Self {
         let positions_on_this_host: Vec<usize> = settings
             .participating_devices()
             .iter()
-            .filter(|participating| !devices_not_opened_at_the_switch.contains(&participating.device))
             .map(|participating| {
                 identifiers_on_this_host
                     .iter()
                     .position(|on_this_host| *on_this_host == participating.device)
-                    .expect("核心层照卡清单校验过、外壳开过点到的没开的卡：参与的卡都开着，除了切换那一刻没开成的")
+                    .expect("外壳只在这一组点的卡都开着时换上它：参与的卡都开着")
             })
             .collect();
         Self {
@@ -294,10 +295,10 @@ impl DiscreteGpusAfterTheOpening {
         }
     }
 
-    /// 切换生效那一刻开成了一张原先没开的卡：它从没开的那几张挪进开成的那几张（排在开成的那几张末尾，与外壳开着的卡同序）。
+    /// 为切换开成了一张原先没开的卡：它从没开的那几张挪进开成的那几张（排在开成的那几张末尾，与外壳开着的卡同序）。
     ///
     /// # Panics
-    /// 它不在没开的那几张里：外壳只开没开的卡。
+    /// 它不在没开的那几张里：外壳只开没开的卡，开着的时候它一直在没开的那几张里（开卡至多一次在开，关卡只关开着的）。
     pub fn note_the_gpu_opened_at_a_switch(&mut self, identifier: &GpuDeviceIdentifier) {
         let position = self
             .not_opened_for_not_taking_part
@@ -307,11 +308,31 @@ impl DiscreteGpusAfterTheOpening {
         let opened_at_the_switch = self.not_opened_for_not_taking_part.remove(position);
         self.opened.push(opened_at_the_switch);
     }
+
+    /// 切换时关掉了一张开着的卡：它从开着的那几张挪回没开的那几张（按总线地址插进去），开着的那几张别的次序不变。
+    ///
+    /// # Panics
+    /// 它不在开着的那几张里：外壳只关开着的卡。
+    pub fn note_the_gpu_closed_at_a_switch(&mut self, identifier: &GpuDeviceIdentifier) {
+        let position = self
+            .opened
+            .iter()
+            .position(|discrete_gpu| discrete_gpu.identifier == *identifier)
+            .expect("外壳只关开着的卡");
+        let closed_at_the_switch = self.opened.remove(position);
+        let position_by_the_bus_address =
+            self.not_opened_for_not_taking_part
+                .partition_point(|not_opened| {
+                    not_opened.pci_bus_address < closed_at_the_switch.pci_bus_address
+                });
+        self.not_opened_for_not_taking_part
+            .insert(position_by_the_bus_address, closed_at_the_switch);
+    }
 }
 
 /// 开卡之后的本机独显：开着的那几张（挂载那一刻开的按总线地址排，之后切换开的依次排在末尾，与外壳开着的卡一一对应、同序），
-/// 与没参与而没开的那几张（按总线地址排）。卡清单（核心层 `GpuInventory`）照它答：两样合起来按总线地址排（没给 `background_gpu_devices` 时
-/// 上限按这个次序一张一个，第 3 条）；没开的照列，切换可以点它。挂载那一刻没有「没开成而跳过」的卡：参与的卡没开成就拒绝挂载，不参与的卡不开。
+/// 与没参与而没开的那几张（挂载时没开的与切换时关掉的，按总线地址排）。卡清单（核心层 `GpuInventory`）照它答：两样合起来按总线地址排（没给 `background_gpu_devices` 时
+/// 上限按这个次序一张一个，第 3 条）；没开的照列，切换可以点它。没有「没开成而跳过」的卡：参与的卡重开到上限都没开成就拒绝挂载，不参与的卡不开。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct DiscreteGpusAfterTheOpening {
     pub opened: Vec<DiscreteGpuOnThisHost>,
@@ -817,11 +838,12 @@ pub fn gpu_opened_under_its_memory_limit<Opened>(
 /// 没给时本机全部独显都参与）就照开卡那一组给它的显存上限开（[`gpu_opened_under_its_memory_limit`]，`open_the_gpu` 是外壳起设备、编管线、量代价那一步，
 /// 测试里交造出来的结局）；不参与的不开、不量代价，记进 [`DiscreteGpusAfterTheOpening::not_opened_for_not_taking_part`]，卡清单照列它，挂着切换点到它时再开。
 /// 显存上限装不下最小的一片的参与的卡照开、不量代价、不进参与的卡、报出来（[`GpuLeftOutUnderItsMemoryLimit`]），挂载不因此拒绝。
-/// 一张参与的卡没开成就拒绝挂载、报出它与原因，后面的卡不再开（D26（后台整理与放置回收） 已定项 13 第 4 条）。
+/// 一张参与的卡开不成就照 `reopenings` 重开（用户定案「挂载那一刻点名的卡开不成：也照这样重开 3 次，都不成才拒绝挂载」，每次重开之前把间隔交给
+/// `wait_before_a_reopening` 等，[`gpu_opened_with_the_reopenings`]）；重开到上限都没开成就拒绝挂载、报出它与每一次的原因，后面的卡不再开。
 /// `participating` 是开卡那一组照同一份 `discrete_gpus` 校验过、取出来的参与的卡（`adapter_inventory::participating_gpus_admitted_by_the_mount_checks`）。
 ///
 /// # Errors
-/// [`BackgroundGpuOpenRefused::GpuUnusable`]：一张参与的卡没开成。
+/// [`BackgroundGpuOpenRefused::GpuUnusable`]：一张参与的卡重开到上限都没开成。
 ///
 /// # Panics
 /// `participating` 里有一张不在 `discrete_gpus` 里：它取自同一份清单。
@@ -830,6 +852,8 @@ pub fn gpus_opened_at_the_mount<Opened>(
     participating: &[ParticipatingGpu],
     smallest_piece_bytes: u64,
     memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
+    reopenings: ReopeningsOfAGpuNotOpened,
+    mut wait_before_a_reopening: impl FnMut(Duration),
     mut open_the_gpu: impl FnMut(
         &DiscreteGpuOnThisHost,
         Option<GpuMemoryInBytes>,
@@ -852,17 +876,22 @@ pub fn gpus_opened_at_the_mount<Opened>(
                 continue;
             }
         };
-        let (opened_gpu, left_out) = gpu_opened_under_its_memory_limit(
-            &discrete_gpu,
-            memory_limit_at_the_opening,
-            smallest_piece_bytes,
-            memory_leaked_in_this_process,
-            &mut open_the_gpu,
-        )
-        .map_err(|cause| BackgroundGpuOpenRefused::GpuUnusable {
-            device: discrete_gpu.identifier.clone(),
-            cause,
-        })?;
+        let (opened_gpu, left_out) =
+            gpu_opened_with_the_reopenings(reopenings, &mut wait_before_a_reopening, || {
+                gpu_opened_under_its_memory_limit(
+                    &discrete_gpu,
+                    memory_limit_at_the_opening,
+                    smallest_piece_bytes,
+                    memory_leaked_in_this_process,
+                    &mut open_the_gpu,
+                )
+            })
+            .map_err(|causes_of_each_opening| {
+                BackgroundGpuOpenRefused::GpuUnusable {
+                    device: discrete_gpu.identifier.clone(),
+                    causes_of_each_opening,
+                }
+            })?;
         opened.push(opened_gpu);
         opened_discrete_gpus.push(discrete_gpu);
         left_out_under_their_memory_limit.extend(left_out);
@@ -958,6 +987,13 @@ mod tests {
     use crate::background_gpu::COST_PROBE_CHUNK;
     use crate::memory_limit_and_chunks::SubmissionOfTheChunk;
 
+    /// 用例里开卡开不成照产品那样至多重开 3 次，间隔为 0：不真等 1 分钟。
+    const REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS: ReopeningsOfAGpuNotOpened =
+        ReopeningsOfAGpuNotOpened {
+            interval: Duration::ZERO,
+            reopenings_at_most: 3,
+        };
+
     /// 本机三张独显，按总线地址排：代价各不相同。
     struct ThreeCardsOnThisHost;
 
@@ -1007,7 +1043,6 @@ mod tests {
         let at_the_mount = ParticipatingGpusOnThisHost::of_the_settings(
             &admitted(None, None, Duration::from_secs(10)),
             &identifiers_on_this_host(),
-            &[],
         );
         assert_eq!(
             at_the_mount.positions_on_this_host(),
@@ -1021,7 +1056,6 @@ mod tests {
                 Duration::from_secs(10),
             ),
             &identifiers_on_this_host(),
-            &[],
         );
         assert_eq!(
             after_the_switch.positions_on_this_host(),
@@ -1247,6 +1281,8 @@ mod tests {
             &participating,
             bytes_of_the_smallest_piece_every_participating_gpu_holds(),
             &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+            |_interval| {},
             |discrete_gpu, memory_limit, _cost_at_the_opening| {
                 if discrete_gpu.pci_bus_address == card_that_does_not_open {
                     Err(reason_the_card_did_not_open(card_that_does_not_open))
@@ -1276,6 +1312,8 @@ mod tests {
             &participating,
             bytes_of_the_smallest_piece_every_participating_gpu_holds(),
             &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+            |_interval| {},
             |discrete_gpu, _memory_limit, _cost_at_the_opening| {
                 cards_handed_to_the_opening.push(discrete_gpu.pci_bus_address.clone());
                 Ok::<_, GpuFailureCause>(discrete_gpu.pci_bus_address.clone())
@@ -1313,13 +1351,6 @@ mod tests {
             ],
             "卡清单照列本机全部独显，连同没开的两张，切换可以点它们"
         );
-        assert!(
-            opening
-                .discrete_gpus
-                .devices_not_opened_at_the_mount()
-                .is_empty(),
-            "没开的卡不是没开成：核心层不因它拒切换"
-        );
     }
 
     /// 第 2 条「不给就用本机全部独立显卡」：没给 `background_gpu_devices` 时三张都参与、都开。
@@ -1368,9 +1399,9 @@ mod tests {
             refused.map(|opening| opening.opened),
             Err(BackgroundGpuOpenRefused::GpuUnusable {
                 device: card("0000:41:00.0"),
-                cause: reason_the_card_did_not_open("0000:41:00.0"),
+                causes_of_each_opening: vec![reason_the_card_did_not_open("0000:41:00.0"); 4],
             }),
-            "点名的那张没开成：拒绝挂载"
+            "点名的那张没开成：拒绝挂载（头一次与 3 次重开都开不成，每一次的原因都报）"
         );
     }
 
@@ -1389,10 +1420,59 @@ mod tests {
             refused.map(|opening| opening.opened),
             Err(BackgroundGpuOpenRefused::GpuUnusable {
                 device: card("0000:81:00.0"),
-                cause: reason_the_card_did_not_open("0000:81:00.0"),
+                causes_of_each_opening: vec![reason_the_card_did_not_open("0000:81:00.0"); 4],
             }),
-            "没给卡列表：总线地址最后那张没开成也拒绝挂载"
+            "没给卡列表：总线地址最后那张没开成也拒绝挂载（头一次与 3 次重开都开不成，每一次的原因都报）"
+        );
+    }
+
+    /// 用户定案「挂载那一刻点名的卡开不成：也照这样重开」：点名的「0000:41:00.0」头两次开不成、第三次开成——挂载照常，它参与；
+    /// 两次重开之前各把间隔交去等一次（产品是 1 分钟，用例给 9 秒、不真等），开卡一共只开它 3 次、没点名的不开。
+    #[test]
+    fn the_named_card_not_opened_twice_at_the_mount_opens_the_third_time_and_takes_part() {
+        let selection = BackgroundGpuSelection {
+            background_gpu_devices: Some(vec![card("0000:41:00.0")]),
+            background_gpu_memory_limit: None,
+            background_gpu_wait_limit: Duration::from_secs(10),
+        };
+        let discrete_gpus = three_discrete_gpus_of_the_enumeration();
+        let participating =
+            participating_gpus_admitted_by_the_mount_checks(&selection, &discrete_gpus)
+                .expect("点名本机的一张卡：校验合法");
+        let mut cards_handed_to_the_opening: Vec<String> = Vec::new();
+        let mut intervals_waited: Vec<Duration> = Vec::new();
+        let opening = gpus_opened_at_the_mount(
+            discrete_gpus,
+            &participating,
+            bytes_of_the_smallest_piece_every_participating_gpu_holds(),
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            ReopeningsOfAGpuNotOpened {
+                interval: Duration::from_secs(9),
+                reopenings_at_most: 3,
+            },
+            |interval| intervals_waited.push(interval),
+            |discrete_gpu, _memory_limit, _cost_at_the_opening| {
+                cards_handed_to_the_opening.push(discrete_gpu.pci_bus_address.clone());
+                if cards_handed_to_the_opening.len() < 3 {
+                    Err(reason_the_card_did_not_open(&discrete_gpu.pci_bus_address))
+                } else {
+                    Ok(discrete_gpu.pci_bus_address.clone())
+                }
+            },
+        )
+        .expect("第三次开成：挂载照常");
+        assert_eq!(
+            cards_handed_to_the_opening,
+            vec!["0000:41:00.0".to_string(); 3],
+            "只开点名的那张，开成了就不再开"
+        );
+        assert_eq!(
+            intervals_waited,
+            vec![Duration::from_secs(9); 2],
+            "两次重开之前各等一个间隔"
         );
+        assert_eq!(opening.opened, vec!["0000:41:00.0".to_string()]);
+        assert_eq!(opening.participating.positions_on_this_host(), &[0]);
     }
 
     /// 开卡那一组点「0000:01:00.0」「0000:41:00.0」两张、各给 `memory_limits_in_bytes` 的显存上限；开卡那一步照外壳那样：量代价的卡拿给它的上限建账、
@@ -1419,6 +1499,8 @@ mod tests {
             &participating,
             bytes_of_the_smallest_piece_every_participating_gpu_holds(),
             &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+            |_interval| {},
             |discrete_gpu, memory_limit, cost_at_the_opening| {
                 if cost_at_the_opening == CostAtTheOpening::MeasuredUnderTheMemoryLimit {
                     let mut account = GpuMemoryAccount::<u32>::with_nothing_held(memory_limit);
@@ -1925,6 +2007,8 @@ mod tests {
             &participating,
             bytes_of_the_smallest_piece_every_participating_gpu_holds(),
             &memory_leaked,
+            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+            |_interval| {},
             |discrete_gpu, _memory_limit, cost_at_the_opening| {
                 Ok((discrete_gpu.pci_bus_address.clone(), cost_at_the_opening))
             },
````

### crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs b/crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs
index c0e4f2c..93f9873 100644
--- a/crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs
+++ b/crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs
@@ -634,7 +634,8 @@ fn failure_of_waiting_for_the_earlier_submission(cause: GpuFailureCause) -> GpuF
         | GpuFailureCause::ReadbackMappingFailed { .. }
         | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
         | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
-        | GpuFailureCause::UnusableAfterResubmissions { .. }) => reported_by_the_wait,
+        | GpuFailureCause::UnusableAfterResubmissions { .. }
+        | GpuFailureCause::NotOpenedAfterTheReopenings { .. }) => reported_by_the_wait,
     }
 }
 
````

### crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs b/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs
index 1c853c8..3f70e71 100644
--- a/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs
+++ b/crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs
@@ -8,6 +8,9 @@
 //! 这一批在这张卡上第 1 + [`RESUBMISSIONS_AFTER_A_TIMEOUT`] 次超时就交回 [`GpuFailureCause::UnusableAfterResubmissions`]（这张卡用不了，带最后一次的原因），这一批放弃、不换 CPU，
 //! 连续不确定的计数不动（`two_kernel_reconciliation::ConsecutiveUncertaintyCounts::reconcile_unless_the_batch_failed`）。别的出错不重交。
 //!
+//! 开卡（起设备、编管线、量代价）每一次至多等 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]（[`outcome_of_opening_a_gpu_within_the_limit`]）；
+//! 一张卡开不成就隔 1 分钟重开、至多重开 3 次，都不成才算它开不成（用户定案，挂载那一刻点名的卡与挂着切换点到的新卡同一种：[`gpu_opened_with_the_reopenings`]）。
+//!
 //! 等法与钟、完成信号拆开：钟是 [`MonotonicClock`]（产品用这个进程的单调钟 [`SteadyClockOfThisProcess`]），完成信号是调用方给的
 //! 「看一次做没做完、最多等这么久」的闭包。不要 GPU 的用例换上假钟与假完成信号，钉住「上限内完成照常、超过上限交回超时出错」。
 
@@ -127,7 +130,8 @@ fn waited_past_the_limit(cause: &GpuFailureCause) -> bool {
         | GpuFailureCause::ReadbackMappingFailed { .. }
         | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
         | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
-        | GpuFailureCause::UnusableAfterResubmissions { .. } => false,
+        | GpuFailureCause::UnusableAfterResubmissions { .. }
+        | GpuFailureCause::NotOpenedAfterTheReopenings { .. } => false,
     }
 }
 
@@ -196,7 +200,8 @@ fn the_attempt_reached_the_gpu<Outcome>(attempt: &Result<Outcome, GpuFailureCaus
             | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
             | GpuFailureCause::ReadbackMappingFailed { .. }
             | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
-            | GpuFailureCause::UnusableAfterResubmissions { .. },
+            | GpuFailureCause::UnusableAfterResubmissions { .. }
+            | GpuFailureCause::NotOpenedAfterTheReopenings { .. },
         ) => true,
     }
 }
@@ -310,6 +315,59 @@ pub fn outcome_of_opening_a_gpu_within_the_limit<Opened: Send + 'static>(
     }
 }
 
+/// 一张卡开不成之后隔多久重开（用户定案「间隔 1 分钟重开」，挂载那一刻点名的卡与挂着切换点到的新卡同一个数）。
+pub const INTERVAL_BEFORE_REOPENING_A_GPU_NOT_OPENED: Duration = Duration::from_secs(60);
+
+/// 一张卡开不成之后至多重开几次（用户定案「最多重开 3 次」）：头一次开不成之后再开这么多次，都不成才算这张卡开不成。
+pub const REOPENINGS_OF_A_GPU_NOT_OPENED_AT_MOST: u32 = 3;
+
+/// 一张卡开不成时怎么重开：头一次开不成就等 `interval` 再开，至多再开 `reopenings_at_most` 次，都不成才算这张卡开不成
+/// （[`gpu_opened_with_the_reopenings`]）。产品是 [`Self::OF_THE_PRODUCT`]；用例换上间隔为 0 的，不真等 1 分钟。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct ReopeningsOfAGpuNotOpened {
+    pub interval: Duration,
+    pub reopenings_at_most: u32,
+}
+
+impl ReopeningsOfAGpuNotOpened {
+    /// 产品的：间隔 [`INTERVAL_BEFORE_REOPENING_A_GPU_NOT_OPENED`]，至多重开 [`REOPENINGS_OF_A_GPU_NOT_OPENED_AT_MOST`] 次。
+    pub const OF_THE_PRODUCT: Self = Self {
+        interval: INTERVAL_BEFORE_REOPENING_A_GPU_NOT_OPENED,
+        reopenings_at_most: REOPENINGS_OF_A_GPU_NOT_OPENED_AT_MOST,
+    };
+
+    /// 连同重开，一张卡至多要等多久才有结局：每一次开卡至多 `each_opening`，开 1 + 重开次数 次，中间隔重开次数个间隔；超出 `Duration` 取它的最大值。
+    #[must_use]
+    pub const fn longest_wait_of_one_gpu(self, each_opening: Duration) -> Duration {
+        each_opening
+            .saturating_mul(self.reopenings_at_most.saturating_add(1))
+            .saturating_add(self.interval.saturating_mul(self.reopenings_at_most))
+    }
+}
+
+/// 开一张卡（`open_once` 开一次，交回开出来的或这一次开不成的原因），开不成就照 `reopenings` 重开：每次重开之前把间隔交给
+/// `wait_before_a_reopening` 等（产品是 `std::thread::sleep`），开成了就交回、不再开。
+///
+/// # Errors
+/// 开了 1 + `reopenings.reopenings_at_most` 次都没开成：每一次的原因，按开的次序（头一次的在前）。
+pub fn gpu_opened_with_the_reopenings<Opened>(
+    reopenings: ReopeningsOfAGpuNotOpened,
+    mut wait_before_a_reopening: impl FnMut(Duration),
+    mut open_once: impl FnMut() -> Result<Opened, GpuFailureCause>,
+) -> Result<Opened, Vec<GpuFailureCause>> {
+    let mut causes_of_each_opening = Vec::new();
+    for reopenings_so_far in 0..=reopenings.reopenings_at_most {
+        if reopenings_so_far > 0 {
+            wait_before_a_reopening(reopenings.interval);
+        }
+        match open_once() {
+            Ok(opened) => return Ok(opened),
+            Err(cause) => causes_of_each_opening.push(cause),
+        }
+    }
+    Err(causes_of_each_opening)
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -976,4 +1034,81 @@ mod tests {
         assert_eq!(gpu.outcome_with_resubmissions(), Err(device_lost));
         assert_eq!((gpu.submissions_made, gpu.waits_made), (1, 0));
     }
+
+    /// 用例里第 `opening` 次开卡（从 1 数）开不成的原因。
+    fn cause_of_the_opening_in_the_test(opening: u32) -> GpuFailureCause {
+        GpuFailureCause::DeviceOrPipelineDidNotStart {
+            reason: format!("用例造的：第 {opening} 次起不了设备"),
+        }
+    }
+
+    /// 照脚本开卡（`outcomes_in_order` 按次序交回，脚本用完就开不成），重开的规矩是间隔 7 秒、至多重开 3 次；交回结局、开了几次与每次重开之前交去等的间隔。
+    fn opening_with_the_reopenings_in_the_test(
+        outcomes_in_order: Vec<Result<u32, GpuFailureCause>>,
+    ) -> (Result<u32, Vec<GpuFailureCause>>, u32, Vec<Duration>) {
+        let mut outcomes = outcomes_in_order.into_iter();
+        let mut openings = 0_u32;
+        let mut waits = Vec::new();
+        let outcome = gpu_opened_with_the_reopenings(
+            ReopeningsOfAGpuNotOpened {
+                interval: Duration::from_secs(7),
+                reopenings_at_most: 3,
+            },
+            |interval| waits.push(interval),
+            || {
+                openings += 1;
+                outcomes
+                    .next()
+                    .unwrap_or_else(|| Err(cause_of_the_opening_in_the_test(openings)))
+            },
+        );
+        (outcome, openings, waits)
+    }
+
+    /// 开不成两次、第三次开成：交回第三次开出来的，开了 3 次，两次重开之前各等一个间隔。
+    #[test]
+    fn the_gpu_not_opened_twice_and_opened_the_third_time_is_handed_back_after_two_intervals() {
+        let (outcome, openings, waits) = opening_with_the_reopenings_in_the_test(vec![
+            Err(cause_of_the_opening_in_the_test(1)),
+            Err(cause_of_the_opening_in_the_test(2)),
+            Ok(42),
+        ]);
+        assert_eq!(outcome, Ok(42), "第三次开成：照常交回");
+        assert_eq!(openings, 3, "开成了就不再开");
+        assert_eq!(
+            waits,
+            vec![Duration::from_secs(7); 2],
+            "两次重开之前各等一个间隔"
+        );
+    }
+
+    /// 头一次与 3 次重开都开不成：交回每一次的原因（按开的次序），一共开 4 次、等 3 个间隔，之后不再开。
+    #[test]
+    fn the_gpu_never_opened_is_reopened_three_times_and_every_cause_is_handed_back_in_order() {
+        let (outcome, openings, waits) = opening_with_the_reopenings_in_the_test(Vec::new());
+        assert_eq!(
+            outcome,
+            Err((1..=4).map(cause_of_the_opening_in_the_test).collect()),
+            "每一次开不成的原因都交回，头一次的在前"
+        );
+        assert_eq!(openings, 4, "头一次加 3 次重开");
+        assert_eq!(waits, vec![Duration::from_secs(7); 3]);
+    }
+
+    /// 产品的重开：间隔 1 分钟、至多重开 3 次；连同重开一张卡至多等 4 次开卡上限加 3 个间隔（5 分钟的上限下是 23 分钟）。
+    #[test]
+    fn the_product_reopens_a_gpu_not_opened_three_times_one_minute_apart() {
+        assert_eq!(
+            ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT,
+            ReopeningsOfAGpuNotOpened {
+                interval: Duration::from_secs(60),
+                reopenings_at_most: 3,
+            }
+        );
+        assert_eq!(
+            ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT
+                .longest_wait_of_one_gpu(WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT),
+            Duration::from_secs(60) * 23
+        );
+    }
 }
````

### crates/singlefs-core/src/background_compute.rs

````diff
diff --git a/crates/singlefs-core/src/background_compute.rs b/crates/singlefs-core/src/background_compute.rs
index 940a0bc..4de2cf9 100644
--- a/crates/singlefs-core/src/background_compute.rs
+++ b/crates/singlefs-core/src/background_compute.rs
@@ -71,30 +71,12 @@ impl GpuDeviceIdentifier {
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
 pub struct GpuMemoryInBytes(pub u64);
 
-/// 本机有哪些显卡、每张多少显存，其中哪几张挂载那一刻没开成：只读。GPU 那一侧（住哪是七·2 的岔路）实现它，测试里一份假清单实现它。
+/// 本机有哪些显卡、每张多少显存：只读。GPU 那一侧（住哪是七·2 的岔路）实现它，测试里一份假清单实现它。
 /// 挂载与切换的校验只读它；没有这一份清单（`None`）就是 GPU 不在。
 pub trait GpuInventory {
     /// 本机全部显卡与各自的显存总量，按「本机全部显卡的列表」的次序：没给 `background_gpu_devices` 时，`background_gpu_memory_limit`
-    /// 按这个次序一张卡一个（第 3 条）。怎么排由实现这份清单的一方定（七·2 的岔路）。同一张卡只出现一次。挂载那一刻没开成的卡照列。
+    /// 按这个次序一张卡一个（第 3 条）。怎么排由实现这份清单的一方定（七·2 的岔路）。同一张卡只出现一次。没开着的卡（不参与而没开的）照列。
     fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)>;
-
-    /// [`Self::devices_with_total_memory`] 里挂载那一刻没开成的那几张与原因（
-    /// 给了 `background_gpu_devices` 而没点名的卡开不了，挂载跳过它、照挂）。参与的卡里有它就拒（第 4 条，挂载与切换同一判：
-    /// [`BackgroundComputeSettingsRefused::ParticipatingGpuDeviceNotOpenedAtTheMount`]）。
-    /// 不开卡的清单（测试里的假清单、开卡之前那一刻的枚举）没有这一样，交回空。
-    /// 挂载那一刻定下、之后不变：答它不现查本机此刻在的卡（一次校验里只有 [`Self::devices_with_total_memory`] 现查一次，判与取参与的卡读的是同一份
-    /// ），所以可以列着此刻已不在 [`Self::devices_with_total_memory`] 里的卡（挂载之后掉了的）；
-    /// 校验只拿参与的卡去比，掉了的卡不参与。
-    fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
-        Vec::new()
-    }
-}
-
-/// 卡清单里挂载那一刻没开成的一张卡（[`GpuInventory::devices_not_opened_at_the_mount`]）：原因同挂着时这张卡出错的报法。
-#[derive(Clone, Debug, PartialEq, Eq)]
-pub struct GpuDeviceNotOpenedAtTheMount {
-    pub device: GpuDeviceIdentifier,
-    pub failure_at_the_mount: BackgroundGpuComputeFailed,
 }
 
 /// GC 搬迁放上 GPU 的三段（第 1 条）。封闭集合。
@@ -199,6 +181,14 @@ pub enum BackgroundGpuComputeFailed {
     },
     /// 这一批的输入 GPU 那一侧第一版不支持，一张卡都没交活：切回 `cpu`（装不进一段的那一种也可以调大显存上限）。
     InputNotSupportedOnTheGpu(InputNotSupportedOnTheGpu),
+    /// 挂着切换点到的这张卡开不成（起不了设备、编不了管线、量代价出错或超时），隔一段时间重开到上限也都没开成（用户定案「开不成就间隔 1 分钟重开，
+    /// 最多重开 3 次……3 次都不成：GC 停下，报警并报出哪张卡、每次开不成的原因，等人处理，不自动再试」）：修好它或切换时不再点它，或切回 `cpu`。
+    /// `failures_of_each_opening` 是每一次开不成照这张卡出错报成的那一个（[`Self::GpuFailed`] 或 [`Self::WaitForTheGpuExceededTheLimit`]），
+    /// 按开的次序，头一次的在前。
+    GpuNotOpenedAfterTheReopenings {
+        device: GpuDeviceIdentifier,
+        failures_of_each_opening: Vec<BackgroundGpuComputeFailed>,
+    },
     /// 这张卡上一次提交等超过 `wait_limit` 之后，在同一张卡上重交了 `resubmissions` 次、次次等超过上限（「在间隔内重试三次，
     /// 如果gpu无法用就报警」「重试是把那一批重新交一次」）：这张卡用不了。修好它或切换时不再点它，或切回 `cpu`。
     /// `last_submission_waited_for` 是最后那一次等超过上限的是哪一次提交（重交之前先等前一次，那一次没等到也算一次）。
@@ -238,6 +228,7 @@ impl BackgroundGpuComputeFailed {
         match self {
             BackgroundGpuComputeFailed::GpuFailed { device, .. }
             | BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit { device, .. }
+            | BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings { device, .. }
             | BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions { device, .. }
             | BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
                 InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice { device, .. },
@@ -385,10 +376,22 @@ pub trait BackgroundGpuCompute: GpuInventory {
 
     /// 从下一批起照这一组设置跑（第 1–3 条）：只在 `settings` 点的那几张卡上、按卡的顺序填满分活，每张卡上合占的显存不超过
     /// 给它的上限，每一处等 GPU 不超过等的上限。核心层在挂载那一刻（选 `gpu` 时）与每次通过校验的切换生效的那一批开头、开算之前调，
-    /// 一批里三段都算完之前不再调（一批之内不换）。实现不为它重开设备，三段的连续不确定计数照旧留着（第 8 条：只在重挂时清零）。
-    /// `settings` 的卡都在这份实现答的卡清单里、都不是它报的挂载那一刻没开成的（核心层照这份清单校验过）。
+    /// 一批里三段都算完之前不再调（一批之内不换）。开着的卡实现不为它重开设备，三段的连续不确定计数照旧留着（第 8 条：只在重挂时清零）。
+    /// `settings` 点到没开着的卡时，实现可以不在这一刻换上：在会话锁外开卡、开完了在之后的两批之间换上
+    /// （[`Self::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]），开着的这段时间照旧照交来之前那一组跑。
+    /// `settings` 的卡都在这份实现答的卡清单里（核心层照这份清单校验过）。
     fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings);
 
+    /// 两批之间（每一批开头与整理两个入口判停没停之前，`BackgroundComputeOfThisMount::put_the_switch_requested_from_the_next_batch_in_effect` 里）：
+    /// 最近交来的那一组点到没开着的卡、实现在会话锁外开着的，开完了就在这一刻换上，交回 `None`；重开到上限都没开成就交回那张卡与每一次开不成的出错
+    /// （[`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]，只交这一次），核心层把有 GPU 路径的后台任务当场记成出错停着、报警，
+    /// 不自动再试，直到下一次通过校验的切换。还在开、没在开的交回 `None`，不等。不开卡的实现（测试里的假实现）没有这一样，交回 `None`。
+    fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
+        &mut self,
+    ) -> Option<BackgroundGpuComputeFailed> {
+        None
+    }
+
     /// 照最近一次交来的那一组设置（[`Self::use_the_settings_from_the_next_batch`]）一张卡都用不上时，那一组下每一批三段都会交回的那个出错
     /// （例如参与的卡的显存上限连最小的一片都装不下、一张都不剩）；用得上交回 `None`。核心层在挂载那一刻与每次切换生效的那一批开头把那一组交给实现之后
     /// 各读一次：不是 `None` 就不拒挂载、不拒切换，有 GPU 路径的后台任务当场记成出错停着
@@ -415,7 +418,7 @@ pub struct ParticipatingGpuDevice {
 
 /// 校验过、选 `gpu` 的一组设置交给实现的样子（[`BackgroundGpuCompute::use_the_settings_from_the_next_batch`]）：参与的卡按上限对应的次序
 /// （没点名是卡清单的次序，点了名是点名的次序）、各自的显存上限，与等 GPU 的上限。只由 [`admit_gpu_settings_against_the_inventory`] 造出：
-/// 至少一张卡、同一张卡只出现一次、每张都在校验时的卡清单里且不是清单报的挂载那一刻没开成的、上限不超过那张卡的显存总量、等的上限不是 0。
+/// 至少一张卡、同一张卡只出现一次、每张都在校验时的卡清单里、上限不超过那张卡的显存总量、等的上限不是 0。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct AdmittedGpuSettings {
     participating_devices: Vec<ParticipatingGpuDevice>,
@@ -741,7 +744,8 @@ impl GcCpuCompensationReading {
 
 /// 一组设置在挂载或切换那一刻被拒（第 4 条：拒，不降级到 cpu）。成员按调用方要改哪一样分：第 4 条点名的四种配错
 /// （GPU 不在、点名的卡不存在、上限个数与卡数不等、上限超过显存总量），条款没写、第一版不支持的两种点法（空卡列表、同一张卡点两次），
-/// 等 GPU 结果的上限给了 0，与参与的卡里有一张挂载那一刻没开成；GC 在 CPU 上补跑的阈值超过 100%。
+/// 等 GPU 结果的上限给了 0；GC 在 CPU 上补跑的阈值超过 100%。切换点到的卡开不开得成不在这里判：切换照收，开不成照
+/// [`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`] 停 GC、报警。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum BackgroundComputeSettingsRefused {
     /// 选了 `gpu` 而 GPU 不在：没有卡清单，或没点名卡而清单里一张卡都没有。改选 `cpu`，或装上 GPU。
@@ -765,9 +769,6 @@ pub enum BackgroundComputeSettingsRefused {
     GpuDeviceNamedMoreThanOnceWhoseMeaningIsUndecided { device: GpuDeviceIdentifier },
     /// 选了 `gpu` 而 `background_gpu_wait_limit` 是 0：一批交出去就算超时，一批都算不成。改成大于 0 的上限。
     GpuWaitLimitOfZero,
-    /// 参与的卡里有一张挂载那一刻没开成（[`GpuInventory::devices_not_opened_at_the_mount`]）：给了卡列表时它被点了名，没给时本机全部显卡都参与。
-    /// 第 4 条拒（同「GPU 不在、驱动不对」）：切换时不点它，或切回 `cpu`，或修好它重挂。
-    ParticipatingGpuDeviceNotOpenedAtTheMount(GpuDeviceNotOpenedAtTheMount),
     /// `background_gc_cpu_compensation_threshold` 超过 100%：已分配不会超过总容量，这个阈值永远越不过。改成 0 到 100 之间。
     GcCpuCompensationThresholdAboveTheWholeCapacity {
         threshold: BackgroundGcCpuCompensationThreshold,
@@ -844,6 +845,22 @@ impl AdmittedBackgroundComputeSettings {
             .failure_under_the_settings_handed_in()
     }
 
+    /// 选 `gpu` 时让它带着的实现把交设置以来在会话锁外开完的卡换上（[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]），
+    /// 交回重开到上限都没开成的那一个出错；选 `cpu` 时 `None`（实现这时不算活）。
+    ///
+    /// # Panics
+    /// 选 `gpu` 而没带实现：选 `gpu` 的那一组在校验那一刻就带上实现。
+    fn failure_of_the_gpus_opened_since_the_gpu_settings_were_handed_in(
+        &self,
+    ) -> Option<BackgroundGpuComputeFailed> {
+        self.gpu_settings.as_ref()?;
+        self.gpu_compute
+            .as_ref()
+            .expect("选 gpu 的那一组在校验那一刻就带上实现：没给实现照 GPU 不在拒了")
+            .lock()
+            .put_the_gpus_opened_since_the_settings_were_handed_in_in_effect()
+    }
+
     /// 这一组带着的 GPU 实现：选 `gpu` 时恒有；选 `cpu` 时是挂载或切换时递进来的那一份（以后切到 `gpu` 没另给实现时用它），没给是 `None`。
     #[must_use]
     pub fn gpu_compute(&self) -> Option<&BackgroundGpuComputeHandle> {
@@ -855,7 +872,7 @@ impl AdmittedBackgroundComputeSettings {
 /// [`BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT`]（不给就是默认）。
 ///
 /// # Errors
-/// [`BackgroundComputeSettingsRefused`] 除阈值越界之外的八种。
+/// [`BackgroundComputeSettingsRefused`] 除阈值越界之外的七种。
 pub fn admit_background_compute_settings_with_the_gpu_compute(
     settings: BackgroundComputeSettings,
     gpu_compute: Option<BackgroundGpuComputeHandle>,
@@ -872,7 +889,7 @@ pub fn admit_background_compute_settings_with_the_gpu_compute(
 /// 选 `cpu` 时卡与上限照收、不校验，给了实现也带着。只读卡清单，不碰盘。
 ///
 /// # Errors
-/// [`BackgroundComputeSettingsRefused`] 的九种。
+/// [`BackgroundComputeSettingsRefused`] 的八种。
 pub fn admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold(
     settings: BackgroundComputeSettings,
     gpu_compute: Option<BackgroundGpuComputeHandle>,
@@ -911,14 +928,13 @@ pub fn admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_com
 /// 过了交回交给实现的那一组：参与的卡按上限对应的次序、各带上限，与等的上限。卡清单只读一次，判与取参与的卡读的是同一份。只读卡清单，不碰盘。
 ///
 /// # Errors
-/// [`BackgroundComputeSettingsRefused`] 除阈值越界之外的八种。
+/// [`BackgroundComputeSettingsRefused`] 除阈值越界之外的七种。
 pub fn admit_gpu_settings_against_the_inventory(
     settings: &BackgroundComputeSettings,
     gpu_inventory: &dyn GpuInventory,
 ) -> Result<AdmittedGpuSettings, BackgroundComputeSettingsRefused> {
     let inventory_read_once = GpuInventoryReadOnce {
         devices_with_total_memory: gpu_inventory.devices_with_total_memory(),
-        devices_not_opened_at_the_mount: gpu_inventory.devices_not_opened_at_the_mount(),
     };
     admit_background_compute_settings(
         BackgroundComputeSettings {
@@ -962,27 +978,21 @@ pub fn admit_gpu_settings_against_the_inventory(
 /// 读过一次的卡清单：判与取参与的卡都读这一份，不读第二次（实现答的清单两次之间可能变）。
 struct GpuInventoryReadOnce {
     devices_with_total_memory: Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)>,
-    devices_not_opened_at_the_mount: Vec<GpuDeviceNotOpenedAtTheMount>,
 }
 
 impl GpuInventory for GpuInventoryReadOnce {
     fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
         self.devices_with_total_memory.clone()
     }
-
-    fn devices_not_opened_at_the_mount(&self) -> Vec<GpuDeviceNotOpenedAtTheMount> {
-        self.devices_not_opened_at_the_mount.clone()
-    }
 }
 
 /// 第 4 条的校验：选 `cpu` 时卡、显存上限与等 GPU 的上限都照收、不校验（切到 `gpu` 那一刻再校验）；选 `gpu` 时，依次判 GPU 在不在、
-/// 点名的卡在不在清单里、上限个数与参与的卡数等不等、每个上限超没超那张卡的显存总量、等 GPU 的上限是不是 0、参与的卡里有没有挂载那一刻没开成的，
-/// 有一样不对就拒。最后那一判排在最后，同产品 GPU 包挂载那一刻的次序：先判设置、再开卡，点名的卡开不了才拒。
+/// 点名的卡在不在清单里、上限个数与参与的卡数等不等、每个上限超没超那张卡的显存总量、等 GPU 的上限是不是 0，有一样不对就拒。
 /// 只读卡清单，不碰盘。只校验：交回的是交进来的那一组，不带实现、装不进挂载；挂载与切换经
 /// [`admit_background_compute_settings_with_the_gpu_compute`]（卡清单由实现答），产品 GPU 包开包时用这一份。
 ///
 /// # Errors
-/// [`BackgroundComputeSettingsRefused`] 除阈值越界之外的八种。
+/// [`BackgroundComputeSettingsRefused`] 除阈值越界之外的七种。
 pub fn admit_background_compute_settings(
     settings: BackgroundComputeSettings,
     gpu_inventory: Option<&dyn GpuInventory>,
@@ -1030,19 +1040,6 @@ pub fn admit_background_compute_settings(
     if settings.background_gpu_wait_limit.is_zero() {
         return Err(BackgroundComputeSettingsRefused::GpuWaitLimitOfZero);
     }
-    let devices_not_opened_at_the_mount = gpu_inventory.devices_not_opened_at_the_mount();
-    for (participating_device, _total_memory) in &participating_devices {
-        if let Some(not_opened) = devices_not_opened_at_the_mount
-            .iter()
-            .find(|not_opened| not_opened.device == *participating_device)
-        {
-            return Err(
-                BackgroundComputeSettingsRefused::ParticipatingGpuDeviceNotOpenedAtTheMount(
-                    not_opened.clone(),
-                ),
-            );
-        }
-    }
     Ok(settings)
 }
 
@@ -1175,7 +1172,9 @@ pub enum BackgroundComputeError {
         segment: GpuComputedSegment,
         failure: BackgroundGpuComputeFailed,
     },
-    /// 挂载那一刻、或挂着切换生效的那一批开头选 `gpu`，实现照交给它的那一组一张卡都用不上（[`BackgroundGpuCompute::failure_under_the_settings_handed_in`]）：
+    /// 挂载那一刻、或挂着切换生效的那一批开头选 `gpu`，实现照交给它的那一组一张卡都用不上（[`BackgroundGpuCompute::failure_under_the_settings_handed_in`]），
+    /// 或那一组点到的卡在会话锁外重开到上限都没开成（[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]，
+    /// 报的是 [`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]）：
     /// 挂载照常做成、切换照收，有 GPU 路径的后台任务当场停着、报这个错，不等那一组下的头一批交回出错。
     GpuUnusableUnderTheSettingsHandedIn { failure: BackgroundGpuComputeFailed },
 }
@@ -1597,9 +1596,12 @@ impl BackgroundComputeOfThisMount {
         Ok(())
     }
 
-    /// 两批之间换上挂着请求、待生效的那一组（没有就什么都不做）：换上的是 `gpu` 的就把它交给带着的实现（从下一批起照它跑），实现照它一张卡都用不上
+    /// 两批之间换上挂着请求、待生效的那一组（没有就不换）：换上的是 `gpu` 的就把它交给带着的实现（从下一批起照它跑），实现照它一张卡都用不上
     /// 就照挂载那一刻一样，有 GPU 路径的后台任务当场记成出错停着（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；切换照收）；
-    /// 用得上就没有停着的（请求切换那一刻已解除全部停下）。
+    /// 用得上就没有停着的（请求切换那一刻已解除全部停下）。之后（换没换都做）生效的是 `gpu` 的，就让实现把交设置以来在会话锁外开完的卡换上
+    /// （[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]）：实现交回重开到上限都没开成的那个出错，
+    /// 就照「一张卡都用不上」一样把有 GPU 路径的后台任务当场记成出错停着、报那个错（用户定案「3 次都不成：GC 停下，报警……不自动再试」），
+    /// 直到下一次通过校验的切换。
     /// 一批开头（[`Self::begin_a_batch`]）调；整理的两个入口在判「整理停没停」之前也调（显式入口一趟开头、后台入口判起不起跑之前），
     /// 切到一张卡都用不上的那一组之后，下一次进来在任何一个写之前就判出停着：不走删除续做、不取走「该跑了」，与挂载那一刻同。
     /// 只在两批之间调：一批之内的切换只记成待生效。不读写盘。
@@ -1611,6 +1613,12 @@ impl BackgroundComputeOfThisMount {
                 tasks_stopped_when_no_gpu_is_usable_under_the_settings_handed_in(&self.in_effect);
             self.replace_the_tasks_stopped(tasks_stopped);
         }
+        if let Some(failure) = self
+            .in_effect
+            .failure_of_the_gpus_opened_since_the_gpu_settings_were_handed_in()
+        {
+            self.replace_the_tasks_stopped(tasks_with_a_gpu_path_stopped_by(&failure));
+        }
     }
 
     /// 一批开头（每批读一次设置的那一处）：先把待生效的换上（[`Self::put_the_switch_requested_from_the_next_batch_in_effect`]），
````

### crates/singlefs-mount-runtime/src/lib.rs

````diff
diff --git a/crates/singlefs-mount-runtime/src/lib.rs b/crates/singlefs-mount-runtime/src/lib.rs
index 39e95bf..e382a69 100644
--- a/crates/singlefs-mount-runtime/src/lib.rs
+++ b/crates/singlefs-mount-runtime/src/lib.rs
@@ -967,7 +967,9 @@ impl<Device: BlockDevice> MountedRuntime<Device> {
             .map_err(RuntimeRefused::Refused)
     }
 
-    /// 挂着切换后台计算的设置，从下一批起生效（不排空待发）。开卡归调用方：开卡器不在这一条路上，`gpu_compute` 由调用方先开好递进来。
+    /// 挂着切换后台计算的设置，从下一批起生效（不排空待发）。开卡器不在这一条路上：新递一份实现的，`gpu_compute` 由调用方先开好递进来；
+    /// 沿用这次挂载那一份而点到它没开着的卡的，切换照收，那一份实现在会话锁外开卡（产品 GPU 包，开不成隔一段时间重开），这一次调用与之后的批都不等它，
+    /// 开完了在两批之间换上；重开到上限都没开成就在那时停 GC、报警（`singlefs_core::background_compute::BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`）。
     ///
     /// # Errors
     /// 同 [`Self::accept`]。
````

