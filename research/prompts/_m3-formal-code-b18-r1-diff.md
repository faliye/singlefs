# 第 18 批代码轮被判的改动全文（2026-10-04）

18-1、18-2 两份补丁原样（已打进主仓，合入验证全绿）与它们的变异增删。

## 18-1 `crates.patch`

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index c615b53..bf66bc6 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -78,6 +78,7 @@ use singlefs_harness::mount_transparency::{
     ComparedVersion, DiskAfterTheRefusal, FieldOutcome, FloorRaisedByTheMount,
     FloorRaisesInTheMountWrites, MountTransparency, MountTransparencyAction,
     ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage, RecoveryFieldsRead,
+    RefusalConditionNotMet, RefusalReasonOnTheD23List,
     UserVisibleTreeRootPointersOfTheContinuation, WritableMountRefusalVerdict,
 };
 use singlefs_harness::numeric_environment_variable::{
@@ -526,6 +527,10 @@ pub struct CrashInjectionTally {
     pub remounts_after_a_second_crash_reading_the_same_content: u64,
     pub remounts_after_a_second_crash_reading_different_content: u64,
     pub remount_idempotence_not_judged: BTreeMap<RemountIdempotenceNotJudged, u64>,
+    /// 二次崩溃之后再挂被拒、拒因是判据 N-配置 的（第三截判 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那一条）：判过的、
+    /// 崩溃态没注入读故障而判据在二次崩溃镜像上复算仍为真、拒得不合法的（判红）。
+    pub remount_refusals_for_a_newer_publish_witnessed_by_system_configuration: u64,
+    pub remount_refusals_with_the_witness_criterion_true_without_read_faults: u64,
     /// 每一截（崩溃镜像本身、挂载与一次发布之后、二次崩溃）判到已知红与新发现的状态数之和。
     pub crash_states_ending_known_red: BTreeMap<usize, u64>,
     pub crash_states_ending_new_finding: u64,
@@ -602,6 +607,8 @@ impl CrashInjectionTally {
             remounts_after_a_second_crash_reading_the_same_content,
             remounts_after_a_second_crash_reading_different_content,
             remount_idempotence_not_judged,
+            remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
+            remount_refusals_with_the_witness_criterion_true_without_read_faults,
             crash_states_ending_known_red,
             crash_states_ending_new_finding,
             mount_transparency_judged,
@@ -701,6 +708,10 @@ impl CrashInjectionTally {
                 .entry(*reason)
                 .or_insert(0) += count;
         }
+        self.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration +=
+            remount_refusals_for_a_newer_publish_witnessed_by_system_configuration;
+        self.remount_refusals_with_the_witness_criterion_true_without_read_faults +=
+            remount_refusals_with_the_witness_criterion_true_without_read_faults;
         for (form, count) in crash_states_ending_known_red {
             *self.crash_states_ending_known_red.entry(*form).or_insert(0) += count;
         }
@@ -892,6 +903,12 @@ impl CrashInjectionTally {
                 .collect::<Vec<_>>(),
             self.remount_idempotence_not_judged
         );
+        let _ = writeln!(
+            text,
+            "二次崩溃之后再挂被拒、拒因是判据 N-配置 的（第三截判 C670 那一条）：{} 次，崩溃态没注入读故障而判据复算仍为真、拒得不合法 {}",
+            self.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
+            self.remount_refusals_with_the_witness_criterion_true_without_read_faults
+        );
         let _ = writeln!(
             text,
             "挂载透明：第一次崩溃镜像上判了 {:?}，二次崩溃镜像上按段判了 {:?}；没判的 {:?}；判红的字段 {:?}；某一样没判的 {:?}",
@@ -1362,6 +1379,7 @@ pub fn inject_crashes_into_history(
                 parameters: &parameters,
                 space_admission: execution.space_admission,
                 reference_filesystem_at_the_first_crash: &reference_filesystem_judgement,
+                read_faults_in_the_first_crash_state: read_faults_in_the_crash_state,
             },
             history.seed,
             crash_point,
@@ -1557,13 +1575,16 @@ pub enum WritableMountAlone {
         instance: InstanceGeneration,
         pool_after: MemoryPool,
     },
+    /// 被拒：成员、它说的理由、那个错误本身（拒得合不合法按它判）与拒之后的池（盘变没变按它比）。
     Refused {
         member: String,
         reason: ObservedRefusalReason,
+        refusal: Box<MountError>,
+        pool_after: MemoryPool,
     },
 }
 
-/// 在 `pool` 的副本上只起一次可写挂载（建池那一份参数、给定的空间准入开关），不接发布；交回挂载之后的池或拒的理由。
+/// 在 `pool` 的副本上只起一次可写挂载（建池那一份参数、给定的空间准入开关），不接发布；交回挂载之后的池，或拒的理由、错误与拒之后的池。
 #[must_use]
 pub fn writable_mount_alone_on(
     pool: &MemoryPool,
@@ -1582,20 +1603,28 @@ pub fn writable_mount_alone_on(
             (*identity, device)
         })
         .collect();
-    match mount_writable_with_space_admission(parameters, &mut devices, space_admission) {
+    let mounted_instance =
+        match mount_writable_with_space_admission(parameters, &mut devices, space_admission) {
+            Err(error) => Err(error),
+            Ok(mounted) => Ok(mounted.output.instance),
+        };
+    let pool_after = MemoryPool {
+        devices: devices
+            .into_iter()
+            .map(|(identity, device)| (identity, device.image))
+            .collect(),
+        device_size_in_bytes: pool.device_size_in_bytes,
+    };
+    match mounted_instance {
         Err(error) => WritableMountAlone::Refused {
             member: format!("{error:?}"),
             reason: refusal_reason_of_mount_error(&error),
+            refusal: Box::new(error),
+            pool_after,
         },
-        Ok(mounted) => WritableMountAlone::Mounted {
-            instance: mounted.output.instance,
-            pool_after: MemoryPool {
-                devices: devices
-                    .into_iter()
-                    .map(|(identity, device)| (identity, device.image))
-                    .collect(),
-                device_size_in_bytes: pool.device_size_in_bytes,
-            },
+        Ok(instance) => WritableMountAlone::Mounted {
+            instance,
+            pool_after,
         },
     }
 }
@@ -2065,7 +2094,8 @@ enum PublishAfterTheMountInTheState {
 }
 
 /// 判可写挂载被拒合不合法要的：崩溃后镜像（拒的判据在它上面复算，拒之后的池要与它逐字节相同）、它上面只读恢复落到的那一版、
-/// 那一次只读读出在不在模型的允许集合里（第一截问模型的结论）。
+/// 那一次只读读出在不在模型的允许集合里（问模型的结论）。第二截交第一次崩溃的镜像与第一截的结论；第三截交二次崩溃镜像与那一个二次崩溃上的结论，
+/// 读故障的认定沿用第一次崩溃那个态的（第二截那次挂载起在第一次崩溃的镜像上，不注入读故障）。
 struct RefusalOnTheCrashImage<'crash> {
     crash_image_materialized: &'crash MemoryPool,
     version_landed_on_after_the_crash: Option<(InstanceGeneration, CheckpointTxg)>,
@@ -2578,6 +2608,8 @@ struct SecondCrashInputs<'history> {
     space_admission: SpaceAdmission,
     /// RefFS 在第一次崩溃上的判定：判绿时带着以读出的内容重新起头的 RefFS，二次崩溃接着它判。
     reference_filesystem_at_the_first_crash: &'history ReferenceFileSystemJudgementOfACrash,
+    /// 第一次崩溃那个态注没注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：二次崩溃之后再挂被拒时判 C670 那一条沿用它。
+    read_faults_in_the_first_crash_state: ReadFaultsInTheCrashState,
 }
 
 /// 第三截：挂载途中的二次崩溃。每个二次崩溃状态上：只读恢复（看 journal）、问模型——允许的版本是模型提交过的每一版，加上这次挂载写出的根
@@ -2608,6 +2640,7 @@ fn judge_second_crashes_inside_the_writable_mount(
         parameters,
         space_admission,
         reference_filesystem_at_the_first_crash,
+        read_faults_in_the_first_crash_state,
     } = inputs;
     let mount_writes = records_across_the_crash.mount_writes();
     let phase_of_each_write = writable_mount_phase_of_each_write(
@@ -2690,11 +2723,21 @@ fn judge_second_crashes_inside_the_writable_mount(
             &image,
             report.effective_root,
         );
+        let second_crash_image_materialized = materialized_crash_image(&image);
         judge_the_remount_after_a_second_crash(
             tally,
             findings,
             &content_after_the_uninterrupted_mount,
-            &materialized_crash_image(&image),
+            &RefusalOnTheCrashImage {
+                crash_image_materialized: &second_crash_image_materialized,
+                version_landed_on_after_the_crash: report.effective_root,
+                read_only_read_back: if disagreement.is_none() {
+                    ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet
+                } else {
+                    ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet
+                },
+                read_faults: read_faults_in_the_first_crash_state,
+            },
             parameters,
             space_admission,
             second.phase,
@@ -2735,9 +2778,10 @@ fn judge_second_crashes_inside_the_writable_mount(
     }
 }
 
-/// I-8.4 的 R-换：二次崩溃镜像（`second_crash_image_materialized`）上再起一次可写挂载，「恢复加写行那一整段」于是做了两次；
+/// I-8.4 的 R-换：二次崩溃镜像（`second_crash_image.crash_image_materialized`）上再起一次可写挂载，「恢复加写行那一整段」于是做了两次；
 /// 再挂之后只读恢复读出的内容要与没崩过的那一次挂载之后读出的（`once`）相同。不同判红（违例记在 [`REPLAY_IDEMPOTENCE_INVARIANT`] 名下）；
-/// 条款没写的情形（[`RemountIdempotenceNotJudged`]）只计数。
+/// 条款没写的情形（[`RemountIdempotenceNotJudged`]）只计数。再挂被拒时 I-8.4 照旧不判，另判 C670 那一条
+/// （[`judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion`]）；没崩过的那一次挂载被拒（`once` 是没判的理由）时不再挂，两样都不判。
 #[allow(
     clippy::too_many_arguments,
     reason = "每个参数各是一样东西：两次挂载的输入、计数与发现、这一个二次崩溃落在哪"
@@ -2746,7 +2790,7 @@ fn judge_the_remount_after_a_second_crash(
     tally: &mut CrashInjectionTally,
     findings: &mut FindingsOfOneHistory,
     once: &Result<ContentReadByRecovery, RemountIdempotenceNotJudged>,
-    second_crash_image_materialized: &MemoryPool,
+    second_crash_image: &RefusalOnTheCrashImage<'_>,
     parameters: &MakeFilesystemParameters,
     space_admission: SpaceAdmission,
     phase: WritableMountPhase,
@@ -2765,14 +2809,36 @@ fn judge_the_remount_after_a_second_crash(
         }
         Ok(content) => content,
     };
-    let remount =
-        writable_mount_alone_on(second_crash_image_materialized, parameters, space_admission);
+    let remount = writable_mount_alone_on(
+        second_crash_image.crash_image_materialized,
+        parameters,
+        space_admission,
+    );
     let pool_after_the_remount = match &remount {
-        WritableMountAlone::Refused { .. } => {
+        WritableMountAlone::Refused {
+            member,
+            refusal,
+            pool_after: pool_after_the_refusal,
+            ..
+        } => {
             *tally
                 .remount_idempotence_not_judged
                 .entry(RemountIdempotenceNotJudged::TheRemountWasRefused)
                 .or_insert(0) += 1;
+            judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion(
+                tally,
+                findings,
+                RemountRefusedAfterASecondCrash {
+                    member,
+                    refusal,
+                    pool_after_the_refusal,
+                    second_crash_image,
+                    phase,
+                },
+                seed,
+                crash_point,
+                image_directory,
+            );
             return;
         }
         WritableMountAlone::Mounted { pool_after, .. } => pool_after,
@@ -2839,6 +2905,133 @@ fn judge_the_remount_after_a_second_crash(
     );
 }
 
+/// 二次崩溃镜像上再挂被拒的那一次：成员、错误、拒之后的池、二次崩溃镜像那一份判拒要的、崩在挂载的哪一段。
+struct RemountRefusedAfterASecondCrash<'remount> {
+    member: &'remount str,
+    refusal: &'remount MountError,
+    pool_after_the_refusal: &'remount MemoryPool,
+    second_crash_image: &'remount RefusalOnTheCrashImage<'remount>,
+    phase: WritableMountPhase,
+}
+
+/// 第三截判 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官） 那一条（攻方改法 P1，`research/prompts/m3-formal-code-b16-r1-main-verification.md`
+/// 第二节 K2-b）：二次崩溃镜像同样是一个崩溃态，再挂的拒因是判据 N-配置、崩溃态没注入读故障、判据在二次崩溃镜像上（撕裂的系统配置槽跳过）
+/// 复算仍为真 ⇒ 拒得不合法，报红（记在 [`CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount`] 那一截）。判官一并判的 U6 那四条
+/// （r5 判决第二节 U6）条款只管第二截，这一截不拿它们定红绿：只差那几条的不报红、不计不合法，判官列出的没满足的条件只随 C670 那一条的红带出来。
+/// 拒因不是判据 N-配置 的不判。
+fn judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion(
+    tally: &mut CrashInjectionTally,
+    findings: &mut FindingsOfOneHistory,
+    remount: RemountRefusedAfterASecondCrash<'_>,
+    seed: HistorySeed,
+    crash_point: &CrashPoint,
+    image_directory: Option<&Path>,
+) {
+    let RemountRefusedAfterASecondCrash {
+        member,
+        refusal,
+        pool_after_the_refusal,
+        second_crash_image,
+        phase,
+    } = remount;
+    let disk = if *pool_after_the_refusal == *second_crash_image.crash_image_materialized {
+        DiskAfterTheRefusal::ByteForByteUnchanged
+    } else {
+        DiskAfterTheRefusal::Changed
+    };
+    let verdict = judge_writable_mount_refusal_of_a_crash_state(
+        second_crash_image.crash_image_materialized,
+        second_crash_image.version_landed_on_after_the_crash,
+        refusal,
+        disk,
+        second_crash_image.read_only_read_back,
+        second_crash_image.read_faults,
+    );
+    let Some(conditions_not_met) =
+        conditions_not_met_by_a_remount_refusal_with_the_witness_criterion_true(tally, verdict)
+    else {
+        return;
+    };
+    let detail = format!(
+        "崩在挂载的 {} 里、二次崩溃镜像上再挂被拒，理由 {:?}；崩溃态没注入读故障，判据在二次崩溃镜像上复算仍为真，拒得不合法（C670；判官列出的没满足的条件 {conditions_not_met:?}）：{member}",
+        phase.name(),
+        RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
+    );
+    let model_disagreement = ModelDisagreement::new(
+        ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
+        "没注入读故障的崩溃态上，见证判据不许为真：二次崩溃之后恢复到一版上，可写挂载（取号、写行、暖机）都该成"
+            .to_string(),
+        detail.clone(),
+    );
+    let image = CrashImage {
+        base: second_crash_image.crash_image_materialized,
+        writes: &[],
+        persisted: Vec::new(),
+    };
+    let observation = crash_state_observation(
+        &image,
+        crash_point,
+        Vec::new(),
+        Some(model_disagreement),
+        RecordCheck::default(),
+    );
+    findings.settle(
+        tally,
+        FailureAtAStage {
+            seed,
+            crash_point,
+            stage: CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(phase),
+            observation,
+            read_back: detail,
+            image: &image,
+        },
+        image_directory,
+    );
+}
+
+/// 第三截那一判的计数与取舍：拒因是判据 N-配置 的记一次判过；判官列出的没满足的条件里有 C670 那一条
+/// （[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]）时再记一次不合法，交回全部没满足的条件（报红用）。
+/// 拒因不是它、C670 那一条不成立（注入过读故障、判据复算为假）、只差 U6 那几条的，交回 None。
+fn conditions_not_met_by_a_remount_refusal_with_the_witness_criterion_true(
+    tally: &mut CrashInjectionTally,
+    verdict: WritableMountRefusalVerdict,
+) -> Option<Vec<RefusalConditionNotMet>> {
+    let conditions_not_met = match verdict {
+        WritableMountRefusalVerdict::ReasonNotOnTheD23List
+        | WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable,
+        )
+        | WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable,
+            ..
+        } => return None,
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
+        ) => {
+            tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration += 1;
+            return None;
+        }
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
+            conditions_not_met,
+        } => {
+            tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration += 1;
+            conditions_not_met
+        }
+    };
+    let witness_criterion_true_without_read_faults = conditions_not_met.iter().any(|condition| {
+        matches!(
+            condition,
+            RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(_)
+        )
+    });
+    if !witness_criterion_true_without_read_faults {
+        return None;
+    }
+    tally.remount_refusals_with_the_witness_criterion_true_without_read_faults += 1;
+    Some(conditions_not_met)
+}
+
 /// 这个崩溃状态在段内扣下的写各是什么种类（同一种只记一次）。段是写表里连续的一截
 /// （`writes_and_segments_with_stream_indexes` 按次序切），第 k 个写的下标是段首加 k——靶向用例摆的持久集合越过段尾时也按这个数。
 fn withheld_write_kinds(
@@ -4912,4 +5105,284 @@ mod tests {
             "没注入读故障的崩溃态上有新发现：{new_findings:?}"
         );
     }
