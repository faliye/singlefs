# c996-code-r2 被判的产品代码改动（2026-11-12）

第一轮判决之后的修补补丁里落在 `crates/*/src/` 下的块，原样。测试与变异表的改动在补丁目录里。

## 补丁 `/home/fy5090/.local/share/singlefs/work/c996-code-r1-fixes/patch/crates.patch`

### crates/singlefs-background-gpu/src/background_gpu_compute.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/background_gpu_compute.rs b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
index 9b2af18c24269c8b5faac3d2eac3e5c1fd74b611..f4f5da2b8686eb4500f46df6a02f1e8d67c2758d 100644
--- a/crates/singlefs-background-gpu/src/background_gpu_compute.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
@@ -7,8 +7,8 @@
 //! 等 GPU 超过上限（第 4 条：超时算出错）在同一张卡上重交 3 次都等超过上限之后（第 4 条「至多 3 次，都超时才报警」）映射成
 //! `GpuUnusableAfterResubmissions`，带卡、重交了几次、等的上限，分得清最后那一次是这一批的那一次提交、还是前一次超时的那一次没做完：这是「这张卡用不了」那一声报警。
 //! 参与的卡的显存上限连最小的一片都装不下、一张都不剩（第 3 条「连最小的一片都装不下，这张卡就不用」）映射成 `EveryParticipatingGpuLeftOutUnderItsMemoryLimit`，
-//! 带每张被晾下的卡、上限与至少要多少；挂载那一刻与切换生效的那一批开头核心层就经 [`BackgroundGpuCompute::failure_under_the_settings_handed_in`] 读到，
-//! 当场报警停着、不拒挂载也不拒切换。
+//! 带每张被晾下的卡、上限与至少要多少；挂载那一刻与切换的那一组换上那一刻核心层就从两批之间那一问的答案里读到
+//! （[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`] 答换上了时带着），当场报警停着、不拒挂载也不拒切换。
 //! 一张参与的卡连这一批最小的一片都装不下映射成 `InputNotSupportedOnTheGpu` 的「一条装不进一段」，带卡、位置与最小的一片的字节数。
 //! 核心层交来的设置（挂载那一刻与每次切换生效的那一批开头）转给两份核那一层，从下一批起照它分活、切段、等；三段的连续不确定计数不动。
 //!
@@ -16,16 +16,18 @@
 //! 测试里换上在 CPU 上算的替身，比对、计数、映射这几样不起卡就钉得住。
 //! 挂载那一刻的入口 [`open_background_gpu_compute`]：只在参与的卡上开卡（卡清单、显存上限、等 GPU 的上限，校验同核心层 `admit_background_compute_settings`），
 //! 交回核心层挂载与切换收的把手；切换点到没开的卡，在会话锁外开、开完了在两批之间换上（`background_gpu::BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`），
+//! 两批之间那一问照最近交来的那一组答还在开卡、换上了（连同用不用得上）、开不成三种（[`LatestGpuSettingsHandedIn`]），核心层开卡期间把切换记成还没生效；
 //! 重开到上限都没开成映射成 `GpuNotOpenedAfterTheReopenings`，带卡与每一次的原因，核心层当场停 GC、报警。新开的卡并进这一份实现：连续不确定的计数不因切换清零。
+//! 切到 `cpu` 时核心层告知一张卡都不参与，开着的卡全部关掉（`background_gpu::BackgroundGpuOverTheCards::use_no_gpu_from_the_next_batch`）。
 
 use singlefs_core::allocator::AllocationRecord;
 use singlefs_core::background_compute::{
     AdmittedGpuSettings, BackgroundGpuCompute, BackgroundGpuComputeFailed,
     BackgroundGpuComputeHandle, ClusterSegmentEntryKey, GpuDeviceIdentifier, GpuEntryDecision,
     GpuFailureOnTheDevice, GpuInventory, GpuMemoryInBytes, GpuShortOfFreeMemoryForTheSmallestPiece,
-    GpuSubmissionsOfTheBatch, InputNotSupportedOnTheGpu, RegionRecordEntryKey,
-    RegionRecordToClassifyOnTheGpu, SubmissionWaitedForPastTheLimit, UnitChecksumOutcome,
-    UnitChecksumToVerifyOnTheGpu,
+    GpuSubmissionsOfTheBatch, InputNotSupportedOnTheGpu, LatestGpuSettingsHandedIn,
+    RegionRecordEntryKey, RegionRecordToClassifyOnTheGpu, SubmissionWaitedForPastTheLimit,
+    UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
 };
 use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
 use singlefs_core::logged_operations::CompactionRegion;
@@ -36,14 +38,14 @@ use crate::background_gpu::{
     open_background_gpu_with_the_memory_leaked_in_this_process, BackgroundGpu,
     BackgroundGpuOpenRefused, BackgroundGpuOverTheCards, BatchNotSplitOverTheGpus,
     CardOfTheBackgroundGpu, ChecksumBatchFailed, GpuFailureCause, GpuNotOpenedAfterTheReopenings,
-    LiveSlotBatchFailed, RegionRecordBatchFailed, SmallestPieceNotHeldOnTheGpu,
-    VulkanInstanceOfThisProcess, WaitForTheOpeningAtASwitch,
+    LatestSettingsHandedIn, LiveSlotBatchFailed, RegionRecordBatchFailed,
+    SmallestPieceNotHeldOnTheGpu, VulkanInstanceOfThisProcess, WaitForTheOpeningAtASwitch,
 };
 use crate::checksum_kernels::{
     reconciled_unit_checksums, ChecksumKernelOutcome, UnitChecksumToVerify,
 };
 use crate::gpus_of_this_host::{
-    GpuLeftOutUnderItsMemoryLimit, GpuMemoryLeakedInThisProcess,
+    GpuLeftOutUnderItsMemoryLimit, GpuMemoryLeakedInThisProcess, GpusLeakedWhenClosedAtASwitch,
     GpusLeakedWhenTheBackgroundGpuIsDropped,
 };
 use crate::live_slot_kernels::reconciled_live_slots_of_the_segments;
@@ -93,12 +95,17 @@ pub trait BothKernelsOfTheThreeSegments: GpuInventory {
     /// （`singlefs_core::background_compute::BackgroundGpuCompute::use_the_settings_from_the_next_batch` 转来的，只在两批之间）。
     fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings);
 
+    /// 从下一批起一张卡都不参与（`singlefs_core::background_compute::BackgroundGpuCompute::use_no_gpu_from_the_next_batch` 转来的，切到 `cpu` 时）：
+    /// 开着的卡全部关掉（[`BackgroundGpuOverTheCards::use_no_gpu_from_the_next_batch`]）。不开卡的替身没有这一样，什么都不做。
+    fn use_no_gpu_from_the_next_batch(&mut self) {}
+
     /// 两批之间：会话锁外为切换开卡的那一次开完了就换上（[`BackgroundGpuOverTheCards::put_the_gpus_opened_since_the_switch_in_effect`]，不等它开完），
-    /// 重开到上限都没开成交回那张卡与每一次的原因（只交一次）。不开卡的替身没有这一样，交回 `None`。
-    fn put_the_gpus_opened_since_the_switch_in_effect(
-        &mut self,
-    ) -> Option<GpuNotOpenedAfterTheReopenings> {
-        None
+    /// 交回最近交来的那一组此刻怎样了（[`LatestSettingsHandedIn`]）。不开卡的替身交来那一刻就换上：答换上了，连同
+    /// [`Self::batch_not_split_over_the_gpus_under_the_settings_in_use`]。
+    fn put_the_gpus_opened_since_the_switch_in_effect(&mut self) -> LatestSettingsHandedIn {
+        LatestSettingsHandedIn::PutInEffect {
+            not_split_over_the_gpus: self.batch_not_split_over_the_gpus_under_the_settings_in_use(),
+        }
     }
 
     /// 此刻这组设置下一批分不分得到卡上（与这一批有几条无关）：分不到交回是哪一种（换设置时量代价出错、参与的卡因显存上限一张都不剩），
@@ -147,9 +154,11 @@ impl<Card: CardOfTheBackgroundGpu> BothKernelsOfTheThreeSegments
         BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch(self, settings);
     }
 
