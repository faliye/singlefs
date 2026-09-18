# 附录二：里程碑「第二个事务」步 4（管理员回退）与步 5（抬 F 与延迟重用）那批代码改动（`git diff 5f9e449 -- crates/` 原样，加五个新文件全文与 `crates/mutations.tsv` 全文；2026-09-17）

## 一、diff（crates/ 下相对提交 5f9e449 的改动）

```diff
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index 511de3a..0acc66c 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -8,17 +8,38 @@
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
+步 4：回退之后 jsn 接可读链末尾而不是 R_old 那条之后（C340 的 P2）	crates/singlefs-core/src/mount.rs	    let next_counter = own_record.counter + 1;	    let next_counter = records.values().map(|record| record.counter).max().unwrap_or(0) + 1;	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
+步 4：影子账一个槽都不隔离	crates/singlefs-core/src/mount.rs	    if shadow_ledger == ShadowLedger::On {	    if shadow_ledger == ShadowLedger::Off {	-p singlefs-harness --test second_transaction_step_four_rollback -- without_the_shadow_ledger	without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit
+步 4：回退候选集不按实例表判（被抛弃时间线的根也能退到）	crates/singlefs-core/src/mount.rs	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg)	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg && false)	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_onto_an_abandoned	rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused
+步 4：前缀第五条不判（回退行的 W 不封顶）	crates/singlefs-core/src/recovery.rs	            if high_water == 0 || record.transaction > high_water {	            if false && (high_water == 0 || record.transaction > high_water) {	-p singlefs-harness --test second_transaction_step_four_rollback -- the_rollback_row_caps	the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water
+步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根	crates/singlefs-checker/src/walk.rs	        if index != newest_index && !abandoned && !below_floor {	        if index != newest_index && !below_floor {	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
+步 4：记录核对器把被后来记录合法覆盖的记录槽当成记录流有洞	crates/singlefs-harness/src/crash.rs	        if in_place(publish.root) && publish.records.iter().all(|record| record_lost(*record)) {	        if in_place(publish.root) && !publish.records.iter().any(|record| in_place(*record)) {	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
+步 5：回收不看释放代（复用窗口置 0）	crates/singlefs-core/src/allocator.rs	            .filter(|record| record.is_released && record.generation <= floor)	            .filter(|record| record.is_released)	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
+步 5：抬 F 的上限不看第 4 新的非空根	crates/singlefs-core/src/mount.rs	    Some(newest_on_every_device.min(fourth_newest))	    Some(newest_on_every_device.max(fourth_newest))	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_above	raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused
+步 5：抬 F 的空发布只推一次（F 只落在一块盘上）	crates/singlefs-core/src/mount.rs	        && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(publishes.len()).expect("次数") < 1	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
+步 5：复用时追加记录而不改写（同盘同槽两条）	crates/singlefs-core/src/allocator.rs	            if self.reclaimed.remove(&key) {	            if self.reclaimed.remove(&key) && false {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
+步 5：checker 的候选集不按 F 收（F 之下的根照走）	crates/singlefs-checker/src/walk.rs	        if index != newest_index && !abandoned && !below_floor {	        if index != newest_index && !abandoned {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
+步 5：F_生效 取各盘 F 最大值的最大值而不是最小值	crates/singlefs-core/src/recovery.rs	    highest_per_device\n        .values()\n        .copied()\n        .min()\n        .unwrap_or(CheckpointTxg(0))	    highest_per_device\n        .values()\n        .copied()\n        .max()\n        .unwrap_or(CheckpointTxg(0))	-p singlefs-harness --test second_transaction_step_five_reuse -- one_device_carrying_the_floor_alone	one_device_carrying_the_floor_alone_does_not_take_effect_on_remount
+步 3 三方第一轮：链首没锚点时不看 txg、无条件接上水位之上最小的一条	crates/singlefs-core/src/recovery.rs	        } else if record.checkpoint_txg != chain_start_txg_without_anchor {	        } else if false && record.checkpoint_txg != chain_start_txg_without_anchor {	-p singlefs-harness --test second_transaction_step_three_second_instance -- torn_anchor_record	torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index 6ec8977..7e4292e 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -33,10 +33,10 @@ pub enum InvariantVerdict {
 }
 
 /// 第一版 checker 判的不变量，按这个次序报；每次都全部报出来，没评估到的报「不适用」。
-pub const IMPLEMENTED_INVARIANTS: [&str; 23] = [
+pub const IMPLEMENTED_INVARIANTS: [&str; 24] = [
     "I-1.1", "I-1.3", "I-1.4", "I-1.6", "I-1.7", "I-2.1", "I-2.3", "I-2.4", "I-2.5", "I-3.1",
-    "I-5.1", "I-5.2", "I-7.1", "I-7.2", "I-7.6", "I-7.7", "I-7.8", "I-9.1", "I-9.2", "I-9.4",
-    "I-9.7", "I-9.10", "I-9.13",
+    "I-3.8", "I-5.1", "I-5.2", "I-7.1", "I-7.2", "I-7.6", "I-7.7", "I-7.8", "I-9.1", "I-9.2",
+    "I-9.4", "I-9.7", "I-9.10", "I-9.13",
 ];
 
 /// 判定累加器：每条不变量记评估了几次、第一处违例、以及整条不适用的理由。
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index d0b681c..6f539f5 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -60,6 +60,8 @@ struct Walk<'reader> {
     /// 最新根下面记账树里的行：(统计量, 设备) → 值。
     accounting: BTreeMap<(u16, u32), u64>,
     accounting_seen: bool,
+    /// 最新根指着的实例表里的 (实例代号, T)：别的根按它判有效（回退之后被抛弃时间线的根不进 I-3.1 的并集）。
+    instance_table_rows: Vec<(u32, u64)>,
 }
 
 impl Walk<'_> {
@@ -209,7 +211,15 @@ impl Walk<'_> {
                 instance_pointer.locations[0].device,
                 instance_pointer.locations[0].slot,
             )) {
-                self.judge_packed_container(&unit, PACKED_TYPE_INSTANCE_TABLE, "实例表单元");
+                let records =
+                    self.judge_packed_container(&unit, PACKED_TYPE_INSTANCE_TABLE, "实例表单元");
+                if is_newest {
+                    if let Some(records) = records {
+                        let mount_root_instance =
+                            u32::from_le_bytes(record[24..28].try_into().expect("4 字节"));
+                        self.judge_instance_table_rows(&records, mount_root_instance);
+                    }
+                }
             }
         } else {
             self.walk_failures
@@ -397,6 +407,45 @@ impl Walk<'_> {
     }
 
     /// 码 3 容器：I-1.7（记录数 × 记录宽 ≤ 声明长度 ≤ 32768 − 136、记录宽 = 登记的宽）。返回记录，用不了返回 None。
+    /// I-3.8（实例表行唯一且低于挂载根）：`kind` 0 行按实例代号唯一、每行的实例代号 < 挂载根（最新根）的实例；
+    /// 链指针记录（`kind` 1）恒为一片的最后一条（D18（块里携带什么信息） 已定项 11）。「行只在回收条件成立后删」对着一个镜像判不了。
+    fn judge_instance_table_rows(&mut self, records: &[Vec<u8>], mount_root_instance: u32) {
+        let mut instances: Vec<u32> = Vec::new();
+        let mut unique = true;
+        let mut below_mount_root = true;
+        let mut chain_record_last = false;
+        for (index, row) in records.iter().enumerate() {
+            match row.first().copied() {
+                Some(0) => {
+                    let instance = u32::from_le_bytes(row[1..5].try_into().expect("4 字节"));
+                    if instances.contains(&instance) {
+                        unique = false;
+                    }
+                    instances.push(instance);
+                    self.instance_table_rows.push((
+                        instance,
+                        u64::from_le_bytes(row[5..13].try_into().expect("8 字节")),
+                    ));
+                    if instance >= mount_root_instance {
+                        below_mount_root = false;
+                    }
+                    chain_record_last = false;
+                }
+                Some(1) => chain_record_last = index + 1 == records.len(),
+                _ => chain_record_last = false,
+            }
+        }
+        self.judgements.judge(
+            "I-3.8",
+            unique && below_mount_root && chain_record_last,
+            || {
+                format!(
+                    "实例表：行的实例代号 {instances:?}（唯一 = {unique}，都低于挂载根实例 {mount_root_instance} = {below_mount_root}），链指针记录在末尾 = {chain_record_last}"
+                )
+            },
+        );
+    }
+
     fn judge_packed_container(
         &mut self,
         unit: &[u8],
@@ -722,14 +771,29 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
         data_unit_objects: Vec::new(),
         accounting: BTreeMap::new(),
         accounting_seen: false,
+        instance_table_rows: Vec::new(),
     };
     // 先走最新的根：它的走读断没断就是 I-7.2；记账行只取最新根下面的。
     walk.walk_root(&roots[newest_index].2.record_bytes, true);
     let newest_failures = walk.walk_failures.clone();
     let accounting_seen = walk.accounting_seen;
     let accounting = walk.accounting.clone();
+    // 别的根只取按最新根指着的实例表仍然有效的：(i, T) 有效 ⟺ 无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti
+    // （D23（journal 的角色与格式） 已定项 14 回退段的候选集规则）；被抛弃时间线的根引用的单元由影子账隔离、不在当前账里。
+    // 再加一条：txg ≥ 最新根带的回退下界 F（D16（发布语义） 已定项 1 的回退候选集）；F 之下的根引用的单元可以已被回收复用，
+    // 它们不在当前账里、也不再是「近 K 代」——I-2.1 只在候选集里的根上判。
+    let instance_table_rows = walk.instance_table_rows.clone();
+    let newest_rollback_floor = u64::from_le_bytes(
+        roots[newest_index].2.record_bytes[130..138]
+            .try_into()
+            .expect("8 字节"),
+    );
     for (index, (_, _, root)) in roots.iter().enumerate() {
-        if index != newest_index {
+        let abandoned = instance_table_rows.iter().any(|(row_instance, row_txg)| {
+            *row_instance == root.instance && root.checkpoint_txg > *row_txg
+        });
+        let below_floor = root.checkpoint_txg < newest_rollback_floor;
+        if index != newest_index && !abandoned && !below_floor {
             walk.walk_root(&root.record_bytes, false);
         }
     }
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 01e35e5..a275976 100644
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
@@ -171,6 +182,55 @@ impl DeviceFreeMap {
         );
         self.deferred_slots += span;
     }
+    /// 隔离一个只被被抛弃根引用的落点：不进已分配、不进 defer、不动空闲计数，只让分配器绕开它（用户数据落点与开放段都不落在它上面）。
+    /// 已分配的槽不会走到这里（那是 R_old 自己也引用的）；同一个槽隔离两次不重复计数。
+    pub fn isolate(&mut self, slot: SlotNumber, span: u64) {
+        let start = Self::index(slot);
+        let end = start + usize::try_from(span).expect("跨度");
+        assert!(end <= self.allocated.len(), "跨度越过单元区末尾");
+        for index in start..end {
+            assert!(
+                !self.allocated[index],
+                "被隔离的槽在当前账里已分配：影子账只隔离 R_old 不引用的槽"
+            );
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
@@ -241,7 +301,9 @@ impl DeviceFreeMap {
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
@@ -268,6 +330,11 @@ pub struct PoolAllocator {
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
@@ -279,15 +346,49 @@ impl PoolAllocator {
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
@@ -309,6 +410,20 @@ impl PoolAllocator {
 
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
@@ -393,6 +508,47 @@ impl PoolAllocator {
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
@@ -409,7 +565,7 @@ mod tests {
             DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
             DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
         ]);
-        pool.mark_format_time_units(&[
+        pool.mark_format_time_units(
             Placement {
                 slot: SlotNumber(50176),
                 span: 2,
@@ -418,7 +574,7 @@ mod tests {
                 slot: SlotNumber(50178),
                 span: 1,
             },
-        ]);
+        );
         pool
     }
 
diff --git a/crates/singlefs-core/src/lib.rs b/crates/singlefs-core/src/lib.rs
index 58faddf..41846e8 100644
--- a/crates/singlefs-core/src/lib.rs
+++ b/crates/singlefs-core/src/lib.rs
@@ -11,8 +11,10 @@ pub mod allocator;
 pub mod block_device;
 pub mod bytes;
 pub mod checksum;
+pub mod instance_table;
 pub mod journal;
 pub mod make_filesystem;
+pub mod mount;
 pub mod pointer;
 pub mod records;
 pub mod recovery;
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 65d8491..af34212 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -23,19 +23,21 @@ use crate::address::{
 use crate::allocator::AllocationRecord;
 use crate::block_device::BlockDevice;
 use crate::checksum::crc32_castagnoli;
+use crate::instance_table::InstanceTableRecords;
 use crate::journal::JournalRecord;
 use crate::make_filesystem::TREE_TABLE_KEY_WIDTH;
 use crate::pointer::{DataPointer, LocationEntry, NodePointer};
 use crate::records::{
-    mapping_key_for_data, parse_extent_record, parse_inode_internal_entry, parse_mapping_entry,
-    AccountingEntry, InodeRecord, TreeTableEntry, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION,
-    TREE_KIND_DEADLIST, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LIVELIST,
-    TREE_KIND_SPARSE_SIDE_TABLE,
+    mapping_key_for_data, mapping_key_for_node, parse_extent_record, parse_inode_internal_entry,
+    parse_mapping_entry, AccountingEntry, InodeRecord, TreeTableEntry, TREE_KIND_ACCOUNTING,
+    TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_EXTENT, TREE_KIND_INODE,
+    TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE,
 };
 use crate::root_record::RootRecord;
+use crate::root_ring::target_for_publish;
 use crate::root_ring::{slot_offset, RootRingSlot};
 use crate::superblock::{FormatTimeGeometry, Superblock};
-use crate::transaction::FIRST_INODE_NUMBER;
+use crate::transaction::{PublishedUnit, TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER};
 use crate::unit::{
     data_unit_payload, parse_data_unit, parse_index_node, parse_packed_unit,
     unit_filesystem_identifier, IndexNodeHeader, PACKED_TYPE_INODE, PACKED_TYPE_INSTANCE_TABLE,
@@ -156,6 +158,9 @@ pub struct JournalScanReport {
     pub prefix_applied: usize,
     pub verification_passed: usize,
     pub verification_failed: usize,
+    /// 这次恢复施加的记录里最大的事务号（空发布的事务号 0 不进 max，D23（journal 的角色与格式） 已定项 19 ①）；
+    /// 可写挂载给上一个实例写行时 W 取它（D18（块里携带什么信息） 已定项 11 的行记录）。
+    pub maximum_applied_transaction: u64,
 }
 
 #[derive(Clone, Debug, PartialEq, Eq)]
@@ -297,6 +302,331 @@ pub fn choose_root(reader: &dyn PoolReader, superblock: &Superblock) -> Option<R
     best
 }
 
+/// 根环全部自证过的根里最大的 checkpoint_txg：新实例的第一次发布取 max(它, 环里全部自证通过的记录的 checkpoint_txg) + 1
+/// （D23（journal 的角色与格式） 已定项 14 第 3 条）；一条都没有时 None。
+#[must_use]
+pub fn highest_root_txg<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    region_devices: &[DeviceIdentity; 3],
+    geometry: &FormatTimeGeometry,
+    filesystem_identifier: &[u8; 16],
+) -> Option<CheckpointTxg> {
+    let mut highest: Option<CheckpointTxg> = None;
+    visit_valid_roots(
+        reader,
+        region_devices,
+        geometry,
+        filesystem_identifier,
+        |root| {
+            highest = Some(highest.map_or(root.checkpoint_txg, |current| {
+                current.max(root.checkpoint_txg)
+            }));
+        },
+    );
+    highest
+}
+
+/// 根环里全部自证过的根，每个可读根槽一条（回退候选集与影子账从这里取）。
+#[must_use]
+pub fn readable_roots<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    region_devices: &[DeviceIdentity; 3],
+    geometry: &FormatTimeGeometry,
+    filesystem_identifier: &[u8; 16],
+) -> Vec<RootRecord> {
+    let mut roots = Vec::new();
+    visit_valid_roots(
+        reader,
+        region_devices,
+        geometry,
+        filesystem_identifier,
+        |root| roots.push(root),
+    );
+    roots
+}
+
+/// 生效的回退下界 F（D16（发布语义） 已定项 1「生效」那一行：恢复后生效值 = 各幸存盘所带 F 最大值的最小值）；
+/// 一块盘上一条根都没有就不算它，一条根都没有时 0。根落在哪块盘按它的 txg 算区域（与写者同一条公式）。
+#[must_use]
+pub fn effective_rollback_floor<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    region_devices: &[DeviceIdentity; 3],
+    geometry: &FormatTimeGeometry,
+    filesystem_identifier: &[u8; 16],
+) -> CheckpointTxg {
+    let mut highest_per_device: BTreeMap<DeviceIdentity, CheckpointTxg> = BTreeMap::new();
+    for root in readable_roots(reader, region_devices, geometry, filesystem_identifier) {
+        let device = region_devices
+            [usize::try_from(target_for_publish(root.checkpoint_txg).region).expect("区域号")];
+        let highest = highest_per_device
+            .entry(device)
+            .or_insert(root.rollback_floor);
+        *highest = (*highest).max(root.rollback_floor);
+    }
+    highest_per_device
+        .values()
+        .copied()
+        .min()
+        .unwrap_or(CheckpointTxg(0))
+}
+
+/// 一条根引用的分配记录（树表 → 分配记录树根节点）：回退的影子账要读每条被抛弃根的账。第 0 代树表（没有分配记录树）给空。
+///
+/// # Errors
+/// 树表或分配记录树根读不到、解不开。
+pub fn allocation_records_under_root(
+    reader: &dyn PoolReader,
+    root: &RootRecord,
+) -> Result<Vec<AllocationRecord>, RecoveryFailure> {
+    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
+    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
+    let tree_table =
+        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: "树表单元",
+        })?;
+    let mut allocation_pointer = None;
+    for bytes in &tree_table.entries {
+        let entry = TreeTableEntry::parse(bytes).ok_or(RecoveryFailure::UnitMalformed {
+            what: "树表条目",
+        })?;
+        if entry.kind == TREE_KIND_ALLOCATION {
+            allocation_pointer = Some(entry.root);
+        }
+    }
+    let Some(allocation_pointer) = allocation_pointer else {
+        return Ok(Vec::new());
+    };
+    let allocation_bytes =
+        read_unit_via_locations(reader, &allocation_pointer.locations, node_bytes)?;
+    let allocation_node =
+        parse_index_node(&allocation_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: "分配记录树根",
+        })?;
+    Ok(allocation_node
+        .entries
+        .iter()
+        .map(|bytes| AllocationRecord::parse(bytes))
+        .collect())
+}
+
+/// 一条根指着的实例表（行与链指针）；单元读不出或解不开都是 `None`（读不出的由走读另报）。
+#[must_use]
+pub fn instance_table_of_root(
+    reader: &dyn PoolReader,
+    root: &RootRecord,
+) -> Option<InstanceTableRecords> {
+    let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
+    let bytes = read_unit_via_locations(reader, &root.instance_table.locations, data_bytes).ok()?;
+    InstanceTableRecords::parse(&bytes)
+}
+
+/// 所选根的实例在它自己指着的实例表里有回退行时，回退行的 W（D23（journal 的角色与格式） 已定项 14 前缀第五条：
+/// 该实例的记录只施加到 W 为止）；没有回退行、实例表读不出或解不开都是 `None`。
+#[must_use]
+pub fn rollback_high_water_of_root(reader: &dyn PoolReader, root: &RootRecord) -> Option<u64> {
+    instance_table_of_root(reader, root)?
+        .rows
+        .iter()
+        .find(|row| row.instance == root.instance && row.is_rollback)
+        .map(|row| row.applied_transaction_high_water)
+}
+
+/// 从盘上按所选根重建「上一版」：全部角色的单元字节、指针、树表、分配记录、记账行与 inode 记录，交给发布路径当上一版
+/// （照抄没重写的角色、经映射释放被换下的角色都靠它；可写挂载在恢复之后调）。`record` 是所选根自己那条记录（读不出时由调用方顶一条）。
+///
+/// # Errors
+/// 树表为空（所选根下面还没有文件版本）⇒ `InvariantViolated { invariant: "挂载", … }`；单元读不到或解不开 ⇒ 走读同款的错。
+pub fn rebuild_version(
+    reader: &dyn PoolReader,
+    root: &RootRecord,
+    record: JournalRecord,
+    record_bytes: Vec<u8>,
+) -> Result<TransactionOutput, RecoveryFailure> {
+    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
+    let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
+    let instance_table_bytes =
+        read_unit_via_locations(reader, &root.instance_table.locations, data_bytes)?;
+    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
+    let tree_table =
+        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: "树表单元",
+        })?;
+    if tree_table.entries.is_empty() {
+        return Err(RecoveryFailure::InvariantViolated {
+            invariant: "挂载",
+            detail: "所选根下面还没有文件版本（树表为空）",
+        });
+    }
+    let mut tree_table_entries = Vec::with_capacity(tree_table.entries.len());
+    for bytes in &tree_table.entries {
+        tree_table_entries.push(TreeTableEntry::parse(bytes).ok_or(
+            RecoveryFailure::UnitMalformed {
+                what: "树表条目"
+            },
+        )?);
+    }
+    let pointer_of = |kind: u16| {
+        tree_table_entries
+            .iter()
+            .find(|entry| entry.kind == kind)
+            .map(|entry| entry.root)
+            .ok_or(RecoveryFailure::UnitMalformed {
+                what: "树表里没有那棵树",
+            })
+    };
+    let extent_pointer = pointer_of(TREE_KIND_EXTENT)?;
+    let inode_root_pointer = pointer_of(TREE_KIND_INODE)?;
+    let allocation_pointer = pointer_of(TREE_KIND_ALLOCATION)?;
+    let accounting_pointer = pointer_of(TREE_KIND_ACCOUNTING)?;
+    let read_node = |pointer: &NodePointer, what: &'static str| {
+        let bytes = read_unit_via_locations(reader, &pointer.locations, node_bytes)?;
+        let node =
+            parse_index_node(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed { what })?;
+        Ok::<(Vec<u8>, IndexNodeHeader), RecoveryFailure>((bytes, node))
+    };
+    let (extent_bytes, extent_node) = read_node(&extent_pointer, "extent 树根")?;
+    let (_, data_pointer) = parse_extent_record(extent_node.entries.first().ok_or(
+        RecoveryFailure::UnitMalformed {
+            what: "extent 树根没有记录",
+        },
+    )?);
+    let data_unit_bytes = read_unit_via_locations(reader, &data_pointer.locations, data_bytes)?;
+    let (inode_root_bytes, inode_root_node) = read_node(&inode_root_pointer, "inode 树根")?;
+    let (_, _, inode_leaf_pointer) =
+        parse_inode_internal_entry(inode_root_node.entries.first().ok_or(
+            RecoveryFailure::UnitMalformed {
+                what: "inode 树根没有条目",
+            },
+        )?);
+    let inode_leaf_bytes =
+        read_unit_via_locations(reader, &inode_leaf_pointer.locations, data_bytes)?;
+    let inode_leaf =
+        parse_packed_unit(&inode_leaf_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: "inode 叶容器",
+        })?;
+    let inode_record = InodeRecord::parse(inode_leaf.records.first().ok_or(
+        RecoveryFailure::UnitMalformed {
+            what: "inode 叶容器没有记录",
+        },
+    )?)
+    .ok_or(RecoveryFailure::UnitMalformed {
+        what: "inode 记录"
+    })?;
+    let (allocation_bytes, allocation_node) = read_node(&allocation_pointer, "分配记录树根")?;
+    let allocation_records: Vec<AllocationRecord> = allocation_node
+        .entries
+        .iter()
+        .map(|bytes| AllocationRecord::parse(bytes))
+        .collect();
+    let (accounting_bytes, accounting_node) = read_node(&accounting_pointer, "记账树根")?;
+    let accounting_entries: Vec<AccountingEntry> = accounting_node
+        .entries
+        .iter()
+        .map(|bytes| AccountingEntry::parse(bytes))
+        .collect();
+    let (mapping_bytes, mapping_node) = read_node(&root.mapping_root, "映射树根")?;
+    let mapping_keys: Vec<Vec<u8>> = mapping_node
+        .entries
+        .iter()
+        .map(|entry| parse_mapping_entry(entry).0)
+        .collect();
+    let mapped_units = vec![
+        (
+            TransactionUnit::Data,
+            mapping_key_for_data(data_pointer.head, data_pointer.write_order),
+        ),
+        (
+            TransactionUnit::ExtentRoot,
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, extent_pointer),
+        ),
+        (
+            TransactionUnit::InodeLeaf,
+            mapping_key_for_node(UNIT_CLASS_PACKED, inode_leaf_pointer),
+        ),
+        (
+            TransactionUnit::InodeRoot,
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer),
+        ),
+        (
+            TransactionUnit::AllocationTree,
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer),
+        ),
+        (
+            TransactionUnit::AccountingTree,
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, accounting_pointer),
+        ),
+    ];
+    let unit =
+        |identity: TransactionUnit, locations: &[LocationEntry; 2], bytes: Vec<u8>| PublishedUnit {
+            slot: locations[0].slot,
+            identity,
+            bytes,
+        };
+    let units = vec![
+        unit(
+            TransactionUnit::Data,
+            &data_pointer.locations,
+            data_unit_bytes,
+        ),
+        unit(
+            TransactionUnit::ExtentRoot,
+            &extent_pointer.locations,
+            extent_bytes,
+        ),
+        unit(
+            TransactionUnit::InodeLeaf,
+            &inode_leaf_pointer.locations,
+            inode_leaf_bytes,
+        ),
+        unit(
+            TransactionUnit::InodeRoot,
+            &inode_root_pointer.locations,
+            inode_root_bytes,
+        ),
+        unit(
+            TransactionUnit::AllocationTree,
+            &allocation_pointer.locations,
+            allocation_bytes,
+        ),
+        unit(
+            TransactionUnit::AccountingTree,
+            &accounting_pointer.locations,
+            accounting_bytes,
+        ),
+        unit(
+            TransactionUnit::MappingTree,
+            &root.mapping_root.locations,
+            mapping_bytes,
+        ),
+        unit(
+            TransactionUnit::TreeTable,
+            &root.tree_table.locations,
+            tree_table_bytes,
+        ),
+        unit(
+            TransactionUnit::InstanceTable,
+            &root.instance_table.locations,
+            instance_table_bytes,
+        ),
+    ];
+    Ok(TransactionOutput {
+        root: *root,
+        record,
+        record_bytes,
+        units,
+        rewritten: Vec::new(),
+        data_pointer,
+        mapping_keys,
+        allocation_records,
+        accounting_entries,
+        tree_table_entries,
+        inode_record,
+        mapped_units,
+        released: Vec::new(),
+        key_order_mismatches: 0,
+    })
+}
+
 /// 根环全部自证过的根里最大的实例代号（取号的 max 里「根环里全部根记录的实例代号」那一半）；一条都没有时 None。
 #[must_use]
 pub fn highest_root_instance<Reader: PoolReader + ?Sized>(
@@ -379,6 +709,7 @@ pub fn replay_journal(
     ring_bytes: u64,
     records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
     verify_named_units: bool,
+    rollback_high_water: Option<u64>,
 ) -> (JournalScanReport, RootRecord) {
     let mut report = JournalScanReport {
         valid_records: records.len(),
@@ -386,23 +717,48 @@ pub fn replay_journal(
         prefix_applied: 0,
         verification_passed: 0,
         verification_failed: 0,
+        maximum_applied_transaction: 0,
     };
     let water = (root.instance, root.checkpoint_txg);
     let mut rebuilt = *root;
+    // 前缀规则不跨实例边界（D23（journal 的角色与格式） 已定项 14 第 1 条）：链从所选根覆盖的最后一条记录之后接，
+    // 下一条的实例代号与所选根不同即停——所以只有所选根自己那个实例的记录是候选；所选根是 mkfs 的第 0 代根时一条都不施加。
     let mut above: Vec<&JournalRecord> = records
         .values()
-        .filter(|record| (record.instance, record.checkpoint_txg) > water)
+        .filter(|record| {
+            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water
+        })
         .collect();
     above.sort_by_key(|record| (record.instance, record.counter));
     report.above_water = above.len();
     let in_flight_limit =
         usize::try_from(journal_in_flight_record_limit(ring_bytes)).expect("在飞上限");
-    let mut expected_next: Option<(InstanceGeneration, u64)> = None;
+    // 链首接在所选根自己那条记录（同实例、checkpoint_txg 相等）之后；那条记录读不出（两份都撕了）时不知道它的 jsn，
+    // 链首只能是 checkpoint_txg = 根的 txg + 1 的那条（第一版一次发布一条记录、txg 每次加一）：水位之上最小的那条若 txg 更大，
+    // 中间就少了一条，断号即止照样成立（里程碑「第二个事务」步 3 三方第一轮攻方腿打中：无锚点时无条件接上会跳过撕掉的一条）。
+    let root_own_record_counter = records
+        .values()
+        .find(|record| {
+            record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg
+        })
+        .map(|record| record.counter);
+    let mut expected_next: Option<(InstanceGeneration, u64)> =
+        root_own_record_counter.map(|counter| (root.instance, counter + 1));
+    let chain_start_txg_without_anchor = CheckpointTxg(root.checkpoint_txg.0 + 1);
     for record in above.into_iter().take(in_flight_limit) {
         if let Some(expected_key) = expected_next {
             if (record.instance, record.counter) != expected_key {
                 break;
             }
+        } else if record.checkpoint_txg != chain_start_txg_without_anchor {
+            break;
+        }
+        // 前缀第五条：所选根的实例有回退行时只施加到回退行的 W 为止——W = 0 就是「之后的一个都不算」，
+        // 空发布（事务号 0）也不许把根推过 T_old。
+        if let Some(high_water) = rollback_high_water {
+            if high_water == 0 || record.transaction > high_water {
+                break;
+            }
         }
         expected_next = Some((record.instance, record.counter + 1));
         if !record.is_commit {
@@ -429,6 +785,8 @@ pub fn replay_journal(
         }
         report.verification_passed += 1;
         report.prefix_applied += 1;
+        report.maximum_applied_transaction =
+            report.maximum_applied_transaction.max(record.transaction);
         rebuilt = RootRecord {
             filesystem_identifier: rebuilt.filesystem_identifier,
             instance: record.instance,
@@ -897,6 +1255,7 @@ pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport
                 superblock.geometry.journal_ring_bytes,
                 &records,
                 policy == JournalPolicy::Consult,
+                rollback_high_water_of_root(reader, &root),
             )
         }
         JournalPolicy::Ignore => (JournalScanReport::default(), root),
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 7eef470..6dd2ef9 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -12,11 +12,12 @@ use std::collections::BTreeMap;
 use singlefs_format::{
     ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, ALLOCATION_RECORD_BYTES,
     ALLOCATION_RECORD_KEY_BYTES, EXTENT_KEY_BYTES, EXTENT_LEAF_RECORD_BYTES, FIRST_TRANSACTION_TXG,
-    INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES, MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES, SLOT_BYTES,
-    SUPERBLOCK_SLOTS_PER_DEVICE, TREE_IDENTIFIER_ACCOUNTING, TREE_IDENTIFIER_ALLOCATION_RECORDS,
-    TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST, TREE_IDENTIFIER_EXTENT,
-    TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LIVELIST, TREE_IDENTIFIER_SPARSE_SIDE_TABLE,
-    TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, TREE_TABLE_ENTRY_BYTES, WARM_UP_EMPTY_PUBLISHES,
+    INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES, INSTANCE_ROW_BYTES, MAPPING_ENTRY_BYTES,
+    MAPPING_KEY_BYTES, SLOT_BYTES, SUPERBLOCK_SLOTS_PER_DEVICE, TREE_IDENTIFIER_ACCOUNTING,
+    TREE_IDENTIFIER_ALLOCATION_RECORDS, TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST,
+    TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LIVELIST,
+    TREE_IDENTIFIER_SPARSE_SIDE_TABLE, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
+    TREE_TABLE_ENTRY_BYTES, WARM_UP_EMPTY_PUBLISHES,
 };
 
 use crate::address::{
@@ -33,7 +34,7 @@ use crate::pointer::{BirthSequence, DataPointer, LocationEntry, NodePointer, Poi
 use crate::records::{
     build_extent_record, build_inode_internal_entry, build_mapping_entry, data_key_tail,
     mapping_key_for_data, mapping_key_for_node, mapping_key_sort_key, node_key_tail,
-    parse_mapping_entry, AccountingEntry, InodeRecord, TreeTableEntry,
+    parse_inode_internal_entry, parse_mapping_entry, AccountingEntry, InodeRecord, TreeTableEntry,
     ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF, STATISTIC_ALLOCATED_BYTES,
     STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_DEFER_QUEUE_BYTES,
     STATISTIC_EMPTY_CLUSTER_SEGMENTS, STATISTIC_FRAGMENTATION_RUNS, STATISTIC_FREE_BYTES,
@@ -48,8 +49,8 @@ use crate::superblock::Superblock;
 use crate::unit::{
     build_data_unit, build_index_node, build_packed_unit, data_unit_payload_capacity,
     index_node_entry_capacity, parse_index_node, unit_filesystem_identifier, DataUnitIdentity,
-    PackedIdentity, WriteOrder, PACKED_TYPE_INODE, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE,
-    UNIT_CLASS_PACKED,
+    PackedIdentity, WriteOrder, PACKED_TYPE_INODE, PACKED_TYPE_INSTANCE_TABLE, UNIT_CLASS_DATA,
+    UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
 };
 
 /// 第一个文件的 inode 号（里程碑步 3 预想）。
@@ -392,6 +393,9 @@ pub enum TransactionUnit {
     AccountingTree,
     MappingTree,
     TreeTable,
+    /// 实例表单元（码 3 打包记录类型 4）：mkfs 种下第一片，之后每次可写挂载写行时重写（D18（块里携带什么信息） 已定项 11）；
+    /// 不在第一个事务的八个角色里，第二个事务起才进发布路径。
+    InstanceTable,
 }
 
 impl TransactionUnit {
@@ -428,6 +432,7 @@ impl TransactionUnit {
             TransactionUnit::AccountingTree => "t6",
             TransactionUnit::MappingTree => "t7",
             TransactionUnit::TreeTable => "t8",
+            TransactionUnit::InstanceTable => "ti",
         }
     }
 
@@ -435,7 +440,7 @@ impl TransactionUnit {
     pub const fn unit_class(self) -> u8 {
         match self {
             TransactionUnit::Data => UNIT_CLASS_DATA,
-            TransactionUnit::InodeLeaf => UNIT_CLASS_PACKED,
+            TransactionUnit::InodeLeaf | TransactionUnit::InstanceTable => UNIT_CLASS_PACKED,
             TransactionUnit::ExtentRoot
             | TransactionUnit::InodeRoot
             | TransactionUnit::AllocationTree
@@ -458,7 +463,9 @@ impl TransactionUnit {
             TransactionUnit::AllocationTree => TreeIdentifier(TREE_IDENTIFIER_ALLOCATION_RECORDS),
             TransactionUnit::AccountingTree => TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING),
             TransactionUnit::MappingTree => TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
-            TransactionUnit::TreeTable => TreeIdentifier(TREE_IDENTIFIER_NONE),
+            TransactionUnit::TreeTable | TransactionUnit::InstanceTable => {
+                TreeIdentifier(TREE_IDENTIFIER_NONE)
+            }
         }
     }
 
@@ -467,7 +474,7 @@ impl TransactionUnit {
     pub const fn placement(self) -> PlacementRule {
         match self {
             TransactionUnit::Data => PlacementRule::UserData,
-            TransactionUnit::InodeLeaf => {
+            TransactionUnit::InodeLeaf | TransactionUnit::InstanceTable => {
                 PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned)
             }
             TransactionUnit::ExtentRoot
@@ -500,7 +507,10 @@ pub struct TransactionOutput {
     pub root: RootRecord,
     pub record: JournalRecord,
     pub record_bytes: Vec<u8>,
+    /// 这一版全部角色的单元：这次重写的是新装的，没重写的从上一版照抄（八个文件 / 固定点角色按 bump 次序，实例表单元在末尾、mkfs 之后第一次重写之前不在）。
     pub units: Vec<PublishedUnit>,
+    /// 这次发布真正写出的角色，按写出的次序（点名项与录制流里的单元写只有这些）。
+    pub rewritten: Vec<TransactionUnit>,
     pub data_pointer: DataPointer,
     pub mapping_keys: Vec<Vec<u8>>,
     pub allocation_records: Vec<AllocationRecord>,
@@ -522,10 +532,29 @@ impl TransactionOutput {
         self.units
             .iter()
             .find(|unit| unit.identity == identity)
-            .expect("八个单元每种一个")
+            .expect("八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在")
+    }
+
+    /// 树表里某棵树的根指针（照抄没重写的角色时用）。
+    #[must_use]
+    pub fn tree_root_pointer(&self, tree: u64) -> NodePointer {
+        self.tree_table_entries
+            .iter()
+            .find(|entry| entry.tree == TreeIdentifier(tree))
+            .expect("树表里每棵登记的树一条")
+            .root
+    }
+
+    /// 树表条目的诞生 txg（七条同一个数：树建起来那次发布）。
+    #[must_use]
+    pub fn tree_birth_txg(&self) -> CheckpointTxg {
+        self.tree_table_entries
+            .first()
+            .expect("树表第 1 版起恒有七条")
+            .birth_txg
     }
 
-    /// 这次发布写出的八个落点，按写者自己记的槽号（位置提示那一路）。释放不走它、走 `placements_to_release_via_mapping`；
+    /// 这一版全部角色的落点，按写者自己记的槽号（位置提示那一路）。释放不走它、走 `placements_to_release_via_mapping`；
     /// 留着给验收拿两条路互相对。
     #[must_use]
     pub fn placements(&self) -> Vec<Placement> {
@@ -539,6 +568,29 @@ impl TransactionOutput {
     }
 }
 
+/// 第一个文件版本换下 mkfs 那片树表单元时要释放的落点：树表这一角色在这次重写、mkfs 的树表单元还登记着、还没释放过。
+fn format_time_tree_table_to_release(
+    allocator: &PoolAllocator,
+    rewritten: &[TransactionUnit],
+) -> Vec<Placement> {
+    if !rewritten.contains(&TransactionUnit::TreeTable) {
+        return Vec::new();
+    }
+    let Some(tree_table) = allocator.format_time_tree_table() else {
+        return Vec::new();
+    };
+    let still_allocated = allocator
+        .devices
+        .first()
+        .and_then(|device_map| allocator.record_for(device_map.device, tree_table.slot))
+        .is_some_and(|record| !record.is_released);
+    if still_allocated {
+        vec![tree_table]
+    } else {
+        Vec::new()
+    }
+}
+
 /// 释放判定路径（D19（块指针的结构与宽度预算） 已定项 5 第 1 条：释放一律经映射，不经提示）：上一版八个单元的落点从上一版的
 /// 映射节点按 key 查出来，查不到就报「不在映射」、一个落点都不释放；映射树与树表豁免映射，各从根记录里指着它们的那条指针取
 /// （D19（块指针的结构与宽度预算） 已定项 11：映射树的根住根记录）。两盘同槽（D2（RAID 条带策略） 已定项 10）。
@@ -551,10 +603,11 @@ impl TransactionOutput {
 pub fn placements_to_release_via_mapping(
     previous: &TransactionOutput,
     allocator: &PoolAllocator,
+    roles: &[TransactionUnit],
 ) -> Result<Vec<Placement>, PublishError> {
     let mapping_node_bytes = &previous.unit(TransactionUnit::MappingTree).bytes;
     let mut placements = Vec::new();
-    for identity in TransactionUnit::IN_BUMP_ORDER {
+    for identity in roles.iter().copied() {
         let locations = match identity {
             TransactionUnit::Data
             | TransactionUnit::ExtentRoot
@@ -572,6 +625,7 @@ pub fn placements_to_release_via_mapping(
             }
             TransactionUnit::MappingTree => previous.root.mapping_root.locations,
             TransactionUnit::TreeTable => previous.root.tree_table.locations,
+            TransactionUnit::InstanceTable => previous.root.instance_table.locations,
         };
         assert_eq!(
             locations[0].slot, locations[1].slot,
@@ -713,24 +767,71 @@ pub struct FirstFile<'content> {
     pub write_time_seconds: u64,
 }
 
-/// 一次「文件的一个版本」的发布要的全部参数：第一个事务与覆盖写只差这些数（都是发布参数，不取系统时钟）。
+/// 这次发布要写的文件版本：四个文件角色（数据、extent 根、inode 叶、inode 根）全部重写。没有它的发布（写行、暖机）文件角色照抄上一版。
 #[derive(Clone, Copy, Debug)]
-pub struct FilePublish<'content> {
-    pub txg: CheckpointTxg,
-    /// jsn 计数器（记录落在环里的槽位）。
-    pub counter: u64,
-    /// 事务号，按实例计数从 1 起（D23（journal 的角色与格式） 已定项 7）。
-    pub transaction: u64,
+pub struct FileVersionPlan<'content> {
     pub content: &'content [u8],
     pub write_time_seconds: u64,
     /// 对象出生代 = 创建那次发布的 checkpoint_txg，覆盖写不改（D8（核心索引结构） 已定项 6；I-9.10（对象出生代与 inode 记录相符））。
     pub inode_object_birth: CheckpointTxg,
     /// 改动计数（D8（核心索引结构） 已定项 6 偏移 88）。
     pub change_count: u64,
-    /// 树表条目的诞生 txg：树建起来那次发布，覆盖写不改。
+}
+
+/// 实例表单元这次发布怎么处理：根记录照抄上一版的指针，或者重写成给定的记录（行记录在前、链指针记录最末，D18（块里携带什么信息） 已定项 11）。
+#[derive(Clone, Debug)]
+pub enum InstanceTablePlan {
+    Carry(NodePointer),
+    Rewrite(Vec<Vec<u8>>),
+}
+
+/// 一次发布的全部参数：哪些角色重写、身份字段取什么。第一个事务、覆盖写、写行、暖机都是它的一种取值，走同一条发布路径
+/// （`.claude/rules/fs-design.md`「一个事务层，所有结构共用」）。
+#[derive(Clone, Debug)]
+pub struct PublishPlan<'content> {
+    pub txg: CheckpointTxg,
+    /// jsn 计数器（记录落在环里的槽位），全池接着走、换实例不归零（D23（journal 的角色与格式） 已定项 14 第 3 条）。
+    pub counter: u64,
+    /// 事务号，按实例计数从 1 起，空发布写 0（D23（journal 的角色与格式） 已定项 7 / 已定项 19 ①）。
+    pub transaction: u64,
+    pub instance: InstanceGeneration,
+    /// 反向链：上一条记录头的 CRC；本实例的第一条恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
+    pub back_chain: u32,
+    pub file: Option<FileVersionPlan<'content>>,
+    pub instance_table: InstanceTablePlan,
+    /// 树表条目的诞生 txg：树建起来那次发布，之后每一版重写都不改。
     pub tree_birth_txg: CheckpointTxg,
-    /// 根记录照旧持有的实例表单元指针。
-    pub instance_table: NodePointer,
+    pub tree_identifier_watermark: u64,
+    pub rollback_floor: CheckpointTxg,
+}
+
+impl PublishPlan<'_> {
+    /// 这次发布重写的角色，按 bump 次序：用户数据先取（自己的政策）；提交内生块按树 ID 升序、树内先叶后根、映射树倒数第二、树表最末
+    /// （D3（空间分配） 已定项 10 ⑤）。实例表单元归树 0，排在全部提交内生块之前（mkfs 也是先实例表后树表；里程碑步 3 的决策点）。
+    #[must_use]
+    pub fn rewritten_roles(&self) -> Vec<TransactionUnit> {
+        let mut roles = Vec::new();
+        if self.file.is_some() {
+            roles.push(TransactionUnit::Data);
+        }
+        if matches!(self.instance_table, InstanceTablePlan::Rewrite(_)) {
+            roles.push(TransactionUnit::InstanceTable);
+        }
+        if self.file.is_some() {
+            roles.extend([
+                TransactionUnit::ExtentRoot,
+                TransactionUnit::InodeLeaf,
+                TransactionUnit::InodeRoot,
+            ]);
+        }
+        roles.extend([
+            TransactionUnit::AllocationTree,
+            TransactionUnit::AccountingTree,
+            TransactionUnit::MappingTree,
+            TransactionUnit::TreeTable,
+        ]);
+        roles
+    }
 }
 
 /// 第一个事务（字节表七 t1..t8 + 六 + 七）：分配八个落点、装八个单元、按 D16（发布语义） 已定项 7 的持久顺序落盘。
@@ -743,30 +844,35 @@ pub fn publish_first_file<Device: BlockDevice>(
     previous_record_bytes: &[u8],
 ) -> Result<TransactionOutput, PublishError> {
     let txg = CheckpointTxg(FIRST_TRANSACTION_TXG);
-    publish_file_version(
+    publish_version(
         pool,
         allocator,
-        FilePublish {
+        PublishPlan {
             txg,
             counter: FIRST_TRANSACTION_TXG,
             transaction: FIRST_TRANSACTION_NUMBER,
-            content: file.content,
-            write_time_seconds: file.write_time_seconds,
-            inode_object_birth: txg,
-            change_count: 1,
+            instance,
+            back_chain: back_chain_of(previous_record_bytes),
+            file: Some(FileVersionPlan {
+                content: file.content,
+                write_time_seconds: file.write_time_seconds,
+                inode_object_birth: txg,
+                change_count: 1,
+            }),
+            instance_table: InstanceTablePlan::Carry(genesis.instance_table),
             tree_birth_txg: txg,
-            instance_table: genesis.instance_table,
+            tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
+            rollback_floor: CheckpointTxg(0),
         },
-        instance,
-        previous_record_bytes,
-        &[],
+        None,
     )
 }
 
-/// 覆盖写（里程碑「第二个事务」步 1 / 步 2）：同一个实例里紧接着上一次发布，把同一个文件的内容整个换掉——
-/// 新数据单元 COW 到新落点、extent 叶记录的指针换成它、inode 记录更新（改动计数 = 这次发布的 checkpoint_txg），
-/// 六个提交内生块与树表单元各 COW 出新版本；上一版的八个单元在同一次发布里释放（释放代 = 这次的 txg）。
-/// txg、jsn、事务号各比上一次加一。
+/// 覆盖写（里程碑「第二个事务」步 1 / 步 2）：同一个实例里接在上一次发布之后再发布一版同一个文件——txg、jsn、事务号各加一，
+/// 对象出生代与容器身份不改，改动计数取这次的 txg；上一版的八个落点经映射释放（进 defer 队列）。
+///
+/// # Errors
+/// 释放判定路径查不到上一版的某个单元（`ReleaseNotInMapping` 一族）、空间不够、装不下、块设备报错，都原样交回。
 pub fn publish_overwrite<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
@@ -774,60 +880,67 @@ pub fn publish_overwrite<Device: BlockDevice>(
     file: FirstFile<'_>,
     instance: InstanceGeneration,
 ) -> Result<TransactionOutput, PublishError> {
-    assert_eq!(
-        previous.root.instance, instance,
-        "覆盖写接在同一个实例的上一次发布之后"
-    );
     let txg = CheckpointTxg(previous.root.checkpoint_txg.0 + 1);
-    let tree_birth_txg = previous
-        .tree_table_entries
-        .first()
-        .expect("树表第 1 版有七条")
-        .birth_txg;
-    let release = placements_to_release_via_mapping(previous, allocator)?;
-    publish_file_version(
+    publish_version(
         pool,
         allocator,
-        FilePublish {
+        PublishPlan {
             txg,
             counter: previous.record.counter + 1,
             transaction: previous.record.transaction + 1,
-            content: file.content,
-            write_time_seconds: file.write_time_seconds,
-            inode_object_birth: previous.inode_record.object_birth,
-            change_count: txg.0,
-            tree_birth_txg,
-            instance_table: previous.root.instance_table,
+            instance,
+            back_chain: back_chain_of(&previous.record_bytes),
+            file: Some(FileVersionPlan {
+                content: file.content,
+                write_time_seconds: file.write_time_seconds,
+                inode_object_birth: previous.inode_record.object_birth,
+                change_count: txg.0,
+            }),
+            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
+            tree_birth_txg: previous.tree_birth_txg(),
+            tree_identifier_watermark: previous.root.tree_identifier_watermark,
+            rollback_floor: previous.root.rollback_floor,
         },
-        instance,
-        &previous.record_bytes,
-        &release,
+        Some(previous),
     )
 }
 
-/// 发布一个文件版本：先做准入（内容装得进一个数据单元、分配记录树装得下这次的记录），再释放上一版的落点、分配八个落点、
-/// 装八个单元、按 D16（发布语义） 已定项 7 的持久顺序落盘。准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立，
-/// 释放与分配都不算数（第二轮攻方腿：`NoSpaceFor` 在释放之后、分配到一半才返回，留下半新的池，拿同一个上一版重试撞断言）。
-fn publish_file_version<Device: BlockDevice>(
+/// 发布一版：先做准入（内容装得进一个数据单元、分配记录树装得下这次的记录），再释放上一版被换下的角色的落点、分配、装单元、
+/// 按 D16（发布语义） 已定项 7 的持久顺序落盘。准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立，
+/// 释放与分配都不算数（第二轮攻方腿：`NoSpaceFor` 在释放之后、分配到一半返回，留下半新的池，拿同一个上一版重试撞断言）。
+///
+/// # Errors
+/// `ContentExceedsDataUnit`、`AllocationRecordsExceedOneNode`、释放判定路径的四种错、`NoSpaceFor`、块设备错。
+pub fn publish_version<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
-    publish: FilePublish<'_>,
-    instance: InstanceGeneration,
-    previous_record_bytes: &[u8],
-    release: &[Placement],
+    plan: PublishPlan<'_>,
+    previous: Option<&TransactionOutput>,
 ) -> Result<TransactionOutput, PublishError> {
+    let rewritten = plan.rewritten_roles();
+    // 释放判定路径先于准入：它只查不改（D19（块指针的结构与宽度预算） 已定项 5 第 1 条），查不到就整次发布不做。
+    let release = match previous {
+        Some(previous_version) => {
+            placements_to_release_via_mapping(previous_version, allocator, &rewritten)?
+        }
+        // 第一个文件版本没有上一版的内存态：它重写树表时换下的是 mkfs 那片第 0 版树表单元，照样进 defer 队列
+        // （D3（空间分配） 已定项 7；不释放它，txg 0 的根离开候选集之后这一槽就永远占着，I-3.1（已分配统计对得上） 在抬 F 之后红）。
+        None => format_time_tree_table_to_release(allocator, &rewritten),
+    };
     // 用户给的内容装不进一个数据单元是调用方能恢复的失败，不是不变量被破坏：报错，不走到 build_data_unit 的断言。
-    let data_unit_capacity = data_unit_payload_capacity();
-    if publish.content.len() > data_unit_capacity {
-        return Err(PublishError::ContentExceedsDataUnit {
-            bytes: publish.content.len(),
-            capacity: data_unit_capacity,
-        });
+    if let Some(file) = &plan.file {
+        let data_unit_capacity = data_unit_payload_capacity();
+        if file.content.len() > data_unit_capacity {
+            return Err(PublishError::ContentExceedsDataUnit {
+                bytes: file.content.len(),
+                capacity: data_unit_capacity,
+            });
+        }
     }
     // 分配记录树第一版只有一个节点：这次发布之后装不下就在动分配器之前报错，不许走到 build_index_node 的断言
-    // （三方代码第一轮攻方腿：第 50 次覆盖写 panic）。释放只改写记录、不加；这次的八个落点每盘各加一条。
+    // （三方代码第一轮攻方腿：第 50 次覆盖写 panic）。释放只改写记录、不加；这次重写的每个角色每盘各加一条。
     let records_after_this_publish =
-        allocator.records().len() + TransactionUnit::IN_BUMP_ORDER.len() * allocator.devices.len();
+        allocator.records().len() + rewritten.len() * allocator.devices.len();
     let allocation_node_capacity = index_node_entry_capacity(
         usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10"),
         usize::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
@@ -840,37 +953,36 @@ fn publish_file_version<Device: BlockDevice>(
     }
     // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。
     let allocator_before_this_publish = allocator.clone();
-    let outcome = publish_admitted_file_version(
-        pool,
-        allocator,
-        publish,
-        instance,
-        previous_record_bytes,
-        release,
-    );
+    let outcome = publish_admitted(pool, allocator, &plan, previous, &rewritten, &release);
     if outcome.is_err() {
         *allocator = allocator_before_this_publish;
     }
     outcome
 }
 
+/// 上一版里某个角色的单元字节（照抄进这一版的 `units`）。
+fn carried_unit(previous: &TransactionOutput, identity: TransactionUnit) -> PublishedUnit {
+    previous.unit(identity).clone()
+}
+
 /// 准入之后的那一段：释放、分配、装单元、落盘。失败时分配器由调用方退回，这里不管。
 #[allow(
     clippy::too_many_lines,
-    reason = "一次发布就是一件能单独验证的事：八个单元的装法与一条持久顺序，拆开只会把顺序藏进几个函数"
+    reason = "一次发布就是一件能单独验证的事：九个角色的装法与一条持久顺序，拆开只会把顺序藏进几个函数"
 )]
-fn publish_admitted_file_version<Device: BlockDevice>(
+fn publish_admitted<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
-    publish: FilePublish<'_>,
-    instance: InstanceGeneration,
-    previous_record_bytes: &[u8],
+    plan: &PublishPlan<'_>,
+    previous: Option<&TransactionOutput>,
+    rewritten: &[TransactionUnit],
     release: &[Placement],
 ) -> Result<TransactionOutput, PublishError> {
-    let txg = publish.txg;
+    let txg = plan.txg;
+    let instance = plan.instance;
     let write_order = WriteOrder {
         instance,
-        transaction: publish.transaction,
+        transaction: plan.transaction,
     };
     let filesystem_identifier = &pool.parameters.filesystem_identifier;
     let mut sequences = BirthSequenceAllocator::default();
@@ -880,121 +992,227 @@ fn publish_admitted_file_version<Device: BlockDevice>(
         allocator.release(*placement, txg);
     }
 
-    // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以八个落点在装任何单元之前全部取定。
+    // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。
     let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
-    for identity in TransactionUnit::IN_BUMP_ORDER {
+    for identity in rewritten {
         let placement = match identity.placement() {
             PlacementRule::UserData => allocator.allocate_user_data(txg),
             PlacementRule::CommitGenerated(footprint) => {
                 allocator.allocate_commit_generated(footprint, txg)
             }
         };
-        let placement = placement.ok_or(PublishError::NoSpaceFor { unit: identity })?;
-        slots.insert(identity, placement.slot);
+        let placement = placement.ok_or(PublishError::NoSpaceFor { unit: *identity })?;
+        slots.insert(*identity, placement.slot);
     }
     let slot_of = |identity: TransactionUnit| slots[&identity];
 
-    // t1 数据单元（字节表二）。
-    let data_identity = DataUnitIdentity {
-        tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
-        object: FIRST_INODE_NUMBER,
-        object_birth: publish.inode_object_birth,
-        anchor_offset: 0,
+    // 文件角色：有新版本就装四个单元，没有就照抄上一版的指针与字节。
+    let carried_file = match &plan.file {
+        Some(_) => None,
+        None => Some(previous.expect("没有文件版本的发布要接在上一版之后：文件角色从它照抄")),
     };
-    let data_unit = build_data_unit(
-        data_identity,
-        txg,
-        filesystem_identifier,
-        write_order,
-        publish.content,
-    );
-    let data_pointer = DataPointer {
-        head: PointerHead {
-            birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
-            birth_txg: txg,
-        },
-        locations: pool.location_entries(slot_of(TransactionUnit::Data), &data_unit),
-        write_order,
-    };
-    let extent_record = build_extent_record(FIRST_INODE_NUMBER, 0, data_pointer);
-    let extent_key: [u8; 24] = extent_record[..usize::try_from(EXTENT_KEY_BYTES).expect("24")]
-        .try_into()
-        .expect("24");
-    let key_order_mismatches = count_key_order_mismatches(&[extent_key]);
-
-    // t3 inode 树叶容器（字节表四）：出生序号先于 t4 的根发号（树内先叶后根）。
-    // 容器身份（容器号、容器出生代）在树建起来那次定下，之后每一版重写都不变（D8（核心索引结构） 已定项 6：一片叶活很多代、反复重写）。
-    let inode_leaf_identity = PackedIdentity {
-        birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
-        record_type: PACKED_TYPE_INODE,
-        container: FIRST_INODE_NUMBER,
-        container_birth: publish.tree_birth_txg,
-    };
-    let inode_leaf_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
-    let inode_record = InodeRecord {
-        inode: FIRST_INODE_NUMBER,
-        object_birth: publish.inode_object_birth,
-        size: u64::try_from(publish.content.len()).expect("文件长度"),
-        change_count: publish.change_count,
-        write_time_seconds: publish.write_time_seconds,
-    };
-    let inode_leaf_unit = build_packed_unit(
-        inode_leaf_identity,
-        u16::try_from(INODE_RECORD_BYTES).expect("140"),
-        &[inode_record.to_bytes()],
-        txg,
-        filesystem_identifier,
-        write_order,
+    let (
+        data_unit,
+        data_pointer,
+        extent_unit,
+        key_order_mismatches,
+        inode_leaf_unit,
+        inode_leaf_pointer,
         inode_leaf_sequence,
-    );
-    let inode_leaf_pointer = NodePointer {
-        head: PointerHead {
-            birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
-            birth_txg: txg,
-        },
-        locations: pool.location_entries(slot_of(TransactionUnit::InodeLeaf), &inode_leaf_unit),
-        instance,
-        birth_sequence: inode_leaf_sequence,
-    };
-
-    // t2 extent 树根兼叶（字节表四·二）。
-    let extent_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_EXTENT), txg, instance);
-    let extent_unit = build_index_node(
-        TreeIdentifier(TREE_IDENTIFIER_EXTENT),
-        0,
-        usize::try_from(EXTENT_KEY_BYTES).expect("24"),
-        &extent_key,
-        &extent_key,
-        txg,
-        filesystem_identifier,
-        instance,
+        inode_record,
+        inode_root_unit,
         extent_sequence,
-        u16::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
-        std::slice::from_ref(&extent_record),
-    );
-
-    // t4 inode 树根：层级 1，key 区间 [ino, ino]，一条 120 字节条目。
-    let inode_root_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
-    let inode_key = FIRST_INODE_NUMBER.to_le_bytes();
-    let inode_root_unit = build_index_node(
-        TreeIdentifier(TREE_IDENTIFIER_INODE),
-        1,
-        8,
-        &inode_key,
-        &inode_key,
-        txg,
-        filesystem_identifier,
-        instance,
         inode_root_sequence,
-        u16::try_from(INODE_INTERNAL_ENTRY).expect("120"),
-        &[build_inode_internal_entry(
-            FIRST_INODE_NUMBER,
-            inode_leaf_identity,
-            inode_leaf_pointer,
-        )],
-    );
+    ) = match (&plan.file, carried_file) {
+        (Some(file), _) => {
+            // t1 数据单元（字节表二）。
+            let data_identity = DataUnitIdentity {
+                tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+                object: FIRST_INODE_NUMBER,
+                object_birth: file.inode_object_birth,
+                anchor_offset: 0,
+            };
+            let data_unit = build_data_unit(
+                data_identity,
+                txg,
+                filesystem_identifier,
+                write_order,
+                file.content,
+            );
+            let data_pointer = DataPointer {
+                head: PointerHead {
+                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+                    birth_txg: txg,
+                },
+                locations: pool.location_entries(slot_of(TransactionUnit::Data), &data_unit),
+                write_order,
+            };
+            let extent_record = build_extent_record(FIRST_INODE_NUMBER, 0, data_pointer);
+            let extent_key: [u8; 24] = extent_record
+                [..usize::try_from(EXTENT_KEY_BYTES).expect("24")]
+                .try_into()
+                .expect("24");
+            let key_order_mismatches = count_key_order_mismatches(&[extent_key]);
+
+            // t3 inode 树叶容器（字节表四）：出生序号先于 t4 的根发号（树内先叶后根）。
+            // 容器身份（容器号、容器出生代）在树建起来那次定下，之后每一版重写都不变（D8（核心索引结构） 已定项 6：一片叶活很多代、反复重写）。
+            let inode_leaf_identity = PackedIdentity {
+                birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
+                record_type: PACKED_TYPE_INODE,
+                container: FIRST_INODE_NUMBER,
+                container_birth: plan.tree_birth_txg,
+            };
+            let inode_leaf_sequence =
+                sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
+            let inode_record = InodeRecord {
+                inode: FIRST_INODE_NUMBER,
+                object_birth: file.inode_object_birth,
+                size: u64::try_from(file.content.len()).expect("文件长度"),
+                change_count: file.change_count,
+                write_time_seconds: file.write_time_seconds,
+            };
+            let inode_leaf_unit = build_packed_unit(
+                inode_leaf_identity,
+                u16::try_from(INODE_RECORD_BYTES).expect("140"),
+                &[inode_record.to_bytes()],
+                txg,
+                filesystem_identifier,
+                write_order,
+                inode_leaf_sequence,
+            );
+            let inode_leaf_pointer = NodePointer {
+                head: PointerHead {
+                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
+                    birth_txg: txg,
+                },
+                locations: pool
+                    .location_entries(slot_of(TransactionUnit::InodeLeaf), &inode_leaf_unit),
+                instance,
+                birth_sequence: inode_leaf_sequence,
+            };
+
+            // t2 extent 树根兼叶（字节表四·二）。
+            let extent_sequence =
+                sequences.next(TreeIdentifier(TREE_IDENTIFIER_EXTENT), txg, instance);
+            let extent_unit = build_index_node(
+                TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+                0,
+                usize::try_from(EXTENT_KEY_BYTES).expect("24"),
+                &extent_key,
+                &extent_key,
+                txg,
+                filesystem_identifier,
+                instance,
+                extent_sequence,
+                u16::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
+                std::slice::from_ref(&extent_record),
+            );
 
-    // t5 分配记录树（字节表五）：mkfs 的两个单元分配代 0、其余是这次发布的 txg，每盘各一条，按 (设备, 槽号) 升序。
+            // t4 inode 树根：层级 1，key 区间 [ino, ino]，一条 120 字节条目。
+            let inode_root_sequence =
+                sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
+            let inode_key = FIRST_INODE_NUMBER.to_le_bytes();
+            let inode_root_unit = build_index_node(
+                TreeIdentifier(TREE_IDENTIFIER_INODE),
+                1,
+                8,
+                &inode_key,
+                &inode_key,
+                txg,
+                filesystem_identifier,
+                instance,
+                inode_root_sequence,
+                u16::try_from(INODE_INTERNAL_ENTRY).expect("120"),
+                &[build_inode_internal_entry(
+                    FIRST_INODE_NUMBER,
+                    inode_leaf_identity,
+                    inode_leaf_pointer,
+                )],
+            );
+            (
+                data_unit,
+                data_pointer,
+                extent_unit,
+                key_order_mismatches,
+                inode_leaf_unit,
+                inode_leaf_pointer,
+                inode_leaf_sequence,
+                inode_record,
+                inode_root_unit,
+                extent_sequence,
+                inode_root_sequence,
+            )
+        }
+        (None, Some(carried)) => {
+            // 照抄：数据指针、inode 记录、四个单元的字节都取上一版；inode 叶的指针从上一版 inode 根的那条条目解出来。
+            let inode_root_bytes = &carried.unit(TransactionUnit::InodeRoot).bytes;
+            let inode_root_node = parse_index_node(inode_root_bytes)
+                .expect("上一版的 inode 根是这次或上次发布装出来的，解得开");
+            let (_, _inode_leaf_identity, inode_leaf_pointer) = parse_inode_internal_entry(
+                inode_root_node
+                    .entries
+                    .first()
+                    .expect("第一版 inode 树根恒有一条条目"),
+            );
+            let extent_unit = carried.unit(TransactionUnit::ExtentRoot).bytes.clone();
+            let extent_pointer_previous = carried.tree_root_pointer(TREE_IDENTIFIER_EXTENT);
+            let inode_root_pointer_previous = carried.tree_root_pointer(TREE_IDENTIFIER_INODE);
+            (
+                carried.unit(TransactionUnit::Data).bytes.clone(),
+                carried.data_pointer,
+                extent_unit,
+                0,
+                carried.unit(TransactionUnit::InodeLeaf).bytes.clone(),
+                inode_leaf_pointer,
+                inode_leaf_pointer.birth_sequence,
+                carried.inode_record,
+                carried.unit(TransactionUnit::InodeRoot).bytes.clone(),
+                extent_pointer_previous.birth_sequence,
+                inode_root_pointer_previous.birth_sequence,
+            )
+        }
+        (None, None) => {
+            unreachable!("上面按 plan.file 分过：没有文件版本时 carried_file 一定是 Some")
+        }
+    };
+    // 实例表单元（码 3 打包记录类型 4）：重写时容器身份照 mkfs（容器 0、出生代 0），归树 0、在树表之前发号（mkfs 也是先实例表后树表）。
+    let instance_table_rewrite = match &plan.instance_table {
+        InstanceTablePlan::Rewrite(records) => {
+            let sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
+            let unit = build_packed_unit(
+                PackedIdentity {
+                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
+                    record_type: PACKED_TYPE_INSTANCE_TABLE,
+                    container: 0,
+                    container_birth: CheckpointTxg(0),
+                },
+                u16::try_from(INSTANCE_ROW_BYTES).expect("88"),
+                records,
+                txg,
+                filesystem_identifier,
+                write_order,
+                sequence,
+            );
+            let pointer = NodePointer {
+                head: PointerHead {
+                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
+                    birth_txg: txg,
+                },
+                locations: pool.location_entries(slot_of(TransactionUnit::InstanceTable), &unit),
+                instance,
+                birth_sequence: sequence,
+            };
+            Some((unit, pointer, sequence))
+        }
+        InstanceTablePlan::Carry(_) => None,
+    };
+    let instance_table_pointer = match (&plan.instance_table, &instance_table_rewrite) {
+        (InstanceTablePlan::Carry(pointer), _) => *pointer,
+        (InstanceTablePlan::Rewrite(_), Some((_, pointer, _))) => *pointer,
+        (InstanceTablePlan::Rewrite(_), None) => unreachable!("重写时上面一定装了单元"),
+    };
+
+    // t5 分配记录树（字节表五）：mkfs 的两个单元分配代 0、其余是各自分配那次发布的 txg，每盘各一条，按 (设备, 槽号) 升序。
     let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();
     allocation_records.sort_by_key(AllocationRecord::sort_key);
     let allocation_sequence = sequences.next(
@@ -1105,18 +1323,27 @@ fn publish_admitted_file_version<Device: BlockDevice>(
             instance,
             birth_sequence: sequence,
         };
-    let extent_pointer = node_pointer(
-        TREE_IDENTIFIER_EXTENT,
-        TransactionUnit::ExtentRoot,
-        &extent_unit,
-        extent_sequence,
-    );
-    let inode_root_pointer = node_pointer(
-        TREE_IDENTIFIER_INODE,
-        TransactionUnit::InodeRoot,
-        &inode_root_unit,
-        inode_root_sequence,
-    );
+    // 文件角色的指针：重写的按这次的落点算，照抄的取上一版。
+    let (extent_pointer, inode_root_pointer) = match carried_file {
+        None => (
+            node_pointer(
+                TREE_IDENTIFIER_EXTENT,
+                TransactionUnit::ExtentRoot,
+                &extent_unit,
+                extent_sequence,
+            ),
+            node_pointer(
+                TREE_IDENTIFIER_INODE,
+                TransactionUnit::InodeRoot,
+                &inode_root_unit,
+                inode_root_sequence,
+            ),
+        ),
+        Some(carried) => (
+            carried.tree_root_pointer(TREE_IDENTIFIER_EXTENT),
+            carried.tree_root_pointer(TREE_IDENTIFIER_INODE),
+        ),
+    };
     let allocation_pointer = node_pointer(
         TREE_IDENTIFIER_ALLOCATION_RECORDS,
         TransactionUnit::AllocationTree,
@@ -1131,6 +1358,7 @@ fn publish_admitted_file_version<Device: BlockDevice>(
     );
 
     // t7 中央映射树（字节表三·二）：码 1 一条 + 码 2 / 码 3 五条；映射树自己、树表、实例表豁免（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
+    // 照抄的文件角色 key 照旧、位置照旧；重写的按这次的指针算。
     let mapped_units_with_locations: Vec<(TransactionUnit, Vec<u8>, [LocationEntry; 2])> = vec![
         (
             TransactionUnit::Data,
@@ -1200,14 +1428,14 @@ fn publish_admitted_file_version<Device: BlockDevice>(
         mapping_sequence,
     );
 
-    // t8 树表单元第 1 版：七条按树 ID 升序（D8（核心索引结构） 已定项 8）；映射树的根住根记录、不进树表（D19（块指针的结构与宽度预算） 已定项 11）；
+    // t8 树表单元：七条按树 ID 升序（D8（核心索引结构） 已定项 8）；映射树的根住根记录、不进树表（D19（块指针的结构与宽度预算） 已定项 11）；
     // 头 ID（D5（快照 / 空间记账机制） 已定项 9）：inode 树写自己、extent 树写它服务的头，其余 0。
     let table_entry =
         |kind: u16, tree: u64, root: NodePointer, head_identifier: u64| TreeTableEntry {
             kind,
             tree: TreeIdentifier(tree),
             root,
-            birth_txg: publish.tree_birth_txg,
+            birth_txg: plan.tree_birth_txg,
             head_identifier,
         };
     let tree_table_entries = vec![
@@ -1278,110 +1506,108 @@ fn publish_admitted_file_version<Device: BlockDevice>(
         tree_table_sequence,
     );
 
-    let units = vec![
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::Data),
-            identity: TransactionUnit::Data,
-            bytes: data_unit,
-        },
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::ExtentRoot),
-            identity: TransactionUnit::ExtentRoot,
-            bytes: extent_unit,
-        },
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::InodeLeaf),
-            identity: TransactionUnit::InodeLeaf,
-            bytes: inode_leaf_unit,
-        },
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::InodeRoot),
-            identity: TransactionUnit::InodeRoot,
-            bytes: inode_root_unit,
-        },
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::AllocationTree),
-            identity: TransactionUnit::AllocationTree,
-            bytes: allocation_unit,
-        },
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::AccountingTree),
-            identity: TransactionUnit::AccountingTree,
-            bytes: accounting_unit,
-        },
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::MappingTree),
-            identity: TransactionUnit::MappingTree,
-            bytes: mapping_unit,
-        },
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::TreeTable),
-            identity: TransactionUnit::TreeTable,
-            bytes: tree_table_unit,
-        },
-    ];
+    // 这一版全部角色的单元：重写的是这次装的，照抄的从上一版拷（八个文件 / 固定点角色按 bump 次序，实例表单元在末尾）。
+    let rewritten_unit = |identity: TransactionUnit, bytes: Vec<u8>| PublishedUnit {
+        slot: slot_of(identity),
+        identity,
+        bytes,
+    };
+    let mut units: Vec<PublishedUnit> = Vec::new();
+    for identity in TransactionUnit::IN_BUMP_ORDER {
+        let unit = match identity {
+            TransactionUnit::Data => match carried_file {
+                None => rewritten_unit(identity, data_unit.clone()),
+                Some(carried) => carried_unit(carried, identity),
+            },
+            TransactionUnit::ExtentRoot => match carried_file {
+                None => rewritten_unit(identity, extent_unit.clone()),
+                Some(carried) => carried_unit(carried, identity),
+            },
+            TransactionUnit::InodeLeaf => match carried_file {
+                None => rewritten_unit(identity, inode_leaf_unit.clone()),
+                Some(carried) => carried_unit(carried, identity),
+            },
+            TransactionUnit::InodeRoot => match carried_file {
+                None => rewritten_unit(identity, inode_root_unit.clone()),
+                Some(carried) => carried_unit(carried, identity),
+            },
+            TransactionUnit::AllocationTree => rewritten_unit(identity, allocation_unit.clone()),
+            TransactionUnit::AccountingTree => rewritten_unit(identity, accounting_unit.clone()),
+            TransactionUnit::MappingTree => rewritten_unit(identity, mapping_unit.clone()),
+            TransactionUnit::TreeTable => rewritten_unit(identity, tree_table_unit.clone()),
+            TransactionUnit::InstanceTable => {
+                unreachable!("IN_BUMP_ORDER 只有八个文件 / 固定点角色")
+            }
+        };
+        units.push(unit);
+    }
+    match (&instance_table_rewrite, previous) {
+        (Some((unit, _, _)), _) => {
+            units.push(rewritten_unit(TransactionUnit::InstanceTable, unit.clone()))
+        }
+        (None, Some(previous_version)) => {
+            if let Some(carried) = previous_version
+                .units
+                .iter()
+                .find(|unit| unit.identity == TransactionUnit::InstanceTable)
+            {
+                units.push(carried.clone());
+            }
+        }
+        (None, None) => {}
+    }
 
-    // 点名项（D23（journal 的角色与格式） 已定项 17）：t1..t8 各一项，key 尾段与映射 key 共用。
-    let named_tails: [(TransactionUnit, [u8; 10]); 8] = [
-        (
-            TransactionUnit::Data,
-            data_key_tail(data_pointer.write_order),
-        ),
-        (
-            TransactionUnit::ExtentRoot,
-            node_key_tail(instance, extent_sequence),
-        ),
-        (
-            TransactionUnit::InodeLeaf,
-            node_key_tail(instance, inode_leaf_sequence),
-        ),
-        (
-            TransactionUnit::InodeRoot,
-            node_key_tail(instance, inode_root_sequence),
-        ),
-        (
-            TransactionUnit::AllocationTree,
-            node_key_tail(instance, allocation_sequence),
-        ),
-        (
-            TransactionUnit::AccountingTree,
-            node_key_tail(instance, accounting_sequence),
-        ),
-        (
-            TransactionUnit::MappingTree,
-            node_key_tail(instance, mapping_sequence),
-        ),
-        (
-            TransactionUnit::TreeTable,
-            node_key_tail(instance, tree_table_sequence),
-        ),
-    ];
-    let named: Vec<NamedUnit> = units
+    // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的每个角色一项，key 尾段与映射 key 共用；照抄的不点名。
+    let key_tail_of = |identity: TransactionUnit| -> [u8; 10] {
+        match identity {
+            TransactionUnit::Data => data_key_tail(data_pointer.write_order),
+            TransactionUnit::ExtentRoot => node_key_tail(instance, extent_sequence),
+            TransactionUnit::InodeLeaf => node_key_tail(instance, inode_leaf_sequence),
+            TransactionUnit::InodeRoot => node_key_tail(instance, inode_root_sequence),
+            TransactionUnit::AllocationTree => node_key_tail(instance, allocation_sequence),
+            TransactionUnit::AccountingTree => node_key_tail(instance, accounting_sequence),
+            TransactionUnit::MappingTree => node_key_tail(instance, mapping_sequence),
+            TransactionUnit::TreeTable => node_key_tail(instance, tree_table_sequence),
+            TransactionUnit::InstanceTable => node_key_tail(
+                instance,
+                instance_table_rewrite
+                    .as_ref()
+                    .map(|(_, _, sequence)| *sequence)
+                    .expect("点名实例表单元的发布一定重写了它"),
+            ),
+        }
+    };
+    let written_units: Vec<&PublishedUnit> = rewritten
         .iter()
-        .zip(named_tails)
-        .map(|(unit, (identity, key_tail))| {
-            assert_eq!(unit.identity, identity, "点名项与单元一一对应");
-            NamedUnit {
-                locations: pool.location_entries(unit.slot, &unit.bytes),
-                unit_class: identity.unit_class(),
-                birth_tree: identity.tree(),
-                birth_txg: txg,
-                key_tail,
-            }
+        .map(|identity| {
+            units
+                .iter()
+                .find(|unit| unit.identity == *identity)
+                .expect("重写的角色都装了单元")
+        })
+        .collect();
+    let named: Vec<NamedUnit> = written_units
+        .iter()
+        .map(|unit| NamedUnit {
+            locations: pool.location_entries(unit.slot, &unit.bytes),
+            unit_class: unit.identity.unit_class(),
+            birth_tree: unit.identity.tree(),
+            birth_txg: txg,
+            key_tail: key_tail_of(unit.identity),
         })
         .collect();
     let record = JournalRecord {
         instance,
-        counter: publish.counter,
+        counter: plan.counter,
         checkpoint_txg: txg,
-        transaction: publish.transaction,
+        transaction: plan.transaction,
         is_commit: true,
-        back_chain: back_chain_of(previous_record_bytes),
+        back_chain: plan.back_chain,
         filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
         new_tree_table: tree_table_pointer,
         new_mapping_root: mapping_pointer,
-        new_tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
-        new_rollback_floor: CheckpointTxg(0),
+        new_tree_identifier_watermark: plan.tree_identifier_watermark,
+        new_rollback_floor: plan.rollback_floor,
         named,
     };
     let record_bytes = record.to_bytes();
@@ -1390,15 +1616,15 @@ fn publish_admitted_file_version<Device: BlockDevice>(
         instance,
         checkpoint_txg: txg,
         tree_table: tree_table_pointer,
-        tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
-        rollback_floor: CheckpointTxg(0),
-        instance_table: publish.instance_table,
+        tree_identifier_watermark: plan.tree_identifier_watermark,
+        rollback_floor: plan.rollback_floor,
+        instance_table: instance_table_pointer,
         mapping_root: mapping_pointer,
     };
     let root_slot = root.to_slot(pool.root_slot_bytes());
 
-    // 持久顺序（D16（发布语义） 已定项 7）：单元与节点 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 超级块槽轮换。
-    for unit in &units {
+    // 持久顺序（D16（发布语义） 已定项 7）：这次重写的单元 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 超级块槽轮换。
+    for unit in &written_units {
         pool.perform(CommitStep::WriteUnitToEveryDevice {
             slot: unit.slot,
             unit: &unit.bytes,
@@ -1406,7 +1632,7 @@ fn publish_admitted_file_version<Device: BlockDevice>(
     }
     pool.perform(CommitStep::Barrier)?;
     pool.perform(CommitStep::WriteJournalRecordToEveryDevice {
-        counter: publish.counter,
+        counter: plan.counter,
         record: &record_bytes,
     })?;
     pool.perform(CommitStep::Barrier)?;
@@ -1415,7 +1641,7 @@ fn publish_admitted_file_version<Device: BlockDevice>(
         root_slot: &root_slot,
     })?;
     pool.perform(CommitStep::RotateSuperblockSlots {
-        journal_tail: publish.counter,
+        journal_tail: plan.counter,
         journal_instance: instance,
     })?;
 
@@ -1424,6 +1650,7 @@ fn publish_admitted_file_version<Device: BlockDevice>(
         record,
         record_bytes,
         units,
+        rewritten: rewritten.to_vec(),
         data_pointer,
         mapping_keys: mapping_entries.into_iter().map(|(key, _)| key).collect(),
         allocation_records,
diff --git a/crates/singlefs-harness/src/bin/first_transaction_on_device.rs b/crates/singlefs-harness/src/bin/first_transaction_on_device.rs
index 7879bb4..fc3c414 100644
--- a/crates/singlefs-harness/src/bin/first_transaction_on_device.rs
+++ b/crates/singlefs-harness/src/bin/first_transaction_on_device.rs
@@ -1,34 +1,87 @@
 //! 虚机档（QEMU/KVM）：在两块真 virtio 盘上跑第一个事务的整条写路，冷重开再恢复读回文件。
 //!
-//!   first_transaction_on_device /dev/vda /dev/vdb <direct | page-cache | skip-first-transaction-barrier>
+//!   first_transaction_on_device /dev/vda /dev/vdb <direct | page-cache | skip-first-transaction-barrier | second-transaction>
 //!
 //! 由 `research/scripts/vm-bench.sh` 送进虚机（`VM_DISKS=2`，设备路径排在参数前面）。结果行以 `E7RESULT` 打头、
 //! 末行报条数（vm-bench.sh 的完整性闸）。这个二进制只判它自己判得了的（恢复读回文件、段序列）；
 //! 「盘上实际收到的是不是程序以为的」由宿主上的 `first_transaction_device_log_check` 拿 QEMU 的设备侧日志判。
 //! `skip-first-transaction-barrier`：第一个事务在盘 0 上漏掉「单元 → journal 记录」那道屏障，而录制器照样记下它——
 //! 程序以为发了，盘上没收到，宿主那一侧必须判红（C6（块层语义假设写错） 的「故意去掉一次屏障」）。
+//! `second-transaction`：第一个事务之后同一个进程里再覆盖写一次（里程碑「第二个事务」步 1 的发布 B），分别报两次发布的挂钟与
+//! 程序交给设备的写请求数、字节、屏障、FUA——E152（按里程碑对比六家文件系统的文件性能） 里程碑二那一轮的 singlefs 臂用它量「第二次写」的稳态代价；
+//! 冷重开读回的是第二版。
 
 use std::path::Path;
 
 use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
+use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator};
 use singlefs_core::block_device::{
     probe_queue_number_under, probe_queue_text_under, BlockDevice, BlockDeviceError,
     DirectInputOutputBlockDevice, PageCachePolicy, PhysicalBlockSizeInBytes,
     PhysicalBlockSizeSource, WriteDurability, SYSFS_CLASS_BLOCK,
 };
 use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
-use singlefs_harness::scenario::{e142_parameters, first_file_content, run_first_transaction};
+use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
+use singlefs_harness::scenario::{
+    e142_parameters, first_file_content, run_first_transaction, FIXED_WRITE_TIME_SECONDS,
+};
 use singlefs_harness::segments::{
     closed_form_state_count, segment_kinds_text, segment_sizes_text, split_into_segments,
     FixedGeometry,
 };
 use singlefs_harness::{RecordedOperation, RecordingBlockDevice, SharedStream};
+use std::time::Instant;
 
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum RunMode {
     Direct,
     PageCache,
     SkipFirstTransactionBarrier,
+    SecondTransaction,
+}
+
+/// 第二版的内容：与第一版不同长度、不同字节，读回时分得开。
+fn second_file_content() -> Vec<u8> {
+    (0..4100usize)
+        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
+        .collect()
+}
+
+/// 一块盘上程序交给设备的调用计数快照，两次快照相减就是一次发布的代价。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct DeviceCallCounts {
+    write_calls: u64,
+    written_bytes: u64,
+    force_unit_access_writes: u64,
+    barrier_calls: u64,
+}
+
+impl DeviceCallCounts {
+    fn of<Inner: BlockDevice>(device: &FaultInjectingDevice<Inner>) -> Self {
+        Self {
+            write_calls: device.write_calls,
+            written_bytes: device.written_bytes,
+            force_unit_access_writes: device.force_unit_access_writes,
+            barrier_calls: device.barrier_calls,
+        }
+    }
+
+    fn since(self, earlier: Self) -> Self {
+        Self {
+            write_calls: self.write_calls - earlier.write_calls,
+            written_bytes: self.written_bytes - earlier.written_bytes,
+            force_unit_access_writes: self.force_unit_access_writes
+                - earlier.force_unit_access_writes,
+            barrier_calls: self.barrier_calls - earlier.barrier_calls,
+        }
+    }
+
+    fn describe(self, index: usize) -> String {
+        format!(
+            "device_{index}_writes={} device_{index}_written_bytes={} device_{index}_force_unit_access_writes={} device_{index}_barriers={}",
+            self.write_calls, self.written_bytes, self.force_unit_access_writes, self.barrier_calls
+        )
+    }
 }
 
 /// 包在真设备外面、录制器里面：数程序真正交给设备的调用，按需吞掉一道屏障。
@@ -161,13 +214,14 @@ fn main() {
         "direct" => RunMode::Direct,
         "page-cache" => RunMode::PageCache,
         "skip-first-transaction-barrier" => RunMode::SkipFirstTransactionBarrier,
+        "second-transaction" => RunMode::SecondTransaction,
         other => {
             eprintln!("不认识的模式 {other}");
             std::process::exit(2)
         }
     };
     let policy = match mode {
-        RunMode::Direct | RunMode::SkipFirstTransactionBarrier => {
+        RunMode::Direct | RunMode::SkipFirstTransactionBarrier | RunMode::SecondTransaction => {
             PageCachePolicy::BypassWithDirectInputOutput
         }
         RunMode::PageCache => PageCachePolicy::GoThroughPageCache,
@@ -297,6 +351,66 @@ fn main() {
         run.output.record.back_chain
     ));
 
+    // 第二个事务（发布 B）：同一个进程里再覆盖写一次，分配器从第一个事务的分配记录重建（与可写挂载同一条路）；
+    // 两次快照之差就是这一次发布交给设备的调用数，挂钟单独计。
+    let expected_content = if mode == RunMode::SecondTransaction {
+        let before: Vec<DeviceCallCounts> = devices
+            .iter()
+            .map(|(_, device)| DeviceCallCounts::of(device.inner()))
+            .collect();
+        let operations_before = stream.operations().len();
+        let device_maps: Vec<DeviceFreeMap> = devices
+            .iter()
+            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
+            .collect();
+        let mut allocator =
+            PoolAllocator::rebuild_from_records(device_maps, run.output.allocation_records.clone());
+        let started = Instant::now();
+        let second = {
+            let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
+            publish_overwrite(
+                &mut pool,
+                &mut allocator,
+                &run.output,
+                FirstFile {
+                    content: &second_file_content(),
+                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
+                },
+                run.output.root.instance,
+            )
+        }
+        .unwrap_or_else(|failure| {
+            eprintln!("第二个事务失败：{failure:?}");
+            std::process::exit(5)
+        });
+        let nanoseconds = started.elapsed().as_nanos();
+        let after_operations = stream.operations();
+        let second_segments =
+            split_into_segments(&after_operations[operations_before..], &geometry);
+        let per_device: Vec<String> = devices
+            .iter()
+            .enumerate()
+            .map(|(index, (_, device))| {
+                DeviceCallCounts::of(device.inner())
+                    .since(before[index])
+                    .describe(index)
+            })
+            .collect();
+        emitter.emit(&format!(
+            "name=second_transaction root_txg={} transaction={} released={} nanoseconds={nanoseconds} operations={} segments={} closed_form={} {}",
+            second.root.checkpoint_txg.0,
+            second.record.transaction,
+            second.released.len(),
+            after_operations.len() - operations_before,
+            segment_sizes_text(&second_segments),
+            closed_form_state_count(&second_segments),
+            per_device.join(" ")
+        ));
+        second_file_content()
+    } else {
+        first_file_content()
+    };
+
     // 冷重开：丢掉写的那一套句柄，按路径重新打开，恢复只通过块设备读盘。
     drop(devices);
     let reopened: Vec<(DeviceIdentity, DirectInputOutputBlockDevice)> = device_paths
@@ -314,7 +428,7 @@ fn main() {
         RecoveryOutcome::FileRead { root, content } => (
             "file_read",
             format!("{}:{}", root.0 .0, root.1 .0),
-            *content == first_file_content(),
+            *content == expected_content,
         ),
         RecoveryOutcome::NoFile { root } => {
             ("no_file", format!("{}:{}", root.0 .0, root.1 .0), false)
@@ -340,3 +454,45 @@ fn main() {
         std::process::exit(1);
     }
 }
+
+#[cfg(test)]
+mod tests {
+    use super::{second_file_content, DeviceCallCounts};
+
+    #[test]
+    fn device_call_counts_subtract_field_by_field_and_describe_with_the_device_index() {
+        let earlier = DeviceCallCounts {
+            write_calls: 11,
+            written_bytes: 172_544,
+            force_unit_access_writes: 1,
+            barrier_calls: 2,
+        };
+        let later = DeviceCallCounts {
+            write_calls: 22,
+            written_bytes: 345_088,
+            force_unit_access_writes: 2,
+            barrier_calls: 4,
+        };
+        let delta = later.since(earlier);
+        assert_eq!(
+            delta,
+            DeviceCallCounts {
+                write_calls: 11,
+                written_bytes: 172_544,
+                force_unit_access_writes: 1,
+                barrier_calls: 2,
+            }
+        );
+        assert_eq!(
+            delta.describe(1),
+            "device_1_writes=11 device_1_written_bytes=172544 device_1_force_unit_access_writes=1 device_1_barriers=2"
+        );
+    }
+
+    #[test]
+    fn second_file_content_differs_from_the_first_in_length_and_bytes() {
+        let second = second_file_content();
+        assert_eq!(second.len(), 4100);
+        assert_ne!(second, singlefs_harness::scenario::first_file_content());
+    }
+}
diff --git a/crates/singlefs-harness/src/crash.rs b/crates/singlefs-harness/src/crash.rs
index f53ee85..08d2a00 100644
--- a/crates/singlefs-harness/src/crash.rs
+++ b/crates/singlefs-harness/src/crash.rs
@@ -476,9 +476,23 @@ pub fn check_records(
         PoolReader::read(image, write.device, write.offset, write.bytes.len())
             .is_some_and(|bytes| bytes == write.bytes)
     };
+    // 一份记录「丢了」= 它不在盘上，而且没有更晚的写落在同一个位置上；回退之后新实例从 R_old 那条记录之后接着写 jsn
+    // （C340（回退之后记录链从哪条之后接没有定义） 取 P1），被抛弃发布的记录槽被后来的记录合法覆盖，不是记录流有洞。
+    let record_lost = |index: usize| {
+        let write = &image.writes[index];
+        !in_place(index)
+            && !image.writes[index + 1..]
+                .iter()
+                .enumerate()
+                .any(|(later_offset, later)| {
+                    later.device == write.device
+                        && later.offset == write.offset
+                        && in_place(index + 1 + later_offset)
+                })
+    };
     let mut check = RecordCheck::default();
     for publish in publishes_in(image.writes) {
-        if in_place(publish.root) && !publish.records.iter().any(|record| in_place(*record)) {
+        if in_place(publish.root) && publish.records.iter().all(|record| record_lost(*record)) {
             check.root_without_record = true;
         }
         if effective_root.is_some_and(|(_, txg)| txg.0 >= publish.checkpoint_txg) {
diff --git a/crates/singlefs-harness/src/scenario.rs b/crates/singlefs-harness/src/scenario.rs
index 4816375..3049002 100644
--- a/crates/singlefs-harness/src/scenario.rs
+++ b/crates/singlefs-harness/src/scenario.rs
@@ -78,7 +78,7 @@ pub fn run_first_transaction<
             .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
             .collect(),
     );
-    allocator.mark_format_time_units(&[
+    allocator.mark_format_time_units(
         Placement {
             slot: INSTANCE_TABLE_SLOT,
             span: 2,
@@ -87,7 +87,7 @@ pub fn run_first_transaction<
             slot: TREE_TABLE_GENESIS_SLOT,
             span: 1,
         },
-    ]);
+    );
     let content = first_file_content();
     let (instance, warm_up_output) = {
         let mut pool = PoolWriter::new(parameters, &mut *devices);
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 489d0e2..8f077bc 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -27,6 +27,7 @@ const UNITS: [(u64, usize); 10] = [
     (50247, 16384),
     (50248, 16384),
 ];
+const INSTANCE_TABLE: u64 = 50176;
 const DATA_UNIT: u64 = 50180;
 const EXTENT_ROOT: u64 = 50240;
 const INODE_LEAF: u64 = 50242;
@@ -473,6 +474,27 @@ fn known_bad_images(clean: &MemoryPool) -> Vec<(&'static str, Mutation)> {
                 mutate_unit(image, INODE_LEAF, |bytes| set_u64(bytes, 136 + 8, 2), true)
             }),
         ),
+        // 实例表里多一行 (1, 3, 0)：行的实例代号等于挂载根的实例 1，不「低于」；链指针记录仍在末尾、行仍唯一，红的只有那一半。
+        (
+            "I-3.8",
+            Box::new(|image: &mut MemoryPool| {
+                mutate_unit(
+                    image,
+                    INSTANCE_TABLE,
+                    |bytes| {
+                        let chain_record: Vec<u8> = bytes[136..224].to_vec();
+                        let mut row = vec![0u8; 88];
+                        row[1..5].copy_from_slice(&1u32.to_le_bytes());
+                        row[5..13].copy_from_slice(&3u64.to_le_bytes());
+                        bytes[136..224].copy_from_slice(&row);
+                        bytes[224..312].copy_from_slice(&chain_record);
+                        set_u16(bytes, 69, 2);
+                        set_u16(bytes, 8, 176);
+                    },
+                    true,
+                );
+            }),
+        ),
         // 类型 2 容器的记录数与声明长度都改成 0。
         (
             "I-9.13",
diff --git a/crates/singlefs-harness/tests/common/mod.rs b/crates/singlefs-harness/tests/common/mod.rs
index c64073e..0b307bd 100644
--- a/crates/singlefs-harness/tests/common/mod.rs
+++ b/crates/singlefs-harness/tests/common/mod.rs
@@ -107,6 +107,28 @@ impl BuiltPool {
         pool.apply(&self.retained_operations()[..self.mkfs_operation_count]);
         pool
     }
+    /// 重开镜像并继续录进同一条流：进程重开之后的可写挂载与发布都要进层 0 的整条流（里程碑「第二个事务」步 0 / 步 3）。
+    pub fn reopen_recorded(&mut self) -> Vec<(DeviceIdentity, Recorded)> {
+        drop(self.devices.take());
+        self.paths
+            .iter()
+            .enumerate()
+            .map(|(index, path)| {
+                let file = FileBackedBlockDevice::open_or_create(
+                    path,
+                    IMAGE_BYTES,
+                    PhysicalBlockSizeInBytes(512),
+                )
+                .expect("重开镜像");
+                let identity = DeviceIdentity(u32::try_from(index).expect("设备号"));
+                (
+                    identity,
+                    RecordingBlockDevice::with_shared_stream(identity, file, self.stream.clone()),
+                )
+            })
+            .collect()
+    }
+
     /// 冷启动：丢掉进程内的设备句柄，按路径重新打开镜像。
     pub fn reopen_cold(&mut self) -> Vec<(DeviceIdentity, FileBackedBlockDevice)> {
         drop(self.devices.take());
@@ -158,7 +180,7 @@ pub fn build_pool(tag: &str) -> BuiltPool {
         DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
         DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
     ]);
-    allocator.mark_format_time_units(&[
+    allocator.mark_format_time_units(
         Placement {
             slot: INSTANCE_TABLE_SLOT,
             span: 2,
@@ -167,7 +189,7 @@ pub fn build_pool(tag: &str) -> BuiltPool {
             slot: TREE_TABLE_GENESIS_SLOT,
             span: 1,
         },
-    ]);
+    );
     let content = file_content();
     let (warm_up, output) = {
         let mut pool = PoolWriter::new(&parameters, &mut devices);
diff --git a/crates/singlefs-harness/tests/first_transaction_step_five_publish.rs b/crates/singlefs-harness/tests/first_transaction_step_five_publish.rs
index bb89271..84edb36 100644
--- a/crates/singlefs-harness/tests/first_transaction_step_five_publish.rs
+++ b/crates/singlefs-harness/tests/first_transaction_step_five_publish.rs
@@ -1,5 +1,5 @@
 //! 里程碑「第一个事务」步 3 / 步 4 / 步 5 的验收：mkfs → 取号 → 暖机两次 → 第一个事务，落到两个文件镜像上，
-//! 录制流按路径切段与 E142（第一个事务的干跑） 第八次跑的产物 `research/results/e142-first-txn-dry-run-2026-09-16-tree-table-200.out`
+//! 录制流按路径切段与 E142（第一个事务的干跑） 第九次跑的产物 `research/results/e142-first-txn-dry-run-2026-09-16-instance-boundary.out`
 //! 逐字对（`name=segments` 五行、`name=root_record` 的反向链、`name=accounting` 的数），盘上的每个单元都由 checker 另一份解析判过。
 
 use std::collections::BTreeSet;
@@ -149,7 +149,7 @@ fn build_pool(tag: &str) -> BuiltPool {
         DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
         DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
     ]);
-    allocator.mark_format_time_units(&[
+    allocator.mark_format_time_units(
         Placement {
             slot: INSTANCE_TABLE_SLOT,
             span: 2,
@@ -158,7 +158,7 @@ fn build_pool(tag: &str) -> BuiltPool {
             slot: TREE_TABLE_GENESIS_SLOT,
             span: 1,
         },
-    ]);
+    );
     let content = file_content();
     let (warm_up, output) = {
         let mut pool = PoolWriter::new(&parameters, &mut devices);
@@ -243,7 +243,9 @@ fn read_journal_record(pool: &BuiltPool, identity: DeviceIdentity, counter: u64)
 
 fn unit_length(identity: TransactionUnit) -> usize {
     match identity {
-        TransactionUnit::Data | TransactionUnit::InodeLeaf => 32768,
+        TransactionUnit::Data | TransactionUnit::InodeLeaf | TransactionUnit::InstanceTable => {
+            32768
+        }
         TransactionUnit::ExtentRoot
         | TransactionUnit::InodeRoot
         | TransactionUnit::AllocationTree
@@ -548,7 +550,9 @@ fn every_index_node_self_checks_with_tight_ascending_keys_and_merkle_checksums_h
                     .kind;
                 key_schema_for_tree_kind(kind).expect("有节点的树都有 key 形态")
             }
-            TransactionUnit::Data | TransactionUnit::InodeLeaf => unreachable!("上面按类跳过了"),
+            TransactionUnit::Data | TransactionUnit::InodeLeaf | TransactionUnit::InstanceTable => {
+                unreachable!("上面按类跳过了")
+            }
         };
         check_index_node_keys(&view, schema).expect("key 区间贴紧、条目严格递增");
         assert_eq!(
@@ -870,15 +874,16 @@ fn allocation_and_accounting_trees_carry_the_byte_table_numbers() {
             .iter()
             .filter(|record| record.generation == CheckpointTxg(0))
             .count(),
-        4,
-        "mkfs 的 m1 / m2 分配代 0"
+        2,
+        "mkfs 的 m1 分配代 0；m2（第 0 版树表）被 A 换下，记录改写成已释放、释放代 3"
     );
     assert_eq!(
         records
             .iter()
             .filter(|record| record.generation == CheckpointTxg(3))
             .count(),
-        16
+        18,
+        "A 的八个落点各两盘 16 条，加 m2 那两条改写成释放代 3"
     );
     assert_eq!(
         records
@@ -946,12 +951,16 @@ fn allocation_and_accounting_trees_carry_the_byte_table_numbers() {
     assert_eq!(value_of(STATISTIC_INODE_WATERMARK), 2);
     for statistic in [
         STATISTIC_UNRECLAIMABLE_BYTES,
-        STATISTIC_DEFER_QUEUE_BYTES,
         STATISTIC_PENDING_DELETE_BYTES,
         STATISTIC_COMMITTED_RESERVATION_BYTES,
     ] {
-        assert_eq!(value_of(statistic), 0, "准入四项 day-1 写 0 行");
+        assert_eq!(value_of(statistic), 0, "准入三项 day-1 写 0 行");
     }
+    assert_eq!(
+        value_of(STATISTIC_DEFER_QUEUE_BYTES),
+        16384,
+        "defer 待释放 = mkfs 那片第 0 版树表单元（1 槽）：A 重写树表把它换下（D3 已定项 7）"
+    );
     for statistic in [
         STATISTIC_ALLOCATED_BYTES,
         STATISTIC_FREE_BYTES,
diff --git a/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs b/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
index 559232f..12aded8 100644
--- a/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
+++ b/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
@@ -15,7 +15,7 @@ use singlefs_harness::crash::{
 use singlefs_harness::segments::StepKind;
 use singlefs_harness::RecordedOperationKind;
 
-/// oracle 那一半：产物第 45 行逐字 `states=262165 … violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=9
+/// oracle 那一半：产物第 45 行逐字 `states=262165 … violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6
 /// verification_failed=0 first_violation=none`，第 48 行 `differing_states=3`。
 #[derive(Debug, PartialEq, Eq)]
 struct OracleCounts {
@@ -185,7 +185,7 @@ fn layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violation
             file_read_states: 7,
             failed_states: 0,
             journal_differing_states: 3,
-            verification_ran_states: 9,
+            verification_ran_states: 6,
             verification_failed_states: 0,
             first_violation: None,
         }
