# m4-gpugc-card-link-check-code-r1：改动全文（两份补丁，按打的次序）

## 第一份：实现

```diff
diff --git a/crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs b/crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs
index 6b0efb7..ad1a5c2 100644
--- a/crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs
+++ b/crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs
@@ -49,8 +49,9 @@ use singlefs_core::background_compute::{
     ClusterSegmentEntryKey, GpuComputedSegment, GpuDeviceIdentifier, GpuEntryCounts,
     GpuEntryDecision, GpuInventory, GpuLeftOutUnderItsMemoryLimit, GpuMemoryInBytes,
     GpuSubmissionsOfOneSegment, GpuSubmissionsOfTheBatch, InputNotSupportedOnTheGpu,
-    RegionRecordEntryKey, RegionRecordToClassifyOnTheGpu, SubmissionWaitedForPastTheLimit,
-    UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+    PciStateOfTheGpuReadWhenJudgedUnusable, RegionRecordEntryKey, RegionRecordToClassifyOnTheGpu,
+    SubmissionWaitedForPastTheLimit, UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
+    BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
 };
 use singlefs_core::compaction_plan::{
     cluster_segment_entry_keys_of_the_unit_area, CompactionRunStop, UnitAreaOfOneDevice,
@@ -184,12 +185,17 @@ impl BothKernelsOfTheThreeSegments for BothKernelsComputedOnTheCpu {
                     cause: GpuFailureCause::NotOpenedAfterTheReopenings {
                         causes_of_each_opening: not_opened.causes_of_each_opening.clone(),
                     },
+                    pci_state_when_judged_unusable: not_opened
+                        .pci_state_when_judged_unusable
+                        .clone(),
                 });
             }
             Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause }) => {
                 return Err(ChecksumBatchFailed::GpuFailed {
                     device: device.clone(),
                     cause: cause.clone(),
+                    pci_state_when_judged_unusable:
+                        PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
                 });
             }
             Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)) => {
@@ -243,12 +249,17 @@ impl BothKernelsOfTheThreeSegments for BothKernelsComputedOnTheCpu {
                     cause: GpuFailureCause::NotOpenedAfterTheReopenings {
                         causes_of_each_opening: not_opened.causes_of_each_opening.clone(),
                     },
+                    pci_state_when_judged_unusable: not_opened
+                        .pci_state_when_judged_unusable
+                        .clone(),
                 });
             }
             Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause }) => {
                 return Err(RegionRecordBatchFailed::GpuFailed {
                     device: device.clone(),
                     cause: cause.clone(),
+                    pci_state_when_judged_unusable:
+                        PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
                 });
             }
             Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)) => {
@@ -313,12 +324,17 @@ impl BothKernelsOfTheThreeSegments for BothKernelsComputedOnTheCpu {
                     cause: GpuFailureCause::NotOpenedAfterTheReopenings {
                         causes_of_each_opening: not_opened.causes_of_each_opening.clone(),
                     },
+                    pci_state_when_judged_unusable: not_opened
+                        .pci_state_when_judged_unusable
+                        .clone(),
                 });
             }
             Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause }) => {
                 return Err(LiveSlotBatchFailed::GpuFailed {
                     device: device.clone(),
                     cause: cause.clone(),
+                    pci_state_when_judged_unusable:
+                        PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
                 });
             }
             Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)) => {
@@ -715,19 +731,29 @@ fn the_batch_that_fails_on_the_gpu_or_times_out_is_reported_as_gpu_failed_and_le
             injected.unit_checksum_batch_failure = Some(ChecksumBatchFailed::GpuFailed {
                 device: card.clone(),
                 cause: cause.clone(),
+                pci_state_when_judged_unusable:
+                    PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
             });
             injected.region_record_batch_failure = Some(RegionRecordBatchFailed::GpuFailed {
                 device: card.clone(),
                 cause: cause.clone(),
+                pci_state_when_judged_unusable:
+                    PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
             });
             injected.live_slot_batch_failure = Some(LiveSlotBatchFailed::GpuFailed {
                 device: card.clone(),
                 cause: cause.clone(),
+                pci_state_when_judged_unusable:
+                    PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
             });
         }
         let failed_round_of_the_implementation =
             one_round_of_the_three_segments(&mut implementation, &inputs);
-        let reported_with_the_card_and_the_cause = core_failure_of_the_gpu(&card, &cause);
+        let reported_with_the_card_and_the_cause = core_failure_of_the_gpu(
+            &card,
+            &cause,
+            &PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+        );
         assert_eq!(
             failed_round_of_the_implementation,
             DecisionsOfTheThreeSegments {
@@ -865,6 +891,7 @@ fn the_card_named_by_a_switch_not_opened_after_the_reopenings_stops_the_compacti
     let not_opened = GpuNotOpenedAfterTheReopenings {
         device: card_not_opened_after_the_reopenings(),
         causes_of_each_opening: causes_of_each_opening.clone(),
+        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
     };
     let mut session = pool.remounted_with_compaction_forced_to_start(
         settings_on_the_card_of_the_test_double(None, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT),
@@ -922,8 +949,15 @@ fn the_card_named_by_a_switch_not_opened_after_the_reopenings_stops_the_compacti
         device: card_not_opened_after_the_reopenings(),
         failures_of_each_opening: causes_of_each_opening
             .iter()
-            .map(|cause| core_failure_of_the_gpu(&card_not_opened_after_the_reopenings(), cause))
+            .map(|cause| {
+                core_failure_of_the_gpu(
+                    &card_not_opened_after_the_reopenings(),
+                    cause,
+                    &PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+                )
+            })
             .collect(),
+        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
     };
     assert_eq!(
         session.background_compute().stopped_after_a_compute_error(),
@@ -1241,6 +1275,7 @@ fn the_gpu_unusable_after_its_resubmissions_stops_the_compaction_reporting_the_c
                 wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
             }),
         },
+        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
     });
     let mut session = pool.remounted_with_compaction_forced_to_start(
         settings_on_the_card_of_the_test_double(None, BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT),
@@ -1273,6 +1308,8 @@ fn the_gpu_unusable_after_its_resubmissions_stops_the_compaction_reporting_the_c
                     wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                     last_submission_waited_for:
                         SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
+                    pci_state_when_judged_unusable:
+                        PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
                 },
             },
             batches_compensated_on_the_cpu: 0,
diff --git a/crates/singlefs-background-gpu/src/background_gpu.rs b/crates/singlefs-background-gpu/src/background_gpu.rs
index 1bbf45b..ae45b24 100644
--- a/crates/singlefs-background-gpu/src/background_gpu.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu.rs
@@ -59,6 +59,7 @@ use singlefs_core::background_compute::{
     AdmittedGpuSettings, BackgroundComputeSettingsRefused, ClusterSegmentEntryKey,
     GpuComputedSegment, GpuDeviceIdentifier, GpuInventory, GpuMemoryInBytes,
     GpuShortOfFreeMemoryForTheSmallestPiece, GpuSubmissionsOfOneSegment, GpuSubmissionsOfTheBatch,
+    PciStateOfTheGpuReadWhenJudgedUnusable,
 };
 use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
 use singlefs_core::logged_operations::CompactionRegion;
@@ -108,6 +109,7 @@ use crate::memory_limit_and_chunks::{
     COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS,
     COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION,
 };
+use crate::pci_state_of_the_gpu::PciStateReaderOfTheGpus;
 use crate::region_record_kernels::{
     decoded_region_record_outcomes, decoded_registers_after_the_pieces,
     little_endian_bytes_of_the_words, packed_region_record_chunk,
@@ -226,9 +228,11 @@ pub enum BackgroundGpuOpenRefused {
     /// `causes_of_each_opening` 是每一次开不成的原因，按开的次序（头一次的在前，1 + 重开次数 个）。
     /// 没点名的卡不开，不走这里。显存上限连最小的一片都装不下的卡不拒，
     /// 不用它、报出来（[`BackgroundGpu::gpus_left_out_under_their_memory_limit`]）。
+    /// `pci_state_when_judged_unusable` 是重开用完、判它开不成那一刻照它的 PCI 地址读的本机 PCI 状态（`crate::pci_state_of_the_gpu`；每一次没开成而还要重开时不读）。
     GpuUnusable {
         device: GpuDeviceIdentifier,
         causes_of_each_opening: Vec<GpuFailureCause>,
+        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
     },
 }
 
@@ -274,10 +278,13 @@ pub struct SmallestPieceNotHeldOnTheGpu {
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum ChecksumBatchFailed {
     /// GPU 出错（含一次提交在同一张卡上重交 3 次都等超过上限、前一次超时的提交在交这一批之前又没等到做完、换设置时量代价出错）：
-    /// 这一批放弃、不换 CPU（第 4 条）。
+    /// 这一批放弃、不换 CPU（第 4 条）。`pci_state_when_judged_unusable` 是判这张卡用不了那一刻读的 PCI 状态
+    /// （等超时用完、卡当场报出错时现读，`crate::pci_state_of_the_gpu::whether_the_cause_judges_the_gpu_unusable_in_a_batch`；
+    /// 切换点到的卡重开到上限都没开成的，带判它开不成那一刻读的那一份；别的没读）。
     GpuFailed {
         device: GpuDeviceIdentifier,
         cause: GpuFailureCause,
+        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
     },
     /// 最小的一片（一个字节的单元）都装不下分到它的那张卡。
     GpuCannotHoldTheSmallestPiece(SmallestPieceNotHeldOnTheGpu),
@@ -306,6 +313,7 @@ pub enum RegionRecordBatchFailed {
     GpuFailed {
         device: GpuDeviceIdentifier,
         cause: GpuFailureCause,
+        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
     },
     /// 映射里有一条 key 长过核装得下的字节数（第一版不支持：核心层写的 key 都是 27 字节）。
     MappingKeyLongerThanTheKernelsHold(MappingKeyLongerThanTheKernelsHold),
@@ -335,6 +343,7 @@ pub enum LiveSlotBatchFailed {
     GpuFailed {
         device: GpuDeviceIdentifier,
         cause: GpuFailureCause,
+        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
     },
     /// 最小的一片（一个段连同一条记录的记录表）都装不下分到它的那张卡。
     GpuCannotHoldTheSmallestPiece(SmallestPieceNotHeldOnTheGpu),
@@ -411,10 +420,13 @@ enum StartOfTheOpeningThread {
 
 /// 挂着切换点到的一张没开的卡，隔一段时间重开到上限也都没开成（第 4 条「3 次都不成，GC 停下，报警里报出哪张卡、每次开不成的原因」），
 /// 或它在这一次挂载里已连着试满、这一次不再开：哪张卡、每一次开不成的原因（按开的次序，头一次的在前，见 [`GpuFailureCause::NotOpenedAfterTheReopenings`]）。
+/// `pci_state_when_judged_unusable` 是判它开不成那一刻照它的 PCI 地址读的本机 PCI 状态（`crate::pci_state_of_the_gpu`，读一次）：
+/// 之后每一批、两批之间每一问报的都是这一份，不重读；下一次换设置清掉它，那张卡再一次被判开不成时才再读。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct GpuNotOpenedAfterTheReopenings {
     pub device: GpuDeviceIdentifier,
     pub causes_of_each_opening: Vec<GpuFailureCause>,
+    pub pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
 }
 
 /// 一张参与的卡对外报的样子（第 2 条：参与的卡都用，并报出来）。
@@ -550,6 +562,9 @@ pub struct BackgroundGpuOverTheCards<Card: CardOfTheBackgroundGpu> {
     /// 此刻本机在的独显（卡清单每次答之前现查）。产品里它带着这个进程的 Vulkan 实例（[`VulkanInstanceOfThisProcess`] 的一份克隆）现枚举：
     /// 排在卡后面，卡都丢了才丢它（实例本身由调用方留着，活过这一次挂载）。
     discrete_gpus_present_now: Box<dyn DiscreteGpusPresentNow + Send>,
+    /// 判一张卡用不了那一刻读它的 PCI 状态的那一份（产品读本机的 PCI 设备目录，测试换成自己的目录）：一批在一张卡上等超时用完或卡当场报出错时、
+    /// 切换点到的卡重开到上限都没开成（或这一次挂载里已试满）时各读一次。
+    pci_state_reader: PciStateReaderOfTheGpus,
 }
 
 /// 产品那一份：开着的卡是 wgpu 设备与六条管线。
@@ -682,6 +697,7 @@ enum OutcomesOfTheBatchNotComputed {
     GpuFailed {
         device: GpuDeviceIdentifier,
         cause: GpuFailureCause,
+        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
     },
     EveryParticipatingGpuLeftOut(Vec<GpuLeftOutUnderItsMemoryLimit>),
     EveryParticipatingGpuShortOfFreeMemory {
@@ -694,9 +710,15 @@ enum OutcomesOfTheBatchNotComputed {
 fn checksum_batch_failed_of(not_computed: OutcomesOfTheBatchNotComputed) -> ChecksumBatchFailed {
     {
         match not_computed {
-            OutcomesOfTheBatchNotComputed::GpuFailed { device, cause } => {
-                ChecksumBatchFailed::GpuFailed { device, cause }
-            }
+            OutcomesOfTheBatchNotComputed::GpuFailed {
+                device,
+                cause,
+                pci_state_when_judged_unusable,
+            } => ChecksumBatchFailed::GpuFailed {
+                device,
+                cause,
+                pci_state_when_judged_unusable,
+            },
             OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(gpus_left_out) => {
                 ChecksumBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(gpus_left_out)
             }
@@ -719,9 +741,15 @@ fn region_record_batch_failed_of(
 ) -> RegionRecordBatchFailed {
     {
         match not_computed {
-            OutcomesOfTheBatchNotComputed::GpuFailed { device, cause } => {
-                RegionRecordBatchFailed::GpuFailed { device, cause }
-            }
+            OutcomesOfTheBatchNotComputed::GpuFailed {
+                device,
+                cause,
+                pci_state_when_judged_unusable,
+            } => RegionRecordBatchFailed::GpuFailed {
+                device,
+                cause,
+                pci_state_when_judged_unusable,
+            },
             OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(gpus_left_out) => {
                 RegionRecordBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(
                     gpus_left_out,
@@ -746,9 +774,15 @@ fn region_record_batch_failed_of(
 fn live_slot_batch_failed_of(not_computed: OutcomesOfTheBatchNotComputed) -> LiveSlotBatchFailed {
     {
         match not_computed {
-            OutcomesOfTheBatchNotComputed::GpuFailed { device, cause } => {
-                LiveSlotBatchFailed::GpuFailed { device, cause }
-            }
+            OutcomesOfTheBatchNotComputed::GpuFailed {
+                device,
+                cause,
+                pci_state_when_judged_unusable,
+            } => LiveSlotBatchFailed::GpuFailed {
+                device,
+                cause,
+                pci_state_when_judged_unusable,
+            },
             OutcomesOfTheBatchNotComputed::EveryParticipatingGpuLeftOut(gpus_left_out) => {
                 LiveSlotBatchFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit(gpus_left_out)
             }
@@ -1057,7 +1091,8 @@ pub fn open_background_gpu(
 /// 量代价出错时账上记着还在跑的提交的那张卡泄漏、不析构，这一次占着的字节当场记进 `memory_leaked_in_this_process`
 /// （`failure_of_the_measurement_dropping_or_leaking_the_card`）。
 /// 没参与的卡不开、不量代价（第 2 条「挂载时只开参与的卡」）：卡清单照列它，之后的切换点到它时在会话锁外开（[`OpeningOfAGpuAtASwitch`]，
-/// 同一份适配器与泄漏账）。参与的卡重开到上限都没开成就拒绝挂载（[`gpus_opened_at_the_mount`]）。参与的卡照开卡那一组给它的显存上限
+/// 同一份适配器与泄漏账）。参与的卡重开到上限都没开成就拒绝挂载（[`gpus_opened_at_the_mount`]），拒绝里带判它开不成那一刻读的本机 PCI 状态
+/// （[`PciStateReaderOfTheGpus::of_this_host`]；挂着之后判卡用不了时也照它读）。参与的卡照开卡那一组给它的显存上限
 /// （先扣掉这个进程里之前几次挂载泄漏在它上面的字节，`memory_leaked_in_this_process`）把量代价那一段（[`COST_PROBE_CHUNK`]）切成几片逐片量、代价取各片之和；
 /// 显存上限连最小的一片都装不下的卡照开、不量代价、不参与、报出来（[`BackgroundGpuOverTheCards::gpus_left_out_under_their_memory_limit`]），挂载不因此拒绝；
 /// 参与的卡因此一张都不剩时照样交回，三段每一批都报「参与的卡一张都不剩」（[`BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut`]），
@@ -1097,12 +1132,14 @@ pub fn open_background_gpu_with_the_memory_leaked_in_this_process(
         memory_leaked_in_this_process.clone(),
         openings_not_returned_in_this_process.clone(),
     );
+    let pci_state_reader = PciStateReaderOfTheGpus::of_this_host();
     let gpus_opened = gpus_opened_at_the_mount(
         discrete_gpus.listed,
         &participating,
         bytes_of_the_smallest_piece_every_participating_gpu_holds(),
         memory_leaked_in_this_process,
         ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT,
+        &pci_state_reader,
         std::thread::sleep,
         |discrete_gpu, memory_limit_at_the_opening, cost_at_the_opening| {
             open_a_gpu(
@@ -1126,6 +1163,7 @@ pub fn open_background_gpu_with_the_memory_leaked_in_this_process(
         }),
         Box::new(open_a_gpu),
         ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT,
+        pci_state_reader,
     ))
 }
 
@@ -1573,9 +1611,11 @@ fn opening_counts_toward_the_failed_openings(cause: &GpuFailureCause) -> bool {
 /// 装不下最小的一片的照开、不量代价（换上那一组时照这张卡的账与开着的卡同一判，再判一次）。`causes_of_each_start` 是起这条线程之前起不了线程的那几次
 /// （各算头一张卡一次没开成、已记进它连着开不成的次数）：原因排在头一张卡的原因前面。
 /// 每一次开卡之前看 `called_off`：置上了就不再开（卸载、切到 `cpu`），也不再睡重开的间隔。
+/// 判一张卡开不成（重开到上限都没开成、或开到它这一刻已试满）那一刻照 `pci_state_reader` 读一次它的 PCI 状态，带进开不成的那一份；
+/// 每一次没开成而还要重开时不读。
 #[allow(
     clippy::too_many_arguments,
-    reason = "参数各是一样东西：开一张卡那一步、要开的卡、等的上限、重开的规矩、起线程之前的原因、叫停、泄漏账与连着开不成的次数"
+    reason = "参数各是一样东西：开一张卡那一步、要开的卡、等的上限、重开的规矩、起线程之前的原因、叫停、泄漏账、连着开不成的次数与读 PCI 状态的那一份"
 )]
 fn gpus_opened_at_a_switch<Card>(
     opening_of_a_gpu_at_a_switch: &mut OpeningOfAGpuAtASwitch<Card>,
@@ -1586,6 +1626,7 @@ fn gpus_opened_at_a_switch<Card>(
     called_off: &AtomicBool,
     memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
     failed_openings_in_this_mount: &FailedOpeningsOfEachGpuInThisMount,
+    pci_state_reader: &PciStateReaderOfTheGpus,
 ) -> OutcomeOfTheOpeningAtASwitch<Card> {
     let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
     let mut opened = Vec::with_capacity(discrete_gpus_to_open.len());
@@ -1605,6 +1646,8 @@ fn gpus_opened_at_a_switch<Card>(
             return OutcomeOfTheOpeningAtASwitch {
                 opened,
                 not_opened: Some(GpuNotOpenedAfterTheReopenings {
+                    pci_state_when_judged_unusable: pci_state_reader
+                        .pci_state_of_the_gpu(&discrete_gpu.identifier),
                     device: discrete_gpu.identifier,
                     causes_of_each_opening,
                 }),
@@ -1655,6 +1698,8 @@ fn gpus_opened_at_a_switch<Card>(
                 return OutcomeOfTheOpeningAtASwitch {
                     opened,
                     not_opened: Some(GpuNotOpenedAfterTheReopenings {
+                        pci_state_when_judged_unusable: pci_state_reader
+                            .pci_state_of_the_gpu(&discrete_gpu.identifier),
                         device: discrete_gpu.identifier,
                         causes_of_each_opening,
                     }),
@@ -2664,7 +2709,8 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
     /// 挂载那一刻开卡之后：开成的卡（与 `discrete_gpus_after_the_opening.opened` 一一对应、同序）、开卡之后的本机独显（连同没参与而没开的）、没点名而没收进清单的那几张、
     /// 开卡那一组参与的卡与因显存上限不用的卡（`gpus_of_this_host::gpus_opened_at_the_mount` 交回的），进程里泄漏的那一份（丢它时记进去），
     /// 卡清单每次答之前现查本机在的独显的那一样（产品带着开卡用的那一份进程级 Vulkan 实例），切换点到没开的卡时开它一次的那一步，
-    /// 与开不成时怎么重开（产品是 [`ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT`]）。测试拿它把替身卡装进来。
+    /// 开不成时怎么重开（产品是 [`ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT`]），与判一张卡用不了那一刻读它的 PCI 状态的那一份
+    /// （产品是 [`PciStateReaderOfTheGpus::of_this_host`]）。测试拿它把替身卡装进来。
     ///
     /// # Panics
     /// 开成的卡与 `discrete_gpus_after_the_opening.opened` 张数不等。
@@ -2683,6 +2729,7 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         discrete_gpus_present_now: Box<dyn DiscreteGpusPresentNow + Send>,
         opening_of_a_gpu_at_a_switch: OpeningOfAGpuAtASwitch<Card>,
         reopenings_of_a_gpu_not_opened: ReopeningsOfAGpuNotOpened,
+        pci_state_reader: PciStateReaderOfTheGpus,
     ) -> Self {
         assert_eq!(
             cards_on_this_host.len(),
@@ -2715,6 +2762,7 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
             gpus_leaked_when_dropped: GpusLeakedWhenTheBackgroundGpuIsDropped::not_yet_dropped(),
             memory_leaked_in_this_process,
             discrete_gpus_present_now,
+            pci_state_reader,
         }
     }
 
@@ -2753,6 +2801,12 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         self.gpus_leaked_when_closed_at_a_switch.clone()
     }
 
+    /// 这一次挂载里判卡用不了时一共读了几次 PCI 状态（切换开卡线程上读的也算；`.claude/rules/fs-design.md` 五条硬要求第 4 条：看得出读了没有）。
+    #[must_use]
+    pub fn pci_state_readings_taken(&self) -> u64 {
+        self.pci_state_reader.readings_taken()
+    }
+
     /// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：接下来 `refusals` 次起切换开卡那条线程都照起不了报，强制进入「起不了开卡线程」那一支。
     pub fn refuse_to_start_the_opening_thread_for_tests(&mut self, refusals: u32) {
         self.start_of_the_opening_thread = StartOfTheOpeningThread::RefusedForTests {
@@ -3051,7 +3105,7 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
     }
 
     /// `discrete_gpus_to_open` 里头一张在这一次挂载里已连着试满（开不成的次数到了 [`ReopeningsOfAGpuNotOpened::openings_at_most`]）的卡，
-    /// 记成它开不成（[`GpuFailureCause::OpeningsInThisMountUsedUp`]，只一个原因）；都没试满交回 `None`。
+    /// 记成它开不成（[`GpuFailureCause::OpeningsInThisMountUsedUp`]，只一个原因），带判它开不成这一刻读的它的 PCI 状态；都没试满交回 `None`（不读）。
     fn gpu_whose_openings_in_this_mount_are_used_up(
         &self,
         discrete_gpus_to_open: &[(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)],
@@ -3073,6 +3127,9 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
                         &self.failed_openings_in_this_mount,
                         &discrete_gpu.identifier,
                     )],
+                    pci_state_when_judged_unusable: self
+                        .pci_state_reader
+                        .pci_state_of_the_gpu(&discrete_gpu.identifier),
                 },
             )
     }
@@ -3136,6 +3193,7 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         let opening_of_a_gpu_at_a_switch = Arc::clone(&self.opening_of_a_gpu_at_a_switch);
         let memory_leaked_in_this_process = self.memory_leaked_in_this_process.clone();
         let failed_openings_in_this_mount = self.failed_openings_in_this_mount.clone();
+        let pci_state_reader = self.pci_state_reader.clone();
         let reopenings = self.reopenings_of_a_gpu_not_opened;
         let called_off = Arc::new(AtomicBool::new(false));
         let called_off_seen_by_the_thread = Arc::clone(&called_off);
@@ -3155,6 +3213,7 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
                 &called_off_seen_by_the_thread,
                 &memory_leaked_in_this_process,
                 &failed_openings_in_this_mount,
+                &pci_state_reader,
             );
             drop(opening_of_a_gpu);
             // 送不出去只有一种：后台 GPU 已经丢了（卸载）或切到了 `cpu`，开出来的卡没人要，在这里照常丢。
@@ -3201,6 +3260,9 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
             outcome: OutcomeOfTheOpeningAtASwitch {
                 opened: Vec::new(),
                 not_opened: Some(GpuNotOpenedAfterTheReopenings {
+                    pci_state_when_judged_unusable: self
+                        .pci_state_reader
+                        .pci_state_of_the_gpu(&first_device_to_open),
                     device: first_device_to_open,
                     causes_of_each_opening: causes_of_each_start,
                 }),
@@ -3389,17 +3451,26 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
                 GpuNotOpenedAfterTheReopenings {
                     device,
                     causes_of_each_opening,
+                    pci_state_when_judged_unusable,
                 },
             )) => {
+                // 判它开不成那一刻读过了：每一批带的都是那一份，不重读。
                 return Err(OutcomesOfTheBatchNotComputed::GpuFailed {
                     device,
                     cause: GpuFailureCause::NotOpenedAfterTheReopenings {
                         causes_of_each_opening,
                     },
+                    pci_state_when_judged_unusable,
                 });
             }
             Some(BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause }) => {
-                return Err(OutcomesOfTheBatchNotComputed::GpuFailed { device, cause });
+                // 换设置时量代价出错不是判一张卡用不了的那三种时刻（这张卡没参与）：不读。
+                return Err(OutcomesOfTheBatchNotComputed::GpuFailed {
+                    device,
+                    cause,
+                    pci_state_when_judged_unusable:
+                        PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+                });
             }
             Some(BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)) => {
                 if entries.is_empty() {
@@ -3435,6 +3506,7 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
                 },
             );
         }
+        let pci_state_reader = &self.pci_state_reader;
         let cards_on_this_host = &mut self.cards_on_this_host;
         let first_entry = entries.start;
         let shifted_by_the_first_entry =
@@ -3478,7 +3550,12 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
         })
         .map_err(|not_planned| match not_planned {
             BatchNotPlannedOverTheGpus::GpuFailed { device, cause } => {
-                OutcomesOfTheBatchNotComputed::GpuFailed { device, cause }
+                OutcomesOfTheBatchNotComputed::GpuFailed {
+                    pci_state_when_judged_unusable: pci_state_reader
+                        .pci_state_if_the_cause_judges_the_gpu_unusable(&device, &cause),
+                    device,
+                    cause,
+                }
             }
             BatchNotPlannedOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out)
                 if gpus_short_of_free_memory.is_empty() =>
@@ -3611,7 +3688,13 @@ impl<Card: CardOfTheBackgroundGpu> BackgroundGpuOverTheCards<Card> {
                         entries_to_hand_out.push(planned.entries);
                     }
                     OutcomesOfACardInTheBatch::Failed(cause) => {
-                        failure = Some(OutcomesOfTheBatchNotComputed::GpuFailed { device, cause });
+                        failure = Some(OutcomesOfTheBatchNotComputed::GpuFailed {
+                            pci_state_when_judged_unusable: self
+                                .pci_state_reader
+                                .pci_state_if_the_cause_judges_the_gpu_unusable(&device, &cause),
+                            device,
+                            cause,
+                        });
                         break 'handing_out;
                     }
                 }
@@ -4260,6 +4343,10 @@ mod tests {
     use crate::checksum_kernels::cpu_reference_outcome;
     use crate::live_slot_kernels::live_slot_chunk_footprint;
     use crate::memory_limit_and_chunks::{GpuMemoryAccount, SubmissionOfTheChunk};
+    use crate::pci_state_of_the_gpu::{
+        pci_state_read_under_the_root_not_present_for_tests,
+        pci_state_reader_under_a_root_not_present_for_tests,
+    };
     use crate::two_kernel_reconciliation::EntryDecision;
 
     const WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU: Duration = Duration::from_secs(10);
@@ -4284,7 +4371,7 @@ mod tests {
     /// 交了几次），账与产品那张卡同一份（`GpuBudgetUnderTheSettings`）。按脚本造三样：交活之前、重交之前等前一次超时的那次提交等满上限没等到
     /// （`earlier_submission_waits_timing_out`，按次序，`true` 是没等到；卡上记着一次还在跑的提交时才等、才取脚本），每一次提交等超过上限
     /// （`submissions_timing_out`，按次序，`true` 是超时，超时的那一段的账照产品记成还在跑的那次提交），卡上空闲的显存（`free_bytes`：一段合占的比它多就建不成缓冲）。
-    /// 脚本用完之后都等到、都做完。另记量了几次代价（`cost_measurements`；`cost_measurement_failing_with` 给了时每次量都交回那个错、不记代价；
+    /// 脚本用完之后都等到、都做完；`next_submission_failing_with` 给了时下一次交一段当场交回那个错（只一次，例：设备丢了），放掉那一段的账。另记量了几次代价（`cost_measurements`；`cost_measurement_failing_with` 给了时每次量都交回那个错、不记代价；
     /// `cost_measured_through_the_checksum_chunk` 时照产品那样把量代价那一段照此刻剩下的界切片、逐段当一次提交走账：先记账、超时就记成还在跑），可点名第几次交成的校验和那一段交回翻过的判定
     /// （`checksum_submission_with_flipped_verdicts`，从 1 数：那一次的结果是坏的，用了它判定就与 CPU 参照不同）。
     /// 析构一次就在 `drops_counted` 上加一（泄漏的卡不析构，计数不动）：用例在卡交出去之前取一份它的克隆。
@@ -4301,6 +4388,7 @@ mod tests {
         cost_measurement_failing_with: Option<GpuFailureCause>,
         drops_counted: Arc<std::sync::atomic::AtomicU32>,
         cost_measured_through_the_checksum_chunk: bool,
+        next_submission_failing_with: Option<GpuFailureCause>,
     }
 
     impl Drop for CardComputedOnTheCpuForTests {
@@ -4342,6 +4430,7 @@ mod tests {
                 cost_measurement_failing_with: None,
                 drops_counted: Arc::new(std::sync::atomic::AtomicU32::new(0)),
                 cost_measured_through_the_checksum_chunk: false,
+                next_submission_failing_with: None,
             }
         }
 
@@ -4381,6 +4470,12 @@ mod tests {
                     reason: "替身卡上空闲的不够".to_string(),
                 });
             }
+            if let Some(cause) = self.next_submission_failing_with.take() {
+                self.budget
+                    .memory_account_mut()
+                    .release(bytes_held_by_the_chunk);
+                return Err(cause);
+            }
             self.submissions_made += 1;
             if self.submissions_timing_out.pop_front().unwrap_or(false) {
                 self.budget
@@ -4616,6 +4711,7 @@ mod tests {
                 Box::new(DiscreteGpusPresentForTests(Arc::clone(&present))),
                 no_opening_at_a_switch(),
                 REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+                pci_state_reader_under_a_root_not_present_for_tests(),
             ),
             present,
         )
@@ -4685,6 +4781,9 @@ mod tests {
                         wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
                     }),
                 },
+                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
+                    "0000:01:00.0"
+                ),
             }),
             "四段各超时一次：这一批第 4 次超时报警"
         );
@@ -4731,6 +4830,145 @@ mod tests {
         );
     }
 
+    /// 外壳装上一张「0000:01:00.0」替身卡（照 `card_as_the_test_wants` 改过），交一批一条 1000 字节的单元，交回这一批的结局与这一次挂载一共读了几次 PCI 状态。
+    fn checksum_batch_on_the_card_at_one_and_the_pci_state_readings(
+        card_as_the_test_wants: impl FnOnce(
+            CardComputedOnTheCpuForTests,
+        ) -> CardComputedOnTheCpuForTests,
+    ) -> (BothChecksumKernelOutcomesOfTheBatch, u64) {
+        let card = card_as_the_test_wants(CardComputedOnTheCpuForTests::of_the_address(
+            "0000:01:00.0",
+            None,
+            0,
+        ));
+        let (mut background_gpu, _present) = background_gpu_over_cards_on_the_cpu(
+            vec![card],
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+        );
+        let unit_bytes = unit_bytes_of(1, 1000);
+        let outcome =
+            background_gpu.both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes));
+        (outcome, background_gpu.pci_state_readings_taken())
+    }
+
+    /// 第 4 条「挂着时一批在同一张卡上等超时、到判那张卡用不了」那一刻查一次这张卡的硬件与链路：一批 1 片在这张卡上 4 次都等超过上限——
+    /// 报这张卡用不了，带判它用不了那一刻照它的地址读的 PCI 状态；3 次超时重交之前都不读，一共只读 1 次。
+    #[test]
+    fn the_card_unusable_after_its_resubmissions_is_read_once_and_the_batch_failure_carries_the_reading(
+    ) {
+        let (outcome, pci_state_readings) =
+            checksum_batch_on_the_card_at_one_and_the_pci_state_readings(|mut card| {
+                card.submissions_timing_out = VecDeque::from([true, true, true, true]);
+                card
+            });
+        assert_eq!(
+            outcome,
+            Err(ChecksumBatchFailed::GpuFailed {
+                device: card_identifier("0000:01:00.0"),
+                cause: GpuFailureCause::UnusableAfterResubmissions {
+                    resubmissions: 3,
+                    last_cause: Box::new(GpuFailureCause::WaitForTheGpuExceededTheLimit {
+                        wait_limit: WAIT_LIMIT_OF_THE_CARDS_ON_THE_CPU,
+                    }),
+                },
+                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
+                    "0000:01:00.0"
+                ),
+            })
+        );
+        assert_eq!(pci_state_readings, 1);
+    }
+
+    /// 同一条「平常的批、每次超时重交之前不查」：一批 1 片超时 3 次、第 4 次交做完——照常交回，一次都不读。
+    #[test]
+    fn the_batch_completing_after_three_timeouts_reads_no_pci_state() {
+        let (outcome, pci_state_readings) =
+            checksum_batch_on_the_card_at_one_and_the_pci_state_readings(|mut card| {
+                card.submissions_timing_out = VecDeque::from([true, true, true]);
+                card
+            });
+        assert!(outcome.is_ok(), "第 4 次交做完：照常交回（{outcome:?}）");
+        assert_eq!(pci_state_readings, 0);
+    }
+
+    /// 第 4 条「卡当场报出错（设备丢失、驱动报的未捕获的错）、后台任务因此停下」那一刻查一次：这一批交到卡上时设备丢了——报这张卡的 GPU 出错，
+    /// 带那一刻读的 PCI 状态，一共只读 1 次。
+    #[test]
+    fn the_card_lost_in_a_batch_is_read_once_and_the_batch_failure_carries_the_reading() {
+        let (outcome, pci_state_readings) =
+            checksum_batch_on_the_card_at_one_and_the_pci_state_readings(|mut card| {
+                card.next_submission_failing_with =
+                    Some(GpuFailureCause::UncapturedErrorOrDeviceLost {
+                        reason: "替身：设备丢了".to_string(),
+                    });
+                card
+            });
+        assert_eq!(
+            outcome,
+            Err(ChecksumBatchFailed::GpuFailed {
+                device: card_identifier("0000:01:00.0"),
+                cause: GpuFailureCause::UncapturedErrorOrDeviceLost {
+                    reason: "替身：设备丢了".to_string(),
+                },
+                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
+                    "0000:01:00.0"
+                ),
+            })
+        );
+        assert_eq!(pci_state_readings, 1);
+    }
+
+    /// 同一条：一批里等提交没等成（不是设备丢了、不是超时用完）不是判卡用不了的那三种时刻——照报这张卡的 GPU 出错，读数那一格是没读，一次都不读。
+    #[test]
+    fn a_batch_failing_without_judging_the_card_unusable_reads_no_pci_state() {
+        let (outcome, pci_state_readings) =
+            checksum_batch_on_the_card_at_one_and_the_pci_state_readings(|mut card| {
+                card.next_submission_failing_with =
+                    Some(GpuFailureCause::WaitForTheSubmissionFailed {
+                        reason: "替身：没等成".to_string(),
+                    });
+                card
+            });
+        assert_eq!(
+            outcome,
+            Err(ChecksumBatchFailed::GpuFailed {
+                device: card_identifier("0000:01:00.0"),
+                cause: GpuFailureCause::WaitForTheSubmissionFailed {
+                    reason: "替身：没等成".to_string(),
+                },
+                pci_state_when_judged_unusable:
+                    PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+            })
+        );
+        assert_eq!(pci_state_readings, 0);
+    }
+
+    /// 第 4 条「等超时用完到判那张卡用不了」也算交活之前那一等：卡上记着前一次超时、还在跑的提交，这一批交活之前 4 次都等满上限没等到——
+    /// 报这张卡用不了（一次都没重交到卡上），带那一刻读的 PCI 状态，一共只读 1 次。
+    #[test]
+    fn the_card_unusable_while_waiting_before_the_batch_is_read_once_and_the_failure_carries_the_reading(
+    ) {
+        let (outcome, pci_state_readings) =
+            checksum_batch_on_the_card_at_one_and_the_pci_state_readings(|card| {
+                let mut card = card.with_a_submission_running_past_its_wait(100);
+                card.earlier_submission_waits_timing_out = VecDeque::from([true, true, true, true]);
+                card
+            });
+        let Err(ChecksumBatchFailed::GpuFailed {
+            cause: GpuFailureCause::UnusableAfterResubmissions { .. },
+            pci_state_when_judged_unusable,
+            ..
+        }) = &outcome
+        else {
+            panic!("交活之前 4 次都没等到：报这张卡用不了，结局是 {outcome:?}");
+        };
+        assert_eq!(
+            pci_state_when_judged_unusable,
+            &pci_state_read_under_the_root_not_present_for_tests("0000:01:00.0")
+        );
+        assert_eq!(pci_state_readings, 1);
+    }
+
     /// 两条区域扫描会去读的记录（R 是第一块盘上槽 57600 起那一段）与空映射。
     fn two_region_records_with_an_empty_mapping() -> (CompactionRegion, Vec<AllocationRecord>) {
         let region =
@@ -5288,6 +5526,7 @@ mod tests {
             ])))),
             opening_noting_the_cards,
             reopenings,
+            pci_state_reader_under_a_root_not_present_for_tests(),
         );
         (background_gpu, cards_handed_to_the_opening)
     }
@@ -5394,6 +5633,7 @@ mod tests {
             ])))),
             opening_noting_the_cards,
             REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+            pci_state_reader_under_a_root_not_present_for_tests(),
         );
         (
             background_gpu,
@@ -5678,6 +5918,9 @@ mod tests {
             );
         let not_opened = GpuNotOpenedAfterTheReopenings {
             device: card_identifier("0000:41:00.0"),
+            pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
+                "0000:41:00.0",
+            ),
             causes_of_each_opening: vec![reason_the_card_does_not_open_at_the_switch(); 4],
         };
         let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
@@ -5725,6 +5968,7 @@ mod tests {
                 cause: GpuFailureCause::NotOpenedAfterTheReopenings {
                     causes_of_each_opening: not_opened.causes_of_each_opening,
                 },
+                pci_state_when_judged_unusable: not_opened.pci_state_when_judged_unusable,
             }),
             "之后那一批报成这张卡的 GPU 出错"
         );
@@ -5749,6 +5993,88 @@ mod tests {
         );
     }
 
+    /// 第 4 条「开卡重开用完都开不成」那一刻查一次、「这张卡再一次被判用不了时才再读一次」：切换只点「0000:41:00.0」，它头一次与 3 次重开都开不成——
+    /// 判它开不成那一刻（会话锁外的开卡线程上）读 1 次；之后两批之间连问 3 次、连交 2 批，报的都是那一份，不再读。
+    #[test]
+    fn the_switch_not_opened_is_read_once_and_every_later_answer_and_batch_carries_that_reading() {
+        let (mut background_gpu, _cards_handed_to_the_opening) =
+            background_gpu_with_one_card_not_opened_at_the_mount(
+                |_discrete_gpu, _memory_limit, _wait_limit, _cost_at_the_opening| {
+                    Err(reason_the_card_does_not_open_at_the_switch())
+                },
+                REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+                &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            );
+        let admitted = switch_naming_one_card_admitted_against(&background_gpu, "0000:41:00.0");
+        background_gpu.use_the_settings_from_the_next_batch(&admitted);
+        let pci_state_read = pci_state_read_under_the_root_not_present_for_tests("0000:41:00.0");
+        let answers: Vec<LatestSettingsHandedIn> = (0..3)
+            .map(|_question| {
+                background_gpu.put_the_gpus_opened_since_the_switch_in_effect(
+                    WaitForTheOpeningAtASwitch::UntilItEnds,
+                )
+            })
+            .collect();
+        assert!(
+            answers.iter().all(|answer| matches!(
+                answer,
+                LatestSettingsHandedIn::NotOpened(not_opened)
+                    if not_opened.pci_state_when_judged_unusable == pci_state_read
+            )),
+            "每一问都答开不成、带判它开不成那一刻读的那一份：{answers:?}"
+        );
+        let batch_outcomes: Vec<BothChecksumKernelOutcomesOfTheBatch> = (0..2)
+            .map(|_batch| checksum_batch_and_the_devices_given_work(&mut background_gpu).0)
+            .collect();
+        assert!(
+            batch_outcomes.iter().all(|outcome| matches!(
+                outcome,
+                Err(ChecksumBatchFailed::GpuFailed {
+                    pci_state_when_judged_unusable,
+                    ..
+                }) if *pci_state_when_judged_unusable == pci_state_read
+            )),
+            "之后每一批都报成这张卡的 GPU 出错、带同一份读数：{batch_outcomes:?}"
+        );
+        assert_eq!(background_gpu.pci_state_readings_taken(), 1);
+    }
+
+    /// 同一条「开卡每一次没开成而还要重开时都不查」：切换点到的「0000:41:00.0」头两次开不成、第三次开成——换上了，一次都不读。
+    #[test]
+    fn the_card_opened_at_its_third_try_at_a_switch_reads_no_pci_state() {
+        let openings_made = Arc::new(std::sync::atomic::AtomicU32::new(0));
+        let openings_counted = Arc::clone(&openings_made);
+        let (mut background_gpu, _cards_handed_to_the_opening) =
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
+            LatestSettingsHandedIn::PutInEffect {
+                not_split_over_the_gpus: None
+            }
+        );
+        assert_eq!(background_gpu.pci_state_readings_taken(), 0);
+    }
+
     /// 第 2 条「切换之后不再参与的卡，切换生效时关掉」：挂载开着「0000:01:00.0」「0000:41:00.0」，切换只点「0000:01:00.0」——
     /// 换上那一刻「0000:41:00.0」关掉（替身卡析构一次、不再开着），卡清单照列它；再切到两张都点，它重开（开卡那一步走一次），两张都参与。
     #[test]
@@ -5885,6 +6211,9 @@ mod tests {
             ),
             LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
                 device: card_identifier("0000:41:00.0"),
+                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
+                    "0000:41:00.0"
+                ),
                 causes_of_each_opening: vec![failure_of_the_cost_measurement_in_the_test(); 4],
             }),
             "切换按这张卡重开到上限都没开成报，每一次的原因是量代价那一次的错"
@@ -5977,6 +6306,7 @@ mod tests {
             ])))),
             opening_noting_the_cards,
             REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+            pci_state_reader_under_a_root_not_present_for_tests(),
         );
         (background_gpu, cards_handed_to_the_opening)
     }
@@ -6214,6 +6544,9 @@ mod tests {
             answered,
             LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
                 device: card_identifier("0000:41:00.0"),
+                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
+                    "0000:41:00.0"
+                ),
                 causes_of_each_opening: vec![GpuFailureCause::OpeningsInThisMountUsedUp {
                     failed_openings: 4
                 }],
@@ -6591,6 +6924,9 @@ mod tests {
             answered,
             LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
                 device: card_identifier("0000:41:00.0"),
+                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
+                    "0000:41:00.0"
+                ),
                 causes_of_each_opening: vec![
                     GpuFailureCause::OpeningPanicked {
                         said_by_the_panic: "替身：开卡次次 panic".to_string(),
@@ -6661,6 +6997,9 @@ mod tests {
             answered,
             LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
                 device: card_identifier("0000:41:00.0"),
+                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
+                    "0000:41:00.0"
+                ),
                 causes_of_each_opening: vec![
                     GpuFailureCause::WaitForTheGpuExceededTheLimit {
                         wait_limit: OPENING_LIMIT_OF_THE_TEST,
@@ -7146,6 +7485,9 @@ mod tests {
                         }
                     ),
                 },
+                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
+                    "0000:01:00.0"
+                ),
             })
         );
         assert_eq!(
@@ -7895,6 +8237,9 @@ mod tests {
     fn not_opened_with_the_openings_of_the_card_at_forty_one_used_up() -> LatestSettingsHandedIn {
         LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
             device: card_identifier("0000:41:00.0"),
+            pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
+                "0000:41:00.0",
+            ),
             causes_of_each_opening: vec![GpuFailureCause::OpeningsInThisMountUsedUp {
                 failed_openings: 4,
             }],
@@ -8007,6 +8352,7 @@ mod tests {
             bytes_of_the_smallest_piece_every_participating_gpu_holds(),
             &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
             REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+            &pci_state_reader_under_a_root_not_present_for_tests(),
             |_interval: Duration| {},
             |gpu_to_open, memory_limit_at_the_opening, cost_at_the_opening| {
                 opening_at_the_mount(
@@ -8194,6 +8540,9 @@ mod tests {
             ),
             LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
                 device: card_identifier("0000:41:00.0"),
+                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
+                    "0000:41:00.0"
+                ),
                 causes_of_each_opening: vec![reason_the_card_does_not_open_at_the_switch(); 2],
             }),
             "连着开不成 2 次之后只剩 2 次可开"
@@ -9375,6 +9724,9 @@ mod tests {
     ) -> LatestSettingsHandedIn {
         LatestSettingsHandedIn::NotOpened(GpuNotOpenedAfterTheReopenings {
             device: card_identifier("0000:41:00.0"),
+            pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
+                "0000:41:00.0",
+            ),
             causes_of_each_opening,
         })
     }
diff --git a/crates/singlefs-background-gpu/src/background_gpu_compute.rs b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
index c5c74b4..0ddf36b 100644
--- a/crates/singlefs-background-gpu/src/background_gpu_compute.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
@@ -26,8 +26,8 @@ use singlefs_core::background_compute::{
     BackgroundGpuComputeHandle, ClusterSegmentEntryKey, GpuDeviceIdentifier, GpuEntryDecision,
     GpuFailureOnTheDevice, GpuInventory, GpuMemoryInBytes, GpuShortOfFreeMemoryForTheSmallestPiece,
     GpuSubmissionsOfTheBatch, InputNotSupportedOnTheGpu, LatestGpuSettingsHandedIn,
-    RegionRecordEntryKey, RegionRecordToClassifyOnTheGpu, SubmissionWaitedForPastTheLimit,
-    UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
+    PciStateOfTheGpuReadWhenJudgedUnusable, RegionRecordEntryKey, RegionRecordToClassifyOnTheGpu,
+    SubmissionWaitedForPastTheLimit, UnitChecksumOutcome, UnitChecksumToVerifyOnTheGpu,
 };
 use singlefs_core::compaction_plan::UnitAreaOfOneDevice;
 use singlefs_core::logged_operations::CompactionRegion;
@@ -295,15 +295,19 @@ pub fn core_decision_of_the_entry<KernelOutcome, CoreOutcome>(
 /// 一张卡上的 GPU 出错报给核心层：等超过上限的三种（这一批的那一次、前一次超时的那一次没做完、冲掉待写的那一次空提交没做完）是
 /// `WaitForTheGpuExceededTheLimit`，带上限本身与是哪一次提交；重交 3 次都等超过上限的（这张卡用不了，第 4 条「都超时才报警」）是
 /// `GpuUnusableAfterResubmissions`，带重交了几次、最后那一次等的上限与是哪一次提交；别的是 `GpuFailed`，原因一一对过去（开卡那几种各对各的成员）、原话照带。卡照带。
+/// `pci_state_when_judged_unusable`（判这张卡用不了那一刻外壳读的 PCI 状态，没读的是没读）照带进带读数的那三种（`GpuFailed`、`GpuUnusableAfterResubmissions`、
+/// `GpuNotOpenedAfterTheReopenings`）；重开到上限都没开成的，每一次开不成报成的那一个带的是没读（每一次没开成而还要重开时不读）。这里不读。
 #[must_use]
 pub fn core_failure_of_the_gpu(
     device: &GpuDeviceIdentifier,
     cause: &GpuFailureCause,
+    pci_state_when_judged_unusable: &PciStateOfTheGpuReadWhenJudgedUnusable,
 ) -> BackgroundGpuComputeFailed {
     let failed_on_the_device =
         |cause_on_the_device: GpuFailureOnTheDevice| BackgroundGpuComputeFailed::GpuFailed {
             device: device.clone(),
             cause: cause_on_the_device,
+            pci_state_when_judged_unusable: pci_state_when_judged_unusable.clone(),
         };
     match cause {
         GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit } => {
@@ -394,6 +398,7 @@ pub fn core_failure_of_the_gpu(
                     wait_limit: *wait_limit,
                     last_submission_waited_for:
                         SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
+                    pci_state_when_judged_unusable: pci_state_when_judged_unusable.clone(),
                 }
             }
             GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
@@ -404,6 +409,7 @@ pub fn core_failure_of_the_gpu(
                 wait_limit: *wait_limit,
                 last_submission_waited_for:
                     SubmissionWaitedForPastTheLimit::EarlierTimedOutSubmissionStillRunning,
+                pci_state_when_judged_unusable: pci_state_when_judged_unusable.clone(),
             },
             GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { wait_limit } => {
                 BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
@@ -412,6 +418,7 @@ pub fn core_failure_of_the_gpu(
                     wait_limit: *wait_limit,
                     last_submission_waited_for:
                         SubmissionWaitedForPastTheLimit::EmptySubmissionFlushingThePendingWrites,
+                    pci_state_when_judged_unusable: pci_state_when_judged_unusable.clone(),
                 }
             }
             // 只有等超过上限的三种才重交（`crate::waiting_for_the_gpu::outcome_resubmitted_after_timeouts`），别的原因当场交回、不包进这一种；
@@ -431,7 +438,11 @@ pub fn core_failure_of_the_gpu(
             | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
             | GpuFailureCause::UnusableAfterResubmissions { .. }
             | GpuFailureCause::NotOpenedAfterTheReopenings { .. }) => {
-                core_failure_of_the_gpu(device, last_cause_other_than_a_wait_past_the_limit)
+                core_failure_of_the_gpu(
+                    device,
+                    last_cause_other_than_a_wait_past_the_limit,
+                    pci_state_when_judged_unusable,
+                )
             }
         },
         GpuFailureCause::NotOpenedAfterTheReopenings {
@@ -440,8 +451,15 @@ pub fn core_failure_of_the_gpu(
             device: device.clone(),
             failures_of_each_opening: causes_of_each_opening
                 .iter()
-                .map(|cause_of_the_opening| core_failure_of_the_gpu(device, cause_of_the_opening))
+                .map(|cause_of_the_opening| {
+                    core_failure_of_the_gpu(
+                        device,
+                        cause_of_the_opening,
+                        &PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+                    )
+                })
                 .collect(),
+            pci_state_when_judged_unusable: pci_state_when_judged_unusable.clone(),
         },
     }
 }
@@ -481,7 +499,11 @@ pub fn core_failure_of_the_batch_not_split_over_the_gpus(
             core_failure_of_the_gpu_not_opened_after_the_reopenings(not_opened)
         }
         BatchNotSplitOverTheGpus::CostNotMeasuredAtTheSwitch { device, cause } => {
-            core_failure_of_the_gpu(device, cause)
+            core_failure_of_the_gpu(
+                device,
+                cause,
+                &PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+            )
         }
         BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut(gpus_left_out) => {
             core_failure_of_every_participating_gpu_left_out(gpus_left_out)
@@ -499,6 +521,7 @@ pub fn core_failure_of_the_gpu_not_opened_after_the_reopenings(
         &GpuFailureCause::NotOpenedAfterTheReopenings {
             causes_of_each_opening: not_opened.causes_of_each_opening.clone(),
         },
+        &not_opened.pci_state_when_judged_unusable,
     )
 }
 
@@ -533,7 +556,11 @@ pub fn core_failure_of_the_checksum_batch(
     failed: &ChecksumBatchFailed,
 ) -> BackgroundGpuComputeFailed {
     match failed {
-        ChecksumBatchFailed::GpuFailed { device, cause } => core_failure_of_the_gpu(device, cause),
+        ChecksumBatchFailed::GpuFailed {
+            device,
+            cause,
+            pci_state_when_judged_unusable,
+        } => core_failure_of_the_gpu(device, cause, pci_state_when_judged_unusable),
         ChecksumBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece_not_held) => {
             core_failure_of_the_smallest_piece_not_held(smallest_piece_not_held)
         }
@@ -560,9 +587,11 @@ pub fn core_failure_of_the_region_record_batch(
     failed: &RegionRecordBatchFailed,
 ) -> BackgroundGpuComputeFailed {
     match failed {
-        RegionRecordBatchFailed::GpuFailed { device, cause } => {
-            core_failure_of_the_gpu(device, cause)
-        }
+        RegionRecordBatchFailed::GpuFailed {
+            device,
+            cause,
+            pci_state_when_judged_unusable,
+        } => core_failure_of_the_gpu(device, cause, pci_state_when_judged_unusable),
         RegionRecordBatchFailed::MappingKeyLongerThanTheKernelsHold(longer) => {
             BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(
                 InputNotSupportedOnTheGpu::MappingKeyLongerThanTheKernelsHold {
@@ -596,7 +625,11 @@ pub fn core_failure_of_the_live_slot_batch(
     failed: &LiveSlotBatchFailed,
 ) -> BackgroundGpuComputeFailed {
     match failed {
-        LiveSlotBatchFailed::GpuFailed { device, cause } => core_failure_of_the_gpu(device, cause),
+        LiveSlotBatchFailed::GpuFailed {
+            device,
+            cause,
+            pci_state_when_judged_unusable,
+        } => core_failure_of_the_gpu(device, cause, pci_state_when_judged_unusable),
         LiveSlotBatchFailed::GpuCannotHoldTheSmallestPiece(smallest_piece_not_held) => {
             core_failure_of_the_smallest_piece_not_held(smallest_piece_not_held)
         }
@@ -786,6 +819,8 @@ mod tests {
             |cause: GpuFailureOnTheDevice| BackgroundGpuComputeFailed::GpuFailed {
                 device: card_named_in_the_failures(),
                 cause,
+                pci_state_when_judged_unusable:
+                    PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
             };
         vec![
             (
@@ -922,6 +957,8 @@ mod tests {
                                 SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
                         },
                     ],
+                    pci_state_when_judged_unusable:
+                        PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
                 },
             ),
         ]
@@ -936,6 +973,8 @@ mod tests {
                 core_failure_of_the_checksum_batch(&ChecksumBatchFailed::GpuFailed {
                     device: card_named_in_the_failures(),
                     cause: cause.clone(),
+                    pci_state_when_judged_unusable:
+                        PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
                 }),
                 expected,
                 "甲段：{cause:?}"
@@ -944,6 +983,8 @@ mod tests {
                 core_failure_of_the_region_record_batch(&RegionRecordBatchFailed::GpuFailed {
                     device: card_named_in_the_failures(),
                     cause: cause.clone(),
+                    pci_state_when_judged_unusable:
+                        PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
                 }),
                 expected,
                 "乙段：{cause:?}"
@@ -952,6 +993,8 @@ mod tests {
                 core_failure_of_the_live_slot_batch(&LiveSlotBatchFailed::GpuFailed {
                     device: card_named_in_the_failures(),
                     cause: cause.clone(),
+                    pci_state_when_judged_unusable:
+                        PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
                 }),
                 expected,
                 "丙段：{cause:?}"
@@ -1052,11 +1095,15 @@ mod tests {
                 resubmissions: 3,
                 wait_limit,
                 last_submission_waited_for,
+                pci_state_when_judged_unusable:
+                    PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
             };
             assert_eq!(
                 core_failure_of_the_checksum_batch(&ChecksumBatchFailed::GpuFailed {
                     device: card_named_in_the_failures(),
                     cause: unusable.clone(),
+                    pci_state_when_judged_unusable:
+                        PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
                 }),
                 expected,
                 "甲段"
@@ -1065,6 +1112,8 @@ mod tests {
                 core_failure_of_the_region_record_batch(&RegionRecordBatchFailed::GpuFailed {
                     device: card_named_in_the_failures(),
                     cause: unusable.clone(),
+                    pci_state_when_judged_unusable:
+                        PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
                 }),
                 expected,
                 "乙段"
@@ -1073,6 +1122,8 @@ mod tests {
                 core_failure_of_the_live_slot_batch(&LiveSlotBatchFailed::GpuFailed {
                     device: card_named_in_the_failures(),
                     cause: unusable,
+                    pci_state_when_judged_unusable:
+                        PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
                 }),
                 expected,
                 "丙段"
@@ -1148,7 +1199,11 @@ mod tests {
                     cause: cause.clone(),
                 }
             ),
-            core_failure_of_the_gpu(&card_named_in_the_failures(), &cause),
+            core_failure_of_the_gpu(
+                &card_named_in_the_failures(),
+                &cause,
+                &PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError
+            ),
             "换设置时量代价出错照那张卡的 GPU 出错报"
         );
     }
@@ -1234,6 +1289,62 @@ mod tests {
         );
     }
 
+    /// 第 4 条「读数随那一次出错进核心层」：外壳判卡用不了那一刻读的一份，报给核心层时照带进那三种（卡当场报出错的 `GpuFailed`、重交用完的
+    /// `GpuUnusableAfterResubmissions`、切换开不成的 `GpuNotOpenedAfterTheReopenings`），重开到上限都没开成的每一次开不成报成的那一个带的是没读；这里不读。
+    #[test]
+    fn the_pci_state_read_by_the_shell_is_carried_into_the_core_failure_of_the_three_moments() {
+        let pci_state_read =
+            crate::pci_state_of_the_gpu::pci_state_read_under_the_root_not_present_for_tests(
+                "0000:01:00.0",
+            );
+        let wait_limit = Duration::from_secs(10);
+        let causes_judging_the_card_unusable = [
+            GpuFailureCause::UncapturedErrorOrDeviceLost {
+                reason: "假的：设备丢了".to_string(),
+            },
+            GpuFailureCause::UnusableAfterResubmissions {
+                resubmissions: 3,
+                last_cause: Box::new(GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit }),
+            },
+        ];
+        for cause in causes_judging_the_card_unusable {
+            assert_eq!(
+                core_failure_of_the_checksum_batch(&ChecksumBatchFailed::GpuFailed {
+                    device: card_named_in_the_failures(),
+                    cause: cause.clone(),
+                    pci_state_when_judged_unusable: pci_state_read.clone(),
+                })
+                .pci_state_when_judged_unusable(),
+                pci_state_read,
+                "{cause:?}"
+            );
+        }
+        let not_opened = core_failure_of_the_gpu_not_opened_after_the_reopenings(
+            &GpuNotOpenedAfterTheReopenings {
+                device: card_named_in_the_failures(),
+                causes_of_each_opening: vec![GpuFailureCause::UncapturedErrorOrDeviceLost {
+                    reason: "假的：开卡时设备丢了".to_string(),
+                }],
+                pci_state_when_judged_unusable: pci_state_read.clone(),
+            },
+        );
+        assert_eq!(
+            not_opened,
+            BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings {
+                device: card_named_in_the_failures(),
+                failures_of_each_opening: vec![BackgroundGpuComputeFailed::GpuFailed {
+                    device: card_named_in_the_failures(),
+                    cause: GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost {
+                        reason: "假的：开卡时设备丢了".to_string(),
+                    },
+                    pci_state_when_judged_unusable:
+                        PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+                }],
+                pci_state_when_judged_unusable: pci_state_read,
+            }
+        );
+    }
+
     /// 一批里同一个条目键出现两次：报给核心层的是不归哪张卡的那一种，两次出现的位置照带，不 panic。
     #[test]
     fn repeated_entry_key_is_reported_to_the_core_with_both_positions_and_no_card() {
diff --git a/crates/singlefs-background-gpu/src/gpus_of_this_host.rs b/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
index 4459521..8fbdcca 100644
--- a/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
+++ b/crates/singlefs-background-gpu/src/gpus_of_this_host.rs
@@ -44,6 +44,7 @@ use crate::memory_limit_and_chunks::{
     EntryCountOfAChunk, GpuMemoryAccount, InputLengthOnTheHostInBytes, PieceOfAnEntry,
     SmallestPieceLargerThanTheRoom,
 };
+use crate::pci_state_of_the_gpu::PciStateReaderOfTheGpus;
 use crate::region_record_kernels::bytes_of_the_smallest_region_record_piece;
 use crate::waiting_for_the_gpu::{
     gpu_opened_with_the_reopenings, CompletionCheck, GpuWaitDeadline, MonotonicClock,
@@ -878,7 +879,8 @@ pub fn gpu_opened_under_its_memory_limit<Opened>(
 /// 测试里交造出来的结局）；不参与的不开、不量代价，记进 [`DiscreteGpusAfterTheOpening::not_opened_for_not_taking_part`]，卡清单照列它，挂着切换点到它时再开。
 /// 显存上限装不下最小的一片的参与的卡照开、不量代价、不进参与的卡、报出来（[`GpuLeftOutUnderItsMemoryLimit`]），挂载不因此拒绝。
 /// 一张参与的卡开不成就照 `reopenings` 重开（第 4 条「挂载那一刻参与的卡重开 3 次都不成，才拒绝挂载」，开卡里 panic 了同样算一次没开成，每次重开之前把间隔交给
-/// `wait_before_a_reopening` 等，[`gpu_opened_with_the_reopenings`]）；重开到上限都没开成就拒绝挂载、报出它与每一次的原因，后面的卡不再开。
+/// `wait_before_a_reopening` 等，[`gpu_opened_with_the_reopenings`]）；重开到上限都没开成就拒绝挂载、报出它与每一次的原因，后面的卡不再开；
+/// 拒绝里带判它开不成那一刻照 `pci_state_reader` 读的它的 PCI 状态（D26（后台整理与放置回收） 已定项 13 第 4 条：读数写进挂载的报错；每一次没开成而还要重开时不读）。
 /// `participating` 是开卡那一组照同一份 `discrete_gpus` 校验过、取出来的参与的卡（`adapter_inventory::participating_gpus_admitted_by_the_mount_checks`）。
 ///
 /// # Errors
@@ -892,6 +894,7 @@ pub fn gpus_opened_at_the_mount<Opened>(
     smallest_piece_bytes: GpuMemoryInBytes,
     memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
     reopenings: ReopeningsOfAGpuNotOpened,
+    pci_state_reader: &PciStateReaderOfTheGpus,
     mut wait_before_a_reopening: impl FnMut(Duration),
     mut open_the_gpu: impl FnMut(
         &DiscreteGpuOnThisHost,
@@ -929,6 +932,8 @@ pub fn gpus_opened_at_the_mount<Opened>(
                 BackgroundGpuOpenRefused::GpuUnusable {
                     device: discrete_gpu.identifier.clone(),
                     causes_of_each_opening,
+                    pci_state_when_judged_unusable: pci_state_reader
+                        .pci_state_of_the_gpu(&discrete_gpu.identifier),
                 }
             })?;
         opened.push(opened_gpu);
@@ -1025,6 +1030,10 @@ mod tests {
     };
     use crate::background_gpu::COST_PROBE_CHUNK;
     use crate::memory_limit_and_chunks::SubmissionOfTheChunk;
+    use crate::pci_state_of_the_gpu::{
+        pci_state_read_under_the_root_not_present_for_tests,
+        pci_state_reader_under_a_root_not_present_for_tests,
+    };
 
     /// 用例里开卡开不成照产品那样至多重开 3 次，间隔为 0：不真等 1 分钟。
     const REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS: ReopeningsOfAGpuNotOpened =
@@ -1325,6 +1334,7 @@ mod tests {
             bytes_of_the_smallest_piece_every_participating_gpu_holds(),
             &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
             REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+            &pci_state_reader_under_a_root_not_present_for_tests(),
             |_interval| {},
             |discrete_gpu, memory_limit, _cost_at_the_opening| {
                 if discrete_gpu.pci_bus_address == card_that_does_not_open {
@@ -1356,6 +1366,7 @@ mod tests {
             bytes_of_the_smallest_piece_every_participating_gpu_holds(),
             &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
             REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+            &pci_state_reader_under_a_root_not_present_for_tests(),
             |_interval| {},
             |discrete_gpu, _memory_limit, _cost_at_the_opening| {
                 cards_handed_to_the_opening.push(discrete_gpu.pci_bus_address.clone());
@@ -1443,6 +1454,9 @@ mod tests {
             Err(BackgroundGpuOpenRefused::GpuUnusable {
                 device: card("0000:41:00.0"),
                 causes_of_each_opening: vec![reason_the_card_did_not_open("0000:41:00.0"); 4],
+                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
+                    "0000:41:00.0"
+                ),
             }),
             "点名的那张没开成：拒绝挂载（头一次与 3 次重开都开不成，每一次的原因都报）"
         );
@@ -1464,6 +1478,9 @@ mod tests {
             Err(BackgroundGpuOpenRefused::GpuUnusable {
                 device: card("0000:81:00.0"),
                 causes_of_each_opening: vec![reason_the_card_did_not_open("0000:81:00.0"); 4],
+                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
+                    "0000:81:00.0"
+                ),
             }),
             "没给卡列表：总线地址最后那张没开成也拒绝挂载（头一次与 3 次重开都开不成，每一次的原因都报）"
         );
@@ -1493,6 +1510,7 @@ mod tests {
                 interval: Duration::from_secs(9),
                 reopenings_at_most: 3,
             },
+            &pci_state_reader_under_a_root_not_present_for_tests(),
             |interval| intervals_waited.push(interval),
             |discrete_gpu, _memory_limit, _cost_at_the_opening| {
                 cards_handed_to_the_opening.push(discrete_gpu.pci_bus_address.clone());
@@ -1518,6 +1536,69 @@ mod tests {
         assert_eq!(opening.participating.positions_on_this_host(), &[0]);
     }
 
+    /// 点名「0000:41:00.0」一张的挂载那一刻开卡，开卡那一步头 `failed_openings` 次开不成、之后开成；读 PCI 状态照 `pci_state_reader`。
+    fn opening_of_the_card_at_forty_one_failing_at_first(
+        failed_openings: usize,
+        pci_state_reader: &PciStateReaderOfTheGpus,
+    ) -> Result<GpusOpenedAtTheMount<String>, BackgroundGpuOpenRefused> {
+        let selection = BackgroundGpuSelection {
+            background_gpu_devices: Some(vec![card("0000:41:00.0")]),
+            background_gpu_memory_limit: None,
+            background_gpu_wait_limit: Duration::from_secs(10),
+        };
+        let discrete_gpus = three_discrete_gpus_of_the_enumeration();
+        let participating =
+            participating_gpus_admitted_by_the_mount_checks(&selection, &discrete_gpus)
+                .expect("点名本机的一张卡：校验合法");
+        let mut openings_made: usize = 0;
+        gpus_opened_at_the_mount(
+            discrete_gpus,
+            &participating,
+            bytes_of_the_smallest_piece_every_participating_gpu_holds(),
+            &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
+            REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+            pci_state_reader,
+            |_interval| {},
+            |discrete_gpu, _memory_limit, _cost_at_the_opening| {
+                openings_made += 1;
+                if openings_made <= failed_openings {
+                    Err(reason_the_card_did_not_open(&discrete_gpu.pci_bus_address))
+                } else {
+                    Ok(discrete_gpu.pci_bus_address.clone())
+                }
+            },
+        )
+    }
+
+    /// 第 4 条「挂载那一刻开卡重开用完、拒绝挂载的，读数写进挂载的报错」：「0000:41:00.0」头一次与 3 次重开都开不成——拒绝挂载，
+    /// 拒绝里带判它开不成那一刻照它的地址读的 PCI 状态（用例的目录下没有它：目录不在、九份文件都不在），整个开卡只读了 1 次（每次没开成而还要重开时不读）。
+    #[test]
+    fn the_card_refusing_the_mount_carries_its_pci_state_read_once_its_reopenings_are_used_up() {
+        let pci_state_reader = pci_state_reader_under_a_root_not_present_for_tests();
+        let refused = opening_of_the_card_at_forty_one_failing_at_first(4, &pci_state_reader);
+        assert_eq!(
+            refused.map(|opening| opening.opened),
+            Err(BackgroundGpuOpenRefused::GpuUnusable {
+                device: card("0000:41:00.0"),
+                causes_of_each_opening: vec![reason_the_card_did_not_open("0000:41:00.0"); 4],
+                pci_state_when_judged_unusable: pci_state_read_under_the_root_not_present_for_tests(
+                    "0000:41:00.0"
+                ),
+            })
+        );
+        assert_eq!(pci_state_reader.readings_taken(), 1);
+    }
+
+    /// 同一条「平常的批、每次超时重交之前、开卡每一次没开成而还要重开时都不查」：「0000:41:00.0」头三次开不成、第四次开成——挂载照常，一次都不读。
+    #[test]
+    fn the_card_opening_at_its_last_reopening_at_the_mount_is_not_read() {
+        let pci_state_reader = pci_state_reader_under_a_root_not_present_for_tests();
+        let opening = opening_of_the_card_at_forty_one_failing_at_first(3, &pci_state_reader)
+            .expect("第四次开成：挂载照常");
+        assert_eq!(opening.opened, vec!["0000:41:00.0".to_string()]);
+        assert_eq!(pci_state_reader.readings_taken(), 0);
+    }
+
     /// 开卡那一组点「0000:01:00.0」「0000:41:00.0」两张、各给 `memory_limits_in_bytes` 的显存上限；开卡那一步照外壳那样：量代价的卡拿给它的上限建账、
     /// 把量代价那一段照这张卡的界切成几片、逐段记账（`GpuMemoryAccount::hold`，记下之后超上限就断言）再放掉，交回（总线地址，量没量代价）。
     fn opening_that_holds_the_cost_probe_in_pieces(
@@ -1543,6 +1624,7 @@ mod tests {
             bytes_of_the_smallest_piece_every_participating_gpu_holds(),
             &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
             REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+            &pci_state_reader_under_a_root_not_present_for_tests(),
             |_interval| {},
             |discrete_gpu, memory_limit, cost_at_the_opening| {
                 if cost_at_the_opening == CostAtTheOpening::MeasuredUnderTheMemoryLimit {
@@ -2058,6 +2140,7 @@ mod tests {
             bytes_of_the_smallest_piece_every_participating_gpu_holds(),
             &memory_leaked,
             REOPENINGS_WITHOUT_WAITING_IN_THE_TESTS,
+            &pci_state_reader_under_a_root_not_present_for_tests(),
             |_interval| {},
             |discrete_gpu, _memory_limit, cost_at_the_opening| {
                 Ok((discrete_gpu.pci_bus_address.clone(), cost_at_the_opening))
diff --git a/crates/singlefs-background-gpu/src/lib.rs b/crates/singlefs-background-gpu/src/lib.rs
index 8621093..3c7dccc 100644
--- a/crates/singlefs-background-gpu/src/lib.rs
+++ b/crates/singlefs-background-gpu/src/lib.rs
@@ -1,6 +1,6 @@
 //! 产品的后台 GPU 路径（D26（后台整理与放置回收） 已定项 13 第 1–4、8 条；等 GPU 结果设超时，超时算出错）。核心层不依赖本包：挂载时由调用方把本包的实现递进去。
 //!
-//! 分三层：
+//! 分四块：
 //! - 纯逻辑，不要 GPU：哪些适配器算显卡与挂载那一刻的校验（[`adapter_inventory`]）、挂载时每张卡量出的一次提交的主机 CPU 时间（只量、只报，不用来分活，
 //!   [`measured_host_cpu_time_per_submission`]）、
 //!   显存上限与切批（[`memory_limit_and_chunks`]）、两份核的比对与连续不确定计数（[`two_kernel_reconciliation`]，与哪一段无关）、
@@ -8,6 +8,7 @@
 //!   （[`region_record_kernels`]）、等 GPU 时的上限与可换的钟（[`waiting_for_the_gpu`]）、
 //!   选段打分里每段活槽数那一段的打包、回读解码与 CPU 参照（[`live_slot_kernels`]）、挂载时开着的本机全部独显里此刻照哪组设置用哪几张、
 //!   每张怎么切、等多久、一批按卡的顺序填满（前面的卡填到它照显存上限一次放得下的量，装不下的才给后面的卡；[`gpus_of_this_host`]）。
+//! - 判一张卡用不了那一刻照它的 PCI 地址读本机的 PCI 状态（目录在不在、使能位、链路、出错计数，[`pci_state_of_the_gpu`]）：只读文件，不要 GPU。
 //! - 起 GPU 的外壳（[`background_gpu`]）：枚举 wgpu 适配器、在参与的卡上起设备、量一次提交的主机 CPU 时间、交一批校验和核对、一批区域扫描的记录或一批聚簇段。
 //! - 核心层要的那一份实现（[`background_gpu_compute`]）：`singlefs_core::background_compute::BackgroundGpuCompute` 三段各一个方法接到上面三段的两份核，
 //!   连续不确定的计数随它活；挂载那一刻只开参与的卡、交回核心层收的把手（[`background_gpu_compute::open_background_gpu_compute`]）。
@@ -30,6 +31,7 @@ pub mod gpus_of_this_host;
 pub mod live_slot_kernels;
 pub mod measured_host_cpu_time_per_submission;
 pub mod memory_limit_and_chunks;
+pub mod pci_state_of_the_gpu;
 pub mod region_record_kernels;
 pub mod two_kernel_reconciliation;
 pub mod waiting_for_the_gpu;
diff --git a/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs b/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs
new file mode 100644
index 0000000..63b6dac
--- /dev/null
+++ b/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs
@@ -0,0 +1,539 @@
+//! 判一张卡用不了、要报警的那一刻照它的 PCI 地址读本机的 PCI 状态（D26（后台整理与放置回收） 已定项 13 第 4 条「判一张卡用不了、要报警的那一刻，查一次这张卡的硬件与链路」）：
+//! PCI 设备目录（本机是 `/sys/bus/pci/devices`，调用方可以换成别的目录）下以卡的地址为名的那个目录在不在、使能位（`enable`）、
+//! 链路此刻与最大的速度和宽度（`current_link_speed`、`current_link_width`、`max_link_speed`、`max_link_width`）、
+//! 高级错误报告的三类出错计数（`aer_dev_correctable`、`aer_dev_fatal`、`aer_dev_nonfatal`）。每一样各自是读到、文件不在、读不通之一，读不到的不当成 0、不当成正常。
+//! 读数的类型住核心层（`singlefs_core::background_compute::PciStateOfTheGpuReadWhenJudgedUnusable`），核心层只带着它进停下记录与报警；读只在这里做。
+//! 不读内核日志、不起任何外部程序，只读普通用户读得到的那几份文件。
+//!
+//! 读的时刻只有三种（第 4 条）：挂着时一批在同一张卡上等超时用完、判它用不了（`GpuFailureCause::UnusableAfterResubmissions`）；开卡重开用完都开不成
+//! （挂着切换的与挂载那一刻的都算）；卡当场报出错（设备丢了、wgpu 报了没接住的错，`GpuFailureCause::UncapturedErrorOrDeviceLost`）使后台任务停下。
+//! 平常的批、每次超时重交之前、开卡每一次没开成而还要重开时都不读：读的那几处在 `crate::background_gpu` 与 `crate::gpus_of_this_host`，
+//! 判一个出错是不是读的时刻在 [`whether_the_cause_judges_the_gpu_unusable_in_a_batch`]。
+
+use std::io::ErrorKind;
+use std::path::{Path, PathBuf};
+use std::sync::atomic::{AtomicU64, Ordering};
+use std::sync::Arc;
+
+use singlefs_core::background_compute::{
+    GpuDeviceIdentifier, PciDeviceDirectoryReading, PciDeviceEnable, PciFileReading,
+    PciStateOfTheGpu, PciStateOfTheGpuReadWhenJudgedUnusable,
+};
+
+use crate::background_gpu::GpuFailureCause;
+
+/// 本机的 PCI 设备目录：每个 PCI 设备在它下面有一个以地址为名的目录。
+pub const PCI_DEVICE_DIRECTORIES_ROOT_OF_THIS_HOST: &str = "/sys/bus/pci/devices";
+
+/// 照卡的 PCI 地址读 PCI 状态的那一份：在哪个目录下读（产品是 [`PCI_DEVICE_DIRECTORIES_ROOT_OF_THIS_HOST`]，测试换成自己造的目录），
+/// 与读了几次（每查一张卡算一次，地址当不了目录名、没真读的那一次也算；克隆出来的几份共用这一个数）。
+#[derive(Clone, Debug)]
+pub struct PciStateReaderOfTheGpus {
+    pci_device_directories_root: PathBuf,
+    readings_taken: Arc<AtomicU64>,
+}
+
+impl PciStateReaderOfTheGpus {
+    /// 产品那一份：读本机的 PCI 设备目录。
+    #[must_use]
+    pub fn of_this_host() -> Self {
+        Self::under_the_pci_device_directories_root(PathBuf::from(
+            PCI_DEVICE_DIRECTORIES_ROOT_OF_THIS_HOST,
+        ))
+    }
+
+    /// 在 `pci_device_directories_root` 下读（每张卡读它下面以地址为名的那个目录）：测试拿它读自己造的目录，不读本机。
+    #[must_use]
+    pub fn under_the_pci_device_directories_root(pci_device_directories_root: PathBuf) -> Self {
+        Self {
+            pci_device_directories_root,
+            readings_taken: Arc::new(AtomicU64::new(0)),
+        }
+    }
+
+    /// 到此刻为止查了几次卡（这一份与它的克隆合起来）。
+    #[must_use]
+    pub fn readings_taken(&self) -> u64 {
+        self.readings_taken.load(Ordering::SeqCst)
+    }
+
+    /// 查一次 `device` 这张卡：它的标识当得了一个目录名就读那个目录与九份文件（[`pci_state_under_the_device_directory`]），
+    /// 当不了就不读、报 [`PciStateOfTheGpuReadWhenJudgedUnusable::PciBusAddressNotADirectoryName`]。查的次数加一。
+    #[must_use]
+    pub fn pci_state_of_the_gpu(
+        &self,
+        device: &GpuDeviceIdentifier,
+    ) -> PciStateOfTheGpuReadWhenJudgedUnusable {
+        self.readings_taken.fetch_add(1, Ordering::SeqCst);
+        match pci_bus_address_usable_as_a_directory_name(device.opaque_bytes()) {
+            Some(pci_bus_address) => {
+                PciStateOfTheGpuReadWhenJudgedUnusable::Read(pci_state_under_the_device_directory(
+                    pci_bus_address,
+                    self.pci_device_directories_root.join(pci_bus_address),
+                ))
+            }
+            None => PciStateOfTheGpuReadWhenJudgedUnusable::PciBusAddressNotADirectoryName {
+                device: device.clone(),
+            },
+        }
+    }
+
+    /// 挂着时一批在 `device` 这张卡上以 `cause` 出错：是判它用不了的时刻（[`whether_the_cause_judges_the_gpu_unusable_in_a_batch`]）就查一次，
+    /// 不是就不查、交回没读。
+    #[must_use]
+    pub fn pci_state_if_the_cause_judges_the_gpu_unusable(
+        &self,
+        device: &GpuDeviceIdentifier,
+        cause: &GpuFailureCause,
+    ) -> PciStateOfTheGpuReadWhenJudgedUnusable {
+        if whether_the_cause_judges_the_gpu_unusable_in_a_batch(cause) {
+            self.pci_state_of_the_gpu(device)
+        } else {
+            PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError
+        }
+    }
+}
+
+/// 挂着时一批在一张卡上的出错是不是判那张卡用不了、要读 PCI 状态的时刻：等超时用完（[`GpuFailureCause::UnusableAfterResubmissions`]）
+/// 与卡当场报出错（[`GpuFailureCause::UncapturedErrorOrDeviceLost`]）是；一次超时（还要重交）、等提交没等成、回读坏了、核留下认不出的判定字、
+/// 建缓冲显存不足与开卡那几种不是。重开到上限都没开成（[`GpuFailureCause::NotOpenedAfterTheReopenings`]）也是判用不了的时刻，可它的读数在判它开不成
+/// 那一刻就读了、随开不成的那一份带着，这里不再读。
+#[must_use]
+pub fn whether_the_cause_judges_the_gpu_unusable_in_a_batch(cause: &GpuFailureCause) -> bool {
+    match cause {
+        GpuFailureCause::UnusableAfterResubmissions { .. }
+        | GpuFailureCause::UncapturedErrorOrDeviceLost { .. } => true,
+        GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
+        | GpuFailureCause::OpeningThreadNotStarted { .. }
+        | GpuFailureCause::OpeningPanicked { .. }
+        | GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned
+        | GpuFailureCause::OpeningCalledOff
+        | GpuFailureCause::OpeningsInThisMountUsedUp { .. }
+        | GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { .. }
+        | GpuFailureCause::WaitForTheSubmissionFailed { .. }
+        | GpuFailureCause::WaitForTheGpuExceededTheLimit { .. }
+        | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
+        | GpuFailureCause::ReadbackMappingFailed { .. }
+        | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
+        | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
+        | GpuFailureCause::NotOpenedAfterTheReopenings { .. } => false,
+    }
+}
+
+/// 卡的标识当得了 PCI 设备目录下的一个目录名时交回那个名字：是 UTF-8、不空、不是 `.` 或 `..`、不带 `/` 与 NUL 字节；
+/// 当不了交回 `None`（拼进路径会读到那个目录之外去，或拼不成路径）。
+#[must_use]
+pub fn pci_bus_address_usable_as_a_directory_name(opaque_bytes: &[u8]) -> Option<&str> {
+    let pci_bus_address = std::str::from_utf8(opaque_bytes).ok()?;
+    let is_one_directory_name = !pci_bus_address.is_empty()
+        && pci_bus_address != "."
+        && pci_bus_address != ".."
+        && !pci_bus_address.contains('/')
+        && !pci_bus_address.contains('\0');
+    is_one_directory_name.then_some(pci_bus_address)
+}
+
+/// 读 `device_directory`（地址 `pci_bus_address` 的那个目录）：目录在不在，与九份文件各自的读数。目录不在时那九份照样各读一次（都报文件不在）。
+#[must_use]
+pub fn pci_state_under_the_device_directory(
+    pci_bus_address: &str,
+    device_directory: PathBuf,
+) -> PciStateOfTheGpu {
+    let text_of = |file_name: &str| {
+        pci_file_reading_with_the_text_mapped(
+            text_of_the_pci_file(&device_directory.join(file_name)),
+            text_without_the_line_end,
+        )
+    };
+    PciStateOfTheGpu {
+        pci_bus_address: pci_bus_address.to_string(),
+        device_directory_reading: reading_of_the_device_directory(&device_directory),
+        device_enable: pci_file_reading_with_the_text_mapped(text_of("enable"), |text| {
+            pci_device_enable_of_the_text(&text)
+        }),
+        current_link_speed: text_of("current_link_speed"),
+        current_link_width: text_of("current_link_width"),
+        maximum_link_speed: text_of("max_link_speed"),
+        maximum_link_width: text_of("max_link_width"),
+        advanced_error_reporting_correctable: text_of("aer_dev_correctable"),
+        advanced_error_reporting_fatal: text_of("aer_dev_fatal"),
+        advanced_error_reporting_nonfatal: text_of("aer_dev_nonfatal"),
+        device_directory,
+    }
+}
+
+/// 一个 PCI 设备目录在不在：打得开是在，系统报找不到是不在，别的错是读不通（带系统报的原话）。
+fn reading_of_the_device_directory(device_directory: &Path) -> PciDeviceDirectoryReading {
+    match std::fs::read_dir(device_directory) {
+        Ok(_entries) => PciDeviceDirectoryReading::Present,
+        Err(error) if error.kind() == ErrorKind::NotFound => PciDeviceDirectoryReading::NotPresent,
+        Err(error) => PciDeviceDirectoryReading::NotReadable {
+            system_error: error.to_string(),
+        },
+    }
+}
+
+/// 读一份 PCI 属性文件的全文：系统报找不到是文件不在，别的错是读不通（带系统报的原话）。内容按 UTF-8 解，解不了的字节换成替换字符（这几份文件都是 ASCII）。
+fn text_of_the_pci_file(path_of_the_file: &Path) -> PciFileReading<String> {
+    match std::fs::read(path_of_the_file) {
+        Ok(bytes) => PciFileReading::Read(String::from_utf8_lossy(&bytes).into_owned()),
+        Err(error) if error.kind() == ErrorKind::NotFound => PciFileReading::FileNotPresent,
+        Err(error) => PciFileReading::NotReadable {
+            system_error: error.to_string(),
+        },
+    }
+}
+
+/// 读到的那一份原文换成 `content_of_the_text` 交回的；文件不在、读不通照原样。
+fn pci_file_reading_with_the_text_mapped<Content>(
+    reading: PciFileReading<String>,
+    content_of_the_text: impl FnOnce(String) -> Content,
+) -> PciFileReading<Content> {
+    match reading {
+        PciFileReading::Read(text) => PciFileReading::Read(content_of_the_text(text)),
+        PciFileReading::FileNotPresent => PciFileReading::FileNotPresent,
+        PciFileReading::NotReadable { system_error } => {
+            PciFileReading::NotReadable { system_error }
+        }
+    }
+}
+
+/// 去掉文件原文末尾的一个换行（内核写这几份文件时末尾带一个），别的照原样：多行的出错计数中间的换行留着。
+fn text_without_the_line_end(mut text: String) -> String {
+    if text.ends_with('\n') {
+        text.pop();
+    }
+    text
+}
+
+/// `enable` 的原文换成使能位：去掉首尾空白是一个十进制数时，0 是关的、别的是开着（带那个数）；不是十进制数的照原文报认不得。
+#[must_use]
+pub fn pci_device_enable_of_the_text(text: &str) -> PciDeviceEnable {
+    let trimmed_text = text.trim();
+    match trimmed_text.parse::<u64>() {
+        Ok(0) => PciDeviceEnable::Disabled,
+        Ok(enable_count) => PciDeviceEnable::Enabled { enable_count },
+        Err(_not_a_decimal_number) => PciDeviceEnable::UnrecognizedText {
+            text: trimmed_text.to_string(),
+        },
+    }
+}
+
+/// 测试用：一个不在的 PCI 设备目录（根目录下、名字带本包：建它要超级用户，测试与产品都不建），照它读的每张卡都是目录不在、九份文件都不在，
+/// 不随哪台机器、哪个临时目录变。用例照它造读数的期望。
+#[cfg(test)]
+pub(crate) fn pci_device_directories_root_not_present_for_tests() -> PathBuf {
+    PathBuf::from("/singlefs-background-gpu-tests-pci-device-directories-never-created")
+}
+
+/// 测试用：在 [`pci_device_directories_root_not_present_for_tests`] 下读的那一份。
+#[cfg(test)]
+pub(crate) fn pci_state_reader_under_a_root_not_present_for_tests() -> PciStateReaderOfTheGpus {
+    PciStateReaderOfTheGpus::under_the_pci_device_directories_root(
+        pci_device_directories_root_not_present_for_tests(),
+    )
+}
+
+/// 测试用：照 [`pci_state_reader_under_a_root_not_present_for_tests`] 读 `pci_bus_address` 那张卡该得的读数：目录不在、九份文件都不在。
+#[cfg(test)]
+pub(crate) fn pci_state_read_under_the_root_not_present_for_tests(
+    pci_bus_address: &str,
+) -> PciStateOfTheGpuReadWhenJudgedUnusable {
+    PciStateOfTheGpuReadWhenJudgedUnusable::Read(PciStateOfTheGpu {
+        pci_bus_address: pci_bus_address.to_string(),
+        device_directory: pci_device_directories_root_not_present_for_tests().join(pci_bus_address),
+        device_directory_reading: PciDeviceDirectoryReading::NotPresent,
+        device_enable: PciFileReading::FileNotPresent,
+        current_link_speed: PciFileReading::FileNotPresent,
+        current_link_width: PciFileReading::FileNotPresent,
+        maximum_link_speed: PciFileReading::FileNotPresent,
+        maximum_link_width: PciFileReading::FileNotPresent,
+        advanced_error_reporting_correctable: PciFileReading::FileNotPresent,
+        advanced_error_reporting_fatal: PciFileReading::FileNotPresent,
+        advanced_error_reporting_nonfatal: PciFileReading::FileNotPresent,
+    })
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    use std::sync::atomic::AtomicU32;
+
+    use singlefs_core::background_compute::{
+        hardware_judgement_of_the_pci_state, HardwareJudgementOfTheGpu, ReasonTheGpuIsOffTheBus,
+    };
+
+    /// 用例自己造的 PCI 设备目录：建在临时目录下（名字带进程号、用例名与本进程里的序号，几条用例并行互不相撞），丢掉时整个删掉（成功、失败、panic 都删）。
+    struct PciDeviceDirectoriesMadeByTheTest {
+        root: PathBuf,
+    }
+
+    impl PciDeviceDirectoriesMadeByTheTest {
+        fn named_after_the_test(test_name: &str) -> Self {
+            static DIRECTORIES_MADE_IN_THIS_PROCESS: AtomicU32 = AtomicU32::new(0);
+            let directory_number = DIRECTORIES_MADE_IN_THIS_PROCESS.fetch_add(1, Ordering::SeqCst);
+            let root = std::env::temp_dir().join(format!(
+                "singlefs-pci-device-directories-{}-{test_name}-{directory_number}",
+                std::process::id()
+            ));
+            std::fs::create_dir_all(&root).expect("临时目录下建得了用例自己的目录");
+            Self { root }
+        }
+
+        /// 在地址 `pci_bus_address` 的目录下写一份 `file_name`，内容 `text`（目录没有就建）。
+        fn with_the_file(self, pci_bus_address: &str, file_name: &str, text: &str) -> Self {
+            let device_directory = self.root.join(pci_bus_address);
+            std::fs::create_dir_all(&device_directory).expect("临时目录下建得了设备目录");
+            std::fs::write(device_directory.join(file_name), text).expect("临时目录下写得了文件");
+            self
+        }
+
+        /// 在地址 `pci_bus_address` 的目录下把 `file_name` 建成一个目录：读它报读不通（是目录、不是文件）。
+        fn with_a_directory_in_place_of_the_file(
+            self,
+            pci_bus_address: &str,
+            file_name: &str,
+        ) -> Self {
+            std::fs::create_dir_all(self.root.join(pci_bus_address).join(file_name))
+                .expect("临时目录下建得了目录");
+            self
+        }
+
+        fn reader(&self) -> PciStateReaderOfTheGpus {
+            PciStateReaderOfTheGpus::under_the_pci_device_directories_root(self.root.clone())
+        }
+    }
+
+    impl Drop for PciDeviceDirectoriesMadeByTheTest {
+        fn drop(&mut self) {
+            // 删不掉不让用例因此 panic（丢的时候可能正在 panic）；留下的只在临时目录里。
+            let _left_in_the_temporary_directory = std::fs::remove_dir_all(&self.root);
+        }
+    }
+
+    fn card(pci_bus_address: &str) -> GpuDeviceIdentifier {
+        GpuDeviceIdentifier::of_opaque_bytes(pci_bus_address.as_bytes().to_vec())
+    }
+
+    const CORRECTABLE_ERRORS_OF_THE_TEST: &str = "RxErr 0\nBadTLP 2\nTOTAL_ERR_COR 2\n";
+
+    /// 一张卡的目录与九份文件全在：读到的每一样是原文去掉末尾换行，使能位解成开着，链路与出错计数照原文带出（多行的中间换行留着），
+    /// 读数里带着照哪个地址、哪个目录读的；判不出硬件坏。读的次数是 1。
+    #[test]
+    fn every_file_present_is_read_as_its_text_with_the_address_and_the_directory_read() {
+        let directories = PciDeviceDirectoriesMadeByTheTest::named_after_the_test("every-file")
+            .with_the_file("0000:01:00.0", "enable", "1\n")
+            .with_the_file("0000:01:00.0", "current_link_speed", "5.0 GT/s PCIe\n")
+            .with_the_file("0000:01:00.0", "current_link_width", "4\n")
+            .with_the_file("0000:01:00.0", "max_link_speed", "32.0 GT/s PCIe\n")
+            .with_the_file("0000:01:00.0", "max_link_width", "16\n")
+            .with_the_file(
+                "0000:01:00.0",
+                "aer_dev_correctable",
+                CORRECTABLE_ERRORS_OF_THE_TEST,
+            )
+            .with_the_file("0000:01:00.0", "aer_dev_fatal", "TOTAL_ERR_FATAL 0\n")
+            .with_the_file("0000:01:00.0", "aer_dev_nonfatal", "TOTAL_ERR_NONFATAL 1\n");
+        let reader = directories.reader();
+        let reading = reader.pci_state_of_the_gpu(&card("0000:01:00.0"));
+        assert_eq!(
+            reading,
+            PciStateOfTheGpuReadWhenJudgedUnusable::Read(PciStateOfTheGpu {
+                pci_bus_address: "0000:01:00.0".to_string(),
+                device_directory: directories.root.join("0000:01:00.0"),
+                device_directory_reading: PciDeviceDirectoryReading::Present,
+                device_enable: PciFileReading::Read(PciDeviceEnable::Enabled { enable_count: 1 }),
+                current_link_speed: PciFileReading::Read("5.0 GT/s PCIe".to_string()),
+                current_link_width: PciFileReading::Read("4".to_string()),
+                maximum_link_speed: PciFileReading::Read("32.0 GT/s PCIe".to_string()),
+                maximum_link_width: PciFileReading::Read("16".to_string()),
+                advanced_error_reporting_correctable: PciFileReading::Read(
+                    "RxErr 0\nBadTLP 2\nTOTAL_ERR_COR 2".to_string()
+                ),
+                advanced_error_reporting_fatal: PciFileReading::Read(
+                    "TOTAL_ERR_FATAL 0".to_string()
+                ),
+                advanced_error_reporting_nonfatal: PciFileReading::Read(
+                    "TOTAL_ERR_NONFATAL 1".to_string()
+                ),
+            })
+        );
+        assert_eq!(
+            reading.hardware_judgement(),
+            Some(HardwareJudgementOfTheGpu::NotJudgedOffTheBus)
+        );
+        assert_eq!(reader.readings_taken(), 1);
+    }
+
+    /// 地址的目录不在：目录报不在、九份文件都报文件不在（不当成 0），判卡不在总线上。
+    #[test]
+    fn the_device_directory_not_present_is_read_as_off_the_bus_with_every_file_not_present() {
+        let directories = PciDeviceDirectoriesMadeByTheTest::named_after_the_test("no-directory")
+            .with_the_file("0000:10:00.0", "enable", "1\n");
+        let reading = directories
+            .reader()
+            .pci_state_of_the_gpu(&card("0000:01:00.0"));
+        let PciStateOfTheGpuReadWhenJudgedUnusable::Read(pci_state) = &reading else {
+            panic!("地址当得了目录名：照它读了，读数是 {reading:?}");
+        };
+        assert_eq!(
+            pci_state.device_directory_reading,
+            PciDeviceDirectoryReading::NotPresent
+        );
+        let file_not_present = PciFileReading::<String>::FileNotPresent;
+        assert_eq!(
+            [
+                &pci_state.current_link_speed,
+                &pci_state.current_link_width,
+                &pci_state.maximum_link_speed,
+                &pci_state.maximum_link_width,
+                &pci_state.advanced_error_reporting_correctable,
+                &pci_state.advanced_error_reporting_fatal,
+                &pci_state.advanced_error_reporting_nonfatal,
+            ],
+            [&file_not_present; 7]
+        );
+        assert_eq!(pci_state.device_enable, PciFileReading::FileNotPresent);
+        assert_eq!(
+            hardware_judgement_of_the_pci_state(pci_state),
+            HardwareJudgementOfTheGpu::OffTheBus(
+                ReasonTheGpuIsOffTheBus::DeviceDirectoryNotPresent
+            )
+        );
+    }
+
+    /// 目录在、使能位读到 0：解成关的，判卡不在总线上。没有的三份出错计数文件报文件不在。
+    #[test]
+    fn the_device_enable_reading_zero_is_read_as_disabled_and_off_the_bus() {
+        let directories = PciDeviceDirectoriesMadeByTheTest::named_after_the_test("disabled")
+            .with_the_file("0000:01:00.0", "enable", "0\n");
+        let reading = directories
+            .reader()
+            .pci_state_of_the_gpu(&card("0000:01:00.0"));
+        let PciStateOfTheGpuReadWhenJudgedUnusable::Read(pci_state) = &reading else {
+            panic!("地址当得了目录名：照它读了，读数是 {reading:?}");
+        };
+        assert_eq!(
+            pci_state.device_enable,
+            PciFileReading::Read(PciDeviceEnable::Disabled)
+        );
+        assert_eq!(
+            pci_state.advanced_error_reporting_fatal,
+            PciFileReading::FileNotPresent
+        );
+        assert_eq!(
+            hardware_judgement_of_the_pci_state(pci_state),
+            HardwareJudgementOfTheGpu::OffTheBus(ReasonTheGpuIsOffTheBus::DeviceDisabled)
+        );
+    }
+
+    /// 使能位那份读不通（那里是个目录）：报读不通、带系统报的原话，不当成关的、不判坏。
+    #[test]
+    fn the_device_enable_not_readable_is_reported_with_the_system_error_and_not_judged_off_the_bus()
+    {
+        let directories =
+            PciDeviceDirectoriesMadeByTheTest::named_after_the_test("enable-unreadable")
+                .with_the_file("0000:01:00.0", "current_link_width", "16\n")
+                .with_a_directory_in_place_of_the_file("0000:01:00.0", "enable");
+        let reading = directories
+            .reader()
+            .pci_state_of_the_gpu(&card("0000:01:00.0"));
+        let PciStateOfTheGpuReadWhenJudgedUnusable::Read(pci_state) = &reading else {
+            panic!("地址当得了目录名：照它读了，读数是 {reading:?}");
+        };
+        assert!(
+            matches!(
+                &pci_state.device_enable,
+                PciFileReading::NotReadable { system_error } if !system_error.is_empty()
+            ),
+            "使能位那份是目录：读不通，读数是 {:?}",
+            pci_state.device_enable
+        );
+        assert_eq!(
+            pci_state.current_link_width,
+            PciFileReading::Read("16".to_string())
+        );
+        assert_eq!(
+            hardware_judgement_of_the_pci_state(pci_state),
+            HardwareJudgementOfTheGpu::NotJudgedOffTheBus
+        );
+    }
+
+    /// 卡的标识当不了一个目录名（带 `/`、是 `..`、不是 UTF-8）：不读那个目录之外的东西，报地址当不了目录名、带着那张卡；读的次数照样每张加一。
+    #[test]
+    fn identifiers_not_usable_as_a_directory_name_are_not_read_outside_the_root() {
+        let directories = PciDeviceDirectoriesMadeByTheTest::named_after_the_test("bad-address");
+        let reader = directories.reader();
+        let identifiers = [
+            card("../0000:01:00.0"),
+            card(".."),
+            GpuDeviceIdentifier::of_opaque_bytes(vec![0xff, 0x30]),
+        ];
+        let readings: Vec<PciStateOfTheGpuReadWhenJudgedUnusable> = identifiers
+            .iter()
+            .map(|device| reader.pci_state_of_the_gpu(device))
+            .collect();
+        assert_eq!(
+            readings,
+            identifiers
+                .iter()
+                .map(|device| {
+                    PciStateOfTheGpuReadWhenJudgedUnusable::PciBusAddressNotADirectoryName {
+                        device: device.clone(),
+                    }
+                })
+                .collect::<Vec<_>>()
+        );
+        assert_eq!(reader.readings_taken(), 3);
+    }
+
+    /// `enable` 的原文：去掉首尾空白之后的十进制数，0 是关的、别的带计数开着；不是十进制数的照去掉空白的原文报认不得。
+    #[test]
+    fn device_enable_texts_are_read_as_disabled_enabled_with_the_count_or_unrecognized() {
+        assert_eq!(
+            [
+                pci_device_enable_of_the_text("0\n"),
+                pci_device_enable_of_the_text("2\n"),
+                pci_device_enable_of_the_text(" on\n"),
+            ],
+            [
+                PciDeviceEnable::Disabled,
+                PciDeviceEnable::Enabled { enable_count: 2 },
+                PciDeviceEnable::UnrecognizedText {
+                    text: "on".to_string()
+                },
+            ]
+        );
+    }
+
+    /// 一批里哪些出错是读的时刻：等超时用完与卡当场报出错是；一次超时（还要重交）、等提交没等成、回读坏了不是。
+    #[test]
+    fn only_the_resubmissions_used_up_and_the_device_lost_judge_the_gpu_unusable_in_a_batch() {
+        let wait_limit = std::time::Duration::from_secs(10);
+        assert_eq!(
+            [
+                GpuFailureCause::UnusableAfterResubmissions {
+                    resubmissions: 3,
+                    last_cause: Box::new(GpuFailureCause::WaitForTheGpuExceededTheLimit {
+                        wait_limit
+                    }),
+                },
+                GpuFailureCause::UncapturedErrorOrDeviceLost {
+                    reason: "替身：设备丢了".to_string(),
+                },
+                GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit },
+                GpuFailureCause::WaitForTheSubmissionFailed {
+                    reason: "替身：没等成".to_string(),
+                },
+                GpuFailureCause::ReadbackMappingFailed {
+                    reason: "替身：映射不成".to_string(),
+                },
+            ]
+            .iter()
+            .map(whether_the_cause_judges_the_gpu_unusable_in_a_batch)
+            .collect::<Vec<bool>>(),
+            vec![true, true, false, false, false]
+        );
+    }
+}
diff --git a/crates/singlefs-core/src/background_compute.rs b/crates/singlefs-core/src/background_compute.rs
index 15b4a42..3e7089f 100644
--- a/crates/singlefs-core/src/background_compute.rs
+++ b/crates/singlefs-core/src/background_compute.rs
@@ -19,6 +19,7 @@
 //! 切到 `cpu` 时告诉实现一张卡都不参与（[`BackgroundGpuCompute::use_no_gpu_from_the_next_batch`]），开着的卡全部关掉。
 
 use std::fmt;
+use std::path::PathBuf;
 use std::sync::{Arc, Mutex, MutexGuard};
 use std::time::Duration;
 
@@ -67,6 +68,12 @@ impl GpuDeviceIdentifier {
     pub fn of_opaque_bytes(opaque_bytes: Vec<u8>) -> Self {
         Self(opaque_bytes)
     }
+
+    /// 标识的字节原样（产品 GPU 包照它当 PCI 地址读那张卡的 PCI 状态）。
+    #[must_use]
+    pub fn opaque_bytes(&self) -> &[u8] {
+        &self.0
+    }
 }
 
 /// 显存的字节数：一张卡的显存总量，或 `background_gpu_memory_limit` 里给一张卡的上限（第 3 条）。
@@ -217,9 +224,12 @@ impl<Outcome> GpuEntryDecision<Outcome> {
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum BackgroundGpuComputeFailed {
     /// 这张卡出错（起不了、设备丢了、等不成、回读坏了、核留下认不出的判定字）：修好它或切换时不再点它，或切回 `cpu`。
+    /// `pci_state_when_judged_unusable` 是挂着时一批里卡当场报出错（[`GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost`]）那一刻读的 PCI 状态；
+    /// 别的原因、与开卡每一次没开成的原因（[`Self::GpuNotOpenedAfterTheReopenings`] 的 `failures_of_each_opening`）都是没读。
     GpuFailed {
         device: GpuDeviceIdentifier,
         cause: GpuFailureOnTheDevice,
+        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
     },
     /// 等这张卡超过了 `background_gpu_wait_limit`：调大上限，或查这张卡是不是卡死了。
     WaitForTheGpuExceededTheLimit {
@@ -234,18 +244,22 @@ pub enum BackgroundGpuComputeFailed {
     /// 或这张卡在这一次挂载里已连着开了 4 次都没开成（开成一次清零）、这一次不再开（[`GpuFailureOnTheDevice::OpeningsInThisMountUsedUp`]）：修好它或切换时不再点它，或切回 `cpu`；
     /// 连着试满的那一种要卸载重挂。`failures_of_each_opening` 是每一次开不成照这张卡出错报成的那一个（[`Self::GpuFailed`] 或
     /// [`Self::WaitForTheGpuExceededTheLimit`]），按开的次序，头一次的在前；连着试满、这一次不再开的只有一个。
+    /// `pci_state_when_judged_unusable` 是判它开不成那一刻读的 PCI 状态（每一次没开成而还要重开时不读）。
     GpuNotOpenedAfterTheReopenings {
         device: GpuDeviceIdentifier,
         failures_of_each_opening: Vec<BackgroundGpuComputeFailed>,
+        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
     },
     /// 这张卡上一次提交等超过 `wait_limit` 之后，在同一张卡上重交了 `resubmissions` 次、次次等超过上限（「在间隔内重试三次，
     /// 如果gpu无法用就报警」「重试是把那一批重新交一次」）：这张卡用不了。修好它或切换时不再点它，或切回 `cpu`。
     /// `last_submission_waited_for` 是最后那一次等超过上限的是哪一次提交（重交之前先等前一次，那一次没等到也算一次）。
+    /// `pci_state_when_judged_unusable` 是判它用不了那一刻读的 PCI 状态（每次超时重交之前不读）。
     GpuUnusableAfterResubmissions {
         device: GpuDeviceIdentifier,
         resubmissions: u32,
         wait_limit: Duration,
         last_submission_waited_for: SubmissionWaitedForPastTheLimit,
+        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
     },
     /// 这一组设置点的卡的显存上限（扣掉卡上已占的）连最小的一片都装不下，参与的卡一张都不剩（
     /// 「不能因为装不下就拒绝」）：开卡、换设置时按与输入无关的最小的一片判，每一批切段时按这一批的最小的一片判（一张卡这一批切不出就这一批不分给它、
@@ -307,6 +321,35 @@ impl BackgroundGpuComputeFailed {
             BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch { .. } => Vec::new(),
         }
     }
+
+    /// 判那张卡用不了那一刻读的 PCI 状态：出在一张卡上、带着读数的三种交回它带的那一份（照样可以是没读）；
+    /// 别的（等超过上限还没用完重交、输入不支持、参与的卡一张都不剩、一批里同一个条目键出现两次）是没读。
+    #[must_use]
+    pub fn pci_state_when_judged_unusable(&self) -> PciStateOfTheGpuReadWhenJudgedUnusable {
+        match self {
+            BackgroundGpuComputeFailed::GpuFailed {
+                pci_state_when_judged_unusable,
+                ..
+            }
+            | BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings {
+                pci_state_when_judged_unusable,
+                ..
+            }
+            | BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
+                pci_state_when_judged_unusable,
+                ..
+            } => pci_state_when_judged_unusable.clone(),
+            BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit { .. }
+            | BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(..)
+            | BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit { .. }
+            | BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece {
+                ..
+            }
+            | BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch { .. } => {
+                PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError
+            }
+        }
+    }
 }
 
 /// 一张卡这一批因空闲显存连最小的一片都建不成而不用（[`BackgroundGpuComputeFailed::EveryParticipatingGpuShortOfFreeMemoryForTheSmallestPiece`] 带着）：
@@ -367,6 +410,147 @@ pub enum GpuFailureOnTheDevice {
     },
 }
 
+/// 判一张卡用不了、要报警的那一刻照它的 PCI 地址读的本机 PCI 状态（D26（后台整理与放置回收） 已定项 13 第 4 条「判一张卡用不了、要报警的那一刻，查一次这张卡的硬件与链路」）。
+/// 读在产品 GPU 包里做（`singlefs-background-gpu` 的 `pci_state_of_the_gpu` 模块）；核心层不读，只带着它进停下记录与报警，停着时每次叫起交回的报警带的是停下那一刻的同一份。
+/// 读的时刻只有三种：挂着时一批在同一张卡上等超时用完、判它用不了（[`BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions`]）；
+/// 开卡重开用完都开不成（[`BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings`]，挂载那一刻拒绝挂载的那一份在产品 GPU 包的挂载拒绝里）；
+/// 卡当场报出错（设备丢了、wgpu 报了没接住的错）使后台任务停下（[`BackgroundGpuComputeFailed::GpuFailed`] 带 [`GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost`]）。
+/// 封闭集合。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum PciStateOfTheGpuReadWhenJudgedUnusable {
+    /// 这个错不是那三种时刻之一，或不归哪张卡：没读。
+    NotReadForThisError,
+    /// 卡的标识当不了 PCI 设备目录下的一个目录名（不是 UTF-8、是空的、是 `.` 或 `..`、带 `/` 或 NUL 字节）：没读，免得读到那个目录之外去。
+    PciBusAddressNotADirectoryName { device: GpuDeviceIdentifier },
+    /// 读了：九样各自是读到、文件不在、读不通之一。
+    Read(PciStateOfTheGpu),
+}
+
+impl PciStateOfTheGpuReadWhenJudgedUnusable {
+    /// 照读数判这张卡在不在总线上（[`hardware_judgement_of_the_pci_state`]）；没读的交回 `None`。
+    #[must_use]
+    pub fn hardware_judgement(&self) -> Option<HardwareJudgementOfTheGpu> {
+        match self {
+            PciStateOfTheGpuReadWhenJudgedUnusable::Read(pci_state) => {
+                Some(hardware_judgement_of_the_pci_state(pci_state))
+            }
+            PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError
+            | PciStateOfTheGpuReadWhenJudgedUnusable::PciBusAddressNotADirectoryName { .. } => None,
+        }
+    }
+}
+
+/// 照一张卡的 PCI 地址读的那一份：照哪个地址、哪个目录读的，与九样各自的读数。读不到的那一样报读不到（[`PciFileReading::FileNotPresent`]、
+/// [`PciFileReading::NotReadable`]），不当成 0，也不当成正常。链路四样（`current_link_speed`、`current_link_width`、`max_link_speed`、`max_link_width`）
+/// 与高级错误报告的三类出错计数（`aer_dev_correctable`、`aer_dev_fatal`、`aer_dev_nonfatal`，每份是多行「名字 计数」）是文件原文、去掉末尾的换行，只报不判；
+/// 判硬件坏只看目录在不在与使能位（[`hardware_judgement_of_the_pci_state`]）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct PciStateOfTheGpu {
+    /// 照这个地址读的：卡的标识原样（wgpu 适配器信息的 `device_pci_bus_id`，形如 `0000:01:00.0`）。
+    pub pci_bus_address: String,
+    /// 读的那个目录：PCI 设备目录（本机是 `/sys/bus/pci/devices`）下以地址为名的那一个。
+    pub device_directory: PathBuf,
+    /// 那个目录在不在：卡在不在总线上。
+    pub device_directory_reading: PciDeviceDirectoryReading,
+    /// `enable`：使能位。
+    pub device_enable: PciFileReading<PciDeviceEnable>,
+    /// `current_link_speed`：链路此刻的速度。
+    pub current_link_speed: PciFileReading<String>,
+    /// `current_link_width`：链路此刻的宽度。
+    pub current_link_width: PciFileReading<String>,
+    /// `max_link_speed`：链路最大的速度。
+    pub maximum_link_speed: PciFileReading<String>,
+    /// `max_link_width`：链路最大的宽度。
+    pub maximum_link_width: PciFileReading<String>,
+    /// `aer_dev_correctable`：可纠正的出错计数。
+    pub advanced_error_reporting_correctable: PciFileReading<String>,
+    /// `aer_dev_fatal`：致命的出错计数。
+    pub advanced_error_reporting_fatal: PciFileReading<String>,
+    /// `aer_dev_nonfatal`：不致命的出错计数。
+    pub advanced_error_reporting_nonfatal: PciFileReading<String>,
+}
+
+/// 一张卡的 PCI 设备目录在不在（[`PciStateOfTheGpu::device_directory_reading`]）。封闭集合。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum PciDeviceDirectoryReading {
+    Present,
+    /// 目录不在：卡不在总线上。
+    NotPresent,
+    /// 打开那个目录报了别的错（`system_error` 是系统报的原话）：读不通，不当成在、也不当成不在。
+    NotReadable {
+        system_error: String,
+    },
+}
+
+/// PCI 设备目录下一个文件的读数。封闭集合。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum PciFileReading<Content> {
+    Read(Content),
+    /// 文件不在（例：不报高级错误报告的设备没有那三份）。
+    FileNotPresent,
+    /// 读那个文件报了别的错（`system_error` 是系统报的原话）。
+    NotReadable {
+        system_error: String,
+    },
+}
+
+/// `enable` 读到的：内核记的这个设备的使能计数（文件原文是一个十进制数），0 是关的。封闭集合。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum PciDeviceEnable {
+    Enabled {
+        enable_count: u64,
+    },
+    Disabled,
+    /// 原文不是一个十进制数（`text` 是去掉首尾空白的原文）：不判开关。
+    UnrecognizedText {
+        text: String,
+    },
+}
+
+/// 照一张卡的 PCI 读数判的硬件好坏（[`hardware_judgement_of_the_pci_state`]）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum HardwareJudgementOfTheGpu {
+    /// 卡不在总线上：判硬件坏。
+    OffTheBus(ReasonTheGpuIsOffTheBus),
+    /// 没判出硬件坏（读到的链路与出错计数只报不判；使能位读不到也落在这里）。
+    NotJudgedOffTheBus,
+}
+
+/// 判卡不在总线上的依据。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ReasonTheGpuIsOffTheBus {
+    /// 那个 PCI 地址的目录不在了。
+    DeviceDirectoryNotPresent,
+    /// 使能位读到了、是关的。
+    DeviceDisabled,
+}
+
+/// 照一张卡的 PCI 读数判硬件坏没坏：只有那个地址的目录不在、或使能位读到了而是关的，才判卡不在总线上；
+/// 目录读不通、使能位文件不在或读不通、原文认不得都不判坏；链路速度与宽度、出错计数不看。不读盘、不读内核导出的设备文件，只看交进来的读数。
+#[must_use]
+pub fn hardware_judgement_of_the_pci_state(
+    pci_state: &PciStateOfTheGpu,
+) -> HardwareJudgementOfTheGpu {
+    match &pci_state.device_directory_reading {
+        PciDeviceDirectoryReading::NotPresent => {
+            return HardwareJudgementOfTheGpu::OffTheBus(
+                ReasonTheGpuIsOffTheBus::DeviceDirectoryNotPresent,
+            );
+        }
+        PciDeviceDirectoryReading::Present | PciDeviceDirectoryReading::NotReadable { .. } => {}
+    }
+    match &pci_state.device_enable {
+        PciFileReading::Read(PciDeviceEnable::Disabled) => {
+            HardwareJudgementOfTheGpu::OffTheBus(ReasonTheGpuIsOffTheBus::DeviceDisabled)
+        }
+        PciFileReading::Read(
+            PciDeviceEnable::Enabled { .. } | PciDeviceEnable::UnrecognizedText { .. },
+        )
+        | PciFileReading::FileNotPresent
+        | PciFileReading::NotReadable { .. } => HardwareJudgementOfTheGpu::NotJudgedOffTheBus,
+    }
+}
+
 /// 等超过上限的是哪一次提交（[`BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit`] 带着）。封闭集合。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum SubmissionWaitedForPastTheLimit {
@@ -1302,6 +1486,21 @@ impl BackgroundComputeError {
             }
         }
     }
+
+    /// 这个错带的、判那张卡用不了那一刻读的 PCI 状态（[`BackgroundGpuComputeFailed::pci_state_when_judged_unusable`]）；
+    /// 只供测试的钩子注入的不归哪张卡，是没读。
+    #[must_use]
+    pub fn pci_state_when_judged_unusable(&self) -> PciStateOfTheGpuReadWhenJudgedUnusable {
+        match self {
+            BackgroundComputeError::InjectedByTheTestOnlyHook => {
+                PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError
+            }
+            BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn { failure }
+            | BackgroundComputeError::GpuSegmentFailed { failure, .. } => {
+                failure.pci_state_when_judged_unusable()
+            }
+        }
+    }
 }
 
 /// 后台任务因一批出错停着时，每次叫起都交回的报警（D26（后台整理与放置回收） 已定项 13 第 4 条：「每次」是每次后台整理被叫起，通道是返回值里必须处理的报警）：
@@ -1316,6 +1515,9 @@ pub struct AlarmOfABackgroundTaskStoppedAfterAComputeError {
     pub task: BackgroundTask,
     /// 停下的那一个错点名的卡（[`BackgroundComputeError::gpu_devices_named`]），按错里的次序；不归哪张卡的是空。
     pub gpu_devices_named_by_the_error: Vec<GpuDeviceIdentifier>,
+    /// 停下的那一个错带的、判那张卡用不了那一刻读的 PCI 状态（[`BackgroundComputeError::pci_state_when_judged_unusable`]）：
+    /// 每次叫起都是停下那一刻的同一份，不重读；不是那三种时刻、不归哪张卡的是没读。
+    pub pci_state_of_the_gpu_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
     pub error_that_stopped_the_task: BackgroundComputeError,
     /// 停下以来叫起了几次（含这一次；一批出错停下的那一趟不算，那一趟报 0）。
     pub wakes_while_stopped: u64,
@@ -1612,6 +1814,7 @@ impl BackgroundComputeOfThisMount {
         Some(AlarmOfABackgroundTaskStoppedAfterAComputeError {
             task,
             gpu_devices_named_by_the_error: stopped.error.gpu_devices_named(),
+            pci_state_of_the_gpu_judged_unusable: stopped.error.pci_state_when_judged_unusable(),
             error_that_stopped_the_task: stopped.error,
             wakes_while_stopped,
             batches_compensated_on_the_cpu: stopped.batches_compensated_on_the_cpu,
@@ -2323,6 +2526,8 @@ mod tests {
                 cause: GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost {
                     reason: "替身：切换之前那张卡上设备丢了".to_string(),
                 },
+                pci_state_when_judged_unusable:
+                    PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
             },
         }
     }
@@ -2726,6 +2931,8 @@ mod tests {
         BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings {
             device: card_for_the_switch_tests(b"0000:41:00.0"),
             failures_of_each_opening: Vec::new(),
+            pci_state_when_judged_unusable:
+                PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
         }
     }
 
@@ -3240,4 +3447,201 @@ mod tests {
             (1, 2)
         );
     }
+
+    /// 一份读到的 PCI 状态：目录照 `device_directory_reading`、使能位照 `device_enable`；链路此刻降到 5.0 GT/s ×4（最大 32.0 GT/s ×16），
+    /// 可纠正的出错计数不是 0，致命的那份不在、不致命的那份读不通。
+    fn pci_state_of_the_card_at_one(
+        device_directory_reading: PciDeviceDirectoryReading,
+        device_enable: PciFileReading<PciDeviceEnable>,
+    ) -> PciStateOfTheGpu {
+        PciStateOfTheGpu {
+            pci_bus_address: "0000:01:00.0".to_string(),
+            device_directory: PathBuf::from("/pci-device-directories-of-the-test/0000:01:00.0"),
+            device_directory_reading,
+            device_enable,
+            current_link_speed: PciFileReading::Read("5.0 GT/s PCIe".to_string()),
+            current_link_width: PciFileReading::Read("4".to_string()),
+            maximum_link_speed: PciFileReading::Read("32.0 GT/s PCIe".to_string()),
+            maximum_link_width: PciFileReading::Read("16".to_string()),
+            advanced_error_reporting_correctable: PciFileReading::Read(
+                "RxErr 3\nTOTAL_ERR_COR 3".to_string(),
+            ),
+            advanced_error_reporting_fatal: PciFileReading::FileNotPresent,
+            advanced_error_reporting_nonfatal: PciFileReading::NotReadable {
+                system_error: "替身：读不通".to_string(),
+            },
+        }
+    }
+
+    /// 第 4 条「只有卡不在总线上（那个 PCI 地址的目录不在了，或使能位是关的）才判硬件坏」：目录不在就判不在总线上，使能位读没读到都一样。
+    #[test]
+    fn the_gpu_whose_device_directory_is_not_present_is_judged_off_the_bus() {
+        assert_eq!(
+            hardware_judgement_of_the_pci_state(&pci_state_of_the_card_at_one(
+                PciDeviceDirectoryReading::NotPresent,
+                PciFileReading::FileNotPresent,
+            )),
+            HardwareJudgementOfTheGpu::OffTheBus(
+                ReasonTheGpuIsOffTheBus::DeviceDirectoryNotPresent
+            )
+        );
+    }
+
+    /// 同一条：目录在、使能位读到 0（关的）判不在总线上。
+    #[test]
+    fn the_gpu_whose_device_enable_reads_disabled_is_judged_off_the_bus() {
+        assert_eq!(
+            hardware_judgement_of_the_pci_state(&pci_state_of_the_card_at_one(
+                PciDeviceDirectoryReading::Present,
+                PciFileReading::Read(PciDeviceEnable::Disabled),
+            )),
+            HardwareJudgementOfTheGpu::OffTheBus(ReasonTheGpuIsOffTheBus::DeviceDisabled)
+        );
+    }
+
+    /// 同一条「链路速度与宽度、出错计数只报不判」：目录在、使能位开着，链路降速降宽、出错计数不是 0，也不判坏。
+    #[test]
+    fn the_gpu_enabled_with_a_degraded_link_and_errors_counted_is_not_judged_off_the_bus() {
+        assert_eq!(
+            hardware_judgement_of_the_pci_state(&pci_state_of_the_card_at_one(
+                PciDeviceDirectoryReading::Present,
+                PciFileReading::Read(PciDeviceEnable::Enabled { enable_count: 1 }),
+            )),
+            HardwareJudgementOfTheGpu::NotJudgedOffTheBus
+        );
+    }
+
+    /// 同一条「读不到的那一样报读不到，不当成 0」：目录在，使能位文件不在、读不通、原文认不得，都不判坏（不当成读到了关的）。
+    #[test]
+    fn the_gpu_whose_device_enable_is_not_read_is_not_judged_off_the_bus() {
+        let judgements: Vec<HardwareJudgementOfTheGpu> = [
+            PciFileReading::FileNotPresent,
+            PciFileReading::NotReadable {
+                system_error: "替身：读不通".to_string(),
+            },
+            PciFileReading::Read(PciDeviceEnable::UnrecognizedText {
+                text: "替身：认不得".to_string(),
+            }),
+        ]
+        .into_iter()
+        .map(|device_enable| {
+            hardware_judgement_of_the_pci_state(&pci_state_of_the_card_at_one(
+                PciDeviceDirectoryReading::Present,
+                device_enable,
+            ))
+        })
+        .collect();
+        assert_eq!(
+            judgements,
+            vec![HardwareJudgementOfTheGpu::NotJudgedOffTheBus; 3]
+        );
+    }
+
+    /// 同一条：目录读不通（不是不在）不判坏，照使能位判。
+    #[test]
+    fn the_gpu_whose_device_directory_is_not_readable_is_judged_by_its_device_enable() {
+        assert_eq!(
+            hardware_judgement_of_the_pci_state(&pci_state_of_the_card_at_one(
+                PciDeviceDirectoryReading::NotReadable {
+                    system_error: "替身：打不开".to_string(),
+                },
+                PciFileReading::Read(PciDeviceEnable::Enabled { enable_count: 2 }),
+            )),
+            HardwareJudgementOfTheGpu::NotJudgedOffTheBus
+        );
+    }
+
+    /// 第 4 条「停着之后每次叫起交回的报警带停下那一刻读的同一份读数，不重读」：一批因「0000:01:00.0」重交用完停下（错里带判它用不了那一刻读的一份），
+    /// 停下那一趟与之后三次叫起的报警都带那一份，与停下记录里那个错带的相同。
+    // harness-test-granularity:one-scenario 同一份停着的状态上按次序叫起几次，钉的是每一次报警带的都是停下那一刻的同一份
+    #[test]
+    fn the_alarm_at_every_wake_while_stopped_carries_the_pci_state_read_when_the_gpu_was_judged_unusable(
+    ) {
+        let pci_state_read =
+            PciStateOfTheGpuReadWhenJudgedUnusable::Read(pci_state_of_the_card_at_one(
+                PciDeviceDirectoryReading::Present,
+                PciFileReading::Read(PciDeviceEnable::Enabled { enable_count: 1 }),
+            ));
+        let error = BackgroundComputeError::GpuSegmentFailed {
+            segment: GpuComputedSegment::UnitChecksumVerification,
+            failure: BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
+                device: card_for_the_switch_tests(b"0000:01:00.0"),
+                resubmissions: 3,
+                wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+                last_submission_waited_for: SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
+                pci_state_when_judged_unusable: pci_state_read.clone(),
+            },
+        };
+        let mut background_compute =
+            BackgroundComputeOfThisMount::of_a_mount(AdmittedBackgroundComputeSettings::PRODUCT);
+        background_compute.begin_a_wake();
+        assert_eq!(
+            background_compute
+                .stop_the_task_after_a_compute_error(BackgroundTask::CompactionRelocation, error),
+            BackgroundTaskAfterTheComputeError::LeftStopped
+        );
+        let pci_states_of_the_alarms: Vec<Option<PciStateOfTheGpuReadWhenJudgedUnusable>> = (0..4)
+            .map(|wake| {
+                if wake > 0 {
+                    background_compute.begin_a_wake();
+                }
+                background_compute
+                    .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
+                    .map(|alarm| alarm.pci_state_of_the_gpu_judged_unusable)
+            })
+            .collect();
+        assert_eq!(
+            pci_states_of_the_alarms,
+            vec![Some(pci_state_read.clone()); 4],
+            "停下那一趟与之后每次叫起的报警都带停下那一刻读的那一份"
+        );
+        assert_eq!(
+            background_compute.stopped_after_a_compute_error()[0]
+                .error
+                .pci_state_when_judged_unusable(),
+            pci_state_read,
+            "停下记录里的那个错带的是同一份"
+        );
+    }
+
+    /// 第 4 条：不是判一张卡用不了的那三种时刻、或不归哪张卡的错不读，报的是没读那个显式成员：只供测试的钩子注入的停下之后报警照报、读数那一格是没读；
+    /// 一批里同一个条目键出现两次、一次等超过上限（还没用完重交）、参与的卡一张都不剩的错带的也是没读。
+    #[test]
+    fn errors_not_judging_a_gpu_unusable_carry_no_pci_state_read() {
+        let mut background_compute =
+            BackgroundComputeOfThisMount::of_a_mount(AdmittedBackgroundComputeSettings::PRODUCT);
+        assert_eq!(
+            background_compute.stop_the_task_after_a_compute_error(
+                BackgroundTask::CompactionRelocation,
+                BackgroundComputeError::InjectedByTheTestOnlyHook,
+            ),
+            BackgroundTaskAfterTheComputeError::LeftStopped
+        );
+        background_compute.begin_a_wake();
+        assert_eq!(
+            background_compute
+                .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
+                .map(|alarm| alarm.pci_state_of_the_gpu_judged_unusable),
+            Some(PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError)
+        );
+        let failures_not_judging_a_gpu_unusable = [
+            BackgroundGpuComputeFailed::SameEntryKeyTwiceInOneBatch {
+                position_of_the_first_occurrence: 0,
+                position_of_the_second_occurrence: 1,
+            },
+            BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit {
+                device: card_for_the_switch_tests(b"0000:01:00.0"),
+                wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
+                submission_waited_for: SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
+            },
+            every_gpu_left_out_on_the_card(b"0000:01:00.0"),
+        ];
+        assert_eq!(
+            failures_not_judging_a_gpu_unusable
+                .iter()
+                .map(BackgroundGpuComputeFailed::pci_state_when_judged_unusable)
+                .collect::<Vec<_>>(),
+            vec![PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError; 3]
+        );
+    }
 }
diff --git a/crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs b/crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs
index fe7ac22..c5bec86 100644
--- a/crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs
+++ b/crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs
@@ -80,6 +80,7 @@ pub fn device_lost_on_the_card_with_eight_gibibytes() -> BackgroundGpuComputeFai
         cause: GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost {
             reason: "假的：设备丢了".to_string(),
         },
+        pci_state_when_judged_unusable: singlefs_core::background_compute::PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
     }
 }
 
diff --git a/crates/singlefs-verification/tests/compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs b/crates/singlefs-verification/tests/compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs
index b2d7c77..1ba61ac 100644
--- a/crates/singlefs-verification/tests/compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs
+++ b/crates/singlefs-verification/tests/compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs
@@ -485,6 +485,7 @@ fn every_participating_gpu_short_of_free_memory_abandons_the_batch_and_leaves_th
                 card_named("card-eight"),
                 card_named("card-sixteen")
             ],
+            pci_state_of_the_gpu_judged_unusable: singlefs_core::background_compute::PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
             error_that_stopped_the_task: error,
             wakes_while_stopped: 0,
             batches_compensated_on_the_cpu: 0,
@@ -886,6 +887,7 @@ fn every_kind_of_failure_of_the_gpu_compute_stops_the_compaction_reporting_its_s
         BackgroundGpuComputeFailed::GpuFailed {
             device: device.clone(),
             cause,
+            pci_state_when_judged_unusable: singlefs_core::background_compute::PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
         }
     };
     let every_kind_of_failure = [
diff --git a/crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs b/crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs
index 7d29eb1..4c98dce 100644
--- a/crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs
+++ b/crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs
@@ -383,6 +383,7 @@ fn alarm_of_the_stop(
     AlarmOfABackgroundTaskStoppedAfterAComputeError {
         task: BackgroundTask::CompactionRelocation,
         gpu_devices_named_by_the_error: vec![card_named("card-eight")],
+        pci_state_of_the_gpu_judged_unusable: singlefs_core::background_compute::PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
         error_that_stopped_the_task: error.clone(),
         wakes_while_stopped,
         batches_compensated_on_the_cpu,
diff --git a/crates/singlefs-verification/tests/compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs b/crates/singlefs-verification/tests/compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs
index 6defd7c..0328545 100644
--- a/crates/singlefs-verification/tests/compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs
+++ b/crates/singlefs-verification/tests/compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs
@@ -218,6 +218,7 @@ fn the_first_wake_after_a_mount_with_no_usable_gpu_returns_the_alarm() {
         Some(AlarmOfABackgroundTaskStoppedAfterAComputeError {
             task: BackgroundTask::CompactionRelocation,
             gpu_devices_named_by_the_error: vec![card_named("card-eight")],
+            pci_state_of_the_gpu_judged_unusable: singlefs_core::background_compute::PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
             error_that_stopped_the_task:
                 BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
                     failure: every_participating_gpu_left_out(),
```