-    fn put_the_gpus_opened_since_the_switch_in_effect(
-        &mut self,
-    ) -> Option<GpuNotOpenedAfterTheReopenings> {
+    fn use_no_gpu_from_the_next_batch(&mut self) {
+        BackgroundGpuOverTheCards::use_no_gpu_from_the_next_batch(self);
+    }
+
+    fn put_the_gpus_opened_since_the_switch_in_effect(&mut self) -> LatestSettingsHandedIn {
         BackgroundGpuOverTheCards::put_the_gpus_opened_since_the_switch_in_effect(
             self,
             WaitForTheOpeningAtASwitch::NotAtAll,
@@ -193,10 +202,11 @@ impl<Kernels> BackgroundGpuComputeThroughBothKernels<Kernels> {
 }
 
 /// 挂载那一刻开后台 GPU 的结局：核心层挂载与切换收的把手，挂载那一刻开着的卡（只有参与的卡，按总线地址排），与开卡那一组点了而因显存上限连最小的一片都装不下、不用的卡（按参与的次序，各带上限与至少要多少；第 3 条
-/// 「连最小的一片都装不下，这张卡就不用」）。参与的卡因此一张都不剩时挂载不拒：核心层挂载那一刻把校验过的那一组交给实现、读
-/// [`BackgroundGpuCompute::failure_under_the_settings_handed_in`]，读到 `EveryParticipatingGpuLeftOutUnderItsMemoryLimit` 就当场报警停着。
+/// 「连最小的一片都装不下，这张卡就不用」）。参与的卡因此一张都不剩时挂载不拒：核心层挂载那一刻把校验过的那一组交给实现、问两批之间那一问
+/// （[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]），答换上了而带着 `EveryParticipatingGpuLeftOutUnderItsMemoryLimit`
+/// 就当场报警停着。
 /// 给了 `background_gpu_devices` 时没点名而交不成一张卡、没收进卡清单的独显各带原因（`unnamed_discrete_gpus_left_off_the_list`）；
-/// 卸载（丢掉核心层收的把手）之后泄漏了哪几张卡，读 `gpus_leaked_when_dropped`。
+/// 卸载（丢掉核心层收的把手）之后泄漏了哪几张卡，读 `gpus_leaked_when_dropped`；挂着切换关卡时泄漏了哪几张卡，随时读 `gpus_leaked_when_closed_at_a_switch`。
 #[derive(Debug)]
 pub struct OpenedBackgroundGpuCompute {
     pub gpu_compute: BackgroundGpuComputeHandle,
@@ -204,6 +214,7 @@ pub struct OpenedBackgroundGpuCompute {
     pub gpus_left_out_under_their_memory_limit: Vec<GpuLeftOutUnderItsMemoryLimit>,
     pub unnamed_discrete_gpus_left_off_the_list: Vec<DiscreteGpuListRefused>,
     pub gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped,
+    pub gpus_leaked_when_closed_at_a_switch: GpusLeakedWhenClosedAtASwitch,
 }
 
 /// 挂载那一刻开后台 GPU、交回核心层挂载与切换收的把手（D26（后台整理与放置回收） 已定项 13「挂载时由调用方把它的实现递进去」）：
@@ -246,6 +257,7 @@ pub fn background_gpu_compute_of_the_opened_gpu(
         .unnamed_discrete_gpus_left_off_the_list()
         .to_vec();
     let gpus_leaked_when_dropped = background_gpu.gpus_leaked_when_dropped();
+    let gpus_leaked_when_closed_at_a_switch = background_gpu.gpus_leaked_when_closed_at_a_switch();
     OpenedBackgroundGpuCompute {
         gpu_compute: BackgroundGpuComputeHandle::of_an_implementation(
             BackgroundGpuComputeOnTheParticipatingGpus::with_no_entry_uncertain(background_gpu),
@@ -254,6 +266,7 @@ pub fn background_gpu_compute_of_the_opened_gpu(
         gpus_left_out_under_their_memory_limit,
         unnamed_discrete_gpus_left_off_the_list,
         gpus_leaked_when_dropped,
+        gpus_leaked_when_closed_at_a_switch,
     }
 }
 
@@ -660,17 +673,38 @@ impl<Kernels: BothKernelsOfTheThreeSegments> BackgroundGpuCompute
         self.kernels.use_the_settings_from_the_next_batch(settings);
     }
 
-    /// 转给两份核那一层（[`BothKernelsOfTheThreeSegments::put_the_gpus_opened_since_the_switch_in_effect`]），重开到上限都没开成的那张照
+    /// 转给两份核那一层；三段的连续不确定计数不动（第 8 条）。
+    fn use_no_gpu_from_the_next_batch(&mut self) {
+        self.kernels.use_no_gpu_from_the_next_batch();
+    }
+
+    /// 转给两份核那一层（[`BothKernelsOfTheThreeSegments::put_the_gpus_opened_since_the_switch_in_effect`]），三种答案一一对过去：
+    /// 换上了而这组设置下一批分不到卡上的照 [`core_failure_of_the_batch_not_split_over_the_gpus`] 报，重开到上限都没开成的那张照
     /// [`core_failure_of_the_gpu_not_opened_after_the_reopenings`] 报；三段的连续不确定计数不动（新开的卡并进这一份实现，不整份换掉）。
     fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
         &mut self,
-    ) -> Option<BackgroundGpuComputeFailed> {
-        self.kernels
+    ) -> LatestGpuSettingsHandedIn {
+        match self
+            .kernels
             .put_the_gpus_opened_since_the_switch_in_effect()
-            .map(|not_opened| core_failure_of_the_gpu_not_opened_after_the_reopenings(&not_opened))
+        {
+            LatestSettingsHandedIn::StillOpeningItsCards => {
+                LatestGpuSettingsHandedIn::StillOpeningTheirCards
+            }
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus,
+            } => LatestGpuSettingsHandedIn::PutInEffect {
+                failure_under_the_settings: not_split_over_the_gpus
+                    .map(|not_split| core_failure_of_the_batch_not_split_over_the_gpus(&not_split)),
+            },
+            LatestSettingsHandedIn::NotOpened(not_opened) => LatestGpuSettingsHandedIn::NotOpened {
+                failure: core_failure_of_the_gpu_not_opened_after_the_reopenings(&not_opened),
+            },
+        }
     }
 
-    /// 问两份核那一层此刻这组设置下一批分不分得到卡上，分不到照 [`core_failure_of_the_batch_not_split_over_the_gpus`] 报。
+    /// 问两份核那一层此刻这组设置下一批分不分得到卡上，分不到照 [`core_failure_of_the_batch_not_split_over_the_gpus`] 报
+    /// （核心层不直接读它，读的是两批之间那一问答换上了时带着的那一个；e202 的包装照转）。
     fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
         self.kernels
             .batch_not_split_over_the_gpus_under_the_settings_in_use()
````

### crates/singlefs-background-gpu/src/background_gpu.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/background_gpu.rs b/crates/singlefs-background-gpu/src/background_gpu.rs
index 23aeb6342efa630ec1f6f1e3abbff10d8bbd7179..dca4c19fb1d025783b5c3a37ff9e7003a78e7507 100644
--- a/crates/singlefs-background-gpu/src/background_gpu.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu.rs
@@ -5,10 +5,13 @@
 //! 参与的卡由设置定（[`BackgroundGpuOverTheCards::participating_gpus`] 报出来），挂着切换时核心层交来新的一组
 //! （[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]，两批之间），从下一批起只在它点的卡上、照它给的显存上限与等的上限跑：
 //! 开着的卡不重开；它点到没开的卡，照用户定案「切换请求照收，在会话锁外开新卡」在另一条线程上开（开卡同挂载那一刻，[`OpeningOfAGpuAtASwitch`]；
-//! 开不成同样隔 1 分钟重开、至多重开 3 次），开着的这段时间照旧照交来之前那一组跑，开成了在之后的两批之间换上
-//! （[`BackgroundGpuOverTheCards::put_the_gpus_opened_since_the_switch_in_effect`]）；重开到上限都没开成就报出那张卡与每一次的原因，
-//! 之后每一批报成它的 GPU 出错（核心层当场停 GC、报警），不自动再试，直到下一次换设置。换上一组时不再点名的卡关掉（用户定案「切换时关掉」），
-//! 之后再点它就重开（分活、切段、等的纯逻辑在 `crate::gpus_of_this_host`）。
+//! 开不成同样隔 1 分钟重开、至多重开 3 次；起不了开卡线程、开卡里 panic 都算那一次没开成），开着的这段时间照旧照交来之前那一组跑，开成了在之后的两批之间换上
+//! （[`BackgroundGpuOverTheCards::put_the_gpus_opened_since_the_switch_in_effect`]，答最近交来的那一组还在开卡、换上了、开不成三种）；
+//! 开卡的结局按交设置的次序认，为更早一次交来的开的、开不成的不报，照最近那一组重开；重开到上限都没开成就报出那张卡与每一次的原因，
+//! 此刻生效那一组之外开着的卡全部关掉，之后每一批报成它的 GPU 出错（核心层当场停 GC、报警），不自动再试，直到下一次换设置。
+//! 同一张卡上前一次开卡到点之后还没回来就不再起下一次（算那一次又没开成）；卸载与切到 `cpu` 时叫停切换开卡线程。
+//! 换上一组时不再点名的卡关掉（用户定案「切换时关掉」），切到 `cpu` 时开着的卡全部关掉（用户定案「全部关掉」），有提交还在跑的那张同卸载整张泄漏、
+//! 累计进对外读得到的一份（[`BackgroundGpuOverTheCards::gpus_leaked_when_closed_at_a_switch`]）；之后再点它就重开（分活、切段、等的纯逻辑在 `crate::gpus_of_this_host`）。
 //! 挂着的时候每一批按卡的顺序填满参与的卡（前面的卡填到它照显存上限一次放得下的量，装不下的才给后面的卡；挂载时量的代价不用来分活）、按显存上限切段、两份核各算一次、逐条比对。三段都接上了：扫描里的校验和核对（[`BackgroundGpuOverTheCards::verify_unit_checksums`]）、
 //! 区域扫描里的单元头解析与映射反查（[`BackgroundGpuOverTheCards::classify_region_records`]）与选段打分的每段活槽数（[`BackgroundGpuOverTheCards::count_live_slots_per_cluster_segment`]）。
 //! 本机的卡清单（核心层 `GpuInventory`）答的是此刻本机在的独显与各自的显存总量（按总线地址排，开着的与没参与而没开的都列；每次答之前现枚举一次，
@@ -41,11 +44,12 @@
 
 use std::future::Future;
 use std::ops::Range;
+use std::sync::atomic::{AtomicBool, Ordering};
 use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, TryRecvError};
 use std::sync::{Arc, Mutex};
 use std::task::{Context, Poll, Wake, Waker};
 use std::thread::JoinHandle;
-use std::time::Duration;
+use std::time::{Duration, Instant};
 
 use singlefs_core::allocator::AllocationRecord;
 use singlefs_core::background_compute::{
@@ -79,8 +83,9 @@ use crate::gpus_of_this_host::{
     memory_limit_of_the_device_under_the_settings, plans_of_a_batch_filling_the_gpus_in_order,
     BatchNotPlannedOverTheGpus, CostAtTheOpening, CostProbeChunk, DiscreteGpusAfterTheOpening,
     GpuBudgetUnderTheSettings, GpuLeakedWhenDropped, GpuLeftOutUnderItsMemoryLimit,
-    GpuMemoryLeakedInThisProcess, GpusLeakedWhenTheBackgroundGpuIsDropped,
-    ParticipatingGpusOnThisHost, PlanOfAGpuForTheBatch, RoomOfAGpuForTheBatch,
+    GpuMemoryLeakedInThisProcess, GpusLeakedWhenClosedAtASwitch,
+    GpusLeakedWhenTheBackgroundGpuIsDropped, ParticipatingGpusOnThisHost, PlanOfAGpuForTheBatch,
+    RoomOfAGpuForTheBatch,
 };
 use crate::live_slot_kernels::{
     decoded_live_slot_outcomes, live_slot_chunk_footprint,
@@ -341,6 +346,47 @@ pub enum BatchNotSplitOverTheGpus {
     EveryParticipatingGpuLeftOut(Vec<GpuLeftOutUnderItsMemoryLimit>),
 }
 
+/// 两批之间那一问（[`BackgroundGpuOverTheCards::put_the_gpus_opened_since_the_switch_in_effect`]）交回的：最近交来的那一组此刻怎样了。封闭集合。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum LatestSettingsHandedIn {
+    /// 还在会话锁外为它开卡：照旧照交来之前那一组跑，这一组还没生效。
+    StillOpeningItsCards,
+    /// 换上了（交来那一刻点的卡都开着就当场换上，开完了在两批之间换上），连同这组设置下一批分不分得到卡上
+    /// （[`BackgroundGpuOverTheCards::batch_not_split_over_the_gpus`]，分得到是 `None`）。
+    PutInEffect {
+        not_split_over_the_gpus: Option<BatchNotSplitOverTheGpus>,
+    },
+    /// 它点到的一张没开的卡重开到上限都没开成：之后每一批报成这张卡的 GPU 出错，直到下一次换设置。
+    NotOpened(GpuNotOpenedAfterTheReopenings),
+}
+
+/// 交来一组设置（或告知一张卡都不参与）的次序号：开卡的结局认它是为哪一次交来的开的，不按设置的内容比
+/// （同一组设置先后交两次是两次，为前一次开的、开不成的不报给后一次）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct HandingInNumber(u64);
+
+impl HandingInNumber {
+    /// 下一次交来的号。
+    fn next(self) -> Self {
+        Self(self.0 + 1)
+    }
+}
+
+/// 交来了、点到没开的卡、还没换上的那一组，连同它是哪一次交来的。
+struct SettingsWaitingForTheOpening {
+    handing_in_number: HandingInNumber,
+    settings: AdmittedGpuSettings,
+}
+
+/// 起切换开卡那条线程的办法（`.claude/rules/fs-design.md` 五条硬要求第 2 条：起不了线程那一支测得到）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum StartOfTheOpeningThread {
+    /// 产品：照 std 起。
+    ByTheStandardLibrary,
+    /// 只供测试：接下来这么多次照起不了线程报，之后照 std 起。
+    RefusedForTests { refusals_left: u32 },
+}
+
 /// 挂着切换点到的一张没开的卡，隔一段时间重开到上限也都没开成（用户定案「3 次都不成：GC 停下，报警并报出哪张卡、每次开不成的原因」）：
 /// 哪张卡、每一次开不成的原因（按开的次序，头一次的在前，1 + 重开次数 个）。
 #[derive(Clone, Debug, PartialEq, Eq)]
@@ -460,10 +506,19 @@ pub struct BackgroundGpuOverTheCards<Card: CardOfTheBackgroundGpu> {
     opening_of_a_gpu_at_a_switch: Arc<Mutex<OpeningOfAGpuAtASwitch<Card>>>,
     /// 开不成时怎么重开（挂载那一刻与切换同一种；产品是 [`ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT`]）。
     reopenings_of_a_gpu_not_opened: ReopeningsOfAGpuNotOpened,
+    /// 最近一次交来设置（或告知一张卡都不参与）的次序号。
+    latest_handing_in: HandingInNumber,
+    /// 此刻生效的那一组点的卡：挂载那一刻是开着的那几张，换上一组时换成它点的，告知一张卡都不参与之后是空。
+    /// 最近交来的那一组开不成时，开着的卡里不在这里的全部关掉。
+    devices_named_by_the_settings_in_effect: Vec<GpuDeviceIdentifier>,
     /// 交来了、点到没开的卡、还没换上的那一组（最近交来的那一组；开卡开完了才换上）。没有是 `None`。
-    settings_waiting_for_the_opening: Option<AdmittedGpuSettings>,
+    settings_waiting_for_the_opening: Option<SettingsWaitingForTheOpening>,
     /// 会话锁外为切换开卡的那一次（至多一次）；没在开是 `None`。
     opening_at_a_switch: Option<OpeningAtASwitch<Card>>,
+    /// 起切换开卡那条线程的办法：产品恒是照 std 起。
+    start_of_the_opening_thread: StartOfTheOpeningThread,
+    /// 切换关卡时泄漏的卡，按关的先后累计（同卸载的规矩：有提交还在跑的那张整张泄漏、不析构）。
+    gpus_leaked_when_closed_at_a_switch: GpusLeakedWhenClosedAtASwitch,
     /// 上一次交回以来三段各自在卡上是怎么交的（[`Self::take_the_submissions_of_the_batch`]）。
     submissions_since_the_last_take: GpuSubmissionsOfTheBatch,
     gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped,
@@ -476,19 +531,42 @@ pub struct BackgroundGpuOverTheCards<Card: CardOfTheBackgroundGpu> {
 /// 产品那一份：开着的卡是 wgpu 设备与六条管线。
 pub type BackgroundGpu = BackgroundGpuOverTheCards<GpuInUse>;
 
-/// 会话锁外为切换开卡的那一次：为哪一组开（交来那一刻点到的没开的卡），与它的结局从哪来。封闭集合。
+/// 会话锁外为切换开卡的那一次：为哪一次交来的设置开（交来那一刻点到的没开的卡），与它的结局从哪来。封闭集合。
 enum OpeningAtASwitch<Card> {
-    /// 开卡线程起来了、在开：开完了经 `outcome_receiver` 送回（至多一次）。
+    /// 开卡线程起来了、在开：开完了经 `outcome_receiver` 送回（至多一次）。`called_off` 置上之后那条线程不再开下一次（卸载、切到 `cpu` 时置）。
     UnderWay {
-        settings: AdmittedGpuSettings,
+        handing_in_number: HandingInNumber,
         outcome_receiver: Receiver<OutcomeOfTheOpeningAtASwitch<Card>>,
         opening_thread: JoinHandle<()>,
+        called_off: Arc<AtomicBool>,
     },
-    /// 起不了开卡线程：头一张要开的卡按开不成报（原因是起不了线程的那个错），下一次两批之间交回。
+    /// 起不了开卡线程：这算头一张要开的卡这一次没开成（`causes_of_each_start` 每次一个原因），隔重开的间隔再起（`not_started_at` 起算），
+    /// 连同线程里的开卡至多开 1 + 重开次数 次。
     ThreadNotStarted {
-        settings: AdmittedGpuSettings,
-        not_opened: GpuNotOpenedAfterTheReopenings,
+        handing_in_number: HandingInNumber,
+        discrete_gpus_to_open: Vec<(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)>,
+        wait_limit: Duration,
+        causes_of_each_start: Vec<GpuFailureCause>,
+        not_started_at: Instant,
     },
+    /// 起不了开卡线程的次数到了重开的上限：结局已定（头一张要开的卡开不成），下一次两批之间收下。
+    EndedWithoutAThread {
+        handing_in_number: HandingInNumber,
+        outcome: OutcomeOfTheOpeningAtASwitch<Card>,
+    },
+}
+
+impl<Card> OpeningAtASwitch<Card> {
+    /// 叫停这一次开卡：开卡线程在下一次开卡之前看到就不再开（正在开的那一次照旧开完）。没起线程的不用叫停。
+    fn call_off(&self) {
+        match self {
+            OpeningAtASwitch::UnderWay { called_off, .. } => {
+                called_off.store(true, Ordering::SeqCst)
+            }
+            OpeningAtASwitch::ThreadNotStarted { .. }
+            | OpeningAtASwitch::EndedWithoutAThread { .. } => {}
+        }
+    }
 }
 
 /// 开卡线程开完了送回的：开成的卡（按开的次序），与重开到上限都没开成的那一张（有一张开不成，后面的就不再开）。
@@ -1035,6 +1113,7 @@ fn opening_of_a_gpu_in_use(
        + Send
        + 'static {
     opening_of_a_card_within_the_limit(
+        WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
         move |discrete_gpu: &DiscreteGpuOnThisHost,
               memory_limit_at_the_opening,
               wait_limit,
@@ -1058,10 +1137,13 @@ fn opening_of_a_gpu_in_use(
 /// 开一张卡那一步（挂载那一刻开参与的卡、切换生效那一刻开挂载时没开的卡，对任何一种卡都一样；`Card` 产品是 [`GpuInUse`]，测试换上替身卡）：
 /// `start_of_the_card` 照这张卡、给它的显存上限、等 GPU 的上限与这个进程里之前几次挂载泄漏在它上面的字节（从 `memory_leaked_in_this_process` 现读）
 /// 交回起这张卡的那一件；那一件连同量代价（[`started_gpu_with_its_cost_measured`]，量不量照 `CostAtTheOpening`；出错时有提交还在跑的那张卡泄漏、
-/// 这一次占着的字节记进 `memory_leaked_in_this_process`）在另一条线程上做，整个至多等 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]
-/// （到点当那张卡没开成，`waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit`）。处置量代价出错的卡在那条线程上：
+/// 这一次占着的字节记进 `memory_leaked_in_this_process`）在另一条线程上做，整个至多等 `opening_limit`（产品是 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]；
+/// 到点当那张卡没开成，`waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit`）。处置量代价出错的卡在那条线程上：
 /// 到点之后那条线程跑完时交回的只是那个错，没有要析构的卡。
+/// 同一张卡上前一次开卡到点之后还没回来（那条线程还挂在驱动里），就不再起下一次：这一次当场算没开成（[`opening_not_started_while_an_earlier_one_has_not_returned`]），
+/// 照重开的间隔往下数（挂载与切换同一个规矩，同一张卡上同时在开的线程至多一条）。
 fn opening_of_a_card_within_the_limit<Card, StartTheCard>(
+    opening_limit: Duration,
     mut start_of_the_card: impl FnMut(&DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>, Duration, u64) -> StartTheCard
         + Send
         + 'static,
@@ -1078,7 +1160,14 @@ where
     Card: CardOfTheBackgroundGpu + Send + 'static,
     StartTheCard: FnOnce() -> Result<Card, GpuFailureCause> + Send + 'static,
 {
+    let devices_with_an_opening_not_returned = Arc::new(Mutex::new(Vec::new()));
     move |discrete_gpu, memory_limit_at_the_opening, wait_limit, cost_at_the_opening| {
+        let Some(opening_not_returned) = OpeningOfTheDeviceNotReturned::noted_unless_one_already_is(
+            &devices_with_an_opening_not_returned,
+            &discrete_gpu.identifier,
+        ) else {
+            return Err(opening_not_started_while_an_earlier_one_has_not_returned());
+        };
         let bytes_leaked_by_earlier_mounts =
             memory_leaked_in_this_process.bytes_leaked_on(&discrete_gpu.identifier);
         let start_the_card = start_of_the_card(
@@ -1088,16 +1177,80 @@ where
             bytes_leaked_by_earlier_mounts,
         );
         let memory_leaked_noted_by_the_opening = memory_leaked_in_this_process.clone();
-        outcome_of_opening_a_gpu_within_the_limit(
-            WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
-            move || {
-                started_gpu_with_its_cost_measured(
-                    start_the_card,
-                    cost_at_the_opening,
-                    &memory_leaked_noted_by_the_opening,
-                )
-            },
-        )
+        outcome_of_opening_a_gpu_within_the_limit(opening_limit, move || {
+            let _returned_when_this_ends = opening_not_returned;
+            started_gpu_with_its_cost_measured(
+                start_the_card,
+                cost_at_the_opening,
+                &memory_leaked_noted_by_the_opening,
+            )
+        })
+    }
+}
+
+/// 一张卡上一次还没回来的开卡：在 `devices_with_an_opening_not_returned` 里记着这张卡，丢它（那一次开卡跑完、或起不了那条线程）时划掉。
+struct OpeningOfTheDeviceNotReturned {
+    devices_with_an_opening_not_returned: Arc<Mutex<Vec<GpuDeviceIdentifier>>>,
+    device: GpuDeviceIdentifier,
+}
+
+impl OpeningOfTheDeviceNotReturned {
+    /// 这张卡上没有还没回来的开卡，就记下这一次、交回它；有就交回 `None`（这一次不起）。
+    fn noted_unless_one_already_is(
+        devices_with_an_opening_not_returned: &Arc<Mutex<Vec<GpuDeviceIdentifier>>>,
+        device: &GpuDeviceIdentifier,
+    ) -> Option<Self> {
+        let mut devices = devices_with_an_opening_not_returned
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner);
+        if devices.contains(device) {
+            return None;
+        }
+        devices.push(device.clone());
+        Some(Self {
+            devices_with_an_opening_not_returned: Arc::clone(devices_with_an_opening_not_returned),
+            device: device.clone(),
+        })
+    }
+}
+
+impl Drop for OpeningOfTheDeviceNotReturned {
+    fn drop(&mut self) {
+        self.devices_with_an_opening_not_returned
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner)
+            .retain(|device| *device != self.device);
+    }
+}
+
+/// 同一张卡上前一次开卡还没回来、这一次不起时报的原因（算这一次没开成，照重开的间隔往下数）。
+fn opening_not_started_while_an_earlier_one_has_not_returned() -> GpuFailureCause {
+    GpuFailureCause::DeviceOrPipelineDidNotStart {
+        reason: "这张卡上前一次开卡到点之后还没回来（还挂在驱动里），这一次不再起".to_string(),
+    }
+}
+
+/// 开一次卡（`open`），开卡里 panic 了就接住、算这一次没开成（原因带着 panic 的原话），不往上传：切换开卡线程照重开的规矩往下走，
+/// 不在两批之间持着会话锁时再 panic。
+fn opening_with_its_panic_caught<Opened>(
+    open: impl FnOnce() -> Result<Opened, GpuFailureCause>,
+) -> Result<Opened, GpuFailureCause> {
+    std::panic::catch_unwind(std::panic::AssertUnwindSafe(open)).unwrap_or_else(|payload| {
+        let said_by_the_panic = payload
+            .downcast_ref::<&str>()
+            .map(|said| (*said).to_string())
+            .or_else(|| payload.downcast_ref::<String>().cloned())
+            .unwrap_or_else(|| "（panic 带的不是字符串）".to_string());
+        Err(GpuFailureCause::DeviceOrPipelineDidNotStart {
+            reason: format!("开卡 panic 了：{said_by_the_panic}"),
+        })
+    })
+}
+
+/// 切换开卡叫停之后（卸载、切到 `cpu`）不再开的那几次报的原因：这一次开卡的结局没人收，原因不报到哪里。
+fn opening_called_off() -> GpuFailureCause {
+    GpuFailureCause::DeviceOrPipelineDidNotStart {
+        reason: "切换开卡叫停了（卸载或切到 cpu），不再开".to_string(),
     }
 }
 
@@ -1105,38 +1258,65 @@ where
 /// （[`gpu_opened_under_its_memory_limit`]，开一次交 `opening_of_a_gpu_at_a_switch`、等 GPU 的上限是这一组的 `wait_limit`），开不成就照 `reopenings`
 /// 重开、每次重开之前在这条线程上睡一个间隔（[`gpu_opened_with_the_reopenings`]）。有一张重开到上限都没开成就停在它，后面的不再开。
 /// 装不下最小的一片的照开、不量代价（换上那一组时照这张卡的账与开着的卡同一判，再判一次）。
+/// 开一次卡里 panic 了照这一次没开成算（[`opening_with_its_panic_caught`]）。`causes_of_each_start` 是起这条线程之前起不了线程的那几次
+/// （各算头一张卡一次没开成）：头一张卡的重开次数扣掉它们、原因排在前面，连同它们至多开 1 + 重开次数 次。
+/// 每一次开卡之前看 `called_off`：置上了就不再开（卸载、切到 `cpu`），也不再睡重开的间隔。
 fn gpus_opened_at_a_switch<Card>(
     opening_of_a_gpu_at_a_switch: &mut OpeningOfAGpuAtASwitch<Card>,
     discrete_gpus_to_open: Vec<(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)>,
     wait_limit: Duration,
     reopenings: ReopeningsOfAGpuNotOpened,
+    causes_of_each_start: Vec<GpuFailureCause>,
+    called_off: &AtomicBool,
     memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
 ) -> OutcomeOfTheOpeningAtASwitch<Card> {
     let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
     let mut opened = Vec::with_capacity(discrete_gpus_to_open.len());
+    let mut causes_before_the_thread = causes_of_each_start;
     for (discrete_gpu, memory_limit) in discrete_gpus_to_open {
-        let opened_with_the_reopenings =
-            gpu_opened_with_the_reopenings(reopenings, std::thread::sleep, || {
-                gpu_opened_under_its_memory_limit(
-                    &discrete_gpu,
-                    memory_limit,
-                    smallest_piece_bytes,
-                    memory_leaked_in_this_process,
-                    |gpu_to_open, memory_limit_at_the_opening, cost_at_the_opening| {
-                        opening_of_a_gpu_at_a_switch(
-                            gpu_to_open,
-                            memory_limit_at_the_opening,
-                            wait_limit,
-                            cost_at_the_opening,
-                        )
-                    },
-                )
-            });
+        let causes_before_this_card = std::mem::take(&mut causes_before_the_thread);
+        let reopenings_of_this_card = ReopeningsOfAGpuNotOpened {
+            interval: reopenings.interval,
+            reopenings_at_most: reopenings
+                .reopenings_at_most
+                .saturating_sub(u32::try_from(causes_before_this_card.len()).unwrap_or(u32::MAX)),
+        };
+        let opened_with_the_reopenings = gpu_opened_with_the_reopenings(
+            reopenings_of_this_card,
+            |interval| {
+                if !called_off.load(Ordering::SeqCst) {
+                    std::thread::sleep(interval);
+                }
+            },
+            || {
+                if called_off.load(Ordering::SeqCst) {
+                    return Err(opening_called_off());
+                }
+                opening_with_its_panic_caught(|| {
+                    gpu_opened_under_its_memory_limit(
+                        &discrete_gpu,
+                        memory_limit,
+                        smallest_piece_bytes,
+                        memory_leaked_in_this_process,
+                        |gpu_to_open, memory_limit_at_the_opening, cost_at_the_opening| {
+                            opening_of_a_gpu_at_a_switch(
+                                gpu_to_open,
+                                memory_limit_at_the_opening,
+                                wait_limit,
+                                cost_at_the_opening,
+                            )
+                        },
+                    )
+                })
+            },
+        );
         match opened_with_the_reopenings {
             Ok((card, _left_out_judged_again_when_the_settings_are_put_in_effect)) => {
                 opened.push(card);
             }
-            Err(causes_of_each_opening) => {
+            Err(causes_in_the_thread) => {
+                let mut causes_of_each_opening = causes_before_this_card;
+                causes_of_each_opening.extend(causes_in_the_thread);
                 return OutcomeOfTheOpeningAtASwitch {
                     opened,
                     not_opened: Some(GpuNotOpenedAfterTheReopenings {
@@ -2136,6 +2316,11 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
             discrete_gpus_after_the_opening.opened.len(),
             "开成的卡与开卡之后的独显清单一一对应"
         );
+        let devices_named_by_the_settings_in_effect = discrete_gpus_after_the_opening
+            .opened
+            .iter()
+            .map(|discrete_gpu| discrete_gpu.identifier.clone())
+            .collect();
         Self {
             cards_on_this_host,
             discrete_gpus_after_the_opening,
@@ -2146,8 +2331,12 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
             cost_not_measured_at_the_last_switch: None,
             opening_of_a_gpu_at_a_switch: Arc::new(Mutex::new(opening_of_a_gpu_at_a_switch)),
             reopenings_of_a_gpu_not_opened,
+            latest_handing_in: HandingInNumber(0),
+            devices_named_by_the_settings_in_effect,
             settings_waiting_for_the_opening: None,
             opening_at_a_switch: None,
+            start_of_the_opening_thread: StartOfTheOpeningThread::ByTheStandardLibrary,
+            gpus_leaked_when_closed_at_a_switch: GpusLeakedWhenClosedAtASwitch::with_none_leaked(),
             submissions_since_the_last_take: GpuSubmissionsOfTheBatch::default(),
             gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped::not_yet_dropped(),
             memory_leaked_in_this_process,
@@ -2183,6 +2372,20 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         self.gpus_leaked_when_dropped.clone()
     }
 
+    /// 挂着切换关卡时泄漏了哪几张卡（有提交还在跑的那张整张泄漏、不析构，同卸载的规矩），按关的先后累计：调用方拿着这一份随时读
+    /// （[`GpusLeakedWhenClosedAtASwitch`]，`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
+    #[must_use]
+    pub fn gpus_leaked_when_closed_at_a_switch(&self) -> GpusLeakedWhenClosedAtASwitch {
+        self.gpus_leaked_when_closed_at_a_switch.clone()
+    }
+
+    /// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：接下来 `refusals` 次起切换开卡那条线程都照起不了报，强制进入「起不了开卡线程」那一支。
+    pub fn refuse_to_start_the_opening_thread_for_tests(&mut self, refusals: u32) {
+        self.start_of_the_opening_thread = StartOfTheOpeningThread::RefusedForTests {
+            refusals_left: refusals,
+        };
+    }
+
     /// 上一次交回以来三段各自在卡上是怎么交的（交成了几次、几片、重交了几次、显存不足重切了几次、分到活的是哪几张卡），交回之后清零。
     pub fn take_the_submissions_of_the_batch(&mut self) -> GpuSubmissionsOfTheBatch {
         std::mem::take(&mut self.submissions_since_the_last_take)
@@ -2212,13 +2415,14 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
     }
 
     /// 从下一批起照核心层交来的这一组跑（D26 已定项 13 第 1–3 条；核心层只在两批之间交）：参与的卡换成它点的那几张、按它的次序，
-    /// 每张卡的显存上限与等的上限换成它给的。开着的卡不重开，各卡的账照旧记着。
-    /// 它点的卡都开着，就当场换上（[`Self::put_the_settings_in_effect`]：不再点名的卡关掉）。它点到没开的卡（挂载时没开的，或之前切换关掉的），
+    /// 每张卡的显存上限与等的上限换成它给的。开着的卡不重开，各卡的账照旧记着。这一次交来记一个新的次序号。
+    /// 它点的卡都开着，就当场换上（[`Self::put_the_settings_in_effect`]：不再点名的卡关掉）。它点到没开的卡（挂载时没开的，或之前切换、切到 `cpu` 关掉的），
     /// 照用户定案「切换请求照收，在会话锁外开新卡」：这一刻不开卡，在另一条线程上按点的次序逐张开（开不成就隔一段时间重开，
     /// [`Self::of_the_cards_opened_at_the_mount`] 给的重开规矩），开着的这段时间照旧照交来之前那一组跑；开完了在之后的两批之间换上
     /// （[`Self::put_the_gpus_opened_since_the_switch_in_effect`]）。上一次切换的卡还在开时又交来一组，记下最近这一组，等那一次开完再看。
     /// 上一次切换留下的「重开到上限都没开成」在这一刻清掉（下一次换设置解除它）。
     pub fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
+        self.latest_handing_in = self.latest_handing_in.next();
         self.not_opened_after_the_reopenings_at_the_last_switch = None;
         let discrete_gpus_to_open = self.discrete_gpus_named_and_not_opened(settings);
         if discrete_gpus_to_open.is_empty() {
@@ -2226,93 +2430,203 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
             self.put_the_settings_in_effect(settings);
             return;
         }
-        self.settings_waiting_for_the_opening = Some(settings.clone());
+        self.settings_waiting_for_the_opening = Some(SettingsWaitingForTheOpening {
+            handing_in_number: self.latest_handing_in,
+            settings: settings.clone(),
+        });
         if self.opening_at_a_switch.is_none() {
-            self.start_the_opening_at_a_switch(settings.clone(), discrete_gpus_to_open);
+            self.start_the_opening_at_a_switch(
+                self.latest_handing_in,
+                discrete_gpus_to_open,
+                settings.wait_limit(),
+                Vec::new(),
+            );
+        }
+    }
+
+    /// 从下一批起一张卡都不参与（核心层切到 `cpu` 时交来；用户定案「切到 cpu 时开着的卡全部关掉」）：这一次也记一个新的次序号，
+    /// 在会话锁外开着的那一次叫停（开卡线程不再开下一次，开出来的没人收、在那条线程上照常丢），已送回还没取走的开卡结局里开成的卡收下再关，
+    /// 还在等开卡的那一组丢掉，开着的卡全部关掉（有提交还在跑的那张整张泄漏，[`Self::close_the_cards`]），「重开到上限都没开成」与「换设置时量代价出错」清掉。
+    /// 之后再交来一组，它点的卡都当没开的卡照重开的规矩开。
+    pub fn use_no_gpu_from_the_next_batch(&mut self) {
+        self.latest_handing_in = self.latest_handing_in.next();
+        self.settings_waiting_for_the_opening = None;
+        self.not_opened_after_the_reopenings_at_the_last_switch = None;
+        self.cost_not_measured_at_the_last_switch = None;
+        if let Some(opening) = self.opening_at_a_switch.take() {
+            opening.call_off();
+            let cards_opened_and_not_taken = match opening {
+                OpeningAtASwitch::UnderWay {
+                    outcome_receiver, ..
+                } => outcome_receiver
+                    .try_recv()
+                    .map_or_else(|_nothing_sent| Vec::new(), |outcome| outcome.opened),
+                OpeningAtASwitch::EndedWithoutAThread { outcome, .. } => outcome.opened,
+                OpeningAtASwitch::ThreadNotStarted { .. } => Vec::new(),
+            };
+            self.take_in_the_cards_opened_at_a_switch(cards_opened_and_not_taken);
         }
+        self.devices_named_by_the_settings_in_effect.clear();
+        self.left_out_under_their_memory_limit.clear();
+        self.close_the_cards(|_device| true);
     }
 
     /// 两批之间（核心层每一批开头与整理两个入口判停没停之前，经 `BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`）：
-    /// 会话锁外为切换开卡的那一次开完了就在这一刻收下它的结局，开成的卡接到开着的卡末尾，再照最近交来、在等的那一组办——
-    /// 开卡为的就是它、有一张重开到上限都没开成：这一次开成的卡关掉、照旧没参与，记下那一张，之后每一批报成它的 GPU 出错，交回它（只交这一次）；
-    /// 它点的卡都开着了：换上它（不再点名的卡关掉）；它还点着没开的卡（开卡开的是更早的一组）：为它再开一次。没有在等的那一组（之后交来的一组不用开卡、已换上）：
-    /// 这一次开成的卡都关掉。还在开（`wait` 是 [`WaitForTheOpeningAtASwitch::NotAtAll`] 时不等它）、没在开，都交回 `None`、什么都不动。
+    /// 会话锁外为切换开卡的那一次开完了就在这一刻收下它的结局（[`Self::take_in_the_outcome_of_the_opening`]），起不了开卡线程的到了间隔就再起；
+    /// 然后交回最近交来的那一组此刻怎样了（[`LatestSettingsHandedIn`]）。`wait` 是 [`WaitForTheOpeningAtASwitch::NotAtAll`] 时还在开就不等，
+    /// [`WaitForTheOpeningAtASwitch::UntilItEnds`] 时一直等到为最近交来的那一组开卡开完。
     ///
     /// # Panics
-    /// 开卡线程 panic 了：照原样在这条线程上再 panic（开一张卡的那一步 panic 时同样再 panic，`waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit`）。
+    /// 开卡线程没送结局就断开了：开一次卡里的 panic 在那条线程上就接住（[`opening_with_its_panic_caught`]），断开只剩这个包自己的代码 panic 一种，照原样再 panic。
     pub fn put_the_gpus_opened_since_the_switch_in_effect(
         &mut self,
         wait: WaitForTheOpeningAtASwitch,
-    ) -> Option<GpuNotOpenedAfterTheReopenings> {
-        let (settings_of_the_opening, outcome) = match self.opening_at_a_switch.take()? {
-            OpeningAtASwitch::ThreadNotStarted {
-                settings,
-                not_opened,
-            } => (
-                settings,
-                OutcomeOfTheOpeningAtASwitch {
-                    opened: Vec::new(),
-                    not_opened: Some(not_opened),
-                },
-            ),
-            OpeningAtASwitch::UnderWay {
-                settings,
-                outcome_receiver,
-                opening_thread,
-            } => {
-                let received = match wait {
-                    WaitForTheOpeningAtASwitch::NotAtAll => outcome_receiver.try_recv(),
-                    WaitForTheOpeningAtASwitch::UntilItEnds => outcome_receiver
-                        .recv()
-                        .map_err(|_disconnected| TryRecvError::Disconnected),
-                };
-                match received {
-                    Ok(outcome) => (settings, outcome),
-                    Err(TryRecvError::Empty) => {
-                        self.opening_at_a_switch = Some(OpeningAtASwitch::UnderWay {
-                            settings,
-                            outcome_receiver,
-                            opening_thread,
-                        });
-                        return None;
+    ) -> LatestSettingsHandedIn {
+        while let Some(opening) = self.opening_at_a_switch.take() {
+            match opening {
+                OpeningAtASwitch::EndedWithoutAThread {
+                    handing_in_number,
+                    outcome,
+                } => self.take_in_the_outcome_of_the_opening(handing_in_number, outcome),
+                OpeningAtASwitch::ThreadNotStarted {
+                    handing_in_number,
+                    discrete_gpus_to_open,
+                    wait_limit,
+                    causes_of_each_start,
+                    not_started_at,
+                } => {
+                    let interval_left = self
+                        .reopenings_of_a_gpu_not_opened
+                        .interval
+                        .saturating_sub(not_started_at.elapsed());
+                    if !interval_left.is_zero() {
+                        match wait {
+                            WaitForTheOpeningAtASwitch::NotAtAll => {
+                                self.opening_at_a_switch =
+                                    Some(OpeningAtASwitch::ThreadNotStarted {
+                                        handing_in_number,
+                                        discrete_gpus_to_open,
+                                        wait_limit,
+                                        causes_of_each_start,
+                                        not_started_at,
+                                    });
+                                break;
+                            }
+                            WaitForTheOpeningAtASwitch::UntilItEnds => {
+                                std::thread::sleep(interval_left);
+                            }
+                        }
                     }
-                    Err(TryRecvError::Disconnected) => {
-                        std::panic::resume_unwind(opening_thread.join().expect_err(
-                            "开卡线程没送结局就断开只有一种：它 panic 了（正常结束一定送了，送的时候这一头还拿着收的那一端）",
-                        ))
+                    self.start_the_opening_at_a_switch(
+                        handing_in_number,
+                        discrete_gpus_to_open,
+                        wait_limit,
+                        causes_of_each_start,
+                    );
+                }
+                OpeningAtASwitch::UnderWay {
+                    handing_in_number,
+                    outcome_receiver,
+                    opening_thread,
+                    called_off,
+                } => {
+                    let received = match wait {
+                        WaitForTheOpeningAtASwitch::NotAtAll => outcome_receiver.try_recv(),
+                        WaitForTheOpeningAtASwitch::UntilItEnds => outcome_receiver
+                            .recv()
+                            .map_err(|_disconnected| TryRecvError::Disconnected),
+                    };
+                    match received {
+                        Ok(outcome) => {
+                            self.take_in_the_outcome_of_the_opening(handing_in_number, outcome);
+                        }
+                        Err(TryRecvError::Empty) => {
+                            self.opening_at_a_switch = Some(OpeningAtASwitch::UnderWay {
+                                handing_in_number,
+                                outcome_receiver,
+                                opening_thread,
+                                called_off,
+                            });
+                            break;
+                        }
+                        Err(TryRecvError::Disconnected) => {
+                            std::panic::resume_unwind(opening_thread.join().expect_err(
+                                "开卡线程没送结局就断开只有一种：它 panic 了（正常结束一定送了，送的时候这一头还拿着收的那一端）",
+                            ))
+                        }
                     }
                 }
             }
-        };
-        let devices_opened_by_this_opening: Vec<GpuDeviceIdentifier> = outcome
-            .opened
-            .iter()
-            .map(|card| card.discrete_gpu().identifier.clone())
-            .collect();
-        for card in outcome.opened {
+        }
+        self.latest_settings_handed_in()
+    }
+
+    /// 最近交来的那一组此刻怎样了：还在等开卡是 [`LatestSettingsHandedIn::StillOpeningItsCards`]；为它开卡重开到上限都没开成是
+    /// [`LatestSettingsHandedIn::NotOpened`]；别的是换上了，连同这组设置下一批分不分得到卡上。
+    fn latest_settings_handed_in(&self) -> LatestSettingsHandedIn {
+        if self.settings_waiting_for_the_opening.is_some() {
+            return LatestSettingsHandedIn::StillOpeningItsCards;
+        }
+        if let Some(not_opened) = &self.not_opened_after_the_reopenings_at_the_last_switch {
+            return LatestSettingsHandedIn::NotOpened(not_opened.clone());
+        }
+        LatestSettingsHandedIn::PutInEffect {
+            not_split_over_the_gpus: self.batch_not_split_over_the_gpus(),
+        }
+    }
+
+    /// 收下切换开卡开成的卡：接到开着的卡末尾，卡清单里从没开的那几张挪到开着的那几张。
+    fn take_in_the_cards_opened_at_a_switch(&mut self, cards_opened: Vec<Card>) {
+        for card in cards_opened {
             self.discrete_gpus_after_the_opening
                 .note_the_gpu_opened_at_a_switch(&card.discrete_gpu().identifier);
             self.cards_on_this_host.push(card);
         }
+    }
+
+    /// 收下为第 `handing_in_number` 次交来的设置开卡的结局：开成的卡先收下（[`Self::take_in_the_cards_opened_at_a_switch`]），再照最近交来、在等的那一组办——
+    /// 开卡为的就是它（次序号相同）、有一张重开到上限都没开成：此刻生效那一组之外开着的卡全部关掉（连同为更早几次交来的设置开成的），记下那一张，
+    /// 之后每一批报成它的 GPU 出错，直到下一次换设置；它点的卡都开着了：换上它（不再点名的卡关掉）；它还点着没开的卡（开卡为的是更早一次交来的，
+    /// 那一次开不成的不报给它）：为它再开一次。没有在等的那一组（之后交来的一组不用开卡、已换上，或告知过一张卡都不参与）：此刻生效那一组之外开着的卡全部关掉。
+    fn take_in_the_outcome_of_the_opening(
+        &mut self,
+        handing_in_number: HandingInNumber,
+        outcome: OutcomeOfTheOpeningAtASwitch<Card>,
+    ) {
+        self.take_in_the_cards_opened_at_a_switch(outcome.opened);
         let Some(settings_waiting) = self.settings_waiting_for_the_opening.take() else {
-            self.close_the_cards(|device| devices_opened_by_this_opening.contains(device));
-            return None;
+            self.close_the_cards_outside_the_settings_in_effect();
+            return;
         };
-        if let Some(not_opened) = outcome.not_opened {
-            if settings_of_the_opening == settings_waiting {
-                self.close_the_cards(|device| devices_opened_by_this_opening.contains(device));
-                self.not_opened_after_the_reopenings_at_the_last_switch = Some(not_opened.clone());
-                return Some(not_opened);
+        if settings_waiting.handing_in_number == handing_in_number {
+            if let Some(not_opened) = outcome.not_opened {
+                self.close_the_cards_outside_the_settings_in_effect();
+                self.not_opened_after_the_reopenings_at_the_last_switch = Some(not_opened);
+                return;
             }
         }
-        let discrete_gpus_to_open = self.discrete_gpus_named_and_not_opened(&settings_waiting);
+        let discrete_gpus_to_open =
+            self.discrete_gpus_named_and_not_opened(&settings_waiting.settings);
         if discrete_gpus_to_open.is_empty() {
-            self.put_the_settings_in_effect(&settings_waiting);
-        } else {
-            self.settings_waiting_for_the_opening = Some(settings_waiting.clone());
-            self.start_the_opening_at_a_switch(settings_waiting, discrete_gpus_to_open);
+            self.put_the_settings_in_effect(&settings_waiting.settings);
+            return;
         }
-        None
+        let waiting_handing_in_number = settings_waiting.handing_in_number;
+        let wait_limit = settings_waiting.settings.wait_limit();
+        self.settings_waiting_for_the_opening = Some(settings_waiting);
+        self.start_the_opening_at_a_switch(
+            waiting_handing_in_number,
+            discrete_gpus_to_open,
+            wait_limit,
+            Vec::new(),
+        );
+    }
+
+    /// 关掉此刻生效那一组没点的开着的卡（[`Self::close_the_cards`]）。
+    fn close_the_cards_outside_the_settings_in_effect(&mut self) {
+        let devices_named_by_the_settings_in_effect =
+            self.devices_named_by_the_settings_in_effect.clone();
+        self.close_the_cards(|device| !devices_named_by_the_settings_in_effect.contains(device));
     }
 
     /// 这一组点到、此刻没开的卡（按点的次序），各带这一组给它的显存上限。
@@ -2333,61 +2647,111 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
             .collect()
     }
 
-    /// 在另一条线程上为 `settings` 开 `discrete_gpus_to_open` 那几张（按给的次序逐张开，开一张同挂载那一刻开卡那一步，[`gpu_opened_under_its_memory_limit`]；
-    /// 开不成就照重开的规矩重开，每次重开之前在那条线程上睡一个间隔；有一张重开到上限都没开成，后面的就不再开）。不等它：调用即返回。
-    /// 起不了那条线程就记成头一张开不成（原因是起不了线程），下一次两批之间交回。
+    /// 起切换开卡那条线程（[`StartOfTheOpeningThread`]）：产品照 std 起；只供测试的那一种照剩下的次数报起不了。
+    fn opening_thread_started(
+        &mut self,
+        body_of_the_thread: impl FnOnce() + Send + 'static,
+    ) -> std::io::Result<JoinHandle<()>> {
+        match &mut self.start_of_the_opening_thread {
+            StartOfTheOpeningThread::ByTheStandardLibrary
+            | StartOfTheOpeningThread::RefusedForTests { refusals_left: 0 } => {
+                std::thread::Builder::new()
+                    .name("singlefs-opening-gpus-at-a-switch".to_string())
+                    .spawn(body_of_the_thread)
+            }
+            StartOfTheOpeningThread::RefusedForTests { refusals_left } => {
+                *refusals_left -= 1;
+                Err(std::io::Error::other("只供测试：照起不了切换开卡线程报"))
+            }
+        }
+    }
+
+    /// 在另一条线程上为第 `handing_in_number` 次交来的设置开 `discrete_gpus_to_open` 那几张（按给的次序逐张开，开一张同挂载那一刻开卡那一步，
+    /// [`gpus_opened_at_a_switch`]；开不成就照重开的规矩重开，每次重开之前在那条线程上睡一个间隔；有一张重开到上限都没开成，后面的就不再开）。
+    /// 不等它：调用即返回。`causes_of_each_start` 是之前几次起不了这条线程的原因。起不了那条线程，就算头一张要开的卡这一次没开成（原因是起不了线程的那个错）：
+    /// 没到重开的上限就隔重开的间隔、在之后的两批之间再起（[`OpeningAtASwitch::ThreadNotStarted`]），到了上限结局就定成头一张开不成。
     fn start_the_opening_at_a_switch(
         &mut self,
-        settings: AdmittedGpuSettings,
+        handing_in_number: HandingInNumber,
         discrete_gpus_to_open: Vec<(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)>,
+        wait_limit: Duration,
+        causes_of_each_start: Vec<GpuFailureCause>,
     ) {
-        let first_device_to_open = discrete_gpus_to_open
-            .first()
-            .map(|(discrete_gpu, _memory_limit)| discrete_gpu.identifier.clone())
-            .expect("只为点到没开的卡的那一组开卡：至少一张");
         let opening_of_a_gpu_at_a_switch = Arc::clone(&self.opening_of_a_gpu_at_a_switch);
         let memory_leaked_in_this_process = self.memory_leaked_in_this_process.clone();
         let reopenings = self.reopenings_of_a_gpu_not_opened;
-        let wait_limit = settings.wait_limit();
+        let called_off = Arc::new(AtomicBool::new(false));
+        let called_off_seen_by_the_thread = Arc::clone(&called_off);
+        let discrete_gpus_opened_by_the_thread = discrete_gpus_to_open.clone();
+        let causes_of_each_start_seen_by_the_thread = causes_of_each_start.clone();
         let (outcome_sender, outcome_receiver) = channel();
-        let started = std::thread::Builder::new()
-            .name("singlefs-opening-gpus-at-a-switch".to_string())
-            .spawn(move || {
-                let mut opening_of_a_gpu = opening_of_a_gpu_at_a_switch.lock().expect(
-                    "开卡那一步只在开卡线程上锁着用：上一次开卡线程 panic 时那一次已经往上报过",
-                );
-                let outcome = gpus_opened_at_a_switch(
-                    &mut *opening_of_a_gpu,
-                    discrete_gpus_to_open,
-                    wait_limit,
-                    reopenings,
-                    &memory_leaked_in_this_process,
-                );
-                drop(opening_of_a_gpu);
-                // 送不出去只有一种：后台 GPU 已经丢了（卸载），开出来的卡没人要，在这里照常丢。
-                let _nobody_takes_this_opening_any_more = outcome_sender.send(outcome);
+        let started = self.opening_thread_started(move || {
+            let mut opening_of_a_gpu = opening_of_a_gpu_at_a_switch.lock().expect(
+                "开卡那一步只在开卡线程上锁着用，开一次卡里的 panic 在锁里就接住了：锁没被毒化",
+            );
+            let outcome = gpus_opened_at_a_switch(
+                &mut *opening_of_a_gpu,
+                discrete_gpus_opened_by_the_thread,
+                wait_limit,
+                reopenings,
+                causes_of_each_start_seen_by_the_thread,
+                &called_off_seen_by_the_thread,
+                &memory_leaked_in_this_process,
+            );
+            drop(opening_of_a_gpu);
+            // 送不出去只有一种：后台 GPU 已经丢了（卸载）或切到了 `cpu`，开出来的卡没人要，在这里照常丢。
+            let _nobody_takes_this_opening_any_more = outcome_sender.send(outcome);
+        });
+        let error = match started {
+            Ok(opening_thread) => {
+                self.opening_at_a_switch = Some(OpeningAtASwitch::UnderWay {
+                    handing_in_number,
+                    outcome_receiver,
+                    opening_thread,
+                    called_off,
+                });
+                return;
+            }
+            Err(error) => error,
+        };
+        let mut causes_of_each_start = causes_of_each_start;
+        causes_of_each_start.push(GpuFailureCause::DeviceOrPipelineDidNotStart {
+            reason: format!("起不了切换开卡的那条线程：{error}"),
+        });
+        let starts_at_most = usize::try_from(reopenings.reopenings_at_most)
+            .map_or(usize::MAX, |reopenings_at_most| {
+                reopenings_at_most.saturating_add(1)
             });
-        self.opening_at_a_switch = Some(match started {
-            Ok(opening_thread) => OpeningAtASwitch::UnderWay {
-                settings,
-                outcome_receiver,
-                opening_thread,
-            },
-            Err(error) => OpeningAtASwitch::ThreadNotStarted {
-                settings,
-                not_opened: GpuNotOpenedAfterTheReopenings {
+        if causes_of_each_start.len() < starts_at_most {
+            self.opening_at_a_switch = Some(OpeningAtASwitch::ThreadNotStarted {
+                handing_in_number,
+                discrete_gpus_to_open,
+                wait_limit,
+                causes_of_each_start,
+                not_started_at: Instant::now(),
+            });
+            return;
+        }
+        let first_device_to_open = discrete_gpus_to_open
+            .first()
+            .map(|(discrete_gpu, _memory_limit)| discrete_gpu.identifier.clone())
+            .expect("只为点到没开的卡的那一组开卡：至少一张");
+        self.opening_at_a_switch = Some(OpeningAtASwitch::EndedWithoutAThread {
+            handing_in_number,
+            outcome: OutcomeOfTheOpeningAtASwitch {
+                opened: Vec::new(),
+                not_opened: Some(GpuNotOpenedAfterTheReopenings {
                     device: first_device_to_open,
-                    causes_of_each_opening: vec![GpuFailureCause::DeviceOrPipelineDidNotStart {
-                        reason: format!("起不了切换开卡的那条线程：{error}"),
-                    }],
-                },
+                    causes_of_each_opening: causes_of_each_start,
+                }),
             },
         });
     }
 
     /// 关掉 `is_closed` 判成要关的开着的卡（用户定案「切换之后不再参与的卡关不关」=「切换时关掉」）：它从开着的那几张挪回没开的那几张（卡清单照列，
     /// 之后再点它就重开），设备丢掉；上面还记着一次等超过上限、没在 GPU 上做完的提交的那张不丢、整张泄漏，这一刻占着的字节记进这个进程的泄漏账
-    /// （同卸载，`gpus_of_this_host::gpus_dropped_leaking_those_still_running`）。参与的卡照它们的标识在剩下的卡里重找位置，次序不变。
+    /// （同卸载，`gpus_of_this_host::gpus_dropped_leaking_those_still_running`），并累计进对外读得到的那一份
+    /// （[`Self::gpus_leaked_when_closed_at_a_switch`]）。参与的卡照它们的标识在剩下的卡里重找位置，次序不变。
     fn close_the_cards(&mut self, is_closed: impl Fn(&GpuDeviceIdentifier) -> bool) {
         let participating_devices: Vec<GpuDeviceIdentifier> = self
             .participating
@@ -2419,6 +2783,8 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         );
         self.memory_leaked_in_this_process
             .note_the_gpus_leaked_when_dropped(&gpus_leaked);
+        self.gpus_leaked_when_closed_at_a_switch
+            .note_the_gpus_leaked(&gpus_leaked);
         self.participating = ParticipatingGpusOnThisHost::of_the_positions(
             participating_devices
                 .iter()
@@ -2443,6 +2809,7 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
             .map(|participating| participating.device.clone())
             .collect();
         self.close_the_cards(|device| !devices_named_by_the_settings.contains(device));
+        self.devices_named_by_the_settings_in_effect = devices_named_by_the_settings;
         let identifiers_on_this_host = self.gpus_opened();
         let participating_by_the_settings =
             ParticipatingGpusOnThisHost::of_the_settings(settings, &identifiers_on_this_host);
@@ -2495,11 +2862,13 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
 
     /// 此刻这组设置下一批分不分得到卡上（与这一批有几条无关）：最近交来的那一组点到的一张没开的卡重开到上限都没开成、换设置时量一张卡的代价出错了，
     /// 或参与的卡因显存上限一张都不剩，就交回是哪一种；分得到交回 `None`（切换点到的卡还在开时照的是交来之前那一组）。
-    /// 挂载那一刻与切换生效那一批开头核心层经 `BackgroundGpuCompute::failure_under_the_settings_handed_in` 读它，不等头一批。
+    /// 两批之间那一问答换上了时带着它（[`LatestSettingsHandedIn::PutInEffect`]），核心层照它判停，不等头一批。
+    /// 告知一张卡都不参与之后（切到 `cpu`）没有生效的那一组，核心层不问它。
     ///
     /// # Panics
     /// 参与的卡一张都不剩，却既没有量代价出错，也没有因显存上限不用的卡：核心层校验过的一组至少点一张卡，换上的那一组点的卡都开着，
-    /// 没参与的只有这两种原因。
+    /// 没参与的只有这两种原因；告知一张卡都不参与（[`Self::use_no_gpu_from_the_next_batch`]）之后、再交来一组换上之前调它（核心层这时生效的选 `cpu`，
+    /// 不问它、不交活）。
     #[must_use]
     pub fn batch_not_split_over_the_gpus(&self) -> Option<BatchNotSplitOverTheGpus> {
         if let Some(not_opened) = &self.not_opened_after_the_reopenings_at_the_last_switch {
@@ -3065,12 +3434,15 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
     }
 }
 
-/// 丢后台 GPU（卸载、丢掉核心层收的把手、测试里展开）：不等还在 GPU 上跑的提交——
+/// 丢后台 GPU（卸载、丢掉核心层收的把手、测试里展开）：会话锁外为切换开卡的那一次叫停（开卡线程不再开下一次），不等还在 GPU 上跑的提交——
 /// 还记着一次等超过上限、没做完的提交的卡整张泄漏（设备、队列、管线都不析构：wgpu-core 30.0.1 的 `Queue` 析构调 `wait_for_idle`，
 /// 即 Vulkan 的 `vkQueueWaitIdle`，不带上限），别的卡照常丢（`gpus_of_this_host::gpus_dropped_leaking_those_still_running`）。
 /// 泄漏了哪几张、各占着多少字节报给调用方（[`BackgroundGpuOverTheCards::gpus_leaked_when_dropped`]），并记进进程里的那一份，新一次挂载先扣掉。
 impl<Card: CardOfTheBackgroundGpu> Drop for BackgroundGpuOverTheCards<Card> {
     fn drop(&mut self) {
+        if let Some(opening) = &self.opening_at_a_switch {
+            opening.call_off();
+        }
         let cards_on_this_host = std::mem::take(&mut self.cards_on_this_host);
         let gpus_leaked = gpus_dropped_leaking_those_still_running(
             cards_on_this_host,
@@ -4602,8 +4974,10 @@ mod tests {
             background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                 WaitForTheOpeningAtASwitch::UntilItEnds
             ),
-            None,
-            "开成了：没有开不成的卡要报"
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            },
+            "开成了：换上，这一组用得上"
         );
         assert_eq!(
             openings_so_far(),
@@ -4631,7 +5005,9 @@ mod tests {
             background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                 WaitForTheOpeningAtASwitch::UntilItEnds
             ),
-            None
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            }
         );
         assert_eq!(openings_so_far().len(), 1, "开着的卡不重开");
     }
@@ -4669,7 +5045,7 @@ mod tests {
             background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                 WaitForTheOpeningAtASwitch::NotAtAll
             ),
-            None,
+            LatestSettingsHandedIn::StillOpeningItsCards,
             "还在开：不等它"
         );
         let (outcome_while_opening, devices_given_work_while_opening) =
@@ -4690,8 +5066,10 @@ mod tests {
             background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                 WaitForTheOpeningAtASwitch::UntilItEnds
             ),
-            None,
-            "开成了：没有开不成的卡要报"
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            },
+            "开成了：换上，这一组用得上"
         );
         let (outcome_after_the_opening, devices_given_work_after_the_opening) =
             checksum_batch_and_the_devices_given_work(&mut background_gpu);
@@ -4741,8 +5119,10 @@ mod tests {
             background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                 WaitForTheOpeningAtASwitch::UntilItEnds
             ),
-            None,
-            "第三次开成：没有开不成的卡要报"
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            },
+            "第三次开成：换上"
         );
         assert_eq!(
             *cards_handed_to_the_opening
@@ -4763,7 +5143,7 @@ mod tests {
     }
 
     /// 用户定案「3 次都不成：GC 停下，报警并报出哪张卡、每次开不成的原因，等人处理，不自动再试」：切换只点「0000:41:00.0」，它头一次与 3 次重开都开不成——
-    /// 两批之间那一问交回这张卡与 4 次的原因（只交这一次，之后不再开），之后那一批报成这张卡的 GPU 出错（带每一次的原因）、一条活都不分出去；
+    /// 两批之间那一问答开不成、带这张卡与 4 次的原因（再问照答，之后不再开），之后那一批报成这张卡的 GPU 出错（带每一次的原因）、一条活都不分出去；
     /// 照旧只开着切换之前那一张。下一次换设置（切回「0000:01:00.0」）解除它，那一批照常只分给它。
     #[test]
     fn the_switch_to_only_a_card_that_does_not_open_is_reported_as_its_gpu_failure_and_gives_it_no_work(
@@ -4786,15 +5166,15 @@ mod tests {
             background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                 WaitForTheOpeningAtASwitch::UntilItEnds
             ),
-            Some(not_opened.clone()),
+            LatestSettingsHandedIn::NotOpened(not_opened.clone()),
             "头一次与 3 次重开都开不成：报出这张卡与每一次的原因"
         );
         assert_eq!(
             background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                 WaitForTheOpeningAtASwitch::UntilItEnds
             ),
-            None,
-            "只报这一次"
+            LatestSettingsHandedIn::NotOpened(not_opened.clone()),
+            "再问照答开不成、不再开"
         );
         assert_eq!(
             *cards_handed_to_the_opening
@@ -4889,7 +5269,9 @@ mod tests {
             background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                 WaitForTheOpeningAtASwitch::UntilItEnds
             ),
-            None
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            }
         );
         assert_eq!(
             *cards_handed_to_the_opening
@@ -4949,6 +5331,7 @@ mod tests {
         let drops_counted = Arc::new(std::sync::atomic::AtomicU32::new(0));
         let drops_counted_by_the_card = Arc::clone(&drops_counted);
         let opening_at_the_switch = opening_of_a_card_within_the_limit(
+            WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
             move |discrete_gpu: &DiscreteGpuOnThisHost,
                   memory_limit,
                   _wait_limit,
@@ -4979,7 +5362,7 @@ mod tests {
             background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
                 WaitForTheOpeningAtASwitch::UntilItEnds
             ),
-            Some(GpuNotOpenedAfterTheReopenings {
+            LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
                 device: card_identifier("0000:41:00.0"),
                 causes_of_each_opening: vec![failure_of_the_cost_measurement_in_the_test(); 4],
             }),
@@ -5002,6 +5385,825 @@ mod tests {
         );
     }
 
+    /// 本机三张独显：「0000:01:00.0」挂载那一刻开着（给它的显存上限 `memory_limit_of_the_card_at_one`；`left_out_at_the_mount` 时它因显存上限不参与、
+    /// 参与的卡一张都不剩），「0000:41:00.0」「0000:81:00.0」没开。切换点到没开的卡时开它那一步交 `opening_at_the_switch`，重开照用例的规矩
+    /// （间隔 0、至多重开 3 次）；开卡那一步被交了哪几张卡记在交回的那一份里。
+    fn background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+        opening_at_the_switch: impl FnMut(
+                &DiscreteGpuOnThisHost,
+                Option<GpuMemoryInBytes>,
+                Duration,
+                CostAtTheOpening,
+            ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
+            + Send
+            + 'static,
+        memory_limit_of_the_card_at_one: Option<u64>,
+        left_out_at_the_mount: bool,
+    ) -> (
+        BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
+        Arc<Mutex<Vec<String>>>,
+    ) {
+        let card_at_one = CardComputedOnTheCpuForTests::of_the_address(
+            "0000:01:00.0",
+            memory_limit_of_the_card_at_one,
+            0,
+        );
+        let opened = card_at_one.discrete_gpu.clone();
+        let not_opened_card =
+            |pci_bus_address: &str, position_in_the_enumeration: usize| DiscreteGpuOnThisHost {
+                position_in_the_enumeration,
+                identifier: card_identifier(pci_bus_address),
+                pci_bus_address: pci_bus_address.to_string(),
+                name: format!("cpu card at {pci_bus_address} not opened at the mount"),
+                total_memory: GpuMemoryInBytes(EIGHT_GIBIBYTES),
+            };
+        let (participating, left_out) = if left_out_at_the_mount {
+            (
+                ParticipatingGpusOnThisHost::of_the_positions(Vec::new()),
+                vec![GpuLeftOutUnderItsMemoryLimit {
+                    device: card_identifier("0000:01:00.0"),
+                    memory_limit: GpuMemoryInBytes(memory_limit_of_the_card_at_one.unwrap_or(0)),
+                    smallest_piece_bytes: bytes_of_the_smallest_piece_every_participating_gpu_holds(
+                    ),
+                    bytes_already_held_on_the_device: 0,
+                }],
+            )
+        } else {
+            (
+                ParticipatingGpusOnThisHost::of_the_positions(vec![0]),
+                Vec::new(),
+            )
+        };
+        let (opening_noting_the_cards, cards_handed_to_the_opening) =
+            opening_noting_the_cards_handed_to_it(opening_at_the_switch);
+        let background_gpu = BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
+            vec![card_at_one],
+            DiscreteGpusAfterTheOpening {
+                opened: vec![opened],
+                not_opened_for_not_taking_part: vec![
+                    not_opened_card("0000:41:00.0", 1),
+                    not_opened_card("0000:81:00.0", 2),
+                ],
+            },
+            Vec::new(),
+            participating,
+            left_out,
+            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            Box::new(DiscreteGpusPresentForTests(Arc::new(Mutex::new(vec![
+                card_identifier("0000:01:00.0"),
+                card_identifier("0000:41:00.0"),
+                card_identifier("0000:81:00.0"),
+            ])))),
+            opening_noting_the_cards,
+            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+        );
+        (background_gpu, cards_handed_to_the_opening)
+    }
+
+    /// 核心层照外壳答的卡清单校验点 `addresses` 那几张（按这个次序）、显存上限 `memory_limits` 的切换。
+    fn switch_naming_the_cards_admitted_against(
+        background_gpu: &BackgroundGpuOverTheCards<CardComputedOnTheCpuForTests>,
+        addresses: &[&str],
+        memory_limits: Option<Vec<u64>>,
+    ) -> AdmittedGpuSettings {
+        admit_gpu_settings_against_the_inventory(
+            &BackgroundComputeSettings {
+                background_compute: BackgroundComputeBackend::Gpu,
+                background_gpu_devices: Some(
+                    addresses
+                        .iter()
+                        .map(|address| card_identifier(address))
+                        .collect(),
+                ),
+                background_gpu_memory_limit: memory_limits
+                    .map(|limits| limits.into_iter().map(GpuMemoryInBytes).collect()),
+                background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+            },
+            background_gpu,
+        )
+        .expect("卡清单列着这几张：切换照收")
+    }
+
+    fn openings_made_so_far(cards_handed_to_the_opening: &Arc<Mutex<Vec<String>>>) -> usize {
+        cards_handed_to_the_opening
+            .lock()
+            .expect("用例没在别处 panic 过")
+            .len()
+    }
+
+    /// 开卡那一步：等用例放开（至多 20 秒，没放开就交回开不成，用例照断言红、不一直挂着）再开成交来的那张卡。
+    fn opening_that_opens_once_released(
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
+            opening_released
+                .recv_timeout(Duration::from_secs(20))
+                .map_err(|_not_released| reason_the_card_does_not_open_at_the_switch())?;
+            opening_that_always_opens(discrete_gpu, memory_limit, wait_limit, cost_at_the_opening)
+        }
+    }
+
+    /// 第 4 条「停着的时候收到任何一次通过校验的切换请求……当场解除停下，从下一批起用新设置」：挂载那一刻「0000:01:00.0」因显存上限不参与、
+    /// 参与的卡一张都不剩；切换只点没开的「0000:41:00.0」（不给上限）——开卡的这段时间那一问答还在开卡（不答切换之前那一组的「一张都不剩」），
+    /// 开完换上之后答换上了、这一组用得上，参与的只有「0000:41:00.0」。
+    #[test]
+    fn the_settings_whose_card_is_still_opening_are_answered_as_still_opening_and_as_usable_once_put_in_effect(
+    ) {
+        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
+        let (mut background_gpu, _cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                opening_that_opens_once_released(opening_released),
+                Some(64),
+                true,
+            );
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::NotAtAll
+            ),
+            LatestSettingsHandedIn::StillOpeningItsCards,
+            "还在开卡：这一组还没生效，不答切换之前那一组用不用得上"
+        );
+        release_the_opening
+            .send(())
+            .expect("开卡那一步还在等放开：收的那一头还在");
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            },
+            "开完换上：这一组用得上"
+        );
+        assert_eq!(
+            participating_devices_of(&background_gpu),
+            vec![card_identifier("0000:41:00.0")]
+        );
+    }
+
+    /// 第 4 条「挂着切换到参与的卡一张都不剩的设置……报的是挂载那一刻同一个成员」用于点没开的卡：切换只点没开的「0000:41:00.0」、给它 64 字节的上限
+    /// （连最小的一片都装不下）——开完换上那一问答换上了，连同这一组下一批分不到卡上（参与的卡因显存上限一张都不剩，晾下的是「0000:41:00.0」）。
+    #[test]
+    fn the_switch_put_in_effect_after_the_opening_with_every_card_left_out_is_answered_with_every_card_left_out(
+    ) {
+        let (mut background_gpu, _cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                opening_that_always_opens,
+                None,
+                false,
+            );
+        let to_the_card_at_forty_one_under_a_tiny_limit = switch_naming_the_cards_admitted_against(
+            &background_gpu,
+            &["0000:41:00.0"],
+            Some(vec![64]),
+        );
+        background_gpu
+            .use_the_settings_from_the_next_batch(&to_the_card_at_forty_one_under_a_tiny_limit);
+        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+            WaitForTheOpeningAtASwitch::UntilItEnds,
+        );
+        assert!(
+            matches!(
+                &answered,
+                LatestSettingsHandedIn::PutInEffect {
+                    not_split_over_the_gpus: Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(left_out)),
+                } if left_out.iter().map(|gpu| gpu.device.clone()).collect::<Vec<_>>()
+                    == vec![card_identifier("0000:41:00.0")]
+            ),
+            "换上了，连同这一组一张卡都用不上：{answered:?}"
+        );
+    }
+
+    /// 第 2 条「切换之后不再参与的卡，切换生效时关掉」：切换点「0000:41:00.0」，它在开（等用例放开）；又来一次切换只点「0000:81:00.0」。放开之后
+    /// 「0000:41:00.0」开成，接着为最近那一组开「0000:81:00.0」，它头一次与 3 次重开都开不成——此刻生效那一组（挂载那一组）之外开着的卡全部关掉：
+    /// 为被盖掉的那一组开成的「0000:41:00.0」关掉（替身卡析构一次），开着的只剩「0000:01:00.0」。
+    #[test]
+    fn the_card_opened_for_a_superseded_switch_is_closed_when_the_latest_switch_does_not_open() {
+        let (release_the_card_at_forty_one, card_at_forty_one_released) =
+            std::sync::mpsc::channel::<()>();
+        let drops_of_the_card_at_forty_one = Arc::new(std::sync::atomic::AtomicU32::new(0));
+        let drops_noted_by_the_opening = Arc::clone(&drops_of_the_card_at_forty_one);
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
+                    Err(reason_the_card_does_not_open_at_the_switch())
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
+        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+            WaitForTheOpeningAtASwitch::UntilItEnds,
+        );
+        assert!(
+            matches!(
+                &answered,
+                LatestSettingsHandedIn::NotOpened(not_opened)
+                    if not_opened.device == card_identifier("0000:81:00.0")
+            ),
+            "最近那一组点的「0000:81:00.0」开不成：{answered:?}"
+        );
+        assert_eq!(
+            background_gpu.gpus_opened(),
+            vec![card_identifier("0000:01:00.0")],
+            "此刻生效那一组之外开着的卡全部关掉"
+        );
+        assert_eq!(
+            drops_of_the_card_at_forty_one.load(std::sync::atomic::Ordering::SeqCst),
+            1,
+            "为被盖掉的那一组开成的卡关掉了"
+        );
+        assert_eq!(
+            participating_devices_of(&background_gpu),
+            vec![card_identifier("0000:01:00.0")]
+        );
+    }
+
+    /// 开卡的结局按交设置的次序认，不按内容比：切换只点「0000:41:00.0」，头一次与 3 次重开都开不成；开卡线程开完 4 次之后、两批之间那一问还没问到时，
+    /// 同一组设置又交了一次——那一问不报上一次的 4 个原因，照最近那一次重开（一共开 8 次），再报这一次的 4 个原因。
+    #[test]
+    fn the_same_settings_handed_in_again_before_the_poll_reopen_the_card_instead_of_taking_the_earlier_failure(
+    ) {
+        let (opened_four_times, every_opening_made) = std::sync::mpsc::channel::<()>();
+        let openings_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
+        let openings_counted = Arc::clone(&openings_made);
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                move |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
+                    let opening =
+                        openings_counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
+                    if opening == 4 {
+                        // 收的那一头还在：用例在等这一声。
+                        let _signal_sent = opened_four_times.send(());
+                    }
+                    Err(reason_the_card_does_not_open_at_the_switch())
+                },
+                None,
+                false,
+            );
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        every_opening_made
+            .recv_timeout(Duration::from_secs(20))
+            .expect("开卡线程 20 秒内开完 4 次");
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+            WaitForTheOpeningAtASwitch::UntilItEnds,
+        );
+        assert_eq!(
+            openings_made_so_far(&cards_handed_to_the_opening),
+            8,
+            "又交的那一次照它重开：一共开 8 次"
+        );
+        assert!(
+            matches!(
+                &answered,
+                LatestSettingsHandedIn::NotOpened(not_opened)
+                    if not_opened.causes_of_each_opening.len() == 4
+            ),
+            "报的是又交那一次的 4 个原因：{answered:?}"
+        );
+    }
+
+    /// 交来的一组点的卡都开着就当场换上，在开的那一次为更早一次交来的设置开成的卡收下时关掉：切换点没开的「0000:41:00.0」，它在开（等用例放开）；
+    /// 又交来只点开着的「0000:01:00.0」的一组——那一问当场答换上了；放开之后「0000:41:00.0」开成、收下，此刻生效那一组没点它，关掉。
+    #[test]
+    fn the_settings_handed_in_while_an_earlier_opening_runs_with_their_cards_open_take_effect_at_once_and_the_card_opened_later_is_closed(
+    ) {
+        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
+        let (mut background_gpu, _cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                opening_that_opens_once_released(opening_released),
+                None,
+                false,
+            );
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        let back_to_the_card_at_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:01:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        background_gpu.use_the_settings_from_the_next_batch(&back_to_the_card_at_one);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::NotAtAll
+            ),
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            },
+            "点的卡都开着：当场换上"
+        );
+        release_the_opening
+            .send(())
+            .expect("开卡那一步还在等放开：收的那一头还在");
+        drop(release_the_opening);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            }
+        );
+        assert_eq!(
+            background_gpu.gpus_opened(),
+            vec![card_identifier("0000:01:00.0")],
+            "为更早一次交来的设置开成的卡收下时关掉"
+        );
+    }
+
+    /// 上一次切换的卡还在开时又交来一组，等那一次开完再看，不为它另起一条：切换点没开的「0000:41:00.0」，它在开（等用例放开）；又交来点
+    /// 「0000:41:00.0」「0000:81:00.0」的一组——放开之后收下「0000:41:00.0」，再为最近那一组只开还没开的「0000:81:00.0」，开卡那一步按次序只被交这两张、各一次。
+    #[test]
+    fn the_settings_handed_in_while_an_earlier_opening_runs_wait_for_it_and_open_only_the_cards_still_not_opened(
+    ) {
+        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
+        let mut open_once_released = opening_that_opens_once_released(opening_released);
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
+                    if discrete_gpu.pci_bus_address == "0000:41:00.0" {
+                        return open_once_released(
+                            discrete_gpu,
+                            memory_limit,
+                            wait_limit,
+                            cost_at_the_opening,
+                        );
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
+        release_the_opening
+            .send(())
+            .expect("开卡那一步还在等放开：收的那一头还在");
+        drop(release_the_opening);
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
+            vec!["0000:41:00.0".to_string(), "0000:81:00.0".to_string()],
+            "每张卡只开一次"
+        );
+        assert_eq!(
+            participating_devices_of(&background_gpu),
+            vec![
+                card_identifier("0000:41:00.0"),
+                card_identifier("0000:81:00.0")
+            ]
+        );
+    }
+
+    /// 用户定案「切到 cpu 时开着的卡全部关掉」：挂载开着「0000:01:00.0」「0000:41:00.0」，告知一张卡都不参与——开着的卡为零（「0000:41:00.0」那张替身卡析构一次），
+    /// 卡清单照列两张；再交来只点「0000:41:00.0」的一组，它当没开的卡开（开卡那一步走一次），开完换上。
+    #[test]
+    fn no_gpu_taking_part_closes_every_open_card_and_the_next_settings_open_their_cards_again() {
+        let (mut background_gpu, cards_handed_to_the_opening, drops_of_the_card_at_forty_one) =
+            background_gpu_with_both_cards_opened_at_the_mount(
+                opening_that_always_opens,
+                |card| card,
+                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            );
+        background_gpu.use_no_gpu_from_the_next_batch();
+        assert!(
+            background_gpu.gpus_opened().is_empty(),
+            "开着的卡为零：{:?}",
+            background_gpu.gpus_opened()
+        );
+        assert_eq!(
+            drops_of_the_card_at_forty_one.load(std::sync::atomic::Ordering::SeqCst),
+            1
+        );
+        assert_eq!(
+            background_gpu.devices_with_total_memory().len(),
+            2,
+            "卡清单照列关掉的卡：切回 gpu 可以再点"
+        );
+        let only_the_card_at_forty_one =
+            switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
+        background_gpu.use_the_settings_from_the_next_batch(&only_the_card_at_forty_one);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            }
+        );
+        assert_eq!(
+            openings_made_so_far(&cards_handed_to_the_opening),
+            1,
+            "切回之后当没开的卡开"
+        );
+        assert_eq!(
+            participating_devices_of(&background_gpu),
+            vec![card_identifier("0000:41:00.0")]
+        );
+    }
+
+    /// 开卡那一步：每被调一次先往 `opening_begun` 送一声，等用例放开（至多 20 秒）之后交回开不成。
+    fn opening_that_does_not_open_once_released(
+        opening_begun: std::sync::mpsc::Sender<()>,
+        opening_released: std::sync::mpsc::Receiver<()>,
+    ) -> impl FnMut(
+        &DiscreteGpuOnThisHost,
+        Option<GpuMemoryInBytes>,
+        Duration,
+        CostAtTheOpening,
+    ) -> Result<CardComputedOnTheCpuForTests, GpuFailureCause>
+           + Send
+           + 'static {
+        move |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
+            // 收的那一头在用例里一直拿着。
+            let _signal_sent = opening_begun.send(());
+            let _released_or_not = opening_released.recv_timeout(Duration::from_secs(20));
+            Err(reason_the_card_does_not_open_at_the_switch())
+        }
+    }
+
+    /// 用户定案「切到 cpu 时……丢掉在等的那一组与还没取走的开卡结局」：切换点没开的「0000:41:00.0」，头一次开卡在开（等用例放开）时告知一张卡都不参与——
+    /// 开卡线程开完这一次就不再开（重开规矩本来还许 3 次），开卡那一步一共只被调一次；开着的卡为零。
+    #[test]
+    fn no_gpu_taking_part_while_a_card_opens_calls_off_the_reopenings() {
+        let (opening_begun, every_opening_begun) = std::sync::mpsc::channel::<()>();
+        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                opening_that_does_not_open_once_released(opening_begun, opening_released),
+                None,
+                false,
+            );
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        every_opening_begun
+            .recv_timeout(Duration::from_secs(20))
+            .expect("开卡线程 20 秒内开始头一次开卡");
+        background_gpu.use_no_gpu_from_the_next_batch();
+        release_the_opening
+            .send(())
+            .expect("开卡那一步还在等放开：收的那一头还在");
+        drop(release_the_opening);
+        // 开卡线程从头一次开卡之前到开完都锁着这一步：拿到锁就是它开完了。
+        drop(
+            background_gpu
+                .opening_of_a_gpu_at_a_switch
+                .lock()
+                .expect("开一次卡里的 panic 在锁里就接住了：锁没被毒化"),
+        );
+        assert_eq!(
+            openings_made_so_far(&cards_handed_to_the_opening),
+            1,
+            "叫停之后不再开"
+        );
+        assert!(background_gpu.gpus_opened().is_empty(), "开着的卡为零");
+    }
+
+    /// 卸载时叫停切换开卡线程：切换点没开的「0000:41:00.0」，头一次开卡在开（等用例放开）时丢掉后台 GPU——开卡线程开完这一次就不再开，开卡那一步一共只被调一次。
+    #[test]
+    fn the_opening_at_a_switch_under_way_at_the_drop_opens_no_more() {
+        let (opening_begun, every_opening_begun) = std::sync::mpsc::channel::<()>();
+        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                opening_that_does_not_open_once_released(opening_begun, opening_released),
+                None,
+                false,
+            );
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        every_opening_begun
+            .recv_timeout(Duration::from_secs(20))
+            .expect("开卡线程 20 秒内开始头一次开卡");
+        let opening_step_kept_by_the_test =
+            Arc::clone(&background_gpu.opening_of_a_gpu_at_a_switch);
+        drop(background_gpu);
+        release_the_opening
+            .send(())
+            .expect("开卡那一步还在等放开：收的那一头还在");
+        drop(release_the_opening);
+        // 开卡线程从头一次开卡之前到开完都锁着这一步：拿到锁就是它开完了。
+        drop(
+            opening_step_kept_by_the_test
+                .lock()
+                .expect("开一次卡里的 panic 在锁里就接住了：锁没被毒化"),
+        );
+        assert_eq!(
+            openings_made_so_far(&cards_handed_to_the_opening),
+            1,
+            "卸载之后不再开"
+        );
+    }
+
+    /// 起不了开卡线程也照重开的规矩（间隔之后再起，至多重开 3 次）：头一次起不了，两批之间那一问到了间隔再起、开成——照常换上，参与的只有「0000:41:00.0」。
+    #[test]
+    fn the_opening_thread_not_started_once_is_started_again_and_the_switch_takes_effect() {
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                opening_that_always_opens,
+                None,
+                false,
+            );
+        background_gpu.refuse_to_start_the_opening_thread_for_tests(1);
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            },
+            "再起一次就开成了"
+        );
+        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 1);
+        assert_eq!(
+            participating_devices_of(&background_gpu),
+            vec![card_identifier("0000:41:00.0")]
+        );
+    }
+
+    /// 起不了开卡线程次次都起不了：头一次与 3 次再起都不成，那一问答开不成，带头一张要开的卡与 4 个原因（1 + 重开次数，每次一个），开卡那一步一次都没被调。
+    #[test]
+    fn the_opening_thread_never_started_is_reported_with_one_cause_for_each_start() {
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                opening_that_always_opens,
+                None,
+                false,
+            );
+        background_gpu.refuse_to_start_the_opening_thread_for_tests(u32::MAX);
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+            WaitForTheOpeningAtASwitch::UntilItEnds,
+        );
+        assert!(
+            matches!(
+                &answered,
+                LatestSettingsHandedIn::NotOpened(not_opened)
+                    if not_opened.device == card_identifier("0000:41:00.0")
+                        && not_opened.causes_of_each_opening.len() == 4
+            ),
+            "1 + 重开次数个原因：{answered:?}"
+        );
+        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 0);
+    }
+
+    /// 开卡里 panic 了算那一次没开成、照重开：切换点到的「0000:41:00.0」头一次开卡 panic、第二次开成——照常换上，开卡那一步一共被调 2 次。
+    #[test]
+    fn an_opening_panicking_once_is_reopened_and_the_switch_takes_effect() {
+        let openings_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
+        let openings_counted = Arc::clone(&openings_made);
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                move |discrete_gpu, memory_limit, wait_limit, cost_at_the_opening| {
+                    let opening =
+                        openings_counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
+                    assert!(opening > 1, "替身：头一次开卡 panic");
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
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        assert_eq!(
+            background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                WaitForTheOpeningAtASwitch::UntilItEnds
+            ),
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            }
+        );
+        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 2);
+        assert_eq!(
+            participating_devices_of(&background_gpu),
+            vec![card_identifier("0000:41:00.0")]
+        );
+    }
+
+    /// 开卡次次 panic：重开到上限之后那一问答开不成，带 4 个原因，每个都带着 panic 的原话；两批之间那一问不再 panic。
+    #[test]
+    fn an_opening_panicking_every_time_is_reported_after_the_reopenings_with_every_cause() {
+        let (mut background_gpu, cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
+                    panic!("替身：开卡次次 panic")
+                },
+                None,
+                false,
+            );
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+            WaitForTheOpeningAtASwitch::UntilItEnds,
+        );
+        assert_eq!(
+            answered,
+            LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
+                device: card_identifier("0000:41:00.0"),
+                causes_of_each_opening: vec![
+                    GpuFailureCause::DeviceOrPipelineDidNotStart {
+                        reason: "开卡 panic 了：替身：开卡次次 panic".to_string(),
+                    };
+                    4
+                ],
+            })
+        );
+        assert_eq!(openings_made_so_far(&cards_handed_to_the_opening), 4);
+    }
+
+    /// 同一张卡前一次开卡没回来就不再起下一次（算这一次又没开成、照间隔往下数）：切换点到的「0000:41:00.0」开卡那一步是产品那一份
+    /// （`opening_of_a_card_within_the_limit`，一次至多等 50 毫秒），起这张卡那一件挂着直到用例放开——头一次到点算没开成，之后 3 次重开都因前一次没回来不起，
+    /// 起卡那一件一共只被起一次（同一张卡上同时在开的线程至多一条），那一问答开不成、4 个原因。
+    #[test]
+    fn an_earlier_opening_of_the_card_not_returned_keeps_the_reopenings_from_starting_another() {
+        const OPENING_LIMIT_OF_THE_TEST: Duration = Duration::from_millis(50);
+        let (release_the_start, start_released) = std::sync::mpsc::channel::<()>();
+        let start_released = Arc::new(Mutex::new(start_released));
+        let starts_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
+        let starts_counted = Arc::clone(&starts_made);
+        let opening_at_the_switch = opening_of_a_card_within_the_limit(
+            OPENING_LIMIT_OF_THE_TEST,
+            move |discrete_gpu: &DiscreteGpuOnThisHost,
+                  memory_limit,
+                  _wait_limit,
+                  bytes_leaked_by_earlier_mounts| {
+                let card = CardComputedOnTheCpuForTests::of_the_address(
+                    &discrete_gpu.pci_bus_address,
+                    memory_limit.map(|memory_limit| memory_limit.0),
+                    bytes_leaked_by_earlier_mounts,
+                );
+                let start_released_seen_by_the_start = Arc::clone(&start_released);
+                let starts_counted_by_the_start = Arc::clone(&starts_counted);
+                move || {
+                    starts_counted_by_the_start.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
+                    let _released_or_not = start_released_seen_by_the_start
+                        .lock()
+                        .expect("用例没在别处 panic 过")
+                        .recv_timeout(Duration::from_secs(20));
+                    Ok(card)
+                }
+            },
+            GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        );
+        let (mut background_gpu, _cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                opening_at_the_switch,
+                None,
+                false,
+            );
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        let answered = background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+            WaitForTheOpeningAtASwitch::UntilItEnds,
+        );
+        let starts_while_the_first_hangs = starts_made.load(std::sync::atomic::Ordering::SeqCst);
+        release_the_start
+            .send(())
+            .expect("起卡那一件还在等放开：收的那一头还在");
+        assert_eq!(
+            starts_while_the_first_hangs, 1,
+            "前一次没回来：同一张卡上不再起"
+        );
+        assert_eq!(
+            answered,
+            LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
+                device: card_identifier("0000:41:00.0"),
+                causes_of_each_opening: vec![
+                    GpuFailureCause::WaitForTheGpuExceededTheLimit {
+                        wait_limit: OPENING_LIMIT_OF_THE_TEST,
+                    },
+                    opening_not_started_while_an_earlier_one_has_not_returned(),
+                    opening_not_started_while_an_earlier_one_has_not_returned(),
+                    opening_not_started_while_an_earlier_one_has_not_returned(),
+                ],
+            })
+        );
+    }
+
+    /// 切换关卡时有提交还在跑的卡照卸载同一个规矩泄漏，另累计进对外读得到的那一份：「0000:41:00.0」上记着一次等超过上限、还在跑的提交（占 600 字节），
+    /// 切换不再点它——切换关卡的泄漏报告里读得到它与 600 字节。
+    #[test]
+    fn the_card_leaked_when_closed_at_a_switch_is_read_from_the_leaks_of_the_switches() {
+        let (mut background_gpu, _cards_handed_to_the_opening, _drops_of_the_card_at_forty_one) =
+            background_gpu_with_both_cards_opened_at_the_mount(
+                opening_that_always_opens,
+                |card| card.with_a_submission_running_past_its_wait(600),
+                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            );
+        let leaks_of_the_switches = background_gpu.gpus_leaked_when_closed_at_a_switch();
+        assert!(leaks_of_the_switches.gpus_leaked().is_empty());
+        let only_the_card_at_one =
+            switch_naming_one_card_admitted_against(&background_gpu, "0000:01:00.0");
+        background_gpu.use_the_settings_from_the_next_batch(&only_the_card_at_one);
+        assert_eq!(
+            leaks_of_the_switches.gpus_leaked(),
+            vec![GpuLeakedWhenDropped {
+                device: card_identifier("0000:41:00.0"),
+                bytes_held_on_the_device: 600,
+                bytes_leaked_by_earlier_mounts: 0,
+            }]
+        );
+    }
+
+    /// 产品那一份实现（两份核那一层是外壳）在两批之间那一问里不等开卡：切换点到的「0000:41:00.0」在开（等用例放开），核心层那一问照常交回、答还在开卡。
+    #[test]
+    fn the_product_implementation_asked_between_two_batches_does_not_wait_for_the_opening() {
+        let (release_the_opening, opening_released) = std::sync::mpsc::channel::<()>();
+        let (mut background_gpu, _cards_handed_to_the_opening) =
+            background_gpu_with_the_card_at_one_opened_and_two_not_opened(
+                opening_that_opens_once_released(opening_released),
+                None,
+                false,
+            );
+        let to_the_card_at_forty_one =
+            switch_naming_the_cards_admitted_against(&background_gpu, &["0000:41:00.0"], None);
+        background_gpu.use_the_settings_from_the_next_batch(&to_the_card_at_forty_one);
+        let mut implementation =
+            crate::background_gpu_compute::BackgroundGpuComputeThroughBothKernels::with_no_entry_uncertain(
+                background_gpu,
+            );
+        let answered = singlefs_core::background_compute::BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
+            &mut implementation,
+        );
+        release_the_opening
+            .send(())
+            .expect("开卡那一步还在等放开：收的那一头还在");
+        assert_eq!(
+            answered,
+            singlefs_core::background_compute::LatestGpuSettingsHandedIn::StillOpeningTheirCards
+        );
+    }
+
     /// 外壳上：校验和那一段一条 1000 字节的单元，头一次交等超过上限，重交之前等前一次那一等又超时 2 回、第 3 回等到再交、做完——
     /// 这一段照常交回、与 CPU 参照相同，这张卡上交了 2 次，这一段记下重交 1 次（只数真交出去的）。
     #[test]
