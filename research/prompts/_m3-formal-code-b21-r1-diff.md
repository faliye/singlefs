# 第 21 批代码轮改动全文（2026-10-04）

合入前备份对合入后主仓（21-1、21-2 两份补丁）。

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index d5d693cf..701f908d 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -104,9 +104,16 @@ pub const CRASH_INJECTION_WORKER_THREADS_ENVIRONMENT_VARIABLE: &str =
 /// 摆崩溃状态的随机源与生成历史那一路岔开：同一个种子，历史怎么生成不受它影响，反过来也一样。
 const CRASH_POINT_SEED_SALT: u64 = 0x63_72_61_73_68_70_74_00;
 
-/// 摆挂载途中的二次崩溃的随机源，与上面两路再岔开（再按第一次崩溃的段号岔开：同一段历史上各个崩溃状态抽的不是同一串）。
+/// 摆挂载途中的二次崩溃的随机源，与上面两路再岔开（再按第一次崩溃的段号与段内子集岔开：同一段历史上各个崩溃状态抽的不是同一串，
+/// [`draw_second_crash_points`]）。
 const SECOND_CRASH_POINT_SEED_SALT: u64 = 0x73_65_63_6f_6e_64_63_72;
 
+/// 第一次崩溃那个段内子集的指纹（[`fingerprint_of_the_persisted_subset`]）取 64 位 FNV-1a：起始值。
+const PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
+
+/// 64 位 FNV-1a 的乘数。
+const PERSISTED_SUBSET_FINGERPRINT_PRIME: u64 = 0x0000_0100_0000_01b3;
+
 /// 每个工作线程摊到的片数：各段历史的长短差得远（一步就被拒的与连发四十次的），片切得比线程多，先跑完的线程接着领下一片。
 const SLICES_PER_WORKER_THREAD: usize = 4;
 
@@ -2794,8 +2801,23 @@ fn exactly_one_second_crash_point(
     }
 }
 
+/// 第一次崩溃那个段内子集的指纹：段内第 k 个写持久了折 1、没持久折 0，逐个按 64 位 FNV-1a 折进去。只用来岔开二次崩溃的随机源，
+/// 不进任何判定。
+fn fingerprint_of_the_persisted_subset(persisted_within_the_segment: &[bool]) -> u64 {
+    persisted_within_the_segment.iter().fold(
+        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,
+        |fingerprint, is_persisted| {
+            (fingerprint ^ u64::from(*is_persisted))
+                .wrapping_mul(PERSISTED_SUBSET_FINGERPRINT_PRIME)
+        },
+    )
+}
+
 /// 挂载的每一段各摆一个二次崩溃：在那一段的写里按种子挑一个 w，崩在 w 上——w 所在那一段里 w 之前的写都落了、w 没落、
 /// w 之后的写各按种子抽；更早的段整段持久，更晚的段一个都没持久（与第一次崩溃同一个枚举域）。挂载没写出某一段的写就不摆那一段。
