# m4-segment-code-r3：修复批三对被判产品代码的改动全文（2026-10-16）

相对第二轮冻结副本（`~/.local/share/singlefs/work/m4-segment-code-r2/repo/`）的 `diff -u`；测试代码、模型、池级 checker 与 checker 档的改动不在这里，见冻结副本。

````diff
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -14,7 +14,7 @@
 use crate::address::{DeviceIdentity, SlotNumber};
 use crate::allocator::{AllocationRecord, Placement, UnitAreaStart};
 use crate::logged_operations::{CompactionIntent, CompactionRegion};
-use crate::region_scan::{RegionScan, RegionScanRefusal};
+use crate::region_scan::{AllocationInRegion, RegionScan, RegionScanRefusal, ScannedAllocation};
 
 /// 选段判据。封闭集合：加一种判据就加一个成员（判决 G2 的 S-b、S-c 由 E10 定要不要加）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -241,6 +241,59 @@
     }
 }
 
+/// 下一批要搬的，跳过这次发布本来就重写的节点（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H4，用户 2026-10-16
+/// 定「规划时补足」）：`rewritten_anyway` 是会话在这一次发布上试算出的、计划过而发布路径不另搬的映射 key（它们照样离开 R，只是不算搬）。
+/// 扫描结果里去掉这几把 key 的可搬项，剩下的照 [`plan_batch`] 取（按槽升序取前 `batch` 个，补上 R 里真要搬的单元）；剩下的一项都没有时
+/// 分两种：R 里本来就没有未释放的记录交 [`PlannedBatch::RegionEmptied`]；R 里只剩这次发布本来就重写的节点（每条未释放记录都是可搬单元、
+/// 它们的 key 都在 `rewritten_anyway` 里）交 [`PlannedBatch::Relocate`] 带上它们全部——计划罩住 R 里每一条记录，调用方照 F2 同一次发布
+/// 删意图，它们不受批大小限，因为本来就重写的节点不是搬迁。剩下的只有钉住、隔离、已死、不一致的交 [`PlannedBatch::NothingMovableButTheRegionIsNotEmpty`]。
+/// `rewritten_anyway` 是空的时就是 [`plan_batch`]。
+///
+/// # Panics
+/// 同 [`plan_batch`]：去掉这几把 key 的可搬项之后，能搬的映射 key 有相同的（输入只来自 `scan_region` 交回的 `Ok`，走不到）。
+#[must_use]
+pub fn plan_batch_skipping_the_units_this_publish_rewrites_anyway(
+    region_scan: &RegionScan,
+    batch: CompactionBatchUnits,
+    rewritten_anyway: &BTreeSet<Vec<u8>>,
+) -> PlannedBatch {
+    let is_rewritten_anyway = |allocation: &ScannedAllocation| match &allocation.class {
+        AllocationInRegion::Movable(unit) => rewritten_anyway.contains(&unit.mapping_key),
+        AllocationInRegion::Pinned
+        | AllocationInRegion::Quarantined
+        | AllocationInRegion::Dead
+        | AllocationInRegion::Inconsistent(_) => false,
+    };
+    let scan_without_the_units_rewritten_anyway = RegionScan {
+        region: region_scan.region,
+        allocations: region_scan
+            .allocations
+            .iter()
+            .filter(|allocation| !is_rewritten_anyway(allocation))
+            .cloned()
+            .collect(),
+    };
+    match plan_batch(&scan_without_the_units_rewritten_anyway, batch) {
+        PlannedBatch::Relocate(relocations) => PlannedBatch::Relocate(relocations),
+        PlannedBatch::RegionEmptied if region_scan.allocations.is_empty() => {
+            PlannedBatch::RegionEmptied
+        }
+        // 去掉的那几项之外一条记录都没有：R 里只剩这次发布本来就重写的节点。
+        PlannedBatch::RegionEmptied => PlannedBatch::Relocate(
+            region_scan
+                .movable_units()
+                .map(|(source, unit)| PlannedRelocation {
+                    mapping_key: unit.mapping_key.clone(),
+                    source,
+                })
+                .collect(),
+        ),
+        PlannedBatch::NothingMovableButTheRegionIsNotEmpty => {
+            PlannedBatch::NothingMovableButTheRegionIsNotEmpty
+        }
+    }
+}
+
 /// 一批至多搬几个单元从哪来。封闭集合。产品路径的批大小条款没写（判决 G2 的 K 归 E10（老化：结构抗老化 vs 事后整理））：
 /// 产品路径照 [`CompactionBatchSize::NotDecidedByTheClauses`]，整理一次运行在签任何意图、发任何一个写之前停下，停机原因
 /// [`CompactionRunStop::BatchSizeNotDecidedByTheClauses`]；只供测试的开关给出一个数才走得进去。
@@ -497,23 +550,6 @@
         });
     }
 
-    /// 签发那次发布照记账换了 `segs0`（`transaction::publish_compaction_step`，判决 `research/prompts/m4-segment-code-r1-main-verification.md`
-    /// 第三节 F3：`segs0` 取那次发布写下的记账第 11 项）：这里记着的活意图是调用方交进去的 `handed_in` 那一条时，换成盘上写下的 `written`，
-    /// R 与排不排他不变；记着别的（或没有）就不动。
-    pub fn take_the_intent_written_by_the_issue(
-        &mut self,
-        handed_in: CompactionIntent,
-        written: CompactionIntent,
-    ) {
-        if let Some(live) = self
-            .live_intent
-            .as_mut()
-            .filter(|live| live.intent == handed_in)
-        {
-            live.intent = written;
-        }
-    }
-
     /// 意图删了（达成或丢掉，删它的那次发布已落盘），或签发那次发布没发成：撤掉活意图连同 R 排他，交回撤掉的那一条。
     pub fn withdraw_the_live_intent(&mut self) -> Option<CompactionIntent> {
         self.live_intent.take().map(|live| live.intent)
@@ -806,8 +842,9 @@
     };
 
     use super::{
-        choose_region, plan_batch, CompactionBatchUnits, PlannedBatch, PlannedRelocation,
-        RegionChoice, RegionChoiceCriterion, SegmentsRefusedByTheRegionScan, UnitAreaOfOneDevice,
+        choose_region, plan_batch, plan_batch_skipping_the_units_this_publish_rewrites_anyway,
+        CompactionBatchUnits, PlannedBatch, PlannedRelocation, RegionChoice, RegionChoiceCriterion,
+        SegmentsRefusedByTheRegionScan, UnitAreaOfOneDevice,
     };
     use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
     use crate::allocator::{AllocationRecord, Placement, UnitAreaStart};
@@ -1265,4 +1302,115 @@
             CompactionBatchUnits::new(2).expect("K 从 1 起"),
         );
     }
