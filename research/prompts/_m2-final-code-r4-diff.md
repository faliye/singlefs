# 附录二：第三轮冻结树到第四轮冻结树的 crates 改动，加第三轮之后的定义/检测器 diff 与两道门禁相对 HEAD 的 diff（生成于 2026-09-25 10:35 JST / 2026-09-25 01:35 UTC）

基准：第三轮冻结副本 `/tmp/claude-1000/m2-final-code-r3/tree/crates/` 到本轮冻结副本 `/tmp/claude-1000/m2-final-code-r4/tree/crates/`（腿读代码一律读本轮冻结副本 `/tmp/claude-1000/m2-final-code-r4/tree/crates/`，不读主工作区——主工作区此刻有 E156、E158 两个执行员在改各自的装置，E142 执行员随后要新建一个导出二进制）。

diff 原始文件（三份，均由主 agent 给出，材料员未重新生成、原样落盘）：
- `/tmp/claude-1000/m2-final-code-r4/r3-to-r4.diff`（`diff -ruN -x target` 生成，23 个文件：`crates/mutations.tsv` 1 份、`src/*.rs` 9 份、`tests/*.rs` 13 份，共 3900 行）；
- `/tmp/claude-1000/m2-final-code-r4/defs-r3-to-r4.diff`（`diff -u` 逐文件生成，3 个文件：`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/hooks/bash-command-detector.sh`，共 669 行）；
- `/tmp/claude-1000/m2-final-code-r4/gates-vs-head.diff`（`git diff` 生成，2 个文件：`.claude/gate.d/55-qemu-first-transaction.sh`、`.claude/gate.d/74-model-differential.sh`，共 238 行）。

太长的一半按派发提示的口径截断：`tests/` 下的 13 份只列文件名与增删行数，不抄全文；`crates/mutations.tsv` 与 `src/*.rs` 共 10 份的改动整段抄进第一节。第一节按 `r3-to-r4.diff` 原有的按文件分段格式，逐段原样节选（含两侧路径与时间戳），未改一字。

## 一、diff（`crates/mutations.tsv` 与 `src/*.rs`，共 10 个文件，第三轮冻结树到本轮冻结树，原样节选自 `r3-to-r4.diff`）