@@ -218,7 +218,7 @@ fn layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_fu
             file_read_states: 7,
             failed_states: 0,
             journal_differing_states: 3,
-            verification_ran_states: 9,
+            verification_ran_states: 6,
             verification_failed_states: 0,
             first_violation: None,
         },
diff --git a/crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs b/crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs
index 09d71ab..ca6e2e9 100644
--- a/crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs
+++ b/crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs
@@ -1,5 +1,5 @@
 //! 里程碑「第一个事务」步 6 的验收：进程外重开两个镜像走完整的挂载路径读出那个文件；坏字节探针与 E142（第一个事务的干跑）
-//! 第八次跑产物 `research/results/e142-first-txn-dry-run-2026-09-16-tree-table-200.out` 的 `name=recover_full` / `name=probe` 行逐字对。
+//! 第九次跑产物 `research/results/e142-first-txn-dry-run-2026-09-16-instance-boundary.out` 的 `name=recover_full` / `name=probe` 行逐字对。
 
 mod common;
 
@@ -39,7 +39,8 @@ fn cold_start_reopens_the_images_and_reads_the_file_back_choosing_instance_one_t
             above_water: 0,
             prefix_applied: 0,
             verification_passed: 0,
-            verification_failed: 0
+            verification_failed: 0,
+            maximum_applied_transaction: 0
         },
         "全环扫描到三条记录，没有一条高于所选根的水位"
     );