````

### crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs b/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs
index ee61c56a2126a6bc42d2de8862b08579a575a0e8..617b03fcb73a26485fa4b05473da5e6b5fa4bac1 100644
--- a/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs
+++ b/crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs
@@ -68,9 +68,9 @@ use singlefs_core::background_compute::{
     BackgroundComputeOfOneBatch, BackgroundComputePathTaken, BackgroundComputeSettings,
     BackgroundGpuCompute, BackgroundGpuComputeFailed, BackgroundGpuComputeHandle,
     ClusterSegmentEntryKey, GpuDeviceIdentifier, GpuEntryDecision, GpuInventory, GpuMemoryInBytes,
-    GpuSubmissionsOfOneSegment, GpuSubmissionsOfTheBatch, ReasonTheBatchRanOnTheCpu,
-    RegionRecordToClassifyOnTheGpu, UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
-    BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+    GpuSubmissionsOfOneSegment, GpuSubmissionsOfTheBatch, LatestGpuSettingsHandedIn,
+    ReasonTheBatchRanOnTheCpu, RegionRecordToClassifyOnTheGpu, UnitChecksumOutcome,
+    UnitChecksumToVerifyOnTheGpu, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
 };
 use singlefs_core::block_device::{
     BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
@@ -3024,9 +3024,13 @@ impl<Inner: BackgroundGpuCompute> BackgroundGpuCompute for GpuComputeRejudgedOnT
         self.inner.use_the_settings_from_the_next_batch(settings);
     }
 
