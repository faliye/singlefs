# 增补 3 第 2 件代码三方第二轮：被判的改动（第一轮判决之后的改法）

基准：实现员开工前的仓副本（与第一轮开工快照 `research/prompts/m2-supp3-item2-code-r1-start-snapshot.sha256` 里 `crates/` 下六个文件逐个同 sha256），对比工作区 2026-09-19 15:2x UTC。命令：`diff -ruN --exclude=target <基准>/crates crates`。

````diff
diff -ruN '--exclude=target' crates/mutations.tsv crates/mutations.tsv
--- crates/mutations.tsv	2026-09-19 05:42:32.680608466 +0000
+++ crates/mutations.tsv	2026-09-19 15:04:21.435460605 +0000
@@ -153,3 +153,12 @@
 增补 3 第 2 件（理想模型自身）：F_生效取各盘最大 F 的最大（只一块盘带新 F 也生效）	crates/singlefs-harness/src/model.rs	        highest_floor_per_device\n            .values()\n            .copied()\n            .min()	        highest_floor_per_device\n            .values()\n            .copied()\n            .max()	-p singlefs-harness --lib -- model::tests::a_raised_floor_carried	a_raised_floor_carried_by_one_device_only_does_not_take_effect
 增补 3 第 2 件（理想模型自身）：分配代只要不小于写它的那次发布就算对	crates/singlefs-harness/src/model.rs	&& record.generation == *written_at	&& record.generation >= *written_at	-p singlefs-harness --lib -- model::tests::a_carried_unit_keeps	a_carried_unit_keeps_the_generation_of_the_publish_that_wrote_it
 增补 3 第 2 件（理想模型自身，D13 已定项 5）：模型模块 use 了 singlefs_core	crates/singlefs-harness/src/model.rs	use std::rc::Rc;	use std::rc::Rc;\nuse singlefs_core as _;	-p singlefs-harness --lib -- model_comparison::tests::the_model_module_uses_only	the_model_module_uses_only_the_standard_library_and_the_format_constants
+增补 3 第 2 件（代码三方第一轮判决第三节第 1 条，攻方变异 W1）：分配记录墙差一，812 条正好装满也拒（> 写成 >=）；逼近墙的取样点判出	crates/singlefs-core/src/transaction.rs	    if records_after_this_publish > allocation_node_capacity {	    if records_after_this_publish >= allocation_node_capacity {	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_wall_sampling	allocation_record_wall_sampling_without_the_checker_refuses_only_above_one_node_by_the_true_count
+增补 3 第 2 件（代码三方第一轮判决第三节第 1 条，攻方变异 W1 同一处）：分配记录墙差一；812 条那一格的边沿用例判出	crates/singlefs-core/src/transaction.rs	    if records_after_this_publish > allocation_node_capacity {	    if records_after_this_publish >= allocation_node_capacity {	-p singlefs-harness --test second_transaction_supplement_three_random_history -- an_overwrite_that_fills_the_allocation_node	an_overwrite_that_fills_the_allocation_node_to_exactly_812_records_succeeds_and_the_next_is_refused
+增补 3 第 2 件（代码三方第一轮判决第三节第 1 条的改法）：模型的分配记录墙只看沿来路的上界、不看镜像上的真条数	crates/singlefs-harness/src/model.rs	                upper_bound_exceeds && true_count_exceeds	                upper_bound_exceeds && (true_count_exceeds || true)	-p singlefs-harness --lib -- model::tests::the_allocation_record_wall_is_permitted	the_allocation_record_wall_is_permitted_only_when_the_counted_records_plus_this_publish_exceed_812
+增补 3 第 2 件（代码三方第一轮判决第三节第 1 条的改法）：抬 F 做完几次之后被墙拒，基数仍数这一步起点那一版	crates/singlefs-harness/src/model.rs	        if self.walls_are_judged_before_the_first_write || publishes_completed == 0 {	        if self.walls_are_judged_before_the_first_write || publishes_completed < usize::MAX {	-p singlefs-harness --lib -- model::tests::mount_and_raise_count	mount_and_raise_count_the_wall_from_the_version_their_admission_starts_from
+增补 3 第 2 件（代码三方第一轮判决第三节第 1 条的改法）：checker 数一条根下的分配记录只数一半	crates/singlefs-checker/src/walk.rs	        records += node.entries.len();	        records += node.entries.len() / 2;	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_records_counted_on_the_image	allocation_records_counted_on_the_image_are_one_per_unit_per_device_and_zero_without_a_file
+增补 3 第 2 件（代码三方第一轮判决第三节第 2 条，攻方变异 R1）：被抛弃时间线的根报成低于 F（只换判别字段）	crates/singlefs-core/src/mount.rs	            exclusion: RollbackCandidateExclusion::OnAbandonedTimeline,	            exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_onto_an_abandoned_root_above_the_floor	rolling_back_onto_an_abandoned_root_above_the_floor_is_refused_for_being_abandoned
+增补 3 第 2 件（代码三方第一轮判决第三节第 2 条的改法）：胶水把回退目标「被抛弃」映射成「低于 F」	crates/singlefs-harness/src/model_comparison.rs	        RollbackCandidateExclusion::OnAbandonedTimeline => {\n            ModelRefusalReason::RollbackTargetOnAbandonedTimeline\n        }	        RollbackCandidateExclusion::OnAbandonedTimeline => {\n            ModelRefusalReason::RollbackTargetBelowEffectiveFloor\n        }	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_onto_an_abandoned_root_above_the_floor	rolling_back_onto_an_abandoned_root_above_the_floor_is_refused_for_being_abandoned
+增补 3 第 2 件（代码三方第一轮判决第三节第 2 条，C368 仍欠的那一半）：发布层把分配器的落点拒绝一律报成「每块盘上都没有」	crates/singlefs-core/src/transaction.rs	            unit: *identity,\n            refusal,\n        })?;	            unit: *identity,\n            refusal: {\n                let _ = refusal;\n                PlacementRefusal::NoFreeSlotOnAnyDevice\n            },\n        })?;	-p singlefs-harness --test second_transaction_supplement_two_unequal_devices -- filling_the_smaller_device	filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking
+增补 3 第 2 件（代码三方第一轮判决第三节第 2 条的改法）：胶水把小盘写满、各盘落点不一致也映射成单元区墙	crates/singlefs-harness/src/model_comparison.rs	        } => ObservedRefusalReason::Unexplained,	        } => explained(ModelRefusalReason::UnitAreaWall),	-p singlefs-harness --lib -- model_comparison::tests::only_a_placement_refused	only_a_placement_refused_on_every_device_passes_as_the_unit_area_wall
diff -ruN '--exclude=target' crates/singlefs-checker/src/walk.rs crates/singlefs-checker/src/walk.rs
--- crates/singlefs-checker/src/walk.rs	2026-09-18 03:51:04.020795012 +0000
+++ crates/singlefs-checker/src/walk.rs	2026-09-19 11:35:43.206194486 +0000
@@ -1250,6 +1250,44 @@
     }
 }
 
+/// 一条根的分配记录树里有几条记录：按 checker 自己的字段表读这条根的树表，找种类 3（分配记录）的条目，数它根节点叶里的条目
+/// （第一版分配记录树只有一个节点；一条记录记一个单元，D3（空间分配） 已定项 7）。只读不判——给理想模型的对拍数分配记录墙的真条数
+/// （增补 3 第 2 件代码三方第一轮判决第三节第 1 条：从镜像上现数，不用分配器的状态）。
+/// 树表里没有分配记录树的条目、或那一条的根指针全零时是 0 条：树表 0 条的一版上没有分配记录树，mkfs 写的两个单元的记录要到
+/// 第一个文件版本才落盘。树表指针全零、树表或分配记录树的节点读不出或条目比字段表窄、分配记录树有内部节点（内部条目格式没有条款，
+/// 总审核 D8-D11 发现 16）时数不出，交回 None。
+#[must_use]
+pub fn allocation_record_count_under_root(
+    reader: &dyn ImageReader,
+    root: &crate::RootView,
+) -> Option<usize> {
+    let mut cache = IndexNodeCache::new();
+    let tree_table_pointer = parse_node_pointer(&root.record_bytes[36..122]);
+    if tree_table_pointer.all_zero {
+        return None;
+    }
+    let tree_table = read_index_node_without_judging(reader, &tree_table_pointer, &mut cache)?;
+    let mut records = 0;
+    for entry in &tree_table.entries {
+        if entry.len() < tree_table_entry_bytes() {
+            return None;
+        }
+        if read_u16(entry, 10) != TREE_KIND_ALLOCATION {
+            continue;
+        }
+        let allocation_root = parse_node_pointer(&entry[14..100]);
+        if allocation_root.all_zero {
+            continue;
+        }
+        let node = read_index_node_without_judging(reader, &allocation_root, &mut cache)?;
+        if node.level > 0 || node.entry_width < allocation_record_bytes() {
+            return None;
+        }
+        records += node.entries.len();
+    }
+    Some(records)
+}
+
 /// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。
 #[must_use]
 pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
diff -ruN '--exclude=target' crates/singlefs-core/src/allocator.rs crates/singlefs-core/src/allocator.rs
--- crates/singlefs-core/src/allocator.rs	2026-09-18 04:10:11.041921018 +0000
+++ crates/singlefs-core/src/allocator.rs	2026-09-19 11:30:16.377210586 +0000
@@ -713,7 +713,8 @@
         }
     }
 
-    /// 用户数据：见 `try_allocate_user_data`；拒绝的原因在这里丢掉，发布路径把 `None` 报成装不下。
+    /// 用户数据：见 `try_allocate_user_data`；拒绝的原因在这里丢掉，只给不看原因的调用方（测试里「一定分得到」的那几处）。
+    /// 发布路径调 `try_allocate_user_data`，按原因报（`PublishError::PlacementRefused`）。
     pub fn allocate_user_data(&mut self, generation: CheckpointTxg) -> Option<Placement> {
         self.try_allocate_user_data(generation).ok()
     }
@@ -760,7 +761,8 @@
         Ok(placement)
     }
 
-    /// 提交内生块：见 `try_allocate_commit_generated`；拒绝的原因在这里丢掉，发布路径把 `None` 报成装不下。
+    /// 提交内生块：见 `try_allocate_commit_generated`；拒绝的原因在这里丢掉，只给不看原因的调用方（测试里「一定分得到」的那几处）。
+    /// 发布路径调 `try_allocate_commit_generated`，按原因报（`PublishError::PlacementRefused`）。
     pub fn allocate_commit_generated(
         &mut self,
         footprint: UnitFootprint,
diff -ruN '--exclude=target' crates/singlefs-core/src/mount.rs crates/singlefs-core/src/mount.rs
--- crates/singlefs-core/src/mount.rs	2026-09-18 03:53:28.310575031 +0000
+++ crates/singlefs-core/src/mount.rs	2026-09-19 11:29:42.426749219 +0000
@@ -43,12 +43,11 @@
     RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion,
     Acquisition(AcquisitionFailed),
     Publish(PublishError),
-    /// 回退的目标根不在根环里（没有那个 (实例, txg) 的可读根槽）。
-    RollbackTargetNotInRing(RollbackTarget),
-    /// 回退的目标根在根环里，却不在回退候选集里：被抛弃时间线的根，或 txg 低于回退下界 F。
+    /// 回退的目标根不能拿来回退，`exclusion` 说是哪一条（管理员要做的决定都是换一条目标；调用方按这个字段分流，不看给人看的文字——
+    /// 增补 3 第 2 件代码三方第一轮判决第三节第 2 条：此前只带一句理由文字，胶水分不出是哪一条）。在任何写之前拒绝。
     RollbackTargetNotACandidate {
         target: RollbackTarget,
-        reason: &'static str,
+        exclusion: RollbackCandidateExclusion,
     },
     /// 要抬的 F 超过上限 min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)（D16（发布语义） 已定项 1）。
     RollbackFloorAboveCeiling {
@@ -62,9 +61,6 @@
         first_row_instance: InstanceGeneration,
         instance_to_acquire: InstanceGeneration,
     },
-    /// 回退的目标根的树表 0 条（还没发布过文件版本）：回退行要重写实例表，没有文件版本的一版上它的落点记在哪没有条款，
-    /// 第一版不支持。在任何写之前拒绝。
-    RollbackToVersionWithoutFileUnsupported(RollbackTarget),
     /// 树表 0 条、而根记录指着的实例表或树表不是 mkfs 写的那一版（指针的诞生 txg 不是 0）：这样一版的分配记录在哪没有条款，
     /// 这个实现自己写不出这样的根（坏盘或别的写者才有），拒绝挂载。
     VersionWithoutFileNotWrittenByMakeFilesystem {
@@ -121,6 +117,21 @@
     },
 }
 
+/// 回退目标被挡下的是哪一条。`mount_rollback` 按成员的次序判，一条目标同时中几条时只报最先判到的那一条。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RollbackCandidateExclusion {
+    /// 根环里没有那个 (实例, txg) 的可读根槽：候选集是根环里的根（D23（journal 的角色与格式） 已定项 14）。
+    NotInRing,
+    /// txg 低于生效的回退下界 F（D16（发布语义） 已定项 1「回退候选集」：txg ≥ F_生效）。
+    BelowEffectiveFloor,
+    /// 最新根指着的实例表里有那个实例的行 (i, Ti, Wi) 且目标的 txg > Ti：被抛弃时间线上的根（D23（journal 的角色与格式） 已定项 14
+    /// 「按实例表判仍然有效」）。
+    OnAbandonedTimeline,
+    /// 目标那一版树表 0 条（还没发布过文件版本）：按前三条它可以在候选集里，但回退行要重写实例表，没有文件版本的一版上它的落点记在哪
+    /// 没有条款（D16（发布语义） 已定项 9 只定了树表 0 条时空发布写零个单元），第一版不支持。
+    TargetVersionWithoutFileUnsupported,
+}
+
 /// 回退的目标：管理员带外从回退候选集里选的那条根（D23（journal 的角色与格式） 已定项 14 的显式例外）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct RollbackTarget {
@@ -1189,7 +1200,10 @@
             root.instance == target.instance && root.checkpoint_txg == target.checkpoint_txg
         })
         .copied()
-        .ok_or(MountError::RollbackTargetNotInRing(target))?;
+        .ok_or(MountError::RollbackTargetNotACandidate {
+            target,
+            exclusion: RollbackCandidateExclusion::NotInRing,
+        })?;
     // 候选集：按最新根指着的实例表判仍然有效——(i, T) 可选 ⟺ 无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti；且 txg ≥ F_生效。
     let newest_table = instance_table_of_root(&*devices, &newest_root)
         .ok_or(MountError::InstanceTableMalformed)?;