+
+    /// 起点（取号、暖机、第一个文件）之后一次覆盖写、不重挂的历史，与跑它用的执行（单元区 384 槽的小盘，不逐步跑 checker）。
+    fn history_of_the_first_file_and_one_overwrite() -> (GeneratedHistory, HistoryExecution) {
+        use singlefs_harness::history::{
+            ContentChoice, ContentLength, HistoryDeviceWidth, PerStepChecker,
+        };
+        (
+            GeneratedHistory {
+                seed: HistorySeed(0),
+                starting_point: HistoryStartingPoint::AfterFirstFile,
+                operations: vec![HistoryOperation::PublishOverwrite(ContentChoice {
+                    length: ContentLength::InsideOneDataUnit { selector: 0 },
+                    fill_seed: 1,
+                })],
+            },
+            HistoryExecution {
+                per_step_checker: PerStepChecker::Skipped,
+                device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
+                space_admission: SpaceAdmission::JudgedByTheFormula,
+            },
+        )
+    }
+
+    /// 那段历史的写表与切段（与 [`inject_crashes_into_history`] 自己切的同一份：同一条执行、同一份几何），和覆盖写那次发布的第一个写在写表里的下标。
+    fn writes_segments_and_first_write_of_the_overwrite(
+        history: &GeneratedHistory,
+        execution: HistoryExecution,
+    ) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, usize) {
+        let stream = SharedStream::retaining_contents();
+        let mut operations_after_the_starting_point = None;
+        let run = execute_history_with(history, execution, &stream, &mut |observation| {
+            if observation.position == StepPosition::StartingPoint {
+                operations_after_the_starting_point = Some(stream.operation_count());
+            }
+        });
+        assert_eq!(run.ending, HistoryEnding::Completed, "前提：这段历史跑完");
+        let operations_after_the_starting_point =
+            operations_after_the_starting_point.expect("起点跑完时登记过一次");
+        let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
+            &stream.retained_operations(),
+            &execution.device_width.fixed_geometry(),
+        );
+        let first_write_of_the_overwrite = stream_indexes
+            .iter()
+            .position(|stream_index| *stream_index >= operations_after_the_starting_point)
+            .expect("覆盖写那次发布写过东西");
+        (writes, segments, first_write_of_the_overwrite)
+    }
+
+    /// 覆盖写那次发布的写里只有系统配置轮换（每块盘一槽）；它还写了记录与根槽（都要扣下）。
+    fn assert_the_overwrite_rotates_one_system_configuration_slot_per_device(
+        writes_of_the_overwrite: &[RetainedWrite],
+    ) {
+        assert_eq!(
+            writes_of_the_overwrite
+                .iter()
+                .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
+                .count(),
+            2,
+            "前提：覆盖写那次发布每块盘轮换一个系统配置槽"
+        );
+        assert!(
+            writes_of_the_overwrite
+                .iter()
+                .any(|write| write.kind == StepKind::RootRecordFua)
+                && writes_of_the_overwrite
+                    .iter()
+                    .any(|write| write.kind == StepKind::JournalRecord),
+            "前提：覆盖写那次发布写了记录与根（这个崩溃态里都不落）"
+        );
+    }
+
+    /// C670 的接线（攻方改法 P2，`research/prompts/m3-formal-code-b16-r1-main-verification.md` 第二节 K2-e）：经崩溃注入的入口摆一个靶向崩溃态——
+    /// 第一个文件之后一次覆盖写，那次发布只落了系统配置轮换，单元、记录与根都没落（盘面照 harness 档
+    /// `writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 的造法；越过了根槽 FUA 与轮换的先后，今天的持久顺序下段内真子集摆不出来）。
+    /// 历史里没有带读故障的步；第二截的可写挂载以判据 N-配置 拒，盘没变、只读读出在允许集合里，U6 四条都满足，C670 那一条判不合法，计 1。
+    /// 判别力：调用点把读故障的认定恒交 `Injected`、第二截换回不判 C670 的旧判官 `judge_writable_mount_refusal`（`crates/mutations.tsv` 两行），
+    /// 这一拒都判成合法，这一条红。harness 档不许依赖 checker 档，接线在 checker 档，用例只能写在这里。
+    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个崩溃态（ExactlyOneCrashState），不枚举
+    #[test]
+    fn a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash(
+    ) {
+        let (history, execution) = history_of_the_first_file_and_one_overwrite();
+        let (writes, segments, first_write_of_the_overwrite) =
+            writes_segments_and_first_write_of_the_overwrite(&history, execution);
+        let writes_of_the_overwrite = &writes[first_write_of_the_overwrite..];
+        assert_the_overwrite_rotates_one_system_configuration_slot_per_device(
+            writes_of_the_overwrite,
+        );
+        let segment_index = segments
+            .iter()
+            .position(|segment| segment[0] == first_write_of_the_overwrite)
+            .expect("覆盖写那次发布从一段的第一个写起（它前面是起点末尾那道屏障）");
+        let withheld_among_them: Vec<usize> = writes_of_the_overwrite
+            .iter()
+            .enumerate()
+            .filter(|(_, write)| write.kind != StepKind::SystemConfigurationSlot)
+            .map(|(within_the_overwrite, _)| within_the_overwrite)
+            .collect();
+        let injection = inject_crashes_into_history(
+            &history,
+            execution,
+            CrashPointDraw::ExactlyOneCrashState {
+                segment_index,
+                writes_from_the_segment_on: writes_of_the_overwrite.len(),
+                withheld_among_them: withheld_among_them.leak(),
+            },
+            None,
+        );
+        let tally = &injection.tally;
+        assert_eq!(injection.crash_points.len(), 1, "只摆写死的那一个崩溃态");
+        assert_eq!(
+            tally.crash_points_after_a_step_reading_under_injected_faults,
+            0,
+            "这段历史没有带读故障的步：{}",
+            tally.render()
+        );
+        assert_eq!(
+            tally.model_contents_compared,
+            1,
+            "前提：只读恢复读出第一个文件那一版，在模型的允许集合里（U6 只读读出那一条满足）：{}",
+            tally.render()
+        );
+        assert_eq!(
+            (
+                tally.writable_mount_refusals_on_the_d23_list_legal,
+                tally.writable_mount_refusals_on_the_d23_list_illegal
+            ),
+            (0, 1),
+            "第二截的可写挂载以判据 N-配置 拒、崩溃态没注入读故障而判据复算为真：拒得不合法，计 1：{}",
+            tally.render()
+        );
+    }
+
+    /// C670 在第三截的判法（攻方改法 P1）：把「只落了覆盖写那次轮换」的盘面当成一个二次崩溃镜像交给第三截，没崩过的那一次挂载做成过（`once` 有内容）。
+    /// 再挂以判据 N-配置 拒；崩溃态没注入读故障、判据在这份盘面上复算为真：I-8.4 照旧记「再挂被拒」不判，C670 那一条判过一次、不合法一次，
+    /// 记一条第三截的发现。判别力：再挂被拒那一支不判 C670、或换回不判 C670 的旧判官（`crates/mutations.tsv` 两行），这一条红。
+    #[test]
+    fn a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal(
+    ) {
+        let (history, execution) = history_of_the_first_file_and_one_overwrite();
+        let (writes, _segments, first_write_of_the_overwrite) =
+            writes_segments_and_first_write_of_the_overwrite(&history, execution);
+        let writes_of_the_overwrite = &writes[first_write_of_the_overwrite..];
+        assert_the_overwrite_rotates_one_system_configuration_slot_per_device(
+            writes_of_the_overwrite,
+        );
+        let mut second_crash_image = MemoryPool::with_devices(
+            &[DeviceIdentity(0), DeviceIdentity(1)],
+            execution.device_width.device_bytes(),
+        );
+        second_crash_image.apply_writes(&writes[..first_write_of_the_overwrite]);
+        let rotation: Vec<RetainedWrite> = writes_of_the_overwrite
+            .iter()
+            .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
+            .cloned()
+            .collect();
+        second_crash_image.apply_writes(&rotation);
+        let recovered =
+            read_recovery_fields(&second_crash_image, ComparedVersion::TheRecoveredVersion);
+        let crash_point = CrashPoint {
+            segment_index: 0,
+            persisted_within_the_segment: vec![false],
+            step: StepPosition::StartingPoint,
+            operation_kind: None,
+        };
+        let mut tally = CrashInjectionTally::default();
+        let mut findings = FindingsOfOneHistory {
+            known_red_hits: Vec::new(),
+            new_findings: Vec::new(),
+        };
+        judge_the_remount_after_a_second_crash(
+            &mut tally,
+            &mut findings,
+            &Ok(content_read_by_the_recovery_report(&recovered.report)),
+            &RefusalOnTheCrashImage {
+                crash_image_materialized: &second_crash_image,
+                version_landed_on_after_the_crash: recovered.report.effective_root,
+                read_only_read_back: ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+                read_faults: ReadFaultsInTheCrashState::NoneInjected,
+            },
+            &execution.device_width.parameters(),
+            execution.space_admission,
+            WritableMountPhase::RowPublish,
+            history.seed,
+            &crash_point,
+            None,
+        );
+        assert_eq!(
+            tally
+                .remount_idempotence_not_judged
+                .get(&RemountIdempotenceNotJudged::TheRemountWasRefused)
+                .copied(),
+            Some(1),
+            "前提：再挂被拒，I-8.4 不判：{}",
+            tally.render()
+        );
+        assert_eq!(
+            (
+                tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
+                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults
+            ),
+            (1, 1),
+            "拒因是判据 N-配置、崩溃态没注入读故障而判据复算为真：判过一次、不合法一次：{}",
+            tally.render()
+        );
+        assert_eq!(
+            findings.known_red_hits.len() + findings.new_findings.len(),
+            1,
+            "第三截记一条：已知红 {:?}，新发现 {:?}",
+            findings.known_red_hits,
+            findings
+                .new_findings
+                .iter()
+                .map(CrashPointFinding::render)
+                .collect::<Vec<_>>()
+        );
+    }
+
+    /// C670 在第三截的判别力（攻方改法 P1，`research/prompts/m3-formal-code-b16-r1-main-verification.md` 第二节 K2-b）：起点加一次覆盖写、不重挂，
+    /// 短段整段枚举；历史里没有带读故障的步。二次崩溃之后再挂以判据 N-配置 被拒、判据在二次崩溃镜像上复算为真的，一个都没有。
+    /// 攻方自造的坏实现 V242u（只在带单元的发布上先轮换后根，`crates/mutations.tsv` 里那一行）在这段历史上第二截看不见（第一次崩溃镜像上 0 个：
+    /// 同一实例的记录照重放），第二截那次挂载的写行是新实例第一次带单元的发布，「轮换落了、根没落」只经二次崩溃摆得出来——攻方原型逐个枚举短段量到 108 态；
+    /// 这里每个崩溃态每段只抽一个二次崩溃，能不能抽中是推的（报告里估了抽中的概率）。
+    // crash-case-check:not-a-crash-case 起点加一步的合成历史、短段整段枚举，不调全量枚举，崩溃态数以百计（推的，没量过）
+    #[test]
+    fn no_remount_after_a_second_crash_without_read_faults_is_refused_writable_for_a_witness_newer_than_the_selected_version(
+    ) {
+        let (history, execution) = history_of_the_first_file_and_one_overwrite();
+        let injection = inject_crashes_into_history(
+            &history,
+            execution,
+            CrashPointDraw::EveryProperSubsetOfShortSegments {
+                segment_writes: 4,
+                sampled_in_longer_segments: 2,
+            },
+            None,
+        );
+        let tally = &injection.tally;
+        assert_eq!(
+            tally.crash_points_after_a_step_reading_under_injected_faults,
+            0,
+            "这段历史没有带读故障的步，每个崩溃态都判 C670 那一条：{}",
+            tally.render()
+        );
+        assert!(
+            injection.crash_points.iter().any(|crash_point| {
+                matches!(
+                    crash_point.operation_kind,
+                    Some(HistoryOperationKind::PublishOverwrite)
+                )
+            }),
+            "覆盖写那一步要摆到崩溃态：{}",
+            tally.render()
+        );
+        assert!(
+            tally
+                .second_crash_points_by_phase
+                .get(&WritableMountPhase::RowPublish)
+                .copied()
+                .unwrap_or(0)
+                >= 1,
+            "前提：写行那一段摆到了二次崩溃：{}",
+            tally.render()
+        );
+        assert_eq!(
+            tally.remount_refusals_with_the_witness_criterion_true_without_read_faults,
+            0,
+            "二次崩溃之后再挂以见证判据为由拒、崩溃态没注入读故障而判据复算为真（C670）：{:?}",
+            injection
+                .new_findings
+                .iter()
+                .filter(|finding| matches!(
+                    finding.stage,
+                    CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(_)
+                ))
+                .map(CrashPointFinding::render)
+                .collect::<Vec<_>>()
+        );
+    }
 }