```diff
diff -ruN -x target /tmp/claude-1000/m2-final-code-r3/tree/crates/mutations.tsv tree/crates/mutations.tsv
--- /tmp/claude-1000/m2-final-code-r3/tree/crates/mutations.tsv	2026-09-24 22:40:06.357335347 +0000
+++ tree/crates/mutations.tsv	2026-09-25 00:43:55.241777657 +0000
@@ -27,7 +27,7 @@
 步 4：回退行不带回退位	crates/singlefs-core/src/mount.rs	        applied_transaction_high_water: 0,\n        is_rollback: true,	        applied_transaction_high_water: 0,\n        is_rollback: false,	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
 步 4：只写被退回的实例那一行、不写中间实例行	crates/singlefs-core/src/mount.rs	    (first_row_instance..instance_to_acquire.0)	    (first_row_instance..first_row_instance + 1)	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
 步 4：回退之后 jsn 接 R_old 那条之后（C340 的 P1，盖掉被抛弃发布的记录槽）	crates/singlefs-core/src/mount.rs	    let next_counter = highest_counter + 1;	    let next_counter = own_record.counter + 1;	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_keeps_the_abandoned_records	rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation
-步 4：影子账一个槽都不隔离	crates/singlefs-core/src/mount.rs	            allocator.isolate_abandoned(device, slot, span_slots);	            let _ = (device, slot, span_slots);	-p singlefs-harness --test second_transaction_step_four_rollback -- without_the_shadow_ledger	without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit
+步 4：影子账一个槽都不隔离	crates/singlefs-core/src/mount.rs	            allocator.isolate_abandoned(device, slot, span_slots);	            let _ = (device, slot, span_slots);	-p singlefs-harness --test second_transaction_step_four_rollback -- without_the_shadow_ledger	without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_the_abandoned_root_reads_a_torn_unit_while_the_witness_keeps_recovery_off_it
 步 4：回退候选集不按实例表判（被抛弃时间线的根也能退到）	crates/singlefs-core/src/mount.rs	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg)	        .any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg && false)	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_onto_an_abandoned	rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused
 步 4：前缀第五条不判（回退行的 W 不封顶）	crates/singlefs-core/src/recovery.rs	            if high_water == 0 || record.transaction > high_water {	            if false && (high_water == 0 || record.transaction > high_water) {	-p singlefs-harness --test second_transaction_step_four_rollback -- the_rollback_row_caps	the_rollback_row_caps_the_prefix_of_the_chosen_roots_instance_at_its_high_water
 步 4：checker 的 I-3.1 并集不按实例表排除被抛弃的根	crates/singlefs-checker/src/walk.rs	            let walked = *index == newest_index || (!abandoned && !below_floor);	            let walked = *index == newest_index || !below_floor;	-p singlefs-harness --test second_transaction_step_four_rollback -- rolling_back_to_the_first_root	rolling_back_to_the_first_root_writes_the_rollback_row_and_the_intermediate_row_and_cold_start_reads_the_first_content
@@ -85,7 +85,7 @@
 增补 2：提交内生块段耗尽不回落（C369）	crates/singlefs-core/src/allocator.rs	            None => self\n                .lowest_commit_generated_fallback_slot(footprint)\n                .map(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot),	            None => None,	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- publish_on_pool_without	publish_on_pool_without_empty_cluster_segment_falls_back_to_lowest_free_slot_on_every_device
 增补 2：提交内生块段耗尽不回落（抬 F 的空发布在唯一全空段被扣住时又报 NoSpaceFor）	crates/singlefs-core/src/allocator.rs	            None => self\n                .lowest_commit_generated_fallback_slot(footprint)\n                .map(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot),	            None => None,	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- raising_the_floor_when_the_only	raising_the_floor_when_the_only_empty_segment_is_held_falls_back_to_slots_outside_the_hold
 增补 2：回落只看已分配位（发出影子账隔离的槽）	crates/singlefs-core/src/allocator.rs	                .all(|slot| !self.is_blocked_for_commit_generated(SlotNumber(slot)));	                .all(|slot| !self.allocated[Self::index(SlotNumber(slot))]);	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- fallback_skips_a_slot_isolated	fallback_skips_a_slot_isolated_by_the_shadow_ledger
-增补 2：回落只看已分配位（抬 F 的空发布发出扣住的槽）	crates/singlefs-core/src/allocator.rs	                .all(|slot| !self.is_blocked_for_commit_generated(SlotNumber(slot)));	                .all(|slot| !self.allocated[Self::index(SlotNumber(slot))]);	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- raising_the_floor_with_no_free_slot	raising_the_floor_with_no_free_slot_outside_the_hold_still_fails_and_the_hold_stays_in_the_process
+增补 2：回落只看已分配位（抬 F 的空发布发出扣住的槽）	crates/singlefs-core/src/allocator.rs	                .all(|slot| !self.is_blocked_for_commit_generated(SlotNumber(slot)));	                .all(|slot| !self.allocated[Self::index(SlotNumber(slot))]);	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- raising_the_floor_with_no_free_slot	raising_the_floor_with_no_free_slot_outside_the_hold_fails_before_any_write_and_hands_the_allocator_back_exactly_as_before_the_raise
 步 0：崩溃镜像的记录提示漏掉基镜像里的记录（种进基镜像的残留记录恢复扫不到）	crates/singlefs-harness/src/crash.rs	        let mut offsets = self\n            .base\n            .journal_record_offsets_hint(device, ring_start, ring_bytes)?;	        let mut offsets: Vec<DeviceOffsetInBytes> = Vec::new();	-p singlefs-harness --test second_transaction_step_zero_layer0 -- residual_record_seeded	residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it
 步 0：恢复把没有回退行的所选根也按 W = 0 封顶（前缀第五条误判，种进基镜像的残留记录不施加）	crates/singlefs-core/src/recovery.rs	                rollback_high_water_of_root(reader, &root),	                Some(0),	-p singlefs-harness --test second_transaction_step_zero_layer0 -- residual_record_seeded	residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it
 步 6：恢复从系统配置 tail 起逐条验证记录的点名单元、失配即中止（E78 的自我中止形态）	crates/singlefs-core/src/recovery.rs	            let records = scan_journal(reader, &system_configuration);	            let records = scan_journal(reader, &system_configuration);\n            let trusted_tail_mismatch = records\n                .values()\n                .filter(|record| record.counter > system_configuration.quantities.journal_tail)\n                .any(|record| {\n                    replay_journal(\n                        reader,\n                        &RootRecord {\n                            instance: record.instance,\n                            checkpoint_txg: CheckpointTxg(record.checkpoint_txg.0 - 1),\n                            ..root\n                        },\n                        system_configuration.immutable.sizes.journal_ring_bytes,\n                        &BTreeMap::from([((record.instance, record.counter), record.clone())]),\n                        true,\n                        None,\n                    )\n                    .0\n                    .verification_failed\n                        > 0\n                });\n            if trusted_tail_mismatch {\n                return RecoveryReport {\n                    outcome: RecoveryOutcome::Failed {\n                        root: Some(root_key),\n                        failure: RecoveryFailure::NoValidRoot,\n                    },\n                    effective_root: None,\n                    journal: JournalScanReport::default(),\n                    mapping_fallbacks,\n                };\n            }	-p singlefs-harness --test second_transaction_step_zero_layer0 -- stale_tail	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
@@ -225,7 +225,7 @@
 步 0 开关二：读者那个口子只把拦下的次数记一笔、读照样放过去（持续读不出退化成只记账）	crates/singlefs-harness/src/fault_injection.rs	self.reads_refused.set(self.reads_refused.get() + 1);\n            return None;	self.reads_refused.set(self.reads_refused.get() + 1);	-p singlefs-harness --test second_transaction_step_zero_test_only_switches -- a_named_root_ring_slot_keeps_failing	a_named_root_ring_slot_keeps_failing_every_read_not_just_the_first_one
 步 0 开关二：块设备那一侧的注入计划恒不命中点名的根环槽	crates/singlefs-harness/src/fault_injection.rs	            FaultPlacement::WithinNamedRootRingSlots(target) => {\n                target.slot_covering(device, offset).is_some()\n            }	            FaultPlacement::WithinNamedRootRingSlots(target) => {\n                let _ = target;\n                false\n            }	-p singlefs-harness --test second_transaction_step_zero_test_only_switches -- the_same_named_root_ring_slots_drive	the_same_named_root_ring_slots_drive_the_shared_fault_plan_on_the_block_device_side
 步 0 开关二：点名位图按「区域 + 槽」算位号（不同的 (区域, 槽) 撞进同一位）	crates/singlefs-harness/src/fault_injection.rs	slot.region * ROOT_RING_SLOTS_PER_REGION_MAXIMUM + slot.slot	slot.region + slot.slot	-p singlefs-harness --test second_transaction_step_zero_test_only_switches -- named_root_ring_slots_name_exactly	named_root_ring_slots_name_exactly_the_pairs_they_were_given
-必红八条第八条（影子账关掉）的 ② 格：点名一组槽时只留最后一个，回退实例四条根里三条照样读得出（那一格根本没造出来）	crates/singlefs-harness/src/fault_injection.rs	.fold(NamedRootRingSlots::NONE, |named, slot| named.with(*slot))	.fold(NamedRootRingSlots::NONE, |_named, slot| NamedRootRingSlots::NONE.with(*slot))	-p singlefs-harness --test second_transaction_step_four_rollback -- with_the_shadow_ledger_off_and_every_root	with_the_shadow_ledger_off_and_every_root_of_the_rollback_instance_unreadable_the_recovery_falls_back_onto_the_abandoned_root_and_reads_a_torn_unit
+必红八条第八条（影子账关掉）的 ② 格：点名一组槽时只留最后一个，回退实例四条根里三条照样读得出（那一格根本没造出来）	crates/singlefs-harness/src/fault_injection.rs	.fold(NamedRootRingSlots::NONE, |named, slot| named.with(*slot))	.fold(NamedRootRingSlots::NONE, |_named, slot| NamedRootRingSlots::NONE.with(*slot))	-p singlefs-harness --test second_transaction_step_four_rollback -- with_the_shadow_ledger_off_and_every_root	with_the_shadow_ledger_off_and_every_root_of_the_rollback_instance_unreadable_the_witness_keeps_recovery_on_the_rollback_target_and_the_abandoned_root_reads_a_torn_unit
 并行线二：读路径自报的设备级读数不加（实现说这次一个设备读都没发，装置在块层照样数到那几次，D17 已定项 5 射程 ① 的对账等于没有）	crates/singlefs-core/src/mounted_read.rs	        observation.device_reads_issued += 1;	        observation.device_reads_issued += 0;	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- random_four_kibibyte_reads	random_four_kibibyte_reads_return_the_written_bytes_and_never_scan_the_journal_ring
 并行线二：这次读了几个单元的计数不加（验收第 4 条那个观测点恒 0，跨单元的页也不报 2）	crates/singlefs-core/src/mounted_read.rs	            observation.data_units_read += 1;	            observation.data_units_read += 0;	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- every_aligned_page	every_aligned_page_reads_back_and_the_pages_that_cross_a_unit_boundary_read_two_units
 并行线二：一次读只解引用第一个单元（跨单元的页少拷后半页，D4 已定项 5 那一成多的页整档漏掉）	crates/singlefs-core/src/mounted_read.rs	        for unit_number in span.first.0..=span.last_inclusive.0 {	        for unit_number in span.first.0..=span.first.0 {	-p singlefs-harness --test second_transaction_parallel_line_two_mounted_read -- corrupting_one_byte	corrupting_one_byte_in_a_data_unit_makes_reads_over_it_report_a_checksum_error_and_leaves_the_other_reads_alone
@@ -248,7 +248,7 @@
 mkfs 清环漏掉最后 1 MiB（并行线四验收的第一条变异）	crates/singlefs-core/src/make_filesystem.rs	device.write_zeroes_at(journal_ring_start, parameters.geometry.journal_ring_bytes)?;	device.write_zeroes_at(journal_ring_start, parameters.geometry.journal_ring_bytes - 1024 * 1024)?;	-p singlefs-harness --test first_transaction_step_one_mkfs	mkfs_zeroes_the_whole_journal_ring_and_writes_the_registered_number_of_bytes
 分段标记少打一段不再被拒收：登记对账从逐项相等放宽成「不超过登记的条数」（并行线四验收的第二条变异）	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	let matches = marked == registered;	let matches = marked.len() <= registered.len();	-p singlefs-harness --bin first_transaction_on_device	segment_timing_registration_refuses_a_run_that_marked_one_segment_fewer
 设备侧日志把清零段折回一步时不查这一条是不是全 0（段里夹一条带内容的写也照折，虚机档那道逐项比对失去判别力）	crates/singlefs-harness/src/device_log.rs	if offset.0 != cursor || *content_hash != fnv1a_64_of_zeros(*length) {	if offset.0 != cursor {	-p singlefs-harness --lib	device_log::tests::a_zero_fill_folds_back_into_one_event_however_the_block_layer_split_it
-影子账两臂报同一个分支名（运行时看不出走了哪一条，五条硬要求第 4 条）	crates/singlefs-core/src/mount.rs	            ShadowLedger::On => "shadow_ledger=on",\n            ShadowLedger::Off => "shadow_ledger=off",	            ShadowLedger::On | ShadowLedger::Off => "shadow_ledger",	-p singlefs-harness --test second_transaction_step_four_rollback	without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_a_recovery_onto_the_abandoned_root_reads_a_torn_unit
+影子账两臂报同一个分支名（运行时看不出走了哪一条，五条硬要求第 4 条）	crates/singlefs-core/src/mount.rs	            ShadowLedger::On => "shadow_ledger=on",\n            ShadowLedger::Off => "shadow_ledger=off",	            ShadowLedger::On | ShadowLedger::Off => "shadow_ledger",	-p singlefs-harness --test second_transaction_step_four_rollback	without_the_shadow_ledger_publishes_after_the_rollback_reuse_the_abandoned_data_slots_and_the_abandoned_root_reads_a_torn_unit_while_the_witness_keeps_recovery_off_it
 mkfs 名义上清根环、实际一个字节都不清（C484 的正题变异：同 fsid 在用过的盘上重做 mkfs）	crates/singlefs-core/src/make_filesystem.rs	device.write_zeroes_at(region_start(region_to_clear), root_ring_region_bytes)?;	device.write_zeroes_at(region_start(region_to_clear), 0)?;	-p singlefs-harness --test first_transaction_step_one_mkfs	remaking_the_pool_with_the_same_filesystem_identifier_leaves_no_root_of_the_old_pool
 mkfs 只清根环的区域 0、区域 1 与 2 不清（清零不按几何走完三段）	crates/singlefs-core/src/make_filesystem.rs	for region_to_clear in 0..ROOT_RING_REGIONS {	for region_to_clear in 0..ROOT_RING_REGIONS - 2 {	-p singlefs-harness --test first_transaction_step_one_mkfs	remaking_the_pool_with_the_same_filesystem_identifier_leaves_no_root_of_the_old_pool
 根环一个区域按 S − 1 个槽算：最后一个根槽不清，旧池留在那一槽的根活下来	crates/singlefs-core/src/root_ring.rs	slots_per_region.count() * u64::from(fixed_structure_slot_spacing)	(slots_per_region.count() - 1) * u64::from(fixed_structure_slot_spacing)	-p singlefs-harness --test first_transaction_step_one_mkfs	remaking_the_pool_with_the_same_filesystem_identifier_leaves_no_root_of_the_old_pool
@@ -304,7 +304,7 @@
 步 3 验收：取号之后没有屏障（取号的系统配置槽写与写行那次发布的单元写并成一段）	crates/singlefs-core/src/transaction.rs	    if let Err(cause) = pool.perform(CommitStep::Barrier) {\n        return Err(pool.roll_back_acquisition(&written, previous_instance, cause));\n    }\n    Ok(instance)	    Ok(instance)	-p singlefs-harness --test second_transaction_step_three_acquisition_barrier_layer0 -- no_unit_of_the_new_instance	no_unit_of_the_new_instance_persists_before_the_acquired_instance_in_any_crash_state_of_the_remount
 步 6 验收第 1 条：层 0 按发布分状态数时根槽写那一段归到下一次发布（按段归发布错一位）	crates/singlefs-harness/src/crash.rs	            let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_write_index]);\n            next_root = Some(Layer0PublishOfState::UpToTheRootOf {\n                root_write_index,\n                instance,\n                checkpoint_txg,\n            });\n        }	            publish_of_segment[segment_index] =\n                next_root.unwrap_or(Layer0PublishOfState::AfterTheLastRoot);\n            let (instance, checkpoint_txg) = root_identity_of_write(&writes[root_write_index]);\n            next_root = Some(Layer0PublishOfState::UpToTheRootOf {\n                root_write_index,\n                instance,\n                checkpoint_txg,\n            });\n            continue;\n        }	-p singlefs-harness --test second_transaction_step_zero_layer0 -- every_crash_state_outside_the_two_unit_segments	every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims
 C481：坏盘输入的基线抽样里去掉「树表 0 条」那一档（报出的基线档数由 2 变 1）	crates/singlefs-harness/src/bad_disk_input.rs	            BaseImageTier::TreeTableWithoutEntries => newest_root_tree_table_has_no_entries(image),	            BaseImageTier::TreeTableWithoutEntries => false,	-p singlefs-harness --test second_transaction_supplement_three_bad_disk_input -- bad_disk_inputs_never_read_back	bad_disk_inputs_never_read_back_an_uncommitted_version_and_panic_only_at_known_sites
-C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷）	crates/singlefs-core/src/mount.rs	    establish_instance(\n        parameters,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            tree_identifier_watermark_of_the_ring,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger: ShadowLedger::On,\n            system_configuration,\n        },\n    )\n}\n	    let instance_before_acquisition = previous_row.instance;\n    let outcome = establish_instance(\n        parameters,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            tree_identifier_watermark_of_the_ring,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger: ShadowLedger::On,\n            system_configuration,\n        },\n    );\n    if let Err(MountError::Publish(_)) = &outcome {\n        let mut rollback_writer = PoolWriter::new(parameters, devices.as_mut_slice());\n        let _ = rollback_writer.perform(\n            crate::transaction::CommitStep::RotateSystemConfigurationSlots {\n                journal_tail: 0,\n                journal_instance: instance_before_acquisition,\n            },\n        );\n    }\n    outcome\n}\n	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- a_write_error_after_the_acquisition	a_write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it
+C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷）	crates/singlefs-core/src/mount.rs	    establish_instance(\n        parameters,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            tree_identifier_watermark_of_the_ring,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger: ShadowLedger::On,\n            system_configuration,\n            rows_of_the_chosen_roots_instance_table,\n            space_admission,\n        },\n    )\n}\n	    let instance_before_acquisition = previous_row.instance;\n    let outcome = establish_instance(\n        parameters,\n        devices,\n        allocator,\n        InstanceStart {\n            chosen_root,\n            effective_root,\n            journal,\n            previous,\n            previous_row,\n            first_txg,\n            next_counter,\n            tree_identifier_watermark_of_the_ring,\n            effective_floor,\n            abandoned_roots_unreadable,\n            shadow_ledger: ShadowLedger::On,\n            system_configuration,\n            rows_of_the_chosen_roots_instance_table,\n            space_admission,\n        },\n    );\n    if let Err(MountError::Publish(_)) = &outcome {\n        let mut rollback_writer = PoolWriter::new(parameters, devices.as_mut_slice());\n        let _ = rollback_writer.perform(\n            crate::transaction::CommitStep::RotateSystemConfigurationSlots {\n                journal_tail: 0,\n                journal_instance: instance_before_acquisition,\n            },\n        );\n    }\n    outcome\n}\n	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- a_write_error_after_the_acquisition	a_write_error_after_the_acquisition_leaves_the_new_instance_in_the_system_configuration_and_the_report_names_it
 增补 2 收口第 44 行：I-3.10 的比较恒成立（未释放记录的分配代与单元头里的诞生代号不比）	crates/singlefs-checker/src/walk.rs	            judgements.judge("I-3.10", record.generation == birth_txg, || {	            judgements.judge("I-3.10", true || record.generation == birth_txg, || {	-p singlefs-harness --test checker_known_bad_images -- an_unreleased_allocation_record	an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant
 增补 2 收口第 44 行：I-3.10 射程 ① 放宽（已释放的记录也拿释放代去比单元头的诞生代号）	crates/singlefs-checker/src/walk.rs	            if record.is_released\n                || !examined_records.insert(	            if false\n                || !examined_records.insert(	-p singlefs-harness --test checker_known_bad_images -- an_unreleased_allocation_record	an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant
 增补 2 收口第 44 行：I-3.10 射程 ② 改读末槽（跨两槽的记录不再取起点槽那个单元头）	crates/singlefs-checker/src/walk.rs	                    record.slot * SLOT_BYTES,\n                    UNIT_HEADER_SCAN_BYTES,	                    (record.slot + record.span_slots - 1) * SLOT_BYTES,\n                    UNIT_HEADER_SCAN_BYTES,	-p singlefs-harness --test checker_known_bad_images -- an_unreleased_allocation_record	an_unreleased_allocation_record_that_kept_the_previous_generation_reddens_only_the_allocation_generation_invariant
@@ -406,7 +406,7 @@
 C394（释放判定不核映射条目位置项里的单元校验和）：释放之前读盘核校验和那一核拿掉（位置项的校验和比不比都当对得上），被改坏的那一份照常回到空闲池	crates/singlefs-core/src/transaction.rs	                    Some(copy) if crc32_castagnoli(&copy) != location.unit_checksum => {\n	                    Some(copy) if false && crc32_castagnoli(&copy) != location.unit_checksum => {\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_released_unit_whose_copies_fail	a_released_unit_whose_copies_fail_the_checksum_in_its_mapping_entry_is_quarantined_instead_of_returning_to_the_free_pool
 C394 N3：分配器释放时不看「留在已分配」那几块盘（对不上那一份的记录照样改写成已释放）	crates/singlefs-core/src/allocator.rs	            if devices_whose_record_stays_allocated.contains(&device.device) {\n                continue;\n            }	            let _ = devices_whose_record_stays_allocated;	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_released_unit_whose_copies_fail	a_released_unit_whose_copies_fail_the_checksum_in_its_mapping_entry_is_quarantined_instead_of_returning_to_the_free_pool
 C394 N3（用户 2026-09-24 定：对不上那一份的分配记录留在已分配）：核出对不上之后照常释放那一份（记录改写成已释放）	crates/singlefs-core/src/transaction.rs	            .filter(|copy| copy.placement == *placement)	            .filter(|copy| copy.placement == *placement && false)	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- after_rebuilding_the_previous_version	after_rebuilding_the_previous_version_from_disk_the_release_still_reads_and_quarantines_the_mismatching_copy
-C394 N1：重读还读不出时不按对不上处置、当成核得上照常释放（读不出的那一份回到空闲池）	crates/singlefs-core/src/transaction.rs	                    None => QuarantinedCopyReading::UnreadableAfterOneReread,\n	                    None => QuarantinedCopyReading::IntactButAnotherCopyFailed,\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_release_checksum_read_that_keeps_failing	a_release_checksum_read_that_keeps_failing_is_read_once_more_and_then_handled_as_a_mismatch
+C394 N1：重读还读不出时不按对不上处置、当成核得上照常释放（读不出的那一份回到空闲池）	crates/singlefs-core/src/transaction.rs	                            CopyCheck::Unreadable => {\n                                QuarantinedCopyReading::UnreadableAfterOneReread\n                            }	                            CopyCheck::Unreadable => {\n                                QuarantinedCopyReading::IntactButAnotherCopyFailed\n                            }	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_release_checksum_read_that_keeps_failing	a_release_checksum_read_that_keeps_failing_is_read_once_more_and_then_handled_as_a_mismatch
 C513（复用豁免不判那次复用合不合法）：回收谓词那一判拿掉（证得出过不了也开脱，回到只看更晚那次写落没落盘）	crates/singlefs-harness/src/crash.rs	    release_generation_at_least <= reclaim_threshold_at_most.0	    release_generation_at_least <= reclaim_threshold_at_most.0 || true	-p singlefs-harness --test second_transaction_supplement_two_record_checker_reuse_legality -- an_illegal_reuse	an_illegal_reuse_whose_later_write_landed_no_longer_excuses_the_missing_unit_in_the_record_checker
 C513：回收谓词判得过严（释放代下界取无穷大，抬 F 之后的合法复用也不开脱，增补 2 收口表第 55 行那 8 个误报回来）	crates/singlefs-harness/src/crash.rs	    let release_generation_at_least = earlier_publish_txg.0.saturating_add(1);	    let release_generation_at_least = u64::MAX;	-p singlefs-harness --test second_transaction_step_zero_layer0 -- stale_tail_with_a_reused_named_unit	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
 并行线一：C490：extent 叶记录 key 的 offset 段写成单元序号（并行线一验收第 4 条第二个变异：读回错位判红）	crates/singlefs-core/src/transaction.rs	        let extent_record = build_extent_record(\n            FIRST_INODE_NUMBER,\n            transaction.payload_start.0,	        let extent_record = build_extent_record(\n            FIRST_INODE_NUMBER,\n            transaction.unit_index_in_file.0,	-p singlefs-harness --test second_transaction_parallel_line_one_sequential_write -- the_second_extent_leaf_record_key_is_the_file_byte_offset_of_the_second_data_unit	the_second_extent_leaf_record_key_is_the_file_byte_offset_of_the_second_data_unit
@@ -483,7 +483,7 @@
 增补 3 第 3 件（用户 2026-09-20 定案第 6 条）：崩溃状态上不算「F 落在回退留下的空档里」，写死成 None（收口表第 43 行那一形恒不匹配）（接替原第 180 行：用例改名）	crates/singlefs-harness/src/crash_injection.rs	        raised_floor_lands_only_on_abandoned_roots:\n            raised_floor_of_the_newest_root_lands_only_on_abandoned_roots(image),	        raised_floor_lands_only_on_abandoned_roots: None,	-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- crash_states_after_raising_the_floor_into_the_gap	crash_states_after_raising_the_floor_into_the_gap_match_the_known_red_form_of_closeout_row_43
 增补 2 第 58 行：第一个事务失败时不交写入口的失败账（接替原第 381 行：用例改名）	crates/singlefs-harness/src/scenario.rs	        failed_step: FirstTransactionPathStep::FirstTransaction,\n        cause: format!("{error:?}"),\n        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),	        failed_step: FirstTransactionPathStep::FirstTransaction,\n        cause: format!("{error:?}"),\n        writes_of_failed_publishes: Vec::new(),	-p singlefs-harness --bin first_transaction_on_device -- first_transaction_path_failures	first_transaction_path_failures_report_every_publish_of_the_failed_step_and_they_equal_the_device_layer_count
 增补 3 第 3 件（用户 2026-09-20 定案第 6 条）：checker 在 I-3.1 的说明文字里少写一项机理标识（判读的一方就认不出机理）；已知红清单那一条（收口表第 43 行）的复现判出（接替原第 478 行：原点名的第 0 条那一形已删）	crates/singlefs-checker/src/walk.rs	、回退下界 F {newest_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"	、低于 F 的根槽 {root_slots_dropped_below_floor} 个"	-p singlefs-harness --test second_transaction_supplement_three_random_history -- raising_the_floor_into_the_gap	raising_the_floor_into_the_gap_left_by_a_rollback_ends_in_the_known_red_form_of_closeout_row_43
-增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2j）：分配器提交内生块那一处把「每块盘上都没有」报成「小盘写满」；抬 F 的空发布拿不到固定点那条写死用例判出（接替原第 166 行：单元区墙取样点换到 256 槽小盘之后只走得到用户数据那一处）	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::Agreed(answer) => answer,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);	            DeviceAgreement::Agreed(answer) => answer,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices: Vec::new() });	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- raising_the_floor_with_no_free_slot_outside_the_hold	raising_the_floor_with_no_free_slot_outside_the_hold_still_fails_and_the_hold_stays_in_the_process
+增补 3 第 2 件（代码三方第二轮判决第三节第 2 条，攻方变异 m2j）：分配器提交内生块那一处把「每块盘上都没有」报成「小盘写满」；抬 F 的空发布拿不到固定点那条写死用例判出（接替原第 166 行：单元区墙取样点换到 256 槽小盘之后只走得到用户数据那一处）	crates/singlefs-core/src/allocator.rs	            DeviceAgreement::Agreed(answer) => answer,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);	            DeviceAgreement::Agreed(answer) => answer,\n            DeviceAgreement::NoAnswerOnAnyDevice => {\n                return Err(PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined { full_devices: Vec::new() });	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- raising_the_floor_with_no_free_slot_outside_the_hold	raising_the_floor_with_no_free_slot_outside_the_hold_fails_before_any_write_and_hands_the_allocator_back_exactly_as_before_the_raise
 代码三方 m2-wave3-code-r1 第四节第 2 条（改法 E）连带：I-3.10 不读下一次挂载会先施加的那一版，第一个事务那条流的快档里 txg 3 记录已落、根槽没落的状态不再评估（逐状态现算的期望红在 I-3.10 评估过的状态数上）	crates/singlefs-checker/src/walk.rs	    tree_table_pointers.extend(tree_table_pointer_of_the_version_the_next_mount_applies_first);	    let _ = tree_table_pointer_of_the_version_the_next_mount_applies_first;	-p singlefs-harness --test first_transaction_step_seven_layer0 -- layer0_partial_enumeration	layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape
 E158 root_choice_repair PC2 的 MultiOffsetReadFailingBlockDevice 读判定取反（该在点名的偏移里才报错，改成不在点名的偏移里才报错——这正是根因一原来那个 bug 的镜像）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	if self.failing_offsets.contains(&offset.0) {\n            return Err(injected_block_device_error("读"));\n        }\n        self.inner.read_at(offset, buffer)	if !self.failing_offsets.contains(&offset.0) {\n            return Err(injected_block_device_error("读"));\n        }\n        self.inner.read_at(offset, buffer)	-p singlefs-harness --bin e158_root_choice_repair	multi_offset_read_failing_block_device_fails_every_named_offset
 E158 root_choice_repair PC2 的本地系统配置字段表合计默认值改成 489（该在没设环境变量时取今天/甲-txg 的 481）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	Err(env::VarError::NotPresent) => 481,	Err(env::VarError::NotPresent) => 489,	-p singlefs-harness --bin e158_root_choice_repair	parse_local_system_configuration_bytes_defaults_to_481_when_unset
@@ -491,8 +491,8 @@
 E158 root_choice_repair H-C1 直接构造：候选2表形态算术改成 1+16*3（该按 Q3-2 量出的 N_w=4 算，登记 5.7 表本地按臂现算的口径改错）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	("候选2表形态 1+16*N_w(N_w=4)", 1 + 16 * 4, 65),	("候选2表形态 1+16*N_w(N_w=4)", 1 + 16 * 3, 65),	-p singlefs-harness --bin e158_root_choice_repair	constants_and_anchors_all_pass
 E158 root_choice_repair 候选 (b) 树表指称「crates 有路」子分支：nearest_older_readable_root_on_same_instance 的实例过滤条件取反（该只挑同一实例更旧的根，改成挑不同实例的根）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	if record.instance != target.instance || record.checkpoint_txg >= target.checkpoint_txg {	if record.instance == target.instance || record.checkpoint_txg >= target.checkpoint_txg {	-p singlefs-harness --bin e158_root_choice_repair	nearest_older_readable_root_on_same_instance_picks_the_closest_ancestor_on_the_same_instance
 E158 root_choice_repair 候选 (b) 分配记录树指称「从内容树反推占用集合」：allocation_record_tree_reachable_placements_via_central_mapping 交回之前清空集合（该带着中央映射树读出的落点，改成永远交回空集）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	        for location in locations {\n            placements.insert((location.device.0, location.slot.0));\n        }\n    }\n    Ok(placements)\n}	        for location in locations {\n            placements.insert((location.device.0, location.slot.0));\n        }\n    }\n    placements.clear();\n    Ok(placements)\n}	-p singlefs-harness --bin e158_root_choice_repair	allocation_record_tree_reachable_placements_via_central_mapping_reflects_written_content_and_errors_on_an_empty_pointer
-增补 2 收口表第 58 行（`raise_rollback_floor` 同形）：抬 F 那一串失败时 core 不交已落盘那几次空发布的账；真设备二进制那一侧的判法（设备一层逐项相等）判出	crates/singlefs-core/src/mount.rs	                publishes\n                    .iter()\n                    .map(|persisted: &TransactionOutput| persisted.writes.clone())\n                    .collect(),	                Vec::new(),	-p singlefs-harness --bin first_transaction_on_device -- failed_raise	failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count
-增补 2 收口表第 58 行（`raise_rollback_floor` 同形）：抬 F 那一串第二次空发布被落点拒绝时报出的已落盘份数成了 0（接替原第 471 行：已落盘次数改成随账交出，原文命中 0 次）	crates/singlefs-core/src/mount.rs	                publishes\n                    .iter()\n                    .map(|persisted: &TransactionOutput| persisted.writes.clone())\n                    .collect(),	                Vec::new(),	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- a_raise_whose_second_empty_publish_is_refused	a_raise_whose_second_empty_publish_is_refused_reports_that_one_publish_of_the_sequence_persisted
+增补 2 收口表第 58 行（`raise_rollback_floor` 同形）：抬 F 那一串失败时 core 不交已落盘那几次空发布的账；真设备二进制那一侧的判法（设备一层逐项相等）判出	crates/singlefs-core/src/mount.rs	                        publishes\n                            .iter()\n                            .map(|persisted: &TransactionOutput| persisted.writes.clone())\n                            .collect(),	                        Vec::new(),	-p singlefs-harness --bin first_transaction_on_device -- failed_raise	failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count
+增补 2 收口表第 58 行（`raise_rollback_floor` 同形）：抬 F 那一串第二次空发布被落点拒绝时报出的已落盘份数成了 0（接替原第 471 行：已落盘次数改成随账交出，原文命中 0 次）	crates/singlefs-core/src/mount.rs	                        publishes\n                            .iter()\n                            .map(|persisted: &TransactionOutput| persisted.writes.clone())\n                            .collect(),	                        Vec::new(),	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- a_raise_whose_second_empty_publish_is_refused	a_raise_whose_second_empty_publish_is_refused_reports_that_one_publish_of_the_sequence_persisted
 增补 2 收口表第 58 行（`raise_rollback_floor` 同形）：抬 F 那一串失败时 core 不交写入口的失败账（与可写挂载共用那一处）	crates/singlefs-core/src/mount.rs	        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),	        writes_of_failed_publishes: Vec::new(),	-p singlefs-harness --bin first_transaction_on_device -- failed_raise	failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count
 增补 2 收口表第 39 行那一族：取号之前的预演取不到落点也照样取号（240 槽小盘上可写挂载取号之后才被落点拒绝、盘上已经写了）	crates/singlefs-core/src/mount.rs	        Err(DryRunRefusal {\n            publish_index,\n            cause: PublishError::PlacementRefused { unit, refusal },\n        }) => {\n            return Err(\n                MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {\n                    instance_to_acquire,\n                    publish_index,\n                    warm_up_publishes_planned: warm_up_publishes_planned.len(),\n                    unit,\n                    refusal,\n                },\n            )\n        }	        Err(DryRunRefusal {\n            publish_index: _,\n            cause: PublishError::PlacementRefused { .. },\n        }) => None,	-p singlefs-harness --test second_transaction_supplement_three_random_history -- a_writable_mount_whose_own_publishes	a_writable_mount_whose_own_publishes_find_no_placement_is_refused_before_acquisition_with_the_disk_unchanged
 增补 2 收口表第 39 行那一族：发布取完落点之后不记根（不转环、不回收；取号之前的预演与真发走同一段，两边一起不回收）	crates/singlefs-core/src/transaction.rs	    allocator.record_root_written_by_this_process(txg);\n    let slot_of = |identity: TransactionUnit| slots[&identity];	    let slot_of = |identity: TransactionUnit| slots[&identity];	-p singlefs-harness --test second_transaction_supplement_three_random_history -- rolling_back_to_the_oldest_ring_root	rolling_back_to_the_oldest_ring_root_reuses_what_the_row_publish_released_and_is_not_refused_before_acquisition
@@ -544,8 +544,8 @@
 E158 root_choice_repair 候选 (b) 分配记录树指称「走全」（session s8）：树表自己的节点漏并进占用集合（该并进 root.tree_table.locations，改成重复并中央映射树自己的节点）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	    for location in &root.tree_table.locations {\n        placements.insert((location.device.0, location.slot.0));\n    }\n    let tree_table_bytes	    for location in &root.mapping_root.locations {\n        placements.insert((location.device.0, location.slot.0));\n    }\n    let tree_table_bytes	-p singlefs-harness --bin e158_root_choice_repair	allocation_record_tree_reachable_placements_via_all_structural_trees_is_a_superset_of_the_central_mapping_only_version
 E158 root_choice_repair 候选 (b) 分配记录树指称「走全」：中央映射树自己的节点漏并进占用集合（该并进 root.mapping_root.locations，改成重复并树表自己的节点）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	    let mut placements =\n        allocation_record_tree_reachable_placements_via_central_mapping(plain_devices, root)?;\n    for location in &root.mapping_root.locations {	    let mut placements =\n        allocation_record_tree_reachable_placements_via_central_mapping(plain_devices, root)?;\n    for location in &root.tree_table.locations {	-p singlefs-harness --bin e158_root_choice_repair	allocation_record_tree_reachable_placements_via_all_structural_trees_is_a_superset_of_the_central_mapping_only_version
 E158 root_choice_repair session s9：classify_diff_pair 的实例表节点判定改成拿 tree_table（该按 root.instance_table.locations 判，改成按 root.tree_table.locations 判——171 格里那两个落点会被错判成别的归类）	crates/singlefs-harness/src/bin/e158_root_choice_repair.rs	if root\n        .instance_table\n        .locations\n        .iter()\n        .any(|location| location_key(location) == pair)\n    {\n        return "instance_table_self_node_not_walked_by_the_all_structural_trees_arm";	if root\n        .tree_table\n        .locations\n        .iter()\n        .any(|location| location_key(location) == pair)\n    {\n        return "instance_table_self_node_not_walked_by_the_all_structural_trees_arm";	-p singlefs-harness --bin e158_root_choice_repair	classify_diff_pair_identifies_the_instance_table_self_node
-C394 N1（用户 2026-09-24 定：读盘失败先重读一次）：读不出不重读、当场按对不上处置（瞬时读错把好的那一份隔离掉）	crates/singlefs-core/src/transaction.rs	                let reading = match read_the_copy().or_else(read_the_copy) {\n	                let reading = match read_the_copy() {\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_release_checksum_read_that_fails_once	a_release_checksum_read_that_fails_once_is_read_again_and_the_intact_copy_is_released_as_usual
-C394 N1：「先重读一次」写成重读两次（条款只重读一次，不多次重试）	crates/singlefs-core/src/transaction.rs	                let reading = match read_the_copy().or_else(read_the_copy) {\n	                let reading = match read_the_copy().or_else(read_the_copy).or_else(read_the_copy) {\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_release_checksum_read_that_keeps_failing	a_release_checksum_read_that_keeps_failing_is_read_once_more_and_then_handled_as_a_mismatch
+C394 N1（用户 2026-09-24 定：读盘失败先重读一次）：读不出不重读、当场按对不上处置（瞬时读错把好的那一份隔离掉）	crates/singlefs-core/src/transaction.rs	                        match check_the_copy_against_its_location_entry(read_the_copy()) {\n	                        match CopyCheck::Unreadable {\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_release_checksum_read_that_fails_once	a_release_checksum_read_that_fails_once_is_read_again_and_the_intact_copy_is_released_as_usual
+C394 N1：「先重读一次」写成重读两次（条款只重读一次，不多次重试）	crates/singlefs-core/src/transaction.rs	                        match check_the_copy_against_its_location_entry(read_the_copy()) {\n	                        match check_the_copy_against_its_location_entry(read_the_copy().or_else(read_the_copy)) {\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_release_checksum_read_that_keeps_failing	a_release_checksum_read_that_keeps_failing_is_read_once_more_and_then_handled_as_a_mismatch
 C394 N3：核出对不上之后照常释放那一份，重挂回收之后那一槽被再发出去（隔离不跨重挂）	crates/singlefs-core/src/transaction.rs	            .filter(|copy| copy.placement == *placement)	            .filter(|copy| copy.placement == *placement && false)	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_quarantined_copy_stays_allocated	a_quarantined_copy_stays_allocated_across_a_remount_and_is_never_handed_out_again
 C394 N3：核出对不上之后照常释放那一份，准入里它不再算已分配（两块盘的可用一样多）	crates/singlefs-core/src/transaction.rs	            .filter(|copy| copy.placement == *placement)	            .filter(|copy| copy.placement == *placement && false)	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_quarantined_copy_counts_as_allocated	a_quarantined_copy_counts_as_allocated_in_the_admission_reading_and_adds_no_term_of_its_own
 C394 × D19 已定项 5（用户 2026-09-25 定：任一份对不上两块盘一起留）：只留对不上的那一份、对得上的那一份照常释放（两块盘的账不对称：冷启动走读判整池失败）	crates/singlefs-core/src/transaction.rs	        for (location, reading) in reading_of_each_copy {\n	        for (location, reading) in reading_of_each_copy\n            .into_iter()\n            .filter(|(_, reading)| *reading != QuarantinedCopyReading::IntactButAnotherCopyFailed)\n        {\n	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_checksum_failure_on_only_one_copy	a_checksum_failure_on_only_one_copy_keeps_both_records_allocated_and_the_pool_stays_writable_and_readable
@@ -664,3 +664,27 @@
 K1 × checker 单测：checker 判记录落不落在叶里只看起点、不看末槽	crates/singlefs-checker/src/position_addressed.rs	    device == leaf_device && span_slots > 0 && slot >= first && slot + span_slots - 1 <= last	    device == leaf_device && span_slots > 0 && slot >= first && slot <= last	-p singlefs-checker --lib -- position_addressed::tests::a_record_crossing	a_record_crossing_the_last_slot_of_its_leaf_does_not_fit
 K1 单测：分配记录树根之下的节点的步号把盘与层级写反	crates/singlefs-core/src/transaction.rs	                "t5@{}.{}.{}",	                "t5@{1}.{0}.{2}",	-p singlefs-core --lib -- transaction::tests::every_transaction_unit_names	every_transaction_unit_names_its_class_tree_and_placement_rule
 K2 × 格式常量：extent 上段一片叶罩的 inode 数写成 142（与 (16384 − 163) ÷ 113 对不上）	crates/singlefs-format/src/lib.rs	pub const EXTENT_TREE_UPPER_LEAF_INODES: u64 = 143;	pub const EXTENT_TREE_UPPER_LEAF_INODES: u64 = 142;	-p singlefs-format --lib -- widths_match_the_first_transaction_byte_table	widths_match_the_first_transaction_byte_table
+实二五 准入（C363 (b) 判决第四节第 2 条）：发布路径不判空间准入（式子判拒的覆盖写照样发出去）	crates/singlefs-core/src/transaction.rs	    if demand_is_judged {	    if false && demand_is_judged {	-p singlefs-harness --test second_transaction_supplement_two_admission_formula -- an_overwrite_whose_ordinary	an_overwrite_whose_ordinary_allocations_exceed_the_available_bytes_is_refused_by_the_space_admission_before_any_write
+实二五 准入：可写挂载取号之前不判空间准入（实例切换的预留拿不到照样取号）	crates/singlefs-core/src/mount.rs	    match start.space_admission {	    match SpaceAdmission::SkippedByTheTestOnlySwitch {	-p singlefs-harness --test second_transaction_supplement_two_admission_formula -- a_writable_mount_whose_instance_switch	a_writable_mount_whose_instance_switch_reserve_does_not_fit_is_refused_by_the_space_admission_before_acquisition_with_the_disk_unchanged
+实二五 准入：需求把固定点也算进去（固定点已在 checkpoint 保留池里扣过，算两遍）	crates/singlefs-core/src/admission.rs	        .filter(|role| space_budget_of_role(**role) == SpaceBudgetOfARole::Demand)	        .filter(|role| space_budget_of_role(**role) != SpaceBudgetOfARole::InstanceSwitchReserve)	-p singlefs-harness --test second_transaction_supplement_two_admission_formula -- an_overwrite_whose_ordinary	an_overwrite_whose_ordinary_allocations_exceed_the_available_bytes_is_refused_by_the_space_admission_before_any_write
+实二五 准入（D28 已定项 4 Σ 名单）：ckpt_cost 漏掉树表那一项	crates/singlefs-core/src/admission.rs	    MetadataBlocks(record_trees_and_accounting_nodes + TREE_TABLE_UNITS_PER_PUBLISH)	    MetadataBlocks(record_trees_and_accounting_nodes)	-p singlefs-harness --test second_transaction_supplement_two_admission_formula -- a_writable_mount_whose_instance_switch	a_writable_mount_whose_instance_switch_reserve_does_not_fit_is_refused_by_the_space_admission_before_acquisition_with_the_disk_unchanged
+实二五 准入（D28 已定项 3）：读数里的挂载期承诺量不扣实例切换的预留	crates/singlefs-core/src/admission.rs	        instance_switch_reserve_on_one_device(\n            allocator.instance_rows_after_this_mounts_row_publish(),\n            checkpoint_cost,\n        ),	        BytesOnOneDevice::ZERO,	-p singlefs-harness --test second_transaction_supplement_two_admission_formula -- a_writable_mount_whose_instance_switch	a_writable_mount_whose_instance_switch_reserve_does_not_fit_is_refused_by_the_space_admission_before_acquisition_with_the_disk_unchanged
+实二五 准入：胶水把发布被空间准入拒映射成模型没有的理由（准入判着的小盘取样判出）	crates/singlefs-harness/src/model_comparison.rs	        PublishError::SpaceAdmissionRefused(_) => explained(ModelRefusalReason::UnitAreaWall),	        PublishError::SpaceAdmissionRefused(_) => ObservedRefusalReason::Unexplained,	-p singlefs-harness --test second_transaction_supplement_three_random_history -- unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged	unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval
+实二五 准入：只供测试的开关关不掉发布路径的准入（关掉准入的那一档照样被式子拒）	crates/singlefs-core/src/transaction.rs	        SpaceAdmission::SkippedByTheTestOnlySwitch => false,	        SpaceAdmission::SkippedByTheTestOnlySwitch => demand\n            .iter()\n            .any(|demand_on_device| demand_on_device.bytes != BytesOnOneDevice::ZERO),	-p singlefs-harness --test second_transaction_supplement_two_admission_formula -- an_overwrite_whose_ordinary	an_overwrite_whose_ordinary_allocations_exceed_the_available_bytes_is_refused_by_the_space_admission_before_any_write
+实二五 C546（抬 F 被拒时扣住的槽不退回）：第一次空发布在任何写之前被拒时分配器不换回抬 F 之前那一份	crates/singlefs-core/src/mount.rs	                    *allocator = allocator_before_the_raise.clone();	                    let _ = &allocator_before_the_raise;	-p singlefs-harness --test second_transaction_supplement_two_commit_generated_fallback -- raising_the_floor_with_no_free_slot_outside_the_hold	raising_the_floor_with_no_free_slot_outside_the_hold_fails_before_any_write_and_hands_the_allocator_back_exactly_as_before_the_raise
+实二五 D19 已定项 5（核出对不上先重读一次）：读得出而对不上不重读、当场隔离	crates/singlefs-core/src/transaction.rs	                    CopyCheck::Unreadable | CopyCheck::ChecksumMismatch => {	                    CopyCheck::ChecksumMismatch => QuarantinedCopyReading::ChecksumMismatch,\n                    CopyCheck::Unreadable => {	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_release_checksum_read_that_returns_corrupted_bytes_once	a_release_checksum_read_that_returns_corrupted_bytes_once_is_read_again_and_the_intact_copy_is_released_as_usual
+实二五 D19 已定项 5（同一处）：读得出而对不上不重读；故障注入快档判出（一次坏读隔离一对好槽，I-3.11 红）	crates/singlefs-core/src/transaction.rs	                    CopyCheck::Unreadable | CopyCheck::ChecksumMismatch => {	                    CopyCheck::ChecksumMismatch => QuarantinedCopyReading::ChecksumMismatch,\n                    CopyCheck::Unreadable => {	-p singlefs-harness --test second_transaction_supplement_three_fault_injection -- fault_injection_fast_tier	fault_injection_fast_tier_returns_errors_instead_of_panicking
+实二五 D19 已定项 5：重读那一次仍对不上时当成对得上（两次都对不上也不隔离）	crates/singlefs-core/src/transaction.rs	                            CopyCheck::ChecksumMismatch => QuarantinedCopyReading::ChecksumMismatch,	                            CopyCheck::ChecksumMismatch => QuarantinedCopyReading::IntactButAnotherCopyFailed,	-p singlefs-harness --test second_transaction_supplement_two_release_checksum_quarantine -- a_release_checksum_read_that_keeps_returning	a_release_checksum_read_that_keeps_returning_corrupted_bytes_is_read_once_more_and_then_quarantined
+实二五 Z9-A（代码轮第二轮判决第四节第 1 条）：被同一张表里另一条罩住的条目不删	crates/singlefs-core/src/mount.rs	                    && other.new_instance >= entry.new_instance	                    && other.new_instance >= entry.new_instance\n                    && false	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- rolling_back_to_the_same_target	rolling_back_to_the_same_target_twenty_four_times_with_every_mount_crashing_after_the_row_publish_rotation_never_fills_the_witness_table
+实二五 Z9-A：罩住判定不看新实例代号（新实例更小的那条也拿去删别的，删掉仍护着根的条目）	crates/singlefs-core/src/mount.rs	                    && other.new_instance >= entry.new_instance	                    && other.new_instance >= InstanceGeneration(0)	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- rolling_back_to_the_newest_root_every_time	rolling_back_to_the_newest_root_every_time_with_every_mount_crashing_after_the_row_publish_rotation_still_fills_the_witness_table_on_the_twenty_fourth
+实二五 Z9-B（代码轮第二轮判决第四节第 3 条）：可写挂载不按所选根实例表里的回退行补见证	crates/singlefs-core/src/mount.rs	        .filter(|row| row.is_rollback)	        .filter(|row| row.is_rollback && false)	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- after_a_rollback_mount_crashed	after_a_rollback_mount_crashed_between_the_row_root_and_its_rotation_each_later_writable_mount_restores_the_missing_witness
+实二五 Z9-B：补见证的 N 取回退行之后第一个有行的实例（中间实例的 (i, 0, 0) 也算，罩不住中间实例的根）	crates/singlefs-core/src/mount.rs	                        && later_row.selected_root_txg != CheckpointTxg(0)	                        && later_row.selected_root_txg >= CheckpointTxg(0)	-p singlefs-harness --test second_transaction_supplement_two_rollback_witness -- the_witness_restored_after	the_witness_restored_after_a_rollback_across_instances_names_the_instance_that_rolled_back_not_the_intermediate_one
+实二五 Z10（代码轮第二轮判决第四节第 5 条）：真设备二进制冷重开之后打的 F 不是所选根从盘上读回的	crates/singlefs-harness/src/bin/first_transaction_on_device.rs	            root.instance.0, root.checkpoint_txg.0, root.rollback_floor.0	            root.instance.0, root.checkpoint_txg.0, 0	-p singlefs-harness --bin first_transaction_on_device -- raise_rollback_floor_mode_prints	tests::raise_rollback_floor_mode_prints_the_floor_read_back_from_the_chosen_root_after_the_cold_reopen
+实二五 checker 坏镜像语料：I-7.11 恒成立（见证条目所选根的实例表罩不住也不红）	crates/singlefs-checker/src/walk.rs	            target_row_covers && missing_intermediate.is_none(),	            true || (target_row_covers && missing_intermediate.is_none()),	-p singlefs-harness --test checker_known_bad_images -- each_rollback_witness_bad_image	each_rollback_witness_bad_image_reddens_its_own_invariant
+实二五 checker（实二一报告第 5 条）：按位置寻址的树根之下挂一个空节点不判 I-1.1	crates/singlefs-checker/src/lib.rs	    if view.entries.is_empty() {\n        return Err(Verdict::KeyOutsideDeclaredRange);\n    }	    if false {\n        return Err(Verdict::KeyOutsideDeclaredRange);\n    }	-p singlefs-harness --test checker_known_bad_images -- an_empty_leaf_hung_below	an_empty_leaf_hung_below_the_root_of_the_allocation_record_tree_reddens_only_the_block_identity_invariant
+E156 M27（第 3 次重跑登记第九节）：R-3 闭式漏掉根（改写节点数少 1）	crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	devices * u64::try_from(touched_leaves.len()).expect("叶数落在 u64 内")\n        + devices * u64::try_from(touched_level1.len()).expect("层级 1 数落在 u64 内")\n        + 1	devices * u64::try_from(touched_leaves.len()).expect("叶数落在 u64 内")\n        + devices * u64::try_from(touched_level1.len()).expect("层级 1 数落在 u64 内")\n        + 0	-p singlefs-harness --bin e156_allocation_basis_counts	pc_closed_form_matches_the_registered_anchor_r3c$
+E156 M28（第 3 次重跑登记第九节）：R-3 闭式每个改动的叶位置只算一块盘（丢了 devices 因子）	crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	devices * u64::try_from(touched_leaves.len()).expect("叶数落在 u64 内")\n        + devices * u64::try_from(touched_level1.len()).expect("层级 1 数落在 u64 内")	u64::try_from(touched_leaves.len()).expect("叶数落在 u64 内")\n        + u64::try_from(touched_level1.len()).expect("层级 1 数落在 u64 内")	-p singlefs-harness --bin e156_allocation_basis_counts	pc_closed_form_matches_the_registered_anchor_r3c$
+E156 M29（第 3 次重跑登记第九节）：R-3 闭式的「改动的记录」只取这次分配的、不取这次释放的	crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	.filter(|record| record.device == device && record.generation == txg)	.filter(|record| record.device == device && record.generation == txg && !record.is_released)	-p singlefs-harness --bin e156_allocation_basis_counts	pc_closed_form_third_group_counts_released_and_allocated_records$
+E156 M30（第 3 次重跑登记第九节）：Q7d-1 覆盖写那一组退回字面 10	crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	        samples.iter().min().copied().unwrap_or(0),\n        samples.iter().max().copied().unwrap_or(0),	        10,\n        10,	-p singlefs-harness --bin e156_allocation_basis_counts	overwrite_steps_match_the_closed_form_and_cross_a_second_leaf$
+E156 M31（第 3 次重跑登记第九节）：K1-1 第 1 项的登记值退回 13	crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	assert_eq!(allocated, 17, "K1-1 第 1 项");	assert_eq!(allocated, 13, "K1-1 第 1 项");	-p singlefs-harness --bin e156_allocation_basis_counts	accounting_after_first_transaction_matches_the_registered_anchor$
+E156 M32（第 3 次重跑登记第九节）：S1(c) 的隔离槽数退回 34	crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	vec![(DeviceIdentity(0), 54), (DeviceIdentity(1), 54)],	vec![(DeviceIdentity(0), 34), (DeviceIdentity(1), 34)],	-p singlefs-harness --bin e156_allocation_basis_counts	rollback_isolation_scenario_matches_the_new_layout$
diff -ruN -x target /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-core/src/admission.rs tree/crates/singlefs-core/src/admission.rs
--- /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-core/src/admission.rs	2026-09-24 03:51:41.943258819 +0000
+++ tree/crates/singlefs-core/src/admission.rs	2026-09-24 23:55:14.650113749 +0000
@@ -6,15 +6,25 @@
 //!
 //! 全是只读的纯函数：不动分配器、不发一个写。
 //!
-//! ⚠️ **这一版只有读数与合取，发布路径与可写挂载都还不调它**（调用点全在用例里）。九项里有两样的数条款没给，这里不替它们定，
-//! 由调用方给：
-//! - checkpoint 保留池要的 ckpt_cost（D28（挂载期承诺量） 已定项 4：Σ（每棵记录树当前的高）+ 记账树每发布的节点数）：
-//!   树高从哪读、「记录树」指哪几棵没有条款（C363（现算保留池时树高从哪读没有条款），用户 2026-09-23 定另开一题，新题还没开）；
-//!   「按每次发布写多少算」那一读法在增补 2 收口表第 ② 行的岔路里。
-//! - 挂载期承诺量里暖机那一半的 c_max：D28（挂载期承诺量） 已定项 4 末条定它与保留池「按同一个现算的 c_max 取」，同一个空白。
+//! 接在两处（C363 (b) 判决 `research/prompts/c363b-r1-main-verification.md` 第四节第 2 条，里程碑「第二个事务」增补 2 收口表第 5 行）：
+//! - 发布路径：`transaction::prepare_the_version_publish` 在算定这次发布的样子之后、读盘核与动分配器之前，按
+//!   [`admission_reading_before_a_publish`] 取读数、按 [`demand_of_the_roles_on_each_device`] 取这次的需求，逐设备合取；
+//! - 可写挂载：`mount::establish_instance` 在取号之前按同一份读数判「实例切换的预留拿得到」（D2（RAID 条带策略） 已定项 13），需求逐盘 0。
 //!
-//! 需求怎么摊到每块盘（C370（需求、可用与 df 没有共同单位））同样由调用方给。准入不够时先推空发布抬 F 再判
-//! （D16（发布语义） 已定项 1，C283（准入失败时不先推发布就报 ENOSPC））不在这里：它押在准入接进发布路径上。
+//! 条款给的量：checkpoint 保留池的 ckpt_cost 按 D28（挂载期承诺量） 已定项 4 的 Σ 名单（分配记录树、中央映射树按树高，
+//! 记账树按每发布的节点数，树表一项，实例表链不进；[`checkpoint_cost_of_the_version_to_build_on`]）；挂载期承诺量暖机那一半的 c_max
+//! 与保留池「按同一个现算的 c_max 取」（D28（挂载期承诺量） 已定项 4 末条）。
+//!
+//! ⚠️ 实现员取的读法（条款没写，交主 agent；[`space_budget_of_role`] 与 [`admission_reading_before_a_publish`] 的文档注释写了依据）：
+//! - 需求只算这次发布的普通分配（用户数据单元、extent 树与 inode 树的节点），固定点与实例表链各有自己那一项保留，不重复算需求；
+//!   一次发布一个普通分配都没有（空发布、写行）就不判——它们的空间在保留池与切换预留里，判它们会让推空发布抬 F（D3（空间分配）
+//!   已定项 17「释放空间这个操作本身不需要申请空间」）与挂载自己那一串被式子挡住；
+//! - 需求按盘字节记（C370（需求、可用与 df 没有共同单位） 2026-09-17 收窄：与「已分配」同口径只能读成盘字节），第一版每个单元落每块盘，
+//!   每块盘的需求相同；
+//! - 读数取分配器此刻的计数（挂载时是回收与影子账隔离之后、写行之前那一刻；admission 读哪一个在那两段里条款没写，见
+//!   [`AdmissionReading::of_allocator`]）。
+//!
+//! 准入不够时先推空发布抬 F 再判（D16（发布语义） 已定项 1，C283（准入失败时不先推发布就报 ENOSPC））没有实现：直接在任何写之前拒。
 
 use std::collections::BTreeSet;
 use std::num::NonZeroU64;
@@ -22,8 +32,9 @@
 use singlefs_format::{INSTANCE_TABLE_PAGE_RECORDS, SLOT_BYTES};
 
 use crate::address::DeviceIdentity;
+use crate::allocation_record_tree::AllocationRecordTreeGeometry;
 use crate::allocator::PoolAllocator;
-use crate::transaction::TransactionUnit;
+use crate::transaction::{MultiLevelCodeTwoTree, TransactionOutput, TransactionUnit};
 
 /// 一块盘上的物理字节：D5（快照 / 空间记账机制） 已定项 7「已分配」的口径——分配记录每个落点每盘一条，逐盘记的是这块盘上真占的字节。
 /// 一份副本的大小也用它：一份副本整个落在一块盘上。
@@ -441,6 +452,151 @@
     }
 }
 
+/// 只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：发布与可写挂载判不判空间准入。产品路径恒 `JudgedByTheFormula`。
+/// 准入接进来之后，「准入放行而落点仍取不到、在任何写之前拒绝」那一条（D3（空间分配） 已定项 5；C545（空间准入罩不住分裂与聚簇段层））
+/// 在健康的历史里走不到——小盘上式子先拒。关掉准入，那一条拒绝路径（取号之前的预演取不到落点、发布取不到落点、抬 F 的空发布取不到固定点）
+/// 照样测得到。装在分配器上（`PoolAllocator::set_space_admission`），可写挂载按调用方给的装（`mount::mount_writable_with_space_admission`）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum SpaceAdmission {
+    JudgedByTheFormula,
+    SkippedByTheTestOnlySwitch,
+}
+
+impl SpaceAdmission {
+    /// 这一次走的是哪一臂，运行时报得出（五条硬要求第 4 条：分支必须可观测）。
+    #[must_use]
+    pub const fn branch_name(self) -> &'static str {
+        match self {
+            SpaceAdmission::JudgedByTheFormula => "space_admission=judged",
+            SpaceAdmission::SkippedByTheTestOnlySwitch => "space_admission=skipped",
+        }
+    }
+}
+
+/// checkpoint 保留池的 ckpt_cost（D28（挂载期承诺量） 已定项 4，Σ 名单照 `research/prompts/c363b-r1-main-verification.md` 第三节 V2-2）：
+/// Σ（分配记录树与中央映射树当前的高）+ 记账树每发布的节点数 + 1（树表每次发布重写一个单元，D16（发布语义） 已定项 9）；
+/// 实例表链不进（它的开销归已定项 3 的切换预留）。「当前」= 这次发布要接在后面的那一版：
+/// - 带文件的一版：两棵树的高从各自根节点的码 2 头里现读（层级 + 1，不用内存里另存一份，已定项 4）；记账树每次发布整批重写
+///   （`transaction` 装记账行那一段），每发布的节点数就是这一版记账树的节点数。
+/// - 树表 0 条的一版（`None`）：没有中央映射树与记账树（0 与 0）；分配记录树只在那一版写过行时有（分配器记着它，
+///   `PoolAllocator::allocation_record_tree_of_the_version_without_file`），它按位置寻址、根的层级由池几何定（D8（核心索引结构） 已定项 14），
+///   高取几何的高——这一处分配器里只有节点与指针、没有根节点的字节可读；没写过行的（mkfs 的第 0 代）一棵都没有，取 0。
+///
+/// 同一个数也是挂载期承诺量里暖机那一半的 c_max（已定项 4 末条「按同一个现算的 c_max 取」）。
+#[must_use]
+pub fn checkpoint_cost_of_the_version_to_build_on(
+    version_to_build_on: Option<&TransactionOutput>,
+    allocator: &PoolAllocator,
+) -> MetadataBlocks {
+    const TREE_TABLE_UNITS_PER_PUBLISH: u64 = 1;
+    let record_trees_and_accounting_nodes = match version_to_build_on {
+        Some(file_version) => {
+            file_version
+                .position_addressed_tree_heights_read_from_the_root_node_headers()
+                .allocation_record_tree
+                + file_version
+                    .height_read_from_the_root_node_header(MultiLevelCodeTwoTree::CentralMapping)
+                + u64::try_from(file_version.accounting_tree.node_count())
+                    .expect("记账树的节点数装得进 u64")
+        }
+        None => match allocator.allocation_record_tree_of_the_version_without_file() {
+            Some(_) => AllocationRecordTreeGeometry::of_allocator(allocator).height(),
+            None => 0,
+        },
+    };
+    MetadataBlocks(record_trees_and_accounting_nodes + TREE_TABLE_UNITS_PER_PUBLISH)
+}
+
+/// 一次发布里一个角色的空间归哪一格（实现员取的读法，条款没写；依据见各成员）：
+/// 准入不等式另一边的「需求」只算 [`SpaceBudgetOfARole::Demand`] 那一格，另外两格已经作为式子里的一项扣在「可用」那一边。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum SpaceBudgetOfARole {
+    /// 普通分配：用户数据单元，与因为用户这次改了内容才重写的 extent 树、inode 树的节点。保留池对它是纯税
+    /// （D23（journal 的角色与格式） 已定项 24「保留池是给 checkpoint 开的一道地板，对普通分配是纯税」）。
+    Demand,
+    /// checkpoint 自己的固定点：ckpt_cost 的 Σ 名单里那几棵（分配记录树、中央映射树、记账树）与树表。
+    /// 它们从式子第八项 checkpoint 保留池里出（D28（挂载期承诺量） 已定项 1「它保证 checkpoint 自己的固定点写得出去」）。
+    CheckpointReservePool,
+    /// 实例表链：写行那次整条重写，从挂载期承诺量里的实例切换预留出（D28（挂载期承诺量） 已定项 3「多的一份给写行那次发布的元数据」、
+    /// 已定项 4「实例表链不进 ckpt_cost，它的开销归已定项 3 的切换预留」）。
+    InstanceSwitchReserve,
+}
+
+/// 这个角色的空间归哪一格（[`SpaceBudgetOfARole`]）。
+#[must_use]
+pub const fn space_budget_of_role(role: TransactionUnit) -> SpaceBudgetOfARole {
+    match role {
+        TransactionUnit::Data(_)
+        | TransactionUnit::ExtentLowerNode(_)
+        | TransactionUnit::ExtentUpperNodeBelowTheRoot(_)
+        | TransactionUnit::ExtentRoot
+        | TransactionUnit::InodeLeafContainer(_)
+        | TransactionUnit::InodeRoot => SpaceBudgetOfARole::Demand,
+        TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
+        | TransactionUnit::AllocationTree
+        | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
+        | TransactionUnit::AccountingTree
+        | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
+        | TransactionUnit::MappingTree
+        | TransactionUnit::TreeTable => SpaceBudgetOfARole::CheckpointReservePool,
+        TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
+            SpaceBudgetOfARole::InstanceSwitchReserve
+        }
+    }
+}
+
+/// 一次发布重写的角色里算需求的那几个（[`space_budget_of_role`] 是 `Demand` 的）要在每块盘上新占的物理字节：
+/// 第一版每个单元落池里每一块盘（`ReplicaCount::of_every_device_in_the_pool` 同一条理由），每块盘的需求相同；
+/// 按盘字节记（C370（需求、可用与 df 没有共同单位） 2026-09-17 收窄：需求要与逐盘物理字节的「已分配」同口径）。
+/// 按 `devices` 的次序每块盘一条，需求为零也写 0（`admit_on_every_device` 要逐盘写全）。
+///
+/// # Panics
+/// 字节数装不进 u64：一次发布的角色数有上界（一版的节点数），装不下说明调用方给的不是一次发布的角色清单。
+#[must_use]
+pub fn demand_of_the_roles_on_each_device(
+    rewritten_roles: &[TransactionUnit],
+    devices: &[DeviceIdentity],
+) -> Vec<DemandOnDevice> {
+    let slots_of_the_demand: u64 = rewritten_roles
+        .iter()
+        .filter(|role| space_budget_of_role(**role) == SpaceBudgetOfARole::Demand)
+        .map(|role| role.span_slots())
+        .sum();
+    let bytes = BytesOnOneDevice::of_slots(slots_of_the_demand);
+    devices
+        .iter()
+        .map(|device| DemandOnDevice {
+            device: *device,
+            bytes,
+        })
+        .collect()
+}
+
+/// 一次发布之前的准入读数（D28（挂载期承诺量） 已定项 1 的九项）：逐盘各项取分配器此刻的计数（[`AdmissionReading::of_allocator`]）；
+/// 挂载期承诺量 = 实例切换的预留（已定项 3），rows0 取这次挂载记在分配器上的那个数
+/// （`PoolAllocator::instance_rows_after_this_mounts_row_publish`，挂载期间常量；mkfs 同一个进程里是 0：mkfs 的实例表一行都没有、
+/// 实例 1 不写行），c_max 与 checkpoint 保留池的 ckpt_cost 是同一个现算的数（[`checkpoint_cost_of_the_version_to_build_on`]）；
+/// 待删占用与已承诺预留取第一版的两个 0。可写挂载在取号之前判的也是这一份（挂载那一刻 rows0 已经记在分配器上）。
+#[must_use]
+pub fn admission_reading_before_a_publish(
+    allocator: &PoolAllocator,
+    version_to_build_on: Option<&TransactionOutput>,
+) -> AdmissionReading {
+    let checkpoint_cost =
+        checkpoint_cost_of_the_version_to_build_on(version_to_build_on, allocator);
+    AdmissionReading::of_allocator(
+        allocator,
+        instance_switch_reserve_on_one_device(
+            allocator.instance_rows_after_this_mounts_row_publish(),
+            checkpoint_cost,
+        ),
+        PoolWideCommitments::of_the_first_version(checkpoint_reserve_pool(
+            checkpoint_cost,
+            ReplicaCount::of_every_device_in_the_pool(allocator),
+        )),
+    )
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
diff -ruN -x target /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-core/src/allocator.rs tree/crates/singlefs-core/src/allocator.rs
--- /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-core/src/allocator.rs	2026-09-24 19:23:04.554198476 +0000
+++ tree/crates/singlefs-core/src/allocator.rs	2026-09-24 23:55:14.659113469 +0000
@@ -17,6 +17,7 @@
 use std::collections::{BTreeMap, BTreeSet};
 
 use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
+use crate::admission::SpaceAdmission;
 use crate::bytes::ByteWriter;
 use crate::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
 use crate::transaction::FrozenPublish;
@@ -760,6 +761,14 @@
     /// 「这一版的失败处置」）。住在分配器上是因为每一次发布都要交分配器进来：冻结着的时候发布路径在任何读写之前拒绝，
     /// 只有 `transaction::resend_the_frozen_publish` 清得掉它。只住内存：重开之后走恢复、换实例代号，冻结的那一次不带过去。
     frozen_publish: Option<Box<FrozenPublish>>,
+    /// 这次挂载的 rows0（D28（挂载期承诺量） 已定项 3）：挂载时读到的实例表行数加写行那次要写的行数，挂载期承诺量里实例切换预留
+    /// 那一项按它算（`admission::admission_reading_before_a_publish`）。挂载期间常量、只住内存、每次挂载重算：
+    /// 可写挂载在取号之前记上（`mount::establish_instance`），`new` 与 `rebuild_from_records` 起步是 0——mkfs 同一个进程里
+    /// mkfs 的实例表一行都没有、取号 1 不写行（D18（块里携带什么信息） 已定项 11），rows0 本来就是 0。
+    instance_rows_after_this_mounts_row_publish: u64,
+    /// 判不判空间准入，只供测试的开关（`admission::SpaceAdmission`）。只住内存：`new` 与 `rebuild_from_records` 起步是判；
+    /// 可写挂载按调用方给的装（`mount::mount_writable_with_space_admission`）。
+    space_admission: SpaceAdmission,
 }
 
 impl PoolAllocator {
@@ -778,9 +787,33 @@
             placements_reclaimed_on_release_by_the_forced_zero_reuse_window: 0,
             root_ring: None,
             frozen_publish: None,
+            instance_rows_after_this_mounts_row_publish: 0,
+            space_admission: SpaceAdmission::JudgedByTheFormula,
         }
     }
 
+    /// 装上判不判空间准入这个只供测试的开关（`admission::SpaceAdmission`）。产品路径一处都不调它。
+    pub fn set_space_admission(&mut self, space_admission: SpaceAdmission) {
+        self.space_admission = space_admission;
+    }
+
+    /// 这会儿装着的空间准入开关：运行时看得出走的是哪一条分支。
+    #[must_use]
+    pub fn space_admission(&self) -> SpaceAdmission {
+        self.space_admission
+    }
+
+    /// 记上这次挂载的 rows0（字段 `instance_rows_after_this_mounts_row_publish` 的注释）。可写挂载在取号之前调一次。
+    pub fn record_instance_rows_after_this_mounts_row_publish(&mut self, rows: u64) {
+        self.instance_rows_after_this_mounts_row_publish = rows;
+    }
+
+    /// 这次挂载的 rows0；没挂载过的（mkfs 同一个进程）是 0。
+    #[must_use]
+    pub fn instance_rows_after_this_mounts_row_publish(&self) -> u64 {
+        self.instance_rows_after_this_mounts_row_publish
+    }
+
     /// 冻结着的那一次发布；没有时 `None`。
     #[must_use]
     pub fn frozen_publish(&self) -> Option<&FrozenPublish> {
diff -ruN -x target /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-core/src/mount.rs tree/crates/singlefs-core/src/mount.rs
--- /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-core/src/mount.rs	2026-09-24 21:35:39.449983013 +0000
+++ tree/crates/singlefs-core/src/mount.rs	2026-09-24 23:55:14.671113097 +0000
@@ -4,6 +4,10 @@
 //! 第一版没有干净关闭标记，重开一律走恢复。
 
 use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
+use crate::admission::{
+    admission_reading_before_a_publish, admit_on_every_device, AdmissionRefusedOnSomeDevices,
+    BytesOnOneDevice, DemandOnDevice, SpaceAdmission,
+};
 use crate::allocation_record_tree::{
     node_pointers_as_far_as_readable, AllocationRecordTreeGeometry,
 };
@@ -27,7 +31,7 @@
     RebuiltVersion, RecoveryFailure, UserVisibleTreeRootPointers,
 };
 use crate::rollback_witness::{
-    rollback_witness_capacity, RollbackWitness, RollbackWitnessEntry, RollbackWitnessTable,
+    rollback_witness_capacity, RollbackWitnessEntry, RollbackWitnessTable,
 };
 use crate::root_record::RootRecord;
 use crate::root_ring::{target_for_publish, RootRingSlot};
@@ -62,7 +66,8 @@
     /// 抬 F 那一串空发布（D16（发布语义） 已定项 1：推到每块盘上都有一条带新 F 的根才生效）里有一次发不出去：错，连同抬 F 自己开的
     /// 写入口交得出的写账（[`PublishSequenceFailed`]，增补 2 收口表第 58 行，与可写挂载那一条同一个形态）。
     /// `writes_of_persisted_publishes` 有几份，前面就有几次已经落盘——带新 F 的根在盘上、调用方的现行版本已经是最后落盘的那一版，
-    /// F 还没在每块盘上生效，抬 F 回收的槽照旧扣着；`cause` 是下一次（份数 + 1，从 1 数）的错。`cause` 自己说的「在任何写之前」
+    /// F 还没在每块盘上生效，抬 F 回收的槽照旧扣着；一份都没有、第一次在任何写之前被拒时，分配器与抬 F 之前逐项相同
+    /// （扣住的槽放开、回收的回到 defer，C546（抬 F 被拒时扣住的槽不退回））。`cause` 是下一次（份数 + 1，从 1 数）的错。`cause` 自己说的「在任何写之前」
     /// （例如 `PublishError::PlacementRefused`）只对出错的这一次成立，不对整串成立（C516（抬 F 那一串发布被拒时前面几次已落盘））。
     RaiseFloorSequencePublishFailed(PublishSequenceFailed),
     /// 回退的目标根不在回退候选集里，`exclusion` 说是哪一条（管理员要做的决定都是换一条目标；调用方按这个字段分流，不看给人看的文字——
@@ -103,6 +108,14 @@
         expected: InstanceGeneration,
         recomputed: InstanceGeneration,
     },
+    /// 空间准入不够（D28（挂载期承诺量） 已定项 1 的式子逐设备合取；C363 (b) 判决 `research/prompts/c363b-r1-main-verification.md`
+    /// 第四节第 2 条把它接进可写挂载）：扣掉这次挂载的实例切换预留（挂载期承诺量，D28（挂载期承诺量） 已定项 3）与别的八项之后，
+    /// 至少一块盘的可用(d) < 0——「实例切换的预留拿得到」这条可写挂载准入合取不成立（D2（RAID 条带策略） 已定项 13：任一不成立即只读挂载；
+    /// 这里交回错误，要不要只读挂载由调用方定）。在**取号之前**返回，一个写都没发、盘上逐字节不变、两块盘系统配置里的实例代号不动。
+    SpaceAdmissionRefusedBeforeAcquisition {
+        instance_to_acquire: InstanceGeneration,
+        refusal: AdmissionRefusedOnSomeDevices,
+    },
     /// 写行那次发布在取号之前的预演（发布路径落盘之前那一段：释放核验、这次要换下的那条实例表旧链逐片核、两棵多层码 2 树的形状、
     /// 分配记录树重写集合的固定点）报错：在**取号之前**拒绝，盘上一个字节都不动、
     /// 两块盘系统配置里的实例代号不动（增补 2 第 20a 行，代码三方第一轮打中）。改之前这些只在发布路径里算，
@@ -231,7 +244,8 @@
     /// （步 4 / 步 5 代码三方第二轮云端攻方腿打中：一条被抛弃根的树表撕裂不能让每次挂载都失败）。
     pub abandoned_roots_unreadable: u64,
     /// 这次挂载从写行那次发布的系统配置轮换起写的回退见证表（D23（journal 的角色与格式） 已定项 14「回退见证」）：
-    /// 盘上原有的按删除规则删过（`rollback_witness_entries_still_needed`），回退时再加这一次的那一条。取号那两次写带的是删过、还没加的那一张。
+    /// 盘上原有的先按所选根实例表里的回退行补回缺的条目，再按删除规则删过（`rollback_witness_tables_of_this_mount`），
+    /// 回退时再加这一次的那一条。取号那两次写带的是删过、还没加的那一张。
     pub rollback_witness_written: RollbackWitnessTable,
 }
 
@@ -279,6 +293,14 @@
             | PreviousVersion::WithFile { table, .. } => table,
         }
     }
+
+    /// 带文件的那一版；树表 0 条的是 `None`（空间准入的 ckpt_cost 按它算，`admission::checkpoint_cost_of_the_version_to_build_on`）。
+    fn file_version(&self) -> Option<&TransactionOutput> {
+        match self {
+            PreviousVersion::WithoutFile { .. } => None,
+            PreviousVersion::WithFile { output, .. } => Some(output),
+        }
+    }
 }
 
 /// 写行那次发布的实例表链怎么重写：这一版那张表的行后面接上这次写的行，被换下的是这一版的整条链
@@ -347,6 +369,13 @@
     shadow_ledger: ShadowLedger,
     /// 恢复（或回退）择到的系统配置：回退见证表的条数上限（R × S − 1）与删除规则读的根环几何从它取。
     system_configuration: crate::system_configuration::SystemConfiguration,
+    /// 所选根（`chosen_root`）指着的那张实例表的全部行：删除规则之前按其中的回退行补回见证表缺的条目
+    /// （`rollback_witness_entries_recovered_from_the_instance_table`）。可写挂载取施加前缀之后那一版的表——施加记录照抄
+    /// 所选根的实例表指针（`recovery::replay_journal`），两者是同一张；回退取最新根那一张（候选集按它判的那一张）。
+    /// 两处都是这次挂载已经读出来的，不为它另读一次盘。
+    rows_of_the_chosen_roots_instance_table: Vec<InstanceRow>,
+    /// 判不判空间准入（只供测试的开关，`admission::SpaceAdmission`）：产品路径恒判。
+    space_admission: SpaceAdmission,
 }
 
 /// 新实例的第一次发布的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1
@@ -945,6 +974,9 @@
             },
         ));
     }
+    // 下面的影子账重算与回收在第一次空发布之前就动分配器：第一次空发布在任何写之前被拒（落点、准入、释放判定），这一串一次都没落盘、
+    // F 没有一条根带出去，分配器整个换回这一份——扣住的槽放开、回收的回到 defer、补的隔离撤掉（C546（抬 F 被拒时扣住的槽不退回））。
+    let allocator_before_the_raise = allocator.clone();
     let system_configuration = choose_system_configuration(&*devices)?;
     // 候选集按现行那一版的实例表判：从它的根指着的第 0 片沿链真读出来、解出来（C502（抬 F 时现行版本里没有实例表单元）；
     // D18（块里携带什么信息） 已定项 11：一张表可以不止一片）。不看 `TransactionOutput::units`——那是这个进程内存里的角色列表，
@@ -1016,7 +1048,7 @@
         .any(|identity| !covered.contains(identity))
         && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS
     {
-        let next = publish_version(
+        let published = publish_version(
             &mut pool,
             allocator,
             PublishPlan {
@@ -1037,18 +1069,29 @@
                 rollback_floor: new_floor,
             },
             Some(&*current),
-        )
-        // 抬 F 的写入口是这里开的、随错一起丢掉：已经落盘的那几次各自的写与失败那一次已记的写都随错交出（增补 2 收口表第 58 行）。
-        .map_err(|cause| {
-            MountError::RaiseFloorSequencePublishFailed(publish_sequence_failed(
-                &pool,
-                publishes
-                    .iter()
-                    .map(|persisted: &TransactionOutput| persisted.writes.clone())
-                    .collect(),
-                cause,
-            ))
-        })?;
+        );
+        let next = match published {
+            Ok(next) => next,
+            Err(cause) => {
+                // 第一次空发布在任何写之前被拒（分配器上没有冻结着它）：这一串什么都没落盘，分配器换回抬 F 之前的那一份。
+                // 落盘途中失败的那一次冻结在分配器上等原样重发（它的字节按回收之后的账装的），不换；第二次起被拒的，
+                // 前面已落盘的根带着新 F 与回收之后的账，扣住位照旧留到 F 生效（C516（抬 F 那一串发布被拒时前面几次已落盘））。
+                if publishes.is_empty() && allocator.frozen_publish().is_none() {
+                    *allocator = allocator_before_the_raise.clone();
+                }
+                // 抬 F 的写入口是这里开的、随错一起丢掉：已经落盘的那几次各自的写与失败那一次已记的写都随错交出（增补 2 收口表第 58 行）。
+                return Err(MountError::RaiseFloorSequencePublishFailed(
+                    publish_sequence_failed(
+                        &pool,
+                        publishes
+                            .iter()
+                            .map(|persisted: &TransactionOutput| persisted.writes.clone())
+                            .collect(),
+                        cause,
+                    ),
+                ));
+            }
+        };
         let device = device_of_txg(next.root.checkpoint_txg);
         if !covered.contains(&device) {
             covered.push(device);
@@ -1492,12 +1535,12 @@
 /// - 读不出、自证不过的槽按「可能住着这样一条根」算，与 D18（块里携带什么信息） 已定项 11 实例表行回收的根环条件同一个读法：
 ///   一个暂时读不出的槽能让条目被删、槽恢复之后被抛弃的根回到择根的候选里，就是 C332 那一格。代价：根环有一个槽持续读不出时条目永远删不掉。
 fn rollback_witness_entries_still_needed(
-    witness: &RollbackWitness,
+    entries: &[RollbackWitnessEntry],
     every_ring_root: Option<&[RootRecord]>,
 ) -> Vec<RollbackWitnessEntry> {
-    witness
-        .entries()
-        .into_iter()
+    entries
+        .iter()
+        .copied()
         .filter(|entry| match every_ring_root {
             None => true,
             Some(roots) => roots.iter().any(|root| {
@@ -1508,17 +1551,82 @@
         .collect()
 }
 
+/// 回退见证删除规则的第二条（D23（journal 的角色与格式） 已定项 14「回退见证的实现取法」①，代码轮第二轮判决第四节第 1 条，
+/// 主 agent 2026-09-25 定，被攻过零轮）：条目 (N1, r, T) 被同一张表里另一条 (N2, r2, T2) 罩住 ⟺ N2 ≥ N1 且 (r2, T2) ≤ (r, T)
+/// （实例代号为主比），罩住的删掉。被罩住的那条抛弃的每一处 (i, t)——(r, T) < (i, t) 且 i < N1——罩住它的那条也抛弃：
+/// (r2, T2) ≤ (r, T) < (i, t) 且 i < N1 ≤ N2，所以删掉之后整张表的并集判法逐字不变，这一条不靠测。
+/// 只在同一张要写出去的表里比：罩住它的那条与它一起落盘，不拿一条还没落盘的条目去删已经落盘的那一条。
+/// 两两比较的上界是表的条数（至多 R × S − 1 = 47）的平方。
+fn rollback_witness_entries_not_covered_by_another(
+    entries: &[RollbackWitnessEntry],
+) -> Vec<RollbackWitnessEntry> {
+    entries
+        .iter()
+        .copied()
+        .filter(|entry| {
+            !entries.iter().any(|other| {
+                other != entry
+                    && other.new_instance >= entry.new_instance
+                    && (other.rollback_target_instance, other.rollback_target_txg)
+                        <= (entry.rollback_target_instance, entry.rollback_target_txg)
+            })
+        })
+        .collect()
+}
+
+/// 按所选根指着的实例表里的回退行，推出每一次回退该有的见证条目（D23（journal 的角色与格式） 已定项 14「回退见证的实现取法」⑥，
+/// 代码轮第二轮判决第四节第 3 条，主 agent 2026-09-25 定，被攻过零轮）：回退行 (r_old, T_old, ·, 回退) 对应条目 (N, r_old, T_old)，
+/// N 是做那次回退的实例——回退那一次挂载写的行是 [r_old, N)：回退行，与中间实例各一行 (i, 0, 0)；N 自己那一行由下一次挂载写，
+/// T 是 N 在那次挂载里择到的根的 txg，恒大于 0（能指着这张表的根只有 N 与之后实例的根）。所以 N = 回退行之后第一条 T ≠ 0 的行的实例；
+/// 回退行之后只有 (i, 0, 0) 或一行都没有 ⇒ 这张表是 N 自己写行那一次写的，N = 所选根自己的实例。
+///
+/// ⚠️ 派发规格的原话是「N 取实例表里回退行之后第一个有行的实例」：回退跨过实例时（例如固定脚本里实例 2 的世界回退到 (1, 3)、
+/// 新实例 3 写行 (1, 3, 0, 回退) 与 (2, 0, 0)）照字面取到的是中间实例 2，条目 (2, 1, 3) 罩不住实例 2 的根 C——落回 C 正是这一条要挡的；
+/// 实例 2 若自己也做过一次回退，(2, …) 还会与盘上那一条说两种目标（I-7.10）。这里取「T ≠ 0」那一读，交主 agent 定。
+///
+/// 回退到 mkfs 的第 0 代根（r_old = 0）不写行（D18（块里携带什么信息） 已定项 11：实例 0 不写行），实例表里没有回退行，这一条推不出来。
+fn rollback_witness_entries_recovered_from_the_instance_table(
+    rows_of_the_chosen_roots_instance_table: &[InstanceRow],
+    chosen_root_instance: InstanceGeneration,
+) -> Vec<RollbackWitnessEntry> {
+    rows_of_the_chosen_roots_instance_table
+        .iter()
+        .filter(|row| row.is_rollback)
+        .map(|rollback_row| {
+            let new_instance = rows_of_the_chosen_roots_instance_table
+                .iter()
+                .filter(|later_row| {
+                    later_row.instance > rollback_row.instance
+                        && later_row.selected_root_txg != CheckpointTxg(0)
+                })
+                .map(|later_row| later_row.instance)
+                .min()
+                .unwrap_or(chosen_root_instance);
+            RollbackWitnessEntry {
+                new_instance,
+                rollback_target_instance: rollback_row.instance,
+                rollback_target_txg: rollback_row.selected_root_txg,
+            }
+        })
+        .collect()
+}
+
 /// 这次挂载要写的两张回退见证表：取号那两次写带删过的旧表（回退那一条还不在里面：见证随写行之后的第一次系统配置轮换写，
 /// D23（journal 的角色与格式） 已定项 14「回退见证」，写序 post），写行那次发布的轮换起带再加上这一次回退那一条的表（普通挂载两张相同）。
-/// 盘上原有的见证读自各盘择到的那一槽（`recovery::rollback_witness_of_the_pool`），按删除规则删过（`rollback_witness_entries_still_needed`）。
+/// 盘上原有的见证读自各盘择到的那一槽（`recovery::rollback_witness_of_the_pool`）；删除规则之前先按所选根实例表里的回退行把缺的条目补回来
+/// （`rollback_witness_entries_recovered_from_the_instance_table`：回退那一次挂载崩在写行的根落了、轮换还没落的那一格，
+/// 条目没落盘而回退行已经在了），再按删除规则删（`rollback_witness_entries_still_needed`，
+/// 与被同一张表里另一条罩住的删 `rollback_witness_entries_not_covered_by_another`）。
 ///
 /// # Errors
 /// 表装不下（条数越过 R × S − 1）⇒ `MountError::Recovery(RecoveryFailure::RollbackWitnessTableFullWhoseHandlingIsUndecided)`：
-/// 条款说表写不满（上限只由根环几何定），那是按「根环每个槽都读得出」推的；删除规则把读不出的槽按「可能住着被抛弃的根」算，
-/// 有槽持续读不出时条目删不掉、表就写得满——那时怎么办条款没写 ⇒ 第一版不支持，在任何写之前返回，盘上逐字节不变。
+/// 有槽持续读不出，或每次回退都崩在写行轮换之后、暖机之前（根环全读得出也一样，代码轮第二轮判决第三节）时条目删不掉、表写得满——
+/// 那时怎么办条款没写（C547（回退见证表的删除规则与写满没有条款））⇒ 第一版不支持，在任何写之前返回，盘上逐字节不变。
 fn rollback_witness_tables_of_this_mount<Device: BlockDevice>(
     devices: &[(DeviceIdentity, Device)],
     system_configuration: &crate::system_configuration::SystemConfiguration,
+    chosen_root: &RootRecord,
+    rows_of_the_chosen_roots_instance_table: &[InstanceRow],
     previous_row: &PreviousInstanceRow,
     instance_to_acquire: InstanceGeneration,
 ) -> Result<(RollbackWitnessTable, RollbackWitnessTable), MountError> {
@@ -1529,8 +1637,20 @@
             .root_ring_slots_per_region,
     );
     let witness = rollback_witness_of_the_pool(devices, system_configuration);
+    let with_the_recovered_entries: BTreeSet<RollbackWitnessEntry> = witness
+        .entries()
+        .into_iter()
+        .chain(rollback_witness_entries_recovered_from_the_instance_table(
+            rows_of_the_chosen_roots_instance_table,
+            chosen_root.instance,
+        ))
+        .collect();
     let every_ring_root = every_root_ring_slot_holds_a_root(devices, system_configuration);
-    let kept = rollback_witness_entries_still_needed(&witness, every_ring_root.as_deref());
+    let still_needed = rollback_witness_entries_still_needed(
+        &with_the_recovered_entries.into_iter().collect::<Vec<_>>(),
+        every_ring_root.as_deref(),
+    );
+    let kept = rollback_witness_entries_not_covered_by_another(&still_needed);
     let this_rollback = previous_row.is_rollback.then_some(RollbackWitnessEntry {
         new_instance: instance_to_acquire,
         rollback_target_instance: previous_row.instance,
@@ -1546,9 +1666,13 @@
     };
     let before_the_row_publish =
         RollbackWitnessTable::of_entries(kept.iter().copied(), capacity).map_err(full)?;
-    let from_the_row_publish =
-        RollbackWitnessTable::of_entries(kept.into_iter().chain(this_rollback), capacity)
-            .map_err(full)?;
+    let from_the_row_publish = RollbackWitnessTable::of_entries(
+        rollback_witness_entries_not_covered_by_another(
+            &kept.into_iter().chain(this_rollback).collect::<Vec<_>>(),
+        ),
+        capacity,
+    )
+    .map_err(full)?;
     Ok((before_the_row_publish, from_the_row_publish))
 }
 
@@ -1576,6 +1700,38 @@
         start.previous.instance_table_chain(),
         &rows_written,
     );
+    // 空间准入（D28（挂载期承诺量） 已定项 1 的式子逐设备合取；C363 (b) 判决第四节第 2 条接进可写挂载）：可写挂载的准入要
+    // 「实例切换的预留拿得到」（D2（RAID 条带策略） 已定项 13），预留是式子里的挂载期承诺量——每块盘上连它与别的八项都扣完，
+    // 可用(d) ≥ 0（这一刻不另要需求：写行与暖机那一串的空间就在切换预留与 checkpoint 保留池里）。rows0 = 挂载时读到的行数加写行那次
+    // 要写的行数（D28（挂载期承诺量） 已定项 3），挂载期间常量，记在分配器上给这次挂载之后的每次发布用。在取号之前拒，盘上逐字节不变。
+    // 只供测试的开关（`SpaceAdmission::SkippedByTheTestOnlySwitch`）关掉准入时不判，这次挂载之后的发布也不判（开关装在分配器上）。
+    allocator.record_instance_rows_after_this_mounts_row_publish(
+        u64::try_from(instance_table_rewrite.rows.len()).expect("实例表的行数装得进 u64"),
+    );
+    allocator.set_space_admission(start.space_admission);
+    match start.space_admission {
+        SpaceAdmission::JudgedByTheFormula => {
+            let no_demand_on_any_device: Vec<DemandOnDevice> = allocator
+                .devices
+                .iter()
+                .map(|device_map| DemandOnDevice {
+                    device: device_map.device,
+                    bytes: BytesOnOneDevice::ZERO,
+                })
+                .collect();
+            admit_on_every_device(
+                &admission_reading_before_a_publish(&allocator, start.previous.file_version()),
+                &no_demand_on_any_device,
+            )
+            .map_err(
+                |refusal| MountError::SpaceAdmissionRefusedBeforeAcquisition {
+                    instance_to_acquire,
+                    refusal,
+                },
+            )?;
+        }
+        SpaceAdmission::SkippedByTheTestOnlySwitch => {}
+    }
     // 取号之后要发的那一串（写行一次、暖机那几次）在取号之前整串预演一遍（分配器的拷贝上，走发布路径落盘之前那一段，
     // 与后面真发读同一个分配器——取号不碰它）：释放核验、树的形状、落点取不到，都在任何写之前返回。
     // 落点取不到怎么算进取号之前的准入，条款没定（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided` 的文档注释）。
