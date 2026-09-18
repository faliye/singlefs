# 附录二：里程碑「第二个事务」步 4 / 步 5 代码三方第二轮改动（第一轮打中之后的五处改法；基准 5f9e449；2026-09-17）

## 一、diff（相对提交 5f9e449，`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-harness/src/crash.rs`、`crates/mutations.tsv`；`mount.rs` 未跟踪，此处为空，全文见下）

```diff
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index 511de3a..543aed0 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -8,17 +8,42 @@
 空闲不随分配减（I-5.2 要红）	crates/singlefs-core/src/allocator.rs	        self.free_slots -= span;	        // 变异：空闲不减	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
 释放时清位图（立即复用）	crates/singlefs-core/src/allocator.rs	        self.deferred_slots += span;\n    }	        self.deferred_slots += span;\n        for index in start..end {\n            self.allocated[index] = false;\n        }\n    }	-p singlefs-harness --test second_transaction_step_one_overwrite -- released_placements	released_placements_are_not_handed_out_again_before_reclaim_exists
 分配记录树装不下不判	crates/singlefs-core/src/transaction.rs	    if records_after_this_publish > allocation_node_capacity {	    if false && records_after_this_publish > allocation_node_capacity {	-p singlefs-harness --test second_transaction_step_one_overwrite -- repeated_overwrites	repeated_overwrites_report_a_full_allocation_node_instead_of_panicking
-释放退回按提示	crates/singlefs-core/src/transaction.rs	    let release = placements_to_release_via_mapping(previous, allocator)?;	    let release = previous.placements();	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_goes_through	release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released
+释放退回按提示	crates/singlefs-core/src/transaction.rs	            placements_to_release_via_mapping(previous_version, allocator, &rewritten)?	            previous_version.placements()	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_goes_through	release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released
 映射查到 key 就算（落点指错不核记录）	crates/singlefs-core/src/transaction.rs	        let record = allocator.record_for(locations[0].device, slot).ok_or(	        let record = allocator\n            .record_for(locations[0].device, slot)\n            .or(allocator.records().first())\n            .ok_or(	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_reports_a_mapping_entry	release_reports_a_mapping_entry_whose_slot_has_no_record_or_the_wrong_span_instead_of_panicking
 失败的发布不退回分配器	crates/singlefs-core/src/transaction.rs	    if outcome.is_err() {	    if false && outcome.is_err() {	-p singlefs-harness --test second_transaction_step_one_overwrite -- publish_running_out_of_space_midway	publish_running_out_of_space_midway_leaves_the_allocator_as_it_was
-内容超长不判	crates/singlefs-core/src/transaction.rs	    if publish.content.len() > data_unit_capacity {	    if false && publish.content.len() > data_unit_capacity {	-p singlefs-harness --test second_transaction_step_one_overwrite -- content_larger_than_a_data_unit	content_larger_than_a_data_unit_payload_is_refused_before_anything_is_touched
+内容超长不判	crates/singlefs-core/src/transaction.rs	        if file.content.len() > data_unit_capacity {	        if false && file.content.len() > data_unit_capacity {	-p singlefs-harness --test second_transaction_step_one_overwrite -- content_larger_than_a_data_unit	content_larger_than_a_data_unit_payload_is_refused_before_anything_is_touched
 oracle 只比 txg 不比实例	crates/singlefs-harness/src/crash.rs	        if (effective_txg, effective_instance) < (newest_txg, newest_instance) {	        if effective_txg < newest_txg {	-p singlefs-harness --lib -- oracle_instance_tests	landing_on_the_lower_instance_of_the_same_txg_is_a_violation
 oracle 只按 txg 找版本	crates/singlefs-harness/src/crash.rs	        version.checkpoint_txg == effective_txg && version.instance == effective_instance	        version.checkpoint_txg == effective_txg	-p singlefs-harness --lib -- oracle_instance_tests	the_same_txg_from_two_instances_are_two_versions
 oracle 放过没版本的更新根	crates/singlefs-harness/src/crash.rs	                (candidate_version.checkpoint_txg, candidate_version.instance)\n                    < (effective_txg, effective_instance)	                false	-p singlefs-harness --lib -- oracle_instance_tests	newer_root_without_any_version_reporting_no_file_is_violation
 Ignore 那一遍恢复不过 oracle	crates/singlefs-harness/src/crash.rs	        tally.ignored_violations += 1;	        tally.ignored_violations += 0;	-p singlefs-harness --test second_transaction_step_zero_layer0 -- targeted_controls	targeted_controls_on_the_second_publish_go_red_where_they_should
-步 1 变异：不释放旧落点	crates/singlefs-core/src/transaction.rs	    let release = placements_to_release_via_mapping(previous, allocator)?;	    let release: Vec<Placement> = Vec::new();	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
+步 1 变异：不释放旧落点	crates/singlefs-core/src/transaction.rs	            placements_to_release_via_mapping(previous_version, allocator, &rewritten)?	            Vec::<Placement>::new()	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
 步 1 变异：defer 行写 0	crates/singlefs-core/src/transaction.rs	            device_map.deferred_slots() * SLOT_BYTES,	            0,	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
 步 1 变异：事务号不加一	crates/singlefs-core/src/transaction.rs	            transaction: previous.record.transaction + 1,	            transaction: previous.record.transaction,	-p singlefs-harness --test second_transaction_step_one_overwrite -- overwrite_publishes	overwrite_publishes_the_second_version_through_the_same_commit_shape
 步 1 变异：改动计数留 1	crates/singlefs-core/src/transaction.rs	            change_count: txg.0,	            change_count: 1,	-p singlefs-harness --test second_transaction_step_one_overwrite -- overwrite_publishes	overwrite_publishes_the_second_version_through_the_same_commit_shape
 步 2 变异：已分配行不随分配更新（I-3.1 要红）	crates/singlefs-core/src/transaction.rs	            device_map.allocated_slots() * SLOT_BYTES,	            0,	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
 步 2 变异：忘了改写分配记录	crates/singlefs-core/src/allocator.rs	            record.is_released = true;	            record.is_released = false;	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
+步 3：不写行（实例表照抄）	crates/singlefs-core/src/mount.rs	            instance_table: InstanceTablePlan::Rewrite(table_after.to_records()),	            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
+步 3：暖机只推一次	crates/singlefs-core/src/mount.rs	        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(warm_up_publishes.len()).expect("次数") < 1	-p singlefs-harness --test second_transaction_step_three_second_instance -- damaging_every_instance_two_root	damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device
+步 3：前缀跨实例边界（把别的实例的记录也接上）	crates/singlefs-core/src/recovery.rs	            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water	            (record.instance, record.checkpoint_txg) > water	-p singlefs-harness --test first_transaction_step_seven_layer0 -- layer0_partial_enumeration	layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape
+步 3：链首锚点错一位（接在所选根自己那条记录之后第二条）	crates/singlefs-core/src/recovery.rs	        root_own_record_counter.map(|counter| (root.instance, counter + 1));	        root_own_record_counter.map(|counter| (root.instance, counter + 2));	-p singlefs-harness --test second_transaction_step_three_second_instance -- stray_record_of_the_previous_instance	stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row
+步 3：I-3.8 判定恒真	crates/singlefs-checker/src/walk.rs	            unique && below_mount_root && chain_record_last,	            unique || below_mount_root || chain_record_last || true,	-p singlefs-harness --test second_transaction_step_three_second_instance -- checker_rejects_an_instance_table_row	checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root
+步 3：重建分配器时不把已释放的放进 defer 队列	crates/singlefs-core/src/allocator.rs	            if record.is_released {\n                device_map.mark_released(record.slot, span);\n            }	            if record.is_released && false {\n                device_map.mark_released(record.slot, span);\n            }	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
+步 4：回退行不带回退位	crates/singlefs-core/src/mount.rs	        applied_transaction_high_water: 0,\n        is_rollback: true,	        applied_transaction_high_water: 0,\n        is_rollback: false,	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
+步 4：只写被退回的实例那一行、不写中间实例行	crates/singlefs-core/src/mount.rs	    for row_instance in first_row_instance..instance.0 {	    for row_instance in first_row_instance..first_row_instance + 1 {	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
+步 4：回退之后 jsn 接 R_old 那条之后（C340 的 P1，盖掉被抛弃发布的记录槽）	crates/singlefs-core/src/mount.rs	    let next_counter = highest_counter + 1;	    let next_counter = own_record.counter + 1;	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_keeps_the_abandoned_records	rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation
+步 4：影子账一个槽都不隔离	crates/singlefs-core/src/mount.rs	    if shadow_ledger == ShadowLedger::On {	    if shadow_ledger == ShadowLedger::Off {	-p singlefs-harness --test second_transaction_step_four_rollback -- without_the_shadow_ledger	without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit
+步 4：回退候选集不按实例表判（被抛弃时间线的根也能退到）	crates/singlefs-core/src/mount.rs	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg)	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg && false)	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_onto_an_abandoned	rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused
+步 4：前缀第五条不判（回退行的 W 不封顶）	crates/singlefs-core/src/recovery.rs	            if high_water == 0 || record.transaction > high_water {	            if false && (high_water == 0 || record.transaction > high_water) {	-p singlefs-harness --test second_transaction_step_four_rollback -- the_rollback_row_caps	the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water
+步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根	crates/singlefs-checker/src/walk.rs	        if index != newest_index && !abandoned && !below_floor {	        if index != newest_index && !below_floor {	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
+步 5：回收不看释放代（复用窗口置 0）	crates/singlefs-core/src/allocator.rs	            .filter(|record| record.is_released && record.generation <= floor)	            .filter(|record| record.is_released)	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
+步 5：抬 F 的上限不看第 4 新的非空根	crates/singlefs-core/src/mount.rs	    Some(newest_on_every_device.min(fourth_newest))	    Some(newest_on_every_device.max(fourth_newest))	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_above	raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused
+步 5：抬 F 的空发布只推一次（F 只落在一块盘上）	crates/singlefs-core/src/mount.rs	        && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(publishes.len()).expect("次数") < 1	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
+步 5：复用时追加记录而不改写（同盘同槽两条）	crates/singlefs-core/src/allocator.rs	            if self.reclaimed.remove(&key) {	            if self.reclaimed.remove(&key) && false {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
+步 5：checker 的候选集不按 F 收（F 之下的根照走）	crates/singlefs-checker/src/walk.rs	        if index != newest_index && !abandoned && !below_floor {	        if index != newest_index && !abandoned {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
+步 5：F_生效 取各盘 F 最大值的最大值而不是最小值	crates/singlefs-core/src/recovery.rs	    highest_per_device\n        .values()\n        .copied()\n        .min()\n        .unwrap_or(CheckpointTxg(0))	    highest_per_device\n        .values()\n        .copied()\n        .max()\n        .unwrap_or(CheckpointTxg(0))	-p singlefs-harness --test second_transaction_step_five_reuse -- one_device_carrying_the_floor_alone	one_device_carrying_the_floor_alone_does_not_take_effect_on_remount
+步 3 三方第一轮：链首没锚点时不看 txg、无条件接上水位之上最小的一条	crates/singlefs-core/src/recovery.rs	        } else if record.checkpoint_txg != chain_start_txg_without_anchor {	        } else if false && record.checkpoint_txg != chain_start_txg_without_anchor {	-p singlefs-harness --test second_transaction_step_three_second_instance -- torn_anchor_record	torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg
+步 4：普通重开不隔离被抛弃根引用的槽（影子账只在回退那一次算）	crates/singlefs-core/src/mount.rs	        &|_| false,\n        ShadowLedger::On,\n    )?;	        &|_| false,\n        ShadowLedger::Off,\n    )?;	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_keeps_the_abandoned_records	rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation
+步 4：回退候选集的 F 用最新根自己带的 F 而不是 F_生效	crates/singlefs-core/src/mount.rs	    if target.checkpoint_txg < effective_floor {	    if target.checkpoint_txg < newest_root.rollback_floor {	-p singlefs-harness --test second_transaction_step_five_reuse -- roots_below_a_floor_carried	roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates
+步 5：回收门槛不看环里最旧有效根	crates/singlefs-core/src/mount.rs	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))	    oldest_valid_root.map_or(effective_floor, |_oldest| effective_floor)	-p singlefs-core reclaim_floor_takes	reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root
+步 4：影子账把被抛弃根账里已释放的落点也隔离	crates/singlefs-core/src/mount.rs	                if record.is_released || !isolated.insert((record.device.0, record.slot.0)) {	                if !isolated.insert((record.device.0, record.slot.0)) {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
+步 5：重建分配器时不认第 0 版树表单元（另一个进程里的第一个文件版本漏释放它）	crates/singlefs-core/src/allocator.rs	                record.generation == CheckpointTxg(0)\n                    && record.span_slots == 1\n                    && !record.is_released	                record.generation == CheckpointTxg(0)\n                    && record.span_slots == 1\n                    && !record.is_released\n                    && false	-p singlefs-core rebuild_from_records_remembers	rebuild_from_records_remembers_the_genesis_tree_table_until_it_is_released
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 01e35e5..6bd429e 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -9,6 +9,8 @@
 
 use singlefs_format::{CLUSTER_SEGMENT_SLOTS, SLOT_BYTES, UNIT_AREA_START_SLOT};
 
+use std::collections::BTreeSet;
+
 use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
 use crate::bytes::ByteWriter;
 
@@ -17,7 +19,7 @@ pub const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
 
 /// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：key (设备 4, 槽号 6) + value (跨度 2 含已释放标志, 分配代或释放代 8)。
 /// 释放时条目不删、不点删：改写成已释放 + 释放代，留到该落点被重新分配时覆盖（覆盖是步 5 回收接上时的事：
-/// 可再分配谓词今天没实现、同一个落点不会再分配到，所以 `record` 今天只追加）。
+/// 回收过的落点被再分配时那条记录改写、不追加，槽号在每盘仍唯一；回收要 F_生效 抬到释放代之上，见 `reclaim_released_up_to`）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
 pub struct AllocationRecord {
     pub device: DeviceIdentity,
@@ -110,6 +112,11 @@ pub struct DeviceFreeMap {
     /// 空闲槽数，独立维护：分配时减、回收放回时加，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️：
     /// 那样 I-5.2（空闲统计对得上） 是恒真式；三方代码第一轮攻方腿打中）。已释放而还在 defer 窗口里的不算空闲。
     free_slots: u64,
+    /// 影子账（D23（journal 的角色与格式） 已定项 14 回退段；D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」）：
+    /// 只被被抛弃根引用的槽，当前账里既不是已分配也不是 defer，分配器却不许发出去，直到被抛弃的根离开根环。只住内存、不进记账行。
+    isolated: Vec<bool>,
+    isolated_per_segment: Vec<u64>,
+    isolated_slots: u64,
     /// 其中已释放、还在 defer 窗口里的槽数（D5（快照 / 空间记账机制） 已定项 4 第 5 项）。
     deferred_slots: u64,
 }
@@ -129,6 +136,9 @@ impl DeviceFreeMap {
             allocated_slots: 0,
             free_slots: unit_area_slots,
             deferred_slots: 0,
+            isolated: vec![false; usize::try_from(unit_area_slots).expect("单元区槽数")],
+            isolated_per_segment: vec![0; usize::try_from(segments).expect("段数")],
+            isolated_slots: 0,
         }
     }
 
@@ -141,6 +151,7 @@ impl DeviceFreeMap {
         slot.0 >= UNIT_AREA_START_SLOT
             && slot.0 < UNIT_AREA_START_SLOT + self.unit_area_slots
             && !self.allocated[Self::index(slot)]
+            && !self.isolated[Self::index(slot)]
     }
 
     #[must_use]
@@ -171,6 +182,54 @@ impl DeviceFreeMap {
         );
         self.deferred_slots += span;
     }
+    /// 隔离一个只被被抛弃根引用的落点：不进已分配、不进 defer、不动空闲计数，只让分配器绕开它（用户数据落点与开放段都不落在它上面）。
+    /// 当前账里已分配的槽也可以隔离（被抛弃根与当前账都引用的那些）：隔离位独立于分配位，等它被释放、回收之后照样发不出去
+    /// （D23（journal 的角色与格式） 已定项 14 主句「被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配」的保守读法，
+    /// 与用户 2026-09-16 定的窄读法措辞「仍被有效根引用的槽不在其内」字面不一致——预想、偏离用户定案的措辞，交 alloc-basis 那一轮定）；
+    /// 同一个槽隔离两次不重复计数。
+    pub fn isolate(&mut self, slot: SlotNumber, span: u64) {
+        let start = Self::index(slot);
+        let end = start + usize::try_from(span).expect("跨度");
+        assert!(end <= self.allocated.len(), "跨度越过单元区末尾");
+        for index in start..end {
+            if !self.isolated[index] {
+                self.isolated[index] = true;
+                let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
+                self.isolated_per_segment[segment] += 1;
+                self.isolated_slots += 1;
+            }
+        }
+    }
+
+    /// 影子账隔离的槽数（D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」的这块盘那一项）。
+    #[must_use]
+    pub fn isolated_slots(&self) -> u64 {
+        self.isolated_slots
+    }
+
+    /// 回收：一个已释放、释放代 ≤ max(F_生效, 环里最旧有效根) 的落点回到空闲（D16（发布语义） 已定项 1 的可再分配谓词）：
+    /// 位图清掉、占着的槽数与 defer 队列各减、空闲加——记账的空闲字节到这一刻才动（里程碑「第二个事务」步 5）。
+    pub fn mark_reclaimed(&mut self, slot: SlotNumber, span: u64) {
+        let start = Self::index(slot);
+        let end = start + usize::try_from(span).expect("跨度");
+        assert!(
+            self.allocated[start..end].iter().all(|taken| *taken),
+            "回收的跨度里有没分配的槽"
+        );
+        let left_free = start > 0 && !self.allocated[start - 1];
+        let right_free = end < self.allocated.len() && !self.allocated[end];
+        // runs 的增量：两边都空是把两段并成一段（−1），两边都占是新开一段（+1），一边空是接上去（不变）。
+        self.free_runs = self.free_runs + 1 - u64::from(left_free) - u64::from(right_free);
+        for index in start..end {
+            self.allocated[index] = false;
+            let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
+            self.used_per_segment[segment] -= 1;
+        }
+        self.allocated_slots -= span;
+        self.deferred_slots -= span;
+        self.free_slots += span;
+    }
+
     #[must_use]
     pub fn free_runs(&self) -> u64 {
         self.free_runs
@@ -241,7 +300,9 @@ impl DeviceFreeMap {
         let per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
         let full_segments = self.allocated.len() / per_segment;
         (0..full_segments)
-            .find(|segment| self.used_per_segment[*segment] == 0)
+            .find(|segment| {
+                self.used_per_segment[*segment] == 0 && self.isolated_per_segment[*segment] == 0
+            })
             .map(|segment| {
                 SlotNumber(
                     UNIT_AREA_START_SLOT
@@ -268,6 +329,11 @@ pub struct PoolAllocator {
     /// C146（无空段时的回落政策全仓无定义） ② 的运行时计数：实际落点与政策函数不一致的次数，第一个事务恒 0。
     pub policy_mismatches: u64,
     records: Vec<AllocationRecord>,
+    /// 已回收、还没被复用的落点（盘上那条记录仍写着已释放；复用时那条记录被改写）。只住内存。
+    reclaimed: BTreeSet<(DeviceIdentity, SlotNumber)>,
+    /// mkfs 写出的第 0 版树表单元的落点：第一个文件版本重写树表时把它释放（COW 换下的单元进 defer 队列，D3（空间分配） 已定项 7）；
+    /// 重开之后上一版从盘上重建、释放经映射与根记录走，这里留空。
+    format_time_tree_table: Option<Placement>,
 }
 
 impl PoolAllocator {
@@ -279,15 +345,63 @@ impl PoolAllocator {
             bump_cursor: 0,
             policy_mismatches: 0,
             records: Vec::new(),
+            reclaimed: BTreeSet::new(),
+            format_time_tree_table: None,
+        }
+    }
+
+    /// 重开时从盘上那棵分配记录树重建分配器（D23（journal 的角色与格式） 已定项 14：defer 队列、分配器游标、记账的现行值从所选根那棵账重新载入）：
+    /// 每条记录把自己那块盘上的槽标成占着，已释放的再进 defer 队列；空闲计数随 `mark_allocated` 减、与写路径同一条事件路径。
+    /// 开放段与 bump 游标只在内存、盘上没有（分配记录树不记它们），重开后按「没有开放段」起步：下一个提交内生块从最低的全空段开——
+    /// 上一段里没用完的槽先留着（C146（无空段时的回落政策全仓无定义） 之外的另一格，里程碑步 3 的决策点）。
+    #[must_use]
+    pub fn rebuild_from_records(
+        devices: Vec<DeviceFreeMap>,
+        records: Vec<AllocationRecord>,
+    ) -> Self {
+        let mut allocator = Self::new(devices);
+        for record in &records {
+            let device_map = allocator
+                .devices
+                .iter_mut()
+                .find(|device_map| device_map.device == record.device)
+                .expect("分配记录的盘在池里：走读逐盘核过");
+            let span = u64::from(record.span_slots);
+            device_map.mark_allocated(record.slot, span);
+            if record.is_released {
+                device_map.mark_released(record.slot, span);
+            }
         }
+        allocator.records = records;
+        // mkfs 那片第 0 版树表单元还没被换下（分配代 0、跨度 1、未释放；m1 实例表跨度 2，字节表一）时，重开之后第一个文件版本
+        // 照样要把它释放——不然 mkfs 与第一个文件版本不在同一个进程里那一格永远漏一槽（步 4 / 步 5 代码三方第一轮本地辩方腿两份样本都指出）。
+        allocator.format_time_tree_table = allocator
+            .records
+            .iter()
+            .find(|record| {
+                record.generation == CheckpointTxg(0)
+                    && record.span_slots == 1
+                    && !record.is_released
+            })
+            .map(|record| Placement {
+                slot: record.slot,
+                span: u64::from(record.span_slots),
+            });
+        allocator
     }
 
     /// mkfs 写在单元区里的两个单元：每盘一条分配代 0 的分配记录（字节表五：20 条里 m1 / m2 各两条，分配代 0）。
     /// 单元区之外的固定结构（超级块、根环、journal 环）不写分配记录、分配器不下探（D3（空间分配） 已定项 10 ④）。
-    pub fn mark_format_time_units(&mut self, placements: &[Placement]) {
-        for placement in placements {
-            self.record(*placement, CheckpointTxg(0));
-        }
+    pub fn mark_format_time_units(&mut self, instance_table: Placement, tree_table: Placement) {
+        self.record(instance_table, CheckpointTxg(0));
+        self.record(tree_table, CheckpointTxg(0));
+        self.format_time_tree_table = Some(tree_table);
+    }
+
+    /// mkfs 写出的第 0 版树表单元（还没被第一个文件版本换下时 `Some`）。
+    #[must_use]
+    pub fn format_time_tree_table(&self) -> Option<Placement> {
+        self.format_time_tree_table
     }
 
     #[must_use]
@@ -309,6 +423,20 @@ impl PoolAllocator {
 
     fn record(&mut self, placement: Placement, generation: CheckpointTxg) {
         for device in &self.devices {
+            let key = (device.device, placement.slot);
+            if self.reclaimed.remove(&key) {
+                // 复用：那条已释放的记录改写成这次的分配（字节表五「已释放记录被重用之后再改写一次」），槽号在每盘仍唯一。
+                let existing = self
+                    .records
+                    .iter_mut()
+                    .find(|record| record.device == device.device && record.slot == placement.slot)
+                    .expect("回收过的落点有它的已释放记录");
+                assert!(existing.is_released, "回收过的落点的记录该是已释放");
+                existing.span_slots = u16::try_from(placement.span).expect("跨度 2 字节");
+                existing.generation = generation;
+                existing.is_released = false;
+                continue;
+            }
             self.records.push(AllocationRecord {
                 device: device.device,
                 slot: placement.slot,
@@ -393,6 +521,47 @@ impl PoolAllocator {
         Some(placement)
     }
 
+    /// 回收：释放代 ≤ `floor` 的已释放落点回到空闲，等着被再分配（D16（发布语义） 已定项 1：可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)；
+    /// 第一版根环 24 槽、种子 txg 0，环里最旧有效根恒 0，`floor` 就是 F_生效）。回收过的不重复回收；返回这次回收的落点（两盘同槽，按盘 0 报）。
+    pub fn reclaim_released_up_to(&mut self, floor: CheckpointTxg) -> Vec<Placement> {
+        let mut reclaimed_now = Vec::new();
+        let candidates: Vec<AllocationRecord> = self
+            .records
+            .iter()
+            .filter(|record| record.is_released && record.generation <= floor)
+            .copied()
+            .collect();
+        for record in candidates {
+            let key = (record.device, record.slot);
+            if !self.reclaimed.insert(key) {
+                continue;
+            }
+            let device_map = self
+                .devices
+                .iter_mut()
+                .find(|device_map| device_map.device == record.device)
+                .expect("分配记录的盘在池里");
+            device_map.mark_reclaimed(record.slot, u64::from(record.span_slots));
+            if record.device == self.devices[0].device {
+                reclaimed_now.push(Placement {
+                    slot: record.slot,
+                    span: u64::from(record.span_slots),
+                });
+            }
+        }
+        reclaimed_now
+    }
+
+    /// 回退的影子账：把一个只被被抛弃根引用的落点在它那块盘上隔离。
+    pub fn isolate_abandoned(&mut self, device: DeviceIdentity, slot: SlotNumber, span: u64) {
+        let device_map = self
+            .devices
+            .iter_mut()
+            .find(|device_map| device_map.device == device)
+            .expect("被抛弃根的分配记录的盘在池里：走读逐盘核过");
+        device_map.isolate(slot, span);
+    }
+
     #[must_use]
     pub fn open_segment(&self) -> Option<SlotNumber> {
         self.open_segment
@@ -409,7 +578,7 @@ mod tests {
             DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
             DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
         ]);
-        pool.mark_format_time_units(&[
+        pool.mark_format_time_units(
             Placement {
                 slot: SlotNumber(50176),
                 span: 2,
@@ -418,7 +587,7 @@ mod tests {
                 slot: SlotNumber(50178),
                 span: 1,
             },
-        ]);
+        );
         pool
     }
 
@@ -550,4 +719,48 @@ mod tests {
             "已释放的 50180 不许再发，下一个偶数空槽对是 50182"
         );
     }
+
+    /// 重建时认出还没被换下的第 0 版树表单元（分配代 0、跨度 1、未释放）；换下之后（已释放）就不认。
+    #[test]
+    fn rebuild_from_records_remembers_the_genesis_tree_table_until_it_is_released() {
+        let device_bytes = 4_u64 * 1024 * 1024 * 1024;
+        let records = |released: bool| {
+            vec![
+                AllocationRecord {
+                    device: DeviceIdentity(0),
+                    slot: SlotNumber(50176),
+                    span_slots: 2,
+                    generation: CheckpointTxg(0),
+                    is_released: false,
+                },
+                AllocationRecord {
+                    device: DeviceIdentity(0),
+                    slot: SlotNumber(50178),
+                    span_slots: 1,
+                    generation: if released {
+                        CheckpointTxg(3)
+                    } else {
+                        CheckpointTxg(0)
+                    },
+                    is_released: released,
+                },
+            ]
+        };
+        let fresh = PoolAllocator::rebuild_from_records(
+            vec![DeviceFreeMap::new(DeviceIdentity(0), device_bytes)],
+            records(false),
+        );
+        assert_eq!(
+            fresh.format_time_tree_table(),
+            Some(Placement {
+                slot: SlotNumber(50178),
+                span: 1
+            })
+        );
+        let replaced = PoolAllocator::rebuild_from_records(
+            vec![DeviceFreeMap::new(DeviceIdentity(0), device_bytes)],
+            records(true),
+        );
+        assert_eq!(replaced.format_time_tree_table(), None);
+    }
 }
```