````

## 18-1 变异

````
C670（18-1 P1）判别力（攻方 V242u）：只在带单元的发布上先轮换系统配置、后写根槽，第二截那次挂载的写行崩出「轮换落了、根没落」只经二次崩溃摆得出，第三截再挂判拒得不合法	crates/singlefs-core/src/transaction.rs	    writer.perform_commit_step(CommitStep::Barrier)?;\n    persist_the_root_then_rotate_the_system_configuration(\n        writer,\n        writes.checkpoint_txg,\n        &writes.root_slot,\n        writes.journal_tail,\n        writes.journal_instance,\n    )\n}	    writer.perform_commit_step(CommitStep::Barrier)?;\n    if writes.units.is_empty() {\n        return persist_the_root_then_rotate_the_system_configuration(\n            writer,\n            writes.checkpoint_txg,\n            &writes.root_slot,\n            writes.journal_tail,\n            writes.journal_instance,\n        );\n    }\n    writer.perform_commit_step(CommitStep::RotateSystemConfigurationSlots {\n        journal_tail: writes.journal_tail,\n        journal_instance: writes.journal_instance,\n    })?;\n    writer.perform_commit_step(CommitStep::WriteRootRecordForceUnitAccess {\n        checkpoint_txg: writes.checkpoint_txg,\n        root_slot: &writes.root_slot,\n    })?;\n    writer.perform_commit_step(CommitStep::Barrier)\n}	-p singlefs-checker-tier --lib -- crash_injection::tests::no_remount_after_a_second_crash_without_read_faults_is_refused_writable_for_a_witness_newer_than_the_selected_version	no_remount_after_a_second_crash_without_read_faults_is_refused_writable_for_a_witness_newer_than_the_selected_version
C670（18-1 P2）接线：第二截调用点把读故障的认定恒交 Injected（没注入读故障的崩溃态也不判见证判据）	crates/singlefs-checker-tier/src/crash_injection.rs	                read_faults: read_faults_in_the_crash_state,	                read_faults: ReadFaultsInTheCrashState::Injected,	-p singlefs-checker-tier --lib -- crash_injection::tests::a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash	a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash
C670（18-1 P2）接线：第二截换回不判 C670 的旧判官 judge_writable_mount_refusal	crates/singlefs-checker-tier/src/crash_injection.rs	            match judge_writable_mount_refusal_of_a_crash_state(\n                refusal_on_the_crash_image.crash_image_materialized,\n                refusal_on_the_crash_image.version_landed_on_after_the_crash,\n                refusal_error,\n                disk,\n                refusal_on_the_crash_image.read_only_read_back,\n                refusal_on_the_crash_image.read_faults,\n            ) {	            match singlefs_harness::mount_transparency::judge_writable_mount_refusal(\n                refusal_on_the_crash_image.crash_image_materialized,\n                refusal_on_the_crash_image.version_landed_on_after_the_crash,\n                refusal_error,\n                disk,\n                refusal_on_the_crash_image.read_only_read_back,\n            ) {	-p singlefs-checker-tier --lib -- crash_injection::tests::a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash	a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash
C670（18-1 P1）第三截：再挂被拒那一支只记 I-8.4 没判、不判 C670 那一条	crates/singlefs-checker-tier/src/crash_injection.rs	                .entry(RemountIdempotenceNotJudged::TheRemountWasRefused)\n                .or_insert(0) += 1;\n            judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion(	                .entry(RemountIdempotenceNotJudged::TheRemountWasRefused)\n                .or_insert(0) += 1;\n            if tally.remounts_after_a_second_crash > 0 {\n                return;\n            }\n            judge_the_remount_refusal_after_a_second_crash_against_the_witness_criterion(	-p singlefs-checker-tier --lib -- crash_injection::tests::a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal	a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal
C670（18-1 P1）第三截：判再挂被拒时读故障恒交 Injected（不沿用第一次崩溃那个态的认定）	crates/singlefs-checker-tier/src/crash_injection.rs	        second_crash_image.read_only_read_back,\n        second_crash_image.read_faults,\n    );	        second_crash_image.read_only_read_back,\n        ReadFaultsInTheCrashState::Injected,\n    );	-p singlefs-checker-tier --lib -- crash_injection::tests::a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal	a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal
C670（18-1 P1）第三截：判再挂被拒换回不判 C670 的旧判官 judge_writable_mount_refusal	crates/singlefs-checker-tier/src/crash_injection.rs	    let verdict = judge_writable_mount_refusal_of_a_crash_state(\n        second_crash_image.crash_image_materialized,\n        second_crash_image.version_landed_on_after_the_crash,\n        refusal,\n        disk,\n        second_crash_image.read_only_read_back,\n        second_crash_image.read_faults,\n    );	    let verdict = singlefs_harness::mount_transparency::judge_writable_mount_refusal(\n        second_crash_image.crash_image_materialized,\n        second_crash_image.version_landed_on_after_the_crash,\n        refusal,\n        disk,\n        second_crash_image.read_only_read_back,\n    );	-p singlefs-checker-tier --lib -- crash_injection::tests::a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal	a_remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal
````

## 18-2 `crates.patch`

````diff
diff --git a/crates/singlefs-harness/src/model.rs b/crates/singlefs-harness/src/model.rs
index 9c31c7b..830cd58 100644
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -2493,30 +2493,27 @@ impl IdealModel {
     /// （推到每块盘上都有一条带新 F 的根，第一版几何上 2 或 3 次），不另判上限（「抬 F 的上限」那一行「卸载抬 F」）；
     /// 做完之后这个进程不再有可写会话。
     ///
+    /// 现行那一版树表 0 条（还没写过文件）：一个字节都不写、F 不动，答零条根、会话关
+    /// （用户 2026-10-04 弹窗定，选项「乙：不推，条款照实现写」，2026-09-26 那条定案；与 `mount::Unmounted::NothingWrittenOnAVersionWithoutFile` 对得上）。
+    /// 回退候选要带文件，而这一版在它的时间线上还没有带文件的版本，抬 F 买不到东西。
+    ///
     /// 模型不比的两样：这一串的根在 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）——[`ObservedRoot`] 不带 flags；
     /// 先写系统配置那一步——模型不建系统配置。容量墙同抬 F：实现在任何写之前把这一串整串预演一遍，模型答允许拒绝的区间。
     ///
     /// # Errors
-    /// 模型里没有可写会话：会话状态与实现对不上。现行那一版树表 0 条：条款写「推一串空发布」，实现在这一版上一个字节都不写
-    /// （`mount::Unmounted::NothingWrittenOnAVersionWithoutFile`，主 agent 2026-09-26 定、没进 D16），两种答法条款没判，模型不答。
+    /// 模型里没有可写会话：会话状态与实现对不上。
     pub fn answer_unmount(&self) -> Result<ModelAnswer, ModelDisagreement> {
         let session = self.open_session()?;
         let current = &session.current;
-        if !current.contents.has_trees() {
-            return Err(ModelDisagreement::new(
-                ModelDisagreementAspect::NotModeled,
-                format!(
-                    "模型里现行版本 {:?} 树表 0 条：卸载推不推空发布条款没判，模型不答",
-                    current.key
-                ),
-                "执行器调了卸载".to_string(),
-            ));
-        }
-        let floor_raised_to_the_current_txg = current.key.checkpoint_txg;
-        let roots = self.empty_publishes_carrying_the_floor_to_every_device(
-            session,
-            floor_raised_to_the_current_txg,
-        );
+        let roots = if current.contents.has_trees() {
+            let floor_raised_to_the_current_txg = current.key.checkpoint_txg;
+            self.empty_publishes_carrying_the_floor_to_every_device(
+                session,
+                floor_raised_to_the_current_txg,
+            )
+        } else {
+            Vec::new()
+        };
         let mut answer = self.answer_for_publishes(
             ModelOperationKind::Unmount,
             BTreeSet::new(),
diff --git a/crates/singlefs-harness/tests/model_answers_unmount_with_the_floor_at_the_current_version.rs b/crates/singlefs-harness/tests/model_answers_unmount_with_the_floor_at_the_current_version.rs
index 99d3f7f..d17cd1b 100644
--- a/crates/singlefs-harness/tests/model_answers_unmount_with_the_floor_at_the_current_version.rs
+++ b/crates/singlefs-harness/tests/model_answers_unmount_with_the_floor_at_the_current_version.rs
@@ -5,7 +5,9 @@
 //! 此前三条 oracle 都放行：模型没有卸载的答，对拍停在卸载之前。
 //!
 //! 池：两块 4 GiB 的稀疏盘。mkfs → 第一次可写挂载（取号 1，零单元发布 txg 1 与暖机 txg 2，拿模型判过）→ 同一次挂载里第一个文件版本
-//! （txg 3，拿模型判过）→ 各条用例自己的后几步。每一步都由实现做、交回的东西换成观测拿模型判；判的是实现交出的东西，不另读盘。
+//! （txg 3，拿模型判过）→ 各条用例自己的后几步；树表 0 条那一版上的卸载（用户 2026-10-04 定：一个字节都不写，模型答零条根）只走第一次挂载。每一步都由实现做、交回的东西换成观测拿模型判；判的是实现交出的东西，不另读盘。
+
+use std::collections::BTreeMap;
 
 use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
@@ -321,16 +323,100 @@ fn an_unmount_whose_roots_carry_the_floor_of_the_current_version_instead_of_its_
     );
 }
 
-/// 现行那一版树表 0 条（第一次挂载之后还没写过文件）：条款写「推一串空发布」，实现一个字节都不写（主 agent 2026-09-26 定、没进 D16），
-/// 两种答法条款没判，模型不答，报「模型答不了」。
+/// 每块盘上写过的扇区与内容（逐字节比卸载前后用）。
+fn written_sectors_of_every_device(
+    pool: &MountedPoolJudgedByTheModel,
+) -> Vec<BTreeMap<u64, Vec<u8>>> {
+    pool.devices
+        .iter()
+        .map(|(_, device)| device.image.written_sectors().clone())
+        .collect()
+}
+
+/// 在现行那一版树表 0 条的 [`first_mount_on_a_fresh_pool`] 上正常卸载（影子账开着，同随机历史的执行器）：
+/// 实现一个字节都不写，交回 `NothingWrittenOnAVersionWithoutFile`；交回它。
+fn unmount_on_a_version_without_trees(pool: &mut MountedPoolJudgedByTheModel) -> Unmounted {
+    let unmounted = unmount(
+        &pool.parameters,
+        &mut pool.devices,
+        &mut pool.mounted.allocator,
+        &mut pool.mounted.current,
+        ShadowLedger::On,
+    )
+    .expect("树表 0 条的一版上正常卸载");
+    assert!(
+        matches!(
+            unmounted,
+            Unmounted::NothingWrittenOnAVersionWithoutFile { .. }
+        ),
+        "树表 0 条的一版上卸载报成「推了一串空发布」"
+    );
+    unmounted
+}
+
+/// 现行那一版树表 0 条（第一次挂载之后还没写过文件）：用户 2026-10-04 定「不推，条款照实现写」（2026-09-26 那条定案）——
+/// 实现一个字节都不写、F 不动，模型答零条根、会话关；对拍拿实现交回的零条根比，对得上；盘上逐字节不变，F 生效值还是 0。
 #[test]
-fn the_model_does_not_answer_unmount_on_a_version_without_trees() {
-    let pool = first_mount_on_a_fresh_pool();
+fn unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees() {
+    let mut pool = first_mount_on_a_fresh_pool();
+    let floor_before = pool.model.effective_rollback_floor();
+    let sectors_before = written_sectors_of_every_device(&pool);
+    let answer = pool
+        .model
+        .answer_unmount()
+        .expect("模型里会话开着、现行那一版树表 0 条也答");
+    let unmounted = unmount_on_a_version_without_trees(&mut pool);
     assert_eq!(
-        pool.model
-            .answer_unmount()
+        written_sectors_of_every_device(&pool),
+        sectors_before,
+        "树表 0 条的一版上卸载一个字节都不写"
+    );
+    // 先拿模型判（对拍那一格先红），再钉模型自己往前走的结果。
+    let counts = judged_by_the_model(
+        &mut pool.model,
+        &answer,
+        &ObservedOutcome::Succeeded(observed_unmount(&unmounted)),
+    );
+    assert_eq!(
+        (counts.successes_matched, counts.roots_compared),
+        (1, 0),
+        "卸载那一步做成、零条根可比：{counts:?}"
+    );
+    assert!(
+        !pool.model.has_writable_session(),
+        "卸载之后这个进程没有可写会话"
+    );
+    assert_eq!(
+        pool.model.effective_rollback_floor(),
+        floor_before,
+        "树表 0 条的一版上卸载不抬 F"
+    );
+    assert_eq!(floor_before, ModelCheckpointTxg(0), "mkfs 之后 F 还是 0");
+}
+
+/// 实现在树表 0 条的一版上推了空发布（假想的实现错：在带文件的池上卸载交回的那两条根，当作这一格实现交回的）：
+/// 模型答零条根，对拍报「写出的根的条数」。
+#[test]
+fn an_unmount_that_pushes_empty_publishes_on_a_version_without_trees_is_reported_at_the_publish_count(
+) {
+    let mut pool_with_a_file = first_mount_on_a_fresh_pool();
+    let first_file = publish_the_first_file_judged(&mut pool_with_a_file);
+    let raised = unmount_raising_the_floor(&mut pool_with_a_file, first_file);
+    let observed = ObservedOutcome::Succeeded(observed_unmount(
+        &Unmounted::FloorRaisedToTheCurrentVersion(raised),
+    ));
+    let pool_without_a_file = first_mount_on_a_fresh_pool();
+    let answer = pool_without_a_file
+        .model
+        .answer_unmount()
+        .expect("模型里会话开着、现行那一版树表 0 条也答");
+    assert_eq!(
+        pool_without_a_file
+            .model
+            .clone()
+            .judge_and_advance(&answer, &observed)
             .map(|_| ())
             .map_err(|disagreement| disagreement.aspect),
-        Err(ModelDisagreementAspect::NotModeled)
+        Err(ModelDisagreementAspect::PublishCount)
     );
 }
````

## 18-2 变异

````
C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：模型在树表 0 条的一版上推空发布（用户 2026-10-04 定不推：一个字节都不写、答零条根）	crates/singlefs-harness/src/model.rs	        let roots = if current.contents.has_trees() {	        let roots = if true {	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees	unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees
C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：实现在树表 0 条的一版上卸载写了东西（用户 2026-10-04 定一个字节都不写；盘上逐字节比）	crates/singlefs-core/src/mount.rs	        PoolVersion::WithoutFile(current_version_without_file) => {\n            return Ok(Unmounted::NothingWrittenOnAVersionWithoutFile {	        PoolVersion::WithoutFile(current_version_without_file) => {\n            devices[0].1.write_at(crate::address::DeviceOffsetInBytes(1 << 30), &[0xFF; 4096], crate::block_device::WriteDurability::Plain).expect("变异");\n            return Ok(Unmounted::NothingWrittenOnAVersionWithoutFile {	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees	unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees
C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：对拍不比根的条数（树表 0 条的一版上实现推了空发布不报）	crates/singlefs-harness/src/model.rs	        if expected_roots.len() != observed_roots.len() {	        if false {	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- an_unmount_that_pushes_empty_publishes_on_a_version_without_trees_is_reported_at_the_publish_count	an_unmount_that_pushes_empty_publishes_on_a_version_without_trees_is_reported_at_the_publish_count
# 删除：
C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：模型在树表 0 条的一版上也答卸载（条款没判的那一格不再停下）
````