diff --git a/crates/singlefs-harness/tests/first_transaction_step_two_data_unit.rs b/crates/singlefs-harness/tests/first_transaction_step_two_data_unit.rs
index d27bddb..a4b2ce4 100644
--- a/crates/singlefs-harness/tests/first_transaction_step_two_data_unit.rs
+++ b/crates/singlefs-harness/tests/first_transaction_step_two_data_unit.rs
@@ -86,7 +86,7 @@ fn mkfs_pool(tag: &str) -> Pool {
         DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
         DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
     ]);
-    allocator.mark_format_time_units(&[
+    allocator.mark_format_time_units(
         Placement {
             slot: INSTANCE_TABLE_SLOT,
             span: 2,
@@ -95,7 +95,7 @@ fn mkfs_pool(tag: &str) -> Pool {
             slot: TREE_TABLE_GENESIS_SLOT,
             span: 1,
         },
-    ]);
+    );
     Pool {
         paths,
         devices,
diff --git a/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs b/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs
index 1b8b3d6..68ad7b3 100644
--- a/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs
+++ b/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs
@@ -218,8 +218,16 @@ fn release_rewrites_the_first_versions_records_and_accounting_moves_them_into_th
             assert!(!record.is_released);
             assert_eq!(record.generation, CheckpointTxg(4), "分配代 = B 的 txg");
             fresh_count += 1;
+        } else if record.slot == SlotNumber(50178) {
+            assert!(record.is_released, "mkfs 的第 0 版树表被 A 换下、已释放");
+            assert_eq!(record.generation, CheckpointTxg(3), "释放代 = A 的 txg");
+            format_time_count += 1;
         } else {
-            assert_eq!(record.generation, CheckpointTxg(0), "mkfs 的单元分配代 0");
+            assert_eq!(
+                record.generation,
+                CheckpointTxg(0),
+                "mkfs 的实例表单元分配代 0"
+            );
             format_time_count += 1;
         }
     }