+    fn use_no_gpu_from_the_next_batch(&mut self) {
+        self.inner.use_no_gpu_from_the_next_batch();
+    }
+
     fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
         &mut self,
-    ) -> Option<BackgroundGpuComputeFailed> {
+    ) -> LatestGpuSettingsHandedIn {
         self.inner
             .put_the_gpus_opened_since_the_settings_were_handed_in_in_effect()
     }
@@ -7789,9 +7793,13 @@ impl<Inner: BackgroundGpuCompute> BackgroundGpuCompute for GpuComputeTimingTheSt
         self.inner.use_the_settings_from_the_next_batch(settings);
     }
 
+    fn use_no_gpu_from_the_next_batch(&mut self) {
+        self.inner.use_no_gpu_from_the_next_batch();
+    }
+
     fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
         &mut self,
-    ) -> Option<BackgroundGpuComputeFailed> {
+    ) -> LatestGpuSettingsHandedIn {
         self.inner
             .put_the_gpus_opened_since_the_settings_were_handed_in_in_effect()
     }
````

### crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs b/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
index 3cc9d49bc9943074765272af4c83bcec300f3dce..97ed9ca04745c02d5399250662f536f608cced14 100644
--- a/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
+++ b/crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs
@@ -7,7 +7,9 @@
 //! （开卡按卡串行、只开参与的卡，卡数是参与的卡数；每一次开卡至多等单卡上限，到点当那一次没开成，由 `waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit` 管；
 //! 开不成隔 1 分钟重开、至多重开 3 次，所以一张卡的期限是 4 个单卡上限加 3 个间隔，`waiting_for_the_gpu::ReopeningsOfAGpuNotOpened::longest_wait_of_one_gpu`）。
 //! 到点挂载拒绝（[`BackgroundGpuNotOpened::TimedOut`] 带到点那一刻的期限），工作线程留在驱动里不杀（没有安全的杀法）；它回来之前闸住新开卡，不再多起一条枚举线程