+
+    /// 扫描结果里第 `position` 项（照 `region_scan_of` 的形状）的计划项：映射 key 与源落点。
+    fn planned_relocation_at(region_scan: &RegionScan, position: usize) -> PlannedRelocation {
+        let allocation = &region_scan.allocations[position];
+        let AllocationInRegion::Movable(unit) = &allocation.class else {
+            panic!("第 {position} 项照形状是能搬的");
+        };
+        PlannedRelocation {
+            mapping_key: unit.mapping_key.clone(),
+            source: allocation.placement,
+        }
+    }
+
+    /// 跳过的集合是空的时，跳过本来就重写的节点那一份计划与 `plan_batch` 逐项相同（判决 m4-segment-code-r2 第三节 H4：没有试算出本来就重写的
+    /// 单元时，计划照证过的核取）。
+    #[test]
+    fn plan_batch_skipping_no_unit_agrees_with_plan_batch_on_every_small_region_scan() {
+        let mut region_scans_compared: u64 = 0;
+        for allocations in 0..=MOST_ALLOCATIONS_IN_A_REGION {
+            for shape_indices in every_sequence(ALLOCATION_SHAPES.len(), allocations) {
+                let shapes: Vec<AllocationShape> = shape_indices
+                    .iter()
+                    .map(|shape_index| ALLOCATION_SHAPES[*shape_index])
+                    .collect();
+                let region_scan = region_scan_of(&shapes);
+                for batch_units in 1..=LARGEST_BATCH_UNITS {
+                    let batch = CompactionBatchUnits::new(batch_units).expect("K 从 1 起");
+                    assert_eq!(
+                        plan_batch_skipping_the_units_this_publish_rewrites_anyway(
+                            &region_scan,
+                            batch,
+                            &BTreeSet::new(),
+                        ),
+                        plan_batch(&region_scan, batch),
+                        "扫描结果的形状 {shapes:?}、K = {batch_units}：一把都不跳过时与 plan_batch 不同"
+                    );
+                    region_scans_compared += 1;
+                }
+            }
+        }
+        assert!(region_scans_compared > 0, "比过的形状不为 0");
+    }
+
+    /// 打头那一个可搬单元是这次发布本来就重写的节点：计划跳过它，补上后面真要搬的单元，补满批大小（判决 m4-segment-code-r2 第三节 H4「补上 R 里
+    /// 真要搬的单元」）。
+    #[test]
+    fn plan_batch_skipping_a_unit_rewritten_anyway_fills_the_batch_with_the_next_movable_units() {
+        let region_scan = region_scan_of(&[
+            AllocationShape::Movable,
+            AllocationShape::Movable,
+            AllocationShape::Movable,
+        ]);
+        let rewritten_anyway: BTreeSet<Vec<u8>> =
+            [planned_relocation_at(&region_scan, 0).mapping_key]
+                .into_iter()
+                .collect();
+        assert_eq!(
+            plan_batch_skipping_the_units_this_publish_rewrites_anyway(
+                &region_scan,
+                CompactionBatchUnits::new(2).expect("K 从 1 起"),
+                &rewritten_anyway,
+            ),
+            PlannedBatch::Relocate(vec![
+                planned_relocation_at(&region_scan, 1),
+                planned_relocation_at(&region_scan, 2),
+            ]),
+            "跳过打头那一个，K = 2 补上后面两个"
+        );
+    }
+
+    /// R 里每一条记录都是可搬单元、它们都是这次发布本来就重写的节点：计划带上它们全部、罩住 R 里每一条记录（调用方照 F2 同一次发布删意图），
+    /// 不受批大小限——本来就重写的节点不是搬迁。
+    #[test]
+    fn plan_batch_skipping_every_unit_plans_them_all_when_only_units_rewritten_anyway_are_left() {
+        let region_scan = region_scan_of(&[AllocationShape::Movable, AllocationShape::Movable]);
+        let rewritten_anyway: BTreeSet<Vec<u8>> = [0, 1]
+            .into_iter()
+            .map(|position| planned_relocation_at(&region_scan, position).mapping_key)
+            .collect();
+        assert_eq!(
+            plan_batch_skipping_the_units_this_publish_rewrites_anyway(
+                &region_scan,
+                CompactionBatchUnits::new(1).expect("K 从 1 起"),
+                &rewritten_anyway,
+            ),
+            PlannedBatch::Relocate(vec![
+                planned_relocation_at(&region_scan, 0),
+                planned_relocation_at(&region_scan, 1),
+            ]),
+            "R 里只剩本来就重写的节点：K = 1 也带上两个，计划罩住 R 里每一条记录"
+        );
+    }
+
+    /// 能搬的都是这次发布本来就重写的节点，R 里另有一条钉住的记录：再推下去也空不了，交「搬不动」（调用方丢意图）。
+    #[test]
+    fn plan_batch_skipping_every_movable_unit_with_a_pinned_record_left_has_nothing_movable() {
+        let region_scan = region_scan_of(&[AllocationShape::Movable, AllocationShape::Pinned]);
+        let rewritten_anyway: BTreeSet<Vec<u8>> =
+            [planned_relocation_at(&region_scan, 0).mapping_key]
+                .into_iter()
+                .collect();
+        assert_eq!(
+            plan_batch_skipping_the_units_this_publish_rewrites_anyway(
+                &region_scan,
+                CompactionBatchUnits::new(1).expect("K 从 1 起"),
+                &rewritten_anyway,
+            ),
+            PlannedBatch::NothingMovableButTheRegionIsNotEmpty,
+            "能搬的都跳过、还剩一条钉住的记录"
+        );
+    }
 }
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -25,19 +25,20 @@
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration,
     SlotNumber, TreeIdentifier,
 };
-use crate::allocator::{PlacementOnDevice, PoolAllocator};
+use crate::allocator::{AllocationRecord, PlacementOnDevice, PoolAllocator};
 use crate::block_device::{
     check_aligned_and_in_range, BlockDevice, BlockDeviceError, DiscardOutcome,
     PhysicalBlockSizeInBytes, WriteDurability,
 };
 use crate::compaction_plan::{
     choose_region, compaction_starts_after_a_user_change, empty_cluster_segments_by_the_records,
-    holes_admit_the_issue, holes_outside_the_region_in_segments_holding_user_data, plan_batch,
-    publish_budget_of_a_compaction_run, region_holds_a_quarantined_slot,
-    region_is_empty_by_the_records, region_scans_of_the_segments_without_an_unreleased_record,
-    the_completed_intent_yielded, CompactionBatchSize, CompactionBatchUnits, CompactionOfThisMount,
-    CompactionRunForcing, CompactionRunStop, PlannedBatch, PlannedRelocation, RegionChoice,
-    RegionChoiceCriterion, UnitAreaOfOneDevice,
+    holes_admit_the_issue, holes_outside_the_region_in_segments_holding_user_data,
+    plan_batch_skipping_the_units_this_publish_rewrites_anyway, publish_budget_of_a_compaction_run,
+    region_holds_a_quarantined_slot, region_is_empty_by_the_records,
+    region_scans_of_the_segments_without_an_unreleased_record, the_completed_intent_yielded,
+    CompactionBatchSize, CompactionBatchUnits, CompactionOfThisMount, CompactionRunForcing,
+    CompactionRunStop, PlannedBatch, PlannedRelocation, RegionChoice, RegionChoiceCriterion,
+    UnitAreaOfOneDevice,
 };
 use crate::instance_table::InstanceRow;
 use crate::instance_table_sweep::unit_head_read_by_sweep;
@@ -61,13 +62,15 @@
 use crate::root_ring::{target_for_publish, RootRingSlot};
 use crate::transaction::{
     central_mapping_locations_in_the_version, prepare_the_version_publish,
+    publish_compaction_step_in_the_session,
     publish_compaction_step_with_the_region_check_of_the_intent_deletion, publish_create_snapshot,
     publish_new_inodes, publish_overwrite, publish_sequential_write,
     publish_snapshot_destruction_intent, publish_unlink_intent, publish_version,
-    walk_the_deletion_queue_until_it_is_empty, CompactionIntentStep,
-    DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile, MappingLookup, PoolVersion,
-    PoolWriter, PublishError, RegionCheckOfTheIntentDeletion, ReleaseChecksumCheck,
-    RelocationBatchRefused, TransactionOutput, TransactionUnit,
+    walk_the_deletion_queue_until_it_is_empty, CompactionIntentStep, CompactionStepRefused,
+    DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile, MappingLookup,
+    PlannedUnitsRewrittenAnyway, PoolVersion, PoolWriter, PublishError,
+    RegionCheckOfTheIntentDeletion, ReleaseChecksumCheck, RelocationBatchRefused,
+    TransactionOutput, TransactionUnit,
 };
 use crate::unit::{unit_filesystem_identifier, UNIT_CLASS_DATA};
 use crate::unlink::DeletionSwitches;
@@ -2275,17 +2278,20 @@
 /// 分配器这一侧的 R 排他与它一致（会话签发、删意图时装上与撤掉，挂载按 logged_ops 装上）。先把发布路径隔离过的槽并进常驻隔离集。
 /// 1. 没有活意图：可写挂载续做停在「没有活意图可续」（只续做、不签新的）；上一条完成的意图全局差值为假就停（攻方 K-h）；选段（强制选的那一段，
 ///    或 greedy 先扫后选），选不出就停；签发前判 R 外非全空段的空洞 ≥ R 里要搬的单元数（攻方 K-g），不够就停；签发：先在分配器上排他 R
-///    （签发那次发布自己的新落点也不进 R），发一次只把意图写进 logged_ops 的发布（`c0` = 现行那一版的 txg，`segs0` 先交 R 那块盘按记录口径的全空段数，签发那次发布照它自己写下的记账第 11 项换，
-///    判决 m4-segment-code-r1 第三节 F3）；
+///    （签发那次发布自己的新落点也不进 R），发一次只把意图写进 logged_ops 的发布（`c0` = 现行那一版的 txg，`segs0` = 签发那次发布之前、
+///    现行那一版 R 那块盘按记录口径的全空段数，判决 m4-segment-code-r2 第三节 H3，用户 2026-10-16 定「取签发之前」）；
 ///    被空间拒就撤排他、停（意图没签成）。
 /// 2. 有活意图：暂停过的重新排他；R 里出现隔离槽（直接查常驻隔离集，钉住与隔离同时成立也算，丁 1 交判第 4 点）、区域扫描拒了 R、扫出隔离类，