@@ -1203,7 +1217,7 @@
     if target.checkpoint_txg < effective_floor {
         return Err(MountError::RollbackTargetNotACandidate {
             target,
-            reason: "txg 低于生效的回退下界 F",
+            exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,
         });
     }
     if newest_table
@@ -1213,13 +1227,16 @@
     {
         return Err(MountError::RollbackTargetNotACandidate {
             target,
-            reason: "实例表里那个实例的行 T 更小：这是被抛弃时间线的根",
+            exclusion: RollbackCandidateExclusion::OnAbandonedTimeline,
         });
     }
     // 回退到树表 0 条的根（例如第一个事务里 txg 1、2 的暖机根）：回退行要重写实例表，没有文件版本的一版上它的落点记在哪、换下的 mkfs 实例表
     // 与候选根引用的单元谁护着都没有条款（D16（发布语义） 已定项 9 只定了树表 0 条时空发布写零个单元）——在任何写之前拒绝。
     if tree_table_has_no_entries(&*devices, &target_root)? {
-        return Err(MountError::RollbackToVersionWithoutFileUnsupported(target));
+        return Err(MountError::RollbackTargetNotACandidate {
+            target,
+            exclusion: RollbackCandidateExclusion::TargetVersionWithoutFileUnsupported,
+        });
     }
     // R_old 自己那条记录读得出就拿它当上一版的记录（只有 jsn 会被用到），读不出就拿最大 jsn 那条顶着，与可写挂载同一条路。
     let own_record = records
diff -ruN '--exclude=target' crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/transaction.rs
--- crates/singlefs-core/src/transaction.rs	2026-09-18 03:53:44.437474541 +0000
+++ crates/singlefs-core/src/transaction.rs	2026-09-19 11:49:11.885126059 +0000
@@ -24,7 +24,9 @@
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
     TreeIdentifier,
 };
-use crate::allocator::{AllocationRecord, Placement, PoolAllocator, UnitFootprint};
+use crate::allocator::{
+    AllocationRecord, Placement, PlacementRefusal, PoolAllocator, UnitFootprint,
+};
 use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
 use crate::journal::{back_chain_of, record_offset, JournalRecord, NamedUnit};
 use crate::make_filesystem::{
@@ -918,11 +920,16 @@
         .map(|(_, locations)| locations)
 }
 
-/// 发布能出的错：单元区放不下（哪一个单元没拿到落点），或底层块设备错。
+/// 发布能出的错：分配器给不出落点（哪一个单元、为什么）、两棵单节点树装不下、内容装不下、释放判定路径对不上，或底层块设备错。
 #[derive(Debug)]
 pub enum PublishError {
-    NoSpaceFor {
+    /// 分配器拒了这个单元的落点，`refusal` 原样带着分配器的原因：每块盘上都没有合政策的落点（容量不够）是 `NoFreeSlotOnAnyDevice`；
+    /// 小盘写满、各盘的落点不一致是第一版不支持的池形状（D2（RAID 条带策略） 已定项 2 ⚠️、已定项 10；D3（空间分配） 已定项 8），
+    /// 不是容量不够。此前这四种一律报成 `NoSpaceFor`（C368（分配器落点只看盘 0，盘不等大时断言失败） 仍欠的那一半；
+    /// 增补 3 第 2 件代码三方第一轮判决第三节第 2 条：胶水把它们都当成单元区墙）。
+    PlacementRefused {
         unit: TransactionUnit,
+        refusal: PlacementRefusal,
     },
     /// 分配记录树第一版只有一个节点，这次发布之后的记录数装不下（每次发布每盘加 8 条、释放只改写不删）。
     AllocationRecordsExceedOneNode {
@@ -1186,7 +1193,8 @@
 /// 对象出生代与容器身份不改，改动计数取这次的 txg；上一版的八个落点经映射释放（进 defer 队列）。
 ///
 /// # Errors
-/// 释放判定路径查不到上一版的某个单元（`ReleaseNotInMapping` 一族）、空间不够、装不下、块设备报错，都原样交回。
+/// 释放判定路径查不到上一版的某个单元（`ReleaseNotInMapping` 一族）、落点被拒（`PlacementRefused`，带分配器的原因）、装不下、
+/// 块设备报错，都原样交回。
 pub fn publish_overwrite<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
@@ -1221,11 +1229,11 @@
 
 /// 发布一版：先做准入（内容装得进一个数据单元，再加 `publish_admission` 的两条），再释放上一版被换下的角色的落点、分配、装单元、
 /// 按 D16（发布语义） 已定项 7 的持久顺序落盘。准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立，
-/// 释放与分配都不算数（第二轮攻方腿：`NoSpaceFor` 在释放之后、分配到一半返回，留下半新的池，拿同一个上一版重试撞断言）；
+/// 释放与分配都不算数（第二轮攻方腿：落点被拒（当时叫 `NoSpaceFor`）在释放之后、分配到一半返回，留下半新的池，拿同一个上一版重试撞断言）；
 /// 落盘那几步里失败的，这次已记的写进写入口的失败账（增补 2 第 20b 行，`PoolWriter::writes_of_failed_publishes`）。
 ///
 /// # Errors
-/// `ContentExceedsDataUnit`、`AllocationRecordsExceedOneNode`、`AccountingEntriesExceedOneNode`、释放判定路径的四种错、`NoSpaceFor`、块设备错。
+/// `ContentExceedsDataUnit`、`AllocationRecordsExceedOneNode`、`AccountingEntriesExceedOneNode`、释放判定路径的四种错、`PlacementRefused`、块设备错。
 pub fn publish_version<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
@@ -1587,12 +1595,15 @@
     let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
     for identity in rewritten {
         let placement = match identity.placement() {
-            PlacementRule::UserData => allocator.allocate_user_data(txg),
+            PlacementRule::UserData => allocator.try_allocate_user_data(txg),
             PlacementRule::CommitGenerated(footprint) => {
-                allocator.allocate_commit_generated(footprint, txg)
+                allocator.try_allocate_commit_generated(footprint, txg)
             }
         };
-        let placement = placement.ok_or(PublishError::NoSpaceFor { unit: *identity })?;
+        let placement = placement.map_err(|refusal| PublishError::PlacementRefused {
+            unit: *identity,
+            refusal,
+        })?;
         slots.insert(*identity, placement.slot);
     }
     let slot_of = |identity: TransactionUnit| slots[&identity];
diff -ruN '--exclude=target' crates/singlefs-harness/src/history.rs crates/singlefs-harness/src/history.rs
--- crates/singlefs-harness/src/history.rs	2026-09-19 05:32:12.485052971 +0000
+++ crates/singlefs-harness/src/history.rs	2026-09-19 11:49:11.928125586 +0000
@@ -19,9 +19,11 @@
 use std::sync::{Mutex, Once};
 
 use singlefs_checker::image::{chosen_superblocks, valid_roots, InvariantVerdict};
-use singlefs_checker::walk::check_pool_image;
+use singlefs_checker::walk::{allocation_record_count_under_root, check_pool_image};
 use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
-use singlefs_core::allocator::{AllocationRecord, DeviceFreeMap, Placement, PoolAllocator};
+use singlefs_core::allocator::{
+    AllocationRecord, DeviceFreeMap, Placement, PlacementRefusal, PoolAllocator,
+};
 use singlefs_core::block_device::{BlockDeviceError, PhysicalBlockSizeInBytes};
 use singlefs_core::journal::back_chain_of;
 use singlefs_core::make_filesystem::{
@@ -45,7 +47,8 @@
 use crate::crash::{MemoryPool, SparseBlockDevice};
 use crate::model::{
     IdealModel, ModelAnswer, ModelCheckpointTxg, ModelDeviceIdentity, ModelDisagreement,
-    ModelJudgementCounts, ModelPoolGeometry, ObservedEffect, ObservedOutcome,
+    ModelJudgementCounts, ModelPoolGeometry, ModelRefusalReason, ModelRootKey, ObservedEffect,
+    ObservedOutcome, ObservedRefusalReason,
 };
 use crate::model_comparison::{
     model_root_key, observed_mount, observed_read_back, observed_root_of_file_version,
@@ -413,6 +416,30 @@
         rollback_targets: RollbackTargetDraw::FloorRootHalfTheTime,
     };
 
+    /// 分配记录墙那一格（增补 3 第 2 件代码三方第一轮判决第三节第 1 条）的取样点：一律从第一个文件起，带文件的版本上多半覆盖写
+    /// （每次每盘加 8 条），夹着可写挂载（写行与暖机每次加 18 或 26 条）、抬 F（回收之后复用改写记录，条数涨得慢）与少量回退，
+    /// 让逼近 812 条时的条数落在不同的余数上——墙的「差一」只在某次准入之后正好 812 条时分得出。配 `PerStepChecker::Skipped` 跑。
+    pub const TOWARD_THE_ALLOCATION_RECORD_WALL: GenerationWeights = GenerationWeights {
+        name: "逼近分配记录墙（812 条那一格的取样点）",
+        starting_points: &[(HistoryStartingPoint::AfterFirstFile, 1)],
+        with_session_closed: &[
+            (HistoryOperationKind::CloseAndMountWritable, 95),
+            (HistoryOperationKind::CloseAndMountRollback, 5),
+        ],
+        with_session_open_without_file: &[
+            (HistoryOperationKind::PublishFirstFile, 90),
+            (HistoryOperationKind::CloseAndMountWritable, 10),
+        ],
+        with_session_open_with_file: &[
+            (HistoryOperationKind::PublishOverwrite, 80),
+            (HistoryOperationKind::CloseAndMountWritable, 12),
+            (HistoryOperationKind::RaiseRollbackFloor, 5),
+            (HistoryOperationKind::CloseAndMountRollback, 2),
+            (HistoryOperationKind::ColdStartRecover, 1),
+        ],
+        rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
+    };
+
     fn for_session(&self, expected: ExpectedSession) -> &'static [(HistoryOperationKind, u64)] {
         match expected {
             ExpectedSession::Closed => self.with_session_closed,
@@ -607,14 +634,16 @@
     }
 }
 
-/// 入口返回 Err 时交给模型的观测：成员映射成的理由、拒之前做完几次发布、录制流在这一步里有没有多出写或屏障。
+/// 入口返回 Err 时交给模型的观测：成员映射成的理由、拒之前做完几次发布、录制流在这一步里有没有多出写或屏障；理由是分配记录墙时
+/// 连同从镜像上数的准入基数（`allocation_records_counted_for_the_wall`）。
 fn observed_refusal(
     member: String,
-    reason: crate::model::ObservedRefusalReason,
+    reason: ObservedRefusalReason,
     publishes_completed: usize,
     stream_length_before: usize,
     stream: &SharedStream,
     reported_ceiling: Option<ModelCheckpointTxg>,
+    allocation_records_counted_on_the_image: Option<u64>,
 ) -> ObservedOutcome {
     ObservedOutcome::Refused {
         member,
@@ -622,9 +651,57 @@
         publishes_completed,
         wrote_anything: stream.operation_count() != stream_length_before,
         reported_ceiling,
+        allocation_records_counted_on_the_image,
     }
 }
 
+/// 两块盘此刻的整份镜像（拷一份，checker 与观察者读它）。
+fn image_of(devices: &[(DeviceIdentity, HistoryDevice)]) -> MemoryPool {
+    MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.inner().image.clone()))
+            .collect(),
+        device_size_in_bytes: HISTORY_DEVICE_BYTES,
+    }
+}
+
+/// 按 checker 的读法在镜像上数一条根下的分配记录（`singlefs_checker::walk::allocation_record_count_under_root`，与实现的分配器
+/// 不共用代码）：根环里 (txg, 实例) 等于 `root` 的那条自证过的根。超级块或那条根找不到、那棵树读不出都是 None。
+#[must_use]
+pub fn allocation_records_on_the_image_under(
+    image: &MemoryPool,
+    root: ModelRootKey,
+) -> Option<u64> {
+    let geometry = chosen_superblocks(image)
+        .into_iter()
+        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))?;
+    let (_, _, view) = valid_roots(image, &geometry)
+        .into_iter()
+        .find(|(_, _, view)| {
+            view.checkpoint_txg == root.checkpoint_txg.0
+                && u64::from(view.instance) == root.instance.0
+        })?;
+    let records = allocation_record_count_under_root(image, &view)?;
+    Some(u64::try_from(records).expect("一个节点至多几百条"))
+}
+
+/// 分配记录墙拒时，从镜像上现数准入的基数（增补 3 第 2 件代码三方第一轮判决第三节第 1 条：用 checker 的解析读镜像，不用分配器的状态）：
+/// 模型点名的那一版下有几条分配记录。理由不是分配记录墙的不数；模型答不了这一步（答案本身就是对不上的那一格）也不数。
+fn allocation_records_counted_for_the_wall(
+    reason: ObservedRefusalReason,
+    answer: Option<&ModelAnswer>,
+    publishes_completed: usize,
+    devices: &[(DeviceIdentity, HistoryDevice)],
+) -> Option<u64> {
+    let ObservedRefusalReason::Explained(ModelRefusalReason::AllocationRecordNodeWall) = reason
+    else {
+        return None;
+    };
+    let root = answer?.root_whose_allocation_records_the_wall_counts(publishes_completed)?;
+    allocation_records_on_the_image_under(&image_of(devices), root)
+}
+
 impl HistoryPool {
     /// 起点：mkfs（加上同一个进程里的取号、暖机、第一个文件）。模型跟着走一遍，第一个文件那次发布拿实现的输出与模型比，判定一并交回。
     fn start(
@@ -727,14 +804,7 @@
 
     /// 两块盘此刻的整份镜像（拷一份，checker 与观察者读它）。
     fn image(&self) -> MemoryPool {
-        MemoryPool {
-            devices: self
-                .devices
-                .iter()
-                .map(|(identity, device)| (*identity, device.inner().image.clone()))
-                .collect(),
-            device_size_in_bytes: HISTORY_DEVICE_BYTES,
-        }
+        image_of(&self.devices)
     }
 }
 
@@ -1457,7 +1527,7 @@
         let counts = &self.model_counts;
         let _ = writeln!(
             text,
-            "  模型对拍 {} 步：该拒而拒 {}、区间里拒 {}、该成而成 {}；比过根 {} 条、分配记录 {} 条、冷启动内容 {} 次、抬 F 上限 {} 次；回退到 txg = F_生效 > 0 的根做成 {} 次",
+            "  模型对拍 {} 步：该拒而拒 {}、区间里拒 {}、该成而成 {}；比过根 {} 条、分配记录 {} 条、冷启动内容 {} 次、抬 F 上限 {} 次；回退到 txg = F_生效 > 0 的根做成 {} 次；分配记录墙按镜像上的真条数放行 {} 次",
             self.model_judged_steps,
             counts.required_refusals_matched,
             counts.permitted_refusals_taken,
@@ -1466,7 +1536,8 @@
             counts.allocation_records_compared,
             counts.cold_start_contents_compared,
             counts.ceilings_compared,
-            counts.rollbacks_accepted_at_the_effective_floor
+            counts.rollbacks_accepted_at_the_effective_floor,
+            counts.allocation_record_wall_refusals_over_one_node
         );
         for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
             let _ = writeln!(
@@ -1500,9 +1571,29 @@
     pub image: &'run MemoryPool,
 }
 
+fn placement_refusal_member(refusal: &PlacementRefusal) -> &'static str {
+    match refusal {
+        PlacementRefusal::NoFreeSlotOnAnyDevice => "NoFreeSlotOnAnyDevice",
+        PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { .. } => {
+            "SomeDevicesFullDeviceSetSelectionUndefined"
+        }
+        PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported { .. } => {
+            "UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported"
+        }
+        PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
+            ..
+        } => "CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined",
+    }
+}
+
 fn publish_error_member(error: &PublishError) -> String {
     let member = match error {
-        PublishError::NoSpaceFor { .. } => "NoSpaceFor",
+        PublishError::PlacementRefused { refusal, .. } => {
+            return format!(
+                "PublishError::PlacementRefused({})",
+                placement_refusal_member(refusal)
+            )
+        }
         PublishError::AllocationRecordsExceedOneNode { .. } => "AllocationRecordsExceedOneNode",
         PublishError::AccountingEntriesExceedOneNode { .. } => "AccountingEntriesExceedOneNode",
         PublishError::ReleaseNotInMapping { .. } => "ReleaseNotInMapping",
@@ -1572,17 +1663,13 @@
         MountError::Publish(cause) => {
             return format!("MountError::Publish({})", publish_error_member(cause))
         }
-        MountError::RollbackTargetNotInRing(_) => "RollbackTargetNotInRing",
-        MountError::RollbackTargetNotACandidate { reason, .. } => {
-            return format!("MountError::RollbackTargetNotACandidate（{reason}）")
+        MountError::RollbackTargetNotACandidate { exclusion, .. } => {
+            return format!("MountError::RollbackTargetNotACandidate({exclusion:?})")
         }
         MountError::RollbackFloorAboveCeiling { .. } => "RollbackFloorAboveCeiling",
         MountError::InstanceRowsOnVersionWithoutFileUnsupported { .. } => {
             "InstanceRowsOnVersionWithoutFileUnsupported"
         }
-        MountError::RollbackToVersionWithoutFileUnsupported(_) => {
-            "RollbackToVersionWithoutFileUnsupported"
-        }
         MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. } => {
             "VersionWithoutFileNotWrittenByMakeFilesystem"
         }
@@ -1739,25 +1826,45 @@
         &previous_record_bytes,
     ) {
         Ok(output) => settle_file_publish(session, model, answer, &records_before, output),
-        Err(error) => {
-            let member = publish_error_member(&error);
-            let verdict = judge_by_model(
-                model,
-                answer,
-                &observed_refusal(
-                    member.clone(),
-                    refusal_reason_of_publish_error(&error),
-                    0,
-                    stream_length_before,
-                    stream,
-                    None,
-                ),
-            );
-            AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
-        }
+        Err(error) => settle_refused_file_publish(
+            model,
+            answer,
+            &error,
+            devices,
+            stream,
+            stream_length_before,
+        ),
     }
 }
 