@@ -246,8 +254,8 @@ fn release_rewrites_the_first_versions_records_and_accounting_moves_them_into_th
         );
         assert_eq!(
             value(STATISTIC_DEFER_QUEUE_BYTES),
-            10 * SLOT_BYTES,
-            "defer 待释放 = A 的八个单元 10 槽"
+            11 * SLOT_BYTES,
+            "defer 待释放 = A 的八个单元 10 槽 + A 换下的 mkfs 树表 1 槽"
         );
         assert_eq!(
             value(STATISTIC_FREE_BYTES),
@@ -322,7 +330,8 @@ fn cold_start_reads_the_second_content_and_the_pool_checker_stays_green() {
             above_water: 0,
             prefix_applied: 0,
             verification_passed: 0,
-            verification_failed: 0
+            verification_failed: 0,
+            maximum_applied_transaction: 0
         },
         "暖机两条 + A + B 四条记录，没有一条高于所选根"
     );
@@ -450,7 +459,8 @@ fn release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not
     let mut pool = build_pool("release-via-mapping");
     let first = pool.output.clone();
     let via_mapping =
-        placements_to_release_via_mapping(&first, &pool.allocator).expect("A 的六条映射都在");
+        placements_to_release_via_mapping(&first, &pool.allocator, &TransactionUnit::IN_BUMP_ORDER)
+            .expect("A 的六条映射都在");
     assert_eq!(
         via_mapping,
         first.placements(),
@@ -568,14 +578,14 @@ fn repeated_overwrites_report_a_full_allocation_node_instead_of_panicking() {
             .iter()
             .filter(|record| record.is_released)
             .count(),
-        16 * 49,
-        "报错在动分配器之前：最后一版的八个落点没被释放"
+        16 * 49 + 2,
+        "报错在动分配器之前：最后一版的八个落点没被释放（加 mkfs 树表那两条，A 换下的）"
     );
 }
 