-///    都发一次只删意图的发布丢掉它（攻方 K-c）；对 R 重扫（每批之前都重扫，丁 1 交判第 3 点）、计划一批：要搬就发——这一批搬完 R 里每一条未释放的记录时
-///    同一次发布删意图（D8（核心索引结构） 已定项 5；发布在算好的账上核 R 按记录全空，核不过照这一批不删意图再发）；别的批照抄意图、在算好的账上核 R 还有未释放的记录，
-///    这一批腾空了 R（计划里的单元都是这次发布本来就重写的节点）就照同一批删意图再发（判决 m4-segment-code-r1 第三节 F2），一个都没搬的不记成一批；
+///    都发一次只删意图的发布丢掉它（攻方 K-c）；对 R 重扫（每批之前都重扫，丁 1 交判第 3 点）、计划一批（跳过这次发布本来就重写的节点、补上 R 里
+///    真要搬的单元，判决 m4-segment-code-r2 第三节 H4）：要搬就发——这一批罩住 R 里每一条未释放的记录时同一次发布删意图（D8（核心索引结构）
+///    已定项 5；R 里只剩这次发布本来就重写的节点时也是这一支；发布在算好的账上核 R 按记录全空，核不过照这一批不删意图再发）；别的批照抄意图、
+///    在算好的账上核 R 还有未释放的记录，这一批腾空了 R（计划外的、这次发布本来就重写的节点也离开 R）就照同一批删意图再发（判决 m4-segment-code-r1
+///    第三节 F2），R 还不空而计划里有本来就重写、不另搬的单元就跳过它们再计划，所以照抄意图的一批至少真搬一个；一个都没搬的不记成一批；
 ///    R 已按记录全空就发一次只删意图的发布；
 ///    一个能搬的都没有而 R 不空就发一次只删意图的发布丢掉它、这一趟停。整理的发布被空间拒就暂停（撤 R 排他、意图留在盘上，攻方 K-k）、停。
 ///
-/// 一次调用至多算三次账、发一次（照抄那一支核出腾空了 R 换删意图再算，删意图那一支核不过换照抄再算），选段那一圈的上界是单元区的整段数。
+/// 一次调用至多算 R 里能搬的单元数加三次账、发一次（每跳过一次本来就重写的单元再算一次；照抄那一支核出腾空了 R 换删意图再算，删意图那一支核不过
+/// 换照抄再算），选段那一圈的上界是单元区的整段数。
 #[allow(
     clippy::too_many_arguments,
     reason = "参数各是一样东西：参数、盘表、分配器、会话这一侧的整理状态、现行那一版、实例代号、批大小、推到哪为止、上一条完成的意图、一批腾空了 R 之后删意图那一支的核怎么判"
@@ -2513,11 +2519,92 @@
             }
         }
     };