+/// 种子是历史种子、盐、第一次崩溃的段号与它段内子集的指纹（[`fingerprint_of_the_persisted_subset`]）四样异或：同一段里不同子集的
+/// 第一次崩溃态各抽各的（攻方改法 F3，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第五节第 1 行；
+/// 只混段号时同一段里的各个第一次崩溃态用同一串随机数，挂载写表相同就摆到同一组二次崩溃）。
 fn draw_second_crash_points(
     seed: HistorySeed,
     crash_point: &CrashPoint,
@@ -2805,7 +2827,8 @@ fn draw_second_crash_points(
     let mut source = SeededRandomSource::from_seed(
         seed.0
             ^ SECOND_CRASH_POINT_SEED_SALT
-            ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64"),
+            ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64")
+            ^ fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment),
     );
     let mut chosen: BTreeSet<SecondCrashPoint> = BTreeSet::new();
     for phase in WritableMountPhase::ALL {
@@ -4305,6 +4328,60 @@ mod tests {
         );
     }
 
+    /// 二次崩溃的种子混进第一次崩溃那个段内子集（攻方改法 F3，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第五节第 1 行）：
+    /// 同一历史种子（7）、同一段号（5）、同一份挂载写表，段内子集不同（第 0 个写落了 / 第 1 个写落了，各三个写）的两个第一次崩溃态，
+    /// 摆出的二次崩溃不同。挂载写表造成两段：写行 4 个写、暖机 8 个写。怎么挑的：在 harness 档草稿探针里照
+    /// [`draw_second_crash_points`] 原样抄一份算这组输入，混子集时两份摆法在写行与暖机两段的持久集合都不同，不混时逐个相同。
+    /// 判别力：种子不混子集（`crates/mutations.tsv` 一行，指纹换成空子集的）时两份摆法逐个相同，这一条红。
+    #[test]
+    fn two_first_crash_states_differing_only_in_the_persisted_subset_draw_different_second_crashes()
+    {
+        let phase_of_each_write: Vec<WritableMountPhase> = (0..12)
+            .map(|write_index| {
+                if write_index < 4 {
+                    WritableMountPhase::RowPublish
+                } else {
+                    WritableMountPhase::WarmUp
+                }
+            })
+            .collect();
+        let segments: Vec<Vec<usize>> = vec![(0..4).collect(), (4..12).collect()];
+        let first_crash_with = |persisted_within_the_segment: Vec<bool>| CrashPoint {
+            segment_index: 5,
+            persisted_within_the_segment,
+            step: StepPosition::Operation(0),
+            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
+        };
+        let first_write_landed = first_crash_with(vec![true, false, false]);
+        let second_write_landed = first_crash_with(vec![false, true, false]);
+        let drawn_after_the_first_write_landed = draw_second_crash_points(
+            HistorySeed(7),
+            &first_write_landed,
+            &segments,
+            &phase_of_each_write,
+        );
+        let drawn_after_the_second_write_landed = draw_second_crash_points(
+            HistorySeed(7),
+            &second_write_landed,
+            &segments,
+            &phase_of_each_write,
+        );
+        assert_eq!(
+            drawn_after_the_first_write_landed,
+            draw_second_crash_points(
+                HistorySeed(7),
+                &first_write_landed,
+                &segments,
+                &phase_of_each_write
+            ),
+            "前提：同一个第一次崩溃态摆两次，摆法相同"
+        );
+        assert_ne!(
+            drawn_after_the_first_write_landed, drawn_after_the_second_write_landed,
+            "同一历史种子、同一段号、段内子集不同的两个第一次崩溃态要摆出不同的二次崩溃"
+        );
+    }
+
     /// 实例表清扫两段（D18（块里携带什么信息） 已定项 11「回收」一条）：挂载那一段的写里，第「清扫之前发了几次」个根槽写之后起是清扫抹头
     /// （FUA 写零的单元头、每块盘一道屏障），之后第一个不带 FUA 的单元写起是清扫发布（单元、屏障、记录、根槽、轮换），两段摆二次崩溃时各摆到一个。
     /// 造的写表：取号；写行那次（单元、记录、根槽、轮换）；一次暖机（单元、根槽）；抹两个头、两道屏障；清扫发布——清扫之前发了 2 次。
@@ -4637,8 +4714,10 @@ mod tests {
     /// unlink 之后再建过 inode 的一版归带文件的一版、第一个文件那条记录不在（删除与并发那条线读码诊断；攻方随机历史种子 32768、245760 两段那一形，
     /// 判决 `research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节「越出攻击面」那一行）：第一个文件之后 unlink 它、再建一个 inode，
     /// 崩在建 inode 那次发布里——根槽落了、紧跟着的系统配置轮换没落，只读恢复落在建 inode 那一版（这段历史最后提交的一版）。
-    /// 崩溃后镜像上的可写挂载做成，之后那一次发布建一个 inode、做成，第二截没有新发现。判别力：带文件那一臂改回「一律覆盖写」
-    /// （`crates/mutations.tsv` 一行），覆盖写被拒成写一个不在树里的 inode 1，计一次发布被拒、记一条第二截的新发现，这一条红。
+    /// 崩溃后镜像上的可写挂载做成，之后那一次发布建一个 inode、做成，第二截没有新发现；发布之后那份盘只读挂载读回的 inode 号比崩溃那一版
+    /// 多恰好一个（改法 a，`research/prompts/m3-formal-code-b20-r1-main-verification.md` 第二节 K2 H1）。判别力：带文件那一臂改回「一律覆盖写」
+    /// （`crates/mutations.tsv` 一行），覆盖写被拒成写一个不在树里的 inode 1，计一次发布被拒、记一条第二截的新发现，这一条红；
+    /// 那一臂建 0 个 inode（`crates/mutations.tsv` 一行）时发布照样做成、池级 checker 照样绿，红在 inode 号多一个那一条。
     // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个崩溃态（ExactlyOneCrashState），不枚举
     #[test]
     fn the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published(
@@ -4743,6 +4822,42 @@ mod tests {
             findings_after_the_writable_mount.is_empty(),
             "第二截没有新发现：{findings_after_the_writable_mount:?}"
         );
+        // 崩溃态是写表前缀到建 inode 那次的根槽为止（段内摆到 root + 1、扣下 root + 1，之后一个都没落），在同一份盘面上照第二截那一种调法
+        // 再挂一次加一次发布，比发布之后那份池与崩溃那一份上只读挂载读回的 inode 号。
+        let mut crash_image = MemoryPool::with_devices(
+            &[DeviceIdentity(0), DeviceIdentity(1)],
+            execution.device_width.device_bytes(),
+        );
+        crash_image.apply_writes(&writes[..=root_of_the_inode_creation]);
+        let mount_run = mount_and_publish_once_on_the_crash_image(
+            &crash_image,
+            &execution.device_width.parameters(),
+            execution,
+            history.seed,
+            &injection.crash_points[0],
+        );
+        assert!(
+            matches!(mount_run.outcome, WritableMountOutcome::Published),
+            "前提：同一份盘面上再挂一次加一次发布，照样做成：{:?}",
+            mount_run.outcome
+        );
+        let inode_numbers_read_back = |pool: &MemoryPool| -> BTreeSet<u64> {
+            singlefs_core::mounted_read::mount_read_only(pool)
+                .expect("崩溃那一版与发布之后那一版都是整次落盘的版本，只读挂载做得成")
+                .mounted
+                .inode_records()
+                .iter()
+                .map(|record| record.inode)
+                .collect()
+        };
+        let inode_numbers_of_the_crashed_version = inode_numbers_read_back(&crash_image);
+        let inode_numbers_after_the_publish = inode_numbers_read_back(&mount_run.pool_after);
+        assert!(
+            inode_numbers_of_the_crashed_version.is_subset(&inode_numbers_after_the_publish)
+                && inode_numbers_after_the_publish.len()
+                    == inode_numbers_of_the_crashed_version.len() + 1,
+            "发布之后只读挂载读回的 inode 号比崩溃那一版多恰好一个：崩溃那一版 {inode_numbers_of_the_crashed_version:?}，发布之后 {inode_numbers_after_the_publish:?}"
+        );
     }
 
     /// 挂载每一段都摆得到二次崩溃的两段历史（U4，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：从第一个文件起的
@@ -6094,4 +6209,190 @@ mod tests {
             tally.render()
         );
     }
+
+    /// 第三截调用点交给判官的「所选那一版」是二次崩溃镜像上只读恢复落到的那一版（判决 `research/prompts/m3-formal-code-b20-r1-main-verification.md`
+    /// 第二节 K1「第四种接错」）：第一次崩溃与上一条同一个（覆盖写一个写都没落，只读恢复落在起点那一版）；第二截那次可写挂载的写行那次发布上
+    /// 摆一个二次崩溃——写行那次的记录（每块盘一份）没落，根槽 FUA 与紧跟着的两槽系统配置轮换落了（越过记录与根槽之间那道屏障，与上一条同一个理由）。
+    /// 二次崩溃镜像上只读恢复落在写行那一版（新实例的根），与第一次崩溃不同；它那次发布的末条记录读不出，计数器等于 c_见证 的记录也读不出：
+    /// 再挂以判据 N-配置 拒，C670 那一条成立在「两条都读不出」那一支。判别力：调用点把所选那一版交成第一次崩溃那一版（`crates/mutations.tsv` 一行）时
+    /// 计数照样 (1, 1)，判官列出的那一支换成「c_见证 大于所选那一版末条记录的计数器」，红在那一支的两条断言上。计数分不开：在 harness 档草稿探针里
+    /// 量过，三段短历史 131 个第一次崩溃态上挂载写表每个前缀与每个前缀扣一个写（177198 个二次崩溃态），以及这一个第一次崩溃态上每个前缀扣至多两个写，
+    /// 两种所选那一版的 (判过, 不合法) 没有一处不同；判官结论不同的只有扣下写行（或第一次暖机）那次记录、根槽落了的这一类，都只差这一支。
+    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
+    #[test]
+    fn a_second_crash_landing_on_the_row_publish_whose_records_were_lost_names_the_selected_version_of_the_second_crash_image(
+    ) {
+        let (history, execution) = history_of_the_first_file_and_one_overwrite();
+        let (writes, segments, first_write_of_the_overwrite) =
+            writes_segments_and_first_write_of_the_overwrite(&history, execution);
+        let overwrite_segment_index = segments
+            .iter()
+            .position(|segment| segment[0] == first_write_of_the_overwrite)
+            .expect("覆盖写那次发布从一段的第一个写起（它前面是起点末尾那道屏障）");
+        let mut first_crash_image = MemoryPool::with_devices(
+            &[DeviceIdentity(0), DeviceIdentity(1)],
+            execution.device_width.device_bytes(),
+        );
+        first_crash_image.apply_writes(&writes[..first_write_of_the_overwrite]);
+        let mount_run = mount_and_publish_once_on_the_crash_image(
+            &first_crash_image,
+            &execution.device_width.parameters(),
+            execution,
+            history.seed,
+            &CrashPoint {
+                segment_index: overwrite_segment_index,
+                persisted_within_the_segment: vec![false],
+                step: StepPosition::Operation(0),
+                operation_kind: Some(HistoryOperationKind::PublishOverwrite),
+            },
+        );
+        assert!(
+            matches!(mount_run.outcome, WritableMountOutcome::Published),
+            "前提：起点那一版上的可写挂载与之后那一次覆盖写都做成：{:?}",
+            mount_run.outcome
+        );
+        let (mount_writes, mount_segments, _mount_stream_indexes) =
+            writes_and_segments_with_stream_indexes(
+                &mount_run.mount_operations,
+                &execution.device_width.fixed_geometry(),
+            );
+        let phase_of_each_write = writable_mount_phase_of_each_write(
+            &mount_writes,
+            mount_run.publishes_before_the_deletion_queue_continuation,
+            mount_run.publishes_before_the_instance_table_sweep,
+        );
+        let root_of_the_row_publish = (0..mount_writes.len())
+            .find(|write| {
+                phase_of_each_write[*write] == WritableMountPhase::RowPublish
+                    && mount_writes[*write].kind == StepKind::RootRecordFua
+            })
+            .expect("前提：写行那次发布写了根槽");
+        let records_of_the_row_publish: Vec<usize> = (0..root_of_the_row_publish)
+            .filter(|write| {
+                phase_of_each_write[*write] == WritableMountPhase::RowPublish
+                    && mount_writes[*write].kind == StepKind::JournalRecord
+            })
+            .collect();
+        let rotation_after_the_root: Vec<usize> = (root_of_the_row_publish + 1..mount_writes.len())
+            .take_while(|write| mount_writes[*write].kind == StepKind::SystemConfigurationSlot)
+            .collect();
+        assert_eq!(
+            (
+                records_of_the_row_publish.len(),
+                rotation_after_the_root.len()
+            ),
+            (2, 2),
+            "前提：写行那次发布在根槽之前每块盘写一份记录，根槽 FUA 之后紧跟着每块盘轮换一槽系统配置"
+        );
+        let segment_of_the_first_record =
+            segment_containing_the_write(&mount_segments, records_of_the_row_publish[0]);
+        let first_write_of_the_segment = mount_segments[segment_of_the_first_record][0];
+        let last_rotation = rotation_after_the_root[rotation_after_the_root.len() - 1];
+        let withheld_among_them: Vec<usize> = records_of_the_row_publish
+            .iter()
+            .map(|record| record - first_write_of_the_segment)
+            .collect();
+        // 前提：两次崩溃之后只读恢复落的版本不同（同一份盘面在这里照摆法叠一遍）。
+        let mut second_crash_image = first_crash_image.clone();
+        second_crash_image.apply_writes(&mount_writes[..first_write_of_the_segment]);
+        let persisted_writes_of_the_second_crash: Vec<RetainedWrite> = (first_write_of_the_segment
+            ..=last_rotation)
+            .filter(|write| !records_of_the_row_publish.contains(write))
+            .map(|write| mount_writes[write].clone())
+            .collect();
+        second_crash_image.apply_writes(&persisted_writes_of_the_second_crash);
+        let version_after_the_first_crash =
+            recover(&first_crash_image, JournalPolicy::Consult).effective_root;
+        let version_after_the_second_crash =
+            recover(&second_crash_image, JournalPolicy::Consult).effective_root;
+        assert_ne!(
+            version_after_the_first_crash, version_after_the_second_crash,
+            "前提：第一次崩溃落在起点那一版、二次崩溃落在写行那一版，两次只读恢复落的版本不同"
+        );
+        let injection = inject_crashes_into_history_drawing_second_crashes(
+            &history,
+            execution,
+            CrashPointDraw::ExactlyOneCrashState {
+                segment_index: overwrite_segment_index,
+                writes_from_the_segment_on: 1,
+                withheld_among_them: &[0],
+            },
+            SecondCrashPointDraw::ExactlyOneSecondCrashState {
+                segment_index: segment_of_the_first_record,
+                writes_from_the_segment_on: last_rotation + 1 - first_write_of_the_segment,
+                withheld_among_them: withheld_among_them.leak(),
+            },
+            None,
+        );
+        let tally = &injection.tally;
+        assert_eq!(
+            (
+                tally.crash_points,
+                tally.writable_mounts_after_the_crash_succeeded,
+                tally.crash_points_after_a_step_reading_under_injected_faults
+            ),
+            (1, 1, 0),
+            "前提：只摆写死的那一个第一次崩溃态、它上面的可写挂载做成、这段历史没有带读故障的步：{}",
+            tally.render()
+        );
+        assert_eq!(
+            (
+                tally.second_crash_points,
+                tally
+                    .second_crash_points_by_phase
+                    .get(&WritableMountPhase::RowPublish)
+                    .copied()
+            ),
+            (1, Some(1)),
+            "前提：只摆写死的那一个二次崩溃，崩在写行里：{}",
+            tally.render()
+        );
+        assert_eq!(
+            (
+                tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
+                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults
+            ),
+            (1, 1),
+            "二次崩溃镜像上再挂以判据 N-配置 拒、崩溃态没注入读故障而判据复算为真：判过一次、不合法一次：{}",
+            tally.render()
+        );
+        let remount_findings: Vec<String> = injection
+            .new_findings
+            .iter()
+            .filter(|finding| {
+                matches!(
+                    finding.stage,
+                    CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(_)
+                )
+            })
+            .map(|finding| {
+                finding
+                    .observation
+                    .model_disagreement
+                    .as_ref()
+                    .unwrap_or_else(|| panic!("拒得不合法记成模型对不上：{}", finding.render()))
+                    .implementation_answer
+                    .clone()
+            })
+            .collect();
+        let [remount_finding] = remount_findings.as_slice() else {
+            panic!("第三截再挂那一判要恰好一条新发现：{remount_findings:?}");
+        };
+        let branch_of_the_second_crash_version = format!(
+            "{:?}",
+            WitnessedCounterBranchThatHolds::NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable
+        );
+        let branch_of_the_first_crash_version = format!(
+            "{:?}",
+            WitnessedCounterBranchThatHolds::WitnessedCounterAboveTheLastRecordOfTheSelectedVersion
+        );
+        assert!(
+            remount_finding.contains(&branch_of_the_second_crash_version),
+            "所选那一版是写行那一版：它的末条记录与计数器等于 c_见证 的记录都读不出，判官要列 {branch_of_the_second_crash_version}：{remount_finding}"
+        );
+        assert!(
+            !remount_finding.contains(&branch_of_the_first_crash_version),
+            "所选那一版不是第一次崩溃落的起点那一版（那一版的末条记录读得出），不许列 {branch_of_the_first_crash_version}：{remount_finding}"
+        );
+    }
 }