+/// 第一个文件、覆盖写被拒：成员映射成理由、分配记录墙时从镜像上数准入基数，交模型比。
+fn settle_refused_file_publish(
+    model: &mut IdealModel,
+    answer: Result<ModelAnswer, ModelDisagreement>,
+    error: &PublishError,
+    devices: &[(DeviceIdentity, HistoryDevice)],
+    stream: &SharedStream,
+    stream_length_before: usize,
+) -> AppliedStep {
+    let member = publish_error_member(error);
+    let reason = refusal_reason_of_publish_error(error);
+    let counted = allocation_records_counted_for_the_wall(reason, answer.as_ref().ok(), 0, devices);
+    let verdict = judge_by_model(
+        model,
+        answer,
+        &observed_refusal(
+            member.clone(),
+            reason,
+            0,
+            stream_length_before,
+            stream,
+            None,
+            counted,
+        ),
+    );
+    AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
+}
+
 /// 第一个文件、覆盖写做成之后：数复用、判分配代（第 1 件）、拿输出与模型比（第 2 件）、把现行版本换成这一版。
 fn settle_file_publish(
     session: &mut WritableSession,
@@ -1827,22 +1934,14 @@
         session.instance,
     ) {
         Ok(output) => settle_file_publish(session, model, answer, &records_before, output),
-        Err(error) => {
-            let member = publish_error_member(&error);
-            let verdict = judge_by_model(
-                model,
-                answer,
-                &observed_refusal(
-                    member.clone(),
-                    refusal_reason_of_publish_error(&error),
-                    0,
-                    stream_length_before,
-                    stream,
-                    None,
-                ),
-            );
-            AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
-        }
+        Err(error) => settle_refused_file_publish(
+            model,
+            answer,
+            &error,
+            devices,
+            stream,
+            stream_length_before,
+        ),
     }
 }
 
@@ -1909,6 +2008,7 @@
                     stream_length_before,
                     stream,
                     None,
+                    None,
                 ),
             );
             AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
@@ -2025,16 +2125,20 @@
         // 挂载半路报错（写行之后的发布失败）：第 1 件不比分配代，入口没交回写出去的那几次发布；模型按「拒之前一个字节都不写」判。
         Err(error) => {
             let member = mount_error_member(&error);
+            let reason = refusal_reason_of_mount_error(&error);
+            let counted =
+                allocation_records_counted_for_the_wall(reason, Some(&answer), 0, &pool.devices);
             let verdict = judge_by_model(
                 &mut pool.model,
                 Ok(answer),
                 &observed_refusal(
                     member.clone(),
-                    refusal_reason_of_mount_error(&error),
+                    reason,
                     0,
                     stream_length_before,
                     &pool.stream,
                     None,
+                    counted,
                 ),
             );
             AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
@@ -2206,16 +2310,24 @@
         // 模型比理由（容量墙按做完的次数判区间）与上限。
         Err(error) => {
             let member = mount_error_member(&error);
+            let reason = refusal_reason_of_mount_error(&error);
+            let counted = allocation_records_counted_for_the_wall(
+                reason,
+                answer.as_ref().ok(),
+                publishes_completed,
+                devices,
+            );
             let verdict = judge_by_model(
                 model,
                 answer,
                 &observed_refusal(
                     member.clone(),
-                    refusal_reason_of_mount_error(&error),
+                    reason,
                     publishes_completed,
                     stream_length_before,
                     stream,
                     reported_ceiling_of_mount_error(&error),
+                    counted,
                 ),
             );
             AppliedStep {
@@ -2318,19 +2430,55 @@
     }
 }
 
-/// 跑一段历史，录制流不留内容（第 1 件只看镜像）。
+/// 每一步之后跑不跑池级 checker。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum PerStepChecker {
+    /// 起点之后与每一步写过盘之后都跑，判红就停（第 1 件的执行器；快档、两个取样点与大档都是这一种）。
+    Run,
+    /// 不跑：只由理想模型、执行器自己的判定与 panic 让历史停下。给分配记录墙那一格（增补 3 第 2 件代码三方第一轮判决第三节第 1 条）：
+    /// 走到 812 条要在几次挂载里连发五十次左右，根环转过一圈之后 checker 在合法状态上判 I-3.1 红（已知红第 0 条），跑 checker 的历史
+    /// 到那里就停了，走不到墙。
+    Skipped,
+}
+
+impl PerStepChecker {
+    #[must_use]
+    pub fn name(self) -> &'static str {
+        match self {
+            PerStepChecker::Run => "每一步之后跑池级 checker",
+            PerStepChecker::Skipped => "不跑池级 checker（只由模型、执行器的判定与 panic 判）",
+        }
+    }
+}
+
+/// 跑一段历史，每一步之后跑池级 checker，录制流不留内容（第 1 件只看镜像）。
 #[must_use]
 pub fn execute_history(history: &GeneratedHistory) -> HistoryRun {
-    execute_history_observing(history, &SharedStream::new(), &mut |_| {})
+    execute_history_with(
+        history,
+        PerStepChecker::Run,
+        &SharedStream::new(),
+        &mut |_| {},
+    )
 }
 