-    // 这一次发布要搬哪几个、对意图做什么、发成了是哪一种。
-    let (planned, intent_step, kind) = match scan
-        .as_ref()
-        .map(|scan| (scan, plan_batch(scan, batch_units)))
-    {
+    // 这一次发布要搬哪几个、对意图做什么、发成了是哪一种：计划跳过这次发布本来就重写的节点、补上 R 里真要搬的单元（判决
+    // `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H4，用户 2026-10-16 定「规划时补足」）。照抄意图的一批在算好的账上
+    // 核计划里每个单元都真搬，核出本来就重写的那几把 key 就并进跳过的集合、再计划一次（[`compaction_publish_planned_for_the_region_scan`]）。
+    // 迭代上界是 R 里能搬的单元数加一：被拒时点名的 key 都在这一次的计划里，计划不取已跳过的，所以每被拒一次跳过的集合至少多一把 key；
+    // 能搬的都跳过之后计划要么罩住 R 里每一条记录（删意图那一支，不再核这一样），要么没有可搬的。跨轮携带的只有跳过的集合。
+    let mut units_this_publish_rewrites_anyway: BTreeSet<Vec<u8>> = BTreeSet::new();
+    let (published, kind) = loop {
+        let (planned, intent_step, kind) = compaction_publish_planned_for_the_region_scan(
+            scan.as_ref(),
+            batch_units,
+            &units_this_publish_rewrites_anyway,
+            &version.allocation_records,
+            region,
+        );
+        match publish_the_planned_compaction_step_falling_back_on_the_region_check(
+            parameters,
+            devices,
+            allocator,
+            &version,
+            &planned,
+            intent_step,
+            kind,
+            region_check_of_the_intent_deletion_after_a_batch_emptying_the_region,
+            instance,
+        ) {
+            Ok(published_and_kind) => break published_and_kind,
+            Err(CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish {
+                mapping_keys,
+            }) => {
+                let skipped_before = units_this_publish_rewrites_anyway.len();
+                units_this_publish_rewrites_anyway.extend(mapping_keys);
+                assert!(
+                    units_this_publish_rewrites_anyway.len() > skipped_before,
+                    "被拒时点名的 key 都在这一次的计划里，计划不取已跳过的：跳过的集合每被拒一次至少多一把 key"
+                );
+            }
+            Err(CompactionStepRefused::Refused(refused)) => {
+                return pause_or_hand_up(allocator, refused)
+            }
+        }
+    };
+    match kind {
+        CompactionPublishKind::BatchCompletingTheIntent
+        | CompactionPublishKind::IntentCompletedWithoutABatch => {
+            allocator.compaction_mut().withdraw_the_live_intent();
+            compaction.observations.intents_completed += 1;
+            *last_completed_intent = Some(intent);
+        }
+        CompactionPublishKind::IntentDropped
+        | CompactionPublishKind::IntentDroppedWithNothingMovable => {
+            allocator.compaction_mut().withdraw_the_live_intent();
+            compaction.observations.intents_dropped += 1;
+        }
+        CompactionPublishKind::Batch | CompactionPublishKind::IntentIssued => {}
+    }
+    *current = PoolVersion::of_a_version_of_trees(published.clone());
+    Ok(CompactionStep::Published {
+        version: Box::new(published),
+        kind,
+    })
+}
+
+/// 有活意图时这一次整理发布的计划：要搬哪几个、对意图做什么、发成了是哪一种（[`advance_compaction_by_one_publish`]）。`scan` 是对 R 重扫的结果
+/// （R 里有隔离槽、区域扫描拒了 R、扫出隔离类时是 `None`），`rewritten_anyway` 是这一次发布上已试算出的、本来就重写而不另搬的映射 key
+/// （[`plan_batch_skipping_the_units_this_publish_rewrites_anyway`]）。
+fn compaction_publish_planned_for_the_region_scan(
+    scan: Option<&RegionScan>,
+    batch_units: CompactionBatchUnits,
+    rewritten_anyway: &BTreeSet<Vec<u8>>,
+    allocation_records_of_the_version: &[AllocationRecord],
+    region: CompactionRegion,
+) -> (
+    Vec<PlannedRelocation>,
+    CompactionIntentStep,
+    CompactionPublishKind,
+) {
+    match scan.map(|scan| {
+        (
+            scan,
+            plan_batch_skipping_the_units_this_publish_rewrites_anyway(
+                scan,
+                batch_units,
+                rewritten_anyway,
+            ),
+        )
+    }) {
         // R 里有隔离槽、区域扫描拒了 R、扫出隔离类：丢意图（攻方 K-c）。
         None => (
             Vec::new(),
@@ -2525,7 +2612,8 @@
             CompactionPublishKind::IntentDropped,
         ),
         Some((scan, PlannedBatch::Relocate(planned))) => {
-            // 这一批搬完 R 里每一条未释放的记录：最后一批，同一次发布删意图（D8（核心索引结构） 已定项 5）。
+            // 这一批罩住 R 里每一条未释放的记录（搬走，或 R 里只剩这次发布本来就重写的节点）：最后一批，同一次发布删意图
+            // （D8（核心索引结构） 已定项 5；判决 m4-segment-code-r1 第三节 F2）。
             if planned.len() == scan.allocations.len() {
                 (
                     planned,
@@ -2533,8 +2621,8 @@
                     CompactionPublishKind::BatchCompletingTheIntent,
                 )
             } else {
-                // 计划里的单元都是这次发布本来就重写的节点时，这一批搬不了也腾空 R：照抄意图那一支在算好的账上核 R，
-                // 全空就照同一批删意图再发（判决 m4-segment-code-r1 第三节 F2）。
+                // 照抄意图那一支在算好的账上核两样：R 被这一批腾空了（计划外的、这次发布本来就重写的节点也离开 R）就照同一批删意图再发
+                // （判决 m4-segment-code-r1 第三节 F2）；计划里有本来就重写、不另搬的就跳过它们再计划（判决 m4-segment-code-r2 第三节 H4）。
                 (
                     planned,
                     CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion,
@@ -2544,7 +2632,7 @@
         }
         Some((_scan, PlannedBatch::RegionEmptied)) => {
             assert!(
-                region_is_empty_by_the_records(&version.allocation_records, region),
+                region_is_empty_by_the_records(allocation_records_of_the_version, region),
                 "扫描里一条未释放记录都没有，R 就按记录全空：扫描与停机谓词读的是同一份记录"
             );
             (
@@ -2558,52 +2646,86 @@
             CompactionIntentStep::Drop,
             CompactionPublishKind::IntentDroppedWithNothingMovable,
         ),
-    };
-    let first_try = publish_the_compaction_step_once(
-        parameters,
-        devices,
+    }
+}
+
+/// 发这一次计划好的整理发布，删意图那一支或照抄那一支在算好的账上核不过时换个对意图的做法再算（[`advance_compaction_by_one_publish`]）。
+/// 照抄意图那一支经 [`publish_compaction_step_in_the_session`] 发，计划里有这次发布本来就重写的单元就不发、交回
+/// [`CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish`]（判决 m4-segment-code-r2 第三节 H4）。发成交回这一版与发成的是哪一种；
+/// 别的被拒原样交回。一次调用至多算三次账、发一次。
+///
+/// 两条「核不过就照抄意图」的退路在产品路径上走得到（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H2）：
+/// 计划里的单元在这次发布里都离开 R（搬走，或本来就重写），删意图那一核不过只有一种来历——释放之前读盘核把一份隔离、那条记录留在已分配，
+/// 也就是一份副本读与重读都读不出（瞬时读错）；下一遍算账再读时它又读得出，照抄意图那一版里 R 就按记录全空了，下一次发布照「R 已按记录全空」
+/// 只删意图。另外两处走得到：只供测试的开关关掉 R 排他、这次发布的新落点落进 R；只供测试的 `region_check` 强制删意图那一核不过。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：参数、盘表、分配器、现行那一版、计划、对意图做什么、发成了是哪一种、一批腾空了 R 之后删意图那一支的核怎么判、实例代号"
+)]
+fn publish_the_planned_compaction_step_falling_back_on_the_region_check<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut [(DeviceIdentity, Device)],
+    allocator: &mut PoolAllocator,
+    version: &TransactionOutput,
+    planned: &[PlannedRelocation],
+    intent_step: CompactionIntentStep,
+    kind: CompactionPublishKind,
+    region_check_of_the_intent_deletion_after_a_batch_emptying_the_region: RegionCheckOfTheIntentDeletion,
+    instance: InstanceGeneration,
+) -> Result<(TransactionOutput, CompactionPublishKind), CompactionStepRefused> {
+    let switches = allocator.deletion_switches();
+    let mut writer = PoolWriter::new(parameters, devices);
+    writer.set_container_tree_capacities(switches.container_tree_capacities());
+    let first_try = publish_compaction_step_in_the_session(
+        &mut writer,
         allocator,
-        &version,
-        &planned,
+        version,
+        planned,
         intent_step,
+        RegionCheckOfTheIntentDeletion::JudgedByTheRecords,
+        PlannedUnitsRewrittenAnyway::RefusedSoThatTheSessionPlansAgain,
         instance,
     );
-    let (published, kind) = match first_try {
-        Ok(published) => (published, kind),
-        // 最后一批在算好的账上 R 还不按记录全空（只供测试的开关关掉排他、这一批的新落点落进了 R）：这一批不删意图，照搬。
-        Err(RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
-            ..
-        }) if kind == CompactionPublishKind::BatchCompletingTheIntent => {
+    match first_try {
+        Ok(published) => Ok((published, kind)),
+        // 最后一批在算好的账上 R 还不按记录全空（一份副本读与重读都读不出而被隔离，那条记录留在已分配；或只供测试的开关关掉排他、
+        // 这一批的新落点落进了 R）：这一批不删意图，照搬、照抄意图。
+        Err(CompactionStepRefused::Refused(
+            RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
+                ..
+            },
+        )) if kind == CompactionPublishKind::BatchCompletingTheIntent => {
             match publish_the_compaction_step_once(
                 parameters,
                 devices,
                 allocator,
-                &version,
-                &planned,
+                version,
+                planned,
                 CompactionIntentStep::CarryTheIntent,
                 instance,
             ) {
-                Ok(published) => (published, CompactionPublishKind::Batch),
-                Err(refused) => return pause_or_hand_up(allocator, refused),
+                Ok(published) => Ok((published, CompactionPublishKind::Batch)),
+                Err(refused) => Err(refused.into()),
             }
         }
-        // 这一批在算好的账上腾空了 R（计划里的单元都是这次发布本来就重写的节点）：同一批删意图再发（判决 m4-segment-code-r1 第三节 F2）。
-        // 删意图那一支再核 R 按记录全空：两次只差 logged_ops 那一条，取落点不读它，核不过只在只供测试的开关关掉排他时走得到，照这一批不删意图再发
-        // （用例经只供测试的 `RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests` 强制进入）。
-        Err(RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried {
-            ..
-        }) if kind == CompactionPublishKind::Batch => {
+        // 这一批在算好的账上腾空了 R（计划外的、这次发布本来就重写的节点也离开 R）：同一批删意图再发（判决 m4-segment-code-r1 第三节 F2）。
+        // 删意图那一支再核 R 按记录全空：两次只差 logged_ops 那一条，取落点不读它，核不过的来历同上一支（瞬时读错、关掉排他），
+        // 照这一批不删意图再发（用例经只供测试的 `RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests`
+        // 或故障注入的瞬时读错强制进入）。
+        Err(CompactionStepRefused::Refused(
+            RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried { .. },
+        )) if kind == CompactionPublishKind::Batch => {
             match publish_the_compaction_step_once_with_the_region_check_of_the_intent_deletion(
                 parameters,
                 devices,
                 allocator,
-                &version,
-                &planned,
+                version,
+                planned,
                 CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
                 region_check_of_the_intent_deletion_after_a_batch_emptying_the_region,
                 instance,
             ) {
-                Ok(published) => (published, CompactionPublishKind::BatchCompletingTheIntent),
+                Ok(published) => Ok((published, CompactionPublishKind::BatchCompletingTheIntent)),
                 Err(
                     RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
                         ..
@@ -2612,38 +2734,19 @@
                     parameters,
                     devices,
                     allocator,
-                    &version,
-                    &planned,
+                    version,
+                    planned,
                     CompactionIntentStep::CarryTheIntent,
                     instance,
                 ) {
-                    Ok(published) => (published, CompactionPublishKind::Batch),
-                    Err(refused) => return pause_or_hand_up(allocator, refused),
+                    Ok(published) => Ok((published, CompactionPublishKind::Batch)),
+                    Err(refused) => Err(refused.into()),
                 },
-                Err(refused) => return pause_or_hand_up(allocator, refused),
+                Err(refused) => Err(refused.into()),
             }
         }
-        Err(refused) => return pause_or_hand_up(allocator, refused),
-    };
-    match kind {
-        CompactionPublishKind::BatchCompletingTheIntent
-        | CompactionPublishKind::IntentCompletedWithoutABatch => {
-            allocator.compaction_mut().withdraw_the_live_intent();
-            compaction.observations.intents_completed += 1;
-            *last_completed_intent = Some(intent);
-        }
-        CompactionPublishKind::IntentDropped
-        | CompactionPublishKind::IntentDroppedWithNothingMovable => {
-            allocator.compaction_mut().withdraw_the_live_intent();
-            compaction.observations.intents_dropped += 1;
-        }
-        CompactionPublishKind::Batch | CompactionPublishKind::IntentIssued => {}
+        Err(refused) => Err(refused),
     }
-    *current = PoolVersion::of_a_version_of_trees(published.clone());
-    Ok(CompactionStep::Published {
-        version: Box::new(published),
-        kind,
-    })
 }
 
 /// 整理的一次发布没发成（活意图在盘上）：空间不够就暂停——撤 R 排他、意图留着（攻方 K-k），不推抬 F（判决 G8 g1）；别的原样交回。
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -7473,8 +7473,10 @@
     /// 删意图时上一版没有活整理意图。
     NoLiveCompactionIntent,
     /// 意图达成那一支（[`CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords`]）在这次发布算好的账上 R 里还有未释放的记录：
-    /// 停机谓词「删意图那一刻 R 按记录全空」（判决 G4）不成立，不删。只在 R 不排他（只供测试的开关关掉排他）、这次发布的新落点落进 R 时走得到，
-    /// 或由只供测试的 [`RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests`] 强制；
+    /// 停机谓词「删意图那一刻 R 按记录全空」（判决 G4）不成立，不删。产品路径上瞬时读错走得到（判决 `research/prompts/m4-segment-code-r2-main-verification.md`
+    /// 第三节 H2）：计划里的每个单元在这次发布里都离开 R（搬走，或本来就重写），只有释放之前读盘核把一份隔离时那条记录才留在已分配——
+    /// 一份副本读与重读都读不出、下一次又读得出，这一遍就核不过而下一遍核得过。另外两处走得到：R 不排他（只供测试的开关关掉排他）、
+    /// 这次发布的新落点落进 R；只供测试的 [`RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests`] 强制。
     /// 调用方照这一批不删意图再发一次（意图留着，下一批接着搬）。
     RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
         region: CompactionRegion,
@@ -7536,7 +7538,9 @@
     CarryTheIntent,
     /// 搬一批、logged_ops 照抄，这一批发布之后 R 按记录全空就不发：交回 `RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried`，调用方照同一批
     /// 删意图再发一次（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F2，G4「目标态到达与最后一批同发布删」：
-    /// 计划里的单元都是这次发布本来就重写的节点时，这一批一个都不搬也腾空了 R）。
+    /// 计划里的单元是这次发布本来就重写的节点时，它们不另搬也离开 R）。会话经 `publish_compaction_step_in_the_session` 另核一样：
+    /// R 还不空而计划里有这次发布本来就重写、不另搬的单元就不发（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H4，
+    /// `PlannedUnitsRewrittenAnyway`）。
     CarryTheIntentUnlessThisPublishEmptiesTheRegion,
     /// 签发：logged_ops 追加这一条意图（签发 txg = 这次发布的 txg）。上一版不许已有活整理意图（全池至多一条，判决 G1 e1）。
     Issue(CompactionIntent),
@@ -7565,10 +7569,10 @@
 /// （判决 G1：意图条目写进 logged_ops 树，签发那次发布写、最后一批同发布删；D8（核心索引结构） 已定项 5 三态表：签发之前崩 = 没有意图，
 /// 签发之后、最后一批之前崩 = 意图还在、挂载续做，最后一批之后 = 意图没了且目标态达成）。别的同 [`publish_relocation_batch`]。
 ///
-/// 签发那次写下的 `segs0` 照判决 G1 字段表的定义句取：这次发布写下的记账第 11 项里 R 所在盘的值（判决
-/// `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F3）。调用方交进来的那条意图里的 `segs0` 只是先算的一个数：这次发布
-/// 自己开了新聚簇段时它比记账第 11 项多一。算好账之后两者不同就照记账那个数换掉、再算一次再发（两次只差 logged_ops 条目里那 8 个字节，
-/// 落点与记录不随它变）；分配器上记着交进来那一条活意图的，换成盘上写下的那一条（R 与排不排他不变）。
+/// 签发那次写下的 `segs0` 就是调用方交进来的那条意图里的：签发那次发布之前的全空聚簇段数（判决
+/// `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H3，用户 2026-10-16 定「取签发之前」；会话按上一版的分配记录数，
+/// `crate::compaction_plan::empty_cluster_segments_by_the_records`）。签发那次发布自己开了新聚簇段时它比这次写下的记账第 11 项多一，
+/// 停机谓词「全局差值」相对它比，这条意图就要多腾出那一段才算有产出。只算一遍账、照交进来的写，不按记账换。
 ///
 /// # Errors
 /// 计划对不上上一版 ⇒ [`RelocationBatchRefused`] 那四种计划错；签发时上一版已有活意图 ⇒ `CompactionIntentAlreadyLive`；删时、照抄而核 R 时上一版没有活意图 ⇒
@@ -7577,8 +7581,7 @@
 /// 这几种都在任何写之前交回（后两种在算账之后、落盘之前：分配器换回进来时那一份），盘上逐字节不变；其余同 `publish_version`（`Publish`）。
 ///
 /// # Panics
-/// `planned` 是空的而 `intent_step` 是两种照抄之一：这样的发布什么都不做，调用方不许走到这里。签发那一支照记账换过 `segs0` 再算一次，
-/// 记账第 11 项与头一次不同：两次只差 logged_ops 条目的字节，取落点与记录都不读它。
+/// `planned` 是空的而 `intent_step` 是两种照抄之一：这样的发布什么都不做，调用方不许走到这里。
 pub fn publish_compaction_step<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
@@ -7615,6 +7618,83 @@
     region_check_of_the_intent_deletion: RegionCheckOfTheIntentDeletion,
     instance: InstanceGeneration,
 ) -> Result<TransactionOutput, RelocationBatchRefused> {
+    match publish_compaction_step_in_the_session(
+        pool,
+        allocator,
+        previous,
+        planned,
+        intent_step,
+        region_check_of_the_intent_deletion,
+        PlannedUnitsRewrittenAnyway::LeftToTheCompactionPublish,
+        instance,
+    ) {
+        Ok(published) => Ok(published),
+        Err(CompactionStepRefused::Refused(refused)) => Err(refused),
+        Err(CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish { mapping_keys }) => {
+            unreachable!(
+                "照 PlannedUnitsRewrittenAnyway::LeftToTheCompactionPublish 发，落盘之前不核计划里有没有本来就重写的单元，交不回这一种：{mapping_keys:?}"
+            )
+        }
+    }
+}
+
+/// 照抄意图的一批（[`CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion`]）在 R 还不空时，计划里有这次发布本来就重写、
+/// 不另搬的单元怎么办（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H4，用户 2026-10-16 定「规划时补足」）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub(crate) enum PlannedUnitsRewrittenAnyway {
+    /// 照发：它们照样离开 R，只是不算搬（公开入口 [`publish_compaction_step`] 一族的行为）。
+    LeftToTheCompactionPublish,
+    /// 不发，交回 [`CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish`]：会话跳过它们、补上 R 里别的可搬单元再计划，
+    /// 照抄意图的一批计划里每个单元都真搬，所以至少搬一个（`mounted_session` 推整理的那一步）。
+    RefusedSoThatTheSessionPlansAgain,
+}
+
+/// 会话这一侧发整理的一次发布被拒（[`publish_compaction_step_in_the_session`]）：公开入口的那些拒绝，或计划里有这次发布本来就重写的单元。
+#[derive(Debug)]
+pub(crate) enum CompactionStepRefused {
+    Refused(RelocationBatchRefused),
+    /// 照抄意图的一批在这次发布算好的账上 R 还不空，而计划里这几把 key 的单元是这次发布本来就重写的节点、不另搬（判决 m4-segment-code-r2
+    /// 第三节 H4）。在算账之后、落盘之前交回：分配器换回进来时那一份，盘上逐字节不变。`mapping_keys` 按计划的次序、至少一把。
+    PlannedUnitsRewrittenAnywayByThisPublish {
+        mapping_keys: Vec<Vec<u8>>,
+    },
+}
+
+impl From<PublishError> for CompactionStepRefused {
+    fn from(cause: PublishError) -> Self {
+        CompactionStepRefused::Refused(RelocationBatchRefused::Publish(cause))
+    }
+}
+
+impl From<RelocationBatchRefused> for CompactionStepRefused {
+    fn from(refused: RelocationBatchRefused) -> Self {
+        CompactionStepRefused::Refused(refused)
+    }
+}
+
+/// 同 [`publish_compaction_step_with_the_region_check_of_the_intent_deletion`]，照抄意图的一批计划里有这次发布本来就重写的单元时照
+/// `planned_units_rewritten_anyway` 办（别的 `intent_step` 不读它）。会话推整理时经它发，交 [`PlannedUnitsRewrittenAnyway::RefusedSoThatTheSessionPlansAgain`]。
+///
+/// # Errors
+/// 同 [`publish_compaction_step_with_the_region_check_of_the_intent_deletion`]（包在 [`CompactionStepRefused::Refused`] 里）；照抄而核 R 那一支
+/// 在算好的账上 R 还不空、计划里有本来就重写的单元、又交了 `RefusedSoThatTheSessionPlansAgain` ⇒ `PlannedUnitsRewrittenAnywayByThisPublish`。
+///
+/// # Panics
+/// 同 [`publish_compaction_step`]。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：写入口、分配器、上一版、计划、对意图做什么、删意图那一核怎么判、计划里有本来就重写的单元怎么办、实例代号"
+)]
+pub(crate) fn publish_compaction_step_in_the_session<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    planned: &[PlannedRelocation],
+    intent_step: CompactionIntentStep,
+    region_check_of_the_intent_deletion: RegionCheckOfTheIntentDeletion,
+    planned_units_rewritten_anyway: PlannedUnitsRewrittenAnyway,
+    instance: InstanceGeneration,
+) -> Result<TransactionOutput, CompactionStepRefused> {
     assert!(
         !(planned.is_empty()
             && matches!(
@@ -7631,29 +7711,30 @@
             CompactionStepCheckBeforePersisting::NothingToCheck,
         ),
         (CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion, None) => {
-            return Err(RelocationBatchRefused::NoLiveCompactionIntent)
+            return Err(RelocationBatchRefused::NoLiveCompactionIntent.into())
         }
         (
             CompactionIntentStep::CarryTheIntentUnlessThisPublishEmptiesTheRegion,
             Some((_issued_txg, intent)),
         ) => (
             LoggedOperationsStepOfThePublish::CarryTheLoggedOperations,
-            CompactionStepCheckBeforePersisting::RegionKeepsAnUnreleasedRecord(intent.region),
+            CompactionStepCheckBeforePersisting::RegionKeepsAnUnreleasedRecord {
+                region: intent.region,
+                planned_units_rewritten_anyway,
+            },
         ),
         (CompactionIntentStep::Issue(intent), None) => (
             LoggedOperationsStepOfThePublish::IssueTheCompactionIntent(intent),
-            CompactionStepCheckBeforePersisting::EmptyClusterSegmentsAtIssueAsTheAccountingWritesThem(
-                intent,
-            ),
+            CompactionStepCheckBeforePersisting::NothingToCheck,
         ),
         (CompactionIntentStep::Issue(_), Some((issued_txg, _))) => {
-            return Err(RelocationBatchRefused::CompactionIntentAlreadyLive { issued_txg })
+            return Err(RelocationBatchRefused::CompactionIntentAlreadyLive { issued_txg }.into())
         }
         (
             CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords
             | CompactionIntentStep::Drop,
             None,
-        ) => return Err(RelocationBatchRefused::NoLiveCompactionIntent),
+        ) => return Err(RelocationBatchRefused::NoLiveCompactionIntent.into()),
         (
             CompactionIntentStep::DeleteOnceTheRegionIsEmptyByTheRecords,
             Some((issued_txg, intent)),
@@ -7677,7 +7758,7 @@
             planned_relocation,
         )?);
     }
