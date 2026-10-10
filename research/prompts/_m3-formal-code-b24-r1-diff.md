# 第 24 批代码轮改动全文（2026-10-04）

合入前备份对合入后主仓（24-1 一份补丁，加主 agent 合入后改的 B1 文件第 40 行一句注释）。

````diff
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index d3b78526..4dded222 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -2433,3 +2433,33 @@ E165 r5 第五段 --gate r2 认了但闸照取甲（KF M = 2、U = 0 历史 3 09
 E165 r5 第五段 --gate r2 认了但闸照取甲，看 config 行的闸字段	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	                    "r2" => {\n                        WitnessGateRule::SomeSlotReportsAReadErrorAndNoDeviceSelfVerifiedBothSlots\n                    }	                    "r2" => {\n                        WitnessGateRule::OneUnreadableOrTornSlotMeansUndecidable\n                    }	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_gate_option_names_the_r2_gate_in_the_config_row_only_on_the_faults_family
 E165 r5 第五段 拒因记错：取号那一刻 (b) 的拒记进闸那一栏（KF M = 2、U = 0 闸拒 91 变 99、(b) 8 变 0，被拒合计不变）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	                return Some(MountRefusalReason::AcquisitionDeviceWithoutASelfVerifiedSlot);	                return Some(MountRefusalReason::TheGateAfterTheOneReread);	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_r2_refusals_of_the_kf_two_mount_cell_split_into_91_at_the_gate_8_without_a_self_verified_slot_and_0_undecidable
 E165 r5 第五段 R2 档的判别力自证仍用取甲那四条（SN0-TP 在 R2 下历史有一部分放行，取甲那一条判「全拒」为假）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	            gate_refusal_proof_under_r2(gate_on, gate_off)	            gate_refusal_proof_under_the_implementation_choice(gate_on, gate_off)	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_r2_gate_refusal_proof_holds_under_r2_and_the_implementation_choice_proof_holds_only_under_the_implementation_choice