@@ -1630,6 +1786,8 @@
         rollback_witness_tables_of_this_mount(
             pool.devices,
             &start.system_configuration,
+            &start.chosen_root,
+            &start.rows_of_the_chosen_roots_instance_table,
             &start.previous_row,
             instance_to_acquire,
         )?;
@@ -1791,13 +1949,26 @@
 /// 环里没有记录）同样走这条路：取号 1、不写行、零单元的发布推到本实例的根覆盖每块盘，第一个文件版本接在 `current` 后面。
 ///
 /// # Errors
-/// 恢复失败、所选根下有文件而环里一条记录都没有、实例表沿链读不出或解不开、取号之前的准入不过（分配记录树、记账树、
-/// 实例表多于一片）、取号之后那一串在分配器的拷贝上取不到落点（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`）、
-/// 取号失败、发布失败。
+/// 恢复失败、所选根下有文件而环里一条记录都没有、实例表沿链读不出或解不开、取号之前的空间准入不过
+/// （`SpaceAdmissionRefusedBeforeAcquisition`）、取号之前的预演报错、取号之后那一串在分配器的拷贝上取不到落点
+/// （`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`）、取号失败、发布失败。
 pub fn mount_writable<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
 ) -> Result<Mounted, MountError> {
+    mount_writable_with_space_admission(parameters, devices, SpaceAdmission::JudgedByTheFormula)
+}
+
+/// 同 [`mount_writable`]，判不判空间准入由调用方给（只供测试的开关 `SpaceAdmission`；产品路径走 `mount_writable`，恒判）。
+/// 开关装在交回的分配器上，这次挂载之后的发布照它判不判。
+///
+/// # Errors
+/// 同 [`mount_writable`]；`SkippedByTheTestOnlySwitch` 时不报 `SpaceAdmissionRefusedBeforeAcquisition`。
+pub fn mount_writable_with_space_admission<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    space_admission: SpaceAdmission,
+) -> Result<Mounted, MountError> {
     let system_configuration = choose_system_configuration(&*devices)?;
     let chosen_root =
         choose_root(&*devices, &system_configuration).ok_or(RecoveryFailure::NoValidRoot)?;
@@ -1859,6 +2030,8 @@
         applied_transaction_high_water: journal.maximum_applied_transaction,
         is_rollback: false,
     };
