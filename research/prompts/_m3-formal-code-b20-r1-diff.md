# 第 20 批代码轮改动全文（2026-10-04）

合入前备份对合入后主仓（含主 agent 重定基时手工接的两段与第 5999 行补的第三个参数）。

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index affcf4a4..4c526c08 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -44,12 +44,14 @@ use singlefs_core::instance_table_sweep::{
 };
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
 use singlefs_core::mount::{mount_writable_with_space_admission, MountError, MountOutput};
+use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
 use singlefs_core::recovery::{
     choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome,
     RecoveryReport,
 };
 use singlefs_core::transaction::{
     publish_first_file, publish_new_inodes, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
+    TransactionOutput,
 };
 use singlefs_core::write_accounting::{WritesByStructureKind, WrittenStructureKind};
 
@@ -517,7 +519,8 @@ pub struct CrashInjectionTally {
     pub writable_mounts_after_the_crash_succeeded: u64,
     pub writable_mounts_after_the_crash_refused_at_the_unit_area_wall: u64,
     pub writable_mounts_after_the_crash_refused: u64,
-    /// 挂载做成之后的那一次发布（带文件的一版上覆盖写、树表 0 条的一版上发第一个文件）：做成的、单元区墙、别的理由（判红）。
+    /// 挂载做成之后的那一次发布（第一个文件还活着的一版上覆盖写、树表 0 条的一版上发第一个文件、第一个文件已删完或正在删的一版上建一个 inode）：
+    /// 做成的、单元区墙、别的理由（判红）。
     pub publishes_after_the_writable_mount: u64,
     pub publishes_after_the_writable_mount_refused_at_the_unit_area_wall: u64,
     pub publishes_after_the_writable_mount_refused: u64,
@@ -1040,6 +1043,25 @@ impl StreamMarks {
     }
 }
 