-//! （[`BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung`]；同一个驱动加载器里再起一条会不会也挂没量过）。它回来之后闸放开、开出来的东西没人要：
+//! （[`BackgroundGpuNotOpened::AnEarlierOpeningIsStillHung`]；同一个驱动加载器里再起一条会不会也挂没量过）。同一个理由管到一次挂载之内：一张卡的一次开卡到点之后
+//! 还没回来，重开（挂载那一刻与挂着切换同一个规矩）不在这张卡上再起一条，算那一次又没开成（`background_gpu::opening_of_a_card_within_the_limit`），
+//! 所以工作线程拒了挂载放开闸时，这一次挂载挂在驱动里的开卡线程每张卡至多一条。它回来之后闸放开、开出来的东西没人要：
 //! 开成的把手照卸载丢（还记着等超过上限的提交的卡泄漏、记进泄漏账；开成时量代价的提交都已做完，没有这种卡），开卡途中量代价出错而提交还在跑的卡
 //! 在开那张卡的线程上已经泄漏、记进泄漏账（`background_gpu::started_gpu_with_its_cost_measured`）。它 panic 了闸同样放开。
 //! 实例对象在超时之后的状态读不到：造不出真挂住的驱动，这一格核不了。
````

### crates/singlefs-background-gpu/src/gpus_of_this_host.rs