-/// 跑一段历史：起点之后与每一步操作之后都对镜像跑池级 checker，再把那一刻的镜像交给观察者。第一次失败（违例、panic）就停。
-/// 盘上的写与屏障录进 `stream`（崩溃注入给开了内容保留的流）。
+/// 跑一段历史，每一步之后跑池级 checker（`execute_history_with` 的 `PerStepChecker::Run`）。
 pub fn execute_history_observing(
     history: &GeneratedHistory,
     stream: &SharedStream,
     observer: &mut dyn FnMut(&StepObservation<'_>),
 ) -> HistoryRun {
+    execute_history_with(history, PerStepChecker::Run, stream, observer)
+}
+
+/// 跑一段历史：起点之后与每一步写过盘之后按 `per_step_checker` 对镜像跑池级 checker，再把那一刻的镜像交给观察者。
+/// 第一次失败（违例、执行器判出的、模型对不上、panic）就停。盘上的写与屏障录进 `stream`（崩溃注入给开了内容保留的流）。
+pub fn execute_history_with(
+    history: &GeneratedHistory,
+    per_step_checker: PerStepChecker,
+    stream: &SharedStream,
+    observer: &mut dyn FnMut(&StepObservation<'_>),
+) -> HistoryRun {
     let mut tally = HistoryTally::default();
     let mut outcomes: Vec<StepOutcome> = Vec::new();
     let mut pool_slot: Option<HistoryPool> = None;
@@ -2341,7 +2489,10 @@
         let pool = pool_slot.insert(started_pool);
         let mut image = pool.image();
         let mut checked_stream_length = stream.operation_count();
-        let violations_after_the_starting_point = violations_on(&image, &mut tally);
+        let violations_after_the_starting_point = match per_step_checker {
+            PerStepChecker::Run => violations_on(&image, &mut tally),
+            PerStepChecker::Skipped => Vec::new(),
+        };
         if let Some(verdict) = &starting_verdict {
             tally.note_model_verdict(verdict);
         }
@@ -2402,7 +2553,10 @@
             } else {
                 let image_before_this_step = std::mem::replace(&mut image, pool.image());
                 checked_stream_length = stream_length;
-                violations = violations_on(&image, &mut tally);
+                violations = match per_step_checker {
+                    PerStepChecker::Run => violations_on(&image, &mut tally),
+                    PerStepChecker::Skipped => Vec::new(),
+                };
                 if !violations.is_empty() {
                     raised_floor_lands_only_on_abandoned = raised_floor.and_then(|new_floor| {
                         raised_floor_lands_only_on_abandoned_roots(
@@ -2722,13 +2876,23 @@
     kept
 }
 
-/// 一段失败的历史收缩到最短：起点不变，签名不变的删法与换法才留下。先截掉失败那一步之后的操作。
+/// 一段失败的历史收缩到最短：起点不变，签名不变的删法与换法才留下。先截掉失败那一步之后的操作。每一次重跑的 checker 与发现它的那一次
+/// 相同（`per_step_checker`）。
 #[must_use]
 pub fn shrink_failing_history(
     history: &GeneratedHistory,
     signature: &FailureSignature,
+    per_step_checker: PerStepChecker,
     worker_threads: usize,
 ) -> GeneratedHistory {
+    let run = |candidate: &GeneratedHistory| {
+        execute_history_with(
+            candidate,
+            per_step_checker,
+            &SharedStream::new(),
+            &mut |_| {},
+        )
+    };
     let still_fails = |operations: &[HistoryOperation]| {
         let candidate = GeneratedHistory {
             seed: history.seed,
@@ -2736,11 +2900,11 @@
             operations: operations.to_vec(),
         };
         matches!(
-            execute_history(&candidate).ending,
+            run(&candidate).ending,
             HistoryEnding::NewFinding { signature: found, .. } if found == *signature
         )
     };
-    let failing_length = match execute_history(history).ending {
+    let failing_length = match run(history).ending {
         HistoryEnding::NewFinding { observation, .. } => match observation.position {
             StepPosition::StartingPoint => 0,
             StepPosition::Operation(step_index) => step_index + 1,
@@ -2771,10 +2935,18 @@
 pub fn shrink_to_reproduction(
     history: &GeneratedHistory,
     signature: &FailureSignature,
+    per_step_checker: PerStepChecker,
     worker_threads: usize,
 ) -> ShrunkReproduction {
-    let shrunk_history = shrink_failing_history(history, signature, worker_threads);
-    let outcomes = execute_history(&shrunk_history).outcomes;
+    let shrunk_history =
+        shrink_failing_history(history, signature, per_step_checker, worker_threads);
+    let outcomes = execute_history_with(
+        &shrunk_history,
+        per_step_checker,
+        &SharedStream::new(),
+        &mut |_| {},
+    )
+    .outcomes;
     ShrunkReproduction {
         history: shrunk_history,
         outcomes,
@@ -2863,6 +3035,7 @@
     pub seed_count: u64,
     pub operations_per_history: usize,
     pub weights: GenerationWeights,
+    pub per_step_checker: PerStepChecker,
     pub tally: HistoryTally,
     /// 以「已知红」收尾的种子：(种子, 清单第几条, 在哪一步)。
     pub known_red_hits: Vec<(HistorySeed, usize, StepPosition)>,
@@ -2876,11 +3049,12 @@
         let mut text = String::new();
         let _ = writeln!(
             text,
-            "种子 [{}, {})，每段 {} 步，比重：{}",
+            "种子 [{}, {})，每段 {} 步，比重：{}；{}",
             self.first_seed,
             self.first_seed + self.seed_count,
             self.operations_per_history,
-            self.weights.name
+            self.weights.name,
+            self.per_step_checker.name()
         );
         text.push_str(&self.tally.render());
         for (form_index, form) in KNOWN_RED_FORMS.iter().enumerate() {
@@ -2916,6 +3090,7 @@
     seed_count: u64,
     operations_per_history: usize,
     weights: &GenerationWeights,
+    per_step_checker: PerStepChecker,
     worker_threads: usize,
     shrinking: FindingShrinking,
 ) -> CampaignReport {
@@ -2933,7 +3108,12 @@
                     operations_per_history,
                     weights,
                 );
-                let run = execute_history(&history);
+                let run = execute_history_with(
+                    &history,
+                    per_step_checker,
+                    &SharedStream::new(),
+                    &mut |_| {},
+                );
                 finished
                     .lock()
                     .expect("别的线程拿着这把锁时只做一次 push，不会在锁里 panic")
@@ -2976,9 +3156,12 @@
         .into_iter()
         .map(|(signature, (history, observation, seeds))| {
             let shrunk = match shrinking {
-                FindingShrinking::EveryFindingClass => {
-                    Some(shrink_to_reproduction(&history, &signature, worker_threads))
-                }
+                FindingShrinking::EveryFindingClass => Some(shrink_to_reproduction(
+                    &history,
+                    &signature,
+                    per_step_checker,
+                    worker_threads,
+                )),
                 FindingShrinking::ReportSeedsOnly => None,
             };
             NewFindingReport {
@@ -2995,6 +3178,7 @@
         seed_count,
         operations_per_history,
         weights: *weights,
+        per_step_checker,
         tally,
         known_red_hits,
         new_findings,
@@ -3198,7 +3382,7 @@
         );
         assert_eq!(
             harness_judgement_of_outcome(&StepOutcome::Refused {
-                member: "MountError::RollbackTargetNotInRing".to_string()
+                member: "MountError::RollbackTargetNotACandidate(NotInRing)".to_string()
             }),
             None
         );
diff -ruN '--exclude=target' crates/singlefs-harness/src/model_comparison.rs crates/singlefs-harness/src/model_comparison.rs
--- crates/singlefs-harness/src/model_comparison.rs	2026-09-19 05:34:04.944445328 +0000
+++ crates/singlefs-harness/src/model_comparison.rs	2026-09-19 11:49:11.937125487 +0000
@@ -2,12 +2,14 @@
 //! 把实现的错误成员映射到模型的拒绝理由。D13（验证路线） 已定项 5 管的是模型本身（`model.rs` 只 `use singlefs_format`）；
 //! 拿实现结局与模型比的这一层可以用 core 的类型。
 //!
-//! 映射只做「这个成员说的是哪条理由」，不做判断：成员说得出条款理由的映射过去，I/O、盘坏、走读失败这类模型里没有的成员一律
-//! `Unexplained`（不建崩溃与设备错的历史里它们都不该出现）。
+//! 映射只做「这个成员说的是哪条理由」，不做判断，按成员与它的判别字段映射、不看给人看的文字：成员说得出条款理由的映射过去，
+//! I/O、盘坏、走读失败、第一版不支持的池形状（小盘写满、各盘落点不一致）这类模型里没有的一律 `Unexplained`（不建崩溃与设备错、
+//! 两块等大盘的历史里它们都不该出现）。
 
 use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::allocator::PlacementRefusal;
 use singlefs_core::block_device::BlockDeviceError;
-use singlefs_core::mount::{InstanceRow, MountError, Mounted};
+use singlefs_core::mount::{InstanceRow, MountError, Mounted, RollbackCandidateExclusion};
 use singlefs_core::recovery::RecoveryOutcome;
 use singlefs_core::transaction::{
     PoolVersion, PublishError, TransactionOutput, TransactionUnit, ZeroUnitPublishOutput,
@@ -130,14 +132,16 @@
 }
 
 fn explained(reason: ModelRefusalReason) -> ObservedRefusalReason {
-    ObservedRefusalReason::Explained(vec![reason])
+    ObservedRefusalReason::Explained(reason)
 }
 
 /// 发布的错误成员说的是哪条理由。
 #[must_use]
 pub fn refusal_reason_of_publish_error(error: &PublishError) -> ObservedRefusalReason {
     match error {
-        PublishError::NoSpaceFor { .. } => explained(ModelRefusalReason::UnitAreaWall),
+        PublishError::PlacementRefused { refusal, .. } => {
+            refusal_reason_of_placement_refusal(refusal)
+        }
         PublishError::AllocationRecordsExceedOneNode { .. } => {
             explained(ModelRefusalReason::AllocationRecordNodeWall)
         }
@@ -159,6 +163,40 @@
     }
 }
 
+/// 分配器拒落点的原因说的是哪条理由：只有每块盘上都没有合政策的落点（容量不够）是单元区墙。小盘写满、各盘落点不一致是第一版不支持的
+/// 池形状，不是容量墙；模型的池是两块等大的盘、两盘的空闲图同样地变，模型里没有它的理由，出现就对不上（增补 3 第 2 件代码三方第一轮判决
+/// 第三节第 2 条：此前 `NoSpaceFor` 装着这四种，一律映射成单元区墙，单元区墙的区间一开就被接走）。
+#[must_use]
+pub fn refusal_reason_of_placement_refusal(refusal: &PlacementRefusal) -> ObservedRefusalReason {
+    match refusal {
+        PlacementRefusal::NoFreeSlotOnAnyDevice => explained(ModelRefusalReason::UnitAreaWall),
+        PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { .. }
+        | PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported { .. }
+        | PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
+            ..
+        } => ObservedRefusalReason::Unexplained,
+    }
+}
+
+/// 回退目标被挡下的那一条说的是哪条理由：按字段一对一映射，不看给人看的文字（增补 3 第 2 件代码三方第一轮判决第三节第 2 条）。
+#[must_use]
+pub fn refusal_reason_of_rollback_candidate_exclusion(
+    exclusion: RollbackCandidateExclusion,
+) -> ObservedRefusalReason {
+    explained(match exclusion {
+        RollbackCandidateExclusion::NotInRing => ModelRefusalReason::RollbackTargetNotInRing,
+        RollbackCandidateExclusion::BelowEffectiveFloor => {
+            ModelRefusalReason::RollbackTargetBelowEffectiveFloor
+        }
+        RollbackCandidateExclusion::OnAbandonedTimeline => {
+            ModelRefusalReason::RollbackTargetOnAbandonedTimeline
+        }
+        RollbackCandidateExclusion::TargetVersionWithoutFileUnsupported => {
+            ModelRefusalReason::RollbackToVersionWithoutFileUnsupported
+        }
+    })
+}
+
 /// 零单元发布只会报块设备错：内存盘不报错，模型里没有它的理由。
 #[must_use]
 pub fn refusal_reason_of_block_device_error(_error: &BlockDeviceError) -> ObservedRefusalReason {
@@ -177,23 +215,15 @@
         MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion => {
             explained(ModelRefusalReason::RaiseWithFormatTimeInstanceTableUnsupported)
         }
-        MountError::RollbackTargetNotInRing(_) => {
-            explained(ModelRefusalReason::RollbackTargetNotInRing)
+        MountError::RollbackTargetNotACandidate { exclusion, .. } => {
+            refusal_reason_of_rollback_candidate_exclusion(*exclusion)
         }
-        // 成员只说「不在候选集里」，两条理由之一（理由文字是给人看的，不拿来分流）。
-        MountError::RollbackTargetNotACandidate { .. } => ObservedRefusalReason::Explained(vec![
-            ModelRefusalReason::RollbackTargetBelowEffectiveFloor,
-            ModelRefusalReason::RollbackTargetOnAbandonedTimeline,
-        ]),
         MountError::RollbackFloorAboveCeiling { .. } => {
             explained(ModelRefusalReason::FloorAboveCeiling)
         }
         MountError::InstanceRowsOnVersionWithoutFileUnsupported { .. } => {
             explained(ModelRefusalReason::RowsOnVersionWithoutFileUnsupported)
         }
-        MountError::RollbackToVersionWithoutFileUnsupported(_) => {
-            explained(ModelRefusalReason::RollbackToVersionWithoutFileUnsupported)
-        }
         MountError::InstanceTableRowsExceedOnePageSecondPageUnsupported { .. } => {
             explained(ModelRefusalReason::InstanceTableOnePageWall)
         }
@@ -224,10 +254,8 @@
         | MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion
         | MountError::Acquisition(_)
         | MountError::Publish(_)
-        | MountError::RollbackTargetNotInRing(_)
         | MountError::RollbackTargetNotACandidate { .. }
         | MountError::InstanceRowsOnVersionWithoutFileUnsupported { .. }
-        | MountError::RollbackToVersionWithoutFileUnsupported(_)
         | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
         | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
         | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
@@ -257,6 +285,130 @@
 
 #[cfg(test)]
 mod tests {
+    use super::*;
+    use crate::model::{
+        IdealModel, ModelDeviceIdentity, ModelDisagreementAspect, ModelPoolGeometry,
+        ObservedOutcome,
+    };
+    use singlefs_core::address::{DeviceIdentity, SlotNumber};
+    use singlefs_core::allocator::CommitGeneratedDeviceAnswer;
+    use singlefs_core::mount::RollbackTarget;
+    use singlefs_core::transaction::TransactionUnit;
+    use singlefs_format::{SLOT_BYTES, UNIT_AREA_START_SLOT};
+
+    fn every_placement_refusal() -> [PlacementRefusal; 4] {
+        [
+            PlacementRefusal::NoFreeSlotOnAnyDevice,
+            PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined {
+                full_devices: vec![DeviceIdentity(1)],
+            },
+            PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported {
+                slot_per_device: vec![
+                    (DeviceIdentity(0), SlotNumber(50_184)),
+                    (DeviceIdentity(1), SlotNumber(50_182)),
+                ],
+            },
+            PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
+                answer_per_device: vec![
+                    (
+                        DeviceIdentity(0),
+                        CommitGeneratedDeviceAnswer::OpenEmptySegment(SlotNumber(196_672)),
+                    ),
+                    (
+                        DeviceIdentity(1),
+                        CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(SlotNumber(196_638)),
+                    ),
+                ],
+            },
+        ]
+    }
+
+    /// 单元区墙的区间开着时（单元区只有 640 槽的小盘，第一个文件之后占槽上界 13 × 64 > 640），发布的落点被拒：每块盘上都没有（容量不够）
+    /// 映射成单元区墙、模型放行；小盘写满、各盘落点不一致是第一版不支持的池形状，映射成模型没有的理由、判「模型说该成、实现拒了」
+    /// （增补 3 第 2 件代码三方第一轮判决第三节第 2 条：此前 `NoSpaceFor` 装着这四种、一律映射成单元区墙，区间一开就被接走）。
+    #[test]
+    fn only_a_placement_refused_on_every_device_passes_as_the_unit_area_wall() {
+        let mut model = IdealModel::after_make_filesystem(ModelPoolGeometry {
+            devices: vec![ModelDeviceIdentity(0), ModelDeviceIdentity(1)],
+            device_size_in_bytes: (UNIT_AREA_START_SLOT + 640) * SLOT_BYTES,
+        });
+        model.acquire_and_warm_up_in_the_make_filesystem_process();
+        let answer = model
+            .answer_publish_first_file(&[3])
+            .expect("会话开着、现行 txg 2");
+        assert!(
+            answer.required_refusals.is_empty(),
+            "条款不要求拒：{answer:?}"
+        );
+        for refusal in every_placement_refusal() {
+            let is_capacity = refusal == PlacementRefusal::NoFreeSlotOnAnyDevice;
+            let error = PublishError::PlacementRefused {
+                unit: TransactionUnit::Data,
+                refusal,
+            };
+            let observed = ObservedOutcome::Refused {
+                member: format!("{error:?}"),
+                reason: refusal_reason_of_publish_error(&error),
+                publishes_completed: 0,
+                wrote_anything: false,
+                reported_ceiling: None,
+                allocation_records_counted_on_the_image: None,
+            };
+            let judged = model
+                .clone()
+                .judge_and_advance(&answer, &observed)
+                .map(|_| ())
+                .map_err(|disagreement| disagreement.aspect);
+            if is_capacity {
+                assert_eq!(judged, Ok(()), "容量不够是单元区墙：{error:?}");
+            } else {
+                assert_eq!(
+                    judged,
+                    Err(ModelDisagreementAspect::RefusedWhenModelRequiresSuccess),
+                    "不是容量墙：{error:?}"
+                );
+            }
+        }
+    }
+
+    /// 回退目标被挡下的每一条映射到它自己那一条理由，四条互不相同（此前「不在候选集里」映射成「低于 F、被抛弃」两条之一）。
+    #[test]
+    fn each_rollback_candidate_exclusion_maps_to_its_own_reason() {
+        let target = RollbackTarget {
+            instance: InstanceGeneration(2),
+            checkpoint_txg: CheckpointTxg(7),
+        };
+        let mapped: Vec<ObservedRefusalReason> = [
+            RollbackCandidateExclusion::NotInRing,
+            RollbackCandidateExclusion::BelowEffectiveFloor,
+            RollbackCandidateExclusion::OnAbandonedTimeline,
+            RollbackCandidateExclusion::TargetVersionWithoutFileUnsupported,
+        ]
+        .into_iter()
+        .map(|exclusion| {
+            refusal_reason_of_mount_error(&MountError::RollbackTargetNotACandidate {
+                target,
+                exclusion,
+            })
+        })
+        .collect();
+        assert_eq!(
+            mapped,
+            vec![
+                ObservedRefusalReason::Explained(ModelRefusalReason::RollbackTargetNotInRing),
+                ObservedRefusalReason::Explained(
+                    ModelRefusalReason::RollbackTargetBelowEffectiveFloor
+                ),
+                ObservedRefusalReason::Explained(
+                    ModelRefusalReason::RollbackTargetOnAbandonedTimeline
+                ),
+                ObservedRefusalReason::Explained(
+                    ModelRefusalReason::RollbackToVersionWithoutFileUnsupported
+                ),
+            ]
+        );
+    }
+
     /// 模型模块只用格式常量那一个 crate（D13（验证路线） 已定项 5）：`model.rs` 里注释之外的每一行都不提 `singlefs_core`、`singlefs_checker`，
     /// `use` 只有 `std` 与 `singlefs_format`（测试模块的 `use super::*` 除外）。胶水（这个文件）可以用 core。
     #[test]
diff -ruN '--exclude=target' crates/singlefs-harness/src/model.rs crates/singlefs-harness/src/model.rs
--- crates/singlefs-harness/src/model.rs	2026-09-19 05:33:52.758621783 +0000
+++ crates/singlefs-harness/src/model.rs	2026-09-19 11:49:11.936125498 +0000
@@ -10,6 +10,11 @@
 //!
 //! 条款把答案留给实现取上界的地方（容量墙），模型答「允许拒绝的区间」，不照抄实现的上界算法；打回重议那几处（增补 2 收口表第 ①、② 行）
 //! 照代码今天的读法写，每一处标「预想，跟收口表第 X 行」。
+//!
+//! 模型不记落点：单元落在哪个槽、分配记录写得对不对（回收门槛、释放时改没改写记录、复用时罩住的槽删没删）归池级 checker 判，
+//! 模型只拿实现交回的每个单元那几条记录比「每块盘一条、仍分配、分配代等于写它的那次发布」（增补 3 第 2 件代码三方第一轮判决第一节 M4 那一格：
+//! 回收门槛差一、释放时不改写记录这几条变异，只留模型时三段都判不出，checker 都判红）。分配记录的真条数模型同样不记：分配记录墙拒时，
+//! 执行器按 checker 的解析从镜像上现数，交给模型判区间的下端（同一判决第三节第 1 条）。
 
 use std::collections::{BTreeMap, BTreeSet};
 use std::rc::Rc;
@@ -163,6 +168,9 @@
     /// 这一版之后分配记录条数的上界：mkfs 两个单元每盘各一条，沿这一版的来路每次发布每个重写的角色每盘至多加一条
     /// （D3（空间分配） 已定项 7：一条记一个单元、释放只改写不删）。回收、复用只会让真数比它小。
     pub allocation_records_upper_bound: u64,
+    /// 写出这一版的那次发布按准入的口径新增几条分配记录：重写的每个角色每盘一条，不抵扣会被复用的已回收记录（增补 2 收口表第 39 行，
+    /// 2026-09-18 用户定保留这个上界准入）。第 0 代根记 mkfs 写的两个单元每盘一条。
+    pub allocation_records_added_by_its_publish: u64,
     /// 同样口径的每盘已占槽数上界：沿来路每次发布写出的单元的槽数之和，释放与回收一概不扣。
     pub occupied_slots_upper_bound_per_device: u64,
 }
@@ -321,10 +329,12 @@
     ColdStartRecover,
 }
 
-/// 一步里计划的一次发布之后的两个上界（容量墙的区间按它判）。
+/// 一步里计划的一次发布之后的两个上界（容量墙区间的上端按它判），与这次发布按准入口径新增的分配记录条数（分配记录墙区间的下端：
+/// 从镜像上现数的基数加上它）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct PlannedPublishUpperBounds {
     allocation_records: u64,
+    allocation_records_added_by_the_admission: u64,
     occupied_slots_per_device: u64,
 }
 
@@ -347,9 +357,29 @@
     planned_publish_upper_bounds: Vec<PlannedPublishUpperBounds>,
     /// 容量墙在第一次写之前一串判完（可写挂载、回退）还是逐次发布判（发布、抬 F）。
     walls_are_judged_before_the_first_write: bool,
+    /// 这一步接在哪一版后面：发布与抬 F 是会话的现行版本，可写挂载是所选根，回退是目标根；冷启动与候选集外的回退没有。
+    starting_version: Option<ModelRootKey>,
     session_after_success: ModelSessionAfterSuccess,
 }
 
+impl ModelAnswer {
+    /// 分配记录墙拒在做完 `publishes_completed` 次发布之后时，执行器在镜像上数哪一条根下的分配记录：逐次判的（发布、抬 F）数拒之前
+    /// 最后写出的那一版（一次都没做完就是这一步的起点），一串判完的（可写挂载、回退）数这一步的起点——实现的准入基数就是这两处的
+    /// 分配器条数（`transaction::publish_admission`、`transaction::publish_sequence_admission`）。
+    #[must_use]
+    pub fn root_whose_allocation_records_the_wall_counts(
+        &self,
+        publishes_completed: usize,
+    ) -> Option<ModelRootKey> {
+        if self.walls_are_judged_before_the_first_write || publishes_completed == 0 {
+            return self.starting_version;
+        }
+        self.expected_roots
+            .get(publishes_completed - 1)
+            .map(|root| root.key)
+    }
+}
+
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum ModelSessionAfterSuccess {
     /// 会话照旧开着，现行版本换成最后写出的那条根。
@@ -411,10 +441,12 @@
     },
 }
 
-/// 实现的错误成员映射到模型的理由：`Explained` 里是「这个成员说的是这几条之一」，`Unexplained` 是模型没有对应理由的成员（I/O、盘坏……）。
-#[derive(Clone, Debug, PartialEq, Eq)]
+/// 实现的错误成员映射到模型的理由：`Explained` 里是这个成员说的那一条，`Unexplained` 是模型没有对应理由的成员（I/O、盘坏、
+/// 第一版不支持的池形状……）。一个成员只映射到一条：此前回退「不在候选集里」映射成「两条之一」，实现报错了是哪一条模型分不出
+/// （增补 3 第 2 件代码三方第一轮判决第三节第 2 条）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum ObservedRefusalReason {
-    Explained(Vec<ModelRefusalReason>),
+    Explained(ModelRefusalReason),
     Unexplained,
 }
 
@@ -431,6 +463,9 @@
         wrote_anything: bool,
         /// 实现报的上限（抬 F 被上限拒时）。
         reported_ceiling: Option<ModelCheckpointTxg>,
+        /// 理由是分配记录墙时，执行器按 checker 的解析在镜像上数的准入基数：`ModelAnswer::root_whose_allocation_records_the_wall_counts`
+        /// 点名的那条根下有几条分配记录。别的理由不数；数不出（镜像上找不到那条根、树读不出）也是 None，这时模型不放行分配记录墙。
+        allocation_records_counted_on_the_image: Option<u64>,
     },
 }
 
@@ -513,6 +548,8 @@
     pub ceilings_compared: u64,
     /// 回退做成、目标的 txg 正好等于 F_生效且 F_生效 > 0（B2 那一格跑到了）。
     pub rollbacks_accepted_at_the_effective_floor: u64,
+    /// 分配记录墙拒、镜像上数的真条数越过一个节点而放行的次数（墙那一格的下端真的判过）。
+    pub allocation_record_wall_refusals_over_one_node: u64,
 }
 
 impl ModelJudgementCounts {
@@ -527,6 +564,8 @@
         self.ceilings_compared += other.ceilings_compared;
         self.rollbacks_accepted_at_the_effective_floor +=
             other.rollbacks_accepted_at_the_effective_floor;
+        self.allocation_record_wall_refusals_over_one_node +=
+            other.allocation_record_wall_refusals_over_one_node;
     }
 }
 
@@ -570,6 +609,7 @@
                 (ModelUnitRole::TreeTable, MAKE_FILESYSTEM_TXG),
             ]),
             allocation_records_upper_bound: 2 * device_count,
+            allocation_records_added_by_its_publish: 2 * device_count,
             occupied_slots_upper_bound_per_device: ModelUnitRole::InstanceTable.span_in_slots()
                 + ModelUnitRole::TreeTable.span_in_slots(),
         };
@@ -745,6 +785,8 @@
             role_written_at.insert(*role, checkpoint_txg);
             slots_written += role.span_in_slots();
         }
+        let allocation_records_added =
+            u64::try_from(rewritten.len()).expect("至多九个角色") * device_count;
         let file = match new_file_content {
             Some(content) => Some(ModelFileVersion {
                 written_by: key,
@@ -760,7 +802,8 @@
             instance_table_rows,
             role_written_at,
             allocation_records_upper_bound: previous.allocation_records_upper_bound
-                + u64::try_from(rewritten.len()).expect("至多九个角色") * device_count,
+                + allocation_records_added,
+            allocation_records_added_by_its_publish: allocation_records_added,
             occupied_slots_upper_bound_per_device: previous.occupied_slots_upper_bound_per_device
                 + slots_written,
         }
@@ -771,6 +814,8 @@
             .iter()
             .map(|root| PlannedPublishUpperBounds {
                 allocation_records: root.allocation_records_upper_bound,
+                allocation_records_added_by_the_admission: root
+                    .allocation_records_added_by_its_publish,
                 occupied_slots_per_device: root.occupied_slots_upper_bound_per_device,
             })
             .collect()
@@ -819,6 +864,7 @@
         );
         Ok(self.answer_for_publishes(
             ModelOperationKind::PublishFirstFile,
+            current.key,
             required_refusals,
             BTreeSet::new(),
             vec![root],
@@ -859,6 +905,7 @@
         );
         Ok(self.answer_for_publishes(
             ModelOperationKind::PublishOverwrite,
+            current.key,
             required_refusals,
             BTreeSet::new(),
             vec![root],
@@ -891,6 +938,7 @@
         );
         Ok(self.answer_for_publishes(
             ModelOperationKind::PublishWithoutUnits,
+            current.key,
             BTreeSet::new(),
             BTreeSet::new(),
             vec![root],
@@ -900,6 +948,7 @@
     fn answer_for_publishes(
         &self,
         operation: ModelOperationKind,
+        starting_version: ModelRootKey,
         mut required_refusals: BTreeSet<ModelRefusalReason>,
         permitted_refusals: BTreeSet<ModelRefusalReason>,
         roots: Vec<ModelRoot>,
@@ -922,6 +971,7 @@
             expected_read_back: None,
             rollback_floor_ceiling: None,
             walls_are_judged_before_the_first_write: false,
+            starting_version: Some(starting_version),
             session_after_success: ModelSessionAfterSuccess::StaysOpen,
         }
     }
@@ -957,6 +1007,7 @@
             rollback_floor_ceiling: None,
             planned_publish_upper_bounds: Vec::new(),
             walls_are_judged_before_the_first_write: true,
+            starting_version: None,
             session_after_success: ModelSessionAfterSuccess::Closed,
         };
         let Some(target_root) = self.root_with_key(target) else {
@@ -1097,8 +1148,13 @@
             ));
             warm_up_publishes += 1;
         }
-        let mut answer =
-            self.answer_for_publishes(operation, required_refusals, BTreeSet::new(), roots);
+        let mut answer = self.answer_for_publishes(
+            operation,
+            base.key,
+            required_refusals,
+            BTreeSet::new(),
+            roots,
+        );
         answer.walls_are_judged_before_the_first_write = true;
         answer.expected_mount = Some((instance, rows_to_write));
         answer.session_after_success = ModelSessionAfterSuccess::OpenedAs(instance);
@@ -1187,6 +1243,7 @@
         }
         let mut answer = self.answer_for_publishes(
             ModelOperationKind::RaiseRollbackFloor,
+            current.key,
             required_refusals,
             permitted_refusals,
             roots,
@@ -1216,17 +1273,21 @@
             rollback_floor_ceiling: None,
             planned_publish_upper_bounds: Vec::new(),
             walls_are_judged_before_the_first_write: true,
+            starting_version: None,
             session_after_success: ModelSessionAfterSuccess::Closed,
         }
     }
 
     /// 容量墙的区间（收口表第 39 行那种：条款把答案留给实现取上界，模型答允许拒绝的区间）：