## 第二份：编码纪律修补（接在第一份之后）

```diff
diff --git a/crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs b/crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs
index ad1a5c2..62e6c30 100644
--- a/crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs
+++ b/crates/singlefs-background-gpu-tests/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs
@@ -957,7 +957,9 @@ fn the_card_named_by_a_switch_not_opened_after_the_reopenings_stops_the_compacti
                 )
             })
             .collect(),
-        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+        pci_state_when_judged_unusable: Box::new(
+            PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+        ),
     };
     assert_eq!(
         session.background_compute().stopped_after_a_compute_error(),
@@ -1308,8 +1310,9 @@ fn the_gpu_unusable_after_its_resubmissions_stops_the_compaction_reporting_the_c
                     wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                     last_submission_waited_for:
                         SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
-                    pci_state_when_judged_unusable:
+                    pci_state_when_judged_unusable: Box::new(
                         PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+                    ),
                 },
             },
             batches_compensated_on_the_cpu: 0,
diff --git a/crates/singlefs-background-gpu/src/background_gpu_compute.rs b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
index 0ddf36b..61dfa6c 100644
--- a/crates/singlefs-background-gpu/src/background_gpu_compute.rs
+++ b/crates/singlefs-background-gpu/src/background_gpu_compute.rs
@@ -307,7 +307,7 @@ pub fn core_failure_of_the_gpu(
         |cause_on_the_device: GpuFailureOnTheDevice| BackgroundGpuComputeFailed::GpuFailed {
             device: device.clone(),
             cause: cause_on_the_device,
-            pci_state_when_judged_unusable: pci_state_when_judged_unusable.clone(),
+            pci_state_when_judged_unusable: Box::new(pci_state_when_judged_unusable.clone()),
         };
     match cause {
         GpuFailureCause::WaitForTheGpuExceededTheLimit { wait_limit } => {
@@ -398,7 +398,7 @@ pub fn core_failure_of_the_gpu(
                     wait_limit: *wait_limit,
                     last_submission_waited_for:
                         SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
-                    pci_state_when_judged_unusable: pci_state_when_judged_unusable.clone(),
+                    pci_state_when_judged_unusable: Box::new(pci_state_when_judged_unusable.clone()),
                 }
             }
             GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit {
@@ -409,7 +409,7 @@ pub fn core_failure_of_the_gpu(
                 wait_limit: *wait_limit,
                 last_submission_waited_for:
                     SubmissionWaitedForPastTheLimit::EarlierTimedOutSubmissionStillRunning,
-                pci_state_when_judged_unusable: pci_state_when_judged_unusable.clone(),
+                pci_state_when_judged_unusable: Box::new(pci_state_when_judged_unusable.clone()),
             },
             GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { wait_limit } => {
                 BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
@@ -418,7 +418,7 @@ pub fn core_failure_of_the_gpu(
                     wait_limit: *wait_limit,
                     last_submission_waited_for:
                         SubmissionWaitedForPastTheLimit::EmptySubmissionFlushingThePendingWrites,
-                    pci_state_when_judged_unusable: pci_state_when_judged_unusable.clone(),
+                    pci_state_when_judged_unusable: Box::new(pci_state_when_judged_unusable.clone()),
                 }
             }
             // 只有等超过上限的三种才重交（`crate::waiting_for_the_gpu::outcome_resubmitted_after_timeouts`），别的原因当场交回、不包进这一种；
@@ -459,7 +459,7 @@ pub fn core_failure_of_the_gpu(
                     )
                 })
                 .collect(),
-            pci_state_when_judged_unusable: pci_state_when_judged_unusable.clone(),
+            pci_state_when_judged_unusable: Box::new(pci_state_when_judged_unusable.clone()),
         },
     }
 }
@@ -819,8 +819,9 @@ mod tests {
             |cause: GpuFailureOnTheDevice| BackgroundGpuComputeFailed::GpuFailed {
                 device: card_named_in_the_failures(),
                 cause,
-                pci_state_when_judged_unusable:
+                pci_state_when_judged_unusable: Box::new(
                     PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+                ),
             };
         vec![
             (
@@ -957,8 +958,9 @@ mod tests {
                                 SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
                         },
                     ],
-                    pci_state_when_judged_unusable:
+                    pci_state_when_judged_unusable: Box::new(
                         PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+                    ),
                 },
             ),
         ]
@@ -1095,8 +1097,9 @@ mod tests {
                 resubmissions: 3,
                 wait_limit,
                 last_submission_waited_for,
-                pci_state_when_judged_unusable:
+                pci_state_when_judged_unusable: Box::new(
                     PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+                ),
             };
             assert_eq!(
                 core_failure_of_the_checksum_batch(&ChecksumBatchFailed::GpuFailed {
@@ -1337,10 +1340,11 @@ mod tests {
                     cause: GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost {
                         reason: "假的：开卡时设备丢了".to_string(),
                     },
-                    pci_state_when_judged_unusable:
+                    pci_state_when_judged_unusable: Box::new(
                         PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+                    ),
                 }],
-                pci_state_when_judged_unusable: pci_state_read,
+                pci_state_when_judged_unusable: Box::new(pci_state_read),
             }
         );
     }
diff --git a/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs b/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs
index 63b6dac..cbac1ff 100644
--- a/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs
+++ b/crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs
@@ -66,12 +66,12 @@ impl PciStateReaderOfTheGpus {
     ) -> PciStateOfTheGpuReadWhenJudgedUnusable {
         self.readings_taken.fetch_add(1, Ordering::SeqCst);
         match pci_bus_address_usable_as_a_directory_name(device.opaque_bytes()) {
-            Some(pci_bus_address) => {
-                PciStateOfTheGpuReadWhenJudgedUnusable::Read(pci_state_under_the_device_directory(
+            Some(pci_bus_address) => PciStateOfTheGpuReadWhenJudgedUnusable::Read(Box::new(
+                pci_state_under_the_device_directory(
                     pci_bus_address,
                     self.pci_device_directories_root.join(pci_bus_address),
-                ))
-            }
+                ),
+            )),
             None => PciStateOfTheGpuReadWhenJudgedUnusable::PciBusAddressNotADirectoryName {
                 device: device.clone(),
             },
@@ -239,7 +239,7 @@ pub(crate) fn pci_state_reader_under_a_root_not_present_for_tests() -> PciStateR
 pub(crate) fn pci_state_read_under_the_root_not_present_for_tests(
     pci_bus_address: &str,
 ) -> PciStateOfTheGpuReadWhenJudgedUnusable {
-    PciStateOfTheGpuReadWhenJudgedUnusable::Read(PciStateOfTheGpu {
+    PciStateOfTheGpuReadWhenJudgedUnusable::Read(Box::new(PciStateOfTheGpu {
         pci_bus_address: pci_bus_address.to_string(),
         device_directory: pci_device_directories_root_not_present_for_tests().join(pci_bus_address),
         device_directory_reading: PciDeviceDirectoryReading::NotPresent,
@@ -251,7 +251,7 @@ pub(crate) fn pci_state_read_under_the_root_not_present_for_tests(
         advanced_error_reporting_correctable: PciFileReading::FileNotPresent,
         advanced_error_reporting_fatal: PciFileReading::FileNotPresent,
         advanced_error_reporting_nonfatal: PciFileReading::FileNotPresent,
-    })
+    }))
 }
 
 #[cfg(test)]
@@ -339,7 +339,7 @@ mod tests {
         let reading = reader.pci_state_of_the_gpu(&card("0000:01:00.0"));
         assert_eq!(
             reading,
-            PciStateOfTheGpuReadWhenJudgedUnusable::Read(PciStateOfTheGpu {
+            PciStateOfTheGpuReadWhenJudgedUnusable::Read(Box::new(PciStateOfTheGpu {
                 pci_bus_address: "0000:01:00.0".to_string(),
                 device_directory: directories.root.join("0000:01:00.0"),
                 device_directory_reading: PciDeviceDirectoryReading::Present,
@@ -357,7 +357,7 @@ mod tests {
                 advanced_error_reporting_nonfatal: PciFileReading::Read(
                     "TOTAL_ERR_NONFATAL 1".to_string()
                 ),
-            })
+            }))
         );
         assert_eq!(
             reading.hardware_judgement(),
diff --git a/crates/singlefs-core/src/background_compute.rs b/crates/singlefs-core/src/background_compute.rs
index 3e7089f..f8448ac 100644
--- a/crates/singlefs-core/src/background_compute.rs
+++ b/crates/singlefs-core/src/background_compute.rs
@@ -226,10 +226,12 @@ pub enum BackgroundGpuComputeFailed {
     /// 这张卡出错（起不了、设备丢了、等不成、回读坏了、核留下认不出的判定字）：修好它或切换时不再点它，或切回 `cpu`。
     /// `pci_state_when_judged_unusable` 是挂着时一批里卡当场报出错（[`GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost`]）那一刻读的 PCI 状态；
     /// 别的原因、与开卡每一次没开成的原因（[`Self::GpuNotOpenedAfterTheReopenings`] 的 `failures_of_each_opening`）都是没读。
+    /// 带读数的三个成员都把读数装箱：读数只在出错时有，装箱之后这个出错类型只多一个指针，装着它的
+    /// [`BackgroundComputeError`]、报警与返回它的 `Result` 不过 clippy `result_large_err` 与 `large_enum_variant` 的线。
     GpuFailed {
         device: GpuDeviceIdentifier,
         cause: GpuFailureOnTheDevice,
-        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
+        pci_state_when_judged_unusable: Box<PciStateOfTheGpuReadWhenJudgedUnusable>,
     },
     /// 等这张卡超过了 `background_gpu_wait_limit`：调大上限，或查这张卡是不是卡死了。
     WaitForTheGpuExceededTheLimit {
@@ -248,7 +250,7 @@ pub enum BackgroundGpuComputeFailed {
     GpuNotOpenedAfterTheReopenings {
         device: GpuDeviceIdentifier,
         failures_of_each_opening: Vec<BackgroundGpuComputeFailed>,
-        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
+        pci_state_when_judged_unusable: Box<PciStateOfTheGpuReadWhenJudgedUnusable>,
     },
     /// 这张卡上一次提交等超过 `wait_limit` 之后，在同一张卡上重交了 `resubmissions` 次、次次等超过上限（「在间隔内重试三次，
     /// 如果gpu无法用就报警」「重试是把那一批重新交一次」）：这张卡用不了。修好它或切换时不再点它，或切回 `cpu`。
@@ -259,7 +261,7 @@ pub enum BackgroundGpuComputeFailed {
         resubmissions: u32,
         wait_limit: Duration,
         last_submission_waited_for: SubmissionWaitedForPastTheLimit,
-        pci_state_when_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
+        pci_state_when_judged_unusable: Box<PciStateOfTheGpuReadWhenJudgedUnusable>,
     },
     /// 这一组设置点的卡的显存上限（扣掉卡上已占的）连最小的一片都装不下，参与的卡一张都不剩（
     /// 「不能因为装不下就拒绝」）：开卡、换设置时按与输入无关的最小的一片判，每一批切段时按这一批的最小的一片判（一张卡这一批切不出就这一批不分给它、
@@ -338,7 +340,7 @@ impl BackgroundGpuComputeFailed {
             | BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
                 pci_state_when_judged_unusable,
                 ..
-            } => pci_state_when_judged_unusable.clone(),
+            } => PciStateOfTheGpuReadWhenJudgedUnusable::clone(pci_state_when_judged_unusable),
             BackgroundGpuComputeFailed::WaitForTheGpuExceededTheLimit { .. }
             | BackgroundGpuComputeFailed::InputNotSupportedOnTheGpu(..)
             | BackgroundGpuComputeFailed::EveryParticipatingGpuLeftOutUnderItsMemoryLimit { .. }
@@ -422,8 +424,9 @@ pub enum PciStateOfTheGpuReadWhenJudgedUnusable {
     NotReadForThisError,
     /// 卡的标识当不了 PCI 设备目录下的一个目录名（不是 UTF-8、是空的、是 `.` 或 `..`、带 `/` 或 NUL 字节）：没读，免得读到那个目录之外去。
     PciBusAddressNotADirectoryName { device: GpuDeviceIdentifier },
-    /// 读了：九样各自是读到、文件不在、读不通之一。
-    Read(PciStateOfTheGpu),
+    /// 读了：九样各自是读到、文件不在、读不通之一。装箱：九样连同地址与目录比另两个成员大三百多字节（clippy `large_enum_variant`），
+    /// 这个枚举又放在出错成员里，不装箱就带得装着它的出错类型、报警与 `Result` 一起变大。
+    Read(Box<PciStateOfTheGpu>),
 }
 
 impl PciStateOfTheGpuReadWhenJudgedUnusable {
@@ -1515,9 +1518,6 @@ pub struct AlarmOfABackgroundTaskStoppedAfterAComputeError {
     pub task: BackgroundTask,
     /// 停下的那一个错点名的卡（[`BackgroundComputeError::gpu_devices_named`]），按错里的次序；不归哪张卡的是空。
     pub gpu_devices_named_by_the_error: Vec<GpuDeviceIdentifier>,
-    /// 停下的那一个错带的、判那张卡用不了那一刻读的 PCI 状态（[`BackgroundComputeError::pci_state_when_judged_unusable`]）：
-    /// 每次叫起都是停下那一刻的同一份，不重读；不是那三种时刻、不归哪张卡的是没读。
-    pub pci_state_of_the_gpu_judged_unusable: PciStateOfTheGpuReadWhenJudgedUnusable,
     pub error_that_stopped_the_task: BackgroundComputeError,
     /// 停下以来叫起了几次（含这一次；一批出错停下的那一趟不算，那一趟报 0）。
     pub wakes_while_stopped: u64,
@@ -1525,6 +1525,18 @@ pub struct AlarmOfABackgroundTaskStoppedAfterAComputeError {
     pub batches_compensated_on_the_cpu: u64,
 }
 
+impl AlarmOfABackgroundTaskStoppedAfterAComputeError {
+    /// 停下的那一个错带的、判那张卡用不了那一刻读的 PCI 状态（[`BackgroundComputeError::pci_state_when_judged_unusable`]）：
+    /// 每次叫起都是停下那一刻的同一份，不重读；不是那三种时刻、不归哪张卡的是没读。
+    /// 从 `error_that_stopped_the_task` 现取、不另存一份：报警放在 `Option` 里、又放进挂载会话那几个拒绝成员，多存一份读数
+    /// 就让装着它的 `Result` 过 clippy `result_large_err` 的线。
+    #[must_use]
+    pub fn pci_state_of_the_gpu_judged_unusable(&self) -> PciStateOfTheGpuReadWhenJudgedUnusable {
+        self.error_that_stopped_the_task
+            .pci_state_when_judged_unusable()
+    }
+}
+
 /// 一种停着的后台任务停下以来叫起了几次（[`BackgroundComputeOfThisMount::alarm_at_the_end_of_a_wake`] 数的）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 struct WakesOfAStoppedTask {
@@ -1814,7 +1826,6 @@ impl BackgroundComputeOfThisMount {
         Some(AlarmOfABackgroundTaskStoppedAfterAComputeError {
             task,
             gpu_devices_named_by_the_error: stopped.error.gpu_devices_named(),
-            pci_state_of_the_gpu_judged_unusable: stopped.error.pci_state_when_judged_unusable(),
             error_that_stopped_the_task: stopped.error,
             wakes_while_stopped,
             batches_compensated_on_the_cpu: stopped.batches_compensated_on_the_cpu,
@@ -2526,8 +2537,9 @@ mod tests {
                 cause: GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost {
                     reason: "替身：切换之前那张卡上设备丢了".to_string(),
                 },
-                pci_state_when_judged_unusable:
+                pci_state_when_judged_unusable: Box::new(
                     PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+                ),
             },
         }
     }
@@ -2931,8 +2943,9 @@ mod tests {
         BackgroundGpuComputeFailed::GpuNotOpenedAfterTheReopenings {
             device: card_for_the_switch_tests(b"0000:41:00.0"),
             failures_of_each_opening: Vec::new(),
-            pci_state_when_judged_unusable:
+            pci_state_when_judged_unusable: Box::new(
                 PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+            ),
         }
     }
 
@@ -3558,10 +3571,10 @@ mod tests {
     fn the_alarm_at_every_wake_while_stopped_carries_the_pci_state_read_when_the_gpu_was_judged_unusable(
     ) {
         let pci_state_read =
-            PciStateOfTheGpuReadWhenJudgedUnusable::Read(pci_state_of_the_card_at_one(
+            PciStateOfTheGpuReadWhenJudgedUnusable::Read(Box::new(pci_state_of_the_card_at_one(
                 PciDeviceDirectoryReading::Present,
                 PciFileReading::Read(PciDeviceEnable::Enabled { enable_count: 1 }),
-            ));
+            )));
         let error = BackgroundComputeError::GpuSegmentFailed {
             segment: GpuComputedSegment::UnitChecksumVerification,
             failure: BackgroundGpuComputeFailed::GpuUnusableAfterResubmissions {
@@ -3569,7 +3582,7 @@ mod tests {
                 resubmissions: 3,
                 wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
                 last_submission_waited_for: SubmissionWaitedForPastTheLimit::SubmissionOfThisBatch,
-                pci_state_when_judged_unusable: pci_state_read.clone(),
+                pci_state_when_judged_unusable: Box::new(pci_state_read.clone()),
             },
         };
         let mut background_compute =
@@ -3587,7 +3600,7 @@ mod tests {
                 }
                 background_compute
                     .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
-                    .map(|alarm| alarm.pci_state_of_the_gpu_judged_unusable)
+                    .map(|alarm| alarm.pci_state_of_the_gpu_judged_unusable())
             })
             .collect();
         assert_eq!(
@@ -3621,7 +3634,7 @@ mod tests {
         assert_eq!(
             background_compute
                 .alarm_at_the_end_of_a_wake(BackgroundTask::CompactionRelocation)
-                .map(|alarm| alarm.pci_state_of_the_gpu_judged_unusable),
+                .map(|alarm| alarm.pci_state_of_the_gpu_judged_unusable()),
             Some(PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError)
         );
         let failures_not_judging_a_gpu_unusable = [
diff --git a/crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs b/crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs
index c5bec86..1634b59 100644
--- a/crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs
+++ b/crates/singlefs-verification/tests/common_background_gpu_compute/mod.rs
@@ -80,7 +80,9 @@ pub fn device_lost_on_the_card_with_eight_gibibytes() -> BackgroundGpuComputeFai
         cause: GpuFailureOnTheDevice::UncapturedErrorOrDeviceLost {
             reason: "假的：设备丢了".to_string(),
         },
-        pci_state_when_judged_unusable: singlefs_core::background_compute::PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+        pci_state_when_judged_unusable: Box::new(
+            singlefs_core::background_compute::PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+        ),
     }
 }
 
diff --git a/crates/singlefs-verification/tests/compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs b/crates/singlefs-verification/tests/compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs
index 1ba61ac..c271b84 100644
--- a/crates/singlefs-verification/tests/compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs
+++ b/crates/singlefs-verification/tests/compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs
@@ -485,7 +485,6 @@ fn every_participating_gpu_short_of_free_memory_abandons_the_batch_and_leaves_th
                 card_named("card-eight"),
                 card_named("card-sixteen")
             ],
-            pci_state_of_the_gpu_judged_unusable: singlefs_core::background_compute::PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
             error_that_stopped_the_task: error,
             wakes_while_stopped: 0,
             batches_compensated_on_the_cpu: 0,
@@ -887,7 +886,9 @@ fn every_kind_of_failure_of_the_gpu_compute_stops_the_compaction_reporting_its_s
         BackgroundGpuComputeFailed::GpuFailed {
             device: device.clone(),
             cause,
-            pci_state_when_judged_unusable: singlefs_core::background_compute::PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+            pci_state_when_judged_unusable: Box::new(
+                singlefs_core::background_compute::PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
+            ),
         }
     };
     let every_kind_of_failure = [
diff --git a/crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs b/crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs
index 4c98dce..7d29eb1 100644
--- a/crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs
+++ b/crates/singlefs-verification/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs
@@ -383,7 +383,6 @@ fn alarm_of_the_stop(
     AlarmOfABackgroundTaskStoppedAfterAComputeError {
         task: BackgroundTask::CompactionRelocation,
         gpu_devices_named_by_the_error: vec![card_named("card-eight")],
-        pci_state_of_the_gpu_judged_unusable: singlefs_core::background_compute::PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
         error_that_stopped_the_task: error.clone(),
         wakes_while_stopped,
         batches_compensated_on_the_cpu,
diff --git a/crates/singlefs-verification/tests/compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs b/crates/singlefs-verification/tests/compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs
index 0328545..6defd7c 100644
--- a/crates/singlefs-verification/tests/compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs
+++ b/crates/singlefs-verification/tests/compaction_stays_stopped_from_the_next_entry_after_a_switch_to_no_usable_gpu.rs
@@ -218,7 +218,6 @@ fn the_first_wake_after_a_mount_with_no_usable_gpu_returns_the_alarm() {
         Some(AlarmOfABackgroundTaskStoppedAfterAComputeError {
             task: BackgroundTask::CompactionRelocation,
             gpu_devices_named_by_the_error: vec![card_named("card-eight")],
-            pci_state_of_the_gpu_judged_unusable: singlefs_core::background_compute::PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError,
             error_that_stopped_the_task:
                 BackgroundComputeError::GpuUnusableUnderTheSettingsHandedIn {
                     failure: every_participating_gpu_left_out(),
```