diff --git a/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs b/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
index f2a1d3e7..983cd97d 100644
--- a/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
+++ b/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
@@ -58,6 +58,11 @@ const JOURNAL_RING_BYTES_OF_128_MEBIBYTES: u64 = 128 << 20;
 /// 释放代 3；第 10 次挂载上的第一个文件版本 txg 29，写它之前环里最旧有效根是 29 − 24 = 5 ≥ 3，U、U + 1 已回收（第 9 次是 26 − 24 = 2 < 3，还没有）。
 /// 攻方在这一格上量过翻转点 9 → 10（`research/prompts/m3-formal-code-b17-r1-opus-model/out/runA1.log` 第 15、17 行）。
 const WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED: usize = 10;
+/// 比上一个少一次：第 9 次挂载上的第一个文件版本 txg 26，写它之前环里最旧有效根 26 − 24 = 2 < 释放代 3，mkfs 的实例表那一对还没过回收门槛。
+/// 回收门槛放松一个 txg（释放代 ≤ 下界 + 1）的坏实现在这一格提前回收、落 U（判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md`
+/// 第二节 K2-1，攻方量过 `research/prompts/m3-formal-code-b19-r1-opus-model/out/h2-slack0.txt` 第 9 行与 `h2-slack1.txt` 第 9 行）。
+const WARM_UP_ONLY_MOUNTS_ONE_SHORT_OF_RECLAIMING_THE_MKFS_INSTANCE_TABLE: usize =
+    WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED - 1;
 /// 实例 2 里接连推这么多次零单元发布之后写的第一个文件版本落到 U（攻方量过翻转点 21 → 22，
 /// `research/prompts/m3-formal-code-b17-r1-opus-model/out/runB1.log` 第 28、29 行）。
 const ZERO_UNIT_PUBLISHES_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED: u64 = 22;