-    /// - 分配记录树（预想，跟收口表第 39 行）：下端（必须拒）是真条数装不下一个节点——模型不知道真条数，这一端不判（装不下还去写会 panic，
-    ///   由第 1 件判）；上端（允许拒）是这一步计划的那次发布之后、沿来路每次发布每个角色每盘都新加一条的上界超过一个节点：一条记一个单元、
-    ///   释放只改写不删（D3（空间分配） 已定项 7），条目 20 字节、key 10（D3（空间分配） 已定项 11），节点 16 KiB 减头（D8（核心索引结构）
-    ///   已定项 11）⇒ 812 条；「每个角色每盘新增一条」是 2026-09-18 用户定保留的上界准入。模型的上界沿来路累加、不看分配器此刻的条数，
-    ///   与实现「此刻条数 + 这次新增」不是同一个算法，只会更宽。
-    /// - 单元区：下端同样不判；上端是占槽上界 × 一个聚簇段的槽数超过单元区（每个落点最坏独占一段：已分配 + defer ≤ 2 × 上界，
+    /// - 分配记录树（预想，跟收口表第 39 行）：允许拒要两头都过。上界这一头：这一步计划的那次发布之后、沿来路每次发布每个角色每盘都新加一条的
+    ///   上界超过一个节点——一条记一个单元、释放只改写不删（D3（空间分配） 已定项 7），条目 20 字节、key 10（D3（空间分配） 已定项 11），
+    ///   节点 16 KiB 减头（D8（核心索引结构） 已定项 11）⇒ 812 条；模型的上界沿来路累加、不看分配器此刻的条数，只会比真数宽。
+    ///   真条数这一头：执行器按 checker 的解析在镜像上现数的准入基数（`ModelAnswer::root_whose_allocation_records_the_wall_counts` 点名的那一版），
+    ///   加上计划里到那一次为止每次按准入口径新增的条数（每个重写的角色每盘一条，2026-09-18 用户定保留的上界准入），超过 812 才算装不下；
+    ///   真条数 ≤ 812 而实现拒了是对不上，数不出基数也不放行（增补 3 第 2 件代码三方第一轮判决第三节第 1 条：此前只有上界那一头，
+    ///   宽到接得住「差一」的误拒——攻方把墙的 `>` 改成 `>=`，长历史里在条款说装得下的格上拒了 44 次，那一刻上界 1376–1778，一次都没判出）。
+    ///   必须拒那一头（真条数装不下而实现做成了）不判：装不下还去写会 panic，由第 1 件判。
+    /// - 单元区：真条数那一头不判；上界那一头是占槽上界 × 一个聚簇段的槽数超过单元区（每个落点最坏独占一段：已分配 + defer ≤ 2 × 上界，
     ///   保留池 10 + 7 c_max 与切换预留（D16（发布语义） 已定项 1；D28（挂载期承诺量） 已定项 3）在 64 倍里）。预想：D28 已定项 1 的准入式子
     ///   第一版没实现，各项没有现值。
     fn capacity_wall_is_permitted(
@@ -1234,15 +1295,40 @@
         reason: ModelRefusalReason,
         answer: &ModelAnswer,
         publishes_completed: usize,
+        allocation_records_counted_on_the_image: Option<u64>,
     ) -> bool {
-        let exceeds = |bounds: &PlannedPublishUpperBounds| match reason {
+        // 一串判完的（可写挂载、回退）看计划里的每一次；逐次判的（发布、抬 F）只看拒的那一次。
+        let judged_publishes: &[PlannedPublishUpperBounds] =
+            if answer.walls_are_judged_before_the_first_write {
+                &answer.planned_publish_upper_bounds
+            } else {
+                answer
+                    .planned_publish_upper_bounds
+                    .get(publishes_completed..=publishes_completed)
+                    .unwrap_or(&[])
+            };
+        match reason {
             ModelRefusalReason::AllocationRecordNodeWall => {
-                bounds.allocation_records > Self::allocation_record_node_capacity()
+                let capacity = Self::allocation_record_node_capacity();
+                let upper_bound_exceeds = judged_publishes
+                    .iter()
+                    .any(|planned| planned.allocation_records > capacity);
+                let true_count_exceeds =
+                    allocation_records_counted_on_the_image.is_some_and(|counted| {
+                        judged_publishes
+                            .iter()
+                            .scan(counted, |records, planned| {
+                                *records += planned.allocation_records_added_by_the_admission;
+                                Some(*records)
+                            })
+                            .any(|records_after_the_publish| records_after_the_publish > capacity)
+                    });
+                upper_bound_exceeds && true_count_exceeds
             }
-            ModelRefusalReason::UnitAreaWall => {
-                bounds.occupied_slots_per_device * CLUSTER_SEGMENT_SLOTS
+            ModelRefusalReason::UnitAreaWall => judged_publishes.iter().any(|planned| {
+                planned.occupied_slots_per_device * CLUSTER_SEGMENT_SLOTS
                     > self.unit_area_slots_per_device()
-            }
+            }),
             ModelRefusalReason::ContentExceedsDataUnitPayload
             | ModelRefusalReason::FirstFileNotRightAfterTheWarmUp
             | ModelRefusalReason::AccountingNodeWall
@@ -1255,14 +1341,6 @@
             | ModelRefusalReason::RollbackToVersionWithoutFileUnsupported
             | ModelRefusalReason::FloorAboveCeiling
             | ModelRefusalReason::RaiseWithFormatTimeInstanceTableUnsupported => false,
-        };
-        if answer.walls_are_judged_before_the_first_write {
-            answer.planned_publish_upper_bounds.iter().any(exceeds)
-        } else {
-            answer
-                .planned_publish_upper_bounds
-                .get(publishes_completed)
-                .is_some_and(exceeds)
         }
     }
 
@@ -1283,32 +1361,39 @@
                 publishes_completed,
                 wrote_anything,
                 reported_ceiling,
+                allocation_records_counted_on_the_image,
             } => {
                 self.judge_reported_ceiling(answer, *reported_ceiling, &mut counts)?;
-                let explained_reasons: &[ModelRefusalReason] = match reason {
-                    ObservedRefusalReason::Explained(reasons) => reasons,
-                    ObservedRefusalReason::Unexplained => &[],
+                let accepted = match reason {
+                    ObservedRefusalReason::Explained(candidate) => {
+                        Some(*candidate).filter(|candidate| {
+                            answer.required_refusals.contains(candidate)
+                                || answer.permitted_refusals.contains(candidate)
+                                || (candidate.is_capacity_wall_with_an_interval()
+                                    && self.capacity_wall_is_permitted(
+                                        *candidate,
+                                        answer,
+                                        *publishes_completed,
+                                        *allocation_records_counted_on_the_image,
+                                    ))
+                        })
+                    }
+                    ObservedRefusalReason::Unexplained => None,
                 };
-                let accepted = explained_reasons.iter().copied().find(|candidate| {
-                    answer.required_refusals.contains(candidate)
-                        || answer.permitted_refusals.contains(candidate)
-                        || (candidate.is_capacity_wall_with_an_interval()
-                            && self.capacity_wall_is_permitted(
-                                *candidate,
-                                answer,
-                                *publishes_completed,
-                            ))
-                });
                 let Some(accepted_reason) = accepted else {
                     let aspect = if answer.required_refusals.is_empty() {
                         ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
                     } else {
                         ModelDisagreementAspect::RefusalReason
                     };
+                    let counted_on_the_image = match allocation_records_counted_on_the_image {
+                        Some(counted) => format!("（镜像上数的准入基数 {counted} 条）"),
+                        None => String::new(),
+                    };
                     return Err(ModelDisagreement::new(
                         aspect,
                         describe_answer(answer),
-                        format!("拒了：{member}"),
+                        format!("拒了：{member}{counted_on_the_image}"),
                     ));
                 };
                 let partial_publishes_allowed = !answer.walls_are_judged_before_the_first_write
@@ -1339,6 +1424,9 @@
                 } else {
                     counts.permitted_refusals_taken += 1;
                 }
+                if accepted_reason == ModelRefusalReason::AllocationRecordNodeWall {
+                    counts.allocation_record_wall_refusals_over_one_node += 1;
+                }
                 for root in completed {
                     self.write_root(root.clone());
                 }
@@ -1877,4 +1965,136 @@
             .expect_err("照抄的数据单元分配代写成了这次的 txg");
         assert_eq!(wrong.aspect, ModelDisagreementAspect::AllocationGeneration);
     }