+    let rows_of_the_chosen_roots_instance_table =
+        previous.instance_table_chain().records.rows.clone();
     establish_instance(
         parameters,
         devices,
@@ -1876,6 +2049,8 @@
             abandoned_roots_unreadable,
             shadow_ledger: ShadowLedger::On,
             system_configuration,
+            rows_of_the_chosen_roots_instance_table,
+            space_admission,
         },
     )
 }
@@ -1886,14 +2061,34 @@
 /// 新实例的第一条 jsn 接在环里最大的 jsn 之后（C340（回退之后记录链从哪条之后接没有定义） 取 P2，预想、等用户定）。
 ///
 /// # Errors
-/// 目标根不在根环、不在候选集、它自己那条记录读不出、取号之前的准入不过（同可写挂载，实例表按 R_old 那一版算）、
-/// 取号之后那一串在分配器的拷贝上取不到落点（同可写挂载）、取号或发布失败。
+/// 目标根不在根环、不在候选集、它自己那条记录读不出、取号之前的空间准入不过（同可写挂载，实例表按 R_old 那一版算）、
+/// 取号之前的预演报错、取号之后那一串在分配器的拷贝上取不到落点（同可写挂载）、取号或发布失败。
 pub fn mount_rollback<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
     target: RollbackTarget,
     shadow_ledger: ShadowLedger,
 ) -> Result<Mounted, MountError> {
+    mount_rollback_with_space_admission(
+        parameters,
+        devices,
+        target,
+        shadow_ledger,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+}
+
+/// 同 [`mount_rollback`]，判不判空间准入由调用方给（只供测试的开关 `SpaceAdmission`，同 [`mount_writable_with_space_admission`]）。
+///
+/// # Errors
+/// 同 [`mount_rollback`]；`SkippedByTheTestOnlySwitch` 时不报 `SpaceAdmissionRefusedBeforeAcquisition`。
+pub fn mount_rollback_with_space_admission<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    target: RollbackTarget,
+    shadow_ledger: ShadowLedger,
+    space_admission: SpaceAdmission,
+) -> Result<Mounted, MountError> {
     let system_configuration = choose_system_configuration(&*devices)?;
     let newest_root =
         choose_root(&*devices, &system_configuration).ok_or(RecoveryFailure::NoValidRoot)?;
@@ -2036,6 +2231,8 @@
             abandoned_roots_unreadable,
             shadow_ledger,
             system_configuration,
+            rows_of_the_chosen_roots_instance_table: newest_table.rows,
+            space_admission,
         },
     )
 }
diff -ruN -x target /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-core/src/transaction.rs tree/crates/singlefs-core/src/transaction.rs
--- /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-core/src/transaction.rs	2026-09-24 22:04:35.929349323 +0000
+++ tree/crates/singlefs-core/src/transaction.rs	2026-09-24 23:55:14.709111916 +0000
@@ -23,6 +23,10 @@
     CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
     SlotNumber, TreeIdentifier,
 };
+use crate::admission::{
+    admission_reading_before_a_publish, admit_on_every_device, demand_of_the_roles_on_each_device,
+    AdmissionRefusedOnSomeDevices, BytesOnOneDevice, SpaceAdmission,
+};
 use crate::allocation_record_tree::{
     build_allocation_record_tree_node, children_of, nodes_holding_records,
     nodes_whose_contents_changed, records_of_each_leaf, AllocationRecordTreeGeometry,
@@ -2190,9 +2194,9 @@
 /// 「真坏的份数」与「为了两块盘的账对称一起留下的好份」。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum QuarantinedCopyReading {
-    /// 读出来了，整单元 CRC-32C 与位置项里的对不上。
+    /// 第一次没对上（读不出或对不上），重读那一次读出来了、整单元 CRC-32C 与位置项里的仍对不上。
     ChecksumMismatch,
-    /// 第一次读与重读一次都读不出（只重读一次，用户 2026-09-24 定）。
+    /// 第一次没对上（读不出或对不上），重读那一次读不出（只重读一次，用户 2026-09-24 定）。
     UnreadableAfterOneReread,
     /// 这一份对得上，同一个单元别的盘上那一份对不上或读不出：两块盘一起留在已分配（用户 2026-09-25 定）。
     IntactButAnotherCopyFailed,
@@ -2678,7 +2682,8 @@
 /// 释放之前按映射条目的位置项读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1，用户 2026-09-23 定案）：
 /// 这次重写的角色里经映射释放的那几个（[`mapped_roles_released_by_this_publish`]），映射条目的每条位置项指的那一份
 /// 整单元读出来算 CRC-32C、与位置项里的比（写的时候位置项的校验和就是整单元 CRC-32C，`make_filesystem::location_entries`）。
-/// **读盘本身失败先重读一次，还读不出就按对不上处置**（用户 2026-09-24 定案，C394 三问的第一问）：只重读一次，不退避、不多次重试。
+/// **读盘本身失败、或读出来核出对不上，都先重读一次，两次都没对上才按对不上处置**（读不出那一半用户 2026-09-24 定案，C394 三问的第一问；
+/// 对不上那一半主 agent 2026-09-25 定，D19（块指针的结构与宽度预算） 已定项 5）：只重读一次，不退避、不多次重试。
 /// **任一份核出对不上（读不出也算），这个单元在每块盘上的分配记录都留在「已分配」**、不改成已释放——另一块盘上那一份对得上也一起留，
 /// 各盘的账保持对称（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-25 定）：交回的是这些单元在池里每块盘上的那一份，
 /// 按角色次序、位置项的次序（设备身份升序），每一份带着它自己读出来的样子（[`QuarantinedCopyReading`]）；发布照常释放（映射条目去掉），
@@ -2714,13 +2719,27 @@
                         unit_bytes,
                     )
                 };
-                // 读不出先重读一次（只一次）；还读不出就按对不上处置。
-                let reading = match read_the_copy().or_else(read_the_copy) {
-                    None => QuarantinedCopyReading::UnreadableAfterOneReread,
+                let check_the_copy_against_its_location_entry = |copy: Option<Vec<u8>>| match copy {
+                    None => CopyCheck::Unreadable,
                     Some(copy) if crc32_castagnoli(&copy) != location.unit_checksum => {
-                        QuarantinedCopyReading::ChecksumMismatch
+                        CopyCheck::ChecksumMismatch
+                    }
+                    Some(_) => CopyCheck::Intact,
+                };
+                // 第一次读不出、或读出来核出对不上，都先重读一次（只一次，D19（块指针的结构与宽度预算） 已定项 5
+                // 「硬规则 1 的读盘核读不出、核出对不上时怎么办」）：一次瞬时坏读不许让单元永久隔离。
+                // 重读那一次对得上就算这一份对得上；两次都没对上，按重读那一次的样子报。
+                let reading = match check_the_copy_against_its_location_entry(read_the_copy()) {
+                    CopyCheck::Intact => QuarantinedCopyReading::IntactButAnotherCopyFailed,
+                    CopyCheck::Unreadable | CopyCheck::ChecksumMismatch => {
+                        match check_the_copy_against_its_location_entry(read_the_copy()) {
+                            CopyCheck::Intact => QuarantinedCopyReading::IntactButAnotherCopyFailed,
+                            CopyCheck::Unreadable => {
+                                QuarantinedCopyReading::UnreadableAfterOneReread
+                            }
+                            CopyCheck::ChecksumMismatch => QuarantinedCopyReading::ChecksumMismatch,
+                        }
                     }
-                    Some(_) => QuarantinedCopyReading::IntactButAnotherCopyFailed,
                 };
                 (*location, reading)
             })
@@ -2751,6 +2770,14 @@
     Ok(quarantined)
 }
 
+/// 释放之前读盘核一份的一次结果（[`copies_failing_the_release_checksum_check`]）：读不出、读出来整单元 CRC-32C 与位置项里的对不上、对得上。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum CopyCheck {
+    Unreadable,
+    ChecksumMismatch,
+    Intact,
+}
+
 /// 一个被换下的单元的落点：两条位置条目同槽，池里每块盘各有一条在册、未释放、跨度对得上的记录；有一样不对就报错、不交回落点。
 ///
 /// **池里每块盘都核一遍，不只核 `locations[0].device` 那一块**：`PoolAllocator::release` 对每块盘都要求
@@ -2857,6 +2884,12 @@
         unit: TransactionUnit,
         refusal: PlacementRefusal,
     },
+    /// 空间准入不够（D28（挂载期承诺量） 已定项 1 的式子逐设备合取；C363 (b) 判决 `research/prompts/c363b-r1-main-verification.md`
+    /// 第四节第 2 条把它接进发布路径）：这次发布的普通分配（用户数据单元、extent 树与 inode 树的节点，`admission::space_budget_of_role`）
+    /// 在至少一块盘上多于可用(d)，带着不够的每一块。在算定这次发布的样子之后、读盘核被换下的单元与动分配器之前返回，
+    /// 一个写都没发、盘上逐字节不变。准入不够时先推空发布抬 F 再判（D16（发布语义） 已定项 1，C283（准入失败时不先推发布就报 ENOSPC））
+    /// 没有实现：调用方收到的就是这一次的拒绝。
+    SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices),
     /// 记账树或中央映射树这次之后的形状算不出来（`code_two_tree::plan_the_tree_after_this_publish`）：要长到 257 层、
     /// 码 2 头的层级 1 字节写不下（多层之后映射树容量准入剩下的唯一一条，D19（块指针的结构与宽度预算） 已定项 5），
     /// 或上一版（从盘上重建的）树按分隔 key 走不到它自己叶里的一把 key。在动分配器、发任何一个写之前返回，盘上逐字节不变。
@@ -4520,6 +4553,27 @@
         pool.code_two_tree_node_capacities(),
         allocator,
     )?;
+    // 空间准入（D28（挂载期承诺量） 已定项 1 的式子逐设备合取）：读数取分配器此刻的计数——这次的释放与取落点都还没做——，
+    // 需求只算这次的普通分配（`admission::space_budget_of_role`）；一个普通分配都没有的发布（空发布、写行）不判，
+    // 它们的空间在保留池与切换预留里（`admission` 的模块文档）。在读盘核与动分配器之前拒，盘上逐字节不变。
+    // 只供测试的开关关掉准入时（`SpaceAdmission::SkippedByTheTestOnlySwitch`）不判，留给落点那一道在任何写之前拒。
+    let demand = demand_of_the_roles_on_each_device(
+        &settled.resolved.rewritten_roles,
+        &device_identities_of_the_accounting_rows,
+    );
+    let demand_is_judged = match allocator.space_admission() {
+        SpaceAdmission::JudgedByTheFormula => demand
+            .iter()
+            .any(|demand_on_device| demand_on_device.bytes != BytesOnOneDevice::ZERO),
+        SpaceAdmission::SkippedByTheTestOnlySwitch => false,
+    };
+    if demand_is_judged {
+        admit_on_every_device(
+            &admission_reading_before_a_publish(allocator, previous),
+            &demand,
+        )
+        .map_err(PublishError::SpaceAdmissionRefused)?;
+    }
     // 经映射核到的那几个单元，释放之前再按位置项读盘核一次校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）：只读，
     // 读不到就在动分配器、发任何一个写之前返回。第一个文件版本换下的 mkfs 那片树表与树表 0 条那一版的分配记录树节点不经映射，不核。
     let quarantine = match (previous, release_checksum_check) {
diff -ruN -x target /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs tree/crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs
--- /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	2026-09-24 21:36:05.371480421 +0000
+++ tree/crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs	2026-09-25 00:46:12.032594828 +0000
@@ -13,12 +13,12 @@
 //! 统计量行），不读内存里的 `PoolAllocator`；`allocated_minus_deferred_matches_referenced` 保留成内存读法，
 //! 只给 U8 的变异反面用（`crates/mutations.tsv` M21）。
 
-use std::collections::HashMap;
+use std::collections::{BTreeMap, BTreeSet, HashMap};
 
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{
-    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
 };
 use singlefs_core::allocator::{
     DeviceFreeMap, Placement, PoolAllocator, ReclaimedReuse, ReuseWindow,
@@ -45,7 +45,8 @@
     InstanceTablePlan, PoolWriter, PublishError, PublishPlan, TransactionOutput, TransactionUnit,
 };
 use singlefs_format::{
-    JOURNAL_RING_DEFAULT_BYTES, ROOT_RING_REGIONS, SLOT_BYTES, UNIT_AREA_START_SLOT,
+    CLUSTER_SEGMENT_SLOTS, JOURNAL_RING_DEFAULT_BYTES, ROOT_RING_REGIONS, SLOT_BYTES,
+    UNIT_AREA_START_SLOT,
 };
 use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
 
@@ -71,15 +72,26 @@
 /// `⌊(16384 − 135) ÷ 20⌋`；节点头 135 字节 = 86（固定头）+ 2 × 10（子节点指针，容量算式的一部分）+ 29（其余头字段）。
 const E156_ALLOCATION_RECORD_NODE_HEADER_BYTES: u64 = 86 + 2 * 10 + 29;
 const E156_ALLOCATION_RECORD_BYTES: u64 = 20;
-/// S1(e)：一次空发布 E 在 D0 上自己的释放槽数。抄自 `research/prompts/e156-preregistration.md` 第 431 行引用的
-/// E153 登记 M0.4；对不上就是「十一」S1 的停机条款，不是这一份自己定的门槛。
-const E156_EMPTY_PUBLISH_EXPECTED_RELEASED_SLOTS: u64 = 4;
-/// S1(e)：一次覆盖写 O 在 D0 上自己的释放槽数。抄自同一份 P11 修订 3（`503 = 13 + 49×10`，逐步都精确成立）。
-const E156_OVERWRITE_EXPECTED_RELEASED_SLOTS: u64 = 10;
-/// S1(f)：一次覆盖写让分配记录条数（D0 + D1 合计）增加多少。抄自登记「四」P18（「每次覆盖写每盘 + 8（共 16）」）。
-const E156_OVERWRITE_EXPECTED_RECORD_DELTA: usize = 16;
-/// S1(f)：第一个事务之后的分配记录条数。抄自登记「四」P18。
-const E156_FIRST_TRANSACTION_EXPECTED_RECORD_COUNT: usize = 20;
+/// S1(f)：第一个事务之后的分配记录条数。E156 第 3 次重跑登记「七」7.2 K1-2（分配记录树按位置寻址之后，
+/// 五个树节点各占一条记录，28 = 2 × 14）；不再是登记第一、二版的 20（那时树只占 1 条记录）。
+const E156_FIRST_TRANSACTION_EXPECTED_RECORD_COUNT: usize = 28;
+/// R-3 本地常量①：分配记录树叶宽 W。**本地常量，值抄自 kb，不从 `crates/` 引**（`.claude/agents/
+/// experiment-runner.md` 入库装置第 ① 条）：抄自 `.claude/kb/decisions/08-核心索引结构.md:251`
+/// `<!-- format-const: ALLOCATION_RECORD_TREE_LEAF_SLOTS = 812 -->`。下面 `anchor_a_d8` 那一行把它
+/// 与 `singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS` 回比（只观测，F21：对不上不作废、不停机）。
+const E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
+/// R-3 本地常量②：分配记录树内部扇出 F。抄自 `.claude/kb/decisions/08-核心索引结构.md:254`
+/// `<!-- format-const: ALLOCATION_RECORD_TREE_INTERNAL_FANOUT = 169 -->`。
+const E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
+/// R-3：一次覆盖写里，非分配记录树自身的「其余单元」在 D0 上的释放槽数（E156 第 3 次重跑登记「七」7.2
+/// R-3：数据 2 + extent 1 + inode 叶 2 + inode 根 1 + 记账 1 + 映射 1 + 树表 1 = 9）。
+const E156_OVERWRITE_OTHER_UNITS_RELEASED_SLOTS: u64 = 9;
+/// R-3：同一批「其余单元」里会产生新分配记录条目的槽数（不含数据：数据槽被 `data_slot()` 复用同一个
+/// 已有的记录条目，不产生新条目，登记「七」7.2 R-4 的算术）。
+const E156_OVERWRITE_OTHER_UNITS_NEW_RECORD_SLOTS: u64 = 7;
+/// R-3：一次空发布里，非分配记录树自身的「其余单元」（记账 1 + 映射 1 + 树表 1 = 3，三者都走 bump、
+/// 每次都是新槽，释放与新增同值，登记「七」7.2 R-5）。
+const E156_EMPTY_PUBLISH_OTHER_UNITS_SLOTS: u64 = 3;
 
 fn parameters() -> MakeFilesystemParameters {
     MakeFilesystemParameters {
@@ -114,6 +126,160 @@
     }
 }
 
+// ============================================================================================
+// R-3（E156 第 3 次重跑登记「七」7.2）：分配记录树按位置寻址之后，节点数不再是常数，只能从「这次发布
+// 实际改动的记录的槽号」现算。这一段只用 [`E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS`] /
+// [`E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`] 两个本地常量与 `allocator.records()` 的真实读数，
+// 不调用 `crates/singlefs-core::allocation_record_tree` 的任何几何函数——那些函数就是被测的实装本身，
+// 拿它们来算「期望值」会让期望值与实装共用同一处错误。
+// ============================================================================================
+
+/// 一个槽所在的分配记录树叶位置 ⌊s ÷ W⌋（本地常量 W）。
+fn e156_leaf_position_of_slot(slot: u64) -> u64 {
+    slot / E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS
+}
+
+/// 一个叶位置所在的层级 1 位置 ⌊k ÷ F⌋（本地常量 F）。
+fn e156_level1_position_of_leaf(leaf: u64) -> u64 {
+    leaf / E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
+}
+
+/// R-3：一次发布里分配记录树自身新写的节点数 = `devices × 叶数 + devices × 层级 1 数 + 1`（根只有一份，
+/// 两块盘共享；叶与层级 1 逐盘各一份，登记「七」7.2 R-3 逐字）。
+fn e156_allocation_record_tree_new_node_count(touched_leaves: &BTreeSet<u64>, devices: u64) -> u64 {
+    let touched_level1: BTreeSet<u64> = touched_leaves
+        .iter()
+        .map(|&leaf| e156_level1_position_of_leaf(leaf))
+        .collect();
+    devices * u64::try_from(touched_leaves.len()).expect("叶数落在 u64 内")
+        + devices * u64::try_from(touched_level1.len()).expect("层级 1 数落在 u64 内")
+        + 1
+}
+
+/// R-3：这次发布里被换下（COW 释放）的分配记录树旧节点数——只有这次触达、且这个位置在这次发布之前
+/// 已经有节点（叶位置 ∈ `existing_leaves`，或它的层级 1 位置 ∈ 现有层级 1 集合）的那些才会释放旧版本。
+fn e156_allocation_record_tree_replaced_node_count(
+    touched_leaves: &BTreeSet<u64>,
+    existing_leaves: &BTreeSet<u64>,
+    devices: u64,
+) -> u64 {
+    let touched_level1: BTreeSet<u64> = touched_leaves
+        .iter()
+        .map(|&leaf| e156_level1_position_of_leaf(leaf))
+        .collect();
+    let existing_level1: BTreeSet<u64> = existing_leaves
+        .iter()
+        .map(|&leaf| e156_level1_position_of_leaf(leaf))
+        .collect();
+    let leaves_kept = touched_leaves.intersection(existing_leaves).count();
+    let level1_kept = touched_level1.intersection(&existing_level1).count();
+    devices * u64::try_from(leaves_kept).expect("叶交集数落在 u64 内")
+        + devices * u64::try_from(level1_kept).expect("层级 1 交集数落在 u64 内")
+        + 1
+}
+
+/// R-3：D0 上这次发布之前，全部分配记录（不论已释放还是仍分配）所在的叶位置集合——「这个位置本来
+/// 有没有节点」的判据（D8（核心索引结构） 已定项 14「没有记录的一段 = 全空闲：那片叶不写」）。
+fn e156_existing_leaf_positions(allocator: &PoolAllocator, device: DeviceIdentity) -> BTreeSet<u64> {
+    allocator
+        .records()
+        .iter()
+        .filter(|record| record.device == device)
+        .map(|record| e156_leaf_position_of_slot(record.slot.0))
+        .collect()
+}
+
+/// R-3：D0 上这次发布翻成已释放或新加的全部记录所在的叶位置集合——真实记录的 `generation` 字段
+/// 在被触达的那一刻（无论新分配还是刚被释放）都改写成这次的 txg（D3（空间分配） 已定项 7 逐字），
+/// 用它筛出「这次改动的记录」不需要另外做前后快照 diff。
+fn e156_touched_leaf_positions(
+    allocator: &PoolAllocator,
+    device: DeviceIdentity,
+    txg: CheckpointTxg,
+) -> BTreeSet<u64> {
+    allocator
+        .records()
+        .iter()
+        .filter(|record| record.device == device && record.generation == txg)
+        .map(|record| e156_leaf_position_of_slot(record.slot.0))
+        .collect()
+}
+
+/// R-3：一次发布（覆盖写或空发布）D0 释放槽数与 D0+D1 记录增量的闭式期望值。
+/// `existing_leaves`：这次发布之前 D0 上已有记录的叶位置集合；`touched_leaves`：这次发布之后
+/// D0 上 `generation == 本次 txg` 的记录所在的叶位置集合。
+fn e156_closed_form_expected(
+    existing_leaves: &BTreeSet<u64>,
+    touched_leaves: &BTreeSet<u64>,
+    is_overwrite: bool,
+) -> (u64, u64) {
+    let new_nodes = e156_allocation_record_tree_new_node_count(touched_leaves, 2);
+    let replaced_nodes =
+        e156_allocation_record_tree_replaced_node_count(touched_leaves, existing_leaves, 2);
+    let (other_released, other_new_records) = if is_overwrite {
+        (
+            E156_OVERWRITE_OTHER_UNITS_RELEASED_SLOTS,
+            E156_OVERWRITE_OTHER_UNITS_NEW_RECORD_SLOTS,
+        )
+    } else {
+        (
+            E156_EMPTY_PUBLISH_OTHER_UNITS_SLOTS,
+            E156_EMPTY_PUBLISH_OTHER_UNITS_SLOTS,
+        )
+    };
+    (
+        other_released + replaced_nodes,
+        2 * (other_new_records + new_nodes),
+    )
+}
+
+/// R-7：一次推空发布（只改一片叶时）自己的固定点槽数——单设备的「新增记录」那一半，不乘 2（登记
+/// 「七」7.2 R-5「一次推空发布只改一片叶时的固定点 8 槽」，与 D0+D1 合计的记录增量相差一个 devices 因子）。
+fn e156_empty_publish_fixed_point_slots(existing_leaves: &BTreeSet<u64>) -> u64 {
+    E156_EMPTY_PUBLISH_OTHER_UNITS_SLOTS
+        + e156_allocation_record_tree_new_node_count(existing_leaves, 2)
+}
+
+/// R-3 本地常量③：根层级规则——最小的 R ≥ 1 使 Σ_盘 ⌈盘上槽数 ÷ (W × F^(R−1))⌉ ≤ F（D8（核心索引结构）
+/// 已定项 14「根罩整个 key 空间、按盘分流」那一行）。只用来与实装的 `AllocationRecordTreeGeometry`
+/// 回比（A-D8，只观测，F21：对不上不作废、不停机），不供 R-3 的节点数闭式使用。
+fn e156_root_level_for_symmetric_devices(unit_area_slots_per_device: u64, device_count: u64) -> u8 {
+    let mut level: u32 = 1;
+    loop {
+        let child_span = E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS
+            * E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT.pow(level - 1);
+        let total_cells = device_count * unit_area_slots_per_device.div_ceil(child_span);
+        if total_cells <= E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT {
+            return u8::try_from(level).expect("根层级落在 u8 内（池子不会大到需要 256 层）");
+        }
+        level += 1;
+    }
+}
+
+/// Q7d-1（R2 ⑤）：一组「这次发布自己的释放」样本的 (最小值, 最大值)——main() 与 U12 单测共用同一个
+/// 函数，M30（Q7d-1 覆盖写那一组退回字面 10）改这里，两处才会一起红。
+fn e156_minimum_and_maximum(samples: &[u64]) -> (u64, u64) {
+    (
+        samples.iter().min().copied().unwrap_or(0),
+        samples.iter().max().copied().unwrap_or(0),
+    )
+}
+
+/// R-7：一块盘开放聚簇段里当前的空闲槽数（登记「五」5.1 HY「e ≥ max(8, f)」的 e）。
+fn e156_open_segment_free_slots(allocator: &PoolAllocator, device: DeviceIdentity) -> u64 {
+    let Some(open_segment_start) = allocator.open_segment() else {
+        return 0;
+    };
+    let device_map = allocator
+        .devices
+        .iter()
+        .find(|map| map.device == device)
+        .expect("这块盘在池里");
+    (0..CLUSTER_SEGMENT_SLOTS)
+        .filter(|offset| device_map.is_free(SlotNumber(open_segment_start.0 + offset)))
+        .count() as u64
+}
+
 /// 覆盖写第 `step` 次的文件内容：定长 3000 字节，字节序列随 `step` 变，保证每次覆盖写都改变用户可见状态（骨架发布 的 O）。
 fn overwrite_content(step: u64) -> Vec<u8> {
     let salt = usize::try_from(step % 251).expect("对 251 取模落在 usize 里");
@@ -862,6 +1028,9 @@
     );
 
     // E（空发布）
+    let existing_leaves_before_empty_publish =
+        e156_existing_leaf_positions(&allocator, DeviceIdentity(0));
+    let records_before_empty_publish = allocator.records().len();
     current = publish_empty(
         parameters,
         devices.as_mut_slice(),
@@ -875,6 +1044,37 @@
         DeviceIdentity(0),
         current.root.checkpoint_txg,
     );
+    // R-5（第七节 7.2）：HK 那一次空发布 D0 释放槽数与记录增量的闭式核对，只在 `label == "HK"`（真实、
+    // 可达的那一臂）上钉硬断言；HK-F0 只观测，不 panic（它的可达性本来就不参与 Q7b/Q7d 的统计）。
+    let touched_leaves_empty_publish = e156_touched_leaf_positions(
+        &allocator,
+        DeviceIdentity(0),
+        current.root.checkpoint_txg,
+    );
+    let (expected_empty_publish_released, expected_empty_publish_record_delta) =
+        e156_closed_form_expected(
+            &existing_leaves_before_empty_publish,
+            &touched_leaves_empty_publish,
+            false,
+        );
+    let empty_publish_record_delta = u64::try_from(
+        allocator.records().len() - records_before_empty_publish,
+    )
+    .expect("空发布的记录增量落在 u64 内");
+    emitter.emit(&format!(
+        "name=r5_empty_publish_closed_form label={label} released_d0={empty_publish_release_slots} expected_released_d0={expected_empty_publish_released} record_delta={empty_publish_record_delta} expected_record_delta={expected_empty_publish_record_delta} fixed_point_slots_one_leaf={}",
+        e156_empty_publish_fixed_point_slots(&existing_leaves_before_empty_publish),
+    ));
+    if label == "HK" {
+        assert_eq!(
+            empty_publish_release_slots, expected_empty_publish_released,
+            "R-5：HK 空发布 D0 释放槽数应等于闭式"
+        );
+        assert_eq!(
+            empty_publish_record_delta, expected_empty_publish_record_delta,
+            "R-5：HK 空发布记录增量应等于闭式"
+        );
+    }
     record_legal_state(
         emitter,
         legal_states,
@@ -1365,8 +1565,9 @@
     );
     assert_eq!(
         remounted.output.isolated_slots_per_device,
-        vec![(DeviceIdentity(0), 34), (DeviceIdentity(1), 34)],
-        "S1c：按 D 那一版实例表判被抛弃的根引用的槽，普通重开照样隔离"
+        vec![(DeviceIdentity(0), 54), (DeviceIdentity(1), 54)],
+        "S1c：按 D 那一版实例表判被抛弃的根引用的槽，普通重开照样隔离（R-6，第七节 7.2：分配记录树\
+         按位置寻址之后每盘 54 槽，不再是原登记的 34）"
     );
     assert_eq!(
         remounted.output.abandoned_roots_unreadable, 0,
@@ -2014,6 +2215,17 @@
     emitter.emit(&format!(
         "name=x8a_prefix pool={pool_label} overwrite_count={overwrite_count} stop_reason={stop_reason} allocated={allocated} free={free} deferred={deferred}"
     ));
+    // R-7（第七节 7.2）：HY 的定义改成 e ≥ max(8, f)——e 是这一刻开放段里的空槽数，f 是一次推空发布
+    // （只改一片叶时）自己的固定点槽数，两者都现算，不再是「≥ 8」这句原文的字面。HX 上同样报这一行，
+    // 只作观测（HX 的定义是填到耗尽，这一条件在 HX 上多半不成立，不构成对照失败）。
+    let open_segment_free_slots = e156_open_segment_free_slots(&allocator, DeviceIdentity(0));
+    let existing_leaves_at_raise = e156_existing_leaf_positions(&allocator, DeviceIdentity(0));
+    let fixed_point_slots = e156_empty_publish_fixed_point_slots(&existing_leaves_at_raise);
+    let hy_threshold = fixed_point_slots.max(8);
+    emitter.emit(&format!(
+        "name=r7_hy_condition pool={pool_label} open_segment_free_slots={open_segment_free_slots} fixed_point_slots={fixed_point_slots} threshold={hy_threshold} holds={}",
+        open_segment_free_slots >= hy_threshold
+    ));
 
     // 探上限：与 HF 单格同一个办法。
     let mut probe_devices = devices_from_memory_pool(&memory_pool_snapshot(&devices));
@@ -2171,6 +2383,23 @@
     ));
 }
 