-    match publish_compaction_step_after_its_check(
+    publish_compaction_step_after_its_check(
         pool,
         allocator,
         previous,
@@ -7685,49 +7766,7 @@
         logged_operations,
         check,
         instance,
-    ) {
-        Ok(published) => Ok(published),
-        Err(CompactionStepRefusedBeforePersisting::Refused(refused)) => Err(refused),
-        Err(
-            CompactionStepRefusedBeforePersisting::EmptyClusterSegmentsAtIssueDifferFromTheAccounting {
-                handed_in,
-                written_by_the_accounting,
-            },
-        ) => {
-            let written = CompactionIntent {
-                region: handed_in.region,
-                allocation_generation_at_issue: handed_in.allocation_generation_at_issue,
-                empty_cluster_segments_at_issue: written_by_the_accounting,
-            };
-            let published = match publish_compaction_step_after_its_check(
-                pool,
-                allocator,
-                previous,
-                &relocations,
-                LoggedOperationsStepOfThePublish::IssueTheCompactionIntent(written),
-                CompactionStepCheckBeforePersisting::EmptyClusterSegmentsAtIssueAsTheAccountingWritesThem(
-                    written,
-                ),
-                instance,
-            ) {
-                Ok(published) => published,
-                Err(CompactionStepRefusedBeforePersisting::Refused(refused)) => return Err(refused),
-                Err(
-                    CompactionStepRefusedBeforePersisting::EmptyClusterSegmentsAtIssueDifferFromTheAccounting {
-                        written_by_the_accounting: written_the_second_time,
-                        ..
-                    },
-                ) => panic!(
-                    "签发那次照记账换过 segs0（{written_by_the_accounting}）再算，记账第 11 项成了 {written_the_second_time}：\
-                     两次只差 logged_ops 条目的字节，取落点与记录都不读它"
-                ),
-            };
-            allocator
-                .compaction_mut()
-                .take_the_intent_written_by_the_issue(handed_in, written);
-            Ok(published)
-        }
-    }
+    )
 }
 
 /// 整理的一次发布落盘之前在算好的账上核什么（[`publish_compaction_step`]）。封闭集合。