+
+    fn refused_by_the_allocation_record_wall(
+        publishes_completed: usize,
+        allocation_records_counted_on_the_image: Option<u64>,
+    ) -> ObservedOutcome {
+        ObservedOutcome::Refused {
+            member: "PublishError::AllocationRecordsExceedOneNode".to_string(),
+            reason: ObservedRefusalReason::Explained(ModelRefusalReason::AllocationRecordNodeWall),
+            publishes_completed,
+            wrote_anything: publishes_completed > 0,
+            reported_ceiling: None,
+            allocation_records_counted_on_the_image,
+        }
+    }
+
+    /// 在第二个实例里再覆盖写 50 次：沿来路的上界 102 + 50 × 16 = 902，远过 812（区间的上界那一头早就开了）。
+    fn model_with_the_upper_bound_far_above_one_allocation_node() -> IdealModel {
+        let mut model = model_after_four_overwrites_in_a_second_instance();
+        for content_byte in 0_u8..50 {
+            let overwrite = model
+                .answer_publish_overwrite(&[content_byte])
+                .expect("会话开着、现行版本带文件");
+            succeed(&mut model, &overwrite);
+        }
+        model
+    }
+
+    /// 分配记录墙（增补 3 第 2 件代码三方第一轮判决第三节第 1 条）：覆盖写每盘加 8 条；镜像上数的基数 796 ⇒ 这次之后正好 812 条、一个节点装得下，
+    /// 墙拒是「模型说该成、实现拒了」，哪怕沿来路的上界早过了 812；基数 797 ⇒ 813 条，放行；数不出基数不放行。
+    #[test]
+    fn the_allocation_record_wall_is_permitted_only_when_the_counted_records_plus_this_publish_exceed_812(
+    ) {
+        let model = model_with_the_upper_bound_far_above_one_allocation_node();
+        let answer = model
+            .answer_publish_overwrite(&[1])
+            .expect("会话开着、现行版本带文件");
+        assert_eq!(IdealModel::allocation_record_node_capacity(), 812);
+        assert!(
+            answer.planned_publish_upper_bounds[0].allocation_records > 812,
+            "上界那一头开着：{:?}",
+            answer.planned_publish_upper_bounds
+        );
+        assert_eq!(
+            answer.planned_publish_upper_bounds[0].allocation_records_added_by_the_admission,
+            16
+        );
+        assert_eq!(
+            answer.root_whose_allocation_records_the_wall_counts(0),
+            Some(model.session.as_ref().expect("会话开着").current.key),
+            "覆盖写数现行版本那一版"
+        );
+        let judged = |counted: Option<u64>| {
+            model
+                .clone()
+                .judge_and_advance(&answer, &refused_by_the_allocation_record_wall(0, counted))
+                .map(|counts| counts.allocation_record_wall_refusals_over_one_node)
+                .map_err(|disagreement| disagreement.aspect)
+        };
+        assert_eq!(
+            judged(Some(796)),
+            Err(ModelDisagreementAspect::RefusedWhenModelRequiresSuccess)
+        );
+        assert_eq!(judged(Some(797)), Ok(1));
+        assert_eq!(
+            judged(None),
+            Err(ModelDisagreementAspect::RefusedWhenModelRequiresSuccess)
+        );
+    }
+
+    /// 可写挂载在第一次写之前一串判完：基数是所选根那一版，加上写行（五个角色每盘一条）与每次暖机（四个角色每盘一条）；一串里有一次
+    /// 越过 812 才放行。抬 F 逐次判：做完一次之后被拒，基数数拒之前最后写出的那一版。
+    #[test]
+    fn mount_and_raise_count_the_wall_from_the_version_their_admission_starts_from() {
+        let mut model = model_with_the_upper_bound_far_above_one_allocation_node();
+        let raise = model
+            .answer_raise_rollback_floor(ModelCheckpointTxg(0))
+            .expect("会话开着、F 不往下抬");
+        assert!(
+            raise.expected_roots.len() >= 2,
+            "{:?}",
+            raise.expected_roots
+        );
+        assert_eq!(
+            raise.root_whose_allocation_records_the_wall_counts(1),
+            Some(raise.expected_roots[0].key),
+            "做完一次之后被拒：数第一次抬 F 写出的那一版"
+        );
+        let second_publish_added =
+            raise.planned_publish_upper_bounds[1].allocation_records_added_by_the_admission;
+        assert_eq!(second_publish_added, 8);
+        let judged_raise = |counted: u64| {
+            model
+                .clone()
+                .judge_and_advance(
+                    &raise,
+                    &refused_by_the_allocation_record_wall(1, Some(counted)),
+                )
+                .map_err(|disagreement| disagreement.aspect)
+                .is_ok()
+        };
+        assert!(!judged_raise(812 - second_publish_added));
+        assert!(judged_raise(813 - second_publish_added));
+
+        let chosen = model.newest_root().key;
+        model.close_session();
+        let mount = model.answer_mount_writable();
+        assert_eq!(
+            mount.root_whose_allocation_records_the_wall_counts(0),
+            Some(chosen)
+        );
+        let added_by_the_mount: u64 = mount
+            .planned_publish_upper_bounds
+            .iter()
+            .map(|planned| planned.allocation_records_added_by_the_admission)
+            .sum();
+        assert_eq!(
+            added_by_the_mount,
+            10 + 8 * u64::try_from(mount.planned_publish_upper_bounds.len() - 1).expect("次数"),
+            "写行每盘 5 条、每次暖机每盘 4 条"
+        );
+        let judged_mount = |counted: u64| {
+            model
+                .clone()
+                .judge_and_advance(
+                    &mount,
+                    &refused_by_the_allocation_record_wall(0, Some(counted)),
+                )
+                .is_ok()
+        };
+        assert!(!judged_mount(812 - added_by_the_mount));
+        assert!(judged_mount(813 - added_by_the_mount));
+    }
 }
diff -ruN '--exclude=target' crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs
--- crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs	2026-09-17 09:49:50.152692591 +0000
+++ crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs	2026-09-19 11:37:39.615582607 +0000
@@ -12,7 +12,8 @@
 use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
 use singlefs_core::block_device::{BlockDevice, WriteDurability};
 use singlefs_core::mount::{
-    mount_rollback, mount_writable, InstanceRow, MountError, Mounted, RollbackTarget, ShadowLedger,
+    mount_rollback, mount_writable, InstanceRow, MountError, Mounted, RollbackCandidateExclusion,
+    RollbackTarget, ShadowLedger,
 };
 use singlefs_core::recovery::{
     choose_root, choose_superblock, recover, replay_journal, scan_journal, JournalPolicy,
@@ -82,7 +83,8 @@
 }
 
 /// 回退到树表 0 条的根（第一个事务里 txg 2 的暖机根），第一版不支持（设计没定），在任何写之前拒绝：第一个事务之后进程退出、重开回退到 (1, 2)
-/// ⇒ 返回 `RollbackToVersionWithoutFileUnsupported`；两盘超级块槽逐字节不变、根环没有新根、录制流一步都没多。
+/// ⇒ 返回 `RollbackTargetNotACandidate`、被挡下的是 `TargetVersionWithoutFileUnsupported`；两盘超级块槽逐字节不变、根环没有新根、
+/// 录制流一步都没多。
 #[test]
 fn rolling_back_to_a_warm_up_root_without_a_file_version_is_refused_before_any_write() {
     let mut pool = build_pool("step-four-rollback-to-warm-up-root");
@@ -97,7 +99,10 @@
     assert!(
         matches!(
             refused,
-            Err(MountError::RollbackToVersionWithoutFileUnsupported(target)) if target == warm_up_root
+            Err(MountError::RollbackTargetNotACandidate {
+                target,
+                exclusion: RollbackCandidateExclusion::TargetVersionWithoutFileUnsupported,
+            }) if target == warm_up_root
         ),
         "暖机根下面没有文件版本：{:?}",
         refused.as_ref().err()
@@ -286,27 +291,32 @@
     }
 }
 
-/// 回退候选集（D23 已定项 14）：实例表里有行 (i, Ti, Wi) 的实例，只有 T ≤ Ti 的根可选——B 的根 (1, 4) 与 C 的根 (2, 8) 都是被抛弃时间线的；
-/// 根环里没有的 (1, 42) 另报。
+/// 回退候选集（D23 已定项 14）：实例表里有行 (i, Ti, Wi) 的实例，只有 T ≤ Ti 的根可选——B 的根 (1, 4) 与 C 的根 (2, 8) 都是被抛弃时间线的
+/// （F 还是 0，不低于 F），报 `OnAbandonedTimeline`；根环里没有的 (1, 42) 报 `NotInRing`。调用方按这个字段分流，不看文字。
 #[test]
 fn rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused() {
     let mut pool = build_through_third_publish("step-four-refused");
     rollback_to_first_root(&mut pool, ShadowLedger::On);
+    let missing = RollbackTarget {
+        instance: InstanceGeneration(1),
+        checkpoint_txg: CheckpointTxg(42),
+    };
     for (target, expected) in [
         (
             RollbackTarget {
                 instance: InstanceGeneration(1),
                 checkpoint_txg: CheckpointTxg(4),
             },
-            "实例表里那个实例的行 T 更小：这是被抛弃时间线的根",
+            RollbackCandidateExclusion::OnAbandonedTimeline,
         ),
         (
             RollbackTarget {
                 instance: InstanceGeneration(2),
                 checkpoint_txg: CheckpointTxg(8),
             },
-            "实例表里那个实例的行 T 更小：这是被抛弃时间线的根",
+            RollbackCandidateExclusion::OnAbandonedTimeline,
         ),
+        (missing, RollbackCandidateExclusion::NotInRing),
     ] {
         let mut devices = pool.reopen_recorded();
         let refused = mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On);
@@ -314,25 +324,14 @@
         match refused {
             Err(MountError::RollbackTargetNotACandidate {
                 target: reported,
-                reason,
-            }) => assert_eq!((reported, reason), (target, expected)),
+                exclusion,
+            }) => assert_eq!((reported, exclusion), (target, expected)),
             other => panic!(
                 "{target:?} 该被拒：{:?}",
                 other.map(|mounted| mounted.output.instance)
             ),
         }
     }