+/// R-7（第一节 R1、第七节 7.2）：HY 原来靠「只跑 3 次覆盖写」留出空间，`e ≥ 8` 从没被装置核过；
+/// 这一次改成 `e ≥ max(8, f)` 现核——3 次不满足就减少覆盖写次数直到满足（登记「五」5.1 逐字）。
+/// 只建前缀、量 `e`/`f`，不跑 `run_small_pool_cell` 的其余部分（避免为找 cap 打印一堆用不上的行）。
+fn e156_find_hy_cap_satisfying_open_segment_condition(parameters: &MakeFilesystemParameters) -> u64 {
+    for cap in (0..=3u64).rev() {
+        let (_devices, allocator, _instance, _current, _overwrite_count, _stop_reason) =
+            build_small_pool_prefix(parameters, Some(cap));
+        let open_segment_free_slots = e156_open_segment_free_slots(&allocator, DeviceIdentity(0));
+        let existing_leaves = e156_existing_leaf_positions(&allocator, DeviceIdentity(0));
+        let fixed_point_slots = e156_empty_publish_fixed_point_slots(&existing_leaves);
+        if open_segment_free_slots >= fixed_point_slots.max(8) {
+            return cap;
+        }
+    }
+    0
+}
+
 // ============================================================================================
 // 岔路 1（第 9 行）：Hh(k)，甲-T1（真实基线）与 G12（旁路评估）多扣的差随洞数怎么长。缩小范围：
 // ρ = 1、回收时点固定「实」、洞位置固定「后」；S 这一维跑 S = 8（mkfs 默认）与 S = 4（下界，方向相反，
@@ -2550,6 +2779,17 @@
         allocation_record_node_capacity, 812,
         "A7：分配记录节点容量应为 812"
     );
+    // A-D8（第七节 7.1）：本地叶宽 / 扇出与实装的 `singlefs_format` 同名常量回比，本地「根层级规则」
+    // 与实装的 `AllocationRecordTreeGeometry::of_allocator` 回比——只观测，不 assert：这两条出自被测
+    // 条款本身，对不上走 F21，不作废、不停机（第十节 F21、第七节 7.1 表头）。
+    let leaf_slots_match_format_crate = E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS
+        == singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS;
+    let internal_fanout_match_format_crate = E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
+        == singlefs_format::ALLOCATION_RECORD_TREE_INTERNAL_FANOUT;
+    let my_root_level = e156_root_level_for_symmetric_devices(unit_area_slot_count, 2);
+    emitter.emit(&format!(
+        "name=anchor_a_d8 local_leaf_slots={E156_ALLOCATION_RECORD_TREE_LEAF_SLOTS} local_internal_fanout={E156_ALLOCATION_RECORD_TREE_INTERNAL_FANOUT} local_leaf_slots_matches_format_crate={leaf_slots_match_format_crate} local_internal_fanout_matches_format_crate={internal_fanout_match_format_crate} my_root_level={my_root_level}"
+    ));
 
     // ===== mkfs + 取号 + 暖机 + 第一个事务：K1、S1(a)(g)、H0/HR/HK 共用的起点 =====
     let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
@@ -2606,6 +2846,17 @@
             span: 1,
         },
     );
+    // A-D8（续）：实装的根层级只由每块盘的槽数决定，与记录内容无关，mkfs 之后就能读——与上面本地
+    // 算出的 `my_root_level` 回比（只观测，F21）。
+    let real_root_level =
+        singlefs_core::allocation_record_tree::AllocationRecordTreeGeometry::of_allocator(
+            &allocator,
+        )
+        .root_level();
+    emitter.emit(&format!(
+        "name=anchor_a_d8_root_level my_root_level={my_root_level} real_root_level={real_root_level} matches={}",
+        my_root_level == real_root_level
+    ));
     let instance = {
         let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
         acquire_instance(&mut pool).expect("取号")
@@ -2631,16 +2882,18 @@
         .expect("第一个事务")
     };
 
-    // K1：txg 3 之后 D0 的记账第 1/2/5 项，与登记里钉的绝对值比对（S1(a)）。
+    // K1：txg 3 之后 D0 的记账第 1/2/5 项，与登记里钉的绝对值比对（S1(a)）。K1-1（第七节 7.2）：
+    // 分配记录树按位置寻址之后第 1 项为 17（不再是原登记的 13：多出的 4 槽是这五个树节点里比原来
+    // 单节点多出的那四个），第 5 项仍是 1。
     let (allocated0, free0, deferred0) = accounting_row_slots(&allocator, DeviceIdentity(0));
+    let k1_1_matches_registered_item1_of_17_and_item5_of_1 = allocated0 == 17 && deferred0 == 1;
     emitter.emit(&format!(
-        "name=k1_after_first_transaction txg={} allocated_slots={allocated0} free_slots={free0} deferred_slots={deferred0} registered_item1_slots=13 registered_item5_slots=1 matches_registered={}",
+        "name=k1_after_first_transaction txg={} allocated_slots={allocated0} free_slots={free0} deferred_slots={deferred0} registered_item1_slots=17 registered_item5_slots=1 matches_registered={k1_1_matches_registered_item1_of_17_and_item5_of_1}",
         current.root.checkpoint_txg.0,
-        allocated0 == 13 && deferred0 == 1,
     ));
     assert!(
-        allocated0 == 13 && deferred0 == 1,
-        "S1(a)：K1 应当逐字匹配登记"
+        k1_1_matches_registered_item1_of_17_and_item5_of_1,
+        "S1(a)：K1-1 应当逐字匹配登记（第七节 7.2，17/1）"
     );
     let mut placements: Vec<(u64, u64)> = current
         .units
@@ -2659,7 +2912,7 @@
     ));
 
     // S1(f)：第一个事务之后的分配记录条数（D0 + D1 合计）。
-    let mut previous_record_count = allocator.records().len();
+    let previous_record_count = allocator.records().len();
     emitter.emit(&format!(
         "name=s1f_record_count txg={} count={previous_record_count}",
         current.root.checkpoint_txg.0
@@ -2709,7 +2962,14 @@
 
     let s1d_cutoff = 3 * E156_SLOTS_PER_REGION + 6;
     let mut baseline_workload_actual_length: u64 = 0;
+    // Q3r.2（第七节 R-3）：闭式不再是常数，逐次核对，出不符不 panic（S1(e)(f) 现在是「两边都查」的
+    // 观测型停机，不是硬 panic：一次不符就让整轮产物都出不来，反而没法看后面每一步的读数）。
+    let mut overwrite_release_samples: Vec<u64> = Vec::new();
+    let mut s1ef_mismatches: u64 = 0;
+    let mut s1ef_leaf_count_histogram: BTreeMap<usize, u64> = BTreeMap::new();
     for step in 1..=baseline_workload_length {
+        let existing_leaves_before_step = e156_existing_leaf_positions(&allocator, DeviceIdentity(0));
+        let records_before_step = allocator.records().len();
         let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
         let outcome = publish_overwrite(
             &mut pool,
@@ -2733,24 +2993,40 @@
         };
         current = published;
         baseline_workload_actual_length = step;
-        // S1(e)：一次 O 自己的释放槽数应恒为 10。
+        // R-3a/R-3b（第七节 7.2）：一次 O 的释放槽数与记录增量按闭式逐次核对（Q3r.2）。
         let released = self_release_slots_of_this_publish(
             &allocator,
             DeviceIdentity(0),
             current.root.checkpoint_txg,
         );
-        assert_eq!(
-            released, E156_OVERWRITE_EXPECTED_RELEASED_SLOTS,
-            "S1(e)：第 {step} 次覆盖写自己的释放槽数应为 10"
+        let touched_leaves_this_step =
+            e156_touched_leaf_positions(&allocator, DeviceIdentity(0), current.root.checkpoint_txg);
+        let (expected_released, expected_record_delta) = e156_closed_form_expected(
+            &existing_leaves_before_step,
+            &touched_leaves_this_step,
+            true,
         );
-        // S1(f)：分配记录条数每次 O 应增 16。
         let record_count = allocator.records().len();
-        assert_eq!(
-            record_count,
-            previous_record_count + E156_OVERWRITE_EXPECTED_RECORD_DELTA,
-            "S1(f)：第 {step} 次覆盖写的分配记录条数增量应为 16"
-        );
-        previous_record_count = record_count;
+        let record_delta = u64::try_from(record_count - records_before_step)
+            .expect("这一步的记录增量落在 u64 内");
+        let step_matches = released == expected_released && record_delta == expected_record_delta;
+        if !step_matches {
+            s1ef_mismatches += 1;
+        }
+        let changed_internal_count = touched_leaves_this_step
+            .iter()
+            .map(|&leaf| e156_level1_position_of_leaf(leaf))
+            .collect::<BTreeSet<_>>()
+            .len();
+        *s1ef_leaf_count_histogram
+            .entry(touched_leaves_this_step.len())
+            .or_insert(0) += 1;
+        emitter.emit(&format!(
+            "name=s1ef_step step={step} txg={} changed_leaves={} changed_internal={changed_internal_count} released_d0={released} expected_released_d0={expected_released} record_delta={record_delta} expected_record_delta={expected_record_delta} matches={step_matches}",
+            current.root.checkpoint_txg.0,
+            touched_leaves_this_step.len(),
+        ));
+        overwrite_release_samples.push(released);
         let pool_snapshot = memory_pool_snapshot(&devices);
         // S1(d)：前 3S + 6 次逐次报第 1/2/5 项与今天两条检查（应当全绿：这些都是合法状态）。
         if step <= s1d_cutoff {
@@ -2779,6 +3055,16 @@
             "name=baseline_workload_completed_full_length length={baseline_workload_length}"
         ));
     }
+    // Q3r.2 汇总：H0 上逐次核对的不符次数与「改动落在几片叶」的直方图（第八节 8.2 要求两个方向都要有：
+    // 落在 1 片叶与落在 ≥ 2 片叶的步各至少一次）。
+    let s1ef_histogram_text: Vec<String> = s1ef_leaf_count_histogram
+        .iter()
+        .map(|(leaf_count, steps)| format!("{leaf_count}:{steps}"))
+        .collect();
+    emitter.emit(&format!(
+        "name=s1ef_summary steps={baseline_workload_actual_length} mismatches={s1ef_mismatches} steps_by_changed_leaf_count={}",
+        s1ef_histogram_text.join(",")
+    ));
     let beta1_basis = basis_of("beta1_h0", &memory_pool_snapshot(&devices), &current, true);
     emitter.emit(&format!(
         "name=beta1_h0_end txg={} accounting_slot={} referenced={}",
@@ -2869,6 +3155,13 @@
         };
         hr_current = published;
         hr_prefix_actual_length = step;
+        // Q7d-1（R2 ⑤）：H0 与 HR 的每一次 O 都要计进「这次发布自己的释放」的取样；HR 这里只量测，
+        // 不逐次核闭式（Q3r.2 只在 H0 一条历史上核，第八节 8.2）。
+        overwrite_release_samples.push(self_release_slots_of_this_publish(
+            &hr_allocator,
+            DeviceIdentity(0),
+            hr_current.root.checkpoint_txg,
+        ));
         record_legal_state(
             &mut emitter,
             &mut legal_states,
@@ -2968,6 +3261,11 @@
         };
         hr_current = published;
         hr_tail_actual_length = step;
+        overwrite_release_samples.push(self_release_slots_of_this_publish(
+            &hr_allocator,
+            DeviceIdentity(0),
+            hr_current.root.checkpoint_txg,
+        ));
         record_legal_state(
             &mut emitter,
             &mut legal_states,
@@ -3006,15 +3304,13 @@
         &mut emitter,
         &mut legal_states,
     );
-    // S1(e)：空发布 E 自己的释放槽数应为 4（E153 登记 M0.4）。
+    // S1(e)：空发布 E 自己的释放槽数不再是登记第一、二版钉的常数 4（分配记录树按位置寻址之后
+    // 不再是一个节点，R-5：这一次 = 8）；闭式核对已经在 `run_hk_family` 里对 `label == "HK"` 做过
+    // （`name=r5_empty_publish_closed_form`），这里只留一行观测方便直接搜。
     emitter.emit(&format!(
         "name=s1e_empty_publish_release slots={}",
         hk.empty_publish_release_slots
     ));
-    assert_eq!(
-        hk.empty_publish_release_slots, E156_EMPTY_PUBLISH_EXPECTED_RELEASED_SLOTS,
-        "S1(e)：空发布 E 自己的释放槽数与 E153 登记 M0.4 的 4 槽不等（停机）"
-    );
 
     let mut hk_forced_to_zero_legal_states: Vec<LegalState> = Vec::new();
     let hk_forced_to_zero = run_hk_family(
@@ -3097,15 +3393,14 @@
         first_red_legal_states.join(" | ")
     ));
 
-    // ===== Q7d-1：按发布种类分组的「这次发布自己的释放」（min/max/count）。H0/HR 的每一次 O 都在循环内被
-    // S1(e) 的 assert_eq! 钉死为 10（已即时核过，这里只汇总计数，不重新起分配器算一遍同一个数）；
-    // E 的唯一样本来自 HK（S1(e) 已单独报过）。 =====
-    let overwrite_count = legal_states
-        .iter()
-        .filter(|state| state.kind == "O")
-        .count();
+    // ===== Q7d-1（R2 ⑤，逐次现量）：按发布种类分组的「这次发布自己的释放」（min/max/count）。
+    // O 不再是常数：`overwrite_release_samples` 逐次收自 H0 与 HR（前缀 + 尾段）的每一次 O；
+    // E 的唯一样本来自 HK（`hk.empty_publish_release_slots`）。 =====
+    let (overwrite_release_minimum, overwrite_release_maximum) =
+        e156_minimum_and_maximum(&overwrite_release_samples);
     emitter.emit(&format!(
-        "name=q7d1_by_kind kind=O min=10 max=10 count={overwrite_count}"
+        "name=q7d1_by_kind kind=O min={overwrite_release_minimum} max={overwrite_release_maximum} count={}",
+        overwrite_release_samples.len()
     ));
     emitter.emit(&format!(
         "name=q7d1_by_kind kind=E min={0} max={0} count=1",
@@ -3243,7 +3538,10 @@
     // ===== 岔路 3（第二段，缩小范围：S = 8、ρ = 1 一个几何取样点，见文件顶注释）=====
     run_hf_single_cell(&parameters, &mut emitter);
     run_small_pool_cell(&mut emitter, "hx", None);
-    run_small_pool_cell(&mut emitter, "hy", Some(3));
+    // R-7：HY 的覆盖写次数不再写死 3——3 次不满足 e ≥ max(8, f) 就减少直到满足（第十二节记录用了几次）。
+    let hy_cap = e156_find_hy_cap_satisfying_open_segment_condition(&small_pool_parameters());
+    emitter.emit(&format!("name=r7_hy_cap chosen_cap={hy_cap}"));
+    run_small_pool_cell(&mut emitter, "hy", Some(hy_cap));
 
     // ===== 岔路 1（第三段 a 的一部分，S 这一维：S = 8（mkfs 默认）与 S = 4（下界，方向相反，
     // 「五、5.6」第六类「至少一个方向相反的取样点」），ρ = 1、回收时点固定「实」、洞位置固定「后」，
@@ -3425,11 +3723,16 @@
     use super::{
         accounting_row_slots, allocated_minus_deferred_matches_referenced,
         allocated_minus_deferred_mismatches_referenced, corrupt_device_zero_accounting,
-        memory_pool_snapshot, parameters, referenced_slots, self_release_slots_of_this_publish,
-        FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
+        e156_allocation_record_tree_new_node_count, e156_closed_form_expected,
+        e156_existing_leaf_positions, e156_touched_leaf_positions,
+        memory_pool_snapshot, parameters, referenced_slots, run_s1c_rollback_isolation_scenario,
+        self_release_slots_of_this_publish, Emitter, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
+    };
+    use std::collections::BTreeSet;
+    use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
+    use singlefs_core::allocator::{
+        AllocationRecord, DeviceFreeMap, Placement, PoolAllocator, ReclaimedReuse,
     };
-    use singlefs_core::address::DeviceIdentity;
-    use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator, ReclaimedReuse};
     use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
     use singlefs_core::make_filesystem::{
         make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
@@ -3498,26 +3801,30 @@
         (allocator, first, devices)
     }
 
+    /// K1-1（E156 第 3 次重跑登记「七」7.2）：分配记录树按位置寻址之后第一个事务的记账第 1 项是 17
+    /// （五个树节点占五条记录，不再是登记第一、二版的单节点 13），第 5 项仍是 1。
     #[test]
     fn accounting_after_first_transaction_matches_the_registered_anchor() {
         let (allocator, first, _devices) = first_transaction_state();
         assert_eq!(first.root.checkpoint_txg.0, 3);
         let (allocated, _free, deferred) = accounting_row_slots(&allocator, DeviceIdentity(0));
-        assert_eq!(allocated, 13, "K1 第 1 项");
-        assert_eq!(deferred, 1, "K1 第 5 项");
+        assert_eq!(allocated, 17, "K1-1 第 1 项");
+        assert_eq!(deferred, 1, "K1-1 第 5 项");
     }
 
+    /// K1-3：β0 的最新根走读引用 = 16（实例表 2 槽 + 第一个事务九个角色共 14 槽，五个分配记录树节点
+    /// 记在这 14 槽里），不再是登记第一、二版的 12。
     #[test]
     fn referenced_slots_counts_the_carried_instance_table_before_its_first_rewrite() {
         let (allocator, first, _devices) = first_transaction_state();
         let referenced = referenced_slots(&first);
         assert_eq!(
-            referenced, 12,
-            "实例表 2 槽（未进 units，靠兜底加回）+ 第一个事务 8 个角色共 10 槽"
+            referenced, 16,
+            "实例表 2 槽（未进 units，靠兜底加回）+ 第一个事务九个角色共 14 槽（K1-3）"
         );
         assert!(
             allocated_minus_deferred_matches_referenced(&allocator, DeviceIdentity(0), referenced),
-            "第一个事务之后 G27 应当成立：13 − 1 == 12"
+            "第一个事务之后 G27 应当成立：17 − 1 == 16（K1-3）"
         );
     }
 
@@ -3565,8 +3872,8 @@
             accounting_row_slots(&allocator, DeviceIdentity(0));
         assert_eq!(
             (memory_allocated, memory_deferred),
-            (13, 1),
-            "U8：同一镜像上内存分配器的行没变（只改了镜像字节）"
+            (17, 1),
+            "U8：同一镜像上内存分配器的行没变（只改了镜像字节，K1-1：17/1）"
         );
     }
 
@@ -3698,4 +4005,163 @@
     // 不暴露推空循环中途的钩子，而空发布本身会重写树表单元（连带新分配 + 释放旧树表槽，登记 D16（发布语义）
     // 已定项 1「非空」段落逐字），使「推空前后记账第 1 项该差多少」不是一条简单算式，贸然钉一个数风险比价值大；
     // 写进交回报告的岔路表，留给下一段。
+
+    /// PC-闭式自测第一、二、四组（E156 第 3 次重跑登记「七」7.2 R-3c）：只在合成的叶位置集合上单测
+    /// 闭式函数本身，不依赖真实分配器。M27（闭式漏掉根）、M28（闭式每个改动的叶位置只算一块盘）应当
+    /// 分别让第一组从 5 变成 4 与 3。
+    #[test]
+    fn pc_closed_form_matches_the_registered_anchor_r3c() {
+        let one_leaf: BTreeSet<u64> = [61].into_iter().collect();
+        let two_leaves: BTreeSet<u64> = [61, 62].into_iter().collect();
+        let leaves_in_two_internal: BTreeSet<u64> = [61, 170].into_iter().collect();
+        assert_eq!(
+            e156_allocation_record_tree_new_node_count(&one_leaf, 2),
+            5,
+            "R-3c：只在第 61 片叶（两盘）"
+        );
+        assert_eq!(
+            e156_allocation_record_tree_new_node_count(&two_leaves, 2),
+            7,
+            "R-3c：第 61、62 片叶"
+        );
+        assert_eq!(
+            e156_allocation_record_tree_new_node_count(&leaves_in_two_internal, 2),
+            9,
+            "R-3c：第 61 与第 170 片叶（第 170 片在第二个层级 1 节点里）"
+        );
+        assert_eq!(
+            e156_allocation_record_tree_new_node_count(&one_leaf, 1),
+            3,
+            "R-3c：一盘、只在第 61 片叶"
+        );
+        assert!(
+            170 * 812 >= 812 * 169,
+            "R-3c：170 号叶应落在第二个层级 1 节点里"
+        );
+    }
+
+    /// PC-闭式第三组（R-3c，M29 的取样点）：合成一份记录——一条在第 61 片叶已释放、一条在第 170 片叶
+    /// 仍分配，两条 `generation` 都是这次的 txg。`e156_touched_leaf_positions` 应当把两条都算进
+    /// 「这次改动」，给出 9；M29（闭式的「改动的记录」只取这次分配的、不取这次释放的）应当让这一格
+    /// 只剩第 170 片叶一条，变成 5。
+    #[test]
+    fn pc_closed_form_third_group_counts_released_and_allocated_records() {
+        let devices = vec![DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES)];
+        let txg = CheckpointTxg(9);
+        let released_slot = 50245u64;
+        let allocated_slot = 170u64 * 812;
+        let records = vec![
+            AllocationRecord {
+                device: DeviceIdentity(0),
+                slot: SlotNumber(released_slot),
+                span_slots: 1,
+                generation: txg,
+                is_released: true,
+            },
+            AllocationRecord {
+                device: DeviceIdentity(0),
+                slot: SlotNumber(allocated_slot),
+                span_slots: 1,
+                generation: txg,
+                is_released: false,
+            },
+        ];
+        let allocator = PoolAllocator::rebuild_from_records(devices, records);
+        let touched = e156_touched_leaf_positions(&allocator, DeviceIdentity(0), txg);
+        assert_eq!(
+            touched, BTreeSet::from([61, 170]),
+            "PC-闭式第三组：应当同时看到释放在第 61 片、分配在第 170 片两条"
+        );
+        assert_eq!(
+            e156_allocation_record_tree_new_node_count(&touched, 2),
+            9,
+            "R-3c：第 61 与第 170 片叶应给出 9"
+        );
+    }
+
+    /// U10（第九节）：β0 之后连续覆盖写，逐次核对 R-3 的闭式；次序不钉死，只保证跑够多步、两个方向
+    /// （落在 1 片叶 / 落在 ≥ 2 片叶）都至少出现一次（第八节 8.2）。前 4 次另外钉 R-4 的绝对值
+    /// （released_d0=14、record_delta=24）。U12（Q7d-1，R2 ⑤）：这段历史上逐次现量的最小值 14、
+    /// 最大值 ≥ 16——M30（Q7d-1 覆盖写那一组退回字面 10）应当让最小值变成 10。
+    #[test]
+    fn overwrite_steps_match_the_closed_form_and_cross_a_second_leaf() {
+        let (mut allocator, first, mut devices) = first_transaction_state();
+        let parameters = parameters();
+        let instance = InstanceGeneration(1);
+        let mut current = first;
+        let mut release_samples: Vec<u64> = Vec::new();
+        let mut saw_single_leaf_step = false;
+        let mut saw_multi_leaf_step = false;
+        for step in 1..=12u64 {
+            let existing_leaves = e156_existing_leaf_positions(&allocator, DeviceIdentity(0));
+            let records_before = allocator.records().len();
+            current = {
+                let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
+                singlefs_core::transaction::publish_overwrite(
+                    &mut pool,
+                    &mut allocator,
+                    &current,
+                    FirstFile {
+                        content: &super::overwrite_content(step),
+                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 + step,
+                    },
+                    instance,
+                )
+                .expect("U10 覆盖写")
+            };
+            let released = self_release_slots_of_this_publish(
+                &allocator,
+                DeviceIdentity(0),
+                current.root.checkpoint_txg,
+            );
+            let touched = e156_touched_leaf_positions(
+                &allocator,
+                DeviceIdentity(0),
+                current.root.checkpoint_txg,
+            );
+            let (expected_released, expected_delta) =
+                e156_closed_form_expected(&existing_leaves, &touched, true);
+            let record_count = allocator.records().len();
+            let delta =
+                u64::try_from(record_count - records_before).expect("这一步的记录增量落在 u64 内");
+            assert_eq!(
+                released, expected_released,
+                "U10：第 {step} 次覆盖写释放槽数应等于闭式"
+            );
+            assert_eq!(
+                delta, expected_delta,
+                "U10：第 {step} 次覆盖写记录增量应等于闭式"
+            );
+            if step <= 4 {
+                assert_eq!(released, 14, "R-4：第 {step} 次覆盖写 D0 释放槽数应为 14");
+                assert_eq!(delta, 24, "R-4：第 {step} 次覆盖写记录增量应为 24");
+            }
+            release_samples.push(released);
+            if touched.len() <= 1 {
+                saw_single_leaf_step = true;
+            } else {
+                saw_multi_leaf_step = true;
+            }
+        }
+        assert!(saw_single_leaf_step, "U10：应当至少有一步只落在 1 片叶");
+        assert!(
+            saw_multi_leaf_step,
+            "U10：应当至少有一步跨到 ≥ 2 片叶（第八节 8.2）"
+        );
+        let (minimum, maximum) = super::e156_minimum_and_maximum(&release_samples);
+        assert_eq!(minimum, 14, "U12：Q7d-1 覆盖写那一组的最小值应为 14");
+        assert!(
+            maximum >= 16,
+            "U12：Q7d-1 覆盖写那一组的最大值应 ≥ 16（跨叶时更贵）"
+        );
+    }
+
+    /// U11（第九节，R-6）：重放 S1(c) 场景（`run_s1c_rollback_isolation_scenario` 内部的
+    /// `assert_eq!` 已经改成 54），只是从单测里再触发一次，M32（S1(c) 的隔离槽数退回 34）应当让这条
+    /// 测试红。
+    #[test]
+    fn rollback_isolation_scenario_matches_the_new_layout() {
+        let mut emitter = Emitter { emitted: 0 };
+        run_s1c_rollback_isolation_scenario(&mut emitter, &parameters());
+    }
 }
diff -ruN -x target /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-harness/src/bin/first_transaction_on_device.rs tree/crates/singlefs-harness/src/bin/first_transaction_on_device.rs
--- /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-harness/src/bin/first_transaction_on_device.rs	2026-09-24 17:28:54.669972875 +0000
+++ tree/crates/singlefs-harness/src/bin/first_transaction_on_device.rs	2026-09-24 23:14:43.842797206 +0000
@@ -16,7 +16,8 @@
 //! `raise-rollback-floor`：`second-instance` 那条路走完，同一次挂载里再覆盖写一次（发布 D，第 4 新的非空有效根落到 A 上），接着把 F 抬到上限
 //! （里程碑「第二个事务」增补 2 收口表第 58 行：抬 F 那一串发布的账在二进制这一侧与设备一层判相等）；发布 D 与发布 C 同样打一行、一行
 //! `name=publish_writes`、一段窗口行，抬 F 一行、那一串空发布各一行 `name=publish_writes`、这一段一行窗口行；失败时照发布失败那样打失败账。
-//! 冷重开读回的是第四版（最后一次抬 F 的空发布的根）。前五个模式打的行一行不变。
+//! 冷重开读回的是第四版（最后一次抬 F 的空发布的根）；`name=recover_cold` 之后多打一行 `name=recover_cold_rollback_floor`：
+//! 冷重开择到的根与它从盘上读回的 F（`rollback_floor_on_disk=`，代码轮第二轮判决 Z10）。前五个模式打的行一行不变。
 //! 每次发布（两次暖机、第一个事务、`second-transaction` 模式下的发布 B）各打一行 `name=publish_writes`：写入口按结构种类记的写调用数与写字节
 //! （里程碑「第二个事务」增补 1 第 1 件）；每段窗口（两次暖机合一段、第一个事务、发布 B）再打一行 `name=publish_writes_against_device`，
 //! 按种类的合计与设备一层数的（`FaultInjectingDevice`，两盘相加）逐项比，对不上 `matches=false`、退出码 1。