+F3-d（24-1）：二次崩溃种子的指纹只折段内前 12 个写（长 28 的段前 16 个写都落了、末 12 个写取 4095 种时只剩 1 个种子；原文同 F3-c（23-1）那一行）	crates/singlefs-checker-tier/src/crash_injection.rs	    persisted_within_the_segment.iter().fold(\n        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,	    persisted_within_the_segment.iter().take(12).fold(\n        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,	-p singlefs-checker-tier --lib -- crash_injection::tests::first_crash_states_of_a_segment_of_twenty_eight_writes_differing_only_in_the_last_twelve_seed_their_second_crashes_differently	first_crash_states_of_a_segment_of_twenty_eight_writes_differing_only_in_the_last_twelve_seed_their_second_crashes_differently
+F3-d（24-1）：二次崩溃种子的指纹只折段内前 16 个写（长 28 的段前 16 个写都落了、末 12 个写取 4095 种时只剩 1 个种子；原文同 F3-c（23-1）那一行）	crates/singlefs-checker-tier/src/crash_injection.rs	    persisted_within_the_segment.iter().fold(\n        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,	    persisted_within_the_segment.iter().take(16).fold(\n        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,	-p singlefs-checker-tier --lib -- crash_injection::tests::first_crash_states_of_a_segment_of_twenty_eight_writes_differing_only_in_the_last_twelve_seed_their_second_crashes_differently	first_crash_states_of_a_segment_of_twenty_eight_writes_differing_only_in_the_last_twelve_seed_their_second_crashes_differently
+F3-d（24-1）：二次崩溃种子的指纹改成按位拼接（逐写左移一位或上这一位：段号与子集掩码在低位撞，段号 0..=63 的 262080 个第一次崩溃态撞成 4096 个）	crates/singlefs-checker-tier/src/crash_injection.rs	            (fingerprint ^ u64::from(*is_persisted))\n                .wrapping_mul(PERSISTED_SUBSET_FINGERPRINT_PRIME)	            (fingerprint << 1) | u64::from(*is_persisted)	-p singlefs-checker-tier --lib -- crash_injection::tests::first_crash_states_of_segments_zero_to_sixty_three_seed_their_second_crashes_differently_across_segments	first_crash_states_of_segments_zero_to_sixty_three_seed_their_second_crashes_differently_across_segments
+F3-d（24-1）：二次崩溃种子不异或段号（同一子集在段号 0..=63 上撞成一个，262080 个只剩 4095 个）	crates/singlefs-checker-tier/src/crash_injection.rs	            ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64")	            ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64") * 0	-p singlefs-checker-tier --lib -- crash_injection::tests::first_crash_states_of_segments_zero_to_sixty_three_seed_their_second_crashes_differently_across_segments	first_crash_states_of_segments_zero_to_sixty_three_seed_their_second_crashes_differently_across_segments
+F3-d（24-1）：二次崩溃种子不异或历史种子（同一子集在历史种子 0 与 1 上撞成一个，8190 个只剩 4095 个）	crates/singlefs-checker-tier/src/crash_injection.rs	        seed.0\n            ^ SECOND_CRASH_POINT_SEED_SALT	        (seed.0 & 0)\n            ^ SECOND_CRASH_POINT_SEED_SALT	-p singlefs-checker-tier --lib -- crash_injection::tests::first_crash_states_under_history_seeds_zero_and_one_seed_their_second_crashes_differently	first_crash_states_under_history_seeds_zero_and_one_seed_their_second_crashes_differently
+F3-d（24-1）：二次崩溃种子的四样由异或改成或（段号 0..=63 的 262080 个第一次崩溃态撞成 13254 个）	crates/singlefs-checker-tier/src/crash_injection.rs	        seed.0\n            ^ SECOND_CRASH_POINT_SEED_SALT\n            ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64")\n            ^ fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment),	        seed.0\n            | SECOND_CRASH_POINT_SEED_SALT\n            | u64::try_from(crash_point.segment_index).expect("段号装得进 u64")\n            | fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment),	-p singlefs-checker-tier --lib -- crash_injection::tests::first_crash_states_of_segments_zero_to_sixty_three_seed_their_second_crashes_differently_across_segments	first_crash_states_of_segments_zero_to_sixty_three_seed_their_second_crashes_differently_across_segments
+崩溃注入第二截的发布臂（24-1 改法 B1-c）：盘面挪到单元区 133 槽（攻方 GREEN-KILLS-SOME-ONLY 那一类：用例照绿而「建 0 个」不红），前提「同一份盘面新挂一次，那一支建 0 个能做成」转假	crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs	const UNIT_AREA_SLOTS: u64 = 136;	const UNIT_AREA_SLOTS: u64 = 133;	-p singlefs-checker-tier --test crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short	the_publish_after_a_mount_whose_deletion_continuation_stays_short_creates_an_inode_and_is_refused_at_the_unit_area_wall
+E171R1 装置 O1-崩溃判官的结果丢掉（照抄原装置那一行，原登记第九节 X4）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    match crash_recovery_disagreement(committed_versions, newest_persisted_root_key, read_back) {	    match crash_recovery_disagreement(committed_versions, newest_persisted_root_key, read_back).filter(|_| false) {	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_read_back_of_a_version_the_model_never_committed_is_red_and_a_committed_one_is_green
+E171R1 装置 O1-对拍判出的不同丢掉（照抄原装置那一行，原登记第九节 X5）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            answer(&self.model).and_then(|answer| self.model.judge_and_advance(&answer, &observed));	            answer(&self.model).and_then(|answer| self.model.judge_and_advance(&answer, &observed).or_else(|_| Ok(Default::default())));	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	an_implementation_refusal_the_model_does_not_allow_is_recorded_as_a_disagreement
+E171R1 装置 O2 把 Violated 当 Holds（照抄原装置那一行，原登记第九节 X6）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    verdicts\n        .iter()\n        .filter(|(_, verdict)| match verdict {\n            InvariantVerdict::Violated(_) => true,	    verdicts\n        .iter()\n        .filter(|(_, verdict)| match verdict {\n            InvariantVerdict::Violated(_) => false,	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	only_a_violated_invariant_counts_as_a_red_pool_checker_verdict
+E171R1 装置批 1 漏掉全持久态（照抄原装置那一行，原登记第九节 X13）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    first.insert(state_count - 1);	    let _ = state_count;	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	sampled_batches_are_disjoint_and_include_the_last_state_and_up_to_four_torn_states
+E171R1 装置 O1-崩溃判官每个触达了的态都记不适用（照抄原装置那一行，原登记第九节 X14）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    !touched_call.is_some_and(|call| model_stopped_at_call.is_some_and(|stopped| call >= stopped))	    !touched_call.is_some_and(|_| true)	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_crash_judge_is_not_applicable_only_from_the_call_where_the_model_stopped
+E171R1 装置 取号或暖机被拒不记成 O1-对拍的红（照抄原装置那一行，代码轮第三轮判决 K4-c 甲）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        self.calls[call_index].model = ModelVerdictOfCall::Disagrees {\n            aspect:\n                singlefs_harness::model::ModelDisagreementAspect::RefusedWhenModelRequiresSuccess	        let _discarded = ModelVerdictOfCall::Disagrees {\n            aspect:\n                singlefs_harness::model::ModelDisagreementAspect::RefusedWhenModelRequiresSuccess	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_refused_acquire_on_a_stream_without_faults_is_recorded_as_a_comparison_red_and_stops_the_model
+E171R1 装置 关掉 O1 时被拒的取号照记红（照抄原装置那一行）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        if !self.model_is_enabled() {\n            return;\n        }\n        let call_index = self.calls.len() - 1;\n        self.calls[call_index].model = ModelVerdictOfCall::Disagrees {\n            aspect:\n	        let call_index = self.calls.len() - 1;\n        self.calls[call_index].model = ModelVerdictOfCall::Disagrees {\n            aspect:\n	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	with_the_model_switched_off_a_refused_acquire_is_not_a_comparison_red
+E171R1 装置 取号 panic 也记成 O1-对拍的红（照抄原装置那一行；第 1 次重跑读法 5：panic 记实现自崩、不比）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        let Some(failure) = failure else {\n            return;\n        };\n        if !self.model_is_enabled() {	        let failure = failure.expect("panic 也记");\n        if !self.model_is_enabled() {	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_panic_in_the_acquire_is_an_implementation_crash_and_not_a_comparison_red
+E171R1 装置 逐行 S5 把返回了错误的调用改成只数 panic 的调用（照抄原装置那一行）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        .filter(|call| matches!(call.outcome, CallOutcome::ReturnedError(_)))	        .filter(|call| matches!(call.outcome, CallOutcome::Panicked(_)))	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_row_s5_counts_the_errored_calls_left_unjudged_and_the_not_applicable_states_from_another_source
+E171R1 装置 逐行 S5 把不适用的来源恒算合规（照抄原装置那一行，锚点换成读法 6 三处来源那一段）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        calls.get(stopped).is_some_and(|call| {	        calls.get(stopped).is_some_and(|_| true) || calls.get(stopped).is_some_and(|call| {	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_row_s5_counts_the_errored_calls_left_unjudged_and_the_not_applicable_states_from_another_source
+E171R1 装置 逐行那一趟不判 S5（照抄原装置那一行）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            if export.ending.is_some() && switches.disabled_oracle.is_none() {	            if false && export.ending.is_some() && switches.disabled_oracle.is_none() {	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_row_run_judges_the_row_s5_and_the_baseline_run_judges_the_baseline_s5
+E171R1 装置 关掉 O1 的逐行那一趟照判 S5（照抄原装置那一行）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            if export.ending.is_some() && switches.disabled_oracle.is_none() {	            if export.ending.is_some() {	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_row_run_judges_the_row_s5_and_the_baseline_run_judges_the_baseline_s5
+E171R1 装置 逐行 S5 又把模型停下之后的错误调用数成「没比」（照抄原装置那一行）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        .filter(|call| matches!(call.model, ModelVerdictOfCall::NotModelled))	        .filter(|call| matches!(call.model, ModelVerdictOfCall::NotModelled | ModelVerdictOfCall::SkippedAfterTheModelStopped))	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_row_s5_counts_the_errored_calls_left_unjudged_and_the_not_applicable_states_from_another_source
+E171R1 装置 V1 卸载那一步不调模型的卸载答（第 1 次重跑登记第九节 V1：模型停在卸载）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	                self.driver.judge(\n                    |model| model.answer_unmount(),\n                    ObservedOutcome::Succeeded(observed),\n                );	                let _ = observed;\n                self.driver.stop_the_model_at(self.driver.calls.len() - 1);	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_unmount_is_compared_with_the_model_and_its_control_turns_only_the_unmount_red
+E171R1 装置 V2 交回值摘要取成常量（第九节 V2）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        whole: sha256_hexadecimal(text.as_bytes()),	        whole: sha256_hexadecimal(b"constant"),	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_return_value_digest_names_every_top_level_field_and_changes_with_one_field
+E171R1 装置 V3 内存对象不记分配器那一项（第九节 V3）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        ("allocator", debug_text(allocator)),\n		-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_memory_objects_are_the_allocator_the_instance_the_warm_up_the_current_version_and_the_mount_output
+E171R1 装置 V5 态上的恢复 panic 之后的态不再判（第九节 V5）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	                    remaining = panicking_ordinal + 1..remaining.end;	                    remaining = remaining.end..remaining.end;	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_state_whose_recovery_panics_is_noted_and_the_states_after_it_are_still_judged
+E171R1 装置 V6 RefFS 的票恒为绿（第九节 V6）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    let mut crash_falls_in = None;	    let mut crash_falls_in = None;\n    if moment != ReferenceFileSystemCrashMoment::InsideStep(StepPosition::StartingPoint) {\n        return ReferenceFileSystemVoteOfAState::Green { allowed: String::new() };\n    }	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_crash_in_the_segment_the_publish_closes_falls_inside_that_publish
+E171R1 装置 V7 RefFS 的崩溃时刻把关段那道屏障归到下一次调用（第九节 V7）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            stream_length_after_the_step: call.return_point,	            stream_length_after_the_step: call.first_operation,	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_crash_in_the_segment_the_publish_closes_falls_inside_that_publish
+E171R1 装置 V8 在途的发布调用记「成」（第九节 V8）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        CallOutcome::ReturnedError(_)\n        | CallOutcome::Panicked(_)\n        | CallOutcome::InFlightAtTheTruncation => OperationOutcome::FailedAfterWriting,\n    };\n    match call.label {	        CallOutcome::InFlightAtTheTruncation => OperationOutcome::Succeeded,\n        CallOutcome::ReturnedError(_) | CallOutcome::Panicked(_) => OperationOutcome::FailedAfterWriting,\n    };\n    match call.label {	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	an_overwrite_in_flight_at_the_truncation_leaves_the_confirmed_version_allowed
+E171R1 装置 V9 流三建 inode 停在叶容器变成两片的那一次（第九节 V9 的同形：少建最后那一次）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	                .is_some_and(|split| self.inode_creations == split + 1)	                .is_some_and(|split| self.inode_creations == split)	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_third_stream_stops_one_creation_after_the_inode_leaf_splits
+E171R1 装置 V10 探针照「注入了读故障」交第二截（第九节 V10）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	const READ_FAULTS_OF_A_LAYER0_STATE: ReadFaultsInTheCrashState =\n    ReadFaultsInTheCrashState::NoneInjected;	const READ_FAULTS_OF_A_LAYER0_STATE: ReadFaultsInTheCrashState =\n    ReadFaultsInTheCrashState::Injected;	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_witness_control_is_refused_for_the_witness_and_judged_illegal_by_the_second_crop
+E171R1 装置 V12 Q5 那一形不把根槽 FUA 改成普通写（第九节 V12）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            changed.operation.kind = RecordedOperationKind::Write;\n		-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_confirmed_loss_form_drops_the_barriers_makes_the_root_write_plain_and_truncates
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index 1e7760e0..b98ebf13 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -4430,6 +4430,125 @@ mod tests {
         );
     }
 
+    /// F3-d 三条用例变的是段末这么多个写的持久子集。
+    const WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT: usize = 12;
+
+    /// 段末 [`WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT`] 个写的持久子集里扣下集合非空的那几种（全持久不是崩在这一段里的态）：4095 种。
+    const FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT: usize =
+        (1 << WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT) - 1;
+
+    /// 一个段上的第一次崩溃态逐个算二次崩溃的种子：段号 `segment_index`，段内前 `writes_landed_before_the_varied_end` 个写都落了，
+    /// 段末 [`WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT`] 个写取扣下集合非空的全部 4095 种（段末第 k 个写落没落取掩码第 k 位），
+    /// 次序照掩码从 0 起。F3-d 三条用例共用这一份搭建，各自数种子、各自断言。
+    fn seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
+        seed: HistorySeed,
+        segment_index: usize,
+        writes_landed_before_the_varied_end: usize,
+    ) -> Vec<SecondCrashPointSeed> {
+        (0..FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT)
+            .map(|persisted_mask_of_the_varied_end| {
+                seed_of_the_second_crash_points(
+                    seed,
+                    &CrashPoint {
+                        segment_index,
+                        persisted_within_the_segment: std::iter::repeat_n(
+                            true,
+                            writes_landed_before_the_varied_end,
+                        )
+                        .chain((0..WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT).map(
+                            |write_within_the_varied_end| {
+                                (persisted_mask_of_the_varied_end >> write_within_the_varied_end)
+                                    & 1
+                                    == 1
+                            },
+                        ))
+                        .collect(),
+                        step: StepPosition::Operation(0),
+                        operation_kind: Some(HistoryOperationKind::PublishOverwrite),
+                    },
+                )
+            })
+            .collect()
+    }
+
+    /// 二次崩溃的种子跨段号也两两不同（攻方改法 F3-d，判决 `research/prompts/m3-formal-code-b23-r1-main-verification.md` 第二节
+    /// 「K1 F3-c 4095 断言的判别力」与第三节第 2 条）：历史种子 0，段号 0..=63 各取长 12 的段上扣下集合非空的 4095 个第一次崩溃态，
+    /// 64 × 4095 = 262080 个种子合成一个集合数，两两不同——不只是同一段号里各自不同，段号 a 上的一个子集与段号 b 上的另一个子集也不撞
+    /// （攻方 `f3_fix` 的 A 那一格就是这样数的：`research/prompts/m3-formal-code-b23-r1-opus-model/probe/opus_probe.rs`）。
+    /// 上一条固定段号 16，种子里去掉段号、或四样改成或，它照绿。
+    /// 判别力：种子不异或段号、指纹改成按位拼接（段号与子集掩码在低位撞）、四样改成或（`crates/mutations.tsv` 各一行）时这一条红；
+    /// 攻方在模型上量过三种各剩 4095、4096、13254 个（`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt` 的 A 那一列）。
+    #[test]
+    fn first_crash_states_of_segments_zero_to_sixty_three_seed_their_second_crashes_differently_across_segments(
+    ) {
+        const SEGMENT_INDEXES: std::ops::RangeInclusive<usize> = 0..=63;
+        let seeds: Vec<SecondCrashPointSeed> = SEGMENT_INDEXES
+            .flat_map(|segment_index| {
+                seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
+                    HistorySeed(0),
+                    segment_index,
+                    0,
+                )
+            })
+            .collect();
+        assert_eq!(
+            seeds.len(),
+            SEGMENT_INDEXES.count() * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
+            "前提：段号 0..=63 各 4095 个第一次崩溃态，共 262080 个"
+        );
+        let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
+        assert_eq!(
+            distinct_seeds.len(),
+            seeds.len(),
+            "历史种子 0、段号 0..=63、长 12 的段上 262080 个第一次崩溃态算出的二次崩溃种子跨段号也两两不同"
+        );
+    }
+
+    /// 长于 12 的段末 12 个写也折进种子（F3-d，同上判决）：历史种子 0、段号 16、长 28 的段，前 16 个写都落了、末 12 个写取扣下集合非空的
+    /// 4095 种，4095 个种子两两不同。真历史里的候选段长到 26（384 槽）/ 30（4 GiB）个写（攻方 `research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-segments.txt`），
+    /// 长 12 的段只看得见前 12 个写：只折前 12 / 16 个写的指纹在上一条照绿，在这里末 12 个写全被丢掉。
+    /// 判别力：指纹只折段内前 12 个写、只折前 16 个写（`crates/mutations.tsv` 各一行）时 4095 个第一次崩溃态只剩 1 个种子，这一条红
+    /// （攻方量过，`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt` 的 B 那一列）。
+    #[test]
+    fn first_crash_states_of_a_segment_of_twenty_eight_writes_differing_only_in_the_last_twelve_seed_their_second_crashes_differently(
+    ) {
+        const WRITES_LANDED_BEFORE_THE_VARIED_END: usize = 16;
+        let seeds = seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
+            HistorySeed(0),
+            16,
+            WRITES_LANDED_BEFORE_THE_VARIED_END,
+        );
+        let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
+        assert_eq!(
+            distinct_seeds.len(),
+            FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
+            "长 28 的段、前 16 个写都落了、末 12 个写取 4095 种：二次崩溃种子两两不同"
+        );
+    }
+
+    /// 二次崩溃的种子随历史种子变（F3-d，同上判决）：段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0 与 1 上各算一遍，
+    /// 8190 个种子合成一个集合数，两两不同。
+    /// 判别力：种子不异或历史种子（`crates/mutations.tsv` 一行）时两个历史种子上同一子集撞成一个，这一条红；
+    /// 攻方在模型上量过它剩 4095 个，按位拼接剩 4096 个、四样改成或剩 5998 个（`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt` 的 C 那一列）。
+    #[test]
+    fn first_crash_states_under_history_seeds_zero_and_one_seed_their_second_crashes_differently() {
+        const HISTORY_SEEDS: [HistorySeed; 2] = [HistorySeed(0), HistorySeed(1)];
+        let seeds: Vec<SecondCrashPointSeed> = HISTORY_SEEDS
+            .into_iter()
+            .flat_map(|seed| {
+                seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
+                    seed, 16, 0,
+                )
+            })
+            .collect();
+        let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
+        assert_eq!(
+            distinct_seeds.len(),
+            HISTORY_SEEDS.len() * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
+            "段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0 与 1 上算出的 8190 个二次崩溃种子两两不同"
+        );
+    }
+
     /// 实例表清扫两段（D18（块里携带什么信息） 已定项 11「回收」一条）：挂载那一段的写里，第「清扫之前发了几次」个根槽写之后起是清扫抹头
     /// （FUA 写零的单元头、每块盘一道屏障），之后第一个不带 FUA 的单元写起是清扫发布（单元、屏障、记录、根槽、轮换），两段摆二次崩溃时各摆到一个。
     /// 造的写表：取号；写行那次（单元、记录、根槽、轮换）；一次暖机（单元、根槽）；抹两个头、两道屏障；清扫发布——清扫之前发了 2 次。
diff --git a/crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs b/crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs
index 93ca1eed..75ec8ee1 100644
--- a/crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs
+++ b/crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs
@@ -22,7 +22,8 @@ use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::mount::{mount_writable_with_space_admission, DeletionQueueContinuationOutcome};
 use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
 use singlefs_core::transaction::{
-    publish_unlink_intent, publish_unlink_step, PoolVersion, PoolWriter, FIRST_INODE_NUMBER,
+    publish_new_inodes, publish_unlink_intent, publish_unlink_step, PoolVersion, PoolWriter,
+    FIRST_INODE_NUMBER,
 };
 use singlefs_core::unit::data_unit_payload_capacity;
 use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
@@ -31,12 +32,13 @@ use singlefs_harness::history::{
 };
 use singlefs_harness::memory_pool::MemoryPool;
 use singlefs_harness::model::{ModelRefusalReason, ObservedRefusalReason};
+use singlefs_harness::scenario::FIXED_WRITE_TIME_SECONDS;
 
 /// 窄盘的单元区槽数（改法 B1，判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节「K2 改法 b 判别力」与第三节第 2 条）：
 /// 续做照样推满仍放不下、第一个文件那条记录 nlink = 0，而这一档上建 0 个 inode 的空发布落得下、建 1 个落不下——
 /// 攻方量过 136 槽时正确判法 `Explained(UnitAreaWall)`、只看 Some 是 `Unexplained`、那一支建 0 个是 `Published`
 /// （`research/prompts/m3-formal-code-b21-r1-opus-model/out/k2b.txt` 第 2、4、6 行）。128 槽（攻方探针按 100..=320 步 2 扫出的第一个短续做盘面）
-/// 上建 0 个也被空间准入拒、照样记 `UnitAreaWall`，分不开那一支建几个。
+/// 上建 0 个也被落点拒（`PlacementRefused`）、照样记 `UnitAreaWall`，分不开那一支建几个。
 const UNIT_AREA_SLOTS: u64 = 136;
 
 /// 第一个文件之后经挂着的会话覆盖写几次。
@@ -102,6 +104,11 @@ fn crash_image_between_the_deletion_batches() -> MemoryPool {
 /// 发布臂建一个 inode，在这份挤的盘面上被单元区墙拒（`Explained(UnitAreaWall)`），不是覆盖写被写入口拒成写一个正在删的 inode（`Unexplained`）。
 /// 判别力：判法只看 `inode_record` 是不是 Some（`crates/mutations.tsv` 一行）时走覆盖写，拒因是 `Unexplained`；那一支建 0 个 inode
 /// （`crates/mutations.tsv` 一行）时空发布落得下、结局是 `Published`，不是 `PublishRefused`，红在解构结局那一处。
+/// 「建 0 个」那一行只在空发布落得下的盘面上分得开：前提块另挂一次同一份盘面、真发一次建 0 个的空发布，要做成（改法 B1-c，判决
+/// `research/prompts/m3-formal-code-b23-r1-main-verification.md` 第二节「K2 B1 对别的盘面参数」与第三节第 2 条）。攻方量过：用例绿的 76 份盘面上
+/// 这条前提成立与「建 0 个」变异红逐份同真同假；单元区 128..=133 槽（覆盖写 3 次、14 个单元、1 批）用例照绿而「建 0 个」不红，
+/// 这几档上空发布被落点拒、这条前提为假（`research/prompts/m3-formal-code-b23-r1-opus-model/out/b1-slots.txt`、`b1-neighbourhood.txt`）。
+/// 判别力：`UNIT_AREA_SLOTS` 换成 133（`crates/mutations.tsv` 一行）时用例红在这条前提上。
 // crash-case-check:not-a-crash-case 靶向用例只在写死的一份崩溃盘面上挂一次、发一次，不枚举崩溃态
 #[test]
 fn the_publish_after_a_mount_whose_deletion_continuation_stays_short_creates_an_inode_and_is_refused_at_the_unit_area_wall(
@@ -138,6 +145,32 @@ fn the_publish_after_a_mount_whose_deletion_continuation_stays_short_creates_an_
             "前提：第一个文件那条记录还在树里、正在删（nlink = 0）"
         );
     }
+    {
+        let mut devices = plain_devices_on(&crash_image);
+        let mut mounted = mount_writable_with_space_admission(
+            &parameters,
+            &mut devices,
+            execution.space_admission,
+        )
+        .expect("前提：同一份盘面再挂一次照样做成可写");
+        let PoolVersion::WithFile(current) = &mounted.current else {
+            panic!("前提：再挂一次交回的现行那一版是带文件的一版");
+        };
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        let publish_creating_no_inode = publish_new_inodes(
+            &mut writer,
+            &mut mounted.allocator,
+            current,
+            0,
+            FIXED_WRITE_TIME_SECONDS,
+            mounted.output.instance,
+        );
+        assert!(
+            publish_creating_no_inode.is_ok(),
+            "前提：同一份盘面新挂一次，那一支建 0 个 inode 的空发布做得成（「建 0 个」变异在这份盘面上才分得开）：{:?}",
+            publish_creating_no_inode.err()
+        );
+    }
     let mount_run = mount_and_publish_once_on_the_crash_image(
         &crash_image,
         &parameters,
diff --git a/crates/singlefs-core/src/root_record.rs b/crates/singlefs-core/src/root_record.rs
index d1b23a31..27244fb3 100644
--- a/crates/singlefs-core/src/root_record.rs
+++ b/crates/singlefs-core/src/root_record.rs
@@ -19,11 +19,14 @@ pub const ROOT_RECORD_FLAG_UNMOUNT_MARKER: u32 = 1 << 0;
 /// 根记录 flags 位 0 的卸载记号（D22（单元原子性怎么合成） 已定项 7）：这条根是不是正常卸载那一串空发布写的
 /// （D16（发布语义） 已定项 1「正常卸载」）。池级 checker 按它给 I-7.9（回退下界 F 不高于抬 F 的上限） 分两支判；
 /// 「记号只由卸载入口打」盘上没有别的东西核它，归层 0 录制流（C557（卸载记号只由卸载入口打没有检查））。
+/// 由记录施加出来的那一版不归下面哪一个成员固定：它取那条记录新根段里的记号（D23（journal 的角色与格式） 已定项 15，`recovery` 施加处），
+/// 记录的新根段照抄那次发布写的根上的记号——卸载那一串发布的记录带 1、施加出来是 [`UnmountMarker::WrittenByTheUnmountSequence`]，
+/// 别的发布的记录带 0。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum UnmountMarker {
     /// flags 位 0 = 1：正常卸载那一串写的根（`mount::unmount`）。
     WrittenByTheUnmountSequence,
-    /// flags 位 0 = 0：别的每一条根——mkfs 的第 0 代、暖机、写行、发布、准入抬 F、回退；由记录施加出来的那一版取那条记录新根段里的记号（D23（journal 的角色与格式） 已定项 15，`recovery` 施加处）。
+    /// flags 位 0 = 0：别的每一条根——mkfs 的第 0 代、暖机、写行、发布、准入抬 F、回退。
     NotWrittenByTheUnmountSequence,
 }
 
diff --git a/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs b/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
index a39ebba6..570f025c 100644
--- a/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
+++ b/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
@@ -56,11 +56,14 @@ const FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START: u64 = 4;
 const JOURNAL_RING_BYTES_OF_128_MEBIBYTES: u64 = 128 << 20;
 /// mkfs 的实例表那一对（U、U + 1）的释放代：实例 2 写行那次发布把它 COW 走。
 const RELEASE_TXG_OF_THE_MKFS_INSTANCE_TABLE: u64 = 3;
-/// 只做过 mkfs 的池只暖机挂载这么多次（根环每区槽数 S 加 2），第这一次挂载上写的第一个文件版本落到 U：每次只暖机的挂载发 3 版，
-/// 第 k 次挂载上的第一个文件版本 txg 3k − 1，写它之前环里最旧有效根比它早一圈（3S 槽），k = S + 2 时是 5 ≥ 释放代 3，U、U + 1 已回收
-/// （k = S + 1 时是 2 < 3，还没有）。S 取 mkfs 写进系统配置的那一个（`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`，今天 8：第 10 次挂载、txg 29）。
-/// 攻方在 S = 8 上量过翻转点 9 → 10（`research/prompts/m3-formal-code-b17-r1-opus-model/out/runA1.log` 第 15、17 行）；正推在 S = 5、6、7、8、12、16
-/// 上量过正确实现翻转在 S + 2、放宽一个 txg 的翻转在 S + 1（判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节 K1-a）。
+/// 只做过 mkfs 的池只暖机挂载这么多次（根环每区槽数 S 加 2），第这一次挂载上写的第一个文件版本落到 U：第 k 次挂载上的第一个文件版本
+/// txg 3k − 1，从 k ≥ 2 起成立（第 1 次挂载上是 3、第 2 次 5、第 3 次 8：正推量过，`research/prompts/m3-formal-code-b23-r1-sonnet-model/out/h2-txg.log`）；
+/// 写它之前环里最旧有效根比它早一圈（3S 槽），k = S + 2 时是 5 ≥ 释放代 3，U、U + 1 已回收（k = S + 1 时是 2 < 3，还没有；S ≥ 5 时两个 k 都 ≥ 6，
+/// 落在式子成立的范围里）。S 取 mkfs 写进系统配置的那一个（`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`，今天 8：第 10 次挂载、txg 29）；
+/// mkfs 这条路径上 S 只能取 5–15（`crates/singlefs-core/src/root_ring.rs` 的 `at_make_filesystem_checked_at_compile_time`：4、16 编不过）。
+/// 攻方在 S = 8 上量过翻转点 9 → 10（`research/prompts/m3-formal-code-b17-r1-opus-model/out/runA1.log` 第 15、17 行）；正推在 S = 5、6、7、8、12、15
+/// 上量过这两条用例照原样绿、放宽一个 txg 的变异红（判决 `research/prompts/m3-formal-code-b23-r1-main-verification.md` 第二节「K3 H2 前提断言」；
+/// 翻转在 S + 2、放宽一个 txg 的翻转在 S + 1 见判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节 K1-a）。
 const WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED: u64 =
     ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM + 2;
 /// 比上一个少一次（S + 1）：这一次挂载上的第一个文件版本写之前环里最旧有效根 2，比释放代 3 差一个 txg，mkfs 的实例表那一对还没过回收门槛。
diff --git a/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs b/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs
index 8c9558dc..a8087c67 100644
--- a/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs
+++ b/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs
@@ -2,7 +2,8 @@
 //! D23（journal 的角色与格式） 已定项 14 的判据 N-配置，而判据在崩溃镜像上复算（撕裂的系统配置槽跳过）仍为真 ⇒ 拒得不合法
 //! （`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state`）。注入过读故障的崩溃态不判这一条
 //! （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。某块盘两槽都撕裂那一支（一份自证过的系统配置都没有，C664）：C670 那一条
-//! 跳过撕裂的槽复算为假、不报，拒照 U6 判——系统配置见证照 R2 判得出（撕裂不算判不出），判据按「某块盘一份自证过的都没有」那一支为真，合法。
+//! 在这一形上照常比（跳过撕裂的槽取 c_见证），用例那一份盘面上复算为假、不报（换一份盘面照常比可以为真）；拒照 U6 判——系统配置见证照 R2
+//! 判得出（撕裂不算判不出），判据按「某块盘一份自证过的都没有」那一支为真，合法。
 //! 小池是单元区 384 槽的两块内存盘，第一个文件之后的盘面；崩溃镜像是内存盘面本身，读不报错。
 //! 见证判据为真的那份盘面取法照 E171（三oracle合谋测试） 第 242 行（系统配置轮换先于根槽 FUA 落盘）：一次覆盖写那次发布只落了系统配置轮换，
 //! 它的单元、记录与根都没落。
````