@@ -539,6 +544,58 @@ fn the_first_file_after_ten_warm_up_only_mounts_lands_on_the_reclaimed_unit_area
     );
 }
 
+/// 回收门槛差一个 txg 的那一格（判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md` 第二节 K2-1）：只做过 mkfs 的池只暖机挂载 9 次，
+/// 第 9 次挂载上写第一个文件（txg 26）。mkfs 的实例表那一对释放代 3，写这一版之前环里最旧有效根是 txg 2，D3（空间分配） 已定项 7 的回收谓词
+/// （已释放 ∧ 释放代 ≤ max(F, 环里最旧有效根)）不成立，U、U + 1 还占着：实现把数据单元放在 U + 4。直接钉实现的落点——I-5.5 按这一版的分配记录现算，
+/// 放松一个 txg 的坏实现落 U 也判成立（C689），这里不靠它；池级 checker 照样一条都不红。
+#[test]
+fn the_first_file_after_nine_warm_up_only_mounts_lands_past_the_mkfs_instance_table_that_is_one_txg_short_of_reclaim(
+) {
+    let stream = SharedStream::retaining_contents();
+    let mut devices = memory_devices(&stream);
+    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
+    let unit_area_start_slot = genesis.unit_area_start.slot().0;
+    let mounted = mount_writable_repeatedly(
+        &mut devices,
+        WARM_UP_ONLY_MOUNTS_ONE_SHORT_OF_RECLAIMING_THE_MKFS_INSTANCE_TABLE,
+    );
+    let PoolVersion::WithoutFile(version) = &mounted.current else {
+        panic!("只暖机过的池上现行那一版树表 0 条")
+    };
+    let first_file = publish_the_first_file_on_the_mount(
+        &mut devices,
+        &mounted,
+        &version.root,
+        &version.record_bytes,
+    );
+    assert_eq!(
+        (
+            first_file.root.checkpoint_txg,
+            first_file.tree_identifiers.inode.0
+        ),
+        (CheckpointTxg(26), 65537),
+        "第 9 次挂载上的第一个文件版本 txg 26，树仍从 mkfs 水位发"
+    );
+    assert_eq!(
+        data_unit_slots_on_each_device(&first_file),
+        vec![
+            (
+                0,
+                unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START
+            ),
+            (
+                1,
+                unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START
+            ),
+        ],
+        "mkfs 的实例表那一对差一个 txg 才过回收门槛：实现把数据单元放在 U + 4，不提前回收 U"
+    );
+    assert_the_first_data_unit_placement_holds_and_nothing_reddens(
+        &memory_pool_of_sparse_devices(&devices),
+        "只暖机挂载 9 次之后写第一个文件",
+    );
+}
+
 /// 攻方原型形 B（z = 22）：mkfs 之后可写挂载两次（实例 2），在实例 2 里接连推 22 次零单元发布，再写第一个文件——
 /// mkfs 的实例表那一对已过回收门槛，数据单元落在 U；I-5.5 按这一版的分配记录现算，成立，池级 checker 一条都不红。
 #[test]
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 315e4049..56effc41 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -51,7 +51,10 @@ use singlefs_core::mount::{
     mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, InstanceRow,
     MountError, RollbackTarget, ShadowLedger, Unmounted,
 };