@@ -7739,47 +7778,14 @@
         region: CompactionRegion,
         region_check: RegionCheckOfTheIntentDeletion,
     },
-    /// 照抄意图：R 还有一条未释放的记录（判决 m4-segment-code-r1 第三节 F2：这一批腾空了 R 就同一次发布删意图）。
-    RegionKeepsAnUnreleasedRecord(CompactionRegion),
-    /// 签发：写进意图的 `segs0` 等于这次发布写下的记账第 11 项里 R 所在盘的值（判决 G1 字段表定义句，m4-segment-code-r1 第三节 F3）。
-    EmptyClusterSegmentsAtIssueAsTheAccountingWritesThem(CompactionIntent),
-}
-
-/// 整理的一次发布在落盘之前被拒（[`publish_compaction_step_after_its_check`]）：交给调用方的拒绝，或签发那一支要照记账换 `segs0` 再算一次。
-enum CompactionStepRefusedBeforePersisting {
-    Refused(RelocationBatchRefused),
-    EmptyClusterSegmentsAtIssueDifferFromTheAccounting {
-        handed_in: CompactionIntent,
-        written_by_the_accounting: u64,
+    /// 照抄意图，依次核：R 还有一条未释放的记录（判决 m4-segment-code-r1 第三节 F2：这一批腾空了 R 就同一次发布删意图）；
+    /// 再照 `planned_units_rewritten_anyway` 核计划里每个单元都真搬了（判决 m4-segment-code-r2 第三节 H4）。
+    RegionKeepsAnUnreleasedRecord {
+        region: CompactionRegion,
+        planned_units_rewritten_anyway: PlannedUnitsRewrittenAnyway,
     },
 }
 
-impl From<PublishError> for CompactionStepRefusedBeforePersisting {
-    fn from(cause: PublishError) -> Self {
-        CompactionStepRefusedBeforePersisting::Refused(RelocationBatchRefused::Publish(cause))
-    }
-}
-
-/// 一版写下的记账第 11 项（全空聚簇段数）里 `device` 那一行的值。
-///
-/// # Panics
-/// 这一版没有这一行：记账树每次发布整批重写，池里每块盘一行第 11 项、代 = 这次的 txg（`accounting_entries_of_this_publish`）。
-fn empty_cluster_segments_written_by_the_version(
-    version: &TransactionOutput,
-    device: DeviceIdentity,
-) -> u64 {
-    version
-        .accounting_entries
-        .iter()
-        .find(|entry| {
-            entry.statistic == STATISTIC_EMPTY_CLUSTER_SEGMENTS
-                && entry.device == device
-                && entry.generation == version.root.checkpoint_txg
-        })
-        .expect("记账树每次发布整批重写，池里每块盘一行第 11 项、代 = 这次的 txg")
-        .value
-}
-
 /// [`publish_compaction_step`] 核过计划之后那一半：按 `logged_operations` 算这一版、落盘之前照 `check` 核、核过才落盘。
 fn publish_compaction_step_after_its_check<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
@@ -7789,9 +7795,8 @@
     logged_operations: LoggedOperationsStepOfThePublish,
     check: CompactionStepCheckBeforePersisting,
     instance: InstanceGeneration,
-) -> Result<TransactionOutput, CompactionStepRefusedBeforePersisting> {
-    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)
-        .map_err(|refused| CompactionStepRefusedBeforePersisting::Refused(refused.into()))?;
+) -> Result<TransactionOutput, CompactionStepRefused> {
+    let txg = checkpoint_txg_of_the_publish_after(previous.root.checkpoint_txg)?;
     // 整理的每一次发布（搬一批、签发、删意图）都是一个事务，事务号接着这个实例（同建快照、删除的意图那一次：不写数据单元也取下一个号）。
     let transaction_of_this_publish = previous.highest_transaction_number_in_this_instance + 1;
     let deletion_and_lineage = DeletionAndLineageOfThePublish {
@@ -7826,66 +7831,91 @@
         relocations,
         &[],
         |version_after_this_publish| {
-            match check {
-            CompactionStepCheckBeforePersisting::NothingToCheck => Ok(()),
-            CompactionStepCheckBeforePersisting::RegionEmptyByTheRecords {
-                region,
-                region_check,
-            } => {
-                let is_the_region_empty_by_the_records = match region_check {
-                    RegionCheckOfTheIntentDeletion::JudgedByTheRecords => {
-                        region_is_empty_by_the_records(
-                            &version_after_this_publish.allocation_records,
-                            region,
-                        )
-                    }
-                    RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests => false,
-                };
-                if is_the_region_empty_by_the_records {
-                    Ok(())
-                } else {
-                    Err(CompactionStepRefusedBeforePersisting::Refused(
-                        RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
-                            region,
-                        },
-                    ))
+            compaction_step_check_on_the_version_after_this_publish(
+                check,
+                relocations,
+                version_after_this_publish,
+            )
+        },
+    )
+}
+
+/// 整理的一次发布落盘之前那一核（[`CompactionStepCheckBeforePersisting`]）：在这次发布算好的这一版上判，过了交 `Ok`，不过交给调用方的拒绝。
+/// 只读这一版与计划，不读盘：判定与随后落盘的读的是同一份算好的账。
+fn compaction_step_check_on_the_version_after_this_publish(
+    check: CompactionStepCheckBeforePersisting,
+    relocations: &[RelocationOfThePublish],
+    version_after_this_publish: &TransactionOutput,
+) -> Result<(), CompactionStepRefused> {
+    match check {
+        CompactionStepCheckBeforePersisting::NothingToCheck => Ok(()),
+        CompactionStepCheckBeforePersisting::RegionEmptyByTheRecords {
+            region,
+            region_check,
+        } => {
+            let is_the_region_empty_by_the_records = match region_check {
+                RegionCheckOfTheIntentDeletion::JudgedByTheRecords => {
+                    region_is_empty_by_the_records(
+                        &version_after_this_publish.allocation_records,
+                        region,
+                    )
                 }
-            }
-            CompactionStepCheckBeforePersisting::RegionKeepsAnUnreleasedRecord(region) => {
-                if region_is_empty_by_the_records(
-                    &version_after_this_publish.allocation_records,
-                    region,
-                ) {
-                    Err(CompactionStepRefusedBeforePersisting::Refused(
-                        RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried {
-                            region,
-                        },
-                    ))
-                } else {
-                    Ok(())
+                RegionCheckOfTheIntentDeletion::RefusedAsIfTheRegionKeptAnUnreleasedRecordForTests => {
+                    false
                 }
+            };
+            if is_the_region_empty_by_the_records {
+                Ok(())
+            } else {
+                Err(
+                    RelocationBatchRefused::RegionNotEmptyByTheRecordsWhenTheIntentWouldBeDeleted {
+                        region,
+                    }
+                    .into(),
+                )
             }
-            CompactionStepCheckBeforePersisting::EmptyClusterSegmentsAtIssueAsTheAccountingWritesThem(
-                handed_in,
-            ) => {
-                let written_by_the_accounting = empty_cluster_segments_written_by_the_version(
-                    version_after_this_publish,
-                    handed_in.region.device(),
+        }
+        CompactionStepCheckBeforePersisting::RegionKeepsAnUnreleasedRecord {
+            region,
+            planned_units_rewritten_anyway,
+        } => {
+            if region_is_empty_by_the_records(
+                &version_after_this_publish.allocation_records,
+                region,
+            ) {
+                return Err(
+                    RelocationBatchRefused::RegionEmptyByTheRecordsWhileTheIntentWouldBeCarried {
+                        region,
+                    }
+                    .into(),
                 );
-                if handed_in.empty_cluster_segments_at_issue == written_by_the_accounting {
-                    Ok(())
-                } else {
-                    Err(
-                        CompactionStepRefusedBeforePersisting::EmptyClusterSegmentsAtIssueDifferFromTheAccounting {
-                            handed_in,
-                            written_by_the_accounting,
-                        },
-                    )
-                }
+            }
+            match planned_units_rewritten_anyway {
+                PlannedUnitsRewrittenAnyway::LeftToTheCompactionPublish => return Ok(()),
+                PlannedUnitsRewrittenAnyway::RefusedSoThatTheSessionPlansAgain => {}
+            }
+            // 真搬了的（不是这次本来就重写的那几个，`settle_the_allocation_record_tree` 的 `relocations_in_effect`）就是这一版记下的搬迁。
+            let relocated_keys: BTreeSet<&[u8]> = version_after_this_publish
+                .units_relocated_by_this_publish
+                .iter()
+                .map(|relocated| relocated.mapping_key.as_slice())
+                .collect();
+            let rewritten_anyway: Vec<Vec<u8>> = relocations
+                .iter()
+                .filter(|relocation| !relocated_keys.contains(relocation.mapping_key.as_slice()))
+                .map(|relocation| relocation.mapping_key.clone())
+                .collect();
+            if rewritten_anyway.is_empty() {
+                Ok(())
+            } else {
+                Err(
+                    CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish {
+                        mapping_keys: rewritten_anyway,
+                    },
+                )
             }
         }
-        },
-    )
+    }
 }
 
 /// 计划里的一项对着上一版核过、带上搬它要的东西（[`publish_relocation_batch`]）：它是这一版进映射的单元（取它的角色与内存里的字节），
--- a/crates/singlefs-reference-filesystem/src/lib.rs
+++ b/crates/singlefs-reference-filesystem/src/lib.rs
@@ -368,7 +368,8 @@
     /// 上一次恢复为止的版本，按历史次序：起点、之前各轮里已确认的发布之后的内容、各次恢复读出的那一份；第 m 版在位置 m − 1。
     versions_up_to_the_last_recovery: Vec<VisibleState>,
     /// S_0 那一刻允许活着的快照集合，去重、至少一个：还没崩溃过时只有「没有快照」；恢复之后是允许集合里与读出的内容相同的每一个状态那一刻的
-    /// 快照集合（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6），再排除之后实现做了的操作说不通的那几个。
+    /// 快照集合（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6），抛弃最新根的那次恢复之后是被抛弃的那一版之前那一版的
+    /// （判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H5），再排除之后实现做了的操作说不通的那几个。
     /// 序列里每个操作在每一个上各施加一次（[`SnapshotChangeOfTheOperation`]），第 k 个下标与 S_0 那一刻的第 k 个一一对应。
     allowed_snapshot_sets_at_the_last_recovery: Vec<Snapshots>,
 }
@@ -683,12 +684,29 @@
                 recovered,
             });
         }