````diff
diff --git a/crates/singlefs-background-gpu/src/gpus_of_this_host.rs b/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
index 504979bb1e2081b7a56dedbfdc7e0986afba7b9e..4cb83a9b26d3c2ced2e77cd4de4b68e444f241ea 100644
--- a/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
+++ b/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
@@ -734,6 +734,40 @@ impl GpusLeakedWhenTheBackgroundGpuIsDropped {
     }
 }
 
+/// 一次挂载里挂着切换关卡时泄漏了哪几张卡（有一次等超过上限、没在 GPU 上做完的提交的那张整张泄漏、不析构：D26（后台整理与放置回收） 已定项 13
+/// 第 4 条「卸载时不等还在跑的提交，有这种提交的那张卡整张泄漏、不析构」用于切换关卡），按关的先后累计：调用方开卡时拿到一份
+/// （`background_gpu_compute::OpenedBackgroundGpuCompute`），随时读（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。几份把手共用同一份。
+#[derive(Clone, Debug, Default)]
+pub struct GpusLeakedWhenClosedAtASwitch {
+    leaked: Arc<Mutex<Vec<GpuLeakedWhenDropped>>>,
+}
+
+impl GpusLeakedWhenClosedAtASwitch {
+    /// 还没有一张卡在切换关卡时泄漏过。
+    #[must_use]
+    pub fn with_none_leaked() -> Self {
+        Self {
+            leaked: Arc::new(Mutex::new(Vec::new())),
+        }
+    }
+
+    /// 到此刻为止切换关卡时泄漏的卡，按关的先后，各带关的那一刻它上面占着的字节；没有是空。
+    #[must_use]
+    pub fn gpus_leaked(&self) -> Vec<GpuLeakedWhenDropped> {
+        self.leaked
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner)
+            .clone()
+    }
+
+    pub(crate) fn note_the_gpus_leaked(&self, gpus_leaked: &[GpuLeakedWhenDropped]) {
+        self.leaked
+            .lock()
+            .unwrap_or_else(std::sync::PoisonError::into_inner)
+            .extend_from_slice(gpus_leaked);
+    }
+}
+
 /// 这个进程里每张卡泄漏着的字节（几次挂载的后台 GPU 丢掉时泄漏的设备不析构，那一片显存这个进程活着就一直占着）：调用方在进程里留一份、
 /// 每次开卡都交同一份（`background_gpu_compute::open_background_gpu_compute`），新一次挂载这张卡的可用余量先扣掉它（第 3 条）。
 /// 同一张卡上每丢一次泄漏一次就按次累加这一次挂载自己泄漏的字节（[`GpuLeakedWhenDropped::bytes_leaked_by_this_mount`]），不覆盖：
````

### crates/singlefs-core/src/background_compute.rs

````diff
diff --git a/crates/singlefs-core/src/background_compute.rs b/crates/singlefs-core/src/background_compute.rs
index 4de2cf9a523b6e100e8d03f12173d0052aac0231..e45d239b5bd0a7bf83f4ca9b969b1c568b8e728e 100644
--- a/crates/singlefs-core/src/background_compute.rs
+++ b/crates/singlefs-core/src/background_compute.rs
@@ -13,8 +13,10 @@
 //! 每块盘都不超过就照旧停着（改第 4 条「不换回 CPU」，同日弹窗定「任一块盘超过就补跑」；[`WhetherTheTaskRunsTheNextBatch`]）。
 //! 打包搬迁不补跑。
 //! 挂载那一刻选 `gpu` 而实现照挂载那一组一张卡都用不上（参与的卡的显存上限连最小的一片都装不下、一张都不剩）时不拒挂载
-//! 有 GPU 路径的后台任务当场记成出错停着（[`BackgroundGpuCompute::failure_under_the_settings_handed_in`]）；
-//! 挂着切到这样一组同样不拒切换，切换生效的那一批开头照挂载那一刻一样读、一样当场停着，报同一个成员。
+//! 有 GPU 路径的后台任务当场记成出错停着（实现答换上了时带着的那个出错，[`LatestGpuSettingsHandedIn::PutInEffect`]）；
+//! 挂着切到这样一组同样不拒切换，实现答换上了那一刻照挂载那一刻一样当场停着，报同一个成员。挂着切换点到没开的卡时实现在会话锁外开卡，
+//! 开卡的这段时间切换还没生效：照旧照切换之前那一组跑（切换之前选 `cpu` 的就在 CPU 上算），实现答换上了才生效（[`BackgroundComputeOfThisMount`]）。
+//! 切到 `cpu` 时告诉实现一张卡都不参与（[`BackgroundGpuCompute::use_no_gpu_from_the_next_batch`]），开着的卡全部关掉。
 
 use std::fmt;
 use std::sync::{Arc, Mutex, MutexGuard};
@@ -378,24 +380,32 @@ pub trait BackgroundGpuCompute: GpuInventory {
     /// 给它的上限，每一处等 GPU 不超过等的上限。核心层在挂载那一刻（选 `gpu` 时）与每次通过校验的切换生效的那一批开头、开算之前调，
     /// 一批里三段都算完之前不再调（一批之内不换）。开着的卡实现不为它重开设备，三段的连续不确定计数照旧留着（第 8 条：只在重挂时清零）。
     /// `settings` 点到没开着的卡时，实现可以不在这一刻换上：在会话锁外开卡、开完了在之后的两批之间换上
-    /// （[`Self::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]），开着的这段时间照旧照交来之前那一组跑。
+    /// （[`Self::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`] 交回 [`LatestGpuSettingsHandedIn::PutInEffect`] 那一问），
+    /// 开着的这段时间照旧照交来之前那一组跑，核心层把这一组记成还没生效。
     /// `settings` 的卡都在这份实现答的卡清单里（核心层照这份清单校验过）。
     fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings);
 
-    /// 两批之间（每一批开头与整理两个入口判停没停之前，`BackgroundComputeOfThisMount::put_the_switch_requested_from_the_next_batch_in_effect` 里）：
-    /// 最近交来的那一组点到没开着的卡、实现在会话锁外开着的，开完了就在这一刻换上，交回 `None`；重开到上限都没开成就交回那张卡与每一次开不成的出错
-    /// （[`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]，只交这一次），核心层把有 GPU 路径的后台任务当场记成出错停着、报警，
-    /// 不自动再试，直到下一次通过校验的切换。还在开、没在开的交回 `None`，不等。不开卡的实现（测试里的假实现）没有这一样，交回 `None`。
+    /// 从下一批起一张卡都不参与（核心层在挂载那一刻与切换生效的那一批开头，生效的那一组选 `cpu` 时调；第 2 条「切换之后不再参与的卡，切换生效时关掉」
+    /// 用于切到 `cpu`：一张卡都不参与）：开着的卡全部关掉，还在等开卡的那一组与没取走的开卡结局丢掉、开卡叫停。之后再交来一组 `gpu` 的设置，
+    /// 它点的卡都当没开的卡照重开的规矩开。不开卡的实现（测试里的假实现）没有这一样，什么都不做。
+    fn use_no_gpu_from_the_next_batch(&mut self) {}
+
+    /// 两批之间（每一批开头与整理两个入口判停没停之前，`BackgroundComputeOfThisMount::put_the_switch_requested_from_the_next_batch_in_effect` 里；
+    /// 交设置之后当场也问一次）：最近交来的那一组此刻怎样了（[`LatestGpuSettingsHandedIn`]）。在会话锁外开的卡开完了就在这一刻换上；
+    /// 还在开的不等。核心层只在交给实现的那一组还没生效时照答案办，别的时候问它只为让实现收下已开完的卡。
+    /// 不开卡的实现（测试里的假实现）交来那一刻就换上：答换上了，连同 [`Self::failure_under_the_settings_handed_in`]。
     fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
         &mut self,
-    ) -> Option<BackgroundGpuComputeFailed> {
-        None
+    ) -> LatestGpuSettingsHandedIn {
+        LatestGpuSettingsHandedIn::PutInEffect {
+            failure_under_the_settings: self.failure_under_the_settings_handed_in(),
+        }
     }
 
-    /// 照最近一次交来的那一组设置（[`Self::use_the_settings_from_the_next_batch`]）一张卡都用不上时，那一组下每一批三段都会交回的那个出错
-    /// （例如参与的卡的显存上限连最小的一片都装不下、一张都不剩）；用得上交回 `None`。核心层在挂载那一刻与每次切换生效的那一批开头把那一组交给实现之后
-    /// 各读一次：不是 `None` 就不拒挂载、不拒切换，有 GPU 路径的后台任务当场记成出错停着
-    /// （[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]），不等那一组下的头一批交回出错。只读，不交活。
+    /// 照最近一次换上的那一组设置一张卡都用不上时，那一组下每一批三段都会交回的那个出错
+    /// （例如参与的卡的显存上限连最小的一片都装不下、一张都不剩）；用得上交回 `None`。核心层不直接读它：
+    /// [`Self::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`] 的缺省答法把它带进「换上了」那一种，
+    /// 核心层照它判停（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]），不等那一组下的头一批交回出错。只读，不交活。
     /// 不开卡的实现（测试里的假实现）没有这一样，交回 `None`。
     fn failure_under_the_settings_handed_in(&self) -> Option<BackgroundGpuComputeFailed> {
         None
@@ -408,6 +418,22 @@ pub trait BackgroundGpuCompute: GpuInventory {
     }
 }
 
+/// 两批之间问实现的那一问（[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]）交回的：
+/// 最近交给它的那一组此刻怎样了。核心层照它定这一组什么时候生效（第 4 条「开成了，下一批起换成新设置」）。封闭集合。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum LatestGpuSettingsHandedIn {
+    /// 实现还在会话锁外为它开卡：照旧照交来之前那一组跑，这一组还没生效。
+    StillOpeningTheirCards,
+    /// 换上了：从下一批起照它跑。`failure_under_the_settings` 是照它一张卡都用不上时每一批三段都会交回的那个出错（用得上是 `None`），
+    /// 核心层照挂载那一刻的停法判（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]）。
+    PutInEffect {
+        failure_under_the_settings: Option<BackgroundGpuComputeFailed>,
+    },
+    /// 它点到的一张卡重开到上限都没开成（[`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]）：核心层把有 GPU 路径的后台任务当场记成出错停着、
+    /// 报警，不自动再试，直到下一次通过校验的切换。
+    NotOpened { failure: BackgroundGpuComputeFailed },
+}
+
 /// 一张参与的卡：它的标识、显存总量（卡清单答的），与给它的显存上限（`None` 是不设上限，第 3 条）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct ParticipatingGpuDevice {
@@ -814,51 +840,41 @@ impl AdmittedBackgroundComputeSettings {
         self.gpu_settings.as_ref()
     }
 
-    /// 选 `gpu` 时把这一组交给它带着的实现，从下一批起照它跑；选 `cpu` 时什么都不做（实现这时不算活，切到 `gpu` 那一批再交）。
-    ///
-    /// # Panics
-    /// 选 `gpu` 而没带实现：选 `gpu` 的那一组在校验那一刻就带上实现。
-    fn hand_the_gpu_settings_to_the_implementation(&self) {
-        let Some(gpu_settings) = &self.gpu_settings else {
-            return;
-        };
-        self.gpu_compute
-            .as_ref()
-            .expect("选 gpu 的那一组在校验那一刻就带上实现：没给实现照 GPU 不在拒了")
-            .lock()
-            .use_the_settings_from_the_next_batch(gpu_settings);
-    }
-
-    /// 选 `gpu` 时问它带着的实现：照交给它的这一组是不是一张卡都用不上（[`BackgroundGpuCompute::failure_under_the_settings_handed_in`]），
-    /// 在 [`Self::hand_the_gpu_settings_to_the_implementation`] 之后调；选 `cpu` 时 `None`（实现这时不算活）。
+    /// 把这一组交给它带着的实现：选 `gpu` 时交这一组（[`BackgroundGpuCompute::use_the_settings_from_the_next_batch`]），从下一批起照它跑
+    /// （点到没开的卡时实现开完才换上）；选 `cpu` 而带着实现时告诉它一张卡都不参与（[`BackgroundGpuCompute::use_no_gpu_from_the_next_batch`]，
+    /// 用户定案「切到 cpu 时开着的卡全部关掉」）；选 `cpu` 而没带实现时什么都不做。
     ///
     /// # Panics
     /// 选 `gpu` 而没带实现：选 `gpu` 的那一组在校验那一刻就带上实现。