-use singlefs_core::records::{TREE_KIND_ALLOCATION, TREE_KIND_DELETED_INODES, TREE_KIND_TOMBSTONE};
+use singlefs_core::records::{
+    TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_DELETED_INODES,
+    TREE_KIND_LINEAGE, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_TOMBSTONE,
+};
 use singlefs_core::recovery::{
     instance_table_chain_of_root, recover, JournalPolicy, PoolReader, RecoveryFailure,
     RecoveryOutcome,
@@ -6113,7 +6116,7 @@ fn add_one_to_a_new_root_segment_field_of_the_first_file_record(
 /// 二·1 那一批新立的几条里，坏镜像建在**写完新池新建文件**那份干净镜像上的：每份改一处、只红它自己那一条
 /// （`research/prompts/m3-inv-b-r1-main-verification.md` 第二节、`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 7、11 行）。
 fn known_bad_images_of_pointer_journal_and_generation_fields() -> Vec<(&'static str, Mutation)> {
-    vec![
+    let mut images: Vec<(&'static str, Mutation)> = vec![
         // I-1.12：A 的 extent 树根里那条数据指针的出生树（指针偏移 34）改成另一棵发过号的树，重封、补到根：单元头里的树照旧是 extent 树。
         (
             POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
@@ -6501,9 +6504,39 @@ fn known_bad_images_of_pointer_journal_and_generation_fields() -> Vec<(&'static
                 );
             }),
         ),
-    ]
+    ];
+    // I-9.27 写 0 那一支的其余六种（判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md` 第二节 K3-3）：照种类 3 那张，
+    // A 的树表里那一种条目的头 ID 从 0 改成 inode 树条目的树 ID，每种一张，接在上面那批后面。
+    images.extend(
+        TREE_KINDS_WITH_A_ZERO_HEAD_IDENTIFIER_BESIDES_THE_ALLOCATION_RECORD_TREE.map(
+            |kind| -> (&'static str, Mutation) {
+                (
+                    HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
+                    Box::new(move |image: &mut MemoryPool| {
+                        rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
+                            image,
+                            kind,
+                            |_own_tree_identifier, _inode_tree_identifier| 0,
+                            |_own_tree_identifier, inode_tree_identifier| inode_tree_identifier,
+                        );
+                    }),
+                )
+            },
+        ),
+    );
+    images
 }
 
+/// I-9.27 头 ID 写 0 的七种（种类 3、4、6、7、8、11、12）里种类 3 之外的六种；A 的树表里每种恰好一条。
+const TREE_KINDS_WITH_A_ZERO_HEAD_IDENTIFIER_BESIDES_THE_ALLOCATION_RECORD_TREE: [u16; 6] = [
+    TREE_KIND_ACCOUNTING,
+    TREE_KIND_LIVELIST,
+    TREE_KIND_SPARSE_SIDE_TABLE,
+    TREE_KIND_DEADLIST,
+    TREE_KIND_LINEAGE,
+    TREE_KIND_LOGGED_OPERATIONS,
+];
+
 /// A 的树表（50252）里种类是 `kind` 的那一条（恰好一条）：头 ID（条目偏移 116）先核等于 `head_identifier_before(那条的树 ID, inode 树条目的树 ID)`，
 /// 再改成 `head_identifier_after(…)`，重封、补到根。别的字段一个都不动。
 fn rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
@@ -6615,16 +6648,28 @@ fn each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_inva
             "干净镜像上 {invariant} 要真被评估过且成立：{verdicts_on_the_clean_image:?}"
         );
     }
-    for (invariant, mutation) in known_bad_images_of_pointer_journal_and_generation_fields() {
-        let mut image = clean.clone();
-        mutation(&mut image);
-        let verdicts = check_pool_image(&image);
-        assert_eq!(
-            violated_invariants(&verdicts),
-            [invariant],
-            "判红的该只有 {invariant}：{verdicts:?}"
-        );
-    }
+    // 逐张判完再断言：哪几张（下标从 0 数，按清单次序）判红的不是只有它自己那一条，一次全列出来。
+    let images_not_reddening_only_their_own_invariant: Vec<String> =
+        known_bad_images_of_pointer_journal_and_generation_fields()
+            .into_iter()
+            .enumerate()
+            .filter_map(|(image_index, (invariant, mutation))| {
+                let mut image = clean.clone();
+                mutation(&mut image);
+                let verdicts = check_pool_image(&image);
+                let violated = violated_invariants(&verdicts);
+                (violated != [invariant]).then(|| {
+                    format!(
+                        "第 {image_index} 张坏镜像判红的该只有 {invariant}，实际 {violated:?}：{verdicts:?}"
+                    )
+                })
+            })
+            .collect();
+    assert_eq!(
+        images_not_reddening_only_their_own_invariant,
+        Vec::<String>::new(),
+        "每张坏镜像判红的只有它自己那一条"
+    );
 }
 
 /// 盘 0 两个系统配置槽：(偏移, 世代号)，被择的那一槽（世代号大的）在前。