-        self.restart_from_the_recovered_state(crash_falls_in, recovered);
+        // 恢复之后允许活着的快照集合：允许集合里与读出的内容相同的每一个状态那一刻的快照集合（判决
+        // `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6）。崩溃判定只比文件；建快照与删快照都是发布末尾，
+        // 写了之后报错或崩在里面的那一次落没落盘读出的文件定不了，两种结局都收，不取下标最大的那一个。
+        let indices_whose_state_is_the_recovered_one: Vec<StateIndex> = allowed
+            .allowed
+            .iter()
+            .filter(|allowed_state| allowed_state.state == recovered)
+            .map(|allowed_state| allowed_state.index)
+            .collect();
+        let allowed_snapshot_sets_after_the_recovery =
+            self.distinct_snapshot_sets_at(&indices_whose_state_is_the_recovered_one);
+        self.restart_from_the_recovered_state(
+            crash_falls_in,
+            recovered,
+            allowed_snapshot_sets_after_the_recovery,
+        );
         Ok(allowed)
     }
 
     /// 崩溃恢复抛弃最新根的那一次（恢复时有读故障）：不判读出的内容，以它重新起头（r6 判决第二节 U2 一行：
-    /// 「崩溃恢复抛弃最新根的那一次，RefFS 以读出内容重新起头、不判那一步」）。
+    /// 「崩溃恢复抛弃最新根的那一次，RefFS 以读出内容重新起头、不判那一步」）。重新起头时允许活着的快照集合取被抛弃的那一版
+    /// 之前那一版（读出的那条根所在的已发布状态）的（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H5，
+    /// 主 agent 照 U2 定的读法）：允许集合里每一个状态都可能是被抛弃的最新根那一版，各取它之前那一版、并起来。
     ///
     /// # Errors
     /// hi 越界、或写了之后报错的操作后面到崩溃为止又起了操作（同 [`Self::allowed_set_for_a_crash_in`]），返回时模型不动。
@@ -698,15 +716,59 @@
         recovered: VisibleState,
     ) -> Result<(), CrashNotAccepted> {
         // 只核 hi 判不判得了；读出的内容在不在允许集合里不判
-        self.allowed_set_for_a_crash_in(crash_falls_in)?;
-        self.restart_from_the_recovered_state(crash_falls_in, recovered);
+        let allowed = self.allowed_set_for_a_crash_in(crash_falls_in)?;
+        let indices_of_the_published_state_before_each_abandoned_one: Vec<StateIndex> = allowed
+            .allowed
+            .iter()
+            .map(|allowed_state| self.index_of_the_published_state_before(allowed_state.index))
+            .collect();
+        let allowed_snapshot_sets_after_the_recovery = self
+            .distinct_snapshot_sets_at(&indices_of_the_published_state_before_each_abandoned_one);
+        self.restart_from_the_recovered_state(
+            crash_falls_in,
+            recovered,
+            allowed_snapshot_sets_after_the_recovery,
+        );
         Ok(())
     }
 