## 二、新文件 `crates/singlefs-core/src/mount.rs` 全文（未跟踪）

```rust
//! 可写挂载（里程碑「第二个事务」步 3）：进程重开镜像之后先走一次恢复，从盘上重建可写态——所选根、施加前缀之后的根、
//! 上一版全部角色的单元与指针、分配器、下一条 jsn——再取实例代号、给上一个实例写行（D18（块里携带什么信息） 已定项 11）、
//! 推空发布直到本实例的根覆盖每块盘（D16（发布语义） 已定项 8 甲′），之后本实例的发布才接在后面。
//! 第一版没有干净关闭标记，重开一律走恢复。

use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use crate::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use crate::block_device::BlockDevice;
use crate::journal::back_chain_of;
use crate::make_filesystem::MakeFilesystemParameters;
use crate::recovery::{
    allocation_records_under_root, choose_root, choose_superblock, effective_rollback_floor,
    highest_root_txg, instance_table_of_root, readable_roots, rebuild_version, replay_journal,
    rollback_high_water_of_root, scan_journal, JournalScanReport, RecoveryFailure,
};
use crate::root_record::RootRecord;
use crate::root_ring::target_for_publish;
use crate::transaction::{
    acquire_instance, publish_version, AcquisitionFailed, InstanceTablePlan, PoolWriter,
    PublishError, PublishPlan, TransactionOutput, TransactionUnit,
};
use singlefs_format::ROOT_RING_REGIONS;
use std::collections::BTreeSet;

pub use crate::instance_table::{InstanceRow, InstanceTableRecords};

/// 可写挂载没做成。
#[derive(Debug)]
pub enum MountError {
    Recovery(RecoveryFailure),
    /// 所选根下面还没有发布过文件版本（树表为空）：第一版的可写挂载只接在有文件的池后面，刚 mkfs 的池走第一次可写挂载那条路。
    NoPublishedVersion,
    /// 所选根指着的实例表单元解不出行与链指针。
    InstanceTableMalformed,
    Acquisition(AcquisitionFailed),
    Publish(PublishError),
    /// 回退的目标根不在根环里（没有那个 (实例, txg) 的可读根槽）。
    RollbackTargetNotInRing(RollbackTarget),
    /// 回退的目标根在根环里，却不在回退候选集里：被抛弃时间线的根，或 txg 低于回退下界 F。
    RollbackTargetNotACandidate {
        target: RollbackTarget,
        reason: &'static str,
    },
    /// 要抬的 F 超过上限 min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)（D16（发布语义） 已定项 1）。
    RollbackFloorAboveCeiling {
        requested: CheckpointTxg,
        ceiling: CheckpointTxg,
    },
}

/// 回退的目标：管理员带外从回退候选集里选的那条根（D23（journal 的角色与格式） 已定项 14 的显式例外）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RollbackTarget {
    pub instance: InstanceGeneration,
    pub checkpoint_txg: CheckpointTxg,
}

/// 只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：关掉影子账，回退之后只被被抛弃根引用的槽照常可分配——
/// C314（回退可以复用被抛弃的根引用的单元） 那两格必红靠它强制进入；产品路径恒 `On`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowLedger {
    On,
    Off,
}

impl From<RecoveryFailure> for MountError {
    fn from(failure: RecoveryFailure) -> Self {
        MountError::Recovery(failure)
    }
}

impl From<PublishError> for MountError {
    fn from(error: PublishError) -> Self {
        MountError::Publish(error)
    }
}

/// 一次可写挂载写出的东西。
#[derive(Debug)]
pub struct MountOutput {
    pub instance: InstanceGeneration,
    /// 恢复择到的根（施加前缀之前）。
    pub chosen_root: RootRecord,
    /// 施加前缀之后的根：写行与照抄都以它为准。
    pub effective_root: RootRecord,
    pub journal: JournalScanReport,
    /// 这次挂载写进实例表的行（上一个实例那一行，中间实例各一行）。
    pub rows_written: Vec<InstanceRow>,
    /// 写行那次发布（本实例的第一次发布）。
    pub row_publish: TransactionOutput,
    /// 之后的暖机空发布，直到本实例的根覆盖每块盘。
    pub warm_up_publishes: Vec<TransactionOutput>,
    /// 影子账隔离的槽数，逐盘（D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」，只住内存）；可写挂载恒 0。
    pub isolated_slots_per_device: Vec<(DeviceIdentity, u64)>,
}

/// 可写挂载的结果：写出的东西、重建并推进过的分配器、接下来的发布要接在后面的那一版。
#[derive(Debug)]
pub struct Mounted {
    pub output: MountOutput,
    pub allocator: PoolAllocator,
    pub current: TransactionOutput,
}

fn map_rebuild_failure(failure: RecoveryFailure) -> MountError {
    match failure {
        RecoveryFailure::InvariantViolated {
            invariant: "挂载",
        ..
        } => MountError::NoPublishedVersion,
        other @ (RecoveryFailure::NoValidSuperblock { .. }
        | RecoveryFailure::SuperblocksDisagree
        | RecoveryFailure::NoValidRoot
        | RecoveryFailure::UnitUnreadable { .. }
        | RecoveryFailure::UnitMalformed { .. }
        | RecoveryFailure::InvariantViolated { .. }
        | RecoveryFailure::MappingMiss { .. }
        | RecoveryFailure::MappingStillUnreadable { .. }) => MountError::Recovery(other),
    }
}

/// 新实例写行那一行的来历：上一个实例（普通挂载）或被退回的实例（回退）那一行；中间实例的行由 `establish_instance` 补 (i, 0, 0)。
struct PreviousInstanceRow {
    instance: InstanceGeneration,
    selected_root_txg: CheckpointTxg,
    applied_transaction_high_water: u64,
    is_rollback: bool,
}

/// 恢复（或回退）之后建立新实例要带的东西。
struct InstanceStart {
    chosen_root: RootRecord,
    effective_root: RootRecord,
    journal: JournalScanReport,
    previous: TransactionOutput,
    table: InstanceTableRecords,
    previous_row: PreviousInstanceRow,
    first_txg: CheckpointTxg,
    next_counter: u64,
    /// 新实例的根带的回退下界 = 恢复后生效的 F（各幸存盘所带 F 最大值的最小值）：与重建分配器时回收用的同一个值，
    /// 一条根带的 F 与它的记账行才说同一件事。
    effective_floor: CheckpointTxg,
}

/// 新实例的第一次发布的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1
/// （D23（journal 的角色与格式） 已定项 14 第 3 条）。
fn first_txg_of_new_instance<Device: BlockDevice>(
    devices: &[(DeviceIdentity, Device)],
    superblock: &crate::superblock::Superblock,
    records: &std::collections::BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
) -> CheckpointTxg {
    let highest_record_txg = records
        .values()
        .map(|record| record.checkpoint_txg)
        .max()
        .unwrap_or(CheckpointTxg(0));
    let highest_ring_txg = highest_root_txg(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
    .unwrap_or(CheckpointTxg(0));
    CheckpointTxg(highest_ring_txg.max(highest_record_txg).0 + 1)
}

/// 可再分配谓词的门槛（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）：根环 24 槽，第 0 代根被盖之前
/// 环里最旧有效根恒 0、门槛就是 F_生效；盖掉之后由环里最旧的有效根接管。
#[must_use]
pub fn reclaim_floor(
    effective_floor: CheckpointTxg,
    oldest_valid_root: Option<CheckpointTxg>,
) -> CheckpointTxg {
    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))
}

/// 一条根按最新根指着的实例表判是不是被抛弃的：有它那个实例的行 (i, Ti, Wi) 且 txg > Ti（D23（journal 的角色与格式） 已定项 14
/// 回退段的候选集规则反过来）。
fn abandoned_by_table(root: &RootRecord, table: &InstanceTableRecords) -> bool {
    table
        .rows
        .iter()
        .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
}

/// 重建分配器：从上一版的分配记录重建，按可再分配谓词的门槛回收，再把被抛弃根引用的每一个槽隔离（影子账；
/// `extra_abandoned` 是这次挂载新抛弃的根——回退时是 (txg, 实例) 大于 R_old 的那些，普通挂载没有）。
/// 影子账只住内存，所以每次挂载都要重算，不只回退那一次（步 4 / 步 5 代码三方第一轮云端攻方腿打中：回退之后普通重开一次隔离就归零）。
///
/// # Errors
/// 被抛弃根的树表或分配记录树读不到、解不开。
#[allow(
    clippy::ptr_arg,
    reason = "choose_root 等走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn rebuilt_allocator<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    superblock: &crate::superblock::Superblock,
    previous: &TransactionOutput,
    extra_abandoned: &dyn Fn(&RootRecord) -> bool,
    shadow_ledger: ShadowLedger,
) -> Result<(PoolAllocator, CheckpointTxg), MountError> {
    let device_maps: Vec<DeviceFreeMap> = devices
        .iter()
        .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
        .collect();
    let mut allocator =
        PoolAllocator::rebuild_from_records(device_maps, previous.allocation_records.clone());
    let effective_floor = effective_rollback_floor(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let roots = readable_roots(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let newest_table = choose_root(devices, superblock)
        .and_then(|newest| instance_table_of_root(devices, &newest));
    let is_abandoned = |root: &RootRecord| {
        extra_abandoned(root)
            || newest_table
                .as_ref()
                .is_some_and(|table| abandoned_by_table(root, table))
    };
    let oldest_valid_root = roots
        .iter()
        .filter(|root| !is_abandoned(root))
        .map(|root| root.checkpoint_txg)
        .min();
    allocator.reclaim_released_up_to(reclaim_floor(effective_floor, oldest_valid_root));
    if shadow_ledger == ShadowLedger::On {
        let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();
        for root in roots.iter().filter(|root| is_abandoned(root)) {
            // 那条根引用的 = 它那一版账里还分配着的落点；账里已释放的是它换下的上一版的，不算它引用。
            for record in allocation_records_under_root(devices, root)? {
                if record.is_released || !isolated.insert((record.device.0, record.slot.0)) {
                    continue;
                }
                allocator.isolate_abandoned(
                    record.device,
                    record.slot,
                    u64::from(record.span_slots),
                );
            }
        }
    }
    Ok((allocator, effective_floor))
}

/// 抬 F 的上限（D16（发布语义） 已定项 1）：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。
/// 有效 = 自证合法 ∧ 按最新根指着的实例表仍然有效 ∧ txg ≥ 今天的 F；非空 = 环里有它自己那条记录且事务号非 0
/// （预想：checker 怎么从盘上认「非空」没有条款，里程碑「第二个事务」步 5 的决策点）。
#[must_use]
pub fn rollback_floor_ceiling<Device: BlockDevice>(
    devices: &[(DeviceIdentity, Device)],
    superblock: &crate::superblock::Superblock,
    records: &std::collections::BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    current_floor: CheckpointTxg,
    table: &InstanceTableRecords,
) -> Option<CheckpointTxg> {
    let valid: Vec<RootRecord> = readable_roots(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| root.checkpoint_txg >= current_floor)
    .filter(|root| {
        !table
            .rows
            .iter()
            .any(|row| row.instance == root.instance && root.checkpoint_txg > row.selected_root_txg)
    })
    .collect();
    let mut newest_per_device: std::collections::BTreeMap<DeviceIdentity, CheckpointTxg> =
        std::collections::BTreeMap::new();
    for root in &valid {
        let device = superblock.region_devices
            [usize::try_from(target_for_publish(root.checkpoint_txg).region).expect("区域号")];
        let newest = newest_per_device
            .entry(device)
            .or_insert(root.checkpoint_txg);
        *newest = (*newest).max(root.checkpoint_txg);
    }
    let newest_on_every_device = newest_per_device.values().copied().min()?;
    let mut non_empty: Vec<CheckpointTxg> = valid
        .iter()
        .filter(|root| {
            records.values().any(|record| {
                record.instance == root.instance
                    && record.checkpoint_txg == root.checkpoint_txg
                    && record.transaction != 0
            })
        })
        .map(|root| root.checkpoint_txg)
        .collect();
    non_empty.sort_unstable_by(|left, right| right.cmp(left));
    let fourth_newest = non_empty
        .get(3)
        .copied()
        .or_else(|| valid.iter().map(|root| root.checkpoint_txg).min())?;
    Some(newest_on_every_device.min(fourth_newest))
}

/// 抬 F 的空发布做完之后的东西。
pub struct RaisedFloor {
    pub ceiling: CheckpointTxg,
    /// 带新 F 的空发布，直到每块盘上都有一条带新 F 的持久根（生效条件）。
    pub publishes: Vec<TransactionOutput>,
    /// 生效之后回收的落点（两盘同槽，按盘 0 报）。
    pub reclaimed: Vec<Placement>,
}

/// 抬回退下界 F 到 `new_floor`（D16（发布语义） 已定项 1）：正常的触发是准入不够，这里是只供测试的强制入口（`.claude/rules/fs-design.md` 五条硬要求第 2 条）。
/// 推空发布直到每块盘上都有带新 F 的根（生效），然后把释放代 ≤ 新 F 的已释放落点回收。
///
/// # Errors
/// `new_floor` 超过上限；发布失败。
pub fn raise_rollback_floor<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    new_floor: CheckpointTxg,
) -> Result<RaisedFloor, MountError> {
    let superblock = choose_superblock(&*devices)?;
    let records = scan_journal(&*devices, &superblock);
    let table = InstanceTableRecords::parse(&current.unit(TransactionUnit::InstanceTable).bytes)
        .ok_or(MountError::InstanceTableMalformed)?;
    let ceiling = rollback_floor_ceiling(
        devices,
        &superblock,
        &records,
        current.root.rollback_floor,
        &table,
    )
    .ok_or(RecoveryFailure::NoValidRoot)?;
    if new_floor > ceiling {
        return Err(MountError::RollbackFloorAboveCeiling {
            requested: new_floor,
            ceiling,
        });
    }
    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    let device_of_txg = |txg: CheckpointTxg| {
        let target = target_for_publish(txg);
        parameters.region_devices[usize::try_from(target.region).expect("区域号")]
    };
    // 回收在写第一条带新 F 的根之前：一条根带的 F 与它的记账行要说同一件事——checker 的 I-3.1（已分配统计对得上） 按那条根自己的 F
    // 取候选集的并集，释放代 ≤ F 的落点不在并集里，记账的「已分配」也不能再算它们。生效（两块盘都有带新 F 的根）之前这个进程不会再发布，
    // 回收的槽在生效之前发不出去。
    let oldest_valid_root = readable_roots(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| !abandoned_by_table(root, &table))
    .map(|root| root.checkpoint_txg)
    .min();
    let reclaimed = allocator.reclaim_released_up_to(reclaim_floor(new_floor, oldest_valid_root));
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let mut covered: Vec<DeviceIdentity> = Vec::new();
    let mut publishes = Vec::new();
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let next = publish_version(
            &mut pool,
            allocator,
            PublishPlan {
                txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
                counter: current.record.counter + 1,
                transaction: 0,
                instance: current.root.instance,
                back_chain: back_chain_of(&current.record_bytes),
                file: None,
                instance_table: InstanceTablePlan::Carry(current.root.instance_table),
                tree_birth_txg: current.tree_birth_txg(),
                tree_identifier_watermark: current.root.tree_identifier_watermark,
                rollback_floor: new_floor,
            },
            Some(&*current),
        )?;
        let device = device_of_txg(next.root.checkpoint_txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
        publishes.push(next.clone());
        *current = next;
    }
    Ok(RaisedFloor {
        ceiling,
        publishes,
        reclaimed,
    })
}

/// 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘：可写挂载与回退共用的后半段。
fn establish_instance<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    mut allocator: PoolAllocator,
    start: InstanceStart,
) -> Result<Mounted, MountError> {
    let isolated_slots_per_device: Vec<(DeviceIdentity, u64)> = allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .collect();
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut pool).map_err(MountError::Acquisition)?;

    // 写行：给 [max(上一个实例, 1), 新实例) 里每个实例各一行——上一个实例 (i, T, W)（回退时是回退行，flags bit0 = 1），
    // 中间实例 (i, 0, 0)；实例 0（mkfs）不写行（D18（块里携带什么信息） 已定项 11）。
    let previous = start.previous;
    let mut rows_written = Vec::new();
    let mut table_after = start.table.clone();
    let first_row_instance = start.previous_row.instance.0.max(1);
    for row_instance in first_row_instance..instance.0 {
        let row = if row_instance == start.previous_row.instance.0 {
            InstanceRow {
                instance: InstanceGeneration(row_instance),
                selected_root_txg: start.previous_row.selected_root_txg,
                applied_transaction_high_water: start.previous_row.applied_transaction_high_water,
                is_rollback: start.previous_row.is_rollback,
            }
        } else {
            InstanceRow {
                instance: InstanceGeneration(row_instance),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
                is_rollback: false,
            }
        };
        rows_written.push(row);
        table_after.rows.push(row);
    }
    let row_publish = publish_version(
        &mut pool,
        &mut allocator,
        PublishPlan {
            txg: start.first_txg,
            counter: start.next_counter,
            transaction: 0,
            instance,
            back_chain: 0,
            file: None,
            instance_table: InstanceTablePlan::Rewrite(table_after.to_records()),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: start.effective_floor,
        },
        Some(&previous),
    )?;

    // 暖机（D16（发布语义） 已定项 8 甲′）：本实例的根覆盖两块盘之前连推空发布，次数按落点公式现算，至多根环的区域数那么多次。
    let all_devices: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    let device_of_txg = |txg: CheckpointTxg| {
        let target = target_for_publish(txg);
        parameters.region_devices[usize::try_from(target.region).expect("区域号")]
    };
    let mut covered: Vec<DeviceIdentity> = vec![device_of_txg(row_publish.root.checkpoint_txg)];
    let mut current = row_publish.clone();
    let mut warm_up_publishes = Vec::new();
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let next = publish_version(
            &mut pool,
            &mut allocator,
            PublishPlan {
                txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
                counter: current.record.counter + 1,
                transaction: 0,
                instance,
                back_chain: back_chain_of(&current.record_bytes),
                file: None,
                instance_table: InstanceTablePlan::Carry(current.root.instance_table),
                tree_birth_txg: current.tree_birth_txg(),
                tree_identifier_watermark: current.root.tree_identifier_watermark,
                rollback_floor: current.root.rollback_floor,
            },
            Some(&current),
        )?;
        let device = device_of_txg(next.root.checkpoint_txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
        warm_up_publishes.push(next.clone());
        current = next;
    }

    Ok(Mounted {
        output: MountOutput {
            instance,
            chosen_root: start.chosen_root,
            effective_root: start.effective_root,
            journal: start.journal,
            rows_written,
            row_publish,
            warm_up_publishes,
            isolated_slots_per_device,
        },
        allocator,
        current,
    })
}

/// 可写挂载：恢复 → 重建上一版与分配器 → 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘。
///
/// # Errors
/// 恢复失败、所选根下没有文件版本、实例表解不开、取号失败、发布失败。
pub fn mount_writable<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
) -> Result<Mounted, MountError> {
    let superblock = choose_superblock(&*devices)?;
    let chosen_root = choose_root(&*devices, &superblock).ok_or(RecoveryFailure::NoValidRoot)?;
    let records = scan_journal(&*devices, &superblock);
    let (journal, effective_root) = replay_journal(
        &*devices,
        &chosen_root,
        superblock.geometry.journal_ring_bytes,
        &records,
        true,
        rollback_high_water_of_root(&*devices, &chosen_root),
    );
    // 所选根自己那条记录读得出就拿它当上一版的记录；读不出（两份都撕了）就拿最大 jsn 那条顶着——本实例的第一条反向链恒 0、
    // 事务号从 1 起，上一版的记录只有 jsn 会被用到，而 jsn 下面另算。
    let own_record = records
        .values()
        .find(|record| {
            record.instance == effective_root.instance
                && record.checkpoint_txg == effective_root.checkpoint_txg
        })
        .or_else(|| records.values().max_by_key(|record| record.counter))
        .cloned()
        .ok_or(MountError::NoPublishedVersion)?;
    let own_record_bytes = own_record.to_bytes();
    let previous = rebuild_version(&*devices, &effective_root, own_record, own_record_bytes)
        .map_err(map_rebuild_failure)?;
    let table = InstanceTableRecords::parse(&previous.unit(TransactionUnit::InstanceTable).bytes)
        .ok_or(MountError::InstanceTableMalformed)?;
    let next_counter = records
        .values()
        .map(|record| record.counter)
        .max()
        .unwrap_or(0)
        + 1;
    let first_txg = first_txg_of_new_instance(devices, &superblock, &records);
    let (allocator, effective_floor) = rebuilt_allocator(
        devices,
        &superblock,
        &previous,
        &|_| false,
        ShadowLedger::On,
    )?;
    let previous_row = PreviousInstanceRow {
        instance: effective_root.instance,
        selected_root_txg: effective_root.checkpoint_txg,
        applied_transaction_high_water: journal.maximum_applied_transaction,
        is_rollback: false,
    };
    establish_instance(
        parameters,
        devices,
        allocator,
        InstanceStart {
            chosen_root,
            effective_root,
            journal,
            previous,
            table,
            previous_row,
            first_txg,
            next_counter,
            effective_floor,
        },
    )
}

/// 管理员回退（D23（journal 的角色与格式） 已定项 14 的显式例外）：带外选一条回退候选集里的旧根 R_old，不施加它之后的任何记录，
/// 取新实例代号，在 R_old 指着的那一版实例表上写回退行 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)，回退行、这次发布的单元与
/// 第一个新根同一次发布；只被被抛弃根引用的槽由影子账隔离（D28（挂载期承诺量） 已定项 1 第九项）；之后暖机同可写挂载。
/// 新实例的第一条 jsn 接在 R_old 自己那条记录之后（C340（回退之后记录链从哪条之后接没有定义） 取 P1，预想、等用户定）。
///
/// # Errors
/// 目标根不在根环、不在候选集、它自己那条记录读不出、被抛弃根的账读不出、取号或发布失败。
pub fn mount_rollback<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    target: RollbackTarget,
    shadow_ledger: ShadowLedger,
) -> Result<Mounted, MountError> {
    let superblock = choose_superblock(&*devices)?;
    let newest_root = choose_root(&*devices, &superblock).ok_or(RecoveryFailure::NoValidRoot)?;
    let records = scan_journal(&*devices, &superblock);
    let roots = readable_roots(
        &*devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let target_root = roots
        .iter()
        .find(|root| {
            root.instance == target.instance && root.checkpoint_txg == target.checkpoint_txg
        })
        .copied()
        .ok_or(MountError::RollbackTargetNotInRing(target))?;
    // 候选集：按最新根指着的实例表判仍然有效——(i, T) 可选 ⟺ 无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti；且 txg ≥ F_生效。
    let newest_table = instance_table_of_root(&*devices, &newest_root)
        .ok_or(MountError::InstanceTableMalformed)?;
    // txg ≥ F_生效（各幸存盘所带 F 最大值的最小值），不是最新根自己带的 F（步 4 / 步 5 代码三方第一轮正推腿判「窄化」）。
    let effective_floor = effective_rollback_floor(
        &*devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    if target.checkpoint_txg < effective_floor {
        return Err(MountError::RollbackTargetNotACandidate {
            target,
            reason: "txg 低于生效的回退下界 F",
        });
    }
    if newest_table
        .rows
        .iter()
        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg)
    {
        return Err(MountError::RollbackTargetNotACandidate {
            target,
            reason: "实例表里那个实例的行 T 更小：这是被抛弃时间线的根",
        });
    }
    // R_old 自己那条记录读得出就拿它当上一版的记录（只有 jsn 会被用到），读不出就拿最大 jsn 那条顶着，与可写挂载同一条路。
    let own_record = records
        .values()
        .find(|record| {
            record.instance == target.instance && record.checkpoint_txg == target.checkpoint_txg
        })
        .or_else(|| records.values().max_by_key(|record| record.counter))
        .cloned()
        .ok_or(MountError::NoPublishedVersion)?;
    // 新实例的第一条 jsn 接在环里最大的 jsn 之后（C340（回退之后记录链从哪条之后接没有定义） 的 P2；预想、等用户定）：
    // 接 R_old 那条之后（P1）会把「B 的记录已提交、根还没落盘」那个窗口里 B 的记录盖掉，崩在回退生效之前那次恢复就不再
    // 「与没发起回退时相同」（步 4 / 步 5 代码三方第一轮云端攻方腿打中）。被抛弃的记录原样留在盘上，靠候选集与实例表挡。
    let highest_counter = records
        .values()
        .map(|record| record.counter)
        .max()
        .unwrap_or(0);
    let next_counter = highest_counter + 1;
    let own_record_bytes = own_record.to_bytes();
    let previous = rebuild_version(&*devices, &target_root, own_record, own_record_bytes)
        .map_err(map_rebuild_failure)?;
    let table = InstanceTableRecords::parse(&previous.unit(TransactionUnit::InstanceTable).bytes)
        .ok_or(MountError::InstanceTableMalformed)?;
    // 不施加 R_old 之后的任何记录：扫描报告只记环里有多少条自证过的。
    let journal = JournalScanReport {
        valid_records: records.len(),
        above_water: 0,
        prefix_applied: 0,
        verification_passed: 0,
        verification_failed: 0,
        maximum_applied_transaction: 0,
    };
    let first_txg = first_txg_of_new_instance(devices, &superblock, &records);
    // 影子账：这次回退新抛弃的根 = 根环里 (txg, 实例) 大于 R_old 的每一条可读根；它们引用的每一个槽都隔离（保守读法，见
    // `DeviceFreeMap::isolate` 的注），连同按实例表早已被抛弃的根一起在重建里算。
    let newly_abandoned = |root: &RootRecord| {
        (root.checkpoint_txg, root.instance) > (target.checkpoint_txg, target.instance)
    };
    let (allocator, _) = rebuilt_allocator(
        devices,
        &superblock,
        &previous,
        &newly_abandoned,
        shadow_ledger,
    )?;
    let previous_row = PreviousInstanceRow {
        instance: target.instance,
        selected_root_txg: target.checkpoint_txg,
        applied_transaction_high_water: 0,
        is_rollback: true,
    };
    establish_instance(
        parameters,
        devices,
        allocator,
        InstanceStart {
            chosen_root: newest_root,
            effective_root: target_root,
            journal,
            previous,
            table,
            previous_row,
            first_txg,
            next_counter,
            effective_floor,
        },
    )
}

#[cfg(test)]
mod reclaim_floor_tests {
    use super::reclaim_floor;
    use crate::address::CheckpointTxg;

    /// 第 0 代根还在环里时门槛就是 F_生效；被盖之后环里最旧的有效根接管（比 F 大时取它）。
    #[test]
    fn reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root() {
        assert_eq!(
            reclaim_floor(CheckpointTxg(11), Some(CheckpointTxg(0))),
            CheckpointTxg(11)
        );
        assert_eq!(
            reclaim_floor(CheckpointTxg(0), Some(CheckpointTxg(25))),
            CheckpointTxg(25)
        );
        assert_eq!(
            reclaim_floor(CheckpointTxg(11), Some(CheckpointTxg(25))),
            CheckpointTxg(25)
        );
        assert_eq!(reclaim_floor(CheckpointTxg(11), None), CheckpointTxg(11));
    }
}
```