-    fn failure_of_the_implementation_under_the_gpu_settings(
-        &self,
-    ) -> Option<BackgroundGpuComputeFailed> {
-        self.gpu_settings.as_ref()?;
-        self.gpu_compute
-            .as_ref()
-            .expect("选 gpu 的那一组在校验那一刻就带上实现：没给实现照 GPU 不在拒了")
-            .lock()
-            .failure_under_the_settings_handed_in()
+    fn hand_the_settings_to_the_implementation(&self) {
+        match (&self.gpu_settings, &self.gpu_compute) {
+            (Some(gpu_settings), gpu_compute) => gpu_compute
+                .as_ref()
+                .expect("选 gpu 的那一组在校验那一刻就带上实现：没给实现照 GPU 不在拒了")
+                .lock()
+                .use_the_settings_from_the_next_batch(gpu_settings),
+            (None, Some(gpu_compute)) => gpu_compute.lock().use_no_gpu_from_the_next_batch(),
+            (None, None) => {}
+        }
     }
 
-    /// 选 `gpu` 时让它带着的实现把交设置以来在会话锁外开完的卡换上（[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]），
-    /// 交回重开到上限都没开成的那一个出错；选 `cpu` 时 `None`（实现这时不算活）。
+    /// 选 `gpu` 时问它带着的实现最近交给它的那一组此刻怎样了
+    /// （[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]，开完的卡在这一刻换上）；
+    /// 选 `cpu` 时 `None`（实现这时不算活）。
     ///
     /// # Panics
     /// 选 `gpu` 而没带实现：选 `gpu` 的那一组在校验那一刻就带上实现。
-    fn failure_of_the_gpus_opened_since_the_gpu_settings_were_handed_in(
+    fn latest_gpu_settings_handed_in_asked_of_the_implementation(
         &self,
-    ) -> Option<BackgroundGpuComputeFailed> {
+    ) -> Option<LatestGpuSettingsHandedIn> {
         self.gpu_settings.as_ref()?;
-        self.gpu_compute
-            .as_ref()
-            .expect("选 gpu 的那一组在校验那一刻就带上实现：没给实现照 GPU 不在拒了")
-            .lock()
-            .put_the_gpus_opened_since_the_settings_were_handed_in_in_effect()
+        Some(
+            self.gpu_compute
+                .as_ref()
+                .expect("选 gpu 的那一组在校验那一刻就带上实现：没给实现照 GPU 不在拒了")
+                .lock()
+                .put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(),
+        )
     }
 
     /// 这一组带着的 GPU 实现：选 `gpu` 时恒有；选 `cpu` 时是挂载或切换时递进来的那一份（以后切到 `gpu` 没另给实现时用它），没给是 `None`。
@@ -1172,9 +1188,9 @@ pub enum BackgroundComputeError {
         segment: GpuComputedSegment,
         failure: BackgroundGpuComputeFailed,
     },
-    /// 挂载那一刻、或挂着切换生效的那一批开头选 `gpu`，实现照交给它的那一组一张卡都用不上（[`BackgroundGpuCompute::failure_under_the_settings_handed_in`]），
-    /// 或那一组点到的卡在会话锁外重开到上限都没开成（[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]，
-    /// 报的是 [`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]）：
+    /// 挂载那一刻、或挂着切换时选 `gpu`，实现答交给它的那一组换上了而照它一张卡都用不上（[`LatestGpuSettingsHandedIn::PutInEffect`] 带着的出错），
+    /// 或那一组点到的卡在会话锁外重开到上限都没开成（[`LatestGpuSettingsHandedIn::NotOpened`]，报的是
+    /// [`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]）：
     /// 挂载照常做成、切换照收，有 GPU 路径的后台任务当场停着、报这个错，不等那一组下的头一批交回出错。
     GpuUnusableUnderTheSettingsHandedIn { failure: BackgroundGpuComputeFailed },
 }
@@ -1247,15 +1263,14 @@ fn tasks_with_a_gpu_path_stopped_by(
         .collect()
 }
 
-/// 把 `settings_handed_in` 交给它带着的实现之后读一次（[`BackgroundGpuCompute::failure_under_the_settings_handed_in`]）：实现照这一组一张卡都用不上，
-/// 就交回有 GPU 路径的每一种后台任务各一条停着（[`tasks_with_a_gpu_path_stopped_by`]）；用得上、或选 `cpu`
-/// 交回空。挂载那一刻与切换生效时共用这一份，两处停着报的是同一个成员。
-fn tasks_stopped_when_no_gpu_is_usable_under_the_settings_handed_in(
-    settings_handed_in: &AdmittedBackgroundComputeSettings,
+/// 实现答一组设置换上了（[`LatestGpuSettingsHandedIn::PutInEffect`]）之后停着的后台任务：照它一张卡都用不上，就是有 GPU 路径的每一种后台任务各一条停着
+/// （[`tasks_with_a_gpu_path_stopped_by`]）；用得上是空。挂载那一刻与切换生效时共用这一份，两处停着报的是同一个成员。
+fn tasks_stopped_when_no_gpu_is_usable_under_the_settings_put_in_effect(
+    failure_under_the_settings: Option<&BackgroundGpuComputeFailed>,
 ) -> Vec<BackgroundTaskStoppedAfterAComputeError> {
-    match settings_handed_in.failure_of_the_implementation_under_the_gpu_settings() {
+    match failure_under_the_settings {
         None => Vec::new(),
-        Some(failure) => tasks_with_a_gpu_path_stopped_by(&failure),
+        Some(failure) => tasks_with_a_gpu_path_stopped_by(failure),
     }
 }
 
@@ -1305,11 +1320,16 @@ pub enum BackgroundTaskAfterTheComputeError {
     StopLiftedByTheSwitchRequestedBeforeTheError,
 }
 
-/// 后台计算在这一次挂载里的状态：此刻生效的设置，与挂着时请求、从下一批开头起用的那一组；因一批出错停着的后台任务。
-/// 生命周期一次挂载，只住内存。
+/// 后台计算在这一次挂载里的状态：此刻生效的设置；已交给实现、实现还在会话锁外开卡而没生效的那一组；挂着时请求、从下一批开头起交给实现的那一组；
+/// 因一批出错停着的后台任务。生命周期一次挂载，只住内存。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct BackgroundComputeOfThisMount {
+    /// 此刻生效的那一组：一批照它定走哪条路。选 `gpu` 的一组在实现答换上了（[`LatestGpuSettingsHandedIn::PutInEffect`]）或开不成
+    /// （[`LatestGpuSettingsHandedIn::NotOpened`]，停着）那一刻才成为生效的；选 `cpu` 的一组交给实现那一刻就生效。
     in_effect: AdmittedBackgroundComputeSettings,
+    /// 已交给实现、实现答还在为它开卡的那一组（选 `gpu`）：这期间照旧照 `in_effect` 跑，它算还没生效的切换请求
+    /// （第 4 条「出错之前已经收到、还没生效的切换请求，在停下那一刻也算数」）。
+    handed_in_while_the_implementation_opens_its_cards: Option<AdmittedBackgroundComputeSettings>,
     requested_from_the_next_batch: Option<AdmittedBackgroundComputeSettings>,
     stopped_after_a_compute_error: Vec<BackgroundTaskStoppedAfterAComputeError>,
     /// 停着的后台任务停下以来叫起了几次：头一次叫起时记下（一次叫起里一批出错停下的，停下那一刻就记下 0），停着的那几条换掉或清掉时一起清掉
@@ -1325,26 +1345,25 @@ pub struct BackgroundComputeOfThisMount {
 }
 
 impl BackgroundComputeOfThisMount {
-    /// 挂载那一刻：挂载给的那一组（校验过）生效，没有待生效的。选 `gpu` 时当场把这一组交给它带着的实现
-    /// （[`BackgroundGpuCompute::use_the_settings_from_the_next_batch`]），头一批起照它跑；实现照这一组一张卡都用不上
-    /// （[`BackgroundGpuCompute::failure_under_the_settings_handed_in`]）时，有 GPU 路径的后台任务当场记成出错停着
+    /// 挂载那一刻：挂载给的那一组（校验过）生效，没有待生效的。当场把这一组交给它带着的实现（[`AdmittedBackgroundComputeSettings`] 选 `gpu`
+    /// 时交这一组，选 `cpu` 而带着实现时告诉它一张卡都不参与）、选 `gpu` 时当场问一次（[`Self::put_the_switch_requested_from_the_next_batch_in_effect`]
+    /// 里的同一判）：实现答换上了而照这一组一张卡都用不上时，有 GPU 路径的后台任务当场记成出错停着
     /// （[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]，补跑的批数 0），挂载照常做成；别的情形没有停着的后台任务。
     #[must_use]
     pub fn of_a_mount(admitted_at_the_mount: AdmittedBackgroundComputeSettings) -> Self {
-        admitted_at_the_mount.hand_the_gpu_settings_to_the_implementation();
-        let stopped_after_a_compute_error =
-            tasks_stopped_when_no_gpu_is_usable_under_the_settings_handed_in(
-                &admitted_at_the_mount,
-            );
-        Self {
-            in_effect: admitted_at_the_mount,
+        let mut background_compute = Self {
+            in_effect: admitted_at_the_mount.clone(),
+            handed_in_while_the_implementation_opens_its_cards: None,
             requested_from_the_next_batch: None,
-            stopped_after_a_compute_error,
+            stopped_after_a_compute_error: Vec::new(),
             wakes_of_the_tasks_stopped: Vec::new(),
             wake_under_way: WakeOfTheBackgroundTasks::NoneUnderWay,
             compute_error_of_the_batch_under_way: None,
             gc_cpu_compensation_judgement: GcCpuCompensationJudgement::ByTheReading,
-        }
+        };
+        background_compute.hand_the_settings_in(admitted_at_the_mount);
+        background_compute.take_the_answer_on_the_settings_handed_in();
+        background_compute
     }
 
     /// 只供测试（`.claude/rules/fs-design.md`「五条硬要求」第 2 条）：从这一刻起，停着的后台任务每一批开头的补跑那一判都当占用不超过阈值
@@ -1383,15 +1402,19 @@ impl BackgroundComputeOfThisMount {
         self.compute_error_of_the_batch_under_way.take()
     }
 
-    /// 一批出错之后（第 4 条）：出错之前就挂着待生效的切换，就当场解除——那一次切换算停着时收到的请求，`task` 不记成停着，
-    /// 下一批开头照常换上它；没有就把 `task` 记成停着，等下一次通过校验的切换请求。停在一次叫起里的（[`Self::begin_a_wake`] 开过、还没结束），
+    /// 一批出错之后（第 4 条）：出错之前就挂着还没生效的切换（请求了还没交给实现，或交给了实现而实现还在为它开卡），就当场解除——
+    /// 那一次切换算停着时收到的请求，`task` 不记成停着，之后照常等它生效；没有就把 `task` 记成停着，等下一次通过校验的切换请求。停在一次叫起里的（[`Self::begin_a_wake`] 开过、还没结束），
     /// 那一次叫起结束时报警不数这一趟（[`Self::alarm_at_the_end_of_a_wake`] 报 0）；停在可写挂载续做里的不是叫起，停下之后头一次叫起照数 1。不读写盘。
     pub(crate) fn stop_the_task_after_a_compute_error(
         &mut self,
         task: BackgroundTask,
         error: BackgroundComputeError,
     ) -> BackgroundTaskAfterTheComputeError {
-        if self.requested_from_the_next_batch.is_some() {
+        if self.requested_from_the_next_batch.is_some()
+            || self
+                .handed_in_while_the_implementation_opens_its_cards
+                .is_some()
+        {
             return BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError;
         }
         if !self.is_stopped_after_a_compute_error(task) {
@@ -1482,7 +1505,8 @@ impl BackgroundComputeOfThisMount {
     }
 
     /// 补跑那一判此刻的读数：`fullest_device` 是调用方从分配器取的池子此刻最满的那一块盘
-    /// （[`AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator`]），阈值取下一批读的那一组（有待生效的取它，没有取生效的）。
+    /// （[`AllocatedBytesAndCapacityOfADevice::of_the_fullest_device_of_the_allocator`]），阈值取下一批读的那一组
+    /// （[`Self::gc_cpu_compensation_threshold_of_the_next_batch`]）。
     #[must_use]
     pub fn gc_cpu_compensation_reading(
         &self,
@@ -1494,12 +1518,16 @@ impl BackgroundComputeOfThisMount {
         }
     }
 
-    /// 下一批读的那一组的补跑阈值：有待生效的取它，没有取生效的。
+    /// 下一批读的那一组的补跑阈值：有请求了还没交给实现的取它，再看交给了实现而实现还在开卡的那一组，都没有取生效的
+    /// （第 4 条：阈值随切换请求一起换、从下一批起生效，不等开卡）。
     fn gc_cpu_compensation_threshold_of_the_next_batch(
         &self,
     ) -> BackgroundGcCpuCompensationThreshold {
         self.requested_from_the_next_batch
             .as_ref()
+            .or(self
+                .handed_in_while_the_implementation_opens_its_cards
+                .as_ref())
             .unwrap_or(&self.in_effect)
             .gc_cpu_compensation_threshold
     }
@@ -1540,6 +1568,16 @@ impl BackgroundComputeOfThisMount {
         &self.in_effect
     }
 
+    /// 已交给实现、实现还在会话锁外为它开卡、还没生效的那一组（选 `gpu`）；没有是 `None`（`.claude/rules/fs-design.md` 五条硬要求第 4 条：
+    /// 运行时看得出这一刻跑的是切换之前那一组）。
+    #[must_use]
+    pub fn handed_in_while_the_implementation_opens_its_cards(
+        &self,
+    ) -> Option<&AdmittedBackgroundComputeSettings> {
+        self.handed_in_while_the_implementation_opens_its_cards
+            .as_ref()
+    }
+
     /// 挂着时请求过、还没到下一批开头的那一组。
     #[must_use]
     pub fn requested_from_the_next_batch(&self) -> Option<&AdmittedBackgroundComputeSettings> {
@@ -1596,42 +1634,83 @@ impl BackgroundComputeOfThisMount {
         Ok(())
     }
 
-    /// 两批之间换上挂着请求、待生效的那一组（没有就不换）：换上的是 `gpu` 的就把它交给带着的实现（从下一批起照它跑），实现照它一张卡都用不上
-    /// 就照挂载那一刻一样，有 GPU 路径的后台任务当场记成出错停着（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；切换照收）；
-    /// 用得上就没有停着的（请求切换那一刻已解除全部停下）。之后（换没换都做）生效的是 `gpu` 的，就让实现把交设置以来在会话锁外开完的卡换上
-    /// （[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]）：实现交回重开到上限都没开成的那个出错，
-    /// 就照「一张卡都用不上」一样把有 GPU 路径的后台任务当场记成出错停着、报那个错（用户定案「3 次都不成：GC 停下，报警……不自动再试」），
-    /// 直到下一次通过校验的切换。
+    /// 两批之间：挂着请求、待生效的那一组有的话交给实现（[`Self::hand_the_settings_in`]），再问实现最近交给它的那一组此刻怎样了
+    /// （[`Self::take_the_answer_on_the_settings_handed_in`]）：选 `gpu` 的一组要等实现答换上了才生效，实现还在会话锁外开卡的这段时间照旧照切换之前那一组跑，
+    /// 这一组算还没生效的切换请求（这期间一批出错照它解除停下，[`Self::stop_the_task_after_a_compute_error`]）；答换上了就照挂载那一刻的停法判
+    /// （照它一张卡都用不上就把有 GPU 路径的后台任务当场记成出错停着，[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；用得上就不停）；
+    /// 答开不成就照「一张卡都用不上」一样停着、报那个错（用户定案「3 次都不成：GC 停下，报警……不自动再试」），直到下一次通过校验的切换。
+    /// 选 `cpu` 的一组交给实现那一刻就生效。
     /// 一批开头（[`Self::begin_a_batch`]）调；整理的两个入口在判「整理停没停」之前也调（显式入口一趟开头、后台入口判起不起跑之前），
     /// 切到一张卡都用不上的那一组之后，下一次进来在任何一个写之前就判出停着：不走删除续做、不取走「该跑了」，与挂载那一刻同。
     /// 只在两批之间调：一批之内的切换只记成待生效。不读写盘。
     pub(crate) fn put_the_switch_requested_from_the_next_batch_in_effect(&mut self) {
         if let Some(requested) = self.requested_from_the_next_batch.take() {
-            self.in_effect = requested;
-            self.in_effect.hand_the_gpu_settings_to_the_implementation();
-            let tasks_stopped =
-                tasks_stopped_when_no_gpu_is_usable_under_the_settings_handed_in(&self.in_effect);
-            self.replace_the_tasks_stopped(tasks_stopped);
+            self.hand_the_settings_in(requested);
         }
-        if let Some(failure) = self
-            .in_effect
-            .failure_of_the_gpus_opened_since_the_gpu_settings_were_handed_in()
-        {
-            self.replace_the_tasks_stopped(tasks_with_a_gpu_path_stopped_by(&failure));
+        self.take_the_answer_on_the_settings_handed_in();
+    }
+
+    /// 把 `settings` 交给它带着的实现（[`AdmittedBackgroundComputeSettings::hand_the_settings_to_the_implementation`]）：选 `cpu` 的当场生效
+    /// （之前交给实现、还在开卡的那一组作废）；选 `gpu` 的记成交给了实现、还没生效，等实现答（[`Self::take_the_answer_on_the_settings_handed_in`]）。
+    fn hand_the_settings_in(&mut self, settings: AdmittedBackgroundComputeSettings) {
+        settings.hand_the_settings_to_the_implementation();
+        match settings.gpu_settings() {
+            None => {
+                self.in_effect = settings;
+                self.handed_in_while_the_implementation_opens_its_cards = None;
+            }
+            Some(_) => {
+                self.handed_in_while_the_implementation_opens_its_cards = Some(settings);
+            }
         }
     }
 
-    /// 一批开头（每批读一次设置的那一处）：先把待生效的换上（[`Self::put_the_switch_requested_from_the_next_batch_in_effect`]），
-    /// 换上的是 `gpu` 的就在开算之前把它交给带着的实现（从这一批起照它跑，一批之内不再交），
-    /// 实现照它一张卡都用不上就照挂载那一刻一样，有 GPU 路径的后台任务当场记成出错停着（[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；
-    /// 切换照收）；再判 `task` 这一批跑不跑（[`Self::whether_the_task_runs_the_next_batch`]，`fullest_device`
+    /// 问实现最近交给它的那一组此刻怎样了（[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]）：
+    /// 有交给了实现、还没生效的那一组就问它带着的实现，照答案办——还在开卡：不动；换上了：它生效，停着的那几条换成照它判出的
+    /// （[`tasks_stopped_when_no_gpu_is_usable_under_the_settings_put_in_effect`]）；开不成：它生效，有 GPU 路径的后台任务记成出错停着、报那个错。
+    /// 没有还没生效的那一组而生效的选 `gpu` 时也问生效那一组带着的实现（让它收下为更早几次交设置开完的卡），答案不用。
+    fn take_the_answer_on_the_settings_handed_in(&mut self) {
+        let Some(handed_in) = self
+            .handed_in_while_the_implementation_opens_its_cards
+            .take()
+        else {
+            let _answer_with_no_settings_waiting = self
+                .in_effect
+                .latest_gpu_settings_handed_in_asked_of_the_implementation();
+            return;
+        };
+        match handed_in.latest_gpu_settings_handed_in_asked_of_the_implementation() {
+            None | Some(LatestGpuSettingsHandedIn::StillOpeningTheirCards) => {
+                self.handed_in_while_the_implementation_opens_its_cards = Some(handed_in);
+            }
+            Some(LatestGpuSettingsHandedIn::PutInEffect {
+                failure_under_the_settings,
+            }) => {
+                self.in_effect = handed_in;
+                self.replace_the_tasks_stopped(
+                    tasks_stopped_when_no_gpu_is_usable_under_the_settings_put_in_effect(
+                        failure_under_the_settings.as_ref(),
+                    ),
+                );
+            }
+            Some(LatestGpuSettingsHandedIn::NotOpened { failure }) => {
+                self.in_effect = handed_in;
+                self.replace_the_tasks_stopped(tasks_with_a_gpu_path_stopped_by(&failure));
+            }
+        }
+    }
+
+    /// 一批开头（每批读一次设置的那一处）：先把待生效的交给实现、问实现最近交给它的那一组此刻怎样了
+    /// （[`Self::put_the_switch_requested_from_the_next_batch_in_effect`]；一批之内不再交、不再问），
+    /// 实现答换上了而照它一张卡都用不上、或答开不成，就照挂载那一刻一样，有 GPU 路径的后台任务当场记成出错停着
+    /// （[`BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn`]；切换照收）；再判 `task` 这一批跑不跑（[`Self::whether_the_task_runs_the_next_batch`]，`fullest_device`
     /// 是调用方这一刻从分配器取的池子最满的那一块盘）——停着而不补跑就不开这一批；最后定这一批走哪条路：补跑的走 CPU、停下以来补跑的批数加一，
-    /// 别的按生效的那一组定。有待生效的切换时没有停着的后台任务（请求切换那一刻解除全部停下，停下那一刻挂着切换的不记成停着），
-    /// 所以先换上再判与先判再换上，只差换上的那一组用不上 GPU 这一种。
+    /// 别的按生效的那一组定（实现还在为交给它的那一组开卡时，生效的仍是切换之前那一组）。有还没生效的切换时没有停着的后台任务
+    /// （请求切换那一刻解除全部停下，停下那一刻挂着切换的不记成停着），所以先问再判与先判再问，只差实现答的那一组用不上 GPU 这一种。
     ///
     /// # Errors
-    /// `task` 停着、这一批不补跑 ⇒ [`BatchNotBegunWhileTheTaskStaysStopped`]：没有待生效的切换时状态一样都没动；有的话那一组已换上、交给了实现，
-    /// 停着的是这一批开头照它记下的那几条。
+    /// `task` 停着、这一批不补跑 ⇒ [`BatchNotBegunWhileTheTaskStaysStopped`]：没有还没生效的切换时状态一样都没动；有的话那一组已交给了实现、
+    /// 实现照它答过，停着的是这一批开头照答案记下的那几条。
     ///
     /// # Panics
     /// 上一批注入的错没被读走：注入只在一批的钩子里，那一批读完设置之后就读走。
@@ -1893,4 +1972,469 @@ mod tests {
             }
         );
     }
+
+    // ── 切换点到没开的卡：交给实现之后、实现开完卡之前，切换还没生效（第 4 条「开成了，下一批起换成新设置」「出错之前已经收到、还没生效的切换请求，
+    // 在停下那一刻也算数」「报的是挂载那一刻同一个成员」；用户定案「切到 cpu 时开着的卡全部关掉」）──
+
+    fn card_for_the_switch_tests(address: &[u8]) -> GpuDeviceIdentifier {
+        GpuDeviceIdentifier::of_opaque_bytes(address.to_vec())
+    }
+
+    /// 替身照 [`BackgroundGpuCompute`] 文档写的约定：交来的一组点的卡都开着就当场换上；点到没开的卡就在会话锁外开（用例把 `opening_ends` 摆成 `true` 就算开完了），
+    /// 开着的这段时间两批之间那一问答还在开卡，开完了换上、答换上了连同这一组用不用得上（`failure_once_put_in_effect`）。告诉它一张卡都不参与时开着的卡全部关掉。
+    struct CardsOpenedByTheTestDoubleState {
+        cards_opened: Vec<GpuDeviceIdentifier>,
+        cards_waiting_for_the_opening: Option<Vec<GpuDeviceIdentifier>>,
+        opening_ends: bool,
+        failure_under_the_settings_in_use: Option<BackgroundGpuComputeFailed>,
+        failure_once_put_in_effect: Option<BackgroundGpuComputeFailed>,
+        times_told_no_gpu_takes_part: u32,
+    }
+
+    struct GpuComputeOpeningCardsWhenTheTestSays {
+        state: Arc<Mutex<CardsOpenedByTheTestDoubleState>>,
+    }
+
+    impl GpuInventory for GpuComputeOpeningCardsWhenTheTestSays {
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
+    impl BackgroundGpuCompute for GpuComputeOpeningCardsWhenTheTestSays {
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
+        fn use_the_settings_from_the_next_batch(&mut self, settings: &AdmittedGpuSettings) {
+            let mut state = self.state.lock().expect("用例没在别处 panic 过");
+            let cards_named: Vec<GpuDeviceIdentifier> = settings
+                .participating_devices()
+                .iter()
+                .map(|participating| participating.device.clone())
+                .collect();
+            if cards_named
+                .iter()
+                .all(|card| state.cards_opened.contains(card))
+            {
+                state.cards_waiting_for_the_opening = None;
+                state.failure_under_the_settings_in_use = state.failure_once_put_in_effect.clone();
+            } else {
+                state.cards_waiting_for_the_opening = Some(cards_named);
+            }
+        }
+
+        fn use_no_gpu_from_the_next_batch(&mut self) {
+            let mut state = self.state.lock().expect("用例没在别处 panic 过");
+            state.cards_opened.clear();
+            state.cards_waiting_for_the_opening = None;
+            state.times_told_no_gpu_takes_part += 1;
+        }
+
+        fn put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(
+            &mut self,
+        ) -> LatestGpuSettingsHandedIn {
+            let mut state = self.state.lock().expect("用例没在别处 panic 过");
+            if state.cards_waiting_for_the_opening.is_some() {
+                if !state.opening_ends {
+                    return LatestGpuSettingsHandedIn::StillOpeningTheirCards;
+                }
+                state.cards_opened = state
+                    .cards_waiting_for_the_opening
+                    .take()
+                    .expect("上面判过在等");
+                state.failure_under_the_settings_in_use = state.failure_once_put_in_effect.clone();
+            }
+            LatestGpuSettingsHandedIn::PutInEffect {
+                failure_under_the_settings: state.failure_under_the_settings_in_use.clone(),
+            }
+        }
+    }
+
+    fn gpu_settings_naming_the_card(
+        address: &[u8],
+        memory_limit: Option<u64>,
+    ) -> BackgroundComputeSettings {
+        BackgroundComputeSettings {
+            background_compute: BackgroundComputeBackend::Gpu,
+            background_gpu_devices: Some(vec![card_for_the_switch_tests(address)]),
+            background_gpu_memory_limit: memory_limit.map(|limit| vec![GpuMemoryInBytes(limit)]),
+            background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+        }
+    }
+
+    /// 池子最满的那一块盘远没到补跑阈值：停着就照旧停着。
+    fn fullest_device_far_below_the_threshold() -> AllocatedBytesAndCapacityOfADevice {
+        AllocatedBytesAndCapacityOfADevice {
+            device: DeviceIdentity(0),
+            allocated_bytes: 0,
+            capacity_bytes: 1 << 30,
+        }
+    }
+
+    fn every_gpu_left_out_on_the_card(address: &[u8]) -> BackgroundGpuComputeFailed {
+        BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit {
+            gpus_left_out: vec![GpuLeftOutUnderItsMemoryLimit {
+                device: card_for_the_switch_tests(address),
+                memory_limit: GpuMemoryInBytes(64),
+                smallest_piece_bytes: 972,
+                bytes_already_held_on_the_device: 0,
+            }],
+        }
+    }
+
+    /// 替身开着「0000:01:00.0」，两个出错照 `failure_under_the_settings_in_use`、`failure_once_put_in_effect` 摆。
+    fn state_with_the_card_at_one_opened(
+        failure_under_the_settings_in_use: Option<BackgroundGpuComputeFailed>,
+        failure_once_put_in_effect: Option<BackgroundGpuComputeFailed>,
+    ) -> Arc<Mutex<CardsOpenedByTheTestDoubleState>> {
+        Arc::new(Mutex::new(CardsOpenedByTheTestDoubleState {
+            cards_opened: vec![card_for_the_switch_tests(b"0000:01:00.0")],
+            cards_waiting_for_the_opening: None,
+            opening_ends: false,
+            failure_under_the_settings_in_use,
+            failure_once_put_in_effect,
+            times_told_no_gpu_takes_part: 0,
+        }))
+    }
+
+    /// 选 `gpu`、点「0000:01:00.0」的一次挂载，替身状态照 `state`。
+    fn mount_on_the_card_at_one(
+        state: &Arc<Mutex<CardsOpenedByTheTestDoubleState>>,
+    ) -> BackgroundComputeOfThisMount {
+        let handle = BackgroundGpuComputeHandle::of_an_implementation(
+            GpuComputeOpeningCardsWhenTheTestSays {
+                state: Arc::clone(state),
+            },
+        );
+        BackgroundComputeOfThisMount::of_a_mount(
+            admit_background_compute_settings_with_the_gpu_compute(
+                gpu_settings_naming_the_card(b"0000:01:00.0", None),
+                Some(handle),
+            )
+            .expect("卡清单列着「0000:01:00.0」"),
+        )
+    }
+
+    fn failure_on_the_card_at_one() -> BackgroundComputeError {
+        BackgroundComputeError::GpuSegmentFailed {
+            segment: GpuComputedSegment::UnitChecksumVerification,
+            failure: BackgroundGpuComputeFailed::GpuFailed {
+                device: card_for_the_switch_tests(b"0000:01:00.0"),
+                cause: GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost {
+                    reason: "替身：切换之前那张卡上设备丢了".to_string(),
+                },
+            },
+        }
+    }
+
+    fn route_of_the_batch(
+        batch: &Result<BackgroundBatchUnderWay, BatchNotBegunWhileTheTaskStaysStopped>,
+    ) -> Option<BackgroundComputeRoute> {
+        batch.as_ref().ok().map(|under_way| under_way.route)
+    }
+
+    /// 第 4 条「出错之前已经收到、还没生效的切换请求，在停下那一刻也算数，照它解除停下」：挂载点「0000:01:00.0」，请求切到没开的「0000:41:00.0」；
+    /// 下一批开头交给实现（实现开卡、照切换之前那一组跑），这一批在「0000:01:00.0」上出错——切换还没生效，照它解除停下；卡开成、换上之后那一批照常在 GPU 上开，
+    /// 生效的是点「0000:41:00.0」的那一组。
+    #[test]
+    fn a_batch_failing_on_the_card_before_the_switch_while_the_new_card_opens_is_lifted_by_that_switch(
+    ) {
+        let state = state_with_the_card_at_one_opened(None, None);
+        let mut background_compute = mount_on_the_card_at_one(&state);
+        background_compute
+            .request_switch_from_the_next_batch(
+                gpu_settings_naming_the_card(b"0000:41:00.0", None),
+                None,
+            )
+            .expect("卡清单列着「0000:41:00.0」");
+        let batch_while_opening = background_compute.begin_a_batch(
+            BackgroundTask::CompactionRelocation,
+            fullest_device_far_below_the_threshold(),
+        );
+        assert_eq!(
+            route_of_the_batch(&batch_while_opening),
+            Some(BackgroundComputeRoute::Gpu),
+            "开卡的这段时间照切换之前那一组在 GPU 上算"
+        );
+        assert!(
+            background_compute
+                .handed_in_while_the_implementation_opens_its_cards()
+                .is_some(),
+            "交给了实现、还在开卡：切换还没生效"
+        );
+        assert_eq!(
+            background_compute.stop_the_task_after_a_compute_error(
+                BackgroundTask::CompactionRelocation,
+                failure_on_the_card_at_one(),
+            ),
+            BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError,
+            "出错之前已经收到、还没生效的切换照它解除停下"
+        );
+        assert!(!background_compute
+            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
+        state.lock().expect("用例没在别处 panic 过").opening_ends = true;
+        let batch_once_opened = background_compute.begin_a_batch(
+            BackgroundTask::CompactionRelocation,
+            fullest_device_far_below_the_threshold(),
+        );
+        assert_eq!(
+            route_of_the_batch(&batch_once_opened),
+            Some(BackgroundComputeRoute::Gpu),
+            "换上之后那一批照常开"
+        );
+        assert_eq!(
+            background_compute
+                .in_effect()
+                .settings()
+                .background_gpu_devices,
+            Some(vec![card_for_the_switch_tests(b"0000:41:00.0")]),
+            "实现答换上了：生效的是切换点的那一组"
+        );
+    }
+
+    /// 同一段历史的对照：同一个错落在核心层把切换交给实现之前（请求还挂着）——照那一次请求当场解除。
+    #[test]
+    fn the_same_error_before_the_handing_in_is_lifted_by_the_switch_request() {
+        let state = state_with_the_card_at_one_opened(None, None);
+        let mut background_compute = mount_on_the_card_at_one(&state);
+        background_compute
+            .request_switch_from_the_next_batch(
+                gpu_settings_naming_the_card(b"0000:41:00.0", None),
+                None,
+            )
+            .expect("卡清单列着「0000:41:00.0」");
+        assert_eq!(
+            background_compute.stop_the_task_after_a_compute_error(
+                BackgroundTask::CompactionRelocation,
+                failure_on_the_card_at_one(),
+            ),
+            BackgroundTaskAfterTheComputeError::StopLiftedByTheSwitchRequestedBeforeTheError
+        );
+    }
+
+    /// 第 4 条「停着的时候收到任何一次通过校验的切换请求……当场解除停下，从下一批起用新设置」：挂载那一刻照挂载那一组一张卡都用不上、停着；
+    /// 请求切到没开的「0000:41:00.0」（装得下）——请求那一刻解除；交给实现那一批开头实现还在开卡，不读切换之前那一组的「用不上」，不再停；
+    /// 卡开成、换上之后照它判，用得上，照常开批。
+    #[test]
+    fn a_mount_stopped_under_its_settings_is_not_stopped_again_while_the_switched_card_opens_nor_once_it_is_put_in_effect(
+    ) {
+        let state = state_with_the_card_at_one_opened(
+            None,
+            Some(every_gpu_left_out_on_the_card(b"0000:01:00.0")),
+        );
+        let mut background_compute = mount_on_the_card_at_one(&state);
+        assert!(
+            background_compute
+                .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation),
+            "挂载那一组一张卡都用不上：挂载就停着"
+        );
+        state
+            .lock()
+            .expect("用例没在别处 panic 过")
+            .failure_once_put_in_effect = None;
+        background_compute
+            .request_switch_from_the_next_batch(
+                gpu_settings_naming_the_card(b"0000:41:00.0", None),
+                None,
+            )
+            .expect("卡清单列着「0000:41:00.0」");
+        assert!(!background_compute
+            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
+        let batch_while_opening = background_compute.begin_a_batch(
+            BackgroundTask::CompactionRelocation,
+            fullest_device_far_below_the_threshold(),
+        );
+        assert!(
+            batch_while_opening.is_ok(),
+            "交给实现那一刻实现还在开卡：不读切换之前那一组的用不上、不再停"
+        );
+        state.lock().expect("用例没在别处 panic 过").opening_ends = true;
+        let batch_once_opened = background_compute.begin_a_batch(
+            BackgroundTask::CompactionRelocation,
+            fullest_device_far_below_the_threshold(),
+        );
+        assert!(batch_once_opened.is_ok(), "换上之后用得上：照常开批");
+        assert!(!background_compute
+            .is_stopped_after_a_compute_error(BackgroundTask::CompactionRelocation));
+    }
+
+    /// 第 4 条「挂着切换到参与的卡一张都不剩的设置……报的是挂载那一刻同一个成员（GpuUnusableUnderTheSettingsHandedIn）」用于点没开的卡：
+    /// 请求切到没开的「0000:41:00.0」、给它装不下最小的一片的上限——开卡的这段时间照旧开批；卡开成、实现答换上了而一张卡都用不上，
+    /// 那一批开头按这个成员停着、不开批。
+    #[test]
+    fn the_switch_put_in_effect_after_the_opening_with_every_card_left_out_stops_as_unusable_and_begins_no_batch(
+    ) {
+        let state = state_with_the_card_at_one_opened(None, None);
+        let mut background_compute = mount_on_the_card_at_one(&state);
+        state
+            .lock()
+            .expect("用例没在别处 panic 过")
+            .failure_once_put_in_effect = Some(every_gpu_left_out_on_the_card(b"0000:41:00.0"));
+        background_compute
+            .request_switch_from_the_next_batch(
+                gpu_settings_naming_the_card(b"0000:41:00.0", Some(64)),
+                None,
+            )
+            .expect("上限不超过显存总量：照收");
+        let batch_while_opening = background_compute.begin_a_batch(
+            BackgroundTask::CompactionRelocation,
+            fullest_device_far_below_the_threshold(),
+        );
+        assert!(
+            batch_while_opening.is_ok(),
+            "开卡的这段时间照切换之前那一组开批"
+        );
+        state.lock().expect("用例没在别处 panic 过").opening_ends = true;
+        let batch_once_opened = background_compute.begin_a_batch(
+            BackgroundTask::CompactionRelocation,
+            fullest_device_far_below_the_threshold(),
+        );
+        assert!(
+            batch_once_opened.is_err(),
+            "换上的那一组一张卡都用不上：不开批"
+        );
+        assert_eq!(
+            background_compute
+                .stopped_after_a_compute_error()
+                .iter()
+                .map(|stopped| stopped.error.clone())
+                .collect::<Vec<_>>(),
+            vec![
+                BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
+                    failure: every_gpu_left_out_on_the_card(b"0000:41:00.0"),
+                }
+            ],
+            "报的是挂载那一刻同一个成员"
+        );
+    }
+
+    /// 对照：同样一张卡都用不上的设置，点的是挂载那一刻开着的「0000:01:00.0」——交给实现那一刻就换上、当场按同一个成员停着、不开批。
+    #[test]
+    fn the_switch_to_every_card_left_out_on_a_card_already_open_stops_at_the_handing_in() {
+        let state = state_with_the_card_at_one_opened(None, None);
+        let mut background_compute = mount_on_the_card_at_one(&state);
+        state
+            .lock()
+            .expect("用例没在别处 panic 过")
+            .failure_once_put_in_effect = Some(every_gpu_left_out_on_the_card(b"0000:01:00.0"));
+        background_compute
+            .request_switch_from_the_next_batch(
+                gpu_settings_naming_the_card(b"0000:01:00.0", Some(64)),
+                None,
+            )
+            .expect("上限不超过显存总量：照收");
+        let batch_at_the_handing_in = background_compute.begin_a_batch(
+            BackgroundTask::CompactionRelocation,
+            fullest_device_far_below_the_threshold(),
+        );
+        assert!(batch_at_the_handing_in.is_err());
+        assert!(background_compute
+            .stopped_after_a_compute_error()
+            .iter()
+            .any(|stopped| matches!(
+                stopped.error,
+                BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn { .. }
+            )));
+    }
+
+    /// 用户定案「切到 cpu 时开着的卡全部关掉」：挂载点「0000:01:00.0」，切到 `cpu`——那一批开头告诉实现一张卡都不参与（替身关掉开着的卡），那一批在 CPU 上算。
+    #[test]
+    fn the_switch_to_the_cpu_tells_the_implementation_that_no_gpu_takes_part() {
+        let state = state_with_the_card_at_one_opened(None, None);
+        let mut background_compute = mount_on_the_card_at_one(&state);
+        background_compute
+            .request_switch_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
+            .expect("选 cpu 照收");
+        let batch_on_the_cpu = background_compute.begin_a_batch(
+            BackgroundTask::CompactionRelocation,
+            fullest_device_far_below_the_threshold(),
+        );
+        assert_eq!(
+            route_of_the_batch(&batch_on_the_cpu),
+            Some(BackgroundComputeRoute::Cpu(
+                ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings
+            ))
+        );
+        let state_after_the_switch = state.lock().expect("用例没在别处 panic 过");
+        assert_eq!(state_after_the_switch.times_told_no_gpu_takes_part, 1);
+        assert!(
+            state_after_the_switch.cards_opened.is_empty(),
+            "开着的卡全部关掉"
+        );
+    }
+
+    /// 用户定案「切到 cpu 时开着的卡全部关掉」之后切回 `gpu`：点的卡都当没开的卡开——开卡的这段时间生效的仍是 `cpu`，那一批在 CPU 上算；
+    /// 开成、实现答换上了之后，下一批起在 GPU 上算。
+    #[test]
+    fn the_switch_back_to_the_gpu_runs_on_the_cpu_while_the_cards_open_and_on_the_gpu_once_put_in_effect(
+    ) {
+        let state = state_with_the_card_at_one_opened(None, None);
+        let mut background_compute = mount_on_the_card_at_one(&state);
+        background_compute
+            .request_switch_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
+            .expect("选 cpu 照收");
+        let _batch_on_the_cpu = background_compute.begin_a_batch(
+            BackgroundTask::CompactionRelocation,
+            fullest_device_far_below_the_threshold(),
+        );
+        background_compute
+            .request_switch_from_the_next_batch(
+                gpu_settings_naming_the_card(b"0000:01:00.0", None),
+                None,
+            )
+            .expect("挂载时递进来的实现还在：切回 gpu 照收");
+        let batch_while_opening = background_compute.begin_a_batch(
+            BackgroundTask::CompactionRelocation,
+            fullest_device_far_below_the_threshold(),
+        );
+        assert_eq!(
+            route_of_the_batch(&batch_while_opening),
+            Some(BackgroundComputeRoute::Cpu(
+                ReasonTheBatchRanOnTheCpu::CpuRequestedByTheSettings
+            )),
+            "开卡的这段时间生效的仍是 cpu：在 CPU 上算"
+        );
+        state.lock().expect("用例没在别处 panic 过").opening_ends = true;
+        let batch_once_opened = background_compute.begin_a_batch(
+            BackgroundTask::CompactionRelocation,
+            fullest_device_far_below_the_threshold(),
+        );
+        assert_eq!(
+            route_of_the_batch(&batch_once_opened),
+            Some(BackgroundComputeRoute::Gpu),
+            "开成之后下一批起在 GPU 上算"
+        );
+    }
 }
````

### crates/singlefs-mount-runtime/src/lib.rs

````diff
diff --git a/crates/singlefs-mount-runtime/src/lib.rs b/crates/singlefs-mount-runtime/src/lib.rs
index e382a69e0296d77e355866a0ab424c16a3073544..b3bf0226c1ff0e213b6d1e076385cc15178b81a8 100644
--- a/crates/singlefs-mount-runtime/src/lib.rs
+++ b/crates/singlefs-mount-runtime/src/lib.rs
@@ -969,7 +969,9 @@ impl<Device: BlockDevice> MountedRuntime<Device> {
 
     /// 挂着切换后台计算的设置，从下一批起生效（不排空待发）。开卡器不在这一条路上：新递一份实现的，`gpu_compute` 由调用方先开好递进来；
     /// 沿用这次挂载那一份而点到它没开着的卡的，切换照收，那一份实现在会话锁外开卡（产品 GPU 包，开不成隔一段时间重开），这一次调用与之后的批都不等它，
-    /// 开完了在两批之间换上；重开到上限都没开成就在那时停 GC、报警（`singlefs_core::background_compute::BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`）。
+    /// 开卡的这段时间切换还没生效、照切换之前那一组跑，开完了在两批之间换上；重开到上限都没开成就在那时停 GC、报警
+    /// （`singlefs_core::background_compute::BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`）。
+    /// 切到 `cpu` 时那一份实现开着的卡全部关掉（`singlefs_core::background_compute::BackgroundGpuCompute::use_no_gpu_from_the_next_batch`）。
     ///
     /// # Errors
     /// 同 [`Self::accept`]。
````