+    /// S_j 那一版之前那一版的下标：上一次恢复之后、j 之前最后一次已确认发布（返回成功的发布调用、fsync、回退、建快照、删快照）的末尾；
+    /// 没有时是 0（上一次恢复读出、重新起头的那一版，还没崩溃过时是起点）。j 之前的操作都返回过，写了之后报错的只能是 hi 之前的最后一个
+    /// （[`Self::allowed_set_for_a_crash_in`] 拒掉别的），所以 j 之前的发布末尾都已确认。
+    fn index_of_the_published_state_before(&self, index: StateIndex) -> StateIndex {
+        let Some(operations_before_the_state) = index.0.checked_sub(1) else {
+            // j 就是 S_0（上一次恢复之后没有已确认的发布）：之前那一版在上一次恢复之前（还没崩溃过时没有），RefFS 不记那时的快照集合，
+            // H5 的读法在这里没有答案。条款没定这一支取什么；要返回「第一版不支持」得给 `CrashNotAccepted` 加成员、连带改调用方的穷举，
+            // 不在这一轮，这一支照旧取 S_0 那一刻的。
+            return StateIndex(0);
+        };
+        self.operations_since_the_last_recovery[..operations_before_the_state]
+            .iter()
+            .enumerate()
+            .rev()
+            .find(|(_, operation)| match operation.outcome {
+                EnteredOutcome::Succeeded => operation.kind.ends_a_publish(),
+                EnteredOutcome::FailedAfterWriting => false,
+            })
+            .map_or(StateIndex(0), |(position, _)| StateIndex(position + 1))
+    }
+
+    /// 这几个下标那一刻允许活着的快照集合，并起来、去重。
+    fn distinct_snapshot_sets_at(&self, indices: &[StateIndex]) -> Vec<Snapshots> {
+        let mut distinct_snapshot_sets = Vec::new();
+        for index in indices {
+            for snapshots in self.snapshot_set_of_each_allowed_start_at(*index) {
+                push_distinct_snapshot_set(&mut distinct_snapshot_sets, snapshots);
+            }
+        }
+        distinct_snapshot_sets
+    }
+
     fn restart_from_the_recovered_state(
         &mut self,
         crash_falls_in: OperationNumber,
         recovered: VisibleState,
+        allowed_snapshot_sets_after_the_recovery: Vec<Snapshots>,
     ) {
         // 前 hi − 1 个返回过；第 hi 个没返回，它是发布也不算已确认；第 hi 个之后的没发生。
         let returned_before_the_crash =
@@ -718,38 +780,6 @@
         // 恢复读出、以它重新起头的那一份也是一个版本：之后可写挂载写的根映到它（r6 用户定案 H1-收严）
         self.versions_up_to_the_last_recovery
             .push(recovered.clone());
-        // 恢复之后允许活着的快照集合：允许集合里与读出的内容相同的每一个状态那一刻的快照集合（判决
-        // `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6）。崩溃判定只比文件；建快照与删快照都是发布末尾，
-        // 写了之后报错或崩在里面的那一次落没落盘读出的文件定不了，两种结局都收，不取下标最大的那一个。
-        // 读出的内容一个都对不上（抛弃最新根的那次恢复不判读出的内容）时照取 S_0 那一刻的。
-        let indices_whose_state_is_the_recovered_one: Vec<StateIndex> = self
-            .allowed_set_for_a_crash_in(crash_falls_in)
-            .map(|allowed| {
-                allowed
-                    .allowed
-                    .iter()
-                    .filter(|allowed_state| allowed_state.state == recovered)
-                    .map(|allowed_state| allowed_state.index)
-                    .collect()
-            })
-            .expect(
-                "两个调用方都先核过 hi 判得了（`allowed_set_for_a_crash_in` 返回 Ok）才走到这里",
-            );
-        let indices_the_snapshot_sets_are_taken_at =
-            if indices_whose_state_is_the_recovered_one.is_empty() {
-                vec![StateIndex(0)]
-            } else {
-                indices_whose_state_is_the_recovered_one
-            };
-        let mut allowed_snapshot_sets_after_the_recovery = Vec::new();
-        for index in indices_the_snapshot_sets_are_taken_at {
-            for snapshots in self.snapshot_set_of_each_allowed_start_at(index) {
-                push_distinct_snapshot_set(
-                    &mut allowed_snapshot_sets_after_the_recovery,
-                    snapshots,
-                );
-            }
-        }
         // 每次恢复之后以读出的内容为起点接着走（r5 判决第二节 U2 ②）
         self.state_at_the_last_recovery = recovered;
         self.allowed_snapshot_sets_at_the_last_recovery = allowed_snapshot_sets_after_the_recovery;
@@ -1594,4 +1624,87 @@
             "以读出的 A 重新起头，之后读回 B 判红：{refused:?}"
         );
     }
+
+    /// 攻方 R6 那一形（判决 `research/prompts/m4-segment-code-r2-main-verification.md` 第三节 H5，读法照 U2）：写 A、发布、建快照 7、
+    /// 覆盖写 B、发布，抛弃最新根（B 那次发布）的那次恢复读出 A。读出的 A 不在允许集合里也不判、照收；重新起头时的快照集合取被抛弃那一版
+    /// 之前那一版（建快照那一版）的，快照 7 照在，读它读出 A 不判红。
+    #[test]
+    fn a_recovery_that_abandoned_the_newest_root_keeps_a_snapshot_created_in_the_published_state_before_it(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        let crash = model.crash_after_every_operation_returned();
+        let allowed = model
+            .allowed_set_for_a_crash_in(crash)
+            .expect("崩溃落在崩溃本身里");
+        assert!(
+            !allowed.contains(&file_with("A")),
+            "读出的 A 不在允许集合里（只有 B 那次发布）：{allowed:?}"
+        );
+        model
+            .restart_after_a_recovery_that_abandoned_the_newest_root(crash, file_with("A"))
+            .expect("抛弃最新根的那一次恢复不判读出的内容");
+        assert_eq!(
+            model.allowed_snapshot_sets(),
+            vec![Snapshots::from([(SnapshotName(7), file_with("A"))])],
+            "快照集合取被抛弃那一版之前那一版（建快照那一版）的：快照 7 照在"
+        );
+        model
+            .judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
+                SnapshotName(7),
+                &file_with("A"),
+            )
+            .expect("读快照 7 读出建它那一刻的 A 不判红");
+        assert_eq!(
+            model.state_after_every_operation_so_far(),
+            &file_with("A"),
+            "以读出的 A 重新起头"
+        );
+    }
+
+    /// 被抛弃的最新根就是建快照 7 的那一次发布：之前那一版（第一个文件那次发布）没有快照，重新起头之后快照 7 不在——
+    /// 读它读出 A 判红，同名再建照收（判决第三节 H5，取的是被抛弃那一版之前那一版，不是被抛弃那一版自己的）。
+    #[test]
+    fn a_recovery_that_abandoned_the_newest_root_written_by_a_snapshot_creation_restarts_without_that_snapshot(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::CreateSnapshot(SnapshotName(7)));
+        let crash = model.crash_after_every_operation_returned();
+        model
+            .restart_after_a_recovery_that_abandoned_the_newest_root(crash, file_with("A"))
+            .expect("抛弃最新根的那一次恢复不判读出的内容");
+        assert_eq!(
+            model.allowed_snapshot_sets(),
+            vec![Snapshots::new()],
+            "快照集合取建快照之前那一版的：没有快照"
+        );
+        let before_the_read = model.clone();
+        assert_eq!(
+            model.judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(
+                SnapshotName(7),
+                &file_with("A")
+            ),
+            Err(SnapshotReadOutsideTheAllowedReads {
+                name: SnapshotName(7),
+                read_out: file_with("A"),
+                allowed_reads: vec![None],
+            }),
+            "快照 7 随被抛弃的那一版一起没了：读出 A 判红"
+        );
+        assert_eq!(model, before_the_read, "判红的读不动模型");
+        assert_eq!(
+            model.submit(
+                Operation::CreateSnapshot(SnapshotName(7)),
+                OperationOutcome::Succeeded
+            ),
+            Ok(SubmittedOperation::Entered(OperationNumber(1))),
+            "快照名 7 空着：同名再建照收"
+        );
+    }
 }
````