## 二、新文件 `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs` 全文（未跟踪）

```rust
//! 里程碑「第二个事务」步 4 的验收：发布 C 之后进程退出、重开走管理员回退到 A 的根 (1, 3)——不施加 A 之后的任何记录、
//! 取实例代号 3、在 A 那一版实例表上写回退行 (1, 3, 0) 与中间实例行 (2, 0, 0)、发布 D（txg 9，jsn 接在 A 那条记录之后 = 4）、
//! 暖机一次（txg 10 落盘 1）——冷启动择实例 3 的根读回第一次的内容；只被被抛弃根引用的槽由影子账隔离；池级 checker 全绿。

mod common;

use common::{build_pool, file_content, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::mount::{
    mount_rollback, mount_writable, InstanceRow, MountError, Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    choose_root, choose_superblock, recover, replay_journal, scan_journal, JournalPolicy,
    RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, TransactionOutput, TransactionUnit,
};
use std::collections::BTreeSet;

const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn second_content() -> Vec<u8> {
    content_of(SECOND_FILE_BYTES, 3)
}

fn third_content() -> Vec<u8> {
    content_of(THIRD_FILE_BYTES, 11)
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 固定脚本到 C：A、B（实例 1）、重开取号 2、写行、暖机两次、C（实例 2）。
fn build_through_third_publish(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current;
    overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(2));
    pool
}

fn first_root() -> RollbackTarget {
    RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(3),
    }
}

/// 进程退出、重开走回退到 A 的根；回来的可写态装回 pool。
fn rollback_to_first_root(pool: &mut BuiltPool, shadow_ledger: ShadowLedger) -> Mounted {
    let mut devices = pool.reopen_recorded();
    let rolled_back =
        mount_rollback(&parameters(), &mut devices, first_root(), shadow_ledger).expect("回退");
    pool.devices = Some(devices);
    pool.allocator = rolled_back.allocator.clone();
    pool.output = rolled_back.current.clone();
    rolled_back
}

fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(CheckpointTxg(txg));
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

/// 一版账里这块盘上占着的每个槽（记录按跨度展开）。
fn slots_of(output: &TransactionOutput, device: DeviceIdentity) -> BTreeSet<u64> {
    output
        .allocation_records
        .iter()
        .filter(|record| record.device == device)
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect()
}

/// 验收第一条：回退行 (1, 3, 0)、中间实例行 (2, 0, 0)；D 的根 (3, 9)、jsn 4（接在 A 那条之后，C340 取 P1）、事务号 0、反向链 0，
/// 重写实例表 + 四个固定点单元；暖机一次落到另一块盘；一条记录都不施加；冷启动读回第一次的内容；被抛弃根独占的槽逐盘 34 个、
/// D 与暖机一个都不落在上面；checker 全绿（I-3.1 的并集按实例表把被抛弃的根排除，I-3.8 看见回退行）。
#[test]
fn rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content(
) {
    let mut pool = build_through_third_publish("step-four-rollback");
    let third = pool.output.clone();
    let rolled_back = rollback_to_first_root(&mut pool, ShadowLedger::On);
    let output = &rolled_back.output;
    assert_eq!(output.instance, InstanceGeneration(3));
    assert_eq!(
        output.rows_written,
        vec![
            InstanceRow {
                instance: InstanceGeneration(1),
                selected_root_txg: CheckpointTxg(3),
                applied_transaction_high_water: 0,
                is_rollback: true,
            },
            InstanceRow {
                instance: InstanceGeneration(2),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
                is_rollback: false,
            },
        ],
        "回退行与中间实例行"
    );
    assert_eq!(output.journal.prefix_applied, 0, "A 之后的记录一条都不施加");
    assert_eq!(
        (
            output.effective_root.instance,
            output.effective_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(3))
    );
    let rollback_publish = &output.row_publish;
    assert_eq!(
        (
            rollback_publish.root.instance,
            rollback_publish.root.checkpoint_txg,
            rollback_publish.record.counter,
            rollback_publish.record.transaction,
            rollback_publish.record.back_chain
        ),
        (InstanceGeneration(3), CheckpointTxg(9), 9, 0, 0),
        "D：txg = max(根环 8, 记录 8) + 1；jsn 接在环里最大的 8 之后（C340 取 P2，被抛弃的记录一条不盖）；本实例第一条反向链 0"
    );
    assert_eq!(
        rollback_publish.rewritten,
        vec![
            TransactionUnit::InstanceTable,
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ]
    );
    assert_eq!(
        output.warm_up_publishes.len(),
        1,
        "D 落盘 0、txg 10 落盘 1，一次就够"
    );
    let warm_up = &output.warm_up_publishes[0];
    assert_eq!(warm_up.root.checkpoint_txg, CheckpointTxg(10));
    assert_ne!(region_device(9), region_device(10));
    assert_eq!(warm_up.record.counter, 10);

    // 影子账：被抛弃的根 B、(2, 5)、(2, 6)、(2, 7)、C 引用而 A 不引用的槽——B 10、写行 6、暖机 4 + 4、C 10 = 34 个槽，逐盘。
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let abandoned: BTreeSet<u64> = slots_of(&third, device)
            .difference(&slots_of(rollback_publish, device))
            .copied()
            .collect();
        assert_eq!(abandoned.len(), 34, "盘 {device:?} 上只被被抛弃根引用的槽");
        // 影子账按保守读法隔离被抛弃根引用的每一个槽：34 个独占的加 mkfs 实例表那 2 个（A 与 B 的根都引用）= 36。
        assert!(
            output.isolated_slots_per_device.contains(&(device, 36)),
            "隔离的槽数（独占 34 + 两边都引用的 mkfs 实例表 2）{:?}",
            output.isolated_slots_per_device
        );
        for publish in std::iter::once(rollback_publish).chain(output.warm_up_publishes.iter()) {
            for placement in publish.placements() {
                for slot in placement.slot.0..placement.slot.0 + placement.span {
                    assert!(
                        !abandoned.contains(&slot),
                        "txg {} 的落点 {slot} 落在被抛弃根引用的槽上",
                        publish.root.checkpoint_txg.0
                    );
                }
            }
        }
    }

    let image = pool.memory_pool();
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(3), CheckpointTxg(10)),
            content: file_content()
        },
        "{:?}",
        report.journal
    );
    assert_eq!(
        report.journal.valid_records, 10,
        "jsn 1–8 原样在（P2 一条不盖）、9 是 D、10 是暖机"
    );
    assert_eq!(
        report.journal.prefix_applied, 0,
        "(3, 10) 之后 jsn 11 一条都没有"
    );
    let verdicts = check_pool_image(&image);
    for (invariant, verdict) in &verdicts {
        assert_eq!(
            *verdict,
            InvariantVerdict::Holds,
            "{invariant} 在回退之后的镜像上要成立"
        );
    }
    for must_hold in ["I-3.1", "I-3.8", "I-5.2", "I-7.2"] {
        assert!(
            verdicts.iter().any(|(name, _)| *name == must_hold),
            "{must_hold} 真被判过"
        );
    }
}

/// 回退候选集（D23 已定项 14）：实例表里有行 (i, Ti, Wi) 的实例，只有 T ≤ Ti 的根可选——B 的根 (1, 4) 与 C 的根 (2, 8) 都是被抛弃时间线的；
/// 根环里没有的 (1, 42) 另报。
#[test]
fn rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused() {
    let mut pool = build_through_third_publish("step-four-refused");
    rollback_to_first_root(&mut pool, ShadowLedger::On);
    for (target, expected) in [
        (
            RollbackTarget {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(4),
            },
            "实例表里那个实例的行 T 更小：这是被抛弃时间线的根",
        ),
        (
            RollbackTarget {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(8),
            },
            "实例表里那个实例的行 T 更小：这是被抛弃时间线的根",
        ),
    ] {
        let mut devices = pool.reopen_recorded();
        let refused = mount_rollback(&parameters(), &mut devices, target, ShadowLedger::On);
        pool.devices = Some(devices);
        match refused {
            Err(MountError::RollbackTargetNotACandidate {
                target: reported,
                reason,
            }) => assert_eq!((reported, reason), (target, expected)),
            other => panic!(
                "{target:?} 该被拒：{:?}",
                other.map(|mounted| mounted.output.instance)
            ),
        }
    }
    let mut devices = pool.reopen_recorded();
    let missing = RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(42),
    };
    let refused = mount_rollback(&parameters(), &mut devices, missing, ShadowLedger::On);
    pool.devices = Some(devices);
    assert!(
        matches!(refused, Err(MountError::RollbackTargetNotInRing(reported)) if reported == missing),
        "根环里没有 (1, 42)"
    );
}

/// C314（回退可以复用被抛弃的根引用的单元） 那一格的必红，影子账开关强制进入：关掉影子账，回退之后再发两版文件，
/// 数据单元落回 B 与 C 的数据槽（50182、50184）；把实例 3 的四个根槽都改坏，恢复挂上 C 的根 (2, 8)，它的数据单元已被盖掉，读不出第三次的内容。
/// 影子账开着：两版数据落 50186、50188，同样改坏四个根槽之后恢复挂上 C 的根、第三次的内容原样读回。
#[test]
fn without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit(
) {
    for (shadow_ledger, expected_slots, expect_third_content_readable) in [
        (ShadowLedger::Off, [50182, 50184], false),
        (ShadowLedger::On, [50186, 50188], true),
    ] {
        let mut pool = build_through_third_publish("step-four-shadow");
        let rolled_back = rollback_to_first_root(&mut pool, shadow_ledger);
        let expected_isolated = if shadow_ledger == ShadowLedger::On {
            36
        } else {
            0
        };
        assert!(
            rolled_back
                .output
                .isolated_slots_per_device
                .iter()
                .all(|(_, isolated)| *isolated == expected_isolated),
            "{shadow_ledger:?}：{:?}",
            rolled_back.output.isolated_slots_per_device
        );
        let fourth = overwrite_in_process(&mut pool, &content_of(3000, 5), InstanceGeneration(3));
        let fifth = overwrite_in_process(&mut pool, &content_of(3100, 9), InstanceGeneration(3));
        assert_eq!(
            [
                fourth.data_pointer.locations[0].slot.0,
                fifth.data_pointer.locations[0].slot.0
            ],
            expected_slots,
            "{shadow_ledger:?} 下回退之后两版的数据落点"
        );
        let mut image = pool.memory_pool();
        for txg in [9u64, 10, 11, 12] {
            let target = target_for_publish(CheckpointTxg(txg));
            image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
        }
        let report = recover(&image, JournalPolicy::Consult);
        if expect_third_content_readable {
            assert_eq!(
                report.outcome,
                RecoveryOutcome::FileRead {
                    root: (InstanceGeneration(2), CheckpointTxg(8)),
                    content: third_content()
                },
                "影子账开着：C 引用的单元一个没被盖，回到 C 读第三次的内容"
            );
        } else {
            assert_ne!(
                report.outcome,
                RecoveryOutcome::FileRead {
                    root: (InstanceGeneration(2), CheckpointTxg(8)),
                    content: third_content()
                },
                "影子账关着：C 的数据单元已被第五版盖掉，第三次的内容读不回来"
            );
        }
    }
}

/// 前缀第五条（D23 已定项 14）：所选根的实例有回退行时，该实例的记录只施加到回退行的 W 为止——直接喂 `replay_journal`：
/// 到 B 为止的镜像上选 A 的根 (1, 3)，不带回退行施加 B 那条（事务号 2）；W = 0 一条都不施加；W = 2 施加到 B；W = 1 停在 B 之前。
#[test]
fn the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water() {
    let mut pool = build_pool("step-four-cap");
    overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    let newest = choose_root(&image, &superblock).expect("B 的根");
    assert_eq!(newest.checkpoint_txg, CheckpointTxg(4));
    let records = scan_journal(&image, &superblock);
    let roots = singlefs_core::recovery::readable_roots(
        &image,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    let first = roots
        .iter()
        .find(|root| root.checkpoint_txg == CheckpointTxg(3))
        .copied()
        .expect("A 的根在环里");
    for (high_water, expected_applied, expected_txg) in [
        (None, 1, 4u64),
        (Some(0), 0, 3),
        (Some(1), 0, 3),
        (Some(2), 1, 4),
    ] {
        let (report, effective) = replay_journal(
            &image,
            &first,
            superblock.geometry.journal_ring_bytes,
            &records,
            true,
            high_water,
        );
        assert_eq!(
            (report.prefix_applied, effective.checkpoint_txg.0),
            (expected_applied, expected_txg),
            "回退行 W = {high_water:?}"
        );
    }
}

/// C340 取 P2：回退之后新实例的第一条 jsn 接在环里最大的 jsn 之后，被抛弃发布的记录一条不盖——B 的记录已提交、根还没落盘时发起回退，
/// 崩在回退生效之前那次恢复才仍「与没发起回退时相同」（步 4 / 步 5 代码三方第一轮云端攻方腿打中 P1 盖掉 B 那条）。
/// 影子账只住内存，所以每次挂载都重算：回退之后普通重开一次，被抛弃根引用的槽照样隔离（同一轮攻方腿打中重开后隔离归零）。
#[test]
fn rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation() {
    let mut pool = build_through_third_publish("step-four-p2-remount");
    let rolled_back = rollback_to_first_root(&mut pool, ShadowLedger::On);
    assert_eq!(
        rolled_back.output.row_publish.record.counter, 9,
        "D 的 jsn 接在 C 的 8 之后"
    );
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    let records = scan_journal(&image, &superblock);
    for (instance, counter) in [(1u32, 4u64), (2, 5), (2, 8), (3, 9), (3, 10)] {
        assert!(
            records.contains_key(&(InstanceGeneration(instance), counter)),
            "记录 ({instance}, jsn {counter}) 该原样在环里：{:?}",
            records.keys().collect::<Vec<_>>()
        );
    }
    let mut devices = pool.reopen_recorded();
    let remounted = mount_writable(&parameters(), &mut devices).expect("回退之后普通重开");
    pool.devices = Some(devices);
    pool.allocator = remounted.allocator.clone();
    pool.output = remounted.current.clone();
    assert_eq!(remounted.output.instance, InstanceGeneration(4));
    assert_eq!(
        remounted.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 36), (DeviceIdentity(1), 36)],
        "按 D 那一版实例表判被抛弃的根（B、实例 2 的四条）引用的槽，普通重开照样隔离"
    );
    // 被抛弃根引用的槽：mkfs 实例表 50176–50177、B 的数据 50182–50183、C 的数据 50184–50185，以及它们的节点。
    let abandoned: BTreeSet<u64> = (50176..50178).chain(50182..50186).collect();
    let next = overwrite_in_process(&mut pool, &content_of(2100, 41), InstanceGeneration(4));
    for placement in next.placements() {
        for slot in placement.slot.0..placement.slot.0 + placement.span {
            assert!(
                !abandoned.contains(&slot),
                "重开后的发布落到了被抛弃根引用的槽 {slot}"
            );
        }
    }
}
```