diff --git a/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs b/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs
index 7ee0e3c8..7ea47c43 100644
--- a/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs
+++ b/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs
@@ -1,7 +1,8 @@
 //! C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）：崩溃注入第二截判拒可写合不合法时，崩溃态没注入读故障、拒的理由是
 //! D23（journal 的角色与格式） 已定项 14 的判据 N-配置，而判据在崩溃镜像上复算（撕裂的系统配置槽跳过）仍为真 ⇒ 拒得不合法
 //! （`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state`）。注入过读故障的崩溃态不判这一条
-//! （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）；撕裂系统配置那一支照 U6 判合法（C664、C331）。
+//! （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。某块盘两槽都撕裂那一支（一份自证过的系统配置都没有，C664）：C670 那一条
+//! 跳过撕裂的槽复算为假、不报，拒照 U6 判——系统配置见证照 R2 判得出（撕裂不算判不出），判据按「某块盘一份自证过的都没有」那一支为真，合法。
 //! 小池是单元区 384 槽的两块内存盘，第一个文件之后的盘面；崩溃镜像是内存盘面本身，读不报错。
 //! 见证判据为真的那份盘面取法照 E171（三oracle合谋测试） 第 242 行（系统配置轮换先于根槽 FUA 落盘）：一次覆盖写那次发布只落了系统配置轮换，
 //! 它的单元、记录与根都没落。
@@ -284,7 +285,8 @@ fn the_same_witness_refusal_in_a_crash_state_with_read_faults_injected_is_legal(
 }
 
 /// 撕裂系统配置那一支：盘 0 两槽都撕裂、没注入读故障，实现在取号之前以「这块盘一份自证过的系统配置都没有」拒（报成判据 N-配置 的拒因，C664）；