@@ -35,7 +36,9 @@
 };
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
 use singlefs_core::mount::{mount_writable, MountError, Mounted, PublishSequenceFailed};
-use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
+use singlefs_core::recovery::{
+    choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome,
+};
 use singlefs_core::transaction::{PoolVersion, TransactionOutput};
 use singlefs_core::write_accounting::{
     WriteCallsAndBytes, WritesByStructureKind, WrittenStructureKind,
@@ -759,6 +762,7 @@
                 | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
                 | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
                 | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
+                | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
                 | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
                 | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
                 | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. } => {
@@ -1055,6 +1059,24 @@
     raise_the_rollback_floor_and_describe(parameters, fourth_version_run, stream, geometry, clock)
 }
 
+/// `raise-rollback-floor` 模式冷重开之后打的那一行：所选根从盘上读回的回退下界 F（代码轮第二轮判决 Z10，
+/// `research/prompts/m2-final-code-r2-main-verification.md` 第四节第 5 条）。所选根照恢复择根的同一套规则现读
+/// （`choose_system_configuration` + `choose_root`：先跳过被回退见证抛弃的根，再按 (txg, 实例) 择新），F 取那条根记录自己的字段——
+/// 不取 `name=raise_rollback_floor` 那一行里内存那一版的 `requested_floor`。择不出系统配置或根时两段都写 `none`。
+fn rollback_floor_read_back_from_the_chosen_root_line(reader: &dyn PoolReader) -> String {
+    let chosen_root = choose_system_configuration(reader)
+        .ok()
+        .and_then(|system_configuration| choose_root(reader, &system_configuration));
+    match chosen_root {
+        Some(root) => format!(
+            "name=recover_cold_rollback_floor chosen_root={}:{} rollback_floor_on_disk={}",
+            root.instance.0, root.checkpoint_txg.0, root.rollback_floor.0
+        ),
+        None => "name=recover_cold_rollback_floor chosen_root=none rollback_floor_on_disk=none"
+            .to_string(),
+    }
+}
+
 /// `raise-rollback-floor`：发布 D 之后、同一次挂载里把 F 抬到上限（`raise_the_rollback_floor_to_its_ceiling`，宿主检查重跑同一份）。
 /// 这一段窗口从抬 F 之前那一刻算起、到它返回为止，按种类的合计是那一串空发布之和，与设备一层逐项比（增补 2 收口表第 58 行
 /// 「二进制那一侧判相等」）。抬 F 只有测试入口（`raise_rollback_floor` 的文档注释），这一档就是那个入口在真设备上的一次。
@@ -1123,6 +1145,7 @@
                 | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
                 | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
                 | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
+                | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
                 | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
                 | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
                 | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. } => {
@@ -1521,6 +1544,16 @@
         })
         .collect();
     let report = recover(&reopened, JournalPolicy::Consult);
+    // 抬 F 模式多打一行：所选根从盘上读回的 F（代码轮第二轮判决 Z10：`name=raise_rollback_floor` 那一行的 `requested_floor=`
+    // 是内存里那一版的 F，冷重开择根并不看它，55 号要独立读回盘上的 F 才判得出「F 抬到 3」）。别的模式一行不多。
+    let rollback_floor_read_back_line = match mode.publishes_after_the_first_transaction() {
+        PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstanceThenFourthVersionThenRaiseOfTheRollbackFloor => {
+            Some(rollback_floor_read_back_from_the_chosen_root_line(&reopened))
+        }
+        PublishesAfterTheFirstTransaction::Nothing
+        | PublishesAfterTheFirstTransaction::SecondVersion
+        | PublishesAfterTheFirstTransaction::SecondVersionThenSecondInstance => None,
+    };
     let (outcome, root, content_matches) = match &report.outcome {
         RecoveryOutcome::FileRead { root, content } => (
             "file_read",
@@ -1547,6 +1580,9 @@
         "name=recover_cold outcome={outcome} root={root} content_matches={content_matches} valid_records={} above_water={} applied={} verification_passed={} mapping_fallbacks={}",
         report.journal.valid_records, report.journal.above_water, report.journal.prefix_applied, report.journal.verification_passed, report.mapping_fallbacks
     ));
+    if let Some(line) = &rollback_floor_read_back_line {
+        emitter.emit(line);
+    }
     // 分段挂钟：每段一行，最后一行是包含自检（各段之和不超过整条路）。
     for line in &segment_timing_lines {
         emitter.emit(line);
@@ -2600,6 +2636,78 @@
         }
     }
 
+    /// 代码轮第二轮判决 Z10（`research/prompts/m2-final-code-r2-main-verification.md` 第四节第 5 条）：`raise-rollback-floor` 那条路在宿主上走到
+    /// 抬 F 之后冷重开，多打的那一行 `name=recover_cold_rollback_floor` 报的是冷重开择到的根 (2, 11) 与它从盘上读回的 F = 3，
+    /// 不是 `name=raise_rollback_floor` 里内存那一版的 `requested_floor`。同一段历史抬 F 之前的盘面上（发布 C 之后、择到 (2, 8)）读回的是 0：
+    /// 这一行跟着盘上的根走，不是写死的数。
+    #[test]
+    fn raise_rollback_floor_mode_prints_the_floor_read_back_from_the_chosen_root_after_the_cold_reopen(
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
+        let before_the_raise: Vec<(DeviceIdentity, SparseBlockDevice)> = switched
+            .devices
+            .iter()
+            .map(|(identity, device)| {
+                let mut copy =
+                    SparseBlockDevice::new(SPARSE_DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
+                copy.image = device.inner().inner().image.clone();
+                (*identity, copy)
+            })
+            .collect();
+        let line_before_the_raise =
+            super::rollback_floor_read_back_from_the_chosen_root_line(&before_the_raise);
+        assert_eq!(
+            field(&line_before_the_raise, "chosen_root"),
+            Some("2:8"),
+            "发布 C 之后择到 (2, 8)：{line_before_the_raise}"
+        );
+        assert_eq!(
+            field(&line_before_the_raise, "rollback_floor_on_disk"),
+            Some("0"),
+            "抬 F 之前盘上的 F 是 0：{line_before_the_raise}"
+        );
+        let raised = match publish_the_fourth_version_and_raise_the_rollback_floor(
+            &parameters,
+            switched,
+            &stream,
+            &geometry,
+            &mut clock,
+        ) {
+            Ok(raised) => raised,
+            Err(failed) => panic!("发布 D、抬 F：{failed:?}"),
+        };
+        let cold: Vec<(DeviceIdentity, SparseBlockDevice)> = raised
+            .devices
+            .into_iter()
+            .map(|(identity, device)| (identity, device.into_inner().into_inner_and_operations().0))
+            .collect();
+        let line = super::rollback_floor_read_back_from_the_chosen_root_line(&cold);
+        assert_eq!(
+            field(&line, "name"),
+            Some("recover_cold_rollback_floor"),
+            "{line}"
+        );
+        assert_eq!(
+            field(&line, "chosen_root"),
+            Some("2:11"),
+            "冷重开择到最后一次抬 F 的空发布的根：{line}"
+        );
+        assert_eq!(
+            field(&line, "rollback_floor_on_disk"),
+            Some("3"),
+            "所选根从盘上读回的 F 是抬到的上限 3：{line}"
+        );
+    }
+
     /// 增补 2 收口表第 58 行，发布 D 那一段（`raise-rollback-floor`）：发布 C 之后装上注入，发布 D 整池第 5 次写报错。
     /// 挂载与发布 C 那几行照打在前面，失败账 4 次写与设备一层（重开之后那个注入计划数的）逐项相等；发布 D 没做成就不抬 F。
     #[test]
diff -ruN -x target /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-harness/src/fault_injection.rs tree/crates/singlefs-harness/src/fault_injection.rs
--- /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-harness/src/fault_injection.rs	2026-09-24 13:10:11.115713920 +0000
+++ tree/crates/singlefs-harness/src/fault_injection.rs	2026-09-24 23:49:34.040476097 +0000
@@ -1782,6 +1782,7 @@
     let measurement_execution = HistoryExecution {
         per_step_checker: PerStepChecker::Skipped,
         device_width: execution.device_width,
+        space_admission: execution.space_admission,
     };
     let measurement_plan = SharedFaultPlan::unarmed(geometry);
     let measurement_stream = SharedStream::new();
diff -ruN -x target /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-harness/src/history.rs tree/crates/singlefs-harness/src/history.rs
--- /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-harness/src/history.rs	2026-09-24 20:56:53.151716323 +0000
+++ tree/crates/singlefs-harness/src/history.rs	2026-09-24 23:55:14.731111232 +0000
@@ -25,6 +25,7 @@
 };
 use singlefs_checker::walk::{allocation_record_count_under_root, check_pool_image};
 use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::allocator::{AllocationRecord, PlacementRefusal, PoolAllocator};
 use singlefs_core::block_device::{BlockDeviceError, PhysicalBlockSizeInBytes};
 use singlefs_core::journal::back_chain_of;