-    let mut devices = pool.reopen_recorded();
-    let missing = RollbackTarget {
-        instance: InstanceGeneration(1),
-        checkpoint_txg: CheckpointTxg(42),
-    };
-    let refused = mount_rollback(&parameters(), &mut devices, missing, ShadowLedger::On);
-    pool.devices = Some(devices);
-    assert!(
-        matches!(refused, Err(MountError::RollbackTargetNotInRing(reported)) if reported == missing),
-        "根环里没有 (1, 42)"
-    );
 }
 
 /// C314（回退可以复用被抛弃的根引用的单元） 那一格的必红，影子账开关强制进入：关掉影子账，回退之后再发两版文件，
diff -ruN '--exclude=target' crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs
--- crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs	2026-09-18 00:04:13.333721021 +0000
+++ crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs	2026-09-19 11:38:40.188752990 +0000
@@ -14,7 +14,7 @@
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
-use singlefs_core::allocator::UnitFootprint;
+use singlefs_core::allocator::{PlacementRefusal, UnitFootprint};
 use singlefs_core::journal::{back_chain_of, record_offset};
 use singlefs_core::records::{
     build_mapping_entry, parse_mapping_entry, STATISTIC_ALLOCATED_BYTES,
@@ -710,7 +710,7 @@
 }
 
 /// 准入之后失败的发布不留半新的池：把 A 开的那个提交内生段用到头、单元区里别的空槽全标成已分配、只留 50182–50183 给 B 的数据单元，
-/// B 释放了 A、分配到了数据单元、第一个提交内生块拿不到 ⇒ `NoSpaceFor`；返回之后分配器要和进去之前一模一样。
+/// B 释放了 A、分配到了数据单元、第一个提交内生块拿不到 ⇒ 落点被拒（`PlacementRefused`，每块盘上都没有）；返回之后分配器要和进去之前一模一样。
 #[test]
 fn publish_running_out_of_space_midway_leaves_the_allocator_as_it_was() {
     let mut pool = build_pool("no-space-midway");
@@ -750,11 +750,12 @@
     assert!(
         matches!(
             result,
-            Err(PublishError::NoSpaceFor {
-                unit: TransactionUnit::ExtentRoot
+            Err(PublishError::PlacementRefused {
+                unit: TransactionUnit::ExtentRoot,
+                refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
             })
         ),
-        "数据单元拿到了 50182，第一个提交内生块拿不到：{result:?}"
+        "数据单元拿到了 50182，第一个提交内生块拿不到（每块盘上都没有：容量不够那一种）：{result:?}"
     );
     assert_eq!(
         pool.allocator.records(),
diff -ruN '--exclude=target' crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs
--- crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs	2026-09-19 05:41:07.583652778 +0000
+++ crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs	2026-09-19 11:49:11.993124872 +0000
@@ -3,17 +3,20 @@
 
 use std::io::Write as _;
 
+use singlefs_checker::image::{chosen_superblocks, valid_roots};
 use singlefs_core::address::CheckpointTxg;
 use singlefs_harness::crash::MemoryPool;
 use singlefs_harness::history::{
-    allocated_and_walked_bytes, classify_failure, execute_history, execute_history_observing,
-    generate_history, raised_floor_lands_only_on_abandoned_roots, run_history_campaign,
-    shrink_to_reproduction, AppliedEffect, ContentChoice, ContentLength, FailureObservation,
-    FailureSignature, FindingShrinking, FloorTargetChoice, GeneratedHistory, GenerationWeights,
-    HistoryEnding, HistoryOperation, HistoryOperationKind, HistorySeed, HistoryStartingPoint,
-    HistoryTally, MountAllocationComparison, NewFindingReport, RecordReuse, RollbackTargetChoice,
+    allocated_and_walked_bytes, allocation_records_on_the_image_under, classify_failure,
+    execute_history, execute_history_observing, execute_history_with, generate_history,
+    raised_floor_lands_only_on_abandoned_roots, run_history_campaign, shrink_to_reproduction,
+    AppliedEffect, ContentChoice, ContentLength, FailureObservation, FailureSignature,
+    FindingShrinking, FloorTargetChoice, GeneratedHistory, GenerationWeights, HistoryEnding,
+    HistoryOperation, HistoryOperationKind, HistorySeed, HistoryStartingPoint, HistoryTally,
+    MountAllocationComparison, NewFindingReport, PerStepChecker, RecordReuse, RollbackTargetChoice,
     StepOutcome, StepPosition, KNOWN_RED_FORMS,
 };
+use singlefs_harness::model::{ModelCheckpointTxg, ModelInstanceGeneration, ModelRootKey};
 use singlefs_harness::SharedStream;
 
 /// 快档的种子区间与每段步数：写死，门禁每次跑同一批。
@@ -31,6 +34,11 @@
 const ROLLBACK_SAMPLING_SEEDS: u64 = 48;
 const ROLLBACK_SAMPLING_OPERATIONS_PER_HISTORY: usize = 30;
 
+/// 分配记录墙那一格的取样点：种子区间与每段步数，写死（判出率见那条用例的注释）。
+const WALL_SAMPLING_FIRST_SEED: u64 = 0;
+const WALL_SAMPLING_SEEDS: u64 = 32;
+const WALL_SAMPLING_OPERATIONS_PER_HISTORY: usize = 150;
+
 /// 报告直接写进进程的标准输出，不经 libtest 的捕获：快档通过时计数照样出现在 `check.sh` 的输出里
 /// （`test-discipline.md`「阴性结果要能和「代码没跑到」分开」）。
 fn print_uncaptured(text: &str) {
@@ -62,17 +70,21 @@
     let refusals = &tally.refusals_by_member;
     for member in [
         "MountError::RollbackFloorAboveCeiling",
-        "MountError::RollbackTargetNotInRing",
-        "MountError::RollbackToVersionWithoutFileUnsupported",
+        "MountError::RollbackTargetNotACandidate(NotInRing)",
+        "MountError::RollbackTargetNotACandidate(TargetVersionWithoutFileUnsupported)",
         "PublishError::ContentExceedsDataUnit",
         "PublishError::FirstFileVersionNotRightAfterTheSecondWarmUp",
     ] {
         assert!(count_of(refusals, member) >= 1, "没见过 {member}");
     }
     assert!(
-        refusals
-            .keys()
-            .any(|member| member.starts_with("MountError::RollbackTargetNotACandidate")),
+        count_of(
+            refusals,
+            "MountError::RollbackTargetNotACandidate(BelowEffectiveFloor)"
+        ) + count_of(
+            refusals,
+            "MountError::RollbackTargetNotACandidate(OnAbandonedTimeline)"
+        ) >= 1,
         "回退的目标没落到过被抛弃的根或 F 之下的根"
     );
     assert!(tally.raises_that_reclaimed >= 1, "抬 F 一次都没回收到落点");
@@ -177,6 +189,7 @@
         FAST_TIER_SEEDS,
         FAST_TIER_OPERATIONS_PER_HISTORY,
         &GenerationWeights::BROAD,
+        PerStepChecker::Run,
         worker_threads_by_default(),
         FindingShrinking::ReportSeedsOnly,
     );
@@ -205,6 +218,7 @@
         REUSE_SAMPLING_SEEDS,
         REUSE_SAMPLING_OPERATIONS_PER_HISTORY,
         &GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
+        PerStepChecker::Run,
         worker_threads_by_default(),
         FindingShrinking::ReportSeedsOnly,
     );
@@ -241,6 +255,7 @@
         ROLLBACK_SAMPLING_SEEDS,
         ROLLBACK_SAMPLING_OPERATIONS_PER_HISTORY,
         &GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
+        PerStepChecker::Run,
         worker_threads_by_default(),
         FindingShrinking::ReportSeedsOnly,
     );
@@ -263,6 +278,224 @@
     );
 }
 
+/// 分配记录墙那一格（增补 3 第 2 件代码三方第一轮判决第三节第 1 条）的取样点：比重取 `GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL`，
+/// 不跑池级 checker（`PerStepChecker::Skipped`：跑 checker 的历史在根环转过一圈之后停在已知红第 0 条，走不到 812 条），种子与步数见常量。
+/// 墙拒时执行器按 checker 的解析从镜像上数准入基数，真条数 ≤ 812 而实现拒了，模型判对不上。先判没有新发现（模型对不上、执行器判出的、
+/// panic 都算），再核这一路真的跑到了：分配记录墙拒过、模型按真条数放行过——这个数只在放行时加，变异下被拒、判红的是分类，不是这条计数。
+#[test]
+fn allocation_record_wall_sampling_without_the_checker_refuses_only_above_one_node_by_the_true_count(
+) {
+    let started = std::time::Instant::now();
+    let report = run_history_campaign(
+        WALL_SAMPLING_FIRST_SEED,
+        WALL_SAMPLING_SEEDS,
+        WALL_SAMPLING_OPERATIONS_PER_HISTORY,
+        &GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
+        PerStepChecker::Skipped,
+        worker_threads_by_default(),
+        FindingShrinking::ReportSeedsOnly,
+    );
+    let rendered = report.render();
+    print_uncaptured(&format!(
+        "── 随机历史：逼近分配记录墙的取样点 ──\n{rendered}用时 {:.1} 秒\n",
+        started.elapsed().as_secs_f64()
+    ));
+    assert!(
+        report.new_findings.is_empty(),
+        "新发现（这一档不跑 checker，只有模型、执行器的判定与 panic）：\n{rendered}"
+    );
+    assert_eq!(report.tally.checker_runs, 0, "这一档不跑池级 checker");
+    assert!(
+        report
+            .tally
+            .model_counts
+            .allocation_record_wall_refusals_over_one_node
+            >= 1,
+        "分配记录墙一次都没按真条数放行过（墙那一格没跑到）"
+    );
+}
+
+/// 镜像上最新那条根（按 checker 的读法，(txg, 实例) 最大）的身份。
+fn newest_root_on_the_image(image: &MemoryPool) -> ModelRootKey {
+    let geometry = chosen_superblocks(image)
+        .into_iter()
+        .find_map(|(_, chosen)| chosen.map(|(_, geometry)| geometry))
+        .expect("历史里的盘上至少一块超级块自证过");
+    let (_, _, newest) = valid_roots(image, &geometry)
+        .into_iter()
+        .max_by_key(|(_, _, view)| (view.checkpoint_txg, view.instance))
+        .expect("根环里至少有 mkfs 的第 0 代根");
+    ModelRootKey {
+        checkpoint_txg: ModelCheckpointTxg(newest.checkpoint_txg),
+        instance: ModelInstanceGeneration(u64::from(newest.instance)),
+    }
+}
+
+/// 分配记录墙的基数按 checker 的解析从镜像上数（`allocation_records_on_the_image_under`，不看分配器）：一条记录记一个单元、每盘一条、
+/// 释放只改写不删（D3（空间分配） 已定项 7）。从 mkfs 起：第 0 代根与零单元写行、暖机那几版树表 0 条、没有分配记录树，数出 0 条；
+/// 第一个文件那一版 20 条（八个单元加 mkfs 的实例表与第 0 版树表，每盘各一条）。从第一个文件起连着覆盖写（回收之前不复用），
+/// 每次每盘加 8 条：36、52、68。
+#[test]
+fn allocation_records_counted_on_the_image_are_one_per_unit_per_device_and_zero_without_a_file() {
+    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
+        length: ContentLength::InsideOneDataUnit { selector: 2999 },
+        fill_seed: 5,
+    });
+    let count_after_each_step = |history: &GeneratedHistory| {
+        let mut counted: Vec<Option<u64>> = Vec::new();
+        let run = execute_history_observing(history, &SharedStream::new(), &mut |observation| {
+            counted.push(allocation_records_on_the_image_under(
+                observation.image,
+                newest_root_on_the_image(observation.image),
+            ));
+        });
+        assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
+        counted
+    };
+    let from_make_filesystem = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
+        operations: vec![
+            HistoryOperation::CloseAndMountWritable,
+            HistoryOperation::PublishFirstFile(ContentChoice {
+                length: ContentLength::InsideOneDataUnit { selector: 2999 },
+                fill_seed: 3,
+            }),
+        ],
+    };
+    assert_eq!(
+        count_after_each_step(&from_make_filesystem),
+        vec![Some(0), Some(0), Some(20)],
+        "起点（txg 0）、挂载的零单元写行与暖机（txg 1、2）、第一个文件（txg 3）"
+    );
+    let from_the_first_file = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: vec![overwrite; 3],
+    };
+    assert_eq!(
+        count_after_each_step(&from_the_first_file),
+        vec![Some(20), Some(36), Some(52), Some(68)]
+    );
+}
+
+/// 分配记录墙的边沿（增补 3 第 2 件代码三方第一轮判决第三节第 1 条，攻方变异 W1 的形态：墙的 `>` 写成 `>=`，正好 812 条也拒）：
+/// 从第一个文件（txg 3，20 条）起连着可写挂载四次——写行与暖机按根环落点每次加 18、18、26、26 条（txg 4–13），108 条——再连着覆盖写。
+/// 回收只在挂载与抬 F 时做，挂载都在根环转圈（txg 24）之前、那时环里最旧的有效根还是 txg 0，一个落点都不回收，所以每次覆盖写正好加 16 条：
+/// 第 44 次覆盖写之后正好 812 条（一个节点装满，条款说装得下），第 45 次要 828 条、被墙拒。不跑池级 checker（根环转圈之后已知红第 0 条
+/// 会先停下）。今天的代码上这段跑完：812 条那一次做成、镜像上数得 812 条；828 条那一次被拒、模型按镜像上的真条数（812 + 16）放行。
+/// W1 下 812 条那一次被拒，模型判「模型说该成、实现拒了」。
+#[test]
+fn an_overwrite_that_fills_the_allocation_node_to_exactly_812_records_succeeds_and_the_next_is_refused(
+) {
+    let overwrite = HistoryOperation::PublishOverwrite(ContentChoice {
+        length: ContentLength::InsideOneDataUnit { selector: 2999 },
+        fill_seed: 11,
+    });
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: std::iter::repeat_n(HistoryOperation::CloseAndMountWritable, 4)
+            .chain(std::iter::repeat_n(overwrite, 45))
+            .collect(),
+    };
+    let mut counted_after_each_step: Vec<Option<u64>> = Vec::new();
+    let run = execute_history_with(
+        &history,
+        PerStepChecker::Skipped,
+        &SharedStream::new(),
+        &mut |observation| {
+            counted_after_each_step.push(allocation_records_on_the_image_under(
+                observation.image,
+                newest_root_on_the_image(observation.image),
+            ));
+        },
+    );
+    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
+    assert_eq!(
+        counted_after_each_step[..6],
+        [Some(20), Some(38), Some(56), Some(82), Some(108), Some(124)],
+        "起点、四次挂载、第一次覆盖写之后镜像上数的条数"
+    );
+    assert!(
+        matches!(
+            run.outcomes[47],
+            StepOutcome::Applied(AppliedEffect::Published { .. })
+        ),
+        "第 44 次覆盖写正好装满一个节点，要做成：{:?}",
+        run.outcomes[47]
+    );
+    assert_eq!(
+        counted_after_each_step[48],
+        Some(812),
+        "第 44 次覆盖写之后镜像上正好 812 条"
+    );
+    assert_eq!(
+        run.outcomes[48],
+        StepOutcome::Refused {
+            member: "PublishError::AllocationRecordsExceedOneNode".to_string()
+        },
+        "第 45 次要 828 条"
+    );
+    assert_eq!(
+        run.tally
+            .model_counts
+            .allocation_record_wall_refusals_over_one_node,
+        1,
+        "模型按镜像上的真条数放行了那一次"
+    );
+}
+
+/// 回退到被抛弃时间线上的根（F 还是 0，不低于 F）要拒，理由是「被抛弃」；模型按判别字段比理由（增补 3 第 2 件代码三方第一轮判决
+/// 第三节第 2 条：攻方变异 R1——被抛弃的根报成「txg 低于 F」、只换理由——此前三组套件三段全绿）。第一个文件（实例 1，txg 3）之后
+/// 可写挂载（实例 2：写行 txg 4、暖机 5）、覆盖写两次（6、7）、回退到根环从新到旧第 2 条 (5, 2)（实例 3：写行 txg 8、暖机 9、10）、
+/// 再回退到从新到旧第 3 条 (7, 2)：最新根的实例表里有回退行 (2, 5)，7 > 5 ⇒ 被抛弃。今天的代码上这段跑完、最后一步报
+/// `OnAbandonedTimeline`；R1 下模型判「拒绝的理由」对不上。
+#[test]
+fn rolling_back_onto_an_abandoned_root_above_the_floor_is_refused_for_being_abandoned() {
+    let overwrite = |fill_seed: u64| {
+        HistoryOperation::PublishOverwrite(ContentChoice {
+            length: ContentLength::InsideOneDataUnit { selector: 2999 },
+            fill_seed,
+        })
+    };
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: vec![
+            HistoryOperation::CloseAndMountWritable,
+            overwrite(6),
+            overwrite(7),
+            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
+                index_from_newest: 2,
+            }),
+            HistoryOperation::CloseAndMountRollback(RollbackTargetChoice::RingRoot {
+                index_from_newest: 3,
+            }),
+        ],
+    };
+    let run = execute_history(&history);
+    assert_eq!(run.ending, HistoryEnding::Completed, "{:?}", run.ending);
+    assert!(
+        matches!(
+            run.outcomes[3],
+            StepOutcome::Applied(AppliedEffect::Mounted { .. })
+        ),
+        "回退到 (5, 2) 要做成：{:?}",
+        run.outcomes[3]
+    );
+    assert_eq!(
+        run.outcomes[4],
+        StepOutcome::Refused {
+            member: "MountError::RollbackTargetNotACandidate(OnAbandonedTimeline)".to_string()
+        }
+    );
+    assert_eq!(
+        run.tally.model_counts.required_refusals_matched, 1,
+        "模型要求拒、理由对上了"
+    );
+}
+
 /// 回退到 txg = F_生效 的根要做成（D16（发布语义） 已定项 1「回退候选集」：txg ≥ F_生效），模型按 B2 那一格判（攻方变异「回退到
 /// txg = F_生效 的根也拒」）。第一个文件（txg 3）之后可写挂载（实例 2：txg 4、5）、覆盖写四次（6–9）、抬 F（选择子 6 ⇒ F = 6：上限是
 /// 第 4 新的非空根 6 与盘 1 上最新的有效根 7 取小，推 txg 10、11 两次）、回退到候选集的下沿 (6, 2)（实例 3：txg 12、13）、冷启动读回。
