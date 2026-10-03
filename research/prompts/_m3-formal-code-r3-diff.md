# 代码轮第三轮被判的改动全文（2026-10-03）

基：代码轮第二轮冻结副本 `/tmp/claude-1000/m3-formal-code-r2/tree/`；今天：主工作区（第 12 批 12-2、第 13 批 13-3 与 13-4、主 agent 一处 clippy 改动、E165 r4 / E170 r1 / E171 三个执行员的装置）。

## `crates/mutations.tsv`

````diff
--- crates/mutations.tsv	2026-10-02 15:34:49.518197067 +0000
+++ crates/mutations.tsv	2026-10-02 18:08:52.382228585 +0000
@@ -1809,7 +1809,7 @@
 C462 空池择系统配置退回报 SystemConfigurationsDisagree	crates/singlefs-core/src/recovery.rs	        (None, None) => Err(RecoveryFailure::NoDeviceInThePool),	        (None, None) => Err(RecoveryFailure::SystemConfigurationsDisagree),	-p singlefs-harness --test system_configuration_choice_names_an_empty_pool_and_devices_that_lost_both_slots	a_pool_without_any_device_is_refused_as_having_no_device
 C463 择系统配置不记两槽都不可择的盘	crates/singlefs-core/src/recovery.rs	                devices_without_a_mountable_system_configuration_slot.push(device);	                let _ = device;	-p singlefs-harness --test system_configuration_choice_names_an_empty_pool_and_devices_that_lost_both_slots	a_device_whose_two_system_configuration_slots_are_both_lost_is_named_by_the_read_only_mount_and_recovery
 C463 只读挂载不交出两槽都不可择的盘	crates/singlefs-core/src/mounted_read.rs	        devices_without_a_mountable_system_configuration_slot: system_configuration_choice\n            .devices_without_a_mountable_system_configuration_slot,	        devices_without_a_mountable_system_configuration_slot: Vec::new(),	-p singlefs-harness --test system_configuration_choice_names_an_empty_pool_and_devices_that_lost_both_slots	a_device_whose_two_system_configuration_slots_are_both_lost_is_named_by_the_read_only_mount_and_recovery
-E165 r2 第四段 扫盘建表的系统配置 fsid 写成全 0（全部落盘那一态的扫盘表条数 1 变 0）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	            filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,\n            this_device: DeviceIdentity(0),	            filesystem_identifier: [0; 16],\n            this_device: DeviceIdentity(0),	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_two_table_sources_agree_on_the_smallest_sizes_and_the_scan_table_of_every_fully_landed_state_holds_every_record
+E165 r2 第四段 扫盘建表的系统配置 fsid 写成全 0（全部落盘那一态的扫盘表条数 1 变 0）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	fn system_configuration_for_the_scan(ring: RingGeometry) -> SystemConfiguration {\n    SystemConfiguration {\n        immutable: SystemImmutableConfiguration {\n            filesystem_identifier: ROOT_FILESYSTEM_IDENTIFIER,	fn system_configuration_for_the_scan(ring: RingGeometry) -> SystemConfiguration {\n    SystemConfiguration {\n        immutable: SystemImmutableConfiguration {\n            filesystem_identifier: [0; 16],	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_two_table_sources_agree_on_the_smallest_sizes_and_the_scan_table_of_every_fully_landed_state_holds_every_record
 E165 r2 第四段 T-表只比键集合、不比值	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	    direct_table != scanned_table	    !direct_table.keys().eq(scanned_table.keys())	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	two_record_tables_with_the_same_keys_and_a_different_new_root_segment_are_reported_as_different
 E165 r2 第四段 T-行只比计数器标签、不比实现结局	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	    describe_outcome(&direct.first_outcome) != describe_outcome(&scanned.first_outcome)\n        || direct.counter_labels != scanned.counter_labels	    direct.counter_labels != scanned.counter_labels	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	two_judged_rows_with_the_same_counter_labels_and_a_failure_against_a_returned_root_are_reported_as_different
 E165 r2 第四段 --disk-order reversed 不生效（盘报的次序还是 0 号盘在前）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	            Self::SecondDeviceBeforeFirst => [1, 0],	            Self::SecondDeviceBeforeFirst => [0, 1],	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	a_reversed_memory_disk_reports_the_second_device_before_the_first
@@ -1848,23 +1848,20 @@
 checker ③ 逐档读到带读者不收之值的槽时跳过它、不整池拒	crates/singlefs-checker/src/image.rs	            refused @ SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. } => {\n                return refused;\n            }	            SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { .. } => {}	-p singlefs-harness --test system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery -- a_slot_carrying_a_value_the_reader_refuses_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery	a_slot_carrying_a_value_the_reader_refuses_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery
 checker ③ 逐档读到的自证过的槽都不可择时报成自证不过	crates/singlefs-checker/src/image.rs	        (None, true) => SystemConfigurationSlotReading::SelfVerifiedButNotChoosable,	        (None, true) => SystemConfigurationSlotReading::NotSelfVerified,	-p singlefs-harness --test system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery -- only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery	only_self_verified_slots_that_are_not_choosable_found_while_reading_every_spacing_choose_nothing_in_both_the_checker_and_recovery
 RefFS：写之前拒的操作也进序列（被拒的覆盖写在之后的发布里出现）	crates/singlefs-reference-filesystem/src/lib.rs	OperationOutcome::RefusedBeforeAnyWrite => return Ok(SubmittedOperation::DidNotHappen),	OperationOutcome::RefusedBeforeAnyWrite => EnteredOutcome::Succeeded,	-p singlefs-reference-filesystem --lib	an_overwrite_refused_before_any_write_did_not_happen_so_the_publish_after_it_does_not_allow_its_content
-RefFS：允许集合放进切开一次发布的 S_j（不看发布末尾）	crates/singlefs-reference-filesystem/src/lib.rs	if operation.kind.ends_a_publish() {	if true {	-p singlefs-reference-filesystem --lib	a_crash_after_two_overwrites_and_one_publish_does_not_allow_the_state_that_cuts_the_publish
-RefFS：lo 取 fsync 调用前一刻再往前一个操作	crates/singlefs-reference-filesystem/src/lib.rs	.map_or(StateIndex(0), |(position, _)| StateIndex(position));	.map_or(StateIndex(0), |(position, _)| StateIndex(position.saturating_sub(1)));	-p singlefs-reference-filesystem --lib	after_a_successful_fsync_the_state_just_before_the_fsync_call_is_the_lower_bound
-RefFS：fsync 报错时把 lo 抬到报错那次	crates/singlefs-reference-filesystem/src/lib.rs	operation.kind == EnteredKind::Fsync && operation.outcome == EnteredOutcome::Succeeded	operation.kind == EnteredKind::Fsync	-p singlefs-reference-filesystem --lib	a_failed_fsync_keeps_the_lower_bound_at_the_previous_successful_fsync
+RefFS：允许集合放进切开一次发布的 S_j（不看发布末尾）	crates/singlefs-reference-filesystem/src/lib.rs	if operation.kind.ends_a_publish() {	if true {	-p singlefs-reference-filesystem --lib	a_crash_inside_a_publish_that_carries_two_overwrites_does_not_allow_the_state_that_cuts_the_publish
+RefFS：fsync 报错时把 lo 抬到报错那次	crates/singlefs-reference-filesystem/src/lib.rs	operation.outcome == EnteredOutcome::Succeeded && operation.kind.ends_a_publish()	operation.kind.ends_a_publish()	-p singlefs-reference-filesystem --lib	a_failed_fsync_keeps_the_lower_bound_at_the_previous_successful_fsync
 RefFS：写了之后报错的发布不算发布末尾（两可丢了「可在」那一半）	crates/singlefs-reference-filesystem/src/lib.rs	if operation.kind.ends_a_publish() {	if operation.kind.ends_a_publish() && operation.outcome == EnteredOutcome::Succeeded {	-p singlefs-reference-filesystem --lib	a_publish_that_failed_after_writing_may_be_seen_after_the_crash
 RefFS：允许集合不截在 hi（崩溃之后的发布也算）	crates/singlefs-reference-filesystem/src/lib.rs	.take(upper_bound.0)	.take(operations.len())	-p singlefs-reference-filesystem --lib	a_crash_inside_an_overwrite_does_not_allow_the_publish_that_comes_after_it
 RefFS：崩溃落在里面的 fsync 也抬 lo	crates/singlefs-reference-filesystem/src/lib.rs	let returned_before_the_crash = &operations[..crash_falls_in.0 - 1];	let returned_before_the_crash = &operations[..crash_falls_in.0.min(operations.len())];	-p singlefs-reference-filesystem --lib	a_crash_inside_an_fsync_does_not_raise_the_lower_bound
 RefFS：恢复之后不按读出内容重新起头（接着原来的直线编号）	crates/singlefs-reference-filesystem/src/lib.rs	self.state_at_the_last_recovery = recovered;\n        self.operations_since_the_last_recovery.clear();	let _ = recovered;	-p singlefs-reference-filesystem --lib	after_a_recovery_the_next_crash_is_judged_from_the_recovered_content_so_the_dropped_overwrite_does_not_come_back
 RefFS：恢复读出的内容不判允许集合	crates/singlefs-reference-filesystem/src/lib.rs	if !allowed.contains(&recovered) {	if false {	-p singlefs-reference-filesystem --lib	a_recovered_state_outside_the_allowed_set_is_refused_and_the_model_does_not_move
-RefFS：回退取第 m+1 次已确认发布	crates/singlefs-reference-filesystem/src/lib.rs	let Some(position) = number.0.checked_sub(1) else {	let Some(position) = number.0.checked_add(0) else {	-p singlefs-reference-filesystem --lib	rolling_back_to_the_first_confirmed_publish_gives_the_content_after_that_publish
-RefFS：回退不算发布末尾	crates/singlefs-reference-filesystem/src/lib.rs	EnteredKind::RollBack => true,	EnteredKind::RollBack => false,	-p singlefs-reference-filesystem --lib	a_crash_right_after_a_rollback_allows_the_rolled_back_content_even_below_the_fsynced_state
+RefFS：回退不算发布末尾	crates/singlefs-reference-filesystem/src/lib.rs	EnteredKind::RollBack => true,	EnteredKind::RollBack => false,	-p singlefs-reference-filesystem --lib	a_crash_inside_a_rollback_allows_the_rolled_back_content_even_below_the_fsynced_state
 RefFS：写了之后报错的发布也算已确认	crates/singlefs-reference-filesystem/src/lib.rs	operation.kind.ends_a_publish() && operation.outcome == EnteredOutcome::Succeeded	operation.kind.ends_a_publish()	-p singlefs-reference-filesystem --lib	a_publish_that_failed_after_writing_is_not_counted_as_a_confirmed_publish_for_rollback
 RefFS：崩溃落在里面的发布也算已确认	crates/singlefs-reference-filesystem/src/lib.rs	&self.operations_since_the_last_recovery[..crash_falls_in.0 - 1];	&self.operations_since_the_last_recovery[..crash_falls_in.0.min(self.operations_since_the_last_recovery.len())];	-p singlefs-reference-filesystem --lib	a_publish_the_crash_fell_inside_is_not_counted_as_a_confirmed_publish_for_rollback
 RefFS：写了之后报错、恢复之前又起的操作照判（不按条款没定拒）	crates/singlefs-reference-filesystem/src/lib.rs	if failed_position + 1 < started_before_the_crash.len() {	if false {	-p singlefs-reference-filesystem --lib	an_overwrite_after_a_failed_fsync_before_the_next_recovery_is_refused_as_not_in_the_first_version_and_the_model_does_not_move
 RefFS：崩溃号超过崩溃本身不拒	crates/singlefs-reference-filesystem/src/lib.rs	if crash_falls_in.0 == 0 || crash_falls_in > crash_operation_number {	if crash_falls_in.0 == 0 {	-p singlefs-reference-filesystem --lib	a_crash_number_beyond_the_crash_itself_or_zero_is_refused_and_the_model_does_not_move
 RefFS：已有文件时再建第一个文件照收	crates/singlefs-reference-filesystem/src/lib.rs	VisibleState::OneFile(_) => return Err(OperationNotAccepted::CreatingTheFirstFileWhileAFileExistsIsNotInTheFirstVersion),	VisibleState::OneFile(_) => (EnteredKind::ContentChange, VisibleState::OneFile(content)),	-p singlefs-reference-filesystem --lib	creating_the_first_file_twice_is_refused_and_the_model_does_not_move
 RefFS：还没有文件时覆盖写照收	crates/singlefs-reference-filesystem/src/lib.rs	VisibleState::NoFile => return Err(OperationNotAccepted::OverwritingWithoutAFileIsNotInTheFirstVersion),	VisibleState::NoFile => (EnteredKind::ContentChange, VisibleState::OneFile(content)),	-p singlefs-reference-filesystem --lib	overwriting_before_any_file_exists_is_refused_and_the_model_does_not_move
-RefFS：回退目标第 0 次当第 1 次收	crates/singlefs-reference-filesystem/src/lib.rs	let Some(position) = number.0.checked_sub(1) else {	let Some(position) = Some(number.0.saturating_sub(1)) else {	-p singlefs-reference-filesystem --lib	rolling_back_to_confirmed_publish_zero_is_refused_and_the_model_does_not_move
 择根胶水把择新键两项对调（代码轮 m3-formal-code-r1 K1-b，攻方 G1）：核看到的 txg 是实例代号、实例代号是 txg，实例代号高、txg 小的根赢过 txg 大的	crates/singlefs-core/src/recovery.rs	                checkpoint_txg: PrefixKernelCheckpointTxg(root.checkpoint_txg.0),\n                instance: PrefixKernelInstanceGeneration(root.instance.0),	                checkpoint_txg: PrefixKernelCheckpointTxg(u64::from(root.instance.0)),\n                instance: PrefixKernelInstanceGeneration(u32::try_from(root.checkpoint_txg.0).unwrap_or(u32::MAX)),	-p singlefs-harness --test root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance	a_later_checkpoint_txg_in_an_earlier_slot_beats_a_higher_instance_through_choose_root
 恢复胶水拿所选槽之外的根调重放（代码轮 m3-formal-code-r1 K1-b）：recover 把读根环次序里第一条自证过的根交给 replay_journal，不是 choose_root 择的那条	crates/singlefs-core/src/recovery.rs	                &root,\n                system_configuration.immutable.sizes.journal_ring_bytes,	                &readable_roots_with_ring_slots(reader, &system_configuration.immutable.region_devices, &system_configuration.immutable.sizes, &system_configuration.immutable.filesystem_identifier)[0].1,\n                system_configuration.immutable.sizes.journal_ring_bytes,	-p singlefs-harness --test root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance	recover_replays_from_the_root_in_the_chosen_slot_not_from_another_self_verified_root
 F 生效值择最新根平局取后者（代码轮 m3-formal-code-r1 K1-d）：newest_root_among 择新键完全相同时换成读根环次序里后面那条，与 choose_root 取前面那条不同，判有效根读了另一张实例表	crates/singlefs-core/src/recovery.rs	            if (candidate.checkpoint_txg, candidate.instance)\n                > (newest_so_far.checkpoint_txg, newest_so_far.instance)	            if (candidate.checkpoint_txg, candidate.instance)\n                >= (newest_so_far.checkpoint_txg, newest_so_far.instance)	-p singlefs-harness --test root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance	of_two_roots_with_the_same_choice_key_the_effective_floor_reads_the_instance_table_of_the_earlier_slot_as_choose_root_does
@@ -1893,7 +1890,7 @@
 O1 判官判据 ③ 只比 txg、不比实例（11-4）	crates/singlefs-harness/src/model.rs	        if chosen < persisted {	        if chosen.checkpoint_txg < persisted.checkpoint_txg {	-p singlefs-harness --test crash_recovery_judge_refuses_a_root_older_than_the_newest_persisted_one	landing_on_the_lower_instance_of_the_same_txg_as_the_newest_persisted_root_is_red
 O1 判官去掉判据 ③（11-4）	crates/singlefs-harness/src/model.rs	        if chosen < persisted {	        if false && chosen < persisted {	-p singlefs-harness --test crash_recovery_judge_refuses_a_root_older_than_the_newest_persisted_one	landing_on_an_older_txg_than_the_newest_persisted_root_is_red
 崩溃注入（11-4）：挂载加一次发布那一个的挂载透明不判	crates/singlefs-checker-tier/src/crash_injection.rs	        (WritableMountOutcome::Published, Some(floor)) => {	        (WritableMountOutcome::Published, Some(floor)) if false => {	-p singlefs-checker-tier --lib	mount_transparency_is_judged_after_every_writable_mount_and_inside_every_mount_phase_without_a_red_field
-崩溃注入（11-4）：二次崩溃镜像上不判挂载透明	crates/singlefs-checker-tier/src/crash_injection.rs	    let Some(floor) = floor_raised_by_the_uninterrupted_mount else {	    let Some(floor) = floor_raised_by_the_uninterrupted_mount.filter(|_| false) else {	-p singlefs-checker-tier --lib	mount_transparency_is_judged_after_every_writable_mount_and_inside_every_mount_phase_without_a_red_field
+崩溃注入（11-4）：二次崩溃镜像上不判挂载透明	crates/singlefs-checker-tier/src/crash_injection.rs	    let Some(floor_raises_in_the_mount_writes) = floor_raises_in_the_uninterrupted_mount_writes	    let Some(floor_raises_in_the_mount_writes) = floor_raises_in_the_uninterrupted_mount_writes.filter(|_| false)	-p singlefs-checker-tier --lib	mount_transparency_is_judged_after_every_writable_mount_and_inside_every_mount_phase_without_a_red_field
 E168 U12 读法乙：B 臂被落点直接拒也算经会话推抬 F（B 臂不经会话也成立的样子，登记修订第 17 条）	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	Some(refusal == Some(DeleteRefusal::NoSpaceAfterRaisingTheFloor))	Some(refusal.is_some())	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	positive_control_u12_holds_for_a_session_refusal_whose_floor_raise_was_refused_in_the_rehearsal_and_not_for_a_landing_refusal
 E168 U12 那个数把 B 臂没出拒绝的格也算进去（登记修订第 17 条）	crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs	verdict.is_refused\n                    && verdict.is_refused_with_no_space_after_raising_the_floor == Some(false)	verdict.is_refused_with_no_space_after_raising_the_floor == Some(false)	-p singlefs-checker-tier --bin e168_delete_ring_window_borrow	positive_control_verdict_counts_the_refusal_and_the_session_floor_raise_refusal_apart_and_stops_on_either
 E165 r3 第二段 「已确认」把崩溃点所在那次发布也算进去（K M = 1、U = 1「有已确认」0 变 39）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	&world.confirmed_user_publishes,	&[world.confirmed_user_publishes.as_slice(), &[crashed.material.version]].concat(),	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_root_choice_state_counts_of_the_smallest_cells_equal_the_pinned_values_and_the_kw_wrap_cells_read_four_and_one
@@ -1965,3 +1962,64 @@
 第 5 批 GPU 判器：派活给每个状态的草稿区停在交接段末端、没跟上内核新加的两张表	crates/singlefs-checker-tier/src/crash_judge_gpu.rs	pub const SCRATCH_WORDS: usize = 255_504;	pub const SCRATCH_WORDS: usize = 247_312;	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_gpu::tests::scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels	scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels
 只读挂载胶水拿所选槽之外的根调重放（代码轮 m3-formal-code-r1 K1-b，改法落地一节余下的一处）：mount_read_only 把读根环次序里第一条自证过的根交给 replay_journal，不是 choose_root 择的那条	crates/singlefs-core/src/mounted_read.rs	        &chosen_root,\n        system_configuration.immutable.sizes.journal_ring_bytes,	        &crate::recovery::readable_roots_with_ring_slots(reader, &system_configuration.immutable.region_devices, &system_configuration.immutable.sizes, &system_configuration.immutable.filesystem_identifier)[0].1,\n        system_configuration.immutable.sizes.journal_ring_bytes,	-p singlefs-harness --test root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance	mount_read_only_replays_from_the_root_in_the_chosen_slot_not_from_another_self_verified_root
 可写挂载读阶段胶水拿所选槽之外的根调重放（代码轮 m3-formal-code-r1 K1-b，改法落地一节余下的一处）：read_stage 把读根环次序里第一条自证过的根交给 replay_journal，不是 choose_root 择的那条	crates/singlefs-core/src/mount.rs	        &chosen_root,\n        system_configuration.immutable.sizes.journal_ring_bytes,	        &crate::recovery::readable_roots_with_ring_slots(cache, &system_configuration.immutable.region_devices, &system_configuration.immutable.sizes, &system_configuration.immutable.filesystem_identifier)[0].1,\n        system_configuration.immutable.sizes.journal_ring_bytes,	-p singlefs-harness --test root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance	mount_writable_replays_from_the_root_in_the_chosen_slot_not_from_another_self_verified_root
+RefFS：lo 取已确认发布调用之前那一刻（不取它自己的末尾）	crates/singlefs-reference-filesystem/src/lib.rs	.map_or(StateIndex(0), |(position, _)| StateIndex(position + 1));	.map_or(StateIndex(0), |(position, _)| StateIndex(position));	-p singlefs-reference-filesystem --lib	after_a_successful_fsync_its_own_index_is_the_lower_bound_so_the_content_before_it_is_not_allowed
+RefFS：回退取第 m+1 版	crates/singlefs-reference-filesystem/src/lib.rs	let Some(position) = number.0.checked_sub(1) else {	let Some(position) = number.0.checked_add(0) else {	-p singlefs-reference-filesystem --lib	rolling_back_to_the_first_confirmed_publish_gives_the_content_after_that_publish
+RefFS：回退目标第 0 版当第 1 版收	crates/singlefs-reference-filesystem/src/lib.rs	let Some(position) = number.0.checked_sub(1) else {	let Some(position) = Some(number.0.saturating_sub(1)) else {	-p singlefs-reference-filesystem --lib	rolling_back_to_version_zero_is_refused_and_the_model_does_not_move
+RefFS：只有 fsync 抬 lo（返回成功的发布调用与回退不抬）	crates/singlefs-reference-filesystem/src/lib.rs	operation.outcome == EnteredOutcome::Succeeded && operation.kind.ends_a_publish()	operation.outcome == EnteredOutcome::Succeeded && operation.kind == EnteredKind::Fsync	-p singlefs-reference-filesystem --lib	after_a_publish_call_returns_the_content_before_it_is_no_longer_allowed
+RefFS：返回成功的回退不抬 lo	crates/singlefs-reference-filesystem/src/lib.rs	operation.outcome == EnteredOutcome::Succeeded && operation.kind.ends_a_publish()	operation.outcome == EnteredOutcome::Succeeded\n                && operation.kind != EnteredKind::RollBack\n                && operation.kind.ends_a_publish()	-p singlefs-reference-filesystem --lib	after_a_rollback_returns_reading_the_content_before_the_rollback_is_red
+RefFS：原样重发成功当成写了之后报错（两可）	crates/singlefs-reference-filesystem/src/lib.rs	OperationOutcome::SucceededOnAVerbatimResendAfterFailingWhileWriting => {\n                EnteredOutcome::Succeeded	OperationOutcome::SucceededOnAVerbatimResendAfterFailingWhileWriting => {\n                EnteredOutcome::FailedAfterWriting	-p singlefs-reference-filesystem --lib	a_publish_resent_verbatim_after_failing_while_writing_raises_the_lower_bound_and_is_a_version
+RefFS：起点不算第 1 版	crates/singlefs-reference-filesystem/src/lib.rs	versions_up_to_the_last_recovery: vec![VisibleState::NoFile],	versions_up_to_the_last_recovery: Vec::new(),	-p singlefs-reference-filesystem --lib	a_freshly_formatted_pool_is_version_one_and_the_first_confirmed_publish_is_version_two
+RefFS：恢复读出、重新起头的那一份不算一版	crates/singlefs-reference-filesystem/src/lib.rs	self.versions_up_to_the_last_recovery\n            .push(recovered.clone());	let _ = &recovered;	-p singlefs-reference-filesystem --lib	after_a_recovery_adopts_a_publish_that_never_returned_rolling_back_to_the_version_that_recovery_restarted_from_gives_the_recovered_content
+RefFS：抛弃最新根的那次恢复照样判允许集合	crates/singlefs-reference-filesystem/src/lib.rs	self.allowed_set_for_a_crash_in(crash_falls_in)?;\n        self.restart_from_the_recovered_state(crash_falls_in, recovered);\n        Ok(())	self.crash_and_recover(crash_falls_in, recovered).map(|_| ())	-p singlefs-reference-filesystem --lib	a_recovery_that_abandoned_the_newest_root_is_not_judged_and_the_next_crash_is_judged_from_what_it_read
+RefFS 翻译：崩溃时刻归给写进那一段的那一步（不归关段那道屏障由哪一步发出）	crates/singlefs-harness/src/reference_filesystem_translation.rs	.find(|(_, stream_length)| *stream_length > closing_stream_index)	.find(|(_, stream_length)| *stream_length >= closing_stream_index)	-p singlefs-harness --test reference_filesystem_translation_of_histories	the_crash_moment_is_the_step_that_issued_the_barrier_closing_the_segment_not_the_step_that_wrote_into_it
+RefFS 翻译：冷启动读出的那一份不交 RefFS 重新起头	crates/singlefs-harness/src/reference_filesystem_translation.rs	Some(HistoryOperation::ColdStartRecover),	Some(HistoryOperation::PublishWithoutUnits),	-p singlefs-harness --test reference_filesystem_translation_of_histories	a_rollback_to_a_root_the_mount_wrote_after_a_cold_start_recovery_is_named_by_the_version_that_recovery_restarted_from
+RefFS 翻译：发布写的根映到发布之前那一版	crates/singlefs-harness/src/reference_filesystem_translation.rs	NewVersionWrittenBy::TheLastRootOfTheStep if is_the_last_root_of_the_step => {\n                        version_after_the_step	NewVersionWrittenBy::TheLastRootOfTheStep if is_the_last_root_of_the_step => {\n                        version_before_the_step	-p singlefs-harness --test reference_filesystem_translation_of_histories	a_crash_after_a_returned_rollback_that_reads_the_rolled_back_content_is_green
+RefFS 翻译：二次崩溃不判（一律绿）	crates/singlefs-harness/src/reference_filesystem_translation.rs	judge_the_recovery(restarted.clone(), crash, read_back).vote	{\n        let _ = (restarted, crash, read_back);\n        ReferenceFileSystemVote::Green\n    }	-p singlefs-harness --test reference_filesystem_translation_of_histories	a_second_crash_inside_the_mount_that_reads_back_an_older_version_than_the_first_recovery_is_red
+checker ② 借槽距删掉（槽 0 无效的盘落到 ③ 逐档）	crates/singlefs-checker/src/image.rs	                .or(spacing_recorded_by_the_first_valid_slot_zero)	                .or(None)	-p singlefs-harness --test checker_narrow_invariants_and_abandoned_roots	a_device_whose_slot_zero_does_not_verify_takes_slot_one_only_at_the_borrowed_spacing_not_a_newer_slot_at_another_spacing
+故障注入已知红（用户 2026-10-03 定 I-8.5 照判红）：判法不认「mkfs 在系统配置槽落盘之前报错、只红 I-8.5、重开走读失败」这一类，那一次落成新发现	crates/singlefs-harness/src/fault_injection.rs	    (make_filesystem_returned_the_error\n        && no_system_configuration_slot_landed	    (false\n        && make_filesystem_returned_the_error\n        && no_system_configuration_slot_landed	-p singlefs-harness --test fault_injection_fast_tier -- make_filesystem_failing_at_its_first_barrier	make_filesystem_failing_at_its_first_barrier_before_any_system_configuration_slot_lands_ends_known_red_not_a_new_finding
+故障注入已知红 mkfs 那一类：不看录制流里系统配置槽写落没落盘，mkfs 写完系统配置之后才报错的只红 I-8.5 也认成已知红	crates/singlefs-harness/src/fault_injection.rs	facts.system_configuration_slot_writes_that_landed == 0;	facts.system_configuration_slot_writes_that_landed < usize::MAX;	-p singlefs-harness --lib -- fault_injection::tests::	i85_only_failure_after_make_filesystem_wrote_its_system_configuration_slots_stays_a_new_finding
+故障注入已知红 mkfs 那一类：起点段哪一步报错都认（取号、暖机、第一个文件报错时 mkfs 已经做完）	crates/singlefs-harness/src/fault_injection.rs	            | StartingPointStep::PublishFirstFile,\n        )\n        | None => false,	            | StartingPointStep::PublishFirstFile,\n        ) => true,\n        None => false,	-p singlefs-harness --lib -- fault_injection::tests::	make_filesystem_known_red_form_takes_no_other_step_signature_or_reopen
+故障注入已知红 mkfs 那一类：重开走到哪一格都认（不要求走读失败、盘上本来就没有一版立得住）	crates/singlefs-harness/src/fault_injection.rs	        | ReopenedVersion::OutsideEverythingTheModelAllows => false,	        | ReopenedVersion::OutsideEverythingTheModelAllows => true,	-p singlefs-harness --lib -- fault_injection::tests::	make_filesystem_known_red_form_takes_no_other_step_signature_or_reopen
+故障注入已知红 mkfs 那一类：I-8.5 之外还红别的也认（签名只要第一条是 I-8.5）	crates/singlefs-harness/src/fault_injection.rs	[(invariant, _)] if *invariant == SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION	[(invariant, _), ..] if *invariant == SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION	-p singlefs-harness --lib -- fault_injection::tests::	make_filesystem_known_red_form_takes_no_other_step_signature_or_reopen
+故障注入已知红 mkfs 那一类：带着模型对不上也认	crates/singlefs-harness/src/fault_injection.rs	        && observation.model_disagreement.is_none()\n        && crate::history::record_check_aspects	        && crate::history::record_check_aspects	-p singlefs-harness --lib -- fault_injection::tests::	make_filesystem_known_red_form_takes_no_other_step_signature_or_reopen
+故障注入已知红 mkfs 那一类的接线：数落盘的系统配置槽写时数成了别的写（mkfs 第一道屏障之前落过清零之外的写，那一次判不成已知红）	crates/singlefs-harness/src/fault_injection.rs	.filter(|write| write.kind == StepKind::SystemConfigurationSlot)	.filter(|write| write.kind != StepKind::SystemConfigurationSlot)	-p singlefs-harness --test fault_injection_fast_tier -- make_filesystem_failing_at_its_first_barrier	make_filesystem_failing_at_its_first_barrier_before_any_system_configuration_slot_lands_ends_known_red_not_a_new_finding
+故障注入已知红 mkfs 那一类的接线：不把起点段报错的是哪一步交给判法	crates/singlefs-harness/src/fault_injection.rs	Some(HarnessJudgement::TheStartingPointReturnedAnError { step, .. }) => Some(*step),	Some(HarnessJudgement::TheStartingPointReturnedAnError { step, .. }) => {\n                    let _ = step;\n                    None\n                }	-p singlefs-harness --test fault_injection_fast_tier -- make_filesystem_failing_at_its_first_barrier	make_filesystem_failing_at_its_first_barrier_before_any_system_configuration_slot_lands_ends_known_red_not_a_new_finding
+E170 导出装置第 1 次重跑的 P10 底态取 Mk 而不是 Mk+（登记第九节 XE4；S5b 该停不停）	crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs	BadTableVersion::SecondRun => middle_plus.clone(),	BadTableVersion::SecondRun => Some(middle.clone()),	-p singlefs-checker-tier --bin e170_o3_spec_executor_export	the_second_run_table_differs_from_the_first_only_in_the_p8_expectation_and_a_final_p10_row_on_the_p9_base
+E171 装置 O1-崩溃判官的结果丢掉（登记第九节 X4；阳性对照 O1-崩溃判官该红不红）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs	    match crash_recovery_disagreement(committed_versions, newest_persisted_root_key, read_back) {	    match crash_recovery_disagreement(committed_versions, newest_persisted_root_key, read_back).filter(|_| false) {	-p singlefs-checker-tier --bin e171_three_oracle_collusion	a_read_back_of_a_version_the_model_never_committed_is_red_and_a_committed_one_is_green
+E171 装置 O1-对拍判出的不同丢掉（登记第九节 X5；阳性对照 O1-对拍该红不红）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs	            answer(&self.model).and_then(|answer| self.model.judge_and_advance(&answer, &observed));	            answer(&self.model).and_then(|answer| self.model.judge_and_advance(&answer, &observed).or_else(|_| Ok(Default::default())));	-p singlefs-checker-tier --bin e171_three_oracle_collusion	an_implementation_refusal_the_model_does_not_allow_is_recorded_as_a_disagreement
+E171 装置 O2 把 Violated 当 Holds（登记第九节 X6；阳性对照 O2 该红不红）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs	    verdicts\n        .iter()\n        .filter(|(_, verdict)| match verdict {\n            InvariantVerdict::Violated(_) => true,	    verdicts\n        .iter()\n        .filter(|(_, verdict)| match verdict {\n            InvariantVerdict::Violated(_) => false,	-p singlefs-checker-tier --bin e171_three_oracle_collusion	only_a_violated_invariant_counts_as_a_red_pool_checker_verdict
+E171 装置批 1 漏掉全持久态（登记第九节 X13；停机 S2 states.tsv 与 E170 导出装置的不再逐字节相同）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs	    first.insert(state_count - 1);	    let _ = state_count;	-p singlefs-checker-tier --bin e171_three_oracle_collusion	sampled_batches_are_disjoint_and_include_the_last_state_and_up_to_four_torn_states
+E171 装置 O1-崩溃判官每个触达了的态都记不适用（登记第九节 X14；第七节末尾两份互比的适用态数不等，停机 S5）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs	    !touched_call.is_some_and(|call| model_stopped_at_call.is_some_and(|stopped| call >= stopped))	    !touched_call.is_some_and(|_| true)	-p singlefs-checker-tier --bin e171_three_oracle_collusion	the_crash_judge_is_not_applicable_only_from_the_call_where_the_model_stopped
+E171 装置已知坏镜像族的补丁只取盘 0 的扇区差（登记第九节 X15；停机 S4）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs	    for (device, bad_sparse) in &bad.devices {	    for (device, bad_sparse) in bad.devices.iter().filter(|(device, _)| device.0 == 0) {	-p singlefs-checker-tier --bin e171_three_oracle_collusion	the_sector_difference_between_two_images_carries_the_changed_sectors_of_every_device
+E165 r4 第一段 可写挂载读到的更新状态读不出那道闸恒假（KF M = 2、U = 1 历史数 9401 变 10517）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	            if newer_publish_is_witnessed(world, entry.first_read_unreadable, landed) {	            if false && newer_publish_is_witnessed(world, entry.first_read_unreadable, landed) {	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_generator_counts_of_the_sampling_cells_equal_the_pinned_values
+E165 r4 第一段 T1 当 TP（重读那一遍仍读不出；KF M = 2、U = 1 历史数 9401 变 4125）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	            menu_index: menu.len(),\n            first_read_unreadable: locations,\n            every_read_unreadable: FaultLocations::NOWHERE,\n        });	            menu_index: menu.len(),\n            first_read_unreadable: locations,\n            every_read_unreadable: locations,\n        });	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_generator_counts_of_the_sampling_cells_equal_the_pinned_values
+E165 r4 第一段 闸开时写行 txg 只看根、不看记录（KF M = 2、U = 1 历史数 9401 变 9761）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	            highest_root_txg.max(highest_record_txg) + 1\n	            highest_root_txg + 1\n	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_generator_counts_of_the_sampling_cells_equal_the_pinned_values
+E165 r4 第一段 系统配置写不枚举撕裂第三态（KF M = 2、U = 0 历史数 1792 变 1662）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	        SystemWriteOutcome::Written,\n        SystemWriteOutcome::Torn,\n    ];	        SystemWriteOutcome::Written,\n    ];	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_generator_counts_of_the_sampling_cells_equal_the_pinned_values
+E165 r4 第一段 最后一次挂载不并系统配置那 9 个组合（KF M = 1、U = 0 历史数 10 变 26）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	    if is_the_last_mount {\n        crash_points.push(FaultCrashPoint::RootLanded {	    if false {\n        crash_points.push(FaultCrashPoint::RootLanded {	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_generator_counts_of_the_sampling_cells_equal_the_pinned_values
+E165 r4 第一段 c_见证 = 0 时判真（KF M = 2、U = 0 历史数 1792 变 13）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	    if largest_tail == 0 {\n        return false;\n    }	    if largest_tail == 0 {\n        return true;\n    }	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_generator_counts_of_the_sampling_cells_equal_the_pinned_values
+E165 r4 第一段 已确认把崩着的那一次用户发布也算进去（KF M = 1、U = 1 有已确认的被判态 0 变 195）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	        world.final_crash = Some(FaultCrashLabel {	        if material.is_user_publish {\n            world.confirmed_user_publishes.push(material.version);\n        }\n        world.final_crash = Some(FaultCrashLabel {	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_generator_counts_of_the_sampling_cells_equal_the_pinned_values
+E165 r4 第一段 一次挂载第一次发布的父取最后一次已确认那一版、不取往下走的那一版（KG M = 2、U = 3 的 KF4 KJ1 与 KJ1T 不再分开）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	            let mut previous_version = landed;	            let mut previous_version = world\n                .confirmed_user_publishes\n                .iter()\n                .copied()\n                .max()\n                .unwrap_or(landed);	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_confirmed_chain_parts_kj1t_from_kj1_when_a_mount_goes_down_from_an_older_version_on_the_gate_off_domain
+E165 r4 第一段 历史里 RI 算不罩（KF M = 2、U = 1 历史里有不罩故障的历史 1218 变 2626）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	const RNSB_FIRST_MENU_INDEX: usize = 11;	const RNSB_FIRST_MENU_INDEX: usize = 9;	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_generator_counts_of_the_sampling_cells_equal_the_pinned_values
+E165 r4 第一段 被判时盘不在让两块盘的记录都读不出（KF M = 1、U = 2 罩着而条款的理想恢复照样丢 0 变 108）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	        let copies_on_devices = [\n            landed.copies_on_devices[0] && !locations.device_is_dead(0),\n            landed.copies_on_devices[1] && !locations.device_is_dead(1),\n        ];	        let any_device_is_dead = locations.device_is_dead(0) || locations.device_is_dead(1);\n        let copies_on_devices = [\n            landed.copies_on_devices[0] && !any_device_is_dead,\n            landed.copies_on_devices[1] && !any_device_is_dead,\n        ];	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_generator_counts_of_the_sampling_cells_equal_the_pinned_values
+E165 r4 第一段 KJ1T 写成按版本序比（与 KJ1 同；KG M = 2、U = 3 的 KF4 两列不再分开）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	            .map(|last| !world.is_the_version_or_a_descendant_of(landed_version, last)),	            .map(|last| landed_version < last),	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_confirmed_chain_parts_kj1t_from_kj1_when_a_mount_goes_down_from_an_older_version_on_the_gate_off_domain
+E165 r4 第一段 KJ2 比真相里的根、不比读视图里读得出的根（KF2 的 KJ2 在 KF M = 1、U = 2 一格 252 变 585）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	            newest_readable: newest_root.map(|newest| newest.version),	            newest_readable: root_choice_newest_root(&fault_landing_of_a_view(\n                world,\n                FaultLocations::NOWHERE,\n            ))\n            .map(|newest| newest.version),	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_generator_counts_of_the_sampling_cells_equal_the_pinned_values
+E165 r4 第一段 RN 落在键最小的根上（KF M = 2、U = 1 历史数 9401 变 9667）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	                if newest.is_none_or(|(_index, version)| material.version > version) {	                if newest.is_none_or(|(_index, version)| material.version < version) {	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_generator_counts_of_the_sampling_cells_equal_the_pinned_values
+E165 r4 第一段 承诺内那一行的违例数不减上限截掉那部分（界内没有取样点，只由手造的格抓）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	        promised += cell.violations[criterion_index] - cell.cut_violations[criterion_index];	        promised += cell.violations[criterion_index];	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_promised_rows_subtract_the_states_the_in_flight_limit_cut
+E165 r4 第一段 读槽对拍不看被判时故障（读不出的槽当成真相里的根比）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	    if is_unreadable_in_this_view {	    if false {	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	a_root_slot_reading_of_a_slot_unreadable_in_the_view_agrees_with_the_truth_only_as_unreadable
+E165 r4 第一段 阳性对照里删掉 KF1 下的 KJ1T 那一行（KJ1T 的对照读数条数少一个格数）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	        (FaultCriterion::Kj1tDescendsFromTheLastConfirmedPublish, 1),\n        (FaultCriterion::Kj2NotEarlierThanAnyReadableRoot, 2),\n    ] {	        (FaultCriterion::Kj2NotEarlierThanAnyReadableRoot, 2),\n    ] {	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	every_fault_criterion_has_a_positive_control_and_every_control_is_red
+E165 r4 第一段 取号之后走了重读那一遍的挂载不记数（KF M = 2、U = 1 走了重读的挂载 5598 变 0）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	        if outcome == MountOutcome::WentDownOnTheSecondRead {\n            acquired.reread_mounts += 1;\n        }	        if outcome == MountOutcome::WentDownOnTheSecondRead {\n            acquired.reread_mounts += 0;\n        }	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_generator_counts_of_the_sampling_cells_equal_the_pinned_values
+E165 r4 第一段 RI 在最新实例是 mkfs 第 0 代时也摆落点（第 0 代根全读不出，往下走那一遍读视图里没有根）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	        if newest_instance == ROOT_CHOICE_MKFS_INSTANCE {\n            return locations;\n        }	        if false {\n            return locations;\n        }	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_generator_counts_of_the_sampling_cells_equal_the_pinned_values
+K3-1：checker 逐档读到可用的槽不先判 I-8.1 三个字段（在飞上限 + 1、自述槽距 8192 的槽在 12288 处被当成不在自己槽距上跳过）	crates/singlefs-checker/src/image.rs	                    journal_ring_geometry_field_that_disagrees(&geometry)	                    journal_ring_geometry_field_that_disagrees(&geometry).filter(|_| false)	-p singlefs-harness --test system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery -- a_slot_whose_in_flight_limit_is_off_by_one	a_slot_whose_in_flight_limit_is_off_by_one_and_whose_spacing_is_not_its_offset_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery
+K3-1：checker 逐档读到可用的槽不先判 I-8.1 三个字段（在飞上限 + 1、自述槽距等于偏移、世代号 0 的槽比世代号输给 8192 处的 1）	crates/singlefs-checker/src/image.rs	                    journal_ring_geometry_field_that_disagrees(&geometry)	                    journal_ring_geometry_field_that_disagrees(&geometry).filter(|_| false)	-p singlefs-harness --test system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery -- a_slot_whose_in_flight_limit_is_off_by_one	a_slot_whose_in_flight_limit_is_off_by_one_and_whose_generation_is_lower_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery
+K4-1：checker 槽 0 无效的盘不借别的盘槽 0 记的槽距、落到逐档（盘 0 择 8192 处世代号 5 的旧槽）	crates/singlefs-checker/src/image.rs	                .or(spacing_recorded_by_the_first_valid_slot_zero)	                .or(spacing_recorded_by_the_first_valid_slot_zero.filter(|_| false))	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search	a_device_whose_slot_zero_fails_borrows_the_spacing_another_slot_zero_records_instead_of_searching
+辩方 D3：checker 择最新根键（txg, 实例）相同时取读根环次序里后一条（与 choose_root 相反）	crates/singlefs-checker/src/walk.rs	                    Some((_, newest_key)) if choice_key <= newest_key => newest_so_far,	                    Some((_, newest_key)) if choice_key < newest_key => newest_so_far,	-p singlefs-harness --test root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance	of_two_roots_with_the_same_choice_key_the_pool_checker_walks_the_root_in_the_earlier_slot_as_choose_root_does
+RefFS 翻译（13-4）：故障注入的读故障档退回照判	crates/singlefs-harness/src/reference_filesystem_translation.rs	        InjectedFault::ReadFails => true,	        InjectedFault::ReadFails => false,	-p singlefs-harness --test reference_filesystem_translation_of_histories	the_reopen_after_a_failing_read_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish
+RefFS 翻译（13-4）：故障注入每一档都照判（带故障的状态不弃权）	crates/singlefs-harness/src/reference_filesystem_translation.rs	        if the_state_after_the_fault_is_outside_the_reference_filesystem_range(fault) {	        if false {	-p singlefs-harness --test reference_filesystem_translation_of_histories	the_reopen_after_a_swallowed_write_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish
+RefFS 翻译（13-4）：故障注入的写报错档退回照判	crates/singlefs-harness/src/reference_filesystem_translation.rs	        InjectedFault::WriteFails => true,	        InjectedFault::WriteFails => false,	-p singlefs-harness --test reference_filesystem_translation_of_histories	the_reopen_after_a_failing_write_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish
+RefFS 翻译（13-4）：故障注入的写被吞档退回照判	crates/singlefs-harness/src/reference_filesystem_translation.rs	        InjectedFault::WriteIsSwallowed => true,	        InjectedFault::WriteIsSwallowed => false,	-p singlefs-harness --test reference_filesystem_translation_of_histories	the_reopen_after_a_swallowed_write_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish
+RefFS 翻译（13-4）：故障注入的读回翻位档退回照判	crates/singlefs-harness/src/reference_filesystem_translation.rs	        InjectedFault::ReadReturnsCorruptedBytes { .. } => true,	        InjectedFault::ReadReturnsCorruptedBytes { .. } => false,	-p singlefs-harness --test reference_filesystem_translation_of_histories	the_reopen_after_a_read_returning_a_flipped_bit_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish
+RefFS 翻译（13-4）：故障注入的刷盘报错档退回照判	crates/singlefs-harness/src/reference_filesystem_translation.rs	        InjectedFault::BarrierFails => true,	        InjectedFault::BarrierFails => false,	-p singlefs-harness --test reference_filesystem_translation_of_histories	the_reopen_after_a_failing_barrier_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish
+RefFS 翻译（13-4）：故障注入的刷盘被吞档退回照判	crates/singlefs-harness/src/reference_filesystem_translation.rs	        InjectedFault::BarrierIsSwallowed => true,	        InjectedFault::BarrierIsSwallowed => false,	-p singlefs-harness --test reference_filesystem_translation_of_histories	the_reopen_after_a_swallowed_barrier_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish
+挂载透明（13-4）：抬 F 落没落从挂载第一个写起算（暖机与抬 F 退回一起算）	crates/singlefs-harness/src/mount_transparency.rs	                    .get(first_write..)	                    .get(..)	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_second_crash_after_the_warm_up_judges_the_floor_unchanged_and_one_after_the_first_floor_raise_write_leaves_it_unjudged
+挂载透明（13-4）：抬 F 的界取到挂载第一个系统配置槽写（不认抬到的 F）	crates/singlefs-harness/src/mount_transparency.rs	                        .is_some_and(|written| written.quantities.rollback_floor == raised_floor)	                        .is_some_and(|written| written.quantities.rollback_floor <= raised_floor)	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_second_crash_after_the_warm_up_judges_the_floor_unchanged_and_one_after_the_first_floor_raise_write_leaves_it_unjudged
+崩溃注入（13-4）：崩溃后镜像上的挂载不交回抬 F 从哪个写起	crates/singlefs-checker-tier/src/crash_injection.rs	        FloorRaisesInTheMountWrites::of_the_mount_output(&mounted.output, &mount_writes)	        FloorRaisesInTheMountWrites::NotPushed	-p singlefs-checker-tier --lib	the_writable_mount_on_a_crash_image_names_the_first_floor_raise_write_after_the_warm_up
````

## `crates/singlefs-checker/src/image.rs`

````diff
--- crates/singlefs-checker/src/image.rs	2026-10-02 15:33:45.022995274 +0000
+++ crates/singlefs-checker/src/image.rs	2026-10-02 18:08:49.852021196 +0000
@@ -445,13 +445,14 @@
     /// 自证过、incompat 位认得、池级字段都在这一版读者收的范围里：可择。
     Usable(SystemConfigurationView, PoolGeometry),
     /// 自证过（magic 与整槽校验和都对）、incompat 位认得、R 与 S 在区间里，而一个池级字段越出这一版读者收的范围
-    /// （[`value_refused_by_this_reader`] 认的那几个 [`crate::Verdict`] 成员）。实现遇到这一槽整池拒绝挂载
+    /// （[`value_refused_by_this_reader`] 认的那几个 [`crate::Verdict`] 成员；逐档读槽 1 时还有 I-8.1 三个字段等式，
+    /// [`RefusedSystemConfigurationValue::JournalRingGeometryFieldDisagrees`]）。实现遇到这一槽整池拒绝挂载
     /// （`singlefs_core::recovery::RecoveryFailure::SystemConfigurationValueRefused`）；checker 同一个处置：报
     /// [`SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS`] 违例、不替这个池作保，不退成「这一槽不可择」静默跳过
     /// （用户 2026-09-27 定系统配置越界整池拒）。
     CarriesAValueThisReaderRefuses {
         offset_in_bytes: u64,
-        refused: crate::Verdict,
+        refused: RefusedSystemConfigurationValue,
     },
     /// 不自证：读不到、不足 4096 字节、magic 或整槽校验和不对（I-8.5（tail 槽有效集非空） 的「自证过」= magic 与整槽校验和都对）。
     NotSelfVerified,
@@ -459,6 +460,68 @@
     SelfVerifiedButNotChoosable,
 }
 
+/// 一个带这一版读者不收之值的系统配置槽（[`SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses`]）拒在哪一格。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum RefusedSystemConfigurationValue {
+    /// 自证与几何那几道（[`crate::check_system_configuration_slot`]、[`geometry_of`]）拒的，是 [`value_refused_by_this_reader`] 认的那几个成员之一。
+    RefusedByTheSlotChecks(crate::Verdict),
+    /// I-8.1（环几何够大） 的三个字段等式里第一个不成立的那一格（[`journal_ring_geometry_field_that_disagrees`]）。只在逐档读槽 1 时这样交回
+    /// （[`slot_one_read_at_every_slot_spacing_the_format_allows`]）：实现那张「读者不收之值」表的最后三格就是这三个等式
+    /// （`singlefs_core::recovery` 的 `system_configuration_values_this_reader_accepts`），逐档读到它整池拒；
+    /// 别的读法上可择的槽照旧由 [`judge_journal_ring_geometry_fields_agree`] 按 I-8.1 判。
+    JournalRingGeometryFieldDisagrees(JournalRingGeometryFieldThatDisagrees),
+}
+
+/// I-8.1（环几何够大） 三个字段等式里的哪一格不成立，次序与实现判这三格的次序相同。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum JournalRingGeometryFieldThatDisagrees {
+    /// 槽里的安全系数 F 不等于格式常量 `JOURNAL_SAFETY_FACTOR`。
+    SafetyFactorNotTheFormatConstant,
+    /// 槽里记的在飞上限不等于 ⌊环长 ÷ 4096 ÷ F⌋。
+    InFlightRecordLimitNotRecomputedFromTheRingLength,
+    /// 槽里记的最坏占用不等于在飞上限 × 4096（乘出来溢出也算不等）。
+    WorstCaseOccupancyNotTheInFlightLimitTimesTheRecordBytes,
+}
+
+/// I-8.1 第二格右边：按同一槽的环长与 F 现算的在飞上限 ⌊环长 ÷ 4096 ÷ F⌋；F 为 0 时没有这个数（None，那一格判不等、不除）。
+const fn in_flight_record_limit_recomputed_from_the_ring(geometry: &PoolGeometry) -> Option<u64> {
+    (geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES)
+        .checked_div(geometry.declared_safety_factor)
+}
+
+/// I-8.1 第三格右边：槽里记的在飞上限 × 4096；乘出来溢出时没有这个数（None，那一格判不等）。
+const fn worst_case_occupancy_recomputed_from_the_limit(geometry: &PoolGeometry) -> Option<u64> {
+    geometry
+        .declared_in_flight_record_limit
+        .checked_mul(JOURNAL_RECORD_BYTES)
+}
+
+/// I-8.1（环几何够大） 的三个字段等式按 F、在飞上限、最坏占用的次序判，交回第一个不成立的；三格都成立交回 None。
+/// 三格与 [`judge_journal_ring_geometry_fields_agree`] 判的是同三个等式（后两格的右边共用上面两个算式）。
+#[must_use]
+pub fn journal_ring_geometry_field_that_disagrees(
+    geometry: &PoolGeometry,
+) -> Option<JournalRingGeometryFieldThatDisagrees> {
+    if geometry.declared_safety_factor != JOURNAL_SAFETY_FACTOR {
+        return Some(JournalRingGeometryFieldThatDisagrees::SafetyFactorNotTheFormatConstant);
+    }
+    if in_flight_record_limit_recomputed_from_the_ring(geometry)
+        != Some(geometry.declared_in_flight_record_limit)
+    {
+        return Some(
+            JournalRingGeometryFieldThatDisagrees::InFlightRecordLimitNotRecomputedFromTheRingLength,
+        );
+    }
+    if worst_case_occupancy_recomputed_from_the_limit(geometry)
+        != Some(geometry.declared_worst_case_occupancy_bytes)
+    {
+        return Some(
+            JournalRingGeometryFieldThatDisagrees::WorstCaseOccupancyNotTheInFlightLimitTimesTheRecordBytes,
+        );
+    }
+    None
+}
+
 /// 一个自证过的系统配置槽被拒的理由是不是「池级字段越出这一版读者收的范围」。穷举写全：新加一个 [`crate::Verdict`] 成员时这里编译不过，
 /// 逼着判它归哪一边。
 #[must_use]
@@ -542,7 +605,7 @@
         if value_refused_by_this_reader(&verdict) {
             SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
                 offset_in_bytes,
-                refused: verdict,
+                refused: RefusedSystemConfigurationValue::RefusedByTheSlotChecks(verdict),
             }
         } else if verdict_says_the_slot_does_not_verify_itself(&verdict) {
             SystemConfigurationSlotReading::NotSelfVerified
@@ -578,6 +641,10 @@
 /// 读到的某一档带这一版读者不收的值（[`SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses`]）：交回按偏移升序读到的第一个，
 /// 由 I-7.13（系统配置池级字段在读者收的范围里） 报违例、整池不作保——D22（单元原子性怎么合成） 已定项 16 第 5 句「越界即拒」，
 /// 与槽 0、槽 1 一个处置；它记的槽距可能正是越界的那一格，不核偏移。
+/// 读到可用的一档，先按实现那张「读者不收之值」表剩下的三格判（I-8.1（环几何够大） 三个字段等式，[`journal_ring_geometry_field_that_disagrees`]），
+/// 有一格不成立就同样当场交回带不收之值，不再核偏移、比世代号：已定项 16 定案「逐档读到自证过而池级字段越界的槽整池拒」不要求
+/// 槽距等于偏移，实现照字面先判值后核偏移（`singlefs_core::recovery` 的 `slot_one_found_by_trying_every_slot_spacing_the_format_allows`；
+/// 代码轮第二轮判决 K3-1，`research/prompts/m3-formal-code-r2-main-verification.md`）。
 /// 一档可用的都没收到时，有自证过而不可择的就交回 [`SystemConfigurationSlotReading::SelfVerifiedButNotChoosable`]
 /// （I-8.5（tail 槽有效集非空） 数它；它记的槽距可能读不出来，不核偏移），再没有就是 [`SystemConfigurationSlotReading::NotSelfVerified`]。
 fn slot_one_read_at_every_slot_spacing_the_format_allows(
@@ -594,6 +661,16 @@
     {
         match read_system_configuration_slot(reader, device, candidate_spacing) {
             SystemConfigurationSlotReading::Usable(view, geometry) => {
+                if let Some(disagreeing_field) =
+                    journal_ring_geometry_field_that_disagrees(&geometry)
+                {
+                    return SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
+                        offset_in_bytes: candidate_spacing,
+                        refused: RefusedSystemConfigurationValue::JournalRingGeometryFieldDisagrees(
+                            disagreeing_field,
+                        ),
+                    };
+                }
                 if geometry.slot_spacing != candidate_spacing {
                     continue;
                 }
@@ -909,7 +986,7 @@
                 },
             );
             let in_flight_record_limit_from_the_ring =
-                (geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES).checked_div(safety_factor);
+                in_flight_record_limit_recomputed_from_the_ring(geometry);
             judgements.judge(
                 JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
                 in_flight_record_limit_from_the_ring
@@ -921,9 +998,8 @@
                     )
                 },
             );
-            let worst_case_occupancy_from_the_limit = geometry
-                .declared_in_flight_record_limit
-                .checked_mul(JOURNAL_RECORD_BYTES);
+            let worst_case_occupancy_from_the_limit =
+                worst_case_occupancy_recomputed_from_the_limit(geometry);
             judgements.judge(
                 JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
                 worst_case_occupancy_from_the_limit
````

## `crates/singlefs-checker/src/walk.rs`

````diff
--- crates/singlefs-checker/src/walk.rs	2026-10-02 15:33:45.023041684 +0000
+++ crates/singlefs-checker/src/walk.rs	2026-10-02 18:08:49.852021196 +0000
@@ -6708,11 +6708,22 @@
         return root_ring_judgements.into_report();
     }
     judge_root_ring_health(&roots, &mut root_ring_judgements);
-    // 择根：按 (txg, 实例) 取最大（D22（单元原子性怎么合成） 已定项 7，与实现 `recovery::choose_root` 同一个读法）。
+    // 择根：按 (txg, 实例) 取最大（D22（单元原子性怎么合成） 已定项 7，与实现 `recovery::choose_root` 同一个读法）；
+    // 最大的不止一条时取读根环次序里最前的那条（`roots` 按 `root_slot_positions` 的区域、槽次序排，与 `recovery::every_root_ring_slot` 同序）：
+    // 只在严格更大时才换。`max_by_key` 在键相同时交回最后一个，与 `choose_root` 相反（代码轮第二轮判决辩方 D3）。
     let newest_index = roots
         .iter()
         .enumerate()
-        .max_by_key(|(_, (_, _, root))| (root.checkpoint_txg, root.instance))
+        .fold(
+            None,
+            |newest_so_far: Option<(usize, (u64, u32))>, (index, (_, _, root))| {
+                let choice_key = (root.checkpoint_txg, root.instance);
+                match newest_so_far {
+                    Some((_, newest_key)) if choice_key <= newest_key => newest_so_far,
+                    Some(_) | None => Some((index, choice_key)),
+                }
+            },
+        )
         .map(|(index, _)| index)
         .expect("上面判过根环里至少一条自证过的根");
     let mut walk = Walk::starting_with(
````

## `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`

````diff
--- crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	2026-10-02 15:00:53.587413423 +0000
+++ crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	2026-10-02 20:54:13.184893031 +0000
@@ -10,7 +10,7 @@
 //! `--only <域>`（可写几个，逗号分隔）、`--max-n <N>`、`--family legal|adversarial|all` 只跑一部分。
 //! 多盘域 M、M6、MC（登记 `research/prompts/e165-r2-prereg.md` 5.3，第三段）只在 `--profile r2 --family multi_disk` 时跑：每条记录在 0、1 号盘
 //! 同一落点各一份、各自落或没落，「完整在盘上」取任一份，直接建表先 0 号盘再 1 号盘；`--family all` 仍只含合法历史与对抗两族。
-//! 择根与重放合判一族 K、K9、KW、KA（登记 `research/prompts/e165-r3-prereg.md` 5.3 至 5.5）只在 `--profile r3 --family root_choice` 时跑：真 `choose_root`、`scan_journal`、`replay_journal` 三步串起来，判据 KJ0 至 KJ3 各报各的；`--family all` 不含它。
+//! 择根与重放合判一族 K、K9、KW、KA（登记 `research/prompts/e165-r3-prereg.md` 5.3 至 5.5）只在 `--profile r3 --family root_choice` 时跑：真 `choose_root`、`scan_journal`、`replay_journal` 三步串起来，判据 KJ0 至 KJ3 各报各的；`--family all` 不含它。带读故障的择根合判一族 KF、KF3、KF9、KFW、KG、KA（登记 `research/prompts/e165-r4-prereg.md` 5.3 至 5.6）只在 `--profile r4 --family root_choice_faults` 时跑：同样调真三步，多出每块盘两个系统配置槽、挂载开头的读故障、可写挂载读到的更新状态读不出那道闸（实现的取甲）与被判时的读故障，判据加 KJ1T，`--profile r4` 的其余族与 r3 档相同。
 //! 线程数取环境变量 `E165_THREADS`，没设取 `SINGLEFS_THREAD_CAP`，都没设取 `available_parallelism`；计数与线程数无关。
 //! 产物是 `E7RESULT` 行，收尾 `name=done emitted=N`；耗时另打 `E7TIMING` 行，不进逐字比对。
 //! `light` 档的界：L 到 9、U 到 5、A 到 3，其余同第一段（2026-10-01 在 10 线程上量到每态挂钟 0.24–1.2 微秒，每个域取到 3 秒之内）。
@@ -8158,7 +8158,7 @@
         RunProfile::r2(MarkerParity::EvenVersionsWrittenByTheUnmountSequence),
         "r3 档与 r2 档是两个档"
     );
-    assert!(split(&["--profile", "r4"]).is_err());
+    assert!(split(&["--profile", "r5"]).is_err());
     assert!(split(&["--profile"]).is_err());
     assert!(split(&["--profile", "r2", "--profile", "r1"]).is_err());
 }
@@ -12118,6 +12118,3711 @@
     }
 }
 
+// ── 带读故障的择根合判一族（登记 `research/prompts/e165-r4-prereg.md` 5.3 至 5.6、第七节：KF、KF3、KF9、KFW、KG、KA；只在 `--profile r4 --family root_choice_faults` 时跑）
+//
+// 被测对象与 r3 的择根合判一族相同：`recover` 的 `Consult` 支里串起来的三步（`choose_root`、`scan_journal`、`replay_journal(…, true)`）。
+// 这一族多出来的东西全在生成器一侧（可信件，按登记 5.3 与 D16 已定项 7、8，D22 已定项 7、8、16，D23 已定项 14、18 的字面写，不调 `mount.rs`、`transaction.rs`）：
+// 每块盘两个系统配置槽、挂载开头的读故障（根槽、系统配置槽的一时读不出）、可写挂载读到的更新状态读不出那道闸（实现的取甲，登记修订 1）、取号那一写与发布末尾的系统配置轮换。
+// 被判的那一次恢复另有 5 种读故障（无、RN、RI、D0、D1）；故障只落在读者一侧，盘上的字节不变。
+
+const FAULT_FAMILY_LABEL: &str = "root_choice_faults";
+/// 每块盘的系统配置槽数、mkfs 写下的世代号、可写挂载的设备数下限：本地常量，抄自 D22 已定项 16「每盘恒 2 个槽……槽世代号从 1 起」与 D2 已定项 13
+/// 「可写设备数 ≥ w 的下限」（w = 2）；[`cross_check_root_choice_fault_local_constants_against_the_implementation`] 逐个回比。
+const FAULT_SYSTEM_SLOTS_PER_DEVICE: usize = 2;
+const FAULT_MKFS_SYSTEM_GENERATION: u64 = 1;
+const FAULT_WRITABLE_DEVICE_COUNT_LOWER_BOUND: usize = 2;
+const FAULT_DEVICE_COUNT: usize = 2;
+/// 整池的系统配置槽数：可写挂载读到的自证过的槽不足这个数就判不出（实现的取甲，登记修订 1）。
+const FAULT_SYSTEM_SLOT_COUNT_OF_THE_POOL: usize =
+    FAULT_DEVICE_COUNT * FAULT_SYSTEM_SLOTS_PER_DEVICE;
+/// 去掉在飞上限的理想恢复用（登记 5.5「上限截掉」：去掉上限之后成立）。
+const FAULT_IN_FLIGHT_LIMIT_REMOVED: usize = 1_000_000_000;
+/// 挂载开头故障菜单的项数（登记 5.3 第 5 条：无，RN、SN0、SN1、SB、RI、RNSB 六种落点各配 T1 / TP 两种时机）。
+const MOUNT_FAULT_MENU_LENGTH: usize = 13;
+const MOUNT_FAULT_MENU_LABELS: [&str; MOUNT_FAULT_MENU_LENGTH] = [
+    "none", "RN-T1", "RN-TP", "SN0-T1", "SN0-TP", "SN1-T1", "SN1-TP", "SB-T1", "SB-TP", "RI-T1",
+    "RI-TP", "RNSB-T1", "RNSB-TP",
+];
+
+labelled_enumeration! {
+    /// 登记 5.4 的六个域：KF、KF3、KF9、KFW 是带读故障的合法历史，KG 是把条款的两道保护拿掉的对照，KA 是 r3 原样的对抗根环。
+    enum FaultDomain {
+        GateOn => "KF",
+        GateOnThreeMounts => "KF3",
+        GateOnSmallRing => "KF9",
+        GateOnWrappingRootRing => "KFW",
+        GateOff => "KG",
+        AdversarialRootRing => "KA",
+    }
+}
+
+impl FaultDomain {
+    /// 这个域的几何（环槽数、每区槽数）与 r3 的哪个域相同。
+    const fn geometry(self) -> RootChoiceDomain {
+        match self {
+            Self::GateOn | Self::GateOnThreeMounts | Self::GateOff => RootChoiceDomain::BigRing,
+            Self::GateOnSmallRing => RootChoiceDomain::SmallRingNine,
+            Self::GateOnWrappingRootRing => RootChoiceDomain::WrappingRootRing,
+            Self::AdversarialRootRing => RootChoiceDomain::AdversarialRootRing,
+        }
+    }
+
+    /// 用户发布每次的记录条数（登记 5.4：KF、KF9、KG 取 {1, 2}，KF3、KFW 取 {1}）。
+    const fn user_publish_record_counts(self) -> &'static [usize] {
+        match self {
+            Self::GateOn | Self::GateOnSmallRing | Self::GateOff => &[1, 2],
+            Self::GateOnThreeMounts | Self::GateOnWrappingRootRing => &[1],
+            Self::AdversarialRootRing => &[],
+        }
+    }
+
+    /// 条款的两道保护开着：可写挂载判 N-配置（实现的取甲）与取号那一刻、写行 txg 看记录（登记 5.3 第 6 条）。KG 关掉。
+    const fn gate_is_on(self) -> bool {
+        match self {
+            Self::GateOn
+            | Self::GateOnThreeMounts
+            | Self::GateOnSmallRing
+            | Self::GateOnWrappingRootRing => true,
+            Self::GateOff | Self::AdversarialRootRing => false,
+        }
+    }
+
+    /// 合法历史：Q1、Q2 的域。KG 与 KA 不是。
+    const fn is_a_legal_history(self) -> bool {
+        self.gate_is_on()
+    }
+
+    const fn has_histories(self) -> bool {
+        match self {
+            Self::AdversarialRootRing => false,
+            Self::GateOn
+            | Self::GateOnThreeMounts
+            | Self::GateOnSmallRing
+            | Self::GateOnWrappingRootRing
+            | Self::GateOff => true,
+        }
+    }
+
+    /// 全界（登记 5.4：KF M ≤ 2、U ≤ 3；KF3 M ≤ 3、U ≤ 3；KF9 M ≤ 2、U ≤ 2；KFW M = 1、U ≤ 14；KG M ≤ 2、U ≤ 3）与轻档（调试用，不是登记的界）。
+    const fn largest_mounts(self, tier: BoundsTier) -> usize {
+        match (self, tier) {
+            (Self::AdversarialRootRing, _) => 0,
+            (Self::GateOnThreeMounts, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 3,
+            (
+                Self::GateOn | Self::GateOnSmallRing | Self::GateOff,
+                BoundsTier::SegmentOne | BoundsTier::SegmentTwo,
+            ) => 2,
+            (Self::GateOnWrappingRootRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 1,
+            (
+                Self::GateOn | Self::GateOnThreeMounts | Self::GateOnSmallRing | Self::GateOff,
+                BoundsTier::Light,
+            ) => 2,
+            (Self::GateOnWrappingRootRing, BoundsTier::Light) => 1,
+        }
+    }
+
+    const fn largest_user_publishes(self, tier: BoundsTier) -> usize {
+        match (self, tier) {
+            (Self::AdversarialRootRing, _) => 0,
+            (
+                Self::GateOn | Self::GateOnThreeMounts | Self::GateOff,
+                BoundsTier::SegmentOne | BoundsTier::SegmentTwo,
+            ) => 3,
+            (Self::GateOnSmallRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 2,
+            (Self::GateOnWrappingRootRing, BoundsTier::SegmentOne | BoundsTier::SegmentTwo) => 14,
+            (
+                Self::GateOn | Self::GateOnThreeMounts | Self::GateOnSmallRing | Self::GateOff,
+                BoundsTier::Light,
+            ) => 1,
+            (Self::GateOnWrappingRootRing, BoundsTier::Light) => 4,
+        }
+    }
+}
+
+/// 一个域在这一档、这一次命令行下要跑的格，按 (M 升序, U 升序)。`largest_user_publishes_cap` 是 `--max-u`（只给调试用）。
+fn fault_cells_of(
+    domain: FaultDomain,
+    tier: BoundsTier,
+    largest_user_publishes_cap: Option<usize>,
+) -> Vec<RootChoiceCell> {
+    if !domain.has_histories() {
+        return vec![RootChoiceCell {
+            mounts: 0,
+            user_publishes: 0,
+        }];
+    }
+    let largest_user_publishes = largest_user_publishes_cap
+        .map_or(domain.largest_user_publishes(tier), |cap| {
+            cap.min(domain.largest_user_publishes(tier))
+        });
+    let mut cells = Vec::new();
+    for mounts in 1..=domain.largest_mounts(tier) {
+        for user_publishes in 0..=largest_user_publishes {
+            cells.push(RootChoiceCell {
+                mounts,
+                user_publishes,
+            });
+        }
+    }
+    cells
+}
+
+// ── 读故障的落点（登记 5.3 第 5 条）：根槽、系统配置槽、整块盘（盘不在）；journal 记录槽不单独加读故障，盘不在时那块盘上的记录拷贝随之读不出
+
+#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
+struct FaultLocations {
+    /// 位 = 根槽下标（区域号主序）。
+    root_slot_bits: u32,
+    /// 位 = 盘号 × 每盘槽数 + 槽号。
+    system_slot_bits: u8,
+    /// 位 = 盘号。
+    dead_device_bits: u8,
+}
+
+impl FaultLocations {
+    const NOWHERE: Self = Self {
+        root_slot_bits: 0,
+        system_slot_bits: 0,
+        dead_device_bits: 0,
+    };
+
+    fn with_root_slot(self, root_slot_index: usize) -> Self {
+        Self {
+            root_slot_bits: self.root_slot_bits | (1 << root_slot_index),
+            ..self
+        }
+    }
+
+    fn with_system_slot(self, device: usize, slot: usize) -> Self {
+        Self {
+            system_slot_bits: self.system_slot_bits
+                | (1 << (device * FAULT_SYSTEM_SLOTS_PER_DEVICE + slot)),
+            ..self
+        }
+    }
+
+    fn with_dead_device(self, device: usize) -> Self {
+        Self {
+            dead_device_bits: self.dead_device_bits | (1 << device),
+            ..self
+        }
+    }
+
+    fn union(self, other: Self) -> Self {
+        Self {
+            root_slot_bits: self.root_slot_bits | other.root_slot_bits,
+            system_slot_bits: self.system_slot_bits | other.system_slot_bits,
+            dead_device_bits: self.dead_device_bits | other.dead_device_bits,
+        }
+    }
+
+    fn device_is_dead(self, device: usize) -> bool {
+        (self.dead_device_bits >> device) & 1 == 1
+    }
+
+    /// 这个根槽读不出：它自己在故障里，或它所在的盘不在。
+    fn root_slot_is_unreadable(self, root_slot_index: usize, slots_per_region: u64) -> bool {
+        (self.root_slot_bits >> root_slot_index) & 1 == 1
+            || self.device_is_dead(root_choice_device_of_root_slot(
+                root_slot_index,
+                slots_per_region,
+            ))
+    }
+
+    fn system_slot_is_unreadable(self, device: usize, slot: usize) -> bool {
+        (self.system_slot_bits >> (device * FAULT_SYSTEM_SLOTS_PER_DEVICE + slot)) & 1 == 1
+            || self.device_is_dead(device)
+    }
+}
+
+/// 根槽下标（区域号主序）所在的盘：区域归属 0 / 1 / 0。
+fn root_choice_device_of_root_slot(root_slot_index: usize, slots_per_region: u64) -> usize {
+    let region = root_slot_index / usize::try_from(slots_per_region).expect("每区槽数 4 或 8");
+    usize::try_from(ROOT_CHOICE_REGION_DEVICES[region]).expect("盘号是 0 或 1")
+}
+
+/// 读者收到的 (盘, 偏移) 落在哪个根槽上（区域归属与槽距照 r3 的内存盘）；不是根槽的落点交回空。
+fn root_choice_root_slot_index_at(
+    device: DeviceIdentity,
+    offset: DeviceOffsetInBytes,
+    slots_per_region: u64,
+) -> Option<usize> {
+    let spacing = u64::from(ROOT_CHOICE_FIXED_STRUCTURE_SLOT_SPACING_BYTES);
+    for region in 0..ROOT_CHOICE_REGION_COUNT {
+        let region_start = root_choice_region_start_in_bytes(region);
+        if offset.0 < region_start || offset.0 >= region_start + slots_per_region * spacing {
+            continue;
+        }
+        let offset_in_region = offset.0 - region_start;
+        let region_device =
+            ROOT_CHOICE_REGION_DEVICES[usize::try_from(region).expect("区域号不超过 2")];
+        if !offset_in_region.is_multiple_of(spacing) || device.0 != region_device {
+            return None;
+        }
+        return Some(root_choice_slot_index_of(
+            region,
+            offset_in_region / spacing,
+            slots_per_region,
+        ));
+    }
+    None
+}
+
+// ── 系统配置槽：每块盘两个，每槽 (世代号, tail, 实例代号) 或「撕裂」（新旧都读不出，D13 已定项 4）
+
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum SystemSlotContent {
+    Torn,
+    Verified {
+        generation: u64,
+        tail: u64,
+        instance: u32,
+    },
+}
+
+type SystemSlots = [[SystemSlotContent; FAULT_SYSTEM_SLOTS_PER_DEVICE]; FAULT_DEVICE_COUNT];
+
+/// mkfs 之后：每块盘两槽都是 (世代号 1, tail 0, 实例 0)（登记 5.3 第 2 条）。
+fn system_slots_right_after_mkfs() -> SystemSlots {
+    [[SystemSlotContent::Verified {
+        generation: FAULT_MKFS_SYSTEM_GENERATION,
+        tail: 0,
+        instance: ROOT_CHOICE_MKFS_INSTANCE,
+    }; FAULT_SYSTEM_SLOTS_PER_DEVICE]; FAULT_DEVICE_COUNT]
+}
+
+/// 一次系统配置写（取号与发布末尾的轮换都是它，登记 5.3 第 3 条）：世代号 = 这块盘两槽里这一刻读得出（不是撕裂、也不在这次挂载一直读不出的落点里）的
+/// 最大世代号 + 1（一槽都读不出时 1），写在槽 世代号 mod 槽数。
+fn system_slot_write_of(
+    slots: &SystemSlots,
+    device: usize,
+    tail: u64,
+    instance: u32,
+    unreadable_all_mount_long: FaultLocations,
+) -> (usize, SystemSlotContent) {
+    let mut largest_generation = 0;
+    for (slot, content) in slots[device].iter().enumerate() {
+        if let SystemSlotContent::Verified { generation, .. } = content {
+            if !unreadable_all_mount_long.system_slot_is_unreadable(device, slot) {
+                largest_generation = largest_generation.max(*generation);
+            }
+        }
+    }
+    let generation = largest_generation + 1;
+    let slot =
+        usize::try_from(generation % u64::try_from(FAULT_SYSTEM_SLOTS_PER_DEVICE).expect("2"))
+            .expect("槽号是 0 或 1");
+    (
+        slot,
+        SystemSlotContent::Verified {
+            generation,
+            tail,
+            instance,
+        },
+    )
+}
+
+/// 这个读视图里读得出的系统配置槽的 tail：个数与最大值（没有时 0）。
+fn readable_system_slots_of(slots: &SystemSlots, locations: FaultLocations) -> (usize, u64) {
+    let mut count = 0;
+    let mut largest_tail = 0;
+    for (device, device_slots) in slots.iter().enumerate() {
+        for (slot, content) in device_slots.iter().enumerate() {
+            if let SystemSlotContent::Verified { tail, .. } = content {
+                if !locations.system_slot_is_unreadable(device, slot) {
+                    count += 1;
+                    largest_tail = largest_tail.max(*tail);
+                }
+            }
+        }
+    }
+    (count, largest_tail)
+}
+
+/// 挂载开头故障菜单的一项（登记 5.3 第 5 条）：时机 T1 = 这次挂载里对那个落点的第一次读读不出、之后每一次读都读得出（D23 已定项 14「重读一次」的 R = 1）；
+/// TP = 这次挂载里对那个落点的每一次读都读不出；两种都在下一次挂载开头恢复。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct MountFaultMenuEntry {
+    menu_index: usize,
+    /// 读阶段第一遍的读视图里读不出的落点（T1 与 TP 相同）。
+    first_read_unreadable: FaultLocations,
+    /// 重读那一遍、取号那一刻、写侧的读里仍读不出的落点：T1 是空，TP 同第一遍。
+    every_read_unreadable: FaultLocations,
+}
+
+impl MountFaultMenuEntry {
+    fn label(self) -> &'static str {
+        MOUNT_FAULT_MENU_LABELS[self.menu_index]
+    }
+
+    /// 历史标签（登记第一节「承诺内」第 1 条）：RNSB 是 D22 已定项 8 写明不罩的双故障形，别的落点罩着。
+    const fn is_inside_the_single_fault_promise(self) -> bool {
+        self.menu_index < RNSB_FIRST_MENU_INDEX
+    }
+}
+
+const RNSB_FIRST_MENU_INDEX: usize = 11;
+
+/// 菜单里六种落点的名字次序：RN、SN0、SN1、SB、RI、RNSB（每种配 T1、TP 两项，前面是「无」）。
+const MOUNT_FAULT_SITE_COUNT: usize = 6;
+
+fn locations_of_a_mount_fault_site(world: &FaultWorld, site_number: usize) -> FaultLocations {
+    let newest_root = world.newest_root_locations();
+    let newest_system_slot_of_device_0 = world.newest_system_slot_locations(0);
+    let newest_system_slot_of_device_1 = world.newest_system_slot_locations(1);
+    match site_number {
+        0 => newest_root,
+        1 => newest_system_slot_of_device_0,
+        2 => newest_system_slot_of_device_1,
+        3 => newest_system_slot_of_device_0.union(newest_system_slot_of_device_1),
+        4 => world.newest_instance_root_locations(),
+        5 => newest_root
+            .union(newest_system_slot_of_device_0)
+            .union(newest_system_slot_of_device_1),
+        unknown_site => panic!("故障落点只有 6 种，收到 {unknown_site}"),
+    }
+}
+
+/// 挂载 `mount_number` 开头的故障菜单：挂载 1 没有故障（只有「无」一项），挂载 2 起 13 项（登记 5.3 第 5 条）。
+fn mount_fault_menu(world: &FaultWorld, mount_number: usize) -> Vec<MountFaultMenuEntry> {
+    let mut menu = vec![MountFaultMenuEntry {
+        menu_index: 0,
+        first_read_unreadable: FaultLocations::NOWHERE,
+        every_read_unreadable: FaultLocations::NOWHERE,
+    }];
+    if mount_number < 2 {
+        return menu;
+    }
+    for site_number in 0..MOUNT_FAULT_SITE_COUNT {
+        let locations = locations_of_a_mount_fault_site(world, site_number);
+        menu.push(MountFaultMenuEntry {
+            menu_index: menu.len(),
+            first_read_unreadable: locations,
+            every_read_unreadable: FaultLocations::NOWHERE,
+        });
+        menu.push(MountFaultMenuEntry {
+            menu_index: menu.len(),
+            first_read_unreadable: locations,
+            every_read_unreadable: locations,
+        });
+    }
+    assert_eq!(menu.len(), MOUNT_FAULT_MENU_LENGTH, "故障菜单 13 项");
+    menu
+}
+
+labelled_enumeration! {
+    /// 被判的那一次恢复的读故障（登记 5.3 第 11 条）：无、根环里键最大的那个根槽读不出、最新实例的全部根读不出、某块盘不在。
+    enum JudgedFault {
+        Nothing => "none",
+        NewestRoot => "RN",
+        NewestInstanceRoots => "RI",
+        DeviceZeroGone => "D0",
+        DeviceOneGone => "D1",
+    }
+}
+
+impl JudgedFault {
+    /// 被判时标签（登记第一节「承诺内」第 2 条）：RI 一次让一个实例的全部根读不出，超出单故障，不罩；别的罩着。
+    const fn is_inside_the_single_fault_promise(self) -> bool {
+        match self {
+            Self::Nothing | Self::NewestRoot | Self::DeviceZeroGone | Self::DeviceOneGone => true,
+            Self::NewestInstanceRoots => false,
+        }
+    }
+
+    fn locations(self, world: &FaultWorld) -> FaultLocations {
+        match self {
+            Self::Nothing => FaultLocations::NOWHERE,
+            Self::NewestRoot => world.newest_root_locations(),
+            Self::NewestInstanceRoots => world.newest_instance_root_locations(),
+            Self::DeviceZeroGone => FaultLocations::NOWHERE.with_dead_device(0),
+            Self::DeviceOneGone => FaultLocations::NOWHERE.with_dead_device(1),
+        }
+    }
+}
+
+// ── 一段带读故障的历史走到某一刻的盘（世界）
+
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum MountOutcome {
+    /// 可写挂载读到的更新状态读不出（重读一次仍读不出），或取号那一刻有槽读不出：在取号之前拒，盘上不变，不取号。
+    Refused,
+    WentDownOnTheFirstRead,
+    WentDownOnTheSecondRead,
+}
+
+#[derive(Clone, Copy, Debug)]
+struct MountLogEntry {
+    fault: MountFaultMenuEntry,
+    outcome: MountOutcome,
+    /// 往下走的挂载：写行那次发布的 txg；被拒的挂载 0。
+    row_write_txg: u64,
+}
+
+/// 系统配置轮换那一段（登记 5.3 第 8 条）里两块盘各自写成什么样：没写、写了、撕裂（新旧都读不出）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum SystemWriteOutcome {
+    Unwritten,
+    Written,
+    Torn,
+}
+
+/// 一次发布的崩溃点（登记 5.3 第 8 条，D13 已定项 4 按设备切段）：记录那一段 2k 份拷贝的任一子集落了、根没落（4^k 个）；
+/// 根落了之后，两块盘的系统配置写各取三态之一（最后一次挂载并成「都没写」一个：被判的三步不读系统配置槽）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum FaultCrashPoint {
+    RecordCopiesLanded {
+        copy_mask: u32,
+    },
+    RootLanded {
+        system_writes: [SystemWriteOutcome; FAULT_DEVICE_COUNT],
+    },
+}
+
+fn fault_crash_points_of(record_count: usize, is_the_last_mount: bool) -> Vec<FaultCrashPoint> {
+    let mut crash_points: Vec<FaultCrashPoint> = (0..(1u32 << (2 * record_count)))
+        .map(|copy_mask| FaultCrashPoint::RecordCopiesLanded { copy_mask })
+        .collect();
+    if is_the_last_mount {
+        crash_points.push(FaultCrashPoint::RootLanded {
+            system_writes: [SystemWriteOutcome::Unwritten; FAULT_DEVICE_COUNT],
+        });
+        return crash_points;
+    }
+    let outcomes = [
+        SystemWriteOutcome::Unwritten,
+        SystemWriteOutcome::Written,
+        SystemWriteOutcome::Torn,
+    ];
+    for first_device in outcomes {
+        for second_device in outcomes {
+            crash_points.push(FaultCrashPoint::RootLanded {
+                system_writes: [first_device, second_device],
+            });
+        }
+    }
+    crash_points
+}
+
+#[derive(Clone, Copy, Debug)]
+struct FaultCrashLabel {
+    version: RootVersion,
+    record_count: usize,
+    point: FaultCrashPoint,
+}
+
+#[derive(Clone, Debug)]
+struct FaultWorld {
+    geometry: RootChoiceDomain,
+    /// 根环三个区域的槽，区域号主序、区域内槽号升序；没写过的槽是 `None`（读成 512 个 0）。
+    root_slots: Vec<Option<Arc<RootChoicePublishMaterial>>>,
+    /// 盘上至少有一份的记录，按写的次序。
+    landed_records: Vec<RootChoiceLandedRecord>,
+    system_slots: SystemSlots,
+    /// 崩溃点所在那次发布之前、fsync 已返回的用户发布，按写的次序。
+    confirmed_user_publishes: Vec<RootVersion>,
+    /// 版本的父链（登记 5.3 第 7 条）：每次挂载里每次发布的父是上一次发布，一次挂载第一次发布的父是这次挂载往下走所用的那一版。
+    parents: Vec<(RootVersion, RootVersion)>,
+    /// 开始过的每次发布（含崩在途中的）与 mkfs 的第 0 代根，按次序；第 0 个是第 0 代根。
+    started_publishes: Vec<Arc<RootChoicePublishMaterial>>,
+    mount_log: Vec<MountLogEntry>,
+    next_instance: u32,
+    refused_mounts: u64,
+    reread_mounts: u64,
+    final_crash: Option<FaultCrashLabel>,
+}
+
+impl FaultWorld {
+    /// mkfs 之后、一次挂载都还没有：第 0 代根在三个区域的槽 0，别处全 0，journal 环空，每块盘两个系统配置槽都是 (世代号 1, tail 0, 实例 0)。
+    fn right_after_mkfs(geometry: RootChoiceDomain) -> Self {
+        let genesis_root = Arc::new(mkfs_root_material());
+        let slots_per_region = geometry.slots_per_region();
+        let mut root_slots = vec![
+            None;
+            usize::try_from(ROOT_CHOICE_REGION_COUNT * slots_per_region)
+                .expect("根槽数不超过 24")
+        ];
+        for region in 0..ROOT_CHOICE_REGION_COUNT {
+            root_slots[root_choice_slot_index_of(region, 0, slots_per_region)] =
+                Some(Arc::clone(&genesis_root));
+        }
+        Self {
+            geometry,
+            root_slots,
+            landed_records: Vec::new(),
+            system_slots: system_slots_right_after_mkfs(),
+            confirmed_user_publishes: Vec::new(),
+            parents: Vec::new(),
+            started_publishes: vec![genesis_root],
+            mount_log: Vec::new(),
+            next_instance: 1,
+            refused_mounts: 0,
+            reread_mounts: 0,
+            final_crash: None,
+        }
+    }
+
+    fn slots_per_region(&self) -> u64 {
+        self.geometry.slots_per_region()
+    }
+
+    /// 开始过几次发布（第 0 代根不算）：新根段按全历史的发布序号取。
+    fn publishes_started(&self) -> u64 {
+        u64::try_from(self.started_publishes.len() - 1).expect("发布次数不超过 u64")
+    }
+
+    fn parent_of(&self, version: RootVersion) -> Option<RootVersion> {
+        self.parents
+            .iter()
+            .find(|(child, _parent)| *child == version)
+            .map(|(_child, parent)| *parent)
+    }
+
+    /// `version` 是 `ancestor` 本身，或在它的后代上（沿父链往上走能走到它）。
+    fn is_the_version_or_a_descendant_of(
+        &self,
+        version: RootVersion,
+        ancestor: RootVersion,
+    ) -> bool {
+        let mut current = version;
+        for _step in 0..=self.parents.len() {
+            if current == ancestor {
+                return true;
+            }
+            match self.parent_of(current) {
+                Some(parent) => current = parent,
+                None => return false,
+            }
+        }
+        false
+    }
+
+    fn fields_of_a_version(&self, version: RootVersion) -> Option<VersionFields> {
+        self.started_publishes
+            .iter()
+            .find(|material| material.version == version)
+            .map(|material| material.fields)
+    }
+
+    /// 根环里键 (txg, 实例代号) 最大的那个根槽（几个槽同键取读根环次序里第一个）。
+    fn newest_root_locations(&self) -> FaultLocations {
+        let mut newest: Option<(usize, RootVersion)> = None;
+        for (root_slot_index, root) in self.root_slots.iter().enumerate() {
+            if let Some(material) = root {
+                if newest.is_none_or(|(_index, version)| material.version > version) {
+                    newest = Some((root_slot_index, material.version));
+                }
+            }
+        }
+        newest.map_or(FaultLocations::NOWHERE, |(root_slot_index, _version)| {
+            FaultLocations::NOWHERE.with_root_slot(root_slot_index)
+        })
+    }
+
+    /// 「RN 那条根的实例」的全部根所在的槽；那个实例是 0（mkfs 的第 0 代）时没有落点。
+    fn newest_instance_root_locations(&self) -> FaultLocations {
+        let newest_instance = self
+            .root_slots
+            .iter()
+            .flatten()
+            .map(|material| material.version)
+            .max()
+            .map_or(ROOT_CHOICE_MKFS_INSTANCE, |version| version.instance);
+        let mut locations = FaultLocations::NOWHERE;
+        if newest_instance == ROOT_CHOICE_MKFS_INSTANCE {
+            return locations;
+        }
+        for (root_slot_index, root) in self.root_slots.iter().enumerate() {
+            if root
+                .as_ref()
+                .is_some_and(|material| material.version.instance == newest_instance)
+            {
+                locations = locations.with_root_slot(root_slot_index);
+            }
+        }
+        locations
+    }
+
+    /// 这块盘两槽里没撕裂、世代号较大的那一槽，世代号相同取槽号大的（两槽都撕了时没有落点）。
+    fn newest_system_slot_locations(&self, device: usize) -> FaultLocations {
+        let mut newest: Option<(u64, usize)> = None;
+        for (slot, content) in self.system_slots[device].iter().enumerate() {
+            if let SystemSlotContent::Verified { generation, .. } = content {
+                if newest.is_none_or(|(largest, _slot)| *generation >= largest) {
+                    newest = Some((*generation, slot));
+                }
+            }
+        }
+        newest.map_or(FaultLocations::NOWHERE, |(_generation, slot)| {
+            FaultLocations::NOWHERE.with_system_slot(device, slot)
+        })
+    }
+}
+
+/// 这次发布最后一条记录的计数器：系统配置轮换写的 tail（登记 5.3 第 4 条：tail = c + k − 1）。
+fn last_counter_of_a_publish(material: &RootChoicePublishMaterial) -> u64 {
+    material
+        .records
+        .last()
+        .expect("每次发布至少一条记录")
+        .parsed
+        .as_ref()
+        .expect("合法历史的每条记录都解得出（生成时断言过）")
+        .counter
+}
+
+impl FaultWorld {
+    /// 这次发布开始了：记下它的父（登记 5.3 第 7 条）与它的材料。崩溃点与整次落盘都在这之后。
+    fn register_a_started_publish(
+        &mut self,
+        material: &Arc<RootChoicePublishMaterial>,
+        parent: RootVersion,
+    ) {
+        self.parents.push((material.version, parent));
+        self.started_publishes.push(Arc::clone(material));
+    }
+
+    fn put_every_record_of_a_publish_on_both_devices(
+        &mut self,
+        material: &Arc<RootChoicePublishMaterial>,
+    ) {
+        for record_index in 0..material.records.len() {
+            self.landed_records.push(RootChoiceWorld::landed_record_of(
+                material,
+                record_index,
+                [true, true],
+            ));
+        }
+    }
+
+    fn put_the_root_of_a_publish_in_its_slot(&mut self, material: &Arc<RootChoicePublishMaterial>) {
+        let slots_per_region = self.slots_per_region();
+        let (region, slot) =
+            root_choice_target_of_publish(material.version.checkpoint_txg, slots_per_region);
+        self.root_slots[root_choice_slot_index_of(region, slot, slots_per_region)] =
+            Some(Arc::clone(material));
+    }
+
+    /// 每块盘一次系统配置写（取号与每次发布末尾的轮换），写进它的落槽。`unreadable_all_mount_long` 是这次挂载里一直读不出的落点（TP 故障）。
+    fn write_the_system_configuration_of_every_device(
+        &mut self,
+        tail: u64,
+        instance: u32,
+        unreadable_all_mount_long: FaultLocations,
+    ) {
+        for device in 0..FAULT_DEVICE_COUNT {
+            let (slot, content) = system_slot_write_of(
+                &self.system_slots,
+                device,
+                tail,
+                instance,
+                unreadable_all_mount_long,
+            );
+            self.system_slots[device][slot] = content;
+        }
+    }
+
+    /// 这次发布整次落了：每条记录两盘各一份，根写进它的落点，两块盘各一次系统配置写（登记 5.3 第 4 条）。
+    /// `is_confirmed`：它之后 fsync 返回了（只有用户发布、而且后面还有用户发布时才是）。
+    fn apply_a_publish_in_full(
+        &mut self,
+        material: &Arc<RootChoicePublishMaterial>,
+        is_confirmed: bool,
+        unreadable_all_mount_long: FaultLocations,
+    ) {
+        self.put_every_record_of_a_publish_on_both_devices(material);
+        self.put_the_root_of_a_publish_in_its_slot(material);
+        self.write_the_system_configuration_of_every_device(
+            last_counter_of_a_publish(material),
+            material.version.instance,
+            unreadable_all_mount_long,
+        );
+        if is_confirmed {
+            self.confirmed_user_publishes.push(material.version);
+        }
+    }
+
+    /// 这次发布崩在 `point` 上之后的盘。
+    fn with_crash_applied(
+        &self,
+        material: &Arc<RootChoicePublishMaterial>,
+        point: FaultCrashPoint,
+        unreadable_all_mount_long: FaultLocations,
+    ) -> Self {
+        let mut world = self.clone();
+        match point {
+            FaultCrashPoint::RecordCopiesLanded { copy_mask } => {
+                for record_index in 0..material.records.len() {
+                    let copies_on_devices = copies_of_a_record_in_a_mask(copy_mask, record_index);
+                    if copies_on_devices[0] || copies_on_devices[1] {
+                        world.landed_records.push(RootChoiceWorld::landed_record_of(
+                            material,
+                            record_index,
+                            copies_on_devices,
+                        ));
+                    }
+                }
+            }
+            FaultCrashPoint::RootLanded { system_writes } => {
+                world.put_every_record_of_a_publish_on_both_devices(material);
+                world.put_the_root_of_a_publish_in_its_slot(material);
+                for (device, system_write) in system_writes.iter().enumerate() {
+                    let (slot, content) = system_slot_write_of(
+                        &self.system_slots,
+                        device,
+                        last_counter_of_a_publish(material),
+                        material.version.instance,
+                        unreadable_all_mount_long,
+                    );
+                    match system_write {
+                        SystemWriteOutcome::Unwritten => {}
+                        SystemWriteOutcome::Written => world.system_slots[device][slot] = content,
+                        SystemWriteOutcome::Torn => {
+                            world.system_slots[device][slot] = SystemSlotContent::Torn;
+                        }
+                    }
+                }
+            }
+        }
+        world.final_crash = Some(FaultCrashLabel {
+            version: material.version,
+            record_count: material.records.len(),
+            point,
+        });
+        world
+    }
+}
+
+// ── 读视图：盘减去读不出的落点
+
+/// 世界在 `locations` 这个读视图里的样子：读不出的根槽当没有根，盘不在时那块盘上的记录拷贝读不出（只有一份在那块盘上的记录整条读不出）。
+fn fault_landing_of_a_view(world: &FaultWorld, locations: FaultLocations) -> RootChoiceLanding<'_> {
+    let slots_per_region = world.slots_per_region();
+    let roots_in_ring_order: Vec<Option<&RootChoicePublishMaterial>> = world
+        .root_slots
+        .iter()
+        .enumerate()
+        .map(|(root_slot_index, root)| {
+            if locations.root_slot_is_unreadable(root_slot_index, slots_per_region) {
+                None
+            } else {
+                root.as_deref()
+            }
+        })
+        .collect();
+    let mut records = Vec::with_capacity(world.landed_records.len());
+    for landed in &world.landed_records {
+        let copies_on_devices = [
+            landed.copies_on_devices[0] && !locations.device_is_dead(0),
+            landed.copies_on_devices[1] && !locations.device_is_dead(1),
+        ];
+        if copies_on_devices[0] || copies_on_devices[1] {
+            records.push(RootChoiceLandedRecordRef {
+                material: &landed.material,
+                record_index: landed.record_index,
+                copies_on_devices,
+            });
+        }
+    }
+    RootChoiceLanding {
+        roots_in_ring_order,
+        records,
+    }
+}
+
+/// 条款的理想恢复在这个读视图上落到哪一版（登记 5.5 KJ3 的写法：读视图里版本最大的根起，同实例逐次往后）；读视图里一条根都没有时 `None`。
+fn ideal_landed_version_of_a_view(
+    view: &RootChoiceLanding<'_>,
+    in_flight_limit: usize,
+) -> Option<RootVersion> {
+    root_choice_newest_root(view).map(|newest| {
+        root_choice_ideal_recovery_from(newest, &view.records, in_flight_limit).landed_version
+    })
+}
+
+/// 可写挂载读到的更新状态读不出那道闸（D23 已定项 14，实现的取甲，登记修订 1）：`landed` 是读阶段落到的那一版，`locations` 是这一遍读视图里读不出的落点。
+/// 为真 = 判不出或见证到比所选那一版更新的发布。写法照 `mount.rs` 的 `newer_publish_witness` 那一段条款文字，不调它。
+fn newer_publish_is_witnessed(
+    world: &FaultWorld,
+    locations: FaultLocations,
+    landed: RootVersion,
+) -> bool {
+    let (readable_slot_count, largest_tail) =
+        readable_system_slots_of(&world.system_slots, locations);
+    if readable_slot_count < FAULT_SYSTEM_SLOT_COUNT_OF_THE_POOL {
+        return true;
+    }
+    if largest_tail == 0 {
+        return false;
+    }
+    let view = fault_landing_of_a_view(world, locations);
+    let last_record_counter_of_the_landed_publish = view
+        .records
+        .iter()
+        .filter(|record| {
+            record.material.version == landed
+                && record.record_index + 1 == record.material.records.len()
+        })
+        .map(counter_of_the_landed_record)
+        .max();
+    if let Some(counter) = last_record_counter_of_the_landed_publish {
+        return largest_tail > counter;
+    }
+    let newest_of_the_record_at_the_tail = view
+        .records
+        .iter()
+        .filter(|record| counter_of_the_landed_record(record) == largest_tail)
+        .map(|record| {
+            (
+                record.material.version.instance,
+                record.material.version.checkpoint_txg,
+            )
+        })
+        .max();
+    match newest_of_the_record_at_the_tail {
+        Some(instance_and_txg) => instance_and_txg > (landed.instance, landed.checkpoint_txg),
+        None => true,
+    }
+}
+
+// ── 被判的三步看到的盘：真相的内存盘加上读故障
+
+/// 三步调用的读者：真相的内存盘（根槽、记录槽摆在各自的落点上），再减去读不出的落点：根槽交读错，盘不在时那块盘上的读全交读错。
+struct FaultedRootChoiceDisk<'disk, 'landing> {
+    inner: &'disk RootChoiceDisk<'landing>,
+    locations: FaultLocations,
+    slots_per_region: u64,
+}
+
+impl PoolReader for FaultedRootChoiceDisk<'_, '_> {
+    fn device_identities(&self) -> Vec<DeviceIdentity> {
+        self.inner.device_identities()
+    }
+
+    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
+        self.inner.device_size_in_bytes(device)
+    }
+
+    fn read(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        length: usize,
+    ) -> Option<Vec<u8>> {
+        if let Ok(device_index) = usize::try_from(device.0) {
+            if device_index < FAULT_DEVICE_COUNT && self.locations.device_is_dead(device_index) {
+                return None;
+            }
+        }
+        if let Some(root_slot_index) =
+            root_choice_root_slot_index_at(device, offset, self.slots_per_region)
+        {
+            if self
+                .locations
+                .root_slot_is_unreadable(root_slot_index, self.slots_per_region)
+            {
+                return None;
+            }
+        }
+        self.inner.read(device, offset, length)
+    }
+
+    fn journal_record_offsets_hint(
+        &self,
+        _device: DeviceIdentity,
+        _ring_start: DeviceOffsetInBytes,
+        _ring_bytes: u64,
+    ) -> Option<Vec<DeviceOffsetInBytes>> {
+        None
+    }
+}
+
+// ── 枚举一个 (域, M, U) 格里的全部历史（登记 5.3 第 6 至 10 条）
+
+/// 走枚举的一方：收工作单元的起点，或判最后一次挂载崩溃点上的每个状态。
+trait FaultVisitor {
+    /// 走到「第 `mount_number` 次挂载开始之前」（只在 `stop_before_mount` 点名的那一次挂载上触发，之后不再往下走）。
+    fn reach_the_start_of_a_mount(
+        &mut self,
+        world: &FaultWorld,
+        mount_number: usize,
+        user_publishes_left: usize,
+    );
+
+    /// 最后一次挂载走完的一个状态：崩溃点已落在 `world` 上；最后一次挂载被拒时是被拒之后的盘（没有崩溃点）。
+    fn visit_a_final_state(&mut self, world: &FaultWorld);
+}
+
+struct FaultEnumeration {
+    domain: FaultDomain,
+    mounts: usize,
+    stop_before_mount: Option<usize>,
+}
+
+/// 一次往下走的挂载的几样不变的设定。
+#[derive(Clone, Copy, Debug)]
+struct MountSetting {
+    mount_number: usize,
+    user_publishes_after_this_mount: usize,
+    /// 这次挂载里一直读不出的落点（TP 故障）：系统配置写算世代号时不看它们。
+    unreadable_all_mount_long: FaultLocations,
+    is_the_last_mount: bool,
+}
+
+impl FaultEnumeration {
+    fn in_flight_limit(&self) -> usize {
+        self.domain.geometry().ring().local_in_flight_limit()
+    }
+
+    /// 第 `mount_number` 次挂载：挂载 2 起开头先取一项故障（13 项），读阶段、闸、取号、写行与暖机的空发布、用户发布 u_m 次，崩溃点在第 u_m 次用户发布里；
+    /// u_m = 0 时在写行或任一次暖机发布里。非最后一次挂载 u_m 取 0..=剩余的用户发布数，最后一次用完剩余的。`only_menu_index` 只走故障菜单的那一项。
+    fn enumerate_a_mount(
+        &self,
+        world: &FaultWorld,
+        mount_number: usize,
+        user_publishes_left: usize,
+        only_menu_index: Option<usize>,
+        visitor: &mut dyn FaultVisitor,
+    ) {
+        if self.stop_before_mount == Some(mount_number) {
+            visitor.reach_the_start_of_a_mount(world, mount_number, user_publishes_left);
+            return;
+        }
+        for entry in mount_fault_menu(world, mount_number) {
+            if only_menu_index.is_some_and(|only| only != entry.menu_index) {
+                continue;
+            }
+            self.enumerate_a_mount_under_a_fault(
+                world,
+                mount_number,
+                user_publishes_left,
+                entry,
+                visitor,
+            );
+        }
+    }
+
+    /// 读阶段：第一遍读视图 = 盘减去故障落点，在它上面做条款的理想恢复；闸为真就用第二遍读视图（盘减去 TP 的落点）重做，仍为真拒；
+    /// 往下走之前再判取号那一刻（两块盘的四个系统配置槽在 TP 视图里有一槽读不出或撕裂就拒）。KG 关掉闸与取号那一刻（登记 5.3 第 6 条）。
+    fn enumerate_a_mount_under_a_fault(
+        &self,
+        world: &FaultWorld,
+        mount_number: usize,
+        user_publishes_left: usize,
+        entry: MountFaultMenuEntry,
+        visitor: &mut dyn FaultVisitor,
+    ) {
+        let limit = self.in_flight_limit();
+        let first_view = fault_landing_of_a_view(world, entry.first_read_unreadable);
+        let mut landed = ideal_landed_version_of_a_view(&first_view, limit)
+            .expect("停机条款 TH9：读阶段第一遍的读视图里一条根都没有");
+        let mut settled_unreadable = entry.first_read_unreadable;
+        let mut outcome = MountOutcome::WentDownOnTheFirstRead;
+        let mut is_refused = false;
+        if self.domain.gate_is_on() {
+            if newer_publish_is_witnessed(world, entry.first_read_unreadable, landed) {
+                let second_view = fault_landing_of_a_view(world, entry.every_read_unreadable);
+                let second_landed = ideal_landed_version_of_a_view(&second_view, limit)
+                    .expect("停机条款 TH9：读阶段第二遍的读视图里一条根都没有");
+                if newer_publish_is_witnessed(world, entry.every_read_unreadable, second_landed) {
+                    is_refused = true;
+                } else {
+                    settled_unreadable = entry.every_read_unreadable;
+                    landed = second_landed;
+                    outcome = MountOutcome::WentDownOnTheSecondRead;
+                }
+            }
+            let (readable_slot_count, _largest_tail) =
+                readable_system_slots_of(&world.system_slots, entry.every_read_unreadable);
+            if readable_slot_count < FAULT_SYSTEM_SLOT_COUNT_OF_THE_POOL {
+                is_refused = true;
+            }
+        }
+        if is_refused {
+            let mut refused_world = world.clone();
+            refused_world.refused_mounts += 1;
+            refused_world.mount_log.push(MountLogEntry {
+                fault: entry,
+                outcome: MountOutcome::Refused,
+                row_write_txg: 0,
+            });
+            if mount_number == self.mounts {
+                visitor.visit_a_final_state(&refused_world);
+            } else {
+                self.enumerate_a_mount(
+                    &refused_world,
+                    mount_number + 1,
+                    user_publishes_left,
+                    None,
+                    visitor,
+                );
+            }
+            return;
+        }
+        self.go_down_and_write(
+            world,
+            (mount_number, user_publishes_left),
+            (entry, outcome, settled_unreadable, landed),
+            visitor,
+        );
+    }
+}
+
+impl FaultEnumeration {
+    /// 往下走：实例代号 = 之前取过号的挂载数 + 1；写行 txg = 往下走那一遍读得出的根与（闸开时）记录的最大 txg + 1；第一条记录的计数器 = 那一遍读得出的最大计数器 + 1；
+    /// 取号那一写每块盘一次系统配置写，tail = 取号那一刻读得出的槽 tail 的最大值（不取崩溃点）；之后写行、暖机、用户发布（登记 5.3 第 6 条）。
+    fn go_down_and_write(
+        &self,
+        world: &FaultWorld,
+        (mount_number, user_publishes_left): (usize, usize),
+        (entry, outcome, settled_unreadable, landed): (
+            MountFaultMenuEntry,
+            MountOutcome,
+            FaultLocations,
+            RootVersion,
+        ),
+        visitor: &mut dyn FaultVisitor,
+    ) {
+        let settled_view = fault_landing_of_a_view(world, settled_unreadable);
+        let highest_root_txg = settled_view
+            .roots_in_ring_order
+            .iter()
+            .flatten()
+            .map(|material| material.version.checkpoint_txg)
+            .max()
+            .expect("停机条款 TH9：往下走那一遍的读视图里一条根都没有");
+        let highest_record_txg = settled_view
+            .records
+            .iter()
+            .map(|record| record.material.version.checkpoint_txg)
+            .max()
+            .unwrap_or(0);
+        let row_write_txg = if self.domain.gate_is_on() {
+            highest_root_txg.max(highest_record_txg) + 1
+        } else {
+            highest_root_txg + 1
+        };
+        let first_counter = settled_view
+            .records
+            .iter()
+            .map(counter_of_the_landed_record)
+            .max()
+            .unwrap_or(0)
+            + 1;
+        let instance = world.next_instance;
+        let mut acquired = world.clone();
+        let (_readable_slot_count, largest_tail_at_the_acquisition) =
+            readable_system_slots_of(&world.system_slots, entry.every_read_unreadable);
+        acquired.write_the_system_configuration_of_every_device(
+            largest_tail_at_the_acquisition,
+            instance,
+            entry.every_read_unreadable,
+        );
+        acquired.next_instance += 1;
+        if outcome == MountOutcome::WentDownOnTheSecondRead {
+            acquired.reread_mounts += 1;
+        }
+        acquired.mount_log.push(MountLogEntry {
+            fault: entry,
+            outcome,
+            row_write_txg,
+        });
+        let overhead_txgs = root_choice_warm_up_plan(row_write_txg);
+        let is_the_last_mount = mount_number == self.mounts;
+        let user_publish_counts: Vec<usize> = if is_the_last_mount {
+            vec![user_publishes_left]
+        } else {
+            (0..=user_publishes_left).collect()
+        };
+        for user_publish_count in user_publish_counts {
+            let setting = MountSetting {
+                mount_number,
+                user_publishes_after_this_mount: user_publishes_left - user_publish_count,
+                unreadable_all_mount_long: entry.every_read_unreadable,
+                is_the_last_mount,
+            };
+            let mut running = acquired.clone();
+            let mut cursor = RootChoiceMountCursor {
+                instance,
+                next_counter: first_counter,
+                next_transaction: 1,
+                previous_chain_value: None,
+                next_checkpoint_txg: overhead_txgs[0],
+            };
+            let mut previous_version = landed;
+            for overhead_txg in &overhead_txgs {
+                assert_eq!(
+                    cursor.next_checkpoint_txg, *overhead_txg,
+                    "开销发布的 txg 逐个加一"
+                );
+                let material = Arc::new(root_choice_publish_material(
+                    &cursor.plan_of_the_next_publish(
+                        self.domain.geometry(),
+                        running.publishes_started(),
+                        1,
+                        0,
+                        false,
+                    ),
+                ));
+                running.register_a_started_publish(&material, previous_version);
+                if user_publish_count == 0 {
+                    self.enumerate_the_crash_points(&running, &material, setting, visitor);
+                }
+                running.apply_a_publish_in_full(
+                    &material,
+                    false,
+                    setting.unreadable_all_mount_long,
+                );
+                cursor = cursor.after_the_publish_of(&material);
+                previous_version = material.version;
+            }
+            if user_publish_count > 0 {
+                self.enumerate_user_publishes(
+                    &running,
+                    cursor,
+                    (previous_version, user_publish_count),
+                    setting,
+                    visitor,
+                );
+            }
+        }
+    }
+
+    /// 第 `user_publishes_in_the_mount`（≥ 1）次用户发布起，每次的 k 与提交标记摆法各一种；崩溃点在这一挂载的最后一次用户发布里。
+    fn enumerate_user_publishes(
+        &self,
+        running: &FaultWorld,
+        cursor: RootChoiceMountCursor,
+        (previous_version, user_publishes_in_the_mount): (RootVersion, usize),
+        setting: MountSetting,
+        visitor: &mut dyn FaultVisitor,
+    ) {
+        for record_count in self.domain.user_publish_record_counts() {
+            for commit_marker_bits in 0..(1u32 << (record_count - 1)) {
+                let material = Arc::new(root_choice_publish_material(
+                    &cursor.plan_of_the_next_publish(
+                        self.domain.geometry(),
+                        running.publishes_started(),
+                        *record_count,
+                        commit_marker_bits,
+                        true,
+                    ),
+                ));
+                let mut branch = running.clone();
+                branch.register_a_started_publish(&material, previous_version);
+                if user_publishes_in_the_mount == 1 {
+                    self.enumerate_the_crash_points(&branch, &material, setting, visitor);
+                } else {
+                    branch.apply_a_publish_in_full(
+                        &material,
+                        true,
+                        setting.unreadable_all_mount_long,
+                    );
+                    self.enumerate_user_publishes(
+                        &branch,
+                        cursor.after_the_publish_of(&material),
+                        (material.version, user_publishes_in_the_mount - 1),
+                        setting,
+                        visitor,
+                    );
+                }
+            }
+        }
+    }
+
+    /// 崩在 `material` 这次发布的每个崩溃点上：最后一次挂载交给 `visitor` 判；之前的挂载接着枚举下一次挂载。
+    fn enumerate_the_crash_points(
+        &self,
+        running: &FaultWorld,
+        material: &Arc<RootChoicePublishMaterial>,
+        setting: MountSetting,
+        visitor: &mut dyn FaultVisitor,
+    ) {
+        for point in fault_crash_points_of(material.records.len(), setting.is_the_last_mount) {
+            let crashed =
+                running.with_crash_applied(material, point, setting.unreadable_all_mount_long);
+            if setting.is_the_last_mount {
+                visitor.visit_a_final_state(&crashed);
+            } else {
+                self.enumerate_a_mount(
+                    &crashed,
+                    setting.mount_number + 1,
+                    setting.user_publishes_after_this_mount,
+                    None,
+                    visitor,
+                );
+            }
+        }
+    }
+}
+
+// ── 判据与计数（登记 5.5）
+
+labelled_enumeration! {
+    /// 判据表（登记 5.5）；阳性对照那段循环遍历它。KJ1T 是这一族多出来的一条（登记第一节）。
+    enum FaultCriterion {
+        Kj0Defined => "KJ0",
+        Kj1NotEarlierThanTheLastConfirmedPublish => "KJ1",
+        Kj1tDescendsFromTheLastConfirmedPublish => "KJ1T",
+        Kj2NotEarlierThanAnyReadableRoot => "KJ2",
+        Kj3LandsOnTheIdealRecovery => "KJ3",
+    }
+}
+
+const FAULT_CRITERION_COUNT: usize = 5;
+const KJ1_INDEX: usize = 1;
+const KJ1T_INDEX: usize = 2;
+const KJ2_INDEX: usize = 3;
+const KJ3_INDEX: usize = 4;
+/// 本地择根序的个数：登记 9.1 的五种加平局取后者赢（v1822），再加 KF1 用的「取键最小的根」。
+const FAULT_LOCAL_RULE_COUNT: usize = 7;
+const FAULT_OLDEST_RULE_INDEX: usize = 6;
+/// 每个 (域, M, U) 格里按「被判时故障 × 历史标签」分的格数。
+const FAULT_CELL_COUNT: usize = 10;
+
+/// 一个 (被判时故障, 历史标签) 格里数的量：状态数、有已确认的态数、上限截掉的态数，与每条判据的判数、违例数（上限截掉的态里的违例另记）。
+#[derive(Clone, Copy, Debug, Default)]
+struct FaultCellTally {
+    states: u64,
+    confirmed_states: u64,
+    cut_states: u64,
+    judged: [u64; FAULT_CRITERION_COUNT],
+    violations: [u64; FAULT_CRITERION_COUNT],
+    cut_violations: [u64; FAULT_CRITERION_COUNT],
+    /// 条款的理想恢复在这一格里 KJ1、KJ1T 各违例几次（TQ3 把实现的数与它并列）。
+    ideal_violations: [u64; 2],
+}
+
+impl FaultCellTally {
+    fn merge(&mut self, later: &Self) {
+        self.states += later.states;
+        self.confirmed_states += later.confirmed_states;
+        self.cut_states += later.cut_states;
+        for criterion in 0..FAULT_CRITERION_COUNT {
+            self.judged[criterion] += later.judged[criterion];
+            self.violations[criterion] += later.violations[criterion];
+            self.cut_violations[criterion] += later.cut_violations[criterion];
+        }
+        for criterion in 0..2 {
+            self.ideal_violations[criterion] += later.ideal_violations[criterion];
+        }
+    }
+}
+
+/// 格的下标：被判时故障的次序 × 2 + 历史标签（0 罩着、1 不罩）。
+fn fault_cell_index_of(judged_fault: JudgedFault, history_is_covered: bool) -> usize {
+    judged_fault.index() * 2 + usize::from(!history_is_covered)
+}
+
+/// M = 2 的格按挂载 2 开头那项故障报的三样。
+#[derive(Clone, Copy, Debug, Default)]
+struct MountTwoFaultCount {
+    histories: u64,
+    refused: u64,
+    reread: u64,
+}
+
+/// 一个 (域, M, U) 格的全部计数。违例类的量在第一次加一时记一个见证。
+#[derive(Clone, Debug)]
+struct FaultTally {
+    cells: [FaultCellTally; FAULT_CELL_COUNT],
+    /// 每个格每条判据的第一个见证。
+    witnesses: Vec<Option<String>>,
+    /// 承诺内（历史罩着 ∧ 被判时罩着 ∧ 不是上限截掉）的态数与 KJ1、KJ1T 的违例数，装置直接数（登记 5.5 末段）。
+    promised_states: u64,
+    promised_violations: [u64; 2],
+    promised_witnesses: [Option<String>; 2],
+    histories: u64,
+    histories_with_an_uncovered_fault: u64,
+    judged_states: u64,
+    confirmed_states: u64,
+    refused_mounts: u64,
+    reread_mounts: u64,
+    states_with_a_torn_system_slot: u64,
+    states_with_same_txg_different_instance_roots: u64,
+    states_with_higher_instance_lower_txg_roots: u64,
+    states_without_a_readable_root: u64,
+    above_newest_root_maximum: usize,
+    confirmed_publishes_maximum: usize,
+    largest_counter: u64,
+    ideal_loss_by_limit: u64,
+    ideal_loss_covered_other: u64,
+    ideal_loss_uncovered: u64,
+    mount_two: [MountTwoFaultCount; MOUNT_FAULT_MENU_LENGTH],
+    panicked_states: u64,
+    chose_nothing_with_a_readable_root: u64,
+    chose_nothing_without_a_readable_root: u64,
+    replay_failed_states: u64,
+    replay_applied_states: u64,
+    slot_readings_checked: u64,
+    slot_readings_differing_from_the_truth: u64,
+    slot_reading_witness: Option<String>,
+    /// KJ0 的对照：在指定的一态上 panic，别的态不算。
+    kj0_control_violations: u64,
+    /// 本地择根序加理想重放读到的 KJ1、KJ1T、KJ2 违例数（登记第七节「本地择根序」那张表；下标 6 是 KF1 的「取键最小的根」）。
+    local_rules: [[u64; 3]; FAULT_LOCAL_RULE_COUNT],
+    /// 本地择根序的 KJ1T 违例数按被判时故障分（C331 的判别力自证：今天的序在被判时无故障的态上的合计）。
+    local_kj1t_by_judged_fault: [[u64; 5]; FAULT_LOCAL_RULE_COUNT],
+    /// KF2 的 KJ1、KJ1T、KJ2、KJ3 读数；KF4 的 KJ1、KJ1T 读数（登记 5.6）。
+    kf2: [u64; 4],
+    kf4: [u64; 2],
+    kf4_states_without_a_root_off_the_confirmed_chain: u64,
+    /// KJ1 与 KJ1T 对同一态判得不同的态数（TQ15）。
+    kj1_and_kj1t_differing_states: u64,
+    kj1_and_kj1t_differing_witness: Option<String>,
+}
+
+impl FaultTally {
+    fn new() -> Self {
+        Self {
+            cells: [FaultCellTally::default(); FAULT_CELL_COUNT],
+            witnesses: vec![None; FAULT_CELL_COUNT * FAULT_CRITERION_COUNT],
+            promised_states: 0,
+            promised_violations: [0; 2],
+            promised_witnesses: [None, None],
+            histories: 0,
+            histories_with_an_uncovered_fault: 0,
+            judged_states: 0,
+            confirmed_states: 0,
+            refused_mounts: 0,
+            reread_mounts: 0,
+            states_with_a_torn_system_slot: 0,
+            states_with_same_txg_different_instance_roots: 0,
+            states_with_higher_instance_lower_txg_roots: 0,
+            states_without_a_readable_root: 0,
+            above_newest_root_maximum: 0,
+            confirmed_publishes_maximum: 0,
+            largest_counter: 0,
+            ideal_loss_by_limit: 0,
+            ideal_loss_covered_other: 0,
+            ideal_loss_uncovered: 0,
+            mount_two: [MountTwoFaultCount::default(); MOUNT_FAULT_MENU_LENGTH],
+            panicked_states: 0,
+            chose_nothing_with_a_readable_root: 0,
+            chose_nothing_without_a_readable_root: 0,
+            replay_failed_states: 0,
+            replay_applied_states: 0,
+            slot_readings_checked: 0,
+            slot_readings_differing_from_the_truth: 0,
+            slot_reading_witness: None,
+            kj0_control_violations: 0,
+            local_rules: [[0; 3]; FAULT_LOCAL_RULE_COUNT],
+            local_kj1t_by_judged_fault: [[0; 5]; FAULT_LOCAL_RULE_COUNT],
+            kf2: [0; 4],
+            kf4: [0; 2],
+            kf4_states_without_a_root_off_the_confirmed_chain: 0,
+            kj1_and_kj1t_differing_states: 0,
+            kj1_and_kj1t_differing_witness: None,
+        }
+    }
+}
+
+fn merge_the_first_witness(earlier: &mut Option<String>, later: &Option<String>) {
+    if earlier.is_none() {
+        earlier.clone_from(later);
+    }
+}
+
+impl FaultTally {
+    /// 违例加一，并在这个格这条判据的第一条违例上记一个见证（枚举次序里最先的那个，合并时按工作单元的次序保留最先的）。
+    fn add_cell_violation(
+        &mut self,
+        cell_index: usize,
+        criterion_index: usize,
+        describe_witness: &dyn Fn() -> String,
+    ) {
+        self.cells[cell_index].violations[criterion_index] += 1;
+        let witness = &mut self.witnesses[cell_index * FAULT_CRITERION_COUNT + criterion_index];
+        if witness.is_none() {
+            *witness = Some(describe_witness());
+        }
+    }
+
+    fn witness_of_a_cell(&self, cell_index: usize, criterion_index: usize) -> &str {
+        self.witnesses[cell_index * FAULT_CRITERION_COUNT + criterion_index]
+            .as_deref()
+            .unwrap_or("none")
+    }
+
+    /// 一条判据在全部格上的违例合计。
+    fn violations_of_a_criterion(&self, criterion_index: usize) -> u64 {
+        self.cells
+            .iter()
+            .map(|cell| cell.violations[criterion_index])
+            .sum()
+    }
+
+    fn judged_of_a_criterion(&self, criterion_index: usize) -> u64 {
+        self.cells
+            .iter()
+            .map(|cell| cell.judged[criterion_index])
+            .sum()
+    }
+
+    fn merge(&mut self, later: &Self) {
+        for (cell, later_cell) in self.cells.iter_mut().zip(&later.cells) {
+            cell.merge(later_cell);
+        }
+        for (witness, later_witness) in self.witnesses.iter_mut().zip(&later.witnesses) {
+            merge_the_first_witness(witness, later_witness);
+        }
+        self.promised_states += later.promised_states;
+        for criterion in 0..2 {
+            self.promised_violations[criterion] += later.promised_violations[criterion];
+            merge_the_first_witness(
+                &mut self.promised_witnesses[criterion],
+                &later.promised_witnesses[criterion],
+            );
+        }
+        self.histories += later.histories;
+        self.histories_with_an_uncovered_fault += later.histories_with_an_uncovered_fault;
+        self.judged_states += later.judged_states;
+        self.confirmed_states += later.confirmed_states;
+        self.refused_mounts += later.refused_mounts;
+        self.reread_mounts += later.reread_mounts;
+        self.states_with_a_torn_system_slot += later.states_with_a_torn_system_slot;
+        self.states_with_same_txg_different_instance_roots +=
+            later.states_with_same_txg_different_instance_roots;
+        self.states_with_higher_instance_lower_txg_roots +=
+            later.states_with_higher_instance_lower_txg_roots;
+        self.states_without_a_readable_root += later.states_without_a_readable_root;
+        self.above_newest_root_maximum = self
+            .above_newest_root_maximum
+            .max(later.above_newest_root_maximum);
+        self.confirmed_publishes_maximum = self
+            .confirmed_publishes_maximum
+            .max(later.confirmed_publishes_maximum);
+        self.largest_counter = self.largest_counter.max(later.largest_counter);
+        self.ideal_loss_by_limit += later.ideal_loss_by_limit;
+        self.ideal_loss_covered_other += later.ideal_loss_covered_other;
+        self.ideal_loss_uncovered += later.ideal_loss_uncovered;
+        for (count, later_count) in self.mount_two.iter_mut().zip(&later.mount_two) {
+            count.histories += later_count.histories;
+            count.refused += later_count.refused;
+            count.reread += later_count.reread;
+        }
+        self.panicked_states += later.panicked_states;
+        self.chose_nothing_with_a_readable_root += later.chose_nothing_with_a_readable_root;
+        self.chose_nothing_without_a_readable_root += later.chose_nothing_without_a_readable_root;
+        self.replay_failed_states += later.replay_failed_states;
+        self.replay_applied_states += later.replay_applied_states;
+        self.slot_readings_checked += later.slot_readings_checked;
+        self.slot_readings_differing_from_the_truth += later.slot_readings_differing_from_the_truth;
+        merge_the_first_witness(&mut self.slot_reading_witness, &later.slot_reading_witness);
+        self.kj0_control_violations += later.kj0_control_violations;
+        for rule in 0..FAULT_LOCAL_RULE_COUNT {
+            for criterion in 0..3 {
+                self.local_rules[rule][criterion] += later.local_rules[rule][criterion];
+            }
+            for fault in 0..5 {
+                self.local_kj1t_by_judged_fault[rule][fault] +=
+                    later.local_kj1t_by_judged_fault[rule][fault];
+            }
+        }
+        for criterion in 0..4 {
+            self.kf2[criterion] += later.kf2[criterion];
+        }
+        for criterion in 0..2 {
+            self.kf4[criterion] += later.kf4[criterion];
+        }
+        self.kf4_states_without_a_root_off_the_confirmed_chain +=
+            later.kf4_states_without_a_root_off_the_confirmed_chain;
+        self.kj1_and_kj1t_differing_states += later.kj1_and_kj1t_differing_states;
+        merge_the_first_witness(
+            &mut self.kj1_and_kj1t_differing_witness,
+            &later.kj1_and_kj1t_differing_witness,
+        );
+    }
+}
+
+/// 把工作单元分给几条线程跑，再按工作单元的次序合并（同 [`root_choice_tally_of_work_units`]，计数的种类不同）。
+fn fault_tally_of_work_units<Unit: Sync>(
+    units: &[Unit],
+    thread_count: usize,
+    work: &(dyn Fn(usize, &Unit) -> FaultTally + Sync),
+) -> FaultTally {
+    let next_unit_index = AtomicUsize::new(0);
+    let finished_units: Mutex<Vec<(usize, FaultTally)>> = Mutex::new(Vec::new());
+    std::thread::scope(|scope| {
+        for _worker in 0..thread_count.clamp(1, units.len().max(1)) {
+            scope.spawn(|| loop {
+                let unit_index = next_unit_index.fetch_add(1, Ordering::Relaxed);
+                let Some(unit) = units.get(unit_index) else {
+                    break;
+                };
+                let tally = work(unit_index, unit);
+                finished_units
+                    .lock()
+                    .expect("别的线程没有在拿着这把锁的时候 panic")
+                    .push((unit_index, tally));
+            });
+        }
+    });
+    let mut finished_units = finished_units
+        .into_inner()
+        .expect("线程都已经收回，没有谁在拿着这把锁的时候 panic");
+    finished_units.sort_by_key(|(unit_index, _tally)| *unit_index);
+    assert_eq!(finished_units.len(), units.len(), "每个工作单元恰好跑一遍");
+    let mut total = FaultTally::new();
+    for (_unit_index, tally) in &finished_units {
+        total.merge(tally);
+    }
+    total
+}
+
+labelled_enumeration! {
+    /// 本地写的择根序（登记第七节「本地择根序」那张表、9.1 六行替换文的比较式各写一份；落点 = 择到的根加理想重放，不调实现）。
+    /// 次序与那张表的列一致：今天的序，v1823 至 v1826，v1822；最后一个是 KF1 用的「取键最小的根」。
+    enum FaultLocalRule {
+        Today => "today",
+        IgnoresTheInstance => "v1823",
+        FirstSelfVerifiedWins => "v1824",
+        TieTakesTheLowerInstance => "v1825",
+        InstanceFirstThenTxg => "v1826",
+        TieTakesTheLater => "v1822",
+        OldestWins => "oldest",
+    }
+}
+
+impl FaultLocalRule {
+    /// 读根环次序里往后走，`candidate` 换掉已择的 `chosen` 吗。
+    fn replaces(self, candidate: RootVersion, chosen: RootVersion) -> bool {
+        match self {
+            Self::Today => {
+                candidate.checkpoint_txg > chosen.checkpoint_txg
+                    || (candidate.checkpoint_txg == chosen.checkpoint_txg
+                        && candidate.instance > chosen.instance)
+            }
+            Self::IgnoresTheInstance => candidate.checkpoint_txg > chosen.checkpoint_txg,
+            Self::FirstSelfVerifiedWins => false,
+            Self::TieTakesTheLowerInstance => {
+                candidate.checkpoint_txg > chosen.checkpoint_txg
+                    || (candidate.checkpoint_txg == chosen.checkpoint_txg
+                        && candidate.instance < chosen.instance)
+            }
+            Self::InstanceFirstThenTxg => {
+                candidate.instance > chosen.instance
+                    || (candidate.instance == chosen.instance
+                        && candidate.checkpoint_txg > chosen.checkpoint_txg)
+            }
+            Self::TieTakesTheLater => {
+                candidate.checkpoint_txg > chosen.checkpoint_txg
+                    || (candidate.checkpoint_txg == chosen.checkpoint_txg
+                        && candidate.instance >= chosen.instance)
+            }
+            Self::OldestWins => candidate < chosen,
+        }
+    }
+
+    fn chooses_among<'world>(
+        self,
+        roots_in_ring_order: &[Option<&'world RootChoicePublishMaterial>],
+    ) -> Option<&'world RootChoicePublishMaterial> {
+        let mut chosen: Option<&RootChoicePublishMaterial> = None;
+        for candidate in roots_in_ring_order.iter().copied().flatten() {
+            if chosen.is_none_or(|already_chosen| {
+                self.replaces(candidate.version, already_chosen.version)
+            }) {
+                chosen = Some(candidate);
+            }
+        }
+        chosen
+    }
+}
+
+/// 一个被判态判之前先算好的真相（判据一侧只用这些，不调 core 的判定函数）。
+struct FaultStateFacts {
+    last_confirmed: Option<RootVersion>,
+    newest_readable: Option<RootVersion>,
+    /// 条款的理想恢复（读视图里版本最大的根起）落到的六个字段。
+    ideal_fields: Option<VersionFields>,
+}
+
+/// 一条落点按各条判据读出来的结果；`Some(true)` 是违例，`Some(false)` 是判过没违例，`None` 是这一态不判这一条。
+#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
+struct FaultCriteriaReading {
+    kj1: Option<bool>,
+    kj1t: Option<bool>,
+    kj2: Option<bool>,
+    kj3: Option<bool>,
+}
+
+/// 落点 `landed_version` 按登记 5.5 的四条判据读：KJ1 版本序不早于最后一次已确认的发布；KJ1T 落在它本身或后代上（父链，登记 5.3 第 7 条）；
+/// KJ2 不早于读视图里任一读得出的根；KJ3 六个字段等于条款的理想恢复。
+fn fault_criteria_of_a_landing(
+    world: &FaultWorld,
+    facts: &FaultStateFacts,
+    landed_version: RootVersion,
+    landed_fields: Option<VersionFields>,
+) -> FaultCriteriaReading {
+    FaultCriteriaReading {
+        kj1: facts.last_confirmed.map(|last| landed_version < last),
+        kj1t: facts
+            .last_confirmed
+            .map(|last| !world.is_the_version_or_a_descendant_of(landed_version, last)),
+        kj2: facts.newest_readable.map(|newest| landed_version < newest),
+        kj3: match (facts.ideal_fields, landed_fields) {
+            (Some(ideal), Some(fields)) => Some(fields != ideal),
+            (None, _) | (_, None) => None,
+        },
+    }
+}
+
+/// 一个被判态的结局在每条判据上的读数：真实现的结局或阳性对照的假结局都走这一段。
+fn count_a_criteria_reading_into_a_cell(
+    reading: FaultCriteriaReading,
+    is_cut_by_the_limit: bool,
+    cell: &mut FaultCellTally,
+) -> [bool; FAULT_CRITERION_COUNT] {
+    let mut violated = [false; FAULT_CRITERION_COUNT];
+    for (criterion_index, verdict) in [
+        (KJ1_INDEX, reading.kj1),
+        (KJ1T_INDEX, reading.kj1t),
+        (KJ2_INDEX, reading.kj2),
+        (KJ3_INDEX, reading.kj3),
+    ] {
+        if let Some(is_violation) = verdict {
+            cell.judged[criterion_index] += 1;
+            if is_violation {
+                cell.violations[criterion_index] += 1;
+                if is_cut_by_the_limit {
+                    cell.cut_violations[criterion_index] += 1;
+                }
+                violated[criterion_index] = true;
+            }
+        }
+    }
+    violated
+}
+
+/// 一个被判态上三步的结局（只在 `run_the_three_steps` 时算；单测里为省时可以关掉）。
+fn outcome_of_the_three_steps_on_a_faulted_disk(
+    disk: &FaultedRootChoiceDisk<'_, '_>,
+    system_configuration: &SystemConfiguration,
+) -> RootChoiceOutcome {
+    IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(true));
+    let outcome = outcome_of_the_three_steps_with_the_panic_hook_quiet(disk, system_configuration);
+    IS_INSIDE_THE_REPLAY_CALL.with(|is_inside| is_inside.set(false));
+    outcome
+}
+
+fn outcome_of_the_three_steps_with_the_panic_hook_quiet(
+    disk: &FaultedRootChoiceDisk<'_, '_>,
+    system_configuration: &SystemConfiguration,
+) -> RootChoiceOutcome {
+    let chosen_root =
+        match panic::catch_unwind(AssertUnwindSafe(|| choose_root(disk, system_configuration))) {
+            Err(_panic_payload) => {
+                return RootChoiceOutcome::Panicked {
+                    step: "choose_root",
+                }
+            }
+            Ok(None) => return RootChoiceOutcome::ChoseNothing,
+            Ok(Some(root)) => root,
+        };
+    let records = match panic::catch_unwind(AssertUnwindSafe(|| {
+        scan_journal(disk, system_configuration)
+    })) {
+        Err(_panic_payload) => {
+            return RootChoiceOutcome::Panicked {
+                step: "scan_journal",
+            }
+        }
+        Ok(records) => records,
+    };
+    let replay = panic::catch_unwind(AssertUnwindSafe(|| {
+        replay_journal(
+            disk,
+            &chosen_root,
+            system_configuration.immutable.sizes.journal_ring_bytes,
+            &records,
+            true,
+        )
+    }));
+    match replay {
+        Err(_panic_payload) => RootChoiceOutcome::Panicked {
+            step: "replay_journal",
+        },
+        Ok(Err(_failure)) => RootChoiceOutcome::Landed {
+            chosen: RootVersion::of_root(&chosen_root),
+            landed_version: RootVersion::of_root(&chosen_root),
+            landed_fields: VersionFields::of_root(&chosen_root),
+            replay_failed: true,
+        },
+        Ok(Ok((_report, returned_root))) => RootChoiceOutcome::Landed {
+            chosen: RootVersion::of_root(&chosen_root),
+            landed_version: RootVersion::of_root(&returned_root),
+            landed_fields: VersionFields::of_root(&returned_root),
+            replay_failed: false,
+        },
+    }
+}
+
+/// 读槽对拍（停机 TH4 的输入，登记 5.5 末段）：被判时故障的落点必须读成 `Bad(Unreadable)`；别的槽读成自证过的，它的 (txg, 实例代号) 等于真相；
+/// 全 0 的槽读成自证不过、而且真相里这个槽是空的。
+fn fault_slot_reading_agrees_with_the_truth(
+    reading: RootRingSlotReading,
+    truth: Option<&RootChoicePublishMaterial>,
+    is_unreadable_in_this_view: bool,
+) -> bool {
+    if is_unreadable_in_this_view {
+        return matches!(
+            reading,
+            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable)
+        );
+    }
+    root_slot_reading_agrees_with_the_truth(reading, truth)
+}
+
+/// 逐槽调 `read_root_ring_slot`，与真相对拍；不一致数一笔、记第一个见证，收尾走停机条款 TH4。
+fn check_the_fault_slot_readings_against_the_truth(
+    disk: &FaultedRootChoiceDisk<'_, '_>,
+    system_configuration: &SystemConfiguration,
+    world: &FaultWorld,
+    tally: &mut FaultTally,
+    describe_state: &dyn Fn() -> String,
+) {
+    tally.slot_readings_checked += 1;
+    let immutable = &system_configuration.immutable;
+    let slots_per_region = world.slots_per_region();
+    for ring_slot in every_root_ring_slot(&immutable.sizes) {
+        let reading = read_root_ring_slot(
+            disk,
+            &immutable.region_devices,
+            &immutable.sizes,
+            &immutable.filesystem_identifier,
+            ring_slot,
+        );
+        let slot_index =
+            root_choice_slot_index_of(ring_slot.region, ring_slot.slot, slots_per_region);
+        let truth = world.root_slots[slot_index].as_deref();
+        let is_unreadable = disk
+            .locations
+            .root_slot_is_unreadable(slot_index, slots_per_region);
+        if !fault_slot_reading_agrees_with_the_truth(reading, truth, is_unreadable) {
+            tally.slot_readings_differing_from_the_truth += 1;
+            if tally.slot_reading_witness.is_none() {
+                tally.slot_reading_witness = Some(format!(
+                    "region{}_slot{}_read_as={reading:?}_truth={}_unreadable_in_the_view={is_unreadable};{}",
+                    ring_slot.region,
+                    ring_slot.slot,
+                    truth.map_or_else(|| "empty".to_string(), |material| material.version.describe()),
+                    describe_state()
+                ));
+            }
+        }
+    }
+}
+
+fn describe_a_system_slot(content: SystemSlotContent) -> String {
+    match content {
+        SystemSlotContent::Torn => "torn".to_string(),
+        SystemSlotContent::Verified {
+            generation,
+            tail,
+            instance,
+        } => format!("g{generation}t{tail}i{instance}"),
+    }
+}
+
+/// 一段历史的写法（见证用）：每次挂载开头的故障与走了哪一遍、写行 txg，开始过的每次发布，系统配置槽，最后一次的崩溃点。
+fn describe_a_fault_history(world: &FaultWorld) -> String {
+    let mounts: Vec<String> = world
+        .mount_log
+        .iter()
+        .enumerate()
+        .map(|(position, entry)| {
+            format!(
+                "mount{}:fault={}:{:?}:row_write_txg={}",
+                position + 1,
+                entry.fault.label(),
+                entry.outcome,
+                entry.row_write_txg
+            )
+        })
+        .collect();
+    let publishes: Vec<String> = world
+        .started_publishes
+        .iter()
+        .skip(1)
+        .map(|material| {
+            format!(
+                "txg{}i{}k{}{}",
+                material.version.checkpoint_txg,
+                material.version.instance,
+                material.records.len(),
+                if material.is_user_publish { "u" } else { "o" }
+            )
+        })
+        .collect();
+    let system_slots: Vec<String> = world
+        .system_slots
+        .iter()
+        .map(|slots| {
+            slots
+                .iter()
+                .map(|content| describe_a_system_slot(*content))
+                .collect::<Vec<String>>()
+                .join("/")
+        })
+        .collect();
+    let slots_per_region = usize::try_from(world.slots_per_region()).expect("每区槽数 4 或 8");
+    let roots: Vec<String> = world
+        .root_slots
+        .iter()
+        .enumerate()
+        .filter_map(|(root_slot_index, root)| {
+            root.as_ref().map(|material| {
+                format!(
+                    "region{}slot{}={}",
+                    root_slot_index / slots_per_region,
+                    root_slot_index % slots_per_region,
+                    material.version.describe()
+                )
+            })
+        })
+        .collect();
+    let confirmed: Vec<String> = world
+        .confirmed_user_publishes
+        .iter()
+        .map(|version| version.describe())
+        .collect();
+    let mut chain_of_the_last_confirmed = Vec::new();
+    let mut current = world.confirmed_user_publishes.iter().copied().max();
+    while let Some(version) = current {
+        chain_of_the_last_confirmed.push(version.describe());
+        current = world.parent_of(version);
+        if chain_of_the_last_confirmed.len() > world.parents.len() {
+            break;
+        }
+    }
+    format!(
+        "{};publishes={};roots_in_ring_order={};confirmed={};parent_chain_of_the_last_confirmed={};system_slots_d0|d1={};{}",
+        mounts.join("+"),
+        publishes.join(","),
+        roots.join("+"),
+        confirmed.join(","),
+        chain_of_the_last_confirmed.join("<-"),
+        system_slots.join("|"),
+        describe_a_fault_crash(world.final_crash)
+    )
+}
+
+fn describe_a_fault_crash(crash: Option<FaultCrashLabel>) -> String {
+    let Some(label) = crash else {
+        return "no_crash_the_last_mount_was_refused".to_string();
+    };
+    let point_text = match label.point {
+        FaultCrashPoint::RecordCopiesLanded { copy_mask } => {
+            format!("record_copies_mask_{copy_mask:b}_root_not_landed")
+        }
+        FaultCrashPoint::RootLanded { system_writes } => {
+            format!("root_landed_system_writes_{system_writes:?}")
+        }
+    };
+    format!(
+        "crash_in_publish_txg{}_instance{}_{}records_{point_text}",
+        label.version.checkpoint_txg, label.version.instance, label.record_count
+    )
+}
+
+// ── 判一个最后一次挂载走完的历史：5 种被判时故障各判一次（登记 5.3 第 11 条）
+
+struct FaultJudgement {
+    domain: FaultDomain,
+    /// 这个格的 M（M = 2 的格按挂载 2 开头那项故障另报）。
+    mounts: usize,
+    system_configuration: SystemConfiguration,
+    in_flight_limit: usize,
+    /// 关掉时不调三步（单测里为省时；产物里恒开）。
+    run_the_three_steps: bool,
+}
+
+/// 一段历史上与被判时故障无关的真相：每个被判态共用。
+struct FaultHistoryFacts {
+    is_inside_the_single_fault_promise: bool,
+    last_confirmed: Option<RootVersion>,
+    parent_of_the_last_confirmed: Option<RootVersion>,
+    /// 根环真相里不是最后一次已确认那一版、也不是它后代的根里键最大的那一条（KF4 的落点）。
+    kf4_landing: Option<RootVersion>,
+    has_same_txg_different_instance_roots: bool,
+    has_higher_instance_lower_txg_roots: bool,
+    has_a_torn_system_slot: bool,
+}
+
+fn facts_of_a_fault_history(world: &FaultWorld, ring_slots: u64) -> FaultHistoryFacts {
+    let versions_in_the_true_ring: Vec<RootVersion> = world
+        .root_slots
+        .iter()
+        .flatten()
+        .map(|material| material.version)
+        .collect();
+    let last_confirmed = world.confirmed_user_publishes.iter().copied().max();
+    if let Some(last) = last_confirmed {
+        for confirmed in &world.confirmed_user_publishes {
+            assert!(
+                world.is_the_version_or_a_descendant_of(last, *confirmed),
+                "停机条款 TH9：已确认的几次发布不在一条父链上（{}、{}）",
+                last.describe(),
+                confirmed.describe()
+            );
+        }
+    }
+    let largest_counter = world
+        .landed_records
+        .iter()
+        .map(|landed| landed.counter)
+        .max()
+        .unwrap_or(0);
+    assert!(
+        largest_counter <= ring_slots,
+        "停机条款 TH9：界内最大计数器 {largest_counter} 超过环槽数 {ring_slots}，journal 环回绕了"
+    );
+    let mut has_same_txg_different_instance_roots = false;
+    let mut has_higher_instance_lower_txg_roots = false;
+    for earlier in &versions_in_the_true_ring {
+        for later in &versions_in_the_true_ring {
+            has_same_txg_different_instance_roots |= earlier.checkpoint_txg == later.checkpoint_txg
+                && earlier.instance != later.instance;
+            has_higher_instance_lower_txg_roots |=
+                earlier.instance > later.instance && earlier.checkpoint_txg < later.checkpoint_txg;
+        }
+    }
+    FaultHistoryFacts {
+        is_inside_the_single_fault_promise: world
+            .mount_log
+            .iter()
+            .all(|entry| entry.fault.is_inside_the_single_fault_promise()),
+        last_confirmed,
+        parent_of_the_last_confirmed: last_confirmed.and_then(|last| world.parent_of(last)),
+        kf4_landing: last_confirmed.and_then(|last| {
+            versions_in_the_true_ring
+                .iter()
+                .copied()
+                .filter(|version| !world.is_the_version_or_a_descendant_of(*version, last))
+                .max()
+        }),
+        has_same_txg_different_instance_roots,
+        has_higher_instance_lower_txg_roots,
+        has_a_torn_system_slot: world
+            .system_slots
+            .iter()
+            .flatten()
+            .any(|content| *content == SystemSlotContent::Torn),
+    }
+}
+
+impl FaultJudgement {
+    fn new(domain: FaultDomain, mounts: usize, run_the_three_steps: bool) -> Self {
+        Self {
+            domain,
+            mounts,
+            system_configuration: system_configuration_for_the_root_choice(domain.geometry()),
+            in_flight_limit: domain.geometry().ring().local_in_flight_limit(),
+            run_the_three_steps,
+        }
+    }
+
+    /// 一段历史：先记与被判时故障无关的计数，再对 5 种被判时故障各判一个状态。
+    fn judge_a_history(
+        &self,
+        world: &FaultWorld,
+        is_the_designated_history: bool,
+        tally: &mut FaultTally,
+    ) {
+        let history = facts_of_a_fault_history(world, self.domain.geometry().ring().ring_slots);
+        tally.histories += 1;
+        tally.refused_mounts += world.refused_mounts;
+        tally.reread_mounts += world.reread_mounts;
+        tally.largest_counter = tally.largest_counter.max(
+            world
+                .landed_records
+                .iter()
+                .map(|landed| landed.counter)
+                .max()
+                .unwrap_or(0),
+        );
+        tally.confirmed_publishes_maximum = tally
+            .confirmed_publishes_maximum
+            .max(world.confirmed_user_publishes.len());
+        if !history.is_inside_the_single_fault_promise {
+            tally.histories_with_an_uncovered_fault += 1;
+        }
+        if self.mounts == 2 {
+            assert_eq!(world.mount_log.len(), 2, "M = 2 的历史恰好开始过两次挂载");
+            let count = &mut tally.mount_two[world.mount_log[1].fault.menu_index];
+            count.histories += 1;
+            count.refused += world.refused_mounts;
+            count.reread += world.reread_mounts;
+        }
+        let truth = fault_landing_of_a_view(world, FaultLocations::NOWHERE);
+        let disk = RootChoiceDisk::of_the_landing(self.domain.geometry(), &truth);
+        for judged_fault in JudgedFault::ALL {
+            let is_the_designated_state =
+                is_the_designated_history && *judged_fault == JudgedFault::Nothing;
+            self.judge_a_state(
+                world,
+                &history,
+                &disk,
+                (*judged_fault, is_the_designated_state),
+                tally,
+            );
+        }
+    }
+}
+
+impl FaultJudgement {
+    /// 一个被判态：读视图 = 真相的盘减去被判时故障的落点。先记与实现无关的计数与分类，再算本地择根序与 KF2、KF4 的假结局读数，最后调三步判四条判据。
+    fn judge_a_state(
+        &self,
+        world: &FaultWorld,
+        history: &FaultHistoryFacts,
+        disk: &RootChoiceDisk<'_>,
+        (judged_fault, is_the_designated_state): (JudgedFault, bool),
+        tally: &mut FaultTally,
+    ) {
+        let locations = judged_fault.locations(world);
+        let view = fault_landing_of_a_view(world, locations);
+        let describe_state = || {
+            format!(
+                "judged_fault={};history_inside_the_promise={};{}",
+                judged_fault.label(),
+                history.is_inside_the_single_fault_promise,
+                describe_a_fault_history(world)
+            )
+        };
+        tally.judged_states += 1;
+        tally.states_with_a_torn_system_slot += u64::from(history.has_a_torn_system_slot);
+        tally.states_with_same_txg_different_instance_roots +=
+            u64::from(history.has_same_txg_different_instance_roots);
+        tally.states_with_higher_instance_lower_txg_roots +=
+            u64::from(history.has_higher_instance_lower_txg_roots);
+        let newest_root = root_choice_newest_root(&view);
+        match newest_root {
+            None => tally.states_without_a_readable_root += 1,
+            Some(newest) => {
+                let above_newest_root = view
+                    .records
+                    .iter()
+                    .filter(|record| {
+                        record.material.version.instance == newest.version.instance
+                            && record.material.version.checkpoint_txg
+                                > newest.version.checkpoint_txg
+                    })
+                    .count();
+                tally.above_newest_root_maximum =
+                    tally.above_newest_root_maximum.max(above_newest_root);
+            }
+        }
+        let ideal = newest_root.map(|newest| {
+            root_choice_ideal_recovery_from(newest, &view.records, self.in_flight_limit)
+        });
+        let facts = FaultStateFacts {
+            last_confirmed: history.last_confirmed,
+            newest_readable: newest_root.map(|newest| newest.version),
+            ideal_fields: ideal.as_ref().map(|recovery| recovery.landed_fields),
+        };
+        let cell_index =
+            fault_cell_index_of(judged_fault, history.is_inside_the_single_fault_promise);
+        tally.cells[cell_index].states += 1;
+        let is_cut_by_the_limit = self.classify_the_ideal_recovery(
+            world,
+            history,
+            &view,
+            newest_root,
+            &ideal,
+            judged_fault,
+            cell_index,
+            tally,
+        );
+        if is_cut_by_the_limit {
+            tally.cells[cell_index].cut_states += 1;
+        }
+        let is_inside_the_promise = history.is_inside_the_single_fault_promise
+            && judged_fault.is_inside_the_single_fault_promise()
+            && !is_cut_by_the_limit;
+        if is_inside_the_promise {
+            tally.promised_states += 1;
+        }
+        self.read_the_local_rules_and_the_fake_outcomes(
+            world,
+            history,
+            &view,
+            &facts,
+            judged_fault,
+            tally,
+        );
+        if !self.run_the_three_steps {
+            return;
+        }
+        if is_the_designated_state {
+            let mut scratch = FaultTally::new();
+            self.count_the_outcome(
+                world,
+                &facts,
+                &RootChoiceOutcome::Panicked {
+                    step: "designated_state",
+                },
+                (cell_index, false, false),
+                &describe_state,
+                &mut scratch,
+            );
+            tally.kj0_control_violations += scratch.violations_of_a_criterion(0);
+        }
+        let faulted_disk = FaultedRootChoiceDisk {
+            inner: disk,
+            locations,
+            slots_per_region: world.slots_per_region(),
+        };
+        check_the_fault_slot_readings_against_the_truth(
+            &faulted_disk,
+            &self.system_configuration,
+            world,
+            tally,
+            &describe_state,
+        );
+        let outcome =
+            outcome_of_the_three_steps_on_a_faulted_disk(&faulted_disk, &self.system_configuration);
+        let violated = self.count_the_outcome(
+            world,
+            &facts,
+            &outcome,
+            (cell_index, is_cut_by_the_limit, newest_root.is_some()),
+            &describe_state,
+            tally,
+        );
+        if is_inside_the_promise {
+            for (position, criterion_index) in [KJ1_INDEX, KJ1T_INDEX].into_iter().enumerate() {
+                if violated[criterion_index] {
+                    tally.promised_violations[position] += 1;
+                    if tally.promised_witnesses[position].is_none() {
+                        tally.promised_witnesses[position] = Some(describe_state());
+                    }
+                }
+            }
+        }
+    }
+}
+
+impl FaultJudgement {
+    /// 有已确认的被判态：数一笔，再分类条款的理想恢复（读视图里版本最大的根起）丢没丢最后一次已确认的发布：
+    /// 去掉在飞上限就不丢 ⇒ 上限截掉；否则被判时与历史都罩着 ⇒ 罩着的其余；否则不罩。交回「是上限截掉」。
+    #[allow(
+        clippy::too_many_arguments,
+        reason = "一个被判态的分类要同时看历史、读视图、理想恢复与两个标签，拆开只是把同一组量再传一遍"
+    )]
+    fn classify_the_ideal_recovery(
+        &self,
+        world: &FaultWorld,
+        history: &FaultHistoryFacts,
+        view: &RootChoiceLanding<'_>,
+        newest_root: Option<&RootChoicePublishMaterial>,
+        ideal: &Option<RootChoiceIdealRecovery>,
+        judged_fault: JudgedFault,
+        cell_index: usize,
+        tally: &mut FaultTally,
+    ) -> bool {
+        let Some(last) = history.last_confirmed else {
+            return false;
+        };
+        tally.confirmed_states += 1;
+        tally.cells[cell_index].confirmed_states += 1;
+        let (Some(newest), Some(ideal_recovery)) = (newest_root, ideal) else {
+            return false;
+        };
+        let ideal_version = ideal_recovery.landed_version;
+        let ideal_is_earlier = ideal_version < last;
+        let ideal_is_off_the_chain = !world.is_the_version_or_a_descendant_of(ideal_version, last);
+        tally.cells[cell_index].ideal_violations[0] += u64::from(ideal_is_earlier);
+        tally.cells[cell_index].ideal_violations[1] += u64::from(ideal_is_off_the_chain);
+        if !(ideal_is_earlier || ideal_is_off_the_chain) {
+            return false;
+        }
+        let without_the_limit =
+            root_choice_ideal_recovery_from(newest, &view.records, FAULT_IN_FLIGHT_LIMIT_REMOVED)
+                .landed_version;
+        let is_cut_by_the_limit = without_the_limit >= last
+            && world.is_the_version_or_a_descendant_of(without_the_limit, last);
+        if is_cut_by_the_limit {
+            tally.ideal_loss_by_limit += 1;
+        } else if history.is_inside_the_single_fault_promise
+            && judged_fault.is_inside_the_single_fault_promise()
+        {
+            tally.ideal_loss_covered_other += 1;
+        } else {
+            tally.ideal_loss_uncovered += 1;
+        }
+        is_cut_by_the_limit
+    }
+
+    /// 本地择根序加理想重放的七种读数（登记第七节「本地择根序」那张表；下标 6 是 KF1），KF2（落在最后一次已确认那一版的父上）与 KF4（落在根环真相里
+    /// 不在已确认那条链上的根中键最大的那条）的假结局读数。全部走 [`fault_criteria_of_a_landing`]，与真实现的结局读同一段判据。
+    fn read_the_local_rules_and_the_fake_outcomes(
+        &self,
+        world: &FaultWorld,
+        history: &FaultHistoryFacts,
+        view: &RootChoiceLanding<'_>,
+        facts: &FaultStateFacts,
+        judged_fault: JudgedFault,
+        tally: &mut FaultTally,
+    ) {
+        for (rule_index, rule) in FaultLocalRule::ALL.iter().enumerate() {
+            let Some(chosen) = rule.chooses_among(&view.roots_in_ring_order) else {
+                continue;
+            };
+            let landed_version =
+                root_choice_ideal_recovery_from(chosen, &view.records, self.in_flight_limit)
+                    .landed_version;
+            let reading = fault_criteria_of_a_landing(world, facts, landed_version, None);
+            for (position, verdict) in [reading.kj1, reading.kj1t, reading.kj2]
+                .into_iter()
+                .enumerate()
+            {
+                tally.local_rules[rule_index][position] += u64::from(verdict == Some(true));
+            }
+            tally.local_kj1t_by_judged_fault[rule_index][judged_fault.index()] +=
+                u64::from(reading.kj1t == Some(true));
+        }
+        if history.last_confirmed.is_none() {
+            return;
+        }
+        if let Some(parent) = history.parent_of_the_last_confirmed {
+            let reading = fault_criteria_of_a_landing(
+                world,
+                facts,
+                parent,
+                world.fields_of_a_version(parent),
+            );
+            tally.kf2[0] += u64::from(reading.kj1 == Some(true));
+            tally.kf2[1] += u64::from(reading.kj1t == Some(true));
+            tally.kf2[2] += u64::from(reading.kj2 == Some(true));
+            // 读视图里一条根都没有时没有理想恢复可比：假结局落在一版上，照 KJ3 的字面算违例。
+            tally.kf2[3] += u64::from(reading.kj3.unwrap_or(facts.ideal_fields.is_none()));
+        }
+        match history.kf4_landing {
+            Some(landing) => {
+                let reading = fault_criteria_of_a_landing(world, facts, landing, None);
+                tally.kf4[0] += u64::from(reading.kj1 == Some(true));
+                tally.kf4[1] += u64::from(reading.kj1t == Some(true));
+            }
+            None => tally.kf4_states_without_a_root_off_the_confirmed_chain += 1,
+        }
+    }
+}
+
+impl FaultJudgement {
+    /// 一个被判态的结局落进判据（各报各的，不合成一个真假）：真实现的结局与阳性对照的假结局走这一段。
+    /// `(cell_index, is_cut_by_the_limit, a_root_is_readable)`：这一态落在哪个格、是不是上限截掉、读视图里有没有读得出的根。
+    /// 交回每条判据这一态是不是违例（承诺内那一行用）。
+    fn count_the_outcome(
+        &self,
+        world: &FaultWorld,
+        facts: &FaultStateFacts,
+        outcome: &RootChoiceOutcome,
+        (cell_index, is_cut_by_the_limit, a_root_is_readable): (usize, bool, bool),
+        describe_state: &dyn Fn() -> String,
+        tally: &mut FaultTally,
+    ) -> [bool; FAULT_CRITERION_COUNT] {
+        let mut violated = [false; FAULT_CRITERION_COUNT];
+        tally.cells[cell_index].judged[0] += 1;
+        match outcome {
+            RootChoiceOutcome::Panicked { step } => {
+                tally.panicked_states += 1;
+                self.add_a_kj0_violation(cell_index, is_cut_by_the_limit, tally, &|| {
+                    format!("panicked_in={step};{}", describe_state())
+                });
+                violated[0] = true;
+            }
+            RootChoiceOutcome::ChoseNothing => {
+                if a_root_is_readable {
+                    tally.chose_nothing_with_a_readable_root += 1;
+                    self.add_a_kj0_violation(cell_index, is_cut_by_the_limit, tally, &|| {
+                        format!(
+                            "choose_root_returned_none_with_a_readable_root;{}",
+                            describe_state()
+                        )
+                    });
+                    violated[0] = true;
+                } else {
+                    tally.chose_nothing_without_a_readable_root += 1;
+                }
+            }
+            RootChoiceOutcome::Landed {
+                chosen,
+                landed_version,
+                landed_fields,
+                replay_failed,
+            } => {
+                tally.replay_failed_states += u64::from(*replay_failed);
+                tally.replay_applied_states += u64::from(landed_version != chosen);
+                let reading = fault_criteria_of_a_landing(
+                    world,
+                    facts,
+                    *landed_version,
+                    Some(*landed_fields),
+                );
+                let counted = count_a_criteria_reading_into_a_cell(
+                    reading,
+                    is_cut_by_the_limit,
+                    &mut tally.cells[cell_index],
+                );
+                for criterion in FaultCriterion::ALL {
+                    let criterion_index = criterion.index();
+                    if criterion_index == 0 || !counted[criterion_index] {
+                        continue;
+                    }
+                    violated[criterion_index] = true;
+                    let witness =
+                        &mut tally.witnesses[cell_index * FAULT_CRITERION_COUNT + criterion_index];
+                    if witness.is_none() {
+                        *witness = Some(format!(
+                            "{};chosen={};landed={};last_confirmed={};newest_readable={};{}",
+                            criterion.label(),
+                            chosen.describe(),
+                            landed_version.describe(),
+                            facts
+                                .last_confirmed
+                                .map_or_else(|| "none".to_string(), RootVersion::describe),
+                            facts
+                                .newest_readable
+                                .map_or_else(|| "none".to_string(), RootVersion::describe),
+                            describe_state()
+                        ));
+                    }
+                }
+                if reading.kj1.is_some() && reading.kj1 != reading.kj1t {
+                    tally.kj1_and_kj1t_differing_states += 1;
+                    if tally.kj1_and_kj1t_differing_witness.is_none() {
+                        tally.kj1_and_kj1t_differing_witness = Some(format!(
+                            "kj1_violated={:?};kj1t_violated={:?};landed={};{}",
+                            reading.kj1,
+                            reading.kj1t,
+                            landed_version.describe(),
+                            describe_state()
+                        ));
+                    }
+                }
+            }
+        }
+        violated
+    }
+
+    fn add_a_kj0_violation(
+        &self,
+        cell_index: usize,
+        is_cut_by_the_limit: bool,
+        tally: &mut FaultTally,
+        describe_witness: &dyn Fn() -> String,
+    ) {
+        tally.add_cell_violation(cell_index, 0, describe_witness);
+        if is_cut_by_the_limit {
+            tally.cells[cell_index].cut_violations[0] += 1;
+        }
+    }
+}
+
+// ── 一个格的计数：枚举、判、按工作单元分给几条线程
+
+/// 工作单元：挂载 M 开始之前的盘，再加故障菜单里的一项。前面几次挂载的全部走法在崩溃之后得到的盘，每个单元接着枚举最后一次挂载的一项故障。
+struct FaultUnit {
+    world: FaultWorld,
+    mount_number: usize,
+    user_publishes_left: usize,
+    menu_index: usize,
+}
+
+struct FaultUnitCollector {
+    units: Vec<FaultUnit>,
+}
+
+impl FaultVisitor for FaultUnitCollector {
+    fn reach_the_start_of_a_mount(
+        &mut self,
+        world: &FaultWorld,
+        mount_number: usize,
+        user_publishes_left: usize,
+    ) {
+        for entry in mount_fault_menu(world, mount_number) {
+            self.units.push(FaultUnit {
+                world: world.clone(),
+                mount_number,
+                user_publishes_left,
+                menu_index: entry.menu_index,
+            });
+        }
+    }
+
+    fn visit_a_final_state(&mut self, _world: &FaultWorld) {
+        panic!("收工作单元的那一遍停在最后一次挂载开始之前，走不到最后一次挂载的崩溃点");
+    }
+}
+
+struct FaultStateJudge<'run> {
+    judgement: &'run FaultJudgement,
+    tally: FaultTally,
+    histories_seen_in_this_unit: u64,
+    is_the_first_unit: bool,
+}
+
+impl FaultVisitor for FaultStateJudge<'_> {
+    fn reach_the_start_of_a_mount(
+        &mut self,
+        _world: &FaultWorld,
+        _mount_number: usize,
+        _user_publishes_left: usize,
+    ) {
+        panic!("判状态的那一遍不设 stop_before_mount");
+    }
+
+    fn visit_a_final_state(&mut self, world: &FaultWorld) {
+        let is_the_designated_history =
+            self.is_the_first_unit && self.histories_seen_in_this_unit == 0;
+        self.histories_seen_in_this_unit += 1;
+        self.judgement
+            .judge_a_history(world, is_the_designated_history, &mut self.tally);
+    }
+}
+
+/// 一个 (域, M, U) 格上全部历史的计数。合法历史与 KG 按工作单元分给几条线程；KA 不走这里。
+fn fault_tally_of_a_cell(
+    domain: FaultDomain,
+    cell: RootChoiceCell,
+    run_the_three_steps: bool,
+    thread_count: usize,
+) -> FaultTally {
+    let collecting = FaultEnumeration {
+        domain,
+        mounts: cell.mounts,
+        stop_before_mount: Some(cell.mounts),
+    };
+    let mut collector = FaultUnitCollector { units: Vec::new() };
+    collecting.enumerate_a_mount(
+        &FaultWorld::right_after_mkfs(domain.geometry()),
+        1,
+        cell.user_publishes,
+        None,
+        &mut collector,
+    );
+    let judgement = FaultJudgement::new(domain, cell.mounts, run_the_three_steps);
+    let continuing = FaultEnumeration {
+        domain,
+        mounts: cell.mounts,
+        stop_before_mount: None,
+    };
+    fault_tally_of_work_units(&collector.units, thread_count, &|unit_index, unit| {
+        let mut judge = FaultStateJudge {
+            judgement: &judgement,
+            tally: FaultTally::new(),
+            histories_seen_in_this_unit: 0,
+            is_the_first_unit: unit_index == 0,
+        };
+        continuing.enumerate_a_mount(
+            &unit.world,
+            unit.mount_number,
+            unit.user_publishes_left,
+            Some(unit.menu_index),
+            &mut judge,
+        );
+        judge.tally
+    })
+}
+
+/// 登记第七节乙类（出自设计员的独立生成器 `anchors_r4.py`，输出 `anchors_r4.out` sha256 5a36b72237f84c52e6901ae5affcfc5431fd4c8b123f5b9e476907b749ba1fc7）：
+/// 每行 = 域 M U，然后依次是：历史数、历史里有不罩故障的历史数、被判态数、有已确认的被判态数、被拒的挂载数、走了重读的挂载数、带撕裂系统配置槽的被判态数、
+/// 同 txg 不同实例同在的被判态数、实例高而 txg 低同在的被判态数、视图里没有根的被判态数、视图里最大根之上的记录条数最大值、
+/// 条款理想恢复丢的被判态按因在飞上限 / 罩着的其余 / 不罩三类的数；KF1 的 KJ1 KJ1T KJ2；KF2 的 KJ1 KJ1T KJ2 KJ3；KF4 的 KJ1 KJ1T；
+/// 本地择根序 today v1823 v1824 v1825 v1826 v1822 各自的 KJ1 KJ1T KJ2。
+const PINNED_FAULT_CELLS_TEXT: &str = "\
+KF 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
+KF 1 1 39 0 195 0 0 0 0 0 0 0 3 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0\n\
+KF 1 2 117 0 585 585 0 0 0 0 0 0 5 0 0 117 468 468 351 585 585 252 585 585 585 117 117 0 117 117 0 468 468 351 117 117 0 117 117 0 117 117 0\n\
+KF 1 3 351 0 1755 1755 0 0 0 0 0 0 4 0 0 351 1404 1404 1053 1755 1755 756 1755 1755 1755 351 351 0 351 351 0 1404 1404 1053 351 351 0 351 351 0 351 351 0\n\
+KF 2 0 1792 236 8960 0 212 1020 650 0 0 0 2 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0\n\
+KF 2 1 9401 1218 47005 0 647 5598 1625 0 0 0 3 0 0 0 0 0 40494 0 0 0 0 0 0 0 0 0 0 0 0 0 0 40494 0 0 0 0 0 0 0 0 0\n\
+KF 2 2 48590 6207 242950 164835 1952 29529 4550 0 0 0 5 0 0 20655 145557 145557 213495 164835 164835 99999 164835 164835 164835 20655 20655 0 20655 20655 0 145557 145557 213495 20655 20655 0 20655 20655 0 20655 20655 0\n\
+KF 2 3 175589 22353 877945 874710 5867 107217 13325 0 0 0 6 0 0 108198 785763 785763 804138 874710 874710 576648 874710 874710 874710 108198 108198 0 108198 108198 0 785763 785763 804138 108198 108198 0 108198 108198 0 108198 108198 0\n\
+KF3 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
+KF3 1 1 5 0 25 0 0 0 0 0 0 0 2 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0\n\
+KF3 1 2 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
+KF3 1 3 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
+KF3 2 0 1792 236 8960 0 212 1020 650 0 0 0 2 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0\n\
+KF3 2 1 1587 214 7935 0 317 810 975 0 0 0 2 0 0 0 0 0 6254 0 0 0 0 0 0 0 0 0 0 0 0 0 0 6254 0 0 0 0 0 0 0 0 0\n\
+KF3 2 2 2292 307 11460 8275 422 1185 1300 0 0 0 3 0 0 991 7205 7205 9049 8275 8275 5623 8275 8275 8275 991 991 0 991 991 0 7205 7205 9049 991 991 0 991 991 0 991 991 0\n\
+KF3 2 3 2477 335 12385 10800 527 1235 1625 0 0 0 3 0 0 1416 9545 9545 9964 10800 10800 7520 10800 10800 10800 1416 1416 0 1416 1416 0 9545 9545 9964 1416 1416 0 1416 1416 0 1416 1416 0\n\
+KF3 3 0 275939 67845 1379695 0 46096 334980 111150 0 0 0 2 0 0 0 0 0 1091126 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1091126 0 0 0 0 0 0 0 0 0\n\
+KF3 3 1 350403 86331 1752015 0 73710 411175 197925 0 0 0 2 0 0 0 0 0 1633188 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1633188 0 0 0 0 0 0 0 0 0\n\
+KF3 3 2 549582 134997 2747910 1782245 118454 634670 323700 0 0 0 3 0 0 157713 1612806 1612806 2553380 1782245 1782245 1335924 1782245 1782245 1782245 157713 157713 0 157713 157713 0 1612806 1612806 2553380 157713 157713 0 157713 157713 0 157713 157713 0\n\
+KF3 3 3 765601 187840 3828005 3371140 162278 884730 454675 0 0 0 3 0 0 277332 3110727 3110727 3647308 3371140 3371140 2653847 3371140 3371140 3371140 277332 277332 0 277332 277332 0 3110727 3110727 3647308 277332 277332 0 277332 277332 0 277332 277332 0\n\
+KF9 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
+KF9 1 1 39 0 195 0 0 0 0 0 0 0 3 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0\n\
+KF9 1 2 117 0 585 585 0 0 0 0 0 0 5 0 0 117 468 468 359 585 585 252 585 585 585 117 117 0 117 117 0 468 468 359 117 117 0 117 117 0 117 117 0\n\
+KF9 2 0 1792 236 8960 0 212 1020 650 0 0 0 2 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0 0 0 0 0 0 4827 0 0 0 0 0 0 0 0 0\n\
+KF9 2 1 9401 1218 47005 0 647 5598 1625 0 0 0 3 0 0 0 0 0 40494 0 0 0 0 0 0 0 0 0 0 0 0 0 0 40494 0 0 0 0 0 0 0 0 0\n\
+KF9 2 2 48590 6207 242950 164835 1952 29529 4550 0 0 0 5 0 0 20655 145557 145557 217831 164835 164835 99999 164835 164835 164835 20655 20655 0 20655 20655 0 145557 145557 217831 20655 20655 0 20655 20655 0 20655 20655 0\n\
+KFW 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
+KFW 1 1 5 0 25 0 0 0 0 0 0 0 2 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0 0 0 0 0 0 15 0 0 0 0 0 0 0 0 0\n\
+KFW 1 2 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
+KFW 1 3 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
+KFW 1 4 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 11 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
+KFW 1 5 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
+KFW 1 6 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
+KFW 1 7 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 11 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
+KFW 1 8 5 0 25 25 0 0 0 0 0 0 3 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
+KFW 1 9 5 0 25 25 0 0 0 0 0 0 2 0 0 5 20 20 15 25 25 12 25 25 25 5 5 0 5 5 0 20 20 15 5 5 0 5 5 0 5 5 0\n\
+KFW 1 10 5 0 25 25 0 0 0 0 0 1 2 0 0 4 16 16 12 25 25 11 25 25 25 4 4 0 4 4 0 16 16 12 4 4 0 4 4 0 4 4 0\n\
+KFW 1 11 5 0 25 25 0 0 0 0 0 5 3 0 0 0 0 0 0 25 25 12 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
+KFW 1 12 5 0 25 25 0 0 0 0 0 5 2 0 0 0 0 0 0 25 25 12 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
+KFW 1 13 5 0 25 25 0 0 0 0 0 5 2 0 0 0 0 0 0 25 25 11 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
+KFW 1 14 5 0 25 25 0 0 0 0 0 5 3 0 0 0 0 0 0 25 25 12 25 25 25 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0\n\
+KG 1 0 10 0 50 0 0 0 0 0 0 0 1 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0 0 0 0 0 0 8 0 0 0 0 0 0 0 0 0\n\
+KG 1 1 39 0 195 0 0 0 0 0 0 0 3 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0 0 0 0 0 0 117 0 0 0 0 0 0 0 0 0\n\
+KG 1 2 117 0 585 585 0 0 0 0 0 0 5 0 0 117 468 468 351 585 585 252 585 585 585 117 117 0 117 117 0 468 468 351 117 117 0 117 117 0 117 117 0\n\
+KG 1 3 351 0 1755 1755 0 0 0 0 0 0 4 0 0 351 1404 1404 1053 1755 1755 756 1755 1755 1755 351 351 0 351 351 0 1404 1404 1053 351 351 0 351 351 0 351 351 0\n\
+KG 2 0 4015 610 20075 0 0 0 925 0 450 0 2 0 0 0 0 0 10824 0 0 0 0 0 0 0 0 0 0 0 0 0 0 10824 0 0 0 0 0 36 0 0 0\n\
+KG 2 1 22092 3648 110460 0 0 0 3120 0 3420 0 3 0 0 0 0 0 88296 0 0 0 0 0 0 0 0 0 0 0 0 0 0 88296 0 0 0 0 0 630 0 0 0\n\
+KG 2 2 98892 14778 494460 334755 0 0 19035 0 21060 0 5 0 1512 51642 292302 292302 423162 334755 334755 188253 334755 334755 334755 53154 53154 0 53154 53154 0 292302 292302 423162 53154 53154 0 58824 58824 7236 53154 53154 0\n\
+KG 2 3 400194 61506 2000970 2000970 0 0 56160 0 104490 0 6 0 19116 265491 1765179 1770093 1756485 2000970 2000970 1261629 2000970 1995300 2000970 265923 284607 0 265923 284607 0 1765179 1770093 1756485 265923 284607 0 309609 346005 49680 265923 284607 0\n\
+";
+
+/// 登记第七节「挂载 2 开头那次故障」表（M = 2 的各格合起来；每行 = 域、故障名、历史数、被拒数、走了重读的数）。
+const PINNED_MOUNT_TWO_FAULT_TEXT: &str = "\
+KF none 28776 310 0\n\
+KF RN-T1 28776 310 3639\n\
+KF RN-TP 25165 338 0\n\
+KF SN0-T1 28776 310 28466\n\
+KF SN0-TP 1238 1238 0\n\
+KF SN1-T1 28776 310 28466\n\
+KF SN1-TP 1238 1238 0\n\
+KF SB-T1 28776 310 28466\n\
+KF SB-TP 1238 1238 0\n\
+KF RI-T1 28776 310 25861\n\
+KF RI-TP 3823 1218 0\n\
+KF RNSB-T1 28776 310 28466\n\
+KF RNSB-TP 1238 1238 0\n\
+KF3 none 910 70 0\n\
+KF3 RN-T1 910 70 195\n\
+KF3 RN-TP 743 98 0\n\
+KF3 SN0-T1 910 70 840\n\
+KF3 SN0-TP 182 182 0\n\
+KF3 SN1-T1 910 70 840\n\
+KF3 SN1-TP 182 182 0\n\
+KF3 SB-T1 910 70 840\n\
+KF3 SB-TP 182 182 0\n\
+KF3 RI-T1 910 70 695\n\
+KF3 RI-TP 307 162 0\n\
+KF3 RNSB-T1 910 70 840\n\
+KF3 RNSB-TP 182 182 0\n\
+KF9 none 7268 105 0\n\
+KF9 RN-T1 7268 105 1182\n\
+KF9 RN-TP 6107 126 0\n\
+KF9 SN0-T1 7268 105 7163\n\
+KF9 SN0-TP 393 393 0\n\
+KF9 SN1-T1 7268 105 7163\n\
+KF9 SN1-TP 393 393 0\n\
+KF9 SB-T1 7268 105 7163\n\
+KF9 SB-TP 393 393 0\n\
+KF9 RI-T1 7268 105 6313\n\
+KF9 RI-TP 1228 378 0\n\
+KF9 RNSB-T1 7268 105 7163\n\
+KF9 RNSB-TP 393 393 0\n\
+KG none 40921 0 0\n\
+KG RN-T1 40271 0 0\n\
+KG RN-TP 40271 0 0\n\
+KG SN0-T1 40921 0 0\n\
+KG SN0-TP 40921 0 0\n\
+KG SN1-T1 40921 0 0\n\
+KG SN1-TP 40921 0 0\n\
+KG SB-T1 40921 0 0\n\
+KG SB-TP 40921 0 0\n\
+KG RI-T1 38831 0 0\n\
+KG RI-TP 38831 0 0\n\
+KG RNSB-T1 40271 0 0\n\
+KG RNSB-TP 40271 0 0\n\
+";
+
+/// 一行钉值（`PINNED_FAULT_CELLS_TEXT` 里的一行）：一个 (域, M, U) 格上生成器一侧的计数与假结局、本地择根序的读数。
+#[derive(Clone, Debug)]
+struct PinnedFaultCell {
+    domain: FaultDomain,
+    mounts: usize,
+    user_publishes: usize,
+    histories: u64,
+    histories_with_an_uncovered_fault: u64,
+    judged_states: u64,
+    confirmed_states: u64,
+    refused_mounts: u64,
+    reread_mounts: u64,
+    states_with_a_torn_system_slot: u64,
+    states_with_same_txg_different_instance_roots: u64,
+    states_with_higher_instance_lower_txg_roots: u64,
+    states_without_a_readable_root: u64,
+    above_newest_root_maximum: usize,
+    /// 条款的理想恢复丢了的被判态：因在飞上限、罩着的其余、不罩。
+    ideal_loss: [u64; 3],
+    kf1: [u64; 3],
+    kf2: [u64; 4],
+    kf4: [u64; 2],
+    /// 本地择根序 today、v1823、v1824、v1825、v1826、v1822 各自的 KJ1、KJ1T、KJ2。
+    local_rules: [[u64; 3]; 6],
+}
+
+fn parse_the_pinned_fault_cells(text: &str) -> Vec<PinnedFaultCell> {
+    text.lines()
+        .map(|line| {
+            let fields: Vec<&str> = line.split_whitespace().collect();
+            let domain = *FaultDomain::ALL
+                .iter()
+                .find(|domain| domain.label() == fields[0])
+                .expect("钉值表里的域名在六个域之内");
+            let numbers: Vec<u64> = fields[3..]
+                .iter()
+                .map(|field| field.parse::<u64>().expect("钉值表里的数是非负整数"))
+                .collect();
+            assert_eq!(numbers.len(), 41, "钉值表每行 41 个数：{line}");
+            let mut local_rules = [[0u64; 3]; 6];
+            for (rule_index, rule) in local_rules.iter_mut().enumerate() {
+                rule.copy_from_slice(&numbers[23 + 3 * rule_index..26 + 3 * rule_index]);
+            }
+            PinnedFaultCell {
+                domain,
+                mounts: fields[1].parse().expect("M 是非负整数"),
+                user_publishes: fields[2].parse().expect("U 是非负整数"),
+                histories: numbers[0],
+                histories_with_an_uncovered_fault: numbers[1],
+                judged_states: numbers[2],
+                confirmed_states: numbers[3],
+                refused_mounts: numbers[4],
+                reread_mounts: numbers[5],
+                states_with_a_torn_system_slot: numbers[6],
+                states_with_same_txg_different_instance_roots: numbers[7],
+                states_with_higher_instance_lower_txg_roots: numbers[8],
+                states_without_a_readable_root: numbers[9],
+                above_newest_root_maximum: usize::try_from(numbers[10])
+                    .expect("记录条数装得进 usize"),
+                ideal_loss: [numbers[11], numbers[12], numbers[13]],
+                kf1: [numbers[14], numbers[15], numbers[16]],
+                kf2: [numbers[17], numbers[18], numbers[19], numbers[20]],
+                kf4: [numbers[21], numbers[22]],
+                local_rules,
+            }
+        })
+        .collect()
+}
+
+fn pinned_fault_cells() -> &'static Vec<PinnedFaultCell> {
+    static PINNED: std::sync::OnceLock<Vec<PinnedFaultCell>> = std::sync::OnceLock::new();
+    PINNED.get_or_init(|| parse_the_pinned_fault_cells(PINNED_FAULT_CELLS_TEXT))
+}
+
+fn pinned_fault_cell(
+    domain: FaultDomain,
+    cell: RootChoiceCell,
+) -> Option<&'static PinnedFaultCell> {
+    pinned_fault_cells().iter().find(|pinned| {
+        pinned.domain == domain
+            && pinned.mounts == cell.mounts
+            && pinned.user_publishes == cell.user_publishes
+    })
+}
+
+/// 本地择根序 today 在 KG 上、被判时无故障的态里的 KJ1T 合计（登记 5.6「C331 的判别力自证」：本地理想恢复算出的数是 648）。
+const PINNED_KG_TODAY_KJ1T_WITHOUT_A_JUDGED_FAULT: u64 = 648;
+
+/// 每个跑到的格，装置数出的生成器一侧的计数与本地择根序的读数等于独立生成器的数；不符走作废条款 TV4（这些量只看生成器与真相，与被测实现无关）。
+/// 条款的理想恢复在合法域里丢了的被判态（因在飞上限、罩着的其余）不是 0 时不当 TV4 停，交回登记的 TF9（条款或实现可能错）。
+fn assert_the_pinned_fault_cell(
+    domain: FaultDomain,
+    cell: RootChoiceCell,
+    tally: &FaultTally,
+) -> Vec<String> {
+    let pinned = pinned_fault_cell(domain, cell).expect("每个跑到的格都在登记第七节乙类的表里");
+    let place = format!(
+        "域 {} M = {} U = {}",
+        domain.label(),
+        cell.mounts,
+        cell.user_publishes
+    );
+    assert_eq!(tally.histories, pinned.histories, "TV4：{place} 历史数");
+    assert_eq!(
+        tally.histories_with_an_uncovered_fault, pinned.histories_with_an_uncovered_fault,
+        "TV4：{place} 历史里有不罩故障的历史数"
+    );
+    assert_eq!(
+        tally.judged_states, pinned.judged_states,
+        "TV4：{place} 被判态数"
+    );
+    assert_eq!(
+        tally.confirmed_states, pinned.confirmed_states,
+        "TV4：{place} 有已确认的被判态数"
+    );
+    assert_eq!(
+        tally.refused_mounts, pinned.refused_mounts,
+        "TV4：{place} 被拒的挂载数"
+    );
+    assert_eq!(
+        tally.reread_mounts, pinned.reread_mounts,
+        "TV4：{place} 走了重读那一遍的挂载数"
+    );
+    assert_eq!(
+        tally.states_with_a_torn_system_slot, pinned.states_with_a_torn_system_slot,
+        "TV4：{place} 带撕裂系统配置槽的被判态数"
+    );
+    assert_eq!(
+        tally.states_with_same_txg_different_instance_roots,
+        pinned.states_with_same_txg_different_instance_roots,
+        "TV4：{place} 根环里同 txg 不同实例同在的被判态数"
+    );
+    assert_eq!(
+        tally.states_with_higher_instance_lower_txg_roots,
+        pinned.states_with_higher_instance_lower_txg_roots,
+        "TV4：{place} 实例代号更高而 txg 更低同在的被判态数"
+    );
+    assert_eq!(
+        tally.states_without_a_readable_root, pinned.states_without_a_readable_root,
+        "TV4：{place} 读视图里一条根都没有的被判态数"
+    );
+    assert_eq!(
+        tally.above_newest_root_maximum, pinned.above_newest_root_maximum,
+        "TV4：{place} 读视图里版本最大的根之上同实例的记录条数的最大值"
+    );
+    assert_eq!(
+        tally.ideal_loss_uncovered, pinned.ideal_loss[2],
+        "TV4：{place} 条款的理想恢复丢了的被判态里「不罩」那一类"
+    );
+    let mut findings = Vec::new();
+    if domain.is_a_legal_history() {
+        if tally.ideal_loss_by_limit != pinned.ideal_loss[0]
+            || tally.ideal_loss_covered_other != pinned.ideal_loss[1]
+        {
+            findings.push(format!(
+                "TF9 {place} 条款的理想恢复丢了已确认发布的被判态：因在飞上限 {}、罩着的其余 {}（登记第七节甲类锚点都是 0）",
+                tally.ideal_loss_by_limit, tally.ideal_loss_covered_other
+            ));
+        }
+    } else {
+        assert_eq!(
+            tally.ideal_loss_by_limit, pinned.ideal_loss[0],
+            "TV4：{place} 条款的理想恢复丢了的被判态里「因在飞上限」那一类"
+        );
+        assert_eq!(
+            tally.ideal_loss_covered_other, pinned.ideal_loss[1],
+            "TV4：{place} 条款的理想恢复丢了的被判态里「罩着的其余」那一类"
+        );
+    }
+    for (rule_index, rule) in FaultLocalRule::ALL.iter().take(6).enumerate() {
+        assert_eq!(
+            tally.local_rules[rule_index],
+            pinned.local_rules[rule_index],
+            "TV4：{place} 本地择根序 {} 的 KJ1 / KJ1T / KJ2 读数",
+            rule.label()
+        );
+    }
+    findings
+}
+
+/// M = 2 的格合起来、按挂载 2 开头那项故障报的历史数、被拒数、重读数等于登记第七节那张表（TV4）。
+fn assert_the_pinned_mount_two_table(
+    domain: FaultDomain,
+    merged_over_the_mount_two_cells: &FaultTally,
+) {
+    let mut checked_rows = 0;
+    for line in PINNED_MOUNT_TWO_FAULT_TEXT.lines() {
+        let fields: Vec<&str> = line.split_whitespace().collect();
+        if fields[0] != domain.label() {
+            continue;
+        }
+        let menu_index = MOUNT_FAULT_MENU_LABELS
+            .iter()
+            .position(|label| *label == fields[1])
+            .expect("钉值表里的故障名在菜单之内");
+        let pinned: Vec<u64> = fields[2..]
+            .iter()
+            .map(|field| field.parse::<u64>().expect("钉值表里的数是非负整数"))
+            .collect();
+        let count = merged_over_the_mount_two_cells.mount_two[menu_index];
+        assert_eq!(
+            [count.histories, count.refused, count.reread],
+            [pinned[0], pinned[1], pinned[2]],
+            "TV4：域 {} M = 2 挂载 2 开头故障 {} 的历史数 / 被拒数 / 重读数",
+            domain.label(),
+            fields[1]
+        );
+        checked_rows += 1;
+    }
+    assert_eq!(
+        checked_rows, MOUNT_FAULT_MENU_LENGTH,
+        "每个域的 M = 2 表 13 行"
+    );
+}
+
+// ── 阳性对照（登记 5.6）：假结局读数与钉值比，走同一段判据；产物里打 `name=control` 行
+
+#[derive(Clone, Debug)]
+struct FaultControlReading {
+    criterion: FaultCriterion,
+    measured_where: String,
+    value: u64,
+    must_see: String,
+    is_red_as_required: bool,
+}
+
+fn fault_reading_that_must_equal(
+    criterion: FaultCriterion,
+    measured_where: String,
+    value: u64,
+    expected: u64,
+) -> FaultControlReading {
+    FaultControlReading {
+        criterion,
+        measured_where,
+        value,
+        must_see: format!("equals_{expected}"),
+        is_red_as_required: value == expected,
+    }
+}
+
+fn fault_place_of(domain: FaultDomain, cell: RootChoiceCell) -> String {
+    format!(
+        "domain={} ring_slots={} slots_per_region={} mounts={} user_publishes={}",
+        domain.label(),
+        domain.geometry().ring().ring_slots,
+        domain.geometry().slots_per_region(),
+        cell.mounts,
+        cell.user_publishes
+    )
+}
+
+/// 一个合法历史或 KG 格上每条判据的对照读数（登记 5.6 表）：KJ0 在 KF M = 1、U = 0 指定一态上 panic（违例数 1）；KJ1、KJ1T、KJ2 在 KF1 下等于第七节 KF1 三列；
+/// KJ1、KJ1T、KJ3 在 KF2 下各等于「有已确认的被判态数」，KJ2 等于第七节 KF2 KJ2 一列；KJ1、KJ1T 在 KF4 下等于第七节 KF4 两列。
+fn fault_control_readings_of_a_cell(
+    domain: FaultDomain,
+    cell: RootChoiceCell,
+    tally: &FaultTally,
+) -> Vec<FaultControlReading> {
+    let place = fault_place_of(domain, cell);
+    let pinned = pinned_fault_cell(domain, cell).expect("每个跑到的格都在登记第七节乙类的表里");
+    let mut readings = Vec::new();
+    if domain == FaultDomain::GateOn && cell.mounts == 1 && cell.user_publishes == 0 {
+        readings.push(fault_reading_that_must_equal(
+            FaultCriterion::Kj0Defined,
+            format!("{place} fake_outcome=panics_on_the_designated_state"),
+            tally.kj0_control_violations,
+            1,
+        ));
+    }
+    let oldest = tally.local_rules[FAULT_OLDEST_RULE_INDEX];
+    for (criterion, position) in [
+        (FaultCriterion::Kj1NotEarlierThanTheLastConfirmedPublish, 0),
+        (FaultCriterion::Kj1tDescendsFromTheLastConfirmedPublish, 1),
+        (FaultCriterion::Kj2NotEarlierThanAnyReadableRoot, 2),
+    ] {
+        readings.push(fault_reading_that_must_equal(
+            criterion,
+            format!("{place} fake_outcome=KF1_chooses_the_oldest_root"),
+            oldest[position],
+            pinned.kf1[position],
+        ));
+    }
+    for (criterion, position, expected) in [
+        (
+            FaultCriterion::Kj1NotEarlierThanTheLastConfirmedPublish,
+            0,
+            pinned.confirmed_states,
+        ),
+        (
+            FaultCriterion::Kj1tDescendsFromTheLastConfirmedPublish,
+            1,
+            pinned.confirmed_states,
+        ),
+        (
+            FaultCriterion::Kj2NotEarlierThanAnyReadableRoot,
+            2,
+            pinned.kf2[2],
+        ),
+        (
+            FaultCriterion::Kj3LandsOnTheIdealRecovery,
+            3,
+            pinned.confirmed_states,
+        ),
+    ] {
+        readings.push(fault_reading_that_must_equal(
+            criterion,
+            format!("{place} fake_outcome=KF2_lands_on_the_parent_of_the_last_confirmed"),
+            tally.kf2[position],
+            expected,
+        ));
+    }
+    for (criterion, position) in [
+        (FaultCriterion::Kj1NotEarlierThanTheLastConfirmedPublish, 0),
+        (FaultCriterion::Kj1tDescendsFromTheLastConfirmedPublish, 1),
+    ] {
+        readings.push(fault_reading_that_must_equal(
+            criterion,
+            format!("{place} fake_outcome=KF4_lands_on_the_newest_root_off_the_confirmed_chain"),
+            tally.kf4[position],
+            pinned.kf4[position],
+        ));
+    }
+    readings
+}
+
+/// KA 上本地写的六种择根序（五种加平局取后者赢）的 KJ2 违例数必须等于 36、18、18、52、0 与 0（登记第七节 KA；今天的序 0）。
+/// 次序：今天的序、平局取低、实例代号为主、比较不看实例、取第一个有效者、平局取后者赢。
+const PINNED_KA_KJ2_OF_THE_LOCAL_RULES: [(FaultLocalRule, u64); 6] = [
+    (FaultLocalRule::Today, 0),
+    (FaultLocalRule::TieTakesTheLowerInstance, 36),
+    (FaultLocalRule::InstanceFirstThenTxg, 18),
+    (FaultLocalRule::IgnoresTheInstance, 18),
+    (FaultLocalRule::FirstSelfVerifiedWins, 52),
+    (FaultLocalRule::TieTakesTheLater, 0),
+];
+
+/// KA（登记 5.4，同 r3）：根环读根环次序的前三个槽各取空或四条根之一，别的槽全 0，journal 环全 0。本地写的一种择根序在这 125 个态上 KJ2 违例几次。
+fn ka_kj2_violations_of_a_local_rule(rule: FaultLocalRule) -> u64 {
+    let materials: Vec<Arc<RootChoicePublishMaterial>> = ROOT_CHOICE_ADVERSARIAL_ROOT_VERSIONS
+        .iter()
+        .enumerate()
+        .map(|(version_index, version)| {
+            Arc::new(root_material_without_records(
+                version.instance,
+                version.checkpoint_txg,
+                NewRootSegment::off_timeline(
+                    1 + u64::try_from(version_index).expect("四条根"),
+                    version.checkpoint_txg,
+                ),
+            ))
+        })
+        .collect();
+    let choices_per_slot = materials.len() + 1;
+    let mut violations = 0;
+    for state_number in 0..choices_per_slot
+        .pow(u32::try_from(ROOT_CHOICE_ADVERSARIAL_FILLED_SLOTS).expect("三个槽"))
+    {
+        let mut remaining_choices = state_number;
+        let mut roots_in_ring_order: Vec<Option<&RootChoicePublishMaterial>> =
+            vec![None; ROOT_CHOICE_ADVERSARIAL_FILLED_SLOTS];
+        for slot_index in (0..ROOT_CHOICE_ADVERSARIAL_FILLED_SLOTS).rev() {
+            let choice = remaining_choices % choices_per_slot;
+            remaining_choices /= choices_per_slot;
+            if choice > 0 {
+                roots_in_ring_order[slot_index] = Some(materials[choice - 1].as_ref());
+            }
+        }
+        let newest_version = roots_in_ring_order
+            .iter()
+            .copied()
+            .flatten()
+            .map(|material| material.version)
+            .max();
+        if let (Some(chosen), Some(newest)) =
+            (rule.chooses_among(&roots_in_ring_order), newest_version)
+        {
+            violations += u64::from(chosen.version < newest);
+        }
+    }
+    violations
+}
+
+/// KA 上每种本地择根序的对照读数（KJ2）：必须等于 [`PINNED_KA_KJ2_OF_THE_LOCAL_RULES`]。
+fn fault_ka_control_readings() -> Vec<FaultControlReading> {
+    let place = fault_place_of(
+        FaultDomain::AdversarialRootRing,
+        RootChoiceCell {
+            mounts: 0,
+            user_publishes: 0,
+        },
+    );
+    PINNED_KA_KJ2_OF_THE_LOCAL_RULES
+        .iter()
+        .map(|(rule, expected)| {
+            fault_reading_that_must_equal(
+                FaultCriterion::Kj2NotEarlierThanAnyReadableRoot,
+                format!("{place} local_choice={}", rule.label()),
+                ka_kj2_violations_of_a_local_rule(*rule),
+                *expected,
+            )
+        })
+        .collect()
+}
+
+// ── 产物行
+
+/// 承诺内（历史罩着 ∧ 被判时罩着 ∧ 不是上限截掉）的违例数，从分格的行里算：罩着的格的违例数减去其中上限截掉那部分（登记 5.5 末段）。
+fn promised_violations_from_the_split_rows(
+    cells: &[FaultCellTally; FAULT_CELL_COUNT],
+    criterion_index: usize,
+) -> u64 {
+    let mut promised = 0;
+    for judged_fault in JudgedFault::ALL {
+        if !judged_fault.is_inside_the_single_fault_promise() {
+            continue;
+        }
+        let cell = &cells[fault_cell_index_of(*judged_fault, true)];
+        promised += cell.violations[criterion_index] - cell.cut_violations[criterion_index];
+    }
+    promised
+}
+
+fn emit_the_rows_of_a_fault_cell(
+    emitter: &mut Emitter,
+    domain: FaultDomain,
+    cell: RootChoiceCell,
+    tally: &FaultTally,
+) {
+    let place = fault_place_of(domain, cell);
+    emitter.result(&format!(
+        "name=root_choice_fault_states {place} histories={} histories_with_an_uncovered_fault={} judged_states={} confirmed_states={} refused_mounts={} reread_mounts={} states_with_a_torn_system_slot={} same_txg_different_instance_roots={} higher_instance_lower_txg_roots={} states_without_a_readable_root={} above_newest_root_max={} in_flight_limit={} ideal_loss_by_limit={} ideal_loss_covered_other={} ideal_loss_uncovered={} promised_states={} confirmed_publishes_max={} largest_counter={} panicked={} choose_root_returned_none_with_a_readable_root={} choose_root_returned_none_without_a_readable_root={} replay_failed={} replay_applied={} kj1_and_kj1t_differing_states={} kj1_and_kj1t_differing_witness={} slot_readings_checked={} slot_readings_differing_from_the_truth={} slot_reading_witness={}",
+        tally.histories,
+        tally.histories_with_an_uncovered_fault,
+        tally.judged_states,
+        tally.confirmed_states,
+        tally.refused_mounts,
+        tally.reread_mounts,
+        tally.states_with_a_torn_system_slot,
+        tally.states_with_same_txg_different_instance_roots,
+        tally.states_with_higher_instance_lower_txg_roots,
+        tally.states_without_a_readable_root,
+        tally.above_newest_root_maximum,
+        domain.geometry().ring().local_in_flight_limit(),
+        tally.ideal_loss_by_limit,
+        tally.ideal_loss_covered_other,
+        tally.ideal_loss_uncovered,
+        tally.promised_states,
+        tally.confirmed_publishes_maximum,
+        tally.largest_counter,
+        tally.panicked_states,
+        tally.chose_nothing_with_a_readable_root,
+        tally.chose_nothing_without_a_readable_root,
+        tally.replay_failed_states,
+        tally.replay_applied_states,
+        tally.kj1_and_kj1t_differing_states,
+        tally.kj1_and_kj1t_differing_witness.as_deref().unwrap_or("none"),
+        tally.slot_readings_checked,
+        tally.slot_readings_differing_from_the_truth,
+        tally.slot_reading_witness.as_deref().unwrap_or("none"),
+    ));
+    let local_rule_fields: Vec<String> = FaultLocalRule::ALL
+        .iter()
+        .enumerate()
+        .map(|(rule_index, rule)| {
+            let reading = tally.local_rules[rule_index];
+            format!(
+                "{}={}/{}/{}",
+                rule.label(),
+                reading[0],
+                reading[1],
+                reading[2]
+            )
+        })
+        .collect();
+    emitter.result(&format!(
+        "name=fake_outcomes {place} local_rules_kj1_kj1t_kj2:{} kf2_kj1_kj1t_kj2_kj3={}/{}/{}/{} kf4_kj1_kj1t={}/{} kf4_states_without_a_root_off_the_confirmed_chain={}",
+        local_rule_fields.join(","),
+        tally.kf2[0],
+        tally.kf2[1],
+        tally.kf2[2],
+        tally.kf2[3],
+        tally.kf4[0],
+        tally.kf4[1],
+        tally.kf4_states_without_a_root_off_the_confirmed_chain,
+    ));
+    emit_the_split_and_promised_rows(emitter, &place, tally);
+    if cell.mounts == 2 {
+        for (menu_index, label) in MOUNT_FAULT_MENU_LABELS.iter().enumerate() {
+            let count = tally.mount_two[menu_index];
+            emitter.result(&format!(
+                "name=mount_two_fault {place} mount_two_fault={label} histories={} refused={} reread={}",
+                count.histories, count.refused, count.reread
+            ));
+        }
+    }
+}
+
+/// 每个 (被判时故障, 历史标签) 格每条判据一行，再加承诺内两行（KJ1、KJ1T，装置直接数的与从分格的行算的各报一遍）。
+fn emit_the_split_and_promised_rows(emitter: &mut Emitter, place: &str, tally: &FaultTally) {
+    for judged_fault in JudgedFault::ALL {
+        for history_is_covered in [true, false] {
+            let cell_index = fault_cell_index_of(*judged_fault, history_is_covered);
+            let cell = &tally.cells[cell_index];
+            for criterion in FaultCriterion::ALL {
+                let criterion_index = criterion.index();
+                let ideal_violations = if criterion_index == KJ1_INDEX {
+                    cell.ideal_violations[0].to_string()
+                } else if criterion_index == KJ1T_INDEX {
+                    cell.ideal_violations[1].to_string()
+                } else {
+                    "not_applicable".to_string()
+                };
+                emitter.result(&format!(
+                    "name=violations {place} judged_fault={} history={} criterion={} states={} confirmed_states={} cut_by_the_limit_states={} judged={} violations={} cut_by_the_limit_violations={} ideal_clause_violations={ideal_violations} witness={}",
+                    judged_fault.label(),
+                    if history_is_covered { "inside_the_promise" } else { "outside_the_promise" },
+                    criterion.label(),
+                    cell.states,
+                    cell.confirmed_states,
+                    cell.cut_states,
+                    cell.judged[criterion_index],
+                    cell.violations[criterion_index],
+                    cell.cut_violations[criterion_index],
+                    tally.witness_of_a_cell(cell_index, criterion_index)
+                ));
+            }
+        }
+    }
+    for (position, criterion_index) in [KJ1_INDEX, KJ1T_INDEX].into_iter().enumerate() {
+        emitter.result(&format!(
+            "name=promised {place} criterion={} promised_states={} violations={} violations_from_the_split_rows={} witness={}",
+            FaultCriterion::ALL[criterion_index].label(),
+            tally.promised_states,
+            tally.promised_violations[position],
+            promised_violations_from_the_split_rows(&tally.cells, criterion_index),
+            tally.promised_witnesses[position].as_deref().unwrap_or("none")
+        ));
+    }
+}
+
+/// 一个域扫完之后的判决行（各判据在这个域全部格上的合计），并记作废与停机。
+fn emit_the_verdict_of_a_fault_domain(
+    emitter: &mut Emitter,
+    findings: &mut RunFindings,
+    domain: FaultDomain,
+    cells: &[RootChoiceCell],
+    total: &FaultTally,
+    tier: BoundsTier,
+) {
+    let largest_mounts = cells.iter().map(|cell| cell.mounts).max().unwrap_or(0);
+    let largest_user_publishes = cells
+        .iter()
+        .map(|cell| cell.user_publishes)
+        .max()
+        .unwrap_or(0);
+    // 登记作废条款 TV10：域是空的（没有一个已确认发布的态；KG 上实例代号更高而 txg 更低同在为 0）。
+    let domain_is_nonempty = total.confirmed_states > 0
+        && (domain != FaultDomain::GateOff
+            || total.states_with_higher_instance_lower_txg_roots > 0);
+    if !domain_is_nonempty {
+        findings.void_conditions.push(format!(
+            "TV10 {} 域是空的（没有一个已确认发布的态，或 KG 上实例代号更高而 txg 更低同在为 0），这个域的 0 违例不算数",
+            domain.label()
+        ));
+    }
+    // 登记作废条款 TV6：承诺内那一行对不回分格的行。
+    let promised_rows_equal_the_split_rows =
+        [KJ1_INDEX, KJ1T_INDEX]
+            .into_iter()
+            .enumerate()
+            .all(|(position, criterion_index)| {
+                total.promised_violations[position]
+                    == promised_violations_from_the_split_rows(&total.cells, criterion_index)
+            });
+    if !promised_rows_equal_the_split_rows {
+        findings.void_conditions.push(format!(
+            "TV6 {} 域里「承诺内」那一行的违例数对不回分格的行",
+            domain.label()
+        ));
+    }
+    // 登记停机条款 TH4：读槽对拍不一致，不当 KJ 的违例记。
+    if total.slot_readings_differing_from_the_truth > 0 {
+        findings.halt_conditions.push(format!(
+            "TH4 {} 域里 read_root_ring_slot 交回的与生成时的真相不同的槽数 {}（见证 {}）",
+            domain.label(),
+            total.slot_readings_differing_from_the_truth,
+            total.slot_reading_witness.as_deref().unwrap_or("none")
+        ));
+    }
+    // 登记第七节甲类、失败条款 TF9：合法历史的域里今天的 replay_journal 报错的被判态数是 0。
+    if domain.is_a_legal_history() && total.replay_failed_states > 0 {
+        findings.halt_conditions.push(format!(
+            "TF9 {} 域里 replay_journal 报错的被判态数 {}（登记第七节甲类锚点是 0）",
+            domain.label(),
+            total.replay_failed_states
+        ));
+    }
+    emitter.result(&format!(
+        "name=verdict domain={} ring_slots={} slots_per_region={} largest_mounts={largest_mounts} largest_user_publishes={largest_user_publishes} bounds={} criteria=root_choice_faults histories={} states={} confirmed_states={} kj0_violations={} panicked={} choose_root_returned_none_with_a_readable_root={} replay_failed={} kj1_violations={} kj1_judged={} kj1t_violations={} kj1t_judged={} kj2_violations={} kj2_judged={} kj3_violations={} kj3_judged={} promised_states={} promised_kj1_violations={} promised_kj1t_violations={} promised_rows_equal_the_split_rows={promised_rows_equal_the_split_rows} kj1_and_kj1t_differing_states={} slot_readings_differing_from_the_truth={} domain_is_nonempty={domain_is_nonempty}",
+        domain.label(),
+        domain.geometry().ring().ring_slots,
+        domain.geometry().slots_per_region(),
+        tier.label(),
+        total.histories,
+        total.judged_states,
+        total.confirmed_states,
+        total.violations_of_a_criterion(0),
+        total.panicked_states,
+        total.chose_nothing_with_a_readable_root,
+        total.replay_failed_states,
+        total.violations_of_a_criterion(KJ1_INDEX),
+        total.judged_of_a_criterion(KJ1_INDEX),
+        total.violations_of_a_criterion(KJ1T_INDEX),
+        total.judged_of_a_criterion(KJ1T_INDEX),
+        total.violations_of_a_criterion(KJ2_INDEX),
+        total.judged_of_a_criterion(KJ2_INDEX),
+        total.violations_of_a_criterion(KJ3_INDEX),
+        total.judged_of_a_criterion(KJ3_INDEX),
+        total.promised_states,
+        total.promised_violations[0],
+        total.promised_violations[1],
+        total.kj1_and_kj1t_differing_states,
+        total.slot_readings_differing_from_the_truth,
+    ));
+}
+
+// ── 本地常量回比（登记第三节回比表，r4 新读的三样；不符走停机条款 TH2）
+
+fn cross_check_root_choice_fault_local_constants_against_the_implementation() {
+    cross_check_root_choice_local_constants_against_the_implementation();
+    assert_eq!(
+        u64::try_from(FAULT_SYSTEM_SLOTS_PER_DEVICE).expect("2 装得进 u64"),
+        singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
+        "TH2：每块盘的系统配置槽数"
+    );
+    assert_eq!(
+        FAULT_MKFS_SYSTEM_GENERATION,
+        singlefs_core::make_filesystem::SYSTEM_CONFIGURATION_GENERATION_AT_MKFS,
+        "TH2：mkfs 写下的系统配置世代号"
+    );
+    assert_eq!(
+        FAULT_WRITABLE_DEVICE_COUNT_LOWER_BOUND,
+        singlefs_core::mount::STRIPE_WIDTH_LOWER_BOUND.0,
+        "TH2：可写挂载的设备数下限"
+    );
+    const {
+        assert!(
+            FAULT_DEVICE_COUNT >= FAULT_WRITABLE_DEVICE_COUNT_LOWER_BOUND,
+            "TH2：两块盘装得下可写挂载的设备数下限（历史里的可写挂载在盘上都不是空操作）"
+        );
+    }
+    assert_eq!(
+        FAULT_SYSTEM_SLOT_COUNT_OF_THE_POOL,
+        FAULT_DEVICE_COUNT * FAULT_SYSTEM_SLOTS_PER_DEVICE,
+        "整池的系统配置槽数"
+    );
+}
+
+// ── 命令行与主流程（`--family root_choice_faults`，只在 `--profile r4`）
+
+struct FaultSelection {
+    bounds: BoundsTier,
+    only: Option<Vec<String>>,
+    largest_user_publishes_cap: Option<usize>,
+}
+
+fn arguments_name_the_root_choice_faults_family(arguments: &[String]) -> bool {
+    arguments
+        .windows(2)
+        .any(|pair| pair[0] == "--family" && pair[1] == FAULT_FAMILY_LABEL)
+}
+
+/// `--bounds segment1|segment2|light`（这一族 segment1 与 segment2 同界、`light` 是调试用的小界）、`--family root_choice_faults`、
+/// `--only KF,KF3,KF9,KFW,KG,KA`、`--max-u <U>`（每个格的 U 封顶，只给调试用）。
+fn parse_the_root_choice_faults_command_line(
+    arguments: &[String],
+) -> Result<FaultSelection, String> {
+    let mut bounds: Option<BoundsTier> = None;
+    let mut only: Option<Vec<String>> = None;
+    let mut largest_user_publishes_cap: Option<usize> = None;
+    let mut position = 0;
+    while position < arguments.len() {
+        let option = arguments[position].as_str();
+        let value = arguments
+            .get(position + 1)
+            .ok_or_else(|| format!("{option} 后面缺一个值"))?
+            .as_str();
+        match option {
+            "--bounds" => {
+                bounds = Some(match value {
+                    "segment1" => BoundsTier::SegmentOne,
+                    "segment2" => BoundsTier::SegmentTwo,
+                    "light" => BoundsTier::Light,
+                    unknown_tier => return Err(format!("不认识的界：{unknown_tier}")),
+                });
+            }
+            "--family" => {
+                if value != FAULT_FAMILY_LABEL {
+                    return Err(format!("不认识的族：{value}"));
+                }
+            }
+            "--only" => {
+                let names: Vec<String> = value.split(',').map(str::to_string).collect();
+                for name in &names {
+                    if !FaultDomain::ALL.iter().any(|domain| domain.label() == name) {
+                        return Err(format!(
+                            "不认识的域：{name}（这一族的域是 KF、KF3、KF9、KFW、KG、KA）"
+                        ));
+                    }
+                }
+                only = Some(names);
+            }
+            "--max-u" => {
+                largest_user_publishes_cap = Some(
+                    value
+                        .parse::<usize>()
+                        .map_err(|error| format!("--max-u 要一个非负整数：{error}"))?,
+                );
+            }
+            unknown_option => return Err(format!("不认识的参数：{unknown_option}")),
+        }
+        position += 2;
+    }
+    Ok(FaultSelection {
+        bounds: bounds.ok_or_else(|| "缺 --bounds".to_string())?,
+        only,
+        largest_user_publishes_cap,
+    })
+}
+
+/// 跑中途收集的、收尾要用的几样读数（判别力自证、C331 的判别力自证）。
+#[derive(Default)]
+struct FaultRunWitnesses {
+    mount_two_by_domain: Vec<(FaultDomain, FaultTally)>,
+    torn_states_of_the_gate_on_domain_at_two_mounts_without_user_publishes: Option<u64>,
+    kg_kf4_at_two_mounts_and_three_user_publishes: Option<[u64; 2]>,
+    kg_local_today_kj1t_without_a_judged_fault: u64,
+    kg_implementation_kj1t_without_a_judged_fault: u64,
+    kg_implementation_kj3_without_a_judged_fault: u64,
+    kg_was_run: bool,
+}
+
+fn emit_the_control_rows(
+    emitter: &mut Emitter,
+    findings: &mut RunFindings,
+    readings: &[FaultControlReading],
+) {
+    for reading in readings {
+        emitter.result(&format!(
+            "name=control criterion={} measured={} value={} must_see={} red_as_required={}",
+            reading.criterion.label(),
+            reading.measured_where.replace(' ', ","),
+            reading.value,
+            reading.must_see,
+            reading.is_red_as_required
+        ));
+        if !reading.is_red_as_required {
+            findings.void_conditions.push(format!(
+                "TV3 判据 {} 的阳性对照没红：{} 量到 {}，必须 {}",
+                reading.criterion.label(),
+                reading.measured_where,
+                reading.value,
+                reading.must_see
+            ));
+        }
+    }
+}
+
+/// 登记第八节「判别力自证」四条与 C331 的判别力自证（5.6）：读数都来自这一次跑出的格，不另跑。
+fn emit_the_self_proofs(
+    emitter: &mut Emitter,
+    findings: &mut RunFindings,
+    witnesses: &FaultRunWitnesses,
+) {
+    let mount_two_of = |domain: FaultDomain| {
+        witnesses
+            .mount_two_by_domain
+            .iter()
+            .find(|(candidate, _tally)| *candidate == domain)
+            .map(|(_domain, tally)| tally)
+    };
+    let label_index = |label: &str| {
+        MOUNT_FAULT_MENU_LABELS
+            .iter()
+            .position(|candidate| *candidate == label)
+            .expect("故障名在菜单之内")
+    };
+    let mut proofs: Vec<(String, bool, String)> = Vec::new();
+    if let (Some(gate_on), Some(gate_off)) = (
+        mount_two_of(FaultDomain::GateOn),
+        mount_two_of(FaultDomain::GateOff),
+    ) {
+        let mut passed = true;
+        let mut detail = Vec::new();
+        for label in ["SN0-TP", "SN1-TP", "SB-TP", "RNSB-TP"] {
+            let with_the_gate = gate_on.mount_two[label_index(label)];
+            let without_the_gate = gate_off.mount_two[label_index(label)];
+            passed &= with_the_gate.histories > 0
+                && with_the_gate.refused == with_the_gate.histories
+                && without_the_gate.refused == 0;
+            detail.push(format!(
+                "{label}:KF_histories={}_KF_refused={}_KG_refused={}",
+                with_the_gate.histories, with_the_gate.refused, without_the_gate.refused
+            ));
+        }
+        proofs.push((
+            "gate_refuses_every_history_of_the_four_tp_faults_and_the_gate_off_domain_refuses_none"
+                .to_string(),
+            passed,
+            detail.join(","),
+        ));
+        let mut passed = true;
+        let mut detail = Vec::new();
+        for label in ["SN0-T1", "SN1-T1", "SB-T1"] {
+            let with_the_gate = gate_on.mount_two[label_index(label)];
+            passed &= with_the_gate.reread > 0;
+            detail.push(format!("{label}:KF_reread={}", with_the_gate.reread));
+        }
+        proofs.push((
+            "t1_takes_the_second_read_in_the_gate_on_domain".to_string(),
+            passed,
+            detail.join(","),
+        ));
+    }
+    if let Some([kj1, kj1t]) = witnesses.kg_kf4_at_two_mounts_and_three_user_publishes {
+        proofs.push((
+            "kf4_parts_kj1_from_kj1t_on_the_gate_off_domain".to_string(),
+            kj1 != kj1t,
+            format!("KF4_kj1={kj1},KF4_kj1t={kj1t}"),
+        ));
+    }
+    if let Some(torn) =
+        witnesses.torn_states_of_the_gate_on_domain_at_two_mounts_without_user_publishes
+    {
+        proofs.push((
+            "torn_third_state_is_enumerated".to_string(),
+            torn > 0,
+            format!("KF_M2_U0_states_with_a_torn_system_slot={torn}"),
+        ));
+    }
+    for (item, passed, detail) in &proofs {
+        emitter.result(&format!(
+            "name=self_proof item={item} passed={passed} detail={detail}"
+        ));
+        if !passed {
+            findings
+                .void_conditions
+                .push(format!("TV3 判别力自证没过：{item}（{detail}）"));
+        }
+    }
+    if witnesses.kg_was_run {
+        emitter.result(&format!(
+            "name=c331_proof domain=KG local_today_kj1t_without_a_judged_fault={} pinned={PINNED_KG_TODAY_KJ1T_WITHOUT_A_JUDGED_FAULT} implementation_kj1t_without_a_judged_fault={} implementation_kj3_without_a_judged_fault={}",
+            witnesses.kg_local_today_kj1t_without_a_judged_fault,
+            witnesses.kg_implementation_kj1t_without_a_judged_fault,
+            witnesses.kg_implementation_kj3_without_a_judged_fault,
+        ));
+        if witnesses
+            .kg_kf4_at_two_mounts_and_three_user_publishes
+            .is_some()
+            && witnesses.kg_local_today_kj1t_without_a_judged_fault == 0
+        {
+            findings.void_conditions.push(
+                "TV3 KG 没造出 C331 那一形：本地理想恢复在被判时无故障的态上 KJ1T 合计为 0（只作废 KG 那一行与依它的「拿掉保护会丢」）"
+                    .to_string(),
+            );
+        }
+    }
+}
+
+/// KA（同 r3）：三步在 125 个对抗根环上，行与判决用 r3 的写法；对照是本地写的六种择根序。
+fn run_the_adversarial_domain_of_the_faults_family(
+    emitter: &mut Emitter,
+    findings: &mut RunFindings,
+    control_readings: &mut Vec<FaultControlReading>,
+) -> u64 {
+    let geometry = RootChoiceDomain::AdversarialRootRing;
+    let cell = RootChoiceCell {
+        mounts: 0,
+        user_publishes: 0,
+    };
+    let tally =
+        root_choice_tally_of_the_adversarial_root_rings(RootChoiceImplementation::TodayThreeSteps);
+    assert_eq!(
+        tally.get(RootChoiceCounter::States),
+        PINNED_ROOT_CHOICE_ADVERSARIAL_STATES,
+        "TV4：KA 状态数"
+    );
+    assert_eq!(
+        tally.get(RootChoiceCounter::StatesWithoutASelfVerifiedRoot),
+        1,
+        "TV4：KA 里根环全空的态数（登记第七节乙类 KA：没有一条自证过的根的 1 态）"
+    );
+    emit_the_rows_of_a_root_choice_cell(emitter, geometry, cell, &tally);
+    let readings = fault_ka_control_readings();
+    emit_the_control_rows(emitter, findings, &readings);
+    control_readings.extend(readings);
+    emit_the_verdict_of_a_root_choice_domain(emitter, findings, geometry, &[cell], &tally);
+    tally.get(RootChoiceCounter::States)
+}
+
+fn run_the_root_choice_faults_family(arguments: &[String]) {
+    let selection = match parse_the_root_choice_faults_command_line(arguments) {
+        Ok(selection) => selection,
+        Err(problem) => {
+            eprintln!("  ✗ {problem}");
+            eprintln!("  → 怎么办：{USAGE}");
+            std::process::exit(2);
+        }
+    };
+    let thread_count = thread_count_from_the_environment();
+    install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
+    let run_started = Instant::now();
+    let mut emitter = Emitter { emitted: 0 };
+    let mut findings = RunFindings::default();
+    let selected_cells: Vec<(FaultDomain, Vec<RootChoiceCell>)> = FaultDomain::ALL
+        .iter()
+        .copied()
+        .filter(|domain| {
+            selection
+                .only
+                .as_ref()
+                .is_none_or(|names| names.iter().any(|name| name == domain.label()))
+        })
+        .map(|domain| {
+            (
+                domain,
+                fault_cells_of(
+                    domain,
+                    selection.bounds,
+                    selection.largest_user_publishes_cap,
+                ),
+            )
+        })
+        .collect();
+    emitter.result(&format!(
+        "name=config experiment=E165 bounds={} family={FAULT_FAMILY_LABEL} only={} max_u={} root_slot_bytes={ROOT_CHOICE_ROOT_SLOT_BYTES} region_devices=0,1,0 system_slots_per_device={FAULT_SYSTEM_SLOTS_PER_DEVICE} witness_gate=the_implementation_choice_one_unreadable_or_torn_slot_means_undecidable profile=r4",
+        selection.bounds.label(),
+        selection
+            .only
+            .as_ref()
+            .map_or_else(|| "every_domain".to_string(), |names| names.join(",")),
+        selection
+            .largest_user_publishes_cap
+            .map_or_else(|| "none".to_string(), |cap| cap.to_string()),
+    ));
+    // ① 常量回比断言（不过当场 panic，进程以非 0 退出，一行结果都不算）。
+    cross_check_root_choice_fault_local_constants_against_the_implementation();
+    emitter.result(
+        "name=self_check check=root_choice_fault_local_constants_equal_the_implementation passed=true",
+    );
+    let self_checks_passed = 1;
+    for ring in [RingGeometry::BIG, RingGeometry::SMALL_NINE] {
+        emitter.result(&format!(
+            "name=in_flight_limit ring_slots={} local={} implementation={}",
+            ring.ring_slots,
+            ring.local_in_flight_limit(),
+            journal_in_flight_record_limit(ring.ring_bytes())
+        ));
+    }
+    let mut witnesses = FaultRunWitnesses::default();
+    let mut control_readings: Vec<FaultControlReading> = Vec::new();
+    let mut total_states: u64 = 0;
+    let the_whole_registered_bound_is_run = selection.largest_user_publishes_cap.is_none()
+        && matches!(
+            selection.bounds,
+            BoundsTier::SegmentOne | BoundsTier::SegmentTwo
+        );
+    for (domain, cells) in &selected_cells {
+        if !domain.has_histories() {
+            total_states += run_the_adversarial_domain_of_the_faults_family(
+                &mut emitter,
+                &mut findings,
+                &mut control_readings,
+            );
+            continue;
+        }
+        let mut domain_total = FaultTally::new();
+        let mut mount_two_merged = FaultTally::new();
+        for cell in cells {
+            let cell_started = Instant::now();
+            let tally = fault_tally_of_a_cell(*domain, *cell, true, thread_count);
+            findings
+                .halt_conditions
+                .extend(assert_the_pinned_fault_cell(*domain, *cell, &tally));
+            assert_eq!(
+                tally.chose_nothing_without_a_readable_root,
+                pinned_fault_cell(*domain, *cell)
+                    .expect("每个跑到的格都在登记第七节乙类的表里")
+                    .states_without_a_readable_root,
+                "TV4：{} 读视图里一条根都没有的被判态里 choose_root 交回空的态数",
+                fault_place_of(*domain, *cell)
+            );
+            emit_the_rows_of_a_fault_cell(&mut emitter, *domain, *cell, &tally);
+            let readings = fault_control_readings_of_a_cell(*domain, *cell, &tally);
+            emit_the_control_rows(&mut emitter, &mut findings, &readings);
+            control_readings.extend(readings);
+            println!(
+                "E7TIMING name=elapsed scope=cell domain={} mounts={} user_publishes={} states={} elapsed_seconds={:.3} threads={thread_count}",
+                domain.label(),
+                cell.mounts,
+                cell.user_publishes,
+                tally.judged_states,
+                cell_started.elapsed().as_secs_f64()
+            );
+            if *domain == FaultDomain::GateOn && cell.mounts == 2 && cell.user_publishes == 0 {
+                witnesses.torn_states_of_the_gate_on_domain_at_two_mounts_without_user_publishes =
+                    Some(tally.states_with_a_torn_system_slot);
+            }
+            if *domain == FaultDomain::GateOff {
+                witnesses.kg_was_run = true;
+                witnesses.kg_local_today_kj1t_without_a_judged_fault +=
+                    tally.local_kj1t_by_judged_fault[0][JudgedFault::Nothing.index()];
+                for history_is_covered in [true, false] {
+                    let none_cell =
+                        &tally.cells[fault_cell_index_of(JudgedFault::Nothing, history_is_covered)];
+                    witnesses.kg_implementation_kj1t_without_a_judged_fault +=
+                        none_cell.violations[KJ1T_INDEX];
+                    witnesses.kg_implementation_kj3_without_a_judged_fault +=
+                        none_cell.violations[KJ3_INDEX];
+                }
+                if cell.mounts == 2 && cell.user_publishes == 3 {
+                    witnesses.kg_kf4_at_two_mounts_and_three_user_publishes = Some(tally.kf4);
+                }
+            }
+            domain_total.merge(&tally);
+            if cell.mounts == 2 {
+                mount_two_merged.merge(&tally);
+            }
+        }
+        if the_whole_registered_bound_is_run && domain.largest_mounts(selection.bounds) >= 2 {
+            assert_the_pinned_mount_two_table(*domain, &mount_two_merged);
+        }
+        if cells.iter().any(|cell| cell.mounts == 2) {
+            witnesses
+                .mount_two_by_domain
+                .push((*domain, mount_two_merged));
+        }
+        total_states += domain_total.judged_states;
+        emit_the_verdict_of_a_fault_domain(
+            &mut emitter,
+            &mut findings,
+            *domain,
+            cells,
+            &domain_total,
+            selection.bounds,
+        );
+    }
+    emit_the_self_proofs(&mut emitter, &mut findings, &witnesses);
+    for condition in &findings.void_conditions {
+        emitter.result(&format!(
+            "name=void condition={}",
+            condition.replace(' ', "_")
+        ));
+    }
+    for condition in &findings.halt_conditions {
+        emitter.result(&format!(
+            "name=halt condition={}",
+            condition.replace(' ', "_")
+        ));
+    }
+    emitter.result(&format!(
+        "name=verdict scope=run bounds={} family={FAULT_FAMILY_LABEL} states={total_states} self_checks_passed={self_checks_passed} control_readings={} controls_not_red={} void_conditions={} halt_conditions={} profile=r4",
+        selection.bounds.label(),
+        control_readings.len(),
+        control_readings
+            .iter()
+            .filter(|reading| !reading.is_red_as_required)
+            .count(),
+        findings.void_conditions.len(),
+        findings.halt_conditions.len(),
+    ));
+    println!(
+        "E7TIMING name=elapsed scope=run states={total_states} elapsed_seconds={:.3} threads={thread_count}",
+        run_started.elapsed().as_secs_f64()
+    );
+    emitter.done();
+    if !findings.void_conditions.is_empty() || !findings.halt_conditions.is_empty() {
+        std::process::exit(1);
+    }
+}
+
 // ── 命令行与主流程
 
 struct RunSelection {
@@ -12128,7 +15833,7 @@
     family_label: &'static str,
 }
 
-const USAGE: &str = "用法：e165_replay_bounded_enumeration --bounds segment1|segment2|light [--family legal|adversarial|all|multi_disk|root_choice] [--only <域>[,<域>…]] [--max-n <N>] [--profile r1|r2|r3] [--marker-parity even|odd] [--table direct|scan|both] [--disk-order normal|reversed]（档不给取 r1；奇偶、建表法、盘次序只在 r2、r3 档有，不给分别取 even、direct、normal；这几个参数写在哪都行，单状态重放、逐态判定行、钉住的红例逐行重放同样认）；域名：L L6 L9 U C C6 A A6 AC X E1 E2 R M M6 MC（多盘三个域只在 --family multi_disk 时跑、要 --profile r2；all 只含合法历史与对抗两族；--family root_choice 只在 --profile r3 时跑，域 K K9 KW KA，认 --bounds、--only、--max-u）。单状态重放：--replay-state <域>:<N>:<序号>；逐态判定行：--list-states <域>:<N>（这两种的域名用产物里 domain= 的写法，U 域是 U_verify_on、U_verify_off）；钉住的红例逐行重放：--pinned-cases <表>（格式见 .claude/gate.d/e165-pinned-red-examples.tsv 文件头）";
+const USAGE: &str = "用法：e165_replay_bounded_enumeration --bounds segment1|segment2|light [--family legal|adversarial|all|multi_disk|root_choice|root_choice_faults] [--only <域>[,<域>…]] [--max-n <N>] [--profile r1|r2|r3|r4] [--marker-parity even|odd] [--table direct|scan|both] [--disk-order normal|reversed]（档不给取 r1；奇偶、建表法、盘次序只在 r2、r3 档有，不给分别取 even、direct、normal；这几个参数写在哪都行，单状态重放、逐态判定行、钉住的红例逐行重放同样认）；域名：L L6 L9 U C C6 A A6 AC X E1 E2 R M M6 MC（多盘三个域只在 --family multi_disk 时跑、要 --profile r2；all 只含合法历史与对抗两族；--family root_choice 只在 --profile r3 时跑，域 K K9 KW KA，认 --bounds、--only、--max-u；--family root_choice_faults 只在 --profile r4 时跑，域 KF KF3 KF9 KFW KG KA，认 --bounds、--only、--max-u；--profile r4 其余族与 r3 档相同）。单状态重放：--replay-state <域>:<N>:<序号>；逐态判定行：--list-states <域>:<N>（这两种的域名用产物里 domain= 的写法，U 域是 U_verify_on、U_verify_off）；钉住的红例逐行重放：--pinned-cases <表>（格式见 .claude/gate.d/e165-pinned-red-examples.tsv 文件头）";
 
 fn parse_the_command_line(arguments: &[String]) -> Result<RunSelection, String> {
     let mut bounds: Option<BoundsTier> = None;
@@ -12275,13 +15980,15 @@
                 .with_reported_disk_order(reported_disk_order),
             remaining_arguments,
         )),
-        "r3" => Ok((
+        "r3" | "r4" => Ok((
             RunProfile::r3(marker_parity)
                 .with_table_source(table_source)
                 .with_reported_disk_order(reported_disk_order),
             remaining_arguments,
         )),
-        unknown_profile => Err(format!("不认识的档：{unknown_profile}（只认 r1、r2、r3）")),
+        unknown_profile => Err(format!(
+            "不认识的档：{unknown_profile}（只认 r1、r2、r3、r4）"
+        )),
     }
 }
 
@@ -12371,6 +16078,20 @@
             return;
         }
     }
+    if arguments_name_the_root_choice_faults_family(&arguments) {
+        let profile_is_r4 = arguments_as_given
+            .windows(2)
+            .any(|pair| pair[0] == "--profile" && pair[1] == "r4");
+        if !profile_is_r4 {
+            eprintln!(
+                "  ✗ --family root_choice_faults 只在 --profile r4 时有（登记 r4 5.1：这一族只在 r4 档）"
+            );
+            eprintln!("  → 怎么办：{USAGE}");
+            std::process::exit(2);
+        }
+        run_the_root_choice_faults_family(&arguments);
+        return;
+    }
     if arguments_name_the_root_choice_family(&arguments) {
         if !run_profile.splits_j6qn_by_i812() {
             eprintln!(
@@ -13575,3 +17296,330 @@
         assert!(parse_the_root_choice_command_line(&arguments("--family root_choice")).is_err());
     }
 }
+
+#[cfg(test)]
+mod root_choice_fault_tests {
+    use super::*;
+
+    fn cell_of(mounts: usize, user_publishes: usize) -> RootChoiceCell {
+        RootChoiceCell {
+            mounts,
+            user_publishes,
+        }
+    }
+
+    #[test]
+    fn the_mount_fault_menu_has_thirteen_entries_in_the_registered_order_and_only_rnsb_is_outside_the_promise(
+    ) {
+        let world = FaultWorld::right_after_mkfs(RootChoiceDomain::BigRing);
+        let menu = mount_fault_menu(&world, 2);
+        let labels: Vec<&str> = menu.iter().map(|entry| entry.label()).collect();
+        assert_eq!(labels, MOUNT_FAULT_MENU_LABELS);
+        let outside_the_promise: Vec<&str> = menu
+            .iter()
+            .filter(|entry| !entry.is_inside_the_single_fault_promise())
+            .map(|entry| entry.label())
+            .collect();
+        assert_eq!(outside_the_promise, ["RNSB-T1", "RNSB-TP"]);
+        assert_eq!(mount_fault_menu(&world, 1).len(), 1, "挂载 1 开头没有故障");
+        for entry in &menu {
+            if entry.label().ends_with("-TP") {
+                assert_eq!(
+                    entry.every_read_unreadable,
+                    entry.first_read_unreadable,
+                    "TP 每一遍都读不出：{}",
+                    entry.label()
+                );
+            } else {
+                assert_eq!(
+                    entry.every_read_unreadable,
+                    FaultLocations::NOWHERE,
+                    "T1 只有第一遍读不出（「无」那一项也没有落点）：{}",
+                    entry.label()
+                );
+            }
+        }
+    }
+
+    #[test]
+    fn the_pinned_text_tables_hold_the_registered_number_of_rows() {
+        assert_eq!(pinned_fault_cells().len(), 49);
+        assert_eq!(
+            PINNED_MOUNT_TWO_FAULT_TEXT.lines().count(),
+            4 * MOUNT_FAULT_MENU_LENGTH
+        );
+    }
+
+    /// 登记 9.2 里生成器一侧那几条变异的取样点：闸恒假、T1 当 TP、写行 txg 只看根、不枚举撕裂第三态、最后一次挂载不并系统配置那 9 个组合、c_见证 = 0 判真、
+    /// 已确认算进崩着的那一次、RI 算不罩、盘不在时两块盘的记录都读不出、KJ2 比真相里的根、RN 取键最小的根，都落在这几个格的计数上。
+    #[test]
+    fn the_generator_counts_of_the_sampling_cells_equal_the_pinned_values() {
+        for (domain, mounts, user_publishes) in [
+            (FaultDomain::GateOn, 1, 0),
+            (FaultDomain::GateOn, 1, 1),
+            (FaultDomain::GateOn, 1, 2),
+            (FaultDomain::GateOn, 2, 0),
+            (FaultDomain::GateOn, 2, 1),
+            (FaultDomain::GateOnWrappingRootRing, 1, 11),
+            (FaultDomain::GateOff, 1, 2),
+        ] {
+            let cell = cell_of(mounts, user_publishes);
+            let tally = fault_tally_of_a_cell(domain, cell, false, 1);
+            let findings = assert_the_pinned_fault_cell(domain, cell, &tally);
+            assert!(findings.is_empty(), "{findings:?}");
+        }
+    }
+
+    #[test]
+    fn the_promised_rows_subtract_the_states_the_in_flight_limit_cut() {
+        let mut cells = [FaultCellTally::default(); FAULT_CELL_COUNT];
+        let covered_without_a_judged_fault = fault_cell_index_of(JudgedFault::Nothing, true);
+        cells[covered_without_a_judged_fault].violations[KJ1_INDEX] = 3;
+        cells[covered_without_a_judged_fault].cut_violations[KJ1_INDEX] = 3;
+        let device_gone = fault_cell_index_of(JudgedFault::DeviceZeroGone, true);
+        cells[device_gone].violations[KJ1_INDEX] = 5;
+        cells[device_gone].cut_violations[KJ1_INDEX] = 1;
+        let instance_roots_gone = fault_cell_index_of(JudgedFault::NewestInstanceRoots, true);
+        cells[instance_roots_gone].violations[KJ1_INDEX] = 7;
+        assert_eq!(
+            promised_violations_from_the_split_rows(&cells, KJ1_INDEX),
+            4,
+            "罩着的格 3 − 3 加 5 − 1；被判时 RI 不罩，不进承诺内"
+        );
+    }
+
+    #[test]
+    fn a_root_slot_reading_of_a_slot_unreadable_in_the_view_agrees_with_the_truth_only_as_unreadable(
+    ) {
+        let material = mkfs_root_material();
+        let unreadable = RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable);
+        assert!(fault_slot_reading_agrees_with_the_truth(
+            unreadable,
+            Some(&material),
+            true
+        ));
+        assert!(!fault_slot_reading_agrees_with_the_truth(
+            unreadable,
+            Some(&material),
+            false
+        ));
+        let zeros = RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified);
+        assert!(!fault_slot_reading_agrees_with_the_truth(zeros, None, true));
+        assert!(fault_slot_reading_agrees_with_the_truth(zeros, None, false));
+    }
+
+    #[test]
+    fn the_local_choice_rules_pick_as_the_registered_comparisons_on_the_adversarial_rings() {
+        for (rule, expected) in PINNED_KA_KJ2_OF_THE_LOCAL_RULES {
+            assert_eq!(
+                ka_kj2_violations_of_a_local_rule(rule),
+                expected,
+                "{}",
+                rule.label()
+            );
+        }
+    }
+
+    #[test]
+    fn a_system_configuration_write_takes_the_generation_after_the_largest_one_it_can_read() {
+        let mut slots = system_slots_right_after_mkfs();
+        let (slot, content) = system_slot_write_of(&slots, 0, 7, 3, FaultLocations::NOWHERE);
+        assert_eq!(slot, 0);
+        assert_eq!(
+            content,
+            SystemSlotContent::Verified {
+                generation: 2,
+                tail: 7,
+                instance: 3
+            }
+        );
+        slots[0][0] = content;
+        let (slot, content) = system_slot_write_of(&slots, 0, 8, 3, FaultLocations::NOWHERE);
+        assert_eq!(
+            (slot, content),
+            (
+                1,
+                SystemSlotContent::Verified {
+                    generation: 3,
+                    tail: 8,
+                    instance: 3
+                }
+            )
+        );
+        let unreadable_slot_zero = FaultLocations::NOWHERE.with_system_slot(0, 0);
+        let (slot, content) = system_slot_write_of(&slots, 0, 9, 4, unreadable_slot_zero);
+        assert_eq!(
+            (slot, content),
+            (
+                0,
+                SystemSlotContent::Verified {
+                    generation: 2,
+                    tail: 9,
+                    instance: 4
+                }
+            ),
+            "一直读不出的槽不算进最大世代号：只剩世代号 1 的那一槽，写 2"
+        );
+        slots[1][1] = SystemSlotContent::Torn;
+        slots[1][0] = SystemSlotContent::Torn;
+        let (_slot, content) = system_slot_write_of(&slots, 1, 1, 1, FaultLocations::NOWHERE);
+        assert_eq!(
+            content,
+            SystemSlotContent::Verified {
+                generation: 1,
+                tail: 1,
+                instance: 1
+            },
+            "一槽都读不出时世代号从 1 起"
+        );
+    }
+
+    #[test]
+    fn the_r4_profile_is_accepted_and_the_faults_family_name_is_recognised() {
+        assert!(arguments_name_the_root_choice_faults_family(&[
+            "--family".to_string(),
+            "root_choice_faults".to_string()
+        ]));
+        assert!(!arguments_name_the_root_choice_faults_family(&[
+            "--family".to_string(),
+            "root_choice".to_string()
+        ]));
+        let selection = parse_the_root_choice_faults_command_line(&[
+            "--bounds".to_string(),
+            "segment1".to_string(),
+            "--family".to_string(),
+            "root_choice_faults".to_string(),
+            "--only".to_string(),
+            "KF,KA".to_string(),
+        ])
+        .expect("登记的命令行");
+        assert_eq!(
+            selection.only,
+            Some(vec!["KF".to_string(), "KA".to_string()])
+        );
+        let as_given = |profile: &str| {
+            split_off_the_profile_options(&[
+                "--profile".to_string(),
+                profile.to_string(),
+                "--bounds".to_string(),
+                "segment1".to_string(),
+            ])
+        };
+        assert_eq!(
+            as_given("r4"),
+            as_given("r3"),
+            "r4 档的其余族与 r3 档相同：只多一族 root_choice_faults"
+        );
+    }
+
+    /// 登记 9.2 里「一次挂载第一次发布的父取最后一次已确认那一版」与「KJ1T 写成按版本序比」两条变异的取样点：KG 的 M = 2、U = 3 里，挂载 2 开头最新实例的全部根读不出（RI-T1）、
+    /// 而挂载 1 有已确认发布的那些单元。挂载 2 往下走的版本比最后一次已确认的更旧，它的发布不在已确认那一版的后代上，KJ1T 与 KJ1 在那里分得开。
+    #[test]
+    fn the_confirmed_chain_parts_kj1t_from_kj1_when_a_mount_goes_down_from_an_older_version_on_the_gate_off_domain(
+    ) {
+        let domain = FaultDomain::GateOff;
+        let collecting = FaultEnumeration {
+            domain,
+            mounts: 2,
+            stop_before_mount: Some(2),
+        };
+        let mut collector = FaultUnitCollector { units: Vec::new() };
+        collecting.enumerate_a_mount(
+            &FaultWorld::right_after_mkfs(domain.geometry()),
+            1,
+            3,
+            None,
+            &mut collector,
+        );
+        let continuing = FaultEnumeration {
+            domain,
+            mounts: 2,
+            stop_before_mount: None,
+        };
+        let judgement = FaultJudgement::new(domain, 2, false);
+        let first_menu_index_of_ri = MOUNT_FAULT_MENU_LABELS
+            .iter()
+            .position(|label| *label == "RI-T1")
+            .expect("RI-T1 在菜单里");
+        let mut tally = FaultTally::new();
+        for unit in collector.units.iter().filter(|unit| {
+            !unit.world.confirmed_user_publishes.is_empty()
+                && unit.menu_index == first_menu_index_of_ri
+        }) {
+            let mut judge = FaultStateJudge {
+                judgement: &judgement,
+                tally: FaultTally::new(),
+                histories_seen_in_this_unit: 0,
+                is_the_first_unit: false,
+            };
+            continuing.enumerate_a_mount(
+                &unit.world,
+                unit.mount_number,
+                unit.user_publishes_left,
+                Some(unit.menu_index),
+                &mut judge,
+            );
+            tally.merge(&judge.tally);
+        }
+        assert!(
+            tally.confirmed_states > 0,
+            "这些单元的最终态都有已确认的发布"
+        );
+        assert_eq!(
+            tally.kf4[1], tally.confirmed_states,
+            "KF4 落在不在已确认那条链上的根里：每个有已确认的态 KJ1T 都违例"
+        );
+        assert!(
+            tally.kf4[0] < tally.kf4[1],
+            "挂载 2 的根（版本比已确认的那一版新）不在它的后代上：KJ1 不违例而 KJ1T 违例，两条分得开（KJ1 {}、KJ1T {}）",
+            tally.kf4[0],
+            tally.kf4[1]
+        );
+    }
+
+    /// 每条判据在每个被判的格上各有对照读数、都红；读数条数照格数逐条判据数（删掉某条判据的某一行对照，条数对不上）。
+    #[test]
+    fn every_fault_criterion_has_a_positive_control_and_every_control_is_red() {
+        let cells = [
+            (FaultDomain::GateOn, cell_of(1, 0)),
+            (FaultDomain::GateOn, cell_of(1, 2)),
+            (FaultDomain::GateOff, cell_of(1, 2)),
+        ];
+        let mut readings = Vec::new();
+        for (domain, cell) in cells {
+            let tally = fault_tally_of_a_cell(domain, cell, true, 1);
+            readings.extend(fault_control_readings_of_a_cell(domain, cell, &tally));
+        }
+        readings.extend(fault_ka_control_readings());
+        for reading in &readings {
+            assert!(reading.is_red_as_required, "对照没红：{reading:?}");
+        }
+        let readings_of = |criterion: FaultCriterion| {
+            readings
+                .iter()
+                .filter(|reading| reading.criterion == criterion)
+                .count()
+        };
+        assert_eq!(readings_of(FaultCriterion::Kj0Defined), 1);
+        assert_eq!(
+            readings_of(FaultCriterion::Kj1NotEarlierThanTheLastConfirmedPublish),
+            3 * cells.len(),
+            "KJ1 在每个格上 KF1、KF2、KF4 各一条对照"
+        );
+        assert_eq!(
+            readings_of(FaultCriterion::Kj1tDescendsFromTheLastConfirmedPublish),
+            3 * cells.len(),
+            "KJ1T 在每个格上 KF1、KF2、KF4 各一条对照"
+        );
+        assert_eq!(
+            readings_of(FaultCriterion::Kj2NotEarlierThanAnyReadableRoot),
+            2 * cells.len() + PINNED_KA_KJ2_OF_THE_LOCAL_RULES.len(),
+            "KJ2 在每个格上 KF1、KF2 各一条，在 KA 上每种本地择根序一条"
+        );
+        assert_eq!(
+            readings_of(FaultCriterion::Kj3LandsOnTheIdealRecovery),
+            cells.len(),
+            "KJ3 在每个格上 KF2 一条对照"
+        );
+    }
+}
````

## `crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs`

````diff
--- crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs	2026-10-01 19:13:15.917454543 +0000
+++ crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs	2026-10-02 17:01:11.458873818 +0000
@@ -1851,10 +1851,13 @@
 
 fn main() {
     let arguments: Vec<String> = std::env::args().skip(1).collect();
-    if arguments.first().map(String::as_str) == Some("bad-states") {
+    let first_word = arguments.first().map(String::as_str);
+    if first_word == Some(BadTableVersion::FirstRun.mode_word())
+        || first_word == Some(BadTableVersion::SecondRun.mode_word())
+    {
         let Some(bad_states_command_line) = parse_bad_states_command_line(&arguments) else {
             eprintln!(
-                "用法：e170_o3_spec_executor_export bad-states --dump-directory <第一段的导出目录> --seed-hex <16 个十六进制字符>；收到 {arguments:?}"
+                "用法：e170_o3_spec_executor_export bad-states（或 bad-states-second-run）--dump-directory <第一段的导出目录> --seed-hex <16 个十六进制字符>；收到 {arguments:?}"
             );
             std::process::exit(2);
         };
@@ -1951,6 +1954,9 @@
 const BAD_UNIT_FLIP_OFFSET: usize = 200;
 const BAD_FLIPPED_BIT: u8 = 0x01;
 const BLIND_ROWS_PER_STREAM: usize = 20;
+/// 第 1 次重跑多出来的那一行（P10）与它要对照底态的那一行（P9）的行名；S5b 按这两个名字找行。
+const LOWER_BOUND_ROW_NAME: &str = "P10-Mk+";
+const FIRST_RUN_MK_PLUS_ROW_NAME: &str = "P9-Mk+";
 const BLIND_PSEUDO_ROOT_STEPS: u64 = 7;
 const BLIND_LOWEST_WATERMARK_STEP: u64 = 1000;
 const BLIND_HIGHEST_WATERMARK_STEP: u64 = 1_000_000;
@@ -1963,11 +1969,13 @@
 const BAD_SYSTEM_CONFIGURATION_ROOT_RING_BASE_OFFSET: usize = 371;
 
 /// 坏状态表里一行属于哪一族（登记 5.5）：P 行与盲测族期望「不在」、决定 Q3；N 行期望「在」、附带。
+/// 第 1 次重跑多一族 P10（`research/prompts/e170-r1-prereg.md` 5.5）：考 A(c) 下界的一行，附带 Q3-g、不进 Q3-a，所以不与 P 行共用标签。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum BadFamily {
     Positive,
     Negative,
     Blind,
+    AcknowledgementLowerBound,
 }
 
 impl BadFamily {
@@ -1976,6 +1984,40 @@
             Self::Positive => "P",
             Self::Negative => "N",
             Self::Blind => "B",
+            Self::AcknowledgementLowerBound => "P10",
+        }
+    }
+}
+
+/// 坏状态表的哪一版：第一次跑的表（`bad-states`，P8 期望「不在」、没有 P10），或第 1 次重跑的表（`bad-states-second-run`，P8 期望「在」、
+/// 每条流末尾多一行 P10；`research/prompts/e170-r1-prereg.md` 5.3、5.5）。两版的补丁、底态、盲测族抽样逐行相同（第十一节 S5）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum BadTableVersion {
+    FirstRun,
+    SecondRun,
+}
+
+impl BadTableVersion {
+    fn mode_word(self) -> &'static str {
+        match self {
+            Self::FirstRun => "bad-states",
+            Self::SecondRun => "bad-states-second-run",
+        }
+    }
+
+    /// `config` 结果行里 `mode=` 写的名字（第一次的模式照旧写 `bad_states`）。
+    fn config_mode_text(self) -> &'static str {
+        match self {
+            Self::FirstRun => "bad_states",
+            Self::SecondRun => "bad_states_second_run",
+        }
+    }
+
+    /// P8 的期望判定。第一次的登记写「不在」；第 1 次重跑的登记 5.5 修订按读法 3 改成「在」（落点 v_{n−1} 仍在 A(c) 里）。
+    fn p8_expected_verdict(self) -> &'static str {
+        match self {
+            Self::FirstRun => "not_in",
+            Self::SecondRun => "in",
         }
     }
 }
@@ -2550,8 +2592,9 @@
     PseudoContinuationOfTwo,
     /// P5（盲测 P5 型）：根 n 的实例代号 i → i − 1，重封。
     RootInstanceLowered,
-    /// P8：根 n 偏移 122 翻一位、记录 n 两份各翻一位，都不重封。
-    NewestRootAndRecordUnreadable,
+    /// P8、P10：根 n 偏移 122 翻一位、记录 n 两份各翻一位，都不重封。补丁与落点两处同一个写法；期望判定由调用处给：
+    /// 第一次的表 P8 写「不在」，第 1 次重跑的表 P8 写「在」、P10（Mk⁺ 底态）写「不在」。
+    NewestRootAndRecordUnreadable { expected_verdict: &'static str },
     /// N1：根 n 的 txg n → n − 2，重封，原位。
     RootTxgLowered,
     /// N2：同 P3，计数器 c_n + 2。
@@ -2685,7 +2728,7 @@
                 ExpectedLanding::OutsideTheVersionTable,
             )
         }
-        BadShape::NewestRootAndRecordUnreadable => {
+        BadShape::NewestRootAndRecordUnreadable { expected_verdict } => {
             let area = if newest.record.named.is_empty() {
                 BAD_RECORD_NEW_ROOT_SEGMENT_START
             } else {
@@ -2700,7 +2743,7 @@
             }
             (
                 patches,
-                "not_in",
+                expected_verdict,
                 ExpectedLanding::Version(facts.previous_version_text(newest.publish_index)),
             )
         }
@@ -2762,9 +2805,12 @@
 /// 一条流的坏状态表：P 行（F、Mk 各 6 行，Mk⁺ 上 P9）、N 行（F 上 N1–N4，Mk⁻ 上 N5）、盲测族 20 行。交回 (底态, 行)。
 /// 盲测族每行依次抽：底态（F 或任一个发过根槽写的调用边界，均匀）、形态（P2 型、P3 型、P5 型，均匀）、形态的参数（P2 型 j ∈ 1..7，
 /// P3 型 d ∈ [1000, 1 000 000]）；两条流共用一串 splitmix64，流一的 20 行先抽。
+/// 第 1 次重跑的表（`BadTableVersion::SecondRun`）与第一次的表只差两处：P8 的期望判定，和盲测族之后多一行 P10（Mk⁺ 底态上同 P8 的改法、
+/// 期望「不在」；这条流没有 Mk⁺ 就没有这一行）。P10 排在最后、不抽随机数，所以别的行的行号、补丁与盲测族抽样都不变（第十一节 S5）。
 fn bad_rows_of_stream(
     facts: &BadStreamFacts<'_>,
     generator: &mut SplitMix64,
+    table_version: BadTableVersion,
     checks: &mut Vec<StopCheck>,
 ) -> (Vec<BaseState>, Vec<BadRow>) {
     let full = facts.all_persisted();
@@ -2789,7 +2835,12 @@
             ),
             ("P4", BadShape::PseudoContinuationOfTwo),
             ("P5", BadShape::RootInstanceLowered),
-            ("P8", BadShape::NewestRootAndRecordUnreadable),
+            (
+                "P8",
+                BadShape::NewestRootAndRecordUnreadable {
+                    expected_verdict: table_version.p8_expected_verdict(),
+                },
+            ),
         ] {
             let name = format!("{label}-{}", base.label);
             rows.push(build_bad_row(
@@ -2882,6 +2933,22 @@
             checks,
         ));
     }
+    let lower_bound_base = match table_version {
+        BadTableVersion::FirstRun => None,
+        BadTableVersion::SecondRun => middle_plus.clone(),
+    };
+    if let Some(lower_bound_base) = lower_bound_base {
+        rows.push(build_bad_row(
+            facts,
+            LOWER_BOUND_ROW_NAME.to_string(),
+            BadFamily::AcknowledgementLowerBound,
+            lower_bound_base,
+            BadShape::NewestRootAndRecordUnreadable {
+                expected_verdict: "not_in",
+            },
+            checks,
+        ));
+    }
     let mut bases = vec![full, middle, middle_minus];
     bases.extend(middle_plus);
     (bases, rows)
@@ -3068,6 +3135,44 @@
     ]
 }
 
+/// 底态的落地向量文本：持久的写记 `2`、没持久的记 `0`（与 `states.tsv` 同一种写法）。
+fn landing_vector_text(persisted: &[bool]) -> String {
+    persisted
+        .iter()
+        .map(|is_persisted| if *is_persisted { '2' } else { '0' })
+        .collect()
+}
+
+/// S5b（`research/prompts/e170-r1-prereg.md` 第十一节）：P10 那一行的底态落地向量与底态镜像摘要，与同一条流 P9-Mk+ 那一行的相同；
+/// P10 行有没有，与 P9-Mk+ 行有没有一致（两行都由 Mk⁺ 给得出才造）。底态取错（例如取了 Mk）时落地向量与摘要都会不同。
+fn lower_bound_row_base_checks(
+    stream_label: &str,
+    rows: &[BadRow],
+    observations: &[BadObservation],
+) -> Vec<StopCheck> {
+    let position_of = |row_name: &str| rows.iter().position(|row| row.name == row_name);
+    let lower_bound = position_of(LOWER_BOUND_ROW_NAME);
+    let mk_plus_p9 = position_of(FIRST_RUN_MK_PLUS_ROW_NAME);
+    let mut checks = vec![StopCheck::new(
+        &format!("{stream_label}_p10_row_exists_exactly_when_p9_mk_plus_row_exists"),
+        mk_plus_p9.is_some(),
+        lower_bound.is_some(),
+    )];
+    if let (Some(lower_bound), Some(mk_plus_p9)) = (lower_bound, mk_plus_p9) {
+        checks.push(StopCheck::new(
+            &format!("{stream_label}_p10_base_landing_vector_equals_p9_mk_plus"),
+            landing_vector_text(&rows[mk_plus_p9].base.persisted),
+            landing_vector_text(&rows[lower_bound].base.persisted),
+        ));
+        checks.push(StopCheck::new(
+            &format!("{stream_label}_p10_base_digest_equals_p9_mk_plus"),
+            &observations[mk_plus_p9].base_digest,
+            &observations[lower_bound].base_digest,
+        ));
+    }
+    checks
+}
+
 /// 一条流造完、判完的坏状态：底态（带层 0 序号对照）、行、实现那一侧的观测、回比。
 struct BadStreamExport {
     name: StreamName,
@@ -3106,6 +3211,7 @@
     seed_hex: &str,
     seed: u64,
     generator: &mut SplitMix64,
+    table_version: BadTableVersion,
 ) -> BadStreamExport {
     let stream = prepare_stream(name);
     let directory = dump_root.join(name.directory_name());
@@ -3130,7 +3236,7 @@
         .collect();
     let facts = bad_stream_facts(&stream);
     checks.extend(bad_local_constant_checks(&facts, &facts.all_persisted()));
-    let (bases, rows) = bad_rows_of_stream(&facts, generator, &mut checks);
+    let (bases, rows) = bad_rows_of_stream(&facts, generator, table_version, &mut checks);
     let mut matched_bases: Vec<(BaseState, Option<u64>, usize)> = Vec::new();
     for base in bases.iter().chain(rows.iter().map(|row| &row.base)) {
         if matched_bases
@@ -3152,6 +3258,13 @@
         .iter()
         .map(|row| observe_bad_row(&stream, row))
         .collect();
+    if table_version == BadTableVersion::SecondRun {
+        checks.extend(lower_bound_row_base_checks(
+            name.directory_name(),
+            &rows,
+            &observations,
+        ));
+    }
     let mut states_text = String::new();
     let mut patches_bytes = Vec::new();
     let mut expected_text = String::new();
@@ -3302,15 +3415,19 @@
 }
 
 struct BadStatesCommandLine {
+    table_version: BadTableVersion,
     dump_directory: PathBuf,
     seed_hex: String,
 }
 
+/// 第一个词认两个模式：`bad-states`（第一次的表）、`bad-states-second-run`（第 1 次重跑的表）；别的词不认。
 fn parse_bad_states_command_line(arguments: &[String]) -> Option<BadStatesCommandLine> {
     let mut words = arguments.iter();
-    if words.next().map(String::as_str) != Some("bad-states") {
-        return None;
-    }
+    let table_version = match words.next().map(String::as_str) {
+        Some("bad-states") => BadTableVersion::FirstRun,
+        Some("bad-states-second-run") => BadTableVersion::SecondRun,
+        Some(_) | None => return None,
+    };
     let mut dump_directory = None;
     let mut seed_hex = None;
     while let Some(word) = words.next() {
@@ -3326,13 +3443,15 @@
             .chars()
             .all(|character| character.is_ascii_hexdigit()))
     .then_some(BadStatesCommandLine {
+        table_version,
         dump_directory: dump_directory?,
         seed_hex,
     })
 }
 
-/// `bad-states` 模式：两条流各造各判，流一先（盲测族的 splitmix64 一串接着抽）。退出码：0 正常；2 命令行不对或五个文件里有已存在的；
-/// 3 回比没过（产物照打，交主 agent）。
+/// `bad-states` 与 `bad-states-second-run` 模式（`BadTableVersion` 区分）：两条流各造各判，流一先（盲测族的 splitmix64 一串接着抽）。
+/// 退出码：0 正常；2 命令行不对或五个文件里有已存在的；3 回比没过（产物照打，交主 agent）。`config` 行的 `mode=` 两个模式各写各的名字，
+/// 第一次的 `bad-states` 模式除此之外输出一个字节不变。
 fn run_bad_states(command_line: &BadStatesCommandLine) -> i32 {
     for name in STREAM_NAMES {
         for file_name in BAD_FILES {
@@ -3354,7 +3473,8 @@
     let mut generator = SplitMix64 { state: seed };
     let mut lines = ResultLines { emitted: 0 };
     lines.result(&format!(
-        "name=config segment=s3 mode=bad_states seed_hex={} seed_u64={seed} blind_rows_per_stream={BLIND_ROWS_PER_STREAM} patched_writes_kind=system_configuration_slot judge_threads=1",
+        "name=config segment=s3 mode={} seed_hex={} seed_u64={seed} blind_rows_per_stream={BLIND_ROWS_PER_STREAM} patched_writes_kind=system_configuration_slot judge_threads=1",
+        command_line.table_version.config_mode_text(),
         command_line.seed_hex
     ));
     let mut failed_checks = 0usize;
@@ -3367,6 +3487,7 @@
             &command_line.seed_hex,
             seed,
             &mut generator,
+            command_line.table_version,
         );
         let (stream_failed, stream_panics) = report_bad_stream(&mut lines, &export);
         failed_checks += stream_failed;
@@ -3729,6 +3850,122 @@
     }
 
     #[test]
+    fn the_second_run_bad_states_mode_is_its_own_word_and_the_first_run_word_still_means_the_first_table(
+    ) {
+        let words = |text: &str| text.split(' ').map(str::to_string).collect::<Vec<String>>();
+        let first_run = parse_bad_states_command_line(&words(
+            "bad-states --dump-directory d --seed-hex 85441ae491419411",
+        ))
+        .expect("合法的命令行");
+        assert_eq!(first_run.table_version, BadTableVersion::FirstRun);
+        let second_run = parse_bad_states_command_line(&words(
+            "bad-states-second-run --dump-directory d --seed-hex 85441ae491419411",
+        ))
+        .expect("合法的命令行");
+        assert_eq!(second_run.table_version, BadTableVersion::SecondRun);
+        assert_eq!(second_run.seed_hex, "85441ae491419411");
+        assert!(parse_bad_states_command_line(&words(
+            "bad-states-second-run-extra --dump-directory d --seed-hex 85441ae491419411"
+        ))
+        .is_none());
+        assert_eq!(BadTableVersion::FirstRun.mode_word(), "bad-states");
+        assert_eq!(
+            BadTableVersion::SecondRun.mode_word(),
+            "bad-states-second-run"
+        );
+    }
+
+    /// 第 1 次重跑登记第十一节 S5 的单测形态：同一个种子、同一条流（流一，驱动快），两版的表逐行相同，只差 P8 的期望判定与末尾多出的 P10；
+    /// P10 的底态与 P9-Mk+ 同一个、补丁与同底态族的 P8-Mk 同一个（同一个改法落在 Mk⁺ 上）。
+    #[test]
+    fn the_second_run_table_differs_from_the_first_only_in_the_p8_expectation_and_a_final_p10_row_on_the_p9_base(
+    ) {
+        let first_run_seed = 0x8546_b125_8e42_7c34_u64;
+        let stream = prepare_stream(StreamName::NewPoolFileCreation);
+        let facts = bad_stream_facts(&stream);
+        let (_, first_run_rows) = bad_rows_of_stream(
+            &facts,
+            &mut SplitMix64 {
+                state: first_run_seed,
+            },
+            BadTableVersion::FirstRun,
+            &mut Vec::new(),
+        );
+        let (_, second_run_rows) = bad_rows_of_stream(
+            &facts,
+            &mut SplitMix64 {
+                state: first_run_seed,
+            },
+            BadTableVersion::SecondRun,
+            &mut Vec::new(),
+        );
+        let patch_texts = |row: &BadRow| -> Vec<(u32, u64, Vec<u8>)> {
+            row.patches
+                .iter()
+                .map(|patch| (patch.device.0, patch.offset, patch.bytes.clone()))
+                .collect()
+        };
+        assert_eq!(
+            second_run_rows.len(),
+            first_run_rows.len() + 1,
+            "第 1 次重跑的表比第一次的多恰一行（P10）"
+        );
+        for (first_run_row, second_run_row) in first_run_rows.iter().zip(&second_run_rows) {
+            assert_eq!(first_run_row.name, second_run_row.name);
+            assert_eq!(first_run_row.family, second_run_row.family);
+            assert_eq!(first_run_row.base.persisted, second_run_row.base.persisted);
+            assert_eq!(patch_texts(first_run_row), patch_texts(second_run_row));
+            assert_eq!(
+                first_run_row.expected_landing.text(),
+                second_run_row.expected_landing.text()
+            );
+            if second_run_row.name.starts_with("P8-") {
+                assert_eq!(
+                    first_run_row.expected_verdict, "not_in",
+                    "{}",
+                    first_run_row.name
+                );
+                assert_eq!(
+                    second_run_row.expected_verdict, "in",
+                    "{}",
+                    second_run_row.name
+                );
+            } else {
+                assert_eq!(
+                    first_run_row.expected_verdict, second_run_row.expected_verdict,
+                    "{}",
+                    first_run_row.name
+                );
+            }
+        }
+        let lower_bound_row = second_run_rows.last().expect("表不空");
+        assert_eq!(lower_bound_row.name, LOWER_BOUND_ROW_NAME);
+        assert_eq!(lower_bound_row.family, BadFamily::AcknowledgementLowerBound);
+        assert_eq!(lower_bound_row.expected_verdict, "not_in");
+        let mk_plus_p9_row = second_run_rows
+            .iter()
+            .find(|row| row.name == FIRST_RUN_MK_PLUS_ROW_NAME)
+            .expect("流一有 Mk⁺ 底态、有 P9 行");
+        assert_eq!(
+            lower_bound_row.base.persisted, mk_plus_p9_row.base.persisted,
+            "P10 的底态与 P9-Mk+ 是同一个落地向量（S5b）；取了 Mk 就差一个写"
+        );
+        let mk_p8_row = second_run_rows
+            .iter()
+            .find(|row| row.name == "P8-Mk")
+            .expect("有 P8-Mk 行");
+        assert_eq!(patch_texts(lower_bound_row), patch_texts(mk_p8_row));
+        assert_eq!(
+            lower_bound_row.expected_landing.text(),
+            mk_p8_row.expected_landing.text()
+        );
+        assert!(
+            lower_bound_row.base.persisted != mk_p8_row.base.persisted,
+            "Mk⁺ 比 Mk 多一个持久的写"
+        );
+    }
+
+    #[test]
     fn the_command_line_takes_dump_an_output_directory_and_an_optional_previous_export() {
         let words = |text: &str| text.split(' ').map(str::to_string).collect::<Vec<String>>();
         let parsed = parse_command_line(&words("dump --output-directory a --compare-with b"))
````

## `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs`

````diff
--- crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs	1970-01-01 00:00:00.000000000 +0000
+++ crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs	2026-10-02 17:26:36.657804185 +0000
@@ -0,0 +1,5670 @@
+//! E171（三oracle合谋测试） 第一段：入库装置。
+//!
+//! 登记：`research/prompts/e171-preregistration.md` 5.3（装置写在哪）、5.5（坏镜像表）、5.6（阳性与阴性对照）、5.8「第一段」、
+//! 第十一节停机 S1–S6 与 S8。它回答问题单 Q1–Q3 里 O1、O2 两列的那一半：两条层 0 流（新池新建文件、固定脚本到 E 再正常卸载）上，
+//! 每次 API 调用之后拿理想模型的答案与实现的结局比（O1-对拍），每个取样的崩溃状态上拿实现只读恢复读回的那一版问理想模型
+//! （O1-崩溃判官），再拿池级 checker 判这一态的崩溃镜像（O2）；O3（Python 执行器）读本装置照 E170（O3规约执行器第一版） 登记 5.3 写出的
+//! 中性格式，不在这个文件里。
+//!
+//! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`、harness、format 与 replay-prefix。
+//! 两条流的驱动、取样、中性格式的写出与读回照抄 `e170_o3_spec_executor_export.rs`（不 `#[path]` 引它），改三处（登记 5.3）：
+//! ① 驱动里的 API 调用不 `.expect`：结局（成功 / 错误成员 / panic）记进 `calls.tsv` 的 `ok` / `err` 列与 `call-outcomes.tsv`，
+//! 出错之后这条流不再往下驱动；② 跟着每次调用推理想模型，记 O1-对拍；③ 每个取样的状态算 O1-崩溃判官与 O2，写 `oracle-verdicts.tsv`。
+//!
+//!   e171_three_oracle_collusion baseline --output-directory <目录>
+//!   e171_three_oracle_collusion row --output-directory <目录>
+//!   e171_three_oracle_collusion known-bad-images --dump-directory <基线导出目录> --output-directory <目录> [--e170-bad-rows]
+//!   以上三种之后都可以加 `--positive-control <O1-对拍|O1-崩溃判官|O2>` 或 `--disable-oracle <O1|O2>`（只对 baseline 与 row）
+//!
+//! `baseline`：回比本地常量（S1）、三批取样、写十个导出文件加 `oracle-verdicts.tsv` 与 `call-outcomes.tsv`；`row`：同上，只是
+//! 不回比常量（坏实现那一趟本来就会让写流与段表变）、只取批 1 与批 2。目录已存在就拒绝。
+//! 标准输出：`E7RESULT` 行（与跑几次无关）与 `E7TIMING` 行（耗时）；收尾 `name=done emitted=N`。
+//! 退出码：0 正常；2 命令行不对或输出目录已存在；3 停机条款 S1、S3、S4、S5 没过（产物照打，交主 agent）。
+
+#[path = "../../../singlefs-harness/tests/common/mod.rs"]
+mod common;
+
+use std::collections::{BTreeMap, BTreeSet};
+use std::path::{Path, PathBuf};
+use std::rc::Rc;
+use std::time::Instant;
+
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_checker_tier::crash::{
+    full_expansion, judge_layer0_state_range, layer0_enumeration_tables, layer0_plan_state_count,
+    quick_tier_expansion, Layer0EnumerationTables, Layer0StateJudgement,
+};
+use singlefs_checker_tier::crash_facts::{extract_flow_facts, FlowFacts};
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+};
+use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
+use singlefs_core::make_filesystem::{INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
+use singlefs_core::mount::{
+    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, RollbackTarget,
+    ShadowLedger, UnmountRaisedTheFloor, Unmounted,
+};
+use singlefs_core::recovery::{PoolReader, RecoveryOutcome, RecoveryReport};
+use singlefs_core::transaction::{
+    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion,
+    PoolWriter, TransactionOutput, WarmUpOutput,
+};
+use singlefs_harness::memory_pool::{
+    newest_persisted_root, writes_and_segments_with_stream_indexes_and_entries, CrashImage,
+    MemoryPool, PublishedVersion, RetainedWrite, WrittenContents,
+};
+use singlefs_harness::model::{
+    crash_recovery_disagreement, IdealModel, ModelAnswer, ModelDeviceIdentity, ModelDisagreement,
+    ModelPoolGeometry, ModelRootKey, ObservedEffect, ObservedFile, ObservedOutcome,
+    ObservedReadBack,
+};
+use singlefs_harness::model_comparison::{
+    model_root_key, observed_mount_carrying_to_zero_unit_publishes,
+    observed_read_back_after_a_crash, observed_root_of_file_version, refusal_reason_of_mount_error,
+    refusal_reason_of_publish_error, refusal_reason_of_rollback_error,
+    reported_ceiling_of_mount_error, root_ring_slot_still_bad_after_one_reread_of_mount_error,
+    root_ring_slot_still_bad_after_one_reread_of_rollback_error,
+};
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::sha256::{sha256_digest, sha256_hexadecimal, DIGEST_BYTES};
+use singlefs_harness::{
+    RecordedEntrySpan, RecordedOperationKind, RecordedPublishEntry, RetainedOperation, SharedStream,
+};
+
+// ───────────────────────────── 本地常量（登记 5.3、S1） ─────────────────────────────
+//
+// 下面这一批与 `e170_o3_spec_executor_export.rs` 第 78–140 行同一批、同样的值与出处注释；`baseline` 开头与 crates 的同名公开常量或现算值回比，
+// 不直接引（决定几何与工作量的数写成本地常量：引了就等于装置与实现共用一个数，两边一起错而逐格相等）。
+// 这些常量只在基线那一趟回比：坏实现改了写流，这些数在坏实现那一趟本来就会变，照数记进产物。
+
+/// 盘数：`tests/common/mod.rs` 第 152–184 行 `formatted_recorded_images` 建两块盘（`for device_number in 0..2u32`）。
+const LOCAL_DEVICE_COUNT: usize = 2;
+/// 每块盘的字节数：`tests/common/mod.rs` 第 29 行 `IMAGE_BYTES = 4 << 30`。
+const LOCAL_DEVICE_BYTES: u64 = 4_294_967_296;
+/// 物理块与 io_min：`tests/common/mod.rs` 第 54–55 行 `physical_block_size: 512`、`minimum_input_output_bytes: 512`。
+const LOCAL_PHYSICAL_BLOCK_BYTES: u32 = 512;
+const LOCAL_MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;
+/// 导出格式的扇区宽（登记 5.3 `sector_bytes`（512））：与 `singlefs_harness::memory_pool::SECTOR_BYTES` 回比。
+const LOCAL_SECTOR_BYTES: u64 = 512;
+/// 固定 fsid：`tests/common/mod.rs` 第 33–35 行 `E142_FILESYSTEM_IDENTIFIER`。
+const LOCAL_FILESYSTEM_IDENTIFIER: [u8; 16] = [
+    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
+];
+/// 固定写时刻：`tests/common/mod.rs` 第 37 行 `FIXED_WRITE_TIME_SECONDS = 1_788_000_000`。
+const LOCAL_FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
+/// 第一个文件的字节数：`tests/common/mod.rs` 第 31 行 `FILE_BYTES = 3000`；内容 `index % 251`（第 71–75 行 `file_content`）。
+const LOCAL_FIRST_FILE_BYTES: usize = 3000;
+const LOCAL_FIRST_FILE_CONTENT_MODULUS: usize = 251;
+
+/// 流一的段长：`tests/crash_enumeration_new_pool_file_creation_stream.rs` 第 466 行。
+const LOCAL_FIRST_STREAM_SEGMENT_LENGTHS: [usize; 11] = [2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2];
+/// 流一的写数：同一份文件第 471 行（取号 2 + 暖机两次各 5 + A 29）。
+const LOCAL_FIRST_STREAM_WRITES: usize = 41;
+/// 流一的根槽写数：暖机两次各一条 + A 一条。
+const LOCAL_FIRST_STREAM_ROOT_WRITES: usize = 3;
+/// 流一全量与快档的状态数：同一份文件第 446 行 `FULL_STATES`、第 450 行 `QUICK_TIER_STATES`。
+const LOCAL_FIRST_STREAM_FULL_STATES: u64 = 16_777_260;
+const LOCAL_FIRST_STREAM_QUICK_TIER_STATES: u64 = 46;
+/// 流一每次写只取两态时的全量闭式：同一份文件第 442 行 `FULL_STATES_WITH_TWO_STATES_PER_WRITE`（单测拿段长表回算它）。
+const LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM: u64 = 16_777_240;
+
+/// 流二的段长（78 段）：`tests/crash_enumeration_fixed_script_stream.rs` 第 438–441 行。
+const LOCAL_SECOND_STREAM_SEGMENT_LENGTHS: [usize; 78] = [
+    2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2,
+    16, // 取号到写行之后的暖机
+    2, 1, 2, 24, 2, 1, 2, 20, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2,
+    28, // C、回退 D、覆盖写
+    2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1,
+    2, // 抬 F、E、卸载
+    16, 2, 1, 2, // 卸载第二次空发布与 txg 19 的轮换
+];
+/// 流二的写数：同一份文件第 448 行 `191 + 5 * 33 + (2 + 2 * 21) + 33 + (2 + 2 * 21)`。
+const LOCAL_SECOND_STREAM_WRITES: usize = 477;
+/// 流二的根槽写数：同一份文件第 482 行（`=> 19`）。
+const LOCAL_SECOND_STREAM_ROOT_WRITES: usize = 19;
+/// 流二全量与快档的状态数：同一份文件第 100 行 `FULL_STATES_THROUGH_THE_UNMOUNT`、第 114 行 `QUICK_TIER_STATES_THROUGH_THE_UNMOUNT`。
+const LOCAL_SECOND_STREAM_FULL_STATES: u64 = 1_662_648_564;
+const LOCAL_SECOND_STREAM_QUICK_TIER_STATES: u64 = 278;
+/// 流二每次写只取两态时的全量闭式：同一份文件第 96 行（单测拿段长表回算它）。
+const LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM: u64 = 1_662_648_449;
+/// E 的数据单元落点：同一份文件第 318 行（`50176`）。
+const LOCAL_SECOND_STREAM_E_DATA_UNIT_SLOT: u64 = 50_176;
+/// 抬 F 推的空发布数：同一份文件第 306 行（`raised.publishes.len(), 2`）。
+const LOCAL_SECOND_STREAM_RAISE_FLOOR_PUBLISHES: usize = 2;
+/// 卸载推的两次空发布的 txg：同一份文件第 336 行（`vec![CheckpointTxg(18), CheckpointTxg(19)]`）。
+const LOCAL_SECOND_STREAM_UNMOUNT_TXGS: [u64; 2] = [18, 19];
+
+/// 取样：批 1 每段内点的分母、批 2 的分母、批 3 的分母（登记 5.4）。
+const FIRST_BATCH_DIVISOR: u64 = 7;
+const SECOND_BATCH_DIVISOR: u64 = 14;
+const THIRD_BATCH_DIVISOR: u64 = 23;
+/// 批 1 在带撕裂写的段里另取几个带「1」的状态、每段最多扫几个序号（登记 5.4）。
+const TORN_STATES_PER_SEGMENT: usize = 4;
+const TORN_SCAN_ORDINALS_PER_SEGMENT: u64 = 100_000;
+
+/// 两条流的名字（登记 5.3 的目录名）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum StreamName {
+    NewPoolFileCreation,
+    FixedScriptThroughEThenUnmount,
+}
+
+impl StreamName {
+    fn directory_name(self) -> &'static str {
+        match self {
+            Self::NewPoolFileCreation => "new_pool_file_creation",
+            Self::FixedScriptThroughEThenUnmount => "fixed_script_through_e_then_unmount",
+        }
+    }
+}
+
+const STREAM_NAMES: [StreamName; 2] = [
+    StreamName::NewPoolFileCreation,
+    StreamName::FixedScriptThroughEThenUnmount,
+];
+
+// ───────────────────────────── 流二的内容生成式（照抄测试文件） ─────────────────────────────
+
+/// `tests/crash_enumeration_fixed_script_stream.rs` 第 50–56 行。
+const SECOND_FILE_BYTES: usize = 4100;
+
+fn second_content() -> Vec<u8> {
+    (0..SECOND_FILE_BYTES)
+        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
+        .collect()
+}
+
+/// 同一份文件第 131–143 行。
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
+/// 第一个文件的内容按本地常量现算，与 `common::file_content()` 回比（S1）。
+fn local_first_file_content() -> Vec<u8> {
+    (0..LOCAL_FIRST_FILE_BYTES)
+        .map(|index| {
+            u8::try_from(index % LOCAL_FIRST_FILE_CONTENT_MODULUS).expect("模 251 小于 256")
+        })
+        .collect()
+}
+
+// ───────────────────────────── 驱动方记的调用（calls.tsv、call-outcomes.tsv） ─────────────────────────────
+
+/// 一次调用的结局：成功；实现交回一个错误（成员名取 `Debug` 文本的第一个标识符）；实现 panic（实现自崩，登记第一节读法 5）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum CallOutcome {
+    Succeeded,
+    ReturnedError(String),
+    Panicked(String),
+}
+
+impl CallOutcome {
+    /// `calls.tsv` 最后一列：E170（O3规约执行器第一版） 登记 5.3 写的 `ok` / `err`；panic 的流不导出，这里不会写到 panic。
+    fn calls_file_word(&self) -> &'static str {
+        match self {
+            Self::Succeeded => "ok",
+            Self::ReturnedError(_) | Self::Panicked(_) => "err",
+        }
+    }
+    /// `call-outcomes.tsv` 里的结局列：成功是 `ok`，错误是 `err:<成员>`，panic 是 `panic`（消息另写结果行）。
+    fn outcome_word(&self) -> String {
+        match self {
+            Self::Succeeded => "ok".to_string(),
+            Self::ReturnedError(member) => format!("err:{member}"),
+            Self::Panicked(_) => "panic".to_string(),
+        }
+    }
+}
+
+/// 理想模型对一次调用的判定（O1-对拍，登记 5.3「调用与模型的对应」那张表）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum ModelVerdictOfCall {
+    /// 模型这一步不交答：取号、暖机（两次都返回之后一起推一步）与卸载（`model.rs` 里没有 `answer_unmount` 一族）。
+    NotModelled,
+    /// 对得上，模型往前走。
+    Agrees,
+    /// 对不上的第一格；模型停在这里，此后的调用不比（读法 6）。
+    Disagrees {
+        aspect: &'static str,
+        model_answer: String,
+        implementation_answer: String,
+    },
+    /// 前面有一次对不上（或前面有一次调用出了错），这次不比。
+    SkippedAfterTheModelStopped,
+}
+
+impl ModelVerdictOfCall {
+    fn word(&self) -> String {
+        match self {
+            Self::NotModelled => "not_modelled".to_string(),
+            Self::Agrees => "agrees".to_string(),
+            Self::Disagrees { aspect, .. } => format!("disagrees:{aspect}"),
+            Self::SkippedAfterTheModelStopped => "skipped_after_the_model_stopped".to_string(),
+        }
+    }
+}
+
+/// 驱动方调的一次 API：标签、它发出的第一条操作的流序下标、返回点（返回时录制流的长度，都从 mkfs 之后数）、结局、模型的判定。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct RecordedCall {
+    label: &'static str,
+    first_operation: usize,
+    return_point: usize,
+    outcome: CallOutcome,
+    model: ModelVerdictOfCall,
+}
+
+/// 一条停机条款 S1 的回比：本地值与 crates 那一侧的值，两边按文本比。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct StopCheck {
+    name: String,
+    local: String,
+    crates: String,
+}
+
+impl StopCheck {
+    fn new(name: &str, local: impl std::fmt::Debug, crates: impl std::fmt::Debug) -> Self {
+        Self {
+            name: name.to_string(),
+            local: format!("{local:?}"),
+            crates: format!("{crates:?}"),
+        }
+    }
+    fn holds(&self) -> bool {
+        self.local == self.crates
+    }
+}
+
+/// 阳性对照要改的那一处观测（登记 5.6「坏实现」行的阳性一栏）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum PositiveControl {
+    /// 流二第一次 `publish` 的观测里那一版的文件内容末字节翻一位，再交 `judge_and_advance`。
+    ComparisonOfTheCalls,
+    /// 批 1 里第一个「实现读回了文件」的态，读回内容末字节翻一位，再交 `crash_recovery_disagreement`。
+    CrashJudge,
+    /// 流一全持久态叠已知坏镜像族第一条，再交 `check_pool_image`。
+    PoolChecker,
+}
+
+impl PositiveControl {
+    fn parse(word: &str) -> Option<Self> {
+        match word {
+            "O1-对拍" => Some(Self::ComparisonOfTheCalls),
+            "O1-崩溃判官" => Some(Self::CrashJudge),
+            "O2" => Some(Self::PoolChecker),
+            _ => None,
+        }
+    }
+}
+
+/// 关掉的那条 oracle（Q3-核，登记第一节读法 9）：关掉的列不算、写成 `off`，别的两列照常。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum DisabledOracle {
+    ComparisonAndCrashJudge,
+    PoolChecker,
+}
+
+/// 一次跑的开关：阳性对照、关一条。
+#[derive(Clone, Copy, Debug, Default)]
+struct RunSwitches {
+    positive_control: Option<PositiveControl>,
+    disabled_oracle: Option<DisabledOracle>,
+}
+
+/// 实现 panic 的消息：`String` 或 `&str`，别的写 `non_text_panic`。
+fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
+    payload
+        .downcast_ref::<String>()
+        .cloned()
+        .or_else(|| {
+            payload
+                .downcast_ref::<&str>()
+                .map(|text| (*text).to_string())
+        })
+        .unwrap_or_else(|| "non_text_panic".to_string())
+}
+
+/// 错误值 `Debug` 文本的第一个标识符（`Variant(..)`、`Variant { .. }`、`Variant`）：成员名。
+fn error_member_of(error: &dyn std::fmt::Debug) -> String {
+    let text = format!("{error:?}");
+    text.chars()
+        .take_while(|character| character.is_alphanumeric() || *character == '_')
+        .collect()
+}
+
+/// 一条流驱动时手里的东西：录制流、调用表、理想模型与它提交过的版本目录。每次调用实现的入口都经 [`StreamDriver::call`]：
+/// 记返回点与结局，入口 panic 记「实现自崩」，错误或 panic 之后这条流不再往下驱动。
+struct StreamDriver {
+    name: StreamName,
+    stream: SharedStream,
+    mkfs_operation_count: usize,
+    switches: RunSwitches,
+    calls: Vec<RecordedCall>,
+    model: IdealModel,
+    /// 每一步之后把模型根环里的每一版并进来（崩溃注入的做法）：O1-崩溃判官拿它问「恢复到的这一版是不是模型提交过的」。
+    committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
+    /// 模型不再往前走的那次调用：第一次对不上的、没有对应答的 `unmount`、出了错或 panic 的调用（读法 6）。
+    model_stopped_at_call: Option<usize>,
+    implementation_crash: Option<String>,
+    halted: bool,
+    comparison_control_applied: bool,
+}
+
+impl StreamDriver {
+    fn new(
+        name: StreamName,
+        stream: SharedStream,
+        mkfs_operation_count: usize,
+        switches: RunSwitches,
+    ) -> Self {
+        let parameters = common::parameters();
+        let model = IdealModel::after_make_filesystem_on_a_journal_ring_of(
+            ModelPoolGeometry {
+                devices: (0..LOCAL_DEVICE_COUNT)
+                    .map(|device_number| {
+                        ModelDeviceIdentity(u32::try_from(device_number).expect("设备号"))
+                    })
+                    .collect(),
+                device_size_in_bytes: LOCAL_DEVICE_BYTES,
+            },
+            parameters.geometry.journal_ring_bytes,
+        );
+        let mut driver = Self {
+            name,
+            stream,
+            mkfs_operation_count,
+            switches,
+            calls: Vec::new(),
+            model,
+            committed_versions: BTreeMap::new(),
+            model_stopped_at_call: None,
+            implementation_crash: None,
+            halted: false,
+            comparison_control_applied: false,
+        };
+        driver.merge_committed_versions();
+        driver
+    }
+
+    /// O1 开着：关一条（Q3-核）关掉 O1 时模型一步都不推、对拍与崩溃判官都不算。
+    fn model_is_enabled(&self) -> bool {
+        self.switches.disabled_oracle != Some(DisabledOracle::ComparisonAndCrashJudge)
+    }
+
+    fn operations_so_far(&self) -> usize {
+        self.stream.operation_count() - self.mkfs_operation_count
+    }
+
+    fn merge_committed_versions(&mut self) {
+        for (key, file) in self.model.committed_versions() {
+            self.committed_versions.insert(key, file);
+        }
+    }
+
+    fn stop_the_model_at(&mut self, call_index: usize) {
+        self.model_stopped_at_call.get_or_insert(call_index);
+    }
+
+    /// 调一次实现的入口并记一行：调之前与返回时各读一次流长。交回 `Err(Some(错误))` 是实现交回了错误（调用方照错误造拒绝的观测），
+    /// `Err(None)` 是 panic。
+    fn call<Output, Failure: std::fmt::Debug>(
+        &mut self,
+        label: &'static str,
+        run: impl FnOnce() -> Result<Output, Failure>,
+    ) -> Result<Output, Option<Failure>> {
+        let first_operation = self.operations_so_far();
+        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(run));
+        let return_point = self.operations_so_far();
+        let call_index = self.calls.len();
+        let (outcome, returned) = match result {
+            Ok(Ok(output)) => (CallOutcome::Succeeded, Ok(output)),
+            Ok(Err(failure)) => (
+                CallOutcome::ReturnedError(error_member_of(&failure)),
+                Err(Some(failure)),
+            ),
+            Err(payload) => {
+                let text = panic_text(&*payload);
+                self.implementation_crash = Some(format!("call_{call_index}_{label}:{text}"));
+                (CallOutcome::Panicked(text), Err(None))
+            }
+        };
+        if outcome != CallOutcome::Succeeded {
+            self.halted = true;
+            self.stop_the_model_at(call_index);
+        }
+        self.calls.push(RecordedCall {
+            label,
+            first_operation,
+            return_point,
+            outcome,
+            model: ModelVerdictOfCall::NotModelled,
+        });
+        returned
+    }
+
+    /// 刚记的这次调用模型没有对应的答（卸载）：模型停在这里。
+    fn note_that_the_model_has_no_answer_for_the_last_call(&mut self) {
+        let call_index = self.calls.len() - 1;
+        self.stop_the_model_at(call_index);
+    }
+
+    /// 取号、暖机两次都返回之后，模型推一步（`acquire_and_warm_up_in_the_make_filesystem_process`）；两次不比。
+    fn advance_the_model_past_the_acquire_and_the_warm_up(&mut self) {
+        if self.model_is_enabled() && self.model_stopped_at_call.is_none() {
+            self.model
+                .acquire_and_warm_up_in_the_make_filesystem_process();
+            self.merge_committed_versions();
+        }
+    }
+
+    /// 拿刚记的这次调用的结局与模型的答案比（与 `history.rs` 的 `judge_by_model` 同一种比法：`answer` 出答案、`judge_and_advance` 比并推）。
+    /// 对得上，模型往前走并把它根环里的版本并进目录；对不上，记第一格并让模型停在这次调用。同一次调用可以交几次观测
+    /// （可写挂载之后准入推的抬 F）：前面一处对不上，后面不再比。
+    fn judge(
+        &mut self,
+        answer: impl FnOnce(&IdealModel) -> Result<ModelAnswer, ModelDisagreement>,
+        mut observed: ObservedOutcome,
+    ) {
+        if !self.model_is_enabled() {
+            return;
+        }
+        let call_index = self.calls.len() - 1;
+        if self
+            .model_stopped_at_call
+            .is_some_and(|stopped| stopped < call_index)
+        {
+            self.calls[call_index].model = ModelVerdictOfCall::SkippedAfterTheModelStopped;
+            return;
+        }
+        if matches!(
+            self.calls[call_index].model,
+            ModelVerdictOfCall::Disagrees { .. }
+        ) {
+            return;
+        }
+        if self.switches.positive_control == Some(PositiveControl::ComparisonOfTheCalls)
+            && self.name == StreamName::FixedScriptThroughEThenUnmount
+            && self.calls[call_index].label == "publish"
+            && !self.comparison_control_applied
+        {
+            if let ObservedOutcome::Succeeded(ObservedEffect::Publishes { roots, .. }) =
+                &mut observed
+            {
+                if let Some(ObservedFile::Decoded(content)) =
+                    roots.first_mut().map(|root| &mut root.file)
+                {
+                    if let Some(last_byte) = content.last_mut() {
+                        *last_byte ^= 1;
+                        self.comparison_control_applied = true;
+                    }
+                }
+            }
+        }
+        let verdict =
+            answer(&self.model).and_then(|answer| self.model.judge_and_advance(&answer, &observed));
+        match verdict {
+            Ok(_) => {
+                self.calls[call_index].model = ModelVerdictOfCall::Agrees;
+                self.merge_committed_versions();
+            }
+            Err(disagreement) => {
+                self.calls[call_index].model = ModelVerdictOfCall::Disagrees {
+                    aspect: disagreement.aspect.name(),
+                    model_answer: disagreement.model_answer,
+                    implementation_answer: disagreement.implementation_answer,
+                };
+                self.stop_the_model_at(call_index);
+            }
+        }
+    }
+
+    /// 实现交回错误时交给模型的观测：成员映射成的理由、拒之前做完几次发布、录制流在这一步里有没有多出写或屏障。
+    fn refusal(
+        &self,
+        error: &dyn std::fmt::Debug,
+        reason: singlefs_harness::model::ObservedRefusalReason,
+        publishes_completed: usize,
+        stream_length_before: usize,
+        reported_ceiling: Option<singlefs_harness::model::ModelCheckpointTxg>,
+        slot_still_bad: Option<singlefs_harness::model::ModelRingPosition>,
+    ) -> ObservedOutcome {
+        ObservedOutcome::Refused {
+            member: error_member_of(error),
+            reason,
+            publishes_completed,
+            wrote_anything: self.stream.operation_count() != stream_length_before,
+            reported_ceiling,
+            reported_root_ring_slot_still_bad_after_one_reread: slot_still_bad,
+        }
+    }
+}
+
+// ───────────────────────────── 驱动两条流（登记 5.3：不 `.expect`、跟着推模型） ─────────────────────────────
+
+fn version(instance: u32, checkpoint_txg: u64, content: Vec<u8>) -> PublishedVersion {
+    PublishedVersion {
+        instance: InstanceGeneration(instance),
+        checkpoint_txg: CheckpointTxg(checkpoint_txg),
+        content,
+    }
+}
+
+/// 一条流驱动时的池：只做过 mkfs 的两块文件镜像（`FormattedPool` 管镜像文件，丢掉就删）、分配器、现行那一版、层 0 手写 oracle 的版本表。
+/// 流一照抄 `tests/common/mod.rs` 第 677 行 `build_pool` 的取号、暖机、新池新建文件三步，流二照抄
+/// `tests/crash_enumeration_fixed_script_stream.rs` 第 218 行起的 `Script::ReuseAfterRaisingFloorThenNormalUnmount` 那一支；
+/// 每一步都经 [`StreamDriver::call`]。
+struct LiveStream {
+    driver: StreamDriver,
+    formatted: common::FormattedPool,
+    allocator: PoolAllocator,
+    instance: Option<InstanceGeneration>,
+    warm_up: Option<WarmUpOutput>,
+    output: Option<TransactionOutput>,
+    versions: Vec<PublishedVersion>,
+    checks: Vec<StopCheck>,
+    /// 暖机返回时整条录制流（含 mkfs）的长度。
+    warm_up_operation_count: usize,
+}
+
+impl LiveStream {
+    fn start(name: StreamName, tag: &str, switches: RunSwitches) -> Self {
+        let formatted = common::format_pool(tag);
+        let mut allocator = PoolAllocator::new(vec![
+            DeviceFreeMap::new(DeviceIdentity(0), LOCAL_DEVICE_BYTES),
+            DeviceFreeMap::new(DeviceIdentity(1), LOCAL_DEVICE_BYTES),
+        ]);
+        allocator.mark_format_time_units(
+            Placement {
+                slot: INSTANCE_TABLE_SLOT,
+                span: 2,
+            },
+            Placement {
+                slot: TREE_TABLE_GENESIS_SLOT,
+                span: 1,
+            },
+        );
+        let driver = StreamDriver::new(
+            name,
+            formatted.stream.clone(),
+            formatted.mkfs_operation_count,
+            switches,
+        );
+        Self {
+            driver,
+            formatted,
+            allocator,
+            instance: None,
+            warm_up: None,
+            output: None,
+            versions: Vec::new(),
+            checks: Vec::new(),
+            warm_up_operation_count: 0,
+        }
+    }
+
+    /// 取号、暖机两次调用；两次都返回之后模型推一步。
+    fn step_acquire_and_warm_up(&mut self) -> bool {
+        let parameters = common::parameters();
+        let genesis = self.formatted.genesis.clone();
+        let devices = self
+            .formatted
+            .devices
+            .as_mut()
+            .expect("format_pool 交回开着的设备");
+        let mut pool = PoolWriter::new(&parameters, devices);
+        let Ok(instance) = self.driver.call("acquire", || acquire_instance(&mut pool)) else {
+            return false;
+        };
+        self.checks.push(StopCheck::new(
+            "first_acquired_instance_is_one",
+            InstanceGeneration(1),
+            instance,
+        ));
+        let Ok(warmed) = self
+            .driver
+            .call("warm_up", || warm_up(&mut pool, &genesis.root, instance))
+        else {
+            return false;
+        };
+        self.warm_up_operation_count = self.driver.stream.operation_count();
+        self.driver
+            .advance_the_model_past_the_acquire_and_the_warm_up();
+        self.instance = Some(instance);
+        self.warm_up = Some(warmed);
+        true
+    }
+
+    /// 新池新建文件（`publish_first_file`）：按模型的 `answer_publish_first_file` 比。
+    fn step_publish_first_file(&mut self) -> bool {
+        let parameters = common::parameters();
+        let content = local_first_file_content();
+        let instance = self.instance.expect("取号成功才走到这一步");
+        let warmed = self.warm_up.as_ref().expect("暖机成功才走到这一步");
+        let devices = self
+            .formatted
+            .devices
+            .as_mut()
+            .expect("format_pool 交回开着的设备");
+        let mut pool = PoolWriter::new(&parameters, devices);
+        let stream_length_before = self.driver.stream.operation_count();
+        let result = self.driver.call("publish", || {
+            publish_first_file(
+                &mut pool,
+                &mut self.allocator,
+                warmed.roots.last().expect("暖机两代根"),
+                FirstFile {
+                    content: &content,
+                    write_time_seconds: LOCAL_FIXED_WRITE_TIME_SECONDS,
+                },
+                instance,
+                &warmed.last_record_bytes,
+            )
+        });
+        match result {
+            Ok(output) => {
+                self.driver.judge(
+                    |model| model.answer_publish_first_file(&content),
+                    ObservedOutcome::Succeeded(ObservedEffect::Publishes {
+                        roots: vec![observed_root_of_file_version(&output)],
+                        reported_ceiling: None,
+                    }),
+                );
+                self.output = Some(output);
+                true
+            }
+            Err(Some(error)) => {
+                let observed = self.driver.refusal(
+                    &error,
+                    refusal_reason_of_publish_error(&error),
+                    0,
+                    stream_length_before,
+                    None,
+                    None,
+                );
+                self.driver
+                    .judge(|model| model.answer_publish_first_file(&content), observed);
+                false
+            }
+            Err(None) => false,
+        }
+    }
+
+    /// 覆盖写（`publish_overwrite`，写时刻取本地常量 + 60）：按模型的 `answer_publish_overwrite` 比。
+    fn step_overwrite(&mut self, content: &[u8], instance: InstanceGeneration) -> bool {
+        let parameters = common::parameters();
+        let previous = self.output.as_ref().expect("上一版带文件才走到这一步");
+        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        let stream_length_before = self.driver.stream.operation_count();
+        let result = self.driver.call("publish", || {
+            publish_overwrite(
+                &mut writer,
+                &mut self.allocator,
+                previous,
+                FirstFile {
+                    content,
+                    write_time_seconds: LOCAL_FIXED_WRITE_TIME_SECONDS + 60,
+                },
+                instance,
+            )
+        });
+        match result {
+            Ok(output) => {
+                self.driver.judge(
+                    |model| model.answer_publish_overwrite(content),
+                    ObservedOutcome::Succeeded(ObservedEffect::Publishes {
+                        roots: vec![observed_root_of_file_version(&output)],
+                        reported_ceiling: None,
+                    }),
+                );
+                self.output = Some(output);
+                true
+            }
+            Err(Some(error)) => {
+                let observed = self.driver.refusal(
+                    &error,
+                    refusal_reason_of_publish_error(&error),
+                    0,
+                    stream_length_before,
+                    None,
+                    None,
+                );
+                self.driver
+                    .judge(|model| model.answer_publish_overwrite(content), observed);
+                false
+            }
+            Err(None) => false,
+        }
+    }
+
+    /// 进程重开之后的可写挂载（取号、写行、暖机）：按模型的 `answer_mount_writable` 比；准入不够时写行之后推的几串抬 F 逐串再比
+    /// （`history.rs` 的 `settle_mount` 同一种比法）。
+    fn step_mount_writable(&mut self) -> bool {
+        if self.driver.model_is_enabled() {
+            self.driver.model.close_session();
+        }
+        let parameters = common::parameters();
+        let stream_length_before = self.driver.stream.operation_count();
+        let formatted = &mut self.formatted;
+        let result = self.driver.call("mount_writable", || {
+            let mut devices = formatted.reopen_recorded();
+            let mounted = mount_writable(&parameters, &mut devices);
+            formatted.devices = Some(devices);
+            mounted
+        });
+        match result {
+            Ok(mounted) => {
+                let (observed, _carried_to_zero_unit_publishes) =
+                    observed_mount_carrying_to_zero_unit_publishes(&mounted);
+                self.driver.judge(
+                    |model| Ok(model.answer_mount_writable()),
+                    ObservedOutcome::Succeeded(observed),
+                );
+                for raised in mounted.output.space_admission.floor_raises() {
+                    let raised_floor = raised
+                        .publishes
+                        .first()
+                        .expect("抬 F 做成至少推一次空发布")
+                        .root
+                        .rollback_floor;
+                    self.driver.judge(
+                        |model| {
+                            model.answer_raise_rollback_floor(
+                                singlefs_harness::model::ModelCheckpointTxg(raised_floor.0),
+                            )
+                        },
+                        ObservedOutcome::Succeeded(ObservedEffect::Publishes {
+                            roots: raised
+                                .publishes
+                                .iter()
+                                .map(observed_root_of_file_version)
+                                .collect(),
+                            reported_ceiling: Some(singlefs_harness::model::ModelCheckpointTxg(
+                                raised.ceiling.0,
+                            )),
+                        }),
+                    );
+                }
+                self.allocator = mounted.allocator;
+                self.output = mounted.current.into_file_version();
+                if self.output.is_none() {
+                    self.driver.halted = true;
+                    self.driver.implementation_crash =
+                        Some("mount_writable_returned_a_version_without_a_file".to_string());
+                }
+                self.output.is_some()
+            }
+            Err(Some(error)) => {
+                let observed = self.driver.refusal(
+                    &error,
+                    refusal_reason_of_mount_error(&error),
+                    0,
+                    stream_length_before,
+                    None,
+                    root_ring_slot_still_bad_after_one_reread_of_mount_error(&error),
+                );
+                self.driver
+                    .judge(|model| Ok(model.answer_mount_writable()), observed);
+                false
+            }
+            Err(None) => false,
+        }
+    }
+
+    /// 同一个会话里挂着的时候回退到 A 的根（一次向前发布）：按模型的 `answer_rollback_while_mounted` 比。
+    fn step_roll_back(&mut self) -> bool {
+        let target = RollbackTarget {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(3),
+        };
+        let target_key = model_root_key(target.instance, target.checkpoint_txg);
+        let parameters = common::parameters();
+        let output = self.output.as_mut().expect("上一版带文件才走到这一步");
+        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
+        let stream_length_before = self.driver.stream.operation_count();
+        let result = self.driver.call("roll_back", || {
+            roll_back_by_a_forward_publish(
+                &parameters,
+                devices,
+                &mut self.allocator,
+                &mut *output,
+                target,
+            )
+        });
+        match result {
+            Ok(_rolled_back) => {
+                let observed = ObservedOutcome::Succeeded(ObservedEffect::Publishes {
+                    roots: vec![observed_root_of_file_version(output)],
+                    reported_ceiling: None,
+                });
+                self.driver.judge(
+                    |model| model.answer_rollback_while_mounted(target_key),
+                    observed,
+                );
+                true
+            }
+            Err(Some(error)) => {
+                let observed = self.driver.refusal(
+                    &error,
+                    refusal_reason_of_rollback_error(&error),
+                    0,
+                    stream_length_before,
+                    None,
+                    root_ring_slot_still_bad_after_one_reread_of_rollback_error(&error),
+                );
+                self.driver.judge(
+                    |model| model.answer_rollback_while_mounted(target_key),
+                    observed,
+                );
+                false
+            }
+            Err(None) => false,
+        }
+    }
+
+    /// 抬 F 到 txg 11（`raise_rollback_floor`，影子账开）：按模型的 `answer_raise_rollback_floor` 比（拒与成都要与实现报的上限相等）。
+    fn step_raise_floor(&mut self) -> bool {
+        let new_floor = CheckpointTxg(11);
+        let parameters = common::parameters();
+        let mut current = self.output.clone().expect("上一版带文件才走到这一步");
+        let txg_before_the_raise = current.root.checkpoint_txg.0;
+        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
+        let stream_length_before = self.driver.stream.operation_count();
+        let result = self.driver.call("raise_floor", || {
+            raise_rollback_floor(
+                &parameters,
+                devices,
+                &mut self.allocator,
+                &mut current,
+                new_floor,
+                ShadowLedger::On,
+            )
+        });
+        let publishes_completed = usize::try_from(
+            current
+                .root
+                .checkpoint_txg
+                .0
+                .saturating_sub(txg_before_the_raise),
+        )
+        .expect("一次抬 F 至多推根环区域数那么多次");
+        self.output = Some(current);
+        let floor = singlefs_harness::model::ModelCheckpointTxg(new_floor.0);
+        match result {
+            Ok(raised) => {
+                self.checks.push(StopCheck::new(
+                    "second_stream_raise_floor_publishes",
+                    LOCAL_SECOND_STREAM_RAISE_FLOOR_PUBLISHES,
+                    raised.publishes.len(),
+                ));
+                let observed = ObservedOutcome::Succeeded(ObservedEffect::Publishes {
+                    roots: raised
+                        .publishes
+                        .iter()
+                        .map(observed_root_of_file_version)
+                        .collect(),
+                    reported_ceiling: Some(singlefs_harness::model::ModelCheckpointTxg(
+                        raised.ceiling.0,
+                    )),
+                });
+                self.driver
+                    .judge(|model| model.answer_raise_rollback_floor(floor), observed);
+                true
+            }
+            Err(Some(error)) => {
+                let observed = self.driver.refusal(
+                    &error,
+                    refusal_reason_of_mount_error(&error),
+                    publishes_completed,
+                    stream_length_before,
+                    reported_ceiling_of_mount_error(&error),
+                    root_ring_slot_still_bad_after_one_reread_of_mount_error(&error),
+                );
+                self.driver
+                    .judge(|model| model.answer_raise_rollback_floor(floor), observed);
+                false
+            }
+            Err(None) => false,
+        }
+    }
+
+    /// 正常卸载（`unmount`，入口记在录制流里）：模型没有卸载的答，不比；模型停在这里。
+    fn step_unmount(&mut self) -> bool {
+        let mut current = PoolVersion::WithFile(self.output.clone().expect("上一版带文件"));
+        let stream = self.driver.stream.clone();
+        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
+        let result = self.driver.call("unmount", || {
+            stream.record_entry(RecordedPublishEntry::Unmount, || {
+                unmount(
+                    &common::parameters(),
+                    devices,
+                    &mut self.allocator,
+                    &mut current,
+                    ShadowLedger::On,
+                )
+            })
+        });
+        self.driver
+            .note_that_the_model_has_no_answer_for_the_last_call();
+        self.output = current.into_file_version();
+        match result {
+            Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
+                self.checks.push(StopCheck::new(
+                    "second_stream_unmount_txgs",
+                    LOCAL_SECOND_STREAM_UNMOUNT_TXGS.to_vec(),
+                    unmounted_txgs_of(&raised),
+                ));
+                true
+            }
+            Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) | Err(_) => false,
+        }
+    }
+}
+
+fn unmounted_txgs_of(raised: &UnmountRaisedTheFloor) -> Vec<u64> {
+    raised
+        .publishes
+        .iter()
+        .map(|publish| publish.root.checkpoint_txg.0)
+        .collect()
+}
+
+impl LiveStream {
+    /// 整条流的脚本：任一步出错或 panic 就停（这条流不再往下驱动）。流一是取号、暖机、新池新建文件；流二接着照
+    /// `tests/crash_enumeration_fixed_script_stream.rs` 的脚本走到 E 再正常卸载。每一步成功之后才往层 0 手写 oracle 的版本表里记一版。
+    fn run_the_script(&mut self) {
+        if !self.step_acquire_and_warm_up() || !self.step_publish_first_file() {
+            return;
+        }
+        if self.driver.name == StreamName::NewPoolFileCreation {
+            return;
+        }
+        self.versions
+            .push(version(1, 3, local_first_file_content()));
+        if !self.step_overwrite(&second_content(), InstanceGeneration(1)) {
+            return;
+        }
+        self.versions.push(version(1, 4, second_content()));
+        // 进程重开：取号 2、写行、暖机两次（txg 5 / 6 落盘 0、txg 7 落盘 1），文件还是第二次的内容。
+        if !self.step_mount_writable() {
+            return;
+        }
+        for checkpoint_txg in 5..=7 {
+            self.versions
+                .push(version(2, checkpoint_txg, second_content()));
+        }
+        if !self.step_overwrite(&third_content(), InstanceGeneration(2)) {
+            return;
+        }
+        self.versions.push(version(2, 8, third_content()));
+        // 同一个会话里挂着的时候回退到 A 的根：一次向前发布 D（txg 9 落盘 0），文件回到第一次的内容。
+        if !self.step_roll_back() {
+            return;
+        }
+        self.versions
+            .push(version(2, 9, local_first_file_content()));
+        let mut latest = Vec::new();
+        for (checkpoint_txg, seed) in [(10u64, 13usize), (11, 17), (12, 19), (13, 23), (14, 29)] {
+            latest = later_content(seed);
+            if !self.step_overwrite(&latest, InstanceGeneration(2)) {
+                return;
+            }
+            self.versions
+                .push(version(2, checkpoint_txg, latest.clone()));
+        }
+        if !self.step_raise_floor() {
+            return;
+        }
+        for checkpoint_txg in 15..=16 {
+            self.versions
+                .push(version(2, checkpoint_txg, latest.clone()));
+        }
+        if !self.step_overwrite(&later_content(31), InstanceGeneration(2)) {
+            return;
+        }
+        let reuse_slot = self
+            .output
+            .as_ref()
+            .and_then(|output| output.data_pointers.first())
+            .and_then(|pointer| pointer.locations.first())
+            .map(|location| location.slot.0);
+        self.checks.push(StopCheck::new(
+            "second_stream_e_data_unit_slot",
+            Some(LOCAL_SECOND_STREAM_E_DATA_UNIT_SLOT),
+            reuse_slot,
+        ));
+        self.versions.push(version(2, 17, later_content(31)));
+        if !self.step_unmount() {
+            return;
+        }
+        for checkpoint_txg in 18..=19 {
+            self.versions
+                .push(version(2, checkpoint_txg, later_content(31)));
+        }
+    }
+}
+
+/// 一条流驱动完之后的样子：mkfs 之后的基镜像、mkfs 之后的录制流、写表与段、写表每项在录制流里的下标、调用表、被判的根、版本表、
+/// 模型提交过的版本目录、S1 回比，以及驱动怎么结束的。
+struct PreparedStream {
+    name: StreamName,
+    base: MemoryPool,
+    operations: Vec<RetainedOperation>,
+    writes: Vec<RetainedWrite>,
+    segments: Vec<Vec<usize>>,
+    stream_indexes: Vec<usize>,
+    calls: Vec<RecordedCall>,
+    judged_root_index: usize,
+    versions: Vec<PublishedVersion>,
+    committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
+    /// 模型不再往前走的那次调用（`None`：整条流的每次调用都比过了）。
+    model_stopped_at_call: Option<usize>,
+    checks: Vec<StopCheck>,
+    ending: StreamEnding,
+}
+
+/// 驱动怎么结束的：每一步都成功；某一步交回错误（这条流在那里截断，仍照常判）；某一步 panic（实现自崩，这条流不导出）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum StreamEnding {
+    Completed,
+    StoppedByAnError { call_index: usize, member: String },
+    ImplementationCrashed { what: String },
+}
+
+/// 驱动一条流并取出它的基镜像、写表与段。`check_against_the_crates` 只在基线那一趟开：流一整条流逐条与 `build_pool` 比（S1）。
+fn prepare_stream(
+    name: StreamName,
+    switches: RunSwitches,
+    check_against_the_crates: bool,
+) -> PreparedStream {
+    let tag = format!("e171-{}", name.directory_name());
+    let mut live = LiveStream::start(name, &tag, switches);
+    live.run_the_script();
+    let ending = match (&live.driver.implementation_crash, live.driver.halted) {
+        (Some(what), _) => StreamEnding::ImplementationCrashed { what: what.clone() },
+        (None, true) => {
+            let call_index = live.driver.calls.len() - 1;
+            StreamEnding::StoppedByAnError {
+                call_index,
+                member: live.driver.calls[call_index].outcome.outcome_word(),
+            }
+        }
+        (None, false) => StreamEnding::Completed,
+    };
+    let all_operations = live.formatted.retained_operations();
+    let mkfs_operation_count = live.formatted.mkfs_operation_count;
+    // 照抄 `tests/crash_enumeration_fixed_script_stream.rs` 第 198–216 行：入口段（卸载入口发的那几段）一起交给切段（C557）。
+    let entry_spans: Vec<RecordedEntrySpan> = live
+        .formatted
+        .stream
+        .entry_spans()
+        .into_iter()
+        .map(|span| RecordedEntrySpan {
+            entry: span.entry,
+            operations: span.operations.start - mkfs_operation_count
+                ..span.operations.end - mkfs_operation_count,
+        })
+        .collect();
+    let operations = all_operations[mkfs_operation_count..].to_vec();
+    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
+        &operations,
+        &entry_spans,
+        &common::geometry(),
+    );
+    let base = live.formatted.memory_pool_after_mkfs();
+    let mut checks = std::mem::take(&mut live.checks);
+    if check_against_the_crates {
+        checks.extend(stream_checks_against_the_crates(
+            name,
+            &live,
+            &all_operations,
+            &operations,
+            &writes,
+            &segments,
+            &stream_indexes,
+            &base,
+        ));
+    }
+    let judged_root_index = writes
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .unwrap_or(usize::MAX);
+    let versions = if name == StreamName::NewPoolFileCreation && judged_root_index != usize::MAX {
+        // 流一的版本表照 `tests/crash_enumeration_new_pool_file_creation_stream.rs` 第 391–399 行 `single_version`。
+        let (instance, checkpoint_txg) =
+            singlefs_harness::memory_pool::root_identity_of_write(&writes[judged_root_index]);
+        vec![PublishedVersion {
+            instance,
+            checkpoint_txg,
+            content: local_first_file_content(),
+        }]
+    } else {
+        std::mem::take(&mut live.versions)
+    };
+    PreparedStream {
+        name,
+        base,
+        operations,
+        writes,
+        segments,
+        stream_indexes,
+        calls: live.driver.calls.clone(),
+        judged_root_index,
+        versions,
+        committed_versions: std::mem::take(&mut live.driver.committed_versions),
+        model_stopped_at_call: live.driver.model_stopped_at_call,
+        checks,
+        ending,
+    }
+}
+
+/// S1 里跟流有关的几条（只在基线那一趟）：段长表、写数、根槽写数、盘数与盘字节数对本地常量；流一整条流逐条与 `build_pool` 比。
+/// 与 `e170_o3_spec_executor_export.rs` 第 550–636 行同一批。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "这几样是 prepare_stream 驱动完之后手里同时有的，拆成结构体只会多一层搬运"
+)]
+fn stream_checks_against_the_crates(
+    name: StreamName,
+    live: &LiveStream,
+    all_operations: &[RetainedOperation],
+    operations: &[RetainedOperation],
+    writes: &[RetainedWrite],
+    segments: &[Vec<usize>],
+    stream_indexes: &[usize],
+    base: &MemoryPool,
+) -> Vec<StopCheck> {
+    let mkfs_operation_count = live.formatted.mkfs_operation_count;
+    let label = name.directory_name();
+    let non_barrier_operations: Vec<usize> = operations
+        .iter()
+        .enumerate()
+        .filter(|(_, retained)| retained.operation.kind != RecordedOperationKind::Barrier)
+        .map(|(stream_index, _)| stream_index)
+        .collect();
+    let (local_segment_lengths, local_writes, local_root_writes): (&[usize], usize, usize) =
+        match name {
+            StreamName::NewPoolFileCreation => (
+                &LOCAL_FIRST_STREAM_SEGMENT_LENGTHS,
+                LOCAL_FIRST_STREAM_WRITES,
+                LOCAL_FIRST_STREAM_ROOT_WRITES,
+            ),
+            StreamName::FixedScriptThroughEThenUnmount => (
+                &LOCAL_SECOND_STREAM_SEGMENT_LENGTHS,
+                LOCAL_SECOND_STREAM_WRITES,
+                LOCAL_SECOND_STREAM_ROOT_WRITES,
+            ),
+        };
+    let root_write_count = writes
+        .iter()
+        .filter(|write| write.kind == StepKind::RootRecordFua)
+        .count();
+    let mut checks = vec![
+        StopCheck::new(
+            &format!("{label}_write_order_is_the_non_barrier_operation_order"),
+            true,
+            non_barrier_operations == stream_indexes,
+        ),
+        StopCheck::new(
+            &format!("{label}_segment_lengths"),
+            local_segment_lengths.to_vec(),
+            segments.iter().map(Vec::len).collect::<Vec<usize>>(),
+        ),
+        StopCheck::new(&format!("{label}_writes"), local_writes, writes.len()),
+        StopCheck::new(
+            &format!("{label}_root_writes"),
+            local_root_writes,
+            root_write_count,
+        ),
+        StopCheck::new(
+            &format!("{label}_device_count"),
+            LOCAL_DEVICE_COUNT,
+            base.devices.len(),
+        ),
+        StopCheck::new(
+            &format!("{label}_device_bytes"),
+            LOCAL_DEVICE_BYTES,
+            base.device_size_in_bytes,
+        ),
+    ];
+    if name == StreamName::NewPoolFileCreation {
+        // 流一的驱动是 `build_pool` 的照抄：整条流（含 mkfs）逐条与 `build_pool` 那一份比。
+        let reference = common::build_pool("e171-first-stream-reference");
+        checks.push(StopCheck::new(
+            "new_pool_file_creation_operations_equal_build_pool",
+            true,
+            reference.retained_operations() == all_operations,
+        ));
+        checks.push(StopCheck::new(
+            "new_pool_file_creation_mkfs_operations_equal_build_pool",
+            mkfs_operation_count,
+            reference.mkfs_operation_count,
+        ));
+        checks.push(StopCheck::new(
+            "new_pool_file_creation_warm_up_return_point_equal_build_pool",
+            live.warm_up_operation_count,
+            reference.warm_up_operation_count,
+        ));
+    }
+    checks
+}
+// ───────────────────────────── 取样（登记 5.4） ─────────────────────────────
+
+/// 批 1 在一个非空区间 [a, b) 里取的点：a、b − 1 与 a + ⌊i·(b − 1 − a)/7⌋（i = 1..6）。
+fn first_batch_points(state_range: &std::ops::Range<u64>) -> Vec<u64> {
+    let first = state_range.start;
+    let span = state_range.end - 1 - first;
+    let mut points = vec![first, state_range.end - 1];
+    for step in 1..FIRST_BATCH_DIVISOR {
+        points.push(first + step * span / FIRST_BATCH_DIVISOR);
+    }
+    points
+}
+
+/// 批 2：a + ⌊(2i + 1)·(b − 1 − a)/14⌋（i = 0..6）。
+fn second_batch_points(state_range: &std::ops::Range<u64>) -> Vec<u64> {
+    let first = state_range.start;
+    let span = state_range.end - 1 - first;
+    (0..SECOND_BATCH_DIVISOR / 2)
+        .map(|step| first + (2 * step + 1) * span / SECOND_BATCH_DIVISOR)
+        .collect()
+}
+
+/// 批 3：a + ⌊i·(b − 1 − a)/23⌋（i = 0..23）。
+fn third_batch_points(state_range: &std::ops::Range<u64>) -> Vec<u64> {
+    let first = state_range.start;
+    let span = state_range.end - 1 - first;
+    (0..=THIRD_BATCH_DIVISOR)
+        .map(|step| first + step * span / THIRD_BATCH_DIVISOR)
+        .collect()
+}
+
+/// 带撕裂写的那一段扫了几个序号、取到几个带「1」的状态（登记 5.4「扫不到 4 个的照实报个数」）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct TornScan {
+    segment_index: usize,
+    scanned: u64,
+    found: usize,
+}
+
+/// 三批的状态序号（各自升序、互不重叠）与撕裂扫描的记录。
+struct SampledOrdinals {
+    batches: [BTreeSet<u64>; 3],
+    torn_scans: Vec<TornScan>,
+}
+
+fn sample_ordinals(
+    state_ranges_by_segment: &[std::ops::Range<u64>],
+    state_count: u64,
+    segment_has_a_torn_write: &[bool],
+    landing_vector_of_ordinal: &dyn Fn(u64) -> String,
+) -> SampledOrdinals {
+    let non_empty = || {
+        state_ranges_by_segment
+            .iter()
+            .filter(|range| !range.is_empty())
+    };
+    let mut first: BTreeSet<u64> = non_empty().flat_map(first_batch_points).collect();
+    first.insert(state_count - 1);
+    let mut torn_scans = Vec::new();
+    for (segment_index, state_range) in state_ranges_by_segment.iter().enumerate() {
+        if state_range.is_empty() || !segment_has_a_torn_write[segment_index] {
+            continue;
+        }
+        let scan_end = state_range
+            .end
+            .min(state_range.start + TORN_SCAN_ORDINALS_PER_SEGMENT);
+        let mut found = 0usize;
+        let mut scanned = 0u64;
+        for ordinal in state_range.start..scan_end {
+            if found == TORN_STATES_PER_SEGMENT {
+                break;
+            }
+            scanned += 1;
+            if landing_vector_of_ordinal(ordinal).contains('1') && first.insert(ordinal) {
+                found += 1;
+            }
+        }
+        torn_scans.push(TornScan {
+            segment_index,
+            scanned,
+            found,
+        });
+    }
+    let second: BTreeSet<u64> = non_empty()
+        .flat_map(second_batch_points)
+        .filter(|ordinal| !first.contains(ordinal))
+        .collect();
+    let third: BTreeSet<u64> = non_empty()
+        .flat_map(third_batch_points)
+        .filter(|ordinal| !first.contains(ordinal) && !second.contains(ordinal))
+        .collect();
+    SampledOrdinals {
+        batches: [first, second, third],
+        torn_scans,
+    }
+}
+
+// ───────────────────────────── 落地向量与镜像摘要（登记 5.3） ─────────────────────────────
+
+/// 把枚举写表上的持久集合折回按写序号的三值向量：原写持久记 `2`；原写没持久而它的撕裂镜像持久记 `1`；都没有记 `0`。
+fn landing_vector_of(
+    persisted: &[bool],
+    recorded_write_count: usize,
+    torn_image_index_by_write: &BTreeMap<usize, usize>,
+) -> String {
+    (0..recorded_write_count)
+        .map(|write_index| {
+            if persisted[write_index] {
+                '2'
+            } else if torn_image_index_by_write
+                .get(&write_index)
+                .is_some_and(|torn_image_index| persisted[*torn_image_index])
+            {
+                '1'
+            } else {
+                '0'
+            }
+        })
+        .collect()
+}
+
+/// 镜像摘要的一个扇区：全零的不进摘要；进的依次是盘身份 u32、扇区号 u64、那 512 字节（小端）。
+fn append_sector_to_digest_message(
+    message: &mut Vec<u8>,
+    device: u32,
+    sector: u64,
+    sector_bytes: &[u8],
+) {
+    if sector_bytes.iter().all(|byte| *byte == 0) {
+        return;
+    }
+    message.extend_from_slice(&device.to_le_bytes());
+    message.extend_from_slice(&sector.to_le_bytes());
+    message.extend_from_slice(sector_bytes);
+}
+
+/// 从 `CrashImage` 直接算镜像摘要：每块盘（身份升序）上可能不全为零的扇区——基镜像写过的扇区与这一态里持久了的带字节的写罩住的扇区——
+/// 合成连续的区间按升序读（读走 `CrashImage` 自己的 `PoolReader::read`），全零的扇区跳过。
+fn crash_image_digest(image: &CrashImage<'_>) -> [u8; DIGEST_BYTES] {
+    let mut message = Vec::new();
+    let mut device_identities = image.device_identities();
+    device_identities.sort_unstable();
+    for device in device_identities {
+        let mut sector_ranges: Vec<(u64, u64)> = image
+            .base
+            .devices
+            .get(&device)
+            .map(|sparse| {
+                sparse
+                    .written_sectors()
+                    .keys()
+                    .map(|sector| (*sector, sector + 1))
+                    .collect()
+            })
+            .unwrap_or_default();
+        for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
+            let carries_bytes = match write.contents {
+                WrittenContents::Bytes(_) => true,
+                WrittenContents::Zeros { .. } => false,
+            };
+            if *is_persisted && carries_bytes && write.device == device {
+                sector_ranges.push((
+                    write.offset.0 / LOCAL_SECTOR_BYTES,
+                    (write.offset.0 + write.length_in_bytes()).div_ceil(LOCAL_SECTOR_BYTES),
+                ));
+            }
+        }
+        for (first_sector, end_sector) in merged_sector_ranges(sector_ranges) {
+            let bytes = PoolReader::read(
+                image,
+                device,
+                DeviceOffsetInBytes(first_sector * LOCAL_SECTOR_BYTES),
+                usize::try_from((end_sector - first_sector) * LOCAL_SECTOR_BYTES)
+                    .expect("区间长装得进 usize"),
+            )
+            .expect("崩溃镜像在盘内、按扇区对齐的区间读得出");
+            for (sector_offset, sector_bytes) in bytes
+                .chunks_exact(usize::try_from(LOCAL_SECTOR_BYTES).expect("512"))
+                .enumerate()
+            {
+                let sector = first_sector + u64::try_from(sector_offset).expect("扇区下标");
+                append_sector_to_digest_message(&mut message, device.0, sector, sector_bytes);
+            }
+        }
+    }
+    sha256_digest(&message)
+}
+
+/// 扇区区间按起点排好、重叠或相接的合并。
+fn merged_sector_ranges(mut sector_ranges: Vec<(u64, u64)>) -> Vec<(u64, u64)> {
+    sector_ranges.sort_unstable();
+    let mut merged: Vec<(u64, u64)> = Vec::new();
+    for (first_sector, end_sector) in sector_ranges {
+        match merged.last_mut() {
+            Some(last) if first_sector <= last.1 => last.1 = last.1.max(end_sector),
+            Some(_) | None => merged.push((first_sector, end_sector)),
+        }
+    }
+    merged
+}
+
+// ───────────────────────────── 中性格式：写出与读回重建（登记 5.3、S4） ─────────────────────────────
+
+const BASE_SECTORS_FILE: &str = "base-sectors.bin";
+const OPERATIONS_FILE: &str = "ops.tsv";
+const OPERATIONS_PAYLOAD_FILE: &str = "ops-payload.bin";
+const CALLS_FILE: &str = "calls.tsv";
+const TORN_FILE: &str = "torn.tsv";
+const TORN_PAYLOAD_FILE: &str = "torn-payload.bin";
+const STATES_FILE: &str = "states.tsv";
+const IMPLEMENTATION_VERDICTS_FILE: &str = "impl-verdicts.tsv";
+const QUICK_TIER_FILE: &str = "quick-tier.tsv";
+const MANIFEST_FILE: &str = "manifest.tsv";
+/// manifest 记 sha256 的那几个文件，按这个次序。
+const FILES_LISTED_IN_THE_MANIFEST: [&str; 9] = [
+    BASE_SECTORS_FILE,
+    OPERATIONS_FILE,
+    OPERATIONS_PAYLOAD_FILE,
+    CALLS_FILE,
+    TORN_FILE,
+    TORN_PAYLOAD_FILE,
+    STATES_FILE,
+    IMPLEMENTATION_VERDICTS_FILE,
+    QUICK_TIER_FILE,
+];
+
+fn operation_kind_name(kind: RecordedOperationKind) -> &'static str {
+    match kind {
+        RecordedOperationKind::Write => "write",
+        RecordedOperationKind::WriteForceUnitAccess => "write_fua",
+        RecordedOperationKind::WriteZeroes => "write_zeroes",
+        RecordedOperationKind::Barrier => "barrier",
+    }
+}
+
+/// 读回来的一条操作：盘身份、种类、偏移、长度、在 `ops-payload.bin` 里的起点（清零与屏障没有）。
+struct NeutralOperation {
+    device: u32,
+    kind: RecordedOperationKind,
+    offset: u64,
+    length: u64,
+    payload_start: Option<usize>,
+}
+
+/// 从导出目录读回一条流：基镜像扇区、操作、两份载荷、撕裂表。只读这几个文件，不碰内存里的 `CrashImage`。
+struct NeutralStream {
+    base_sectors: BTreeMap<(u32, u64), Vec<u8>>,
+    operations: Vec<NeutralOperation>,
+    payload: Vec<u8>,
+    torn_bytes_by_write: BTreeMap<usize, (usize, usize)>,
+    torn_payload: Vec<u8>,
+}
+
+fn read_file(directory: &Path, file_name: &str) -> Vec<u8> {
+    std::fs::read(directory.join(file_name))
+        .unwrap_or_else(|error| panic!("读 {}：{error}", directory.join(file_name).display()))
+}
+
+fn read_text_lines(directory: &Path, file_name: &str) -> Vec<String> {
+    String::from_utf8(read_file(directory, file_name))
+        .expect("导出的 tsv 是 UTF-8")
+        .lines()
+        .map(str::to_string)
+        .collect()
+}
+
+fn parse_number<Number: std::str::FromStr>(field: &str, what: &str) -> Number {
+    field
+        .parse()
+        .unwrap_or_else(|_| panic!("{what} 解析不了：{field:?}"))
+}
+
+fn read_neutral_stream(directory: &Path) -> NeutralStream {
+    let base_bytes = read_file(directory, BASE_SECTORS_FILE);
+    let record_bytes = 4 + 8 + usize::try_from(LOCAL_SECTOR_BYTES).expect("512");
+    assert!(
+        base_bytes.len().is_multiple_of(record_bytes),
+        "base-sectors.bin 是整条记录"
+    );
+    let base_sectors = base_bytes
+        .chunks_exact(record_bytes)
+        .map(|record| {
+            let device = u32::from_le_bytes(record[0..4].try_into().expect("4 字节"));
+            let sector = u64::from_le_bytes(record[4..12].try_into().expect("8 字节"));
+            ((device, sector), record[12..].to_vec())
+        })
+        .collect();
+    let operations = read_text_lines(directory, OPERATIONS_FILE)
+        .iter()
+        .map(|line| {
+            let fields: Vec<&str> = line.split('\t').collect();
+            assert_eq!(fields.len(), 7, "ops.tsv 一行七列：{line:?}");
+            let kind = match fields[2] {
+                "write" => RecordedOperationKind::Write,
+                "write_fua" => RecordedOperationKind::WriteForceUnitAccess,
+                "write_zeroes" => RecordedOperationKind::WriteZeroes,
+                "barrier" => RecordedOperationKind::Barrier,
+                unknown => panic!("ops.tsv 里认不出的种类 {unknown:?}"),
+            };
+            NeutralOperation {
+                device: parse_number(fields[1], "盘身份"),
+                kind,
+                offset: parse_number(fields[3], "偏移"),
+                length: parse_number(fields[4], "长度"),
+                payload_start: (fields[5] != "-").then(|| parse_number(fields[5], "载荷起点")),
+            }
+        })
+        .collect();
+    let torn_bytes_by_write = read_text_lines(directory, TORN_FILE)
+        .iter()
+        .map(|line| {
+            let fields: Vec<&str> = line.split('\t').collect();
+            assert_eq!(fields.len(), 4, "torn.tsv 一行四列：{line:?}");
+            (
+                parse_number(fields[0], "写序号"),
+                (
+                    parse_number(fields[1], "撕裂字节起点"),
+                    parse_number(fields[2], "撕裂字节长度"),
+                ),
+            )
+        })
+        .collect();
+    NeutralStream {
+        base_sectors,
+        operations,
+        payload: read_file(directory, OPERATIONS_PAYLOAD_FILE),
+        torn_bytes_by_write,
+        torn_payload: read_file(directory, TORN_PAYLOAD_FILE),
+    }
+}
+
+/// 从中性格式按落地向量重建一态的镜像（按流序叠：`2` 放新字节、`1` 放 `torn-payload.bin` 里那一段、`0` 不动），算镜像摘要。
+fn rebuilt_image_digest(stream: &NeutralStream, landing_vector: &str) -> [u8; DIGEST_BYTES] {
+    let sector_bytes = usize::try_from(LOCAL_SECTOR_BYTES).expect("512");
+    let mut sectors = stream.base_sectors.clone();
+    let mut landings = landing_vector.chars();
+    let mut write_index = 0usize;
+    for operation in &stream.operations {
+        if operation.kind == RecordedOperationKind::Barrier {
+            continue;
+        }
+        let landing = landings.next().expect("落地向量与写数一样长");
+        let first_sector = operation.offset / LOCAL_SECTOR_BYTES;
+        let length = usize::try_from(operation.length).expect("写长装得进 usize");
+        let bytes: Option<&[u8]> = match (landing, operation.kind) {
+            ('0', _) => None,
+            ('2', RecordedOperationKind::WriteZeroes) => {
+                let end_sector = (operation.offset + operation.length) / LOCAL_SECTOR_BYTES;
+                let covered: Vec<(u32, u64)> = sectors
+                    .range((operation.device, first_sector)..(operation.device, end_sector))
+                    .map(|(key, _)| *key)
+                    .collect();
+                for key in covered {
+                    sectors.remove(&key);
+                }
+                None
+            }
+            ('2', _) => {
+                let start = operation.payload_start.expect("带字节的写有载荷起点");
+                Some(&stream.payload[start..start + length])
+            }
+            ('1', _) => {
+                let (start, torn_length) = stream.torn_bytes_by_write[&write_index];
+                assert_eq!(torn_length, length, "撕裂那一段罩整次写");
+                Some(&stream.torn_payload[start..start + torn_length])
+            }
+            (unknown, _) => panic!("落地向量里认不出的字符 {unknown:?}"),
+        };
+        if let Some(bytes) = bytes {
+            for (sector_offset, chunk) in bytes.chunks_exact(sector_bytes).enumerate() {
+                let sector = first_sector + u64::try_from(sector_offset).expect("扇区下标");
+                sectors.insert((operation.device, sector), chunk.to_vec());
+            }
+        }
+        write_index += 1;
+    }
+    assert!(landings.next().is_none(), "落地向量比写数长");
+    let mut message = Vec::new();
+    for ((device, sector), bytes) in &sectors {
+        append_sector_to_digest_message(&mut message, *device, *sector, bytes);
+    }
+    sha256_digest(&message)
+}
+
+fn root_text(root: Option<(InstanceGeneration, CheckpointTxg)>) -> String {
+    match root {
+        Some((instance, checkpoint_txg)) => format!("{},{}", instance.0, checkpoint_txg.0),
+        None => "none".to_string(),
+    }
+}
+
+/// outcome 的种类；走不下去的那一种带上失败的名字（空白换成下划线，不破坏 tsv 与结果行）。
+fn outcome_text(outcome: &RecoveryOutcome) -> String {
+    match outcome {
+        RecoveryOutcome::NoFile { .. } => "no_file".to_string(),
+        RecoveryOutcome::FileRead { .. } => "file_read".to_string(),
+        RecoveryOutcome::Failed { failure, .. } => format!("failed:{failure:?}")
+            .chars()
+            .map(|character| {
+                if character.is_whitespace() {
+                    '_'
+                } else {
+                    character
+                }
+            })
+            .collect(),
+    }
+}
+
+fn list_text(items: &[String]) -> String {
+    if items.is_empty() {
+        "-".to_string()
+    } else {
+        items.join(",")
+    }
+}
+
+fn sha256_text(digest: &[u8; DIGEST_BYTES]) -> String {
+    digest.iter().map(|byte| format!("{byte:02x}")).collect()
+}
+
+/// 一个状态上实现的判定（登记第一节读法 6：「实现都放行」= 池级 checker 一条 Violated 都没有，且记录核对器两项都为假）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct ImplementationVerdict {
+    consulted_root: String,
+    consulted_outcome: String,
+    ignored_root: String,
+    ignored_outcome: String,
+    red_items: Vec<String>,
+    pool_checker_red: Vec<String>,
+    root_without_record: bool,
+    claimed_state_missing_unit: bool,
+}
+
+impl ImplementationVerdict {
+    fn of(judgement: &Layer0StateJudgement) -> Self {
+        let report_root = |report: &RecoveryReport| root_text(report.effective_root);
+        Self {
+            consulted_root: report_root(&judgement.consulted),
+            consulted_outcome: outcome_text(&judgement.consulted.outcome),
+            ignored_root: report_root(&judgement.ignored),
+            ignored_outcome: outcome_text(&judgement.ignored.outcome),
+            red_items: judgement.red_items(),
+            pool_checker_red: judgement
+                .pool_checker
+                .iter()
+                .filter(|(_, verdict)| match verdict {
+                    InvariantVerdict::Violated(_) => true,
+                    InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => false,
+                })
+                .map(|(invariant, _)| (*invariant).to_string())
+                .collect(),
+            root_without_record: judgement.records.root_without_record,
+            claimed_state_missing_unit: judgement.records.claimed_state_missing_unit,
+        }
+    }
+}
+
+// ───────────────────────────── 每个取样状态上的三条 oracle（O1-崩溃判官、O2；O3 在 Python 一侧） ─────────────────────────────
+
+/// O1-崩溃判官在一个状态上的结论（登记第一节读法 1、6）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum CrashJudgeVerdict {
+    /// 这一态有写落在模型没有答的调用里，或落在对拍判出不同（或出错）之后的调用里（读法 6）。
+    NotApplicable,
+    Green,
+    /// 红：模型那一侧说的话（`ModelDisagreement::implementation_answer`，空白换成下划线）。
+    Red {
+        what: String,
+    },
+    /// 关一条（Q3-核）关掉了 O1：这一列不算。
+    Off,
+}
+
+impl CrashJudgeVerdict {
+    fn word(&self) -> &'static str {
+        match self {
+            Self::NotApplicable => "na",
+            Self::Green => "green",
+            Self::Red { .. } => "red",
+            Self::Off => "off",
+        }
+    }
+}
+
+/// 一遍恢复报告的摘要（触达判法 ③，登记第一节读法 4）：择到的根、结局种类、读回内容的 sha256、施加的记录条数。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct RecoverySummary {
+    root: String,
+    outcome: String,
+    read_back_sha256: String,
+    records_applied: usize,
+}
+
+impl RecoverySummary {
+    fn of(report: &RecoveryReport) -> Self {
+        Self {
+            root: root_text(report.effective_root),
+            outcome: outcome_text(&report.outcome),
+            read_back_sha256: match &report.outcome {
+                RecoveryOutcome::FileRead { content, .. } => sha256_hexadecimal(content),
+                RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. } => "-".to_string(),
+            },
+            records_applied: report.journal.prefix_applied,
+        }
+    }
+    fn text(&self) -> String {
+        format!(
+            "{}\t{}\t{}\t{}",
+            self.root, self.outcome, self.read_back_sha256, self.records_applied
+        )
+    }
+}
+
+/// 池级 checker 判红的不变量名（`Violated` 才算；`NotApplicable` 不算红）。
+fn violated_invariant_names(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<String> {
+    verdicts
+        .iter()
+        .filter(|(_, verdict)| match verdict {
+            InvariantVerdict::Violated(_) => true,
+            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => false,
+        })
+        .map(|(invariant, _)| (*invariant).to_string())
+        .collect()
+}
+
+/// 一个取样状态上看到的：批号、层 0 序号、段号、落地向量、镜像摘要、`judge_crash_image` 的判定（`impl-verdicts.tsv`）、
+/// 三条 oracle 的结论与各自耗时、触达的最后一次调用。
+struct StateObservation {
+    batch: u8,
+    ordinal: u64,
+    segment_index: usize,
+    landing_vector: String,
+    image_digest: String,
+    verdict: ImplementationVerdict,
+    judge_nanoseconds: u128,
+    mask_matches_the_facts: bool,
+    /// 这一态里持久或撕裂的写属于的最后一次调用（一个写都没落的态是 None）。
+    touched_call: Option<usize>,
+    crash_judge: CrashJudgeVerdict,
+    /// O2 直接调 `check_pool_image` 判红的不变量（关掉 O2 时 None）。
+    pool_checker_red: Option<Vec<String>>,
+    /// S3：直接调的结果与 `judge_crash_image(...).pool_checker` 判红的不变量集合相同。
+    pool_checker_matches_the_judgement: bool,
+    consulted: RecoverySummary,
+    ignored: RecoverySummary,
+    o1_nanoseconds: u128,
+    o2_nanoseconds: u128,
+    recovery_nanoseconds: u128,
+}
+
+/// 一次观测的状态：阳性对照里「批 1 第一个读回了文件的态」只翻一次。
+struct ObservationControls {
+    switches: RunSwitches,
+    crash_judge_control_pending: bool,
+}
+
+/// 把写表每一项落在哪一次调用里算出来：写在录制流里的下标落在 `first_operation` 不超过它的最后一次调用里。
+fn call_index_of_each_write(stream: &PreparedStream) -> Vec<Option<usize>> {
+    stream
+        .stream_indexes
+        .iter()
+        .map(|operation_index| {
+            stream
+                .calls
+                .iter()
+                .rposition(|call| call.first_operation <= *operation_index)
+        })
+        .collect()
+}
+
+/// O1-崩溃判官在一个状态上的判定：交法照 `crash_injection.rs` 第 1083–1107 行——「盘上已持久的最新根槽」由 `newest_persisted_root`
+/// 从写流数出（只看写表里的原写，不含撕裂镜像），读回由 `observed_read_back_after_a_crash` 从看 journal 那一遍恢复报告造，
+/// 模型那一侧给每一步之后并起来的版本目录，交给 `crash_recovery_disagreement`（S8：三处调用都在这个函数里）。
+fn crash_judge_of_the_state(
+    stream: &PreparedStream,
+    image: &CrashImage<'_>,
+    consulted: &RecoveryReport,
+    batch: u8,
+    controls: &mut ObservationControls,
+) -> CrashJudgeVerdict {
+    let newest_persisted_root_key =
+        newest_persisted_root(&stream.writes, &image.persisted[..stream.writes.len()])
+            .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
+    let mut read_back = observed_read_back_after_a_crash(consulted);
+    if controls.crash_judge_control_pending && batch == 1 {
+        if let ObservedReadBack::FileRead { content, .. } = &mut read_back {
+            if let Some(last_byte) = content.last_mut() {
+                *last_byte ^= 1;
+                controls.crash_judge_control_pending = false;
+            }
+        }
+    }
+    crash_judge_verdict(
+        &stream.committed_versions,
+        newest_persisted_root_key,
+        &read_back,
+    )
+}
+
+/// 模型的 `crash_recovery_disagreement` 判一态读回的版本：有不同记红（带实现那一侧说的话），没有不同记绿。
+fn crash_judge_verdict(
+    committed_versions: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
+    newest_persisted_root_key: Option<ModelRootKey>,
+    read_back: &ObservedReadBack,
+) -> CrashJudgeVerdict {
+    match crash_recovery_disagreement(committed_versions, newest_persisted_root_key, read_back) {
+        Some(disagreement) => CrashJudgeVerdict::Red {
+            what: disagreement
+                .implementation_answer
+                .split_whitespace()
+                .collect::<Vec<&str>>()
+                .join("_"),
+        },
+        None => CrashJudgeVerdict::Green,
+    }
+}
+
+/// 一态上 O1-崩溃判官适用不适用（读法 6）：这一态里有写落在模型停下的那次调用（含）之后的调用里，模型的版本目录里没有那些调用写出的版本，不适用。
+fn crash_judge_is_applicable(
+    touched_call: Option<usize>,
+    model_stopped_at_call: Option<usize>,
+) -> bool {
+    !touched_call.is_some_and(|call| model_stopped_at_call.is_some_and(|stopped| call >= stopped))
+}
+
+/// 按序号升序把取样的状态切成连续的几段，每段调一次 `judge_layer0_state_range`：回调里给的那个 `CrashImage` 折回落地向量、算摘要，
+/// 再算 O1-崩溃判官与 O2（直接调 `check_pool_image`），并单独计恢复的耗时。回调的持久集合另与 `crash_facts` 按序号现算的掩码比。
+/// 某个状态上实现 panic（实现自崩）：交回 `Err`，带最后一个判完的序号。
+fn observe_sampled_states(
+    stream: &PreparedStream,
+    tables: &Layer0EnumerationTables,
+    facts: &FlowFacts,
+    torn_image_index_by_write: &BTreeMap<usize, usize>,
+    sampled: &SampledOrdinals,
+    switches: RunSwitches,
+) -> Result<Vec<StateObservation>, String> {
+    let mut batch_of_ordinal: BTreeMap<u64, u8> = BTreeMap::new();
+    for (batch_offset, batch) in sampled.batches.iter().enumerate() {
+        for ordinal in batch {
+            let batch_number = u8::try_from(batch_offset + 1).expect("三批");
+            assert!(
+                batch_of_ordinal.insert(*ordinal, batch_number).is_none(),
+                "三批互不重叠"
+            );
+        }
+    }
+    let ordinals: Vec<u64> = batch_of_ordinal.keys().copied().collect();
+    let mut runs: Vec<std::ops::Range<u64>> = Vec::new();
+    for ordinal in ordinals {
+        match runs.last_mut() {
+            Some(run) if run.end == ordinal => run.end += 1,
+            Some(_) | None => runs.push(ordinal..ordinal + 1),
+        }
+    }
+    let recorded_write_count = stream.writes.len();
+    let call_of_write = call_index_of_each_write(stream);
+    let model_is_enabled =
+        switches.disabled_oracle != Some(DisabledOracle::ComparisonAndCrashJudge);
+    let pool_checker_is_enabled = switches.disabled_oracle != Some(DisabledOracle::PoolChecker);
+    let mut controls = ObservationControls {
+        switches,
+        crash_judge_control_pending: switches.positive_control == Some(PositiveControl::CrashJudge),
+    };
+    let mut observations: Vec<StateObservation> = Vec::new();
+    for run in runs {
+        let judged = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+            judge_layer0_state_range(
+                &stream.base,
+                &stream.writes,
+                &stream.segments,
+                stream.judged_root_index,
+                &stream.versions,
+                &full_expansion,
+                run,
+                &mut |ordinal, image, judgement| {
+                    let batch = batch_of_ordinal[&ordinal];
+                    let landing_vector = landing_vector_of(
+                        &image.persisted,
+                        recorded_write_count,
+                        torn_image_index_by_write,
+                    );
+                    let touched_call = landing_vector
+                        .chars()
+                        .zip(&call_of_write)
+                        .filter(|(landing, _)| *landing != '0')
+                        .filter_map(|(_, call)| *call)
+                        .max();
+                    let verdict = ImplementationVerdict::of(judgement);
+                    let started = Instant::now();
+                    let crash_judge = if !model_is_enabled {
+                        CrashJudgeVerdict::Off
+                    } else if !crash_judge_is_applicable(touched_call, stream.model_stopped_at_call)
+                    {
+                        CrashJudgeVerdict::NotApplicable
+                    } else {
+                        crash_judge_of_the_state(
+                            stream,
+                            image,
+                            &judgement.consulted,
+                            batch,
+                            &mut controls,
+                        )
+                    };
+                    let o1_nanoseconds = started.elapsed().as_nanos();
+                    let started = Instant::now();
+                    let pool_checker_red = pool_checker_is_enabled.then(|| {
+                        let direct = if controls.switches.positive_control
+                            == Some(PositiveControl::PoolChecker)
+                            && stream.name == StreamName::NewPoolFileCreation
+                            && ordinal == tables.state_count - 1
+                        {
+                            pool_checker_on_the_first_known_bad_image(image)
+                        } else {
+                            check_pool_image(image)
+                        };
+                        violated_invariant_names(&direct)
+                    });
+                    let o2_nanoseconds = started.elapsed().as_nanos();
+                    let pool_checker_matches_the_judgement = pool_checker_red
+                        .as_ref()
+                        .is_none_or(|direct| *direct == verdict.pool_checker_red)
+                        || controls.switches.positive_control == Some(PositiveControl::PoolChecker);
+                    let started = Instant::now();
+                    let _ = singlefs_core::recovery::recover(
+                        image,
+                        singlefs_core::recovery::JournalPolicy::Consult,
+                    );
+                    let recovery_nanoseconds = started.elapsed().as_nanos();
+                    observations.push(StateObservation {
+                        batch,
+                        ordinal,
+                        segment_index: tables
+                            .state_ranges_by_segment
+                            .partition_point(|state_range| state_range.end <= ordinal),
+                        landing_vector,
+                        image_digest: sha256_text(&crash_image_digest(image)),
+                        verdict,
+                        judge_nanoseconds: o1_nanoseconds + o2_nanoseconds + recovery_nanoseconds,
+                        mask_matches_the_facts: facts.persisted_writes_of_state(ordinal)
+                            == image.persisted,
+                        touched_call,
+                        crash_judge,
+                        pool_checker_red,
+                        pool_checker_matches_the_judgement,
+                        consulted: RecoverySummary::of(&judgement.consulted),
+                        ignored: RecoverySummary::of(&judgement.ignored),
+                        o1_nanoseconds,
+                        o2_nanoseconds,
+                        recovery_nanoseconds,
+                    });
+                },
+            )
+        }));
+        match judged {
+            Ok(state_count_of_the_stream) => assert_eq!(
+                state_count_of_the_stream, tables.state_count,
+                "judge_layer0_state_range 交回的整条流状态数与枚举表的相同"
+            ),
+            Err(payload) => {
+                return Err(format!(
+                    "{}:after_ordinal_{}",
+                    panic_text(&*payload),
+                    observations
+                        .last()
+                        .map_or_else(|| "none".to_string(), |last| last.ordinal.to_string())
+                ));
+            }
+        }
+    }
+    observations.sort_by_key(|observation| (observation.batch, observation.ordinal));
+    Ok(observations)
+}
+
+// ───────────────────────────── 一条流导出（登记 5.3 的十个文件，照抄 E170 的写出） ─────────────────────────────
+
+/// 写出的一个文件：名字、字节数、sha256；带耗时的两个文件另算一份去掉耗时之后的 sha256（登记第十二节修订 V2b）。
+struct WrittenFile {
+    name: &'static str,
+    bytes: usize,
+    sha256: String,
+    sha256_without_timing: Option<String>,
+}
+
+/// `impl-verdicts.tsv` 每行去掉最后一列（耗时）；`manifest.tsv` 去掉记 `impl-verdicts.tsv` 摘要的那一行。其余文件原样。
+fn contents_without_timing(file_name: &str, contents: &[u8]) -> Option<Vec<u8>> {
+    let text = std::str::from_utf8(contents).ok()?;
+    let kept: Vec<String> = match file_name {
+        IMPLEMENTATION_VERDICTS_FILE => text
+            .lines()
+            .map(|line| {
+                line.rsplit_once('\t')
+                    .map_or(line, |(without_timing, _)| without_timing)
+                    .to_string()
+            })
+            .collect(),
+        MANIFEST_FILE => text
+            .lines()
+            .filter(|line| !line.starts_with(&format!("sha256:{IMPLEMENTATION_VERDICTS_FILE}\t")))
+            .map(str::to_string)
+            .collect(),
+        _ => return None,
+    };
+    Some(
+        kept.iter()
+            .flat_map(|line| format!("{line}\n").into_bytes())
+            .collect(),
+    )
+}
+
+fn write_export_file(directory: &Path, name: &'static str, contents: &[u8]) -> WrittenFile {
+    std::fs::write(directory.join(name), contents)
+        .unwrap_or_else(|error| panic!("写 {}：{error}", directory.join(name).display()));
+    WrittenFile {
+        name,
+        bytes: contents.len(),
+        sha256: sha256_hexadecimal(contents),
+        sha256_without_timing: contents_without_timing(name, contents)
+            .map(|kept| sha256_hexadecimal(&kept)),
+    }
+}
+
+/// `base-sectors.bin` 的内容（登记 5.3）：基镜像里每个不全为零的扇区，盘身份 u32、扇区号 u64、512 字节。
+/// 第一段写导出、第三段核导出与重驱动的流相同（`bad-states` 模式）共用这一份。
+fn base_sectors_file_bytes(stream: &PreparedStream) -> Vec<u8> {
+    let mut base_sectors = Vec::new();
+    for (device, sparse) in &stream.base.devices {
+        for (sector, bytes) in sparse.written_sectors() {
+            if bytes.iter().all(|byte| *byte == 0) {
+                continue;
+            }
+            base_sectors.extend_from_slice(&device.0.to_le_bytes());
+            base_sectors.extend_from_slice(&sector.to_le_bytes());
+            base_sectors.extend_from_slice(bytes);
+        }
+    }
+    base_sectors
+}
+
+/// `ops.tsv` 的文本与 `ops-payload.bin` 的字节（登记 5.3），同上两种模式共用。
+fn operations_file_text_and_payload(stream: &PreparedStream) -> (String, Vec<u8>) {
+    let mut operations_text = String::new();
+    let mut payload = Vec::new();
+    for (stream_index, retained) in stream.operations.iter().enumerate() {
+        let operation = &retained.operation;
+        let (payload_start, content_digest) = match operation.kind {
+            RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
+                let contents = retained
+                    .contents
+                    .as_ref()
+                    .expect("流开了内容保留，普通写与 FUA 写带字节");
+                let start = payload.len();
+                payload.extend_from_slice(contents);
+                (start.to_string(), sha256_hexadecimal(contents))
+            }
+            RecordedOperationKind::WriteZeroes | RecordedOperationKind::Barrier => {
+                ("-".to_string(), "-".to_string())
+            }
+        };
+        operations_text.push_str(&format!(
+            "{stream_index}\t{}\t{}\t{}\t{}\t{payload_start}\t{content_digest}\n",
+            operation.device.0,
+            operation_kind_name(operation.kind),
+            operation.offset.0,
+            operation.length,
+        ));
+    }
+    (operations_text, payload)
+}
+
+/// `calls.tsv` 的文本（E170 登记 5.3）：调用序号、名字、发出的第一条 op 的流序下标、返回点、`ok` / `err`。
+fn calls_file_text(stream: &PreparedStream) -> String {
+    stream
+        .calls
+        .iter()
+        .enumerate()
+        .map(|(call_index, call)| {
+            format!(
+                "{call_index}\t{}\t{}\t{}\t{}\n",
+                call.label,
+                call.first_operation,
+                call.return_point,
+                call.outcome.calls_file_word()
+            )
+        })
+        .collect()
+}
+
+/// tsv 里的自由文本：制表符与换行换成空格。
+fn cell_text(text: &str) -> String {
+    text.replace(['\t', '\n', '\r'], " ")
+}
+
+/// `call-outcomes.tsv`：每次调用一行——序号、名字、第一条 op、返回点、结局（`ok` / `err:<成员>` / `panic`）、O1-对拍的判定、
+/// 对不上时模型与实现各说了什么。不含耗时（两次基线逐字节相同，作废条款 Z1）。
+fn call_outcomes_file_text(stream: &PreparedStream) -> String {
+    stream
+        .calls
+        .iter()
+        .enumerate()
+        .map(|(call_index, call)| {
+            let (model_answer, implementation_answer) = match &call.model {
+                ModelVerdictOfCall::Disagrees {
+                    model_answer,
+                    implementation_answer,
+                    ..
+                } => (cell_text(model_answer), cell_text(implementation_answer)),
+                ModelVerdictOfCall::NotModelled
+                | ModelVerdictOfCall::Agrees
+                | ModelVerdictOfCall::SkippedAfterTheModelStopped => {
+                    ("-".to_string(), "-".to_string())
+                }
+            };
+            let panic_message = match &call.outcome {
+                CallOutcome::Panicked(text) => cell_text(text),
+                CallOutcome::Succeeded | CallOutcome::ReturnedError(_) => "-".to_string(),
+            };
+            format!(
+                "{call_index}\t{}\t{}\t{}\t{}\t{}\t{model_answer}\t{implementation_answer}\t{panic_message}\n",
+                call.label,
+                call.first_operation,
+                call.return_point,
+                call.outcome.outcome_word(),
+                call.model.word()
+            )
+        })
+        .collect()
+}
+
+/// `stream-status.tsv`：驱动怎么结束的、模型停在哪次调用、被判的根写在不在（`key<TAB>value`）。
+fn stream_status_file_text(stream: &PreparedStream) -> String {
+    let ending = match &stream.ending {
+        StreamEnding::Completed => "completed".to_string(),
+        StreamEnding::StoppedByAnError { call_index, member } => {
+            format!("stopped_by_an_error:call_{call_index}:{member}")
+        }
+        StreamEnding::ImplementationCrashed { what } => {
+            format!(
+                "implementation_crashed:{}",
+                cell_text(what).replace(' ', "_")
+            )
+        }
+    };
+    format!(
+        "ending\t{ending}\nmodel_stopped_at_call\t{}\njudged_root_index\t{}\nwrites\t{}\ncalls\t{}\n",
+        stream
+            .model_stopped_at_call
+            .map_or_else(|| "none".to_string(), |call| call.to_string()),
+        if stream.judged_root_index == usize::MAX {
+            "none".to_string()
+        } else {
+            stream.judged_root_index.to_string()
+        },
+        stream.writes.len(),
+        stream.calls.len()
+    )
+}
+
+/// `oracle-verdicts.tsv`：每个取样状态一行（次序与 `states.tsv` 相同）——状态号、批号、层 0 序号、段号、触达的最后一次调用、
+/// O1-崩溃判官（`green` / `red` / `na` / `off`）与红的内容、O2（同）与判红的不变量、O2 直接调与 `judge_crash_image` 内那一遍是否一致、
+/// 看 journal 那一遍恢复的摘要（根、结局、读回 sha256、施加记录数）、不看 journal 那一遍的摘要、层 0 的 `red_items`（附带：记录核对器与手写版本表的 oracle）。
+fn oracle_verdicts_file_text(observations: &[StateObservation]) -> String {
+    observations
+        .iter()
+        .enumerate()
+        .map(|(state_number, observation)| {
+            let (o2_word, o2_items) = match &observation.pool_checker_red {
+                None => ("off", "-".to_string()),
+                Some(items) if items.is_empty() => ("green", "-".to_string()),
+                Some(items) => ("red", items.join(",")),
+            };
+            let o1_detail = match &observation.crash_judge {
+                CrashJudgeVerdict::Red { what } => cell_text(what),
+                CrashJudgeVerdict::Green
+                | CrashJudgeVerdict::NotApplicable
+                | CrashJudgeVerdict::Off => "-".to_string(),
+            };
+            format!(
+                "{state_number}\t{}\t{}\t{}\t{}\t{}\t{o1_detail}\t{o2_word}\t{o2_items}\t{}\t{}\t{}\t{}\n",
+                observation.batch,
+                observation.ordinal,
+                observation.segment_index,
+                observation
+                    .touched_call
+                    .map_or_else(|| "-".to_string(), |call| call.to_string()),
+                observation.crash_judge.word(),
+                observation.pool_checker_matches_the_judgement,
+                observation.consulted.text(),
+                observation.ignored.text(),
+                field_text(&list_text(&observation.verdict.red_items)),
+            )
+        })
+        .collect()
+}
+
+const ORACLE_VERDICTS_FILE: &str = "oracle-verdicts.tsv";
+const CALL_OUTCOMES_FILE: &str = "call-outcomes.tsv";
+const STREAM_STATUS_FILE: &str = "stream-status.tsv";
+
+/// 按 E170 登记 5.3 写一条流的十个文件，manifest 最后写（它记前九个的 sha256）；之后再写本实验自己的三个：
+/// `oracle-verdicts.tsv`、`call-outcomes.tsv`、`stream-status.tsv`（不进 manifest：manifest 与 E170 导出装置写的同一份逐字节可比，停机 S2）。
+fn write_stream_files(
+    directory: &Path,
+    stream: &PreparedStream,
+    tables: &Layer0EnumerationTables,
+    observations: &[StateObservation],
+    full_states: u64,
+    quick_tier_states: u64,
+) -> Vec<WrittenFile> {
+    let base_sectors = base_sectors_file_bytes(stream);
+    let (operations_text, payload) = operations_file_text_and_payload(stream);
+    let calls_text = calls_file_text(stream);
+    let mut torn_text = String::new();
+    let mut torn_payload = Vec::new();
+    let mut torn_rows: Vec<(usize, usize)> = tables
+        .torn_images
+        .iter()
+        .map(|row| (row.write_index, row.torn_image_index))
+        .collect();
+    torn_rows.sort_unstable();
+    for (write_index, torn_image_index) in torn_rows {
+        let torn_bytes = tables.enumerated_writes[torn_image_index]
+            .bytes()
+            .expect("撕裂镜像是带字节的写");
+        torn_text.push_str(&format!(
+            "{write_index}\t{}\t{}\t{}\n",
+            torn_payload.len(),
+            torn_bytes.len(),
+            sha256_hexadecimal(torn_bytes)
+        ));
+        torn_payload.extend_from_slice(torn_bytes);
+    }
+    let mut states_text = String::new();
+    let mut verdicts_text = String::new();
+    for (state_number, observation) in observations.iter().enumerate() {
+        states_text.push_str(&format!(
+            "{state_number}\t{}\t{}\t{}\t{}\t{}\n",
+            observation.ordinal,
+            observation.segment_index,
+            observation.batch,
+            observation.landing_vector,
+            observation.image_digest
+        ));
+        let verdict = &observation.verdict;
+        verdicts_text.push_str(&format!(
+            "{state_number}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
+            verdict.consulted_root,
+            verdict.consulted_outcome,
+            verdict.ignored_root,
+            verdict.ignored_outcome,
+            list_text(&verdict.red_items),
+            list_text(&verdict.pool_checker_red),
+            verdict.root_without_record,
+            verdict.claimed_state_missing_unit,
+            observation.judge_nanoseconds
+        ));
+    }
+    let quick_tier_text =
+        format!("quick_tier_states\t{quick_tier_states}\nfull_states\t{full_states}\n");
+    let mut files = vec![
+        write_export_file(directory, BASE_SECTORS_FILE, &base_sectors),
+        write_export_file(directory, OPERATIONS_FILE, operations_text.as_bytes()),
+        write_export_file(directory, OPERATIONS_PAYLOAD_FILE, &payload),
+        write_export_file(directory, CALLS_FILE, calls_text.as_bytes()),
+        write_export_file(directory, TORN_FILE, torn_text.as_bytes()),
+        write_export_file(directory, TORN_PAYLOAD_FILE, &torn_payload),
+        write_export_file(directory, STATES_FILE, states_text.as_bytes()),
+        write_export_file(
+            directory,
+            IMPLEMENTATION_VERDICTS_FILE,
+            verdicts_text.as_bytes(),
+        ),
+        write_export_file(directory, QUICK_TIER_FILE, quick_tier_text.as_bytes()),
+    ];
+    let device_list: Vec<String> = stream
+        .base
+        .devices
+        .keys()
+        .map(|device| device.0.to_string())
+        .collect();
+    let mut manifest_text = format!(
+        "stream\t{}\ndevices\t{}\ndevice_bytes\t{}\nsector_bytes\t{LOCAL_SECTOR_BYTES}\nops\t{}\nwrites\t{}\ncalls\t{}\nsampled_states\t{}\n",
+        stream.name.directory_name(),
+        device_list.join(","),
+        stream.base.device_size_in_bytes,
+        stream.operations.len(),
+        stream.writes.len(),
+        stream.calls.len(),
+        observations.len(),
+    );
+    for (listed_name, file) in FILES_LISTED_IN_THE_MANIFEST.iter().zip(&files) {
+        assert_eq!(*listed_name, file.name, "manifest 的次序与写出的次序相同");
+        manifest_text.push_str(&format!("sha256:{}\t{}\n", file.name, file.sha256));
+    }
+    files.push(write_export_file(
+        directory,
+        MANIFEST_FILE,
+        manifest_text.as_bytes(),
+    ));
+    files.push(write_export_file(
+        directory,
+        ORACLE_VERDICTS_FILE,
+        oracle_verdicts_file_text(observations).as_bytes(),
+    ));
+    files
+        .extend(write_the_files_that_exist_even_when_the_stream_is_not_exported(directory, stream));
+    files
+}
+
+/// 驱动里实现自崩、或没有根槽写可判的流不导出状态；这两个文件照写（比对器靠它们认出「实现自崩」与「出错截断」）。
+fn write_the_files_that_exist_even_when_the_stream_is_not_exported(
+    directory: &Path,
+    stream: &PreparedStream,
+) -> Vec<WrittenFile> {
+    vec![
+        write_export_file(
+            directory,
+            CALL_OUTCOMES_FILE,
+            call_outcomes_file_text(stream).as_bytes(),
+        ),
+        write_export_file(
+            directory,
+            STREAM_STATUS_FILE,
+            stream_status_file_text(stream).as_bytes(),
+        ),
+    ]
+}
+
+/// 一条流导出之后的样子。`not_exported` 有值：这条流没有导出状态（驱动里实现 panic、没有根槽写可判、某个状态上实现 panic、
+/// 或流水线里别处 panic），只有 `call-outcomes.tsv` 与 `stream-status.tsv`（能写出来的话）。
+struct StreamExport {
+    name: StreamName,
+    checks: Vec<StopCheck>,
+    calls: Vec<RecordedCall>,
+    ending: Option<StreamEnding>,
+    model_stopped_at_call: Option<usize>,
+    not_exported: Option<String>,
+    operation_count: usize,
+    write_count: usize,
+    segment_count: usize,
+    tearable_write_count: usize,
+    full_states: u64,
+    quick_tier_states: u64,
+    torn_scans: Vec<TornScan>,
+    observations: Vec<StateObservation>,
+    files: Vec<WrittenFile>,
+    rebuilt_digest_mismatches: usize,
+    rebuilt_states: usize,
+    elapsed_nanoseconds: u128,
+    /// 驱动这条流（含每次调用与推模型、O1-对拍）、判一批状态、写出导出文件各花多久（装置进程里 `Instant` 计）。
+    drive_nanoseconds: u128,
+    observe_nanoseconds: u128,
+    write_nanoseconds: u128,
+}
+
+impl StreamExport {
+    /// 什么都没导出的一条流：流水线在驱动之后、导出之前 panic 时用它占位。
+    fn not_exported(name: StreamName, why: String, started: Instant) -> Self {
+        Self {
+            name,
+            checks: Vec::new(),
+            calls: Vec::new(),
+            ending: None,
+            model_stopped_at_call: None,
+            not_exported: Some(why),
+            operation_count: 0,
+            write_count: 0,
+            segment_count: 0,
+            tearable_write_count: 0,
+            full_states: 0,
+            quick_tier_states: 0,
+            torn_scans: Vec::new(),
+            observations: Vec::new(),
+            files: Vec::new(),
+            rebuilt_digest_mismatches: 0,
+            rebuilt_states: 0,
+            elapsed_nanoseconds: started.elapsed().as_nanos(),
+            drive_nanoseconds: 0,
+            observe_nanoseconds: 0,
+            write_nanoseconds: 0,
+        }
+    }
+}
+
+/// 基线与坏实现那一趟的差别：基线回比本地常量、取三批；坏实现那一趟不回比（写流与段表本来就会变）、只取批 1 与批 2。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum RunKind {
+    Baseline,
+    BadImplementationRow,
+}
+
+fn export_stream(
+    name: StreamName,
+    output_root: &Path,
+    run_kind: RunKind,
+    switches: RunSwitches,
+) -> StreamExport {
+    let started = Instant::now();
+    let is_baseline = run_kind == RunKind::Baseline;
+    let stream = prepare_stream(name, switches, is_baseline);
+    let drive_nanoseconds = started.elapsed().as_nanos();
+    let label = name.directory_name();
+    let directory = output_root.join(label);
+    std::fs::create_dir_all(&directory)
+        .unwrap_or_else(|error| panic!("建 {}：{error}", directory.display()));
+    // 回比只在基线那一趟：坏实现那一趟本来就会让取号、txg、落点、写数与段表变，那些不是停机条款。
+    let mut checks = if is_baseline {
+        stream.checks.clone()
+    } else {
+        Vec::new()
+    };
+    let mut export = StreamExport::not_exported(name, String::new(), started);
+    export.not_exported = None;
+    export.calls = stream.calls.clone();
+    export.ending = Some(stream.ending.clone());
+    export.model_stopped_at_call = stream.model_stopped_at_call;
+    export.operation_count = stream.operations.len();
+    export.write_count = stream.writes.len();
+    export.segment_count = stream.segments.len();
+    export.drive_nanoseconds = drive_nanoseconds;
+    let unexported_reason = match &stream.ending {
+        StreamEnding::ImplementationCrashed { what } => Some(format!(
+            "implementation_crashed_in_a_call:{}",
+            what.replace(' ', "_")
+        )),
+        StreamEnding::Completed | StreamEnding::StoppedByAnError { .. }
+            if stream.judged_root_index == usize::MAX =>
+        {
+            Some("no_root_write_to_judge".to_string())
+        }
+        StreamEnding::Completed | StreamEnding::StoppedByAnError { .. } => None,
+    };
+    if let Some(reason) = unexported_reason {
+        export.files =
+            write_the_files_that_exist_even_when_the_stream_is_not_exported(&directory, &stream);
+        export.checks = checks;
+        export.not_exported = Some(reason);
+        export.elapsed_nanoseconds = started.elapsed().as_nanos();
+        return export;
+    }
+    let tables = layer0_enumeration_tables(
+        &stream.base,
+        &stream.writes,
+        &stream.segments,
+        &full_expansion,
+    );
+    let full_states = layer0_plan_state_count(
+        &stream.base,
+        &stream.writes,
+        &stream.segments,
+        &full_expansion,
+    );
+    let quick_tier_states = layer0_plan_state_count(
+        &stream.base,
+        &stream.writes,
+        &stream.segments,
+        &quick_tier_expansion,
+    );
+    let facts = extract_flow_facts(
+        &stream.base,
+        &stream.writes,
+        &stream.segments,
+        &full_expansion,
+    );
+    if is_baseline {
+        checks.extend(state_count_checks_against_the_crates(
+            name,
+            &stream,
+            &tables,
+            &facts,
+            full_states,
+            quick_tier_states,
+        ));
+    }
+    let torn_image_index_by_write: BTreeMap<usize, usize> = tables
+        .torn_images
+        .iter()
+        .map(|row| (row.write_index, row.torn_image_index))
+        .collect();
+    let segment_has_a_torn_write: Vec<bool> = stream
+        .segments
+        .iter()
+        .map(|segment| {
+            segment
+                .iter()
+                .any(|write_index| tables.is_tearable_by_write[*write_index])
+        })
+        .collect();
+    let recorded_write_count = stream.writes.len();
+    let mut sampled = sample_ordinals(
+        &tables.state_ranges_by_segment,
+        tables.state_count,
+        &segment_has_a_torn_write,
+        &|ordinal| {
+            landing_vector_of(
+                &facts.persisted_writes_of_state(ordinal),
+                recorded_write_count,
+                &torn_image_index_by_write,
+            )
+        },
+    );
+    if !is_baseline {
+        sampled.batches[2].clear();
+    }
+    let observe_started = Instant::now();
+    let observations = match observe_sampled_states(
+        &stream,
+        &tables,
+        &facts,
+        &torn_image_index_by_write,
+        &sampled,
+        switches,
+    ) {
+        Ok(observations) => observations,
+        Err(what) => {
+            export.files = write_the_files_that_exist_even_when_the_stream_is_not_exported(
+                &directory, &stream,
+            );
+            export.checks = checks;
+            export.not_exported = Some(format!("implementation_crashed_in_a_state:{what}"));
+            export.elapsed_nanoseconds = started.elapsed().as_nanos();
+            return export;
+        }
+    };
+    let observe_nanoseconds = observe_started.elapsed().as_nanos();
+    let write_started = Instant::now();
+    let files = write_stream_files(
+        &directory,
+        &stream,
+        &tables,
+        &observations,
+        full_states,
+        quick_tier_states,
+    );
+    // S4：只读刚写出的文件，按 states.tsv 的落地向量重建，摘要与 states.tsv 里（直接从 CrashImage 算的）那一列比。
+    let neutral = read_neutral_stream(&directory);
+    let mut rebuilt_states = 0usize;
+    let mut rebuilt_digest_mismatches = 0usize;
+    for line in read_text_lines(&directory, STATES_FILE) {
+        let fields: Vec<&str> = line.split('\t').collect();
+        assert_eq!(fields.len(), 6, "states.tsv 一行六列：{line:?}");
+        rebuilt_states += 1;
+        if sha256_text(&rebuilt_image_digest(&neutral, fields[4])) != fields[5] {
+            rebuilt_digest_mismatches += 1;
+        }
+    }
+    export.write_nanoseconds = write_started.elapsed().as_nanos();
+    export.observe_nanoseconds = observe_nanoseconds;
+    export.checks = checks;
+    export.tearable_write_count = torn_image_index_by_write.len();
+    export.full_states = full_states;
+    export.quick_tier_states = quick_tier_states;
+    export.torn_scans = sampled.torn_scans;
+    export.observations = observations;
+    export.files = files;
+    export.rebuilt_digest_mismatches = rebuilt_digest_mismatches;
+    export.rebuilt_states = rebuilt_states;
+    export.elapsed_nanoseconds = started.elapsed().as_nanos();
+    export
+}
+
+/// S1 里跟状态数有关的几条（只在基线那一趟）：全量与快档状态数、每次写只取两态时的闭式、枚举表与 `crash_facts` 数的总数，
+/// 对本地常量。与 `e170_o3_spec_executor_export.rs` 第 1298–1334 行同一批。
+fn state_count_checks_against_the_crates(
+    name: StreamName,
+    stream: &PreparedStream,
+    tables: &Layer0EnumerationTables,
+    facts: &FlowFacts,
+    full_states: u64,
+    quick_tier_states: u64,
+) -> Vec<StopCheck> {
+    let label = name.directory_name();
+    let (local_full, local_quick, local_closed_form) = match name {
+        StreamName::NewPoolFileCreation => (
+            LOCAL_FIRST_STREAM_FULL_STATES,
+            LOCAL_FIRST_STREAM_QUICK_TIER_STATES,
+            LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM,
+        ),
+        StreamName::FixedScriptThroughEThenUnmount => (
+            LOCAL_SECOND_STREAM_FULL_STATES,
+            LOCAL_SECOND_STREAM_QUICK_TIER_STATES,
+            LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM,
+        ),
+    };
+    vec![
+        StopCheck::new(&format!("{label}_full_states"), local_full, full_states),
+        StopCheck::new(
+            &format!("{label}_quick_tier_states"),
+            local_quick,
+            quick_tier_states,
+        ),
+        StopCheck::new(
+            &format!("{label}_two_state_closed_form"),
+            local_closed_form,
+            singlefs_harness::memory_pool::closed_form_state_count(&stream.segments),
+        ),
+        StopCheck::new(
+            &format!("{label}_enumeration_tables_state_count"),
+            full_states,
+            tables.state_count,
+        ),
+        StopCheck::new(
+            &format!("{label}_flow_facts_state_count"),
+            full_states,
+            facts.state_count,
+        ),
+    ]
+}
+
+// ───────────────────────────── 结果行、V2 与 main ─────────────────────────────
+
+/// `E7RESULT` 行计数（收尾 `name=done emitted=N`，N 把 done 自己也算进去）；`E7TIMING` 行不计数、不进入库产物。
+struct ResultLines {
+    emitted: u64,
+}
+
+impl ResultLines {
+    fn result(&mut self, body: &str) {
+        self.emitted += 1;
+        println!("E7RESULT {body}");
+    }
+    fn timing(body: &str) {
+        println!("E7TIMING {body}");
+    }
+    fn finish(&mut self) {
+        self.emitted += 1;
+        println!("E7RESULT name=done emitted={}", self.emitted);
+    }
+}
+
+/// 结果行里的值不许带空白（按空格切字段）。
+fn field_text(text: &str) -> String {
+    text.chars()
+        .filter(|character| !character.is_whitespace())
+        .collect()
+}
+
+/// S1 里与流无关的几条：本地常量对 `tests/common/mod.rs` 与 harness 的同名公开常量或现算值。
+fn stream_independent_checks() -> Vec<StopCheck> {
+    let parameters = common::parameters();
+    vec![
+        StopCheck::new("device_bytes", LOCAL_DEVICE_BYTES, common::IMAGE_BYTES),
+        StopCheck::new(
+            "physical_block_bytes",
+            LOCAL_PHYSICAL_BLOCK_BYTES,
+            parameters.geometry.physical_block_size,
+        ),
+        StopCheck::new(
+            "minimum_input_output_bytes",
+            LOCAL_MINIMUM_INPUT_OUTPUT_BYTES,
+            parameters.geometry.minimum_input_output_bytes,
+        ),
+        StopCheck::new(
+            "sector_bytes",
+            LOCAL_SECTOR_BYTES,
+            singlefs_harness::memory_pool::SECTOR_BYTES,
+        ),
+        StopCheck::new(
+            "filesystem_identifier",
+            LOCAL_FILESYSTEM_IDENTIFIER,
+            common::E142_FILESYSTEM_IDENTIFIER,
+        ),
+        StopCheck::new(
+            "fixed_write_time_seconds",
+            LOCAL_FIXED_WRITE_TIME_SECONDS,
+            common::FIXED_WRITE_TIME_SECONDS,
+        ),
+        StopCheck::new(
+            "first_file_bytes",
+            LOCAL_FIRST_FILE_BYTES,
+            common::FILE_BYTES,
+        ),
+        StopCheck::new(
+            "first_file_content",
+            true,
+            local_first_file_content() == common::file_content(),
+        ),
+    ]
+}
+
+/// 中位取排好序之后下标 ⌊(n − 1)/2⌋ 那一个，p95 取下标 ⌈0.95·n⌉ − 1 那一个。
+fn timing_summary(mut nanoseconds: Vec<u128>) -> (u128, u128, u128, u128) {
+    if nanoseconds.is_empty() {
+        return (0, 0, 0, 0);
+    }
+    nanoseconds.sort_unstable();
+    let count = nanoseconds.len();
+    let total: u128 = nanoseconds.iter().sum();
+    let p95_index = (count * 95).div_ceil(100) - 1;
+    (
+        total,
+        nanoseconds[(count - 1) / 2],
+        nanoseconds[p95_index],
+        nanoseconds[count - 1],
+    )
+}
+
+// ───────────────────────────── 已知坏镜像族（登记 5.5 第三族）：照抄 `checker_known_bad_images.rs` ─────────────────────────────
+//
+// 那份用例是 O2 自己的回归语料：每一份都照 O2 的判据造来让 O2 红。下面这个模块逐行照抄它的导入（第 7–68 行）、常量与辅助函数（第 70–745 行）、
+// `known_bad_images`（第 747–1294 行），再抄 `known_bad_images` 用到而散在第 1295 行之后的几个常量与辅助函数（行号在模块末尾的注释里）。
+// 文件 sha256 e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820；它的 `#[test]` 没有抄。照抄、不 `#[path]` 引：
+// 那个文件有自己的 `mod common;` 与全部测试，私有的 `known_bad_images` 也引不到。条目名与条数在 `known-bad-images` 模式里与原文件逐条比（停机 S4、断言 A8）。
+#[allow(
+    dead_code,
+    unused_imports,
+    clippy::all,
+    clippy::pedantic,
+    reason = "逐行照抄 checker_known_bad_images.rs 的辅助函数与坏镜像表：那份文件里别的族用的辅助函数在这里没人调，抄来的写法也不照本文件的 lint 改"
+)]
+mod known_bad_images_copied_from_the_harness_test {
+    use super::common::{
+        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
+        copy_each_root_into_the_new_root_segment_of_its_journal_records, crash_state_devices,
+        format_pool, geometry, memory_pool_of_sparse_devices, parameters,
+        publish_overwrite_in_process, rewrite_journal_record_and_rechain_the_records_after_it,
+        BuiltPool, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
+    };
+    use singlefs_checker::image::{
+        InvariantVerdict, ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
+        DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
+        INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
+        INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
+        JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
+        LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
+        NAMED_ENTRY_MATCHES_THE_UNIT_HEADER, POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
+        PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
+        ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES, ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
+        SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
+        SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
+        SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
+        SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
+        TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
+        UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
+    };
+    use singlefs_checker::walk::check_pool_image;
+    use singlefs_core::address::{
+        CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+    };
+    use singlefs_core::allocation_record_tree::AllocationRecordTreeNodePosition;
+    use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
+    use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
+    use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
+    use singlefs_core::journal::{
+        back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish,
+        JournalRecordPlaceInPublish,
+    };
+    use singlefs_core::make_filesystem::{
+        allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
+        INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
+    };
+    use singlefs_core::mount::{
+        mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, InstanceRow,
+        MountError, RollbackTarget, ShadowLedger, Unmounted,
+    };
+    use singlefs_core::recovery::{
+        instance_table_chain_of_root, recover, JournalPolicy, PoolReader, RecoveryFailure,
+        RecoveryOutcome,
+    };
+    use singlefs_core::system_configuration::SystemImmutableSizes;
+    use singlefs_core::transaction::{
+        acquire_instance, publish_first_file, publish_instance_table_on_version_without_file,
+        publish_overwrite, publish_sequential_write, publish_without_units, warm_up, FirstFile,
+        InstanceTableOnlyPublishPlan, InstanceTableRewrite, PoolVersion, PoolWriter,
+        TransactionOutput, TransactionUnit, ZeroUnitPublishPlan,
+    };
+    use singlefs_format::{
+        ALLOCATION_RECORD_TREE_LEAF_SLOTS, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES,
+        JOURNAL_SAFETY_FACTOR, NODE_BYTES, TREE_IDENTIFIER_EXTENT,
+    };
+    use singlefs_harness::history::{allocation_statistic_mechanism, AllocationStatisticMechanism};
+    use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+    use singlefs_harness::segments::StepKind;
+    use singlefs_harness::{RetainedOperation, SharedStream};
+
+    const SLOT: u64 = 16384;
+    const DEVICES: [u32; 2] = [0, 1];
+    /// 写完新池新建文件之后单元区里的单元：(起点槽, 字节数)。分配记录树按位置寻址（D8（核心索引结构） 已定项 14）：4 GiB 两块盘上根在第 2 层，
+    /// A 写出它的五个节点——两块盘各自的叶 61（50245、50246）、两块盘各自的第 1 层节点 0（50247、50248）与根（50249），先叶后根。
+    const UNITS: [(u64, usize); 14] = [
+        (50176, 32768),
+        (50178, 16384),
+        (50180, 32768),
+        (50240, 16384),
+        (50242, 32768),
+        (50244, 16384),
+        (50245, 16384),
+        (50246, 16384),
+        (50247, 16384),
+        (50248, 16384),
+        (50249, 16384),
+        (50250, 16384),
+        (50251, 16384),
+        (50252, 16384),
+    ];
+    /// 再覆盖写一次（发布 B，txg 4）之后单元区里的单元：A 的十四个槽仍在盘上（根环里 A 的根还引用着它们），加上 B 写出的十二个单元
+    /// （落点由 `overwrite_in_one_instance.rs` 的验收钉住）。
+    const UNITS_AFTER_OVERWRITE: [(u64, usize); 26] = [
+        (50176, 32768),
+        (50178, 16384),
+        (50180, 32768),
+        (50182, 32768),
+        (50240, 16384),
+        (50242, 32768),
+        (50244, 16384),
+        (50245, 16384),
+        (50246, 16384),
+        (50247, 16384),
+        (50248, 16384),
+        (50249, 16384),
+        (50250, 16384),
+        (50251, 16384),
+        (50252, 16384),
+        (50253, 16384),
+        (50254, 32768),
+        (50256, 16384),
+        (50257, 16384),
+        (50258, 16384),
+        (50259, 16384),
+        (50260, 16384),
+        (50261, 16384),
+        (50262, 16384),
+        (50263, 16384),
+        (50264, 16384),
+    ];
+    /// 发布 B 写出的分配记录树两片叶（盘 0 的叶 61、盘 1 的叶 61；I-3.9、I-3.10、I-5.4 那几份坏镜像改的就是它们）与树表（I-9.14 那一份改它）。
+    /// 每片叶只装它那块盘上的记录：一个单元两盘各一份落点，两条记录分在两片叶里。
+    const ALLOCATION_LEAVES_AFTER_OVERWRITE: [u64; 2] = [50257, 50258];
+    const TREE_TABLE_AFTER_OVERWRITE: u64 = 50264;
+    const INSTANCE_TABLE: u64 = 50176;
+    /// mkfs 种下的第 0 版树表单元：第一个文件版本换下它之后只有第 0 代根还引用。
+    const GENESIS_TREE_TABLE: u64 = 50178;
+    const DATA_UNIT: u64 = 50180;
+    const EXTENT_ROOT: u64 = 50240;
+    const INODE_LEAF: u64 = 50242;
+    const INODE_ROOT: u64 = 50244;
+    /// 新池新建文件（A）写出的分配记录树两片叶（盘 0、盘 1 各自的叶 61）：发布 B 之后只有 A 的根还（经 A 的第 1 层节点）指着它们。
+    const ALLOCATION_LEAVES: [u64; 2] = [50245, 50246];
+    const ACCOUNTING_ROOT: u64 = 50250;
+    const TREE_TABLE: u64 = 50252;
+
+    fn read(pool: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
+        PoolReader::read(
+            pool,
+            DeviceIdentity(device),
+            DeviceOffsetInBytes(offset),
+            length,
+        )
+        .expect("读")
+    }
+
+    fn write(pool: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
+        pool.devices
+            .get_mut(&DeviceIdentity(device))
+            .expect("盘")
+            .write(DeviceOffsetInBytes(offset), bytes);
+    }
+
+    /// 按类重封：先算载荷 CRC，再算头校验和（码 1：头 105、CRC 在 101；码 3：头 107、CRC 在 89；码 2：头 86 + 2k、CRC 在 76 + 2k）。
+    fn reseal_unit(bytes: &mut [u8]) {
+        let (header_end, payload_crc_offset) = match bytes[6] {
+            1 => (105usize, 101usize),
+            3 => (107, 89),
+            _ => (
+                86 + 2 * usize::from(bytes[51]),
+                76 + 2 * usize::from(bytes[51]),
+            ),
+        };
+        let payload_crc = crc32_castagnoli(&bytes[header_end..]);
+        bytes[payload_crc_offset..payload_crc_offset + 4]
+            .copy_from_slice(&payload_crc.to_le_bytes());
+        singlefs_core::unit::seal_header_checksum(bytes, header_end);
+    }
+
+    /// 根环全部槽：(盘, 偏移)。区域 r 在盘 [0, 1, 0][r] 上，起点 64 × 16384 + r × 3 MiB，槽距 4096。
+    fn root_slots() -> Vec<(u32, u64)> {
+        (0..3u64)
+            .flat_map(|region| {
+                (0..8u64).map(move |slot| {
+                    (
+                        [0u32, 1, 0][usize::try_from(region).expect("区域")],
+                        64 * SLOT + region * 3 * (1 << 20) + slot * 4096,
+                    )
+                })
+            })
+            .collect()
+    }
+
+    fn location_pattern(device: u32, slot: u64, checksum: u32) -> Vec<u8> {
+        [
+            device.to_le_bytes().to_vec(),
+            slot.to_le_bytes()[..6].to_vec(),
+            checksum.to_le_bytes().to_vec(),
+        ]
+        .concat()
+    }
+
+    fn replace_all(haystack: &mut [u8], needle: &[u8], replacement: &[u8]) -> bool {
+        let mut changed = false;
+        let mut index = 0;
+        while index + needle.len() <= haystack.len() {
+            if haystack[index..index + needle.len()] == *needle {
+                haystack[index..index + needle.len()].copy_from_slice(replacement);
+                changed = true;
+                index += needle.len();
+            } else {
+                index += 1;
+            }
+        }
+        changed
+    }
+
+    /// 一个单元（两盘同槽）的整单元校验和从 old 变成 new：把引用它的位置条目逐个补上，被补的单元重封、再往上补，直到根槽。
+    /// `units` 是这份镜像单元区里的单元表：写完新池新建文件的是 `UNITS`，再覆盖写一次的是 `UNITS_AFTER_OVERWRITE`。
+    fn propagate(pool: &mut MemoryPool, units: &[(u64, usize)], mut changes: Vec<(u64, u32, u32)>) {
+        while let Some((slot, old, new)) = changes.pop() {
+            if old == new {
+                continue;
+            }
+            for (container, length) in units.iter().copied() {
+                if container == slot {
+                    continue;
+                }
+                let mut bytes = read(pool, 0, container * SLOT, length);
+                let mut touched = false;
+                for device in DEVICES {
+                    touched |= replace_all(
+                        &mut bytes,
+                        &location_pattern(device, slot, old),
+                        &location_pattern(device, slot, new),
+                    );
+                }
+                if touched {
+                    let before = crc32_castagnoli(&read(pool, 0, container * SLOT, length));
+                    reseal_unit(&mut bytes);
+                    for device in DEVICES {
+                        write(pool, device, container * SLOT, &bytes);
+                    }
+                    changes.push((container, before, crc32_castagnoli(&bytes)));
+                }
+            }
+            for (device, offset) in root_slots() {
+                let mut bytes = read(pool, device, offset, 512);
+                let mut touched = false;
+                for location_device in DEVICES {
+                    touched |= replace_all(
+                        &mut bytes,
+                        &location_pattern(location_device, slot, old),
+                        &location_pattern(location_device, slot, new),
+                    );
+                }
+                if touched {
+                    let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
+                    bytes[138..170].copy_from_slice(&digest);
+                    write(pool, device, offset, &bytes);
+                }
+            }
+        }
+        copy_each_root_into_the_new_root_segment_of_its_journal_records(pool);
+    }
+
+    /// journal 记录头里 `checkpoint_txg`（8 字节）的偏移（D23（journal 的角色与格式） 已定项 4 / 已定项 17 的字段表）。
+    const JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET: usize = 26;
+
+    /// 只改一块盘上的那一份单元（另一块盘留着原样）：重封被改那一份的两道校验和，再把它的新整单元校验和补进引用它的
+    /// **那一块盘那一条**位置条目——位置条目逐盘各带一个整单元校验和（D4（校验和位置），指针 86 / 88 里位置条目 14 × 2），
+    /// 所以补完两份各自仍与自己那条位置条目对得上，I-2.1（校验和与内容匹配） 不跟着红；父单元两盘同内容，它自己的整单元校验和
+    /// 往上补那一段照旧走 `propagate`。
+    fn mutate_one_device_copy_of_unit(
+        pool: &mut MemoryPool,
+        units: &[(u64, usize)],
+        device: u32,
+        slot: u64,
+        change: impl Fn(&mut Vec<u8>),
+    ) {
+        let length = units
+            .iter()
+            .find(|(start, _)| *start == slot)
+            .expect("登记过的单元")
+            .1;
+        let before = read(pool, device, slot * SLOT, length);
+        let mut bytes = before.clone();
+        change(&mut bytes);
+        reseal_unit(&mut bytes);
+        write(pool, device, slot * SLOT, &bytes);
+        let (old, new) = (crc32_castagnoli(&before), crc32_castagnoli(&bytes));
+        let mut parents_changed = Vec::new();
+        for (container, container_length) in units.iter().copied() {
+            if container == slot {
+                continue;
+            }
+            let container_before = read(pool, 0, container * SLOT, container_length);
+            let mut container_bytes = container_before.clone();
+            if replace_all(
+                &mut container_bytes,
+                &location_pattern(device, slot, old),
+                &location_pattern(device, slot, new),
+            ) {
+                reseal_unit(&mut container_bytes);
+                for each_device in DEVICES {
+                    write(pool, each_device, container * SLOT, &container_bytes);
+                }
+                parents_changed.push((
+                    container,
+                    crc32_castagnoli(&container_before),
+                    crc32_castagnoli(&container_bytes),
+                ));
+            }
+        }
+        for (root_device, offset) in root_slots() {
+            let mut root_bytes = read(pool, root_device, offset, 512);
+            if replace_all(
+                &mut root_bytes,
+                &location_pattern(device, slot, old),
+                &location_pattern(device, slot, new),
+            ) {
+                let digest = wide_checksum_with_field_zeroed(&root_bytes, 512, 138);
+                root_bytes[138..170].copy_from_slice(&digest);
+                write(pool, root_device, offset, &root_bytes);
+            }
+        }
+        propagate(pool, units, parents_changed);
+    }
+
+    /// 改一个单元（两盘一起改）：`reseal` 为真时重封它的两道校验和；新的整单元校验和沿引用链补上去。
+    fn mutate_unit_of(
+        pool: &mut MemoryPool,
+        units: &[(u64, usize)],
+        slot: u64,
+        change: impl Fn(&mut Vec<u8>),
+        reseal: bool,
+    ) {
+        let length = units
+            .iter()
+            .find(|(start, _)| *start == slot)
+            .expect("登记过的单元")
+            .1;
+        let before = read(pool, 0, slot * SLOT, length);
+        let mut bytes = before.clone();
+        change(&mut bytes);
+        if reseal {
+            reseal_unit(&mut bytes);
+        }
+        for device in DEVICES {
+            write(pool, device, slot * SLOT, &bytes);
+        }
+        propagate(
+            pool,
+            units,
+            vec![(slot, crc32_castagnoli(&before), crc32_castagnoli(&bytes))],
+        );
+    }
+
+    /// 写完新池新建文件那份镜像上改一个单元。
+    fn mutate_unit(pool: &mut MemoryPool, slot: u64, change: impl Fn(&mut Vec<u8>), reseal: bool) {
+        mutate_unit_of(pool, &UNITS, slot, change, reseal);
+    }
+
+    /// 分配记录树的两片叶（两块盘各一片）逐片改、各自重封，新的整单元校验和沿引用链补上去（叶 → 那块盘的第 1 层节点 → 根 → 树表 → 根槽）；
+    /// `change` 返回它在那一片叶里改了几条记录，这里交回两片合起来的条数。
+    fn mutate_allocation_leaves(
+        pool: &mut MemoryPool,
+        units: &[(u64, usize)],
+        leaves: [u64; 2],
+        change: impl Fn(&mut Vec<u8>) -> usize,
+    ) -> usize {
+        let changed_records = std::cell::Cell::new(0usize);
+        for leaf in leaves {
+            mutate_unit_of(
+                pool,
+                units,
+                leaf,
+                |bytes| changed_records.set(changed_records.get() + change(bytes)),
+                true,
+            );
+        }
+        changed_records.get()
+    }
+
+    fn verdict(pool: &MemoryPool, invariant: &str) -> InvariantVerdict {
+        check_pool_image(pool)
+            .into_iter()
+            .find(|(identifier, _)| *identifier == invariant)
+            .expect("清单里有")
+            .1
+    }
+
+    /// journal 记录头里这三个字段的偏移（D23（journal 的角色与格式） 已定项 17 的字段表；记录头自证校验和在 46、罩整条 4096）。
+    const JOURNAL_RECORD_INSTANCE_OFFSET: usize = 16;
+    const JOURNAL_RECORD_TRANSACTION_OFFSET: usize = 78;
+
+    /// 这几份镜像上环里的记录条数上界：读到这个计数器还没读完就说明镜像变了，断言会响（别让列表被悄悄截断）。
+    const RECORDS_READ_BACK_AT_MOST: u64 = 64;
+
+    /// 一块盘的 journal 环里从计数器 1 起连着的记录，读成 (计数器, 实例代号, 事务号)；读到没有 magic 的那一格就停。
+    /// 计数器与环槽一一对应（`record_offset`），所以这里按计数器取偏移，不整条环扫。
+    fn journal_records_on_device(pool: &MemoryPool, device: u32) -> Vec<(u64, u32, u64)> {
+        let mut records = Vec::new();
+        for counter in 1..=RECORDS_READ_BACK_AT_MOST {
+            let offset = singlefs_core::journal::record_offset(
+                counter,
+                singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
+            )
+            .0;
+            let bytes = read(pool, device, offset, 4096);
+            if bytes[..4] != singlefs_core::journal::JOURNAL_MAGIC {
+                break;
+            }
+            records.push((
+                counter,
+                u32::from_le_bytes(
+                    bytes[JOURNAL_RECORD_INSTANCE_OFFSET..JOURNAL_RECORD_INSTANCE_OFFSET + 4]
+                        .try_into()
+                        .expect("4 字节"),
+                ),
+                get_u64(&bytes, JOURNAL_RECORD_TRANSACTION_OFFSET),
+            ));
+        }
+        assert!(
+            u64::try_from(records.len()).expect("条数") < RECORDS_READ_BACK_AT_MOST,
+            "环里的记录多到读满了上界 {RECORDS_READ_BACK_AT_MOST}：这几份镜像上不该有这么多"
+        );
+        records
+    }
+
+    /// 一条 journal 记录的事务号改成 `transaction`，头校验和重封（罩整条 4096，字段在 46）；两盘各一份都改——
+    /// 事务号重号是写者算错号，不是一块盘的字节坏了，两盘会写出同一条记录。
+    fn set_transaction_number_of_journal_record(
+        pool: &mut MemoryPool,
+        counter: u64,
+        expected_transaction: u64,
+        transaction: u64,
+    ) {
+        let offset = singlefs_core::journal::record_offset(
+            counter,
+            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
+        )
+        .0;
+        for device in DEVICES {
+            let mut record = read(pool, device, offset, 4096);
+            assert_eq!(
+                get_u64(&record, JOURNAL_RECORD_TRANSACTION_OFFSET),
+                expected_transaction,
+                "盘 {device} 计数器 {counter} 那条记录的事务号"
+            );
+            set_u64(&mut record, JOURNAL_RECORD_TRANSACTION_OFFSET, transaction);
+            let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
+            record[46..78].copy_from_slice(&digest);
+            write(pool, device, offset, &record);
+        }
+    }
+
+    /// 系统配置字段表（`.claude/kb/layout/01-first-txn.md` 一）：加密类型在整槽校验和（155，宽 32）、自身 MAC（16）、nonce 水位（12）、
+    /// KDF 标识（4）之后，偏移 219；第一版只读 0（不加密）。
+    const SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET: usize = 219;
+    /// 登记过的加密算法，第一版读者不读加密的卷：带它的系统配置槽自证过也整池拒绝挂载。
+    const ENCRYPTION_TYPE_THE_FIRST_VERSION_DOES_NOT_READ: u8 = 1;
+    /// I-7.13（系统配置池级字段在读者收的范围里） 那份坏镜像改的那一槽：盘 1 的槽 0。
+    const DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT: u32 = 1;
+    const OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT: u64 = 0;
+    /// 系统配置字段表（`.claude/kb/layout/01-first-txn.md` 一）几何段里几个 16 KiB 槽号与环长的偏移：journal 环起点 325、环长 333、
+    /// 根环起点 371、单元区起始槽号 417。checker 与实现今天各自按这几个偏移读；这里照字段表各写一份。
+    const SYSTEM_CONFIGURATION_JOURNAL_RING_START_SLOT_OFFSET: usize = 325;
+    const SYSTEM_CONFIGURATION_JOURNAL_RING_BYTES_OFFSET: usize = 333;
+    const SYSTEM_CONFIGURATION_ROOT_RING_BASE_SLOT_OFFSET: usize = 371;
+    const SYSTEM_CONFIGURATION_UNIT_AREA_START_SLOT_OFFSET: usize = 417;
+    /// 系统配置自举头里的本盘设备号（4 字节）：fsid（偏移 102，宽 16）之后 20 字节、再 1 字节之后，偏移 139。
+    const SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_OFFSET: usize = 139;
+    /// 第一版 mkfs 写进这几格的值（字段表那一列）：journal 环从槽 1024 起、默认环长 768 MiB、根环从槽 64 起、单元区从槽 50176 起。
+    const JOURNAL_RING_START_SLOT_OF_THE_POOL: u64 = 1024;
+    const JOURNAL_RING_BYTES_OF_THE_POOL: u64 = 805_306_368;
+    const ROOT_RING_BASE_SLOT_OF_THE_POOL: u64 = 64;
+    const UNIT_AREA_START_SLOT_OF_THE_POOL: u64 = 50176;
+    /// 聚簇段 64 槽（D3（空间分配） 已定项 10 ①）：单元区起始槽号要落在它的整数倍上。
+    const CLUSTER_SEGMENT_SLOTS_OF_THE_POOL: u64 = 64;
+
+    /// 系统配置槽重封：自证校验和在 155，罩整槽 4096。
+    fn mutate_system_configuration_slot(
+        pool: &mut MemoryPool,
+        device: u32,
+        slot_offset: u64,
+        change: impl Fn(&mut Vec<u8>),
+    ) {
+        let mut bytes = read(pool, device, slot_offset, 4096);
+        change(&mut bytes);
+        let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 155);
+        bytes[155..187].copy_from_slice(&digest);
+        write(pool, device, slot_offset, &bytes);
+    }
+
+    fn set_u64(bytes: &mut [u8], offset: usize, value: u64) {
+        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
+    }
+    fn set_u16(bytes: &mut [u8], offset: usize, value: u16) {
+        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
+    }
+    fn get_u32(bytes: &[u8], offset: usize) -> u32 {
+        u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 字节"))
+    }
+
+    fn set_u32(bytes: &mut [u8], offset: usize, value: u32) {
+        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
+    }
+
+    /// 两块盘对调盘体：每块盘两个系统配置槽里的本盘设备号改成另一块盘的身份（盘 0 的两槽写 1、盘 1 的两槽写 0），整槽重封，
+    /// 别的字节不动。与盘体真对调不同的只有这一格：根环区域、单元都还在原来的盘上，所以别的不变量照判成立。
+    fn swap_the_own_device_numbers_of_the_two_devices(image: &mut MemoryPool) {
+        let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
+        for (device, own_device_number_written) in [(0u32, 1u32), (1, 0)] {
+            for slot_offset in [0, spacing] {
+                mutate_system_configuration_slot(image, device, slot_offset, |bytes| {
+                    set_u32(
+                        bytes,
+                        SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_OFFSET,
+                        own_device_number_written,
+                    );
+                });
+            }
+        }
+    }
+
+    /// I-7.13（系统配置池级字段在读者收的范围里） 的几何那五格（实审 Y4 的 A、B、C、D、E，用户 2026-09-27 定「读字段，不等于常量就整池拒」）：
+    /// 盘 1 槽 0 的一个几何字段改掉、整槽重封，别的三槽照原样。每项带 checker 该报的那个 `Verdict` 成员名（违例说明里带着它）。
+    fn known_bad_images_of_the_system_configuration_geometry() -> Vec<(&'static str, Mutation)> {
+        let on_the_refused_slot = |change: fn(&mut Vec<u8>)| -> Mutation {
+            Box::new(move |image: &mut MemoryPool| {
+                mutate_system_configuration_slot(
+                    image,
+                    DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT,
+                    OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT,
+                    change,
+                );
+            })
+        };
+        vec![
+            // A：单元区起始槽号往后挪一段（50176 → 50240），环长不动：不是环长现算的那个槽。
+            (
+                "UnitAreaStartNotTheSlotAfterTheJournalRing",
+                on_the_refused_slot(|bytes| {
+                    set_u64(
+                        bytes,
+                        SYSTEM_CONFIGURATION_UNIT_AREA_START_SLOT_OFFSET,
+                        UNIT_AREA_START_SLOT_OF_THE_POOL + CLUSTER_SEGMENT_SLOTS_OF_THE_POOL,
+                    );
+                }),
+            ),
+            // D：环长少一槽（768 MiB − 16 KiB），单元区起始槽号跟着改成它现算的 50175（两者自洽）：不在聚簇段边界上。
+            (
+                "UnitAreaStartOffTheClusterSegmentBoundary",
+                on_the_refused_slot(|bytes| {
+                    set_u64(
+                        bytes,
+                        SYSTEM_CONFIGURATION_JOURNAL_RING_BYTES_OFFSET,
+                        JOURNAL_RING_BYTES_OF_THE_POOL - SLOT,
+                    );
+                    set_u64(
+                        bytes,
+                        SYSTEM_CONFIGURATION_UNIT_AREA_START_SLOT_OFFSET,
+                        UNIT_AREA_START_SLOT_OF_THE_POOL - 1,
+                    );
+                }),
+            ),
+            // B：journal 环起点写 1023：实现按常量 1024 写记录、扫环，按字段扫的读者判的是错位 16 KiB 的另一个环。
+            (
+                "JournalRingStartSlotNotTheFirstVersionConstant",
+                on_the_refused_slot(|bytes| {
+                    set_u64(
+                        bytes,
+                        SYSTEM_CONFIGURATION_JOURNAL_RING_START_SLOT_OFFSET,
+                        JOURNAL_RING_START_SLOT_OF_THE_POOL - 1,
+                    );
+                }),
+            ),
+            // C：根环起点写 65：实现按常量 64 写根，按字段读的读者读的是另一个根环。
+            (
+                "RootRingBaseSlotNotTheFirstVersionConstant",
+                on_the_refused_slot(|bytes| {
+                    set_u64(
+                        bytes,
+                        SYSTEM_CONFIGURATION_ROOT_RING_BASE_SLOT_OFFSET,
+                        ROOT_RING_BASE_SLOT_OF_THE_POOL + 1,
+                    );
+                }),
+            ),
+            // E：根环起点写 0：先于槽距判，报起点、不报「槽 1 不在根环起点之前」。
+            (
+                "RootRingBaseSlotNotTheFirstVersionConstant",
+                on_the_refused_slot(|bytes| {
+                    set_u64(bytes, SYSTEM_CONFIGURATION_ROOT_RING_BASE_SLOT_OFFSET, 0);
+                }),
+            ),
+        ]
+    }
+
+    /// 树表单元里第 index 条条目（条目区从 115 + 2 × 8 = 131 起，每条 200）。
+    fn tree_table_entry_offset(index: usize) -> usize {
+        131 + 200 * index
+    }
+
+    /// 一个码 2 节点的自描述三段（D18（块里携带什么信息） 已定项 7：key 宽 1 字节在 51、条目数 2 字节在 82 + 2k、
+    /// 条目宽 2 字节在 84 + 2k；声明长度 2 字节在 8，条目区从 86 + 2k + 29 起）改成「条目宽 = key 宽、只装一条条目」：
+    /// 条目数改 1、声明长度改 k、largest_key 贴到那一条条目的 key 上、条目区之后的字节清零。
+    ///
+    /// 这么改是为了让**除 I-1.10（码 2 条目宽等于字段表宽） 之外的每一道都还成立**：声明长度 = 条目数 × 条目宽、
+    /// 条目宽 ≥ key 宽（`index_node_view` 判的那两道）、key 区间贴紧首末条目（I-1.1（块头自述逻辑地址））、
+    /// 声明长度之后的补齐为零且在载荷 CRC 内（I-2.3（补齐字节为零且参与校验和））。
+    /// 盘上留下的就是一个处处自洽、唯独条目宽与那棵树的字段表不符的节点——
+    /// I-1.10（码 2 条目宽等于字段表宽） 判的正是这一格，判不出来它，走读下一步就按字段表的固定偏移切这条 k 字节的条目、当场越界
+    /// （panic 面普查 R13）。
+    fn narrow_the_entry_width_to_the_key_width(bytes: &mut [u8]) {
+        let key_width = usize::from(bytes[51]);
+        let key_width_field = u16::try_from(key_width).expect("key 宽装得进 u16");
+        let entries_start = 86 + 2 * key_width + 29;
+        let first_key: Vec<u8> = bytes[entries_start..entries_start + key_width].to_vec();
+        set_u16(bytes, 8, key_width_field);
+        set_u16(bytes, 82 + 2 * key_width, 1);
+        set_u16(bytes, 84 + 2 * key_width, key_width_field);
+        bytes[52 + key_width..52 + 2 * key_width].copy_from_slice(&first_key);
+        for byte in &mut bytes[entries_start + key_width..] {
+            *byte = 0;
+        }
+    }
+
+    /// 码 2 节点头里出生序号那 4 字节的偏移**扣掉 2k**（D18（块里携带什么信息） 已定项 7 的字段表：
+    /// 诞生代号 52 + 2k、fsid 60 + 2k、写序 68 + 2k、出生序号 72 + 2k；k 是节点自述的 key 宽，在偏移 51）。
+    const INDEX_NODE_BIRTH_SEQUENCE_OFFSET_BEFORE_THE_KEYS: usize = 72;
+    /// 码 1 数据单元头里诞生代号那 8 字节的偏移（D18（块里携带什么信息） 已定项 7 的字段表）。
+    const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;
+    /// extent 树上段叶（key 宽 24）里第一条条目内联的那条数据指针中诞生代号那 8 字节的偏移：
+    /// 条目区从 115 + 2 × 24 = 163 起，条目 = key (0, inode 8, 0) 24 + 标签 1（2 = 单单元内联）+ 数据指针 88
+    /// （D8（核心索引结构） 已定项 14），指针里诞生代号在 42。
+    const EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET: usize = 163 + 24 + 1 + 42;
+    /// 一条树表条目里诞生 txg 那 8 字节的偏移：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8
+    /// （D8（核心索引结构） 已定项 8 的字段表）。
+    const TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8;
+    /// 分配记录树叶（key 宽 10）：条目数在 82 + 2 × 10、条目区从 115 + 2 × 10 起，每条 20 = key (设备 4, 槽号 6) +
+    /// value (跨度段 2，最高位是已释放标志, 分配代或释放代 8)。
+    const ALLOCATION_NODE_ENTRY_COUNT_OFFSET: usize = 102;
+    const ALLOCATION_NODE_ENTRY_START: usize = 135;
+    const ALLOCATION_RECORD_BYTES: usize = 20;
+    const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
+
+    fn get_u64(bytes: &[u8], offset: usize) -> u64 {
+        u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
+    }
+
+    /// 中央映射树（根兼叶，key 宽 27、条目 55）里唯一那条码 1 条目：key 的出生 txg（key 偏移 9）从 `from` 改成 `to`。
+    /// 码 1 排在最前（类标签 1 最小），头里的最小 key 就是它的 key，跟着改；改完仍按 key 升序。
+    fn rewrite_the_data_unit_mapping_key_birth_txg(bytes: &mut [u8], from: u64, to: u64) {
+        const MAPPING_KEY_BIRTH_TXG_OFFSET: usize = 9;
+        const HEADER_SMALLEST_KEY_OFFSET: usize = 52;
+        let key_width = usize::from(bytes[51]);
+        assert_eq!(key_width, 27, "映射 key 宽 27");
+        let entries_start = 115 + 2 * key_width;
+        assert_eq!(bytes[entries_start], 1, "第一条是码 1 那一条");
+        assert_eq!(
+            bytes[entries_start..entries_start + key_width],
+            bytes[HEADER_SMALLEST_KEY_OFFSET..HEADER_SMALLEST_KEY_OFFSET + key_width],
+            "头里的最小 key 就是第一条的 key"
+        );
+        for key_start in [entries_start, HEADER_SMALLEST_KEY_OFFSET] {
+            assert_eq!(
+                get_u64(bytes, key_start + MAPPING_KEY_BIRTH_TXG_OFFSET),
+                from,
+                "码 1 映射 key 的出生 txg 本来是 {from}"
+            );
+            set_u64(bytes, key_start + MAPPING_KEY_BIRTH_TXG_OFFSET, to);
+        }
+    }
+
+    /// 实审 B2 新立的「中央映射条目的 key 与它指的单元头相符」：I-1.11（映射 key 与单元头相符）。
+    const MAPPING_KEY_INVARIANT: &str =
+        singlefs_checker::image::MAPPING_KEY_MATCHES_THE_UNIT_HEADER;
+
+    /// 中央映射树（根兼叶，key 宽 27、条目 55）里指着 `slot` 那个码 2 节点的那条条目：key 尾段的出生序号（key 偏移 21，4 字节）+1。
+    /// 头里的最小 / 最大 key 是它时跟着改；改完仍按 key 严格升序（码 2 的 key 按出生树分开，这一条与左右邻不同出生树）。
+    fn raise_the_mapping_key_tail_of_the_node_at(bytes: &mut [u8], slot: u64) {
+        const MAPPING_KEY_TAIL_OFFSET: usize = 21;
+        const MAPPING_ENTRY_BYTES: usize = 55;
+        const MAPPING_ENTRY_FIRST_LOCATION_SLOT_OFFSET: usize = 27 + 4;
+        const HEADER_SMALLEST_KEY_OFFSET: usize = 52;
+        let key_width = usize::from(bytes[51]);
+        assert_eq!(key_width, 27, "映射 key 宽 27");
+        let header_largest_key_offset = HEADER_SMALLEST_KEY_OFFSET + key_width;
+        let entry_count = usize::from(u16::from_le_bytes([
+            bytes[82 + 2 * key_width],
+            bytes[83 + 2 * key_width],
+        ]));
+        let entries_start = 115 + 2 * key_width;
+        let entry_start = (0..entry_count)
+            .map(|index| entries_start + index * MAPPING_ENTRY_BYTES)
+            .find(|start| {
+                bytes[*start] == 2
+                    && get_u64(bytes, start + MAPPING_ENTRY_FIRST_LOCATION_SLOT_OFFSET)
+                        & 0xFFFF_FFFF_FFFF
+                        == slot
+            })
+            .expect("映射里有指着这个码 2 节点的那一条");
+        let old_key = bytes[entry_start..entry_start + key_width].to_vec();
+        let tail = entry_start + MAPPING_KEY_TAIL_OFFSET;
+        let birth_sequence = u32::from_le_bytes(bytes[tail..tail + 4].try_into().expect("4 字节"));
+        bytes[tail..tail + 4].copy_from_slice(&(birth_sequence + 1).to_le_bytes());
+        let new_key = bytes[entry_start..entry_start + key_width].to_vec();
+        for header_key in [HEADER_SMALLEST_KEY_OFFSET, header_largest_key_offset] {
+            if bytes[header_key..header_key + key_width] == old_key[..] {
+                bytes[header_key..header_key + key_width].copy_from_slice(&new_key);
+            }
+        }
+    }
+
+    /// 分配记录树叶里，把代字段等于 `from` 的已释放记录改成 `to`；返回改了几条。
+    fn rewrite_released_generations(bytes: &mut [u8], from: u64, to: u64) -> usize {
+        let count = usize::from(u16::from_le_bytes([
+            bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
+            bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
+        ]));
+        let mut rewritten = 0;
+        for index in 0..count {
+            let record = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
+            let span_field = u16::from_le_bytes([bytes[record + 10], bytes[record + 11]]);
+            if span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0
+                && get_u64(bytes, record + 12) == from
+            {
+                set_u64(bytes, record + 12, to);
+                rewritten += 1;
+            }
+        }
+        rewritten
+    }
+
+    /// 分配记录树叶里槽号是 `slot` 的记录（两盘各一条）跨度改成 `span_slots`，已释放标志照留；返回改了几条。
+    fn set_allocation_record_span(bytes: &mut [u8], slot: u64, span_slots: u16) -> usize {
+        let count = usize::from(u16::from_le_bytes([
+            bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
+            bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
+        ]));
+        let mut rewritten = 0;
+        for index in 0..count {
+            let record = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
+            let mut slot_bytes = [0u8; 8];
+            slot_bytes[..6].copy_from_slice(&bytes[record + 4..record + 10]);
+            if u64::from_le_bytes(slot_bytes) == slot {
+                let span_field = u16::from_le_bytes([bytes[record + 10], bytes[record + 11]]);
+                set_u16(
+                    bytes,
+                    record + 10,
+                    (span_field & ALLOCATION_RECORD_RELEASED_FLAG) | span_slots,
+                );
+                rewritten += 1;
+            }
+        }
+        rewritten
+    }
+
+    /// 一份坏镜像：它要让哪一条判违例，以及怎么从干净镜像改出来。
+    type Mutation = Box<dyn Fn(&mut MemoryPool)>;
+
+    fn known_bad_images(clean: &MemoryPool) -> Vec<(&'static str, Mutation)> {
+        let leaf_head_checksum = crc32_castagnoli(&read(clean, 0, INODE_LEAF * SLOT, 16384));
+        let stray_node = {
+            let mut stray = read(clean, 0, EXTENT_ROOT * SLOT, 16384);
+            set_u64(&mut stray, 42, 65624);
+            reseal_unit(&mut stray);
+            stray
+        };
+        let orphan_with_instance_two = {
+            let mut orphan = read(clean, 0, DATA_UNIT * SLOT, 32768);
+            orphan[91..95].copy_from_slice(&2u32.to_le_bytes());
+            reseal_unit(&mut orphan);
+            orphan
+        };
+        let orphan_for_second_sentence = orphan_with_instance_two.clone();
+        let record_with_instance_two = {
+            let offset = singlefs_core::journal::record_offset(
+                3,
+                singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
+            )
+            .0;
+            let mut record = read(clean, 0, offset, 4096);
+            record[16..20].copy_from_slice(&2u32.to_le_bytes());
+            let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
+            record[46..78].copy_from_slice(&digest);
+            record
+        };
+        vec![
+            // 第 0 代根（暖机 txg 1、2）引用的 mkfs 树表单元（50178，A 换下之后只有它们引用）内容改掉再重封：那条候选根指着的块被复用。
+            (
+                "I-7.4",
+                Box::new(|image: &mut MemoryPool| {
+                    let mut unit = read(image, 0, GENESIS_TREE_TABLE * SLOT, 16384);
+                    set_u64(&mut unit, 42, 77);
+                    reseal_unit(&mut unit);
+                    for device in DEVICES {
+                        write(image, device, GENESIS_TREE_TABLE * SLOT, &unit);
+                    }
+                }),
+            ),
+            // 同一个单元的头抹成零、不重封：从第 0 代根出发的遍历读不到它。
+            (
+                "I-4.8",
+                Box::new(|image: &mut MemoryPool| {
+                    let erased = vec![0u8; 16384];
+                    for device in DEVICES {
+                        write(image, device, GENESIS_TREE_TABLE * SLOT, &erased);
+                    }
+                }),
+            ),
+            // 数据单元五元组里的锚点偏移改成 32768（extent key 的偏移是 0）。
+            (
+                "I-1.1",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(image, DATA_UNIT, |bytes| set_u64(bytes, 67, 32768), true)
+                }),
+            ),
+            // extent 树根头里的树 ID 改成 99。
+            (
+                "I-1.3",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(image, EXTENT_ROOT, |bytes| set_u64(bytes, 42, 99), true)
+                }),
+            ),
+            // inode 叶容器头里的 fsid 改掉。
+            (
+                "I-1.4",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(image, INODE_LEAF, |bytes| set_u64(bytes, 81, 0xDEAD), true)
+                }),
+            ),
+            // 盘 1 被择的那个系统配置槽里 fsid 改成别的池的（一块外来的旧盘）；单元头照旧。
+            (
+                "I-1.4",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_system_configuration_slot(image, 1, 4096, |bytes| bytes[102] ^= 0xFF)
+                }),
+            ),
+            // 码 3 容器类身份段的首字节改成 1。
+            (
+                "I-1.6",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(image, INODE_LEAF, |bytes| bytes[42] = 1, true)
+                }),
+            ),
+            // 类型 2 容器的记录宽改成 139（登记的是 140）。
+            (
+                "I-1.7",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(image, INODE_LEAF, |bytes| set_u16(bytes, 71, 139), true)
+                }),
+            ),
+            // 记账树根自述的条目宽缩到 key 宽 22（登记的字段表宽 34），别的自描述字段跟着调成自洽的一份。
+            (
+                "I-1.10",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(
+                        image,
+                        ACCOUNTING_ROOT,
+                        |bytes| narrow_the_entry_width_to_the_key_width(bytes),
+                        true,
+                    )
+                }),
+            ),
+            // 数据单元盘 0 那一份翻一个载荷字节，不补任何校验和。
+            (
+                "I-2.1",
+                Box::new(|image: &mut MemoryPool| {
+                    let mut bytes = read(image, 0, DATA_UNIT * SLOT, 32768);
+                    bytes[200] ^= 1;
+                    write(image, 0, DATA_UNIT * SLOT, &bytes);
+                }),
+            ),
+            // 数据单元补齐区写一个非零字节，载荷 CRC 跟着重算（校验和对、补齐不为 0）。
+            (
+                "I-2.3",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(image, DATA_UNIT, |bytes| bytes[32000] = 1, true)
+                }),
+            ),
+            // 数据单元头里翻一个字节，不重封头校验和（整单元校验和照样补上去，I-2.1 不先挡）。
+            (
+                "I-2.4",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(image, DATA_UNIT, |bytes| bytes[60] ^= 1, false)
+                }),
+            ),
+            // 树表里 extent 树那条的根指针，两条位置条目对调（盘 1 在前）。
+            (
+                "I-2.5",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(
+                        image,
+                        TREE_TABLE,
+                        |bytes| {
+                            let pointer = tree_table_entry_offset(0) + 14;
+                            let first: Vec<u8> = bytes[pointer + 50..pointer + 64].to_vec();
+                            let second: Vec<u8> = bytes[pointer + 64..pointer + 78].to_vec();
+                            bytes[pointer + 50..pointer + 64].copy_from_slice(&second);
+                            bytes[pointer + 64..pointer + 78].copy_from_slice(&first);
+                        },
+                        true,
+                    );
+                }),
+            ),
+            // 记账树里盘 0 的「已分配」多记 16384。
+            (
+                "I-3.1",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(
+                        image,
+                        ACCOUNTING_ROOT,
+                        |bytes| adjust_accounting(bytes, 1, 0, 16384),
+                        true,
+                    )
+                }),
+            ),
+            // 树表里分配记录树那条的根指针，盘 0 那条位置条目改指槽 50243（压在 inode 叶容器的第二个槽上）。
+            (
+                "I-5.1",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(
+                        image,
+                        TREE_TABLE,
+                        |bytes| {
+                            let location = tree_table_entry_offset(2) + 14 + 50 + 4;
+                            bytes[location..location + 6]
+                                .copy_from_slice(&50243u64.to_le_bytes()[..6]);
+                        },
+                        true,
+                    );
+                }),
+            ),
+            // 记账树里盘 0 的「空闲」多记 16384。
+            (
+                "I-5.2",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(
+                        image,
+                        ACCOUNTING_ROOT,
+                        |bytes| adjust_accounting(bytes, 2, 0, 16384),
+                        true,
+                    )
+                }),
+            ),
+            // 根环全部槽抹成 0。
+            (
+                "I-7.1",
+                Box::new(|image: &mut MemoryPool| {
+                    for (device, offset) in root_slots() {
+                        write(image, device, offset, &[0u8; 512]);
+                    }
+                }),
+            ),
+            // inode 叶容器两份都改坏一个字节（不补校验和）：最新的根走不完。
+            (
+                "I-7.2",
+                Box::new(|image: &mut MemoryPool| {
+                    for device in DEVICES {
+                        let mut bytes = read(image, device, INODE_LEAF * SLOT, 32768);
+                        bytes[5000] ^= 1;
+                        write(image, device, INODE_LEAF * SLOT, &bytes);
+                    }
+                }),
+            ),
+            // 盘 0 被择的那个系统配置槽（槽 1，世代 5）里逐区域设备改成 [0, 0, 0]。
+            (
+                "I-7.6",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_system_configuration_slot(image, 0, 4096, |bytes| {
+                        bytes[379..391].copy_from_slice(&[0u8; 12])
+                    })
+                }),
+            ),
+            // 盘 1 槽 0 的加密类型改成 1、重算整槽校验和：这一槽自证过而带这一版读者不收的值，实现整池拒绝挂载
+            // （A3-checker-2 第四节第 1 条，别的三槽照原样）。
+            (
+                SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_system_configuration_slot(
+                        image,
+                        DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT,
+                        OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT,
+                        |bytes| {
+                            bytes[SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET] =
+                                ENCRYPTION_TYPE_THE_FIRST_VERSION_DOES_NOT_READ
+                        },
+                    )
+                }),
+            ),
+            // 两块盘四个系统配置槽里的本盘设备号互换、整槽重封（实审 Y2-b 越格线索：盘体对调之后的池可写挂载因本盘设备号拒）。
+            (
+                SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
+                Box::new(swap_the_own_device_numbers_of_the_two_devices),
+            ),
+            // C322（取号那一步的屏障怎么放没有条款） 的撞号镜像：没人引用的槽 50302 上放一份写序实例代号 2 的数据单元（头与载荷
+            // 校验和都过），两盘系统配置都还是实例 1 ⇒ ① 红（旧的「各盘相等且 ≥ 根环」在这里判成立）。
+            (
+                "I-7.7",
+                Box::new(move |image: &mut MemoryPool| {
+                    write(image, 0, 50302 * SLOT, &orphan_with_instance_two)
+                }),
+            ),
+            // journal 环里一个空槽放一条实例代号 2 的记录（头校验和重封）⇒ ① 红：扫描方向要读 journal 环。
+            (
+                "I-7.7",
+                Box::new(move |image: &mut MemoryPool| {
+                    write(
+                        image,
+                        0,
+                        singlefs_core::journal::record_offset(
+                            9,
+                            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
+                        )
+                        .0,
+                        &record_with_instance_two,
+                    )
+                }),
+            ),
+            // 盘 0 较新那一槽的实例代号改成 2，再放那份实例代号 2 的孤儿 ⇒ 各盘最大号 [2, 1] 不等而较大者有东西带着，② 红（① 成立）。
+            (
+                "I-7.7",
+                Box::new(move |image: &mut MemoryPool| {
+                    mutate_system_configuration_slot(image, 0, 4096, |bytes| {
+                        bytes[477..481].copy_from_slice(&2u32.to_le_bytes())
+                    });
+                    write(image, 0, 50302 * SLOT, &orphan_for_second_sentence);
+                }),
+            ),
+            // 在没人引用的槽 50300 上放一个树 ID 65624（高于水位 65548）的码 2 节点（头校验和过、诞生代号 3）。
+            (
+                "I-7.8",
+                Box::new(move |image: &mut MemoryPool| write(image, 0, 50300 * SLOT, &stray_node)),
+            ),
+            // 树表里 inode 树那条的根指针改指 inode 叶容器（码 3），校验和取那个容器前 16 KiB 的 CRC。
+            (
+                "I-9.1",
+                Box::new(move |image: &mut MemoryPool| {
+                    mutate_unit(
+                        image,
+                        TREE_TABLE,
+                        |bytes| {
+                            let pointer = tree_table_entry_offset(1) + 14;
+                            for location in [pointer + 50, pointer + 64] {
+                                bytes[location + 4..location + 10]
+                                    .copy_from_slice(&INODE_LEAF.to_le_bytes()[..6]);
+                                bytes[location + 10..location + 14]
+                                    .copy_from_slice(&leaf_head_checksum.to_le_bytes());
+                            }
+                        },
+                        true,
+                    );
+                }),
+            ),
+            // inode 树根那条内部条目的身份引用里容器号改成 2（子头里是 1）。
+            (
+                "I-9.2",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(image, INODE_ROOT, |bytes| set_u64(bytes, 131 + 18, 2), true)
+                }),
+            ),
+            // 容器号改成 5，而容器里那条记录的 inode 是 1。
+            (
+                "I-9.4",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(image, INODE_LEAF, |bytes| set_u64(bytes, 53, 5), true)
+                }),
+            ),
+            // inode 记录的预留段（记录偏移 139）写一个非零字节。
+            (
+                "I-9.7",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(image, INODE_LEAF, |bytes| bytes[136 + 139] = 1, true)
+                }),
+            ),
+            // inode 记录的对象出生代改成 2（数据单元五元组里是 3）。
+            (
+                "I-9.10",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(image, INODE_LEAF, |bytes| set_u64(bytes, 136 + 8, 2), true)
+                }),
+            ),
+            // inode 记录的 blocks（记录偏移 48）写回按分到一个 32 KiB 单元算的 64，而 size 3000 ⇒ ⌈3000 ÷ 512⌉ = 6。
+            (
+                "I-9.15",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(
+                        image,
+                        INODE_LEAF,
+                        |bytes| set_u64(bytes, 136 + INODE_RECORD_BLOCKS_OFFSET, 64),
+                        true,
+                    )
+                }),
+            ),
+            // 树表里 livelist 与稀疏旁表两条互换种类：盘上仍按树 ID 升序（I-1.1 不红），按发号次序 livelist 的号 65542 大过稀疏旁表的号 65541。
+            (
+                "I-9.16",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(
+                        image,
+                        TREE_TABLE,
+                        |bytes| swap_the_kinds_of_the_livelist_and_sparse_side_table_entries(bytes),
+                        true,
+                    )
+                }),
+            ),
+            // 实例表里多一行 (1, 3, 0)：行的实例代号等于挂载根的实例 1，不「低于」；链指针记录仍在末尾、行仍唯一，红的只有那一半。
+            (
+                "I-3.8",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(
+                        image,
+                        INSTANCE_TABLE,
+                        |bytes| {
+                            let chain_record: Vec<u8> = bytes[136..224].to_vec();
+                            let mut row = vec![0u8; 88];
+                            row[1..5].copy_from_slice(&1u32.to_le_bytes());
+                            row[5..13].copy_from_slice(&3u64.to_le_bytes());
+                            bytes[136..224].copy_from_slice(&row);
+                            bytes[224..312].copy_from_slice(&chain_record);
+                            set_u16(bytes, 69, 2);
+                            set_u16(bytes, 8, 176);
+                        },
+                        true,
+                    );
+                }),
+            ),
+            // 盘 0 环内偏移 4096 那条记录（w4，暖机第二条空发布）的反向链字段改成 0，头校验和重封：
+            // 它不是本实例第一条（本实例第一条是 w1、计数器 1），链值必须等于 CRC32C(w1 的 311 字节头)。
+            (
+                "I-8.6",
+                Box::new(|image: &mut MemoryPool| {
+                    let offset = singlefs_core::journal::record_offset(
+                        2,
+                        singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
+                    )
+                    .0;
+                    let mut record = read(image, 0, offset, 4096);
+                    let back_chain_field =
+                        JOURNAL_RECORD_BACK_CHAIN_OFFSET..JOURNAL_RECORD_BACK_CHAIN_OFFSET + 4;
+                    assert_ne!(
+                        u32::from_le_bytes(
+                            record[back_chain_field.clone()].try_into().expect("4 字节")
+                        ),
+                        0,
+                        "w4 的反向链本来是 CRC32C(w1 的头)，非 0"
+                    );
+                    record[back_chain_field].copy_from_slice(&0u32.to_le_bytes());
+                    let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
+                    record[46..78].copy_from_slice(&digest);
+                    write(image, 0, offset, &record);
+                }),
+            ),
+            // inode 叶容器**盘 1 那一份**的载荷改一个字节、重封载荷校验和与头校验和，类身份段与写序一个字节不动：
+            // 两份归并成同一组（归并键逐字节相同）而载荷不同。盘 1 那条位置条目跟着补上 ⇒ I-2.1 不先挡。
+            (
+                "I-1.8",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_one_device_copy_of_unit(image, &UNITS, 1, INODE_LEAF, |bytes| {
+                        assert_eq!(bytes[20000], 0, "记录区之后的补齐本来是 0");
+                        bytes[20000] = 1;
+                    });
+                }),
+            ),
+            // extent 树根（码 2，key 宽 24）头里的出生序号 +1，引用它的树表条目里那条指针一个字节不动：
+            // 头里的出生身份与引用它的指针记的不符 ⇒ I-1.2（块头写序已发布） 红。
+            // 出生序号不是已发布谓词的输入（谓词只读诞生代号与写序），所以 I-4.2（无被引用未提交块） 不跟着红。
+            // 中央映射里 extent 根那条 key 的尾段就是它的出生序号（D19（块指针的结构与宽度预算） 已定项 6）：跟着 +1，
+            // 不跟着改就是映射 key 与单元头不符、映射 key 那一条（I-1.11（映射 key 与单元头相符）） 一起红。
+            (
+                "I-1.2",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(
+                        image,
+                        EXTENT_ROOT,
+                        |bytes| {
+                            let offset = INDEX_NODE_BIRTH_SEQUENCE_OFFSET_BEFORE_THE_KEYS
+                                + 2 * usize::from(bytes[51]);
+                            let birth_sequence = u32::from_le_bytes(
+                                bytes[offset..offset + 4].try_into().expect("4 字节"),
+                            );
+                            bytes[offset..offset + 4]
+                                .copy_from_slice(&(birth_sequence + 1).to_le_bytes());
+                        },
+                        true,
+                    );
+                    mutate_unit(
+                        image,
+                        MAPPING_ROOT,
+                        |bytes| raise_the_mapping_key_tail_of_the_node_at(bytes, EXTENT_ROOT),
+                        true,
+                    );
+                }),
+            ),
+            // 中央映射里数据单元那条 key 的出生 txg 从 3 改成 4，数据单元一个字节不动：映射 key 与它指的单元头里的诞生代号不符
+            // ⇒ 映射 key 那一条（I-1.11（映射 key 与单元头相符）） 红。
+            (
+                MAPPING_KEY_INVARIANT,
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(
+                        image,
+                        MAPPING_ROOT,
+                        |bytes| rewrite_the_data_unit_mapping_key_birth_txg(bytes, 3, 4),
+                        true,
+                    );
+                }),
+            ),
+            // 数据单元头里的诞生代号从 3 改成 4（最新的根就是 txg 3、实例 1），引用它的 extent 记录里那条数据指针的
+            // 诞生代号跟着改成 4：指针与头仍然一致（I-1.2（块头写序已发布） 不跟着红），而这个**被引用**的块按
+            // C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的已发布谓词判不成已发布（i == 挂载根实例 ⇒ 要 b ≤ 挂载根的 txg）
+            // ⇒ I-4.2（无被引用未提交块） 红：盘上留下的正是「提交了的根指着一个还没提交的块」。
+            // 分配记录树里那个落点的分配代也跟着改成 4（两盘各一条）：记录与单元同源，不跟着改就是 I-3.10（已分配记录的分配代等于
+            // 它罩住的单元的诞生代号） 一起红，判别力算不到 I-4.2 头上。
+            (
+                "I-4.2",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(
+                        image,
+                        DATA_UNIT,
+                        |bytes| {
+                            assert_eq!(
+                                get_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET),
+                                3,
+                                "数据单元的诞生代号本来是 3"
+                            );
+                            set_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET, 4);
+                        },
+                        true,
+                    );
+                    mutate_unit(
+                        image,
+                        EXTENT_ROOT,
+                        |bytes| {
+                            assert_eq!(
+                                get_u64(bytes, EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET),
+                                3,
+                                "extent 记录里那条数据指针的诞生代号本来是 3"
+                            );
+                            set_u64(bytes, EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET, 4);
+                        },
+                        true,
+                    );
+                    // 中央映射里那个数据单元的 key 带着它的诞生代号（D19（块指针的结构与宽度预算） 已定项 6）：跟着改成 4，
+                    // 不跟着改就是映射 key 与单元头的出生身份不符、I-1.2 一起红（实审 B1 起 checker 拿映射 key 比单元头）。
+                    mutate_unit(
+                        image,
+                        MAPPING_ROOT,
+                        |bytes| rewrite_the_data_unit_mapping_key_birth_txg(bytes, 3, 4),
+                        true,
+                    );
+                    assert_eq!(
+                        mutate_allocation_leaves(image, &UNITS, ALLOCATION_LEAVES, |bytes| {
+                            rewrite_unreleased_allocation_generation(bytes, DATA_UNIT, 3, 4)
+                        }),
+                        2,
+                        "数据单元两盘各一条未释放记录（两片叶各一条），分配代本来是 3"
+                    );
+                }),
+            ),
+            // 记账里的 inode 号水位从 2 改成 1：树里最大的 key 就是 1，水位不再严格大于它（I-9.6）。
+            // 别的行一个字节都不动 ⇒ I-3.1 / I-5.2 那两条照旧绿。
+            (
+                "I-9.6",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(
+                        image,
+                        ACCOUNTING_ROOT,
+                        |bytes| set_accounting(bytes, 12, 0xFFFF_FFFF, 1),
+                        true,
+                    );
+                }),
+            ),
+            // inode 根那条条目的分隔 key 从 1 改成 2（连着把节点头的 key 区间也改成 [2, 2]，
+            // 不然 I-1.1（key 区间罩住条目） 先红、这一格就成了搭车）：分隔 key 2 大于这个孩子的最小 key 1 ⇒ I-9.12 红。
+            (
+                "I-9.12",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(
+                        image,
+                        INODE_ROOT,
+                        |bytes| {
+                            set_u64(bytes, 131, 2);
+                            set_u64(bytes, 52, 2);
+                            set_u64(bytes, 60, 2);
+                        },
+                        true,
+                    );
+                }),
+            ),
+            // 类型 2 容器的记录数与声明长度都改成 0。
+            (
+                "I-9.13",
+                Box::new(|image: &mut MemoryPool| {
+                    mutate_unit(
+                        image,
+                        INODE_LEAF,
+                        |bytes| {
+                            set_u16(bytes, 69, 0);
+                            set_u16(bytes, 8, 0);
+                            bytes[136..276].fill(0);
+                        },
+                        true,
+                    );
+                }),
+            ),
+        ]
+    }
+
+    /// 发布 B（txg 4）之后的干净镜像：A 的十二个单元（十四个槽）在这次发布里改写成已释放、释放代 4，树表是 B 写的那一版，
+    /// A 那一版仍被根环里 A 的根指着。I-3.9（释放代落在停止引用它的那一格区间里）与 I-9.14（树表条目的诞生 txg 跨根不变）
+    /// 要的跨根比较到这里才有内容：写完新池新建文件的那份镜像上只有 A 的根有树表条目（mkfs 种的第 0 版树表是空的、
+    /// 两次暖机空发布不写树表），I-9.14 在那份镜像上报「不适用」。
+
+    // 以下是散在原文件第 1295 行之后、被上面用到的常量与辅助函数，原文件行号：3071 1543 3179 3261 3246 3119 3573 3113 3115 3116 3114 3111
+
+    /// inode 记录里 `blocks` 的偏移（D8（核心索引结构） 已定项 6 的偏移表：size 在 40，blocks 在 48）。
+    const INODE_RECORD_BLOCKS_OFFSET: usize = 48;
+
+    /// 记录头里反向链那 4 字节的偏移：事务段 78 + 事务号 8 + 提交标记 1 + 本次发布内序号 4（D23（journal 的角色与格式） 已定项 4 / 已定项 8）。
+    const JOURNAL_RECORD_BACK_CHAIN_OFFSET: usize = 91;
+
+    /// 新池新建文件写出的中央映射树根（bump 次序：分配记录树五个节点、记账树、映射树、树表）。
+    const MAPPING_ROOT: u64 = 50251;
+
+    /// 记账树叶里 (统计量, 设备) 那一行的值加 delta（条目区从 115 + 2 × 22 = 159 起，每条 34：统计量 2、树 8、设备 4、代 8、值 8、seq 4）。
+    fn adjust_accounting(bytes: &mut [u8], statistic: u16, device: u32, delta: u64) {
+        let count = usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
+        for index in 0..count {
+            let row = 159 + 34 * index;
+            if u16::from_le_bytes([bytes[row], bytes[row + 1]]) == statistic
+                && u32::from_le_bytes(bytes[row + 10..row + 14].try_into().expect("4")) == device
+            {
+                let value = u64::from_le_bytes(bytes[row + 22..row + 30].try_into().expect("8"));
+                bytes[row + 22..row + 30].copy_from_slice(&(value + delta).to_le_bytes());
+                return;
+            }
+        }
+        panic!("记账树里没有 ({statistic}, {device}) 这一行");
+    }
+
+    /// 记账树叶里 (统计量, 设备) 那一行的值改成 value（偏移口径同 `adjust_accounting`）。
+    fn set_accounting(bytes: &mut [u8], statistic: u16, device: u32, value: u64) {
+        let count = usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
+        for index in 0..count {
+            let row = 159 + 34 * index;
+            if u16::from_le_bytes([bytes[row], bytes[row + 1]]) == statistic
+                && u32::from_le_bytes(bytes[row + 10..row + 14].try_into().expect("4")) == device
+            {
+                bytes[row + 22..row + 30].copy_from_slice(&value.to_le_bytes());
+                return;
+            }
+        }
+        panic!("记账树里没有 ({statistic}, {device}) 这一行");
+    }
+
+    /// 树表单元里 livelist 与稀疏旁表两条互换种类（条目偏移 10 的 2 字节）；树 ID、根指针不动。
+    fn swap_the_kinds_of_the_livelist_and_sparse_side_table_entries(tree_table: &mut [u8]) {
+        for (index, kind_before, kind_after) in [
+            (
+                LIVELIST_TREE_TABLE_ENTRY_INDEX,
+                TREE_KIND_LIVELIST,
+                TREE_KIND_SPARSE_SIDE_TABLE,
+            ),
+            (
+                SPARSE_SIDE_TABLE_TREE_TABLE_ENTRY_INDEX,
+                TREE_KIND_SPARSE_SIDE_TABLE,
+                TREE_KIND_LIVELIST,
+            ),
+        ] {
+            let kind_offset = tree_table_entry_offset(index) + TREE_TABLE_ENTRY_KIND_OFFSET;
+            assert_eq!(
+                u16::from_le_bytes([tree_table[kind_offset], tree_table[kind_offset + 1]]),
+                kind_before,
+                "树表第 {index} 条的种类"
+            );
+            set_u16(tree_table, kind_offset, kind_after);
+        }
+    }
+
+    /// 分配记录树叶里槽号是 `slot`、**未带**已释放标志、分配代等于 `from` 的记录（两盘各一条）改成 `to`；返回改了几条。
+    fn rewrite_unreleased_allocation_generation(
+        bytes: &mut [u8],
+        slot: u64,
+        from: u64,
+        to: u64,
+    ) -> usize {
+        let count = usize::from(u16::from_le_bytes([
+            bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
+            bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
+        ]));
+        let mut rewritten = 0;
+        for index in 0..count {
+            let record = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
+            let mut slot_bytes = [0u8; 8];
+            slot_bytes[..6].copy_from_slice(&bytes[record + 4..record + 10]);
+            let span_field = u16::from_le_bytes([bytes[record + 10], bytes[record + 11]]);
+            if u64::from_le_bytes(slot_bytes) == slot
+                && span_field & ALLOCATION_RECORD_RELEASED_FLAG == 0
+                && get_u64(bytes, record + 12) == from
+            {
+                set_u64(bytes, record + 12, to);
+                rewritten += 1;
+            }
+        }
+        rewritten
+    }
+
+    /// 新池新建文件那一版树表里十一条按发号次序排：livelist 是第 4 条（从 0 数，号 65541）、稀疏旁表第 5 条（号 65542）。
+    const LIVELIST_TREE_TABLE_ENTRY_INDEX: usize = 4;
+
+    const TREE_KIND_LIVELIST: u16 = 6;
+
+    const TREE_KIND_SPARSE_SIDE_TABLE: u16 = 7;
+
+    const SPARSE_SIDE_TABLE_TREE_TABLE_ENTRY_INDEX: usize = 5;
+
+    /// 树表条目里种类那 2 字节的偏移（树 ID 8 + 条目长度 2 之后，D8（核心索引结构） 已定项 8）。
+    const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
+
+    /// 对外的口子：第 747 行的 `known_bad_images` 是私有的，这里转一手（类型就是第 745 行的 `Mutation`）。
+    pub(super) fn entries_of_the_known_bad_images(
+        clean: &MemoryPool,
+    ) -> Vec<(&'static str, Box<dyn Fn(&mut MemoryPool)>)> {
+        known_bad_images(clean)
+    }
+}
+
+// ───────────────────────────── 坏镜像：镜像物化、补丁与摘要（登记 5.5） ─────────────────────────────
+
+/// 一处字节补丁：在底态镜像上把 (盘, 偏移) 起的这几个字节换掉（与 E170 的 `bad-patches.bin` 同一种补丁）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct BytePatch {
+    device: u32,
+    offset: u64,
+    bytes: Vec<u8>,
+}
+
+/// 把一个崩溃镜像物化成 `MemoryPool`：基镜像拷一份，按写表次序施加持久了的写。
+fn materialized_memory_pool(image: &CrashImage<'_>) -> MemoryPool {
+    let mut pool = image.base.clone();
+    let persisted_writes: Vec<RetainedWrite> = image
+        .writes
+        .iter()
+        .zip(&image.persisted)
+        .filter(|(_, is_persisted)| **is_persisted)
+        .map(|(write, _)| write.clone())
+        .collect();
+    pool.apply_writes(&persisted_writes);
+    pool
+}
+
+/// 阳性对照 O2：把已知坏镜像族的第一条改法施加在这一态的物化镜像上，再交 `check_pool_image`。
+fn pool_checker_on_the_first_known_bad_image(
+    image: &CrashImage<'_>,
+) -> Vec<(&'static str, InvariantVerdict)> {
+    let clean = materialized_memory_pool(image);
+    let entries =
+        known_bad_images_copied_from_the_harness_test::entries_of_the_known_bad_images(&clean);
+    let (_, apply_the_first_entry) = entries
+        .first()
+        .expect("已知坏镜像族至少一条（S4：装置数出的条数要是 41）");
+    let mut bad = clean.clone();
+    apply_the_first_entry(&mut bad);
+    check_pool_image(&bad)
+}
+
+/// 一个 `MemoryPool` 的镜像摘要（E170 登记 5.3 的算法）：每块盘（身份升序）、每个不全为零的扇区（扇区号升序）依次喂进盘身份 u32、扇区号 u64、512 字节。
+fn memory_pool_digest(pool: &MemoryPool) -> [u8; DIGEST_BYTES] {
+    let mut message = Vec::new();
+    for (device, sparse) in &pool.devices {
+        for (sector, bytes) in sparse.written_sectors() {
+            append_sector_to_digest_message(&mut message, device.0, *sector, bytes);
+        }
+    }
+    sha256_digest(&message)
+}
+
+/// 从中性格式按落地向量重建一态的 `MemoryPool`（按流序叠，规则与 [`rebuilt_image_digest`] 相同；两处各算各的摘要，S4 互比）。
+fn rebuilt_memory_pool(stream: &NeutralStream, landing_vector: &str) -> MemoryPool {
+    let mut pool =
+        MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], LOCAL_DEVICE_BYTES);
+    for ((device, sector), bytes) in &stream.base_sectors {
+        pool.devices
+            .get_mut(&DeviceIdentity(*device))
+            .expect("基镜像的盘在池里")
+            .write(DeviceOffsetInBytes(sector * LOCAL_SECTOR_BYTES), bytes);
+    }
+    let mut landings = landing_vector.chars();
+    let mut write_index = 0usize;
+    for operation in &stream.operations {
+        if operation.kind == RecordedOperationKind::Barrier {
+            continue;
+        }
+        let landing = landings.next().expect("落地向量与写数一样长");
+        let length = usize::try_from(operation.length).expect("写长装得进 usize");
+        let device = pool
+            .devices
+            .get_mut(&DeviceIdentity(operation.device))
+            .expect("写的盘在池里");
+        match (landing, operation.kind) {
+            ('0', _) => {}
+            ('2', RecordedOperationKind::WriteZeroes) => {
+                device.zero_fill(DeviceOffsetInBytes(operation.offset), operation.length);
+            }
+            ('2', _) => {
+                let start = operation.payload_start.expect("带字节的写有载荷起点");
+                device.write(
+                    DeviceOffsetInBytes(operation.offset),
+                    &stream.payload[start..start + length],
+                );
+            }
+            ('1', _) => {
+                let (start, torn_length) = stream.torn_bytes_by_write[&write_index];
+                assert_eq!(torn_length, length, "撕裂那一段罩整次写");
+                device.write(
+                    DeviceOffsetInBytes(operation.offset),
+                    &stream.torn_payload[start..start + torn_length],
+                );
+            }
+            (unknown, _) => panic!("落地向量里认不出的字符 {unknown:?}"),
+        }
+        write_index += 1;
+    }
+    assert!(landings.next().is_none(), "落地向量比写数长");
+    pool
+}
+
+/// 把一串补丁施加到 `MemoryPool` 上：补丁的起点与长度不一定按扇区对齐（E170 的 P 行只翻一个字节），
+/// 所以先读出罩住它的整扇区、把补丁字节叠上去、再整扇区写回。
+fn apply_patches(pool: &mut MemoryPool, patches: &[BytePatch]) {
+    let sector_bytes = LOCAL_SECTOR_BYTES;
+    for patch in patches {
+        let device = pool
+            .devices
+            .get_mut(&DeviceIdentity(patch.device))
+            .expect("补丁的盘在池里");
+        let patch_length = u64::try_from(patch.bytes.len()).expect("补丁长装得进 u64");
+        let first_sector_start = patch.offset / sector_bytes * sector_bytes;
+        let end = (patch.offset + patch_length).div_ceil(sector_bytes) * sector_bytes;
+        let mut covered = device.read(
+            DeviceOffsetInBytes(first_sector_start),
+            usize::try_from(end - first_sector_start).expect("罩住补丁的扇区长装得进 usize"),
+        );
+        let start_in_covered =
+            usize::try_from(patch.offset - first_sector_start).expect("补丁起点在罩住的扇区里");
+        covered[start_in_covered..start_in_covered + patch.bytes.len()]
+            .copy_from_slice(&patch.bytes);
+        device.write(DeviceOffsetInBytes(first_sector_start), &covered);
+    }
+}
+
+/// 两份镜像逐 512 字节扇区的差（缺的扇区按全零），相邻的差异扇区并成一段补丁；补丁带「改坏之后」的字节。
+fn sector_difference_as_patches(clean: &MemoryPool, bad: &MemoryPool) -> Vec<BytePatch> {
+    let sector_bytes = usize::try_from(LOCAL_SECTOR_BYTES).expect("512");
+    let zero_sector = vec![0u8; sector_bytes];
+    let mut patches: Vec<BytePatch> = Vec::new();
+    for (device, bad_sparse) in &bad.devices {
+        let clean_sparse = clean.devices.get(device).expect("两份镜像的盘相同");
+        let sectors: BTreeSet<u64> = bad_sparse
+            .written_sectors()
+            .keys()
+            .chain(clean_sparse.written_sectors().keys())
+            .copied()
+            .collect();
+        let mut open: Option<(u64, BytePatch)> = None;
+        for sector in sectors {
+            let bad_bytes = bad_sparse
+                .written_sectors()
+                .get(&sector)
+                .unwrap_or(&zero_sector);
+            let clean_bytes = clean_sparse
+                .written_sectors()
+                .get(&sector)
+                .unwrap_or(&zero_sector);
+            if bad_bytes == clean_bytes {
+                continue;
+            }
+            match &mut open {
+                Some((last_sector, patch)) if *last_sector + 1 == sector => {
+                    patch.bytes.extend_from_slice(bad_bytes);
+                    *last_sector = sector;
+                }
+                Some(_) | None => {
+                    if let Some((_, finished)) = open.take() {
+                        patches.push(finished);
+                    }
+                    open = Some((
+                        sector,
+                        BytePatch {
+                            device: device.0,
+                            offset: sector * LOCAL_SECTOR_BYTES,
+                            bytes: bad_bytes.clone(),
+                        },
+                    ));
+                }
+            }
+        }
+        if let Some((_, finished)) = open.take() {
+            patches.push(finished);
+        }
+    }
+    patches
+}
+
+// ───────────────────────────── `known-bad-images` 模式：41 条已知坏镜像与 E170 的 P 行、盲测族上的 O2（登记 5.5） ─────────────────────────────
+
+/// 原用例文件的 sha256（登记第三节、A8）：变了先重数条目再比，不照抄的那一份当作没变。
+const KNOWN_BAD_TEST_FILE_SHA256: &str =
+    "e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820";
+/// `known_bad_images(clean)` 的条目数（登记第七节 A8：第 747–1294 行里 `Box::new(` 41 处，字面名 38 条、常量名 3 条）。
+const KNOWN_BAD_ENTRY_COUNT: usize = 41;
+
+/// 原用例文件里 `known_bad_images` 一函数里每条的目标不变量名，按文件次序：十二格缩进的一行写字面名（`"I-x.y",`）或常量名。
+fn entry_names_in_the_original_test_file(text: &str) -> Vec<String> {
+    let mut inside = false;
+    let mut names = Vec::new();
+    for line in text.lines() {
+        if line.starts_with("fn known_bad_images(clean: &MemoryPool)") {
+            inside = true;
+            continue;
+        }
+        if inside && line == "}" {
+            break;
+        }
+        if !inside || !line.starts_with("            ") || line.starts_with("             ") {
+            continue;
+        }
+        let token = line.trim().trim_end_matches(',');
+        if let Some(literal) = token
+            .strip_prefix('"')
+            .and_then(|rest| rest.strip_suffix('"'))
+        {
+            if literal.starts_with("I-") {
+                names.push(literal.to_string());
+            }
+        } else if !token.is_empty()
+            && token
+                .chars()
+                .all(|character| character.is_ascii_uppercase() || character == '_')
+        {
+            names.push(match token {
+                "SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS" => {
+                    singlefs_checker::image::SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
+                        .to_string()
+                }
+                "SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY" => {
+                    singlefs_checker::image::SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY
+                        .to_string()
+                }
+                "MAPPING_KEY_INVARIANT" => {
+                    singlefs_checker::image::MAPPING_KEY_MATCHES_THE_UNIT_HEADER.to_string()
+                }
+                unresolved => format!("unresolved_constant:{unresolved}"),
+            });
+        }
+    }
+    names
+}
+
+/// 补丁文本：`设备:偏移:在补丁文件里的起点:长度`，逗号隔开（E170 的 `bad-states.tsv` 同一种写法）；没有补丁写 `-`。
+fn patch_list_text(patches: &[BytePatch], patches_file: &mut Vec<u8>) -> String {
+    if patches.is_empty() {
+        return "-".to_string();
+    }
+    patches
+        .iter()
+        .map(|patch| {
+            let start = patches_file.len();
+            patches_file.extend_from_slice(&patch.bytes);
+            format!(
+                "{}:{}:{start}:{}",
+                patch.device,
+                patch.offset,
+                patch.bytes.len()
+            )
+        })
+        .collect::<Vec<String>>()
+        .join(",")
+}
+
+/// 读 E170 的补丁文本回成补丁。
+fn patches_of_a_patch_list_text(list: &str, patches_file: &[u8]) -> Vec<BytePatch> {
+    if list == "-" {
+        return Vec::new();
+    }
+    list.split(',')
+        .map(|item| {
+            let fields: Vec<&str> = item.split(':').collect();
+            assert_eq!(fields.len(), 4, "补丁四段：{item:?}");
+            let start: usize = parse_number(fields[2], "补丁起点");
+            let length: usize = parse_number(fields[3], "补丁长度");
+            BytePatch {
+                device: parse_number(fields[0], "补丁的盘"),
+                offset: parse_number(fields[1], "补丁偏移"),
+                bytes: patches_file[start..start + length].to_vec(),
+            }
+        })
+        .collect()
+}
+
+/// 41 条已知坏镜像在流一全持久态上的 O2：每一条施加在干净镜像的拷贝上，一边直接判（`MemoryPool`），一边取补丁（逐扇区的差）再叠回干净镜像判，
+/// 两边判红的不变量集合要相等（S4）、目标不变量要在里面（S4）。交回（停机条款没过的条数）。
+fn known_bad_images_on_the_first_stream(
+    dump_directory: &Path,
+    output_directory: &Path,
+    lines: &mut ResultLines,
+) -> usize {
+    let mut failed = 0usize;
+    let stream_directory = dump_directory.join(StreamName::NewPoolFileCreation.directory_name());
+    let neutral = read_neutral_stream(&stream_directory);
+    let write_count = neutral
+        .operations
+        .iter()
+        .filter(|operation| operation.kind != RecordedOperationKind::Barrier)
+        .count();
+    let all_persisted = "2".repeat(write_count);
+    let clean = rebuilt_memory_pool(&neutral, &all_persisted);
+    let clean_digest = sha256_text(&memory_pool_digest(&clean));
+    let reference_digest = sha256_text(&memory_pool_digest(
+        &common::build_pool("e171-known-bad").memory_pool(),
+    ));
+    let states_digest_of_the_full_state = read_text_lines(&stream_directory, STATES_FILE)
+        .iter()
+        .map(|line| {
+            line.split('\t')
+                .map(str::to_string)
+                .collect::<Vec<String>>()
+        })
+        .find(|fields| fields[4] == all_persisted)
+        .map(|fields| fields[5].clone());
+    for check in [
+        StopCheck::new(
+            "known_bad_clean_image_digest_equals_build_pool",
+            &reference_digest,
+            &clean_digest,
+        ),
+        StopCheck::new(
+            "known_bad_clean_image_digest_equals_the_full_state_of_the_dump",
+            Some(&clean_digest),
+            states_digest_of_the_full_state.as_ref(),
+        ),
+    ] {
+        failed += usize::from(!check.holds());
+        lines.result(&format!(
+            "name=s4 check={} local={} crates={} holds={}",
+            check.name,
+            field_text(&check.local),
+            field_text(&check.crates),
+            check.holds()
+        ));
+    }
+    let test_file_path = Path::new(env!("CARGO_MANIFEST_DIR"))
+        .join("../singlefs-harness/tests/checker_known_bad_images.rs");
+    let test_file_text = std::fs::read_to_string(&test_file_path)
+        .unwrap_or_else(|error| panic!("读 {}：{error}", test_file_path.display()));
+    let test_file_sha256 = sha256_hexadecimal(test_file_text.as_bytes());
+    let entries =
+        known_bad_images_copied_from_the_harness_test::entries_of_the_known_bad_images(&clean);
+    let names_here: Vec<&str> = entries.iter().map(|(name, _)| *name).collect();
+    let names_in_the_file = entry_names_in_the_original_test_file(&test_file_text);
+    for check in [
+        StopCheck::new(
+            "known_bad_test_file_sha256_is_the_copied_one",
+            KNOWN_BAD_TEST_FILE_SHA256,
+            &test_file_sha256,
+        ),
+        StopCheck::new(
+            "known_bad_entry_count_here",
+            KNOWN_BAD_ENTRY_COUNT,
+            entries.len(),
+        ),
+        StopCheck::new(
+            "known_bad_entry_count_in_the_original_file",
+            KNOWN_BAD_ENTRY_COUNT,
+            names_in_the_file.len(),
+        ),
+        StopCheck::new(
+            "known_bad_entry_names_equal_the_original_file_in_order",
+            &names_in_the_file,
+            &names_here,
+        ),
+    ] {
+        failed += usize::from(!check.holds());
+        lines.result(&format!(
+            "name=s4 check={} local={} crates={} holds={}",
+            check.name,
+            field_text(&check.local),
+            field_text(&check.crates),
+            check.holds()
+        ));
+    }
+    let clean_red = violated_invariant_names(&check_pool_image(&clean));
+    lines.result(&format!(
+        "name=known_bad_clean_image invariants_red={}",
+        field_text(&list_text(&clean_red))
+    ));
+    let mut states_text = String::new();
+    let mut patches_file: Vec<u8> = Vec::new();
+    let mut o2_text = String::new();
+    for (entry_index, (target, apply)) in entries.iter().enumerate() {
+        let mut bad = clean.clone();
+        apply(&mut bad);
+        let direct_red = violated_invariant_names(&check_pool_image(&bad));
+        let patches = sector_difference_as_patches(&clean, &bad);
+        let mut patched = clean.clone();
+        apply_patches(&mut patched, &patches);
+        let patched_red = violated_invariant_names(&check_pool_image(&patched));
+        let patched_digest = sha256_text(&memory_pool_digest(&patched));
+        let digest_equals_the_direct_one = patched_digest == sha256_text(&memory_pool_digest(&bad));
+        let sets_equal = direct_red == patched_red;
+        let target_is_red = patched_red.iter().any(|name| name == target);
+        failed += usize::from(!(sets_equal && digest_equals_the_direct_one && target_is_red));
+        states_text.push_str(&format!(
+            "{entry_index}\t{all_persisted}\t{clean_digest}\t{patched_digest}\t{}\n",
+            patch_list_text(&patches, &mut patches_file)
+        ));
+        o2_text.push_str(&format!(
+            "{entry_index}\t{target}\t{}\t{}\t{sets_equal}\t{target_is_red}\n",
+            list_text(&patched_red),
+            list_text(&direct_red)
+        ));
+        lines.result(&format!(
+            "name=known_bad_entry index={entry_index} target={target} o2={} patched_red={} direct_red={} sets_equal={sets_equal} digest_equals_the_direct_one={digest_equals_the_direct_one} target_is_red={target_is_red} patches={}",
+            if patched_red.is_empty() { "green" } else { "red" },
+            field_text(&list_text(&patched_red)),
+            field_text(&list_text(&direct_red)),
+            patches.len()
+        ));
+    }
+    std::fs::create_dir_all(output_directory)
+        .unwrap_or_else(|error| panic!("建 {}：{error}", output_directory.display()));
+    for (file_name, contents) in [
+        ("known-bad-states.tsv", states_text.into_bytes()),
+        ("known-bad-patches.bin", patches_file),
+        ("known-bad-o2.tsv", o2_text.into_bytes()),
+    ] {
+        let written = write_export_file(output_directory, file_name, &contents);
+        lines.result(&format!(
+            "name=file file={file_name} bytes={} sha256={}",
+            written.bytes, written.sha256
+        ));
+    }
+    failed
+}
+
+/// E170 的 `bad-states` 模式造的坏状态（P 行、N 行、盲测族）上的 O2：每一行读 `bad-states.tsv` 的落地向量与补丁，从中性格式重建底态、叠补丁，
+/// 底态与补丁之后的镜像摘要与 E170 写的两个摘要逐行比，再调 `check_pool_image`，判红的不变量与 E170 `bad-impl-verdicts.tsv` 里池级 checker 那一列逐行比（S3）。
+/// 交回（对不上的行数）。只读 `bad-states.tsv`、`bad-patches.bin`、`bad-impl-verdicts.tsv`，不读期望文件。
+fn e170_bad_rows_on_both_streams(
+    dump_directory: &Path,
+    output_directory: &Path,
+    lines: &mut ResultLines,
+) -> usize {
+    let mut failed = 0usize;
+    let mut o2_text = String::new();
+    for name in STREAM_NAMES {
+        let label = name.directory_name();
+        let stream_directory = dump_directory.join(label);
+        let neutral = read_neutral_stream(&stream_directory);
+        let patches_file = read_file(&stream_directory, "bad-patches.bin");
+        let state_rows = read_text_lines(&stream_directory, "bad-states.tsv");
+        let verdict_rows = read_text_lines(&stream_directory, "bad-impl-verdicts.tsv");
+        assert_eq!(
+            state_rows.len(),
+            verdict_rows.len(),
+            "bad-states.tsv 与 bad-impl-verdicts.tsv 同行数"
+        );
+        for (state_row, verdict_row) in state_rows.iter().zip(&verdict_rows) {
+            let state_fields: Vec<&str> = state_row.split('\t').collect();
+            assert_eq!(
+                state_fields.len(),
+                5,
+                "bad-states.tsv 一行五列：{state_row:?}"
+            );
+            let verdict_fields: Vec<&str> = verdict_row.split('\t').collect();
+            assert_eq!(
+                verdict_fields.len(),
+                10,
+                "bad-impl-verdicts.tsv 一行十列：{verdict_row:?}"
+            );
+            let row_id = state_fields[0];
+            let mut pool = rebuilt_memory_pool(&neutral, state_fields[1]);
+            let base_digest = sha256_text(&memory_pool_digest(&pool));
+            // 不打补丁的底态上的 O2（登记 5.6 坏镜像行的阴性：F、Mk、Mk⁺、Mk⁻ 各一张，必须不红）。
+            let base_red = violated_invariant_names(&check_pool_image(&pool));
+            apply_patches(
+                &mut pool,
+                &patches_of_a_patch_list_text(state_fields[4], &patches_file),
+            );
+            let patched_digest = sha256_text(&memory_pool_digest(&pool));
+            let o2_red = violated_invariant_names(&check_pool_image(&pool));
+            let o2_text_of_the_row = list_text(&o2_red);
+            let e170_pool_checker_column = verdict_fields[6];
+            let digests_equal = base_digest == state_fields[2] && patched_digest == state_fields[3];
+            let o2_equals_e170 = o2_text_of_the_row == e170_pool_checker_column;
+            failed += usize::from(!(digests_equal && o2_equals_e170));
+            o2_text.push_str(&format!(
+                "{label}\t{row_id}\t{}\t{o2_text_of_the_row}\t{e170_pool_checker_column}\t{o2_equals_e170}\t{digests_equal}\t{}\t{base_digest}\n",
+                if o2_red.is_empty() { "green" } else { "red" },
+                list_text(&base_red)
+            ));
+            lines.result(&format!(
+                "name=e170_bad_row stream={label} row={row_id} o2={} o2_red={} e170_pool_checker={} o2_equals_e170={o2_equals_e170} digests_equal={digests_equal} base_o2_red={}",
+                if o2_red.is_empty() { "green" } else { "red" },
+                field_text(&o2_text_of_the_row),
+                field_text(e170_pool_checker_column),
+                field_text(&list_text(&base_red))
+            ));
+        }
+    }
+    std::fs::create_dir_all(output_directory)
+        .unwrap_or_else(|error| panic!("建 {}：{error}", output_directory.display()));
+    let written = write_export_file(output_directory, "e170-bad-rows-o2.tsv", o2_text.as_bytes());
+    lines.result(&format!(
+        "name=file file=e170-bad-rows-o2.tsv bytes={} sha256={}",
+        written.bytes, written.sha256
+    ));
+    failed
+}
+
+/// 一条流的结果行（`E7RESULT` 与 `E7TIMING`）；交回 (S1 没过的条数, S4 对不上的态数, 掩码对不上的态数, S3 对不上的态数,
+/// O1-对拍判红的调用数, O1-崩溃判官判红的态数, O2 判红的态数)。
+fn report_stream(lines: &mut ResultLines, export: &StreamExport) -> [usize; 7] {
+    let label = export.name.directory_name();
+    let mut failed_checks = 0usize;
+    for check in &export.checks {
+        failed_checks += usize::from(!check.holds());
+        lines.result(&format!(
+            "name=s1 stream={label} check={} local={} crates={} holds={}",
+            check.name,
+            field_text(&check.local),
+            field_text(&check.crates),
+            check.holds()
+        ));
+    }
+    let ending_text = match &export.ending {
+        Some(StreamEnding::Completed) => "completed".to_string(),
+        Some(StreamEnding::StoppedByAnError { call_index, member }) => {
+            format!(
+                "stopped_by_an_error:call_{call_index}:{}",
+                field_text(member)
+            )
+        }
+        Some(StreamEnding::ImplementationCrashed { what }) => {
+            format!("implementation_crashed:{}", field_text(what))
+        }
+        None => "pipeline_panicked".to_string(),
+    };
+    lines.result(&format!(
+        "name=stream stream={label} ending={ending_text} exported={} not_exported_why={} operations={} writes={} segments={} tearable_writes={} calls={} full_states={} quick_tier_states={} sampled_states={} model_stopped_at_call={}",
+        export.not_exported.is_none(),
+        export
+            .not_exported
+            .as_deref()
+            .map_or_else(|| "-".to_string(), field_text),
+        export.operation_count,
+        export.write_count,
+        export.segment_count,
+        export.tearable_write_count,
+        export.calls.len(),
+        export.full_states,
+        export.quick_tier_states,
+        export.observations.len(),
+        export
+            .model_stopped_at_call
+            .map_or_else(|| "none".to_string(), |call| call.to_string())
+    ));
+    let mut red_calls = 0usize;
+    for (call_index, call) in export.calls.iter().enumerate() {
+        red_calls += usize::from(matches!(call.model, ModelVerdictOfCall::Disagrees { .. }));
+        lines.result(&format!(
+            "name=call stream={label} call={call_index} label={} first_operation={} return_point={} status={} o1_comparison={}",
+            call.label,
+            call.first_operation,
+            call.return_point,
+            field_text(&call.outcome.outcome_word()),
+            field_text(&call.model.word()),
+        ));
+    }
+    for scan in &export.torn_scans {
+        lines.result(&format!(
+            "name=torn_scan stream={label} segment={} scanned={} found={} wanted={TORN_STATES_PER_SEGMENT}",
+            scan.segment_index, scan.scanned, scan.found
+        ));
+    }
+    let mut crash_judge_red = 0usize;
+    let mut pool_checker_red = 0usize;
+    for batch in 1..=3u8 {
+        let in_batch: Vec<&StateObservation> = export
+            .observations
+            .iter()
+            .filter(|observation| observation.batch == batch)
+            .collect();
+        let count_where = |predicate: &dyn Fn(&StateObservation) -> bool| {
+            in_batch
+                .iter()
+                .filter(|observation| predicate(observation))
+                .count()
+        };
+        let crash_judge_is =
+            |word: &'static str| count_where(&|observation| observation.crash_judge.word() == word);
+        let o2_red = count_where(&|observation| {
+            observation
+                .pool_checker_red
+                .as_ref()
+                .is_some_and(|items| !items.is_empty())
+        });
+        crash_judge_red += crash_judge_is("red");
+        pool_checker_red += o2_red;
+        let segments: BTreeSet<usize> = in_batch
+            .iter()
+            .map(|observation| observation.segment_index)
+            .collect();
+        lines.result(&format!(
+            "name=batch stream={label} batch={batch} states={} segments_touched={} states_with_a_torn_write={} o1_crash_judge_red={} o1_crash_judge_green={} o1_crash_judge_not_applicable={} o1_crash_judge_off={} o2_red={o2_red} o2_matches_judgement_false={} judge_crash_image_pool_checker_red={} record_checker_red_states={} consulted_oracle_red_states={} ignored_oracle_red_states={} consulted_effective_root_none={} consulted_failed_outcomes={}",
+            in_batch.len(),
+            segments.len(),
+            count_where(&|observation| observation.landing_vector.contains('1')),
+            crash_judge_is("red"),
+            crash_judge_is("green"),
+            crash_judge_is("na"),
+            crash_judge_is("off"),
+            count_where(&|observation| !observation.pool_checker_matches_the_judgement),
+            count_where(&|observation| !observation.verdict.pool_checker_red.is_empty()),
+            count_where(&|observation| observation
+                .verdict
+                .red_items
+                .iter()
+                .any(|item| item.starts_with("record_checker:"))),
+            count_where(&|observation| observation
+                .verdict
+                .red_items
+                .iter()
+                .any(|item| item.starts_with("journal_consulted_oracle:"))),
+            count_where(&|observation| observation
+                .verdict
+                .red_items
+                .iter()
+                .any(|item| item.starts_with("journal_ignored_oracle:"))),
+            count_where(&|observation| observation.verdict.consulted_root == "none"),
+            count_where(&|observation| observation.verdict.consulted_outcome.starts_with("failed:")),
+        ));
+        for (stage, pick) in [
+            (
+                "o1",
+                (|observation: &StateObservation| observation.o1_nanoseconds)
+                    as fn(&StateObservation) -> u128,
+            ),
+            ("o2", |observation| observation.o2_nanoseconds),
+            ("recovery", |observation| observation.recovery_nanoseconds),
+        ] {
+            let (total, median, p95, maximum) = timing_summary(
+                in_batch
+                    .iter()
+                    .map(|observation| pick(observation))
+                    .collect(),
+            );
+            ResultLines::timing(&format!(
+                "name=stage_time stream={label} batch={batch} stage={stage} states={} total_ns={total} median_ns={median} p95_ns={p95} max_ns={maximum}",
+                in_batch.len()
+            ));
+        }
+    }
+    let mask_mismatches = export
+        .observations
+        .iter()
+        .filter(|observation| !observation.mask_matches_the_facts)
+        .count();
+    let s3_mismatches = export
+        .observations
+        .iter()
+        .filter(|observation| !observation.pool_checker_matches_the_judgement)
+        .count();
+    lines.result(&format!(
+        "name=s4 stream={label} rebuilt_states={} digest_mismatches={}",
+        export.rebuilt_states, export.rebuilt_digest_mismatches
+    ));
+    lines.result(&format!(
+        "name=crash_facts_mask stream={label} states={} mismatches={mask_mismatches}",
+        export.observations.len()
+    ));
+    lines.result(&format!(
+        "name=s3 stream={label} states={} o2_direct_differs_from_judge_crash_image={s3_mismatches}",
+        export.observations.len()
+    ));
+    for file in &export.files {
+        match &file.sha256_without_timing {
+            None => lines.result(&format!(
+                "name=file stream={label} file={} bytes={} sha256={}",
+                file.name, file.bytes, file.sha256
+            )),
+            Some(without_timing) => {
+                lines.result(&format!(
+                    "name=file stream={label} file={} sha256_without_timing={without_timing}",
+                    file.name
+                ));
+                ResultLines::timing(&format!(
+                    "name=file_with_timing stream={label} file={} bytes={} sha256={}",
+                    file.name, file.bytes, file.sha256
+                ));
+            }
+        }
+    }
+    ResultLines::timing(&format!(
+        "name=stream_time stream={label} elapsed_ns={} drive_ns={} observe_ns={} write_ns={}",
+        export.elapsed_nanoseconds,
+        export.drive_nanoseconds,
+        export.observe_nanoseconds,
+        export.write_nanoseconds
+    ));
+    [
+        failed_checks,
+        export.rebuilt_digest_mismatches,
+        mask_mismatches,
+        s3_mismatches,
+        red_calls,
+        crash_judge_red,
+        pool_checker_red,
+    ]
+}
+
+/// 基线上 O1 不适用的来源只有两种（登记第十一节 S5）：模型没有答的 `unmount`，与对拍判出不同之后。基线上不许有对拍判出不同，
+/// 所以模型只许停在流二的 `unmount` 那一次调用（流一一次都不停）。这两条按 `StopCheck` 的形式回比：期望（本地）与实际（这一趟）。
+fn s5_checks_of_the_baseline(export: &StreamExport) -> Vec<StopCheck> {
+    let label = export.name.directory_name();
+    let expected_stop = match export.name {
+        StreamName::NewPoolFileCreation => None,
+        StreamName::FixedScriptThroughEThenUnmount => export
+            .calls
+            .iter()
+            .rposition(|call| call.label == "unmount"),
+    };
+    vec![
+        StopCheck::new(
+            &format!("{label}_o1_model_stops_only_at_the_unmount"),
+            expected_stop,
+            export.model_stopped_at_call,
+        ),
+        StopCheck::new(
+            &format!("{label}_every_call_is_judged_or_is_the_unmount"),
+            true,
+            export.calls.iter().all(|call| {
+                matches!(
+                    call.model,
+                    ModelVerdictOfCall::Agrees | ModelVerdictOfCall::NotModelled
+                )
+            }),
+        ),
+    ]
+}
+
+struct CommandLine {
+    run_kind: RunKind,
+    output_directory: PathBuf,
+    switches: RunSwitches,
+}
+
+struct KnownBadImagesCommandLine {
+    dump_directory: PathBuf,
+    output_directory: PathBuf,
+    with_e170_bad_rows: bool,
+}
+
+enum ParsedCommand {
+    Dump(CommandLine),
+    KnownBadImages(KnownBadImagesCommandLine),
+}
+
+fn parse_command_line(arguments: &[String]) -> Option<ParsedCommand> {
+    let mut words = arguments.iter();
+    let mode = words.next()?.as_str();
+    let mut output_directory = None;
+    let mut dump_directory = None;
+    let mut with_e170_bad_rows = false;
+    let mut switches = RunSwitches::default();
+    while let Some(word) = words.next() {
+        match word.as_str() {
+            "--output-directory" => output_directory = Some(PathBuf::from(words.next()?)),
+            "--dump-directory" => dump_directory = Some(PathBuf::from(words.next()?)),
+            "--e170-bad-rows" => with_e170_bad_rows = true,
+            "--positive-control" => {
+                switches.positive_control = Some(PositiveControl::parse(words.next()?)?);
+            }
+            "--disable-oracle" => {
+                switches.disabled_oracle = Some(match words.next()?.as_str() {
+                    "O1" => DisabledOracle::ComparisonAndCrashJudge,
+                    "O2" => DisabledOracle::PoolChecker,
+                    _ => return None,
+                });
+            }
+            _ => return None,
+        }
+    }
+    match mode {
+        "baseline" | "row" => Some(ParsedCommand::Dump(CommandLine {
+            run_kind: if mode == "baseline" {
+                RunKind::Baseline
+            } else {
+                RunKind::BadImplementationRow
+            },
+            output_directory: output_directory?,
+            switches,
+        })),
+        "known-bad-images" => Some(ParsedCommand::KnownBadImages(KnownBadImagesCommandLine {
+            dump_directory: dump_directory?,
+            output_directory: output_directory?,
+            with_e170_bad_rows,
+        })),
+        _ => None,
+    }
+}
+
+fn run_known_bad_images(command_line: &KnownBadImagesCommandLine) -> i32 {
+    let mut lines = ResultLines { emitted: 0 };
+    lines.result(&format!(
+        "name=config segment=s1 mode=known-bad-images e170_bad_rows={}",
+        command_line.with_e170_bad_rows
+    ));
+    let known_bad_failed = known_bad_images_on_the_first_stream(
+        &command_line.dump_directory,
+        &command_line.output_directory,
+        &mut lines,
+    );
+    let e170_rows_failed = if command_line.with_e170_bad_rows {
+        e170_bad_rows_on_both_streams(
+            &command_line.dump_directory,
+            &command_line.output_directory,
+            &mut lines,
+        )
+    } else {
+        0
+    };
+    lines.result(&format!(
+        "name=verdict known_bad_entries_failing_s4={known_bad_failed} e170_bad_rows_failing_s3_or_s4={e170_rows_failed}"
+    ));
+    lines.finish();
+    if known_bad_failed > 0 || e170_rows_failed > 0 {
+        3
+    } else {
+        0
+    }
+}
+
+fn run_dump(command_line: &CommandLine) -> i32 {
+    let run_kind = command_line.run_kind;
+    let switches = command_line.switches;
+    let mut lines = ResultLines { emitted: 0 };
+    lines.result(&format!(
+        "name=config segment=s1 mode={} streams={} sector_bytes={LOCAL_SECTOR_BYTES} first_batch_divisor={FIRST_BATCH_DIVISOR} second_batch_divisor={SECOND_BATCH_DIVISOR} third_batch_divisor={THIRD_BATCH_DIVISOR} torn_states_per_segment={TORN_STATES_PER_SEGMENT} torn_scan_ordinals_per_segment={TORN_SCAN_ORDINALS_PER_SEGMENT} third_batch_sampled={} positive_control={} disabled_oracle={}",
+        if run_kind == RunKind::Baseline { "baseline" } else { "row" },
+        STREAM_NAMES.len(),
+        run_kind == RunKind::Baseline,
+        switches
+            .positive_control
+            .map_or_else(|| "none".to_string(), |control| format!("{control:?}")),
+        switches
+            .disabled_oracle
+            .map_or_else(|| "none".to_string(), |oracle| format!("{oracle:?}")),
+    ));
+    let mut failed_checks = 0usize;
+    if run_kind == RunKind::Baseline {
+        for check in stream_independent_checks() {
+            failed_checks += usize::from(!check.holds());
+            lines.result(&format!(
+                "name=s1 stream=both check={} local={} crates={} holds={}",
+                check.name,
+                field_text(&check.local),
+                field_text(&check.crates),
+                check.holds()
+            ));
+        }
+    }
+    let mut totals = [0usize; 7];
+    let mut streams_not_exported = 0usize;
+    let mut s5_failed = 0usize;
+    for name in STREAM_NAMES {
+        let started = Instant::now();
+        let output_root = command_line.output_directory.clone();
+        let exported = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+            export_stream(name, &output_root, run_kind, switches)
+        }));
+        let mut export = exported.unwrap_or_else(|payload| {
+            StreamExport::not_exported(
+                name,
+                format!("pipeline_panicked:{}", panic_text(&*payload)),
+                started,
+            )
+        });
+        streams_not_exported += usize::from(export.not_exported.is_some());
+        // S5 只判不带对照、不关 oracle 的基线：阳性对照会让模型在翻过的那次调用停下，关掉 O1 时每次调用都不比。
+        if run_kind == RunKind::Baseline
+            && export.not_exported.is_none()
+            && switches.positive_control.is_none()
+            && switches.disabled_oracle.is_none()
+        {
+            let s5 = s5_checks_of_the_baseline(&export);
+            s5_failed += s5.iter().filter(|check| !check.holds()).count();
+            export.checks.extend(s5);
+        }
+        let stream_totals = report_stream(&mut lines, &export);
+        for (total, value) in totals.iter_mut().zip(stream_totals) {
+            *total += value;
+        }
+    }
+    failed_checks += totals[0];
+    let [_, rebuilt_digest_mismatches, mask_mismatches, s3_mismatches, red_calls, crash_judge_red, pool_checker_red] =
+        totals;
+    lines.result(&format!(
+        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={rebuilt_digest_mismatches} crash_facts_mask_mismatches={mask_mismatches} s3_o2_judgement_mismatches={s3_mismatches} streams_not_exported={streams_not_exported} o1_comparison_red_calls={red_calls} o1_crash_judge_red_states={crash_judge_red} o2_red_states={pool_checker_red}"
+    ));
+    lines.finish();
+    let stop_clause_hit = failed_checks > 0
+        || rebuilt_digest_mismatches > 0
+        || s3_mismatches > 0
+        || (run_kind == RunKind::Baseline && (s5_failed > 0 || streams_not_exported > 0));
+    if stop_clause_hit {
+        3
+    } else {
+        0
+    }
+}
+
+fn main() {
+    let arguments: Vec<String> = std::env::args().skip(1).collect();
+    let Some(parsed) = parse_command_line(&arguments) else {
+        eprintln!(
+            "用法：e171_three_oracle_collusion baseline|row --output-directory <目录> [--positive-control O1-对拍|O1-崩溃判官|O2 | --disable-oracle O1|O2]，或 known-bad-images --dump-directory <基线导出目录> --output-directory <目录> [--e170-bad-rows]；收到 {arguments:?}"
+        );
+        std::process::exit(2);
+    };
+    let output_directory = match &parsed {
+        ParsedCommand::Dump(command_line) => &command_line.output_directory,
+        ParsedCommand::KnownBadImages(command_line) => &command_line.output_directory,
+    };
+    if output_directory.exists() {
+        eprintln!(
+            "输出目录 {} 已存在：装置不覆盖已有的文件，换一个目录",
+            output_directory.display()
+        );
+        std::process::exit(2);
+    }
+    let exit_code = match &parsed {
+        ParsedCommand::Dump(command_line) => run_dump(command_line),
+        ParsedCommand::KnownBadImages(command_line) => run_known_bad_images(command_line),
+    };
+    if exit_code != 0 {
+        std::process::exit(exit_code);
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    /// 空消息的 sha256（`python3 -c "import hashlib; print(hashlib.sha256(b'').hexdigest())"` 现算）。
+    const EMPTY_MESSAGE_SHA256: &str =
+        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
+    /// 盘 1、扇区 2、512 个 0xAB 那一条的摘要（`struct.pack('<IQ', 1, 2) + b'\xab' * 512` 喂 hashlib 现算）。
+    const ONE_SECTOR_OF_0XAB_SHA256: &str =
+        "c71cae1b3fc678e4cabfa21d575c728edd44a4b372b3389ce763856273abecbf";
+
+    #[test]
+    fn operation_kind_name_covers_every_recorded_operation_kind() {
+        assert_eq!(operation_kind_name(RecordedOperationKind::Write), "write");
+        assert_eq!(
+            operation_kind_name(RecordedOperationKind::WriteForceUnitAccess),
+            "write_fua"
+        );
+        assert_eq!(
+            operation_kind_name(RecordedOperationKind::WriteZeroes),
+            "write_zeroes"
+        );
+        assert_eq!(
+            operation_kind_name(RecordedOperationKind::Barrier),
+            "barrier"
+        );
+    }
+
+    #[test]
+    fn a_torn_landing_folds_to_one_and_a_persisted_write_to_two() {
+        // 写表：0 持久、1 没持久、2 原写没持久而撕裂镜像（枚举写表下标 3）持久。
+        let persisted = vec![true, false, false, true];
+        let torn_image_index_by_write = BTreeMap::from([(2usize, 3usize)]);
+        assert_eq!(
+            landing_vector_of(&persisted, 3, &torn_image_index_by_write),
+            "201",
+            "撕裂那一态折回 1，不是 2"
+        );
+        let persisted_with_the_torn_image_absent = vec![true, false, false, false];
+        assert_eq!(
+            landing_vector_of(
+                &persisted_with_the_torn_image_absent,
+                3,
+                &torn_image_index_by_write
+            ),
+            "200"
+        );
+    }
+
+    #[test]
+    fn an_all_zero_sector_does_not_enter_the_image_digest() {
+        let mut message = Vec::new();
+        append_sector_to_digest_message(&mut message, 0, 7, &[0u8; 512]);
+        assert!(message.is_empty(), "全零扇区不进摘要");
+        assert_eq!(sha256_text(&sha256_digest(&message)), EMPTY_MESSAGE_SHA256);
+        append_sector_to_digest_message(&mut message, 1, 2, &[0xAB; 512]);
+        assert_eq!(message.len(), 4 + 8 + 512);
+        assert_eq!(
+            sha256_text(&sha256_digest(&message)),
+            ONE_SECTOR_OF_0XAB_SHA256
+        );
+    }
+
+    #[test]
+    fn batch_points_follow_the_registered_formulas_on_a_hundred_state_range() {
+        let state_range = 10u64..110;
+        assert_eq!(
+            first_batch_points(&state_range)
+                .into_iter()
+                .collect::<BTreeSet<u64>>(),
+            BTreeSet::from([10, 24, 38, 52, 66, 80, 94, 109])
+        );
+        assert_eq!(
+            second_batch_points(&state_range),
+            vec![17, 31, 45, 59, 73, 87, 101]
+        );
+        let third = third_batch_points(&state_range);
+        assert_eq!(third.len(), 24);
+        assert_eq!(third[0], 10);
+        assert_eq!(third[1], 14, "10 + ⌊99/23⌋");
+        assert_eq!(third[23], 109);
+    }
+
+    #[test]
+    fn batch_points_on_short_ranges_collapse_onto_the_first_batch() {
+        let single = 5u64..6;
+        assert_eq!(
+            first_batch_points(&single)
+                .into_iter()
+                .collect::<BTreeSet<u64>>(),
+            BTreeSet::from([5])
+        );
+        let eight = 0u64..8;
+        assert_eq!(
+            first_batch_points(&eight)
+                .into_iter()
+                .collect::<BTreeSet<u64>>(),
+            (0..8).collect::<BTreeSet<u64>>()
+        );
+        assert_eq!(second_batch_points(&eight), vec![0, 1, 2, 3, 4, 5, 6]);
+    }
+
+    #[test]
+    fn sampled_batches_are_disjoint_and_include_the_last_state_and_up_to_four_torn_states() {
+        // 两段：[0, 8) 带撕裂写，序号为奇数的状态带 1；[8, 108) 不带。总数 109，最后那个是 108。
+        let ranges = vec![0u64..8, 8..108];
+        let landing = |ordinal: u64| {
+            if ordinal % 2 == 1 {
+                "21".to_string()
+            } else {
+                "20".to_string()
+            }
+        };
+        let sampled = sample_ordinals(&ranges, 109, &[true, false], &landing);
+        let [first, second, third] = &sampled.batches;
+        assert!(first.contains(&108), "最后那个全持久状态进批 1");
+        assert!(first.is_disjoint(second) && first.is_disjoint(third) && second.is_disjoint(third));
+        assert_eq!(
+            sampled.torn_scans,
+            vec![TornScan {
+                segment_index: 0,
+                scanned: 8,
+                found: 0
+            }],
+            "[0, 8) 的点批 1 已经全取了，扫完 8 个一个新的都没有"
+        );
+    }
+
+    #[test]
+    fn torn_scan_stops_after_four_new_states() {
+        let ranges: Vec<std::ops::Range<u64>> = std::iter::once(0u64..1000).collect();
+        let landing = |ordinal: u64| {
+            if ordinal % 3 == 2 {
+                "1".to_string()
+            } else {
+                "0".to_string()
+            }
+        };
+        let sampled = sample_ordinals(&ranges, 1001, &[true], &landing);
+        // 批 1 的格点 0、142、285、428、570、713、856、999；带 1 的是 2、5、8、11（都不在格点上）。
+        assert_eq!(sampled.torn_scans[0].found, 4);
+        assert_eq!(sampled.torn_scans[0].scanned, 12);
+        for ordinal in [2u64, 5, 8, 11] {
+            assert!(sampled.batches[0].contains(&ordinal));
+        }
+    }
+
+    /// 两条流本地段长表的闭式 1 + Σ(2^n − 1) 与写数，与本地抄来的闭式、写数逐个相等（改段长表里任何一个数，这里都红）。
+    #[test]
+    fn local_segment_tables_reproduce_the_closed_forms_and_write_counts_copied_from_the_tests() {
+        let closed_form = |lengths: &[usize]| -> u64 {
+            1 + lengths
+                .iter()
+                .map(|length| (1u64 << length) - 1)
+                .sum::<u64>()
+        };
+        assert_eq!(
+            closed_form(&LOCAL_FIRST_STREAM_SEGMENT_LENGTHS),
+            LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM
+        );
+        assert_eq!(
+            LOCAL_FIRST_STREAM_SEGMENT_LENGTHS.iter().sum::<usize>(),
+            LOCAL_FIRST_STREAM_WRITES
+        );
+        assert_eq!(
+            closed_form(&LOCAL_SECOND_STREAM_SEGMENT_LENGTHS),
+            LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM
+        );
+        assert_eq!(
+            LOCAL_SECOND_STREAM_SEGMENT_LENGTHS.iter().sum::<usize>(),
+            LOCAL_SECOND_STREAM_WRITES
+        );
+        assert_eq!(LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM, 1_662_648_449);
+        assert_eq!(LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM, 16_777_240);
+    }
+
+    fn bytes_write(device: u32, offset: u64, bytes: Vec<u8>) -> RetainedWrite {
+        RetainedWrite {
+            device: DeviceIdentity(device),
+            kind: StepKind::SystemConfigurationSlot,
+            is_force_unit_access: false,
+            offset: DeviceOffsetInBytes(offset),
+            contents: WrittenContents::Bytes(bytes),
+        }
+    }
+
+    fn neutral(
+        device: u32,
+        kind: RecordedOperationKind,
+        offset: u64,
+        length: u64,
+        payload_start: Option<usize>,
+    ) -> NeutralOperation {
+        NeutralOperation {
+            device,
+            kind,
+            offset,
+            length,
+            payload_start,
+        }
+    }
+
+    /// 小池上同一个状态两条路算摘要：`CrashImage` 直接读（带撕裂镜像的枚举写表）与中性格式按落地向量重建，两份逐字节相同；
+    /// 落地向量按写序号是「2 2 0 1」。
+    #[test]
+    fn rebuilding_from_the_neutral_format_matches_the_crash_image_digest_on_a_small_pool() {
+        let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], 1 << 20);
+        base.apply_writes(&[
+            bytes_write(0, 0, vec![0x11; 512]),
+            bytes_write(1, 8192, vec![0x22; 1024]),
+        ]);
+        let written_at_4096 = vec![0x33; 1024];
+        let zeroes_over_the_base_sector = RetainedWrite {
+            device: DeviceIdentity(0),
+            kind: StepKind::ZeroFill,
+            is_force_unit_access: false,
+            offset: DeviceOffsetInBytes(0),
+            contents: WrittenContents::Zeros { length: 512 },
+        };
+        let not_persisted = vec![0x44; 512];
+        let overwrite_new = vec![0x55; 1024];
+        let torn = [vec![0x55; 512], vec![0x22; 512]].concat();
+        let writes = vec![
+            bytes_write(0, 4096, written_at_4096.clone()),
+            zeroes_over_the_base_sector,
+            bytes_write(1, 0, not_persisted.clone()),
+            bytes_write(1, 8192, overwrite_new.clone()),
+            bytes_write(1, 8192, torn.clone()),
+        ];
+        let image = CrashImage {
+            base: &base,
+            writes: &writes,
+            persisted: vec![true, true, false, false, true],
+        };
+        let landing = landing_vector_of(&image.persisted, 4, &BTreeMap::from([(3usize, 4usize)]));
+        assert_eq!(landing, "2201");
+        let payload = [written_at_4096, not_persisted, overwrite_new].concat();
+        let stream = NeutralStream {
+            base_sectors: BTreeMap::from([
+                ((0u32, 0u64), vec![0x11; 512]),
+                ((1, 16), vec![0x22; 512]),
+                ((1, 17), vec![0x22; 512]),
+            ]),
+            operations: vec![
+                neutral(0, RecordedOperationKind::Write, 4096, 1024, Some(0)),
+                neutral(0, RecordedOperationKind::Barrier, 0, 0, None),
+                neutral(0, RecordedOperationKind::WriteZeroes, 0, 512, None),
+                neutral(1, RecordedOperationKind::Write, 0, 512, Some(1024)),
+                neutral(
+                    1,
+                    RecordedOperationKind::WriteForceUnitAccess,
+                    8192,
+                    1024,
+                    Some(1536),
+                ),
+            ],
+            payload,
+            torn_bytes_by_write: BTreeMap::from([(3usize, (0usize, 1024usize))]),
+            torn_payload: torn,
+        };
+        let from_the_image = crash_image_digest(&image);
+        assert_eq!(rebuilt_image_digest(&stream, &landing), from_the_image);
+        assert_ne!(
+            rebuilt_image_digest(&stream, "2202"),
+            from_the_image,
+            "撕裂那一态记成持久，重建出来的是新字节，摘要必须不同"
+        );
+    }
+
+    #[test]
+    fn contents_without_timing_drops_only_the_timing_column_and_the_verdicts_digest_line() {
+        assert_eq!(
+            contents_without_timing(
+                IMPLEMENTATION_VERDICTS_FILE,
+                b"0\ta\tb\t123\n1\tc\td\t4567\n"
+            ),
+            Some(b"0\ta\tb\n1\tc\td\n".to_vec())
+        );
+        assert_eq!(
+            contents_without_timing(
+                MANIFEST_FILE,
+                b"stream\tx\nsha256:states.tsv\taa\nsha256:impl-verdicts.tsv\tbb\n"
+            ),
+            Some(b"stream\tx\nsha256:states.tsv\taa\n".to_vec())
+        );
+        assert_eq!(contents_without_timing(STATES_FILE, b"0\t1\n"), None);
+    }
+
+    #[test]
+    fn timing_summary_takes_the_lower_middle_and_the_ninety_fifth_percentile() {
+        assert_eq!(timing_summary(vec![]), (0, 0, 0, 0));
+        assert_eq!(timing_summary(vec![4, 1, 3, 2]), (10, 2, 4, 4));
+        let hundred: Vec<u128> = (1..=100).collect();
+        assert_eq!(timing_summary(hundred), (5050, 50, 95, 100));
+    }
+
+    #[test]
+    fn merged_sector_ranges_join_overlapping_and_touching_ranges_only() {
+        assert_eq!(
+            merged_sector_ranges(vec![(10, 12), (0, 2), (2, 3), (11, 15), (20, 21)]),
+            vec![(0, 3), (10, 15), (20, 21)]
+        );
+    }
+
+    /// 两块盘上各改一个扇区的两份镜像：补丁要两块盘都带上，叠回干净镜像之后摘要与改坏的那份相同（X15：补丁提取只取盘 0 的差，这里红）。
+    #[test]
+    fn the_sector_difference_between_two_images_carries_the_changed_sectors_of_every_device() {
+        let devices = [DeviceIdentity(0), DeviceIdentity(1)];
+        let mut clean = MemoryPool::with_devices(&devices, 1 << 20);
+        clean.apply_writes(&[bytes_write(0, 0, vec![0x11; 512])]);
+        let mut bad = clean.clone();
+        bad.apply_writes(&[
+            bytes_write(0, 512, vec![0x44; 512]),
+            bytes_write(1, 1024, vec![0x55; 1024]),
+        ]);
+        let patches = sector_difference_as_patches(&clean, &bad);
+        assert_eq!(
+            patches,
+            vec![
+                BytePatch {
+                    device: 0,
+                    offset: 512,
+                    bytes: vec![0x44; 512]
+                },
+                BytePatch {
+                    device: 1,
+                    offset: 1024,
+                    bytes: vec![0x55; 1024]
+                },
+            ],
+            "盘 1 上相邻的两个扇区并成一段补丁"
+        );
+        let mut patched = clean.clone();
+        apply_patches(&mut patched, &patches);
+        assert_eq!(
+            sha256_text(&memory_pool_digest(&patched)),
+            sha256_text(&memory_pool_digest(&bad))
+        );
+        assert_ne!(
+            sha256_text(&memory_pool_digest(&clean)),
+            sha256_text(&memory_pool_digest(&bad)),
+            "改坏之前与之后摘要不同"
+        );
+    }
+
+    /// E170 的 P 行只翻一个字节：补丁不按扇区对齐，施加时读出罩住它的整扇区、只换那一个字节、再整扇区写回。
+    #[test]
+    fn a_one_byte_patch_inside_a_sector_changes_only_that_byte() {
+        let devices = [DeviceIdentity(0), DeviceIdentity(1)];
+        let mut pool = MemoryPool::with_devices(&devices, 1 << 20);
+        pool.apply_writes(&[bytes_write(1, 512, vec![0x11; 1024])]);
+        apply_patches(
+            &mut pool,
+            &[BytePatch {
+                device: 1,
+                offset: 700,
+                bytes: vec![0xEE],
+            }],
+        );
+        let read_back = pool.devices[&DeviceIdentity(1)].read(DeviceOffsetInBytes(512), 1024);
+        assert_eq!(read_back[700 - 512], 0xEE);
+        assert!(read_back
+            .iter()
+            .enumerate()
+            .all(|(index, byte)| index == 700 - 512 || *byte == 0x11));
+    }
+
+    /// 池级 checker 的判定里只有 `Violated` 算 O2 红（X6：`Violated` 当 `Holds`，这里红）。
+    #[test]
+    fn only_a_violated_invariant_counts_as_a_red_pool_checker_verdict() {
+        let verdicts: Vec<(&'static str, InvariantVerdict)> = vec![
+            ("I-1.1", InvariantVerdict::Violated("坏".to_string())),
+            ("I-1.2", InvariantVerdict::Holds),
+            ("I-1.3", InvariantVerdict::NotApplicable("没有对象")),
+        ];
+        assert_eq!(
+            violated_invariant_names(&verdicts),
+            vec!["I-1.1".to_string()]
+        );
+    }
+
+    /// 模型不认的版本读回来记红、认的记绿（X4：崩溃判官的结果丢掉，这里红）。
+    #[test]
+    fn a_read_back_of_a_version_the_model_never_committed_is_red_and_a_committed_one_is_green() {
+        let committed_key = model_root_key(InstanceGeneration(1), CheckpointTxg(3));
+        let committed = BTreeMap::from([(committed_key, Some(Rc::<[u8]>::from(vec![7u8, 8, 9])))]);
+        let green = ObservedReadBack::FileRead {
+            root: committed_key,
+            content: vec![7, 8, 9],
+        };
+        assert_eq!(
+            crash_judge_verdict(&committed, Some(committed_key), &green),
+            CrashJudgeVerdict::Green
+        );
+        let flipped_last_byte = ObservedReadBack::FileRead {
+            root: committed_key,
+            content: vec![7, 8, 8],
+        };
+        assert!(matches!(
+            crash_judge_verdict(&committed, Some(committed_key), &flipped_last_byte),
+            CrashJudgeVerdict::Red { .. }
+        ));
+        let never_committed = ObservedReadBack::NoFile {
+            root: model_root_key(InstanceGeneration(9), CheckpointTxg(99)),
+        };
+        assert!(matches!(
+            crash_judge_verdict(&committed, None, &never_committed),
+            CrashJudgeVerdict::Red { .. }
+        ));
+    }
+
+    /// 模型停在第 3 次调用：第 2 次及之前的调用里写的态适用，第 3 次及之后的不适用；一个写都没落的态与模型没停的流都适用
+    /// （X14：流二每一态都记不适用，这里红）。
+    #[test]
+    fn the_crash_judge_is_not_applicable_only_from_the_call_where_the_model_stopped() {
+        assert!(crash_judge_is_applicable(Some(2), Some(3)));
+        assert!(!crash_judge_is_applicable(Some(3), Some(3)));
+        assert!(!crash_judge_is_applicable(Some(5), Some(3)));
+        assert!(crash_judge_is_applicable(None, Some(0)));
+        assert!(crash_judge_is_applicable(Some(5), None));
+    }
+
+    /// 模型没有会话时问发布第一个文件：答案本身就是一格对不上，驱动把它记成 `Disagrees`、模型停在这次调用，下一次调用不比
+    /// （X5：`judge_and_advance` 的不同丢掉，这里红）。
+    #[test]
+    fn a_comparison_the_model_cannot_answer_is_recorded_as_a_disagreement_and_stops_the_model() {
+        let mut driver = StreamDriver::new(
+            StreamName::NewPoolFileCreation,
+            SharedStream::retaining_contents(),
+            0,
+            RunSwitches::default(),
+        );
+        let refused: Result<(), &str> = Err("拒");
+        assert!(driver.call("publish", || refused).is_err());
+        driver.judge(
+            |model| model.answer_publish_first_file(b"x"),
+            ObservedOutcome::Succeeded(ObservedEffect::Publishes {
+                roots: Vec::new(),
+                reported_ceiling: None,
+            }),
+        );
+        assert!(matches!(
+            driver.calls[0].model,
+            ModelVerdictOfCall::Disagrees { .. }
+        ));
+        assert_eq!(driver.model_stopped_at_call, Some(0));
+        let accepted: Result<(), &str> = Ok(());
+        assert!(driver.call("publish", || accepted).is_ok());
+        driver.judge(
+            |model| model.answer_publish_first_file(b"x"),
+            ObservedOutcome::Succeeded(ObservedEffect::Publishes {
+                roots: Vec::new(),
+                reported_ceiling: None,
+            }),
+        );
+        assert_eq!(
+            driver.calls[1].model,
+            ModelVerdictOfCall::SkippedAfterTheModelStopped
+        );
+    }
+
+    /// 模型这一步要求成功、实现却交回一个说不出理由的错误：`judge_and_advance` 自己判出一格对不上，驱动记成 `Disagrees` 并让模型停在这次调用
+    /// （X5：`judge_and_advance` 的不同丢掉，这里红；上一条测的是模型连答案都给不出的那一路）。
+    #[test]
+    fn an_implementation_refusal_the_model_does_not_allow_is_recorded_as_a_disagreement() {
+        let mut driver = StreamDriver::new(
+            StreamName::NewPoolFileCreation,
+            SharedStream::retaining_contents(),
+            0,
+            RunSwitches::default(),
+        );
+        driver.advance_the_model_past_the_acquire_and_the_warm_up();
+        let refused: Result<(), &str> = Err("拒");
+        assert!(driver.call("publish", || refused).is_err());
+        let observed = driver.refusal(
+            &"拒",
+            singlefs_harness::model::ObservedRefusalReason::Unexplained,
+            0,
+            0,
+            None,
+            None,
+        );
+        driver.judge(|model| model.answer_publish_first_file(b"x"), observed);
+        assert!(
+            matches!(driver.calls[0].model, ModelVerdictOfCall::Disagrees { .. }),
+            "模型要求成功、实现拒了：{:?}",
+            driver.calls[0].model
+        );
+        assert_eq!(driver.model_stopped_at_call, Some(0));
+    }
+
+    /// 原用例文件里一个函数的条目名按文件次序取：字面名与常量名都认，更深缩进的行不认。
+    #[test]
+    fn entry_names_are_read_in_file_order_from_the_twelve_space_indented_lines_of_the_function() {
+        let text = "fn known_bad_images(clean: &MemoryPool) -> Vec<(&'static str, Mutation)> {\n    vec![\n        (\n            \"I-7.4\",\n            Box::new(|image| {\n                let x = 1;\n                \"I-9.9\";\n            }),\n        ),\n        (\n            MAPPING_KEY_INVARIANT,\n            Box::new(|image| {}),\n        ),\n    ]\n}\nfn other() {\n            \"I-0.0\",\n}\n";
+        assert_eq!(
+            entry_names_in_the_original_test_file(text),
+            vec![
+                "I-7.4".to_string(),
+                singlefs_checker::image::MAPPING_KEY_MATCHES_THE_UNIT_HEADER.to_string()
+            ]
+        );
+    }
+
+    #[test]
+    fn an_error_member_is_the_first_identifier_of_its_debug_text() {
+        #[derive(Debug)]
+        #[allow(
+            dead_code,
+            reason = "字段只经 Debug 文本读，测的就是 Debug 文本的第一个标识符"
+        )]
+        enum Failure {
+            Plain,
+            WithFields { reason: u8 },
+            Tuple(u8),
+        }
+        assert_eq!(error_member_of(&Failure::Plain), "Plain");
+        assert_eq!(
+            error_member_of(&Failure::WithFields { reason: 1 }),
+            "WithFields"
+        );
+        assert_eq!(error_member_of(&Failure::Tuple(2)), "Tuple");
+    }
+
+    #[test]
+    fn a_patch_list_text_round_trips_through_the_patches_file() {
+        let patches = vec![
+            BytePatch {
+                device: 1,
+                offset: 4096,
+                bytes: vec![1, 2, 3],
+            },
+            BytePatch {
+                device: 0,
+                offset: 0,
+                bytes: vec![9; 5],
+            },
+        ];
+        let mut patches_file = Vec::new();
+        let list = patch_list_text(&patches, &mut patches_file);
+        assert_eq!(list, "1:4096:0:3,0:0:3:5");
+        assert_eq!(patches_of_a_patch_list_text(&list, &patches_file), patches);
+        assert_eq!(patch_list_text(&[], &mut Vec::new()), "-");
+    }
+
+    #[test]
+    fn a_call_outcome_writes_ok_or_err_into_the_calls_file_and_a_word_into_the_outcomes_file() {
+        assert_eq!(CallOutcome::Succeeded.calls_file_word(), "ok");
+        assert_eq!(
+            CallOutcome::ReturnedError("NoSpace".to_string()).calls_file_word(),
+            "err"
+        );
+        assert_eq!(
+            CallOutcome::ReturnedError("NoSpace".to_string()).outcome_word(),
+            "err:NoSpace"
+        );
+        assert_eq!(
+            CallOutcome::Panicked("炸".to_string()).outcome_word(),
+            "panic"
+        );
+    }
+}
````

## `crates/singlefs-checker-tier/src/crash_injection.rs`

````diff
--- crates/singlefs-checker-tier/src/crash_injection.rs	2026-10-02 14:44:16.997207352 +0000
+++ crates/singlefs-checker-tier/src/crash_injection.rs	2026-10-02 18:08:52.381489166 +0000
@@ -74,13 +74,17 @@
 };
 use singlefs_harness::mount_transparency::{
     judge_mount_transparency, judge_writable_mount_refusal, read_recovery_fields, ComparedVersion,
-    DiskAfterTheRefusal, FieldOutcome, FloorRaisedByTheMount, FloorRaisesAtTheSecondCrash,
+    DiskAfterTheRefusal, FieldOutcome, FloorRaisedByTheMount, FloorRaisesInTheMountWrites,
     MountTransparency, MountTransparencyAction, ReadOnlyReadBackOfTheCrashImage,
     RecoveryFieldsRead, WritableMountRefusalVerdict,
 };
 use singlefs_harness::numeric_environment_variable::{
     number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
 };
+use singlefs_harness::reference_filesystem_translation::{
+    judge_a_second_crash_after_the_restart, JointVerdictOfTheReferenceFileSystemAndTheModel,
+    ReferenceFileSystemJudgementOfACrash, ReferenceFileSystemTranslation, ReferenceFileSystemVote,
+};
 use singlefs_harness::scenario::FIXED_WRITE_TIME_SECONDS;
 use singlefs_harness::segments::{FixedGeometry, StepKind};
 use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};
@@ -267,7 +271,8 @@
     Acquisition,
     /// 写行那次发布：到它的根槽写、连同紧跟着的系统配置槽轮换为止。
     RowPublish,
-    /// 写行那次之后的暖机空发布（与挂载在暖机之后推的抬 F）。
+    /// 写行那次之后的暖机空发布（与挂载在暖机之后推的抬 F：摆二次崩溃时两者算一段，F 那一样按抬 F 的写有没有持久分开判，
+    /// [`FloorRaisesInTheMountWrites::at_a_second_crash`]）。
     WarmUp,
 }
 
@@ -406,6 +411,13 @@
     }
 }
 
+/// RefFS 判的两截（G1 用户定案的调用点：第一截与二次崩溃那一截；挂载加一次发布之后那一截不判）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum StageTheReferenceFileSystemJudges {
+    CrashImage,
+    SecondCrashInsideTheWritableMount,
+}
+
 /// 崩溃状态上以「已知红」收尾的一次：种子、清单第几条、哪个崩溃状态、哪一截。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct KnownRedAtACrashPoint {
@@ -500,6 +512,13 @@
     /// 崩溃后镜像上的可写挂载被拒、理由在 D23（journal 的角色与格式） 已定项 14 那张单子上的：拒得合法的、不合法的（判红）。
     pub writable_mount_refusals_on_the_d23_list_legal: u64,
     pub writable_mount_refusals_on_the_d23_list_illegal: u64,
+    /// 第一截与二次崩溃那一截上 RefFS 与 O1 判官两票合起来（G1 用户定案：任一红即红），按哪一边红分开计；
+    /// RefFS 弃权的按理由另计（两截并在一起）。
+    pub reference_filesystem_and_model_verdicts_on_crash_images:
+        BTreeMap<JointVerdictOfTheReferenceFileSystemAndTheModel, u64>,
+    pub reference_filesystem_and_model_verdicts_on_second_crashes:
+        BTreeMap<JointVerdictOfTheReferenceFileSystemAndTheModel, u64>,
+    pub reference_filesystem_abstentions: BTreeMap<&'static str, u64>,
 }
 
 impl CrashInjectionTally {
@@ -563,6 +582,9 @@
             mount_transparency_fields_not_judged,
             writable_mount_refusals_on_the_d23_list_legal,
             writable_mount_refusals_on_the_d23_list_illegal,
+            reference_filesystem_and_model_verdicts_on_crash_images,
+            reference_filesystem_and_model_verdicts_on_second_crashes,
+            reference_filesystem_abstentions,
         } = following;
         self.histories += histories;
         self.histories_completed += histories_completed;
@@ -687,6 +709,49 @@
             writable_mount_refusals_on_the_d23_list_legal;
         self.writable_mount_refusals_on_the_d23_list_illegal +=
             writable_mount_refusals_on_the_d23_list_illegal;
+        for (verdict, count) in reference_filesystem_and_model_verdicts_on_crash_images {
+            *self
+                .reference_filesystem_and_model_verdicts_on_crash_images
+                .entry(*verdict)
+                .or_insert(0) += count;
+        }
+        for (verdict, count) in reference_filesystem_and_model_verdicts_on_second_crashes {
+            *self
+                .reference_filesystem_and_model_verdicts_on_second_crashes
+                .entry(*verdict)
+                .or_insert(0) += count;
+        }
+        for (reason, count) in reference_filesystem_abstentions {
+            *self
+                .reference_filesystem_abstentions
+                .entry(reason)
+                .or_insert(0) += count;
+        }
+    }
+
+    /// 记一票：RefFS 与 O1 判官两票合起来的那一格，RefFS 弃权时连理由。
+    fn note_the_reference_filesystem_vote(
+        &mut self,
+        stage: StageTheReferenceFileSystemJudges,
+        vote: &ReferenceFileSystemVote,
+        model_disagreement: Option<&ModelDisagreement>,
+    ) {
+        let joint = JointVerdictOfTheReferenceFileSystemAndTheModel::of(vote, model_disagreement);
+        let verdicts = match stage {
+            StageTheReferenceFileSystemJudges::CrashImage => {
+                &mut self.reference_filesystem_and_model_verdicts_on_crash_images
+            }
+            StageTheReferenceFileSystemJudges::SecondCrashInsideTheWritableMount => {
+                &mut self.reference_filesystem_and_model_verdicts_on_second_crashes
+            }
+        };
+        *verdicts.entry(joint).or_insert(0) += 1;
+        if let ReferenceFileSystemVote::Abstained(reason) = vote {
+            *self
+                .reference_filesystem_abstentions
+                .entry(reason.name())
+                .or_insert(0) += 1;
+        }
     }
 
     /// 给人看的一整块：绝对数——摆了几个崩溃状态、恢复成了几次、checker 与记录核对器跑了几次、模型判了几次、新发现几条。
@@ -818,6 +883,19 @@
         );
         let _ = writeln!(
             text,
+            "RefFS 与 O1 判官两票（任一红即红）：第一截 {:?}；二次崩溃 {:?}；RefFS 弃权的理由 {:?}",
+            self.reference_filesystem_and_model_verdicts_on_crash_images
+                .iter()
+                .map(|(verdict, count)| (verdict.name(), *count))
+                .collect::<Vec<_>>(),
+            self.reference_filesystem_and_model_verdicts_on_second_crashes
+                .iter()
+                .map(|(verdict, count)| (verdict.name(), *count))
+                .collect::<Vec<_>>(),
+            self.reference_filesystem_abstentions
+        );
+        let _ = writeln!(
+            text,
             "崩溃状态：以已知红收尾 {:?}、新发现 {}",
             self.crash_states_ending_known_red, self.crash_states_ending_new_finding
         );
@@ -889,6 +967,9 @@
         after_the_last_finished_step: 0,
         after_each_step: Vec::new(),
     };
+    // RefFS 的输入（G1 用户定案：第一截与二次崩溃那一截的判定 = RefFS ∧ O1 判官全四条）：跑历史时逐步攒。
+    let mut reference_filesystem_translation =
+        ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
     let run = execute_history_with(history, execution, &stream, &mut |observation| {
         for (key, file) in observation.model.committed_versions() {
             committed_versions.insert(key, file);
@@ -898,6 +979,7 @@
         marks
             .after_each_step
             .push((observation.position, operations_so_far));
+        reference_filesystem_translation.note_an_observed_step(observation, operations_so_far);
     });
     marks.after_make_filesystem = run.operations_written_by_make_filesystem;
 
@@ -923,6 +1005,14 @@
     };
     let (writes, segments, stream_indexes) =
         writes_and_segments_with_stream_indexes(&operations, &geometry);
+    // 崩溃状态摆在最后一个跑完的操作之前，停下那一步不进 RefFS（与 O1 那一路同一个范围）。
+    let reference_filesystem_history =
+        reference_filesystem_translation.translate(&operations, &geometry);
+    assert_eq!(
+        reference_filesystem_history.segment_count(),
+        segments.len(),
+        "RefFS 的崩溃时刻按同一条录制流、同一条切段规则切"
+    );
     let step_kinds: Vec<HistoryOperationKind> = history
         .operations
         .iter()
@@ -1016,6 +1106,17 @@
         tally.model_judgements += 1;
         let disagreement =
             crash_recovery_disagreement(&committed_versions, newest_persisted, &read_back);
+        // RefFS 那一票：崩溃时刻按切开当前段的那道屏障由哪一步发出归属（r6 用户定案 H1-收严，只给 RefFS 用；
+        // O1 判官与 `crash_point.step` 照旧按段里第一个被扣下的写）。
+        let reference_filesystem_judgement = reference_filesystem_history.judge_a_crash(
+            reference_filesystem_history.crash_moment_in_segment(crash_point.segment_index),
+            &read_back,
+        );
+        tally.note_the_reference_filesystem_vote(
+            StageTheReferenceFileSystemJudges::CrashImage,
+            &reference_filesystem_judgement.vote,
+            disagreement.as_ref(),
+        );
         if disagreement.is_none() {
             match &read_back {
                 ObservedReadBack::FileRead { .. } => tally.model_contents_compared += 1,
@@ -1044,7 +1145,8 @@
         if record_check.claimed_state_missing_unit {
             tally.record_claimed_state_missing_unit += 1;
         }
-        let violations = checker_violations_on(&image, &mut tally);
+        let mut violations = checker_violations_on(&image, &mut tally);
+        violations.extend(reference_filesystem_judgement.vote.violation());
         if !(violations.is_empty()
             && disagreement.is_none()
             && record_check == RecordCheck::default())
@@ -1155,9 +1257,11 @@
                 newest_persisted_before_the_mount: newest_persisted,
                 content_after_the_uninterrupted_mount,
                 fields_read_after_the_first_crash: &fields_read_after_the_crash,
-                floor_raised_by_the_uninterrupted_mount: mount_run.floor_raised_by_the_mount,
+                floor_raises_in_the_uninterrupted_mount_writes: mount_run
+                    .floor_raises_in_the_mount_writes,
                 parameters: &parameters,
                 space_admission: execution.space_admission,
+                reference_filesystem_at_the_first_crash: &reference_filesystem_judgement,
             },
             history.seed,
             crash_point,
@@ -1441,6 +1545,8 @@
     pub pool_after: MemoryPool,
     /// 挂载交回值里推没推抬 F、最后抬到的 F（挂载透明要它）；挂载被拒时 None。
     pub floor_raised_by_the_mount: Option<FloorRaisedByTheMount>,
+    /// 挂载推的抬 F 从挂载那一段写表（`mount_operations` 按同一份几何切出来的写表）的哪个写起（二次崩溃上 F 判不判按它分）；挂载被拒时 None。
+    pub floor_raises_in_the_mount_writes: Option<FloorRaisesInTheMountWrites>,
 }
 
 /// 挂载之后那一次发布写的内容：随种子与崩溃状态变，装得进一个数据单元（模型只罩一个数据单元的文件）。
@@ -1486,6 +1592,17 @@
         .as_ref()
         .ok()
         .map(|mounted| FloorRaisedByTheMount::of_the_mount_output(&mounted.output));
+    // 录制流此刻只有挂载那一段：按建池那一份参数的几何切出写表，与调用方切二次崩溃的写表同一张。
+    let floor_raises_in_the_mount_writes = mounted.as_ref().ok().map(|mounted| {
+        let geometry = FixedGeometry {
+            fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
+            journal_ring_bytes: parameters.geometry.journal_ring_bytes,
+            root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
+        };
+        let (mount_writes, _mount_segments, _mount_stream_indexes) =
+            writes_and_segments_with_stream_indexes(&stream.retained_operations(), &geometry);
+        FloorRaisesInTheMountWrites::of_the_mount_output(&mounted.output, &mount_writes)
+    });
     let outcome = match mounted {
         Err(error) => WritableMountOutcome::MountRefused {
             member: format!("{error:?}"),
@@ -1543,6 +1660,7 @@
         publish_operations,
         pool_after,
         floor_raised_by_the_mount,
+        floor_raises_in_the_mount_writes,
     }
 }
 
@@ -2008,32 +2126,26 @@
     }
 }
 
-/// 二次崩溃镜像上的挂载透明（r5 判决第二节 U6，用户定 F6「加二次」）：只读恢复读出的与 R1 比。挂载推了抬 F 而二次崩溃崩在暖机那一段
-/// （抬 F 那几串接在暖机之后）时，F 那一样不判（条款没写落在哪一处）。没崩过的那一次挂载被拒的不判。
+/// 二次崩溃镜像上的挂载透明（r5 判决第二节 U6，用户定 F6「加二次」）：只读恢复读出的与 R1 比。挂载推了抬 F 而二次崩溃镜像上
+/// 抬 F 的写有持久的（从第一串把新 F 写进系统配置的那个写起，`persisted_in_the_mount` 与挂载那一段写表逐条对应）时，F 那一样不判
+/// （条款没写落在哪一处）；暖机那一次发布的写崩出来的态照 r5 判决 U5 前半句判 F 不变（代码轮第二轮判决 K2-1）。没崩过的那一次挂载被拒的不判。
 fn mount_transparency_violations_of_a_second_crash(
     tally: &mut CrashInjectionTally,
     fields_read_after_the_first_crash: &RecoveryFieldsRead,
     fields_read_after_the_second_crash: &RecoveryFieldsRead,
-    floor_raised_by_the_uninterrupted_mount: Option<FloorRaisedByTheMount>,
+    floor_raises_in_the_uninterrupted_mount_writes: Option<FloorRaisesInTheMountWrites>,
+    persisted_in_the_mount: &[bool],
     phase: WritableMountPhase,
 ) -> Vec<(&'static str, String)> {
-    let Some(floor) = floor_raised_by_the_uninterrupted_mount else {
+    let Some(floor_raises_in_the_mount_writes) = floor_raises_in_the_uninterrupted_mount_writes
+    else {
         *tally
             .mount_transparency_not_judged
             .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_THE_UNINTERRUPTED_MOUNT_WAS_REFUSED)
             .or_insert(0) += 1;
         return Vec::new();
     };
-    let floor_raises = match (floor, phase) {
-        (FloorRaisedByTheMount::RaisedTo(_), WritableMountPhase::WarmUp) => {
-            FloorRaisesAtTheSecondCrash::SomeMayHaveLanded
-        }
-        (FloorRaisedByTheMount::NotRaised, _)
-        | (
-            FloorRaisedByTheMount::RaisedTo(_),
-            WritableMountPhase::Acquisition | WritableMountPhase::RowPublish,
-        ) => FloorRaisesAtTheSecondCrash::NoneLandedYet,
-    };
+    let floor_raises = floor_raises_in_the_mount_writes.at_a_second_crash(persisted_in_the_mount);
     let transparency = judge_mount_transparency(
         fields_read_after_the_first_crash,
         fields_read_after_the_second_crash,
@@ -2180,11 +2292,13 @@
         Result<ContentReadByRecovery, RemountIdempotenceNotJudged>,
     /// 第一次崩溃镜像上的只读恢复与挂载透明要比的字段（R1）。
     fields_read_after_the_first_crash: &'history RecoveryFieldsRead,
-    /// 没崩过的那一次挂载交回的推没推抬 F；挂载被拒时 None。
-    floor_raised_by_the_uninterrupted_mount: Option<FloorRaisedByTheMount>,
+    /// 没崩过的那一次挂载推的抬 F 从挂载那一段写表的哪个写起（没推是 [`FloorRaisesInTheMountWrites::NotPushed`]）；挂载被拒时 None。
+    floor_raises_in_the_uninterrupted_mount_writes: Option<FloorRaisesInTheMountWrites>,
     /// 二次崩溃之后再挂用的建池参数与空间准入开关（与没崩过的那一次挂载同一份）。
     parameters: &'history MakeFilesystemParameters,
     space_admission: SpaceAdmission,
+    /// RefFS 在第一次崩溃上的判定：判绿时带着以读出的内容重新起头的 RefFS，二次崩溃接着它判。
+    reference_filesystem_at_the_first_crash: &'history ReferenceFileSystemJudgementOfACrash,
 }
 
 /// 第三截：挂载途中的二次崩溃。每个二次崩溃状态上：只读恢复（看 journal）、问模型——允许的版本是模型提交过的每一版，加上这次挂载写出的根
@@ -2209,9 +2323,10 @@
         newest_persisted_before_the_mount,
         content_after_the_uninterrupted_mount,
         fields_read_after_the_first_crash,
-        floor_raised_by_the_uninterrupted_mount,
+        floor_raises_in_the_uninterrupted_mount_writes,
         parameters,
         space_admission,
+        reference_filesystem_at_the_first_crash,
     } = inputs;
     let mount_writes = records_across_the_crash.mount_writes();
     let phase_of_each_write = writable_mount_phase_of_each_write(mount_writes);
@@ -2276,6 +2391,15 @@
             newest_persisted,
             &read_back_after_the_second_crash,
         );
+        let reference_filesystem_vote = judge_a_second_crash_after_the_restart(
+            reference_filesystem_at_the_first_crash,
+            &read_back_after_the_second_crash,
+        );
+        tally.note_the_reference_filesystem_vote(
+            StageTheReferenceFileSystemJudges::SecondCrashInsideTheWritableMount,
+            &reference_filesystem_vote,
+            disagreement.as_ref(),
+        );
         tally.second_crash_record_checks += 1;
         let record_check = records_across_the_crash.record_check_after_a_second_crash(
             &persisted_in_the_mount,
@@ -2296,11 +2420,13 @@
         );
         tally.second_crash_checker_runs += 1;
         let mut violations = checker_violations_of(&image);
+        violations.extend(reference_filesystem_vote.violation());
         violations.extend(mount_transparency_violations_of_a_second_crash(
             tally,
             fields_read_after_the_first_crash,
             &fields_read_after_the_second_crash,
-            floor_raised_by_the_uninterrupted_mount,
+            floor_raises_in_the_uninterrupted_mount_writes,
+            &persisted_in_the_mount,
             second.phase,
         ));
         if violations.is_empty() && disagreement.is_none() && record_check == RecordCheck::default()
@@ -3418,6 +3544,102 @@
         );
     }
 
+    /// 代码轮第二轮判决 K2-1：崩溃后镜像上那次可写挂载推了抬 F 时，交回抬 F 从挂载写表的哪个写起——一个系统配置槽写，
+    /// 前面紧挨着暖机末尾的系统配置轮换；二次崩溃上 F 判不判按它分（分法两侧的判定由 harness 档
+    /// `mount_transparency_judged_field_by_field_after_a_writable_mount` 钉）。盘面照攻方 fms12 的造法：单元区 256 槽的小盘、
+    /// 关掉空间准入跑第一个文件之后重挂与 12 次覆盖写，再判准入挂载一次。
+    #[test]
+    fn the_writable_mount_on_a_crash_image_names_the_first_floor_raise_write_after_the_warm_up() {
+        use singlefs_harness::history::{
+            ContentChoice, ContentLength, HistoryDeviceWidth, PerStepChecker,
+        };
+        let width = HistoryDeviceWidth::UnitAreaOf256Slots;
+        let mut operations = vec![HistoryOperation::CloseAndMountWritable];
+        operations.extend(std::iter::repeat_n(
+            HistoryOperation::PublishOverwrite(ContentChoice {
+                length: ContentLength::InsideOneDataUnit { selector: 200 },
+                fill_seed: 11,
+            }),
+            12,
+        ));
+        let history = GeneratedHistory {
+            seed: HistorySeed(0),
+            starting_point: HistoryStartingPoint::AfterFirstFile,
+            operations,
+        };
+        let stream = SharedStream::retaining_contents();
+        let run = execute_history_with(
+            &history,
+            HistoryExecution {
+                per_step_checker: PerStepChecker::Skipped,
+                device_width: width,
+                space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
+            },
+            &stream,
+            &mut |_observation| {},
+        );
+        assert_eq!(
+            run.outcomes.len(),
+            history.operations.len(),
+            "写死的那段历史每一步都要跑完：{:?}",
+            run.ending
+        );
+        let mut crash_image = MemoryPool::with_devices(
+            &[DeviceIdentity(0), DeviceIdentity(1)],
+            width.device_bytes(),
+        );
+        crash_image.apply(&stream.retained_operations());
+        let crash_point = CrashPoint {
+            segment_index: 0,
+            persisted_within_the_segment: Vec::new(),
+            step: StepPosition::StartingPoint,
+            operation_kind: None,
+        };
+        let mount_run = mount_and_publish_once_on_the_crash_image(
+            &crash_image,
+            &width.parameters(),
+            HistoryExecution {
+                per_step_checker: PerStepChecker::Skipped,
+                device_width: width,
+                space_admission: SpaceAdmission::JudgedByTheFormula,
+            },
+            HistorySeed(0),
+            &crash_point,
+        );
+        assert!(
+            matches!(
+                mount_run.floor_raised_by_the_mount,
+                Some(FloorRaisedByTheMount::RaisedTo(_))
+            ),
+            "前提：这次挂载在写行与暖机之后推了抬 F：{:?}",
+            mount_run.floor_raised_by_the_mount
+        );
+        let Some(FloorRaisesInTheMountWrites::FirstWriteAt(first_floor_raise_write)) =
+            mount_run.floor_raises_in_the_mount_writes
+        else {
+            panic!(
+                "挂载推了抬 F，交回的却不是抬 F 从哪个写起：{:?}",
+                mount_run.floor_raises_in_the_mount_writes
+            );
+        };
+        let (mount_writes, _mount_segments, _mount_stream_indexes) =
+            writes_and_segments_with_stream_indexes(
+                &mount_run.mount_operations,
+                &width.fixed_geometry(),
+            );
+        assert_eq!(
+            (
+                mount_writes[first_floor_raise_write - 1].kind,
+                mount_writes[first_floor_raise_write].kind
+            ),
+            (
+                StepKind::SystemConfigurationSlot,
+                StepKind::SystemConfigurationSlot
+            ),
+            "抬 F 从把新 F 写进系统配置的那个写起，前面紧挨着暖机末尾的系统配置轮换"
+        );
+    }
+
     /// 重放行写出的两种摆法（只抽样、短段整段枚举）都经重放入口的解析原样读回，连同种子区间与取样参数；
     /// 默认值取的是另一组，读回的不是写出的那一组就红。
     #[test]
````

## `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`

````diff
--- crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	2026-10-02 15:33:47.487007664 +0000
+++ crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	2026-10-02 18:08:49.852021196 +0000
@@ -206,7 +206,8 @@
 
 // 全池没有一个可用的槽 0 时逐档读一块盘的槽 1（image::slot_one_read_at_every_slot_spacing_the_format_allows）：4096 起、每 512 一档、
 // 到根环起点（这一版读者只收 64 × 16384）减 4096 为止；只收可用、且记的槽距等于所在偏移的，取世代号最大的、同号取偏移小的；
-// 读到带不收之值的那一档按偏移升序交回第一个；一档可用的都没收到时，见过自证过而不可择的就交回那一类（I-8.5 数它），再没有就是不自证
+// 读到带不收之值的那一档按偏移升序交回第一个；读到可用的一档先判 I-8.1 三个字段等式（实现那张不收之值表的最后三格，代码轮第二轮判决 K3-1），
+// 不成立同样当场交回带不收之值、不核偏移不比世代号；一档可用的都没收到时，见过自证过而不可择的就交回那一类（I-8.5 数它），再没有就是不自证
 fn checker_slot_one_by_reading_every_spacing(device: u32) -> CheckerSlot {
     var best = empty_checker_slot(2u);
     var have_best = false;
@@ -217,6 +218,9 @@
         if (s.reading == 1u) {
             return s;
         }
+        if (s.reading == SLOT_USABLE && !journal_ring_geometry_fields_agree(s)) {
+            return empty_checker_slot(SLOT_CARRIES_A_REFUSED_VALUE);
+        }
         if (s.reading == 0u && s.spacing == candidate) {
             if (!have_best || lt64(best.generation, s.generation)) {
                 best = s;
````

## `crates/singlefs-harness/Cargo.toml`

````diff
--- crates/singlefs-harness/Cargo.toml	2026-09-27 12:42:30.156267794 +0000
+++ crates/singlefs-harness/Cargo.toml	2026-10-02 16:17:24.607088834 +0000
@@ -9,4 +9,7 @@
 singlefs-checker = { path = "../singlefs-checker" }
 singlefs-core = { path = "../singlefs-core" }
 singlefs-format = { path = "../singlefs-format" }
+# RefFS（D13（验证路线） 已定项 3）：调用方的翻译层（src/reference_filesystem_translation.rs）依赖它；方向只许 harness → RefFS
+# （门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格 ⑤⑥⑦）。
+singlefs-reference-filesystem = { path = "../singlefs-reference-filesystem" }
 
````

## `crates/singlefs-harness/src/fault_injection.rs`

````diff
--- crates/singlefs-harness/src/fault_injection.rs	2026-10-02 01:47:19.331550625 +0000
+++ crates/singlefs-harness/src/fault_injection.rs	2026-10-02 18:08:52.381489166 +0000
@@ -31,7 +31,9 @@
 use std::sync::mpsc;
 use std::time::Instant;
 
-use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::image::{
+    InvariantVerdict, SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
+};
 use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
 use singlefs_core::block_device::{
@@ -50,7 +52,8 @@
     newest_ring_root_and_slot_count, raised_floor_lands_only_on_abandoned_roots, AppliedEffect,
     ColdStartReadBack, FailureObservation, FailureSignature, GeneratedHistory, GenerationWeights,
     HarnessJudgement, HistoryEnding, HistoryExecution, HistoryOperation, HistoryOperationKind,
-    HistoryRun, HistorySeed, PerStepChecker, SeededRandomSource, StepOutcome, StepPosition,
+    HistoryRun, HistorySeed, PerStepChecker, SeededRandomSource, StartingPointStep, StepOutcome,
+    StepPosition,
 };
 use crate::memory_pool::{
     newest_persisted_root, root_identity_written_by, writes_and_segments_with_stream_indexes,
@@ -64,6 +67,10 @@
     model_ring_position, model_root_key, observed_read_back_after_a_crash,
 };
 use crate::numeric_environment_variable::number_from_the_variable_value_panicking_when_unreadable;
+use crate::reference_filesystem_translation::{
+    JointVerdictOfTheReferenceFileSystemAndTheModel, ReferenceFileSystemTranslation,
+    ReferenceFileSystemVote, WhatTheReopenMustRecover,
+};
 use crate::segments::{FixedGeometry, StepKind};
 use crate::{RecordedOperation, RecordedOperationKind, SharedStream};
 
@@ -1352,11 +1359,116 @@
     }
 }
 
+/// 只有故障注入撞得到的「已知红」形态：随机历史不注入，撞不到；两边共用的那份清单是 `history::KNOWN_RED_FORMS`。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub enum KnownRedFormOnlyAfterAFault {
+    /// 起点段里 mkfs 自己把错交回来，录制流里一次系统配置槽写都没落盘；注入之后的镜像上池级 checker 只判红
+    /// I-8.5（tail 槽有效集非空）一条，重开走读失败（[`ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo`]）。
+    /// 用户 2026-10-03 定：盘上全池没有一个自证过的系统配置槽，就是 I-8.5 那一行写的「全池皆废」，红是对的，照判、记在这里；
+    /// 它不是要修的缺口。
+    MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded,
+}
+
+impl KnownRedFormOnlyAfterAFault {
+    /// 报告里的名字。
+    #[must_use]
+    pub const fn name(self) -> &'static str {
+        match self {
+            KnownRedFormOnlyAfterAFault::MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded => {
+                "mkfs 在系统配置槽落盘之前报错，全池皆废、只红 I-8.5"
+            }
+        }
+    }
+}
+
+/// 一次注入以「已知红」收尾时撞上的是哪一条：两边共用的清单里第几条，或只有故障注入撞得到的那几条之一。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub enum KnownRedFormAtAFault {
+    /// `history::KNOWN_RED_FORMS` 的第几条（从 0 数）。
+    SharedWithRandomHistories(usize),
+    OnlyAfterAFault(KnownRedFormOnlyAfterAFault),
+}
+
+impl KnownRedFormAtAFault {
+    /// 报告里的一段：共用清单写第几条，故障注入自己那几条写名字。
+    #[must_use]
+    pub fn render(self) -> String {
+        match self {
+            KnownRedFormAtAFault::SharedWithRandomHistories(form) => format!("第 {form} 条"),
+            KnownRedFormAtAFault::OnlyAfterAFault(form) => {
+                format!("故障注入那一条（{}）", form.name())
+            }
+        }
+    }
+}
+
+/// 判故障注入自己那几条「已知红」要看的事实，都是 [`inject_one_fault`] 判到这一步时本来就拿着的：不另读盘、不另跑实现。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct FactsAfterTheFault {
+    /// 注入那一段历史停在起点段某一步把错交回来上（`HarnessJudgement::TheStartingPointReturnedAnError`）时是哪一步；
+    /// 别的结局（跑完了、停在起点之后的某一步、停在 panic 上）是 None。
+    pub starting_point_step_that_returned_an_error: Option<StartingPointStep>,
+    /// 录制流（只记真正落到盘上的写）里写在系统配置槽上的写有几次（[`StepKind::SystemConfigurationSlot`]；整段清零不算）。
+    pub system_configuration_slot_writes_that_landed: usize,
+    /// 重开走到了哪一格。
+    pub reopened: ReopenedVersion,
+}
+
+/// 这条失败是不是只有故障注入撞得到的那几条「已知红」之一；不是就交回 None，照共用清单与新发现往下判。
+///
+/// mkfs 没做完那一条要五样同时成立，少一样就不认（照报新发现，不放宽）：
+/// 1. 停在起点段、把错交回来的是 mkfs 自己（取号、暖机、第一个文件报错时 mkfs 已经写下了系统配置槽）；
+/// 2. 录制流里一次系统配置槽写都没落盘（mkfs 写完系统配置之后才报错的，盘上有自证过的槽，I-8.5 红就不是这一形）；
+/// 3. 重开走读失败、而盘上本来就没有一版立得住；
+/// 4. 这条失败里没有 panic、执行器判出的失败、模型对不上、记录核对器判红；
+/// 5. checker（连同 RefFS 借违例那一栏带的那一票）判红的恰好一条，就是 I-8.5。
+#[must_use]
+pub fn known_red_form_only_after_a_fault(
+    observation: &FailureObservation,
+    facts: &FactsAfterTheFault,
+) -> Option<KnownRedFormOnlyAfterAFault> {
+    let make_filesystem_returned_the_error = match facts.starting_point_step_that_returned_an_error
+    {
+        Some(StartingPointStep::MakeFilesystem) => true,
+        Some(
+            StartingPointStep::ReadThePoolForTheSession
+            | StartingPointStep::AcquireInstance
+            | StartingPointStep::WarmUp
+            | StartingPointStep::PublishFirstFile,
+        )
+        | None => false,
+    };
+    let no_system_configuration_slot_landed =
+        facts.system_configuration_slot_writes_that_landed == 0;
+    let the_reopen_found_nothing_to_recover = match facts.reopened {
+        ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo => true,
+        ReopenedVersion::CommittedByTheModel
+        | ReopenedVersion::TheVersionTheFaultedStepWasWriting
+        | ReopenedVersion::OutsideEverythingTheModelAllows => false,
+    };
+    let only_the_checker_flagged_it = observation.panic.is_none()
+        && observation.harness_judgement.is_none()
+        && observation.model_disagreement.is_none()
+        && crate::history::record_check_aspects(&observation.record_check).is_empty();
+    let only_some_device_carries_a_self_verified_system_configuration_is_violated = matches!(
+        observation.violations.as_slice(),
+        [(invariant, _)] if *invariant == SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION
+    );
+    (make_filesystem_returned_the_error
+        && no_system_configuration_slot_landed
+        && the_reopen_found_nothing_to_recover
+        && only_the_checker_flagged_it
+        && only_some_device_carries_a_self_verified_system_configuration_is_violated)
+        .then_some(
+            KnownRedFormOnlyAfterAFault::MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded,
+        )
+}
+
 /// 「已知红」收尾的一次注入。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct KnownRedAtAFault {
     pub seed: HistorySeed,
-    pub form: usize,
+    pub form: KnownRedFormAtAFault,
     pub drawn: DrawnFault,
 }
 
@@ -1397,10 +1509,14 @@
     pub reopens_outside_everything_the_model_allows: u64,
     /// 重开走读失败，而盘上本来就没有一版立得住（`failed_reopen_is_the_right_answer`）：只记不判。
     pub reopens_that_correctly_found_nothing_to_recover: u64,
+    /// 重开上 RefFS 与 O1 判官两票合起来（G1 用户定案：任一红即红），按哪一边红分开计；RefFS 弃权的按理由另计。
+    pub reopens_by_the_reference_filesystem_and_model_verdicts:
+        BTreeMap<JointVerdictOfTheReferenceFileSystemAndTheModel, u64>,
+    pub reference_filesystem_abstentions_at_the_reopen: BTreeMap<&'static str, u64>,
     pub checker_runs_on_the_image_after_the_fault: u64,
     pub invariant_holds: BTreeMap<&'static str, u64>,
     pub invariant_not_applicable: BTreeMap<&'static str, u64>,
-    pub faults_ending_known_red: BTreeMap<usize, u64>,
+    pub faults_ending_known_red: BTreeMap<KnownRedFormAtAFault, u64>,
     pub faults_ending_new_finding: u64,
 }
 
@@ -1454,6 +1570,18 @@
             other.reopens_outside_everything_the_model_allows;
         self.reopens_that_correctly_found_nothing_to_recover +=
             other.reopens_that_correctly_found_nothing_to_recover;
+        for (verdict, count) in &other.reopens_by_the_reference_filesystem_and_model_verdicts {
+            *self
+                .reopens_by_the_reference_filesystem_and_model_verdicts
+                .entry(*verdict)
+                .or_insert(0) += count;
+        }
+        for (reason, count) in &other.reference_filesystem_abstentions_at_the_reopen {
+            *self
+                .reference_filesystem_abstentions_at_the_reopen
+                .entry(reason)
+                .or_insert(0) += count;
+        }
         self.checker_runs_on_the_image_after_the_fault +=
             other.checker_runs_on_the_image_after_the_fault;
         for (key, count) in &other.invariant_holds {
@@ -1583,6 +1711,15 @@
         );
         let _ = writeln!(
             text,
+            "重开上 RefFS 与 O1 判官两票（任一红即红）：{:?}；RefFS 弃权的理由 {:?}",
+            self.reopens_by_the_reference_filesystem_and_model_verdicts
+                .iter()
+                .map(|(verdict, count)| (verdict.name(), *count))
+                .collect::<Vec<_>>(),
+            self.reference_filesystem_abstentions_at_the_reopen
+        );
+        let _ = writeln!(
+            text,
             "注入之后的镜像上跑池级 checker {} 次：判绿 {} 条次、不适用 {} 条次",
             self.checker_runs_on_the_image_after_the_fault,
             self.invariant_holds.values().sum::<u64>(),
@@ -1591,7 +1728,11 @@
         let _ = writeln!(
             text,
             "以「已知红」收尾 {:?}、新发现 {}",
-            self.faults_ending_known_red, self.faults_ending_new_finding
+            self.faults_ending_known_red
+                .iter()
+                .map(|(form, count)| (form.render(), *count))
+                .collect::<BTreeMap<_, _>>(),
+            self.faults_ending_new_finding
         );
         text
     }
@@ -1980,6 +2121,20 @@
     acquired_instances_left: &'run mut Vec<AcquiredInstanceLeft>,
 }
 
+impl FaultInjectionAccumulator<'_> {
+    /// 一次注入以「已知红」收尾：按形态计一次，再记下是哪个种子上的哪一次注入。
+    fn record_a_known_red_hit(
+        &mut self,
+        seed: HistorySeed,
+        form: KnownRedFormAtAFault,
+        drawn: DrawnFault,
+    ) {
+        *self.tally.faults_ending_known_red.entry(form).or_insert(0) += 1;
+        self.known_red_hits
+            .push(KnownRedAtAFault { seed, form, drawn });
+    }
+}
+
 /// 注入那一段的结局是不是「入口返回了错误」：没 panic、checker 没判红，而且那一段要么返回了 `Err`，
 /// 要么是冷启动、读回报了错。注入的块设备错在模型里没有对应的拒绝理由（`ObservedRefusalReason::Unexplained`），
 /// 模型因此必然报「该成却拒了」——那一格不算失败，它就是这一件要的结果。
@@ -2188,13 +2343,23 @@
     // 最后一次观察那一刻的模型与录制流长度：历史停下的那一步不调观察者，它之前那一步的这两样就是「停下那一步开始时」的。
     let mut model_at_the_last_observation: Option<crate::model::IdealModel> = None;
     let mut stream_length_at_the_last_observation = 0usize;
+    // RefFS 的输入（G1 用户定案：重开的判定 = RefFS ∧ O1 判官全四条）：开着注入跑的那一遍逐步攒，停下的那一步跑完之后补。
+    let mut reference_filesystem_translation =
+        ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
     let run = execute_history_with_faults(history, execution, &stream, &plan, &mut |observation| {
         for (key, file) in observation.model.committed_versions() {
             committed_by_the_armed_run.insert(key, file);
         }
         model_at_the_last_observation = Some(observation.model.clone());
         stream_length_at_the_last_observation = stream.operation_count();
+        reference_filesystem_translation
+            .note_an_observed_step(observation, stream_length_at_the_last_observation);
     });
+    reference_filesystem_translation.note_the_step_the_history_stopped_in(
+        &history.operations,
+        &run,
+        stream.operation_count(),
+    );
     let fired = plan.fired();
     accumulator.tally.faults += 1;
     *accumulator
@@ -2352,6 +2517,35 @@
     } else {
         crash_recovery_disagreement(&allowed, newest_persisted, &read_back)
     };
+    // RefFS 那一票：重开 = 停下之后崩、全部步都返回过（停下那一步报错的按「写了之后报错」两可）。每一档注入 RefFS 都弃权、O1 判官照旧：
+    // U1 射程句「带故障的状态（说谎的设备、丢整块盘 C519）不适用」管全部带故障的状态，括号里是举例（调度记录第 12 批「12-2 交回」
+    // 那一条）；弃权按理由计进 `reference_filesystem_abstentions_at_the_reopen`。
+    let reference_filesystem_vote = reference_filesystem_translation
+        .translate(&operations, &geometry)
+        .judge_a_reopen_after_an_injected_fault(
+            drawn.fault,
+            if failed_reopen_is_the_right_answer(drawn.fault, ending_observation) {
+                WhatTheReopenMustRecover::NothingBecauseNoVersionStands
+            } else {
+                WhatTheReopenMustRecover::AVersionTheHistoryLeft
+            },
+            &read_back,
+        );
+    *accumulator
+        .tally
+        .reopens_by_the_reference_filesystem_and_model_verdicts
+        .entry(JointVerdictOfTheReferenceFileSystemAndTheModel::of(
+            &reference_filesystem_vote,
+            disagreement.as_ref(),
+        ))
+        .or_insert(0) += 1;
+    if let ReferenceFileSystemVote::Abstained(reason) = &reference_filesystem_vote {
+        *accumulator
+            .tally
+            .reference_filesystem_abstentions_at_the_reopen
+            .entry(reason.name())
+            .or_insert(0) += 1;
+    }
     let reopened = if nothing_was_left_to_recover {
         accumulator
             .tally
@@ -2375,7 +2569,7 @@
             .reopens_outside_everything_the_model_allows += 1;
         ReopenedVersion::OutsideEverythingTheModelAllows
     };
-    let violations_after_the_fault = checker_violations_on(&image, accumulator.tally);
+    let mut violations_after_the_fault = checker_violations_on(&image, accumulator.tally);
     let reopen_text = format!("{reopened:?}（{}）", describe_read_back(&read_back));
     // C378：注入落在一次挂载里、那一步返回了错误，而重开的盘上系统配置的实例代号比这一步之前大——取号之后的写行或暖机报了错，
     // 新号没回卷。认下的已知行为：记进这一格、在报告里点名，不进新发现，也不挡下面的判定。
@@ -2468,7 +2662,9 @@
         }
     });
 
-    // 三、判：注入那一步返回错误的那一格不算失败；别的失败（panic、checker 判红、重开走到模型不允许的版本）逐条分类。
+    // 三、判：注入那一步返回错误的那一格不算失败；别的失败（panic、checker 判红、重开走到模型或 RefFS 不允许的版本）逐条分类。
+    // RefFS 判红的那一项借违例那一栏带着走（今天每一档 RefFS 都弃权，这一项是空的，不会落进 `left_by_the_lying_device` 的豁免）。
+    violations_after_the_fault.extend(reference_filesystem_vote.violation());
     let mut failures: Vec<FailureObservation> = Vec::new();
     if !surfaced {
         if let Some(observation) = ending_observation {
@@ -2491,25 +2687,43 @@
             record_check: RecordCheck::default(),
         });
     }
+    let facts_after_the_fault = FactsAfterTheFault {
+        starting_point_step_that_returned_an_error: ending_observation.and_then(|observation| {
+            match &observation.harness_judgement {
+                Some(HarnessJudgement::TheStartingPointReturnedAnError { step, .. }) => Some(*step),
+                Some(_) | None => None,
+            }
+        }),
+        system_configuration_slot_writes_that_landed: writes
+            .iter()
+            .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
+            .count(),
+        reopened,
+    };
     for observation in failures {
         if observation.panic.is_some() {
             accumulator.tally.faults_that_panicked += 1;
         }
-        // 「已知红」先认：清单里那两条是实现与 checker 读法对不上的老账（增补 2 收口表第 ②、43 行），
+        // 「已知红」先认：清单里那几条是认下来的形态（共用清单是实现与 checker 读法对不上的老账，增补 2 收口表第 ②、43 行；
+        // 故障注入自己那一条是用户 2026-10-03 定的「mkfs 没写下系统配置槽，全池皆废判红是对的」），
         // 与设备说不说谎无关，说谎的设备把它撞出来时照样记进「已知红」那一格，不许被下面那一条接走。
+        if let Some(form) = known_red_form_only_after_a_fault(&observation, &facts_after_the_fault)
+        {
+            accumulator.record_a_known_red_hit(
+                history.seed,
+                KnownRedFormAtAFault::OnlyAfterAFault(form),
+                *drawn,
+            );
+            continue;
+        }
         match classify_failure(observation) {
             HistoryEnding::Completed => {}
             HistoryEnding::KnownRed { form, .. } => {
-                *accumulator
-                    .tally
-                    .faults_ending_known_red
-                    .entry(form)
-                    .or_insert(0) += 1;
-                accumulator.known_red_hits.push(KnownRedAtAFault {
-                    seed: history.seed,
-                    form,
-                    drawn: *drawn,
-                });
+                accumulator.record_a_known_red_hit(
+                    history.seed,
+                    KnownRedFormAtAFault::SharedWithRandomHistories(form),
+                    *drawn,
+                );
             }
             HistoryEnding::NewFinding {
                 signature,
@@ -2688,7 +2902,8 @@
             self.execution.name()
         );
         text.push_str(&self.tally.render());
-        let mut known_red_by_form: BTreeMap<usize, Vec<&KnownRedAtAFault>> = BTreeMap::new();
+        let mut known_red_by_form: BTreeMap<KnownRedFormAtAFault, Vec<&KnownRedAtAFault>> =
+            BTreeMap::new();
         for hit in &self.known_red_hits {
             known_red_by_form.entry(hit.form).or_default().push(hit);
         }
@@ -2696,7 +2911,8 @@
             let first = hits.first().expect("这一条至少有一次命中");
             let _ = writeln!(
                 text,
-                "「已知红」第 {form} 条：{} 次，第一次在种子 {}（{}）",
+                "「已知红」{}：{} 次，第一次在种子 {}（{}）",
+                form.render(),
                 hits.len(),
                 first.seed.0,
                 first.drawn.render()
@@ -3421,6 +3637,196 @@
         );
     }
 
+    /// 注入之后的镜像上 checker 判红的就是 `violated_invariants` 那几条、模型对不上的是 `model_disagreement` 的一条失败，停在起点段。
+    fn failure_after_the_fault_at_the_starting_point(
+        violated_invariants: &[&'static str],
+        model_disagreement: Option<ModelDisagreement>,
+    ) -> FailureObservation {
+        FailureObservation {
+            position: StepPosition::StartingPoint,
+            operation_kind: None,
+            violations: violated_invariants
+                .iter()
+                .map(|invariant| (*invariant, "池里 2 块盘一个都没自证过".to_string()))
+                .collect(),
+            panic: None,
+            newest_ring_root_txg: None,
+            root_ring_slot_count: None,
+            harness_judgement: None,
+            model_disagreement,
+            raised_floor_lands_only_on_abandoned_roots: None,
+            record_check: RecordCheck::default(),
+        }
+    }
+
+    /// 注入之后的镜像上只红 I-8.5 的那条失败（mkfs 没做完那一形的签名）。
+    fn failure_where_only_some_device_carries_a_self_verified_system_configuration_is_violated(
+    ) -> FailureObservation {
+        failure_after_the_fault_at_the_starting_point(
+            &[SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION],
+            None,
+        )
+    }
+
+    /// 判那一形要的三样事实，逐样由调用方给。
+    fn facts_after_the_fault(
+        starting_point_step_that_returned_an_error: Option<StartingPointStep>,
+        system_configuration_slot_writes_that_landed: usize,
+        reopened: ReopenedVersion,
+    ) -> FactsAfterTheFault {
+        FactsAfterTheFault {
+            starting_point_step_that_returned_an_error,
+            system_configuration_slot_writes_that_landed,
+            reopened,
+        }
+    }
+
+    /// mkfs 报错、重开走读失败；落盘的系统配置槽写有几次由调用方给。
+    fn make_filesystem_failed_with_system_configuration_slot_writes_landed(
+        system_configuration_slot_writes_that_landed: usize,
+    ) -> FactsAfterTheFault {
+        facts_after_the_fault(
+            Some(StartingPointStep::MakeFilesystem),
+            system_configuration_slot_writes_that_landed,
+            ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo,
+        )
+    }
+
+    /// 对照（用户 2026-10-03 定的那一形只认「mkfs 在系统配置槽落盘之前报错」）：同一个签名（只红 I-8.5、重开走读失败、mkfs 报的错），
+    /// 而录制流里系统配置槽写已经落了盘——mkfs 写完系统配置之后才报错，盘上本该有自证过的槽，I-8.5 红就不是「mkfs 没做完」能交代的，
+    /// 照报新发现。同一条失败换成一次都没落盘时认成那一形，证明这一格不是因为判法什么都不认才绿。
+    #[test]
+    fn i85_only_failure_after_make_filesystem_wrote_its_system_configuration_slots_stays_a_new_finding(
+    ) {
+        let observation =
+            failure_where_only_some_device_carries_a_self_verified_system_configuration_is_violated(
+            );
+        let slots_written_by_make_filesystem =
+            2 * usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
+                .expect("每盘槽数装得进 usize");
+        assert_eq!(
+            known_red_form_only_after_a_fault(
+                &observation,
+                &make_filesystem_failed_with_system_configuration_slot_writes_landed(0)
+            ),
+            Some(
+                KnownRedFormOnlyAfterAFault::MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded
+            ),
+            "mkfs 在系统配置槽落盘之前报错、只红 I-8.5、重开走读失败：这是用户定的那一形，要认"
+        );
+        assert_eq!(
+            known_red_form_only_after_a_fault(
+                &observation,
+                &make_filesystem_failed_with_system_configuration_slot_writes_landed(
+                    slots_written_by_make_filesystem
+                )
+            ),
+            None,
+            "系统配置槽已经落盘（{slots_written_by_make_filesystem} 次）之后的 I-8.5 红不是 mkfs 没做完那一形"
+        );
+        match classify_failure(observation) {
+            HistoryEnding::NewFinding { signature, .. } => assert_eq!(
+                signature,
+                FailureSignature::CheckerViolations {
+                    invariants: vec![SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION],
+                },
+                "共用清单接不走它，签名照旧是只红 I-8.5"
+            ),
+            ending @ (HistoryEnding::Completed | HistoryEnding::KnownRed { .. }) => {
+                panic!("只红 I-8.5 的失败不在共用清单里，该是新发现：{ending:?}")
+            }
+        }
+    }
+
+    /// mkfs 没做完那一形不放宽：别的步报错、多红一条或换一条、重开走到别的格、带着模型对不上的，一律不认。
+    #[test]
+    fn make_filesystem_known_red_form_takes_no_other_step_signature_or_reopen() {
+        let observation =
+            failure_where_only_some_device_carries_a_self_verified_system_configuration_is_violated(
+            );
+        let facts = make_filesystem_failed_with_system_configuration_slot_writes_landed(0);
+        assert!(
+            known_red_form_only_after_a_fault(&observation, &facts).is_some(),
+            "基准那一条要认，下面每一格只改一样"
+        );
+        for step in [
+            StartingPointStep::ReadThePoolForTheSession,
+            StartingPointStep::AcquireInstance,
+            StartingPointStep::WarmUp,
+            StartingPointStep::PublishFirstFile,
+        ] {
+            let facts_of_another_step = facts_after_the_fault(
+                Some(step),
+                0,
+                ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo,
+            );
+            assert_eq!(
+                known_red_form_only_after_a_fault(&observation, &facts_of_another_step),
+                None,
+                "{}报错时 mkfs 已经做完，不是这一形",
+                step.name()
+            );
+        }
+        let facts_without_a_starting_point_error = facts_after_the_fault(
+            None,
+            0,
+            ReopenedVersion::NothingWasLeftToRecoverAndTheReopenSaidSo,
+        );
+        assert_eq!(
+            known_red_form_only_after_a_fault(&observation, &facts_without_a_starting_point_error),
+            None,
+            "历史没停在起点段的报错上，不是这一形"
+        );
+        for reopened in [
+            ReopenedVersion::CommittedByTheModel,
+            ReopenedVersion::TheVersionTheFaultedStepWasWriting,
+            ReopenedVersion::OutsideEverythingTheModelAllows,
+        ] {
+            let facts_of_another_reopen =
+                facts_after_the_fault(Some(StartingPointStep::MakeFilesystem), 0, reopened);
+            assert_eq!(
+                known_red_form_only_after_a_fault(&observation, &facts_of_another_reopen),
+                None,
+                "重开走到 {reopened:?}，不是这一形"
+            );
+        }
+        let with_one_more_violation = failure_after_the_fault_at_the_starting_point(
+            &[
+                SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
+                "I-4.1",
+            ],
+            None,
+        );
+        assert_eq!(
+            known_red_form_only_after_a_fault(&with_one_more_violation, &facts),
+            None,
+            "除了 I-8.5 还红别的，不是这一形"
+        );
+        let with_another_violation =
+            failure_after_the_fault_at_the_starting_point(&["I-4.1"], None);
+        assert_eq!(
+            known_red_form_only_after_a_fault(&with_another_violation, &facts),
+            None,
+            "红的不是 I-8.5，不是这一形"
+        );
+        let with_a_model_disagreement = failure_after_the_fault_at_the_starting_point(
+            &[SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION],
+            Some(ModelDisagreement {
+                aspect: ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
+                model_answer: "该成".to_string(),
+                implementation_answer: "拒了".to_string(),
+                implementation_reported_ceiling: None,
+                implementation_reported_root_ring_slot_still_bad_after_one_reread: None,
+                implementation_rows_written: None,
+            }),
+        );
+        assert_eq!(
+            known_red_form_only_after_a_fault(&with_a_model_disagreement, &facts),
+            None,
+            "带着模型对不上，不是这一形"
+        );
+    }
+
     /// 哪两种是「设备说谎」：报错那四种的盘面不一致照算新发现，说谎那两种单记一格
     /// （`InjectedFault::the_device_lies` 分的就是这一刀）。
     #[test]
````

## `crates/singlefs-harness/src/lib.rs`

````diff
--- crates/singlefs-harness/src/lib.rs	2026-10-02 14:44:16.997207352 +0000
+++ crates/singlefs-harness/src/lib.rs	2026-10-02 16:17:24.607297533 +0000
@@ -24,6 +24,7 @@
 pub mod new_pool_file_creation_regions;
 pub mod numeric_environment_variable;
 pub mod read_tally;
+pub mod reference_filesystem_translation;
 pub mod scenario;
 pub mod segments;
 pub mod sha256;
````

## `crates/singlefs-harness/src/mount_transparency.rs`

````diff
--- crates/singlefs-harness/src/mount_transparency.rs	2026-10-02 14:44:16.997340212 +0000
+++ crates/singlefs-harness/src/mount_transparency.rs	2026-10-02 18:08:52.381597435 +0000
@@ -2,8 +2,9 @@
 //! 逐字段表与二次崩溃、拒绝合法照 `research/prompts/m3-formal-r5-main-verification.md` 第二节 U5、U6 两行与第四节第 5、6 条，零轮）的判法。
 //!
 //! 射程：不带故障、可写挂载做成的崩溃状态。崩溃镜像上只读恢复一次（R1），在它上面做一个动作——只挂载（取号、写行、暖机与挂载自己推的抬 F），
-//! 或挂载加一次发布——之后再只读恢复一次（R2），逐字段比（[`judge_mount_transparency`]）；挂载途中三个相位的二次崩溃镜像上，
-//! 只读恢复读出的与 R1 比（用户 2026-10-02 定 F6「加二次」）。可写挂载被拒的状态不判透明，判拒绝本身合不合法（[`judge_writable_mount_refusal`]）。
+//! 或挂载加一次发布——之后再只读恢复一次（R2），逐字段比（[`judge_mount_transparency`]）；挂载途中各相位的二次崩溃镜像上，
+//! 只读恢复读出的与 R1 比（用户 2026-10-02 定 F6「加二次」）；抬 F 的写有持久的那几态 F 不判（[`FloorRaisesInTheMountWrites`]）。
+//! 可写挂载被拒的状态不判透明，判拒绝本身合不合法（[`judge_writable_mount_refusal`]）。
 //!
 //! 读盘的只有 [`read_recovery_fields`] 与 [`judge_writable_mount_refusal`] 里在崩溃镜像上复算拒的判据那一截；[`judge_mount_transparency`]
 //! 只吃读好的字段与挂载交回值。checker 档的崩溃注入调它，harness 档的轻用例也调它（harness 档不依赖 checker 档）。
@@ -24,9 +25,12 @@
     UserVisibleTreeRootPointers,
 };
 use singlefs_core::root_record::{RootRecord, UnmountMarker};
-use singlefs_core::system_configuration::SystemImmutableConfiguration;
+use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableConfiguration};
 use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
 
+use crate::memory_pool::RetainedWrite;
+use crate::segments::StepKind;
+
 /// 一次只读恢复读出的 POSIX 内容与存在性。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum PosixContentRead {
@@ -247,14 +251,77 @@
     }
 }
 
-/// 二次崩溃镜像上，挂载推的抬 F 那几串可能已经落了一部分没有：挂载没推，或崩在暖机之前（取号、写行），一步都还没落。
+/// 二次崩溃镜像上，挂载推的抬 F 那几串可能已经落了一部分没有：挂载没推，或抬 F 的写一个都没持久（崩在取号、写行、暖机里，
+/// 暖机那一次发布连同它末尾的系统配置轮换带的都是挂载之前的 F），一步都还没落——F 照 r5 判决 U5 前半句判不变。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum FloorRaisesAtTheSecondCrash {
     NoneLandedYet,
-    /// 挂载推了抬 F，二次崩溃崩在暖机那一段（抬 F 那几串接在暖机之后）：F 生效值落在 R1 的与抬到的之间哪一处，条款没写。
+    /// 挂载推了抬 F，二次崩溃镜像上抬 F 的写有持久的（从第一串把新 F 写进系统配置的那个写起）：F 生效值落在 R1 的与抬到的之间哪一处，条款没写。
     SomeMayHaveLanded,
 }
 
+/// 挂载那一段写流（屏障不进表，与二次崩溃切段的是同一张写表）里，挂载推的抬 F 从第几个写起。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum FloorRaisesInTheMountWrites {
+    NotPushed,
+    /// 第一串抬 F 的第一个写的下标：那一串先把新 F 写进每块盘的系统配置、过一道屏障，之后才发带新 F 的空发布
+    /// （`singlefs_core::transaction::write_the_raised_floor_into_every_system_configuration`）；这个下标之前的写带的都是挂载之前的 F。
+    FirstWriteAt(usize),
+}
+
+impl FloorRaisesInTheMountWrites {
+    /// 在这次挂载的写表（`mount_writes`）里找第一串抬 F 的第一个写：第一个带着那一串新 F 的系统配置槽写
+    /// （新 F 取 `MountOutput::space_admission` 交回的第一串第一次发布的根里的 F，与写进系统配置的是同一个值）。
+    ///
+    /// # Panics
+    /// 挂载交回推了抬 F，`mount_writes` 里却没有带那一串新 F 的系统配置槽写：交来的写表不是这次挂载的。
+    #[must_use]
+    pub fn of_the_mount_output(output: &MountOutput, mount_writes: &[RetainedWrite]) -> Self {
+        let Some(first_raise) = output.space_admission.floor_raises().first() else {
+            return FloorRaisesInTheMountWrites::NotPushed;
+        };
+        let raised_floor = first_raise
+            .publishes
+            .first()
+            .expect("抬 F 那一串至少推一次带新 F 的空发布（RaisedFloor::publishes 的契约）")
+            .root
+            .rollback_floor;
+        let first_write = mount_writes
+            .iter()
+            .position(|write| {
+                write.kind == StepKind::SystemConfigurationSlot
+                    && write
+                        .bytes()
+                        .and_then(|bytes| SystemConfiguration::parse_slot(bytes).ok())
+                        .is_some_and(|written| written.quantities.rollback_floor == raised_floor)
+            })
+            .expect("抬 F 那一串第一次发布之前先把新 F 写进每块盘的系统配置，那几个写在这次挂载的写表里");
+        FloorRaisesInTheMountWrites::FirstWriteAt(first_write)
+    }
+
+    /// 一个二次崩溃状态上抬 F 那几串落没落（`persisted_in_the_mount` 与挂载那一段写表逐条对应）：
+    /// 从抬 F 的第一个写起有一个持久的，就算可能落了一部分。
+    ///
+    /// # Panics
+    /// `persisted_in_the_mount` 比抬 F 第一个写的下标还短：持久集合与写表对不上。
+    #[must_use]
+    pub fn at_a_second_crash(self, persisted_in_the_mount: &[bool]) -> FloorRaisesAtTheSecondCrash {
+        match self {
+            FloorRaisesInTheMountWrites::NotPushed => FloorRaisesAtTheSecondCrash::NoneLandedYet,
+            FloorRaisesInTheMountWrites::FirstWriteAt(first_write) => {
+                let persisted_from_the_floor_raises = persisted_in_the_mount
+                    .get(first_write..)
+                    .expect("持久集合与挂载那一段写表逐条对应，抬 F 的第一个写在表里");
+                if persisted_from_the_floor_raises.contains(&true) {
+                    FloorRaisesAtTheSecondCrash::SomeMayHaveLanded
+                } else {
+                    FloorRaisesAtTheSecondCrash::NoneLandedYet
+                }
+            }
+        }
+    }
+}
+
 /// 做的是哪个动作。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum MountTransparencyAction {
@@ -418,7 +485,7 @@
 /// 某一样为什么没判：条款没写这一格该是什么，不替它定红绿，只计数（交主 agent）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub enum FieldNotJudged {
-    /// 二次崩溃崩在暖机那一段、挂载推了抬 F：F 生效值落在哪一处条款没写。
+    /// 挂载推了抬 F、二次崩溃镜像上抬 F 的写有持久的（[`FloorRaisesAtTheSecondCrash::SomeMayHaveLanded`]）：F 生效值落在哪一处条款没写。
     FloorOfASecondCrashAfterTheFloorRaisesBegan,
     /// 二次崩溃镜像上的恢复可以合法地由记录重建暖机那一版（施加条数不为 0）：扫描报告的式子只写到挂载做成之后。
     JournalScanReportOfASecondCrashImage,
````

## `crates/singlefs-harness/src/reference_filesystem_translation.rs`

````diff
--- crates/singlefs-harness/src/reference_filesystem_translation.rs	1970-01-01 00:00:00.000000000 +0000
+++ crates/singlefs-harness/src/reference_filesystem_translation.rs	2026-10-02 18:08:52.381631994 +0000
@@ -0,0 +1,1028 @@
+//! RefFS（`singlefs-reference-filesystem`）的调用方一侧：把一段历史、每步的结局、崩溃时刻翻成 RefFS 的五样输入
+//! （`research/prompts/m3-formal-r6-main-verification.md` 第二节 X2 一行的白名单：操作结局、hi 与它的归属规则、回退目标 m、
+//! 每次恢复读出的内容、显式发布点），再把 RefFS 的判定交回调用点。依赖只许这一个方向：harness 档依赖 RefFS，RefFS 不依赖 harness
+//! （门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格 ⑤⑥⑦）。
+//!
+//! 照 r6 用户定案 H1-收严写：
+//! - 崩溃时刻按「切开当前段的那道屏障（或那次 FUA 写）由哪一步发出」归属（[`segment_closing_stream_indexes`]、
+//!   [`TranslatedHistory::crash_moment_in_segment`]）；只给 RefFS 用，O1 判官的归属照崩溃注入今天「段里第一个被扣下的写」不动。
+//! - m 表（根 → 版本号）由这里攒：实现写下的每一条根（发布调用、回退、可写挂载的写行与暖机、抬 F、卸载）映到写它时的现行版本，
+//!   现行版本含每次恢复读出、重新起头的那一份（冷启动恢复与抛弃最新根的那次挂载）。
+//!
+//! 一步历史在 RefFS 里是什么（第一版的操作集合：第一个文件、覆盖写、发布调用、回退；fsync 今天的历史里没有）：
+//! 发布第一个文件 = 建第一个文件 + 发布调用；覆盖写 = 覆盖 + 发布调用；零单元发布 = 发布调用；管理员回退 = 回退到 m；
+//! 可写挂载、抬 F 不改内容、不进 RefFS 的操作序列，它们写的根映到现行版本；冷启动 = 一次恢复（判读出的内容，以它重新起头）；
+//! 抛弃最新根的那次挂载 = 一次带读故障的恢复（不判，以读出的内容重新起头，r6 判决第二节 U2 一行）。
+//! 原样重发这一种结局今天的历史走不出来（入口报错历史就停），这里不产出它。
+
+use std::collections::BTreeMap;
+
+use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
+use singlefs_reference_filesystem::{
+    AllowedState, CrashNotAccepted, FileContent, Operation, OperationNotAccepted, OperationNumber,
+    OperationOutcome, ReferenceFileSystem, SubmittedOperation, VersionNumber, VisibleState,
+};
+
+use crate::fault_injection::InjectedFault;
+use crate::history::{
+    AppliedEffect, HarnessJudgement, HistoryEnding, HistoryOperation, HistoryRun,
+    HistoryStartingPoint, StepObservation, StepOutcome, StepPosition,
+};
+use crate::memory_pool::root_identity_written_by;
+use crate::model::{ModelDisagreement, ModelRootKey, ObservedReadBack};
+use crate::model_comparison::model_root_key;
+use crate::scenario::first_file_content;
+use crate::segments::{FixedGeometry, SegmentAfterOperation, SegmentClosingRule, StepKind};
+use crate::{RecordedOperationKind, RetainedOperation};
+
+/// RefFS 判红时记进失败观察的那一项的名字（与挂载透明、I-8.4 一样，不是池级 checker 的不变量，借违例那一栏带着走）。
+pub const REFERENCE_FILESYSTEM_VIOLATION_NAME: &str = "RefFS：恢复读出不在允许集合里";
+
+/// 调用方看到的一步怎么收尾，按 RefFS 的结局白名单分。
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum StepEndingSeenByTheTranslation {
+    /// 入口返回 Ok；起点段没有 [`AppliedEffect`]。这一步里有恢复（冷启动、抛弃最新根的那次挂载）时带着它读出的内容。
+    ReturnedOk {
+        effect: Option<AppliedEffect>,
+        read_out_of_the_recovery_in_the_step: Option<VisibleState>,
+    },
+    /// 入口在任何写之前拒（历史照判过的拒：模型核过拒之前一个写都没发）。
+    RefusedBeforeAnyWrite,
+    /// 入口返回了错误，写没写过不知道（故障注入那一步）：按「写了之后报错」算，两可。
+    ReturnedAnErrorAfterItMayHaveWritten,
+    /// 入口没返回（panic）：这一步在途。
+    NeverReturned,
+    /// 前提不成立，入口没调。
+    NotCalled,
+}
+
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct StepSeenByTheTranslation {
+    position: StepPosition,
+    operation: Option<HistoryOperation>,
+    ending: StepEndingSeenByTheTranslation,
+    /// 这一步跑完时录制流里有几步（与崩溃注入登记的「每一步跑完时流里的步数」同一个数）。
+    stream_length_after_the_step: usize,
+}
+
+/// 一段历史跑的过程中攒下的东西：起点、每一步的操作与结局、每一步跑完时流有多长。跑完之后 [`Self::translate`] 翻成 RefFS 的输入。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct ReferenceFileSystemTranslation {
+    starting_point: HistoryStartingPoint,
+    steps: Vec<StepSeenByTheTranslation>,
+}
+
+/// 读出的内容按 RefFS 的可见状态写；恢复失败时没有状态。
+fn visible_state_of(outcome: &RecoveryOutcome) -> Option<VisibleState> {
+    match outcome {
+        RecoveryOutcome::FileRead { content, .. } => {
+            Some(VisibleState::OneFile(FileContent(content.clone())))
+        }
+        RecoveryOutcome::NoFile { .. } => Some(VisibleState::NoFile),
+        RecoveryOutcome::Failed { .. } => None,
+    }
+}
+
+impl ReferenceFileSystemTranslation {
+    #[must_use]
+    pub fn for_a_history_starting(starting_point: HistoryStartingPoint) -> Self {
+        Self {
+            starting_point,
+            steps: Vec::new(),
+        }
+    }
+
+    /// 历史执行器的观察者里每一步调一次（起点也调）。这一步里有恢复（冷启动、抛弃最新根的那次挂载）时，在这一步跑完的镜像上
+    /// 再做一次只读恢复取读出的内容：这一步自己的那次恢复读的是同一份字节（冷启动只读不写；那次挂载之后最新的根是它暖机写的、
+    /// 内容照抄它恢复到的那一版）。
+    pub fn note_an_observed_step(
+        &mut self,
+        observation: &StepObservation<'_>,
+        stream_length_after_the_step: usize,
+    ) {
+        let read_out_of_the_recovery_in_the_step =
+            match (observation.operation, observation.outcome) {
+                (
+                    Some(HistoryOperation::ColdStartRecover),
+                    Some(StepOutcome::Applied(AppliedEffect::Recovered { .. })),
+                )
+                | (
+                    Some(HistoryOperation::CrashRecoveryAbandoningTheNewestRoot),
+                    Some(StepOutcome::Applied(AppliedEffect::Mounted { .. })),
+                ) => visible_state_of(&recover(observation.image, JournalPolicy::Consult).outcome),
+                (Some(_) | None, Some(_) | None) => None,
+            };
+        let ending = match observation.outcome {
+            None => StepEndingSeenByTheTranslation::ReturnedOk {
+                effect: None,
+                read_out_of_the_recovery_in_the_step,
+            },
+            Some(StepOutcome::Applied(effect)) => StepEndingSeenByTheTranslation::ReturnedOk {
+                effect: Some(effect.clone()),
+                read_out_of_the_recovery_in_the_step,
+            },
+            // 走到观察者的拒都是模型判过的拒：模型核过拒之前一个写都没发（`ModelDisagreementAspect::WroteBeforeRefusing`）。
+            Some(StepOutcome::Refused { .. }) => {
+                StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite
+            }
+            Some(StepOutcome::NotApplicable(_)) => StepEndingSeenByTheTranslation::NotCalled,
+        };
+        self.steps.push(StepSeenByTheTranslation {
+            position: observation.position,
+            operation: observation.operation.copied(),
+            ending,
+            stream_length_after_the_step,
+        });
+    }
+
+    /// 历史停在某一步（判红、panic、入口报错）时，那一步观察者没调：照 `run` 补上它。历史跑完的不补。
+    /// 入口返回 Ok 的照常翻（这一步里有恢复时读出的内容拿不到，那次恢复不进 RefFS）；入口报错的按「写了之后报错」算；
+    /// 没有结局（panic）的按在途算。
+    pub fn note_the_step_the_history_stopped_in(
+        &mut self,
+        history_operations: &[HistoryOperation],
+        run: &HistoryRun,
+        stream_length_at_the_end: usize,
+    ) {
+        let observation = match &run.ending {
+            HistoryEnding::Completed => return,
+            HistoryEnding::KnownRed { observation, .. }
+            | HistoryEnding::NewFinding { observation, .. } => observation,
+        };
+        let (operation, ending) = match observation.position {
+            StepPosition::StartingPoint => {
+                let ending = match (&observation.panic, &observation.harness_judgement) {
+                    (Some(_), Some(_) | None) => StepEndingSeenByTheTranslation::NeverReturned,
+                    (None, Some(HarnessJudgement::TheStartingPointReturnedAnError { .. })) => {
+                        StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
+                    }
+                    (
+                        None,
+                        Some(
+                            HarnessJudgement::AllocationGenerationIsNotThePublishTxg { .. }
+                            | HarnessJudgement::ColdStartRecoveryFailedOnCheckerGreenImage { .. },
+                        )
+                        | None,
+                    ) => StepEndingSeenByTheTranslation::ReturnedOk {
+                        effect: None,
+                        read_out_of_the_recovery_in_the_step: None,
+                    },
+                };
+                (None, ending)
+            }
+            StepPosition::Operation(step_index) => {
+                let ending = match (&observation.panic, run.outcomes.get(step_index)) {
+                    (Some(_), Some(_) | None) | (None, None) => {
+                        StepEndingSeenByTheTranslation::NeverReturned
+                    }
+                    (None, Some(StepOutcome::Applied(effect))) => {
+                        StepEndingSeenByTheTranslation::ReturnedOk {
+                            effect: Some(effect.clone()),
+                            read_out_of_the_recovery_in_the_step: None,
+                        }
+                    }
+                    (None, Some(StepOutcome::Refused { .. })) => {
+                        StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
+                    }
+                    (None, Some(StepOutcome::NotApplicable(_))) => {
+                        StepEndingSeenByTheTranslation::NotCalled
+                    }
+                };
+                (history_operations.get(step_index).copied(), ending)
+            }
+        };
+        self.steps.push(StepSeenByTheTranslation {
+            position: observation.position,
+            operation,
+            ending,
+            stream_length_after_the_step: stream_length_at_the_end,
+        });
+    }
+
+    /// 翻成 RefFS 的输入：每一步的事件、m 表（根 → 版本号）、每一段录制流由哪一道屏障关上。
+    /// `operations` 是整条录制流（开了内容保留，根槽写要字节），与攒步数时是同一条。
+    #[must_use]
+    pub fn translate(
+        &self,
+        operations: &[RetainedOperation],
+        geometry: &FixedGeometry,
+    ) -> TranslatedHistory {
+        let root_writes: Vec<(usize, ModelRootKey)> = operations
+            .iter()
+            .enumerate()
+            .filter_map(|(stream_index, retained)| {
+                match retained.operation.kind {
+                    RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {}
+                    RecordedOperationKind::WriteZeroes | RecordedOperationKind::Barrier => {
+                        return None
+                    }
+                }
+                if geometry.classify(&retained.operation) != StepKind::RootRecordFua {
+                    return None;
+                }
+                let bytes = retained.contents.as_ref()?;
+                let (instance, checkpoint_txg) = root_identity_written_by(bytes);
+                Some((stream_index, model_root_key(instance, checkpoint_txg)))
+            })
+            .collect();
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        let mut versions_of_the_roots: BTreeMap<ModelRootKey, VersionNumber> = BTreeMap::new();
+        let mut steps: Vec<TranslatedStep> = Vec::new();
+        let mut stopped: Option<TranslationStopped> = None;
+        let mut stream_length_before_the_step = 0usize;
+        for seen in &self.steps {
+            let version_before_the_step = model.newest_version();
+            let events = match self.events_of(seen, &versions_of_the_roots) {
+                Ok(events) => events,
+                Err(reason) => {
+                    stopped = Some(TranslationStopped {
+                        position: seen.position,
+                        verdict: StoppedVerdict::Abstained(reason),
+                    });
+                    steps.push(TranslatedStep::without_events(seen.position));
+                    break;
+                }
+            };
+            if let Err(verdict) = apply_every_event(&mut model, &events.events) {
+                stopped = Some(TranslationStopped {
+                    position: seen.position,
+                    verdict,
+                });
+                steps.push(TranslatedStep::without_events(seen.position));
+                break;
+            }
+            let version_after_the_step = model.newest_version();
+            let roots_of_the_step: Vec<ModelRootKey> = root_writes
+                .iter()
+                .filter(|(stream_index, _)| {
+                    (stream_length_before_the_step..seen.stream_length_after_the_step)
+                        .contains(stream_index)
+                })
+                .map(|(_, root)| *root)
+                .collect();
+            for (index_in_the_step, root) in roots_of_the_step.iter().enumerate() {
+                let is_the_last_root_of_the_step = index_in_the_step + 1 == roots_of_the_step.len();
+                let version = match events.new_version_is_written_by {
+                    NewVersionWrittenBy::TheLastRootOfTheStep if is_the_last_root_of_the_step => {
+                        version_after_the_step
+                    }
+                    NewVersionWrittenBy::TheLastRootOfTheStep => version_before_the_step,
+                    NewVersionWrittenBy::EveryRootOfTheStep => version_after_the_step,
+                };
+                versions_of_the_roots.insert(*root, version);
+            }
+            stream_length_before_the_step = seen.stream_length_after_the_step;
+            steps.push(TranslatedStep {
+                position: seen.position,
+                events: events.events,
+                crash_inside: events.crash_inside,
+                stream_length_after_the_step: seen.stream_length_after_the_step,
+            });
+        }
+        TranslatedHistory {
+            steps,
+            stopped,
+            versions_of_the_roots,
+            segment_closing_stream_indexes: segment_closing_stream_indexes(operations),
+        }
+    }
+
+    /// 一步在 RefFS 里的事件。回退要的 m 在 `versions_of_the_roots` 里查（实现报的回退根写在它之前那几步里）。
+    fn events_of(
+        &self,
+        seen: &StepSeenByTheTranslation,
+        versions_of_the_roots: &BTreeMap<ModelRootKey, VersionNumber>,
+    ) -> Result<EventsOfAStep, TranslationStopReason> {
+        let publish_with_content = |change: Option<Operation>| {
+            let publish_outcome = match &seen.ending {
+                StepEndingSeenByTheTranslation::ReturnedOk { .. }
+                | StepEndingSeenByTheTranslation::NeverReturned => OperationOutcome::Succeeded,
+                StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten => {
+                    OperationOutcome::FailedAfterWriting
+                }
+                StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite
+                | StepEndingSeenByTheTranslation::NotCalled => {
+                    return EventsOfAStep::nothing();
+                }
+            };
+            let mut events: Vec<ReferenceFileSystemEvent> = change
+                .into_iter()
+                .map(|operation| ReferenceFileSystemEvent::Submit {
+                    operation,
+                    outcome: OperationOutcome::Succeeded,
+                })
+                .collect();
+            events.push(ReferenceFileSystemEvent::Submit {
+                operation: Operation::ExplicitPublish,
+                outcome: publish_outcome,
+            });
+            EventsOfAStep {
+                events,
+                new_version_is_written_by: NewVersionWrittenBy::TheLastRootOfTheStep,
+                crash_inside: CrashInsideTheStep::Judged,
+            }
+        };
+        let Some(operation) = seen.operation else {
+            return Ok(match self.starting_point {
+                HistoryStartingPoint::AfterMakeFilesystem => EventsOfAStep::nothing(),
+                HistoryStartingPoint::AfterFirstFile => publish_with_content(Some(
+                    Operation::CreateTheFirstFile(FileContent(first_file_content())),
+                )),
+            });
+        };
+        Ok(match operation {
+            HistoryOperation::PublishFirstFile(content) => publish_with_content(Some(
+                Operation::CreateTheFirstFile(FileContent(content.bytes())),
+            )),
+            HistoryOperation::PublishOverwrite(content) => publish_with_content(Some(
+                Operation::OverwriteTheFile(FileContent(content.bytes())),
+            )),
+            HistoryOperation::PublishWithoutUnits => publish_with_content(None),
+            HistoryOperation::RollBackWhileMounted(_) => match &seen.ending {
+                StepEndingSeenByTheTranslation::ReturnedOk {
+                    effect: Some(AppliedEffect::RolledBack { target, .. }),
+                    ..
+                } => {
+                    let target_root = model_root_key(target.instance, target.checkpoint_txg);
+                    let Some(version) = versions_of_the_roots.get(&target_root) else {
+                        return Err(TranslationStopReason::TheRollbackTargetRootHasNoVersion {
+                            target: target_root,
+                        });
+                    };
+                    EventsOfAStep {
+                        events: vec![ReferenceFileSystemEvent::Submit {
+                            operation: Operation::RollBackToVersion(*version),
+                            outcome: OperationOutcome::Succeeded,
+                        }],
+                        new_version_is_written_by: NewVersionWrittenBy::TheLastRootOfTheStep,
+                        crash_inside: CrashInsideTheStep::Judged,
+                    }
+                }
+                StepEndingSeenByTheTranslation::ReturnedOk {
+                    effect:
+                        Some(
+                            AppliedEffect::Published { .. }
+                            | AppliedEffect::Mounted { .. }
+                            | AppliedEffect::RaisedFloor { .. }
+                            | AppliedEffect::Recovered { .. },
+                        )
+                        | None,
+                    ..
+                }
+                | StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
+                | StepEndingSeenByTheTranslation::NeverReturned => {
+                    // 实现只在 Ok 里报回退根（`AppliedEffect::RolledBack`）；没报就翻不出 m。
+                    return Err(TranslationStopReason::TheRollbackDidNotReportItsTarget);
+                }
+                StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite
+                | StepEndingSeenByTheTranslation::NotCalled => EventsOfAStep::nothing(),
+            },
+            // 可写挂载与抬 F 不改内容：不进 RefFS 的操作序列（H1 只让发布调用、fsync、回退抬 lo），它们写的根映到现行版本。
+            HistoryOperation::CloseAndMountWritable | HistoryOperation::RaiseRollbackFloor(_) => {
+                EventsOfAStep::nothing()
+            }
+            HistoryOperation::ColdStartRecover => match &seen.ending {
+                StepEndingSeenByTheTranslation::ReturnedOk {
+                    read_out_of_the_recovery_in_the_step: Some(read_out),
+                    ..
+                } => EventsOfAStep {
+                    events: vec![ReferenceFileSystemEvent::RecoverWithEveryOperationReturned(
+                        read_out.clone(),
+                    )],
+                    new_version_is_written_by: NewVersionWrittenBy::EveryRootOfTheStep,
+                    crash_inside: CrashInsideTheStep::Judged,
+                },
+                StepEndingSeenByTheTranslation::ReturnedOk {
+                    read_out_of_the_recovery_in_the_step: None,
+                    ..
+                }
+                | StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite
+                | StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
+                | StepEndingSeenByTheTranslation::NeverReturned
+                | StepEndingSeenByTheTranslation::NotCalled => EventsOfAStep::nothing(),
+            },
+            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot => {
+                let events = match &seen.ending {
+                    StepEndingSeenByTheTranslation::ReturnedOk {
+                        read_out_of_the_recovery_in_the_step: Some(read_out),
+                        ..
+                    } => vec![
+                        ReferenceFileSystemEvent::RestartAfterARecoveryThatAbandonedTheNewestRoot(
+                            read_out.clone(),
+                        ),
+                    ],
+                    StepEndingSeenByTheTranslation::ReturnedOk {
+                        read_out_of_the_recovery_in_the_step: None,
+                        ..
+                    }
+                    | StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite
+                    | StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
+                    | StepEndingSeenByTheTranslation::NeverReturned
+                    | StepEndingSeenByTheTranslation::NotCalled => Vec::new(),
+                };
+                // 这一步的恢复带读故障（藏起来的那几处），崩溃镜像上读不到那份故障：崩在它里面的状态 RefFS 不判。
+                EventsOfAStep {
+                    events,
+                    new_version_is_written_by: NewVersionWrittenBy::EveryRootOfTheStep,
+                    crash_inside: CrashInsideTheStep::NotJudgedBecauseItsRecoveryHadReadFaults,
+                }
+            }
+        })
+    }
+}
+
+/// 一步翻出来的 RefFS 事件与它写的根怎么映版本。
+struct EventsOfAStep {
+    events: Vec<ReferenceFileSystemEvent>,
+    new_version_is_written_by: NewVersionWrittenBy,
+    crash_inside: CrashInsideTheStep,
+}
+
+impl EventsOfAStep {
+    fn nothing() -> Self {
+        Self {
+            events: Vec::new(),
+            new_version_is_written_by: NewVersionWrittenBy::EveryRootOfTheStep,
+            crash_inside: CrashInsideTheStep::Judged,
+        }
+    }
+}
+
+/// 一步出了新版本时，这一步写的根里哪几条映到新版本：发布与回退只有最后一条根是它自己的（之前的是会话推的抬 F），
+/// 恢复在这一步的写之前（之后的写都是以它重新起头的挂载写的）。没出新版本时全映到现行版本，两种一样。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum NewVersionWrittenBy {
+    TheLastRootOfTheStep,
+    EveryRootOfTheStep,
+}
+
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum CrashInsideTheStep {
+    Judged,
+    NotJudgedBecauseItsRecoveryHadReadFaults,
+}
+
+/// 交给 RefFS 的一件事。
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum ReferenceFileSystemEvent {
+    Submit {
+        operation: Operation,
+        outcome: OperationOutcome,
+    },
+    /// 序列里的操作都返回过之后的一次恢复（冷启动）：判读出的内容，以它重新起头。
+    RecoverWithEveryOperationReturned(VisibleState),
+    /// 抛弃最新根的那次恢复：不判，以读出的内容重新起头。
+    RestartAfterARecoveryThatAbandonedTheNewestRoot(VisibleState),
+}
+
+/// 事件交进 RefFS 之后：进了序列的操作号，或者没有。
+enum EventApplied {
+    Entered(OperationNumber),
+    NothingEntered,
+}
+
+fn apply_event(
+    model: &mut ReferenceFileSystem,
+    event: &ReferenceFileSystemEvent,
+) -> Result<EventApplied, StoppedVerdict> {
+    match event {
+        ReferenceFileSystemEvent::Submit { operation, outcome } => {
+            match model.submit(operation.clone(), *outcome) {
+                Ok(SubmittedOperation::Entered(number)) => Ok(EventApplied::Entered(number)),
+                Ok(SubmittedOperation::DidNotHappen) => Ok(EventApplied::NothingEntered),
+                Err(refusal) => Err(StoppedVerdict::Abstained(
+                    TranslationStopReason::TheReferenceFileSystemDoesNotTakeTheOperation(refusal),
+                )),
+            }
+        }
+        ReferenceFileSystemEvent::RecoverWithEveryOperationReturned(read_out) => {
+            let crash = model.crash_after_every_operation_returned();
+            match model.crash_and_recover(crash, read_out.clone()) {
+                Ok(_) => Ok(EventApplied::NothingEntered),
+                Err(refusal) => Err(stopped_verdict_of_a_refused_recovery(refusal)),
+            }
+        }
+        ReferenceFileSystemEvent::RestartAfterARecoveryThatAbandonedTheNewestRoot(read_out) => {
+            let crash = model.crash_after_every_operation_returned();
+            match model
+                .restart_after_a_recovery_that_abandoned_the_newest_root(crash, read_out.clone())
+            {
+                Ok(()) => Ok(EventApplied::NothingEntered),
+                Err(refusal) => Err(stopped_verdict_of_a_refused_recovery(refusal)),
+            }
+        }
+    }
+}
+
+fn apply_every_event(
+    model: &mut ReferenceFileSystem,
+    events: &[ReferenceFileSystemEvent],
+) -> Result<Option<OperationNumber>, StoppedVerdict> {
+    let mut last_entered = None;
+    for event in events {
+        match apply_event(model, event)? {
+            EventApplied::Entered(number) => last_entered = Some(number),
+            EventApplied::NothingEntered => {}
+        }
+    }
+    Ok(last_entered)
+}
+
+fn stopped_verdict_of_a_refused_recovery(refusal: CrashNotAccepted) -> StoppedVerdict {
+    match refusal {
+        CrashNotAccepted::RecoveredStateOutsideTheAllowedSet { allowed, recovered } => {
+            StoppedVerdict::Red {
+                reason: format!(
+                    "历史里一次恢复读出 {}，允许集合 {}",
+                    describe_visible_state(&recovered),
+                    describe_allowed_states(&allowed.allowed)
+                ),
+            }
+        }
+        CrashNotAccepted::OperationAfterAFailureAfterWritingBeforeTheNextRecoveryIsNotInTheFirstVersion { .. } => {
+            StoppedVerdict::Abstained(TranslationStopReason::AnOperationAfterAFailureAfterWritingIsNotInTheFirstVersion)
+        }
+        CrashNotAccepted::CrashOperationIsOutsideTheOperationsSinceTheLastRecovery { .. } => {
+            panic!("历史里的恢复取的 hi 是「序列里的操作都返回过之后」那个号，不会越界：{refusal:?}")
+        }
+    }
+}
+
+/// 给人看：没有文件，或文件多少字节、末字节（内容可以有几万字节，不整份打）。
+fn describe_visible_state(state: &VisibleState) -> String {
+    match state {
+        VisibleState::NoFile => "没有文件".to_string(),
+        VisibleState::OneFile(FileContent(bytes)) => {
+            format!("文件 {} 字节（末字节 {:?}）", bytes.len(), bytes.last())
+        }
+    }
+}
+
+fn describe_allowed_states(allowed: &[AllowedState]) -> String {
+    let described: Vec<String> = allowed
+        .iter()
+        .map(|allowed_state| {
+            format!(
+                "S_{} = {}",
+                allowed_state.index.0,
+                describe_visible_state(&allowed_state.state)
+            )
+        })
+        .collect();
+    format!("{{{}}}", described.join("，"))
+}
+
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct TranslatedStep {
+    position: StepPosition,
+    events: Vec<ReferenceFileSystemEvent>,
+    crash_inside: CrashInsideTheStep,
+    stream_length_after_the_step: usize,
+}
+
+impl TranslatedStep {
+    /// 翻译停在这一步：它与之后的步都不交给 RefFS。
+    fn without_events(position: StepPosition) -> Self {
+        Self {
+            position,
+            events: Vec::new(),
+            crash_inside: CrashInsideTheStep::Judged,
+            stream_length_after_the_step: usize::MAX,
+        }
+    }
+}
+
+/// 翻译为什么停在某一步。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum TranslationStopReason {
+    /// 回退报的目标根不在 m 表里（m 表收这一池里写过根的每一个入口；查不到说明表漏收了一种入口）。
+    TheRollbackTargetRootHasNoVersion { target: ModelRootKey },
+    /// 回退没返回 Ok（报错或没返回）：实现只在 Ok 里报回退根，翻不出 m。
+    TheRollbackDidNotReportItsTarget,
+    /// RefFS 不收这一步（第一版不支持的操作，或回退目标不是一个版本）。
+    TheReferenceFileSystemDoesNotTakeTheOperation(OperationNotAccepted),
+    /// 一个操作写了之后报错、恢复之前又起了操作（RefFS 第一版不支持）。
+    AnOperationAfterAFailureAfterWritingIsNotInTheFirstVersion,
+}
+
+impl TranslationStopReason {
+    #[must_use]
+    pub fn name(&self) -> &'static str {
+        match self {
+            TranslationStopReason::TheRollbackTargetRootHasNoVersion { .. } => {
+                "回退目标根在 m 表里查不到"
+            }
+            TranslationStopReason::TheRollbackDidNotReportItsTarget => "回退没返回 Ok、没报目标根",
+            TranslationStopReason::TheReferenceFileSystemDoesNotTakeTheOperation(_) => {
+                "RefFS 不收这一步"
+            }
+            TranslationStopReason::AnOperationAfterAFailureAfterWritingIsNotInTheFirstVersion => {
+                "写后报错、恢复之前又起了操作"
+            }
+        }
+    }
+}
+
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum StoppedVerdict {
+    Abstained(TranslationStopReason),
+    /// 历史里一次恢复（冷启动）读出的内容 RefFS 就判红了：之后的崩溃都带着这一判。
+    Red {
+        reason: String,
+    },
+}
+
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct TranslationStopped {
+    position: StepPosition,
+    verdict: StoppedVerdict,
+}
+
+/// 崩溃落在哪：切开当前段的那道屏障（或那次 FUA 写）由哪一步发出；当前段到流尾都没关上的，算在全部步都返回之后。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum ReferenceFileSystemCrashMoment {
+    InsideStep(StepPosition),
+    AfterEveryTranslatedStep,
+}
+
+/// 每一段录制流由录制流里第几步关上（关上它的那道屏障或那次 FUA 写的下标）；流尾那一段没关上的是 `None`。
+/// 切法与 `memory_pool::writes_and_segments_with_stream_indexes` 同一条规则（[`SegmentClosingRule`]），段号一一对应。
+#[must_use]
+pub fn segment_closing_stream_indexes(operations: &[RetainedOperation]) -> Vec<Option<usize>> {
+    let mut closing_stream_indexes = Vec::new();
+    let mut closing_rule = SegmentClosingRule::default();
+    for (stream_index, retained) in operations.iter().enumerate() {
+        match closing_rule.after(&retained.operation) {
+            SegmentAfterOperation::Closes => closing_stream_indexes.push(Some(stream_index)),
+            SegmentAfterOperation::StaysOpen => {}
+        }
+    }
+    if closing_rule.current_segment_has_writes() {
+        closing_stream_indexes.push(None);
+    }
+    closing_stream_indexes
+}
+
+/// 关上这一段的那一步：第一个「跑完时流里的步数」大于关段下标的那一条登记。登记里找不到（关段的那一步没登记，
+/// 例如历史停下的那一步），或这一段没关上，都算在全部步都返回之后。
+#[must_use]
+pub fn crash_moment_of_a_segment_closed_at(
+    closing_stream_index: Option<usize>,
+    stream_length_after_each_step: &[(StepPosition, usize)],
+) -> ReferenceFileSystemCrashMoment {
+    let Some(closing_stream_index) = closing_stream_index else {
+        return ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep;
+    };
+    stream_length_after_each_step
+        .iter()
+        .find(|(_, stream_length)| *stream_length > closing_stream_index)
+        .map_or(
+            ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
+            |(position, _)| ReferenceFileSystemCrashMoment::InsideStep(*position),
+        )
+}
+
+/// RefFS 对一个崩溃状态投的票。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum ReferenceFileSystemVote {
+    Green,
+    Red { reason: String },
+    Abstained(ReferenceFileSystemAbstention),
+}
+
+/// RefFS 为什么不投票。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum ReferenceFileSystemAbstention {
+    /// 故障注入之后的状态带故障（每一档都是：写报错、写被吞、读报错、读回翻位、刷盘报错、刷盘被吞）：U1 的射程句
+    /// 「带故障的状态（说谎的设备、丢整块盘 C519）不适用」管全部带故障的状态，括号里是举例（调度记录
+    /// `records/2026-10-01-里程碑三形式化接入.md` 第 12 批「12-2 交回」那一条，主 agent 判故障注入里 RefFS 一律弃权）；O1 判官照旧。
+    TheStateCarriesAnInjectedFault,
+    /// 故障注入之后盘上本来就没有一版立得住（mkfs 自己报错），重开走读失败正是要的结局（`fault_injection::failed_reopen_is_the_right_answer`）。
+    NoVersionStandsAfterTheInjectedFault,
+    /// 翻译在崩溃之前（或崩溃落在的那一步）停了。
+    TheTranslationStoppedBeforeTheCrash(TranslationStopReason),
+    /// 崩溃落在一次带读故障的恢复那一步里（抛弃最新根的那次挂载）。
+    TheCrashIsInsideAStepWhoseRecoveryHadReadFaults,
+    /// 挂载途中的二次崩溃：第一次崩溃那一判 RefFS 没判绿，没有重新起头的起点。
+    TheFirstCrashWasNotJudgedGreen,
+    /// 写了之后报错的操作后面到崩溃为止又起了操作（RefFS 第一版不支持）。
+    AnOperationAfterAFailureAfterWritingIsNotInTheFirstVersion,
+}
+
+impl ReferenceFileSystemAbstention {
+    #[must_use]
+    pub fn name(&self) -> &'static str {
+        match self {
+            ReferenceFileSystemAbstention::TheStateCarriesAnInjectedFault => {
+                "带故障的状态，U1 射程外"
+            }
+            ReferenceFileSystemAbstention::NoVersionStandsAfterTheInjectedFault => {
+                "注入之后盘上没有一版立得住"
+            }
+            ReferenceFileSystemAbstention::TheTranslationStoppedBeforeTheCrash(reason) => {
+                reason.name()
+            }
+            ReferenceFileSystemAbstention::TheCrashIsInsideAStepWhoseRecoveryHadReadFaults => {
+                "崩在带读故障的那次恢复里"
+            }
+            ReferenceFileSystemAbstention::TheFirstCrashWasNotJudgedGreen => {
+                "第一次崩溃 RefFS 没判绿"
+            }
+            ReferenceFileSystemAbstention::AnOperationAfterAFailureAfterWritingIsNotInTheFirstVersion => {
+                "写后报错之后又起了操作"
+            }
+        }
+    }
+}
+
+impl ReferenceFileSystemVote {
+    /// 判红时记进失败观察违例那一栏的一项（[`REFERENCE_FILESYSTEM_VIOLATION_NAME`]）；别的票没有。
+    #[must_use]
+    pub fn violation(&self) -> Option<(&'static str, String)> {
+        match self {
+            ReferenceFileSystemVote::Red { reason } => {
+                Some((REFERENCE_FILESYSTEM_VIOLATION_NAME, reason.clone()))
+            }
+            ReferenceFileSystemVote::Green | ReferenceFileSystemVote::Abstained(_) => None,
+        }
+    }
+}
+
+/// RefFS 与 O1 判官两票合起来（G1 用户定案：任一红即红），按哪一边红分开计。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum JointVerdictOfTheReferenceFileSystemAndTheModel {
+    BothGreen,
+    OnlyTheReferenceFileSystemRed,
+    OnlyTheModelRed,
+    BothRed,
+    ReferenceFileSystemAbstainedAndTheModelGreen,
+    ReferenceFileSystemAbstainedAndTheModelRed,
+}
+
+impl JointVerdictOfTheReferenceFileSystemAndTheModel {
+    #[must_use]
+    pub fn of(vote: &ReferenceFileSystemVote, model: Option<&ModelDisagreement>) -> Self {
+        match (vote, model) {
+            (ReferenceFileSystemVote::Green, None) => Self::BothGreen,
+            (ReferenceFileSystemVote::Red { .. }, None) => Self::OnlyTheReferenceFileSystemRed,
+            (ReferenceFileSystemVote::Green, Some(_)) => Self::OnlyTheModelRed,
+            (ReferenceFileSystemVote::Red { .. }, Some(_)) => Self::BothRed,
+            (ReferenceFileSystemVote::Abstained(_), None) => {
+                Self::ReferenceFileSystemAbstainedAndTheModelGreen
+            }
+            (ReferenceFileSystemVote::Abstained(_), Some(_)) => {
+                Self::ReferenceFileSystemAbstainedAndTheModelRed
+            }
+        }
+    }
+
+    #[must_use]
+    pub const fn name(self) -> &'static str {
+        match self {
+            Self::BothGreen => "两者都绿",
+            Self::OnlyTheReferenceFileSystemRed => "只 RefFS 红",
+            Self::OnlyTheModelRed => "只 O1 红",
+            Self::BothRed => "两者都红",
+            Self::ReferenceFileSystemAbstainedAndTheModelGreen => "RefFS 弃权、O1 绿",
+            Self::ReferenceFileSystemAbstainedAndTheModelRed => "RefFS 弃权、O1 红",
+        }
+    }
+}
+
+/// 故障注入之后的重开要读出什么（`fault_injection::failed_reopen_is_the_right_answer` 的两种答案）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum WhatTheReopenMustRecover {
+    AVersionTheHistoryLeft,
+    NothingBecauseNoVersionStands,
+}
+
+/// 一次崩溃 RefFS 的判定：票，判绿时连同以读出的内容重新起头之后的 RefFS（挂载途中再崩一次时接着判）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct ReferenceFileSystemJudgementOfACrash {
+    pub vote: ReferenceFileSystemVote,
+    pub restarted: Option<ReferenceFileSystem>,
+}
+
+/// 翻好的一段历史：每一步的 RefFS 事件、翻译停在哪、m 表、每一段由哪一道屏障关上。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct TranslatedHistory {
+    steps: Vec<TranslatedStep>,
+    stopped: Option<TranslationStopped>,
+    versions_of_the_roots: BTreeMap<ModelRootKey, VersionNumber>,
+    segment_closing_stream_indexes: Vec<Option<usize>>,
+}
+
+/// 恢复读回的东西按 RefFS 的可见状态写；读失败的交回失败那一句。
+fn read_out_of(read_back: &ObservedReadBack) -> Result<VisibleState, String> {
+    match read_back {
+        ObservedReadBack::FileRead { content, .. } => {
+            Ok(VisibleState::OneFile(FileContent(content.clone())))
+        }
+        ObservedReadBack::NoFile { .. } => Ok(VisibleState::NoFile),
+        ObservedReadBack::Failed { what } => Err(what.clone()),
+    }
+}
+
+/// 在 `model` 上判一次崩溃（落在第 `crash_falls_in` 个操作里）之后恢复读回的东西。
+fn judge_the_recovery(
+    mut model: ReferenceFileSystem,
+    crash_falls_in: OperationNumber,
+    read_back: &ObservedReadBack,
+) -> ReferenceFileSystemJudgementOfACrash {
+    let recovered = match read_out_of(read_back) {
+        Ok(recovered) => recovered,
+        Err(what) => {
+            return ReferenceFileSystemJudgementOfACrash {
+                vote: ReferenceFileSystemVote::Red {
+                    reason: format!("恢复失败、没有读出的状态（{what}）"),
+                },
+                restarted: None,
+            }
+        }
+    };
+    match model.crash_and_recover(crash_falls_in, recovered) {
+        Ok(_) => ReferenceFileSystemJudgementOfACrash {
+            vote: ReferenceFileSystemVote::Green,
+            restarted: Some(model),
+        },
+        Err(CrashNotAccepted::RecoveredStateOutsideTheAllowedSet { allowed, recovered }) => {
+            ReferenceFileSystemJudgementOfACrash {
+                vote: ReferenceFileSystemVote::Red {
+                    reason: format!(
+                        "崩溃落在第 {} 个操作里，读出 {}，允许集合 {}",
+                        crash_falls_in.0,
+                        describe_visible_state(&recovered),
+                        describe_allowed_states(&allowed.allowed)
+                    ),
+                },
+                restarted: None,
+            }
+        }
+        Err(CrashNotAccepted::OperationAfterAFailureAfterWritingBeforeTheNextRecoveryIsNotInTheFirstVersion { .. }) => {
+            ReferenceFileSystemJudgementOfACrash {
+                vote: ReferenceFileSystemVote::Abstained(
+                    ReferenceFileSystemAbstention::AnOperationAfterAFailureAfterWritingIsNotInTheFirstVersion,
+                ),
+                restarted: None,
+            }
+        }
+        Err(refusal @ CrashNotAccepted::CrashOperationIsOutsideTheOperationsSinceTheLastRecovery { .. }) => {
+            panic!("hi 取的是这一步最后进序列的操作号或「都返回过之后」那个号，不会越界：{refusal:?}")
+        }
+    }
+}
+
+/// 挂载途中的二次崩溃：第一次崩溃判绿、以它读出的内容重新起头之后，挂载不进 RefFS 的操作序列，二次崩溃落在「都返回过之后」。
+/// 第一次没判绿（没有重新起头的 RefFS）的弃权。
+#[must_use]
+pub fn judge_a_second_crash_after_the_restart(
+    first_crash: &ReferenceFileSystemJudgementOfACrash,
+    read_back: &ObservedReadBack,
+) -> ReferenceFileSystemVote {
+    let Some(restarted) = &first_crash.restarted else {
+        return ReferenceFileSystemVote::Abstained(
+            ReferenceFileSystemAbstention::TheFirstCrashWasNotJudgedGreen,
+        );
+    };
+    let crash = restarted.crash_after_every_operation_returned();
+    judge_the_recovery(restarted.clone(), crash, read_back).vote
+}
+
+/// 这一档注入之后的状态在不在 RefFS 的射程外（[`ReferenceFileSystemAbstention::TheStateCarriesAnInjectedFault`]）。
+/// 每一档单列一臂：加一档注入时编译器逼着在这里定它，不跟着别的档一起落进射程里。
+const fn the_state_after_the_fault_is_outside_the_reference_filesystem_range(
+    fault: InjectedFault,
+) -> bool {
+    match fault {
+        InjectedFault::WriteFails => true,
+        InjectedFault::WriteIsSwallowed => true,
+        InjectedFault::ReadFails => true,
+        InjectedFault::ReadReturnsCorruptedBytes { .. } => true,
+        InjectedFault::BarrierFails => true,
+        InjectedFault::BarrierIsSwallowed => true,
+    }
+}
+
+impl TranslatedHistory {
+    /// 录制流切出的段数：调用方核它与自己切的段数相同（同一条流、同一条规则）。
+    #[must_use]
+    pub fn segment_count(&self) -> usize {
+        self.segment_closing_stream_indexes.len()
+    }
+
+    /// 崩在第 `segment_index` 段里的崩溃落在哪一步：关上这一段的那一步。
+    ///
+    /// # Panics
+    /// 段号超出这条流切出的段数（调用方的段与这里不是同一条流切的）。
+    #[must_use]
+    pub fn crash_moment_in_segment(&self, segment_index: usize) -> ReferenceFileSystemCrashMoment {
+        let closing_stream_index = *self
+            .segment_closing_stream_indexes
+            .get(segment_index)
+            .expect("段号来自同一条录制流、同一条切段规则");
+        let stream_length_after_each_step: Vec<(StepPosition, usize)> = self
+            .steps
+            .iter()
+            .map(|step| (step.position, step.stream_length_after_the_step))
+            .collect();
+        crash_moment_of_a_segment_closed_at(closing_stream_index, &stream_length_after_each_step)
+    }
+
+    /// m 表里这条根的版本号；不在表里是 None。
+    #[must_use]
+    pub fn version_of_the_root(&self, root: ModelRootKey) -> Option<VersionNumber> {
+        self.versions_of_the_roots.get(&root).copied()
+    }
+
+    /// 从头把事件交给一个新的 RefFS，到崩溃为止；交回它与 hi。崩溃落在某一步里时 hi 是那一步最后进序列的操作，
+    /// 那一步一个操作都没进序列（挂载、抬 F）时是「都返回过之后」那个号。
+    fn replay_up_to(
+        &self,
+        moment: ReferenceFileSystemCrashMoment,
+    ) -> Result<(ReferenceFileSystem, OperationNumber), ReferenceFileSystemVote> {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        for step in &self.steps {
+            if let Some(stopped) = self
+                .stopped
+                .as_ref()
+                .filter(|stopped| stopped.position == step.position)
+            {
+                return Err(match &stopped.verdict {
+                    StoppedVerdict::Abstained(reason) => ReferenceFileSystemVote::Abstained(
+                        ReferenceFileSystemAbstention::TheTranslationStoppedBeforeTheCrash(
+                            reason.clone(),
+                        ),
+                    ),
+                    StoppedVerdict::Red { reason } => ReferenceFileSystemVote::Red {
+                        reason: reason.clone(),
+                    },
+                });
+            }
+            let crash_is_inside_this_step =
+                moment == ReferenceFileSystemCrashMoment::InsideStep(step.position);
+            if crash_is_inside_this_step
+                && step.crash_inside == CrashInsideTheStep::NotJudgedBecauseItsRecoveryHadReadFaults
+            {
+                return Err(ReferenceFileSystemVote::Abstained(
+                    ReferenceFileSystemAbstention::TheCrashIsInsideAStepWhoseRecoveryHadReadFaults,
+                ));
+            }
+            let last_entered = apply_every_event(&mut model, &step.events)
+                .expect("翻译时同一串事件交给同一个起点的 RefFS 收过");
+            if crash_is_inside_this_step {
+                let crash_falls_in =
+                    last_entered.unwrap_or_else(|| model.crash_after_every_operation_returned());
+                return Ok((model, crash_falls_in));
+            }
+        }
+        let crash_falls_in = model.crash_after_every_operation_returned();
+        Ok((model, crash_falls_in))
+    }
+
+    /// 一次崩溃：照 `moment` 交事件到崩溃为止，判恢复读回的东西。
+    #[must_use]
+    pub fn judge_a_crash(
+        &self,
+        moment: ReferenceFileSystemCrashMoment,
+        read_back: &ObservedReadBack,
+    ) -> ReferenceFileSystemJudgementOfACrash {
+        match self.replay_up_to(moment) {
+            Ok((model, crash_falls_in)) => judge_the_recovery(model, crash_falls_in, read_back),
+            Err(vote) => ReferenceFileSystemJudgementOfACrash {
+                vote,
+                restarted: None,
+            },
+        }
+    }
+
+    /// 故障注入之后的重开：历史停下之后崩（崩溃落在「都返回过之后」）。注入之后的状态在 RefFS 射程外的弃权
+    /// （[`the_state_after_the_fault_is_outside_the_reference_filesystem_range`]，今天每一档都是），
+    /// 盘上本来没有一版立得住的弃权（重开走读失败是对的，O1 一侧同样不判）。
+    #[must_use]
+    pub fn judge_a_reopen_after_an_injected_fault(
+        &self,
+        fault: InjectedFault,
+        what_the_reopen_must_recover: WhatTheReopenMustRecover,
+        read_back: &ObservedReadBack,
+    ) -> ReferenceFileSystemVote {
+        if the_state_after_the_fault_is_outside_the_reference_filesystem_range(fault) {
+            return ReferenceFileSystemVote::Abstained(
+                ReferenceFileSystemAbstention::TheStateCarriesAnInjectedFault,
+            );
+        }
+        match what_the_reopen_must_recover {
+            WhatTheReopenMustRecover::NothingBecauseNoVersionStands => {
+                ReferenceFileSystemVote::Abstained(
+                    ReferenceFileSystemAbstention::NoVersionStandsAfterTheInjectedFault,
+                )
+            }
+            WhatTheReopenMustRecover::AVersionTheHistoryLeft => {
+                self.judge_a_crash(
+                    ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
+                    read_back,
+                )
+                .vote
+            }
+        }
+    }
+}
````

## `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`

````diff
--- crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs	2026-10-02 01:43:55.133024724 +0000
+++ crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs	2026-10-02 16:17:24.607320573 +0000
@@ -1269,6 +1269,94 @@
     );
 }
 
+/// 同一个槽距 8192 的池、盘 0 的槽 0 自证不过，再在盘 0 的偏移 4096 放一份自述槽距 4096、世代号 9 的系统配置
+/// （盘 0 槽 1 的拷贝改这两处、重封整槽校验和）：借盘 1 记的槽距（②，2026-09-14 用户定甲）只在 8192 那一档找槽 1，
+/// 读到 mkfs 写的世代号 1；逐档（③，全池槽 0 都不可用时才走，C651 第一轮判决）会在 4096 那一档收下世代号更大的那一份。
+/// 两档在这个取样点上结局不同，删掉 ② 落到 ③ 的变异在这里红（上一条用例 ② 与 ③ 同值，分不开）。
+#[test]
+fn a_device_whose_slot_zero_does_not_verify_takes_slot_one_only_at_the_borrowed_spacing_not_a_newer_slot_at_another_spacing(
+) {
+    const SPACING: u32 = 8192;
+    const ANOTHER_SPACING: u32 = 4096;
+    const BYTE_IN_THE_RESERVED_AREA_OF_A_SYSTEM_CONFIGURATION_SLOT: usize = 2048;
+    const SYSTEM_CONFIGURATION_SLOT_BYTES: usize = 4096;
+    const SYSTEM_CONFIGURATION_CHECKSUM_OFFSET: usize = 155;
+    /// 槽世代号 8 字节在 147（fsid 102 起 16 字节、之后 20 + 1、本盘设备号 4、设备数 4；`singlefs_checker::check_system_configuration_slot`），
+    /// 固定结构槽距 4 字节在 429（`singlefs_checker::image::geometry_of`）。
+    const SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET: usize = 147;
+    const SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET: usize = 429;
+    const GENERATION_OF_THE_SLOT_AT_ANOTHER_SPACING: u64 = 9;
+    let mut make_filesystem_parameters = parameters();
+    make_filesystem_parameters
+        .geometry
+        .minimum_input_output_bytes = SPACING;
+    make_filesystem_parameters
+        .geometry
+        .fixed_structure_slot_spacing = SPACING;
+    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [0u32, 1]
+        .into_iter()
+        .map(|device| {
+            (
+                DeviceIdentity(device),
+                SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
+            )
+        })
+        .collect();
+    make_filesystem(&make_filesystem_parameters, &mut devices).expect("槽距 8192 的池");
+    let mut image = MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.image.clone()))
+            .collect(),
+        device_size_in_bytes: IMAGE_BYTES,
+    };
+    let mut slot_zero = read_bytes(&image, 0, 0, SYSTEM_CONFIGURATION_SLOT_BYTES);
+    slot_zero[BYTE_IN_THE_RESERVED_AREA_OF_A_SYSTEM_CONFIGURATION_SLOT] ^= 0xff;
+    write_bytes(&mut image, 0, 0, &slot_zero);
+    let mut slot_at_another_spacing = read_bytes(
+        &image,
+        0,
+        u64::from(SPACING),
+        SYSTEM_CONFIGURATION_SLOT_BYTES,
+    );
+    slot_at_another_spacing[SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET
+        ..SYSTEM_CONFIGURATION_SLOT_GENERATION_OFFSET + 8]
+        .copy_from_slice(&GENERATION_OF_THE_SLOT_AT_ANOTHER_SPACING.to_le_bytes());
+    slot_at_another_spacing
+        [SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET..SYSTEM_CONFIGURATION_SLOT_SPACING_OFFSET + 4]
+        .copy_from_slice(&ANOTHER_SPACING.to_le_bytes());
+    let digest = wide_checksum_with_field_zeroed(
+        &slot_at_another_spacing,
+        SYSTEM_CONFIGURATION_SLOT_BYTES,
+        SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
+    );
+    slot_at_another_spacing
+        [SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32]
+        .copy_from_slice(&digest);
+    write_bytes(
+        &mut image,
+        0,
+        u64::from(ANOTHER_SPACING),
+        &slot_at_another_spacing,
+    );
+    let verified = singlefs_checker::image::verified_system_configuration_slots(&image);
+    let generations_and_spacings_on_device_zero: Vec<(u64, u64)> = verified
+        .iter()
+        .find(|(identity, _)| *identity == 0)
+        .map(|(_, slots)| {
+            slots
+                .iter()
+                .map(|(view, geometry)| (view.slot_generation, geometry.slot_spacing))
+                .collect()
+        })
+        .unwrap_or_default();
+    assert_eq!(
+        generations_and_spacings_on_device_zero,
+        vec![(1, u64::from(SPACING))],
+        "盘 0 的槽 1 只在盘 1 记的槽距 8192 处找，取 mkfs 写的世代号 1；4096 处那一份世代号 9 不收"
+    );
+}
+
 // ─── ⑧ 新立的「中央映射条目的 key 与它指的单元头相符」（I-1.11（映射 key 与单元头相符）） ───
 
 /// 映射 key 的字段序（类标签, 出生树, 出生 txg, 实例代号, 尾段 6 字节），按字段比无符号整数（D8（核心索引结构） 已定项 11）。
````

## `crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs`

````diff
--- crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs	2026-10-02 10:36:34.486837922 +0000
+++ crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs	2026-10-02 18:08:49.852021196 +0000
@@ -6,6 +6,7 @@
 //! journal 环装不下 F 条记录时 mkfs 在任何写之前拒掉（I-8.1（环几何够大）、D23（journal 的角色与格式） 已定项 18 / 已定项 19）。
 //! 盘都是内存稀疏盘（`singlefs_harness::memory_pool::SparseBlockDevice`），恢复、只读挂载与各个读者读的是 `MemoryPool`。
 
+use singlefs_checker::image::chosen_system_configurations;
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
 };
@@ -588,6 +589,7 @@
 
 /// 逐档试只在全池没有一个槽 0 可择时才走：只有盘 0 的槽 0 自证不过、盘 1 的槽 0 可择（记着槽距 4096），盘 0 的槽 1 照旧按
 /// 借来的 4096 找，盘 0 上 8192 处那份世代号 5 的旧槽读都不读（改前改后都绿）。盘 0 不借、也逐档试，择到的就是那份旧槽。
+/// 池级 checker 在同一份镜像上同样借、盘 0 同样择 4096 处世代号 1（代码轮第二轮判决 K4-1：恢复与 checker 两臂在同一份镜像上各问一次）。
 #[test]
 fn a_device_whose_slot_zero_fails_borrows_the_spacing_another_slot_zero_records_instead_of_searching(
 ) {
@@ -611,6 +613,18 @@
         (DeviceIdentity(0), 4096, 1),
         "盘 0 按盘 1 槽 0 记的 4096 找到本池的槽 1，不去试 8192"
     );
+    // 池级 checker 在同一份镜像上同一个读法（`singlefs_checker::image::system_configuration_slot_readings` 借池里第一块可用槽 0 记的槽距）：
+    // 盘 0 择 4096 处世代号 1，不择 8192 处世代号 5 的旧槽（代码轮第二轮判决 K4-1）。
+    let chosen_by_the_checker_on_device_zero = chosen_system_configurations(&image)
+        .into_iter()
+        .find(|(device, _)| *device == 0)
+        .and_then(|(_, chosen)| chosen)
+        .map(|(view, geometry)| (geometry.slot_spacing, view.slot_generation));
+    assert_eq!(
+        chosen_by_the_checker_on_device_zero,
+        Some((4096, 1)),
+        "checker 盘 0 也借盘 1 槽 0 记的 4096，择到本池的槽 1，不落到逐档去择 8192 处世代号 5 的旧槽"
+    );
 }
 
 /// 世代号相同时取偏移小的那一档（与槽 0、槽 1 世代号相同取槽 0 同一个次序）：槽距 4096 的池，8192 处躺着一份世代号同为 1、
````

## `crates/singlefs-harness/tests/fault_injection_fast_tier.rs`

````diff
--- crates/singlefs-harness/tests/fault_injection_fast_tier.rs	2026-10-02 03:16:50.027789898 +0000
+++ crates/singlefs-harness/tests/fault_injection_fast_tier.rs	2026-10-02 16:51:29.037691950 +0000
@@ -27,7 +27,7 @@
     FaultDeviceSelector, FaultInjectingBlockDevice, FaultInjectionCampaign, FaultInjectionReport,
     FaultInjectionTally, FaultInjectionWorkerThreads, FaultOccurrence, FaultOutcome,
     FaultPlacement, FaultSchedule, FaultedSegment, FaultedSegmentKind, FlippedBit, InjectedFault,
-    SharedFaultPlan,
+    KnownRedAtAFault, KnownRedFormAtAFault, KnownRedFormOnlyAfterAFault, SharedFaultPlan,
 };
 use singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
 use singlefs_harness::history::{
@@ -792,6 +792,90 @@
     }
 }
 
+/// 快档在 2026-10-03 落进这一形的那一段：种子基 + 7（7463871032432355120），快档的规模与比重。
+const SEED_WHERE_MAKE_FILESYSTEM_FAILS_BEFORE_ANY_SYSTEM_CONFIGURATION_SLOT_LANDS: u64 =
+    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 7;
+
+/// 用户 2026-10-03 定：mkfs 在写到系统配置之前失败的盘面就是全池皆废，I-8.5（tail 槽有效集非空）照判红，红是对的；
+/// 故障注入的「已知红」清单加这一类。快档在种子 7463871032432355120 上摆的那一个注入点（整池第 1 次 barrier 报错，落在 mkfs 第一道屏障上，
+/// 那一刻 mkfs 只清过零、写过实例表与树表的创世单元，系统配置槽一次都没写）原样重跑：注入之后的镜像上只红 I-8.5、
+/// 重开走读失败，判法记成那一条「已知红」，不记新发现。
+///
+/// 判别力自证：判法改回不认这一类（`known_red_form_only_after_a_fault` 恒交回 None，`crates/mutations.tsv` 同名那一条），
+/// 这一次落成新发现 `CheckerViolations { invariants: ["I-8.5"] }`，这里红。
+#[test]
+fn make_filesystem_failing_at_its_first_barrier_before_any_system_configuration_slot_lands_ends_known_red_not_a_new_finding(
+) {
+    let history = generate_history_with_weights(
+        HistorySeed(SEED_WHERE_MAKE_FILESYSTEM_FAILS_BEFORE_ANY_SYSTEM_CONFIGURATION_SLOT_LANDS),
+        FAST_TIER_OPERATIONS_PER_HISTORY,
+        &GenerationWeights::BROAD,
+    );
+    let injection = inject_one_fault_into_the_segment(
+        &history,
+        CHECKED_AFTER_EVERY_STEP,
+        InjectedFault::BarrierFails,
+        FaultedSegment::TheStartingPoint,
+        1,
+    );
+    let rendered = injection.tally.render();
+    print_uncaptured(&format!(
+        "── mkfs 第一道屏障报错：全池皆废、只红 I-8.5 ──\n{rendered}"
+    ));
+    let expected_drawn = DrawnFault {
+        fault: InjectedFault::BarrierFails,
+        call_ordinal: 1,
+        segment: FaultedSegment::TheStartingPoint,
+    };
+    assert_eq!(
+        injection.drawn_faults,
+        vec![expected_drawn],
+        "注入点是整池第 1 次 barrier、落在起点段\n{rendered}"
+    );
+    // 停下的是 mkfs 自己，重开走读失败而盘上本来就没有一版立得住：这一形的前两样事实真的在。
+    assert_eq!(
+        injection.tally.starting_point_failures,
+        BTreeMap::from([(StartingPointStep::MakeFilesystem.name(), 1)]),
+        "起点段里把错交回来的是 mkfs\n{rendered}"
+    );
+    assert_eq!(
+        injection
+            .tally
+            .reopens_that_correctly_found_nothing_to_recover,
+        1,
+        "重开走读失败、盘上本来就没有一版立得住\n{rendered}"
+    );
+    assert!(
+        injection.new_findings.is_empty(),
+        "mkfs 在系统配置槽落盘之前报错、只红 I-8.5 是认下的「已知红」，不是新发现：{:?}\n{rendered}",
+        injection
+            .new_findings
+            .iter()
+            .map(|finding| &finding.signature)
+            .collect::<Vec<_>>()
+    );
+    let expected_form = KnownRedFormAtAFault::OnlyAfterAFault(
+        KnownRedFormOnlyAfterAFault::MakeFilesystemFailedBeforeAnySystemConfigurationSlotLanded,
+    );
+    assert_eq!(
+        injection.known_red_hits,
+        vec![KnownRedAtAFault {
+            seed: HistorySeed(
+                SEED_WHERE_MAKE_FILESYSTEM_FAILS_BEFORE_ANY_SYSTEM_CONFIGURATION_SLOT_LANDS
+            ),
+            form: expected_form,
+            drawn: expected_drawn,
+        }],
+        "这一次记成「mkfs 没写下系统配置槽」那一条已知红\n{rendered}"
+    );
+    assert_eq!(
+        injection.tally.faults_ending_known_red,
+        BTreeMap::from([(expected_form, 1)]),
+        "计数里以这一条已知红收尾一次\n{rendered}"
+    );
+    assert_eq!(injection.tally.faults_that_panicked, 0, "{rendered}");
+}
+
 /// 线程数换了，报告逐字相同：计数按片的次序相加、新发现按种子从小到大留第一个
 /// （`implementation-workflow.md`「测试与崩溃检测优先多线程」的「合并要确定」）。
 #[test]
````

## `crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs`

````diff
--- crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs	2026-10-02 14:44:16.997412752 +0000
+++ crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs	2026-10-02 18:08:52.381631994 +0000
@@ -21,8 +21,9 @@
 };
 use singlefs_harness::mount_transparency::{
     judge_mount_transparency, read_recovery_fields, ComparedVersion, FieldNotJudged, FieldOutcome,
-    FloorRaisedByTheMount, FloorRaisesAtTheSecondCrash, MountTransparency, MountTransparencyAction,
-    MountTransparencyField, PosixContentRead, RecoveryFieldsRead, VersionFieldsRead,
+    FloorRaisedByTheMount, FloorRaisesAtTheSecondCrash, FloorRaisesInTheMountWrites,
+    MountTransparency, MountTransparencyAction, MountTransparencyField, PosixContentRead,
+    RecoveryFieldsRead, VersionFieldsRead,
 };
 use singlefs_harness::segments::StepKind;
 use singlefs_harness::{RecordingBlockDevice, SharedStream};
@@ -308,6 +309,62 @@
     }
 }
 
+/// 挂载会推抬 F 的那块小盘上挂载一次，二次崩溃摆在抬 F 第一个写的两侧（界由崩溃注入用的同一个分法 [`FloorRaisesInTheMountWrites`] 取）。
+struct SecondCrashesOnBothSidesOfTheFirstFloorRaiseWrite {
+    /// 界之前那几个写的种类：取号、写行、暖机连同暖机末尾的系统配置轮换。
+    kinds_of_the_writes_before_the_floor_raises: Vec<StepKind>,
+    /// 界之前的写全落、抬 F 的写一个没落。
+    after_the_warm_up: JudgedScenario,
+    /// 再多落抬 F 的第一个写（一块盘的系统配置里带上了新 F）。
+    after_the_first_floor_raise_write: JudgedScenario,
+}
+
+/// 两态 F 那一样判不判，由 [`FloorRaisesInTheMountWrites::at_a_second_crash`] 按持久集合定（与崩溃注入同一个函数）。
+fn second_crashes_on_both_sides_of_the_first_floor_raise_write(
+) -> SecondCrashesOnBothSidesOfTheFirstFloorRaiseWrite {
+    let width = HistoryDeviceWidth::UnitAreaOf256Slots;
+    let crash_image = pool_where_the_mount_raises_the_floor();
+    let parameters = width.parameters();
+    let stream = SharedStream::retaining_contents();
+    let mut devices = devices_of(&crash_image, &parameters, &stream);
+    let mounted = mount(&parameters, &mut devices);
+    let operations = stream.retained_operations();
+    let (writes, _segments, stream_indexes) =
+        writes_and_segments_with_stream_indexes(&operations, &width.fixed_geometry());
+    let floor_raises = FloorRaisesInTheMountWrites::of_the_mount_output(&mounted.output, &writes);
+    let FloorRaisesInTheMountWrites::FirstWriteAt(first_floor_raise_write) = floor_raises else {
+        panic!("前提：这次挂载在写行与暖机之后推了抬 F：{floor_raises:?}");
+    };
+    let before_the_mount = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
+    let second_crash_with_the_first_writes_landed = |landed_writes: usize| {
+        let mut second_crash_image = crash_image.clone();
+        second_crash_image.apply(&operations[..stream_indexes[landed_writes]]);
+        let persisted_in_the_mount: Vec<bool> = (0..writes.len())
+            .map(|write_index| write_index < landed_writes)
+            .collect();
+        JudgedScenario {
+            before_the_mount: before_the_mount.clone(),
+            after_the_action: read_recovery_fields(
+                &second_crash_image,
+                ComparedVersion::TheRecoveredVersion,
+            ),
+            action: MountTransparencyAction::SecondCrashInsideTheMount(
+                floor_raises.at_a_second_crash(&persisted_in_the_mount),
+            ),
+        }
+    };
+    SecondCrashesOnBothSidesOfTheFirstFloorRaiseWrite {
+        kinds_of_the_writes_before_the_floor_raises: writes[..first_floor_raise_write]
+            .iter()
+            .map(|write| write.kind)
+            .collect(),
+        after_the_warm_up: second_crash_with_the_first_writes_landed(first_floor_raise_write),
+        after_the_first_floor_raise_write: second_crash_with_the_first_writes_landed(
+            first_floor_raise_write + 1,
+        ),
+    }
+}
+
 fn assert_holds(verdict: &MountTransparency, field: MountTransparencyField) {
     assert_eq!(
         verdict.outcome_of(field),
@@ -662,3 +719,56 @@
     compared_version_mut(&mut scenario.after_the_action).content = PosixContentRead::NoFile;
     assert_only_this_field_is_violated(&scenario.judge(), MountTransparencyField::PosixContent);
 }
+
+/// 代码轮第二轮判决 K2-1：挂载推了抬 F 时，暖机那一次发布的写崩出来的二次崩溃态照 r5 判决 U5 前半句判 F 不变，
+/// 抬 F 的第一个写起才算 F 不判（攻方量到 fms12 那一次挂载暖机那几段 4 107 态以前整段记成不判）。
+#[test]
+fn a_second_crash_after_the_warm_up_judges_the_floor_unchanged_and_one_after_the_first_floor_raise_write_leaves_it_unjudged(
+) {
+    let SecondCrashesOnBothSidesOfTheFirstFloorRaiseWrite {
+        kinds_of_the_writes_before_the_floor_raises,
+        after_the_warm_up,
+        after_the_first_floor_raise_write,
+    } = second_crashes_on_both_sides_of_the_first_floor_raise_write();
+    assert!(
+        kinds_of_the_writes_before_the_floor_raises
+            .iter()
+            .filter(|kind| **kind == StepKind::RootRecordFua)
+            .count()
+            >= 2
+            && kinds_of_the_writes_before_the_floor_raises.last()
+                == Some(&StepKind::SystemConfigurationSlot),
+        "抬 F 的界在暖机之后：界之前有写行与暖机两次发布的根槽写，紧挨着界的是暖机末尾的系统配置轮换：\
+         {kinds_of_the_writes_before_the_floor_raises:?}"
+    );
+    assert_eq!(
+        after_the_warm_up.action,
+        MountTransparencyAction::SecondCrashInsideTheMount(
+            FloorRaisesAtTheSecondCrash::NoneLandedYet
+        ),
+        "暖机全落、抬 F 的写一个没落：抬 F 一步都还没落"
+    );
+    assert_holds(
+        &after_the_warm_up.judge(),
+        MountTransparencyField::RollbackFloor,
+    );
+    assert_eq!(
+        after_the_first_floor_raise_write.action,
+        MountTransparencyAction::SecondCrashInsideTheMount(
+            FloorRaisesAtTheSecondCrash::SomeMayHaveLanded
+        ),
+        "抬 F 的第一个写落了：抬 F 那几串可能落了一部分"
+    );
+    let verdict = after_the_first_floor_raise_write.judge();
+    assert_eq!(
+        verdict.outcome_of(MountTransparencyField::RollbackFloor),
+        Some(&FieldOutcome::NotJudged(
+            FieldNotJudged::FloorOfASecondCrashAfterTheFloorRaisesBegan
+        )),
+        "抬 F 开始之后的 F 不判：{verdict:#?}"
+    );
+    assert!(
+        verdict.violated_fields().is_empty(),
+        "合法挂载途中再崩一次，一样都不许红：{verdict:#?}"
+    );
+}
````

## `crates/singlefs-harness/tests/reference_filesystem_translation_of_histories.rs`

````diff
--- crates/singlefs-harness/tests/reference_filesystem_translation_of_histories.rs	1970-01-01 00:00:00.000000000 +0000
+++ crates/singlefs-harness/tests/reference_filesystem_translation_of_histories.rs	2026-10-02 18:08:52.381701134 +0000
@@ -0,0 +1,327 @@
+//! RefFS 的调用方翻译层（`singlefs_harness::reference_filesystem_translation`）在真历史上的判定：r6 攻方第一节那三类
+//! （`research/prompts/m3-formal-r6-opus-output.md`「一、X1」：回退之后又崩溃、回退在持久之前就返回的坏实现、恢复之后再回退到挂载写的根），
+//! 合法的判绿、坏实现的判红；崩溃时刻按切开当前段的那道屏障由哪一步发出归属；故障注入的每一档 RefFS 都弃权（带故障的状态在 U1 射程外）。
+//! 这里不枚举崩溃状态：历史照常跑完，读回由用例按「合法实现会读出什么」「坏实现会读出什么」交入（坏实现在这一层造不出，
+//! 它读出的东西造得出——攻方副本上量过的那 9 态读回回退之前的内容）。
+
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_harness::fault_injection::{FlippedBit, InjectedFault};
+use singlefs_harness::history::{
+    execute_history_with, AppliedEffect, ContentChoice, ContentLength, GeneratedHistory,
+    HistoryDeviceWidth, HistoryEnding, HistoryExecution, HistoryOperation, HistorySeed,
+    HistoryStartingPoint, PerStepChecker, RollbackTargetChoice, StepOutcome, StepPosition,
+};
+use singlefs_harness::model::{ModelRootKey, ObservedReadBack};
+use singlefs_harness::model_comparison::model_root_key;
+use singlefs_harness::reference_filesystem_translation::{
+    crash_moment_of_a_segment_closed_at, judge_a_second_crash_after_the_restart,
+    segment_closing_stream_indexes, ReferenceFileSystemAbstention, ReferenceFileSystemCrashMoment,
+    ReferenceFileSystemTranslation, ReferenceFileSystemVote, TranslatedHistory,
+    WhatTheReopenMustRecover,
+};
+use singlefs_harness::scenario::first_file_content;
+use singlefs_harness::{RecordedOperation, RecordedOperationKind, RetainedOperation, SharedStream};
+use singlefs_reference_filesystem::VersionNumber;
+
+/// 一次覆盖写的内容：一个数据单元装得下的 201 字节（攻方副本 `small` 的同一个选择子）。
+fn overwrite_content(fill_seed: u64) -> ContentChoice {
+    ContentChoice {
+        length: ContentLength::InsideOneDataUnit { selector: 200 },
+        fill_seed,
+    }
+}
+
+/// 根环里次新的那条根：覆盖写之后是起点那次第一个文件的根。
+const RING_ROOT_BEFORE_THE_NEWEST: RollbackTargetChoice = RollbackTargetChoice::RingRoot {
+    index_from_newest: 1,
+};
+
+/// 照常跑完一段历史（不跑每步的 checker），边跑边攒 RefFS 的输入，再翻成 RefFS 的事件与 m 表。
+fn translated_run_of(operations: Vec<HistoryOperation>) -> (TranslatedHistory, Vec<StepOutcome>) {
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations,
+    };
+    let execution = HistoryExecution {
+        per_step_checker: PerStepChecker::Skipped,
+        device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
+        space_admission: SpaceAdmission::JudgedByTheFormula,
+    };
+    let stream = SharedStream::retaining_contents();
+    let mut translation =
+        ReferenceFileSystemTranslation::for_a_history_starting(history.starting_point);
+    let run = execute_history_with(&history, execution, &stream, &mut |observation| {
+        translation.note_an_observed_step(observation, stream.operation_count());
+    });
+    assert_eq!(run.ending, HistoryEnding::Completed, "这段历史照常跑完");
+    let translated = translation.translate(
+        &stream.retained_operations(),
+        &execution.device_width.fixed_geometry(),
+    );
+    (translated, run.outcomes)
+}
+
+/// 第 `step_index` 步是一次做成的回退：交回它报的目标根。
+fn rollback_target_of(outcomes: &[StepOutcome], step_index: usize) -> ModelRootKey {
+    match &outcomes[step_index] {
+        StepOutcome::Applied(AppliedEffect::RolledBack { target, .. }) => {
+            model_root_key(target.instance, target.checkpoint_txg)
+        }
+        other @ (StepOutcome::Applied(_)
+        | StepOutcome::Refused { .. }
+        | StepOutcome::NotApplicable(_)) => {
+            panic!("第 {step_index} 步要是一次做成的回退：{other:?}")
+        }
+    }
+}
+
+fn file_read_back(root: ModelRootKey, content: Vec<u8>) -> ObservedReadBack {
+    ObservedReadBack::FileRead { root, content }
+}
+
+/// 起点（第一个文件）→ 覆盖写 → 回退到根环里次新的那条（第一个文件那一版）。
+fn overwrite_then_roll_back_to_the_first_file() -> (TranslatedHistory, ModelRootKey) {
+    let (translated, outcomes) = translated_run_of(vec![
+        HistoryOperation::PublishOverwrite(overwrite_content(7)),
+        HistoryOperation::RollBackWhileMounted(RING_ROOT_BEFORE_THE_NEWEST),
+    ]);
+    let target = rollback_target_of(&outcomes, 1);
+    assert_eq!(
+        translated.version_of_the_root(target),
+        Some(VersionNumber(2)),
+        "第 1 版是空池，第 2 版是起点那次第一个文件"
+    );
+    (translated, target)
+}
+
+#[test]
+fn a_crash_after_a_returned_rollback_that_reads_the_rolled_back_content_is_green() {
+    let (translated, target) = overwrite_then_roll_back_to_the_first_file();
+    let judgement = translated.judge_a_crash(
+        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
+        &file_read_back(target, first_file_content()),
+    );
+    assert_eq!(
+        judgement.vote,
+        ReferenceFileSystemVote::Green,
+        "回退返回之后崩溃、读回回退目标：合法"
+    );
+}
+
+#[test]
+fn a_rollback_that_returned_before_its_root_was_durable_reads_back_the_content_before_it_and_is_red(
+) {
+    let (translated, target) = overwrite_then_roll_back_to_the_first_file();
+    let judgement = translated.judge_a_crash(
+        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
+        &file_read_back(target, overwrite_content(7).bytes()),
+    );
+    assert!(
+        matches!(judgement.vote, ReferenceFileSystemVote::Red { .. }),
+        "切开那一段的屏障在回退返回之后才发、读回回退之前的覆盖写：回退抬了 lo，判红：{:?}",
+        judgement.vote
+    );
+}
+
+#[test]
+fn a_crash_inside_a_rollback_may_read_back_the_content_before_it() {
+    let (translated, target) = overwrite_then_roll_back_to_the_first_file();
+    let judgement = translated.judge_a_crash(
+        ReferenceFileSystemCrashMoment::InsideStep(StepPosition::Operation(1)),
+        &file_read_back(target, overwrite_content(7).bytes()),
+    );
+    assert_eq!(
+        judgement.vote,
+        ReferenceFileSystemVote::Green,
+        "回退还在途：回退之前的覆盖写合法"
+    );
+}
+
+#[test]
+fn a_rollback_to_a_root_the_mount_wrote_after_a_cold_start_recovery_is_named_by_the_version_that_recovery_restarted_from(
+) {
+    let (translated, outcomes) = translated_run_of(vec![
+        HistoryOperation::PublishOverwrite(overwrite_content(7)),
+        HistoryOperation::ColdStartRecover,
+        HistoryOperation::CloseAndMountWritable,
+        HistoryOperation::PublishOverwrite(overwrite_content(9)),
+        HistoryOperation::RollBackWhileMounted(RING_ROOT_BEFORE_THE_NEWEST),
+    ]);
+    let target = rollback_target_of(&outcomes, 4);
+    assert_eq!(
+        translated.version_of_the_root(target),
+        Some(VersionNumber(4)),
+        "空池、第一个文件、第一次覆盖写、冷启动读出重新起头的那一份：挂载写的根映到第 4 版"
+    );
+    let judgement = translated.judge_a_crash(
+        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
+        &file_read_back(target, overwrite_content(7).bytes()),
+    );
+    assert_eq!(
+        judgement.vote,
+        ReferenceFileSystemVote::Green,
+        "回退到挂载写的根，之后读回第一次覆盖写的内容"
+    );
+}
+
+#[test]
+fn a_second_crash_inside_the_mount_that_reads_back_an_older_version_than_the_first_recovery_is_red()
+{
+    let (translated, outcomes) = translated_run_of(vec![HistoryOperation::PublishOverwrite(
+        overwrite_content(7),
+    )]);
+    assert!(matches!(outcomes[0], StepOutcome::Applied(_)), "覆盖写做成");
+    let root = model_root_key(InstanceGeneration(1), CheckpointTxg(4));
+    let first_crash = translated.judge_a_crash(
+        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
+        &file_read_back(root, overwrite_content(7).bytes()),
+    );
+    assert_eq!(
+        first_crash.vote,
+        ReferenceFileSystemVote::Green,
+        "第一次崩溃读回覆盖写"
+    );
+    let vote = judge_a_second_crash_after_the_restart(
+        &first_crash,
+        &file_read_back(root, first_file_content()),
+    );
+    assert!(
+        matches!(vote, ReferenceFileSystemVote::Red { .. }),
+        "挂载途中再崩一次，读回比第一次恢复读出的更旧：{vote:?}"
+    );
+}
+
+fn write_on_device_zero() -> RetainedOperation {
+    RetainedOperation {
+        operation: RecordedOperation {
+            device: DeviceIdentity(0),
+            kind: RecordedOperationKind::Write,
+            offset: DeviceOffsetInBytes(1 << 20),
+            length: 512,
+            content_hash: 0,
+        },
+        contents: Some(vec![0x5a; 512]),
+    }
+}
+
+fn barrier_on_device_zero() -> RetainedOperation {
+    RetainedOperation {
+        operation: RecordedOperation {
+            device: DeviceIdentity(0),
+            kind: RecordedOperationKind::Barrier,
+            offset: DeviceOffsetInBytes(0),
+            length: 0,
+            content_hash: 0,
+        },
+        contents: None,
+    }
+}
+
+#[test]
+fn the_crash_moment_is_the_step_that_issued_the_barrier_closing_the_segment_not_the_step_that_wrote_into_it(
+) {
+    // 第 0 步写了一次就返回（没发屏障），第 1 步发的屏障关上那一段：段里第一个被扣下的写在第 0 步里，切段的屏障是第 1 步发的。
+    let operations = vec![write_on_device_zero(), barrier_on_device_zero()];
+    let stream_length_after_each_step = [
+        (StepPosition::Operation(0), 1),
+        (StepPosition::Operation(1), 2),
+    ];
+    let closing = segment_closing_stream_indexes(&operations);
+    assert_eq!(
+        closing,
+        vec![Some(1)],
+        "一段，由录制流第 1 步（那道屏障）关上"
+    );
+    assert_eq!(
+        crash_moment_of_a_segment_closed_at(closing[0], &stream_length_after_each_step),
+        ReferenceFileSystemCrashMoment::InsideStep(StepPosition::Operation(1)),
+        "崩溃时刻归给发那道屏障的第 1 步：第 0 步那时已经返回"
+    );
+}
+
+/// 覆盖写返回之后注入 `fault`、重开读回覆盖写之前那一版（第一个文件）：RefFS 照判的话这是红（最后一次发布返回过，lo 抬到它），
+/// 弃权时它不投票。
+fn reopen_vote_after_reading_back_the_version_before_the_last_publish(
+    fault: InjectedFault,
+) -> ReferenceFileSystemVote {
+    let (translated, _outcomes) = translated_run_of(vec![HistoryOperation::PublishOverwrite(
+        overwrite_content(7),
+    )]);
+    let root = model_root_key(InstanceGeneration(1), CheckpointTxg(3));
+    translated.judge_a_reopen_after_an_injected_fault(
+        fault,
+        WhatTheReopenMustRecover::AVersionTheHistoryLeft,
+        &file_read_back(root, first_file_content()),
+    )
+}
+
+fn assert_no_reference_filesystem_vote_because_the_state_carries_the_fault(
+    vote: &ReferenceFileSystemVote,
+    fault: InjectedFault,
+) {
+    assert_eq!(
+        vote,
+        &ReferenceFileSystemVote::Abstained(
+            ReferenceFileSystemAbstention::TheStateCarriesAnInjectedFault
+        ),
+        "{} 之后的状态带故障、在 U1 射程外：RefFS 那一票弃权，O1 判官照旧",
+        fault.name()
+    );
+}
+
+#[test]
+fn the_reopen_after_a_swallowed_write_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish(
+) {
+    let fault = InjectedFault::WriteIsSwallowed;
+    let vote = reopen_vote_after_reading_back_the_version_before_the_last_publish(fault);
+    assert_no_reference_filesystem_vote_because_the_state_carries_the_fault(&vote, fault);
+}
+
+#[test]
+fn the_reopen_after_a_failing_write_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish(
+) {
+    let fault = InjectedFault::WriteFails;
+    let vote = reopen_vote_after_reading_back_the_version_before_the_last_publish(fault);
+    assert_no_reference_filesystem_vote_because_the_state_carries_the_fault(&vote, fault);
+}
+
+#[test]
+fn the_reopen_after_a_failing_read_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish(
+) {
+    let fault = InjectedFault::ReadFails;
+    let vote = reopen_vote_after_reading_back_the_version_before_the_last_publish(fault);
+    assert_no_reference_filesystem_vote_because_the_state_carries_the_fault(&vote, fault);
+}
+
+#[test]
+fn the_reopen_after_a_read_returning_a_flipped_bit_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish(
+) {
+    let fault = InjectedFault::ReadReturnsCorruptedBytes {
+        flipped_bit: FlippedBit {
+            byte_index: 0,
+            bit_index: 0,
+        },
+    };
+    let vote = reopen_vote_after_reading_back_the_version_before_the_last_publish(fault);
+    assert_no_reference_filesystem_vote_because_the_state_carries_the_fault(&vote, fault);
+}
+
+#[test]
+fn the_reopen_after_a_failing_barrier_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish(
+) {
+    let fault = InjectedFault::BarrierFails;
+    let vote = reopen_vote_after_reading_back_the_version_before_the_last_publish(fault);
+    assert_no_reference_filesystem_vote_because_the_state_carries_the_fault(&vote, fault);
+}
+
+#[test]
+fn the_reopen_after_a_swallowed_barrier_gets_no_reference_filesystem_vote_even_when_it_reads_back_the_version_before_the_last_publish(
+) {
+    let fault = InjectedFault::BarrierIsSwallowed;
+    let vote = reopen_vote_after_reading_back_the_version_before_the_last_publish(fault);
+    assert_no_reference_filesystem_vote_because_the_state_carries_the_fault(&vote, fault);
+}
````

## `crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs`

````diff
--- crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs	2026-10-02 15:34:49.517424702 +0000
+++ crates/singlefs-harness/tests/root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance.rs	2026-10-02 18:08:49.852216724 +0000
@@ -5,13 +5,16 @@
 //! - 恢复（`recovery::recover`）、只读挂载（`mounted_read::mount_read_only`）与可写挂载的读阶段（`mount::mount_writable`）
 //!   拿所选槽里那条根调重放，不是根环里别的自证过的根（同一格「胶水拿所选根调重放」）。
 //! - 择新键完全相同的两条根：择根取读根环次序里前面那条（已定项 24、核的平局取法）；算 F 生效值时判有效根用的那张实例表，
-//!   也是前面那条根指着的（同一判决 K1-d：`newest_root_among` 文档说与 `choose_root` 同一个择法）。
+//!   也是前面那条根指着的（同一判决 K1-d：`newest_root_among` 文档说与 `choose_root` 同一个择法）；池级 checker
+//!   （`singlefs_checker::walk::check_pool_image`）择最新根走读，同样取前面那条（代码轮第二轮判决辩方 D3）。
 //!
 //! 池是只做过 mkfs 的池，根照 mkfs 那条根改几项、重新编码写进 mkfs 那条根之后空着的槽（择根只读根环，不看别的结构）；
 //! 只读挂载那一格要沿根读中央映射树与树表，用建过新池新建文件的池，根照新池新建文件那条根改。
 mod common;
 
 use common::{build_pool, format_pool, parameters};
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::mount::mount_writable;
@@ -441,3 +444,46 @@
         "F 生效值判有效根读的是择根择的那条根指着的表（前面那条，读得出），不是后面那条（读不出，重读一次仍读不出就报错）"
     );
 }
+
+/// 同一份造法（择新键完全相同的两条根：前面是 mkfs 那条，后面那条只把实例表指针两条位置条目的校验和改掉），问池级 checker：
+/// 它择最新根走读（I-7.2「最新的根走得完」），键相同取读根环次序里前面那条，与 `choose_root` 同（代码轮第二轮判决辩方 D3）。
+/// 前面那条的实例表读得出，I-7.2 成立；取了后面那条，实例表读不出，I-7.2 判违例。
+#[test]
+fn of_two_roots_with_the_same_choice_key_the_pool_checker_walks_the_root_in_the_earlier_slot_as_choose_root_does(
+) {
+    let MkfsPoolWithTwoEmptySlotsAfterItsRoot {
+        mut image,
+        system_configuration,
+        mkfs_root,
+        earlier_empty_slot: _,
+        later_empty_slot,
+    } = mkfs_pool_with_two_empty_slots_after_its_root("tied-choice-key-pool-checker");
+    let mut same_key_root_with_an_unreadable_instance_table = mkfs_root;
+    for location in &mut same_key_root_with_an_unreadable_instance_table
+        .instance_table
+        .locations
+    {
+        location.unit_checksum = !location.unit_checksum;
+    }
+    write_root_into_ring_slot(
+        &mut image,
+        &system_configuration,
+        later_empty_slot,
+        &same_key_root_with_an_unreadable_instance_table,
+    );
+    assert_eq!(
+        choose_root(&image, &system_configuration),
+        Some(mkfs_root),
+        "实现择根取前面那条（mkfs 那条）"
+    );
+
+    let newest_root_walk_verdict = check_pool_image(&image)
+        .into_iter()
+        .find(|(invariant, _)| *invariant == "I-7.2")
+        .map(|(_, verdict)| verdict);
+    assert_eq!(
+        newest_root_walk_verdict,
+        Some(InvariantVerdict::Holds),
+        "checker 择最新根取读根环次序里前面那条（实例表读得出），走得完；取后面那条（实例表读不出）I-7.2 就判违例"
+    );
+}
````

## `crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs`

````diff
--- crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs	2026-10-02 14:44:04.259257937 +0000
+++ crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs	2026-10-02 18:08:49.852238634 +0000
@@ -1,12 +1,14 @@
 //! 全池没有可用的系统配置槽 0 时，池级 checker 与恢复找槽 1 的读法相同、结论相同（C651（全池没有可用的系统配置槽 0 时 checker
 //! 与实现找槽 1 的读法不同），第一轮判决 `research/prompts/m3-c651-r1-main-verification.md` 第四节三条与「用户定案」W1）：
 //! 在格式允许的每个槽距上各读一槽（不小于 4096、512 的整数倍、不越过根环起点 × 16384 − 4096），只收自证过、且自述的固定结构槽距
-//! 等于所在偏移的槽；几档都收得到时取世代号最大的，世代号相同取偏移小的；读到带读者不收之值的槽整池拒；
+//! 等于所在偏移的槽；几档都收得到时取世代号最大的，世代号相同取偏移小的；读到带读者不收之值的槽整池拒（读到可用的一档也先判
+//! I-8.1 三个字段等式、再核偏移，代码轮第二轮判决 K3-1 `research/prompts/m3-formal-code-r2-main-verification.md`）；
 //! mkfs 在写任何元数据之前把每块盘的 [0, 根环基址) 整段写 0，在用过的盘上换一个 io_min 重做 mkfs，旧池别的槽距上的槽 1 不再被择中。
 //! 盘都是内存稀疏盘（`singlefs_harness::memory_pool::SparseBlockDevice`），两边读的是同一份 `MemoryPool`。
 
 use singlefs_checker::image::{
     chosen_system_configurations, system_configuration_slot_readings, InvariantVerdict,
+    JournalRingGeometryFieldThatDisagrees, RefusedSystemConfigurationValue,
     SystemConfigurationSlotReading, SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
     SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
 };
@@ -20,7 +22,7 @@
 use singlefs_core::mount::mount_writable;
 use singlefs_core::recovery::{
     choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryFailure,
-    RecoveryOutcome,
+    RecoveryOutcome, SystemConfigurationValueOutsideWhatThisReaderAccepts,
 };
 use singlefs_core::root_ring::RootRingSlotsPerRegion;
 use singlefs_core::system_configuration::{
@@ -43,6 +45,10 @@
 const FIRST_INCOMPAT_BYTE_OFFSET: usize = 6;
 /// journal 环起点字段写成第一版常量 1024 之外的一个槽号：自证得过、读者不收（D22（单元原子性怎么合成） 已定项 16 第 1 句）。
 const JOURNAL_RING_START_SLOT_THE_READER_REFUSES: u64 = 1025;
+/// 系统配置槽里在飞上限那 4 字节的偏移（与 `journal_ring_geometry_fields_are_recomputed_when_choosing_the_system_configuration.rs` 同一处）。
+const IN_FLIGHT_RECORD_LIMIT_OFFSET_IN_THE_SLOT: usize = 345;
+/// 代码轮第二轮判决 K3-a 攻方原型放槽的偏移：io_min 8192 的池里，8192 之后下一个不是 8192 整数倍的 512 档（槽距 12288 的槽 1 也在这里）。
+const OFFSET_OF_THE_PLANTED_SLOT: u64 = 12288;
 
 fn geometry_with_minimum_input_output_bytes(
     minimum_input_output_bytes: u32,
@@ -360,6 +366,128 @@
     );
 }
 
+/// io_min 8192 的池（槽距 8192），两块盘的槽 0 整槽写 0，盘 0 在 12288 处放一份由 mkfs 写下的槽 1 改出来的槽：在飞上限 + 1、
+/// `edit_the_slot` 再改几格、整槽校验和重封（代码轮第二轮判决 K3-a 攻方原型 `k3slot` 那两种槽的造法）。交回这份镜像。
+fn io_min_8192_pool_with_an_in_flight_limit_plus_one_slot_planted_at_12288_on_device_zero(
+    edit_the_slot: impl Fn(&mut SystemConfiguration),
+) -> MemoryPool {
+    let mut image = formatted_image_with_minimum_input_output_bytes(
+        MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING,
+    );
+    let mut planted = slot_at(
+        &image,
+        DeviceIdentity(0),
+        u64::from(MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING),
+    );
+    edit_the_slot(&mut planted);
+    let mut planted_bytes = planted.to_slot();
+    let in_flight_record_limit_field = IN_FLIGHT_RECORD_LIMIT_OFFSET_IN_THE_SLOT
+        ..IN_FLIGHT_RECORD_LIMIT_OFFSET_IN_THE_SLOT + std::mem::size_of::<u32>();
+    let recorded_in_flight_record_limit = u32::from_le_bytes(
+        planted_bytes[in_flight_record_limit_field.clone()]
+            .try_into()
+            .expect("在飞上限是 4 字节"),
+    );
+    planted_bytes[in_flight_record_limit_field]
+        .copy_from_slice(&(recorded_in_flight_record_limit + 1).to_le_bytes());
+    reseal_system_configuration_slot(&mut planted_bytes);
+    image
+        .devices
+        .get_mut(&DeviceIdentity(0))
+        .expect("池里有这块盘")
+        .write(
+            DeviceOffsetInBytes(OFFSET_OF_THE_PLANTED_SLOT),
+            &planted_bytes,
+        );
+    wipe_the_slot_at_on_every_device(&mut image, 0);
+    image
+}
+
+/// 同一份镜像上恢复与 checker 的结局：恢复整池拒、点名盘 0、拒的是在飞上限那一格；checker 盘 0 的槽 1 读成 12288 处那份带不收之值的槽、
+/// 报 I-7.13（系统配置池级字段在读者收的范围里） 违例，盘 1 照旧择到 8192 处 mkfs 写下的世代号 1。
+fn assert_recovery_refuses_the_pool_and_the_checker_reads_the_planted_slot_as_carrying_a_refused_value(
+    image: &MemoryPool,
+) {
+    let recovery_choice = choose_system_configuration(image);
+    assert!(
+        matches!(
+            recovery_choice,
+            Err(RecoveryFailure::SystemConfigurationValueRefused {
+                device: DeviceIdentity(0),
+                value: SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalInFlightRecordLimitNotRecomputedFromTheRingLength { .. },
+            })
+        ),
+        "恢复逐档读到 12288 处那一份，先判值、整池拒、点名盘 0 的在飞上限：{recovery_choice:?}"
+    );
+    let readings = system_configuration_slot_readings(image);
+    assert_eq!(
+        readings
+            .iter()
+            .find(|(device, _)| *device == 0)
+            .map(|(_, slots)| slots[1].clone()),
+        Some(SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses {
+            offset_in_bytes: OFFSET_OF_THE_PLANTED_SLOT,
+            refused: RefusedSystemConfigurationValue::JournalRingGeometryFieldDisagrees(
+                JournalRingGeometryFieldThatDisagrees::InFlightRecordLimitNotRecomputedFromTheRingLength,
+            ),
+        }),
+        "checker 盘 0 逐档读到 12288 处那一份，先判 I-8.1 三格、当场交回带不收之值，不核偏移、不比世代号：{readings:?}"
+    );
+    assert_eq!(
+        chosen_by_the_checker(image),
+        vec![
+            (0, None),
+            (
+                1,
+                Some((u64::from(MINIMUM_INPUT_OUTPUT_BYTES_OF_AN_8192_SPACING), 1))
+            )
+        ],
+        "checker 盘 0 择不到；盘 1 没有那一份，照旧择到 8192 处世代号 1"
+    );
+    assert!(
+        matches!(
+            verdict_of(
+                image,
+                SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
+            ),
+            InvariantVerdict::Violated(_)
+        ),
+        "checker 报 I-7.13 违例、整池不作保，与恢复整池拒同一个结局"
+    );
+}
+
+/// 攻方原型 `inflight_plus1_spacing_8192_at_12288`：12288 处那一份自述槽距 8192（不在自己记的槽距上）、世代号 1、在飞上限 + 1。
+/// 实现先判值、整池拒；checker 改前先核偏移，把它当「不在自己槽距上」跳过，择 8192 处世代号 1、全绿。
+#[test]
+fn a_slot_whose_in_flight_limit_is_off_by_one_and_whose_spacing_is_not_its_offset_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery(
+) {
+    let image =
+        io_min_8192_pool_with_an_in_flight_limit_plus_one_slot_planted_at_12288_on_device_zero(
+            |_unchanged| {},
+        );
+    assert_recovery_refuses_the_pool_and_the_checker_reads_the_planted_slot_as_carrying_a_refused_value(
+        &image,
+    );
+}
+
+/// 攻方原型 `inflight_plus1_spacing_equals_offset_gen0`：12288 处那一份自述槽距 12288（等于所在偏移）、世代号 0、在飞上限 + 1。
+/// 实现先判值、整池拒；checker 改前核过偏移之后比世代号，0 比 8192 处的 1 小，择 8192 处世代号 1、全绿。
+#[test]
+fn a_slot_whose_in_flight_limit_is_off_by_one_and_whose_generation_is_lower_found_while_reading_every_spacing_refuses_the_pool_in_both_the_checker_and_recovery(
+) {
+    let image =
+        io_min_8192_pool_with_an_in_flight_limit_plus_one_slot_planted_at_12288_on_device_zero(
+            |planted| {
+                planted.immutable.sizes.fixed_structure_slot_spacing =
+                    u32::try_from(OFFSET_OF_THE_PLANTED_SLOT).expect("12288 装得进 4 字节");
+                planted.quantities.slot_generation = 0;
+            },
+        );
+    assert_recovery_refuses_the_pool_and_the_checker_reads_the_planted_slot_as_carrying_a_refused_value(
+        &image,
+    );
+}
+
 /// 逐档读到的自证过的槽都不可择（4096 处本池的槽 1 也抹掉，8192 处放一份 incompat 位图不认识的槽）：恢复报 incompat 位图不认识、
 /// 点名盘 0；checker 两块盘都择不到，而那一槽自证过，I-8.5（tail 槽有效集非空） 照样成立。
 #[test]
````

## `crates/singlefs-reference-filesystem/src/lib.rs`

````diff
--- crates/singlefs-reference-filesystem/src/lib.rs	2026-10-02 14:44:08.496522660 +0000
+++ crates/singlefs-reference-filesystem/src/lib.rs	2026-10-02 16:17:24.607390473 +0000
@@ -5,10 +5,14 @@
 //! 形态是用户 2026-10-02 的定案：语义级（`research/prompts/m3-formal-7-forks.md` Y2）；按发布边界的前缀 Q、另起、单独一个包
 //! （`research/prompts/m3-formal-r4-main-verification.md` 用户定案 F1、F3、F4）；回退目标作输入
 //! （`research/prompts/m3-formal-r5-main-verification.md` 用户定案 G2）。条款候选取那份判决第二节 U1、U2、U4 与第四节第 1、2、4、8 条（零轮）。
+//! 回退怎么算是 r6 的用户定案 H1-收严（`research/prompts/m3-formal-r6-main-verification.md` 用户定案一节）：返回成功的发布调用、fsync、
+//! 回退都抬 lo；回退算一次已确认发布、它的下标算发布末尾；回退目标 m 是版本号，版本含每次恢复读出、重新起头的那一份；
+//! 操作结局多「原样重发」一种。条款句取那份判决第二节 U2、X2 两行与第四节（零轮）。
 //!
-//! 输入只有五样，都由调用方交入：每个操作的结局、崩溃落在第几个操作里（hi）、回退目标 m、每次恢复读出的内容、
+//! 输入只有五样（r6 判决第二节 X2 一行的白名单），都由调用方交入：每个操作的结局（成、在任何写之前拒、写了之后报错、原样重发）、
+//! 崩溃落在第几个操作里（hi；按切开当前段的那道屏障由哪一步发出归属，归属由调用方做）、回退目标 m（版本号）、每次恢复读出的内容、
 //! 显式发布点（发布调用与 fsync 这两种操作本身）。这个包不读盘、不读录制流、不枚举崩溃点，依赖只有 std。
-//! 只管串行提交；任意子集的丢法不做（r5 判决第四节第 8 条：串行提交下出局）。
+//! 只管串行提交；任意子集的丢法不做（r5 判决第四节第 8 条：串行提交下出局）。自动触发的攒批发布落地之前只接显式发布点（r6 判决第二节 U2 一行）。
 #![forbid(unsafe_code)]
 
 /// 一个文件的全部字节。
@@ -27,12 +31,13 @@
 pub enum Operation {
     CreateTheFirstFile(FileContent),
     OverwriteTheFile(FileContent),
-    /// 发布调用：一个显式发布点。
+    /// 发布调用：一个显式发布点；返回成功时抬起下界 lo。
     ExplicitPublish,
     /// 一个显式发布点；返回成功时抬起下界 lo。
     Fsync,
-    /// 回退到整段历史里第 m 次已确认发布之后的内容；m 由调用方从实现报的回退根翻出来，选得对不对这里不判。
-    RollBackToConfirmedPublish(ConfirmedPublishNumber),
+    /// 回退到第 m 版的内容；m 由调用方按实现报的回退根在「根 → 版本」表里查出来，选得对不对这里不判。
+    /// 回退是挂着时的一次向前发布：返回成功时抬起下界 lo，它的下标是发布末尾，它之后的内容是一个新版本。
+    RollBackToVersion(VersionNumber),
 }
 
 /// 调用方交入的结局。
@@ -43,6 +48,8 @@
     RefusedBeforeAnyWrite,
     /// 写了之后报错：这一步做没做成两可。
     FailedAfterWriting,
+    /// 落盘途中失败、冻结之后原样重发成功：这一步做成了，与「成」一样抬 lo、算已确认。
+    SucceededOnAVerbatimResendAfterFailingWhileWriting,
 }
 
 /// 上一次恢复（还没崩溃过时是起点）之后进了操作序列的第几个操作，从 1 起；在任何写之前被拒的不占号。
@@ -61,9 +68,11 @@
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
 pub struct StateIndex(pub usize);
 
-/// 整段历史里第几次已确认的发布，从 1 起。
+/// RefFS 历史里第几版，从 1 起，按历史次序编、跨崩溃累计：第 1 版是起点（新格式化的空池）；之后每一次已确认的发布
+/// （返回成功的发布调用、fsync、回退）之后的内容，与每一次恢复读出、以它重新起头的那一份，各占一个号。
+/// 调用方把实现写下的每一条根（发布调用、回退、可写挂载的写行与暖机、抬 F、卸载）映到写它时的现行版本（r6 用户定案 H1-收严）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
-pub struct ConfirmedPublishNumber(pub usize);
+pub struct VersionNumber(pub usize);
 
 /// 交进来的操作进没进操作序列。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -79,10 +88,10 @@
     CreatingTheFirstFileWhileAFileExistsIsNotInTheFirstVersion,
     /// 还没有文件时覆盖写：条款没写它在 POSIX 面上是什么，第一版不支持。
     OverwritingWithoutAFileIsNotInTheFirstVersion,
-    /// 回退目标 m 不是 1 到「到这一步为止已确认的发布数」之间的数。
-    RollBackTargetIsNotAConfirmedPublish {
-        requested: ConfirmedPublishNumber,
-        confirmed_publishes_so_far: usize,
+    /// 回退目标 m 不是 1 到「到这一步为止的版本数」之间的数。
+    RollBackTargetIsNotAVersion {
+        requested: VersionNumber,
+        versions_so_far: usize,
     },
 }
 
@@ -93,7 +102,7 @@
     pub state: VisibleState,
 }
 
-/// 一次崩溃之后恢复允许读出的状态：{S_j : j = lo 或 j 恰是某次显式发布的末尾，lo ≤ j ≤ hi}，按下标升序。
+/// 一次崩溃之后恢复允许读出的状态：{S_j : j = lo 或 j 恰是某次显式发布或回退的末尾，lo ≤ j ≤ hi}，按下标升序。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct AllowedSet {
     pub lower_bound: StateIndex,
@@ -165,7 +174,7 @@
     state_after: VisibleState,
 }
 
-/// 序列里已确认的发布（返回成功的发布调用、fsync 与回退）各自发布之后的内容，按历史次序。
+/// 序列里已确认的发布（返回成功的发布调用、fsync 与回退，原样重发成功的也算）各自发布之后的内容，按历史次序。
 fn confirmed_publishes_among(
     operations: &[EnteredOperation],
 ) -> impl Iterator<Item = &VisibleState> {
@@ -194,21 +203,22 @@
                 .clone(),
         }
     };
-    // 崩溃落在第 hi 个操作里：前 hi − 1 个返回过；第 hi 个没返回，它是 fsync 也不抬下界
-    // （r5 判决第二节 U1：返回成功的 fsync 才抬）。
+    // 崩溃落在第 hi 个操作里：前 hi − 1 个返回过；第 hi 个没返回，它是发布调用、fsync 或回退也不抬下界
+    // （r5 判决第二节 U1：返回成功的才抬）。
     let returned_before_the_crash = &operations[..crash_falls_in.0 - 1];
-    // lo = 最后一次返回成功的 fsync 等到的那次发布的末尾；串行下就是 fsync 调用之前已取号成功的操作数：位置 p（从 0 起）上的
-    // fsync 是第 p + 1 个操作，它之前取号成功的操作有 p 个。没有返回成功的 fsync 时 lo = 0，起点是上一次恢复读出的内容。
-    // .claude/kb/decisions/16-发布语义.md@已定项 7：发布的持久顺序（fsync 等系统配置轮换持久之后才返回）；
-    // fsync 报错时下界停在上一次返回成功的 fsync：.claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
+    // lo = 崩溃之前最后一次返回成功的发布调用、fsync 或回退的末尾（r6 用户定案 H1-收严：三者同一个返回条件——
+    // .claude/kb/decisions/16-发布语义.md@已定项 7：发布的持久顺序 里「fsync 等系统配置轮换持久之后才返回」与回退
+    // 「它的根持久之后才向管理员确认，与 fsync 同一个返回条件」）。位置 p（从 0 起）上的是第 p + 1 个操作，施加之后是 S_{p+1}；
+    // 发布调用与 fsync 不改内容，S_{p+1} 与调用之前那一刻相同。没有时 lo = 0，起点是上一次恢复读出的内容。
+    // 报错时下界停在上一次返回成功的那一个：.claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
     let lower_bound = returned_before_the_crash
         .iter()
         .enumerate()
         .rev()
         .find(|(_, operation)| {
-            operation.kind == EnteredKind::Fsync && operation.outcome == EnteredOutcome::Succeeded
+            operation.outcome == EnteredOutcome::Succeeded && operation.kind.ends_a_publish()
         })
-        .map_or(StateIndex(0), |(position, _)| StateIndex(position));
+        .map_or(StateIndex(0), |(position, _)| StateIndex(position + 1));
     let upper_bound = crash_falls_in.state_after();
     // Q：j 只取 lo 与恰是某次显式发布末尾的下标，lo ≤ j ≤ hi（用户定案 F1）。一次发布整体施加或整体不施加，
     // 切开一次发布的 S_j 不在集合里：.claude/kb/decisions/16-发布语义.md@已定项 4：一次发布整体施加或整体不施加，第二份 replay 停在句法层；
@@ -249,8 +259,8 @@
     state_at_the_last_recovery: VisibleState,
     /// 上一次恢复之后进了序列的操作；第 k 个在位置 k − 1。
     operations_since_the_last_recovery: Vec<EnteredOperation>,
-    /// 上一次恢复之前各轮里已确认的发布，各自发布之后的内容，按历史次序。
-    confirmed_publishes_before_the_last_recovery: Vec<VisibleState>,
+    /// 上一次恢复为止的版本，按历史次序：起点、之前各轮里已确认的发布之后的内容、各次恢复读出的那一份；第 m 版在位置 m − 1。
+    versions_up_to_the_last_recovery: Vec<VisibleState>,
 }
 
 impl ReferenceFileSystem {
@@ -258,10 +268,31 @@
         Self {
             state_at_the_last_recovery: VisibleState::NoFile,
             operations_since_the_last_recovery: Vec::new(),
-            confirmed_publishes_before_the_last_recovery: Vec::new(),
+            // 起点是第 1 版：mkfs 写的根与 mkfs 之后可写挂载写的根映到它（r6 用户定案 H1-收严「m 的编号覆盖每一个写根的入口」）
+            versions_up_to_the_last_recovery: vec![VisibleState::NoFile],
         }
     }
 
+    /// 到这一刻为止的版本，按历史次序（第 m 版在位置 m − 1）。
+    fn versions_so_far(&self) -> Vec<&VisibleState> {
+        self.versions_up_to_the_last_recovery
+            .iter()
+            .chain(confirmed_publishes_among(
+                &self.operations_since_the_last_recovery,
+            ))
+            .collect()
+    }
+
+    /// 到这一刻为止最新的版本号：调用方把这一刻写下的根映到它。
+    pub fn newest_version(&self) -> VersionNumber {
+        VersionNumber(self.versions_so_far().len())
+    }
+
+    /// 崩溃落在崩溃本身里（序列里的操作都返回过之后才崩）时 hi 取的号：序列长度 + 1。
+    pub fn crash_after_every_operation_returned(&self) -> OperationNumber {
+        OperationNumber(self.operations_since_the_last_recovery.len() + 1)
+    }
+
     /// 序列里的操作全都做成时的状态。
     fn state_after_every_operation_so_far(&self) -> &VisibleState {
         self.operations_since_the_last_recovery
@@ -271,30 +302,24 @@
             })
     }
 
-    /// 回退的目标内容 = 整段历史里第 m 次已确认发布之后的内容（用户定案 G2）。
-    fn content_after_confirmed_publish(
+    /// 回退的目标内容 = 第 m 版的内容（用户定案 G2，m 的编号照 r6 用户定案 H1-收严）。
+    fn content_of_version(
         &self,
-        number: ConfirmedPublishNumber,
+        number: VersionNumber,
     ) -> Result<VisibleState, OperationNotAccepted> {
-        let confirmed: Vec<&VisibleState> = self
-            .confirmed_publishes_before_the_last_recovery
-            .iter()
-            .chain(confirmed_publishes_among(
-                &self.operations_since_the_last_recovery,
-            ))
-            .collect();
-        let not_confirmed = OperationNotAccepted::RollBackTargetIsNotAConfirmedPublish {
+        let versions = self.versions_so_far();
+        let not_a_version = OperationNotAccepted::RollBackTargetIsNotAVersion {
             requested: number,
-            confirmed_publishes_so_far: confirmed.len(),
+            versions_so_far: versions.len(),
         };
-        // 第 m 次从 1 起数，在第 m − 1 位：.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
+        // 第 m 版从 1 起数，在第 m − 1 位：.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
         // （回退的新根取 R_old 那一版的用户可见树）
         let Some(position) = number.0.checked_sub(1) else {
-            return Err(not_confirmed);
+            return Err(not_a_version);
         };
-        match confirmed.get(position) {
+        match versions.get(position) {
             Some(state) => Ok((*state).clone()),
-            None => Err(not_confirmed),
+            None => Err(not_a_version),
         }
     }
 
@@ -312,6 +337,11 @@
             // .claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换（「在任何写之前拒」）
             OperationOutcome::RefusedBeforeAnyWrite => return Ok(SubmittedOperation::DidNotHappen),
             OperationOutcome::Succeeded => EnteredOutcome::Succeeded,
+            // 原样重发成功＝做成：.claude/kb/decisions/23-journal的角色与格式.md@已定项 14：重放的下界由所选根给出；管理员回退、失败处置与实例切换
+            // （「这一版的失败处置」：重发成功才建下一次发布）；r6 判决第二节「G2 回退被拒」一行补的结局
+            OperationOutcome::SucceededOnAVerbatimResendAfterFailingWhileWriting => {
+                EnteredOutcome::Succeeded
+            }
             // 写了之后报错＝两可：.claude/kb/decisions/16-发布语义.md@已定项 17：fsync 报错之后，调用方能假设什么
             OperationOutcome::FailedAfterWriting => EnteredOutcome::FailedAfterWriting,
         };
@@ -327,10 +357,9 @@
             },
             Operation::ExplicitPublish => (EnteredKind::ExplicitPublish, state_so_far.clone()),
             Operation::Fsync => (EnteredKind::Fsync, state_so_far.clone()),
-            Operation::RollBackToConfirmedPublish(number) => (
-                EnteredKind::RollBack,
-                self.content_after_confirmed_publish(number)?,
-            ),
+            Operation::RollBackToVersion(number) => {
+                (EnteredKind::RollBack, self.content_of_version(number)?)
+            }
         };
         self.operations_since_the_last_recovery
             .push(EnteredOperation {
@@ -353,7 +382,7 @@
     ) -> Result<AllowedSet, CrashNotAccepted> {
         let operations = &self.operations_since_the_last_recovery;
         // hi 由调用方交入（r5 判决用户定案 G2 那一句与第四节第 2 条的输入白名单）；落在崩溃本身里 = 序列里的操作都返回过之后才崩。
-        let crash_operation_number = OperationNumber(operations.len() + 1);
+        let crash_operation_number = self.crash_after_every_operation_returned();
         if crash_falls_in.0 == 0 || crash_falls_in > crash_operation_number {
             return Err(
                 CrashNotAccepted::CrashOperationIsOutsideTheOperationsSinceTheLastRecovery {
@@ -403,6 +432,22 @@
         Ok(allowed)
     }
 
+    /// 崩溃恢复抛弃最新根的那一次（恢复时有读故障）：不判读出的内容，以它重新起头（r6 判决第二节 U2 一行：
+    /// 「崩溃恢复抛弃最新根的那一次，RefFS 以读出内容重新起头、不判那一步」）。
+    ///
+    /// # Errors
+    /// hi 越界、或写了之后报错的操作后面到崩溃为止又起了操作（同 [`Self::allowed_set_for_a_crash_in`]），返回时模型不动。
+    pub fn restart_after_a_recovery_that_abandoned_the_newest_root(
+        &mut self,
+        crash_falls_in: OperationNumber,
+        recovered: VisibleState,
+    ) -> Result<(), CrashNotAccepted> {
+        // 只核 hi 判不判得了；读出的内容在不在允许集合里不判
+        self.allowed_set_for_a_crash_in(crash_falls_in)?;
+        self.restart_from_the_recovered_state(crash_falls_in, recovered);
+        Ok(())
+    }
+
     fn restart_from_the_recovered_state(
         &mut self,
         crash_falls_in: OperationNumber,
@@ -414,8 +459,10 @@
         let confirmed: Vec<VisibleState> = confirmed_publishes_among(returned_before_the_crash)
             .cloned()
             .collect();
-        self.confirmed_publishes_before_the_last_recovery
-            .extend(confirmed);
+        self.versions_up_to_the_last_recovery.extend(confirmed);
+        // 恢复读出、以它重新起头的那一份也是一个版本：之后可写挂载写的根映到它（r6 用户定案 H1-收严）
+        self.versions_up_to_the_last_recovery
+            .push(recovered.clone());
         // 每次恢复之后以读出的内容为起点接着走（r5 判决第二节 U2 ②）
         self.state_at_the_last_recovery = recovered;
         self.operations_since_the_last_recovery.clear();
@@ -487,7 +534,7 @@
             .expect("崩溃落在崩溃本身里，号 4 合法");
         assert!(
             allowed.contains(&file_with("A")),
-            "fsync 之后的 A 必须允许：{allowed:?}"
+            "发布调用之后的 A 必须允许：{allowed:?}"
         );
         assert!(
             !allowed.contains(&file_with("B")),
@@ -496,20 +543,20 @@
     }
 
     #[test]
-    fn a_crash_after_two_overwrites_and_one_publish_does_not_allow_the_state_that_cuts_the_publish()
-    {
+    fn a_crash_inside_a_publish_that_carries_two_overwrites_does_not_allow_the_state_that_cuts_the_publish(
+    ) {
         let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
         succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
         succeed(&mut model, Operation::Fsync);
         succeed(&mut model, Operation::OverwriteTheFile(content("B")));
         succeed(&mut model, Operation::OverwriteTheFile(content("C")));
-        succeed(&mut model, Operation::ExplicitPublish);
+        let publish = succeed(&mut model, Operation::ExplicitPublish);
         let allowed = model
-            .allowed_set_for_a_crash_in(OperationNumber(6))
-            .expect("号 6 是崩溃本身");
+            .allowed_set_for_a_crash_in(publish)
+            .expect("崩溃落在第 5 个操作发布调用里");
         assert_eq!(
             allowed_indices(&allowed),
-            vec![1, 2, 5],
+            vec![2, 5],
             "只取 lo 与发布末尾：{allowed:?}"
         );
         assert!(
@@ -518,12 +565,13 @@
         );
         assert!(
             allowed.contains(&file_with("C")),
-            "发布末尾的 C 允许：{allowed:?}"
+            "在途的发布可能已发布，末尾的 C 允许：{allowed:?}"
         );
     }
 
     #[test]
-    fn after_a_successful_fsync_the_state_just_before_the_fsync_call_is_the_lower_bound() {
+    fn after_a_successful_fsync_its_own_index_is_the_lower_bound_so_the_content_before_it_is_not_allowed(
+    ) {
         let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
         succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
         succeed(&mut model, Operation::Fsync);
@@ -534,8 +582,8 @@
             .expect("号 5 是崩溃本身");
         assert_eq!(
             allowed.lower_bound,
-            StateIndex(3),
-            "第 4 个操作是 fsync，它之前取号成功的有 3 个"
+            StateIndex(4),
+            "第 4 个操作是返回成功的 fsync，下界是它的末尾"
         );
         assert!(
             !allowed.contains(&file_with("A")),
@@ -558,7 +606,7 @@
         let allowed = model
             .allowed_set_for_a_crash_in(OperationNumber(5))
             .expect("报错的 fsync 是崩溃前最后一个操作，判得了");
-        assert_eq!(allowed.lower_bound, StateIndex(1), "下界停在第一次 fsync");
+        assert_eq!(allowed.lower_bound, StateIndex(2), "下界停在第一次 fsync");
         assert!(
             allowed.contains(&file_with("A")),
             "报错那次发布两可，A 允许：{allowed:?}"
@@ -585,7 +633,7 @@
             .expect("报错的发布是崩溃前最后一个操作，判得了");
         assert_eq!(
             allowed_indices(&allowed),
-            vec![1, 2, 4],
+            vec![2, 4],
             "报错的发布末尾也在集合里：{allowed:?}"
         );
         assert!(
@@ -607,7 +655,7 @@
         assert_eq!(allowed.upper_bound, StateIndex(3), "hi 是覆盖写那一步");
         assert_eq!(
             allowed_indices(&allowed),
-            vec![1, 2],
+            vec![2],
             "崩溃之后的发布没发生：{allowed:?}"
         );
         assert!(
@@ -626,7 +674,11 @@
         let allowed = model
             .allowed_set_for_a_crash_in(fsync)
             .expect("崩溃落在第 4 个操作 fsync 里");
-        assert_eq!(allowed.lower_bound, StateIndex(0), "fsync 没返回，下界不抬");
+        assert_eq!(
+            allowed.lower_bound,
+            StateIndex(2),
+            "下界在返回过的发布调用上；fsync 没返回，不抬"
+        );
         assert!(
             allowed.contains(&file_with("A")),
             "发布调用之后的 A 允许：{allowed:?}"
@@ -691,10 +743,7 @@
         succeed(&mut model, Operation::OverwriteTheFile(content("B")));
         succeed(&mut model, Operation::ExplicitPublish);
         succeed(&mut model, Operation::OverwriteTheFile(content("C")));
-        let rollback = succeed(
-            &mut model,
-            Operation::RollBackToConfirmedPublish(ConfirmedPublishNumber(1)),
-        );
+        let rollback = succeed(&mut model, Operation::RollBackToVersion(VersionNumber(2)));
         let allowed = model
             .allowed_set_for_a_crash_in(OperationNumber(rollback.0 + 1))
             .expect("号 7 是崩溃本身");
@@ -706,29 +755,29 @@
         assert_eq!(
             after_the_rollback.state,
             file_with("A"),
-            "第 1 次已确认发布之后是 A"
+            "第 1 版是起点，第 2 版是第一次已确认发布之后的 A"
         );
     }
 
     #[test]
-    fn a_crash_right_after_a_rollback_allows_the_rolled_back_content_even_below_the_fsynced_state()
-    {
+    fn a_crash_inside_a_rollback_allows_the_rolled_back_content_even_below_the_fsynced_state() {
         let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
         succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
         succeed(&mut model, Operation::ExplicitPublish);
         succeed(&mut model, Operation::OverwriteTheFile(content("B")));
         succeed(&mut model, Operation::Fsync);
-        succeed(
-            &mut model,
-            Operation::RollBackToConfirmedPublish(ConfirmedPublishNumber(1)),
-        );
+        let rollback = succeed(&mut model, Operation::RollBackToVersion(VersionNumber(2)));
         let allowed = model
-            .allowed_set_for_a_crash_in(OperationNumber(6))
-            .expect("号 6 是崩溃本身");
-        assert_eq!(allowed.lower_bound, StateIndex(3), "fsync 把下界抬到 B");
+            .allowed_set_for_a_crash_in(rollback)
+            .expect("崩溃落在第 5 个操作回退里");
+        assert_eq!(allowed.lower_bound, StateIndex(4), "fsync 把下界抬到 B");
         assert!(
             allowed.contains(&file_with("A")),
-            "回退那次发布的末尾是 A，允许：{allowed:?}"
+            "在途的回退可能已发布，它的末尾 A 允许：{allowed:?}"
+        );
+        assert!(
+            allowed.contains(&file_with("B")),
+            "回退没返回，fsync 之后的 B 允许：{allowed:?}"
         );
     }
 
@@ -743,20 +792,17 @@
             Operation::ExplicitPublish,
             OperationOutcome::FailedAfterWriting,
         );
-        model
-            .crash_and_recover(OperationNumber(5), file_with("B"))
-            .expect("报错那次发布可在，读出 B 合法");
         let refused = model.submit(
-            Operation::RollBackToConfirmedPublish(ConfirmedPublishNumber(2)),
+            Operation::RollBackToVersion(VersionNumber(3)),
             OperationOutcome::Succeeded,
         );
         assert_eq!(
             refused,
-            Err(OperationNotAccepted::RollBackTargetIsNotAConfirmedPublish {
-                requested: ConfirmedPublishNumber(2),
-                confirmed_publishes_so_far: 1,
+            Err(OperationNotAccepted::RollBackTargetIsNotAVersion {
+                requested: VersionNumber(3),
+                versions_so_far: 2,
             }),
-            "报错的发布不算已确认，只有 1 次"
+            "报错的发布不算已确认：只有起点与 A 两版"
         );
     }
 
@@ -768,19 +814,19 @@
         succeed(&mut model, Operation::OverwriteTheFile(content("B")));
         let publish = succeed(&mut model, Operation::ExplicitPublish);
         model
-            .crash_and_recover(publish, file_with("B"))
-            .expect("崩在发布调用里，读出 B 合法");
+            .crash_and_recover(publish, file_with("A"))
+            .expect("崩在发布调用里，读出 A 合法");
         let refused = model.submit(
-            Operation::RollBackToConfirmedPublish(ConfirmedPublishNumber(2)),
+            Operation::RollBackToVersion(VersionNumber(4)),
             OperationOutcome::Succeeded,
         );
         assert_eq!(
             refused,
-            Err(OperationNotAccepted::RollBackTargetIsNotAConfirmedPublish {
-                requested: ConfirmedPublishNumber(2),
-                confirmed_publishes_so_far: 1,
+            Err(OperationNotAccepted::RollBackTargetIsNotAVersion {
+                requested: VersionNumber(4),
+                versions_so_far: 3,
             }),
-            "崩溃落在里面的发布没返回，不算已确认"
+            "崩溃落在里面的发布没返回，不算已确认：起点、A、恢复读出的 A 三版"
         );
     }
 
@@ -863,23 +909,171 @@
     }
 
     #[test]
-    fn rolling_back_to_confirmed_publish_zero_is_refused_and_the_model_does_not_move() {
+    fn rolling_back_to_version_zero_is_refused_and_the_model_does_not_move() {
         let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
         succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
         succeed(&mut model, Operation::ExplicitPublish);
         let before = model.clone();
         let refused = model.submit(
-            Operation::RollBackToConfirmedPublish(ConfirmedPublishNumber(0)),
+            Operation::RollBackToVersion(VersionNumber(0)),
             OperationOutcome::Succeeded,
         );
         assert_eq!(
             refused,
-            Err(OperationNotAccepted::RollBackTargetIsNotAConfirmedPublish {
-                requested: ConfirmedPublishNumber(0),
-                confirmed_publishes_so_far: 1,
+            Err(OperationNotAccepted::RollBackTargetIsNotAVersion {
+                requested: VersionNumber(0),
+                versions_so_far: 2,
             }),
-            "第几次从 1 起数"
+            "第几版从 1 起数"
         );
         assert_eq!(model, before, "拒时模型不动");
     }
+
+    // ─── r6 用户定案 H1-收严 ───
+
+    #[test]
+    fn after_a_publish_call_returns_the_content_before_it_is_no_longer_allowed() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        let publish = succeed(&mut model, Operation::ExplicitPublish);
+        let allowed = model
+            .allowed_set_for_a_crash_in(OperationNumber(publish.0 + 1))
+            .expect("崩溃落在崩溃本身里");
+        assert_eq!(
+            allowed.lower_bound,
+            publish.state_after(),
+            "返回成功的发布调用抬 lo 到它的末尾"
+        );
+        assert!(
+            !allowed.contains(&file_with("A")),
+            "第二次发布调用返回之后 A 不许再出现：{allowed:?}"
+        );
+    }
+
+    #[test]
+    fn after_a_rollback_returns_reading_the_content_before_the_rollback_is_red() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        let rollback = succeed(&mut model, Operation::RollBackToVersion(VersionNumber(2)));
+        let crash = model.crash_after_every_operation_returned();
+        let refused = model.crash_and_recover(crash, file_with("B"));
+        assert!(
+            matches!(
+                refused,
+                Err(CrashNotAccepted::RecoveredStateOutsideTheAllowedSet { .. })
+            ),
+            "回退返回之后读回回退之前的 B：回退在持久之前就返回的那一类坏实现：{refused:?}"
+        );
+        let allowed = model
+            .crash_and_recover(crash, file_with("A"))
+            .expect("回退之后读出回退目标 A 合法");
+        assert_eq!(
+            allowed.lower_bound,
+            rollback.state_after(),
+            "返回成功的回退抬 lo 到它的末尾"
+        );
+    }
+
+    #[test]
+    fn a_publish_resent_verbatim_after_failing_while_writing_raises_the_lower_bound_and_is_a_version(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        enter(
+            &mut model,
+            Operation::ExplicitPublish,
+            OperationOutcome::SucceededOnAVerbatimResendAfterFailingWhileWriting,
+        );
+        assert_eq!(
+            model.newest_version(),
+            VersionNumber(3),
+            "原样重发成功的发布是第 3 版"
+        );
+        let allowed = model
+            .allowed_set_for_a_crash_in(model.crash_after_every_operation_returned())
+            .expect("崩溃落在崩溃本身里");
+        assert!(
+            !allowed.contains(&file_with("A")),
+            "原样重发成功＝做成，A 不许再出现：{allowed:?}"
+        );
+        assert!(allowed.contains(&file_with("B")), "B 必须允许：{allowed:?}");
+    }
+
+    #[test]
+    fn a_freshly_formatted_pool_is_version_one_and_the_first_confirmed_publish_is_version_two() {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        assert_eq!(model.newest_version(), VersionNumber(1), "起点是第 1 版");
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        assert_eq!(
+            model.newest_version(),
+            VersionNumber(1),
+            "还没发布的覆盖不是新版本"
+        );
+        succeed(&mut model, Operation::ExplicitPublish);
+        assert_eq!(
+            model.newest_version(),
+            VersionNumber(2),
+            "第一次已确认的发布是第 2 版"
+        );
+    }
+
+    #[test]
+    fn after_a_recovery_adopts_a_publish_that_never_returned_rolling_back_to_the_version_that_recovery_restarted_from_gives_the_recovered_content(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        let publish = succeed(&mut model, Operation::ExplicitPublish);
+        model
+            .crash_and_recover(publish, file_with("A"))
+            .expect("崩在第一个文件那次发布里，恢复采纳在飞的 A 合法");
+        assert_eq!(
+            model.newest_version(),
+            VersionNumber(2),
+            "在飞的发布不算已确认；恢复读出、重新起头的 A 是第 2 版"
+        );
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        let rollback = succeed(&mut model, Operation::RollBackToVersion(VersionNumber(2)));
+        let allowed = model
+            .allowed_set_for_a_crash_in(model.crash_after_every_operation_returned())
+            .expect("崩溃落在崩溃本身里");
+        assert_eq!(
+            allowed.allowed,
+            vec![AllowedState {
+                index: rollback.state_after(),
+                state: file_with("A"),
+            }],
+            "回退到恢复重新起头的那一版，之后只允许它的内容"
+        );
+    }
+
+    #[test]
+    fn a_recovery_that_abandoned_the_newest_root_is_not_judged_and_the_next_crash_is_judged_from_what_it_read(
+    ) {
+        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
+        succeed(&mut model, Operation::CreateTheFirstFile(content("A")));
+        succeed(&mut model, Operation::ExplicitPublish);
+        succeed(&mut model, Operation::OverwriteTheFile(content("B")));
+        succeed(&mut model, Operation::Fsync);
+        let crash = model.crash_after_every_operation_returned();
+        model
+            .restart_after_a_recovery_that_abandoned_the_newest_root(crash, file_with("A"))
+            .expect("抛弃最新根的那一次恢复不判：fsync 之后读出 A 也照收");
+        let next_crash = model.crash_after_every_operation_returned();
+        let refused = model.crash_and_recover(next_crash, file_with("B"));
+        assert!(
+            matches!(
+                refused,
+                Err(CrashNotAccepted::RecoveredStateOutsideTheAllowedSet { .. })
+            ),
+            "以读出的 A 重新起头，之后读回 B 判红：{refused:?}"
+        );
+    }
 }
````

## `crates/singlefs-replay-prefix/src/lib.rs`

````diff
--- crates/singlefs-replay-prefix/src/lib.rs	2026-10-02 11:32:52.309117579 +0000
+++ crates/singlefs-replay-prefix/src/lib.rs	2026-10-02 18:08:49.852238634 +0000
@@ -3069,7 +3069,10 @@
     let chosen_key = root_choice_key_of(root_slots[chosen_slot as int]);
     match replayed {
         Ok(outcome) => {
-            // 每一条自证过的根 ≤ 所选根（择根核）≤ 落点（J1）：择新序可传递。
+            // 每一条自证过的根 ≤ 所选根（择根核）≤ 落点：择新序可传递。后一个 ≤ 由 requires 里 replay_core 的 M2
+            // （replay_follows_the_prefix_rules）推得出（lemma_replay_core_result_lands_no_earlier_than_the_chosen_root 只凭 M2 证它），
+            // requires 里另列的 J1（replay_lands_no_earlier_than_the_chosen_root）是多余前提，不是这一步非用不可的
+            // （代码轮第一轮判决「改法落地」一节，research/prompts/m3-formal-code-r1-main-verification.md）。
             assert forall|slot_index: int|
                 0 <= slot_index < root_slots.len() && (#[trigger] root_slots[slot_index]) is Some
                     implies version_is_not_later_than(
````