-/// 跳过撕裂的槽复算判据为假（c_见证 等于第一个文件那一版末条的计数器），C670 那一条不报，U6 照甲把撕裂判成按真：合法。
+/// C670 那一条跳过撕裂的槽复算判据为假（c_见证 等于第一个文件那一版末条的计数器），不报。U6 那四条里判据在崩溃镜像上复算：R2 之下撕裂的槽
+/// 不算判不出（没有槽设备报读错），盘 0 一份自证过的系统配置都没有 ⇒ 按真：合法。
 #[test]
 fn a_refusal_on_a_crash_image_without_read_faults_where_a_device_has_both_system_configuration_slots_torn_is_legal(
 ) {
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index 8fb824aa..99665dbc 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -2391,3 +2391,14 @@ C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：
 删除批第 16 批 N3：挂载续做的几批不照抄这条记录上一版的写入时间、取 0	crates/singlefs-core/src/transaction.rs	                write_time_seconds: record.write_time_seconds,	                write_time_seconds: 0,	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version	the_batches_of_a_mount_continuation_after_the_intent_copy_the_time_the_intent_wrote_on_disk
 删除批第 16 批 N3：挂载续做的几批不照抄这条记录上一版的写入时间、取一个固定时间	crates/singlefs-core/src/transaction.rs	                write_time_seconds: record.write_time_seconds,	                write_time_seconds: 1_788_000_000,	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version	the_batches_of_a_mount_continuation_after_the_intent_copy_the_time_the_intent_wrote_on_disk
 删除批第 16 批 N3：挂载从盘上读回 inode 记录时丢掉三个时间秒、续做的几批照抄到 0	crates/singlefs-core/src/records.rs	        let write_time_seconds = reader.get_u64();	        let write_time_seconds = { let _ = reader.get_u64(); 0 };	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version	the_batches_of_a_mount_continuation_after_the_intent_copy_the_time_the_intent_wrote_on_disk
+第21批 H2 回收门槛放松一个 txg（b19 判决 K2-1）：释放代比回收下界多 1 也回收，只暖机挂载 9 次之后第一个文件提前落 U、不落 U + 4	crates/singlefs-core/src/allocator.rs	            .filter(|record| record.is_released && record.generation <= floor)	            .filter(|record| record.is_released && record.generation.0 <= floor.0 + 1)	-p singlefs-harness --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement	the_first_file_after_nine_warm_up_only_mounts_lands_past_the_mkfs_instance_table_that_is_one_txg_short_of_reclaim
+第21批 I-9.27 写 0 的六种（b19 判决 K3-3）：种类 4（TREE_KIND_ACCOUNTING）不归 Zero 那一臂、落到 NotJudged，它的头 ID 改成 inode 树树 ID 的坏镜像不红	crates/singlefs-checker/src/walk.rs	            TREE_KIND_ALLOCATION\n            | TREE_KIND_ACCOUNTING\n	            TREE_KIND_ALLOCATION\n	-p singlefs-harness --test checker_known_bad_images	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
+第21批 I-9.27 写 0 的六种（b19 判决 K3-3）：种类 6（TREE_KIND_LIVELIST）不归 Zero 那一臂、落到 NotJudged，它的头 ID 改成 inode 树树 ID 的坏镜像不红	crates/singlefs-checker/src/walk.rs	            | TREE_KIND_ACCOUNTING\n            | TREE_KIND_LIVELIST\n	            | TREE_KIND_ACCOUNTING\n	-p singlefs-harness --test checker_known_bad_images	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
+第21批 I-9.27 写 0 的六种（b19 判决 K3-3）：种类 7（TREE_KIND_SPARSE_SIDE_TABLE）不归 Zero 那一臂、落到 NotJudged，它的头 ID 改成 inode 树树 ID 的坏镜像不红	crates/singlefs-checker/src/walk.rs	            | TREE_KIND_LIVELIST\n            | TREE_KIND_SPARSE_SIDE_TABLE\n	            | TREE_KIND_LIVELIST\n	-p singlefs-harness --test checker_known_bad_images	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
+第21批 I-9.27 写 0 的六种（b19 判决 K3-3）：种类 8（TREE_KIND_DEADLIST）不归 Zero 那一臂、落到 NotJudged，它的头 ID 改成 inode 树树 ID 的坏镜像不红	crates/singlefs-checker/src/walk.rs	            | TREE_KIND_SPARSE_SIDE_TABLE\n            | TREE_KIND_DEADLIST\n	            | TREE_KIND_SPARSE_SIDE_TABLE\n	-p singlefs-harness --test checker_known_bad_images	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
+第21批 I-9.27 写 0 的六种（b19 判决 K3-3）：种类 11（TREE_KIND_LINEAGE）不归 Zero 那一臂、落到 NotJudged，它的头 ID 改成 inode 树树 ID 的坏镜像不红	crates/singlefs-checker/src/walk.rs	            | TREE_KIND_DEADLIST\n            | TREE_KIND_LINEAGE\n	            | TREE_KIND_DEADLIST\n	-p singlefs-harness --test checker_known_bad_images	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
+第21批 I-9.27 写 0 的六种（b19 判决 K3-3）：种类 12（TREE_KIND_LOGGED_OPERATIONS）不归 Zero 那一臂、落到 NotJudged，它的头 ID 改成 inode 树树 ID 的坏镜像不红	crates/singlefs-checker/src/walk.rs	            | TREE_KIND_LINEAGE\n            | TREE_KIND_LOGGED_OPERATIONS => Self::Zero,	            | TREE_KIND_LINEAGE => Self::Zero,	-p singlefs-harness --test checker_known_bad_images	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
+F3（21-2）：二次崩溃的种子不混第一次崩溃那个段内子集（指纹换成空子集的，同一段号的各个第一次崩溃态用同一串随机数）	crates/singlefs-checker-tier/src/crash_injection.rs	            ^ fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment),	            ^ fingerprint_of_the_persisted_subset(&[]),	-p singlefs-checker-tier --lib -- crash_injection::tests::two_first_crash_states_differing_only_in_the_persisted_subset_draw_different_second_crashes	two_first_crash_states_differing_only_in_the_persisted_subset_draw_different_second_crashes
+崩溃注入第二截的发布臂（21-2 改法 a）：第一个文件已删完的带文件一版上那一支建 0 个 inode（发布照样做成、池级 checker 照样绿，只读挂载读回的 inode 号不多）	crates/singlefs-checker-tier/src/crash_injection.rs	                PoolVersion::WithFile(current) => publish_new_inodes(\n                    &mut writer,\n                    &mut allocator,\n                    current,\n                    1,	                PoolVersion::WithFile(current) => publish_new_inodes(\n                    &mut writer,\n                    &mut allocator,\n                    current,\n                    0,	-p singlefs-checker-tier --lib -- crash_injection::tests::the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published	the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published
+崩溃注入第二截的发布臂（21-2 改法 b）：判法只看第一个文件那条记录在不在树里（nlink = 0 正在删的那一版上照旧覆盖写，被拒成写一个正在删的 inode，Unexplained）	crates/singlefs-checker-tier/src/crash_injection.rs	        .is_some_and(|record| record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)	        .is_some()	-p singlefs-checker-tier --test crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short	the_publish_after_a_mount_whose_deletion_continuation_stays_short_creates_an_inode_and_is_refused_at_the_unit_area_wall
+C670（21-2）接线：第三截调用点把所选那一版交成第一次崩溃那一版（不是二次崩溃镜像上只读恢复落到的那一版；计数照样 (1, 1)，判官列出的那一支变）	crates/singlefs-checker-tier/src/crash_injection.rs	                crash_image_materialized: &second_crash_image_materialized,\n                version_landed_on_after_the_crash: report.effective_root,	                crash_image_materialized: &second_crash_image_materialized,\n                version_landed_on_after_the_crash: fields_read_after_the_first_crash.report.effective_root,	-p singlefs-checker-tier --lib -- crash_injection::tests::a_second_crash_landing_on_the_row_publish_whose_records_were_lost_names_the_selected_version_of_the_second_crash_image	a_second_crash_landing_on_the_row_publish_whose_records_were_lost_names_the_selected_version_of_the_second_crash_image
diff --git a/crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs b/crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs
new file mode 100644
index 00000000..5f46e2ea
--- /dev/null
+++ b/crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs
@@ -0,0 +1,158 @@
+//! checker 档模块：crash_injection
+//! 崩溃注入第二截那一次发布在「挂载续做推满仍放不下」那一版上走哪一臂（判决 `research/prompts/m3-formal-code-b20-r1-main-verification.md`
+//! 第二节 K2 H2，改法 b）：第一个文件那条记录还在树里、nlink = 0（正在删），覆盖写在这一版上被写入口拒成写一个正在删的 inode
+//! （D8（核心索引结构） 已定项 16 第 4 条），发布臂要建一个 inode；这一版本来就挤，建 inode 被单元区墙拒，是模型认得的拒。
+//! 盘面照 harness 档 `admission_raises_the_floor_before_refusing.rs` 的造法（不为环窗口补足的窄盘、准入关着），
+//! 形状取攻方探针 `research/prompts/m3-formal-code-b20-r1-opus-model/probes/b20_opus_k2_short_continuation.rs` 量到的第一个短续做盘面。
+
+#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
+mod common_admission;
+
+use common_admission::{
+    plain_devices_on,
+    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
+    OVERWRITE_BYTES,
+};
+use singlefs_checker_tier::crash_injection::{
+    mount_and_publish_once_on_the_crash_image, CrashPoint, WritableMountOutcome,
+};
+use singlefs_core::address::InodeNumber;
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::mount::{mount_writable_with_space_admission, DeletionQueueContinuationOutcome};
+use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
+use singlefs_core::transaction::{
+    publish_unlink_intent, publish_unlink_step, PoolVersion, PoolWriter, FIRST_INODE_NUMBER,
+};
+use singlefs_core::unit::data_unit_payload_capacity;
+use singlefs_core::unlink::{UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
+use singlefs_harness::history::{
+    HistoryDeviceWidth, HistoryExecution, HistorySeed, PerStepChecker, StepPosition,
+};
+use singlefs_harness::memory_pool::MemoryPool;
+use singlefs_harness::model::{ModelRefusalReason, ObservedRefusalReason};
+
+/// 窄盘的单元区槽数（攻方探针按 100..=320 步 2 扫过，这一档是第一个续做推满仍放不下的）。
+const UNIT_AREA_SLOTS: u64 = 128;
+
+/// 第一个文件之后经挂着的会话覆盖写几次。
+const OVERWRITES_BEFORE_THE_UNLINK: usize = 3;
+
+/// 顺序写之后第一个文件占几个数据单元。
+const DATA_UNITS_OF_THE_FIRST_FILE: usize = 14;
+
+/// unlink 意图之后发几批（每批释放一个数据单元）再崩。
+const DELETION_BATCHES_AFTER_THE_INTENT: usize = 1;
+
+/// 在不补足的窄盘上起池、第一个文件覆盖写几次、顺序写长到几个数据单元，再不经会话发 unlink 意图与之后几批（批大小 1）就崩：
+/// 交回崩在那一刻的整份盘面（照 `admission_raises_the_floor_before_refusing.rs` 的 `pool_crashed_between_deletion_batches`）。
+fn crash_image_between_the_deletion_batches() -> MemoryPool {
+    let mut pool =
+        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
+            HistoryDeviceWidth::UnitAreaOf384Slots,
+            UNIT_AREA_SLOTS,
+        );
+    for _ in 0..OVERWRITES_BEFORE_THE_UNLINK {
+        pool.overwrite(OVERWRITE_BYTES)
+            .expect("准入关着、单元区 128 槽：覆盖写落得下");
+    }
+    pool.sequential_write((DATA_UNITS_OF_THE_FIRST_FILE - 1) * data_unit_payload_capacity() + 100)
+        .expect("准入关着、单元区 128 槽：顺序写落得下");
+    let parameters = pool.parameters.clone();
+    let write_time_seconds = pool.write_time_seconds;
+    let session = pool.session.as_mut().expect("顺序写之后会话还开着");
+    let mut current = session
+        .current
+        .version_of_trees()
+        .expect("第一个文件之后的一版有树")
+        .clone();
+    let mut writer = PoolWriter::new(&parameters, pool.devices.as_mut_slice());
+    current = publish_unlink_intent(
+        &mut writer,
+        &mut session.allocator,
+        &current,
+        InodeNumber(FIRST_INODE_NUMBER),
+        write_time_seconds,
+        session.instance,
+    )
+    .expect("意图那次发布落得下");
+    for _ in 0..DELETION_BATCHES_AFTER_THE_INTENT {
+        current = publish_unlink_step(
+            &mut writer,
+            &mut session.allocator,
+            &current,
+            InodeNumber(FIRST_INODE_NUMBER),
+            UnlinkBatchSize::CappedForTests(
+                UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在只供测试的批大小范围里"),
+            ),
+            session.instance,
+        )
+        .expect("一批释放一个数据单元，落得下");
+    }
+    drop(writer);
+    pool.session = None;
+    pool.image()
+}
+
+/// 判法里 nlink 那一半（`crash_injection.rs` 的 `first_file_is_a_live_record`）：第一个文件那条记录在树里而 nlink = 0 时，
+/// 发布臂建一个 inode，在这份挤的盘面上被单元区墙拒（`Explained(UnitAreaWall)`），不是覆盖写被写入口拒成写一个正在删的 inode（`Unexplained`）。
+/// 判别力：判法只看 `inode_record` 是不是 Some（`crates/mutations.tsv` 一行）时走覆盖写，拒因是 `Unexplained`，红在最后一条断言。
+// crash-case-check:not-a-crash-case 靶向用例只在写死的一份崩溃盘面上挂一次、发一次，不枚举崩溃态
+#[test]
+fn the_publish_after_a_mount_whose_deletion_continuation_stays_short_creates_an_inode_and_is_refused_at_the_unit_area_wall(
+) {
+    let crash_image = crash_image_between_the_deletion_batches();
+    let execution = HistoryExecution {
+        per_step_checker: PerStepChecker::Skipped,
+        device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
+        space_admission: SpaceAdmission::JudgedByTheFormula,
+    };
+    let parameters = execution.device_width.parameters();
+    {
+        let mut devices = plain_devices_on(&crash_image);
+        let mounted = mount_writable_with_space_admission(
+            &parameters,
+            &mut devices,
+            execution.space_admission,
+        )
+        .expect("前提：挂载续做推满仍放不下时挂载照样做成可写");
+        assert!(
+            matches!(
+                mounted.output.deletion_queue_continuation_outcome,
+                DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. }
+            ),
+            "前提：挂载续做推满抬 F 仍放不下、队列留着：{:?}",
+            mounted.output.deletion_queue_continuation_outcome
+        );
+        let PoolVersion::WithFile(current) = &mounted.current else {
+            panic!("前提：挂载交回的现行那一版是带文件的一版");
+        };
+        assert_eq!(
+            current.inode_record.map(|record| record.nlink),
+            Some(NLINK_OF_AN_INODE_BEING_DELETED),
+            "前提：第一个文件那条记录还在树里、正在删（nlink = 0）"
+        );
+    }
+    let mount_run = mount_and_publish_once_on_the_crash_image(
+        &crash_image,
+        &parameters,
+        execution,
+        HistorySeed(0),
+        &CrashPoint {
+            segment_index: 0,
+            persisted_within_the_segment: Vec::new(),
+            step: StepPosition::StartingPoint,
+            operation_kind: None,
+        },
+    );
+    let WritableMountOutcome::PublishRefused { member, reason } = &mount_run.outcome else {
+        panic!(
+            "这份盘面上挂载做成、之后那一次发布被拒：{:?}",
+            mount_run.outcome
+        );
+    };
+    assert_eq!(
+        *reason,
+        ObservedRefusalReason::Explained(ModelRefusalReason::UnitAreaWall),
+        "正在删的第一个文件那一版上，发布臂建一个 inode、被单元区墙拒（模型认得的拒），不是覆盖写被拒成写一个正在删的 inode：{member}"
+    );
+}
````