## 二、新文件 `crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs` 全文（未跟踪）

```rust
//! 里程碑「第二个事务」步 5 的验收：回退之后再覆盖写四次（txg 11–14；第一次把 A 的八个单元释放、释放代 11），抬回退下界 F 到 11
//! （上限 = min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根) = min(13, 11)；两次空发布 txg 15、16 让两块盘各有一条带 F = 11 的根），
//! 释放代 ≤ 11 的落点回收、之后的仍在 defer 队列里；发布 E（txg 17）把数据单元落回 50178（mkfs 树表那 1 槽回收了、50179 从没分配过；mkfs 实例表那片 50176 也回收了但 B 的根还引用它、影子账隔离着；A 的数据单元 50180 排在后面）；冷启动读回 E；checker 全绿。
//! 必红：不抬 F 就回收（复用窗口置 0），第 0 代根还在候选集里（F = 0）、它们引用的 mkfs 树表单元被 E 盖掉，checker 在它们上判 I-2.1 红。

mod common;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::Placement;
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::{
    mount_rollback, mount_writable, raise_rollback_floor, MountError, RaisedFloor, RollbackTarget,
    ShadowLedger,
};
use singlefs_core::recovery::{
    choose_superblock, readable_roots, recover, JournalPolicy, RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 固定脚本到 D：A、B、重开取号 2、写行、暖机两次、C、重开回退到 (1, 3)、D、暖机一次。
fn build_through_rollback(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted.current;
    overwrite_in_process(&mut pool, &content_of(2500, 11), InstanceGeneration(2));
    let mut reopened = pool.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut reopened,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    )
    .expect("回退");
    pool.devices = Some(reopened);
    pool.allocator = rolled_back.allocator;
    pool.output = rolled_back.current;
    pool
}

/// 回退之后再覆盖写四次（txg 11–14）：第一次释放 A 的八个单元（释放代 11）。
fn four_overwrites_after_the_rollback(pool: &mut BuiltPool) -> Vec<TransactionOutput> {
    [17usize, 19, 23, 29]
        .iter()
        .map(|seed| {
            overwrite_in_process(pool, &content_of(3000 + seed, *seed), InstanceGeneration(3))
        })
        .collect()
}

fn raise_floor(pool: &mut BuiltPool, new_floor: CheckpointTxg) -> Result<RaisedFloor, MountError> {
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let raised = raise_rollback_floor(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        new_floor,
    );
    pool.output = current;
    raised
}

fn newest_root_floor(pool: &BuiltPool) -> CheckpointTxg {
    let image = pool.memory_pool();
    let superblock = choose_superblock(&image).expect("超级块");
    readable_roots(
        &image,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
    .into_iter()
    .max_by_key(|root| (root.checkpoint_txg, root.instance))
    .expect("根")
    .rollback_floor
}

/// 验收第一、二条：上限 11；两次空发布带 F = 11 落到两块盘；A 的八个落点（10 槽）回收、defer 队列从 40 槽减到 30；E 的数据单元落 50180、
/// 它的分配记录改写成代 17、未释放；后释放的（代 12–14）仍占着；冷启动读回 E；根记录 F = 11；checker 全绿（A 的根在 F 之下、不在候选集）。
#[test]
fn raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it(
) {
    let mut pool = build_through_rollback("step-five-reuse");
    let overwrites = four_overwrites_after_the_rollback(&mut pool);
    assert_eq!(overwrites[0].root.checkpoint_txg, CheckpointTxg(11));
    assert_eq!(overwrites[3].root.checkpoint_txg, CheckpointTxg(14));
    for device in &pool.allocator.devices {
        assert_eq!(
            device.deferred_slots(),
            51,
            "A 的账里 mkfs 树表 1 槽已释放；D 释放 A 的四个固定点单元与 mkfs 实例表（6 槽）、暖机释放 D 的四个（4 槽）、四次覆盖写各释放上一版的 10 个槽"
        );
    }
    let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F 到 11");
    assert_eq!(
        raised.ceiling,
        CheckpointTxg(11),
        "min(每块盘最新的有效根 14 / 13, 第 4 新的非空 11)"
    );
    assert_eq!(
        raised
            .publishes
            .iter()
            .map(|publish| (
                publish.root.checkpoint_txg.0,
                publish.root.rollback_floor.0,
                publish.record.transaction
            ))
            .collect::<Vec<_>>(),
        vec![(15, 11, 0), (16, 11, 0)],
        "两次带新 F 的空发布"
    );
    assert!(raised.reclaimed.contains(&Placement {
        slot: SlotNumber(50180),
        span: 2
    }));
    assert_eq!(
        raised.reclaimed.len(),
        18,
        "释放代 ≤ 11 的落点：A 放掉的 mkfs 树表（代 3）、D 放掉的 5 个（代 9）、暖机放掉的 4 个（代 10）、第一次覆盖写放掉 A 的 8 个（代 11）"
    );
    for device in &pool.allocator.devices {
        assert_eq!(
            device.deferred_slots(),
            38,
            "回收了 1 + 6 + 4 + 10 = 21 个槽，抬 F 的两次空发布又各放掉上一版的 4 个"
        );
        assert!(device.is_free(SlotNumber(50180)) && device.is_free(SlotNumber(50181)));
        for later in &overwrites[..3] {
            let slot = later.data_pointer.locations[0].slot;
            assert!(
                !device.is_free(slot),
                "释放代 > F 的数据单元 {slot:?} 仍占着"
            );
        }
    }
    let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
    assert_eq!(reuse.root.checkpoint_txg, CheckpointTxg(17));
    assert_eq!(
        reuse.data_pointer.locations[0].slot,
        SlotNumber(50178),
        "E 的数据单元落回最低的可再分配偶数槽对 50178–50179：mkfs 树表那 1 槽（A 换下、释放代 3）回收了、50179 从没分配过；mkfs 实例表那片 50176 虽被 D 放掉、也回收了，但 B 的根还引用它、被影子账隔离；A 的数据单元 50180 排在后面"
    );
    let reused_record = pool
        .allocator
        .record_for(DeviceIdentity(0), SlotNumber(50178))
        .expect("50178 的记录");
    assert_eq!(
        (
            reused_record.generation,
            reused_record.is_released,
            reused_record.span_slots
        ),
        (CheckpointTxg(17), false, 2),
        "复用时那条记录改写"
    );
    assert_eq!(
        pool.allocator
            .records()
            .iter()
            .filter(|record| record.device == DeviceIdentity(0) && record.slot == SlotNumber(50178))
            .count(),
        1,
        "同盘同槽只有一条记录"
    );
    for device in &pool.allocator.devices {
        assert_eq!(device.deferred_slots(), 48, "E 又释放了第四版的 10 个槽");
    }
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(11));
    let image = pool.memory_pool();
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(3), CheckpointTxg(17)),
            content: content_of(2000, 31)
        },
        "{:?}",
        report.journal
    );
    let verdicts = check_pool_image(&image);
    for (invariant, verdict) in &verdicts {
        assert_eq!(
            *verdict,
            InvariantVerdict::Holds,
            "{invariant} 在 E 之后的镜像上要成立"
        );
    }
    for must_hold in ["I-2.1", "I-3.1", "I-5.2", "I-7.2"] {
        assert!(
            verdicts.iter().any(|(name, _)| *name == must_hold),
            "{must_hold} 真被判过"
        );
    }
}

/// 生效（D16 已定项 1）：每块幸存盘上都有带新 F 的持久根才生效，恢复后生效值 = 各盘所带 F 最大值的最小值——把 txg 16 的根槽（盘 1 上唯一带 F = 11 的根）改坏，
/// 重开之后 F_生效 回到 0：所选根是 txg 15、链上 txg 16 的记录照样施加（同实例），盘上写着已释放的 59 个槽一个都不回收（新实例写行再放 6、暖机两次各放 4 ⇒ 73）；根槽都好时回收 21 个槽（59 − 21 + 14 = 52）。
#[test]
fn one_device_carrying_the_floor_alone_does_not_take_effect_on_remount() {
    for (damage_second_carrier, expected_deferred, expected_chosen_txg) in
        [(false, 52, 16), (true, 73, 15)]
    {
        let mut pool = build_through_rollback("step-five-effective");
        four_overwrites_after_the_rollback(&mut pool);
        let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F");
        let second_carrier = raised.publishes[1].root.checkpoint_txg;
        assert_eq!(second_carrier, CheckpointTxg(16));
        let mut devices = pool.reopen_recorded();
        if damage_second_carrier {
            let target = target_for_publish(second_carrier);
            let device =
                parameters().region_devices[usize::try_from(target.region).expect("区域号")];
            let offset = slot_offset(target, 4096);
            let (_, recorded) = devices
                .iter_mut()
                .find(|(identity, _)| *identity == device)
                .expect("那块盘");
            let mut bytes = vec![0u8; 4096];
            recorded.read_at(offset, &mut bytes).expect("读根槽");
            bytes[100] ^= 0xff;
            recorded
                .write_at(offset, &bytes, WriteDurability::Plain)
                .expect("改坏根槽");
        }
        let mounted = mount_writable(&parameters(), &mut devices).expect("重开");
        pool.devices = Some(devices);
        assert_eq!(
            mounted.output.chosen_root.checkpoint_txg,
            CheckpointTxg(expected_chosen_txg)
        );
        for device in &mounted.allocator.devices {
            assert_eq!(
                device.deferred_slots(),
                expected_deferred,
                "改坏第二块盘的载体 = {damage_second_carrier}：F_生效 = 各盘 F 最大值的最小值；数里含写行放掉的 6 与暖机两次放掉的 8"
            );
        }
    }
}

/// 上限：第 4 新的非空持久有效根是 11（非空的有 14、13、12、11；D 与暖机是空发布不算；A 的根 3 是第 5 新）⇒ 抬到 12 被拒；
/// 抬到 11 之后上限仍是 11，再抬 12 仍被拒。
#[test]
fn raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused() {
    let mut pool = build_through_rollback("step-five-ceiling");
    four_overwrites_after_the_rollback(&mut pool);
    for attempt in [0, 1] {
        let refused = raise_floor(&mut pool, CheckpointTxg(12));
        assert!(
            matches!(
                refused,
                Err(MountError::RollbackFloorAboveCeiling {
                    requested: CheckpointTxg(12),
                    ceiling: CheckpointTxg(11)
                })
            ),
            "第 {attempt} 次：上限 11"
        );
        if attempt == 0 {
            raise_floor(&mut pool, CheckpointTxg(11)).expect("抬到上限");
        }
    }
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(11));
}

/// 必红（C22（刚释放的块立即重分配）、复用窗口置 0）：不抬 F、直接把释放代 ≤ 11 的落点回收，E 落回 50178；第 0 代根还在候选集里
/// （F = 0），checker 走它们时 mkfs 树表单元的校验和与位置条目对不上 ⇒ I-2.1 红。
#[test]
fn reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red(
) {
    let mut pool = build_through_rollback("step-five-window-zero");
    four_overwrites_after_the_rollback(&mut pool);
    let reclaimed = pool.allocator.reclaim_released_up_to(CheckpointTxg(11));
    assert_eq!(reclaimed.len(), 18);
    let reuse = overwrite_in_process(&mut pool, &content_of(2000, 31), InstanceGeneration(3));
    assert_eq!(
        reuse.data_pointer.locations[0].slot,
        SlotNumber(50178),
        "mkfs 树表那片被拿走：第 0 代根（F = 0 时仍是候选）还引用它"
    );
    assert_eq!(newest_root_floor(&pool), CheckpointTxg(0), "F 没抬");
    let verdicts = check_pool_image(&pool.memory_pool());
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert!(
        violated.contains(&"I-2.1"),
        "A 的根还是候选，它的数据单元被盖了：{verdicts:?}"
    );
}

/// 回退候选集的 F 用 F_生效（各幸存盘所带 F 最大值的最小值），不是最新根自己带的 F：抬到 11 之后把盘 1 的载体（txg 16）改坏，
/// F_生效 回到 0，txg 9 的根 D 仍是候选、退得到；按最新根（txg 15，F = 11）自己的 F 判会把它拒掉。
#[test]
fn roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates() {
    let mut pool = build_through_rollback("step-five-candidate-floor");
    four_overwrites_after_the_rollback(&mut pool);
    let raised = raise_floor(&mut pool, CheckpointTxg(11)).expect("抬 F");
    let second_carrier = raised.publishes[1].root.checkpoint_txg;
    let mut devices = pool.reopen_recorded();
    let target = target_for_publish(second_carrier);
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let offset = slot_offset(target, 4096);
    let (_, recorded) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("那块盘");
    let mut bytes = vec![0u8; 4096];
    recorded.read_at(offset, &mut bytes).expect("读根槽");
    bytes[100] ^= 0xff;
    recorded
        .write_at(offset, &bytes, WriteDurability::Plain)
        .expect("改坏根槽");
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices,
        RollbackTarget {
            instance: InstanceGeneration(3),
            checkpoint_txg: CheckpointTxg(9),
        },
        ShadowLedger::On,
    )
    .expect("F_生效 是 0，txg 9 的根仍在候选集里");
    pool.devices = Some(devices);
    assert_eq!(rolled_back.output.instance, InstanceGeneration(4));
    assert_eq!(
        rolled_back.output.row_publish.root.rollback_floor,
        CheckpointTxg(0),
        "新实例的根写 F_生效"
    );
}
```