@@ -32,7 +33,8 @@
     allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
 };
 use singlefs_core::mount::{
-    mount_rollback, mount_writable, raise_rollback_floor, MountError, RollbackTarget, ShadowLedger,
+    mount_rollback_with_space_admission, mount_writable_with_space_admission, raise_rollback_floor,
+    MountError, RollbackTarget, ShadowLedger,
 };
 use singlefs_core::recovery::{
     allocation_records_under_root, choose_root, choose_system_configuration,
@@ -721,6 +723,8 @@
 /// 一段历史跑到哪了：两块盘（与它们的宽度）、这个进程的会话、挂载的次数、理想模型、录制流（判「拒绝之前写没写盘」）。
 struct HistoryPool {
     device_width: HistoryDeviceWidth,
+    /// 判不判空间准入（只供测试的开关，`singlefs_core::admission::SpaceAdmission`）：起点那条会话装在分配器上，挂载照它传。
+    space_admission: SpaceAdmission,
     devices: Vec<(DeviceIdentity, HistoryDevice)>,
     session: Option<WritableSession>,
     successful_mounts: usize,
@@ -845,6 +849,7 @@
     fn start(
         starting_point: HistoryStartingPoint,
         device_width: HistoryDeviceWidth,
+        space_admission: SpaceAdmission,
         stream: &SharedStream,
         fault_plan: &SharedFaultPlan,
     ) -> Result<(Self, Option<ModelVerdict>), StartingPointFailure> {
@@ -894,6 +899,7 @@
                 // 与做过挂载的会话同一条路（增补 2 收口表第 ② 行）。
                 let mut allocator =
                     allocator_after_make_filesystem(&parameters, &devices, &genesis);
+                allocator.set_space_admission(space_admission);
                 let content = first_file_content();
                 let started = acquire_warm_up_and_publish_the_first_file(
                     &parameters,
@@ -933,6 +939,7 @@
         Ok((
             Self {
                 device_width,
+                space_admission,
                 devices,
                 session,
                 successful_mounts: 0,
@@ -1913,6 +1920,7 @@
                 placement_refusal_member(refusal)
             )
         }
+        PublishError::SpaceAdmissionRefused(_) => "SpaceAdmissionRefused",
         PublishError::AllocationRecordTreeRewriteSetDidNotSettle { .. } => {
             "AllocationRecordTreeRewriteSetDidNotSettle"
         }
@@ -2063,6 +2071,9 @@
         MountError::InstanceGenerationChangedBeforeAcquisition { .. } => {
             "InstanceGenerationChangedBeforeAcquisition"
         }
+        MountError::SpaceAdmissionRefusedBeforeAcquisition { .. } => {
+            "SpaceAdmissionRefusedBeforeAcquisition"
+        }
         MountError::RowPublishAdmissionRefusedBeforeAcquisition { cause, .. } => {
             return format!(
                 "MountError::RowPublishAdmissionRefusedBeforeAcquisition({})",
@@ -2529,7 +2540,8 @@
     let image_before_mount = pool.image();
     let answer = pool.model.answer_mount_writable();
     let stream_length_before = pool.stream.operation_count();
-    let mounted = mount_writable(parameters, &mut pool.devices);
+    let mounted =
+        mount_writable_with_space_admission(parameters, &mut pool.devices, pool.space_admission);
     settle_mount(
         pool,
         mounted,
@@ -2584,7 +2596,13 @@
         .model
         .answer_mount_rollback(model_root_key(target.instance, target.checkpoint_txg));
     let stream_length_before = pool.stream.operation_count();
-    let mounted = mount_rollback(parameters, &mut pool.devices, target, ShadowLedger::On);
+    let mounted = mount_rollback_with_space_admission(
+        parameters,
+        &mut pool.devices,
+        target,
+        ShadowLedger::On,
+        pool.space_admission,
+    );
     settle_mount(
         pool,
         mounted,
@@ -2827,6 +2845,9 @@
 pub struct HistoryExecution {
     pub per_step_checker: PerStepChecker,
     pub device_width: HistoryDeviceWidth,
+    /// 判不判空间准入（只供测试的开关，`singlefs_core::admission::SpaceAdmission`）。取样点里要测「准入放行而落点取不到」那条兜底拒绝的
+    /// （小盘上式子先拒，走不到落点那一道）装 `SkippedByTheTestOnlySwitch`；别的一律判。
+    pub space_admission: SpaceAdmission,
 }
 
 impl HistoryExecution {
@@ -2834,16 +2855,25 @@
     pub const CHECKED_ON_FOUR_GIBIBYTE_DEVICES: HistoryExecution = HistoryExecution {
         per_step_checker: PerStepChecker::Run,
         device_width: HistoryDeviceWidth::FourGibibytes,
+        space_admission: SpaceAdmission::JudgedByTheFormula,
     };
 
     /// 报告里的名字。
     #[must_use]
     pub fn name(self) -> String {
-        format!(
-            "{}；{}",
-            self.per_step_checker.name(),
-            self.device_width.name()
-        )
+        match self.space_admission {
+            SpaceAdmission::JudgedByTheFormula => format!(
+                "{}；{}",
+                self.per_step_checker.name(),
+                self.device_width.name()
+            ),
+            SpaceAdmission::SkippedByTheTestOnlySwitch => format!(
+                "{}；{}；空间准入关掉（只供测试的开关，{}）",
+                self.per_step_checker.name(),
+                self.device_width.name(),
+                self.space_admission.branch_name()
+            ),
+        }
     }
 }
 
@@ -2909,6 +2939,7 @@
         let started = HistoryPool::start(
             history.starting_point,
             execution.device_width,
+            execution.space_admission,
             stream,
             fault_plan,
         );
diff -ruN -x target /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-harness/src/model_comparison.rs tree/crates/singlefs-harness/src/model_comparison.rs
--- /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-harness/src/model_comparison.rs	2026-09-24 20:07:28.373234637 +0000
+++ tree/crates/singlefs-harness/src/model_comparison.rs	2026-09-24 23:12:53.152121378 +0000
@@ -191,6 +191,8 @@
         PublishError::PlacementRefused { refusal, .. } => {
             refusal_reason_of_placement_refusal(refusal)
         }
+        // 空间准入（D28（挂载期承诺量） 已定项 1 的式子）判这次的普通分配不够：模型的「单元区装不下」就是这一条准入（模型答允许拒绝的区间）。
+        PublishError::SpaceAdmissionRefused(_) => explained(ModelRefusalReason::UnitAreaWall),
         PublishError::ContentExceedsDataUnit { .. } => {
             explained(ModelRefusalReason::ContentExceedsDataUnitPayload)
         }
@@ -287,6 +289,10 @@
         MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
             refusal, ..
         } => refusal_reason_of_placement_refusal(refusal),
+        // 取号之前空间准入不够（实例切换的预留拿不到）：同发布那一条，是模型的单元区墙。
+        MountError::SpaceAdmissionRefusedBeforeAcquisition { .. } => {
+            explained(ModelRefusalReason::UnitAreaWall)
+        }
         // 树表 0 条、而实例表已经不是 mkfs 那一片：零故障走得到（写过行的那一版上再挂载一次），模型照代码今天的读法划进必须拒。
         MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. } => {
             explained(ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem)
@@ -320,6 +326,7 @@
         | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
         | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
         | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
+        | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
         | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
         | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
         | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. } => None,
```

## 二、`tests/` 下的改动（13 个文件，只列文件名与增删行数；增删数由材料员对 `r3-to-r4.diff` 里对应段落数 `+`/`-` 开头且非 `+++`/`---` 的行得出，不抄全文）

| 文件（`crates/singlefs-harness/` 下的相对路径） | + | − |
|---|---|---|
| tests/checker_known_bad_images.rs | +275 | -1 |
| tests/common/mod.rs | +29 | -0 |
| tests/second_transaction_step_four_rollback.rs | +64 | -29 |
| tests/second_transaction_step_three_formatted_pool.rs | +13 | -2 |
| tests/second_transaction_supplement_three_bad_disk_input.rs | +2 | -0 |
| tests/second_transaction_supplement_three_crash_injection.rs | +3 | -0 |
| tests/second_transaction_supplement_three_fault_injection.rs | +8 | -0 |
| tests/second_transaction_supplement_three_random_history.rs | +85 | -2 |
| tests/second_transaction_supplement_two_admission_formula.rs | +246 | -3 |
| tests/second_transaction_supplement_two_commit_generated_fallback.rs | +37 | -11 |
| tests/second_transaction_supplement_two_release_checksum_quarantine.rs | +126 | -5 |
| tests/second_transaction_supplement_two_rollback_witness.rs | +401 | -38 |
| tests/second_transaction_supplement_two_root_ring_turn_in_one_mount.rs | +5 | -1 |

## 三、新文件全文

`r3-to-r4.diff` 用 `diff -ruN` 生成：23 个文件的两侧路径下都存在同名文件（没有一段显示 `/dev/null` 或「No such file or directory」），这一轮没有新建文件，本节空。

## 四、定义与检测器相对第三轮冻结的 diff（`defs-r3-to-r4.diff`，3 个文件，整段进）

```diff
--- /tmp/claude-1000/m2-final-code-r3/defs/.claude/agent-common.md	2026-09-24 22:47:56.775221502 +0000
+++ defs/.claude/agent-common.md	2026-09-25 00:58:10.306815767 +0000
@@ -54,7 +54,7 @@
 - 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
 - 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
   等长活时不起缓存计时器，结束本轮直接等完成通知。
-  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）对四种写法在执行前拒绝：前台没有超时的等待循环、把活放出追踪（`disown`、`nohup … &` 这类）、run_in_background 里以单独的 `&` 收尾而之后同一条命令里没有 `wait` 的作业、起看门狗的错误写法（只有主 agent 起看门狗）；run_in_background 里的等待循环与按模式找进程这类命令只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
+  等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；项目 settings 里的 Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）对五种写法在执行前拒绝：前台没有超时的等待循环、把活放出追踪（`disown`、`nohup … &` 这类）、run_in_background 里以单独的 `&` 收尾而之后同一条命令里没有 `wait` 的作业、用 `>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 整份覆盖 `research/results/` 下已存在又没进 git 的产物（要换就按日期另存新文件名，旧的留着）、起看门狗的错误写法（只有主 agent 起看门狗）；run_in_background 里的等待循环与按模式找进程这类命令只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
 
 ## 门禁
 
--- /tmp/claude-1000/m2-final-code-r3/defs/.claude/main-agent.md	2026-09-24 22:47:56.776427505 +0000
+++ defs/.claude/main-agent.md	2026-09-25 00:58:10.307819614 +0000
@@ -26,7 +26,7 @@
 
 ## 派出去之后
 
-盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的（前台无超时等待循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&`、子 agent 跑重型测试、写撇号类角标、覆盖未跟踪文件）在执行前拒绝，不碰已经在跑的东西。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨、本实例底下有没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程（确认接着盯写 `--ack 进程:<pid>`）、子 agent 从派发起的运行时间跨过一个整小时（每个整点报一次，那个整点之后主 agent 给它发过消息就不报；一格多长在 `watch.conf` 的 `ask-every-minutes`），或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
+盯住，不能一直干等，也不强制结束：子 agent 与脚本可以跑很久，结不结束、怎么处置由主 agent 看了定；hook 不停任务和脚本：检出类的只记录；拒绝一种危险写法的（前台无超时等待循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&`、用 shell 的 `>`、`tee`、`cp`、`mv` 整份覆盖 `research/results/` 下未跟踪的产物、子 agent 跑重型测试、写撇号类角标、覆盖未跟踪文件）在执行前拒绝，不碰已经在跑的东西。每次派发（包括续做）之后，立刻用 Bash 的 `run_in_background` 起看门狗 `bash research/scripts/watch.sh <这次派出的 agent id，逗号分隔>`（只填 agent 号：会话目录它自己找，阈值在 `research/scripts/watch.conf` 里配，不在命令行上手敲；调用里不再加 `&`、`nohup`、`disown`）：它每 4 分钟复检一次子 agent 的会话记录与本实例底下的长进程、每 15 秒读一次 hook 的检出记录，发现没超时的等待循环、一次工具调用超过 8 分钟、同一命令反复且输出不变、按模式找进程、10 分钟无动静、结束本轮而手里没有在跑的后台任务、进程写的文件 20 分钟不涨、本实例底下有没带 `SINGLEFS_HEAVY_TESTS` 前缀的重型测试进程（确认接着盯写 `--ack 进程:<pid>`）、子 agent 从派发起的运行时间跨过一个整小时（每个整点报一次，那个整点之后主 agent 给它发过消息就不报；一格多长在 `watch.conf` 的 `ask-every-minutes`），或 hook 检出本会话的问题命令，就退出并叫醒主 agent；子 agent 全部交回或被停也退出，没有告警时每 60 分钟定时回报一次。
 
 改了定义、共用约束或规则之后，当场给每个在跑的子 agent 发消息：写明改了哪一条、它手上哪一步要停掉或改做。
 
--- /tmp/claude-1000/m2-final-code-r3/defs/.claude/hooks/bash-command-detector.sh	2026-09-24 22:47:56.779714661 +0000
+++ defs/.claude/hooks/bash-command-detector.sh	2026-09-25 00:58:10.308837971 +0000
@@ -1,5 +1,5 @@
 #!/usr/bin/env bash
-# PreToolUse hook（Bash）：检出可能出问题的命令，记下来交给主 agent 判断；起看门狗的错误写法、前台没超时的等待循环、把活放出追踪的写法与 run_in_background 里后面没有 wait 的单独 `&` 在执行前拒绝，其余只记不拦，不停任何在跑的命令与脚本。
+# PreToolUse hook（Bash）：检出可能出问题的命令，记下来交给主 agent 判断；起看门狗的错误写法、前台没超时的等待循环、把活放出追踪的写法、run_in_background 里后面没有 wait 的单独 `&` 与整份覆盖 `research/results/` 下未跟踪产物的写法在执行前拒绝，其余只记不拦，不停任何在跑的命令与脚本。
 #
 # 按模式找进程（`pgrep -f`、`pkill -f`、`killall`）不在这里判：上游 SOP 的 `.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh`
 # 在执行前拒绝（`.claude/settings.json` 里与本 hook 注册在同一条 Bash matcher 下），判据与 shell-lint 的 S2、S3 同一份、只认命令位置。
@@ -10,9 +10,9 @@
 # 只负责检出问题，交给主 agent 去判断；子 agent 与脚本结不结束由主 agent 定。
 #
 # 检出两种，写进检出记录（默认 /tmp/claude-1000/agent-hook-detections.jsonl，环境变量 AGENT_HOOK_DETECTIONS 可改），每条一行 JSON：
-#   ① 没有 `timeout` 的 `until` / `while` 等待循环（里面有 `sleep`）：等的条件不成立时会一直等（前台的在执行前拒绝，见「拒绝四种」②，拒了不再记）；
+#   ① 没有 `timeout` 的 `until` / `while` 等待循环（里面有 `sleep`）：等的条件不成立时会一直等（前台的在执行前拒绝，见「拒绝五种」②，拒了不再记）；
 #   ② `run_in_background` 起的命令里又自己放后台：外层 shell 当场退出（下面列的形态里，命令位置上的 disown、coproc、setsid -f、nohup … &、
-#      tmux / screen 的分离模式、systemd-run 在执行前拒绝，见「拒绝四种」③；单独的 `&` 而之后同一层没有 `wait` 的在执行前拒绝，见「拒绝四种」④；
+#      tmux / screen 的分离模式、systemd-run 在执行前拒绝，见「拒绝五种」③；单独的 `&` 而之后同一层没有 `wait` 的在执行前拒绝，见「拒绝五种」④；
 #      拒了不再记。这里剩下的只记：起了几个 `&`、之后的 `wait "$pid"` 只等了其中一个，与 `start-stop-daemon -b`），
 #      harness 的完成通知当场发出，真正在跑的东西跑完不再叫醒谁（2026-09-18 一个门禁分诊员这样起门禁与三个缓存计时器，
 #      records/2026-09-16-subagent拆分提案.md 第三十三节）。认的形态：`disown`、`coproc`、`setsid -f/--fork`、同一行带单独 `&` 的 `nohup`、
@@ -28,7 +28,7 @@
 # `records/2026-09-16-subagent拆分提案.md` 第二十五节）；喂给 bash / sh 的 heredoc 正文照查。其余只看顶层命令文本，
 # 引号里当数据写的等待循环也会被记一条，主 agent 看了判断即可。
 #
-# 拒绝四种（退出 2，stderr 写原因与出路；拒了不记检出）。它们拒的是一种危险写法、在执行之前，不停在跑的任务，
+# 拒绝五种（退出 2，stderr 写原因与出路；拒了不记检出）。它们拒的是一种危险写法、在执行之前，不停在跑的任务，
 # 是 .claude/singlefs-ai-sop/rules/command-safety.md「检出和处置分开」那一条写明的例外（「拒绝某一种危险写法的钩子不在此列」）。
 # ① 命令位置上起看门狗——`research/scripts/watch.sh`（带 `--selftest`、`--report`、`--dry-run` 的不盯，不算）
 #   或 `research/scripts/agent-watch.py watch`——而 run_in_background 不是 true，或整条命令里有单独的 `&`、`nohup`、`disown`、`setsid`、
@@ -68,25 +68,43 @@
 #   `&` 之后按进程号轮询（`tail --pid`、`while kill -0`）不算 wait，照拒。起了几个 `&`、之后的 `wait "$pid"` 只等了其中一个的，
 #   机器分不出等全没有，不拒，照检出 ② 记一条。引号里当数据写的、写进文件的 heredoc 正文、注释里的不拒。
 #   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法。
+# ⑤ 命令位置上会整份覆盖 `research/results/` 下一个已存在、又没进 git（`git ls-files --error-unmatch` 失败）的文件：重定向 `>`、`>|`、`&>`、`>& 文件`
+#   （前面带 fd 号的 `2>` 一样截断，一样算）、不带 `-a` / `--append` 的 `tee` 写的每个文件、`cp` / `mv` / `install` 的目标（`-t 目录`、或目标是已有目录、
+#   以 `/` 结尾的，写的是 目录/源的文件名，源里的通配按这一刻的文件展开）。2026-09-25 E158 第九段执行员用 `cp` 把 s8 同名的三份未跟踪产物整份覆盖，
+#   旧字节找不回（records/2026-09-16-subagent拆分提案.md 第四十节那张表第 24 行）；write-guard.sh 只拦 Write / Edit 工具，拦不到 shell。
+#   前台、run_in_background 一样拒，主 agent 与子 agent 都拒。
+#   放行：`>>`、`&>>`、`tee -a` 追加；目标不存在（新文件名）；目标已进 git（`git add` 过就算，git 兜得住）；目标不在仓库根的 `research/results/` 下；
+#   `2>&1`、`>&2`、`>&-` 这类 fd 复制与关闭；`cp` / `mv` 带 `-n`、`--no-clobber`、`--update=none`、`-b`、`--backup` 的（已有的目标不被整份盖掉）；`install -d`；
+#   同一条命令里排在写之前 `git add` 过、或被 `mv` 挪走的那个文件（拒绝信息给的两条出路写进同一条命令也认）。
+#   目标按命令里的 cd / pushd 跟着算；cd 算不出来（`cd "$变量"`、`cd -`、`popd`）或没有 cd 时按仓库根（这个 hook 所在的仓，`.claude/hooks/` 往上两级）解析。
+#   这条命令里前面独立赋过值的变量（`out=…`、`export out=…`）在目标里的 `$out`、`${out}` 先代入。
+#   目标这一刻算不出来（没赋过值的变量、命令替换、通配、花括号展开）的不拦：算不出的段换成 `*` 去对 research/results/ 下已有的文件，
+#   对得上没进 git 的就记一条检出（对不上的一定是新文件名，不记）；git 自己出错、判不了进没进 git 的也只记检出。
+#   每一层（`bash -c '…'`、喂给 shell 的 heredoc 正文、命令替换与进程替换）都判，nice / timeout / env / sudo 这类前缀、`capped.sh N` 与 `bash 包装脚本` 剥开往里找；
+#   引号里当数据写的、写进文件的 heredoc 正文、注释里的不拒。
+#   判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法、xargs 与 `find -exec` 起的 cp、`cp -r` 整目录拷进已有目录时里面逐个文件、
+#   `dd of=`、`sed -i`、`sort -o`、`rsync`、`ln -f` 与 python 里 `open(…, 'w')` 这类别的写法；圆括号子 shell 里的 cd 当成对后面的命令也生效；
+#   不看 hook 输入里的 cwd（没有 cd 的相对路径一律按仓库根解析）。
 #
 # 切词、切简单命令、认命令位置、剥前缀与包装、跟 cd：同目录的 lib_shell_words.py，与 heavy-test-guard.sh 共用一份，按文件路径导入，
 # 这里只留检出与拒绝的判定；那几个函数在 .claude/hooks/ 别的文件里再定义一份，门禁 63 号判红。
 # 读不到它时这条命令照常执行，stderr 报一句并记一条检出。
 #
-#   bash-command-detector.sh             # 从 stdin 读 hook 的 JSON；拒绝四种之一退出 2（拒绝），其余退出 0
-#   bash-command-detector.sh --selftest  # 走一遍检出、不检出、拒绝与放行；BASH_COMMAND_DETECTOR_DISABLE_CHECK=1（两种检出与后三种拒绝都关）、
+#   bash-command-detector.sh             # 从 stdin 读 hook 的 JSON；拒绝五种之一退出 2（拒绝），其余退出 0
+#   bash-command-detector.sh --selftest  # 走一遍检出、不检出、拒绝与放行；BASH_COMMAND_DETECTOR_DISABLE_CHECK=1（两种检出、后四种拒绝与 ⑤ 的检出都关）、
 #                                        # BASH_COMMAND_DETECTOR_DISABLE_SELF_BACKGROUND=1（只关第三种）或
 #                                        # BASH_COMMAND_DETECTOR_KEEP_HEREDOC_BODIES=1（前两种不剥 heredoc 正文）或
 #                                        # BASH_COMMAND_DETECTOR_ALLOW_WATCHDOG_MISUSE=1（起看门狗的错误写法也放行）或
 #                                        # BASH_COMMAND_DETECTOR_ALLOW_FOREGROUND_WAIT_LOOP=1（前台没超时的等待循环也放行）或
 #                                        # BASH_COMMAND_DETECTOR_ALLOW_DETACHING=1（把活放出追踪的写法也放行）或
 #                                        # BASH_COMMAND_DETECTOR_ALLOW_UNWAITED_AMPERSAND=1（run_in_background 里后面没有 wait 的单独 & 也放行）或
+#                                        # BASH_COMMAND_DETECTOR_ALLOW_RESULTS_OVERWRITE=1（整份覆盖 research/results/ 下未跟踪产物也放行、也不记检出）或
 #                                        # BASH_COMMAND_DETECTOR_REFUSE_EVERY_WAIT_LOOP=1（外层 timeout 与 run_in_background 都不算，等待循环一律拒）时自检必须判红
 set -uo pipefail
 HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
 # python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 agent-write-scope.sh 同一个坑）。
 python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
-import importlib.util, json, os, re, shutil, subprocess, sys, tempfile
+import glob, importlib.util, json, os, re, shutil, subprocess, sys, tempfile
 from datetime import datetime, timezone
 from typing import NamedTuple
 
@@ -148,10 +166,10 @@
         findings.append("run_in_background 里又自己放后台（nohup / setsid / disown / 后面没有 wait 的 &）：完成通知当场发出，跑完的那个不会叫醒你")
     return findings
 
-def record(hook_input, detections_path):
-    """检出就追加一行；返回写了几行。永远不拦命令。"""
+def record(hook_input, detections_path, extra_findings=()):
+    """检出就追加一行；返回写了几行。永远不拦命令。extra_findings 是入口另外判出、只记不拦的（⑤ 目标算不出、git 判不了的）。"""
     tool_input = hook_input.get("tool_input") or {}
-    findings = findings_for(tool_input.get("command") or "", bool(tool_input.get("run_in_background")))
+    findings = findings_for(tool_input.get("command") or "", bool(tool_input.get("run_in_background"))) + list(extra_findings)
     if not findings:
         return 0
     append_detection(hook_input, findings, detections_path)
@@ -439,6 +457,371 @@
         return []
     return unwaited_background_jobs(command)
 
+# ⑤ 整份覆盖 research/results/ 下未跟踪的产物：按命令位置认会整份覆盖文件的写法，跟着 cd 与这条命令里赋过的变量算目标
+OVERWRITING_REDIRECTS = {">", ">|", "&>", ">&"}   # >> 与 &>> 是追加；>& 后面跟 fd 号或 - 时是复制、关闭 fd，不写文件
+FILE_DESCRIPTOR_WORD = re.compile(r"^(?:\d+-?|-)$")
+ASSIGNED_VARIABLE = re.compile(r"\$(?:\{([A-Za-z_][A-Za-z0-9_]*)\}|([A-Za-z_][A-Za-z0-9_]*))")
+# 目标词里这一刻算不出的段：变量（含 ${…} 的各种展开与特殊参数）、花括号展开；命令替换另由共用模块的 substitution_span 认
+UNKNOWN_PIECE = re.compile(r"\$\{[^}]*\}|\$[A-Za-z_][A-Za-z0-9_]*|\$[0-9@*#?$!-]|\{[^{}/]*(?:,|\.\.)[^{}/]*\}")
+GLOB_CHARACTERS = set("*?[")
+RESULTS_DIRECTORY_WORDS = ("research", "results")
+# cp / mv / install：命令名 → 要带一个值的短选项；带值的长选项（不写 = 时值是下一个词）
+COPY_SHORT_OPTIONS_WITH_VALUE = {"cp": "tS", "mv": "tS", "install": "tSgmo"}
+COPY_LONG_OPTIONS_WITH_VALUE = {"--target-directory", "--suffix", "--no-preserve", "--sparse", "--group", "--mode", "--owner", "--strip-program"}
+GIT_GLOBAL_OPTIONS_WITH_VALUE = {"-C", "-c", "--git-dir", "--work-tree", "--namespace", "--config-env"}
+GIT_ADD_OPTIONS_THAT_DO_NOT_KEEP_BYTES = {"-n", "--dry-run", "-u", "--update", "-N", "--intent-to-add", "-p", "--patch", "-i", "--interactive", "-e", "--edit", "--refresh"}
+
+class OverwriteCandidate(NamedTuple):
+    form: str                            # 认出的写法：>、>|、&>、>&、tee、cp、mv、install
+    target: str                          # 目标词（这条命令里前面赋过值的变量已代入）
+    directory: str | None                # 那一刻的当前目录；cd 算不出来是 None，按仓库根解析
+    sources: tuple = ()                  # cp / mv / install 的源
+    into_directory: bool | None = False  # True：-t 给的目录；None：看目标是不是已有目录、以 / 结尾；False：目标就是写的那个文件
+
+class PreservingStep(NamedTuple):
+    """同一条命令里让旧字节留住的一步：git add 的路径（进了 git）、mv 挪走的源（换了名字还在）；排在它后面再写这些路径不算覆盖。"""
+    words: tuple
+    directory: str | None
+
+class CopyArguments(NamedTuple):
+    operands: list                 # 非选项参数：源在前，目标在最后（给了 -t 时全是源）
+    target_directory: str | None   # -t / --target-directory 给的目录
+    treats_target_as_file: bool    # -T / --no-target-directory
+    keeps_existing_target: bool    # -n / --no-clobber / --update=none… / -b / --backup：已有的目标不被整份盖掉
+    creates_directories: bool      # install -d / --directory：参数全是要建的目录
+
+def substitute_assigned_variables(word, variables):
+    """`$名字`、`${名字}` 换成这条命令里前面赋过的值；没赋过的变量、命令替换、特殊参数照留。"""
+    return ASSIGNED_VARIABLE.sub(lambda match: variables.get(match.group(1) or match.group(2), match.group(0)), word)
+
+def tee_files(arguments):
+    """tee 的参数：带 -a / --append 的交 []（追加），否则交它写的每个文件（`-` 是标准输出，不算）。"""
+    files, options_ended = [], False
+    for argument in arguments:
+        if options_ended or argument == "-" or not argument.startswith("-"):
+            if argument != "-":
+                files.append(argument)
+        elif argument == "--":
+            options_ended = True
+        elif argument.startswith("--"):
+            if len(argument) >= 3 and "--append".startswith(argument):
+                return []
+        elif "a" in argument[1:]:
+            return []
+    return files
+
+def parse_copy_arguments(name, arguments):
+    """cp / mv / install 的参数按 GNU 的选项规矩切开（选项可以排在文件名后面，`--` 之后全是文件名）。"""
+    operands, target_directory, treats_target_as_file, keeps_existing_target, creates_directories = [], None, False, False, False
+    short_options_with_value = COPY_SHORT_OPTIONS_WITH_VALUE[name]
+    position = 0
+    while position < len(arguments):
+        argument = arguments[position]
+        position += 1
+        if argument == "--":
+            operands += arguments[position:]
+            break
+        if argument == "-" or not argument.startswith("-"):
+            operands.append(argument)
+            continue
+        if argument.startswith("--"):
+            option, has_value, value = argument.partition("=")
+            if option in COPY_LONG_OPTIONS_WITH_VALUE and not has_value:
+                value = arguments[position] if position < len(arguments) else ""
+                position += 1
+            if option == "--target-directory":
+                target_directory = value
+            elif option == "--no-target-directory":
+                treats_target_as_file = True
+            elif option in ("--no-clobber", "--backup") or (option == "--update" and value.startswith("none")):
+                keeps_existing_target = True
+            elif option == "--directory" and name == "install":
+                creates_directories = True
+            continue
+        letters = argument[1:]
+        for index, letter in enumerate(letters):
+            if letter in short_options_with_value:
+                value = letters[index + 1:]
+                if not value:
+                    value = arguments[position] if position < len(arguments) else ""
+                    position += 1
+                if letter == "t":
+                    target_directory = value
+                break
+            if letter == "T":
+                treats_target_as_file = True
+            elif letter in "nb":
+                keeps_existing_target = True
+            elif letter == "d" and name == "install":
+                creates_directories = True
+    return CopyArguments(operands, target_directory, treats_target_as_file, keeps_existing_target, creates_directories)
+
+def git_added_paths(arguments):
+    """git 的参数：是 `git [全局选项] add …` 且真把内容放进 git（不是 -n、-u、-N、-p 这类）时，交回 (-C 给的目录列表, 加进去的路径)；否则交 None。
+    `-A` / `--all` 不带路径时加的是整个工作树，交 ["."]（按 -C 与当前目录算，仓库根上跑时就是整个仓）。"""
+    position, change_directories = 0, []
+    while position < len(arguments) and arguments[position].startswith("-"):
+        option = arguments[position].partition("=")[0]
+        has_value_inline = "=" in arguments[position]
+        position += 1
+        if option in GIT_GLOBAL_OPTIONS_WITH_VALUE and not has_value_inline:
+            if option == "-C" and position < len(arguments):
+                change_directories.append(arguments[position])
+            position += 1
+    if arguments[position:position + 1] != ["add"]:
+        return None
+    paths, adds_everything, options_ended = [], False, False
+    for argument in arguments[position + 1:]:
+        if options_ended or not argument.startswith("-"):
+            paths.append(argument)
+        elif argument == "--":
+            options_ended = True
+        elif argument in GIT_ADD_OPTIONS_THAT_DO_NOT_KEEP_BYTES:
+            return None
+        elif argument in ("-A", "--all"):
+            adds_everything = True
+    if not paths and adds_everything:
+        paths = ["."]
+    return change_directories, paths
+
+def follow_simple_command(words, directory, variables, steps, depth):
+    """一条简单命令：会整份覆盖文件的写法（tee、cp、mv、install）与让旧字节留住的一步（git add、mv 挪走源）追加进 steps；
+    跟着 cd / pushd 换目录，跟着独立的赋值与 export 记变量（改 variables）；bash -c 的那段代码按这一刻的目录与变量递归进去。交回这条之后的当前目录。"""
+    skipped = shell_words.skip_prefixes(words)
+    if skipped.command_position is None:
+        for name, value in skipped.assignments.items():
+            variables[name] = substitute_assigned_variables(value, variables)
+        return directory
+    name, arguments = None, []
+    for _, _, name, arguments in wrapped_commands(words):
+        pass
+    if name is None:
+        return directory
+    arguments = [substitute_assigned_variables(argument, variables) for argument in arguments]
+    if name == "export":
+        for argument in arguments:
+            assignment = shell_words.ASSIGNMENT.match(argument)
+            if assignment:
+                variables[assignment.group(1)] = assignment.group(2)
+    elif name in ("cd", "pushd"):
+        return shell_words.changed_directory(directory, arguments)
+    elif name == "popd":
+        return None
+    elif name == "tee":
+        steps += [OverwriteCandidate("tee", file, directory) for file in tee_files(arguments)]
+    elif name in COPY_SHORT_OPTIONS_WITH_VALUE:
+        parsed = parse_copy_arguments(name, arguments)
+        if parsed.creates_directories:
+            return directory
+        sources = tuple(parsed.operands if parsed.target_directory is not None else parsed.operands[:-1])
+        if not parsed.keeps_existing_target:
+            if parsed.target_directory is not None:
+                steps.append(OverwriteCandidate(name, parsed.target_directory, directory, sources, True))
+            elif len(parsed.operands) >= 2:
+                steps.append(OverwriteCandidate(name, parsed.operands[-1], directory, sources, False if parsed.treats_target_as_file else None))
+        if name == "mv" and not parsed.keeps_existing_target and sources:
+            steps.append(PreservingStep(sources, directory))
+    elif name == "git":
+        added = git_added_paths(arguments)
+        if added is not None:
+            git_directory = directory
+            for change in added[0]:
+                git_directory = shell_words.changed_directory(git_directory, [change])
+            steps.append(PreservingStep(tuple(added[1]), git_directory))
+    elif name in shell_words.SHELL_NAMES:
+        code = shell_words.shell_invocation(arguments).code_string
+        if code is not None:
+            steps += overwrite_steps(code, directory, variables, depth + 1)
+    return directory
+
+def command_pieces(tokens):
+    """一层的记号按命令分隔符切成一段段：一段至多一条简单命令，连同它的重定向与替换正文；只有重定向、没有词的也成一段
+    （`(…) > 文件`、`{ …; } > 文件` 右括号之后那段，共用模块的 simple_commands 不交它）。数组赋值 `名字=(…)` 的括号留在段里。"""
+    pieces, piece, position = [], [], 0
+    while position < len(tokens):
+        is_operator, text = tokens[position]
+        if (is_operator is False and shell_words.ARRAY_ASSIGNMENT_START.match(text)
+                and tokens[position + 1:position + 2] == [(True, "(")]):
+            end = position + 2
+            while end < len(tokens) and tokens[end] != (True, ")"):
+                end += 1
+            piece += tokens[position:end + 1]
+            position = end + 1
+            continue
+        if is_operator and text in shell_words.COMMAND_SEPARATORS:
+            if piece:
+                pieces.append(piece)
+            piece = []
+        else:
+            piece.append(tokens[position])
+        position += 1
+    if piece:
+        pieces.append(piece)
+    return pieces
+
+def overwrite_steps(text, directory, variables=None, depth=0):
+    """整条命令每一层（顶层、bash -c 的代码、喂给 shell 的 heredoc 正文、命令替换与进程替换）命令位置上，会整份覆盖文件的写法
+    （OverwriteCandidate：重定向、tee、cp、mv、install）与让旧字节留住的一步（PreservingStep），按执行的先后交回。
+    喂给 shell 的 heredoc 正文留在原位按行切（共用模块的 strip_data_heredocs 只剥喂给别的命令的）；directory 是开头的当前目录。"""
+    if depth > shell_words.MAXIMUM_NESTING:
+        return []
+    variables = dict(variables or {})
+    steps = []
+    for piece in command_pieces(shell_words.shell_tokens(shell_words.strip_data_heredocs(text))):
+        for is_operator, substitution in piece:
+            if is_operator is None:
+                steps += overwrite_steps(substitution, directory, variables, depth + 1)
+        for (is_operator, operator), (next_is_operator, target) in zip(piece, piece[1:]):
+            if (is_operator and operator in OVERWRITING_REDIRECTS and next_is_operator is False
+                    and not (operator == ">&" and FILE_DESCRIPTOR_WORD.match(target))):
+                steps.append(OverwriteCandidate(operator, substitute_assigned_variables(target, variables), directory))
+        commands = shell_words.simple_commands(piece)[0]
+        if commands and commands[0]:
+            directory = follow_simple_command(commands[0], directory, variables, steps, depth)
+    return steps
+
+def target_pattern(word):
+    """目标词里这一刻算不出的段（没赋过值的变量、命令替换、花括号展开）换成 *：交回 (样式, 有没有算不出的段, 有没有通配)。"""
+    pieces, position, has_unknown_piece, has_glob = [], 0, False, False
+    while position < len(word):
+        if word.startswith("$(", position) or word[position] == "`":
+            position = shell_words.substitution_span(word, position)[0]
+            pieces.append("*")
+            has_unknown_piece = True
+            continue
+        unknown = UNKNOWN_PIECE.match(word, position)
+        if unknown:
+            pieces.append("*")
+            has_unknown_piece = True
+            position = unknown.end()
+            continue
+        has_glob = has_glob or word[position] in GLOB_CHARACTERS
+        pieces.append(word[position])
+        position += 1
+    return "".join(pieces), has_unknown_piece, has_glob
+
+def is_inside(path, directory):
+    """path 在 directory 里面（不含 directory 自己）；按字面比、解开符号链接之后比，有一边在里面就算。"""
+    return any(os.path.normpath(candidate).startswith(os.path.normpath(parent) + os.sep)
+               for candidate, parent in ((path, directory), (os.path.realpath(path), os.path.realpath(directory))))
+
+def destination_words(candidate, base):
+    """这一步写到的目标词：重定向与 tee 是目标本身；cp / mv / install 写进目录时是 目录/每个源的文件名（源里的通配按这一刻的文件展开，
+    一个都展不开的 cp 会报错、什么都不写；源里有算不出的段，文件名换成 *）。"""
+    into_directory = candidate.into_directory
+    if into_directory is None:
+        pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(candidate.target))
+        into_directory = candidate.target.endswith("/") or (
+            not has_unknown_piece and not has_glob and os.path.isdir(shell_words.resolve_path(base, pattern)))
+    if not into_directory:
+        return [candidate.target]
+    names = []
+    for source in candidate.sources:
+        pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(source))
+        if has_unknown_piece:
+            names.append("*")
+        elif has_glob:
+            names += [os.path.basename(match.rstrip("/")) for match in sorted(glob.glob(shell_words.resolve_path(base, pattern)))]
+        else:
+            names.append(os.path.basename(source.rstrip("/")))
+    return [f"{candidate.target.rstrip('/')}/{name}" for name in names]
+
+def resolve_destination(word, base, repository_root):
+    """目标词按 base 解开：交回 (确定的绝对路径, [])；有算不出的段或通配时交回 (None, 样式在 research/results/ 下对得上的已有文件)，
+    样式落不进 research/results/ 的交 (None, [])。`"$变量"/research/results/…` 这类变量在前的，从 research/results/ 那一段起按仓库根接。"""
+    results_directory = os.path.join(repository_root, *RESULTS_DIRECTORY_WORDS)
+    pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(word))
+    path = shell_words.resolve_path(base, pattern)
+    if not has_unknown_piece and not has_glob:
+        return path, []
+    anchor = pattern.find("/".join(RESULTS_DIRECTORY_WORDS) + "/")
+    if anchor > 0 and "*" in pattern[:anchor] and not is_inside(path, results_directory):
+        path = os.path.join(repository_root, pattern[anchor:])
+    if not is_inside(path, results_directory):
+        return None, []
+    return None, sorted(match for match in glob.glob(path) if os.path.isfile(match))
+
+def preserved_paths(step, base):
+    """PreservingStep 里的路径按 base 解开（通配按这一刻的文件展开，有算不出的段的不算）。"""
+    paths = []
+    for word in step.words:
+        pattern, has_unknown_piece, has_glob = target_pattern(os.path.expanduser(word))
+        if has_unknown_piece:
+            continue
+        resolved = shell_words.resolve_path(base, pattern)
+        paths += glob.glob(resolved) if has_glob else [resolved]
+    return [os.path.normpath(path) for path in paths]
+
+def is_preserved(path, preserved):
+    """path 是同一条命令里前面 git add 过、或 mv 挪走了的那个文件（或在那样一个目录里面）。"""
+    return any(os.path.normpath(path) == kept or is_inside(path, kept) for kept in preserved)
+
+def git_tracks(repository_root, path):
+    """`git ls-files --error-unmatch`：进过 git 交 True，没进交 False，git 自己出错（不是仓、路径在仓外、超时）交 None。"""
+    try:
+        completed = subprocess.run(["git", "-C", repository_root, "ls-files", "--error-unmatch", "--", path],
+                                   capture_output=True, text=True, timeout=20)
+    except (OSError, subprocess.TimeoutExpired):
+        return None
+    return {0: True, 1: False}.get(completed.returncode)
+
+def untracked_among(repository_root, paths):
+    """几份已有的文件里没进 git 的，一次 git ls-files 查完；git 自己出错交 None。"""
+    if not paths:
+        return []
+    try:
+        completed = subprocess.run(["git", "-C", repository_root, "ls-files", "-z", "--full-name", "--", *paths],
+                                   capture_output=True, text=True, timeout=20)
+    except (OSError, subprocess.TimeoutExpired):
+        return None
+    if completed.returncode != 0:
+        return None
+    top = os.path.realpath(repository_root)
+    tracked = {os.path.normpath(os.path.join(top, name)) for name in completed.stdout.split("\0") if name}
+    return [path for path in paths if os.path.normpath(os.path.realpath(path)) not in tracked]
+
+def results_overwrite_verdict(command, repository_root):
+    """整条命令里会整份覆盖 research/results/ 下文件的每一步：目标确定、已存在、没进 git、前面也没被 git add 或 mv 挪走的，交进要拒的
+    （`写法 相对仓库根的路径`）；目标这一刻算不出、样式对得上已有的未跟踪文件的，与 git 判不了的，交进要记检出的。"""
+    results_directory = os.path.join(repository_root, *RESULTS_DIRECTORY_WORDS)
+    preserved, refused, noted = [], [], []
+    for step in overwrite_steps(command, repository_root):
+        base = step.directory if step.directory is not None else repository_root
+        if isinstance(step, PreservingStep):
+            preserved += preserved_paths(step, base)
+            continue
+        for word in destination_words(step, base):
+            exact, matches = resolve_destination(word, base, repository_root)
+            if exact is not None:
+                if not is_inside(exact, results_directory) or not os.path.isfile(exact) or is_preserved(exact, preserved):
+                    continue
+                tracked = git_tracks(repository_root, exact)
+                relative = os.path.relpath(exact, repository_root)
+                if tracked is False:
+                    refused.append(f"{step.form} {relative}")
+                elif tracked is None:
+                    noted.append(f"判不了 {relative} 进没进 git（git ls-files 出错），{step.form} 覆盖它这一步没拦")
+                continue
+            untracked = untracked_among(repository_root, [match for match in matches if not is_preserved(match, preserved)])
+            if untracked is None:
+                noted.append(f"判不了 {step.form} {word} 对得上的文件进没进 git（git ls-files 出错），没拦")
+            elif untracked:
+                shown = "、".join(os.path.relpath(path, repository_root) for path in untracked[:3])
+                noted.append(f"{step.form} {word} 的目标这一刻算不出来（变量、命令替换或通配），没拦；"
+                             f"它对得上 research/results/ 下 {len(untracked)} 份已有的未跟踪文件：{shown}{' 等' if len(untracked) > 3 else ''}")
+    return list(dict.fromkeys(refused)), list(dict.fromkeys(noted))
+
+def results_overwrite_refusal(command, repository_root):
+    """交回 (要拒的写法与目标, 要记的检出)；前台、run_in_background 一样判，主 agent 与子 agent 都判。"""
+    if (os.environ.get("BASH_COMMAND_DETECTOR_DISABLE_CHECK") == "1"
+            or os.environ.get("BASH_COMMAND_DETECTOR_ALLOW_RESULTS_OVERWRITE") == "1"):
+        return [], []
+    return results_overwrite_verdict(command, repository_root)
+
+def repository_root_of(hook_dir):
+    """这个 hook 所在的仓：`.claude/hooks/` 往上两级。"""
+    return os.path.realpath(os.path.join(hook_dir, os.pardir, os.pardir))
+
 def selftest(hook_dir):
     if shell_words is None:
         print(f"  ✗ 自检：读不到共用切词模块 {shell_words_library_path(hook_dir)}（{shell_words_error!r}）")
@@ -613,6 +996,122 @@
     ]
     for label, command, background, want in unwaited_ampersand_cases:
         results.append((f"单独的 &:{label}", want, 1 if unwaited_ampersand_refusal(command, background) else 0))
+    # 整份覆盖 research/results/ 下未跟踪的产物：在临时 git 仓里造已跟踪与未跟踪的文件，判定与真实入口都指着它，不碰真仓
+    repository = os.path.realpath(os.path.join(work, "repository"))
+    os.makedirs(os.path.join(repository, "research", "results", "sub"))
+    elsewhere = os.path.join(work, "elsewhere")        # 仓外同样的相对路径：cd 过去之后写的是它，不是仓里那份
+    sources = os.path.join(work, "sources")            # cp 进目录时的源，文件名与仓里未跟踪的那份相同
+    not_a_repository = os.path.join(work, "not-a-repository")
+    for directory in (os.path.join(elsewhere, "research", "results"), sources, os.path.join(not_a_repository, "research", "results")):
+        os.makedirs(directory)
+    for path in (*(os.path.join(repository, relative) for relative in (
+                     "research/results/untracked.out", "research/results/tracked.out", "research/results/1",
+                     "research/results/sub/deep.out", "scratch.log")),
+                 os.path.join(elsewhere, "research", "results", "untracked.out"), os.path.join(sources, "untracked.out"),
+                 os.path.join(work, "outside.log"), os.path.join(not_a_repository, "research", "results", "untracked.out")):
+        with open(path, "w", encoding="utf-8") as handle:
+            handle.write("旧字节\n")
+    git_ready = (subprocess.run(["git", "init", "-q", repository], capture_output=True).returncode == 0
+                 and subprocess.run(["git", "-C", repository, "add", "research/results/tracked.out"], capture_output=True).returncode == 0)
+    results.append(("覆盖产物:临时 git 仓建得起来（git init、git add）", 1, int(git_ready)))
+    outside = os.path.join(work, "outside.log")
+    # (说明, 命令, 该不该拒绝)；目标按临时仓的根解析
+    results_overwrite_cases = [
+        ("> 覆盖未跟踪的产物", "python3 x.py > research/results/untracked.out", 1),
+        ("tee 不带 -a", "cargo run --release | tee research/results/untracked.out", 1),
+        ("cp 的目标（现场原句的形态）", "cp /tmp/claude-1000/e158-s9/config-evidence-runs/yi-ring-retained.out research/results/untracked.out", 1),
+        ("bash -c 里的 >", "bash -c 'python3 x.py > research/results/untracked.out'", 1),
+        (">|", "python3 x.py >| research/results/untracked.out", 1),
+        ("&>", "cargo test &> research/results/untracked.out", 1),
+        ("2> 同样截断", "cargo test 2> research/results/untracked.out", 1),
+        (">& 后面跟文件名", "cargo test >& research/results/untracked.out", 1),
+        ("tee -i 不是追加", "make | tee -i research/results/untracked.out", 1),
+        ("tee 写两份、其中一份是未跟踪的", "make | tee research/results/fresh.out research/results/untracked.out", 1),
+        ("mv 的目标", "mv /tmp/fresh.out research/results/untracked.out", 1),
+        ("install 的目标", "install -m 644 fresh.out research/results/untracked.out", 1),
+        ("cp 进已有目录、源的文件名撞上", f"cp {sources}/untracked.out research/results/", 1),
+        ("cp -t 目录", f"cp -t research/results {sources}/untracked.out", 1),
+        ("cp 源里的通配按这一刻展开", f"cp {sources}/*.out research/results", 1),
+        ("cd 进去之后写相对路径", "cd research/results && python3 x.py > untracked.out", 1),
+        ("cd 进子目录之后写", "cd research/results/sub && echo > deep.out", 1),
+        ("绝对路径", f"echo > {repository}/research/results/untracked.out", 1),
+        ("喂给 bash 的 heredoc 正文里的 >", "bash <<'EOF'\npython3 x.py > research/results/untracked.out\nEOF", 1),
+        ("命令替换里的 >", "lines=$(python3 x.py > research/results/untracked.out; echo done)", 1),
+        ("cat 把 heredoc 写进未跟踪的产物", "cat > research/results/untracked.out <<'EOF'\n一行\nEOF", 1),
+        ("花括号整组重定向", "{ echo a; echo b; } > research/results/untracked.out", 1),
+        ("圆括号整组重定向", "(echo a; echo b) > research/results/untracked.out", 1),
+        ("timeout、capped.sh 包着的 bash -c 整条重定向", "timeout 60 bash research/scripts/capped.sh 4 bash -c 'cargo run' > research/results/untracked.out", 1),
+        ("前面赋过值的变量代入", 'out=research/results/untracked.out; python3 x.py > "$out"', 1),
+        ("export 过的变量代入", 'export OUT=research/results/untracked.out; python3 x.py > "${OUT}"', 1),
+        ("cd 到变量路径算不出来、按仓库根解析", 'cd "$scratch" && python3 x.py > research/results/untracked.out', 1),
+        ("进程替换里的 tee", "make > >(tee research/results/untracked.out)", 1),
+        ("整个循环重定向", "for arm in a b; do python3 x.py $arm; done > research/results/untracked.out", 1),
+        ("sudo 包着的 tee", "echo x | sudo tee research/results/untracked.out", 1),
+        ("git add 的是另一份，这一份照拒", "git add research/results/tracked.out && python3 x.py > research/results/untracked.out", 1),
+        (">> 追加", "python3 x.py >> research/results/untracked.out", 0),
+        ("&>> 追加", "cargo test &>> research/results/untracked.out", 0),
+        ("写新文件名", "python3 x.py > research/results/untracked-2026-09-25.out", 0),
+        ("写已跟踪的文件（git 兜得住）", "python3 x.py > research/results/tracked.out", 0),
+        ("/tmp 目标（已存在、不在任何仓里）", f"python3 x.py > {outside}", 0),
+        ("仓里 research/results/ 外面的未跟踪文件", "python3 x.py > scratch.log", 0),
+        ("2>&1 不是写一个叫 1 的文件（research/results/1 未跟踪地存在）", "cd research/results && python3 x.py >> untracked.out 2>&1", 0),
+        ("tee -a", "make | tee -a research/results/untracked.out", 0),
+        ("tee --append", "make | tee --append research/results/untracked.out", 0),
+        ("cp -n 不盖已有的", "cp -n fresh.out research/results/untracked.out", 0),
+        ("cp --backup 旧的留成备份", "cp --backup=numbered fresh.out research/results/untracked.out", 0),
+        ("install -d 建目录", "install -d research/results/untracked.out", 0),
+        ("cp 的源是产物、目标在外面", "cp research/results/untracked.out /tmp/copy.out", 0),
+        ("读产物、写别处", "sort < research/results/untracked.out > /tmp/sorted.out", 0),
+        ("cd 到仓外之后同样的相对路径", f"cd {elsewhere} && python3 x.py > research/results/untracked.out", 0),
+        ("同一条命令里先挪到带日期的旧名再写", "mv research/results/untracked.out research/results/untracked-2026-09-24-old.out && python3 x.py > research/results/untracked.out", 0),
+        ("同一条命令里先 git add 再写", "git add research/results/untracked.out && python3 x.py > research/results/untracked.out", 0),
+        ("引号里当数据写", "echo 'python3 x.py > research/results/untracked.out'", 0),
+        ("写进笔记的 heredoc 正文", "cat > note.md <<'EOF'\npython3 x.py > research/results/untracked.out\nEOF", 0),
+        ("注释里写的", "ls  # > research/results/untracked.out", 0),
+        ("目标里有算不出的变量：不拦（只记检出）", "for arm in a b; do python3 x.py > research/results/untracked$arm.out; done", 0),
+        ("目标里有命令替换、对不上已有文件", "python3 x.py > research/results/untracked-$(date +%F).out", 0),
+    ]
+    for label, command, want in results_overwrite_cases:
+        results.append((f"覆盖产物:{label}", want, 1 if git_ready and results_overwrite_refusal(command, repository)[0] else 0))
+    # 只记检出、不拦：(说明, 命令, 仓库根, 该不该记)
+    results_noted_cases = [
+        ("算不出的变量对得上一份未跟踪的产物", "for arm in a b; do python3 x.py > research/results/untracked$arm.out; done", repository, 1),
+        ("变量在前、research/results/ 在后", 'python3 x.py > "$CLAUDE_PROJECT_DIR"/research/results/untracked.out', repository, 1),
+        ("算不出的段对不上任何已有文件（一定是新文件名）", "for arm in a b; do python3 x.py > research/results/e-$arm.out; done", repository, 0),
+        ("命令替换当日期、对不上已有文件", "python3 x.py > research/results/untracked-$(date +%F).out", repository, 0),
+        ("对得上的只有已跟踪的", "for arm in a b; do python3 x.py > research/results/tracked$arm.out; done", repository, 0),
+        ("git 判不了（不是 git 仓）只记不拦", "python3 x.py > research/results/untracked.out", not_a_repository, 1),
+        ("拒了的不另记", "python3 x.py > research/results/untracked.out", repository, 0),
+    ]
+    for label, command, root, want in results_noted_cases:
+        refused_here, noted_here = results_overwrite_refusal(command, root) if git_ready else ([], [])
+        results.append((f"覆盖产物检出:{label}", want, int(bool(noted_here))))
+    results.append(("覆盖产物检出:git 判不了的那一条不拦", 0, int(bool(results_overwrite_refusal("python3 x.py > research/results/untracked.out", not_a_repository)[0]))))
+    # 走真实入口：把 hook 与共用切词模块拷进临时仓的 .claude/hooks/，仓库根取它往上两级
+    repository_hooks = os.path.join(repository, ".claude", "hooks")
+    os.makedirs(repository_hooks)
+    shutil.copy(os.path.join(hook_dir, "bash-command-detector.sh"), repository_hooks)
+    shutil.copy(shell_words_library_path(hook_dir), repository_hooks)
+    def through_entry(command, run_in_background=False):
+        before_entry = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
+        completed = subprocess.run(["bash", os.path.join(repository_hooks, "bash-command-detector.sh")], capture_output=True, text=True,
+                                   env=dict(os.environ, AGENT_HOOK_DETECTIONS=detections),
+                                   input=json.dumps({"tool_name": "Bash", "session_id": "s", "tool_input": {
+                                       "command": command, "run_in_background": run_in_background}}))
+        after_entry = sum(1 for _ in open(detections)) if os.path.exists(detections) else 0
+        return completed, after_entry - before_entry
+    scene, scene_recorded = through_entry("cp /tmp/claude-1000/e158-s9/config-evidence-runs/yi-ring-retained.out research/results/untracked.out")
+    results.append(("stdin:cp 覆盖未跟踪的产物拒绝（退出码 2）", 2, scene.returncode))
+    results.append(("stdin:拒绝时 stderr 写了出路（按日期另存、先 git add 或挪到带日期的旧名）", 1,
+                    int("✗" in scene.stderr and "→" in scene.stderr and "research/results/untracked.out" in scene.stderr
+                        and "按日期另存一个新文件名，旧的留着；确实要换掉旧产物，先 `git add` 它或者把它挪到带日期的旧名" in scene.stderr)))
+    results.append(("stdin:拒了的不记检出", 0, scene_recorded))
+    background_overwrite = through_entry("python3 x.py > research/results/untracked.out", True)[0]
+    results.append(("stdin:run_in_background 里的 > 覆盖同样拒绝（退出码 2）", 2, background_overwrite.returncode))
+    appended, appended_recorded = through_entry("python3 x.py >> research/results/untracked.out")
+    results.append(("stdin:>> 追加放行（退出码 0）、不记检出", (0, 0), (appended.returncode, appended_recorded)))
+    unresolved, unresolved_recorded = through_entry("for arm in a b; do python3 x.py > research/results/untracked$arm.out; done")
+    results.append(("stdin:目标算不出、对得上未跟踪产物的放行（退出码 0）并记一条检出", (0, 1), (unresolved.returncode, unresolved_recorded)))
     # 同走 run_in_background 的那一条：只记检出
     results.append(("等待循环:同一条走 run_in_background 记一条检出", 1,
                     record({"tool_name": "Bash", "session_id": "s", "tool_input": {"command": "until grep -q x f; do sleep 10; done", "run_in_background": True}},
@@ -700,9 +1199,11 @@
         print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
     if failures:
         print("    → 看 findings_for()、record()、watchdog_rejection()、wait_loop_refusal() / unbounded_wait_loops()、detaching_refusal() / detaching_forms()、"
-              "unwaited_ampersand_refusal() / unwaited_background_jobs() / jobs_with_endings() 与共用的 lib_shell_words.py；"
+              "unwaited_ampersand_refusal() / unwaited_background_jobs() / jobs_with_endings()、"
+              "results_overwrite_refusal() / results_overwrite_verdict() / overwrite_steps() / follow_simple_command() / destination_words() 与共用的 lib_shell_words.py；"
+              "「覆盖产物:临时 git 仓建得起来」红的是本机的 git 起不来，先看 git init 能不能跑；"
               "BASH_COMMAND_DETECTOR_DISABLE_CHECK、_DISABLE_SELF_BACKGROUND、_KEEP_HEREDOC_BODIES、_ALLOW_WATCHDOG_MISUSE、"
-              "_ALLOW_FOREGROUND_WAIT_LOOP、_REFUSE_EVERY_WAIT_LOOP、_ALLOW_DETACHING 或 _ALLOW_UNWAITED_AMPERSAND 设着的话这里本来就该红")
+              "_ALLOW_FOREGROUND_WAIT_LOOP、_REFUSE_EVERY_WAIT_LOOP、_ALLOW_DETACHING、_ALLOW_UNWAITED_AMPERSAND 或 _ALLOW_RESULTS_OVERWRITE 设着的话这里本来就该红")
         return 1
     summary = ("没超时的等待循环、run_in_background 里又自己放后台记进检出记录，普通命令、重定向里的 & 、前台的 & 、写进文件的 heredoc 正文与按模式找进程（归上游钩子）不记；"
                "起看门狗不用 run_in_background、或带 &、nohup、setsid、disown、丢进 /dev/null 的拒绝（退出码 2），run_in_background 只写 watch.sh 的、把名字当参数的、"
@@ -711,7 +1212,10 @@
                "不等结束的 systemd-run 前台后台都拒，不带 & 的 nohup、裸 setsid、systemd-run --wait、并行加 wait 与当数据写的放行；"
                "run_in_background 里以单独的 & 收尾、之后同一层同一对圆括号里没有 wait 的拒绝（结尾 &、& 后接 echo $!、capped.sh 包着的、"
                "子 shell、命令替换、bash -c 与喂给 shell 的 heredoc 里的，wait -n 与按 pid 轮询不算 wait），之后用 wait / wait \"$pid\" 等的、"
-               "&&、2>&1、|&、&>、当数据写的与前台的放行")
+               "&&、2>&1、|&、&>、当数据写的与前台的放行；在临时 git 仓里，整份覆盖 research/results/ 下已存在又没进 git 的文件拒绝（>、>|、&>、2>、>& 文件、"
+               "不带 -a 的 tee、cp / mv / install 的目标与 -t、写进已有目录，bash -c、heredoc、命令替换、进程替换、整组重定向里的，cd 跟着算、cd 算不出来按仓库根、"
+               "前面赋过值的变量代入），>>、&>>、tee -a、新文件名、已跟踪、仓外与 research/results/ 外、2>&1、cp -n / --backup、install -d、同一条命令里先 git add 或 mv 挪走的、"
+               "当数据写的放行；目标算不出而对得上未跟踪产物的、git 判不了的只记检出")
     print(f"  ✓ 自检通过（查了 {len(results)} 种）：{summary}")
     return 0
 
@@ -734,6 +1238,7 @@
             pass
         return 0
     tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
+    noted_overwrites = []
     try:
         reasons, calls = watchdog_rejection(tool_input.get("command") or "", tool_input.get("run_in_background"))
     except Exception as error:
@@ -778,13 +1283,25 @@
                   "确实要在前台轮询一个文件，外面套一层 timeout：`timeout <秒> bash -c 'until …; do sleep …; done'`。"
                   "这一道只拒这种写法，不停你在跑的任何东西", file=sys.stderr)
             return 2
+        try:
+            overwrites, noted_overwrites = results_overwrite_refusal(tool_input.get("command") or "", repository_root_of(hook_dir))
+        except Exception as error:
+            print(f"  ! bash-command-detector.sh 没判成整份覆盖 research/results/ 产物的写法（{error!r}），这条命令照常执行", file=sys.stderr)
+            overwrites, noted_overwrites = [], []
+        if overwrites:
+            print(f"  ✗ 要整份覆盖 research/results/ 下已存在、又没进 git 的产物（认出的写法与目标：{'；'.join(overwrites)}）："
+                  "旧字节没有任何一份副本，盖掉就找不回来", file=sys.stderr)
+            print("     → 怎么办：按日期另存一个新文件名，旧的留着；确实要换掉旧产物，先 `git add` 它或者把它挪到带日期的旧名"
+                  "（`git add` 或 `mv 旧名 带日期的旧名` 可以写在同一条命令里、排在写之前）。"
+                  "追加用 `>>` 或 `tee -a`，不拦。这一道只拒这种写法，不停你在跑的任何东西", file=sys.stderr)
+            return 2
     if reasons:
         print(f"  ✗ 看门狗起法不对（{'；'.join(reasons)}；认出的调用：{'、'.join(calls)}）：这样起的看门狗叫不醒主 agent，等于没盯", file=sys.stderr)
         print("     → 怎么办：看门狗用 Bash 的 run_in_background: true 起，命令只写 `bash research/scripts/watch.sh [--ack …] <agent 号，逗号分隔>`，"
               "不加 `&`、`nohup`、`disown`，不重定向输出（只盯自己起的长进程写 `bash research/scripts/watch.sh --processes`）", file=sys.stderr)
         return 2
     try:
-        record(hook_input, os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS)
+        record(hook_input, os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS, noted_overwrites)
     except Exception:
         return 0
     return 0
```

## 五、门禁 55、74 号相对 HEAD 的 diff（`gates-vs-head.diff`，2 个文件，整段进）

```diff
diff --git a/.claude/gate.d/55-qemu-first-transaction.sh b/.claude/gate.d/55-qemu-first-transaction.sh
index a281f68..2028e7d 100755
--- a/.claude/gate.d/55-qemu-first-transaction.sh
+++ b/.claude/gate.d/55-qemu-first-transaction.sh
@@ -1,22 +1,23 @@
 #!/usr/bin/env bash
-# gate-stage: QEMU 真设备上的第一个事务、发布 B 与第二个实例（两块 virtio 盘、设备侧独立录制、漏一道屏障与走页缓存两个对照必须判红）
+# gate-stage: QEMU 真设备上的第一个事务、发布 B、第二个实例、发布 D 与抬 F（两块 virtio 盘、设备侧独立录制、漏一道屏障与走页缓存两个对照必须判红）
 # 不声明 gate-covers：共享清单 2026-09-16 起没有「QEMU 真实负载」这一项（QEMU 移交给本工程），而「最终判据」要的是真实负载 + 崩溃注入 + checker 全绿，
 # 这一道今天只有真实负载与设备侧录制、没有崩溃注入，不冒充覆盖它。
 #
 # C6（块层语义假设写错）要的：「程序以为发了什么」与「盘上实际收到了什么」由两条不共享代码的路比。
 #   程序那一条 = 被测程序自己的录制器（宿主上同参数重跑，同字节）；
 #   盘那一条   = QEMU 的 blklogwrites 过滤节点在来宾之外按 dm-log-writes 格式记下的每个写（带数据）与每个 FLUSH。
-# 五次虚机跑（并行）：
+# 六次虚机跑（并行）：
 #   direct                          O_DIRECT 真写路：设备侧日志逐项等于程序的录制流（末尾至多一个关机 FLUSH），
 #                                   来宾块层的 FLUSH 数 = 屏障 + FUA 写数，冷重开读回文件，段序列与 E142 产物逐字相同，宿主从盘镜像再读回一次
 #   skip-first-transaction-barrier  盘 0 漏掉「单元 → journal 记录」那道屏障：设备侧比对必须判红，且红在盘 0
 #   page-cache                      读写走页缓存（阳性对照）：回写合并 / 重排写，设备侧比对必须判红
 #   second-transaction              第一个事务之后同一个进程里覆盖写一次（发布 B，里程碑「第二个事务」步 1）：冷重开择 (1, 4) 读回第二版
 #   second-instance                 发布 B 之后同一对盘冷重开、可写挂载（取号 → 写行 txg 5 → 暖机 txg 6、7）再发布 C（步 3）：冷重开择 (2, 8) 读回第三版
+#   raise-rollback-floor            second-instance 那条路走完，同一次挂载里再发布 D（txg 9，第四版），把 F 抬到上限 3、推两次空发布 txg 10、11
+#                                   （增补 2 收口表第 58 行）：冷重开择 (2, 11) 读回第四版
 #
-# ⚠️ 后两个模式的设备侧比对只核得到第一个事务那一段前缀：`first_transaction_device_log_check` 在宿主上只重跑第一个事务，
-# 发布 B / 可写挂载 / 发布 C 的写它算不出来。所以这两档判的是「第一个事务那段逐项对得上（divergence 的 program 侧为 None，
-# 说明前缀整段相等），而且盘上确实多收到了写」——B / 挂载 / C 的逐项比对还欠着，账在 C455 与里程碑「第二个事务」增补 2 收口表第 30 行。
+# 设备侧比对每一档都逐项比到这一档最后那次发布：`first_transaction_device_log_check` 的第一个参数是模式，宿主照模式重跑
+# 发布 B / 可写挂载与发布 C / 发布 D 与抬 F（`name=host_rerun` 行列出重跑了哪几段）。所以后三档照 direct 判：退出码 0、两块盘都没有分歧。
 #
 # 判别力样本 fixtures/55-qemu-first-transaction.sh/{red,green}（89 号跑）拿预录输出喂：样本目录里放一个 `.qemu-prerecorded` 标记文件，再按 `<模式>/out.txt`、`vm-exit`、
 # `check.txt`、`check-exit` 摆好某一轮真跑留下来的原样输出，本阶段就不起虚机、不编译，只拿同一段判定代码判它们
@@ -28,24 +29,26 @@ cd "$ROOT" 2>/dev/null || exit 2
 
 fail() { echo "  ✗ $1"; echo "     → 怎么办：$2"; exit 1; }
 
-MODES=(direct skip-first-transaction-barrier page-cache second-transaction second-instance)
+MODES=(direct skip-first-transaction-barrier page-cache second-transaction second-instance raise-rollback-floor)
 
 # 冷重开之后应当择到的根。前三个模式停在第一个事务（实例 1、txg 3）；发布 B 走到 txg 4；
-# 第二个实例再走一遍「取号 → 写行 → 暖机 ×2 → 发布 C」，落在实例 2 的 txg 8。
+# 第二个实例再走一遍「取号 → 写行 → 暖机 ×2 → 发布 C」，落在实例 2 的 txg 8；
+# 抬 F 那一档接着发布 D（txg 9）、抬 F 推两次空发布（txg 10、11），落在实例 2 的 txg 11。
 cold_root_of() {
   case "$1" in
     direct|skip-first-transaction-barrier|page-cache) printf '1:3' ;;
     second-transaction)                               printf '1:4' ;;
     second-instance)                                  printf '2:8' ;;
+    raise-rollback-floor)                             printf '2:11' ;;
     *)                                                printf '' ;;
   esac
 }
 
