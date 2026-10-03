# 第 3 批合入的 diff（相对 HEAD fcc4d340；新 crate 与 E165 装置整份在冻结副本里读）

```diff
diff --git a/Cargo.lock b/Cargo.lock
index 3e0da527..4ba020f6 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -175,6 +175,12 @@ dependencies = [
  "unicode-width",
 ]
 
+[[package]]
+name = "convert_case"
+version = "0.4.0"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "6245d59a3e82a7fc217c5828a6692dbc6dfb63a0c8c90495621f7b9d79704a0e"
+
 [[package]]
 name = "crunchy"
 version = "0.2.4"
@@ -807,6 +813,7 @@ name = "singlefs-core"
 version = "0.1.0"
 dependencies = [
  "singlefs-format",
+ "singlefs-replay-prefix",
 ]
 
 [[package]]
@@ -822,6 +829,13 @@ dependencies = [
  "singlefs-format",
 ]
 
+[[package]]
+name = "singlefs-replay-prefix"
+version = "0.1.0"
+dependencies = [
+ "vstd",
+]
+
 [[package]]
 name = "slab"
 version = "0.4.12"
@@ -871,6 +885,17 @@ dependencies = [
  "unicode-ident",
 ]
 
+[[package]]
+name = "synstructure"
+version = "0.13.2"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "728a70f3dbaf5bab7f0c4b1ac8d7ae5ea60a4b5549c8a5914361c99147a709d2"
+dependencies = [
+ "proc-macro2",
+ "quote",
+ "syn 2.0.119",
+]
+
 [[package]]
 name = "thiserror"
 version = "2.0.21"
@@ -918,6 +943,71 @@ version = "0.2.15"
 source = "registry+https://github.com/rust-lang/crates.io-index"
 checksum = "accd4ea62f7bb7a82fe23066fb0957d48ef677f6eeb8215f372f52e48bb32426"
 
+[[package]]
+name = "verus_builtin"
+version = "0.0.0-2026-09-16-0054"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "5e4a9e69d45ddbe303075756d3fc2fcc965eb9dc70c80651d0e4458a6af1ed83"
+
+[[package]]
+name = "verus_builtin_macros"
+version = "0.0.0-2026-09-20-0158"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "213ca3e8a80925aec251d3f1952a213d6090156b58ad81dccc3a2f1fed77c99f"
+dependencies = [
+ "convert_case",
+ "proc-macro2",
+ "quote",
+ "syn 2.0.119",
+ "synstructure",
+ "verus_prettyplease",
+ "verus_syn",
+]
+
+[[package]]
+name = "verus_prettyplease"
+version = "0.0.0-2026-09-06-0133"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "71956c0dcf55f57fd5a5335fff16981f7e0912d2e887914518c5a64fe36c063c"
+dependencies = [
+ "proc-macro2",
+ "verus_syn",
+]
+
+[[package]]
+name = "verus_state_machines_macros"
+version = "0.0.0-2026-09-06-0133"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "5f78ea16629331605a440e9e0a259dd3492a78212186ba82253ad0988127407f"
+dependencies = [
+ "indexmap",
+ "proc-macro2",
+ "quote",
+ "verus_syn",
+]
+
+[[package]]
+name = "verus_syn"
+version = "0.0.0-2026-09-06-0133"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "7d8b53574934e1f608dfccd809ce17f3733d6eb99c14ce8220326121929e1637"
+dependencies = [
+ "proc-macro2",
+ "quote",
+ "unicode-ident",
+]
+
+[[package]]
+name = "vstd"
+version = "0.0.0-2026-09-20-0158"
+source = "registry+https://github.com/rust-lang/crates.io-index"
+checksum = "bb7a36bbc0193492141eef73272853a5a8d73288bed1ea5344563b0382cdf7c9"
+dependencies = [
+ "verus_builtin",
+ "verus_builtin_macros",
+ "verus_state_machines_macros",
+]
+
 [[package]]
 name = "wasm-bindgen"
 version = "0.2.129"
diff --git a/Cargo.toml b/Cargo.toml
index 70558a65..b12ca8e6 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -5,6 +5,7 @@ resolver = "2"
 members = [
     "crates/singlefs-format",
     "crates/singlefs-core",
+    "crates/singlefs-replay-prefix",
     "crates/singlefs-harness",
     "crates/singlefs-checker",
     "crates/singlefs-checker-tier",
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index dc8342f7..34d1bfd2 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -20,8 +20,8 @@ Ignore 那一遍恢复不过 oracle	crates/singlefs-checker-tier/src/crash.rs
 步 2 变异：忘了改写分配记录	crates/singlefs-core/src/allocator.rs	            record.is_released = true;	            record.is_released = false;	-p singlefs-harness --test overwrite_in_one_instance -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
 步 3：不写行（实例表照抄）	crates/singlefs-core/src/mount.rs	        instance_table: InstanceTablePlan::Rewrite(instance_table),	        instance_table: InstanceTablePlan::Carry(previous.root.instance_table),	-p singlefs-harness --test writable_mount_takes_a_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
 步 3：暖机只推一次	crates/singlefs-core/src/mount.rs	        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(warm_up_publishes.len()).expect("次数") < 1	-p singlefs-harness --test writable_mount_takes_a_second_instance -- damaging_every_instance_two_root	damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device
-步 3：前缀跨实例边界（把别的实例的记录也接上）	crates/singlefs-core/src/recovery.rs	            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water	            (record.instance, record.checkpoint_txg) > water	-p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- layer0_quick_tier_matches	layer0_quick_tier_matches_the_full_tally_shape
-步 3：链首锚点错一位（接在所选根自己那条记录之后第二条）	crates/singlefs-core/src/recovery.rs	        root_own_record_counter.map(|counter| (root.instance, counter + 1));	        root_own_record_counter.map(|counter| (root.instance, counter + 2));	-p singlefs-harness --test writable_mount_takes_a_second_instance -- stray_record_of_the_previous_instance	stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row
+步 3：前缀跨实例边界（把别的实例的记录也接上）	crates/singlefs-replay-prefix/src/lib.rs	    record.instance == root_instance && (record.instance > root_instance || (record.instance	    (record.instance > root_instance || (record.instance	-p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- layer0_quick_tier_matches	layer0_quick_tier_matches_the_full_tally_shape
+步 3：链首锚点错一位（接在所选根自己那条记录之后第二条）	crates/singlefs-replay-prefix/src/lib.rs	            { (root_instance, counter + 1) },	            { (root_instance, counter + 2) },	-p singlefs-harness --test writable_mount_takes_a_second_instance -- stray_record_of_the_previous_instance	stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row
 步 3：I-3.8 判定恒真	crates/singlefs-checker/src/walk.rs	            unique && below_mount_root && chain_record_last,	            unique || below_mount_root || chain_record_last || true,	-p singlefs-harness --test writable_mount_takes_a_second_instance -- checker_rejects_an_instance_table_row	checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root
 步 3：重建分配器时不把已释放的放进 defer 队列	crates/singlefs-core/src/allocator.rs	            if record.is_released {\n                device_map.mark_released(record.slot, span);\n            }	            if record.is_released && false {\n                device_map.mark_released(record.slot, span);\n            }	-p singlefs-harness --test writable_mount_takes_a_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
 步 4：影子账一个槽都不隔离	crates/singlefs-core/src/mount.rs	            allocator.isolate_abandoned(device, slot, span_slots);	            let _ = (device, slot, span_slots);	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored with_the_shadow_ledger_a_publish_after_a_recovery	with_the_shadow_ledger_a_publish_after_a_recovery_keeps_off_the_abandoned_data_slot_and_the_abandoned_root_reads_the_third_content
@@ -33,7 +33,7 @@ Ignore 那一遍恢复不过 oracle	crates/singlefs-checker-tier/src/crash.rs
 步 5：复用时追加记录而不改写（同盘同槽两条）	crates/singlefs-core/src/allocator.rs	            existing.span_slots = u16::try_from(placement.span).expect("跨度 2 字节");\n            existing.generation = generation;\n            existing.is_released = false;	            records.push(AllocationRecord {\n                device,\n                slot: placement.slot,\n                span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),\n                generation,\n                is_released: false,\n            });	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
 步 5：checker 的候选集不按 F 收（F 之下的根照走）	crates/singlefs-checker/src/walk.rs	            let walked = *index == newest_index || (!abandoned && !below_floor);	            let walked = *index == newest_index || !abandoned;	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
 步 5（2026-09-26 取 SysPre 之后改写这一行）：F_生效 退回「各盘根上 F 最大值的最小值」、不读系统配置	crates/singlefs-core/src/recovery.rs	    highest_on_the_roots\n        .max(highest_in_the_system_configurations)\n        .unwrap_or(CheckpointTxg(0))	    {\n        let mut highest_per_device: BTreeMap<DeviceIdentity, CheckpointTxg> = BTreeMap::new();\n        for root in readable_roots(reader, region_devices, immutable_sizes, filesystem_identifier) {\n            let device = region_devices[usize::try_from(crate::root_ring::target_for_publish(root.checkpoint_txg, immutable_sizes.root_ring_slots_per_region).region).expect("区域号")];\n            let highest = highest_per_device.entry(device).or_insert(root.rollback_floor);\n            *highest = (*highest).max(root.rollback_floor);\n        }\n        highest_per_device.values().copied().min().unwrap_or(CheckpointTxg(0))\n    }	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored floor_carried_by_the_system_configuration_takes_effect	floor_carried_by_the_system_configuration_takes_effect_on_remount_even_when_one_device_lost_its_root_carrying_it
-步 3 三方第一轮：链首没锚点时不看 txg、无条件接上水位之上最小的一条	crates/singlefs-core/src/recovery.rs	        } else if record.checkpoint_txg != chain_start_txg_without_anchor\n	        } else if false && record.checkpoint_txg != chain_start_txg_without_anchor\n	-p singlefs-harness --test writable_mount_takes_a_second_instance -- torn_anchor_record	torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg_when_the_next_record_is_torn_too
+步 3 三方第一轮：链首没锚点时不看 txg、无条件接上水位之上最小的一条	crates/singlefs-replay-prefix/src/lib.rs	                if record.checkpoint_txg != chain_start_txg_without_anchor\n	                if false && record.checkpoint_txg != chain_start_txg_without_anchor\n	-p singlefs-harness --test writable_mount_takes_a_second_instance -- torn_anchor_record	torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg_when_the_next_record_is_torn_too
 步 4：普通重开不隔离被抛弃根引用的槽（影子账只在回退那一次算）	crates/singlefs-core/src/mount.rs	    mount_writable_with_test_only_switches(parameters, devices, space_admission, ShadowLedger::On)	    mount_writable_with_test_only_switches(parameters, devices, space_admission, ShadowLedger::Off)	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored the_plain_remount_after_a_recovery	the_plain_remount_after_a_recovery_that_abandoned_roots_keeps_the_isolation
 步 4：回退候选集的 F 用最新根自己带的 F 而不是 F_生效	crates/singlefs-core/src/mount.rs	    if target.checkpoint_txg < effective_floor {	    if target.checkpoint_txg < newest_root.rollback_floor {	-p singlefs-checker-tier --test rollback_floor_written_into_the_system_configuration_first_and_normal_unmount -- rolling_back_below_a_floor	rolling_back_below_a_floor_only_the_system_configuration_carries_is_refused_before_any_write
 步 5：回收门槛不看环里最旧有效根	crates/singlefs-core/src/mount.rs	    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))	    oldest_valid_root.map_or(effective_floor, |_oldest| effective_floor)	-p singlefs-core reclaim_floor_takes	reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root
@@ -223,7 +223,7 @@ E156 referenced_slots 丢掉实例表的兜底加回	crates/singlefs-checker-tie
 并行线二：读路径自报的设备级读数不加（实现说这次一个设备读都没发，装置在块层照样数到那几次，D17 已定项 5 射程 ① 的对账等于没有）	crates/singlefs-core/src/mounted_read.rs	        observation.device_reads_issued += 1;	        observation.device_reads_issued += 0;	-p singlefs-harness --test mounted_read_by_offset -- random_four_kibibyte_reads	random_four_kibibyte_reads_return_the_written_bytes_and_never_scan_the_journal_ring
 并行线二：这次读了几个单元的计数不加（验收第 4 条那个观测点恒 0，跨单元的页也不报 2）	crates/singlefs-core/src/mounted_read.rs	            observation.data_units_read += 1;	            observation.data_units_read += 0;	-p singlefs-harness --test mounted_read_by_offset -- every_aligned_page	every_aligned_page_reads_back_and_the_pages_that_cross_a_unit_boundary_read_two_units
 并行线二：一次读只解引用第一个单元（跨单元的页少拷后半页，D4 已定项 5 那一成多的页整档漏掉）	crates/singlefs-core/src/mounted_read.rs	        for unit_number in span.first.0..=span.last_inclusive.0 {	        for unit_number in span.first.0..=span.first.0 {	-p singlefs-harness --test mounted_read_by_offset -- corrupting_one_byte	corrupting_one_byte_in_a_data_unit_makes_reads_over_it_report_a_checksum_error_and_leaves_the_other_reads_alone
-只供测试的开关：关掉点名验证时 verification_passed 照加（两臂在健康镜像上报同一个数，运行时看不出走了哪一条）	crates/singlefs-core/src/recovery.rs	        if verify_named_units {\n            report.verification_passed += 1;\n        }	        report.verification_passed += 1;	-p singlefs-harness --test recovery_reads_the_created_file_after_reopen	the_named_unit_verification_switch_shows_up_in_the_runtime_counter
+只供测试的开关：关掉点名验证时 verification_passed 照加（两臂在健康镜像上报同一个数，运行时看不出走了哪一条）	crates/singlefs-replay-prefix/src/lib.rs	        if verify_named_units {\n            report.verification_passed += 1;\n        }	        report.verification_passed += 1;	-p singlefs-harness --test recovery_reads_the_created_file_after_reopen	the_named_unit_verification_switch_shows_up_in_the_runtime_counter
 I-1.10：条目宽与字段表宽的比对改成恒真（走读照样停下，只是不再判红）	crates/singlefs-checker/src/walk.rs	.judge("I-1.10", entry_width == entry_field_table_bytes, || {	.judge("I-1.10", true, || {	-p singlefs-harness --test checker_known_bad_images -- an_entry_width	an_entry_width_that_is_not_the_field_table_width_reddens_only_the_entry_width_invariant
 I-1.10：判红之后不停下，照样按字段表的固定偏移解那条窄条目（checker 自己被坏镜像打死）	crates/singlefs-checker/src/walk.rs	            if entry_width != entry_field_table_bytes {\n                return;\n            }	            if false && entry_width != entry_field_table_bytes {\n                return;\n            }	-p singlefs-harness --test checker_known_bad_images -- an_entry_width	an_entry_width_that_is_not_the_field_table_width_reddens_only_the_entry_width_invariant
 extent 叶记录的读者不判条目宽（按 112 的固定偏移切 24 字节的条目）	crates/singlefs-core/src/records.rs	    if bytes.len() < usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112") {	    if false && bytes.len() < usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112") {	-p singlefs-core --lib -- entry_readers_refuse	entry_readers_refuse_an_entry_narrower_than_its_field_table
@@ -351,8 +351,8 @@ E156 snapshot_allocated_generations 认反方向（只收「已释放」的记
 增补 2 第 21 行（C366，挂载层）：写行那次发布的 jsn 取 txg（所选根自己那条记录读不出时写行 txg 5、jsn 应是 4）	crates/singlefs-core/src/mount.rs	                RowPublishIdentity {\n                    txg: start.first_txg,\n                    counter: start.next_counter,	                RowPublishIdentity {\n                    txg: start.first_txg,\n                    counter: start.first_txg.0,	-p singlefs-harness --test warm_up_journal_counter -- a_writable_mount_after_a_two_record_publish_numbers_its_records_and_tail_from_the_jsn_one_ahead_of_the_txg	a_writable_mount_after_a_two_record_publish_numbers_its_records_and_tail_from_the_jsn_one_ahead_of_the_txg
 增补 2 第 21 行（C366，挂载层）：带文件的一版上暖机的 jsn 取上一版的 txg + 1	crates/singlefs-core/src/mount.rs	        counter: current_file_version.record.counter + 1,	        counter: current_file_version.root.checkpoint_txg.0 + 1,	-p singlefs-harness --test warm_up_journal_counter -- a_writable_mount_after_a_two_record_publish_numbers_its_records_and_tail_from_the_jsn_one_ahead_of_the_txg	a_writable_mount_after_a_two_record_publish_numbers_its_records_and_tail_from_the_jsn_one_ahead_of_the_txg
 增补 2 第 21 行（C366，挂载层）：发布轮换系统配置槽时 tail 写 txg、不写 jsn	crates/singlefs-core/src/transaction.rs	    writer.perform_commit_step(CommitStep::RotateSystemConfigurationSlots {\n        journal_tail,\n        journal_instance,\n    })	    writer.perform_commit_step(CommitStep::RotateSystemConfigurationSlots {\n        journal_tail: checkpoint_txg.0,\n        journal_instance,\n    })	-p singlefs-harness --test warm_up_journal_counter -- warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two	warm_up_after_a_journal_counter_other_than_zero_counts_records_on_from_it_while_txg_stays_one_and_two
-增补 2 第 61 行（C519 已知丢失）：施加前的点名单元验证改成任一条位置条目验过即过（改法 Z2 被悄悄落地时已知丢失那条先红）	crates/singlefs-core/src/recovery.rs	                named.locations.iter().all(|location| {	                named.locations.iter().any(|location| {	-p singlefs-harness --test whole_device_loss_after_warm_up -- c519_known_loss	c519_known_loss_after_the_warm_up_covers_both_devices_losing_the_acknowledged_version_root_device_falls_back_to_the_row_publish_version
-增补 2 第 62 行（C533 复现）：前缀跨实例边界，写行那条孤记录进了水位之上	crates/singlefs-core/src/recovery.rs	            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water	            (record.instance, record.checkpoint_txg) > water	-p singlefs-harness --test row_publish_record_without_its_root -- c533_row_publish_record_persisted	c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_are_free_slots_that_the_next_mount_overwrites
+增补 2 第 61 行（C519 已知丢失）：施加前的点名单元验证改成任一条位置条目验过即过（改法 Z2 被悄悄落地时已知丢失那条先红）	crates/singlefs-core/src/recovery.rs	            named.locations.iter().all(|location| {	            named.locations.iter().any(|location| {	-p singlefs-harness --test whole_device_loss_after_warm_up -- c519_known_loss	c519_known_loss_after_the_warm_up_covers_both_devices_losing_the_acknowledged_version_root_device_falls_back_to_the_row_publish_version
+增补 2 第 62 行（C533 复现）：前缀跨实例边界，写行那条孤记录进了水位之上	crates/singlefs-replay-prefix/src/lib.rs	    record.instance == root_instance && (record.instance > root_instance || (record.instance	    (record.instance > root_instance || (record.instance	-p singlefs-harness --test row_publish_record_without_its_root -- c533_row_publish_record_persisted	c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_are_free_slots_that_the_next_mount_overwrites
 增补 2 第 9 行 P1（C497）：树表 0 条的一版上写行时分配记录树节点先于实例表取落点	crates/singlefs-core/src/transaction.rs	        let rewritten_roles: Vec<TransactionUnit> = instance_table_roles\n            .iter()\n            .copied()\n            .chain(\n                allocation_plan\n                    .rewritten_nodes()\n                    .into_iter()\n                    .map(role_of_allocation_record_tree_node),\n            )\n            .collect();	        let rewritten_roles: Vec<TransactionUnit> = allocation_plan\n            .rewritten_nodes()\n            .into_iter()\n            .map(role_of_allocation_record_tree_node)\n            .chain(instance_table_roles.iter().copied())\n            .collect();	-p singlefs-harness --test presumed_clause_checks -- c497_every_publish	c497_every_publish_that_rewrites_the_instance_table_bumps_it_before_every_other_commit_generated_block
 增补 2 第 9 行 P3（C498）：暖机不看本实例的根覆盖没覆盖两块盘、恒推 R 次	crates/singlefs-core/src/mount.rs	    while all_devices\n        .iter()\n        .any(|identity| !covered.contains(identity))\n        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	    while u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	-p singlefs-harness --test presumed_clause_checks -- c498_warm_up_publishes	c498_warm_up_publishes_after_remounts_rollback_and_a_crash_in_the_middle_of_a_mount_are_counted_by_the_clause
 增补 2 第 9 行 P5（C499）：可写挂载认「上一次干净关闭」（tail 那条就是所选根那次、下一格没有记录）就只取 tail 那一条、不扫全环	crates/singlefs-core/src/mount.rs	    let records = scan_journal(cache, system_configuration);\n    let (journal, effective_root) = replay_journal(	    let tail_record = |counter: u64| {\n        crate::recovery::PoolReader::read(\n            cache,\n            PoolReader::device_identities(cache)[0],\n            crate::journal::record_offset(\n                counter,\n                system_configuration.immutable.sizes.journal_ring_bytes,\n            ),\n            4096,\n        )\n        .and_then(|bytes| {\n            crate::journal::JournalRecord::parse(\n                &bytes,\n                crate::unit::unit_filesystem_identifier(\n                    &system_configuration.immutable.filesystem_identifier,\n                ),\n            )\n        })\n    };\n    let tail = system_configuration.quantities.journal_tail;\n    let closed_cleanly = tail > 0\n        && tail_record(tail).is_some_and(|record| {\n            record.instance == chosen_root.instance\n                && record.checkpoint_txg == chosen_root.checkpoint_txg\n        })\n        && tail_record(tail + 1).is_none();\n    let records: std::collections::BTreeMap<_, _> = if closed_cleanly {\n        tail_record(tail)\n            .map(|record| ((record.instance, record.counter), record))\n            .into_iter()\n            .collect()\n    } else {\n        scan_journal(cache, system_configuration)\n    };\n    let (journal, effective_root) = replay_journal(	-p singlefs-harness --test presumed_clause_checks -- c499_remount_after	c499_remount_after_a_clean_close_and_after_a_crash_both_scan_the_whole_ring_and_recover_by_the_same_rules
@@ -403,7 +403,7 @@ C513（复用豁免不判那次复用合不合法）：回收谓词那一判拿
 C513：回收谓词判得过严（释放代下界取无穷大，抬 F 之后的合法复用也不开脱，增补 2 收口表第 55 行那 8 个误报回来）	crates/singlefs-checker-tier/src/crash.rs	    let release_generation_at_least = earlier_publish_txg.0.saturating_add(1);	    let release_generation_at_least = u64::MAX;	-p singlefs-checker-tier --test crash_enumeration_fixed_script_stream -- stale_tail_with_a_reused_named_unit	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
 并行线一：C490：extent 叶记录 key 的 offset 段写成单元序号（并行线一验收第 4 条第二个变异：读回错位判红）	crates/singlefs-core/src/transaction.rs	        let extent_record = build_extent_record(\n            FIRST_INODE_NUMBER,\n            transaction.payload_start.0,	        let extent_record = build_extent_record(\n            FIRST_INODE_NUMBER,\n            transaction.unit_index_in_file.0,	-p singlefs-harness --test sequential_write_split_into_one_unit_transactions -- the_second_extent_leaf_record_key_is_the_file_byte_offset_of_the_second_data_unit	the_second_extent_leaf_record_key_is_the_file_byte_offset_of_the_second_data_unit
 并行线一：C491：共享的提交内生块挪到一次发布的第一条记录里点名（末条只点名自己那个数据单元）	crates/singlefs-core/src/transaction.rs	    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles_named_before_the_last_record\n            .iter()\n            .map(|data_role| vec![*data_role])\n            .collect();\n    roles_named_by_each_record.push(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !data_roles_named_before_the_last_record.contains(identity))\n            .collect(),\n    );	    let _ = data_roles_named_before_the_last_record;\n    if data_roles.len() < 2 {\n        return vec![rewritten.to_vec()];\n    }\n    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles.iter().map(|data_role| vec![*data_role]).collect();\n    roles_named_by_each_record[0].extend(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !matches!(identity, TransactionUnit::Data(_))),\n    );	-p singlefs-harness --test sequential_write_split_into_one_unit_transactions -- publish_of_two_data_units_names_the_shared_commit_generated_units_only_in_its_last_record	publish_of_two_data_units_names_the_shared_commit_generated_units_only_in_its_last_record
-并行线一：C376：每条记录都当成它那次发布的末条（发布边界不认，前缀停在发布中间也施加）	crates/singlefs-core/src/recovery.rs	        if !record_ends_its_publish(record) {	        if false {	-p singlefs-harness --test multi_unit_file_write_and_read -- crash_before_the_second_record_of_the_two_record_publish_keeps_the_old_file	crash_before_the_second_record_of_the_two_record_publish_keeps_the_old_file
+并行线一：C376：每条记录都当成它那次发布的末条（发布边界不认，前缀停在发布中间也施加）	crates/singlefs-replay-prefix/src/lib.rs	        if !record_ends_its_publish(record) {	        if false {	-p singlefs-harness --test multi_unit_file_write_and_read -- crash_before_the_second_record_of_the_two_record_publish_keeps_the_old_file	crash_before_the_second_record_of_the_two_record_publish_keeps_the_old_file
 K2：extent 树下段根的层级取「罩的单元数大于单元数」的最低一层（144 个单元装满一片叶时白长一层）	crates/singlefs-core/src/extent_tree.rs	        .find(|level| lower_span_in_data_units(*level) >= data_units)	        .find(|level| lower_span_in_data_units(*level) > data_units)	-p singlefs-harness --test multi_unit_file_write_and_read -- the_lower_extent_segment	the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline
 并行线一：文件变短时上一版多出来的数据单元不释放	crates/singlefs-core/src/transaction.rs	        .chain(previous_data_units_without_a_successor)	        .chain(Vec::<TransactionUnit>::new())	-p singlefs-harness --test multi_unit_file_write_and_read -- shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has	shrinking_a_multi_unit_file_releases_the_data_units_it_no_longer_has
 并行线一：从盘上重建上一版只取第一个文件的第一个数据指针（多单元文件重开之后只剩第一个数据单元）	crates/singlefs-core/src/recovery.rs	    let data_pointers_with_units = extents.data_pointers();	    let data_pointers_with_units: Vec<(u64, DataPointer)> =\n        extents.data_pointers().into_iter().take(1).collect();	-p singlefs-harness --test multi_unit_file_write_and_read -- reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping	reopened_writable_mount_carries_every_data_unit_and_the_next_write_releases_them_through_the_mapping
@@ -411,7 +411,7 @@ K2：extent 树下段根的层级取「罩的单元数大于单元数」的最
 并行线一：记录读者不判点名项区越没越过 4096（并行线一验收第 4 条第三个变异：点名项超过 67 仍塞进一条记录 ⇒ 记录解析拒收）	crates/singlefs-core/src/journal.rs	        if payload_end > record_bytes\n            || crc32_castagnoli	        if crc32_castagnoli	-p singlefs-core --lib -- journal::tests::record_whose_header_claims_more_named_units_than_one_record_holds_is_refused_by_the_parser	record_whose_header_claims_more_named_units_than_one_record_holds_is_refused_by_the_parser
 并行线一：一个数据单元的发布也切成两条记录（末条只点名共享内生块）	crates/singlefs-core/src/transaction.rs	    let data_roles_named_before_the_last_record = &data_roles[..data_roles.len().saturating_sub(1)];	    let data_roles_named_before_the_last_record = &data_roles[..];	-p singlefs-harness --test sequential_write_split_into_one_unit_transactions -- sequential_write_that_fits_one_data_unit_publishes_through_the_single_transaction_path	sequential_write_that_fits_one_data_unit_publishes_through_the_single_transaction_path
 并行线一：层 0：共享内生块挪到第一条记录里点名（里程碑并行线一验收第 4 条第一个变异「提交标记提前到第一条记录」在一事务一记录下的形态：半次发布施加上去）	crates/singlefs-core/src/transaction.rs	    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles_named_before_the_last_record\n            .iter()\n            .map(|data_role| vec![*data_role])\n            .collect();\n    roles_named_by_each_record.push(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !data_roles_named_before_the_last_record.contains(identity))\n            .collect(),\n    );	    let _ = data_roles_named_before_the_last_record;\n    if data_roles.len() < 2 {\n        return vec![rewritten.to_vec()];\n    }\n    let mut roles_named_by_each_record: Vec<Vec<TransactionUnit>> =\n        data_roles.iter().map(|data_role| vec![*data_role]).collect();\n    roles_named_by_each_record[0].extend(\n        rewritten\n            .iter()\n            .copied()\n            .filter(|identity| !matches!(identity, TransactionUnit::Data(_))),\n    );	-p singlefs-checker-tier --test crash_enumeration_multi_record_publish_stream -- every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it	every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it
-并行线一：层 0：发布边界不认（每条记录都当末条）	crates/singlefs-core/src/recovery.rs	        if !record_ends_its_publish(record) {	        if false {	-p singlefs-checker-tier --test crash_enumeration_multi_record_publish_stream -- every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it	every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it
+并行线一：层 0：发布边界不认（每条记录都当末条）	crates/singlefs-replay-prefix/src/lib.rs	        if !record_ends_its_publish(record) {	        if false {	-p singlefs-checker-tier --test crash_enumeration_multi_record_publish_stream -- every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it	every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it
 步 1 验收第 4 条 × K2：extent 上段叶条目内联的数据指针忘了换（覆盖写之后仍指上一版的第一个数据单元，读回等于旧内容）	crates/singlefs-core/src/transaction.rs	                    None => ExtentUpperLeafTarget::InlineDataUnit(\n                        *data_pointers\n                            .first()	                    None => ExtentUpperLeafTarget::InlineDataUnit(\n                        *previous\n                            .map_or(data_pointers, |previous_version| {\n                                previous_version.data_pointers.as_slice()\n                            })\n                            .first()	-p singlefs-harness --test overwrite_in_one_instance -- cold_start_reads_the_second_content_and_the_pool_checker_stays_green	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
 并行线一：数据单元头里的锚点偏移写成单元序号（与 extent key 的 offset 段不符：冷启动顺序读在第 1 个单元当场拒，I-1.1）	crates/singlefs-core/src/transaction.rs	            anchor_offset: transaction.payload_start.0,	            anchor_offset: transaction.unit_index_in_file.0,	-p singlefs-harness --test multi_unit_file_write_and_read -- units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte	units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte
 并行线一：数据单元的净荷字节倒序写（长度、锚点、写序都对，只有内容错：冷启动顺序读回与写入逐字节比对判红）	crates/singlefs-core/src/transaction.rs	            transaction.payload_of(file.content),	            &transaction.payload_of(file.content).iter().rev().copied().collect::<Vec<u8>>(),	-p singlefs-harness --test multi_unit_file_write_and_read -- units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte	units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte
@@ -445,7 +445,7 @@ E158 root_choice_repair Q1 的 Φ1 故障数表：两份都读失败改记成 1
 E158 root_choice_repair Q2-1 的 candidate_fault_sets_of_weight 权重公式：一条记录按 1 个单位算（该按 2 份自证算）	crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs	let weight_one_count = weight - record_count * 2;	let weight_one_count = weight - record_count;	-p singlefs-checker-tier --bin e158_root_choice_repair	candidate_fault_sets_of_weight_mixes_root_slots_and_journal_records_by_weight
 E158 root_choice_repair Q2-1 的 judge_recovery_outcome「倒挂」判定取反（该在根不在时间线上时判倒挂，改成在时间线上时判）	crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs	if !timeline.contains(&chosen) {	if timeline.contains(&chosen) {	-p singlefs-checker-tier --bin e158_root_choice_repair	judge_recovery_outcome_passes_when_root_on_timeline_and_content_matches
 E158 root_choice_repair Q2-1 的 candidate_fault_sets_of_weight 起点错位（记录计数从 1 起，权重 0 时枚不出空集合）	crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs	let mut record_count = 0u64;	let mut record_count = 1u64;	-p singlefs-checker-tier --bin e158_root_choice_repair	candidate_fault_sets_of_weight_at_zero_is_only_the_empty_set
-P6 后一半：锚点读不出时不看本次发布内序号，链首接在下一次发布的第二条上（三方第一轮 K4-b）	crates/singlefs-core/src/recovery.rs	            || record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST\n	            || false\n	-p singlefs-harness --test multi_unit_file_write_and_read -- without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish	without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish
+P6 后一半：锚点读不出时不看本次发布内序号，链首接在下一次发布的第二条上（三方第一轮 K4-b）	crates/singlefs-replay-prefix/src/lib.rs	                    || record.ordinal_within_publish != 1 {\n                    break ;\n                }\n            },\n        }\n        // [原文 3119–3126] 反向链（已定项 7、I-8.6）：Holds | PreviousRecordNotOnDisk 接着走，Broken 断。[改写] 三态由回调交成布尔\n        if back_chain_broken(index) {\n            break ;\n        }\n        // [原文 3127–3134] 一次发布的第一条序号是 1（已定项 4 读者规则；C539）。\n        if open.is_empty() && record.ordinal_within_publish != 1 {	                    || false {\n                    break ;\n                }\n            },\n        }\n        // [原文 3119–3126] 反向链（已定项 7、I-8.6）：Holds | PreviousRecordNotOnDisk 接着走，Broken 断。[改写] 三态由回调交成布尔\n        if back_chain_broken(index) {\n            break ;\n        }\n        // [原文 3127–3134] 一次发布的第一条序号是 1（已定项 4 读者规则；C539）。\n        if open.is_empty() && record.ordinal_within_publish != 1 && expected_next.is_some() {	-p singlefs-harness --test multi_unit_file_write_and_read -- without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish	without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish
 P6 后一半：多条记录的发布每条都写本次发布内序号 1	crates/singlefs-core/src/transaction.rs	            // 恢复只从序号 1 那条接链首（已定项 14 注 1）。\n            ordinal_within_publish: JournalRecordOrdinalWithinPublish::of_record_at_offset(\n                record_offset_in_this_publish,\n            ),\n	            // 恢复只从序号 1 那条接链首（已定项 14 注 1）。\n            ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,\n	-p singlefs-harness --test multi_unit_file_write_and_read -- without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish	without_an_anchor_the_chain_head_must_be_the_first_record_of_the_next_publish
 P6 后一半：本次发布内序号从 0 起	crates/singlefs-core/src/journal.rs	        Self(u32::try_from(record_offset_in_this_publish + 1).expect(	        Self(u32::try_from(record_offset_in_this_publish).expect(	-p singlefs-core --lib -- journal::tests::the_ordinal_of_the_record_at_each_offset_of_the_publish_is_that_offset_plus_one	journal::tests::the_ordinal_of_the_record_at_each_offset_of_the_publish_is_that_offset_plus_one
 P6 后一半：记录头写者不写本次发布内序号（恒写 1）	crates/singlefs-core/src/journal.rs	        writer.put_u32(self.ordinal_within_publish.0);	        writer.put_u32(1);	-p singlefs-core --lib -- journal::tests::the_ordinal_within_publish_sits_right_after_the_commit_marker_and_round_trips	journal::tests::the_ordinal_within_publish_sits_right_after_the_commit_marker_and_round_trips
@@ -509,12 +509,12 @@ E158 root_choice_repair Φ2 系统配置槽这一类证据：newest_self_certifi
 实二十：一个角色都不点名的发布一条记录都不切出来	crates/singlefs-core/src/transaction.rs	    if roles_named_by_the_last_transaction.is_empty() {	    if false {	-p singlefs-core --lib -- transaction::tests::the_last_transaction_spills_over_as_many_records_as_its_named_units_need	transaction::tests::the_last_transaction_spills_over_as_many_records_as_its_named_units_need_and_they_all_share_its_transaction
 实二十：空发布记录不带末条标志	crates/singlefs-core/src/transaction.rs	        // 含空发布记录，那一条记录标志位 0 也写 1）。\n        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,	        // 含空发布记录，那一条记录标志位 0 也写 1）。\n        place_in_publish: JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,	-p singlefs-harness --test new_pool_file_creation_publish -- root_slots_system_configurations_and_journal_ring_hold_the_published_state	root_slots_system_configurations_and_journal_ring_hold_the_published_state
 实二十：树表 0 条那一版上写行那次发布的记录不带末条标志	crates/singlefs-core/src/transaction.rs	            place_in_publish: if transaction_of_the_next_record.is_none() {\n                JournalRecordPlaceInPublish::LastRecordOfThePublish\n            } else {\n	            place_in_publish: if transaction_of_the_next_record.is_none() {\n                JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow\n            } else {\n	-p singlefs-harness --test writable_mount_of_a_formatted_pool -- writable_mount_after_a_crash_right_after_acquiring_an_instance	writable_mount_after_a_crash_right_after_acquiring_an_instance_writes_a_row_for_the_burnt_instance
-实二十：锚点取读法甲——所选根那次发布读得出的记录里 jsn 最大那条，不看末条标志（替换第 438 行那条锚点已腐化的变异）	crates/singlefs-core/src/recovery.rs	                && record_ends_its_publish(record)\n        })\n        .map(|record| record.counter)\n        .collect();	        })\n        .map(|record| record.counter)\n        .max()\n        .into_iter()\n        .collect();	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- when_the_last_record_of_the_chosen_roots_publish_is_unreadable	when_the_last_record_of_the_chosen_roots_publish_is_unreadable_its_readable_earlier_record_is_no_anchor_and_the_next_publish_starts_the_chain
-实二十：所选根那次发布带末条标志的多于一条时不停下（条款没写那一格照常接链）	crates/singlefs-core/src/recovery.rs	    if counters_carrying_the_last_record_flag.len() > 1 {	    if false {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- two_last_record_flags_in_the_chosen_roots_publish	two_last_record_flags_in_the_chosen_roots_publish_stop_recovery_and_a_writable_mount_before_any_write
-实二十：一次发布之内跳号不断链	crates/singlefs-core/src/recovery.rs	            if u64::from(record.ordinal_within_publish.0)\n                != u64::from(previous_record_of_the_open_publish.ordinal_within_publish.0) + 1\n            {	            if false {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- an_ordinal_that_skips_within_a_publish	an_ordinal_that_skips_within_a_publish_breaks_the_chain_and_the_publish_is_not_applied
-实二十：一个事务的提交标记没出现、下一条换了事务号也照接	crates/singlefs-core/src/recovery.rs	            if !previous_record_of_the_open_publish.is_commit\n                && record.transaction != previous_record_of_the_open_publish.transaction\n            {	            if false {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- transaction_whose_commit_marker_never_appears	transaction_whose_commit_marker_never_appears_before_the_next_transaction_keeps_its_publish_unapplied
-实二十：末条不带提交标记也整次施加	crates/singlefs-core/src/recovery.rs	        if !record.is_commit && record_ends_its_publish(record) {	        if false {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- last_record_of_the_publish_without_the_commit_marker	last_record_of_the_publish_without_the_commit_marker_keeps_its_publish_unapplied
-实二十：层 0 L8：恢复在不带提交标记的记录上就停（一事务一条时的旧写法，跨两条记录的事务永远施加不上）	crates/singlefs-core/src/recovery.rs	        if !record.is_commit && record_ends_its_publish(record) {	        if !record.is_commit {	-p singlefs-checker-tier --test crash_enumeration_record_spill_over_stream -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
+实二十：锚点取读法甲——所选根那次发布读得出的记录里 jsn 最大那条，不看末条标志（替换第 438 行那条锚点已腐化的变异）	crates/singlefs-replay-prefix/src/lib.rs	root_checkpoint_txg\n            && record_ends_its_publish(record) {\n            // [原文 2976–2984]\n            if found.is_some() {\n                return Err(CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlag);\n            }\n            found = Some(record.counter);	root_checkpoint_txg {\n            found = Some(record.counter);	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- when_the_last_record_of_the_chosen_roots_publish_is_unreadable	when_the_last_record_of_the_chosen_roots_publish_is_unreadable_its_readable_earlier_record_is_no_anchor_and_the_next_publish_starts_the_chain
+实二十：所选根那次发布带末条标志的多于一条时不停下（条款没写那一格照常接链）	crates/singlefs-replay-prefix/src/lib.rs	            if found.is_some() {\n                return Err(CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlag);\n            }\n            found = Some(record.counter);	            if found.is_none() {\n                found = Some(record.counter);\n            }	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- two_last_record_flags_in_the_chosen_roots_publish	two_last_record_flags_in_the_chosen_roots_publish_stop_recovery_and_a_writable_mount_before_any_write
+实二十：一次发布之内跳号不断链	crates/singlefs-replay-prefix/src/lib.rs	            if record.ordinal_within_publish as u64\n                != previous_record_of_the_open_publish.ordinal_within_publish as u64 + 1 {	            if false {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- an_ordinal_that_skips_within_a_publish	an_ordinal_that_skips_within_a_publish_breaks_the_chain_and_the_publish_is_not_applied
+实二十：一个事务的提交标记没出现、下一条换了事务号也照接	crates/singlefs-replay-prefix/src/lib.rs	            if !previous_record_of_the_open_publish.is_commit && record.transaction\n                != previous_record_of_the_open_publish.transaction {	            if false {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- transaction_whose_commit_marker_never_appears	transaction_whose_commit_marker_never_appears_before_the_next_transaction_keeps_its_publish_unapplied
+实二十：末条不带提交标记也整次施加	crates/singlefs-replay-prefix/src/lib.rs	        if !record.is_commit && record_ends_its_publish(record) {	        if false {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- last_record_of_the_publish_without_the_commit_marker	last_record_of_the_publish_without_the_commit_marker_keeps_its_publish_unapplied
+实二十：层 0 L8：恢复在不带提交标记的记录上就停（一事务一条时的旧写法，跨两条记录的事务永远施加不上）	crates/singlefs-replay-prefix/src/lib.rs	        if !record.is_commit && record_ends_its_publish(record) {	        if !record.is_commit {	-p singlefs-checker-tier --test crash_enumeration_record_spill_over_stream -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
 实二十：层 0 L8：恢复把不带末条标志的记录也当末条（半次发布施加上去）	crates/singlefs-core/src/recovery.rs	        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => false,	        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => true,	-p singlefs-checker-tier --test crash_enumeration_record_spill_over_stream -- every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records	every_crash_state_of_a_publish_whose_one_transaction_spills_over_two_records_applies_it_whole_or_not_at_all
 实二十：I-8.9 不判记录标志其余位	crates/singlefs-checker/src/walk.rs	            if record.record_flags_byte & !RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0 {	            if false {	-p singlefs-harness --test checker_known_bad_images -- each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant	each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant_on_its_registered_criteria
 实二十：I-8.9 不判序号 0	crates/singlefs-checker/src/walk.rs	            if record.ordinal_within_publish == 0 {	            if false {	-p singlefs-harness --test checker_known_bad_images -- each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant	each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant_on_its_registered_criteria
@@ -610,8 +610,8 @@ E158 root_choice_repair session s9（实十九提醒）：allocation_record_tree
 增补 2 收口表第 58 行（真设备抬 F 模式）：宿主检查把发布 D 那一段的终点记成发布 C（发布 D 那一段不单独比、红了也指不到它）	crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs	            window_ends.push((ProgramWindow::FourthTransaction, stream.operation_count()));	            window_ends.push((ProgramWindow::ThirdTransaction, stream.operation_count()));	-p singlefs-checker-tier --bin new_pool_file_creation_device_log_check	raise_rollback_floor_mode_compares_the_raise_window_and_is_red_there_when_one_step_changed
 实二二三 a（D23 已定项 14「这一版的失败处置」）：冻结着一次没重发的发布时发布路径不拒（另建下一次发布，同一 (实例, txg) 留下两条末条）	crates/singlefs-core/src/transaction.rs	    match allocator.frozen_publish() {\n        None => Ok(()),\n	    match None::<&FrozenPublish> {\n        None => Ok(()),\n	-p singlefs-harness --test publish_failure_resent_unchanged -- publish_that_fails_midway	publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish
 实二二三 a：原样重发成功之后分配器不换成那次发布成立之后的一份（下一次发布的落点落在重发的那次发布的单元上）	crates/singlefs-core/src/transaction.rs	    *allocator = allocator_after_the_publish;\n    Ok(Some(\n	    let _ = allocator_after_the_publish;\n    Ok(Some(\n	-p singlefs-harness --test publish_failure_resent_unchanged -- publish_that_fails_midway	publish_that_fails_midway_is_frozen_and_resent_byte_for_byte_before_the_next_publish
-实二二三 b（C539，D23 已定项 4 读者规则）：锚点读得出时下一次发布的首条序号不是 1 照接	crates/singlefs-core/src/recovery.rs	        if records_of_the_open_publish.is_empty()\n            && record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST\n        {\n	        if false\n            && records_of_the_open_publish.is_empty()\n            && record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST\n        {\n	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- with_a_readable_anchor_a_next_publish	with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens
-实二二三 b（C540，D23 已定项 4 读者规则）：带末条标志的那一条之后同一 (实例, txg) 还有记录照字面按标志认边界	crates/singlefs-core/src/recovery.rs	        if record_ends_its_publish(record)\n            && readable_record_of_the_same_publish_follows(record, records)\n        {\n	        if false\n            && record_ends_its_publish(record)\n            && readable_record_of_the_same_publish_follows(record, records)\n        {\n	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- last_record_flag_followed_by	last_record_flag_followed_by_the_record_of_the_same_publish_breaks_the_chain_at_the_flag_and_the_checker_reddens
+实二二三 b（C539，D23 已定项 4 读者规则）：锚点读得出时下一次发布的首条序号不是 1 照接	crates/singlefs-replay-prefix/src/lib.rs	        if open.is_empty() && record.ordinal_within_publish != 1 {	        if false && open.is_empty() && record.ordinal_within_publish != 1 {	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- with_a_readable_anchor_a_next_publish	with_a_readable_anchor_a_next_publish_whose_first_ordinal_is_not_one_is_not_applied_and_the_checker_reddens
+实二二三 b（C540，D23 已定项 4 读者规则）：带末条标志的那一条之后同一 (实例, txg) 还有记录照字面按标志认边界	crates/singlefs-replay-prefix/src/lib.rs	        if record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(	        if false && record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(	-p singlefs-harness --test journal_last_record_flag_of_a_multi_record_publish -- last_record_flag_followed_by	last_record_flag_followed_by_the_record_of_the_same_publish_breaks_the_chain_at_the_flag_and_the_checker_reddens
 实二二三 d（D19 已定项 5，用户 2026-09-25 定）：映射条目两条位置项指同一块盘时不拒（另一块盘那一份没核就释放）	crates/singlefs-core/src/transaction.rs	    if locations[0].device == locations[1].device {\n	    if false && locations[0].device == locations[1].device {\n	-p singlefs-harness --test release_checksum_quarantine -- two_mapping_locations_on_the_same_device	two_mapping_locations_on_the_same_device_are_refused_as_a_damaged_mapping_entry_before_anything_is_written
 实二二三 e（I-3.1 / I-3.11 的隔离读法，用户 2026-09-25 定）：已分配而没有根引用的记录一律不豁免	crates/singlefs-checker/src/walk.rs	        if some_copy_fails {\n            for device in devices_of_the_unit {\n	        if false && some_copy_fails {\n            for device in devices_of_the_unit {\n	-p singlefs-harness --test release_checksum_quarantine -- quarantined_record_is_exempted	quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read
 实二二三 e：已分配而没有根引用的记录不读那一份、一律豁免（读得出且对得上的也豁免）	crates/singlefs-checker/src/walk.rs	            .any(|device| !copy_matches_its_location_entry(*device));	            .any(|_device| true);	-p singlefs-harness --test release_checksum_quarantine -- quarantined_record_is_exempted	quarantined_record_is_exempted_from_the_allocated_statistics_only_while_its_copy_fails_the_checker_read
@@ -917,8 +917,8 @@ E158 root_choice_repair 第 2 次跑：V2 的「没拦到」取反（拦到了
 实审 A2a 第 21 条：只读挂载把没有 extent 条目的 inode 当写过内容	crates/singlefs-core/src/mounted_read.rs	            None | Some(ExtentUpperLeafTarget::NoDataUnit) => DataUnitsOfTheInode::NoneWritten,	            None | Some(ExtentUpperLeafTarget::NoDataUnit) => DataUnitsOfTheInode::SomeWritten,	-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode	a_new_empty_inode_opens_under_the_read_only_mount_with_no_data_unit
 实审 A2a 第 21 条：只读挂载把指着数据单元的 inode 当没写过	crates/singlefs-core/src/mounted_read.rs	            ) => DataUnitsOfTheInode::SomeWritten,	            ) => DataUnitsOfTheInode::NoneWritten,	-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode	a_file_overwritten_with_empty_content_still_opens_with_its_one_data_unit
 实审 A2a 第 21 条：没写过的 inode 长度 0 也按除法给一个单元	crates/singlefs-core/src/write_request_split.rs	        DataUnitsOfTheInode::NoneWritten if file_size_in_bytes == 0 => 0,	        DataUnitsOfTheInode::NoneWritten if file_size_in_bytes == 0 => 1,	-p singlefs-core --lib	a_never_written_inode_of_length_zero_implies_no_data_unit_and_a_written_one_implies_one
-实审 A2a 第 25 条：链值不等的记录照样进重放前缀	crates/singlefs-core/src/recovery.rs	            BackChainJudgement::Broken => break,	            BackChainJudgement::Broken => {}	-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode	a_journal_record_whose_back_chain_disagrees_with_the_previous_record_is_left_out_of_the_replayed_prefix
-实审 A2a 第 25 条：前一条不在盘上也断前缀	crates/singlefs-core/src/recovery.rs	            BackChainJudgement::Holds | BackChainJudgement::PreviousRecordNotOnDisk => {}	            BackChainJudgement::Holds => {}\n            BackChainJudgement::PreviousRecordNotOnDisk => break,	-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode	a_journal_record_whose_previous_record_is_not_on_disk_is_replayed_without_judging_its_back_chain
+实审 A2a 第 25 条：链值不等的记录照样进重放前缀	crates/singlefs-core/src/recovery.rs	            BackChainJudgement::Broken => true,	            BackChainJudgement::Broken => false,	-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode	a_journal_record_whose_back_chain_disagrees_with_the_previous_record_is_left_out_of_the_replayed_prefix
+实审 A2a 第 25 条：前一条不在盘上也断前缀	crates/singlefs-core/src/recovery.rs	            BackChainJudgement::Holds | BackChainJudgement::PreviousRecordNotOnDisk => false,	            BackChainJudgement::Holds => false,\n            BackChainJudgement::PreviousRecordNotOnDisk => true,	-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode	a_journal_record_whose_previous_record_is_not_on_disk_is_replayed_without_judging_its_back_chain
 实审 A2a 第 25 条：链值比较写反（对得上的记录被挡）	crates/singlefs-core/src/recovery.rs	            return if back_chain_of(&bytes) == record.back_chain {	            return if back_chain_of(&bytes) != record.back_chain {	-p singlefs-harness --test core_review_geometry_back_chain_and_empty_inode	a_journal_record_whose_back_chain_disagrees_with_the_previous_record_is_left_out_of_the_replayed_prefix
 实审 A2a 第 37 条：槽 0 不可择的盘不借池里别的盘槽 0 记的槽距	crates/singlefs-core/src/recovery.rs	            .or(slot_spacing_recorded_by_the_first_mountable_slot_zero)	            .or(None)	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_device_whose_slot_zero_fails_borrows_the_spacing_another_slot_zero_records_instead_of_searching
 实审A1b Q1：会话推的抬 F 那一串自己取不到落点时仍当抬 F 报错（不当推满仍不够）	crates/singlefs-core/src/mount.rs	        Err(refusal) if floor_raise_refused_for_space(&refusal) => {	        Err(refusal) if false && floor_raise_refused_for_space(&refusal) => {	-p singlefs-harness --test a_floor_raise_refused_for_space_counts_as_short_of_space -- a_floor_raise_pushed_by_the_session_whose_own_publishes_find_no_slot_reports_no_space	a_floor_raise_pushed_by_the_session_whose_own_publishes_find_no_slot_reports_no_space
@@ -1000,8 +1000,8 @@ E158 root_choice_repair 第 3 次跑 M14：读计数只数读成的读、不数
 实审 A2b 第 27 条：分配记录读者不经共用判定	crates/singlefs-core/src/recovery.rs	    let tree_table_entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)?;\n    let Some(allocation_entry) = tree_table_entries	    let tree_table_entries: Vec<TreeTableEntry> = tree_table.entries.iter().filter_map(|bytes| TreeTableEntry::parse(bytes)).collect();\n    let Some(allocation_entry) = tree_table_entries	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_tree_table_carrying_one_kind_of_tree_twice_is_refused_as_damaged_by_every_reader
 实审 A2b 第 27 条：用户可见两棵树的读者不经共用判定	crates/singlefs-core/src/recovery.rs	    let tree_table_entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)?;\n    let mut pointers = UserVisibleTreeRootPointers::ABSENT;	    let tree_table_entries: Vec<TreeTableEntry> = tree_table.entries.iter().filter_map(|bytes| TreeTableEntry::parse(bytes)).collect();\n    let mut pointers = UserVisibleTreeRootPointers::ABSENT;	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_tree_table_carrying_one_kind_of_tree_twice_is_refused_as_damaged_by_every_reader
 实审 A2b 第 27 条：挂载态把同一种树两条报成条目解不开（与别的读者报的不同）	crates/singlefs-core/src/mounted_read.rs	                OpenPoolForReadFailure::Walk(RecoveryFailure::from(refusal))	                OpenPoolForReadFailure::TreeTableEntryMalformed	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_tree_table_carrying_one_kind_of_tree_twice_is_refused_as_damaged_by_every_reader
-实审 A2b 第 36 条：所选根的 txg 改回裸加一	crates/singlefs-core/src/recovery.rs	    let chain_start_txg_without_anchor = root\n        .checkpoint_txg\n        .0\n        .checked_add(1)\n        .map(CheckpointTxg)\n        .ok_or(RecoveryFailure::UnitMalformed {\n            what: CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR,\n        })?;	    let chain_start_txg_without_anchor = CheckpointTxg(root.checkpoint_txg.0 + 1);	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged_instead_of_panicking
-实审 A2b 第 36 条：所选根的 txg 加二才算溢出（边界）	crates/singlefs-core/src/recovery.rs	        .checked_add(1)\n        .map(CheckpointTxg)	        .checked_add(2)\n        .map(CheckpointTxg)	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged_instead_of_panicking
+实审 A2b 第 36 条：所选根的 txg 改回裸加一	crates/singlefs-replay-prefix/src/lib.rs	    if root_checkpoint_txg == u64::MAX {	    if false {	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged_instead_of_panicking
+实审 A2b 第 36 条：所选根的 txg 加二才算溢出（边界）	crates/singlefs-replay-prefix/src/lib.rs	    if root_checkpoint_txg == u64::MAX {	    if root_checkpoint_txg >= u64::MAX - 1 {	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged_instead_of_panicking
 实审 A2b 第 37 条：全池槽 0 都不可择时改回只试最小槽距 4096	crates/singlefs-core/src/recovery.rs	            None => slot_one_found_by_trying_every_slot_spacing_the_format_allows(reader, device)?,	            None => mountable_slot_or_refusal(\n                device,\n                reader.read(device, DeviceOffsetInBytes(FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES), slot_bytes),\n            )?,	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	slot_one_is_found_by_trying_every_spacing_when_no_slot_zero_in_the_pool_verifies
 实审 A2b 第 37 条：逐档按 1024 字节一档试	crates/singlefs-core/src/recovery.rs	const SLOT_SPACING_SEARCH_STEP_BYTES: u64 = 512;	const SLOT_SPACING_SEARCH_STEP_BYTES: u64 = 1024;	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_slot_spacing_that_is_a_multiple_of_512_bytes_only_is_found_too
 实审 A2b 第 37 条：逐档试不核记的槽距等于所在偏移	crates/singlefs-core/src/recovery.rs	                if recorded_spacing != candidate_spacing {	                if recorded_spacing == 0 {	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_self_verifying_slot_whose_recorded_spacing_is_not_its_own_offset_is_not_taken
@@ -1474,3 +1474,24 @@ GPU 派活 量内核大小时不数调用点	crates/singlefs-checker-tier/src/cr
 GPU 派活 量宽度时每个状态变慢了还接着加宽	crates/singlefs-checker-tier/src/crash_judge_dispatch.rs	    seconds / width as f64 <= best_earlier * CALIBRATION_WORSE_THAN_THE_BEST	    seconds / width as f64 <= best_earlier * 1e9	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_dispatch::tests::the_dispatch_width_is_the_quickest_per_state_before_the_card_slows_down	the_dispatch_width_is_the_quickest_per_state_before_the_card_slows_down
 GPU 派活 算得太久的派活下一次不减半	crates/singlefs-checker-tier/src/crash_judge_dispatch.rs	    (width / 2).clamp(NARROWEST_DISPATCH.min(capacity).max(1), capacity.max(1))	    width.clamp(NARROWEST_DISPATCH.min(capacity).max(1), capacity.max(1))	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_dispatch::tests::a_dispatch_that_took_too_long_halves_the_next_one	a_dispatch_that_took_too_long_halves_the_next_one
 GPU 派活 过了拐点的那一档每次重量都再试	crates/singlefs-checker-tier/src/crash_judge_dispatch.rs	        knee.min(widest)	        knee	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_dispatch::tests::a_width_past_the_knee_is_not_tried_again	a_width_past_the_knee_is_not_tried_again
+E165 J2 不查点名单元（记录在就算完整在盘上）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	RecordPresence::OnDisk(unit_state) => unit_state == UnitState::BothCopiesGood,	RecordPresence::OnDisk(_unit_state) => true,	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	root_plus_disk_is_violated_when_a_named_unit_copy_is_bad
+E165 J3 的上限写死成大环的 21	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	applied_record_count <= truth.in_flight_limit	applied_record_count <= 21	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_in_flight_limit_control_is_red_on_the_small_ring_and_green_on_the_big_ring
+E165 j* 不看在飞上限	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	if in_flight_limit.is_some_and(|limit| record_count_so_far > limit) {	if in_flight_limit.is_some_and(|_limit| false) {	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_farthest_version_stops_at_the_in_flight_limit_on_the_small_ring
+E165 J6 不比第二次调用的根、只要有结局就判真	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	let fixed_point_holds = matches!(&second, ReplayOutcome::Returned(second_returned) if second_returned.root == first.root);	let fixed_point_holds = matches!(&second, ReplayOutcome::Returned(_) | ReplayOutcome::Failed(_) | ReplayOutcome::Panicked);	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_fixed_point_control_is_red
+E165 J5a 的「包含于」写成「相等」	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	inner.low == inner.high || (outer.low <= inner.low && inner.high <= outer.high)	(inner.low == inner.high && outer.low == outer.high) || (inner.low == outer.low && inner.high == outer.high)	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	ideal_recovery_on_one_later_record_breaks_the_applied_set_reading_once_and_the_landed_version_reading_never
+E165 J5b 的大于写成大于等于	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	inner.high > outer.high	inner.high >= outer.high	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	ideal_recovery_on_one_later_record_breaks_the_applied_set_reading_once_and_the_landed_version_reading_never
+E165 S 的 (j) 连着看两种读法都走「全表看」	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	        FollowReading::ConsecutiveGapContinues => {\n            a_same_publish_record_follows_before_a_later_txg(record, records)\n        }\n        FollowReading::ConsecutiveGapStops => {\n            a_same_publish_record_follows_without_a_gap(record, records)\n        }	        FollowReading::ConsecutiveGapContinues => {\n            a_same_publish_record_follows_anywhere_in_the_table(record, records)\n        }\n        FollowReading::ConsecutiveGapStops => {\n            a_same_publish_record_follows_anywhere_in_the_table(record, records)\n        }	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_three_readings_part_on_a_later_txg_and_on_a_missing_counter
+E165 S 的「连着看·缺号即停」不在缺号处停	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	        let Some(later) = records.get(&(record.instance, counter)) else {\n            return false;\n        };	        let Some(later) = records.get(&(record.instance, counter)) else {\n            continue;\n        };	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_three_readings_part_on_a_later_txg_and_on_a_missing_counter
+E165 生成器把反向链写成 0	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	Some(previous_slot) => previous_slot.chain_value_of_header,	Some(_previous_slot) => 0,	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	every_generated_record_carries_the_back_chain_of_the_record_before_it
+E165 生成器只给末条带这次发布的新根段	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	let segment = segment_of_this_publish;	let segment = if is_last_record_of_the_publish { segment_of_this_publish } else { NewRootSegment::off_timeline(0, checkpoint_txg) };	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	every_record_of_a_publish_carries_the_same_new_root_segment
+E165 子集枚举漏掉空集	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	0..(1u32 << bit_count)	1..(1u32 << bit_count)	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_state_counts_of_the_smallest_sizes_equal_the_pinned_values_and_the_empty_subset_is_counted
+E165 阳性对照循环只跑判据表第一行	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	for criterion in Criterion::ALL {	for criterion in Criterion::ALL.iter().take(1) {	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	every_criterion_has_a_positive_control_and_every_control_is_red
+E165 本地 F 改成 2	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	const LOCAL_JOURNAL_SAFETY_FACTOR: u64 = 3;	const LOCAL_JOURNAL_SAFETY_FACTOR: u64 = 2;	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	local_constants_equal_the_implementation_constants_and_the_in_flight_limits_are_21_3_2
+E165 A-结构 不要求带末条标志	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	                        && record.is_commit\n                        && record_carries_the_last_record_flag(record)\n                        && VersionFields::of_record(record).checkpoint_txg	                        && record.is_commit\n                        && VersionFields::of_record(record).checkpoint_txg	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_structure_control_is_red
+E165 「记录在」把 1 号盘的残留也算进时间线	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	RecordPresence::NotOnDisk => false,	RecordPresence::NotOnDisk => state.residue.is_some_and(|residue| residue.record_index == record_index),	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	a_residue_on_the_second_device_does_not_complete_a_publish
+E165 逐态判定行漏掉只加了一次的量	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	1 => Some(counter.label().to_string()),	1 => None,	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_counters_a_state_added_are_listed_in_order_with_their_multiplicity
+E165 单状态重放的参数两种写法都要三段	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	let expected_parts = if wants_one_state { 3 } else { 2 };	let expected_parts = if wants_one_state { 3 } else { 3 };	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_state_probe_argument_names_a_domain_a_size_and_an_ordinal
+崩溃放量判法闭包：取前缀核 singlefs-replay-prefix 停在判器闭包门外（判器闭包里没有它）	crates/singlefs-checker-tier/src/crash_identity.rs	加一个模块不改任何判定",\n    ),\n];	加一个模块不改任何判定",\n    ),\n    (\n        "crates/singlefs-replay-prefix/src/lib.rs",\n        "变异：取前缀核停在判器闭包门外",\n    ),\n];	-p singlefs-checker-tier --lib -- the_judging_closure_of_the_judges_holds	the_judging_closure_of_the_judges_holds_recovery_the_checker_and_the_kernels_and_stops_before_the_pipeline
+崩溃放量判法闭包：取前缀核 singlefs-replay-prefix 停在判器闭包门外（改核判法摘要不变）	crates/singlefs-checker-tier/src/crash_identity.rs	加一个模块不改任何判定",\n    ),\n];	加一个模块不改任何判定",\n    ),\n    (\n        "crates/singlefs-replay-prefix/src/lib.rs",\n        "变异：取前缀核停在判器闭包门外",\n    ),\n];	-p singlefs-checker-tier --lib -- the_judging_digest_ignores	the_judging_digest_ignores_the_pipeline_and_the_progress_module_but_follows_every_judge
+取前缀核单测：每条记录都当成它那次发布的末条（末条没到也施加）	crates/singlefs-replay-prefix/src/lib.rs	        if !record_ends_its_publish(record) {	        if false {	-p singlefs-replay-prefix --lib -- a_publish_whose_last_record_is_missing	a_publish_whose_last_record_is_missing_is_not_applied_and_the_root_segment_stays
+取前缀核单测：施加一次发布之后新根段不换（停在所选根的）	crates/singlefs-replay-prefix/src/lib.rs	        rebuilt = record.new_root_segment;	        rebuilt = root_segment;	-p singlefs-replay-prefix --lib -- a_two_record_publish_after_the_anchor	a_two_record_publish_after_the_anchor_is_applied_whole_and_the_new_root_segment_comes_from_its_last_record
diff --git a/crates/singlefs-checker-tier/src/crash_identity.rs b/crates/singlefs-checker-tier/src/crash_identity.rs
index 658035b1..85510143 100644
--- a/crates/singlefs-checker-tier/src/crash_identity.rs
+++ b/crates/singlefs-checker-tier/src/crash_identity.rs
@@ -1128,7 +1128,7 @@ mod tests {
         }
     }
 
-    /// 判器的判法闭包：两个判器、它们调的恢复与池级 checker、格式常量、内存池里的状态拼装、五份内核原文都在；流水线（录入、存储、
+    /// 判器的判法闭包：两个判器、它们调的恢复（连同恢复调的取前缀核 `singlefs-replay-prefix`）与池级 checker、格式常量、内存池里的状态拼装、五份内核原文都在；流水线（录入、存储、
     /// 身份、对账）、进度模块、设点的测试文件、实现的写路径、装置二进制都不在。停止表里每一条都指到现存文件、都真的拦到了（没拦到的是死条目）。
     #[test]
     fn the_judging_closure_of_the_judges_holds_recovery_the_checker_and_the_kernels_and_stops_before_the_pipeline(
@@ -1161,6 +1161,7 @@ mod tests {
             "crates/singlefs-checker/src/walk.rs",
             "crates/singlefs-checker/src/image.rs",
             "crates/singlefs-core/src/recovery.rs",
+            "crates/singlefs-replay-prefix/src/lib.rs",
             "crates/singlefs-format/src/lib.rs",
             "crates/singlefs-harness/src/memory_pool.rs",
         ] {
@@ -1220,7 +1221,7 @@ mod tests {
     }
 
     /// 判定只随判器变：在仓副本里给流水线、存储、身份、进度模块、设点的测试文件、实现的写路径各加一条真语句，判法摘要不动；
-    /// 给 CPU 判器、GPU 判器、它的输入表、恢复、池级 checker 走树各加一条，摘要变；一份内核原文改一个字节，摘要变。
+    /// 给 CPU 判器、GPU 判器、它的输入表、恢复、取前缀核、池级 checker 走树各加一条，摘要变；一份内核原文改一个字节，摘要变。
     #[test]
     fn the_judging_digest_ignores_the_pipeline_and_the_progress_module_but_follows_every_judge() {
         let root = repository_root();
@@ -1271,6 +1272,7 @@ mod tests {
             "crates/singlefs-checker-tier/src/crash_judge_gpu.rs",
             "crates/singlefs-checker-tier/src/crash_judge_tables.rs",
             "crates/singlefs-core/src/recovery.rs",
+            "crates/singlefs-replay-prefix/src/lib.rs",
             "crates/singlefs-checker/src/walk.rs",
         ] {
             assert_ne!(
diff --git a/crates/singlefs-core/Cargo.toml b/crates/singlefs-core/Cargo.toml
index 52e6fccd..3f36a25f 100644
--- a/crates/singlefs-core/Cargo.toml
+++ b/crates/singlefs-core/Cargo.toml
@@ -7,3 +7,5 @@ description = "核心层：地址 newtype 与块设备抽象（步 0），事务
 
 [dependencies]
 singlefs-format = { path = "../singlefs-format" }
+# 取 journal 前缀的纯函数核（Verus 验的那一份），recovery::replay_journal 调它。
+singlefs-replay-prefix = { path = "../singlefs-replay-prefix" }
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index a58214b6..5087a333 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -17,6 +17,7 @@ use singlefs_format::{
     MAPPING_ENTRY_BYTES, NODE_BYTES, ROOT_RECORD_BYTES, ROOT_RING_REGIONS, SLOT_BYTES,
     SYSTEM_CONFIGURATION_SLOT_BYTES,
 };
+use singlefs_replay_prefix::{replay_core, CoreFailure, CoreRecord};
 
 use crate::address::{
     CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
@@ -45,7 +46,7 @@ use crate::instance_table::{
 };
 use crate::journal::{
     back_chain_of, record_offset, slot_after_the_journal_ring, JournalRecord,
-    JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish, FIRST_JOURNAL_COUNTER,
+    JournalRecordPlaceInPublish, FIRST_JOURNAL_COUNTER,
 };
 use crate::make_filesystem::TREE_TABLE_KEY_WIDTH;
 use crate::pointer::{location_entries_ascend_by_device, DataPointer, LocationEntry, NodePointer};
@@ -2936,22 +2937,6 @@ fn record_ends_its_publish(record: &JournalRecord) -> bool {
     }
 }
 
-/// 同一 (实例代号, checkpoint_txg) 里 `record` 之后（计数器更大）还有没有读得出的记录。同一实例里计数器与 checkpoint_txg 一起往上走，
-/// 所以从下一个计数器起按计数器升序看到 txg 越过这一条的就停。
-fn readable_record_of_the_same_publish_follows(
-    record: &JournalRecord,
-    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
-) -> bool {
-    let Some(next_counter) = record.counter.checked_add(1) else {
-        return false;
-    };
-    records
-        .range((record.instance, next_counter)..=(record.instance, u64::MAX))
-        .map(|(_, later)| later)
-        .take_while(|later| later.checkpoint_txg <= record.checkpoint_txg)
-        .any(|later| later.checkpoint_txg == record.checkpoint_txg)
-}
-
 /// 所选根覆盖的最后一条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 14 注 1，读法乙，用户 2026-09-24 定）：
 /// 「那条」按末条标志认——所选根那次发布（与所选根同实例、同 checkpoint_txg）读得出的几条里带「本次发布末条」标志的那一条。
 /// 一条都不带 ⇒ `Ok(None)`，就算「那条读不出」，链首走「序号为 1 的第一条可读记录」那一支；**不取**读得出的同 txg 记录里
@@ -3069,162 +3054,110 @@ pub fn replay_journal(
         verification_failed: 0,
         maximum_applied_transaction: 0,
     };
-    let water = (root.instance, root.checkpoint_txg);
-    let mut rebuilt = *root;
-    // 前缀规则不跨实例边界（D23（journal 的角色与格式） 已定项 14 第 1 条）：链从所选根覆盖的最后一条记录之后接，
-    // 下一条的实例代号与所选根不同即停——所以只有所选根自己那个实例的记录是候选；所选根是 mkfs 的第 0 代根时一条都不施加。
-    let mut above: Vec<&JournalRecord> = records
-        .values()
-        .filter(|record| {
-            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water
-        })
-        .collect();
-    above.sort_by_key(|record| (record.instance, record.counter));
-    report.above_water = above.len();
+    // 取前缀那一截在 `singlefs_replay_prefix::replay_core` 里（Verus 验的是那一份）；这里只做三件：把记录表摊成核的入参、
+    // 把两处读盘（反向链重读、点名单元校验）包成回调交给核、照核交回的结果拼报告与新根。
     let in_flight_limit =
         usize::try_from(journal_in_flight_record_limit(ring_bytes)).expect("在飞上限");
-    // 链首锚点 = 所选根覆盖的最后一条：那次发布里带末条标志的那一条（已定项 14 注 1，读法乙）。
-    // 取那次发布 jsn 最小那条时，一次发布切成多条记录的那一版上链首落在那次发布自己的第二条（它不在水位之上），
-    // 水位之上的第一条对不上号、一条都不施加（三方第一轮 K3：零故障少施加）。
-    // 带标志的那条读不出（两份都撕了）时不知道它的 jsn，链首只能是水位之上第一条可读记录，前提是它的 checkpoint_txg = 根的 txg + 1、
-    // 本次发布内序号为 1（已定项 14 注 1 / 已定项 4）。txg 更大 ⇒ 中间少了一次发布，断号即止（里程碑「覆盖写、释放、回退与复用」步 3
-    // 三方第一轮攻方腿打中：无锚点时无条件接上会跳过撕掉的一条）；序号不是 1 ⇒ 下一次发布的开头缺了，同样断号即止——
-    // 只看 txg 时，下一次发布的第一条也读不出、第二条读得出，链首就接在第二条上，缺了第一条的那次发布照样整体施加
-    // （三方第一轮 K4-b：多接）。
-    let root_own_record_counter = counter_of_the_last_record_the_root_covers(root, records)?;
-    let mut expected_next: Option<(InstanceGeneration, u64)> =
-        root_own_record_counter.map(|counter| (root.instance, counter + 1));
-    // 所选根的 checkpoint_txg 是盘上读来的 8 字节，可以是 u64 的最大值而自证校验和照样对得上：加一溢出报损坏、不 panic
-    // （审阅第 36 条）。在循环之前算：txg 最大值的根后面接不上任何一次发布，锚点读不读得出都一样。
-    let chain_start_txg_without_anchor = root
-        .checkpoint_txg
-        .0
-        .checked_add(1)
-        .map(CheckpointTxg)
-        .ok_or(RecoveryFailure::UnitMalformed {
-            what: CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR,
-        })?;
-    // 这次发布已经过了前四条、还没走到末条的记录（第五条：末条到了才整体施加）。
-    let mut records_of_the_open_publish: Vec<&JournalRecord> = Vec::new();
-    for record in above.into_iter().take(in_flight_limit) {
-        if let Some(expected_key) = expected_next {
-            if (record.instance, record.counter) != expected_key {
-                break;
-            }
-        } else if record.checkpoint_txg != chain_start_txg_without_anchor
-            || record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST
-        {
-            break;
-        }
-        // 反向链（D23（journal 的角色与格式） 已定项 7「jsn 严格连续 + 校验和 + 反向链」；I-8.6（反向链算法））：
-        // 校验和自洽、jsn 恰好接得上而链值不等的记录（上一条时间线的残留、改过的镜像）断在这一条，它所在的这次发布整体不施加。
-        match judge_back_chain_against_the_previous_record_on_disk(
-            reader, record, ring_bytes, records,
+    // `records` 按 (实例代号, 计数器) 升序，`values()` 就是那个次序；核里「末条之后同发布还有没有记录」那一判靠这个次序。
+    let records_in_key_order: Vec<&JournalRecord> = records.values().collect();
+    // 核的「新根段」取记录在表里的下标：`None` 是一条都没施加，`Some(下标)` 是最后施加的那次发布的末条。
+    let core_records: Vec<CoreRecord<Option<usize>>> = records_in_key_order
+        .iter()
+        .enumerate()
+        .map(|(index, record)| CoreRecord {
+            instance: record.instance.0,
+            counter: record.counter,
+            checkpoint_txg: record.checkpoint_txg.0,
+            transaction: record.transaction,
+            is_commit: record.is_commit,
+            ordinal_within_publish: record.ordinal_within_publish.0,
+            ends_its_publish: record_ends_its_publish(record),
+            new_root_segment: Some(index),
+        })
+        .collect();
+    // 反向链（D23（journal 的角色与格式） 已定项 7「jsn 严格连续 + 校验和 + 反向链」；I-8.6（反向链算法））：
+    // 校验和自洽、jsn 恰好接得上而链值不等的记录（上一条时间线的残留、改过的镜像）断在这一条，它所在的这次发布整体不施加。
+    let back_chain_broken =
+        |index: usize| match judge_back_chain_against_the_previous_record_on_disk(
+            reader,
+            records_in_key_order[index],
+            ring_bytes,
+            records,
         ) {
-            BackChainJudgement::Holds | BackChainJudgement::PreviousRecordNotOnDisk => {}
-            BackChainJudgement::Broken => break,
-        }
-        // 一次发布的第一条序号是 1（D23（journal 的角色与格式） 已定项 4 读者规则：锚点读得出时，下一次发布的首条序号不是 1，
-        // 当那条记录损坏、断在这一条，那次发布整体不施加；C539（锚点读得出时下一次发布的首条序号不是 1））。锚点读不出那一支上面已经判过；
-        // 这一判管锚点读得出时接上的那一次，与之后每一次新开的发布。
-        if records_of_the_open_publish.is_empty()
-            && record.ordinal_within_publish != JournalRecordOrdinalWithinPublish::FIRST
-        {
-            break;
-        }
-        if let Some(previous_record_of_the_open_publish) = records_of_the_open_publish.last() {
-            // 一次发布的记录共享一个 checkpoint_txg（D16（发布语义） 已定项 6）：末条还没到、下一条已经换了 txg
-            // ⇒ 这次发布缺了末条，断在这里，它整体不施加。
-            if record.checkpoint_txg != previous_record_of_the_open_publish.checkpoint_txg {
-                break;
-            }
-            // 一次发布的 N 条记录序号依次 1..N、与 jsn 同步（D23（journal 的角色与格式） 已定项 4）：一次发布之内跳号，
-            // 当这条记录损坏、断链即止——这次发布走不到末条，整体不施加。
-            if u64::from(record.ordinal_within_publish.0)
-                != u64::from(previous_record_of_the_open_publish.ordinal_within_publish.0) + 1
-            {
-                break;
-            }
-            // 提交标记（D23（journal 的角色与格式） 已定项 7）：一个事务可以跨多条记录，只有它的最后一条带提交标记
-            // （最后一个事务装不下一条记录时末条再跨记录，已定项 17）。上一条没带提交标记而这一条换了事务号 ⇒
-            // 那个事务的提交标记没出现，它被丢掉（已定项 7「丢掉提交标记还没出现的那个事务的全部记录」），
-            // 它所在的这次发布因此不完整、整体不施加（第五条）。
-            if !previous_record_of_the_open_publish.is_commit
-                && record.transaction != previous_record_of_the_open_publish.transaction
-            {
-                break;
-            }
-        }
-        expected_next = Some((record.instance, record.counter + 1));
-        // 末条不带提交标记 ⇒ 这次发布最后一个事务没提交，这次发布整体不施加。末条之外不带提交标记的，是一个跨多条记录的事务
-        // 还没写到它的最后一条，接着往下走（它换了事务号还没等到提交标记，上面那一判断链）。
-        if !record.is_commit && record_ends_its_publish(record) {
-            break;
-        }
-        // 带末条标志的这一条之后，同一 (实例代号, checkpoint_txg) 里还有读得出的记录（D23（journal 的角色与格式） 已定项 4 读者规则：
-        // 当这条记录损坏、断在带标志的这一条，那次发布整体不施加；C540（末条标志坏在一次发布中间，读者切出两次发布））。
-        // 合法历史里一次发布只有真正的最后一条带标志（已定项 17），失败的那次原样重发（已定项 14「这一版的失败处置」），走到这里要一条记录坏了而
-        // 校验和恰好仍对得上，或者镜像是改出来的。
-        if record_ends_its_publish(record)
-            && readable_record_of_the_same_publish_follows(record, records)
-        {
-            break;
+            BackChainJudgement::Holds | BackChainJudgement::PreviousRecordNotOnDisk => false,
+            BackChainJudgement::Broken => true,
+        };
+    let named_units_verified = |index: usize| {
+        records_in_key_order[index].named.iter().all(|named| {
+            let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
+                return false;
+            };
+            named.locations.iter().all(|location| {
+                reader
+                    .read(
+                        location.device,
+                        location.slot.to_device_offset(),
+                        unit_bytes,
+                    )
+                    .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
+            })
+        })
+    };
+    let outcome = match replay_core(
+        root.instance.0,
+        root.checkpoint_txg.0,
+        None,
+        &core_records,
+        in_flight_limit,
+        verify_named_units,
+        back_chain_broken,
+        named_units_verified,
+    ) {
+        Ok(outcome) => outcome,
+        // 报错里的计数器清单核不产，由原来那个函数再算一遍（它在同一张表上必然报同一个错）。
+        Err(CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlag) => {
+            return Err(
+                match counter_of_the_last_record_the_root_covers(root, records) {
+                    Err(failure) => failure,
+                    Ok(_) => unreachable!("核报了所选根那次发布带末条标志的多于一条"),
+                },
+            );
         }
-        let all_verified = !verify_named_units
-            || record.named.iter().all(|named| {
-                let Some(unit_bytes) = unit_bytes_for_class(named.unit_class) else {
-                    return false;
-                };
-                named.locations.iter().all(|location| {
-                    reader
-                        .read(
-                            location.device,
-                            location.slot.to_device_offset(),
-                            unit_bytes,
-                        )
-                        .is_some_and(|bytes| crc32_castagnoli(&bytes) == location.unit_checksum)
-                })
+        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {
+            return Err(RecoveryFailure::UnitMalformed {
+                what: CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR,
             });
-        if !all_verified {
-            report.verification_failed += 1;
-            break;
-        }
-        // 只数**真验过**的：`verify_named_units` 关掉时上面那句把 `all_verified` 短路成真、一个单元都没读，
-        // 这一格再加一就等于宣称验过了（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支必须可观测——
-        // 两臂在健康镜像上报出逐字相同的数，运行时就看不出走了哪一条）。
-        if verify_named_units {
-            report.verification_passed += 1;
         }
-        records_of_the_open_publish.push(record);
-        // 第五条：末条没到就接着往下走，这次发布先不施加。
-        if !record_ends_its_publish(record) {
-            continue;
+    };
+    report.above_water = outcome.report.above_water;
+    report.prefix_applied = outcome.report.prefix_applied;
+    report.verification_passed = outcome.report.verification_passed;
+    report.verification_failed = outcome.report.verification_failed;
+    report.maximum_applied_transaction = outcome.report.maximum_applied_transaction;
+    let rebuilt = match outcome.rebuilt {
+        None => *root,
+        Some(index) => {
+            let record = records_in_key_order[index];
+            // 每条记录都带整次发布的新根段（已定项 15），施加末条那一份。
+            RootRecord {
+                filesystem_identifier: root.filesystem_identifier,
+                // 新根段（D23（journal 的角色与格式） 已定项 15）里没有 flags：由记录施加出来的这一版没有一条根槽写过它，
+                // 不是卸载那一串写下的根（D22（单元原子性怎么合成） 已定项 7「其余根写 0」）。它只当下一次发布接在后面的那一版，
+                // 发布照自己的计划写记号、不照抄这一项。
+                unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
+                instance: record.instance,
+                checkpoint_txg: record.checkpoint_txg,
+                tree_table: record.new_tree_table,
+                tree_identifier_watermark: record.new_tree_identifier_watermark,
+                rollback_floor: record.new_rollback_floor,
+                instance_table: root.instance_table,
+                mapping_root: record.new_mapping_root,
+                // 与实例表指针同一条理由：新根段（D23（journal 的角色与格式） 已定项 15）里没有这一项，施加记录只能照抄被施加的那条根的。
+                // 施加一条写行记录而它的根槽没落盘时，重建出来的这一版仍指着上一版的分配记录树——那正是「这次写行没有成立」该有的账。
+                allocation_record_tree_root: root.allocation_record_tree_root,
+            }
         }
-        report.prefix_applied += records_of_the_open_publish.len();
-        report.maximum_applied_transaction = records_of_the_open_publish
-            .iter()
-            .map(|applied| applied.transaction)
-            .fold(report.maximum_applied_transaction, u64::max);
-        records_of_the_open_publish.clear();
-        // 每条记录都带整次发布的新根段（已定项 15），施加末条那一份。
-        rebuilt = RootRecord {
-            filesystem_identifier: rebuilt.filesystem_identifier,
-            // 新根段（D23（journal 的角色与格式） 已定项 15）里没有 flags：由记录施加出来的这一版没有一条根槽写过它，
-            // 不是卸载那一串写下的根（D22（单元原子性怎么合成） 已定项 7「其余根写 0」）。它只当下一次发布接在后面的那一版，
-            // 发布照自己的计划写记号、不照抄这一项。
-            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
-            instance: record.instance,
-            checkpoint_txg: record.checkpoint_txg,
-            tree_table: record.new_tree_table,
-            tree_identifier_watermark: record.new_tree_identifier_watermark,
-            rollback_floor: record.new_rollback_floor,
-            instance_table: rebuilt.instance_table,
-            mapping_root: record.new_mapping_root,
-            // 与实例表指针同一条理由：新根段（D23（journal 的角色与格式） 已定项 15）里没有这一项，施加记录只能照抄被施加的那条根的。
-            // 施加一条写行记录而它的根槽没落盘时，重建出来的这一版仍指着上一版的分配记录树——那正是「这次写行没有成立」该有的账。
-            allocation_record_tree_root: rebuilt.allocation_record_tree_root,
-        };
-    }
+    };
     Ok((report, rebuilt))
 }
 
```
