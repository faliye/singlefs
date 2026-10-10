# 第 23 批代码轮改动全文（2026-10-04）

合入前备份对合入后主仓（23-1 一份补丁）。

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index 701f908d..1e7760e0 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -2813,9 +2813,26 @@ fn fingerprint_of_the_persisted_subset(persisted_within_the_segment: &[bool]) ->
     )
 }
 
+/// 一个第一次崩溃态上按种子摆二次崩溃（[`draw_second_crash_points`]）用的随机源种子。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+struct SecondCrashPointSeed(u64);
+
+/// 历史种子、盐、第一次崩溃的段号与它段内子集的指纹（[`fingerprint_of_the_persisted_subset`]）四样异或。
+fn seed_of_the_second_crash_points(
+    seed: HistorySeed,
+    crash_point: &CrashPoint,
+) -> SecondCrashPointSeed {
+    SecondCrashPointSeed(
+        seed.0
+            ^ SECOND_CRASH_POINT_SEED_SALT
+            ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64")
+            ^ fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment),
+    )
+}
+
 /// 挂载的每一段各摆一个二次崩溃：在那一段的写里按种子挑一个 w，崩在 w 上——w 所在那一段里 w 之前的写都落了、w 没落、
 /// w 之后的写各按种子抽；更早的段整段持久，更晚的段一个都没持久（与第一次崩溃同一个枚举域）。挂载没写出某一段的写就不摆那一段。