+/// 崩溃窗口末尾那个写在录制流里的下标：窗口从写表第 `first_write_of_the_segment` 个写起数 `writes_from_the_segment_on` 个
+/// （靶向用例的窗口可以越过段尾），末尾是写表第 `first_write_of_the_segment + writes_from_the_segment_on - 1` 个写。
+/// 读故障的认定（[`StreamMarks::read_faults_in_a_crash_state`]）按录制流下标与每一步跑完时流里的步数比；写表下标不算屏障，
+/// 流里每过一道屏障两者就差一，拿写表下标去比会把窗口末尾认早（攻方 K2-c，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节）。
+///
+/// # Panics
+/// `writes_from_the_segment_on` 是 0（崩溃窗口里至少有一个写：段内真子集的长度是段的写数，靶向用例的写数由 [`draw_crash_points`] 拒 0），
+/// 或窗口末尾越过写表（`stream_indexes` 与写表逐条对应）。
+fn stream_index_of_the_last_write_of_the_crash_window(
+    stream_indexes: &[usize],
+    first_write_of_the_segment: usize,
+    writes_from_the_segment_on: usize,
+) -> usize {
+    let last_write_within_the_crash_window = writes_from_the_segment_on
+        .checked_sub(1)
+        .expect("崩溃窗口里至少有一个写：段内真子集的长度是段的写数，靶向用例的写数在摆的时候拒 0");
+    stream_indexes[first_write_of_the_segment + last_write_within_the_crash_window]
+}
+
 /// 这一类历史步在注入的读故障下读盘：崩溃恢复抛弃根（那次可写挂载期间最新那条根的根槽与它点名的单元读回全 0，
 /// `singlefs_harness::history` 的 `DeviceReadingZerosOverHiddenRanges`）。别的步读的就是盘上的字节。
 fn operation_reads_under_injected_faults(kind: HistoryOperationKind) -> bool {
@@ -1070,6 +1092,28 @@ pub fn inject_crashes_into_history(
     execution: HistoryExecution,
     draw: CrashPointDraw,
     image_directory: Option<&Path>,
+) -> HistoryCrashInjection {
+    inject_crashes_into_history_drawing_second_crashes(
+        history,
+        execution,
+        draw,
+        SecondCrashPointDraw::OneInEachMountPhaseBySeed,
+        image_directory,
+    )
+}
+
+/// [`inject_crashes_into_history`]，挂载途中的二次崩溃按 `second_crash_draw` 摆（靶向用例经它摆写死的那一个二次崩溃）。
+///
+/// # Panics
+/// 同 [`inject_crashes_into_history`]；`second_crash_draw` 是 [`SecondCrashPointDraw::ExactlyOneSecondCrashState`] 而写死的段号、
+/// 写数或扣下的下标在某一次挂载写出的写表上不成立（见那一支）。
+#[must_use]
+pub fn inject_crashes_into_history_drawing_second_crashes(
+    history: &GeneratedHistory,
+    execution: HistoryExecution,
+    draw: CrashPointDraw,
+    second_crash_draw: SecondCrashPointDraw,
+    image_directory: Option<&Path>,
 ) -> HistoryCrashInjection {
     let stream = SharedStream::retaining_contents();
     let mut committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = BTreeMap::new();
@@ -1292,7 +1336,11 @@ pub fn inject_crashes_into_history(
         let crash_image_materialized = materialized_crash_image(&image);
         let read_faults_in_the_crash_state = marks.read_faults_in_a_crash_state(
             &step_kinds,
-            stream_indexes[first_write_of_the_segment + writes_in_the_segment - 1],
+            stream_index_of_the_last_write_of_the_crash_window(
+                &stream_indexes,
+                first_write_of_the_segment,
+                writes_in_the_segment,
+            ),
         );
         match read_faults_in_the_crash_state {
             ReadFaultsInTheCrashState::NoneInjected => {}
@@ -1396,6 +1444,7 @@ pub fn inject_crashes_into_history(
                 space_admission: execution.space_admission,
                 reference_filesystem_at_the_first_crash: &reference_filesystem_judgement,
                 read_faults_in_the_first_crash_state: read_faults_in_the_crash_state,
+                second_crash_draw,
             },
             history.seed,
             crash_point,
@@ -1715,8 +1764,9 @@ fn content_of_the_publish_after_the_crash(seed: HistorySeed, crash_point: &Crash
 }
 
 /// 在 `crash_image_materialized` 的副本上起可写挂载（建池那一份参数、那一段历史的空间准入开关），做成了就接着发一次布：
-/// 挂载交回的现行版本带文件时覆盖写，树表 0 条时发第一个文件（与随机历史里这两个入口同一种调法），有树而文件集合为空（删完那一版）时
-/// 建一个 size 0 的 inode（D8（核心索引结构） 已定项 16 第 5 条）。
+/// 挂载交回的现行版本带文件、第一个文件那条记录还活着时覆盖写，树表 0 条时发第一个文件（与随机历史里这两个入口同一种调法），
+/// 有树而文件集合为空（删完那一版）时、或带文件而第一个文件已删完或正在删（unlink 之后再建过 inode）时建一个 size 0 的 inode
+/// （D8（核心索引结构） 已定项 16 第 4、5 条：写入口拒不在树里与正在删的 inode）。
 #[must_use]
 pub fn mount_and_publish_once_on_the_crash_image(
     crash_image_materialized: &MemoryPool,
@@ -1797,11 +1847,24 @@ pub fn mount_and_publish_once_on_the_crash_image(
             let mut allocator = mounted.allocator;
             let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
             let published = match &mounted.current {
-                PoolVersion::WithFile(current) => publish_overwrite(
+                PoolVersion::WithFile(current) if first_file_is_a_live_record(current) => {
+                    publish_overwrite(
+                        &mut writer,
+                        &mut allocator,
+                        current,
+                        file,
+                        mounted.output.instance,
+                    )
+                    .map(|_| ())
+                }
+                // 带文件的一版而第一个文件已删完（unlink 之后再建过 inode：树里只剩别的 inode）或正在删（nlink = 0）：覆盖写只写第一个文件，
+                // 在这一版上被拒成写一个不在树里、或正在删的 inode（D8（核心索引结构） 已定项 16 第 4 条）；与删完那一版一样建一个 size 0 的 inode。
+                PoolVersion::WithFile(current) => publish_new_inodes(
                     &mut writer,
                     &mut allocator,
                     current,
-                    file,
+                    1,
+                    FIXED_WRITE_TIME_SECONDS,
                     mounted.output.instance,
                 )
                 .map(|_| ()),
@@ -1892,6 +1955,16 @@ pub fn publishes_before_the_instance_table_sweep_of_the_mount_output(
     })
 }
 
+/// 带文件的一版里第一个文件那条记录还活着（在树里、nlink 不是 0）：覆盖写只写第一个文件，写入口拒删完与正在删的
+/// （`transaction::live_record_of_the_first_file`，D8（核心索引结构） 已定项 16 第 4 条）。unlink 之后再建过 inode 的一版仍归带文件的一版
+/// （`TransactionOutput::is_a_version_with_trees_and_no_file` 为假），它的 `inode_record` 是 None。
+fn first_file_is_a_live_record(version: &TransactionOutput) -> bool {
+    version
+        .inode_record
+        .as_ref()
+        .is_some_and(|record| record.nlink != NLINK_OF_AN_INODE_BEING_DELETED)
+}
+
 /// 一次发布写没写只有续做的删除步才写的那几种结构：文件的 extent 树、inode 树与 `deleted_inodes` 树（数据单元也算，挂载里没有别的发布写它）。
 /// 写行、暖机与抬 F 的空发布只重写分配记录树、记账树、中央映射树、树表、实例表与固定结构（2026-10-03 在副本上量的：崩在意图之后重挂，
 /// 写行与两次暖机写的是这几种，续做那一步写了 extent 树与 inode 树根）。
@@ -2167,7 +2240,7 @@ struct RefusalOnTheCrashImage<'crash> {
 /// （[`HistoryThenWritableMountRecords::record_check_after_the_writable_mount`]）。挂载被拒而理由在 D23（journal 的角色与格式） 已定项 14
 /// 那张单子上的，判拒绝本身（`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state`：判据在崩溃镜像上复算为真、
 /// 盘上逐字节不变、只读读出在允许集合里；崩溃态没注入读故障而判据 N-配置 复算为真的另判不合法，C670），合法的只计数，不合法的报红。别的理由被拒按「模型说该成、实现拒了」报：
-/// 模型对可写挂载与覆盖写、第一个文件从不要求拒，容量墙之外也不许拒（`IdealModel::answer_mount_writable`）。
+/// 模型对可写挂载与之后那一次发布（覆盖写活着的第一个文件、第一个文件、建 inode）从不要求拒，容量墙之外也不许拒（`IdealModel::answer_mount_writable`）。
 #[allow(
     clippy::too_many_arguments,
     reason = "每个参数各是一样东西：计数与发现、挂载那一趟与它之后的记录核对、判拒绝要的崩溃镜像那一份、这一个崩溃状态落在哪"
@@ -2252,7 +2325,7 @@ fn judge_the_writable_mount_on_the_crash_image(
     let model_disagreement = refusal.map(|implementation_answer| {
         ModelDisagreement::new(
             ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
-            "崩溃之后恢复到一版上，可写挂载（取号、写行、暖机）与之后的覆盖写或第一个文件都该成（单元区墙之外模型不许拒）"
+            "崩溃之后恢复到一版上，可写挂载（取号、写行、暖机）与之后那一次发布（第一个文件还活着时覆盖写、树表 0 条时第一个文件、第一个文件已删完或正在删时建一个 inode）都该成（单元区墙之外模型不许拒）"
                 .to_string(),
             implementation_answer,
         )
@@ -2630,7 +2703,7 @@ pub fn writes_of_the_instance_table_sweep_phases(
     by_phase
 }
 
-/// 挂载途中的一次二次崩溃：挂载那一段的第几段、段内哪几个写持久了、崩在挂载的哪一段。
+/// 挂载途中的一次二次崩溃：挂载那一段的第几段、段内哪几个写持久了（靶向用例摆的那一个从这一段的第一个写起数，可以越过段尾）、崩在挂载的哪一段。
 #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
 struct SecondCrashPoint {
     segment_index: usize,
@@ -2638,6 +2711,76 @@ struct SecondCrashPoint {
     phase: WritableMountPhase,
 }
 
+/// 挂载途中的二次崩溃怎么摆（[`judge_second_crashes_inside_the_writable_mount`]）。
+#[derive(Clone, Copy)]
+pub enum SecondCrashPointDraw {
+    /// 挂载的每一段各按种子摆一个（[`draw_second_crash_points`]）：[`inject_crashes_into_history`] 与战役都走这一支。
+    OneInEachMountPhaseBySeed,
+    /// 每个第一次崩溃态上都只摆这一个二次崩溃（靶向用例用，攻方改法 F2，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节 K2）：
+    /// 从挂载那一段写表切出的第 `segment_index` 段的第一个写起数 `writes_from_the_segment_on` 个写，其中下标（从这一段的第一个写数起）
+    /// 在 `withheld_among_them` 里的扣下、别的持久；更早的段整段持久，这几个写之后的一个都没持久。这几个写可以越过这一段的末尾，
+    /// 越过的那几道屏障当设备没守（同 [`CrashPointDraw::ExactlyOneCrashState`]：今天的持久顺序下根槽 FUA 先于轮换、各成一段，
+    /// 「轮换落了、根没落」在段内真子集里摆不出来）。崩在挂载的哪一段按第一个被扣下的写取。
+    /// 段号不在挂载写出的段里、这几个写越过挂载写表末尾、`writes_from_the_segment_on` 是 0、扣下的下标越界或一个都没有时 panic——
+    /// 写死的输入写错了，不悄悄摆别的状态（[`exactly_one_second_crash_point`]）。
+    ExactlyOneSecondCrashState {
+        segment_index: usize,
+        writes_from_the_segment_on: usize,
+        withheld_among_them: &'static [usize],
+    },
+}
+
+/// 靶向用例那一个二次崩溃（[`SecondCrashPointDraw::ExactlyOneSecondCrashState`]）：持久集合从挂载第 `segment_index` 段的第一个写起数，
+/// 可以越过段尾；崩在挂载的哪一段取第一个被扣下的写的（`phase_of_each_write` 与挂载写表逐条对应）。
+///
+/// # Panics
+/// 段号不在 `mount_segments` 里、`writes_from_the_segment_on` 是 0 或越过挂载写表末尾、扣下的下标一个都没有或越界。
+fn exactly_one_second_crash_point(
+    mount_segments: &[Vec<usize>],
+    phase_of_each_write: &[WritableMountPhase],
+    segment_index: usize,
+    writes_from_the_segment_on: usize,
+    withheld_among_them: &[usize],
+) -> SecondCrashPoint {
+    let first_write_of_the_segment = mount_segments
+        .get(segment_index)
+        .and_then(|segment| segment.first().copied())
+        .unwrap_or_else(|| {
+            panic!(
+                "靶向用例写死的挂载第 {segment_index} 段不在挂载写出的 {} 段里",
+                mount_segments.len()
+            )
+        });
+    assert!(
+        writes_from_the_segment_on > 0
+            && first_write_of_the_segment + writes_from_the_segment_on
+                <= phase_of_each_write.len(),
+        "靶向用例从挂载第 {segment_index} 段起数 {writes_from_the_segment_on} 个写：要至少一个、不越过挂载写表末尾（{} 个写）",
+        phase_of_each_write.len()
+    );
+    let first_withheld = withheld_among_them
+        .iter()
+        .min()
+        .copied()
+        .filter(|_| {
+            withheld_among_them
+                .iter()
+                .all(|withheld| *withheld < writes_from_the_segment_on)
+        })
+        .unwrap_or_else(|| {
+            panic!(
+                "靶向用例扣下的写 {withheld_among_them:?} 要至少一个、都落在从挂载第 {segment_index} 段起数的 {writes_from_the_segment_on} 个写里"
+            )
+        });
+    SecondCrashPoint {
+        segment_index,
+        persisted_within_the_segment: (0..writes_from_the_segment_on)
+            .map(|within_the_state| !withheld_among_them.contains(&within_the_state))
+            .collect(),
+        phase: phase_of_each_write[first_write_of_the_segment + first_withheld],
+    }
+}
+
 /// 挂载的每一段各摆一个二次崩溃：在那一段的写里按种子挑一个 w，崩在 w 上——w 所在那一段里 w 之前的写都落了、w 没落、
 /// w 之后的写各按种子抽；更早的段整段持久，更晚的段一个都没持久（与第一次崩溃同一个枚举域）。挂载没写出某一段的写就不摆那一段。
 fn draw_second_crash_points(
@@ -2720,6 +2863,8 @@ struct SecondCrashInputs<'history> {
     reference_filesystem_at_the_first_crash: &'history ReferenceFileSystemJudgementOfACrash,
     /// 第一次崩溃那个态注没注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：二次崩溃之后再挂被拒时判 C670 那一条沿用它。
     read_faults_in_the_first_crash_state: ReadFaultsInTheCrashState,
+    /// 二次崩溃怎么摆：挂载每一段按种子各摆一个，或靶向用例写死的那一个。
+    second_crash_draw: SecondCrashPointDraw,
 }
 
 /// 第三截：挂载途中的二次崩溃。每个二次崩溃状态上：只读恢复（看 journal）、问模型——允许的版本是模型提交过的每一版，加上这次挂载写出的根
@@ -2752,6 +2897,7 @@ fn judge_second_crashes_inside_the_writable_mount(
         space_admission,
         reference_filesystem_at_the_first_crash,
         read_faults_in_the_first_crash_state,
+        second_crash_draw,
     } = inputs;
     let mount_writes = records_across_the_crash.mount_writes();
     let phase_of_each_write = writable_mount_phase_of_each_write(
@@ -2759,8 +2905,22 @@ fn judge_second_crashes_inside_the_writable_mount(
         publishes_before_the_deletion_queue_continuation,
         publishes_before_the_instance_table_sweep,
     );
-    let second_crash_points =
-        draw_second_crash_points(seed, crash_point, mount_segments, &phase_of_each_write);
+    let second_crash_points = match second_crash_draw {
+        SecondCrashPointDraw::OneInEachMountPhaseBySeed => {
+            draw_second_crash_points(seed, crash_point, mount_segments, &phase_of_each_write)
+        }
+        SecondCrashPointDraw::ExactlyOneSecondCrashState {
+            segment_index,
+            writes_from_the_segment_on,
+            withheld_among_them,
+        } => vec![exactly_one_second_crash_point(
+            mount_segments,
+            &phase_of_each_write,
+            segment_index,
+            writes_from_the_segment_on,
+            withheld_among_them,
+        )],
+    };
     let mut versions_allowed = committed_versions.clone();
     if let Some(content) = &content_read_back_after_the_crash {
         for write in mount_writes {
@@ -3999,6 +4159,7 @@ mod tests {
     use super::*;
     use singlefs_core::mount::FloorRaiseFailedAfterTheMountsPublishes;
     use singlefs_harness::history::{generate_history, HistoryStartingPoint};
+    use singlefs_harness::mount_transparency::WitnessedCounterBranchThatHolds;
 
     const FOUR_CRASH_POINTS: CrashPointDraw = CrashPointDraw::Sampled {
         crash_points_per_history: 4,
@@ -4452,6 +4613,117 @@ mod tests {
         );
     }
 
+    /// unlink 之后再建过 inode 的一版归带文件的一版、第一个文件那条记录不在（删除与并发那条线读码诊断；攻方随机历史种子 32768、245760 两段那一形，
+    /// 判决 `research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节「越出攻击面」那一行）：第一个文件之后 unlink 它、再建一个 inode，
+    /// 崩在建 inode 那次发布里——根槽落了、紧跟着的系统配置轮换没落，只读恢复落在建 inode 那一版（这段历史最后提交的一版）。
+    /// 崩溃后镜像上的可写挂载做成，之后那一次发布建一个 inode、做成，第二截没有新发现。判别力：带文件那一臂改回「一律覆盖写」
+    /// （`crates/mutations.tsv` 一行），覆盖写被拒成写一个不在树里的 inode 1，计一次发布被拒、记一条第二截的新发现，这一条红。
+    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个崩溃态（ExactlyOneCrashState），不枚举
+    #[test]
+    fn the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published(
+    ) {
+        use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};
+        let execution = HistoryExecution {
+            per_step_checker: PerStepChecker::Skipped,
+            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
+            space_admission: SpaceAdmission::JudgedByTheFormula,
+        };
+        let history = GeneratedHistory {
+            seed: HistorySeed(0),
+            starting_point: HistoryStartingPoint::AfterFirstFile,
+            operations: vec![HistoryOperation::Unlink, HistoryOperation::CreateInode],
+        };
+        let stream = SharedStream::retaining_contents();
+        let mut operations_after_the_unlink = None;
+        let run = execute_history_with(&history, execution, &stream, &mut |observation| {
+            if observation.position == StepPosition::Operation(0) {
+                operations_after_the_unlink = Some(stream.operation_count());
+            }
+        });
+        assert_eq!(
+            run.ending,
+            HistoryEnding::Completed,
+            "前提：unlink 与建 inode 都跑完"
+        );
+        let operations_after_the_unlink =
+            operations_after_the_unlink.expect("unlink 那一步跑完时登记过一次");
+        let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
+            &stream.retained_operations(),
+            &execution.device_width.fixed_geometry(),
+        );
+        let root_of_the_inode_creation = (0..writes.len())
+            .rev()
+            .find(|write| {
+                stream_indexes[*write] >= operations_after_the_unlink
+                    && writes[*write].kind == StepKind::RootRecordFua
+            })
+            .expect("前提：建 inode 那次发布写了根槽");
+        assert_eq!(
+            writes
+                .get(root_of_the_inode_creation + 1)
+                .map(|write| write.kind),
+            Some(StepKind::SystemConfigurationSlot),
+            "前提：建 inode 那次发布的根槽 FUA 之后紧跟着轮换系统配置"
+        );
+        let segment_of_the_root =
+            segment_containing_the_write(&segments, root_of_the_inode_creation);
+        let first_write_of_the_segment = segments[segment_of_the_root][0];
+        let injection = inject_crashes_into_history(
+            &history,
+            execution,
+            CrashPointDraw::ExactlyOneCrashState {
+                segment_index: segment_of_the_root,
+                writes_from_the_segment_on: root_of_the_inode_creation + 2
+                    - first_write_of_the_segment,
+                withheld_among_them: vec![
+                    root_of_the_inode_creation + 1 - first_write_of_the_segment,
+                ]
+                .leak(),
+            },
+            None,
+        );
+        let tally = &injection.tally;
+        assert_eq!(
+            injection
+                .crash_points
+                .iter()
+                .map(|crash_point| crash_point.operation_kind)
+                .collect::<Vec<_>>(),
+            [Some(HistoryOperationKind::CreateInode)],
+            "前提：只摆写死的那一个崩溃态，崩在建 inode 那次发布里"
+        );
+        assert_eq!(
+            tally.crash_points_that_land_before_the_last_committed_version,
+            0,
+            "前提：建 inode 那次发布的根落了，崩溃态落在这段历史最后提交的那一版上：{}",
+            tally.render()
+        );
+        assert_eq!(
+            tally.writable_mounts_after_the_crash_succeeded,
+            1,
+            "前提：崩溃后镜像上的可写挂载做成：{}",
+            tally.render()
+        );
+        let findings_after_the_writable_mount: Vec<String> = injection
+            .new_findings
+            .iter()
+            .filter(|finding| finding.stage == CrashStateStage::AfterTheWritableMountAndOnePublish)
+            .map(CrashPointFinding::render)
+            .collect();
+        assert_eq!(
+            (
+                tally.publishes_after_the_writable_mount,
+                tally.publishes_after_the_writable_mount_refused
+            ),
+            (1, 0),
+            "第一个文件已删完的带文件一版上，挂载之后那一次发布建一个 inode、做成：{findings_after_the_writable_mount:?}"
+        );
+        assert!(
+            findings_after_the_writable_mount.is_empty(),
+            "第二截没有新发现：{findings_after_the_writable_mount:?}"
+        );
+    }
+
     /// 挂载每一段都摆得到二次崩溃的两段历史（U4，`research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：从第一个文件起的
     /// 随机一段（起点就是攻方 `research/prompts/m3-inv-a-r1-opus-output.md` 一·2 的「新池新建文件做完的镜像」），与第一个文件之后经挂着的会话
     /// unlink 它的一段（意图那次 + 最后一批：崩溃状态有落在意图那一版上的，那一版上的可写挂载续做、写出续做那一段，
@@ -5216,6 +5488,29 @@ mod tests {
         );
     }
 
+    /// 读故障认定拿的崩溃窗口末尾（攻方改法 F5，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节 K2）：
+    /// 写表 6 个写落在录制流第 0、1、3、4、6、9 步（第 2、5、7、8 步是屏障）。从写表第 2 个写起数 3 个，窗口末尾是写表第 4 个写、
+    /// 录制流第 6 步——交出去的是 6，不是写表下标 4；窗口越到写表末尾、只有一个写时同样按录制流下标交。
+    #[test]
+    fn the_end_of_a_crash_window_is_named_by_its_stream_index_and_not_by_its_write_index() {
+        let stream_indexes = [0, 1, 3, 4, 6, 9];
+        assert_eq!(
+            stream_index_of_the_last_write_of_the_crash_window(&stream_indexes, 2, 3),
+            6,
+            "写表第 2、3、4 个写，末尾那个在录制流第 6 步"
+        );
+        assert_eq!(
+            stream_index_of_the_last_write_of_the_crash_window(&stream_indexes, 4, 2),
+            9,
+            "窗口越到写表末尾：写表第 5 个写在录制流第 9 步"
+        );
+        assert_eq!(
+            stream_index_of_the_last_write_of_the_crash_window(&stream_indexes, 3, 1),
+            4,
+            "窗口只有写表第 3 个写：录制流第 4 步"
+        );
+    }
+
     /// C670：「崩溃恢复抛弃根」那一步一个都没往录制流里发（取号之前就拒了），盘面与没走这一步的历史一样：之后的崩溃态都不算注入过读故障。
     #[test]
     fn an_abandoning_recovery_that_sent_nothing_leaves_every_crash_state_without_read_faults() {
@@ -5387,7 +5682,8 @@ mod tests {
     /// `writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 的造法；越过了根槽 FUA 与轮换的先后，今天的持久顺序下段内真子集摆不出来）。
     /// 历史里没有带读故障的步；第二截的可写挂载以判据 N-配置 拒，盘没变、只读读出在允许集合里，U6 四条都满足，C670 那一条判不合法，计 1。
     /// 判别力：调用点把读故障的认定恒交 `Injected`、第二截换回不判 C670 的旧判官 `judge_writable_mount_refusal`（`crates/mutations.tsv` 两行），
-    /// 这一拒都判成合法，这一条红。harness 档不许依赖 checker 档，接线在 checker 档，用例只能写在这里。
+    /// 这一拒都判成合法，这一条红；调用点把所选那一版交成 None 时计数照样 (0, 1)，不合法改由「判据复算为假」给出，红在条件那两条断言上
+    /// （攻方改法 F1）。harness 档不许依赖 checker 档，接线在 checker 档，用例只能写在这里。
     // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个崩溃态（ExactlyOneCrashState），不枚举
     #[test]
     fn a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash(
@@ -5442,6 +5738,69 @@ mod tests {
             "第二截的可写挂载以判据 N-配置 拒、崩溃态没注入读故障而判据复算为真：拒得不合法，计 1：{}",
             tally.render()
         );
+        // 不合法是哪一条给的（攻方改法 F1，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节 K2）：调用点把所选那一版交成 None 时，
+        // C670 那一条不成立、U6「判据复算为假」那一条顶上来，计数照样是 (0, 1)；只有看判官列出的没满足的条件才分得开。
+        let conditions_not_met = conditions_not_met_named_by_the_one_illegal_writable_mount_refusal(
+            &injection.new_findings,
+        );
+        let witness_criterion_true_without_read_faults = format!(
+            "{:?}",
+            RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(
+                WitnessedCounterBranchThatHolds::WitnessedCounterAboveTheLastRecordOfTheSelectedVersion
+            )
+        )
+        .split('(')
+        .next()
+        .expect("split 至少交回一段")
+        .to_string();
+        let criterion_false_when_recomputed = format!(
+            "{:?}",
+            RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
+        );
+        assert!(
+            conditions_not_met.contains(&witness_criterion_true_without_read_faults),
+            "拒得不合法要由 C670 那一条（{witness_criterion_true_without_read_faults}）给出：判官列出的没满足的条件 {conditions_not_met}"
+        );
+        assert!(
+            !conditions_not_met.contains(&criterion_false_when_recomputed),
+            "判据在崩溃镜像上复算为真（U6 那一条满足），不许列出 {criterion_false_when_recomputed}：判官列出的没满足的条件 {conditions_not_met}"
+        );
+    }
+
+    /// 第二截那一条拒得不合法的发现里，判官列出的没满足的条件（[`judge_the_writable_mount_on_the_crash_image`] 写进实现那一半的
+    /// 「拒得不合法（…）：」那一段，`Vec<RefusalConditionNotMet>` 的 Debug 文本）。
+    ///
+    /// # Panics
+    /// 第二截的新发现不是恰好一条、它没有模型对不上那一半、或那一半里没有「拒得不合法（…）：」那一段。
+    fn conditions_not_met_named_by_the_one_illegal_writable_mount_refusal(
+        new_findings: &[CrashPointFinding],
+    ) -> String {
+        let findings_after_the_writable_mount: Vec<&CrashPointFinding> = new_findings
+            .iter()
+            .filter(|finding| finding.stage == CrashStateStage::AfterTheWritableMountAndOnePublish)
+            .collect();
+        let [finding] = findings_after_the_writable_mount.as_slice() else {
+            panic!(
+                "第二截要恰好一条新发现（那一次拒得不合法）：{:?}",
+                findings_after_the_writable_mount
+                    .iter()
+                    .map(|finding| finding.render())
+                    .collect::<Vec<_>>()
+            );
+        };
+        let implementation_answer = &finding
+            .observation
+            .model_disagreement
+            .as_ref()
+            .unwrap_or_else(|| panic!("拒得不合法记成模型对不上：{}", finding.render()))
+            .implementation_answer;
+        implementation_answer
+            .split_once("拒得不合法（")
+            .and_then(|(_, after)| after.split_once("）："))
+            .map(|(conditions, _)| conditions.to_string())
+            .unwrap_or_else(|| {
+                panic!("实现那一半里没有「拒得不合法（…）：」那一段：{implementation_answer}")
+            })
     }
 
     /// C670 在第三截的判法（攻方改法 P1）：把「只落了覆盖写那次轮换」的盘面当成一个二次崩溃镜像交给第三截，没崩过的那一次挂载做成过（`once` 有内容）。
@@ -5590,4 +5949,128 @@ mod tests {
                 .collect::<Vec<_>>()
         );
     }
+
+    /// C670 在第三截经调用点的判法（攻方改法 F2，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节 K2）：第一个文件之后一次覆盖写的历史，
+    /// 第一次崩溃摆在覆盖写那次发布的第一个写上（它一个写都没落，盘面就是起点那一版）；第二截那次可写挂载的写行那次发布上摆一个二次崩溃——
+    /// 根槽 FUA 没落、紧跟着的两槽系统配置轮换落了（越过根槽那一段的段尾：今天的持久顺序下根槽 FUA 先于轮换、各成一段，段内真子集摆不出来，
+    /// 与 [`a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash`] 越过段尾同一个理由）。
+    /// 新实例的记录不跨实例边界施加，二次崩溃镜像上只读恢复落在起点那一版，见证值比它新：再挂以判据 N-配置 拒；崩溃态没注入读故障、判据复算为真，
+    /// 第三截判过一次、不合法一次。判别力：调用点（[`judge_second_crashes_inside_the_writable_mount`] 拼 [`RefusalOnTheCrashImage`] 那几行）
+    /// 把读故障恒交 `Injected`、再挂的盘换成第一次崩溃的镜像、换成没叠这一段的基线（`crates/mutations.tsv` 三行），不合法都变 0。
+    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
+    #[test]
+    fn a_second_crash_where_only_the_rotation_of_the_row_publish_landed_is_an_illegal_witness_refusal_through_the_call_site(
+    ) {
+        let (history, execution) = history_of_the_first_file_and_one_overwrite();
+        let (writes, segments, first_write_of_the_overwrite) =
+            writes_segments_and_first_write_of_the_overwrite(&history, execution);
+        let overwrite_segment_index = segments
+            .iter()
+            .position(|segment| segment[0] == first_write_of_the_overwrite)
+            .expect("覆盖写那次发布从一段的第一个写起（它前面是起点末尾那道屏障）");
+        // 第二截那次挂载写出的写表：与注入里那一次同一份盘面（覆盖写一个写都没落）、同一份参数；挂载之后那一次发布写的内容随崩溃点变，不在挂载写表里。
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
+        let rotation_after_the_root: Vec<usize> = (root_of_the_row_publish + 1..mount_writes.len())
+            .take_while(|write| mount_writes[*write].kind == StepKind::SystemConfigurationSlot)
+            .collect();
+        assert_eq!(
+            rotation_after_the_root.len(),
+            2,
+            "前提：写行那次发布的根槽 FUA 之后紧跟着每块盘轮换一槽系统配置"
+        );
+        let segment_of_the_root =
+            segment_containing_the_write(&mount_segments, root_of_the_row_publish);
+        let first_write_of_the_segment_of_the_root = mount_segments[segment_of_the_root][0];
+        let last_rotation = rotation_after_the_root[rotation_after_the_root.len() - 1];
+        let injection = inject_crashes_into_history_drawing_second_crashes(
+            &history,
+            execution,
+            CrashPointDraw::ExactlyOneCrashState {
+                segment_index: overwrite_segment_index,
+                writes_from_the_segment_on: 1,
+                withheld_among_them: &[0],
+            },
+            SecondCrashPointDraw::ExactlyOneSecondCrashState {
+                segment_index: segment_of_the_root,
+                writes_from_the_segment_on: last_rotation + 1
+                    - first_write_of_the_segment_of_the_root,
+                withheld_among_them: vec![
+                    root_of_the_row_publish - first_write_of_the_segment_of_the_root,
+                ]
+                .leak(),
+            },
+            None,
+        );
+        let tally = &injection.tally;
+        assert_eq!(
+            (tally.crash_points, tally.writable_mounts_after_the_crash_succeeded),
+            (1, 1),
+            "前提：只摆写死的那一个第一次崩溃态，它上面的可写挂载做成（没崩过的那一次读得出内容）：{}",
+            tally.render()
+        );
+        assert_eq!(
+            tally.crash_points_after_a_step_reading_under_injected_faults,
+            0,
+            "这段历史没有带读故障的步：{}",
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
+    }
 }
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
index 8e612da3..ab130785 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
@@ -160,9 +160,6 @@ const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: u32 = 116u;
 // mkfs 种下的树 ID 水位（singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS），I-5.5 认法 ① 拿 extent 树条目的树 ID 比它
 const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u32 = 65536u;
 
-// 从 mkfs 水位第一次建树那一版，每块盘上落点最小的数据单元比单元区起始槽号多几槽（walk.rs 的 FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START）
-const FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START: u32 = 4u;
-
 // 违例位：位 i 在第 i / 32 字的第 i % 32 位（位 i ↔ IMPLEMENTED_INVARIANTS[i]），存进结论的第 POOL_CHECKER_VIOLATED_FIRST_WORD 字起。
 var<private> violated: array<u32, POOL_CHECKER_VIOLATED_WORDS>;
 
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index 9cbf6c89..5bc40da1 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -196,7 +196,9 @@ pub const REFERENCE_INTERVALS_ARE_EXACT: &str = "I-3.5";
 /// 不拿字面 65537 比（D8（核心索引结构） 已定项 8「头 ID 8 字节的取值」句末的关系判据，判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K1）。
 pub const HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES: &str = "I-9.27";
 /// I-5.5：从 mkfs 水位第一次建树的那一版（树表非空、种类 1 条目的树 ID 是 65536、每条诞生 txg 都等于这一版的 `checkpoint_txg`）引用的数据单元里，
-/// 每块盘上落点槽号最小的那一个等于单元区起始槽号 U + 4（D3（空间分配） 已定项 8 射程，判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2）。
+/// 每块盘上落点槽号最小的那一个等于这一版分配记录照 D3（空间分配） 已定项 8 第 1 条那条政策算出的那一对的起点槽号（起点最小、32768 对齐、
+/// 两槽都空、不在开放段里；新池新建文件那一次是单元区起始槽号 U + 4，回收之后不再是）（D3（空间分配） 已定项 8 射程，判决
+/// `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2；按分配记录现算：判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K2）。
 pub const FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK: &str = "I-5.5";
 /// I-9.28：根环里全部读得出且自证合法的根（不按实例表判有效与否，含被崩溃恢复抛弃的旧线上的根），任两条根各自的树表里同一树 ID
 /// （条目偏移 0）的条目，诞生 txg 相同。与 I-9.14 的分工：I-9.14 只比同一条时间线上的根，这一条不分时间线（树 ID 被重发时
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index 597ba75d..b816479c 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -2359,3 +2359,11 @@ C673 ⑤ I-9.27 判法（19-1）：种类 1、9、10 的头 ID 不比 inode 树
 第9批 I-7.9 一圈窗口：只取锚在被判那条根上的窗口（由记录施加出来那一类洞，txg 1 顶上来又红）	crates/singlefs-checker/src/walk.rs	                && window_anchored_at_the_raising_root.contains(root.checkpoint_txg)\n                && window_anchored_at_the_newest_root_of_the_image.contains(root.checkpoint_txg)\n	                && window_anchored_at_the_raising_root.contains(root.checkpoint_txg)\n	-p singlefs-harness --test pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window -- a_remount_over_a_root_ring_hole_whose_withheld_root_its_record_applies_keeps_an_old_floor_raise_within_its_ceiling	a_remount_over_a_root_ring_hole_whose_withheld_root_its_record_applies_keeps_an_old_floor_raise_within_its_ceiling
 第9批 I-7.9 一圈窗口：只取锚在被判那条根上的窗口（K1 类洞，txg 2 顶上来又红）	crates/singlefs-checker/src/walk.rs	                && window_anchored_at_the_raising_root.contains(root.checkpoint_txg)\n                && window_anchored_at_the_newest_root_of_the_image.contains(root.checkpoint_txg)\n	                && window_anchored_at_the_raising_root.contains(root.checkpoint_txg)\n	-p singlefs-harness --test pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window -- a_remount_over_a_root_ring_hole_left_by_a_withheld_row_publish_keeps_an_old_floor_raise_within_its_ceiling	a_remount_over_a_root_ring_hole_left_by_a_withheld_row_publish_keeps_an_old_floor_raise_within_its_ceiling
 第9批 I-7.9 一圈窗口坏镜像：准入那一支恒判成立（F 高于窗口里的上限不再红）	crates/singlefs-checker/src/walk.rs	        judgements.judge("I-7.9", raised_floor <= ceiling.lowest_possible, || {	        judgements.judge("I-7.9", true, || {	-p singlefs-harness --test pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window -- a_floor_raised_above_the_ceiling_of_the_roots_inside_the_window_still_reddens_the_rollback_floor_invariant	a_floor_raised_above_the_ceiling_of_the_roots_inside_the_window_still_reddens_the_rollback_floor_invariant
+C670（20-1 F1）接线：第二截调用点把所选那一版交成 None（不合法改由 U6「判据复算为假」给出，计数照样 (0, 1)）	crates/singlefs-checker-tier/src/crash_injection.rs	                crash_image_materialized: &crash_image_materialized,\n                version_landed_on_after_the_crash: report.effective_root,	                crash_image_materialized: &crash_image_materialized,\n                version_landed_on_after_the_crash: None,	-p singlefs-checker-tier --lib -- crash_injection::tests::a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash	a_crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash
+C670（20-1 F2）接线：第三截调用点把读故障的认定恒交 Injected（不沿用第一次崩溃那个态的认定）	crates/singlefs-checker-tier/src/crash_injection.rs	                read_faults: read_faults_in_the_first_crash_state,	                read_faults: ReadFaultsInTheCrashState::Injected,	-p singlefs-checker-tier --lib -- crash_injection::tests::a_second_crash_where_only_the_rotation_of_the_row_publish_landed_is_an_illegal_witness_refusal_through_the_call_site	a_second_crash_where_only_the_rotation_of_the_row_publish_landed_is_an_illegal_witness_refusal_through_the_call_site
+C670（20-1 F2）接线：第三截调用点再挂的盘交成第一次崩溃的镜像（不是二次崩溃镜像）	crates/singlefs-checker-tier/src/crash_injection.rs	                crash_image_materialized: &second_crash_image_materialized,	                crash_image_materialized,	-p singlefs-checker-tier --lib -- crash_injection::tests::a_second_crash_where_only_the_rotation_of_the_row_publish_landed_is_an_illegal_witness_refusal_through_the_call_site	a_second_crash_where_only_the_rotation_of_the_row_publish_landed_is_an_illegal_witness_refusal_through_the_call_site
+C670（20-1 F2）接线：第三截调用点再挂的盘交成没叠二次崩溃那一段的基线 second_base	crates/singlefs-checker-tier/src/crash_injection.rs	                crash_image_materialized: &second_crash_image_materialized,	                crash_image_materialized: &second_base,	-p singlefs-checker-tier --lib -- crash_injection::tests::a_second_crash_where_only_the_rotation_of_the_row_publish_landed_is_an_illegal_witness_refusal_through_the_call_site	a_second_crash_where_only_the_rotation_of_the_row_publish_landed_is_an_illegal_witness_refusal_through_the_call_site
+C670（20-1 F5）读故障认定的崩溃窗口末尾交成写表下标（不是录制流下标）	crates/singlefs-checker-tier/src/crash_injection.rs	    stream_indexes[first_write_of_the_segment + last_write_within_the_crash_window]	    first_write_of_the_segment + last_write_within_the_crash_window	-p singlefs-checker-tier --lib -- crash_injection::tests::the_end_of_a_crash_window_is_named_by_its_stream_index_and_not_by_its_write_index	the_end_of_a_crash_window_is_named_by_its_stream_index_and_not_by_its_write_index
+崩溃注入第二截的发布臂（20-1）：带文件的一版一律覆盖写（unlink 之后再建过 inode 的一版上被拒成写一个不在树里的 inode 1）	crates/singlefs-checker-tier/src/crash_injection.rs	                PoolVersion::WithFile(current) if first_file_is_a_live_record(current) => {	                PoolVersion::WithFile(current) if true || first_file_is_a_live_record(current) => {	-p singlefs-checker-tier --lib -- crash_injection::tests::the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published	the_publish_after_a_writable_mount_on_a_version_created_after_the_first_file_was_unlinked_creates_an_inode_and_is_published
+C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：模型答树表 0 条那一版上卸载之后会话照旧开着（判决 m3-formal-code-b18-r1 第二节 K3「会话关」那条断言）	crates/singlefs-harness/src/model.rs	        answer.session_after_success = ModelSessionAfterSuccess::Closed;	        answer.session_after_success = if current.contents.has_trees() {\n            ModelSessionAfterSuccess::Closed\n        } else {\n            ModelSessionAfterSuccess::StaysOpen\n        };	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees	unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees
+C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：模型关会话时把现行那一版那条根带的 F 记成它的 txg（树表 0 条那一版上卸载一条根都不写、F 却动了；判决 m3-formal-code-b18-r1 第二节 K3「F 不动」那条断言）	crates/singlefs-harness/src/model.rs	            ModelSessionAfterSuccess::Closed => self.session = None,	            ModelSessionAfterSuccess::Closed => {\n                if let Some(session) = self.session.take() {\n                    if let Some(root) = self\n                        .ring\n                        .values_mut()\n                        .find(|root| root.key == session.current.key)\n                    {\n                        root.rollback_floor = session.current.key.checkpoint_txg;\n                    }\n                }\n            }	-p singlefs-harness --test model_answers_unmount_with_the_floor_at_the_current_version -- unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees	unmount_on_a_version_without_trees_writes_nothing_and_the_model_answers_zero_roots_and_agrees
````