-# 这个模式的输出里必须逐条出现的行，一行一条基本正则。五个模式共有的那几条（段序列、事务计数、
-# 冷重开、块层 FLUSH 数）在判定循环里另查，这里只列各模式独有的——没有这几条，新加的两档就等于没跑。
+# 这个模式的输出里必须逐条出现的行，一行一条基本正则。六个模式共有的那几条（段序列、事务计数、
+# 冷重开、块层 FLUSH 数）在判定循环里另查，这里只列各模式独有的——没有这几条，后加的三档就等于没跑。
 required_lines_of() {
   case "$1" in
-    second-transaction|second-instance)
+    second-transaction|second-instance|raise-rollback-floor)
       cat <<'PATTERNS'
 name=second_transaction root_txg=4 transaction=2 released=
 name=second_transaction .*segments=16+2+1+2 closed_form=
@@ -56,7 +59,7 @@ PATTERNS
     *) : ;;
   esac
   case "$1" in
-    second-instance)
+    second-instance|raise-rollback-floor)
       cat <<'PATTERNS'
 name=writable_mount instance=2 chosen_root=1:4 rows_written=1 row_publish_root=2:5 warm_up_txgs=6,7 nanoseconds=
 name=publish_writes publish=instance_row txg=5 write_calls=
@@ -67,6 +70,23 @@ name=third_transaction root_txg=8 transaction=1 released=
 name=third_transaction .*segments=16+2+1+2 closed_form=
 name=publish_writes publish=third_transaction txg=8 write_calls=
 name=publish_writes_against_device window=third_transaction publishes=1 .*matches=true
+PATTERNS
+      ;;
+    *) : ;;
+  esac
+  case "$1" in
+    raise-rollback-floor)
+      cat <<'PATTERNS'
+name=fourth_transaction root_txg=9 transaction=2 released=
+name=fourth_transaction .*segments=16+2+1+2 closed_form=
+name=publish_writes publish=fourth_transaction txg=9 write_calls=
+name=publish_writes_against_device window=fourth_transaction publishes=1 .*matches=true
+name=raise_rollback_floor floor_before=0 requested_floor=3 ceiling=3 publishes=2 root_txgs=10,11 
+name=recover_cold_rollback_floor chosen_root=2:11 rollback_floor_on_disk=3$
+name=raise_rollback_floor .*segments=8+2+1+10+2+1+2 closed_form=
+name=publish_writes publish=raise_rollback_floor txg=10 write_calls=
+name=publish_writes publish=raise_rollback_floor txg=11 write_calls=
+name=publish_writes_against_device window=raise_rollback_floor publishes=2 .*matches=true
 PATTERNS
       ;;
     *) : ;;
@@ -78,13 +98,26 @@ PATTERNS
 forbidden_lines_of() {
   case "$1" in
     direct|skip-first-transaction-barrier|page-cache)
-      printf '%s\n' 'name=second_transaction ' 'name=writable_mount ' 'name=third_transaction ' ;;
+      printf '%s\n' 'name=second_transaction ' 'name=writable_mount ' 'name=third_transaction ' 'name=fourth_transaction ' 'name=raise_rollback_floor ' ;;
     second-transaction)
-      printf '%s\n' 'name=writable_mount ' 'name=third_transaction ' ;;
+      printf '%s\n' 'name=writable_mount ' 'name=third_transaction ' 'name=fourth_transaction ' 'name=raise_rollback_floor ' ;;
+    second-instance)
+      printf '%s\n' 'name=fourth_transaction ' 'name=raise_rollback_floor ' ;;
     *) : ;;
   esac
 }
 
+# 宿主检查照这个模式重跑的段（它的 `name=host_rerun` 行里 windows= 那一串，段名与 first_transaction_device_log_check.rs
+# 的 ProgramWindow::name 相同）：第一个事务之后那几次发布的写有没有进逐项比对，以这一行为准。只登记第一个事务之后还有发布的三档。
+host_rerun_windows_of() {
+  case "$1" in
+    second-transaction)   printf 'mkfs,instance_acquisition,warm_up,first_transaction,second_transaction' ;;
+    second-instance)      printf 'mkfs,instance_acquisition,warm_up,first_transaction,second_transaction,reopen_and_writable_mount,third_transaction' ;;
+    raise-rollback-floor) printf 'mkfs,instance_acquisition,warm_up,first_transaction,second_transaction,reopen_and_writable_mount,third_transaction,fourth_transaction,raise_rollback_floor' ;;
+    *)                    printf '' ;;
+  esac
+}
+
 prerecorded=0
 [[ -f "$ROOT/.qemu-prerecorded" ]] && prerecorded=1
 
@@ -180,7 +213,7 @@ for mode in "${MODES[@]}"; do
   grep -aq "name=recover_cold outcome=file_read root=$cold_root content_matches=true" "$out" \
     || fail "虚机跑 $mode 冷重开之后没按 ($cold_root) 读回文件" "看 $mode 的 name=recover_cold 行与来宾的 stderr（vm-bench.sh 保留现场用 VM_KEEP=1）；根对不上先查这一档该走到哪一次发布。"
   checks=$((checks + 1))
-  grep -aq "name=transaction policy_mismatches=0 key_order_mismatches=0 root_txg=3 back_chain=$EXPECTED_BACK_CHAIN" "$out" \
+  grep -aq "name=transaction key_order_mismatches=0 root_txg=3 back_chain=$EXPECTED_BACK_CHAIN" "$out" \
     || fail "虚机跑 $mode 的第一个事务计数或反向链与产物不符" "产物 name=root_record 行的 back_chain 是 $EXPECTED_BACK_CHAIN；两个运行时计数要是 0。"
   checks=$((checks + 1))
   while IFS= read -r pattern; do
@@ -209,14 +242,13 @@ for mode in "${MODES[@]}"; do
     minimum="$(sed -n 's/.*minimum_io=\([0-9]*\).*/\1/p' <<<"$geometry")"
     disks=()
     [[ "$mode" == direct ]] && disks=("$work/$mode/disk0.img" "$work/$mode/disk1.img")
-    "$CHECK" "$work/$mode/log0.img" "$work/$mode/log1.img" "$bytes" "$physical" "$minimum" "${disks[@]}" >"$work/$mode/check.txt" 2>&1
+    "$CHECK" "$mode" "$work/$mode/log0.img" "$work/$mode/log1.img" "$bytes" "$physical" "$minimum" "${disks[@]}" >"$work/$mode/check.txt" 2>&1
     echo "$?" >"$work/$mode/check-exit"
   fi
 done
 
 verdict() { cat "$work/$1/check-exit"; }
 judged_device_logs=0
-prefix_only=()
 for mode in "${MODES[@]}"; do
   case "$mode" in
     direct)
@@ -239,28 +271,24 @@ for mode in "${MODES[@]}"; do
              fail "对照 page-cache 没有判红：走页缓存的回写与 O_DIRECT 的逐个写在这道比对里分不出来" "比对失去判别力；看设备侧日志里写的条数与长度。"; }
       checks=$((checks + 1))
       ;;
-    second-transaction|second-instance)
-      # 宿主那一侧只重跑得了第一个事务，所以这两档必然判红。要判的是**红在哪**：
-      # divergence 的 program 侧为 None，说明程序那条录制流整段是设备侧日志的前缀（第一个事务那段逐项相等），
-      # 分歧只出在「盘上还多收到了后面那几次发布的写」。红在别处（program=Some(...)）就是第一个事务那段本身对不上。
-      [[ "$(verdict "$mode")" == 1 ]] \
-        || { sed 's/^/        /' "$work/$mode/check.txt"
-             fail "$mode：设备侧比对没有判红，而宿主只重跑得了第一个事务" "盘上本该多出发布 B（以及第二个实例的挂载与发布 C）的写；判绿说明那些写根本没到盘上，或者比对把多出来的事件吞了。"; }
+    second-transaction|second-instance|raise-rollback-floor)
+      # 宿主检查照模式重跑到这一档最后那次发布，这几段与第一个事务一样逐项比，照 direct 判：退出码 0、两块盘都没有分歧，
+      # 且 `name=host_rerun` 列出的段就是这一档该跑的那几段（少一段，那一段的写就没进比对）。
+      [[ "$(verdict "$mode")" == 0 ]] || { sed 's/^/        /' "$work/$mode/check.txt"
+        fail "$mode：设备侧日志与程序的录制流对不上" "上面 divergence_window= 指着第一处不一致落在哪一段、divergence= 指着下标与两侧各是什么，先判是写路的错还是比对口径的错；退出码 2 是用法错或宿主重跑失败，看 check.txt 末尾那句。"; }
       checks=$((checks + 1))
       for device in 0 1; do
-        device_line="$(grep -a "name=device_log device=$device " "$work/$mode/check.txt" | head -1)"
-        [[ -n "$device_line" ]] || fail "$mode：check.txt 里没有盘 $device 的 name=device_log 行" "看 $work/$mode/check.txt；两块盘各要有一行。"
-        grep -q 'divergence=at=[0-9][0-9]*program=Nonedevice=Some(Write' <<<"$device_line" \
-          || { echo "        $device_line"
-               fail "$mode 盘 $device：第一个事务那段不是设备侧日志的前缀" "divergence 的 program 侧不是 None，说明分歧落在第一个事务里面，不是后面多出来的发布；先按 direct 那一档查写路。"; }
-        expected_writes="$(sed -n 's/.* expected_writes=\([0-9]*\) .*/\1/p' <<<"$device_line")"
-        observed_writes="$(sed -n 's/.* observed_writes=\([0-9]*\) .*/\1/p' <<<"$device_line")"
-        [[ "$expected_writes" =~ ^[0-9]+$ && "$observed_writes" =~ ^[0-9]+$ && "$observed_writes" -gt "$expected_writes" ]] \
-          || { echo "        $device_line"
-               fail "$mode 盘 $device：盘上收到的写（$observed_writes）没有多于第一个事务那段（$expected_writes）" "这一档该在第一个事务之后再发布一次；盘上没多收到写就是那次发布压根没落到这块盘上。"; }
-        checks=$((checks + 2))
+        grep -q "name=device_log device=$device .* divergence_window=none divergence=none" "$work/$mode/check.txt" \
+          || { sed 's/^/        /' "$work/$mode/check.txt"
+               fail "$mode 盘 $device：宿主检查退出 0，却没有这块盘 divergence=none 的 name=device_log 行" "退出码与结果行对不上，先查 crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs 的 main 与 compare_one_device。"; }
+        checks=$((checks + 1))
       done
-      prefix_only+=("$mode")
+      host_rerun_windows="$(host_rerun_windows_of "$mode")"
+      [[ -n "$host_rerun_windows" ]] || fail "模式 $mode 没有登记宿主检查该重跑的段" "在 host_rerun_windows_of 里给它补一行；不补，宿主少重跑一段也看不出来。"
+      grep -q "name=host_rerun mode=$mode windows=$host_rerun_windows operations_by_window=" "$work/$mode/check.txt" \
+        || { sed 's/^/        /' "$work/$mode/check.txt"
+             fail "$mode：宿主检查重跑的段不是 $host_rerun_windows" "看 check.txt 的 name=host_rerun 行；段名单对不上，先查送给 first_transaction_device_log_check 的第一个参数是不是 $mode。"; }
+      checks=$((checks + 1))
       ;;
     *) fail "模式 $mode 没有登记设备侧日志的判据" "在这个 case 里给它补一支；不补就等于这一档的设备侧比对一个字都不判。" ;;
   esac
@@ -268,11 +296,11 @@ for mode in "${MODES[@]}"; do
 done
 
 [[ "$judged_device_logs" == "${#MODES[@]}" ]] || fail "判了 $judged_device_logs 档设备侧日志，而这一轮跑了 ${#MODES[@]} 档" "两个数对不上就有一档被整个跳过；看上面那个 for 循环。"
-uncovered="发布 B / 可写挂载 / 发布 C 的设备侧逐项比对（宿主 first_transaction_device_log_check 只重跑第一个事务）：${prefix_only[*]:-无}；崩溃注入：五档都没有"
+uncovered="崩溃注入：${#MODES[@]} 档都没有"
 if ((prerecorded)); then
   echo "  ✓ 预录档（.qemu-prerecorded，没起虚机）：判了 ${#MODES[@]} 档 ${MODES[*]}、$checks 项检查全过；样本里没摆的 ${#missing[@]} 档：${missing[*]:-无}"
   echo "     → 这一档不算真跑过 QEMU：真跑要在仓顶层不带 .qemu-prerecorded 跑一遍本阶段。"
   exit 3
 fi
-echo "  ✓ QEMU 真设备：${#MODES[@]} 次虚机跑（${MODES[*]}）、$checks 项检查全过；direct 设备侧逐项对得上、两个对照都红在该红的地方，发布 B 与第二个实例两档核到第一个事务那段前缀且盘上确实多收到了写"
+echo "  ✓ QEMU 真设备：${#MODES[@]} 次虚机跑（${MODES[*]}）、$checks 项检查全过；direct、second-transaction、second-instance、raise-rollback-floor 设备侧逐项对得上（宿主照模式重跑到这一档最后那次发布），两个对照都红在该红的地方"
 echo "     ! 这一道没罩到：$uncovered"
diff --git a/.claude/gate.d/74-model-differential.sh b/.claude/gate.d/74-model-differential.sh
index 756ac3c..37183aa 100755
--- a/.claude/gate.d/74-model-differential.sh
+++ b/.claude/gate.d/74-model-differential.sh
@@ -3,12 +3,13 @@
 # gate-covers: 模型对拍
 #
 # 被测的是 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs` 那五段：快档、偏向抬 F 之后复用的取样点、
-# 偏向抬 F 之后回退的取样点、逼近分配记录墙的取样点、小盘上逼近单元区墙的取样点（后两段每一步之后也跑池级 checker，已知红第 0 条那一形只记不停）。每段的报告里有一行「模型对拍 N 步：…」（`crates/singlefs-harness/src/history.rs` 的报告渲染），
+# 偏向抬 F 之后回退的取样点、越过原分配记录墙的取样点、小盘上逼近单元区墙的取样点（后两段每一步之后也跑池级 checker，已知红第 0 条那一形只记不停）。每段的报告里有一行「模型对拍 N 步：…」（`crates/singlefs-harness/src/history.rs` 的报告渲染），
 # 模型模块 `crates/singlefs-harness/src/model.rs` 只用 `singlefs_format` 的常量（D13（验证路线） 已定项 5）。
 # 这里在 release 下单跑那一个测试二进制，要求五段都打出「模型对拍」那一行、步数都大于 0——测试绿而模型一步没判，等于没对拍。
 # 判别力：模型对拍自己会不会红，由 `crates/mutations.tsv` 第 146–178 行（门禁 59 号）证明；这个阶段只判「跑了、判过、没报对不上」。
 # 样本：被判目录里没有 Cargo.toml 而有 `model-differential-cargo-output.log` 时，不跑 cargo，只判那份录好的输出
 # （`.claude/gate.d/fixtures/74-model-differential.sh/` 的红绿样本走这一支）。
+# gate-overlap:copy-kept 59-crates-mutation-replay.sh 两边开头那 8 行（取仓根、问 research/scripts/stage-must-run.sh 能不能复用上一次整轮全绿的判定、能复用就退 77）逐字相同；59 号是提交时才执行的重型阶段，改了它的实现，提交之前没有办法证明没改坏，所以两边各留一份，不抽成共用
 set -uo pipefail
 ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
 cd "$ROOT" 2>/dev/null || exit 2
@@ -35,7 +36,7 @@ if [[ "$scope_rc" != 0 ]]; then
 fi
 
 TEST_BINARY="second_transaction_supplement_three_random_history"
-SECTIONS=("随机历史快档" "随机历史：偏向抬 F 之后复用的取样点" "随机历史：偏向抬 F 之后回退的取样点" "随机历史：逼近分配记录墙的取样点" "随机历史：小盘上逼近单元区墙的取样点")
+SECTIONS=("随机历史快档" "随机历史：偏向抬 F 之后复用的取样点" "随机历史：偏向抬 F 之后回退的取样点" "随机历史：越过原分配记录墙的取样点" "随机历史：小盘上逼近单元区墙的取样点")
 
 log="$(mktemp)"
 if [[ -f Cargo.toml && -d crates/singlefs-harness ]]; then
```