-/// 种子是历史种子、盐、第一次崩溃的段号与它段内子集的指纹（[`fingerprint_of_the_persisted_subset`]）四样异或：同一段里不同子集的
+/// 种子是历史种子、盐、第一次崩溃的段号与它段内子集的指纹四样异或（[`seed_of_the_second_crash_points`]）：同一段里不同子集的
 /// 第一次崩溃态各抽各的（攻方改法 F3，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第五节第 1 行；
 /// 只混段号时同一段里的各个第一次崩溃态用同一串随机数，挂载写表相同就摆到同一组二次崩溃）。
 fn draw_second_crash_points(
@@ -2824,12 +2841,8 @@ fn draw_second_crash_points(
     segments: &[Vec<usize>],
     phase_of_each_write: &[WritableMountPhase],
 ) -> Vec<SecondCrashPoint> {
-    let mut source = SeededRandomSource::from_seed(
-        seed.0
-            ^ SECOND_CRASH_POINT_SEED_SALT
-            ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64")
-            ^ fingerprint_of_the_persisted_subset(&crash_point.persisted_within_the_segment),
-    );
+    let mut source =
+        SeededRandomSource::from_seed(seed_of_the_second_crash_points(seed, crash_point).0);
     let mut chosen: BTreeSet<SecondCrashPoint> = BTreeSet::new();
     for phase in WritableMountPhase::ALL {
         let writes_of_the_phase: Vec<(usize, usize)> = segments
@@ -4382,6 +4395,41 @@ mod tests {
         );
     }
 
+    /// 二次崩溃的种子把段内子集整个折进去（攻方改法 F3-c，判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节
+    /// 「K2 F3 内联用例判别力」与第三节第 2 条）：同一历史种子（0）、同一段号（16）、长 12 的段，段内扣下集合非空的全部 4095 个第一次崩溃态
+    /// （12 个写的 4096 个持久子集去掉全持久的那一个：全持久不是崩在这一段里的态）各算一次种子，两两不同。上一条只比两个子集，
+    /// 两个子集的第一个写一个落了一个没落，只折第一个写的弱指纹也分得开。攻方在副本上量过：入库指纹 4095 个种子各不相同，
+    /// 只折第一个写落没落时剩 2 个，只取第一个被扣下的写的下标时剩 12 个（`research/prompts/m3-formal-code-b21-r1-opus-model/out/f3.txt`）。
+    /// 判别力：指纹只折段内第一个写落没落（`crates/mutations.tsv` 一行）时种子只剩 2 个，这一条红。
+    #[test]
+    fn all_first_crash_states_of_a_segment_of_twelve_writes_seed_their_second_crashes_differently()
+    {
+        const WRITES_OF_THE_SEGMENT: usize = 12;
+        let all_persisted_mask = (1_u32 << WRITES_OF_THE_SEGMENT) - 1;
+        let seeds: BTreeSet<SecondCrashPointSeed> = (0..all_persisted_mask)
+            .map(|persisted_mask| {
+                seed_of_the_second_crash_points(
+                    HistorySeed(0),
+                    &CrashPoint {
+                        segment_index: 16,
+                        persisted_within_the_segment: (0..WRITES_OF_THE_SEGMENT)
+                            .map(|write_within_the_segment| {
+                                (persisted_mask >> write_within_the_segment) & 1 == 1
+                            })
+                            .collect(),
+                        step: StepPosition::Operation(0),
+                        operation_kind: Some(HistoryOperationKind::PublishOverwrite),
+                    },
+                )
+            })
+            .collect();
+        assert_eq!(
+            seeds.len(),
+            4095,
+            "长 12 的段上 4095 个第一次崩溃态（扣下集合非空）算出的二次崩溃种子两两不同"
+        );
+    }
+
     /// 实例表清扫两段（D18（块里携带什么信息） 已定项 11「回收」一条）：挂载那一段的写里，第「清扫之前发了几次」个根槽写之后起是清扫抹头
     /// （FUA 写零的单元头、每块盘一道屏障），之后第一个不带 FUA 的单元写起是清扫发布（单元、屏障、记录、根槽、轮换），两段摆二次崩溃时各摆到一个。
     /// 造的写表：取号；写行那次（单元、记录、根槽、轮换）；一次暖机（单元、根槽）；抹两个头、两道屏障；清扫发布——清扫之前发了 2 次。
@@ -4714,14 +4762,19 @@ mod tests {
     /// unlink 之后再建过 inode 的一版归带文件的一版、第一个文件那条记录不在（删除与并发那条线读码诊断；攻方随机历史种子 32768、245760 两段那一形，
     /// 判决 `research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节「越出攻击面」那一行）：第一个文件之后 unlink 它、再建一个 inode，
     /// 崩在建 inode 那次发布里——根槽落了、紧跟着的系统配置轮换没落，只读恢复落在建 inode 那一版（这段历史最后提交的一版）。
-    /// 崩溃后镜像上的可写挂载做成，之后那一次发布建一个 inode、做成，第二截没有新发现；发布之后那份盘只读挂载读回的 inode 号比崩溃那一版
-    /// 多恰好一个（改法 a，`research/prompts/m3-formal-code-b20-r1-main-verification.md` 第二节 K2 H1）。判别力：带文件那一臂改回「一律覆盖写」
-    /// （`crates/mutations.tsv` 一行），覆盖写被拒成写一个不在树里的 inode 1，计一次发布被拒、记一条第二截的新发现，这一条红；
-    /// 那一臂建 0 个 inode（`crates/mutations.tsv` 一行）时发布照样做成、池级 checker 照样绿，红在 inode 号多一个那一条。
+    /// 崩溃后镜像上的可写挂载做成，之后那一次发布建一个 inode、做成，第二截没有新发现；发布之后那份盘只读挂载读回的 inode 记录是崩溃那一版的
+    /// 每一条原样加新建的那一条，新建那一条整条等于第二截那一支的调法该写出的（改法 a，`research/prompts/m3-formal-code-b20-r1-main-verification.md`
+    /// 第二节 K2 H1；比整条是改法 A1，`research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节「K2 改法 a 判别力」与第三节第 2 条）。
+    /// 判别力：带文件那一臂改回「一律覆盖写」（`crates/mutations.tsv` 一行），覆盖写被拒成写一个不在树里的 inode 1，计一次发布被拒、
+    /// 记一条第二截的新发现，这一条红；那一臂建 0 个 inode、新建那一条 nlink = 0、出生代取上一版、size 4096、写入时间 0
+    /// （`crates/mutations.tsv` 各一行；攻方量过后四种池级 checker 照样绿，`research/prompts/m3-formal-code-b21-r1-opus-model/out/k2a.txt`），
+    /// 红在比整条记录那一条。
     // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个崩溃态（ExactlyOneCrashState），不枚举
     #[test]
     fn the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published(
     ) {
+        use singlefs_core::records::{InodeRecord, NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES};
+        use singlefs_core::transaction::FIRST_INODE_NUMBER;
         use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};
         let execution = HistoryExecution {
             per_step_checker: PerStepChecker::Skipped,
@@ -4841,22 +4894,40 @@ mod tests {
             "前提：同一份盘面上再挂一次加一次发布，照样做成：{:?}",
             mount_run.outcome
         );
-        let inode_numbers_read_back = |pool: &MemoryPool| -> BTreeSet<u64> {
+        let read_only_mount = |pool: &MemoryPool| {
             singlefs_core::mounted_read::mount_read_only(pool)
                 .expect("崩溃那一版与发布之后那一版都是整次落盘的版本，只读挂载做得成")
+        };
+        let crashed_version = read_only_mount(&crash_image);
+        let version_after_the_publish = read_only_mount(&mount_run.pool_after);
+        // 第一个文件取 1 号（已删完），unlink 之后历史里建的那一个取 2 号：崩溃那一版树里只剩 2 号，inode 号水位 3。
+        assert_eq!(
+            crashed_version
                 .mounted
                 .inode_records()
                 .iter()
                 .map(|record| record.inode)
-                .collect()
-        };
-        let inode_numbers_of_the_crashed_version = inode_numbers_read_back(&crash_image);
-        let inode_numbers_after_the_publish = inode_numbers_read_back(&mount_run.pool_after);
-        assert!(
-            inode_numbers_of_the_crashed_version.is_subset(&inode_numbers_after_the_publish)
-                && inode_numbers_after_the_publish.len()
-                    == inode_numbers_of_the_crashed_version.len() + 1,
-            "发布之后只读挂载读回的 inode 号比崩溃那一版多恰好一个：崩溃那一版 {inode_numbers_of_the_crashed_version:?}，发布之后 {inode_numbers_after_the_publish:?}"
+                .collect::<Vec<_>>(),
+            [FIRST_INODE_NUMBER + 1],
+            "前提：崩溃那一版树里只剩历史里建的那一个 inode"
+        );
+        // 照第二截那一支的调法 `publish_new_inodes(…, 1, FIXED_WRITE_TIME_SECONDS, 挂载的实例)` 该写出的那一条：号从水位 3 起发，
+        // 对象出生代与改动计数都是这次发布的 checkpoint_txg（发布之后那一版的根），活着（nlink = 1），长度 0（blocks 由 size 现算，跟着是 0）。
+        let txg_of_the_publish = version_after_the_publish.effective_root.checkpoint_txg;
+        let mut expected_records_after_the_publish =
+            crashed_version.mounted.inode_records().to_vec();
+        expected_records_after_the_publish.push(InodeRecord {
+            inode: FIRST_INODE_NUMBER + 2,
+            object_birth: txg_of_the_publish,
+            nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
+            size: 0,
+            change_count: txg_of_the_publish.0,
+            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
+        });
+        assert_eq!(
+            version_after_the_publish.mounted.inode_records(),
+            expected_records_after_the_publish.as_slice(),
+            "发布之后只读挂载读回的 inode 记录 = 崩溃那一版的每一条原样 + 新建的那一条（号、出生代、nlink、size、改动计数、写入时间逐个对上）"
         );
     }
 
@@ -6218,6 +6289,8 @@ mod tests {
     /// 计数照样 (1, 1)，判官列出的那一支换成「c_见证 大于所选那一版末条记录的计数器」，红在那一支的两条断言上。计数分不开：在 harness 档草稿探针里
     /// 量过，三段短历史 131 个第一次崩溃态上挂载写表每个前缀与每个前缀扣一个写（177198 个二次崩溃态），以及这一个第一次崩溃态上每个前缀扣至多两个写，
     /// 两种所选那一版的 (判过, 不合法) 没有一处不同；判官结论不同的只有扣下写行（或第一次暖机）那次记录、根槽落了的这一类，都只差这一支。
+    /// 那几处没量到的越屏障靶向态里计数分得开（攻方量过，判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节 K3），
+    /// 钉在 [`a_second_crash_losing_the_records_of_the_row_publish_after_a_first_crash_losing_the_records_of_the_first_file_counts_one_illegal_witness_refusal`]。
     // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
     #[test]
     fn a_second_crash_landing_on_the_row_publish_whose_records_were_lost_names_the_selected_version_of_the_second_crash_image(
@@ -6395,4 +6468,201 @@ mod tests {
             "所选那一版不是第一次崩溃落的起点那一版（那一版的末条记录读得出），不许列 {branch_of_the_first_crash_version}：{remount_finding}"
         );
     }
+
+    /// 第三截「所选那一版」的接线在计数层分得开的靶向态（攻方改法 K3-a，判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md`
+    /// 第二节 K3 与第三节第 2 条；攻方探针 `research/prompts/m3-formal-code-b21-r1-opus-model/probe/opus_probe.rs` 的 `k3_sweep`，
+    /// 产物 `…/out/k3-sweep.txt` 那一行 `first(last=80,withheld=[0, 1, 4]) second(last=19,withheld=[0, 1, 3])`）。与上两条同一段历史。
+    /// 第一次崩溃从第一个文件那次发布的第一份记录起，摆到覆盖写那次的根槽为止：第一个文件那次的两份记录与它根槽之后第二槽系统配置轮换扣下，
+    /// 别的（第一个文件那次的根槽、第一槽轮换，覆盖写那次的单元、记录与根槽）都落了——越过记录与根槽之间、根槽与轮换之间的屏障。
+    /// 第二截那次可写挂载的写行那次发布上摆一个二次崩溃：写行那次的两份记录与根槽之后第一槽轮换扣下，根槽落了，第二槽轮换没摆到。
+    /// 第三截再挂以判据 N-配置 拒；所选那一版取二次崩溃镜像上只读恢复落到的那一版（写行那一版），它的末条记录与计数器等于 c_见证 的记录都读不出，
+    /// 判据为真：判过一次、不合法一次。判别力：调用点把所选那一版交成第一次崩溃那一版（`crates/mutations.tsv` 一行，原文与上一条那一行相同）时
+    /// 判据复算为假、计数变 (1, 0)，红在最后一条断言（攻方在副本上量过，`research/prompts/m3-formal-code-b21-r1-opus-model/out/k3-sweep.txt`）。
+    /// 上一条那一态在计数层分不开这两种接线，只在判官列出的那一支上分得开；这一态在计数层就分得开。
+    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
+    #[test]
+    fn a_second_crash_losing_the_records_of_the_row_publish_after_a_first_crash_losing_the_records_of_the_first_file_counts_one_illegal_witness_refusal(
+    ) {
+        let (history, execution) = history_of_the_first_file_and_one_overwrite();
+        let (writes, segments, first_write_of_the_overwrite) =
+            writes_segments_and_first_write_of_the_overwrite(&history, execution);
+        let root_of_the_overwrite = (first_write_of_the_overwrite..writes.len())
+            .find(|write| writes[*write].kind == StepKind::RootRecordFua)
+            .expect("前提：覆盖写那次发布写了根槽");
+        let root_of_the_first_file = (0..first_write_of_the_overwrite)
+            .rev()
+            .find(|write| writes[*write].kind == StepKind::RootRecordFua)
+            .expect("前提：第一个文件那次发布写了根槽");
+        let root_before_the_first_file = (0..root_of_the_first_file)
+            .rev()
+            .find(|write| writes[*write].kind == StepKind::RootRecordFua)
+            .expect("前提：第一个文件之前那次发布写了根槽");
+        let records_of_the_first_file: Vec<usize> = (root_before_the_first_file + 1
+            ..root_of_the_first_file)
+            .filter(|write| writes[*write].kind == StepKind::JournalRecord)
+            .collect();
+        let rotation_after_the_first_file: Vec<usize> = (root_of_the_first_file + 1
+            ..first_write_of_the_overwrite)
+            .filter(|write| writes[*write].kind == StepKind::SystemConfigurationSlot)
+            .collect();
+        assert_eq!(
+            (
+                records_of_the_first_file.len(),
+                rotation_after_the_first_file.len()
+            ),
+            (2, 2),
+            "前提：第一个文件那次发布在根槽之前每块盘写一份记录，根槽之后每块盘轮换一槽系统配置"
+        );
+        let first_crash_segment_index =
+            segment_containing_the_write(&segments, records_of_the_first_file[0]);
+        let first_write_of_the_first_crash_segment = segments[first_crash_segment_index][0];
+        let withheld_in_the_first_crash: Vec<usize> = records_of_the_first_file
+            .iter()
+            .chain(std::iter::once(&rotation_after_the_first_file[1]))
+            .map(|write| write - first_write_of_the_first_crash_segment)
+            .collect();
+        let writes_of_the_first_crash =
+            root_of_the_overwrite + 1 - first_write_of_the_first_crash_segment;
+        let mut first_crash_image = MemoryPool::with_devices(
+            &[DeviceIdentity(0), DeviceIdentity(1)],
+            execution.device_width.device_bytes(),
+        );
+        first_crash_image.apply_writes(&writes[..first_write_of_the_first_crash_segment]);
+        let persisted_writes_of_the_first_crash: Vec<RetainedWrite> = (0
+            ..writes_of_the_first_crash)
+            .filter(|within_the_state| !withheld_in_the_first_crash.contains(within_the_state))
+            .map(|within_the_state| {
+                writes[first_write_of_the_first_crash_segment + within_the_state].clone()
+            })
+            .collect();
+        first_crash_image.apply_writes(&persisted_writes_of_the_first_crash);
+        // 第二截那次挂载写出的写表：与注入里那一次同一份盘面、同一份参数；挂载之后那一次发布写的内容随崩溃点变，不在挂载写表里。
+        let mount_run = mount_and_publish_once_on_the_crash_image(
+            &first_crash_image,
+            &execution.device_width.parameters(),
+            execution,
+            history.seed,
+            &CrashPoint {
+                segment_index: first_crash_segment_index,
+                persisted_within_the_segment: (0..writes_of_the_first_crash)
+                    .map(|within_the_state| {
+                        !withheld_in_the_first_crash.contains(&within_the_state)
+                    })
+                    .collect(),
+                step: StepPosition::Operation(0),
+                operation_kind: Some(HistoryOperationKind::PublishOverwrite),
+            },
+        );
+        assert!(
+            matches!(mount_run.outcome, WritableMountOutcome::Published),
+            "前提：第一次崩溃镜像上的可写挂载与之后那一次发布都做成：{:?}",
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
+        let second_crash_segment_index =
+            segment_containing_the_write(&mount_segments, records_of_the_row_publish[0]);
+        let first_write_of_the_second_crash_segment = mount_segments[second_crash_segment_index][0];
+        let withheld_in_the_second_crash: Vec<usize> = records_of_the_row_publish
+            .iter()
+            .chain(std::iter::once(&rotation_after_the_root[0]))
+            .map(|write| write - first_write_of_the_second_crash_segment)
+            .collect();
+        // 前提：两次崩溃之后只读恢复落的版本不同（同一份盘面在这里照摆法叠一遍）。
+        let mut second_crash_image = first_crash_image.clone();
+        second_crash_image.apply_writes(&mount_writes[..first_write_of_the_second_crash_segment]);
+        let persisted_writes_of_the_second_crash: Vec<RetainedWrite> =
+            (first_write_of_the_second_crash_segment..=rotation_after_the_root[0])
+                .filter(|write| {
+                    !withheld_in_the_second_crash
+                        .contains(&(write - first_write_of_the_second_crash_segment))
+                })
+                .map(|write| mount_writes[write].clone())
+                .collect();
+        second_crash_image.apply_writes(&persisted_writes_of_the_second_crash);
+        assert_ne!(
+            recover(&first_crash_image, JournalPolicy::Consult).effective_root,
+            recover(&second_crash_image, JournalPolicy::Consult).effective_root,
+            "前提：第一次崩溃落在覆盖写那一版、二次崩溃落在写行那一版，两次只读恢复落的版本不同"
+        );
+        let injection = inject_crashes_into_history_drawing_second_crashes(
+            &history,
+            execution,
+            CrashPointDraw::ExactlyOneCrashState {
+                segment_index: first_crash_segment_index,
+                writes_from_the_segment_on: writes_of_the_first_crash,
+                withheld_among_them: withheld_in_the_first_crash.leak(),
+            },
+            SecondCrashPointDraw::ExactlyOneSecondCrashState {
+                segment_index: second_crash_segment_index,
+                writes_from_the_segment_on: rotation_after_the_root[0] + 1
+                    - first_write_of_the_second_crash_segment,
+                withheld_among_them: withheld_in_the_second_crash.leak(),
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
+            "二次崩溃镜像上再挂以判据 N-配置 拒，对着二次崩溃镜像上只读恢复落到的那一版复算判据为真：判过一次、不合法一次：{}",
+            tally.render()
+        );
+    }
 }
diff --git a/crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs b/crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs
index 5f46e2ea..93ca1eed 100644
--- a/crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs
+++ b/crates/singlefs-checker-tier/tests/crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short.rs
@@ -3,7 +3,8 @@
 //! 第二节 K2 H2，改法 b）：第一个文件那条记录还在树里、nlink = 0（正在删），覆盖写在这一版上被写入口拒成写一个正在删的 inode
 //! （D8（核心索引结构） 已定项 16 第 4 条），发布臂要建一个 inode；这一版本来就挤，建 inode 被单元区墙拒，是模型认得的拒。
 //! 盘面照 harness 档 `admission_raises_the_floor_before_refusing.rs` 的造法（不为环窗口补足的窄盘、准入关着），
-//! 形状取攻方探针 `research/prompts/m3-formal-code-b20-r1-opus-model/probes/b20_opus_k2_short_continuation.rs` 量到的第一个短续做盘面。
+//! 覆盖写次数、单元数与批数取攻方探针 `research/prompts/m3-formal-code-b20-r1-opus-model/probes/b20_opus_k2_short_continuation.rs`
+//! 量到的第一个短续做盘面，单元区槽数换成分得开那一支建几个的那一档（`UNIT_AREA_SLOTS`）。
 
 #[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
 mod common_admission;
@@ -31,8 +32,12 @@ use singlefs_harness::history::{
 use singlefs_harness::memory_pool::MemoryPool;
 use singlefs_harness::model::{ModelRefusalReason, ObservedRefusalReason};
 
-/// 窄盘的单元区槽数（攻方探针按 100..=320 步 2 扫过，这一档是第一个续做推满仍放不下的）。
-const UNIT_AREA_SLOTS: u64 = 128;
+/// 窄盘的单元区槽数（改法 B1，判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节「K2 改法 b 判别力」与第三节第 2 条）：
+/// 续做照样推满仍放不下、第一个文件那条记录 nlink = 0，而这一档上建 0 个 inode 的空发布落得下、建 1 个落不下——
+/// 攻方量过 136 槽时正确判法 `Explained(UnitAreaWall)`、只看 Some 是 `Unexplained`、那一支建 0 个是 `Published`
+/// （`research/prompts/m3-formal-code-b21-r1-opus-model/out/k2b.txt` 第 2、4、6 行）。128 槽（攻方探针按 100..=320 步 2 扫出的第一个短续做盘面）
+/// 上建 0 个也被空间准入拒、照样记 `UnitAreaWall`，分不开那一支建几个。
+const UNIT_AREA_SLOTS: u64 = 136;
 
 /// 第一个文件之后经挂着的会话覆盖写几次。
 const OVERWRITES_BEFORE_THE_UNLINK: usize = 3;
@@ -53,10 +58,10 @@ fn crash_image_between_the_deletion_batches() -> MemoryPool {
         );
     for _ in 0..OVERWRITES_BEFORE_THE_UNLINK {
         pool.overwrite(OVERWRITE_BYTES)
-            .expect("准入关着、单元区 128 槽：覆盖写落得下");
+            .expect("准入关着、单元区 136 槽：覆盖写落得下");
     }
     pool.sequential_write((DATA_UNITS_OF_THE_FIRST_FILE - 1) * data_unit_payload_capacity() + 100)
-        .expect("准入关着、单元区 128 槽：顺序写落得下");
+        .expect("准入关着、单元区 136 槽：顺序写落得下");
     let parameters = pool.parameters.clone();
     let write_time_seconds = pool.write_time_seconds;
     let session = pool.session.as_mut().expect("顺序写之后会话还开着");
@@ -95,7 +100,8 @@ fn crash_image_between_the_deletion_batches() -> MemoryPool {
 
 /// 判法里 nlink 那一半（`crash_injection.rs` 的 `first_file_is_a_live_record`）：第一个文件那条记录在树里而 nlink = 0 时，
 /// 发布臂建一个 inode，在这份挤的盘面上被单元区墙拒（`Explained(UnitAreaWall)`），不是覆盖写被写入口拒成写一个正在删的 inode（`Unexplained`）。
-/// 判别力：判法只看 `inode_record` 是不是 Some（`crates/mutations.tsv` 一行）时走覆盖写，拒因是 `Unexplained`，红在最后一条断言。
+/// 判别力：判法只看 `inode_record` 是不是 Some（`crates/mutations.tsv` 一行）时走覆盖写，拒因是 `Unexplained`；那一支建 0 个 inode
+/// （`crates/mutations.tsv` 一行）时空发布落得下、结局是 `Published`，不是 `PublishRefused`，红在解构结局那一处。
 // crash-case-check:not-a-crash-case 靶向用例只在写死的一份崩溃盘面上挂一次、发一次，不枚举崩溃态
 #[test]
 fn the_publish_after_a_mount_whose_deletion_continuation_stays_short_creates_an_inode_and_is_refused_at_the_unit_area_wall(
diff --git a/crates/singlefs-core/src/root_record.rs b/crates/singlefs-core/src/root_record.rs
index 67526d73..d1b23a31 100644
--- a/crates/singlefs-core/src/root_record.rs
+++ b/crates/singlefs-core/src/root_record.rs
@@ -23,7 +23,7 @@ pub const ROOT_RECORD_FLAG_UNMOUNT_MARKER: u32 = 1 << 0;
 pub enum UnmountMarker {
     /// flags 位 0 = 1：正常卸载那一串写的根（`mount::unmount`）。
     WrittenByTheUnmountSequence,
-    /// flags 位 0 = 0：别的每一条根——mkfs 的第 0 代、暖机、写行、发布、准入抬 F、回退，以及由记录施加出来的那一版。
+    /// flags 位 0 = 0：别的每一条根——mkfs 的第 0 代、暖机、写行、发布、准入抬 F、回退；由记录施加出来的那一版取那条记录新根段里的记号（D23（journal 的角色与格式） 已定项 15，`recovery` 施加处）。
     NotWrittenByTheUnmountSequence,
 }
 
diff --git a/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs b/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
index 983cd97d..a39ebba6 100644
--- a/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
+++ b/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
@@ -6,7 +6,7 @@
 //!   真被评估过且成立；第一个文件版本离开根环之后只剩覆盖写那几版（诞生 txg 不等于这一版的 txg），一版都不判。
 //! - I-5.5 的判据按这一版的分配记录现算（用户 2026-10-04 定，判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md`
 //!   第二节 K2、第五节第 1 行）：mkfs 的实例表（U、U + 1）被 COW 走、过了回收门槛之后，实现照 D3（空间分配） 已定项 8 第 1 条
-//!   把第一个文件的数据单元放在 U——只做过 mkfs 的池只暖机挂载 10 次之后写第一个文件、同一次挂载里推 22 次零单元发布之后写第一个文件，
+//!   把第一个文件的数据单元放在 U——只做过 mkfs 的池只暖机挂载 S + 2 次（S 是根环每区槽数，今天 10 次）之后写第一个文件、同一次挂载里推 22 次零单元发布之后写第一个文件，
 //!   两段合法历史上 I-5.5 成立（攻方原型 `research/prompts/m3-formal-code-b17-r1-opus-model/opus_b17_first_data_unit_attack.rs`
 //!   的形 A、形 B，写死 U + 4 的判据在这两段上误红）；一块盘上有两个数据单元的那一版，取的是每块盘上最小的那一个。
 //!
@@ -54,14 +54,19 @@ const ROOT_RING_SLOT_COUNT: u64 = ROOT_RING_REGIONS * ROOT_RING_SLOTS_PER_REGION
 const FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START: u64 = 4;
 /// 非默认环长的那一种：128 MiB 环，单元区起始槽号现算（攻方在这一格上量过 9216，`research/prompts/c673-clauses-r1-opus-model/out/probe.out`）。
 const JOURNAL_RING_BYTES_OF_128_MEBIBYTES: u64 = 128 << 20;
-/// 只做过 mkfs 的池只暖机挂载这么多次，第这一次挂载上写的第一个文件版本落到 U：实例 2 写行时把 mkfs 的实例表（U、U + 1）COW 走、
-/// 释放代 3；第 10 次挂载上的第一个文件版本 txg 29，写它之前环里最旧有效根是 29 − 24 = 5 ≥ 3，U、U + 1 已回收（第 9 次是 26 − 24 = 2 < 3，还没有）。
-/// 攻方在这一格上量过翻转点 9 → 10（`research/prompts/m3-formal-code-b17-r1-opus-model/out/runA1.log` 第 15、17 行）。
-const WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED: usize = 10;
-/// 比上一个少一次：第 9 次挂载上的第一个文件版本 txg 26，写它之前环里最旧有效根 26 − 24 = 2 < 释放代 3，mkfs 的实例表那一对还没过回收门槛。
+/// mkfs 的实例表那一对（U、U + 1）的释放代：实例 2 写行那次发布把它 COW 走。
+const RELEASE_TXG_OF_THE_MKFS_INSTANCE_TABLE: u64 = 3;
+/// 只做过 mkfs 的池只暖机挂载这么多次（根环每区槽数 S 加 2），第这一次挂载上写的第一个文件版本落到 U：每次只暖机的挂载发 3 版，
+/// 第 k 次挂载上的第一个文件版本 txg 3k − 1，写它之前环里最旧有效根比它早一圈（3S 槽），k = S + 2 时是 5 ≥ 释放代 3，U、U + 1 已回收
+/// （k = S + 1 时是 2 < 3，还没有）。S 取 mkfs 写进系统配置的那一个（`ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`，今天 8：第 10 次挂载、txg 29）。
+/// 攻方在 S = 8 上量过翻转点 9 → 10（`research/prompts/m3-formal-code-b17-r1-opus-model/out/runA1.log` 第 15、17 行）；正推在 S = 5、6、7、8、12、16
+/// 上量过正确实现翻转在 S + 2、放宽一个 txg 的翻转在 S + 1（判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节 K1-a）。
+const WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED: u64 =
+    ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM + 2;
+/// 比上一个少一次（S + 1）：这一次挂载上的第一个文件版本写之前环里最旧有效根 2，比释放代 3 差一个 txg，mkfs 的实例表那一对还没过回收门槛。
 /// 回收门槛放松一个 txg（释放代 ≤ 下界 + 1）的坏实现在这一格提前回收、落 U（判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md`
 /// 第二节 K2-1，攻方量过 `research/prompts/m3-formal-code-b19-r1-opus-model/out/h2-slack0.txt` 第 9 行与 `h2-slack1.txt` 第 9 行）。
-const WARM_UP_ONLY_MOUNTS_ONE_SHORT_OF_RECLAIMING_THE_MKFS_INSTANCE_TABLE: usize =
+const WARM_UP_ONLY_MOUNTS_ONE_SHORT_OF_RECLAIMING_THE_MKFS_INSTANCE_TABLE: u64 =
     WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED - 1;
 /// 实例 2 里接连推这么多次零单元发布之后写的第一个文件版本落到 U（攻方量过翻转点 21 → 22，
 /// `research/prompts/m3-formal-code-b17-r1-opus-model/out/runB1.log` 第 28、29 行）。
@@ -468,7 +473,7 @@ fn data_unit_slots_on_each_device(first_file: &TransactionOutput) -> Vec<(u32, u
 /// 只做过 mkfs 的池接连可写挂载 `mounts` 次（每次挂上只写行、暖机，不写文件），交回最后那一次挂载。
 fn mount_writable_repeatedly(
     devices: &mut Vec<(DeviceIdentity, MemoryDevice)>,
-    mounts: usize,
+    mounts: u64,
 ) -> Mounted {
     let mut last_mount = mount_writable(&parameters(), devices).expect("第 1 次可写挂载");
     for _ in 1..mounts {
@@ -501,21 +506,19 @@ fn publish_the_first_file_on_the_mount(
     .expect("树表 0 条的一版上发第一个文件版本")
 }
 
-/// 攻方原型形 A（k = 10）：只做过 mkfs 的池只暖机挂载 10 次，第 10 次挂载上写第一个文件。实例 2 写行时 mkfs 的实例表（U、U + 1）
+/// 攻方原型形 A（k = S + 2，今天 10）：只做过 mkfs 的池只暖机挂载 S + 2 次，第 S + 2 次挂载上写第一个文件。实例 2 写行时 mkfs 的实例表（U、U + 1）
 /// 被 COW 走，到这一版之前已过回收门槛：实现照 D3（空间分配） 已定项 8 第 1 条（起点最小、32768 对齐、两槽都空的那一对）把数据单元放在 U。
 /// 这一版仍从 mkfs 水位第一次建树（认法 ①②），I-5.5 按这一版的分配记录现算：U、U + 1 是这一版自己分配的、算空，该在 U，成立；
 /// 写死 U + 4 的判据在这里误红。池级 checker 一条都不红。
 #[test]
 fn the_first_file_after_ten_warm_up_only_mounts_lands_on_the_reclaimed_unit_area_start_and_holds_the_first_data_unit_placement(
 ) {
+    let mounts = WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED;
     let stream = SharedStream::retaining_contents();
     let mut devices = memory_devices(&stream);
     let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
     let unit_area_start_slot = genesis.unit_area_start.slot().0;
-    let mounted = mount_writable_repeatedly(
-        &mut devices,
-        WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED,
-    );
+    let mounted = mount_writable_repeatedly(&mut devices, mounts);
     let PoolVersion::WithoutFile(version) = &mounted.current else {
         panic!("只暖机过的池上现行那一版树表 0 条")
     };
@@ -525,13 +528,16 @@ fn the_first_file_after_ten_warm_up_only_mounts_lands_on_the_reclaimed_unit_area
         &version.root,
         &version.record_bytes,
     );
+    let oldest_valid_root_txg_before_the_first_file =
+        first_file.root.checkpoint_txg.0 - ROOT_RING_SLOT_COUNT;
+    assert!(
+        oldest_valid_root_txg_before_the_first_file >= RELEASE_TXG_OF_THE_MKFS_INSTANCE_TABLE,
+        "前提：第 {mounts} 次挂载上的第一个文件版本（{:?}）写之前环里最旧有效根 txg {oldest_valid_root_txg_before_the_first_file} 不早于 mkfs 实例表那一对的释放代 {RELEASE_TXG_OF_THE_MKFS_INSTANCE_TABLE}",
+        first_file.root.checkpoint_txg
+    );
     assert_eq!(
-        (
-            first_file.root.checkpoint_txg,
-            first_file.tree_identifiers.inode.0
-        ),
-        (CheckpointTxg(29), 65537),
-        "第 10 次挂载上的第一个文件版本 txg 29，树仍从 mkfs 水位发"
+        first_file.tree_identifiers.inode.0, 65537,
+        "第 {mounts} 次挂载上的第一个文件版本，树仍从 mkfs 水位发"
     );
     assert_eq!(
         data_unit_slots_on_each_device(&first_file),
@@ -540,25 +546,23 @@ fn the_first_file_after_ten_warm_up_only_mounts_lands_on_the_reclaimed_unit_area
     );
     assert_the_first_data_unit_placement_holds_and_nothing_reddens(
         &memory_pool_of_sparse_devices(&devices),
-        "只暖机挂载 10 次之后写第一个文件",
+        &format!("只暖机挂载 {mounts} 次之后写第一个文件"),
     );
 }
 
-/// 回收门槛差一个 txg 的那一格（判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md` 第二节 K2-1）：只做过 mkfs 的池只暖机挂载 9 次，
-/// 第 9 次挂载上写第一个文件（txg 26）。mkfs 的实例表那一对释放代 3，写这一版之前环里最旧有效根是 txg 2，D3（空间分配） 已定项 7 的回收谓词
+/// 回收门槛差一个 txg 的那一格（判决 `research/prompts/m3-formal-code-b19-r1-main-verification.md` 第二节 K2-1）：只做过 mkfs 的池只暖机挂载 S + 1 次
+/// （今天 9），这一次挂载上写第一个文件（今天 txg 26）。mkfs 的实例表那一对释放代 3，写这一版之前环里最旧有效根是 txg 2，D3（空间分配） 已定项 7 的回收谓词
 /// （已释放 ∧ 释放代 ≤ max(F, 环里最旧有效根)）不成立，U、U + 1 还占着：实现把数据单元放在 U + 4。直接钉实现的落点——I-5.5 按这一版的分配记录现算，
 /// 放松一个 txg 的坏实现落 U 也判成立（C689），这里不靠它；池级 checker 照样一条都不红。
 #[test]
 fn the_first_file_after_nine_warm_up_only_mounts_lands_past_the_mkfs_instance_table_that_is_one_txg_short_of_reclaim(
 ) {
+    let mounts = WARM_UP_ONLY_MOUNTS_ONE_SHORT_OF_RECLAIMING_THE_MKFS_INSTANCE_TABLE;
     let stream = SharedStream::retaining_contents();
     let mut devices = memory_devices(&stream);
     let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
     let unit_area_start_slot = genesis.unit_area_start.slot().0;
-    let mounted = mount_writable_repeatedly(
-        &mut devices,
-        WARM_UP_ONLY_MOUNTS_ONE_SHORT_OF_RECLAIMING_THE_MKFS_INSTANCE_TABLE,
-    );
+    let mounted = mount_writable_repeatedly(&mut devices, mounts);
     let PoolVersion::WithoutFile(version) = &mounted.current else {
         panic!("只暖机过的池上现行那一版树表 0 条")
     };
@@ -568,13 +572,17 @@ fn the_first_file_after_nine_warm_up_only_mounts_lands_past_the_mkfs_instance_ta
         &version.root,
         &version.record_bytes,
     );
+    let oldest_valid_root_txg_before_the_first_file =
+        first_file.root.checkpoint_txg.0 - ROOT_RING_SLOT_COUNT;
     assert_eq!(
-        (
-            first_file.root.checkpoint_txg,
-            first_file.tree_identifiers.inode.0
-        ),
-        (CheckpointTxg(26), 65537),
-        "第 9 次挂载上的第一个文件版本 txg 26，树仍从 mkfs 水位发"
+        oldest_valid_root_txg_before_the_first_file + 1,
+        RELEASE_TXG_OF_THE_MKFS_INSTANCE_TABLE,
+        "前提：第 {mounts} 次挂载上的第一个文件版本（{:?}）写之前环里最旧有效根比 mkfs 实例表那一对的释放代早恰好一个 txg",
+        first_file.root.checkpoint_txg
+    );
+    assert_eq!(
+        first_file.tree_identifiers.inode.0, 65537,
+        "第 {mounts} 次挂载上的第一个文件版本，树仍从 mkfs 水位发"
     );
     assert_eq!(
         data_unit_slots_on_each_device(&first_file),
@@ -592,7 +600,7 @@ fn the_first_file_after_nine_warm_up_only_mounts_lands_past_the_mkfs_instance_ta
     );
     assert_the_first_data_unit_placement_holds_and_nothing_reddens(
         &memory_pool_of_sparse_devices(&devices),
-        "只暖机挂载 9 次之后写第一个文件",
+        &format!("只暖机挂载 {mounts} 次之后写第一个文件"),
     );
 }
 
diff --git a/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs b/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs
index 7ea47c43..8c9558dc 100644
--- a/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs
+++ b/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs
@@ -285,7 +285,8 @@ fn the_same_witness_refusal_in_a_crash_state_with_read_faults_injected_is_legal(
 }
 
 /// 撕裂系统配置那一支：盘 0 两槽都撕裂、没注入读故障，实现在取号之前以「这块盘一份自证过的系统配置都没有」拒（报成判据 N-配置 的拒因，C664）；
-/// C670 那一条跳过撕裂的槽复算判据为假（c_见证 等于第一个文件那一版末条的计数器），不报。U6 那四条里判据在崩溃镜像上复算：R2 之下撕裂的槽
+/// C670 那一条在「某块盘两槽都撕裂」这一形上照常比（跳过撕裂的槽取 c_见证），不因盘 0 一份自证过的都没有就按真；这份盘面上从盘 1 自证过的槽取、
+/// c_见证 等于第一个文件那一版末条的计数器，复算为假，不报（换一份盘面照常比可以为真）。U6 那四条里判据在崩溃镜像上复算：R2 之下撕裂的槽
 /// 不算判不出（没有槽设备报读错），盘 0 一份自证过的系统配置都没有 ⇒ 按真：合法。
 #[test]
 fn a_refusal_on_a_crash_image_without_read_faults_where_a_device_has_both_system_configuration_slots_torn_is_legal(
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index 59176aa4..4fba76e9 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -2404,3 +2404,19 @@ F3（21-2）：二次崩溃的种子不混第一次崩溃那个段内子集（
 C670（21-2）接线：第三截调用点把所选那一版交成第一次崩溃那一版（不是二次崩溃镜像上只读恢复落到的那一版；计数照样 (1, 1)，判官列出的那一支变）	crates/singlefs-checker-tier/src/crash_injection.rs	                crash_image_materialized: &second_crash_image_materialized,\n                version_landed_on_after_the_crash: report.effective_root,	                crash_image_materialized: &second_crash_image_materialized,\n                version_landed_on_after_the_crash: fields_read_after_the_first_crash.report.effective_root,	-p singlefs-checker-tier --lib -- crash_injection::tests::a_second_crash_landing_on_the_row_publish_whose_records_were_lost_names_the_selected_version_of_the_second_crash_image	a_second_crash_landing_on_the_row_publish_whose_records_were_lost_names_the_selected_version_of_the_second_crash_image
 E171 装置名字数法认错 REFERENCE_INTERVALS_ARE_EXACT（E170 第 2 次重跑登记修订 2 的 12.14；新分支的 I-3.5 换成 I-3.4，known_bad_entry_names_equal_the_original_file_in_order 那一格两边不等）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs	"REFERENCE_INTERVALS_ARE_EXACT" => "I-3.5".to_string(),	"REFERENCE_INTERVALS_ARE_EXACT" => "I-3.4".to_string(),	-p singlefs-checker-tier --bin e171_three_oracle_collusion	entry_names_are_read_in_file_order_from_the_twelve_space_indented_lines_of_the_function
 E171 装置名字数法认错 FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK（E170 第 2 次重跑登记修订 2 的 12.14；新分支的 I-5.5 换成 I-5.4）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs	"FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK" => {\n                    "I-5.5".to_string()	"FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK" => {\n                    "I-5.4".to_string()	-p singlefs-checker-tier --bin e171_three_oracle_collusion	entry_names_are_read_in_file_order_from_the_twelve_space_indented_lines_of_the_function
+C670（23-1 K3-a）接线：第三截调用点把所选那一版交成第一次崩溃那一版，越屏障靶向态（第一次崩溃丢第一个文件那次的记录、二次崩溃丢写行那次的记录）上计数由 (1, 1) 变 (1, 0)	crates/singlefs-checker-tier/src/crash_injection.rs	                crash_image_materialized: &second_crash_image_materialized,\n                version_landed_on_after_the_crash: report.effective_root,	                crash_image_materialized: &second_crash_image_materialized,\n                version_landed_on_after_the_crash: fields_read_after_the_first_crash.report.effective_root,	-p singlefs-checker-tier --lib -- crash_injection::tests::a_second_crash_losing_the_records_of_the_row_publish_after_a_first_crash_losing_the_records_of_the_first_file_counts_one_illegal_witness_refusal	a_second_crash_losing_the_records_of_the_row_publish_after_a_first_crash_losing_the_records_of_the_first_file_counts_one_illegal_witness_refusal
+F3-c（23-1）：二次崩溃种子的指纹只折段内第一个写落没落（长 12 的段 4095 个第一次崩溃态只剩 2 个种子）	crates/singlefs-checker-tier/src/crash_injection.rs	    persisted_within_the_segment.iter().fold(\n        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,	    persisted_within_the_segment.iter().take(1).fold(\n        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,	-p singlefs-checker-tier --lib -- crash_injection::tests::all_first_crash_states_of_a_segment_of_twelve_writes_seed_their_second_crashes_differently	all_first_crash_states_of_a_segment_of_twelve_writes_seed_their_second_crashes_differently
+A1（23-1）：建 inode 的发布把新 inode 写成 nlink = 0（正在删；池级 checker 照样绿，I-9.5 未实现）	crates/singlefs-core/src/transaction.rs	            nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,\n            // 建出来还没写过数据：长度 0，blocks 跟着是 0。	            nlink: NLINK_OF_AN_INODE_BEING_DELETED,\n            // 建出来还没写过数据：长度 0，blocks 跟着是 0。	-p singlefs-checker-tier --lib -- crash_injection::tests::the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published	the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published
+A1（23-1）：建 inode 的发布把新 inode 的对象出生代写成上一版的 checkpoint_txg（早一版；池级 checker 照样绿）	crates/singlefs-core/src/transaction.rs	            object_birth: txg,\n            nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,	            object_birth: previous.root.checkpoint_txg,\n            nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,	-p singlefs-checker-tier --lib -- crash_injection::tests::the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published	the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published
+A1（23-1）：建 inode 的发布把新 inode 的 size 写成 4096（blocks 跟着成 8；池级 checker 照样绿）	crates/singlefs-core/src/transaction.rs	            // 建出来还没写过数据：长度 0，blocks 跟着是 0。\n            size: 0,	            // 建出来还没写过数据：长度 0，blocks 跟着是 0。\n            size: 4096,	-p singlefs-checker-tier --lib -- crash_injection::tests::the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published	the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published
+A1（23-1）：崩溃注入第二截那一支（第一个文件已删完或正在删的带文件一版）建 inode 时写入时间交 0（池级 checker 照样绿）	crates/singlefs-checker-tier/src/crash_injection.rs	                PoolVersion::WithFile(current) => publish_new_inodes(\n                    &mut writer,\n                    &mut allocator,\n                    current,\n                    1,\n                    FIXED_WRITE_TIME_SECONDS,	                PoolVersion::WithFile(current) => publish_new_inodes(\n                    &mut writer,\n                    &mut allocator,\n                    current,\n                    1,\n                    0,	-p singlefs-checker-tier --lib -- crash_injection::tests::the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published	the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published
+崩溃注入第二截的发布臂（23-1 改法 B1）：第一个文件正在删的带文件一版上那一支建 0 个 inode（单元区 136 槽的短续做盘面上空发布落得下，结局 Published 而不是被单元区墙拒；原文同 21-2 改法 a 那一行）	crates/singlefs-checker-tier/src/crash_injection.rs	                PoolVersion::WithFile(current) => publish_new_inodes(\n                    &mut writer,\n                    &mut allocator,\n                    current,\n                    1,	                PoolVersion::WithFile(current) => publish_new_inodes(\n                    &mut writer,\n                    &mut allocator,\n                    current,\n                    0,	-p singlefs-checker-tier --test crash_injection_publish_after_a_mount_whose_deletion_continuation_stays_short	the_publish_after_a_mount_whose_deletion_continuation_stays_short_creates_an_inode_and_is_refused_at_the_unit_area_wall
+E165 r5 第三段 r5 档时间线记录的后三项写回 r1 常量（L 域 N ≤ 5、C8a 实例表指针 762 变 0；生成器与真相同错，J8 在今天的实现上照样 0）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	                self.fields_beyond_the_projection_of_version(publish_ordinal),	                RunProfile::R1.fields_beyond_the_projection_of_version(publish_ordinal),	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_r5_generator_puts_the_three_fields_of_each_publishs_root_into_every_record_of_that_publish
+E165 r5 第三段 r5 档时间线记录的后三项写回 r1 常量，看 C8a 钉值（实例表指针 762 变 0、卸载记号 620 变 762）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	                self.fields_beyond_the_projection_of_version(publish_ordinal),	                RunProfile::R1.fields_beyond_the_projection_of_version(publish_ordinal),	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_c8a_control_reads_762_762_620_on_the_r5_generator_and_0_on_the_r1_generator
+E165 r5 第三段 J8a 拿所选根当期望（回到旧规则；L 域 N = 2、P_1 落了那一态 J8a 两格各 1）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	    let expected_fields_of_the_rebuilt_root = fields_from_the_new_root_segment;	    let expected_fields_of_the_rebuilt_root = if true { RecordFieldsBeyondTheProjection::of_root(chosen_root) } else { fields_from_the_new_root_segment };	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	todays_replay_journal_breaks_no_j8_cell_on_the_big_ring_up_to_two_records
+E165 r5 第三段 J8b 拿落到的那一版的 fsid 当期望（C8c 假结局 J8b 762 变 0）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	    let expected_filesystem_identifier = chosen_root.filesystem_identifier;	    let expected_filesystem_identifier = landed_version_root.map_or(chosen_root.filesystem_identifier, |landed_root| landed_root.filesystem_identifier);	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_c8c_control_reads_762_on_j8b_and_0_on_the_three_j8a_cells_and_on_the_r1_generator
+E165 r5 第三段 J8c 只比卸载记号、漏比实例表指针等三项（手造：一条没施加、返回根的实例表指针被改过，J8c 1 变 0）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	            FieldsBeyondTheProjection::of_root(returned_root)\n                == FieldsBeyondTheProjection::of_root(chosen_root);	            returned_root.unmount_marker == chosen_root.unmount_marker;	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	j8c_counts_a_returned_root_that_kept_the_projection_but_changed_the_instance_table
+E165 r5 第三段 h(d) 的卸载记号取与 R_0 相同的值（A 域 n = 1 第一个字母那条记录的卸载记号由与 R_0 相反变成相同）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	                    unmount_marker: the_other_unmount_marker(marker_of_the_first_version),	                    unmount_marker: the_other_unmount_marker(the_other_unmount_marker(marker_of_the_first_version)),	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_records_off_the_timeline_carry_tails_distinct_from_every_version_and_the_unmount_marker_opposite_to_the_first_version
+E165 r5 第三段 生成器真相同一投影两份后三项时不停、取先到的那一份（C8e 手造真相应当停在 PH6 而没有）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	                *noted_fields == fields,	                true || *noted_fields == fields,	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	a_generated_record_truth_given_two_tails_for_one_projection_halts_on_ph6
+E165 r5 第三段 --profile r5 解析成 r3 档（任意 r5 跑法产物里没有 j8a_ 字段）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	        "r5" => Ok((\n            RunProfile::r5(marker_parity)	        "r5" => Ok((\n            RunProfile::r3(marker_parity)	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	without_profile_the_three_r1_product_commands_stay_on_the_r1_profile_and_the_marker_parity_needs_r2
+E165 r5 第三段 C8b 施加过时写的卸载记号写反成 Written（L 域 N ≤ 5、even 读数 142 变 620）	crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs	const UNMOUNT_MARKER_THE_C8B_CONTROL_WRITES_WHEN_APPLIED: UnmountMarker =\n    UnmountMarker::NotWrittenByTheUnmountSequence;	const UNMOUNT_MARKER_THE_C8B_CONTROL_WRITES_WHEN_APPLIED: UnmountMarker =\n    UnmountMarker::WrittenByTheUnmountSequence;	-p singlefs-checker-tier --bin e165_replay_bounded_enumeration	the_c8b_control_reads_142_on_even_and_620_on_odd_and_a_threshold_of_400_parts_the_two_parities
````