## 三、`crates/mutations.tsv` 全文

```
# crates 的变异表：每行六段，制表符分隔——变异名 <TAB> 文件 <TAB> 原文 <TAB> 替换文 <TAB> cargo test 的参数 <TAB> 必须红的测试名。
# 原文 / 替换文里的 \n 表示换行；原文在文件里必须恰好命中一次（锚点腐化就红，门禁 59 号）。
# 门禁 59 号把仓拷到临时目录、逐条改坏、跑点名的测试、要求那条测试判红、还原；一条没红就整道红。
# 这是 show-me-test.md「存进仓的变异清单，交给门禁反复复跑」那一条在 crates 上的形态：
# 三方对抗每一轮打中之后的改法，各留一条「改回去它就红」的变异，改法被悄悄撤回时这里先响。
逐盘核退回只核总数	crates/singlefs-core/src/recovery.rs	        && placement_sets.all(|placements| placements == first_device_placements)	        && placement_sets.all(|_placements| true)	-p singlefs-core --lib -- allocation_records_per_device	placement_recorded_on_one_device_only_is_rejected_even_when_the_total_is_even
同盘槽号不核唯一	crates/singlefs-core/src/recovery.rs	        if !slots_per_device	        if false && !slots_per_device	-p singlefs-core --lib -- allocation_records_per_device	two_records_for_the_same_slot_on_the_same_device_are_rejected
空闲不随分配减（I-5.2 要红）	crates/singlefs-core/src/allocator.rs	        self.free_slots -= span;	        // 变异：空闲不减	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
释放时清位图（立即复用）	crates/singlefs-core/src/allocator.rs	        self.deferred_slots += span;\n    }	        self.deferred_slots += span;\n        for index in start..end {\n            self.allocated[index] = false;\n        }\n    }	-p singlefs-harness --test second_transaction_step_one_overwrite -- released_placements	released_placements_are_not_handed_out_again_before_reclaim_exists
分配记录树装不下不判	crates/singlefs-core/src/transaction.rs	    if records_after_this_publish > allocation_node_capacity {	    if false && records_after_this_publish > allocation_node_capacity {	-p singlefs-harness --test second_transaction_step_one_overwrite -- repeated_overwrites	repeated_overwrites_report_a_full_allocation_node_instead_of_panicking
释放退回按提示	crates/singlefs-core/src/transaction.rs	            placements_to_release_via_mapping(previous_version, allocator, &rewritten)?	            previous_version.placements()	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_goes_through	release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released
映射查到 key 就算（落点指错不核记录）	crates/singlefs-core/src/transaction.rs	        let record = allocator.record_for(locations[0].device, slot).ok_or(	        let record = allocator\n            .record_for(locations[0].device, slot)\n            .or(allocator.records().first())\n            .ok_or(	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_reports_a_mapping_entry	release_reports_a_mapping_entry_whose_slot_has_no_record_or_the_wrong_span_instead_of_panicking
失败的发布不退回分配器	crates/singlefs-core/src/transaction.rs	    if outcome.is_err() {	    if false && outcome.is_err() {	-p singlefs-harness --test second_transaction_step_one_overwrite -- publish_running_out_of_space_midway	publish_running_out_of_space_midway_leaves_the_allocator_as_it_was
内容超长不判	crates/singlefs-core/src/transaction.rs	        if file.content.len() > data_unit_capacity {	        if false && file.content.len() > data_unit_capacity {	-p singlefs-harness --test second_transaction_step_one_overwrite -- content_larger_than_a_data_unit	content_larger_than_a_data_unit_payload_is_refused_before_anything_is_touched
oracle 只比 txg 不比实例	crates/singlefs-harness/src/crash.rs	        if (effective_txg, effective_instance) < (newest_txg, newest_instance) {	        if effective_txg < newest_txg {	-p singlefs-harness --lib -- oracle_instance_tests	landing_on_the_lower_instance_of_the_same_txg_is_a_violation
oracle 只按 txg 找版本	crates/singlefs-harness/src/crash.rs	        version.checkpoint_txg == effective_txg && version.instance == effective_instance	        version.checkpoint_txg == effective_txg	-p singlefs-harness --lib -- oracle_instance_tests	the_same_txg_from_two_instances_are_two_versions
oracle 放过没版本的更新根	crates/singlefs-harness/src/crash.rs	                (candidate_version.checkpoint_txg, candidate_version.instance)\n                    < (effective_txg, effective_instance)	                false	-p singlefs-harness --lib -- oracle_instance_tests	newer_root_without_any_version_reporting_no_file_is_violation
Ignore 那一遍恢复不过 oracle	crates/singlefs-harness/src/crash.rs	        tally.ignored_violations += 1;	        tally.ignored_violations += 0;	-p singlefs-harness --test second_transaction_step_zero_layer0 -- targeted_controls	targeted_controls_on_the_second_publish_go_red_where_they_should
步 1 变异：不释放旧落点	crates/singlefs-core/src/transaction.rs	            placements_to_release_via_mapping(previous_version, allocator, &rewritten)?	            Vec::<Placement>::new()	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
步 1 变异：defer 行写 0	crates/singlefs-core/src/transaction.rs	            device_map.deferred_slots() * SLOT_BYTES,	            0,	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
步 1 变异：事务号不加一	crates/singlefs-core/src/transaction.rs	            transaction: previous.record.transaction + 1,	            transaction: previous.record.transaction,	-p singlefs-harness --test second_transaction_step_one_overwrite -- overwrite_publishes	overwrite_publishes_the_second_version_through_the_same_commit_shape
步 1 变异：改动计数留 1	crates/singlefs-core/src/transaction.rs	            change_count: txg.0,	            change_count: 1,	-p singlefs-harness --test second_transaction_step_one_overwrite -- overwrite_publishes	overwrite_publishes_the_second_version_through_the_same_commit_shape
步 2 变异：已分配行不随分配更新（I-3.1 要红）	crates/singlefs-core/src/transaction.rs	            device_map.allocated_slots() * SLOT_BYTES,	            0,	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
步 2 变异：忘了改写分配记录	crates/singlefs-core/src/allocator.rs	            record.is_released = true;	            record.is_released = false;	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
步 3：不写行（实例表照抄）	crates/singlefs-core/src/mount.rs	            instance_table: InstanceTablePlan::Rewrite(table_after.to_records()),	            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
步 3：暖机只推一次	crates/singlefs-core/src/mount.rs	        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(warm_up_publishes.len()).expect("次数") < 1	-p singlefs-harness --test second_transaction_step_three_second_instance -- damaging_every_instance_two_root	damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device
步 3：前缀跨实例边界（把别的实例的记录也接上）	crates/singlefs-core/src/recovery.rs	            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water	            (record.instance, record.checkpoint_txg) > water	-p singlefs-harness --test first_transaction_step_seven_layer0 -- layer0_partial_enumeration	layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape
步 3：链首锚点错一位（接在所选根自己那条记录之后第二条）	crates/singlefs-core/src/recovery.rs	        root_own_record_counter.map(|counter| (root.instance, counter + 1));	        root_own_record_counter.map(|counter| (root.instance, counter + 2));	-p singlefs-harness --test second_transaction_step_three_second_instance -- stray_record_of_the_previous_instance	stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row
步 3：I-3.8 判定恒真	crates/singlefs-checker/src/walk.rs	            unique && below_mount_root && chain_record_last,	            unique || below_mount_root || chain_record_last || true,	-p singlefs-harness --test second_transaction_step_three_second_instance -- checker_rejects_an_instance_table_row	checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root
步 3：重建分配器时不把已释放的放进 defer 队列	crates/singlefs-core/src/allocator.rs	            if record.is_released {\n                device_map.mark_released(record.slot, span);\n            }	            if record.is_released && false {\n                device_map.mark_released(record.slot, span);\n            }	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
步 4：回退行不带回退位	crates/singlefs-core/src/mount.rs	        applied_transaction_high_water: 0,\n        is_rollback: true,	        applied_transaction_high_water: 0,\n        is_rollback: false,	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
步 4：只写被退回的实例那一行、不写中间实例行	crates/singlefs-core/src/mount.rs	    for row_instance in first_row_instance..instance.0 {	    for row_instance in first_row_instance..first_row_instance + 1 {	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
步 4：回退之后 jsn 接 R_old 那条之后（C340 的 P1，盖掉被抛弃发布的记录槽）	crates/singlefs-core/src/mount.rs	    let next_counter = highest_counter + 1;	    let next_counter = own_record.counter + 1;	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_keeps_the_abandoned_records	rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation
步 4：影子账一个槽都不隔离	crates/singlefs-core/src/mount.rs	    if shadow_ledger == ShadowLedger::On {	    if shadow_ledger == ShadowLedger::Off {	-p singlefs-harness --test second_transaction_step_four_rollback -- without_the_shadow_ledger	without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit
步 4：回退候选集不按实例表判（被抛弃时间线的根也能退到）	crates/singlefs-core/src/mount.rs	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg)	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg && false)	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_onto_an_abandoned	rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused
步 4：前缀第五条不判（回退行的 W 不封顶）	crates/singlefs-core/src/recovery.rs	            if high_water == 0 || record.transaction > high_water {	            if false && (high_water == 0 || record.transaction > high_water) {	-p singlefs-harness --test second_transaction_step_four_rollback -- the_rollback_row_caps	the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water
步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根	crates/singlefs-checker/src/walk.rs	        if index != newest_index && !abandoned && !below_floor {	        if index != newest_index && !below_floor {	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
步 5：回收不看释放代（复用窗口置 0）	crates/singlefs-core/src/allocator.rs	            .filter(|record| record.is_released && record.generation <= floor)	            .filter(|record| record.is_released)	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
步 5：抬 F 的上限不看第 4 新的非空根	crates/singlefs-core/src/mount.rs	    Some(newest_on_every_device.min(fourth_newest))	    Some(newest_on_every_device.max(fourth_newest))	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_above	raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused
步 5：抬 F 的空发布只推一次（F 只落在一块盘上）	crates/singlefs-core/src/mount.rs	        && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(publishes.len()).expect("次数") < 1	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：复用时追加记录而不改写（同盘同槽两条）	crates/singlefs-core/src/allocator.rs	            if self.reclaimed.remove(&key) {	            if self.reclaimed.remove(&key) && false {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：checker 的候选集不按 F 收（F 之下的根照走）	crates/singlefs-checker/src/walk.rs	        if index != newest_index && !abandoned && !below_floor {	        if index != newest_index && !abandoned {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：F_生效 取各盘 F 最大值的最大值而不是最小值	crates/singlefs-core/src/recovery.rs	    highest_per_device\n        .values()\n        .copied()\n        .min()\n        .unwrap_or(CheckpointTxg(0))	    highest_per_device\n        .values()\n        .copied()\n        .max()\n        .unwrap_or(CheckpointTxg(0))	-p singlefs-harness --test second_transaction_step_five_reuse -- one_device_carrying_the_floor_alone	one_device_carrying_the_floor_alone_does_not_take_effect_on_remount
步 3 三方第一轮：链首没锚点时不看 txg、无条件接上水位之上最小的一条	crates/singlefs-core/src/recovery.rs	        } else if record.checkpoint_txg != chain_start_txg_without_anchor {	        } else if false && record.checkpoint_txg != chain_start_txg_without_anchor {	-p singlefs-harness --test second_transaction_step_three_second_instance -- torn_anchor_record	torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg
步 4：普通重开不隔离被抛弃根引用的槽（影子账只在回退那一次算）	crates/singlefs-core/src/mount.rs	        &|_| false,\n        ShadowLedger::On,\n    )?;	        &|_| false,\n        ShadowLedger::Off,\n    )?;	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_keeps_the_abandoned_records	rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation
步 4：回退候选集的 F 用最新根自己带的 F 而不是 F_生效	crates/singlefs-core/src/mount.rs	    if target.checkpoint_txg < effective_floor {	    if target.checkpoint_txg < newest_root.rollback_floor {	-p singlefs-harness --test second_transaction_step_five_reuse -- roots_below_a_floor_carried	roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates
步 5：回收门槛不看环里最旧有效根	crates/singlefs-core/src/mount.rs	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))	    oldest_valid_root.map_or(effective_floor, |_oldest| effective_floor)	-p singlefs-core reclaim_floor_takes	reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root
步 4：影子账把被抛弃根账里已释放的落点也隔离	crates/singlefs-core/src/mount.rs	                if record.is_released || !isolated.insert((record.device.0, record.slot.0)) {	                if !isolated.insert((record.device.0, record.slot.0)) {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：重建分配器时不认第 0 版树表单元（另一个进程里的第一个文件版本漏释放它）	crates/singlefs-core/src/allocator.rs	                record.generation == CheckpointTxg(0)\n                    && record.span_slots == 1\n                    && !record.is_released	                record.generation == CheckpointTxg(0)\n                    && record.span_slots == 1\n                    && !record.is_released\n                    && false	-p singlefs-core rebuild_from_records_remembers	rebuild_from_records_remembers_the_genesis_tree_table_until_it_is_released
```
