# 附录二：第一轮冻结树到第二轮冻结树的 crates 改动，加 HEAD 之后打进的定义改动 diff（生成于 2026-09-25 04:46 JST / 2026-09-24 19:46 UTC）

基准：第一轮冻结副本 `/tmp/claude-1000/m2-final-code-r1/tree/crates/` 到本轮冻结副本 `/tmp/claude-1000/m2-final-code-r2/tree/crates/`（腿读代码一律读本轮冻结副本，不读主工作区——主工作区在腿跑着的时候还会被实二一继续改 `allocator.rs`、`transaction.rs`、`lib.rs`、`singlefs-format/src/lib.rs`，并新建 `allocation_record_tree.rs`、`extent_tree.rs`）。

diff 原始文件（两份，均由主 agent 给出，材料员未重新生成、原样落盘）：
- `/tmp/claude-1000/m2-final-code-r2/r1-to-r2.diff`（`diff -ruN -x target` 生成，43 个文件：`crates/mutations.tsv` 1 份、`src/*.rs` 23 份、`tests/*.rs` 19 份，共 16019 行）；
- `/tmp/claude-1000/m2-final-code-r2/defs-vs-head.diff`（`git diff HEAD -- <8 份定义>` 生成，8 个文件，共 237 行）。

太长的一半按派发提示的口径截断：`tests/` 下的 19 份只列文件名与增删行数，不抄全文；`crates/mutations.tsv` 与 `src/*.rs` 24 份的改动整段抄进第一节。第一节用 `diff -ruN -x target` 原有的按文件分段格式，每段前一行 `diff -ruN -x target <相对路径>`（省去两侧绝对路径与 `-x target` 之外的信息，只留材料员从原始文件里逐段切出、未改一字的差异本体）。

## 一、diff（`crates/mutations.tsv` 与 `src/*.rs`，共 24 个文件，第一轮冻结树到本轮冻结树，原样节选自 `r1-to-r2.diff`）

```diff
diff -ruN -x target tree/crates/mutations.tsv
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/mutations.tsv	2026-09-24 16:27:05.398761624 +0000
+++ tree/crates/mutations.tsv	2026-09-24 18:48:58.254848952 +0000
@@ -8,19 +8,15 @@
 空闲不随分配减（I-5.2 要红）	crates/singlefs-core/src/allocator.rs	        self.free_slots -= span;	        // 变异：空闲不减	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
 释放时清位图（立即复用）	crates/singlefs-core/src/allocator.rs	        self.deferred_slots += span;\n    }	        self.deferred_slots += span;\n        for index in start..end {\n            self.allocated[index] = false;\n        }\n    }	-p singlefs-harness --test second_transaction_step_one_overwrite -- released_placements	released_placements_are_not_handed_out_again_before_reclaim_exists
 分配记录树装不下不判	crates/singlefs-core/src/transaction.rs	    if records_after_this_publish > allocation_node_capacity {	    if false && records_after_this_publish > allocation_node_capacity {	-p singlefs-harness --test second_transaction_step_one_overwrite -- repeated_overwrites	repeated_overwrites_report_a_full_allocation_node_instead_of_panicking
-释放退回按提示	crates/singlefs-core/src/transaction.rs	            placements_to_release_via_mapping(previous_version, allocator, &rewritten)?	            previous_version.placements()	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_goes_through	release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released
 映射查到 key 就算（落点指错不核记录）	crates/singlefs-core/src/transaction.rs	        let record =\n            allocator\n                .record_for(device, slot)\n                .ok_or(PublishError::ReleaseTargetNotAllocated {	        let record = allocator\n            .record_for(device, slot)\n            .or(allocator.records().first())\n            .ok_or(PublishError::ReleaseTargetNotAllocated {	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_reports_a_mapping_entry_whose_slot	release_reports_a_mapping_entry_whose_slot_has_no_record_or_the_wrong_span_instead_of_panicking
-失败的发布不退回分配器	crates/singlefs-core/src/transaction.rs	    if outcome.is_err() {	    if false && outcome.is_err() {	-p singlefs-harness --test second_transaction_step_one_overwrite -- publish_running_out_of_space_midway	publish_running_out_of_space_midway_leaves_the_allocator_as_it_was
+失败的发布不退回分配器	crates/singlefs-core/src/transaction.rs	        Err(refusal) => {\n            *allocator = allocator_before_this_publish;\n            return Err(refusal);\n        }\n    };\n    // 落盘那几步里失败：分配器换回去、这次发布冻结在它上面等原样重发（`FrozenPublish`）。\n	        Err(refusal) => {\n            return Err(refusal);\n        }\n    };\n    // 落盘那几步里失败：分配器换回去、这次发布冻结在它上面等原样重发（`FrozenPublish`）。\n	-p singlefs-harness --test second_transaction_step_one_overwrite -- publish_running_out_of_space_midway	publish_running_out_of_space_midway_leaves_the_allocator_as_it_was
 内容超长不判	crates/singlefs-core/src/transaction.rs	        if file.content.len() > data_unit_capacity {	        if false && file.content.len() > data_unit_capacity {	-p singlefs-harness --test second_transaction_step_one_overwrite -- content_larger_than_a_data_unit	content_larger_than_a_data_unit_payload_is_refused_before_anything_is_touched
 oracle 只比 txg 不比实例	crates/singlefs-harness/src/crash.rs	        if (effective_txg, effective_instance) < (newest_txg, newest_instance) {	        if effective_txg < newest_txg {	-p singlefs-harness --lib -- oracle_instance_tests	landing_on_the_lower_instance_of_the_same_txg_is_a_violation
 oracle 只按 txg 找版本	crates/singlefs-harness/src/crash.rs	        version.checkpoint_txg == effective_txg && version.instance == effective_instance	        version.checkpoint_txg == effective_txg	-p singlefs-harness --lib -- oracle_instance_tests	the_same_txg_from_two_instances_are_two_versions
 oracle 放过没版本的更新根	crates/singlefs-harness/src/crash.rs	                (candidate_version.checkpoint_txg, candidate_version.instance)\n                    < (effective_txg, effective_instance)	                false	-p singlefs-harness --lib -- oracle_instance_tests	newer_root_without_any_version_reporting_no_file_is_violation
 Ignore 那一遍恢复不过 oracle	crates/singlefs-harness/src/crash.rs	        tally.ignored_violations += 1;	        tally.ignored_violations += 0;	-p singlefs-harness --test second_transaction_step_zero_layer0 -- targeted_controls	targeted_controls_on_the_second_publish_go_red_where_they_should
-步 1 变异：不释放旧落点	crates/singlefs-core/src/transaction.rs	            placements_to_release_via_mapping(previous_version, allocator, &rewritten)?	            Vec::<Placement>::new()	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
-步 1 变异：defer 行写 0	crates/singlefs-core/src/transaction.rs	            device_map.deferred_slots() * SLOT_BYTES,	            0,	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
 步 1 变异：事务号不加一	crates/singlefs-core/src/transaction.rs	            transaction: previous.highest_transaction_number_in_this_instance + 1,	            transaction: previous.highest_transaction_number_in_this_instance,	-p singlefs-harness --test second_transaction_step_one_overwrite -- overwrite_publishes	overwrite_publishes_the_second_version_through_the_same_commit_shape
 步 1 变异：改动计数留 1	crates/singlefs-core/src/transaction.rs	            change_count: txg.0,	            change_count: 1,	-p singlefs-harness --test second_transaction_step_one_overwrite -- overwrite_publishes	overwrite_publishes_the_second_version_through_the_same_commit_shape
-步 2 变异：已分配行不随分配更新（I-3.1 要红）	crates/singlefs-core/src/transaction.rs	            device_map.allocated_slots() * SLOT_BYTES,	            0,	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
 步 2 变异：忘了改写分配记录	crates/singlefs-core/src/allocator.rs	            record.is_released = true;	            record.is_released = false;	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
 步 3：不写行（实例表照抄）	crates/singlefs-core/src/mount.rs	            instance_table: InstanceTablePlan::Rewrite(instance_table),	            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
 步 3：暖机只推一次	crates/singlefs-core/src/mount.rs	        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(warm_up_publishes.len()).expect("次数") < 1	-p singlefs-harness --test second_transaction_step_three_second_instance -- damaging_every_instance_two_root	damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device
@@ -63,7 +59,7 @@
 步 5：「非空」只比 inode 树的根指针、不比 extent 树的	crates/singlefs-core/src/mount.rs	    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree\n        || root_pointers.extent_tree != previous_valid_root_pointers.extent_tree	    root_pointers.inode_tree != previous_valid_root_pointers.inode_tree	-p singlefs-core --lib -- root_is_non_empty_when_either	root_is_non_empty_when_either_user_visible_tree_pointer_differs_from_the_previous_valid_root
 步 3：只做过 mkfs 的池重开后分配器不认 mkfs 写在单元区里的两个单元	crates/singlefs-core/src/mount.rs	    allocator.mark_format_time_units(instance_table_placement, tree_table_placement);	    let _ = (instance_table_placement, tree_table_placement);	-p singlefs-harness --test second_transaction_step_three_formatted_pool	writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold
 步 3：只做过 mkfs 的池上零单元暖机的反向链写 0	crates/singlefs-core/src/mount.rs	                back_chain: back_chain_of(&current_version_without_file.record_bytes),	                back_chain: 0,	-p singlefs-harness --test second_transaction_step_three_formatted_pool	writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold
-步 3：零单元发布在记录与根之间少一道屏障	crates/singlefs-core/src/transaction.rs	        writer.perform(CommitStep::Barrier)?;\n        persist_the_root_then_rotate_the_system_configuration(\n            writer,\n            plan.txg,	        persist_the_root_then_rotate_the_system_configuration(\n            writer,\n            plan.txg,	-p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- the_formatted_pool_mount_and_first_file_stream	the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence
+步 3：记录与根之间少一道屏障（实二二三起三条发布路径共用 persist_publish_writes，零单元发布那一段也少了这一道）	crates/singlefs-core/src/transaction.rs	    writer.perform(CommitStep::Barrier)?;\n    persist_the_root_then_rotate_the_system_configuration(\n        writer,\n        writes.checkpoint_txg,\n	    persist_the_root_then_rotate_the_system_configuration(\n        writer,\n        writes.checkpoint_txg,\n	-p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- the_formatted_pool_mount_and_first_file_stream	the_formatted_pool_mount_and_first_file_stream_keeps_the_first_transaction_segment_sequence
 步 3：只做过 mkfs 的池重开后把 mkfs 实例表按 1 槽记	crates/singlefs-core/src/mount.rs	        placement_of(&root.instance_table, TransactionUnit::InstanceTable)?;	        placement_of(&root.instance_table, TransactionUnit::TreeTable)?;	-p singlefs-harness --test second_transaction_step_three_formatted_pool_layer0 -- every_crash_state_outside_the_unit_segment_of_the_formatted_pool	every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims
 步 3：写行时这次要写的行没接进重写出去的那条实例表链（带文件的一版与树表 0 条的一版共用这一处拼法）	crates/singlefs-core/src/mount.rs	    rows.extend_from_slice(rows_written);	    rows.extend_from_slice(&[]);	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_formatted_pool_mounted_twice	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
 步 5：算抬 F 上限时有效根的树表读不出，按「树表里没有这两棵树」猜而不是拒绝	crates/singlefs-core/src/mount.rs	        Ok(pointers) => Ok(pointers),\n        Err(failure) => Err(	        Ok(pointers) => Ok(pointers),\n        Err(_failure) if true => Ok(UserVisibleTreeRootPointers::ABSENT),\n        Err(failure) => Err(	-p singlefs-harness --test second_transaction_step_five_reuse -- raising_the_floor_is_refused_when	raising_the_floor_is_refused_when_a_valid_root_tree_table_is_unreadable
@@ -98,12 +94,9 @@
 步 6：层 0 计数不记「不适用」（评估过 + 不适用 ≠ 状态数）	crates/singlefs-harness/src/crash.rs	            InvariantVerdict::NotApplicable(_) => {\n                *tally\n                    .checker_not_applicable_states\n                    .entry(invariant)\n                    .or_insert(0) += 1;\n            }	            InvariantVerdict::NotApplicable(_) => {}	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
 增补 2 第 21 行：暖机把 jsn 计数器写成 txg（C366；接在 jsn 40 之后时记录落进 jsn 1、2 那两格、tail 写 2）	crates/singlefs-core/src/transaction.rs	txg: CheckpointTxg(txg_number),\n                counter: previous_counter + 1,	txg: CheckpointTxg(txg_number),\n                counter: txg_number,	-p singlefs-harness --test second_transaction_supplement_two_warm_up_counter -- warm_up_after_a_journal_counter	warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two
 增补 2 第 19 行：出生序号发号器挪回每装一个文件对象重建一次（同一个 checkpoint 的第二个对象从 0 重数）	crates/singlefs-core/src/transaction.rs	) -> FileVersionUnits {\n    let txg = checkpoint.txg;	) -> FileVersionUnits {\n    let mut sequences_rebuilt_on_every_call = BirthSequenceAllocator::default();\n    let sequences = &mut sequences_rebuilt_on_every_call;\n    let txg = checkpoint.txg;	-p singlefs-core --lib -- transaction::tests::second_file_object	second_file_object_in_the_same_checkpoint_continues_birth_sequences_instead_of_restarting_at_zero
-增补 2 第 28 行：记账树装不下不判（80 块盘走到装节点的断言 panic）	crates/singlefs-core/src/transaction.rs	    if accounting_entries_of_this_publish > accounting_node_capacity {	    if false && accounting_entries_of_this_publish > accounting_node_capacity {	-p singlefs-harness --test second_transaction_supplement_two_accounting_node_full -- eightieth_device	eightieth_device_overflows_the_accounting_node_and_the_publish_is_refused_before_anything_is_written
-增补 2 第 28 行：记账树正好装满 477 行也判装不下（79 块盘发布被拒）	crates/singlefs-core/src/transaction.rs	    if accounting_entries_of_this_publish > accounting_node_capacity {	    if accounting_entries_of_this_publish >= accounting_node_capacity {	-p singlefs-harness --test second_transaction_supplement_two_accounting_node_full -- seventy_nine_devices	seventy_nine_devices_fill_the_accounting_node_exactly_and_still_publish
 增补 2 第 30 行：真设备二进制的挂载窗口从重开那一刻算起（取号的两次系统配置槽写进了设备一层的数、不在按种类的账里）	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	        pool_writes_between(&counts_after_acquisition, &counts_after_mount),	        pool_writes_between(&vec![DeviceCallCounts { write_calls: 0, written_bytes: 0, force_unit_access_writes: 0, barrier_calls: 0 }; counts_after_mount.len()], &counts_after_mount),	-p singlefs-harness --bin first_transaction_on_device -- second_instance_mode	second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches
 增补 2 第 30 行：每道屏障都覆盖「第一道屏障」那份计数（挂载窗口从最后一道屏障算起；2026-09-21 随通用故障注入包装搬到 fault_injection.rs）	crates/singlefs-harness/src/fault_injection.rs	        state\n            .counts_when_the_first_barrier_arrived\n            .entry(device)\n            .or_insert(counts);	        state\n            .counts_when_the_first_barrier_arrived\n            .insert(device, counts);	-p singlefs-harness --bin first_transaction_on_device -- second_instance_mode	second_instance_mode_mounts_writes_the_row_warms_up_publishes_the_third_version_and_every_window_matches
-增补 2 第 20a 行：写行那次发布的准入不在取号之前算（取号之后发布才被拒，实例代号已经烧掉）	crates/singlefs-core/src/mount.rs	    refuse_publishes_before_acquisition_that_do_not_pass_admission(\n        &allocator,\n        &start,\n        instance_to_acquire,\n        &instance_table_rewrite,\n        rows_written.len(),\n        warm_up_publishes_planned.len(),\n    )?;\n	    // 变异：写行那次发布的准入不在取号之前算\n	-p singlefs-harness --test second_transaction_supplement_two_row_publish_admission -- writable_mount_that_cannot_publish	writable_mount_that_cannot_publish_the_rows_is_refused_before_the_instance_is_acquired
-增补 2 第 20b 行：中途失败的发布不把这次已记的写交出去（与设备一层记的合计对不上）	crates/singlefs-core/src/transaction.rs	    if let Err(cause) = persist(pool) {\n        pool.count_failed_publish(&writes_before_this_publish);\n        return Err(PublishError::BlockDevice(cause));\n    }	    if let Err(cause) = persist(pool) {\n        return Err(PublishError::BlockDevice(cause));\n    }	-p singlefs-harness --test second_transaction_supplement_one_write_accounting -- publish_that_fails_midway	publish_that_fails_midway_hands_out_what_it_already_wrote_so_the_retry_still_adds_up
+增补 2 第 20b 行：中途失败的发布不把这次已记的写交出去（与设备一层记的合计对不上）	crates/singlefs-core/src/transaction.rs	    if let Err(cause) = persist_publish_writes(pool, &writes) {\n        pool.count_failed_publish(&writes_before_this_publish);\n        let allocator_after_the_publish =\n	    if let Err(cause) = persist_publish_writes(pool, &writes) {\n        let allocator_after_the_publish =\n	-p singlefs-harness --test second_transaction_supplement_one_write_accounting -- publish_that_fails_midway	publish_that_fails_midway_hands_out_what_it_already_wrote_so_the_retry_still_adds_up
 增补 2 第 20c 行：用户数据只排除当前那一个开放段（回落把开放段置空之后，那一段对用户数据开放）	crates/singlefs-core/src/allocator.rs	        let cluster_segments = &self.cluster_segments;	        let cluster_segments = &self.open_segment.into_iter().collect::<BTreeSet<SlotNumber>>();	-p singlefs-core --lib -- allocator::tests::user_data_does_not_land	user_data_does_not_land_in_a_cluster_segment_that_the_fallback_closed
 收口表第 22 行 C22（刚释放的块立即重分配）：候选根那一格不判 I-4.8（复用窗口置 0 之后 I-4.8 不红）	crates/singlefs-checker/src/walk.rs	                .judge("I-4.8", !walked_into_reused_or_erased_unit, || {	                .judge("I-4.8", true, || {	-p singlefs-harness --test second_transaction_step_five_reuse -- reclaiming_without_raising_the_floor	reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red
 增补 2 第 20a 行：取号之前只算写行那一次，暖机那几次空发布不算（写行发完、暖机才被拒，实例代号已经烧掉）	crates/singlefs-core/src/mount.rs	        let shapes: Vec<PublishShape> =\n            std::iter::once(PublishShape::row_publish_rewriting_instance_table_pages(\n                instance_table_rewrite.pages_after_this_publish(),\n            ))\n            .chain(std::iter::repeat_n(\n                PublishShape::EMPTY_PUBLISH,\n                warm_up_publishes_planned,\n            ))\n            .collect();	        let shapes: Vec<PublishShape> = vec![PublishShape::row_publish_rewriting_instance_table_pages(\n            instance_table_rewrite.pages_after_this_publish(),\n        )];	-p singlefs-harness --test second_transaction_supplement_two_row_publish_admission -- writable_mount_whose_first_warm_up	writable_mount_whose_first_warm_up_publish_does_not_fit_is_refused_before_the_instance_is_acquired
@@ -160,10 +153,6 @@
 增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2h）：分配器用户数据那一处把「每块盘上都没有」报成「小盘写满」；小盘段判出	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::Agreed(slot) => slot,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);	            DeviceAgreement::Agreed(slot) => slot,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices: Vec::new() });	-p singlefs-harness --test second_transaction_supplement_three_random_history -- unit_area_wall_sampling	unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device
 增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2i）：分配器用户数据那一处把「小盘写满」报成「每块盘上都没有」；等大的小盘走不到，不等盘那条用例判出	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::SomeDevicesWithoutAnswer(full_devices) => {\n                return Err(\n                    PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices },\n                );\n            }\n            DeviceAgreement::AnswersDiffer(slot_per_device) => {	            DeviceAgreement::SomeDevicesWithoutAnswer(_full_devices) => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);\n            }\n            DeviceAgreement::AnswersDiffer(slot_per_device) => {	-p singlefs-harness --test second_transaction_supplement_two_unequal_devices -- filling_the_smaller_device	filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking
 增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4a）：根环转过之后回收门槛取「环里最旧有效根 + 1」；逼近分配记录墙那一段照跑 checker 判出	crates/singlefs-core/src/mount.rs	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(if oldest.0 > 0 { CheckpointTxg(oldest.0 + 1) } else { oldest }))	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_wall_sampling	allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count
-增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4b）：分配记录过 600 条之后记账「已分配」少记一槽；逼近分配记录墙那一段照跑 checker 判出	crates/singlefs-core/src/transaction.rs	            device_map.allocated_slots() * SLOT_BYTES,	            (device_map.allocated_slots() - u64::from(allocator.records().len() > 600)) * SLOT_BYTES,	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_wall_sampling	allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count
-增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4b 同一处）：分配记录过 600 条之后记账「已分配」少记一槽；直接钉已分配统计的那条用例判出	crates/singlefs-core/src/transaction.rs	            device_map.allocated_slots() * SLOT_BYTES,	            (device_map.allocated_slots() - u64::from(allocator.records().len() > 600)) * SLOT_BYTES,	-p singlefs-harness --test second_transaction_step_one_overwrite -- allocated_statistic_equals	allocated_statistic_equals_the_span_sum_of_the_allocation_records_past_six_hundred_records
-增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1c；要带调试符号的构建，函数改名会悄悄失效）：只在抬 F 路径的准入里多算一个角色；抬 F 逼近墙的写死用例判出	crates/singlefs-core/src/transaction.rs	        records_before_this_publish + rewritten.len() * allocator.devices.len();	        records_before_this_publish + (rewritten.len() + usize::from(std::backtrace::Backtrace::force_capture().to_string().contains("raise_rollback_floor"))) * allocator.devices.len();	-p singlefs-harness --test second_transaction_supplement_three_random_history -- raising_the_floor_with_a_second_empty_publish	raising_the_floor_with_a_second_empty_publish_that_fills_the_allocation_node_to_exactly_812_records_succeeds
-增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1d；要带调试符号的构建，函数改名会悄悄失效）：只在回退路径的准入里多算一个角色；回退逼近墙的写死用例判出	crates/singlefs-core/src/transaction.rs	        records_before_this_publish + rewritten.len() * allocator.devices.len();	        records_before_this_publish + (rewritten.len() + usize::from(std::backtrace::Backtrace::force_capture().to_string().contains("mount_rollback"))) * allocator.devices.len();	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_a_root_whose_warm_up	rolling_back_to_a_root_whose_warm_up_fills_the_allocation_node_to_exactly_812_records_succeeds
 增补 3 第 3 件：抽 0 个崩溃点时照样抽一个（抽崩溃点会改变这段历史怎么跑，判别力那一半失效）	crates/singlefs-harness/src/crash_injection.rs	            for _ in 0..crash_points_per_history {	            for _ in 0..crash_points_per_history.max(1) {	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- drawing_crash_points	drawing_crash_points_does_not_change_the_generated_history
 增补 3 第 3 件：判崩溃点时取择根而不是施加记录前缀之后实际走的那条根（记录已写、根槽未写那一格内容与根身份配不上）	crates/singlefs-harness/src/crash_injection.rs	        let read_back = observed_read_back_after_a_crash(&report);	        let read_back = crate::model_comparison::observed_read_back(&report.outcome);	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- crash_injection_fast_tier	crash_injection_fast_tier_recovers_only_into_versions_the_model_committed
 增补 3 第 3 件（用户 2026-09-20 定案第 1 条）：屏障不再切段（屏障进了枚举域，少一道屏障就把两段并成一段）	crates/singlefs-harness/src/crash.rs	            RecordedOperationKind::Barrier => {\n                if !current.is_empty() {\n                    segments.push(std::mem::take(&mut current));\n                }\n            }	            RecordedOperationKind::Barrier => {}	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- every_crash_state_of_a_written_out_history	every_crash_state_of_a_written_out_history_recovers_into_a_committed_version
@@ -184,8 +173,8 @@
 增补 3 第 4 件：注入的「读回改坏的字节」其实一位都没翻（坏字节注不进去）	crates/singlefs-harness/src/fault_injection.rs	                    buffer[index] ^= mask;	                    buffer[index] ^= 0;	-p singlefs-harness --lib -- fault_injection::tests::a_corrupted_read	a_corrupted_read_flips_one_bit_in_the_buffer_and_leaves_the_device_alone
 增补 3 第 4 件：被吞掉的那一次写其实落了盘（设备说谎那一形注不进去）	crates/singlefs-harness/src/fault_injection.rs	            Some(InjectedFault::WriteIsSwallowed) => return Ok(()),	            Some(InjectedFault::WriteIsSwallowed) => {}	-p singlefs-harness --lib -- fault_injection::tests::a_swallowed_write	a_swallowed_write_reports_success_and_changes_nothing_on_the_device
 增补 3 第 4 件：每块盘第一道屏障那一刻的计数根本不记（真设备那一路按它切挂载窗口，不记就切不出来）	crates/singlefs-harness/src/fault_injection.rs	        state\n            .counts_when_the_first_barrier_arrived\n            .entry(device)\n            .or_insert(counts);	        let _ = (&mut state, device, counts);	-p singlefs-harness --lib -- fault_injection::tests::the_counts_when_the_first_barrier_arrived	the_counts_when_the_first_barrier_arrived_are_frozen_at_that_moment
-增补 3 第 4 件（被测的性质）：发布落盘阶段的块设备错改成 panic，不再把错返回上来	crates/singlefs-core/src/transaction.rs	    if let Err(cause) = persist(pool) {\n        pool.count_failed_publish(&writes_before_this_publish);\n        return Err(PublishError::BlockDevice(cause));\n    }	    persist(pool).expect("落盘那几步不报错");	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- fault_injection_fast_tier	fault_injection_fast_tier_returns_errors_instead_of_panicking
-增补 3 第 4 件（增补 2 收口表第 40 行 / C381 的判别力）：发布失败时分配器不再退回（C381 的候选改法之一），下一次发布就不会盖掉那条根指着的单元	crates/singlefs-core/src/transaction.rs	    if outcome.is_err() {\n        *allocator = allocator_before_this_publish;\n    }	    if false {\n        *allocator = allocator_before_this_publish;\n    }	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- a_publish_that_fails	a_publish_that_fails_on_the_system_configuration_slot_leaves_a_root_whose_units_the_next_publish_overwrites
+增补 3 第 4 件（被测的性质）：发布落盘阶段的块设备错改成 panic，不再把错返回上来	crates/singlefs-core/src/transaction.rs	    if let Err(cause) = persist_publish_writes(pool, &writes) {\n        pool.count_failed_publish(&writes_before_this_publish);\n        let allocator_after_the_publish =\n	    if let Err(cause) = persist_publish_writes(pool, &writes)\n        .map_err(|cause| -> BlockDeviceError { panic!("落盘那几步不报错：{cause:?}") })\n    {\n        pool.count_failed_publish(&writes_before_this_publish);\n        let allocator_after_the_publish =\n	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- fault_injection_fast_tier	fault_injection_fast_tier_returns_errors_instead_of_panicking
+增补 3 第 4 件 × 这一版的失败处置（D23 已定项 14，实二二三）：落盘失败的发布不冻结——同一个进程拿同一个上一版另建一次发布，把那条已落盘的根指着的单元盖掉（C381 那一格回来）	crates/singlefs-core/src/transaction.rs	        allocator.freeze_publish(FrozenPublish {\n            writes,\n            version,\n            allocator_after_the_publish,\n        });\n	        let _ = (writes, version, allocator_after_the_publish);\n	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- a_publish_that_fails_on_the_system_configuration_slot	a_publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root
 增补 3 第 4 件（增补 2 收口表第 40 行 / C381 的前提）：发布的持久顺序把系统配置槽轮换挪到根槽 FUA 写之前，最后一步失败时根就不会已经落盘	crates/singlefs-core/src/transaction.rs	    writer.perform(CommitStep::WriteRootRecordForceUnitAccess {\n        checkpoint_txg,\n        root_slot,\n    })?;\n    writer.perform(CommitStep::RotateSystemConfigurationSlots {\n        journal_tail,\n        journal_instance,\n    })	    writer.perform(CommitStep::RotateSystemConfigurationSlots {\n        journal_tail,\n        journal_instance,\n    })?;\n    writer.perform(CommitStep::WriteRootRecordForceUnitAccess {\n        checkpoint_txg,\n        root_slot,\n    })	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- a_publish_that_fails	a_publish_that_fails_on_the_system_configuration_slot_leaves_a_root_whose_units_the_next_publish_overwrites
 增补 3 第 4 件：一块盘两个系统配置槽都无效就整体 return Err（每盘一份的冗余当场作废，D22 已定项 8 第 1 条）	crates/singlefs-core/src/recovery.rs	            (None, None) => {\n                // 这块盘上的两份都废了，但别的盘各自还带着一份完整的系统配置：跳过它，别让整池挂不上。\n                if first_device_with_no_valid_system_configuration_slot.is_none() {\n                    first_device_with_no_valid_system_configuration_slot = Some(device);\n                }\n                continue;\n            }	            (None, None) => {\n                return Err(RecoveryFailure::NoValidSystemConfiguration {\n                    first_device_with_no_valid_system_configuration_slot: device,\n                })\n            }	-p singlefs-harness --test system_configuration_per_device_redundancy	both_system_configuration_slots_unreadable_on_device_one_still_read_the_file_back
 增补 3 第 4 件：只在已经择到一份之后才容许跳过整盘（坏的是次序里第一块盘时照旧挂不上）	crates/singlefs-core/src/recovery.rs	            (None, None) => {\n                // 这块盘上的两份都废了，但别的盘各自还带着一份完整的系统配置：跳过它，别让整池挂不上。\n                if first_device_with_no_valid_system_configuration_slot.is_none() {\n                    first_device_with_no_valid_system_configuration_slot = Some(device);\n                }\n                continue;\n            }	            (None, None) => {\n                if chosen.is_none() {\n                    return Err(RecoveryFailure::NoValidSystemConfiguration {\n                        first_device_with_no_valid_system_configuration_slot: device,\n                    });\n                }\n                continue;\n            }	-p singlefs-harness --test system_configuration_per_device_redundancy	both_system_configuration_slots_unreadable_on_device_zero_still_read_the_file_back
@@ -240,7 +229,6 @@
 并行线二：读路径自报的设备级读数不加（实现说这次一个设备读都没发，装置在块层照样数到那几次，D17 已定项 5 射程 ① 的对账等于没有）	crates/singlefs-core/src/mounted_read.rs	        observation.device_reads_issued += 1;	        observation.device_reads_issued += 0;	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- random_four_kibibyte_reads	random_four_kibibyte_reads_return_the_written_bytes_and_never_scan_the_journal_ring
 并行线二：这次读了几个单元的计数不加（验收第 4 条那个观测点恒 0，跨单元的页也不报 2）	crates/singlefs-core/src/mounted_read.rs	            observation.data_units_read += 1;	            observation.data_units_read += 0;	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- every_aligned_page	every_aligned_page_reads_back_and_the_pages_that_cross_a_unit_boundary_read_two_units
 并行线二：一次读只解引用第一个单元（跨单元的页少拷后半页，D4 已定项 5 那一成多的页整档漏掉）	crates/singlefs-core/src/mounted_read.rs	        for unit_number in span.first.0..=span.last_inclusive.0 {	        for unit_number in span.first.0..=span.first.0 {	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- corrupting_one_byte	corrupting_one_byte_in_a_data_unit_makes_reads_over_it_report_a_checksum_error_and_leaves_the_other_reads_alone
-并行线二：中央映射的根是内部节点也照收（把内部条目当 55 字节映射条目解，「整片映射都在挂载态里」这条前提没人守，读数 3 也就没了依据）	crates/singlefs-core/src/mounted_read.rs	    if mapping_root.level != CENTRAL_MAPPING_TREE_ROOT_LEVEL {	    if false && mapping_root.level != CENTRAL_MAPPING_TREE_ROOT_LEVEL {	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- a_central_mapping_root	a_central_mapping_root_that_is_an_internal_node_is_refused_instead_of_being_read_as_entries
 只供测试的开关：关掉点名验证时 verification_passed 照加（两臂在健康镜像上报同一个数，运行时看不出走了哪一条）	crates/singlefs-core/src/recovery.rs	        if verify_named_units {\n            report.verification_passed += 1;\n        }	        report.verification_passed += 1;	-p singlefs-harness --test first_transaction_step_six_recovery	the_named_unit_verification_switch_shows_up_in_the_runtime_counter
 I-1.10：条目宽与字段表宽的比对改成恒真（走读照样停下，只是不再判红）	crates/singlefs-checker/src/walk.rs	.judge("I-1.10", entry_width == entry_field_table_bytes, || {	.judge("I-1.10", true, || {	-p singlefs-harness --test checker_known_bad_images -- an_entry_width	an_entry_width_that_is_not_the_field_table_width_reddens_only_the_entry_width_invariant
 I-1.10：判红之后不停下，照样按字段表的固定偏移解那条窄条目（checker 自己被坏镜像打死）	crates/singlefs-checker/src/walk.rs	            if entry_width != entry_field_table_bytes {\n                return;\n            }	            if false && entry_width != entry_field_table_bytes {\n                return;\n            }	-p singlefs-harness --test checker_known_bad_images -- an_entry_width	an_entry_width_that_is_not_the_field_table_width_reddens_only_the_entry_width_invariant
@@ -268,7 +256,6 @@
 普查 R2：核心层映射条目读者的字段表宽度判去掉（又按固定偏移切到 55，单测那一格）	crates/singlefs-core/src/records.rs	    if bytes.len() < usize::try_from(MAPPING_ENTRY_BYTES).expect("55") {	    if false && bytes.len() < usize::try_from(MAPPING_ENTRY_BYTES).expect("55") {	-p singlefs-core --lib -- entry_readers_refuse_an_entry_narrower_than_its_field_table	entry_readers_refuse_an_entry_narrower_than_its_field_table
 普查 R2：核心层映射条目读者的字段表宽度判去掉（坏盘输入那一格：中央映射树根的条目宽缩到 key 宽）	crates/singlefs-core/src/records.rs	    if bytes.len() < usize::try_from(MAPPING_ENTRY_BYTES).expect("55") {	    if false && bytes.len() < usize::try_from(MAPPING_ENTRY_BYTES).expect("55") {	-p singlefs-harness --test second_transaction_supplement_three_bad_disk_input -- every_fixed_panic_site	every_fixed_panic_site_reports_its_own_error_member_instead_of_panicking
 普查 R2：映射条目的字段表宽度判判成闭区间（刚好 55 字节的合法条目被误拒）	crates/singlefs-core/src/records.rs	    if bytes.len() < usize::try_from(MAPPING_ENTRY_BYTES).expect("55") {	    if bytes.len() <= usize::try_from(MAPPING_ENTRY_BYTES).expect("55") {	-p singlefs-core --lib -- entry_readers_refuse_an_entry_narrower_than_its_field_table	entry_readers_refuse_an_entry_narrower_than_its_field_table
-普查 R2：checker 走读中央映射条目之前不判条目宽（walk.rs 又切 entry[27..55]，checker 自己倒下）	crates/singlefs-checker/src/walk.rs	            if mapping.entry_width < mapping_entry_bytes() {	            if false && mapping.entry_width < mapping_entry_bytes() {	-p singlefs-harness --test second_transaction_supplement_three_bad_disk_input -- every_fixed_panic_site	every_fixed_panic_site_reports_its_own_error_member_instead_of_panicking
 普查 R2：挂载态读路径不判映射条目宽（open_pool_for_read 直接按 55 切）	crates/singlefs-core/src/mounted_read.rs	        let (key, locations) =\n            parse_mapping_entry(entry_bytes).ok_or(OpenPoolForReadFailure::RecordMalformed {\n                what: "映射条目",\n            })?;	        let (key, locations) = parse_mapping_entry(entry_bytes).expect("映射条目宽 55");	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- a_central_mapping_root_whose_entry_width	a_central_mapping_root_whose_entry_width_is_narrower_than_the_field_table_is_refused_instead_of_slicing_past_the_entry
 普查 R2：释放判定路径把「映射条目切不动」报成「不在映射」（两件要分流的事并成一个成员）	crates/singlefs-core/src/transaction.rs	            return MappingLookup::EntryNarrowerThanItsFieldTable {\n                entry_bytes: entry.len(),\n            };	            return MappingLookup::NodeMalformedOrNoEntryWithThisKey;	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_reports_a_mapping_entry_narrower	release_reports_a_mapping_entry_narrower_than_its_field_table_instead_of_slicing_past_it
 普查 R10：释放前的校验退回只核第一条位置条目那块盘（两盘的账不对称时 release 又在断言上 panic）	crates/singlefs-core/src/transaction.rs	    for device in allocator.devices.iter().map(|device_map| device_map.device) {	    for device in allocator.devices.iter().map(|device_map| device_map.device).take(1) {	-p singlefs-harness --test second_transaction_supplement_three_bad_disk_input -- every_fixed_panic_site	every_fixed_panic_site_reports_its_own_error_member_instead_of_panicking
@@ -291,9 +278,7 @@
 C506：挂载把 S 读成编译期的 8，不读系统配置槽偏移 362 那一字节	crates/singlefs-core/src/system_configuration.rs	u64::from(bytes[usize::try_from(ROOT_RING_SLOTS_PER_REGION_OFFSET).expect("362")]),	8,	-p singlefs-harness --test system_configuration_slots_per_region -- the_ring_geometry_follows	the_ring_geometry_follows_the_slots_per_region_on_disk_not_a_compile_time_constant
 C506：一个区域按编译期的 8 个槽算，不按传进来的 S 算（mkfs 清根环清错长度）	crates/singlefs-core/src/root_ring.rs	slots_per_region.count() * u64::from(fixed_structure_slot_spacing)	8 * u64::from(fixed_structure_slot_spacing)	-p singlefs-harness --test system_configuration_slots_per_region -- the_ring_geometry_follows	the_ring_geometry_follows_the_slots_per_region_on_disk_not_a_compile_time_constant
 C506：checker 不判每区槽数 S 的区间（按不存在的几何走读根环）	crates/singlefs-checker/src/image.rs	if !(ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PER_REGION_MAXIMUM)\n        .contains(&slots_per_region)\n    {	if false\n        && !(ROOT_RING_SLOTS_PER_REGION_MINIMUM..=ROOT_RING_SLOTS_PER_REGION_MAXIMUM)\n            .contains(&slots_per_region)\n    {	-p singlefs-harness --test system_configuration_slots_per_region -- the_pool_checker_refuses	the_pool_checker_refuses_the_same_out_of_range_slots_per_region_the_mount_refuses
-X1（2026-09-23 代码轮攻方腿）：中央映射树的写侧容量准入整条去掉，装不下时走到 build_index_node 的断言	crates/singlefs-core/src/transaction.rs	    if mapping_entries_of_this_publish > mapping_node_capacity {\n        return Err(PublishError::MappingEntriesExceedOneNode {\n            entries: mapping_entries_of_this_publish,\n            capacity: mapping_node_capacity,\n        });\n    }	    let _ = (mapping_entries_of_this_publish, mapping_node_capacity);	-p singlefs-harness --test second_transaction_mapping_node_admission -- one_inode_leaf_container_past_the_mapping_node_is_refused_before_the_instance_generation_is_acquired	one_inode_leaf_container_past_the_mapping_node_is_refused_before_the_instance_generation_is_acquired
-X1（2026-09-23 代码轮攻方腿）：中央映射树容量准入的门槛写成大于等于，正好装满一个节点的那一次发布被误拒	crates/singlefs-core/src/transaction.rs	    if mapping_entries_of_this_publish > mapping_node_capacity {	    if mapping_entries_of_this_publish >= mapping_node_capacity {	-p singlefs-harness --test second_transaction_mapping_node_admission -- a_publish_whose_mapping_entries_exactly_fill_the_node_passes_admission	a_publish_whose_mapping_entries_exactly_fill_the_node_passes_admission
-C511（2026-09-23 用户定案，I-9.14 射程收窄的那一条）：候选集不再把被回退行判出局的根剔掉（被抛弃时间线的根又进了遍历与并集）	crates/singlefs-checker/src/walk.rs	            let abandoned = instance_table_rows.iter().any(|row| {\n                row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg\n            });	            let abandoned = false;	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
+C511（2026-09-23 用户定案，I-9.14 射程收窄的那一条）：候选集不再把被回退行判出局的根剔掉（被抛弃时间线的根又进了遍历与并集）	crates/singlefs-checker/src/walk.rs	            let abandoned = instance_table_rows.iter().any(|row| {\n                row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg\n            }) || abandoned_by_the_witness(root);\n	            let abandoned = false;\n	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
 C511（2026-09-23 用户定案，I-9.14 收窄反向那一头）：「同一条时间线」读成「同一个实例」，I-9.14 那一遍只拿最新根那个实例的根比（跨过回退行、仍在现行线上的回退目标 A 不再比）	crates/singlefs-checker/src/walk.rs	    judge_tree_table_birth_txg(&scanned, judgements);	    scanned.retain(|root| roots[root.root_index].2.instance == roots[newest_index].2.instance);\n    judge_tree_table_birth_txg(&scanned, judgements);	-p singlefs-harness --test checker_known_bad_images -- birth_txg_that_the_line_after_a_rollback	birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_birth_invariant
 C511（2026-09-23 用户定案，I-9.14 收窄正向那一头）：I-3.9 与 I-9.14 那一遍拿根环里每一条根比，不按实例表把被回退切掉的根剔掉（回到「所有 sighting 相等」）	crates/singlefs-checker/src/walk.rs	    judge_release_generation_and_tree_table_birth(\n        reader,\n        &roots,\n        &candidate_indexes,\n        newest_index,	    judge_release_generation_and_tree_table_birth(\n        reader,\n        &roots,\n        &(0..roots.len()).collect::<Vec<usize>>(),\n        newest_index,	-p singlefs-harness --test checker_known_bad_images -- birth_txg_recorded_differently_only_on_the_timeline	birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_rollback_is_not_compared_and_every_invariant_holds
 C512（2026-09-23 用户定案）：写行那次发布不把分配记录树的根写进根记录（那一版的账重开之后取不回来，被换下的实例表成了空闲槽）	crates/singlefs-core/src/transaction.rs	        mapping_root: previous_root.mapping_root,\n        allocation_record_tree_root,	        mapping_root: previous_root.mapping_root,\n        allocation_record_tree_root: NodePointer::empty_root(),	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- the_third_writable_mount_keeps	the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool
@@ -319,7 +304,7 @@
 步 3 验收：取号之后没有屏障（取号的系统配置槽写与写行那次发布的单元写并成一段）	crates/singlefs-core/src/transaction.rs	    if let Err(cause) = pool.perform(CommitStep::Barrier) {\n        return Err(pool.roll_back_acquisition(&written, previous_instance, cause));\n    }\n    Ok(instance)	    Ok(instance)	-p singlefs-harness --test second_transaction_step_three_acquisition_barrier_layer0 -- no_unit_of_the_new_instance	no_unit_of_the_new_instance_persists_before_the_acquired_instance_in_any_crash_state_of_the_remount
 步 6 验收第 1 条：层 0 按发布分状态数时根槽写那一段归到下一次发布（按段归发布错一位）	crates/singlefs-harness/src/crash.rs	            let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_write_index]);\n            next_root = Some(Layer0PublishOfState::UpToTheRootOf {\n                root_write_index,\n                instance,\n                checkpoint_txg,\n            });\n        }	            publish_of_segment[segment_index] =\n                next_root.unwrap_or(Layer0PublishOfState::AfterTheLastRoot);\n            let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_write_index]);\n            next_root = Some(Layer0PublishOfState::UpToTheRootOf {\n                root_write_index,\n                instance,\n                checkpoint_txg,\n            });\n            continue;\n        }	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
 C481：坏盘输入的基线抽样里去掉「树表 0 条」那一档（报出的基线档数由 2 变 1）	crates/singlefs-harness/src/bad_disk_input.rs	            BaseImageTier::TreeTableWithoutEntries => newest_root_tree_table_has_no_entries(image),	            BaseImageTier::TreeTableWithoutEntries => false,	-p singlefs-harness --test second_transaction_supplement_three_bad_disk_input -- bad_disk_inputs_never_read_back	bad_disk_inputs_never_read_back_an_uncommitted_version_and_panic_only_at_known_sites
-C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷）	crates/singlefs-core/src/mount.rs	    establish_instance(\n        parameters,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            tree_identifier_watermark_of_the_ring,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger: ShadowLedger::On,\n        },\n    )\n}	    let instance_before_acquisition = previous_row.instance;\n    let outcome = establish_instance(\n        parameters,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            tree_identifier_watermark_of_the_ring,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger: ShadowLedger::On,\n        },\n    );\n    if let Err(MountError::Publish(_)) = &outcome {\n        let mut rollback_writer = PoolWriter::new(parameters, devices.as_mut_slice());\n        let _ = rollback_writer.perform(\n            crate::transaction::CommitStep::RotateSystemConfigurationSlots {\n                journal_tail: 0,\n                journal_instance: instance_before_acquisition,\n            },\n        );\n    }\n    outcome\n}	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- a_write_error_after_the_acquisition	a_write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it
+C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷）	crates/singlefs-core/src/mount.rs	    establish_instance(\n        parameters,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            tree_identifier_watermark_of_the_ring,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger: ShadowLedger::On,\n            system_configuration,\n        },\n    )\n}\n	    let instance_before_acquisition = previous_row.instance;\n    let outcome = establish_instance(\n        parameters,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            tree_identifier_watermark_of_the_ring,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger: ShadowLedger::On,\n            system_configuration,\n        },\n    );\n    if let Err(MountError::Publish(_)) = &outcome {\n        let mut rollback_writer = PoolWriter::new(parameters, devices.as_mut_slice());\n        let _ = rollback_writer.perform(\n            crate::transaction::CommitStep::RotateSystemConfigurationSlots {\n                journal_tail: 0,\n                journal_instance: instance_before_acquisition,\n            },\n        );\n    }\n    outcome\n}\n	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- a_write_error_after_the_acquisition	a_write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it
 增补 2 收口第 44 行：I-3.10 的比较恒成立（未释放记录的分配代与单元头里的诞生代号不比）	crates/singlefs-checker/src/walk.rs	            judgements.judge("I-3.10", record.generation == birth_txg, || {	            judgements.judge("I-3.10", true || record.generation == birth_txg, || {	-p singlefs-harness --test checker_known_bad_images -- an_unreleased_allocation_record	an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant
 增补 2 收口第 44 行：I-3.10 射程 ① 放宽（已释放的记录也拿释放代去比单元头的诞生代号）	crates/singlefs-checker/src/walk.rs	            if record.is_released\n                || !examined_records.insert(	            if false\n                || !examined_records.insert(	-p singlefs-harness --test checker_known_bad_images -- an_unreleased_allocation_record	an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant
 增补 2 收口第 44 行：I-3.10 射程 ② 改读末槽（跨两槽的记录不再取起点槽那个单元头）	crates/singlefs-checker/src/walk.rs	                    record.slot * SLOT_BYTES,\n                    UNIT_HEADER_SCAN_BYTES,	                    (record.slot + record.span_slots - 1) * SLOT_BYTES,\n                    UNIT_HEADER_SCAN_BYTES,	-p singlefs-harness --test checker_known_bad_images -- an_unreleased_allocation_record	an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant
@@ -341,7 +326,6 @@
 C511 第 3 步（D8 已定项 8 ②）：回退行那次发布的树 ID 水位沿回退到的那一版带（暖机根的 11），不取根环里的 max；推到 (1, 3) 离开根环之后 I-7.8 判红	crates/singlefs-core/src/mount.rs	        ring.max(version_to_build_on.tree_identifier_watermark)	        version_to_build_on.tree_identifier_watermark	-p singlefs-harness --test second_transaction_step_four_rollback -- after_rolling_back_to_a_warm_up_root	after_rolling_back_to_a_warm_up_root_the_ring_watermark_outlives_the_file_version_roots_leaving_the_ring
 C511 第 3 步（D8 已定项 8 ②）：回退行那次发布的树 ID 水位沿回退到的那一版带，不取根环里的 max；偏向回退的随机历史抽样判出 I-7.8（这一档的必红是抽样断言，随测试周期的种子基重验）	crates/singlefs-core/src/mount.rs	        ring.max(version_to_build_on.tree_identifier_watermark)	        version_to_build_on.tree_identifier_watermark	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rollback_heavy_random_histories	rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms
 C511 第 3 步（D8 已定项 8 ②：号永不重发）：回退到树表 0 条的一版之后再发第一个文件版本，八棵树照 mkfs 的水位 11 重发 11..18，不从那一版带过来的水位起发	crates/singlefs-core/src/transaction.rs	        FileVersionTreeIdentifiers::issued_from_watermark(\n            version_to_build_on.tree_identifier_watermark,\n        )	        FileVersionTreeIdentifiers::issued_from_watermark(\n            TREE_IDENTIFIER_WATERMARK_AT_MKFS,\n        )	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_a_warm_up_root_while	rolling_back_to_a_warm_up_root_while_the_ring_still_holds_a_file_version_carries_the_ring_watermark_and_the_next_first_file_issues_fresh_tree_identifiers
-C511 第 3 步：checker 对中央映射树根的 I-1.3 退回写死树 ID 15，不按根记录里那条根指针的出生树判（回退之后重新发号的映射树判不了）	crates/singlefs-checker/src/walk.rs	            parse_node_pointer(mapping_root_pointer).birth_tree,	            15,	-p singlefs-harness --test checker_known_bad_images -- a_central_mapping_root_whose_header_tree_differs	a_central_mapping_root_whose_header_tree_differs_from_its_root_pointer_birth_tree_reddens_only_the_tree_identifier_invariant
 C511 第 3 步：publish_first_file 退回按「水位 = 11 ⇒ 树还没建」判，不看树表条数；回退到暖机根之后那一版水位 19、树表 0 条，第一个文件版本被拒	crates/singlefs-core/src/transaction.rs	    if tree_table_entries != 0 {	    if version_to_build_on.tree_identifier_watermark != TREE_IDENTIFIER_WATERMARK_AT_MKFS {	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_a_warm_up_root_while	rolling_back_to_a_warm_up_root_while_the_ring_still_holds_a_file_version_carries_the_ring_watermark_and_the_next_first_file_issues_fresh_tree_identifiers
 C511 第 3 步：从水位起连号发八棵树的号不先判装不装得下（盘上读来的水位离 u64::MAX 不到八个号时越界 panic，不交回错误成员）	crates/singlefs-core/src/transaction.rs	            .checked_add(\n                TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH - TREE_IDENTIFIER_WATERMARK_AT_MKFS,\n            )\n            .ok_or(TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees {\n                tree_identifier_watermark,\n            })?;	            .wrapping_add(\n                TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH - TREE_IDENTIFIER_WATERMARK_AT_MKFS,\n            );	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- first_file_version_that_cannot_judge	first_file_version_that_cannot_judge_or_issue_its_trees_is_refused_before_any_write
 C511 第 3 步：publish_first_file 读不出那一版的树表时当成 0 条接着写，不在写之前交回 TreeTableOfTheVersionToBuildOnUnreadable	crates/singlefs-core/src/transaction.rs	        .map_err(|failure| PublishError::TreeTableOfTheVersionToBuildOnUnreadable { failure })?;	        .unwrap_or(0);	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- first_file_version_that_cannot_judge	first_file_version_that_cannot_judge_or_issue_its_trees_is_refused_before_any_write
@@ -368,7 +352,7 @@
 增补 2 第 30 行：程序之后许两个 FLUSH（关机之后多出第二个也放过）	crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs	const ACCEPTED_TRAILING_FLUSHES: usize = 1;	const ACCEPTED_TRAILING_FLUSHES: usize = 2;	-p singlefs-harness --bin first_transaction_device_log_check	device_log_from_another_mode_or_with_anything_after_the_program_is_red_after_the_program
 增补 2 第 30 行：录制流投到盘上时 FUA 写之后不跟 FLUSH（发布 B、挂载、发布 C 三段与设备一层数的对不上）	crates/singlefs-harness/src/device_log.rs	                if operation.kind == RecordedOperationKind::WriteForceUnitAccess {\n                    events.push(DeviceEvent::Flush);	                if operation.kind == RecordedOperationKind::Barrier {\n                    events.push(DeviceEvent::Flush);	-p singlefs-harness --bin first_transaction_on_device	the_recorded_stream_projects_onto_each_device_the_writes_and_flushes_the_device_layer_counted
 增补 2 第 30 行：模式名 second-instance 写成下划线（跑批脚本送来的参数认不回）	crates/singlefs-harness/src/on_device_modes.rs	            OnDeviceRunMode::SecondInstance => "second-instance",	            OnDeviceRunMode::SecondInstance => "second_instance",	-p singlefs-harness --lib -- on_device_modes	every_mode_argument_reads_back_as_the_same_mode_and_unknown_text_is_refused
-增补 2 第 30 行：second-instance 冷重开该读回的取成第二版	crates/singlefs-harness/src/on_device_modes.rs	            PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance => {\n                third_file_content()\n            }	            PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance => {\n                second_file_content()\n            }	-p singlefs-harness --lib -- on_device_modes	the_three_versions_differ_in_length_and_bytes_and_each_mode_reads_back_its_last_one
+增补 2 第 30 行：second-instance 冷重开该读回的取成第二版	crates/singlefs-harness/src/on_device_modes.rs	            PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance => {\n                third_file_content()\n            }	            PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance => {\n                second_file_content()\n            }	-p singlefs-harness --lib -- on_device_modes	the_four_versions_differ_in_length_and_bytes_and_each_mode_reads_back_its_last_one
 增补 2 第 58 行：真设备二进制失败窗口按种类那一侧不算失败账	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	    let failed_publishes: Vec<&WritesByStructureKind> = writes_of_failed_publishes.iter().collect();	    let failed_publishes: Vec<&WritesByStructureKind> = Vec::new();	-p singlefs-harness --bin first_transaction_on_device	second_version_publish_failing_midway_reports_the_writes_it_landed_and_they_equal_the_device_layer_count
 增补 2 第 58 行：发布 B / 发布 C 失败时不交写入口的失败账	crates/singlefs-harness/src/on_device_modes.rs	        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),	        writes_of_failed_publishes: Vec::new(),	-p singlefs-harness --bin first_transaction_on_device	third_version_publish_failing_midway_reports_the_writes_it_landed_and_they_equal_the_device_layer_count
 增补 2 第 21 行（C366，挂载层）：写行那次发布的 jsn 取 txg（所选根自己那条记录读不出时写行 txg 5、jsn 应是 4）	crates/singlefs-core/src/mount.rs	                RowPublishIdentity {\n                    txg: start.first_txg,\n                    counter: start.next_counter,	                RowPublishIdentity {\n                    txg: start.first_txg,\n                    counter: start.first_txg.0,	-p singlefs-harness --test second_transaction_supplement_two_warm_up_counter -- c366_when_the_chosen_root_own_record_is_unreadable	c366_when_the_chosen_root_own_record_is_unreadable_the_new_instance_numbers_records_and_tail_from_the_highest_readable_record
@@ -376,7 +360,6 @@
 增补 2 第 21 行（C366，挂载层）：发布轮换系统配置槽时 tail 写 txg、不写 jsn	crates/singlefs-core/src/transaction.rs	    writer.perform(CommitStep::RotateSystemConfigurationSlots {\n        journal_tail,\n        journal_instance,\n    })	    writer.perform(CommitStep::RotateSystemConfigurationSlots {\n        journal_tail: checkpoint_txg.0,\n        journal_instance,\n    })	-p singlefs-harness --test second_transaction_supplement_two_warm_up_counter -- c366_when_the_chosen_root_own_record_is_unreadable	c366_when_the_chosen_root_own_record_is_unreadable_the_new_instance_numbers_records_and_tail_from_the_highest_readable_record
 增补 2 第 61 行（C519 已知丢失）：施加前的点名单元验证改成任一条位置条目验过即过（改法 Z2 被悄悄落地时已知丢失那条先红）	crates/singlefs-core/src/recovery.rs	                named.locations.iter().all(|location| {	                named.locations.iter().any(|location| {	-p singlefs-harness --test second_transaction_supplement_two_c519_whole_device_loss_after_warm_up -- c519_known_loss	c519_known_loss_after_the_warm_up_covers_both_devices_losing_the_acknowledged_version_root_device_falls_back_to_the_row_publish_version
 增补 2 第 62 行（C533 复现）：前缀跨实例边界，写行那条孤记录进了水位之上	crates/singlefs-core/src/recovery.rs	            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water	            (record.instance, record.checkpoint_txg) > water	-p singlefs-harness --test second_transaction_supplement_two_c533_row_publish_record_without_its_root -- c533_row_publish_record_persisted	c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_are_free_slots_that_the_next_mount_overwrites
-增补 2 第 9 行 P1（C497）：带文件的一版上重写实例表的发布把实例表挪到树表单元之后取落点	crates/singlefs-core/src/transaction.rs	        if let InstanceTablePlan::Rewrite(rewrite) = &self.instance_table {\n            rewritten_roles.extend(instance_table_page_roles_in_bump_order(\n                rewrite.pages_after_this_publish(),\n            ));\n        }\n        if self.file.is_some() {\n            rewritten_roles.push(TransactionUnit::ExtentRoot);\n        }\n        for index in &inode_tree.rewritten {\n            rewritten_roles.push(TransactionUnit::InodeLeafContainer(*index));\n        }\n        if !inode_tree.rewritten.is_empty() {\n            rewritten_roles.push(TransactionUnit::InodeRoot);\n        }\n        rewritten_roles.extend([\n            TransactionUnit::AllocationTree,\n            TransactionUnit::AccountingTree,\n            TransactionUnit::MappingTree,\n            TransactionUnit::TreeTable,\n        ]);\n	        if self.file.is_some() {\n            rewritten_roles.push(TransactionUnit::ExtentRoot);\n        }\n        for index in &inode_tree.rewritten {\n            rewritten_roles.push(TransactionUnit::InodeLeafContainer(*index));\n        }\n        if !inode_tree.rewritten.is_empty() {\n            rewritten_roles.push(TransactionUnit::InodeRoot);\n        }\n        rewritten_roles.extend([\n            TransactionUnit::AllocationTree,\n            TransactionUnit::AccountingTree,\n            TransactionUnit::MappingTree,\n            TransactionUnit::TreeTable,\n        ]);\n        if let InstanceTablePlan::Rewrite(rewrite) = &self.instance_table {\n            rewritten_roles.extend(instance_table_page_roles_in_bump_order(\n                rewrite.pages_after_this_publish(),\n            ));\n        }\n	-p singlefs-harness --test second_transaction_supplement_two_presumed_clause_checks -- c497_every_publish	c497_every_publish_that_rewrites_the_instance_table_bumps_it_before_every_other_commit_generated_block
 增补 2 第 9 行 P1（C497）：树表 0 条的一版上写行时分配记录节点先于实例表取落点	crates/singlefs-core/src/transaction.rs	    let mut instance_table_slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();\n    for identity in &instance_table_roles {\n        let placement = allocate_placement_for_role(allocator, *identity, txg)?;\n        instance_table_slots.insert(*identity, placement.slot);\n    }\n    let allocation_placement =\n        allocate_placement_for_role(allocator, TransactionUnit::AllocationTree, txg)?;\n	    let allocation_placement =\n        allocate_placement_for_role(allocator, TransactionUnit::AllocationTree, txg)?;\n    let mut instance_table_slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();\n    for identity in &instance_table_roles {\n        let placement = allocate_placement_for_role(allocator, *identity, txg)?;\n        instance_table_slots.insert(*identity, placement.slot);\n    }\n	-p singlefs-harness --test second_transaction_supplement_two_presumed_clause_checks -- c497_every_publish	c497_every_publish_that_rewrites_the_instance_table_bumps_it_before_every_other_commit_generated_block
 增补 2 第 9 行 P3（C498）：暖机不看本实例的根覆盖没覆盖两块盘、恒推 R 次	crates/singlefs-core/src/mount.rs	    while all_devices\n        .iter()\n        .any(|identity| !covered.contains(identity))\n        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	    while u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	-p singlefs-harness --test second_transaction_supplement_two_presumed_clause_checks -- c498_warm_up_publishes	c498_warm_up_publishes_after_remounts_rollback_and_a_crash_in_the_middle_of_a_mount_are_counted_by_the_clause
 增补 2 第 9 行 P5（C499）：可写挂载认「上一次干净关闭」（tail 那条就是所选根那次、下一格没有记录）就只取 tail 那一条、不扫全环	crates/singlefs-core/src/mount.rs	    let records = scan_journal(&*devices, &system_configuration);\n    let (journal, effective_root) = replay_journal(	    let tail_record = |counter: u64| {\n        crate::recovery::PoolReader::read(\n            &*devices,\n            devices[0].0,\n            crate::journal::record_offset(\n                counter,\n                system_configuration.immutable.sizes.journal_ring_bytes,\n            ),\n            4096,\n        )\n        .and_then(|bytes| {\n            crate::journal::JournalRecord::parse(\n                &bytes,\n                crate::unit::unit_filesystem_identifier(\n                    &system_configuration.immutable.filesystem_identifier,\n                ),\n            )\n        })\n    };\n    let tail = system_configuration.quantities.journal_tail;\n    let closed_cleanly = tail > 0\n        && tail_record(tail).is_some_and(|record| {\n            record.instance == chosen_root.instance\n                && record.checkpoint_txg == chosen_root.checkpoint_txg\n        })\n        && tail_record(tail + 1).is_none();\n    let records: std::collections::BTreeMap<_, _> = if closed_cleanly {\n        tail_record(tail)\n            .map(|record| ((record.instance, record.counter), record))\n            .into_iter()\n            .collect()\n    } else {\n        scan_journal(&*devices, &system_configuration)\n    };\n    let (journal, effective_root) = replay_journal(	-p singlefs-harness --test second_transaction_supplement_two_presumed_clause_checks -- c499_remount_after	c499_remount_after_a_clean_close_and_after_a_crash_both_scan_the_whole_ring_and_recover_by_the_same_rules
@@ -394,8 +377,6 @@
 C483 ②：挂载态打开时 inode 树根只按位置提示读（不经映射回退）	crates/singlefs-core/src/mounted_read.rs	    let inode_root = read_mapped_tree_root(\n        reader,\n        inode_entry.tree,\n        INODE_KEY_WIDTH_IN_BYTES,\n        &inode_entry.root,\n        root,\n        filesystem_identifier_in_unit_headers,\n        &central_mapping_locations_of_key,\n        &mut tree_node_stale_location_hint_hops,\n    )?;	    let inode_root = read_tree_root(\n        reader,\n        inode_entry.tree,\n        INODE_KEY_WIDTH_IN_BYTES,\n        &inode_entry.root,\n        root,\n        filesystem_identifier_in_unit_headers,\n    )?;	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- an_inode_tree_root_whose	an_inode_tree_root_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping
 C483 ②：挂载态打开时 inode 叶容器只按位置提示读（不经映射回退）	crates/singlefs-core/src/mounted_read.rs	        let container_bytes = read_mapped_tree_node_via_hint_then_central_mapping(\n            reader,\n            &child,\n            MappedTreeNodeClass::PackedRecordUnit,\n            &central_mapping_locations_of_key,\n            &mut tree_node_stale_location_hint_hops,\n        )?;	        let container_bytes = read_unit_via_locations(\n            reader,\n            &child.locations,\n            usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768"),\n        )?;	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- an_inode_leaf_container_whose	an_inode_leaf_container_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping
 C483 ②：冷走读沿树表读树根时只按位置提示读（不经映射回退，挂载态回退了而冷走读没回退）	crates/singlefs-core/src/recovery.rs	            read_mapped_tree_root(\n                reader,\n                entry.tree,\n                key_width,\n                &entry.root,\n                root,\n                expected_filesystem_identifier,\n                &central_mapping_locations_of_key,\n                mapping_fallbacks,\n            )?,	            read_tree_root(\n                reader,\n                entry.tree,\n                key_width,\n                &entry.root,\n                root,\n                expected_filesystem_identifier,\n            )?,	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- an_extent_tree_root_whose	an_extent_tree_root_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping
-C483 ②：冷走读读 inode 叶容器只按位置提示读（不经映射回退）	crates/singlefs-core/src/recovery.rs	        let leaf_bytes = read_mapped_tree_node_via_hint_then_central_mapping(\n            reader,\n            &child,\n            MappedTreeNodeClass::PackedRecordUnit,\n            &|mapping_key: &[u8]| {\n                central_mapping_locations_among_entries(&roots.mapping.entries, mapping_key)\n            },\n            mapping_fallbacks,\n        )?;	        let leaf_bytes = read_unit_via_locations(reader, &child.locations, data_unit_bytes)?;	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- an_inode_leaf_container_whose	an_inode_leaf_container_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping
-C483 ①：挂载态的中央映射长成两层也照收（内部条目被当 55 字节映射条目解，D19 已定项 5「挂载态怎么读映射」的第一版限制没人守）	crates/singlefs-core/src/mounted_read.rs	    if mapping_root.level != CENTRAL_MAPPING_TREE_ROOT_LEVEL {	    if false && mapping_root.level != CENTRAL_MAPPING_TREE_ROOT_LEVEL {	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- central_mapping_grown	central_mapping_grown_into_two_levels_is_refused_at_open_without_reading_past_its_root
 C318 实现：准入读数的「被抛弃根独占量」不取影子账隔离的槽、恒 0（影子账隔离的单元又不进不等式）	crates/singlefs-core/src/admission.rs	                abandoned_root_exclusive: BytesOnOneDevice::of_slots(device_map.isolated_slots()),	                abandoned_root_exclusive: BytesOnOneDevice::ZERO,	-p singlefs-harness --test second_transaction_supplement_two_admission_formula	after_the_rollback_the_admission_reading_with_the_shadow_ledger_is_short_of_the_one_without_it_by_exactly_the_isolated_slots_on_each_device
 C318 实现：准入读数的「被抛弃根独占量」不取影子账隔离的槽、恒 0（从分配器取读数那一处）	crates/singlefs-core/src/admission.rs	                abandoned_root_exclusive: BytesOnOneDevice::of_slots(device_map.isolated_slots()),	                abandoned_root_exclusive: BytesOnOneDevice::ZERO,	-p singlefs-core --lib -- admission::tests	a_reading_of_an_allocator_takes_each_devices_own_capacity_allocated_deferred_and_isolated_slots
 C318 实现：可用(d) 不扣第九项「被抛弃根独占量」	crates/singlefs-core/src/admission.rs	                    terms.mount_time_commitment,\n                    terms.abandoned_root_exclusive,\n	                    terms.mount_time_commitment,\n	-p singlefs-harness --test second_transaction_supplement_two_admission_formula	after_the_rollback_the_admission_reading_with_the_shadow_ledger_is_short_of_the_one_without_it_by_exactly_the_isolated_slots_on_each_device
@@ -422,10 +403,10 @@
 实例表第二片：只拿第 0 片解整张表时把带下一片的第 0 片当整张表	crates/singlefs-core/src/instance_table.rs	            InstanceTableChainRecord::NextPage(_) => None,	            InstanceTableChainRecord::NextPage(_) => Some(Self {\n                rows: first_page.rows,\n            }),	-p singlefs-harness --test second_transaction_supplement_two_instance_table_chain -- the_reader_follows	the_reader_follows_the_chain_into_the_second_page_and_a_single_page_parse_refuses_a_chained_first_page
 实例表第二片：0 行算 0 片	crates/singlefs-core/src/instance_table.rs	    rows.div_ceil(rows_per_page).max(1)	    rows.div_ceil(rows_per_page)	-p singlefs-core --lib -- chain_record_tests	pages_for_rows_is_at_least_one_and_opens_a_page_every_three_hundred_sixty_nine_rows
 实例表第二片：片数向下取整（370 行算一片）	crates/singlefs-core/src/instance_table.rs	    rows.div_ceil(rows_per_page).max(1)	    (rows / rows_per_page).max(1)	-p singlefs-harness --test second_transaction_supplement_two_instance_table_page_full -- writable_mounts_fill	writable_mounts_fill_the_instance_table_page_and_the_three_hundred_seventy_first_opens_the_second_page
-C394（释放判定不核映射条目位置项里的单元校验和）：释放之前读盘核校验和那一核拿掉（位置项的校验和比不比都当对得上），被改坏的那一份照常回到空闲池	crates/singlefs-core/src/transaction.rs	            if crc32_castagnoli(&copy) != location.unit_checksum && !failing.contains(&quarantined)	            if false && crc32_castagnoli(&copy) != location.unit_checksum && !failing.contains(&quarantined)	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_released_unit_whose_copies_fail	a_released_unit_whose_copies_fail_the_checksum_in_its_mapping_entry_is_quarantined_instead_of_returning_to_the_free_pool
+C394（释放判定不核映射条目位置项里的单元校验和）：释放之前读盘核校验和那一核拿掉（位置项的校验和比不比都当对得上），被改坏的那一份照常回到空闲池	crates/singlefs-core/src/transaction.rs	                    Some(copy) if crc32_castagnoli(&copy) != location.unit_checksum => {\n	                    Some(copy) if false && crc32_castagnoli(&copy) != location.unit_checksum => {\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_released_unit_whose_copies_fail	a_released_unit_whose_copies_fail_the_checksum_in_its_mapping_entry_is_quarantined_instead_of_returning_to_the_free_pool
 C394 N3：分配器释放时不看「留在已分配」那几块盘（对不上那一份的记录照样改写成已释放）	crates/singlefs-core/src/allocator.rs	            if devices_whose_record_stays_allocated.contains(&device.device) {\n                continue;\n            }	            let _ = devices_whose_record_stays_allocated;	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_released_unit_whose_copies_fail	a_released_unit_whose_copies_fail_the_checksum_in_its_mapping_entry_is_quarantined_instead_of_returning_to_the_free_pool
 C394 N3（用户 2026-09-24 定：对不上那一份的分配记录留在已分配）：核出对不上之后照常释放那一份（记录改写成已释放）	crates/singlefs-core/src/transaction.rs	            .filter(|copy| copy.placement == *placement)	            .filter(|copy| copy.placement == *placement && false)	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- after_rebuilding_the_previous_version	after_rebuilding_the_previous_version_from_disk_the_release_still_reads_and_quarantines_the_mismatching_copy
-C394 N1：重读还读不出时不按对不上处置、当成核得上照常释放（读不出的那一份回到空闲池）	crates/singlefs-core/src/transaction.rs	                if !failing.contains(&quarantined) {\n                    failing.push(quarantined);\n                }\n                continue;	                continue;	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_release_checksum_read_that_keeps_failing	a_release_checksum_read_that_keeps_failing_is_read_once_more_and_then_handled_as_a_mismatch
+C394 N1：重读还读不出时不按对不上处置、当成核得上照常释放（读不出的那一份回到空闲池）	crates/singlefs-core/src/transaction.rs	                    None => QuarantinedCopyReading::UnreadableAfterOneReread,\n	                    None => QuarantinedCopyReading::IntactButAnotherCopyFailed,\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_release_checksum_read_that_keeps_failing	a_release_checksum_read_that_keeps_failing_is_read_once_more_and_then_handled_as_a_mismatch
 C513（复用豁免不判那次复用合不合法）：回收谓词那一判拿掉（证得出过不了也开脱，回到只看更晚那次写落没落盘）	crates/singlefs-harness/src/crash.rs	    release_generation_at_least <= reclaim_threshold_at_most.0	    release_generation_at_least <= reclaim_threshold_at_most.0 || true	-p singlefs-harness --test second_transaction_supplement_two_record_checker_reuse_legality -- an_illegal_reuse	an_illegal_reuse_whose_later_write_landed_no_longer_excuses_the_missing_unit_in_the_record_checker
 C513：回收谓词判得过严（释放代下界取无穷大，抬 F 之后的合法复用也不开脱，增补 2 收口表第 55 行那 8 个误报回来）	crates/singlefs-harness/src/crash.rs	    let release_generation_at_least = earlier_publish_txg.0.saturating_add(1);	    let release_generation_at_least = u64::MAX;	-p singlefs-harness --test second_transaction_step_zero_layer0 -- stale_tail_with_a_reused_named_unit	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
 并行线一：C490：extent 叶记录 key 的 offset 段写成单元序号（并行线一验收第 4 条第二个变异：读回错位判红）	crates/singlefs-core/src/transaction.rs	        let extent_record = build_extent_record(\n            FIRST_INODE_NUMBER,\n            transaction.payload_start.0,	        let extent_record = build_extent_record(\n            FIRST_INODE_NUMBER,\n            transaction.unit_index_in_file.0,	-p singlefs-harness --test second_transaction_parallel_line_one_sequential_write -- the_second_extent_leaf_record_key_is_the_file_byte_offset_of_the_second_data_unit	the_second_extent_leaf_record_key_is_the_file_byte_offset_of_the_second_data_unit
@@ -436,14 +417,13 @@
 并行线一：从盘上重建上一版只读 extent 根兼叶的第一条记录（多单元文件重开之后只剩第一个数据单元）	crates/singlefs-core/src/recovery.rs	    for extent_record_bytes in &extent_node.entries {	    for extent_record_bytes in extent_node.entries.iter().take(1) {	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- a_reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping	a_reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping
 并行线一：挂载态打开文件不核 extent key 的 offset 段是不是那个单元的文件字节偏移	crates/singlefs-core/src/mounted_read.rs	            if record.file_offset_in_bytes() != expected_file_offset {	            if false {	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- a_mount_state_open_refuses_an_extent_key_whose_offset_segment_is_the_unit_index_instead_of_the_file_byte_offset	a_mount_state_open_refuses_an_extent_key_whose_offset_segment_is_the_unit_index_instead_of_the_file_byte_offset
 并行线一：记录读者不判点名项区越没越过 4096（并行线一验收第 4 条第三个变异：点名项超过 67 仍塞进一条记录 ⇒ 记录解析拒收）	crates/singlefs-core/src/journal.rs	        if payload_end > record_bytes\n            || crc32_castagnoli	        if crc32_castagnoli	-p singlefs-core --lib -- journal::tests::a_record_whose_header_claims_more_named_units_than_one_record_holds_is_refused_by_the_parser	a_record_whose_header_claims_more_named_units_than_one_record_holds_is_refused_by_the_parser
-并行线一：映射条目数不算数据单元（多单元文件的映射节点条目数与准入算的对不上）	crates/singlefs-core/src/transaction.rs	        + data_units_after_this_publish		-p singlefs-harness --test second_transaction_parallel_line_one_sequential_write -- a_publish_of_two_data_units_names_the_shared_commit_generated_units_only_in_its_last_record	a_publish_of_two_data_units_names_the_shared_commit_generated_units_only_in_its_last_record
 并行线一：一个数据单元的发布也切成两条记录（末条只点名共享内生块）	crates/singlefs-core/src/transaction.rs	    let data_roles_named_before_the_last_record = &data_roles[..data_roles.len().saturating_sub(1)];	    let data_roles_named_before_the_last_record = &data_roles[..];	-p singlefs-harness --test second_transaction_parallel_line_one_sequential_write -- a_sequential_write_that_fits_one_data_unit_publishes_through_the_single_transaction_path	a_sequential_write_that_fits_one_data_unit_publishes_through_the_single_transaction_path
 并行线一：层 0：共享内生块挪到第一条记录里点名（里程碑并行线一验收第 4 条第一个变异「提交标记提前到第一条记录」在一事务一记录下的形态：半次发布施加上去）	crates/singlefs-core/src/transaction.rs	    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles_named_before_the_last_record\n            .iter()\n            .map(|data_role| vec![*data_role])\n            .collect();\n    roles_named_by_each_record.push(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !data_roles_named_before_the_last_record.contains(identity))\n            .collect(),\n    );	    let _ = data_roles_named_before_the_last_record;\n    if data_roles.len() < 2 {\n        return vec![rewritten.to_vec()];\n    }\n    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles.iter().map(|data_role| vec![*data_role]).collect();\n    roles_named_by_each_record[0].extend(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !matches!(identity, TransactionUnit::Data(_))),\n    );	-p singlefs-harness --test second_transaction_parallel_line_one_layer0 -- every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it	every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it
 并行线一：层 0：发布边界不认（每条记录都当末条）	crates/singlefs-core/src/recovery.rs	        if !record_ends_its_publish(record) {	        if false {	-p singlefs-harness --test second_transaction_parallel_line_one_layer0 -- every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it	every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it
 步 1 验收第 4 条（第 331 行那条锚点腐化了，照原意重写锚点）：extent 叶记录的指针忘了换（覆盖写之后仍指上一版的第一个数据单元，读回等于旧内容）	crates/singlefs-core/src/transaction.rs	        (Some(file), _) => build_file_version_units(\n            &checkpoint,\n            file,\n            &file_content_transactions,\n            &FileVersionSlots {\n                data: file_content_transactions\n                    .iter()\n                    .map(|transaction| {\n                        slot_of(TransactionUnit::Data(transaction.unit_index_in_file))\n                    })\n                    .collect(),\n            },\n            &mut sequences,\n        ),	        (Some(file), _) => {\n            let mut built = build_file_version_units(\n                &checkpoint,\n                file,\n                &file_content_transactions,\n                &FileVersionSlots {\n                    data: file_content_transactions\n                        .iter()\n                        .map(|transaction| {\n                            slot_of(TransactionUnit::Data(transaction.unit_index_in_file))\n                        })\n                        .collect(),\n                },\n                &mut sequences,\n            );\n            if let Some(previous_version) = previous {\n                let stale_extent_record =\n                    build_extent_record(FIRST_INODE_NUMBER, 0, previous_version.data_pointers[0]);\n                let stale_extent_key: [u8; 24] =\n                    stale_extent_record[..24].try_into().expect("24");\n                built.extent_unit = build_index_node(\n                    TreeIdentifier(TREE_IDENTIFIER_EXTENT),\n                    0,\n                    24,\n                    &stale_extent_key,\n                    &stale_extent_key,\n                    txg,\n                    filesystem_identifier,\n                    instance,\n                    built.extent_sequence,\n                    112,\n                    std::slice::from_ref(&stale_extent_record),\n                );\n            }\n            built\n        }	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start_reads_the_second_content_and_the_pool_checker_stays_green	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
 并行线一：数据单元头里的锚点偏移写成单元序号（与 extent key 的 offset 段不符：冷启动顺序读在第 1 个单元当场拒，I-1.1）	crates/singlefs-core/src/transaction.rs	            anchor_offset: transaction.payload_start.0,	            anchor_offset: transaction.unit_index_in_file.0,	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte	units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte
 并行线一：数据单元的净荷字节倒序写（长度、锚点、写序都对，只有内容错：冷启动顺序读回与写入逐字节比对判红）	crates/singlefs-core/src/transaction.rs	            transaction.payload_of(file.content),	            &transaction.payload_of(file.content).iter().rev().copied().collect::<Vec<u8>>(),	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte	units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte
-并行线一：C394：释放之前读盘核校验和只走这次重写的角色（文件变短换下的尾巴不核，被改坏的那一份照常回到空闲池）	crates/singlefs-core/src/transaction.rs	    let mut failing = Vec::new();\n    for identity in roles_replaced_via_mapping(previous, roles) {	    let mut failing = Vec::new();\n    for identity in roles.iter().copied() {	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- shrinking_a_multi_unit_file_checks_the_released_tail_against_its_mapping_checksums_before_releasing_it	shrinking_a_multi_unit_file_checks_the_released_tail_against_its_mapping_checksums_before_releasing_it
+并行线一：C394：释放之前读盘核校验和只走这次重写的角色（文件变短换下的尾巴不核，被改坏的那一份照常回到空闲池）	crates/singlefs-core/src/transaction.rs	    roles_replaced_via_mapping(previous, roles)\n        .into_iter()\n	    roles\n        .iter()\n        .copied()\n	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- shrinking_a_multi_unit_file_checks_the_released_tail_against_its_mapping_checksums_before_releasing_it	shrinking_a_multi_unit_file_checks_the_released_tail_against_its_mapping_checksums_before_releasing_it
 岔路 7（G27）立的 I-3.11（已分配减 defer 等于最新根走读）：判定恒真（defer 账对不上也判绿）	crates/singlefs-checker/src/walk.rs	                matches!((allocated, deferred), (Some(allocated), Some(deferred)) if deferred.checked_add(referenced_by_the_newest_root) == Some(allocated)),	                true || matches!((allocated, deferred), (Some(allocated), Some(deferred)) if deferred.checked_add(referenced_by_the_newest_root) == Some(allocated)),	-p singlefs-harness --test checker_known_bad_images -- live_unit_recorded_as_deferred	live_unit_recorded_as_deferred_reddens_only_the_allocated_minus_deferred_invariant
 I-3.11 不减第 5 项（判别力自证：造出来的 defer 为 0 的基底上再记一槽 defer，判定由红转绿）	crates/singlefs-checker/src/walk.rs	deferred.checked_add(referenced_by_the_newest_root) == Some(allocated)	Some(referenced_by_the_newest_root) == Some(allocated)	-p singlefs-harness --test checker_known_bad_images -- on_a_synthetic_base	on_a_synthetic_base_with_an_empty_defer_queue_one_deferred_slot_reddens_the_allocated_minus_deferred_invariant
 I-3.11 的等号放宽成 ≥（defer 多记一槽那份坏镜像判绿）	crates/singlefs-checker/src/walk.rs	deferred.checked_add(referenced_by_the_newest_root) == Some(allocated)	deferred.checked_add(referenced_by_the_newest_root) >= Some(allocated)	-p singlefs-harness --test checker_known_bad_images -- live_unit_recorded_as_deferred	live_unit_recorded_as_deferred_reddens_only_the_allocated_minus_deferred_invariant
@@ -471,18 +451,17 @@
 代码三方 m2-wave3-code-r1 第四节第 2 条（改法 E）：I-3.10 不读下一次挂载会先施加的那一版的分配记录树（B 的记录已落、根槽没落时分配代写错，崩溃镜像上不红）	crates/singlefs-checker/src/walk.rs	    tree_table_pointers.extend(tree_table_pointer_of_the_version_the_next_mount_applies_first);	    let _ = tree_table_pointer_of_the_version_the_next_mount_applies_first;	-p singlefs-harness --test checker_known_bad_images -- an_allocation_generation_past_its_unit_birth	an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_record_carries_reddens_the_allocation_generation_invariant_before_the_mount
 代码三方 m2-wave3-code-r1 第四节第 3 条（Y5）：带文件的一版把自己的分配记录树根也写进根记录（D16 已定项 9：带文件的一版这一项写全 0）	crates/singlefs-core/src/transaction.rs	        allocation_record_tree_root: NodePointer::empty_root(),	        allocation_record_tree_root: allocation_pointer,	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- root_record_of_a_file_version	root_record_of_a_file_version_leaves_the_allocation_record_tree_pointer_zero
 代码三方 m2-wave3-code-r1 第四节第 4 条：tree_table_entry_count 恒报 0（影子账把带文件的被抛弃根当成树表 0 条的一版，只隔离实例表与树表）	crates/singlefs-core/src/recovery.rs	    Ok(tree_table.entries.len())	    Ok(tree_table.entries.len().min(0))	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root_writes_the_rollback_row	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
-代码三方 m2-wave3-code-r1 第四节第 4 条：只读挂载认中央映射树的根退回写死树 ID 15，不按根记录里映射根指针的出生树（回退之后再发的第一个文件版本映射树是 23，打不开）	crates/singlefs-core/src/mounted_read.rs	    let mapping_tree = root.mapping_root.head.birth_tree;	    let mapping_tree = TreeIdentifier(15);	-p singlefs-harness --test second_transaction_step_four_rollback -- the_read_only_mount_after_rolling_back	the_read_only_mount_after_rolling_back_to_a_warm_up_root_finds_the_central_mapping_under_the_tree_its_root_pointer_names
 代码三方 m2-wave3-code-r1 第四节第 4 条：根记录里分配记录树根指针的偏移错一位（写者 342 写到 343，记录总长 457 不变；全零指针的往返单测看不出）	crates/singlefs-core/src/root_record.rs	        self.allocation_record_tree_root.write_to(&mut writer);\n        writer.put_u8(0); // 算法类型：未加密\n        writer.skip(12 + 16); // nonce、MAC：第一版留位全 0	        writer.skip(1);\n        self.allocation_record_tree_root.write_to(&mut writer);\n        writer.put_u8(0); // 算法类型：未加密\n        writer.skip(12 + 16 - 1); // nonce、MAC：第一版留位全 0	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- root_record_of_a_file_version	root_record_of_a_file_version_leaves_the_allocation_record_tree_pointer_zero
 E158 root_choice_repair Q2-1 的 candidate_fault_sets_of_weight 权重公式：一条记录按 1 个单位算（该按 2 份自证算）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	let weight_one_count = weight - record_count * 2;	let weight_one_count = weight - record_count;	-p singlefs-harness --bin e158_root_choice_repair	candidate_fault_sets_of_weight_mixes_root_slots_and_journal_records_by_weight
 E158 root_choice_repair Q2-1 的 judge_recovery_outcome「倒挂」判定取反（该在根不在时间线上时判倒挂，改成在时间线上时判）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	if !timeline.contains(&chosen) {	if timeline.contains(&chosen) {	-p singlefs-harness --bin e158_root_choice_repair	judge_recovery_outcome_passes_when_root_on_timeline_and_content_matches
 E158 root_choice_repair Q2-1 的 candidate_fault_sets_of_weight 起点错位（记录计数从 1 起，权重 0 时枚不出空集合）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	let mut record_count = 0u64;	let mut record_count = 1u64;	-p singlefs-harness --bin e158_root_choice_repair	candidate_fault_sets_of_weight_at_zero_is_only_the_empty_set
 P6 后一半：锚点读不出时不看本次发布内序号，链首接在下一次发布的第二条上（三方第一轮 K4-b）	crates/singlefs-core/src/recovery.rs	            || record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST\n	            || false\n	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish	without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish
-P6 后一半：多条记录的发布每条都写本次发布内序号 1	crates/singlefs-core/src/transaction.rs	            ordinal_within_publish: JournalRecordOrdinalWithinPublish::of_record_at_offset(\n                record_offset_in_this_publish,\n            ),	            ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish	without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish
+P6 后一半：多条记录的发布每条都写本次发布内序号 1	crates/singlefs-core/src/transaction.rs	            // 恢复只从序号 1 那条接链首（已定项 14 注 1）。\n            ordinal_within_publish: JournalRecordOrdinalWithinPublish::of_record_at_offset(\n                record_offset_in_this_publish,\n            ),\n	            // 恢复只从序号 1 那条接链首（已定项 14 注 1）。\n            ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,\n	-p singlefs-harness --test second_transaction_parallel_line_one_multi_unit_file -- without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish	without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish
 P6 后一半：本次发布内序号从 0 起	crates/singlefs-core/src/journal.rs	            u32::try_from(record_offset_in_this_publish + 1)	            u32::try_from(record_offset_in_this_publish)	-p singlefs-core --lib -- journal::tests::the_ordinal_of_the_record_at_offset_k_of_a_publish_is_k_plus_one	journal::tests::the_ordinal_of_the_record_at_offset_k_of_a_publish_is_k_plus_one
 P6 后一半：记录头写者不写本次发布内序号（恒写 1）	crates/singlefs-core/src/journal.rs	        writer.put_u32(self.ordinal_within_publish.0);	        writer.put_u32(1);	-p singlefs-core --lib -- journal::tests::the_ordinal_within_publish_sits_right_after_the_commit_marker_and_round_trips	journal::tests::the_ordinal_within_publish_sits_right_after_the_commit_marker_and_round_trips
 P6 后一半：记录头读者跳过本次发布内序号、一律当 1	crates/singlefs-core/src/journal.rs	        let ordinal_within_publish = JournalRecordOrdinalWithinPublish(reader.get_u32());	        let ordinal_within_publish = {\n            reader.skip(4);\n            JournalRecordOrdinalWithinPublish::FIRST\n        };	-p singlefs-core --lib -- journal::tests::the_ordinal_within_publish_sits_right_after_the_commit_marker_and_round_trips	journal::tests::the_ordinal_within_publish_sits_right_after_the_commit_marker_and_round_trips
 P6 后一半：空发布记录的本次发布内序号写成 2	crates/singlefs-core/src/transaction.rs	        // 空发布记录也写 1（D23（journal 的角色与格式） 已定项 4）：它一条就是一次发布。\n        ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,	        // 空发布记录也写 1（D23（journal 的角色与格式） 已定项 4）：它一条就是一次发布。\n        ordinal_within_publish: JournalRecordOrdinalWithinPublish(2),	-p singlefs-harness --test first_transaction_step_five_publish -- root_slots_system_configurations_and_journal_ring_hold_the_published_state	root_slots_system_configurations_and_journal_ring_hold_the_published_state
-P6 后一半：树表 0 条那一版上写行那次发布的本次发布内序号写成 2	crates/singlefs-core/src/transaction.rs	        // 这次发布只有这一条记录（D23（journal 的角色与格式） 已定项 4：只有一条时是 1）。\n        ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,	        // 这次发布只有这一条记录（D23（journal 的角色与格式） 已定项 4：只有一条时是 1）。\n        ordinal_within_publish: JournalRecordOrdinalWithinPublish(2),	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- writable_mount_after_a_crash_right_after_acquiring_an_instance	writable_mount_after_a_crash_right_after_acquiring_an_instance_writes_a_row_for_the_burnt_instance
+P6 后一半：树表 0 条那一版上写行那次发布的本次发布内序号写成 2	crates/singlefs-core/src/transaction.rs	            ordinal_within_publish: JournalRecordOrdinalWithinPublish::of_record_at_offset(\n                record_offset_in_this_publish,\n            ),\n            place_in_publish: if transaction_of_the_next_record.is_none() {\n	            ordinal_within_publish: JournalRecordOrdinalWithinPublish::of_record_at_offset(\n                record_offset_in_this_publish + 1,\n            ),\n            place_in_publish: if transaction_of_the_next_record.is_none() {\n	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- writable_mount_after_a_crash_right_after_acquiring_an_instance	writable_mount_after_a_crash_right_after_acquiring_an_instance_writes_a_row_for_the_burnt_instance
 记录头 311：池级 checker 读本次发布内序号的偏移错成反向链那一格	crates/singlefs-checker/src/lib.rs	const JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET: usize = 87;	const JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET: usize = 91;	-p singlefs-harness --test first_transaction_step_five_publish -- root_slots_system_configurations_and_journal_ring_hold_the_published_state	root_slots_system_configurations_and_journal_ring_hold_the_published_state
 记录头 311：池级 checker 的反向链偏移没跟着后挪 4 字节	crates/singlefs-checker/src/lib.rs	const JOURNAL_BACK_CHAIN_OFFSET: usize = 91;	const JOURNAL_BACK_CHAIN_OFFSET: usize = 87;	-p singlefs-harness --test first_transaction_step_five_publish -- root_slots_system_configurations_and_journal_ring_hold_the_published_state	root_slots_system_configurations_and_journal_ring_hold_the_published_state
 记录头 311：池级 checker 的载荷校验和偏移没跟着后挪 4 字节	crates/singlefs-checker/src/lib.rs	const JOURNAL_PAYLOAD_CHECKSUM_OFFSET: usize = 95;	const JOURNAL_PAYLOAD_CHECKSUM_OFFSET: usize = 91;	-p singlefs-harness --test first_transaction_step_five_publish -- root_slots_system_configurations_and_journal_ring_hold_the_published_state	root_slots_system_configurations_and_journal_ring_hold_the_published_state
@@ -519,7 +498,6 @@
 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时不记根（不转环、不回收），拷贝上被拒的那一次与真发的不同	crates/singlefs-core/src/mount.rs	        copy.record_root_written_by_this_process(txg);	        // 变异：拷贝上不记根（不转环、不回收）	-p singlefs-harness --test second_transaction_supplement_three_random_history -- a_writable_mount_whose_own_publishes	a_writable_mount_whose_own_publishes_find_no_placement_is_refused_before_acquisition_with_the_disk_unchanged
 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时不释放换下的落点（回退到环里最旧的根时写行当场回收的几片拷贝上看不见，真发得出来的回退被拒）	crates/singlefs-core/src/mount.rs	            copy.release(placement, txg);	            let _ = (placement, txg);	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_the_oldest_ring_root	rolling_back_to_the_oldest_ring_root_reuses_what_the_row_publish_released_and_is_not_refused_before_acquisition
 增补 2 收口表第 39 行那一族：胶水把「取号之前在拷贝上取不到落点」映射成模型没有的理由（单元区墙那一格对不上）	crates/singlefs-harness/src/model_comparison.rs	        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {\n            refusal, ..\n        } => refusal_reason_of_placement_refusal(refusal),	        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. } => {\n            ObservedRefusalReason::Unexplained\n        }	-p singlefs-harness --test second_transaction_supplement_three_random_history -- a_writable_mount_whose_own_publishes	a_writable_mount_whose_own_publishes_find_no_placement_is_refused_before_acquisition_with_the_disk_unchanged
-增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，树表 0 条那一版上写行只取实例表、漏了分配记录树节点（拷贝上取的与真发的不同，挂载自己的断言判出）	crates/singlefs-core/src/mount.rs	                instance_table_page_roles_in_bump_order(\n                    instance_table_rewrite.pages_after_this_publish(),\n                )\n                .into_iter()\n                .chain(std::iter::once(TransactionUnit::AllocationTree))\n                .collect(),	                instance_table_page_roles_in_bump_order(\n                    instance_table_rewrite.pages_after_this_publish(),\n                )\n                .into_iter()\n                .collect(),	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_formatted_pool_mounted_twice	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
 收口表第 26 行 I-7.9：判法拿掉（抬 F 的根带的 F 高于上限也判成立）	crates/singlefs-checker/src/walk.rs	        judgements.judge("I-7.9", raised_floor <= ceiling.lowest_possible, || {	        judgements.judge("I-7.9", true, || {	-p singlefs-harness --test checker_known_bad_images -- raising_the_floor_above_its_ceiling	raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant
 收口表第 26 行 I-7.9：上限算法改宽（每块盘上最新的有效根与第 4 新的非空有效根取大，不取小）	crates/singlefs-checker/src/walk.rs	    newest_valid_root_txg_on_every_device.min(fourth_newest_non_empty_or_oldest_valid)	    newest_valid_root_txg_on_every_device.max(fourth_newest_non_empty_or_oldest_valid)	-p singlefs-harness --test checker_known_bad_images -- raising_the_floor_above_its_ceiling	raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant
 收口表第 26 行 I-7.9：上限算法改宽（第 4 新的非空有效根改成第 3 新的）	crates/singlefs-checker/src/walk.rs	    let fourth_newest_non_empty_or_oldest_valid = newest_first\n        .get(3)	    let fourth_newest_non_empty_or_oldest_valid = newest_first\n        .get(2)	-p singlefs-harness --test checker_known_bad_images -- raising_the_floor_above_its_ceiling	raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant
@@ -545,7 +523,7 @@
 实二十：末条再跨记录时不装满 67 项就开下一条（替换第 440 行那条锚点已腐化的变异）	crates/singlefs-core/src/transaction.rs	                .chunks(named_unit_capacity)	                .chunks(named_unit_capacity - 1)	-p singlefs-harness --test second_transaction_parallel_line_three_many_inodes -- a_publish_naming_more_units_than_one_journal_record_holds_spills	a_publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transaction_over_two_records_and_only_the_second_ends_the_publish
 实二十：一个角色都不点名的发布一条记录都不切出来	crates/singlefs-core/src/transaction.rs	    if roles_named_by_the_last_transaction.is_empty() {	    if false {	-p singlefs-core --lib -- transaction::tests::the_last_transaction_spills_over_as_many_records_as_its_named_units_need	transaction::tests::the_last_transaction_spills_over_as_many_records_as_its_named_units_need_and_they_all_share_its_transaction
 实二十：空发布记录不带末条标志	crates/singlefs-core/src/transaction.rs	        // 含空发布记录，那一条记录标志位 0 也写 1）。\n        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,	        // 含空发布记录，那一条记录标志位 0 也写 1）。\n        place_in_publish: JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,	-p singlefs-harness --test first_transaction_step_five_publish -- root_slots_system_configurations_and_journal_ring_hold_the_published_state	root_slots_system_configurations_and_journal_ring_hold_the_published_state
-实二十：树表 0 条那一版上写行那次发布的记录不带末条标志	crates/singlefs-core/src/transaction.rs	        // 只有这一条 ⇒ 它就是这次发布的末条（D23（journal 的角色与格式） 已定项 17）。\n        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,	        // 只有这一条 ⇒ 它就是这次发布的末条（D23（journal 的角色与格式） 已定项 17）。\n        place_in_publish: JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- writable_mount_after_a_crash_right_after_acquiring_an_instance	writable_mount_after_a_crash_right_after_acquiring_an_instance_writes_a_row_for_the_burnt_instance
+实二十：树表 0 条那一版上写行那次发布的记录不带末条标志	crates/singlefs-core/src/transaction.rs	            place_in_publish: if transaction_of_the_next_record.is_none() {\n                JournalRecordPlaceInPublish::LastRecordOfThePublish\n            } else {\n	            place_in_publish: if transaction_of_the_next_record.is_none() {\n                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow\n            } else {\n	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- writable_mount_after_a_crash_right_after_acquiring_an_instance	writable_mount_after_a_crash_right_after_acquiring_an_instance_writes_a_row_for_the_burnt_instance
 实二十：锚点取读法甲——所选根那次发布读得出的记录里 jsn 最大那条，不看末条标志（替换第 438 行那条锚点已腐化的变异）	crates/singlefs-core/src/recovery.rs	                && record_ends_its_publish(record)\n        })\n        .map(|record| record.counter)\n        .collect();	        })\n        .map(|record| record.counter)\n        .max()\n        .into_iter()\n        .collect();	-p singlefs-harness --test second_transaction_parallel_line_one_last_record_flag -- when_the_last_record_of_the_chosen_roots_publish_is_unreadable	when_the_last_record_of_the_chosen_roots_publish_is_unreadable_its_readable_earlier_record_is_no_anchor_and_the_next_publish_starts_the_chain
 实二十：所选根那次发布带末条标志的多于一条时不停下（条款没写那一格照常接链）	crates/singlefs-core/src/recovery.rs	    if counters_carrying_the_last_record_flag.len() > 1 {	    if false {	-p singlefs-harness --test second_transaction_parallel_line_one_last_record_flag -- two_last_record_flags_in_the_chosen_roots_publish	two_last_record_flags_in_the_chosen_roots_publish_stop_recovery_and_a_writable_mount_before_any_write
 实二十：一次发布之内跳号不断链	crates/singlefs-core/src/recovery.rs	            if u64::from(record.ordinal_within_publish.0)\n                != u64::from(previous_record_of_the_open_publish.ordinal_within_publish.0) + 1\n            {	            if false {	-p singlefs-harness --test second_transaction_parallel_line_one_last_record_flag -- an_ordinal_that_skips_within_a_publish	an_ordinal_that_skips_within_a_publish_breaks_the_chain_and_the_publish_is_not_applied
@@ -566,12 +544,12 @@
 E158 root_choice_repair 候选 (b) 分配记录树指称「走全」（session s8）：树表自己的节点漏并进占用集合（该并进 root.tree_table.locations，改成重复并中央映射树自己的节点）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	    for location in &root.tree_table.locations {\n        placements.insert((location.device.0, location.slot.0));\n    }\n    let tree_table_bytes	    for location in &root.mapping_root.locations {\n        placements.insert((location.device.0, location.slot.0));\n    }\n    let tree_table_bytes	-p singlefs-harness --bin e158_root_choice_repair	allocation_record_tree_reachable_placements_via_all_structural_trees_is_a_superset_of_the_central_mapping_only_version
 E158 root_choice_repair 候选 (b) 分配记录树指称「走全」：中央映射树自己的节点漏并进占用集合（该并进 root.mapping_root.locations，改成重复并树表自己的节点）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	    let mut placements =\n        allocation_record_tree_reachable_placements_via_central_mapping(plain_devices, root)?;\n    for location in &root.mapping_root.locations {	    let mut placements =\n        allocation_record_tree_reachable_placements_via_central_mapping(plain_devices, root)?;\n    for location in &root.tree_table.locations {	-p singlefs-harness --bin e158_root_choice_repair	allocation_record_tree_reachable_placements_via_all_structural_trees_is_a_superset_of_the_central_mapping_only_version
 E158 root_choice_repair session s9：classify_diff_pair 的实例表节点判定改成拿 tree_table（该按 root.instance_table.locations 判，改成按 root.tree_table.locations 判——171 格里那两个落点会被错判成别的归类）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	if root\n        .instance_table\n        .locations\n        .iter()\n        .any(|location| location_key(location) == pair)\n    {\n        return "instance_table_self_node_not_walked_by_the_all_structural_trees_arm";	if root\n        .tree_table\n        .locations\n        .iter()\n        .any(|location| location_key(location) == pair)\n    {\n        return "instance_table_self_node_not_walked_by_the_all_structural_trees_arm";	-p singlefs-harness --bin e158_root_choice_repair	classify_diff_pair_identifies_the_instance_table_self_node
-C394 N1（用户 2026-09-24 定：读盘失败先重读一次）：读不出不重读、当场按对不上处置（瞬时读错把好的那一份隔离掉）	crates/singlefs-core/src/transaction.rs	            let Some(copy) = read_the_copy().or_else(read_the_copy) else {	            let Some(copy) = read_the_copy() else {	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_release_checksum_read_that_fails_once	a_release_checksum_read_that_fails_once_is_read_again_and_the_intact_copy_is_released_as_usual
-C394 N1：「先重读一次」写成重读两次（条款只重读一次，不多次重试）	crates/singlefs-core/src/transaction.rs	            let Some(copy) = read_the_copy().or_else(read_the_copy) else {	            let Some(copy) = read_the_copy().or_else(read_the_copy).or_else(read_the_copy) else {	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_release_checksum_read_that_keeps_failing	a_release_checksum_read_that_keeps_failing_is_read_once_more_and_then_handled_as_a_mismatch
+C394 N1（用户 2026-09-24 定：读盘失败先重读一次）：读不出不重读、当场按对不上处置（瞬时读错把好的那一份隔离掉）	crates/singlefs-core/src/transaction.rs	                let reading = match read_the_copy().or_else(read_the_copy) {\n	                let reading = match read_the_copy() {\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_release_checksum_read_that_fails_once	a_release_checksum_read_that_fails_once_is_read_again_and_the_intact_copy_is_released_as_usual
+C394 N1：「先重读一次」写成重读两次（条款只重读一次，不多次重试）	crates/singlefs-core/src/transaction.rs	                let reading = match read_the_copy().or_else(read_the_copy) {\n	                let reading = match read_the_copy().or_else(read_the_copy).or_else(read_the_copy) {\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_release_checksum_read_that_keeps_failing	a_release_checksum_read_that_keeps_failing_is_read_once_more_and_then_handled_as_a_mismatch
 C394 N3：核出对不上之后照常释放那一份，重挂回收之后那一槽被再发出去（隔离不跨重挂）	crates/singlefs-core/src/transaction.rs	            .filter(|copy| copy.placement == *placement)	            .filter(|copy| copy.placement == *placement && false)	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_quarantined_copy_stays_allocated	a_quarantined_copy_stays_allocated_across_a_remount_and_is_never_handed_out_again
 C394 N3：核出对不上之后照常释放那一份，准入里它不再算已分配（两块盘的可用一样多）	crates/singlefs-core/src/transaction.rs	            .filter(|copy| copy.placement == *placement)	            .filter(|copy| copy.placement == *placement && false)	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_quarantined_copy_counts_as_allocated	a_quarantined_copy_counts_as_allocated_in_the_admission_reading_and_adds_no_term_of_its_own
-C394 N3（2026-09-24 故障注入快档打中的那一格）：只一部分副本对不上时照做、不在写之前停（两块盘的分配记录从此不对称，冷启动走读判整池失败）	crates/singlefs-core/src/transaction.rs	        if !devices_whose_copy_failed.is_empty() && !every_copy_failed {	        if false && !devices_whose_copy_failed.is_empty() && !every_copy_failed {	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_checksum_failure_on_only_one_copy	a_checksum_failure_on_only_one_copy_returns_the_unsupported_member_before_anything_is_written
-C394：位置项指的盘不在池里时不在写之前交回点名条款没写的成员（当成读不出、按对不上处置，却没有那块盘的记录可留）	crates/singlefs-core/src/transaction.rs	            if !pool_devices.contains(&location.device) {	            if false && !pool_devices.contains(&location.device) {	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_mapping_location_on_a_device_outside_the_pool	a_mapping_location_on_a_device_outside_the_pool_returns_the_undecided_clause_member_before_anything_is_written
+C394 × D19 已定项 5（用户 2026-09-25 定：任一份对不上两块盘一起留）：只留对不上的那一份、对得上的那一份照常释放（两块盘的账不对称：冷启动走读判整池失败）	crates/singlefs-core/src/transaction.rs	        for (location, reading) in reading_of_each_copy {\n	        for (location, reading) in reading_of_each_copy\n            .into_iter()\n            .filter(|(_, reading)| *reading != QuarantinedCopyReading::IntactButAnotherCopyFailed)\n        {\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_checksum_failure_on_only_one_copy	a_checksum_failure_on_only_one_copy_keeps_both_records_allocated_and_the_pool_stays_writable_and_readable
+D19 已定项 5（用户 2026-09-25 定）：映射条目的位置项指池外的盘时不当映射条目损坏拒（接着读盘核那一份，没有那块盘的记录可留）	crates/singlefs-core/src/transaction.rs	            .find(|location| !pool_devices.contains(&location.device))\n	            .find(|location| false && !pool_devices.contains(&location.device))\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_mapping_location_on_a_device_outside_the_pool	a_mapping_location_on_a_device_outside_the_pool_is_refused_as_a_damaged_mapping_entry_before_anything_is_written
 实例表第二片写路径（D3 已定项 10 ⑤，用户 2026-09-24 定尾片先）：各片在 bump 次序里第 0 片先（头片先）	crates/singlefs-core/src/transaction.rs	    (0..pages)\n        .rev()\n	    (0..pages)\n	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- a_row_publish_past_one_page	a_row_publish_past_one_page_writes_the_tail_page_first_and_the_first_page_chains_to_it
 实例表第二片写路径（D18 已定项 11，用户 2026-09-24 定一片写满 369 行再开下一片）：一片只写 368 行就开下一片	crates/singlefs-core/src/instance_table.rs	    rows.chunks(instance_rows_per_page()).collect()	    rows.chunks(instance_rows_per_page() - 1).collect()	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- a_row_publish_past_one_page	a_row_publish_past_one_page_writes_the_tail_page_first_and_the_first_page_chains_to_it
 实例表第二片写路径：链指针记录「有下一片」写成 0（读者按「无下一片而指针不全零」拒收，两片的表读不出）	crates/singlefs-core/src/instance_table.rs	                writer.put_u8(CHAIN_RECORD_HAS_NEXT_PAGE);	                writer.put_u8(CHAIN_RECORD_NO_NEXT_PAGE);	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- a_row_publish_past_one_page	a_row_publish_past_one_page_writes_the_tail_page_first_and_the_first_page_chains_to_it
@@ -583,8 +561,93 @@
 实例表第二片写路径（D18 已定项 11：0 行也是一片）：0 行时切出 0 片（写者按片数取落点、按切片装片，两者不等）	crates/singlefs-core/src/instance_table.rs	    if rows.is_empty() {\n        return vec![rows];\n    }	    if false {\n        return vec![rows];\n    }	-p singlefs-core --lib -- chain_record_tests	rows_fill_a_page_of_three_hundred_sixty_nine_before_the_next_page_opens
 实例表第二片写路径：写行那次只给第 0 片取落点（多于一片的链装不出来，一次挂载写 370 行就失败）	crates/singlefs-core/src/transaction.rs	    (0..pages)\n        .rev()\n	    (0..pages.min(1))\n        .rev()\n	-p singlefs-harness --test second_transaction_supplement_two_instance_table_page_full -- mount_after_crashes	mount_after_crashes_right_after_acquisition_fills_the_page_and_one_more_row_opens_the_second_page
 实例表第二片写路径：写行那次只给第 0 片取落点（回退要写 370 行时失败）	crates/singlefs-core/src/transaction.rs	    (0..pages)\n        .rev()\n	    (0..pages.min(1))\n        .rev()\n	-p singlefs-harness --test second_transaction_supplement_two_instance_table_page_full -- rollback_counts	rollback_counts_the_rows_of_the_table_it_rolls_back_to
-实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，带文件的一版上写行只按一片取实例表角色（多于一片时拷贝上取的与真发的不同，挂载自己的断言判出）	crates/singlefs-core/src/mount.rs	                let roles = PublishShape::row_publish_rewriting_instance_table_pages(\n                    instance_table_rewrite.pages_after_this_publish(),\n                )\n                .rewritten_roles();	                let roles = PublishShape::ROW_PUBLISH.rewritten_roles();	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- a_row_publish_past_one_page	a_row_publish_past_one_page_writes_the_tail_page_first_and_the_first_page_chains_to_it
-实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，树表 0 条那一版上写行只按一片取实例表角色（多于一片时拷贝上取的与真发的不同，挂载自己的断言判出）	crates/singlefs-core/src/mount.rs	                instance_table_page_roles_in_bump_order(\n                    instance_table_rewrite.pages_after_this_publish(),\n                )\n                .into_iter()\n                .chain(std::iter::once(TransactionUnit::AllocationTree))	                instance_table_page_roles_in_bump_order(1)\n                .into_iter()\n                .chain(std::iter::once(TransactionUnit::AllocationTree))	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- a_version_without_file_past_one_page	a_version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both
 实例表第二片 × 增补 2 收口表第 39 行那一族：发完之后比对时，树表 0 条那一版上写行真取到的落点只认第 0 片（第 1 片起的点名项被错配到别的角色上，挂载自己的断言判出）	crates/singlefs-core/src/mount.rs	            instance_table_page_roles_in_bump_order(instance_table_pages)	            instance_table_page_roles_in_bump_order(1)	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- a_version_without_file_past_one_page	a_version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both
-实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，带文件的一版上写行不释放被换下的那条实例表旧链（经映射那一路从这一轮起跳过实例表；回退到环里最旧的根时写行当场回收的那一片拷贝上看不见）	crates/singlefs-core/src/mount.rs	                let Ok(mut released) =\n                    instance_table_chain_to_release(&instance_table_rewrite.replaced_chain, &copy)\n                else {\n                    return PlacementsOnTheCopy::ReleaseCheckFailedBeforeTheFirstPlacement;\n                };	                let mut released = Vec::new();	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_the_oldest_ring_root	rolling_back_to_the_oldest_ring_root_reuses_what_the_row_publish_released_and_is_not_refused_before_acquisition
 E158 root_choice_repair session s9：search_minimum_weight_that_triggers_rootback 权重档边界判定改成中途才停（该在 weight >= effective_ceiling 时停，改成 weight > effective_ceiling——传 Some(0) 时不会在权重 0 停，会多搜一档）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	        if weight >= effective_ceiling {	        if weight > effective_ceiling {	-p singlefs-harness --bin e158_root_choice_repair	search_minimum_weight_that_triggers_rootback_honors_an_explicit_weight_ceiling
+E158 root_choice_repair session s9：mount_writable_trajectory 的故障装配条件改成两个都要满足（该在 step==0 或 persistent 时装故障，改成只在 step==0 且 persistent 时装——瞬时模式第 0 步不再装故障）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	        let active_targets: &[(DeviceIdentity, DeviceOffsetInBytes)] =\n            if step == 0 || persistent {	        let active_targets: &[(DeviceIdentity, DeviceOffsetInBytes)] =\n            if step == 0 && persistent {	-p singlefs-harness --bin e158_root_choice_repair	mount_writable_trajectory_distinguishes_persistent_from_transient_faults
+树分裂 核心规则：叶切在末尾（左半留 n − 1 条）而不是从中间切	crates/singlefs-core/src/code_two_tree.rs	            let right_keys = keys.split_off(keys.len().div_ceil(2));	            let right_keys = keys.split_off(keys.len() - 1);	-p singlefs-core --lib -- code_two_tree::tests::inserting_nine_keys_into_a_leaf_of_eight_splits_it_in_the_middle_and_grows_a_root	code_two_tree::tests::inserting_nine_keys_into_a_leaf_of_eight_splits_it_in_the_middle_and_grows_a_root
+树分裂 核心规则：插到最左分隔 key 之下时不压低它（D8 已定项 11 ④）	crates/singlefs-core/src/code_two_tree.rs	            children[0].separator_key = key.clone();\n            0	            0	-p singlefs-core --lib -- code_two_tree::tests::a_key_below_the_leftmost_separator_lowers_it_and_leaves_the_right_leaf_carried	code_two_tree::tests::a_key_below_the_leftmost_separator_lowers_it_and_leaves_the_right_leaf_carried
+树分裂 核心规则：根只剩一个孩子时不降高（D8 已定项 11 ③）	crates/singlefs-core/src/code_two_tree.rs	            PlanningNode::Internal { children, .. } if children.len() == 1 => {	            PlanningNode::Internal { children, .. } if false && children.len() == 1 => {	-p singlefs-core --lib -- code_two_tree::tests::deleting_every_key_of_a_leaf_drops_it_and_a_root_left_with_one_child_hands_the_root_to_it	code_two_tree::tests::deleting_every_key_of_a_leaf_drops_it_and_a_root_left_with_one_child_hands_the_root_to_it
+树分裂 核心规则：内部节点装不下也不切（根多出来的孩子不往上长一层）	crates/singlefs-core/src/code_two_tree.rs	            if children.len() > capacity.internal_entries {	            if false && children.len() > capacity.internal_entries {	-p singlefs-core --lib -- code_two_tree::tests::a_leaf_split_that_overflows_a_full_root_splits_the_root_too_and_the_tree_grows_to_three_levels	code_two_tree::tests::a_leaf_split_that_overflows_a_full_root_splits_the_root_too_and_the_tree_grows_to_three_levels
+树分裂 核心规则：根分裂不查层级字节写不写得下（第 256 层照长）	crates/singlefs-core/src/code_two_tree.rs	        .checked_add(1)\n        .ok_or(CodeTwoTreeRefusal::HeightBeyondTheLevelField)?;	        .checked_add(0)\n        .ok_or(CodeTwoTreeRefusal::HeightBeyondTheLevelField)?;	-p singlefs-core --lib -- code_two_tree::tests::the_level_byte_bounds_the_height	code_two_tree::tests::the_level_byte_bounds_the_height
+树分裂 内部条目宽写成 key + 87（不是 key + 子指针 86）	crates/singlefs-core/src/code_two_tree.rs	    key_width_in_bytes + usize::try_from(NODE_POINTER_BYTES).expect("86")\n}	    key_width_in_bytes + usize::try_from(NODE_POINTER_BYTES).expect("86") + 1\n}	-p singlefs-core --lib -- code_two_tree::tests::internal_entries_are_the_key_plus_an_eighty_six_byte_child_pointer	code_two_tree::tests::internal_entries_are_the_key_plus_an_eighty_six_byte_child_pointer
+树分裂 内部节点头里的 key 区间写成 [最小, 最小]（不是子树覆盖区间，D18 已定项 2）	crates/singlefs-core/src/code_two_tree.rs	            (smallest.bytes().to_vec(), largest.bytes().to_vec())	            (smallest.bytes().to_vec(), smallest.bytes().to_vec())	-p singlefs-harness --test second_transaction_supplement_two_tree_split -- under_small_node_capacities_the_first_file_version_writes_multi_level_trees_that_every_reader_accepts	under_small_node_capacities_the_first_file_version_writes_multi_level_trees_that_every_reader_accepts
+树分裂 上一版的叶一律不算照抄（没碰到的叶也整片重写）	crates/singlefs-core/src/code_two_tree.rs	            keys: keys.clone(),\n            carried_from: Some(position),	            keys: keys.clone(),\n            carried_from: None,	-p singlefs-harness --test second_transaction_supplement_two_tree_split -- an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes	an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes
+树分裂 中央映射树上一版被换下的节点不释放（只释放记账树那一族）	crates/singlefs-core/src/transaction.rs	        for tree in [\n            MultiLevelCodeTwoTree::Accounting,\n            MultiLevelCodeTwoTree::CentralMapping,\n        ] {\n            let previous_shape = &previous.multi_level_tree(tree).shape;	        for tree in [MultiLevelCodeTwoTree::Accounting] {\n            let previous_shape = &previous.multi_level_tree(tree).shape;	-p singlefs-harness --test second_transaction_supplement_two_tree_split -- an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes	an_empty_publish_rewrites_only_the_changed_mapping_path_carries_the_rest_and_releases_exactly_the_replaced_nodes
+树分裂 从盘上重建时只给记账树的根记映射 key（根之下的节点换下时经映射查不到）	crates/singlefs-core/src/recovery.rs	    for (node, pointer) in accounting_shape\n        .nodes()\n        .iter()\n        .zip(&accounting_tree.version.pointers)\n    {	    for (node, pointer) in accounting_shape\n        .nodes()\n        .iter()\n        .zip(&accounting_tree.version.pointers)\n        .rev()\n        .take(1)\n    {	-p singlefs-harness --test second_transaction_supplement_two_tree_split -- a_pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on	a_pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on
+树分裂 checker：孩子层级不是父层级减一也判成立	crates/singlefs-checker/src/walk.rs	            self.judgements.judge("I-1.1", view.level == level, || {	            self.judgements.judge("I-1.1", true, || {	-p singlefs-harness --test second_transaction_supplement_two_tree_split -- the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root	the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root
+树分裂 checker：分隔 key 大于孩子头里的最小 key 也判成立	crates/singlefs-checker/src/walk.rs	                fields(separator_key) <= fields(smallest_key),	                true,	-p singlefs-harness --test second_transaction_supplement_two_tree_split -- the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root	the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root
+树分裂 checker：分隔 key 不大于左邻孩子头里的最大 key 也判成立	crates/singlefs-checker/src/walk.rs	                    .judge("I-1.1", fields(separator_key) > fields(largest_key), || {	                    .judge("I-1.1", true, || {	-p singlefs-harness --test second_transaction_supplement_two_tree_split -- the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root	the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root
+树分裂 checker：内部节点头里的 key 区间不是子树覆盖区间也判成立	crates/singlefs-checker/src/walk.rs	            let covers = *smallest_key == view.smallest_key && *largest_key == view.largest_key;	            let covers = true;	-p singlefs-harness --test second_transaction_supplement_two_tree_split -- the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root	the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root
+树分裂 checker：多层码 2 树的条目宽不判 I-1.10	crates/singlefs-checker/src/walk.rs	                    .judge("I-1.10", entry_width == field_table_bytes, || {	                    .judge("I-1.10", true, || {	-p singlefs-harness --test second_transaction_supplement_two_tree_split -- the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root	the_pool_checker_reddens_each_broken_rule_of_a_multi_level_tree_root
+树分裂 映射树内部节点容量按 key + 87 算（143 变 142）	crates/singlefs-core/src/code_two_tree.rs	    key_width_in_bytes + usize::try_from(NODE_POINTER_BYTES).expect("86")\n}	    key_width_in_bytes + usize::try_from(NODE_POINTER_BYTES).expect("86") + 1\n}	-p singlefs-harness --test second_transaction_mapping_node_admission -- the_central_mapping_node_holds_two_hundred_ninety_four_entries_and_an_internal_node_one_hundred_forty_three	the_central_mapping_node_holds_two_hundred_ninety_four_entries_and_an_internal_node_one_hundred_forty_three
+树分裂 叶从中间切时左半留 ⌊n ÷ 2⌋ 条（295 条切成 147 + 148）	crates/singlefs-core/src/code_two_tree.rs	            let right_keys = keys.split_off(keys.len().div_ceil(2));	            let right_keys = keys.split_off(keys.len() / 2);	-p singlefs-harness --test second_transaction_mapping_node_admission -- two_hundred_ninety_five_mapping_keys_grow_the_tree_to_two_levels_instead_of_being_refused	two_hundred_ninety_five_mapping_keys_grow_the_tree_to_two_levels_instead_of_being_refused
+树分裂 取号之前的推算不删记账树节点的旧映射 key（推出来的重写数与真发布的对不上）	crates/singlefs-core/src/transaction.rs	        for key in self.accounting_node_mapping_keys.values() {\n            mapping_keys.remove(key);\n        }		-p singlefs-harness --test second_transaction_mapping_node_admission -- the_rewritten_role_counts_inferred_before_acquisition_equal_the_ones_each_empty_publish_really_rewrites	the_rewritten_role_counts_inferred_before_acquisition_equal_the_ones_each_empty_publish_really_rewrites
+树分裂 记账树叶容量按格式多算一条（478：80 块盘切成 240 + 243）	crates/singlefs-core/src/code_two_tree.rs	            leaf_entries: index_node_entry_capacity(key_width_in_bytes, leaf_entry_width_in_bytes),	            leaf_entries: index_node_entry_capacity(key_width_in_bytes, leaf_entry_width_in_bytes) + 1,	-p singlefs-harness --test second_transaction_supplement_two_accounting_node_full -- eightieth_device_splits_the_accounting_tree_into_two_leaves_under_a_root_instead_of_refusing	eightieth_device_splits_the_accounting_tree_into_two_leaves_under_a_root_instead_of_refusing
+树分裂 读树时叶里的条目不收进来（挂载态的映射条目是空的，提示过期的解引用查映射查不到）	crates/singlefs-core/src/code_two_tree.rs	        leaf_entries_in_key_order.extend(node.header.entries.iter().cloned());		-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- a_stale_hint_under_a_two_level_central_mapping_still_costs_three_device_reads_because_the_whole_tree_is_in_the_mount_state	a_stale_hint_under_a_two_level_central_mapping_still_costs_three_device_reads_because_the_whole_tree_is_in_the_mount_state
+树分裂 读树时不判内部条目宽（55 字节的映射条目当 113 字节的内部条目切）	crates/singlefs-core/src/code_two_tree.rs	        if header.entry_width < internal_width {	        if false && header.entry_width < internal_width {	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- a_central_mapping_root_claiming_level_one_over_mapping_entries_is_refused_instead_of_being_read_as_entries	a_central_mapping_root_claiming_level_one_over_mapping_entries_is_refused_instead_of_being_read_as_entries
+树分裂 读树时孩子该有的层级写成父层级（不减一）	crates/singlefs-core/src/code_two_tree.rs	                Some(header.level - 1),	                Some(header.level),	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- central_mapping_grown_into_two_levels_is_read_whole_into_the_mount_state_and_the_file_reads_back	central_mapping_grown_into_two_levels_is_read_whole_into_the_mount_state_and_the_file_reads_back
+树分裂 读树时不核内部节点的子树覆盖区间	crates/singlefs-core/src/code_two_tree.rs	            if first_child.header.smallest_key != header.smallest_key\n                || last_child.header.largest_key != header.largest_key\n            {	            if false {	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- a_two_level_central_mapping_whose_root_header_does_not_cover_its_leaf_is_refused_by_both_readers	a_two_level_central_mapping_whose_root_header_does_not_cover_its_leaf_is_refused_by_both_readers
+树分裂 层 0：冷走读核映射条目数时记账树按一个节点算（多层记账树的镜像走读失败）	crates/singlefs-core/src/recovery.rs	        + roots.accounting.version.node_count()	        + 1	-p singlefs-harness --test second_transaction_supplement_two_tree_split_layer0 -- every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes	every_tree_split_stream_recovers_cleanly_in_every_state_after_its_unit_writes
+树分裂 树高从根节点头读成层级（不加一，D8 已定项 11 ⑤ / D28 已定项 4）	crates/singlefs-core/src/transaction.rs	        u64::from(root_header.level) + 1	        u64::from(root_header.level)	-p singlefs-harness --test second_transaction_supplement_two_tree_split_layer0 -- the_accounting_streams_read_the_tree_height_from_the_root_node_header	the_accounting_streams_read_the_tree_height_from_the_root_node_header
+释放退回按提示（树分裂换锚点：释放判定路径按「这次换下的上一版角色」查）	crates/singlefs-core/src/transaction.rs	        Some(previous_version) => placements_to_release_via_mapping(\n            previous_version,\n            allocator,\n            &previous_roles_replaced,\n        )?,	        Some(previous_version) => previous_version.placements(),	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_goes_through	release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released
+步 1 变异：不释放旧落点（树分裂换锚点）	crates/singlefs-core/src/transaction.rs	        Some(previous_version) => placements_to_release_via_mapping(\n            previous_version,\n            allocator,\n            &previous_roles_replaced,\n        )?,	        Some(previous_version) => Vec::<Placement>::new(),	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
+步 1 变异：defer 行写 0（树分裂换锚点：记账行按行种类取值）	crates/singlefs-core/src/transaction.rs	                PerDeviceAccountingRow::DeferQueueBytes => device_map.deferred_slots() * SLOT_BYTES,	                PerDeviceAccountingRow::DeferQueueBytes => 0,	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
+步 2 变异：已分配行不随分配更新（I-3.1 要红；树分裂换锚点）	crates/singlefs-core/src/transaction.rs	                PerDeviceAccountingRow::AllocatedBytes => device_map.allocated_slots() * SLOT_BYTES,	                PerDeviceAccountingRow::AllocatedBytes => 0,	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
+增补 2 第 20a 行：写行那次发布的准入不在取号之前算（取号之后发布才被拒，实例代号已经烧掉；树分裂换锚点：多带节点容量一个参数）	crates/singlefs-core/src/mount.rs	    refuse_publishes_before_acquisition_that_do_not_pass_admission(\n        &allocator,\n        &start,\n        instance_to_acquire,\n        &instance_table_rewrite,\n        rows_written.len(),\n        warm_up_publishes_planned.len(),\n        pool.code_two_tree_node_capacities(),\n    )?;\n	    // 变异：写行那次发布的准入不在取号之前算\n	-p singlefs-harness --test second_transaction_supplement_two_row_publish_admission -- writable_mount_that_cannot_publish	writable_mount_that_cannot_publish_the_rows_is_refused_before_the_instance_is_acquired
+增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4b）：分配记录过 600 条之后记账「已分配」少记一槽；逼近分配记录墙那一段照跑 checker 判出（树分裂换锚点）	crates/singlefs-core/src/transaction.rs	                PerDeviceAccountingRow::AllocatedBytes => device_map.allocated_slots() * SLOT_BYTES,	                PerDeviceAccountingRow::AllocatedBytes => (device_map.allocated_slots() - u64::from(allocator.records().len() > 600)) * SLOT_BYTES,	-p singlefs-harness --test second_transaction_supplement_three_random_history -- allocation_record_wall_sampling	allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count
+增补 3 第 2 件（代码三方第二轮判决第三节第 3 条，攻方变异 m4b 同一处）：分配记录过 600 条之后记账「已分配」少记一槽；直接钉已分配统计的那条用例判出（树分裂换锚点）	crates/singlefs-core/src/transaction.rs	                PerDeviceAccountingRow::AllocatedBytes => device_map.allocated_slots() * SLOT_BYTES,	                PerDeviceAccountingRow::AllocatedBytes => (device_map.allocated_slots() - u64::from(allocator.records().len() > 600)) * SLOT_BYTES,	-p singlefs-harness --test second_transaction_step_one_overwrite -- allocated_statistic_equals	allocated_statistic_equals_the_span_sum_of_the_allocation_records_past_six_hundred_records
+增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1c；要带调试符号的构建，函数改名会悄悄失效）：只在抬 F 路径的准入里多算一个角色；抬 F 逼近墙的写死用例判出（树分裂换锚点：准入按重写角色数算）	crates/singlefs-core/src/transaction.rs	        records_before_this_publish + rewritten_role_count * allocator.devices.len();	        records_before_this_publish + (rewritten_role_count + usize::from(std::backtrace::Backtrace::force_capture().to_string().contains("raise_rollback_floor"))) * allocator.devices.len();	-p singlefs-harness --test second_transaction_supplement_three_random_history -- raising_the_floor_with_a_second_empty_publish	raising_the_floor_with_a_second_empty_publish_that_fills_the_allocation_node_to_exactly_812_records_succeeds
+增补 3 第 2 件（代码三方第二轮判决第三节第 4 条，攻方变异 m1d；要带调试符号的构建，函数改名会悄悄失效）：只在回退路径的准入里多算一个角色；回退逼近墙的写死用例判出（树分裂换锚点：准入按重写角色数算）	crates/singlefs-core/src/transaction.rs	        records_before_this_publish + rewritten_role_count * allocator.devices.len();	        records_before_this_publish + (rewritten_role_count + usize::from(std::backtrace::Backtrace::force_capture().to_string().contains("mount_rollback"))) * allocator.devices.len();	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_a_root_whose_warm_up	rolling_back_to_a_root_whose_warm_up_fills_the_allocation_node_to_exactly_812_records_succeeds
+普查 R2：checker 走读中央映射条目之前不判条目宽（walk.rs 又切 entry[27..55]，checker 自己倒下；树分裂换锚点：多层码 2 树按层守条目宽）	crates/singlefs-checker/src/walk.rs	                if view.entry_width < bytes_the_walk_needs {	                if false && view.entry_width < bytes_the_walk_needs {	-p singlefs-harness --test second_transaction_supplement_three_bad_disk_input -- every_fixed_panic_site	every_fixed_panic_site_reports_its_own_error_member_instead_of_panicking
+C511 第 3 步：checker 对中央映射树根的 I-1.3 退回写死树 ID 15，不按根记录里那条根指针的出生树判（回退之后重新发号的映射树判不了；树分裂换锚点）	crates/singlefs-checker/src/walk.rs	            tree: parse_node_pointer(mapping_root_pointer).birth_tree,	            tree: 15,	-p singlefs-harness --test checker_known_bad_images -- a_central_mapping_root_whose_header_tree_differs	a_central_mapping_root_whose_header_tree_differs_from_its_root_pointer_birth_tree_reddens_only_the_tree_identifier_invariant
+增补 2 第 9 行 P1（C497）：带文件的一版上重写实例表的发布把实例表挪到树表单元之后取落点（树分裂换锚点：角色清单在两棵多层树的节点之后才收尾）	crates/singlefs-core/src/transaction.rs	        rewritten_roles.push(TransactionUnit::TreeTable);\n        Ok(ResolvedPublish {	        rewritten_roles.push(TransactionUnit::TreeTable);\n        if let Some(position) = rewritten_roles.iter().position(|role| *role == TransactionUnit::InstanceTable) {\n            let moved = rewritten_roles.remove(position);\n            rewritten_roles.push(moved);\n        }\n        Ok(ResolvedPublish {	-p singlefs-harness --test second_transaction_supplement_two_presumed_clause_checks -- c497_every_publish	c497_every_publish_that_rewrites_the_instance_table_bumps_it_before_every_other_commit_generated_block
+C483 ②：冷走读读 inode 叶容器只按位置提示读（不经映射回退；树分裂换锚点：映射条目取整棵映射树的叶）	crates/singlefs-core/src/recovery.rs	        let leaf_bytes = read_mapped_tree_node_via_hint_then_central_mapping(\n            reader,\n            &child,\n            MappedTreeNodeClass::PackedRecordUnit,\n            &|mapping_key: &[u8]| {\n                central_mapping_locations_among_entries(\n                    &roots.mapping.leaf_entries_in_key_order,\n                    mapping_key,\n                )\n            },\n            mapping_fallbacks,\n        )?;	        let leaf_bytes = read_unit_via_locations(reader, &child.locations, data_unit_bytes)?;	-p singlefs-harness --test second_transaction_supplement_two_tree_nodes_and_the_central_mapping -- an_inode_leaf_container_whose	an_inode_leaf_container_whose_location_hint_points_at_an_empty_slot_reads_back_through_the_central_mapping
+并行线一：映射条目不算数据单元（多单元文件的映射 key 与 resolve 预先算的对不上；树分裂换锚点：映射条目数由这一版每个进映射单元的 key 定）	crates/singlefs-core/src/transaction.rs	                for transaction in self.file_content_transactions() {	                for transaction in self.file_content_transactions().into_iter().take(1) {	-p singlefs-harness --test second_transaction_parallel_line_one_sequential_write -- a_publish_of_two_data_units_names_the_shared_commit_generated_units_only_in_its_last_record	a_publish_of_two_data_units_names_the_shared_commit_generated_units_only_in_its_last_record
+代码三方 m2-wave3-code-r1 第四节第 4 条：只读挂载认中央映射树的根退回写死树 ID 15，不按根记录里映射根指针的出生树（回退之后再发的第一个文件版本映射树是 23，打不开；树分裂换锚点）	crates/singlefs-core/src/mounted_read.rs	        &MultiLevelCodeTwoTree::CentralMapping.read_expectation(root.mapping_root.head.birth_tree),	        &MultiLevelCodeTwoTree::CentralMapping.read_expectation(TreeIdentifier(15)),	-p singlefs-harness --test second_transaction_step_four_rollback -- the_read_only_mount_after_rolling_back	the_read_only_mount_after_rolling_back_to_a_warm_up_root_finds_the_central_mapping_under_the_tree_its_root_pointer_names
+树分裂 规划删不掉上一版的 key 也照常往下走（从盘上重建的映射树分隔 key 坏了，取号之前不拒）	crates/singlefs-core/src/code_two_tree.rs	        return Err(CodeTwoTreeRefusal::PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt);	        return Ok(());	-p singlefs-harness --test second_transaction_supplement_two_tree_split -- a_rebuilt_central_mapping_root_whose_separator_hides_a_key_is_refused_before_the_instance_generation_is_acquired	a_rebuilt_central_mapping_root_whose_separator_hides_a_key_is_refused_before_the_instance_generation_is_acquired
+增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，树表 0 条那一版上写行只取实例表、漏了分配记录树节点（拷贝上取的与真发的不同，挂载自己的断言判出；树分裂换锚点：每一次的角色表按次序排好）	crates/singlefs-core/src/mount.rs	                            .into_iter()\n                            .chain(std::iter::once(TransactionUnit::AllocationTree))\n                            .collect()\n                        } else {	                            .into_iter()\n                            .collect()\n                        } else {	-p singlefs-harness --test second_transaction_step_three_formatted_pool -- a_formatted_pool_mounted_twice	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
+树分裂 取号之前在拷贝上取落点时按固定的四个固定点角色推（不按两棵多层树的形状，多层池挂载时拷贝上取的与真发的对不上，挂载自己的断言判出）	crates/singlefs-core/src/mount.rs	                        rewritten_roles_of_a_publish_without_content(*shape, &nodes.rewritten_roles)	                        { let _ = nodes; shape.rewritten_roles() }	-p singlefs-harness --test second_transaction_supplement_two_tree_split -- a_pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on	a_pool_with_multi_level_trees_mounts_writable_and_the_row_and_warm_up_publishes_carry_on
+树分裂 格式常量字面量改掉一个数：ACCOUNTING_INTERNAL_ENTRY_BYTES（108 写成 109，不等于记账 key 22 + 子指针 86）	crates/singlefs-format/src/lib.rs	pub const ACCOUNTING_INTERNAL_ENTRY_BYTES: u64 = 108;	pub const ACCOUNTING_INTERNAL_ENTRY_BYTES: u64 = 109;	-p singlefs-format --lib -- pointer_record_and_entry_width_literals	pointer_record_and_entry_width_literals_equal_the_field_sums_they_stand_for
+树分裂 格式常量字面量改掉一个数：CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES（113 写成 114，不等于映射 key 27 + 子指针 86）	crates/singlefs-format/src/lib.rs	pub const CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES: u64 = 113;	pub const CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES: u64 = 114;	-p singlefs-format --lib -- pointer_record_and_entry_width_literals	pointer_record_and_entry_width_literals_equal_the_field_sums_they_stand_for
+实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，带文件的一版上写行只按一片取实例表角色（多于一片时拷贝上取的与真发的不同，挂载自己的断言判出）（树分裂换锚点：拷贝上那一串的形状表第 0 次）	crates/singlefs-core/src/mount.rs	                    std::iter::once(PublishShape::row_publish_rewriting_instance_table_pages(\n                        instance_table_rewrite.pages_after_this_publish(),\n                    ))\n                    .chain(std::iter::repeat_n(\n                        PublishShape::EMPTY_PUBLISH,\n                        warm_up_publish_txgs.len(),	                    std::iter::once(PublishShape::ROW_PUBLISH)\n                    .chain(std::iter::repeat_n(\n                        PublishShape::EMPTY_PUBLISH,\n                        warm_up_publish_txgs.len(),	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- a_row_publish_past_one_page	a_row_publish_past_one_page_writes_the_tail_page_first_and_the_first_page_chains_to_it
+实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，树表 0 条那一版上写行只按一片取实例表角色（多于一片时拷贝上取的与真发的不同，挂载自己的断言判出）（树分裂换锚点：每一次的角色表按次序排好）	crates/singlefs-core/src/mount.rs	                            instance_table_page_roles_in_bump_order(\n                                instance_table_rewrite.pages_after_this_publish(),\n                            )\n                            .into_iter()	                            instance_table_page_roles_in_bump_order(1)\n                            .into_iter()	-p singlefs-harness --test second_transaction_supplement_two_instance_table_second_page_write -- a_version_without_file_past_one_page	a_version_without_file_past_one_page_writes_two_pages_and_the_next_mount_releases_both
+实例表第二片 × 增补 2 收口表第 39 行那一族：取号之前在拷贝上走那一串时，带文件的一版上写行不释放被换下的那条实例表旧链（经映射那一路从这一轮起跳过实例表；回退到环里最旧的根时写行当场回收的那一片拷贝上看不见）（树分裂换锚点：第 0 次释放那一段按两棵多层树的形状重排过）	crates/singlefs-core/src/mount.rs	                    let checked = instance_table_chain_to_release(\n                        &instance_table_rewrite.replaced_chain,\n                        &copy,\n                    )\n                    .and_then(|mut released| {\n	                    let checked = Ok::<Vec<Placement>, PublishError>(Vec::new())\n                    .and_then(|mut released| {\n	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_the_oldest_ring_root	rolling_back_to_the_oldest_ring_root_reuses_what_the_row_publish_released_and_is_not_refused_before_acquisition
+增补 2 收口表第 58 行（真设备抬 F 模式）：模式名 raise-rollback-floor 写成下划线（跑批脚本送来的参数认不回）	crates/singlefs-harness/src/on_device_modes.rs	            OnDeviceRunMode::RaiseRollbackFloor => "raise-rollback-floor",	            OnDeviceRunMode::RaiseRollbackFloor => "raise_rollback_floor",	-p singlefs-harness --lib -- on_device_modes	every_mode_argument_reads_back_as_the_same_mode_and_unknown_text_is_refused
+增补 2 收口表第 58 行（真设备抬 F 模式）：真设备二进制抬 F 窗口按种类那一侧不算那一串空发布	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	        raise_publishes.push(&raise_publish.writes);	        raise_publishes.clear();	-p singlefs-harness --bin first_transaction_on_device -- raise_rollback_floor_mode	raise_rollback_floor_mode_publishes_the_fourth_version_raises_the_floor_to_its_ceiling_above_zero_and_every_window_matches_the_device_layer_count
+增补 2 收口表第 58 行（真设备抬 F 模式）：真设备二进制抬 F 失败时不用抬 F 交回的已落盘账	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	                        writes_of_persisted_publishes,\n                        writes_of_failed_publishes,	                        &[],\n                        writes_of_failed_publishes,	-p singlefs-harness --bin first_transaction_on_device -- failed_raise	failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count
+增补 2 收口表第 58 行（真设备抬 F 模式）：抬 F 那一段没登记进 raise-rollback-floor 的分段表	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	            "raise_rollback_floor",\n            "cold_reopen_and_recover",	            "cold_reopen_and_recover",	-p singlefs-harness --bin first_transaction_on_device -- every_mode_registers	every_mode_registers_its_own_segments_with_no_repeats
+增补 2 收口表第 58 行（真设备抬 F 模式）：宿主检查把抬 F 那一段的终点记成发布 C（抬 F 那一段不单独比、红了也指不到它）	crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs	            window_ends.push((ProgramWindow::RaiseRollbackFloor, stream.operation_count()));	            window_ends.push((ProgramWindow::ThirdTransaction, stream.operation_count()));	-p singlefs-harness --bin first_transaction_device_log_check	raise_rollback_floor_mode_compares_the_raise_window_and_is_red_there_when_one_step_changed
+E158 root_choice_repair session s9（实十九提醒）：allocation_record_tree_reachable_placements_via_central_mapping 的多层根拦截条件取反（该在 level != 0 时拒绝，改成在 level == 0 时拒绝——level=0 的正常叶反而被拦，level=1 的多层根反而放行）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	    if mapping_root.level != 0 {	    if mapping_root.level == 0 {	-p singlefs-harness --bin e158_root_choice_repair	allocation_record_tree_reachable_placements_via_central_mapping_rejects_a_multi_level_root
+增补 2 收口表第 58 行（真设备抬 F 模式）：raise-rollback-floor 发布 C 之后不再覆盖写一次（第 4 新的非空有效根够不到 A，F 的上限退回 0）	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	    let fourth_version_run = publish_the_fourth_version_and_describe(\n        parameters,\n        third_version_run,\n        stream,\n        geometry,\n        clock,\n    )?;	    let fourth_version_run = third_version_run;	-p singlefs-harness --bin first_transaction_on_device -- raise_rollback_floor_mode	raise_rollback_floor_mode_publishes_the_fourth_version_raises_the_floor_to_its_ceiling_above_zero_and_every_window_matches_the_device_layer_count
+增补 2 收口表第 58 行（真设备抬 F 模式）：抬 F 抬到现行的 F、不抬到 core 现算的上限	crates/singlefs-harness/src/on_device_modes.rs	    raise_rollback_floor(\n        parameters,\n        devices,\n        allocator,\n        current_version,\n        ceiling,	    let ceiling = current_version.root.rollback_floor;\n    raise_rollback_floor(\n        parameters,\n        devices,\n        allocator,\n        current_version,\n        ceiling,	-p singlefs-harness --bin first_transaction_on_device -- raise_rollback_floor_mode	raise_rollback_floor_mode_publishes_the_fourth_version_raises_the_floor_to_its_ceiling_above_zero_and_every_window_matches_the_device_layer_count
+增补 2 收口表第 58 行（真设备抬 F 模式）：真设备二进制发布 D 失败时丢掉失败账那几行	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	            lines.extend(failed.lines);\n            return Err(FailedRun {\n                lines,\n                cause: format!("发布 D：{}", failed.cause),	            return Err(FailedRun {\n                lines,\n                cause: format!("发布 D：{}", failed.cause),	-p singlefs-harness --bin first_transaction_on_device -- fourth_version_publish_failing	fourth_version_publish_failing_midway_reports_the_writes_it_landed_and_they_equal_the_device_layer_count
+增补 2 收口表第 58 行（真设备抬 F 模式）：raise-rollback-floor 冷重开该读回的取成第三版（没算发布 D；接替原「冷重开该读回的取成第二版」那一行）	crates/singlefs-harness/src/on_device_modes.rs	            PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstanceThenFourthVersionThenRaiseOfTheRollbackFloor => {\n                fourth_file_content()\n            }	            PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstanceThenFourthVersionThenRaiseOfTheRollbackFloor => {\n                third_file_content()\n            }	-p singlefs-harness --lib -- on_device_modes	the_four_versions_differ_in_length_and_bytes_and_each_mode_reads_back_its_last_one
+增补 2 收口表第 58 行（真设备抬 F 模式）：宿主检查不重跑发布 D（抬 F 接在发布 C 上，发布 D 那一段空着）	crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs	            let mut current_version = publish_the_fourth_version(\n                parameters,\n                &mut second_instance.devices,\n                &mut second_instance.allocator,\n                &second_instance.current_version,\n            )\n            .map_err(|failed| format!("发布 D：{:?}", failed.cause))?;	            let mut current_version = second_instance.current_version.clone();	-p singlefs-harness --bin first_transaction_device_log_check	every_mode_reruns_its_own_windows_and_matches_a_log_that_received_exactly_the_program_events
+增补 2 收口表第 58 行（真设备抬 F 模式）：宿主检查把发布 D 那一段的终点记成发布 C（发布 D 那一段不单独比、红了也指不到它）	crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs	            window_ends.push((ProgramWindow::FourthTransaction, stream.operation_count()));	            window_ends.push((ProgramWindow::ThirdTransaction, stream.operation_count()));	-p singlefs-harness --bin first_transaction_device_log_check	raise_rollback_floor_mode_compares_the_raise_window_and_is_red_there_when_one_step_changed
+实二二三 a（D23 已定项 14「这一版的失败处置」）：冻结着一次没重发的发布时发布路径不拒（另建下一次发布，同一 (实例, txg) 留下两条末条）	crates/singlefs-core/src/transaction.rs	    match allocator.frozen_publish() {\n        None => Ok(()),\n	    match None::<&FrozenPublish> {\n        None => Ok(()),\n	-p singlefs-harness --test second_transaction_supplement_two_publish_failure_resent_unchanged -- a_publish_that_fails_midway	a_publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish
+实二二三 a：原样重发成功之后分配器不换成那次发布成立之后的一份（下一次发布的落点落在重发的那次发布的单元上）	crates/singlefs-core/src/transaction.rs	    *allocator = allocator_after_the_publish;\n    Ok(Some(\n	    let _ = allocator_after_the_publish;\n    Ok(Some(\n	-p singlefs-harness --test second_transaction_supplement_two_publish_failure_resent_unchanged -- a_publish_that_fails_midway	a_publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish
+实二二三 b（C539，D23 已定项 4 读者规则）：锚点读得出时下一次发布的首条序号不是 1 照接	crates/singlefs-core/src/recovery.rs	        if records_of_the_open_publish.is_empty()\n            && record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST\n        {\n	        if false\n            && records_of_the_open_publish.is_empty()\n            && record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST\n        {\n	-p singlefs-harness --test second_transaction_parallel_line_one_last_record_flag -- with_a_readable_anchor_a_next_publish	with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens
+实二二三 b（C540，D23 已定项 4 读者规则）：带末条标志的那一条之后同一 (实例, txg) 还有记录照字面按标志认边界	crates/singlefs-core/src/recovery.rs	        if record_ends_its_publish(record)\n            && a_readable_record_of_the_same_publish_follows(record, records)\n        {\n	        if false\n            && record_ends_its_publish(record)\n            && a_readable_record_of_the_same_publish_follows(record, records)\n        {\n	-p singlefs-harness --test second_transaction_parallel_line_one_last_record_flag -- a_last_record_flag_followed_by	a_last_record_flag_followed_by_a_record_of_the_same_publish_breaks_the_chain_at_the_flag_and_the_checker_reddens
+实二二三 d（D19 已定项 5，用户 2026-09-25 定）：映射条目两条位置项指同一块盘时不拒（另一块盘那一份没核就释放）	crates/singlefs-core/src/transaction.rs	        if locations[0].device == locations[1].device {\n	        if false && locations[0].device == locations[1].device {\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- two_mapping_locations_on_the_same_device	two_mapping_locations_on_the_same_device_are_refused_as_a_damaged_mapping_entry_before_anything_is_written
+实二二三 e（I-3.1 / I-3.11 的隔离读法，用户 2026-09-25 定）：已分配而没有根引用的记录一律不豁免	crates/singlefs-checker/src/walk.rs	        if some_copy_fails {\n            for device in devices_of_the_unit {\n	        if false && some_copy_fails {\n            for device in devices_of_the_unit {\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_quarantined_record_is_exempted	a_quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read
+实二二三 e：已分配而没有根引用的记录不读那一份、一律豁免（读得出且对得上的也豁免）	crates/singlefs-checker/src/walk.rs	                .is_some_and(|unit| check_unit(&unit).is_ok())\n        };\n	                .is_some_and(|_unit| false)\n        };\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_quarantined_record_is_exempted	a_quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read
+实二二三 e（按单元判，主 agent 2026-09-25 定）：只豁免读不出或对不上的那一份的记录（对得上那块盘上的那一条照红）	crates/singlefs-checker/src/walk.rs	        if some_copy_fails {\n            for device in devices_of_the_unit {\n                *exempted.entry(device).or_insert(0) += span_slots;\n            }\n        }\n	        for device in devices_of_the_unit {\n            if !copy_self_verifies(device) {\n                *exempted.entry(device).or_insert(0) += span_slots;\n            }\n        }\n        let _ = some_copy_fails;\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_unit_with_one_failing_copy	a_unit_with_one_failing_copy_exempts_its_records_on_every_device_and_two_intact_copies_do_not
+实二二三 f（D23 已定项 17，实十六接续 Q7）：树表 0 条的一版上写行不按写入口装的「一条记录装几项」切（点名项装不下一条也只写一条）	crates/singlefs-core/src/transaction.rs	    let named_entry_capacity = pool.journal_record_named_entry_capacity();\n    let roles_named_by_each_record =\n        roles_named_by_each_record_of_the_publish(&named_roles, named_entry_capacity);\n	    let named_entry_capacity = JournalRecordNamedEntryCapacity::FromTheRecordFormat;\n    let roles_named_by_each_record =\n        roles_named_by_each_record_of_the_publish(&named_roles, named_entry_capacity);\n	-p singlefs-harness --test second_transaction_supplement_two_multi_record_transaction_zero_publishes -- a_row_publish_whose_named_units	a_row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_and_the_checker_accept_it
+实二二三 f / i：树表 0 条的一版上写行跨多条记录时每条都带提交标记（事务号 0 的事务被切开）	crates/singlefs-core/src/transaction.rs	            is_commit: transaction_of_the_next_record != Some(transaction_offset),\n            ordinal_within_publish: JournalRecordOrdinalWithinPublish::of_record_at_offset(\n                record_offset_in_this_publish,\n            ),\n            place_in_publish: if	            is_commit: true,\n            ordinal_within_publish: JournalRecordOrdinalWithinPublish::of_record_at_offset(\n                record_offset_in_this_publish,\n            ),\n            place_in_publish: if	-p singlefs-harness --test second_transaction_supplement_two_multi_record_transaction_zero_publishes -- a_row_publish_whose_named_units	a_row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_and_the_checker_accept_it
+实二二三 i（主 agent 2026-09-24 定）：读者把提交标记 2..=255 读成「不带」（与 checker 的 I-8.8 说的不是一件事）	crates/singlefs-core/src/journal.rs	            2..=u8::MAX => return None,\n	            2..=u8::MAX => false,\n	-p singlefs-harness --test second_transaction_supplement_two_multi_record_transaction_zero_publishes -- a_commit_marker_other_than	a_commit_marker_other_than_zero_or_one_counts_as_torn_and_breaks_the_chain_and_the_checker_reddens
+实二二三 g（C332，D23 已定项 14「回退见证」）：择根不跳过被见证表抛弃的根（回退实例的根都读不出时落到被抛弃的 C）	crates/singlefs-core/src/recovery.rs	            if rollback_witness.abandons(candidate.instance, candidate.checkpoint_txg) {\n                return;\n            }\n	            if false && rollback_witness.abandons(candidate.instance, candidate.checkpoint_txg) {\n                return;\n            }\n	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- with_the_rollback_instances_roots_unreadable	with_the_rollback_instances_roots_unreadable_recovery_does_not_return_to_the_abandoned_timeline
+实二二三 g（C332）：重放不在被见证表抛弃的记录上停（落到 R_old 之后把被抛弃的 B、C 的记录施加回来）	crates/singlefs-core/src/recovery.rs	        if rollback_witness.abandons(record.instance, record.checkpoint_txg) {\n            break;\n        }\n	        if false && rollback_witness.abandons(record.instance, record.checkpoint_txg) {\n            break;\n        }\n	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- with_the_rollback_instances_roots_unreadable	with_the_rollback_instances_roots_unreadable_recovery_does_not_return_to_the_abandoned_timeline
+实二二三 g：见证表不落进系统配置槽（to_slot 不写那 753 字节）	crates/singlefs-core/src/system_configuration.rs	        bytes[witness_start..witness_end].copy_from_slice(&self.rollback_witness.to_bytes());\n	        let _ = (witness_start, witness_end);\n	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- with_the_rollback_instances_roots_unreadable	with_the_rollback_instances_roots_unreadable_recovery_does_not_return_to_the_abandoned_timeline
+实二二三 g（写序 post）：回退那一次挂载从写行那次的轮换起不带这一次回退那一条（一直写删过的旧表）	crates/singlefs-core/src/mount.rs	    pool.write_rollback_witness_from_now_on(rollback_witness_from_the_row_publish);\n	    let _ = &rollback_witness_from_the_row_publish;\n	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- the_witness_rides_from_the_first_rotation	the_witness_rides_from_the_first_rotation_after_the_row_publish_and_every_later_system_configuration_write
+实二二三 g（删除规则）：根环有槽读不出时照删条目	crates/singlefs-core/src/mount.rs	            None => true,\n            Some(roots) => roots.iter().any(|root| {\n	            None => false,\n            Some(roots) => roots.iter().any(|root| {\n	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- an_entry_is_dropped_only_when	an_entry_is_dropped_only_when_every_ring_slot_is_readable_and_no_root_lies_between_the_target_and_the_new_instance
+实二二三 g（删除规则）：环里没有落在 [r_old, N) 的根时条目也不删	crates/singlefs-core/src/mount.rs	                entry.rollback_target_instance <= root.instance\n                    && root.instance < entry.new_instance\n	                entry.rollback_target_instance <= root.instance\n                    || root.instance < entry.new_instance\n	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- an_entry_is_dropped_only_when	an_entry_is_dropped_only_when_every_ring_slot_is_readable_and_no_root_lies_between_the_target_and_the_new_instance
+实二二三 g：回退目标被见证表抛弃时照当候选（回退回被抛弃的时间线）	crates/singlefs-core/src/mount.rs	        || rollback_witness_of_the_pool(&*devices, &system_configuration)\n            .abandons(target.instance, target.checkpoint_txg)\n	        || false\n	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- a_rollback_target_abandoned_by_the_witness	a_rollback_target_abandoned_by_the_witness_is_not_a_candidate
+实二二三 g（条数上限 R × S − 1）：见证表上限按定宽的 47 算、不按这个池的 S（表满不拒，写出这个池自己读不回的表）	crates/singlefs-core/src/mount.rs	    let capacity = rollback_witness_capacity(\n        system_configuration\n            .immutable\n            .sizes\n            .root_ring_slots_per_region,\n    );\n	    let capacity = 47;\n    let _ = rollback_witness_capacity;\n	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- a_rollback_whose_witness_table	a_rollback_whose_witness_table_would_exceed_its_capacity_is_refused_before_any_write
+实二二三 g（checker I-7.10）：同一个新实例两种回退目标不判红	crates/singlefs-checker/src/walk.rs	                                (recorded_instance, recorded_txg) == target,\n	                                (recorded_instance, recorded_txg) == target || true,\n	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- contradicting_or_unparseable	contradicting_or_unparseable_witness_tables_redden_the_witness_consistency_invariant
+实二二三 g（checker I-7.10）：校验和过而见证表解不开的槽不判红	crates/singlefs-checker/src/walk.rs	            Err(what) => judgements.judge("I-7.10", false, || {\n	            Err(what) => judgements.judge("I-7.10", true, || {\n	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- contradicting_or_unparseable	contradicting_or_unparseable_witness_tables_redden_the_witness_consistency_invariant
+实二二三 g（checker I-7.11）：所选根的实例表罩不住见证条目不判红	crates/singlefs-checker/src/walk.rs	            target_row_covers && missing_intermediate.is_none(),\n	            target_row_covers || missing_intermediate.is_none() || true,\n	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- a_witness_entry_the_chosen_roots	a_witness_entry_the_chosen_roots_instance_table_does_not_cover_reddens_the_witness_table_invariant
+实二二三 g（checker I-7.11）：回退目标那一行的 T 要严格小于目标 txg 才算罩得住（回退行 (r_old, T_old) 自己罩不住，健康镜像上红）	crates/singlefs-checker/src/walk.rs	                && row.published_checkpoint_txg <= entry.rollback_target_txg\n	                && row.published_checkpoint_txg < entry.rollback_target_txg\n	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- the_pool_checker_holds_both_witness	the_pool_checker_holds_both_witness_invariants_after_a_rollback
+实二二三 g（回退见证的判法，D23 已定项 14）：(r_old, T_old) ≤ (i, T) 也算抛弃（R_old 自己被抛弃）	crates/singlefs-core/src/rollback_witness.rs	        (self.rollback_target_instance, self.rollback_target_txg) < (instance, checkpoint_txg)\n	        (self.rollback_target_instance, self.rollback_target_txg) <= (instance, checkpoint_txg)\n	-p singlefs-core --lib -- rollback_witness	an_entry_abandons_exactly_the_roots_after_the_rollback_target_and_before_the_new_instance
+实二二三 g（checker I-7.11）：回退到 mkfs 的第 0 代根时也要实例 0 那一行（实例 0 不写行，健康镜像上红）	crates/singlefs-checker/src/walk.rs	        let target_row_covers = entry.rollback_target_instance == 0\n            || instance_table_rows	        let target_row_covers = false\n            || instance_table_rows	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- a_rollback_to_the_make_filesystem_root	a_rollback_to_the_make_filesystem_root_holds_the_witness_table_invariant_without_a_row_for_instance_zero
+实二二三 h（D18 已定项 11 第五个合取）：取号之前的预演不查写行那次经映射换下的映射条目的位置项（坏映射条目在取号之后才报，号烧掉）	crates/singlefs-core/src/mount.rs	                        refuse_mapping_entries_that_do_not_name_two_pool_devices(\n                            output,\n                            &released_roles_of_the_row_publish,\n                            &pool_devices,\n                        )?;\n	                        let _ = &pool_devices;\n	-p singlefs-harness --test second_transaction_supplement_two_row_publish_checks_before_acquisition -- a_row_publish_release_check	a_row_publish_release_check_that_fails_on_a_damaged_mapping_entry_refuses_the_mount_before_acquisition
+实二二三 h（Z4-1）：树表 0 条的一版上写行的释放核与条数准入不在取号之前判（装不下时取号之后才拒，每试一次烧一个号）	crates/singlefs-core/src/mount.rs	    if let (PreviousVersion::WithoutFile { root, .. }, true) =\n        (&start.previous, row_publish_rewrites_the_instance_table)\n	    if let (PreviousVersion::WithoutFile { root, .. }, true) =\n        (&start.previous, false && row_publish_rewrites_the_instance_table)\n	-p singlefs-harness --test second_transaction_supplement_two_row_publish_checks_before_acquisition -- a_row_publish_on_a_version_without_file	a_row_publish_on_a_version_without_file_that_outgrows_the_allocation_node_is_refused_before_acquisition
diff -ruN -x target tree/crates/singlefs-checker/src/image.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-checker/src/image.rs	2026-09-24 14:49:12.386896006 +0000
+++ tree/crates/singlefs-checker/src/image.rs	2026-09-24 18:48:58.241379617 +0000
@@ -34,12 +34,12 @@
 }
 
 /// 第一版 checker 判的不变量，按这个次序报；每次都全部报出来，没评估到的报「不适用」。
-pub const IMPLEMENTED_INVARIANTS: [&str; 44] = [
+pub const IMPLEMENTED_INVARIANTS: [&str; 46] = [
     "I-1.1", "I-1.2", "I-1.3", "I-1.4", "I-1.6", "I-1.7", "I-1.8", "I-1.10", "I-2.1", "I-2.3",
     "I-2.4", "I-2.5", "I-3.1", "I-3.8", "I-3.9", "I-3.10", "I-3.11", "I-4.2", "I-4.8", "I-5.1",
     "I-5.2", "I-5.4", "I-7.1", "I-7.2", "I-7.3", "I-7.4", "I-7.6", "I-7.7", "I-7.8", "I-7.9",
-    "I-8.6", "I-8.7", "I-8.8", "I-8.9", "I-9.1", "I-9.2", "I-9.4", "I-9.6", "I-9.7", "I-9.10",
-    "I-9.12", "I-9.13", "I-9.14", "I-9.15",
+    "I-7.10", "I-7.11", "I-8.6", "I-8.7", "I-8.8", "I-8.9", "I-9.1", "I-9.2", "I-9.4", "I-9.6",
+    "I-9.7", "I-9.10", "I-9.12", "I-9.13", "I-9.14", "I-9.15",
 ];
 
 /// 判定累加器：每条不变量记评估了几次、第一处违例、以及整条不适用的理由。
@@ -206,6 +206,77 @@
         .collect()
 }
 
+/// 一个自证过的系统配置槽里的回退见证表（D23（journal 的角色与格式） 已定项 14「回退见证」）：哪块盘、槽世代号、解出来的条目或解不开的那一样。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct RollbackWitnessOfASlot {
+    pub device: u32,
+    pub slot_generation: u64,
+    pub witness: Result<Vec<crate::RollbackWitnessEntryView>, &'static str>,
+}
+
+/// 每盘两槽里自证过的系统配置槽各自的回退见证表（读法与 `verified_system_configuration_slots` 相同：槽 1 按槽 0 记的槽距找）。
+/// 条数上限按那一槽自述的 R、S 算（R × S − 1）。
+#[must_use]
+pub fn rollback_witness_of_every_verified_slot(
+    reader: &dyn ImageReader,
+) -> Vec<RollbackWitnessOfASlot> {
+    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
+    let mut witnesses = Vec::new();
+    for device in reader.devices() {
+        let slot_zero = reader.read(device, 0, slot_bytes);
+        let spacing = slot_zero
+            .as_deref()
+            .and_then(parse_system_configuration_slot)
+            .map_or(
+                FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
+                |(_, geometry)| geometry.slot_spacing,
+            );
+        let slot_one = reader.read(device, spacing, slot_bytes);
+        for bytes in [slot_zero, slot_one].into_iter().flatten() {
+            let Some((view, geometry)) = parse_system_configuration_slot(&bytes) else {
+                continue;
+            };
+            witnesses.push(RollbackWitnessOfASlot {
+                device,
+                slot_generation: view.slot_generation,
+                witness: crate::rollback_witness_of_system_configuration_slot(
+                    &bytes,
+                    (geometry.regions * geometry.slots_per_region).saturating_sub(1),
+                ),
+            });
+        }
+    }
+    witnesses
+}
+
+/// 一个池此刻的回退见证：每块盘上见证表解得开的槽里世代号最大的那一槽的条目，各盘取并集（与实现同一个读法：解不开见证表的槽
+/// 在实现那边就是读不出的槽，不参与择槽）。
+#[must_use]
+pub fn rollback_witness_of_the_pool(
+    slots: &[RollbackWitnessOfASlot],
+) -> Vec<crate::RollbackWitnessEntryView> {
+    let mut chosen_per_device: BTreeMap<u32, (u64, &Vec<crate::RollbackWitnessEntryView>)> =
+        BTreeMap::new();
+    for slot in slots {
+        let Ok(entries) = &slot.witness else {
+            continue;
+        };
+        let is_newer = chosen_per_device
+            .get(&slot.device)
+            .is_none_or(|(generation, _)| slot.slot_generation > *generation);
+        if is_newer {
+            chosen_per_device.insert(slot.device, (slot.slot_generation, entries));
+        }
+    }
+    let mut union: Vec<crate::RollbackWitnessEntryView> = chosen_per_device
+        .values()
+        .flat_map(|(_, entries)| entries.iter().copied())
+        .collect();
+    union.sort_unstable();
+    union.dedup();
+    union
+}
+
 /// 每盘择一个系统配置：两槽里世代号大的那一份（相等取槽 0）。
 #[must_use]
 pub fn chosen_system_configurations(
diff -ruN -x target tree/crates/singlefs-checker/src/lib.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-checker/src/lib.rs	2026-09-24 14:49:12.386896006 +0000
+++ tree/crates/singlefs-checker/src/lib.rs	2026-09-24 18:48:58.241397937 +0000
@@ -12,7 +12,9 @@
 use singlefs_format::{
     index_node_header_bytes, DATA_UNIT_BYTES, DATA_UNIT_HEADER_BYTES, JOURNAL_HEADER_BYTES,
     JOURNAL_NAMED_ENTRY_BYTES, JOURNAL_RECORD_BYTES, NODE_BYTES,
-    NONCE_MAC_ALGORITHM_RESERVED_BYTES, PACKED_UNIT_HEADER_BYTES, ROOT_RECORD_BYTES,
+    NONCE_MAC_ALGORITHM_RESERVED_BYTES, PACKED_UNIT_HEADER_BYTES, ROLLBACK_WITNESS_COUNT_BYTES,
+    ROLLBACK_WITNESS_ENTRIES_MAXIMUM, ROLLBACK_WITNESS_ENTRY_BYTES,
+    ROLLBACK_WITNESS_TABLE_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT, ROOT_RECORD_BYTES,
     SYSTEM_CONFIGURATION_SLOT_BYTES, WIDE_CHECKSUM_BYTES,
 };
 
@@ -126,6 +128,69 @@
     RootRingSlotsPerRegionOutsideTheFormatInterval,
 }
 
+/// 回退见证表的一个条目（D23（journal 的角色与格式） 已定项 14「回退见证」）：新实例代号 4 + 回退目标 R_old 的实例代号 4 + txg 8，小端。
+/// 偏移与宽度只从 `singlefs-format` 取，解析在 checker 这边另写一份。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub struct RollbackWitnessEntryView {
+    pub new_instance: u32,
+    pub rollback_target_instance: u32,
+    pub rollback_target_txg: u64,
+}
+
+impl RollbackWitnessEntryView {
+    /// (实例代号, txg) 这一处被这次回退抛弃：(r_old, T_old) < (i, T)（实例代号为主比）且 i < N。
+    #[must_use]
+    pub fn abandons(&self, instance: u32, checkpoint_txg: u64) -> bool {
+        (self.rollback_target_instance, self.rollback_target_txg) < (instance, checkpoint_txg)
+            && instance < self.new_instance
+    }
+}
+
+/// 一个自证过的系统配置槽里的回退见证表：槽内偏移 481 起，条数 1 字节 + 47 个 16 字节的条目位。`capacity` 是这个池的条数上限
+/// （R × S − 1，R、S 读自同一槽）。条数不超过上限、条目按 (N, r_old, T_old) 严格升序、每一条 r_old < N、条数之后的条目位全 0，
+/// 四样都满足才交回条目；不满足交回哪一样不满足（I-7.10（回退见证表各槽自洽、各盘一致） 的违例说明要它）。
+///
+/// # Errors
+/// 上面四样里第一样不满足的，一句话。
+pub fn rollback_witness_of_system_configuration_slot(
+    slot: &[u8],
+    capacity: u64,
+) -> Result<Vec<RollbackWitnessEntryView>, &'static str> {
+    let start = usize::try_from(ROLLBACK_WITNESS_TABLE_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT)
+        .expect("481");
+    let count_bytes = usize::try_from(ROLLBACK_WITNESS_COUNT_BYTES).expect("1");
+    let entry_bytes = usize::try_from(ROLLBACK_WITNESS_ENTRY_BYTES).expect("16");
+    let positions = usize::try_from(ROLLBACK_WITNESS_ENTRIES_MAXIMUM).expect("47");
+    if slot.len() < start + count_bytes + positions * entry_bytes {
+        return Err("槽比见证表的末尾短");
+    }
+    let count = u64::from(slot[start]);
+    if count > capacity.min(ROLLBACK_WITNESS_ENTRIES_MAXIMUM) {
+        return Err("条数超过这个池的上限 R × S − 1");
+    }
+    let mut entries: Vec<RollbackWitnessEntryView> = Vec::new();
+    for position in 0..positions {
+        let base = start + count_bytes + position * entry_bytes;
+        let entry = RollbackWitnessEntryView {
+            new_instance: read_u32(slot, base),
+            rollback_target_instance: read_u32(slot, base + 4),
+            rollback_target_txg: read_u64(slot, base + 8),
+        };
+        if u64::try_from(position).expect("47 以内") < count {
+            if entry.rollback_target_instance >= entry.new_instance {
+                return Err("有一条的回退目标实例代号不小于新实例代号");
+            }
+            if entries.last().is_some_and(|previous| *previous >= entry) {
+                return Err("条目不按 (新实例, 目标实例, 目标 txg) 严格升序");
+            }
+            entries.push(entry);
+        } else if slot[base..base + entry_bytes].iter().any(|byte| *byte != 0) {
+            return Err("条数之后的条目位不全是 0");
+        }
+    }
+    Ok(entries)
+}
+
 /// 系统配置槽解出来的几个要紧字段。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct SystemConfigurationView {
@@ -449,6 +514,28 @@
         }
     }
     Ok(())
+}
+
+/// 判一个多层码 2 树内部节点的 key（D8（核心索引结构） 已定项 11）：自述 key 宽等于形态宽、条目按分隔 key 严格递增、节点不空。
+/// 头里的 key 区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2），不贴紧首末两条分隔 key——它要等孩子读回来才判得了，
+/// 由走读的一方判，这里不判。
+pub fn check_internal_node_separators(
+    view: &IndexNodeView,
+    schema: KeySchema,
+) -> Result<(), Verdict> {
+    if schema.width() != view.key_width {
+        return Err(Verdict::KeyWidthMismatch);
+    }
+    if view.entries.is_empty() {
+        return Err(Verdict::KeyOutsideDeclaredRange);
+    }
+    let key_of = |entry: &Vec<u8>| schema.fields(&entry[..view.key_width]);
+    for pair in view.entries.windows(2) {
+        if key_of(&pair[0]) >= key_of(&pair[1]) {
+            return Err(Verdict::KeysNotStrictlyAscending);
+        }
+    }
+    Ok(())
 }
 
 /// 码 3 打包记录单元解出来的头与记录（D18（块里携带什么信息） 已定项 11 / 已定项 16：记录区从 136 起）。
diff -ruN -x target tree/crates/singlefs-checker/src/walk.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-checker/src/walk.rs	2026-09-24 16:15:19.532697725 +0000
+++ tree/crates/singlefs-checker/src/walk.rs	2026-09-24 18:48:58.241408187 +0000
@@ -8,20 +8,22 @@
 
 use singlefs_format::{
     ACCOUNTING_ENTRY_BYTES, ALLOCATION_RECORD_BYTES, DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES,
-    INODE_INTERNAL_ENTRY, JOURNAL_RECORD_BYTES, MAPPING_ENTRY_BYTES, NODE_BYTES, SLOT_BYTES,
-    TREE_TABLE_ENTRY_BYTES,
+    INODE_INTERNAL_ENTRY, JOURNAL_RECORD_BYTES, MAPPING_ENTRY_BYTES, NODE_BYTES,
+    NODE_POINTER_BYTES, SLOT_BYTES, TREE_TABLE_ENTRY_BYTES,
 };
 
 use crate::image::{
     chosen_system_configurations, judge_location_order, parse_data_pointer, parse_node_pointer,
-    read_referenced_unit, root_slot_positions, valid_roots, verified_system_configuration_slots,
-    ImageReader, InvariantVerdict, Judgements, PointerView, PoolGeometry,
+    read_referenced_unit, rollback_witness_of_every_verified_slot, rollback_witness_of_the_pool,
+    root_slot_positions, valid_roots, verified_system_configuration_slots, ImageReader,
+    InvariantVerdict, Judgements, PointerView, PoolGeometry, RollbackWitnessOfASlot,
 };
 use crate::{
-    back_chain_of_record_header, check_index_node_keys, check_journal_record, check_unit,
-    checksum_field_holds, crc32_castagnoli_table, index_node_view, key_schema_for_tree_kind,
-    packed_unit_view, read_six_byte_unsigned, read_u16, read_u32, read_u64, KEY_SCHEMA_ALLOCATION,
-    KEY_SCHEMA_MAPPING, KEY_SCHEMA_TREE_TABLE,
+    back_chain_of_record_header, check_index_node_keys, check_internal_node_separators,
+    check_journal_record, check_unit, checksum_field_holds, crc32_castagnoli_table,
+    index_node_view, key_schema_for_tree_kind, packed_unit_view, read_six_byte_unsigned, read_u16,
+    read_u32, read_u64, KEY_SCHEMA_ACCOUNTING, KEY_SCHEMA_ALLOCATION, KEY_SCHEMA_MAPPING,
+    KEY_SCHEMA_TREE_TABLE,
 };
 
 const TREE_KIND_EXTENT: u16 = 1;
@@ -96,6 +98,59 @@
     }
 }
 
+/// 读一个码 2 节点时内部节点的 key 区间怎么判（I-1.1）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum KeyRangeReading {
+    /// 区间贴紧首末两条条目的 key：叶一律这样；inode 树根、树表与根兼叶的那几棵今天也这样写。
+    FirstAndLastEntries,
+    /// 多层码 2 树（记账树、中央映射树）：叶贴紧首末条目；内部节点的区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2），
+    /// 要等孩子读回来才判得了，由走读那一方判（`Walk::walk_code_two_subtree` ③），读节点这一步只判分隔 key 严格递增。
+    SubtreeCoverageOfInternalNodesJudgedByTheCaller,
+}
+
+/// 多层码 2 树一个节点的条目宽在走读里怎么对待。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum EntryWidthInTheWalk {
+    /// 按 I-1.10（码 2 条目宽等于字段表宽） 判：记账树叶 34、内部节点 108。
+    JudgedAsInvariantOneTen { field_table_bytes: usize },
+    /// 只守走读按固定偏移切要几个字节，不判「该有多宽」：中央映射树（C307 还开着，I-1.10 不罩它）。
+    GuardedForTheWalkOnly { bytes_the_walk_needs: usize },
+}
+
+/// 走一棵多层码 2 树要的几样。
+#[derive(Clone, Copy, Debug)]
+struct MultiLevelTreeInTheWalk<'name> {
+    tree: u64,
+    schema: crate::KeySchema,
+    leaf_entry_width: EntryWidthInTheWalk,
+    internal_entry_width: EntryWidthInTheWalk,
+    name: &'name str,
+}
+
+/// 走一个节点连同它下面的结果：交回它头里的 key 区间（父节点判两条不等式与覆盖区间要它）与这棵子树走全了没有。
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum SubtreeInTheWalk {
+    Walked {
+        smallest_key: Vec<u8>,
+        largest_key: Vec<u8>,
+        is_complete: bool,
+    },
+    /// 这一遍之前别的根已经走过它（`visited_units`）：那时按它自己的父条目判过，这里不再判。
+    AlreadyWalkedFromAnotherRoot,
+    /// 读不出、头用不了、层级不对或条目宽不对：走读失败已记，这一支不往下走。
+    NotWalkable,
+}
+
+/// 指向码 2 / 码 3 的节点指针宽：头部 50 + 位置条目 14 × 2 + 实例代号 4 + 出生序号 4。
+fn node_pointer_bytes() -> usize {
+    usize::try_from(NODE_POINTER_BYTES).expect("86")
+}
+
+/// 多层码 2 树内部节点的条目宽：本树 key + 子指针 86（D8（核心索引结构） 已定项 11：记账树 108、中央映射树 113）。
+fn internal_entry_bytes(schema: crate::KeySchema) -> usize {
+    schema.width() + node_pointer_bytes()
+}
+
 /// 一次走读的状态：判定累加器、被引用单元的物理范围、走读有没有断。
 struct Walk<'reader> {
     reader: &'reader dyn ImageReader,
@@ -377,13 +432,15 @@
         true
     }
 
-    /// 读一个码 2 节点并判头、树 ID（I-1.3）与 key 区间（I-1.1）。
+    /// 读一个码 2 节点并判头、树 ID（I-1.3）与 key 区间（I-1.1）。`key_range_reading` 说内部节点的 key 区间怎么判：
+    /// 首末两条条目的 key（inode 树根、树表等今天的写法），或多层码 2 树的子树覆盖区间（孩子读回来之后由走读那一方判）。
     fn read_index_node(
         &mut self,
         pointer_bytes: &[u8],
         expected_tree: u64,
         schema: crate::KeySchema,
         what: &str,
+        key_range_reading: KeyRangeReading,
     ) -> Option<crate::IndexNodeView> {
         let pointer = parse_node_pointer(pointer_bytes);
         if pointer.all_zero {
@@ -425,7 +482,15 @@
                     view.tree_identifier
                 )
             });
-        let keys = check_index_node_keys(&view, schema);
+        let keys = match (key_range_reading, view.level) {
+            (KeyRangeReading::FirstAndLastEntries, _)
+            | (KeyRangeReading::SubtreeCoverageOfInternalNodesJudgedByTheCaller, 0) => {
+                check_index_node_keys(&view, schema)
+            }
+            (KeyRangeReading::SubtreeCoverageOfInternalNodesJudgedByTheCaller, _) => {
+                check_internal_node_separators(&view, schema)
+            }
+        };
         self.judgements.judge("I-1.1", keys.is_ok(), || {
             format!("{what}：key 宽 / key 区间与条目不符（{keys:?}）")
         });
@@ -448,6 +513,7 @@
             0,
             KEY_SCHEMA_ALLOCATION,
             "树表 0 条那一版的分配记录树的根",
+            KeyRangeReading::FirstAndLastEntries,
         );
         // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
         self.walk_tree_table_and_central_mapping_root(&record[36..122], &record[256..342]);
@@ -473,9 +539,13 @@
         mapping_root_pointer: &[u8],
     ) {
         // 树表单元：不属于任何树（树 ID 0）。
-        let Some(tree_table) =
-            self.read_index_node(tree_table_pointer, 0, KEY_SCHEMA_TREE_TABLE, "树表单元")
-        else {
+        let Some(tree_table) = self.read_index_node(
+            tree_table_pointer,
+            0,
+            KEY_SCHEMA_TREE_TABLE,
+            "树表单元",
+            KeyRangeReading::FirstAndLastEntries,
+        ) else {
             return;
         };
         for entry in &tree_table.entries {
@@ -485,33 +555,29 @@
         // I-1.3（块头树 ID 一致） 的「实际引用它的树」按那条指针头部的出生树取（D19（块指针的结构与宽度预算） 已定项 7），
         // 与树表条目里的树 ID 对树表指着的根是同一个读法。号不写死 15：回退到树表 0 条的一版之后再发第一个文件版本，
         // 八棵树从水位重新发号（C511（回退到无文件那一版之后诞生代怎么接））。
-        if let Some(mapping) = self.read_index_node(
-            mapping_root_pointer,
-            parse_node_pointer(mapping_root_pointer).birth_tree,
-            KEY_SCHEMA_MAPPING,
-            "中央映射树的根",
-        ) {
-            // 条目宽是映射树根头里的一个**盘上字段**（`index_node_view` 只判了它 ≥ key 宽 27）：切到偏移 55 之前
-            // 先判一次，窄的如实记一条走读断了、不按字段表的固定偏移切（普查 R2 / R13 的映射那一处）。
-            // **这里判的是「今天走读要几个字节」，不是「映射条目该有多宽」**：后者归 C307（映射树两种 key 宽怎么装进
-            // 一棵定宽 key 的树），那一条还开着，I-1.10（码 2 条目宽等于字段表宽） 的射程里也没有中央映射树
-            // （`field_table_width_of_tree_kind` 不认种类 5）。
-            if mapping.entry_width < mapping_entry_bytes() {
-                self.walk_failures.push(format!(
-                    "中央映射树的根自述的条目宽 {} 小于走读要的 {} 字节，映射条目这一支走不下去",
-                    mapping.entry_width,
-                    mapping_entry_bytes()
-                ));
-            } else {
-                self.walk_central_mapping_entries(&mapping);
-            }
-        }
+        // 映射树可以是多层（D8（核心索引结构） 已定项 11）：整棵走下去，每个节点按父条目核层级、区间与分隔 key（I-1.1）。
+        // 叶的条目宽**只守走读要几个字节，不判「映射条目该有多宽」**：后者归 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树），
+        // 那一条还开着，I-1.10（码 2 条目宽等于字段表宽） 的射程里也没有中央映射树（`field_table_width_of_tree_kind` 不认种类 5）。
+        let mapping_tree = MultiLevelTreeInTheWalk {
+            tree: parse_node_pointer(mapping_root_pointer).birth_tree,
+            schema: KEY_SCHEMA_MAPPING,
+            leaf_entry_width: EntryWidthInTheWalk::GuardedForTheWalkOnly {
+                bytes_the_walk_needs: mapping_entry_bytes(),
+            },
+            internal_entry_width: EntryWidthInTheWalk::GuardedForTheWalkOnly {
+                bytes_the_walk_needs: internal_entry_bytes(KEY_SCHEMA_MAPPING),
+            },
+            name: "中央映射树",
+        };
+        let mut leaf_entries = Vec::new();
+        self.walk_code_two_subtree(mapping_root_pointer, None, &mapping_tree, &mut leaf_entries);
+        self.walk_central_mapping_entries(&leaf_entries);
     }
 
-    /// 中央映射树根的条目逐条走：每条条目里那两条位置条目按升序判，指的单元两份各读一遍。
-    /// 入参的条目宽由调用方判过 ≥ 55（`walk_tree_table_and_central_mapping_root` 那一道），这里按字段表的固定偏移切。
-    fn walk_central_mapping_entries(&mut self, mapping: &crate::IndexNodeView) {
-        for entry in &mapping.entries {
+    /// 中央映射树叶里的条目逐条走：每条条目里那两条位置条目按升序判，指的单元两份各读一遍。
+    /// 入参的条目宽由走叶的那一步判过 ≥ 55（`EntryWidthInTheWalk::GuardedForTheWalkOnly`），这里按字段表的固定偏移切。
+    fn walk_central_mapping_entries(&mut self, entries: &[Vec<u8>]) {
+        for entry in entries {
             let locations_pointer: Vec<u8> =
                 [vec![0u8; 50], entry[27..55].to_vec(), vec![0u8; 8]].concat();
             let view = parse_node_pointer(&locations_pointer);
@@ -536,6 +602,180 @@
         }
     }
 
+    /// 多层码 2 树（记账树、中央映射树，D8（核心索引结构） 已定项 11）的一个节点连同它下面：节点照 `read_index_node` 判头、
+    /// 出生身份（I-1.2 / I-4.2）、树 ID（I-1.3）与自己的 key（I-1.1：叶的区间贴紧首末条目、内部节点的分隔 key 严格递增），
+    /// 再按引用它的父条目判它在树里的身份——**I-1.1（块头自述逻辑地址）**：索引节点的身份是树 ID + 层级 + key 区间
+    /// （D18（块里携带什么信息） 已定项 2 的子树覆盖区间），全从块头自带的那几样读，不另存旁路表：
+    /// ① 孩子头里的层级 = 父层级 − 1（层级 0 是叶）；
+    /// ② 分隔 key_i ≤ 第 i 个孩子头里的最小 key，且 > 第 i − 1 个孩子头里的最大 key（它管的区间就是父条目给的那一段）；
+    /// ③ 内部节点头里的区间 = [第一个孩子头里的最小 key, 最后一个孩子头里的最大 key]。
+    /// 条目宽：叶按 `leaf_entry_width`、内部节点按 `internal_entry_width`（记账树判 I-1.10，中央映射树只守走读要几个字节）。
+    /// 叶里的条目按走到的次序（= key 升序）追加进 `leaf_entries`。
+    ///
+    /// 迭代与递归的上界：每一层下降层级减一（①不成立的孩子不往下走），每个单元只走一次（`visited_units`）。
+    fn walk_code_two_subtree(
+        &mut self,
+        pointer_bytes: &[u8],
+        expected_level: Option<u8>,
+        tree: &MultiLevelTreeInTheWalk<'_>,
+        leaf_entries: &mut Vec<Vec<u8>>,
+    ) -> SubtreeInTheWalk {
+        let pointer = parse_node_pointer(pointer_bytes);
+        let what = match expected_level {
+            None => format!("{}（树 {}）的根", tree.name, tree.tree),
+            Some(level) => format!("{}（树 {}）层级 {level} 的节点", tree.name, tree.tree),
+        };
+        if pointer.all_zero {
+            if expected_level.is_some() {
+                self.walk_failures
+                    .push(format!("{what}：父条目里的子指针全零"));
+            }
+            return SubtreeInTheWalk::NotWalkable;
+        }
+        let already_walked_from_another_root = self
+            .visited_units
+            .contains(&(pointer.locations[0].device, pointer.locations[0].slot));
+        let Some(view) = self.read_index_node(
+            pointer_bytes,
+            tree.tree,
+            tree.schema,
+            &what,
+            KeyRangeReading::SubtreeCoverageOfInternalNodesJudgedByTheCaller,
+        ) else {
+            return if already_walked_from_another_root {
+                SubtreeInTheWalk::AlreadyWalkedFromAnotherRoot
+            } else {
+                SubtreeInTheWalk::NotWalkable
+            };
+        };
+        if let Some(level) = expected_level {
+            self.judgements.judge("I-1.1", view.level == level, || {
+                format!(
+                    "{what}：头里的层级是 {}，引用它的父节点要层级 {level}（父层级减一）",
+                    view.level
+                )
+            });
+            if view.level != level {
+                self.walk_failures
+                    .push(format!("{what}：层级不是父层级减一，这一支走不下去"));
+                return SubtreeInTheWalk::NotWalkable;
+            }
+        }
+        let entry_width_reading = if view.level == 0 {
+            tree.leaf_entry_width
+        } else {
+            tree.internal_entry_width
+        };
+        if !self.entry_width_holds(&view, entry_width_reading, &what) {
+            return SubtreeInTheWalk::NotWalkable;
+        }
+        if view.level == 0 {
+            leaf_entries.extend(view.entries.iter().cloned());
+            return SubtreeInTheWalk::Walked {
+                smallest_key: view.smallest_key,
+                largest_key: view.largest_key,
+                is_complete: true,
+            };
+        }
+        let key_width = view.key_width;
+        let fields = |key: &[u8]| tree.schema.fields(key);
+        let mut is_complete = true;
+        let mut children: Vec<(Vec<u8>, SubtreeInTheWalk)> = Vec::with_capacity(view.entries.len());
+        for entry in &view.entries {
+            let separator_key = entry[..key_width].to_vec();
+            let child_pointer_bytes = &entry[key_width..key_width + node_pointer_bytes()];
+            judge_location_order(
+                &mut self.judgements,
+                &parse_node_pointer(child_pointer_bytes),
+                &format!("{what} 的子指针"),
+            );
+            let child = self.walk_code_two_subtree(
+                child_pointer_bytes,
+                Some(view.level - 1),
+                tree,
+                leaf_entries,
+            );
+            match &child {
+                SubtreeInTheWalk::Walked {
+                    is_complete: child_is_complete,
+                    ..
+                } => is_complete &= *child_is_complete,
+                SubtreeInTheWalk::AlreadyWalkedFromAnotherRoot => {}
+                SubtreeInTheWalk::NotWalkable => is_complete = false,
+            }
+            children.push((separator_key, child));
+        }
+        // ② 两条不等式：只判读回来、这一遍第一次走到的孩子（别的根走过的那几个，那时已经按它们自己的父条目判过）。
+        for (position, (separator_key, child)) in children.iter().enumerate() {
+            let SubtreeInTheWalk::Walked { smallest_key, .. } = child else {
+                continue;
+            };
+            self.judgements.judge(
+                "I-1.1",
+                fields(separator_key) <= fields(smallest_key),
+                || format!("{what}：第 {position} 条的分隔 key 大于这个孩子头里的最小 key"),
+            );
+            if position == 0 {
+                continue;
+            }
+            if let (_, SubtreeInTheWalk::Walked { largest_key, .. }) = &children[position - 1] {
+                self.judgements
+                    .judge("I-1.1", fields(separator_key) > fields(largest_key), || {
+                        format!("{what}：第 {position} 条的分隔 key 不大于左邻孩子头里的最大 key")
+                    });
+            }
+        }
+        // ③ 头里的区间是子树覆盖区间。
+        if let (
+            Some((_, SubtreeInTheWalk::Walked { smallest_key, .. })),
+            Some((_, SubtreeInTheWalk::Walked { largest_key, .. })),
+        ) = (children.first(), children.last())
+        {
+            let covers = *smallest_key == view.smallest_key && *largest_key == view.largest_key;
+            self.judgements.judge("I-1.1", covers, || {
+                format!("{what}：头里的 key 区间不是子树覆盖区间（第一个孩子的最小 key 到最后一个孩子的最大 key）")
+            });
+        }
+        SubtreeInTheWalk::Walked {
+            smallest_key: view.smallest_key,
+            largest_key: view.largest_key,
+            is_complete,
+        }
+    }
+
+    /// 一个多层码 2 树节点的条目宽：按 I-1.10 判（记账树），或只守走读要几个字节（中央映射树）；不成立就不往下切。
+    fn entry_width_holds(
+        &mut self,
+        view: &crate::IndexNodeView,
+        reading: EntryWidthInTheWalk,
+        what: &str,
+    ) -> bool {
+        match reading {
+            EntryWidthInTheWalk::JudgedAsInvariantOneTen { field_table_bytes } => {
+                let entry_width = view.entry_width;
+                self.judgements
+                    .judge("I-1.10", entry_width == field_table_bytes, || {
+                        format!(
+                            "{what}：头里自述的条目宽 {entry_width} 不等于这一层的条目字段表宽度 {field_table_bytes}"
+                        )
+                    });
+                entry_width == field_table_bytes
+            }
+            EntryWidthInTheWalk::GuardedForTheWalkOnly {
+                bytes_the_walk_needs,
+            } => {
+                if view.entry_width < bytes_the_walk_needs {
+                    self.walk_failures.push(format!(
+                        "{what}自述的条目宽 {} 小于走读要的 {bytes_the_walk_needs} 字节，这一支走不下去",
+                        view.entry_width
+                    ));
+                    return false;
+                }
+                true
+            }
+        }
+    }
+
     /// 树表的一条条目：条目宽是树表单元头里的一个**盘上字段**（`index_node_view` 只判了它 ≥ key 宽 8），而下面三步——
     /// 读树 ID（偏移 0）、读种类（偏移 10）、切根指针（`entry[14..100]`）——都按字段表的固定偏移切，窄一个字节就当场越界。
     /// 切之前先判一次，窄的如实记一条「走读断了」（I-4.8（近 K 代根校验和自洽） / I-7.4（近 K 代块未被复用） 由它说话），
@@ -573,7 +813,42 @@
                 }
             }
         }
-        let Some(node) = self.read_index_node(&entry[14..100], tree, schema, &what) else {
+        // 记账树可以是多层（D8（核心索引结构） 已定项 11）：整棵走下去，每个节点按父条目判层级、区间与分隔 key（I-1.1），
+        // 条目宽按层判 I-1.10（叶 34、内部节点 108 = key 22 + 子指针 86）。记账行只取走全了的那一遍：有一个节点读不出、
+        // 头用不了或条目宽不对，行就数不全，I-3.1 那几条按「这一版没有记账可比」报不适用，不拿半张账去比。
+        if kind == TREE_KIND_ACCOUNTING {
+            let accounting_tree = MultiLevelTreeInTheWalk {
+                tree,
+                schema,
+                leaf_entry_width: EntryWidthInTheWalk::JudgedAsInvariantOneTen {
+                    field_table_bytes: field_table_width_of_tree_kind(TREE_KIND_ACCOUNTING)
+                        .expect("记账树登记了条目字段表"),
+                },
+                internal_entry_width: EntryWidthInTheWalk::JudgedAsInvariantOneTen {
+                    field_table_bytes: internal_entry_bytes(schema),
+                },
+                name: "记账树",
+            };
+            let mut rows = Vec::new();
+            if let SubtreeInTheWalk::Walked {
+                is_complete: true, ..
+            } = self.walk_code_two_subtree(&entry[14..100], None, &accounting_tree, &mut rows)
+            {
+                self.accounting_seen = true;
+                for row in &rows {
+                    self.accounting
+                        .insert((read_u16(row, 0), read_u32(row, 10)), read_u64(row, 22));
+                }
+            }
+            return;
+        }
+        let Some(node) = self.read_index_node(
+            &entry[14..100],
+            tree,
+            schema,
+            &what,
+            KeyRangeReading::FirstAndLastEntries,
+        ) else {
             return;
         };
         // I-1.10：头里自述的条目宽等于这棵树登记的条目字段表宽度，**先于**按字段表解条目判
@@ -594,21 +869,15 @@
         }
         match kind {
             TREE_KIND_INODE => self.walk_inode_root(&node, tree),
-            TREE_KIND_EXTENT | TREE_KIND_ALLOCATION | TREE_KIND_ACCOUNTING if node.level > 0 => {
-                // 另外四棵树的内部节点条目格式没有条款（总审核 D8-D11 发现 16）：走不下去就如实说。
+            TREE_KIND_EXTENT | TREE_KIND_ALLOCATION if node.level > 0 => {
+                // 这两棵树按 key 空间另定结构（D8（核心索引结构） 已定项 14，三方 `m2-keyspace-r1`），内部节点今天没有条款：
+                // 走不下去就如实说。
                 self.judgements.not_applicable(
                     "I-7.2",
-                    "extent / 分配 / 记账树有内部节点，而它们的内部条目格式还没有条款",
+                    "extent / 分配记录树有内部节点，而它们按 key 空间定的结构还没有条款",
                 );
             }
             TREE_KIND_EXTENT => self.walk_extent_leaf(&node, tree),
-            TREE_KIND_ACCOUNTING => {
-                self.accounting_seen = true;
-                for row in &node.entries {
-                    self.accounting
-                        .insert((read_u16(row, 0), read_u32(row, 10)), read_u64(row, 22));
-                }
-            }
             _ => {}
         }
     }
@@ -1377,7 +1646,21 @@
             references.is_complete = false;
             continue;
         };
-        collect_tree_references(&mut references, &node, read_u16(entry, 10));
+        collect_tree_references(reader, cache, &mut references, &node, read_u16(entry, 10));
+    }
+    // 中央映射树根之下的节点（多层时，D8（核心索引结构） 已定项 11）：只有它们的父节点指着，不数它们，这条根的引用集合就少几片。
+    // 只在「走到叶」那个深度上数（引用集合只在那个深度上作数）。
+    if depth == ReferenceScanDepth::EveryReferencedPlacement && !mapping_root.all_zero {
+        match read_index_node_without_judging(reader, &mapping_root, cache) {
+            Some(mapping_root_node) => note_every_node_below(
+                reader,
+                cache,
+                &mut references,
+                &mapping_root_node,
+                KEY_SCHEMA_MAPPING,
+            ),
+            None => references.is_complete = false,
+        }
     }
     references
 }
@@ -1429,9 +1712,44 @@
     }
 }
 
-/// 一棵树的根节点下面还引用了什么。第一版只有「extent 叶的数据指针」与「inode 树内部条目的子指针」两种下探；
-/// 别的形状（另外四棵树的内部节点、条目格式没有条款的树）走不下去，如实记成数不全。
+/// 多层码 2 树（记账树、中央映射树）一个节点下面的全部节点都记进引用集合（它们各占一个槽）：按内部条目里的子指针逐层往下，
+/// 条目宽窄于 key 宽 + 86、孩子读不出、孩子的层级不是父层级减一，都如实记成数不全、不往下走（那几样由主走读的 I-1.1 / I-1.10 说话）。
+/// 递归的上界：每下一层层级减一。
+fn note_every_node_below(
+    reader: &dyn ImageReader,
+    cache: &mut IndexNodeCache,
+    references: &mut RootReferences,
+    node: &crate::IndexNodeView,
+    schema: crate::KeySchema,
+) {
+    if node.level == 0 {
+        return;
+    }
+    let key_width = schema.width();
+    if node.key_width != key_width || node.entry_width < internal_entry_bytes(schema) {
+        references.is_complete = false;
+        return;
+    }
+    for entry in &node.entries {
+        let child_pointer = parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]);
+        references.note(&child_pointer);
+        let Some(child) = read_index_node_without_judging(reader, &child_pointer, cache) else {
+            references.is_complete = false;
+            continue;
+        };
+        if child.level + 1 != node.level {
+            references.is_complete = false;
+            continue;
+        }
+        note_every_node_below(reader, cache, references, &child, schema);
+    }
+}
+
+/// 一棵树的根节点下面还引用了什么：「extent 叶的数据指针」「inode 树内部条目的子指针」与记账树根之下的全部节点三种下探；
+/// 别的形状（extent 与分配记录树的内部节点、条目格式没有条款的树）走不下去，如实记成数不全。
 fn collect_tree_references(
+    reader: &dyn ImageReader,
+    cache: &mut IndexNodeCache,
     references: &mut RootReferences,
     node: &crate::IndexNodeView,
     tree_kind_code: u16,
@@ -1470,10 +1788,8 @@
             }
         }
         TreeKindForReferenceScan::Accounting => {
-            // 记账行不引用别的单元；有内部节点时它的条目格式没有条款（总审核 D8-D11 发现 16）。
-            if node.level > 0 {
-                references.is_complete = false;
-            }
+            // 记账行不引用别的单元；多层时根之下的节点各占一个槽（D8（核心索引结构） 已定项 11：内部条目 = key 22 + 子指针 86）。
+            note_every_node_below(reader, cache, references, node, KEY_SCHEMA_ACCOUNTING);
         }
         TreeKindForReferenceScan::WithoutWalkableEntryFormat(_) => references.is_complete = false,
     }
@@ -2289,6 +2605,82 @@
     }
 }
 
+/// 隔离的记录（D19（块指针的结构与宽度预算） 已定项 5：释放之前读盘核出对不上的那个单元，每块盘上的分配记录都留在已分配，
+/// 映射条目去掉、没有任何根再引用它）在 I-3.1（已分配统计对得上） 与 I-3.11（已分配减 defer 等于最新根走读） 上怎么认
+/// （用户 2026-09-25 定；**按单元判**是主 agent 同日定的读法，I-3.1 / I-3.11 两行的措辞随后由书记员改）：已分配而没有根引用的记录，
+/// 它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出、或自证不过，这个单元在每块盘上的那几条记录都豁免；
+/// 这个单元每一份都读得出且对得上，照旧判违例。按单元而不按份判：写者只要一份核出对不上就把两块盘的记录一起留在已分配
+/// （D19 已定项 5「另一块盘上那一份对得上也一起留，各盘的账保持对称」），按份判的话对得上那块盘上的那一条恒红。
+///
+/// 读的是 `root` 那棵分配记录树（树表里种类 3 那一条指着的根节点，第一版只有一个节点）：未释放、`referenced_start_slots`
+/// 里没有哪个引用起在它那一槽的记录，按单元（起点槽、跨度——第一版一个单元两盘同槽）归组；一组的每一份按 (盘, 槽, 跨度) 读那一整份单元、
+/// 交 `check_unit` 判（magic、头校验和、载荷 CRC）——有一份读不出或任一关不过，整组豁免。逐盘交回豁免的槽数；
+/// 树表或分配记录树读不出、有内部节点时一格都不豁免（交回空表，照旧判）。
+fn quarantined_slots_exempted_per_device(
+    reader: &dyn ImageReader,
+    root: &crate::RootView,
+    referenced_start_slots: &BTreeSet<(u32, u64)>,
+    cache: &mut IndexNodeCache,
+) -> BTreeMap<u32, u64> {
+    let mut exempted: BTreeMap<u32, u64> = BTreeMap::new();
+    let tree_table_pointer = parse_node_pointer(&root.record_bytes[36..122]);
+    if tree_table_pointer.all_zero {
+        return exempted;
+    }
+    let Some(tree_table) = read_index_node_without_judging(reader, &tree_table_pointer, cache)
+    else {
+        return exempted;
+    };
+    // 一个单元（起点槽、跨度）→ 记着它、没有根引用的那几块盘。
+    let mut unreferenced_units: BTreeMap<(u64, u64), Vec<u32>> = BTreeMap::new();
+    for entry in &tree_table.entries {
+        if entry.len() < tree_table_entry_bytes() || read_u16(entry, 10) != TREE_KIND_ALLOCATION {
+            continue;
+        }
+        let allocation_root = parse_node_pointer(&entry[14..100]);
+        if allocation_root.all_zero {
+            continue;
+        }
+        let Some(node) = read_index_node_without_judging(reader, &allocation_root, cache) else {
+            continue;
+        };
+        if node.level > 0 || node.entry_width < allocation_record_bytes() {
+            continue;
+        }
+        for record in node
+            .entries
+            .iter()
+            .map(|record_bytes| parse_allocation_record(record_bytes))
+        {
+            if record.is_released || referenced_start_slots.contains(&(record.device, record.slot))
+            {
+                continue;
+            }
+            unreferenced_units
+                .entry((record.slot, record.span_slots))
+                .or_default()
+                .push(record.device);
+        }
+    }
+    for ((slot, span_slots), devices_of_the_unit) in unreferenced_units {
+        let copy_self_verifies = |device: u32| {
+            usize::try_from(span_slots * SLOT_BYTES)
+                .ok()
+                .and_then(|unit_bytes| reader.read(device, slot * SLOT_BYTES, unit_bytes))
+                .is_some_and(|unit| check_unit(&unit).is_ok())
+        };
+        let some_copy_fails = devices_of_the_unit
+            .iter()
+            .any(|device| !copy_self_verifies(*device));
+        if some_copy_fails {
+            for device in devices_of_the_unit {
+                *exempted.entry(device).or_insert(0) += span_slots;
+            }
+        }
+    }
+    exempted
+}
+
 /// 一条根的分配记录树里有几条记录：按 checker 自己的字段表读这条根的树表，找种类 3（分配记录）的条目，数它根节点叶里的条目
 /// （第一版分配记录树只有一个节点；一条记录记一个单元，D3（空间分配） 已定项 7）。只读不判——给理想模型的对拍数分配记录墙的真条数
 /// （增补 3 第 2 件代码三方第一轮判决第三节第 1 条：从镜像上现数，不用分配器的状态）。
@@ -3430,6 +3822,114 @@
     slots_per_device
 }
 
+/// I-7.10（回退见证表各槽自洽、各盘一致）：每块盘两槽里自证过的系统配置槽，槽里的回退见证表都解得开
+/// （条数不超过 R × S − 1、条目按 (新实例, 目标实例, 目标 txg) 严格升序、每一条目标实例代号小于新实例代号、条数之后的条目位全 0，
+/// `rollback_witness_of_system_configuration_slot`）；同一个新实例代号在各盘各槽里记的回退目标相同（一次回退一条，
+/// 写它的只有那一次回退挂载，不许两个槽说两个目标）。各盘、各槽的条目集合可以不同：轮换写到一半崩了一新一旧，挂载按删除规则删过的条目
+/// 旧槽里还留着——比的只是同一个新实例代号有没有两种说法。
+fn judge_rollback_witness_tables(slots: &[RollbackWitnessOfASlot], judgements: &mut Judgements) {
+    let mut target_of_each_new_instance: BTreeMap<u32, (u32, u64, u32)> = BTreeMap::new();
+    for slot in slots {
+        match &slot.witness {
+            Err(what) => judgements.judge("I-7.10", false, || {
+                format!(
+                    "盘 {} 世代号 {} 那一槽的回退见证表解不开：{what}",
+                    slot.device, slot.slot_generation
+                )
+            }),
+            Ok(entries) => {
+                judgements.judge("I-7.10", true, String::new);
+                for entry in entries {
+                    let target = (entry.rollback_target_instance, entry.rollback_target_txg);
+                    match target_of_each_new_instance.get(&entry.new_instance) {
+                        None => {
+                            target_of_each_new_instance
+                                .insert(entry.new_instance, (target.0, target.1, slot.device));
+                        }
+                        Some((instance, txg, device)) => {
+                            let (recorded_instance, recorded_txg, recorded_device) =
+                                (*instance, *txg, *device);
+                            judgements.judge(
+                                "I-7.10",
+                                (recorded_instance, recorded_txg) == target,
+                                || {
+                                    format!(
+                                        "新实例 {} 的回退目标两种说法：盘 {recorded_device} 记 ({recorded_instance}, {recorded_txg})，盘 {} 世代号 {} 记 ({}, {})",
+                                        entry.new_instance, slot.device, slot.slot_generation, target.0, target.1
+                                    )
+                                },
+                            );
+                        }
+                    }
+                }
+            }
+        }
+    }
+    if slots.is_empty() {
+        judgements.not_applicable("I-7.10", "池里没有一个自证过的系统配置槽：见证表无从读起");
+    }
+}
+
+/// I-7.11（回退见证表与所选根的实例表对得上）：所选根 = 根环里自证过、不被见证表（各盘择到的那一槽取并集）抛弃的根里
+/// (txg, 实例) 最大的那一条（D23（journal 的角色与格式） 已定项 14「回退见证」：择根先跳过被任一条目抛弃的根）。
+/// 见证表里新实例代号 N 不大于所选根实例代号的每一条 (N, r_old, T_old)，所选根指着的实例表都罩得住它：r_old 那一行记的 T 不大于 T_old
+/// （r_old 是 0——回退到 mkfs 的第 0 代根——时不要这一行：实例 0 不写行、只有 txg 0 那一条根），
+/// (r_old, N) 之间的每个实例都有一行、T 是 0——条目抛弃的根按那张表也判抛弃。回退那一次挂载写回退行 (r_old, T_old) 与中间实例的 (i, 0, 0)、
+/// 之后的挂载只接行不回头改旧行，所以所选根在回退之后的时间线上时恒罩得住；罩不住就是见证写错了、或回退行没写上。
+/// 「所选根不被见证表抛弃」这一格每个有根的镜像上都判一次：每一条根都被见证表抛弃（择根择不出来）就红。
+/// 新实例代号大于所选根实例的条目不判：所选根不在那次回退之后的时间线上（回退实例的根读不出、落回 R_old 那一格），那张表里本来就没有它的行。
+fn judge_rollback_witness_against_the_chosen_roots_table(
+    rollback_witness: &[crate::RollbackWitnessEntryView],
+    a_root_survives_the_witness: bool,
+    chosen_root: &crate::RootView,
+    instance_table_rows: &[InstanceTableRow],
+    judgements: &mut Judgements,
+) {
+    judgements.judge("I-7.11", a_root_survives_the_witness, || {
+        format!(
+            "根环里每一条自证过的根都被回退见证表抛弃（{} 条条目）：择根择不出来",
+            rollback_witness.len()
+        )
+    });
+    if !a_root_survives_the_witness {
+        return;
+    }
+    for entry in rollback_witness
+        .iter()
+        .filter(|entry| entry.new_instance <= chosen_root.instance)
+    {
+        // 回退到 mkfs 的第 0 代根（目标实例 0）：实例 0 不写行（D18（块里携带什么信息） 已定项 11），它只有 txg 0 那一条根，
+        // 没有越过目标的根要抛弃——这一格不要行。
+        let target_row_covers = entry.rollback_target_instance == 0
+            || instance_table_rows.iter().any(|row| {
+                row.instance == entry.rollback_target_instance
+                    && row.published_checkpoint_txg <= entry.rollback_target_txg
+            });
+        let missing_intermediate =
+            (entry.rollback_target_instance + 1..entry.new_instance).find(|instance| {
+                !instance_table_rows
+                    .iter()
+                    .any(|row| row.instance == *instance && row.published_checkpoint_txg == 0)
+            });
+        judgements.judge(
+            "I-7.11",
+            target_row_covers && missing_intermediate.is_none(),
+            || {
+                format!(
+                    "所选根（实例 {}、txg {}）的实例表罩不住见证条目 (新实例 {}, 目标 ({}, {}))：目标实例那一行 T ≤ 目标 txg {}；中间实例 {:?} 没有 T = 0 的行",
+                    chosen_root.instance,
+                    chosen_root.checkpoint_txg,
+                    entry.new_instance,
+                    entry.rollback_target_instance,
+                    entry.rollback_target_txg,
+                    if target_row_covers { "有" } else { "没有" },
+                    missing_intermediate
+                )
+            },
+        );
+    }
+}
+
 /// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。
 #[must_use]
 pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
@@ -3519,12 +4019,31 @@
         return root_ring_judgements.into_report();
     }
     judge_root_ring_health(&roots, &mut root_ring_judgements);
-    let newest_index = roots
+    // 回退见证表（D23（journal 的角色与格式） 已定项 14「回退见证」）：各槽自洽、各盘一致判 I-7.10；择根先跳过被见证表抛弃的根
+    // （与实现 `recovery::choose_root` 同一个读法：各盘择到的那一槽里的表取并集），再按 (txg, 实例) 取最大。
+    let witness_of_every_slot = rollback_witness_of_every_verified_slot(reader);
+    judge_rollback_witness_tables(&witness_of_every_slot, &mut root_ring_judgements);
+    let rollback_witness = rollback_witness_of_the_pool(&witness_of_every_slot);
+    let abandoned_by_the_witness = |root: &crate::RootView| {
+        rollback_witness
+            .iter()
+            .any(|entry| entry.abandons(root.instance, root.checkpoint_txg))
+    };
+    let newest_index_not_abandoned_by_the_witness = roots
         .iter()
         .enumerate()
+        .filter(|(_, (_, _, root))| !abandoned_by_the_witness(root))
         .max_by_key(|(_, (_, _, root))| (root.checkpoint_txg, root.instance))
-        .map(|(index, _)| index)
-        .expect("非空");
+        .map(|(index, _)| index);
+    // 每一条根都被见证表抛弃（坏镜像才有）：I-7.11 判红，别的几条照最大的那一条走下去，不早退。
+    let newest_index = newest_index_not_abandoned_by_the_witness.unwrap_or_else(|| {
+        roots
+            .iter()
+            .enumerate()
+            .max_by_key(|(_, (_, _, root))| (root.checkpoint_txg, root.instance))
+            .map(|(index, _)| index)
+            .expect("非空")
+    });
     let mut walk = Walk {
         reader,
         judgements: root_ring_judgements,
@@ -3550,6 +4069,11 @@
     // I-3.11（已分配减 defer 等于最新根走读）要的「从最新有效根走读到的、这块盘上被引用的槽数」：走法与 I-3.1 同一个
     // （`note_reference` 记下的 (设备, 起点槽, 跨度)），只取最新根——这一刻 `references` 里还只有最新根这一遍记下的。
     let slots_referenced_by_the_newest_root = slots_referenced_per_device(&walk.references);
+    let start_slots_referenced_by_the_newest_root: BTreeSet<(u32, u64)> = walk
+        .references
+        .keys()
+        .map(|(device, start_slot, _)| (*device, *start_slot))
+        .collect();
     // I-4.8（近 K 代根校验和自洽）与 I-7.4（近 K 代块未被复用）：候选集里任一根（最新根也在候选集里）出发遍历，所有块的校验和
     // 都与父指针一致、走读不断——最新根那一次各算一格；候选集只剩最新根时两条都还判得到（本地攻方腿：全称量词在单元素集合上照样成立）。
     let newest_txg = roots[newest_index].2.checkpoint_txg;
@@ -3570,6 +4094,13 @@
     // 再加一条：txg ≥ 最新根带的回退下界 F（D16（发布语义） 已定项 1 的回退候选集）；F 之下的根引用的单元可以已被回收复用，
     // 它们不在当前账里、也不再是「近 K 代」——I-2.1 只在候选集里的根上判。
     let instance_table_rows = walk.instance_table_rows.clone();
+    judge_rollback_witness_against_the_chosen_roots_table(
+        &rollback_witness,
+        newest_index_not_abandoned_by_the_witness.is_some(),
+        &roots[newest_index].2,
+        &instance_table_rows,
+        &mut walk.judgements,
+    );
     let newest_rollback_floor = u64::from_le_bytes(
         roots[newest_index].2.record_bytes[130..138]
             .try_into()
@@ -3582,9 +4113,10 @@
         .iter()
         .enumerate()
         .filter(|(index, (_, _, root))| {
+            // 被抛弃：按最新根指着的实例表判，或被回退见证表抛弃（D23（journal 的角色与格式） 已定项 14「回退见证」）。
             let abandoned = instance_table_rows.iter().any(|row| {
                 row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg
-            });
+            }) || abandoned_by_the_witness(root);
             let below_floor = root.checkpoint_txg < newest_rollback_floor;
             let walked = *index == newest_index || (!abandoned && !below_floor);
             if !walked {
@@ -3809,19 +4341,45 @@
     // I-3.1 / I-5.2：最新根下面记账树的「已分配」「空闲」逐盘对遍历得到的和与容量。
     // I-3.11：同一块盘上「已分配」减「defer 待释放」对只走最新根那一遍得到的和。
     let slots_referenced_by_every_walked_version = slots_referenced_per_device(&walk.references);
+    let start_slots_referenced_by_every_walked_version: BTreeSet<(u32, u64)> = walk
+        .references
+        .keys()
+        .map(|(device, start_slot, _)| (*device, *start_slot))
+        .collect();
+    // 隔离的记录（已分配而没有根引用、checker 自己读那一份也读不出或自证不过）在这两条上豁免：I-3.1 对全部走过的版本的引用、
+    // I-3.11 对最新根这一遍的引用各算一份（`quarantined_slots_exempted_per_device`）。
+    let newest_root_view = &roots[newest_index].2;
+    let quarantined_exempted_against_every_walked_version = quarantined_slots_exempted_per_device(
+        reader,
+        newest_root_view,
+        &start_slots_referenced_by_every_walked_version,
+        &mut index_node_cache,
+    );
+    let quarantined_exempted_against_the_newest_root = quarantined_slots_exempted_per_device(
+        reader,
+        newest_root_view,
+        &start_slots_referenced_by_the_newest_root,
+        &mut index_node_cache,
+    );
     if accounting_seen {
         for device in &devices {
-            let walked = slots_referenced_by_every_walked_version
+            let quarantined_exempted = quarantined_exempted_against_every_walked_version
                 .get(device)
                 .copied()
                 .unwrap_or(0)
                 * SLOT_BYTES;
+            let walked = slots_referenced_by_every_walked_version
+                .get(device)
+                .copied()
+                .unwrap_or(0)
+                * SLOT_BYTES
+                + quarantined_exempted;
             let allocated = accounting
                 .get(&(STATISTIC_ALLOCATED_BYTES, *device))
                 .copied();
             judgements.judge("I-3.1", allocated == Some(walked), || {
                 format!(
-                    "盘 {device}：记账的已分配 {allocated:?}，遍历全部有效根得到 {walked}{}",
+                    "盘 {device}：记账的已分配 {allocated:?}，遍历全部有效根得到 {walked}（其中隔离豁免 {quarantined_exempted}）{}",
                     mechanism()
                 )
             });
@@ -3837,16 +4395,22 @@
             let deferred = accounting
                 .get(&(STATISTIC_DEFER_QUEUE_BYTES, *device))
                 .copied();
-            let referenced_by_the_newest_root = slots_referenced_by_the_newest_root
+            let quarantined_exempted_newest = quarantined_exempted_against_the_newest_root
                 .get(device)
                 .copied()
                 .unwrap_or(0)
                 * SLOT_BYTES;
+            let referenced_by_the_newest_root = slots_referenced_by_the_newest_root
+                .get(device)
+                .copied()
+                .unwrap_or(0)
+                * SLOT_BYTES
+                + quarantined_exempted_newest;
             judgements.judge(
                 "I-3.11",
                 matches!((allocated, deferred), (Some(allocated), Some(deferred)) if deferred.checked_add(referenced_by_the_newest_root) == Some(allocated)),
                 || {
-                    format!("盘 {device}：记账的已分配 {allocated:?} 减 defer 待释放 {deferred:?}，不等于从最新根（txg {newest_txg}）走读到的 {referenced_by_the_newest_root}")
+                    format!("盘 {device}：记账的已分配 {allocated:?} 减 defer 待释放 {deferred:?}，不等于从最新根（txg {newest_txg}）走读到的 {referenced_by_the_newest_root}（其中隔离豁免 {quarantined_exempted_newest}）")
                 },
             );
         }
diff -ruN -x target tree/crates/singlefs-core/src/allocator.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-core/src/allocator.rs	2026-09-24 16:15:19.532697725 +0000
+++ tree/crates/singlefs-core/src/allocator.rs	2026-09-24 18:48:58.241408187 +0000
@@ -19,6 +19,7 @@
 use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
 use crate::bytes::ByteWriter;
 use crate::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
+use crate::transaction::FrozenPublish;
 
 /// 跨度段的最高位借作已释放标志（D3（空间分配） 已定项 7 / 已定项 11）：0 = 仍分配、1 = 已释放，不占表达跨度值的位。
 pub const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
@@ -745,6 +746,10 @@
     /// （`make_filesystem::root_ring_occupancy_after_make_filesystem`）。`None`：建分配器的一方没装（`new` 与 `rebuild_from_records`
     /// 都不装），挂载内回收与轮转清隔离位都不做，与只在挂载和抬 F 时回收的样子相同。
     root_ring: Option<RootRingOccupancy>,
+    /// 落盘中途失败、冻结着等原样重发的那一次发布（`transaction::FrozenPublish`，D23（journal 的角色与格式） 已定项 14
+    /// 「这一版的失败处置」）。住在分配器上是因为每一次发布都要交分配器进来：冻结着的时候发布路径在任何读写之前拒绝，
+    /// 只有 `transaction::resend_the_frozen_publish` 清得掉它。只住内存：重开之后走恢复、换实例代号，冻结的那一次不带过去。
+    frozen_publish: Option<Box<FrozenPublish>>,
 }
 
 impl PoolAllocator {
@@ -762,9 +767,33 @@
             reuse_window: ReuseWindow::GatedByTheRollbackFloor,
             placements_reclaimed_on_release_by_the_forced_zero_reuse_window: 0,
             root_ring: None,
+            frozen_publish: None,
         }
     }
 
+    /// 冻结着的那一次发布；没有时 `None`。
+    #[must_use]
+    pub fn frozen_publish(&self) -> Option<&FrozenPublish> {
+        self.frozen_publish.as_deref()
+    }
+
+    /// 把落盘中途失败的那一次发布冻结在这里（`transaction::persist_the_publish_or_freeze_it`）。
+    ///
+    /// # Panics
+    /// 已经冻结着一次：发布路径的第一道就拒（`transaction::refuse_while_a_publish_is_frozen`），冻结着的时候走不到落盘。
+    pub(crate) fn freeze_publish(&mut self, frozen: FrozenPublish) {
+        assert!(
+            self.frozen_publish.is_none(),
+            "冻结着一次发布时发布路径的第一道就拒，走不到第二次落盘失败"
+        );
+        self.frozen_publish = Some(Box::new(frozen));
+    }
+
+    /// 取走冻结着的那一次发布（重发时）。
+    pub(crate) fn take_frozen_publish(&mut self) -> Option<FrozenPublish> {
+        self.frozen_publish.take().map(|frozen| *frozen)
+    }
+
     /// 装上复用窗口这个只供测试的开关（`ReuseWindow`）。产品路径一处都不调它。
     pub fn set_reuse_window(&mut self, reuse_window: ReuseWindow) {
         self.reuse_window = reuse_window;
diff -ruN -x target tree/crates/singlefs-core/src/code_two_tree.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-core/src/code_two_tree.rs	1970-01-01 00:00:00.000000000 +0000
+++ tree/crates/singlefs-core/src/code_two_tree.rs	2026-09-24 16:50:54.714136181 +0000
@@ -0,0 +1,1366 @@
+//! 多层码 2 树的结构规则（D8（核心索引结构） 已定项 11「多层码 2 树怎么长、怎么收」）：记账树与中央映射树共用这一份，
+//! 不各写一套（已定项 14：一套 btree 实现管权威态的树与照分裂做的树）。
+//!
+//! 这里只管「这次发布之后树长什么样」：读上一版的形状（每个节点装哪些 key、父子关系）与这一版的 key 集合，
+//! 算出这一版每个节点装哪些 key、它照抄上一版哪个节点还是这次重写、上一版哪些节点被换下。只读——不碰分配器、不发写、
+//! 不装字节，交回拒绝时盘上逐字节不变。装字节、取落点在 `crate::transaction` 那一侧；从盘上读回一棵树在 [`read_code_two_tree`]。
+//!
+//! 规则逐条对应已定项 11：
+//! ① 分裂出来的节点与别的单元落在同一段、整条根到叶的路径照常 COW：这里只把路径上的节点标成「这次重写」，写序由发布路径统一走；
+//! ② 叶装不下时从中间切：插入之后条目数超过容量就切成两半，左半 ⌈n ÷ 2⌉ 条、右半其余；内部节点同一条；
+//! ③ 收缩只摘空节点，根只剩一个孩子时降高一层：删到空的节点从父节点里摘掉，不做「低于一半合并」与借条目；
+//!    根只剩一个孩子时孩子当根，孩子不重写；
+//! ④ 分隔 key 跟着维护：插到最左分隔 key 之下时把它压低；切出来的右半在父节点里的分隔 key 取它自己的最小 key；
+//! ⑤ 树高 = 根节点头里的层级 + 1（[`CodeTwoTreeShape::height`]；发布路径从根节点的字节现读，见 `crate::transaction`）。
+//!
+//! 节点的身份、层级、覆盖区间一律读块头自带的那几样，不另存旁路表：读回来的每个节点都按父条目核它的层级与区间
+//! （[`read_code_two_tree`]）。
+
+use std::collections::BTreeSet;
+
+use singlefs_format::NODE_POINTER_BYTES;
+
+use crate::address::{SlotNumber, TreeIdentifier};
+use crate::bytes::{ByteReader, ByteWriter};
+use crate::pointer::NodePointer;
+use crate::recovery::RecoveryFailure;
+use crate::root_record::RootRecord;
+use crate::unit::{index_node_entry_capacity, parse_index_node, IndexNodeHeader};
+
+/// 一棵码 2 树的 key 怎么比：各字段的字节宽，盘上小端；比较时逐字段按无符号整数、自左向右（D8（核心索引结构） 已定项 11），
+/// 小端存储不构成 memcmp 序。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CodeTwoKeyFieldWidths(&'static [usize]);
+
+impl CodeTwoKeyFieldWidths {
+    /// 记账条目的 key 22：统计量标签 2 + 树 ID 8 + 设备 4 + 代 8（D5（快照 / 空间记账机制） 已定项 5）。
+    pub const ACCOUNTING: Self = Self(&[2, 8, 4, 8]);
+    /// 中央映射 key 27：类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 尾段 6（D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10：
+    /// 码 1 的尾段是事务号低 48 位；码 2 / 码 3 是出生序号 4 + 补零 2，按 6 字节整数比与按出生序号比同序）。
+    pub const CENTRAL_MAPPING: Self = Self(&[1, 8, 8, 4, 6]);
+
+    /// key 宽：各字段宽之和。
+    #[must_use]
+    pub fn key_width_in_bytes(self) -> usize {
+        self.0.iter().sum()
+    }
+
+    fn fields_in_comparison_order(self, key_bytes: &[u8]) -> Vec<u64> {
+        assert_eq!(
+            key_bytes.len(),
+            self.key_width_in_bytes(),
+            "key 的字节数等于这棵树的 key 宽：调用方从条目里切前 key 宽 个字节（D8 已定项 11）"
+        );
+        let mut offset_in_bytes = 0;
+        self.0
+            .iter()
+            .map(|field_width_in_bytes| {
+                let mut little_endian = [0u8; 8];
+                little_endian[..*field_width_in_bytes].copy_from_slice(
+                    &key_bytes[offset_in_bytes..offset_in_bytes + field_width_in_bytes],
+                );
+                offset_in_bytes += field_width_in_bytes;
+                u64::from_le_bytes(little_endian)
+            })
+            .collect()
+    }
+}
+
+/// 一把码 2 key：盘上的字节与按字段读出来的无符号整数。全序只看后者（[`CodeTwoKeyFieldWidths`]）；
+/// 同一棵树里字段相等的两把 key 字节也相等，所以派生的全序先比字段、再比字节，与只比字段同序。
+#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub struct CodeTwoTreeKey {
+    fields_in_comparison_order: Vec<u64>,
+    bytes: Vec<u8>,
+}
+
+impl CodeTwoTreeKey {
+    /// # Panics
+    /// `key_bytes` 的长度不是这棵树的 key 宽：调用方按 key 宽切出来的，不等说明切错了。
+    #[must_use]
+    pub fn new(key_bytes: &[u8], field_widths: CodeTwoKeyFieldWidths) -> Self {
+        Self {
+            fields_in_comparison_order: field_widths.fields_in_comparison_order(key_bytes),
+            bytes: key_bytes.to_vec(),
+        }
+    }
+
+    #[must_use]
+    pub fn bytes(&self) -> &[u8] {
+        &self.bytes
+    }
+}
+
+/// 一个码 2 节点最多装几条条目：叶按这棵树的叶条目宽算，内部节点按内部条目宽算（D8（核心索引结构） 已定项 11：
+/// 内部节点的条目 = 本树 key + 子指针 86；记账树 108、中央映射树 113）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CodeTwoTreeNodeCapacity {
+    pub leaf_entries: usize,
+    pub internal_entries: usize,
+}
+
+impl CodeTwoTreeNodeCapacity {
+    /// 格式算出来的容量：(16384 − 含预留位的头) ÷ 条目宽（`unit::index_node_entry_capacity`）。
+    #[must_use]
+    pub fn of_the_node_format(key_width_in_bytes: usize, leaf_entry_width_in_bytes: usize) -> Self {
+        Self {
+            leaf_entries: index_node_entry_capacity(key_width_in_bytes, leaf_entry_width_in_bytes),
+            internal_entries: index_node_entry_capacity(
+                key_width_in_bytes,
+                internal_entry_width_in_bytes(key_width_in_bytes),
+            ),
+        }
+    }
+}
+
+/// 内部节点的条目宽：本树 key + 子指针 86（D8（核心索引结构） 已定项 11）。
+#[must_use]
+pub fn internal_entry_width_in_bytes(key_width_in_bytes: usize) -> usize {
+    key_width_in_bytes + usize::try_from(NODE_POINTER_BYTES).expect("86")
+}
+
+/// 一条内部节点条目：分隔 key 打头（D8（核心索引结构） 已定项 11「条目里 key 一律是条目的前 key 宽 个字节」），后跟指向孩子的节点指针。
+#[must_use]
+pub fn build_internal_entry(separator_key: &[u8], child: NodePointer) -> Vec<u8> {
+    let mut writer = ByteWriter::new(internal_entry_width_in_bytes(separator_key.len()));
+    writer.put(separator_key);
+    child.write_to(&mut writer);
+    writer.into_bytes()
+}
+
+/// 解一条内部节点条目：条目宽是节点头里的**盘上字段**，窄于 key 宽 + 86 时交回 `None`（调用方报「条目窄于字段表」），
+/// 这里是盘上字节进字段表的边界。
+#[must_use]
+pub fn parse_internal_entry(
+    entry: &[u8],
+    key_width_in_bytes: usize,
+) -> Option<(Vec<u8>, NodePointer)> {
+    if entry.len() < internal_entry_width_in_bytes(key_width_in_bytes) {
+        return None;
+    }
+    let mut reader = ByteReader::at(entry, key_width_in_bytes);
+    Some((
+        entry[..key_width_in_bytes].to_vec(),
+        NodePointer::read_from(&mut reader),
+    ))
+}
+
+/// 一个码 2 节点在一版树里的位置：层级（0 是叶）与同层从左数第几个（按 key 升序）。派生的全序（层级升序、同层按 key 升序）
+/// 就是这棵树在一次发布里的 bump 次序与出生序号的发号次序：树内先叶后根、同层按 key 升序（D3（空间分配） 已定项 10 ⑤、
+/// D19（块指针的结构与宽度预算） 已定项 9），根在最高那一层、排最末。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub struct CodeTwoTreeNodePosition {
+    pub level: u8,
+    pub index_in_level: u32,
+}
+
+/// 一个内部节点的一条条目在形状里的样子：分隔 key 与它指着的孩子。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct CodeTwoTreeChild {
+    pub separator_key: CodeTwoTreeKey,
+    pub child: CodeTwoTreeNodePosition,
+}
+
+/// 一个节点装的东西：叶装条目的 key（按 key 升序），内部节点装孩子（按分隔 key 升序）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum CodeTwoTreeNodeContents {
+    Leaf { keys: Vec<CodeTwoTreeKey> },
+    Internal { children: Vec<CodeTwoTreeChild> },
+}
+
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct CodeTwoTreeShapeNode {
+    pub position: CodeTwoTreeNodePosition,
+    pub contents: CodeTwoTreeNodeContents,
+}
+
+/// 一版树的形状：节点按 bump 次序（[`CodeTwoTreeNodePosition`] 的全序），根在最末。还没建过的树没有节点。
+#[derive(Clone, Debug, Default, PartialEq, Eq)]
+pub struct CodeTwoTreeShape {
+    nodes: Vec<CodeTwoTreeShapeNode>,
+}
+
+impl CodeTwoTreeShape {
+    #[must_use]
+    pub fn nodes(&self) -> &[CodeTwoTreeShapeNode] {
+        &self.nodes
+    }
+
+    /// 根：bump 次序里的最后一个节点。还没建过的树交回 `None`。
+    #[must_use]
+    pub fn root(&self) -> Option<&CodeTwoTreeShapeNode> {
+        self.nodes.last()
+    }
+
+    /// 树高 = 根的层级 + 1（D8（核心索引结构） 已定项 11 ⑤）；还没建过的树是 0。
+    #[must_use]
+    pub fn height(&self) -> u64 {
+        self.root()
+            .map_or(0, |root| u64::from(root.position.level) + 1)
+    }
+
+    #[must_use]
+    pub fn node(&self, position: CodeTwoTreeNodePosition) -> Option<&CodeTwoTreeShapeNode> {
+        self.nodes
+            .binary_search_by_key(&position, |node| node.position)
+            .ok()
+            .map(|index| &self.nodes[index])
+    }
+
+    /// 这个位置是不是根。
+    #[must_use]
+    pub fn is_root(&self, position: CodeTwoTreeNodePosition) -> bool {
+        self.root().is_some_and(|root| root.position == position)
+    }
+
+    /// 叶里的全部 key，从左到右。
+    #[must_use]
+    pub fn keys_in_order(&self) -> Vec<&CodeTwoTreeKey> {
+        self.nodes
+            .iter()
+            .filter_map(|node| match &node.contents {
+                CodeTwoTreeNodeContents::Leaf { keys } => Some(keys.iter()),
+                CodeTwoTreeNodeContents::Internal { .. } => None,
+            })
+            .flatten()
+            .collect()
+    }
+
+    /// 一个节点的子树里最小与最大的 key（D18（块里携带什么信息） 已定项 2：码 2 头的 key 区间取子树覆盖区间）。
+    /// 空的叶（还没装过条目的根）交回 `None`。
+    ///
+    /// # Panics
+    /// 形状里的子引用指到了不存在的位置：形状由 [`plan_the_tree_after_this_publish`] 或 [`read_code_two_tree`] 造出来，
+    /// 两处都按层级逐层建引用，指不到说明造它的那一方写错了。
+    #[must_use]
+    pub fn key_range_of_the_subtree(
+        &self,
+        position: CodeTwoTreeNodePosition,
+    ) -> Option<(&CodeTwoTreeKey, &CodeTwoTreeKey)> {
+        let mut leftmost = position;
+        let smallest = loop {
+            match &self
+                .node(leftmost)
+                .expect("子引用指着形状里的节点")
+                .contents
+            {
+                CodeTwoTreeNodeContents::Leaf { keys } => break keys.first()?,
+                CodeTwoTreeNodeContents::Internal { children } => {
+                    leftmost = children.first()?.child;
+                }
+            }
+        };
+        let mut rightmost = position;
+        let largest = loop {
+            match &self
+                .node(rightmost)
+                .expect("子引用指着形状里的节点")
+                .contents
+            {
+                CodeTwoTreeNodeContents::Leaf { keys } => break keys.last()?,
+                CodeTwoTreeNodeContents::Internal { children } => {
+                    rightmost = children.last()?.child;
+                }
+            }
+        };
+        Some((smallest, largest))
+    }
+}
+
+/// 这一版的一个节点从哪来：照抄上一版的某个节点（字节、落点、指针一个不动），或这次重写。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CodeTwoTreeNodeOrigin {
+    CarriedFrom(CodeTwoTreeNodePosition),
+    RewrittenThisPublish,
+}
+
+/// 这次发布之后树长什么样（[`plan_the_tree_after_this_publish`] 交回）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct CodeTwoTreePlan {
+    pub shape: CodeTwoTreeShape,
+    /// 与 `shape.nodes()` 同序。
+    pub origins: Vec<CodeTwoTreeNodeOrigin>,
+    /// 上一版里这次被换下的节点（没被照抄进这一版的），按上一版的 bump 次序。
+    pub replaced_previous_nodes: Vec<CodeTwoTreeNodePosition>,
+}
+
+impl CodeTwoTreePlan {
+    /// 这次重写的节点，按 bump 次序（= 这棵树里出生序号的发号次序）。
+    #[must_use]
+    pub fn rewritten_positions(&self) -> Vec<CodeTwoTreeNodePosition> {
+        self.shape
+            .nodes()
+            .iter()
+            .zip(&self.origins)
+            .filter(|(_, origin)| **origin == CodeTwoTreeNodeOrigin::RewrittenThisPublish)
+            .map(|(node, _)| node.position)
+            .collect()
+    }
+
+    /// 这个位置上的节点从哪来。
+    ///
+    /// # Panics
+    /// 这个位置不在这一版里：调用方从同一份计划里取的位置。
+    #[must_use]
+    pub fn origin_of(&self, position: CodeTwoTreeNodePosition) -> CodeTwoTreeNodeOrigin {
+        let index = self
+            .shape
+            .nodes()
+            .binary_search_by_key(&position, |node| node.position)
+            .expect("位置取自同一份计划");
+        self.origins[index]
+    }
+}
+
+/// 这次之后的树算不出来，在任何落盘动作之前交回。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CodeTwoTreeRefusal {
+    /// 这次之后树要长到 257 层：码 2 头里的层级是 1 字节（D18（块里携带什么信息） 已定项 18 偏移 50），写不下。
+    /// 这是多层码 2 树唯一的结构上限（D19（块指针的结构与宽度预算） 已定项 5 的映射树容量准入在多层之后就是它）。
+    HeightBeyondTheLevelField,
+    /// 上一版的形状里，按分隔 key 从根往下走，走不到装着某把 key 的那片叶：形状是从盘上读来的（重建出来的上一版），
+    /// 分隔 key 与孩子的区间对不上时删不掉它。读回来的形状按父条目核过区间（[`read_code_two_tree`]），走到这里说明核漏了；
+    /// 不断言，交回让这次发布整次不做。
+    PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt,
+}
+
+/// 规划中的一个节点：`carried_from` 是 `Some` ⟺ 从上一版那个位置照抄、这次没被碰过。
+#[derive(Clone, Debug)]
+enum PlanningNode {
+    Leaf {
+        keys: Vec<CodeTwoTreeKey>,
+        carried_from: Option<CodeTwoTreeNodePosition>,
+    },
+    Internal {
+        level: u8,
+        children: Vec<PlanningChild>,
+        carried_from: Option<CodeTwoTreeNodePosition>,
+    },
+}
+
+#[derive(Clone, Debug)]
+struct PlanningChild {
+    separator_key: CodeTwoTreeKey,
+    node: PlanningNode,
+}
+
+impl PlanningNode {
+    fn empty_leaf() -> Self {
+        PlanningNode::Leaf {
+            keys: Vec::new(),
+            carried_from: None,
+        }
+    }
+
+    fn level(&self) -> u8 {
+        match self {
+            PlanningNode::Leaf { .. } => 0,
+            PlanningNode::Internal { level, .. } => *level,
+        }
+    }
+
+    fn entry_count(&self) -> usize {
+        match self {
+            PlanningNode::Leaf { keys, .. } => keys.len(),
+            PlanningNode::Internal { children, .. } => children.len(),
+        }
+    }
+
+    /// 这次碰过它：路径上的节点整条 COW（D8（核心索引结构） 已定项 11 ①）。
+    fn mark_rewritten(&mut self) {
+        match self {
+            PlanningNode::Leaf { carried_from, .. }
+            | PlanningNode::Internal { carried_from, .. } => *carried_from = None,
+        }
+    }
+
+    fn smallest_key(&self) -> Option<&CodeTwoTreeKey> {
+        match self {
+            PlanningNode::Leaf { keys, .. } => keys.first(),
+            PlanningNode::Internal { children, .. } => children.first()?.node.smallest_key(),
+        }
+    }
+}
+
+/// 从上一版的形状建规划树：每个节点都标成「照抄自原位置」。
+fn planning_node_of(shape: &CodeTwoTreeShape, position: CodeTwoTreeNodePosition) -> PlanningNode {
+    match &shape
+        .node(position)
+        .expect("形状里的子引用指着形状里的节点（造形状的一方逐层建引用）")
+        .contents
+    {
+        CodeTwoTreeNodeContents::Leaf { keys } => PlanningNode::Leaf {
+            keys: keys.clone(),
+            carried_from: Some(position),
+        },
+        CodeTwoTreeNodeContents::Internal { children } => PlanningNode::Internal {
+            level: position.level,
+            children: children
+                .iter()
+                .map(|child| PlanningChild {
+                    separator_key: child.separator_key.clone(),
+                    node: planning_node_of(shape, child.child),
+                })
+                .collect(),
+            carried_from: Some(position),
+        },
+    }
+}
+
+/// 按分隔 key 找孩子（插入用）：最后一个分隔 key ≤ key 的那一个；key 比第一个分隔 key 还小时落到第 0 个孩子，
+/// 并把第 0 个分隔 key 压低到 key（D8（核心索引结构） 已定项 11 ④）。
+fn route_for_insertion(children: &mut [PlanningChild], key: &CodeTwoTreeKey) -> usize {
+    match children
+        .iter()
+        .rposition(|child| child.separator_key <= *key)
+    {
+        Some(index) => index,
+        None => {
+            children[0].separator_key = key.clone();
+            0
+        }
+    }
+}
+
+/// 按分隔 key 找孩子（删除用）：同上，不改分隔 key。
+fn route_for_deletion(children: &[PlanningChild], key: &CodeTwoTreeKey) -> usize {
+    children
+        .iter()
+        .rposition(|child| child.separator_key <= *key)
+        .unwrap_or(0)
+}
+
+/// 从中间切：左半留 ⌈n ÷ 2⌉ 条，交回右半与它在父节点里的分隔 key（它自己的最小 key）。
+fn split_in_the_middle(node: &mut PlanningNode) -> PlanningChild {
+    match node {
+        PlanningNode::Leaf { keys, .. } => {
+            let right_keys = keys.split_off(keys.len().div_ceil(2));
+            PlanningChild {
+                separator_key: right_keys[0].clone(),
+                node: PlanningNode::Leaf {
+                    keys: right_keys,
+                    carried_from: None,
+                },
+            }
+        }
+        PlanningNode::Internal {
+            level, children, ..
+        } => {
+            let right_children = children.split_off(children.len().div_ceil(2));
+            PlanningChild {
+                separator_key: right_children[0].separator_key.clone(),
+                node: PlanningNode::Internal {
+                    level: *level,
+                    children: right_children,
+                    carried_from: None,
+                },
+            }
+        }
+    }
+}
+
+/// 插一把 key；节点装不下时切开，交回右半（由父节点接上）。路径上每个节点都标成重写。
+fn insert_below(
+    node: &mut PlanningNode,
+    key: CodeTwoTreeKey,
+    capacity: CodeTwoTreeNodeCapacity,
+) -> Option<PlanningChild> {
+    node.mark_rewritten();
+    match node {
+        PlanningNode::Leaf { keys, .. } => {
+            let insertion_index = keys.partition_point(|existing| *existing < key);
+            assert!(
+                keys.get(insertion_index) != Some(&key),
+                "插入的 key 不在树里：插入集合 = 这一版的 key 减上一版的 key"
+            );
+            keys.insert(insertion_index, key);
+            if keys.len() > capacity.leaf_entries {
+                Some(split_in_the_middle(node))
+            } else {
+                None
+            }
+        }
+        PlanningNode::Internal { children, .. } => {
+            let child_index = route_for_insertion(children, &key);
+            if let Some(right_half) = insert_below(&mut children[child_index].node, key, capacity) {
+                children.insert(child_index + 1, right_half);
+            }
+            if children.len() > capacity.internal_entries {
+                Some(split_in_the_middle(node))
+            } else {
+                None
+            }
+        }
+    }
+}
+
+/// 在根上插一把 key：根装不下时切开、长出新根，树高 + 1。
+fn insert_at_the_root(
+    root: &mut PlanningNode,
+    key: CodeTwoTreeKey,
+    capacity: CodeTwoTreeNodeCapacity,
+) -> Result<(), CodeTwoTreeRefusal> {
+    let Some(right_half) = insert_below(root, key, capacity) else {
+        return Ok(());
+    };
+    let new_root_level = root
+        .level()
+        .checked_add(1)
+        .ok_or(CodeTwoTreeRefusal::HeightBeyondTheLevelField)?;
+    let left_half = std::mem::replace(root, PlanningNode::empty_leaf());
+    let left_separator = left_half
+        .smallest_key()
+        .expect("刚装满、切开过的节点左半不空")
+        .clone();
+    *root = PlanningNode::Internal {
+        level: new_root_level,
+        children: vec![
+            PlanningChild {
+                separator_key: left_separator,
+                node: left_half,
+            },
+            right_half,
+        ],
+        carried_from: None,
+    };
+    Ok(())
+}
+
+/// 删一把 key；删空的孩子从父节点里摘掉（只摘空节点，D8（核心索引结构） 已定项 11 ③）。交回找没找到。
+fn delete_below(node: &mut PlanningNode, key: &CodeTwoTreeKey) -> bool {
+    match node {
+        PlanningNode::Leaf { keys, .. } => match keys.binary_search(key) {
+            Ok(index) => {
+                keys.remove(index);
+                node.mark_rewritten();
+                true
+            }
+            Err(_) => false,
+        },
+        PlanningNode::Internal { children, .. } => {
+            let child_index = route_for_deletion(children, key);
+            if !delete_below(&mut children[child_index].node, key) {
+                return false;
+            }
+            if children[child_index].node.entry_count() == 0 {
+                children.remove(child_index);
+            }
+            node.mark_rewritten();
+            true
+        }
+    }
+}
+
+/// 根上删一把 key，删完之后根空了就变回空叶，根只剩一个孩子就让孩子当根（降高一层，孩子不重写），直到不再成立。
+fn delete_at_the_root(
+    root: &mut PlanningNode,
+    key: &CodeTwoTreeKey,
+) -> Result<(), CodeTwoTreeRefusal> {
+    if !delete_below(root, key) {
+        return Err(CodeTwoTreeRefusal::PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt);
+    }
+    // 迭代上界：每一轮树高减一，至多根的层级那么多轮。
+    loop {
+        let only_child = match root {
+            PlanningNode::Internal { children, .. } if children.is_empty() => {
+                *root = PlanningNode::empty_leaf();
+                return Ok(());
+            }
+            PlanningNode::Internal { children, .. } if children.len() == 1 => {
+                children.pop().expect("刚判过恰一个孩子").node
+            }
+            PlanningNode::Internal { .. } | PlanningNode::Leaf { .. } => return Ok(()),
+        };
+        *root = only_child;
+    }
+}
+
+/// 把规划树按层摊平：后序、从左到右，每一层里的节点就按 key 升序排好（同层左边的子树先走完）。
+fn flatten(
+    node: PlanningNode,
+    nodes_by_level: &mut Vec<Vec<(CodeTwoTreeNodeContents, CodeTwoTreeNodeOrigin)>>,
+) -> CodeTwoTreeNodePosition {
+    let level = node.level();
+    let (contents, carried_from) = match node {
+        PlanningNode::Leaf { keys, carried_from } => {
+            (CodeTwoTreeNodeContents::Leaf { keys }, carried_from)
+        }
+        PlanningNode::Internal {
+            children,
+            carried_from,
+            ..
+        } => {
+            let children = children
+                .into_iter()
+                .map(|child| CodeTwoTreeChild {
+                    separator_key: child.separator_key,
+                    child: flatten(child.node, nodes_by_level),
+                })
+                .collect();
+            (CodeTwoTreeNodeContents::Internal { children }, carried_from)
+        }
+    };
+    let level_index = usize::from(level);
+    if nodes_by_level.len() <= level_index {
+        nodes_by_level.resize_with(level_index + 1, Vec::new);
+    }
+    let index_in_level = u32::try_from(nodes_by_level[level_index].len())
+        .expect("一层的节点数装得进 u32：条目数有上界");
+    nodes_by_level[level_index].push((
+        contents,
+        match carried_from {
+            Some(previous_position) => CodeTwoTreeNodeOrigin::CarriedFrom(previous_position),
+            None => CodeTwoTreeNodeOrigin::RewrittenThisPublish,
+        },
+    ));
+    CodeTwoTreeNodePosition {
+        level,
+        index_in_level,
+    }
+}
+
+/// 这次发布之后树长什么样：上一版有、这一版没有的 key 先删（按 key 升序），这一版有、上一版没有的 key 再插（按 key 升序）。
+/// 先删后插：一次发布里同一个 key 的中间状态不落叶（D8（核心索引结构） 已定项 13），这里只决定这一版的形状；
+/// 先删让删空的节点先摘掉、新 key 再按分隔 key 找落处，树不因一次发布里的换号而先长后缩。
+///
+/// # Errors
+/// 这次之后树要长到 257 层 ⇒ `HeightBeyondTheLevelField`；上一版的形状按分隔 key 走不到它自己叶里的 key ⇒
+/// `PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt`。两样都在任何落盘动作之前交回。
+///
+/// # Panics
+/// 这一版一把 key 都没有：记账树每次发布恒有池级 3 行，中央映射树恒有分配记录树、记账树与 inode 树根的条目，
+/// 两棵树都不会删到空（调用方给的是这两棵树）。
+pub fn plan_the_tree_after_this_publish(
+    previous: &CodeTwoTreeShape,
+    keys_after_this_publish: &BTreeSet<CodeTwoTreeKey>,
+    capacity: CodeTwoTreeNodeCapacity,
+) -> Result<CodeTwoTreePlan, CodeTwoTreeRefusal> {
+    assert!(
+        !keys_after_this_publish.is_empty(),
+        "记账树恒有池级 3 行、中央映射树恒有分配记录树、记账树与 inode 树根的条目：这一版的 key 不会是空集"
+    );
+    assert!(
+        capacity.leaf_entries >= 1 && capacity.internal_entries >= 2,
+        "叶至少装 1 条、内部节点至少装 2 个孩子：切出来的两半才都不空、根分裂才装得下两个孩子"
+    );
+    let previous_keys: BTreeSet<CodeTwoTreeKey> =
+        previous.keys_in_order().into_iter().cloned().collect();
+    let mut root = match previous.root() {
+        Some(previous_root) => planning_node_of(previous, previous_root.position),
+        None => PlanningNode::empty_leaf(),
+    };
+    for deleted in previous_keys.difference(keys_after_this_publish) {
+        delete_at_the_root(&mut root, deleted)?;
+    }
+    for inserted in keys_after_this_publish.difference(&previous_keys) {
+        insert_at_the_root(&mut root, inserted.clone(), capacity)?;
+    }
+    let mut nodes_by_level = Vec::new();
+    flatten(root, &mut nodes_by_level);
+    let mut nodes = Vec::new();
+    let mut origins = Vec::new();
+    for (level, nodes_of_this_level) in nodes_by_level.into_iter().enumerate() {
+        for (index_in_level, (contents, origin)) in nodes_of_this_level.into_iter().enumerate() {
+            nodes.push(CodeTwoTreeShapeNode {
+                position: CodeTwoTreeNodePosition {
+                    level: u8::try_from(level).expect("层级是 1 字节：根分裂时判过"),
+                    index_in_level: u32::try_from(index_in_level).expect("摊平时判过"),
+                },
+                contents,
+            });
+            origins.push(origin);
+        }
+    }
+    let carried: BTreeSet<CodeTwoTreeNodePosition> = origins
+        .iter()
+        .filter_map(|origin| match origin {
+            CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => Some(*previous_position),
+            CodeTwoTreeNodeOrigin::RewrittenThisPublish => None,
+        })
+        .collect();
+    let replaced_previous_nodes = previous
+        .nodes()
+        .iter()
+        .map(|node| node.position)
+        .filter(|position| !carried.contains(position))
+        .collect();
+    Ok(CodeTwoTreePlan {
+        shape: CodeTwoTreeShape { nodes },
+        origins,
+        replaced_previous_nodes,
+    })
+}
+
+/// 一版树在内存里的样子：形状加每个节点这一版的指针（与 `shape.nodes()` 同序）。节点的字节住
+/// `TransactionOutput::units` 里它那个角色的单元（`transaction::TransactionUnit` 的记账树 / 中央映射树两族）。
+#[derive(Clone, Debug, Default, PartialEq, Eq)]
+pub struct CodeTwoTreeVersion {
+    pub shape: CodeTwoTreeShape,
+    pub pointers: Vec<NodePointer>,
+}
+
+impl CodeTwoTreeVersion {
+    /// 这个位置上的节点这一版的指针。
+    ///
+    /// # Panics
+    /// 位置不在这一版里：调用方从同一份形状里取的位置。
+    #[must_use]
+    pub fn pointer_of(&self, position: CodeTwoTreeNodePosition) -> NodePointer {
+        let index = self
+            .shape
+            .nodes()
+            .binary_search_by_key(&position, |node| node.position)
+            .expect("位置取自同一份形状");
+        self.pointers[index]
+    }
+
+    /// 根的指针（树表条目或根记录里写的那一条）。
+    ///
+    /// # Panics
+    /// 树还没建过：带文件的一版里记账树与中央映射树恒有根。
+    #[must_use]
+    pub fn root_pointer(&self) -> NodePointer {
+        *self
+            .pointers
+            .last()
+            .expect("带文件的一版里记账树与中央映射树恒有根")
+    }
+
+    /// 节点数。
+    #[must_use]
+    pub fn node_count(&self) -> usize {
+        self.shape.nodes().len()
+    }
+}
+
+/// 从盘上读一棵多层码 2 树时对它的期望：哪棵树、key 怎么比，内部条目窄于 key 宽 + 86 时报错叫它什么。
+#[derive(Clone, Copy, Debug)]
+pub struct CodeTwoTreeReadExpectation {
+    pub tree: TreeIdentifier,
+    pub field_widths: CodeTwoKeyFieldWidths,
+    pub internal_entry_name: &'static str,
+}
+
+/// 从盘上读回来的一棵多层码 2 树：形状、每个节点的指针与字节（与 `version.shape.nodes()` 同序），与叶里的全部条目（按 key 升序）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct CodeTwoTreeReadFromDisk {
+    pub version: CodeTwoTreeVersion,
+    pub node_bytes: Vec<Vec<u8>>,
+    pub leaf_entries_in_key_order: Vec<Vec<u8>>,
+}
+
+/// 读回来还没摊平的一个节点。
+struct NodeReadFromDisk {
+    pointer: NodePointer,
+    bytes: Vec<u8>,
+    header: IndexNodeHeader,
+    children: Vec<(CodeTwoTreeKey, NodeReadFromDisk)>,
+}
+
+/// 从盘上读一棵多层码 2 树时核到哪一步。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CodeTwoTreeHeaderJudgement {
+    /// 冷启动走读与挂载态打开：每个节点的头都按指着它的指针与父条目核（树 ID、key 宽、出生身份、fsid、层级、区间、分隔 key），
+    /// 与走读核别的树根同一套口径、同一组判定名（根那一个节点的报错逐字沿用 `recovery` 核树根的那几句）。
+    EveryHeaderAgainstItsReference,
+    /// 从盘上重建上一版（`recovery::rebuild_version`）：只核把节点拼成一棵树非核不可的那几样——层级逐层减一、节点不空、
+    /// 内部条目不窄于 key 宽 + 86、同一个节点不被两条父条目引用；头里的出生身份、区间与分隔 key 不核，与重建路径读别的树同一个口径
+    /// （解得开就收，叶条目窄于字段表由解条目的一方报）。分隔 key 与孩子对不上时，下一次发布按它删 key 删不掉，由规划那一步交回
+    /// `CodeTwoTreeRefusal::PreviousShapeRoutesAKeyAwayFromTheLeafHoldingIt`。
+    OnlyWhatTheShapeNeeds,
+}
+
+fn violated(detail: &'static str) -> RecoveryFailure {
+    RecoveryFailure::InvariantViolated {
+        invariant: "I-1.1",
+        detail,
+    }
+}
+
+/// 核一个节点的头：只有 `EveryHeaderAgainstItsReference` 才走到这里。根（`is_root`）的报错逐字沿用 `recovery` 核树根的那几句，
+/// 读路径上几种坏法报的成员因此不随树长没长多层而变。
+fn judge_the_header_against_its_reference(
+    header: &IndexNodeHeader,
+    pointer: &NodePointer,
+    expectation: &CodeTwoTreeReadExpectation,
+    root: &RootRecord,
+    expected_filesystem_identifier: u64,
+    is_root: bool,
+) -> Result<(), RecoveryFailure> {
+    let detail = |of_the_root: &'static str, of_a_node_below_the_root: &'static str| {
+        if is_root {
+            of_the_root
+        } else {
+            of_a_node_below_the_root
+        }
+    };
+    if header.tree != expectation.tree {
+        return Err(RecoveryFailure::InvariantViolated {
+            invariant: "I-1.3",
+            detail: detail(
+                "根头里的树 ID 与树表不符",
+                "码 2 节点头里的树 ID 与引用它的树不符",
+            ),
+        });
+    }
+    if header.key_width != expectation.field_widths.key_width_in_bytes() {
+        return Err(RecoveryFailure::InvariantViolated {
+            invariant: "E142 走读同款",
+            detail: detail(
+                "根自述 key 宽与树的种类不符",
+                "码 2 节点自述 key 宽与树的种类不符",
+            ),
+        });
+    }
+    if header.birth_txg > root.checkpoint_txg || header.instance > root.instance {
+        return Err(RecoveryFailure::InvariantViolated {
+            invariant: "I-1.2",
+            detail: detail("树根诞生于根之后", "码 2 节点诞生于根之后"),
+        });
+    }
+    if header.filesystem_identifier != expected_filesystem_identifier {
+        return Err(RecoveryFailure::InvariantViolated {
+            invariant: "I-1.4",
+            detail: detail("树根 fsid 不符", "码 2 节点 fsid 不符"),
+        });
+    }
+    if header.birth_sequence != pointer.birth_sequence {
+        return Err(RecoveryFailure::InvariantViolated {
+            invariant: "I-1.2",
+            detail: detail("树根出生序号与指针不符", "码 2 节点出生序号与指针不符"),
+        });
+    }
+    if header.level == 0 {
+        let key_width = expectation.field_widths.key_width_in_bytes();
+        let first_and_last_hold = match (header.entries.first(), header.entries.last()) {
+            (Some(first), Some(last)) => {
+                first[..key_width] == header.smallest_key[..]
+                    && last[..key_width] == header.largest_key[..]
+            }
+            (None, _) | (_, None) => true,
+        };
+        if !first_and_last_hold {
+            return Err(violated(detail(
+                "根 key 区间与条目不符",
+                "叶的 key 区间不是首末两条条目的 key",
+            )));
+        }
+    }
+    Ok(())
+}
+
+/// 从根指针往下读一棵多层码 2 树（记账树、中央映射树），节点的身份、层级、覆盖区间只信它自己头里写的那几样，逐个按父条目核
+/// （核到哪一步看 `judgement`，[`CodeTwoTreeHeaderJudgement`]）：
+/// - 头：树 ID 是这棵树（I-1.3）、key 宽是这棵树的、fsid 是本池（I-1.4）、诞生不晚于根、出生序号与指着它的指针相同（I-1.2）；
+/// - 层级：孩子 = 父 − 1（I-1.1：层级是索引节点身份的一段），层级 0 是叶；
+/// - 区间：叶的 key 区间是首末两条条目的 key；内部节点的是子树覆盖区间 [第一个孩子的最小 key, 最后一个孩子的最大 key]
+///   （D18（块里携带什么信息） 已定项 2）；第 i 个孩子的区间落在父条目给的那一段里：分隔 key_i ≤ 孩子头里的最小 key，
+///   且孩子头里的最大 key < 分隔 key_{i+1}（D8（核心索引结构） 已定项 11 ④ 的两条不等式）；条目按 key 严格递增；
+/// - 两种核法都判：节点不空、同一个单元不被两条父条目引用（走成 DAG 就按层级指数放大读量）。
+///
+/// `read_node` 按指针读一个节点的字节：不进映射的树（中央映射树）只按位置条目读，进映射的树（记账树）走读时提示读不出经映射回退。
+///
+/// # Errors
+/// 节点读不到、解不开 ⇒ 读者交回的错或 `UnitMalformed`；内部条目窄于 key 宽 + 86 ⇒ `EntryNarrowerThanItsFieldTable`；
+/// 上面任一条核不过 ⇒ `InvariantViolated`（I-1.1 / I-1.2 / I-1.3 / I-1.4 / E142 走读同款）。
+pub(crate) fn read_code_two_tree(
+    root_pointer: &NodePointer,
+    expectation: &CodeTwoTreeReadExpectation,
+    judgement: CodeTwoTreeHeaderJudgement,
+    root: &RootRecord,
+    expected_filesystem_identifier: u64,
+    read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
+) -> Result<CodeTwoTreeReadFromDisk, RecoveryFailure> {
+    let mut units_seen: BTreeSet<SlotNumber> = BTreeSet::new();
+    let reading = TreeReading {
+        expectation,
+        judgement,
+        root,
+        expected_filesystem_identifier,
+    };
+    let root_node =
+        reading.read_node_and_its_subtree(root_pointer, None, read_node, &mut units_seen)?;
+    let mut nodes_by_level: Vec<Vec<(CodeTwoTreeNodeContents, NodePointer, Vec<u8>)>> = Vec::new();
+    let mut leaf_entries_in_key_order = Vec::new();
+    flatten_read(
+        root_node,
+        expectation.field_widths,
+        &mut nodes_by_level,
+        &mut leaf_entries_in_key_order,
+    );
+    let mut nodes = Vec::new();
+    let mut pointers = Vec::new();
+    let mut node_bytes = Vec::new();
+    for (level, nodes_of_this_level) in nodes_by_level.into_iter().enumerate() {
+        for (index_in_level, (contents, pointer, bytes)) in
+            nodes_of_this_level.into_iter().enumerate()
+        {
+            nodes.push(CodeTwoTreeShapeNode {
+                position: CodeTwoTreeNodePosition {
+                    level: u8::try_from(level).expect("层级来自 1 字节的头字段"),
+                    index_in_level: u32::try_from(index_in_level).expect("一层的节点数装得进 u32"),
+                },
+                contents,
+            });
+            pointers.push(pointer);
+            node_bytes.push(bytes);
+        }
+    }
+    Ok(CodeTwoTreeReadFromDisk {
+        version: CodeTwoTreeVersion {
+            shape: CodeTwoTreeShape { nodes },
+            pointers,
+        },
+        node_bytes,
+        leaf_entries_in_key_order,
+    })
+}
+
+/// 读一棵树时一路不变的那几样。
+struct TreeReading<'reading> {
+    expectation: &'reading CodeTwoTreeReadExpectation,
+    judgement: CodeTwoTreeHeaderJudgement,
+    root: &'reading RootRecord,
+    expected_filesystem_identifier: u64,
+}
+
+impl TreeReading<'_> {
+    fn judges_every_header(&self) -> bool {
+        match self.judgement {
+            CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference => true,
+            CodeTwoTreeHeaderJudgement::OnlyWhatTheShapeNeeds => false,
+        }
+    }
+
+    fn key(&self, key_bytes: &[u8]) -> CodeTwoTreeKey {
+        CodeTwoTreeKey::new(key_bytes, self.expectation.field_widths)
+    }
+
+    fn read_node_and_its_subtree(
+        &self,
+        pointer: &NodePointer,
+        expected_level: Option<u8>,
+        read_node: &mut dyn FnMut(&NodePointer) -> Result<Vec<u8>, RecoveryFailure>,
+        units_seen: &mut BTreeSet<SlotNumber>,
+    ) -> Result<NodeReadFromDisk, RecoveryFailure> {
+        if !units_seen.insert(pointer.locations[0].slot) {
+            return Err(violated("同一个码 2 节点被两条父条目引用"));
+        }
+        let bytes = read_node(pointer)?;
+        let header = parse_index_node(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: if expected_level.is_none() {
+                "树根节点"
+            } else {
+                "多层码 2 树的节点"
+            },
+        })?;
+        if self.judges_every_header() {
+            judge_the_header_against_its_reference(
+                &header,
+                pointer,
+                self.expectation,
+                self.root,
+                self.expected_filesystem_identifier,
+                expected_level.is_none(),
+            )?;
+        }
+        let key_width = self.expectation.field_widths.key_width_in_bytes();
+        if header.key_width != key_width {
+            return Err(RecoveryFailure::InvariantViolated {
+                invariant: "E142 走读同款",
+                detail: "码 2 节点自述 key 宽与树的种类不符",
+            });
+        }
+        if let Some(expected) = expected_level {
+            if header.level != expected {
+                return Err(violated("孩子的层级不是父层级减一"));
+            }
+        }
+        if header.entries.is_empty() {
+            return Err(violated("多层码 2 树的节点一条条目都没有"));
+        }
+        if header.level == 0 {
+            return self.leaf_read_from_disk(pointer, bytes, header);
+        }
+        let internal_width = internal_entry_width_in_bytes(key_width);
+        if header.entry_width < internal_width {
+            return Err(RecoveryFailure::EntryNarrowerThanItsFieldTable {
+                what: self.expectation.internal_entry_name,
+                entry_bytes: header.entry_width,
+                field_table_bytes: internal_width,
+            });
+        }
+        let mut children: Vec<(CodeTwoTreeKey, NodeReadFromDisk)> =
+            Vec::with_capacity(header.entries.len());
+        // 迭代上界是这个节点的条目数；跨轮携带的是已经读回来的孩子（下一个孩子的分隔 key 要比上一个孩子头里的最大 key 大）。
+        for entry in &header.entries {
+            let (separator_bytes, child_pointer) =
+                parse_internal_entry(entry, key_width).expect("条目宽上面判过不窄于 key 宽 + 86");
+            let separator_key = self.key(&separator_bytes);
+            if self.judges_every_header() {
+                if let Some((previous_separator, previous_child)) = children.last() {
+                    if separator_key <= *previous_separator {
+                        return Err(violated("内部节点的分隔 key 不严格递增"));
+                    }
+                    if separator_key <= self.key(&previous_child.header.largest_key) {
+                        return Err(violated("分隔 key 不大于左邻孩子头里的最大 key"));
+                    }
+                }
+            }
+            let child = self.read_node_and_its_subtree(
+                &child_pointer,
+                Some(header.level - 1),
+                read_node,
+                units_seen,
+            )?;
+            if self.judges_every_header() && separator_key > self.key(&child.header.smallest_key) {
+                return Err(violated("分隔 key 大于孩子头里的最小 key"));
+            }
+            children.push((separator_key, child));
+        }
+        if self.judges_every_header() {
+            let (_, first_child) = children.first().expect("上面判过节点不空");
+            let (_, last_child) = children.last().expect("上面判过节点不空");
+            if first_child.header.smallest_key != header.smallest_key
+                || last_child.header.largest_key != header.largest_key
+            {
+                return Err(violated(
+                    "内部节点的 key 区间不是子树覆盖区间（第一个孩子的最小 key 到最后一个孩子的最大 key）",
+                ));
+            }
+        }
+        Ok(NodeReadFromDisk {
+            pointer: *pointer,
+            bytes,
+            header,
+            children,
+        })
+    }
+
+    fn leaf_read_from_disk(
+        &self,
+        pointer: &NodePointer,
+        bytes: Vec<u8>,
+        header: IndexNodeHeader,
+    ) -> Result<NodeReadFromDisk, RecoveryFailure> {
+        // 叶条目窄于它那棵树的字段表（记账 34、映射 55）不在这里判：这一步只切 key（`parse_index_node` 判过条目宽 ≥ key 宽），
+        // 解条目的一方各按各的报（冷走读与重建报 `EntryNarrowerThanItsFieldTable`，挂载态报 `RecordMalformed`），
+        // 与树长成多层之前逐字相同。
+        if self.judges_every_header() {
+            let key_width = self.expectation.field_widths.key_width_in_bytes();
+            let keys_ascend = header
+                .entries
+                .windows(2)
+                .all(|pair| self.key(&pair[0][..key_width]) < self.key(&pair[1][..key_width]));
+            if !keys_ascend {
+                return Err(violated("叶里的条目不按 key 严格递增"));
+            }
+        }
+        Ok(NodeReadFromDisk {
+            pointer: *pointer,
+            bytes,
+            header,
+            children: Vec::new(),
+        })
+    }
+}
+
+fn flatten_read(
+    node: NodeReadFromDisk,
+    field_widths: CodeTwoKeyFieldWidths,
+    nodes_by_level: &mut Vec<Vec<(CodeTwoTreeNodeContents, NodePointer, Vec<u8>)>>,
+    leaf_entries_in_key_order: &mut Vec<Vec<u8>>,
+) -> CodeTwoTreeNodePosition {
+    let level = node.header.level;
+    let key_width = field_widths.key_width_in_bytes();
+    let contents = if level == 0 {
+        let keys = node
+            .header
+            .entries
+            .iter()
+            .map(|entry| CodeTwoTreeKey::new(&entry[..key_width], field_widths))
+            .collect();
+        leaf_entries_in_key_order.extend(node.header.entries.iter().cloned());
+        CodeTwoTreeNodeContents::Leaf { keys }
+    } else {
+        let children = node
+            .children
+            .into_iter()
+            .map(|(separator_key, child)| CodeTwoTreeChild {
+                separator_key,
+                child: flatten_read(
+                    child,
+                    field_widths,
+                    nodes_by_level,
+                    leaf_entries_in_key_order,
+                ),
+            })
+            .collect();
+        CodeTwoTreeNodeContents::Internal { children }
+    };
+    let level_index = usize::from(level);
+    if nodes_by_level.len() <= level_index {
+        nodes_by_level.resize_with(level_index + 1, Vec::new);
+    }
+    let index_in_level =
+        u32::try_from(nodes_by_level[level_index].len()).expect("一层的节点数装得进 u32");
+    nodes_by_level[level_index].push((contents, node.pointer, node.bytes));
+    CodeTwoTreeNodePosition {
+        level,
+        index_in_level,
+    }
+}
+
+/// 在一版树里按 key 找它所在的那片叶：从根按分隔 key 往下走（最后一个分隔 key ≤ key 的孩子；比第一个还小就走第 0 个）。
+/// 交回叶的位置；树还没建过交回 `None`。
+#[must_use]
+pub fn leaf_position_routed_to(
+    shape: &CodeTwoTreeShape,
+    key: &CodeTwoTreeKey,
+) -> Option<CodeTwoTreeNodePosition> {
+    let mut position = shape.root()?.position;
+    // 迭代上界：每一轮下降一层。
+    loop {
+        match &shape.node(position)?.contents {
+            CodeTwoTreeNodeContents::Leaf { .. } => return Some(position),
+            CodeTwoTreeNodeContents::Internal { children } => {
+                let index = children
+                    .iter()
+                    .rposition(|child| child.separator_key <= *key)
+                    .unwrap_or(0);
+                position = children.get(index)?.child;
+            }
+        }
+    }
+}
+
+/// 一版树里每个节点头里该写的 key 区间（子树覆盖区间），按 `shape.nodes()` 同序。
+///
+/// # Panics
+/// 形状里有空节点：规划与读回都不交出空节点（空的叶只在树删到空时出现，调用方的两棵树不会删到空）。
+#[must_use]
+pub fn key_ranges_in_bump_order(shape: &CodeTwoTreeShape) -> Vec<(Vec<u8>, Vec<u8>)> {
+    shape
+        .nodes()
+        .iter()
+        .map(|node| {
+            let (smallest, largest) = shape
+                .key_range_of_the_subtree(node.position)
+                .expect("规划与读回都不交出空节点");
+            (smallest.bytes().to_vec(), largest.bytes().to_vec())
+        })
+        .collect()
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    const TEST_FIELD_WIDTHS: CodeTwoKeyFieldWidths = CodeTwoKeyFieldWidths(&[8]);
+
+    fn key(value: u64) -> CodeTwoTreeKey {
+        CodeTwoTreeKey::new(&value.to_le_bytes(), TEST_FIELD_WIDTHS)
+    }
+
+    fn keys(values: impl IntoIterator<Item = u64>) -> BTreeSet<CodeTwoTreeKey> {
+        values.into_iter().map(key).collect()
+    }
+
+    fn capacity(leaf_entries: usize, internal_entries: usize) -> CodeTwoTreeNodeCapacity {
+        CodeTwoTreeNodeCapacity {
+            leaf_entries,
+            internal_entries,
+        }
+    }
+
+    /// 每片叶装的 key，按层级 0 从左到右。
+    fn leaves(shape: &CodeTwoTreeShape) -> Vec<Vec<u64>> {
+        shape
+            .nodes()
+            .iter()
+            .filter_map(|node| match &node.contents {
+                CodeTwoTreeNodeContents::Leaf { keys } => Some(
+                    keys.iter()
+                        .map(|leaf_key| u64::from_le_bytes(leaf_key.bytes().try_into().expect("8")))
+                        .collect(),
+                ),
+                CodeTwoTreeNodeContents::Internal { .. } => None,
+            })
+            .collect()
+    }
+
+    #[test]
+    fn internal_entries_are_the_key_plus_an_eighty_six_byte_child_pointer() {
+        use singlefs_format::{
+            ACCOUNTING_INTERNAL_ENTRY_BYTES, ACCOUNTING_KEY_BYTES,
+            CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES, MAPPING_KEY_BYTES,
+        };
+        let width = |key_bytes: u64| {
+            u64::try_from(internal_entry_width_in_bytes(
+                usize::try_from(key_bytes).expect("key 宽"),
+            ))
+            .expect("条目宽")
+        };
+        assert_eq!(width(ACCOUNTING_KEY_BYTES), ACCOUNTING_INTERNAL_ENTRY_BYTES);
+        assert_eq!(
+            width(MAPPING_KEY_BYTES),
+            CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES
+        );
+        assert_eq!(
+            CodeTwoKeyFieldWidths::ACCOUNTING.key_width_in_bytes(),
+            usize::try_from(ACCOUNTING_KEY_BYTES).expect("22")
+        );
+        assert_eq!(
+            CodeTwoKeyFieldWidths::CENTRAL_MAPPING.key_width_in_bytes(),
+            usize::try_from(MAPPING_KEY_BYTES).expect("27")
+        );
+    }
+
+    #[test]
+    fn inserting_nine_keys_into_a_leaf_of_eight_splits_it_in_the_middle_and_grows_a_root() {
+        let plan = plan_the_tree_after_this_publish(
+            &CodeTwoTreeShape::default(),
+            &keys(1..=9),
+            capacity(8, 4),
+        )
+        .expect("两层装得下");
+        assert_eq!(plan.shape.height(), 2);
+        assert_eq!(
+            leaves(&plan.shape),
+            vec![vec![1, 2, 3, 4, 5], vec![6, 7, 8, 9]]
+        );
+        assert!(plan
+            .origins
+            .iter()
+            .all(|origin| *origin == CodeTwoTreeNodeOrigin::RewrittenThisPublish));
+        let CodeTwoTreeNodeContents::Internal { children } =
+            &plan.shape.root().expect("有根").contents
+        else {
+            panic!("根是内部节点");
+        };
+        assert_eq!(children[0].separator_key, key(1));
+        assert_eq!(
+            children[1].separator_key,
+            key(6),
+            "右半的分隔 key 取它自己的最小 key"
+        );
+    }
+
+    #[test]
+    fn a_key_below_the_leftmost_separator_lowers_it_and_leaves_the_right_leaf_carried() {
+        let before = plan_the_tree_after_this_publish(
+            &CodeTwoTreeShape::default(),
+            &keys([10, 20, 30, 40, 50]),
+            capacity(4, 4),
+        )
+        .expect("两层")
+        .shape;
+        assert_eq!(leaves(&before), vec![vec![10, 20, 30], vec![40, 50]]);
+        let after = plan_the_tree_after_this_publish(
+            &before,
+            &keys([5, 10, 20, 30, 40, 50]),
+            capacity(4, 4),
+        )
+        .expect("两层");
+        let CodeTwoTreeNodeContents::Internal { children } =
+            &after.shape.root().expect("根").contents
+        else {
+            panic!("根是内部节点");
+        };
+        assert_eq!(
+            children[0].separator_key,
+            key(5),
+            "最左分隔 key 压低到新插的 key"
+        );
+        assert_eq!(
+            after.origin_of(CodeTwoTreeNodePosition {
+                level: 0,
+                index_in_level: 1
+            }),
+            CodeTwoTreeNodeOrigin::CarriedFrom(CodeTwoTreeNodePosition {
+                level: 0,
+                index_in_level: 1
+            }),
+            "右叶一个 key 都没动：照抄"
+        );
+        assert_eq!(
+            after.replaced_previous_nodes,
+            vec![
+                CodeTwoTreeNodePosition {
+                    level: 0,
+                    index_in_level: 0
+                },
+                CodeTwoTreeNodePosition {
+                    level: 1,
+                    index_in_level: 0
+                }
+            ],
+            "改了的左叶与根被换下"
+        );
+    }
+
+    #[test]
+    fn deleting_every_key_of_a_leaf_drops_it_and_a_root_left_with_one_child_hands_the_root_to_it() {
+        let before = plan_the_tree_after_this_publish(
+            &CodeTwoTreeShape::default(),
+            &keys([10, 20, 30, 40, 50]),
+            capacity(4, 4),
+        )
+        .expect("两层")
+        .shape;
+        let after = plan_the_tree_after_this_publish(&before, &keys([10, 20, 30]), capacity(4, 4))
+            .expect("删成一层");
+        assert_eq!(after.shape.height(), 1, "根只剩一个孩子：降高一层");
+        assert_eq!(leaves(&after.shape), vec![vec![10, 20, 30]]);
+        assert_eq!(
+            after.origins,
+            vec![CodeTwoTreeNodeOrigin::CarriedFrom(
+                CodeTwoTreeNodePosition {
+                    level: 0,
+                    index_in_level: 0
+                }
+            )],
+            "当根的孩子不重写"
+        );
+    }
+
+    #[test]
+    fn a_leaf_split_that_overflows_a_full_root_splits_the_root_too_and_the_tree_grows_to_three_levels(
+    ) {
+        let before = plan_the_tree_after_this_publish(
+            &CodeTwoTreeShape::default(),
+            &keys(1..=4),
+            capacity(2, 2),
+        )
+        .expect("两层");
+        assert_eq!(before.shape.height(), 2);
+        assert_eq!(leaves(&before.shape), vec![vec![1, 2], vec![3, 4]]);
+        let after = plan_the_tree_after_this_publish(&before.shape, &keys(1..=5), capacity(2, 2))
+            .expect("三层");
+        assert_eq!(after.shape.height(), 3, "叶切开让根装不下，根也切开");
+        assert_eq!(leaves(&after.shape), vec![vec![1, 2], vec![3, 4], vec![5]]);
+    }
+
+    #[test]
+    fn the_level_byte_bounds_the_height() {
+        assert_eq!(u8::MAX.checked_add(1), None);
+        let mut root = PlanningNode::Internal {
+            level: u8::MAX,
+            children: vec![
+                PlanningChild {
+                    separator_key: key(1),
+                    node: PlanningNode::empty_leaf(),
+                },
+                PlanningChild {
+                    separator_key: key(2),
+                    node: PlanningNode::empty_leaf(),
+                },
+            ],
+            carried_from: None,
+        };
+        // 根的层级是 255、孩子是空叶（只为造出这一格，层级不连贯）：插一把 key 进孩子不分裂，根不切；
+        // 把内部容量压到 1 让根装不下，根分裂要长出第 256 层 ⇒ 拒绝。
+        let refused = insert_at_the_root(&mut root, key(3), capacity(4, 1));
+        assert_eq!(refused, Err(CodeTwoTreeRefusal::HeightBeyondTheLevelField));
+    }
+}
diff -ruN -x target tree/crates/singlefs-core/src/journal.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-core/src/journal.rs	2026-09-24 14:49:12.386931846 +0000
+++ tree/crates/singlefs-core/src/journal.rs	2026-09-24 18:48:58.241495366 +0000
@@ -245,8 +245,8 @@
         bytes
     }
 
-    /// 读者：magic、类型、整条校验和、fsid、载荷校验和四关，外加两条当损坏的（D23（journal 的角色与格式） 已定项 4）：
-    /// 记录标志位 0 之外有位为 1、本次发布内序号为 0。当损坏就是与校验和不过同一个结局——这条记录不算在，前缀在它之前断
+    /// 读者：magic、类型、整条校验和、fsid、载荷校验和四关，外加三条当损坏的（D23（journal 的角色与格式） 已定项 4 / 已定项 7）：
+    /// 记录标志位 0 之外有位为 1、本次发布内序号为 0、提交标记不是 0 也不是 1。当损坏就是与校验和不过同一个结局——这条记录不算在，前缀在它之前断
     /// （已定项 22 断号即止）。不查反向链，也不查一次发布之内跳不跳号：那两样要看别的记录，是前缀取法的事。
     #[must_use]
     pub fn parse(bytes: &[u8], expected_filesystem_identifier: u64) -> Option<Self> {
@@ -273,7 +273,13 @@
         let checkpoint_txg = CheckpointTxg(reader.get_u64());
         reader.skip(12 + 32);
         let transaction = reader.get_u64();
-        let is_commit = reader.get_u8() == 1;
+        // 提交标记只有 0（不带）与 1（带）两个取值（D23（journal 的角色与格式） 已定项 7）：读到 2..=255 当这条记录损坏、断链即止，
+        // 与已定项 4 读者规则那几格同一处置（主 agent 2026-09-24 定：不许把它静默读成「不带」——checker 的 I-8.8 判它违例，两边说的要是一件事）。
+        let is_commit = match reader.get_u8() {
+            0 => false,
+            1 => true,
+            2..=u8::MAX => return None,
+        };
         let ordinal_within_publish = JournalRecordOrdinalWithinPublish(reader.get_u32());
         // 序号从 1 起（已定项 4）：读到 0 当这条记录损坏。
         if ordinal_within_publish.0 == 0 {
diff -ruN -x target tree/crates/singlefs-core/src/lib.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-core/src/lib.rs	2026-09-24 02:30:54.772147939 +0000
+++ tree/crates/singlefs-core/src/lib.rs	2026-09-24 18:48:58.241504616 +0000
@@ -12,6 +12,7 @@
 pub mod block_device;
 pub mod bytes;
 pub mod checksum;
+pub mod code_two_tree;
 pub mod inode_tree;
 pub mod instance_table;
 pub mod journal;
@@ -21,6 +22,7 @@
 pub mod pointer;
 pub mod records;
 pub mod recovery;
+pub mod rollback_witness;
 pub mod root_record;
 pub mod root_ring;
 pub mod system_configuration;
diff -ruN -x target tree/crates/singlefs-core/src/make_filesystem.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-core/src/make_filesystem.rs	2026-09-24 08:50:33.721227553 +0000
+++ tree/crates/singlefs-core/src/make_filesystem.rs	2026-09-24 18:48:58.241509386 +0000
@@ -349,6 +349,8 @@
                 journal_tail: 0,
                 journal_instance: instance,
             },
+            // mkfs 之后还没有回退过：见证表空，槽内那 753 字节全 0（第一个事务的字节不变）。
+            rollback_witness: crate::rollback_witness::RollbackWitnessTable::EMPTY,
         };
         let slot = system_configuration.to_slot();
         for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
diff -ruN -x target tree/crates/singlefs-core/src/mounted_read.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-core/src/mounted_read.rs	2026-09-24 14:49:12.386988446 +0000
+++ tree/crates/singlefs-core/src/mounted_read.rs	2026-09-24 16:50:54.714217512 +0000
@@ -22,12 +22,13 @@
 
 use std::cell::Cell;
 
-use singlefs_format::{DATA_UNIT_BYTES, INODE_RECORD_BYTES, MAPPING_KEY_BYTES, NODE_BYTES};
+use singlefs_format::{DATA_UNIT_BYTES, INODE_RECORD_BYTES, NODE_BYTES};
 
 use crate::address::{
     DataUnitIndexInFile, FileOffsetInBytes, InodeNumber, SlotNumber, TreeIdentifier,
 };
 use crate::checksum::crc32_castagnoli;
+use crate::code_two_tree::{read_code_two_tree, CodeTwoTreeHeaderJudgement};
 use crate::pointer::{DataPointer, LocationEntry, NodePointer};
 use crate::records::{
     mapping_key_for_data, parse_extent_record, parse_inode_internal_entry, parse_mapping_entry,
@@ -35,11 +36,11 @@
 };
 use crate::recovery::{
     choose_root, choose_system_configuration, read_mapped_tree_node_via_hint_then_central_mapping,
-    read_mapped_tree_root, read_tree_root, read_unit_via_locations, replay_journal,
-    rollback_high_water_of_root, scan_journal, JournalScanReport, MappedTreeNodeClass, PoolReader,
-    RecoveryFailure,
+    read_mapped_tree_root, read_unit_via_locations, replay_journal, rollback_high_water_of_root,
+    scan_journal, JournalScanReport, MappedTreeNodeClass, PoolReader, RecoveryFailure,
 };
 use crate::root_record::RootRecord;
+use crate::transaction::MultiLevelCodeTwoTree;
 use crate::unit::{
     data_unit_payload, data_unit_payload_capacity, parse_data_unit, parse_index_node,
     parse_packed_unit, unit_filesystem_identifier, DataUnitHeader, UnitError, PACKED_TYPE_INODE,
@@ -52,13 +53,6 @@
 const EXTENT_KEY_THIRD_SEGMENT_OFFSET_IN_BYTES: usize = 16;
 /// extent 树的根兼叶层级：第一版一棵树只有一个节点，记录直接装在根里（`transaction::build_file_version_units`）。
 const EXTENT_TREE_ROOT_LEVEL: u8 = 0;
-/// 中央映射树的根兼叶层级：第一版同样只有一个节点，55 字节的映射条目直接装在根里。
-/// 挂载态把这一片**整个**读进内存，于是解引用时查映射不再发设备读——[`OpenFileForRead::read_at`] 报的
-/// `device_reads_issued` 是按这个前提算的（提示过期的一次解引用 = 两条提示各试一次 + 映射落点一次 = 3）。
-/// 映射长到根成了内部节点的那天这条前提不再成立，第一版不走树：在这里拒绝，不把内部条目当映射条目解
-/// （D19（块指针的结构与宽度预算） 已定项 5「挂载态怎么读映射」：整片读、多层拒绝打开，第一版的限制；
-/// 那一维正是已定项 5 自陈零测量的缓存命中维，C418（位置权威三臂的缓存命中维零测量））。
-const CENTRAL_MAPPING_TREE_ROOT_LEVEL: u8 = 0;
 /// inode 树根的层级：根是内部节点，每条条目指一片码 3 叶容器（D8（核心索引结构） 已定项 6）。
 const INODE_TREE_ROOT_LEVEL: u8 = 1;
 /// extent 叶记录 key 宽 24，inode 树 key 宽 8（`recovery::key_width_for_kind` 同一份登记）。
@@ -202,13 +196,6 @@
         expected_level: u8,
         found_level: u8,
     },
-    /// 中央映射树长成了多层（根不是根兼叶）：第一版挂载态把映射整片读进来、之后查映射不发读，
-    /// 多层映射第一版不支持，拒绝打开（D19（块指针的结构与宽度预算） 已定项 5「挂载态怎么读映射」）。
-    /// 在读映射树根之后、读任何别的单元之前返回。
-    CentralMappingWithMoreThanOneLevelIsNotSupportedInTheFirstVersion {
-        mapping_tree: TreeIdentifier,
-        mapping_root_level: u8,
-    },
     /// 读回来的单元解不开（头坏了、载荷校验和不对、补齐非零）。
     UnitMalformed {
         what: &'static str,
@@ -310,6 +297,13 @@
     pub leaves: u64,
 }
 
+/// 打开挂载态时读中央映射树读了几个节点（每个节点一次，不按位置条目数）与树高（根的层级加一）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CentralMappingTreeReadsAtOpen {
+    pub node_reads: u64,
+    pub height: u64,
+}
+
 /// 挂载态：池打开之后留在内存里的派生态（D21（权威态与派生态的分界））。**不存数据单元的载荷**——
 /// 第一版不做读缓存、不做预读，每次读都去盘上解引用。
 pub struct MountedPoolForRead {
@@ -322,6 +316,8 @@
     /// inode 树全部叶容器里的记录，按叶序、叶内次序。
     inode_records: Vec<InodeRecord>,
     central_mapping_entries: Vec<CentralMappingEntryInMountState>,
+    /// 打开时读进来的中央映射树有几个节点、多高（D19（块指针的结构与宽度预算） 已定项 5：多层时整棵读进挂载态）。
+    central_mapping_tree_reads_at_open: CentralMappingTreeReadsAtOpen,
     observation_since_open: Cell<ReadPathObservation>,
     /// 打开这一趟里树节点（extent 树根、inode 树根、inode 叶容器）的位置提示读不出、转去查中央映射的次数
     /// （D19（块指针的结构与宽度预算） 已定项 5 硬规则 3 的观测点；与 [`ReadPathObservation`] 分开数，
@@ -331,14 +327,14 @@
 
 /// 打开挂载态：把根记录到树表条目到两棵用户可见的树这一段读回内存，之后按偏移读不再碰它们。
 ///
-/// 读的次序：中央映射树的根（自举豁免，只按提示读，D19（块指针的结构与宽度预算） 已定项 8）→ 树表（同样豁免）
+/// 读的次序：中央映射树（自举豁免，只按父指针里的位置条目读，D19（块指针的结构与宽度预算） 已定项 8；多层时整棵读进来）→ 树表（同样豁免）
 /// → 树表条目 → extent 树根兼叶 → inode 树根 → 每一片 inode 叶容器。
 ///
 /// **树节点的位置提示读不出时经中央映射回退**（D19（块指针的结构与宽度预算） 已定项 8）：extent 树根、inode 树根、
 /// 每一片 inode 叶容器走 `recovery::read_mapped_tree_node_via_hint_then_central_mapping`，与冷启动走读
 /// （`recovery::walk_to_file`）同一条；查的是这一趟最先读进来的那片映射条目。映射树根与树表是自举豁免，只按提示读。
 ///
-/// 打开这一段发了几次块层读**不由核心层自己数**：这几步走的是恢复路径那几个读者（`read_tree_root`、
+/// 打开这一段发了几次块层读**不由核心层自己数**：这几步走的是恢复路径那几个读者（`code_two_tree::read_code_two_tree`、
 /// `read_unit_via_locations`），它们不带计数。要这个数就在块层数（D17（实现分层与第三方管道） 已定项 5 射程 ①：
 /// 录制钩子挂在块设备接口上），装置侧的计数器在 `singlefs_harness::read_tally`。
 /// 核心层自己报的那几个数只罩打开之后的每一次读（[`ReadPathObservation`]）。
@@ -353,29 +349,30 @@
     let filesystem_identifier_in_unit_headers =
         unit_filesystem_identifier(&root.filesystem_identifier);
 
-    // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）；它自己不进映射（自举豁免）。
+    // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）；它的节点都不进映射（自举豁免），只按父指针里的位置条目读。
     // 它不进树表，是哪棵树由根记录里它那条根指针的出生树说（第一个文件版本那次从水位发的号）。
-    let mapping_tree = root.mapping_root.head.birth_tree;
-    let mapping_root = read_tree_root(
-        reader,
-        mapping_tree,
-        usize::try_from(MAPPING_KEY_BYTES).expect("27"),
+    // 多层时整棵读进挂载态（D19（块指针的结构与宽度预算） 已定项 5「挂载态怎么读映射」）：之后解引用查的是这份内存里的条目，
+    // 不再为映射发设备读——[`OpenFileForRead::read_at`] 报的 `device_reads_issued` 按这个前提算
+    // （提示过期的一次解引用 = 两条提示各试一次 + 映射落点一次 = 3，不随映射树有几层变）。每个节点按父条目核层级与区间
+    // （`code_two_tree::read_code_two_tree`，与冷启动走读同一套）。
+    let mapping_tree = read_code_two_tree(
         &root.mapping_root,
+        &MultiLevelCodeTwoTree::CentralMapping.read_expectation(root.mapping_root.head.birth_tree),
+        CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,
         root,
         filesystem_identifier_in_unit_headers,
+        &mut |pointer: &NodePointer| {
+            read_unit_via_locations(reader, &pointer.locations, node_bytes)
+        },
     )?;
-    if mapping_root.level != CENTRAL_MAPPING_TREE_ROOT_LEVEL {
-        return Err(
-            OpenPoolForReadFailure::CentralMappingWithMoreThanOneLevelIsNotSupportedInTheFirstVersion {
-                mapping_tree,
-                mapping_root_level: mapping_root.level,
-            },
-        );
-    }
-    // 映射条目的宽度是映射树根自述的（`read_tree_root` 只判了它 ≥ key 宽 27）：切到偏移 55 之前判一次，
-    // 窄的报 `RecordMalformed`，不按字段表的固定偏移切下去（panic 面普查 R2）。
-    let mut central_mapping_entries = Vec::with_capacity(mapping_root.entries.len());
-    for entry_bytes in &mapping_root.entries {
+    let central_mapping_tree_node_reads_at_open =
+        u64::try_from(mapping_tree.version.node_count()).expect("节点数");
+    let central_mapping_tree_height_at_open = mapping_tree.version.shape.height();
+    // 映射条目的宽度是映射树叶自述的（读树那一步判过它不窄于 55）：这里仍按条目解，窄的报 `RecordMalformed`，
+    // 不按字段表的固定偏移切下去（panic 面普查 R2）。
+    let mut central_mapping_entries =
+        Vec::with_capacity(mapping_tree.leaf_entries_in_key_order.len());
+    for entry_bytes in &mapping_tree.leaf_entries_in_key_order {
         let (key, locations) =
             parse_mapping_entry(entry_bytes).ok_or(OpenPoolForReadFailure::RecordMalformed {
                 what: "映射条目",
@@ -537,6 +534,10 @@
         extent_leaf_records,
         inode_records,
         central_mapping_entries,
+        central_mapping_tree_reads_at_open: CentralMappingTreeReadsAtOpen {
+            node_reads: central_mapping_tree_node_reads_at_open,
+            height: central_mapping_tree_height_at_open,
+        },
         observation_since_open: Cell::new(ReadPathObservation::default()),
         tree_node_stale_location_hint_hops_at_open: u64::try_from(
             tree_node_stale_location_hint_hops,
@@ -617,6 +618,13 @@
         self.tree_node_stale_location_hint_hops_at_open
     }
 
+    /// 打开时把中央映射树读进挂载态读了几个节点、树高：之后解引用查映射一次设备读都不发，多层也一样
+    /// （D19（块指针的结构与宽度预算） 已定项 5「挂载态怎么读映射」）。
+    #[must_use]
+    pub const fn central_mapping_tree_reads_at_open(&self) -> CentralMappingTreeReadsAtOpen {
+        self.central_mapping_tree_reads_at_open
+    }
+
     /// 打开时沿 extent 树读了几个节点、树高、叶片数：之后按偏移读不再碰 extent 树，顺序读 M 个单元的 extent 节点读取次数就是它。
     #[must_use]
     pub const fn extent_tree_reads_at_open(&self) -> ExtentTreeReadsAtOpen {
diff -ruN -x target tree/crates/singlefs-core/src/mount.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-core/src/mount.rs	2026-09-24 16:15:19.532769843 +0000
+++ tree/crates/singlefs-core/src/mount.rs	2026-09-24 18:48:58.241518146 +0000
@@ -14,24 +14,31 @@
 use crate::pointer::{slot_shared_by_both_location_entries, LocationEntriesOnDifferentSlots};
 use crate::recovery::{
     allocation_records_of_version_without_file, allocation_records_under_root, choose_root,
-    choose_system_configuration, effective_rollback_floor, highest_root_txg,
-    highest_tree_identifier_watermark_in_the_ring, instance_table_chain_of_root,
+    choose_system_configuration, effective_rollback_floor, every_root_ring_slot_holds_a_root,
+    highest_root_txg, highest_tree_identifier_watermark_in_the_ring, instance_table_chain_of_root,
     instance_table_of_root, instance_table_page_pointers_as_far_as_readable, readable_roots,
     readable_roots_with_ring_slots, rebuild_version, replay_journal, rollback_high_water_of_root,
-    scan_journal, tree_table_has_no_entries, user_visible_tree_root_pointers, InstanceTableChain,
-    JournalScanReport, RebuildVersionFailure, RebuiltVersion, RecoveryFailure,
-    UserVisibleTreeRootPointers,
+    rollback_witness_of_the_pool, scan_journal, tree_table_has_no_entries,
+    user_visible_tree_root_pointers, InstanceTableChain, JournalScanReport, RebuildVersionFailure,
+    RebuiltVersion, RecoveryFailure, UserVisibleTreeRootPointers,
+};
+use crate::rollback_witness::{
+    rollback_witness_capacity, RollbackWitness, RollbackWitnessEntry, RollbackWitnessTable,
 };
 use crate::root_record::RootRecord;
 use crate::root_ring::{target_for_publish, RootRingSlot};
 use crate::transaction::{
     acquire_expected_instance, instance_generation_to_acquire, instance_table_chain_to_release,
-    instance_table_page_roles_in_bump_order, placements_to_release_via_mapping,
-    publish_instance_table_on_version_without_file, publish_sequence_admission, publish_version,
-    publish_without_units, AcquisitionFailed, ExpectedInstanceAcquisitionFailed,
-    InstanceTableOnlyPublishPlan, InstanceTablePlan, InstanceTableRewrite, PlacementRule,
-    PoolVersion, PoolWriter, PublishError, PublishPlan, PublishShape, TransactionOutput,
-    TransactionUnit, ZeroUnitPublishPlan,
+    instance_table_page_roles_in_bump_order,
+    multi_level_tree_nodes_of_a_publish_sequence_without_content, multi_level_tree_of_role,
+    placements_to_release_via_mapping, publish_instance_table_on_version_without_file,
+    publish_sequence_admission, publish_version, publish_without_units,
+    refuse_mapping_entries_that_do_not_name_two_pool_devices,
+    release_check_and_admission_of_a_row_publish_on_a_version_without_file,
+    rewritten_roles_of_a_publish_without_content, AcquisitionFailed, CodeTwoTreeNodeCapacities,
+    ExpectedInstanceAcquisitionFailed, InstanceTableOnlyPublishPlan, InstanceTablePlan,
+    InstanceTableRewrite, PlacementRule, PoolVersion, PoolWriter, PublishError, PublishPlan,
+    PublishShape, TransactionOutput, TransactionUnit, TreeNodeWrittenBy, ZeroUnitPublishPlan,
 };
 use crate::write_accounting::WritesByStructureKind;
 use singlefs_format::ROOT_RING_REGIONS;
@@ -158,8 +165,8 @@
     NotInRing,
     /// txg 低于生效的回退下界 F（D16（发布语义） 已定项 1「回退候选集」：txg ≥ F_生效）。
     BelowEffectiveFloor,
-    /// 最新根指着的实例表里有那个实例的行 (i, Ti, Wi) 且目标的 txg > Ti：被抛弃时间线上的根（D23（journal 的角色与格式） 已定项 14
-    /// 「按实例表判仍然有效」）。
+    /// 最新根指着的实例表里有那个实例的行 (i, Ti, Wi) 且目标的 txg > Ti，或者它被回退见证表抛弃：被抛弃时间线上的根
+    /// （D23（journal 的角色与格式） 已定项 14「按实例表判仍然有效」与「回退见证」）。
     OnAbandonedTimeline,
 }
 
@@ -221,6 +228,9 @@
     /// 被抛弃根里树表或分配记录树读不出、解不开的条数：这样的根影子账罩不到，只计数、不拒绝挂载
     /// （步 4 / 步 5 代码三方第二轮云端攻方腿打中：一条被抛弃根的树表撕裂不能让每次挂载都失败）。
     pub abandoned_roots_unreadable: u64,
+    /// 这次挂载从写行那次发布的系统配置轮换起写的回退见证表（D23（journal 的角色与格式） 已定项 14「回退见证」）：
+    /// 盘上原有的按删除规则删过（`rollback_witness_entries_still_needed`），回退时再加这一次的那一条。取号那两次写带的是删过、还没加的那一张。
+    pub rollback_witness_written: RollbackWitnessTable,
 }
 
 /// 可写挂载的结果：写出的东西、重建并推进过的分配器、接下来的发布要接在后面的那一版。
@@ -333,6 +343,8 @@
     /// 这次挂载走的影子账那一臂：产品路径恒 `On`，回退那条路由调用方给。
     /// 带着它只为一件事——让 `MountOutput::shadow_ledger_branch` 报得出走了哪一条（五条硬要求第 4 条）。
     shadow_ledger: ShadowLedger,
+    /// 恢复（或回退）择到的系统配置：回退见证表的条数上限（R × S − 1）与删除规则读的根环几何从它取。
+    system_configuration: crate::system_configuration::SystemConfiguration,
 }
 
 /// 新实例的第一次发布的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1
@@ -585,8 +597,12 @@
         .collect();
     let newest_table = choose_root(devices, system_configuration)
         .and_then(|newest| instance_table_of_root(devices, &newest));
+    // 被回退见证表抛弃的根同样是被抛弃时间线上的根（D23（journal 的角色与格式） 已定项 14「回退见证」）：最新根落回 R_old 时
+    // （回退实例的根都读不出）它那一版的实例表里没有回退行，只有见证表认得出它们。
+    let rollback_witness = rollback_witness_of_the_pool(devices, system_configuration);
     let is_abandoned = |root: &RootRecord| {
         extra_abandoned(root)
+            || rollback_witness.abandons(root.instance, root.checkpoint_txg)
             || newest_table
                 .as_ref()
                 .is_some_and(|table| abandoned_by_table(root, table))
@@ -900,6 +916,20 @@
     new_floor: CheckpointTxg,
     shadow_ledger: ShadowLedger,
 ) -> Result<RaisedFloor, MountError> {
+    // 分配器上冻结着一次没重发的发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）：这一串的第一次空发布本来就会被拒，
+    // 而下面的影子账与回收在发布之前就动分配器——重发成功时分配器整个换成那次发布之后的一份，这些改动会被一起丢掉。第一道就拒，一样都不动。
+    if let Some(frozen) = allocator.frozen_publish() {
+        return Err(MountError::RaiseFloorSequencePublishFailed(
+            PublishSequenceFailed {
+                cause: PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
+                    checkpoint_txg: frozen.checkpoint_txg(),
+                    instance: frozen.instance(),
+                },
+                writes_of_persisted_publishes: Vec::new(),
+                writes_of_failed_publishes: Vec::new(),
+            },
+        ));
+    }
     let system_configuration = choose_system_configuration(&*devices)?;
     // 候选集按现行那一版的实例表判：从它的根指着的第 0 片沿链真读出来、解出来（C502（抬 F 时现行版本里没有实例表单元）；
     // D18（块里携带什么信息） 已定项 11：一张表可以不止一片）。不看 `TransactionOutput::units`——那是这个进程内存里的角色列表，
@@ -1093,9 +1123,16 @@
             // 取号之前那道准入按 `PublishShape::EMPTY_PUBLISH` 算这一次要加几条分配记录；这里钉住它算的就是这张计划的形状。
             // 形状要先把计划算成「这次之后 inode 树是什么样、重写哪些角色」才知道（`PublishPlan::resolve`），
             // 而算它只读、不发写：算不过就在这里交回，盘上逐字节不变。
+            let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator
+                .devices
+                .iter()
+                .map(|device_map| device_map.device)
+                .collect();
             let resolved_empty_publish = plan.resolve(
                 Some(current_file_version),
                 &current_file_version.tree_identifiers,
+                &device_identities_of_the_accounting_rows,
+                pool.code_two_tree_node_capacities(),
             )?;
             assert_eq!(
                 resolved_empty_publish.shape(),
@@ -1148,8 +1185,12 @@
 /// 可写挂载与回退各按自己的 `start` 算：回退的那一版是 R_old 指着的表、要写的是 [max(r_old, 1), 新实例)。
 /// **旧链那一项两臂都判**：树表 0 条的一版上写行同样重写整条链（`publish_instance_table_on_version_without_file`）；
 /// 那一版上要写的行为空时写行那次是零单元发布、不碰实例表，不判。
-/// 分配记录树、记账树与中央映射树那三条只在带文件的一版上判：树表 0 条 ⇒ 这一版没有这三棵树，写行与暖机都不写它们
-/// （D16（发布语义） 已定项 9 那五样一样都不写）。
+/// 树表 0 条的一版上写行那次还重写那一版自己的分配记录树节点（C512（树表 0 条的一版上被换下的单元记在哪） 定案之后）：
+/// 上一版那片分配记录树节点的释放核与这次之后的分配记录条数准入同样在取号之前判
+/// （`transaction::release_check_and_admission_of_a_row_publish_on_a_version_without_file`，与发布路径同一个函数；
+/// D18（块里携带什么信息） 已定项 11「可写挂载的顺序」第五个合取，2026-09-24 用户定；此前只在取号之后的发布路径里判，
+/// 实例表长到 40 片起每试一次可写挂载就烧一个实例代号——`research/prompts/m2-final-code-r1-opus-output.md` Z4-1）。
+/// 暖机在那一版上是零单元发布，不写分配记录树。记账树与中央映射树两条只在带文件的一版上判：树表 0 条 ⇒ 这一版没有这两棵树。
 fn refuse_publishes_before_acquisition_that_do_not_pass_admission(
     allocator: &PoolAllocator,
     start: &InstanceStart,
@@ -1157,6 +1198,7 @@
     instance_table_rewrite: &InstanceTableRewrite,
     rows_to_write: usize,
     warm_up_publishes_planned: usize,
+    code_two_tree_node_capacities: CodeTwoTreeNodeCapacities,
 ) -> Result<(), MountError> {
     let row_publish_rewrites_the_instance_table = match &start.previous {
         PreviousVersion::WithFile { .. } => true,
@@ -1171,6 +1213,21 @@
                 },
             )?;
     }
+    if let (PreviousVersion::WithoutFile { root, .. }, true) =
+        (&start.previous, row_publish_rewrites_the_instance_table)
+    {
+        release_check_and_admission_of_a_row_publish_on_a_version_without_file(
+            root,
+            instance_table_rewrite,
+            allocator,
+        )
+        .map_err(
+            |cause| MountError::RowPublishAdmissionRefusedBeforeAcquisition {
+                instance_to_acquire,
+                cause,
+            },
+        )?;
+    }
     if let PreviousVersion::WithFile { output, .. } = &start.previous {
         let shapes: Vec<PublishShape> =
             std::iter::once(PublishShape::row_publish_rewriting_instance_table_pages(
@@ -1181,12 +1238,10 @@
                 warm_up_publishes_planned,
             ))
             .collect();
-        // 写行与暖机都不碰 inode 树、不写文件内容 ⇒ 这一串里每次发布之后的叶容器数与数据单元数都是上一版那两个数，
-        // 映射条目那一条准入按它们算。
-        let inode_leaf_containers = output.inode_leaf_containers.len();
-        let data_units = output.data_pointers.len();
-        publish_sequence_admission(allocator, &shapes, inode_leaf_containers, data_units).map_err(
-            |refusal| match refusal.publish_index {
+        // 写行与暖机都不碰 inode 树、不写文件内容 ⇒ 这一串里每次只换分配记录树与记账树节点的映射 key，两棵多层码 2 树
+        // 每次重写几个节点从上一版的形状推（`publish_sequence_admission`），容量与取号之后那几次发布的写入口上装的相同。
+        publish_sequence_admission(allocator, &shapes, output, code_two_tree_node_capacities)
+            .map_err(|refusal| match refusal.publish_index {
                 0 => MountError::RowPublishAdmissionRefusedBeforeAcquisition {
                     instance_to_acquire,
                     cause: refusal.cause,
@@ -1197,8 +1252,7 @@
                     warm_up_publishes_planned,
                     cause: refusal.cause,
                 },
-            },
-        )?;
+            })?;
     }
     Ok(())
 }
@@ -1218,9 +1272,14 @@
     /// 每一次的每个角色都取到了：按次序交回每一次取到的。
     TakenByEveryPublish(Vec<PlacementsTakenByOnePublish>),
     Refused(PlacementRefusedOnTheCopy),
-    /// 写行那次（带文件的一版上）经映射查换下的落点就报错了（`transaction::placements_to_release_via_mapping` 的那几种）：
-    /// 发布路径走到同一处报同一个错（它只读内存里的上一版与分配器，取号不碰这两样），这里不判、不拦，照旧交给发布路径报。
-    ReleaseCheckFailedBeforeTheFirstPlacement,
+    /// 第 `publish_index` 次（从 0 数，0 是写行那次）在拷贝上释放换下的落点时释放核验报错（`transaction::placements_to_release_via_mapping`
+    /// 的那几种，或映射条目的位置项不是池里两块不同的盘——`transaction::refuse_mapping_entries_that_do_not_name_two_pool_devices`）：
+    /// 发布路径走到同一处报同一个错（它只读内存里的上一版与分配器，取号不碰这两样）。D18（块里携带什么信息） 已定项 11「可写挂载的顺序」
+    /// 第五个合取：预演里写行那次的释放核验报错，同样判这次不能可写——在取号之前拒，不烧实例号。暖机那几次预演取不到落点同样不能可写。
+    ReleaseCheckRefused {
+        publish_index: usize,
+        cause: PublishError,
+    },
 }
 
 /// 一个角色在分配器上取落点：与发布路径同一条政策（`TransactionUnit::placement`：用户数据按政策函数，其余是提交内生块）。
@@ -1244,17 +1303,28 @@
 /// 先释放这一次换下的落点（进 defer、槽仍占着），再按 bump 次序给这次重写的每个角色取落点，取完记这条根盖掉的根环槽
 /// （`PoolAllocator::record_root_written_by_this_process`：根环转过就按谓词回收、清离开根环的被抛弃根的隔离位）。重写的角色：
 /// 带文件的一版上写行是 `PublishShape::row_publish_rewriting_instance_table_pages`（这次之后实例表链几片就重写几个实例表角色，
-/// 尾片先）、暖机是 `PublishShape::EMPTY_PUBLISH`（`publish_empty_after` 那条断言钉住暖机与它相同），
-/// 暖机换下的就是上一次在拷贝上取到的那几个；写行换下的是这一版的整条实例表链（`transaction::instance_table_chain_to_release`，
-/// 排最前）加经映射查到的那几个，与 `transaction::publish_version` 同序。树表 0 条那一版上写行是实例表链各片（尾片先）加分配记录树节点
-/// （这次没有行要写时是零单元发布），暖机是零单元发布。那一版上写行换下的旧链与分配记录树节点不在这里释放：它之后只有零单元发布，
-/// 释放不释放，取到的落点都一样。
+/// 尾片先）、暖机是 `PublishShape::EMPTY_PUBLISH`（`publish_empty_after` 那条断言钉住暖机与它相同）里不属于多层码 2 树的那几个，
+/// 加这一次记账树与中央映射树要重写的节点——按两棵树的形状逐次推
+/// （`transaction::multi_level_tree_nodes_of_a_publish_sequence_without_content`，与发布路径同一个规划），插在树表之前。
+/// 写行换下的是这一版的整条实例表链（`transaction::instance_table_chain_to_release`，排最前）加经映射查到的那几个，
+/// 与 `transaction::publish_version` 同序；暖机换下的：不属于多层码 2 树的角色是上一次在拷贝上为它取到的那个，
+/// 两棵树换下的节点是哪一次写出来的就取哪一次的落点（这一串之前那一版的经那一版的映射或父指针查）。
+/// 树表 0 条那一版上写行是实例表链各片（尾片先）加分配记录树节点（这次没有行要写时是零单元发布），暖机是零单元发布。
+/// 那一版上写行换下的旧链与分配记录树节点不在这里释放：它之后只有零单元发布，释放不释放，取到的落点都一样。
 ///
 /// 发布路径在取落点之前还读盘核换下的每一份的校验和，对不上（读不出也算）的那一份在它那块盘上的分配记录留在已分配、不释放
 /// （`transaction::copies_failing_the_release_checksum_check`，D19（块指针的结构与宽度预算） 已定项 5），这里不做（它要读盘）、照常释放。
 /// 两边只在那一槽这一串里被回收时分叉：拷贝上它回收了、可能再发出去，真发时它一直占着；这一串里回收它，得这一串自己的根把根环里
 /// 比它旧的有效根全盖掉（回退到环里最旧的那条根、其余全被抛弃时走得到）。所以这一串里没有一份核出对不上时，这里取到的与真发起来取到的逐项相同
 /// （`establish_instance` 在这一串发完之后断言）；有一份核出对不上时可能不同，那一格见 `establish_instance` 的注释。
+#[allow(
+    clippy::too_many_lines,
+    reason = "一串发布在拷贝上逐次释放、取落点、记根：每一步与发布路径逐步对齐，拆开只会把次序藏进几个函数"
+)]
+#[allow(
+    clippy::too_many_arguments,
+    reason = "这一串的输入各是一样：上一版、实例表怎么重写、要写的行数、各次的 txg、要取的号、两棵树的节点容量，与准入读的是同一组"
+)]
 fn placements_of_the_publishes_after_acquisition_on_a_copy(
     allocator: &PoolAllocator,
     previous: &PreviousVersion,
@@ -1262,56 +1332,194 @@
     rows_to_write: usize,
     row_publish_txg: CheckpointTxg,
     warm_up_publish_txgs: &[CheckpointTxg],
+    instance_to_acquire: InstanceGeneration,
+    code_two_tree_node_capacities: CodeTwoTreeNodeCapacities,
 ) -> PlacementsOnTheCopy {
     let mut copy = allocator.clone();
-    let (roles_of_the_row_publish, released_by_the_row_publish, roles_of_a_warm_up_publish) =
+    let txgs: Vec<CheckpointTxg> = std::iter::once(row_publish_txg)
+        .chain(warm_up_publish_txgs.iter().copied())
+        .collect();
+    // 每一次重写的角色、这一次换下的上一版角色（只第 0 次有：写行那次换下的都是挂载时那一版的），与两棵多层码 2 树每一次换下的节点
+    // （后几次换下的可以是这一串里前面某一次写出来的，落点取那一次在拷贝上为那个角色取到的）。
+    let (roles_of_each_publish, released_roles_of_the_row_publish, tree_nodes_of_each_publish) =
         match previous {
             PreviousVersion::WithFile { output, .. } => {
-                let roles = PublishShape::row_publish_rewriting_instance_table_pages(
-                    instance_table_rewrite.pages_after_this_publish(),
+                let shapes: Vec<PublishShape> =
+                    std::iter::once(PublishShape::row_publish_rewriting_instance_table_pages(
+                        instance_table_rewrite.pages_after_this_publish(),
+                    ))
+                    .chain(std::iter::repeat_n(
+                        PublishShape::EMPTY_PUBLISH,
+                        warm_up_publish_txgs.len(),
+                    ))
+                    .collect();
+                let device_identities: Vec<DeviceIdentity> = allocator
+                    .devices
+                    .iter()
+                    .map(|device_map| device_map.device)
+                    .collect();
+                // 取号之前那道准入（`refuse_publishes_before_acquisition_that_do_not_pass_admission`）刚按同一个上一版、同一串形状、
+                // 同一个容量推过这两棵树、没拒；推得出推不出只看 key 的次序与条数，与 txg、实例代号的具体值无关。
+                let tree_nodes = multi_level_tree_nodes_of_a_publish_sequence_without_content(
+                    output,
+                    &shapes,
+                    &txgs,
+                    instance_to_acquire,
+                    &device_identities,
+                    code_two_tree_node_capacities,
                 )
-                .rewritten_roles();
-                let Ok(mut released) =
-                    instance_table_chain_to_release(&instance_table_rewrite.replaced_chain, &copy)
-                else {
-                    return PlacementsOnTheCopy::ReleaseCheckFailedBeforeTheFirstPlacement;
-                };
-                let Ok(released_via_mapping) =
-                    placements_to_release_via_mapping(output, &copy, &roles)
-                else {
-                    return PlacementsOnTheCopy::ReleaseCheckFailedBeforeTheFirstPlacement;
-                };
-                released.extend(released_via_mapping);
+                .expect("取号之前那道准入刚按同样的输入推过这两棵树、没拒");
+                let roles_of_each_publish: Vec<Vec<TransactionUnit>> = shapes
+                    .iter()
+                    .zip(&tree_nodes)
+                    .map(|(shape, nodes)| {
+                        rewritten_roles_of_a_publish_without_content(*shape, &nodes.rewritten_roles)
+                    })
+                    .collect();
+                let released_roles_of_the_row_publish: Vec<TransactionUnit> = roles_of_each_publish
+                    [0]
+                .iter()
+                .copied()
+                .filter(|identity| multi_level_tree_of_role(*identity).is_none())
+                .chain(
+                    tree_nodes[0]
+                        .replaced
+                        .iter()
+                        .map(|written_by| match written_by {
+                            TreeNodeWrittenBy::TheVersionBeforeTheSequence(role) => *role,
+                            TreeNodeWrittenBy::PublishOfTheSequence { .. } => {
+                                unreachable!("第 0 次换下的节点都是挂载时那一版的")
+                            }
+                        }),
+                )
+                .collect();
                 (
-                    roles,
-                    released,
-                    PublishShape::EMPTY_PUBLISH.rewritten_roles(),
+                    roles_of_each_publish,
+                    released_roles_of_the_row_publish,
+                    tree_nodes,
                 )
             }
-            PreviousVersion::WithoutFile { .. } if rows_to_write == 0 => {
-                (Vec::new(), Vec::new(), Vec::new())
-            }
+            PreviousVersion::WithoutFile { .. } if rows_to_write == 0 => (
+                txgs.iter().map(|_| Vec::new()).collect(),
+                Vec::new(),
+                Vec::new(),
+            ),
             PreviousVersion::WithoutFile { .. } => (
-                instance_table_page_roles_in_bump_order(
-                    instance_table_rewrite.pages_after_this_publish(),
-                )
-                .into_iter()
-                .chain(std::iter::once(TransactionUnit::AllocationTree))
-                .collect(),
+                txgs.iter()
+                    .enumerate()
+                    .map(|(publish_index, _)| {
+                        if publish_index == 0 {
+                            instance_table_page_roles_in_bump_order(
+                                instance_table_rewrite.pages_after_this_publish(),
+                            )
+                            .into_iter()
+                            .chain(std::iter::once(TransactionUnit::AllocationTree))
+                            .collect()
+                        } else {
+                            Vec::new()
+                        }
+                    })
+                    .collect(),
                 Vec::new(),
                 Vec::new(),
             ),
         };
-    let txgs = std::iter::once(row_publish_txg).chain(warm_up_publish_txgs.iter().copied());
-    let mut released_by_the_next_publish = released_by_the_row_publish;
     let mut taken_by_every_publish: Vec<PlacementsTakenByOnePublish> = Vec::new();
-    for (publish_index, txg) in txgs.enumerate() {
-        let roles = if publish_index == 0 {
-            &roles_of_the_row_publish
+    for (publish_index, txg) in txgs.iter().copied().enumerate() {
+        let roles = &roles_of_each_publish[publish_index];
+        // 先释放这一次换下的（进 defer、槽仍占着），再取这一次的落点（D3（空间分配） 已定项 7）。
+        let released: Vec<Placement> = if publish_index == 0 {
+            match previous {
+                PreviousVersion::WithFile { output, .. } => {
+                    // 整条实例表旧链排最前，再是经映射查到的那几个（实例表豁免映射，经映射那一路跳过它），与发布路径同序；
+                    // 经映射换下的那几个，映射条目的位置项要各指池里一块不同的盘（发布路径在读盘核之前判的同一件事，不读盘）。
+                    let pool_devices: Vec<DeviceIdentity> = copy
+                        .devices
+                        .iter()
+                        .map(|device_map| device_map.device)
+                        .collect();
+                    let checked = instance_table_chain_to_release(
+                        &instance_table_rewrite.replaced_chain,
+                        &copy,
+                    )
+                    .and_then(|mut released| {
+                        released.extend(placements_to_release_via_mapping(
+                            output,
+                            &copy,
+                            &released_roles_of_the_row_publish,
+                        )?);
+                        refuse_mapping_entries_that_do_not_name_two_pool_devices(
+                            output,
+                            &released_roles_of_the_row_publish,
+                            &pool_devices,
+                        )?;
+                        Ok(released)
+                    });
+                    match checked {
+                        Ok(released) => released,
+                        Err(cause) => {
+                            return PlacementsOnTheCopy::ReleaseCheckRefused {
+                                publish_index,
+                                cause,
+                            }
+                        }
+                    }
+                }
+                PreviousVersion::WithoutFile { .. } => Vec::new(),
+            }
         } else {
-            &roles_of_a_warm_up_publish
+            let taken_by_the_publish_before = &taken_by_every_publish[publish_index - 1];
+            // 不属于多层码 2 树的角色：这一次重写的，上一次也重写过，换下的就是上一次为它取到的那个落点。
+            let mut released: Vec<Placement> = roles
+                .iter()
+                .filter(|identity| multi_level_tree_of_role(**identity).is_none())
+                .filter_map(|role| {
+                    taken_by_the_publish_before
+                        .iter()
+                        .find(|(taken_role, _)| taken_role == role)
+                        .map(|(_, slot)| Placement {
+                            slot: *slot,
+                            span: role.span_slots(),
+                        })
+                })
+                .collect();
+            let tree_nodes_replaced = tree_nodes_of_each_publish
+                .get(publish_index)
+                .map_or(&[][..], |nodes| nodes.replaced.as_slice());
+            for written_by in tree_nodes_replaced {
+                match written_by {
+                    TreeNodeWrittenBy::TheVersionBeforeTheSequence(role) => {
+                        let PreviousVersion::WithFile { output, .. } = previous else {
+                            unreachable!("树表 0 条的一版没有多层码 2 树")
+                        };
+                        match placements_to_release_via_mapping(output, &copy, &[*role]) {
+                            Ok(placements) => released.extend(placements),
+                            Err(cause) => {
+                                return PlacementsOnTheCopy::ReleaseCheckRefused {
+                                    publish_index,
+                                    cause,
+                                }
+                            }
+                        }
+                    }
+                    TreeNodeWrittenBy::PublishOfTheSequence {
+                        publish_index: written_by_publish,
+                        role,
+                    } => {
+                        let (_, slot) = taken_by_every_publish[*written_by_publish]
+                            .iter()
+                            .find(|(taken_role, _)| taken_role == role)
+                            .expect("那一次重写的节点在那一次取到了落点");
+                        released.push(Placement {
+                            slot: *slot,
+                            span: role.span_slots(),
+                        });
+                    }
+                }
+            }
+            released
         };
-        for placement in std::mem::take(&mut released_by_the_next_publish) {
+        for placement in released {
             copy.release(placement, txg);
         }
         let mut taken_by_this_publish: PlacementsTakenByOnePublish = Vec::new();
@@ -1328,19 +1536,6 @@
             }
         }
         copy.record_root_written_by_this_process(txg);
-        // 下一次（暖机）换下的是这一次取到的那几个同角色的落点：这一次重写了下一次要重写的每个角色。
-        released_by_the_next_publish = roles_of_a_warm_up_publish
-            .iter()
-            .filter_map(|role| {
-                taken_by_this_publish
-                    .iter()
-                    .find(|(taken_role, _)| taken_role == role)
-                    .map(|(_, slot)| Placement {
-                        slot: *slot,
-                        span: role.span_slots(),
-                    })
-            })
-            .collect();
         taken_by_every_publish.push(taken_by_this_publish);
     }
     PlacementsOnTheCopy::TakenByEveryPublish(taken_by_every_publish)
@@ -1358,7 +1553,15 @@
             .map(|role| (*role, output.unit(*role).slot))
             .collect(),
         PoolVersion::WithoutFile(version_without_file) => {
-            let named = &version_without_file.record.named;
+            // 点名项按取落点的次序排在这次发布的各条记录里（末条再跨记录时前几条在 `earlier_records_of_this_publish`，
+            // D23（journal 的角色与格式） 已定项 17），按记录的次序接起来就是整次发布的点名项。
+            let named: Vec<&crate::journal::NamedUnit> = version_without_file
+                .earlier_records_of_this_publish
+                .iter()
+                .map(|written| &written.record)
+                .chain(std::iter::once(&version_without_file.record))
+                .flat_map(|record| record.named.iter())
+                .collect();
             let Some(instance_table_pages) = named.len().checked_sub(1) else {
                 return Vec::new();
             };
@@ -1477,6 +1680,77 @@
     ))
 }
 
+/// 回退见证的删除规则（D23（journal 的角色与格式） 已定项 14「回退见证」：条目什么时候删随实现；实二二三定，交代码三方）：
+/// 条目 (N, r_old, T_old) 删掉 ⟺ 根环每一个槽都读得出、自证过（`every_root_ring_slot_holds_a_root`），而且其中没有一条根的
+/// 实例代号落在 [r_old, N) 里。别的一律留着。
+///
+/// 为什么是这两条：
+/// - 条目抛弃的是实例代号落在 (r_old, N) 里的根、与实例 r_old 里 txg 越过 T_old 的根；它还护着重放——所选根是实例 r_old、txg 不超过
+///   T_old 的根时，重放会走到 r_old 越过 T_old 的被抛弃记录（`recovery::replay_journal` 那一判）。环里没有一条根的实例代号落在
+///   [r_old, N) 里，这两样都没有对象；之后新写的根的实例代号都不小于这次挂载取的号（大于 N），环里再也长不出这样的根，删掉永远安全。
+/// - 读不出、自证不过的槽按「可能住着这样一条根」算，与 D18（块里携带什么信息） 已定项 11 实例表行回收的根环条件同一个读法：
+///   一个暂时读不出的槽能让条目被删、槽恢复之后被抛弃的根回到择根的候选里，就是 C332 那一格。代价：根环有一个槽持续读不出时条目永远删不掉。
+fn rollback_witness_entries_still_needed(
+    witness: &RollbackWitness,
+    every_ring_root: Option<&[RootRecord]>,
+) -> Vec<RollbackWitnessEntry> {
+    witness
+        .entries()
+        .into_iter()
+        .filter(|entry| match every_ring_root {
+            None => true,
+            Some(roots) => roots.iter().any(|root| {
+                entry.rollback_target_instance <= root.instance
+                    && root.instance < entry.new_instance
+            }),
+        })
+        .collect()
+}
+
+/// 这次挂载要写的两张回退见证表：取号那两次写带删过的旧表（回退那一条还不在里面：见证随写行之后的第一次系统配置轮换写，
+/// D23（journal 的角色与格式） 已定项 14「回退见证」，写序 post），写行那次发布的轮换起带再加上这一次回退那一条的表（普通挂载两张相同）。
+/// 盘上原有的见证读自各盘择到的那一槽（`recovery::rollback_witness_of_the_pool`），按删除规则删过（`rollback_witness_entries_still_needed`）。
+///
+/// # Errors
+/// 表装不下（条数越过 R × S − 1）⇒ `MountError::Recovery(RecoveryFailure::RollbackWitnessTableFullWhoseHandlingIsUndecided)`：
+/// 条款说表写不满（上限只由根环几何定），那是按「根环每个槽都读得出」推的；删除规则把读不出的槽按「可能住着被抛弃的根」算，
+/// 有槽持续读不出时条目删不掉、表就写得满——那时怎么办条款没写 ⇒ 第一版不支持，在任何写之前返回，盘上逐字节不变。
+fn rollback_witness_tables_of_this_mount<Device: BlockDevice>(
+    devices: &[(DeviceIdentity, Device)],
+    system_configuration: &crate::system_configuration::SystemConfiguration,
+    previous_row: &PreviousInstanceRow,
+    instance_to_acquire: InstanceGeneration,
+) -> Result<(RollbackWitnessTable, RollbackWitnessTable), MountError> {
+    let capacity = rollback_witness_capacity(
+        system_configuration
+            .immutable
+            .sizes
+            .root_ring_slots_per_region,
+    );
+    let witness = rollback_witness_of_the_pool(devices, system_configuration);
+    let every_ring_root = every_root_ring_slot_holds_a_root(devices, system_configuration);
+    let kept = rollback_witness_entries_still_needed(&witness, every_ring_root.as_deref());
+    let this_rollback = previous_row.is_rollback.then_some(RollbackWitnessEntry {
+        new_instance: instance_to_acquire,
+        rollback_target_instance: previous_row.instance,
+        rollback_target_txg: previous_row.selected_root_txg,
+    });
+    let full = |refused: crate::rollback_witness::RollbackWitnessTableFull| {
+        MountError::Recovery(
+            RecoveryFailure::RollbackWitnessTableFullWhoseHandlingIsUndecided {
+                entries: refused.entries,
+                capacity: refused.capacity,
+            },
+        )
+    };
+    let before_the_row_publish =
+        RollbackWitnessTable::of_entries(kept.iter().copied(), capacity).map_err(full)?;
+    let from_the_row_publish =
+        RollbackWitnessTable::of_entries(kept.into_iter().chain(this_rollback), capacity)
+            .map_err(full)?;
+    Ok((before_the_row_publish, from_the_row_publish))
+}
+
 /// 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘：可写挂载与回退共用的后半段。
 fn establish_instance<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
@@ -1508,6 +1782,7 @@
         &instance_table_rewrite,
         rows_written.len(),
         warm_up_publishes_planned.len(),
+        pool.code_two_tree_node_capacities(),
     )?;
     // 这一串自己的落点也在取号之前先取一遍（分配器的拷贝上，与后面真发读同一个分配器——取号不碰它）：取不到就在任何写之前返回。
     // 怎么把它们算进取号之前的准入，条款没定（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided` 的文档注释）。
@@ -1518,6 +1793,8 @@
         rows_written.len(),
         start.first_txg,
         &warm_up_publishes_planned,
+        instance_to_acquire,
+        pool.code_two_tree_node_capacities(),
     ) {
         PlacementsOnTheCopy::TakenByEveryPublish(taken) => Some(taken),
         PlacementsOnTheCopy::Refused(refused) => {
@@ -1531,8 +1808,37 @@
                 },
             )
         }
-        PlacementsOnTheCopy::ReleaseCheckFailedBeforeTheFirstPlacement => None,
+        PlacementsOnTheCopy::ReleaseCheckRefused {
+            publish_index: 0,
+            cause,
+        } => {
+            return Err(MountError::RowPublishAdmissionRefusedBeforeAcquisition {
+                instance_to_acquire,
+                cause,
+            })
+        }
+        PlacementsOnTheCopy::ReleaseCheckRefused {
+            publish_index: warm_up_publish_index,
+            cause,
+        } => {
+            return Err(MountError::WarmUpAdmissionRefusedBeforeAcquisition {
+                instance_to_acquire,
+                warm_up_publish_index,
+                warm_up_publishes_planned: warm_up_publishes_planned.len(),
+                cause,
+            })
+        }
     };
+    // 回退见证表（D23（journal 的角色与格式） 已定项 14「回退见证」）：装不下在取号之前拒；取号那两次写带删过的旧表，
+    // 写行那次发布的轮换起带再加上这一次回退那一条的表。
+    let (rollback_witness_before_the_row_publish, rollback_witness_from_the_row_publish) =
+        rollback_witness_tables_of_this_mount(
+            pool.devices,
+            &start.system_configuration,
+            &start.previous_row,
+            instance_to_acquire,
+        )?;
+    pool.write_rollback_witness_from_now_on(rollback_witness_before_the_row_publish);
     let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(
         |failure| match failure {
             ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite {
@@ -1551,6 +1857,9 @@
         instance, instance_to_acquire,
         "acquire_expected_instance 写之前重算、与判定时的号不等就不写：交回的就是列行与判准入用的那个号"
     );
+    // 取号那两次写之后的第一次系统配置写就是写行那次发布的轮换（写行在根 FUA 之后才轮换，D16（发布语义） 已定项 7）：
+    // 从这里起带着这一次回退那一条（D23（journal 的角色与格式） 已定项 14「回退见证」随写行之后的第一次系统配置轮换写）。
+    pool.write_rollback_witness_from_now_on(rollback_witness_from_the_row_publish);
 
     // 取号之后的每一次发布失败都带着这次挂载的写入口交得出的账返回（`PublishSequenceFailed`）：写入口随错一起丢掉。
     let row_publish = match &start.previous {
@@ -1676,6 +1985,7 @@
             shadow_ledger_branch: start.shadow_ledger.branch_name(),
             isolated_slots_per_device,
             abandoned_roots_unreadable: start.abandoned_roots_unreadable,
+            rollback_witness_written: rollback_witness_from_the_row_publish,
         },
         allocator,
         current,
@@ -1705,15 +2015,17 @@
         true,
         rollback_high_water_of_root(&*devices, &chosen_root),
     )?;
-    // 所选根覆盖的最后一条记录读得出就拿它当上一版的记录：同实例、同 checkpoint_txg 的记录里 jsn 最大的那条
-    // （D23（journal 的角色与格式） 已定项 14 注 1，P6 2026-09-23 定；一次发布切成多条记录时那次发布的末条）。
-    // 读不出（两份都撕了）就拿最大 jsn 那条顶着——本实例的第一条反向链恒 0、
+    // 所选根覆盖的最后一条记录读得出就拿它当上一版的记录：同实例、同 checkpoint_txg 的记录里带「本次发布末条」标志的那一条
+    // （D23（journal 的角色与格式） 已定项 14 注 1，读法乙，用户 2026-09-24 定：「那条」按末条标志认；一次发布切成多条记录时它是那次发布的末条）。
+    // 读不出（两份都撕了，或那次发布读得出的几条都不带标志）就拿环里最大 jsn 那条顶着——本实例的第一条反向链恒 0、
     // 事务号从 1 起，上一版的记录只有 jsn 会被用到，而 jsn 下面另算。树表 0 条的根用不到记录（只做过 mkfs 的池环里一条都没有）。
     let own_record = records
         .values()
         .filter(|record| {
             record.instance == effective_root.instance
                 && record.checkpoint_txg == effective_root.checkpoint_txg
+                && record.place_in_publish
+                    == crate::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
         })
         .max_by_key(|record| record.counter)
         .or_else(|| records.values().max_by_key(|record| record.counter))
@@ -1768,6 +2080,7 @@
             effective_floor,
             abandoned_roots_unreadable,
             shadow_ledger: ShadowLedger::On,
+            system_configuration,
         },
     )
 }
@@ -1822,10 +2135,14 @@
             exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,
         });
     }
+    // 被回退见证表抛弃的根同样在被抛弃的时间线上（D23（journal 的角色与格式） 已定项 14「回退见证」）：最新根落回 R_old 时
+    // （回退实例的根都读不出）它那一版的实例表里没有回退行，只按实例表判会把被抛弃的根放回候选集。
     if newest_table
         .rows
         .iter()
         .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg)
+        || rollback_witness_of_the_pool(&*devices, &system_configuration)
+            .abandons(target.instance, target.checkpoint_txg)
     {
         return Err(MountError::RollbackTargetNotACandidate {
             target,
@@ -1923,6 +2240,7 @@
             effective_floor,
             abandoned_roots_unreadable,
             shadow_ledger,
+            system_configuration,
         },
     )
 }
diff -ruN -x target tree/crates/singlefs-core/src/recovery.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-core/src/recovery.rs	2026-09-24 16:15:19.532788812 +0000
+++ tree/crates/singlefs-core/src/recovery.rs	2026-09-24 18:48:58.241538946 +0000
@@ -14,7 +14,7 @@
     journal_in_flight_record_limit, ACCOUNTING_ENTRY_BYTES, ALLOCATION_RECORD_BYTES,
     DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES, FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
     INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT,
-    MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES, NODE_BYTES, ROOT_RING_REGIONS, SLOT_BYTES,
+    MAPPING_ENTRY_BYTES, NODE_BYTES, ROOT_RING_REGIONS, SLOT_BYTES,
     SYSTEM_CONFIGURATION_SLOT_BYTES, UNIT_AREA_START_SLOT,
 };
 
@@ -25,6 +25,9 @@
 use crate::allocator::{unit_area_slots_of_device, AllocationRecord};
 use crate::block_device::BlockDevice;
 use crate::checksum::crc32_castagnoli;
+use crate::code_two_tree::{
+    read_code_two_tree, CodeTwoTreeHeaderJudgement, CodeTwoTreeReadFromDisk,
+};
 use crate::inode_tree::{InodeLeafContainer, InodeLeafContainerIndexInTree};
 use crate::instance_table::{
     InstanceTableChainRecord, InstanceTablePage, InstanceTablePageIndex, InstanceTableRecords,
@@ -40,6 +43,7 @@
     TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_EXTENT,
     TREE_KIND_INODE, TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE,
 };
+use crate::rollback_witness::RollbackWitness;
 use crate::root_record::RootRecord;
 use crate::root_ring::target_for_publish;
 use crate::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegionOutOfRange};
@@ -47,8 +51,8 @@
     SystemConfiguration, SystemConfigurationSlotRefusal, SystemImmutableSizes,
 };
 use crate::transaction::{
-    FileVersionTreeIdentifiers, InodeLeafContainerVersion, PublishedUnit, TransactionOutput,
-    TransactionUnit, FIRST_INODE_NUMBER,
+    FileVersionTreeIdentifiers, InodeLeafContainerVersion, MultiLevelCodeTwoTree, PublishedUnit,
+    TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
 };
 use crate::unit::{
     data_unit_payload, data_unit_payload_capacity, parse_data_unit, parse_index_node,
@@ -200,6 +204,11 @@
         checkpoint_txg: CheckpointTxg,
         counters: Vec<u64>,
     },
+    /// 可写挂载或回退要写的回退见证表装不下：删除规则删过之后（回退时再加上这一次那一条）条目数 `entries` 越过这个池的上限
+    /// `capacity`（根环槽数减 1，D23（journal 的角色与格式） 已定项 14「回退见证」）。条款说表写不满，那是按「根环每个槽都读得出」推的；
+    /// 删除规则把读不出的槽按「可能住着被抛弃的根」算，有槽持续读不出时条目删不掉、表就写得满——那时怎么办条款没写 ⇒ 第一版不支持：
+    /// 挂载在取号之前返回，盘上逐字节不变。
+    RollbackWitnessTableFullWhoseHandlingIsUndecided { entries: usize, capacity: usize },
 }
 
 /// 恢复的结果：择到的根下面没有文件（第 0 代）、读回文件、或走不下去。`root` 恒是**所选**的那条根。
@@ -443,53 +452,61 @@
     }
 }
 
-/// 从盘上重建上一版、读一条根的分配记录时用的中央映射树根（自举豁免，只按根记录里的位置条目读，D19（块指针的结构与宽度预算） 已定项 8）：
-/// 到第一次有提示读不出、要经映射回退时，或重建走到映射树根那一步时才读，读过一次就留着——盘上字节连同解开的节点，重建要把字节照抄进上一版。
-/// 不提前读：提示都读得出的镜像上，读序与报错的次序照旧。读法同重建原来读映射树根那一步（只解节点、不核自描述），
-/// 冷走读那一份（`CentralMappingRootReadOnFirstUse`）另核树 ID、key 宽、出生身份与 fsid。
-struct CentralMappingRootWithBytesReadOnFirstUse<'reader> {
+/// 从盘上重建上一版、读一条根的分配记录时用的中央映射树（自举豁免，只按父指针里的位置条目读，D19（块指针的结构与宽度预算） 已定项 8）：
+/// 到第一次有提示读不出、要经映射回退时，或重建走到映射树那一步时才读，读过一次就留着——每个节点的盘上字节连同形状，重建要把字节照抄进上一版。
+/// 多层时整棵读回来（D8（核心索引结构） 已定项 11），按「拼得成一棵树」核（`code_two_tree::CodeTwoTreeHeaderJudgement::OnlyWhatTheShapeNeeds`，
+/// 与重建原来读映射树根那一步同一个口径：解得开就收、不核自描述），冷走读那一份（`CentralMappingTreeReadOnFirstUse`）另核树 ID、
+/// key 宽、出生身份、fsid、层级与区间。不提前读：提示都读得出的镜像上，读序与报错的次序照旧。
+struct CentralMappingTreeWithBytesReadOnFirstUse<'reader> {
     reader: &'reader dyn PoolReader,
-    mapping_root: NodePointer,
-    bytes_and_node: OnceCell<(Vec<u8>, IndexNodeHeader)>,
+    root: &'reader RootRecord,
+    tree: OnceCell<CodeTwoTreeReadFromDisk>,
 }
 
-impl<'reader> CentralMappingRootWithBytesReadOnFirstUse<'reader> {
-    fn new(reader: &'reader dyn PoolReader, mapping_root: NodePointer) -> Self {
+impl<'reader> CentralMappingTreeWithBytesReadOnFirstUse<'reader> {
+    fn new(reader: &'reader dyn PoolReader, root: &'reader RootRecord) -> Self {
         Self {
             reader,
-            mapping_root,
-            bytes_and_node: OnceCell::new(),
+            root,
+            tree: OnceCell::new(),
         }
     }
 
-    fn bytes_and_node(&self) -> Result<&(Vec<u8>, IndexNodeHeader), RecoveryFailure> {
-        if let Some(read) = self.bytes_and_node.get() {
+    fn tree(&self) -> Result<&CodeTwoTreeReadFromDisk, RecoveryFailure> {
+        if let Some(read) = self.tree.get() {
             return Ok(read);
         }
-        let bytes = read_unit_via_locations(
-            self.reader,
-            &self.mapping_root.locations,
-            usize::try_from(NODE_BYTES).expect("16384"),
+        let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
+        let tree = read_code_two_tree(
+            &self.root.mapping_root,
+            &MultiLevelCodeTwoTree::CentralMapping
+                .read_expectation(self.root.mapping_root.head.birth_tree),
+            CodeTwoTreeHeaderJudgement::OnlyWhatTheShapeNeeds,
+            self.root,
+            unit_filesystem_identifier(&self.root.filesystem_identifier),
+            &mut |pointer: &NodePointer| {
+                read_unit_via_locations(self.reader, &pointer.locations, node_bytes)
+            },
         )?;
-        let node = parse_index_node(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
-            what: "映射树根",
-        })?;
-        Ok(self.bytes_and_node.get_or_init(|| (bytes, node)))
+        Ok(self.tree.get_or_init(|| tree))
     }
 
     fn locations_of_key(
         &self,
         mapping_key: &[u8],
     ) -> Result<Option<[LocationEntry; 2]>, RecoveryFailure> {
-        central_mapping_locations_among_entries(&self.bytes_and_node()?.1.entries, mapping_key)
+        central_mapping_locations_among_entries(
+            &self.tree()?.leaf_entries_in_key_order,
+            mapping_key,
+        )
     }
 
-    fn into_bytes_and_node(self) -> Result<(Vec<u8>, IndexNodeHeader), RecoveryFailure> {
-        self.bytes_and_node()?;
+    fn into_tree(self) -> Result<CodeTwoTreeReadFromDisk, RecoveryFailure> {
+        self.tree()?;
         Ok(self
-            .bytes_and_node
+            .tree
             .into_inner()
-            .expect("上一行刚把映射树根读进来，读不出已经返回了"))
+            .expect("上一行刚把映射树读进来，读不出已经返回了"))
     }
 }
 
@@ -635,12 +652,15 @@
     }
 }
 
-/// 三个区域全部槽逐个验自证校验和，取 `(checkpoint_txg, 实例代号)` 最大的。
+/// 三个区域全部槽逐个验自证校验和，先跳过被回退见证表抛弃的根（D23（journal 的角色与格式） 已定项 14「回退见证」：
+/// 择根先跳过被任一条目抛弃的根，再照 D22（单元原子性怎么合成） 已定项 7 择新），再取 `(checkpoint_txg, 实例代号)` 最大的。
+/// 见证表从盘上现读（[`rollback_witness_of_the_pool`]）：各盘择到的那一槽里的表取并集。
 #[must_use]
 pub fn choose_root(
     reader: &dyn PoolReader,
     system_configuration: &SystemConfiguration,
 ) -> Option<RootRecord> {
+    let rollback_witness = rollback_witness_of_the_pool(reader, system_configuration);
     let mut best: Option<RootRecord> = None;
     visit_valid_roots(
         reader,
@@ -648,6 +668,9 @@
         &system_configuration.immutable.sizes,
         &system_configuration.immutable.filesystem_identifier,
         |candidate| {
+            if rollback_witness.abandons(candidate.instance, candidate.checkpoint_txg) {
+                return;
+            }
             let candidate_key = (candidate.checkpoint_txg, candidate.instance);
             if best.is_none_or(|current| candidate_key > (current.checkpoint_txg, current.instance))
             {
@@ -658,6 +681,69 @@
     best
 }
 
+/// 一个池此刻的回退见证（D23（journal 的角色与格式） 已定项 14「回退见证」）：每块盘两槽里自证过（fsid 与本池相同）、
+/// 世代号最大的那一槽里的见证表，各盘取并集（`RollbackWitness`）。一块盘两槽都读不出就不算它；一槽都读不出时是空的——
+/// 那时系统配置本身就择不出来，挂载在择系统配置那一步已经报错（见证表读不出就是系统配置槽读不出）。
+#[must_use]
+pub fn rollback_witness_of_the_pool<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    system_configuration: &SystemConfiguration,
+) -> RollbackWitness {
+    let spacing = u64::from(
+        system_configuration
+            .immutable
+            .sizes
+            .fixed_structure_slot_spacing,
+    );
+    let chosen_on_each_device: Vec<SystemConfiguration> = reader
+        .device_identities()
+        .into_iter()
+        .filter_map(|device| {
+            verified_system_configuration_slots(
+                reader,
+                device,
+                spacing,
+                &system_configuration.immutable.filesystem_identifier,
+            )
+            .into_iter()
+            .max_by_key(|slot| slot.quantities.slot_generation)
+        })
+        .collect();
+    RollbackWitness::of_tables(
+        chosen_on_each_device
+            .iter()
+            .map(|chosen| &chosen.rollback_witness),
+    )
+}
+
+/// 根环每一个槽都读得出、都自证过（是这个池的一条根）时交回全部根，按区域、槽的次序；有一个槽读不出或自证不过就是 `None`。
+/// 回退见证的删除规则要它（`mount`）：读不出的槽按「可能有被抛弃的根」算，与 D18（块里携带什么信息） 已定项 11 行回收的根环条件同一个读法。
+#[must_use]
+pub fn every_root_ring_slot_holds_a_root<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    system_configuration: &SystemConfiguration,
+) -> Option<Vec<RootRecord>> {
+    let sizes = &system_configuration.immutable.sizes;
+    let root_slot_bytes = usize::try_from(sizes.physical_block_size).expect("根槽宽");
+    let mut roots = Vec::new();
+    for region in 0..ROOT_RING_REGIONS {
+        let device =
+            system_configuration.immutable.region_devices[usize::try_from(region).expect("区域号")];
+        for slot in 0..sizes.root_ring_slots_per_region.count() {
+            let offset = slot_offset(
+                RootRingSlot { region, slot },
+                sizes.fixed_structure_slot_spacing,
+            );
+            let bytes = reader.read(device, offset, root_slot_bytes)?;
+            roots.push(RootRecord::parse_slot(
+                &bytes,
+                &system_configuration.immutable.filesystem_identifier,
+            )?);
+        }
+    }
+    Some(roots)
+}
+
 /// 根环全部自证过的根里最大的 checkpoint_txg：新实例的第一次发布取 max(它, 环里全部自证通过的记录的 checkpoint_txg) + 1
 /// （D23（journal 的角色与格式） 已定项 14 第 3 条）；一条都没有时 None。
 #[must_use]
@@ -894,8 +980,7 @@
     let Some(allocation_pointer) = allocation_pointer else {
         return Ok(Vec::new());
     };
-    let central_mapping_root =
-        CentralMappingRootWithBytesReadOnFirstUse::new(reader, root.mapping_root);
+    let central_mapping_root = CentralMappingTreeWithBytesReadOnFirstUse::new(reader, root);
     let mut stale_location_hint_hops_not_exposed_by_this_reader = 0usize;
     let allocation_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
         reader,
@@ -1121,7 +1206,7 @@
 
 /// 从盘上按所选根重建「上一版」：全部角色的单元字节、指针、树表、分配记录、记账行与 inode 记录，交给发布路径当上一版
 /// （照抄没重写的角色、经映射释放被换下的角色都靠它；可写挂载在恢复之后调）。`record_standing_for_root` 是所选根覆盖的最后一条记录
-/// （同实例、同 checkpoint_txg 里 jsn 最大的那条，D23（journal 的角色与格式） 已定项 14 注 1；读不出时由调用方顶一条；树表 0 条时用不到）。
+/// （同实例、同 checkpoint_txg 里带「本次发布末条」标志的那一条，D23（journal 的角色与格式） 已定项 14 注 1 读法乙；读不出时由调用方顶一条；树表 0 条时用不到）。
 /// extent 根兼叶里的每条记录指的数据单元都读回来：一个文件跨多个单元时（并行线一）每个单元一个角色。
 /// 豁免三类之外的单元位置提示读不出时经这一版的中央映射回退（D19（块指针的结构与宽度预算） 已定项 8）；
 /// 提示与映射都读不出的**数据单元**不算失败：照抄它的位置项、不读内容，那一项的字节是空的（D19 已定项 5，用户 2026-09-24 定 N2）。
@@ -1195,8 +1280,7 @@
     // 豁免三类之外的单元（数据单元、四棵树的根、inode 叶容器）位置提示读不出时经这一版的中央映射回退，与冷走读同一条
     // （D19（块指针的结构与宽度预算） 已定项 8）；映射树根、树表、实例表是自举豁免，只按根记录里的位置条目读。
     // 映射树根到第一次要回退、或走到它那一步时才读：提示都读得出的镜像上读序与报错次序照旧。
-    let central_mapping_root =
-        CentralMappingRootWithBytesReadOnFirstUse::new(reader, root.mapping_root);
+    let central_mapping_root = CentralMappingTreeWithBytesReadOnFirstUse::new(reader, root);
     let central_mapping_locations_of_key =
         |mapping_key: &[u8]| central_mapping_root.locations_of_key(mapping_key);
     // 回退的次数这里不交出去：`RebuiltVersion` 与可写挂载今天没有接多跳观测点的口子（挂载态的读有，`MountedPoolForRead`）。
@@ -1329,14 +1413,28 @@
     )?;
     let allocation_records: Vec<AllocationRecord> =
         allocation_records_of_node(reader, &allocation_node)?;
-    let (accounting_bytes, accounting_node) = read_node(
+    // 记账树可以是多层（D8（核心索引结构） 已定项 11）：从根往下整棵读回来，节点进映射、提示读不出经这一版的中央映射回退
+    // （与读别的树根同一条）；中央映射树同样整棵读回来（下面）。两棵树按「拼得成一棵树」核（`code_two_tree::read_code_two_tree`）。
+    let expected_filesystem_identifier = unit_filesystem_identifier(&root.filesystem_identifier);
+    let accounting_tree = read_code_two_tree(
         &accounting_pointer,
-        "记账树根",
-        &mut stale_location_hint_hops_not_exposed_by_this_reader,
+        &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),
+        CodeTwoTreeHeaderJudgement::OnlyWhatTheShapeNeeds,
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
     let mut accounting_entries: Vec<AccountingEntry> =
-        Vec::with_capacity(accounting_node.entries.len());
-    for bytes in &accounting_node.entries {
+        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());
+    for bytes in &accounting_tree.leaf_entries_in_key_order {
         accounting_entries.push(AccountingEntry::parse(bytes).ok_or(
             RecoveryFailure::EntryNarrowerThanItsFieldTable {
                 what: "记账条目",
@@ -1353,10 +1451,11 @@
     {
         return Err(RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree.into());
     }
-    // 映射树根是自举豁免：不经映射回退，只按根记录里的位置条目读；上面有提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份。
-    let (mapping_bytes, mapping_node) = central_mapping_root.into_bytes_and_node()?;
-    let mut mapping_keys: Vec<Vec<u8>> = Vec::with_capacity(mapping_node.entries.len());
-    for entry in &mapping_node.entries {
+    // 映射树是自举豁免：不经映射回退，只按父指针里的位置条目读；上面有提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份。
+    let mapping_tree = central_mapping_root.into_tree()?;
+    let mut mapping_keys: Vec<Vec<u8>> =
+        Vec::with_capacity(mapping_tree.leaf_entries_in_key_order.len());
+    for entry in &mapping_tree.leaf_entries_in_key_order {
         let (key, _locations) =
             parse_mapping_entry(entry).ok_or(RecoveryFailure::EntryNarrowerThanItsFieldTable {
                 what: "映射条目",
@@ -1401,11 +1500,19 @@
             TransactionUnit::AllocationTree,
             mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer),
         ),
-        (
-            TransactionUnit::AccountingTree,
-            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, accounting_pointer),
-        ),
     ]);
+    // 记账树的每个节点都进映射（码 2 树节点，D19（块指针的结构与宽度预算） 已定项 8），按 bump 次序、根在最末。
+    let accounting_shape = &accounting_tree.version.shape;
+    for (node, pointer) in accounting_shape
+        .nodes()
+        .iter()
+        .zip(&accounting_tree.version.pointers)
+    {
+        mapped_units.push((
+            MultiLevelCodeTwoTree::Accounting.role_of_node(node.position, accounting_shape),
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
+        ));
+    }
     let unit =
         |identity: TransactionUnit, locations: &[LocationEntry; 2], bytes: Vec<u8>| PublishedUnit {
             slot: locations[0].slot,
@@ -1453,16 +1560,27 @@
             &allocation_pointer.locations,
             allocation_bytes,
         ),
-        unit(
-            TransactionUnit::AccountingTree,
-            &accounting_pointer.locations,
-            accounting_bytes,
-        ),
-        unit(
-            TransactionUnit::MappingTree,
-            &root.mapping_root.locations,
-            mapping_bytes,
-        ),
+    ]);
+    // 记账树与中央映射树的每个节点，按 bump 次序（树内先叶后根），与发布路径装出来的 `units` 同序。
+    for (tree, read) in [
+        (MultiLevelCodeTwoTree::Accounting, &accounting_tree),
+        (MultiLevelCodeTwoTree::CentralMapping, &mapping_tree),
+    ] {
+        let shape = &read.version.shape;
+        for ((node, pointer), bytes) in shape
+            .nodes()
+            .iter()
+            .zip(&read.version.pointers)
+            .zip(&read.node_bytes)
+        {
+            units.push(unit(
+                tree.role_of_node(node.position, shape),
+                &pointer.locations,
+                bytes.clone(),
+            ));
+        }
+    }
+    units.extend([
         unit(
             TransactionUnit::TreeTable,
             &root.tree_table.locations,
@@ -1486,6 +1604,8 @@
         mapping_keys,
         allocation_records,
         accounting_entries,
+        accounting_tree: accounting_tree.version,
+        central_mapping_tree: mapping_tree.version,
         tree_table_entries,
         tree_identifiers,
         inode_record,
@@ -1658,6 +1778,22 @@
     }
 }
 
+/// 同一 (实例代号, checkpoint_txg) 里 `record` 之后（计数器更大）还有没有读得出的记录。同一实例里计数器与 checkpoint_txg 一起往上走，
+/// 所以从下一个计数器起按计数器升序看到 txg 越过这一条的就停。
+fn a_readable_record_of_the_same_publish_follows(
+    record: &JournalRecord,
+    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
+) -> bool {
+    let Some(next_counter) = record.counter.checked_add(1) else {
+        return false;
+    };
+    records
+        .range((record.instance, next_counter)..=(record.instance, u64::MAX))
+        .map(|(_, later)| later)
+        .take_while(|later| later.checkpoint_txg <= record.checkpoint_txg)
+        .any(|later| later.checkpoint_txg == record.checkpoint_txg)
+}
+
 /// 所选根覆盖的最后一条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 14 注 1，读法乙，用户 2026-09-24 定）：
 /// 「那条」按末条标志认——所选根那次发布（与所选根同实例、同 checkpoint_txg）读得出的几条里带「本次发布末条」标志的那一条。
 /// 一条都不带 ⇒ `Ok(None)`，就算「那条读不出」，链首走「序号为 1 的第一条可读记录」那一支；**不取**读得出的同 txg 记录里
@@ -1698,7 +1834,8 @@
 /// [`counter_of_the_last_record_the_root_covers`]）。
 /// 施加的单位是一次发布（第六条）：前五条判出来的前缀里，一次发布的记录要一直走到带末条标志的那一条（[`record_ends_its_publish`]）
 /// 才整体施加；前缀停在一次发布中间（末条没到、断号、校验不过、回退行的 W 截在中间、下一条换了 txg、一次发布之内跳号、
-/// 一个事务的提交标记没出现）⇒ 那次发布整体不施加。
+/// 一个事务的提交标记没出现）⇒ 那次发布整体不施加。读者规则另外两格（D23（journal 的角色与格式） 已定项 4）同样断链、那次发布不施加：
+/// 一次发布的首条序号不是 1（断在这一条）；带末条标志的那一条之后同一 (实例代号, checkpoint_txg) 里还有读得出的记录（断在带标志的那一条）。
 ///
 /// # Errors
 /// 所选根那次发布读得出的记录里带末条标志的多于一条（锚点认哪一条条款没写）⇒
@@ -1711,6 +1848,10 @@
     verify_named_units: bool,
     rollback_high_water: Option<u64>,
 ) -> Result<(JournalScanReport, RootRecord), RecoveryFailure> {
+    // 被回退见证表抛弃的记录不施加（D23（journal 的角色与格式） 已定项 14「回退见证」随实现：见证表同时管择根与重放；
+    // 前缀第五条读见证的这一读法交代码三方）。见证表住系统配置槽里，读不出就是系统配置读不出，恢复报错、不按空表施加。
+    let system_configuration = choose_system_configuration(reader)?;
+    let rollback_witness = rollback_witness_of_the_pool(reader, &system_configuration);
     let mut report = JournalScanReport {
         valid_records: records.len(),
         above_water: 0,
@@ -1757,6 +1898,20 @@
         {
             break;
         }
+        // 被回退见证表抛弃的记录（所选根那个实例里 txg 越过回退目标的那一段）：它之后同一实例的记录只会更靠后，断在这里。
+        // 所选根被见证表抛弃的不会被择中（`choose_root`），所以这一判拦的是「所选根是 R_old 或更早、它之后的被抛弃记录还在环里」
+        // 那一格（C332（回退实例两个根都读不出时回退被撤销） 里落到 R_old 的那一支）。
+        if rollback_witness.abandons(record.instance, record.checkpoint_txg) {
+            break;
+        }
+        // 一次发布的第一条序号是 1（D23（journal 的角色与格式） 已定项 4 读者规则：锚点读得出时，下一次发布的首条序号不是 1，
+        // 当那条记录损坏、断在这一条，那次发布整体不施加；C539（锚点读得出时下一次发布的首条序号不是 1））。锚点读不出那一支上面已经判过；
+        // 这一判管锚点读得出时接上的那一次，与之后每一次新开的发布。
+        if records_of_the_open_publish.is_empty()
+            && record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST
+        {
+            break;
+        }
         // 前缀第五条：所选根的实例有回退行时只施加到回退行的 W 为止——W = 0 就是「之后的一个都不算」，
         // 空发布（事务号 0）也不许把根推过 T_old。
         if let Some(high_water) = rollback_high_water {
@@ -1793,6 +1948,15 @@
         if !record.is_commit && record_ends_its_publish(record) {
             break;
         }
+        // 带末条标志的这一条之后，同一 (实例代号, checkpoint_txg) 里还有读得出的记录（D23（journal 的角色与格式） 已定项 4 读者规则：
+        // 当这条记录损坏、断在带标志的这一条，那次发布整体不施加；C540（末条标志坏在一次发布中间，读者切出两次发布））。
+        // 合法历史里一次发布只有真正的最后一条带标志（已定项 17），失败的那次原样重发（已定项 14「这一版的失败处置」），走到这里要一条记录坏了而
+        // 校验和恰好仍对得上，或者镜像是改出来的。
+        if record_ends_its_publish(record)
+            && a_readable_record_of_the_same_publish_follows(record, records)
+        {
+            break;
+        }
         let all_verified = !verify_named_units
             || record.named.iter().all(|named| {
                 let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
@@ -1859,33 +2023,7 @@
     }
 }
 
-/// 读一棵**不进映射**的树的根节点（自举豁免三类里的中央映射树根：父指针里的位置条目是权威，
-/// D19（块指针的结构与宽度预算） 已定项 8）并核它的自描述，核法同 [`read_mapped_tree_root`]。
-/// 挂载态的读（`crate::mounted_read`）打开时走同一条，不另写一份。
-pub(crate) fn read_tree_root(
-    reader: &dyn PoolReader,
-    tree: TreeIdentifier,
-    key_width: usize,
-    pointer: &NodePointer,
-    root: &RootRecord,
-    expected_filesystem_identifier: u64,
-) -> Result<IndexNodeHeader, RecoveryFailure> {
-    let bytes = read_unit_via_locations(
-        reader,
-        &pointer.locations,
-        usize::try_from(NODE_BYTES).expect("16384"),
-    )?;
-    tree_root_checked_against_its_pointer(
-        &bytes,
-        tree,
-        key_width,
-        pointer,
-        root,
-        expected_filesystem_identifier,
-    )
-}
-
-/// 读一棵进映射的树的根节点（extent、inode、分配记录、记账）：位置提示读不出时经中央映射回退
+/// 读一棵进映射的树的根节点（extent、inode、分配记录；记账树多层，走 `code_two_tree::read_code_two_tree`）：位置提示读不出时经中央映射回退
 /// （[`read_mapped_tree_node_via_hint_then_central_mapping`]），读到之后核它的自描述：树 ID、key 宽、出生身份、fsid、
 /// key 区间与条目相符。挂载态的读（`crate::mounted_read`）打开时走同一条，不另写一份。
 #[allow(
@@ -2019,44 +2157,52 @@
 /// mkfs 写在单元区里的 2 个落点加第一个事务的 8 个落点（字节表五：20 条记录，每盘 10 条），之后每次发布只多不少。
 const FIRST_TRANSACTION_PLACEMENTS_PER_DEVICE: usize = 10;
 
-/// 冷走读里的中央映射树根（自举豁免，只按父指针里的位置条目读）：到第一次有树根的提示读不出、要经映射回退时，
+/// 冷走读里的中央映射树（自举豁免，只按父指针里的位置条目读）：到第一次有树根的提示读不出、要经映射回退时，
 /// 或走到 `TreeRoots` 那一步时才读，读过一次就留着。不提前读：提示都读得出的镜像上，走读的读序与判红次序照旧。
-struct CentralMappingRootReadOnFirstUse<'walk> {
+/// 映射树多层时整棵读回来（D19（块指针的结构与宽度预算） 已定项 5），每个节点按父条目核（`code_two_tree::read_code_two_tree`）。
+struct CentralMappingTreeReadOnFirstUse<'walk> {
     reader: &'walk dyn PoolReader,
     root: &'walk RootRecord,
     expected_filesystem_identifier: u64,
-    node: OnceCell<IndexNodeHeader>,
+    tree: OnceCell<CodeTwoTreeReadFromDisk>,
 }
 
-impl CentralMappingRootReadOnFirstUse<'_> {
-    fn node(&self) -> Result<&IndexNodeHeader, RecoveryFailure> {
-        if let Some(node) = self.node.get() {
-            return Ok(node);
-        }
-        let node = read_tree_root(
-            self.reader,
-            self.root.mapping_root.head.birth_tree,
-            usize::try_from(MAPPING_KEY_BYTES).expect("27"),
+impl CentralMappingTreeReadOnFirstUse<'_> {
+    fn tree(&self) -> Result<&CodeTwoTreeReadFromDisk, RecoveryFailure> {
+        if let Some(tree) = self.tree.get() {
+            return Ok(tree);
+        }
+        let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
+        let tree = read_code_two_tree(
             &self.root.mapping_root,
+            &MultiLevelCodeTwoTree::CentralMapping
+                .read_expectation(self.root.mapping_root.head.birth_tree),
+            CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,
             self.root,
             self.expected_filesystem_identifier,
+            &mut |pointer: &NodePointer| {
+                read_unit_via_locations(self.reader, &pointer.locations, node_bytes)
+            },
         )?;
-        Ok(self.node.get_or_init(|| node))
+        Ok(self.tree.get_or_init(|| tree))
     }
 
     fn locations_of_key(
         &self,
         mapping_key: &[u8],
     ) -> Result<Option<[LocationEntry; 2]>, RecoveryFailure> {
-        central_mapping_locations_among_entries(&self.node()?.entries, mapping_key)
+        central_mapping_locations_among_entries(
+            &self.tree()?.leaf_entries_in_key_order,
+            mapping_key,
+        )
     }
 
-    fn into_node(self) -> Result<IndexNodeHeader, RecoveryFailure> {
-        self.node()?;
+    fn into_tree(self) -> Result<CodeTwoTreeReadFromDisk, RecoveryFailure> {
+        self.tree()?;
         Ok(self
-            .node
+            .tree
             .into_inner()
-            .expect("上一行刚把映射树根读进来，读不出已经返回了"))
+            .expect("上一行刚把映射树读进来，读不出已经返回了"))
     }
 }
 
@@ -2064,8 +2210,8 @@
     extent: IndexNodeHeader,
     inode: IndexNodeHeader,
     allocation: IndexNodeHeader,
-    accounting: IndexNodeHeader,
-    mapping: IndexNodeHeader,
+    accounting: CodeTwoTreeReadFromDisk,
+    mapping: CodeTwoTreeReadFromDisk,
 }
 
 /// 沿树走到第一个文件：根记录 → 实例表 / 树表 → inode 树 → inode 记录 → extent 树 → 指针 → 数据单元。
@@ -2135,15 +2281,16 @@
             })?,
         );
     }
-    let central_mapping_root = CentralMappingRootReadOnFirstUse {
+    let central_mapping_tree = CentralMappingTreeReadOnFirstUse {
         reader,
         root,
         expected_filesystem_identifier,
-        node: OnceCell::new(),
+        tree: OnceCell::new(),
     };
     let central_mapping_locations_of_key =
-        |mapping_key: &[u8]| central_mapping_root.locations_of_key(mapping_key);
+        |mapping_key: &[u8]| central_mapping_tree.locations_of_key(mapping_key);
     let mut by_kind: BTreeMap<u16, IndexNodeHeader> = BTreeMap::new();
+    let mut accounting_tree: Option<CodeTwoTreeReadFromDisk> = None;
     for entry in &entries {
         if entry.tree.0 >= root.tree_identifier_watermark {
             return Err(RecoveryFailure::InvariantViolated {
@@ -2154,6 +2301,26 @@
         if entry.root == NodePointer::empty_root() {
             continue;
         }
+        // 记账树可以是多层（D8（核心索引结构） 已定项 11）：整棵读回来，每个节点按父条目核；节点进映射，提示读不出经映射回退。
+        if entry.kind == TREE_KIND_ACCOUNTING {
+            accounting_tree = Some(read_code_two_tree(
+                &entry.root,
+                &MultiLevelCodeTwoTree::Accounting.read_expectation(entry.tree),
+                CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,
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
         let key_width = key_width_for_kind(entry.kind).ok_or(RecoveryFailure::UnitMalformed {
             what: "树的种类没登记",
         })?;
@@ -2180,10 +2347,12 @@
         extent: take(TREE_KIND_EXTENT)?,
         inode: take(TREE_KIND_INODE)?,
         allocation: take(TREE_KIND_ALLOCATION)?,
-        accounting: take(TREE_KIND_ACCOUNTING)?,
+        accounting: accounting_tree.ok_or(RecoveryFailure::UnitMalformed {
+            what: "树表里缺一棵有根的树",
+        })?,
         // 中央映射树不进树表：它是哪棵树由根记录里它那条根指针的出生树说（第一个文件版本那次从水位发的号）。
         // 上面有树根的提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份，不再读一次。
-        mapping: central_mapping_root.into_node()?,
+        mapping: central_mapping_tree.into_tree()?,
     };
     let device_identities = reader.device_identities();
     let device_count = device_identities.len();
@@ -2197,21 +2366,24 @@
             detail: "分配记录不是每个落点每盘各一条：各盘的（槽, 跨度, 代, 已释放）集合不同，或少于 10 个落点",
         });
     }
-    if roots.accounting.entries.len() != 3 + 6 * device_count {
+    if roots.accounting.leaf_entries_in_key_order.len() != 3 + 6 * device_count {
         return Err(RecoveryFailure::InvariantViolated {
             invariant: "E142 走读同款",
             detail: "记账条目数不是 3 + 6 × 盘数",
         });
     }
-    // 进映射的单元：extent 根、inode 根、分配记录树、记账树各一条，加上 inode 树的每一片叶容器与文件的每一个数据单元
-    // （映射树自己、树表、实例表豁免，D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
+    // 进映射的单元：extent 根、inode 根、分配记录树各一条，记账树每个节点一条，加上 inode 树的每一片叶容器与文件的每一个数据单元
+    // （映射树自己的节点、树表、实例表豁免，D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
     // inode 树的叶容器数 = 它的根的条目数（根恒是层级 1，下面每条条目一片容器；层级检查在下面）；
     // 数据单元数 = extent 根兼叶的记录条数（第一版 extent 树只有一个节点，一条记录指一个数据单元）。
-    let mapping_entries_expected = 4 + roots.inode.entries.len() + roots.extent.entries.len();
-    if roots.mapping.entries.len() != mapping_entries_expected {
+    let mapping_entries_expected = 3
+        + roots.accounting.version.node_count()
+        + roots.inode.entries.len()
+        + roots.extent.entries.len();
+    if roots.mapping.leaf_entries_in_key_order.len() != mapping_entries_expected {
         return Err(RecoveryFailure::InvariantViolated {
             invariant: "E142 走读同款",
-            detail: "映射条目数不是 4 + inode 叶容器数 + extent 记录数",
+            detail: "映射条目数不是 3 + 记账树节点数 + inode 叶容器数 + extent 记录数",
         });
     }
     // 已释放的记录合法（D3（空间分配） 已定项 7：改写不删），它的代是释放代，同样不许晚于根。
@@ -2223,7 +2395,7 @@
             });
         }
     }
-    for entry_bytes in &roots.accounting.entries {
+    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {
         let entry = AccountingEntry::parse(entry_bytes).ok_or(
             RecoveryFailure::EntryNarrowerThanItsFieldTable {
                 what: "记账条目",
@@ -2266,7 +2438,10 @@
             &child,
             MappedTreeNodeClass::PackedRecordUnit,
             &|mapping_key: &[u8]| {
-                central_mapping_locations_among_entries(&roots.mapping.entries, mapping_key)
+                central_mapping_locations_among_entries(
+                    &roots.mapping.leaf_entries_in_key_order,
+                    mapping_key,
+                )
             },
             mapping_fallbacks,
         )?;
@@ -2368,7 +2543,10 @@
             reader,
             pointer,
             &|mapping_key: &[u8]| {
-                central_mapping_locations_among_entries(&roots.mapping.entries, mapping_key)
+                central_mapping_locations_among_entries(
+                    &roots.mapping.leaf_entries_in_key_order,
+                    mapping_key,
+                )
             },
             mapping_fallbacks,
         )?
diff -ruN -x target tree/crates/singlefs-core/src/rollback_witness.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-core/src/rollback_witness.rs	1970-01-01 00:00:00.000000000 +0000
+++ tree/crates/singlefs-core/src/rollback_witness.rs	2026-09-24 18:48:58.241559766 +0000
@@ -0,0 +1,314 @@
+//! 回退见证（D23（journal 的角色与格式） 已定项 14「回退见证」，C332（回退实例两个根都读不出时回退被撤销） 的修法，用户 2026-09-24 定）：
+//! 管理员回退在系统配置槽里记一张见证表，一个条目记一次回退——新实例代号 N、回退目标 R_old 的实例代号 r_old 与 txg T_old。
+//! 根 (i, T) 被条目 (N, r_old, T_old) 抛弃 ⟺ (r_old, T_old) < (i, T)（实例代号为主比）且 i < N；择根先跳过被任一条目抛弃的根。
+//!
+//! 盘上形态：系统配置槽内偏移 481 起（紧接字段表），条数 1 字节 + 47 个条目位 × 16 字节 = 753 字节定宽，罩在整槽校验和里、越过 512
+//! （`singlefs_format::ROLLBACK_WITNESS_*`）。一个池的条数上限 = 根环槽数减 1（R × S − 1，S 读自这个池的系统配置）。
+//! 条目按 (N, r_old, T_old) 升序排、不重复；条数之后的条目位全 0。读到别的样子就是这一槽读不出（见证表读不出就是系统配置槽读不出）。
+
+use std::collections::BTreeSet;
+
+use singlefs_format::{
+    ROLLBACK_WITNESS_COUNT_BYTES, ROLLBACK_WITNESS_ENTRIES_MAXIMUM, ROLLBACK_WITNESS_ENTRY_BYTES,
+    ROLLBACK_WITNESS_TABLE_BYTES, ROOT_RING_REGIONS,
+};
+
+use crate::address::{CheckpointTxg, InstanceGeneration};
+use crate::root_ring::RootRingSlotsPerRegion;
+
+/// 见证表定宽的条目位数（47）。数组长度要一个编译期的 `usize`，`try_from` 在常量里用不了。
+#[allow(
+    clippy::cast_possible_truncation,
+    reason = "47 装得进任何宽度的 usize；下一句编译期断言钉住没丢值"
+)]
+const ENTRY_POSITIONS: usize = ROLLBACK_WITNESS_ENTRIES_MAXIMUM as usize;
+const _: () = assert!(ENTRY_POSITIONS as u64 == ROLLBACK_WITNESS_ENTRIES_MAXIMUM);
+
+/// 一次回退的见证：回退那一次挂载取的新实例代号，与管理员选的回退目标 R_old。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub struct RollbackWitnessEntry {
+    pub new_instance: InstanceGeneration,
+    pub rollback_target_instance: InstanceGeneration,
+    pub rollback_target_txg: CheckpointTxg,
+}
+
+impl RollbackWitnessEntry {
+    /// 条目位全 0 的那个值（没用上的条目位写它）。
+    const UNUSED_POSITION: Self = Self {
+        new_instance: InstanceGeneration(0),
+        rollback_target_instance: InstanceGeneration(0),
+        rollback_target_txg: CheckpointTxg(0),
+    };
+
+    /// (实例代号, txg) 这一处（一条根，或一条 journal 记录的 (实例代号, checkpoint_txg)）被这次回退抛弃：
+    /// (r_old, T_old) < (i, T)，按实例代号为主比，且 i < N。
+    #[must_use]
+    pub fn abandons(&self, instance: InstanceGeneration, checkpoint_txg: CheckpointTxg) -> bool {
+        (self.rollback_target_instance, self.rollback_target_txg) < (instance, checkpoint_txg)
+            && instance < self.new_instance
+    }
+}
+
+/// 一个池的见证表条数上限：根环槽数减 1（R × S − 1），S 是这个池系统配置里的每区槽数。
+#[must_use]
+pub fn rollback_witness_capacity(root_ring_slots_per_region: RootRingSlotsPerRegion) -> usize {
+    usize::try_from(ROOT_RING_REGIONS * root_ring_slots_per_region.count() - 1)
+        .expect("R × S − 1 至多 47")
+}
+
+/// 一张见证表：按 (N, r_old, T_old) 升序、不重复，条数不超过定宽的 47。`Copy`：它是系统配置槽内容的一部分，跟着系统配置按值传。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct RollbackWitnessTable {
+    positions: [RollbackWitnessEntry; ENTRY_POSITIONS],
+    count: usize,
+}
+
+/// 见证表装不下：条目数越过了这个池的上限（`capacity`）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct RollbackWitnessTableFull {
+    pub entries: usize,
+    pub capacity: usize,
+}
+
+impl RollbackWitnessTable {
+    /// 一条条目都没有的表：mkfs 写它，从没回退过的池一直是它（盘上那 753 字节全 0）。
+    pub const EMPTY: Self = Self {
+        positions: [RollbackWitnessEntry::UNUSED_POSITION; ENTRY_POSITIONS],
+        count: 0,
+    };
+
+    /// 由一组条目装一张表（排序、去重）。
+    ///
+    /// # Errors
+    /// 去重之后多于 `capacity` 条（`capacity` 自己不超过定宽的 47）。
+    pub fn of_entries(
+        entries: impl IntoIterator<Item = RollbackWitnessEntry>,
+        capacity: usize,
+    ) -> Result<Self, RollbackWitnessTableFull> {
+        let sorted: BTreeSet<RollbackWitnessEntry> = entries.into_iter().collect();
+        let capacity = capacity.min(ENTRY_POSITIONS);
+        if sorted.len() > capacity {
+            return Err(RollbackWitnessTableFull {
+                entries: sorted.len(),
+                capacity,
+            });
+        }
+        let mut table = Self::EMPTY;
+        table.count = sorted.len();
+        for (position, entry) in sorted.into_iter().enumerate() {
+            table.positions[position] = entry;
+        }
+        Ok(table)
+    }
+
+    #[must_use]
+    pub fn entries(&self) -> &[RollbackWitnessEntry] {
+        &self.positions[..self.count]
+    }
+
+    /// (实例代号, txg) 这一处被表里任一条目抛弃。
+    #[must_use]
+    pub fn abandons(&self, instance: InstanceGeneration, checkpoint_txg: CheckpointTxg) -> bool {
+        self.entries()
+            .iter()
+            .any(|entry| entry.abandons(instance, checkpoint_txg))
+    }
+
+    /// 盘上的 753 字节：条数 1 字节，之后 47 个条目位，每位 新实例代号 4 + R_old 实例代号 4 + R_old txg 8（小端），没用上的位全 0。
+    #[must_use]
+    pub fn to_bytes(&self) -> Vec<u8> {
+        let mut bytes =
+            Vec::with_capacity(usize::try_from(ROLLBACK_WITNESS_TABLE_BYTES).expect("753"));
+        bytes.push(u8::try_from(self.count).expect("条数至多 47"));
+        for entry in &self.positions {
+            bytes.extend_from_slice(&entry.new_instance.0.to_le_bytes());
+            bytes.extend_from_slice(&entry.rollback_target_instance.0.to_le_bytes());
+            bytes.extend_from_slice(&entry.rollback_target_txg.0.to_le_bytes());
+        }
+        bytes
+    }
+
+    /// 读者：753 字节按上面的样子解。条数不超过这个池的上限（`capacity`）、条目按 (N, r_old, T_old) 严格升序、每一条 r_old < N、
+    /// 条数之后的条目位全 0——任一不满足就是 `None`（这一槽的见证表读不出，等于这一槽读不出）。
+    #[must_use]
+    pub fn parse(bytes: &[u8], capacity: usize) -> Option<Self> {
+        if bytes.len() < usize::try_from(ROLLBACK_WITNESS_TABLE_BYTES).expect("753") {
+            return None;
+        }
+        let count = usize::from(bytes[0]);
+        if count > capacity.min(ENTRY_POSITIONS) {
+            return None;
+        }
+        let count_bytes = usize::try_from(ROLLBACK_WITNESS_COUNT_BYTES).expect("1");
+        let entry_bytes = usize::try_from(ROLLBACK_WITNESS_ENTRY_BYTES).expect("16");
+        let mut table = Self::EMPTY;
+        for position in 0..ENTRY_POSITIONS {
+            let start = count_bytes + position * entry_bytes;
+            let field =
+                |offset: usize, width: usize| &bytes[start + offset..start + offset + width];
+            let entry = RollbackWitnessEntry {
+                new_instance: InstanceGeneration(u32::from_le_bytes(
+                    field(0, 4).try_into().expect("4 字节"),
+                )),
+                rollback_target_instance: InstanceGeneration(u32::from_le_bytes(
+                    field(4, 4).try_into().expect("4 字节"),
+                )),
+                rollback_target_txg: CheckpointTxg(u64::from_le_bytes(
+                    field(8, 8).try_into().expect("8 字节"),
+                )),
+            };
+            if position < count {
+                let follows_the_previous = position == 0 || table.positions[position - 1] < entry;
+                if entry.rollback_target_instance >= entry.new_instance || !follows_the_previous {
+                    return None;
+                }
+                table.positions[position] = entry;
+            } else if entry != RollbackWitnessEntry::UNUSED_POSITION {
+                return None;
+            }
+        }
+        table.count = count;
+        Some(table)
+    }
+}
+
+/// 一个池此刻的见证：各盘择到的那一槽（两槽里自证过、世代号最大的）里的见证表取并集。
+/// 各盘在一次轮换写的中途崩了会一新一旧，取并集才不丢那一条；各盘两槽里旧的那一槽不并进来——挂载删掉的条目留在旧槽里，
+/// 并进来就删不掉（它只在择到的那一槽读不出、退回旧槽时回来，那时它已经抛弃不了环里的任何根，见 `mount` 的删除规则）。
+#[derive(Clone, Debug, Default, PartialEq, Eq)]
+pub struct RollbackWitness {
+    entries: BTreeSet<RollbackWitnessEntry>,
+}
+
+impl RollbackWitness {
+    /// 把几张表并进来。
+    #[must_use]
+    pub fn of_tables<'table>(
+        tables: impl IntoIterator<Item = &'table RollbackWitnessTable>,
+    ) -> Self {
+        Self {
+            entries: tables
+                .into_iter()
+                .flat_map(|table| table.entries().iter().copied())
+                .collect(),
+        }
+    }
+
+    #[must_use]
+    pub fn entries(&self) -> Vec<RollbackWitnessEntry> {
+        self.entries.iter().copied().collect()
+    }
+
+    /// (实例代号, txg) 这一处被任一条目抛弃。
+    #[must_use]
+    pub fn abandons(&self, instance: InstanceGeneration, checkpoint_txg: CheckpointTxg) -> bool {
+        self.entries
+            .iter()
+            .any(|entry| entry.abandons(instance, checkpoint_txg))
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn entry(new_instance: u32, target_instance: u32, target_txg: u64) -> RollbackWitnessEntry {
+        RollbackWitnessEntry {
+            new_instance: InstanceGeneration(new_instance),
+            rollback_target_instance: InstanceGeneration(target_instance),
+            rollback_target_txg: CheckpointTxg(target_txg),
+        }
+    }
+
+    /// 已定项 14「回退见证」的判法：(r_old, T_old) < (i, T)（实例代号为主比）且 i < N。回退到 (1, 3)、新实例 4：
+    /// 实例 1 里 txg 3 之后的根、实例 2 与 3 的全部根被抛弃；R_old 自己、它之前的根、新实例及之后的根不被抛弃。
+    #[test]
+    fn an_entry_abandons_exactly_the_roots_after_the_rollback_target_and_before_the_new_instance() {
+        let rollback = entry(4, 1, 3);
+        for (instance, txg, abandoned) in [
+            (1, 2, false),
+            (1, 3, false),
+            (1, 4, true),
+            (1, 9, true),
+            (2, 1, true),
+            (3, 12, true),
+            (4, 5, false),
+            (5, 1, false),
+            (0, 0, false),
+        ] {
+            assert_eq!(
+                rollback.abandons(InstanceGeneration(instance), CheckpointTxg(txg)),
+                abandoned,
+                "({instance}, {txg})"
+            );
+        }
+    }
+
+    /// 盘上 753 字节来回一趟不变；条目位按 (N, r_old, T_old) 升序排、去重；没用上的位全 0。
+    #[test]
+    fn a_table_round_trips_through_its_753_bytes_sorted_and_deduplicated() {
+        let table = RollbackWitnessTable::of_entries(
+            [entry(7, 2, 10), entry(4, 1, 3), entry(7, 2, 10)],
+            23,
+        )
+        .expect("两条装得下");
+        assert_eq!(table.entries(), &[entry(4, 1, 3), entry(7, 2, 10)]);
+        let bytes = table.to_bytes();
+        assert_eq!(bytes.len(), 753);
+        assert_eq!(bytes[0], 2);
+        assert!(
+            bytes[33..].iter().all(|byte| *byte == 0),
+            "条数之后的条目位全 0"
+        );
+        assert_eq!(RollbackWitnessTable::parse(&bytes, 23), Some(table));
+        assert_eq!(
+            RollbackWitnessTable::parse(&RollbackWitnessTable::EMPTY.to_bytes(), 23),
+            Some(RollbackWitnessTable::EMPTY),
+            "全 0 的 753 字节就是空表"
+        );
+    }
+
+    /// 读者的四格都当这一槽的见证表读不出：条数超过这个池的上限、条目不升序、r_old 不小于 N、条数之后的条目位不是 0。
+    #[test]
+    fn a_count_beyond_the_capacity_an_unsorted_entry_a_target_not_below_the_new_instance_or_a_nonzero_unused_position_is_unreadable(
+    ) {
+        let two = RollbackWitnessTable::of_entries([entry(4, 1, 3), entry(7, 2, 10)], 23)
+            .expect("两条")
+            .to_bytes();
+        assert!(
+            RollbackWitnessTable::parse(&two, 1).is_none(),
+            "条数 2 超过上限 1"
+        );
+        let mut unsorted = two.clone();
+        unsorted[1..17].copy_from_slice(&two[17..33]);
+        unsorted[17..33].copy_from_slice(&two[1..17]);
+        assert!(
+            RollbackWitnessTable::parse(&unsorted, 23).is_none(),
+            "不升序"
+        );
+        let mut target_not_below = two.clone();
+        target_not_below[5..9].copy_from_slice(&4u32.to_le_bytes());
+        assert!(
+            RollbackWitnessTable::parse(&target_not_below, 23).is_none(),
+            "r_old 不小于 N"
+        );
+        let mut trailing = two;
+        trailing[40] = 1;
+        assert!(
+            RollbackWitnessTable::parse(&trailing, 23).is_none(),
+            "条数之后的条目位不是 0"
+        );
+    }
+
+    /// 装不下就报条数与上限，不截断。
+    #[test]
+    fn more_entries_than_the_capacity_are_refused_not_truncated() {
+        assert_eq!(
+            RollbackWitnessTable::of_entries([entry(4, 1, 3), entry(7, 2, 10)], 1),
+            Err(RollbackWitnessTableFull {
+                entries: 2,
+                capacity: 1
+            })
+        );
+    }
+}
diff -ruN -x target tree/crates/singlefs-core/src/system_configuration.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-core/src/system_configuration.rs	2026-09-23 08:50:42.277021530 +0000
+++ tree/crates/singlefs-core/src/system_configuration.rs	2026-09-24 18:48:58.241568846 +0000
@@ -9,6 +9,7 @@
 use singlefs_format::{
     journal_in_flight_record_limit, DATA_UNIT_BYTES, FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
     JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, JOURNAL_SAFETY_FACTOR, LOC_ENTRY, NODE_BYTES,
+    ROLLBACK_WITNESS_TABLE_BYTES, ROLLBACK_WITNESS_TABLE_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT,
     ROOT_RING_BASE_SLOT, ROOT_RING_CHUNK_BYTES, ROOT_RING_PRIME_STEP, ROOT_RING_REGIONS,
     SLOT_BYTES, SYSTEM_CONFIGURATION_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES, UNIT_AREA_START_SLOT,
     WIDE_CHECKSUM_BYTES,
@@ -17,6 +18,7 @@
 use crate::address::{DeviceIdentity, InstanceGeneration};
 use crate::bytes::{ByteReader, ByteWriter};
 use crate::checksum::{wide_checksum_field_holds, wide_checksum_with_field_zeroed};
+use crate::rollback_witness::{rollback_witness_capacity, RollbackWitnessTable};
 use crate::root_ring::{RootRingSlotsPerRegion, RootRingSlotsPerRegionOutOfRange};
 
 pub const SYSTEM_CONFIGURATION_MAGIC: [u8; 4] = *b"SFSB";
@@ -194,13 +196,16 @@
     pub const FIELD_TABLE_BYTES: u64 = 52;
 }
 
-/// 一个系统配置槽的内容，按 D22（单元原子性怎么合成） 已定项 26 的可改性分四类装。
+/// 一个系统配置槽的内容，按 D22（单元原子性怎么合成） 已定项 26 的可改性分四类装，另加字段表之后的回退见证表。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct SystemConfiguration {
     pub immutable: SystemImmutableConfiguration,
     pub mutable: SystemMutableConfiguration,
     pub runtime: SystemRuntimeConfiguration,
     pub quantities: SystemRuntimeQuantities,
+    /// 回退见证表（D23（journal 的角色与格式） 已定项 14「回退见证」）：住字段表之后、槽内偏移 481 起的 753 字节，
+    /// 不在 481 字节的字段表与四档可改性里——它不是配置，是文件系统自己维护的、随每一次系统配置写整张带着的表。
+    pub rollback_witness: RollbackWitnessTable,
 }
 
 fn slot_bytes() -> usize {
@@ -389,6 +394,13 @@
             writer.put_u32(self.quantities.journal_instance.0);
         });
         let (mut bytes, accounting) = slot.finish();
+        // 回退见证表紧接字段表（D23（journal 的角色与格式） 已定项 14「回退见证」）：越过 512、罩在下面那个整槽校验和里。
+        let witness_start =
+            usize::try_from(ROLLBACK_WITNESS_TABLE_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT)
+                .expect("481");
+        let witness_end =
+            witness_start + usize::try_from(ROLLBACK_WITNESS_TABLE_BYTES).expect("753");
+        bytes[witness_start..witness_end].copy_from_slice(&self.rollback_witness.to_bytes());
         let digest = wide_checksum_with_field_zeroed(
             &bytes,
             slot_bytes(),
@@ -420,6 +432,15 @@
             u64::from(bytes[usize::try_from(ROOT_RING_SLOTS_PER_REGION_OFFSET).expect("362")]),
         )
         .map_err(SystemConfigurationSlotRefusal::RootRingSlotsPerRegionOutOfRange)?;
+        // 见证表读不出就是这一槽读不出（D23（journal 的角色与格式） 已定项 14「回退见证」）：条数上限按这一槽自述的 S 算。
+        let witness_start =
+            usize::try_from(ROLLBACK_WITNESS_TABLE_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT)
+                .expect("481");
+        let rollback_witness = RollbackWitnessTable::parse(
+            &bytes[witness_start..],
+            rollback_witness_capacity(root_ring_slots_per_region),
+        )
+        .ok_or(SystemConfigurationSlotRefusal::NotSelfDescribing)?;
         let mut reader = ByteReader::at(bytes, FSID_OFFSET);
         let filesystem_identifier: [u8; 16] = reader.take(16).try_into().expect("切了 16 字节");
         reader.skip(16 + 4 + 1);
@@ -468,6 +489,7 @@
                 journal_tail,
                 journal_instance,
             },
+            rollback_witness,
         })
     }
 }
@@ -476,7 +498,8 @@
 /// 错误成员按调用方要做的决定分）。封闭集合，`match` 不写通配臂。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum SystemConfigurationSlotRefusal {
-    /// magic、整槽校验和、incompat 位三关里有一关不过 ⇒ **这一槽不可择**：换一槽、换一盘还可以试，
+    /// magic、整槽校验和、incompat 位三关里有一关不过，或字段表之后的回退见证表读不出（D23（journal 的角色与格式） 已定项 14
+    /// 「回退见证」：见证表读不出就是系统配置槽读不出）⇒ **这一槽不可择**：换一槽、换一盘还可以试，
     /// 系统配置每盘两槽、池里每盘一份买的就是这份冗余（D22（单元原子性怎么合成） 已定项 8 第 1 条）。
     NotSelfDescribing,
     /// 自述的每区槽数 S 落在格式承诺的区间之外 ⇒ **整池拒绝挂载**，不换一槽再试：S 是池级字段，
@@ -522,9 +545,58 @@
                 journal_tail: 0,
                 journal_instance: InstanceGeneration(0),
             },
+            rollback_witness: RollbackWitnessTable::EMPTY,
         }
     }
 
+    /// 回退见证表住字段表之后（偏移 481 起）、越过 512，跟着系统配置来回一趟不变；见证表读不出（条数超过这个池的上限 R × S − 1，
+    /// 整槽校验和照样重封过）就是这一槽读不出（D23（journal 的角色与格式） 已定项 14「回退见证」）。
+    #[test]
+    fn the_rollback_witness_rides_after_the_field_table_past_512_and_an_unreadable_one_makes_the_slot_unreadable(
+    ) {
+        use crate::address::CheckpointTxg;
+        use crate::rollback_witness::RollbackWitnessEntry;
+        let mut with_witness = sample();
+        with_witness.rollback_witness = RollbackWitnessTable::of_entries(
+            [
+                RollbackWitnessEntry {
+                    new_instance: InstanceGeneration(4),
+                    rollback_target_instance: InstanceGeneration(1),
+                    rollback_target_txg: CheckpointTxg(3),
+                },
+                RollbackWitnessEntry {
+                    new_instance: InstanceGeneration(6),
+                    rollback_target_instance: InstanceGeneration(4),
+                    rollback_target_txg: CheckpointTxg(9),
+                },
+            ],
+            23,
+        )
+        .expect("两条装得下");
+        let slot = with_witness.to_slot();
+        assert_eq!(slot[481], 2, "条数紧接字段表");
+        assert_eq!(
+            u64::from_le_bytes(slot[506..514].try_into().expect("8 字节")),
+            9,
+            "第二条占 [498, 514)，它的 txg 那 8 字节落在 [506, 514)：越过 512"
+        );
+        assert_eq!(SystemConfiguration::parse_slot(&slot), Ok(with_witness));
+        let mut unreadable = slot;
+        unreadable[481] = 24;
+        let digest = wide_checksum_with_field_zeroed(
+            &unreadable,
+            4096,
+            SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
+        );
+        unreadable[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
+            .copy_from_slice(&digest);
+        assert_eq!(
+            SystemConfiguration::parse_slot(&unreadable),
+            Err(SystemConfigurationSlotRefusal::NotSelfDescribing),
+            "S = 8 的池条数上限 23：见证表读不出，这一槽读不出"
+        );
+    }
+
     #[test]
     fn system_configuration_is_481_bytes_in_a_4096_slot_and_round_trips() {
         let slot = sample().to_slot();
diff -ruN -x target tree/crates/singlefs-core/src/transaction.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-core/src/transaction.rs	2026-09-24 16:15:19.532814592 +0000
+++ tree/crates/singlefs-core/src/transaction.rs	2026-09-24 18:48:58.241580116 +0000
@@ -7,18 +7,17 @@
 //! 被换下的八个单元在同一次发布里释放（分配记录改写成已释放 + 释放代，条目不删，D3（空间分配） 已定项 7）。
 //! 每一步都经过块设备接口，录制器挂在那层（D17（实现分层与第三方管道） 已定项 5）。
 
-use std::collections::BTreeMap;
+use std::collections::{BTreeMap, BTreeSet};
 
 use singlefs_format::{
-    ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, ALLOCATION_RECORD_BYTES,
-    ALLOCATION_RECORD_KEY_BYTES, EXTENT_KEY_BYTES, EXTENT_LEAF_RECORD_BYTES, INODE_INTERNAL_ENTRY,
-    INODE_RECORD_BYTES, INSTANCE_ROW_BYTES, JOURNAL_NAMED_ENTRIES_PER_RECORD, MAPPING_ENTRY_BYTES,
-    MAPPING_KEY_BYTES, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
-    TREE_IDENTIFIER_ACCOUNTING, TREE_IDENTIFIER_ALLOCATION_RECORDS,
-    TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST, TREE_IDENTIFIER_EXTENT,
-    TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LIVELIST, TREE_IDENTIFIER_SPARSE_SIDE_TABLE,
-    TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, TREE_IDENTIFIER_WATERMARK_AT_MKFS,
-    TREE_TABLE_ENTRY_BYTES, WARM_UP_EMPTY_PUBLISHES,
+    ACCOUNTING_ENTRY_BYTES, ALLOCATION_RECORD_BYTES, ALLOCATION_RECORD_KEY_BYTES, EXTENT_KEY_BYTES,
+    EXTENT_LEAF_RECORD_BYTES, INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES, INSTANCE_ROW_BYTES,
+    JOURNAL_NAMED_ENTRIES_PER_RECORD, MAPPING_ENTRY_BYTES, SLOT_BYTES,
+    SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, TREE_IDENTIFIER_ACCOUNTING,
+    TREE_IDENTIFIER_ALLOCATION_RECORDS, TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST,
+    TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LIVELIST,
+    TREE_IDENTIFIER_SPARSE_SIDE_TABLE, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
+    TREE_IDENTIFIER_WATERMARK_AT_MKFS, TREE_TABLE_ENTRY_BYTES, WARM_UP_EMPTY_PUBLISHES,
 };
 
 use crate::address::{
@@ -30,6 +29,13 @@
 };
 use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
 use crate::checksum::crc32_castagnoli;
+use crate::code_two_tree::{
+    build_internal_entry, internal_entry_width_in_bytes, leaf_position_routed_to,
+    plan_the_tree_after_this_publish, CodeTwoKeyFieldWidths, CodeTwoTreeKey,
+    CodeTwoTreeNodeCapacity, CodeTwoTreeNodeContents, CodeTwoTreeNodeOrigin,
+    CodeTwoTreeNodePosition, CodeTwoTreePlan, CodeTwoTreeReadExpectation, CodeTwoTreeRefusal,
+    CodeTwoTreeShape, CodeTwoTreeVersion,
+};
 use crate::inode_tree::{
     write_records_into_leaf_containers, InodeLeafContainer, InodeLeafContainerIndexInTree,
     InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal,
@@ -51,10 +57,9 @@
 };
 use crate::records::{
     build_extent_record, build_inode_internal_entry, build_mapping_entry, data_key_tail,
-    mapping_key_for_data, mapping_key_for_node, mapping_key_sort_key, node_key_tail,
-    parse_mapping_entry, AccountingEntry, InodeRecord, TreeTableEntry,
-    ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF, STATISTIC_ALLOCATED_BYTES,
-    STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_DEFER_QUEUE_BYTES,
+    mapping_key_for_data, mapping_key_for_node, node_key_tail, parse_mapping_entry,
+    AccountingEntry, InodeRecord, TreeTableEntry, ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
+    STATISTIC_ALLOCATED_BYTES, STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_DEFER_QUEUE_BYTES,
     STATISTIC_EMPTY_CLUSTER_SEGMENTS, STATISTIC_FRAGMENTATION_RUNS, STATISTIC_FREE_BYTES,
     STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION, STATISTIC_PENDING_DELETE_BYTES,
     STATISTIC_UNRECLAIMABLE_BYTES, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
@@ -64,6 +69,7 @@
     highest_root_instance, tree_table_entry_count, verified_system_configuration_slots, PoolReader,
     RecoveryFailure,
 };
+use crate::rollback_witness::RollbackWitnessTable;
 use crate::root_record::RootRecord;
 use crate::root_ring::{slot_offset, target_for_publish};
 use crate::system_configuration::{
@@ -113,6 +119,45 @@
     Barrier,
 }
 
+/// 一条 journal 记录装几个点名项，只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支必须能被测试强制进入）。
+/// 末条再跨记录（D23（journal 的角色与格式） 已定项 17）在产品路径上要一次发布点名 68 项以上才走得到——树表 0 条的一版上写行要实例表长到
+/// 67 片（两万四千多行），带文件的一版要六十几片 inode 叶容器；压小这个数就能用几个单元造出跨记录的发布。产品路径恒 `FromTheRecordFormat`。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum JournalRecordNamedEntryCapacity {
+    /// 记录格式装得下的：(4096 − 311) ÷ 56 = 67 项（`JOURNAL_NAMED_ENTRIES_PER_RECORD`）。
+    FromTheRecordFormat,
+    /// 压到这么多项（1..=67）。
+    CappedForTests { named_entries_per_record: usize },
+}
+
+impl JournalRecordNamedEntryCapacity {
+    /// 一条记录装几个点名项。
+    #[must_use]
+    pub fn named_entries_per_record(self) -> usize {
+        match self {
+            JournalRecordNamedEntryCapacity::FromTheRecordFormat => {
+                usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("67")
+            }
+            JournalRecordNamedEntryCapacity::CappedForTests {
+                named_entries_per_record,
+            } => named_entries_per_record,
+        }
+    }
+
+    /// 走的是哪一臂（五条硬要求第 4 条：分支必须可观测）。
+    #[must_use]
+    pub const fn branch_name(self) -> &'static str {
+        match self {
+            JournalRecordNamedEntryCapacity::FromTheRecordFormat => {
+                "journal_record_named_entries=from_the_record_format"
+            }
+            JournalRecordNamedEntryCapacity::CappedForTests { .. } => {
+                "journal_record_named_entries=capped_for_tests"
+            }
+        }
+    }
+}
+
 /// 一个池的写入口：几块盘 + mkfs 参数（几何与区域归属都从这里取）。
 pub struct PoolWriter<'pool, Device: BlockDevice> {
     pub parameters: &'pool MakeFilesystemParameters,
@@ -125,6 +170,13 @@
     /// 落盘阶段中途失败的发布各自已记的写，一次失败一份，按失败的先后排（增补 2 第 20b 行）。成功发布的账是两次快照之差、只在成功路径上取，
     /// 失败那次落盘的写不属于任何一次成功发布 ⇒ 不交出去就与设备一层的合计对不上。另立一份、不并进任何一次成功发布的账。
     writes_of_failed_publishes: Vec<WritesByStructureKind>,
+    /// 记账树与中央映射树的节点容量，只供测试的开关（`CodeTwoTreeNodeCapacities`）；新开的写入口按产品路径起步。
+    code_two_tree_node_capacities: CodeTwoTreeNodeCapacities,
+    /// 一条 journal 记录装几个点名项，只供测试的开关（[`JournalRecordNamedEntryCapacity`]）；新开的写入口按记录格式起步。
+    journal_record_named_entry_capacity: JournalRecordNamedEntryCapacity,
+    /// 这个写入口之后的每一次系统配置写带哪一张回退见证表（D23（journal 的角色与格式） 已定项 14「回退见证」）：
+    /// `None` 照抄那块盘上择到的那一槽里的（新开的写入口按它起步，不读盘）；可写挂载与回退按删除规则与这次的回退定好之后装上（`mount`）。
+    rollback_witness_to_write: Option<RollbackWitnessTable>,
 }
 
 impl<'pool, Device: BlockDevice> PoolWriter<'pool, Device> {
@@ -139,11 +191,71 @@
             has_writes_since_barrier: true,
             writes_by_structure_kind: WritesByStructureKind::NOTHING_WRITTEN,
             writes_of_failed_publishes: Vec::new(),
+            code_two_tree_node_capacities: CodeTwoTreeNodeCapacities::FromTheNodeFormat,
+            journal_record_named_entry_capacity:
+                JournalRecordNamedEntryCapacity::FromTheRecordFormat,
+            rollback_witness_to_write: None,
         }
     }
 }
 
 impl<Device: BlockDevice> PoolWriter<'_, Device> {
+    /// 装上节点容量这个只供测试的开关（`CodeTwoTreeNodeCapacities`）。产品路径一处都不调它。
+    ///
+    /// # Panics
+    /// 压的容量大于格式算出来的（节点装不下，走到 `build_index_node` 的断言），或叶小于 1 条、内部节点小于 2 个孩子
+    /// （切出来的两半要都不空、根分裂出来的新根要装得下两个孩子）：开关给错了。
+    pub fn set_code_two_tree_node_capacities(&mut self, capacities: CodeTwoTreeNodeCapacities) {
+        for tree in [
+            MultiLevelCodeTwoTree::Accounting,
+            MultiLevelCodeTwoTree::CentralMapping,
+        ] {
+            let capped = capacities.of_tree(tree);
+            let of_the_format = tree.node_capacity_of_the_node_format();
+            assert!(
+                (1..=of_the_format.leaf_entries).contains(&capped.leaf_entries)
+                    && (2..=of_the_format.internal_entries).contains(&capped.internal_entries),
+                "{tree:?} 的容量压到 {capped:?}：叶要在 1..={}、内部节点要在 2..={} 之内",
+                of_the_format.leaf_entries,
+                of_the_format.internal_entries
+            );
+        }
+        self.code_two_tree_node_capacities = capacities;
+    }
+
+    /// 这会儿装着的节点容量：运行时看得出走的是哪一条分支。
+    #[must_use]
+    pub fn code_two_tree_node_capacities(&self) -> CodeTwoTreeNodeCapacities {
+        self.code_two_tree_node_capacities
+    }
+
+    /// 装上「一条 journal 记录装几个点名项」这个只供测试的开关（[`JournalRecordNamedEntryCapacity`]）。产品路径一处都不调它。
+    ///
+    /// # Panics
+    /// 压的项数是 0 或大于记录格式装得下的 67 项（`JOURNAL_NAMED_ENTRIES_PER_RECORD`）：开关给错了。
+    pub fn set_journal_record_named_entry_capacity(
+        &mut self,
+        capacity: JournalRecordNamedEntryCapacity,
+    ) {
+        let of_the_format = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("67");
+        assert!(
+            (1..=of_the_format).contains(&capacity.named_entries_per_record()),
+            "一条记录装的点名项压到 {capacity:?}：要在 1..={of_the_format} 之内"
+        );
+        self.journal_record_named_entry_capacity = capacity;
+    }
+
+    /// 这会儿装着的「一条记录装几个点名项」：运行时看得出走的是哪一条分支。
+    #[must_use]
+    pub fn journal_record_named_entry_capacity(&self) -> JournalRecordNamedEntryCapacity {
+        self.journal_record_named_entry_capacity
+    }
+
+    /// 从下一次系统配置写起每一次都带这一张回退见证表（挂载按删除规则与这次的回退定好之后装上）。
+    pub(crate) fn write_rollback_witness_from_now_on(&mut self, table: RollbackWitnessTable) {
+        self.rollback_witness_to_write = Some(table);
+    }
+
     pub fn perform(&mut self, step: CommitStep<'_>) -> Result<(), BlockDeviceError> {
         if !matches!(step, CommitStep::Barrier) {
             self.has_writes_since_barrier = true;
@@ -223,17 +335,29 @@
     ) -> Result<(), BlockDeviceError> {
         let spacing = u64::from(self.parameters.geometry.fixed_structure_slot_spacing);
         let identity = self.devices[index].0;
-        let slot_generation = verified_system_configuration_slots(
+        let verified_on_this_device = verified_system_configuration_slots(
             &*self.devices,
             identity,
             spacing,
             &self.parameters.filesystem_identifier,
-        )
-        .iter()
-        .map(|system_configuration| system_configuration.quantities.slot_generation)
-        .max()
-        .unwrap_or(0)
+        );
+        let slot_generation = verified_on_this_device
+            .iter()
+            .map(|system_configuration| system_configuration.quantities.slot_generation)
+            .max()
+            .unwrap_or(0)
             + 1;
+        // 回退见证表每一次系统配置写都整张带着（D23（journal 的角色与格式） 已定项 14「回退见证」）：挂载定了这次要写哪一张就写那一张，
+        // 没定就照抄这块盘上择到的那一槽（两槽里自证过、世代号最大的）里的——与算世代号读的是同两槽，不多读一次盘。
+        let rollback_witness = match self.rollback_witness_to_write {
+            Some(table) => table,
+            None => verified_on_this_device
+                .iter()
+                .max_by_key(|system_configuration| system_configuration.quantities.slot_generation)
+                .map_or(RollbackWitnessTable::EMPTY, |system_configuration| {
+                    system_configuration.rollback_witness
+                }),
+        };
         let system_configuration = SystemConfiguration {
             immutable: SystemImmutableConfiguration {
                 filesystem_identifier: self.parameters.filesystem_identifier,
@@ -249,6 +373,7 @@
                 journal_tail,
                 journal_instance,
             },
+            rollback_witness,
         };
         self.has_writes_since_barrier = true;
         let slot_bytes = system_configuration.to_slot();
@@ -568,11 +693,192 @@
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct VersionWithoutFilePublishOutput {
     pub root: RootRecord,
+    /// 这次发布的末条记录（下一条的反向链要罩它的头）。
     pub record: JournalRecord,
     pub record_bytes: Vec<u8>,
+    /// 末条之前的那几条：写行那次发布点名项多于一条记录装得下时末条再跨记录（D23（journal 的角色与格式） 已定项 17）；
+    /// 零单元发布与只有一条记录的写行是空的。
+    pub earlier_records_of_this_publish: Vec<WrittenJournalRecord>,
     pub writes: WritesByStructureKind,
 }
 
+/// 一次发布要交给设备的全部字节，按 D16（发布语义） 已定项 7 的持久顺序排好：这次重写的单元 → 屏障 → journal 记录 → 屏障 →
+/// 根槽 FUA → 系统配置槽轮换。三条发布路径（带单元的、零单元的、树表 0 条上写行的）都先装成它、再交给
+/// [`persist_publish_writes`] 落盘；落盘中途失败的那一次原样冻结着它，重发时照它逐字节再发一遍
+/// （D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：checkpoint_txg、计数器、本次发布内序号、记录标志、
+/// 单元的位置与字节都不变）。系统配置槽不在里面：它的世代号按写那一刻盘上自证过的槽现算（D22（单元原子性怎么合成） 已定项 16，逐盘计），
+/// 内容是 tail、实例代号与写入口手里的回退见证表。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct PublishWrites {
+    /// 这次重写的单元，按写的次序（bump 次序）。零单元发布是空的。
+    pub units: Vec<PublishedUnit>,
+    /// 这次发布的 journal 记录，按计数器升序。
+    pub records: Vec<JournalRecordWrite>,
+    pub checkpoint_txg: CheckpointTxg,
+    /// 根槽 FUA 写的整槽字节。
+    pub root_slot: Vec<u8>,
+    /// 系统配置里的 tail：这次发布末条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 18）。
+    pub journal_tail: u64,
+    pub journal_instance: InstanceGeneration,
+}
+
+/// 一条要写进 journal 环的记录：计数器（定落在环里哪一槽）与整条 4096 字节。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct JournalRecordWrite {
+    pub counter: u64,
+    pub bytes: Vec<u8>,
+}
+
+/// 把一次发布的字节按持久顺序交给设备（D16（发布语义） 已定项 7）。三条发布路径与重发冻结的那一次共用这一处——
+/// 各抄一份会分叉，而「根槽在系统配置槽之前」正是崩溃窗口那几格的前提。
+///
+/// # Errors
+/// 块设备报的第一个错原样交回，之后的步骤一步都不发。
+fn persist_publish_writes<Device: BlockDevice>(
+    writer: &mut PoolWriter<'_, Device>,
+    writes: &PublishWrites,
+) -> Result<(), BlockDeviceError> {
+    for unit in &writes.units {
+        writer.perform(CommitStep::WriteUnitToEveryDevice {
+            slot: unit.slot,
+            unit: &unit.bytes,
+            identity: unit.identity,
+        })?;
+    }
+    writer.perform(CommitStep::Barrier)?;
+    for record in &writes.records {
+        writer.perform(CommitStep::WriteJournalRecordToEveryDevice {
+            counter: record.counter,
+            record: &record.bytes,
+        })?;
+    }
+    writer.perform(CommitStep::Barrier)?;
+    persist_the_root_then_rotate_the_system_configuration(
+        writer,
+        writes.checkpoint_txg,
+        &writes.root_slot,
+        writes.journal_tail,
+        writes.journal_instance,
+    )
+}
+
+/// 落盘阶段中途失败、冻结着等原样重发的那一次发布（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：发布不接受失败，
+/// 失败的那次冻结，下一次发布之前逐字节原样重发它，重发成功才建下一次发布）。住在分配器上
+/// （`PoolAllocator::frozen_publish`）：每一次发布都要交分配器进来，冻结着的时候发布路径在任何读写之前拒绝
+/// （`PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet`），只有 [`resend_the_frozen_publish`] 清得掉它。
+/// 于是同一 (实例代号, checkpoint_txg) 不会先后写出两次不同的发布、留下两条带末条标志的记录。
+///
+/// 零单元发布（`publish_without_units`）不经分配器、不冻结：它的调用方（暖机、可写挂载取号之后那一串）一失败就整个挂载返回错误，
+/// 下一次发布在下一次挂载里、换了实例代号。
+#[derive(Clone, Debug)]
+pub struct FrozenPublish {
+    writes: PublishWrites,
+    /// 重发成功之后交出去的那一版；它的写账在重发那一刻现记（`writes` 字段），这里是空账。
+    version: PoolVersion,
+    /// 这次发布成立之后的分配器（释放、取落点、记根都做完的那一份）。冻结期间调用方手里的分配器回到发布之前的样子，
+    /// 重发成功时整个换成它——冻结期间对那个分配器做的别的改动随之丢掉（发布路径冻结期间一个都不许进来）。
+    allocator_after_the_publish: PoolAllocator,
+}
+
+impl FrozenPublish {
+    #[must_use]
+    pub fn checkpoint_txg(&self) -> CheckpointTxg {
+        self.writes.checkpoint_txg
+    }
+
+    #[must_use]
+    pub fn instance(&self) -> InstanceGeneration {
+        self.writes.journal_instance
+    }
+
+    /// 重发时要逐字节再发一遍的那些字节。
+    #[must_use]
+    pub fn writes(&self) -> &PublishWrites {
+        &self.writes
+    }
+}
+
+/// 发布路径的第一道：分配器上冻结着一次没重发的发布，就在任何读写之前拒绝（D23（journal 的角色与格式） 已定项 14
+/// 「这一版的失败处置」：重发成功才建下一次发布）。
+///
+/// # Errors
+/// `PublishFrozenAfterAWriteFailureIsNotResentYet`，带着冻结的那一次的 checkpoint_txg 与实例代号。
+fn refuse_while_a_publish_is_frozen(allocator: &PoolAllocator) -> Result<(), PublishError> {
+    match allocator.frozen_publish() {
+        None => Ok(()),
+        Some(frozen) => Err(
+            PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet {
+                checkpoint_txg: frozen.checkpoint_txg(),
+                instance: frozen.instance(),
+            },
+        ),
+    }
+}
+
+/// 把装好、还没落盘的一次发布落盘（经分配器的两条发布路径共用：带文件的一版、树表 0 条的一版上写行）：
+/// 成功就把这次落盘的写账填进交回的那一版；落盘那几步里失败，这次已记的写进写入口的失败账（增补 2 第 20b 行），
+/// 分配器换回发布之前的样子，这次发布冻结在它上面等原样重发（[`FrozenPublish`]）。
+///
+/// # Errors
+/// 块设备报的错（`PublishError::BlockDevice`），这时这次发布已冻结。
+fn persist_the_publish_or_freeze_it<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    allocator_before_this_publish: PoolAllocator,
+    writes: PublishWrites,
+    version: PoolVersion,
+) -> Result<PoolVersion, PublishError> {
+    let writes_before_this_publish = pool.writes_by_structure_kind.clone();
+    if let Err(cause) = persist_publish_writes(pool, &writes) {
+        pool.count_failed_publish(&writes_before_this_publish);
+        let allocator_after_the_publish =
+            std::mem::replace(allocator, allocator_before_this_publish);
+        allocator.freeze_publish(FrozenPublish {
+            writes,
+            version,
+            allocator_after_the_publish,
+        });
+        return Err(PublishError::BlockDevice(cause));
+    }
+    Ok(version.with_writes(
+        pool.writes_by_structure_kind
+            .since(&writes_before_this_publish),
+    ))
+}
+
+/// 把分配器上冻结着的那次发布逐字节原样重发（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）：单元、journal 记录、
+/// 根槽照冻结时装好的字节再发一遍（checkpoint_txg、计数器、本次发布内序号、记录标志、单元的位置都不变），系统配置槽照常轮换。
+/// 成功就把分配器换成这次发布成立之后的那一份、交回那一版（写账是重发这一遍真写出去的）；没有冻结着的发布就什么都不写、交回 `None`。
+///
+/// # Errors
+/// 重发那几步里又有一步报错（`PublishError::BlockDevice`）：这一遍已记的写进失败账，那次发布照旧冻结着。
+pub fn resend_the_frozen_publish<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+) -> Result<Option<PoolVersion>, PublishError> {
+    let Some(frozen) = allocator.take_frozen_publish() else {
+        return Ok(None);
+    };
+    let writes_before_the_resend = pool.writes_by_structure_kind.clone();
+    if let Err(cause) = persist_publish_writes(pool, &frozen.writes) {
+        pool.count_failed_publish(&writes_before_the_resend);
+        allocator.freeze_publish(frozen);
+        return Err(PublishError::BlockDevice(cause));
+    }
+    let FrozenPublish {
+        writes: _,
+        version,
+        allocator_after_the_publish,
+    } = frozen;
+    *allocator = allocator_after_the_publish;
+    Ok(Some(
+        version.with_writes(
+            pool.writes_by_structure_kind
+                .since(&writes_before_the_resend),
+        ),
+    ))
+}
+
 /// 发布的最后两步（D16（发布语义） 已定项 7 的持久顺序）：根槽 FUA 写 → 系统配置槽轮换。
 /// 三条发布路径（带单元的、零单元的、树表 0 条上只写实例表的）共用这一处，不各抄一份——
 /// 抄出来的三份会分叉，而「根槽在系统配置槽之前」正是崩溃窗口那几格的前提。
@@ -637,25 +943,21 @@
         // 零单元发布一个字节都不写：分配记录树照抄上一版的那一条指针，账也因此一条都没变。
         allocation_record_tree_root: previous_root.allocation_record_tree_root,
     };
-    let root_slot = root.to_slot(pool.root_slot_bytes());
-    let writes_before_this_publish = pool.writes_by_structure_kind.clone();
-    // 中途失败时这次已记的写要交出去（增补 2 第 20b 行）：四步收在一个闭包里，失败在这里记账再把错原样交回。
-    let persist = |writer: &mut PoolWriter<'_, Device>| -> Result<(), BlockDeviceError> {
-        writer.perform(CommitStep::Barrier)?;
-        writer.perform(CommitStep::WriteJournalRecordToEveryDevice {
+    // 零单元：屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换（单元那一段是空的，头一道屏障前面没有写，写入口不发它）。
+    let writes = PublishWrites {
+        units: Vec::new(),
+        records: vec![JournalRecordWrite {
             counter: plan.counter,
-            record: &record_bytes,
-        })?;
-        writer.perform(CommitStep::Barrier)?;
-        persist_the_root_then_rotate_the_system_configuration(
-            writer,
-            plan.txg,
-            &root_slot,
-            plan.counter,
-            plan.instance,
-        )
+            bytes: record_bytes.clone(),
+        }],
+        checkpoint_txg: plan.txg,
+        root_slot: root.to_slot(pool.root_slot_bytes()),
+        journal_tail: plan.counter,
+        journal_instance: plan.instance,
     };
-    if let Err(cause) = persist(pool) {
+    let writes_before_this_publish = pool.writes_by_structure_kind.clone();
+    // 中途失败时这次已记的写要交出去（增补 2 第 20b 行）：失败在这里记账再把错原样交回。零单元发布不冻结（见 `FrozenPublish`）。
+    if let Err(cause) = persist_publish_writes(pool, &writes) {
         pool.count_failed_publish(&writes_before_this_publish);
         return Err(cause);
     }
@@ -663,6 +965,7 @@
         root,
         record,
         record_bytes,
+        earlier_records_of_this_publish: Vec::new(),
         writes: pool
             .writes_by_structure_kind
             .since(&writes_before_this_publish),
@@ -707,9 +1010,10 @@
 /// D8（核心索引结构） 已定项 8 ②）：树表、映射树根照抄（这一版的树表仍是 mkfs 那片 0 条的）。
 ///
 /// # Errors
+/// 分配器上冻结着一次没重发的发布（`PublishFrozenAfterAWriteFailureIsNotResentYet`）、
 /// 被换下的任一片的三样核不过（`ReleaseTarget*` / `ReleaseSpanMismatch`）、这次之后的分配记录装不进一个节点
 /// （`AllocationRecordsExceedOneNode`）、落点取不到（`PlacementRefused`）、块设备报错。
-/// 前三样在任何写之前返回，盘上逐字节不变；块设备错交回时分配器回到发布之前的样子。
+/// 前四样在任何写之前返回，盘上逐字节不变；块设备错交回时分配器回到发布之前的样子、这次发布冻结在它上面等原样重发（[`FrozenPublish`]）。
 ///
 /// # Panics
 /// 计划带着的旧链第 0 片不是上一版根记录指着的那一片：调用方拼行与拼旧链读的是同一条根。
@@ -719,38 +1023,74 @@
     previous_root: &RootRecord,
     plan: InstanceTableOnlyPublishPlan<'_>,
 ) -> Result<VersionWithoutFilePublishOutput, PublishError> {
+    refuse_while_a_publish_is_frozen(allocator)?;
     assert_eq!(
         plan.instance_table.replaced_chain.first(),
         Some(&previous_root.instance_table),
         "计划带着的旧链是上一版根记录指着的那一条：调用方拼行与拼旧链读的是同一条根"
     );
-    let swapped_out = placements_to_release_on_a_version_without_file(
+    let swapped_out = release_check_and_admission_of_a_row_publish_on_a_version_without_file(
         previous_root,
-        &plan.instance_table.replaced_chain,
+        plan.instance_table,
         allocator,
     )?;
-    version_without_file_row_publish_admission(
-        allocator,
-        plan.instance_table.pages_after_this_publish(),
-    )?;
     // 失败就换回去：取落点会动分配器（bump 指针、位图、记录），中途报错不留半新的池。
     let allocator_before_this_publish = allocator.clone();
-    let published = publish_instance_table_after_the_release_check(
+    let built = publish_instance_table_after_the_release_check(
         pool,
         allocator,
         previous_root,
         plan,
         swapped_out,
     );
-    if published.is_err() {
-        *allocator = allocator_before_this_publish;
-    }
-    published
+    let (writes, output) = match built {
+        Ok(built) => built,
+        Err(refusal) => {
+            *allocator = allocator_before_this_publish;
+            return Err(refusal);
+        }
+    };
+    persist_the_publish_or_freeze_it(
+        pool,
+        allocator,
+        allocator_before_this_publish,
+        writes,
+        PoolVersion::WithoutFile(output),
+    )
+    .map(|version| {
+        version.into_version_without_file().expect(
+            "交给落盘的是树表 0 条的一版，交回的就是它（`PoolVersion::with_writes` 不换成员）",
+        )
+    })
+}
+
+/// 树表 0 条的一版上写行那次发布的释放核验与准入（只查不改）：被换下的整条实例表旧链与上一版那片分配记录树节点逐片核三样
+/// （`placements_to_release_on_a_version_without_file`），这次之后的分配记录装得进一个节点（`version_without_file_row_publish_admission`）。
+/// 发布路径（`publish_instance_table_on_version_without_file`）与可写挂载取号之前的预演（`mount`，D18（块里携带什么信息） 已定项 11
+/// 「可写挂载的顺序」第五个合取：预演里写行那次的释放核验报错，同样判这次不能可写）都调它，两处判的是同一件事、同一个次序。
+///
+/// # Errors
+/// 旧链或分配记录树节点的三样核不过（`ReleaseTarget*` / `ReleaseSpanMismatch`）、分配记录装不下（`AllocationRecordsExceedOneNode`）。
+pub(crate) fn release_check_and_admission_of_a_row_publish_on_a_version_without_file(
+    previous_root: &RootRecord,
+    instance_table: &InstanceTableRewrite,
+    allocator: &PoolAllocator,
+) -> Result<PlacementsReleasedByTheRowPublish, PublishError> {
+    let swapped_out = placements_to_release_on_a_version_without_file(
+        previous_root,
+        &instance_table.replaced_chain,
+        allocator,
+    )?;
+    version_without_file_row_publish_admission(
+        allocator,
+        instance_table.pages_after_this_publish(),
+    )?;
+    Ok(swapped_out)
 }
 
 /// 写行那次发布换下的：上一版的整条实例表链，与上一版那片分配记录树节点
 /// （上一版是 mkfs 的第 0 代时根记录那一项全零、没有这一片）。
-struct PlacementsReleasedByTheRowPublish {
+pub(crate) struct PlacementsReleasedByTheRowPublish {
     /// 旧链每一片的落点，按片序号。
     instance_table_chain: Vec<Placement>,
     allocation_record_node: Option<Placement>,
@@ -942,15 +1282,16 @@
     pages_tail_first
 }
 
-/// `publish_instance_table_on_version_without_file` 的后半段：释放、取落点、装单元、落盘。分出来只为把「失败就把分配器换回去」
-/// 收在一处——中间每一步都可能提前返回，散在调用点上就会漏掉某一条路径。
+/// `publish_instance_table_on_version_without_file` 的后半段：释放、取落点、装单元，交回装好、还没落盘的字节与这一版
+/// （写账是空的，落盘由调用方做）。分出来只为把「失败就把分配器换回去」收在一处——中间每一步都可能提前返回，
+/// 散在调用点上就会漏掉某一条路径。
 fn publish_instance_table_after_the_release_check<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
     previous_root: &RootRecord,
     plan: InstanceTableOnlyPublishPlan<'_>,
     swapped_out: PlacementsReleasedByTheRowPublish,
-) -> Result<VersionWithoutFilePublishOutput, PublishError> {
+) -> Result<(PublishWrites, VersionWithoutFilePublishOutput), PublishError> {
     let txg = plan.txg;
     let instance = plan.instance;
     let filesystem_identifier = &pool.parameters.filesystem_identifier;
@@ -1019,29 +1360,26 @@
         instance,
         birth_sequence: allocation_sequence,
     };
-    let record = JournalRecord {
-        instance,
-        counter: plan.counter,
-        checkpoint_txg: txg,
-        transaction: 0,
-        is_commit: true,
-        // 这次发布只有这一条记录（D23（journal 的角色与格式） 已定项 4：只有一条时是 1）。
-        ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,
-        // 只有这一条 ⇒ 它就是这次发布的末条（D23（journal 的角色与格式） 已定项 17）。
-        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
-        back_chain: plan.back_chain,
-        filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
-        new_tree_table: previous_root.tree_table,
-        new_mapping_root: previous_root.mapping_root,
-        new_tree_identifier_watermark: plan.tree_identifier_watermark,
-        new_rollback_floor: plan.rollback_floor,
-        // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的角色各一项，按 bump 次序——实例表链的各片（尾片先），分配记录树。
-        named: instance_table_roles
-            .iter()
-            .map(|identity| {
+    // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的角色各一项，按 bump 次序——实例表链的各片（尾片先），分配记录树。
+    let named_roles: Vec<TransactionUnit> = instance_table_roles
+        .iter()
+        .copied()
+        .chain(std::iter::once(TransactionUnit::AllocationTree))
+        .collect();
+    let named_unit_of = |identity: TransactionUnit| -> NamedUnit {
+        match identity {
+            TransactionUnit::AllocationTree => NamedUnit {
+                locations: allocation_locations,
+                unit_class: TransactionUnit::AllocationTree.unit_class(),
+                // 树 ID 0：这一版还没登记过任何树，那一片由根记录独占持有（见 `build_allocation_record_node` 的文档注释）。
+                birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
+                birth_txg: txg,
+                key_tail: node_key_tail(instance, allocation_sequence),
+            },
+            TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                 let page = instance_table_pages
                     .iter()
-                    .find(|page| page.role == *identity)
+                    .find(|page| page.role == identity)
                     .expect("每个取了落点的实例表角色都装了一片");
                 NamedUnit {
                     locations: page.pointer.locations,
@@ -1051,18 +1389,78 @@
                     birth_txg: txg,
                     key_tail: node_key_tail(instance, page.pointer.birth_sequence),
                 }
-            })
-            .chain([NamedUnit {
-                locations: allocation_locations,
-                unit_class: TransactionUnit::AllocationTree.unit_class(),
-                // 树 ID 0：这一版还没登记过任何树，那一片由根记录独占持有（见 `build_allocation_record_node` 的文档注释）。
-                birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
-                birth_txg: txg,
-                key_tail: node_key_tail(instance, allocation_sequence),
-            }])
-            .collect(),
+            }
+            TransactionUnit::Data(_)
+            | TransactionUnit::ExtentRoot
+            | TransactionUnit::InodeLeafContainer(_)
+            | TransactionUnit::InodeRoot
+            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
+            | TransactionUnit::AccountingTree
+            | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
+            | TransactionUnit::MappingTree
+            | TransactionUnit::TreeTable => unreachable!(
+                "树表 0 条的一版上写行只点名实例表链各片与分配记录树节点（`named_roles` 就是这两样）"
+            ),
+        }
     };
-    let record_bytes = record.to_bytes();
+    // 这次发布切成几条记录（D23（journal 的角色与格式） 已定项 17「末条再跨记录」，与带文件那一路同一个切法：
+    // `roles_named_by_each_record_of_the_publish`）：点名项多于一条记录装得下的（实例表长到 67 片起）就装满一条再开下一条。
+    // 写行那次发布不承载事务，这几条都是事务号 0、同属一个事务（`transaction_offset_of_each_record_of_the_publish` 全是 0）：
+    // 提交标记只在它的最后一条上（已定项 7），「本次发布末条」标志只在真正的最后一条上（已定项 17），本次发布内序号依次 1..N（已定项 4），
+    // 反向链第一条接 `plan.back_chain`、之后每条接这次发布里前一条的头（已定项 8），每条都带整次发布的新根段（已定项 15）。
+    let named_entry_capacity = pool.journal_record_named_entry_capacity();
+    let roles_named_by_each_record =
+        roles_named_by_each_record_of_the_publish(&named_roles, named_entry_capacity);
+    let transaction_offset_of_each_record =
+        transaction_offset_of_each_record_of_the_publish(&named_roles, named_entry_capacity);
+    let mut written_records: Vec<WrittenJournalRecord> =
+        Vec::with_capacity(roles_named_by_each_record.len());
+    // 迭代次数的上界是这次的记录条数（≥ 1）；跨轮携带的只有已经装好的记录（下一条的反向链要罩前一条的头）。
+    for (record_offset_in_this_publish, roles_of_this_record) in
+        roles_named_by_each_record.iter().enumerate()
+    {
+        let back_chain = match written_records.last() {
+            None => plan.back_chain,
+            Some(previous_record_of_this_publish) => {
+                back_chain_of(&previous_record_of_this_publish.bytes)
+            }
+        };
+        let transaction_offset = transaction_offset_of_each_record[record_offset_in_this_publish];
+        let transaction_of_the_next_record = transaction_offset_of_each_record
+            .get(record_offset_in_this_publish + 1)
+            .copied();
+        let record = JournalRecord {
+            instance,
+            counter: plan.counter + u64::try_from(record_offset_in_this_publish).expect("记录序号"),
+            checkpoint_txg: txg,
+            // 写行那次发布不承载事务（D23（journal 的角色与格式） 已定项 19 ①）：事务号 0 加这条属于的事务序号（恒 0）。
+            transaction: transaction_offset,
+            is_commit: transaction_of_the_next_record != Some(transaction_offset),
+            ordinal_within_publish: JournalRecordOrdinalWithinPublish::of_record_at_offset(
+                record_offset_in_this_publish,
+            ),
+            place_in_publish: if transaction_of_the_next_record.is_none() {
+                JournalRecordPlaceInPublish::LastRecordOfThePublish
+            } else {
+                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
+            },
+            back_chain,
+            filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
+            new_tree_table: previous_root.tree_table,
+            new_mapping_root: previous_root.mapping_root,
+            new_tree_identifier_watermark: plan.tree_identifier_watermark,
+            new_rollback_floor: plan.rollback_floor,
+            named: roles_of_this_record
+                .iter()
+                .copied()
+                .map(named_unit_of)
+                .collect(),
+        };
+        let bytes = record.to_bytes();
+        written_records.push(WrittenJournalRecord { record, bytes });
+    }
+    let last_counter_of_this_publish =
+        plan.counter + u64::try_from(written_records.len() - 1).expect("一次发布至少一条记录");
     let root = RootRecord {
         filesystem_identifier: previous_root.filesystem_identifier,
         instance,
@@ -1074,53 +1472,57 @@
         mapping_root: previous_root.mapping_root,
         allocation_record_tree_root,
     };
-    let root_slot = root.to_slot(pool.root_slot_bytes());
-    let writes_before_this_row_publish = pool.writes_by_structure_kind.clone();
-    // 中途失败时这次已记的写要交出去（增补 2 第 20b 行）：六步收在一个闭包里，失败在这里记账再把错原样交回。
-    let persist = |writer: &mut PoolWriter<'_, Device>| -> Result<(), BlockDeviceError> {
-        // 单元写按 bump 次序：实例表链的各片（尾片先），分配记录树那一片最后。
-        for identity in &instance_table_roles {
+    // 单元写按 bump 次序：实例表链的各片（尾片先），分配记录树那一片最后。
+    let units: Vec<PublishedUnit> = instance_table_roles
+        .iter()
+        .map(|identity| {
             let page = instance_table_pages
                 .iter()
                 .find(|page| page.role == *identity)
                 .expect("每个取了落点的实例表角色都装了一片");
-            writer.perform(CommitStep::WriteUnitToEveryDevice {
+            PublishedUnit {
                 slot: instance_table_slots[identity],
-                unit: &page.bytes,
                 identity: *identity,
-            })?;
-        }
-        writer.perform(CommitStep::WriteUnitToEveryDevice {
+                bytes: page.bytes.clone(),
+            }
+        })
+        .chain(std::iter::once(PublishedUnit {
             slot: allocation_placement.slot,
-            unit: &allocation_unit,
             identity: TransactionUnit::AllocationTree,
-        })?;
-        writer.perform(CommitStep::Barrier)?;
-        writer.perform(CommitStep::WriteJournalRecordToEveryDevice {
-            counter: plan.counter,
-            record: &record_bytes,
-        })?;
-        writer.perform(CommitStep::Barrier)?;
-        persist_the_root_then_rotate_the_system_configuration(
-            writer,
-            txg,
-            &root_slot,
-            plan.counter,
-            instance,
-        )
+            bytes: allocation_unit,
+        }))
+        .collect();
+    let writes = PublishWrites {
+        units,
+        records: written_records
+            .iter()
+            .map(|written_record| JournalRecordWrite {
+                counter: written_record.record.counter,
+                bytes: written_record.bytes.clone(),
+            })
+            .collect(),
+        checkpoint_txg: txg,
+        root_slot: root.to_slot(pool.root_slot_bytes()),
+        // 系统配置里的 tail 存这次发布末条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 18）。
+        journal_tail: last_counter_of_this_publish,
+        journal_instance: instance,
     };
-    if let Err(cause) = persist(pool) {
-        pool.count_failed_publish(&writes_before_this_row_publish);
-        return Err(PublishError::BlockDevice(cause));
-    }
-    Ok(VersionWithoutFilePublishOutput {
-        root,
+    let WrittenJournalRecord {
         record,
-        record_bytes,
-        writes: pool
-            .writes_by_structure_kind
-            .since(&writes_before_this_row_publish),
-    })
+        bytes: record_bytes,
+    } = written_records
+        .pop()
+        .expect("一次发布至少一条记录（`roles_named_by_each_record_of_the_publish` 至少给一项）");
+    Ok((
+        writes,
+        VersionWithoutFilePublishOutput {
+            root,
+            record,
+            record_bytes,
+            earlier_records_of_this_publish: written_records,
+            writes: WritesByStructureKind::NOTHING_WRITTEN,
+        },
+    ))
 }
 
 /// 一个角色这次发布的落点：用户数据按政策函数，其余是提交内生块（码 3 容器按数据单元那一档，D3（空间分配） 已定项 10 ⑤）。
@@ -1193,6 +1595,31 @@
             PoolVersion::WithFile(version) => Some(version),
         }
     }
+    /// 树表 0 条的那一版；带文件的一版时 `None`。
+    #[must_use]
+    pub fn into_version_without_file(self) -> Option<VersionWithoutFilePublishOutput> {
+        match self {
+            PoolVersion::WithoutFile(version) => Some(version),
+            PoolVersion::WithFile(_) => None,
+        }
+    }
+    /// 这一版的写账：这次发布交给设备、设备报了成功的写（落盘或重发那一遍之后现记）。
+    #[must_use]
+    pub fn writes(&self) -> &WritesByStructureKind {
+        match self {
+            PoolVersion::WithoutFile(version) => &version.writes,
+            PoolVersion::WithFile(version) => &version.writes,
+        }
+    }
+    /// 换上写账、别的一个字段都不动（成员不换）。
+    #[must_use]
+    fn with_writes(mut self, writes: WritesByStructureKind) -> Self {
+        match &mut self {
+            PoolVersion::WithoutFile(version) => version.writes = writes,
+            PoolVersion::WithFile(version) => version.writes = writes,
+        }
+        self
+    }
 }
 
 /// 第一个事务写出的八个单元，声明序 = D3（空间分配） 已定项 10 ⑤ 的 bump 次序（按树 ID 升序、树内先叶后根、映射树倒数第二、树表最末），
@@ -1210,7 +1637,16 @@
     InodeLeafContainer(InodeLeafContainerIndexInTree),
     InodeRoot,
     AllocationTree,
+    /// 记账树里根之下的一个节点（D8（核心索引结构） 已定项 11：多层码 2 树）：带它在这一版树里的位置（层级、同层从左数第几个）。
+    /// 根兼叶那一档没有这个角色——记账树只有一个节点时它就是 `AccountingTree`。位置是这一版的：上一版的节点照抄进来时
+    /// 位置可以变（它左边的叶切开了），角色跟着这一版的位置走（`code_two_tree::CodeTwoTreePlan::origins` 记着它从哪来）。
+    AccountingTreeNodeBelowTheRoot(CodeTwoTreeNodePosition),
+    /// 记账树的根（只有一个节点时它就是根兼叶）。
     AccountingTree,
+    /// 中央映射树里根之下的一个节点，同 `AccountingTreeNodeBelowTheRoot`。映射树的节点都豁免映射
+    /// （D19（块指针的结构与宽度预算） 已定项 8「映射树自己的节点」）：父条目里的位置条目是权威。
+    MappingTreeNodeBelowTheRoot(CodeTwoTreeNodePosition),
+    /// 中央映射树的根（只有一个节点时它就是根兼叶），根指针住根记录（D19（块指针的结构与宽度预算） 已定项 11）。
     MappingTree,
     TreeTable,
     /// 实例表单元（码 3 打包记录类型 4）：mkfs 种下第一片，之后每次可写挂载写行时重写（D18（块里携带什么信息） 已定项 11）；
@@ -1224,6 +1660,124 @@
     InstanceTablePageAfterTheFirst(InstanceTablePageIndex),
 }
 
+/// 多层码 2 树里哪一棵：记账树或中央映射树（D8（核心索引结构） 已定项 11 的两棵照分裂做的树）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum MultiLevelCodeTwoTree {
+    Accounting,
+    CentralMapping,
+}
+
+impl MultiLevelCodeTwoTree {
+    /// 这棵树在这一版里某个位置上的节点的角色：根是 `AccountingTree` / `MappingTree`，根之下各按位置。
+    #[must_use]
+    pub fn role_of_node(
+        self,
+        position: CodeTwoTreeNodePosition,
+        shape: &CodeTwoTreeShape,
+    ) -> TransactionUnit {
+        match (self, shape.is_root(position)) {
+            (MultiLevelCodeTwoTree::Accounting, true) => TransactionUnit::AccountingTree,
+            (MultiLevelCodeTwoTree::Accounting, false) => {
+                TransactionUnit::AccountingTreeNodeBelowTheRoot(position)
+            }
+            (MultiLevelCodeTwoTree::CentralMapping, true) => TransactionUnit::MappingTree,
+            (MultiLevelCodeTwoTree::CentralMapping, false) => {
+                TransactionUnit::MappingTreeNodeBelowTheRoot(position)
+            }
+        }
+    }
+
+    /// 叶条目的字段表宽：记账条目 34、映射条目 55。
+    #[must_use]
+    pub fn leaf_entry_width_in_bytes(self) -> usize {
+        match self {
+            MultiLevelCodeTwoTree::Accounting => {
+                usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34")
+            }
+            MultiLevelCodeTwoTree::CentralMapping => {
+                usize::try_from(MAPPING_ENTRY_BYTES).expect("55")
+            }
+        }
+    }
+
+    #[must_use]
+    pub const fn key_field_widths(self) -> CodeTwoKeyFieldWidths {
+        match self {
+            MultiLevelCodeTwoTree::Accounting => CodeTwoKeyFieldWidths::ACCOUNTING,
+            MultiLevelCodeTwoTree::CentralMapping => CodeTwoKeyFieldWidths::CENTRAL_MAPPING,
+        }
+    }
+
+    /// 从盘上读这棵树时对它的期望（`code_two_tree::read_code_two_tree`）：这一版里它的树 ID 由调用方给（从水位发的号）。
+    #[must_use]
+    pub fn read_expectation(self, tree_identifier: TreeIdentifier) -> CodeTwoTreeReadExpectation {
+        CodeTwoTreeReadExpectation {
+            tree: tree_identifier,
+            field_widths: self.key_field_widths(),
+            internal_entry_name: match self {
+                MultiLevelCodeTwoTree::Accounting => "记账树内部条目",
+                MultiLevelCodeTwoTree::CentralMapping => "中央映射树内部条目",
+            },
+        }
+    }
+
+    /// 格式算出来的节点容量：记账树叶 477、内部 150，中央映射树叶 294、内部 143。
+    #[must_use]
+    pub fn node_capacity_of_the_node_format(self) -> CodeTwoTreeNodeCapacity {
+        CodeTwoTreeNodeCapacity::of_the_node_format(
+            self.key_field_widths().key_width_in_bytes(),
+            self.leaf_entry_width_in_bytes(),
+        )
+    }
+}
+
+/// 记账树与中央映射树的节点容量从哪来：产品路径按节点格式算；只供测试的开关把两棵树各自压小
+/// （`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支必须能被测试强制进入——格式算出来的容量下，记账树要 80 块盘才分裂，
+/// 中央映射树今天走得到的条目数装不满一个节点，分裂、收缩、降高这几条写序不压小就进不去）。只住写入口的内存：重开之后按产品路径起步。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CodeTwoTreeNodeCapacities {
+    FromTheNodeFormat,
+    CappedForTests {
+        accounting: CodeTwoTreeNodeCapacity,
+        central_mapping: CodeTwoTreeNodeCapacity,
+    },
+}
+
+impl CodeTwoTreeNodeCapacities {
+    /// 运行时报出走的是哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支必须可观测）。
+    #[must_use]
+    pub const fn name(self) -> &'static str {
+        match self {
+            CodeTwoTreeNodeCapacities::FromTheNodeFormat => {
+                "code_two_tree_node_capacities_from_the_node_format"
+            }
+            CodeTwoTreeNodeCapacities::CappedForTests { .. } => {
+                "code_two_tree_node_capacities_capped_for_tests"
+            }
+        }
+    }
+
+    /// 这棵树这会儿一个节点最多装几条。
+    #[must_use]
+    pub fn of_tree(self, tree: MultiLevelCodeTwoTree) -> CodeTwoTreeNodeCapacity {
+        match (self, tree) {
+            (CodeTwoTreeNodeCapacities::FromTheNodeFormat, _) => {
+                tree.node_capacity_of_the_node_format()
+            }
+            (
+                CodeTwoTreeNodeCapacities::CappedForTests { accounting, .. },
+                MultiLevelCodeTwoTree::Accounting,
+            ) => accounting,
+            (
+                CodeTwoTreeNodeCapacities::CappedForTests {
+                    central_mapping, ..
+                },
+                MultiLevelCodeTwoTree::CentralMapping,
+            ) => central_mapping,
+        }
+    }
+}
+
 impl TransactionUnit {
     pub const IN_BUMP_ORDER: [TransactionUnit; 8] = [
         TransactionUnit::Data(DataUnitIndexInFile::FIRST),
@@ -1261,7 +1815,14 @@
             TransactionUnit::InodeLeafContainer(index) => format!("t3+{}", index.0),
             TransactionUnit::InodeRoot => "t4".to_string(),
             TransactionUnit::AllocationTree => "t5".to_string(),
+            // 字节表七只登记了根兼叶那一档的 t6 / t7；根之下的节点写成「t6@层级.同层序号」，只出现在报错与用例消息里。
+            TransactionUnit::AccountingTreeNodeBelowTheRoot(position) => {
+                format!("t6@{}.{}", position.level, position.index_in_level)
+            }
             TransactionUnit::AccountingTree => "t6".to_string(),
+            TransactionUnit::MappingTreeNodeBelowTheRoot(position) => {
+                format!("t7@{}.{}", position.level, position.index_in_level)
+            }
             TransactionUnit::MappingTree => "t7".to_string(),
             TransactionUnit::TreeTable => "t8".to_string(),
             TransactionUnit::InstanceTable => "ti".to_string(),
@@ -1292,7 +1853,9 @@
             | TransactionUnit::InodeLeafContainer(_)
             | TransactionUnit::InodeRoot
             | TransactionUnit::AllocationTree
+            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree
+            | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
             | TransactionUnit::MappingTree
             | TransactionUnit::TreeTable => false,
         }
@@ -1308,7 +1871,9 @@
             TransactionUnit::ExtentRoot
             | TransactionUnit::InodeRoot
             | TransactionUnit::AllocationTree
+            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree
+            | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
             | TransactionUnit::MappingTree
             | TransactionUnit::TreeTable => UNIT_CLASS_INDEX_NODE,
         }
@@ -1322,8 +1887,11 @@
             TransactionUnit::Data(_) | TransactionUnit::ExtentRoot => trees.extent,
             TransactionUnit::InodeLeafContainer(_) | TransactionUnit::InodeRoot => trees.inode,
             TransactionUnit::AllocationTree => trees.allocation_records,
-            TransactionUnit::AccountingTree => trees.accounting,
-            TransactionUnit::MappingTree => trees.central_mapping,
+            TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
+            | TransactionUnit::AccountingTree => trees.accounting,
+            TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
+                trees.central_mapping
+            }
             TransactionUnit::TreeTable
             | TransactionUnit::InstanceTable
             | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
@@ -1345,7 +1913,9 @@
             TransactionUnit::ExtentRoot
             | TransactionUnit::InodeRoot
             | TransactionUnit::AllocationTree
+            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree
+            | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
             | TransactionUnit::MappingTree
             | TransactionUnit::TreeTable => PlacementRule::CommitGenerated(UnitFootprint::OneSlot),
         }
@@ -1499,6 +2069,10 @@
     pub mapping_keys: Vec<Vec<u8>>,
     pub allocation_records: Vec<AllocationRecord>,
     pub accounting_entries: Vec<AccountingEntry>,
+    /// 这一版记账树的形状与每个节点的指针（D8（核心索引结构） 已定项 11：多层码 2 树）；节点的字节住 `units` 里记账树那一族角色。
+    pub accounting_tree: CodeTwoTreeVersion,
+    /// 这一版中央映射树的形状与每个节点的指针，同上；根指针同时住根记录。
+    pub central_mapping_tree: CodeTwoTreeVersion,
     pub tree_table_entries: Vec<TreeTableEntry>,
     /// 这一版那八棵树各自的号：第一个文件版本那次从水位发出来，之后每一版照抄（从盘上重建的版本按树表条目的种类与根记录里
     /// 中央映射树根指针的出生树读回来）。
@@ -1528,13 +2102,27 @@
     pub highest_transaction_number_in_this_instance: u64,
 }
 
-/// 释放之前按位置项读盘核校验和、核出对不上（读不出也算）而隔离的一份：哪个角色、哪块盘、那一份的落点（槽取位置项里的，跨度取角色的）。
-/// 隔离就是那块盘上这个落点的分配记录留在已分配。
+/// 释放之前按位置项读盘核校验和、这个单元有一份核出对不上（读不出也算）而隔离的一份：哪个角色、哪块盘、那一份的落点
+/// （槽取位置项里的，跨度取角色的）、那一份自己读出来的样子。隔离就是那块盘上这个落点的分配记录留在已分配；
+/// 一个单元只要有一份对不上，每块盘上那一份都隔离（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-25 定：各盘的账保持对称）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct CopyQuarantinedAfterReleaseChecksumMismatch {
     pub unit: TransactionUnit,
     pub device: DeviceIdentity,
     pub placement: Placement,
+    pub reading: QuarantinedCopyReading,
+}
+
+/// 隔离的那一份自己读出来是什么样子（硬规则 1 的读盘核，D19（块指针的结构与宽度预算） 已定项 5）。计数报出时分得开
+/// 「真坏的份数」与「为了两块盘的账对称一起留下的好份」。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum QuarantinedCopyReading {
+    /// 读出来了，整单元 CRC-32C 与位置项里的对不上。
+    ChecksumMismatch,
+    /// 第一次读与重读一次都读不出（只重读一次，用户 2026-09-24 定）。
+    UnreadableAfterOneReread,
+    /// 这一份对得上，同一个单元别的盘上那一份对不上或读不出：两块盘一起留在已分配（用户 2026-09-25 定）。
+    IntactButAnotherCopyFailed,
 }
 
 /// 一版里的一片 inode 叶容器：装了什么、这一版它的指针在哪。字节不放这里，放 `TransactionOutput::units`
@@ -1582,6 +2170,31 @@
             .expect("八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在")
     }
 
+    /// 一棵多层码 2 树的高：从它这一版根节点的码 2 头里现读层级 + 1（D8（核心索引结构） 已定项 11 ⑤；
+    /// D28（挂载期承诺量） 已定项 4：ckpt_cost 里每棵码 2 树的高从根节点头现读，不用内存里另存一份）。
+    ///
+    /// # Panics
+    /// 这一版里这棵树的根单元解不开：它是这个进程装出来的，或从盘上重建时校验过的（`recovery::rebuild_version`）。
+    #[must_use]
+    pub fn height_read_from_the_root_node_header(&self, tree: MultiLevelCodeTwoTree) -> u64 {
+        let root_role = match tree {
+            MultiLevelCodeTwoTree::Accounting => TransactionUnit::AccountingTree,
+            MultiLevelCodeTwoTree::CentralMapping => TransactionUnit::MappingTree,
+        };
+        let root_header = parse_index_node(&self.unit(root_role).bytes)
+            .expect("根单元是这个进程装的，或重建时解过、核过的");
+        u64::from(root_header.level) + 1
+    }
+
+    /// 这一版一棵多层码 2 树（形状与指针）。
+    #[must_use]
+    pub fn multi_level_tree(&self, tree: MultiLevelCodeTwoTree) -> &CodeTwoTreeVersion {
+        match tree {
+            MultiLevelCodeTwoTree::Accounting => &self.accounting_tree,
+            MultiLevelCodeTwoTree::CentralMapping => &self.central_mapping_tree,
+        }
+    }
+
     /// 树表里某棵树的根指针（照抄没重写的角色时用）。
     #[must_use]
     pub fn tree_root_pointer(&self, tree: u64) -> NodePointer {
@@ -1676,6 +2289,7 @@
             | TransactionUnit::InodeLeafContainer(_)
             | TransactionUnit::InodeRoot
             | TransactionUnit::AllocationTree
+            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
             | TransactionUnit::AccountingTree => {
                 if !has_a_placement_in_the_previous_version(previous, identity) {
                     continue;
@@ -1694,6 +2308,11 @@
                     }
                 }
             }
+            // 映射树根之下的节点同样豁免映射：父条目里的位置条目是权威（D19（块指针的结构与宽度预算） 已定项 8），
+            // 上一版记着每个节点的那条指针。
+            TransactionUnit::MappingTreeNodeBelowTheRoot(position) => {
+                previous.central_mapping_tree.pointer_of(position).locations
+            }
             TransactionUnit::MappingTree => previous.root.mapping_root.locations,
             TransactionUnit::TreeTable => previous.root.tree_table.locations,
             // 整条旧链由 `instance_table_chain_to_release` 按计划带着的那条链释放。
@@ -1783,10 +2402,13 @@
         TransactionUnit::Data(index) => {
             index.0 < u64::try_from(previous.data_pointers.len()).expect("单元数")
         }
+        // 多层码 2 树的节点按上一版的位置点名（`previous_roles_replaced_by_this_publish` 从计划里的「被换下的节点」取），恒在上一版里。
         TransactionUnit::ExtentRoot
         | TransactionUnit::InodeRoot
         | TransactionUnit::AllocationTree
+        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
         | TransactionUnit::AccountingTree
+        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
         | TransactionUnit::MappingTree
         | TransactionUnit::TreeTable
         | TransactionUnit::InstanceTable => true,
@@ -1808,121 +2430,188 @@
         .mapped_units
         .iter()
         .find(|(mapped, _)| *mapped == identity)
-        .expect("进映射的单元每个一把 key（数据、extent 根、每片 inode 叶容器、inode 根、分配记录树、记账树）");
-    mapping_locations_for_key(&previous.unit(TransactionUnit::MappingTree).bytes, key)
+        .expect("进映射的单元每个一把 key（数据、extent 根、每片 inode 叶容器、inode 根、分配记录树、记账树的每个节点）");
+    central_mapping_locations_in_the_version(previous, key)
+}
+
+/// 在一版的中央映射树里按 key 查落点：按这一版的形状从根沿分隔 key 走到那片叶（D8（核心索引结构） 已定项 11：
+/// 最后一个分隔 key ≤ key 的那个孩子），叶的字节取 `units` 里它那个角色的单元、在叶里按 key 查（`mapping_locations_for_key`）。
+/// 查的是叶的字节，不是内存里记的 key：叶被改过就照实查不到。
+#[must_use]
+pub fn central_mapping_locations_in_the_version(
+    version: &TransactionOutput,
+    key: &[u8],
+) -> MappingLookup {
+    let shape = &version.central_mapping_tree.shape;
+    let Some(leaf) = leaf_position_routed_to(
+        shape,
+        &CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
+    ) else {
+        return MappingLookup::NodeMalformedOrNoEntryWithThisKey;
+    };
+    let leaf_role = MultiLevelCodeTwoTree::CentralMapping.role_of_node(leaf, shape);
+    mapping_locations_for_key(&version.unit(leaf_role).bytes, key)
+}
+
+/// 经映射释放的角色（数据、extent 根、inode 叶容器与根、分配记录树、记账树）：这次重写的角色里走得到映射条目的那几个，按角色次序。
+/// 映射树、树表、实例表豁免映射（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12），硬规则 1 的读盘核挂在「经映射核到那条映射条目之后」，
+/// 它们不在其内；这次新多出来、上一版没有落点的角色也不在其内。
+fn mapped_roles_released_by_this_publish(
+    previous: &TransactionOutput,
+    roles: &[TransactionUnit],
+) -> Vec<TransactionUnit> {
+    roles_replaced_via_mapping(previous, roles)
+        .into_iter()
+        .filter(|identity| match identity {
+            TransactionUnit::Data(_)
+            | TransactionUnit::ExtentRoot
+            | TransactionUnit::InodeLeafContainer(_)
+            | TransactionUnit::InodeRoot
+            | TransactionUnit::AllocationTree
+            | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
+            | TransactionUnit::AccountingTree => true,
+            TransactionUnit::MappingTreeNodeBelowTheRoot(_)
+            | TransactionUnit::MappingTree
+            | TransactionUnit::TreeTable
+            | TransactionUnit::InstanceTable
+            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,
+        })
+        .filter(|identity| has_a_placement_in_the_previous_version(previous, *identity))
+        .collect()
+}
+
+/// 一个经映射释放的角色在上一版映射里的那两条位置项。
+///
+/// # Panics
+/// 在上一版的映射里查不到：调用方先走 `placements_to_release_via_mapping`、它按同一个上一版、同一串角色查过，
+/// 查不到时它已经报了 `ReleaseNotInMapping` 或 `MappingEntryNarrowerThanItsFieldTable`，走不到这里。
+fn mapping_locations_of_a_released_unit(
+    previous: &TransactionOutput,
+    identity: TransactionUnit,
+) -> [LocationEntry; 2] {
+    match mapping_locations_of_a_mapped_unit(previous, identity) {
+        MappingLookup::Found(locations) => locations,
+        MappingLookup::NodeMalformedOrNoEntryWithThisKey
+        | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => panic!(
+            "{identity:?} 在上一版的映射里查不到：placements_to_release_via_mapping 刚按同一个上一版查过，查不到时它已经报错返回"
+        ),
+    }
+}
+
+/// 经映射释放的每个单元，映射条目的两条位置项各指池里的一块盘、两条指的不是同一块（D19（块指针的结构与宽度预算） 已定项 5
+/// 「硬规则 1 的读盘核读不出、核出对不上时怎么办」：位置项指向一块不在池里的盘，或两条位置项指同一块盘，当映射条目损坏，
+/// 在任何写之前拒绝，盘上不变；用户 2026-09-25 定）。只读内存里的上一版、不读盘：发布路径在读盘核之前判，
+/// 可写挂载在取号之前的预演里按写行那一次换下的角色判（`mount` 的 `placements_of_the_publishes_after_acquisition_on_a_copy`），
+/// 两处判的是同一件事。
+///
+/// # Errors
+/// 位置项指的盘不在 `pool_devices` 里 ⇒ `MappingEntryLocationOnADeviceOutsideThePool`；
+/// 两条位置项指同一块盘 ⇒ `MappingEntryLocationsOnTheSameDevice`。按角色次序报第一个。
+///
+/// # Panics
+/// 同 [`mapping_locations_of_a_released_unit`]。
+pub fn refuse_mapping_entries_that_do_not_name_two_pool_devices(
+    previous: &TransactionOutput,
+    roles: &[TransactionUnit],
+    pool_devices: &[DeviceIdentity],
+) -> Result<(), PublishError> {
+    for identity in mapped_roles_released_by_this_publish(previous, roles) {
+        let locations = mapping_locations_of_a_released_unit(previous, identity);
+        if let Some(outside) = locations
+            .iter()
+            .find(|location| !pool_devices.contains(&location.device))
+        {
+            return Err(PublishError::MappingEntryLocationOnADeviceOutsideThePool {
+                unit: identity,
+                device: outside.device,
+                slot: outside.slot,
+            });
+        }
+        if locations[0].device == locations[1].device {
+            return Err(PublishError::MappingEntryLocationsOnTheSameDevice {
+                unit: identity,
+                device: locations[0].device,
+                slot: locations[0].slot,
+            });
+        }
+    }
+    Ok(())
 }
 
 /// 释放之前按映射条目的位置项读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1，用户 2026-09-23 定案）：
-/// 这次重写的角色里经映射释放的那几个（数据、extent 根、inode 叶容器与根、分配记录树、记账树），映射条目的每条位置项指的那一份
+/// 这次重写的角色里经映射释放的那几个（[`mapped_roles_released_by_this_publish`]），映射条目的每条位置项指的那一份
 /// 整单元读出来算 CRC-32C、与位置项里的比（写的时候位置项的校验和就是整单元 CRC-32C，`make_filesystem::location_entries`）。
 /// **读盘本身失败先重读一次，还读不出就按对不上处置**（用户 2026-09-24 定案，C394 三问的第一问）：只重读一次，不退避、不多次重试。
-/// 交回对不上的那几份，按角色次序、位置项次序，不重复；发布照常释放（映射条目去掉），只是对不上的那一份在它那块盘上的分配记录
-/// 留在「已分配」、不改成已释放（`PoolAllocator::release_leaving_the_record_allocated_on`，同一次定案的第三问），发布照成。
-/// 映射树、树表、实例表豁免映射，硬规则 1 的读盘核挂在「经映射核到那条映射条目之后」，它们不核。
-///
-/// **一个单元只有一部分副本对不上时停在这一格**（`ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported`）：
-/// 照定案只把对不上的那几块盘的记录留在已分配、别的盘照常释放，两块盘的分配记录从此不对称——第一版的冷启动走读要求每块盘的
-/// （槽, 跨度, 代, 已释放）集合相同（`recovery::allocation_records_are_one_per_device`，不相同就判整池走读失败），分配器的落点也按
-/// 「各盘一致才分配」（`PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported`），这两处怎么容下不对称的账没有条款。
-/// 每一份都对不上（或都读不出）时两块盘的记录一起留在已分配，账仍对称，照定案做。
+/// **任一份核出对不上（读不出也算），这个单元在每块盘上的分配记录都留在「已分配」**、不改成已释放——另一块盘上那一份对得上也一起留，
+/// 各盘的账保持对称（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-25 定）：交回的是这些单元在池里每块盘上的那一份，
+/// 按角色次序、位置项的次序（设备身份升序），每一份带着它自己读出来的样子（[`QuarantinedCopyReading`]）；发布照常释放（映射条目去掉），
+/// 只是这些份的记录留在已分配（`PoolAllocator::release_leaving_the_record_allocated_on`），发布照成。
 ///
 /// 只读，在动分配器、发任何一个写之前调；调用方先走 `placements_to_release_via_mapping`、它核过了才走到这里。
 ///
 /// # Errors
-/// 某条位置项指的盘不在这个池里 ⇒ `ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided`：
-/// 那一份读不到不是读盘失败，「在它那块盘上的分配记录留在已分配」也没有那块盘可留，条款没写这一格；读到之前的几份不算数。
-/// 一个单元有的副本对不上、有的对得上 ⇒ `ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported`（见上）。
+/// 映射条目的位置项指池外的盘、或两条指同一块盘（[`refuse_mapping_entries_that_do_not_name_two_pool_devices`]）：
+/// 在读任何一份之前返回。
 ///
 /// # Panics
-/// 某个经映射释放的角色在上一版的映射里查不到：`placements_to_release_via_mapping` 刚按同一个上一版、同一串角色查过，
-/// 查不到时它已经报了 `ReleaseNotInMapping` 或 `MappingEntryNarrowerThanItsFieldTable`，走不到这里。
+/// 同 [`mapping_locations_of_a_released_unit`]。
 pub fn copies_failing_the_release_checksum_check<Reader: PoolReader + ?Sized>(
     previous: &TransactionOutput,
     roles: &[TransactionUnit],
     reader: &Reader,
 ) -> Result<Vec<CopyQuarantinedAfterReleaseChecksumMismatch>, PublishError> {
     let pool_devices = reader.device_identities();
-    let mut failing = Vec::new();
-    for identity in roles_replaced_via_mapping(previous, roles) {
-        match identity {
-            TransactionUnit::Data(_)
-            | TransactionUnit::ExtentRoot
-            | TransactionUnit::InodeLeafContainer(_)
-            | TransactionUnit::InodeRoot
-            | TransactionUnit::AllocationTree
-            | TransactionUnit::AccountingTree => {}
-            TransactionUnit::MappingTree
-            | TransactionUnit::TreeTable
-            | TransactionUnit::InstanceTable
-            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => continue,
-        }
-        if !has_a_placement_in_the_previous_version(previous, identity) {
-            continue;
-        }
-        let MappingLookup::Found(locations) =
-            mapping_locations_of_a_mapped_unit(previous, identity)
-        else {
-            panic!(
-                "{identity:?} 在上一版的映射里查不到：placements_to_release_via_mapping 刚按同一个上一版查过，查不到时它已经报错返回"
-            );
-        };
+    refuse_mapping_entries_that_do_not_name_two_pool_devices(previous, roles, &pool_devices)?;
+    let mut quarantined = Vec::new();
+    for identity in mapped_roles_released_by_this_publish(previous, roles) {
+        let locations = mapping_locations_of_a_released_unit(previous, identity);
         let unit_bytes = usize::try_from(identity.span_slots() * SLOT_BYTES)
             .expect("一个单元两槽以内，字节数装得进 usize");
-        let failing_before_this_unit = failing.len();
-        for location in &locations {
-            if !pool_devices.contains(&location.device) {
-                return Err(
-                    PublishError::ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided {
-                        unit: identity,
-                        device: location.device,
-                        slot: location.slot,
-                    },
-                );
-            }
-            let quarantined = CopyQuarantinedAfterReleaseChecksumMismatch {
+        let reading_of_each_copy: Vec<(LocationEntry, QuarantinedCopyReading)> = locations
+            .iter()
+            .map(|location| {
+                let read_the_copy = || {
+                    reader.read(
+                        location.device,
+                        location.slot.to_device_offset(),
+                        unit_bytes,
+                    )
+                };
+                // 读不出先重读一次（只一次）；还读不出就按对不上处置。
+                let reading = match read_the_copy().or_else(read_the_copy) {
+                    None => QuarantinedCopyReading::UnreadableAfterOneReread,
+                    Some(copy) if crc32_castagnoli(&copy) != location.unit_checksum => {
+                        QuarantinedCopyReading::ChecksumMismatch
+                    }
+                    Some(_) => QuarantinedCopyReading::IntactButAnotherCopyFailed,
+                };
+                (*location, reading)
+            })
+            .collect();
+        let some_copy_failed = reading_of_each_copy
+            .iter()
+            .any(|(_, reading)| match reading {
+                QuarantinedCopyReading::ChecksumMismatch
+                | QuarantinedCopyReading::UnreadableAfterOneReread => true,
+                QuarantinedCopyReading::IntactButAnotherCopyFailed => false,
+            });
+        if !some_copy_failed {
+            continue;
+        }
+        // 两条位置项各指池里一块不同的盘（上面判过；第一版池里恰好两块盘，mkfs 断言过），两份都留：按位置项的次序（设备身份升序）各交一份。
+        for (location, reading) in reading_of_each_copy {
+            quarantined.push(CopyQuarantinedAfterReleaseChecksumMismatch {
                 unit: identity,
                 device: location.device,
                 placement: Placement {
                     slot: location.slot,
                     span: identity.span_slots(),
                 },
-            };
-            let read_the_copy = || {
-                reader.read(
-                    location.device,
-                    location.slot.to_device_offset(),
-                    unit_bytes,
-                )
-            };
-            // 读不出先重读一次（只一次）；还读不出就按对不上处置。
-            let Some(copy) = read_the_copy().or_else(read_the_copy) else {
-                if !failing.contains(&quarantined) {
-                    failing.push(quarantined);
-                }
-                continue;
-            };
-            if crc32_castagnoli(&copy) != location.unit_checksum && !failing.contains(&quarantined)
-            {
-                failing.push(quarantined);
-            }
-        }
-        let devices_whose_copy_failed: Vec<DeviceIdentity> = failing[failing_before_this_unit..]
-            .iter()
-            .map(|copy| copy.device)
-            .collect();
-        let every_copy_failed = pool_devices
-            .iter()
-            .all(|device| devices_whose_copy_failed.contains(device));
-        if !devices_whose_copy_failed.is_empty() && !every_copy_failed {
-            return Err(
-                PublishError::ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported {
-                    unit: identity,
-                    devices_whose_copy_failed,
-                },
-            );
+                reading,
+            });
         }
     }
-    Ok(failing)
+    Ok(quarantined)
 }
 
 /// 一个被换下的单元的落点：两条位置条目同槽，池里每块盘各有一条在册、未释放、跨度对得上的记录；有一样不对就报错、不交回落点。
@@ -2030,25 +2719,17 @@
         refusal: PlacementRefusal,
     },
     /// 分配记录树第一版只有一个节点，这次发布之后的记录数装不下（每次发布每盘加 8 条、释放只改写不删）。
-    AllocationRecordsExceedOneNode {
-        records: usize,
-        capacity: usize,
-    },
-    /// 记账树第一版只有一个节点，这次发布要写的记账行装不下：行数 = 池级 3 行 + 每块盘 6 行，只随盘数变（477 条的节点在第 80 块盘时装不下）。
-    AccountingEntriesExceedOneNode {
-        entries: usize,
-        capacity: usize,
-    },
-    /// 中央映射树第一版也只有一个节点，这次发布要写的映射条目装不下：条目数 = 进映射的四个单节点角色
-    /// （extent 根、inode 根、分配记录树、记账树）+ 这一版的 inode 叶容器数 + 这一版文件的数据单元数。
-    MappingEntriesExceedOneNode {
-        entries: usize,
-        capacity: usize,
+    AllocationRecordsExceedOneNode { records: usize, capacity: usize },
+    /// 记账树或中央映射树这次之后的形状算不出来（`code_two_tree::plan_the_tree_after_this_publish`）：要长到 257 层、
+    /// 码 2 头的层级 1 字节写不下（多层之后映射树容量准入剩下的唯一一条，D19（块指针的结构与宽度预算） 已定项 5），
+    /// 或上一版（从盘上重建的）树按分隔 key 走不到它自己叶里的一把 key。在动分配器、发任何一个写之前返回，盘上逐字节不变。
+    /// 两棵树装不下一个节点时不再报错、照 D8（核心索引结构） 已定项 11 分裂（里程碑「第二个事务」增补 2 收口表第 28 行）。
+    MultiLevelCodeTwoTreeRefused {
+        tree: MultiLevelCodeTwoTree,
+        refusal: CodeTwoTreeRefusal,
     },
     /// 释放判定路径在上一版的映射里查不到这个单元（D19（块指针的结构与宽度预算） 已定项 5 第 1 条：不按提示释放）。
-    ReleaseNotInMapping {
-        unit: TransactionUnit,
-    },
+    ReleaseNotInMapping { unit: TransactionUnit },
     /// 映射查出来的落点在**这块盘**的分配记录里没有条目。`device` 带着是哪块盘：`PoolAllocator::release` 对池里
     /// 每块盘都要求一条对得上的记录，而两块盘的分配记录树对不对称是**盘上读来的**、不是不变量（panic 面普查 R10）。
     ReleaseTargetNotAllocated {
@@ -2087,31 +2768,24 @@
         entry_bytes: usize,
         field_table_bytes: usize,
     },
-    /// 释放之前按映射条目的位置项读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1），这条位置项指的盘不在这个池里：
-    /// 映射条目是盘上读来的字节（坏镜像、外来镜像才有），这一份读不到不是「读盘本身失败」——用户 2026-09-24 定的「先重读一次、
-    /// 还读不出就按对不上处置、那一份在它那块盘上的分配记录留在已分配」三句都没有对象（池里没有那块盘、也就没有那条记录可留），
-    /// 条款没写这一格 ⇒ 第一版不支持：在动分配器、发任何一个写之前返回，盘上逐字节不变。
-    ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided {
+    /// 经映射释放的这个单元，映射条目的一条位置项指的盘不在这个池里（D19（块指针的结构与宽度预算） 已定项 5「硬规则 1 的读盘核读不出、
+    /// 核出对不上时怎么办」：当映射条目损坏，用户 2026-09-25 定）。映射条目是盘上读来的字节，坏镜像、外来镜像才有。
+    /// 在动分配器、读盘核任何一份、发任何一个写之前返回，盘上逐字节不变。
+    MappingEntryLocationOnADeviceOutsideThePool {
         unit: TransactionUnit,
         device: DeviceIdentity,
         slot: SlotNumber,
     },
-    /// 释放之前按映射条目的位置项读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1），这个单元只有一部分副本对不上
-    /// （或读不出，先重读一次之后）、别的副本对得上。用户 2026-09-24 定「对不上的那一份在它那块盘上的分配记录留在已分配」，照做的话
-    /// 另一块盘的那一份照常释放，两块盘的分配记录从此不对称；而第一版的冷启动走读要求每块盘的记录集合相同
-    /// （`recovery::allocation_records_are_one_per_device`，不同就判整池走读失败——2026-09-24 故障注入快档打中），
-    /// 用户数据落点也按「各盘一致才分配」，这两处怎么容下不对称的账没有条款 ⇒ 第一版不支持：在动分配器、发任何一个写之前返回，
-    /// 盘上逐字节不变。`devices_whose_copy_failed` 是对不上的那几块盘。
-    ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported {
+    /// 经映射释放的这个单元，映射条目的两条位置项指同一块盘（同上一条：当映射条目损坏）——另一块盘上那一份从来没被核过就会被释放。
+    /// 在动分配器、读盘核任何一份、发任何一个写之前返回，盘上逐字节不变。`slot` 是第一条位置项的槽。
+    MappingEntryLocationsOnTheSameDevice {
         unit: TransactionUnit,
-        devices_whose_copy_failed: Vec<DeviceIdentity>,
+        device: DeviceIdentity,
+        slot: SlotNumber,
     },
     /// 用户给的内容装不进一个数据单元（32768 − 头 − 预留）：`publish_first_file` 与 `publish_overwrite` 按一个数据单元写
     /// （它们的契约），多单元的内容走 `publish_sequential_write`。
-    ContentExceedsDataUnit {
-        bytes: usize,
-        capacity: usize,
-    },
+    ContentExceedsDataUnit { bytes: usize, capacity: usize },
     /// 这个文件要的数据单元多于一片 extent 叶装得下的记录数（144，(16384 − 163) ÷ 112）：树就得长出内部节点，
     /// 而 **extent 树内部节点条目的格式仓里没有条款**——D8（核心索引结构） 已定项 11 只定码 2 节点的通用排法
     /// （定宽条目、key 打头），不定 extent 内部条目在 key 之后带什么（只带 86 字节子指针是 110，照 inode 树内部条目
@@ -2138,19 +2812,23 @@
     /// （2026-09-23 崩溃注入快档打中）。判的是树表条数，不是水位（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步：
     /// 回退到树表 0 条的一版时水位带着根环里的 max，已经不是 mkfs 的 11）。
     /// 同一个文件再写一版走 `publish_overwrite`。一个字节都不写。
-    FirstFileVersionOnAVersionThatAlreadyHasAFile {
-        tree_table_entries: usize,
-    },
+    FirstFileVersionOnAVersionThatAlreadyHasAFile { tree_table_entries: usize },
     /// `publish_first_file` 要判「那一版有没有过文件版本」得读它的树表单元，而那一片两份都读不出或解不开
     /// （`failure` 原样带着恢复路径那一格的原因）：判不了就不写，一个字节都不写。
-    TreeTableOfTheVersionToBuildOnUnreadable {
-        failure: RecoveryFailure,
-    },
+    TreeTableOfTheVersionToBuildOnUnreadable { failure: RecoveryFailure },
     /// `publish_first_file` 从那一版的树 ID 水位起连号发八棵树的号，而水位（盘上读来的 8 字节）加八个号越过 `u64::MAX`：
     /// 发不出号就不写，一个字节都不写。
     TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(
         TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees,
     ),
+    /// 分配器上冻结着一次落盘中途失败、还没原样重发的发布（`FrozenPublish`，D23（journal 的角色与格式） 已定项 14「这一版的失败处置」：
+    /// 重发成功才建下一次发布）：先 `resend_the_frozen_publish`，成功之后在它交回的那一版上接着发。在任何读写之前返回，盘上逐字节不变。
+    /// 带的是冻结的那一次的 checkpoint_txg 与实例代号。
+    PublishFrozenAfterAWriteFailureIsNotResentYet {
+        checkpoint_txg: CheckpointTxg,
+        instance: InstanceGeneration,
+    },
+    /// 块设备报的错。落盘阶段报的（带文件的一版、树表 0 条的一版上写行）：这次发布已冻结在分配器上等原样重发。
     BlockDevice(BlockDeviceError),
 }
 
@@ -2391,6 +3069,14 @@
 #[derive(Clone, Debug)]
 pub struct ResolvedPublish {
     pub inode_tree: InodeLeafContainersAfterThisPublish,
+    /// 这次之后记账树长什么样（D8（核心索引结构） 已定项 11 的分裂与收缩）。
+    pub accounting_tree: CodeTwoTreePlan,
+    /// 这次之后中央映射树长什么样，同上；按下面那一份映射 key 算。
+    pub central_mapping_tree: CodeTwoTreePlan,
+    /// 这一版每个进映射的单元与它的映射 key，按 bump 次序（每个数据单元、extent 根、每片 inode 叶容器、inode 根、分配记录树、
+    /// 记账树的每个节点）：这次重写的按这次要发的出生身份现算（出生序号按 bump 次序发，D19（块指针的结构与宽度预算） 已定项 9），
+    /// 照抄的取上一版的。装映射节点那一段（`publish_admitted`）核装出来的与它逐项相等。
+    pub mapped_units: Vec<(TransactionUnit, Vec<u8>)>,
     pub rewritten_roles: Vec<TransactionUnit>,
 }
 
@@ -2412,6 +3098,222 @@
                 .count(),
         }
     }
+
+    /// 一棵多层码 2 树这次之后的计划。
+    #[must_use]
+    pub fn multi_level_tree(&self, tree: MultiLevelCodeTwoTree) -> &CodeTwoTreePlan {
+        match tree {
+            MultiLevelCodeTwoTree::Accounting => &self.accounting_tree,
+            MultiLevelCodeTwoTree::CentralMapping => &self.central_mapping_tree,
+        }
+    }
+
+    /// 这次发布换下的上一版角色：这次重写的角色里不属于多层码 2 树的那些（同一个角色上一版那一份被换下），
+    /// 加两棵多层码 2 树里上一版被换下的节点（按上一版的位置点名；这一版照抄进来的节点角色可能换了位置，但它不被换下）。
+    /// 释放判定路径与释放之前读盘核校验和读的都是这一张（`placements_to_release_via_mapping`、`copies_failing_the_release_checksum_check`）。
+    #[must_use]
+    pub fn previous_roles_replaced_by_this_publish(
+        &self,
+        previous: &TransactionOutput,
+    ) -> Vec<TransactionUnit> {
+        // 按 bump 次序：不属于多层码 2 树的角色（树表除外）、记账树换下的节点、中央映射树换下的节点、树表最末——
+        // 与这次重写的角色同一个次序，释放清单与读盘核校验和的次序都照它。
+        let mut replaced: Vec<TransactionUnit> = self
+            .rewritten_roles
+            .iter()
+            .copied()
+            .filter(|identity| {
+                multi_level_tree_of_role(*identity).is_none()
+                    && *identity != TransactionUnit::TreeTable
+            })
+            .collect();
+        for tree in [
+            MultiLevelCodeTwoTree::Accounting,
+            MultiLevelCodeTwoTree::CentralMapping,
+        ] {
+            let previous_shape = &previous.multi_level_tree(tree).shape;
+            replaced.extend(
+                self.multi_level_tree(tree)
+                    .replaced_previous_nodes
+                    .iter()
+                    .map(|position| tree.role_of_node(*position, previous_shape)),
+            );
+        }
+        if self.rewritten_roles.contains(&TransactionUnit::TreeTable) {
+            replaced.push(TransactionUnit::TreeTable);
+        }
+        replaced
+    }
+}
+
+/// 一个角色属于哪棵多层码 2 树（记账树、中央映射树的根与根之下的节点）；别的角色交回 `None`。
+#[must_use]
+pub const fn multi_level_tree_of_role(identity: TransactionUnit) -> Option<MultiLevelCodeTwoTree> {
+    match identity {
+        TransactionUnit::AccountingTreeNodeBelowTheRoot(_) | TransactionUnit::AccountingTree => {
+            Some(MultiLevelCodeTwoTree::Accounting)
+        }
+        TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
+            Some(MultiLevelCodeTwoTree::CentralMapping)
+        }
+        TransactionUnit::Data(_)
+        | TransactionUnit::ExtentRoot
+        | TransactionUnit::InodeLeafContainer(_)
+        | TransactionUnit::InodeRoot
+        | TransactionUnit::AllocationTree
+        | TransactionUnit::TreeTable
+        | TransactionUnit::InstanceTable
+        | TransactionUnit::InstanceTablePageAfterTheFirst(_) => None,
+    }
+}
+
+/// 记账树里不带设备维的三行（D5（快照 / 空间记账机制） 已定项 8）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum PoolWideAccountingRow {
+    InodeNumberWatermark,
+    PendingDeleteBytes,
+    CommittedReservationBytes,
+}
+
+impl PoolWideAccountingRow {
+    const ALL: [Self; 3] = [
+        PoolWideAccountingRow::InodeNumberWatermark,
+        PoolWideAccountingRow::PendingDeleteBytes,
+        PoolWideAccountingRow::CommittedReservationBytes,
+    ];
+
+    const fn statistic(self) -> u16 {
+        match self {
+            PoolWideAccountingRow::InodeNumberWatermark => STATISTIC_INODE_WATERMARK,
+            PoolWideAccountingRow::PendingDeleteBytes => STATISTIC_PENDING_DELETE_BYTES,
+            PoolWideAccountingRow::CommittedReservationBytes => {
+                STATISTIC_COMMITTED_RESERVATION_BYTES
+            }
+        }
+    }
+
+    /// 这一行记在哪棵树上：inode 号水位记在 inode 树上，另两行不属于任何一棵树。
+    const fn tree(self, inode_tree: TreeIdentifier) -> TreeIdentifier {
+        match self {
+            PoolWideAccountingRow::InodeNumberWatermark => inode_tree,
+            PoolWideAccountingRow::PendingDeleteBytes
+            | PoolWideAccountingRow::CommittedReservationBytes => {
+                TreeIdentifier(TREE_IDENTIFIER_NONE)
+            }
+        }
+    }
+}
+
+/// 记账树里每块盘各一行的六项（D5（快照 / 空间记账机制） 已定项 8）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum PerDeviceAccountingRow {
+    AllocatedBytes,
+    FreeBytes,
+    UnreclaimableBytes,
+    DeferQueueBytes,
+    FragmentationRuns,
+    EmptyClusterSegments,
+}
+
+impl PerDeviceAccountingRow {
+    const ALL: [Self; 6] = [
+        PerDeviceAccountingRow::AllocatedBytes,
+        PerDeviceAccountingRow::FreeBytes,
+        PerDeviceAccountingRow::UnreclaimableBytes,
+        PerDeviceAccountingRow::DeferQueueBytes,
+        PerDeviceAccountingRow::FragmentationRuns,
+        PerDeviceAccountingRow::EmptyClusterSegments,
+    ];
+
+    const fn statistic(self) -> u16 {
+        match self {
+            PerDeviceAccountingRow::AllocatedBytes => STATISTIC_ALLOCATED_BYTES,
+            PerDeviceAccountingRow::FreeBytes => STATISTIC_FREE_BYTES,
+            PerDeviceAccountingRow::UnreclaimableBytes => STATISTIC_UNRECLAIMABLE_BYTES,
+            PerDeviceAccountingRow::DeferQueueBytes => STATISTIC_DEFER_QUEUE_BYTES,
+            PerDeviceAccountingRow::FragmentationRuns => STATISTIC_FRAGMENTATION_RUNS,
+            PerDeviceAccountingRow::EmptyClusterSegments => STATISTIC_EMPTY_CLUSTER_SEGMENTS,
+        }
+    }
+}
+
+/// 一次发布写的记账行（池级三行加每块盘六行，代 = 这次的 txg、seq 一律 1，D8（核心索引结构） 已定项 10），按 key 升序。
+/// 准入之前算记账树形状（只要 key，值给 0）与装记账树（值从分配器取）读的是这一份行表。
+fn accounting_entries_of_this_publish(
+    txg: CheckpointTxg,
+    inode_tree: TreeIdentifier,
+    device_identities: &[DeviceIdentity],
+    pool_wide_value: &dyn Fn(PoolWideAccountingRow) -> u64,
+    per_device_value: &dyn Fn(PerDeviceAccountingRow, DeviceIdentity) -> u64,
+) -> Vec<AccountingEntry> {
+    let mut entries: Vec<AccountingEntry> = PoolWideAccountingRow::ALL
+        .into_iter()
+        .map(|row| AccountingEntry {
+            statistic: row.statistic(),
+            tree: row.tree(inode_tree),
+            device: DeviceIdentity(STATISTIC_NO_DEVICE_DIMENSION),
+            generation: txg,
+            value: pool_wide_value(row),
+            sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
+        })
+        .collect();
+    for device in device_identities {
+        for row in PerDeviceAccountingRow::ALL {
+            entries.push(AccountingEntry {
+                statistic: row.statistic(),
+                tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
+                device: *device,
+                generation: txg,
+                value: per_device_value(row, *device),
+                sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
+            });
+        }
+    }
+    entries.sort_by_key(AccountingEntry::sort_key);
+    entries
+}
+
+/// 一个这次发布里新写出的码 2 / 码 3 单元的映射 key（D19（块指针的结构与宽度预算） 已定项 6）：类标签、出生 (树, txg)、实例代号、出生序号。
+/// key 不看落点：这次要写哪几个单元、各自的出生序号在取落点之前就定了（出生序号按 bump 次序发，D19（块指针的结构与宽度预算） 已定项 9）。
+fn mapping_key_of_a_node_born_in_this_publish(
+    unit_class: u8,
+    tree: TreeIdentifier,
+    txg: CheckpointTxg,
+    instance: InstanceGeneration,
+    birth_sequence: BirthSequence,
+) -> Vec<u8> {
+    mapping_key_for_node(
+        unit_class,
+        NodePointer {
+            head: PointerHead {
+                birth_tree: tree,
+                birth_txg: txg,
+            },
+            locations: NodePointer::empty_root().locations,
+            instance,
+            birth_sequence,
+        },
+    )
+}
+
+/// 从上一版取一个照抄进来的角色的映射 key。
+///
+/// # Panics
+/// 上一版里没有这个角色的映射 key：照抄进来的角色恒是上一版里进映射的那一个（上一版装映射节点时逐个记下了）。
+fn mapping_key_carried_from(previous: &TransactionOutput, identity: TransactionUnit) -> Vec<u8> {
+    previous
+        .mapped_units
+        .iter()
+        .find(|(mapped, _)| *mapped == identity)
+        .map(|(_, key)| key.clone())
+        .expect("照抄进来的角色在上一版里进映射、有一把 key")
+}
+
+/// 这张计划要写的记账树与中央映射树，在一切落盘动作之前算完：记账树按这一版的记账 key，中央映射树按这一版每个进映射单元的 key。
+struct MultiLevelTreesOfThisPublish {
+    accounting_tree: CodeTwoTreePlan,
+    central_mapping_tree: CodeTwoTreePlan,
+    mapped_units: Vec<(TransactionUnit, Vec<u8>)>,
 }
 
 impl PublishPlan<'_> {
@@ -2440,15 +3342,20 @@
         records
     }
 
-    /// 把这张计划算成「这次之后 inode 树是什么样、这次重写哪些角色」。`trees` 是这一版那八棵树的号
+    /// 把这张计划算成「这次之后 inode 树、记账树、中央映射树各长什么样、这次重写哪些角色」。`trees` 是这一版那八棵树的号
     /// （接在上一版之后就是上一版的，第一个文件版本是这次发出来的），新分裂出来的叶容器的出生树取其中的 inode 树。
+    /// `device_identities` 是记账行按盘展开的那几块盘（与装记账行时的分配器同一批），`capacities` 是两棵多层码 2 树的节点容量
+    /// （写入口上那个只供测试的开关，产品路径按节点格式算）。
     ///
     /// # Errors
-    /// inode 记录的落法要的条款仓里还没定 ⇒ `InodeTreeWriteRefused`（`crate::inode_tree` 的三格）。
+    /// inode 记录的落法要的条款仓里还没定 ⇒ `InodeTreeWriteRefused`（`crate::inode_tree` 的三格）；
+    /// 记账树或中央映射树这次之后的形状算不出来 ⇒ `MultiLevelCodeTwoTreeRefused`。
     pub fn resolve(
         &self,
         previous: Option<&TransactionOutput>,
         trees: &FileVersionTreeIdentifiers,
+        device_identities: &[DeviceIdentity],
+        capacities: CodeTwoTreeNodeCapacities,
     ) -> Result<ResolvedPublish, PublishError> {
         let containers_before = previous
             .map(TransactionOutput::inode_leaf_container_contents)
@@ -2478,17 +3385,219 @@
         if !inode_tree.rewritten.is_empty() {
             rewritten_roles.push(TransactionUnit::InodeRoot);
         }
-        rewritten_roles.extend([
-            TransactionUnit::AllocationTree,
-            TransactionUnit::AccountingTree,
-            TransactionUnit::MappingTree,
-            TransactionUnit::TreeTable,
-        ]);
+        rewritten_roles.push(TransactionUnit::AllocationTree);
+        let MultiLevelTreesOfThisPublish {
+            accounting_tree,
+            central_mapping_tree,
+            mapped_units,
+        } = self.plan_the_multi_level_trees(
+            previous,
+            trees,
+            &inode_tree,
+            device_identities,
+            capacities,
+        )?;
+        // 树内先叶后根、同层按 key 升序（D3（空间分配） 已定项 10 ⑤），记账树（树 14）在前、中央映射树倒数第二、树表最末。
+        for (tree, plan) in [
+            (MultiLevelCodeTwoTree::Accounting, &accounting_tree),
+            (MultiLevelCodeTwoTree::CentralMapping, &central_mapping_tree),
+        ] {
+            rewritten_roles.extend(
+                plan.rewritten_positions()
+                    .into_iter()
+                    .map(|position| tree.role_of_node(position, &plan.shape)),
+            );
+        }
+        rewritten_roles.push(TransactionUnit::TreeTable);
         Ok(ResolvedPublish {
             inode_tree,
+            accounting_tree,
+            central_mapping_tree,
+            mapped_units,
             rewritten_roles,
         })
     }
+
+    /// 记账树与中央映射树这次之后的形状，与这一版每个进映射单元的映射 key（`resolve` 的后一半）。
+    fn plan_the_multi_level_trees(
+        &self,
+        previous: Option<&TransactionOutput>,
+        trees: &FileVersionTreeIdentifiers,
+        inode_tree: &InodeLeafContainersAfterThisPublish,
+        device_identities: &[DeviceIdentity],
+        capacities: CodeTwoTreeNodeCapacities,
+    ) -> Result<MultiLevelTreesOfThisPublish, PublishError> {
+        let txg = self.txg;
+        let instance = self.instance;
+        let empty_shape = CodeTwoTreeShape::default();
+        let previous_shape_of = |tree: MultiLevelCodeTwoTree| match previous {
+            Some(previous_version) => &previous_version.multi_level_tree(tree).shape,
+            None => &empty_shape,
+        };
+        let refused = |tree: MultiLevelCodeTwoTree| {
+            move |refusal: CodeTwoTreeRefusal| PublishError::MultiLevelCodeTwoTreeRefused {
+                tree,
+                refusal,
+            }
+        };
+        // 记账树：每次发布整批重写（代 = 这次的 txg），key 只看统计量、树、盘与代，不看值。
+        let accounting_keys: BTreeSet<CodeTwoTreeKey> = accounting_entries_of_this_publish(
+            txg,
+            trees.inode,
+            device_identities,
+            &|_| 0,
+            &|_, _| 0,
+        )
+        .iter()
+        .map(|entry| CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::ACCOUNTING))
+        .collect();
+        let accounting_tree = plan_the_tree_after_this_publish(
+            previous_shape_of(MultiLevelCodeTwoTree::Accounting),
+            &accounting_keys,
+            capacities.of_tree(MultiLevelCodeTwoTree::Accounting),
+        )
+        .map_err(refused(MultiLevelCodeTwoTree::Accounting))?;
+
+        // 这一版每个进映射的单元的 key：重写的现算，照抄的取上一版的。出生序号在各自那棵树里按 bump 次序从 0 发（数据单元不发，
+        // 码 1 的 key 带写序）：extent 根是 extent 树这次唯一的码 2 单元、分配记录树一个节点，inode 树先叶后根。
+        let node_key = |unit_class: u8, tree: TreeIdentifier, birth_sequence: usize| {
+            mapping_key_of_a_node_born_in_this_publish(
+                unit_class,
+                tree,
+                txg,
+                instance,
+                BirthSequence(
+                    u32::try_from(birth_sequence).expect("一棵树一次发布的单元数装得进 u32"),
+                ),
+            )
+        };
+        let carried = |identity: TransactionUnit| {
+            mapping_key_carried_from(
+                previous.expect("照抄的角色只出现在接着上一版的发布里"),
+                identity,
+            )
+        };
+        let mut mapped_units: Vec<(TransactionUnit, Vec<u8>)> = Vec::new();
+        match &self.file {
+            Some(_) => {
+                for transaction in self.file_content_transactions() {
+                    mapped_units.push((
+                        TransactionUnit::Data(transaction.unit_index_in_file),
+                        mapping_key_for_data(
+                            PointerHead {
+                                birth_tree: trees.extent,
+                                birth_txg: txg,
+                            },
+                            WriteOrder {
+                                instance,
+                                transaction: transaction.transaction_number,
+                            },
+                        ),
+                    ));
+                }
+                mapped_units.push((
+                    TransactionUnit::ExtentRoot,
+                    node_key(UNIT_CLASS_INDEX_NODE, trees.extent, 0),
+                ));
+            }
+            None => {
+                let carried_version =
+                    previous.expect("没有文件版本的发布要接在上一版之后：文件角色从它照抄");
+                for position in 0..carried_version.data_pointers.len() {
+                    let identity = TransactionUnit::Data(DataUnitIndexInFile(
+                        u64::try_from(position).expect("单元序号"),
+                    ));
+                    mapped_units.push((identity, carried(identity)));
+                }
+                mapped_units.push((
+                    TransactionUnit::ExtentRoot,
+                    carried(TransactionUnit::ExtentRoot),
+                ));
+            }
+        }
+        let mut inode_tree_birth_sequence = 0;
+        for position in 0..inode_tree.containers.len() {
+            let index = InodeLeafContainerIndexInTree::of_position(position);
+            let identity = TransactionUnit::InodeLeafContainer(index);
+            if inode_tree.rewritten.contains(&index) {
+                mapped_units.push((
+                    identity,
+                    node_key(UNIT_CLASS_PACKED, trees.inode, inode_tree_birth_sequence),
+                ));
+                inode_tree_birth_sequence += 1;
+            } else {
+                mapped_units.push((identity, carried(identity)));
+            }
+        }
+        mapped_units.push(if inode_tree.rewritten.is_empty() {
+            (
+                TransactionUnit::InodeRoot,
+                carried(TransactionUnit::InodeRoot),
+            )
+        } else {
+            (
+                TransactionUnit::InodeRoot,
+                node_key(
+                    UNIT_CLASS_INDEX_NODE,
+                    trees.inode,
+                    inode_tree_birth_sequence,
+                ),
+            )
+        });
+        mapped_units.push((
+            TransactionUnit::AllocationTree,
+            node_key(UNIT_CLASS_INDEX_NODE, trees.allocation_records, 0),
+        ));
+        let previous_accounting_shape = previous_shape_of(MultiLevelCodeTwoTree::Accounting);
+        let mut accounting_birth_sequence = 0;
+        for (node, origin) in accounting_tree
+            .shape
+            .nodes()
+            .iter()
+            .zip(&accounting_tree.origins)
+        {
+            let identity = MultiLevelCodeTwoTree::Accounting
+                .role_of_node(node.position, &accounting_tree.shape);
+            match origin {
+                CodeTwoTreeNodeOrigin::RewrittenThisPublish => {
+                    mapped_units.push((
+                        identity,
+                        node_key(
+                            UNIT_CLASS_INDEX_NODE,
+                            trees.accounting,
+                            accounting_birth_sequence,
+                        ),
+                    ));
+                    accounting_birth_sequence += 1;
+                }
+                CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => {
+                    let previous_identity = MultiLevelCodeTwoTree::Accounting
+                        .role_of_node(*previous_position, previous_accounting_shape);
+                    mapped_units.push((identity, carried(previous_identity)));
+                }
+            }
+        }
+        let mapping_keys: BTreeSet<CodeTwoTreeKey> = mapped_units
+            .iter()
+            .map(|(_, key)| CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING))
+            .collect();
+        assert_eq!(
+            mapping_keys.len(),
+            mapped_units.len(),
+            "进映射的单元各一把 key，互不相撞（D19（块指针的结构与宽度预算） 已定项 6：出生身份各不相同）"
+        );
+        let central_mapping_tree = plan_the_tree_after_this_publish(
+            previous_shape_of(MultiLevelCodeTwoTree::CentralMapping),
+            &mapping_keys,
+            capacities.of_tree(MultiLevelCodeTwoTree::CentralMapping),
+        )
+        .map_err(refused(MultiLevelCodeTwoTree::CentralMapping))?;
+        Ok(MultiLevelTreesOfThisPublish {
+            accounting_tree,
+            central_mapping_tree,
+            mapped_units,
+        })
+    }
 }
 
 /// 文件版本那条 inode 记录（D8（核心索引结构） 已定项 6 的字段表）：size = 这次写的内容长度；
@@ -2532,6 +3641,8 @@
     instance: InstanceGeneration,
     previous_record_bytes: &[u8],
 ) -> Result<TransactionOutput, PublishError> {
+    // 下面补记零单元根会动分配器：冻结着一次没重发的发布时连它也不许做，第一道就拒（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）。
+    refuse_while_a_publish_is_frozen(allocator)?;
     // 「树还没建起来」看那一版的树表有几条（C511（回退到无文件那一版之后诞生代怎么接） 第 3 步），不看水位：
     // 回退到树表 0 条的一版时水位带着根环里的 max（D8（核心索引结构） 已定项 8 ②），那一版没有树、水位却早已不是 mkfs 的 11。
     let tree_table_entries = tree_table_entry_count(&*pool.devices, version_to_build_on)
@@ -2868,8 +3979,21 @@
     previous: Option<&TransactionOutput>,
     trees: FileVersionTreeIdentifiers,
 ) -> Result<TransactionOutput, PublishError> {
-    // 先算这次之后 inode 树是什么样、这次重写哪些角色：只读，条款没写的那几格在这里交回，盘上逐字节不变。
-    let resolved = plan.resolve(previous, &trees)?;
+    // 第一道：冻结着一次没重发的发布就在任何读写之前拒（D23（journal 的角色与格式） 已定项 14「这一版的失败处置」）。
+    refuse_while_a_publish_is_frozen(allocator)?;
+    // 先算这次之后 inode 树、记账树、中央映射树各是什么样、这次重写哪些角色：只读，条款没写的那几格与多层码 2 树的拒绝
+    // 在这里交回，盘上逐字节不变。记账行按分配器里的那几块盘展开（装记账行时读的是同一批）。
+    let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator
+        .devices
+        .iter()
+        .map(|device_map| device_map.device)
+        .collect();
+    let resolved = plan.resolve(
+        previous,
+        &trees,
+        &device_identities_of_the_accounting_rows,
+        pool.code_two_tree_node_capacities(),
+    )?;
     let rewritten = resolved.rewritten_roles.clone();
     // extent 树第一版只有一个节点（根兼叶）：这次写的数据单元一个一条叶记录，多于一片叶装得下的就要长出内部节点，
     // 而 extent 内部条目的格式没有条款 ⇒ 在动分配器之前拒掉（`ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided`）。
@@ -2893,10 +4017,19 @@
     // 点名项一条记录装 67 个（D23（journal 的角色与格式） 已定项 12 / 已定项 17）：最后一个事务要点名的项装不下一条记录时
     // 末条再跨记录（`roles_named_by_each_record_of_the_publish`），这里不拒。
     // 释放判定路径先于准入：它只查不改（D19（块指针的结构与宽度预算） 已定项 5 第 1 条），查不到就整次发布不做。
-    let release = match previous {
+    // 换下的是上一版的角色：多层码 2 树的节点按计划里「上一版被换下的节点」点名，别的角色同一个角色上一版那一份。
+    let previous_roles_replaced = match previous {
         Some(previous_version) => {
-            placements_to_release_via_mapping(previous_version, allocator, &rewritten)?
+            resolved.previous_roles_replaced_by_this_publish(previous_version)
         }
+        None => Vec::new(),
+    };
+    let release = match previous {
+        Some(previous_version) => placements_to_release_via_mapping(
+            previous_version,
+            allocator,
+            &previous_roles_replaced,
+        )?,
         // 第一个文件版本没有上一版的内存态：它重写树表时换下的是 mkfs 那片第 0 版树表单元，照样进 defer 队列
         // （D3（空间分配） 已定项 7；不释放它，txg 0 的根离开候选集之后这一槽就永远占着，I-3.1（已分配统计对得上） 在抬 F 之后红）。
         None => format_time_tree_table_to_release(allocator, &rewritten),
@@ -2922,28 +4055,17 @@
     // 经映射核到的那几个单元，释放之前再按位置项读盘核一次校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）：只读，
     // 读不到就在动分配器、发任何一个写之前返回。第一个文件版本换下的 mkfs 那片树表与树表 0 条那一版的分配记录树节点不经映射，不核。
     let quarantine = match previous {
-        Some(previous_version) => {
-            copies_failing_the_release_checksum_check(previous_version, &rewritten, &*pool.devices)?
-        }
+        Some(previous_version) => copies_failing_the_release_checksum_check(
+            previous_version,
+            &previous_roles_replaced,
+            &*pool.devices,
+        )?,
         None => Vec::new(),
     };
-    // 映射条目那一条准入按这一版的数据单元数算：重写文件内容就是这次写的那几个，照抄就是上一版的那几个。
-    let data_units_after_this_publish = match &plan.file {
-        Some(_) => rewritten_data_units,
-        None => previous
-            .expect("没有文件版本的发布要接在上一版之后：文件角色从它照抄（`publish_admitted` 依赖同一条）")
-            .data_pointers
-            .len(),
-    };
-    publish_admission(
-        allocator,
-        &rewritten,
-        resolved.inode_tree.containers.len(),
-        data_units_after_this_publish,
-    )?;
+    publish_admission(allocator, &rewritten)?;
     // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。
     let allocator_before_this_publish = allocator.clone();
-    let outcome = publish_admitted(
+    let built = publish_admitted(
         pool,
         allocator,
         &plan,
@@ -2953,85 +4075,421 @@
         &quarantine,
         trees,
     );
-    if outcome.is_err() {
-        *allocator = allocator_before_this_publish;
-    }
-    outcome
+    let (writes, output) = match built {
+        Ok(built) => built,
+        Err(refusal) => {
+            *allocator = allocator_before_this_publish;
+            return Err(refusal);
+        }
+    };
+    // 落盘那几步里失败：分配器换回去、这次发布冻结在它上面等原样重发（`FrozenPublish`）。
+    persist_the_publish_or_freeze_it(
+        pool,
+        allocator,
+        allocator_before_this_publish,
+        writes,
+        PoolVersion::WithFile(output),
+    )
+    .map(|version| {
+        version
+            .into_file_version()
+            .expect("交给落盘的是带文件的一版，交回的就是它（`PoolVersion::with_writes` 不换成员）")
+    })
 }
 
-/// 一次发布的准入里与这次写什么内容无关的那三条：分配记录树、记账树与中央映射树第一版各只有一个节点（分裂不做），
-/// 这次发布之后都要装得下。
+/// 一次发布的准入里与这次写什么内容无关的那一条：分配记录树第一版只有一个节点（按 key 空间定形状的结构在三方 `m2-keyspace-r1` 判，
+/// 这一轮不动它），这次发布之后要装得下——这次重写的每个角色每盘各加一条，两棵多层码 2 树分裂多出来的节点也各算一个角色。
+/// 记账树与中央映射树装不下一个节点时照 D8（核心索引结构） 已定项 11 分裂，不在这里拒；它们的形状在 `PublishPlan::resolve` 里算，
+/// 算不出来（长到 257 层）在那里交回（`MultiLevelCodeTwoTreeRefused`）。
 /// 只读——不动分配器、不发一个写，算不过时盘上逐字节不变。两处调它：发布路径在动分配器之前（`publish_version`）；
 /// 可写挂载在**取号之前**按这次挂载要发的那几次（写行 + 暖机）算一遍（`publish_sequence_admission`，增补 2 第 20a 行：
 /// 算不过就不许先把实例代号烧掉——取号是两次系统配置槽写加一道屏障，之后再拒绝，池此后每试一次可写挂载就多烧一个代号）。
 /// 两处读的是同一个内存里的分配器，取号不碰它，所以两次必定同答案；发布路径那一遍仍留着，它是动分配器之前的最后一道。
 ///
-/// 映射条目那一条要这次之后这一版的 inode 叶容器数与文件的数据单元数（照抄的也各占一条），两个数由调用方给。
-///
 /// # Errors
-/// `AllocationRecordsExceedOneNode`（这次之后的分配记录条数越过一个节点）、`AccountingEntriesExceedOneNode`（记账行数越过一个节点）、
-/// `MappingEntriesExceedOneNode`（映射条目数越过一个节点）。
+/// `AllocationRecordsExceedOneNode`（这次之后的分配记录条数越过一个节点）。
 pub fn publish_admission(
     allocator: &PoolAllocator,
     rewritten: &[TransactionUnit],
-    inode_leaf_containers_after_this_publish: usize,
-    data_units_after_this_publish: usize,
 ) -> Result<(), PublishError> {
-    admission_of_one_publish(
-        allocator,
-        allocator.records().len(),
-        rewritten,
-        inode_leaf_containers_after_this_publish,
-        data_units_after_this_publish,
-    )
+    admission_of_one_publish(allocator, allocator.records().len(), rewritten.len())
 }
 
-/// 接连几次发布的准入，在第一次动分配器之前一次算完：按次序逐次走 `publish_admission` 那三条，前面几次要新增的分配记录
-/// 算进后面几次的基数。基数只加不减，理由与单次那条同一句——释放只改写记录、不加，回收要 F 抬到释放代之上（`reclaim_released_up_to`），
+/// 接连几次不写文件内容、不碰 inode 树的发布（写行 + 暖机）的准入，在第一次动分配器之前一次算完：每一次重写的角色 =
+/// 形状给的那几个不属于多层码 2 树的角色 + 这一次记账树与中央映射树要重写的节点，前面几次要新增的分配记录算进后面几次的基数。
+/// 基数只加不减，理由与单次那条同一句——释放只改写记录、不加，回收要 F 抬到释放代之上（`reclaim_released_up_to`），
 /// 而这一串（写行 + 暖机）里不抬 F。
 ///
-/// 映射条目那一条按 `inode_leaf_containers_after_every_publish_in_this_sequence` 与
-/// `data_units_after_every_publish_in_this_sequence` 算：这一串里没有一次碰 inode 树、也没有一次写文件内容
-/// （写行与暖机都不碰），所以每一次之后的叶容器数与数据单元数都是上一版那两个数，调用方各给一个就够；下面两条断言钉住这个前提。
+/// 两棵多层码 2 树每一次重写几个节点按上一次之后的形状现算，与发布路径同一个 `code_two_tree::plan_the_tree_after_this_publish`
+/// （「一次发布最多分裂几次」由它算出，不另立上界）：这一串里每一次换掉的映射 key 只有分配记录树与记账树节点的那几把
+/// （写行与暖机不碰文件、不碰 inode 树），新 key 的出生 txg 比上一版里每一把都大、一次比一次大。key 逐字段比、类标签与出生树之后
+/// 先比出生 txg（D8（核心索引结构） 已定项 11），它们在中央映射树里落在哪、树怎么分裂只看这个次序，不看 txg 与实例代号的具体值——
+/// 所以取号之前拿「上一版的 txg + 1、+ 2 …」与上一版的实例代号代进去算出的形状，与取号之后真发布时算出的逐节点相同。
+/// 记账树的 key 只看统计量、树、盘与代，每次发布整批换代，形状只看行数与容量。`capacities` 与取号之后那几次发布的写入口上装的相同。
 ///
 /// # Errors
 /// `PublishSequenceRefusal`：第几次算不过（从 0 数），连它的 `PublishError` 一起交回。
 ///
 /// # Panics
-/// 传进来的形状里有一次重写叶容器或写文件内容：那时树可能分裂、文件的单元数可能变，一个数罩不住整串。
+/// 传进来的形状里有一次重写叶容器或写文件内容：那时换掉的映射 key 不止上面那几把，这里的推算罩不住。
 pub fn publish_sequence_admission(
     allocator: &PoolAllocator,
     shapes: &[PublishShape],
-    inode_leaf_containers_after_every_publish_in_this_sequence: usize,
-    data_units_after_every_publish_in_this_sequence: usize,
+    previous: &TransactionOutput,
+    capacities: CodeTwoTreeNodeCapacities,
 ) -> Result<(), PublishSequenceRefusal> {
+    let device_identities: Vec<DeviceIdentity> = allocator
+        .devices
+        .iter()
+        .map(|device_map| device_map.device)
+        .collect();
+    let rewritten_role_counts = rewritten_role_counts_of_a_publish_sequence(
+        &device_identities,
+        shapes,
+        previous,
+        capacities,
+    )?;
     let mut records_before_this_publish = allocator.records().len();
-    for (publish_index, shape) in shapes.iter().enumerate() {
+    for (publish_index, rewritten_role_count) in rewritten_role_counts.into_iter().enumerate() {
+        admission_of_one_publish(allocator, records_before_this_publish, rewritten_role_count)
+            .map_err(|cause| PublishSequenceRefusal {
+                publish_index,
+                cause,
+            })?;
+        records_before_this_publish += rewritten_role_count * allocator.devices.len();
+    }
+    Ok(())
+}
+
+/// 接连几次不写文件内容、不碰 inode 树的发布（写行 + 暖机）每一次重写几个角色：形状给的那几个不属于多层码 2 树的角色，
+/// 加这一次记账树与中央映射树要重写的节点（推法见 [`publish_sequence_admission`]）。`device_identities` 是记账行按盘展开的那几块盘。
+///
+/// # Errors
+/// 某一次两棵多层码 2 树的形状算不出来 ⇒ `PublishSequenceRefusal`（第几次、`MultiLevelCodeTwoTreeRefused`）。
+///
+/// # Panics
+/// 传进来的形状里有一次重写叶容器或写文件内容：那时换掉的映射 key 不止分配记录树与记账树的那几把，这里的推算罩不住。
+pub fn rewritten_role_counts_of_a_publish_sequence(
+    device_identities: &[DeviceIdentity],
+    shapes: &[PublishShape],
+    previous: &TransactionOutput,
+    capacities: CodeTwoTreeNodeCapacities,
+) -> Result<Vec<usize>, PublishSequenceRefusal> {
+    let txgs: Vec<CheckpointTxg> = (1..=u64::try_from(shapes.len()).expect("这一串的次数"))
+        .map(|offset| CheckpointTxg(previous.root.checkpoint_txg.0 + offset))
+        .collect();
+    let tree_nodes = multi_level_tree_nodes_of_a_publish_sequence_without_content(
+        previous,
+        shapes,
+        &txgs,
+        previous.root.instance,
+        device_identities,
+        capacities,
+    )?;
+    Ok(shapes
+        .iter()
+        .zip(&tree_nodes)
+        .map(|(shape, nodes)| {
+            rewritten_roles_of_a_publish_without_content(*shape, &nodes.rewritten_roles).len()
+        })
+        .collect())
+}
+
+/// 一次不写文件内容、不碰 inode 树的发布重写的全部角色，按 bump 次序：形状给的那几个不属于多层码 2 树的角色
+/// （写行的实例表链各片最前、尾片先，分配记录树、树表最末），两棵多层码 2 树这一次重写的节点插在树表之前（D3（空间分配） 已定项 10 ⑤：
+/// 记账树（树 14）、中央映射树倒数第二、树表最末）——与发布路径 `PublishPlan::resolve` 排出来的逐项相同。
+#[must_use]
+pub fn rewritten_roles_of_a_publish_without_content(
+    shape: PublishShape,
+    rewritten_tree_node_roles: &[TransactionUnit],
+) -> Vec<TransactionUnit> {
+    let mut roles: Vec<TransactionUnit> = shape
+        .rewritten_roles()
+        .into_iter()
+        .filter(|identity| {
+            multi_level_tree_of_role(*identity).is_none() && *identity != TransactionUnit::TreeTable
+        })
+        .collect();
+    roles.extend_from_slice(rewritten_tree_node_roles);
+    roles.push(TransactionUnit::TreeTable);
+    roles
+}
+
+/// 取号之前推一串不写文件内容的发布（写行 + 暖机）时，一次发布里两棵多层码 2 树重写哪些节点、换下哪些节点。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct MultiLevelTreeNodesOfAPublishWithoutContent {
+    /// 这一次两棵树重写的节点角色，按 bump 次序（记账树先叶后根，再中央映射树先叶后根）。
+    pub rewritten_roles: Vec<TransactionUnit>,
+    /// 这一次换下的节点（上一次之后那两棵树里没被照抄进这一次的）：各自是哪一次写出来的。
+    pub replaced: Vec<TreeNodeWrittenBy>,
+}
+
+/// 一个多层码 2 树节点是哪一次发布写出来的：这一串之前那一版（按它在那一版里的角色——释放时经那一版的映射或父指针查落点），
+/// 或这一串里第几次（从 0 数，按它在那一次里的角色——释放时取那一次为这个角色取到的落点）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum TreeNodeWrittenBy {
+    TheVersionBeforeTheSequence(TransactionUnit),
+    PublishOfTheSequence {
+        publish_index: usize,
+        role: TransactionUnit,
+    },
+}
+
+/// 接连几次不写文件内容、不碰 inode 树的发布（写行 + 暖机），每一次两棵多层码 2 树重写哪些节点、换下哪些节点，在第一次动分配器之前按
+/// 与发布路径同一个 `code_two_tree::plan_the_tree_after_this_publish` 一次推完。`txgs` 与 `instance` 是这一串要用的 txg 与实例代号
+/// （只进新 key 的出生身份：key 逐字段比、类标签与出生树之后先比出生 txg，它们比上一版里每一把都大、一次比一次大，
+/// 树怎么长只看这个次序，推出来的与真发布时逐节点相同）。
+///
+/// # Errors
+/// 某一次两棵多层码 2 树的形状算不出来 ⇒ `PublishSequenceRefusal`（第几次、`MultiLevelCodeTwoTreeRefused`）。
+///
+/// # Panics
+/// 形状里有一次重写叶容器或写文件内容（那时换掉的映射 key 不止分配记录树与记账树的那几把），或 `txgs` 与 `shapes` 不等长。
+pub fn multi_level_tree_nodes_of_a_publish_sequence_without_content(
+    previous: &TransactionOutput,
+    shapes: &[PublishShape],
+    txgs: &[CheckpointTxg],
+    instance: InstanceGeneration,
+    device_identities: &[DeviceIdentity],
+    capacities: CodeTwoTreeNodeCapacities,
+) -> Result<Vec<MultiLevelTreeNodesOfAPublishWithoutContent>, PublishSequenceRefusal> {
+    assert_eq!(shapes.len(), txgs.len(), "这一串每一次一个 txg");
+    let mut trees_before_this_publish = MultiLevelTreesBeforeAPublishWithoutContent::of(previous);
+    let mut every_publish = Vec::with_capacity(shapes.len());
+    for (publish_index, (shape, txg)) in shapes.iter().zip(txgs).enumerate() {
         assert_eq!(
             shape.rewritten_inode_leaf_containers, 0,
-            "这一串里每一次发布之后的叶容器数是同一个数（调用方给的那一个）：\
-             有一次重写叶容器就可能分裂、树就长了，那时要改成每次各带一个数"
+            "这一串里的发布不碰 inode 树：换掉的映射 key 只有分配记录树与记账树的"
         );
         assert_eq!(
             shape.rewritten_data_units, 0,
-            "这一串里每一次发布之后的数据单元数是同一个数（调用方给的那一个）：\
-             有一次写文件内容，单元数就可能变，那时要改成每次各带一个数"
+            "这一串里的发布不写文件内容：换掉的映射 key 只有分配记录树与记账树的"
         );
-        let rewritten = shape.rewritten_roles();
-        admission_of_one_publish(
-            allocator,
-            records_before_this_publish,
-            &rewritten,
-            inode_leaf_containers_after_every_publish_in_this_sequence,
-            data_units_after_every_publish_in_this_sequence,
+        let (trees_after_this_publish, nodes_of_this_publish) = trees_before_this_publish
+            .after_a_publish_without_content(
+                publish_index,
+                *txg,
+                instance,
+                &previous.tree_identifiers,
+                device_identities,
+                capacities,
+            )
+            .map_err(|cause| PublishSequenceRefusal {
+                publish_index,
+                cause,
+            })?;
+        every_publish.push(nodes_of_this_publish);
+        trees_before_this_publish = trees_after_this_publish;
+    }
+    Ok(every_publish)
+}
+
+/// 取号之前推算一串不写文件内容的发布时，两棵多层码 2 树在某一次之前的样子：两棵树的形状与每个节点是哪一次写出来的、
+/// 中央映射树里全部的 key、分配记录树与记账树每个节点的映射 key（这一串里换掉的就是这几把）。
+struct MultiLevelTreesBeforeAPublishWithoutContent {
+    accounting_shape: CodeTwoTreeShape,
+    accounting_written_by: BTreeMap<CodeTwoTreeNodePosition, TreeNodeWrittenBy>,
+    central_mapping_shape: CodeTwoTreeShape,
+    central_mapping_written_by: BTreeMap<CodeTwoTreeNodePosition, TreeNodeWrittenBy>,
+    mapping_keys: BTreeSet<CodeTwoTreeKey>,
+    allocation_node_mapping_key: CodeTwoTreeKey,
+    accounting_node_mapping_keys: BTreeMap<CodeTwoTreeNodePosition, CodeTwoTreeKey>,
+}
+
+impl MultiLevelTreesBeforeAPublishWithoutContent {
+    fn of(previous: &TransactionOutput) -> Self {
+        let key_of = |identity: TransactionUnit| {
+            CodeTwoTreeKey::new(
+                &mapping_key_carried_from(previous, identity),
+                CodeTwoKeyFieldWidths::CENTRAL_MAPPING,
+            )
+        };
+        let written_by_the_version_before = |tree: MultiLevelCodeTwoTree| {
+            let shape = &previous.multi_level_tree(tree).shape;
+            shape
+                .nodes()
+                .iter()
+                .map(|node| {
+                    (
+                        node.position,
+                        TreeNodeWrittenBy::TheVersionBeforeTheSequence(
+                            tree.role_of_node(node.position, shape),
+                        ),
+                    )
+                })
+                .collect()
+        };
+        Self {
+            accounting_shape: previous.accounting_tree.shape.clone(),
+            accounting_written_by: written_by_the_version_before(MultiLevelCodeTwoTree::Accounting),
+            central_mapping_shape: previous.central_mapping_tree.shape.clone(),
+            central_mapping_written_by: written_by_the_version_before(
+                MultiLevelCodeTwoTree::CentralMapping,
+            ),
+            mapping_keys: previous
+                .mapped_units
+                .iter()
+                .map(|(_, key)| CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING))
+                .collect(),
+            allocation_node_mapping_key: key_of(TransactionUnit::AllocationTree),
+            accounting_node_mapping_keys: previous
+                .accounting_tree
+                .shape
+                .nodes()
+                .iter()
+                .map(|node| {
+                    (
+                        node.position,
+                        key_of(
+                            MultiLevelCodeTwoTree::Accounting
+                                .role_of_node(node.position, &previous.accounting_tree.shape),
+                        ),
+                    )
+                })
+                .collect(),
+        }
+    }
+
+    /// 一次不写文件内容的发布之后两棵树的样子，与这一次两棵树重写、换下的节点。
+    fn after_a_publish_without_content(
+        &self,
+        publish_index: usize,
+        txg: CheckpointTxg,
+        instance: InstanceGeneration,
+        trees: &FileVersionTreeIdentifiers,
+        device_identities: &[DeviceIdentity],
+        capacities: CodeTwoTreeNodeCapacities,
+    ) -> Result<(Self, MultiLevelTreeNodesOfAPublishWithoutContent), PublishError> {
+        let accounting_keys: BTreeSet<CodeTwoTreeKey> = accounting_entries_of_this_publish(
+            txg,
+            trees.inode,
+            device_identities,
+            &|_| 0,
+            &|_, _| 0,
         )
-        .map_err(|cause| PublishSequenceRefusal {
-            publish_index,
-            cause,
+        .iter()
+        .map(|entry| CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::ACCOUNTING))
+        .collect();
+        let accounting_plan = plan_the_tree_after_this_publish(
+            &self.accounting_shape,
+            &accounting_keys,
+            capacities.of_tree(MultiLevelCodeTwoTree::Accounting),
+        )
+        .map_err(|refusal| PublishError::MultiLevelCodeTwoTreeRefused {
+            tree: MultiLevelCodeTwoTree::Accounting,
+            refusal,
+        })?;
+        let new_node_key = |tree: TreeIdentifier, birth_sequence: usize| {
+            CodeTwoTreeKey::new(
+                &mapping_key_of_a_node_born_in_this_publish(
+                    UNIT_CLASS_INDEX_NODE,
+                    tree,
+                    txg,
+                    instance,
+                    BirthSequence(u32::try_from(birth_sequence).expect("出生序号装得进 u32")),
+                ),
+                CodeTwoKeyFieldWidths::CENTRAL_MAPPING,
+            )
+        };
+        let mut mapping_keys = self.mapping_keys.clone();
+        mapping_keys.remove(&self.allocation_node_mapping_key);
+        for key in self.accounting_node_mapping_keys.values() {
+            mapping_keys.remove(key);
+        }
+        let allocation_node_mapping_key = new_node_key(trees.allocation_records, 0);
+        mapping_keys.insert(allocation_node_mapping_key.clone());
+        let mut accounting_node_mapping_keys = BTreeMap::new();
+        let mut accounting_birth_sequence = 0;
+        for (node, origin) in accounting_plan
+            .shape
+            .nodes()
+            .iter()
+            .zip(&accounting_plan.origins)
+        {
+            let key = match origin {
+                CodeTwoTreeNodeOrigin::RewrittenThisPublish => {
+                    accounting_birth_sequence += 1;
+                    new_node_key(trees.accounting, accounting_birth_sequence - 1)
+                }
+                CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => self
+                    .accounting_node_mapping_keys
+                    .get(previous_position)
+                    .expect("照抄的节点在上一次的形状里、有一把 key")
+                    .clone(),
+            };
+            mapping_keys.insert(key.clone());
+            accounting_node_mapping_keys.insert(node.position, key);
+        }
+        let central_mapping_plan = plan_the_tree_after_this_publish(
+            &self.central_mapping_shape,
+            &mapping_keys,
+            capacities.of_tree(MultiLevelCodeTwoTree::CentralMapping),
+        )
+        .map_err(|refusal| PublishError::MultiLevelCodeTwoTreeRefused {
+            tree: MultiLevelCodeTwoTree::CentralMapping,
+            refusal,
         })?;
-        records_before_this_publish += rewritten.len() * allocator.devices.len();
+        let mut rewritten_roles = Vec::new();
+        let mut replaced = Vec::new();
+        let mut written_by_after_this_publish = |tree: MultiLevelCodeTwoTree,
+                                                 plan: &CodeTwoTreePlan,
+                                                 written_by_before: &BTreeMap<
+            CodeTwoTreeNodePosition,
+            TreeNodeWrittenBy,
+        >| {
+            replaced.extend(plan.replaced_previous_nodes.iter().map(|position| {
+                *written_by_before
+                    .get(position)
+                    .expect("被换下的节点在上一次的形状里")
+            }));
+            plan.shape
+                .nodes()
+                .iter()
+                .zip(&plan.origins)
+                .map(|(node, origin)| {
+                    let written_by = match origin {
+                        CodeTwoTreeNodeOrigin::RewrittenThisPublish => {
+                            let role = tree.role_of_node(node.position, &plan.shape);
+                            rewritten_roles.push(role);
+                            TreeNodeWrittenBy::PublishOfTheSequence {
+                                publish_index,
+                                role,
+                            }
+                        }
+                        CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => *written_by_before
+                            .get(previous_position)
+                            .expect("照抄的节点在上一次的形状里"),
+                    };
+                    (node.position, written_by)
+                })
+                .collect::<BTreeMap<CodeTwoTreeNodePosition, TreeNodeWrittenBy>>()
+        };
+        let accounting_written_by = written_by_after_this_publish(
+            MultiLevelCodeTwoTree::Accounting,
+            &accounting_plan,
+            &self.accounting_written_by,
+        );
+        let central_mapping_written_by = written_by_after_this_publish(
+            MultiLevelCodeTwoTree::CentralMapping,
+            &central_mapping_plan,
+            &self.central_mapping_written_by,
+        );
+        Ok((
+            Self {
+                accounting_shape: accounting_plan.shape,
+                accounting_written_by,
+                central_mapping_shape: central_mapping_plan.shape,
+                central_mapping_written_by,
+                mapping_keys,
+                allocation_node_mapping_key,
+                accounting_node_mapping_keys,
+            },
+            MultiLevelTreeNodesOfAPublishWithoutContent {
+                rewritten_roles,
+                replaced,
+            },
+        ))
     }
-    Ok(())
 }
 
 /// 一串发布的准入里第几次算不过（从 0 数）、为什么。
@@ -3041,78 +4499,18 @@
     pub cause: PublishError,
 }
 
-/// 一次发布的那三条准入，分配记录的基数由调用方给：发布路径给的是分配器此刻的记录数，取号之前那一串给的是把前面几次
-/// 要新增的算进去之后的数。映射条目那一条按这次发布之后 inode 树有几片叶容器、文件有几个数据单元算，两个数由调用方给
-/// （发布路径从 `PublishPlan::resolve` 算出来的树与角色清单取，取号之前那一串取上一版的）。
+/// 一次发布的那一条准入，分配记录的基数由调用方给：发布路径给的是分配器此刻的记录数，取号之前那一串给的是把前面几次
+/// 要新增的算进去之后的数。`rewritten_role_count` 是这次重写的角色数（两棵多层码 2 树的每个重写节点各算一个）。
 fn admission_of_one_publish(
     allocator: &PoolAllocator,
     records_before_this_publish: usize,
-    rewritten: &[TransactionUnit],
-    inode_leaf_containers_after_this_publish: usize,
-    data_units_after_this_publish: usize,
+    rewritten_role_count: usize,
 ) -> Result<(), PublishError> {
     // 分配记录树第一版只有一个节点（判在 `refuse_when_the_allocation_records_do_not_fit_one_node`，
     // 与树表 0 条那一版写行时的准入共用一处）。释放只改写记录、不加；这次重写的每个角色每盘各加一条。
     let records_after_this_publish =
-        records_before_this_publish + rewritten.len() * allocator.devices.len();
-    refuse_when_the_allocation_records_do_not_fit_one_node(records_after_this_publish)?;
-    // 记账树第一版也只有一个节点：行数只随盘数变（代码三方第二轮攻方腿 Y3：约 80 块盘）。
-    // 装行那一段按同一个 `allocator.devices` 装，并断言行数与这里算的相等。
-    let accounting_entries_of_this_publish = accounting_entry_count(allocator.devices.len());
-    let accounting_node_capacity = index_node_entry_capacity(
-        usize::try_from(ACCOUNTING_KEY_BYTES).expect("22"),
-        usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
-    );
-    if accounting_entries_of_this_publish > accounting_node_capacity {
-        return Err(PublishError::AccountingEntriesExceedOneNode {
-            entries: accounting_entries_of_this_publish,
-            capacity: accounting_node_capacity,
-        });
-    }
-    // 中央映射树第一版同样只有一个节点：条目数随这一版的 inode 叶容器数与文件的数据单元数变。
-    // 装映射节点那一段按同一个 `mapping_entry_count` 断言条目数与这里算的相等。
-    let mapping_entries_of_this_publish = mapping_entry_count(
-        inode_leaf_containers_after_this_publish,
-        data_units_after_this_publish,
-    );
-    let mapping_node_capacity = index_node_entry_capacity(
-        usize::try_from(MAPPING_KEY_BYTES).expect("27"),
-        usize::try_from(MAPPING_ENTRY_BYTES).expect("55"),
-    );
-    if mapping_entries_of_this_publish > mapping_node_capacity {
-        return Err(PublishError::MappingEntriesExceedOneNode {
-            entries: mapping_entries_of_this_publish,
-            capacity: mapping_node_capacity,
-        });
-    }
-    Ok(())
-}
-
-/// 记账树里不带设备维的行：inode 号水位、待删占用、已承诺预留（D5（快照 / 空间记账机制） 已定项 8）。
-const POOL_WIDE_ACCOUNTING_ENTRIES: usize = 3;
-/// 记账树里每块盘各一行的：已分配、空闲、不可回收、defer 待释放、碎片段数、全空聚簇段数。
-const ACCOUNTING_ENTRIES_PER_DEVICE: usize = 6;
-
-/// 一次发布写的记账行数：池级行加每块盘各一组。准入与装行共用这一处。
-fn accounting_entry_count(device_count: usize) -> usize {
-    POOL_WIDE_ACCOUNTING_ENTRIES + ACCOUNTING_ENTRIES_PER_DEVICE * device_count
-}
-
-/// 进映射、每一版恒各一个的角色数：extent 根、inode 根、分配记录树、记账树（第一版这四棵树各只有一个节点，extent 树见
-/// `ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided`）。映射树自己、树表、实例表豁免映射
-/// （D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12），不在这个数里。
-const MAPPING_ENTRIES_OF_THE_FOUR_SINGLE_NODE_ROLES: usize = 4;
-
-/// 一次发布写的映射条目数：四个单节点角色各一条，加这一版每片 inode 叶容器、文件的每个数据单元各一条
-/// （照抄的也各占一条——映射节点每次发布整片重写，装的是这一版全部进映射的单元）。
-/// 准入与装映射节点共用这一处。
-fn mapping_entry_count(
-    inode_leaf_containers_after_this_publish: usize,
-    data_units_after_this_publish: usize,
-) -> usize {
-    MAPPING_ENTRIES_OF_THE_FOUR_SINGLE_NODE_ROLES
-        + inode_leaf_containers_after_this_publish
-        + data_units_after_this_publish
+        records_before_this_publish + rewritten_role_count * allocator.devices.len();
+    refuse_when_the_allocation_records_do_not_fit_one_node(records_after_this_publish)
 }
 
 /// 一次发布切成几条 journal 记录、每条点名哪几个角色（D23（journal 的角色与格式） 已定项 17，C491（多条记录时共享内生块在哪条点名没定）
@@ -3129,6 +4527,7 @@
 #[must_use]
 pub fn roles_named_by_each_record_of_the_publish(
     rewritten: &[TransactionUnit],
+    named_entry_capacity: JournalRecordNamedEntryCapacity,
 ) -> Vec<Vec<TransactionUnit>> {
     let data_roles: Vec<TransactionUnit> = rewritten
         .iter()
@@ -3156,7 +4555,7 @@
         // 一个角色都不点名的发布照样写一条记录（它一条就是一次发布）。
         roles_named_by_each_record.push(roles_named_by_the_last_transaction);
     } else {
-        let named_unit_capacity = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("67");
+        let named_unit_capacity = named_entry_capacity.named_entries_per_record();
         roles_named_by_each_record.extend(
             roles_named_by_the_last_transaction
                 .chunks(named_unit_capacity)
@@ -3171,13 +4570,16 @@
 /// 其余的——最后一个事务那条记录与它装不下再跨出去的几条（D23（journal 的角色与格式） 已定项 17）——都属于最后一个事务，
 /// 共享它的事务号（已定项 7「同一事务的全部记录共享它」）。
 #[must_use]
-pub fn transaction_offset_of_each_record_of_the_publish(rewritten: &[TransactionUnit]) -> Vec<u64> {
+pub fn transaction_offset_of_each_record_of_the_publish(
+    rewritten: &[TransactionUnit],
+    named_entry_capacity: JournalRecordNamedEntryCapacity,
+) -> Vec<u64> {
     let transactions_named_one_record_each = rewritten
         .iter()
         .filter(|identity| matches!(identity, TransactionUnit::Data(_)))
         .count()
         .saturating_sub(1);
-    (0..roles_named_by_each_record_of_the_publish(rewritten).len())
+    (0..roles_named_by_each_record_of_the_publish(rewritten, named_entry_capacity).len())
         .map(|record_offset_in_this_publish| {
             u64::try_from(record_offset_in_this_publish.min(transactions_named_one_record_each))
                 .expect("一次发布的事务数装得进 u64")
@@ -3493,7 +4895,8 @@
     }
 }
 
-/// 准入之后的那一段：释放、分配、装单元、落盘。失败时分配器由调用方退回，这里不管。
+/// 准入之后的那一段：释放、分配、装单元，交回装好、还没落盘的字节（按持久顺序排好的 [`PublishWrites`]）与这一版
+/// （写账是空的，落盘由调用方做：`persist_the_publish_or_freeze_it`）。失败时分配器由调用方退回，这里不管。
 #[allow(
     clippy::too_many_lines,
     clippy::too_many_arguments,
@@ -3509,15 +4912,17 @@
     release: &[Placement],
     quarantine: &[CopyQuarantinedAfterReleaseChecksumMismatch],
     trees: FileVersionTreeIdentifiers,
-) -> Result<TransactionOutput, PublishError> {
+) -> Result<(PublishWrites, TransactionOutput), PublishError> {
     let rewritten = &resolved.rewritten_roles;
     let txg = plan.txg;
     let instance = plan.instance;
     // 这次发布切成几条记录、每条点名哪几个角色、属于第几个事务（D23（journal 的角色与格式） 已定项 17：最后一个事务装不下一条
     // 记录时末条再跨记录）。
-    let roles_named_by_each_record = roles_named_by_each_record_of_the_publish(rewritten);
+    let named_entry_capacity = pool.journal_record_named_entry_capacity();
+    let roles_named_by_each_record =
+        roles_named_by_each_record_of_the_publish(rewritten, named_entry_capacity);
     let transaction_offset_of_each_record =
-        transaction_offset_of_each_record_of_the_publish(rewritten);
+        transaction_offset_of_each_record_of_the_publish(rewritten, named_entry_capacity);
     let records_in_this_publish =
         u64::try_from(roles_named_by_each_record.len()).expect("记录条数");
     // 一个数据单元一个事务（C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案）：第 k 个事务是第 `plan.transaction + k` 号，
@@ -3664,14 +5069,6 @@
 
     // t6 记账树（D5（快照 / 空间记账机制） 已定项 8）：两盘 15 行——带设备维的六项每盘一行、池级三行；
     // 全部来自分配器在分配那一刻增量维护的数，不扫盘（`.claude/rules/fs-design.md` 第一格）。seq 一律 1（D8（核心索引结构） 已定项 10）。
-    let pool_wide = |statistic: u16, tree: TreeIdentifier, value: u64| AccountingEntry {
-        statistic,
-        tree,
-        device: DeviceIdentity(STATISTIC_NO_DEVICE_DIMENSION),
-        generation: txg,
-        value,
-        sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
-    };
     // inode 号水位 = 下一个可用号（D5（快照 / 空间记账机制） 已定项 4 第 12 项）：每次发布重写这一行，
     // 这次建了几个 inode 就在上一版的水位上加几。树里最大的 key 恒小于它（I-9.6（水位大于两处最大号））——
     // 这里按「已经发出去的号」算，不按「树里现在有什么」算：号一旦发出去就不再复用（D8（核心索引结构） 已定项 6）。
@@ -3682,82 +5079,69 @@
         // 第一个事务：树里只有 inode 1，下一个可用号是 2（字节表六那一行）。
         None => FIRST_INODE_NUMBER + 1,
     } + new_inodes_of_this_publish;
-    let mut accounting_entries = vec![
-        pool_wide(
-            STATISTIC_INODE_WATERMARK,
-            trees.inode,
-            inode_number_watermark,
-        ),
-        // 这两行与准入读数读的是同一个常量（`admission`）：一处定义，两处各抄一个 0 会分叉。
-        pool_wide(
-            STATISTIC_PENDING_DELETE_BYTES,
-            TreeIdentifier(TREE_IDENTIFIER_NONE),
-            crate::admission::PENDING_DELETE_OF_THE_FIRST_VERSION.0,
-        ),
-        pool_wide(
-            STATISTIC_COMMITTED_RESERVATION_BYTES,
-            TreeIdentifier(TREE_IDENTIFIER_NONE),
-            crate::admission::COMMITTED_RESERVATION_OF_THE_FIRST_VERSION.0,
-        ),
-    ];
-    for device_map in &allocator.devices {
-        let per_device = |statistic: u16, value: u64| AccountingEntry {
-            statistic,
-            tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
-            device: device_map.device,
-            generation: txg,
-            value,
-            sequence: ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF,
-        };
-        accounting_entries.push(per_device(
-            STATISTIC_ALLOCATED_BYTES,
-            device_map.allocated_slots() * SLOT_BYTES,
-        ));
-        // 第 2 项独立维护：分配器在分配那一刻减它，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️）。
-        accounting_entries.push(per_device(
-            STATISTIC_FREE_BYTES,
-            device_map.free_slots() * SLOT_BYTES,
-        ));
-        accounting_entries.push(per_device(
-            STATISTIC_UNRECLAIMABLE_BYTES,
-            crate::admission::UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED.0,
-        ));
-        // 第 5 项：已释放、还在 defer 窗口里的（它们仍算在已分配里：占着空间、被根环里的有效根引用）。
-        accounting_entries.push(per_device(
-            STATISTIC_DEFER_QUEUE_BYTES,
-            device_map.deferred_slots() * SLOT_BYTES,
-        ));
-        accounting_entries.push(per_device(
-            STATISTIC_FRAGMENTATION_RUNS,
-            device_map.free_runs(),
-        ));
-        accounting_entries.push(per_device(
-            STATISTIC_EMPTY_CLUSTER_SEGMENTS,
-            device_map.empty_segments(),
-        ));
-    }
-    assert_eq!(
-        accounting_entries.len(),
-        accounting_entry_count(allocator.devices.len()),
-        "准入按 accounting_entry_count 判过装不装得下：这里装的行数要与它相等，改了行的构成要一起改那两个常量"
+    let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator
+        .devices
+        .iter()
+        .map(|device_map| device_map.device)
+        .collect();
+    let accounting_entries = accounting_entries_of_this_publish(
+        txg,
+        trees.inode,
+        &device_identities_of_the_accounting_rows,
+        &|row| match row {
+            PoolWideAccountingRow::InodeNumberWatermark => inode_number_watermark,
+            // 这两行与准入读数读的是同一个常量（`admission`）：一处定义，两处各抄一个 0 会分叉。
+            PoolWideAccountingRow::PendingDeleteBytes => {
+                crate::admission::PENDING_DELETE_OF_THE_FIRST_VERSION.0
+            }
+            PoolWideAccountingRow::CommittedReservationBytes => {
+                crate::admission::COMMITTED_RESERVATION_OF_THE_FIRST_VERSION.0
+            }
+        },
+        &|row, device| {
+            let device_map = allocator
+                .devices
+                .iter()
+                .find(|device_map| device_map.device == device)
+                .expect("行按分配器里的那几块盘展开");
+            match row {
+                PerDeviceAccountingRow::AllocatedBytes => device_map.allocated_slots() * SLOT_BYTES,
+                // 第 2 项独立维护：分配器在分配那一刻减它，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️）。
+                PerDeviceAccountingRow::FreeBytes => device_map.free_slots() * SLOT_BYTES,
+                PerDeviceAccountingRow::UnreclaimableBytes => {
+                    crate::admission::UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED.0
+                }
+                // 第 5 项：已释放、还在 defer 窗口里的（它们仍算在已分配里：占着空间、被根环里的有效根引用）。
+                PerDeviceAccountingRow::DeferQueueBytes => device_map.deferred_slots() * SLOT_BYTES,
+                PerDeviceAccountingRow::FragmentationRuns => device_map.free_runs(),
+                PerDeviceAccountingRow::EmptyClusterSegments => device_map.empty_segments(),
+            }
+        },
     );
-    accounting_entries.sort_by_key(AccountingEntry::sort_key);
-    let accounting_sequence = sequences.next(trees.accounting, txg, instance);
-    let accounting_unit = build_index_node(
-        trees.accounting,
-        0,
-        usize::try_from(ACCOUNTING_KEY_BYTES).expect("22"),
-        &accounting_entries[0].key_bytes(),
-        &accounting_entries[accounting_entries.len() - 1].key_bytes(),
+    let multi_level_tree_context = MultiLevelTreeBuildContext {
         txg,
         filesystem_identifier,
         instance,
-        accounting_sequence,
-        u16::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
+        device_identities: &device_identities,
+    };
+    // 记账树按计划装（D8（核心索引结构） 已定项 11）：行装不下一个节点时从中间切，节点按先叶后根发出生序号、取这次的落点。
+    let accounting_built = build_multi_level_tree(
+        MultiLevelCodeTwoTree::Accounting,
+        trees.accounting,
+        &resolved.accounting_tree,
+        previous,
         &accounting_entries
             .iter()
-            .map(AccountingEntry::to_bytes)
-            .collect::<Vec<_>>(),
+            .map(|entry| {
+                (
+                    CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::ACCOUNTING),
+                    entry.to_bytes(),
+                )
+            })
+            .collect(),
+        &multi_level_tree_context,
+        &slots,
+        &mut sequences,
     );
 
     let node_pointer = |tree: TreeIdentifier,
@@ -3801,15 +5185,10 @@
         &allocation_unit,
         allocation_sequence,
     );
-    let accounting_pointer = node_pointer(
-        trees.accounting,
-        TransactionUnit::AccountingTree,
-        &accounting_unit,
-        accounting_sequence,
-    );
+    let accounting_pointer = accounting_built.version.root_pointer();
 
-    // t7 中央映射树（字节表三·二）：码 1 每个数据单元一条 + 码 2 / 码 3 四条再加每片 inode 叶容器一条；
-    // 映射树自己、树表、实例表豁免（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
+    // t7 中央映射树（字节表三·二）：码 1 每个数据单元一条 + 码 2 / 码 3 的 extent 根、每片 inode 叶容器、inode 根、分配记录树、
+    // 记账树的每个节点各一条；映射树自己、树表、实例表豁免（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
     // 照抄的角色 key 照旧、位置照旧；重写的按这次的指针算。数据单元的码 1 key 带写序（事务号），
     // 一事务一单元 ⇒ 同一个文件的各个单元 key 不撞（D19（块指针的结构与宽度预算） 已定项 6 压在切分纪律上）。
     let mut mapped_units_with_locations: Vec<(TransactionUnit, Vec<u8>, [LocationEntry; 2])> =
@@ -3849,49 +5228,53 @@
             mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer),
             allocation_pointer.locations,
         ),
-        (
-            TransactionUnit::AccountingTree,
-            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, accounting_pointer),
-            accounting_pointer.locations,
-        ),
     ]);
+    for (node, pointer) in accounting_built
+        .version
+        .shape
+        .nodes()
+        .iter()
+        .zip(&accounting_built.version.pointers)
+    {
+        mapped_units_with_locations.push((
+            MultiLevelCodeTwoTree::Accounting
+                .role_of_node(node.position, &accounting_built.version.shape),
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, *pointer),
+            pointer.locations,
+        ));
+    }
     let mapped_units: Vec<(TransactionUnit, Vec<u8>)> = mapped_units_with_locations
         .iter()
         .map(|(identity, key, _)| (*identity, key.clone()))
         .collect();
-    let mut mapping_entries: Vec<(Vec<u8>, [LocationEntry; 2])> = mapped_units_with_locations
-        .into_iter()
-        .map(|(_, key, locations)| (key, locations))
-        .collect();
     assert_eq!(
-        mapping_entries.len(),
-        mapping_entry_count(inode_tree_units.leaf_containers.len(), data_pointers.len()),
-        "准入按 mapping_entry_count 判过装不装得下：这里装的条目数要与它相等，改了进映射的角色要一起改那个常量"
-    );
-    mapping_entries.sort_by_key(|(key, _)| mapping_key_sort_key(key));
-    let mapping_sequence = sequences.next(trees.central_mapping, txg, instance);
-    let mapping_unit = build_index_node(
-        trees.central_mapping,
-        0,
-        usize::try_from(MAPPING_KEY_BYTES).expect("27"),
-        &mapping_entries[0].0,
-        &mapping_entries[mapping_entries.len() - 1].0,
-        txg,
-        filesystem_identifier,
-        instance,
-        mapping_sequence,
-        u16::try_from(MAPPING_ENTRY_BYTES).expect("55"),
-        &mapping_entries
-            .iter()
-            .map(|(key, locations)| build_mapping_entry(key, *locations))
-            .collect::<Vec<_>>(),
+        mapped_units, resolved.mapped_units,
+        "中央映射树的形状按 resolve 现算的那一份映射 key 算过：这里装出来的 key 要逐项相等（出生序号按 bump 次序发，D19 已定项 9）"
     );
-    let mapping_pointer = node_pointer(
+    let mapping_entries_by_key: BTreeMap<CodeTwoTreeKey, Vec<u8>> = mapped_units_with_locations
+        .iter()
+        .map(|(_, key, locations)| {
+            (
+                CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING),
+                build_mapping_entry(key, *locations),
+            )
+        })
+        .collect();
+    let mapping_keys: Vec<Vec<u8>> = mapping_entries_by_key
+        .keys()
+        .map(|key| key.bytes().to_vec())
+        .collect();
+    let mapping_built = build_multi_level_tree(
+        MultiLevelCodeTwoTree::CentralMapping,
         trees.central_mapping,
-        TransactionUnit::MappingTree,
-        &mapping_unit,
-        mapping_sequence,
+        &resolved.central_mapping_tree,
+        previous,
+        &mapping_entries_by_key,
+        &multi_level_tree_context,
+        &slots,
+        &mut sequences,
     );
+    let mapping_pointer = mapping_built.version.root_pointer();
 
     // t8 树表单元：七条按树 ID 升序（D8（核心索引结构） 已定项 8）；映射树的根住根记录、不进树表（D19（块指针的结构与宽度预算） 已定项 11）；
     // 头 ID（D5（快照 / 空间记账机制） 已定项 9）：inode 树写自己、extent 树写它服务的头，其余 0。
@@ -3978,7 +5361,7 @@
     );
 
     // 这一版全部角色的单元：重写的是这次装的，照抄的从上一版拷（按 bump 次序：每个数据单元、extent 根、每片 inode 叶容器、
-    // inode 根、四个固定点单元，实例表链的各片在末尾）。
+    // inode 根、分配记录树、记账树与中央映射树的每个节点（树内先叶后根）、树表，实例表链的各片在末尾）。
     let rewritten_unit = |identity: TransactionUnit, bytes: Vec<u8>| PublishedUnit {
         slot: slot_of(identity),
         identity,
@@ -4020,14 +5403,8 @@
         TransactionUnit::AllocationTree,
         allocation_unit.clone(),
     ));
-    units.push(rewritten_unit(
-        TransactionUnit::AccountingTree,
-        accounting_unit.clone(),
-    ));
-    units.push(rewritten_unit(
-        TransactionUnit::MappingTree,
-        mapping_unit.clone(),
-    ));
+    units.extend(accounting_built.units.iter().cloned());
+    units.extend(mapping_built.units.iter().cloned());
     units.push(rewritten_unit(
         TransactionUnit::TreeTable,
         tree_table_unit.clone(),
@@ -4079,8 +5456,17 @@
                     .birth_sequence,
             ),
             TransactionUnit::AllocationTree => node_key_tail(instance, allocation_sequence),
-            TransactionUnit::AccountingTree => node_key_tail(instance, accounting_sequence),
-            TransactionUnit::MappingTree => node_key_tail(instance, mapping_sequence),
+            TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
+            | TransactionUnit::AccountingTree => node_key_tail(
+                instance,
+                accounting_built.birth_sequence_of_a_rewritten_node(identity),
+            ),
+            TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
+                node_key_tail(
+                    instance,
+                    mapping_built.birth_sequence_of_a_rewritten_node(identity),
+                )
+            }
             TransactionUnit::TreeTable => node_key_tail(instance, tree_table_sequence),
             TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                 node_key_tail(
@@ -4183,41 +5569,23 @@
         // 两处都写就成了同一个量的两份手抄。`root_record_of_a_file_version_leaves_the_allocation_record_tree_pointer_zero` 钉住。
         allocation_record_tree_root: NodePointer::empty_root(),
     };
-    let root_slot = root.to_slot(pool.root_slot_bytes());
-
     // 持久顺序（D16（发布语义） 已定项 7）：这次重写的单元 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 系统配置槽轮换。
-    // 中途失败时这次已记的写要交出去（增补 2 第 20b 行）：六步收在一个闭包里，失败在这里记账再把错原样交回——
-    // 账是两次快照之差、只在成功路径上取，失败那次落盘的写不交出去就不属于任何一次发布，与设备一层的合计对不上。
-    let writes_before_this_publish = pool.writes_by_structure_kind.clone();
-    let persist = |writer: &mut PoolWriter<'_, Device>| -> Result<(), BlockDeviceError> {
-        for unit in &written_units {
-            writer.perform(CommitStep::WriteUnitToEveryDevice {
-                slot: unit.slot,
-                unit: &unit.bytes,
-                identity: unit.identity,
-            })?;
-        }
-        writer.perform(CommitStep::Barrier)?;
-        for written_record in &written_records {
-            writer.perform(CommitStep::WriteJournalRecordToEveryDevice {
+    // 这里只装、不落盘：调用方拿它落盘，中途失败时这次已记的写进失败账（增补 2 第 20b 行）、这次发布冻结等原样重发。
+    let writes = PublishWrites {
+        units: written_units.into_iter().cloned().collect(),
+        records: written_records
+            .iter()
+            .map(|written_record| JournalRecordWrite {
                 counter: written_record.record.counter,
-                record: &written_record.bytes,
-            })?;
-        }
-        writer.perform(CommitStep::Barrier)?;
+                bytes: written_record.bytes.clone(),
+            })
+            .collect(),
+        checkpoint_txg: txg,
+        root_slot: root.to_slot(pool.root_slot_bytes()),
         // 系统配置里的 tail 存这次发布末条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 18）。
-        persist_the_root_then_rotate_the_system_configuration(
-            writer,
-            txg,
-            &root_slot,
-            last_counter_of_this_publish,
-            instance,
-        )
+        journal_tail: last_counter_of_this_publish,
+        journal_instance: instance,
     };
-    if let Err(cause) = persist(pool) {
-        pool.count_failed_publish(&writes_before_this_publish);
-        return Err(PublishError::BlockDevice(cause));
-    }
 
     let WrittenJournalRecord {
         record,
@@ -4225,7 +5593,7 @@
     } = written_records
         .pop()
         .expect("一次发布至少一条记录（`roles_named_by_each_record_of_the_publish` 至少给一项）");
-    Ok(TransactionOutput {
+    let output = TransactionOutput {
         root,
         record,
         record_bytes,
@@ -4233,9 +5601,11 @@
         units,
         rewritten: rewritten.to_vec(),
         data_pointers,
-        mapping_keys: mapping_entries.into_iter().map(|(key, _)| key).collect(),
+        mapping_keys,
         allocation_records,
         accounting_entries,
+        accounting_tree: accounting_built.version,
+        central_mapping_tree: mapping_built.version,
         tree_table_entries,
         tree_identifiers: trees,
         inode_record,
@@ -4254,10 +5624,168 @@
         highest_transaction_number_in_this_instance: plan
             .highest_transaction_number_before_this_publish
             .max(last_transaction_of_this_publish),
-        writes: pool
-            .writes_by_structure_kind
-            .since(&writes_before_this_publish),
-    })
+        // 落盘之后由 `persist_the_publish_or_freeze_it` 换成这次真写出去的账。
+        writes: WritesByStructureKind::NOTHING_WRITTEN,
+    };
+    Ok((writes, output))
+}
+
+/// 装一棵多层码 2 树时整次发布共用的身份字段。
+struct MultiLevelTreeBuildContext<'build> {
+    txg: CheckpointTxg,
+    filesystem_identifier: &'build [u8; 16],
+    instance: InstanceGeneration,
+    /// 位置条目按设备身份升序（I-2.5）。
+    device_identities: &'build [DeviceIdentity],
+}
+
+/// 一棵多层码 2 树这一版装好的样子。
+struct BuiltMultiLevelTree {
+    version: CodeTwoTreeVersion,
+    /// 这一版的全部节点单元，按 bump 次序（先叶后根、同层按 key 升序），根在最末。
+    units: Vec<PublishedUnit>,
+    /// 这次重写的节点各自的出生序号（点名项的 key 尾段要它）。
+    birth_sequences_of_rewritten_nodes: BTreeMap<TransactionUnit, BirthSequence>,
+}
+
+impl BuiltMultiLevelTree {
+    /// # Panics
+    /// 这个角色这次没重写：点名项只点名这次重写的角色。
+    fn birth_sequence_of_a_rewritten_node(&self, identity: TransactionUnit) -> BirthSequence {
+        *self
+            .birth_sequences_of_rewritten_nodes
+            .get(&identity)
+            .expect("点名的是这次重写的节点")
+    }
+}
+
+/// 按计划装一棵多层码 2 树这一版的全部节点（D8（核心索引结构） 已定项 11）：重写的节点按 bump 次序逐个装——先叶后根、
+/// 同层按 key 升序（D3（空间分配） 已定项 10 ⑤）、出生序号按同一个次序发（D19（块指针的结构与宽度预算） 已定项 9）、
+/// 落点取这次分配的；叶装这一版那几把 key 的完整条目，内部节点装「分隔 key + 孩子这一版的指针」（孩子先装好，指针才写得出），
+/// 头里的 key 区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2），层级取计划里的位置。照抄的节点字节、落点、指针全取
+/// 上一版那个位置上的，角色换成这一版的位置。
+///
+/// # Panics
+/// 计划里叶的 key 与 `leaf_entry_bytes_by_key` 的 key 不是同一个集合（计划按 `resolve` 现算的 key 算，条目按这次装出来的，
+/// 两边出生序号的发号次序不同步就对不上）；照抄的节点不在上一版里；重写的节点没拿到落点——三样都是发布路径自己的不变量。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "装一棵树要的八样：哪一棵、它的号、计划、上一版、条目、身份字段、落点、发号器，各自独立"
+)]
+fn build_multi_level_tree(
+    tree: MultiLevelCodeTwoTree,
+    tree_identifier: TreeIdentifier,
+    plan: &CodeTwoTreePlan,
+    previous: Option<&TransactionOutput>,
+    leaf_entry_bytes_by_key: &BTreeMap<CodeTwoTreeKey, Vec<u8>>,
+    context: &MultiLevelTreeBuildContext<'_>,
+    slots: &BTreeMap<TransactionUnit, SlotNumber>,
+    sequences: &mut BirthSequenceAllocator,
+) -> BuiltMultiLevelTree {
+    let planned_keys: Vec<&CodeTwoTreeKey> = plan.shape.keys_in_order();
+    assert!(
+        planned_keys.len() == leaf_entry_bytes_by_key.len()
+            && planned_keys
+                .iter()
+                .zip(leaf_entry_bytes_by_key.keys())
+                .all(|(planned, built)| *planned == built),
+        "{tree:?}：计划里叶的 key 与这次装出来的条目的 key 不是同一个集合"
+    );
+    let key_width = tree.key_field_widths().key_width_in_bytes();
+    let key_ranges = crate::code_two_tree::key_ranges_in_bump_order(&plan.shape);
+    let mut pointers_by_position: BTreeMap<CodeTwoTreeNodePosition, NodePointer> = BTreeMap::new();
+    let mut pointers: Vec<NodePointer> = Vec::with_capacity(plan.shape.nodes().len());
+    let mut units: Vec<PublishedUnit> = Vec::with_capacity(plan.shape.nodes().len());
+    let mut birth_sequences_of_rewritten_nodes = BTreeMap::new();
+    // 迭代上界是这一版的节点数；跨轮携带的是已经装好的节点的指针（父节点的条目要孩子这一版的指针，孩子在 bump 次序里排在前面）。
+    for ((node, origin), (smallest_key, largest_key)) in
+        plan.shape.nodes().iter().zip(&plan.origins).zip(key_ranges)
+    {
+        let identity = tree.role_of_node(node.position, &plan.shape);
+        let (pointer, unit) = match origin {
+            CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => {
+                let previous_version = previous.expect("照抄的节点只出现在接着上一版的发布里");
+                let previous_tree = previous_version.multi_level_tree(tree);
+                let carried = previous_version
+                    .unit(tree.role_of_node(*previous_position, &previous_tree.shape));
+                (
+                    previous_tree.pointer_of(*previous_position),
+                    PublishedUnit {
+                        slot: carried.slot,
+                        identity,
+                        bytes: carried.bytes.clone(),
+                    },
+                )
+            }
+            CodeTwoTreeNodeOrigin::RewrittenThisPublish => {
+                let (entry_width, entries): (usize, Vec<Vec<u8>>) = match &node.contents {
+                    CodeTwoTreeNodeContents::Leaf { keys } => (
+                        tree.leaf_entry_width_in_bytes(),
+                        keys.iter()
+                            .map(|key| leaf_entry_bytes_by_key[key].clone())
+                            .collect(),
+                    ),
+                    CodeTwoTreeNodeContents::Internal { children } => (
+                        internal_entry_width_in_bytes(key_width),
+                        children
+                            .iter()
+                            .map(|child| {
+                                build_internal_entry(
+                                    child.separator_key.bytes(),
+                                    pointers_by_position[&child.child],
+                                )
+                            })
+                            .collect(),
+                    ),
+                };
+                let birth_sequence = sequences.next(tree_identifier, context.txg, context.instance);
+                let bytes = build_index_node(
+                    tree_identifier,
+                    node.position.level,
+                    key_width,
+                    &smallest_key,
+                    &largest_key,
+                    context.txg,
+                    context.filesystem_identifier,
+                    context.instance,
+                    birth_sequence,
+                    u16::try_from(entry_width).expect("条目宽 2 字节"),
+                    &entries,
+                );
+                let slot = *slots
+                    .get(&identity)
+                    .expect("这次重写的节点都在角色清单里、取了落点");
+                birth_sequences_of_rewritten_nodes.insert(identity, birth_sequence);
+                (
+                    NodePointer {
+                        head: PointerHead {
+                            birth_tree: tree_identifier,
+                            birth_txg: context.txg,
+                        },
+                        locations: location_entries(context.device_identities, slot, &bytes),
+                        instance: context.instance,
+                        birth_sequence,
+                    },
+                    PublishedUnit {
+                        slot,
+                        identity,
+                        bytes,
+                    },
+                )
+            }
+        };
+        pointers_by_position.insert(node.position, pointer);
+        pointers.push(pointer);
+        units.push(unit);
+    }
+    BuiltMultiLevelTree {
+        version: CodeTwoTreeVersion {
+            shape: plan.shape.clone(),
+            pointers,
+        },
+        units,
+        birth_sequences_of_rewritten_nodes,
+    }
 }
 
 #[cfg(test)]
@@ -4536,7 +6064,10 @@
     fn the_last_transaction_spills_over_as_many_records_as_its_named_units_need_and_they_all_share_its_transaction(
     ) {
         let spilling_with_data = rewritten_roles(2, 130);
-        let records_with_data = roles_named_by_each_record_of_the_publish(&spilling_with_data);
+        let records_with_data = roles_named_by_each_record_of_the_publish(
+            &spilling_with_data,
+            JournalRecordNamedEntryCapacity::FromTheRecordFormat,
+        );
         assert_eq!(
             records_with_data.iter().map(Vec::len).collect::<Vec<_>>(),
             vec![1, 67, 67, 3],
@@ -4552,15 +6083,20 @@
             "几条合起来按 bump 次序恰好点名每个重写角色一次"
         );
         assert_eq!(
-            transaction_offset_of_each_record_of_the_publish(&spilling_with_data),
+            transaction_offset_of_each_record_of_the_publish(
+                &spilling_with_data,
+                JournalRecordNamedEntryCapacity::FromTheRecordFormat
+            ),
             vec![0, 1, 1, 1],
             "跨出去的两条属于最后一个事务"
         );
 
         let spilling_without_data = rewritten_roles(0, 62);
         assert_eq!(spilling_without_data.len(), 68);
-        let records_without_data =
-            roles_named_by_each_record_of_the_publish(&spilling_without_data);
+        let records_without_data = roles_named_by_each_record_of_the_publish(
+            &spilling_without_data,
+            JournalRecordNamedEntryCapacity::FromTheRecordFormat,
+        );
         assert_eq!(
             records_without_data
                 .iter()
@@ -4569,31 +6105,46 @@
             vec![67, 1]
         );
         assert_eq!(
-            transaction_offset_of_each_record_of_the_publish(&spilling_without_data),
+            transaction_offset_of_each_record_of_the_publish(
+                &spilling_without_data,
+                JournalRecordNamedEntryCapacity::FromTheRecordFormat
+            ),
             vec![0, 0],
             "一个事务跨两条"
         );
 
         let fitting = rewritten_roles(3, 1);
         assert_eq!(
-            roles_named_by_each_record_of_the_publish(&fitting)
-                .iter()
-                .map(Vec::len)
-                .collect::<Vec<_>>(),
+            roles_named_by_each_record_of_the_publish(
+                &fitting,
+                JournalRecordNamedEntryCapacity::FromTheRecordFormat
+            )
+            .iter()
+            .map(Vec::len)
+            .collect::<Vec<_>>(),
             vec![1, 1, 8]
         );
         assert_eq!(
-            transaction_offset_of_each_record_of_the_publish(&fitting),
+            transaction_offset_of_each_record_of_the_publish(
+                &fitting,
+                JournalRecordNamedEntryCapacity::FromTheRecordFormat
+            ),
             vec![0, 1, 2]
         );
 
         assert_eq!(
-            roles_named_by_each_record_of_the_publish(&[]),
+            roles_named_by_each_record_of_the_publish(
+                &[],
+                JournalRecordNamedEntryCapacity::FromTheRecordFormat
+            ),
             vec![Vec::<TransactionUnit>::new()],
             "一个角色都不重写的发布照样一条记录"
         );
         assert_eq!(
-            transaction_offset_of_each_record_of_the_publish(&[]),
+            transaction_offset_of_each_record_of_the_publish(
+                &[],
+                JournalRecordNamedEntryCapacity::FromTheRecordFormat
+            ),
             vec![0]
         );
     }
diff -ruN -x target tree/crates/singlefs-core/src/write_accounting.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-core/src/write_accounting.rs	2026-09-24 16:15:19.532852641 +0000
+++ tree/crates/singlefs-core/src/write_accounting.rs	2026-09-24 16:50:54.714292753 +0000
@@ -53,8 +53,12 @@
             TransactionUnit::InodeLeafContainer(_) => WrittenStructureKind::InodeTreeLeafContainer,
             TransactionUnit::InodeRoot => WrittenStructureKind::InodeTreeRoot,
             TransactionUnit::AllocationTree => WrittenStructureKind::AllocationRecordTreeNode,
-            TransactionUnit::AccountingTree => WrittenStructureKind::AccountingTreeNode,
-            TransactionUnit::MappingTree => WrittenStructureKind::CentralMappingTreeNode,
+            // 多层之后根之下的节点与根同一种：按树数，不按层数（增补 1 的账按结构种类记）。
+            TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
+            | TransactionUnit::AccountingTree => WrittenStructureKind::AccountingTreeNode,
+            TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
+                WrittenStructureKind::CentralMappingTreeNode
+            }
             TransactionUnit::TreeTable => WrittenStructureKind::TreeTableUnit,
             TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                 WrittenStructureKind::InstanceTableUnit
diff -ruN -x target tree/crates/singlefs-format/src/lib.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-format/src/lib.rs	2026-09-24 08:24:01.668782064 +0000
+++ tree/crates/singlefs-format/src/lib.rs	2026-09-24 18:48:58.241580116 +0000
@@ -127,6 +127,12 @@
 /// 记账条目一条：key 22 + value 8 + seq 4（D5（快照 / 空间记账机制） 已定项 5；D8（核心索引结构） 已定项 7）。format-const: ACCOUNTING_ENTRY_BYTES
 pub const ACCOUNTING_ENTRY_BYTES: u64 = 34;
 
+/// 记账树内部节点条目：分隔 key 22 + 子指针 86（D8（核心索引结构） 已定项 11「内部节点的条目 = 本树 key + 子指针 86」）。format-const: ACCOUNTING_INTERNAL_ENTRY_BYTES
+pub const ACCOUNTING_INTERNAL_ENTRY_BYTES: u64 = 108;
+
+/// 中央映射树内部节点条目：分隔 key 27 + 子指针 86（D8（核心索引结构） 已定项 11）。format-const: CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES
+pub const CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES: u64 = 113;
+
 /// 第一个事务这次发布写出的记账行数（D5（快照 / 空间记账机制） 已定项 8，2026-09-14 用户定案 15 行）。
 pub const FIRST_TRANSACTION_ACCOUNTING_ROWS: u64 = 15;
 
@@ -222,6 +228,27 @@
 
 pub const ROOT_RING_PRIME_STEP: u64 = 3;
 pub const ROOT_RING_CHUNK_BYTES: u64 = 1024 * 1024;
+
+/// 回退见证表（D23（journal 的角色与格式） 已定项 14「回退见证」，用户 2026-09-24 定）住系统配置槽里、紧接着字段表之后：
+/// 从槽内偏移 481（`SYSTEM_CONFIGURATION_BYTES`）起，罩在系统配置的整槽校验和里、越过 512 字节。落点随实现取在这里，
+/// 写回 D22（单元原子性怎么合成） 已定项 9 的字段表归书记员。
+pub const ROLLBACK_WITNESS_TABLE_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT: u64 =
+    SYSTEM_CONFIGURATION_BYTES;
+
+/// 见证表头：条数 1 字节（上限 47 装得下）。
+pub const ROLLBACK_WITNESS_COUNT_BYTES: u64 = 1;
+
+/// 一个见证条目：新实例代号 4 + 回退目标 R_old 的实例代号 4 + R_old 的 txg 8（已定项 14「回退见证」）。
+pub const ROLLBACK_WITNESS_ENTRY_BYTES: u64 = 16;
+
+/// 见证表按 S 的上界定宽：条数上限 = 根环槽数减 1（R × S − 1，只由根环几何定），S 取格式承诺区间的上界 16 时是 47 条。
+/// 一个池自己的上限按它系统配置里的 S 算（`R × S − 1`），多出来的条目位写 0。
+pub const ROLLBACK_WITNESS_ENTRIES_MAXIMUM: u64 =
+    ROOT_RING_REGIONS * ROOT_RING_SLOTS_PER_REGION_MAXIMUM - 1;
+
+/// 见证表的定宽：1 + 47 × 16 = 753 字节，占槽内 [481, 1234)。
+pub const ROLLBACK_WITNESS_TABLE_BYTES: u64 =
+    ROLLBACK_WITNESS_COUNT_BYTES + ROLLBACK_WITNESS_ENTRIES_MAXIMUM * ROLLBACK_WITNESS_ENTRY_BYTES;
 /// 根环起点是 16 KiB 槽号（1 MiB）。
 pub const ROOT_RING_BASE_SLOT: u64 = 64;
 /// 根环区域归属第一版写死 0 / 1 / 0（D2（RAID 条带策略） 已定项 7，2026-09-14 用户定案）。
@@ -369,6 +396,16 @@
             ACCOUNTING_KEY_BYTES + 8 + 4,
             "记账条目 = key + value 8 + seq 4"
         );
+        assert_eq!(
+            ACCOUNTING_INTERNAL_ENTRY_BYTES,
+            ACCOUNTING_KEY_BYTES + NODE_POINTER_BYTES,
+            "记账树内部条目 = key + 指向码 2 的指针"
+        );
+        assert_eq!(
+            CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES,
+            MAPPING_KEY_BYTES + NODE_POINTER_BYTES,
+            "中央映射树内部条目 = key + 指向码 2 的指针"
+        );
     }
 
     /// 每棵树的码 2 节点头宽写成整数字面量（门禁 27 号要核），与 `index_node_header_bytes` 在那棵树的 key 宽上算出来的一个数不差；
@@ -407,6 +444,30 @@
         );
     }
 
+    /// 回退见证表紧接着字段表、越过 512、装在 4096 的槽里（D23 已定项 14「回退见证」：放在字段表之后、越过 512，靠整槽校验和认撕裂）。
+    #[test]
+    fn the_rollback_witness_table_follows_the_field_table_crosses_512_and_fits_in_the_slot() {
+        let table_end = std::hint::black_box(
+            ROLLBACK_WITNESS_TABLE_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT
+                + ROLLBACK_WITNESS_TABLE_BYTES,
+        );
+        assert_eq!(
+            ROLLBACK_WITNESS_TABLE_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT,
+            481
+        );
+        assert_eq!(
+            ROLLBACK_WITNESS_ENTRIES_MAXIMUM, 47,
+            "R × S 上界 − 1 = 3 × 16 − 1"
+        );
+        assert_eq!(ROLLBACK_WITNESS_TABLE_BYTES, 753);
+        assert_eq!(table_end, 1234);
+        assert!(table_end > 512, "越过 512");
+        assert!(
+            table_end <= SYSTEM_CONFIGURATION_SLOT_BYTES,
+            "装在 4096 的槽里"
+        );
+    }
+
     /// 系统配置的 481 与它所在的槽：槽宽预想 4096 时余 3615，装得下下一条 86 字节指针；按此前的 512 槽算只余 31、塞不进（D22 已定项 2 重开的理由之一）。
     #[test]
     fn system_configuration_fits_in_the_slot_with_room_for_one_more_pointer() {
diff -ruN -x target tree/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	2026-09-24 16:27:40.842241332 +0000
+++ tree/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	2026-09-24 17:01:54.423058125 +0000
@@ -51,7 +51,7 @@
 use singlefs_core::unit::{parse_index_node, unit_filesystem_identifier};
 use singlefs_core::write_accounting::{WritesByStructureKind, WrittenStructureKind};
 use singlefs_format::JOURNAL_RECORD_BYTES;
-use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
+use singlefs_harness::crash::{MemoryPool, SparseBlockDevice, SparseDevice};
 use singlefs_harness::fault_injection::{
     injected_block_device_error, FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice,
     FaultOccurrence, FaultPlacement, FaultSchedule, InjectedFault, NamedRootRingSlots,
@@ -1903,6 +1903,82 @@
     }
 }
 
+/// session s9（`m2-rootchoice-repair-r1-forks.md` 岔路单第 1 行还差项④）：op1 起的多次挂载轨迹——
+/// 第一节读法写死表「持续」＝从 op1 这次调用起到这段历史结束都有效，「瞬时」＝只在 op1 那次调用期间
+/// 有效；第八节 8.1「abandoned_roots_unreadable，按 op1 起的每一次挂载」要的轨迹正是这个函数产出的。
+/// **与 `attempt_faulted_operation` 的关键差别**：那个函数每次都从 `node.pool` 重新
+/// `devices_from_pool`，故障目标之外的写不回原池（`SparseDevice` 是普通值类型、`.clone()` 深拷贝，
+/// 不共享底层字节，这是刻意的「试一次看结局」语义，见它的用法）；这里要的是「op1 落盘之后接着挂」，
+/// 所以每一步显式把 `FaultInjectingBlockDevice::inner().image` 取出来拼回一个新的 `MemoryPool`，
+/// 喂给下一步——**这是这个函数与 `attempt_faulted_operation` 的唯一结构性差别，其余装故障、判结局的
+/// 写法逐字照抄，不引入第二套注入逻辑**。`persistent = true` 时每一步都用同一组 `fault_targets`
+/// （对应「持续」）；`persistent = false` 时只有第一步（op1 自己）带故障，之后的步不装任何故障
+/// （对应「瞬时」，且与「瞽时」的读法一致：故障在 op1 那次调用返回后即撤）。某一步交回 `Err` 就停在
+/// 那一步，`results` 里这一步与之后的步都记 `None`（挂载失败之后没有新的池状态可以接着挂，不能凭空
+/// 编一个「本该」发生的计数）。
+fn mount_writable_trajectory(
+    starting_pool: &MemoryPool,
+    parameters: &MakeFilesystemParameters,
+    fixed_geometry: FixedGeometry,
+    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
+    persistent: bool,
+    steps: usize,
+) -> Vec<Option<u64>> {
+    let mut pool = starting_pool.clone();
+    let mut results = Vec::new();
+    for step in 0..steps {
+        let active_targets: &[(DeviceIdentity, DeviceOffsetInBytes)] =
+            if step == 0 || persistent {
+                fault_targets
+            } else {
+                &[]
+            };
+        let plain = devices_from_pool(&pool, IMAGE_BYTES);
+        let mut wrapped: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
+            Vec::new();
+        for (identity, device) in plain {
+            let plan = SharedFaultPlan::unarmed(fixed_geometry);
+            if let Some((_, offset)) = active_targets
+                .iter()
+                .find(|(target_device, _)| *target_device == identity)
+            {
+                plan.arm(FaultSchedule {
+                    fault: InjectedFault::ReadFails,
+                    device: FaultDeviceSelector::OnlyDevice(identity),
+                    placement: FaultPlacement::OffsetExactly(*offset),
+                    counting: FaultCounting::AcrossThePool,
+                    occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
+                });
+            }
+            wrapped.push((
+                identity,
+                FaultInjectingBlockDevice::new(identity, device, plan),
+            ));
+        }
+        match mount_writable(parameters, &mut wrapped) {
+            Ok(mounted) => {
+                results.push(Some(mounted.output.abandoned_roots_unreadable));
+                let next_devices: BTreeMap<DeviceIdentity, SparseDevice> = wrapped
+                    .iter()
+                    .map(|(identity, device)| (*identity, device.inner().image.clone()))
+                    .collect();
+                pool = MemoryPool {
+                    devices: next_devices,
+                    device_size_in_bytes: IMAGE_BYTES,
+                };
+            }
+            Err(_) => {
+                results.push(None);
+                break;
+            }
+        }
+    }
+    while results.len() < steps {
+        results.push(None);
+    }
+    results
+}
+
 /// H1 家族里的一个「op1 之前」节点：mkfs → `mount_writable` → 首个文件 →
 /// `overwrites_before_rollback` 次覆盖写 → 关闭 → `mount_rollback` 到 `rollback_target` →
 /// `overwrites_after_rollback` 次覆盖写 → 关闭。op1 本身（受注入的一步）不在这个结构体里，
@@ -2177,6 +2253,20 @@
 /// **射程**：只走了中央映射树一条内容树（`crates::mounted_read::open_pool_for_read` 走读 extent / inode
 /// 树是为了服务文件读，与「占用集合」这个问题不是同一件事——占用靠中央映射，不靠 extent/inode）；
 /// 有快照、多版本共享落点的情形没有覆盖（第一版这批历史都不产生快照，见交回报告）。
+///
+/// **只读一层、把根当叶解**——实十九（2026-09-24 16:50 UTC 前后落地）之后，中央映射树条目数一旦
+/// 超过单节点叶容量（`transaction.rs` 第 1334 行「中央映射树叶 294」）就会长成多层，那时根节点是
+/// 内部节点（`level > 0`），装着的是子节点指针、不是 `parse_mapping_entry` 认得的叶条目格式。
+/// 这里显式核 `level == 0` 再往下解，不靠「宽度不够、`parse_mapping_entry` 自然读不出」这种隐式失败
+/// 兜底——内部条目宽约 114 字节（`transaction.rs` 同一行「内部 143」按 16384 / 143 反推），比
+/// `MAPPING_ENTRY_BYTES`=55 宽，`parse_mapping_entry` 的长度检查拦不住它，会把内部指针字节当叶
+/// 条目误解出一对看似合法的 `LocationEntry`——这正是「不许静默算错」要挡的那种失败，`level` 检查把
+/// 它变成一条会报错、不会算错的路。E158 这批历史全部只写一个文件、只有一个逻辑映射 key（跨 n1/m/σ
+/// 的全部覆盖写都在原地更新同一条条目，不新增条目），条目数最多到 1（见交回报告的现场核对），远低于
+/// 294，这批历史至今不会撞上多层——但装置不该靠这条事实免检，`level` 一变就该显式报错，不是碰巧躲过。
+///
+/// # Errors
+/// 中央映射树根两份都读不出、解不开，或根节点 `level != 0`（长成了多层，这个函数还不会整棵读）。
 fn allocation_record_tree_reachable_placements_via_central_mapping(
     plain_devices: &[(DeviceIdentity, SparseBlockDevice)],
     root: &RootRecord,
@@ -2185,6 +2275,13 @@
         .ok_or_else(|| "中央映射树根两份都读不出".to_string())?;
     let mapping_root = parse_index_node(&mapping_root_bytes)
         .map_err(|error| format!("中央映射树根解不开: {error:?}"))?;
+    if mapping_root.level != 0 {
+        return Err(format!(
+            "中央映射树根不是叶（level={}，entries={}）：树已经长成多层，这个函数只会整棵读单叶版本，不认内部节点条目",
+            mapping_root.level,
+            mapping_root.entries.len()
+        ));
+    }
     let mut placements = BTreeSet::new();
     for entry_bytes in &mapping_root.entries {
         let (_key, locations) =
@@ -2811,11 +2908,19 @@
         geometry.label,
         summary.subset_diff_pairs.len()
     ));
-    for ((device, slot), (min_generation, max_generation, min_root_txg, max_root_txg, referenced_elsewhere)) in
-        &summary.subset_diff_pair_context
+    for (
+        (device, slot),
+        (
+            minimum_generation,
+            maximum_generation,
+            minimum_abandoned_root_txg,
+            maximum_abandoned_root_txg,
+            referenced_elsewhere,
+        ),
+    ) in &summary.subset_diff_pair_context
     {
         emit_result(&format!(
-            "name=q1_2_subset_diff_pair_context geometry={} device={device} slot={slot} min_generation={min_generation} max_generation={max_generation} min_abandoned_root_txg={min_root_txg} max_abandoned_root_txg={max_root_txg} referenced_by_some_readable_roots_own_mapping={referenced_elsewhere}",
+            "name=q1_2_subset_diff_pair_context geometry={} device={device} slot={slot} minimum_generation={minimum_generation} maximum_generation={maximum_generation} minimum_abandoned_root_txg={minimum_abandoned_root_txg} maximum_abandoned_root_txg={maximum_abandoned_root_txg} referenced_by_some_readable_roots_own_mapping={referenced_elsewhere}",
             geometry.label
         ));
     }
@@ -2911,6 +3016,36 @@
                 allocation_attempt.error_debug,
                 allocation_total_bytes
             ));
+
+            // session s9（岔路单第 1 行还差项④，8.1「持续故障下 op1 及其后两次挂载各一个值；瞬时
+            // 故障下 op1 一个值、后两次（不注入）各一个值」）：用 PC1-a 已经选中的这同一个
+            // (历史, 被抛弃根, 分配记录树单元, 两份都读失败) 当轨迹的起点——`allocation_attempt`
+            // 已经证明这一格会触发，是这条轨迹天然、可复现的落点，不另挑一个未经验证的构造。
+            let persistent_trajectory = mount_writable_trajectory(
+                &node.pool,
+                &parameters,
+                fixed_geometry,
+                &allocation_fault_targets,
+                true,
+                3,
+            );
+            emit_result(&format!(
+                "name=q1_1a_op1_trajectory geometry={} mode=persistent trajectory={:?}",
+                geometry.label, persistent_trajectory
+            ));
+            let transient_trajectory = mount_writable_trajectory(
+                &node.pool,
+                &parameters,
+                fixed_geometry,
+                &allocation_fault_targets,
+                false,
+                3,
+            );
+            emit_result(&format!(
+                "name=q1_1a_op1_trajectory geometry={} mode=transient trajectory={:?}",
+                geometry.label, transient_trajectory
+            ));
+
             let pristine_allocation =
                 allocation_records_under_root(&plain_devices, &record).map(|records| {
                     records
@@ -5025,6 +5160,7 @@
     use super::*;
     use singlefs_core::address::SlotNumber;
     use singlefs_core::pointer::NodePointer;
+    use singlefs_core::unit::build_index_node;
 
     /// PC-Nw：手写一段假账（两次回退，两段被抛弃根都读得出），不碰 `crates/`，只证计数函数会数到 2。
     #[test]
@@ -5845,6 +5981,169 @@
         assert_eq!(deduplicated.len(), 2, "两条根不应该解到同一个 (设备, 偏移)");
     }
 
+    /// session s9（实十九提醒，2026-09-24 16:50 UTC 前后落地）：中央映射树条目数超过单节点叶容量
+    /// （`transaction.rs` 第 1334 行「中央映射树叶 294」）会长成多层，根节点变成内部节点
+    /// （`level > 0`）。这里手工构造一个合法但 `level=1` 的假节点，覆盖真实中央映射树根的两份物理
+    /// 拷贝，验证 `allocation_record_tree_reachable_placements_via_central_mapping` 显式拦下它——
+    /// 不是靠 `parse_mapping_entry` 的宽度检查侥幸拦住：内部条目宽约 114 字节（`transaction.rs`
+    /// 同一行「内部 143」按 16384 / 143 反推），比 `MAPPING_ENTRY_BYTES`=55 宽，那条宽度检查拦不住它。
+    #[test]
+    fn allocation_record_tree_reachable_placements_via_central_mapping_rejects_a_multi_level_root(
+    ) {
+        let node = bootstrap(&GEOMETRY_PRIMARY, 0).expect("bootstrap 应当成功");
+        let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
+        let newest = readable_roots_independent(&node.pool, &geometry_view)
+            .into_iter()
+            .max()
+            .expect("bootstrap 之后至少有一条根");
+        let record = root_record_of(&node.pool, &geometry_view, newest).expect("根记录读得出");
+        let mut plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
+
+        let real_bytes = read_node_bytes(&plain_devices, &record.mapping_root.locations)
+            .expect("真实的中央映射树根读得出");
+        let real_header = parse_index_node(&real_bytes).expect("真实的中央映射树根解得开");
+        assert_eq!(real_header.level, 0, "bootstrap 之后中央映射树根应当仍是叶");
+
+        // 内部条目宽的估计值：只要比 MAPPING_ENTRY_BYTES=55 宽，就足够触发要测的那条路
+        // （宽度检查拦不住，必须靠显式的 level 检查）。
+        let fake_entry_width = 114u16;
+        let fake_bytes = build_index_node(
+            real_header.tree,
+            1,
+            real_header.key_width,
+            &real_header.smallest_key,
+            &real_header.largest_key,
+            real_header.birth_txg,
+            &FILESYSTEM_IDENTIFIER,
+            real_header.instance,
+            real_header.birth_sequence,
+            fake_entry_width,
+            &[vec![0u8; usize::from(fake_entry_width)]],
+        );
+
+        for location in &record.mapping_root.locations {
+            let (_, device) = plain_devices
+                .iter_mut()
+                .find(|(identity, _)| *identity == location.device)
+                .expect("这块盘在池里");
+            device
+                .write_at(
+                    location.slot.to_device_offset(),
+                    &fake_bytes,
+                    WriteDurability::Plain,
+                )
+                .expect("写入假节点应当成功");
+        }
+
+        let outcome = allocation_record_tree_reachable_placements_via_central_mapping(
+            &plain_devices,
+            &record,
+        );
+        let error =
+            outcome.expect_err("level=1 的根节点必须被拒绝，不能被当成叶解出假的落点集合");
+        assert!(
+            error.contains("level=1"),
+            "错误信息要点名是哪个 level 拦下的，实际: {error}"
+        );
+    }
+
+    /// session s9（`m2-rootchoice-repair-r1-forks.md` 岔路单第 1 行还差项④）：持续故障与瞬时故障
+    /// 在 op1 起的三次挂载轨迹上必须不同——持续（`persistent=true`）故障不撤，每一步都该继续触发；
+    /// 瞬时（`persistent=false`）故障只在 op1 那一步有效，撤掉之后的两步该恢复成不触发。用与
+    /// `run_ledger_fault_positive_controls`（PC1-a）完全相同的「按固定次序找第一个满足『某条被抛弃
+    /// 根的分配记录树节点不共享』的历史」选出同一个落点，不是另挑一个未经验证的构造。
+    #[test]
+    fn mount_writable_trajectory_distinguishes_persistent_from_transient_faults() {
+        let geometry = &GEOMETRY_PRIMARY;
+        let parameters = parameters_for(geometry);
+        let fixed_geometry = fixed_geometry_for(geometry);
+        let family = ledger_fault_history_family(geometry);
+
+        for history_node in &family.nodes {
+            let node = &history_node.node;
+            let Ok(pool_geometry) = independent_geometry(&node.pool) else {
+                continue;
+            };
+            let readable = readable_roots_independent(&node.pool, &pool_geometry);
+            let Some(event) = node.rollback_events.first() else {
+                continue;
+            };
+            let abandoned_readable: Vec<TimelineRoot> = event
+                .abandoned
+                .iter()
+                .copied()
+                .filter(|root| readable.contains(root))
+                .collect();
+            let plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
+
+            for &abandoned_root in &abandoned_readable {
+                let Some(record) = root_record_of(&node.pool, &pool_geometry, abandoned_root)
+                else {
+                    continue;
+                };
+                let Ok(allocation_locations) =
+                    locate_allocation_record_tree(&plain_devices, &record)
+                else {
+                    continue;
+                };
+                let allocation_keys: BTreeSet<(u32, u64)> =
+                    allocation_locations.iter().map(location_key).collect();
+                if is_unit_shared(
+                    &plain_devices,
+                    &node.pool,
+                    &pool_geometry,
+                    abandoned_root,
+                    &allocation_keys,
+                ) {
+                    continue;
+                }
+
+                let allocation_fault_targets =
+                    fault_targets_for(&allocation_locations, FaultSeverity::Both);
+                let persistent = mount_writable_trajectory(
+                    &node.pool,
+                    &parameters,
+                    fixed_geometry,
+                    &allocation_fault_targets,
+                    true,
+                    3,
+                );
+                let transient = mount_writable_trajectory(
+                    &node.pool,
+                    &parameters,
+                    fixed_geometry,
+                    &allocation_fault_targets,
+                    false,
+                    3,
+                );
+                assert_eq!(persistent.len(), 3, "轨迹要报满 3 步（跑前登记 8.1 原文）");
+                assert_eq!(transient.len(), 3);
+                assert_eq!(
+                    persistent[0], transient[0],
+                    "op1 这一步两种模式的故障目标相同，第一步的结局必须相同"
+                );
+                assert!(
+                    persistent[0].is_some_and(|count| count > 0),
+                    "PC1-a 已经证明这一格会触发，第一步的计数必须 > 0"
+                );
+                assert_eq!(
+                    persistent[1],
+                    persistent[0],
+                    "持续故障不撤，第二步该继续触发（这一格是单一故障不共享的落点，不随挂载次数变化）"
+                );
+                assert_eq!(persistent[2], persistent[0], "持续故障第三步同理");
+                assert_eq!(
+                    transient[1],
+                    Some(0),
+                    "瞬时故障撤掉之后，第二步该恢复成不触发"
+                );
+                assert_eq!(transient[2], Some(0), "瞬时故障第三步同理");
+                return;
+            }
+        }
+        panic!("H1 家族里应当至少有一格满足 PC1-a 的挑选条件（跑前登记 5.2 阳性对照）");
+    }
+
     /// session s9（`m2-rootchoice-repair-r1-forks.md` 岔路单第 2 行还差项②）：权重上限机制——
     /// 显式传 `Some(0)` 时只穷举权重 0 那一档就停（不管这一档打不打中），`stopped_by_weight_ceiling`
     /// 如实反映「还有没搜到的档」；`full_space_subset_count` 恒等于完整证据空间的子集数，不随上限变，
diff -ruN -x target tree/crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs	2026-09-24 02:30:54.123358156 +0000
+++ tree/crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs	2026-09-24 17:28:54.670021224 +0000
@@ -4,7 +4,8 @@
 //!
 //! 模式就是虚机里那一次 `first_transaction_on_device` 的模式，宿主照它重跑同样几步（`singlefs_harness::on_device_modes`，两边跑同一份）：
 //! 前三个模式停在第一个事务；`second-transaction` 接着发布 B；`second-instance` 再接着冷重开、可写挂载与发布 C
-//! （里程碑「第二个事务」增补 2 收口表第 30 行：在这之前宿主只重跑第一个事务，后面三段的写逐项比不到）。
+//! （里程碑「第二个事务」增补 2 收口表第 30 行：在这之前宿主只重跑第一个事务，后面三段的写逐项比不到）；
+//! `raise-rollback-floor` 在发布 C 之后同一次挂载里再发布 D、再把 F 抬到上限（第 58 行：抬 F 那一串空发布同样逐项比）。
 //! 三个数取虚机里那次跑的 `name=geometry` 行。判据：程序的事件是设备侧日志的逐项前缀，前缀之后至多一个 FLUSH
 //! （虚机关机时补的那一个；2026-09-14 第一次跑 direct 时每块盘各多出这一个，见 records 十一·六）。
 //! 每块盘另报程序每一段在这块盘上投出几件事（`expected_events_by_window`），与第一处不一致落在哪一段（`divergence_window`；
@@ -13,6 +14,7 @@
 //! 1 = 有一块盘对不上（打印第一处）或读不回；2 = 用法、日志读不了、或宿主重跑那条写路失败。结果行同样以 `E7RESULT` 打头。
 
 use singlefs_core::address::DeviceIdentity;
+use singlefs_core::allocator::PoolAllocator;
 use singlefs_core::block_device::{
     DirectInputOutputBlockDevice, PageCachePolicy, PhysicalBlockSizeInBytes,
     PhysicalBlockSizeSource,
@@ -20,14 +22,16 @@
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
 use singlefs_core::mount::mount_writable;
 use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
+use singlefs_core::transaction::TransactionOutput;
 use singlefs_harness::crash::SparseBlockDevice;
 use singlefs_harness::device_log::{
     compare_allowing_trailing_flushes, declared_zero_fills, expected_device_events,
     fold_declared_zero_fills, parse_device_log, DeviceEvent, DeviceLog,
 };
 use singlefs_harness::on_device_modes::{
-    allocator_rebuilt_from_the_records_of, publish_the_second_version, publish_the_third_version,
-    OnDeviceRunMode, PublishesAfterTheFirstTransaction,
+    allocator_rebuilt_from_the_records_of, publish_the_fourth_version, publish_the_second_version,
+    publish_the_third_version, raise_the_rollback_floor_to_its_ceiling, OnDeviceRunMode,
+    PublishesAfterTheFirstTransaction,
 };
 use singlefs_harness::scenario::{e142_parameters, run_first_transaction, ScenarioPoint};
 use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};
@@ -48,6 +52,8 @@
     SecondTransaction,
     ReopenAndWritableMount,
     ThirdTransaction,
+    FourthTransaction,
+    RaiseRollbackFloor,
 }
 
 impl ProgramWindow {
@@ -60,6 +66,8 @@
             ProgramWindow::SecondTransaction => "second_transaction",
             ProgramWindow::ReopenAndWritableMount => "reopen_and_writable_mount",
             ProgramWindow::ThirdTransaction => "third_transaction",
+            ProgramWindow::FourthTransaction => "fourth_transaction",
+            ProgramWindow::RaiseRollbackFloor => "raise_rollback_floor",
         }
     }
 }
@@ -142,7 +150,8 @@
     match publishes {
         PublishesAfterTheFirstTransaction::Nothing => {}
         PublishesAfterTheFirstTransaction::SecondVersion
-        | PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance => {
+        | PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance
+        | PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstanceThenFourthVersionThenRaiseOfTheRollbackFloor => {
             let mut allocator = allocator_rebuilt_from_the_records_of(&devices, &run.output);
             publish_the_second_version(parameters, &mut devices, &mut allocator, &run.output)
                 .map_err(|failed| format!("发布 B：{:?}", failed.cause))?;
@@ -153,38 +162,27 @@
         PublishesAfterTheFirstTransaction::Nothing
         | PublishesAfterTheFirstTransaction::SecondVersion => {}
         PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance => {
-            let mut reopened: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
-                devices
-                    .into_iter()
-                    .map(|(identity, closed)| {
-                        (
-                            identity,
-                            RecordingBlockDevice::with_shared_stream(
-                                identity,
-                                closed.into_inner_and_operations().0,
-                                stream.clone(),
-                            ),
-                        )
-                    })
-                    .collect();
-            let mut mounted = mount_writable(parameters, &mut reopened)
-                .map_err(|failure| format!("可写挂载：{failure:?}"))?;
-            window_ends.push((
-                ProgramWindow::ReopenAndWritableMount,
-                stream.operation_count(),
-            ));
-            let mounted_version = mounted.current.file_version().cloned().ok_or_else(|| {
-                "可写挂载之后现行那一版没有文件：发布 B 之后重开，上一版带文件".to_string()
-            })?;
-            publish_the_third_version(
+            rerun_the_second_instance_on_the_host(parameters, devices, stream, &mut window_ends)?;
+        }
+        PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstanceThenFourthVersionThenRaiseOfTheRollbackFloor => {
+            let mut second_instance =
+                rerun_the_second_instance_on_the_host(parameters, devices, stream, &mut window_ends)?;
+            let mut current_version = publish_the_fourth_version(
                 parameters,
-                &mut reopened,
-                &mut mounted.allocator,
-                &mounted_version,
-                mounted.output.instance,
+                &mut second_instance.devices,
+                &mut second_instance.allocator,
+                &second_instance.current_version,
             )
-            .map_err(|failed| format!("发布 C：{:?}", failed.cause))?;
-            window_ends.push((ProgramWindow::ThirdTransaction, stream.operation_count()));
+            .map_err(|failed| format!("发布 D：{:?}", failed.cause))?;
+            window_ends.push((ProgramWindow::FourthTransaction, stream.operation_count()));
+            raise_the_rollback_floor_to_its_ceiling(
+                parameters,
+                &mut second_instance.devices,
+                &mut second_instance.allocator,
+                &mut current_version,
+            )
+            .map_err(|failure| format!("抬 F：{failure:?}"))?;
+            window_ends.push((ProgramWindow::RaiseRollbackFloor, stream.operation_count()));
         }
     }
     Ok(HostRerun {
@@ -193,6 +191,65 @@
     })
 }
 
+/// 宿主重跑到发布 C 为止之后的样子：重开之后的那一套盘、挂载交回又被发布 C 推进过的分配器、现行那一版（发布 C）。
+struct SecondInstanceOnTheHost {
+    devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>,
+    allocator: PoolAllocator,
+    current_version: TransactionOutput,
+}
+
+/// 发布 B 之后：同一份镜像交给新的录制器（冷重开）、可写挂载、发布 C，两段的终点记进 `window_ends`。
+///
+/// # Errors
+/// 可写挂载失败、挂载之后现行那一版没有文件、发布 C 失败：交回一句原因。
+fn rerun_the_second_instance_on_the_host(
+    parameters: &MakeFilesystemParameters,
+    devices_after_the_second_version: Vec<(
+        DeviceIdentity,
+        RecordingBlockDevice<SparseBlockDevice>,
+    )>,
+    stream: &SharedStream,
+    window_ends: &mut Vec<(ProgramWindow, usize)>,
+) -> Result<SecondInstanceOnTheHost, String> {
+    let mut reopened: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
+        devices_after_the_second_version
+            .into_iter()
+            .map(|(identity, closed)| {
+                (
+                    identity,
+                    RecordingBlockDevice::with_shared_stream(
+                        identity,
+                        closed.into_inner_and_operations().0,
+                        stream.clone(),
+                    ),
+                )
+            })
+            .collect();
+    let mut mounted = mount_writable(parameters, &mut reopened)
+        .map_err(|failure| format!("可写挂载：{failure:?}"))?;
+    window_ends.push((
+        ProgramWindow::ReopenAndWritableMount,
+        stream.operation_count(),
+    ));
+    let mounted_version = mounted.current.file_version().cloned().ok_or_else(|| {
+        "可写挂载之后现行那一版没有文件：发布 B 之后重开，上一版带文件".to_string()
+    })?;
+    let third_version = publish_the_third_version(
+        parameters,
+        &mut reopened,
+        &mut mounted.allocator,
+        &mounted_version,
+        mounted.output.instance,
+    )
+    .map_err(|failed| format!("发布 C：{:?}", failed.cause))?;
+    window_ends.push((ProgramWindow::ThirdTransaction, stream.operation_count()));
+    Ok(SecondInstanceOnTheHost {
+        devices: reopened,
+        allocator: mounted.allocator,
+        current_version: third_version,
+    })
+}
+
 fn parse_number(text: &str, what: &str) -> u64 {
     text.parse().unwrap_or_else(|_error| {
         eprintln!("{what} 不是数：{text}");
@@ -534,6 +591,9 @@
                 OnDeviceRunMode::SecondInstance => {
                     "mkfs,instance_acquisition,warm_up,first_transaction,second_transaction,reopen_and_writable_mount,third_transaction"
                 }
+                OnDeviceRunMode::RaiseRollbackFloor => {
+                    "mkfs,instance_acquisition,warm_up,first_transaction,second_transaction,reopen_and_writable_mount,third_transaction,fourth_transaction,raise_rollback_floor"
+                }
             };
             let described = rerun.describe(mode);
             assert_eq!(field(&described, "mode"), Some(mode.argument()));
@@ -728,6 +788,131 @@
             );
         }
     }
+
+    /// `raise-rollback-floor`：发布 D 与抬 F 两段真在比对里（增补 2 收口表第 58 行）。每一段里改一步（少一个写、少一个 FLUSH、
+    /// 最后一个写的内容变了）两块盘都判红、红在被改的那一段（`fourth_transaction` / `raise_rollback_floor`）；`second-instance` 的盘
+    /// （没有发布 D、没抬 F）拿这个模式去比，程序的事件在发布 D 那一段开头对不上；反过来这个模式的盘拿 `second-instance` 去比，红在程序之后。
+    #[test]
+    fn raise_rollback_floor_mode_compares_the_raise_window_and_is_red_there_when_one_step_changed()
+    {
+        let raise = rerun_of(OnDeviceRunMode::RaiseRollbackFloor);
+        let second_instance = rerun_of(OnDeviceRunMode::SecondInstance);
+        type StepChange = fn(&mut Vec<DeviceEvent>) -> bool;
+        let changes: [(&str, StepChange); 3] = [
+            ("少一个写", |events| {
+                match events
+                    .iter()
+                    .position(|event| matches!(event, DeviceEvent::Write { .. }))
+                {
+                    Some(position) => {
+                        events.remove(position);
+                        true
+                    }
+                    None => false,
+                }
+            }),
+            ("少一个 FLUSH", |events| {
+                match events
+                    .iter()
+                    .position(|event| matches!(event, DeviceEvent::Flush))
+                {
+                    Some(position) => {
+                        events.remove(position);
+                        true
+                    }
+                    None => false,
+                }
+            }),
+            ("最后一个写的内容变了", |events| {
+                match events
+                    .iter_mut()
+                    .rev()
+                    .find(|event| matches!(event, DeviceEvent::Write { .. }))
+                {
+                    Some(DeviceEvent::Write { content_hash, .. }) => {
+                        *content_hash ^= 1;
+                        true
+                    }
+                    Some(DeviceEvent::Flush | DeviceEvent::Discard { .. } | DeviceEvent::Mark)
+                    | None => false,
+                }
+            }),
+        ];
+        for identity in [DeviceIdentity(0), DeviceIdentity(1)] {
+            let unchanged = compare_one_device(
+                &raise,
+                identity,
+                &device_log_that_received(
+                    &raise,
+                    identity,
+                    flattened(events_by_window(&raise, identity)),
+                ),
+            );
+            assert!(unchanged.matches, "{}", unchanged.line);
+            for changed_window in [
+                ProgramWindow::FourthTransaction,
+                ProgramWindow::RaiseRollbackFloor,
+            ] {
+                for (change_name, change) in &changes {
+                    let mut windows = events_by_window(&raise, identity);
+                    let (_, events) = windows
+                        .iter_mut()
+                        .find(|(window, _)| *window == changed_window)
+                        .expect("raise-rollback-floor 跑了发布 D 与抬 F 两段");
+                    assert!(
+                        change(events),
+                        "盘 {} 的 {} 里找不到可改的那一步（{change_name}）",
+                        identity.0,
+                        changed_window.name()
+                    );
+                    let log = device_log_that_received(&raise, identity, flattened(windows));
+                    let comparison = compare_one_device(&raise, identity, &log);
+                    assert!(
+                        !comparison.matches,
+                        "盘 {} 的 {} {change_name}：必须判红\n{}",
+                        identity.0,
+                        changed_window.name(),
+                        comparison.line
+                    );
+                    assert_eq!(
+                        field(&comparison.line, "divergence_window"),
+                        Some(changed_window.name()),
+                        "{change_name}：{}",
+                        comparison.line
+                    );
+                }
+            }
+
+            let without_the_raise = device_log_that_received(
+                &second_instance,
+                identity,
+                flattened(events_by_window(&second_instance, identity)),
+            );
+            let missing_raise = compare_one_device(&raise, identity, &without_the_raise);
+            assert!(!missing_raise.matches);
+            assert_eq!(
+                field(&missing_raise.line, "divergence_window"),
+                Some(ProgramWindow::FourthTransaction.name()),
+                "{}",
+                missing_raise.line
+            );
+
+            let with_the_raise = device_log_that_received(
+                &raise,
+                identity,
+                flattened(events_by_window(&raise, identity)),
+            );
+            let raise_not_in_the_program =
+                compare_one_device(&second_instance, identity, &with_the_raise);
+            assert!(!raise_not_in_the_program.matches);
+            assert_eq!(
+                field(&raise_not_in_the_program.line, "divergence_window"),
+                Some(AFTER_THE_PROGRAM),
+                "{}",
+                raise_not_in_the_program.line
+            );
+        }
+    }
 
     #[test]
     fn every_position_is_attributed_to_the_window_whose_events_contain_it() {
diff -ruN -x target tree/crates/singlefs-harness/src/bin/first_transaction_on_device.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-harness/src/bin/first_transaction_on_device.rs	2026-09-24 16:15:19.532852641 +0000
+++ tree/crates/singlefs-harness/src/bin/first_transaction_on_device.rs	2026-09-24 17:28:54.669972875 +0000
@@ -1,6 +1,6 @@
 //! 虚机档（QEMU/KVM）：在两块真 virtio 盘上跑第一个事务的整条写路，冷重开再恢复读回文件。
 //!
-//!   first_transaction_on_device /dev/vda /dev/vdb <direct | page-cache | skip-first-transaction-barrier | second-transaction | second-instance>
+//!   first_transaction_on_device /dev/vda /dev/vdb <direct | page-cache | skip-first-transaction-barrier | second-transaction | second-instance | raise-rollback-floor>
 //!
 //! 由 `research/scripts/vm-bench.sh` 送进虚机（`VM_DISKS=2`，设备路径排在参数前面）。结果行以 `E7RESULT` 打头、
 //! 末行报条数（vm-bench.sh 的完整性闸）。这个二进制只判它自己判得了的（恢复读回文件、段序列）；
@@ -13,6 +13,10 @@
 //! `second-instance`：发布 B 之后丢掉写的那一套句柄、同一对盘冷重开，走可写挂载（恢复、取号、写行、暖机，里程碑「第二个事务」步 3），
 //! 再发布一次（发布 C）；挂载一行、发布 C 一行，写行与挂载里的每次暖机、发布 C 各一行 `name=publish_writes`，挂载与发布 C 各一段窗口行；
 //! 冷重开读回的是第三版（实例 2 的根）。前四个模式打的行一行不变。
+//! `raise-rollback-floor`：`second-instance` 那条路走完，同一次挂载里再覆盖写一次（发布 D，第 4 新的非空有效根落到 A 上），接着把 F 抬到上限
+//! （里程碑「第二个事务」增补 2 收口表第 58 行：抬 F 那一串发布的账在二进制这一侧与设备一层判相等）；发布 D 与发布 C 同样打一行、一行
+//! `name=publish_writes`、一段窗口行，抬 F 一行、那一串空发布各一行 `name=publish_writes`、这一段一行窗口行；失败时照发布失败那样打失败账。
+//! 冷重开读回的是第四版（最后一次抬 F 的空发布的根）。前五个模式打的行一行不变。
 //! 每次发布（两次暖机、第一个事务、`second-transaction` 模式下的发布 B）各打一行 `name=publish_writes`：写入口按结构种类记的写调用数与写字节
 //! （里程碑「第二个事务」增补 1 第 1 件）；每段窗口（两次暖机合一段、第一个事务、发布 B）再打一行 `name=publish_writes_against_device`，
 //! 按种类的合计与设备一层数的（`FaultInjectingDevice`，两盘相加）逐项比，对不上 `matches=false`、退出码 1。
@@ -24,12 +28,13 @@
 use std::path::Path;
 
 use singlefs_core::address::DeviceIdentity;
+use singlefs_core::allocator::PoolAllocator;
 use singlefs_core::block_device::{
     probe_queue_number_under, probe_queue_text_under, BlockDevice, DirectInputOutputBlockDevice,
     PageCachePolicy, PhysicalBlockSizeSource, SYSFS_CLASS_BLOCK,
 };
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
-use singlefs_core::mount::{mount_writable, MountError, Mounted};
+use singlefs_core::mount::{mount_writable, MountError, Mounted, PublishSequenceFailed};
 use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
 use singlefs_core::transaction::{PoolVersion, TransactionOutput};
 use singlefs_core::write_accounting::{
@@ -40,7 +45,8 @@
     FaultOccurrence, FaultPlacement, FaultSchedule, InjectedFault, SharedFaultPlan,
 };
 use singlefs_harness::on_device_modes::{
-    allocator_rebuilt_from_the_records_of, publish_the_second_version, publish_the_third_version,
+    allocator_rebuilt_from_the_records_of, publish_the_fourth_version, publish_the_second_version,
+    publish_the_third_version, raise_the_rollback_floor_to_its_ceiling, FailedPublish,
     OnDeviceRunMode, PublishesAfterTheFirstTransaction,
 };
 use singlefs_harness::scenario::{
@@ -85,6 +91,18 @@
             "third_transaction",
             "cold_reopen_and_recover",
         ],
+        OnDeviceRunMode::RaiseRollbackFloor => &[
+            "mkfs",
+            "instance_acquisition",
+            "warm_up",
+            "first_transaction",
+            "second_transaction",
+            "reopen_and_writable_mount",
+            "third_transaction",
+            "fourth_transaction",
+            "raise_rollback_floor",
+            "cold_reopen_and_recover",
+        ],
     }
 }
 
@@ -414,6 +432,22 @@
     }
 }
 
+/// 发布 B 之后那几段（可写挂载、发布 C、抬 F）攒下的结果行：分段时间那几行是在那几个函数里取的表，挑出来并进分段行
+/// （跑完一起打），别的结果行照旧按次序打。
+fn emit_the_lines_after_the_second_version(
+    emitter: &mut Emitter,
+    segment_timing_lines: &mut Vec<String>,
+    lines: &[String],
+) {
+    for line in lines {
+        if line.starts_with("name=segment_timing ") {
+            segment_timing_lines.push(line.clone());
+        } else {
+            emitter.emit(line);
+        }
+    }
+}
+
 /// 块层计数（`/sys/class/block/<名>/stat`，内核 `Documentation/block/stat.rst` 的字段序）：写请求数（下标 4）、写扇区数（下标 6）、
 /// FLUSH 请求数（下标 15，5.5 起才有，老内核没有就是 None）。
 fn block_layer_counters(device_path: &str) -> Option<(u64, u64, Option<u64>)> {
@@ -574,10 +608,14 @@
     })
 }
 
-/// `second-instance` 模式在发布 B 之后写出的东西：重开之后的那一套句柄（冷恢复之前要丢掉）、按次序要打的结果行、
+/// `second-instance` 模式在发布 B 之后写出的东西：重开之后的那一套句柄（冷恢复之前要丢掉）、挂载交回又被发布 C 推进过的分配器、
+/// 现行那一版（发布 C；`raise-rollback-floor` 抬完 F 之后是最后一次抬 F 的空发布）、重开之后这一段的注入计划、按次序要打的结果行、
 /// 每段窗口按种类的合计与设备一层是否都相等。
 struct SecondInstanceRun<Inner: BlockDevice> {
     devices: Vec<(DeviceIdentity, CountedDevice<Inner>)>,
+    allocator: PoolAllocator,
+    current_version: TransactionOutput,
+    plan: SharedFaultPlan,
     lines: Vec<String>,
     every_window_matches_device: bool,
 }
@@ -821,70 +859,334 @@
         return Err(FailedRun { lines, cause });
     };
     let instance = mounted.output.instance;
-    let operations_before_third = stream.operations().len();
-    let third_started = Instant::now();
-    let published = publish_the_third_version(
-        parameters,
-        devices.as_mut_slice(),
-        &mut mounted.allocator,
-        &current,
-        instance,
-    );
-    let third_nanoseconds = third_started.elapsed().as_nanos();
-    let counts_after_third = device_call_counts(&devices, &plan);
-    let third = match published {
+    let third = match publish_an_overwrite_in_the_mounted_instance_and_describe(
+        "third_transaction",
+        &mut devices,
+        &plan,
+        &counts_after_mount,
+        stream,
+        geometry,
+        |devices| {
+            publish_the_third_version(
+                parameters,
+                devices,
+                &mut mounted.allocator,
+                &current,
+                instance,
+            )
+        },
+    ) {
         Ok(third) => third,
         Err(failed) => {
+            lines.extend(failed.lines);
+            return Err(FailedRun {
+                lines,
+                cause: format!("发布 C：{}", failed.cause),
+            });
+        }
+    };
+    lines.extend(third.lines);
+    lines.push(clock.mark("third_transaction"));
+
+    Ok(SecondInstanceRun {
+        devices,
+        allocator: mounted.allocator,
+        current_version: third.version,
+        plan,
+        lines,
+        every_window_matches_device: mount_window_matches_device && third.window_matches_device,
+    })
+}
+
+/// 同一次挂载里一次覆盖写成功之后：这一版、这一段的结果行（发布一行、`name=publish_writes` 一行、窗口行一行）、
+/// 这一段按种类的合计与设备一层是否相等。
+struct OverwriteInTheMountedInstance {
+    version: TransactionOutput,
+    lines: Vec<String>,
+    window_matches_device: bool,
+}
+
+/// 同一次挂载里的一次覆盖写（发布 C；`raise-rollback-floor` 还有发布 D）：`publish` 发这一次，挂钟只计它；
+/// 窗口从 `counts_before` 那一刻算起、到它返回为止，按种类的合计与设备一层逐项比。结果行以 `window` 为名（`name=<window>` 一行
+/// 与 `publish=<window>`、`window=<window>` 各一行）。
+///
+/// # Errors
+/// 发布失败：交回失败账那几行（增补 2 收口表第 58 行）与失败原因的 Debug 文本，调用方在前面接上它前面几段的结果行。
+fn publish_an_overwrite_in_the_mounted_instance_and_describe<Inner, Publish>(
+    window: &str,
+    devices: &mut [(DeviceIdentity, CountedDevice<Inner>)],
+    plan: &SharedFaultPlan,
+    counts_before: &[DeviceCallCounts],
+    stream: &SharedStream,
+    geometry: &FixedGeometry,
+    publish: Publish,
+) -> Result<OverwriteInTheMountedInstance, FailedRun>
+where
+    Inner: BlockDevice,
+    Publish: FnOnce(
+        &mut [(DeviceIdentity, CountedDevice<Inner>)],
+    ) -> Result<TransactionOutput, FailedPublish>,
+{
+    let operations_before = stream.operations().len();
+    let started = Instant::now();
+    let published = publish(devices);
+    let nanoseconds = started.elapsed().as_nanos();
+    let counts_after = device_call_counts(devices, plan);
+    let version = match published {
+        Ok(version) => version,
+        Err(failed) => {
             let cause = format!("{:?}", failed.cause);
             let (failure_lines, _) = describe_failed_window(
-                "third_transaction",
+                window,
                 &cause,
                 &[],
                 &failed.writes_of_failed_publishes,
-                pool_writes_between(&counts_after_mount, &counts_after_third),
+                pool_writes_between(counts_before, &counts_after),
             );
+            return Err(FailedRun {
+                lines: failure_lines,
+                cause,
+            });
+        }
+    };
+    let operations = stream.operations();
+    let segments = split_into_segments(&operations[operations_before..], geometry);
+    let per_device: Vec<String> = counts_after
+        .iter()
+        .enumerate()
+        .map(|(index, later)| later.since(counts_before[index]).describe(index))
+        .collect();
+    let mut lines = vec![
+        format!(
+            "name={window} root_txg={} transaction={} released={} nanoseconds={nanoseconds} operations={} segments={} closed_form={} {}",
+            version.root.checkpoint_txg.0,
+            version.record.transaction,
+            version.released.len(),
+            operations.len() - operations_before,
+            segment_sizes_text(&segments),
+            closed_form_state_count(&segments),
+            per_device.join(" ")
+        ),
+        describe_publish_writes(window, version.root.checkpoint_txg.0, &version.writes),
+    ];
+    let (window_line, window_matches_device) = publish_writes_against_device(
+        window,
+        &[&version.writes],
+        pool_writes_between(counts_before, &counts_after),
+    );
+    lines.push(window_line);
+    Ok(OverwriteInTheMountedInstance {
+        version,
+        lines,
+        window_matches_device,
+    })
+}
+
+/// `raise-rollback-floor` 的发布 D：发布 C 之后、同一次挂载里接在发布 C 那一版上再覆盖写一次（`publish_the_fourth_version`，
+/// 宿主检查重跑同一份），这一段窗口从发布 C 返回那一刻算起。结果行与发布 C 同形，名字是 `fourth_transaction`。
+///
+/// # Errors
+/// 发布 D 失败：接在前面几段的结果行后面交回失败账那几行（增补 2 收口表第 58 行）与一句原因。
+fn publish_the_fourth_version_and_describe<Inner: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    second_instance_run: SecondInstanceRun<Inner>,
+    stream: &SharedStream,
+    geometry: &FixedGeometry,
+    clock: &mut SegmentClock,
+) -> Result<SecondInstanceRun<Inner>, FailedRun> {
+    let SecondInstanceRun {
+        mut devices,
+        mut allocator,
+        current_version: third_version,
+        plan,
+        mut lines,
+        every_window_matches_device,
+    } = second_instance_run;
+    let counts_after_third = device_call_counts(&devices, &plan);
+    let fourth = match publish_an_overwrite_in_the_mounted_instance_and_describe(
+        "fourth_transaction",
+        &mut devices,
+        &plan,
+        &counts_after_third,
+        stream,
+        geometry,
+        |devices| publish_the_fourth_version(parameters, devices, &mut allocator, &third_version),
+    ) {
+        Ok(fourth) => fourth,
+        Err(failed) => {
+            lines.extend(failed.lines);
+            return Err(FailedRun {
+                lines,
+                cause: format!("发布 D：{}", failed.cause),
+            });
+        }
+    };
+    lines.extend(fourth.lines);
+    lines.push(clock.mark("fourth_transaction"));
+
+    Ok(SecondInstanceRun {
+        devices,
+        allocator,
+        current_version: fourth.version,
+        plan,
+        lines,
+        every_window_matches_device: every_window_matches_device && fourth.window_matches_device,
+    })
+}
+
+/// `raise-rollback-floor` 在发布 C 之后的两段：同一次挂载里发布 D（第 4 新的非空有效根落到 A 上），再把 F 抬到上限。`main` 与用例走这同一个函数。
+///
+/// # Errors
+/// 发布 D 或抬 F 失败：交回到那一刻为止的结果行（失败账那几行在内）与一句原因。
+fn publish_the_fourth_version_and_raise_the_rollback_floor<Inner: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    third_version_run: SecondInstanceRun<Inner>,
+    stream: &SharedStream,
+    geometry: &FixedGeometry,
+    clock: &mut SegmentClock,
+) -> Result<SecondInstanceRun<Inner>, FailedRun> {
+    let fourth_version_run = publish_the_fourth_version_and_describe(
+        parameters,
+        third_version_run,
+        stream,
+        geometry,
+        clock,
+    )?;
+    raise_the_rollback_floor_and_describe(parameters, fourth_version_run, stream, geometry, clock)
+}
+
+/// `raise-rollback-floor`：发布 D 之后、同一次挂载里把 F 抬到上限（`raise_the_rollback_floor_to_its_ceiling`，宿主检查重跑同一份）。
+/// 这一段窗口从抬 F 之前那一刻算起、到它返回为止，按种类的合计是那一串空发布之和，与设备一层逐项比（增补 2 收口表第 58 行
+/// 「二进制那一侧判相等」）。抬 F 只有测试入口（`raise_rollback_floor` 的文档注释），这一档就是那个入口在真设备上的一次。
+/// 结果行里 `requested_floor` 是抬完之后最后一次空发布的根带的 F，`ceiling` 是 core 抬 F 时算的上限。
+///
+/// # Errors
+/// 抬 F 失败：那一串空发布里有一次发不出去时，失败账那几行照可写挂载失败的判法打（已落盘的那几次各一行、失败那一次一行、
+/// 两样相加与设备一层比一行）；别的错在任何写之前，只打停下的原因。都接在前面几段的结果行后面交回。
+fn raise_the_rollback_floor_and_describe<Inner: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    second_instance_run: SecondInstanceRun<Inner>,
+    stream: &SharedStream,
+    geometry: &FixedGeometry,
+    clock: &mut SegmentClock,
+) -> Result<SecondInstanceRun<Inner>, FailedRun> {
+    let SecondInstanceRun {
+        mut devices,
+        mut allocator,
+        mut current_version,
+        plan,
+        mut lines,
+        every_window_matches_device,
+    } = second_instance_run;
+    let floor_before_raise = current_version.root.rollback_floor;
+    let counts_before_raise = device_call_counts(&devices, &plan);
+    let operations_before_raise = stream.operations().len();
+    let raise_started = Instant::now();
+    let raised = raise_the_rollback_floor_to_its_ceiling(
+        parameters,
+        &mut devices,
+        &mut allocator,
+        &mut current_version,
+    );
+    let raise_nanoseconds = raise_started.elapsed().as_nanos();
+    let counts_after_raise = device_call_counts(&devices, &plan);
+    let raised = match raised {
+        Ok(raised) => raised,
+        Err(failure) => {
+            let cause = format!("{failure:?}");
+            let failure_lines = match &failure {
+                // 抬 F 的写入口随错丢掉，这一串的账随错交回（已经落盘的那几次空发布、失败那一次已记的写）。
+                MountError::RaiseFloorSequencePublishFailed(PublishSequenceFailed {
+                    cause: _,
+                    writes_of_persisted_publishes,
+                    writes_of_failed_publishes,
+                }) => {
+                    describe_failed_window(
+                        "raise_rollback_floor",
+                        &cause,
+                        writes_of_persisted_publishes,
+                        writes_of_failed_publishes,
+                        pool_writes_between(&counts_before_raise, &counts_after_raise),
+                    )
+                    .0
+                }
+                // 在第一次空发布之前就停下的：选系统配置、读实例表、算上限、超上限，这一段没有发布的账可比。
+                // 可写挂载与回退那几条（取号、写行、暖机、回退目标）抬 F 走不到，照样列全，不写通配臂。
+                MountError::Recovery(_)
+                | MountError::FileVersionWithoutAnyJournalRecord
+                | MountError::InstanceTableMalformed
+                | MountError::Acquisition(_)
+                | MountError::Publish(_)
+                | MountError::RollbackTargetNotACandidate { .. }
+                | MountError::RollbackFloorAboveCeiling { .. }
+                | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
+                | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
+                | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
+                | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
+                | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
+                | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
+                | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. } => {
+                    vec![describe_run_failure("raise_rollback_floor", &cause)]
+                }
+            };
             lines.extend(failure_lines);
             return Err(FailedRun {
                 lines,
-                cause: format!("发布 C：{cause}"),
+                cause: format!("抬 F：{cause}"),
             });
         }
     };
-    let third_operations = stream.operations();
-    let third_segments =
-        split_into_segments(&third_operations[operations_before_third..], geometry);
-    let per_device_third: Vec<String> = counts_after_third
+    let raise_operations = stream.operations();
+    let raise_segments =
+        split_into_segments(&raise_operations[operations_before_raise..], geometry);
+    let per_device_raise: Vec<String> = counts_after_raise
         .iter()
         .enumerate()
-        .map(|(index, later)| later.since(counts_after_mount[index]).describe(index))
+        .map(|(index, later)| later.since(counts_before_raise[index]).describe(index))
+        .collect();
+    let raise_txgs: Vec<String> = raised
+        .publishes
+        .iter()
+        .map(|publish| publish.root.checkpoint_txg.0.to_string())
         .collect();
     lines.push(format!(
-        "name=third_transaction root_txg={} transaction={} released={} nanoseconds={third_nanoseconds} operations={} segments={} closed_form={} {}",
-        third.root.checkpoint_txg.0,
-        third.record.transaction,
-        third.released.len(),
-        third_operations.len() - operations_before_third,
-        segment_sizes_text(&third_segments),
-        closed_form_state_count(&third_segments),
-        per_device_third.join(" ")
-    ));
-    lines.push(describe_publish_writes(
-        "third_transaction",
-        third.root.checkpoint_txg.0,
-        &third.writes,
+        "name=raise_rollback_floor floor_before={} requested_floor={} ceiling={} publishes={} root_txgs={} reclaimed={} abandoned_roots_unreadable={} nanoseconds={raise_nanoseconds} operations={} segments={} closed_form={} {}",
+        floor_before_raise.0,
+        current_version.root.rollback_floor.0,
+        raised.ceiling.0,
+        raised.publishes.len(),
+        raise_txgs.join(","),
+        raised.reclaimed.len(),
+        raised.abandoned_roots_unreadable,
+        raise_operations.len() - operations_before_raise,
+        segment_sizes_text(&raise_segments),
+        closed_form_state_count(&raise_segments),
+        per_device_raise.join(" ")
     ));
-    let (third_window_line, third_window_matches) = publish_writes_against_device(
-        "third_transaction",
-        &[&third.writes],
-        pool_writes_between(&counts_after_mount, &counts_after_third),
+    let mut raise_publishes: Vec<&WritesByStructureKind> = Vec::new();
+    for raise_publish in &raised.publishes {
+        lines.push(describe_publish_writes(
+            "raise_rollback_floor",
+            raise_publish.root.checkpoint_txg.0,
+            &raise_publish.writes,
+        ));
+        raise_publishes.push(&raise_publish.writes);
+    }
+    let (raise_window_line, raise_window_matches) = publish_writes_against_device(
+        "raise_rollback_floor",
+        &raise_publishes,
+        pool_writes_between(&counts_before_raise, &counts_after_raise),
     );
-    lines.push(third_window_line);
-    lines.push(clock.mark("third_transaction"));
+    lines.push(raise_window_line);
+    lines.push(clock.mark("raise_rollback_floor"));
 
     Ok(SecondInstanceRun {
         devices,
+        allocator,
+        current_version,
+        plan,
         lines,
-        every_window_matches_device: mount_window_matches_device && third_window_matches,
+        every_window_matches_device: every_window_matches_device && raise_window_matches,
     })
 }
 
@@ -910,7 +1212,8 @@
         OnDeviceRunMode::Direct
         | OnDeviceRunMode::SkipFirstTransactionBarrier
         | OnDeviceRunMode::SecondTransaction
-        | OnDeviceRunMode::SecondInstance => PageCachePolicy::BypassWithDirectInputOutput,
+        | OnDeviceRunMode::SecondInstance
+        | OnDeviceRunMode::RaiseRollbackFloor => PageCachePolicy::BypassWithDirectInputOutput,
         OnDeviceRunMode::PageCache => PageCachePolicy::GoThroughPageCache,
     };
     let mut emitter = Emitter { emitted: 0 };
@@ -1121,7 +1424,8 @@
     match mode.publishes_after_the_first_transaction() {
         PublishesAfterTheFirstTransaction::Nothing => {}
         PublishesAfterTheFirstTransaction::SecondVersion
-        | PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance => {
+        | PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance
+        | PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstanceThenFourthVersionThenRaiseOfTheRollbackFloor => {
             let second = publish_the_second_version_and_describe(
                 &parameters,
                 &mut devices,
@@ -1139,7 +1443,20 @@
         }
     }
 
-    // `second-instance`：发布 B 之后同一对盘冷重开、可写挂载，再发布 C。
+    // `second-instance`：发布 B 之后同一对盘冷重开、可写挂载，再发布 C；`raise-rollback-floor` 接着在同一次挂载里发布 D、抬 F。
+    let reopen_by_path = |closed_devices: Vec<(DeviceIdentity, DirectInputOutputBlockDevice)>| {
+        drop(closed_devices);
+        device_paths
+            .iter()
+            .enumerate()
+            .map(|(index, path)| {
+                (
+                    DeviceIdentity(u32::try_from(index).expect("设备号")),
+                    open(path, policy),
+                )
+            })
+            .collect::<Vec<_>>()
+    };
     let devices = match mode.publishes_after_the_first_transaction() {
         PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance => {
             let switched = switch_instance_and_publish_third_version(
@@ -1148,33 +1465,44 @@
                 &stream,
                 &geometry,
                 &mut clock,
-                |closed_devices| {
-                    drop(closed_devices);
-                    device_paths
-                        .iter()
-                        .enumerate()
-                        .map(|(index, path)| {
-                            (
-                                DeviceIdentity(u32::try_from(index).expect("设备号")),
-                                open(path, policy),
-                            )
-                        })
-                        .collect()
-                },
+                reopen_by_path,
             )
             .unwrap_or_else(|failed| exit_after_a_failed_run(&mut emitter, &failed));
-            // 这一档的两条分段行是在那个函数里取的表，攒在它的 `lines` 里；挑出来并进分段行，
-            // 别的结果行照旧按次序打。
-            for line in &switched.lines {
-                if line.starts_with("name=segment_timing ") {
-                    segment_timing_lines.push(line.clone());
-                } else {
-                    emitter.emit(line);
-                }
-            }
+            emit_the_lines_after_the_second_version(
+                &mut emitter,
+                &mut segment_timing_lines,
+                &switched.lines,
+            );
             every_window_matches_device &= switched.every_window_matches_device;
             switched.devices
         }
+        PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstanceThenFourthVersionThenRaiseOfTheRollbackFloor => {
+            let raised = switch_instance_and_publish_third_version(
+                &parameters,
+                devices,
+                &stream,
+                &geometry,
+                &mut clock,
+                reopen_by_path,
+            )
+            .and_then(|switched| {
+                publish_the_fourth_version_and_raise_the_rollback_floor(
+                    &parameters,
+                    switched,
+                    &stream,
+                    &geometry,
+                    &mut clock,
+                )
+            })
+            .unwrap_or_else(|failed| exit_after_a_failed_run(&mut emitter, &failed));
+            emit_the_lines_after_the_second_version(
+                &mut emitter,
+                &mut segment_timing_lines,
+                &raised.lines,
+            );
+            every_window_matches_device &= raised.every_window_matches_device;
+            raised.devices
+        }
         PublishesAfterTheFirstTransaction::Nothing
         | PublishesAfterTheFirstTransaction::SecondVersion => devices,
     };
@@ -1255,8 +1583,8 @@
     use singlefs_harness::crash::SparseBlockDevice;
     use singlefs_harness::fault_injection::{FaultInjectingBlockDevice, SharedFaultPlan};
     use singlefs_harness::on_device_modes::{
-        allocator_rebuilt_from_the_records_of, publish_the_second_version, third_file_content,
-        OnDeviceRunMode,
+        allocator_rebuilt_from_the_records_of, fourth_file_content, publish_the_second_version,
+        third_file_content, OnDeviceRunMode,
     };
     use singlefs_harness::scenario::{e142_parameters, run_first_transaction};
     use singlefs_harness::segments::FixedGeometry;
@@ -1611,7 +1939,8 @@
     }
 
     /// 每一档登记的段都互不相同、而且都在这一档真会走到的那几段里：
-    /// `second-instance` 比 `second-transaction` 多两段（重开可写挂载、发布 C），后者比 `direct` 多一段（发布 B）。
+    /// `second-instance` 比 `second-transaction` 多两段（重开可写挂载、发布 C），后者比 `direct` 多一段（发布 B）；
+    /// `raise-rollback-floor` 比 `second-instance` 多两段（发布 D、抬 F）。
     #[test]
     fn every_mode_registers_its_own_segments_with_no_repeats() {
         for mode in OnDeviceRunMode::ALL {
@@ -1636,14 +1965,32 @@
             registered_segments_of(OnDeviceRunMode::SecondInstance).len(),
             8
         );
+        assert_eq!(
+            registered_segments_of(OnDeviceRunMode::RaiseRollbackFloor),
+            [
+                "mkfs",
+                "instance_acquisition",
+                "warm_up",
+                "first_transaction",
+                "second_transaction",
+                "reopen_and_writable_mount",
+                "third_transaction",
+                "fourth_transaction",
+                "raise_rollback_floor",
+                "cold_reopen_and_recover"
+            ],
+            "raise-rollback-floor 比 second-instance 多发布 D 与抬 F 两段，排在发布 C 之后、冷重开之前"
+        );
     }
 
     use super::{
-        describe_failed_window, describe_first_transaction_path_failure, device_call_counts,
+        describe_first_transaction_path_failure, device_call_counts,
+        publish_the_fourth_version_and_describe,
+        publish_the_fourth_version_and_raise_the_rollback_floor,
         publish_the_second_version_and_describe, publish_the_third_version_and_describe,
-        reopen_and_mount_writable, FailedRun, WritableMountRun,
+        raise_the_rollback_floor_and_describe, reopen_and_mount_writable, FailedRun,
+        SecondInstanceRun,
     };
-    use singlefs_core::mount::{raise_rollback_floor, MountError, Mounted, ShadowLedger};
     use singlefs_harness::device_log::{expected_device_events, DeviceEvent};
     use singlefs_harness::fault_injection::{FaultSchedule, InjectedFault};
     use singlefs_harness::scenario::{FirstTransactionPathStep, ScenarioPoint};
@@ -2023,87 +2370,350 @@
         }
     }
 
-    /// 抬 F 失败（增补 2 收口表第 58 行「`raise_rollback_floor` 同形」）：抬 F 的写入口是它自己开的、随错丢掉，
-    /// 已经落盘的那几次空发布与失败那一次已记的写随 `MountError::RaiseFloorSequencePublishFailed` 交回，照可写挂载那一段的判法
-    /// （`describe_failed_window`）与设备一层逐项相等。二进制的五个模式里没有抬 F，这一段只在这里走：`second-instance` 那条路走到
-    /// 可写挂载做完（实例 2，现行 txg 7），装上注入，抬到现行的 F（0）——推空发布直到两块盘上都有一条带这个 F 的根（txg 8、9）。
-    /// 注入摆在之后整池第 16 次写：第一次空发布 13 次写已经落盘（四个固定点单元与 journal 记录每盘一份、根槽一次、系统配置每盘一次），
-    /// 第二次的第 3 个写报错 ⇒ 失败账 2 次写，两样相加 15 次与设备一层逐项相等。
+    /// `second-instance` 那条路在宿主上走到发布 C（实例 2，txg 8）：稀疏内存盘代替 virtio 盘，「冷重开」把镜像交给新句柄。
+    /// 交回这一段的注入计划还没装注入。
+    fn second_instance_run_up_to_the_third_version(
+        parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
+        geometry: &FixedGeometry,
+        stream: &SharedStream,
+        clock: &mut SegmentClock,
+    ) -> SecondInstanceRun<SparseBlockDevice> {
+        let plan = SharedFaultPlan::unarmed(*geometry);
+        let mut devices = counted_sparse_devices(stream, &plan);
+        let run = run_first_transaction(parameters, &mut devices, stream, |_point, _devices| {})
+            .expect("第一个事务");
+        let mut allocator = allocator_rebuilt_from_the_records_of(&devices, &run.output);
+        publish_the_second_version(parameters, &mut devices, &mut allocator, &run.output)
+            .expect("发布 B");
+        let switched = match switch_instance_and_publish_third_version(
+            parameters,
+            devices,
+            stream,
+            geometry,
+            clock,
+            reopen_sparse_devices,
+        ) {
+            Ok(switched) => switched,
+            Err(failed) => panic!("第二个实例：{failed:?}"),
+        };
+        assert_eq!(
+            switched.current_version.root.checkpoint_txg,
+            CheckpointTxg(8),
+            "发布 C"
+        );
+        switched
+    }
+
+    /// `raise-rollback-floor` 模式在宿主上照同一条路跑一遍（增补 2 收口表第 58 行「二进制那一侧判相等」，成功那一路）：
+    /// 发布 C（txg 8）之后同一次挂载里发布 D（txg 9，第四版），再把 F 抬到上限。发布 D 之后按新到旧数非空有效根是 D、C、B、A（txg 9、8、4、3），
+    /// 上限 = min(每块盘上最新的有效根, 第 4 新的非空有效根) = min(盘 1 的 txg 7, txg 3) = 3（D16（发布语义） 已定项 1「抬 F 的上限」；
+    /// 根环三个区域的设备归属是 0 / 1 / 0，按 txg 轮流落，盘 0 上最新的是 txg 9、盘 1 上是 txg 7）。推两次带 F = 3 的空发布 txg 10、11，
+    /// 两块盘各落一条（生效）。F 的两个数排在最前：没有发布 D 时第 4 新的非空有效根够不到 A，上限是 0，红在这里。
+    /// 发布 D 一行、抬 F 一行、三次发布各一行 `name=publish_writes`、两段窗口按种类的合计与设备一层逐项相等；录制流投到每块盘上的写与 FLUSH
+    /// 与设备一层数到的相等（宿主检查拿这个投法当程序的信念，发布 D 与抬 F 两段也一样）；冷恢复择 (2, 11) 读回第四版。
     #[test]
-    fn failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count(
+    fn raise_rollback_floor_mode_publishes_the_fourth_version_raises_the_floor_to_its_ceiling_above_zero_and_every_window_matches_the_device_layer_count(
     ) {
         let parameters = e142_parameters(512, 512);
         let geometry = geometry_of(&parameters);
         let stream = SharedStream::new();
-        let plan = SharedFaultPlan::unarmed(geometry);
-        let mut devices = counted_sparse_devices(&stream, &plan);
-        let run = run_first_transaction(&parameters, &mut devices, &stream, |_point, _devices| {})
-            .expect("第一个事务");
-        let mut allocator = allocator_rebuilt_from_the_records_of(&devices, &run.output);
-        publish_the_second_version(&parameters, &mut devices, &mut allocator, &run.output)
-            .expect("发布 B");
         let mut clock = SegmentClock::start();
-        let WritableMountRun {
-            devices: mut mounted_devices,
-            mounted:
-                Mounted {
-                    allocator: mut mounted_allocator,
-                    current,
-                    ..
-                },
-            plan: mounted_plan,
-            ..
-        } = match reopen_and_mount_writable(
+        let switched = second_instance_run_up_to_the_third_version(
             &parameters,
-            devices,
+            &geometry,
+            &stream,
+            &mut clock,
+        );
+        let lines_before_the_fourth_version = switched.lines.len();
+        let operations_before_the_fourth_version = stream.operation_count();
+        let counts_before_the_fourth_version =
+            device_call_counts(&switched.devices, &switched.plan);
+        let raised = match publish_the_fourth_version_and_raise_the_rollback_floor(
+            &parameters,
+            switched,
             &stream,
             &geometry,
-            SharedFaultPlan::unarmed(geometry),
             &mut clock,
-            reopen_sparse_devices,
         ) {
-            Ok(mounted) => mounted,
-            Err(failed) => panic!("可写挂载：{failed:?}"),
+            Ok(raised) => raised,
+            Err(failed) => panic!("发布 D、抬 F：{failed:?}"),
         };
-        let mut current = current
-            .into_file_version()
-            .expect("发布 B 之后重开，现行那一版带文件");
-        assert_eq!(current.root.checkpoint_txg, CheckpointTxg(7));
-        let counts_before_raise = device_call_counts(&mounted_devices, &mounted_plan);
-        mounted_plan.arm(FaultSchedule::the_nth_call_across_the_pool(
-            InjectedFault::WriteFails,
-            16,
-        ));
-        let raised = raise_rollback_floor(
+        let new_lines = &raised.lines[lines_before_the_fourth_version..];
+
+        let raise_line = lines_named(new_lines, "raise_rollback_floor");
+        assert_eq!(raise_line.len(), 1, "{new_lines:#?}");
+        let raise_line = raise_line[0];
+        assert_eq!(
+            field(raise_line, "ceiling"),
+            Some("3"),
+            "第 4 新的非空有效根是 A（txg 3），比盘 1 上最新的 txg 7 小：{raise_line}"
+        );
+        assert_eq!(
+            field(raise_line, "requested_floor"),
+            Some("3"),
+            "F 抬到上限：{raise_line}"
+        );
+        assert_eq!(raised.current_version.root.rollback_floor, CheckpointTxg(3));
+        assert_eq!(field(raise_line, "floor_before"), Some("0"), "{raise_line}");
+        assert_eq!(field(raise_line, "publishes"), Some("2"), "{raise_line}");
+        assert_eq!(
+            field(raise_line, "root_txgs"),
+            Some("10,11"),
+            "{raise_line}"
+        );
+        assert_eq!(
+            field(raise_line, "reclaimed"),
+            Some("1"),
+            "释放代 ≤ 3 的已释放落点（两盘同槽，按盘 0 报）：{raise_line}"
+        );
+        assert_eq!(
+            raised.current_version.root.checkpoint_txg,
+            CheckpointTxg(11)
+        );
+
+        let fourth_lines = lines_named(new_lines, "fourth_transaction");
+        assert_eq!(fourth_lines.len(), 1, "{new_lines:#?}");
+        assert_eq!(field(fourth_lines[0], "root_txg"), Some("9"));
+        assert_eq!(
+            field(fourth_lines[0], "transaction"),
+            Some("2"),
+            "同一个实例里发布 C 之后的下一个事务号"
+        );
+        assert_eq!(
+            field(fourth_lines[0], "segments"),
+            Some("16+2+1+2"),
+            "发布 D 与发布 C 同型"
+        );
+
+        let publish_lines: Vec<(Option<&str>, Option<&str>)> =
+            lines_named(new_lines, "publish_writes")
+                .into_iter()
+                .map(|line| (field(line, "publish"), field(line, "txg")))
+                .collect();
+        assert_eq!(
+            publish_lines,
+            vec![
+                (Some("fourth_transaction"), Some("9")),
+                (Some("raise_rollback_floor"), Some("10")),
+                (Some("raise_rollback_floor"), Some("11")),
+            ]
+        );
+        let windows = lines_named(new_lines, "publish_writes_against_device");
+        let window_summary: Vec<(Option<&str>, Option<&str>, Option<&str>)> = windows
+            .iter()
+            .map(|line| {
+                (
+                    field(line, "window"),
+                    field(line, "publishes"),
+                    field(line, "matches"),
+                )
+            })
+            .collect();
+        assert_eq!(
+            window_summary,
+            vec![
+                (Some("fourth_transaction"), Some("1"), Some("true")),
+                (Some("raise_rollback_floor"), Some("2"), Some("true")),
+            ],
+            "发布 D 与抬 F 那一串空发布按种类的合计都等于设备一层数的：{windows:#?}"
+        );
+        for window in &windows {
+            assert_eq!(
+                field(window, "by_kind_write_calls"),
+                field(window, "device_write_calls"),
+                "{window}"
+            );
+            assert_eq!(
+                field(window, "by_kind_written_bytes"),
+                field(window, "device_written_bytes"),
+                "{window}"
+            );
+        }
+        assert!(raised.every_window_matches_device);
+        assert_eq!(
+            clock.marked_segments(),
+            [
+                "reopen_and_writable_mount",
+                "third_transaction",
+                "fourth_transaction",
+                "raise_rollback_floor"
+            ]
+        );
+        assert_eq!(
+            lines_named(new_lines, "segment_timing").len(),
+            2,
+            "发布 D 与抬 F 各自的分段行：{new_lines:#?}"
+        );
+
+        let operations = stream.retained_operations();
+        let new_operations = &operations[operations_before_the_fourth_version..];
+        let counts_after_the_raise = device_call_counts(&raised.devices, &raised.plan);
+        for (index, identity) in [DeviceIdentity(0), DeviceIdentity(1)]
+            .into_iter()
+            .enumerate()
+        {
+            let counted =
+                counts_after_the_raise[index].since(counts_before_the_fourth_version[index]);
+            let projected = expected_device_events(new_operations, identity);
+            let projected_writes = projected
+                .iter()
+                .filter(|event| matches!(event, DeviceEvent::Write { .. }))
+                .count();
+            let projected_flushes = projected
+                .iter()
+                .filter(|event| matches!(event, DeviceEvent::Flush))
+                .count();
+            assert!(
+                counted.write_calls > 0,
+                "发布 D 与抬 F 两段盘 {} 收到了写",
+                identity.0
+            );
+            assert_eq!(
+                u64::try_from(projected_writes).expect("件数"),
+                counted.write_calls,
+                "发布 D 与抬 F 两段盘 {}：录制流投出的写与设备一层数到的",
+                identity.0
+            );
+            assert_eq!(
+                u64::try_from(projected_flushes).expect("件数"),
+                counted.barrier_calls + counted.force_unit_access_writes,
+                "发布 D 与抬 F 两段盘 {}：录制流投出的 FLUSH 与设备一层转发的屏障 + FUA 写",
+                identity.0
+            );
+        }
+
+        let cold: Vec<(DeviceIdentity, SparseBlockDevice)> = raised
+            .devices
+            .into_iter()
+            .map(|(identity, device)| (identity, device.into_inner().into_inner_and_operations().0))
+            .collect();
+        let report = recover(&cold, JournalPolicy::Consult);
+        match report.outcome {
+            RecoveryOutcome::FileRead { root, content } => {
+                assert_eq!(root, (InstanceGeneration(2), CheckpointTxg(11)));
+                assert!(content == fourth_file_content(), "冷恢复读回第四版");
+            }
+            RecoveryOutcome::NoFile { root } => panic!("冷恢复择到 {root:?} 却没有文件"),
+            RecoveryOutcome::Failed { root, failure } => {
+                panic!("冷恢复失败：{root:?} {failure:?}")
+            }
+        }
+    }
+
+    /// 增补 2 收口表第 58 行，发布 D 那一段（`raise-rollback-floor`）：发布 C 之后装上注入，发布 D 整池第 5 次写报错。
+    /// 挂载与发布 C 那几行照打在前面，失败账 4 次写与设备一层（重开之后那个注入计划数的）逐项相等；发布 D 没做成就不抬 F。
+    #[test]
+    fn fourth_version_publish_failing_midway_reports_the_writes_it_landed_and_they_equal_the_device_layer_count(
+    ) {
+        let parameters = e142_parameters(512, 512);
+        let geometry = geometry_of(&parameters);
+        let stream = SharedStream::new();
+        let mut clock = SegmentClock::start();
+        let switched = second_instance_run_up_to_the_third_version(
             &parameters,
-            &mut mounted_devices,
-            &mut mounted_allocator,
-            &mut current,
-            CheckpointTxg(0),
-            ShadowLedger::On,
-        );
-        let Err(MountError::RaiseFloorSequencePublishFailed(failed)) = raised else {
-            panic!(
-                "注入的写错该让抬 F 那一串停在第二次空发布：{:?}",
-                raised.as_ref().err()
+            &geometry,
+            &stream,
+            &mut clock,
+        );
+        switched
+            .plan
+            .arm(FaultSchedule::the_nth_call_across_the_pool(
+                InjectedFault::WriteFails,
+                5,
+            ));
+        let failed = failed_run_of(
+            publish_the_fourth_version_and_raise_the_rollback_floor(
+                &parameters,
+                switched,
+                &stream,
+                &geometry,
+                &mut clock,
+            ),
+            "发布 D",
+        );
+        for earlier_line in ["writable_mount", "third_transaction"] {
+            assert_eq!(
+                lines_named(&failed.lines, earlier_line).len(),
+                1,
+                "{earlier_line} 那一行在失败那几行前面照打：{:#?}",
+                failed.lines
             );
+        }
+        assert_the_failed_window_is_reconciled(&failed.lines, "fourth_transaction", &[], "4", "4");
+        assert!(lines_named(&failed.lines, "fourth_transaction").is_empty());
+        assert!(
+            lines_named(&failed.lines, "raise_rollback_floor").is_empty(),
+            "发布 D 没做成就不抬 F：{:#?}",
+            failed.lines
+        );
+        assert!(failed.cause.starts_with("发布 D："), "{}", failed.cause);
+    }
+
+    /// 抬 F 失败（增补 2 收口表第 58 行「`raise_rollback_floor` 同形」）：抬 F 的写入口是它自己开的、随错丢掉，
+    /// 已经落盘的那几次空发布与失败那一次已记的写随 `MountError::RaiseFloorSequencePublishFailed` 交回；`raise-rollback-floor`
+    /// 模式走的那个函数（`raise_the_rollback_floor_and_describe`）照可写挂载那一段的判法打失败账、与设备一层逐项比。
+    /// `raise-rollback-floor` 那条路走到发布 D（txg 9），装上注入，把 F 抬到上限（3）——推空发布直到两块盘上都有一条带这个 F 的根。
+    /// 注入摆在之后整池第 16 次写：第一次空发布（txg 10）13 次写已经落盘（四个固定点单元与 journal 记录每盘一份、根槽一次、系统配置每盘一次），
+    /// 第二次的第 3 个写报错 ⇒ 失败账 2 次写，两样相加 15 次与设备一层逐项相等。前面几段的结果行照打在失败那几行前面，成功那一行不打。
+    #[test]
+    fn failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count(
+    ) {
+        let parameters = e142_parameters(512, 512);
+        let geometry = geometry_of(&parameters);
+        let stream = SharedStream::new();
+        let mut clock = SegmentClock::start();
+        let switched = second_instance_run_up_to_the_third_version(
+            &parameters,
+            &geometry,
+            &stream,
+            &mut clock,
+        );
+        let published_the_fourth_version = match publish_the_fourth_version_and_describe(
+            &parameters,
+            switched,
+            &stream,
+            &geometry,
+            &mut clock,
+        ) {
+            Ok(published) => published,
+            Err(failed) => panic!("发布 D：{failed:?}"),
         };
-        assert_eq!(
-            current.root.checkpoint_txg,
-            CheckpointTxg(8),
-            "调用方的现行版本已经是落盘的第一次空发布"
+        published_the_fourth_version
+            .plan
+            .arm(FaultSchedule::the_nth_call_across_the_pool(
+                InjectedFault::WriteFails,
+                16,
+            ));
+        let failed = failed_run_of(
+            raise_the_rollback_floor_and_describe(
+                &parameters,
+                published_the_fourth_version,
+                &stream,
+                &geometry,
+                &mut clock,
+            ),
+            "抬 F",
         );
-        let (lines, _) = describe_failed_window(
+        for earlier_line in ["writable_mount", "third_transaction", "fourth_transaction"] {
+            assert_eq!(
+                lines_named(&failed.lines, earlier_line).len(),
+                1,
+                "{earlier_line} 那一行在失败那几行前面照打：{:#?}",
+                failed.lines
+            );
+        }
+        assert_the_failed_window_is_reconciled(
+            &failed.lines,
             "raise_rollback_floor",
-            &format!("{:?}", failed.cause),
-            &failed.writes_of_persisted_publishes,
-            &failed.writes_of_failed_publishes,
-            pool_writes_between(
-                &counts_before_raise,
-                &device_call_counts(&mounted_devices, &mounted_plan),
-            ),
+            &["13"],
+            "2",
+            "15",
+        );
+        assert!(
+            lines_named(&failed.lines, "raise_rollback_floor").is_empty(),
+            "失败的抬 F 不打成功那一行"
         );
-        assert_the_failed_window_is_reconciled(&lines, "raise_rollback_floor", &["13"], "2", "15");
+        assert!(failed.cause.starts_with("抬 F："), "{}", failed.cause);
     }
 
     /// 宿主检查（`first_transaction_device_log_check`）拿录制流投到每块盘上的事件当「程序的信念」：每个写一件、FUA 写之后一个 FLUSH、
diff -ruN -x target tree/crates/singlefs-harness/src/history.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-harness/src/history.rs	2026-09-24 16:15:19.532886740 +0000
+++ tree/crates/singlefs-harness/src/history.rs	2026-09-24 18:48:58.241580116 +0000
@@ -1928,8 +1928,7 @@
             )
         }
         PublishError::AllocationRecordsExceedOneNode { .. } => "AllocationRecordsExceedOneNode",
-        PublishError::AccountingEntriesExceedOneNode { .. } => "AccountingEntriesExceedOneNode",
-        PublishError::MappingEntriesExceedOneNode { .. } => "MappingEntriesExceedOneNode",
+        PublishError::MultiLevelCodeTwoTreeRefused { .. } => "MultiLevelCodeTwoTreeRefused",
         PublishError::ReleaseNotInMapping { .. } => "ReleaseNotInMapping",
         PublishError::ReleaseTargetNotAllocated { .. } => "ReleaseTargetNotAllocated",
         PublishError::ReleaseTargetAlreadyReleased { .. } => "ReleaseTargetAlreadyReleased",
@@ -1940,12 +1939,12 @@
         PublishError::MappingEntryNarrowerThanItsFieldTable { .. } => {
             "MappingEntryNarrowerThanItsFieldTable"
         }
-        PublishError::ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided {
-            ..
-        } => "ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided",
-        PublishError::ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported {
-            ..
-        } => "ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported",
+        PublishError::MappingEntryLocationOnADeviceOutsideThePool { .. } => {
+            "MappingEntryLocationOnADeviceOutsideThePool"
+        }
+        PublishError::MappingEntryLocationsOnTheSameDevice { .. } => {
+            "MappingEntryLocationsOnTheSameDevice"
+        }
         PublishError::ContentExceedsDataUnit { .. } => "ContentExceedsDataUnit",
         PublishError::ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided { .. } => {
             "ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided"
@@ -1963,6 +1962,9 @@
         PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_) => {
             "TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees"
         }
+        PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. } => {
+            "PublishFrozenAfterAWriteFailureIsNotResentYet"
+        }
         PublishError::BlockDevice(cause) => {
             return format!(
                 "PublishError::BlockDevice({})",
@@ -2025,6 +2027,9 @@
             ..
         } => "RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided"
             .to_string(),
+        RecoveryFailure::RollbackWitnessTableFullWhoseHandlingIsUndecided { .. } => {
+            "RecoveryFailure::RollbackWitnessTableFullWhoseHandlingIsUndecided".to_string()
+        }
     }
 }
 
diff -ruN -x target tree/crates/singlefs-harness/src/model_comparison.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-harness/src/model_comparison.rs	2026-09-24 16:15:19.532944978 +0000
+++ tree/crates/singlefs-harness/src/model_comparison.rs	2026-09-24 18:48:58.241685305 +0000
@@ -69,7 +69,15 @@
         }
         TransactionUnit::InodeRoot => ModelUnitRole::InodeRoot,
         TransactionUnit::AllocationTree => ModelUnitRole::AllocationTree,
+        // 模型罩的池两块盘、15 行记账，映射条目恒 6 条：两棵树恒只有一个节点（根兼叶），根之下的节点只在压小容量的
+        // 只供测试的开关下、或记账行装不下一个节点的池（80 块盘起）上出现，随机历史与层 0 的固定脚本都不装那个开关。
+        TransactionUnit::AccountingTreeNodeBelowTheRoot(position) => {
+            panic!("模型今天只罩记账树一个节点的池：{position:?}")
+        }
         TransactionUnit::AccountingTree => ModelUnitRole::AccountingTree,
+        TransactionUnit::MappingTreeNodeBelowTheRoot(position) => {
+            panic!("模型今天只罩中央映射树一个节点的池：{position:?}")
+        }
         TransactionUnit::MappingTree => ModelUnitRole::MappingTree,
         TransactionUnit::TreeTable => ModelUnitRole::TreeTable,
         // 模型把实例表链当一个角色：各片的落点与分配记录按这次之后的行数现算片数（`model::instance_table_pages_for_rows`）。
@@ -172,9 +180,6 @@
         PublishError::AllocationRecordsExceedOneNode { .. } => {
             explained(ModelRefusalReason::AllocationRecordNodeWall)
         }
-        PublishError::AccountingEntriesExceedOneNode { .. } => {
-            explained(ModelRefusalReason::AccountingNodeWall)
-        }
         PublishError::ContentExceedsDataUnit { .. } => {
             explained(ModelRefusalReason::ContentExceedsDataUnitPayload)
         }
@@ -188,9 +193,9 @@
         // extent 树要长内部节点那一条同理：随机历史一次都不调 `publish_sequential_write`，写的文件恒一个数据单元，它出现就是对不上。
         // inode 树写入被拒那一条同样：随机历史一次都不调 `publish_new_inodes`，
         // 而它跑的那几种发布每次最多改一片叶容器、重写的角色最多九个。
-        // 映射节点装不下那一条同理：条目数 = 五个固定角色 + 叶容器数，随机历史里恒是 1 片叶 ⇒ 恒 6 条，
-        // 离一个节点的 294 条差得远；它出现就是模型与实现对不上。
-        PublishError::MappingEntriesExceedOneNode { .. }
+        // 多层码 2 树算不出形状那一条同理：两棵树装不下一个节点时分裂、不拒，只有长到 257 层或上一版的形状按分隔 key 走不通才拒，
+        // 随机历史里两棵树恒只有一个节点；它出现就是模型与实现对不上。
+        PublishError::MultiLevelCodeTwoTreeRefused { .. }
         | PublishError::InodeTreeWriteRefused(_)
         | PublishError::ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided { .. }
         | PublishError::ReleaseNotInMapping { .. }
@@ -199,15 +204,15 @@
         | PublishError::ReleaseTargetLocationsOnDifferentSlots { .. }
         | PublishError::ReleaseSpanMismatch { .. }
         | PublishError::MappingEntryNarrowerThanItsFieldTable { .. }
-        // 释放之前读盘核校验和时位置项指的盘不在池里、或只有一部分副本对不上：前者只有坏镜像、外来镜像上有，后者要盘上那一份坏了
-        // 或读出坏字节才走得到；健康的内存盘上都不该出现，模型里没有它们的理由（两格条款都没写）。
-        | PublishError::ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided {
-            ..
-        }
-        | PublishError::ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported { .. }
+        // 映射条目的位置项指池外的盘、或两条指同一块盘（当映射条目损坏）：只有坏镜像、外来镜像上有；
+        // 健康的内存盘上都不该出现，模型里没有它们的理由。
+        | PublishError::MappingEntryLocationOnADeviceOutsideThePool { .. }
+        | PublishError::MappingEntryLocationsOnTheSameDevice { .. }
         // 第一个文件版本读不出那一版的树表、水位离 u64::MAX 不到八个号：健康的内存盘上都不该出现。
         | PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. }
         | PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_)
+        // 冻结着一次没重发的发布：随机历史里一次发布失败就整段停下、不接着发，健康的内存盘上不该出现。
+        | PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
         | PublishError::BlockDevice(_) => ObservedRefusalReason::Unexplained,
     }
 }
diff -ruN -x target tree/crates/singlefs-harness/src/model.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-harness/src/model.rs	2026-09-24 16:15:19.532924658 +0000
+++ tree/crates/singlefs-harness/src/model.rs	2026-09-24 16:50:54.714341234 +0000
@@ -291,8 +291,6 @@
     FirstFileVersionOnAVersionThatAlreadyHasAFile,
     /// 分配记录树第一版只有一个节点（容量墙，收口表第 39 行：模型答允许拒绝的区间）。
     AllocationRecordNodeWall,
-    /// 记账树第一版只有一个节点。
-    AccountingNodeWall,
     /// 单元区装不下（D28（挂载期承诺量） 已定项 1 的准入；模型答允许拒绝的区间）。
     UnitAreaWall,
     /// 回退目标不在根环里（D23（journal 的角色与格式） 已定项 14：候选集是根环里的根）。
@@ -321,7 +319,6 @@
                 "要建在上面的那一版已经有过文件版本（再写一版走覆盖写）"
             }
             ModelRefusalReason::AllocationRecordNodeWall => "分配记录树一个节点装不下",
-            ModelRefusalReason::AccountingNodeWall => "记账树一个节点装不下",
             ModelRefusalReason::UnitAreaWall => "单元区装不下",
             ModelRefusalReason::RollbackTargetNotInRing => "回退目标不在根环里",
             ModelRefusalReason::RollbackTargetBelowEffectiveFloor => "回退目标低于 F_生效",
@@ -340,7 +337,6 @@
             ModelRefusalReason::ContentExceedsDataUnitPayload
             | ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn
             | ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile
-            | ModelRefusalReason::AccountingNodeWall
             | ModelRefusalReason::RollbackTargetNotInRing
             | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
             | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
@@ -1027,7 +1023,7 @@
         &self,
         operation: ModelOperationKind,
         starting_version: ModelRootKey,
-        mut required_refusals: BTreeSet<ModelRefusalReason>,
+        required_refusals: BTreeSet<ModelRefusalReason>,
         permitted_refusals: BTreeSet<ModelRefusalReason>,
         roots: Vec<ModelRoot>,
     ) -> ModelAnswer {
@@ -1036,9 +1032,12 @@
             root.role_written_at.get(&ModelUnitRole::AccountingTree)
                 == Some(&root.key.checkpoint_txg)
         });
-        if writes_accounting_rows && self.accounting_rows_exceed_one_node() {
-            required_refusals.insert(ModelRefusalReason::AccountingNodeWall);
-        }
+        // 记账行装不下一个节点时记账树分裂、不拒（D8（核心索引结构） 已定项 11），分裂多出来的节点每个各占一条分配记录；
+        // 模型的分配记录账按「记账树一个节点」数，只罩行数装得进一个节点的池（盘数不到 80）——超出就是模型没罩到，不是实现错。
+        assert!(
+            !(writes_accounting_rows && self.accounting_rows_exceed_one_node()),
+            "模型今天只罩记账树一个节点的池：分裂多出来的节点模型的分配记录账没算"
+        );
         ModelAnswer {
             operation,
             required_refusals,
@@ -1394,7 +1393,6 @@
             ModelRefusalReason::ContentExceedsDataUnitPayload
             | ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn
             | ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile
-            | ModelRefusalReason::AccountingNodeWall
             | ModelRefusalReason::RollbackTargetNotInRing
             | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
             | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
diff -ruN -x target tree/crates/singlefs-harness/src/on_device_modes.rs
--- /tmp/claude-1000/m2-final-code-r1/tree/crates/singlefs-harness/src/on_device_modes.rs	2026-09-24 02:30:54.748746865 +0000
+++ tree/crates/singlefs-harness/src/on_device_modes.rs	2026-09-24 17:28:54.669972875 +0000
@@ -1,13 +1,18 @@
-//! 虚机档 `first_transaction_on_device` 的五个模式，以及第一个事务之后那两次发布（发布 B、可写挂载之后的发布 C）的写路。
+//! 虚机档 `first_transaction_on_device` 的六个模式，以及第一个事务之后那几步写（发布 B、可写挂载之后的发布 C、
+//! `raise-rollback-floor` 在发布 C 之后的发布 D 与抬 F）的写路。
 //!
-//! 虚机里的真设备与宿主上的 `first_transaction_device_log_check` 都跑这一份：模式名、两版的内容与写入时刻、每次发布接在哪一版上，
-//! 两边不各抄一份，宿主重跑的才是同参数同字节的那条写路（里程碑「第二个事务」增补 2 收口表第 30 行：宿主那一侧要接着重跑发布 B、
-//! 可写挂载与发布 C，把程序录制流与设备侧日志逐项比）。设备侧日志与这里不共享一行代码。
+//! 虚机里的真设备与宿主上的 `first_transaction_device_log_check` 都跑这一份：模式名、各版的内容与写入时刻、每次发布接在哪一版上、
+//! 抬 F 抬到多少，两边不各抄一份，宿主重跑的才是同参数同字节的那条写路（里程碑「第二个事务」增补 2 收口表第 30 行：宿主那一侧要接着重跑发布 B、
+//! 可写挂载与发布 C，把程序录制流与设备侧日志逐项比；第 58 行：抬 F 那一串发布同样要在真设备上跑、同样逐项比）。设备侧日志与这里不共享一行代码。
 
 use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
 use singlefs_core::allocator::{DeviceFreeMap, PoolAllocator};
 use singlefs_core::block_device::BlockDevice;
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
+use singlefs_core::mount::{
+    raise_rollback_floor, rollback_floor_ceiling, MountError, RaisedFloor, ShadowLedger,
+};
+use singlefs_core::recovery::{choose_system_configuration, instance_table_chain_of_root};
 use singlefs_core::transaction::{
     publish_overwrite, FirstFile, PoolWriter, PublishError, TransactionOutput,
 };
@@ -15,7 +20,7 @@
 
 use crate::scenario::{first_file_content, FIXED_WRITE_TIME_SECONDS};
 
-/// 虚机档的五个模式（命令行最后一个参数）。
+/// 虚机档的六个模式（命令行最后一个参数）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum OnDeviceRunMode {
     Direct,
@@ -23,6 +28,9 @@
     SkipFirstTransactionBarrier,
     SecondTransaction,
     SecondInstance,
+    /// `second-instance` 那条路走完（发布 C 之后），同一次挂载里再覆盖写一次（发布 D，[`publish_the_fourth_version`]），
+    /// 接着把 F 抬到上限（[`raise_the_rollback_floor_to_its_ceiling`]）。
+    RaiseRollbackFloor,
 }
 
 /// 第一个事务之后，这个模式在同一对盘上还跑哪几步写。
@@ -33,15 +41,20 @@
     SecondVersion,
     /// 发布 B 之后丢掉写的那一套句柄、同一对盘冷重开，走可写挂载（恢复、取号、写行、暖机，步 3），再发布 C。
     SecondVersionThenSecondInstance,
+    /// 与上一个相同，发布 C 之后同一次挂载里再覆盖写一次（发布 D：按新到旧数，D、C、B、A 是前 4 条非空有效根，第 4 新的落在 A（txg 3），
+    /// F 的上限才抬得过 0），
+    /// 再把 F 抬到上限：推带新 F 的空发布直到每块盘上都有一条（D16（发布语义） 已定项 1「抬 F 的上限」「生效」）。
+    SecondVersionThenSecondInstanceThenFourthVersionThenRaiseOfTheRollbackFloor,
 }
 
 impl OnDeviceRunMode {
-    pub const ALL: [OnDeviceRunMode; 5] = [
+    pub const ALL: [OnDeviceRunMode; 6] = [
         OnDeviceRunMode::Direct,
         OnDeviceRunMode::PageCache,
         OnDeviceRunMode::SkipFirstTransactionBarrier,
         OnDeviceRunMode::SecondTransaction,
         OnDeviceRunMode::SecondInstance,
+        OnDeviceRunMode::RaiseRollbackFloor,
     ];
 
     /// 命令行上的写法（跑批脚本与门禁 55 号按它送参数）。
@@ -53,6 +66,7 @@
             OnDeviceRunMode::SkipFirstTransactionBarrier => "skip-first-transaction-barrier",
             OnDeviceRunMode::SecondTransaction => "second-transaction",
             OnDeviceRunMode::SecondInstance => "second-instance",
+            OnDeviceRunMode::RaiseRollbackFloor => "raise-rollback-floor",
         }
     }
 
@@ -86,10 +100,13 @@
             OnDeviceRunMode::SecondInstance => {
                 PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance
             }
+            OnDeviceRunMode::RaiseRollbackFloor => {
+                PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstanceThenFourthVersionThenRaiseOfTheRollbackFloor
+            }
         }
     }
 
-    /// 这个模式最后发布的那一版文件内容：冷重开之后该读回的。
+    /// 这个模式最后发布的那一版文件内容：冷重开之后该读回的。抬 F 的空发布不碰 inode 树，`raise-rollback-floor` 读回的是发布 D 写的第四版。
     #[must_use]
     pub fn last_published_content(self) -> Vec<u8> {
         match self.publishes_after_the_first_transaction() {
@@ -98,6 +115,9 @@
             PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance => {
                 third_file_content()
             }
+            PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstanceThenFourthVersionThenRaiseOfTheRollbackFloor => {
+                fourth_file_content()
+            }
         }
     }
 }
@@ -106,6 +126,8 @@
 pub const SECOND_VERSION_WRITE_TIME_SECONDS: u64 = FIXED_WRITE_TIME_SECONDS + 60;
 /// 发布 C 的写入时刻：第一个事务之后 120 秒。
 pub const THIRD_VERSION_WRITE_TIME_SECONDS: u64 = FIXED_WRITE_TIME_SECONDS + 120;
+/// 发布 D 的写入时刻：第一个事务之后 180 秒。
+pub const FOURTH_VERSION_WRITE_TIME_SECONDS: u64 = FIXED_WRITE_TIME_SECONDS + 180;
 
 /// 第二版的内容（发布 B）：与第一版不同长度、不同字节，读回时分得开。
 #[must_use]
@@ -123,6 +145,14 @@
         .collect()
 }
 
+/// 第四版的内容（`raise-rollback-floor` 模式的发布 D）：与前三版都不同长度、不同字节。
+#[must_use]
+pub fn fourth_file_content() -> Vec<u8> {
+    (0..1800usize)
+        .map(|index| u8::try_from((index * 7 + 19) % 253).expect("小于 256"))
+        .collect()
+}
+
 /// 一次发布失败：发布交回的错，以及这次发布的写入口上落盘阶段失败的发布各自已记的写
 /// （`PoolWriter::writes_of_failed_publishes`，增补 2 收口表第 58 行：失败那次落盘的写不属于任何一次成功发布的账，
 /// 不交出来，设备一层数到的写就对不上）。落盘之前就失败的（准入、释放判定、分配）一个写都没发，这一份是空的。
@@ -194,6 +224,68 @@
     )
 }
 
+/// `raise-rollback-floor` 的发布 D：发布 C 之后、同一次挂载里接在发布 C 那一版上覆盖写第四版，实例照发布 C 的（挂载取到的那个）。
+/// 按新到旧数，它与 C、B、A 是这段历史上前 4 条非空有效根，第 4 新的落在 A（txg 3），F 的上限因此抬得过 0
+/// （[`raise_the_rollback_floor_to_its_ceiling`]）。
+///
+/// # Errors
+/// 发布的错，连同这次写入口的失败账（[`FailedPublish`]）。
+pub fn publish_the_fourth_version<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut [(DeviceIdentity, Device)],
+    allocator: &mut PoolAllocator,
+    third_version: &TransactionOutput,
+) -> Result<TransactionOutput, FailedPublish> {
+    publish_one_overwrite(
+        parameters,
+        devices,
+        allocator,
+        third_version,
+        OverwrittenVersion {
+            content: &fourth_file_content(),
+            write_time_seconds: FOURTH_VERSION_WRITE_TIME_SECONDS,
+            instance: third_version.root.instance,
+        },
+    )
+}
+
+/// `raise-rollback-floor` 模式：发布 D 之后、同一次挂载里把 F 抬到上限，推带这个 F 的空发布直到每块盘上都有一条（生效），
+/// `current_version` 随之换成最后落盘的那一次空发布（失败时是最后落盘的那一次，见 [`MountError::RaiseFloorSequencePublishFailed`]）。
+///
+/// 上限照 D16（发布语义） 已定项 1「抬 F 的上限」由 core 的 [`rollback_floor_ceiling`] 现算（min(每块盘上最新的持久有效根,
+/// 第 4 新的非空持久有效根)），入参与 [`raise_rollback_floor`] 自己算上限时的一样：现行那一版的根指着的实例表、现行的 F、盘上读得到的根。
+/// 这里算完到 `raise_rollback_floor` 再算一次之间没有写，两次读的是同一份盘上状态；对不上时 `raise_rollback_floor`
+/// 在任何写之前拒绝（`RollbackFloorAboveCeiling`）。
+///
+/// # Errors
+/// 选系统配置、读现行那一版的实例表、算上限的错（都在任何写之前）；[`raise_rollback_floor`] 的错原样交回：
+/// 那一串空发布里有一次发不出去时是 `RaiseFloorSequencePublishFailed`，带着这一串的写账。
+pub fn raise_the_rollback_floor_to_its_ceiling<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    allocator: &mut PoolAllocator,
+    current_version: &mut TransactionOutput,
+) -> Result<RaisedFloor, MountError> {
+    let system_configuration = choose_system_configuration(&*devices)?;
+    let instance_table = instance_table_chain_of_root(&*devices, &current_version.root)
+        .map_err(|_unreadable_or_malformed| MountError::InstanceTableMalformed)?
+        .records;
+    let ceiling = rollback_floor_ceiling(
+        devices,
+        &system_configuration,
+        current_version.root.rollback_floor,
+        &instance_table,
+    )?;
+    raise_rollback_floor(
+        parameters,
+        devices,
+        allocator,
+        current_version,
+        ceiling,
+        ShadowLedger::On,
+    )
+}
+
 /// 一次覆盖写要写的那一版：内容、写入时刻、发布用的实例代号。
 struct OverwrittenVersion<'content> {
     content: &'content [u8],
@@ -236,22 +328,34 @@
             assert_eq!(OnDeviceRunMode::from_argument(mode.argument()), Some(mode));
         }
         assert_eq!(OnDeviceRunMode::from_argument("second_instance"), None);
+        assert_eq!(OnDeviceRunMode::from_argument("raise_rollback_floor"), None);
         assert_eq!(
             OnDeviceRunMode::every_argument_for_usage(),
-            "direct | page-cache | skip-first-transaction-barrier | second-transaction | second-instance"
+            "direct | page-cache | skip-first-transaction-barrier | second-transaction | second-instance | raise-rollback-floor"
         );
     }
 
-    /// 三版内容两两不同（长度与字节）：冷重开读回哪一版分得开。
+    /// 四版内容两两不同（长度与字节）：冷重开读回哪一版分得开。
     #[test]
-    fn the_three_versions_differ_in_length_and_bytes_and_each_mode_reads_back_its_last_one() {
+    fn the_four_versions_differ_in_length_and_bytes_and_each_mode_reads_back_its_last_one() {
         let first = first_file_content();
         let second = second_file_content();
         let third = third_file_content();
-        assert_eq!((first.len(), second.len(), third.len()), (3000, 4100, 2500));
+        let fourth = fourth_file_content();
+        assert_eq!(
+            (first.len(), second.len(), third.len(), fourth.len()),
+            (3000, 4100, 2500, 1800)
+        );
         assert_ne!(second, first);
         assert_ne!(third, first);
         assert_ne!(third, second);
+        for earlier in [&first, &second, &third] {
+            assert_ne!(
+                &fourth[..],
+                &earlier[..fourth.len()],
+                "第四版的字节与前三版同长的前缀也不同"
+            );
+        }
         assert_eq!(OnDeviceRunMode::Direct.last_published_content(), first);
         assert_eq!(OnDeviceRunMode::PageCache.last_published_content(), first);
         assert_eq!(
@@ -266,5 +370,10 @@
             OnDeviceRunMode::SecondInstance.last_published_content(),
             third
         );
+        assert_eq!(
+            OnDeviceRunMode::RaiseRollbackFloor.last_published_content(),
+            fourth,
+            "抬 F 的空发布不碰 inode 树，冷重开读回的是发布 D 那一版"
+        );
     }
 }
```

## 二、`tests/` 下的改动（19 个文件，只列文件名与增删行数；增删数由材料员对 `r1-to-r2.diff` 里对应段落数 `+`/`-` 开头且非 `+++`/`---` 的行得出，不抄全文）

| 文件（`crates/singlefs-harness/` 下的相对路径） | + | − |
|---|---|---|
| tests/common_tree_split/mod.rs | +288 | -0 |
| tests/first_transaction_step_five_publish.rs | +7 | -0 |
| tests/second_transaction_mapping_node_admission.rs | +119 | -142 |
| tests/second_transaction_parallel_line_one_last_record_flag.rs | +71 | -0 |
| tests/second_transaction_parallel_line_one_multi_unit_file.rs | +3 | -2 |
| tests/second_transaction_parallel_line_two_mounted_read.rs | +192 | -33 |
| tests/second_transaction_supplement_one_write_accounting.rs | +9 | -14 |
| tests/second_transaction_supplement_three_fault_injection.rs | +57 | -31 |
| tests/second_transaction_supplement_two_accounting_node_full.rs | +87 | -44 |
| tests/second_transaction_supplement_two_multi_record_transaction_zero_publishes.rs | +372 | -0 |
| tests/second_transaction_supplement_two_publish_failure_resent_unchanged.rs | +295 | -0 |
| tests/second_transaction_supplement_two_release_checksum_quarantine.rs | +277 | -49 |
| tests/second_transaction_supplement_two_rollback_witness_layer0.rs | +457 | -0 |
| tests/second_transaction_supplement_two_rollback_witness.rs | +716 | -0 |
| tests/second_transaction_supplement_two_row_publish_checks_before_acquisition.rs | +246 | -0 |
| tests/second_transaction_supplement_two_tree_nodes_and_the_central_mapping.rs | +108 | -34 |
| tests/second_transaction_supplement_two_tree_split_layer0.rs | +385 | -0 |
| tests/second_transaction_supplement_two_tree_split.rs | +793 | -0 |
| tests/system_configuration_mutability_classes.rs | +5 | -0 |

## 三、新文件全文

`r1-to-r2.diff` 用 `diff -ruN` 生成：本轮新建的两个文件（`crates/singlefs-core/src/code_two_tree.rs`、`crates/singlefs-core/src/rollback_witness.rs`，均属 `src/`）在第一节的 diff 里已经是整份 `+` 行（对侧目录里不存在同名文件，`diff -ruN` 把新文件的全部内容当增量输出），不再重复附一份全文；`tests/` 下的新文件同理已含在第一节被截断的行数统计里，全文按第二节的口径不抄。

## 四、定义相对 HEAD 的 diff（`defs-vs-head.diff`，8 个文件，`git diff HEAD` 生成，整段进）

```diff
diff --git a/.claude/agent-common.md b/.claude/agent-common.md
index f2f9cf7..bacea6b 100644
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -18,6 +18,7 @@
 - 每个定义都照守、不再写进各自「开工先读：」一行的三处：跑命令照 `.claude/singlefs-ai-sop/rules/command-safety.md`「退不回去的操作，动手前先想一遍」「`pkill -f` / `killall` 一律禁用」两节；
   给人看的文字照 `.claude/singlefs-ai-sop/rules/writing-discipline.md`「说人话」一节；说外部状态之前现查（`.claude/singlefs-ai-sop/rules/verify-before-claiming.md` 开头一节）。
 - 本机时钟是 UTC，人在东京（JST，UTC+9）；报告里的时刻写清是哪个时区。
+- 候选、臂、方案、判据、提问编号有了变体，起一个新名字（那一族里下一个没用过的号，或一个短的描述性名字），不在原名后面加撇号类角标（U+2032、U+2033、U+2034、U+02B9、U+02BA）；全仓由门禁 12 号判，写法见 `.claude/rules/path-moves.md`「变体起新名字，不用角标」。
 - 派发提示里没给、定义里也没写的项目事实（某份 kb 在哪、某条决策的原文），去仓里现查，不凭印象补。
 - **找不到历史实验的数据、提示或产物，去 `git log` 里看。** 上一轮及更早的实验记录不留在工作区：
   这一轮提交之后由下一次提交删掉上一次那批，本轮的留着（`.claude/gate.d/91-archive-past-rounds.sh` 判这一条）。
@@ -42,13 +43,18 @@
 - 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。
 - 不读派发提示给的禁读清单里的文件。
 - 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
+- 重型测试（层 0、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。`crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
+- 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，名字含 `layer0` 的测试二进制也不许。
+- 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
 - 要编译、跑门禁阶段、跑变异或产物的，开跑前 `ps -o pid,args -u "$(id -u)"` 看负载：有性能测量在跑（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`）就报「在等 pid …（命令）」停下；只有别的 `cargo`、`gate.sh` 在跑的，加 `nice -n 19` 照常跑（cargo 自己排队拿文件锁），报告里记下 `ps` 看到了什么、等锁等了多久。定义另有更严要求的照定义。
+- 读大文件（超过 500 行）先 grep 定位、再按行段读（Read 的 offset / limit），不整份读；同一份文件读过一次、之后没改过，就不再整份重读。长命令的输出落进草稿目录的文件，只读汇总行（`tail -n 20`、`grep -c`、`grep ✗`），不把整份日志、整份 diff 读进上下文。
+- 派发提示里有「线程上限：N」的，编译、测试、变异、产物、原型扫描的每条命令都用 `bash research/scripts/capped.sh N <命令>` 起（它把 cargo 与 `crates/singlefs-harness` 各装置读的线程变量一次设成 N）；实验装置自己的线程变量（`KS_THREADS` 这类）同样设成不超过 N。主 agent 发消息改了 N，从下一条命令起照新的。
 - **崩溃点测试不衡量时间成本，也不为省时间缩范围**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，也不用先算它值不值。
 - 禁止自行扩大任务范围，只改自己的部分。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
 - 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
 - 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
   等长活时不起缓存计时器，结束本轮直接等完成通知。
-  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）只记不拦，把没超时的等待循环、按模式找进程这类命令交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
+  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）对两种写法在执行前拒绝：前台没有超时的等待循环、把活放出追踪（`disown`、`nohup … &` 这类）；run_in_background 里的等待循环与按模式找进程这类命令只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
 
 ## 门禁
 
diff --git a/.claude/agents/crash-verifier.md b/.claude/agents/crash-verifier.md
index b74d381..a2ad771 100644
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -1,6 +1,6 @@
 ---
 name: crash-verifier
-description: 崩溃一致性验证员：crates/ 改动写完、走过三方对抗之后，逐个跑层 0 崩溃点重放、QEMU 真设备、herd7 内存序、crates 变异表这几道重阶段，原样交判定与计数。只在主 agent 点名派发、并给出这次改动的范围时用；不要自动派发。
+description: 崩溃一致性验证员：提交代码时（或用户要求时）逐个跑层 0 崩溃点重放、QEMU 真设备、herd7 内存序、crates 变异表这几道重阶段，原样交判定与计数。只在主 agent 点名派发、并给出这次改动的范围时用；不要自动派发。
 tools: Read, Bash
 model: sonnet
 omitClaudeMd: true
@@ -16,12 +16,13 @@ omitClaudeMd: true
 ## 输入（主 agent 必须给）
 
 - 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
+- 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
 - 报告路径与草稿目录。
 
 ## 做什么
 
 1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号，等它结束。
-2. 按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个：`nice -n 19 bash .claude/gate.d/<文件>`，54 号默认不带 `--full`（快档加核全绿标记），派发提示点名要层 0 全量时才带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
+2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认不带 `--full`（快档加核全绿标记），派发提示点名要层 0 全量时才带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
 3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
 4. 计数行原样抄：崩溃状态数、恢复结果、与 E142 产物或闭式比对的那几行。输出里读不到判定行的阶段记「作废」，不记通过。
 5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证。这三条命令不单独下判断。
diff --git a/.claude/agents/experiment-runner.md b/.claude/agents/experiment-runner.md
index e6305da..cf04f24 100644
--- a/.claude/agents/experiment-runner.md
+++ b/.claude/agents/experiment-runner.md
@@ -29,7 +29,7 @@ omitClaudeMd: true
 3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
 4. 跑产物进 `research/results/`；跑前删旧输出，跑完认本轮才有的完成标记。重跑已有实验时，`research/results/` 里已有的产物不覆盖：与它逐字节一致就不新存；对不上就按今天的日期另存一份，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
 4b. 产物出来之后先自查齐不齐：按登记的「臂 × 历史 × 几何」数一遍产物里每一类结果行的条数与字段，缺行、缺字段（例如某一族没有臂字段）、某一族一次都没造出被测形状，都列进报告，不许只看末行完成标记。报告里每个判据按每条臂全列判定，不挑「突出发现」报——对某条候选不利的那一半同样要写。分族、分臂的计数用一条命令从产物里数，报告与实验页贴那条命令和它的原样输出，不用文字重写族名单；给「这几族为什么是 0」写解释之前，先把那几族的产物行贴出来，确认它们真是 0。
-5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。送进虚机跑的实验，开跑前先 `bash research/scripts/vm-bench.sh --selftest`（`.claude/kb/vm-harness.md`）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；门禁 65 号查读子进程输出的循环里一边取时间一边输出。
+5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；门禁 65 号查读子进程输出的循环里一边取时间一边输出。
 6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）；其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
 7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。
 7b. 实验页的 `### 影响的决策` 一节照第 7 步写实验页的规则写，形态与判据不在这份定义里复述。这一步你要做的三件：① 要写支撑或推翻时，先 `grep -n 'E<号>（' .claude/kb/decisions/<那条决策的文件>` 看那条分项的「**依据**」段引没引这个实验——引了才写支撑或推翻，没引就写备料，并把「这一格该不该升成支撑」列进报告交主 agent，**不自己去改决策文件**（写范围闸也不放行）。② 那条决策还在 `.claude/decision-links-pending` 里时不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。③ 重跑已有实验、换了产物、加了历史条目之后，表里每一行都重新回看一遍。
diff --git a/.claude/agents/gate-triage.md b/.claude/agents/gate-triage.md
index f7d3fb9..60c48be 100644
--- a/.claude/agents/gate-triage.md
+++ b/.claude/agents/gate-triage.md
@@ -1,6 +1,6 @@
 ---
 name: gate-triage
-description: 门禁分诊：跑准入门禁，把每个红阶段判成这一轮的改动、别的会话的改动还是环境，原样抄下一步。只在主 agent 点名派发、并给出这一轮的暂存状态时用；不要自动派发。
+description: 门禁分诊：提交代码时（或用户要求时）跑准入门禁，把每个红阶段判成这一轮的改动、别的会话的改动还是环境，原样抄下一步。只在主 agent 点名派发、并给出这一轮的暂存状态时用；不要自动派发。
 tools: Read, Bash
 model: sonnet
 omitClaudeMd: true
@@ -16,13 +16,14 @@ omitClaudeMd: true
 ## 输入（主 agent 必须给）
 
 - 这一轮的改动已经按 `research/scripts/stage-mine.py` 暂存了没有（没暂存就不派你，或者主 agent 明写「跑全量工作区」）。
+- 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
 - 这一轮暂存区 diff 的范围：`git diff --cached --stat` 原样。主 agent 明写跑全量工作区时，暂存区多半是空的，这时主 agent 另给「这一轮改过的文件清单」，归属按清单判。
 - 报告路径与草稿目录。
 
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载；另有别的 `gate.sh` 在跑就等它结束。
-2. 暂存过的跑 `nice -n 19 bash .claude/scripts/gate.sh --staged`；主 agent 明写全量的跑不带参数。超过 Bash 单次上限时后台跑、结束后读输出。
+2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`（用户要求时 `=user-request`）；主 agent 明写全量的跑不带参数（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（54 号只核全绿标记），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号单跑照带。超过 Bash 单次上限时后台跑、结束后读输出。
 3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（54 号的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径 ⇒「这一轮」，下一步原样抄 54 号出路里的三行命令，一条都没碰 ⇒「不是这一轮」；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
 4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。
 
diff --git a/.claude/agents/implementation-writer.md b/.claude/agents/implementation-writer.md
index 7a8701f..9e38948 100644
--- a/.claude/agents/implementation-writer.md
+++ b/.claude/agents/implementation-writer.md
@@ -25,7 +25,7 @@ omitClaudeMd: true
 1. 开跑前照共用约束「不做」一节看负载。
 2. 读条款与代码，写实现与测试。名字、分支、类型、错误照 `code-discipline.md`：不缩写、不写 `_ =>`、newtype、`expect` 写清依赖哪条不变量。
 3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红，记「改坏哪一行 → 哪条断言红」与同时红了哪些测试；每份副本用它自己的 target（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）；改坏的副本要还原时，从原件拷回之后对被改的文件 `touch`，或删掉这份副本重拷，不用 `rsync -a` 拷回了事；先跑一份不改动的副本：已经红的测试记成基线红集，变异证明只看基线红集之外的测试，你要证明的那条在基线红集里就停下交回；被测代码里有 `debug_assert` 的，debug 下先红的可能是它，照实记红在哪一条，要测试断言自己红就在 `--release` 下再跑一次。`crates/mutations.tsv` 在的话（门禁 59 号），每条再加一行变异，原文在文件里恰好命中一次。
-4. 跑 `nice -n 19 bash .claude/scripts/check.sh`，贴末尾原样输出；再跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）。
+4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `mutations-append.tsv`，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --check`、`check.sh` 那一套 lint 下的 `cargo clippy`、`cargo build --offline --all-targets`，各贴末尾原样输出。全量 `cargo test --all`、层 0 各流的快档与全量、门禁阶段都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。交回前对主工作区现状 `git apply --check` 一次，打不上就在副本里按现状重做再交。
 5. 要改的文件在输入给的「别的会话正在改」清单里：不碰那个文件，也不做只能落在它上面的条款，报告写明卡在哪个文件、哪条条款；新文件要靠清单里的文件才进编译（模块声明、`Cargo.toml` 的 `[[test]]`）的，那条整条停下，不写那个新文件；其余照做。`crates/mutations.tsv` 例外：只在末尾追加整行，不改别人的行。
 6. 改动里碰到条款没写、要做设计判断的地方，停在那一处，写进报告交主 agent，不自己定。「停在那一处」的写法看这条分支走不走得到：
    - 走得到（包括要坏盘、崩溃、瞬时读错才走得到）：在**任何落盘动作之前**返回一个错误成员，名字说清是哪条条款没定、第一版不支持；写一条用例钉住「返回这个成员、盘上逐字节不变」（`DiskSnapshot` 之类比系统配置槽、根环、录制流步数），不钉它之后的行为。判定与后面的写要读同一份输入：写之前重算、对不上就不写。
@@ -39,7 +39,7 @@ omitClaudeMd: true
 
 ## 产出
 
-- 报告：这一轮写过的文件清单（`crates/mutations.tsv` 另写追加了哪几行的变异名；你自己列；别的会话同时在改 `crates/` 时，`git diff --stat` 分不出谁改的），再附 `git diff --stat -- crates litmus` 原样；每条新测试的「改坏哪一行 → 哪条断言红」、`check.sh` 结果、停下交主 agent 的设计问题。
+- 报告：这一轮写过的文件清单（`crates/mutations.tsv` 另写追加了哪几行的变异名；你自己列；别的会话同时在改 `crates/` 时，`git diff --stat` 分不出谁改的），再附 `git diff --stat -- crates litmus` 原样；每条新测试的「改坏哪一行 → 哪条断言红」、第 4 步那几样的末尾原样输出、停下交主 agent 的设计问题。
 
 ## 没做什么（固定会有的）
 
diff --git a/.claude/agents/mutation-triage.md b/.claude/agents/mutation-triage.md
index 7ca7419..d755ae9 100644
--- a/.claude/agents/mutation-triage.md
+++ b/.claude/agents/mutation-triage.md
@@ -22,7 +22,7 @@ omitClaudeMd: true
 
 1. 开跑前照共用约束「不做」一节看负载。
 2. 先逐条做子串计数：原文在源文件里不是恰好一次的，列出来（第七类），这张表不跑。
-3. 在 `research/` 下跑：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（路径相对 `research/`；报「基线就是红的」时先查是不是在仓根下跑）；表头写着「被测装置是 shell 探针」的，照表头写的复跑方式逐条跑，判据用表头那句；crates 那张表直接跑 `GATE_MUTATION_TARGET_DIR=<草稿目录>/target nice -n 19 bash .claude/gate.d/59-crates-mutation-replay.sh`（它自己拷副本、整张表逐条跑，从输出里取主 agent 点名的条目；target 放你的草稿目录，不用它默认那个跨轮共用的）。
+3. 在 `research/` 下跑：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（路径相对 `research/`；报「基线就是红的」时先查是不是在仓根下跑）；表头写着「被测装置是 shell 探针」的，照表头写的复跑方式逐条跑，判据用表头那句；crates 那张表你不跑：整表复跑是重型测试，只在提交时由崩溃验证员跑门禁 59 号（`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」，`heavy-test-guard.sh` 会拒绝你跑它）；主 agent 给你那一次 59 号的输出路径，你从里面取点名的条目分类。
 4. 确认整张表跑完：`mutate.sh` 收尾要有「已还原，基线仍全绿」；59 号的做法没有这句，要表里每一条都有一行 ✓ 或列进「有变异没红」，编不过的列在「没跑到」里、按无效计；对不上就是中途退出，这一次的数不算，照实报。
 5. 报抓到 / 无效 / 没红三个数；与改动前的数比，「无效」变多要单列。
 6. 没红与无效的逐条按 `mutation-sampling.md` 分类；判「取样点不敏感」的，解出让两个式子跨过整数边界的那个输入；判「等价」的，写出在所有输入上同值的理由。
diff --git a/.claude/main-agent.md b/.claude/main-agent.md
index b2a98d7..dba8b5a 100644
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -10,32 +10,40 @@
 - **崩溃点测试不衡量时间成本，也不为省时间缩范围**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，也不用先算它值不值。
 - **禁止自行扩大任务范围，只改自己的部分**。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
 - **测试结果与预期不符，禁止立刻直接修改方向和结论，先检查代码中有没有bug。**
-- 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动，发消息协商。
+
+- **禁止在subagent中跑重型测试（层 0 全量、QEMU、herd7、`crates` 变异整表、全量测试、整轮门禁）等**。
 
 ## 一轮怎么开、怎么收
 
 1. **开工前写死这一轮的课题与出口**：要关哪几项、做到什么算完（形态照实验的岔路单）。
 2. **冒出新点先判阻塞**：只问一句——**这个决策能不能留到下次开？它挡不挡着本轮的课题？** 挡着就现在修；不挡就记进该记的地方（里程碑收口表、`.claude/kb/checks-owed.md`、决策正文），往后延。判据不是花了多少 token、跑了多久。
-3. **派一个 agent 之前说清它关的是哪一条已经抓到的问题**；说不出来的，那条该进记录、不该进本轮。
-4. **本轮出结论之后回到那张表再判一次**：延下来的哪些现在开、哪些继续留、哪些不做，逐条写去向。不做也要写明依据。
-5. **收拢要定期做**：把开着的线收进一张表，逐条判「本轮关 / 下轮开 / 不做」，一条都不许无声消失。
+3. **重型测试**（层 0 全量、QEMU、herd7、`crates` 变异整表、全量测试、整轮门禁）提交之外任务确实要跑，先弹窗问用户，同意了才跑（`.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」）。
+4. **派实现员之前先列出它要动的 `crates/` 文件**，与在跑的实现员要动的文件有交集，就排在那一个后面，或并给同一个实现员做，不并行改同一批文件。
+5. **派一个 agent 之前说清它关的是哪一条已经抓到的问题**；说不出来的，那条该进记录、不该进本轮。
+6. **本轮出结论之后回到那张表再判一次**：延下来的哪些现在开、哪些继续留、哪些不做，逐条写去向。不做也要写明依据。
+7. **收拢要定期做**：把开着的线收进一张表，逐条判「本轮关 / 下轮开 / 不做」，一条都不许无声消失。
+8. 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动，发消息协商。
 
 ## 派出去之后
 
-盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 只负责检出，不拦、不停任务和脚本。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨，或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
+盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的（前台无超时等待循环、把活放出追踪、子 agent 跑重型测试、写撇号类角标、覆盖未跟踪文件）在执行前拒绝，不碰已经在跑的东西。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨、子 agent 从派发起的运行时间跨过一个整小时（每个整点报一次，那个整点之后主 agent 给它发过消息就不报；一格多长在 `watch.conf` 的 `ask-every-minutes`），或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
+
+改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息：写明改了哪一条、它手上哪一步要停掉或改做——它们沿用派发那一刻的定义，看不到后来的改动。
 
-子 agent 等长活时不续提示缓存，主 agent 也不定时 ping 它。临时派不读共用约束的 agent（general-purpose 这类）干带长等待的活时，派发提示里写上共用约束「长活可以等」那一条里后台命令怎么写。
+子 agent 等长活时不续提示缓存；主 agent 除了看门狗报「跑满 N 小时」时的那一条例行询问，不另外定时 ping 它。临时派不读共用约束的 agent（general-purpose 这类）干带长等待的活时，派发提示里写上共用约束「长活可以等」那一条里后台命令怎么写。
 
-叫醒之后主 agent 判断：只是慢就再起一个看门狗接着盯；是问题就决定发消息让它改、停掉重派、还是报给用户。主 agent 自己起的长命令同样放后台，命令照共用约束「长活可以等」那一条写（不在里面再把活放到后台），由看门狗盯着进程：没有子 agent 在跑时用 `bash research/scripts/watch.sh --processes`。
+叫醒之后主 agent 判断：只是慢就再起一个看门狗接着盯；是问题就决定发消息让它改、停掉重派、还是报给用户。收到「跑满 N 小时要主 agent 问一次」就给它发一条例行询问：在做什么、还差几步、在等哪个进程、预计多久，等的已经结束就接着做或交回，并写明把回答写进它草稿目录的 `progress.md`（子 agent 多半没有 SendMessage）；它结束这一轮之后读那份文件，没写就读它会话记录里最后一段正文；发完再起看门狗。主 agent 自己起的长命令同样放后台，命令照共用约束「长活可以等」那一条写（不在里面再把活放到后台），由看门狗盯着进程：没有子 agent 在跑时用 `bash research/scripts/watch.sh --processes`。
 
 ## 交回怎么读
 
 判决与写回只引产物与代码，agent 的结论句一律当线索：以仓里的产物为准，现查过再写进 kb。
 
-交回按子 agent 的 SubagentHandback 消息判。后台派的子 agent 每结束一轮，harness 发一条 status 为 completed 的通知；结果写「This agent has not reported yet: it is waiting on its own background work」、或 note 写「the result below may be interim」的，是它结束本轮去等自己起的后台任务：不当交回、不当出错，照旧等它的交回消息与看门狗。status 为 completed、没有交回消息、note 与 result 里也没有这两句的，是它结束本轮时手里没有还在跑的后台任务，不会自己醒（看门狗报「结束本轮却不会醒」）：看它最后在等什么，发消息让它接着做或交回，或者停掉。主 agent 用 TaskStop 停掉的子 agent 不再交回，它的看门狗按被停退出。
+交回按子 agent 的 SubagentHandback 消息判。**交回之后不再给它发消息**（续做闸 `.claude/hooks/continuation-guard.sh` 拒绝）：要补的活新派一个同类 agent，派发提示指到它的报告与产物；要它会话里的进度，用 `research/scripts/agent-handover.py` 抽交接摘要给新 agent 读。还没交回、在干活或在等自己后台任务的，照旧能收整点询问与纠正。后台派的子 agent 每结束一轮，harness 发一条 status 为 completed 的通知；结果写「This agent has not reported yet: it is waiting on its own background work」、或 note 写「the result below may be interim」的，是它结束本轮去等自己起的后台任务：不当交回、不当出错，照旧等它的交回消息与看门狗。status 为 completed、没有交回消息、note 与 result 里也没有这两句的，是它结束本轮时手里没有还在跑的后台任务，不会自己醒（看门狗报「结束本轮却不会醒」）：看它最后在等什么，发消息让它接着做或交回，或者停掉。主 agent 用 TaskStop 停掉的子 agent 不再交回，它的看门狗按被停退出。
 
 ## 派发提示怎么写
 
+同时在跑的重活（编译、测试、变异、产物、原型扫描）不止一个时，每份派发提示写一行「线程上限：N」，N = 整机核数 ÷ 同时在跑的重活数（向下取整，至少 1）；重活数变了，给在跑的 agent 发消息改 N。
+
 在不影响正确性的前提下写简练：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉。事实、路径、行号、数与判据一个不少，嫌长就指到文件，不删内容。
 
 ## 什么时候派哪个 agent
@@ -43,11 +51,11 @@
 | 什么时候 | 派谁、按什么次序 |
 |---|---|
 | 要推论：设计判断、取舍，拿依据（包括实验结论）去推翻或确立决策，实现改动的对抗 | 主 agent 写 `research/prompts/_<轮>-body.md` → `three-way-materials` → 同一条消息并行派 `three-way-forward` 或 `three-way-defense`（一轮一条）、`three-way-attack`、`three-way-local-attack` 或 `three-way-local-defense`（一轮一条）→ 全部交齐后，有腿交了模型或产物就派 `three-way-verifier` → 主 agent 写判决；三轮之后停 |
-| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ `crash-verifier`（层 0、QEMU、herd7、crates 变异表） |
-| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ `crash-verifier` |
+| 改 `crates/`：条款已定、验收标准写得清、改动有界（照已定分项实现、补测试与变异、修原因已知的红） | `implementation-writer`（后台派，主 agent 审 diff）→ 上一行的三方（代码轮）→ 提交时派 `crash-verifier` 跑层 0、QEMU、herd7、crates 变异表（命令带 `SINGLEFS_HEAVY_TESTS=commit`，见「暂存之后、提交之前跑门禁」那一行；平时不跑） |
+| 改 `crates/`：探索性的、条款没写全、要用户边看边拍板 | 主 agent 自己写、直接和用户对话 → 上一行的三方（代码轮）→ 提交时 `crash-verifier` |
 | 跑变异表、看抓到 / 无效 / 没红 | `mutation-triage` → 要补取样点、补断言的：`crates/` 那侧 `implementation-writer`（输入给分诊报告），`research/` 那侧 `experiment-designer` 写重跑登记 → `experiment-runner` |
 | 一个阶段任务结束：里程碑一步、一轮判决、一段实验、一批定义或脚本改完，这一批交回到齐、暂存之前 | `sweep` 写事实表（阶段同步第一段；输入给改动范围与做成的事，只用过、没留改动的也写；交回的事实表过了 `research/scripts/stale-candidates.py --check-facts`）→ 候选表按组切段，每段不超过约 200 行，一段派一个 `sweep` 逐行判（第二段；输入同样给做成的事）→ 交回齐了主 agent 跑 `stale-candidates.py --check-report 候选表 各份报告`，退出码 0 才往下，再**逐行全看**：每一条判定都对着载体今天的原文核一遍，不抽样、不按判定种类挑。判错的那一段重派，重派交回照样全看 → 主 agent 逐处判，自己改或交 `kb-scribe` → 主 agent 写 `research/prompts/<阶段>-sync.md`（格式见门禁 68 号文件头，68 号判形式）→ 下一行 |
-| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，就在 HEAD + 暂存区的 worktree 里后台跑那棵树里的 `bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记，整轮门禁的 54 号只核标记）→ `gate-triage` |
+| 暂存之后、提交之前跑门禁 | 主 agent 用 `research/scripts/stage-mine.py` 暂存 → 派 `crash-verifier` 跑它那几道，命令带 `SINGLEFS_HEAVY_TESTS=commit`：这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入，就在 HEAD + 暂存区的 worktree 里跑那棵树里的 `SINGLEFS_HEAVY_TESTS=commit bash .claude/gate.d/54-layer0-replay.sh --full <它的根>`（层 0 全量只在这一步跑，判绿按输入哈希写全绿标记，整轮门禁的 54 号只核标记），55、57、59 同样带前缀 → `gate-triage` 带 `SINGLEFS_HEAVY_TESTS=commit` 跑 `gate.sh --staged` 并分诊（54、55、57、59 在 `gate.sh` 里照各自的复用判定走，它不直接调）；用户要求时两处都换成 `=user-request` |
 | 撤回一个数、改格式常量、新立一条判据 | `sweep` |
 | 定案之后写回 kb | `kb-scribe`（主 agent 给逐条规格）；规格动了某条分项的「**依据**」段时，同一份规格要带上那个实验页 `### 影响的决策` 表里对应那几行（门禁 75 号那条双向检查两侧要同时到位，而判一个实验撑不撑一条分项是主 agent 的活）；它翻了分项状态、33 号红（变异表锚点腐化）→ `experiment-runner` 只修锚点 |
 | 要建计数实验，或重跑已有实验 | 主 agent 写岔路单（`.claude/rules/three-way-inference.md`「交岔路时写岔路单」；不是为岔路建的实验写同格式的问题单）→ `experiment-designer` 写跑前登记或重跑登记 → 主 agent 删掉登记里待删的问法（有的话）→ `experiment-runner` → 每段交回主 agent 对着岔路单判够不够：一行开着的都不剩就停、交岔路表；续派时派发提示点名还差的那几行（续派闸 `.claude/hooks/runner-dispatch-guard.sh` 查这一句） |
diff --git a/.claude/rules/implementation-workflow.md b/.claude/rules/implementation-workflow.md
index bf1a7fe..4e1485c 100644
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -43,7 +43,19 @@
 
 ⚠️ **不许拿「我只改了文档」当理由。** 「我只动了一条」是需要被证明的断言，不是事实（`.claude/rules/fs-design.md`「门禁的范围必须可判定」）。
 
-## 提交前必跑 herd7 与 QEMU
+## 重型测试只在提交时跑
+
+**重型测试**：层 0 全量（`.claude/gate.d/54-layer0-replay.sh` 与 `--test` 目标名含 `layer0` 的测试）、QEMU（55 号、`vm-bench.sh`）、herd7（57 号、`.claude/scripts/lkmm.sh`）、`crates` 变异整表（59 号）、全量 `cargo test`（`--all`、`--workspace`、`check.sh`）、整轮门禁（`gate.sh`、`gate-staged.sh`）、全部实验复跑（87 号）、E152 装置。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。
+
+| 场合 | 跑不跑 |
+|---|---|
+| 每次提交代码 | **必须跑**：主 agent 在提交流程里后台起（命令带 `SINGLEFS_HEAVY_TESTS=commit`），看门狗盯；git 的 pre-commit hook 照旧跑整轮门禁 |
+| 用户当场要求，或任务确实要跑 | 任务确实要跑时主 agent 先弹窗问用户，用户同意了才跑；命令带 `SINGLEFS_HEAVY_TESTS=user-request` |
+| 其余任何时候 | 不跑 |
+| 崩溃验证员、门禁分诊员 | 各跑各的那一部分，只在提交时或用户要求时（命令带 `SINGLEFS_HEAVY_TESTS=commit` 或 `=user-request`）：崩溃验证员跑 54、55、57、59 号那几道；门禁分诊员跑门禁其余阶段并分诊，54、55、57、59 靠「输入没变就复用上一次全绿判定」不重跑。谁都不把整轮全量从头跑一遍 |
+| 其余子 agent | **一律不跑**，只跑自己动到的测试二进制与 fmt / clippy / build |
+
+由 `.claude/hooks/heavy-test-guard.sh`（Bash 的 PreToolUse）在执行前拒绝：其余子 agent 跑上面任何一样拒绝；崩溃验证员、门禁分诊员跑自己那一部分之外的、或不带那个环境变量前缀的拒绝；主 agent 不带那个前缀也拒绝。派发提示要子 agent 跑重型测试的，由 `.claude/hooks/runner-dispatch-guard.sh` 拒绝。
 
 herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样本、模板与规则段落都不在 SOP 里，怎么测、怎么验、接不接进门禁由本工程自己定。两样都是本工程自己的阶段：
 
@@ -52,7 +64,7 @@ herd7 / LKMM 与 QEMU 不归上游 singlefs-ai-sop 管：相关的脚本、样
 | herd7 / LKMM | `.claude/gate.d/57-lkmm.sh`（逻辑在 `.claude/scripts/lkmm.sh`） | `litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符；缺 herd7 直接红，不静默跳过 |
 | QEMU 真设备 | `.claude/gate.d/55-qemu-first-transaction.sh` | 两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比 |
 
-两道都在 `gate.sh` 里，提交前跑全量门禁就把它们带上了；单跑一道不算跑过门禁。
+两道都在 `gate.sh` 里，提交时跑整轮门禁就把它们带上了；单跑一道不算跑过门禁。
 `--staged` 那条路（几个会话共写一个仓时）同样跑它们。
 
 ## 测试与崩溃检测优先多线程
```