-/// 可再分配谓词（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）在步 5 之前没有实现：这条用例把今天的形态钉住——
-/// 释放过的落点一个都不再发出去、defer 只增不减、空闲随分配单调减。它是 C22（刚释放的块立即重分配）的弱形态：把 `mark_released` 改成清位图（立即复用）它红；
-/// 步 5 把回收接上那天它也必须红（defer 会减、50180 会回来），到时改成按谓词判。
+/// 可再分配谓词（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）：F 不抬（恒 0）时一个已释放的落点都回不来——
+/// 这条用例把 F = 0 的形态钉住：释放过的落点一个都不再发出去、defer 只增不减、空闲随分配单调减。它是 C22（刚释放的块立即重分配）的弱形态：
+/// 把 `mark_released` 改成清位图（立即复用）它红；抬 F 之后的回收与复用在步 5 的用例里（`second_transaction_step_five_reuse.rs`）。
 #[test]
 fn released_placements_are_not_handed_out_again_before_reclaim_exists() {
     let mut pool = build_pool("no-reclaim-yet");
@@ -588,8 +598,8 @@ fn released_placements_are_not_handed_out_again_before_reclaim_exists() {
     let device_map = &pool.allocator.devices[0];
     assert_eq!(
         device_map.deferred_slots(),
-        20,
-        "A 与 B 的 20 槽都在 defer 队列里，一个都没放回"
+        21,
+        "A 与 B 的 20 槽加 mkfs 树表那 1 槽都在 defer 队列里，一个都没放回"
     );
     assert_eq!(
         device_map.allocated_slots(),
@@ -759,7 +769,11 @@ fn publish_running_out_of_space_midway_leaves_the_allocator_as_it_was() {
         "失败的发布退回：A 的记录没改写成已释放、B 的数据单元没留下记录"
     );
     let device_map = &pool.allocator.devices[0];
-    assert_eq!(device_map.deferred_slots(), 0, "A 的落点没进 defer 队列");
+    assert_eq!(
+        device_map.deferred_slots(),
+        1,
+        "A 的落点没进 defer 队列；队列里只有 A 换下的 mkfs 树表那 1 槽"
+    );
     assert!(
         device_map.is_free(SlotNumber(50182)),
         "B 拿到过的数据槽退回去了"
diff --git a/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs b/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
index 1d9e898..b894829 100644
--- a/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
+++ b/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
@@ -7,8 +7,11 @@ mod common;
 
 use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
 use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::mount::{
+    mount_rollback, mount_writable, raise_rollback_floor, RollbackTarget, ShadowLedger,
+};
 use singlefs_core::recovery::RecoveryOutcome;
-use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
+use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
 use singlefs_harness::crash::{
     closed_form_state_count, enumerate_layer0_selecting_versions, enumerate_layer0_versions,
     evaluate_state_for_versions, writes_and_segments, Layer0Tally, MemoryPool, PublishedVersion,
@@ -28,16 +31,46 @@ struct Prepared {
     base: MemoryPool,
     writes: Vec<RetainedWrite>,
     segments: Vec<Vec<usize>>,
-    /// B 的根槽 FUA 写在写表里的下标。
+    /// 被判的那次根槽 FUA 写在写表里的下标：脚本最后一次发布的根（到 B 为止就是 B 的，到 C 为止是 C 的）。
     judged_root_index: usize,
     /// A 的根槽 FUA 写在写表里的下标：它之后的写都是 B 的。
     first_root_index: usize,
     versions: Vec<PublishedVersion>,
 }
 
-fn overwrite(pool: &mut BuiltPool) {
+/// 固定脚本跑到哪一步（里程碑「第二个事务」步 0）：到 B 为止是靶向对照用的两次发布流；到 C 为止多了进程重开、可写挂载
+/// （取号、写行、暖机两次）与发布 C，是层 0 全量跑的那条流。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum Script {
+    SecondVersionOnly,
+    ThirdVersion,
+    /// 到 D 为止：C 之后进程再退出、重开走管理员回退到 A 的根 (1, 3)——取号 3、写回退行与中间实例行的发布 D（txg 9）、暖机一次（txg 10）。
+    RollbackToFirstVersion,
+    /// 到 E 为止：回退之后再覆盖写四次（txg 11–14，第一次释放 A 的数据单元、释放代 11），抬 F 到 11（两次空发布 txg 15、16），
+    /// 再发布 E（txg 17）——数据单元落回最低的可再分配偶数槽对 50176。
+    ReuseAfterRaisingFloor,
+}
+
+const THIRD_FILE_BYTES: usize = 2500;
+
+fn later_content(seed: usize) -> Vec<u8> {
+    (0..3000 + seed)
+        .map(|index| u8::try_from((index * 5 + seed) % 251).expect("小于 256"))
+        .collect()
+}
+
+fn third_content() -> Vec<u8> {
+    (0..THIRD_FILE_BYTES)
+        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
+        .collect()
+}
+
+fn overwrite(
+    pool: &mut BuiltPool,
+    content: &[u8],
+    instance: InstanceGeneration,
+) -> TransactionOutput {
     let parameters = parameters();
-    let content = second_content();
     let devices = pool.devices.as_mut().expect("镜像还开着");
     let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
     publish_overwrite(
@@ -45,52 +78,204 @@ fn overwrite(pool: &mut BuiltPool) {
         &mut pool.allocator,
         &pool.output,
         FirstFile {
-            content: &content,
+            content,
             write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
         },
-        InstanceGeneration(1),
+        instance,
     )
-    .expect("覆盖写");
+    .expect("覆盖写")
 }
 
-fn prepare(tag: &str) -> Prepared {
+fn prepare(tag: &str, script: Script) -> Prepared {
     let mut pool = build_pool(tag);
-    overwrite(&mut pool);
+    let second = overwrite(&mut pool, &second_content(), InstanceGeneration(1));
+    let mut versions = vec![
+        PublishedVersion {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(3),
+            content: file_content(),
+        },
+        PublishedVersion {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(4),
+            content: second_content(),
+        },
+    ];
+    if script != Script::SecondVersionOnly {
+        pool.output = second;
+        // 进程重开：取号 2、写行、暖机两次（txg 5 / 6 落盘 0、txg 7 落盘 1），文件还是第二次的内容。
+        let mut devices = pool.reopen_recorded();
+        let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
+        pool.devices = Some(devices);
+        pool.allocator = mounted.allocator;
+        pool.output = mounted.current;
+        for txg in 5..=7 {
+            versions.push(PublishedVersion {
+                instance: InstanceGeneration(2),
+                checkpoint_txg: CheckpointTxg(txg),
+                content: second_content(),
+            });
+        }
+        let third = overwrite(&mut pool, &third_content(), InstanceGeneration(2));
+        versions.push(PublishedVersion {
+            instance: InstanceGeneration(2),
+            checkpoint_txg: CheckpointTxg(8),
+            content: third_content(),
+        });
+        if script != Script::ThirdVersion {
+            pool.output = third;
+            // 进程再退出、重开走回退到 A 的根：取号 3、发布 D（txg 9 落盘 0）写回退行 (1, 3, 0) 与中间实例行 (2, 0, 0)、
+            // 暖机一次（txg 10 落盘 1）；文件回到第一次的内容。
+            let mut reopened_for_rollback = pool.reopen_recorded();
+            let rolled_back = mount_rollback(
+                &parameters(),
+                &mut reopened_for_rollback,
+                RollbackTarget {
+                    instance: InstanceGeneration(1),
+                    checkpoint_txg: CheckpointTxg(3),
+                },
+                ShadowLedger::On,
+            )
+            .expect("回退");
+            pool.devices = Some(reopened_for_rollback);
+            pool.allocator = rolled_back.allocator;
+            pool.output = rolled_back.current;
+            for txg in 9..=10 {
+                versions.push(PublishedVersion {
+                    instance: InstanceGeneration(3),
+                    checkpoint_txg: CheckpointTxg(txg),
+                    content: file_content(),
+                });
+            }
+            if script == Script::ReuseAfterRaisingFloor {
+                let mut latest = Vec::new();
+                for (txg, seed) in [(11u64, 17usize), (12, 19), (13, 23), (14, 29)] {
+                    latest = later_content(seed);
+                    pool.output = overwrite(&mut pool, &latest, InstanceGeneration(3));
+                    versions.push(PublishedVersion {
+                        instance: InstanceGeneration(3),
+                        checkpoint_txg: CheckpointTxg(txg),
+                        content: latest.clone(),
+                    });
+                }
+                let mut current = pool.output.clone();
+                let raise_devices = pool.devices.as_mut().expect("镜像还开着");
+                let raised = raise_rollback_floor(
+                    &parameters(),
+                    raise_devices,
+                    &mut pool.allocator,
+                    &mut current,
+                    CheckpointTxg(11),
+                )
+                .expect("抬 F");
+                assert_eq!(raised.publishes.len(), 2, "txg 15 落盘 0、txg 16 落盘 1");
+                pool.output = current;
+                for txg in 15..=16 {
+                    versions.push(PublishedVersion {
+                        instance: InstanceGeneration(3),
+                        checkpoint_txg: CheckpointTxg(txg),
+                        content: latest.clone(),
+                    });
+                }
+                let reuse = overwrite(&mut pool, &later_content(31), InstanceGeneration(3));
+                assert_eq!(
+                    reuse.data_pointer.locations[0].slot.0, 50176,
+                    "E 的数据单元落回最低的可再分配偶数槽对（mkfs 实例表那片，D 放掉的）"
+                );
+                versions.push(PublishedVersion {
+                    instance: InstanceGeneration(3),
+                    checkpoint_txg: CheckpointTxg(17),
+                    content: later_content(31),
+                });
+            }
+        }
+    }
     let base = pool.memory_pool_after_mkfs();
     let operations = pool.retained_operations();
     let (writes, segments) =
         writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
-    assert_eq!(
-        segments.iter().map(Vec::len).collect::<Vec<_>>(),
-        vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 2],
-        "A 的两个超级块槽写与 B 的 16 个单元写合成一段：整条流按屏障切，不按发布切"
-    );
-    assert_eq!(writes.len(), 54, "取号 2 + 暖机 10 + A 21 + B 21 次写");
+    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
+    match script {
+        Script::SecondVersionOnly => {
+            assert_eq!(
+                sizes,
+                vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 2],
+                "A 的两个超级块槽写与 B 的 16 个单元写合成一段：整条流按屏障切，不按发布切"
+            );
+            assert_eq!(writes.len(), 54, "取号 2 + 暖机 10 + A 21 + B 21 次写");
+        }
+        Script::ThirdVersion => {
+            // B 的两个超级块槽写与取号的两个合成一段（4）；写行发布 10 个单元写（实例表 + 四个固定点单元，各两盘）；
+            // 每次暖机 8 个单元写与上一次发布的两个超级块槽写合成一段（10）；C 的 16 个单元写与 txg 7 的超级块槽写合成 18。
+            assert_eq!(
+                sizes,
+                vec![
+                    2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 4, 10, 2, 1, 10, 2, 1, 10, 2, 1, 18, 2,
+                    1, 2
+                ],
+                "固定脚本到 C 为止的段序列"
+            );
+            assert_eq!(
+                writes.len(),
+                54 + 2 + 15 + 13 + 13 + 21,
+                "取号 2 + 写行 15 + 暖机 13 × 2 + C 21"
+            );
+        }
+        Script::RollbackToFirstVersion => {
+            // C 的两个超级块槽写与回退取号的两个合成一段（4）；D 是写回退行的发布：10 个单元写（实例表 + 四个固定点单元，各两盘）；
+            // 暖机一次 8 个单元写与 D 的两个超级块槽写合成一段（10）。
+            assert_eq!(
+                sizes,
+                vec![
+                    2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 4, 10, 2, 1, 10, 2, 1, 10, 2, 1, 18, 2,
+                    1, 4, 10, 2, 1, 10, 2, 1, 2
+                ],
+                "固定脚本到 D 为止的段序列"
+            );
+            assert_eq!(
+                writes.len(),
+                118 + 2 + 15 + 13,
+                "到 C 118 + 取号 2 + D 15 + 暖机 13"
+            );
+        }
+        Script::ReuseAfterRaisingFloor => {
+            // 四次覆盖写各 16 个单元写并上一次的两个超级块槽写（18）；抬 F 的两次空发布各 8 个单元写并上两个超级块槽写（10）；E 同覆盖写。
+            assert_eq!(
+                sizes,
+                vec![
+                    2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 4, 10, 2, 1, 10, 2, 1, 10, 2, 1, 18, 2,
+                    1, 4, 10, 2, 1, 10, 2, 1, 18, 2, 1, 18, 2, 1, 18, 2, 1, 18, 2, 1, 10, 2, 1, 10,
+                    2, 1, 18, 2, 1, 2
+                ],
+                "固定脚本到 E 为止的段序列"
+            );
+            assert_eq!(
+                writes.len(),
+                148 + 4 * 21 + 2 * 13 + 21,
+                "到 D 148 + 四次覆盖写 84 + 抬 F 两次 26 + E 21"
+            );
+        }
+    }
     let root_indexes: Vec<usize> = writes
         .iter()
         .enumerate()
         .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
         .map(|(index, _)| index)
         .collect();
-    assert_eq!(root_indexes.len(), 4, "暖机两代 + A + B 四条根槽写");
+    let expected_roots = match script {
+        Script::SecondVersionOnly => 4,
+        Script::ThirdVersion => 8,
+        Script::RollbackToFirstVersion => 10,
+        Script::ReuseAfterRaisingFloor => 17,
+    };
+    assert_eq!(root_indexes.len(), expected_roots, "每次发布一条根槽写");
     Prepared {
         base,
         writes,
         segments,
-        judged_root_index: root_indexes[3],
+        judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
         first_root_index: root_indexes[2],
-        versions: vec![
-            PublishedVersion {
-                instance: InstanceGeneration(1),
-                checkpoint_txg: CheckpointTxg(3),
-                content: file_content(),
-            },
-            PublishedVersion {
-                instance: InstanceGeneration(1),
-                checkpoint_txg: CheckpointTxg(4),
-                content: second_content(),
-            },
-        ],
+        versions,
     }
 }
 
@@ -127,11 +312,27 @@ fn assert_checker_and_record_checker_clean(tally: &Layer0Tally) {
     }
 }
 
-/// 平时跑的那一份：两个 18 写的段不展开（只以整段持久进入后面的状态），其余每段任意子集。
+/// 到 C 为止的固定脚本：段序列登记表（layout/01-first-txn.md 八）「装置钉住」的那条数组由 `prepare` 里的断言钉住；层 0 全量与快的那条都在到 D 的脚本上跑。
+#[test]
+fn the_fixed_script_through_the_third_publish_keeps_its_registered_segment_sequence() {
+    let prepared = prepare("layer0-c-registered", Script::ThirdVersion);
+    assert_eq!(prepared.segments.len(), 26);
+    assert_eq!(closed_form_state_count(&prepared.segments), 789_555);
+}
+
+/// 到 D 为止的固定脚本：同上，33 段、闭式 791624。
+#[test]
+fn the_fixed_script_through_the_rollback_publish_keeps_its_registered_segment_sequence() {
+    let prepared = prepare("layer0-d-registered", Script::RollbackToFirstVersion);
+    assert_eq!(prepared.segments.len(), 33);
+    assert_eq!(closed_form_state_count(&prepared.segments), 791_624);
+}
+
+/// 平时跑的那一份：三个 18 写的段与三个 10 写的段不展开（只以整段持久进入后面的状态），其余每段任意子集。
 #[test]
 fn every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims() {
-    let prepared = prepare("layer0-b-fast");
-    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 18;
+    let prepared = prepare("layer0-e-fast", Script::ReuseAfterRaisingFloor);
+    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 10;
     let tally = enumerate_layer0_selecting_versions(
         &prepared.base,
         &prepared.writes,
@@ -143,7 +344,7 @@ fn every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_r
     let expanded: Vec<Vec<usize>> = prepared
         .segments
         .iter()
-        .filter(|segment| segment.len() < 18)
+        .filter(|segment| segment.len() < 10)
         .cloned()
         .collect();
     assert_eq!(
@@ -151,7 +352,10 @@ fn every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_r
         closed_form_state_count(&expanded),
         "展开的段按闭式数"
     );
-    assert_eq!(tally.states, 26);
+    assert_eq!(
+        tally.states, 108,
+        "1 + 二十个 2 写段各 3 + 两个 4 写段各 15 + 十七个 1 写段各 1"
+    );
     assert_eq!(
         tally.violations, 0,
         "第一处违例：{:?}",
@@ -166,14 +370,14 @@ fn every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_r
     assert_checker_and_record_checker_clean(&tally);
 }
 
-/// 全量：两次发布的流 13 段、闭式 524312 个状态（第一个事务的 262165 减掉末尾那个 2 写的段、加上 B 的四段与合成的 18 写段）。
+/// 全量：固定脚本到 E 为止 54 段、闭式 2 104 413 个状态（八个 18 写段各 262143、七个 10 写段各 1023、两个 4 写段各 15，其余 2 写段各 3、1 写段各 1）。
 /// 54 号门禁在 release 下跑它，认下面打印的 `LAYER0B` 行里 `exhaustive=true`。
 #[test]
-#[ignore = "全量 524312 个状态、每个两遍恢复 + checker，debug 下几分钟；门禁 54 号在 release 下跑"]
-fn full_enumeration_of_the_two_publish_stream_is_exhaustive_and_clean() {
-    let prepared = prepare("layer0-b-full");
+#[ignore = "全量 2104413 个状态、每个两遍恢复 + checker，debug 下半小时以上；门禁 54 号在 release 下跑"]
+fn full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean() {
+    let prepared = prepare("layer0-e-full", Script::ReuseAfterRaisingFloor);
     let closed_form = closed_form_state_count(&prepared.segments);
-    assert_eq!(closed_form, 524_312, "闭式：1 + Σ(2^|段| − 1)，十三段");
+    assert_eq!(closed_form, 2_104_413, "闭式：1 + Σ(2^|段| − 1)，五十四段");
     let tally = enumerate_layer0_versions(
         &prepared.base,
         &prepared.writes,
@@ -216,7 +420,7 @@ fn full_enumeration_of_the_two_publish_stream_is_exhaustive_and_clean() {
 /// 靶向的阳性对照：把持久集合手工摆成四个形状，oracle、journal 承重、记录核对器各要在它该红的那一格红。
 #[test]
 fn targeted_controls_on_the_second_publish_go_red_where_they_should() {
-    let prepared = prepare("layer0-b-controls");
+    let prepared = prepare("layer0-b-controls", Script::SecondVersionOnly);
     let all = vec![true; prepared.writes.len()];
     let is_second_publish = |index: usize| index > prepared.first_root_index;
     let kinds_of_second_publish = |kind: StepKind| -> Vec<usize> {
```

## 二、新文件 `crates/singlefs-core/src/mount.rs` 全文

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
    /// 回退的目标根自己那条记录读不出：新实例的第一条 jsn 要接在它之后（C340（回退之后记录链从哪条之后接没有定义） 取 P1）。
    RollbackRecordUnreadable(RollbackTarget),
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

/// 从上一版的分配记录重建分配器，再按 F_生效 把可再分配的已释放落点回收（D16（发布语义） 已定项 1）。
fn rebuilt_allocator<Device: BlockDevice>(
    devices: &[(DeviceIdentity, Device)],
    superblock: &crate::superblock::Superblock,
    previous: &TransactionOutput,
) -> (PoolAllocator, CheckpointTxg) {
    let device_maps: Vec<DeviceFreeMap> = devices
        .iter()
        .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
        .collect();
    let mut allocator =
        PoolAllocator::rebuild_from_records(device_maps, previous.allocation_records.clone());
    let floor = effective_rollback_floor(
        devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    );
    allocator.reclaim_released_up_to(floor);
    (allocator, floor)
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
    let reclaimed = allocator.reclaim_released_up_to(new_floor);
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
    let (allocator, effective_floor) = rebuilt_allocator(devices, &superblock, &previous);
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
    if target.checkpoint_txg < newest_root.rollback_floor {
        return Err(MountError::RollbackTargetNotACandidate {
            target,
            reason: "txg 低于回退下界 F",
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
    let own_record = records
        .values()
        .find(|record| {
            record.instance == target.instance && record.checkpoint_txg == target.checkpoint_txg
        })
        .cloned()
        .ok_or(MountError::RollbackRecordUnreadable(target))?;
    let next_counter = own_record.counter + 1;
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
    let (mut allocator, effective_floor) = rebuilt_allocator(devices, &superblock, &previous);
    // 影子账：被抛弃的根 = 根环里 (txg, 实例) 大于 R_old 的每一条可读根；只隔离它们引用而 R_old 不引用的槽（窄读法，
    // 2026-09-16 用户定案）。
    if shadow_ledger == ShadowLedger::On {
        let own_slots: BTreeSet<(u32, u64)> = previous
            .allocation_records
            .iter()
            .map(|record| (record.device.0, record.slot.0))
            .collect();
        let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();
        for root in &roots {
            if (root.checkpoint_txg, root.instance) <= (target.checkpoint_txg, target.instance) {
                continue;
            }
            for record in allocation_records_under_root(&*devices, root)? {
                let key = (record.device.0, record.slot.0);
                if own_slots.contains(&key) || !isolated.insert(key) {
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
```

## 二、新文件 `crates/singlefs-core/src/instance_table.rs` 全文

```rust
//! 实例表单元里的记录（D18（块里携带什么信息） 已定项 11）：`kind` 0 行与末尾的链指针记录。
//! 恢复（回退行的 W）、可写挂载与回退（写行）都从这里解；checker 是独立解析器，不共用这份。

use crate::address::{CheckpointTxg, InstanceGeneration};
use crate::bytes::{ByteReader, ByteWriter};
use crate::unit::parse_packed_unit;
use singlefs_format::INSTANCE_ROW_BYTES;

/// 实例表 `kind` 0 行记录（D18（块里携带什么信息） 已定项 11）：
/// `kind 1 | 实例代号 4 | 所选根的 checkpoint_txg 8 | 属于该实例的最大已施加事务号 W 8 | flags 1（bit0 = 回退行）| 预留 66`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceRow {
    pub instance: InstanceGeneration,
    pub selected_root_txg: CheckpointTxg,
    pub applied_transaction_high_water: u64,
    pub is_rollback: bool,
}

const INSTANCE_ROW_KIND_ROW: u8 = 0;
const INSTANCE_ROW_KIND_CHAIN: u8 = 1;
const INSTANCE_ROW_FLAG_ROLLBACK: u8 = 0b0000_0001;

impl InstanceRow {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize::try_from(INSTANCE_ROW_BYTES).expect("88"));
        writer.put_u8(INSTANCE_ROW_KIND_ROW);
        writer.put_u32(self.instance.0);
        writer.put_u64(self.selected_root_txg.0);
        writer.put_u64(self.applied_transaction_high_water);
        writer.put_u8(if self.is_rollback {
            INSTANCE_ROW_FLAG_ROLLBACK
        } else {
            0
        });
        writer.skip(66);
        writer.assert_position(INSTANCE_ROW_BYTES, "实例表行记录");
        writer.into_bytes()
    }

    /// `kind` 0 才是行；链指针记录（`kind` 1）与别的 `kind` 返回 `None`。flags 只认 bit0，别的位非 0 拒收。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != usize::try_from(INSTANCE_ROW_BYTES).expect("88")
            || bytes[0] != INSTANCE_ROW_KIND_ROW
        {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 1);
        let instance = InstanceGeneration(reader.get_u32());
        let selected_root_txg = CheckpointTxg(reader.get_u64());
        let applied_transaction_high_water = reader.get_u64();
        let flags = reader.get_u8();
        if flags & !INSTANCE_ROW_FLAG_ROLLBACK != 0 {
            return None;
        }
        Some(Self {
            instance,
            selected_root_txg,
            applied_transaction_high_water,
            is_rollback: flags & INSTANCE_ROW_FLAG_ROLLBACK != 0,
        })
    }
}

/// 一片实例表单元里的记录：行在前、链指针记录最末（D18（块里携带什么信息） 已定项 11）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceTableRecords {
    pub rows: Vec<InstanceRow>,
    pub chain_record: Vec<u8>,
}

impl InstanceTableRecords {
    /// 从一片实例表单元解出来；行与链指针之外的 `kind`、链指针不在最末、没有链指针，都是 `None`。
    #[must_use]
    pub fn parse(instance_table_unit: &[u8]) -> Option<Self> {
        let unit = parse_packed_unit(instance_table_unit).ok()?;
        let (last, rows_bytes) = unit.records.split_last()?;
        if last.first().copied() != Some(INSTANCE_ROW_KIND_CHAIN) {
            return None;
        }
        let mut rows = Vec::with_capacity(rows_bytes.len());
        for bytes in rows_bytes {
            rows.push(InstanceRow::parse(bytes)?);
        }
        Some(Self {
            rows,
            chain_record: last.clone(),
        })
    }

    #[must_use]
    pub fn to_records(&self) -> Vec<Vec<u8>> {
        let mut records: Vec<Vec<u8>> = self.rows.iter().map(InstanceRow::to_bytes).collect();
        records.push(self.chain_record.clone());
        records
    }
}
```

## 二、新文件 `crates/singlefs-harness/tests/second_transaction_step_three_second_instance.rs` 全文

```rust
//! 里程碑「第二个事务」步 3 的验收：发布 B 之后进程退出、重开两个镜像走可写挂载——恢复、从盘上重建上一版与分配器、
//! 取实例代号 2、给实例 1 写行 (1, 4, 0)、写行发布 txg 5、暖机两次（txg 6 落盘 0 白费、txg 7 落盘 1）——再发布 C；
//! 冷启动择实例 2 的根读回第三次的内容；实例表读回那一行；池级 checker 全绿（含 I-3.8）。

mod common;

use common::{build_pool, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::journal::back_chain_of;
use singlefs_core::journal::record_offset;
use singlefs_core::mount::{mount_writable, InstanceRow, InstanceTableRecords, Mounted};
use singlefs_core::records::{
    STATISTIC_ALLOCATED_BYTES, STATISTIC_DEFER_QUEUE_BYTES, STATISTIC_FREE_BYTES,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_overwrite, publish_version, FirstFile, InstanceTablePlan, PoolWriter, PublishPlan,
    TransactionOutput, TransactionUnit,
};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_format::SLOT_BYTES;

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

/// 同一个进程、同一个实例里再发布一版（发布 B 用；步 3 的「预置一条实例 1 的记录」也用它造）。
fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写")
}

/// A → B 之后进程退出、重开并可写挂载。
fn build_publish_second_version_and_remount(tag: &str) -> (BuiltPool, TransactionOutput, Mounted) {
    let mut pool = build_pool(tag);
    let second = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    pool.output = second.clone();
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    (pool, second, mounted)
}

fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(CheckpointTxg(txg));
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

fn accounting_value(output: &TransactionOutput, statistic: u16, device: DeviceIdentity) -> u64 {
    output
        .accounting_entries
        .iter()
        .find(|entry| entry.statistic == statistic && entry.device == device)
        .map(|entry| entry.value)
        .expect("带设备维的行每盘一行")
}

/// 验收第一、二条：取号 2；行 (1, 4, 0)；写行发布 txg 5 重写实例表 + 四个固定点单元、事务号 0、反向链 0；暖机两次落到两块盘；
/// 发布 C 接在后面（txg 8、事务号 1、反向链接 txg 7 那条）；冷启动择 (2, 8) 读回第三次的内容；记账与分配器对得上；checker 全绿。
#[test]
fn remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version()
{
    let (mut pool, second, mut mounted) =
        build_publish_second_version_and_remount("step-three-remount");
    let output = &mounted.output;
    assert_eq!(
        output.instance,
        InstanceGeneration(2),
        "取号 = max(超级块 1, 根环 1) + 1"
    );
    assert_eq!(
        (
            output.chosen_root.instance,
            output.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(4)),
        "B 的根持久了：所选根就是它"
    );
    assert_eq!(output.effective_root, output.chosen_root, "没有记录要施加");
    assert_eq!(output.journal.valid_records, 4);
    assert_eq!(output.journal.prefix_applied, 0);
    assert_eq!(
        output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(4),
            applied_transaction_high_water: 0,
            is_rollback: false,
        }],
        "实例 1 那一行 (1, 4, 0)；实例 0 不写行"
    );

    let row = &output.row_publish;
    assert_eq!(
        (row.root.instance, row.root.checkpoint_txg),
        (InstanceGeneration(2), CheckpointTxg(5))
    );
    assert_eq!(row.record.counter, 5, "jsn 全池接着走");
    assert_eq!(row.record.transaction, 0, "空发布的事务号 0");
    assert_eq!(row.record.back_chain, 0, "本实例的第一条反向链恒 0");
    assert_eq!(
        row.rewritten,
        [
            TransactionUnit::InstanceTable,
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ],
        "写行发布重写实例表 + 四个固定点单元（记账树已存在 ⇒ 空发布也重写，D16 已定项 9）"
    );
    assert_eq!(row.record.named.len(), 5, "点名的只有重写的五个");
    assert_eq!(row.data_pointer, second.data_pointer, "文件角色照抄 B 的");
    assert_eq!(row.inode_record, second.inode_record);
    for identity in [
        TransactionUnit::Data,
        TransactionUnit::ExtentRoot,
        TransactionUnit::InodeLeaf,
        TransactionUnit::InodeRoot,
    ] {
        assert_eq!(
            row.unit(identity).slot,
            second.unit(identity).slot,
            "{identity:?} 照抄、不换落点"
        );
    }
    assert_eq!(
        row.tree_root_pointer(singlefs_format::TREE_IDENTIFIER_EXTENT),
        second.tree_root_pointer(singlefs_format::TREE_IDENTIFIER_EXTENT),
        "树表里 extent 根指针照抄 B 的"
    );
    let table = InstanceTableRecords::parse(&row.unit(TransactionUnit::InstanceTable).bytes)
        .expect("写出的实例表单元解得开");
    assert_eq!(table.rows, output.rows_written);
    assert_eq!(
        row.released.len(),
        5,
        "写行发布释放 mkfs 的实例表单元 + B 的四个固定点单元"
    );

    assert_eq!(
        output
            .warm_up_publishes
            .iter()
            .map(|publish| publish.root.checkpoint_txg.0)
            .collect::<Vec<_>>(),
        [6, 7],
        "实例 2 从 txg 5 起要两次：txg 6 落盘 0 白费、txg 7 落盘 1"
    );
    assert_eq!(region_device(5), DeviceIdentity(0));
    assert_eq!(region_device(6), DeviceIdentity(0));
    assert_eq!(region_device(7), DeviceIdentity(1));
    for publish in &output.warm_up_publishes {
        assert_eq!(
            publish.rewritten,
            [
                TransactionUnit::AllocationTree,
                TransactionUnit::AccountingTree,
                TransactionUnit::MappingTree,
                TransactionUnit::TreeTable,
            ]
        );
        assert_eq!(publish.record.transaction, 0);
        assert_eq!(
            publish.released.len(),
            4,
            "每次暖机释放上一次的四个固定点单元"
        );
    }
    assert_eq!(
        mounted.current.root.checkpoint_txg,
        CheckpointTxg(7),
        "接下来的发布接在 txg 7 后面"
    );

    // 发布 C：实例 2 的第一次文件发布。
    let third = third_content();
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let third_publish = publish_overwrite(
        &mut writer,
        &mut mounted.allocator,
        &mounted.current,
        FirstFile {
            content: &third,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
        },
        InstanceGeneration(2),
    )
    .expect("发布 C");
    assert_eq!(
        (
            third_publish.root.instance,
            third_publish.root.checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(8))
    );
    assert_eq!(third_publish.record.counter, 8);
    assert_eq!(
        third_publish.record.transaction, 1,
        "事务号按实例计数从 1 起"
    );
    assert_eq!(
        third_publish.record.back_chain,
        back_chain_of(&mounted.current.record_bytes),
        "反向链接 txg 7 那条"
    );
    assert_eq!(third_publish.rewritten, TransactionUnit::IN_BUMP_ORDER);
    assert_eq!(
        third_publish.released.len(),
        8,
        "释放 B 的四个文件单元 + txg 7 的四个固定点单元"
    );
    assert_eq!(
        third_publish.inode_record.object_birth,
        CheckpointTxg(3),
        "对象出生代照旧"
    );
    let device_map = &mounted.allocator.devices[0];
    // 占着：mkfs 3 + A 10 + B 10 + txg5（实例表 2 + 4）+ txg6 4 + txg7 4 + C 10 = 47；
    // defer：mkfs 树表 1 + A 10 + B 10 + mkfs 实例表 2 + txg5 的 4 + txg6 的 4 + txg7 的 4 = 35。
    assert_eq!(device_map.allocated_slots(), 47);
    assert_eq!(device_map.deferred_slots(), 35);
    assert_eq!(device_map.free_slots(), 211_968 - 47);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert_eq!(
            accounting_value(&third_publish, STATISTIC_ALLOCATED_BYTES, device),
            47 * SLOT_BYTES
        );
        assert_eq!(
            accounting_value(&third_publish, STATISTIC_DEFER_QUEUE_BYTES, device),
            35 * SLOT_BYTES
        );
        assert_eq!(
            accounting_value(&third_publish, STATISTIC_FREE_BYTES, device),
            (211_968 - 47) * SLOT_BYTES
        );
    }
    assert_eq!(
        third_publish.unit(TransactionUnit::Data).slot,
        SlotNumber(50184),
        "C 的数据单元落在 B 之后的下一对偶数空槽（A、B 的都占着）"
    );

    // 冷启动：择 (2, 8)，读回第三次的内容；八条记录都在环里、没有一条高于水位。
    let reopened = pool.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(8)),
            content: third.clone()
        }
    );
    assert_eq!(report.journal.valid_records, 8);
    assert_eq!(report.journal.above_water, 0);
    assert_eq!(report.journal.prefix_applied, 0);
    assert_eq!(
        recover(&reopened, JournalPolicy::Ignore).outcome,
        report.outcome,
        "根槽已持久：看不看 journal 一样"
    );

    // 池级 checker：全部判 Holds，I-3.8 真被评估过。
    let image = pool.memory_pool();
    let verdicts = check_pool_image(&image);
    let violated: Vec<_> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .collect();
    assert!(violated.is_empty(), "checker 判红：{violated:?}");
    for invariant in ["I-3.1", "I-3.8", "I-5.2", "I-7.2", "I-7.7"] {
        assert!(
            verdicts
                .iter()
                .any(|(name, verdict)| *name == invariant && *verdict == InvariantVerdict::Holds),
            "{invariant} 要真被评估过且成立"
        );
    }
}

/// 验收第二条的反面（D16 已定项 8 买的东西）：把实例 2 在盘 0 上的两个根槽（txg 5、6）都改坏，冷启动仍择到盘 1 上 txg 7 那个根、
/// 内容还是第二次的；暖机只推一次的话盘 1 上没有实例 2 的根，恢复退回实例 1。
#[test]
fn damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device() {
    let (pool, _second, mounted) =
        build_publish_second_version_and_remount("step-three-one-device-damaged");
    let mut image = pool.memory_pool();
    for txg in [5u64, 6] {
        let target = target_for_publish(CheckpointTxg(txg));
        image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
    }
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.effective_root,
        Some((InstanceGeneration(2), CheckpointTxg(7))),
        "盘 0 上实例 2 的根都坏了，盘 1 上 txg 7 那个还在"
    );
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(7)),
            content: second_content()
        }
    );
    assert_eq!(mounted.output.warm_up_publishes.len(), 2);
}

/// 验收第三条：预置一条实例 1 的记录（jsn 5、txg 5、事务号 3，点名的单元也在盘上），把它的根槽改坏 ⇒ 重开那一刻所选根是 B 的根、
/// 同一个实例、没有回退行 ⇒ 恢复必须施加它，行写 (1, 5, 3)，写行发布的 txg 从 6 起。
#[test]
fn stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row(
) {
    let mut pool = build_pool("step-three-stray-record");
    let second = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    pool.output = second;
    let stray = overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(1));
    assert_eq!(
        (
            stray.root.checkpoint_txg,
            stray.record.counter,
            stray.record.transaction
        ),
        (CheckpointTxg(5), 5, 3)
    );
    let mut devices: Vec<(DeviceIdentity, Recorded)> = pool.reopen_recorded();
    // 把 txg 5 的根槽改坏：只留记录与单元，模拟「根槽没持久」。
    let target = target_for_publish(CheckpointTxg(5));
    let device = region_device(5);
    let offset = slot_offset(target, 4096);
    let (_, damaged_device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("区域归属的盘在池里");
    let mut garbage = vec![0u8; 4096];
    garbage[0] = 0xff;
    damaged_device
        .write_at(offset, &garbage, WriteDurability::Plain)
        .expect("改坏根槽");
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    assert_eq!(
        (
            mounted.output.chosen_root.instance,
            mounted.output.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(4))
    );
    assert_eq!(mounted.output.journal.prefix_applied, 1, "jsn 5 被施加");
    assert_eq!(mounted.output.journal.maximum_applied_transaction, 3);
    assert_eq!(
        mounted.output.effective_root.checkpoint_txg,
        CheckpointTxg(5),
        "施加之后的根是第 5 代"
    );
    assert_eq!(
        mounted.output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(5),
            applied_transaction_high_water: 3,
            is_rollback: false,
        }]
    );
    assert_eq!(
        mounted.output.row_publish.root.checkpoint_txg,
        CheckpointTxg(6),
        "新实例的第一次发布 = max(根环 4, 记录 5) + 1"
    );
    assert_eq!(mounted.output.row_publish.record.counter, 6);
    pool.devices = Some(devices);
    let reopened = pool.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), mounted.current.root.checkpoint_txg),
            content: third_content()
        },
        "施加了那条记录之后照抄的文件角色就是第三次的内容"
    );
}

/// I-3.8（实例表行唯一且低于挂载根）的坏镜像：拿真写者再发一版，实例表里多写一行 (2, 7, 0)——行的实例代号等于挂载根的实例，
/// 别的都合法；池级 checker 只有 I-3.8 判红。
#[test]
fn checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root() {
    let (mut pool, _second, mut mounted) =
        build_publish_second_version_and_remount("step-three-bad-row");
    let table =
        InstanceTableRecords::parse(&mounted.current.unit(TransactionUnit::InstanceTable).bytes)
            .expect("实例表解得开");
    let mut bad_table = table.clone();
    bad_table.rows.push(InstanceRow {
        instance: InstanceGeneration(2),
        selected_root_txg: CheckpointTxg(7),
        applied_transaction_high_water: 0,
        is_rollback: false,
    });
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let current = mounted.current.clone();
    publish_version(
        &mut writer,
        &mut mounted.allocator,
        PublishPlan {
            txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
            counter: current.record.counter + 1,
            transaction: 0,
            instance: InstanceGeneration(2),
            back_chain: back_chain_of(&current.record_bytes),
            file: None,
            instance_table: InstanceTablePlan::Rewrite(bad_table.to_records()),
            tree_birth_txg: current.tree_birth_txg(),
            tree_identifier_watermark: current.root.tree_identifier_watermark,
            rollback_floor: current.root.rollback_floor,
        },
        Some(&current),
    )
    .expect("写者不拦这一行：拦它的是 checker");
    let verdicts = check_pool_image(&pool.memory_pool());
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(violated, ["I-3.8"], "只有 I-3.8 红：{verdicts:?}");
}

/// 前缀链首接在所选根自己那条记录之后（D23 已定项 14 第 1 条「链从所选根覆盖的最后一条记录之后接」）：同一实例里再发两版（txg 5、6），
/// 把两个根槽改坏、再把 jsn 5 的记录两份都改坏——所选根 (1, 4) 自己那条 jsn 4 读得出，链首该是 jsn 5，jsn 5 读不出 ⇒ 断号即止，
/// jsn 6 一条都不施加，读回第二次的内容；从「水位之上最小的那条」接的写法会施加 jsn 6、读出第四版。
#[test]
fn one_missing_record_right_after_the_chosen_root_stops_the_prefix_even_when_later_records_are_intact(
) {
    let mut pool = build_pool("step-three-chain-start");
    let second = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    pool.output = second;
    let third = overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(1));
    pool.output = third;
    let fourth = overwrite_in_process(&mut pool, &content_of(3300, 17), InstanceGeneration(1));
    assert_eq!(
        (fourth.root.checkpoint_txg, fourth.record.counter),
        (CheckpointTxg(6), 6)
    );
    let mut image = pool.memory_pool();
    for txg in [5u64, 6] {
        let target = target_for_publish(CheckpointTxg(txg));
        image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
    }
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        image.flip_byte(device, record_offset(5, JOURNAL_RING_DEFAULT_BYTES), 300);
    }
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(4)),
            content: second_content()
        },
        "jsn 5 读不出、jsn 6 不许接上：{:?}",
        report.journal
    );
    assert_eq!(report.journal.valid_records, 5, "jsn 1–4、6");
    assert_eq!(report.journal.prefix_applied, 0);
}

/// 三方第一轮攻方腿打中的一格：所选根自己那条记录读不出时链首没有锚点，不许把水位之上最小的那条无条件接上——同一实例再发三版
/// （txg 4、5、6），改坏 txg 5、6 的根槽让所选根退到 (1, 4)，再改坏 jsn 4 与 jsn 5 两份镜像：jsn 6 的 txg 是 6、不是 4 + 1 ⇒ 一条都不施加、
/// 读回第二版；只改坏 jsn 4 时 jsn 5 的 txg 正是 5 ⇒ 接上、再顺着 jsn 6 施加到第四版。写行随之：前一种写 (1, 4, 0)，后一种 (1, 6, 4)。
#[test]
fn torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg() {
    for (
        tear_jsn_five,
        expected_effective_root,
        expected_content,
        expected_applied,
        expected_row,
    ) in [
        (
            true,
            CheckpointTxg(4),
            second_content(),
            0,
            (CheckpointTxg(4), 0),
        ),
        (
            false,
            CheckpointTxg(6),
            content_of(3300, 17),
            2,
            (CheckpointTxg(6), 4),
        ),
    ] {
        let mut pool = build_pool("step-three-torn-anchor");
        pool.output = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
        pool.output = overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(1));
        let fourth = overwrite_in_process(&mut pool, &content_of(3300, 17), InstanceGeneration(1));
        assert_eq!(fourth.record.counter, 6);
        let mut image = pool.memory_pool();
        for txg in [5u64, 6] {
            let target = target_for_publish(CheckpointTxg(txg));
            image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
        }
        let torn: Vec<u64> = if tear_jsn_five { vec![4, 5] } else { vec![4] };
        for jsn in &torn {
            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                image.flip_byte(device, record_offset(*jsn, JOURNAL_RING_DEFAULT_BYTES), 300);
            }
        }
        let report = recover(&image, JournalPolicy::Consult);
        assert_eq!(
            (report.outcome, report.effective_root),
            (
                RecoveryOutcome::FileRead {
                    root: (InstanceGeneration(1), CheckpointTxg(4)),
                    content: expected_content
                },
                Some((InstanceGeneration(1), expected_effective_root))
            ),
            "撕掉 {torn:?}（所选根 (1, 4)）：{:?}",
            report.journal
        );
        assert_eq!(report.journal.prefix_applied, expected_applied);
        // 同一段历史做可写挂载：写出的行 W 只罩住真被施加的前缀。
        let mut devices = pool.reopen_recorded();
        for txg in [5u64, 6] {
            let target = target_for_publish(CheckpointTxg(txg));
            let device = region_device(txg);
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
        for jsn in &torn {
            for (_, recorded) in devices.iter_mut() {
                let offset = record_offset(*jsn, JOURNAL_RING_DEFAULT_BYTES);
                let mut bytes = vec![0u8; 4096];
                recorded.read_at(offset, &mut bytes).expect("读记录");
                bytes[300] ^= 0xff;
                recorded
                    .write_at(offset, &bytes, WriteDurability::Plain)
                    .expect("改坏记录");
            }
        }
        let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
        pool.devices = Some(devices);
        assert_eq!(
            mounted
                .output
                .rows_written
                .iter()
                .map(|row| (row.selected_root_txg, row.applied_transaction_high_water))
                .collect::<Vec<_>>(),
            vec![expected_row],
            "撕掉 {torn:?} 之后写的行"
        );
    }
}
```

## 二、新文件 `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs` 全文

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
        (InstanceGeneration(3), CheckpointTxg(9), 4, 0, 0),
        "D：txg = max(根环 8, 记录 8) + 1；jsn 接在 A 的 jsn 3 之后；本实例第一条反向链 0"
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
    assert_eq!(warm_up.record.counter, 5);

    // 影子账：被抛弃的根 B、(2, 5)、(2, 6)、(2, 7)、C 引用而 A 不引用的槽——B 10、写行 6、暖机 4 + 4、C 10 = 34 个槽，逐盘。
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let abandoned: BTreeSet<u64> = slots_of(&third, device)
            .difference(&slots_of(rollback_publish, device))
            .copied()
            .collect();
        assert_eq!(abandoned.len(), 34, "盘 {device:?} 上只被被抛弃根引用的槽");
        assert!(
            output.isolated_slots_per_device.contains(&(device, 34)),
            "被抛弃根独占量 {:?}",
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
        report.journal.valid_records, 8,
        "jsn 1–3 实例 1、4–5 实例 3（盖掉了 B 与写行那两条）、6–8 实例 2"
    );
    assert_eq!(
        report.journal.prefix_applied, 0,
        "(3, 10) 之后 jsn 6 是实例 2 的，不许接"
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
            34
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
```

## 二、新文件 `crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs` 全文

```rust
//! 里程碑「第二个事务」步 5 的验收：回退之后再覆盖写四次（txg 11–14；第一次把 A 的八个单元释放、释放代 11），抬回退下界 F 到 11
//! （上限 = min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根) = min(13, 11)；两次空发布 txg 15、16 让两块盘各有一条带 F = 11 的根），
//! 释放代 ≤ 11 的落点回收、之后的仍在 defer 队列里；发布 E（txg 17）把数据单元落回最低的可再分配偶数槽对 50176（mkfs 实例表那片，D 放掉的）；冷启动读回 E；checker 全绿。
//! 必红：不抬 F 就回收（复用窗口置 0），A 的根还在候选集里、它的数据单元被 E 盖掉，checker 在 A 的根上判 I-2.1 红。

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
        SlotNumber(50176),
        "E 的数据单元落回最低的可再分配偶数槽对：mkfs 那片实例表单元的落点（D 放掉的，释放代 9）；A 的数据单元 50180 排在它后面"
    );
    let reused_record = pool
        .allocator
        .record_for(DeviceIdentity(0), SlotNumber(50176))
        .expect("50176 的记录");
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
            .filter(|record| record.device == DeviceIdentity(0) && record.slot == SlotNumber(50176))
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

/// 必红（C22（刚释放的块立即重分配）、复用窗口置 0）：不抬 F、直接把释放代 ≤ 11 的落点回收，E 落回 50180；A 的根 (1, 3) 还在候选集里
/// （F = 0），checker 走它时第一个数据单元的校验和与位置条目对不上 ⇒ I-2.1 红。
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
        SlotNumber(50176),
        "mkfs 实例表的落点先被拿走"
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
```

## 三、`crates/mutations.tsv` 全文

```tsv
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
步 4：回退之后 jsn 接可读链末尾而不是 R_old 那条之后（C340 的 P2）	crates/singlefs-core/src/mount.rs	    let next_counter = own_record.counter + 1;	    let next_counter = records.values().map(|record| record.counter).max().unwrap_or(0) + 1;	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
步 4：影子账一个槽都不隔离	crates/singlefs-core/src/mount.rs	    if shadow_ledger == ShadowLedger::On {	    if shadow_ledger == ShadowLedger::Off {	-p singlefs-harness --test second_transaction_step_four_rollback -- without_the_shadow_ledger	without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit
步 4：回退候选集不按实例表判（被抛弃时间线的根也能退到）	crates/singlefs-core/src/mount.rs	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg)	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg && false)	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_onto_an_abandoned	rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused
步 4：前缀第五条不判（回退行的 W 不封顶）	crates/singlefs-core/src/recovery.rs	            if high_water == 0 || record.transaction > high_water {	            if false && (high_water == 0 || record.transaction > high_water) {	-p singlefs-harness --test second_transaction_step_four_rollback -- the_rollback_row_caps	the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water
步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根	crates/singlefs-checker/src/walk.rs	        if index != newest_index && !abandoned && !below_floor {	        if index != newest_index && !below_floor {	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
步 4：记录核对器把被后来记录合法覆盖的记录槽当成记录流有洞	crates/singlefs-harness/src/crash.rs	        if in_place(publish.root) && publish.records.iter().all(|record| record_lost(*record)) {	        if in_place(publish.root) && !publish.records.iter().any(|record| in_place(*record)) {	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
步 5：回收不看释放代（复用窗口置 0）	crates/singlefs-core/src/allocator.rs	            .filter(|record| record.is_released && record.generation <= floor)	            .filter(|record| record.is_released)	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
步 5：抬 F 的上限不看第 4 新的非空根	crates/singlefs-core/src/mount.rs	    Some(newest_on_every_device.min(fourth_newest))	    Some(newest_on_every_device.max(fourth_newest))	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_above	raising_the_floor_above_the_fourth_newest_non_empty_root_is_refused
步 5：抬 F 的空发布只推一次（F 只落在一块盘上）	crates/singlefs-core/src/mount.rs	        && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(publishes.len()).expect("次数") < 1	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：复用时追加记录而不改写（同盘同槽两条）	crates/singlefs-core/src/allocator.rs	            if self.reclaimed.remove(&key) {	            if self.reclaimed.remove(&key) && false {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：checker 的候选集不按 F 收（F 之下的根照走）	crates/singlefs-checker/src/walk.rs	        if index != newest_index && !abandoned && !below_floor {	        if index != newest_index && !abandoned {	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
步 5：F_生效 取各盘 F 最大值的最大值而不是最小值	crates/singlefs-core/src/recovery.rs	    highest_per_device\n        .values()\n        .copied()\n        .min()\n        .unwrap_or(CheckpointTxg(0))	    highest_per_device\n        .values()\n        .copied()\n        .max()\n        .unwrap_or(CheckpointTxg(0))	-p singlefs-harness --test second_transaction_step_five_reuse -- one_device_carrying_the_floor_alone	one_device_carrying_the_floor_alone_does_not_take_effect_on_remount
步 3 三方第一轮：链首没锚点时不看 txg、无条件接上水位之上最小的一条	crates/singlefs-core/src/recovery.rs	        } else if record.checkpoint_txg != chain_start_txg_without_anchor {	        } else if false && record.checkpoint_txg != chain_start_txg_without_anchor {	-p singlefs-harness --test second_transaction_step_three_second_instance -- torn_anchor_record	torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg
```
