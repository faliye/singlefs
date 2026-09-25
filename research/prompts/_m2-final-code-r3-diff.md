# 附录二：第二轮冻结树到第三轮冻结树的 crates 改动，加第二轮之后打进的定义改动 diff（生成于 2026-09-25 08:32 JST / 2026-09-24 23:32 UTC）

基准：第二轮冻结副本 `/tmp/claude-1000/m2-final-code-r2/tree/crates/` 到本轮冻结副本 `/tmp/claude-1000/m2-final-code-r3/tree/crates/`（腿读代码一律读本轮冻结副本，不读主工作区——主工作区在腿跑着的时候还会被实二五继续改 `transaction.rs`、`mount.rs`、`allocator.rs`、`recovery.rs`、`rollback_witness.rs`、`walk.rs`）。

diff 原始文件（两份，均由主 agent 给出，材料员未重新生成、原样落盘）：
- `/tmp/claude-1000/m2-final-code-r3/r2-to-r3.diff`（`diff -ruN -x target` 生成，51 个文件：`crates/mutations.tsv` 1 份、`src/*.rs` 18 份、`tests/*.rs` 32 份，共 16766 行）；
- `/tmp/claude-1000/m2-final-code-r3/defs-r2-to-r3.diff`（`git diff`/`diff -u` 逐文件生成，4 个文件，共 44 行）。

太长的一半按派发提示的口径截断：`tests/` 下的 32 份只列文件名与增删行数，不抄全文；`crates/mutations.tsv` 与 `src/*.rs` 共 19 份的改动整段抄进第一节。第一节用 `diff -ruN -x target` 原有的按文件分段格式，每段前一行 `diff -ruN -x target <相对路径>`（省去两侧绝对路径与 `-x target` 之外的信息，只留材料员从原始文件里逐段切出、未改一字的差异本体）。

## 一、diff（`crates/mutations.tsv` 与 `src/*.rs`，共 19 个文件，第二轮冻结树到本轮冻结树，原样节选自 `r2-to-r3.diff`）

```diff
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/mutations.tsv tree/crates/mutations.tsv
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/mutations.tsv	2026-09-24 18:48:58.254848952 +0000
+++ tree/crates/mutations.tsv	2026-09-24 22:40:06.357335347 +0000
@@ -7,7 +7,7 @@
 同盘槽号不核唯一	crates/singlefs-core/src/recovery.rs	        if !slots_per_device	        if false && !slots_per_device	-p singlefs-core --lib -- allocation_records_per_device	two_records_for_the_same_slot_on_the_same_device_are_rejected
 空闲不随分配减（I-5.2 要红）	crates/singlefs-core/src/allocator.rs	        self.free_slots -= span;	        // 变异：空闲不减	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
 释放时清位图（立即复用）	crates/singlefs-core/src/allocator.rs	        self.deferred_slots += span;\n    }	        self.deferred_slots += span;\n        for index in start..end {\n            self.allocated[index] = false;\n        }\n    }	-p singlefs-harness --test second_transaction_step_one_overwrite -- released_placements	released_placements_are_not_handed_out_again_before_reclaim_exists
-分配记录树装不下不判	crates/singlefs-core/src/transaction.rs	    if records_after_this_publish > allocation_node_capacity {	    if false && records_after_this_publish > allocation_node_capacity {	-p singlefs-harness --test second_transaction_step_one_overwrite -- repeated_overwrites	repeated_overwrites_report_a_full_allocation_node_instead_of_panicking
+K1：分配记录树按槽号找叶时叶宽算成两倍（记录进了不罩它的叶，按位置寻址的核判出）	crates/singlefs-core/src/allocation_record_tree.rs	            index_in_device: slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS,	            index_in_device: slot.0 / (2 * ALLOCATION_RECORD_TREE_LEAF_SLOTS),	-p singlefs-harness --test second_transaction_step_one_overwrite -- repeated_overwrites_go_past	repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree
 映射查到 key 就算（落点指错不核记录）	crates/singlefs-core/src/transaction.rs	        let record =\n            allocator\n                .record_for(device, slot)\n                .ok_or(PublishError::ReleaseTargetNotAllocated {	        let record = allocator\n            .record_for(device, slot)\n            .or(allocator.records().first())\n            .ok_or(PublishError::ReleaseTargetNotAllocated {	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_reports_a_mapping_entry_whose_slot	release_reports_a_mapping_entry_whose_slot_has_no_record_or_the_wrong_span_instead_of_panicking
 失败的发布不退回分配器	crates/singlefs-core/src/transaction.rs	        Err(refusal) => {\n            *allocator = allocator_before_this_publish;\n            return Err(refusal);\n        }\n    };\n    // 落盘那几步里失败：分配器换回去、这次发布冻结在它上面等原样重发（`FrozenPublish`）。\n	        Err(refusal) => {\n            return Err(refusal);\n        }\n    };\n    // 落盘那几步里失败：分配器换回去、这次发布冻结在它上面等原样重发（`FrozenPublish`）。\n	-p singlefs-harness --test second_transaction_step_one_overwrite -- publish_running_out_of_space_midway	publish_running_out_of_space_midway_leaves_the_allocator_as_it_was
 内容超长不判	crates/singlefs-core/src/transaction.rs	        if file.content.len() > data_unit_capacity {	        if false && file.content.len() > data_unit_capacity {	-p singlefs-harness --test second_transaction_step_one_overwrite -- content_larger_than_a_data_unit	content_larger_than_a_data_unit_payload_is_refused_before_anything_is_touched
@@ -18,7 +18,7 @@
 步 1 变异：事务号不加一	crates/singlefs-core/src/transaction.rs	            transaction: previous.highest_transaction_number_in_this_instance + 1,	            transaction: previous.highest_transaction_number_in_this_instance,	-p singlefs-harness --test second_transaction_step_one_overwrite -- overwrite_publishes	overwrite_publishes_the_second_version_through_the_same_commit_shape
 步 1 变异：改动计数留 1	crates/singlefs-core/src/transaction.rs	            change_count: txg.0,	            change_count: 1,	-p singlefs-harness --test second_transaction_step_one_overwrite -- overwrite_publishes	overwrite_publishes_the_second_version_through_the_same_commit_shape
 步 2 变异：忘了改写分配记录	crates/singlefs-core/src/allocator.rs	            record.is_released = true;	            record.is_released = false;	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
-步 3：不写行（实例表照抄）	crates/singlefs-core/src/mount.rs	            instance_table: InstanceTablePlan::Rewrite(instance_table),	            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
+步 3：不写行（实例表照抄）	crates/singlefs-core/src/mount.rs	        instance_table: InstanceTablePlan::Rewrite(instance_table),	        instance_table: InstanceTablePlan::Carry(previous.root.instance_table),	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
 步 3：暖机只推一次	crates/singlefs-core/src/mount.rs	        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(warm_up_publishes.len()).expect("次数") < 1	-p singlefs-harness --test second_transaction_step_three_second_instance -- damaging_every_instance_two_root	damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device
 步 3：前缀跨实例边界（把别的实例的记录也接上）	crates/singlefs-core/src/recovery.rs	            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water	            (record.instance, record.checkpoint_txg) > water	-p singlefs-harness --test first_transaction_step_seven_layer0 -- layer0_partial_enumeration	layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape
 步 3：链首锚点错一位（接在所选根自己那条记录之后第二条）	crates/singlefs-core/src/recovery.rs	        root_own_record_counter.map(|counter| (root.instance, counter + 1));	        root_own_record_counter.map(|counter| (root.instance, counter + 2));	-p singlefs-harness --test second_transaction_step_three_second_instance -- stray_record_of_the_previous_instance	stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row
@@ -58,7 +58,7 @@
 步 5：「非空」的前一条取任意可读根（不排除被抛弃的根与 F 之下的根）	crates/singlefs-core/src/mount.rs	    let previous_candidates: &[RootRecord] = &valid;	    let previous_candidates: &[RootRecord] = &readable;	-p singlefs-harness --test second_transaction_step_five_reuse -- the_rollback_publish_is_compared	the_rollback_publish_is_compared_with_the_previous_valid_root_and_not_with_the_abandoned_root_before_it
 步 5：「非空」只比 inode 树的根指针、不比 extent 树的	crates/singlefs-core/src/mount.rs	    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree\n        || root_pointers.extent_tree != previous_valid_root_pointers.extent_tree	    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree	-p singlefs-core --lib -- root_is_non_empty_when_either	root_is_non_empty_when_either_user_visible_tree_pointer_differs_from_the_previous_valid_root
 步 3：只做过 mkfs 的池重开后分配器不认 mkfs 写在单元区里的两个单元	crates/singlefs-core/src/mount.rs	    allocator.mark_format_time_units(instance_table_placement, tree_table_placement);	    let _ = (instance_table_placement, tree_table_placement);	-p singlefs-harness --test second_transaction_step_three_formatted_pool	writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold
-步 3：只做过 mkfs 的池上零单元暖机的反向链写 0	crates/singlefs-core/src/mount.rs	                back_chain: back_chain_of(&current_version_without_file.record_bytes),	                back_chain: 0,	-p singlefs-harness --test second_transaction_step_three_formatted_pool	writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold
+步 3：只做过 mkfs 的池上零单元暖机的反向链写 0	crates/singlefs-core/src/mount.rs	        back_chain: back_chain_of(&current_version_without_file.record_bytes),	        back_chain: 0,	-p singlefs-harness --test second_transaction_step_three_formatted_pool	writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold
 步 3：记录与根之间少一道屏障（实二二三起三条发布路径共用 persist_publish_writes，零单元发布那一段也少了这一道）	crates/singlefs-core/src/transaction.rs	    writer.perform(CommitStep::Barrier)?;\n    persist_the_root_then_rotate_the_system_configuration(\n        writer,\n        writes.checkpoint_txg,\n	    persist_the_root_then_rotate_the_system_configuration(\n        writer,\n        writes.checkpoint_txg,\n	-p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- the_formatted_pool_mount_and_first_file_stream	the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence
 步 3：只做过 mkfs 的池重开后把 mkfs 实例表按 1 槽记	crates/singlefs-core/src/mount.rs	        placement_of(&root.instance_table, TransactionUnit::InstanceTable)?;	        placement_of(&root.instance_table, TransactionUnit::TreeTable)?;	-p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- every_crash_state_outside_the_unit_segment_of_the_formatted_pool	every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims
 步 3：写行时这次要写的行没接进重写出去的那条实例表链（带文件的一版与树表 0 条的一版共用这一处拼法）	crates/singlefs-core/src/mount.rs	    rows.extend_from_slice(rows_written);	    rows.extend_from_slice(&[]);	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_formatted_pool_mounted_twice	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
@@ -67,7 +67,7 @@
 第一个事务 步 1：mkfs 不核根环区域归属是不是第一版写死的 0 / 1 / 0	crates/singlefs-core/src/make_filesystem.rs	    if devices.len() == 2 && parameters.region_devices != FIRST_VERSION_REGION_DEVICES {	    if false {	-p singlefs-harness --test first_transaction_step_one_mkfs -- region_layout_other_than	region_layout_other_than_zero_one_zero_is_refused_before_any_write
 步 3：第一个文件版本的 txg 写死 3，不从它要建在上面的那一版接着算（2026-09-23 用户定案：`FIRST_TRANSACTION_TXG` 只管 mkfs 那条流）	crates/singlefs-core/src/transaction.rs	    let first_file_version_txg = CheckpointTxg(version_to_build_on.checkpoint_txg.0 + 1);	    let first_file_version_txg = CheckpointTxg(3);	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_formatted_pool_mounted_twice	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
 步 3：publish_first_file 不核要建在上面的那条根与上一条记录说的是不是同一版	crates/singlefs-core/src/transaction.rs	    if !follows_directly {	    if false {	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- first_file_version_whose_root_and_previous_record_disagree	first_file_version_whose_root_and_previous_record_disagree_is_refused_before_any_write
-步 3：空池挂载的零单元暖机把回退下界写成 1（流与第一个事务那条不再逐项相同）	crates/singlefs-core/src/mount.rs	                rollback_floor: current_version_without_file.root.rollback_floor,	                rollback_floor: CheckpointTxg(1),	-p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- formatted_pool_mount_stream_has_the_same	formatted_pool_mount_stream_has_the_same_base_writes_and_segments_as_the_first_transaction_stream
+步 3：空池挂载的零单元暖机把回退下界写成 1（流与第一个事务那条不再逐项相同）	crates/singlefs-core/src/mount.rs	        rollback_floor: current_version_without_file.root.rollback_floor,	        rollback_floor: CheckpointTxg(1),	-p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- formatted_pool_mount_stream_has_the_same	formatted_pool_mount_stream_has_the_same_base_writes_and_segments_as_the_first_transaction_stream
 增补 1：发布路径漏计数据单元的写（第一个事务按种类的合计对不上录制器在设备一层记下的写）	crates/singlefs-core/src/transaction.rs	                    self.writes_by_structure_kind.count_write_call(kind, unit);	                    if identity != TransactionUnit::Data { self.writes_by_structure_kind.count_write_call(kind, unit); }	-p singlefs-harness --test second_transaction_supplement_one_write_accounting -- first_transaction_and_overwrite_writes_by_kind	first_transaction_and_overwrite_writes_by_kind_add_up_to_the_recorded_writes_and_the_byte_table
 增补 1：发布路径漏计实例表单元的写（写行发布按种类的合计对不上录制器）	crates/singlefs-core/src/transaction.rs	                    self.writes_by_structure_kind.count_write_call(kind, unit);	                    if identity != TransactionUnit::InstanceTable { self.writes_by_structure_kind.count_write_call(kind, unit); }	-p singlefs-harness --test second_transaction_supplement_one_write_accounting -- writable_remount_row_publish	writable_remount_row_publish_and_each_warm_up_publish_add_up_to_the_recorded_writes
 增补 1：写入口漏计系统配置槽的写（暖机按种类的合计对不上录制器）	crates/singlefs-core/src/transaction.rs	        self.writes_by_structure_kind\n            .count_write_call(WrittenStructureKind::SystemConfigurationSlot, &slot_bytes);	        drop(slot_bytes);	-p singlefs-harness --test second_transaction_supplement_one_write_accounting -- first_transaction_and_overwrite_writes_by_kind	first_transaction_and_overwrite_writes_by_kind_add_up_to_the_recorded_writes_and_the_byte_table
@@ -99,8 +99,8 @@
 增补 2 第 20b 行：中途失败的发布不把这次已记的写交出去（与设备一层记的合计对不上）	crates/singlefs-core/src/transaction.rs	    if let Err(cause) = persist_publish_writes(pool, &writes) {\n        pool.count_failed_publish(&writes_before_this_publish);\n        let allocator_after_the_publish =\n	    if let Err(cause) = persist_publish_writes(pool, &writes) {\n        let allocator_after_the_publish =\n	-p singlefs-harness --test second_transaction_supplement_one_write_accounting -- publish_that_fails_midway	publish_that_fails_midway_hands_out_what_it_already_wrote_so_the_retry_still_adds_up
 增补 2 第 20c 行：用户数据只排除当前那一个开放段（回落把开放段置空之后，那一段对用户数据开放）	crates/singlefs-core/src/allocator.rs	        let cluster_segments = &self.cluster_segments;	        let cluster_segments = &self.open_segment.into_iter().collect::<BTreeSet<SlotNumber>>();	-p singlefs-core --lib -- allocator::tests::user_data_does_not_land	user_data_does_not_land_in_a_cluster_segment_that_the_fallback_closed
 收口表第 22 行 C22（刚释放的块立即重分配）：候选根那一格不判 I-4.8（复用窗口置 0 之后 I-4.8 不红）	crates/singlefs-checker/src/walk.rs	                .judge("I-4.8", !walked_into_reused_or_erased_unit, || {	                .judge("I-4.8", true, || {	-p singlefs-harness --test second_transaction_step_five_reuse -- reclaiming_without_raising_the_floor	reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red
-增补 2 第 20a 行：取号之前只算写行那一次，暖机那几次空发布不算（写行发完、暖机才被拒，实例代号已经烧掉）	crates/singlefs-core/src/mount.rs	        let shapes: Vec<PublishShape> =\n            std::iter::once(PublishShape::row_publish_rewriting_instance_table_pages(\n                instance_table_rewrite.pages_after_this_publish(),\n            ))\n            .chain(std::iter::repeat_n(\n                PublishShape::EMPTY_PUBLISH,\n                warm_up_publishes_planned,\n            ))\n            .collect();	        let shapes: Vec<PublishShape> = vec![PublishShape::row_publish_rewriting_instance_table_pages(\n            instance_table_rewrite.pages_after_this_publish(),\n        )];	-p singlefs-harness --test second_transaction_supplement_two_row_publish_admission -- writable_mount_whose_first_warm_up	writable_mount_whose_first_warm_up_publish_does_not_fit_is_refused_before_the_instance_is_acquired
-增补 2 第 20a 行：取号之前只把第一次暖机算进去（写行装得下、暖机第 2 次装不下的池照样放行）	crates/singlefs-core/src/mount.rs	                PublishShape::EMPTY_PUBLISH,\n                warm_up_publishes_planned,	                PublishShape::EMPTY_PUBLISH,\n                warm_up_publishes_planned.min(1),	-p singlefs-harness --test second_transaction_supplement_two_row_publish_admission -- writable_mount_whose_second_warm_up	writable_mount_whose_second_warm_up_publish_does_not_fit_is_refused_before_the_instance_is_acquired
+增补 2 第 20a 行：取号之前的预演只演写行那一次、暖机那几次不演（预演与真发取到的落点对不上，挂载自己的断言判出）	crates/singlefs-core/src/mount.rs	    for (warm_up_position, planned_txg) in warm_up_publish_txgs.iter().enumerate() {	    for (warm_up_position, planned_txg) in warm_up_publish_txgs.iter().take(0).enumerate() {	-p singlefs-harness --test second_transaction_supplement_two_row_publish_admission -- writable_mount_on_the_pool_whose_row_publish	writable_mount_on_the_pool_whose_row_publish_used_to_be_refused_before_acquisition_succeeds
+增补 2 第 20a 行：取号之前的预演只演第一次暖机（计划推两次暖机的池上预演与真发对不上，挂载自己的断言判出）	crates/singlefs-core/src/mount.rs	    for (warm_up_position, planned_txg) in warm_up_publish_txgs.iter().enumerate() {	    for (warm_up_position, planned_txg) in warm_up_publish_txgs.iter().take(1).enumerate() {	-p singlefs-harness --test second_transaction_supplement_two_row_publish_admission -- writable_mount_on_the_pool_whose_second_warm_up	writable_mount_on_the_pool_whose_second_warm_up_used_to_be_refused_before_acquisition_succeeds
 增补 2 第 11 行：第一个事务的改动计数写回 1（字段表定义是最后一次改动所在发布的 checkpoint_txg = 3）	crates/singlefs-core/src/transaction.rs	                change_count: first_file_version_txg.0,	                change_count: 1,	-p singlefs-harness --test first_transaction_step_five_publish -- inode_and_extent_lookups	inode_and_extent_lookups_from_the_root_read_the_first_file_back
 C374 I-3.9：释放代的区间判据整条拿掉（判定恒真）	crates/singlefs-checker/src/walk.rs	        let in_the_witnessed_interval = record.generation > last_referencing_txg\n            && first_root_without_it.is_some_and(|txg| record.generation <= txg);	        let in_the_witnessed_interval = true;	-p singlefs-harness --test checker_known_bad_images -- each_c374_bad_image	each_c374_bad_image_reddens_only_its_own_invariant_after_the_overwrite
 C374 I-9.14：树表条目诞生 txg 跨根相等的判定恒真	crates/singlefs-checker/src/walk.rs	        let first_birth_txg = sightings[0].birth_txg;\n        let birth_txg_is_the_same_across_roots = sightings\n            .iter()\n            .all(|sighting| sighting.birth_txg == first_birth_txg);	        let birth_txg_is_the_same_across_roots = true;	-p singlefs-harness --test checker_known_bad_images -- each_c374_bad_image	each_c374_bad_image_reddens_only_its_own_invariant_after_the_overwrite
@@ -140,11 +140,11 @@
 增补 3 第 2 件（理想模型自身）：F_生效取各盘最大 F 的最大（只一块盘带新 F 也生效）	crates/singlefs-harness/src/model.rs	        highest_floor_per_device\n            .values()\n            .copied()\n            .min()	        highest_floor_per_device\n            .values()\n            .copied()\n            .max()	-p singlefs-harness --lib -- model::tests::raised_floor_carried	raised_floor_carried_by_one_device_only_does_not_take_effect
 增补 3 第 2 件（理想模型自身）：分配代只要不小于写它的那次发布就算对	crates/singlefs-harness/src/model.rs	&& record.generation == *written_at	&& record.generation >= *written_at	-p singlefs-harness --lib -- model::tests::carried_unit_keeps	carried_unit_keeps_the_generation_of_the_publish_that_wrote_it
 增补 3 第 2 件（理想模型自身，D13 已定项 5）：模型模块 use 了 singlefs_core	crates/singlefs-harness/src/model.rs	use std::rc::Rc;	use std::rc::Rc;\nuse singlefs_core as _;	-p singlefs-harness --lib -- model_comparison::tests::the_model_module_uses_only	the_model_module_uses_only_the_standard_library_and_the_format_constants
-增补 3 第 2 件（代码三方第一轮判决第三节第 1 条，攻方变异 W1）：分配记录墙差一，812 条正好装满也拒（> 写成 >=）；逼近墙的取样点判出	crates/singlefs-core/src/transaction.rs	    if records_after_this_publish > allocation_node_capacity {	    if records_after_this_publish >= allocation_node_capacity {	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_wall_sampling	allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count
-增补 3 第 2 件（代码三方第一轮判决第三节第 1 条，攻方变异 W1 同一处）：分配记录墙差一；812 条那一格的边沿用例判出	crates/singlefs-core/src/transaction.rs	    if records_after_this_publish > allocation_node_capacity {	    if records_after_this_publish >= allocation_node_capacity {	-p singlefs-harness --test second_transaction_supplement_three_random_history -- an_overwrite_that_fills_the_allocation_node	an_overwrite_that_fills_the_allocation_node_to_exactly_812_records_succeeds_and_the_next_is_refused
-增补 3 第 2 件（代码三方第一轮判决第三节第 1 条的改法）：模型的分配记录墙只看沿来路的上界、不看镜像上的真条数	crates/singlefs-harness/src/model.rs	                upper_bound_exceeds && true_count_exceeds	                upper_bound_exceeds && (true_count_exceeds || true)	-p singlefs-harness --lib -- model::tests::the_allocation_record_wall_is_permitted	the_allocation_record_wall_is_permitted_only_when_the_counted_records_plus_this_publish_exceed_812
-增补 3 第 2 件（代码三方第一轮判决第三节第 1 条的改法）：抬 F 做完几次之后被墙拒，基数仍数这一步起点那一版	crates/singlefs-harness/src/model.rs	        if self.walls_are_judged_before_the_first_write || publishes_completed == 0 {	        if self.walls_are_judged_before_the_first_write || publishes_completed < usize::MAX {	-p singlefs-harness --lib -- model::tests::mount_and_raise_count	mount_and_raise_count_the_wall_from_the_version_their_admission_starts_from
-增补 3 第 2 件（代码三方第一轮判决第三节第 1 条的改法）：checker 数一条根下的分配记录只数一半	crates/singlefs-checker/src/walk.rs	        records += node.entries.len();	        records += node.entries.len() / 2;	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_records_counted_on_the_image	allocation_records_counted_on_the_image_are_one_per_unit_per_device_and_zero_without_a_file
+K1：分配记录树根之下的节点头里 key 区间的上端写成起点（按位置规定罩的那一段写错；越过原分配记录墙的取样点照跑 checker 判出）	crates/singlefs-core/src/allocation_record_tree.rs	                    allocation_record_key_bytes(position.device, last_slot),	                    allocation_record_key_bytes(position.device, first_slot),	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_sampling	allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall
+K1（同一处）：分配记录树根之下的节点头里 key 区间的上端写成起点；越过 812 条那一段写死的历史每一步跑 checker 判出	crates/singlefs-core/src/allocation_record_tree.rs	                    allocation_record_key_bytes(position.device, last_slot),	                    allocation_record_key_bytes(position.device, first_slot),	-p singlefs-harness --test second_transaction_supplement_three_random_history -- overwrites_raising_the_floor_and_rolling_back_past_812	overwrites_raising_the_floor_and_rolling_back_past_812_allocation_records_all_succeed
+K1 × 模型：分配记录树一次至多重写几个节点的上界少数了每层末端那一个节点	crates/singlefs-harness/src/model.rs	                        (reach - 1) / span - UNIT_AREA_START_SLOT / span + 1	                        (reach - 1) / span - UNIT_AREA_START_SLOT / span	-p singlefs-harness --lib -- model::tests::allocation_record_tree_nodes_rewritten	allocation_record_tree_nodes_rewritten_by_the_first_file_version_on_4_gib_are_at_most_nine
+K1 × 模型：分配记录树一次至多重写几个节点的上界不往下迭代（停在单元区里全部节点，占槽上界放得太宽）	crates/singlefs-harness/src/model.rs	            if next >= bound {	            if true {	-p singlefs-harness --lib -- model::tests::allocation_record_tree_nodes_rewritten	allocation_record_tree_nodes_rewritten_by_the_first_file_version_on_4_gib_are_at_most_nine
+增补 3 第 2 件（代码三方第一轮判决第三节第 1 条的改法）：checker 数一条根下的分配记录只数一半	crates/singlefs-checker/src/walk.rs	        records += records_of_the_tree.len();	        records += records_of_the_tree.len() / 2;	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_records_counted_on_the_image	allocation_records_counted_on_the_image_are_one_per_unit_per_device_and_zero_without_a_file
 增补 3 第 2 件（代码三方第一轮判决第三节第 2 条，攻方变异 R1）：被抛弃时间线的根报成低于 F（只换判别字段）	crates/singlefs-core/src/mount.rs	            exclusion: RollbackCandidateExclusion::OnAbandonedTimeline,	            exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_onto_an_abandoned_root_above_the_floor	rolling_back_onto_an_abandoned_root_above_the_floor_is_refused_for_being_abandoned
 增补 3 第 2 件（代码三方第一轮判决第三节第 2 条的改法）：胶水把回退目标「被抛弃」映射成「低于 F」	crates/singlefs-harness/src/model_comparison.rs	        RollbackCandidateExclusion::OnAbandonedTimeline => {\n            ModelRefusalReason::RollbackTargetOnAbandonedTimeline\n        }	        RollbackCandidateExclusion::OnAbandonedTimeline => {\n            ModelRefusalReason::RollbackTargetBelowEffectiveFloor\n        }	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_onto_an_abandoned_root_above_the_floor	rolling_back_onto_an_abandoned_root_above_the_floor_is_refused_for_being_abandoned
 增补 3 第 2 件（代码三方第一轮判决第三节第 2 条，C368 仍欠的那一半）：发布层把分配器的落点拒绝一律报成「每块盘上都没有」	crates/singlefs-core/src/transaction.rs	        unit: identity,\n        refusal,\n    })\n}	        unit: identity,\n        refusal: {\n            let _ = refusal;\n            PlacementRefusal::NoFreeSlotOnAnyDevice\n        },\n    })\n}	-p singlefs-harness --test second_transaction_supplement_two_unequal_devices -- filling_the_smaller_device	filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking
@@ -152,7 +152,7 @@
 增补 3 第 2 件（代码三方第二轮判决第三节第 1 条的改法）：胶水把回退候选排除的三条揉成一条（「不在环里」也报成「低于 F」）	crates/singlefs-harness/src/model_comparison.rs	        RollbackCandidateExclusion::NotInRing => ModelRefusalReason::RollbackTargetNotInRing,	        RollbackCandidateExclusion::NotInRing => ModelRefusalReason::RollbackTargetBelowEffectiveFloor,	-p singlefs-harness --lib -- model_comparison::tests::each_rollback_candidate_exclusion	each_rollback_candidate_exclusion_maps_to_its_own_reason
 增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2h）：分配器用户数据那一处把「每块盘上都没有」报成「小盘写满」；小盘段判出	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::Agreed(slot) => slot,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);	            DeviceAgreement::Agreed(slot) => slot,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices: Vec::new() });	-p singlefs-harness --test second_transaction_supplement_three_random_history -- unit_area_wall_sampling	unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device
 增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2i）：分配器用户数据那一处把「小盘写满」报成「每块盘上都没有」；等大的小盘走不到，不等盘那条用例判出	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::SomeDevicesWithoutAnswer(full_devices) => {\n                return Err(\n                    PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices },\n                );\n            }\n            DeviceAgreement::AnswersDiffer(slot_per_device) => {	            DeviceAgreement::SomeDevicesWithoutAnswer(_full_devices) => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);\n            }\n            DeviceAgreement::AnswersDiffer(slot_per_device) => {	-p singlefs-harness --test second_transaction_supplement_two_unequal_devices -- filling_the_smaller_device	filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking
-增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4a）：根环转过之后回收门槛取「环里最旧有效根 + 1」；逼近分配记录墙那一段照跑 checker 判出	crates/singlefs-core/src/mount.rs	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(if oldest.0 > 0 { CheckpointTxg(oldest.0 + 1) } else { oldest }))	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_wall_sampling	allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count
+增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4a）：根环转过之后回收门槛取「环里最旧有效根 + 1」；逼近分配记录墙那一段照跑 checker 判出	crates/singlefs-core/src/mount.rs	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(if oldest.0 > 0 { CheckpointTxg(oldest.0 + 1) } else { oldest }))	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_sampling	allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall
 增补 3 第 3 件：抽 0 个崩溃点时照样抽一个（抽崩溃点会改变这段历史怎么跑，判别力那一半失效）	crates/singlefs-harness/src/crash_injection.rs	            for _ in 0..crash_points_per_history {	            for _ in 0..crash_points_per_history.max(1) {	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- drawing_crash_points	drawing_crash_points_does_not_change_the_generated_history
 增补 3 第 3 件：判崩溃点时取择根而不是施加记录前缀之后实际走的那条根（记录已写、根槽未写那一格内容与根身份配不上）	crates/singlefs-harness/src/crash_injection.rs	        let read_back = observed_read_back_after_a_crash(&report);	        let read_back = crate::model_comparison::observed_read_back(&report.outcome);	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- crash_injection_fast_tier	crash_injection_fast_tier_recovers_only_into_versions_the_model_committed
 增补 3 第 3 件（用户 2026-09-20 定案第 1 条）：屏障不再切段（屏障进了枚举域，少一道屏障就把两段并成一段）	crates/singlefs-harness/src/crash.rs	            RecordedOperationKind::Barrier => {\n                if !current.is_empty() {\n                    segments.push(std::mem::take(&mut current));\n                }\n            }	            RecordedOperationKind::Barrier => {}	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- every_crash_state_of_a_written_out_history	every_crash_state_of_a_written_out_history_recovers_into_a_committed_version
@@ -264,7 +264,7 @@
 增补 2 收口第 27 行：只供测试的根槽点名判定恒不命中（开关失效，点名的槽照读）	crates/singlefs-harness/src/fault_injection.rs	        self.named_slots.named_slots().into_iter().find(|slot| {	        self.named_slots.named_slots().into_iter().take(0).find(|slot| {	-p singlefs-harness --test second_transaction_supplement_two_unreadable_abandoned_root_slot	an_abandoned_root_whose_own_root_slot_is_unreadable_is_neither_isolated_nor_counted
 增补 2 收口第 26 行：I-1.2 的出生身份比对恒成立（头与引用它的指针不符也判绿）	crates/singlefs-checker/src/walk.rs	        let matches_the_pointer = identity.write_order.birth_txg() == pointer.birth_txg	        let matches_the_pointer = true || identity.write_order.birth_txg() == pointer.birth_txg	-p singlefs-harness --test checker_known_bad_images	the_birth_identity_bad_images_redden_only_their_own_invariant
 增补 2 收口第 26 行：I-4.2 的已发布谓词恒成立（被引用的块不判提交没提交）	crates/singlefs-checker/src/walk.rs	        let is_published = published.holds_for(identity.write_order);	        let is_published = published.holds_for(identity.write_order) || true;	-p singlefs-harness --test checker_known_bad_images	the_birth_identity_bad_images_redden_only_their_own_invariant
-增补 2 收口第 26 行：走读数据单元时不判出生身份（I-1.2 / I-4.2 的调用点被摘掉）	crates/singlefs-checker/src/walk.rs	            self.judge_birth_identity_of_a_referenced_unit(&unit, &pointer, "数据单元");	            let _ = &pointer;	-p singlefs-harness --test checker_known_bad_images	the_birth_identity_bad_images_redden_only_their_own_invariant
+增补 2 收口第 26 行：走读数据单元时不判出生身份（I-1.2 / I-4.2 的调用点被摘掉）	crates/singlefs-checker/src/walk.rs	        self.judge_birth_identity_of_a_referenced_unit(&unit, &pointer, "数据单元");	        let _ = &pointer;	-p singlefs-harness --test checker_known_bad_images	the_birth_identity_bad_images_redden_only_their_own_invariant
 C504（树表条目宽在走读里无守卫）：走读树表条目之前不判条目宽，三步固定偏移直接切	crates/singlefs-checker/src/walk.rs	        if entry.len() < tree_table_entry_bytes() {\n            self.walk_failures.push(format!(	        if false && entry.len() < tree_table_entry_bytes() {\n            self.walk_failures.push(format!(	-p singlefs-harness --test second_transaction_supplement_three_bad_disk_input -- bad_disk_inputs_never_read_back	bad_disk_inputs_never_read_back_an_uncommitted_version_and_panic_only_at_known_sites
 步 3（R1）：树表 0 条的一版上写行时，被换下的那片实例表不释放（defer 队列里少 2 槽）	crates/singlefs-core/src/transaction.rs	    allocator.release(released, txg);	    let _ = released;	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_formatted_pool_mounted_twice	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
 步 3（三处写死的前提之二）：第一个文件版本照抄的实例表指针取错成树表指针	crates/singlefs-core/src/transaction.rs	            instance_table: InstanceTablePlan::Carry(version_to_build_on.instance_table),	            instance_table: InstanceTablePlan::Carry(version_to_build_on.tree_table),	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_formatted_pool_mounted_twice	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
@@ -282,7 +282,7 @@
 C511（2026-09-23 用户定案，I-9.14 收窄反向那一头）：「同一条时间线」读成「同一个实例」，I-9.14 那一遍只拿最新根那个实例的根比（跨过回退行、仍在现行线上的回退目标 A 不再比）	crates/singlefs-checker/src/walk.rs	    judge_tree_table_birth_txg(&scanned, judgements);	    scanned.retain(|root| roots[root.root_index].2.instance == roots[newest_index].2.instance);\n    judge_tree_table_birth_txg(&scanned, judgements);	-p singlefs-harness --test checker_known_bad_images -- birth_txg_that_the_line_after_a_rollback	birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_birth_invariant
 C511（2026-09-23 用户定案，I-9.14 收窄正向那一头）：I-3.9 与 I-9.14 那一遍拿根环里每一条根比，不按实例表把被回退切掉的根剔掉（回到「所有 sighting 相等」）	crates/singlefs-checker/src/walk.rs	    judge_release_generation_and_tree_table_birth(\n        reader,\n        &roots,\n        &candidate_indexes,\n        newest_index,	    judge_release_generation_and_tree_table_birth(\n        reader,\n        &roots,\n        &(0..roots.len()).collect::<Vec<usize>>(),\n        newest_index,	-p singlefs-harness --test checker_known_bad_images -- birth_txg_recorded_differently_only_on_the_timeline	birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_rollback_is_not_compared_and_every_invariant_holds
 C512（2026-09-23 用户定案）：写行那次发布不把分配记录树的根写进根记录（那一版的账重开之后取不回来，被换下的实例表成了空闲槽）	crates/singlefs-core/src/transaction.rs	        mapping_root: previous_root.mapping_root,\n        allocation_record_tree_root,	        mapping_root: previous_root.mapping_root,\n        allocation_record_tree_root: NodePointer::empty_root(),	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- the_third_writable_mount_keeps	the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool
-C512（2026-09-23 用户定案）：重开时从这一版的分配记录树重建账，却把带已释放标志的记录丢掉（被换下的那一片当场成了空闲槽）	crates/singlefs-core/src/mount.rs	    let mut allocator = PoolAllocator::rebuild_from_records(device_maps, records);	    let mut allocator = PoolAllocator::rebuild_from_records(\n        device_maps,\n        records.into_iter().filter(|record| !record.is_released).collect(),\n    );	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- the_third_writable_mount_keeps	the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool
+C512（2026-09-23 用户定案）：重开时从这一版的分配记录树重建账，却把带已释放标志的记录丢掉（被换下的那一片当场成了空闲槽）	crates/singlefs-core/src/mount.rs	    let mut allocator = PoolAllocator::rebuild_from_records(device_maps, tree.records.clone());	    let mut allocator = PoolAllocator::rebuild_from_records(\n        device_maps,\n        tree.records\n            .iter()\n            .filter(|record| !record.is_released)\n            .copied()\n            .collect(),\n    );	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- the_third_writable_mount_keeps	the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool
 I-8.7（实例内事务号不重号） ② 同号两条之间夹着的记录不看：一次空发布插进一个事务中间也判绿（夹进空发布那份交错镜像不红）	crates/singlefs-checker/src/walk.rs	                    .find(|(_, between)| between.instance == record.instance);	                    .find(|_| false);	-p singlefs-harness --test checker_known_bad_images -- each_transaction_boundary_bad_image	each_transaction_boundary_bad_image_reddens_only_its_own_invariant_on_its_registered_criteria
 I-8.7 ② 的「被别的非 0 号隔开之后又出现」不写进说明（夹进另一个事务那份交错镜像只剩 ①）	crates/singlefs-checker/src/walk.rs	            let reappears_after_another_number = first_counter_of_this_number != *counter;	            let reappears_after_another_number = false;	-p singlefs-harness --test checker_known_bad_images -- each_transaction_boundary_bad_image	each_transaction_boundary_bad_image_reddens_only_its_own_invariant_on_its_registered_criteria
 I-8.7 回到严格递增：同号相邻也判红（一事务跨两条的阳性对照红，D23 已定项 7 允许的形态被当成重号）	crates/singlefs-checker/src/walk.rs	            if record.transaction == previous_transaction {	            if false {	-p singlefs-harness --test checker_known_bad_images -- one_transaction_over_two_records	one_transaction_over_two_records_with_the_commit_marker_on_the_last_holds_both_transaction_boundary_invariants
@@ -356,11 +356,11 @@
 增补 2 第 58 行：真设备二进制失败窗口按种类那一侧不算失败账	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	    let failed_publishes: Vec<&WritesByStructureKind> = writes_of_failed_publishes.iter().collect();	    let failed_publishes: Vec<&WritesByStructureKind> = Vec::new();	-p singlefs-harness --bin first_transaction_on_device	second_version_publish_failing_midway_reports_the_writes_it_landed_and_they_equal_the_device_layer_count
 增补 2 第 58 行：发布 B / 发布 C 失败时不交写入口的失败账	crates/singlefs-harness/src/on_device_modes.rs	        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),	        writes_of_failed_publishes: Vec::new(),	-p singlefs-harness --bin first_transaction_on_device	third_version_publish_failing_midway_reports_the_writes_it_landed_and_they_equal_the_device_layer_count
 增补 2 第 21 行（C366，挂载层）：写行那次发布的 jsn 取 txg（所选根自己那条记录读不出时写行 txg 5、jsn 应是 4）	crates/singlefs-core/src/mount.rs	                RowPublishIdentity {\n                    txg: start.first_txg,\n                    counter: start.next_counter,	                RowPublishIdentity {\n                    txg: start.first_txg,\n                    counter: start.first_txg.0,	-p singlefs-harness --test second_transaction_supplement_two_warm_up_counter -- c366_when_the_chosen_root_own_record_is_unreadable	c366_when_the_chosen_root_own_record_is_unreadable_the_new_instance_numbers_records_and_tail_from_the_highest_readable_record
-增补 2 第 21 行（C366，挂载层）：带文件的一版上暖机的 jsn 取上一版的 txg + 1	crates/singlefs-core/src/mount.rs	                counter: current_file_version.record.counter + 1,	                counter: current_file_version.root.checkpoint_txg.0 + 1,	-p singlefs-harness --test second_transaction_supplement_two_warm_up_counter -- c366_when_the_chosen_root_own_record_is_unreadable	c366_when_the_chosen_root_own_record_is_unreadable_the_new_instance_numbers_records_and_tail_from_the_highest_readable_record
+增补 2 第 21 行（C366，挂载层）：带文件的一版上暖机的 jsn 取上一版的 txg + 1	crates/singlefs-core/src/mount.rs	        counter: current_file_version.record.counter + 1,	        counter: current_file_version.root.checkpoint_txg.0 + 1,	-p singlefs-harness --test second_transaction_supplement_two_warm_up_counter -- c366_when_the_chosen_root_own_record_is_unreadable	c366_when_the_chosen_root_own_record_is_unreadable_the_new_instance_numbers_records_and_tail_from_the_highest_readable_record
 增补 2 第 21 行（C366，挂载层）：发布轮换系统配置槽时 tail 写 txg、不写 jsn	crates/singlefs-core/src/transaction.rs	    writer.perform(CommitStep::RotateSystemConfigurationSlots {\n        journal_tail,\n        journal_instance,\n    })	    writer.perform(CommitStep::RotateSystemConfigurationSlots {\n        journal_tail: checkpoint_txg.0,\n        journal_instance,\n    })	-p singlefs-harness --test second_transaction_supplement_two_warm_up_counter -- c366_when_the_chosen_root_own_record_is_unreadable	c366_when_the_chosen_root_own_record_is_unreadable_the_new_instance_numbers_records_and_tail_from_the_highest_readable_record
 增补 2 第 61 行（C519 已知丢失）：施加前的点名单元验证改成任一条位置条目验过即过（改法 Z2 被悄悄落地时已知丢失那条先红）	crates/singlefs-core/src/recovery.rs	                named.locations.iter().all(|location| {	                named.locations.iter().any(|location| {	-p singlefs-harness --test second_transaction_supplement_two_c519_whole_device_loss_after_warm_up -- c519_known_loss	c519_known_loss_after_the_warm_up_covers_both_devices_losing_the_acknowledged_version_root_device_falls_back_to_the_row_publish_version
 增补 2 第 62 行（C533 复现）：前缀跨实例边界，写行那条孤记录进了水位之上	crates/singlefs-core/src/recovery.rs	            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water	            (record.instance, record.checkpoint_txg) > water	-p singlefs-harness --test second_transaction_supplement_two_c533_row_publish_record_without_its_root -- c533_row_publish_record_persisted	c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_are_free_slots_that_the_next_mount_overwrites
-增补 2 第 9 行 P1（C497）：树表 0 条的一版上写行时分配记录节点先于实例表取落点	crates/singlefs-core/src/transaction.rs	    let mut instance_table_slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();\n    for identity in &instance_table_roles {\n        let placement = allocate_placement_for_role(allocator, *identity, txg)?;\n        instance_table_slots.insert(*identity, placement.slot);\n    }\n    let allocation_placement =\n        allocate_placement_for_role(allocator, TransactionUnit::AllocationTree, txg)?;\n	    let allocation_placement =\n        allocate_placement_for_role(allocator, TransactionUnit::AllocationTree, txg)?;\n    let mut instance_table_slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();\n    for identity in &instance_table_roles {\n        let placement = allocate_placement_for_role(allocator, *identity, txg)?;\n        instance_table_slots.insert(*identity, placement.slot);\n    }\n	-p singlefs-harness --test second_transaction_supplement_two_presumed_clause_checks -- c497_every_publish	c497_every_publish_that_rewrites_the_instance_table_bumps_it_before_every_other_commit_generated_block
+增补 2 第 9 行 P1（C497）：树表 0 条的一版上写行时分配记录树节点先于实例表取落点	crates/singlefs-core/src/transaction.rs	        let rewritten_roles: Vec<TransactionUnit> = instance_table_roles\n            .iter()\n            .copied()\n            .chain(\n                allocation_plan\n                    .rewritten_nodes()\n                    .into_iter()\n                    .map(role_of_allocation_record_tree_node),\n            )\n            .collect();	        let rewritten_roles: Vec<TransactionUnit> = allocation_plan\n            .rewritten_nodes()\n            .into_iter()\n            .map(role_of_allocation_record_tree_node)\n            .chain(instance_table_roles.iter().copied())\n            .collect();	-p singlefs-harness --test second_transaction_supplement_two_presumed_clause_checks -- c497_every_publish	c497_every_publish_that_rewrites_the_instance_table_bumps_it_before_every_other_commit_generated_block
 增补 2 第 9 行 P3（C498）：暖机不看本实例的根覆盖没覆盖两块盘、恒推 R 次	crates/singlefs-core/src/mount.rs	    while all_devices\n        .iter()\n        .any(|identity| !covered.contains(identity))\n        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	    while u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	-p singlefs-harness --test second_transaction_supplement_two_presumed_clause_checks -- c498_warm_up_publishes	c498_warm_up_publishes_after_remounts_rollback_and_a_crash_in_the_middle_of_a_mount_are_counted_by_the_clause
 增补 2 第 9 行 P5（C499）：可写挂载认「上一次干净关闭」（tail 那条就是所选根那次、下一格没有记录）就只取 tail 那一条、不扫全环	crates/singlefs-core/src/mount.rs	    let records = scan_journal(&*devices, &system_configuration);\n    let (journal, effective_root) = replay_journal(	    let tail_record = |counter: u64| {\n        crate::recovery::PoolReader::read(\n            &*devices,\n            devices[0].0,\n            crate::journal::record_offset(\n                counter,\n                system_configuration.immutable.sizes.journal_ring_bytes,\n            ),\n            4096,\n        )\n        .and_then(|bytes| {\n            crate::journal::JournalRecord::parse(\n                &bytes,\n                crate::unit::unit_filesystem_identifier(\n                    &system_configuration.immutable.filesystem_identifier,\n                ),\n            )\n        })\n    };\n    let tail = system_configuration.quantities.journal_tail;\n    let closed_cleanly = tail > 0\n        && tail_record(tail).is_some_and(|record| {\n            record.instance == chosen_root.instance\n                && record.checkpoint_txg == chosen_root.checkpoint_txg\n        })\n        && tail_record(tail + 1).is_none();\n    let records: std::collections::BTreeMap<_, _> = if closed_cleanly {\n        tail_record(tail)\n            .map(|record| ((record.instance, record.counter), record))\n            .into_iter()\n            .collect()\n    } else {\n        scan_journal(&*devices, &system_configuration)\n    };\n    let (journal, effective_root) = replay_journal(	-p singlefs-harness --test second_transaction_supplement_two_presumed_clause_checks -- c499_remount_after	c499_remount_after_a_clean_close_and_after_a_crash_both_scan_the_whole_ring_and_recover_by_the_same_rules
 增补 2 第 8 行（C495）：第一个文件版本的上一条记录读不出时当它接得上、jsn 从 1 起	crates/singlefs-core/src/transaction.rs	    .map(|record| (record.checkpoint_txg, record.counter));	    .map(|record| (record.checkpoint_txg, record.counter))\n    .or(Some((version_to_build_on.checkpoint_txg, 0)));	-p singlefs-harness --test second_transaction_supplement_two_c495_first_file_version_previous_record -- c495_first_file_version_whose_previous_record_is_unreadable	c495_first_file_version_whose_previous_record_is_unreadable_on_both_copies_is_refused_before_any_write
@@ -373,7 +373,7 @@
 C483 ②：树节点的位置提示读不出时不经映射回退（直接报 UnitUnreadable，挂载态与冷走读一起退回）	crates/singlefs-core/src/recovery.rs	    if let Ok(bytes) = read_unit_via_locations(reader, &pointer.locations, unit_bytes) {\n        return Ok(bytes);\n    }	    let bytes = read_unit_via_locations(reader, &pointer.locations, unit_bytes)?;\n    if bytes.len() == unit_bytes {\n        return Ok(bytes);\n    }	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- an_extent_tree_root_whose	an_extent_tree_root_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping
 C483 ②：树节点经映射回退时多跳计数不加（D19 已定项 5 硬规则 3 的观测点对树节点等于没有）	crates/singlefs-core/src/recovery.rs	    *stale_location_hint_hops += 1;	    *stale_location_hint_hops += 0;	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- an_extent_tree_root_whose	an_extent_tree_root_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping
 C483 ②：树节点经映射仍读不出时报成 UnitUnreadable（与数据单元那一条回退的成员分叉）	crates/singlefs-core/src/recovery.rs	        RecoveryFailure::MappingStillUnreadable {\n            slot: mapped_locations[0].slot,\n        }	        RecoveryFailure::UnitUnreadable {\n            slot: mapped_locations[0].slot,\n        }	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- an_extent_tree_root_moved	an_extent_tree_root_moved_without_updating_the_central_mapping_is_still_unreadable_after_the_hop
-C483 ②：挂载态打开时 extent 树根只按位置提示读（不经映射回退，攻方腿量到的「整池挂不上」）	crates/singlefs-core/src/mounted_read.rs	    let extent_root = read_mapped_tree_root(\n        reader,\n        extent_entry.tree,\n        EXTENT_KEY_WIDTH_IN_BYTES,\n        &extent_entry.root,\n        root,\n        filesystem_identifier_in_unit_headers,\n        &central_mapping_locations_of_key,\n        &mut tree_node_stale_location_hint_hops,\n    )?;	    let extent_root = read_tree_root(\n        reader,\n        extent_entry.tree,\n        EXTENT_KEY_WIDTH_IN_BYTES,\n        &extent_entry.root,\n        root,\n        filesystem_identifier_in_unit_headers,\n    )?;	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- an_extent_tree_root_whose	an_extent_tree_root_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping
+C483 ② × K4：打开文件时 extent 树节点只按位置提示读（经映射回退那一路查不到任何 key，提示过期就打不开）	crates/singlefs-core/src/mounted_read.rs	            |mapping_key: &[u8]| Ok(self.central_mapping_lookup(mapping_key));	            |_mapping_key: &[u8]| Ok(None);	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- an_extent_tree_root_whose	an_extent_tree_root_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping
 C483 ②：挂载态打开时 inode 树根只按位置提示读（不经映射回退）	crates/singlefs-core/src/mounted_read.rs	    let inode_root = read_mapped_tree_root(\n        reader,\n        inode_entry.tree,\n        INODE_KEY_WIDTH_IN_BYTES,\n        &inode_entry.root,\n        root,\n        filesystem_identifier_in_unit_headers,\n        &central_mapping_locations_of_key,\n        &mut tree_node_stale_location_hint_hops,\n    )?;	    let inode_root = read_tree_root(\n        reader,\n        inode_entry.tree,\n        INODE_KEY_WIDTH_IN_BYTES,\n        &inode_entry.root,\n        root,\n        filesystem_identifier_in_unit_headers,\n    )?;	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- an_inode_tree_root_whose	an_inode_tree_root_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping
 C483 ②：挂载态打开时 inode 叶容器只按位置提示读（不经映射回退）	crates/singlefs-core/src/mounted_read.rs	        let container_bytes = read_mapped_tree_node_via_hint_then_central_mapping(\n            reader,\n            &child,\n            MappedTreeNodeClass::PackedRecordUnit,\n            &central_mapping_locations_of_key,\n            &mut tree_node_stale_location_hint_hops,\n        )?;	        let container_bytes = read_unit_via_locations(\n            reader,\n            &child.locations,\n            usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768"),\n        )?;	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- an_inode_leaf_container_whose	an_inode_leaf_container_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping
 C483 ②：冷走读沿树表读树根时只按位置提示读（不经映射回退，挂载态回退了而冷走读没回退）	crates/singlefs-core/src/recovery.rs	            read_mapped_tree_root(\n                reader,\n                entry.tree,\n                key_width,\n                &entry.root,\n                root,\n                expected_filesystem_identifier,\n                &central_mapping_locations_of_key,\n                mapping_fallbacks,\n            )?,	            read_tree_root(\n                reader,\n                entry.tree,\n                key_width,\n                &entry.root,\n                root,\n                expected_filesystem_identifier,\n            )?,	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- an_extent_tree_root_whose	an_extent_tree_root_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping
@@ -390,7 +390,7 @@
 实例表第二片：读者读到第 0 片就停，下一片的行不接	crates/singlefs-core/src/recovery.rs	            InstanceTableChainRecord::NextPage(next_page) => {\n                page_pointers.push(next_page);\n                page_index = page_index.next();\n            }	            InstanceTableChainRecord::NextPage(_next_page) => {\n                return InstanceTableChainRead {\n                    page_pointers,\n                    outcome: Ok(InstanceTableRecords { rows }),\n                };\n            }	-p singlefs-harness --test second_transaction_supplement_two_instance_table_chain -- the_reader_follows	the_reader_follows_the_chain_into_the_second_page_and_a_single_page_parse_refuses_a_chained_first_page
 实例表第二片：读者读到第 0 片就停，回退候选集看不到第二片上的行	crates/singlefs-core/src/recovery.rs	            InstanceTableChainRecord::NextPage(next_page) => {\n                page_pointers.push(next_page);\n                page_index = page_index.next();\n            }	            InstanceTableChainRecord::NextPage(_next_page) => {\n                return InstanceTableChainRead {\n                    page_pointers,\n                    outcome: Ok(InstanceTableRecords { rows }),\n                };\n            }	-p singlefs-harness --test second_transaction_supplement_two_instance_table_chain -- rollback_candidate_set	rollback_candidate_set_reads_the_row_that_lives_on_the_second_page
 实例表第二片：冷启动走读不沿链读（第二片坏了照样走完）	crates/singlefs-core/src/recovery.rs	            instance_table_chain_of_root(reader, root)?;	            let _ = instance_table_chain_of_root(reader, root);	-p singlefs-harness --test second_transaction_supplement_two_instance_table_chain -- cold_start_walks	cold_start_walks_every_page_of_the_instance_table_and_an_unreadable_second_page_stops_it
-实例表第二片：取号之前不按释放判定路径逐片核被换下的旧链（旧链上一片不在账里时取号写完才在发布路径里撞上，号烧掉）	crates/singlefs-core/src/mount.rs	    if row_publish_rewrites_the_instance_table {	    if false && row_publish_rewrites_the_instance_table {	-p singlefs-harness --test second_transaction_supplement_two_instance_table_chain -- writable_mount_follows_the_chain	writable_mount_follows_the_chain_and_refuses_a_page_missing_from_the_allocation_records_before_acquisition
+实例表第二片：取号之前的预演不按释放判定路径逐片核被换下的旧链（旧链上一片不在账里时取号写完才撞上，号烧掉；预演走的是发布路径落盘之前那一段，这一处两边共用；树表 0 条的一版上写行那一处）	crates/singlefs-core/src/transaction.rs	    let chain = instance_table_chain_to_release(&instance_table.replaced_chain, allocator)?;	    let chain: Vec<Placement> = {\n        let _ = &instance_table.replaced_chain;\n        Vec::new()\n    };	-p singlefs-harness --test second_transaction_supplement_two_instance_table_chain -- writable_mount_follows_the_chain	writable_mount_follows_the_chain_and_refuses_a_page_missing_from_the_allocation_records_before_acquisition
 实例表第二片：影子账只隔离根记录指着的第 0 片	crates/singlefs-core/src/mount.rs	            let instance_table_pages =\n                instance_table_page_pointers_as_far_as_readable(devices, root);	            let instance_table_pages = vec![root.instance_table];	-p singlefs-harness --test second_transaction_supplement_two_instance_table_chain -- rollback_isolates	rollback_isolates_every_page_of_the_instance_table_chain_that_only_abandoned_roots_reference
 实例表第二片：checker 拿第 0 片的身份判链上每一片	crates/singlefs-checker/src/walk.rs	            let expected_identity = (0, PACKED_TYPE_INSTANCE_TABLE, page_index, 0);	            let expected_identity = (0, PACKED_TYPE_INSTANCE_TABLE, 0, 0);	-p singlefs-harness --test second_transaction_supplement_two_instance_table_chain -- the_pool_checker_walks	the_pool_checker_walks_both_pages_of_a_valid_chain_and_every_invariant_holds
 实例表第二片：checker 不沿链走（链指针指到别的单元不红）	crates/singlefs-checker/src/walk.rs	                ChainRecordView::NextPage(next_page) => {\n                    pointer = next_page;\n                    page_index += 1;\n                }	                ChainRecordView::NextPage(_next_page) => break,	-p singlefs-harness --test second_transaction_supplement_two_instance_table_chain -- chain_pointer_to_another	chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant
@@ -412,15 +412,15 @@
 并行线一：C490：extent 叶记录 key 的 offset 段写成单元序号（并行线一验收第 4 条第二个变异：读回错位判红）	crates/singlefs-core/src/transaction.rs	        let extent_record = build_extent_record(\n            FIRST_INODE_NUMBER,\n            transaction.payload_start.0,	        let extent_record = build_extent_record(\n            FIRST_INODE_NUMBER,\n            transaction.unit_index_in_file.0,	-p singlefs-harness --test second_transaction_parallel_line_one_sequential_write -- the_second_extent_leaf_record_key_is_the_file_byte_offset_of_the_second_data_unit	the_second_extent_leaf_record_key_is_the_file_byte_offset_of_the_second_data_unit
 并行线一：C491：共享的提交内生块挪到一次发布的第一条记录里点名（末条只点名自己那个数据单元）	crates/singlefs-core/src/transaction.rs	    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles_named_before_the_last_record\n            .iter()\n            .map(|data_role| vec![*data_role])\n            .collect();\n    roles_named_by_each_record.push(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !data_roles_named_before_the_last_record.contains(identity))\n            .collect(),\n    );	    let _ = data_roles_named_before_the_last_record;\n    if data_roles.len() < 2 {\n        return vec![rewritten.to_vec()];\n    }\n    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles.iter().map(|data_role| vec![*data_role]).collect();\n    roles_named_by_each_record[0].extend(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !matches!(identity, TransactionUnit::Data(_))),\n    );	-p singlefs-harness --test second_transaction_parallel_line_one_sequential_write -- a_publish_of_two_data_units_names_the_shared_commit_generated_units_only_in_its_last_record	a_publish_of_two_data_units_names_the_shared_commit_generated_units_only_in_its_last_record
 并行线一：C376：每条记录都当成它那次发布的末条（发布边界不认，前缀停在发布中间也施加）	crates/singlefs-core/src/recovery.rs	        if !record_ends_its_publish(record) {	        if false {	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- a_crash_before_the_second_record_of_a_two_record_publish_keeps_the_old_file	a_crash_before_the_second_record_of_a_two_record_publish_keeps_the_old_file
-并行线一：extent 树要长内部节点也不拒（内部条目格式没有条款那一格被跳过）	crates/singlefs-core/src/transaction.rs	    if rewritten_data_units > extent_leaf_capacity {	    if false {	-p singlefs-harness --test parallel_line_one_sequential_write -- a_sequential_write_needing_more_data_units_than_one_extent_leaf_holds_is_refused_before_any_write	a_sequential_write_needing_more_data_units_than_one_extent_leaf_holds_is_refused_before_any_write
+K2：extent 树下段根的层级取「罩的单元数大于单元数」的最低一层（144 个单元装满一片叶时白长一层）	crates/singlefs-core/src/extent_tree.rs	        .find(|level| lower_span_in_data_units(*level) >= data_units)	        .find(|level| lower_span_in_data_units(*level) > data_units)	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- the_lower_extent_segment	the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline
 并行线一：文件变短时上一版多出来的数据单元不释放	crates/singlefs-core/src/transaction.rs	        .chain(previous_data_units_without_a_successor)	        .chain(Vec::<TransactionUnit>::new())	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has	shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has
-并行线一：从盘上重建上一版只读 extent 根兼叶的第一条记录（多单元文件重开之后只剩第一个数据单元）	crates/singlefs-core/src/recovery.rs	    for extent_record_bytes in &extent_node.entries {	    for extent_record_bytes in extent_node.entries.iter().take(1) {	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- a_reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping	a_reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping
+并行线一：从盘上重建上一版只取第一个文件的第一个数据指针（多单元文件重开之后只剩第一个数据单元）	crates/singlefs-core/src/recovery.rs	    let data_pointers_with_units = extents.data_pointers();	    let data_pointers_with_units: Vec<(u64, DataPointer)> =\n        extents.data_pointers().into_iter().take(1).collect();	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- a_reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping	a_reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping
 并行线一：挂载态打开文件不核 extent key 的 offset 段是不是那个单元的文件字节偏移	crates/singlefs-core/src/mounted_read.rs	            if record.file_offset_in_bytes() != expected_file_offset {	            if false {	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- a_mount_state_open_refuses_an_extent_key_whose_offset_segment_is_the_unit_index_instead_of_the_file_byte_offset	a_mount_state_open_refuses_an_extent_key_whose_offset_segment_is_the_unit_index_instead_of_the_file_byte_offset
 并行线一：记录读者不判点名项区越没越过 4096（并行线一验收第 4 条第三个变异：点名项超过 67 仍塞进一条记录 ⇒ 记录解析拒收）	crates/singlefs-core/src/journal.rs	        if payload_end > record_bytes\n            || crc32_castagnoli	        if crc32_castagnoli	-p singlefs-core --lib -- journal::tests::a_record_whose_header_claims_more_named_units_than_one_record_holds_is_refused_by_the_parser	a_record_whose_header_claims_more_named_units_than_one_record_holds_is_refused_by_the_parser
 并行线一：一个数据单元的发布也切成两条记录（末条只点名共享内生块）	crates/singlefs-core/src/transaction.rs	    let data_roles_named_before_the_last_record = &data_roles[..data_roles.len().saturating_sub(1)];	    let data_roles_named_before_the_last_record = &data_roles[..];	-p singlefs-harness --test second_transaction_parallel_line_one_sequential_write -- a_sequential_write_that_fits_one_data_unit_publishes_through_the_single_transaction_path	a_sequential_write_that_fits_one_data_unit_publishes_through_the_single_transaction_path
 并行线一：层 0：共享内生块挪到第一条记录里点名（里程碑并行线一验收第 4 条第一个变异「提交标记提前到第一条记录」在一事务一记录下的形态：半次发布施加上去）	crates/singlefs-core/src/transaction.rs	    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles_named_before_the_last_record\n            .iter()\n            .map(|data_role| vec![*data_role])\n            .collect();\n    roles_named_by_each_record.push(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !data_roles_named_before_the_last_record.contains(identity))\n            .collect(),\n    );	    let _ = data_roles_named_before_the_last_record;\n    if data_roles.len() < 2 {\n        return vec![rewritten.to_vec()];\n    }\n    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles.iter().map(|data_role| vec![*data_role]).collect();\n    roles_named_by_each_record[0].extend(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !matches!(identity, TransactionUnit::Data(_))),\n    );	-p singlefs-harness --test second_transaction_parallel_line_one_layer0 -- every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it	every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it
 并行线一：层 0：发布边界不认（每条记录都当末条）	crates/singlefs-core/src/recovery.rs	        if !record_ends_its_publish(record) {	        if false {	-p singlefs-harness --test second_transaction_parallel_line_one_layer0 -- every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it	every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it
-步 1 验收第 4 条（第 331 行那条锚点腐化了，照原意重写锚点）：extent 叶记录的指针忘了换（覆盖写之后仍指上一版的第一个数据单元，读回等于旧内容）	crates/singlefs-core/src/transaction.rs	        (Some(file), _) => build_file_version_units(\n            &checkpoint,\n            file,\n            &file_content_transactions,\n            &FileVersionSlots {\n                data: file_content_transactions\n                    .iter()\n                    .map(|transaction| {\n                        slot_of(TransactionUnit::Data(transaction.unit_index_in_file))\n                    })\n                    .collect(),\n            },\n            &mut sequences,\n        ),	        (Some(file), _) => {\n            let mut built = build_file_version_units(\n                &checkpoint,\n                file,\n                &file_content_transactions,\n                &FileVersionSlots {\n                    data: file_content_transactions\n                        .iter()\n                        .map(|transaction| {\n                            slot_of(TransactionUnit::Data(transaction.unit_index_in_file))\n                        })\n                        .collect(),\n                },\n                &mut sequences,\n            );\n            if let Some(previous_version) = previous {\n                let stale_extent_record =\n                    build_extent_record(FIRST_INODE_NUMBER, 0, previous_version.data_pointers[0]);\n                let stale_extent_key: [u8; 24] =\n                    stale_extent_record[..24].try_into().expect("24");\n                built.extent_unit = build_index_node(\n                    TreeIdentifier(TREE_IDENTIFIER_EXTENT),\n                    0,\n                    24,\n                    &stale_extent_key,\n                    &stale_extent_key,\n                    txg,\n                    filesystem_identifier,\n                    instance,\n                    built.extent_sequence,\n                    112,\n                    std::slice::from_ref(&stale_extent_record),\n                );\n            }\n            built\n        }	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start_reads_the_second_content_and_the_pool_checker_stays_green	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
+步 1 验收第 4 条 × K2：extent 上段叶条目内联的数据指针忘了换（覆盖写之后仍指上一版的第一个数据单元，读回等于旧内容）	crates/singlefs-core/src/transaction.rs	                    None => ExtentUpperLeafTarget::InlineDataUnit(\n                        *data_pointers\n                            .first()	                    None => ExtentUpperLeafTarget::InlineDataUnit(\n                        *previous\n                            .map_or(data_pointers, |previous_version| {\n                                previous_version.data_pointers.as_slice()\n                            })\n                            .first()	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start_reads_the_second_content_and_the_pool_checker_stays_green	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
 并行线一：数据单元头里的锚点偏移写成单元序号（与 extent key 的 offset 段不符：冷启动顺序读在第 1 个单元当场拒，I-1.1）	crates/singlefs-core/src/transaction.rs	            anchor_offset: transaction.payload_start.0,	            anchor_offset: transaction.unit_index_in_file.0,	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte	units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte
 并行线一：数据单元的净荷字节倒序写（长度、锚点、写序都对，只有内容错：冷启动顺序读回与写入逐字节比对判红）	crates/singlefs-core/src/transaction.rs	            transaction.payload_of(file.content),	            &transaction.payload_of(file.content).iter().rev().copied().collect::<Vec<u8>>(),	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte	units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte
 并行线一：C394：释放之前读盘核校验和只走这次重写的角色（文件变短换下的尾巴不核，被改坏的那一份照常回到空闲池）	crates/singlefs-core/src/transaction.rs	    roles_replaced_via_mapping(previous, roles)\n        .into_iter()\n	    roles\n        .iter()\n        .copied()\n	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- shrinking_a_multi_unit_file_checks_the_released_tail_against_its_mapping_checksums_before_releasing_it	shrinking_a_multi_unit_file_checks_the_released_tail_against_its_mapping_checksums_before_releasing_it
@@ -471,8 +471,8 @@
 增补 2 收口表第 ② 行（根环表跟不上改成断言）：记到跳号的根时不断言、照旧按猜的环回收	crates/singlefs-core/src/allocator.rs	            checkpoint_txg, expected_txg,	            checkpoint_txg, checkpoint_txg,	-p singlefs-core --lib -- a_root_ring_occupancy_that_missed_a_root	a_root_ring_occupancy_that_missed_a_root_panics_instead_of_reclaiming_by_a_guessed_ring
 增补 2 收口表第 ② 行（根环表跟不上改成断言）：根环表记下不跳号的根之后不回收	crates/singlefs-core/src/allocator.rs	        let reclaimed =\n            self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);	        let reclaimed: Vec<AllocationRecord> = Vec::new();	-p singlefs-core --lib -- a_root_ring_occupancy_that_follows_every_root	a_root_ring_occupancy_that_follows_every_root_reclaims_as_the_ring_turns
 增补 2 收口表第 ② 行：mkfs 同一个进程那条会话不装根环表（根环转过之后 I-3.1 在合法状态上红，原已知红第 0 条那一形）	crates/singlefs-core/src/make_filesystem.rs	    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(\n        parameters, genesis,\n    ));	    let _ = root_ring_occupancy_after_make_filesystem(parameters, genesis);	-p singlefs-harness --test second_transaction_supplement_three_random_history -- turning_the_root_ring_with_overwrites_in_the_make_filesystem_process	turning_the_root_ring_with_overwrites_in_the_make_filesystem_process_runs_to_the_end_with_the_checker_green_after_every_step
-增补 2 收口表第 38 行顺带发现 O1：影子账不认树表 0 条那一版自己那片分配记录树节点（被抛弃根的那一片回退之后是空闲槽）	crates/singlefs-core/src/mount.rs	                .chain([(&root.tree_table, TransactionUnit::TreeTable)])\n                .chain(allocation_record_node)	                .chain([(&root.tree_table, TransactionUnit::TreeTable)])	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- rolling_back_before_any_file_version_isolates	rolling_back_before_any_file_version_isolates_the_allocation_record_node_only_the_abandoned_roots_reference
-增补 2 收口表第 38 行顺带发现 O1（同一处）：两片实例表链的被抛弃根，隔离里少那片分配记录树节点	crates/singlefs-core/src/mount.rs	                .chain([(&root.tree_table, TransactionUnit::TreeTable)])\n                .chain(allocation_record_node)	                .chain([(&root.tree_table, TransactionUnit::TreeTable)])	-p singlefs-harness --test second_transaction_supplement_two_instance_table_chain -- rollback_isolates_every_page	rollback_isolates_every_page_of_the_instance_table_chain_that_only_abandoned_roots_reference
+增补 2 收口表第 38 行顺带发现 O1：影子账不认树表 0 条那一版自己那棵分配记录树的节点（被抛弃根的那几片回退之后是空闲槽）	crates/singlefs-core/src/mount.rs	                .chain([(&root.tree_table, TransactionUnit::TreeTable)])\n                .chain(\n                    allocation_record_tree_nodes\n                        .iter()\n                        .map(|pointer| (pointer, TransactionUnit::AllocationTree)),\n                )	                .chain([(&root.tree_table, TransactionUnit::TreeTable)])	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- rolling_back_before_any_file_version_isolates	rolling_back_before_any_file_version_isolates_the_allocation_record_node_only_the_abandoned_roots_reference
+增补 2 收口表第 38 行顺带发现 O1（同一处）：两片实例表链的被抛弃根，隔离里少那棵分配记录树的节点	crates/singlefs-core/src/mount.rs	                .chain([(&root.tree_table, TransactionUnit::TreeTable)])\n                .chain(\n                    allocation_record_tree_nodes\n                        .iter()\n                        .map(|pointer| (pointer, TransactionUnit::AllocationTree)),\n                )	                .chain([(&root.tree_table, TransactionUnit::TreeTable)])	-p singlefs-harness --test second_transaction_supplement_two_instance_table_chain -- rollback_isolates_every_page	rollback_isolates_every_page_of_the_instance_table_chain_that_only_abandoned_roots_reference
 增补 2 收口表第 58 行：暖机第二次空发布失败时 core 不交第一次已落盘的账	crates/singlefs-core/src/transaction.rs	            writes_of_persisted_publishes: writes.clone(),	            writes_of_persisted_publishes: Vec::new(),	-p singlefs-harness --bin first_transaction_on_device -- first_transaction_path_failures	first_transaction_path_failures_report_every_publish_of_the_failed_step_and_they_equal_the_device_layer_count
 增补 2 收口表第 58 行：第一个事务那条路停在暖机时不转交已落盘那几次的账	crates/singlefs-harness/src/scenario.rs	                writes_of_persisted_publishes: failed.writes_of_persisted_publishes,	                writes_of_persisted_publishes: Vec::new(),	-p singlefs-harness --bin first_transaction_on_device -- first_transaction_path_failures	first_transaction_path_failures_report_every_publish_of_the_failed_step_and_they_equal_the_device_layer_count
 增补 2 收口表第 58 行：可写挂载在取号之后的发布里失败时 core 不交写入口的失败账	crates/singlefs-core/src/mount.rs	        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),	        writes_of_failed_publishes: Vec::new(),	-p singlefs-harness --bin first_transaction_on_device -- failed_writable_mount	failed_writable_mount_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count
@@ -494,9 +494,9 @@
 增补 2 收口表第 58 行（`raise_rollback_floor` 同形）：抬 F 那一串失败时 core 不交已落盘那几次空发布的账；真设备二进制那一侧的判法（设备一层逐项相等）判出	crates/singlefs-core/src/mount.rs	                publishes\n                    .iter()\n                    .map(|persisted: &TransactionOutput| persisted.writes.clone())\n                    .collect(),	                Vec::new(),	-p singlefs-harness --bin first_transaction_on_device -- failed_raise	failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count
 增补 2 收口表第 58 行（`raise_rollback_floor` 同形）：抬 F 那一串第二次空发布被落点拒绝时报出的已落盘份数成了 0（接替原第 471 行：已落盘次数改成随账交出，原文命中 0 次）	crates/singlefs-core/src/mount.rs	                publishes\n                    .iter()\n                    .map(|persisted: &TransactionOutput| persisted.writes.clone())\n                    .collect(),	                Vec::new(),	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- a_raise_whose_second_empty_publish_is_refused	a_raise_whose_second_empty_publish_is_refused_reports_that_one_publish_of_the_sequence_persisted
 增补 2 收口表第 58 行（`raise_rollback_floor` 同形）：抬 F 那一串失败时 core 不交写入口的失败账（与可写挂载共用那一处）	crates/singlefs-core/src/mount.rs	        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),	        writes_of_failed_publishes: Vec::new(),	-p singlefs-harness --bin first_transaction_on_device -- failed_raise	failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count
-增补 2 收口表第 39 行那一族：取号之前在分配器的拷贝上取不到落点也照样取号（240 槽小盘上可写挂载取号之后才被落点拒绝、盘上已经写了）	crates/singlefs-core/src/mount.rs	        PlacementsOnTheCopy::Refused(refused) => {\n            return Err(\n                MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {\n                    instance_to_acquire,\n                    publish_index: refused.publish_index,\n                    warm_up_publishes_planned: warm_up_publishes_planned.len(),\n                    unit: refused.unit,\n                    refusal: refused.refusal,\n                },\n            )\n        }	        PlacementsOnTheCopy::Refused(_) => None,	-p singlefs-harness --test second_transaction_supplement_three_random_history -- a_writable_mount_whose_own_publishes	a_writable_mount_whose_own_publishes_find_no_placement_is_refused_before_acquisition_with_the_disk_unchanged
-增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时不记根（不转环、不回收），拷贝上被拒的那一次与真发的不同	crates/singlefs-core/src/mount.rs	        copy.record_root_written_by_this_process(txg);	        // 变异：拷贝上不记根（不转环、不回收）	-p singlefs-harness --test second_transaction_supplement_three_random_history -- a_writable_mount_whose_own_publishes	a_writable_mount_whose_own_publishes_find_no_placement_is_refused_before_acquisition_with_the_disk_unchanged
-增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时不释放换下的落点（回退到环里最旧的根时写行当场回收的几片拷贝上看不见，真发得出来的回退被拒）	crates/singlefs-core/src/mount.rs	            copy.release(placement, txg);	            let _ = (placement, txg);	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_the_oldest_ring_root	rolling_back_to_the_oldest_ring_root_reuses_what_the_row_publish_released_and_is_not_refused_before_acquisition
+增补 2 收口表第 39 行那一族：取号之前的预演取不到落点也照样取号（240 槽小盘上可写挂载取号之后才被落点拒绝、盘上已经写了）	crates/singlefs-core/src/mount.rs	        Err(DryRunRefusal {\n            publish_index,\n            cause: PublishError::PlacementRefused { unit, refusal },\n        }) => {\n            return Err(\n                MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {\n                    instance_to_acquire,\n                    publish_index,\n                    warm_up_publishes_planned: warm_up_publishes_planned.len(),\n                    unit,\n                    refusal,\n                },\n            )\n        }	        Err(DryRunRefusal {\n            publish_index: _,\n            cause: PublishError::PlacementRefused { .. },\n        }) => None,	-p singlefs-harness --test second_transaction_supplement_three_random_history -- a_writable_mount_whose_own_publishes	a_writable_mount_whose_own_publishes_find_no_placement_is_refused_before_acquisition_with_the_disk_unchanged
+增补 2 收口表第 39 行那一族：发布取完落点之后不记根（不转环、不回收；取号之前的预演与真发走同一段，两边一起不回收）	crates/singlefs-core/src/transaction.rs	    allocator.record_root_written_by_this_process(txg);\n    let slot_of = |identity: TransactionUnit| slots[&identity];	    let slot_of = |identity: TransactionUnit| slots[&identity];	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_the_oldest_ring_root	rolling_back_to_the_oldest_ring_root_reuses_what_the_row_publish_released_and_is_not_refused_before_acquisition
+增补 2 收口表第 39 行那一族：发布不释放换下的落点（取号之前的预演与真发走同一段；回退到环里最旧的根时写行当场回收的几片哪一边都看不见）	crates/singlefs-core/src/transaction.rs	        allocator.release_leaving_the_record_allocated_on(\n            *placement,\n            txg,\n            &devices_whose_copy_failed_the_checksum,\n        );	        let _ = (placement, &devices_whose_copy_failed_the_checksum);	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_the_oldest_ring_root	rolling_back_to_the_oldest_ring_root_reuses_what_the_row_publish_released_and_is_not_refused_before_acquisition
 增补 2 收口表第 39 行那一族：胶水把「取号之前在拷贝上取不到落点」映射成模型没有的理由（单元区墙那一格对不上）	crates/singlefs-harness/src/model_comparison.rs	        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {\n            refusal, ..\n        } => refusal_reason_of_placement_refusal(refusal),	        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. } => {\n            ObservedRefusalReason::Unexplained\n        }	-p singlefs-harness --test second_transaction_supplement_three_random_history -- a_writable_mount_whose_own_publishes	a_writable_mount_whose_own_publishes_find_no_placement_is_refused_before_acquisition_with_the_disk_unchanged
 收口表第 26 行 I-7.9：判法拿掉（抬 F 的根带的 F 高于上限也判成立）	crates/singlefs-checker/src/walk.rs	        judgements.judge("I-7.9", raised_floor <= ceiling.lowest_possible, || {	        judgements.judge("I-7.9", true, || {	-p singlefs-harness --test checker_known_bad_images -- raising_the_floor_above_its_ceiling	raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant
 收口表第 26 行 I-7.9：上限算法改宽（每块盘上最新的有效根与第 4 新的非空有效根取大，不取小）	crates/singlefs-checker/src/walk.rs	    newest_valid_root_txg_on_every_device.min(fourth_newest_non_empty_or_oldest_valid)	    newest_valid_root_txg_on_every_device.max(fourth_newest_non_empty_or_oldest_valid)	-p singlefs-harness --test checker_known_bad_images -- raising_the_floor_above_its_ceiling	raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant
@@ -555,13 +555,13 @@
 实例表第二片写路径：链指针记录「有下一片」写成 0（读者按「无下一片而指针不全零」拒收，两片的表读不出）	crates/singlefs-core/src/instance_table.rs	                writer.put_u8(CHAIN_RECORD_HAS_NEXT_PAGE);	                writer.put_u8(CHAIN_RECORD_NO_NEXT_PAGE);	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- a_row_publish_past_one_page	a_row_publish_past_one_page_writes_the_tail_page_first_and_the_first_page_chains_to_it
 实例表第二片写路径（每次写行 COW 重写整条链）：被换下的旧链只释放第 0 片（第 1 片起永远占着）	crates/singlefs-core/src/transaction.rs	    replaced_chain\n        .iter()\n        .enumerate()	    replaced_chain\n        .iter()\n        .take(1)\n        .enumerate()	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- the_next_mount_rewrites	the_next_mount_rewrites_the_whole_two_page_chain_and_releases_both_old_pages
 实例表第二片写路径：带文件的一版上写行时不释放被换下的那条旧链	crates/singlefs-core/src/transaction.rs	            let mut chain_then_the_rest =\n                instance_table_chain_to_release(&rewrite.replaced_chain, allocator)?;	            let mut chain_then_the_rest: Vec<Placement> = Vec::new();	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- the_next_mount_rewrites	the_next_mount_rewrites_the_whole_two_page_chain_and_releases_both_old_pages
-实例表第二片写路径：树表 0 条的一版上写行时旧链只释放第 0 片	crates/singlefs-core/src/transaction.rs	    for released in swapped_out.instance_table_chain.iter().copied() {	    for released in swapped_out.instance_table_chain.iter().copied().take(1) {	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- a_version_without_file_past_one_page	a_version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both
+实例表第二片写路径：树表 0 条的一版上写行时旧链只释放第 0 片	crates/singlefs-core/src/transaction.rs	    let chain = instance_table_chain_to_release(&instance_table.replaced_chain, allocator)?;	    let chain: Vec<Placement> =\n        instance_table_chain_to_release(&instance_table.replaced_chain, allocator)?\n            .into_iter()\n            .take(1)\n            .collect();	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- a_version_without_file_past_one_page	a_version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both
 实例表第二片（实四 O3）：I-3.9 的引用集合不认实例表链第 1 片起的落点（旧链第 1 片的已释放记录被跳过，释放代写错不红）	crates/singlefs-checker/src/walk.rs	    if depth == ReferenceScanDepth::EveryReferencedPlacement {\n        note_instance_table_pages_after_the_first(	    if false && depth == ReferenceScanDepth::EveryReferencedPlacement {\n        note_instance_table_pages_after_the_first(	-p singlefs-harness --test checker_known_bad_images -- a_release_generation_outside_its_interval_on_the_second_page	a_release_generation_outside_its_interval_on_the_second_page_of_a_replaced_instance_table_chain_reddens_the_release_generation_invariant
 实例表第二片（理想模型）：写行那次发布只按一片数实例表（多出来的几片不进分配记录与占槽的上界）	crates/singlefs-harness/src/model.rs	            instance_table_pages_for_rows(instance_table_rows.len()) - 1	            instance_table_pages_for_rows(instance_table_rows.len()) - instance_table_pages_for_rows(instance_table_rows.len())	-p singlefs-harness --lib -- model::tests::a_mount_that_writes_more_rows	a_mount_that_writes_more_rows_than_one_page_holds_counts_every_page_of_the_instance_table_chain
 实例表第二片写路径（D18 已定项 11：0 行也是一片）：0 行时切出 0 片（写者按片数取落点、按切片装片，两者不等）	crates/singlefs-core/src/instance_table.rs	    if rows.is_empty() {\n        return vec![rows];\n    }	    if false {\n        return vec![rows];\n    }	-p singlefs-core --lib -- chain_record_tests	rows_fill_a_page_of_three_hundred_sixty_nine_before_the_next_page_opens
 实例表第二片写路径：写行那次只给第 0 片取落点（多于一片的链装不出来，一次挂载写 370 行就失败）	crates/singlefs-core/src/transaction.rs	    (0..pages)\n        .rev()\n	    (0..pages.min(1))\n        .rev()\n	-p singlefs-harness --test second_transaction_supplement_two_instance_table_page_full -- mount_after_crashes	mount_after_crashes_right_after_acquisition_fills_the_page_and_one_more_row_opens_the_second_page
 实例表第二片写路径：写行那次只给第 0 片取落点（回退要写 370 行时失败）	crates/singlefs-core/src/transaction.rs	    (0..pages)\n        .rev()\n	    (0..pages.min(1))\n        .rev()\n	-p singlefs-harness --test second_transaction_supplement_two_instance_table_page_full -- rollback_counts	rollback_counts_the_rows_of_the_table_it_rolls_back_to
-实例表第二片 × 增补 2 收口表第 39 行那一族：发完之后比对时，树表 0 条那一版上写行真取到的落点只认第 0 片（第 1 片起的点名项被错配到别的角色上，挂载自己的断言判出）	crates/singlefs-core/src/mount.rs	            instance_table_page_roles_in_bump_order(instance_table_pages)	            instance_table_page_roles_in_bump_order(1)	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- a_version_without_file_past_one_page	a_version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both
+实例表第二片：树表 0 条的一版上写行只给第 0 片取落点（第 1 片没有落点，装链时查不到）	crates/singlefs-core/src/transaction.rs	    let instance_table_roles =\n        instance_table_page_roles_in_bump_order(instance_table.pages_after_this_publish());	    let instance_table_roles = instance_table_page_roles_in_bump_order(1);	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- a_version_without_file_past_one_page	a_version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both
 E158 root_choice_repair session s9：search_minimum_weight_that_triggers_rootback 权重档边界判定改成中途才停（该在 weight >= effective_ceiling 时停，改成 weight > effective_ceiling——传 Some(0) 时不会在权重 0 停，会多搜一档）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	        if weight >= effective_ceiling {	        if weight > effective_ceiling {	-p singlefs-harness --bin e158_root_choice_repair	search_minimum_weight_that_triggers_rootback_honors_an_explicit_weight_ceiling
 E158 root_choice_repair session s9：mount_writable_trajectory 的故障装配条件改成两个都要满足（该在 step==0 或 persistent 时装故障，改成只在 step==0 且 persistent 时装——瞬时模式第 0 步不再装故障）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	        let active_targets: &[(DeviceIdentity, DeviceOffsetInBytes)] =\n            if step == 0 || persistent {	        let active_targets: &[(DeviceIdentity, DeviceOffsetInBytes)] =\n            if step == 0 && persistent {	-p singlefs-harness --bin e158_root_choice_repair	mount_writable_trajectory_distinguishes_persistent_from_transient_faults
 树分裂 核心规则：叶切在末尾（左半留 n − 1 条）而不是从中间切	crates/singlefs-core/src/code_two_tree.rs	            let right_keys = keys.split_off(keys.len().div_ceil(2));	            let right_keys = keys.split_off(keys.len() - 1);	-p singlefs-core --lib -- code_two_tree::tests::inserting_nine_keys_into_a_leaf_of_eight_splits_it_in_the_middle_and_grows_a_root	code_two_tree::tests::inserting_nine_keys_into_a_leaf_of_eight_splits_it_in_the_middle_and_grows_a_root
@@ -581,23 +581,23 @@
 树分裂 checker：多层码 2 树的条目宽不判 I-1.10	crates/singlefs-checker/src/walk.rs	                    .judge("I-1.10", entry_width == field_table_bytes, || {	                    .judge("I-1.10", true, || {	-p singlefs-harness --test second_transaction_supplement_two_tree_split -- the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root	the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root
 树分裂 映射树内部节点容量按 key + 87 算（143 变 142）	crates/singlefs-core/src/code_two_tree.rs	    key_width_in_bytes + usize::try_from(NODE_POINTER_BYTES).expect("86")\n}	    key_width_in_bytes + usize::try_from(NODE_POINTER_BYTES).expect("86") + 1\n}	-p singlefs-harness --test second_transaction_mapping_node_admission -- the_central_mapping_node_holds_two_hundred_ninety_four_entries_and_an_internal_node_one_hundred_forty_three	the_central_mapping_node_holds_two_hundred_ninety_four_entries_and_an_internal_node_one_hundred_forty_three
 树分裂 叶从中间切时左半留 ⌊n ÷ 2⌋ 条（295 条切成 147 + 148）	crates/singlefs-core/src/code_two_tree.rs	            let right_keys = keys.split_off(keys.len().div_ceil(2));	            let right_keys = keys.split_off(keys.len() / 2);	-p singlefs-harness --test second_transaction_mapping_node_admission -- two_hundred_ninety_five_mapping_keys_grow_the_tree_to_two_levels_instead_of_being_refused	two_hundred_ninety_five_mapping_keys_grow_the_tree_to_two_levels_instead_of_being_refused
-树分裂 取号之前的推算不删记账树节点的旧映射 key（推出来的重写数与真发布的对不上）	crates/singlefs-core/src/transaction.rs	        for key in self.accounting_node_mapping_keys.values() {\n            mapping_keys.remove(key);\n        }		-p singlefs-harness --test second_transaction_mapping_node_admission -- the_rewritten_role_counts_inferred_before_acquisition_equal_the_ones_each_empty_publish_really_rewrites	the_rewritten_role_counts_inferred_before_acquisition_equal_the_ones_each_empty_publish_really_rewrites
+K4：打开文件时按需读 extent 树的节点数多记一倍（实现自报的与块层数到的对不上）	crates/singlefs-core/src/mounted_read.rs	            node_reads += 1;\n            read_mapped_tree_node_via_hint_then_central_mapping(	            node_reads += 2;\n            read_mapped_tree_node_via_hint_then_central_mapping(	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- random_four_kibibyte_reads	random_four_kibibyte_reads_return_the_written_bytes_and_never_scan_the_journal_ring
 树分裂 记账树叶容量按格式多算一条（478：80 块盘切成 240 + 243）	crates/singlefs-core/src/code_two_tree.rs	            leaf_entries: index_node_entry_capacity(key_width_in_bytes, leaf_entry_width_in_bytes),	            leaf_entries: index_node_entry_capacity(key_width_in_bytes, leaf_entry_width_in_bytes) + 1,	-p singlefs-harness --test second_transaction_supplement_two_accounting_node_full -- eightieth_device_splits_the_accounting_tree_into_two_leaves_under_a_root_instead_of_refusing	eightieth_device_splits_the_accounting_tree_into_two_leaves_under_a_root_instead_of_refusing
 树分裂 读树时叶里的条目不收进来（挂载态的映射条目是空的，提示过期的解引用查映射查不到）	crates/singlefs-core/src/code_two_tree.rs	        leaf_entries_in_key_order.extend(node.header.entries.iter().cloned());		-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- a_stale_hint_under_a_two_level_central_mapping_still_costs_three_device_reads_because_the_whole_tree_is_in_the_mount_state	a_stale_hint_under_a_two_level_central_mapping_still_costs_three_device_reads_because_the_whole_tree_is_in_the_mount_state
 树分裂 读树时不判内部条目宽（55 字节的映射条目当 113 字节的内部条目切）	crates/singlefs-core/src/code_two_tree.rs	        if header.entry_width < internal_width {	        if false && header.entry_width < internal_width {	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- a_central_mapping_root_claiming_level_one_over_mapping_entries_is_refused_instead_of_being_read_as_entries	a_central_mapping_root_claiming_level_one_over_mapping_entries_is_refused_instead_of_being_read_as_entries
 树分裂 读树时孩子该有的层级写成父层级（不减一）	crates/singlefs-core/src/code_two_tree.rs	                Some(header.level - 1),	                Some(header.level),	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- central_mapping_grown_into_two_levels_is_read_whole_into_the_mount_state_and_the_file_reads_back	central_mapping_grown_into_two_levels_is_read_whole_into_the_mount_state_and_the_file_reads_back
 树分裂 读树时不核内部节点的子树覆盖区间	crates/singlefs-core/src/code_two_tree.rs	            if first_child.header.smallest_key != header.smallest_key\n                || last_child.header.largest_key != header.largest_key\n            {	            if false {	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- a_two_level_central_mapping_whose_root_header_does_not_cover_its_leaf_is_refused_by_both_readers	a_two_level_central_mapping_whose_root_header_does_not_cover_its_leaf_is_refused_by_both_readers
 树分裂 层 0：冷走读核映射条目数时记账树按一个节点算（多层记账树的镜像走读失败）	crates/singlefs-core/src/recovery.rs	        + roots.accounting.version.node_count()	        + 1	-p singlefs-harness --test second_transaction_supplement_two_tree_split_layer0 -- every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes	every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes
-树分裂 树高从根节点头读成层级（不加一，D8 已定项 11 ⑤ / D28 已定项 4）	crates/singlefs-core/src/transaction.rs	        u64::from(root_header.level) + 1	        u64::from(root_header.level)	-p singlefs-harness --test second_transaction_supplement_two_tree_split_layer0 -- the_accounting_streams_read_the_tree_height_from_the_root_node_header	the_accounting_streams_read_the_tree_height_from_the_root_node_header
+树分裂 树高从根节点头读成层级（不加一，D8 已定项 11 ⑤ / D28 已定项 4）	crates/singlefs-core/src/transaction.rs	        u64::from(root_header.level) + 1\n    }	        u64::from(root_header.level)\n    }	-p singlefs-harness --test second_transaction_supplement_two_tree_split_layer0 -- the_accounting_streams_read_the_tree_height_from_the_root_node_header	the_accounting_streams_read_the_tree_height_from_the_root_node_header
 释放退回按提示（树分裂换锚点：释放判定路径按「这次换下的上一版角色」查）	crates/singlefs-core/src/transaction.rs	        Some(previous_version) => placements_to_release_via_mapping(\n            previous_version,\n            allocator,\n            &previous_roles_replaced,\n        )?,	        Some(previous_version) => previous_version.placements(),	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_goes_through	release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released
 步 1 变异：不释放旧落点（树分裂换锚点）	crates/singlefs-core/src/transaction.rs	        Some(previous_version) => placements_to_release_via_mapping(\n            previous_version,\n            allocator,\n            &previous_roles_replaced,\n        )?,	        Some(previous_version) => Vec::<Placement>::new(),	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
 步 1 变异：defer 行写 0（树分裂换锚点：记账行按行种类取值）	crates/singlefs-core/src/transaction.rs	                PerDeviceAccountingRow::DeferQueueBytes => device_map.deferred_slots() * SLOT_BYTES,	                PerDeviceAccountingRow::DeferQueueBytes => 0,	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
 步 2 变异：已分配行不随分配更新（I-3.1 要红；树分裂换锚点）	crates/singlefs-core/src/transaction.rs	                PerDeviceAccountingRow::AllocatedBytes => device_map.allocated_slots() * SLOT_BYTES,	                PerDeviceAccountingRow::AllocatedBytes => 0,	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
-增补 2 第 20a 行：写行那次发布的准入不在取号之前算（取号之后发布才被拒，实例代号已经烧掉；树分裂换锚点：多带节点容量一个参数）	crates/singlefs-core/src/mount.rs	    refuse_publishes_before_acquisition_that_do_not_pass_admission(\n        &allocator,\n        &start,\n        instance_to_acquire,\n        &instance_table_rewrite,\n        rows_written.len(),\n        warm_up_publishes_planned.len(),\n        pool.code_two_tree_node_capacities(),\n    )?;\n	    // 变异：写行那次发布的准入不在取号之前算\n	-p singlefs-harness --test second_transaction_supplement_two_row_publish_admission -- writable_mount_that_cannot_publish	writable_mount_that_cannot_publish_the_rows_is_refused_before_the_instance_is_acquired
-增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4b）：分配记录过 600 条之后记账「已分配」少记一槽；逼近分配记录墙那一段照跑 checker 判出（树分裂换锚点）	crates/singlefs-core/src/transaction.rs	                PerDeviceAccountingRow::AllocatedBytes => device_map.allocated_slots() * SLOT_BYTES,	                PerDeviceAccountingRow::AllocatedBytes => (device_map.allocated_slots() - u64::from(allocator.records().len() > 600)) * SLOT_BYTES,	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_wall_sampling	allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count
+增补 2 第 20a 行：取号之前的预演报了写行那次的错也照样取号（取号之后发布才被拒，实例代号已经烧掉）	crates/singlefs-core/src/mount.rs	        Err(DryRunRefusal {\n            publish_index: 0,\n            cause,\n        }) => {\n            return Err(MountError::RowPublishAdmissionRefusedBeforeAcquisition {\n                instance_to_acquire,\n                cause,\n            })\n        }	        Err(DryRunRefusal {\n            publish_index: 0,\n            cause: _,\n        }) => None,	-p singlefs-harness --test second_transaction_supplement_two_row_publish_checks_before_acquisition -- a_row_publish_release_check	a_row_publish_release_check_that_fails_on_a_damaged_mapping_entry_refuses_the_mount_before_acquisition
+增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4b）：分配记录过 600 条之后记账「已分配」少记一槽；逼近分配记录墙那一段照跑 checker 判出（树分裂换锚点）	crates/singlefs-core/src/transaction.rs	                PerDeviceAccountingRow::AllocatedBytes => device_map.allocated_slots() * SLOT_BYTES,	                PerDeviceAccountingRow::AllocatedBytes => (device_map.allocated_slots() - u64::from(allocator.records().len() > 600)) * SLOT_BYTES,	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_sampling	allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall
 增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4b 同一处）：分配记录过 600 条之后记账「已分配」少记一槽；直接钉已分配统计的那条用例判出（树分裂换锚点）	crates/singlefs-core/src/transaction.rs	                PerDeviceAccountingRow::AllocatedBytes => device_map.allocated_slots() * SLOT_BYTES,	                PerDeviceAccountingRow::AllocatedBytes => (device_map.allocated_slots() - u64::from(allocator.records().len() > 600)) * SLOT_BYTES,	-p singlefs-harness --test second_transaction_step_one_overwrite -- allocated_statistic_equals	allocated_statistic_equals_the_span_sum_of_the_allocation_records_past_six_hundred_records
-增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1c；要带调试符号的构建，函数改名会悄悄失效）：只在抬 F 路径的准入里多算一个角色；抬 F 逼近墙的写死用例判出（树分裂换锚点：准入按重写角色数算）	crates/singlefs-core/src/transaction.rs	        records_before_this_publish + rewritten_role_count * allocator.devices.len();	        records_before_this_publish + (rewritten_role_count + usize::from(std::backtrace::Backtrace::force_capture().to_string().contains("raise_rollback_floor"))) * allocator.devices.len();	-p singlefs-harness --test second_transaction_supplement_three_random_history -- raising_the_floor_with_a_second_empty_publish	raising_the_floor_with_a_second_empty_publish_that_fills_the_allocation_node_to_exactly_812_records_succeeds
-增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1d；要带调试符号的构建，函数改名会悄悄失效）：只在回退路径的准入里多算一个角色；回退逼近墙的写死用例判出（树分裂换锚点：准入按重写角色数算）	crates/singlefs-core/src/transaction.rs	        records_before_this_publish + rewritten_role_count * allocator.devices.len();	        records_before_this_publish + (rewritten_role_count + usize::from(std::backtrace::Backtrace::force_capture().to_string().contains("mount_rollback"))) * allocator.devices.len();	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_a_root_whose_warm_up	rolling_back_to_a_root_whose_warm_up_fills_the_allocation_node_to_exactly_812_records_succeeds
+K2：extent 上段叶条目内联那一种的标签写成 3（格式登记的是 2）	crates/singlefs-core/src/extent_tree.rs	pub const EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT: u8 = 2;	pub const EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT: u8 = 3;	-p singlefs-harness --test first_transaction_step_five_publish -- inode_and_extent_lookups	inode_and_extent_lookups_from_the_root_read_the_first_file_back
+K1 × checker：分配记录树根按位置规定罩的那一段上端写成 0xFE…（checker 自己那份几何算错，第一个事务的节点头对不上它）	crates/singlefs-checker/src/position_addressed.rs	        AllocationRecordTreeCell::Root => (vec![0u8; 10], vec![0xFFu8; 10]),	        AllocationRecordTreeCell::Root => (vec![0u8; 10], vec![0xFEu8; 10]),	-p singlefs-harness --test first_transaction_step_five_publish -- every_index_node_self_checks	every_index_node_self_checks_with_tight_ascending_keys_and_merkle_checksums_hold
 普查 R2：checker 走读中央映射条目之前不判条目宽（walk.rs 又切 entry[27..55]，checker 自己倒下；树分裂换锚点：多层码 2 树按层守条目宽）	crates/singlefs-checker/src/walk.rs	                if view.entry_width < bytes_the_walk_needs {	                if false && view.entry_width < bytes_the_walk_needs {	-p singlefs-harness --test second_transaction_supplement_three_bad_disk_input -- every_fixed_panic_site	every_fixed_panic_site_reports_its_own_error_member_instead_of_panicking
 C511 第 3 步：checker 对中央映射树根的 I-1.3 退回写死树 ID 15，不按根记录里那条根指针的出生树判（回退之后重新发号的映射树判不了；树分裂换锚点）	crates/singlefs-checker/src/walk.rs	            tree: parse_node_pointer(mapping_root_pointer).birth_tree,	            tree: 15,	-p singlefs-harness --test checker_known_bad_images -- a_central_mapping_root_whose_header_tree_differs	a_central_mapping_root_whose_header_tree_differs_from_its_root_pointer_birth_tree_reddens_only_the_tree_identifier_invariant
 增补 2 第 9 行 P1（C497）：带文件的一版上重写实例表的发布把实例表挪到树表单元之后取落点（树分裂换锚点：角色清单在两棵多层树的节点之后才收尾）	crates/singlefs-core/src/transaction.rs	        rewritten_roles.push(TransactionUnit::TreeTable);\n        Ok(ResolvedPublish {	        rewritten_roles.push(TransactionUnit::TreeTable);\n        if let Some(position) = rewritten_roles.iter().position(|role| *role == TransactionUnit::InstanceTable) {\n            let moved = rewritten_roles.remove(position);\n            rewritten_roles.push(moved);\n        }\n        Ok(ResolvedPublish {	-p singlefs-harness --test second_transaction_supplement_two_presumed_clause_checks -- c497_every_publish	c497_every_publish_that_rewrites_the_instance_table_bumps_it_before_every_other_commit_generated_block
@@ -605,13 +605,13 @@
 并行线一：映射条目不算数据单元（多单元文件的映射 key 与 resolve 预先算的对不上；树分裂换锚点：映射条目数由这一版每个进映射单元的 key 定）	crates/singlefs-core/src/transaction.rs	                for transaction in self.file_content_transactions() {	                for transaction in self.file_content_transactions().into_iter().take(1) {	-p singlefs-harness --test second_transaction_parallel_line_one_sequential_write -- a_publish_of_two_data_units_names_the_shared_commit_generated_units_only_in_its_last_record	a_publish_of_two_data_units_names_the_shared_commit_generated_units_only_in_its_last_record
 代码三方 m2-wave3-code-r1 第四节第 4 条：只读挂载认中央映射树的根退回写死树 ID 15，不按根记录里映射根指针的出生树（回退之后再发的第一个文件版本映射树是 23，打不开；树分裂换锚点）	crates/singlefs-core/src/mounted_read.rs	        &MultiLevelCodeTwoTree::CentralMapping.read_expectation(root.mapping_root.head.birth_tree),	        &MultiLevelCodeTwoTree::CentralMapping.read_expectation(TreeIdentifier(15)),	-p singlefs-harness --test second_transaction_step_four_rollback -- the_read_only_mount_after_rolling_back	the_read_only_mount_after_rolling_back_to_a_warm_up_root_finds_the_central_mapping_under_the_tree_its_root_pointer_names
 树分裂 规划删不掉上一版的 key 也照常往下走（从盘上重建的映射树分隔 key 坏了，取号之前不拒）	crates/singlefs-core/src/code_two_tree.rs	        return Err(CodeTwoTreeRefusal::PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt);	        return Ok(());	-p singlefs-harness --test second_transaction_supplement_two_tree_split -- a_rebuilt_central_mapping_root_whose_separator_hides_a_key_is_refused_before_the_instance_generation_is_acquired	a_rebuilt_central_mapping_root_whose_separator_hides_a_key_is_refused_before_the_instance_generation_is_acquired
-增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，树表 0 条那一版上写行只取实例表、漏了分配记录树节点（拷贝上取的与真发的不同，挂载自己的断言判出；树分裂换锚点：每一次的角色表按次序排好）	crates/singlefs-core/src/mount.rs	                            .into_iter()\n                            .chain(std::iter::once(TransactionUnit::AllocationTree))\n                            .collect()\n                        } else {	                            .into_iter()\n                            .collect()\n                        } else {	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_formatted_pool_mounted_twice	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
-树分裂 取号之前在拷贝上取落点时按固定的四个固定点角色推（不按两棵多层树的形状，多层池挂载时拷贝上取的与真发的对不上，挂载自己的断言判出）	crates/singlefs-core/src/mount.rs	                        rewritten_roles_of_a_publish_without_content(*shape, &nodes.rewritten_roles)	                        { let _ = nodes; shape.rewritten_roles() }	-p singlefs-harness --test second_transaction_supplement_two_tree_split -- a_pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on	a_pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on
+K2：extent 下段叶里的记录不判 offset 段是不是净荷容量的整数倍（单元序号写进 offset 段的镜像报的不是位置那一条）	crates/singlefs-core/src/extent_tree.rs	            if locality != 0 || record_inode != inode || !offset.is_multiple_of(payload) {	            if locality != 0 || record_inode != inode {	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- a_mount_state_open_refuses_an_extent_key	a_mount_state_open_refuses_an_extent_key_whose_offset_segment_is_the_unit_index_instead_of_the_file_byte_offset
+K1：从盘上读分配记录树时不判记录落在它所在叶按位置罩的那一段里（被抛弃根那棵账里挪到单元区末尾的记录报成别的成员）	crates/singlefs-core/src/allocation_record_tree.rs	                if self.geometry.leaf_of_slot(record.device, record.slot) != node\n                    || !self.geometry.record_fits_in_its_leaf(&record)\n                {	                if false {	-p singlefs-harness --test second_transaction_step_four_rollback -- an_abandoned_roots_allocation_record_whose_span	an_abandoned_roots_allocation_record_whose_span_runs_past_the_unit_area_is_counted_and_does_not_panic
 树分裂 格式常量字面量改掉一个数：ACCOUNTING_INTERNAL_ENTRY_BYTES（108 写成 109，不等于记账 key 22 + 子指针 86）	crates/singlefs-format/src/lib.rs	pub const ACCOUNTING_INTERNAL_ENTRY_BYTES: u64 = 108;	pub const ACCOUNTING_INTERNAL_ENTRY_BYTES: u64 = 109;	-p singlefs-format --lib -- pointer_record_and_entry_width_literals	pointer_record_and_entry_width_literals_equal_the_field_sums_they_stand_for
 树分裂 格式常量字面量改掉一个数：CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES（113 写成 114，不等于映射 key 27 + 子指针 86）	crates/singlefs-format/src/lib.rs	pub const CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES: u64 = 113;	pub const CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES: u64 = 114;	-p singlefs-format --lib -- pointer_record_and_entry_width_literals	pointer_record_and_entry_width_literals_equal_the_field_sums_they_stand_for
-实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，带文件的一版上写行只按一片取实例表角色（多于一片时拷贝上取的与真发的不同，挂载自己的断言判出）（树分裂换锚点：拷贝上那一串的形状表第 0 次）	crates/singlefs-core/src/mount.rs	                    std::iter::once(PublishShape::row_publish_rewriting_instance_table_pages(\n                        instance_table_rewrite.pages_after_this_publish(),\n                    ))\n                    .chain(std::iter::repeat_n(\n                        PublishShape::EMPTY_PUBLISH,\n                        warm_up_publish_txgs.len(),	                    std::iter::once(PublishShape::ROW_PUBLISH)\n                    .chain(std::iter::repeat_n(\n                        PublishShape::EMPTY_PUBLISH,\n                        warm_up_publish_txgs.len(),	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- a_row_publish_past_one_page	a_row_publish_past_one_page_writes_the_tail_page_first_and_the_first_page_chains_to_it
-实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，树表 0 条那一版上写行只按一片取实例表角色（多于一片时拷贝上取的与真发的不同，挂载自己的断言判出）（树分裂换锚点：每一次的角色表按次序排好）	crates/singlefs-core/src/mount.rs	                            instance_table_page_roles_in_bump_order(\n                                instance_table_rewrite.pages_after_this_publish(),\n                            )\n                            .into_iter()	                            instance_table_page_roles_in_bump_order(1)\n                            .into_iter()	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- a_version_without_file_past_one_page	a_version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both
-实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，带文件的一版上写行不释放被换下的那条实例表旧链（经映射那一路从这一轮起跳过实例表；回退到环里最旧的根时写行当场回收的那一片拷贝上看不见）（树分裂换锚点：第 0 次释放那一段按两棵多层树的形状重排过）	crates/singlefs-core/src/mount.rs	                    let checked = instance_table_chain_to_release(\n                        &instance_table_rewrite.replaced_chain,\n                        &copy,\n                    )\n                    .and_then(|mut released| {\n	                    let checked = Ok::<Vec<Placement>, PublishError>(Vec::new())\n                    .and_then(|mut released| {\n	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_the_oldest_ring_root	rolling_back_to_the_oldest_ring_root_reuses_what_the_row_publish_released_and_is_not_refused_before_acquisition
+K2 × checker：extent 上段节点按位置规定罩的那一段上端多罩一个 inode（checker 自己那份几何算错，第一个事务的上段根兼叶对不上它）	crates/singlefs-checker/src/position_addressed.rs	        extent_key(first.saturating_add(span - 1), u64::MAX),	        extent_key(first.saturating_add(span), u64::MAX),	-p singlefs-harness --test first_transaction_step_five_publish -- every_index_node_self_checks	every_index_node_self_checks_with_tight_ascending_keys_and_merkle_checksums_hold
+K2 × checker：extent 下段节点按位置规定罩的那一段上端少罩最后一个单元的字节（checker 的 I-1.1 在多单元文件上判红）	crates/singlefs-checker/src/position_addressed.rs	            last_unit\n                .saturating_add(1)\n                .saturating_mul(payload)\n                .saturating_sub(1),	            last_unit.saturating_mul(payload),	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- the_lower_extent_segment	the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline
+C544 / Z4-1：重开时从树表 0 条那一版的分配记录树重建账，却不记下那棵树（下一次写行不知道上一版的节点）	crates/singlefs-core/src/mount.rs	    allocator.note_allocation_record_tree_of_the_version_without_file(tree);	    let _ = tree;	-p singlefs-harness --test second_transaction_supplement_two_row_publish_checks_before_acquisition -- row_publishes_on_a_version_without_file	row_publishes_on_a_version_without_file_with_a_sixty_six_page_instance_table_keep_mounting_writable_past_812_allocation_records
 增补 2 收口表第 58 行（真设备抬 F 模式）：模式名 raise-rollback-floor 写成下划线（跑批脚本送来的参数认不回）	crates/singlefs-harness/src/on_device_modes.rs	            OnDeviceRunMode::RaiseRollbackFloor => "raise-rollback-floor",	            OnDeviceRunMode::RaiseRollbackFloor => "raise_rollback_floor",	-p singlefs-harness --lib -- on_device_modes	every_mode_argument_reads_back_as_the_same_mode_and_unknown_text_is_refused
 增补 2 收口表第 58 行（真设备抬 F 模式）：真设备二进制抬 F 窗口按种类那一侧不算那一串空发布	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	        raise_publishes.push(&raise_publish.writes);	        raise_publishes.clear();	-p singlefs-harness --bin first_transaction_on_device -- raise_rollback_floor_mode	raise_rollback_floor_mode_publishes_the_fourth_version_raises_the_floor_to_its_ceiling_above_zero_and_every_window_matches_the_device_layer_count
 增补 2 收口表第 58 行（真设备抬 F 模式）：真设备二进制抬 F 失败时不用抬 F 交回的已落盘账	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	                        writes_of_persisted_publishes,\n                        writes_of_failed_publishes,	                        &[],\n                        writes_of_failed_publishes,	-p singlefs-harness --bin first_transaction_on_device -- failed_raise	failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count
@@ -649,5 +649,18 @@
 实二二三 g（checker I-7.11）：回退目标那一行的 T 要严格小于目标 txg 才算罩得住（回退行 (r_old, T_old) 自己罩不住，健康镜像上红）	crates/singlefs-checker/src/walk.rs	                && row.published_checkpoint_txg <= entry.rollback_target_txg\n	                && row.published_checkpoint_txg < entry.rollback_target_txg\n	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- the_pool_checker_holds_both_witness	the_pool_checker_holds_both_witness_invariants_after_a_rollback
 实二二三 g（回退见证的判法，D23 已定项 14）：(r_old, T_old) ≤ (i, T) 也算抛弃（R_old 自己被抛弃）	crates/singlefs-core/src/rollback_witness.rs	        (self.rollback_target_instance, self.rollback_target_txg) < (instance, checkpoint_txg)\n	        (self.rollback_target_instance, self.rollback_target_txg) <= (instance, checkpoint_txg)\n	-p singlefs-core --lib -- rollback_witness	an_entry_abandons_exactly_the_roots_after_the_rollback_target_and_before_the_new_instance
 实二二三 g（checker I-7.11）：回退到 mkfs 的第 0 代根时也要实例 0 那一行（实例 0 不写行，健康镜像上红）	crates/singlefs-checker/src/walk.rs	        let target_row_covers = entry.rollback_target_instance == 0\n            || instance_table_rows	        let target_row_covers = false\n            || instance_table_rows	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- a_rollback_to_the_make_filesystem_root	a_rollback_to_the_make_filesystem_root_holds_the_witness_table_invariant_without_a_row_for_instance_zero
-实二二三 h（D18 已定项 11 第五个合取）：取号之前的预演不查写行那次经映射换下的映射条目的位置项（坏映射条目在取号之后才报，号烧掉）	crates/singlefs-core/src/mount.rs	                        refuse_mapping_entries_that_do_not_name_two_pool_devices(\n                            output,\n                            &released_roles_of_the_row_publish,\n                            &pool_devices,\n                        )?;\n	                        let _ = &pool_devices;\n	-p singlefs-harness --test second_transaction_supplement_two_row_publish_checks_before_acquisition -- a_row_publish_release_check	a_row_publish_release_check_that_fails_on_a_damaged_mapping_entry_refuses_the_mount_before_acquisition
-实二二三 h（Z4-1）：树表 0 条的一版上写行的释放核与条数准入不在取号之前判（装不下时取号之后才拒，每试一次烧一个号）	crates/singlefs-core/src/mount.rs	    if let (PreviousVersion::WithoutFile { root, .. }, true) =\n        (&start.previous, row_publish_rewrites_the_instance_table)\n	    if let (PreviousVersion::WithoutFile { root, .. }, true) =\n        (&start.previous, false && row_publish_rewrites_the_instance_table)\n	-p singlefs-harness --test second_transaction_supplement_two_row_publish_checks_before_acquisition -- a_row_publish_on_a_version_without_file	a_row_publish_on_a_version_without_file_that_outgrows_the_allocation_node_is_refused_before_acquisition
+实二二三 h（D18 已定项 11 第五个合取）：取号之前的预演不查写行那次经映射换下的映射条目的位置项（坏映射条目在取号之后才报，号烧掉）	crates/singlefs-core/src/transaction.rs	            refuse_mapping_entries_that_do_not_name_two_pool_devices(\n                previous_version,\n                &settled.previous_roles_replaced,\n                &device_identities_of_the_accounting_rows,\n            )?;	            let _ = &device_identities_of_the_accounting_rows;	-p singlefs-harness --test second_transaction_supplement_two_row_publish_checks_before_acquisition -- a_row_publish_release_check	a_row_publish_release_check_that_fails_on_a_damaged_mapping_entry_refuses_the_mount_before_acquisition
+C544 / Z4-1 × K1：分配记录按叶分装时按槽号除以叶宽取余（记录装进不罩它的叶，下一次重开读不回那棵树）	crates/singlefs-core/src/allocation_record_tree.rs	            .entry(geometry.leaf_of_slot(record.device, record.slot))	            .entry(geometry.leaf_of_slot(\n                record.device,\n                SlotNumber(record.slot.0 % ALLOCATION_RECORD_TREE_LEAF_SLOTS),\n            ))	-p singlefs-harness --test second_transaction_supplement_two_row_publish_checks_before_acquisition -- row_publishes_on_a_version_without_file	row_publishes_on_a_version_without_file_with_a_sixty_six_page_instance_table_keep_mounting_writable_past_812_allocation_records
+K1 / K2 × D28 已定项 4：按 key 空间定形状的两棵树的高从根节点头读成层级（不加一）	crates/singlefs-core/src/transaction.rs	            u64::from(root_header.level) + 1\n        };	            u64::from(root_header.level)\n        };	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- the_lower_extent_segment	the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline
+K1 单测：分配记录树根的层级取「切出来的格数小于 4」的最低一层（4 GiB 两盘多长一层）	crates/singlefs-core/src/allocation_record_tree.rs	                cells <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT	                cells < 4	-p singlefs-core --lib -- allocation_record_tree::tests::two_four_gibibyte_devices	two_four_gibibyte_devices_give_a_tree_of_height_three
+K1 单测：分配记录树根的层级按八倍扇出判装不装得下（1 TiB 两盘少长一层：第 2 层切出 980 格，装得下 1352 就停在第 2 层）	crates/singlefs-core/src/allocation_record_tree.rs	                cells <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT	                cells <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT * 8	-p singlefs-core --lib -- allocation_record_tree::tests::two_one_tebibyte_devices	two_one_tebibyte_devices_give_a_tree_of_height_four
+K1 单测：分配记录树按槽号找叶时叶宽算成两倍（叶的边界不在 812 的整数倍上）	crates/singlefs-core/src/allocation_record_tree.rs	            index_in_device: slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS,	            index_in_device: slot.0 / (2 * ALLOCATION_RECORD_TREE_LEAF_SLOTS),	-p singlefs-core --lib -- allocation_record_tree::tests::leaves_split_the_slots	leaves_split_the_slots_at_multiples_of_the_leaf_width
+K1 单测：记录变了只重写那片叶、不连它的祖先（父节点里指它的那条子指针成了旧的）	crates/singlefs-core/src/allocation_record_tree.rs	            insert_the_node_and_its_ancestors(geometry, leaf, &mut changed);	            changed.insert(leaf);	-p singlefs-core --lib -- allocation_record_tree::tests::a_changed_record_changes	a_changed_record_changes_its_leaf_and_every_ancestor_only
+K1 单测：内部条目的 key 不在孩子那一层的格点上也认成孩子	crates/singlefs-core/src/allocation_record_tree.rs	        if !slot.0.is_multiple_of(child_span) || !self.has_device(device) {	        if !self.has_device(device) {	-p singlefs-core --lib -- allocation_record_tree::tests::an_entry_key_off_the_grid	an_entry_key_off_the_grid_names_no_child
+K2 单测：extent 树下段根的层级取「罩的单元数大于单元数」的最低一层（144 个单元白长一层）	crates/singlefs-core/src/extent_tree.rs	        .find(|level| lower_span_in_data_units(*level) >= data_units)	        .find(|level| lower_span_in_data_units(*level) > data_units)	-p singlefs-core --lib -- extent_tree::tests::a_lower_segment_grows	a_lower_segment_grows_a_level_past_one_hundred_and_forty_four_data_units
+K2 单测：extent 树上段根的层级取「罩的 inode 数不小于最大 inode 号」的最低一层（inode 143 还算在第 0 片叶里）	crates/singlefs-core/src/extent_tree.rs	        .find(|level| upper_span_in_inodes(*level) > largest_inode)	        .find(|level| upper_span_in_inodes(*level) >= largest_inode)	-p singlefs-core --lib -- extent_tree::tests::the_upper_segment_of_inode_one	the_upper_segment_of_inode_one_is_a_single_leaf
+K2 单测：extent 上段叶条目不认识的标签当成「没有单元」	crates/singlefs-core/src/extent_tree.rs	            unrecognized => {\n                return Err(ExtentUpperLeafEntryMalformed::UnrecognizedTag(unrecognized))	            _unrecognized => {\n                ExtentUpperLeafTarget::NoDataUnit	-p singlefs-core --lib -- extent_tree::tests::upper_leaf_entries_round_trip	upper_leaf_entries_round_trip_and_refuse_an_unknown_tag
+K1 × checker 单测：checker 那份分配记录树根的层级按四倍扇出判装不装得下	crates/singlefs-checker/src/position_addressed.rs	            <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT\n    })	            <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT * 4\n    })	-p singlefs-checker --lib -- position_addressed::tests::the_root_level_follows	the_root_level_follows_the_device_sizes
+K1 × checker 单测：checker 判记录落不落在叶里只看起点、不看末槽	crates/singlefs-checker/src/position_addressed.rs	    device == leaf_device && span_slots > 0 && slot >= first && slot + span_slots - 1 <= last	    device == leaf_device && span_slots > 0 && slot >= first && slot <= last	-p singlefs-checker --lib -- position_addressed::tests::a_record_crossing	a_record_crossing_the_last_slot_of_its_leaf_does_not_fit
+K1 单测：分配记录树根之下的节点的步号把盘与层级写反	crates/singlefs-core/src/transaction.rs	                "t5@{}.{}.{}",	                "t5@{1}.{0}.{2}",	-p singlefs-core --lib -- transaction::tests::every_transaction_unit_names	every_transaction_unit_names_its_class_tree_and_placement_rule
+K2 × 格式常量：extent 上段一片叶罩的 inode 数写成 142（与 (16384 − 163) ÷ 113 对不上）	crates/singlefs-format/src/lib.rs	pub const EXTENT_TREE_UPPER_LEAF_INODES: u64 = 143;	pub const EXTENT_TREE_UPPER_LEAF_INODES: u64 = 142;	-p singlefs-format --lib -- widths_match_the_first_transaction_byte_table	widths_match_the_first_transaction_byte_table
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-checker/src/lib.rs tree/crates/singlefs-checker/src/lib.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-checker/src/lib.rs	2026-09-24 18:48:58.241397937 +0000
+++ tree/crates/singlefs-checker/src/lib.rs	2026-09-24 19:50:56.219113424 +0000
@@ -7,6 +7,7 @@
 #![forbid(unsafe_code)]
 
 pub mod image;
+pub mod position_addressed;
 pub mod walk;
 
 use singlefs_format::{
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-checker/src/position_addressed.rs tree/crates/singlefs-checker/src/position_addressed.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-checker/src/position_addressed.rs	1970-01-01 00:00:00.000000000 +0000
+++ tree/crates/singlefs-checker/src/position_addressed.rs	2026-09-24 20:56:52.992717813 +0000
@@ -0,0 +1,338 @@
+//! 按 key 空间定形状的两棵派生树（分配记录树、extent 树，D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K1 / K2）在 checker 这边的几何：
+//! 只与实现共享格式常量模块（D13（验证路线） 已定项 5），几何、key 的字节、位置怎么算按条款另写一份，不用实现的解析。
+//!
+//! - 分配记录树：叶 k 罩一块盘上的槽 `[k × W, (k + 1) × W)`，层级 L 的节点罩 `W × F^L` 个槽（W = 812、F = 169）；根罩整个 key 空间，
+//!   它的层级是最小的 R ≥ 1 使 Σ_盘 ⌈盘上槽数 ÷ W·F^(R−1)⌉ ≤ F。节点头的 key 区间写这个节点按位置规定罩的那一段
+//!   （D18（块里携带什么信息） 已定项 2 对按位置寻址的树那一句）；父条目的 key 是孩子那一段的起点。
+//! - extent 树：上段按 inode 号的位置（叶罩 143 个 inode 号、内部扇出 147），下段一个文件一棵按数据单元号的位置（叶罩 144 个单元、扇出 147），
+//!   上段叶条目 113 带标签 0 / 1 / 2（没有单元 / 下段根指针 / 内联数据指针）。
+
+use singlefs_format::{
+    ALLOCATION_RECORD_TREE_INTERNAL_FANOUT, ALLOCATION_RECORD_TREE_LEAF_SLOTS, DATA_POINTER_BYTES,
+    DATA_UNIT_BYTES, DATA_UNIT_PAYLOAD_OFFSET, EXTENT_TREE_INTERNAL_FANOUT,
+    EXTENT_TREE_LOWER_LEAF_DATA_UNITS, EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES,
+    EXTENT_TREE_UPPER_LEAF_INODES, NODE_POINTER_BYTES,
+};
+
+use crate::{read_six_byte_unsigned, read_u32, read_u64};
+
+/// 分配记录树 key 里槽号那一段是 6 字节：一块盘上最大的槽号。
+const LARGEST_SLOT_NUMBER: u64 = (1 << 48) - 1;
+
+/// 一个数据单元装多少字节用户数据（32768 − 含预留位的头 134）：extent key 的 offset 段是文件字节偏移（D8（核心索引结构） 已定项 3），
+/// 第 n 个单元是 n × 它。
+#[must_use]
+pub fn data_unit_payload_capacity_in_bytes() -> u64 {
+    DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET
+}
+
+/// 分配记录树层级 L 上一个节点罩几个槽：W × F^L，乘到装不进 u64 就停在最大值。
+#[must_use]
+pub fn allocation_record_tree_span_in_slots(level: u8) -> u64 {
+    (0..level).fold(ALLOCATION_RECORD_TREE_LEAF_SLOTS, |span, _| {
+        span.saturating_mul(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT)
+    })
+}
+
+/// 这个池的分配记录树根该在哪一层（`device_slots` 是池里每块盘的槽数）；盘多到连 255 层都装不下时 `None`。
+#[must_use]
+pub fn allocation_record_tree_root_level(device_slots: &[u64]) -> Option<u8> {
+    (1..=u8::MAX).find(|root_level| {
+        let child_span = allocation_record_tree_span_in_slots(root_level - 1);
+        device_slots
+            .iter()
+            .map(|slots| slots.div_ceil(child_span))
+            .sum::<u64>()
+            <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
+    })
+}
+
+/// 分配记录树里一个节点的位置：根，或根之下 (层级, 盘, 同盘同层序号)。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub enum AllocationRecordTreeCell {
+    Root,
+    BelowTheRoot { level: u8, device: u32, index: u64 },
+}
+
+/// 分配记录树 key（设备 4, 槽号 6，小端）的字节。
+#[must_use]
+pub fn allocation_record_key(device: u32, slot: u64) -> Vec<u8> {
+    let mut key = device.to_le_bytes().to_vec();
+    key.extend_from_slice(&slot.to_le_bytes()[..6]);
+    key
+}
+
+/// 一个分配记录树节点按位置规定罩的那一段，(最小, 最大) 两把 key 的字节。
+#[must_use]
+pub fn allocation_record_tree_cell_range(cell: AllocationRecordTreeCell) -> (Vec<u8>, Vec<u8>) {
+    match cell {
+        AllocationRecordTreeCell::Root => (vec![0u8; 10], vec![0xFFu8; 10]),
+        AllocationRecordTreeCell::BelowTheRoot {
+            level,
+            device,
+            index,
+        } => {
+            let span = allocation_record_tree_span_in_slots(level);
+            let first = index.saturating_mul(span).min(LARGEST_SLOT_NUMBER);
+            let last = index
+                .saturating_mul(span)
+                .saturating_add(span - 1)
+                .min(LARGEST_SLOT_NUMBER);
+            (
+                allocation_record_key(device, first),
+                allocation_record_key(device, last),
+            )
+        }
+    }
+}
+
+/// 父节点（层级 `parent_level`；根时 `parent` 是 `Root`）里一条内部条目的 key 指的孩子：槽号落在孩子那一层的格点上、盘在池里；
+/// 父节点在根之下时孩子与它同盘、落在它那一段里。对不上交回 `None`。
+#[must_use]
+pub fn allocation_record_tree_child_of_entry_key(
+    parent: AllocationRecordTreeCell,
+    parent_level: u8,
+    entry_key: &[u8],
+    pool_devices: &[u32],
+) -> Option<AllocationRecordTreeCell> {
+    let child_level = parent_level.checked_sub(1)?;
+    let device = read_u32(entry_key, 0);
+    let slot = read_six_byte_unsigned(entry_key, 4);
+    let child_span = allocation_record_tree_span_in_slots(child_level);
+    if !slot.is_multiple_of(child_span) || !pool_devices.contains(&device) {
+        return None;
+    }
+    let index = slot / child_span;
+    match parent {
+        AllocationRecordTreeCell::Root => {}
+        AllocationRecordTreeCell::BelowTheRoot {
+            device: parent_device,
+            index: parent_index,
+            ..
+        } => {
+            if device != parent_device
+                || index / ALLOCATION_RECORD_TREE_INTERNAL_FANOUT != parent_index
+            {
+                return None;
+            }
+        }
+    }
+    Some(AllocationRecordTreeCell::BelowTheRoot {
+        level: child_level,
+        device,
+        index,
+    })
+}
+
+/// 一条分配记录落不落在这片叶里：同盘、起点槽与末槽都在叶那一段里（D8（核心索引结构） 已定项 14「记录的末槽不越过它所在叶的末槽」）。
+#[must_use]
+pub fn allocation_record_fits_in_the_leaf(
+    leaf: AllocationRecordTreeCell,
+    device: u32,
+    slot: u64,
+    span_slots: u64,
+) -> bool {
+    let AllocationRecordTreeCell::BelowTheRoot {
+        level: 0,
+        device: leaf_device,
+        index,
+    } = leaf
+    else {
+        return false;
+    };
+    let first = index.saturating_mul(ALLOCATION_RECORD_TREE_LEAF_SLOTS);
+    let last = first.saturating_add(ALLOCATION_RECORD_TREE_LEAF_SLOTS - 1);
+    device == leaf_device && span_slots > 0 && slot >= first && slot + span_slots - 1 <= last
+}
+
+/// extent 树上段层级 L 上一个节点罩几个 inode 号。
+#[must_use]
+pub fn extent_upper_span_in_inodes(level: u8) -> u64 {
+    (0..level).fold(EXTENT_TREE_UPPER_LEAF_INODES, |span, _| {
+        span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
+    })
+}
+
+/// extent 树下段层级 L 上一个节点罩几个数据单元。
+#[must_use]
+pub fn extent_lower_span_in_data_units(level: u8) -> u64 {
+    (0..level).fold(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, |span, _| {
+        span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
+    })
+}
+
+/// extent key（locality 8, inode 8, offset 8，小端）的字节。
+#[must_use]
+pub fn extent_key(inode: u64, offset_in_bytes: u64) -> Vec<u8> {
+    let mut key = 0u64.to_le_bytes().to_vec();
+    key.extend_from_slice(&inode.to_le_bytes());
+    key.extend_from_slice(&offset_in_bytes.to_le_bytes());
+    key
+}
+
+/// 上段 (层级, 序号) 按位置规定罩的那一段：`[(0, 首个 inode, 0), (0, 末个 inode, u64 最大)]`。
+#[must_use]
+pub fn extent_upper_cell_range(level: u8, index: u64) -> (Vec<u8>, Vec<u8>) {
+    let span = extent_upper_span_in_inodes(level);
+    let first = index.saturating_mul(span);
+    (
+        extent_key(first, 0),
+        extent_key(first.saturating_add(span - 1), u64::MAX),
+    )
+}
+
+/// 下段 (层级, 序号) 按位置规定罩的那一段：`[(0, inode, 首个单元 × P), (0, inode, (末个单元 + 1) × P − 1)]`。
+#[must_use]
+pub fn extent_lower_cell_range(inode: u64, level: u8, index: u64) -> (Vec<u8>, Vec<u8>) {
+    let span = extent_lower_span_in_data_units(level);
+    let payload = data_unit_payload_capacity_in_bytes();
+    let first_unit = index.saturating_mul(span);
+    let last_unit = first_unit.saturating_add(span - 1);
+    (
+        extent_key(inode, first_unit.saturating_mul(payload)),
+        extent_key(
+            inode,
+            last_unit
+                .saturating_add(1)
+                .saturating_mul(payload)
+                .saturating_sub(1),
+        ),
+    )
+}
+
+/// 上段一个内部节点（层级 `parent_level`、序号 `parent_index`）里一条条目的 key 指的孩子 (层级, 序号)：key 是 (0, 首个 inode, 0)、
+/// 首个 inode 落在孩子那一层的格点上、在父节点那一段里。对不上交回 `None`。
+#[must_use]
+pub fn extent_upper_child_of_entry_key(
+    parent_level: u8,
+    parent_index: u64,
+    entry_key: &[u8],
+) -> Option<(u8, u64)> {
+    let child_level = parent_level.checked_sub(1)?;
+    let (locality, first_inode, offset) = (
+        read_u64(entry_key, 0),
+        read_u64(entry_key, 8),
+        read_u64(entry_key, 16),
+    );
+    let child_span = extent_upper_span_in_inodes(child_level);
+    if locality != 0 || offset != 0 || !first_inode.is_multiple_of(child_span) {
+        return None;
+    }
+    let child_index = first_inode / child_span;
+    (child_index / EXTENT_TREE_INTERNAL_FANOUT == parent_index)
+        .then_some((child_level, child_index))
+}
+
+/// 下段一个内部节点里一条条目的 key 指的孩子 (层级, 序号)：key 是 (0, 这个文件, 孩子那一段的起点偏移)。对不上交回 `None`。
+#[must_use]
+pub fn extent_lower_child_of_entry_key(
+    inode: u64,
+    parent_level: u8,
+    parent_index: u64,
+    entry_key: &[u8],
+) -> Option<(u8, u64)> {
+    let child_level = parent_level.checked_sub(1)?;
+    let (locality, key_inode, offset) = (
+        read_u64(entry_key, 0),
+        read_u64(entry_key, 8),
+        read_u64(entry_key, 16),
+    );
+    let child_offset_span = extent_lower_span_in_data_units(child_level)
+        .saturating_mul(data_unit_payload_capacity_in_bytes());
+    if locality != 0 || key_inode != inode || !offset.is_multiple_of(child_offset_span) {
+        return None;
+    }
+    let child_index = offset / child_offset_span;
+    (child_index / EXTENT_TREE_INTERNAL_FANOUT == parent_index)
+        .then_some((child_level, child_index))
+}
+
+/// 上段叶条目的样子（按字段表另写一份）：inode 号与标签；标签 1 的载荷是节点指针 86（后 2 字节补零），标签 2 的是数据指针 88。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum ExtentUpperLeafEntryView {
+    NoDataUnit {
+        inode: u64,
+    },
+    LowerSegmentRoot {
+        inode: u64,
+        pointer: Vec<u8>,
+    },
+    InlineDataUnit {
+        inode: u64,
+        pointer: Vec<u8>,
+    },
+    /// 条目窄于 113、标签不是 0 / 1 / 2、key 的 locality 或 offset 段不是 0、字段表写零的字节不是零。
+    Malformed,
+}
+
+/// 解一条上段叶条目。
+#[must_use]
+pub fn extent_upper_leaf_entry_view(entry: &[u8]) -> ExtentUpperLeafEntryView {
+    let entry_bytes = usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113");
+    if entry.len() < entry_bytes || read_u64(entry, 0) != 0 || read_u64(entry, 16) != 0 {
+        return ExtentUpperLeafEntryView::Malformed;
+    }
+    let inode = read_u64(entry, 8);
+    let payload = &entry[25..entry_bytes];
+    let node_pointer_bytes = usize::try_from(NODE_POINTER_BYTES).expect("86");
+    let data_pointer_bytes = usize::try_from(DATA_POINTER_BYTES).expect("88");
+    match entry[24] {
+        0 if payload.iter().all(|byte| *byte == 0) => {
+            ExtentUpperLeafEntryView::NoDataUnit { inode }
+        }
+        1 if payload[node_pointer_bytes..].iter().all(|byte| *byte == 0) => {
+            ExtentUpperLeafEntryView::LowerSegmentRoot {
+                inode,
+                pointer: payload[..node_pointer_bytes].to_vec(),
+            }
+        }
+        2 => ExtentUpperLeafEntryView::InlineDataUnit {
+            inode,
+            pointer: payload[..data_pointer_bytes].to_vec(),
+        },
+        _tag_or_padding_the_field_table_does_not_allow => ExtentUpperLeafEntryView::Malformed,
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// 4 GiB 两盘 3 层、1 TiB 两盘 4 层（判决 S1 的本地腿算术），与实现各算各的。
+    #[test]
+    fn the_root_level_follows_the_device_sizes() {
+        let four_gibibytes = (4u64 << 30) / 16384;
+        assert_eq!(
+            allocation_record_tree_root_level(&[four_gibibytes, four_gibibytes]),
+            Some(2)
+        );
+        let one_tebibyte = (1u64 << 40) / 16384;
+        assert_eq!(
+            allocation_record_tree_root_level(&[one_tebibyte, one_tebibyte]),
+            Some(3)
+        );
+    }
+
+    /// 一条跨过叶末槽的记录（起在叶的末槽、跨 2）不落在叶里；起在偶数槽的两槽记录落得进。
+    #[test]
+    fn a_record_crossing_the_last_slot_of_its_leaf_does_not_fit() {
+        let leaf = AllocationRecordTreeCell::BelowTheRoot {
+            level: 0,
+            device: 0,
+            index: 61,
+        };
+        let last_slot_of_the_leaf = 62 * 812 - 1;
+        assert!(!allocation_record_fits_in_the_leaf(
+            leaf,
+            0,
+            last_slot_of_the_leaf,
+            2
+        ));
+        assert!(allocation_record_fits_in_the_leaf(
+            leaf,
+            0,
+            last_slot_of_the_leaf - 1,
+            2
+        ));
+    }
+}
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-checker/src/walk.rs tree/crates/singlefs-checker/src/walk.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-checker/src/walk.rs	2026-09-24 18:48:58.241408187 +0000
+++ tree/crates/singlefs-checker/src/walk.rs	2026-09-24 22:03:26.635379293 +0000
@@ -7,9 +7,19 @@
 use std::collections::{BTreeMap, BTreeSet};
 
 use singlefs_format::{
-    ACCOUNTING_ENTRY_BYTES, ALLOCATION_RECORD_BYTES, DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES,
-    INODE_INTERNAL_ENTRY, JOURNAL_RECORD_BYTES, MAPPING_ENTRY_BYTES, NODE_BYTES,
-    NODE_POINTER_BYTES, SLOT_BYTES, TREE_TABLE_ENTRY_BYTES,
+    ACCOUNTING_ENTRY_BYTES, ALLOCATION_RECORD_BYTES, ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES,
+    DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES, EXTENT_TREE_INTERNAL_ENTRY_BYTES,
+    EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES, INODE_INTERNAL_ENTRY, JOURNAL_RECORD_BYTES,
+    MAPPING_ENTRY_BYTES, NODE_BYTES, NODE_POINTER_BYTES, SLOT_BYTES, TREE_TABLE_ENTRY_BYTES,
+};
+
+use crate::position_addressed::{
+    allocation_record_fits_in_the_leaf, allocation_record_tree_cell_range,
+    allocation_record_tree_child_of_entry_key, allocation_record_tree_root_level,
+    data_unit_payload_capacity_in_bytes, extent_lower_cell_range, extent_lower_child_of_entry_key,
+    extent_lower_span_in_data_units, extent_upper_cell_range, extent_upper_child_of_entry_key,
+    extent_upper_leaf_entry_view, extent_upper_span_in_inodes, AllocationRecordTreeCell,
+    ExtentUpperLeafEntryView,
 };
 
 use crate::image::{
@@ -22,8 +32,8 @@
     back_chain_of_record_header, check_index_node_keys, check_internal_node_separators,
     check_journal_record, check_unit, checksum_field_holds, crc32_castagnoli_table,
     index_node_view, key_schema_for_tree_kind, packed_unit_view, read_six_byte_unsigned, read_u16,
-    read_u32, read_u64, KEY_SCHEMA_ACCOUNTING, KEY_SCHEMA_ALLOCATION, KEY_SCHEMA_MAPPING,
-    KEY_SCHEMA_TREE_TABLE,
+    read_u32, read_u64, KEY_SCHEMA_ACCOUNTING, KEY_SCHEMA_ALLOCATION, KEY_SCHEMA_EXTENT,
+    KEY_SCHEMA_MAPPING, KEY_SCHEMA_TREE_TABLE,
 };
 
 const TREE_KIND_EXTENT: u16 = 1;
@@ -106,6 +116,9 @@
     /// 多层码 2 树（记账树、中央映射树）：叶贴紧首末条目；内部节点的区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2），
     /// 要等孩子读回来才判得了，由走读那一方判（`Walk::walk_code_two_subtree` ③），读节点这一步只判分隔 key 严格递增。
     SubtreeCoverageOfInternalNodesJudgedByTheCaller,
+    /// 按位置寻址的两棵树（分配记录树、extent 树，D8（核心索引结构） 已定项 14）：每个节点的区间是它的位置规定罩的那一段
+    /// （D18（块里携带什么信息） 已定项 2 对按位置寻址的树那一句），不贴紧首末条目，由走读那一方按位置判；读节点这一步只判条目 key 严格递增、节点不空。
+    PrescribedByThePositionJudgedByTheCaller,
 }
 
 /// 多层码 2 树一个节点的条目宽在走读里怎么对待。
@@ -487,7 +500,8 @@
             | (KeyRangeReading::SubtreeCoverageOfInternalNodesJudgedByTheCaller, 0) => {
                 check_index_node_keys(&view, schema)
             }
-            (KeyRangeReading::SubtreeCoverageOfInternalNodesJudgedByTheCaller, _) => {
+            (KeyRangeReading::SubtreeCoverageOfInternalNodesJudgedByTheCaller, _)
+            | (KeyRangeReading::PrescribedByThePositionJudgedByTheCaller, _) => {
                 check_internal_node_separators(&view, schema)
             }
         };
@@ -506,15 +520,11 @@
             is_newest.then_some(mount_root_instance),
         );
         // 树表 0 条那一版的分配记录树的根住根记录（C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案）：
-        // 那一版没有树表条目可放，也还没登记过任何树 ⇒ 那一片的树 ID 是 0（由根记录独占持有，同树表单元与实例表单元）。
-        // 带文件的一版这一项恒全零，`read_index_node` 对全零指针直接交 `None`。
-        self.read_index_node(
-            &record[342..428],
-            0,
-            KEY_SCHEMA_ALLOCATION,
-            "树表 0 条那一版的分配记录树的根",
-            KeyRangeReading::FirstAndLastEntries,
-        );
+        // 那一版没有树表条目可放，也还没登记过任何树 ⇒ 那棵树的树 ID 是 0（由根记录独占持有，同树表单元与实例表单元）。
+        // 按位置寻址、可以多层（D8（核心索引结构） 已定项 14）：整棵走下去。带文件的一版这一项恒全零，全零指针不走。
+        if !parse_node_pointer(&record[342..428]).all_zero {
+            self.walk_allocation_record_tree(&record[342..428], 0, "树表 0 条那一版的分配记录树");
+        }
         // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
         self.walk_tree_table_and_central_mapping_root(&record[36..122], &record[256..342]);
     }
@@ -842,6 +852,19 @@
             }
             return;
         }
+        // 分配记录树与 extent 树按 key 空间定形状（D8（核心索引结构） 已定项 14）：整棵按位置走下去，每个节点核它罩的就是它的位置规定的那一段。
+        if kind == TREE_KIND_ALLOCATION {
+            if !parse_node_pointer(&entry[14..100]).all_zero {
+                self.walk_allocation_record_tree(&entry[14..100], tree, "分配记录树");
+            }
+            return;
+        }
+        if kind == TREE_KIND_EXTENT {
+            if !parse_node_pointer(&entry[14..100]).all_zero {
+                self.walk_extent_upper_node(&entry[14..100], None, tree);
+            }
+            return;
+        }
         let Some(node) = self.read_index_node(
             &entry[14..100],
             tree,
@@ -867,18 +890,8 @@
                 return;
             }
         }
-        match kind {
-            TREE_KIND_INODE => self.walk_inode_root(&node, tree),
-            TREE_KIND_EXTENT | TREE_KIND_ALLOCATION if node.level > 0 => {
-                // 这两棵树按 key 空间另定结构（D8（核心索引结构） 已定项 14，三方 `m2-keyspace-r1`），内部节点今天没有条款：
-                // 走不下去就如实说。
-                self.judgements.not_applicable(
-                    "I-7.2",
-                    "extent / 分配记录树有内部节点，而它们按 key 空间定的结构还没有条款",
-                );
-            }
-            TREE_KIND_EXTENT => self.walk_extent_leaf(&node, tree),
-            _ => {}
+        if kind == TREE_KIND_INODE {
+            self.walk_inode_root(&node, tree);
         }
     }
 
@@ -1193,52 +1206,406 @@
         })
     }
 
-    fn walk_extent_leaf(&mut self, leaf: &crate::IndexNodeView, tree: u64) {
-        for record in &leaf.entries {
-            let inode = read_u64(record, 8);
-            let offset = read_u64(record, 16);
-            let pointer = parse_data_pointer(&record[24..112]);
-            judge_location_order(&mut self.judgements, &pointer, "extent 记录的数据指针");
-            for location in &pointer.locations {
-                self.note_reference(location.device, location.slot, 2, "数据单元");
+    /// extent 树里指向一个数据单元的一条（下段叶的 extent 叶记录，或上段叶条目里内联的那一个）：数据指针 88 字节，
+    /// 所在的 key 是 (0, `inode`, `offset`)。读那个数据单元、判头、出生身份、树 ID 与五元组。
+    fn walk_extent_data_pointer(
+        &mut self,
+        pointer_bytes: &[u8],
+        inode: u64,
+        offset: u64,
+        tree: u64,
+    ) {
+        let pointer = parse_data_pointer(pointer_bytes);
+        judge_location_order(&mut self.judgements, &pointer, "extent 记录的数据指针");
+        for location in &pointer.locations {
+            self.note_reference(location.device, location.slot, 2, "数据单元");
+        }
+        let Some(unit) = read_referenced_unit(
+            self.reader,
+            &mut self.judgements,
+            &pointer.locations,
+            data_unit_bytes(),
+            "数据单元",
+        ) else {
+            self.walk_failures
+                .push("数据单元两份都读不到对得上的".to_string());
+            return;
+        };
+        if !self
+            .visited_units
+            .insert((pointer.locations[0].device, pointer.locations[0].slot))
+            || !self.judge_unit_header(&unit, 1, "数据单元")
+        {
+            return;
+        }
+        self.judge_birth_identity_of_a_referenced_unit(&unit, &pointer, "数据单元");
+        self.judgements
+            .judge("I-1.3", read_u64(&unit, 43) == tree, || {
+                format!(
+                    "数据单元头里的树 ID {} 不是 extent 树 {tree}",
+                    read_u64(&unit, 43)
+                )
+            });
+        let five_tuple_holds = read_u64(&unit, 51) == inode
+            && read_u64(&unit, 67) == offset
+            && read_u64(&unit, 43) == tree;
+        self.judgements.judge("I-1.1", five_tuple_holds, || {
+            format!("数据单元五元组（树 {}、对象 {}、锚点 {}）与 extent key（{tree}, {inode}, {offset}）不符", read_u64(&unit, 43), read_u64(&unit, 51), read_u64(&unit, 67))
+        });
+        self.data_unit_objects.push((
+            inode,
+            read_u64(&unit, 59),
+            format!("inode {inode} 偏移 {offset} 的数据单元"),
+        ));
+    }
+
+    /// 判一个按位置寻址的节点在树里的位置（I-1.1（块头自述逻辑地址）：索引节点的身份是树 ID + 层级 + key 区间，
+    /// 按位置寻址的树的区间是这个节点按位置规定罩的那一段，D18（块里携带什么信息） 已定项 2）与这一层的条目宽（I-1.10）。
+    /// 交回这一支还往不往下走。
+    fn position_and_entry_width_hold(
+        &mut self,
+        view: &crate::IndexNodeView,
+        level: u8,
+        prescribed_range: &(Vec<u8>, Vec<u8>),
+        entry_width: usize,
+        what: &str,
+    ) -> bool {
+        self.judgements.judge("I-1.1", view.level == level, || {
+            format!(
+                "{what}：头里的层级是 {}，它的位置要层级 {level}",
+                view.level
+            )
+        });
+        if view.level != level {
+            self.walk_failures.push(format!(
+                "{what}：层级不是它的位置规定的那一层，这一支走不下去"
+            ));
+            return false;
+        }
+        self.judgements.judge(
+            "I-1.1",
+            view.smallest_key == prescribed_range.0 && view.largest_key == prescribed_range.1,
+            || format!("{what}：头里的 key 区间不是它的位置规定罩的那一段"),
+        );
+        self.judgements
+            .judge("I-1.10", view.entry_width == entry_width, || {
+                format!(
+                    "{what}：头里自述的条目宽 {} 不等于这一层的条目字段表宽度 {entry_width}",
+                    view.entry_width
+                )
+            });
+        view.entry_width == entry_width
+    }
+
+    /// 分配记录树（D8（核心索引结构） 已定项 14：按绝对槽号按位置寻址）整棵走下去：根的层级是池几何定的那一层（checker 自己按每块盘的槽数算），
+    /// 每个节点核层级、它罩的就是它的位置规定的那一段、条目宽（叶 20、内部 96），内部条目的 key 是一个孩子那一段的起点、按位置严格递增，
+    /// 叶里每条记录落在这片叶里（末槽不越过叶的末槽）。
+    fn walk_allocation_record_tree(&mut self, root_pointer_bytes: &[u8], tree: u64, name: &str) {
+        let pool_devices = self.reader.devices();
+        let device_slots: Vec<u64> = pool_devices
+            .iter()
+            .map(|device| self.reader.device_bytes(*device).unwrap_or(0) / SLOT_BYTES)
+            .collect();
+        let Some(root_level) = allocation_record_tree_root_level(&device_slots) else {
+            self.walk_failures.push(format!(
+                "{name}（树 {tree}）：池里的盘多到根装不下，算不出根该在哪一层"
+            ));
+            return;
+        };
+        self.walk_allocation_record_node(
+            root_pointer_bytes,
+            AllocationRecordTreeCell::Root,
+            root_level,
+            tree,
+            name,
+            &pool_devices,
+        );
+    }
+
+    /// 递归的上界：每一层下降层级减一，每个单元只走一次（`visited_units`）。
+    fn walk_allocation_record_node(
+        &mut self,
+        pointer_bytes: &[u8],
+        cell: AllocationRecordTreeCell,
+        level: u8,
+        tree: u64,
+        name: &str,
+        pool_devices: &[u32],
+    ) {
+        let what = match cell {
+            AllocationRecordTreeCell::Root => format!("{name}（树 {tree}）的根"),
+            AllocationRecordTreeCell::BelowTheRoot {
+                level: cell_level,
+                device,
+                index,
+            } => format!("{name}（树 {tree}）层级 {cell_level} 盘 {device} 第 {index} 格的节点"),
+        };
+        let Some(view) = self.read_index_node(
+            pointer_bytes,
+            tree,
+            KEY_SCHEMA_ALLOCATION,
+            &what,
+            KeyRangeReading::PrescribedByThePositionJudgedByTheCaller,
+        ) else {
+            return;
+        };
+        let entry_width = if level == 0 {
+            allocation_record_bytes()
+        } else {
+            usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES).expect("96")
+        };
+        if !self.position_and_entry_width_hold(
+            &view,
+            level,
+            &allocation_record_tree_cell_range(cell),
+            entry_width,
+            &what,
+        ) {
+            return;
+        }
+        if level == 0 {
+            for entry in &view.entries {
+                let record = parse_allocation_record(entry);
+                self.judgements.judge(
+                    "I-1.1",
+                    allocation_record_fits_in_the_leaf(
+                        cell,
+                        record.device,
+                        record.slot,
+                        record.span_slots,
+                    ),
+                    || {
+                        format!(
+                            "{what}：盘 {} 槽 {} 跨 {} 的记录不落在这片叶按位置罩的那一段里，或末槽越过叶的末槽",
+                            record.device, record.slot, record.span_slots
+                        )
+                    },
+                );
             }
-            let Some(unit) = read_referenced_unit(
-                self.reader,
-                &mut self.judgements,
-                &pointer.locations,
-                data_unit_bytes(),
-                "数据单元",
-            ) else {
-                self.walk_failures
-                    .push("数据单元两份都读不到对得上的".to_string());
+            return;
+        }
+        let key_width = KEY_SCHEMA_ALLOCATION.width();
+        let mut previous_child: Option<AllocationRecordTreeCell> = None;
+        for entry in &view.entries {
+            let child = allocation_record_tree_child_of_entry_key(
+                cell,
+                level,
+                &entry[..key_width],
+                pool_devices,
+            )
+            .filter(|child| previous_child.is_none_or(|previous| previous < *child));
+            self.judgements.judge("I-1.1", child.is_some(), || {
+                format!("{what}：一条内部条目的 key 不是这个节点里一个孩子那一段的起点，或孩子不按位置严格递增")
+            });
+            let Some(child) = child else {
+                self.walk_failures.push(format!(
+                    "{what}：内部条目指不到一个合法的孩子，这一支走不下去"
+                ));
                 continue;
             };
-            if !self
-                .visited_units
-                .insert((pointer.locations[0].device, pointer.locations[0].slot))
-                || !self.judge_unit_header(&unit, 1, "数据单元")
-            {
+            previous_child = Some(child);
+            let child_pointer_bytes = &entry[key_width..key_width + node_pointer_bytes()];
+            judge_location_order(
+                &mut self.judgements,
+                &parse_node_pointer(child_pointer_bytes),
+                &format!("{what} 的子指针"),
+            );
+            self.walk_allocation_record_node(
+                child_pointer_bytes,
+                child,
+                level - 1,
+                tree,
+                name,
+                pool_devices,
+            );
+        }
+    }
+
+    /// extent 树上段一个节点连同它下面（D8（核心索引结构） 已定项 14：上段按 inode 号按位置寻址）：`position` 是 (层级, 序号)，
+    /// 根那一个（`None`）的层级取它自己头里的、序号 0。每个节点核层级、位置规定的那一段、条目宽（叶 113、内部 110）；叶条目核标签
+    /// （0 / 1 / 2）、inode 号落在叶罩的那一段里，标签 1 走进那个文件的下段，标签 2 走那个内联的数据单元（key (0, inode, 0)）。
+    fn walk_extent_upper_node(
+        &mut self,
+        pointer_bytes: &[u8],
+        position: Option<(u8, u64)>,
+        tree: u64,
+    ) {
+        let what = match position {
+            None => format!("extent 树（树 {tree}）的根"),
+            Some((level, index)) => {
+                format!("extent 树（树 {tree}）上段层级 {level} 第 {index} 格的节点")
+            }
+        };
+        let Some(view) = self.read_index_node(
+            pointer_bytes,
+            tree,
+            KEY_SCHEMA_EXTENT,
+            &what,
+            KeyRangeReading::PrescribedByThePositionJudgedByTheCaller,
+        ) else {
+            return;
+        };
+        let (level, index) = position.unwrap_or((view.level, 0));
+        let entry_width = if level == 0 {
+            usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113")
+        } else {
+            usize::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110")
+        };
+        if !self.position_and_entry_width_hold(
+            &view,
+            level,
+            &extent_upper_cell_range(level, index),
+            entry_width,
+            &what,
+        ) {
+            return;
+        }
+        let span = extent_upper_span_in_inodes(level);
+        let first_inode = index.saturating_mul(span);
+        let last_inode = first_inode.saturating_add(span - 1);
+        if level == 0 {
+            for entry in &view.entries {
+                let entry_view = extent_upper_leaf_entry_view(entry);
+                let inode_of_the_entry = match &entry_view {
+                    ExtentUpperLeafEntryView::NoDataUnit { inode }
+                    | ExtentUpperLeafEntryView::LowerSegmentRoot { inode, .. }
+                    | ExtentUpperLeafEntryView::InlineDataUnit { inode, .. } => Some(*inode),
+                    ExtentUpperLeafEntryView::Malformed => None,
+                };
+                self.judgements.judge(
+                    "I-1.1",
+                    inode_of_the_entry
+                        .is_some_and(|inode| (first_inode..=last_inode).contains(&inode)),
+                    || {
+                        format!("{what}：一条上段叶条目解不开（标签不是 0 / 1 / 2 或该是零的字节不是零），或它的 inode 号不在这片叶罩的那一段里")
+                    },
+                );
+                match entry_view {
+                    ExtentUpperLeafEntryView::LowerSegmentRoot { inode, pointer } => {
+                        judge_location_order(
+                            &mut self.judgements,
+                            &parse_node_pointer(&pointer),
+                            &format!("{what} 里 inode {inode} 的下段根指针"),
+                        );
+                        self.walk_extent_lower_node(&pointer, None, inode, tree);
+                    }
+                    ExtentUpperLeafEntryView::InlineDataUnit { inode, pointer } => {
+                        self.walk_extent_data_pointer(&pointer, inode, 0, tree);
+                    }
+                    ExtentUpperLeafEntryView::NoDataUnit { .. }
+                    | ExtentUpperLeafEntryView::Malformed => {}
+                }
+            }
+            return;
+        }
+        let key_width = KEY_SCHEMA_EXTENT.width();
+        let mut previous_child: Option<(u8, u64)> = None;
+        for entry in &view.entries {
+            let child = extent_upper_child_of_entry_key(level, index, &entry[..key_width])
+                .filter(|child| previous_child.is_none_or(|previous| previous < *child));
+            self.judgements.judge("I-1.1", child.is_some(), || {
+                format!("{what}：一条内部条目的 key 不是这个节点里一个孩子那一段的起点，或孩子不按位置严格递增")
+            });
+            let Some(child) = child else {
+                self.walk_failures.push(format!(
+                    "{what}：内部条目指不到一个合法的孩子，这一支走不下去"
+                ));
                 continue;
+            };
+            previous_child = Some(child);
+            let child_pointer_bytes = &entry[key_width..key_width + node_pointer_bytes()];
+            judge_location_order(
+                &mut self.judgements,
+                &parse_node_pointer(child_pointer_bytes),
+                &format!("{what} 的子指针"),
+            );
+            self.walk_extent_upper_node(child_pointer_bytes, Some(child), tree);
+        }
+    }
+
+    /// 一个文件 extent 树下段的一个节点连同它下面（按数据单元号按位置寻址）：根那一个（`None`）的层级取它自己头里的、序号 0。
+    /// 每个节点核层级、位置规定的那一段、条目宽（叶 112、内部 110）；叶里每条 extent 叶记录的 inode 段是这个文件、offset 段是
+    /// 单元号 × 净荷容量、落在叶罩的那一段里，再走它指的数据单元。
+    fn walk_extent_lower_node(
+        &mut self,
+        pointer_bytes: &[u8],
+        position: Option<(u8, u64)>,
+        inode: u64,
+        tree: u64,
+    ) {
+        let what = match position {
+            None => format!("extent 树（树 {tree}）inode {inode} 下段的根"),
+            Some((level, index)) => {
+                format!("extent 树（树 {tree}）inode {inode} 下段层级 {level} 第 {index} 格的节点")
             }
-            self.judge_birth_identity_of_a_referenced_unit(&unit, &pointer, "数据单元");
-            self.judgements
-                .judge("I-1.3", read_u64(&unit, 43) == tree, || {
-                    format!(
-                        "数据单元头里的树 ID {} 不是 extent 树 {tree}",
-                        read_u64(&unit, 43)
-                    )
-                });
-            let five_tuple_holds = read_u64(&unit, 51) == inode
-                && read_u64(&unit, 67) == offset
-                && read_u64(&unit, 43) == tree;
-            self.judgements.judge("I-1.1", five_tuple_holds, || {
-                format!("数据单元五元组（树 {}、对象 {}、锚点 {}）与 extent key（{tree}, {inode}, {offset}）不符", read_u64(&unit, 43), read_u64(&unit, 51), read_u64(&unit, 67))
+        };
+        let Some(view) = self.read_index_node(
+            pointer_bytes,
+            tree,
+            KEY_SCHEMA_EXTENT,
+            &what,
+            KeyRangeReading::PrescribedByThePositionJudgedByTheCaller,
+        ) else {
+            return;
+        };
+        let (level, index) = position.unwrap_or((view.level, 0));
+        let entry_width = if level == 0 {
+            extent_leaf_record_bytes()
+        } else {
+            usize::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110")
+        };
+        if !self.position_and_entry_width_hold(
+            &view,
+            level,
+            &extent_lower_cell_range(inode, level, index),
+            entry_width,
+            &what,
+        ) {
+            return;
+        }
+        if level == 0 {
+            let span = extent_lower_span_in_data_units(0);
+            let first_unit = index.saturating_mul(span);
+            let last_unit = first_unit.saturating_add(span - 1);
+            let payload = data_unit_payload_capacity_in_bytes();
+            for record in &view.entries {
+                let (record_inode, offset) = (read_u64(record, 8), read_u64(record, 16));
+                self.judgements.judge(
+                    "I-1.1",
+                    read_u64(record, 0) == 0
+                        && record_inode == inode
+                        && offset.is_multiple_of(payload)
+                        && (first_unit..=last_unit).contains(&(offset / payload)),
+                    || {
+                        format!("{what}：extent 叶记录的 key（inode {record_inode}、偏移 {offset}）不是 (0, 这个文件, 单元号 × 净荷容量)，或不落在这片叶罩的那一段里")
+                    },
+                );
+                self.walk_extent_data_pointer(&record[24..112], record_inode, offset, tree);
+            }
+            return;
+        }
+        let key_width = KEY_SCHEMA_EXTENT.width();
+        let mut previous_child: Option<(u8, u64)> = None;
+        for entry in &view.entries {
+            let child = extent_lower_child_of_entry_key(inode, level, index, &entry[..key_width])
+                .filter(|child| previous_child.is_none_or(|previous| previous < *child));
+            self.judgements.judge("I-1.1", child.is_some(), || {
+                format!("{what}：一条内部条目的 key 不是这个文件里一个孩子那一段的起点，或孩子不按位置严格递增")
             });
-            self.data_unit_objects.push((
-                inode,
-                read_u64(&unit, 59),
-                format!("inode {inode} 偏移 {offset} 的数据单元"),
-            ));
+            let Some(child) = child else {
+                self.walk_failures.push(format!(
+                    "{what}：内部条目指不到一个合法的孩子，这一支走不下去"
+                ));
+                continue;
+            };
+            previous_child = Some(child);
+            let child_pointer_bytes = &entry[key_width..key_width + node_pointer_bytes()];
+            judge_location_order(
+                &mut self.judgements,
+                &parse_node_pointer(child_pointer_bytes),
+                &format!("{what} 的子指针"),
+            );
+            self.walk_extent_lower_node(child_pointer_bytes, Some(child), inode, tree);
         }
     }
 }
@@ -1607,6 +1974,19 @@
     // 不数它，被抛弃根的影子账与 I-3.9 的引用集合就少一片。
     let allocation_record_tree_root = parse_node_pointer(&record[342..428]);
     references.note(&allocation_record_tree_root);
+    // 那棵树按位置寻址、可以多层（D8（核心索引结构） 已定项 14）：根之下的节点只有父节点指着，走到叶那个深度上逐个数进来。
+    if depth == ReferenceScanDepth::EveryReferencedPlacement
+        && !allocation_record_tree_root.all_zero
+    {
+        match allocation_record_tree_without_judging(reader, &allocation_record_tree_root, cache) {
+            Some((node_pointers, _)) => {
+                for pointer in &node_pointers {
+                    references.note(pointer);
+                }
+            }
+            None => references.is_complete = false,
+        }
+    }
     let tree_table_pointer = parse_node_pointer(&record[36..122]);
     references.note(&tree_table_pointer);
     if tree_table_pointer.all_zero {
@@ -1646,7 +2026,14 @@
             references.is_complete = false;
             continue;
         };
-        collect_tree_references(reader, cache, &mut references, &node, read_u16(entry, 10));
+        collect_tree_references(
+            reader,
+            cache,
+            &mut references,
+            &tree_root,
+            &node,
+            read_u16(entry, 10),
+        );
     }
     // 中央映射树根之下的节点（多层时，D8（核心索引结构） 已定项 11）：只有它们的父节点指着，不数它们，这条根的引用集合就少几片。
     // 只在「走到叶」那个深度上数（引用集合只在那个深度上作数）。
@@ -1745,25 +2132,193 @@
     }
 }
 
-/// 一棵树的根节点下面还引用了什么：「extent 叶的数据指针」「inode 树内部条目的子指针」与记账树根之下的全部节点三种下探；
-/// 别的形状（extent 与分配记录树的内部节点、条目格式没有条款的树）走不下去，如实记成数不全。
+/// 一棵分配记录树整棵读、不判（按绝对槽号按位置寻址，D8（核心索引结构） 已定项 14）：交回每个节点的指针（根在内）与全部记录。
+/// 节点读不出、层级不是它的位置规定的那一层、内部条目指不到一个合法的孩子、条目窄于字段表：交回 `None`（数不全；判定归主走读的
+/// I-1.1 / I-1.10 / I-2.1）。迭代上界：每个节点至多进一次（按第一条位置条目记过的不再读），每下一层层级减一。
+fn allocation_record_tree_without_judging(
+    reader: &dyn ImageReader,
+    root_pointer: &PointerView,
+    cache: &mut IndexNodeCache,
+) -> Option<(Vec<PointerView>, Vec<AllocationRecordView>)> {
+    let pool_devices = reader.devices();
+    let device_slots: Vec<u64> = pool_devices
+        .iter()
+        .map(|device| reader.device_bytes(*device).unwrap_or(0) / SLOT_BYTES)
+        .collect();
+    let root_level = allocation_record_tree_root_level(&device_slots)?;
+    let internal_entry_bytes =
+        usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES).expect("96");
+    let key_width = KEY_SCHEMA_ALLOCATION.width();
+    let mut node_pointers = Vec::new();
+    let mut records = Vec::new();
+    let mut seen: BTreeSet<(u32, u64)> = BTreeSet::new();
+    let mut pending = vec![(*root_pointer, AllocationRecordTreeCell::Root, root_level)];
+    while let Some((pointer, cell, level)) = pending.pop() {
+        if pointer.all_zero
+            || !seen.insert((pointer.locations[0].device, pointer.locations[0].slot))
+        {
+            return None;
+        }
+        node_pointers.push(pointer);
+        let node = read_index_node_without_judging(reader, &pointer, cache)?;
+        if node.level != level || node.key_width != key_width {
+            return None;
+        }
+        if level == 0 {
+            if node.entry_width < allocation_record_bytes() {
+                return None;
+            }
+            records.extend(
+                node.entries
+                    .iter()
+                    .map(|entry| parse_allocation_record(entry)),
+            );
+            continue;
+        }
+        if node.entry_width < internal_entry_bytes {
+            return None;
+        }
+        for entry in &node.entries {
+            let child = allocation_record_tree_child_of_entry_key(
+                cell,
+                level,
+                &entry[..key_width],
+                &pool_devices,
+            )?;
+            pending.push((
+                parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]),
+                child,
+                level - 1,
+            ));
+        }
+    }
+    Some((node_pointers, records))
+}
+
+/// 一棵 extent 树整棵读、不判（上段与每个文件的下段，D8（核心索引结构） 已定项 14）：交回每个节点的指针（根在内）与全部数据指针
+/// （下段叶的 extent 叶记录与上段叶条目里内联的那一个）。读不下去的同 [`allocation_record_tree_without_judging`] 交回 `None`。
+fn extent_tree_without_judging(
+    reader: &dyn ImageReader,
+    root_pointer: &PointerView,
+    cache: &mut IndexNodeCache,
+) -> Option<(Vec<PointerView>, Vec<PointerView>)> {
+    /// 待读的一个节点：上段 (层级, 序号)，或某个 inode 下段 (层级, 序号)；根那一个层级取它自己头里的。
+    enum Pending {
+        Upper(PointerView, Option<(u8, u64)>),
+        Lower(PointerView, u64, Option<(u8, u64)>),
+    }
+    let internal_entry_bytes = usize::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110");
+    let upper_leaf_entry_bytes = usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113");
+    let key_width = KEY_SCHEMA_EXTENT.width();
+    let mut node_pointers = Vec::new();
+    let mut data_pointers = Vec::new();
+    let mut seen: BTreeSet<(u32, u64)> = BTreeSet::new();
+    let mut pending = vec![Pending::Upper(*root_pointer, None)];
+    while let Some(next) = pending.pop() {
+        let (pointer, lower_of_inode, position) = match next {
+            Pending::Upper(pointer, position) => (pointer, None, position),
+            Pending::Lower(pointer, inode, position) => (pointer, Some(inode), position),
+        };
+        if pointer.all_zero
+            || !seen.insert((pointer.locations[0].device, pointer.locations[0].slot))
+        {
+            return None;
+        }
+        node_pointers.push(pointer);
+        let node = read_index_node_without_judging(reader, &pointer, cache)?;
+        let (level, index) = position.unwrap_or((node.level, 0));
+        if node.level != level || node.key_width != key_width {
+            return None;
+        }
+        match (lower_of_inode, level) {
+            (None, 0) => {
+                if node.entry_width < upper_leaf_entry_bytes {
+                    return None;
+                }
+                for entry in &node.entries {
+                    match extent_upper_leaf_entry_view(entry) {
+                        ExtentUpperLeafEntryView::NoDataUnit { .. } => {}
+                        ExtentUpperLeafEntryView::LowerSegmentRoot {
+                            inode,
+                            pointer: lower_root_pointer,
+                        } => {
+                            pending.push(Pending::Lower(
+                                parse_node_pointer(&lower_root_pointer),
+                                inode,
+                                None,
+                            ));
+                        }
+                        ExtentUpperLeafEntryView::InlineDataUnit {
+                            pointer: inline_data_pointer,
+                            ..
+                        } => {
+                            data_pointers.push(parse_data_pointer(&inline_data_pointer));
+                        }
+                        ExtentUpperLeafEntryView::Malformed => return None,
+                    }
+                }
+            }
+            (Some(_), 0) => {
+                if node.entry_width < extent_leaf_record_bytes() {
+                    return None;
+                }
+                data_pointers.extend(
+                    node.entries
+                        .iter()
+                        .map(|record| parse_data_pointer(&record[24..112])),
+                );
+            }
+            (None, _) => {
+                if node.entry_width < internal_entry_bytes {
+                    return None;
+                }
+                for entry in &node.entries {
+                    let child = extent_upper_child_of_entry_key(level, index, &entry[..key_width])?;
+                    pending.push(Pending::Upper(
+                        parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]),
+                        Some(child),
+                    ));
+                }
+            }
+            (Some(inode), _) => {
+                if node.entry_width < internal_entry_bytes {
+                    return None;
+                }
+                for entry in &node.entries {
+                    let child =
+                        extent_lower_child_of_entry_key(inode, level, index, &entry[..key_width])?;
+                    pending.push(Pending::Lower(
+                        parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]),
+                        inode,
+                        Some(child),
+                    ));
+                }
+            }
+        }
+    }
+    Some((node_pointers, data_pointers))
+}
+
+/// 一棵树的根节点下面还引用了什么：extent 树与分配记录树整棵按位置读（每个节点、数据指针、分配记录）、
+/// 「inode 树内部条目的子指针」与记账树根之下的全部节点；条目格式没有条款的树走不下去，如实记成数不全。
 fn collect_tree_references(
     reader: &dyn ImageReader,
     cache: &mut IndexNodeCache,
     references: &mut RootReferences,
+    tree_root: &PointerView,
     node: &crate::IndexNodeView,
     tree_kind_code: u16,
 ) {
     // 条目宽窄于字段表的（节点头里那个字段被改过）不按短条目去解，如实记成数不全：宽度本身由 I-1.7 / I-1.1 说话。
     match TreeKindForReferenceScan::of(tree_kind_code) {
         TreeKindForReferenceScan::Extent => {
-            if node.level == 0 && node.entry_width >= extent_leaf_record_bytes() {
-                for record in &node.entries {
-                    let data = parse_data_pointer(&record[24..112]);
-                    references.note(&data);
+            match extent_tree_without_judging(reader, tree_root, cache) {
+                Some((node_pointers, data_pointers)) => {
+                    for pointer in node_pointers.iter().chain(&data_pointers) {
+                        references.note(pointer);
+                    }
                 }
-            } else {
-                references.is_complete = false;
+                None => references.is_complete = false,
             }
         }
         TreeKindForReferenceScan::Inode => {
@@ -1777,14 +2332,14 @@
             }
         }
         TreeKindForReferenceScan::Allocation => {
-            if node.level == 0 && node.entry_width >= allocation_record_bytes() {
-                for record in &node.entries {
-                    references
-                        .allocation_records
-                        .push(parse_allocation_record(record));
+            match allocation_record_tree_without_judging(reader, tree_root, cache) {
+                Some((node_pointers, records)) => {
+                    for pointer in &node_pointers {
+                        references.note(pointer);
+                    }
+                    references.allocation_records.extend(records);
                 }
-            } else {
-                references.is_complete = false;
+                None => references.is_complete = false,
             }
         }
         TreeKindForReferenceScan::Accounting => {
@@ -2025,8 +2580,9 @@
 /// I-5.4（分配记录罩住的槽互不相交）：候选集里每条有效根的分配记录树，同一块盘上任意两条记录（不论已分配、已释放、释放代是否已低于
 /// 回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交（`.claude/kb/invariants.md` I-5.4 那一行）。I-5.1（物理范围不重叠） 管的是
 /// 树里的引用，判不出记录之间的重叠：代码三方第二轮 Z1-d 的镜像上引用互不重叠，而两条分配记录罩住同一个槽，下一次可写挂载重建分配器时 panic。
-/// 几条根指着同一个分配记录树节点时（照抄上一版的根）那个节点只判一次；每个 (节点, 盘) 判一格，那块盘上只有一条记录也算判过。
-/// 走不到记录的根不判：树表或分配记录树读不出（由 I-2.1 / I-7.2 说话）、分配记录树有内部节点（内部条目格式没有条款，总审核 D8-D11 发现 16）。
+/// 几条根指着同一个分配记录树根时（照抄上一版的根）那棵树只判一次；每个 (树, 盘) 判一格，那块盘上只有一条记录也算判过。
+/// 树按位置寻址、可以多层（D8（核心索引结构） 已定项 14）：整棵读下来，同一块盘上跨叶的记录一起比。
+/// 走不到记录的根不判：树表或分配记录树读不出、位置对不上（由 I-2.1 / I-1.1 / I-7.2 说话）。
 fn judge_allocation_records_disjoint(
     reader: &dyn ImageReader,
     roots: &[(u64, u64, crate::RootView)],
@@ -2063,21 +2619,19 @@
             )) {
                 continue;
             }
-            let Some(node) = read_index_node_without_judging(reader, &allocation_root, cache)
+            let Some((_, records)) =
+                allocation_record_tree_without_judging(reader, &allocation_root, cache)
             else {
                 continue;
             };
-            if node.level > 0 || node.entry_width < allocation_record_bytes() {
-                continue;
-            }
-            judge_allocation_record_ranges_of_one_node(&node, root.checkpoint_txg, judgements);
+            judge_allocation_record_ranges(&records, root.checkpoint_txg, judgements);
             judged_allocation_nodes += 1;
         }
     }
     if judged_allocation_nodes == 0 {
         judgements.not_applicable(
             "I-5.4",
-            "候选集里没有一条根的分配记录树走得到叶：第 0 代树表是空的，或树表 / 分配记录树读不出、有内部节点",
+            "候选集里没有一条根的分配记录树走得到叶：第 0 代树表是空的，或树表 / 分配记录树读不出、位置对不上",
         );
     }
 }
@@ -2208,14 +2762,11 @@
         )) {
             continue;
         }
-        let Some(node) = read_index_node_without_judging(reader, pointer, cache) else {
+        let Some((_, records)) = allocation_record_tree_without_judging(reader, pointer, cache)
+        else {
             continue;
         };
-        if node.level > 0 || node.entry_width < allocation_record_bytes() {
-            continue;
-        }
-        for entry in &node.entries {
-            let record = parse_allocation_record(entry);
+        for record in records {
             if record.is_released
                 || !examined_records.insert((record.device, record.slot, record.generation))
             {
@@ -2245,7 +2796,7 @@
         judgements.not_applicable(
             "I-3.10",
             if examined_records.is_empty() {
-                "候选集里没有一片分配记录树走得到未释放的记录（第 0 代树表是空的，或分配记录树读不出、有内部节点）"
+                "候选集里没有一棵分配记录树走得到未释放的记录（第 0 代树表是空的，或分配记录树读不出、位置对不上）"
             } else {
                 "未释放的分配记录罩住的起点槽上一个可用的单元头都读不出：读不出本身归 I-1.1 与 I-2.1"
             },
@@ -2572,15 +3123,14 @@
     }
 }
 
-/// 一个分配记录树叶上逐盘判 I-5.4：同一块盘上的记录按起点排好，相邻两条不相交就是两两不相交（跨度非负）。
-fn judge_allocation_record_ranges_of_one_node(
-    node: &crate::IndexNodeView,
+/// 一棵分配记录树的全部记录上逐盘判 I-5.4：同一块盘上的记录按起点排好，相邻两条不相交就是两两不相交（跨度非负）。
+fn judge_allocation_record_ranges(
+    records: &[AllocationRecordView],
     root_checkpoint_txg: u64,
     judgements: &mut Judgements,
 ) {
     let mut ranges_per_device: BTreeMap<u32, Vec<(u64, u64)>> = BTreeMap::new();
-    for entry in &node.entries {
-        let record = parse_allocation_record(entry);
+    for record in records {
         ranges_per_device
             .entry(record.device)
             .or_default()
@@ -2612,10 +3162,10 @@
 /// 这个单元每一份都读得出且对得上，照旧判违例。按单元而不按份判：写者只要一份核出对不上就把两块盘的记录一起留在已分配
 /// （D19 已定项 5「另一块盘上那一份对得上也一起留，各盘的账保持对称」），按份判的话对得上那块盘上的那一条恒红。
 ///
-/// 读的是 `root` 那棵分配记录树（树表里种类 3 那一条指着的根节点，第一版只有一个节点）：未释放、`referenced_start_slots`
+/// 读的是 `root` 那棵分配记录树（树表里种类 3 那一条指着的根，按位置整棵走下去，`allocation_record_tree_without_judging`）：未释放、`referenced_start_slots`
 /// 里没有哪个引用起在它那一槽的记录，按单元（起点槽、跨度——第一版一个单元两盘同槽）归组；一组的每一份按 (盘, 槽, 跨度) 读那一整份单元、
 /// 交 `check_unit` 判（magic、头校验和、载荷 CRC）——有一份读不出或任一关不过，整组豁免。逐盘交回豁免的槽数；
-/// 树表或分配记录树读不出、有内部节点时一格都不豁免（交回空表，照旧判）。
+/// 树表或分配记录树读不出、位置对不上时那一棵一格都不豁免（照旧判）。
 fn quarantined_slots_exempted_per_device(
     reader: &dyn ImageReader,
     root: &crate::RootView,
@@ -2641,17 +3191,12 @@
         if allocation_root.all_zero {
             continue;
         }
-        let Some(node) = read_index_node_without_judging(reader, &allocation_root, cache) else {
+        let Some((_, records)) =
+            allocation_record_tree_without_judging(reader, &allocation_root, cache)
+        else {
             continue;
         };
-        if node.level > 0 || node.entry_width < allocation_record_bytes() {
-            continue;
-        }
-        for record in node
-            .entries
-            .iter()
-            .map(|record_bytes| parse_allocation_record(record_bytes))
-        {
+        for record in records {
             if record.is_released || referenced_start_slots.contains(&(record.device, record.slot))
             {
                 continue;
@@ -2710,11 +3255,9 @@
         if allocation_root.all_zero {
             continue;
         }
-        let node = read_index_node_without_judging(reader, &allocation_root, &mut cache)?;
-        if node.level > 0 || node.entry_width < allocation_record_bytes() {
-            return None;
-        }
-        records += node.entries.len();
+        let (_, records_of_the_tree) =
+            allocation_record_tree_without_judging(reader, &allocation_root, &mut cache)?;
+        records += records_of_the_tree.len();
     }
     Some(records)
 }
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/allocation_record_tree.rs tree/crates/singlefs-core/src/allocation_record_tree.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/allocation_record_tree.rs	1970-01-01 00:00:00.000000000 +0000
+++ tree/crates/singlefs-core/src/allocation_record_tree.rs	2026-09-24 20:56:53.030717457 +0000
@@ -0,0 +1,993 @@
+//! 分配记录树按 key 空间定形状（D8（核心索引结构） 已定项 14「分配记录树」，用户 2026-09-24 定 K1）：不分裂，按绝对槽号按位置寻址。
+//!
+//! - **叶**：盘 d 上第 k 片叶罩槽 `[k × W, (k + 1) × W)`，W = [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]（偶数、≤ 叶条目容量 812）；
+//!   记录按起点槽落叶，记录的末槽不越过它所在叶的末槽（两槽单元起在偶数槽、W 取偶数 ⇒ 永远不跨叶）。
+//! - **往上各层按位置寻址**：层级 L（L ≥ 1）上盘 d 第 j 个节点罩槽 `[j × S_L, (j + 1) × S_L)`，S_L = W × F^L，
+//!   F = [`ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`]（内部条目 96 = 孩子罩的那一段的起点 key 10 + 子指针 86）。子节点罩哪一段由它在父节点里的
+//!   那一格算出：父节点的条目 key 就是孩子那一段的起点，checker 与读者都按位置独立算出孩子该罩的那一段逐节点核。
+//! - **根**罩整个 key 空间（`[全 0, 全 FF]`，盘这一维也在根里分路）：根的层级 R 是池几何的确定函数——最小的 R ≥ 1 使
+//!   Σ_盘 ⌈盘上绝对槽数 ÷ S_{R−1}⌉ ≤ F（根装得下每块盘上 R − 1 层那一层的全部格）。树高 = 根节点头里的层级 + 1（D8 已定项 11 ⑤）。
+//! - **没有记录的一段 = 全空闲**：那片叶不写，父节点那一格留空；孩子一个都没有的内部节点同样不写。根恒在（池里恒有 mkfs 那两个单元的记录）。
+//!
+//! 盘这一维怎么进位置寻址（根里按盘分路、根之下每个节点只属于一块盘），条款没写到，是实现员的取法（交回里写明）。
+//!
+//! 这里只管结构：几何、这一版有哪些节点、哪些节点的内容这次变了、这次之后树长什么样、装一个节点的字节、从盘上读回一棵树并逐节点核位置。
+//! 取落点、发出生序号、写盘在 `crate::transaction` 那一侧；交回拒绝的一样都不动盘。
+
+use std::collections::{BTreeMap, BTreeSet};
+
+use singlefs_format::{
+    ALLOCATION_RECORD_BYTES, ALLOCATION_RECORD_KEY_BYTES,
+    ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES, ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
+    ALLOCATION_RECORD_TREE_LEAF_SLOTS, SLOT_BYTES, UNIT_AREA_START_SLOT,
+};
+
+use crate::address::{
+    CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier,
+};
+use crate::allocator::{AllocationRecord, PoolAllocator};
+use crate::bytes::{ByteReader, ByteWriter};
+use crate::pointer::{BirthSequence, NodePointer};
+use crate::recovery::{PoolReader, RecoveryFailure};
+use crate::root_record::RootRecord;
+use crate::unit::{build_index_node, parse_index_node, IndexNodeHeader};
+
+/// 槽号字段是 6 字节（D3（空间分配） 已定项 7 的 key 段宽）：一块盘上最大的槽号。
+const LARGEST_SLOT_NUMBER: u64 = (1 << 48) - 1;
+
+/// 根之下一个节点的位置：层级（0 是叶）、它属于哪块盘、同一块盘同一层从槽号 0 起数第几个（按位置，不按有没有节点）。
+/// 派生的全序（层级升序、同层按盘再按序号升序 = 按 key 升序）就是这棵树在一次发布里的 bump 次序与出生序号的发号次序：
+/// 树内先叶后根、同层按 key 升序（D3（空间分配） 已定项 10 ⑤、D19（块指针的结构与宽度预算） 已定项 9）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub struct AllocationRecordTreeNodePosition {
+    pub level: u8,
+    pub device: DeviceIdentity,
+    pub index_in_device: u64,
+}
+
+/// 分配记录树里的一个节点：根之下的按位置，根只有一个。派生的全序把根排在最末（先叶后根）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum AllocationRecordTreeNode {
+    BelowTheRoot(AllocationRecordTreeNodePosition),
+    Root,
+}
+
+/// 分配记录树的 key（设备 4, 槽号 6，盘上小端）的字节。
+#[must_use]
+pub fn allocation_record_key_bytes(device: DeviceIdentity, slot: SlotNumber) -> Vec<u8> {
+    let mut writer = ByteWriter::new(usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10"));
+    writer.put_u32(device.0);
+    writer.put_six_byte_unsigned(slot.0);
+    writer.into_bytes()
+}
+
+fn key_bytes_width() -> usize {
+    usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10")
+}
+
+fn parse_allocation_record_key(key: &[u8]) -> (DeviceIdentity, SlotNumber) {
+    let mut reader = ByteReader::at(key, 0);
+    (
+        DeviceIdentity(reader.get_u32()),
+        SlotNumber(reader.get_six_byte_unsigned()),
+    )
+}
+
+/// 这个池上分配记录树的几何：池里每块盘的绝对槽数（按设备身份升序）与由它们定下的根层级。
+/// 写者按分配器里的盘算（[`AllocationRecordTreeGeometry::of_allocator`]），读者按盘的字节数算（[`AllocationRecordTreeGeometry::of_reader`]），
+/// checker 另写一份按同一条公式算。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct AllocationRecordTreeGeometry {
+    slots_of_each_device: Vec<(DeviceIdentity, u64)>,
+    root_level: u8,
+}
+
+impl AllocationRecordTreeGeometry {
+    /// 从每块盘的绝对槽数（设备字节数 ÷ 16384）算。
+    ///
+    /// # Panics
+    /// 池里一块盘都没有，或盘多到连 255 层的根都装不下每块盘各一格（多于 169 块盘）：池里恒有两块盘（mkfs 断言过）。
+    #[must_use]
+    pub fn of_devices(slots_of_each_device: &[(DeviceIdentity, u64)]) -> Self {
+        assert!(
+            !slots_of_each_device.is_empty(),
+            "池里至少一块盘：mkfs 断言过两块"
+        );
+        let mut slots_of_each_device = slots_of_each_device.to_vec();
+        slots_of_each_device.sort_by_key(|(device, _)| *device);
+        // 迭代上界是 255 层；每一轮 S_{R−1} 乘 F，槽数有上界 2^48，至多 6 轮就到每块盘一格。
+        let root_level = (1..=u8::MAX)
+            .find(|candidate_root_level| {
+                let child_span = span_in_slots_at_level(candidate_root_level - 1);
+                let cells: u64 = slots_of_each_device
+                    .iter()
+                    .map(|(_, slots)| slots.div_ceil(child_span))
+                    .sum();
+                cells <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
+            })
+            .expect("盘数不多于扇出 169 时，槽数有上界 2^48、层级至多 6 就装得下每块盘各一格");
+        Self {
+            slots_of_each_device,
+            root_level,
+        }
+    }
+
+    /// 写者那一侧：分配器里每块盘的单元区末端就是这块盘的绝对槽数（单元区从 [`UNIT_AREA_START_SLOT`] 起到设备末尾）。
+    #[must_use]
+    pub fn of_allocator(allocator: &PoolAllocator) -> Self {
+        let slots: Vec<(DeviceIdentity, u64)> = allocator
+            .devices
+            .iter()
+            .map(|device_map| {
+                (
+                    device_map.device,
+                    UNIT_AREA_START_SLOT + device_map.unit_area_slots(),
+                )
+            })
+            .collect();
+        Self::of_devices(&slots)
+    }
+
+    /// 读者那一侧：池里每块盘的字节数 ÷ 16384。
+    ///
+    /// # Panics
+    /// 读者列出的盘说不出自己的字节数：两个实现都从同一张表里取（`PoolReader::device_size_in_bytes`）。
+    #[must_use]
+    pub fn of_reader<Reader: PoolReader + ?Sized>(reader: &Reader) -> Self {
+        let slots: Vec<(DeviceIdentity, u64)> = reader
+            .device_identities()
+            .into_iter()
+            .map(|device| {
+                (
+                    device,
+                    reader
+                        .device_size_in_bytes(device)
+                        .expect("读者列出的盘说得出自己的字节数")
+                        / SLOT_BYTES,
+                )
+            })
+            .collect();
+        Self::of_devices(&slots)
+    }
+
+    /// 根的层级（树高 − 1）。
+    #[must_use]
+    pub fn root_level(&self) -> u8 {
+        self.root_level
+    }
+
+    /// 树高：根的层级 + 1（D8（核心索引结构） 已定项 11 ⑤）。
+    #[must_use]
+    pub fn height(&self) -> u64 {
+        u64::from(self.root_level) + 1
+    }
+
+    /// 这块盘在不在池里。
+    #[must_use]
+    pub fn has_device(&self, device: DeviceIdentity) -> bool {
+        self.slots_of_each_device
+            .iter()
+            .any(|(candidate, _)| *candidate == device)
+    }
+
+    /// 一个槽所在的那片叶。
+    #[must_use]
+    pub fn leaf_of_slot(
+        &self,
+        device: DeviceIdentity,
+        slot: SlotNumber,
+    ) -> AllocationRecordTreeNode {
+        AllocationRecordTreeNode::BelowTheRoot(AllocationRecordTreeNodePosition {
+            level: 0,
+            device,
+            index_in_device: slot.0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS,
+        })
+    }
+
+    /// 一个节点的父节点；根没有父节点。
+    #[must_use]
+    pub fn parent_of(&self, node: AllocationRecordTreeNode) -> Option<AllocationRecordTreeNode> {
+        match node {
+            AllocationRecordTreeNode::Root => None,
+            AllocationRecordTreeNode::BelowTheRoot(position) => {
+                let parent_level = position.level + 1;
+                if parent_level == self.root_level {
+                    Some(AllocationRecordTreeNode::Root)
+                } else {
+                    Some(AllocationRecordTreeNode::BelowTheRoot(
+                        AllocationRecordTreeNodePosition {
+                            level: parent_level,
+                            device: position.device,
+                            index_in_device: position.index_in_device
+                                / ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
+                        },
+                    ))
+                }
+            }
+        }
+    }
+
+    /// 一个节点的层级。
+    #[must_use]
+    pub fn level_of(&self, node: AllocationRecordTreeNode) -> u8 {
+        match node {
+            AllocationRecordTreeNode::Root => self.root_level,
+            AllocationRecordTreeNode::BelowTheRoot(position) => position.level,
+        }
+    }
+
+    /// 一个节点按位置规定罩的那一段 key，(最小, 最大) 两把 key 的字节：节点头里的 key 区间就写它
+    /// （D18（块里携带什么信息） 已定项 2：按位置寻址的树，节点头的 key 区间写这个节点按位置规定罩的那一段）。
+    /// 根罩整个 key 空间；根之下的节点罩它那块盘上 `[j × S_L, (j + 1) × S_L)`（末端不越过 6 字节槽号的最大值）。
+    #[must_use]
+    pub fn key_range_of(&self, node: AllocationRecordTreeNode) -> (Vec<u8>, Vec<u8>) {
+        match node {
+            AllocationRecordTreeNode::Root => (
+                vec![0u8; key_bytes_width()],
+                vec![u8::MAX; key_bytes_width()],
+            ),
+            AllocationRecordTreeNode::BelowTheRoot(position) => {
+                let (first_slot, last_slot) = slot_range_of(position);
+                (
+                    allocation_record_key_bytes(position.device, first_slot),
+                    allocation_record_key_bytes(position.device, last_slot),
+                )
+            }
+        }
+    }
+
+    /// 父节点里一条内部条目的 key 指的是哪个孩子：key 是孩子那一段的起点，槽号要落在孩子那一层的格点上；父节点在根之下时
+    /// 孩子与父节点同盘、落在父节点那一段里。对不上交回 `None`（盘上读来的 key 可以是任何值）。
+    #[must_use]
+    pub fn child_named_by_entry_key(
+        &self,
+        parent: AllocationRecordTreeNode,
+        entry_key: &[u8],
+    ) -> Option<AllocationRecordTreeNode> {
+        let parent_level = self.level_of(parent);
+        let child_level = parent_level.checked_sub(1)?;
+        let (device, slot) = parse_allocation_record_key(entry_key);
+        let child_span = span_in_slots_at_level(child_level);
+        if !slot.0.is_multiple_of(child_span) || !self.has_device(device) {
+            return None;
+        }
+        let child = AllocationRecordTreeNodePosition {
+            level: child_level,
+            device,
+            index_in_device: slot.0 / child_span,
+        };
+        let child_node = AllocationRecordTreeNode::BelowTheRoot(child);
+        (self.parent_of(child_node) == Some(parent)).then_some(child_node)
+    }
+
+    /// 一条记录落在它所在的叶里：同盘、起点槽与末槽都在叶那一段里（D8 已定项 14「记录的末槽不越过它所在叶的末槽」）。
+    #[must_use]
+    pub fn record_fits_in_its_leaf(&self, record: &AllocationRecord) -> bool {
+        let AllocationRecordTreeNode::BelowTheRoot(leaf) =
+            self.leaf_of_slot(record.device, record.slot)
+        else {
+            return false;
+        };
+        let (_, last_slot) = slot_range_of(leaf);
+        record.span_slots > 0 && record.slot.0 + u64::from(record.span_slots) - 1 <= last_slot.0
+    }
+}
+
+/// 层级 L 上一个节点罩几个槽：W × F^L，乘到装不进 u64 就停在 u64 的最大值（那么宽的一层罩得住整个 6 字节槽号空间）。
+#[must_use]
+pub fn span_in_slots_at_level(level: u8) -> u64 {
+    (0..level).fold(ALLOCATION_RECORD_TREE_LEAF_SLOTS, |span, _| {
+        span.saturating_mul(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT)
+    })
+}
+
+/// 根之下一个节点罩的槽：`[j × S_L, (j + 1) × S_L)` 的首末两槽，末槽不越过 6 字节槽号的最大值。
+fn slot_range_of(position: AllocationRecordTreeNodePosition) -> (SlotNumber, SlotNumber) {
+    let span = span_in_slots_at_level(position.level);
+    let first_slot = position.index_in_device.saturating_mul(span);
+    let last_slot = first_slot.saturating_add(span - 1).min(LARGEST_SLOT_NUMBER);
+    (
+        SlotNumber(first_slot.min(LARGEST_SLOT_NUMBER)),
+        SlotNumber(last_slot),
+    )
+}
+
+/// 每片有记录的叶里装的记录，按 key 升序。
+///
+/// # Panics
+/// 有一条记录越过它所在叶的末槽：写者发的落点两槽单元起在偶数槽、W 取偶数（[`ALLOCATION_RECORD_TREE_LEAF_SLOTS`] 的断言），
+/// 从盘上读来的记录进分配器之前由 [`read_allocation_record_tree`] 逐条判过。
+#[must_use]
+pub fn records_of_each_leaf(
+    geometry: &AllocationRecordTreeGeometry,
+    records: &[AllocationRecord],
+) -> BTreeMap<AllocationRecordTreeNode, Vec<AllocationRecord>> {
+    let mut by_leaf: BTreeMap<AllocationRecordTreeNode, Vec<AllocationRecord>> = BTreeMap::new();
+    for record in records {
+        assert!(
+            geometry.record_fits_in_its_leaf(record),
+            "分配记录（盘 {:?} 槽 {:?} 跨 {}）越过它所在叶的末槽：两槽单元起在偶数槽、叶宽取偶数，盘上读来的记录进来之前判过",
+            record.device,
+            record.slot,
+            record.span_slots
+        );
+        by_leaf
+            .entry(geometry.leaf_of_slot(record.device, record.slot))
+            .or_default()
+            .push(*record);
+    }
+    for leaf_records in by_leaf.values_mut() {
+        leaf_records.sort_by_key(AllocationRecord::sort_key);
+    }
+    by_leaf
+}
+
+/// 这些记录在树里要哪些节点：有记录的叶、它们的每一个祖先、根。
+#[must_use]
+pub fn nodes_holding_records(
+    geometry: &AllocationRecordTreeGeometry,
+    records: &[AllocationRecord],
+) -> BTreeSet<AllocationRecordTreeNode> {
+    let mut nodes = BTreeSet::new();
+    nodes.insert(AllocationRecordTreeNode::Root);
+    for leaf in records_of_each_leaf(geometry, records).into_keys() {
+        insert_the_node_and_its_ancestors(geometry, leaf, &mut nodes);
+    }
+    nodes
+}
+
+fn insert_the_node_and_its_ancestors(
+    geometry: &AllocationRecordTreeGeometry,
+    node: AllocationRecordTreeNode,
+    nodes: &mut BTreeSet<AllocationRecordTreeNode>,
+) {
+    let mut current = Some(node);
+    // 迭代上界是根的层级 + 1：每一轮往上一层，根没有父节点。
+    while let Some(ancestor) = current {
+        if !nodes.insert(ancestor) && ancestor != node {
+            break;
+        }
+        current = geometry.parent_of(ancestor);
+    }
+}
+
+/// 从上一版的记录到这一版的记录，内容变了的节点：记录清单（逐字节）不同的叶——这一版新有、上一版有而这一版没有的都算——
+/// 与它们的每一个祖先、根。叶一变，它这一版的指针就变，父节点那一格跟着变，一路到根（COW 叶 + 全部祖先）。
+#[must_use]
+pub fn nodes_whose_contents_changed(
+    geometry: &AllocationRecordTreeGeometry,
+    previous_records: &[AllocationRecord],
+    records_after: &[AllocationRecord],
+) -> BTreeSet<AllocationRecordTreeNode> {
+    let before = records_of_each_leaf(geometry, previous_records);
+    let after = records_of_each_leaf(geometry, records_after);
+    let mut changed = BTreeSet::new();
+    let leaves: BTreeSet<AllocationRecordTreeNode> =
+        before.keys().chain(after.keys()).copied().collect();
+    for leaf in leaves {
+        if before.get(&leaf) != after.get(&leaf) {
+            insert_the_node_and_its_ancestors(geometry, leaf, &mut changed);
+        }
+    }
+    if !changed.is_empty() {
+        changed.insert(AllocationRecordTreeNode::Root);
+    }
+    changed
+}
+
+/// 一版分配记录树里一个节点从哪来：照抄上一版同一个位置上的节点（字节、落点、指针一个不动），或这次重写。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum AllocationRecordTreeNodeOrigin {
+    CarriedFromThePreviousVersion,
+    RewrittenThisPublish,
+}
+
+/// 这次发布之后分配记录树长什么样。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct AllocationRecordTreePlan {
+    /// 这一版的每个节点与它从哪来，按 bump 次序（根之下的按位置升序，根在最末）。
+    pub nodes: Vec<(AllocationRecordTreeNode, AllocationRecordTreeNodeOrigin)>,
+    /// 上一版里这次被换下的节点（重写了的、这一版不再有的），按上一版的 bump 次序。
+    pub replaced_previous_nodes: Vec<AllocationRecordTreeNode>,
+}
+
+impl AllocationRecordTreePlan {
+    /// 这次重写的节点，按 bump 次序（= 这棵树里出生序号的发号次序）。
+    #[must_use]
+    pub fn rewritten_nodes(&self) -> Vec<AllocationRecordTreeNode> {
+        self.nodes
+            .iter()
+            .filter(|(_, origin)| *origin == AllocationRecordTreeNodeOrigin::RewrittenThisPublish)
+            .map(|(node, _)| *node)
+            .collect()
+    }
+
+    /// 这一版的全部节点，按 bump 次序。
+    #[must_use]
+    pub fn node_set(&self) -> BTreeSet<AllocationRecordTreeNode> {
+        self.nodes.iter().map(|(node, _)| *node).collect()
+    }
+}
+
+/// 这次之后树长什么样：这一版的节点是 `nodes_after`（有记录的叶、它们的祖先、根），其中在 `rewritten` 里的、或上一版没有的，这次重写；
+/// 别的照抄上一版同一个位置上的节点。上一版有而这一版没有的、这次重写的，都换下。
+///
+/// `rewritten` 由调用方给：它要罩住内容变了的节点（[`nodes_whose_contents_changed`]），可以多罩（多罩的那几个内容没变、照样重写一份）。
+/// 调用方在分配器的一份拷贝上把这次的释放与取落点走一遍、按走出来的记录算内容变了的节点，不在 `rewritten` 里就并进去再走一遍，直到罩住
+/// （固定点：分配记录树的节点也要给自己记分配记录，取落点又会改记录）。
+#[must_use]
+pub fn plan_the_tree_after_this_publish(
+    previous_nodes: &BTreeSet<AllocationRecordTreeNode>,
+    nodes_after: &BTreeSet<AllocationRecordTreeNode>,
+    rewritten: &BTreeSet<AllocationRecordTreeNode>,
+) -> AllocationRecordTreePlan {
+    let nodes = nodes_after
+        .iter()
+        .map(|node| {
+            let origin = if rewritten.contains(node) || !previous_nodes.contains(node) {
+                AllocationRecordTreeNodeOrigin::RewrittenThisPublish
+            } else {
+                AllocationRecordTreeNodeOrigin::CarriedFromThePreviousVersion
+            };
+            (*node, origin)
+        })
+        .collect();
+    let replaced_previous_nodes = previous_nodes
+        .iter()
+        .filter(|node| rewritten.contains(node) || !nodes_after.contains(node))
+        .copied()
+        .collect();
+    AllocationRecordTreePlan {
+        nodes,
+        replaced_previous_nodes,
+    }
+}
+
+/// 一个内部节点这一版的孩子，按位置升序（= 按 key 升序）：`nodes` 是这一版的全部节点。
+#[must_use]
+pub fn children_of(
+    geometry: &AllocationRecordTreeGeometry,
+    nodes: &BTreeSet<AllocationRecordTreeNode>,
+    parent: AllocationRecordTreeNode,
+) -> Vec<AllocationRecordTreeNode> {
+    nodes
+        .iter()
+        .copied()
+        .filter(|node| geometry.parent_of(*node) == Some(parent))
+        .collect()
+}
+
+/// 一版分配记录树在内存里的样子：每个节点与它这一版的指针，按 bump 次序（根之下的按位置升序，根在最末）。
+/// 节点的字节住 `transaction::TransactionOutput::units` 里它那个角色的单元。
+#[derive(Clone, Debug, Default, PartialEq, Eq)]
+pub struct AllocationRecordTreeVersion {
+    pub nodes: Vec<(AllocationRecordTreeNode, NodePointer)>,
+}
+
+impl AllocationRecordTreeVersion {
+    /// 这一版的全部节点。
+    #[must_use]
+    pub fn node_set(&self) -> BTreeSet<AllocationRecordTreeNode> {
+        self.nodes.iter().map(|(node, _)| *node).collect()
+    }
+
+    /// 这个节点这一版的指针；这一版没有它交回 `None`。
+    #[must_use]
+    pub fn pointer_of(&self, node: AllocationRecordTreeNode) -> Option<NodePointer> {
+        self.nodes
+            .iter()
+            .find(|(candidate, _)| *candidate == node)
+            .map(|(_, pointer)| *pointer)
+    }
+
+    /// 根的指针（树表条目或根记录那一项里写的那一条）。
+    ///
+    /// # Panics
+    /// 这一版没有根：有记录的一版恒有根（池里恒有 mkfs 那两个单元的记录）。
+    #[must_use]
+    pub fn root_pointer(&self) -> NodePointer {
+        self.pointer_of(AllocationRecordTreeNode::Root)
+            .expect("有记录的一版恒有根")
+    }
+}
+
+/// 一个节点装的东西：叶装记录（按 key 升序），内部节点装孩子与孩子这一版的指针（按位置升序）。
+pub enum AllocationRecordTreeNodeContents<'contents> {
+    Leaf(&'contents [AllocationRecord]),
+    Internal(Vec<(AllocationRecordTreeNode, NodePointer)>),
+}
+
+/// 装一个分配记录树节点的字节（码 2，D18（块里携带什么信息） 已定项 18 的头）：key 区间写这个节点按位置规定罩的那一段；
+/// 叶的条目是分配记录 20，内部节点的条目是「孩子那一段的起点 key + 孩子这一版的指针」96。
+///
+/// # Panics
+/// 叶里一条记录都没有（缺席的叶不写），或内部节点一个孩子都没有：规划只交出有记录的叶与它们的祖先。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "装一个节点要的：几何、哪个节点、装什么、树号，与码 2 头的四样身份字段，各自独立"
+)]
+#[must_use]
+pub fn build_allocation_record_tree_node(
+    geometry: &AllocationRecordTreeGeometry,
+    node: AllocationRecordTreeNode,
+    contents: &AllocationRecordTreeNodeContents<'_>,
+    tree: TreeIdentifier,
+    birth_txg: CheckpointTxg,
+    filesystem_identifier: &[u8; 16],
+    instance: InstanceGeneration,
+    birth_sequence: BirthSequence,
+) -> Vec<u8> {
+    let (smallest_key, largest_key) = geometry.key_range_of(node);
+    let (entry_width, entries): (u64, Vec<Vec<u8>>) = match contents {
+        AllocationRecordTreeNodeContents::Leaf(records) => {
+            assert!(!records.is_empty(), "缺席的叶不写：规划只交出有记录的叶");
+            (
+                ALLOCATION_RECORD_BYTES,
+                records.iter().map(AllocationRecord::to_bytes).collect(),
+            )
+        }
+        AllocationRecordTreeNodeContents::Internal(children) => {
+            assert!(
+                !children.is_empty(),
+                "孩子一个都没有的内部节点不写：规划只交出有记录的叶的祖先"
+            );
+            (
+                ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES,
+                children
+                    .iter()
+                    .map(|(child, pointer)| {
+                        let (child_start, _) = geometry.key_range_of(*child);
+                        let mut writer = ByteWriter::new(
+                            usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES)
+                                .expect("96"),
+                        );
+                        writer.put(&child_start);
+                        pointer.write_to(&mut writer);
+                        writer.into_bytes()
+                    })
+                    .collect(),
+            )
+        }
+    };
+    build_index_node(
+        tree,
+        geometry.level_of(node),
+        key_bytes_width(),
+        &smallest_key,
+        &largest_key,
+        birth_txg,
+        filesystem_identifier,
+        instance,
+        birth_sequence,
+        u16::try_from(entry_width).expect("条目宽 2 字节"),
+        &entries,
+    )
+}
+
+/// 从盘上读一棵分配记录树时核到哪一步（同 `code_two_tree::CodeTwoTreeHeaderJudgement` 的两档）。**位置那几样两档都核**：
+/// 节点的层级、节点头里的 key 区间是不是它的位置规定的那一段、父节点的条目 key 是不是一个合法孩子的起点、记录落不落在它所在叶里——
+/// 这一版在内存里按位置记节点、按叶分记录，读回来的位置对不上，下一次发布算「哪片叶变了」就算错。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum AllocationRecordTreeHeaderJudgement {
+    /// 另核每个节点头的树 ID、fsid、诞生不晚于根、出生序号与指着它的指针相同（冷走读、影子账）。
+    EveryHeaderAgainstItsReference,
+    /// 头里的出生身份不核（从盘上重建上一版、挂载时重建分配器，与重建路径读别的树同一个口径）。
+    OnlyWhatThePositionsNeed,
+}
+
+/// 从盘上读回来的一棵分配记录树：每个节点、它的指针与字节（bump 次序），与全部记录（按 key 升序）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct AllocationRecordTreeReadFromDisk {
+    pub nodes: Vec<(AllocationRecordTreeNode, NodePointer, Vec<u8>)>,
+    pub records: Vec<AllocationRecord>,
+}
+
+impl AllocationRecordTreeReadFromDisk {
+    /// 读回来的这一版（节点与指针）。
+    #[must_use]
+    pub fn version(&self) -> AllocationRecordTreeVersion {
+        AllocationRecordTreeVersion {
+            nodes: self
+                .nodes
+                .iter()
+                .map(|(node, pointer, _)| (*node, *pointer))
+                .collect(),
+        }
+    }
+}
+
+fn violated(detail: &'static str) -> RecoveryFailure {
+    RecoveryFailure::InvariantViolated {
+        invariant: "I-1.1",
+        detail,
+    }
+}
+
+/// 读一棵分配记录树时一路不变的那几样。
+struct AllocationRecordTreeReading<'reading> {
+    geometry: &'reading AllocationRecordTreeGeometry,
+    tree: TreeIdentifier,
+    judgement: AllocationRecordTreeHeaderJudgement,
+    root: &'reading RootRecord,
+    expected_filesystem_identifier: u64,
+}
+
+impl AllocationRecordTreeReading<'_> {
+    /// 核一个节点头里的出生身份（只有 `EveryHeaderAgainstItsReference` 才核）。根那一个的报错逐字沿用 `recovery` 核树根的那几句
+    /// （同 `code_two_tree` 的做法），读路径上几种坏法报的成员不随树长没长多层而变。
+    fn judge_the_identity_in_the_header(
+        &self,
+        header: &IndexNodeHeader,
+        pointer: &NodePointer,
+        is_root: bool,
+    ) -> Result<(), RecoveryFailure> {
+        match self.judgement {
+            AllocationRecordTreeHeaderJudgement::OnlyWhatThePositionsNeed => return Ok(()),
+            AllocationRecordTreeHeaderJudgement::EveryHeaderAgainstItsReference => {}
+        }
+        let detail = |of_the_root: &'static str, of_a_node_below_the_root: &'static str| {
+            if is_root {
+                of_the_root
+            } else {
+                of_a_node_below_the_root
+            }
+        };
+        if header.tree != self.tree {
+            return Err(RecoveryFailure::InvariantViolated {
+                invariant: "I-1.3",
+                detail: detail(
+                    "根头里的树 ID 与树表不符",
+                    "分配记录树节点头里的树 ID 与引用它的树不符",
+                ),
+            });
+        }
+        if header.key_width != key_bytes_width() {
+            return Err(RecoveryFailure::InvariantViolated {
+                invariant: "E142 走读同款",
+                detail: detail(
+                    "根自述 key 宽与树的种类不符",
+                    "分配记录树节点自述 key 宽不是 10",
+                ),
+            });
+        }
+        if header.birth_txg > self.root.checkpoint_txg || header.instance > self.root.instance {
+            return Err(RecoveryFailure::InvariantViolated {
+                invariant: "I-1.2",
+                detail: detail("树根诞生于根之后", "分配记录树节点诞生于根之后"),
+            });
+        }
+        if header.filesystem_identifier != self.expected_filesystem_identifier {
+            return Err(RecoveryFailure::InvariantViolated {
+                invariant: "I-1.4",
+                detail: detail("树根 fsid 不符", "分配记录树节点 fsid 不符"),
+            });
+        }
+        if header.birth_sequence != pointer.birth_sequence {
+            return Err(RecoveryFailure::InvariantViolated {
+                invariant: "I-1.2",
+                detail: detail("树根出生序号与指针不符", "分配记录树节点出生序号与指针不符"),
+            });
+        }
+        Ok(())
+    }
+
+    /// 读一个节点和它下面整棵子树，按后序把 (节点, 指针, 字节) 推进 `nodes`、叶里的记录推进 `records`。
+    fn read_node_and_its_subtree(
+        &self,
+        node: AllocationRecordTreeNode,
+        pointer: &NodePointer,
+        read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
+        units_seen: &mut BTreeSet<SlotNumber>,
+        nodes: &mut Vec<(AllocationRecordTreeNode, NodePointer, Vec<u8>)>,
+        records: &mut Vec<AllocationRecord>,
+    ) -> Result<(), RecoveryFailure> {
+        if !units_seen.insert(pointer.locations[0].slot) {
+            return Err(violated("同一个分配记录树节点被两条父条目引用"));
+        }
+        let bytes = read_node(pointer)?;
+        let is_root = node == AllocationRecordTreeNode::Root;
+        let header = parse_index_node(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: if is_root {
+                "分配记录树根"
+            } else {
+                "分配记录树节点"
+            },
+        })?;
+        self.judge_the_identity_in_the_header(&header, pointer, is_root)?;
+        if header.key_width != key_bytes_width() {
+            return Err(RecoveryFailure::InvariantViolated {
+                invariant: "E142 走读同款",
+                detail: "分配记录树节点自述 key 宽不是 10",
+            });
+        }
+        if header.level != self.geometry.level_of(node) {
+            return Err(violated("分配记录树节点的层级不是它的位置规定的那一层"));
+        }
+        let (smallest_key, largest_key) = self.geometry.key_range_of(node);
+        if header.smallest_key != smallest_key || header.largest_key != largest_key {
+            return Err(violated(
+                "分配记录树节点头里的 key 区间不是它的位置规定罩的那一段",
+            ));
+        }
+        if header.entries.is_empty() {
+            return Err(violated(
+                "分配记录树节点一条条目都没有（缺席的一段不写节点）",
+            ));
+        }
+        if header.level == 0 {
+            let first_new_record = records.len();
+            for entry in &header.entries {
+                let record = AllocationRecord::parse(entry).ok_or(
+                    RecoveryFailure::EntryNarrowerThanItsFieldTable {
+                        what: "分配记录",
+                        entry_bytes: entry.len(),
+                        field_table_bytes: usize::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
+                    },
+                )?;
+                if self.geometry.leaf_of_slot(record.device, record.slot) != node
+                    || !self.geometry.record_fits_in_its_leaf(&record)
+                {
+                    return Err(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
+                        what: "分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽",
+                    });
+                }
+                records.push(record);
+            }
+            let keys_ascend = records[first_new_record..]
+                .windows(2)
+                .all(|pair| pair[0].sort_key() < pair[1].sort_key());
+            if !keys_ascend {
+                return Err(violated("分配记录树叶里的记录不按 key 严格递增"));
+            }
+            nodes.push((node, *pointer, bytes));
+            return Ok(());
+        }
+        let internal_width =
+            usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES).expect("96");
+        if header.entry_width < internal_width {
+            return Err(RecoveryFailure::EntryNarrowerThanItsFieldTable {
+                what: "分配记录树内部条目",
+                entry_bytes: header.entry_width,
+                field_table_bytes: internal_width,
+            });
+        }
+        let mut previous_child: Option<AllocationRecordTreeNode> = None;
+        // 迭代上界是这个节点的条目数；跨轮携带的是上一个孩子（孩子要按位置严格递增）。
+        for entry in &header.entries {
+            let child = self
+                .geometry
+                .child_named_by_entry_key(node, &entry[..key_bytes_width()])
+                .ok_or(violated(
+                    "分配记录树内部条目的 key 不是这个节点里一个孩子那一段的起点",
+                ))?;
+            if previous_child.is_some_and(|previous| previous >= child) {
+                return Err(violated("分配记录树内部节点的孩子不按位置严格递增"));
+            }
+            previous_child = Some(child);
+            let mut reader = ByteReader::at(entry, key_bytes_width());
+            let child_pointer = NodePointer::read_from(&mut reader);
+            self.read_node_and_its_subtree(
+                child,
+                &child_pointer,
+                read_node,
+                units_seen,
+                nodes,
+                records,
+            )?;
+        }
+        nodes.push((node, *pointer, bytes));
+        Ok(())
+    }
+}
+
+/// 从根指针往下读一整棵分配记录树（D8（核心索引结构） 已定项 14：挂载时分配记录树整棵读进挂载态，用户 K4），逐节点按位置核：
+/// 根的层级是池几何定的那一层；每个节点头里的 key 区间是它的位置规定罩的那一段；内部条目的 key 是一个孩子那一段的起点、按位置严格递增；
+/// 孩子的层级是父层级减一；节点不空；叶里的记录按 key 严格递增、每条都落在它所在叶里（末槽不越过叶的末槽）；
+/// 同一个单元不被两条父条目引用。`judgement` 为 `EveryHeaderAgainstItsReference` 时另核头里的树 ID、fsid、诞生、出生序号。
+/// 记录还要对这个池的几何判一遍（设备在池里、槽号在单元区里、跨度不越过单元区末尾、同盘不相交），由调用方做（`recovery`）。
+///
+/// # Errors
+/// 节点读不到、解不开 ⇒ 读者交回的错或 `UnitMalformed`；条目窄于字段表 ⇒ `EntryNarrowerThanItsFieldTable`；
+/// 记录不在它所在叶里 ⇒ `AllocationRecordOutsideThePoolGeometry`；别的核不过 ⇒ `InvariantViolated`（I-1.1 / I-1.2 / I-1.3 / I-1.4）。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "读一棵树要的：根指针、几何、树号、核到哪一步、根、fsid、读节点的口子，各自独立"
+)]
+pub fn read_allocation_record_tree(
+    root_pointer: &NodePointer,
+    geometry: &AllocationRecordTreeGeometry,
+    tree: TreeIdentifier,
+    judgement: AllocationRecordTreeHeaderJudgement,
+    root: &RootRecord,
+    expected_filesystem_identifier: u64,
+    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
+) -> Result<AllocationRecordTreeReadFromDisk, RecoveryFailure> {
+    let reading = AllocationRecordTreeReading {
+        geometry,
+        tree,
+        judgement,
+        root,
+        expected_filesystem_identifier,
+    };
+    let mut nodes = Vec::new();
+    let mut records = Vec::new();
+    let mut units_seen = BTreeSet::new();
+    reading.read_node_and_its_subtree(
+        AllocationRecordTreeNode::Root,
+        root_pointer,
+        read_node,
+        &mut units_seen,
+        &mut nodes,
+        &mut records,
+    )?;
+    nodes.sort_by_key(|(node, _, _)| *node);
+    Ok(AllocationRecordTreeReadFromDisk { nodes, records })
+}
+
+/// 从根指针往下，读得出多少就认多少：交回走得到的每个节点的指针（根在内；读不出、解不开、位置对不上的节点它自己的指针照样交回，
+/// 只是不再往它下面走）。影子账认一条被抛弃根引用着哪些单元时用它（`mount`）：这棵树的节点都是这条根引用的单元，
+/// 少认一个，被抛弃根的那一个就不隔离、回退之后可能被发出去——同实例表那条链「认得出的都算上」。
+#[must_use]
+pub fn node_pointers_as_far_as_readable(
+    root_pointer: &NodePointer,
+    geometry: &AllocationRecordTreeGeometry,
+    read_node: &mut dyn FnMut(&NodePointer) -> Option<Vec<u8>>,
+) -> Vec<NodePointer> {
+    let mut pointers = Vec::new();
+    let mut units_seen = BTreeSet::new();
+    let mut pending = vec![(AllocationRecordTreeNode::Root, *root_pointer)];
+    // 迭代上界：每个节点至多进一次（按它第一条位置条目的槽认过的不再读），节点数有上界。
+    while let Some((node, pointer)) = pending.pop() {
+        if !units_seen.insert(pointer.locations[0].slot) {
+            continue;
+        }
+        pointers.push(pointer);
+        if geometry.level_of(node) == 0 {
+            continue;
+        }
+        let Some(header) = read_node(&pointer).and_then(|bytes| parse_index_node(&bytes).ok())
+        else {
+            continue;
+        };
+        if header.level != geometry.level_of(node)
+            || header.entry_width
+                < usize::try_from(ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES).expect("96")
+        {
+            continue;
+        }
+        for entry in &header.entries {
+            if let Some(child) =
+                geometry.child_named_by_entry_key(node, &entry[..key_bytes_width()])
+            {
+                let child_pointer =
+                    NodePointer::read_from(&mut ByteReader::at(entry, key_bytes_width()));
+                pending.push((child, child_pointer));
+            }
+        }
+    }
+    pointers
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn record(device: u32, slot: u64, span_slots: u16) -> AllocationRecord {
+        AllocationRecord {
+            device: DeviceIdentity(device),
+            slot: SlotNumber(slot),
+            span_slots,
+            generation: CheckpointTxg(1),
+            is_released: false,
+        }
+    }
+
+    fn geometry_of_two_four_gibibyte_devices() -> AllocationRecordTreeGeometry {
+        let slots = (4u64 << 30) / SLOT_BYTES;
+        AllocationRecordTreeGeometry::of_devices(&[
+            (DeviceIdentity(0), slots),
+            (DeviceIdentity(1), slots),
+        ])
+    }
+
+    /// 本地腿两次抽样一致的层数（判决 S1）：4 GiB 两盘 3 层——叶罩 812 槽、层级 1 罩 137228 槽，每盘 262144 槽要两格，两盘四格装进根。
+    #[test]
+    fn two_four_gibibyte_devices_give_a_tree_of_height_three() {
+        let geometry = geometry_of_two_four_gibibyte_devices();
+        assert_eq!(geometry.root_level(), 2);
+        assert_eq!(geometry.height(), 3);
+        assert_eq!(span_in_slots_at_level(1), 812 * 169);
+    }
+
+    /// 1 TiB 两盘 4 层（判决 S1 的 1 TiB 那一格）。
+    #[test]
+    fn two_one_tebibyte_devices_give_a_tree_of_height_four() {
+        let slots = (1u64 << 40) / SLOT_BYTES;
+        let geometry = AllocationRecordTreeGeometry::of_devices(&[
+            (DeviceIdentity(0), slots),
+            (DeviceIdentity(1), slots),
+        ]);
+        assert_eq!(geometry.height(), 4);
+    }
+
+    /// 叶按绝对槽号划：槽 811 与 812 分在两片叶，第 k 片叶的 key 区间就是 `[(盘, k × 812), (盘, (k + 1) × 812 − 1)]`。
+    #[test]
+    fn leaves_split_the_slots_at_multiples_of_the_leaf_width() {
+        let geometry = geometry_of_two_four_gibibyte_devices();
+        let leaf_of = |slot: u64| geometry.leaf_of_slot(DeviceIdentity(1), SlotNumber(slot));
+        assert_ne!(leaf_of(811), leaf_of(812));
+        assert_eq!(leaf_of(812), leaf_of(1623));
+        let (smallest, largest) = geometry.key_range_of(leaf_of(50176));
+        assert_eq!(
+            (smallest, largest),
+            (
+                allocation_record_key_bytes(DeviceIdentity(1), SlotNumber(61 * 812)),
+                allocation_record_key_bytes(DeviceIdentity(1), SlotNumber(62 * 812 - 1))
+            )
+        );
+    }
+
+    /// 一个叶的记录变了：那片叶、它的父节点与根变，同一块盘上另一片叶与另一块盘上的整条路径不变。
+    #[test]
+    fn a_changed_record_changes_its_leaf_and_every_ancestor_only() {
+        let geometry = geometry_of_two_four_gibibyte_devices();
+        let before = vec![
+            record(0, 50176, 2),
+            record(0, 51000, 1),
+            record(1, 50176, 2),
+            record(1, 51000, 1),
+        ];
+        let mut after = before.clone();
+        after[1].is_released = true;
+        let changed = nodes_whose_contents_changed(&geometry, &before, &after);
+        let leaf_of_the_changed_record =
+            geometry.leaf_of_slot(DeviceIdentity(0), SlotNumber(51000));
+        let mut expected = BTreeSet::new();
+        expected.insert(leaf_of_the_changed_record);
+        expected.insert(
+            geometry
+                .parent_of(leaf_of_the_changed_record)
+                .expect("叶有父节点"),
+        );
+        expected.insert(AllocationRecordTreeNode::Root);
+        assert_eq!(changed, expected);
+    }
+
+    /// 父节点里一条条目的 key 不在孩子那一层的格点上、或指到别的盘、或落在父节点那一段之外，都不是它的孩子。
+    #[test]
+    fn an_entry_key_off_the_grid_names_no_child() {
+        let geometry = geometry_of_two_four_gibibyte_devices();
+        let root = AllocationRecordTreeNode::Root;
+        let level_one_start = span_in_slots_at_level(1);
+        assert!(geometry
+            .child_named_by_entry_key(
+                root,
+                &allocation_record_key_bytes(DeviceIdentity(0), SlotNumber(level_one_start))
+            )
+            .is_some());
+        assert!(geometry
+            .child_named_by_entry_key(
+                root,
+                &allocation_record_key_bytes(DeviceIdentity(0), SlotNumber(level_one_start + 2))
+            )
+            .is_none());
+        assert!(geometry
+            .child_named_by_entry_key(
+                root,
+                &allocation_record_key_bytes(DeviceIdentity(7), SlotNumber(0))
+            )
+            .is_none());
+        let level_one = geometry
+            .child_named_by_entry_key(
+                root,
+                &allocation_record_key_bytes(DeviceIdentity(0), SlotNumber(0)),
+            )
+            .expect("盘 0 第 0 个层级 1 节点");
+        assert!(geometry
+            .child_named_by_entry_key(
+                level_one,
+                &allocation_record_key_bytes(DeviceIdentity(1), SlotNumber(812))
+            )
+            .is_none());
+    }
+}
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/allocator.rs tree/crates/singlefs-core/src/allocator.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/allocator.rs	2026-09-24 18:48:58.241408187 +0000
+++ tree/crates/singlefs-core/src/allocator.rs	2026-09-24 19:23:04.554198476 +0000
@@ -627,6 +627,14 @@
     record_at_the_placement_slot
 }
 
+/// 树表 0 条那一版的分配记录树（它的根住根记录那一项，树号 0，节点豁免映射）：每个节点与它的指针，与这一版的全部记录
+/// （按 (设备, 槽号) 升序）——下一次发布按记录判哪几片叶变了、按指针照抄或换下节点。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct AllocationRecordTreeOfTheVersionWithoutFile {
+    pub version: crate::allocation_record_tree::AllocationRecordTreeVersion,
+    pub records: Vec<AllocationRecord>,
+}
+
 /// 一块盘上的一个落点：被抛弃的根引用着的单元按盘记（两盘各一份，D2（RAID 条带策略） 已定项 10）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct PlacementOnDevice {
@@ -733,10 +741,12 @@
     /// mkfs 写出的第 0 版树表单元的落点：第一个文件版本重写树表时把它释放（COW 换下的单元进 defer 队列，D3（空间分配） 已定项 7）；
     /// 重开之后上一版从盘上重建、释放经映射与根记录走，这里留空。
     format_time_tree_table: Option<Placement>,
-    /// 树表 0 条的那一版写行时写下的分配记录树节点的落点（根记录那一项指着它，C512（树表 0 条的一版上被换下的单元记在哪））：
-    /// 这一版上再发一次时（再写一次行，或发第一个文件版本）要把它换下，而那一版没有上一版的内存态可查——
-    /// 与 `format_time_tree_table` 同一个用处。mkfs 的第 0 代与带文件的一版都留空。
-    allocation_record_node_of_the_version_without_file: Option<Placement>,
+    /// 树表 0 条的那一版写行时写下的那棵分配记录树（根记录那一项指着它的根，C512（树表 0 条的一版上被换下的单元记在哪））：
+    /// 每个节点与它的指针，连同那一版的全部记录。这一版上再发一次时（再写一次行，或发第一个文件版本）要照抄或换下它的节点、
+    /// 按它的记录判哪几片叶变了，而那一版没有上一版的内存态可查——与 `format_time_tree_table` 同一个用处。
+    /// mkfs 的第 0 代与带文件的一版都留空。
+    allocation_record_tree_of_the_version_without_file:
+        Option<AllocationRecordTreeOfTheVersionWithoutFile>,
     /// 复用窗口，只供测试的开关（`ReuseWindow`）。只住内存：重开之后按产品路径起步（`rebuild_from_records` 走 `new`）。
     reuse_window: ReuseWindow,
     /// 复用窗口置 0 时在 `release` 里当场回收掉的落点数（逐盘各算一条，与 `reclaim_released_up_to` 的返回值口径不同）：
@@ -763,7 +773,7 @@
             records: Vec::new(),
             reclaimed: BTreeSet::new(),
             format_time_tree_table: None,
-            allocation_record_node_of_the_version_without_file: None,
+            allocation_record_tree_of_the_version_without_file: None,
             reuse_window: ReuseWindow::GatedByTheRollbackFloor,
             placements_reclaimed_on_release_by_the_forced_zero_reuse_window: 0,
             root_ring: None,
@@ -865,19 +875,27 @@
         self.format_time_tree_table
     }
 
-    /// 重开一个「树表 0 条、写过行」的池时记下它那片分配记录树节点住哪（根记录那一项给的落点）：
-    /// 下一次发布要把它换下，而这一版没有上一版的内存态可查。
-    pub fn note_allocation_record_node_of_the_version_without_file(
+    /// 记下树表 0 条那一版的分配记录树（写行那次发布刚写出来的，或重开一个「树表 0 条、写过行」的池时从盘上整棵读回来的）：
+    /// 下一次发布要照抄或换下它的节点，而这一版没有上一版的内存态可查。
+    pub fn note_allocation_record_tree_of_the_version_without_file(
         &mut self,
-        placement: Placement,
+        tree: AllocationRecordTreeOfTheVersionWithoutFile,
     ) {
-        self.allocation_record_node_of_the_version_without_file = Some(placement);
+        self.allocation_record_tree_of_the_version_without_file = Some(tree);
     }
 
-    /// 树表 0 条那一版的分配记录树节点（这一版写过行、而它还没被换下时 `Some`）。
+    /// 树表 0 条那一版的分配记录树（这一版写过行、而它还没被第一个文件版本换下时 `Some`）。
     #[must_use]
-    pub fn allocation_record_node_of_the_version_without_file(&self) -> Option<Placement> {
-        self.allocation_record_node_of_the_version_without_file
+    pub fn allocation_record_tree_of_the_version_without_file(
+        &self,
+    ) -> Option<&AllocationRecordTreeOfTheVersionWithoutFile> {
+        self.allocation_record_tree_of_the_version_without_file
+            .as_ref()
+    }
+
+    /// 第一个文件版本把树表 0 条那一版的分配记录树整棵换下之后（树号从 0 换成那一版发出来的号，节点一个都不照抄）清掉它。
+    pub fn forget_the_allocation_record_tree_of_the_version_without_file(&mut self) {
+        self.allocation_record_tree_of_the_version_without_file = None;
     }
 
     #[must_use]
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/extent_tree.rs tree/crates/singlefs-core/src/extent_tree.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/extent_tree.rs	1970-01-01 00:00:00.000000000 +0000
+++ tree/crates/singlefs-core/src/extent_tree.rs	2026-09-24 20:56:53.036717401 +0000
@@ -0,0 +1,1203 @@
+//! extent 树按 key 空间定形状（D8（核心索引结构） 已定项 14「extent 树」，用户 2026-09-24 定 K2）：两段，都按位置寻址，不分裂。
+//!
+//! - **上段**按 inode 号的位置寻址：叶 j 罩 inode 号 `[j × U, (j + 1) × U)`，U = [`EXTENT_TREE_UPPER_LEAF_INODES`]；层级 L 上第 j 个节点罩
+//!   `[j × U × F^L, (j + 1) × U × F^L)`，F = [`EXTENT_TREE_INTERNAL_FANOUT`]。上段的根是它最高那一层的第 0 个节点（罩从 inode 0 起的一段），
+//!   层级是罩得住有条目的最大 inode 号的最低一层。上段叶条目（[`ExtentUpperLeafEntry`]，113 字节）带一个标签字节区分
+//!   「没有单元（0）/ 下段根指针（1）/ 内联数据指针（2）」。
+//! - **下段**一个文件一棵，按数据单元号的位置寻址：叶罩 [`EXTENT_TREE_LOWER_LEAF_DATA_UNITS`] 个单元，层级 L 上一个节点罩 144 × F^L 个单元，
+//!   根是罩得住这个文件全部单元的最低一层的第 0 个节点。叶的条目是 extent 叶记录 112（key (0, inode, 文件字节偏移) + 数据指针 88），
+//!   内部条目是「孩子那一段的起点 key + 子指针」110。洞就是缺席：没有单元的一段不写节点，父节点那一格留空。
+//! - **只有一个数据单元的文件不建下段**：上段叶条目里直接放那个数据指针（标签 2）。写路径只写标签 1 与 2（今天每个文件版本从偏移 0 顺序写、
+//!   长度 0 的内容也写一个声明长度 0 的数据单元，一个单元都没有的文件写不出来）；标签 0 由读者与 checker 认（「这个 inode 没有单元」）。
+//!
+//! 节点头的 key 区间写这个节点按位置规定罩的那一段（D18（块里携带什么信息） 已定项 2 对按位置寻址的树那一句），key 宽恒是 24
+//! （上段与下段同一棵树、同一个 key 宽，头宽 163）：上段节点罩 `[(0, 首个 inode, 0), (0, 末个 inode, u64 最大)]`，
+//! 下段节点罩 `[(0, inode, 首个单元 × P), (0, inode, (末个单元 + 1) × P − 1)]`（P = 数据单元净荷容量，offset 段是文件字节偏移，D8 已定项 3）。
+//!
+//! 上段叶条目的字段表、上段一片叶罩几个 inode、上段扇出是实现员的取法（D8 已定项 14 交给实现员，交回里写明）。
+//! 这里只管结构：几何、字节怎么装、从盘上读回来并逐节点核位置。取落点、发出生序号、写盘在 `crate::transaction` 那一侧。
+
+use std::collections::BTreeSet;
+
+use singlefs_format::{
+    DATA_POINTER_BYTES, EXTENT_KEY_BYTES, EXTENT_LEAF_RECORD_BYTES,
+    EXTENT_TREE_INTERNAL_ENTRY_BYTES, EXTENT_TREE_INTERNAL_FANOUT,
+    EXTENT_TREE_LOWER_LEAF_DATA_UNITS, EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES,
+    EXTENT_TREE_UPPER_LEAF_INODES, NODE_POINTER_BYTES,
+};
+
+use crate::address::{CheckpointTxg, InstanceGeneration, SlotNumber, TreeIdentifier};
+use crate::bytes::{ByteReader, ByteWriter};
+use crate::pointer::{BirthSequence, DataPointer, NodePointer};
+use crate::records::{build_extent_record, parse_extent_record};
+use crate::recovery::RecoveryFailure;
+use crate::root_record::RootRecord;
+use crate::unit::{
+    build_index_node, data_unit_payload_capacity, parse_index_node, IndexNodeHeader,
+};
+
+/// 上段叶条目的标签：这个 inode 没有数据单元（D8（核心索引结构） 已定项 14 的「没有单元」）。
+pub const EXTENT_UPPER_LEAF_ENTRY_TAG_NO_DATA_UNIT: u8 = 0;
+/// 上段叶条目的标签：载荷是这个文件下段根的节点指针。
+pub const EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT: u8 = 1;
+/// 上段叶条目的标签：载荷是这个文件唯一那个数据单元的数据指针（内联）。
+pub const EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT: u8 = 2;
+
+fn key_width() -> usize {
+    usize::try_from(EXTENT_KEY_BYTES).expect("24")
+}
+
+/// extent 树的 key：(locality 8, inode 8, offset 8)，盘上小端（D8（核心索引结构） 已定项 3）。第一版 locality 恒 0。
+#[must_use]
+pub fn extent_key_bytes(inode: u64, offset_in_bytes: u64) -> Vec<u8> {
+    let mut writer = ByteWriter::new(key_width());
+    writer.put_u64(0);
+    writer.put_u64(inode);
+    writer.put_u64(offset_in_bytes);
+    writer.into_bytes()
+}
+
+/// 一把 extent key 的三段：(locality, inode, offset)。
+#[must_use]
+pub fn extent_key_fields(key: &[u8]) -> (u64, u64, u64) {
+    let mut reader = ByteReader::at(key, 0);
+    (reader.get_u64(), reader.get_u64(), reader.get_u64())
+}
+
+fn payload_capacity_in_bytes() -> u64 {
+    u64::try_from(data_unit_payload_capacity()).expect("32634")
+}
+
+/// 上段层级 L 上一个节点罩几个 inode 号：U × F^L，乘到装不进 u64 就停在 u64 的最大值。
+#[must_use]
+pub fn upper_span_in_inodes(level: u8) -> u64 {
+    (0..level).fold(EXTENT_TREE_UPPER_LEAF_INODES, |span, _| {
+        span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
+    })
+}
+
+/// 下段层级 L 上一个节点罩几个数据单元：144 × F^L，乘到装不进 u64 就停在 u64 的最大值。
+#[must_use]
+pub fn lower_span_in_data_units(level: u8) -> u64 {
+    (0..level).fold(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, |span, _| {
+        span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
+    })
+}
+
+/// 上段罩得住 inode 号 `largest_inode` 的最低一层（上段根的层级）。
+#[must_use]
+pub fn upper_root_level_for(largest_inode: u64) -> u8 {
+    (0..=u8::MAX)
+        .find(|level| upper_span_in_inodes(*level) > largest_inode)
+        .expect("第 8 层起罩的 inode 号已超过 u64 的上界")
+}
+
+/// 下段罩得住 `data_units` 个单元（单元号 0 .. data_units − 1）的最低一层（下段根的层级）。
+#[must_use]
+pub fn lower_root_level_for(data_units: u64) -> u8 {
+    (0..=u8::MAX)
+        .find(|level| lower_span_in_data_units(*level) >= data_units)
+        .expect("第 8 层起罩的单元数已超过 u64 的上界")
+}
+
+/// 上段一个节点的位置：层级（0 是叶）与同层从 inode 0 起数第几个。派生的全序（层级升序、同层按序号升序 = 按 key 升序）
+/// 是上段里的 bump 次序（先叶后根）；上段的根是最高那一层的第 0 个。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub struct ExtentUpperNodePosition {
+    pub level: u8,
+    pub index: u64,
+}
+
+impl ExtentUpperNodePosition {
+    /// 这个位置罩的 inode 号：首个与末个。
+    #[must_use]
+    pub fn inode_range(self) -> (u64, u64) {
+        let span = upper_span_in_inodes(self.level);
+        let first = self.index.saturating_mul(span);
+        (first, first.saturating_add(span - 1))
+    }
+
+    /// 节点头里写的 key 区间：`[(0, 首个 inode, 0), (0, 末个 inode, u64 最大)]`。
+    #[must_use]
+    pub fn key_range(self) -> (Vec<u8>, Vec<u8>) {
+        let (first_inode, last_inode) = self.inode_range();
+        (
+            extent_key_bytes(first_inode, 0),
+            extent_key_bytes(last_inode, u64::MAX),
+        )
+    }
+
+    /// 父节点（上一层）的位置。
+    #[must_use]
+    pub fn parent(self) -> Self {
+        Self {
+            level: self.level + 1,
+            index: self.index / EXTENT_TREE_INTERNAL_FANOUT,
+        }
+    }
+}
+
+/// 上段从 inode `inode` 所在的叶到根（层级 `root_level` 的第 0 个）那一串位置，叶在前、根在末。
+///
+/// # Panics
+/// `root_level` 罩不住这个 inode：调用方按 [`upper_root_level_for`] 取的层级。
+#[must_use]
+pub fn upper_path_of_inode(inode: u64, root_level: u8) -> Vec<ExtentUpperNodePosition> {
+    assert!(
+        upper_span_in_inodes(root_level) > inode,
+        "上段根的层级按有条目的最大 inode 号取，罩得住它"
+    );
+    let mut path = vec![ExtentUpperNodePosition {
+        level: 0,
+        index: inode / EXTENT_TREE_UPPER_LEAF_INODES,
+    }];
+    // 迭代上界是根的层级：每一轮往上一层。
+    while path.last().expect("至少叶那一个").level < root_level {
+        let parent = path.last().expect("至少叶那一个").parent();
+        path.push(parent);
+    }
+    path
+}
+
+/// 下段一个节点的位置（一个文件的下段里）：层级（0 是叶）与同层从单元 0 起数第几个。派生的全序是下段里的 bump 次序（先叶后根）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub struct ExtentLowerNodePosition {
+    pub level: u8,
+    pub index: u64,
+}
+
+impl ExtentLowerNodePosition {
+    /// 这个位置罩的数据单元号：首个与末个。
+    #[must_use]
+    pub fn data_unit_range(self) -> (u64, u64) {
+        let span = lower_span_in_data_units(self.level);
+        let first = self.index.saturating_mul(span);
+        (first, first.saturating_add(span - 1))
+    }
+
+    /// 节点头里写的 key 区间：`[(0, inode, 首个单元 × P), (0, inode, (末个单元 + 1) × P − 1)]`，乘到装不进 u64 就停在最大值。
+    #[must_use]
+    pub fn key_range(self, inode: u64) -> (Vec<u8>, Vec<u8>) {
+        let (first_unit, last_unit) = self.data_unit_range();
+        let payload = payload_capacity_in_bytes();
+        let first_offset = first_unit.saturating_mul(payload);
+        let last_offset = last_unit
+            .saturating_add(1)
+            .saturating_mul(payload)
+            .saturating_sub(1);
+        (
+            extent_key_bytes(inode, first_offset),
+            extent_key_bytes(inode, last_offset),
+        )
+    }
+
+    /// 这个位置下面那一层的孩子里，罩着单元号 `data_unit` 的那一个。
+    #[must_use]
+    pub fn child_holding_data_unit(self, data_unit: u64) -> Self {
+        Self {
+            level: self.level - 1,
+            index: data_unit / lower_span_in_data_units(self.level - 1),
+        }
+    }
+}
+
+/// 一个 `data_units` 个单元（单元号 0 .. data_units − 1、没有洞）的文件，下段的全部节点，按 bump 次序（先叶后根、同层按位置升序）。
+/// 只有一个单元的文件不建下段（内联），交回空的。
+#[must_use]
+pub fn lower_segment_nodes_of_a_file_without_holes(
+    data_units: u64,
+) -> Vec<ExtentLowerNodePosition> {
+    if data_units <= 1 {
+        return Vec::new();
+    }
+    let root_level = lower_root_level_for(data_units);
+    let mut nodes = Vec::new();
+    for level in 0..=root_level {
+        let nodes_in_this_level = data_units.div_ceil(lower_span_in_data_units(level));
+        nodes
+            .extend((0..nodes_in_this_level).map(|index| ExtentLowerNodePosition { level, index }));
+    }
+    nodes
+}
+
+/// 下段一个内部节点这一版的孩子（没有洞的文件里）：罩得着单元 0 .. data_units − 1 的那几个，按位置升序。
+#[must_use]
+pub fn lower_children_of_a_file_without_holes(
+    parent: ExtentLowerNodePosition,
+    data_units: u64,
+) -> Vec<ExtentLowerNodePosition> {
+    let child_level = parent.level - 1;
+    let child_span = lower_span_in_data_units(child_level);
+    let (first_unit, last_unit) = parent.data_unit_range();
+    let last_unit_with_data = last_unit.min(data_units - 1);
+    (first_unit / child_span..=last_unit_with_data / child_span)
+        .map(|index| ExtentLowerNodePosition {
+            level: child_level,
+            index,
+        })
+        .collect()
+}
+
+/// 上段叶条目的载荷：标签 0 / 1 / 2 三种。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ExtentUpperLeafTarget {
+    NoDataUnit,
+    LowerSegmentRoot(NodePointer),
+    InlineDataUnit(DataPointer),
+}
+
+/// 上段叶条目 113：key 24（0, inode, 0）+ 标签 1 + 载荷 88（[`EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct ExtentUpperLeafEntry {
+    pub inode: u64,
+    pub target: ExtentUpperLeafTarget,
+}
+
+/// 一条上段叶条目解不开：盘上读来的字节可以是任何值。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ExtentUpperLeafEntryMalformed {
+    NarrowerThanItsFieldTable,
+    /// 标签不是 0 / 1 / 2。
+    UnrecognizedTag(u8),
+    /// key 的 locality 段或 offset 段不是 0，或载荷里该是零的字节不是零。
+    NonZeroBytesWhereTheFieldTableSaysZero,
+}
+
+impl ExtentUpperLeafEntry {
+    #[must_use]
+    pub fn to_bytes(&self) -> Vec<u8> {
+        let mut writer =
+            ByteWriter::new(usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113"));
+        writer.put(&extent_key_bytes(self.inode, 0));
+        match self.target {
+            ExtentUpperLeafTarget::NoDataUnit => {
+                writer.put_u8(EXTENT_UPPER_LEAF_ENTRY_TAG_NO_DATA_UNIT);
+                writer.skip(usize::try_from(DATA_POINTER_BYTES).expect("88"));
+            }
+            ExtentUpperLeafTarget::LowerSegmentRoot(pointer) => {
+                writer.put_u8(EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT);
+                pointer.write_to(&mut writer);
+                writer.skip(usize::try_from(DATA_POINTER_BYTES - NODE_POINTER_BYTES).expect("2"));
+            }
+            ExtentUpperLeafTarget::InlineDataUnit(pointer) => {
+                writer.put_u8(EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT);
+                pointer.write_to(&mut writer);
+            }
+        }
+        writer.assert_position(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES, "extent 上段叶条目");
+        writer.into_bytes()
+    }
+
+    /// 上段叶条目的读者：这里是盘上字节进字段表的边界。
+    ///
+    /// # Errors
+    /// 窄于 113、标签不认识、该是零的字节不是零。
+    pub fn parse(bytes: &[u8]) -> Result<Self, ExtentUpperLeafEntryMalformed> {
+        if bytes.len() < usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113") {
+            return Err(ExtentUpperLeafEntryMalformed::NarrowerThanItsFieldTable);
+        }
+        let (locality, inode, offset) = extent_key_fields(&bytes[..key_width()]);
+        if locality != 0 || offset != 0 {
+            return Err(ExtentUpperLeafEntryMalformed::NonZeroBytesWhereTheFieldTableSaysZero);
+        }
+        let tag = bytes[key_width()];
+        let payload = &bytes
+            [key_width() + 1..usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113")];
+        let node_pointer_bytes = usize::try_from(NODE_POINTER_BYTES).expect("86");
+        let target = match tag {
+            EXTENT_UPPER_LEAF_ENTRY_TAG_NO_DATA_UNIT => {
+                if payload.iter().any(|byte| *byte != 0) {
+                    return Err(
+                        ExtentUpperLeafEntryMalformed::NonZeroBytesWhereTheFieldTableSaysZero,
+                    );
+                }
+                ExtentUpperLeafTarget::NoDataUnit
+            }
+            EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT => {
+                if payload[node_pointer_bytes..].iter().any(|byte| *byte != 0) {
+                    return Err(
+                        ExtentUpperLeafEntryMalformed::NonZeroBytesWhereTheFieldTableSaysZero,
+                    );
+                }
+                ExtentUpperLeafTarget::LowerSegmentRoot(NodePointer::read_from(
+                    &mut ByteReader::at(payload, 0),
+                ))
+            }
+            EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT => ExtentUpperLeafTarget::InlineDataUnit(
+                DataPointer::read_from(&mut ByteReader::at(payload, 0)),
+            ),
+            unrecognized => {
+                return Err(ExtentUpperLeafEntryMalformed::UnrecognizedTag(unrecognized))
+            }
+        };
+        Ok(Self { inode, target })
+    }
+}
+
+/// 内部节点条目 110：孩子那一段的起点 key 24 + 子指针 86。
+#[must_use]
+pub fn build_extent_internal_entry(child_start_key: &[u8], child: &NodePointer) -> Vec<u8> {
+    let mut writer =
+        ByteWriter::new(usize::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110"));
+    writer.put(child_start_key);
+    child.write_to(&mut writer);
+    writer.into_bytes()
+}
+
+/// 装 extent 树一个节点时整次发布共用的身份字段。
+#[derive(Clone, Copy, Debug)]
+pub struct ExtentTreeNodeIdentity<'identity> {
+    pub tree: TreeIdentifier,
+    pub birth_txg: CheckpointTxg,
+    pub filesystem_identifier: &'identity [u8; 16],
+    pub instance: InstanceGeneration,
+}
+
+/// 装下段一片叶：这片叶罩的那几个单元的 extent 叶记录（key 升序）。`data_pointers` 是这个文件全部单元的指针（第 i 项是单元 i）。
+///
+/// # Panics
+/// 这片叶罩的单元一个都不在这个文件里：规划只交出罩着单元的叶。
+#[must_use]
+pub fn build_lower_leaf(
+    identity: &ExtentTreeNodeIdentity<'_>,
+    inode: u64,
+    position: ExtentLowerNodePosition,
+    data_pointers: &[DataPointer],
+    birth_sequence: BirthSequence,
+) -> Vec<u8> {
+    let (first_unit, last_unit) = position.data_unit_range();
+    let payload = payload_capacity_in_bytes();
+    let records: Vec<Vec<u8>> = data_pointers
+        .iter()
+        .enumerate()
+        .map(|(unit, pointer)| (u64::try_from(unit).expect("单元序号"), pointer))
+        .filter(|(unit, _)| (first_unit..=last_unit).contains(unit))
+        .map(|(unit, pointer)| build_extent_record(inode, unit * payload, *pointer))
+        .collect();
+    assert!(!records.is_empty(), "规划只交出罩着单元的叶");
+    let (smallest_key, largest_key) = position.key_range(inode);
+    build_index_node(
+        identity.tree,
+        0,
+        key_width(),
+        &smallest_key,
+        &largest_key,
+        identity.birth_txg,
+        identity.filesystem_identifier,
+        identity.instance,
+        birth_sequence,
+        u16::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
+        &records,
+    )
+}
+
+/// 装下段一个内部节点：每个孩子一条「孩子那一段的起点 key + 孩子这一版的指针」。
+///
+/// # Panics
+/// 一个孩子都没有：规划只交出罩着单元的节点。
+#[must_use]
+pub fn build_lower_internal_node(
+    identity: &ExtentTreeNodeIdentity<'_>,
+    inode: u64,
+    position: ExtentLowerNodePosition,
+    children: &[(ExtentLowerNodePosition, NodePointer)],
+    birth_sequence: BirthSequence,
+) -> Vec<u8> {
+    assert!(!children.is_empty(), "规划只交出罩着单元的节点");
+    let entries: Vec<Vec<u8>> = children
+        .iter()
+        .map(|(child, pointer)| build_extent_internal_entry(&child.key_range(inode).0, pointer))
+        .collect();
+    let (smallest_key, largest_key) = position.key_range(inode);
+    build_index_node(
+        identity.tree,
+        position.level,
+        key_width(),
+        &smallest_key,
+        &largest_key,
+        identity.birth_txg,
+        identity.filesystem_identifier,
+        identity.instance,
+        birth_sequence,
+        u16::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110"),
+        &entries,
+    )
+}
+
+/// 装上段一片叶：条目按 inode 号升序。
+///
+/// # Panics
+/// 一条条目都没有，或有条目的 inode 号不在这片叶罩的那一段里：规划只把条目放进罩着它的那片叶。
+#[must_use]
+pub fn build_upper_leaf(
+    identity: &ExtentTreeNodeIdentity<'_>,
+    position: ExtentUpperNodePosition,
+    entries: &[ExtentUpperLeafEntry],
+    birth_sequence: BirthSequence,
+) -> Vec<u8> {
+    let (first_inode, last_inode) = position.inode_range();
+    assert!(
+        !entries.is_empty()
+            && entries
+                .iter()
+                .all(|entry| (first_inode..=last_inode).contains(&entry.inode)),
+        "上段叶里的条目都落在它罩的那一段 inode 号里"
+    );
+    let (smallest_key, largest_key) = position.key_range();
+    build_index_node(
+        identity.tree,
+        0,
+        key_width(),
+        &smallest_key,
+        &largest_key,
+        identity.birth_txg,
+        identity.filesystem_identifier,
+        identity.instance,
+        birth_sequence,
+        u16::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES).expect("113"),
+        &entries
+            .iter()
+            .map(ExtentUpperLeafEntry::to_bytes)
+            .collect::<Vec<_>>(),
+    )
+}
+
+/// 装上段一个内部节点：每个孩子一条「孩子那一段的起点 key + 孩子这一版的指针」。
+///
+/// # Panics
+/// 一个孩子都没有：规划只交出有条目的叶的祖先。
+#[must_use]
+pub fn build_upper_internal_node(
+    identity: &ExtentTreeNodeIdentity<'_>,
+    position: ExtentUpperNodePosition,
+    children: &[(ExtentUpperNodePosition, NodePointer)],
+    birth_sequence: BirthSequence,
+) -> Vec<u8> {
+    assert!(!children.is_empty(), "规划只交出有条目的叶的祖先");
+    let entries: Vec<Vec<u8>> = children
+        .iter()
+        .map(|(child, pointer)| build_extent_internal_entry(&child.key_range().0, pointer))
+        .collect();
+    let (smallest_key, largest_key) = position.key_range();
+    build_index_node(
+        identity.tree,
+        position.level,
+        key_width(),
+        &smallest_key,
+        &largest_key,
+        identity.birth_txg,
+        identity.filesystem_identifier,
+        identity.instance,
+        birth_sequence,
+        u16::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110"),
+        &entries,
+    )
+}
+
+/// 一版 extent 树在内存里的样子（第一版只有第一个文件有内容，下段只有它那一棵）：上段每个节点与它这一版的指针（bump 次序，根在最末）、
+/// 第一个文件下段每个节点与它这一版的指针（bump 次序；内联时为空）。节点的字节住 `transaction::TransactionOutput::units`。
+#[derive(Clone, Debug, Default, PartialEq, Eq)]
+pub struct ExtentTreeVersion {
+    pub upper_nodes: Vec<(ExtentUpperNodePosition, NodePointer)>,
+    pub lower_nodes: Vec<(ExtentLowerNodePosition, NodePointer)>,
+}
+
+impl ExtentTreeVersion {
+    /// 上段根的位置（最高那一层的第 0 个）。
+    ///
+    /// # Panics
+    /// 上段一个节点都没有：带文件的一版里第一个文件恒有一条上段叶条目。
+    #[must_use]
+    pub fn upper_root(&self) -> ExtentUpperNodePosition {
+        self.upper_nodes.last().expect("带文件的一版里上段恒有根").0
+    }
+
+    /// 上段的高（上段根的层级 + 1，D8（核心索引结构） 已定项 11 ⑤ 的读法）。
+    #[must_use]
+    pub fn upper_height(&self) -> u64 {
+        u64::from(self.upper_root().level) + 1
+    }
+
+    /// 第一个文件下段的高；内联（没有下段）时 0。
+    #[must_use]
+    pub fn lower_height(&self) -> u64 {
+        self.lower_nodes
+            .last()
+            .map_or(0, |(position, _)| u64::from(position.level) + 1)
+    }
+
+    #[must_use]
+    pub fn upper_pointer_of(&self, position: ExtentUpperNodePosition) -> Option<NodePointer> {
+        self.upper_nodes
+            .iter()
+            .find(|(candidate, _)| *candidate == position)
+            .map(|(_, pointer)| *pointer)
+    }
+
+    #[must_use]
+    pub fn lower_pointer_of(&self, position: ExtentLowerNodePosition) -> Option<NodePointer> {
+        self.lower_nodes
+            .iter()
+            .find(|(candidate, _)| *candidate == position)
+            .map(|(_, pointer)| *pointer)
+    }
+}
+
+/// 从盘上读 extent 树时核到哪一步（同分配记录树那两档）：**位置那几样两档都核**——层级、节点头的 key 区间是位置规定的那一段、
+/// 内部条目的 key 是一个孩子那一段的起点、按位置严格递增、叶条目落在叶罩的那一段里、按 key 严格递增。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ExtentTreeHeaderJudgement {
+    /// 另核每个节点头的树 ID、fsid、诞生不晚于根、出生序号与指着它的指针相同。
+    EveryHeaderAgainstItsReference,
+    /// 头里的出生身份不核（从盘上重建上一版）。
+    OnlyWhatThePositionsNeed,
+}
+
+/// 读 extent 树时一路不变的那几样。
+#[derive(Clone, Copy)]
+pub struct ExtentTreeReading<'reading> {
+    pub tree: TreeIdentifier,
+    pub judgement: ExtentTreeHeaderJudgement,
+    pub root: &'reading RootRecord,
+    pub expected_filesystem_identifier: u64,
+}
+
+fn violated(detail: &'static str) -> RecoveryFailure {
+    RecoveryFailure::InvariantViolated {
+        invariant: "I-1.1",
+        detail,
+    }
+}
+
+impl ExtentTreeReading<'_> {
+    /// 读一个节点、核头与位置：交回解开的头。`level_and_key_range` 是这个位置的层级与它按位置规定罩的那一段；
+    /// `already_read` 是调用方为了先认出根的层级已经读过一遍的字节（根那一个），不再读第二次；
+    /// `already_read_is_the_root_of_the_tree` 说这是不是整棵树的根（上段的根）。
+    fn read_and_judge_node(
+        &self,
+        pointer: &NodePointer,
+        level_and_key_range: (u8, &(Vec<u8>, Vec<u8>)),
+        already_read: Option<Vec<u8>>,
+        already_read_is_the_root_of_the_tree: bool,
+        read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
+        units_seen: &mut BTreeSet<SlotNumber>,
+    ) -> Result<(Vec<u8>, IndexNodeHeader), RecoveryFailure> {
+        let (expected_level, key_range) = level_and_key_range;
+        if !units_seen.insert(pointer.locations[0].slot) {
+            return Err(violated("同一个 extent 树节点被两条父条目引用"));
+        }
+        let bytes = match already_read {
+            Some(bytes) => bytes,
+            None => read_node(pointer)?,
+        };
+        // 整棵树的根（上段的根，树表条目指着它）那一个的报错逐字沿用 `recovery` 核树根的那几句（同 `code_two_tree` 的做法）。
+        let is_the_root_of_the_tree = already_read_is_the_root_of_the_tree;
+        let detail = |of_the_root: &'static str, of_a_node_below_the_root: &'static str| {
+            if is_the_root_of_the_tree {
+                of_the_root
+            } else {
+                of_a_node_below_the_root
+            }
+        };
+        let header = parse_index_node(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: detail("树根节点", "extent 树节点"),
+        })?;
+        match self.judgement {
+            ExtentTreeHeaderJudgement::OnlyWhatThePositionsNeed => {}
+            ExtentTreeHeaderJudgement::EveryHeaderAgainstItsReference => {
+                if header.tree != self.tree {
+                    return Err(RecoveryFailure::InvariantViolated {
+                        invariant: "I-1.3",
+                        detail: detail(
+                            "根头里的树 ID 与树表不符",
+                            "extent 树节点头里的树 ID 与引用它的树不符",
+                        ),
+                    });
+                }
+                if header.key_width != key_width() {
+                    return Err(RecoveryFailure::InvariantViolated {
+                        invariant: "E142 走读同款",
+                        detail: detail(
+                            "根自述 key 宽与树的种类不符",
+                            "extent 树节点自述 key 宽不是 24",
+                        ),
+                    });
+                }
+                if header.birth_txg > self.root.checkpoint_txg
+                    || header.instance > self.root.instance
+                {
+                    return Err(RecoveryFailure::InvariantViolated {
+                        invariant: "I-1.2",
+                        detail: detail("树根诞生于根之后", "extent 树节点诞生于根之后"),
+                    });
+                }
+                if header.filesystem_identifier != self.expected_filesystem_identifier {
+                    return Err(RecoveryFailure::InvariantViolated {
+                        invariant: "I-1.4",
+                        detail: detail("树根 fsid 不符", "extent 树节点 fsid 不符"),
+                    });
+                }
+                if header.birth_sequence != pointer.birth_sequence {
+                    return Err(RecoveryFailure::InvariantViolated {
+                        invariant: "I-1.2",
+                        detail: detail("树根出生序号与指针不符", "extent 树节点出生序号与指针不符"),
+                    });
+                }
+            }
+        }
+        if header.key_width != key_width() {
+            return Err(RecoveryFailure::InvariantViolated {
+                invariant: "E142 走读同款",
+                detail: "extent 树节点自述 key 宽不是 24",
+            });
+        }
+        if header.level != expected_level {
+            return Err(violated("extent 树节点的层级不是它的位置规定的那一层"));
+        }
+        if header.smallest_key != key_range.0 || header.largest_key != key_range.1 {
+            return Err(violated(
+                "extent 树节点头里的 key 区间不是它的位置规定罩的那一段",
+            ));
+        }
+        if header.entries.is_empty() {
+            return Err(violated(
+                "extent 树节点一条条目都没有（缺席的一段不写节点）",
+            ));
+        }
+        Ok((bytes, header))
+    }
+
+    /// 解一个内部节点的条目：每条的 key 是一个孩子那一段的起点，孩子按位置严格递增。交回 (孩子那一段的起点 key, 子指针)。
+    fn internal_entries(
+        header: &IndexNodeHeader,
+    ) -> Result<Vec<(Vec<u8>, NodePointer)>, RecoveryFailure> {
+        let internal_width = usize::try_from(EXTENT_TREE_INTERNAL_ENTRY_BYTES).expect("110");
+        if header.entry_width < internal_width {
+            return Err(RecoveryFailure::EntryNarrowerThanItsFieldTable {
+                what: "extent 树内部条目",
+                entry_bytes: header.entry_width,
+                field_table_bytes: internal_width,
+            });
+        }
+        Ok(header
+            .entries
+            .iter()
+            .map(|entry| {
+                (
+                    entry[..key_width()].to_vec(),
+                    NodePointer::read_from(&mut ByteReader::at(entry, key_width())),
+                )
+            })
+            .collect())
+    }
+}
+
+/// 从盘上读回来的一个文件的下段：每个节点、它的指针与字节（bump 次序），与全部数据指针（按单元号升序，带单元号）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct ExtentLowerSegmentReadFromDisk {
+    pub nodes: Vec<(ExtentLowerNodePosition, NodePointer, Vec<u8>)>,
+    pub data_pointers: Vec<(u64, DataPointer)>,
+}
+
+/// 从下段根指针往下读一个文件的整个下段，逐节点按位置核：根的层级由根节点头自述、它罩的那一段要是从单元 0 起的那一格；
+/// 每个节点头的 key 区间是它的位置规定的那一段；内部条目的 key 是一个孩子那一段的起点（inode 段是这个文件、offset 段落在孩子那一层的格点上、
+/// 在父节点那一段里）、按位置严格递增；孩子的层级是父层级减一；叶里的记录 inode 段是这个文件、offset 段是单元号 × 净荷容量、
+/// 落在叶罩的那一段里、按 key 严格递增；同一个单元不被两条父条目引用。
+///
+/// # Errors
+/// 节点读不到、解不开 ⇒ 读者交回的错或 `UnitMalformed`；条目窄于字段表 ⇒ `EntryNarrowerThanItsFieldTable`；别的核不过 ⇒ `InvariantViolated`。
+pub fn read_lower_segment(
+    reading: &ExtentTreeReading<'_>,
+    inode: u64,
+    root_pointer: &NodePointer,
+    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
+    units_seen: &mut BTreeSet<SlotNumber>,
+) -> Result<ExtentLowerSegmentReadFromDisk, RecoveryFailure> {
+    // 根的层级只有它自己的头说得出：先读出来认层级，再按那一层第 0 格核它（字节交下去，不读第二次）。
+    let bytes = read_node(root_pointer)?;
+    let root_level = parse_index_node(&bytes)
+        .map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: "extent 树下段根",
+        })?
+        .level;
+    let mut segment = ExtentLowerSegmentReadFromDisk {
+        nodes: Vec::new(),
+        data_pointers: Vec::new(),
+    };
+    read_lower_node_and_its_subtree(
+        reading,
+        inode,
+        ExtentLowerNodePosition {
+            level: root_level,
+            index: 0,
+        },
+        root_pointer,
+        Some(bytes),
+        read_node,
+        units_seen,
+        &mut segment,
+    )?;
+    segment.nodes.sort_by_key(|(position, _, _)| *position);
+    Ok(segment)
+}
+
+#[allow(
+    clippy::too_many_arguments,
+    reason = "读一棵子树要的：一路不变的那几样、哪个文件、哪个位置、指针、已经读过的字节、读节点的口子、见过的单元、读出来的东西"
+)]
+fn read_lower_node_and_its_subtree(
+    reading: &ExtentTreeReading<'_>,
+    inode: u64,
+    position: ExtentLowerNodePosition,
+    pointer: &NodePointer,
+    already_read: Option<Vec<u8>>,
+    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
+    units_seen: &mut BTreeSet<SlotNumber>,
+    segment: &mut ExtentLowerSegmentReadFromDisk,
+) -> Result<(), RecoveryFailure> {
+    let (bytes, header) = reading.read_and_judge_node(
+        pointer,
+        (position.level, &position.key_range(inode)),
+        already_read,
+        false,
+        read_node,
+        units_seen,
+    )?;
+    if position.level == 0 {
+        let (first_unit, last_unit) = position.data_unit_range();
+        let payload = payload_capacity_in_bytes();
+        let mut previous_unit: Option<u64> = None;
+        for entry in &header.entries {
+            let (key, data_pointer) = parse_extent_record(entry).ok_or(
+                RecoveryFailure::EntryNarrowerThanItsFieldTable {
+                    what: "extent 叶记录",
+                    entry_bytes: entry.len(),
+                    field_table_bytes: usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
+                },
+            )?;
+            let (locality, record_inode, offset) = extent_key_fields(&key);
+            if locality != 0 || record_inode != inode || !offset.is_multiple_of(payload) {
+                return Err(violated(
+                    "extent 叶记录的 key 不是 (0, 这个文件, 单元号 × 净荷容量)",
+                ));
+            }
+            let unit = offset / payload;
+            if !(first_unit..=last_unit).contains(&unit)
+                || previous_unit.is_some_and(|previous| previous >= unit)
+            {
+                return Err(violated(
+                    "extent 叶记录不在叶罩的那一段里，或不按 key 严格递增",
+                ));
+            }
+            previous_unit = Some(unit);
+            segment.data_pointers.push((unit, data_pointer));
+        }
+        segment.nodes.push((position, *pointer, bytes));
+        return Ok(());
+    }
+    let mut previous_child: Option<ExtentLowerNodePosition> = None;
+    // 迭代上界是这个节点的条目数；跨轮携带的是上一个孩子（孩子要按位置严格递增）。
+    for (child_start_key, child_pointer) in ExtentTreeReading::internal_entries(&header)? {
+        let (locality, child_inode, offset) = extent_key_fields(&child_start_key);
+        let payload = payload_capacity_in_bytes();
+        let child_span = lower_span_in_data_units(position.level - 1);
+        let child_offset_span = child_span.saturating_mul(payload);
+        if locality != 0 || child_inode != inode || !offset.is_multiple_of(child_offset_span) {
+            return Err(violated(
+                "extent 下段内部条目的 key 不是这个文件里一个孩子那一段的起点",
+            ));
+        }
+        let child = ExtentLowerNodePosition {
+            level: position.level - 1,
+            index: offset / child_offset_span,
+        };
+        if child.data_unit_range().0 / lower_span_in_data_units(position.level) != position.index
+            || previous_child.is_some_and(|previous| previous >= child)
+        {
+            return Err(violated(
+                "extent 下段内部条目的孩子不在父节点那一段里，或不按位置严格递增",
+            ));
+        }
+        previous_child = Some(child);
+        read_lower_node_and_its_subtree(
+            reading,
+            inode,
+            child,
+            &child_pointer,
+            None,
+            read_node,
+            units_seen,
+            segment,
+        )?;
+    }
+    segment.nodes.push((position, *pointer, bytes));
+    Ok(())
+}
+
+/// 上段一个内部节点的条目指的孩子：条目 key 是孩子那一段的起点 (0, 首个 inode, 0)，首个 inode 落在孩子那一层的格点上、在父节点那一段里。
+fn upper_child_named_by_entry_key(
+    parent: ExtentUpperNodePosition,
+    child_start_key: &[u8],
+) -> Option<ExtentUpperNodePosition> {
+    let (locality, first_inode, offset) = extent_key_fields(child_start_key);
+    let child_level = parent.level.checked_sub(1)?;
+    let child_span = upper_span_in_inodes(child_level);
+    if locality != 0 || offset != 0 || !first_inode.is_multiple_of(child_span) {
+        return None;
+    }
+    let child = ExtentUpperNodePosition {
+        level: child_level,
+        index: first_inode / child_span,
+    };
+    (child.parent() == parent).then_some(child)
+}
+
+/// 上段一片叶的条目：每条落在叶罩的那一段 inode 号里、按 inode 号严格递增。
+fn upper_leaf_entries(
+    position: ExtentUpperNodePosition,
+    header: &IndexNodeHeader,
+) -> Result<Vec<ExtentUpperLeafEntry>, RecoveryFailure> {
+    let (first_inode, last_inode) = position.inode_range();
+    let mut entries: Vec<ExtentUpperLeafEntry> = Vec::with_capacity(header.entries.len());
+    for entry_bytes in &header.entries {
+        let entry =
+            ExtentUpperLeafEntry::parse(entry_bytes).map_err(|malformed| match malformed {
+                ExtentUpperLeafEntryMalformed::NarrowerThanItsFieldTable => {
+                    RecoveryFailure::EntryNarrowerThanItsFieldTable {
+                        what: "extent 上段叶条目",
+                        entry_bytes: entry_bytes.len(),
+                        field_table_bytes: usize::try_from(EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES)
+                            .expect("113"),
+                    }
+                }
+                ExtentUpperLeafEntryMalformed::UnrecognizedTag(_) => {
+                    RecoveryFailure::InvariantViolated {
+                        invariant: "E142 走读同款",
+                        detail: "extent 上段叶条目的标签不是 0 / 1 / 2",
+                    }
+                }
+                ExtentUpperLeafEntryMalformed::NonZeroBytesWhereTheFieldTableSaysZero => {
+                    RecoveryFailure::InvariantViolated {
+                        invariant: "E142 走读同款",
+                        detail: "extent 上段叶条目里字段表写零的字节不是零",
+                    }
+                }
+            })?;
+        if !(first_inode..=last_inode).contains(&entry.inode)
+            || entries
+                .last()
+                .is_some_and(|previous| previous.inode >= entry.inode)
+        {
+            return Err(violated(
+                "extent 上段叶条目不在叶罩的那一段 inode 号里，或不按 inode 号严格递增",
+            ));
+        }
+        entries.push(entry);
+    }
+    Ok(entries)
+}
+
+/// 从盘上读回来的 extent 树上段：每个节点、它的指针与字节（bump 次序，根在最末），与全部叶条目（按 inode 号升序）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct ExtentUpperSegmentReadFromDisk {
+    pub nodes: Vec<(ExtentUpperNodePosition, NodePointer, Vec<u8>)>,
+    pub entries: Vec<ExtentUpperLeafEntry>,
+}
+
+/// 从 extent 树根指针往下读整个上段，逐节点按位置核（同下段那几样，inode 号代单元号）；叶条目解开、核标签。不读下段。
+///
+/// # Errors
+/// 同 [`read_lower_segment`]；叶条目标签不认识、字段表写零的字节不是零 ⇒ `InvariantViolated`（E142 走读同款）。
+pub fn read_upper_segment(
+    reading: &ExtentTreeReading<'_>,
+    root_pointer: &NodePointer,
+    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
+    units_seen: &mut BTreeSet<SlotNumber>,
+) -> Result<ExtentUpperSegmentReadFromDisk, RecoveryFailure> {
+    let bytes = read_node(root_pointer)?;
+    let root_level = parse_index_node(&bytes)
+        .map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: "extent 树根",
+        })?
+        .level;
+    let mut segment = ExtentUpperSegmentReadFromDisk {
+        nodes: Vec::new(),
+        entries: Vec::new(),
+    };
+    read_upper_node_and_its_subtree(
+        reading,
+        ExtentUpperNodePosition {
+            level: root_level,
+            index: 0,
+        },
+        root_pointer,
+        Some(bytes),
+        read_node,
+        units_seen,
+        &mut segment,
+    )?;
+    segment.nodes.sort_by_key(|(position, _, _)| *position);
+    Ok(segment)
+}
+
+#[allow(
+    clippy::too_many_arguments,
+    reason = "读一棵子树要的：一路不变的那几样、哪个位置、指针、已经读过的字节、读节点的口子、见过的单元、读出来的东西"
+)]
+fn read_upper_node_and_its_subtree(
+    reading: &ExtentTreeReading<'_>,
+    position: ExtentUpperNodePosition,
+    pointer: &NodePointer,
+    already_read: Option<Vec<u8>>,
+    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
+    units_seen: &mut BTreeSet<SlotNumber>,
+    segment: &mut ExtentUpperSegmentReadFromDisk,
+) -> Result<(), RecoveryFailure> {
+    let is_the_root_of_the_tree = already_read.is_some();
+    let (bytes, header) = reading.read_and_judge_node(
+        pointer,
+        (position.level, &position.key_range()),
+        already_read,
+        is_the_root_of_the_tree,
+        read_node,
+        units_seen,
+    )?;
+    if position.level == 0 {
+        segment
+            .entries
+            .extend(upper_leaf_entries(position, &header)?);
+        segment.nodes.push((position, *pointer, bytes));
+        return Ok(());
+    }
+    let mut previous_child: Option<ExtentUpperNodePosition> = None;
+    // 迭代上界是这个节点的条目数；跨轮携带的是上一个孩子（孩子要按位置严格递增）。
+    for (child_start_key, child_pointer) in ExtentTreeReading::internal_entries(&header)? {
+        let child = upper_child_named_by_entry_key(position, &child_start_key)
+            .filter(|child| previous_child.is_none_or(|previous| previous < *child))
+            .ok_or(violated(
+                "extent 上段内部条目的 key 不是这个节点里一个孩子那一段的起点，或孩子不按位置严格递增",
+            ))?;
+        previous_child = Some(child);
+        read_upper_node_and_its_subtree(
+            reading,
+            child,
+            &child_pointer,
+            None,
+            read_node,
+            units_seen,
+            segment,
+        )?;
+    }
+    segment.nodes.push((position, *pointer, bytes));
+    Ok(())
+}
+
+/// 按需读（D8（核心索引结构） 已定项 14「挂载怎么读」：extent 树按需，打开文件时按位置走下去，用户 K4）：从上段根按 inode 号的位置
+/// 走到罩着它的那片叶，交回它那一条叶条目与这一趟读了几个节点；那一段缺席（父节点那一格留空）或叶里没有这个 inode 交回 `None`。
+/// 只读这一条路径上的节点，每个节点照样按位置核。
+///
+/// # Errors
+/// 同 [`read_upper_segment`]。
+pub fn find_upper_leaf_entry(
+    reading: &ExtentTreeReading<'_>,
+    root_pointer: &NodePointer,
+    inode: u64,
+    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
+) -> Result<(Option<ExtentUpperLeafEntry>, u64), RecoveryFailure> {
+    let mut units_seen = BTreeSet::new();
+    let bytes = read_node(root_pointer)?;
+    let root_level = parse_index_node(&bytes)
+        .map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: "extent 树根",
+        })?
+        .level;
+    let mut position = ExtentUpperNodePosition {
+        level: root_level,
+        index: 0,
+    };
+    let mut pointer = *root_pointer;
+    let mut already_read = Some(bytes);
+    let mut nodes_read = 0u64;
+    // 迭代上界是根的层级 + 1：每一轮往下一层，叶那一轮交回。
+    loop {
+        let is_the_root_of_the_tree = already_read.is_some();
+        let (_, header) = reading.read_and_judge_node(
+            &pointer,
+            (position.level, &position.key_range()),
+            already_read.take(),
+            is_the_root_of_the_tree,
+            read_node,
+            &mut units_seen,
+        )?;
+        nodes_read += 1;
+        let (first_inode, last_inode) = position.inode_range();
+        if !(first_inode..=last_inode).contains(&inode) {
+            return Ok((None, nodes_read));
+        }
+        if position.level == 0 {
+            let entry = upper_leaf_entries(position, &header)?
+                .into_iter()
+                .find(|entry| entry.inode == inode);
+            return Ok((entry, nodes_read));
+        }
+        let mut next = None;
+        let mut previous_child: Option<ExtentUpperNodePosition> = None;
+        for (child_start_key, child_pointer) in ExtentTreeReading::internal_entries(&header)? {
+            let child = upper_child_named_by_entry_key(position, &child_start_key)
+                .filter(|child| previous_child.is_none_or(|previous| previous < *child))
+                .ok_or(violated(
+                    "extent 上段内部条目的 key 不是这个节点里一个孩子那一段的起点，或孩子不按位置严格递增",
+                ))?;
+            previous_child = Some(child);
+            let (child_first_inode, child_last_inode) = child.inode_range();
+            if (child_first_inode..=child_last_inode).contains(&inode) {
+                next = Some((child, child_pointer));
+            }
+        }
+        let Some((child, child_pointer)) = next else {
+            return Ok((None, nodes_read));
+        };
+        position = child;
+        pointer = child_pointer;
+    }
+}
+
+/// 一个文件在 extent 树里的样子：没有单元、内联的那一个单元、或下段。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum ExtentsOfAFileReadFromDisk {
+    NoDataUnit,
+    Inline(DataPointer),
+    LowerSegment(ExtentLowerSegmentReadFromDisk),
+}
+
+impl ExtentsOfAFileReadFromDisk {
+    /// 这个文件的数据指针，带单元号，按单元号升序。
+    #[must_use]
+    pub fn data_pointers(&self) -> Vec<(u64, DataPointer)> {
+        match self {
+            ExtentsOfAFileReadFromDisk::NoDataUnit => Vec::new(),
+            ExtentsOfAFileReadFromDisk::Inline(pointer) => vec![(0, *pointer)],
+            ExtentsOfAFileReadFromDisk::LowerSegment(segment) => segment.data_pointers.clone(),
+        }
+    }
+}
+
+/// 从盘上读回来的整棵 extent 树：上段，与每条标签 1 的叶条目指的下段（按 inode 号升序）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct ExtentTreeReadFromDisk {
+    pub upper: ExtentUpperSegmentReadFromDisk,
+    pub files: Vec<(u64, ExtentsOfAFileReadFromDisk)>,
+}
+
+/// 从根指针往下读整棵 extent 树（冷走读、从盘上重建上一版）：上段整段，再按每条叶条目读那个文件的下段；每个节点按位置核，
+/// 同一个单元不被两条父条目引用（上段与各文件的下段之间也不许共用）。
+///
+/// # Errors
+/// 同 [`read_upper_segment`] 与 [`read_lower_segment`]。
+pub fn read_extent_tree(
+    reading: &ExtentTreeReading<'_>,
+    root_pointer: &NodePointer,
+    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
+) -> Result<ExtentTreeReadFromDisk, RecoveryFailure> {
+    let mut units_seen = BTreeSet::new();
+    let upper = read_upper_segment(reading, root_pointer, read_node, &mut units_seen)?;
+    let mut files = Vec::with_capacity(upper.entries.len());
+    for entry in &upper.entries {
+        let extents = match entry.target {
+            ExtentUpperLeafTarget::NoDataUnit => ExtentsOfAFileReadFromDisk::NoDataUnit,
+            ExtentUpperLeafTarget::InlineDataUnit(pointer) => {
+                ExtentsOfAFileReadFromDisk::Inline(pointer)
+            }
+            ExtentUpperLeafTarget::LowerSegmentRoot(lower_root) => {
+                ExtentsOfAFileReadFromDisk::LowerSegment(read_lower_segment(
+                    reading,
+                    entry.inode,
+                    &lower_root,
+                    read_node,
+                    &mut units_seen,
+                )?)
+            }
+        };
+        files.push((entry.inode, extents));
+    }
+    Ok(ExtentTreeReadFromDisk { upper, files })
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// 下段：两个单元就要建下段（根兼叶），144 个还是一片叶，145 个长出层级 1（两片叶加一个根）。
+    #[test]
+    fn a_lower_segment_grows_a_level_past_one_hundred_and_forty_four_data_units() {
+        assert!(lower_segment_nodes_of_a_file_without_holes(1).is_empty());
+        assert_eq!(
+            lower_segment_nodes_of_a_file_without_holes(2),
+            vec![ExtentLowerNodePosition { level: 0, index: 0 }]
+        );
+        assert_eq!(lower_segment_nodes_of_a_file_without_holes(144).len(), 1);
+        assert_eq!(
+            lower_segment_nodes_of_a_file_without_holes(145),
+            vec![
+                ExtentLowerNodePosition { level: 0, index: 0 },
+                ExtentLowerNodePosition { level: 0, index: 1 },
+                ExtentLowerNodePosition { level: 1, index: 0 },
+            ]
+        );
+        assert_eq!(
+            lower_children_of_a_file_without_holes(
+                ExtentLowerNodePosition { level: 1, index: 0 },
+                145
+            ),
+            vec![
+                ExtentLowerNodePosition { level: 0, index: 0 },
+                ExtentLowerNodePosition { level: 0, index: 1 },
+            ]
+        );
+    }
+
+    /// 上段：inode 1 落在第 0 片叶，上段根就是那片叶；inode 143 起要长一层。
+    #[test]
+    fn the_upper_segment_of_inode_one_is_a_single_leaf() {
+        assert_eq!(upper_root_level_for(1), 0);
+        assert_eq!(upper_root_level_for(142), 0);
+        assert_eq!(upper_root_level_for(143), 1);
+        assert_eq!(
+            upper_path_of_inode(143, 1),
+            vec![
+                ExtentUpperNodePosition { level: 0, index: 1 },
+                ExtentUpperNodePosition { level: 1, index: 0 },
+            ]
+        );
+    }
+
+    /// 上段叶条目 113 字节来回：三种标签各解回自己；不认识的标签、标签 1 的补齐非零都解不开。
+    #[test]
+    fn upper_leaf_entries_round_trip_and_refuse_an_unknown_tag() {
+        let no_unit = ExtentUpperLeafEntry {
+            inode: 7,
+            target: ExtentUpperLeafTarget::NoDataUnit,
+        };
+        let lower = ExtentUpperLeafEntry {
+            inode: 8,
+            target: ExtentUpperLeafTarget::LowerSegmentRoot(NodePointer::empty_root()),
+        };
+        for entry in [no_unit, lower] {
+            let bytes = entry.to_bytes();
+            assert_eq!(bytes.len(), 113);
+            assert_eq!(ExtentUpperLeafEntry::parse(&bytes), Ok(entry));
+        }
+        let mut unknown = no_unit.to_bytes();
+        unknown[24] = 3;
+        assert_eq!(
+            ExtentUpperLeafEntry::parse(&unknown),
+            Err(ExtentUpperLeafEntryMalformed::UnrecognizedTag(3))
+        );
+        let mut padding = lower.to_bytes();
+        padding[112] = 1;
+        assert_eq!(
+            ExtentUpperLeafEntry::parse(&padding),
+            Err(ExtentUpperLeafEntryMalformed::NonZeroBytesWhereTheFieldTableSaysZero)
+        );
+    }
+}
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/lib.rs tree/crates/singlefs-core/src/lib.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/lib.rs	2026-09-24 18:48:58.241504616 +0000
+++ tree/crates/singlefs-core/src/lib.rs	2026-09-24 19:18:53.194747347 +0000
@@ -8,11 +8,13 @@
 
 pub mod address;
 pub mod admission;
+pub mod allocation_record_tree;
 pub mod allocator;
 pub mod block_device;
 pub mod bytes;
 pub mod checksum;
 pub mod code_two_tree;
+pub mod extent_tree;
 pub mod inode_tree;
 pub mod instance_table;
 pub mod journal;
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/mounted_read.rs tree/crates/singlefs-core/src/mounted_read.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/mounted_read.rs	2026-09-24 16:50:54.714217512 +0000
+++ tree/crates/singlefs-core/src/mounted_read.rs	2026-09-24 22:03:32.608439714 +0000
@@ -17,8 +17,9 @@
 //!   记录条数要等于文件大小按 D4（校验和位置） 已定项 5 的除法算出来的单元数；对不上就拒绝打开这个文件，不猜。
 //!   于是「第 i 条记录就是文件第 i 个数据单元」这件事由 key 本身担保，解引用时再核单元头的锚点偏移与 key 相等。
 //!
-//! extent 树的节点只在打开时读（[`MountedPoolForRead::extent_tree_reads_at_open`]）：之后按偏移读一个字节都不再碰它，
-//! 顺序读 M 个单元时 extent 节点的读取次数 ≤ 树高 + 叶数（里程碑「第二个事务」并行线一验收第 2 条）。
+//! extent 树按需读（D8（核心索引结构） 已定项 14「挂载怎么读」，用户 K4）：打开池时不读它，打开一个文件时从上段根按 inode 号的位置
+//! 走到它那一条叶条目、再把它的下段整段读下来（内联的就是那一个数据指针）——[`OpenFileForRead::extent_tree_reads_at_open`]；
+//! 之后按偏移读一个字节都不再碰它，顺序读 M 个单元时 extent 节点的读取次数 ≤ 树高 + 叶数（里程碑「第二个事务」并行线一验收第 2 条）。
 
 use std::cell::Cell;
 
@@ -29,10 +30,14 @@
 };
 use crate::checksum::crc32_castagnoli;
 use crate::code_two_tree::{read_code_two_tree, CodeTwoTreeHeaderJudgement};
+use crate::extent_tree::{
+    extent_key_bytes, find_upper_leaf_entry, read_lower_segment, ExtentTreeHeaderJudgement,
+    ExtentTreeReading, ExtentUpperLeafTarget,
+};
 use crate::pointer::{DataPointer, LocationEntry, NodePointer};
 use crate::records::{
-    mapping_key_for_data, parse_extent_record, parse_inode_internal_entry, parse_mapping_entry,
-    InodeRecord, TreeTableEntry, TREE_KIND_EXTENT, TREE_KIND_INODE,
+    mapping_key_for_data, parse_inode_internal_entry, parse_mapping_entry, InodeRecord,
+    TreeTableEntry, TREE_KIND_EXTENT, TREE_KIND_INODE,
 };
 use crate::recovery::{
     choose_root, choose_system_configuration, read_mapped_tree_node_via_hint_then_central_mapping,
@@ -51,12 +56,9 @@
 const EXTENT_KEY_INODE_SEGMENT_OFFSET_IN_BYTES: usize = 8;
 /// extent 叶记录 key 第三段（offset 段，文件字节偏移，D8（核心索引结构） 已定项 3）的起点。
 const EXTENT_KEY_THIRD_SEGMENT_OFFSET_IN_BYTES: usize = 16;
-/// extent 树的根兼叶层级：第一版一棵树只有一个节点，记录直接装在根里（`transaction::build_file_version_units`）。
-const EXTENT_TREE_ROOT_LEVEL: u8 = 0;
 /// inode 树根的层级：根是内部节点，每条条目指一片码 3 叶容器（D8（核心索引结构） 已定项 6）。
 const INODE_TREE_ROOT_LEVEL: u8 = 1;
-/// extent 叶记录 key 宽 24，inode 树 key 宽 8（`recovery::key_width_for_kind` 同一份登记）。
-const EXTENT_KEY_WIDTH_IN_BYTES: usize = 24;
+/// inode 树 key 宽 8（`recovery::key_width_for_kind` 同一份登记）。
 const INODE_KEY_WIDTH_IN_BYTES: usize = 8;
 
 /// 读路径的运行时观测点：并行线二验收第 4 条要的两个数（这次读了几个单元、位置提示过期的多跳次数），
@@ -220,6 +222,9 @@
 pub enum OpenFileFailure {
     /// inode 树里没有这个号。
     NoSuchInode { inode: InodeNumber },
+    /// 按位置走 extent 树（上段到这个 inode 的叶条目、再到它的下段）时读不到、解不开或位置核不过
+    /// （`extent_tree::find_upper_leaf_entry` / `read_lower_segment` 的判定，与冷启动走读同一套口径）。
+    ExtentTreeWalk(RecoveryFailure),
     /// 这个 inode 的第 `unit_index_in_file` 条 extent 叶记录，key 的 offset 段不是那个单元第一个字节的文件偏移
     /// （`unit_index_in_file` × 净荷容量，D8（核心索引结构） 已定项 3）：记录错位、有洞，或 offset 段写成了单元序号。
     /// 报第一处对不上的那一条；不按位次猜着往下读。
@@ -283,17 +288,17 @@
     pub observation: ReadPathObservation,
 }
 
-/// 打开挂载态时沿 extent 树读了几个节点，与这棵树的高度、叶片数（里程碑「第二个事务」并行线一验收第 2 条的观测点）。
+/// 打开一个文件时沿 extent 树读了几个节点，与这一路的高度、叶片数（里程碑「第二个事务」并行线一验收第 2 条的观测点）。
 /// 打开之后按偏移读一个 extent 节点都不再读 ⇒ 顺序读 M 个单元时 extent 节点的读取次数就是 `node_reads`，
-/// 验收要它 ≤ `height + leaves`，不是每个单元从 inode 重走一遍。第一版 extent 树只有一个节点（根兼叶；长出内部节点那一档
-/// 写侧在落盘之前拒掉，`transaction::PublishError::ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided`）。
+/// 验收要它 ≤ `height + leaves`，不是每个单元从 inode 重走一遍。按位置寻址的两段（D8（核心索引结构） 已定项 14）：
+/// 上段从根走到这个 inode 那一片叶，一层读一个；下段整段读（内联时没有下段）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct ExtentTreeReadsAtOpen {
     /// 打开时向块层要过几个 extent 树节点（每读一个节点算一次，不按位置条目数）。
     pub node_reads: u64,
-    /// 树高：根的层级加一（层级 0 是叶）。
+    /// 这一路的高：上段的高加这个文件下段的高（内联时下段是 0）。
     pub height: u64,
-    /// 叶片数：读到的层级 0 的节点个数。
+    /// 叶片数：读到的层级 0 的节点个数（上段那一片叶加下段的叶）。
     pub leaves: u64,
 }
 
@@ -310,9 +315,8 @@
     root: RootRecord,
     filesystem_identifier_in_unit_headers: u64,
     extent_tree: TreeIdentifier,
-    extent_tree_reads_at_open: ExtentTreeReadsAtOpen,
-    /// extent 树根兼叶里的记录，**按盘上次序**（= key 升序，I-9.12（分隔 key 落在孩子区间之外） 那一族由 checker 判）。
-    extent_leaf_records: Vec<ExtentLeafRecordInMountState>,
+    /// extent 树根（上段的根）的指针：树表条目里那一条。打开池时不读 extent 树，打开文件时从它按位置走下去（用户 K4）。
+    extent_tree_root: NodePointer,
     /// inode 树全部叶容器里的记录，按叶序、叶内次序。
     inode_records: Vec<InodeRecord>,
     central_mapping_entries: Vec<CentralMappingEntryInMountState>,
@@ -415,36 +419,9 @@
             .map(|entry| entry.locations))
     };
     let mut tree_node_stale_location_hint_hops: usize = 0;
+    // extent 树打开池时不读（D8（核心索引结构） 已定项 14「挂载怎么读」：按需，打开文件时按位置走下去，用户 K4）：
+    // 这里只记下树表条目里它的号与根指针。
     let extent_entry = entry_of_kind(TREE_KIND_EXTENT)?;
-    let extent_root = read_mapped_tree_root(
-        reader,
-        extent_entry.tree,
-        EXTENT_KEY_WIDTH_IN_BYTES,
-        &extent_entry.root,
-        root,
-        filesystem_identifier_in_unit_headers,
-        &central_mapping_locations_of_key,
-        &mut tree_node_stale_location_hint_hops,
-    )?;
-    let extent_tree_node_reads = 1;
-    if extent_root.level != EXTENT_TREE_ROOT_LEVEL {
-        return Err(OpenPoolForReadFailure::TreeLevelUnexpected {
-            tree: extent_entry.tree,
-            expected_level: EXTENT_TREE_ROOT_LEVEL,
-            found_level: extent_root.level,
-        });
-    }
-    let mut extent_leaf_records = Vec::with_capacity(extent_root.entries.len());
-    for record_bytes in &extent_root.entries {
-        let (key, data_unit_pointer) =
-            parse_extent_record(record_bytes).ok_or(OpenPoolForReadFailure::RecordMalformed {
-                what: "extent 叶记录",
-            })?;
-        extent_leaf_records.push(ExtentLeafRecordInMountState {
-            key,
-            data_unit_pointer,
-        });
-    }
 
     let inode_entry = entry_of_kind(TREE_KIND_INODE)?;
     let inode_root = read_mapped_tree_root(
@@ -525,13 +502,7 @@
         root: *root,
         filesystem_identifier_in_unit_headers,
         extent_tree: extent_entry.tree,
-        extent_tree_reads_at_open: ExtentTreeReadsAtOpen {
-            node_reads: extent_tree_node_reads,
-            height: u64::from(extent_root.level) + 1,
-            // 根的层级核过是 0（根兼叶）：读到的叶就是根这一片。
-            leaves: 1,
-        },
-        extent_leaf_records,
+        extent_tree_root: extent_entry.root,
         inode_records,
         central_mapping_entries,
         central_mapping_tree_reads_at_open: CentralMappingTreeReadsAtOpen {
@@ -625,44 +596,105 @@
         self.central_mapping_tree_reads_at_open
     }
 
-    /// 打开时沿 extent 树读了几个节点、树高、叶片数：之后按偏移读不再碰 extent 树，顺序读 M 个单元的 extent 节点读取次数就是它。
-    #[must_use]
-    pub const fn extent_tree_reads_at_open(&self) -> ExtentTreeReadsAtOpen {
-        self.extent_tree_reads_at_open
-    }
-
     /// 挂载态里这一版的 inode 记录（按叶序、叶内次序）。
     #[must_use]
     pub fn inode_records(&self) -> &[InodeRecord] {
         &self.inode_records
     }
 
-    /// 挂载态里这一版的 extent 叶记录（按盘上次序）。
-    #[must_use]
-    pub fn extent_leaf_records(&self) -> &[ExtentLeafRecordInMountState] {
-        &self.extent_leaf_records
-    }
-
-    /// 打开一个文件：从挂载态里挑出它的 inode 记录与它的数据单元记录，当场核位次定位的前提。
+    /// 打开一个文件：从挂载态里挑出它的 inode 记录，再按位置走 extent 树拿它的数据单元记录（D8（核心索引结构） 已定项 14「挂载怎么读」：
+    /// extent 树按需读，用户 K4）——上段从根按 inode 号的位置走到它那一片叶、取它那一条叶条目（`extent_tree::find_upper_leaf_entry`），
+    /// 标签 1 就把它的下段整段读下来（`extent_tree::read_lower_segment`），标签 2 就是内联的那一个数据指针，标签 0 与缺席一个单元都没有。
+    /// 每个节点按位置核（与冷启动走读同一套），提示读不出经中央映射回退（查的是打开池时读进来的那份映射条目，不再发设备读）。
+    /// 读完当场核位次定位的前提。
     ///
     /// # Errors
-    /// 见 [`OpenFileFailure`]：没有这个 inode / 记录不是按 key 升序 / 记录条数与文件大小算出的单元数对不上。
-    pub fn open_file(&self, inode: InodeNumber) -> Result<OpenFileForRead<'_>, OpenFileFailure> {
+    /// 见 [`OpenFileFailure`]：没有这个 inode / 走 extent 树读不到或核不过 / 记录条数与文件大小算出的单元数对不上。
+    pub fn open_file(
+        &self,
+        reader: &dyn PoolReader,
+        inode: InodeNumber,
+    ) -> Result<OpenFileForRead<'_>, OpenFileFailure> {
         let inode_record = self
             .inode_records
             .iter()
             .find(|record| record.inode == inode.0)
             .copied()
             .ok_or(OpenFileFailure::NoSuchInode { inode })?;
-        let data_unit_records: Vec<ExtentLeafRecordInMountState> = self
-            .extent_leaf_records
-            .iter()
-            .filter(|record| record.inode_number() == inode)
-            .copied()
+        let reading = ExtentTreeReading {
+            tree: self.extent_tree,
+            judgement: ExtentTreeHeaderJudgement::EveryHeaderAgainstItsReference,
+            root: &self.root,
+            expected_filesystem_identifier: self.filesystem_identifier_in_unit_headers,
+        };
+        let central_mapping_locations_of_key =
+            |mapping_key: &[u8]| Ok(self.central_mapping_lookup(mapping_key));
+        let mut tree_node_stale_location_hint_hops: usize = 0;
+        let mut node_reads: u64 = 0;
+        let mut read_node = |pointer: &NodePointer| {
+            node_reads += 1;
+            read_mapped_tree_node_via_hint_then_central_mapping(
+                reader,
+                pointer,
+                MappedTreeNodeClass::IndexNode,
+                &central_mapping_locations_of_key,
+                &mut tree_node_stale_location_hint_hops,
+            )
+        };
+        let (entry, upper_nodes_read) =
+            find_upper_leaf_entry(&reading, &self.extent_tree_root, inode.0, &mut read_node)
+                .map_err(OpenFileFailure::ExtentTreeWalk)?;
+        // 上段每一层读一个节点（叶那一片在内），所以上段的高就是这一路读的上段节点数。
+        let upper_height = upper_nodes_read;
+        let (data_pointers, lower_height, lower_leaves): (Vec<(u64, DataPointer)>, u64, u64) =
+            match entry.map(|found| found.target) {
+                None | Some(ExtentUpperLeafTarget::NoDataUnit) => (Vec::new(), 0, 0),
+                Some(ExtentUpperLeafTarget::InlineDataUnit(pointer)) => (vec![(0, pointer)], 0, 0),
+                Some(ExtentUpperLeafTarget::LowerSegmentRoot(lower_root)) => {
+                    let segment = read_lower_segment(
+                        &reading,
+                        inode.0,
+                        &lower_root,
+                        &mut read_node,
+                        &mut std::collections::BTreeSet::new(),
+                    )
+                    .map_err(OpenFileFailure::ExtentTreeWalk)?;
+                    let lower_height = segment
+                        .nodes
+                        .last()
+                        .map_or(0, |(position, _, _)| u64::from(position.level) + 1);
+                    let lower_leaves = u64::try_from(
+                        segment
+                            .nodes
+                            .iter()
+                            .filter(|(position, _, _)| position.level == 0)
+                            .count(),
+                    )
+                    .expect("叶片数");
+                    (segment.data_pointers, lower_height, lower_leaves)
+                }
+            };
+        let extent_tree_reads_at_open = ExtentTreeReadsAtOpen {
+            node_reads,
+            height: upper_height + lower_height,
+            leaves: 1 + lower_leaves,
+        };
+        let payload_capacity = payload_capacity_in_bytes();
+        let data_unit_records: Vec<ExtentLeafRecordInMountState> = data_pointers
+            .into_iter()
+            .map(|(unit, data_unit_pointer)| {
+                let key: [u8; 24] =
+                    extent_key_bytes(inode.0, unit.saturating_mul(payload_capacity))
+                        .try_into()
+                        .expect("extent key 24 字节");
+                ExtentLeafRecordInMountState {
+                    key,
+                    data_unit_pointer,
+                }
+            })
             .collect();
         // 第 i 条记录的 offset 段要等于第 i 个单元第一个字节的文件偏移（D8（核心索引结构） 已定项 3）：
         // 于是「第 i 条记录 = 文件第 i 个数据单元」由 key 本身担保，读的时候按单元序号取记录。
-        let payload_capacity = payload_capacity_in_bytes();
         for (position, record) in data_unit_records.iter().enumerate() {
             let unit_index_in_file =
                 DataUnitIndexInFile(u64::try_from(position).expect("单元序号"));
@@ -695,6 +727,11 @@
             mount: self,
             inode_record,
             data_unit_records,
+            extent_tree_reads_at_open,
+            tree_node_stale_location_hint_hops_at_open: u64::try_from(
+                tree_node_stale_location_hint_hops,
+            )
+            .expect("多跳次数不超过这一趟读的节点数"),
         })
     }
 
@@ -718,6 +755,8 @@
     inode_record: InodeRecord,
     /// 第 i 项就是文件第 i 个数据单元（位次定位，前提在 [`MountedPoolForRead::open_file`] 里核过）。
     data_unit_records: Vec<ExtentLeafRecordInMountState>,
+    extent_tree_reads_at_open: ExtentTreeReadsAtOpen,
+    tree_node_stale_location_hint_hops_at_open: u64,
 }
 
 impl OpenFileForRead<'_> {
@@ -727,6 +766,25 @@
         self.inode_record
     }
 
+    /// 打开这个文件时沿 extent 树读了几个节点、这一路的高、叶片数：之后按偏移读不再碰 extent 树，
+    /// 顺序读 M 个单元的 extent 节点读取次数就是它。
+    #[must_use]
+    pub const fn extent_tree_reads_at_open(&self) -> ExtentTreeReadsAtOpen {
+        self.extent_tree_reads_at_open
+    }
+
+    /// 打开这个文件那一趟里 extent 树节点的位置提示读不出、经中央映射回退的次数（D19（块指针的结构与宽度预算） 已定项 5 硬规则 3）。
+    #[must_use]
+    pub const fn tree_node_stale_location_hint_hops_at_open(&self) -> u64 {
+        self.tree_node_stale_location_hint_hops_at_open
+    }
+
+    /// 这个文件按位次排好的数据单元记录（第 i 项是文件第 i 个数据单元）。
+    #[must_use]
+    pub fn data_unit_records(&self) -> &[ExtentLeafRecordInMountState] {
+        &self.data_unit_records
+    }
+
     /// 这个文件有几个数据单元。
     #[must_use]
     pub fn data_unit_count(&self) -> u64 {
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/mount.rs tree/crates/singlefs-core/src/mount.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/mount.rs	2026-09-24 18:48:58.241518146 +0000
+++ tree/crates/singlefs-core/src/mount.rs	2026-09-24 21:35:39.449983013 +0000
@@ -4,6 +4,9 @@
 //! 第一版没有干净关闭标记，重开一律走恢复。
 
 use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
+use crate::allocation_record_tree::{
+    node_pointers_as_far_as_readable, AllocationRecordTreeGeometry,
+};
 use crate::allocator::{
     AllocationRecord, DeviceFreeMap, Placement, PlacementOnDevice, PlacementRefusal, PoolAllocator,
     ReclaimedReuse, RootRingOccupancy, RootRingOccupant,
@@ -12,6 +15,7 @@
 use crate::journal::back_chain_of;
 use crate::make_filesystem::MakeFilesystemParameters;
 use crate::pointer::{slot_shared_by_both_location_entries, LocationEntriesOnDifferentSlots};
+use crate::recovery::read_unit_via_locations;
 use crate::recovery::{
     allocation_records_of_version_without_file, allocation_records_under_root, choose_root,
     choose_system_configuration, effective_rollback_floor, every_root_ring_slot_holds_a_root,
@@ -28,17 +32,13 @@
 use crate::root_record::RootRecord;
 use crate::root_ring::{target_for_publish, RootRingSlot};
 use crate::transaction::{
-    acquire_expected_instance, instance_generation_to_acquire, instance_table_chain_to_release,
-    instance_table_page_roles_in_bump_order,
-    multi_level_tree_nodes_of_a_publish_sequence_without_content, multi_level_tree_of_role,
-    placements_to_release_via_mapping, publish_instance_table_on_version_without_file,
-    publish_sequence_admission, publish_version, publish_without_units,
-    refuse_mapping_entries_that_do_not_name_two_pool_devices,
-    release_check_and_admission_of_a_row_publish_on_a_version_without_file,
-    rewritten_roles_of_a_publish_without_content, AcquisitionFailed, CodeTwoTreeNodeCapacities,
-    ExpectedInstanceAcquisitionFailed, InstanceTableOnlyPublishPlan, InstanceTablePlan,
-    InstanceTableRewrite, PlacementRule, PoolVersion, PoolWriter, PublishError, PublishPlan,
-    PublishShape, TransactionOutput, TransactionUnit, TreeNodeWrittenBy, ZeroUnitPublishPlan,
+    acquire_expected_instance, instance_generation_to_acquire, prepare_the_publish_without_units,
+    prepare_the_row_publish_on_a_version_without_file, prepare_the_version_publish,
+    publish_instance_table_on_version_without_file, publish_version, publish_without_units,
+    role_of_allocation_record_tree_node, AcquisitionFailed, ExpectedInstanceAcquisitionFailed,
+    InstanceTableOnlyPublishPlan, InstanceTablePlan, InstanceTableRewrite, PoolVersion, PoolWriter,
+    PublishError, PublishPlan, ReleaseChecksumCheck, TransactionOutput, TransactionUnit,
+    VersionWithoutFilePublishOutput, ZeroUnitPublishPlan,
 };
 use crate::write_accounting::WritesByStructureKind;
 use singlefs_format::ROOT_RING_REGIONS;
@@ -103,18 +103,20 @@
         expected: InstanceGeneration,
         recomputed: InstanceGeneration,
     },
-    /// 写行那次发布的准入（这次之后的分配记录条数、这次要写的记账行数；这次要换下的那条实例表旧链逐片核不过）算不过：
-    /// 在**取号之前**拒绝，盘上一个字节都不动、
-    /// 两块盘系统配置里的实例代号不动（增补 2 第 20a 行，代码三方第一轮打中）。改之前这两条只在发布路径里算，
+    /// 写行那次发布在取号之前的预演（发布路径落盘之前那一段：释放核验、这次要换下的那条实例表旧链逐片核、两棵多层码 2 树的形状、
+    /// 分配记录树重写集合的固定点）报错：在**取号之前**拒绝，盘上一个字节都不动、
+    /// 两块盘系统配置里的实例代号不动（增补 2 第 20a 行，代码三方第一轮打中）。改之前这些只在发布路径里算，
     /// 取号（两次系统配置槽写 + 一道屏障）已经写完才报出来，而取号的回卷只管取号自己那几次写报错、管不到之后的发布失败 ⇒
-    /// 分配记录树满了的池此后每试一次可写挂载就再烧一个实例代号。
+    /// 那样的池此后每试一次可写挂载就再烧一个实例代号。
     RowPublishAdmissionRefusedBeforeAcquisition {
         instance_to_acquire: InstanceGeneration,
         cause: PublishError,
     },
-    /// 暖机那几次空发布（写行之后推到本实例的根覆盖每块盘，D16（发布语义） 已定项 8 戊）里第几次的准入算不过：连写行那次一起
-    /// 在取号之前算，算不过在任何写之前返回（增补 2 第 20a 行，2026-09-18 用户定案）。改之前取号之前只算写行那一次，
-    /// 「写行装得下、暖机第 N 次装不下」的池是取号写完、写行也发完，暖机才报错——实例代号照样烧掉。
+    /// 暖机那几次空发布（写行之后推到本实例的根覆盖每块盘，D16（发布语义） 已定项 8 戊）里第几次在取号之前的预演报错：连写行那次一起
+    /// 在取号之前预演，报错在任何写之前返回（增补 2 第 20a 行，2026-09-18 用户定案）。分配记录树按位置寻址之后
+    /// （D8（核心索引结构） 已定项 14）没有「一个节点装不下」那道墙，暖机这几次还报得出的只剩两棵多层码 2 树长过 256 层、
+    /// 分配记录树重写集合迭代不收敛（只在强制复用窗口为 0 的只供测试的开关下可能）这类，落点取不到另报
+    /// `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`。
     WarmUpAdmissionRefusedBeforeAcquisition {
         instance_to_acquire: InstanceGeneration,
         /// 第几次暖机空发布算不过，从 1 数。
@@ -125,7 +127,7 @@
     },
     /// 取号之前在分配器的一份拷贝上把这次挂载取号之后要发的那一串（写行一次、暖机 `warm_up_publishes_planned` 次）逐次取落点，
     /// 第 `publish_index` 次（从 0 数，0 是写行那次）的 `unit` 取不到，`refusal` 是分配器给的原因
-    /// （`placements_of_the_publishes_after_acquisition_on_a_copy`）。**取号之前的准入怎么把这次挂载要写的落点算进去，条款没定**：
+    /// （`dry_run_of_the_publishes_after_acquisition`）。**取号之前的准入怎么把这次挂载要写的落点算进去，条款没定**：
     /// 可写挂载的准入要「实例切换的预留拿得到」（D2（RAID 条带策略） 已定项 13；D23（journal 的角色与格式） 已定项 14「切换要用的块在挂载准入时预留」），
     /// 预留按 D28（挂载期承诺量） 已定项 3 算，其中暖机那一半的 c_max 与已定项 4 的 checkpoint 保留池从哪读没有条款
     /// （C363（现算保留池时树高从哪读没有条款）），D28（挂载期承诺量） 已定项 1 那条式子另一边的「需求」怎么摊到每块盘也没有
@@ -516,20 +518,32 @@
             // 读不全的链照样把认得出的那几片算上——少算一片，被抛弃根的那一片就不隔离、回退之后可能被发出去。
             let instance_table_pages =
                 instance_table_page_pointers_as_far_as_readable(devices, root);
-            // 写过行的一版那片分配记录树节点同样是这条根引用的单元（D23（journal 的角色与格式） 已定项 14：被抛弃时间线的根离开根环之前，
-            // 它们引用的单元不许重新分配）；漏了它，被抛弃根的这一片既不隔离、也不在回退目标那一版的账里，回退之后是空闲槽。
-            // mkfs 的第 0 代与照抄它的暖机根那一项全零，没有这一片。
-            let allocation_record_node = (root.allocation_record_tree_root
-                != crate::pointer::NodePointer::empty_root())
-            .then_some((
-                &root.allocation_record_tree_root,
-                TransactionUnit::AllocationTree,
-            ));
+            // 写过行的一版那棵分配记录树的节点同样是这条根引用的单元（D23（journal 的角色与格式） 已定项 14：被抛弃时间线的根离开根环之前，
+            // 它们引用的单元不许重新分配）；漏了它们，被抛弃根的这几片既不隔离、也不在回退目标那一版的账里，回退之后是空闲槽。
+            // 树可以多层（D8（核心索引结构） 已定项 14）：读得出多少认多少，读不出的节点它自己那一片照样认（父条目里有它的指针）。
+            // mkfs 的第 0 代与照抄它的暖机根那一项全零，没有这棵树。
+            let node_bytes = usize::try_from(singlefs_format::NODE_BYTES).expect("16384");
+            let allocation_record_tree_nodes: Vec<crate::pointer::NodePointer> =
+                if root.allocation_record_tree_root == crate::pointer::NodePointer::empty_root() {
+                    Vec::new()
+                } else {
+                    node_pointers_as_far_as_readable(
+                        &root.allocation_record_tree_root,
+                        &AllocationRecordTreeGeometry::of_reader(devices),
+                        &mut |pointer: &crate::pointer::NodePointer| {
+                            read_unit_via_locations(devices, &pointer.locations, node_bytes).ok()
+                        },
+                    )
+                };
             for (pointer, unit) in instance_table_pages
                 .iter()
                 .map(|page_pointer| (page_pointer, TransactionUnit::InstanceTable))
                 .chain([(&root.tree_table, TransactionUnit::TreeTable)])
-                .chain(allocation_record_node)
+                .chain(
+                    allocation_record_tree_nodes
+                        .iter()
+                        .map(|pointer| (pointer, TransactionUnit::AllocationTree)),
+                )
             {
                 let slot = slot_shared_by_both_location_entries(&pointer.locations).ok()?;
                 for (identity, _) in devices {
@@ -675,14 +689,16 @@
 
 /// 树表 0 条的那一版的账从哪来，只有两条路：
 ///
-/// - 根记录那一项（分配记录树根指针）不是全零 ⇒ 这一版写过行，写行那次发布把这一版的全部分配记录写成了一个节点
-///   （C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案）：从它重建，被换下的那一片实例表因此带着已释放标志回来，
-///   不会被当空闲槽发出去。那一片分配记录树节点自己的落点记进分配器——下一次发布要换下它，而这一版没有上一版的内存态可查。
+/// - 根记录那一项（分配记录树根指针）不是全零 ⇒ 这一版写过行，写行那次发布把这一版的全部分配记录写进了那棵分配记录树
+///   （C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案；按绝对槽号按位置寻址，D8（核心索引结构） 已定项 14）：
+///   整棵读回来重建（挂载时分配记录树整棵读，用户 K4），被换下的实例表因此带着已释放标志回来，不会被当空闲槽发出去。
+///   那棵树的节点与记录记进分配器——下一次发布要照抄或换下它们，而这一版没有上一版的内存态可查。
 /// - 全零 ⇒ mkfs 的第 0 代（或照抄它的暖机根）：账由实例表与树表两条指针直接算（`format_time_allocator`）。
 ///
 /// # Errors
-/// 分配记录树读不出、解不开、不止一层、条目宽或结构值判红 ⇒ `Recovery(...)`；
-/// 全零那一条上两条指针任一条不是 mkfs 写的那一版 ⇒ `VersionWithoutFileNotWrittenByMakeFilesystem`（见 `format_time_allocator`）。
+/// 分配记录树读不出、解不开、位置对不上、条目宽或结构值判红 ⇒ `Recovery(...)`；某个节点两条位置条目不同槽 ⇒
+/// `FormatTimeUnitLocationsOnDifferentSlots`；全零那一条上两条指针任一条不是 mkfs 写的那一版 ⇒
+/// `VersionWithoutFileNotWrittenByMakeFilesystem`（见 `format_time_allocator`）。
 #[allow(
     clippy::ptr_arg,
     reason = "allocation_records_of_version_without_file 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
@@ -692,23 +708,22 @@
     device_maps: Vec<DeviceFreeMap>,
     root: &RootRecord,
 ) -> Result<PoolAllocator, MountError> {
-    let Some(records) =
+    let Some(tree) =
         allocation_records_of_version_without_file(devices, root).map_err(MountError::Recovery)?
     else {
         return format_time_allocator(device_maps, root);
     };
-    let node_placement = Placement {
-        slot: slot_shared_by_both_location_entries(&root.allocation_record_tree_root.locations)
-            .map_err(
-                |disagreement| MountError::FormatTimeUnitLocationsOnDifferentSlots {
-                    unit: TransactionUnit::AllocationTree,
-                    disagreement,
-                },
-            )?,
-        span: TransactionUnit::AllocationTree.span_slots(),
-    };
-    let mut allocator = PoolAllocator::rebuild_from_records(device_maps, records);
-    allocator.note_allocation_record_node_of_the_version_without_file(node_placement);
+    // 每个节点两条位置条目同槽（第一版两盘同槽）：下一次发布照抄或换下它们时按一个落点释放。
+    for (node, pointer) in &tree.version.nodes {
+        slot_shared_by_both_location_entries(&pointer.locations).map_err(|disagreement| {
+            MountError::FormatTimeUnitLocationsOnDifferentSlots {
+                unit: role_of_allocation_record_tree_node(*node),
+                disagreement,
+            }
+        })?;
+    }
+    let mut allocator = PoolAllocator::rebuild_from_records(device_maps, tree.records.clone());
+    allocator.note_allocation_record_tree_of_the_version_without_file(tree);
     Ok(allocator)
 }
 
@@ -1061,8 +1076,33 @@
     tree_identifier_watermark: u64,
 }
 
-/// 写行那次发布、上一版带文件：在上一版那张实例表后面接上这次写的行，重写整条实例表链与四个固定点单元（D18（块里携带什么信息） 已定项 11；
+/// 写行那次发布、上一版带文件的计划：在上一版那张实例表后面接上这次写的行，重写整条实例表链与固定点单元（D18（块里携带什么信息） 已定项 11；
 /// 记账树已经存在 ⇒ 空发布也重写固定点单元，D16（发布语义） 已定项 9）。事务号 0、本实例第一条反向链 0。
+/// 取号之后的发布与取号之前的预演读的是同一张计划（`publish_rows_on_file_version`、`dry_run_of_the_publishes_after_acquisition`）。
+fn row_publish_plan_on_file_version<'plan>(
+    previous: &TransactionOutput,
+    instance_table: InstanceTableRewrite,
+    identity: &RowPublishIdentity,
+) -> PublishPlan<'plan> {
+    PublishPlan {
+        txg: identity.txg,
+        counter: identity.counter,
+        transaction: 0,
+        // 新实例的第一条记录（反向链恒 0），事务号按实例各算各的，从这里重新从 1 起。
+        highest_transaction_number_before_this_publish: 0,
+        instance: identity.instance,
+        back_chain: 0,
+        file: None,
+        // 写行那次发布不碰 inode 树。
+        new_inode_records: &[],
+        instance_table: InstanceTablePlan::Rewrite(instance_table),
+        tree_birth_txg: previous.tree_birth_txg(),
+        tree_identifier_watermark: identity.tree_identifier_watermark,
+        rollback_floor: identity.rollback_floor,
+    }
+}
+
+/// 写行那次发布、上一版带文件：按 [`row_publish_plan_on_file_version`] 发。
 fn publish_rows_on_file_version<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
@@ -1073,27 +1113,53 @@
     publish_version(
         pool,
         allocator,
-        PublishPlan {
-            txg: identity.txg,
-            counter: identity.counter,
-            transaction: 0,
-            // 新实例的第一条记录（反向链恒 0），事务号按实例各算各的，从这里重新从 1 起。
-            highest_transaction_number_before_this_publish: 0,
-            instance: identity.instance,
-            back_chain: 0,
-            file: None,
-            // 写行那次发布不碰 inode 树。
-            new_inode_records: &[],
-            instance_table: InstanceTablePlan::Rewrite(instance_table),
-            tree_birth_txg: previous.tree_birth_txg(),
-            tree_identifier_watermark: identity.tree_identifier_watermark,
-            rollback_floor: identity.rollback_floor,
-        },
+        row_publish_plan_on_file_version(previous, instance_table, &identity),
         Some(previous),
     )
 }
 
-/// 暖机的一次空发布，接在现行那一版后面：带文件的一版重写四个固定点单元（记账树存在，D16（发布语义） 已定项 9），
+/// 暖机的一次空发布接在带文件的现行那一版后面时的计划：重写固定点单元（记账树存在，D16（发布语义） 已定项 9），不写文件内容、不碰 inode 树、
+/// 实例表照抄。取号之后的发布与取号之前的预演读的是同一张计划。
+fn empty_publish_plan_after_file_version<'plan>(
+    current_file_version: &TransactionOutput,
+    instance: InstanceGeneration,
+) -> PublishPlan<'plan> {
+    PublishPlan {
+        txg: CheckpointTxg(current_file_version.root.checkpoint_txg.0 + 1),
+        counter: current_file_version.record.counter + 1,
+        transaction: 0,
+        highest_transaction_number_before_this_publish: current_file_version
+            .highest_transaction_number_in_this_instance,
+        instance,
+        back_chain: back_chain_of(&current_file_version.record_bytes),
+        file: None,
+        // 暖机那几次空发布不碰 inode 树。
+        new_inode_records: &[],
+        instance_table: InstanceTablePlan::Carry(current_file_version.root.instance_table),
+        tree_birth_txg: current_file_version.tree_birth_txg(),
+        // 暖机接在本实例写行那次发布之后：那一版的水位已经取过根环里的 max，这里照抄。
+        tree_identifier_watermark: current_file_version.root.tree_identifier_watermark,
+        rollback_floor: current_file_version.root.rollback_floor,
+    }
+}
+
+/// 暖机的一次空发布接在树表 0 条的现行那一版后面时的计划（零单元，D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）。
+fn empty_publish_plan_after_version_without_file(
+    current_version_without_file: &VersionWithoutFilePublishOutput,
+    instance: InstanceGeneration,
+) -> ZeroUnitPublishPlan {
+    ZeroUnitPublishPlan {
+        txg: CheckpointTxg(current_version_without_file.root.checkpoint_txg.0 + 1),
+        counter: current_version_without_file.record.counter + 1,
+        instance,
+        back_chain: back_chain_of(&current_version_without_file.record_bytes),
+        rollback_floor: current_version_without_file.root.rollback_floor,
+        // 暖机接在本实例写行那次发布之后：那一版的水位已经取过根环里的 max，这里照抄。
+        tree_identifier_watermark: current_version_without_file.root.tree_identifier_watermark,
+    }
+}
+
+/// 暖机的一次空发布，接在现行那一版后面：带文件的一版重写固定点单元（记账树存在，D16（发布语义） 已定项 9），
 /// 树表 0 条的一版写零个单元（同一条已定项「树表 0 条 ⇒ 零单元」）。
 fn publish_empty_after<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
@@ -1102,67 +1168,21 @@
     instance: InstanceGeneration,
 ) -> Result<PoolVersion, PublishError> {
     match current {
-        PoolVersion::WithFile(current_file_version) => {
-            let plan = PublishPlan {
-                txg: CheckpointTxg(current_file_version.root.checkpoint_txg.0 + 1),
-                counter: current_file_version.record.counter + 1,
-                transaction: 0,
-                highest_transaction_number_before_this_publish: current_file_version
-                    .highest_transaction_number_in_this_instance,
-                instance,
-                back_chain: back_chain_of(&current_file_version.record_bytes),
-                file: None,
-                // 暖机那几次空发布不碰 inode 树。
-                new_inode_records: &[],
-                instance_table: InstanceTablePlan::Carry(current_file_version.root.instance_table),
-                tree_birth_txg: current_file_version.tree_birth_txg(),
-                // 暖机接在本实例写行那次发布之后：那一版的水位已经取过根环里的 max，这里照抄。
-                tree_identifier_watermark: current_file_version.root.tree_identifier_watermark,
-                rollback_floor: current_file_version.root.rollback_floor,
-            };
-            // 取号之前那道准入按 `PublishShape::EMPTY_PUBLISH` 算这一次要加几条分配记录；这里钉住它算的就是这张计划的形状。
-            // 形状要先把计划算成「这次之后 inode 树是什么样、重写哪些角色」才知道（`PublishPlan::resolve`），
-            // 而算它只读、不发写：算不过就在这里交回，盘上逐字节不变。
-            let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator
-                .devices
-                .iter()
-                .map(|device_map| device_map.device)
-                .collect();
-            let resolved_empty_publish = plan.resolve(
-                Some(current_file_version),
-                &current_file_version.tree_identifiers,
-                &device_identities_of_the_accounting_rows,
-                pool.code_two_tree_node_capacities(),
-            )?;
-            assert_eq!(
-                resolved_empty_publish.shape(),
-                PublishShape::EMPTY_PUBLISH,
-                "暖机这次空发布的形状与取号之前算准入用的那一个相同（不写文件内容、不碰 inode 树、实例表照抄）"
-            );
-            // 映射条目那一条准入在取号之前按上一版的叶容器数算过：这次不碰 inode 树 ⇒ 算出来的树与上一版逐片相同。
-            assert_eq!(
-                resolved_empty_publish.inode_tree.containers.len(),
-                current_file_version.inode_leaf_containers.len(),
-                "暖机这次空发布之后的叶容器数与取号之前算映射条目准入用的那一个相同"
-            );
-            publish_version(pool, allocator, plan, Some(current_file_version))
-                .map(PoolVersion::WithFile)
-        }
+        PoolVersion::WithFile(current_file_version) => publish_version(
+            pool,
+            allocator,
+            empty_publish_plan_after_file_version(current_file_version, instance),
+            Some(current_file_version),
+        )
+        .map(PoolVersion::WithFile),
         PoolVersion::WithoutFile(current_version_without_file) => {
             let published = publish_without_units(
                 pool,
                 &current_version_without_file.root,
-                ZeroUnitPublishPlan {
-                    txg: CheckpointTxg(current_version_without_file.root.checkpoint_txg.0 + 1),
-                    counter: current_version_without_file.record.counter + 1,
+                empty_publish_plan_after_version_without_file(
+                    current_version_without_file,
                     instance,
-                    back_chain: back_chain_of(&current_version_without_file.record_bytes),
-                    rollback_floor: current_version_without_file.root.rollback_floor,
-                    // 暖机接在本实例写行那次发布之后：那一版的水位已经取过根环里的 max，这里照抄。
-                    tree_identifier_watermark: current_version_without_file
-                        .root
-                        .tree_identifier_watermark,
-                },
+                ),
             )
             .map_err(PublishError::from)?;
             // 零单元发布不经分配器：根落盘之后在这里记它盖掉的根环槽（`PoolAllocator::record_root_written_by_this_process`）。
@@ -1172,373 +1192,151 @@
     }
 }
 
-/// 这次挂载在取号之后要发的那几次——写行一次、暖机 `warm_up_publishes_planned` 次——的准入，在取号之前一串算完
-/// （增补 2 第 20a 行：写行那一半是代码三方第一轮打中的，暖机那一半是 2026-09-18 用户定案）：分配记录树、记账树与中央映射树
-/// 在每一次之后都要装得下，前面几次要新增的分配记录算进后面几次的基数；算不过就在任何写之前返回——取号写出去的实例代号一去不回，
-/// 回卷只管取号自己那几次写报错，管不到取号之后的发布失败。
-/// 读的是内存里这个分配器，与发布路径那一遍同一份输入（取号不碰它），发布路径那一遍仍在、是动分配器之前的最后一道。
-/// 实例表按写行那次发布要写的那条链算（`instance_table_rewrite`：这一版的行接上这次写的行、被换下的是这一版的整条链；
-/// 写行那次发布拼的正是它，号在取号写之前重算、不等就不写（`acquire_expected_instance`），所以这里判的与写出去的是同一张表）：
-/// 这次之后几片，写行那次发布就重写几个实例表角色（行一片写满 369 行再开下一片，D18（块里携带什么信息） 已定项 11）；
-/// 被换下的那条旧链逐片按释放判定路径核三样（`instance_table_chain_to_release`，只查不改），核不过同样在取号之前拒——
-/// 旧链上有一片不在这一版的分配记录里（只有别的实现写的或坏镜像上有），取号之后才在发布路径里报出来，号就烧了。
-/// 可写挂载与回退各按自己的 `start` 算：回退的那一版是 R_old 指着的表、要写的是 [max(r_old, 1), 新实例)。
-/// **旧链那一项两臂都判**：树表 0 条的一版上写行同样重写整条链（`publish_instance_table_on_version_without_file`）；
-/// 那一版上要写的行为空时写行那次是零单元发布、不碰实例表，不判。
-/// 树表 0 条的一版上写行那次还重写那一版自己的分配记录树节点（C512（树表 0 条的一版上被换下的单元记在哪） 定案之后）：
-/// 上一版那片分配记录树节点的释放核与这次之后的分配记录条数准入同样在取号之前判
-/// （`transaction::release_check_and_admission_of_a_row_publish_on_a_version_without_file`，与发布路径同一个函数；
-/// D18（块里携带什么信息） 已定项 11「可写挂载的顺序」第五个合取，2026-09-24 用户定；此前只在取号之后的发布路径里判，
-/// 实例表长到 40 片起每试一次可写挂载就烧一个实例代号——`research/prompts/m2-final-code-r1-opus-output.md` Z4-1）。
-/// 暖机在那一版上是零单元发布，不写分配记录树。记账树与中央映射树两条只在带文件的一版上判：树表 0 条 ⇒ 这一版没有这两棵树。
-fn refuse_publishes_before_acquisition_that_do_not_pass_admission(
-    allocator: &PoolAllocator,
-    start: &InstanceStart,
-    instance_to_acquire: InstanceGeneration,
-    instance_table_rewrite: &InstanceTableRewrite,
-    rows_to_write: usize,
-    warm_up_publishes_planned: usize,
-    code_two_tree_node_capacities: CodeTwoTreeNodeCapacities,
-) -> Result<(), MountError> {
-    let row_publish_rewrites_the_instance_table = match &start.previous {
-        PreviousVersion::WithFile { .. } => true,
-        PreviousVersion::WithoutFile { .. } => rows_to_write > 0,
-    };
-    if row_publish_rewrites_the_instance_table {
-        instance_table_chain_to_release(&instance_table_rewrite.replaced_chain, allocator)
-            .map_err(
-                |cause| MountError::RowPublishAdmissionRefusedBeforeAcquisition {
-                    instance_to_acquire,
-                    cause,
-                },
-            )?;
-    }
-    if let (PreviousVersion::WithoutFile { root, .. }, true) =
-        (&start.previous, row_publish_rewrites_the_instance_table)
-    {
-        release_check_and_admission_of_a_row_publish_on_a_version_without_file(
-            root,
-            instance_table_rewrite,
-            allocator,
-        )
-        .map_err(
-            |cause| MountError::RowPublishAdmissionRefusedBeforeAcquisition {
-                instance_to_acquire,
-                cause,
-            },
-        )?;
-    }
-    if let PreviousVersion::WithFile { output, .. } = &start.previous {
-        let shapes: Vec<PublishShape> =
-            std::iter::once(PublishShape::row_publish_rewriting_instance_table_pages(
-                instance_table_rewrite.pages_after_this_publish(),
-            ))
-            .chain(std::iter::repeat_n(
-                PublishShape::EMPTY_PUBLISH,
-                warm_up_publishes_planned,
-            ))
-            .collect();
-        // 写行与暖机都不碰 inode 树、不写文件内容 ⇒ 这一串里每次只换分配记录树与记账树节点的映射 key，两棵多层码 2 树
-        // 每次重写几个节点从上一版的形状推（`publish_sequence_admission`），容量与取号之后那几次发布的写入口上装的相同。
-        publish_sequence_admission(allocator, &shapes, output, code_two_tree_node_capacities)
-            .map_err(|refusal| match refusal.publish_index {
-                0 => MountError::RowPublishAdmissionRefusedBeforeAcquisition {
-                    instance_to_acquire,
-                    cause: refusal.cause,
-                },
-                warm_up_publish_index => MountError::WarmUpAdmissionRefusedBeforeAcquisition {
-                    instance_to_acquire,
-                    warm_up_publish_index,
-                    warm_up_publishes_planned,
-                    cause: refusal.cause,
-                },
-            })?;
-    }
-    Ok(())
-}
-
 /// 取号之后那一串里一次发布取到的落点：这次重写的角色，按取落点的次序（bump 次序）各一个槽。零单元的发布一个都不取。
 type PlacementsTakenByOnePublish = Vec<(TransactionUnit, SlotNumber)>;
 
-/// 取号之后那一串里第几次发布（从 0 数：0 是写行那次，之后是暖机）的哪个角色在分配器的拷贝上取不到落点、分配器给的原因。
-struct PlacementRefusedOnTheCopy {
+/// 取号之后那一串里第几次发布（从 0 数：0 是写行那次，之后是暖机）在分配器的拷贝上预演时报的错。
+struct DryRunRefusal {
     publish_index: usize,
-    unit: TransactionUnit,
-    refusal: PlacementRefusal,
-}
-
-/// 在分配器的拷贝上走那一串的结局。
-enum PlacementsOnTheCopy {
-    /// 每一次的每个角色都取到了：按次序交回每一次取到的。
-    TakenByEveryPublish(Vec<PlacementsTakenByOnePublish>),
-    Refused(PlacementRefusedOnTheCopy),
-    /// 第 `publish_index` 次（从 0 数，0 是写行那次）在拷贝上释放换下的落点时释放核验报错（`transaction::placements_to_release_via_mapping`
-    /// 的那几种，或映射条目的位置项不是池里两块不同的盘——`transaction::refuse_mapping_entries_that_do_not_name_two_pool_devices`）：
-    /// 发布路径走到同一处报同一个错（它只读内存里的上一版与分配器，取号不碰这两样）。D18（块里携带什么信息） 已定项 11「可写挂载的顺序」
-    /// 第五个合取：预演里写行那次的释放核验报错，同样判这次不能可写——在取号之前拒，不烧实例号。暖机那几次预演取不到落点同样不能可写。
-    ReleaseCheckRefused {
-        publish_index: usize,
-        cause: PublishError,
-    },
-}
-
-/// 一个角色在分配器上取落点：与发布路径同一条政策（`TransactionUnit::placement`：用户数据按政策函数，其余是提交内生块）。
-fn take_placement_for_role(
-    allocator: &mut PoolAllocator,
-    role: TransactionUnit,
-    txg: CheckpointTxg,
-) -> Result<Placement, PlacementRefusal> {
-    match role.placement() {
-        PlacementRule::UserData => allocator.try_allocate_user_data(txg),
-        PlacementRule::CommitGenerated(footprint) => {
-            allocator.try_allocate_commit_generated(footprint, txg)
-        }
-    }
+    cause: PublishError,
 }
 
-/// 取号之前把这次挂载取号之后要发的那一串——写行一次、暖机 `warm_up_publish_txgs` 那几次——在分配器的一份拷贝上逐次取一遍落点。
-/// 只动拷贝、不碰盘、不读盘。
-///
-/// 动分配器的步骤与发布路径逐步相同（`transaction::publish_version` 与 `transaction::publish_instance_table_on_version_without_file`）：
-/// 先释放这一次换下的落点（进 defer、槽仍占着），再按 bump 次序给这次重写的每个角色取落点，取完记这条根盖掉的根环槽
-/// （`PoolAllocator::record_root_written_by_this_process`：根环转过就按谓词回收、清离开根环的被抛弃根的隔离位）。重写的角色：
-/// 带文件的一版上写行是 `PublishShape::row_publish_rewriting_instance_table_pages`（这次之后实例表链几片就重写几个实例表角色，
-/// 尾片先）、暖机是 `PublishShape::EMPTY_PUBLISH`（`publish_empty_after` 那条断言钉住暖机与它相同）里不属于多层码 2 树的那几个，
-/// 加这一次记账树与中央映射树要重写的节点——按两棵树的形状逐次推
-/// （`transaction::multi_level_tree_nodes_of_a_publish_sequence_without_content`，与发布路径同一个规划），插在树表之前。
-/// 写行换下的是这一版的整条实例表链（`transaction::instance_table_chain_to_release`，排最前）加经映射查到的那几个，
-/// 与 `transaction::publish_version` 同序；暖机换下的：不属于多层码 2 树的角色是上一次在拷贝上为它取到的那个，
-/// 两棵树换下的节点是哪一次写出来的就取哪一次的落点（这一串之前那一版的经那一版的映射或父指针查）。
-/// 树表 0 条那一版上写行是实例表链各片（尾片先）加分配记录树节点（这次没有行要写时是零单元发布），暖机是零单元发布。
-/// 那一版上写行换下的旧链与分配记录树节点不在这里释放：它之后只有零单元发布，释放不释放，取到的落点都一样。
+/// 取号之前把这次挂载取号之后要发的那一串——写行一次、暖机 `warm_up_publish_txgs` 那几次——在分配器的一份拷贝上整串预演一遍，
+/// 交回每一次取到的落点。只动拷贝、不碰盘；**走的就是发布路径落盘之前那一段**（带文件的一版 `transaction::prepare_the_version_publish`，
+/// 树表 0 条的一版上写行 `transaction::prepare_the_row_publish_on_a_version_without_file`，零单元发布
+/// `transaction::prepare_the_publish_without_units`），计划也是取号之后那几次发布用的同一张（`row_publish_plan_on_file_version`、
+/// `empty_publish_plan_after_file_version`、`empty_publish_plan_after_version_without_file`），不另推一份：
+/// 这一串每一次重写哪些角色——分配记录树重写哪几个节点要在分配器上走到固定点才知道（D8（核心索引结构） 已定项 14）——
+/// 与真发时是同一段代码算的。写行那次的释放核验、树的形状、落点取不到，都在这里先报出来，在取号之前拒（D18（块里携带什么信息） 已定项 11
+/// 「可写挂载的顺序」第五个合取；增补 2 第 20a 行）。
 ///
 /// 发布路径在取落点之前还读盘核换下的每一份的校验和，对不上（读不出也算）的那一份在它那块盘上的分配记录留在已分配、不释放
-/// （`transaction::copies_failing_the_release_checksum_check`，D19（块指针的结构与宽度预算） 已定项 5），这里不做（它要读盘）、照常释放。
+/// （D19（块指针的结构与宽度预算） 已定项 5），这里不读盘（`ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition`）、照常释放。
 /// 两边只在那一槽这一串里被回收时分叉：拷贝上它回收了、可能再发出去，真发时它一直占着；这一串里回收它，得这一串自己的根把根环里
 /// 比它旧的有效根全盖掉（回退到环里最旧的那条根、其余全被抛弃时走得到）。所以这一串里没有一份核出对不上时，这里取到的与真发起来取到的逐项相同
 /// （`establish_instance` 在这一串发完之后断言）；有一份核出对不上时可能不同，那一格见 `establish_instance` 的注释。
-#[allow(
-    clippy::too_many_lines,
-    reason = "一串发布在拷贝上逐次释放、取落点、记根：每一步与发布路径逐步对齐，拆开只会把次序藏进几个函数"
-)]
-#[allow(
-    clippy::too_many_arguments,
-    reason = "这一串的输入各是一样：上一版、实例表怎么重写、要写的行数、各次的 txg、要取的号、两棵树的节点容量，与准入读的是同一组"
-)]
-fn placements_of_the_publishes_after_acquisition_on_a_copy(
+///
+/// # Errors
+/// 第几次（从 0 数）的准备报了什么错（[`DryRunRefusal`]）。
+fn dry_run_of_the_publishes_after_acquisition<Device: BlockDevice>(
+    pool: &PoolWriter<'_, Device>,
     allocator: &PoolAllocator,
-    previous: &PreviousVersion,
+    start: &InstanceStart,
     instance_table_rewrite: &InstanceTableRewrite,
     rows_to_write: usize,
-    row_publish_txg: CheckpointTxg,
     warm_up_publish_txgs: &[CheckpointTxg],
     instance_to_acquire: InstanceGeneration,
-    code_two_tree_node_capacities: CodeTwoTreeNodeCapacities,
-) -> PlacementsOnTheCopy {
+) -> Result<Vec<PlacementsTakenByOnePublish>, DryRunRefusal> {
     let mut copy = allocator.clone();
-    let txgs: Vec<CheckpointTxg> = std::iter::once(row_publish_txg)
-        .chain(warm_up_publish_txgs.iter().copied())
-        .collect();
-    // 每一次重写的角色、这一次换下的上一版角色（只第 0 次有：写行那次换下的都是挂载时那一版的），与两棵多层码 2 树每一次换下的节点
-    // （后几次换下的可以是这一串里前面某一次写出来的，落点取那一次在拷贝上为那个角色取到的）。
-    let (roles_of_each_publish, released_roles_of_the_row_publish, tree_nodes_of_each_publish) =
-        match previous {
-            PreviousVersion::WithFile { output, .. } => {
-                let shapes: Vec<PublishShape> =
-                    std::iter::once(PublishShape::row_publish_rewriting_instance_table_pages(
-                        instance_table_rewrite.pages_after_this_publish(),
-                    ))
-                    .chain(std::iter::repeat_n(
-                        PublishShape::EMPTY_PUBLISH,
-                        warm_up_publish_txgs.len(),
-                    ))
-                    .collect();
-                let device_identities: Vec<DeviceIdentity> = allocator
-                    .devices
-                    .iter()
-                    .map(|device_map| device_map.device)
-                    .collect();
-                // 取号之前那道准入（`refuse_publishes_before_acquisition_that_do_not_pass_admission`）刚按同一个上一版、同一串形状、
-                // 同一个容量推过这两棵树、没拒；推得出推不出只看 key 的次序与条数，与 txg、实例代号的具体值无关。
-                let tree_nodes = multi_level_tree_nodes_of_a_publish_sequence_without_content(
-                    output,
-                    &shapes,
-                    &txgs,
+    let refused_at = |publish_index: usize| {
+        move |cause: PublishError| DryRunRefusal {
+            publish_index,
+            cause,
+        }
+    };
+    let row_publish = match &start.previous {
+        PreviousVersion::WithFile { output, .. } => {
+            let plan = row_publish_plan_on_file_version(
+                output,
+                instance_table_rewrite.clone(),
+                &RowPublishIdentity {
+                    txg: start.first_txg,
+                    counter: start.next_counter,
+                    instance: instance_to_acquire,
+                    rollback_floor: start.effective_floor,
+                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
+                },
+            );
+            prepare_the_version_publish(
+                pool,
+                &mut copy,
+                &plan,
+                Some(output),
+                output.tree_identifiers,
+                ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
+            )
+            .map(|(_, rehearsed)| PoolVersion::WithFile(rehearsed))
+            .map_err(refused_at(0))?
+        }
+        PreviousVersion::WithoutFile { root, .. } if rows_to_write == 0 => {
+            let (_, rehearsed) = prepare_the_publish_without_units(
+                pool,
+                root,
+                ZeroUnitPublishPlan {
+                    txg: start.first_txg,
+                    counter: start.next_counter,
+                    instance: instance_to_acquire,
+                    back_chain: 0,
+                    rollback_floor: start.effective_floor,
+                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
+                },
+            );
+            copy.record_root_written_by_this_process(rehearsed.root.checkpoint_txg);
+            PoolVersion::WithoutFile(rehearsed)
+        }
+        PreviousVersion::WithoutFile { root, .. } => {
+            prepare_the_row_publish_on_a_version_without_file(
+                pool,
+                &mut copy,
+                root,
+                InstanceTableOnlyPublishPlan {
+                    txg: start.first_txg,
+                    counter: start.next_counter,
+                    instance: instance_to_acquire,
+                    back_chain: 0,
+                    rollback_floor: start.effective_floor,
+                    instance_table: instance_table_rewrite,
+                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
+                },
+            )
+            .map(|(_, rehearsed)| PoolVersion::WithoutFile(rehearsed))
+            .map_err(refused_at(0))?
+        }
+    };
+    let mut taken_by_every_publish = vec![placements_taken_by(&row_publish)];
+    let mut current = row_publish;
+    // 迭代上界是暖机的次数；跨轮携带的是预演出来的现行那一版（下一次的计划接着它）。
+    for (warm_up_position, planned_txg) in warm_up_publish_txgs.iter().enumerate() {
+        let publish_index = warm_up_position + 1;
+        let next = match &current {
+            PoolVersion::WithFile(current_file_version) => {
+                let plan = empty_publish_plan_after_file_version(
+                    current_file_version,
                     instance_to_acquire,
-                    &device_identities,
-                    code_two_tree_node_capacities,
-                )
-                .expect("取号之前那道准入刚按同样的输入推过这两棵树、没拒");
-                let roles_of_each_publish: Vec<Vec<TransactionUnit>> = shapes
-                    .iter()
-                    .zip(&tree_nodes)
-                    .map(|(shape, nodes)| {
-                        rewritten_roles_of_a_publish_without_content(*shape, &nodes.rewritten_roles)
-                    })
-                    .collect();
-                let released_roles_of_the_row_publish: Vec<TransactionUnit> = roles_of_each_publish
-                    [0]
-                .iter()
-                .copied()
-                .filter(|identity| multi_level_tree_of_role(*identity).is_none())
-                .chain(
-                    tree_nodes[0]
-                        .replaced
-                        .iter()
-                        .map(|written_by| match written_by {
-                            TreeNodeWrittenBy::TheVersionBeforeTheSequence(role) => *role,
-                            TreeNodeWrittenBy::PublishOfTheSequence { .. } => {
-                                unreachable!("第 0 次换下的节点都是挂载时那一版的")
-                            }
-                        }),
-                )
-                .collect();
-                (
-                    roles_of_each_publish,
-                    released_roles_of_the_row_publish,
-                    tree_nodes,
+                );
+                prepare_the_version_publish(
+                    pool,
+                    &mut copy,
+                    &plan,
+                    Some(current_file_version),
+                    current_file_version.tree_identifiers,
+                    ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
                 )
+                .map(|(_, rehearsed)| PoolVersion::WithFile(rehearsed))
+                .map_err(refused_at(publish_index))?
             }
-            PreviousVersion::WithoutFile { .. } if rows_to_write == 0 => (
-                txgs.iter().map(|_| Vec::new()).collect(),
-                Vec::new(),
-                Vec::new(),
-            ),
-            PreviousVersion::WithoutFile { .. } => (
-                txgs.iter()
-                    .enumerate()
-                    .map(|(publish_index, _)| {
-                        if publish_index == 0 {
-                            instance_table_page_roles_in_bump_order(
-                                instance_table_rewrite.pages_after_this_publish(),
-                            )
-                            .into_iter()
-                            .chain(std::iter::once(TransactionUnit::AllocationTree))
-                            .collect()
-                        } else {
-                            Vec::new()
-                        }
-                    })
-                    .collect(),
-                Vec::new(),
-                Vec::new(),
-            ),
-        };
-    let mut taken_by_every_publish: Vec<PlacementsTakenByOnePublish> = Vec::new();
-    for (publish_index, txg) in txgs.iter().copied().enumerate() {
-        let roles = &roles_of_each_publish[publish_index];
-        // 先释放这一次换下的（进 defer、槽仍占着），再取这一次的落点（D3（空间分配） 已定项 7）。
-        let released: Vec<Placement> = if publish_index == 0 {
-            match previous {
-                PreviousVersion::WithFile { output, .. } => {
-                    // 整条实例表旧链排最前，再是经映射查到的那几个（实例表豁免映射，经映射那一路跳过它），与发布路径同序；
-                    // 经映射换下的那几个，映射条目的位置项要各指池里一块不同的盘（发布路径在读盘核之前判的同一件事，不读盘）。
-                    let pool_devices: Vec<DeviceIdentity> = copy
-                        .devices
-                        .iter()
-                        .map(|device_map| device_map.device)
-                        .collect();
-                    let checked = instance_table_chain_to_release(
-                        &instance_table_rewrite.replaced_chain,
-                        &copy,
-                    )
-                    .and_then(|mut released| {
-                        released.extend(placements_to_release_via_mapping(
-                            output,
-                            &copy,
-                            &released_roles_of_the_row_publish,
-                        )?);
-                        refuse_mapping_entries_that_do_not_name_two_pool_devices(
-                            output,
-                            &released_roles_of_the_row_publish,
-                            &pool_devices,
-                        )?;
-                        Ok(released)
-                    });
-                    match checked {
-                        Ok(released) => released,
-                        Err(cause) => {
-                            return PlacementsOnTheCopy::ReleaseCheckRefused {
-                                publish_index,
-                                cause,
-                            }
-                        }
-                    }
-                }
-                PreviousVersion::WithoutFile { .. } => Vec::new(),
-            }
-        } else {
-            let taken_by_the_publish_before = &taken_by_every_publish[publish_index - 1];
-            // 不属于多层码 2 树的角色：这一次重写的，上一次也重写过，换下的就是上一次为它取到的那个落点。
-            let mut released: Vec<Placement> = roles
-                .iter()
-                .filter(|identity| multi_level_tree_of_role(**identity).is_none())
-                .filter_map(|role| {
-                    taken_by_the_publish_before
-                        .iter()
-                        .find(|(taken_role, _)| taken_role == role)
-                        .map(|(_, slot)| Placement {
-                            slot: *slot,
-                            span: role.span_slots(),
-                        })
-                })
-                .collect();
-            let tree_nodes_replaced = tree_nodes_of_each_publish
-                .get(publish_index)
-                .map_or(&[][..], |nodes| nodes.replaced.as_slice());
-            for written_by in tree_nodes_replaced {
-                match written_by {
-                    TreeNodeWrittenBy::TheVersionBeforeTheSequence(role) => {
-                        let PreviousVersion::WithFile { output, .. } = previous else {
-                            unreachable!("树表 0 条的一版没有多层码 2 树")
-                        };
-                        match placements_to_release_via_mapping(output, &copy, &[*role]) {
-                            Ok(placements) => released.extend(placements),
-                            Err(cause) => {
-                                return PlacementsOnTheCopy::ReleaseCheckRefused {
-                                    publish_index,
-                                    cause,
-                                }
-                            }
-                        }
-                    }
-                    TreeNodeWrittenBy::PublishOfTheSequence {
-                        publish_index: written_by_publish,
-                        role,
-                    } => {
-                        let (_, slot) = taken_by_every_publish[*written_by_publish]
-                            .iter()
-                            .find(|(taken_role, _)| taken_role == role)
-                            .expect("那一次重写的节点在那一次取到了落点");
-                        released.push(Placement {
-                            slot: *slot,
-                            span: role.span_slots(),
-                        });
-                    }
-                }
+            PoolVersion::WithoutFile(current_version_without_file) => {
+                let (_, rehearsed) = prepare_the_publish_without_units(
+                    pool,
+                    &current_version_without_file.root,
+                    empty_publish_plan_after_version_without_file(
+                        current_version_without_file,
+                        instance_to_acquire,
+                    ),
+                );
+                copy.record_root_written_by_this_process(rehearsed.root.checkpoint_txg);
+                PoolVersion::WithoutFile(rehearsed)
             }
-            released
         };
-        for placement in released {
-            copy.release(placement, txg);
-        }
-        let mut taken_by_this_publish: PlacementsTakenByOnePublish = Vec::new();
-        for role in roles {
-            match take_placement_for_role(&mut copy, *role, txg) {
-                Ok(placement) => taken_by_this_publish.push((*role, placement.slot)),
-                Err(refusal) => {
-                    return PlacementsOnTheCopy::Refused(PlacementRefusedOnTheCopy {
-                        publish_index,
-                        unit: *role,
-                        refusal,
-                    })
-                }
-            }
-        }
-        copy.record_root_written_by_this_process(txg);
-        taken_by_every_publish.push(taken_by_this_publish);
+        assert_eq!(
+            next.root().checkpoint_txg,
+            *planned_txg,
+            "预演的暖机 txg 与计划里的那一个相同：计划逐个加一，计划接着现行那一版的 txg 加一"
+        );
+        taken_by_every_publish.push(placements_taken_by(&next));
+        current = next;
     }
-    PlacementsOnTheCopy::TakenByEveryPublish(taken_by_every_publish)
+    Ok(taken_by_every_publish)
 }
 
 /// 一次发布真取到的落点，按取落点的次序：带文件的一版是这次重写的每个角色（`TransactionOutput::rewritten`）；
@@ -1562,12 +1360,15 @@
                 .chain(std::iter::once(&version_without_file.record))
                 .flat_map(|record| record.named.iter())
                 .collect();
-            let Some(instance_table_pages) = named.len().checked_sub(1) else {
-                return Vec::new();
-            };
-            instance_table_page_roles_in_bump_order(instance_table_pages)
-                .into_iter()
-                .chain(std::iter::once(TransactionUnit::AllocationTree))
+            assert_eq!(
+                named.len(),
+                version_without_file.rewritten.len(),
+                "树表 0 条的一版上一次发布的点名项与它写出的角色一一对应（写行那次按同一张角色清单点名，零单元发布两边都空）"
+            );
+            version_without_file
+                .rewritten
+                .iter()
+                .copied()
                 .zip(named)
                 .map(|(role, named_unit)| {
                 (
@@ -1775,52 +1576,46 @@
         start.previous.instance_table_chain(),
         &rows_written,
     );
-    refuse_publishes_before_acquisition_that_do_not_pass_admission(
+    // 取号之后要发的那一串（写行一次、暖机那几次）在取号之前整串预演一遍（分配器的拷贝上，走发布路径落盘之前那一段，
+    // 与后面真发读同一个分配器——取号不碰它）：释放核验、树的形状、落点取不到，都在任何写之前返回。
+    // 落点取不到怎么算进取号之前的准入，条款没定（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided` 的文档注释）。
+    let placements_planned = match dry_run_of_the_publishes_after_acquisition(
+        &pool,
         &allocator,
         &start,
-        instance_to_acquire,
         &instance_table_rewrite,
         rows_written.len(),
-        warm_up_publishes_planned.len(),
-        pool.code_two_tree_node_capacities(),
-    )?;
-    // 这一串自己的落点也在取号之前先取一遍（分配器的拷贝上，与后面真发读同一个分配器——取号不碰它）：取不到就在任何写之前返回。
-    // 怎么把它们算进取号之前的准入，条款没定（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided` 的文档注释）。
-    let placements_planned = match placements_of_the_publishes_after_acquisition_on_a_copy(
-        &allocator,
-        &start.previous,
-        &instance_table_rewrite,
-        rows_written.len(),
-        start.first_txg,
         &warm_up_publishes_planned,
         instance_to_acquire,
-        pool.code_two_tree_node_capacities(),
     ) {
-        PlacementsOnTheCopy::TakenByEveryPublish(taken) => Some(taken),
-        PlacementsOnTheCopy::Refused(refused) => {
+        Ok(taken) => Some(taken),
+        Err(DryRunRefusal {
+            publish_index,
+            cause: PublishError::PlacementRefused { unit, refusal },
+        }) => {
             return Err(
                 MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
                     instance_to_acquire,
-                    publish_index: refused.publish_index,
+                    publish_index,
                     warm_up_publishes_planned: warm_up_publishes_planned.len(),
-                    unit: refused.unit,
-                    refusal: refused.refusal,
+                    unit,
+                    refusal,
                 },
             )
         }
-        PlacementsOnTheCopy::ReleaseCheckRefused {
+        Err(DryRunRefusal {
             publish_index: 0,
             cause,
-        } => {
+        }) => {
             return Err(MountError::RowPublishAdmissionRefusedBeforeAcquisition {
                 instance_to_acquire,
                 cause,
             })
         }
-        PlacementsOnTheCopy::ReleaseCheckRefused {
+        Err(DryRunRefusal {
             publish_index: warm_up_publish_index,
             cause,
-        } => {
+        }) => {
             return Err(MountError::WarmUpAdmissionRefusedBeforeAcquisition {
                 instance_to_acquire,
                 warm_up_publish_index,
@@ -1952,7 +1747,7 @@
     }
     // 取号之前在拷贝上取的落点就是真发起来取的：同一个分配器、同一串分配动作。这一串里有一份换下的单元读盘核校验和对不上时不比——
     // 真发时那一份的分配记录留在已分配，拷贝上没做那一步、照常释放，那一槽若在这一串里被回收（回退到环里最旧的那条根、其余全被抛弃），两边可以不同
-    // （`placements_of_the_publishes_after_acquisition_on_a_copy` 的文档注释）；那一格取号之前判的与真发的不是同一串，是这一版留着的缺口。
+    // （`dry_run_of_the_publishes_after_acquisition` 的文档注释）；那一格取号之前判的与真发的不是同一串，是这一版留着的缺口。
     let placements_taken: Vec<PlacementsTakenByOnePublish> = std::iter::once(&row_publish)
         .chain(&warm_up_publishes)
         .map(placements_taken_by)
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/recovery.rs tree/crates/singlefs-core/src/recovery.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/recovery.rs	2026-09-24 18:48:58.241538946 +0000
+++ tree/crates/singlefs-core/src/recovery.rs	2026-09-24 20:56:53.071717073 +0000
@@ -11,23 +11,32 @@
 use std::collections::{BTreeMap, BTreeSet};
 
 use singlefs_format::{
-    journal_in_flight_record_limit, ACCOUNTING_ENTRY_BYTES, ALLOCATION_RECORD_BYTES,
-    DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES, FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
-    INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT,
-    MAPPING_ENTRY_BYTES, NODE_BYTES, ROOT_RING_REGIONS, SLOT_BYTES,
-    SYSTEM_CONFIGURATION_SLOT_BYTES, UNIT_AREA_START_SLOT,
+    journal_in_flight_record_limit, ACCOUNTING_ENTRY_BYTES, DATA_UNIT_BYTES,
+    FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES, INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES,
+    JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, MAPPING_ENTRY_BYTES, NODE_BYTES,
+    ROOT_RING_REGIONS, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES, UNIT_AREA_START_SLOT,
 };
 
 use crate::address::{
     CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
     SlotNumber, TreeIdentifier,
 };
-use crate::allocator::{unit_area_slots_of_device, AllocationRecord};
+use crate::allocation_record_tree::{
+    read_allocation_record_tree, AllocationRecordTreeGeometry, AllocationRecordTreeHeaderJudgement,
+    AllocationRecordTreeReadFromDisk,
+};
+use crate::allocator::{
+    unit_area_slots_of_device, AllocationRecord, AllocationRecordTreeOfTheVersionWithoutFile,
+};
 use crate::block_device::BlockDevice;
 use crate::checksum::crc32_castagnoli;
 use crate::code_two_tree::{
     read_code_two_tree, CodeTwoTreeHeaderJudgement, CodeTwoTreeReadFromDisk,
 };
+use crate::extent_tree::{
+    read_extent_tree, ExtentTreeHeaderJudgement, ExtentTreeReadFromDisk, ExtentTreeReading,
+    ExtentTreeVersion, ExtentsOfAFileReadFromDisk,
+};
 use crate::inode_tree::{InodeLeafContainer, InodeLeafContainerIndexInTree};
 use crate::instance_table::{
     InstanceTableChainRecord, InstanceTablePage, InstanceTablePageIndex, InstanceTableRecords,
@@ -38,10 +47,10 @@
 use crate::make_filesystem::TREE_TABLE_KEY_WIDTH;
 use crate::pointer::{DataPointer, LocationEntry, NodePointer};
 use crate::records::{
-    mapping_key_for_data, mapping_key_for_node, parse_extent_record, parse_inode_internal_entry,
-    parse_mapping_entry, AccountingEntry, InodeRecord, TreeTableEntry, STATISTIC_INODE_WATERMARK,
-    TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_EXTENT,
-    TREE_KIND_INODE, TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE,
+    mapping_key_for_data, mapping_key_for_node, parse_inode_internal_entry, parse_mapping_entry,
+    AccountingEntry, InodeRecord, TreeTableEntry, STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING,
+    TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_EXTENT, TREE_KIND_INODE,
+    TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE,
 };
 use crate::rollback_witness::RollbackWitness;
 use crate::root_record::RootRecord;
@@ -51,8 +60,9 @@
     SystemConfiguration, SystemConfigurationSlotRefusal, SystemImmutableSizes,
 };
 use crate::transaction::{
-    FileVersionTreeIdentifiers, InodeLeafContainerVersion, MultiLevelCodeTwoTree, PublishedUnit,
-    TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
+    role_of_allocation_record_tree_node, role_of_extent_upper_node, FileVersionTreeIdentifiers,
+    InodeLeafContainerVersion, MultiLevelCodeTwoTree, PublishedUnit, TransactionOutput,
+    TransactionUnit, FIRST_INODE_NUMBER,
 };
 use crate::unit::{
     data_unit_payload, data_unit_payload_capacity, parse_data_unit, parse_index_node,
@@ -876,34 +886,10 @@
         .unwrap_or(CheckpointTxg(0))
 }
 
-/// 把一个索引节点的条目逐条解成分配记录，再按这个池的几何判一遍。
-///
-/// 两道判分得开：前一道是**字段表**（条目宽够不够装下 20 字节的记录），后一道是**几何**
-/// （设备身份、槽号、跨度、两条记录罩不罩同一个槽）。分配器那一侧的下标、位图长度与两条断言
-/// 全部按这两道已经判过来写（`allocator::DeviceFreeMap::index` / `mark_allocated` 的消息指着这里）。
-///
-/// # Errors
-/// 条目宽不足 ⇒ [`RecoveryFailure::EntryNarrowerThanItsFieldTable`]；
-/// 结构值出了这个池的几何 ⇒ [`RecoveryFailure::AllocationRecordOutsideThePoolGeometry`]。
-fn allocation_records_of_node(
-    reader: &dyn PoolReader,
-    node: &IndexNodeHeader,
-) -> Result<Vec<AllocationRecord>, RecoveryFailure> {
-    let mut records = Vec::with_capacity(node.entries.len());
-    for bytes in &node.entries {
-        records.push(AllocationRecord::parse(bytes).ok_or(
-            RecoveryFailure::EntryNarrowerThanItsFieldTable {
-                what: "分配记录",
-                entry_bytes: bytes.len(),
-                field_table_bytes: usize::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
-            },
-        )?);
-    }
-    allocation_records_fit_the_pool_geometry(reader, &records)?;
-    Ok(records)
-}
-
 /// 盘上读来的分配记录逐条对这个池的几何判一遍：这几个字段可以是任何值，它们不是不变量。
+/// 读分配记录树时两道判分得开：前一道是**字段表**与**位置**（条目宽够不够装下 20 字节的记录、记录落不落在它所在叶里，
+/// `crate::allocation_record_tree::read_allocation_record_tree`），这一道是**几何**（设备身份、槽号、跨度、两条记录罩不罩同一个槽）。
+/// 分配器那一侧的下标、位图长度与两条断言全部按这两道已经判过来写（`allocator::DeviceFreeMap::index` / `mark_allocated` 的消息指着这里）。
 ///
 /// 四样各对着 panic 面普查里的一条：设备身份不在池里（R9，`PoolAllocator::rebuild_from_records` 的 `expect`）、
 /// 槽号在单元区起点之下（R6，`DeviceFreeMap::index` 的减法）、跨度越过单元区末尾与同一块盘上两条记录罩住同一个槽
@@ -968,70 +954,73 @@
         parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
             what: "树表单元",
         })?;
-    let mut allocation_pointer = None;
+    let mut allocation_entry = None;
     for bytes in &tree_table.entries {
         let entry = TreeTableEntry::parse(bytes).ok_or(RecoveryFailure::UnitMalformed {
             what: "树表条目",
         })?;
         if entry.kind == TREE_KIND_ALLOCATION {
-            allocation_pointer = Some(entry.root);
+            allocation_entry = Some(entry);
         }
     }
-    let Some(allocation_pointer) = allocation_pointer else {
+    let Some(allocation_entry) = allocation_entry else {
         return Ok(Vec::new());
     };
     let central_mapping_root = CentralMappingTreeWithBytesReadOnFirstUse::new(reader, root);
     let mut stale_location_hint_hops_not_exposed_by_this_reader = 0usize;
-    let allocation_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
-        reader,
-        &allocation_pointer,
-        MappedTreeNodeClass::IndexNode,
-        &|mapping_key: &[u8]| central_mapping_root.locations_of_key(mapping_key),
-        &mut stale_location_hint_hops_not_exposed_by_this_reader,
+    // 分配记录树按绝对槽号按位置寻址、可以多层（D8（核心索引结构） 已定项 14）：整棵读回来，节点进映射、提示读不出经这条根的中央映射回退。
+    let tree = read_allocation_record_tree(
+        &allocation_entry.root,
+        &AllocationRecordTreeGeometry::of_reader(reader),
+        allocation_entry.tree,
+        AllocationRecordTreeHeaderJudgement::OnlyWhatThePositionsNeed,
+        root,
+        unit_filesystem_identifier(&root.filesystem_identifier),
+        &mut |pointer: &NodePointer| {
+            read_mapped_tree_node_via_hint_then_central_mapping(
+                reader,
+                pointer,
+                MappedTreeNodeClass::IndexNode,
+                &|mapping_key: &[u8]| central_mapping_root.locations_of_key(mapping_key),
+                &mut stale_location_hint_hops_not_exposed_by_this_reader,
+            )
+        },
     )?;
-    let allocation_node =
-        parse_index_node(&allocation_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
-            what: "分配记录树根",
-        })?;
-    // 第一版的分配记录树只有一个节点（层 0），多层的树这条路还不会走（里程碑「第二个事务」步 6 的欠账）；
-    // 读到层 > 0 的根就报格式错，不把内部节点的指针当分配记录解。
-    if allocation_node.level != 0 {
-        return Err(RecoveryFailure::UnitMalformed {
-            what: "分配记录树根不止一层",
-        });
-    }
-    allocation_records_of_node(reader, &allocation_node)
+    allocation_records_fit_the_pool_geometry(reader, &tree.records)?;
+    Ok(tree.records)
 }
 
-/// 树表 0 条的那一版自己那棵分配记录树（根指针住根记录那一项，C512（树表 0 条的一版上被换下的单元记在哪））：
+/// 树表 0 条的那一版自己那棵分配记录树（根指针住根记录那一项，C512（树表 0 条的一版上被换下的单元记在哪））：整棵读回来
+/// （D8（核心索引结构） 已定项 14：挂载时分配记录树整棵读），交回它的节点与指针、全部记录。树号 0、节点豁免映射，只按位置条目读。
 /// 指针全零 ⇒ `None`，那是 mkfs 的第 0 代（那一版的账由实例表与树表两条指针直接算）。
 ///
 /// # Errors
-/// 分配记录树根读不到、解不开、不止一层；条目宽或结构值判红（见 `allocation_records_of_node`）。
+/// 分配记录树的节点读不到、解不开、位置对不上（`crate::allocation_record_tree::read_allocation_record_tree`）；结构值判红
+/// （`allocation_records_fit_the_pool_geometry`）。
 pub fn allocation_records_of_version_without_file(
     reader: &dyn PoolReader,
     root: &RootRecord,
-) -> Result<Option<Vec<AllocationRecord>>, RecoveryFailure> {
+) -> Result<Option<AllocationRecordTreeOfTheVersionWithoutFile>, RecoveryFailure> {
     if root.allocation_record_tree_root == NodePointer::empty_root() {
         return Ok(None);
     }
     let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
-    let allocation_bytes = read_unit_via_locations(
-        reader,
-        &root.allocation_record_tree_root.locations,
-        node_bytes,
+    let tree = read_allocation_record_tree(
+        &root.allocation_record_tree_root,
+        &AllocationRecordTreeGeometry::of_reader(reader),
+        TreeIdentifier(crate::transaction::TREE_IDENTIFIER_NONE),
+        AllocationRecordTreeHeaderJudgement::OnlyWhatThePositionsNeed,
+        root,
+        unit_filesystem_identifier(&root.filesystem_identifier),
+        &mut |pointer: &NodePointer| {
+            read_unit_via_locations(reader, &pointer.locations, node_bytes)
+        },
     )?;
-    let allocation_node =
-        parse_index_node(&allocation_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
-            what: "分配记录树根",
-        })?;
-    // 与 `allocation_records_under_root` 同一条：第一版的分配记录树只有一个节点（层 0）。
-    if allocation_node.level != 0 {
-        return Err(RecoveryFailure::UnitMalformed {
-            what: "分配记录树根不止一层",
-        });
-    }
-    allocation_records_of_node(reader, &allocation_node).map(Some)
+    allocation_records_fit_the_pool_geometry(reader, &tree.records)?;
+    Ok(Some(AllocationRecordTreeOfTheVersionWithoutFile {
+        version: tree.version(),
+        records: tree.records,
+    }))
 }
 
 /// 一条根指着的整张实例表（全部行）：沿链读到「无下一片」为止（[`instance_table_chain_of_root`]）；
@@ -1298,33 +1287,37 @@
                 .map_err(|_error| RecoveryFailure::UnitMalformed { what })?;
             Ok::<(Vec<u8>, IndexNodeHeader), RecoveryFailure>((bytes, node))
         };
-    let (extent_bytes, extent_node) = read_node(
-        &extent_pointer,
-        "extent 树根",
-        &mut stale_location_hint_hops_not_exposed_by_this_reader,
-    )?;
-    if extent_node.entries.is_empty() {
-        return Err(RecoveryFailure::UnitMalformed {
-            what: "extent 树根没有记录",
-        }
-        .into());
-    }
-    // extent 根兼叶里的记录按 key 升序，第 i 条就是文件第 i 个数据单元（一个文件跨多个单元，并行线一）；
-    // 每条指的数据单元都读回来，照抄进这一版的角色（覆盖写经映射释放它们要这几个 key，写行与暖机照抄它们的字节）。
+    // extent 树按 key 空间定形状（D8（核心索引结构） 已定项 14 的两段）：整棵读回来，每个节点按位置核，节点进映射、提示读不出经映射回退。
+    // 第一个文件的数据指针按单元号排，第 i 个就是文件第 i 个数据单元（一个文件跨多个单元，并行线一）；
+    // 每个指的数据单元都读回来，照抄进这一版的角色（覆盖写经映射释放它们要这几个 key，写行与暖机照抄它们的字节）。
     // 提示与映射都读不出的数据单元照抄它的位置项、不读内容，挂载照常，读到那个文件时才报错（D19（块指针的结构与宽度预算）
     // 已定项 5，用户 2026-09-24 定 N2）：它在这一版 `units` 里那一项的字节是空的。照抄它的发布只搬这一项、不写它的字节，
     // 重写它的发布按新内容装；释放它的发布照映射条目读盘核（已定项 5 硬规则 1），不读这里的字节。
-    let mut data_pointers: Vec<DataPointer> = Vec::with_capacity(extent_node.entries.len());
+    let expected_filesystem_identifier = unit_filesystem_identifier(&root.filesystem_identifier);
+    let extent_tree_read = read_extent_tree(
+        &ExtentTreeReading {
+            tree: tree_identifiers.extent,
+            judgement: ExtentTreeHeaderJudgement::OnlyWhatThePositionsNeed,
+            root,
+            expected_filesystem_identifier,
+        },
+        &extent_pointer,
+        &mut |pointer: &NodePointer| {
+            read_mapped_tree_node_via_hint_then_central_mapping(
+                reader,
+                pointer,
+                MappedTreeNodeClass::IndexNode,
+                &central_mapping_locations_of_key,
+                &mut stale_location_hint_hops_not_exposed_by_this_reader,
+            )
+        },
+    )?;
+    let first_file_extents = extents_of_the_first_file(extent_tree_read)?;
+    let mut data_pointers: Vec<DataPointer> =
+        Vec::with_capacity(first_file_extents.data_pointers.len());
     let mut data_unit_contents_in_file_order: Vec<Vec<u8>> =
-        Vec::with_capacity(extent_node.entries.len());
-    for extent_record_bytes in &extent_node.entries {
-        let (_, data_pointer) = parse_extent_record(extent_record_bytes).ok_or(
-            RecoveryFailure::EntryNarrowerThanItsFieldTable {
-                what: "extent 叶记录",
-                entry_bytes: extent_node.entry_width,
-                field_table_bytes: usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
-            },
-        )?;
+        Vec::with_capacity(first_file_extents.data_pointers.len());
+    for data_pointer in first_file_extents.data_pointers.iter().copied() {
         let content_or_nothing_when_unreadable = match read_data_unit_via_hint_then_central_mapping(
             reader,
             &data_pointer,
@@ -1406,16 +1399,28 @@
         .ok_or(RecoveryFailure::UnitMalformed {
             what: "inode 树里没有第一个文件那条记录",
         })?;
-    let (allocation_bytes, allocation_node) = read_node(
+    // 分配记录树按绝对槽号按位置寻址（D8（核心索引结构） 已定项 14）：整棵读回来，每个节点按位置核，节点进映射、提示读不出经映射回退。
+    let allocation_tree_read = read_allocation_record_tree(
         &allocation_pointer,
-        "分配记录树根",
-        &mut stale_location_hint_hops_not_exposed_by_this_reader,
+        &AllocationRecordTreeGeometry::of_reader(reader),
+        tree_identifiers.allocation_records,
+        AllocationRecordTreeHeaderJudgement::OnlyWhatThePositionsNeed,
+        root,
+        expected_filesystem_identifier,
+        &mut |pointer: &NodePointer| {
+            read_mapped_tree_node_via_hint_then_central_mapping(
+                reader,
+                pointer,
+                MappedTreeNodeClass::IndexNode,
+                &central_mapping_locations_of_key,
+                &mut stale_location_hint_hops_not_exposed_by_this_reader,
+            )
+        },
     )?;
-    let allocation_records: Vec<AllocationRecord> =
-        allocation_records_of_node(reader, &allocation_node)?;
+    allocation_records_fit_the_pool_geometry(reader, &allocation_tree_read.records)?;
+    let allocation_records: Vec<AllocationRecord> = allocation_tree_read.records.clone();
     // 记账树可以是多层（D8（核心索引结构） 已定项 11）：从根往下整棵读回来，节点进映射、提示读不出经这一版的中央映射回退
     // （与读别的树根同一条）；中央映射树同样整棵读回来（下面）。两棵树按「拼得成一棵树」核（`code_two_tree::read_code_two_tree`）。
-    let expected_filesystem_identifier = unit_filesystem_identifier(&root.filesystem_identifier);
     let accounting_tree = read_code_two_tree(
         &accounting_pointer,
         &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),
@@ -1479,10 +1484,10 @@
             )
         })
         .collect();
-    mapped_units.push((
-        TransactionUnit::ExtentRoot,
-        mapping_key_for_node(UNIT_CLASS_INDEX_NODE, extent_pointer),
-    ));
+    // extent 树与分配记录树的每个节点都进映射（码 2 树节点，D19（块指针的结构与宽度预算） 已定项 8），按 bump 次序。
+    for (role, pointer, _) in &first_file_extents.nodes_in_bump_order {
+        mapped_units.push((*role, mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer)));
+    }
     for (position, container) in inode_leaf_containers.iter().enumerate() {
         mapped_units.push((
             TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(
@@ -1491,16 +1496,16 @@
             mapping_key_for_node(UNIT_CLASS_PACKED, container.pointer),
         ));
     }
-    mapped_units.extend([
-        (
-            TransactionUnit::InodeRoot,
-            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer),
-        ),
-        (
-            TransactionUnit::AllocationTree,
-            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer),
-        ),
-    ]);
+    mapped_units.push((
+        TransactionUnit::InodeRoot,
+        mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer),
+    ));
+    for (node, pointer, _) in &allocation_tree_read.nodes {
+        mapped_units.push((
+            role_of_allocation_record_tree_node(*node),
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
+        ));
+    }
     // 记账树的每个节点都进映射（码 2 树节点，D19（块指针的结构与宽度预算） 已定项 8），按 bump 次序、根在最末。
     let accounting_shape = &accounting_tree.version.shape;
     for (node, pointer) in accounting_shape
@@ -1531,11 +1536,9 @@
             )
         })
         .collect();
-    units.push(unit(
-        TransactionUnit::ExtentRoot,
-        &extent_pointer.locations,
-        extent_bytes,
-    ));
+    for (role, pointer, bytes) in first_file_extents.nodes_in_bump_order {
+        units.push(unit(role, &pointer.locations, bytes));
+    }
     for ((position, container), bytes) in inode_leaf_containers
         .iter()
         .enumerate()
@@ -1549,18 +1552,19 @@
             bytes,
         ));
     }
-    units.extend([
-        unit(
-            TransactionUnit::InodeRoot,
-            &inode_root_pointer.locations,
-            inode_root_bytes,
-        ),
-        unit(
-            TransactionUnit::AllocationTree,
-            &allocation_pointer.locations,
-            allocation_bytes,
-        ),
-    ]);
+    units.push(unit(
+        TransactionUnit::InodeRoot,
+        &inode_root_pointer.locations,
+        inode_root_bytes,
+    ));
+    let allocation_record_tree = allocation_tree_read.version();
+    for (node, pointer, bytes) in allocation_tree_read.nodes {
+        units.push(unit(
+            role_of_allocation_record_tree_node(node),
+            &pointer.locations,
+            bytes,
+        ));
+    }
     // 记账树与中央映射树的每个节点，按 bump 次序（树内先叶后根），与发布路径装出来的 `units` 同序。
     for (tree, read) in [
         (MultiLevelCodeTwoTree::Accounting, &accounting_tree),
@@ -1603,6 +1607,8 @@
         data_pointers,
         mapping_keys,
         allocation_records,
+        allocation_record_tree,
+        extent_tree: first_file_extents.version,
         accounting_entries,
         accounting_tree: accounting_tree.version,
         central_mapping_tree: mapping_tree.version,
@@ -2112,6 +2118,96 @@
     Ok(node)
 }
 
+/// 从盘上读回来的 extent 树里第一个文件的样子：它的数据指针（第 i 个是单元 i）、这一版 extent 树的节点与指针、
+/// 每个节点的角色、指针与字节（bump 次序：下段先叶后根，再上段先叶后根）。
+pub(crate) struct ExtentsOfTheFirstFile {
+    pub data_pointers: Vec<DataPointer>,
+    pub version: ExtentTreeVersion,
+    pub nodes_in_bump_order: Vec<(TransactionUnit, NodePointer, Vec<u8>)>,
+}
+
+/// 从整棵读回来的 extent 树里取第一个文件（第一版只有第一个文件有内容，`transaction` 只按它建这一版在内存里的样子）：
+/// 上段里要恰好只有它那一条叶条目，它的单元从 0 起连号、没有洞。
+///
+/// # Errors
+/// 上段里没有第一个文件的条目、或还有别的 inode 的条目、第一个文件一个单元都没有、单元有洞 ⇒ `UnitMalformed`：
+/// 这几样今天的写路径都写不出来（只有它写 extent 树、每个文件版本从偏移 0 顺序写、长度 0 的内容也写一个单元），
+/// 盘上读来的却可以是任何样子——第一版不支持，在任何落盘动作之前交回。
+pub(crate) fn extents_of_the_first_file(
+    tree: ExtentTreeReadFromDisk,
+) -> Result<ExtentsOfTheFirstFile, RecoveryFailure> {
+    let ExtentTreeReadFromDisk { upper, files } = tree;
+    let [(inode, extents)] =
+        <[(u64, ExtentsOfAFileReadFromDisk); 1]>::try_from(files).map_err(|_files| {
+            RecoveryFailure::UnitMalformed {
+                what: "extent 树上段里不是恰好一条叶条目（第一版只有第一个文件有内容）",
+            }
+        })?;
+    if inode != FIRST_INODE_NUMBER {
+        return Err(RecoveryFailure::UnitMalformed {
+            what: "extent 树上段里那一条叶条目不是第一个文件的",
+        });
+    }
+    let data_pointers_with_units = extents.data_pointers();
+    if data_pointers_with_units.is_empty() {
+        return Err(RecoveryFailure::UnitMalformed {
+            what: "extent 树里第一个文件一个数据单元都没有",
+        });
+    }
+    if data_pointers_with_units
+        .iter()
+        .enumerate()
+        .any(|(position, (unit, _))| u64::try_from(position).expect("单元序号") != *unit)
+    {
+        return Err(RecoveryFailure::UnitMalformed {
+            what: "extent 树里第一个文件的单元有洞（第一版不支持）",
+        });
+    }
+    let lower_nodes = match extents {
+        ExtentsOfAFileReadFromDisk::LowerSegment(segment) => segment.nodes,
+        ExtentsOfAFileReadFromDisk::NoDataUnit | ExtentsOfAFileReadFromDisk::Inline(_) => {
+            Vec::new()
+        }
+    };
+    let upper_root_level = upper
+        .nodes
+        .last()
+        .map(|(position, _, _)| position.level)
+        .expect("读回来的上段至少有根");
+    let version = ExtentTreeVersion {
+        upper_nodes: upper
+            .nodes
+            .iter()
+            .map(|(position, pointer, _)| (*position, *pointer))
+            .collect(),
+        lower_nodes: lower_nodes
+            .iter()
+            .map(|(position, pointer, _)| (*position, *pointer))
+            .collect(),
+    };
+    let nodes_in_bump_order = lower_nodes
+        .into_iter()
+        .map(|(position, pointer, bytes)| {
+            (TransactionUnit::ExtentLowerNode(position), pointer, bytes)
+        })
+        .chain(upper.nodes.into_iter().map(|(position, pointer, bytes)| {
+            (
+                role_of_extent_upper_node(position, upper_root_level),
+                pointer,
+                bytes,
+            )
+        }))
+        .collect();
+    Ok(ExtentsOfTheFirstFile {
+        data_pointers: data_pointers_with_units
+            .into_iter()
+            .map(|(_, pointer)| pointer)
+            .collect(),
+        version,
+        nodes_in_bump_order,
+    })
+}
+
 /// 分配记录「每个落点每盘各一条」（两盘同槽、同一批字段）：同一块盘上一个槽只许一条记录，每块盘各自的（槽, 跨度, 代, 已释放）集合相同，
 /// 每盘不少于 10 个落点（mkfs 2 + 第一个事务 8）。同盘同槽两条记录（代不同）在集合里是两个元素、两盘对称就过——第二轮攻方腿打中，
 /// 走读自己不判 key 严格递增，这里逐盘核槽号不重复。
@@ -2207,9 +2303,11 @@
 }
 
 struct TreeRoots {
-    extent: IndexNodeHeader,
+    extent: ExtentsOfTheFirstFile,
+    /// extent 树的号（数据单元头里的出生树要与它相同）。
+    extent_tree: TreeIdentifier,
     inode: IndexNodeHeader,
-    allocation: IndexNodeHeader,
+    allocation: AllocationRecordTreeReadFromDisk,
     accounting: CodeTwoTreeReadFromDisk,
     mapping: CodeTwoTreeReadFromDisk,
 }
@@ -2291,6 +2389,8 @@
         |mapping_key: &[u8]| central_mapping_tree.locations_of_key(mapping_key);
     let mut by_kind: BTreeMap<u16, IndexNodeHeader> = BTreeMap::new();
     let mut accounting_tree: Option<CodeTwoTreeReadFromDisk> = None;
+    let mut allocation_tree: Option<AllocationRecordTreeReadFromDisk> = None;
+    let mut extent_tree: Option<(TreeIdentifier, ExtentsOfTheFirstFile)> = None;
     for entry in &entries {
         if entry.tree.0 >= root.tree_identifier_watermark {
             return Err(RecoveryFailure::InvariantViolated {
@@ -2301,6 +2401,50 @@
         if entry.root == NodePointer::empty_root() {
             continue;
         }
+        // 分配记录树与 extent 树按 key 空间定形状（D8（核心索引结构） 已定项 14）：整棵读回来，每个节点按位置与父条目核；
+        // 节点进映射，提示读不出经映射回退。
+        if entry.kind == TREE_KIND_ALLOCATION {
+            allocation_tree = Some(read_allocation_record_tree(
+                &entry.root,
+                &AllocationRecordTreeGeometry::of_reader(reader),
+                entry.tree,
+                AllocationRecordTreeHeaderJudgement::EveryHeaderAgainstItsReference,
+                root,
+                expected_filesystem_identifier,
+                &mut |pointer: &NodePointer| {
+                    read_mapped_tree_node_via_hint_then_central_mapping(
+                        reader,
+                        pointer,
+                        MappedTreeNodeClass::IndexNode,
+                        &central_mapping_locations_of_key,
+                        mapping_fallbacks,
+                    )
+                },
+            )?);
+            continue;
+        }
+        if entry.kind == TREE_KIND_EXTENT {
+            let read = read_extent_tree(
+                &ExtentTreeReading {
+                    tree: entry.tree,
+                    judgement: ExtentTreeHeaderJudgement::EveryHeaderAgainstItsReference,
+                    root,
+                    expected_filesystem_identifier,
+                },
+                &entry.root,
+                &mut |pointer: &NodePointer| {
+                    read_mapped_tree_node_via_hint_then_central_mapping(
+                        reader,
+                        pointer,
+                        MappedTreeNodeClass::IndexNode,
+                        &central_mapping_locations_of_key,
+                        mapping_fallbacks,
+                    )
+                },
+            )?;
+            extent_tree = Some((entry.tree, extents_of_the_first_file(read)?));
+            continue;
+        }
         // 记账树可以是多层（D8（核心索引结构） 已定项 11）：整棵读回来，每个节点按父条目核；节点进映射，提示读不出经映射回退。
         if entry.kind == TREE_KIND_ACCOUNTING {
             accounting_tree = Some(read_code_two_tree(
@@ -2343,24 +2487,27 @@
             what: "树表里缺一棵有根的树",
         })
     };
+    let missing_tree = || RecoveryFailure::UnitMalformed {
+        what: "树表里缺一棵有根的树",
+    };
+    let (extent_tree_identifier, extent) = extent_tree.ok_or_else(missing_tree)?;
     let roots = TreeRoots {
-        extent: take(TREE_KIND_EXTENT)?,
+        extent,
+        extent_tree: extent_tree_identifier,
         inode: take(TREE_KIND_INODE)?,
-        allocation: take(TREE_KIND_ALLOCATION)?,
-        accounting: accounting_tree.ok_or(RecoveryFailure::UnitMalformed {
-            what: "树表里缺一棵有根的树",
-        })?,
+        allocation: allocation_tree.ok_or_else(missing_tree)?,
+        accounting: accounting_tree.ok_or_else(missing_tree)?,
         // 中央映射树不进树表：它是哪棵树由根记录里它那条根指针的出生树说（第一个文件版本那次从水位发的号）。
         // 上面有树根的提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份，不再读一次。
         mapping: central_mapping_tree.into_tree()?,
     };
     let device_identities = reader.device_identities();
     let device_count = device_identities.len();
-    let allocation_records: Vec<AllocationRecord> =
-        allocation_records_of_node(reader, &roots.allocation)?;
+    allocation_records_fit_the_pool_geometry(reader, &roots.allocation.records)?;
+    let allocation_records: &[AllocationRecord] = &roots.allocation.records;
     // 每个落点每盘一条（两盘同槽）：第一个事务 10 × 盘数，每次覆盖写再加 8 × 盘数（换下的那些改写、不删）。
     // 只核总数是盘数的整数倍拦不住「一盘多一条、另一盘少一条」——发布 B 三方第一轮正推腿打中，改成逐盘核同一批（槽, 跨度）。
-    if !allocation_records_are_one_per_device(&allocation_records, &device_identities) {
+    if !allocation_records_are_one_per_device(allocation_records, &device_identities) {
         return Err(RecoveryFailure::InvariantViolated {
             invariant: "E142 走读同款",
             detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于 10 个落点",
@@ -2372,22 +2519,24 @@
             detail: "记账条目数不是 3 + 6 × 盘数",
         });
     }
-    // 进映射的单元：extent 根、inode 根、分配记录树各一条，记账树每个节点一条，加上 inode 树的每一片叶容器与文件的每一个数据单元
+    // 进映射的单元：inode 根一条，extent 树、分配记录树、记账树每个节点一条，加上 inode 树的每一片叶容器与文件的每一个数据单元
     // （映射树自己的节点、树表、实例表豁免，D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
     // inode 树的叶容器数 = 它的根的条目数（根恒是层级 1，下面每条条目一片容器；层级检查在下面）；
-    // 数据单元数 = extent 根兼叶的记录条数（第一版 extent 树只有一个节点，一条记录指一个数据单元）。
-    let mapping_entries_expected = 3
+    // 数据单元数 = extent 树里第一个文件的数据指针数（下段叶记录条数，或内联的那一个）。
+    let mapping_entries_expected = 1
+        + roots.extent.nodes_in_bump_order.len()
+        + roots.allocation.nodes.len()
         + roots.accounting.version.node_count()
         + roots.inode.entries.len()
-        + roots.extent.entries.len();
+        + roots.extent.data_pointers.len();
     if roots.mapping.leaf_entries_in_key_order.len() != mapping_entries_expected {
         return Err(RecoveryFailure::InvariantViolated {
             invariant: "E142 走读同款",
-            detail: "映射条目数不是 3 + 记账树节点数 + inode 叶容器数 + extent 记录数",
+            detail: "映射条目数不是 1 + extent 树、分配记录树、记账树的节点数 + inode 叶容器数 + 数据单元数",
         });
     }
     // 已释放的记录合法（D3（空间分配） 已定项 7：改写不删），它的代是释放代，同样不许晚于根。
-    for record in &allocation_records {
+    for record in allocation_records {
         if record.generation > root.checkpoint_txg || record.span_slots == 0 {
             return Err(RecoveryFailure::InvariantViolated {
                 invariant: "E142 走读同款",
@@ -2492,31 +2641,12 @@
         return Ok(None);
     };
 
-    // extent 树：这个文件的记录按 key 升序，第 i 条是文件第 i 个数据单元，key = (locality 0, inode 1, 第 i 个单元第一个字节的文件偏移)
-    // （D8（核心索引结构） 已定项 3：offset 段是文件字节偏移；并行线一一个文件跨多个单元）。条数要等于 inode size 按净荷容量除出来的
-    // 单元数（D4（校验和位置） 已定项 5；与写侧切分同一条除法）——文件没有洞，第一版不写稀疏文件。
+    // extent 树：这个文件的数据指针按单元号排，第 i 个是文件第 i 个数据单元（读 extent 树时已按位置核过：下段叶记录的 key 是
+    // (locality 0, inode 1, 第 i 个单元第一个字节的文件偏移)、单元从 0 起连号，D8（核心索引结构） 已定项 3 / 已定项 14）。
+    // 个数要等于 inode size 按净荷容量除出来的单元数（D4（校验和位置） 已定项 5；与写侧切分同一条除法）——文件没有洞，第一版不写稀疏文件。
     // 解引用先按位置提示、读不到再查映射（D19（块指针的结构与宽度预算） 已定项 5）。
     let payload_capacity_in_bytes = u64::try_from(data_unit_payload_capacity()).expect("32634");
-    let mut records_of_this_file: Vec<([u8; 24], DataPointer)> = Vec::new();
-    for record_bytes in &roots.extent.entries {
-        let (key, pointer) = parse_extent_record(record_bytes).ok_or(
-            RecoveryFailure::EntryNarrowerThanItsFieldTable {
-                what: "extent 叶记录",
-                entry_bytes: record_bytes.len(),
-                field_table_bytes: usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
-            },
-        )?;
-        if key[8..16] == FIRST_INODE_NUMBER.to_le_bytes() {
-            records_of_this_file.push((key, pointer));
-        }
-    }
-    if records_of_this_file.is_empty() {
-        return Err(RecoveryFailure::InvariantViolated {
-            invariant: "E142 走读同款",
-            detail: "extent 树里没有这个文件的记录",
-        });
-    }
-    if u64::try_from(records_of_this_file.len()).expect("记录条数")
+    if u64::try_from(roots.extent.data_pointers.len()).expect("单元数")
         != data_unit_count_of_a_sequential_write(inode_record.size)
     {
         return Err(RecoveryFailure::InvariantViolated {
@@ -2526,19 +2656,10 @@
     }
     let mut content: Vec<u8> =
         Vec::with_capacity(usize::try_from(inode_record.size).expect("文件长度装得进 usize"));
-    // 迭代次数的上界是这个文件的记录条数（上面核过等于单元数）；跨轮携带的只有已经拼出来的内容，每一个提前出口都是交回一个错。
-    for (position, (key, pointer)) in records_of_this_file.iter().enumerate() {
+    // 迭代次数的上界是这个文件的单元数（上面核过等于 inode size 除出来的单元数）；跨轮携带的只有已经拼出来的内容，每一个提前出口都是交回一个错。
+    for (position, pointer) in roots.extent.data_pointers.iter().enumerate() {
         let unit_index_in_file = DataUnitIndexInFile(u64::try_from(position).expect("单元序号"));
         let first_file_byte = unit_index_in_file.first_file_byte(payload_capacity_in_bytes);
-        let mut wanted_key = [0u8; 24];
-        wanted_key[8..16].copy_from_slice(&FIRST_INODE_NUMBER.to_le_bytes());
-        wanted_key[16..24].copy_from_slice(&first_file_byte.0.to_le_bytes());
-        if *key != wanted_key {
-            return Err(RecoveryFailure::InvariantViolated {
-                invariant: "I-1.1",
-                detail: "文件第 i 条 extent 记录的 key 不是 (0, inode, i × 净荷容量)：offset 段不是文件字节偏移，或记录有洞、错位",
-            });
-        }
         let bytes = read_data_unit_via_hint_then_central_mapping(
             reader,
             pointer,
@@ -2554,7 +2675,7 @@
         let header = parse_data_unit(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
             what: "数据单元",
         })?;
-        if header.identity.tree.0 != roots.extent.tree.0
+        if header.identity.tree != roots.extent_tree
             || header.identity.object != FIRST_INODE_NUMBER
             || header.identity.anchor_offset != first_file_byte.0
         {
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/transaction.rs tree/crates/singlefs-core/src/transaction.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/transaction.rs	2026-09-24 18:48:58.241580116 +0000
+++ tree/crates/singlefs-core/src/transaction.rs	2026-09-24 22:04:35.929349323 +0000
@@ -10,9 +10,8 @@
 use std::collections::{BTreeMap, BTreeSet};
 
 use singlefs_format::{
-    ACCOUNTING_ENTRY_BYTES, ALLOCATION_RECORD_BYTES, ALLOCATION_RECORD_KEY_BYTES, EXTENT_KEY_BYTES,
-    EXTENT_LEAF_RECORD_BYTES, INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES, INSTANCE_ROW_BYTES,
-    JOURNAL_NAMED_ENTRIES_PER_RECORD, MAPPING_ENTRY_BYTES, SLOT_BYTES,
+    ACCOUNTING_ENTRY_BYTES, EXTENT_KEY_BYTES, INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES,
+    INSTANCE_ROW_BYTES, JOURNAL_NAMED_ENTRIES_PER_RECORD, MAPPING_ENTRY_BYTES, SLOT_BYTES,
     SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, TREE_IDENTIFIER_ACCOUNTING,
     TREE_IDENTIFIER_ALLOCATION_RECORDS, TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST,
     TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LIVELIST,
@@ -24,6 +23,12 @@
     CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
     SlotNumber, TreeIdentifier,
 };
+use crate::allocation_record_tree::{
+    build_allocation_record_tree_node, children_of, nodes_holding_records,
+    nodes_whose_contents_changed, records_of_each_leaf, AllocationRecordTreeGeometry,
+    AllocationRecordTreeNode, AllocationRecordTreeNodeContents, AllocationRecordTreeNodeOrigin,
+    AllocationRecordTreeNodePosition, AllocationRecordTreePlan, AllocationRecordTreeVersion,
+};
 use crate::allocator::{
     AllocationRecord, Placement, PlacementRefusal, PoolAllocator, UnitFootprint,
 };
@@ -36,6 +41,12 @@
     CodeTwoTreeNodePosition, CodeTwoTreePlan, CodeTwoTreeReadExpectation, CodeTwoTreeRefusal,
     CodeTwoTreeShape, CodeTwoTreeVersion,
 };
+use crate::extent_tree::{
+    build_lower_internal_node, build_lower_leaf, build_upper_internal_node, build_upper_leaf,
+    lower_children_of_a_file_without_holes, lower_segment_nodes_of_a_file_without_holes,
+    upper_path_of_inode, upper_root_level_for, ExtentLowerNodePosition, ExtentTreeNodeIdentity,
+    ExtentTreeVersion, ExtentUpperLeafEntry, ExtentUpperLeafTarget, ExtentUpperNodePosition,
+};
 use crate::inode_tree::{
     write_records_into_leaf_containers, InodeLeafContainer, InodeLeafContainerIndexInTree,
     InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal,
@@ -78,8 +89,8 @@
 };
 use crate::unit::{
     build_data_unit, build_index_node, build_packed_unit, data_unit_payload_capacity,
-    index_node_entry_capacity, parse_index_node, unit_filesystem_identifier, DataUnitIdentity,
-    WriteOrder, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
+    parse_index_node, unit_filesystem_identifier, DataUnitIdentity, WriteOrder, UNIT_CLASS_DATA,
+    UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
 };
 use crate::write_accounting::{WritesByStructureKind, WrittenStructureKind};
 use crate::write_request_split::{
@@ -699,6 +710,8 @@
     /// 末条之前的那几条：写行那次发布点名项多于一条记录装得下时末条再跨记录（D23（journal 的角色与格式） 已定项 17）；
     /// 零单元发布与只有一条记录的写行是空的。
     pub earlier_records_of_this_publish: Vec<WrittenJournalRecord>,
+    /// 这次发布真正写出的角色，按写出的次序（= 点名项的次序：实例表链各片尾片先，再分配记录树重写的节点先叶后根）；零单元发布是空的。
+    pub rewritten: Vec<TransactionUnit>,
     pub writes: WritesByStructureKind,
 }
 
@@ -911,6 +924,41 @@
     previous_root: &RootRecord,
     plan: ZeroUnitPublishPlan,
 ) -> Result<VersionWithoutFilePublishOutput, BlockDeviceError> {
+    let (writes, output) = prepare_the_publish_without_units(pool, previous_root, plan);
+    let writes_before_this_publish = pool.writes_by_structure_kind.clone();
+    // 中途失败时这次已记的写要交出去（增补 2 第 20b 行）：失败在这里记账再把错原样交回。零单元发布不冻结（见 `FrozenPublish`）。
+    if let Err(cause) = persist_publish_writes(pool, &writes) {
+        pool.count_failed_publish(&writes_before_this_publish);
+        return Err(cause);
+    }
+    let VersionWithoutFilePublishOutput {
+        root,
+        record,
+        record_bytes,
+        earlier_records_of_this_publish,
+        rewritten,
+        writes: _nothing_written_before_persisting,
+    } = output;
+    Ok(VersionWithoutFilePublishOutput {
+        root,
+        record,
+        record_bytes,
+        earlier_records_of_this_publish,
+        rewritten,
+        writes: pool
+            .writes_by_structure_kind
+            .since(&writes_before_this_publish),
+    })
+}
+
+/// 零单元发布落盘之前的全部：装好记录与根，交回要交给设备的字节与这一版（写账是空的）。发布路径（[`publish_without_units`]）
+/// 与可写挂载取号之前的预演（`mount`）读的是同一份。
+#[must_use]
+pub fn prepare_the_publish_without_units<Device: BlockDevice>(
+    pool: &PoolWriter<'_, Device>,
+    previous_root: &RootRecord,
+    plan: ZeroUnitPublishPlan,
+) -> (PublishWrites, VersionWithoutFilePublishOutput) {
     let record = JournalRecord {
         instance: plan.instance,
         counter: plan.counter,
@@ -955,21 +1003,17 @@
         journal_tail: plan.counter,
         journal_instance: plan.instance,
     };
-    let writes_before_this_publish = pool.writes_by_structure_kind.clone();
-    // 中途失败时这次已记的写要交出去（增补 2 第 20b 行）：失败在这里记账再把错原样交回。零单元发布不冻结（见 `FrozenPublish`）。
-    if let Err(cause) = persist_publish_writes(pool, &writes) {
-        pool.count_failed_publish(&writes_before_this_publish);
-        return Err(cause);
-    }
-    Ok(VersionWithoutFilePublishOutput {
-        root,
-        record,
-        record_bytes,
-        earlier_records_of_this_publish: Vec::new(),
-        writes: pool
-            .writes_by_structure_kind
-            .since(&writes_before_this_publish),
-    })
+    (
+        writes,
+        VersionWithoutFilePublishOutput {
+            root,
+            record,
+            record_bytes,
+            earlier_records_of_this_publish: Vec::new(),
+            rewritten: Vec::new(),
+            writes: WritesByStructureKind::NOTHING_WRITTEN,
+        },
+    )
 }
 
 /// 写行那次发布的参数，树表 0 条的一版上（D18（块里携带什么信息） 已定项 11「每次可写挂载都写行」）：身份字段同零单元发布，
@@ -989,21 +1033,23 @@
     pub tree_identifier_watermark: u64,
 }
 
-/// 写行那次发布，上一版树表 0 条：**重写实例表链与分配记录树**（链有几片就几个实例表单元，加一个分配记录树节点），
+/// 写行那次发布，上一版树表 0 条：**重写实例表链与分配记录树**（链有几片就几个实例表单元，加分配记录树这次内容变了的节点），
 /// 落盘顺序同别的发布（D16（发布语义） 已定项 7）——单元写 → 屏障 → journal 记录（点名这几个单元）→ 屏障 → 根槽 FUA → 系统配置槽轮换。
 ///
-/// 为什么是这两个单元：树表 0 条 ⇒ 这一版没有记账树，D16（发布语义） 已定项 9 那五样（记账行、记账树节点、映射条目、
-/// 树表单元、树表条目）一样都不写；而 D18（块里携带什么信息） 已定项 11 要求每次可写挂载都写行、写行 COW 重写整条链 ⇒ 实例表自己那一个单元
-/// 必写。**分配记录树是第二个**：写行换下上一版那片实例表，这条释放要有地方记，不然重开之后只能从根记录那两条指针重建账、
-/// 那一片就成了空闲槽，而根环里 txg 更低的候选根还指着它（2026-09-23 用户定案随 C512（树表 0 条的一版上被换下的单元记在哪））。
+/// 为什么是这两样：树表 0 条 ⇒ 这一版没有记账树，D16（发布语义） 已定项 9 那五样（记账行、记账树节点、映射条目、
+/// 树表单元、树表条目）一样都不写；而 D18（块里携带什么信息） 已定项 11 要求每次可写挂载都写行、写行 COW 重写整条链 ⇒ 实例表自己那几个单元
+/// 必写。**分配记录树是第二样**：写行换下上一版那条实例表链，这条释放要有地方记，不然重开之后只能从根记录那两条指针重建账、
+/// 那几片就成了空闲槽，而根环里 txg 更低的候选根还指着它们（2026-09-23 用户定案随 C512（树表 0 条的一版上被换下的单元记在哪））。
 /// 它的根指针住**根记录**新加的那一项，不进树表——进树表 `tree_table_has_no_entries` 当场翻面，
-/// 按 `PreviousVersion::WithoutFile` / `WithFile` 分流的每一处跟着变。
-/// 落点照 D3（空间分配） 已定项 5 从聚簇段 bump、按已定项 10 ⑤ 各自那一档取（实例表各片最前、尾片先，分配记录树节点在后），
+/// 按 `PreviousVersion::WithoutFile` / `WithFile` 分流的每一处跟着变。这棵树照 D8（核心索引结构） 已定项 14 按绝对槽号按位置寻址，
+/// 树号 0（这一版还没登记过任何树），节点豁免映射；重写哪几个节点照带文件那一路走到固定点
+/// （[`settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file`]），没变的节点照抄上一版的。
+/// 落点照 D3（空间分配） 已定项 5 从聚簇段 bump、按已定项 10 ⑤ 各自那一档取（实例表各片最前、尾片先，分配记录树节点在后、先叶后根），
 /// 与这几个角色在别的发布路径上走的是同一条规则。
 ///
-/// 被换下的（上一版的整条实例表链、上一版的分配记录树节点）在同一次发布里释放（释放先于分配，D3（空间分配） 已定项 7）：
-/// 两者都豁免映射（实例表见 D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12；分配记录树的根这一版住根记录），
-/// 实例表链的各片取计划带着的那条旧链（`instance_table_chain_to_release`），分配记录树节点取上一版根记录里那一条指针，
+/// 被换下的（上一版的整条实例表链、上一版那棵分配记录树这次重写的节点）在同一次发布里释放（释放先于分配，D3（空间分配） 已定项 7）：
+/// 两样都豁免映射（实例表见 D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12；这一版的分配记录树根住根记录），
+/// 实例表链的各片取计划带着的那条旧链（`instance_table_chain_to_release`），分配记录树的节点取分配器记着的那一版那棵树的指针，
 /// 三样逐盘核过（`placement_to_release_after_checking_every_device`）。上一版是 mkfs 的第 0 代（分配记录树根指针全零）时只释放实例表。
 ///
 /// 新根照上一版的根，只换 checkpoint_txg、实例代号、回退下界、实例表指针、分配记录树根指针与树 ID 水位（取计划里给的，
@@ -1011,9 +1057,9 @@
 ///
 /// # Errors
 /// 分配器上冻结着一次没重发的发布（`PublishFrozenAfterAWriteFailureIsNotResentYet`）、
-/// 被换下的任一片的三样核不过（`ReleaseTarget*` / `ReleaseSpanMismatch`）、这次之后的分配记录装不进一个节点
-/// （`AllocationRecordsExceedOneNode`）、落点取不到（`PlacementRefused`）、块设备报错。
-/// 前四样在任何写之前返回，盘上逐字节不变；块设备错交回时分配器回到发布之前的样子、这次发布冻结在它上面等原样重发（[`FrozenPublish`]）。
+/// 被换下的任一片的三样核不过（`ReleaseTarget*` / `ReleaseSpanMismatch`）、落点取不到（`PlacementRefused`）、
+/// 分配记录树重写集的固定点没定下来（`AllocationRecordTreeRewriteSetDidNotSettle`）、块设备报错。
+/// 前几样在任何写之前返回，盘上逐字节不变；块设备错交回时分配器回到发布之前的样子、这次发布冻结在它上面等原样重发（[`FrozenPublish`]）。
 ///
 /// # Panics
 /// 计划带着的旧链第 0 片不是上一版根记录指着的那一片：调用方拼行与拼旧链读的是同一条根。
@@ -1024,25 +1070,10 @@
     plan: InstanceTableOnlyPublishPlan<'_>,
 ) -> Result<VersionWithoutFilePublishOutput, PublishError> {
     refuse_while_a_publish_is_frozen(allocator)?;
-    assert_eq!(
-        plan.instance_table.replaced_chain.first(),
-        Some(&previous_root.instance_table),
-        "计划带着的旧链是上一版根记录指着的那一条：调用方拼行与拼旧链读的是同一条根"
-    );
-    let swapped_out = release_check_and_admission_of_a_row_publish_on_a_version_without_file(
-        previous_root,
-        plan.instance_table,
-        allocator,
-    )?;
     // 失败就换回去：取落点会动分配器（bump 指针、位图、记录），中途报错不留半新的池。
     let allocator_before_this_publish = allocator.clone();
-    let built = publish_instance_table_after_the_release_check(
-        pool,
-        allocator,
-        previous_root,
-        plan,
-        swapped_out,
-    );
+    let built =
+        prepare_the_row_publish_on_a_version_without_file(pool, allocator, previous_root, plan);
     let (writes, output) = match built {
         Ok(built) => built,
         Err(refusal) => {
@@ -1064,150 +1095,118 @@
     })
 }
 
-/// 树表 0 条的一版上写行那次发布的释放核验与准入（只查不改）：被换下的整条实例表旧链与上一版那片分配记录树节点逐片核三样
-/// （`placements_to_release_on_a_version_without_file`），这次之后的分配记录装得进一个节点（`version_without_file_row_publish_admission`）。
-/// 发布路径（`publish_instance_table_on_version_without_file`）与可写挂载取号之前的预演（`mount`，D18（块里携带什么信息） 已定项 11
-/// 「可写挂载的顺序」第五个合取：预演里写行那次的释放核验报错，同样判这次不能可写）都调它，两处判的是同一件事、同一个次序。
-///
-/// # Errors
-/// 旧链或分配记录树节点的三样核不过（`ReleaseTarget*` / `ReleaseSpanMismatch`）、分配记录装不下（`AllocationRecordsExceedOneNode`）。
-pub(crate) fn release_check_and_admission_of_a_row_publish_on_a_version_without_file(
-    previous_root: &RootRecord,
-    instance_table: &InstanceTableRewrite,
-    allocator: &PoolAllocator,
-) -> Result<PlacementsReleasedByTheRowPublish, PublishError> {
-    let swapped_out = placements_to_release_on_a_version_without_file(
-        previous_root,
-        &instance_table.replaced_chain,
-        allocator,
-    )?;
-    version_without_file_row_publish_admission(
-        allocator,
-        instance_table.pages_after_this_publish(),
-    )?;
-    Ok(swapped_out)
-}
-
-/// 写行那次发布换下的：上一版的整条实例表链，与上一版那片分配记录树节点
-/// （上一版是 mkfs 的第 0 代时根记录那一项全零、没有这一片）。
-pub(crate) struct PlacementsReleasedByTheRowPublish {
-    /// 旧链每一片的落点，按片序号。
-    instance_table_chain: Vec<Placement>,
-    allocation_record_node: Option<Placement>,
-}
-
-/// 写行那次发布换下的落点：上一版的整条实例表链（`replaced_chain`，计划带着的），加上一版的分配记录树节点
-/// （上一版是 mkfs 的第 0 代时那一条指针全零、没有这一片）。
-/// 只查不改（D19（块指针的结构与宽度预算） 已定项 5 第 1 条），查不齐就一个落点都不释放。
-///
-/// # Errors
-/// 任一条指针的三样（在册、没释放过、跨度对得上）核不过 ⇒ `ReleaseTarget*` / `ReleaseSpanMismatch`。
-fn placements_to_release_on_a_version_without_file(
-    previous_root: &RootRecord,
-    replaced_chain: &[NodePointer],
-    allocator: &PoolAllocator,
-) -> Result<PlacementsReleasedByTheRowPublish, PublishError> {
-    let instance_table_chain = instance_table_chain_to_release(replaced_chain, allocator)?;
-    let allocation_record_node =
-        if previous_root.allocation_record_tree_root == NodePointer::empty_root() {
-            None
-        } else {
-            Some(placement_to_release_after_checking_every_device(
-                TransactionUnit::AllocationTree,
-                &previous_root.allocation_record_tree_root.locations,
-                allocator,
-            )?)
-        };
-    Ok(PlacementsReleasedByTheRowPublish {
-        instance_table_chain,
-        allocation_record_node,
-    })
-}
-
-/// 写行那次发布的准入：这次之后的分配记录条数装不进一个节点就在动分配器之前拒掉，不许走到 `build_index_node` 的断言。
-/// 与 `admission_of_one_publish` 的第一条共用 `refuse_when_the_allocation_records_do_not_fit_one_node`；
-/// 记账与映射那两条在这一格没有对象（树表 0 条 ⇒ 这一版没有记账树、没有映射条目），所以不调那一整道。
-/// 释放只改写记录、不加条数，所以基数是分配器此刻的记录数；这次新增的是重写的那几个角色（实例表链每一片、分配记录树节点）每盘各一条。
-///
-/// # Errors
-/// `AllocationRecordsExceedOneNode`。
-fn version_without_file_row_publish_admission(
-    allocator: &PoolAllocator,
-    instance_table_pages_after_this_publish: usize,
-) -> Result<(), PublishError> {
-    let rewritten_roles = instance_table_pages_after_this_publish + 1;
-    refuse_when_the_allocation_records_do_not_fit_one_node(
-        allocator.records().len() + rewritten_roles * allocator.devices.len(),
-    )
+/// 树表 0 条那一版写行时这一次的样子：分配记录树这次之后的计划、这次重写的角色（实例表各片尾片先，再分配记录树重写的节点，先叶后根）、
+/// 要释放的落点（整条旧链在前，再是分配记录树换下的节点）。
+struct SettledRowPublishOnAVersionWithoutFile {
+    allocation_record_tree: AllocationRecordTreePlan,
+    rewritten_roles: Vec<TransactionUnit>,
+    release: Vec<Placement>,
 }
 
-/// 分配记录树第一版只有一个节点（分裂不做）：这次发布之后装不下就报错，不许走到 `build_index_node` 的断言
-/// （三方代码第一轮攻方腿：第 50 次覆盖写 panic）。两条准入路径共用这一处，两处各抄一份会分叉。
+/// 树表 0 条那一版写行时，分配记录树重写哪几个节点（固定点，同带文件那一路的 [`settle_the_allocation_record_tree`]）：上一版的树是分配器记着的
+/// 那一棵（`PoolAllocator::allocation_record_tree_of_the_version_without_file`；mkfs 的第 0 代没有树，节点全新），换下的节点逐盘核三样
+/// （`placement_to_release_after_checking_every_device`，只查不改）。在分配器的拷贝上走，不动 `allocator`。
 ///
 /// # Errors
-/// `AllocationRecordsExceedOneNode`。
-fn refuse_when_the_allocation_records_do_not_fit_one_node(
-    records_after_this_publish: usize,
-) -> Result<(), PublishError> {
-    let allocation_node_capacity = index_node_entry_capacity(
-        usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10"),
-        usize::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
-    );
-    if records_after_this_publish > allocation_node_capacity {
-        return Err(PublishError::AllocationRecordsExceedOneNode {
-            records: records_after_this_publish,
-            capacity: allocation_node_capacity,
-        });
-    }
-    Ok(())
-}
-
-/// 分配记录树那一片单元（字节表五）：分配器此刻的每一条记录按 (设备, 槽号) 升序装进一个层 0 节点。
-/// 两条发布路径共用这一处——带文件的那一版的 t5 与树表 0 条那一版写行时的那一片，装出来的字节按同一条规则；
-/// 两处各抄一份会分叉，而「重开之后从这一片重建出来的账与发布时那个分配器相同」正压在两处装的是同一个东西上。
-///
-/// `tree` 两条路径不同，这是唯一的差别：带文件的那一版里它是登记在树表里的分配记录树（第一个文件版本那次发的号，mkfs 那条流上是 13）；
-/// 树表 0 条那一版还没登记过任何树（那一版的水位还没被那几个号推过，写 13 会当场判红 I-7.8（树 ID 水位不小于盘上出现过的最大树 ID）），
-/// 那一片**不属于任何树**（树 ID 0）、由根记录独占持有，与树表单元、实例表单元同一个身份（D22（单元原子性怎么合成） 已定项 7 / 已定项 12）。
+/// 旧链或换下的节点三样核不过（`ReleaseTarget*` / `ReleaseSpanMismatch`）、拷贝上取不到落点（`PlacementRefused`）、
+/// 轮数用完（`AllocationRecordTreeRewriteSetDidNotSettle`）。
 ///
 /// # Panics
-/// 分配器一条记录都没有（层 0 节点的 key 区间取不出来）：池里恒有 mkfs 那两个单元的记录，取不出来说明账已经坏了。
-fn build_allocation_record_node(
-    allocator: &PoolAllocator,
-    tree: TreeIdentifier,
+/// 上一版根记录那一项不是全零，而分配器没记着那一版的分配记录树，或记着的那一棵的根不是那一项指的：分配器的这一项由同一进程的写行
+/// 或挂载时整棵读回来记下（`mount`），与根记录说的是同一版。
+fn settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(
+    previous_root: &RootRecord,
+    instance_table: &InstanceTableRewrite,
     txg: CheckpointTxg,
-    filesystem_identifier: &[u8; 16],
-    instance: InstanceGeneration,
-    birth_sequence: BirthSequence,
-) -> Vec<u8> {
-    let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();
-    allocation_records.sort_by_key(AllocationRecord::sort_key);
-    let (first, last) = (
-        allocation_records
-            .first()
-            .expect("池里恒有 mkfs 写在单元区里那两个单元的分配记录")
-            .key_bytes(),
-        allocation_records
-            .last()
-            .expect("池里恒有 mkfs 写在单元区里那两个单元的分配记录")
-            .key_bytes(),
-    );
-    build_index_node(
-        tree,
-        0,
-        usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10"),
-        &first,
-        &last,
-        txg,
-        filesystem_identifier,
-        instance,
-        birth_sequence,
-        u16::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
-        &allocation_records
+    allocator: &PoolAllocator,
+) -> Result<SettledRowPublishOnAVersionWithoutFile, PublishError> {
+    let geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
+    let previous_tree = if previous_root.allocation_record_tree_root == NodePointer::empty_root() {
+        None
+    } else {
+        let tree = allocator
+            .allocation_record_tree_of_the_version_without_file()
+            .expect("根记录那一项不是全零时，分配器记着那一版的分配记录树（同一进程的写行或挂载时读回来的）");
+        assert_eq!(
+            tree.version.root_pointer(),
+            previous_root.allocation_record_tree_root,
+            "分配器记着的那一棵就是根记录那一项指的那一棵"
+        );
+        Some(tree)
+    };
+    let previous_nodes: BTreeSet<AllocationRecordTreeNode> = previous_tree
+        .map(|tree| tree.version.node_set())
+        .unwrap_or_default();
+    let previous_records: &[AllocationRecord] =
+        previous_tree.map_or(&[], |tree| tree.records.as_slice());
+    let chain = instance_table_chain_to_release(&instance_table.replaced_chain, allocator)?;
+    let instance_table_roles =
+        instance_table_page_roles_in_bump_order(instance_table.pages_after_this_publish());
+    let mut rewritten: BTreeSet<AllocationRecordTreeNode> = BTreeSet::new();
+    let mut nodes_after: BTreeSet<AllocationRecordTreeNode> = previous_nodes.clone();
+    // 迭代上界与跨轮携带的同 `settle_the_allocation_record_tree`。
+    for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {
+        let rewritten_among_the_nodes_after: BTreeSet<AllocationRecordTreeNode> =
+            rewritten.intersection(&nodes_after).copied().collect();
+        let allocation_plan = crate::allocation_record_tree::plan_the_tree_after_this_publish(
+            &previous_nodes,
+            &nodes_after,
+            &rewritten_among_the_nodes_after,
+        );
+        let rewritten_by_the_plan: BTreeSet<AllocationRecordTreeNode> =
+            allocation_plan.rewritten_nodes().into_iter().collect();
+        let mut release = chain.clone();
+        for node in &allocation_plan.replaced_previous_nodes {
+            let pointer = previous_tree
+                .and_then(|tree| tree.version.pointer_of(*node))
+                .expect("换下的节点在上一版那棵树里");
+            release.push(placement_to_release_after_checking_every_device(
+                role_of_allocation_record_tree_node(*node),
+                &pointer.locations,
+                allocator,
+            )?);
+        }
+        let rewritten_roles: Vec<TransactionUnit> = instance_table_roles
             .iter()
-            .map(AllocationRecord::to_bytes)
-            .collect::<Vec<_>>(),
-    )
+            .copied()
+            .chain(
+                allocation_plan
+                    .rewritten_nodes()
+                    .into_iter()
+                    .map(role_of_allocation_record_tree_node),
+            )
+            .collect();
+        let mut rehearsal = allocator.clone();
+        for placement in &release {
+            rehearsal.release(*placement, txg);
+        }
+        // 释放本身弄脏的叶先并进重写集再取落点（同 `settle_the_allocation_record_tree`）。
+        let changed_by_the_release =
+            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
+        if !changed_by_the_release.is_subset(&rewritten_by_the_plan) {
+            rewritten.extend(changed_by_the_release);
+            nodes_after.extend(nodes_holding_records(&geometry, rehearsal.records()));
+            continue;
+        }
+        for identity in &rewritten_roles {
+            allocate_placement_for_role(&mut rehearsal, *identity, txg)?;
+        }
+        let changed =
+            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
+        let holding_records = nodes_holding_records(&geometry, rehearsal.records());
+        if holding_records == nodes_after && changed.is_subset(&rewritten_by_the_plan) {
+            return Ok(SettledRowPublishOnAVersionWithoutFile {
+                allocation_record_tree: allocation_plan,
+                rewritten_roles,
+                release,
+            });
+        }
+        rewritten.extend(changed);
+        nodes_after = holding_records;
+    }
+    Err(PublishError::AllocationRecordTreeRewriteSetDidNotSettle {
+        rounds: ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT,
+    })
 }
 
 /// 这次重写出去的实例表链上的一片：角色、单元字节、指着它的指针（第 0 片的进根记录，第 k 片的进第 k − 1 片的链指针记录）。
@@ -1222,7 +1221,7 @@
 /// **尾片先装、先发出生序号**（D3（空间分配） 已定项 10 ⑤，用户 2026-09-24 定案：第 k 片当第 k + 1 片的父，照先叶后根——第 k 片的
 /// 链指针记录要第 k + 1 片的落点、整单元校验和与出生序号，第 k + 1 片装好了它才装得出来）。落点由调用方先按同一个次序取好
 /// （`instance_table_page_roles_in_bump_order`），这里按角色查。交回的各片按链上的次序，第 0 片在前。
-/// 带文件的一版上写行（`publish_admitted`）与树表 0 条的一版上写行（`publish_instance_table_after_the_release_check`）共用这一处：
+/// 带文件的一版上写行（`publish_admitted`）与树表 0 条的一版上写行（`prepare_the_row_publish_on_a_version_without_file`）共用这一处：
 /// 两处各装一份，片的切法与发号次序会分叉。
 ///
 /// # Panics
@@ -1282,16 +1281,27 @@
     pages_tail_first
 }
 
-/// `publish_instance_table_on_version_without_file` 的后半段：释放、取落点、装单元，交回装好、还没落盘的字节与这一版
-/// （写账是空的，落盘由调用方做）。分出来只为把「失败就把分配器换回去」收在一处——中间每一步都可能提前返回，
-/// 散在调用点上就会漏掉某一条路径。
-fn publish_instance_table_after_the_release_check<Device: BlockDevice>(
-    pool: &mut PoolWriter<'_, Device>,
+/// 树表 0 条那一版上写行那次发布，落盘之前的全部：走分配记录树重写集的固定点、释放、取落点、装单元，交回装好、还没落盘的字节与这一版
+/// （写账是空的，落盘由调用方做）。只动 `allocator`：失败时它停在中途，由调用方换回去——发布路径换回进来时那一份，
+/// 可写挂载取号之前的预演拿的本来就是拷贝（`mount`，D18（块里携带什么信息） 已定项 11「可写挂载的顺序」第五个合取：预演里写行那次报错，
+/// 同样判这次不能可写）。两处走的是同一段代码。
+///
+/// # Errors
+/// 旧链或换下的分配记录树节点三样核不过、落点取不到、固定点没定下来（见 [`publish_instance_table_on_version_without_file`]）。
+///
+/// # Panics
+/// 计划带着的旧链第 0 片不是上一版根记录指着的那一片：调用方拼行与拼旧链读的是同一条根。
+pub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(
+    pool: &PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
     previous_root: &RootRecord,
     plan: InstanceTableOnlyPublishPlan<'_>,
-    swapped_out: PlacementsReleasedByTheRowPublish,
 ) -> Result<(PublishWrites, VersionWithoutFilePublishOutput), PublishError> {
+    assert_eq!(
+        plan.instance_table.replaced_chain.first(),
+        Some(&previous_root.instance_table),
+        "计划带着的旧链是上一版根记录指着的那一条：调用方拼行与拼旧链读的是同一条根"
+    );
     let txg = plan.txg;
     let instance = plan.instance;
     let filesystem_identifier = &pool.parameters.filesystem_identifier;
@@ -1300,37 +1310,36 @@
         // 写行那次发布不承载事务（D23（journal 的角色与格式） 已定项 19 ①：空发布与写行的记录写事务号 0）。
         transaction: 0,
     };
-    for released in swapped_out.instance_table_chain.iter().copied() {
+    let settled = settle_the_allocation_record_tree_of_a_row_publish_on_a_version_without_file(
+        previous_root,
+        plan.instance_table,
+        txg,
+        allocator,
+    )?;
+    let previous_tree_pointers = allocator
+        .allocation_record_tree_of_the_version_without_file()
+        .filter(|_| previous_root.allocation_record_tree_root != NodePointer::empty_root())
+        .map(|tree| tree.version.clone());
+    for released in settled.release.iter().copied() {
         allocator.release(released, txg);
     }
-    if let Some(previous_allocation_record_node) = swapped_out.allocation_record_node {
-        allocator.release(previous_allocation_record_node, txg);
-    }
-    // 实例表链的各片最前、尾片先（D3（空间分配） 已定项 10 ⑤），分配记录树那一片在它们之后取：
-    // 这一片自己的分配记录也要进它自己那个节点，所以落点都取完才装节点。
-    let instance_table_roles =
-        instance_table_page_roles_in_bump_order(plan.instance_table.pages_after_this_publish());
-    let mut instance_table_slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
-    for identity in &instance_table_roles {
+    // 实例表链的各片最前、尾片先（D3（空间分配） 已定项 10 ⑤），分配记录树的节点在它们之后按位置先叶后根取：
+    // 这些节点自己的分配记录也要进这棵树，所以落点都取完才装节点。
+    let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
+    for identity in &settled.rewritten_roles {
         let placement = allocate_placement_for_role(allocator, *identity, txg)?;
-        instance_table_slots.insert(*identity, placement.slot);
+        slots.insert(*identity, placement.slot);
     }
-    let allocation_placement =
-        allocate_placement_for_role(allocator, TransactionUnit::AllocationTree, txg)?;
-    // 这一版的分配记录树节点换成了刚取的这一片：下一次发布（再写一次行，或在这一版上发第一个文件版本）要换下它，
-    // 而那时手里只有这个分配器——记的要是上一版那一片（这次刚释放掉的那一片），新写的这一片就永远没人释放，
-    // I-3.1（已分配统计对得上） 在抬 F 之后当场红。
-    allocator.note_allocation_record_node_of_the_version_without_file(allocation_placement);
-    // 两个落点取完、这次不再分配：记下这条根盖掉的根环槽（同 `publish_admitted` 那一处；失败时分配器由调用方整个换回去）。
+    // 落点取完、这次不再分配：记下这条根盖掉的根环槽（同 `publish_admitted` 那一处；失败时分配器由调用方整个换回去）。
     allocator.record_root_written_by_this_process(txg);
-    // 这一次发布的提交内生块（实例表链的各片、分配记录树那一片）都归树 0（这一版还没登记过任何树），在 (txg, 实例) 上连着发出生序号：
-    // 实例表各片按 bump 次序（尾片先）在前，分配记录树那一片最后。
+    // 这一次发布的提交内生块（实例表链的各片、分配记录树的节点）都归树 0（这一版还没登记过任何树），在 (txg, 实例) 上连着发出生序号：
+    // 实例表各片按 bump 次序（尾片先）在前，分配记录树的节点在后（先叶后根）。
     let mut sequences = BirthSequenceAllocator::default();
     let device_identities: Vec<DeviceIdentity> =
         pool.devices.iter().map(|(identity, _)| *identity).collect();
     let instance_table_pages = build_instance_table_chain(
         &plan.instance_table.rows,
-        &|identity| instance_table_slots[&identity],
+        &|identity| slots[&identity],
         txg,
         write_order,
         filesystem_identifier,
@@ -1341,41 +1350,62 @@
         .first()
         .expect("一条链至少一片：0 行也写出空的那一片")
         .pointer;
-    let allocation_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
-    let allocation_unit = build_allocation_record_node(
-        allocator,
+    // 分配记录树：树号 0（这一版还没登记过任何树，写 13 会当场判红 I-7.8（树 ID 水位不小于盘上出现过的最大树 ID）），
+    // 由根记录独占持有，与树表单元、实例表单元同一个身份（D22（单元原子性怎么合成） 已定项 7 / 已定项 12）。
+    let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();
+    allocation_records.sort_by_key(AllocationRecord::sort_key);
+    let allocation_geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
+    let allocation_built = build_allocation_record_tree(
+        &settled.allocation_record_tree,
+        &allocation_geometry,
+        &allocation_records,
+        &|node| {
+            previous_tree_pointers
+                .as_ref()
+                .and_then(|version| version.pointer_of(node))
+        },
         TreeIdentifier(TREE_IDENTIFIER_NONE),
-        txg,
-        filesystem_identifier,
-        instance,
-        allocation_sequence,
+        &MultiLevelTreeBuildContext {
+            txg,
+            filesystem_identifier,
+            instance,
+            device_identities: &device_identities,
+        },
+        &|identity| slots[&identity],
+        &mut sequences,
     );
-    let allocation_locations = pool.location_entries(allocation_placement.slot, &allocation_unit);
-    let allocation_record_tree_root = NodePointer {
-        head: PointerHead {
-            birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
-            birth_txg: txg,
+    let allocation_record_tree_root = allocation_built.version.root_pointer();
+    // 这一版的分配记录树换成了刚装的这一棵：下一次发布（再写一次行，或在这一版上发第一个文件版本）要照抄或换下它的节点，
+    // 而那时手里只有这个分配器——记的要是上一版那一棵（这次刚换下几个节点的那一棵），新写的节点就永远没人释放，
+    // I-3.1（已分配统计对得上） 在抬 F 之后当场红。
+    allocator.note_allocation_record_tree_of_the_version_without_file(
+        crate::allocator::AllocationRecordTreeOfTheVersionWithoutFile {
+            version: allocation_built.version.clone(),
+            records: allocation_records,
         },
-        locations: allocation_locations,
-        instance,
-        birth_sequence: allocation_sequence,
-    };
-    // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的角色各一项，按 bump 次序——实例表链的各片（尾片先），分配记录树。
-    let named_roles: Vec<TransactionUnit> = instance_table_roles
-        .iter()
-        .copied()
-        .chain(std::iter::once(TransactionUnit::AllocationTree))
-        .collect();
+    );
+    // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的角色各一项，按 bump 次序——实例表链的各片（尾片先），分配记录树重写的节点。
+    let named_roles: Vec<TransactionUnit> = settled.rewritten_roles.clone();
     let named_unit_of = |identity: TransactionUnit| -> NamedUnit {
         match identity {
-            TransactionUnit::AllocationTree => NamedUnit {
-                locations: allocation_locations,
-                unit_class: TransactionUnit::AllocationTree.unit_class(),
-                // 树 ID 0：这一版还没登记过任何树，那一片由根记录独占持有（见 `build_allocation_record_node` 的文档注释）。
-                birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
-                birth_txg: txg,
-                key_tail: node_key_tail(instance, allocation_sequence),
-            },
+            TransactionUnit::AllocationTreeNodeBelowTheRoot(_) | TransactionUnit::AllocationTree => {
+                let unit = allocation_built
+                    .rewritten_units
+                    .iter()
+                    .find(|unit| unit.identity == identity)
+                    .expect("点名的分配记录树节点是这次重写的");
+                NamedUnit {
+                    locations: pool.location_entries(unit.slot, &unit.bytes),
+                    unit_class: identity.unit_class(),
+                    // 树 ID 0：这一版还没登记过任何树，这棵树由根记录独占持有。
+                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
+                    birth_txg: txg,
+                    key_tail: node_key_tail(
+                        instance,
+                        allocation_built.birth_sequences_of_rewritten_nodes[&identity],
+                    ),
+                }
+            }
             TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                 let page = instance_table_pages
                     .iter()
@@ -1391,6 +1421,8 @@
                 }
             }
             TransactionUnit::Data(_)
+            | TransactionUnit::ExtentLowerNode(_)
+            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
             | TransactionUnit::ExtentRoot
             | TransactionUnit::InodeLeafContainer(_)
             | TransactionUnit::InodeRoot
@@ -1472,25 +1504,23 @@
         mapping_root: previous_root.mapping_root,
         allocation_record_tree_root,
     };
-    // 单元写按 bump 次序：实例表链的各片（尾片先），分配记录树那一片最后。
-    let units: Vec<PublishedUnit> = instance_table_roles
+    // 单元写按 bump 次序：实例表链的各片（尾片先），分配记录树重写的节点（先叶后根）最后。
+    let units: Vec<PublishedUnit> = settled
+        .rewritten_roles
         .iter()
+        .filter(|identity| identity.is_a_page_of_the_instance_table())
         .map(|identity| {
             let page = instance_table_pages
                 .iter()
                 .find(|page| page.role == *identity)
                 .expect("每个取了落点的实例表角色都装了一片");
             PublishedUnit {
-                slot: instance_table_slots[identity],
+                slot: slots[identity],
                 identity: *identity,
                 bytes: page.bytes.clone(),
             }
         })
-        .chain(std::iter::once(PublishedUnit {
-            slot: allocation_placement.slot,
-            identity: TransactionUnit::AllocationTree,
-            bytes: allocation_unit,
-        }))
+        .chain(allocation_built.rewritten_units.iter().cloned())
         .collect();
     let writes = PublishWrites {
         units,
@@ -1520,6 +1550,7 @@
             record,
             record_bytes,
             earlier_records_of_this_publish: written_records,
+            rewritten: settled.rewritten_roles,
             writes: WritesByStructureKind::NOTHING_WRITTEN,
         },
     ))
@@ -1630,12 +1661,22 @@
     /// 每个单元一个角色，落点、映射 key、点名项与按结构种类记的账都按角色走，共用一个角色就分不开两个单元。
     /// 只有一个单元的文件（第一个事务那一档）就是 [`DataUnitIndexInFile::FIRST`]。
     Data(DataUnitIndexInFile),
+    /// 第一个文件那一棵 extent 树下段里的一个节点（D8（核心索引结构） 已定项 14：下段一个文件一棵、按数据单元号按位置寻址），
+    /// 带它在下段里的位置（层级、同层序号）；下段的根也是这一族（最高那一层的第 0 个）。只有一个数据单元的文件不建下段（内联），没有这一族。
+    ExtentLowerNode(ExtentLowerNodePosition),
+    /// extent 树上段里根之下的一个节点（按 inode 号按位置寻址），带它在上段里的位置。上段只有一层时没有这一族（根兼叶是 `ExtentRoot`）。
+    ExtentUpperNodeBelowTheRoot(ExtentUpperNodePosition),
+    /// extent 树的根：上段的根（树表条目里 extent 树那一条指着它）。
     ExtentRoot,
     /// inode 树的一片叶容器（码 3 打包记录类型 2，一容器 233 条 140 字节记录，D8（核心索引结构） 已定项 6）。
     /// 带的是它在树里的叶序，不是容器号：一棵树可以有好几片，角色按叶序分开
     /// （落点、映射 key、点名项、按结构种类记的账都按角色走，共用一个角色就分不开两片叶）。
     InodeLeafContainer(InodeLeafContainerIndexInTree),
     InodeRoot,
+    /// 分配记录树里根之下的一个节点（D8（核心索引结构） 已定项 14：按绝对槽号按位置寻址），带它的位置（层级、盘、同盘同层序号）。
+    /// 位置由槽号算出、不随版本挪动：上一版同一个位置上的节点照抄进来时角色不变。
+    AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition),
+    /// 分配记录树的根（罩整个 key 空间；带文件的一版里树表条目指着它，树表 0 条的一版里根记录那一项指着它）。
     AllocationTree,
     /// 记账树里根之下的一个节点（D8（核心索引结构） 已定项 11：多层码 2 树）：带它在这一版树里的位置（层级、同层从左数第几个）。
     /// 根兼叶那一档没有这个角色——记账树只有一个节点时它就是 `AccountingTree`。位置是这一版的：上一版的节点照抄进来时
@@ -1808,12 +1849,25 @@
         match self {
             TransactionUnit::Data(DataUnitIndexInFile::FIRST) => "t1".to_string(),
             TransactionUnit::Data(index) => format!("t1+{}", index.0),
+            // 字节表七只登记了 extent 树根兼叶那一档的 t2；下段与上段根之下的节点写成「t2下@层级.序号」「t2上@层级.序号」，
+            // 只出现在报错与用例消息里。
+            TransactionUnit::ExtentLowerNode(position) => {
+                format!("t2下@{}.{}", position.level, position.index)
+            }
+            TransactionUnit::ExtentUpperNodeBelowTheRoot(position) => {
+                format!("t2上@{}.{}", position.level, position.index)
+            }
             TransactionUnit::ExtentRoot => "t2".to_string(),
             TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST) => {
                 "t3".to_string()
             }
             TransactionUnit::InodeLeafContainer(index) => format!("t3+{}", index.0),
             TransactionUnit::InodeRoot => "t4".to_string(),
+            // 分配记录树根之下的节点写成「t5@层级.盘.同盘同层序号」，只出现在报错与用例消息里。
+            TransactionUnit::AllocationTreeNodeBelowTheRoot(position) => format!(
+                "t5@{}.{}.{}",
+                position.level, position.device.0, position.index_in_device
+            ),
             TransactionUnit::AllocationTree => "t5".to_string(),
             // 字节表七只登记了根兼叶那一档的 t6 / t7；根之下的节点写成「t6@层级.同层序号」，只出现在报错与用例消息里。
             TransactionUnit::AccountingTreeNodeBelowTheRoot(position) => {
@@ -1849,9 +1903,12 @@
                 true
             }
             TransactionUnit::Data(_)
+            | TransactionUnit::ExtentLowerNode(_)
+            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
             | TransactionUnit::ExtentRoot
             | TransactionUnit::InodeLeafContainer(_)
             | TransactionUnit::InodeRoot
+            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
             | TransactionUnit::AllocationTree
             | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree
@@ -1868,8 +1925,11 @@
             TransactionUnit::InodeLeafContainer(_)
             | TransactionUnit::InstanceTable
             | TransactionUnit::InstanceTablePageAfterTheFirst(_) => UNIT_CLASS_PACKED,
-            TransactionUnit::ExtentRoot
+            TransactionUnit::ExtentLowerNode(_)
+            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
+            | TransactionUnit::ExtentRoot
             | TransactionUnit::InodeRoot
+            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
             | TransactionUnit::AllocationTree
             | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree
@@ -1884,9 +1944,13 @@
     #[must_use]
     pub const fn tree(self, trees: &FileVersionTreeIdentifiers) -> TreeIdentifier {
         match self {
-            TransactionUnit::Data(_) | TransactionUnit::ExtentRoot => trees.extent,
+            TransactionUnit::Data(_)
+            | TransactionUnit::ExtentLowerNode(_)
+            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
+            | TransactionUnit::ExtentRoot => trees.extent,
             TransactionUnit::InodeLeafContainer(_) | TransactionUnit::InodeRoot => trees.inode,
-            TransactionUnit::AllocationTree => trees.allocation_records,
+            TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
+            | TransactionUnit::AllocationTree => trees.allocation_records,
             TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree => trees.accounting,
             TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
@@ -1910,8 +1974,11 @@
             | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                 PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned)
             }
-            TransactionUnit::ExtentRoot
+            TransactionUnit::ExtentLowerNode(_)
+            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
+            | TransactionUnit::ExtentRoot
             | TransactionUnit::InodeRoot
+            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
             | TransactionUnit::AllocationTree
             | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree
@@ -2067,7 +2134,13 @@
     /// 这一版文件每个数据单元的指针，第 i 项是文件第 i 个单元（extent 根兼叶里的记录按同一次序排）。
     pub data_pointers: Vec<DataPointer>,
     pub mapping_keys: Vec<Vec<u8>>,
+    /// 这一版的全部分配记录，按 (设备, 槽号) 升序（它们分装在分配记录树的各片叶里）。
     pub allocation_records: Vec<AllocationRecord>,
+    /// 这一版分配记录树的每个节点与它的指针（D8（核心索引结构） 已定项 14：按绝对槽号按位置寻址）；节点的字节住 `units` 里分配记录树那一族角色。
+    pub allocation_record_tree: AllocationRecordTreeVersion,
+    /// 这一版 extent 树上段与第一个文件下段的每个节点与它的指针（D8（核心索引结构） 已定项 14：两段按位置寻址）；
+    /// 节点的字节住 `units` 里 extent 树那一族角色。
+    pub extent_tree: ExtentTreeVersion,
     pub accounting_entries: Vec<AccountingEntry>,
     /// 这一版记账树的形状与每个节点的指针（D8（核心索引结构） 已定项 11：多层码 2 树）；节点的字节住 `units` 里记账树那一族角色。
     pub accounting_tree: CodeTwoTreeVersion,
@@ -2125,6 +2198,15 @@
     IntactButAnotherCopyFailed,
 }
 
+/// 按 key 空间定形状的两棵派生树这一版的高（每个都从根节点头现读，[`TransactionOutput::position_addressed_tree_heights_read_from_the_root_node_headers`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct PositionAddressedTreeHeights {
+    pub allocation_record_tree: u64,
+    pub extent_tree_upper_segment: u64,
+    /// 第一个文件的下段；只有一个数据单元（内联）时 0。
+    pub extent_tree_lower_segment_of_the_file: u64,
+}
+
 /// 一版里的一片 inode 叶容器：装了什么、这一版它的指针在哪。字节不放这里，放 `TransactionOutput::units`
 /// 里那个角色的单元（重写的是新装的，照抄的是上一版那一份）。
 #[derive(Clone, Debug, PartialEq, Eq)]
@@ -2186,6 +2268,34 @@
         u64::from(root_header.level) + 1
     }
 
+    /// 按 key 空间定形状的两棵派生树的高，同样从根节点的码 2 头里现读层级 + 1（D8（核心索引结构） 已定项 14 末句：
+    /// 树高照已定项 11「树高 = 根节点头层级 + 1」读；D28（挂载期承诺量） 已定项 4 的 ckpt_cost 读它）：
+    /// 分配记录树读它的根；extent 树读上段的根，与第一个文件下段的根（内联、没有下段时是 0）。
+    ///
+    /// # Panics
+    /// 这一版里那个根单元解不开：它是这个进程装的，或从盘上重建时校验过的（`recovery::rebuild_version`）。
+    #[must_use]
+    pub fn position_addressed_tree_heights_read_from_the_root_node_headers(
+        &self,
+    ) -> PositionAddressedTreeHeights {
+        let height_of = |root_role: TransactionUnit| {
+            let root_header = parse_index_node(&self.unit(root_role).bytes)
+                .expect("根单元是这个进程装的，或重建时解过、核过的");
+            u64::from(root_header.level) + 1
+        };
+        PositionAddressedTreeHeights {
+            allocation_record_tree: height_of(TransactionUnit::AllocationTree),
+            extent_tree_upper_segment: height_of(TransactionUnit::ExtentRoot),
+            extent_tree_lower_segment_of_the_file: self
+                .extent_tree
+                .lower_nodes
+                .last()
+                .map_or(0, |(position, _)| {
+                    height_of(TransactionUnit::ExtentLowerNode(*position))
+                }),
+        }
+    }
+
     /// 这一版一棵多层码 2 树（形状与指针）。
     #[must_use]
     pub fn multi_level_tree(&self, tree: MultiLevelCodeTwoTree) -> &CodeTwoTreeVersion {
@@ -2228,9 +2338,10 @@
     }
 }
 
-/// 第一个文件版本在树表 0 条的那一版上换下的落点：mkfs 那片树表单元，加那一版写行时写下的分配记录树节点
-/// （C512（树表 0 条的一版上被换下的单元记在哪）：它住根记录那一项，重开时由 `mount` 记进分配器）。
-/// 两者都只在「这一角色这次重写、那一片还登记着、还没释放过」时进来。
+/// 第一个文件版本在树表 0 条的那一版上换下的落点：mkfs 那片树表单元，加那一版写行时写下的那棵分配记录树的每个节点
+/// （C512（树表 0 条的一版上被换下的单元记在哪）：它的根住根记录那一项，重开时由 `mount` 整棵读进分配器）——那棵树的树号是 0，
+/// 这一版的分配记录树换成第一个文件版本发出来的号，节点一个都不照抄（D18（块里携带什么信息） 已定项 7 的树 ID 进块头）。
+/// 都只在「这一角色这次重写、那一片还登记着、还没释放过」时进来。
 fn format_time_tree_table_to_release(
     allocator: &PoolAllocator,
     rewritten: &[TransactionUnit],
@@ -2249,11 +2360,19 @@
         }
     }
     if rewritten.contains(&TransactionUnit::AllocationTree) {
-        if let Some(allocation_node) = allocator
-            .allocation_record_node_of_the_version_without_file()
-            .filter(&still_allocated)
-        {
-            released.push(allocation_node);
+        if let Some(tree) = allocator.allocation_record_tree_of_the_version_without_file() {
+            released.extend(
+                tree.version
+                    .nodes
+                    .iter()
+                    .map(|(node, pointer)| Placement {
+                        slot: slot_shared_by_both_location_entries(&pointer.locations).expect(
+                            "树表 0 条那一版的分配记录树：本进程写的两盘同槽，`mount` 记它之前逐节点判过两条位置条目同槽",
+                        ),
+                        span: role_of_allocation_record_tree_node(*node).span_slots(),
+                    })
+                    .filter(&still_allocated),
+            );
         }
     }
     released
@@ -2285,9 +2404,12 @@
     for identity in roles_replaced_via_mapping(previous, roles) {
         let locations = match identity {
             TransactionUnit::Data(_)
+            | TransactionUnit::ExtentLowerNode(_)
+            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
             | TransactionUnit::ExtentRoot
             | TransactionUnit::InodeLeafContainer(_)
             | TransactionUnit::InodeRoot
+            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
             | TransactionUnit::AllocationTree
             | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree => {
@@ -2331,8 +2453,8 @@
 /// `replaced_chain` 是那条旧链每一片的指针，按片序号（第 0 片是上一版根记录里那一条，第 k 片是第 k − 1 片链指针记录里那一条）。
 /// 实例表豁免映射（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12），不经映射、不做释放之前的读盘核；每一片照样按
 /// `placement_to_release_after_checking_every_device` 核三样（两条位置条目同槽、池里每块盘在册、没释放过、跨度对得上）。
-/// 只查不改：带文件的一版上写行（`publish_version`）、树表 0 条的一版上写行（`publish_instance_table_on_version_without_file`）
-/// 与可写挂载取号之前的准入（`mount`）都调它，三处判的是同一件事。
+/// 只查不改：带文件的一版上写行（`publish_version`）与树表 0 条的一版上写行（`publish_instance_table_on_version_without_file`）都调它；
+/// 可写挂载取号之前的预演（`mount`）走的是这两处落盘之前那一段，判的是同一件事。
 ///
 /// # Errors
 /// 某一片的三样核不过 ⇒ `ReleaseTargetLocationsOnDifferentSlots` / `ReleaseTargetNotAllocated` /
@@ -2402,6 +2524,18 @@
         TransactionUnit::Data(index) => {
             index.0 < u64::try_from(previous.data_pointers.len()).expect("单元数")
         }
+        // 按位置寻址的两棵树的节点：上一版有没有那个位置（这次新长出来的节点在上一版里没有落点）。
+        TransactionUnit::ExtentLowerNode(position) => {
+            previous.extent_tree.lower_pointer_of(position).is_some()
+        }
+        TransactionUnit::ExtentUpperNodeBelowTheRoot(position) => {
+            previous.extent_tree.upper_pointer_of(position).is_some()
+                && position.level != previous.extent_tree.upper_root().level
+        }
+        TransactionUnit::AllocationTreeNodeBelowTheRoot(position) => previous
+            .allocation_record_tree
+            .pointer_of(AllocationRecordTreeNode::BelowTheRoot(position))
+            .is_some(),
         // 多层码 2 树的节点按上一版的位置点名（`previous_roles_replaced_by_this_publish` 从计划里的「被换下的节点」取），恒在上一版里。
         TransactionUnit::ExtentRoot
         | TransactionUnit::InodeRoot
@@ -2464,9 +2598,12 @@
         .into_iter()
         .filter(|identity| match identity {
             TransactionUnit::Data(_)
+            | TransactionUnit::ExtentLowerNode(_)
+            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
             | TransactionUnit::ExtentRoot
             | TransactionUnit::InodeLeafContainer(_)
             | TransactionUnit::InodeRoot
+            | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
             | TransactionUnit::AllocationTree
             | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree => true,
@@ -2501,7 +2638,7 @@
 /// 经映射释放的每个单元，映射条目的两条位置项各指池里的一块盘、两条指的不是同一块（D19（块指针的结构与宽度预算） 已定项 5
 /// 「硬规则 1 的读盘核读不出、核出对不上时怎么办」：位置项指向一块不在池里的盘，或两条位置项指同一块盘，当映射条目损坏，
 /// 在任何写之前拒绝，盘上不变；用户 2026-09-25 定）。只读内存里的上一版、不读盘：发布路径在读盘核之前判，
-/// 可写挂载在取号之前的预演里按写行那一次换下的角色判（`mount` 的 `placements_of_the_publishes_after_acquisition_on_a_copy`），
+/// 可写挂载在取号之前的预演里按写行那一次换下的角色判（`mount` 的 `dry_run_of_the_publishes_after_acquisition`），
 /// 两处判的是同一件事。
 ///
 /// # Errors
@@ -2707,7 +2844,9 @@
     }
 }
 
-/// 发布能出的错：分配器给不出落点（哪一个单元、为什么）、三棵单节点树装不下、内容装不下、释放判定路径对不上，或底层块设备错。
+/// 发布能出的错：分配器给不出落点（哪一个单元、为什么）、多层码 2 树的形状算不出来、内容装不下、释放判定路径对不上，或底层块设备错。
+/// 按 key 空间定形状的分配记录树与 extent 树不分裂、没有装不下这一格（D8（核心索引结构） 已定项 14），此前那两个成员
+/// （分配记录装不进一个节点、extent 树要长内部节点）随之去掉。
 #[derive(Debug)]
 pub enum PublishError {
     /// 分配器拒了这个单元的落点，`refusal` 原样带着分配器的原因：每块盘上都没有合政策的落点（容量不够）是 `NoFreeSlotOnAnyDevice`；
@@ -2718,8 +2857,6 @@
         unit: TransactionUnit,
         refusal: PlacementRefusal,
     },
-    /// 分配记录树第一版只有一个节点，这次发布之后的记录数装不下（每次发布每盘加 8 条、释放只改写不删）。
-    AllocationRecordsExceedOneNode { records: usize, capacity: usize },
     /// 记账树或中央映射树这次之后的形状算不出来（`code_two_tree::plan_the_tree_after_this_publish`）：要长到 257 层、
     /// 码 2 头的层级 1 字节写不下（多层之后映射树容量准入剩下的唯一一条，D19（块指针的结构与宽度预算） 已定项 5），
     /// 或上一版（从盘上重建的）树按分隔 key 走不到它自己叶里的一把 key。在动分配器、发任何一个写之前返回，盘上逐字节不变。
@@ -2786,16 +2923,10 @@
     /// 用户给的内容装不进一个数据单元（32768 − 头 − 预留）：`publish_first_file` 与 `publish_overwrite` 按一个数据单元写
     /// （它们的契约），多单元的内容走 `publish_sequential_write`。
     ContentExceedsDataUnit { bytes: usize, capacity: usize },
-    /// 这个文件要的数据单元多于一片 extent 叶装得下的记录数（144，(16384 − 163) ÷ 112）：树就得长出内部节点，
-    /// 而 **extent 树内部节点条目的格式仓里没有条款**——D8（核心索引结构） 已定项 11 只定码 2 节点的通用排法
-    /// （定宽条目、key 打头），不定 extent 内部条目在 key 之后带什么（只带 86 字节子指针是 110，照 inode 树内部条目
-    /// 再带 26 字节身份引用是 136；E157（并行线一两条条款的计数模型） 把两档当旋钮扫、没选，池级 checker 在这一格报
-    /// 「内部条目格式还没有条款」）。第一版不支持：在任何落盘动作之前返回，盘上逐字节不变。
-    /// `data_units` 是切分算出的单元数，`extent_leaf_capacity` 是一片叶装得下的记录数。
-    ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided {
-        data_units: usize,
-        extent_leaf_capacity: usize,
-    },
+    /// 分配记录树这次重写哪几个节点的固定点在 `rounds` 轮里没有定下来（`settle_the_allocation_record_tree`）。产品路径上走不到这里
+    /// （释放不在这次发布里空出槽、多一个角色只把取落点那一串拉长，重写集与节点都单调）；只供测试的复用窗口置 0 开关下单调性推不出来，
+    /// 条款没写那一格怎么办 ⇒ 第一版不支持：在任何落盘动作之前返回，盘上逐字节不变。
+    AllocationRecordTreeRewriteSetDidNotSettle { rounds: usize },
     /// 这次发布要往 inode 树里写的记录，落法要的条款仓里还没定（[`InodeTreeWriteRefusal`] 的三格）：
     /// 在分配器、块设备都还没被碰过的时候返回，盘上逐字节不变。
     InodeTreeWriteRefused(InodeTreeWriteRefusal),
@@ -2989,7 +3120,7 @@
 ///
 /// 映射条目那一条准入要的是**这次之后 inode 树一共几片叶容器**（映射节点每次发布整片重写，照抄的那几片也各占一条），
 /// 形状答不了它——「重写几片」与「一共几片」是两个数。那个数各条路径自己给：发布路径从 `PublishPlan::resolve`
-/// 算出来的树取，取号之前那一串由 `publish_sequence_admission` 的调用方给。
+/// 算出来的树取；取号之前那一串走的是同一条发布路径（`mount` 的 `dry_run_of_the_publishes_after_acquisition`），也从那里取。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct PublishShape {
     /// 这次重写几个数据单元；0 = 这次不写文件内容（数据单元与 extent 树根照抄上一版）。
@@ -3029,8 +3160,8 @@
     /// （D3（空间分配） 已定项 10 ⑤）。实例表单元归树 0，排在全部提交内生块之前（mkfs 也是先实例表后树表；里程碑步 3 的决策点），
     /// 多于一片时尾片先（[`instance_table_page_roles_in_bump_order`]）。
     ///
-    /// ⚠️ **叶容器在这里按 0 起的连号排，不是它们在树里的真实叶序**：形状只答「几个角色」（准入按条数算，
-    /// `publish_sequence_admission` 在取号之前要的就是这个数），答不了「改的是哪几片」——那要看上一版的树长什么样。
+    /// ⚠️ **叶容器在这里按 0 起的连号排，不是它们在树里的真实叶序**：形状只答「几个角色」，
+    /// 答不了「改的是哪几片」——那要看上一版的树长什么样。
     /// 真实的角色清单由 [`PublishPlan::rewritten_roles`] 给。
     #[must_use]
     pub fn rewritten_roles(self) -> Vec<TransactionUnit> {
@@ -3069,6 +3200,11 @@
 #[derive(Clone, Debug)]
 pub struct ResolvedPublish {
     pub inode_tree: InodeLeafContainersAfterThisPublish,
+    /// 这次之后 extent 树长什么样（D8（核心索引结构） 已定项 14 的两段）。
+    pub extent_tree: ExtentTreePlan,
+    /// 这次之后分配记录树长什么样（D8（核心索引结构） 已定项 14：按绝对槽号按位置寻址）。重写哪几个节点由发布路径在分配器的拷贝上
+    /// 走到固定点定下来（`settle_the_allocation_record_tree`），交给 `PublishPlan::resolve` 的就是那一份。
+    pub allocation_record_tree: AllocationRecordTreePlan,
     /// 这次之后记账树长什么样（D8（核心索引结构） 已定项 11 的分裂与收缩）。
     pub accounting_tree: CodeTwoTreePlan,
     /// 这次之后中央映射树长什么样，同上；按下面那一份映射 key 算。
@@ -3108,25 +3244,54 @@
         }
     }
 
-    /// 这次发布换下的上一版角色：这次重写的角色里不属于多层码 2 树的那些（同一个角色上一版那一份被换下），
-    /// 加两棵多层码 2 树里上一版被换下的节点（按上一版的位置点名；这一版照抄进来的节点角色可能换了位置，但它不被换下）。
+    /// 这次发布换下的上一版角色：这次重写的角色里不属于那四棵多节点树（extent 树、分配记录树、记账树、中央映射树）的那些
+    /// （同一个角色上一版那一份被换下），加 extent 树与分配记录树按计划换下的节点、两棵多层码 2 树里上一版被换下的节点
+    /// （都按上一版的位置点名；这一版照抄进来的节点不被换下）。
     /// 释放判定路径与释放之前读盘核校验和读的都是这一张（`placements_to_release_via_mapping`、`copies_failing_the_release_checksum_check`）。
     #[must_use]
     pub fn previous_roles_replaced_by_this_publish(
         &self,
         previous: &TransactionOutput,
     ) -> Vec<TransactionUnit> {
-        // 按 bump 次序：不属于多层码 2 树的角色（树表除外）、记账树换下的节点、中央映射树换下的节点、树表最末——
-        // 与这次重写的角色同一个次序，释放清单与读盘核校验和的次序都照它。
-        let mut replaced: Vec<TransactionUnit> = self
-            .rewritten_roles
-            .iter()
-            .copied()
-            .filter(|identity| {
-                multi_level_tree_of_role(*identity).is_none()
-                    && *identity != TransactionUnit::TreeTable
-            })
-            .collect();
+        // 按 bump 次序：数据单元与实例表、extent 树换下的节点、inode 树、分配记录树换下的节点、记账树换下的节点、
+        // 中央映射树换下的节点、树表最末——与这次重写的角色同一个次序，释放清单与读盘核校验和的次序都照它。
+        let is_a_node_of_a_tree_with_its_own_replacement_list = |identity: TransactionUnit| {
+            multi_level_tree_of_role(identity).is_some()
+                || allocation_record_tree_node_of_role(identity).is_some()
+                || matches!(
+                    identity,
+                    TransactionUnit::ExtentLowerNode(_)
+                        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
+                        | TransactionUnit::ExtentRoot
+                )
+        };
+        let rewritten_before_the_extent_tree =
+            self.rewritten_roles.iter().copied().filter(|identity| {
+                matches!(
+                    identity,
+                    TransactionUnit::Data(_)
+                        | TransactionUnit::InstanceTable
+                        | TransactionUnit::InstanceTablePageAfterTheFirst(_)
+                )
+            });
+        let mut replaced: Vec<TransactionUnit> = rewritten_before_the_extent_tree.collect();
+        replaced.extend(self.extent_tree.replaced_previous_roles.iter().copied());
+        replaced.extend(self.rewritten_roles.iter().copied().filter(|identity| {
+            !is_a_node_of_a_tree_with_its_own_replacement_list(*identity)
+                && !matches!(
+                    identity,
+                    TransactionUnit::Data(_)
+                        | TransactionUnit::InstanceTable
+                        | TransactionUnit::InstanceTablePageAfterTheFirst(_)
+                        | TransactionUnit::TreeTable
+                )
+        }));
+        replaced.extend(
+            self.allocation_record_tree
+                .replaced_previous_nodes
+                .iter()
+                .map(|node| role_of_allocation_record_tree_node(*node)),
+        );
         for tree in [
             MultiLevelCodeTwoTree::Accounting,
             MultiLevelCodeTwoTree::CentralMapping,
@@ -3157,9 +3322,12 @@
             Some(MultiLevelCodeTwoTree::CentralMapping)
         }
         TransactionUnit::Data(_)
+        | TransactionUnit::ExtentLowerNode(_)
+        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
         | TransactionUnit::ExtentRoot
         | TransactionUnit::InodeLeafContainer(_)
         | TransactionUnit::InodeRoot
+        | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
         | TransactionUnit::AllocationTree
         | TransactionUnit::TreeTable
         | TransactionUnit::InstanceTable
@@ -3167,6 +3335,231 @@
     }
 }
 
+/// 分配记录树的一个节点在一版里的角色：根是 `AllocationTree`，根之下按位置（位置不随版本挪动）。
+#[must_use]
+pub const fn role_of_allocation_record_tree_node(
+    node: AllocationRecordTreeNode,
+) -> TransactionUnit {
+    match node {
+        AllocationRecordTreeNode::Root => TransactionUnit::AllocationTree,
+        AllocationRecordTreeNode::BelowTheRoot(position) => {
+            TransactionUnit::AllocationTreeNodeBelowTheRoot(position)
+        }
+    }
+}
+
+/// 一个角色是不是分配记录树的节点、是哪一个；别的角色交回 `None`。
+#[must_use]
+pub const fn allocation_record_tree_node_of_role(
+    identity: TransactionUnit,
+) -> Option<AllocationRecordTreeNode> {
+    match identity {
+        TransactionUnit::AllocationTree => Some(AllocationRecordTreeNode::Root),
+        TransactionUnit::AllocationTreeNodeBelowTheRoot(position) => {
+            Some(AllocationRecordTreeNode::BelowTheRoot(position))
+        }
+        TransactionUnit::Data(_)
+        | TransactionUnit::ExtentLowerNode(_)
+        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
+        | TransactionUnit::ExtentRoot
+        | TransactionUnit::InodeLeafContainer(_)
+        | TransactionUnit::InodeRoot
+        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
+        | TransactionUnit::AccountingTree
+        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
+        | TransactionUnit::MappingTree
+        | TransactionUnit::TreeTable
+        | TransactionUnit::InstanceTable
+        | TransactionUnit::InstanceTablePageAfterTheFirst(_) => None,
+    }
+}
+
+/// extent 树上段一个位置在一版里的角色：这一版上段的根（最高那一层 `upper_root_level` 的第 0 个）是 `ExtentRoot`，别的按位置。
+#[must_use]
+pub fn role_of_extent_upper_node(
+    position: ExtentUpperNodePosition,
+    upper_root_level: u8,
+) -> TransactionUnit {
+    if position.level == upper_root_level {
+        TransactionUnit::ExtentRoot
+    } else {
+        TransactionUnit::ExtentUpperNodeBelowTheRoot(position)
+    }
+}
+
+/// 一版 extent 树里全部节点的角色，按 bump 次序（下段先叶后根，再上段先叶后根、上段根最末，D3（空间分配） 已定项 10 ⑤「树内先叶后根」：
+/// 上段叶条目要下段根这一版的指针，下段排在前面）。
+#[must_use]
+pub fn extent_tree_roles_in_bump_order(version: &ExtentTreeVersion) -> Vec<TransactionUnit> {
+    let lower = version
+        .lower_nodes
+        .iter()
+        .map(|(position, _)| TransactionUnit::ExtentLowerNode(*position));
+    let upper_root_level = version.upper_root().level;
+    let upper = version
+        .upper_nodes
+        .iter()
+        .map(|(position, _)| role_of_extent_upper_node(*position, upper_root_level));
+    lower.chain(upper).collect()
+}
+
+/// 这次发布之后 extent 树一个节点从哪来：照抄上一版同一个位置上的节点，或这次重写。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ExtentTreeNodeOrigin {
+    CarriedFromThePreviousVersion,
+    RewrittenThisPublish,
+}
+
+/// 这次发布之后 extent 树长什么样（第一版只有第一个文件有内容，D8（核心索引结构） 已定项 14 的两段）：
+/// 写文件内容的发布重写第一个文件的整个下段（每个数据单元都是新的，下段每片叶都变）与上段里从它那片叶到根那一串，
+/// 上段别的节点照抄；不写文件内容的发布整棵照抄。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct ExtentTreePlan {
+    /// 第一个文件下段这一版的节点，bump 次序（先叶后根）；只有一个数据单元（内联）时为空。
+    pub lower_nodes: Vec<(ExtentLowerNodePosition, ExtentTreeNodeOrigin)>,
+    /// 上段这一版的节点，bump 次序（先叶后根，根在最末）。
+    pub upper_nodes: Vec<(ExtentUpperNodePosition, ExtentTreeNodeOrigin)>,
+    /// 上一版被这次换下的 extent 树节点的角色（按上一版的位置与根层级取名），按上一版的 bump 次序。
+    pub replaced_previous_roles: Vec<TransactionUnit>,
+}
+
+impl ExtentTreePlan {
+    /// 这一版上段根的层级。
+    ///
+    /// # Panics
+    /// 上段一个节点都没有：规划恒交出第一个文件那片叶到根那一串。
+    #[must_use]
+    pub fn upper_root_level(&self) -> u8 {
+        self.upper_nodes
+            .last()
+            .expect("上段恒有第一个文件那片叶到根那一串")
+            .0
+            .level
+    }
+
+    /// 这次重写的 extent 树角色，按 bump 次序（= extent 树里出生序号的发号次序）。
+    #[must_use]
+    pub fn rewritten_roles(&self) -> Vec<TransactionUnit> {
+        let upper_root_level = self.upper_root_level();
+        self.lower_nodes
+            .iter()
+            .filter(|(_, origin)| *origin == ExtentTreeNodeOrigin::RewrittenThisPublish)
+            .map(|(position, _)| TransactionUnit::ExtentLowerNode(*position))
+            .chain(
+                self.upper_nodes
+                    .iter()
+                    .filter(|(_, origin)| *origin == ExtentTreeNodeOrigin::RewrittenThisPublish)
+                    .map(|(position, _)| role_of_extent_upper_node(*position, upper_root_level)),
+            )
+            .collect()
+    }
+
+    /// 这一版全部 extent 树角色，按 bump 次序。
+    #[must_use]
+    pub fn roles(&self) -> Vec<TransactionUnit> {
+        let upper_root_level = self.upper_root_level();
+        self.lower_nodes
+            .iter()
+            .map(|(position, _)| TransactionUnit::ExtentLowerNode(*position))
+            .chain(
+                self.upper_nodes
+                    .iter()
+                    .map(|(position, _)| role_of_extent_upper_node(*position, upper_root_level)),
+            )
+            .collect()
+    }
+}
+
+/// 这次发布之后的 extent 树。写文件内容的发布：第一个文件有 `data_units` 个单元（从 0 起连号、没有洞）——一个就内联、不建下段，
+/// 两个起建下段（整段重写）；上段从第一个文件那片叶到根那一串重写，上段别的节点照抄。不写文件内容的发布（`data_units` 为 `None`）整棵照抄。
+///
+/// # Panics
+/// 写文件内容的发布没有上一版、却也不是第一个文件版本以外的东西：没有上一版时上段只有这一串，不会走到照抄那一支。
+#[must_use]
+pub fn plan_the_extent_tree_after_this_publish(
+    previous: Option<&ExtentTreeVersion>,
+    data_units_of_the_file_written_this_publish: Option<u64>,
+) -> ExtentTreePlan {
+    let Some(data_units) = data_units_of_the_file_written_this_publish else {
+        let carried = previous.expect("不写文件内容的发布接在带文件的一版之后：extent 树整棵照抄");
+        return ExtentTreePlan {
+            lower_nodes: carried
+                .lower_nodes
+                .iter()
+                .map(|(position, _)| {
+                    (
+                        *position,
+                        ExtentTreeNodeOrigin::CarriedFromThePreviousVersion,
+                    )
+                })
+                .collect(),
+            upper_nodes: carried
+                .upper_nodes
+                .iter()
+                .map(|(position, _)| {
+                    (
+                        *position,
+                        ExtentTreeNodeOrigin::CarriedFromThePreviousVersion,
+                    )
+                })
+                .collect(),
+            replaced_previous_roles: Vec::new(),
+        };
+    };
+    let previous_upper_root_level = previous.map(|version| version.upper_root().level);
+    let upper_root_level =
+        upper_root_level_for(FIRST_INODE_NUMBER).max(previous_upper_root_level.unwrap_or(0));
+    let path_of_the_file: BTreeSet<ExtentUpperNodePosition> =
+        upper_path_of_inode(FIRST_INODE_NUMBER, upper_root_level)
+            .into_iter()
+            .collect();
+    let previous_upper: BTreeSet<ExtentUpperNodePosition> = previous
+        .map(|version| {
+            version
+                .upper_nodes
+                .iter()
+                .map(|(position, _)| *position)
+                .collect()
+        })
+        .unwrap_or_default();
+    let upper_nodes: Vec<(ExtentUpperNodePosition, ExtentTreeNodeOrigin)> = path_of_the_file
+        .union(&previous_upper)
+        .map(|position| {
+            let origin = if path_of_the_file.contains(position) {
+                ExtentTreeNodeOrigin::RewrittenThisPublish
+            } else {
+                ExtentTreeNodeOrigin::CarriedFromThePreviousVersion
+            };
+            (*position, origin)
+        })
+        .collect();
+    let lower_nodes = lower_segment_nodes_of_a_file_without_holes(data_units)
+        .into_iter()
+        .map(|position| (position, ExtentTreeNodeOrigin::RewrittenThisPublish))
+        .collect();
+    // 换下的：上一版第一个文件的整个下段（这次每个数据单元都换了），与上一版上段里这次重写的那几个（按上一版的根层级取名）。
+    let replaced_previous_roles = match (previous, previous_upper_root_level) {
+        (Some(version), Some(previous_root_level)) => version
+            .lower_nodes
+            .iter()
+            .map(|(position, _)| TransactionUnit::ExtentLowerNode(*position))
+            .chain(
+                version
+                    .upper_nodes
+                    .iter()
+                    .filter(|(position, _)| path_of_the_file.contains(position))
+                    .map(|(position, _)| role_of_extent_upper_node(*position, previous_root_level)),
+            )
+            .collect(),
+        (None, _) | (Some(_), None) => Vec::new(),
+    };
+    ExtentTreePlan {
+        lower_nodes,
+        upper_nodes,
+        replaced_previous_roles,
+    }
+}
+
 /// 记账树里不带设备维的三行（D5（快照 / 空间记账机制） 已定项 8）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum PoolWideAccountingRow {
@@ -3309,6 +3702,12 @@
         .expect("照抄进来的角色在上一版里进映射、有一把 key")
 }
 
+/// 按 key 空间定形状的两棵树这次之后的样子（`resolve` 给映射排 key 要它们的节点清单）。
+struct MappedPositionAddressedTrees<'plans> {
+    extent: &'plans ExtentTreePlan,
+    allocation_records: &'plans AllocationRecordTreePlan,
+}
+
 /// 这张计划要写的记账树与中央映射树，在一切落盘动作之前算完：记账树按这一版的记账 key，中央映射树按这一版每个进映射单元的 key。
 struct MultiLevelTreesOfThisPublish {
     accounting_tree: CodeTwoTreePlan,
@@ -3342,10 +3741,12 @@
         records
     }
 
-    /// 把这张计划算成「这次之后 inode 树、记账树、中央映射树各长什么样、这次重写哪些角色」。`trees` 是这一版那八棵树的号
+    /// 把这张计划算成「这次之后 extent 树、inode 树、记账树、中央映射树各长什么样、这次重写哪些角色」。`trees` 是这一版那八棵树的号
     /// （接在上一版之后就是上一版的，第一个文件版本是这次发出来的），新分裂出来的叶容器的出生树取其中的 inode 树。
     /// `device_identities` 是记账行按盘展开的那几块盘（与装记账行时的分配器同一批），`capacities` 是两棵多层码 2 树的节点容量
-    /// （写入口上那个只供测试的开关，产品路径按节点格式算）。
+    /// （写入口上那个只供测试的开关，产品路径按节点格式算）。`allocation_record_tree` 是这次之后分配记录树的样子：它重写哪几个节点
+    /// 要在分配器的拷贝上走到固定点才知道（取落点会改记录），由调用方定下来交进来（`settle_the_allocation_record_tree`），
+    /// 这里按它排角色、发映射 key。
     ///
     /// # Errors
     /// inode 记录的落法要的条款仓里还没定 ⇒ `InodeTreeWriteRefused`（`crate::inode_tree` 的三格）；
@@ -3356,6 +3757,7 @@
         trees: &FileVersionTreeIdentifiers,
         device_identities: &[DeviceIdentity],
         capacities: CodeTwoTreeNodeCapacities,
+        allocation_record_tree: AllocationRecordTreePlan,
     ) -> Result<ResolvedPublish, PublishError> {
         let containers_before = previous
             .map(TransactionOutput::inode_leaf_container_contents)
@@ -3366,8 +3768,14 @@
             self.txg,
             trees.inode,
         )?;
-        let mut rewritten_roles: Vec<TransactionUnit> = self
-            .file_content_transactions()
+        let file_content_transactions = self.file_content_transactions();
+        let extent_tree = plan_the_extent_tree_after_this_publish(
+            previous.map(|version| &version.extent_tree),
+            self.file
+                .as_ref()
+                .map(|_| u64::try_from(file_content_transactions.len()).expect("这次写的单元数")),
+        );
+        let mut rewritten_roles: Vec<TransactionUnit> = file_content_transactions
             .iter()
             .map(|transaction| TransactionUnit::Data(transaction.unit_index_in_file))
             .collect();
@@ -3376,16 +3784,19 @@
                 rewrite.pages_after_this_publish(),
             ));
         }
-        if self.file.is_some() {
-            rewritten_roles.push(TransactionUnit::ExtentRoot);
-        }
+        rewritten_roles.extend(extent_tree.rewritten_roles());
         for index in &inode_tree.rewritten {
             rewritten_roles.push(TransactionUnit::InodeLeafContainer(*index));
         }
         if !inode_tree.rewritten.is_empty() {
             rewritten_roles.push(TransactionUnit::InodeRoot);
         }
-        rewritten_roles.push(TransactionUnit::AllocationTree);
+        rewritten_roles.extend(
+            allocation_record_tree
+                .rewritten_nodes()
+                .into_iter()
+                .map(role_of_allocation_record_tree_node),
+        );
         let MultiLevelTreesOfThisPublish {
             accounting_tree,
             central_mapping_tree,
@@ -3393,6 +3804,10 @@
         } = self.plan_the_multi_level_trees(
             previous,
             trees,
+            &MappedPositionAddressedTrees {
+                extent: &extent_tree,
+                allocation_records: &allocation_record_tree,
+            },
             &inode_tree,
             device_identities,
             capacities,
@@ -3411,6 +3826,8 @@
         rewritten_roles.push(TransactionUnit::TreeTable);
         Ok(ResolvedPublish {
             inode_tree,
+            extent_tree,
+            allocation_record_tree,
             accounting_tree,
             central_mapping_tree,
             mapped_units,
@@ -3423,6 +3840,7 @@
         &self,
         previous: Option<&TransactionOutput>,
         trees: &FileVersionTreeIdentifiers,
+        position_addressed_trees: &MappedPositionAddressedTrees<'_>,
         inode_tree: &InodeLeafContainersAfterThisPublish,
         device_identities: &[DeviceIdentity],
         capacities: CodeTwoTreeNodeCapacities,
@@ -3459,7 +3877,7 @@
         .map_err(refused(MultiLevelCodeTwoTree::Accounting))?;
 
         // 这一版每个进映射的单元的 key：重写的现算，照抄的取上一版的。出生序号在各自那棵树里按 bump 次序从 0 发（数据单元不发，
-        // 码 1 的 key 带写序）：extent 根是 extent 树这次唯一的码 2 单元、分配记录树一个节点，inode 树先叶后根。
+        // 码 1 的 key 带写序）：extent 树下段先叶后根、再上段先叶后根，inode 树先叶后根，分配记录树按位置先叶后根。
         let node_key = |unit_class: u8, tree: TreeIdentifier, birth_sequence: usize| {
             mapping_key_of_a_node_born_in_this_publish(
                 unit_class,
@@ -3495,10 +3913,6 @@
                         ),
                     ));
                 }
-                mapped_units.push((
-                    TransactionUnit::ExtentRoot,
-                    node_key(UNIT_CLASS_INDEX_NODE, trees.extent, 0),
-                ));
             }
             None => {
                 let carried_version =
@@ -3509,10 +3923,47 @@
                     ));
                     mapped_units.push((identity, carried(identity)));
                 }
-                mapped_units.push((
-                    TransactionUnit::ExtentRoot,
-                    carried(TransactionUnit::ExtentRoot),
-                ));
+            }
+        }
+        // extent 树的每个节点（下段与上段）都进映射（码 2 节点，D19（块指针的结构与宽度预算） 已定项 8）：重写的按 bump 次序发出生序号，
+        // 照抄的取上一版那个角色的 key（上段的根层级这次没变时角色名照旧；变了的只有这次重写的那一串）。
+        let extent_plan = position_addressed_trees.extent;
+        let previous_upper_root_level =
+            previous.map(|previous_version| previous_version.extent_tree.upper_root().level);
+        let mut extent_birth_sequence = 0;
+        let upper_root_level = extent_plan.upper_root_level();
+        let extent_nodes = extent_plan
+            .lower_nodes
+            .iter()
+            .map(|(position, origin)| {
+                (
+                    TransactionUnit::ExtentLowerNode(*position),
+                    TransactionUnit::ExtentLowerNode(*position),
+                    *origin,
+                )
+            })
+            .chain(extent_plan.upper_nodes.iter().map(|(position, origin)| {
+                (
+                    role_of_extent_upper_node(*position, upper_root_level),
+                    role_of_extent_upper_node(
+                        *position,
+                        previous_upper_root_level.unwrap_or(upper_root_level),
+                    ),
+                    *origin,
+                )
+            }));
+        for (identity, identity_in_the_previous_version, origin) in extent_nodes {
+            match origin {
+                ExtentTreeNodeOrigin::RewrittenThisPublish => {
+                    mapped_units.push((
+                        identity,
+                        node_key(UNIT_CLASS_INDEX_NODE, trees.extent, extent_birth_sequence),
+                    ));
+                    extent_birth_sequence += 1;
+                }
+                ExtentTreeNodeOrigin::CarriedFromThePreviousVersion => {
+                    mapped_units.push((identity, carried(identity_in_the_previous_version)));
+                }
             }
         }
         let mut inode_tree_birth_sequence = 0;
@@ -3544,10 +3995,27 @@
                 ),
             )
         });
-        mapped_units.push((
-            TransactionUnit::AllocationTree,
-            node_key(UNIT_CLASS_INDEX_NODE, trees.allocation_records, 0),
-        ));
+        // 分配记录树的每个节点都进映射：重写的按位置先叶后根发出生序号，照抄的取上一版同一个位置那个角色的 key（位置不随版本挪动）。
+        let mut allocation_birth_sequence = 0;
+        for (node, origin) in &position_addressed_trees.allocation_records.nodes {
+            let identity = role_of_allocation_record_tree_node(*node);
+            match origin {
+                AllocationRecordTreeNodeOrigin::RewrittenThisPublish => {
+                    mapped_units.push((
+                        identity,
+                        node_key(
+                            UNIT_CLASS_INDEX_NODE,
+                            trees.allocation_records,
+                            allocation_birth_sequence,
+                        ),
+                    ));
+                    allocation_birth_sequence += 1;
+                }
+                AllocationRecordTreeNodeOrigin::CarriedFromThePreviousVersion => {
+                    mapped_units.push((identity, carried(identity)));
+                }
+            }
+        }
         let previous_accounting_shape = previous_shape_of(MultiLevelCodeTwoTree::Accounting);
         let mut accounting_birth_sequence = 0;
         for (node, origin) in accounting_tree
@@ -3931,7 +4399,7 @@
     publish_version_of_trees(pool, allocator, plan, previous, trees)
 }
 
-/// 发布一版：先做准入（extent 树装得下这个文件的数据单元，再加 `publish_admission` 的三条），
+/// 发布一版：先做准入（extent 树装得下这个文件的数据单元），
 /// 再释放上一版被换下的角色的落点、分配、装单元、
 /// 按 D16（发布语义） 已定项 7 的持久顺序落盘。准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立，
 /// 释放与分配都不算数（第二轮攻方腿：落点被拒（当时叫 `NoSpaceFor`）在释放之后、分配到一半返回，留下半新的池，拿同一个上一版重试撞断言）；
@@ -3942,9 +4410,8 @@
 /// （回退到树表 0 条的一版之后）走 `publish_first_file`，它按要建在上面的那一版的水位发。
 ///
 /// # Errors
-/// `InodeTreeWriteRefused`、`ContentExceedsDataUnit`、
-/// `AllocationRecordsExceedOneNode`、`AccountingEntriesExceedOneNode`、`MappingEntriesExceedOneNode`、
-/// 释放判定路径的四种错、`PlacementRefused`、块设备错。
+/// `InodeTreeWriteRefused`、`ContentExceedsDataUnit`、`MultiLevelCodeTwoTreeRefused`、
+/// `AllocationRecordTreeRewriteSetDidNotSettle`、释放判定路径的四种错、`PlacementRefused`、块设备错。
 ///
 /// # Panics
 /// 没有上一版、而计划写的新水位不是 mkfs 那条流第一次发布之后的 19：调用方把一个水位早已推高的池当成了 mkfs 那条流，
@@ -3981,99 +4448,15 @@
 ) -> Result<TransactionOutput, PublishError> {
     // 第一道：冻结着一次没重发的发布就在任何读写之前拒（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）。
     refuse_while_a_publish_is_frozen(allocator)?;
-    // 先算这次之后 inode 树、记账树、中央映射树各是什么样、这次重写哪些角色：只读，条款没写的那几格与多层码 2 树的拒绝
-    // 在这里交回，盘上逐字节不变。记账行按分配器里的那几块盘展开（装记账行时读的是同一批）。
-    let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator
-        .devices
-        .iter()
-        .map(|device_map| device_map.device)
-        .collect();
-    let resolved = plan.resolve(
-        previous,
-        &trees,
-        &device_identities_of_the_accounting_rows,
-        pool.code_two_tree_node_capacities(),
-    )?;
-    let rewritten = resolved.rewritten_roles.clone();
-    // extent 树第一版只有一个节点（根兼叶）：这次写的数据单元一个一条叶记录，多于一片叶装得下的就要长出内部节点，
-    // 而 extent 内部条目的格式没有条款 ⇒ 在动分配器之前拒掉（`ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided`）。
-    // 读的是同一张角色清单：清单里有几个数据单元，装 extent 叶时就装几条记录（`publish_admitted`）。
-    let rewritten_data_units = rewritten
-        .iter()
-        .filter(|identity| matches!(identity, TransactionUnit::Data(_)))
-        .count();
-    let extent_leaf_capacity = index_node_entry_capacity(
-        usize::try_from(EXTENT_KEY_BYTES).expect("24"),
-        usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
-    );
-    if rewritten_data_units > extent_leaf_capacity {
-        return Err(
-            PublishError::ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided {
-                data_units: rewritten_data_units,
-                extent_leaf_capacity,
-            },
-        );
-    }
-    // 点名项一条记录装 67 个（D23（journal 的角色与格式） 已定项 12 / 已定项 17）：最后一个事务要点名的项装不下一条记录时
-    // 末条再跨记录（`roles_named_by_each_record_of_the_publish`），这里不拒。
-    // 释放判定路径先于准入：它只查不改（D19（块指针的结构与宽度预算） 已定项 5 第 1 条），查不到就整次发布不做。
-    // 换下的是上一版的角色：多层码 2 树的节点按计划里「上一版被换下的节点」点名，别的角色同一个角色上一版那一份。
-    let previous_roles_replaced = match previous {
-        Some(previous_version) => {
-            resolved.previous_roles_replaced_by_this_publish(previous_version)
-        }
-        None => Vec::new(),
-    };
-    let release = match previous {
-        Some(previous_version) => placements_to_release_via_mapping(
-            previous_version,
-            allocator,
-            &previous_roles_replaced,
-        )?,
-        // 第一个文件版本没有上一版的内存态：它重写树表时换下的是 mkfs 那片第 0 版树表单元，照样进 defer 队列
-        // （D3（空间分配） 已定项 7；不释放它，txg 0 的根离开候选集之后这一槽就永远占着，I-3.1（已分配统计对得上） 在抬 F 之后红）。
-        None => format_time_tree_table_to_release(allocator, &rewritten),
-    };
-    // 整条实例表链重写（写行、回退那一次）：被换下的那条旧链逐片释放，指针取计划带着的那条链（第 1 片起的不在上一版的内存态里）。
-    // 旧链排在别的角色前面，与实例表在 bump 次序里排最前同一个次序。
-    let release = match &plan.instance_table {
-        InstanceTablePlan::Rewrite(rewrite) => {
-            if let Some(previous_version) = previous {
-                assert_eq!(
-                    rewrite.replaced_chain.first(),
-                    Some(&previous_version.root.instance_table),
-                    "计划带着的旧链是上一版根记录指着的那一条：调用方拼行与拼旧链读的是同一条根"
-                );
-            }
-            let mut chain_then_the_rest =
-                instance_table_chain_to_release(&rewrite.replaced_chain, allocator)?;
-            chain_then_the_rest.extend(release);
-            chain_then_the_rest
-        }
-        InstanceTablePlan::Carry(_) => release,
-    };
-    // 经映射核到的那几个单元，释放之前再按位置项读盘核一次校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）：只读，
-    // 读不到就在动分配器、发任何一个写之前返回。第一个文件版本换下的 mkfs 那片树表与树表 0 条那一版的分配记录树节点不经映射，不核。
-    let quarantine = match previous {
-        Some(previous_version) => copies_failing_the_release_checksum_check(
-            previous_version,
-            &previous_roles_replaced,
-            &*pool.devices,
-        )?,
-        None => Vec::new(),
-    };
-    publish_admission(allocator, &rewritten)?;
     // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。
     let allocator_before_this_publish = allocator.clone();
-    let built = publish_admitted(
+    let built = prepare_the_version_publish(
         pool,
         allocator,
         &plan,
         previous,
-        &resolved,
-        &release,
-        &quarantine,
         trees,
+        ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt,
     );
     let (writes, output) = match built {
         Ok(built) => built,
@@ -4097,421 +4480,233 @@
     })
 }
 
-/// 一次发布的准入里与这次写什么内容无关的那一条：分配记录树第一版只有一个节点（按 key 空间定形状的结构在三方 `m2-keyspace-r1` 判，
-/// 这一轮不动它），这次发布之后要装得下——这次重写的每个角色每盘各加一条，两棵多层码 2 树分裂多出来的节点也各算一个角色。
-/// 记账树与中央映射树装不下一个节点时照 D8（核心索引结构） 已定项 11 分裂，不在这里拒；它们的形状在 `PublishPlan::resolve` 里算，
-/// 算不出来（长到 257 层）在那里交回（`MultiLevelCodeTwoTreeRefused`）。
-/// 只读——不动分配器、不发一个写，算不过时盘上逐字节不变。两处调它：发布路径在动分配器之前（`publish_version`）；
-/// 可写挂载在**取号之前**按这次挂载要发的那几次（写行 + 暖机）算一遍（`publish_sequence_admission`，增补 2 第 20a 行：
-/// 算不过就不许先把实例代号烧掉——取号是两次系统配置槽写加一道屏障，之后再拒绝，池此后每试一次可写挂载就多烧一个代号）。
-/// 两处读的是同一个内存里的分配器，取号不碰它，所以两次必定同答案；发布路径那一遍仍留着，它是动分配器之前的最后一道。
-///
-/// # Errors
-/// `AllocationRecordsExceedOneNode`（这次之后的分配记录条数越过一个节点）。
-pub fn publish_admission(
-    allocator: &PoolAllocator,
-    rewritten: &[TransactionUnit],
-) -> Result<(), PublishError> {
-    admission_of_one_publish(allocator, allocator.records().len(), rewritten.len())
+/// 释放之前读不读盘核被换下的每一份的校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ReleaseChecksumCheck {
+    /// 发布路径：读盘核，核出对不上（读不出也算）的单元在每块盘上的分配记录留在已分配。
+    ReadEveryReplacedCopyBeforeReleasingIt,
+    /// 可写挂载取号之前的预演（`mount`）：不读盘、照常释放——预演只要这一串取落点的次序与结局；
+    /// 两边只在「有一份核出对不上、那一槽又在这一串里被回收」时分叉，那一格见 `mount::establish_instance` 的注释。
+    SkippedByTheDryRunBeforeAcquisition,
 }
 
-/// 接连几次不写文件内容、不碰 inode 树的发布（写行 + 暖机）的准入，在第一次动分配器之前一次算完：每一次重写的角色 =
-/// 形状给的那几个不属于多层码 2 树的角色 + 这一次记账树与中央映射树要重写的节点，前面几次要新增的分配记录算进后面几次的基数。
-/// 基数只加不减，理由与单次那条同一句——释放只改写记录、不加，回收要 F 抬到释放代之上（`reclaim_released_up_to`），
-/// 而这一串（写行 + 暖机）里不抬 F。
-///
-/// 两棵多层码 2 树每一次重写几个节点按上一次之后的形状现算，与发布路径同一个 `code_two_tree::plan_the_tree_after_this_publish`
-/// （「一次发布最多分裂几次」由它算出，不另立上界）：这一串里每一次换掉的映射 key 只有分配记录树与记账树节点的那几把
-/// （写行与暖机不碰文件、不碰 inode 树），新 key 的出生 txg 比上一版里每一把都大、一次比一次大。key 逐字段比、类标签与出生树之后
-/// 先比出生 txg（D8（核心索引结构） 已定项 11），它们在中央映射树里落在哪、树怎么分裂只看这个次序，不看 txg 与实例代号的具体值——
-/// 所以取号之前拿「上一版的 txg + 1、+ 2 …」与上一版的实例代号代进去算出的形状，与取号之后真发布时算出的逐节点相同。
-/// 记账树的 key 只看统计量、树、盘与代，每次发布整批换代，形状只看行数与容量。`capacities` 与取号之后那几次发布的写入口上装的相同。
+/// 带文件的一版的一次发布，落盘之前的全部：算这次之后每棵树的样子（分配记录树重写哪几个节点走到固定点，
+/// [`settle_the_allocation_record_tree`]）、释放判定、释放之前读盘核（`release_checksum_check` 说核不核）、释放、取落点、装单元，
+/// 交回装好、还没落盘的字节与这一版（写账是空的）。只动 `allocator`：失败时它停在中途，由调用方换回去——发布路径换回进来时那一份，
+/// 可写挂载取号之前的预演拿的本来就是拷贝。两处走的是同一段代码，预演取到的落点因此就是真发时取到的（`mount::establish_instance` 断言）。
 ///
 /// # Errors
-/// `PublishSequenceRefusal`：第几次算不过（从 0 数），连它的 `PublishError` 一起交回。
-///
-/// # Panics
-/// 传进来的形状里有一次重写叶容器或写文件内容：那时换掉的映射 key 不止上面那几把，这里的推算罩不住。
-pub fn publish_sequence_admission(
-    allocator: &PoolAllocator,
-    shapes: &[PublishShape],
-    previous: &TransactionOutput,
-    capacities: CodeTwoTreeNodeCapacities,
-) -> Result<(), PublishSequenceRefusal> {
-    let device_identities: Vec<DeviceIdentity> = allocator
+/// `InodeTreeWriteRefused`、`MultiLevelCodeTwoTreeRefused`、释放判定路径的几种错、`PlacementRefused`。
+pub fn prepare_the_version_publish<Device: BlockDevice>(
+    pool: &PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    plan: &PublishPlan<'_>,
+    previous: Option<&TransactionOutput>,
+    trees: FileVersionTreeIdentifiers,
+    release_checksum_check: ReleaseChecksumCheck,
+) -> Result<(PublishWrites, TransactionOutput), PublishError> {
+    // 先算这次之后每棵树是什么样、这次重写哪些角色、换下哪些：只读（固定点在分配器的拷贝上走），条款没写的那几格与多层码 2 树的拒绝、
+    // 释放判定的错、取不到落点都在这里交回，盘上逐字节不变。记账行按分配器里的那几块盘展开（装记账行时读的是同一批）。
+    let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator
         .devices
         .iter()
         .map(|device_map| device_map.device)
         .collect();
-    let rewritten_role_counts = rewritten_role_counts_of_a_publish_sequence(
-        &device_identities,
-        shapes,
+    let settled = settle_the_allocation_record_tree(
+        plan,
         previous,
-        capacities,
+        &trees,
+        &device_identities_of_the_accounting_rows,
+        pool.code_two_tree_node_capacities(),
+        allocator,
     )?;
-    let mut records_before_this_publish = allocator.records().len();
-    for (publish_index, rewritten_role_count) in rewritten_role_counts.into_iter().enumerate() {
-        admission_of_one_publish(allocator, records_before_this_publish, rewritten_role_count)
-            .map_err(|cause| PublishSequenceRefusal {
-                publish_index,
-                cause,
-            })?;
-        records_before_this_publish += rewritten_role_count * allocator.devices.len();
-    }
-    Ok(())
-}
-
-/// 接连几次不写文件内容、不碰 inode 树的发布（写行 + 暖机）每一次重写几个角色：形状给的那几个不属于多层码 2 树的角色，
-/// 加这一次记账树与中央映射树要重写的节点（推法见 [`publish_sequence_admission`]）。`device_identities` 是记账行按盘展开的那几块盘。
-///
-/// # Errors
-/// 某一次两棵多层码 2 树的形状算不出来 ⇒ `PublishSequenceRefusal`（第几次、`MultiLevelCodeTwoTreeRefused`）。
-///
-/// # Panics
-/// 传进来的形状里有一次重写叶容器或写文件内容：那时换掉的映射 key 不止分配记录树与记账树的那几把，这里的推算罩不住。
-pub fn rewritten_role_counts_of_a_publish_sequence(
-    device_identities: &[DeviceIdentity],
-    shapes: &[PublishShape],
-    previous: &TransactionOutput,
-    capacities: CodeTwoTreeNodeCapacities,
-) -> Result<Vec<usize>, PublishSequenceRefusal> {
-    let txgs: Vec<CheckpointTxg> = (1..=u64::try_from(shapes.len()).expect("这一串的次数"))
-        .map(|offset| CheckpointTxg(previous.root.checkpoint_txg.0 + offset))
-        .collect();
-    let tree_nodes = multi_level_tree_nodes_of_a_publish_sequence_without_content(
+    // 经映射核到的那几个单元，释放之前再按位置项读盘核一次校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）：只读，
+    // 读不到就在动分配器、发任何一个写之前返回。第一个文件版本换下的 mkfs 那片树表与树表 0 条那一版的分配记录树节点不经映射，不核。
+    let quarantine = match (previous, release_checksum_check) {
+        (Some(previous_version), ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt) => {
+            copies_failing_the_release_checksum_check(
+                previous_version,
+                &settled.previous_roles_replaced,
+                &*pool.devices,
+            )?
+        }
+        (Some(previous_version), ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition) => {
+            // 预演不读盘，但映射条目的位置项指池外的盘、两条指同一块盘那一判不读盘，照样判（发布路径在读盘核之前判的同一件事）。
+            refuse_mapping_entries_that_do_not_name_two_pool_devices(
+                previous_version,
+                &settled.previous_roles_replaced,
+                &device_identities_of_the_accounting_rows,
+            )?;
+            Vec::new()
+        }
+        (None, _) => Vec::new(),
+    };
+    publish_admitted(
+        pool,
+        allocator,
+        plan,
         previous,
-        shapes,
-        &txgs,
-        previous.root.instance,
-        device_identities,
-        capacities,
-    )?;
-    Ok(shapes
-        .iter()
-        .zip(&tree_nodes)
-        .map(|(shape, nodes)| {
-            rewritten_roles_of_a_publish_without_content(*shape, &nodes.rewritten_roles).len()
-        })
-        .collect())
-}
-
-/// 一次不写文件内容、不碰 inode 树的发布重写的全部角色，按 bump 次序：形状给的那几个不属于多层码 2 树的角色
-/// （写行的实例表链各片最前、尾片先，分配记录树、树表最末），两棵多层码 2 树这一次重写的节点插在树表之前（D3（空间分配） 已定项 10 ⑤：
-/// 记账树（树 14）、中央映射树倒数第二、树表最末）——与发布路径 `PublishPlan::resolve` 排出来的逐项相同。
-#[must_use]
-pub fn rewritten_roles_of_a_publish_without_content(
-    shape: PublishShape,
-    rewritten_tree_node_roles: &[TransactionUnit],
-) -> Vec<TransactionUnit> {
-    let mut roles: Vec<TransactionUnit> = shape
-        .rewritten_roles()
-        .into_iter()
-        .filter(|identity| {
-            multi_level_tree_of_role(*identity).is_none() && *identity != TransactionUnit::TreeTable
-        })
-        .collect();
-    roles.extend_from_slice(rewritten_tree_node_roles);
-    roles.push(TransactionUnit::TreeTable);
-    roles
+        &settled.resolved,
+        &settled.release,
+        &quarantine,
+        trees,
+    )
 }
 
-/// 取号之前推一串不写文件内容的发布（写行 + 暖机）时，一次发布里两棵多层码 2 树重写哪些节点、换下哪些节点。
-#[derive(Clone, Debug, PartialEq, Eq)]
-pub struct MultiLevelTreeNodesOfAPublishWithoutContent {
-    /// 这一次两棵树重写的节点角色，按 bump 次序（记账树先叶后根，再中央映射树先叶后根）。
-    pub rewritten_roles: Vec<TransactionUnit>,
-    /// 这一次换下的节点（上一次之后那两棵树里没被照抄进这一次的）：各自是哪一次写出来的。
-    pub replaced: Vec<TreeNodeWrittenBy>,
+/// 这次发布算定了的样子：每棵树这次之后的形状与重写的角色、这次换下的上一版角色、要释放的落点（按 bump 次序）。
+struct SettledPublish {
+    resolved: ResolvedPublish,
+    previous_roles_replaced: Vec<TransactionUnit>,
+    release: Vec<Placement>,
 }
 
-/// 一个多层码 2 树节点是哪一次发布写出来的：这一串之前那一版（按它在那一版里的角色——释放时经那一版的映射或父指针查落点），
-/// 或这一串里第几次（从 0 数，按它在那一次里的角色——释放时取那一次为这个角色取到的落点）。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub enum TreeNodeWrittenBy {
-    TheVersionBeforeTheSequence(TransactionUnit),
-    PublishOfTheSequence {
-        publish_index: usize,
-        role: TransactionUnit,
-    },
+/// 这次发布换下的上一版角色与要释放的落点（释放判定路径，只查不改，D19（块指针的结构与宽度预算） 已定项 5 第 1 条）：
+/// 整条实例表旧链排最前，再是经映射查到的那几个（第一个文件版本没有上一版的内存态：换下的是 mkfs 那片树表与树表 0 条那一版的分配记录树）。
+///
+/// # Errors
+/// 释放判定路径的几种错（`ReleaseNotInMapping` 一族、`ReleaseTarget*`、`ReleaseSpanMismatch`）。
+fn placements_released_by_this_publish(
+    plan: &PublishPlan<'_>,
+    previous: Option<&TransactionOutput>,
+    resolved: &ResolvedPublish,
+    allocator: &PoolAllocator,
+) -> Result<(Vec<TransactionUnit>, Vec<Placement>), PublishError> {
+    // 换下的是上一版的角色：多层码 2 树与按位置寻址的两棵树的节点按计划里「上一版被换下的节点」点名，别的角色同一个角色上一版那一份。
+    let previous_roles_replaced = match previous {
+        Some(previous_version) => {
+            resolved.previous_roles_replaced_by_this_publish(previous_version)
+        }
+        None => Vec::new(),
+    };
+    let release = match previous {
+        Some(previous_version) => placements_to_release_via_mapping(
+            previous_version,
+            allocator,
+            &previous_roles_replaced,
+        )?,
+        // 第一个文件版本没有上一版的内存态：它重写树表时换下的是 mkfs 那片第 0 版树表单元，照样进 defer 队列
+        // （D3（空间分配） 已定项 7；不释放它，txg 0 的根离开候选集之后这一槽就永远占着，I-3.1（已分配统计对得上） 在抬 F 之后红）。
+        None => format_time_tree_table_to_release(allocator, &resolved.rewritten_roles),
+    };
+    // 整条实例表链重写（写行、回退那一次）：被换下的那条旧链逐片释放，指针取计划带着的那条链（第 1 片起的不在上一版的内存态里）。
+    // 旧链排在别的角色前面，与实例表在 bump 次序里排最前同一个次序。
+    let release = match &plan.instance_table {
+        InstanceTablePlan::Rewrite(rewrite) => {
+            if let Some(previous_version) = previous {
+                assert_eq!(
+                    rewrite.replaced_chain.first(),
+                    Some(&previous_version.root.instance_table),
+                    "计划带着的旧链是上一版根记录指着的那一条：调用方拼行与拼旧链读的是同一条根"
+                );
+            }
+            let mut chain_then_the_rest =
+                instance_table_chain_to_release(&rewrite.replaced_chain, allocator)?;
+            chain_then_the_rest.extend(release);
+            chain_then_the_rest
+        }
+        InstanceTablePlan::Carry(_) => release,
+    };
+    Ok((previous_roles_replaced, release))
 }
 
-/// 接连几次不写文件内容、不碰 inode 树的发布（写行 + 暖机），每一次两棵多层码 2 树重写哪些节点、换下哪些节点，在第一次动分配器之前按
-/// 与发布路径同一个 `code_two_tree::plan_the_tree_after_this_publish` 一次推完。`txgs` 与 `instance` 是这一串要用的 txg 与实例代号
-/// （只进新 key 的出生身份：key 逐字段比、类标签与出生树之后先比出生 txg，它们比上一版里每一把都大、一次比一次大，
-/// 树怎么长只看这个次序，推出来的与真发布时逐节点相同）。
+/// 分配记录树这次重写哪几个节点、这一版有哪几个节点（D8（核心索引结构） 已定项 14：按位置寻址，只重写内容变了的叶与它们的祖先，
+/// 没有记录的一段不写节点）。它们要在取落点之前定下来（角色清单、出生序号、映射 key、中央映射树的形状都要它），而取落点又会改记录——
+/// 这次写的每个单元、连分配记录树自己的节点，每盘各记一条；换下的每一份把它那条改成已释放（D3（空间分配） 已定项 7；这就是释放链）。
+/// 所以走一个固定点：从「重写集空、节点照上一版」起，按它算这次发布的全部角色，在分配器的**拷贝**上把释放与取落点照发布路径的次序
+/// 走一遍，按走出来的记录算有记录的节点与内容变了的节点（[`nodes_whose_contents_changed`]）；节点对得上、变了的都在重写集里（或本来就是新节点）
+/// 就定下，否则节点换成走出来的那一批、变了的并进重写集再走一遍。多罩的节点内容没变、照样重写一份（合法的 COW）。
+/// 真发时走的是同一串释放与取落点，得出同一份记录（释放之前读盘核出对不上的那几份记录留在已分配，只会让变了的节点更少）。
+/// 没有上一版（第一个文件版本）时这棵树的树号换成这次发出来的（树表 0 条那一版的分配记录树是树 0），节点全部重写、不照抄。
+///
+/// **走得完**：产品路径上释放只改写记录、不在这次发布里空出槽（回收要等根记下之后），取落点时分配记录树的节点之后只剩单槽的提交内生块、
+/// 按同一串槽位往后取——多一个角色只把这一串拉长，有记录的节点只多不少，重写集也只增不减，都以池里的节点数为上界。
+/// 只供测试的复用窗口置 0 开关（`ReuseWindow::ForcedToZero`）让释放当场空出槽、回落可能取回它们，那一格的单调性推不出来：
+/// 给 [`ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT`] 轮，走不完交回 `AllocationRecordTreeRewriteSetDidNotSettle`（条款没写这一格怎么办）。
 ///
 /// # Errors
-/// 某一次两棵多层码 2 树的形状算不出来 ⇒ `PublishSequenceRefusal`（第几次、`MultiLevelCodeTwoTreeRefused`）。
-///
-/// # Panics
-/// 形状里有一次重写叶容器或写文件内容（那时换掉的映射 key 不止分配记录树与记账树的那几把），或 `txgs` 与 `shapes` 不等长。
-pub fn multi_level_tree_nodes_of_a_publish_sequence_without_content(
-    previous: &TransactionOutput,
-    shapes: &[PublishShape],
-    txgs: &[CheckpointTxg],
-    instance: InstanceGeneration,
+/// 同 [`PublishPlan::resolve`]、[`placements_released_by_this_publish`]，与拷贝上取不到落点（`PlacementRefused`）——
+/// 真发时同一个落点同样取不到；轮数用完 ⇒ `AllocationRecordTreeRewriteSetDidNotSettle`。都在任何落盘动作之前交回。
+fn settle_the_allocation_record_tree(
+    plan: &PublishPlan<'_>,
+    previous: Option<&TransactionOutput>,
+    trees: &FileVersionTreeIdentifiers,
     device_identities: &[DeviceIdentity],
     capacities: CodeTwoTreeNodeCapacities,
-) -> Result<Vec<MultiLevelTreeNodesOfAPublishWithoutContent>, PublishSequenceRefusal> {
-    assert_eq!(shapes.len(), txgs.len(), "这一串每一次一个 txg");
-    let mut trees_before_this_publish = MultiLevelTreesBeforeAPublishWithoutContent::of(previous);
-    let mut every_publish = Vec::with_capacity(shapes.len());
-    for (publish_index, (shape, txg)) in shapes.iter().zip(txgs).enumerate() {
-        assert_eq!(
-            shape.rewritten_inode_leaf_containers, 0,
-            "这一串里的发布不碰 inode 树：换掉的映射 key 只有分配记录树与记账树的"
-        );
-        assert_eq!(
-            shape.rewritten_data_units, 0,
-            "这一串里的发布不写文件内容：换掉的映射 key 只有分配记录树与记账树的"
+    allocator: &PoolAllocator,
+) -> Result<SettledPublish, PublishError> {
+    let geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
+    let (previous_nodes, previous_records): (
+        BTreeSet<AllocationRecordTreeNode>,
+        &[AllocationRecord],
+    ) = match previous {
+        Some(previous_version) => (
+            previous_version.allocation_record_tree.node_set(),
+            &previous_version.allocation_records,
+        ),
+        None => (BTreeSet::new(), &[]),
+    };
+    let mut rewritten: BTreeSet<AllocationRecordTreeNode> = BTreeSet::new();
+    let mut nodes_after: BTreeSet<AllocationRecordTreeNode> = previous_nodes.clone();
+    // 迭代上界是 ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT 轮；跨轮携带的是重写集（只增不减）与这一版的节点（换成上一轮走出来的那一批）。
+    for _round in 0..ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT {
+        let rewritten_among_the_nodes_after: BTreeSet<AllocationRecordTreeNode> =
+            rewritten.intersection(&nodes_after).copied().collect();
+        let allocation_plan = crate::allocation_record_tree::plan_the_tree_after_this_publish(
+            &previous_nodes,
+            &nodes_after,
+            &rewritten_among_the_nodes_after,
         );
-        let (trees_after_this_publish, nodes_of_this_publish) = trees_before_this_publish
-            .after_a_publish_without_content(
-                publish_index,
-                *txg,
-                instance,
-                &previous.tree_identifiers,
-                device_identities,
-                capacities,
-            )
-            .map_err(|cause| PublishSequenceRefusal {
-                publish_index,
-                cause,
-            })?;
-        every_publish.push(nodes_of_this_publish);
-        trees_before_this_publish = trees_after_this_publish;
-    }
-    Ok(every_publish)
-}
-
-/// 取号之前推算一串不写文件内容的发布时，两棵多层码 2 树在某一次之前的样子：两棵树的形状与每个节点是哪一次写出来的、
-/// 中央映射树里全部的 key、分配记录树与记账树每个节点的映射 key（这一串里换掉的就是这几把）。
-struct MultiLevelTreesBeforeAPublishWithoutContent {
-    accounting_shape: CodeTwoTreeShape,
-    accounting_written_by: BTreeMap<CodeTwoTreeNodePosition, TreeNodeWrittenBy>,
-    central_mapping_shape: CodeTwoTreeShape,
-    central_mapping_written_by: BTreeMap<CodeTwoTreeNodePosition, TreeNodeWrittenBy>,
-    mapping_keys: BTreeSet<CodeTwoTreeKey>,
-    allocation_node_mapping_key: CodeTwoTreeKey,
-    accounting_node_mapping_keys: BTreeMap<CodeTwoTreeNodePosition, CodeTwoTreeKey>,
-}
-
-impl MultiLevelTreesBeforeAPublishWithoutContent {
-    fn of(previous: &TransactionOutput) -> Self {
-        let key_of = |identity: TransactionUnit| {
-            CodeTwoTreeKey::new(
-                &mapping_key_carried_from(previous, identity),
-                CodeTwoKeyFieldWidths::CENTRAL_MAPPING,
-            )
-        };
-        let written_by_the_version_before = |tree: MultiLevelCodeTwoTree| {
-            let shape = &previous.multi_level_tree(tree).shape;
-            shape
-                .nodes()
-                .iter()
-                .map(|node| {
-                    (
-                        node.position,
-                        TreeNodeWrittenBy::TheVersionBeforeTheSequence(
-                            tree.role_of_node(node.position, shape),
-                        ),
-                    )
-                })
-                .collect()
-        };
-        Self {
-            accounting_shape: previous.accounting_tree.shape.clone(),
-            accounting_written_by: written_by_the_version_before(MultiLevelCodeTwoTree::Accounting),
-            central_mapping_shape: previous.central_mapping_tree.shape.clone(),
-            central_mapping_written_by: written_by_the_version_before(
-                MultiLevelCodeTwoTree::CentralMapping,
-            ),
-            mapping_keys: previous
-                .mapped_units
-                .iter()
-                .map(|(_, key)| CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING))
-                .collect(),
-            allocation_node_mapping_key: key_of(TransactionUnit::AllocationTree),
-            accounting_node_mapping_keys: previous
-                .accounting_tree
-                .shape
-                .nodes()
-                .iter()
-                .map(|node| {
-                    (
-                        node.position,
-                        key_of(
-                            MultiLevelCodeTwoTree::Accounting
-                                .role_of_node(node.position, &previous.accounting_tree.shape),
-                        ),
-                    )
-                })
-                .collect(),
-        }
-    }
-
-    /// 一次不写文件内容的发布之后两棵树的样子，与这一次两棵树重写、换下的节点。
-    fn after_a_publish_without_content(
-        &self,
-        publish_index: usize,
-        txg: CheckpointTxg,
-        instance: InstanceGeneration,
-        trees: &FileVersionTreeIdentifiers,
-        device_identities: &[DeviceIdentity],
-        capacities: CodeTwoTreeNodeCapacities,
-    ) -> Result<(Self, MultiLevelTreeNodesOfAPublishWithoutContent), PublishError> {
-        let accounting_keys: BTreeSet<CodeTwoTreeKey> = accounting_entries_of_this_publish(
-            txg,
-            trees.inode,
+        let rewritten_by_the_plan: BTreeSet<AllocationRecordTreeNode> =
+            allocation_plan.rewritten_nodes().into_iter().collect();
+        let resolved = plan.resolve(
+            previous,
+            trees,
             device_identities,
-            &|_| 0,
-            &|_, _| 0,
-        )
-        .iter()
-        .map(|entry| CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::ACCOUNTING))
-        .collect();
-        let accounting_plan = plan_the_tree_after_this_publish(
-            &self.accounting_shape,
-            &accounting_keys,
-            capacities.of_tree(MultiLevelCodeTwoTree::Accounting),
-        )
-        .map_err(|refusal| PublishError::MultiLevelCodeTwoTreeRefused {
-            tree: MultiLevelCodeTwoTree::Accounting,
-            refusal,
-        })?;
-        let new_node_key = |tree: TreeIdentifier, birth_sequence: usize| {
-            CodeTwoTreeKey::new(
-                &mapping_key_of_a_node_born_in_this_publish(
-                    UNIT_CLASS_INDEX_NODE,
-                    tree,
-                    txg,
-                    instance,
-                    BirthSequence(u32::try_from(birth_sequence).expect("出生序号装得进 u32")),
-                ),
-                CodeTwoKeyFieldWidths::CENTRAL_MAPPING,
-            )
-        };
-        let mut mapping_keys = self.mapping_keys.clone();
-        mapping_keys.remove(&self.allocation_node_mapping_key);
-        for key in self.accounting_node_mapping_keys.values() {
-            mapping_keys.remove(key);
-        }
-        let allocation_node_mapping_key = new_node_key(trees.allocation_records, 0);
-        mapping_keys.insert(allocation_node_mapping_key.clone());
-        let mut accounting_node_mapping_keys = BTreeMap::new();
-        let mut accounting_birth_sequence = 0;
-        for (node, origin) in accounting_plan
-            .shape
-            .nodes()
-            .iter()
-            .zip(&accounting_plan.origins)
-        {
-            let key = match origin {
-                CodeTwoTreeNodeOrigin::RewrittenThisPublish => {
-                    accounting_birth_sequence += 1;
-                    new_node_key(trees.accounting, accounting_birth_sequence - 1)
-                }
-                CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => self
-                    .accounting_node_mapping_keys
-                    .get(previous_position)
-                    .expect("照抄的节点在上一次的形状里、有一把 key")
-                    .clone(),
-            };
-            mapping_keys.insert(key.clone());
-            accounting_node_mapping_keys.insert(node.position, key);
+            capacities,
+            allocation_plan,
+        )?;
+        let (previous_roles_replaced, release) =
+            placements_released_by_this_publish(plan, previous, &resolved, allocator)?;
+        let mut rehearsal = allocator.clone();
+        for placement in &release {
+            rehearsal.release(*placement, plan.txg);
+        }
+        // 释放本身弄脏的叶（连祖先与根）这一轮的计划还没罩住：先并进重写集再取落点。这样取落点被拒时交回的角色
+        // 是罩住它们的那一版 bump 次序里第一个取不到的（分配记录树的节点排在前面），不是只罩了一部分的那一版里的。
+        // 这一版的节点也先并进释放之后装着记录的那几个（上一版没有树时释放之前就有的记录也在内：mkfs 的两个单元）；
+        // 释放只改写记录、不删，这几个节点取完落点之后仍装着记录，下面「装着记录的节点等于这一版的节点」那一判不会因此来回。
+        let changed_by_the_release =
+            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
+        if !changed_by_the_release.is_subset(&rewritten_by_the_plan) {
+            rewritten.extend(changed_by_the_release);
+            nodes_after.extend(nodes_holding_records(&geometry, rehearsal.records()));
+            continue;
         }
-        let central_mapping_plan = plan_the_tree_after_this_publish(
-            &self.central_mapping_shape,
-            &mapping_keys,
-            capacities.of_tree(MultiLevelCodeTwoTree::CentralMapping),
-        )
-        .map_err(|refusal| PublishError::MultiLevelCodeTwoTreeRefused {
-            tree: MultiLevelCodeTwoTree::CentralMapping,
-            refusal,
-        })?;
-        let mut rewritten_roles = Vec::new();
-        let mut replaced = Vec::new();
-        let mut written_by_after_this_publish = |tree: MultiLevelCodeTwoTree,
-                                                 plan: &CodeTwoTreePlan,
-                                                 written_by_before: &BTreeMap<
-            CodeTwoTreeNodePosition,
-            TreeNodeWrittenBy,
-        >| {
-            replaced.extend(plan.replaced_previous_nodes.iter().map(|position| {
-                *written_by_before
-                    .get(position)
-                    .expect("被换下的节点在上一次的形状里")
-            }));
-            plan.shape
-                .nodes()
-                .iter()
-                .zip(&plan.origins)
-                .map(|(node, origin)| {
-                    let written_by = match origin {
-                        CodeTwoTreeNodeOrigin::RewrittenThisPublish => {
-                            let role = tree.role_of_node(node.position, &plan.shape);
-                            rewritten_roles.push(role);
-                            TreeNodeWrittenBy::PublishOfTheSequence {
-                                publish_index,
-                                role,
-                            }
-                        }
-                        CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => *written_by_before
-                            .get(previous_position)
-                            .expect("照抄的节点在上一次的形状里"),
-                    };
-                    (node.position, written_by)
-                })
-                .collect::<BTreeMap<CodeTwoTreeNodePosition, TreeNodeWrittenBy>>()
-        };
-        let accounting_written_by = written_by_after_this_publish(
-            MultiLevelCodeTwoTree::Accounting,
-            &accounting_plan,
-            &self.accounting_written_by,
-        );
-        let central_mapping_written_by = written_by_after_this_publish(
-            MultiLevelCodeTwoTree::CentralMapping,
-            &central_mapping_plan,
-            &self.central_mapping_written_by,
-        );
-        Ok((
-            Self {
-                accounting_shape: accounting_plan.shape,
-                accounting_written_by,
-                central_mapping_shape: central_mapping_plan.shape,
-                central_mapping_written_by,
-                mapping_keys,
-                allocation_node_mapping_key,
-                accounting_node_mapping_keys,
-            },
-            MultiLevelTreeNodesOfAPublishWithoutContent {
-                rewritten_roles,
-                replaced,
-            },
-        ))
+        for identity in &resolved.rewritten_roles {
+            allocate_placement_for_role(&mut rehearsal, *identity, plan.txg)?;
+        }
+        let changed =
+            nodes_whose_contents_changed(&geometry, previous_records, rehearsal.records());
+        let holding_records = nodes_holding_records(&geometry, rehearsal.records());
+        if holding_records == nodes_after && changed.is_subset(&rewritten_by_the_plan) {
+            return Ok(SettledPublish {
+                resolved,
+                previous_roles_replaced,
+                release,
+            });
+        }
+        rewritten.extend(changed);
+        nodes_after = holding_records;
     }
+    Err(PublishError::AllocationRecordTreeRewriteSetDidNotSettle {
+        rounds: ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT,
+    })
 }
 
-/// 一串发布的准入里第几次算不过（从 0 数）、为什么。
-#[derive(Debug)]
-pub struct PublishSequenceRefusal {
-    pub publish_index: usize,
-    pub cause: PublishError,
-}
-
-/// 一次发布的那一条准入，分配记录的基数由调用方给：发布路径给的是分配器此刻的记录数，取号之前那一串给的是把前面几次
-/// 要新增的算进去之后的数。`rewritten_role_count` 是这次重写的角色数（两棵多层码 2 树的每个重写节点各算一个）。
-fn admission_of_one_publish(
-    allocator: &PoolAllocator,
-    records_before_this_publish: usize,
-    rewritten_role_count: usize,
-) -> Result<(), PublishError> {
-    // 分配记录树第一版只有一个节点（判在 `refuse_when_the_allocation_records_do_not_fit_one_node`，
-    // 与树表 0 条那一版写行时的准入共用一处）。释放只改写记录、不加；这次重写的每个角色每盘各加一条。
-    let records_after_this_publish =
-        records_before_this_publish + rewritten_role_count * allocator.devices.len();
-    refuse_when_the_allocation_records_do_not_fit_one_node(records_after_this_publish)
-}
+/// 分配记录树重写集的固定点最多走几轮（[`settle_the_allocation_record_tree`]）。产品路径上三四轮就定（第一轮重写集是空的、
+/// 释放弄脏的叶在取落点之前先并进来；下一轮罩住这次写的那几片叶与它们的祖先、再一轮罩住释放链弄脏的叶）；
+/// 只供测试的复用窗口置 0 开关下单调性推不出来，这个数给它一个边。
+pub const ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT: usize = 64;
 
 /// 一次发布切成几条 journal 记录、每条点名哪几个角色（D23（journal 的角色与格式） 已定项 17，C491（多条记录时共享内生块在哪条点名没定）
 /// 2026-09-23 定）：这次写了 N 个数据单元（N ≥ 2）⇒ N 个事务（C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案），
@@ -4592,16 +4787,14 @@
     previous.unit(identity).clone()
 }
 
-/// 一个文件对象这次发布的两类内容角色（数据单元、extent 根）：单元字节、指针、出生序号。
-/// 重写时由 `build_file_version_units` 装，照抄时从上一版取（`carried_file_version_units`）。
-/// 这个对象的 inode 记录不在这里——它进 inode 树，由 `build_inode_tree_units` 按叶容器装。
+/// 一个文件对象这次发布的数据单元：单元字节与指针。重写时由 `build_file_version_units` 装，照抄时从上一版取
+/// （`carried_file_version_units`）。extent 树的节点不在这里——它们按这些指针由 `build_extent_tree` 装；
+/// 这个对象的 inode 记录也不在这里——它进 inode 树，由 `build_inode_tree_units` 按叶容器装。
 struct FileVersionUnits {
     /// 文件的数据单元，第 i 项是文件第 i 个单元（与 `data_pointers` 同序）。
     data_units: Vec<Vec<u8>>,
-    /// 指向每个数据单元的指针，第 i 项是文件第 i 个单元；extent 根兼叶里的记录按同一个次序装（key 升序）。
+    /// 指向每个数据单元的指针，第 i 项是文件第 i 个单元；extent 树下段的叶记录（或内联的那一条）按同一个次序装（key 升序）。
     data_pointers: Vec<DataPointer>,
-    extent_unit: Vec<u8>,
-    extent_sequence: BirthSequence,
     /// C319（请求内单元按 key 升序发出没有条款也没有检查）的运行时计数；照抄的一版恒 0。
     key_order_mismatches: u64,
 }
@@ -4625,21 +4818,17 @@
     data: Vec<SlotNumber>,
 }
 
-/// 装一个文件对象的内容单元（字节表二、四·二）：切分纪律切出的每个一单元事务装一个数据单元（写序带那个事务的号，
-/// 锚点偏移与 extent key 的 offset 段都是这个单元第一个字节的文件偏移，D8（核心索引结构） 已定项 3、D9（加密） 已定项 6），
-/// extent 根兼叶按 key 升序装全部记录。出生序号从调用方传进来的发号器取，发号器的作用域是一次 checkpoint、
-/// 不是这一次调用（D19（块指针的结构与宽度预算） 已定项 9：同一棵树内每写出一个码 2 或码 3 单元加 1，作用域换到下一个 checkpoint 时清零）：
-/// 同一个 checkpoint 里装第二个对象时序号接着数，不在同一个 (树, txg, 实例) 上从 0 重数、撞出重复的映射 key（里程碑「第二个事务」增补 2 第 19 行）。
+/// 装一个文件对象的数据单元（字节表二）：切分纪律切出的每个一单元事务装一个数据单元（写序带那个事务的号，
+/// 锚点偏移与 extent key 的 offset 段都是这个单元第一个字节的文件偏移，D8（核心索引结构） 已定项 3、D9（加密） 已定项 6）。
+/// 数据单元是码 1、不发出生序号；extent 树的节点按这里交出的指针另装（`build_extent_tree`）。
 ///
 /// # Panics
-/// 事务数与落点数不等，或多于一片 extent 叶装得下的记录数：两样都由发布路径在动分配器之前算定
-/// （角色清单按同一张切分排、`publish_version_of_trees` 先判过叶装不装得下），不等说明调用方给错了输入。
+/// 事务数与落点数不等：角色清单按同一张切分排，不等说明调用方给错了输入。
 fn build_file_version_units(
     checkpoint: &FileVersionCheckpoint<'_>,
     file: &FileVersionPlan<'_>,
     transactions: &[OneUnitTransaction],
     slots: &FileVersionSlots,
-    sequences: &mut BirthSequenceAllocator,
 ) -> FileVersionUnits {
     let txg = checkpoint.txg;
     let instance = checkpoint.write_order.instance;
@@ -4651,9 +4840,8 @@
     );
     let mut data_units = Vec::with_capacity(transactions.len());
     let mut data_pointers = Vec::with_capacity(transactions.len());
-    let mut extent_records = Vec::with_capacity(transactions.len());
     let mut extent_keys: Vec<[u8; 24]> = Vec::with_capacity(transactions.len());
-    // 迭代次数的上界是这次的单元数（准入判过不超过一片 extent 叶）；跨轮携带的只有往四个表里追加的那一项，没有提前出口。
+    // 迭代次数的上界是这次的单元数；跨轮携带的只有往三个表里追加的那一项，没有提前出口。
     for (transaction, slot) in transactions.iter().zip(&slots.data) {
         let write_order = WriteOrder {
             instance,
@@ -4691,41 +4879,350 @@
                 .try_into()
                 .expect("24"),
         );
-        extent_records.push(extent_record);
         data_units.push(data_unit);
         data_pointers.push(data_pointer);
     }
     let key_order_mismatches = count_key_order_mismatches(&extent_keys);
-    let smallest_extent_key = extent_keys
-        .first()
-        .expect("切分至少给一个事务（长度 0 的内容也写一个声明长度 0 的数据单元）");
-    let largest_extent_key = extent_keys.last().expect("同上：至少一个事务");
-
-    // t2 extent 树根兼叶（字节表四·二）：key 区间 = 第一条与最后一条记录的 key（I-1.1（key 区间罩住条目））。
-    let extent_sequence = sequences.next(checkpoint.trees.extent, txg, instance);
-    let extent_unit = build_index_node(
-        checkpoint.trees.extent,
-        0,
-        usize::try_from(EXTENT_KEY_BYTES).expect("24"),
-        smallest_extent_key,
-        largest_extent_key,
-        txg,
-        filesystem_identifier,
-        instance,
-        extent_sequence,
-        u16::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
-        &extent_records,
-    );
-
     FileVersionUnits {
         data_units,
         data_pointers,
-        extent_unit,
-        extent_sequence,
         key_order_mismatches,
     }
 }
 
+/// 这一版 extent 树装好的样子：每个节点这一版的指针、全部节点的单元（bump 次序：下段先叶后根、上段先叶后根），与这次重写的节点各自的出生序号。
+struct BuiltExtentTree {
+    version: ExtentTreeVersion,
+    units: Vec<PublishedUnit>,
+    birth_sequences_of_rewritten_nodes: BTreeMap<TransactionUnit, BirthSequence>,
+}
+
+/// 按计划装 extent 树这一版的全部节点（D8（核心索引结构） 已定项 14 的两段）：重写的节点按 bump 次序逐个装、发出生序号、取这次的落点——
+/// 下段先叶后根（叶装它罩的那几个单元的 extent 叶记录，内部节点装「孩子那一段的起点 key + 孩子这一版的指针」），再上段先叶后根
+/// （第一个文件那一条叶条目：一个数据单元就内联它的数据指针（标签 2），多个就放下段根这一版的指针（标签 1）；同一片叶里别的 inode 的条目照抄上一版那片叶的）。
+/// 照抄的节点字节、落点、指针全取上一版同一个位置上的。`data_pointers` 是第一个文件这一版每个数据单元的指针（第 i 项是单元 i）。
+///
+/// # Panics
+/// 重写的节点没拿到落点、照抄的节点不在上一版里、上一版那片上段叶解不开：三样都是发布路径自己的不变量
+/// （角色清单按同一份计划排、上一版的字节是这个进程装的或重建时核过的）。
+fn build_extent_tree(
+    plan: &ExtentTreePlan,
+    previous: Option<&TransactionOutput>,
+    data_pointers: &[DataPointer],
+    tree: TreeIdentifier,
+    context: &MultiLevelTreeBuildContext<'_>,
+    slots: &BTreeMap<TransactionUnit, SlotNumber>,
+    sequences: &mut BirthSequenceAllocator,
+) -> BuiltExtentTree {
+    let identity = ExtentTreeNodeIdentity {
+        tree,
+        birth_txg: context.txg,
+        filesystem_identifier: context.filesystem_identifier,
+        instance: context.instance,
+    };
+    let data_units = u64::try_from(data_pointers.len()).expect("单元数");
+    let mut units: Vec<PublishedUnit> = Vec::new();
+    let mut birth_sequences_of_rewritten_nodes = BTreeMap::new();
+    let mut rewritten_node = |role: TransactionUnit,
+                              build: &dyn Fn(BirthSequence) -> Vec<u8>,
+                              units_so_far: &mut Vec<PublishedUnit>|
+     -> NodePointer {
+        let birth_sequence = sequences.next(tree, context.txg, context.instance);
+        let bytes = build(birth_sequence);
+        let slot = *slots
+            .get(&role)
+            .expect("这次重写的 extent 树节点都在角色清单里、取了落点");
+        birth_sequences_of_rewritten_nodes.insert(role, birth_sequence);
+        let pointer = NodePointer {
+            head: PointerHead {
+                birth_tree: tree,
+                birth_txg: context.txg,
+            },
+            locations: location_entries(context.device_identities, slot, &bytes),
+            instance: context.instance,
+            birth_sequence,
+        };
+        units_so_far.push(PublishedUnit {
+            slot,
+            identity: role,
+            bytes,
+        });
+        pointer
+    };
+    let carried_node = |role_in_the_previous_version: TransactionUnit,
+                        role: TransactionUnit,
+                        units_so_far: &mut Vec<PublishedUnit>| {
+        let previous_version = previous.expect("照抄的 extent 树节点只出现在接着上一版的发布里");
+        let carried = previous_version.unit(role_in_the_previous_version);
+        units_so_far.push(PublishedUnit {
+            slot: carried.slot,
+            identity: role,
+            bytes: carried.bytes.clone(),
+        });
+    };
+    let mut lower_nodes: Vec<(ExtentLowerNodePosition, NodePointer)> = Vec::new();
+    // 迭代上界是下段的节点数；跨轮携带的是已经装好的节点的指针（父节点的条目要孩子这一版的指针，孩子在 bump 次序里排在前面）。
+    for (position, origin) in &plan.lower_nodes {
+        let role = TransactionUnit::ExtentLowerNode(*position);
+        let pointer = match origin {
+            ExtentTreeNodeOrigin::RewrittenThisPublish if position.level == 0 => rewritten_node(
+                role,
+                &|birth_sequence| {
+                    build_lower_leaf(
+                        &identity,
+                        FIRST_INODE_NUMBER,
+                        *position,
+                        data_pointers,
+                        birth_sequence,
+                    )
+                },
+                &mut units,
+            ),
+            ExtentTreeNodeOrigin::RewrittenThisPublish => {
+                let children: Vec<(ExtentLowerNodePosition, NodePointer)> =
+                    lower_children_of_a_file_without_holes(*position, data_units)
+                        .into_iter()
+                        .map(|child| {
+                            let (_, pointer) = lower_nodes
+                                .iter()
+                                .find(|(built, _)| *built == child)
+                                .expect("孩子在 bump 次序里排在父节点前面、先装好了");
+                            (child, *pointer)
+                        })
+                        .collect();
+                rewritten_node(
+                    role,
+                    &|birth_sequence| {
+                        build_lower_internal_node(
+                            &identity,
+                            FIRST_INODE_NUMBER,
+                            *position,
+                            &children,
+                            birth_sequence,
+                        )
+                    },
+                    &mut units,
+                )
+            }
+            ExtentTreeNodeOrigin::CarriedFromThePreviousVersion => {
+                carried_node(role, role, &mut units);
+                previous
+                    .and_then(|previous_version| {
+                        previous_version.extent_tree.lower_pointer_of(*position)
+                    })
+                    .expect("照抄的下段节点在上一版里")
+            }
+        };
+        lower_nodes.push((*position, pointer));
+    }
+    let upper_root_level = plan.upper_root_level();
+    let previous_upper_root_level =
+        previous.map(|previous_version| previous_version.extent_tree.upper_root().level);
+    let mut upper_nodes: Vec<(ExtentUpperNodePosition, NodePointer)> = Vec::new();
+    // 迭代上界是上段的节点数；跨轮携带的同下段。
+    for (position, origin) in &plan.upper_nodes {
+        let role = role_of_extent_upper_node(*position, upper_root_level);
+        let role_in_the_previous_version = previous_upper_root_level
+            .map(|previous_root_level| role_of_extent_upper_node(*position, previous_root_level));
+        let pointer = match origin {
+            ExtentTreeNodeOrigin::RewrittenThisPublish if position.level == 0 => {
+                let target = match lower_nodes.last() {
+                    None => ExtentUpperLeafTarget::InlineDataUnit(
+                        *data_pointers
+                            .first()
+                            .expect("写文件内容的发布至少一个数据单元（长度 0 的内容也写一个）"),
+                    ),
+                    Some((_, lower_root_pointer)) => {
+                        ExtentUpperLeafTarget::LowerSegmentRoot(*lower_root_pointer)
+                    }
+                };
+                // 同一片叶里别的 inode 的条目照抄上一版那片叶的（第一版只有第一个文件有内容，今天这一串是空的）。
+                let mut entries: Vec<ExtentUpperLeafEntry> = role_in_the_previous_version
+                    .filter(|_| {
+                        previous.is_some_and(|previous_version| {
+                            previous_version
+                                .extent_tree
+                                .upper_pointer_of(*position)
+                                .is_some()
+                        })
+                    })
+                    .map(|previous_role| {
+                        let previous_leaf = parse_index_node(
+                            &previous.expect("上一版有这片叶").unit(previous_role).bytes,
+                        )
+                        .expect("上一版的上段叶是这个进程装的，或重建时核过的");
+                        previous_leaf
+                            .entries
+                            .iter()
+                            .map(|entry| {
+                                ExtentUpperLeafEntry::parse(entry)
+                                    .expect("上一版的上段叶条目是这个进程装的，或重建时核过的")
+                            })
+                            .filter(|entry| entry.inode != FIRST_INODE_NUMBER)
+                            .collect()
+                    })
+                    .unwrap_or_default();
+                entries.push(ExtentUpperLeafEntry {
+                    inode: FIRST_INODE_NUMBER,
+                    target,
+                });
+                entries.sort_by_key(|entry| entry.inode);
+                rewritten_node(
+                    role,
+                    &|birth_sequence| {
+                        build_upper_leaf(&identity, *position, &entries, birth_sequence)
+                    },
+                    &mut units,
+                )
+            }
+            ExtentTreeNodeOrigin::RewrittenThisPublish => {
+                let children: Vec<(ExtentUpperNodePosition, NodePointer)> = upper_nodes
+                    .iter()
+                    .filter(|(child, _)| {
+                        child.level + 1 == position.level && child.parent() == *position
+                    })
+                    .copied()
+                    .collect();
+                rewritten_node(
+                    role,
+                    &|birth_sequence| {
+                        build_upper_internal_node(&identity, *position, &children, birth_sequence)
+                    },
+                    &mut units,
+                )
+            }
+            ExtentTreeNodeOrigin::CarriedFromThePreviousVersion => {
+                carried_node(
+                    role_in_the_previous_version.expect("照抄的上段节点只出现在接着上一版的发布里"),
+                    role,
+                    &mut units,
+                );
+                previous
+                    .and_then(|previous_version| {
+                        previous_version.extent_tree.upper_pointer_of(*position)
+                    })
+                    .expect("照抄的上段节点在上一版里")
+            }
+        };
+        upper_nodes.push((*position, pointer));
+    }
+    BuiltExtentTree {
+        version: ExtentTreeVersion {
+            upper_nodes,
+            lower_nodes,
+        },
+        units,
+        birth_sequences_of_rewritten_nodes,
+    }
+}
+
+/// 这一版分配记录树装好的样子：每个节点这一版的指针（bump 次序）、这次重写的节点的单元（bump 次序），与它们各自的出生序号。
+struct BuiltAllocationRecordTree {
+    version: AllocationRecordTreeVersion,
+    rewritten_units: Vec<PublishedUnit>,
+    birth_sequences_of_rewritten_nodes: BTreeMap<TransactionUnit, BirthSequence>,
+}
+
+/// 按计划装分配记录树这一版的全部节点（D8（核心索引结构） 已定项 14）：重写的节点按 bump 次序（按位置先叶后根）逐个装——叶装它罩的那一段里的全部记录
+/// （按 key 升序），内部节点装它每个孩子「那一段的起点 key + 孩子这一版的指针」；头里的 key 区间是这个节点按位置规定罩的那一段。
+/// 出生序号按同一个次序发，落点取这次分配的。照抄的节点只取指针（`previous_pointer_of`）：带文件的一版上它的字节由调用方从上一版照抄进 `units`，
+/// 树表 0 条的一版上不写照抄的节点。`records` 是这次发布取完落点之后分配器里的全部记录。
+///
+/// # Panics
+/// 重写的节点没拿到落点，或照抄的节点在上一版里没有指针：角色清单按同一份计划排，上一版的节点是这个进程记的或重建时读回来的。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "装一棵树要的：计划、几何、记录、上一版节点的指针、树号、身份字段、落点、发号器，各自独立"
+)]
+fn build_allocation_record_tree(
+    plan: &AllocationRecordTreePlan,
+    geometry: &AllocationRecordTreeGeometry,
+    records: &[AllocationRecord],
+    previous_pointer_of: &dyn Fn(AllocationRecordTreeNode) -> Option<NodePointer>,
+    tree: TreeIdentifier,
+    context: &MultiLevelTreeBuildContext<'_>,
+    slot_of_rewritten_role: &dyn Fn(TransactionUnit) -> SlotNumber,
+    sequences: &mut BirthSequenceAllocator,
+) -> BuiltAllocationRecordTree {
+    let records_by_leaf = records_of_each_leaf(geometry, records);
+    let node_set = plan.node_set();
+    let mut pointers: BTreeMap<AllocationRecordTreeNode, NodePointer> = BTreeMap::new();
+    let mut rewritten_units = Vec::new();
+    let mut birth_sequences_of_rewritten_nodes = BTreeMap::new();
+    // 迭代上界是这一版的节点数；跨轮携带的是已经装好的节点的指针（父节点的条目要孩子这一版的指针，孩子在 bump 次序里排在前面）。
+    for (node, origin) in &plan.nodes {
+        let role = role_of_allocation_record_tree_node(*node);
+        let pointer = match origin {
+            AllocationRecordTreeNodeOrigin::CarriedFromThePreviousVersion => {
+                previous_pointer_of(*node).expect("照抄的分配记录树节点在上一版里有指针")
+            }
+            AllocationRecordTreeNodeOrigin::RewrittenThisPublish => {
+                let birth_sequence = sequences.next(tree, context.txg, context.instance);
+                let contents = match geometry.level_of(*node) {
+                    0 => AllocationRecordTreeNodeContents::Leaf(
+                        records_by_leaf
+                            .get(node)
+                            .map(Vec::as_slice)
+                            .expect("这一版的叶都有记录（规划按有记录的叶排）"),
+                    ),
+                    _ => AllocationRecordTreeNodeContents::Internal(
+                        children_of(geometry, &node_set, *node)
+                            .into_iter()
+                            .map(|child| {
+                                (
+                                    child,
+                                    *pointers
+                                        .get(&child)
+                                        .expect("孩子在 bump 次序里排在父节点前面、先装好了"),
+                                )
+                            })
+                            .collect(),
+                    ),
+                };
+                let bytes = build_allocation_record_tree_node(
+                    geometry,
+                    *node,
+                    &contents,
+                    tree,
+                    context.txg,
+                    context.filesystem_identifier,
+                    context.instance,
+                    birth_sequence,
+                );
+                let slot = slot_of_rewritten_role(role);
+                birth_sequences_of_rewritten_nodes.insert(role, birth_sequence);
+                let pointer = NodePointer {
+                    head: PointerHead {
+                        birth_tree: tree,
+                        birth_txg: context.txg,
+                    },
+                    locations: location_entries(context.device_identities, slot, &bytes),
+                    instance: context.instance,
+                    birth_sequence,
+                };
+                rewritten_units.push(PublishedUnit {
+                    slot,
+                    identity: role,
+                    bytes,
+                });
+                pointer
+            }
+        };
+        pointers.insert(*node, pointer);
+    }
+    BuiltAllocationRecordTree {
+        version: AllocationRecordTreeVersion {
+            nodes: plan
+                .nodes
+                .iter()
+                .map(|(node, _)| (*node, pointers[node]))
+                .collect(),
+        },
+        rewritten_units,
+        birth_sequences_of_rewritten_nodes,
+    }
+}
+
 /// 这一版 inode 树的全部单元：每片叶容器（重写的是这次装的、没重写的照抄上一版），加上重写了的根。
 struct InodeTreeUnits {
     /// 左起按 key 序，每片一项。
@@ -4873,7 +5370,7 @@
     }
 }
 
-/// 照抄上一版的文件内容角色：数据指针、每个数据单元与 extent 根的字节都取上一版。
+/// 照抄上一版的数据单元：数据指针与每个数据单元的字节都取上一版（extent 树的节点照抄在 `build_extent_tree` 里）。
 fn carried_file_version_units(carried: &TransactionOutput) -> FileVersionUnits {
     FileVersionUnits {
         data_units: (0..carried.data_pointers.len())
@@ -4887,10 +5384,6 @@
             })
             .collect(),
         data_pointers: carried.data_pointers.clone(),
-        extent_unit: carried.unit(TransactionUnit::ExtentRoot).bytes.clone(),
-        extent_sequence: carried
-            .tree_root_pointer(carried.tree_identifiers.extent.0)
-            .birth_sequence,
         key_order_mismatches: 0,
     }
 }
@@ -4904,7 +5397,7 @@
               参数各是一样东西：写入口、分配器、计划、上一版、算好的树、要释放的、要隔离的、树号"
 )]
 fn publish_admitted<Device: BlockDevice>(
-    pool: &mut PoolWriter<'_, Device>,
+    pool: &PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
     plan: &PublishPlan<'_>,
     previous: Option<&TransactionOutput>,
@@ -4955,6 +5448,11 @@
             &devices_whose_copy_failed_the_checksum,
         );
     }
+    // 第一个文件版本把树表 0 条那一版的分配记录树整棵换下了（上面释放的就是它的节点，`format_time_tree_table_to_release`）：
+    // 分配器不再记着它，之后的发布按这一版自己的分配记录树走。
+    if previous.is_none() {
+        allocator.forget_the_allocation_record_tree_of_the_version_without_file();
+    }
 
     // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。
     let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
@@ -4976,7 +5474,7 @@
         device_identities: &device_identities,
     };
 
-    // 文件内容角色：有新版本就按切分装每个数据单元与 extent 根，没有就照抄上一版的指针与字节。
+    // 文件内容角色：有新版本就按切分装每个数据单元，没有就照抄上一版的指针与字节。
     let carried_file = match &plan.file {
         Some(_) => None,
         None => Some(previous.expect("没有文件版本的发布要接在上一版之后：文件角色从它照抄")),
@@ -4984,8 +5482,6 @@
     let FileVersionUnits {
         data_units,
         data_pointers,
-        extent_unit,
-        extent_sequence,
         key_order_mismatches,
     } = match (&plan.file, carried_file) {
         (Some(file), _) => build_file_version_units(
@@ -5000,13 +5496,28 @@
                     })
                     .collect(),
             },
-            &mut sequences,
         ),
         (None, Some(carried)) => carried_file_version_units(carried),
         (None, None) => {
             unreachable!("上面按 plan.file 分过：没有文件版本时 carried_file 一定是 Some")
         }
     };
+    let multi_level_tree_context = MultiLevelTreeBuildContext {
+        txg,
+        filesystem_identifier,
+        instance,
+        device_identities: &device_identities,
+    };
+    // t2 extent 树（D8（核心索引结构） 已定项 14 的两段）：写文件内容的发布重写下段整段与上段那一串，别的照抄。
+    let extent_built = build_extent_tree(
+        &resolved.extent_tree,
+        previous,
+        &data_pointers,
+        trees.extent,
+        &multi_level_tree_context,
+        &slots,
+        &mut sequences,
+    );
 
     // t3 inode 树（字节表四）：这次重写的叶容器与根都在这里装；没重写的叶容器取上一版的指针，字节下面照抄。
     let previous_inode_leaf_containers: &[InodeLeafContainerVersion] = match previous {
@@ -5054,17 +5565,45 @@
         }
     };
 
-    // t5 分配记录树（字节表五）：mkfs 的两个单元分配代 0、其余是各自分配那次发布的 txg，每盘各一条，按 (设备, 槽号) 升序。
-    let allocation_sequence = sequences.next(trees.allocation_records, txg, instance);
+    // t5 分配记录树（字节表五；D8（核心索引结构） 已定项 14 按绝对槽号按位置寻址）：mkfs 的两个单元分配代 0、其余是各自分配那次发布的 txg，
+    // 每盘各一条，按 (设备, 槽号) 升序分装进各自那片叶。重写哪几个节点在准入之前走到固定点定下来（`settle_the_allocation_record_tree`）：
+    // 这里取完落点之后内容变了的节点都在里面（释放之前读盘核出对不上的那几份留在已分配，只会让变了的更少）。
     let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();
     allocation_records.sort_by_key(AllocationRecord::sort_key);
-    let allocation_unit = build_allocation_record_node(
-        allocator,
+    let allocation_geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
+    let rewritten_allocation_nodes: BTreeSet<AllocationRecordTreeNode> = resolved
+        .allocation_record_tree
+        .rewritten_nodes()
+        .into_iter()
+        .collect();
+    let previous_allocation_records: &[AllocationRecord] = match previous {
+        Some(previous_version) => &previous_version.allocation_records,
+        None => &[],
+    };
+    assert!(
+        nodes_whose_contents_changed(
+            &allocation_geometry,
+            previous_allocation_records,
+            &allocation_records
+        )
+        .is_subset(&rewritten_allocation_nodes)
+            && nodes_holding_records(&allocation_geometry, &allocation_records)
+                == resolved.allocation_record_tree.node_set(),
+        "准入之前在分配器拷贝上走到的固定点，与真发时取完落点的记录说的是同一棵树：两边走的是同一串释放与取落点"
+    );
+    let allocation_built = build_allocation_record_tree(
+        &resolved.allocation_record_tree,
+        &allocation_geometry,
+        &allocation_records,
+        &|node| {
+            previous.and_then(|previous_version| {
+                previous_version.allocation_record_tree.pointer_of(node)
+            })
+        },
         trees.allocation_records,
-        txg,
-        filesystem_identifier,
-        instance,
-        allocation_sequence,
+        &multi_level_tree_context,
+        &slot_of,
+        &mut sequences,
     );
 
     // t6 记账树（D5（快照 / 空间记账机制） 已定项 8）：两盘 15 行——带设备维的六项每盘一行、池级三行；
@@ -5118,12 +5657,6 @@
             }
         },
     );
-    let multi_level_tree_context = MultiLevelTreeBuildContext {
-        txg,
-        filesystem_identifier,
-        instance,
-        device_identities: &device_identities,
-    };
     // 记账树按计划装（D8（核心索引结构） 已定项 11）：行装不下一个节点时从中间切，节点按先叶后根发出生序号、取这次的落点。
     let accounting_built = build_multi_level_tree(
         MultiLevelCodeTwoTree::Accounting,
@@ -5156,16 +5689,13 @@
         instance,
         birth_sequence: sequence,
     };
-    // 文件内容角色的指针：重写的按这次的落点算，照抄的取上一版。
-    let extent_pointer = match carried_file {
-        None => node_pointer(
-            trees.extent,
-            TransactionUnit::ExtentRoot,
-            &extent_unit,
-            extent_sequence,
-        ),
-        Some(carried) => carried.tree_root_pointer(trees.extent.0),
-    };
+    // extent 树根（上段的根）的指针：装树时已经按这次的落点算好（照抄的取上一版）。
+    let extent_pointer = *extent_built
+        .version
+        .upper_nodes
+        .last()
+        .map(|(_, pointer)| pointer)
+        .expect("上段恒有第一个文件那片叶到根那一串");
     // inode 树根的指针：这次重写了就按这次的落点算，一片叶都没重写就取上一版树表里那一条。
     let inode_root_pointer = match (&inode_tree_units.rewritten_root, previous) {
         (Some(root), _) => node_pointer(
@@ -5179,16 +5709,11 @@
             unreachable!("没有上一版的发布（第一个文件版本）恒要写 inode 记录 ⇒ 恒重写 inode 树")
         }
     };
-    let allocation_pointer = node_pointer(
-        trees.allocation_records,
-        TransactionUnit::AllocationTree,
-        &allocation_unit,
-        allocation_sequence,
-    );
+    let allocation_pointer = allocation_built.version.root_pointer();
     let accounting_pointer = accounting_built.version.root_pointer();
 
-    // t7 中央映射树（字节表三·二）：码 1 每个数据单元一条 + 码 2 / 码 3 的 extent 根、每片 inode 叶容器、inode 根、分配记录树、
-    // 记账树的每个节点各一条；映射树自己、树表、实例表豁免（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
+    // t7 中央映射树（字节表三·二）：码 1 每个数据单元一条 + 码 2 / 码 3 的 extent 树每个节点、每片 inode 叶容器、inode 根、
+    // 分配记录树每个节点、记账树的每个节点各一条；映射树自己、树表、实例表豁免（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
     // 照抄的角色 key 照旧、位置照旧；重写的按这次的指针算。数据单元的码 1 key 带写序（事务号），
     // 一事务一单元 ⇒ 同一个文件的各个单元 key 不撞（D19（块指针的结构与宽度预算） 已定项 6 压在切分纪律上）。
     let mut mapped_units_with_locations: Vec<(TransactionUnit, Vec<u8>, [LocationEntry; 2])> =
@@ -5205,11 +5730,29 @@
                 )
             })
             .collect();
-    mapped_units_with_locations.push((
-        TransactionUnit::ExtentRoot,
-        mapping_key_for_node(UNIT_CLASS_INDEX_NODE, extent_pointer),
-        extent_pointer.locations,
-    ));
+    for (role, pointer) in extent_tree_roles_in_bump_order(&extent_built.version)
+        .into_iter()
+        .zip(
+            extent_built
+                .version
+                .lower_nodes
+                .iter()
+                .map(|(_, pointer)| pointer)
+                .chain(
+                    extent_built
+                        .version
+                        .upper_nodes
+                        .iter()
+                        .map(|(_, pointer)| pointer),
+                ),
+        )
+    {
+        mapped_units_with_locations.push((
+            role,
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
+            pointer.locations,
+        ));
+    }
     for container in &inode_tree_units.leaf_containers {
         mapped_units_with_locations.push((
             TransactionUnit::InodeLeafContainer(container.index),
@@ -5217,18 +5760,18 @@
             container.pointer.locations,
         ));
     }
-    mapped_units_with_locations.extend([
-        (
-            TransactionUnit::InodeRoot,
-            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer),
-            inode_root_pointer.locations,
-        ),
-        (
-            TransactionUnit::AllocationTree,
-            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer),
-            allocation_pointer.locations,
-        ),
-    ]);
+    mapped_units_with_locations.push((
+        TransactionUnit::InodeRoot,
+        mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer),
+        inode_root_pointer.locations,
+    ));
+    for (node, pointer) in &allocation_built.version.nodes {
+        mapped_units_with_locations.push((
+            role_of_allocation_record_tree_node(*node),
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
+            pointer.locations,
+        ));
+    }
     for (node, pointer) in accounting_built
         .version
         .shape
@@ -5377,10 +5920,7 @@
             Some(carried) => carried_unit(carried, identity),
         });
     }
-    units.push(match carried_file {
-        None => rewritten_unit(TransactionUnit::ExtentRoot, extent_unit.clone()),
-        Some(carried) => carried_unit(carried, TransactionUnit::ExtentRoot),
-    });
+    units.extend(extent_built.units.iter().cloned());
     for container in &inode_tree_units.leaf_containers {
         let identity = TransactionUnit::InodeLeafContainer(container.index);
         units.push(if container.is_rewritten_this_publish {
@@ -5399,10 +5939,23 @@
             TransactionUnit::InodeRoot,
         ),
     });
-    units.push(rewritten_unit(
-        TransactionUnit::AllocationTree,
-        allocation_unit.clone(),
-    ));
+    // 分配记录树按 bump 次序：重写的是这次装的，照抄的从上一版同一个位置那个角色拷。
+    for (node, _) in &allocation_built.version.nodes {
+        let identity = role_of_allocation_record_tree_node(*node);
+        units.push(
+            match allocation_built
+                .rewritten_units
+                .iter()
+                .find(|unit| unit.identity == identity)
+            {
+                Some(allocation_record_tree_node_unit) => allocation_record_tree_node_unit.clone(),
+                None => carried_unit(
+                    previous.expect("照抄的分配记录树节点只出现在接着上一版的发布里"),
+                    identity,
+                ),
+            },
+        );
+    }
     units.extend(accounting_built.units.iter().cloned());
     units.extend(mapping_built.units.iter().cloned());
     units.push(rewritten_unit(
@@ -5437,7 +5990,15 @@
                     .expect("点名的数据单元是这次装的那几个之一")
                     .write_order,
             ),
-            TransactionUnit::ExtentRoot => node_key_tail(instance, extent_sequence),
+            TransactionUnit::ExtentLowerNode(_)
+            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
+            | TransactionUnit::ExtentRoot => node_key_tail(
+                instance,
+                *extent_built
+                    .birth_sequences_of_rewritten_nodes
+                    .get(&identity)
+                    .expect("点名的 extent 树节点是这次重写的"),
+            ),
             TransactionUnit::InodeLeafContainer(index) => node_key_tail(
                 instance,
                 inode_tree_units
@@ -5455,7 +6016,14 @@
                     .expect("点名 inode 根的发布一定重写了它")
                     .birth_sequence,
             ),
-            TransactionUnit::AllocationTree => node_key_tail(instance, allocation_sequence),
+            TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
+            | TransactionUnit::AllocationTree => node_key_tail(
+                instance,
+                *allocation_built
+                    .birth_sequences_of_rewritten_nodes
+                    .get(&identity)
+                    .expect("点名的分配记录树节点是这次重写的"),
+            ),
             TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree => node_key_tail(
                 instance,
@@ -5603,6 +6171,8 @@
         data_pointers,
         mapping_keys,
         allocation_records,
+        allocation_record_tree: allocation_built.version,
+        extent_tree: extent_built.version,
         accounting_entries,
         accounting_tree: accounting_built.version,
         central_mapping_tree: mapping_built.version,
@@ -5888,31 +6458,47 @@
             u64::try_from(content.len()).expect("内容长度"),
             FIRST_TRANSACTION_NUMBER,
         );
-        let first = build_file_version_units(
-            &checkpoint,
-            &file,
-            &one_unit_transactions,
-            &FileVersionSlots {
-                data: vec![SlotNumber(50176)],
-            },
-            &mut checkpoint_sequences,
-        );
-        let second = build_file_version_units(
-            &checkpoint,
-            &file,
-            &one_unit_transactions,
-            &FileVersionSlots {
-                data: vec![SlotNumber(50178)],
-            },
-            &mut checkpoint_sequences,
-        );
+        let context = MultiLevelTreeBuildContext {
+            txg,
+            filesystem_identifier: &filesystem_identifier,
+            instance,
+            device_identities: &device_identities,
+        };
+        let extent_plan = plan_the_extent_tree_after_this_publish(None, Some(1));
+        let extent_tree_of =
+            |data_slot: u64, extent_root_slot: u64, sequences: &mut BirthSequenceAllocator| {
+                let units = build_file_version_units(
+                    &checkpoint,
+                    &file,
+                    &one_unit_transactions,
+                    &FileVersionSlots {
+                        data: vec![SlotNumber(data_slot)],
+                    },
+                );
+                let mut extent_slots = BTreeMap::new();
+                extent_slots.insert(TransactionUnit::ExtentRoot, SlotNumber(extent_root_slot));
+                build_extent_tree(
+                    &extent_plan,
+                    None,
+                    &units.data_pointers,
+                    checkpoint.trees.extent,
+                    &context,
+                    &extent_slots,
+                    sequences,
+                )
+            };
+        let first = extent_tree_of(50176, 50240, &mut checkpoint_sequences);
+        let second = extent_tree_of(50178, 50241, &mut checkpoint_sequences);
         assert_eq!(
-            (first.extent_sequence, second.extent_sequence),
+            (
+                first.birth_sequences_of_rewritten_nodes[&TransactionUnit::ExtentRoot],
+                second.birth_sequences_of_rewritten_nodes[&TransactionUnit::ExtentRoot]
+            ),
             (BirthSequence(0), BirthSequence(1)),
             "同一个 checkpoint 的第二个对象接着数：extent 树 0、1"
         );
         assert_eq!(
-            parse_index_node(&second.extent_unit)
+            parse_index_node(&second.units[0].bytes)
                 .expect("刚装的 extent 根解得开")
                 .birth_sequence,
             BirthSequence(1),
@@ -6003,6 +6589,16 @@
             assert_eq!(identity.tag().len(), 2);
         }
         assert_eq!(
+            TransactionUnit::AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition {
+                level: 0,
+                device: DeviceIdentity(1),
+                index_in_device: 61,
+            })
+            .tag(),
+            "t5@0.1.61",
+            "分配记录树根之下的节点按位置写步号"
+        );
+        assert_eq!(
             TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST)
                 .placement(),
             PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned),
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/write_accounting.rs tree/crates/singlefs-core/src/write_accounting.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-core/src/write_accounting.rs	2026-09-24 16:50:54.714292753 +0000
+++ tree/crates/singlefs-core/src/write_accounting.rs	2026-09-24 20:56:53.114716670 +0000
@@ -49,10 +49,14 @@
     pub const fn of_unit(identity: TransactionUnit) -> WrittenStructureKind {
         match identity {
             TransactionUnit::Data(_) => WrittenStructureKind::DataUnit,
-            TransactionUnit::ExtentRoot => WrittenStructureKind::ExtentTreeNode,
+            // 按 key 空间定形状的两棵树同样按树数、不按段与层数（extent 树上段与下段、分配记录树每一层都是这一种）。
+            TransactionUnit::ExtentLowerNode(_)
+            | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
+            | TransactionUnit::ExtentRoot => WrittenStructureKind::ExtentTreeNode,
             TransactionUnit::InodeLeafContainer(_) => WrittenStructureKind::InodeTreeLeafContainer,
             TransactionUnit::InodeRoot => WrittenStructureKind::InodeTreeRoot,
-            TransactionUnit::AllocationTree => WrittenStructureKind::AllocationRecordTreeNode,
+            TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
+            | TransactionUnit::AllocationTree => WrittenStructureKind::AllocationRecordTreeNode,
             // 多层之后根之下的节点与根同一种：按树数，不按层数（增补 1 的账按结构种类记）。
             TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree => WrittenStructureKind::AccountingTreeNode,
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-format/src/lib.rs tree/crates/singlefs-format/src/lib.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-format/src/lib.rs	2026-09-24 18:48:58.241580116 +0000
+++ tree/crates/singlefs-format/src/lib.rs	2026-09-24 19:10:19.219676936 +0000
@@ -133,6 +133,33 @@
 /// 中央映射树内部节点条目：分隔 key 27 + 子指针 86（D8（核心索引结构） 已定项 11）。format-const: CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES
 pub const CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES: u64 = 113;
 
+/// 分配记录树一片叶罩几个槽（W，D8（核心索引结构） 已定项 14「分配记录树」：按绝对槽号按位置寻址，叶 k 罩 `[k × W, (k + 1) × W)`，
+/// W 取偶数、W ≤ 叶条目容量 812）：取叶条目容量本身 812。条款把具体取值交给实现员，交回里写明，主 agent 定后补进已定项 14。
+pub const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
+
+/// 分配记录树内部节点条目：孩子罩的那一段的起点 key 10 + 子指针 86（D8（核心索引结构） 已定项 11 那一行「照码 2 btree 做时它们的内部条目是 96」）。
+pub const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES: u64 = 96;
+
+/// 分配记录树内部节点的扇出：(16384 − 135) ÷ 96 的整数部分。一个内部节点罩 169 个孩子那么宽的一段。
+pub const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
+
+/// extent 树内部节点条目（上段与下段同一种）：孩子罩的那一段的起点 key 24 + 子指针 86（D8（核心索引结构） 已定项 11 那一行，
+/// extent 110 是用户 2026-09-24 定的）。
+pub const EXTENT_TREE_INTERNAL_ENTRY_BYTES: u64 = 110;
+
+/// extent 树内部节点的扇出（上段与下段同一种）：(16384 − 163) ÷ 110 的整数部分（D8（核心索引结构） 已定项 14「内部扇出 147」）。
+pub const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;
+
+/// extent 树下段一片叶罩几个数据单元（D8（核心索引结构） 已定项 14「叶罩 144 个单元」）：(16384 − 163) ÷ 112 的整数部分。
+pub const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;
+
+/// extent 树上段叶条目：key 24（locality 0、inode 号、offset 0）+ 标签 1 + 载荷 88（标签 1 时是下段根的节点指针 86 加 2 字节零，
+/// 标签 2 时是那个数据单元的数据指针 88，标签 0 时全零）。D8（核心索引结构） 已定项 14 把上段叶条目的完整字段表交给实现员，交回里写明。
+pub const EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES: u64 = 113;
+
+/// extent 树上段一片叶罩几个 inode 号：(16384 − 163) ÷ 113 的整数部分（一个 inode 号至多一条上段叶条目，叶永远装得下）。
+pub const EXTENT_TREE_UPPER_LEAF_INODES: u64 = 143;
+
 /// 第一个事务这次发布写出的记账行数（D5（快照 / 空间记账机制） 已定项 8，2026-09-14 用户定案 15 行）。
 pub const FIRST_TRANSACTION_ACCOUNTING_ROWS: u64 = 15;
 
@@ -290,6 +317,49 @@
         assert_eq!(MAPPING_ENTRY_BYTES, 55, "映射条目一宽");
         assert_eq!(EXTENT_LEAF_RECORD_BYTES, 112, "extent 叶记录");
         assert_eq!(
+            ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES,
+            ALLOCATION_RECORD_KEY_BYTES + NODE_POINTER_BYTES,
+            "分配记录树内部条目 = key 10 + 子指针"
+        );
+        assert_eq!(
+            ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
+            (NODE_BYTES - ALLOCATION_RECORDS_TREE_INDEX_NODE_HEADER_BYTES)
+                / ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES,
+            "分配记录树内部扇出 = 节点里条目区装得下几条内部条目"
+        );
+        assert!(
+            ALLOCATION_RECORD_TREE_LEAF_SLOTS.is_multiple_of(2)
+                && ALLOCATION_RECORD_TREE_LEAF_SLOTS
+                    <= (NODE_BYTES - ALLOCATION_RECORDS_TREE_INDEX_NODE_HEADER_BYTES)
+                        / ALLOCATION_RECORD_BYTES,
+            "分配记录树叶宽取偶数、不超过叶条目容量（D8 已定项 14）"
+        );
+        assert_eq!(
+            EXTENT_TREE_INTERNAL_ENTRY_BYTES,
+            EXTENT_KEY_BYTES + NODE_POINTER_BYTES,
+            "extent 内部条目 = key 24 + 子指针"
+        );
+        assert_eq!(
+            EXTENT_TREE_INTERNAL_FANOUT,
+            (NODE_BYTES - EXTENT_TREE_INDEX_NODE_HEADER_BYTES) / EXTENT_TREE_INTERNAL_ENTRY_BYTES,
+            "extent 内部扇出 147"
+        );
+        assert_eq!(
+            EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
+            (NODE_BYTES - EXTENT_TREE_INDEX_NODE_HEADER_BYTES) / EXTENT_LEAF_RECORD_BYTES,
+            "extent 下段叶罩 144 个单元"
+        );
+        assert_eq!(
+            EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES,
+            EXTENT_KEY_BYTES + 1 + DATA_POINTER_BYTES,
+            "extent 上段叶条目 = key 24 + 标签 1 + 载荷 88（两种指针里宽的那一种）"
+        );
+        assert_eq!(
+            EXTENT_TREE_UPPER_LEAF_INODES,
+            (NODE_BYTES - EXTENT_TREE_INDEX_NODE_HEADER_BYTES) / EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES,
+            "extent 上段叶罩几个 inode 号"
+        );
+        assert_eq!(
             INODE_INTERNAL_ENTRY,
             8 + 26 + NODE_POINTER_BYTES,
             "inode 内部条目 = 分隔 key 8 + 身份引用 26 + 子指针"
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-harness/src/bad_disk_input.rs tree/crates/singlefs-harness/src/bad_disk_input.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-harness/src/bad_disk_input.rs	2026-09-23 22:56:34.090557683 +0000
+++ tree/crates/singlefs-harness/src/bad_disk_input.rs	2026-09-24 21:01:15.127131029 +0000
@@ -830,24 +830,25 @@
     let Some(reached) = reach_the_tree_table(image) else {
         return false;
     };
-    // extent / 分配记录 / 记账三棵要的是「非空的叶」：按记录改的那几条坏法只坏叶（那几棵树内部节点的条目格式
-    // 还没有条款）。inode 树只要非空：条目宽那一条对内部节点与叶都做得出来。
-    let non_empty_leaf = [
-        TREE_KIND_EXTENT,
-        TREE_KIND_ALLOCATION_RECORDS,
-        TREE_KIND_ACCOUNTING,
-    ]
-    .into_iter()
-    .all(|tree_kind| {
-        chain_from_the_tree_table(image, &reached, tree_kind).is_some_and(|chain| {
-            index_node_is_a_leaf(&chain.tree_root_node)
-                && IndexNodeEntryLayout::of(&chain.tree_root_node).entry_count > 0
-        })
-    });
+    // extent / 记账两棵要的是「非空的叶」：按记录改的那几条坏法坏的是根那一片（第一版只有 inode 1 有 extent，extent 树上段只有根兼叶；
+    // 记账树在两块盘的池上是根兼叶）。分配记录树按位置寻址、根恒在第 1 层以上（D8（核心索引结构） 已定项 14），坏记录的坏法沿路走到叶，
+    // 这里只要根非空。inode 树只要非空：条目宽那一条对内部节点与叶都做得出来。
+    let non_empty_leaf = [TREE_KIND_EXTENT, TREE_KIND_ACCOUNTING]
+        .into_iter()
+        .all(|tree_kind| {
+            chain_from_the_tree_table(image, &reached, tree_kind).is_some_and(|chain| {
+                index_node_is_a_leaf(&chain.tree_root_node)
+                    && IndexNodeEntryLayout::of(&chain.tree_root_node).entry_count > 0
+            })
+        });
+    let allocation_record_tree_is_non_empty =
+        chain_from_the_tree_table(image, &reached, TREE_KIND_ALLOCATION_RECORDS)
+            .is_some_and(|chain| IndexNodeEntryLayout::of(&chain.tree_root_node).entry_count > 0);
     // 中央映射树的根住根记录、不经树表（D19 已定项 11）：条目宽那一条要它非空。
     let central_mapping_is_non_empty = open_chain_to_the_central_mapping_tree_root(image)
         .is_some_and(|chain| IndexNodeEntryLayout::of(&chain.mapping_root_node).entry_count > 0);
     non_empty_leaf
+        && allocation_record_tree_is_non_empty
         && central_mapping_is_non_empty
         && chain_from_the_tree_table(image, &reached, TREE_KIND_INODE)
             .is_some_and(|chain| IndexNodeEntryLayout::of(&chain.tree_root_node).entry_count > 0)
@@ -1610,6 +1611,88 @@
     }
 }
 
+/// 从一棵树的根往下走、沿路读到的一个节点：它落在的两块盘与槽、它的字节，与上一层里指着它的是第几条条目。
+struct NodeBelowTheRoot {
+    devices: [u32; 2],
+    slot: u64,
+    bytes: Vec<u8>,
+    entry_index_in_the_parent: usize,
+}
+
+/// 从一棵树的根（`root_node`）往下走到一片叶：每一层在内部条目里取 key 满足 `goes_at_or_before_the_target` 的最后一条
+/// （一条都不满足取第 0 条），读它的子指针（紧跟 key 的 86 字节）指的节点。交回沿路读到的节点，根之下第一层在前、叶在末；
+/// 根自己就是叶时交回空的。哪一层一条条目都没有、条目宽窄于 key + 子指针、子指针读不出来，交回 `None`。
+/// 分配记录树与 extent 树按位置寻址（D8（核心索引结构） 已定项 14），根之下还有节点，坏记录的坏法要走到叶。
+fn path_down_to_the_leaf_holding(
+    image: &MemoryPool,
+    root_node: &[u8],
+    goes_at_or_before_the_target: &dyn Fn(&[u8]) -> bool,
+) -> Option<Vec<NodeBelowTheRoot>> {
+    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
+    let mut path: Vec<NodeBelowTheRoot> = Vec::new();
+    // 迭代上界是树高（码 2 头的层级是 1 字节，至多 256 层）；跨轮携带的是走到的那个节点。
+    for _level in 0..=u8::MAX {
+        let current: &[u8] = path.last().map_or(root_node, |node| node.bytes.as_slice());
+        if index_node_is_a_leaf(current) {
+            return Some(path);
+        }
+        let layout = IndexNodeEntryLayout::of(current);
+        let key_width = index_node_key_width(current);
+        if layout.entry_count == 0 || layout.entry_width < key_width + 86 {
+            return None;
+        }
+        let chosen = (0..layout.entry_count)
+            .rev()
+            .find(|index| {
+                let start = layout.offset_of_entry(*index);
+                goes_at_or_before_the_target(&current[start..start + key_width])
+            })
+            .unwrap_or(0);
+        let pointer_start = layout.offset_of_entry(chosen) + key_width;
+        let (devices, slot, bytes) = read_unit_via_pointer(
+            image,
+            &current[pointer_start..pointer_start + 86],
+            node_bytes,
+        )?;
+        path.push(NodeBelowTheRoot {
+            devices,
+            slot,
+            bytes,
+            entry_index_in_the_parent: chosen,
+        });
+    }
+    None
+}
+
+/// 沿 `path`（[`path_down_to_the_leaf_holding`] 交回的那一路）从叶往上：每个节点重封、写回它的两块盘，它的整单元校验和写进
+/// 上一层指着它的那条条目的子指针两条位置条目（最上面那个的上一层是 `root_node`）。根自己由调用方接着封
+/// （[`seal_and_write_back_the_chain`]）。
+fn reseal_the_path_below_the_root(
+    image: &mut MemoryPool,
+    root_node: &mut [u8],
+    path: &mut [NodeBelowTheRoot],
+) {
+    // 迭代上界是这一路的节点数；每一轮只动这一层与它上一层。
+    for position in (0..path.len()).rev() {
+        let (above, from_here) = path.split_at_mut(position);
+        let node = &mut from_here[0];
+        reseal_index_node(&mut node.bytes);
+        write_unit_to_both_devices(image, node.devices, node.slot, &node.bytes);
+        let checksum = crc32_castagnoli_table(&node.bytes);
+        let parent: &mut [u8] = match above.last_mut() {
+            Some(parent) => &mut parent.bytes,
+            None => root_node,
+        };
+        let pointer_start = IndexNodeEntryLayout::of(parent)
+            .offset_of_entry(node.entry_index_in_the_parent)
+            + index_node_key_width(parent);
+        write_unit_checksum_into_both_locations(
+            &mut parent[pointer_start..pointer_start + 86],
+            checksum,
+        );
+    }
+}
+
 /// 把改过的链逐环写回镜像：树根 → 树表条目里的整单元校验和 → 树表节点 → 根记录里的整单元校验和 → 根槽的自证校验和。
 /// 少重算一道，读者在解析之前就先拒了，指着普查那几族的坏法一次也打不到。
 fn seal_and_write_back_the_chain(mut chain: ChainToATreeRoot, image: &mut MemoryPool) {
@@ -1947,31 +2030,31 @@
     InstanceTablePlacementReleasedOnTheSecondDeviceOnly,
 }
 
-/// 把「根槽 → 树表 → 分配记录树根」这条链上第一条分配记录改成「起点贴着单元区末尾、跨度 32767 槽」
-/// （越过单元区末尾，普查 R6 / R7），并把链上的五道校验和逐道重算：分配记录树根两道 →
-/// 树表条目里那条根指针的整单元校验和 → 树表节点两道 → 根槽里树表指针的整单元校验和 → 根槽的自证校验和。
+/// 把「根槽 → 树表 → 分配记录树根 → …… → 最左那片叶」这条链上那片叶的第一条分配记录改成「起点贴着单元区末尾、跨度 32767 槽」
+/// （越过单元区末尾，普查 R6 / R7），并把链上的校验和逐道重算：叶两道 → 上一层第 0 条条目子指针里的整单元校验和、那一层两道 →
+/// …… → 分配记录树根两道 → 树表条目里那条根指针的整单元校验和 → 树表节点两道 → 根槽里树表指针的整单元校验和 → 根槽的自证校验和。
 ///
-/// **槽号也要挪，只改跨度不够**：几何判那四样按「设备身份 → 槽号下界 → 跨度上界 → 同盘两条罩同一个槽」的次序判，
-/// 一条从单元区低处起跨 32767 槽的记录会先被**最后**那一样接走（它罩过了后面每一条记录），
-/// 于是「跨度越过单元区末尾」那一判被遮蔽、去掉它也不红。挪到 `unit_area_end_slot - 1` 之后这条记录谁都不罩，
-/// 跨度那一判是**唯一**拦着它的东西。
+/// **分配记录树按绝对槽号按位置寻址之后（D8（核心索引结构） 已定项 14）**，这条记录先撞上的是叶那一判：起点不在这片叶按位置罩的那一段里
+/// （读者报「分配记录不在它所在叶按位置罩的那一段里，或末槽越过叶的末槽」）。「跨度越过单元区末尾」那一判只剩罩着盘末尾的那片叶上
+/// 的记录够得着（一条落在叶里、末槽越过盘末尾的记录），这条坏法打不到它。
 ///
-/// **三份字节由调用方从盘上读来、改完自己写回去**：坏盘输入那一路读的是 [`MemoryPool`]，
-/// 步 4 回退那一路读的是录制设备，两边共用这一份，偏移与重算口径只有这一处
+/// **字节由调用方从盘上读来、改完自己写回去**：`allocation_tree_nodes_from_the_root` 是从根起、每层沿第 0 条条目往下读到的那一路
+/// （根在前、叶在末）。坏盘输入那一路读的是 [`MemoryPool`]，步 4 回退那一路读的是录制设备，两边共用这一份，偏移与重算口径只有这一处
 /// （`code-discipline.md`「重复要生成，不许手抄」）。跨度字段整个写成
 /// [`SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED`]：跨度取低 15 位的最大值、已释放位跟着清掉
 /// （只有还着的记录才会被影子账拿去隔离）。
 ///
-/// 交回「坏在哪」那句话；树表里没有分配记录树、它的根不是叶、或者一条记录都没有时交回 `None`。
+/// 交回「坏在哪」那句话；树表里没有分配记录树、这一路的末一个不是叶、中间有一层没有条目、或者叶里一条记录都没有时交回 `None`。
 #[must_use]
 pub fn move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal_the_chain(
     root_slot: &mut [u8],
     tree_table_node: &mut [u8],
-    allocation_tree_node: &mut [u8],
+    allocation_tree_nodes_from_the_root: &mut [Vec<u8>],
     unit_area_end_slot: u64,
 ) -> Option<String> {
-    let layout = IndexNodeEntryLayout::of(allocation_tree_node);
-    if layout.entry_count == 0 || !index_node_is_a_leaf(allocation_tree_node) {
+    let leaf = allocation_tree_nodes_from_the_root.last_mut()?;
+    let layout = IndexNodeEntryLayout::of(leaf);
+    if layout.entry_count == 0 || !index_node_is_a_leaf(leaf) {
         return None;
     }
     let tree_table_layout = IndexNodeEntryLayout::of(tree_table_node);
@@ -1984,22 +2067,36 @@
 
     let first_entry = layout.offset_of_entry(0);
     let slot_offset = first_entry + ALLOCATION_RECORD_SLOT_OFFSET;
-    let old_slot = read_six_byte_unsigned_at(allocation_tree_node, slot_offset);
+    let old_slot = read_six_byte_unsigned_at(leaf, slot_offset);
     let last_slot_of_the_unit_area = unit_area_end_slot.checked_sub(1)?;
-    write_six_byte_unsigned_at(
-        allocation_tree_node,
-        slot_offset,
-        last_slot_of_the_unit_area,
-    );
+    write_six_byte_unsigned_at(leaf, slot_offset, last_slot_of_the_unit_area);
     let span_offset = first_entry + ALLOCATION_RECORD_SPAN_OFFSET;
-    let old_span_field = read_u16_at(allocation_tree_node, span_offset);
-    allocation_tree_node[span_offset..span_offset + 2]
+    let old_span_field = read_u16_at(leaf, span_offset);
+    leaf[span_offset..span_offset + 2]
         .copy_from_slice(&SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED.to_le_bytes());
-    reseal_index_node(allocation_tree_node);
+    reseal_index_node(leaf);
+    // 迭代上界是这一路的节点数；每一轮把下一层的整单元校验和写进这一层第 0 条条目的子指针、再封这一层。
+    for level_from_the_root in (0..allocation_tree_nodes_from_the_root.len() - 1).rev() {
+        let (above, below) =
+            allocation_tree_nodes_from_the_root.split_at_mut(level_from_the_root + 1);
+        let node = above.last_mut()?;
+        let child_checksum = crc32_castagnoli_table(&below[0]);
+        let node_layout = IndexNodeEntryLayout::of(node);
+        if node_layout.entry_count == 0 || index_node_is_a_leaf(node) {
+            return None;
+        }
+        let pointer_start = node_layout.offset_of_entry(0) + index_node_key_width(node);
+        write_unit_checksum_into_both_locations(
+            &mut node[pointer_start..pointer_start + 86],
+            child_checksum,
+        );
+        reseal_index_node(node);
+    }
+    let allocation_tree_root = allocation_tree_nodes_from_the_root.first()?;
 
     let root_pointer_start = tree_table_layout.offset_of_entry(allocation_entry_index)
         + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
-    let allocation_tree_checksum = crc32_castagnoli_table(allocation_tree_node);
+    let allocation_tree_checksum = crc32_castagnoli_table(allocation_tree_root);
     write_unit_checksum_into_both_locations(
         &mut tree_table_node[root_pointer_start..root_pointer_start + 86],
         allocation_tree_checksum,
@@ -2015,31 +2112,73 @@
     reseal_wide_checksum_field(root_slot, cover_end, ROOT_SELF_CHECKSUM_OFFSET);
 
     Some(format!(
-        "分配器：这条根那棵账里第一条分配记录的槽号 {old_slot} → {last_slot_of_the_unit_area}（单元区末尾是 {unit_area_end_slot}）、跨度字段 {old_span_field:#06x} → {SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED:#06x}（跨度 {} → 32767 槽，已释放位清掉）；链上五道校验和重算",
+        "分配器：这条根那棵账里第一条分配记录的槽号 {old_slot} → {last_slot_of_the_unit_area}（单元区末尾是 {unit_area_end_slot}）、跨度字段 {old_span_field:#06x} → {SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED:#06x}（跨度 {} → 32767 槽，已释放位清掉）；链上每一道校验和重算",
         old_span_field & SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED
     ))
 }
 
-/// 分配器那一族：把分配记录树根里的一条记录改坏（普查 R6 / R7 / R8 / R9）。
+/// 分配记录树（按绝对槽号按位置寻址，D8（核心索引结构） 已定项 14）里一把 key 的两段：(盘, 槽号)，按数值比（盘上小端，逐字节比不对）。
+fn allocation_record_key_fields(key: &[u8]) -> (u32, u64) {
+    (
+        read_u32_at(key, ALLOCATION_RECORD_DEVICE_OFFSET),
+        read_six_byte_unsigned_at(key, ALLOCATION_RECORD_SLOT_OFFSET),
+    )
+}
+
+/// 分配器那一族：把分配记录树某一片叶里的一条记录改坏（普查 R6 / R7 / R8 / R9）。分配记录树按位置寻址、根恒在第 1 层以上
+/// （D8（核心索引结构） 已定项 14）：记录住在叶里，改哪一片叶按坏法定（实例表那一条改它所在的那片，别的改最左那片），
+/// 改完沿路往上把整单元校验和补到根、再补树表与根槽。
 fn rewrite_an_allocation_record(
     image: &mut MemoryPool,
     damage: AllocationRecordDamage,
 ) -> Option<String> {
     let mut chain = open_chain_to_the_root_of(image, TREE_KIND_ALLOCATION_RECORDS)?;
-    let layout = IndexNodeEntryLayout::of(&chain.tree_root_node);
-    // 内部节点的条目不是分配记录（那几棵树的内部条目格式还没有条款），只坏叶。
-    if layout.entry_count == 0 || !index_node_is_a_leaf(&chain.tree_root_node) {
+    let target_key = match damage {
+        AllocationRecordDamage::InstanceTablePlacementReleasedOnTheSecondDeviceOnly => {
+            let instance_table_pointer = &chain.root_slot
+                [ROOT_INSTANCE_TABLE_POINTER_OFFSET..ROOT_INSTANCE_TABLE_POINTER_OFFSET + 86];
+            (
+                read_u32_at(
+                    instance_table_pointer,
+                    POINTER_SECOND_LOCATION_OFFSET + LOCATION_DEVICE_OFFSET_IN_ENTRY,
+                ),
+                read_six_byte_unsigned_at(
+                    instance_table_pointer,
+                    POINTER_FIRST_LOCATION_OFFSET + LOCATION_SLOT_OFFSET_IN_ENTRY,
+                ),
+            )
+        }
+        AllocationRecordDamage::SlotBelowTheUnitArea
+        | AllocationRecordDamage::SpanPastTheEndOfTheUnitArea
+        | AllocationRecordDamage::SameSlotAsTheSecondRecord
+        | AllocationRecordDamage::DeviceOutsideThePool => (0, 0),
+    };
+    let mut path = path_down_to_the_leaf_holding(image, &chain.tree_root_node, &|entry_key| {
+        allocation_record_key_fields(entry_key) <= target_key
+    })?;
+    let leaf = &mut path.last_mut()?.bytes;
+    let what = damage_an_allocation_record_in_the_leaf(leaf, &chain.root_slot, damage)?;
+    reseal_the_path_below_the_root(image, &mut chain.tree_root_node, &mut path);
+    seal_and_write_back_the_chain(chain, image);
+    Some(what)
+}
+
+/// 把 `leaf`（分配记录树的一片叶）里的一条记录按坏法改掉；交回「坏在哪」那句话，这片叶上没有要坏的对象时交回 `None`。
+fn damage_an_allocation_record_in_the_leaf(
+    leaf: &mut [u8],
+    root_slot: &[u8],
+    damage: AllocationRecordDamage,
+) -> Option<String> {
+    let layout = IndexNodeEntryLayout::of(leaf);
+    if layout.entry_count == 0 || !index_node_is_a_leaf(leaf) {
         return None;
     }
     let first_entry = layout.offset_of_entry(0);
     let what = match damage {
         AllocationRecordDamage::SlotBelowTheUnitArea => {
-            let old = read_six_byte_unsigned_at(
-                &chain.tree_root_node,
-                first_entry + ALLOCATION_RECORD_SLOT_OFFSET,
-            );
+            let old = read_six_byte_unsigned_at(leaf, first_entry + ALLOCATION_RECORD_SLOT_OFFSET);
             write_six_byte_unsigned_at(
-                &mut chain.tree_root_node,
+                leaf,
                 first_entry + ALLOCATION_RECORD_SLOT_OFFSET,
                 SLOT_FAR_BELOW_THE_UNIT_AREA,
             );
@@ -2049,8 +2188,8 @@
         }
         AllocationRecordDamage::SpanPastTheEndOfTheUnitArea => {
             let span_offset = first_entry + ALLOCATION_RECORD_SPAN_OFFSET;
-            let old = read_u16_at(&chain.tree_root_node, span_offset);
-            chain.tree_root_node[span_offset..span_offset + 2].copy_from_slice(
+            let old = read_u16_at(leaf, span_offset);
+            leaf[span_offset..span_offset + 2].copy_from_slice(
                 &SPAN_FIELD_HOLDING_THE_LARGEST_SPAN_AND_NOT_RELEASED.to_le_bytes(),
             );
             format!(
@@ -2063,9 +2202,9 @@
                 return None;
             }
             let second_entry = layout.offset_of_entry(1);
-            let key_width = index_node_key_width(&chain.tree_root_node);
-            let second_key = chain.tree_root_node[second_entry..second_entry + key_width].to_vec();
-            chain.tree_root_node[first_entry..first_entry + key_width].copy_from_slice(&second_key);
+            let key_width = index_node_key_width(leaf);
+            let second_key = leaf[second_entry..second_entry + key_width].to_vec();
+            leaf[first_entry..first_entry + key_width].copy_from_slice(&second_key);
             format!(
                 "分配器：第一条分配记录的 key 换成第二条的（盘 {}、槽 {}），两条罩住同一个槽",
                 read_u32_at(&second_key, ALLOCATION_RECORD_DEVICE_OFFSET),
@@ -2074,8 +2213,8 @@
         }
         AllocationRecordDamage::DeviceOutsideThePool => {
             let device_offset = first_entry + ALLOCATION_RECORD_DEVICE_OFFSET;
-            let old = read_u32_at(&chain.tree_root_node, device_offset);
-            chain.tree_root_node[device_offset..device_offset + 4]
+            let old = read_u32_at(leaf, device_offset);
+            leaf[device_offset..device_offset + 4]
                 .copy_from_slice(&DEVICE_IDENTITY_OUTSIDE_THE_POOL.to_le_bytes());
             format!(
                 "分配器：第一条分配记录的设备身份 {old} → {DEVICE_IDENTITY_OUTSIDE_THE_POOL}（池里只有 0 与 1）"
@@ -2084,7 +2223,7 @@
         AllocationRecordDamage::InstanceTablePlacementReleasedOnTheSecondDeviceOnly => {
             // 这一版实例表的落点：每次发布都重写实例表，所以这个落点一定走到释放判定路径
             // （`TransactionUnit::InstanceTable` 的落点从根记录里那条指针取，不经映射）。
-            let instance_table_pointer = &chain.root_slot
+            let instance_table_pointer = &root_slot
                 [ROOT_INSTANCE_TABLE_POINTER_OFFSET..ROOT_INSTANCE_TABLE_POINTER_OFFSET + 86];
             let slot = read_six_byte_unsigned_at(
                 instance_table_pointer,
@@ -2097,30 +2236,25 @@
             );
             let entry_index = (0..layout.entry_count).find(|index| {
                 let entry = layout.offset_of_entry(*index);
-                read_u32_at(
-                    &chain.tree_root_node,
-                    entry + ALLOCATION_RECORD_DEVICE_OFFSET,
-                ) == second_device
-                    && read_six_byte_unsigned_at(
-                        &chain.tree_root_node,
-                        entry + ALLOCATION_RECORD_SLOT_OFFSET,
-                    ) == slot
+                read_u32_at(leaf, entry + ALLOCATION_RECORD_DEVICE_OFFSET) == second_device
+                    && read_six_byte_unsigned_at(leaf, entry + ALLOCATION_RECORD_SLOT_OFFSET)
+                        == slot
             })?;
             let entry = layout.offset_of_entry(entry_index);
             let span_offset = entry + ALLOCATION_RECORD_SPAN_OFFSET;
-            let old_span_field = read_u16_at(&chain.tree_root_node, span_offset);
+            let old_span_field = read_u16_at(leaf, span_offset);
             if old_span_field & SPAN_FIELD_RELEASED_BIT != 0 {
                 return None;
             }
-            chain.tree_root_node[span_offset..span_offset + 2]
+            leaf[span_offset..span_offset + 2]
                 .copy_from_slice(&(old_span_field | SPAN_FIELD_RELEASED_BIT).to_le_bytes());
             let generation_offset = entry + ALLOCATION_RECORD_GENERATION_OFFSET;
             let old_generation = u64::from_le_bytes(
-                chain.tree_root_node[generation_offset..generation_offset + 8]
+                leaf[generation_offset..generation_offset + 8]
                     .try_into()
                     .expect("8 字节"),
             );
-            chain.tree_root_node[generation_offset..generation_offset + 8]
+            leaf[generation_offset..generation_offset + 8]
                 .copy_from_slice(&RELEASE_GENERATION_ABOVE_EVERY_ROOT.to_le_bytes());
             format!(
                 "分配器：实例表落点（槽 {slot}）在盘 {second_device} 上那条分配记录（第 {entry_index} 条）改成已释放、释放代 {old_generation} → {RELEASE_GENERATION_ABOVE_EVERY_ROOT}（高过任何根，挂载时不会被回收掉）；盘 {} 上那条原样",
@@ -2131,7 +2265,6 @@
             )
         }
     };
-    seal_and_write_back_the_chain(chain, image);
     Some(what)
 }
 
@@ -2220,22 +2353,62 @@
     ))
 }
 
-/// 判别力那一条：把最新那条根下 extent 树第一条记录指的数据单元整个重写，**只**重算这个单元自己的两道校验和，
-/// extent 记录里位置条目上的整单元校验和原样留着。
-/// 挡着它的只有恢复里 `read_unit_via_locations` 那一道整单元 CRC 比对：去掉它，恢复就读回这一版从没提交过的内容。
-fn rewrite_a_data_unit_resealing_only_its_own_checksums(image: &mut MemoryPool) -> Option<String> {
+/// extent 树上段叶条目的标签：载荷是下段根的节点指针 / 这个文件唯一那个数据单元的数据指针（D8（核心索引结构） 已定项 14）。
+const EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT: u8 = 1;
+const EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT: u8 = 2;
+
+/// 最新那条根下 extent 树第一个数据单元的数据指针那 88 字节：extent 树按位置寻址（D8（核心索引结构） 已定项 14），
+/// 上段沿第 0 条条目走到最左那片叶，取第一条上段叶条目（key 24 + 标签 1 + 载荷 88）；标签 2 时载荷就是那个数据指针，
+/// 标签 1 时载荷的前 86 字节是下段根指针，下段再沿第 0 条条目走到最左那片叶，取第一条 extent 叶记录（key 24 + 数据指针 88）的指针。
+/// 哪一步走不通（条目宽窄于字段表、标签是 0 或不认识、节点读不出）交回 `None`。
+fn first_data_pointer_of_the_extent_tree(image: &MemoryPool) -> Option<Vec<u8>> {
     let chain = open_chain_to_the_root_of(image, TREE_KIND_EXTENT)?;
-    let layout = IndexNodeEntryLayout::of(&chain.tree_root_node);
-    if layout.entry_count == 0
-        || !index_node_is_a_leaf(&chain.tree_root_node)
-        || layout.entry_width < usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112")
-    {
+    let upper_path = path_down_to_the_leaf_holding(image, &chain.tree_root_node, &|_| false)?;
+    let upper_leaf: &[u8] = upper_path
+        .last()
+        .map_or(chain.tree_root_node.as_slice(), |node| {
+            node.bytes.as_slice()
+        });
+    let upper_layout = IndexNodeEntryLayout::of(upper_leaf);
+    let key_width = index_node_key_width(upper_leaf);
+    if upper_layout.entry_count == 0 || upper_layout.entry_width < key_width + 1 + 88 {
         return None;
     }
-    let key_width = index_node_key_width(&chain.tree_root_node);
-    let data_pointer_start = layout.offset_of_entry(0) + key_width;
-    let view =
-        parse_data_pointer(&chain.tree_root_node[data_pointer_start..data_pointer_start + 88]);
+    let tag_offset = upper_layout.offset_of_entry(0) + key_width;
+    let payload = &upper_leaf[tag_offset + 1..tag_offset + 1 + 88];
+    match upper_leaf[tag_offset] {
+        EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT => Some(payload.to_vec()),
+        EXTENT_UPPER_LEAF_ENTRY_TAG_LOWER_SEGMENT_ROOT => {
+            let (_, _, lower_root) = read_unit_via_pointer(
+                image,
+                &payload[..86],
+                usize::try_from(NODE_BYTES).expect("16384"),
+            )?;
+            let lower_path = path_down_to_the_leaf_holding(image, &lower_root, &|_| false)?;
+            let lower_leaf: &[u8] = lower_path
+                .last()
+                .map_or(lower_root.as_slice(), |node| node.bytes.as_slice());
+            let lower_layout = IndexNodeEntryLayout::of(lower_leaf);
+            if lower_layout.entry_count == 0
+                || lower_layout.entry_width
+                    < usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112")
+            {
+                return None;
+            }
+            let data_pointer_start =
+                lower_layout.offset_of_entry(0) + index_node_key_width(lower_leaf);
+            Some(lower_leaf[data_pointer_start..data_pointer_start + 88].to_vec())
+        }
+        _ => None,
+    }
+}
+
+/// 判别力那一条：把最新那条根下 extent 树第一个数据单元整个重写，**只**重算这个单元自己的两道校验和，
+/// 指着它的那条数据指针里位置条目上的整单元校验和原样留着。
+/// 挡着它的只有恢复里 `read_unit_via_locations` 那一道整单元 CRC 比对：去掉它，恢复就读回这一版从没提交过的内容。
+fn rewrite_a_data_unit_resealing_only_its_own_checksums(image: &mut MemoryPool) -> Option<String> {
+    let data_pointer = first_data_pointer_of_the_extent_tree(image)?;
+    let view = parse_data_pointer(&data_pointer);
     if view.all_zero || view.locations[0].slot != view.locations[1].slot {
         return None;
     }
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs tree/crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	2026-09-24 05:10:20.899145112 +0000
+++ tree/crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	2026-09-24 21:36:05.371480421 +0000
@@ -2698,7 +2698,7 @@
         "β0 上镜像读法与内存读法应当一致（都还没 corrupt）"
     );
 
-    // ===== H0：暖机之后连续覆盖写 6N 次（N = 3S，S 是每区槽数；ρ = 1，每次都是 O），撞墙即截断 =====
+    // ===== H0：暖机之后连续覆盖写 6N 次（N = 3S，S 是每区槽数；ρ = 1，每次都是 O），写失败即截断 =====
     let ring_length = ROOT_RING_REGIONS * E156_SLOTS_PER_REGION;
     let baseline_workload_length = 6 * ring_length;
     let hr_prefix_length = 3 * ring_length;
@@ -2724,17 +2724,10 @@
         let published = match outcome {
             Ok(published) => published,
             Err(failure) => {
-                if let PublishError::AllocationRecordsExceedOneNode { records, capacity } = failure
-                {
-                    assert_eq!(capacity, 812, "A7：撞墙时的节点容量应为 812");
-                    emitter.emit(&format!(
-                        "name=baseline_workload_truncated_by_write_failure requested_length={baseline_workload_length} actual_length={baseline_workload_actual_length} failed_at_step={step} records={records} capacity={capacity}"
-                    ));
-                } else {
-                    emitter.emit(&format!(
-                        "name=baseline_workload_truncated_by_write_failure requested_length={baseline_workload_length} actual_length={baseline_workload_actual_length} failed_at_step={step} failure={failure:?}"
-                    ));
-                }
+                // 分配记录树按位置寻址之后（D8（核心索引结构） 已定项 14）没有一个节点 812 条那道墙，截断只剩别的失败。
+                emitter.emit(&format!(
+                    "name=baseline_workload_truncated_by_write_failure requested_length={baseline_workload_length} actual_length={baseline_workload_actual_length} failed_at_step={step} failure={failure:?}"
+                ));
                 break;
             }
         };
@@ -2851,7 +2844,7 @@
         &hr_current,
         &memory_pool_snapshot(&hr_devices),
     );
-    // 同 H0：分配记录树没有回收，前缀段大概率在到 3N 之前就撞上 `AllocationRecordsExceedOneNode`（见 H0 那段注释）。
+    // 同 H0：分配记录树按位置寻址之后（D8（核心索引结构） 已定项 14）没有一个节点 812 条那道墙，截断只剩别的失败。
     let mut hr_prefix_actual_length: u64 = 0;
     for step in 1..=hr_prefix_length {
         let mut pool = PoolWriter::new(&parameters, hr_devices.as_mut_slice());
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-harness/src/history.rs tree/crates/singlefs-harness/src/history.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-harness/src/history.rs	2026-09-24 18:48:58.241580116 +0000
+++ tree/crates/singlefs-harness/src/history.rs	2026-09-24 20:56:53.151716323 +0000
@@ -51,8 +51,8 @@
 use crate::fault_injection::{FaultInjectingBlockDevice, SharedFaultPlan};
 use crate::model::{
     IdealModel, ModelAnswer, ModelCheckpointTxg, ModelDeviceIdentity, ModelDisagreement,
-    ModelJudgementCounts, ModelPoolGeometry, ModelRefusalReason, ModelRootKey, ObservedEffect,
-    ObservedOutcome, ObservedRefusalReason,
+    ModelJudgementCounts, ModelPoolGeometry, ModelRootKey, ObservedEffect, ObservedOutcome,
+    ObservedRefusalReason,
 };
 use crate::model_comparison::{
     model_root_key, observed_mount, observed_read_back, observed_root_of_file_version,
@@ -508,12 +508,12 @@
         rollback_targets: RollbackTargetDraw::FloorRootHalfTheTime,
     };
 
-    /// 分配记录墙那一格（增补 3 第 2 件代码三方第一轮判决第三节第 1 条）的取样点：一律从第一个文件起，带文件的版本上多半覆盖写
-    /// （每次每盘加 8 条），夹着可写挂载（写行与暖机每次加 18 或 26 条）、抬 F（回收之后复用改写记录，条数涨得慢）与少量回退，
-    /// 让逼近 812 条时的条数落在不同的余数上——墙的「差一」只在某次准入之后正好 812 条时分得出。配 `PerStepChecker::Run` 跑
+    /// 原分配记录墙那一格（增补 3 第 2 件代码三方第一轮判决第三节第 1 条）的取样点：一律从第一个文件起，带文件的版本上多半覆盖写，
+    /// 夹着可写挂载、抬 F（回收之后复用改写记录，条数涨得慢）与少量回退，把分配记录攒过 812 条。分配记录树按绝对槽号按位置寻址之后
+    /// （D8（核心索引结构） 已定项 14）那道墙拆了，这一段看的是越过 812 条之后多叶的树每一步都判绿、模型对得上。配 `PerStepChecker::Run` 跑
     /// （第二轮判决第三节第 3 条：第一轮配的是 `Skipped`，攻方两条只有 checker 看得见的变异在它上面一段都不红）。
     pub const TOWARD_THE_ALLOCATION_RECORD_WALL: GenerationWeights = GenerationWeights {
-        name: "逼近分配记录墙（812 条那一格的取样点）",
+        name: "越过原分配记录墙（812 条那一格的取样点）",
         starting_points: &[(HistoryStartingPoint::AfterFirstFile, 1)],
         with_session_closed: &[
             (HistoryOperationKind::CloseAndMountWritable, 95),
@@ -756,8 +756,7 @@
     }
 }
 
-/// 入口返回 Err 时交给模型的观测：成员映射成的理由、拒之前做完几次发布、录制流在这一步里有没有多出写或屏障；理由是分配记录墙时
-/// 连同从镜像上数的准入基数（`allocation_records_counted_for_the_wall`）。
+/// 入口返回 Err 时交给模型的观测：成员映射成的理由、拒之前做完几次发布、录制流在这一步里有没有多出写或屏障。
 fn observed_refusal(
     member: String,
     reason: ObservedRefusalReason,
@@ -765,7 +764,6 @@
     stream_length_before: usize,
     stream: &SharedStream,
     reported_ceiling: Option<ModelCheckpointTxg>,
-    allocation_records_counted_on_the_image: Option<u64>,
 ) -> ObservedOutcome {
     ObservedOutcome::Refused {
         member,
@@ -773,7 +771,6 @@
         publishes_completed,
         wrote_anything: stream.operation_count() != stream_length_before,
         reported_ceiling,
-        allocation_records_counted_on_the_image,
     }
 }
 
@@ -811,23 +808,6 @@
     Some(u64::try_from(records).expect("一个节点至多几百条"))
 }
 
-/// 分配记录墙拒时，从镜像上现数准入的基数（增补 3 第 2 件代码三方第一轮判决第三节第 1 条：用 checker 的解析读镜像，不用分配器的状态）：
-/// 模型点名的那一版下有几条分配记录。理由不是分配记录墙的不数；模型答不了这一步（答案本身就是对不上的那一格）也不数。
-fn allocation_records_counted_for_the_wall(
-    reason: ObservedRefusalReason,
-    answer: Option<&ModelAnswer>,
-    publishes_completed: usize,
-    devices: &[(DeviceIdentity, HistoryDevice)],
-    device_width: HistoryDeviceWidth,
-) -> Option<u64> {
-    let ObservedRefusalReason::Explained(ModelRefusalReason::AllocationRecordNodeWall) = reason
-    else {
-        return None;
-    };
-    let root = answer?.root_whose_allocation_records_the_wall_counts(publishes_completed)?;
-    allocation_records_on_the_image_under(&image_of(devices, device_width), root)
-}
-
 /// 起点段里 mkfs 之后那三步：取号、暖机、第一个文件。任一步返回错误就交回是哪一步、报的什么。
 /// 写入口借着 `devices`，交回的错里因此不带镜像——镜像由调用方在写入口放手之后自己取。
 fn acquire_warm_up_and_publish_the_first_file(
@@ -1603,6 +1583,9 @@
     pub mount_publishes_compared: u64,
     pub mounts_with_previous_records_unreadable: u64,
     pub highest_checkpoint_txg: u64,
+    /// 一步之后可写会话的分配器里最多有几条分配记录（两块盘合计）：分配记录树按位置寻址之后没有一个节点 812 条那道墙
+    /// （D8（核心索引结构） 已定项 14），这个数越过 812 说明历史走过了原先那道墙。
+    pub most_allocation_records_in_one_version: u64,
     pub histories_that_turned_the_root_ring: u64,
     pub checker_runs: u64,
     /// 这一步一个写都没发（录制流一步没多）：镜像逐字节不变，checker 的结论沿用上一次，不重跑。
@@ -1668,6 +1651,9 @@
         self.highest_checkpoint_txg = self
             .highest_checkpoint_txg
             .max(other.highest_checkpoint_txg);
+        self.most_allocation_records_in_one_version = self
+            .most_allocation_records_in_one_version
+            .max(other.most_allocation_records_in_one_version);
         self.histories_that_turned_the_root_ring += other.histories_that_turned_the_root_ring;
         self.checker_runs += other.checker_runs;
         self.checker_runs_skipped_because_nothing_was_written +=
@@ -1777,13 +1763,14 @@
         let mut text = String::new();
         let _ = writeln!(
             text,
-            "历史 {} 段：跑完 {}、以已知红收尾 {:?}、新发现 {}；根环转过一圈的 {} 段；最高 txg {}",
+            "历史 {} 段：跑完 {}、以已知红收尾 {:?}、新发现 {}；根环转过一圈的 {} 段；最高 txg {}；一版里最多 {} 条分配记录",
             self.histories,
             self.histories_completed,
             self.histories_ended_known_red,
             self.histories_ended_new_finding,
             self.histories_that_turned_the_root_ring,
-            self.highest_checkpoint_txg
+            self.highest_checkpoint_txg,
+            self.most_allocation_records_in_one_version
         );
         for kind in HistoryOperationKind::ALL {
             let tally = self
@@ -1847,7 +1834,7 @@
         let counts = &self.model_counts;
         let _ = writeln!(
             text,
-            "  模型对拍 {} 步：该拒而拒 {}、区间里拒 {}、该成而成 {}；比过根 {} 条、分配记录 {} 条、冷启动内容 {} 次、抬 F 上限 {} 次；回退到 txg = F_生效 > 0 的根做成 {} 次；分配记录墙按镜像上的真条数放行 {} 次；单元区墙按区间放行 {} 次",
+            "  模型对拍 {} 步：该拒而拒 {}、区间里拒 {}、该成而成 {}；比过根 {} 条、分配记录 {} 条、冷启动内容 {} 次、抬 F 上限 {} 次；回退到 txg = F_生效 > 0 的根做成 {} 次；单元区墙按区间放行 {} 次",
             self.model_judged_steps,
             counts.required_refusals_matched,
             counts.permitted_refusals_taken,
@@ -1857,7 +1844,6 @@
             counts.cold_start_contents_compared,
             counts.ceilings_compared,
             counts.rollbacks_accepted_at_the_effective_floor,
-            counts.allocation_record_wall_refusals_over_one_node,
             counts.unit_area_wall_refusals_in_the_interval
         );
         for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
@@ -1927,7 +1913,9 @@
                 placement_refusal_member(refusal)
             )
         }
-        PublishError::AllocationRecordsExceedOneNode { .. } => "AllocationRecordsExceedOneNode",
+        PublishError::AllocationRecordTreeRewriteSetDidNotSettle { .. } => {
+            "AllocationRecordTreeRewriteSetDidNotSettle"
+        }
         PublishError::MultiLevelCodeTwoTreeRefused { .. } => "MultiLevelCodeTwoTreeRefused",
         PublishError::ReleaseNotInMapping { .. } => "ReleaseNotInMapping",
         PublishError::ReleaseTargetNotAllocated { .. } => "ReleaseTargetNotAllocated",
@@ -1946,9 +1934,6 @@
             "MappingEntryLocationsOnTheSameDevice"
         }
         PublishError::ContentExceedsDataUnit { .. } => "ContentExceedsDataUnit",
-        PublishError::ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided { .. } => {
-            "ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided"
-        }
         PublishError::InodeTreeWriteRefused(_) => "InodeTreeWriteRefused",
         PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn { .. } => {
             "FirstFileVersionDoesNotFollowTheVersionItBuildsOn"
@@ -2201,7 +2186,6 @@
     write_time_seconds: u64,
 ) -> AppliedStep {
     let HistoryPool {
-        device_width,
         devices,
         session,
         model,
@@ -2230,37 +2214,22 @@
         &previous_record_bytes,
     ) {
         Ok(output) => settle_file_publish(session, model, answer, &records_before, output),
-        Err(error) => settle_refused_file_publish(
-            model,
-            answer,
-            &error,
-            devices,
-            *device_width,
-            stream,
-            stream_length_before,
-        ),
+        Err(error) => {
+            settle_refused_file_publish(model, answer, &error, stream, stream_length_before)
+        }
     }
 }
 
-/// 第一个文件、覆盖写被拒：成员映射成理由、分配记录墙时从镜像上数准入基数，交模型比。
+/// 第一个文件、覆盖写被拒：成员映射成理由，交模型比。
 fn settle_refused_file_publish(
     model: &mut IdealModel,
     answer: Result<ModelAnswer, ModelDisagreement>,
     error: &PublishError,
-    devices: &[(DeviceIdentity, HistoryDevice)],
-    device_width: HistoryDeviceWidth,
     stream: &SharedStream,
     stream_length_before: usize,
 ) -> AppliedStep {
     let member = publish_error_member(error);
     let reason = refusal_reason_of_publish_error(error);
-    let counted = allocation_records_counted_for_the_wall(
-        reason,
-        answer.as_ref().ok(),
-        0,
-        devices,
-        device_width,
-    );
     let verdict = judge_by_model(
         model,
         answer,
@@ -2271,7 +2240,6 @@
             stream_length_before,
             stream,
             None,
-            counted,
         ),
     );
     AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
@@ -2317,7 +2285,6 @@
     write_time_seconds: u64,
 ) -> AppliedStep {
     let HistoryPool {
-        device_width,
         devices,
         session,
         model,
@@ -2347,15 +2314,9 @@
         session.instance,
     ) {
         Ok(output) => settle_file_publish(session, model, answer, &records_before, output),
-        Err(error) => settle_refused_file_publish(
-            model,
-            answer,
-            &error,
-            devices,
-            *device_width,
-            stream,
-            stream_length_before,
-        ),
+        Err(error) => {
+            settle_refused_file_publish(model, answer, &error, stream, stream_length_before)
+        }
     }
 }
 
@@ -2424,7 +2385,6 @@
                     stream_length_before,
                     stream,
                     None,
-                    None,
                 ),
             );
             AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
@@ -2542,13 +2502,6 @@
         Err(error) => {
             let member = mount_error_member(&error);
             let reason = refusal_reason_of_mount_error(&error);
-            let counted = allocation_records_counted_for_the_wall(
-                reason,
-                Some(&answer),
-                0,
-                &pool.devices,
-                pool.device_width,
-            );
             let verdict = judge_by_model(
                 &mut pool.model,
                 Ok(answer),
@@ -2559,7 +2512,6 @@
                     stream_length_before,
                     &pool.stream,
                     None,
-                    counted,
                 ),
             );
             AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
@@ -2648,7 +2600,6 @@
     choice: FloorTargetChoice,
 ) -> AppliedStep {
     let HistoryPool {
-        device_width,
         devices,
         session,
         model,
@@ -2733,13 +2684,6 @@
         Err(error) => {
             let member = mount_error_member(&error);
             let reason = refusal_reason_of_mount_error(&error);
-            let counted = allocation_records_counted_for_the_wall(
-                reason,
-                answer.as_ref().ok(),
-                publishes_completed,
-                devices,
-                *device_width,
-            );
             let verdict = judge_by_model(
                 model,
                 answer,
@@ -2750,7 +2694,6 @@
                     stream_length_before,
                     stream,
                     reported_ceiling_of_mount_error(&error),
-                    counted,
                 ),
             );
             AppliedStep {
@@ -3047,6 +2990,9 @@
                 tally.highest_checkpoint_txg = tally
                     .highest_checkpoint_txg
                     .max(session.current.root().checkpoint_txg.0);
+                tally.most_allocation_records_in_one_version = tally
+                    .most_allocation_records_in_one_version
+                    .max(u64::try_from(session.allocator.records().len()).expect("条数装得进 u64"));
             }
             let raised_floor = if let StepOutcome::Applied(AppliedEffect::RaisedFloor {
                 new_floor,
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-harness/src/model_comparison.rs tree/crates/singlefs-harness/src/model_comparison.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-harness/src/model_comparison.rs	2026-09-24 18:48:58.241685305 +0000
+++ tree/crates/singlefs-harness/src/model_comparison.rs	2026-09-24 20:07:28.373234637 +0000
@@ -42,9 +42,12 @@
     }
 }
 
+/// 实现的一个单元是模型的哪个角色。分配记录树根之下的节点不对应角色（`None`）：D8（核心索引结构） 已定项 14 按绝对槽号按位置寻址，
+/// 一次发布重写根之下哪几片由这次动了哪些槽定，模型不记落点、不记是哪几片（占槽只取上界）；那几片的分配代、位置与记录归池级 checker 判
+/// （I-1.1 位置、I-3.10 每个节点都有记录）。
 #[must_use]
-pub fn model_unit_role(unit: TransactionUnit) -> ModelUnitRole {
-    match unit {
+pub fn model_unit_role(unit: TransactionUnit) -> Option<ModelUnitRole> {
+    let role = match unit {
         // 模型罩的发布（第一个文件版本 / 覆盖写、写行、暖机空发布）写的文件恒只有一个数据单元：多单元只会从
         // `publish_sequential_write` 那条路径出来，而随机历史与层 0 的固定脚本一次都不调它。
         TransactionUnit::Data(index) => {
@@ -55,7 +58,15 @@
             );
             ModelUnitRole::Data
         }
+        // 模型罩的文件恒一个数据单元、只有 inode 1：extent 树上段只有根兼叶，那个单元内嵌在它的条目里（D8（核心索引结构） 已定项 14），
+        // 没有下段、上段也没有根之下的节点。
         TransactionUnit::ExtentRoot => ModelUnitRole::ExtentRoot,
+        TransactionUnit::ExtentLowerNode(position) => {
+            panic!("模型今天只罩一个数据单元的文件（内嵌、没有下段）：{position:?}")
+        }
+        TransactionUnit::ExtentUpperNodeBelowTheRoot(position) => {
+            panic!("模型今天只罩 inode 1 一个文件（上段只有根兼叶）：{position:?}")
+        }
         // 模型罩的三种发布（第一个文件版本 / 覆盖写、写行、暖机空发布）里 inode 树恒只有最左那一片叶容器：
         // 第二片只会从 `publish_new_inodes` 那条路径出来，而随机历史与层 0 的固定脚本一次都不调它
         // （`history.rs` 的操作集合里没有「建 inode」，`ModelPublishKind` 也只有那三种）。
@@ -69,6 +80,7 @@
         }
         TransactionUnit::InodeRoot => ModelUnitRole::InodeRoot,
         TransactionUnit::AllocationTree => ModelUnitRole::AllocationTree,
+        TransactionUnit::AllocationTreeNodeBelowTheRoot(_) => return None,
         // 模型罩的池两块盘、15 行记账，映射条目恒 6 条：两棵树恒只有一个节点（根兼叶），根之下的节点只在压小容量的
         // 只供测试的开关下、或记账行装不下一个节点的池（80 块盘起）上出现，随机历史与层 0 的固定脚本都不装那个开关。
         TransactionUnit::AccountingTreeNodeBelowTheRoot(position) => {
@@ -84,7 +96,8 @@
         TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
             ModelUnitRole::InstanceTable
         }
-    }
+    };
+    Some(role)
 }
 
 #[must_use]
@@ -103,7 +116,8 @@
     let unit_allocation_records = output
         .units
         .iter()
-        .map(|unit| {
+        .filter_map(|unit| {
+            let role = model_unit_role(unit.identity)?;
             let records = output
                 .allocation_records
                 .iter()
@@ -114,7 +128,7 @@
                     is_released: record.is_released,
                 })
                 .collect();
-            (model_unit_role(unit.identity), records)
+            Some((role, records))
         })
         .collect();
     ObservedRoot {
@@ -177,9 +191,6 @@
         PublishError::PlacementRefused { refusal, .. } => {
             refusal_reason_of_placement_refusal(refusal)
         }
-        PublishError::AllocationRecordsExceedOneNode { .. } => {
-            explained(ModelRefusalReason::AllocationRecordNodeWall)
-        }
         PublishError::ContentExceedsDataUnit { .. } => {
             explained(ModelRefusalReason::ContentExceedsDataUnitPayload)
         }
@@ -190,14 +201,14 @@
             explained(ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile)
         }
         // 释放判定路径的五种：上一版的映射或分配记录与上一版对不上、盘上那条指针的两条位置条目不同槽，健康的历史里不该出现。
-        // extent 树要长内部节点那一条同理：随机历史一次都不调 `publish_sequential_write`，写的文件恒一个数据单元，它出现就是对不上。
+        // 分配记录树重写集合迭代不收敛那一条同理：只有强制复用窗口为 0 的只供测试的开关下才可能走到，随机历史不装那个开关。
         // inode 树写入被拒那一条同样：随机历史一次都不调 `publish_new_inodes`，
         // 而它跑的那几种发布每次最多改一片叶容器、重写的角色最多九个。
         // 多层码 2 树算不出形状那一条同理：两棵树装不下一个节点时分裂、不拒，只有长到 257 层或上一版的形状按分隔 key 走不通才拒，
         // 随机历史里两棵树恒只有一个节点；它出现就是模型与实现对不上。
         PublishError::MultiLevelCodeTwoTreeRefused { .. }
         | PublishError::InodeTreeWriteRefused(_)
-        | PublishError::ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided { .. }
+        | PublishError::AllocationRecordTreeRewriteSetDidNotSettle { .. }
         | PublishError::ReleaseNotInMapping { .. }
         | PublishError::ReleaseTargetNotAllocated { .. }
         | PublishError::ReleaseTargetAlreadyReleased { .. }
@@ -397,7 +408,7 @@
         ]
     }
 
-    /// 单元区墙的区间开着时（单元区只有 640 槽的小盘，第一个文件之后占槽上界 13 × 64 > 640），发布的落点被拒：每块盘上都没有（容量不够）
+    /// 单元区墙的区间开着时（单元区只有 640 槽的小盘，第一个文件之后占槽上界 17 × 64 > 640），发布的落点被拒：每块盘上都没有（容量不够）
     /// 映射成单元区墙、模型放行；小盘写满、各盘落点不一致是第一版不支持的池形状，映射成模型没有的理由、判「模型说该成、实现拒了」
     /// （增补 3 第 2 件代码三方第一轮判决第三节第 2 条：此前 `NoSpaceFor` 装着这四种、一律映射成单元区墙，区间一开就被接走）。
     #[test]
@@ -426,7 +437,6 @@
                 publishes_completed: 0,
                 wrote_anything: false,
                 reported_ceiling: None,
-                allocation_records_counted_on_the_image: None,
             };
             let judged = model
                 .clone()
diff -ruN -x target /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-harness/src/model.rs tree/crates/singlefs-harness/src/model.rs
--- /tmp/claude-1000/m2-final-code-r2/tree/crates/singlefs-harness/src/model.rs	2026-09-24 16:50:54.714341234 +0000
+++ tree/crates/singlefs-harness/src/model.rs	2026-09-24 20:56:53.162716220 +0000
@@ -13,16 +13,17 @@
 //!
 //! 模型不记落点：单元落在哪个槽、分配记录写得对不对（回收门槛、释放时改没改写记录、复用时罩住的槽删没删）归池级 checker 判，
 //! 模型只拿实现交回的每个单元那几条记录比「每块盘一条、仍分配、分配代等于写它的那次发布」（增补 3 第 2 件代码三方第一轮判决第一节 M4 那一格：
-//! 回收门槛差一、释放时不改写记录这几条变异，只留模型时三段都判不出，checker 都判红）。分配记录的真条数模型同样不记：分配记录墙拒时，
-//! 执行器按 checker 的解析从镜像上现数，交给模型判区间的下端（同一判决第三节第 1 条）。
+//! 回收门槛差一、释放时不改写记录这几条变异，只留模型时三段都判不出，checker 都判红）。分配记录的真条数模型同样不记；分配记录树按绝对槽号
+//! 按位置寻址之后（D8（核心索引结构） 已定项 14）没有「一个节点装不下」那道墙，模型也就没有那条拒绝理由。
 
 use std::collections::{BTreeMap, BTreeSet};
 use std::rc::Rc;
 
 use singlefs_format::{
-    index_node_header_bytes, ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, ALLOCATION_RECORD_BYTES,
-    ALLOCATION_RECORD_KEY_BYTES, CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES, DATA_UNIT_PAYLOAD_OFFSET,
-    INSTANCE_TABLE_PAGE_RECORDS, NODE_BYTES, ROOT_RING_REGIONS, ROOT_RING_REGION_DEVICES,
+    index_node_header_bytes, ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES,
+    ALLOCATION_RECORD_TREE_INTERNAL_FANOUT, ALLOCATION_RECORD_TREE_LEAF_SLOTS,
+    CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES, DATA_UNIT_PAYLOAD_OFFSET, INSTANCE_TABLE_PAGE_RECORDS,
+    NODE_BYTES, ROOT_RING_REGIONS, ROOT_RING_REGION_DEVICES,
     ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM, SLOT_BYTES, UNIT_AREA_START_SLOT,
 };
 
@@ -135,22 +136,8 @@
         }
     }
 
-    /// 这次发布往分配记录树里加几条（每块盘一条）：重写的每个角色各一条。
-    /// 零单元发布一个字节都不写，一条不加；树表 0 条的一版上写行那次**建起这一版自己的分配记录树**
-    /// （C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案），实例表与那片节点各一条。
-    fn allocation_records_added_per_device(self) -> u64 {
-        match self {
-            ModelPublishKind::FirstFileVersion
-            | ModelPublishKind::OverwriteFileVersion
-            | ModelPublishKind::RowsOnFileVersion
-            | ModelPublishKind::EmptyOnFileVersion
-            | ModelPublishKind::RowsOnVersionWithoutFile => {
-                u64::try_from(self.rewritten_roles().len()).expect("至多九个角色")
-            }
-            ModelPublishKind::ZeroUnit => 0,
-        }
-    }
-
+    /// 这次发布重写的角色。分配记录树（`ModelUnitRole::AllocationTree`）在这里只记它的根：根之下重写几个节点按位置寻址现算上界
+    /// （`IdealModel::allocation_record_tree_nodes_rewritten_upper_bound`），模型不记是哪几个。
     fn rewritten_roles(self) -> &'static [ModelUnitRole] {
         match self {
             ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion => &[
@@ -215,13 +202,7 @@
     /// 这一版每个角色的单元是哪次发布写的：它的分配记录的分配代就是这个 txg（D3（空间分配） 已定项 3 / 7：value = 分配代；
     /// COW：单元只在分配它的那次发布里写）。mkfs 写的实例表与第 0 版树表记 txg 0。
     pub role_written_at: BTreeMap<ModelUnitRole, ModelCheckpointTxg>,
-    /// 这一版之后分配记录条数的上界：mkfs 两个单元每盘各一条，沿这一版的来路每次发布每个重写的角色每盘至多加一条
-    /// （D3（空间分配） 已定项 7：一条记一个单元、释放只改写不删）。回收、复用只会让真数比它小。
-    pub allocation_records_upper_bound: u64,
-    /// 写出这一版的那次发布按准入的口径新增几条分配记录：重写的每个角色每盘一条，不抵扣会被复用的已回收记录（增补 2 收口表第 39 行，
-    /// 2026-09-18 用户定保留这个上界准入）。第 0 代根记 mkfs 写的两个单元每盘一条。
-    pub allocation_records_added_by_its_publish: u64,
-    /// 同样口径的每盘已占槽数上界：沿来路每次发布写出的单元的槽数之和，释放与回收一概不扣。
+    /// 每盘已占槽数的上界：沿来路每次发布写出的单元的槽数之和，释放与回收一概不扣（分配记录树每次重写的节点数取按位置寻址现算的上界）。
     pub occupied_slots_upper_bound_per_device: u64,
 }
 
@@ -289,8 +270,6 @@
     /// 第一个文件版本要建在上面的那一版已经有过文件版本：那条路径按「树还没建起来」写，树表条目的诞生 txg 与
     /// inode 1 的对象出生代都会取这次的 txg，与环里那些旧根记的对不上（I-9.14、I-9.10）。再写一版走覆盖写。
     FirstFileVersionOnAVersionThatAlreadyHasAFile,
-    /// 分配记录树第一版只有一个节点（容量墙，收口表第 39 行：模型答允许拒绝的区间）。
-    AllocationRecordNodeWall,
     /// 单元区装不下（D28（挂载期承诺量） 已定项 1 的准入；模型答允许拒绝的区间）。
     UnitAreaWall,
     /// 回退目标不在根环里（D23（journal 的角色与格式） 已定项 14：候选集是根环里的根）。
@@ -318,7 +297,6 @@
             ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile => {
                 "要建在上面的那一版已经有过文件版本（再写一版走覆盖写）"
             }
-            ModelRefusalReason::AllocationRecordNodeWall => "分配记录树一个节点装不下",
             ModelRefusalReason::UnitAreaWall => "单元区装不下",
             ModelRefusalReason::RollbackTargetNotInRing => "回退目标不在根环里",
             ModelRefusalReason::RollbackTargetBelowEffectiveFloor => "回退目标低于 F_生效",
@@ -333,7 +311,7 @@
     /// 容量墙：模型只答允许拒绝的区间（按这一步计划的每次发布之后的上界判），不答必须拒。
     fn is_capacity_wall_with_an_interval(self) -> bool {
         match self {
-            ModelRefusalReason::AllocationRecordNodeWall | ModelRefusalReason::UnitAreaWall => true,
+            ModelRefusalReason::UnitAreaWall => true,
             ModelRefusalReason::ContentExceedsDataUnitPayload
             | ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn
             | ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile
@@ -370,12 +348,9 @@
     ColdStartRecover,
 }
 
-/// 一步里计划的一次发布之后的两个上界（容量墙区间的上端按它判），与这次发布按准入口径新增的分配记录条数（分配记录墙区间的下端：
-/// 从镜像上现数的基数加上它）。
+/// 一步里计划的一次发布之后的上界（单元区墙区间的上端按它判）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct PlannedPublishUpperBounds {
-    allocation_records: u64,
-    allocation_records_added_by_the_admission: u64,
     occupied_slots_per_device: u64,
 }
 
@@ -398,29 +373,9 @@
     planned_publish_upper_bounds: Vec<PlannedPublishUpperBounds>,
     /// 容量墙在第一次写之前一串判完（可写挂载、回退）还是逐次发布判（发布、抬 F）。
     walls_are_judged_before_the_first_write: bool,
-    /// 这一步接在哪一版后面：发布与抬 F 是会话的现行版本，可写挂载是所选根，回退是目标根；冷启动与候选集外的回退没有。
-    starting_version: Option<ModelRootKey>,
     session_after_success: ModelSessionAfterSuccess,
 }
 
-impl ModelAnswer {
-    /// 分配记录墙拒在做完 `publishes_completed` 次发布之后时，执行器在镜像上数哪一条根下的分配记录：逐次判的（发布、抬 F）数拒之前
-    /// 最后写出的那一版（一次都没做完就是这一步的起点），一串判完的（可写挂载、回退）数这一步的起点——实现的准入基数就是这两处的
-    /// 分配器条数（`transaction::publish_admission`、`transaction::publish_sequence_admission`）。
-    #[must_use]
-    pub fn root_whose_allocation_records_the_wall_counts(
-        &self,
-        publishes_completed: usize,
-    ) -> Option<ModelRootKey> {
-        if self.walls_are_judged_before_the_first_write || publishes_completed == 0 {
-            return self.starting_version;
-        }
-        self.expected_roots
-            .get(publishes_completed - 1)
-            .map(|root| root.key)
-    }
-}
-
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum ModelSessionAfterSuccess {
     /// 会话照旧开着，现行版本换成最后写出的那条根。
@@ -504,9 +459,6 @@
         wrote_anything: bool,
         /// 实现报的上限（抬 F 被上限拒时）。
         reported_ceiling: Option<ModelCheckpointTxg>,
-        /// 理由是分配记录墙时，执行器按 checker 的解析在镜像上数的准入基数：`ModelAnswer::root_whose_allocation_records_the_wall_counts`
-        /// 点名的那条根下有几条分配记录。别的理由不数；数不出（镜像上找不到那条根、树读不出）也是 None，这时模型不放行分配记录墙。
-        allocation_records_counted_on_the_image: Option<u64>,
     },
 }
 
@@ -589,8 +541,6 @@
     pub ceilings_compared: u64,
     /// 回退做成、目标的 txg 正好等于 F_生效且 F_生效 > 0（B2 那一格跑到了）。
     pub rollbacks_accepted_at_the_effective_floor: u64,
-    /// 分配记录墙拒、镜像上数的真条数越过一个节点而放行的次数（墙那一格的下端真的判过）。
-    pub allocation_record_wall_refusals_over_one_node: u64,
     /// 单元区墙拒（实现报每块盘上都没有合政策的落点）、在允许拒绝的区间里而放行的次数（小盘那一段的落点拒绝真的走到了、判过）。
     pub unit_area_wall_refusals_in_the_interval: u64,
 }
@@ -607,8 +557,6 @@
         self.ceilings_compared += other.ceilings_compared;
         self.rollbacks_accepted_at_the_effective_floor +=
             other.rollbacks_accepted_at_the_effective_floor;
-        self.allocation_record_wall_refusals_over_one_node +=
-            other.allocation_record_wall_refusals_over_one_node;
         self.unit_area_wall_refusals_in_the_interval +=
             other.unit_area_wall_refusals_in_the_interval;
     }
@@ -639,7 +587,6 @@
     /// 没有可写会话。
     #[must_use]
     pub fn after_make_filesystem(geometry: ModelPoolGeometry) -> Self {
-        let device_count = u64::try_from(geometry.devices.len()).expect("盘数");
         let genesis = ModelRoot {
             key: ModelRootKey {
                 checkpoint_txg: MAKE_FILESYSTEM_TXG,
@@ -653,8 +600,6 @@
                 (ModelUnitRole::InstanceTable, MAKE_FILESYSTEM_TXG),
                 (ModelUnitRole::TreeTable, MAKE_FILESYSTEM_TXG),
             ]),
-            allocation_records_upper_bound: 2 * device_count,
-            allocation_records_added_by_its_publish: 2 * device_count,
             occupied_slots_upper_bound_per_device: ModelUnitRole::InstanceTable.span_in_slots()
                 + ModelUnitRole::TreeTable.span_in_slots(),
         };
@@ -802,9 +747,58 @@
         Some(newest_on_every_device.min(fourth_newest_non_empty))
     }
 
-    /// 分配记录树一个节点装几条（D3（空间分配） 已定项 11：key 10、条目 20）。
-    fn allocation_record_node_capacity() -> u64 {
-        index_node_entry_capacity(ALLOCATION_RECORD_KEY_BYTES, ALLOCATION_RECORD_BYTES)
+    /// 分配记录树一次发布至多重写几个节点（每个节点每盘占一个槽）。
+    ///
+    /// 树的形状照 D8（核心索引结构） 已定项 14 另算一份：按绝对槽号按位置寻址，叶罩 812 个槽，内部节点罩 169 个孩子那么宽的一段；
+    /// 根按盘分路，根的层号取「每块盘按下一层的宽度切出来的段数之和装得进一个根」的最低一层（实现员的取法，交回里写明）。
+    ///
+    /// 重写的是根，与这次之后还装着记录、内容变了的节点。模型不记哪几片变了，取「这次之后可能装着记录的节点」的全数：
+    /// 记录只住在落点上；落点按 D3（空间分配） 已定项 10 从最低处取（开新段取最低的全空段，回落取最低的空槽），每个落点至多多开一个
+    /// 64 槽的段 ⇒ 沿来路写过 P 个槽之后，落点都在单元区起点往后 `64 × P` 个槽里（单元区墙那一格同一个读法）。
+    /// 这次重写的节点自己也占槽，P 里含着要求的那个数：设重写数为 A，A ≤ g(A)（g 是「写过 P₀ + A 个槽之后可能装着记录的节点数」，
+    /// 对 A 单调不减），从 g(∞)（单元区里的全部节点）起往下迭代，每一步都仍是 A 的上界，停在不动点。
+    fn allocation_record_tree_nodes_rewritten_upper_bound(
+        &self,
+        occupied_slots_before_the_tree_nodes: u64,
+    ) -> u64 {
+        let device_slots = self.geometry.device_size_in_bytes / SLOT_BYTES;
+        let device_count = u64::try_from(self.geometry.devices.len()).expect("盘数");
+        let span_at_level = |level: u32| {
+            ALLOCATION_RECORD_TREE_LEAF_SLOTS
+                .saturating_mul(ALLOCATION_RECORD_TREE_INTERNAL_FANOUT.saturating_pow(level))
+        };
+        let root_level = (1_u32..)
+            .find(|level| {
+                device_count * device_slots.div_ceil(span_at_level(level - 1))
+                    <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
+            })
+            .expect("层号往上走，每块盘切出来的段数终归是 1，盘数不到 169 就装得进一个根");
+        let nodes_that_may_hold_records = |occupied_slots: u64| {
+            let reach = UNIT_AREA_START_SLOT
+                .saturating_add(CLUSTER_SEGMENT_SLOTS.saturating_mul(occupied_slots))
+                .min(device_slots);
+            let per_device: u64 = if reach <= UNIT_AREA_START_SLOT {
+                0
+            } else {
+                (0..root_level)
+                    .map(|level| {
+                        let span = span_at_level(level);
+                        (reach - 1) / span - UNIT_AREA_START_SLOT / span + 1
+                    })
+                    .sum()
+            };
+            1 + device_count * per_device
+        };
+        let mut bound = nodes_that_may_hold_records(u64::MAX);
+        loop {
+            let next = nodes_that_may_hold_records(
+                occupied_slots_before_the_tree_nodes.saturating_add(bound),
+            );
+            if next >= bound {
+                return bound;
+            }
+            bound = next;
+        }
     }
 
     /// 一次写记账行的发布要几行、一个节点装几行（D5（快照 / 空间记账机制） 已定项 8）。
@@ -839,13 +833,17 @@
             checkpoint_txg,
             instance,
         };
-        let device_count = u64::try_from(self.geometry.devices.len()).expect("盘数");
         let mut role_written_at = previous.role_written_at.clone();
         let mut slots_written = 0;
+        let mut rewrites_the_allocation_record_tree = false;
         let rewritten = kind.rewritten_roles();
         for role in rewritten {
             role_written_at.insert(*role, checkpoint_txg);
-            slots_written += role.span_in_slots();
+            if *role == ModelUnitRole::AllocationTree {
+                rewrites_the_allocation_record_tree = true;
+            } else {
+                slots_written += role.span_in_slots();
+            }
         }
         // 实例表链多于一片时第 1 片起每一片也是一个重写的单元：`rewritten_roles` 里实例表只记一个角色，多出来的几片在这里补上。
         let instance_table_pages_after_the_first = if kind.rewrites_the_instance_table() {
@@ -855,9 +853,11 @@
         };
         slots_written +=
             instance_table_pages_after_the_first * ModelUnitRole::InstanceTable.span_in_slots();
-        let allocation_records_added = (kind.allocation_records_added_per_device()
-            + instance_table_pages_after_the_first)
-            * device_count;
+        if rewrites_the_allocation_record_tree {
+            slots_written += self.allocation_record_tree_nodes_rewritten_upper_bound(
+                previous.occupied_slots_upper_bound_per_device + slots_written,
+            ) * ModelUnitRole::AllocationTree.span_in_slots();
+        }
         let file = match new_file_content {
             Some(content) => Some(ModelFileVersion {
                 written_by: key,
@@ -872,9 +872,6 @@
             file,
             instance_table_rows,
             role_written_at,
-            allocation_records_upper_bound: previous.allocation_records_upper_bound
-                + allocation_records_added,
-            allocation_records_added_by_its_publish: allocation_records_added,
             occupied_slots_upper_bound_per_device: previous.occupied_slots_upper_bound_per_device
                 + slots_written,
         }
@@ -884,9 +881,6 @@
         roots
             .iter()
             .map(|root| PlannedPublishUpperBounds {
-                allocation_records: root.allocation_records_upper_bound,
-                allocation_records_added_by_the_admission: root
-                    .allocation_records_added_by_its_publish,
                 occupied_slots_per_device: root.occupied_slots_upper_bound_per_device,
             })
             .collect()
@@ -938,7 +932,6 @@
         );
         Ok(self.answer_for_publishes(
             ModelOperationKind::PublishFirstFile,
-            current.key,
             required_refusals,
             BTreeSet::new(),
             vec![root],
@@ -979,7 +972,6 @@
         );
         Ok(self.answer_for_publishes(
             ModelOperationKind::PublishOverwrite,
-            current.key,
             required_refusals,
             BTreeSet::new(),
             vec![root],
@@ -1012,7 +1004,6 @@
         );
         Ok(self.answer_for_publishes(
             ModelOperationKind::PublishWithoutUnits,
-            current.key,
             BTreeSet::new(),
             BTreeSet::new(),
             vec![root],
@@ -1022,7 +1013,6 @@
     fn answer_for_publishes(
         &self,
         operation: ModelOperationKind,
-        starting_version: ModelRootKey,
         required_refusals: BTreeSet<ModelRefusalReason>,
         permitted_refusals: BTreeSet<ModelRefusalReason>,
         roots: Vec<ModelRoot>,
@@ -1032,11 +1022,11 @@
             root.role_written_at.get(&ModelUnitRole::AccountingTree)
                 == Some(&root.key.checkpoint_txg)
         });
-        // 记账行装不下一个节点时记账树分裂、不拒（D8（核心索引结构） 已定项 11），分裂多出来的节点每个各占一条分配记录；
-        // 模型的分配记录账按「记账树一个节点」数，只罩行数装得进一个节点的池（盘数不到 80）——超出就是模型没罩到，不是实现错。
+        // 记账行装不下一个节点时记账树分裂、不拒（D8（核心索引结构） 已定项 11），分裂多出来的节点每个各占一个槽；
+        // 模型的占槽上界按「记账树一个节点」数，只罩行数装得进一个节点的池（盘数不到 80）——超出就是模型没罩到，不是实现错。
         assert!(
             !(writes_accounting_rows && self.accounting_rows_exceed_one_node()),
-            "模型今天只罩记账树一个节点的池：分裂多出来的节点模型的分配记录账没算"
+            "模型今天只罩记账树一个节点的池：分裂多出来的节点模型的占槽上界没算"
         );
         ModelAnswer {
             operation,
@@ -1048,7 +1038,6 @@
             expected_read_back: None,
             rollback_floor_ceiling: None,
             walls_are_judged_before_the_first_write: false,
-            starting_version: Some(starting_version),
             session_after_success: ModelSessionAfterSuccess::StaysOpen,
         }
     }
@@ -1084,7 +1073,6 @@
             rollback_floor_ceiling: None,
             planned_publish_upper_bounds: Vec::new(),
             walls_are_judged_before_the_first_write: true,
-            starting_version: None,
             session_after_success: ModelSessionAfterSuccess::Closed,
         };
         let Some(target_root) = self.root_with_key(target) else {
@@ -1216,13 +1204,8 @@
             ));
             warm_up_publishes += 1;
         }
-        let mut answer = self.answer_for_publishes(
-            operation,
-            base.key,
-            required_refusals,
-            BTreeSet::new(),
-            roots,
-        );
+        let mut answer =
+            self.answer_for_publishes(operation, required_refusals, BTreeSet::new(), roots);
         answer.walls_are_judged_before_the_first_write = true;
         answer.expected_mount = Some((instance, rows_to_write));
         answer.session_after_success = ModelSessionAfterSuccess::OpenedAs(instance);
@@ -1304,7 +1287,6 @@
         }
         let mut answer = self.answer_for_publishes(
             ModelOperationKind::RaiseRollbackFloor,
-            current.key,
             required_refusals,
             permitted_refusals,
             roots,
@@ -1334,20 +1316,12 @@
             rollback_floor_ceiling: None,
             planned_publish_upper_bounds: Vec::new(),
             walls_are_judged_before_the_first_write: true,
-            starting_version: None,
             session_after_success: ModelSessionAfterSuccess::Closed,
         }
     }
 
-    /// 容量墙的区间（收口表第 39 行那种：条款把答案留给实现取上界，模型答允许拒绝的区间）：
-    /// - 分配记录树（预想，跟收口表第 39 行）：允许拒要两头都过。上界这一头：这一步计划的那次发布之后、沿来路每次发布每个角色每盘都新加一条的
-    ///   上界超过一个节点——一条记一个单元、释放只改写不删（D3（空间分配） 已定项 7），条目 20 字节、key 10（D3（空间分配） 已定项 11），
-    ///   节点 16 KiB 减头（D8（核心索引结构） 已定项 11）⇒ 812 条；模型的上界沿来路累加、不看分配器此刻的条数，只会比真数宽。
-    ///   真条数这一头：执行器按 checker 的解析在镜像上现数的准入基数（`ModelAnswer::root_whose_allocation_records_the_wall_counts` 点名的那一版），
-    ///   加上计划里到那一次为止每次按准入口径新增的条数（每个重写的角色每盘一条，2026-09-18 用户定保留的上界准入），超过 812 才算装不下；
-    ///   真条数 ≤ 812 而实现拒了是对不上，数不出基数也不放行（增补 3 第 2 件代码三方第一轮判决第三节第 1 条：此前只有上界那一头，
-    ///   宽到接得住「差一」的误拒——攻方把墙的 `>` 改成 `>=`，长历史里在条款说装得下的格上拒了 44 次，那一刻上界 1376–1778，一次都没判出）。
-    ///   必须拒那一头（真条数装不下而实现做成了）不判：装不下还去写会 panic，由第 1 件判。
+    /// 容量墙的区间（收口表第 39 行那种：条款把答案留给实现取上界，模型答允许拒绝的区间）。分配记录树按位置寻址之后
+    /// （D8（核心索引结构） 已定项 14）没有「一个节点装不下」那道墙，剩单元区这一道：
     /// - 单元区：真条数那一头不判；上界那一头是占槽上界 × 一个聚簇段的槽数超过单元区（每个落点最坏独占一段：已分配 + defer ≤ 2 × 上界，
     ///   保留池 10 + 7 c_max 与切换预留（D16（发布语义） 已定项 1；D28（挂载期承诺量） 已定项 3）在 64 倍里）。预想：D28 已定项 1 的准入式子
     ///   第一版没实现，各项没有现值。
@@ -1356,7 +1330,6 @@
         reason: ModelRefusalReason,
         answer: &ModelAnswer,
         publishes_completed: usize,
-        allocation_records_counted_on_the_image: Option<u64>,
     ) -> bool {
         // 一串判完的（可写挂载、回退）看计划里的每一次；逐次判的（发布、抬 F）只看拒的那一次。
         let judged_publishes: &[PlannedPublishUpperBounds] =
@@ -1369,23 +1342,6 @@
                     .unwrap_or(&[])
             };
         match reason {
-            ModelRefusalReason::AllocationRecordNodeWall => {
-                let capacity = Self::allocation_record_node_capacity();
-                let upper_bound_exceeds = judged_publishes
-                    .iter()
-                    .any(|planned| planned.allocation_records > capacity);
-                let true_count_exceeds =
-                    allocation_records_counted_on_the_image.is_some_and(|counted| {
-                        judged_publishes
-                            .iter()
-                            .scan(counted, |records, planned| {
-                                *records += planned.allocation_records_added_by_the_admission;
-                                Some(*records)
-                            })
-                            .any(|records_after_the_publish| records_after_the_publish > capacity)
-                    });
-                upper_bound_exceeds && true_count_exceeds
-            }
             ModelRefusalReason::UnitAreaWall => judged_publishes.iter().any(|planned| {
                 planned.occupied_slots_per_device * CLUSTER_SEGMENT_SLOTS
                     > self.unit_area_slots_per_device()
@@ -1418,7 +1374,6 @@
                 publishes_completed,
                 wrote_anything,
                 reported_ceiling,
-                allocation_records_counted_on_the_image,
             } => {
                 self.judge_reported_ceiling(answer, *reported_ceiling, &mut counts)?;
                 let accepted = match reason {
@@ -1431,7 +1386,6 @@
                                         *candidate,
                                         answer,
                                         *publishes_completed,
-                                        *allocation_records_counted_on_the_image,
                                     ))
                         })
                     }
@@ -1443,14 +1397,10 @@
                     } else {
                         ModelDisagreementAspect::RefusalReason
                     };
-                    let counted_on_the_image = match allocation_records_counted_on_the_image {
-                        Some(counted) => format!("（镜像上数的准入基数 {counted} 条）"),
-                        None => String::new(),
-                    };
                     return Err(ModelDisagreement::new(
                         aspect,
                         describe_answer(answer),
-                        format!("拒了：{member}{counted_on_the_image}"),
+                        format!("拒了：{member}"),
                     ));
                 };
                 let partial_publishes_allowed = !answer.walls_are_judged_before_the_first_write
@@ -1481,9 +1431,6 @@
                 } else {
                     counts.permitted_refusals_taken += 1;
                 }
-                if accepted_reason == ModelRefusalReason::AllocationRecordNodeWall {
-                    counts.allocation_record_wall_refusals_over_one_node += 1;
-                }
                 if accepted_reason == ModelRefusalReason::UnitAreaWall {
                     counts.unit_area_wall_refusals_in_the_interval += 1;
                 }
@@ -1966,8 +1913,9 @@
     }
 
     /// 实例表多于一片（用户 2026-09-24 定尾片先、一片写满 369 行再开下一片）：可写挂载不再拒，写行那次发布整条链重写，
-    /// 链上每一片都是一个重写的单元、每盘各加一条分配记录。第一个文件之后号推到 370（等于连着 369 次取号之后崩溃），
-    /// 下一次可写挂载取 371、写 [1, 371) 共 370 行 ⇒ 两片：写行那次发布加 (两片 + 四个固定点单元) × 2 盘 = 12 条、占 2 × 2 + 4 = 8 槽。
+    /// 链上每一片都是一个重写的单元、各占两个槽。第一个文件之后号推到 370（等于连着 369 次取号之后崩溃），
+    /// 下一次可写挂载取 371、写 [1, 371) 共 370 行 ⇒ 两片：写行那次发布占 2 × 2 + 3（记账树、映射树、树表）槽，
+    /// 加分配记录树这次至多重写的节点数。
     #[test]
     fn a_mount_that_writes_more_rows_than_one_page_holds_counts_every_page_of_the_instance_table_chain(
     ) {
@@ -1988,15 +1936,45 @@
         );
         let row_publish = &mount.expected_roots[0];
         assert_eq!(row_publish.instance_table_rows.len(), 370);
+        let slots_before = model.newest_root().occupied_slots_upper_bound_per_device;
+        let slots_outside_the_allocation_record_tree = 2 * 2 + 3;
+        assert_eq!(
+            row_publish.occupied_slots_upper_bound_per_device - slots_before,
+            slots_outside_the_allocation_record_tree
+                + model.allocation_record_tree_nodes_rewritten_upper_bound(
+                    slots_before + slots_outside_the_allocation_record_tree
+                ),
+            "两片实例表各 2 槽、三个固定点单元各 1 槽，加分配记录树这次至多重写的节点"
+        );
+    }
+
+    /// 分配记录树一次发布至多重写几个节点（D8（核心索引结构） 已定项 14：叶罩 812 槽、内部扇出 169）：4 GiB 两块盘，根在第 2 层；
+    /// 第一个文件版本之前每盘写过 3 槽（mkfs 的实例表 2、树表 1），这次树之外写 9 槽（数据 2、extent 根 1、inode 叶 2、inode 根 1、
+    /// 记账树、映射树、树表各 1）。设这次重写 A 个节点：落点都在单元区起点 50176 往后 64 × (12 + A) 槽里。
+    /// A = 9 时那一段到 51520，罩叶 61..=63 三片、第 1 层 1 个 ⇒ 1 + 2 × 4 = 9，是不动点；从单元区全部 529 个节点往下迭代停在这里。
+    #[test]
+    fn allocation_record_tree_nodes_rewritten_by_the_first_file_version_on_4_gib_are_at_most_nine()
+    {
+        let model = two_device_model();
         assert_eq!(
-            row_publish.allocation_records_added_by_its_publish, 12,
-            "两片实例表加四个固定点单元，每盘一条"
+            model.allocation_record_tree_nodes_rewritten_upper_bound(u64::MAX / 128),
+            1 + 2 * (262 + 2),
+            "单元区里的全部节点：每盘叶 61..=322、第 1 层 0..=1，加根"
         );
+        assert_eq!(
+            model.allocation_record_tree_nodes_rewritten_upper_bound(3 + 9),
+            9
+        );
+        let mut model = model;
+        model.acquire_and_warm_up_in_the_make_filesystem_process();
         let slots_before = model.newest_root().occupied_slots_upper_bound_per_device;
+        assert_eq!(slots_before, 3, "暖机是零单元发布，一个槽都不占");
+        let first = model
+            .answer_publish_first_file(&[3])
+            .expect("会话开着、现行 txg 2");
         assert_eq!(
-            row_publish.occupied_slots_upper_bound_per_device - slots_before,
-            8,
-            "两片实例表各 2 槽、四个固定点单元各 1 槽"
+            first.expected_roots[0].occupied_slots_upper_bound_per_device,
+            3 + 9 + 9
         );
     }
 
@@ -2154,136 +2132,4 @@
             .expect_err("照抄的数据单元分配代写成了这次的 txg");
         assert_eq!(wrong.aspect, ModelDisagreementAspect::AllocationGeneration);
     }
-
-    fn refused_by_the_allocation_record_wall(
-        publishes_completed: usize,
-        allocation_records_counted_on_the_image: Option<u64>,
-    ) -> ObservedOutcome {
-        ObservedOutcome::Refused {
-            member: "PublishError::AllocationRecordsExceedOneNode".to_string(),
-            reason: ObservedRefusalReason::Explained(ModelRefusalReason::AllocationRecordNodeWall),
-            publishes_completed,
-            wrote_anything: publishes_completed > 0,
-            reported_ceiling: None,
-            allocation_records_counted_on_the_image,
-        }
-    }
-
-    /// 在第二个实例里再覆盖写 50 次：沿来路的上界 102 + 50 × 16 = 902，远过 812（区间的上界那一头早就开了）。
-    fn model_with_the_upper_bound_far_above_one_allocation_node() -> IdealModel {
-        let mut model = model_after_four_overwrites_in_a_second_instance();
-        for content_byte in 0_u8..50 {
-            let overwrite = model
-                .answer_publish_overwrite(&[content_byte])
-                .expect("会话开着、现行版本带文件");
-            succeed(&mut model, &overwrite);
-        }
-        model
-    }
-
-    /// 分配记录墙（增补 3 第 2 件代码三方第一轮判决第三节第 1 条）：覆盖写每盘加 8 条；镜像上数的基数 796 ⇒ 这次之后正好 812 条、一个节点装得下，
-    /// 墙拒是「模型说该成、实现拒了」，哪怕沿来路的上界早过了 812；基数 797 ⇒ 813 条，放行；数不出基数不放行。
-    #[test]
-    fn the_allocation_record_wall_is_permitted_only_when_the_counted_records_plus_this_publish_exceed_812(
-    ) {
-        let model = model_with_the_upper_bound_far_above_one_allocation_node();
-        let answer = model
-            .answer_publish_overwrite(&[1])
-            .expect("会话开着、现行版本带文件");
-        assert_eq!(IdealModel::allocation_record_node_capacity(), 812);
-        assert!(
-            answer.planned_publish_upper_bounds[0].allocation_records > 812,
-            "上界那一头开着：{:?}",
-            answer.planned_publish_upper_bounds
-        );
-        assert_eq!(
-            answer.planned_publish_upper_bounds[0].allocation_records_added_by_the_admission,
-            16
-        );
-        assert_eq!(
-            answer.root_whose_allocation_records_the_wall_counts(0),
-            Some(model.session.as_ref().expect("会话开着").current.key),
-            "覆盖写数现行版本那一版"
-        );
-        let judged = |counted: Option<u64>| {
-            model
-                .clone()
-                .judge_and_advance(&answer, &refused_by_the_allocation_record_wall(0, counted))
-                .map(|counts| counts.allocation_record_wall_refusals_over_one_node)
-                .map_err(|disagreement| disagreement.aspect)
-        };
-        assert_eq!(
-            judged(Some(796)),
-            Err(ModelDisagreementAspect::RefusedWhenModelRequiresSuccess)
-        );
-        assert_eq!(judged(Some(797)), Ok(1));
-        assert_eq!(
-            judged(None),
-            Err(ModelDisagreementAspect::RefusedWhenModelRequiresSuccess)
-        );
-    }
-
-    /// 可写挂载在第一次写之前一串判完：基数是所选根那一版，加上写行（五个角色每盘一条）与每次暖机（四个角色每盘一条）；一串里有一次
-    /// 越过 812 才放行。抬 F 逐次判：做完一次之后被拒，基数数拒之前最后写出的那一版。
-    #[test]
-    fn mount_and_raise_count_the_wall_from_the_version_their_admission_starts_from() {
-        let mut model = model_with_the_upper_bound_far_above_one_allocation_node();
-        let raise = model
-            .answer_raise_rollback_floor(ModelCheckpointTxg(0))
-            .expect("会话开着、F 不往下抬");
-        assert!(
-            raise.expected_roots.len() >= 2,
-            "{:?}",
-            raise.expected_roots
-        );
-        assert_eq!(
-            raise.root_whose_allocation_records_the_wall_counts(1),
-            Some(raise.expected_roots[0].key),
-            "做完一次之后被拒：数第一次抬 F 写出的那一版"
-        );
-        let second_publish_added =
-            raise.planned_publish_upper_bounds[1].allocation_records_added_by_the_admission;
-        assert_eq!(second_publish_added, 8);
-        let judged_raise = |counted: u64| {
-            model
-                .clone()
-                .judge_and_advance(
-                    &raise,
-                    &refused_by_the_allocation_record_wall(1, Some(counted)),
-                )
-                .map_err(|disagreement| disagreement.aspect)
-                .is_ok()
-        };
-        assert!(!judged_raise(812 - second_publish_added));
-        assert!(judged_raise(813 - second_publish_added));
-
-        let chosen = model.newest_root().key;
-        model.close_session();
-        let mount = model.answer_mount_writable();
-        assert_eq!(
-            mount.root_whose_allocation_records_the_wall_counts(0),
-            Some(chosen)
-        );
-        let added_by_the_mount: u64 = mount
-            .planned_publish_upper_bounds
-            .iter()
-            .map(|planned| planned.allocation_records_added_by_the_admission)
-            .sum();
-        assert_eq!(
-            added_by_the_mount,
-            10 + 8 * u64::try_from(mount.planned_publish_upper_bounds.len() - 1).expect("次数"),
-            "写行每盘 5 条、每次暖机每盘 4 条"
-        );
-        let judged_mount = |counted: u64| {
-            model
-                .clone()
-                .judge_and_advance(
-                    &mount,
-                    &refused_by_the_allocation_record_wall(0, Some(counted)),
-                )
-                .is_ok()
-        };
-        assert!(!judged_mount(812 - added_by_the_mount));
-        assert!(judged_mount(813 - added_by_the_mount));
-    }
 }
```

## 二、`tests/` 下的改动（32 个文件，只列文件名与增删行数；增删数由材料员对 `r2-to-r3.diff` 里对应段落数 `+`/`-` 开头且非 `+++`/`---` 的行得出，不抄全文）

| 文件（`crates/singlefs-harness/` 下的相对路径） | + | − |
|---|---|---|
| tests/checker_known_bad_images.rs | +189 | -134 |
| tests/first_transaction_step_five_publish.rs | +203 | -68 |
| tests/first_transaction_step_six_recovery.rs | +3 | -2 |
| tests/parallel_line_one_sequential_write.rs | +6 | -49 |
| tests/second_transaction_mapping_node_admission.rs | +16 | -63 |
| tests/second_transaction_parallel_line_one_multi_unit_file.rs | +106 | -15 |
| tests/second_transaction_parallel_line_one_sequential_write.rs | +41 | -11 |
| tests/second_transaction_parallel_line_three_many_inodes.rs | +31 | -6 |
| tests/second_transaction_parallel_line_two_mounted_read.rs | +90 | -32 |
| tests/second_transaction_position_addressed_trees_layer0.rs | +457 | -0 |
| tests/second_transaction_step_five_reuse.rs | +18 | -11 |
| tests/second_transaction_step_four_rollback.rs | +100 | -40 |
| tests/second_transaction_step_one_overwrite.rs | +132 | -85 |
| tests/second_transaction_step_three_formatted_pool.rs | +86 | -54 |
| tests/second_transaction_step_three_second_instance.rs | +63 | -30 |
| tests/second_transaction_supplement_one_write_accounting.rs | +51 | -23 |
| tests/second_transaction_supplement_three_bad_disk_input.rs | +21 | -14 |
| tests/second_transaction_supplement_three_random_history.rs | +102 | -196 |
| tests/second_transaction_supplement_two_accounting_node_full.rs | +6 | -3 |
| tests/second_transaction_supplement_two_admission_formula.rs | +56 | -13 |
| tests/second_transaction_supplement_two_c533_row_publish_record_without_its_root.rs | +43 | -41 |
| tests/second_transaction_supplement_two_commit_generated_fallback.rs | +52 | -26 |
| tests/second_transaction_supplement_two_instance_table_chain.rs | +5 | -3 |
| tests/second_transaction_supplement_two_instance_table_second_page_write.rs | +36 | -8 |
| tests/second_transaction_supplement_two_multi_record_transaction_zero_publishes.rs | +39 | -32 |
| tests/second_transaction_supplement_two_rebuild_with_an_unreadable_data_unit.rs | +1 | -1 |
| tests/second_transaction_supplement_two_reused_record_overlap.rs | +13 | -6 |
| tests/second_transaction_supplement_two_row_publish_admission.rs | +102 | -256 |
| tests/second_transaction_supplement_two_row_publish_checks_before_acquisition.rs | +67 | -59 |
| tests/second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs | +42 | -19 |
| tests/second_transaction_supplement_two_tree_split.rs | +54 | -13 |
| tests/second_transaction_supplement_two_unreadable_abandoned_root_slot.rs | +23 | -21 |

## 三、新文件全文

`r2-to-r3.diff` 用 `diff -ruN` 生成：本轮新建的三个 `src/` 文件（`crates/singlefs-core/src/allocation_record_tree.rs`、`crates/singlefs-core/src/extent_tree.rs`、`crates/singlefs-checker/src/position_addressed.rs`）在第一节的 diff 里已经是整份 `+` 行（对侧目录里不存在同名文件，`diff -ruN` 把新文件的全部内容当增量输出），不再重复附一份全文；`tests/` 下新建的 `crates/singlefs-harness/tests/second_transaction_position_addressed_trees_layer0.rs`（第二节表里 `+457 -0` 那一行）同理已含在第二节被截断的行数统计里，全文按第二节的口径不抄。

## 四、定义相对第二轮冻结的 diff（`defs-r2-to-r3.diff`，4 个文件，整段进）

```diff
--- /tmp/claude-1000/m2-final-code-r2/defs/.claude/agent-common.md	2026-09-24 19:28:06.096563176 +0000
+++ defs/.claude/agent-common.md	2026-09-24 22:47:56.775221502 +0000
@@ -54,7 +54,7 @@
 - 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
 - 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
   等长活时不起缓存计时器，结束本轮直接等完成通知。
-  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）对两种写法在执行前拒绝：前台没有超时的等待循环、把活放出追踪（`disown`、`nohup … &` 这类）；run_in_background 里的等待循环与按模式找进程这类命令只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
+  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）对四种写法在执行前拒绝：前台没有超时的等待循环、把活放出追踪（`disown`、`nohup … &` 这类）、run_in_background 里以单独的 `&` 收尾而之后同一条命令里没有 `wait` 的作业、起看门狗的错误写法（只有主 agent 起看门狗）；run_in_background 里的等待循环与按模式找进程这类命令只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
 
 ## 门禁
 
--- /tmp/claude-1000/m2-final-code-r2/defs/.claude/main-agent.md	2026-09-24 19:28:06.103397261 +0000
+++ defs/.claude/main-agent.md	2026-09-24 22:47:56.776427505 +0000
@@ -26,9 +26,9 @@
 
 ## 派出去之后
 
-盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的（前台无超时等待循环、把活放出追踪、子 agent 跑重型测试、写撇号类角标、覆盖未跟踪文件）在执行前拒绝，不碰已经在跑的东西。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨、子 agent 从派发起的运行时间跨过一个整小时（每个整点报一次，那个整点之后主 agent 给它发过消息就不报；一格多长在 `watch.conf` 的 `ask-every-minutes`），或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
+盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的（前台无超时等待循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&`、子 agent 跑重型测试、写撇号类角标、覆盖未跟踪文件）在执行前拒绝，不碰已经在跑的东西。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨、本实例底下有没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程（确认接着盯写 `--ack 进程:<pid>`）、子 agent 从派发起的运行时间跨过一个整小时（每个整点报一次，那个整点之后主 agent 给它发过消息就不报；一格多长在 `watch.conf` 的 `ask-every-minutes`），或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
 
-改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息：写明改了哪一条、它手上哪一步要停掉或改做——它们沿用派发那一刻的定义，看不到后来的改动。
+改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息：写明改了哪一条、它手上哪一步要停掉或改做。
 
 子 agent 等长活时不续提示缓存；主 agent 除了看门狗报「跑满 N 小时」时的那一条例行询问，不另外定时 ping 它。临时派不读共用约束的 agent（general-purpose 这类）干带长等待的活时，派发提示里写上共用约束「长活可以等」那一条里后台命令怎么写。
 
--- /tmp/claude-1000/m2-final-code-r2/defs/.claude/agents/crash-verifier.md	2026-09-24 19:28:06.097796672 +0000
+++ defs/.claude/agents/crash-verifier.md	2026-09-24 22:47:56.777483201 +0000
@@ -10,7 +10,7 @@
 
 开工先读 `.claude/agent-common.md`；这份定义开了 `omitClaudeMd`，不继承项目 CLAUDE.md 与它 `@` 的规则，要用的规则照共用约束「规则怎么读」一节读。
 
-只跑、只报，不修。你跑的是 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 3 步与「提交前必跑 herd7 与 QEMU」里最重的那几道。
+只跑、只报，不修。你跑的是 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 3 步与「重型测试只在提交时跑」里最重的那几道。
 开工先读：`.claude/singlefs-ai-sop/skills/crash-test/SKILL.md`「判读纪律」；`.claude/singlefs-ai-sop/rules/test-discipline.md`「崩溃一致性只能靠崩溃点重放验证」「阴性结果要能和「代码没跑到」分开」；`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「文件系统特有的反推缺口」。
 
 ## 输入（主 agent 必须给）
--- /tmp/claude-1000/m2-final-code-r2/defs/.claude/rules/implementation-workflow.md	2026-09-24 19:28:06.104441137 +0000
+++ defs/.claude/rules/implementation-workflow.md	2026-09-24 22:47:56.778569067 +0000
@@ -1,4 +1,4 @@
-# 实现改动的流程：写代码 → 三方对抗 → checker，提交前跑 herd7 与 QEMU
+# 实现改动的流程：写代码 → 三方对抗 → checker，重型测试只在提交时跑
 
 **这是 singlefs 的项目本地规则**，不在共享 SOP 里：它压在本机的三方论证（`.claude/rules/three-way-inference.md`）与
 本工程接管的 herd7 / QEMU 装置上，别的项目没有这两样。共享规则在 `.claude/singlefs-ai-sop/rules/`。
```

## 五、检测器第四种拒绝那一段（`bash-command-detector.sh`，冻结副本 `/tmp/claude-1000/m2-final-code-r3/defs/.claude/hooks/bash-command-detector.sh`）

文件头注释第 61–70 行（第 ④ 种拒绝的说明）：

```text
# ④ run_in_background 为 true 的命令里，有一个作业以单独的 `&` 收尾（不是 `&&`、`>&`、`&>`、`|&`），之后同一层、同一对圆括号里再没有 `wait`
#   （不带参数的 `wait`，或带进程号的 `wait "$pid"`；只带选项的 `wait -n` 不算）：外层 shell 起完它就退出，完成通知当场发出，
#   真跑完的那个进程叫不醒任何人。2026-09-25 一天里子 agent 至少三次在 run_in_background 里写 `… > 日志 2>&1 &`
#   （records/2026-09-16-subagent拆分提案.md 第四十节那张表第 23 行）。主 agent 与子 agent 都拒；前台的 `… &` 不拒。
#   按共用模块切出的记号逐层判：`bash -c '…'`（capped.sh N、nice 这类包着的剥开）、喂给 shell 的 heredoc 正文、命令替换各是一层，
#   里面的 `&` 要在同一层里等；圆括号是子 shell，`(… &); wait`、`… & (wait)` 里的 `wait` 等不到那个作业，照拒；
#   `{ …; }`、循环与 if 的身子与外面同一层。
#   `&` 之后按进程号轮询（`tail --pid`、`while kill -0`）不算 wait，照拒。起了几个 `&`、之后的 `wait "$pid"` 只等了其中一个的，
#   机器分不出等全没有，不拒，照检出 ② 记一条。引号里当数据写的、写进文件的 heredoc 正文、注释里的不拒。
#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法。
```

函数 `unwaited_ampersand_refusal`（第 433–441 行）：

```python
def unwaited_ampersand_refusal(command, run_in_background):
    """run_in_background 为 true 的命令里有以单独的 & 收尾、之后同一层没有 wait 的作业，就交回认出的作业；否则交 []。前台不拒。"""
    if (os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_CHECK") == "1"
            or os.environ.get("BASH_COMMAND_DETECTOR_ALLOW_UNWAITED_AMPERSAND") == "1"):
        return []
    if run_in_background is not True:
        return []
    return unwaited_background_jobs(command)

```