@@ -634,6 +867,7 @@
         seed_count,
         operations_per_history,
         &weights,
+        PerStepChecker::Run,
         worker_threads,
         shrinking,
     );
@@ -689,7 +923,7 @@
             seed.0
         );
     };
-    let shrunk = shrink_to_reproduction(&history, &signature, worker_threads);
+    let shrunk = shrink_to_reproduction(&history, &signature, PerStepChecker::Run, worker_threads);
     print_uncaptured(
         &NewFindingReport {
             signature,
diff -ruN '--exclude=target' crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs
--- crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs	2026-09-18 14:10:30.585413788 +0000
+++ crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs	2026-09-19 11:38:34.904135809 +0000
@@ -3,7 +3,7 @@
 //! 池里没有全空的 64 槽聚簇段、开放段也用满，每块盘上却还有大片空槽：一次发布要成功，提交内生块落在回落政策函数给的槽上。
 //! 回落与 bump 游标绕开同一套位（已分配、影子账隔离、抬 F 扣住），不是绕开影子账或回收扣住的第二条路。
 //! 另钉增补 2 第 ② 行里「抬 F 的空发布分配不到固定点（开放段满、唯一全空段正是扣住的那一段）」补了回落之后的行为：
-//! 扣住的段外面还有不被挡的空槽时抬 F 成功、落点一个都不在扣住的槽上；连一个都没有时照旧报 `NoSpaceFor`、扣住位留在这个进程里。
+//! 扣住的段外面还有不被挡的空槽时抬 F 成功、落点一个都不在扣住的槽上；连一个都没有时照旧落点被拒（`PlacementRefused`，每块盘上都没有）、扣住位留在这个进程里。
 //!
 //! 「没有全空段」的形态直接改空闲图造（开放段剩下的槽占满、每个全空段占掉段首一槽），不写分配记录：
 //! 第一版分配记录树只有一个节点（812 条），用真发布占满 3312 个段装不下。所以这些池上记账与记录对不上，这里不跑池级 checker。
@@ -291,7 +291,8 @@
     }
 }
 
-/// 同一格，扣住的槽之外一个空槽都没有：抬 F 之前把每块盘上的空槽全占掉，回收出来的全是扣住的槽 ⇒ 第一次空发布照旧报 `NoSpaceFor`、
+/// 同一格，扣住的槽之外一个空槽都没有：抬 F 之前把每块盘上的空槽全占掉，回收出来的全是扣住的槽 ⇒ 第一次空发布照旧落点被拒（`PlacementRefused`、
+/// 每块盘上都没有）、
 /// 一个写都没发；扣住位留在这个进程里：记账算它们空闲，分配器却一个都发不出去。
 #[test]
 fn raising_the_floor_with_no_free_slot_outside_the_hold_still_fails_and_the_hold_stays_in_the_process(
@@ -309,11 +310,12 @@
     assert!(
         matches!(
             refused,
-            Err(MountError::Publish(PublishError::NoSpaceFor {
-                unit: TransactionUnit::AllocationTree
+            Err(MountError::Publish(PublishError::PlacementRefused {
+                unit: TransactionUnit::AllocationTree,
+                refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
             }))
         ),
-        "回收出来的全是扣住的槽，第一个固定点就拿不到：{:?}",
+        "回收出来的全是扣住的槽，第一个固定点就拿不到（每块盘上都没有：容量不够那一种）：{:?}",
         refused.as_ref().err()
     );
     assert_eq!(
diff -ruN '--exclude=target' crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs
--- crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs	2026-09-17 17:28:08.704276558 +0000
+++ crates/singlefs-harness/tests/second_transaction_supplement_two_unequal_devices.rs	2026-09-19 11:38:12.891646630 +0000
@@ -1,7 +1,8 @@
 //! 里程碑「第二个事务」增补 2 收进来的 C368（分配器落点只看盘 0，盘不等大时断言失败）：D2（RAID 条带策略） 已定项 2「各盘不必等大」，
 //! D3（空间分配） 已定项 8 第 1 条「在每一块被选中的设备上各自取该设备内」。
 //! 两块不等大的盘（盘 0 4 GiB、盘 1 3 GiB 加 33 槽）：mkfs、第一个事务、可写挂载今天都接受；填到小盘单元区末尾之后，
-//! 分配按设备取落点，小盘答不出就在动任何状态之前拒绝、发布报 `NoSpaceFor`，不 panic、一个写都不发。
+//! 分配按设备取落点，小盘答不出就在动任何状态之前拒绝、发布报 `PlacementRefused` 并带着分配器的原因（不再一律报装不下），
+//! 不 panic、一个写都不发。
 //! 另钉两个第一版不支持、没有条款的分支（拒绝成员的名字说哪条没定）：各盘给提交内生块的去处不同（一块开段一块回落，D3 已定项 8 待办 ①）；
 //! 各盘的用户数据落点不同（`Placement` 两盘同槽）。
 //!
@@ -229,7 +230,7 @@
 
 /// C368 的验收：小盘单元区末尾那一对偶数槽在两块盘上都空时，用户数据按设备取、两块盘都落在那里；之后小盘一个空槽都没有、
 /// 大盘在小盘末尾之后还有 1 万多个槽——用户数据、一槽节点、两槽容器都拒成「小盘满了、设备集合怎么选没有条款」，分配器一样没动；
-/// 走发布路径报 `NoSpaceFor`（数据单元先分配），录制流一步都不多、分配器退回。只看盘 0 的写法在这里 panic（小盘越界）。
+/// 走发布路径报 `PlacementRefused`（数据单元先分配），原因照样是小盘满了，录制流一步都不多、分配器退回。只看盘 0 的写法在这里 panic（小盘越界）。
 #[test]
 fn filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking(
 ) {
@@ -296,16 +297,19 @@
 
     let operations_before = pool.stream.operations().len();
     let refused = try_overwrite(&mut pool);
-    assert!(
-        matches!(
-            refused,
-            Err(PublishError::NoSpaceFor {
-                unit: TransactionUnit::Data
-            })
+    match &refused {
+        Err(PublishError::PlacementRefused {
+            unit: TransactionUnit::Data,
+            refusal,
+        }) => assert_eq!(
+            *refusal,
+            PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined {
+                full_devices: vec![DeviceIdentity(1)],
+            },
+            "发布层报的是分配器的原因（小盘满了），不是一律报装不下"
         ),
-        "{:?}",
-        refused.as_ref().err()
-    );
+        other => panic!("数据单元的落点该被拒：{:?}", other.as_ref().err()),
+    }
     assert_eq!(
         pool.stream.operations().len(),
         operations_before,
@@ -316,7 +320,7 @@
 
 /// 各盘给提交内生块的去处不同：小盘单元区里只剩末尾三个槽、一个全空段都没有 ⇒ 小盘回落到 196638；大盘在小盘末尾之后还有全空段 ⇒
 /// 开段 196672。各盘上的聚簇段要不要对齐没有条款（D3（空间分配） 已定项 8 待办 ①），拒成那个成员、分配器不动；发布里数据单元两块盘
-/// 同落 196638，extent 树根在这里被拒 ⇒ `NoSpaceFor`，录制流一步都不多。
+/// 同落 196638，extent 树根在这里被拒 ⇒ `PlacementRefused`、原因是各盘去处不同，录制流一步都不多。
 #[test]
 fn devices_answering_different_places_for_a_commit_generated_block_are_refused_before_anything_is_written(
 ) {
@@ -358,16 +362,19 @@
 
     let operations_before = pool.stream.operations().len();
     let refused = try_overwrite(&mut pool);
-    assert!(
-        matches!(
-            refused,
-            Err(PublishError::NoSpaceFor {
-                unit: TransactionUnit::ExtentRoot
-            })
+    match &refused {
+        Err(PublishError::PlacementRefused {
+            unit: TransactionUnit::ExtentRoot,
+            refusal,
+        }) => assert!(
+            matches!(
+                refusal,
+                PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined { .. }
+            ),
+            "发布层报的是分配器的原因（各盘去处不同），不是一律报装不下：{refusal:?}"
         ),
-        "{:?}",
-        refused.as_ref().err()
-    );
+        other => panic!("extent 树根的落点该被拒：{:?}", other.as_ref().err()),
+    }
     assert_eq!(
         pool.stream.operations().len(),
         operations_before,
@@ -377,7 +384,8 @@
 }
 
 /// 各盘的用户数据落点不同（等大的池，只在盘 0 上隔离 50182–50183：只有拼出来的、两盘分配记录不对称的镜像走得到）：
-/// 盘 0 答 50184、盘 1 答 50182，`Placement` 两盘同槽装不下，拒成那个成员、分配器不动；覆盖写报 `NoSpaceFor`，盘上逐项不变。
+/// 盘 0 答 50184、盘 1 答 50182，`Placement` 两盘同槽装不下，拒成那个成员、分配器不动；覆盖写报 `PlacementRefused`、原因是各盘落点不同，
+/// 盘上逐项不变。
 #[test]
 fn user_data_slots_that_differ_across_devices_are_refused_before_anything_is_written() {
     let mut pool = common::build_pool("supplement-two-user-data-disagree");
@@ -417,16 +425,22 @@
         },
         InstanceGeneration(1),
     );
-    assert!(
-        matches!(
-            refused,
-            Err(PublishError::NoSpaceFor {
-                unit: TransactionUnit::Data
-            })
+    match &refused {
+        Err(PublishError::PlacementRefused {
+            unit: TransactionUnit::Data,
+            refusal,
+        }) => assert_eq!(
+            *refusal,
+            PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported {
+                slot_per_device: vec![
+                    (DeviceIdentity(0), SlotNumber(50_184)),
+                    (DeviceIdentity(1), SlotNumber(50_182)),
+                ],
+            },
+            "发布层报的是分配器的原因（各盘落点不同），不是一律报装不下"
         ),
-        "{:?}",
-        refused.as_ref().err()
-    );
+        other => panic!("数据单元的落点该被拒：{:?}", other.as_ref().err()),
+    }
     assert_eq!(fingerprint(&pool.allocator), before, "失败的发布退回分配器");
     assert_eq!(
         disk_snapshot(&pool.memory_pool(), &pool.stream),
````

## 历史版本

### 2026-09-19
- 首版。
