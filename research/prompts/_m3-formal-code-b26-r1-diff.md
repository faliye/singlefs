# 第 26 批代码轮第一轮：改动全文（2026-10-05）

三份补丁按合入次序：26-2、26-3（打在根环线第 10b 批之后的树上）、26-1r（26-1 在合入 26-2、26-3 之后的树上重打）。每份照实现员交回的原样。

## m3-formal-26-2

### crates.patch

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_identity.rs b/crates/singlefs-checker-tier/src/crash_identity.rs
index 47def7a..2b4f9d0 100644
--- a/crates/singlefs-checker-tier/src/crash_identity.rs
+++ b/crates/singlefs-checker-tier/src/crash_identity.rs
@@ -898,6 +898,16 @@ pub const RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED: &[RandomHistoryThrough
         library_module: "crates/singlefs-harness/src/model_comparison.rs",
         reason: "崩溃注入拿随机历史当输入：战役经 crash_injection.rs 用它把崩溃之后读回的结果对到理想模型；它只在自己的单元测试模块里引随机历史脚手架（词法闭包照收），判定随对拍的写法变",
     },
+    RandomHistoryThroughALibraryModuleAllowed {
+        test_target: "crash_injection_campaign",
+        library_module: "crates/singlefs-harness/src/reference_filesystem_translation.rs",
+        reason: "崩溃注入拿随机历史当输入：战役经 crash_injection.rs 用 RefFS 翻译层把历史操作、每步结局与崩溃时刻翻成 RefFS 操作再判恢复读出（形式化接入线 12-2），翻译层直接引随机历史脚手架的操作与观察类型，判定随历史怎么生成、怎么执行变",
+    },
+    RandomHistoryThroughALibraryModuleAllowed {
+        test_target: "crash_injection_campaign",
+        library_module: "crates/singlefs-harness/src/fault_injection.rs",
+        reason: "崩溃注入拿随机历史当输入：RefFS 翻译层把历史操作翻成 RefFS 操作时要认每步的故障结局（形式化接入线 12-2），经它引故障注入的结局类型，故障注入又引随机历史脚手架的执行器；判定随带故障执行历史的写法变",
+    },
 ];
 
 /// 随机历史脚手架的一个引入者判下来是什么。
diff --git a/crates/singlefs-checker-tier/tests/a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs b/crates/singlefs-checker-tier/tests/a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs
index 6330d1b..0af7cc0 100644
--- a/crates/singlefs-checker-tier/tests/a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs
+++ b/crates/singlefs-checker-tier/tests/a_publish_returns_only_after_the_system_configuration_rotation_is_durable.rs
@@ -10,6 +10,9 @@
 //! 每个可达的崩溃状态里，池里两块盘两槽全部自证过的系统配置的 journal tail 取最大（见证值，与可写挂载判 N-配置、取号写的见证值同一取法）
 //! 都不小于这次发布末条记录的计数器。计数器全池接着走（D23（journal 的角色与格式） 已定项 14 第 3 条），后面换了实例也照比。
 //! 见证值只由落在系统配置两槽上的写定：崩溃状态只在「还没被屏障放行的那一段」里的系统配置槽写上取子集，别的写不改它。
+//! 唯一许落在系统配置两槽上的别的写是 mkfs 每块盘最先发的 [0, 根环基址) 整段清零（D22（单元原子性怎么合成） 已定项 8 第 3 条）：
+//! 它发在那块盘第一次系统配置槽写之前，那时这块盘上系统配置两槽还是空白镜像的全零，清零不改见证值；
+//! 同一块盘上已有系统配置槽写（不论放没放行）之后再清零到两槽上，判法就不成立，照旧停下。
 //! 撕开的原地覆写不另枚举：被撕的是较旧的那一槽（两槽轮换），撕坏自证不过，与那一写没持久对见证值是同一个数。
 
 #[path = "../../singlefs-harness/tests/common/mod.rs"]
@@ -19,6 +22,8 @@ use common::{
     build_pool, format_pool, geometry, parameters, publish_overwrite_in_process, BuiltPool,
     Recorded, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
 };
+use std::collections::BTreeSet;
+
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
 };
@@ -125,8 +130,9 @@ fn lowest_witness_over_the_open_subsets(
 /// 录制流每个崩溃点（已发出前 p 步，p 从 0 到流长）上，全部可达崩溃状态里见证值的最小值；交回的第 p 项是第 p 个崩溃点的。
 /// 切段照 [`SegmentClosingRule`]（与层 0 同一条规则）：段关上时段里的写全持久，还开着的那一段里的写任取子集。
 ///
-/// 循环：按录制流逐步（轮数 = 步数），跨轮携带「已持久的系统配置槽写施加出的镜像」与「当前段里的系统配置槽写」，
-/// 段关上时把后者按次序施加进前者再清空；没有提前出口。
+/// 循环：按录制流逐步（轮数 = 步数），跨轮携带「已持久的系统配置槽写施加出的镜像」、「当前段里的系统配置槽写」与
+/// 「发过系统配置槽写的盘」（只增不减），段关上时把第二样按次序施加进第一样再清空；没有提前出口。
+/// 落在系统配置两槽上的整段清零只许发在那块盘第一次系统配置槽写之前（那时两槽在空白镜像上全零，清零不改见证值），否则停下。
 fn lowest_witness_at_every_crash_point(operations: &[RetainedOperation]) -> Vec<u64> {
     let classifying_geometry = geometry();
     let system_configuration_end_in_bytes = SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
@@ -135,13 +141,20 @@ fn lowest_witness_at_every_crash_point(operations: &[RetainedOperation]) -> Vec<
         MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], IMAGE_BYTES);
     let mut open_system_configuration_writes: Vec<&RetainedOperation> = Vec::new();
     let mut closing_rule = SegmentClosingRule::default();
+    let mut devices_with_a_system_configuration_slot_write: BTreeSet<DeviceIdentity> =
+        BTreeSet::new();
     let mut lowest_witnesses = vec![witness_on(&persisted_image)];
     for retained in operations {
         match classifying_geometry.classify(&retained.operation) {
-            StepKind::SystemConfigurationSlot => open_system_configuration_writes.push(retained),
+            StepKind::SystemConfigurationSlot => {
+                devices_with_a_system_configuration_slot_write.insert(retained.operation.device);
+                open_system_configuration_writes.push(retained);
+            }
             StepKind::ZeroFill => assert!(
-                retained.operation.offset.0 >= system_configuration_end_in_bytes,
-                "整段清零落在系统配置两槽上：见证值不再只由系统配置槽写定，这份判法不成立：{:?}",
+                retained.operation.offset.0 >= system_configuration_end_in_bytes
+                    || !devices_with_a_system_configuration_slot_write
+                        .contains(&retained.operation.device),
+                "整段清零落在一块已发过系统配置槽写的盘的系统配置两槽上：见证值不再只由系统配置槽写定，这份判法不成立：{:?}",
                 retained.operation
             ),
             StepKind::UnitWrite
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs
index 5bc356c..7a1cef7 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs
@@ -506,6 +506,8 @@ fn begin_line(stream: &str) -> String {
 /// 先红后改的那一条：两类违例、两遍恢复、好几段里都红，今天的计数只留一条 `first_violation`；发现表按签名去重，每个签名的状态数、
 /// 最先 3 个状态与逐状态重判的相同，发现日志跑完是定稿（一个签名一行、汇总行的数对得上）；跑的过程中每并进一片，这一片带来的
 /// 新签名、跨过 10 的签名已经落在文件里（观察者在下一片并进来之前读得到），同一签名只报一次「新」。
+/// C577 那道屏障之后（46 个状态）这份版本表下最多的签名只有 8 个状态，没有签名跨过 10：跨台阶那一报在这里只核「没跨就不报」，
+/// 「跨过就报一次」这一面这条流上没有对象。
 #[test]
 fn red_states_are_deduplicated_by_signature_and_the_findings_log_is_readable_while_running() {
     let stream = first_stream("findings-dedupe");
@@ -579,15 +581,30 @@ fn red_states_are_deduplicated_by_signature_and_the_findings_log_is_readable_whi
         .map(Vec::len)
         .max()
         .expect("有签名");
+    // 钉值出处：2026-10-04 主工作区 checker 档日志 `LAYER0_FINDINGS signatures=15 red_states=25 states=46`，断言报「15 个签名，最多的 8 个状态」
+    // （C577 屏障让暖机末尾两次系统配置槽写自成一段，54 个状态那时写的门槛 10 跟着落空）。
     assert!(
-        rejudged.red_states_by_signature.len() >= 4 && largest_signature_states >= 10,
-        "至少四个签名、至少一个签名跨过 10 个状态：{} 个签名，最多的 {largest_signature_states} 个状态",
+        rejudged.red_states_by_signature.len() >= 4 && largest_signature_states >= 8,
+        "至少四个签名、至少一个签名跨过 8 个状态：{} 个签名，最多的 {largest_signature_states} 个状态",
         rejudged.red_states_by_signature.len()
     );
     assert_findings_match_the_rejudged(&tally.findings, &rejudged);
 
     let log_text = std::fs::read_to_string(&findings_log_path).expect("读发现日志");
     assert_final_section_matches_the_rejudged(&log_text, &rejudged, &begin_line("unnamed"));
+    let signatures_crossing_ten = rejudged
+        .red_states_by_signature
+        .values()
+        .filter(|red_states| red_states.len() >= 10)
+        .count();
+    assert_eq!(
+        log_text
+            .lines()
+            .filter(|line| line.starts_with("layer0_finding_threshold\t"))
+            .count(),
+        signatures_crossing_ten,
+        "跨台阶那一报恰好每个跨过 10 的签名一行（46 个状态的流上没有签名跨过 100），没跨的不报"
+    );
 
     // 跑的过程中：观察者在第 k 个状态那一片并进来之前读的文件，恰好有序号小于 k 的状态带来的新签名与跨台阶。
     for (position, (signature, red_states)) in rejudged
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
index 02f4627..102946f 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
@@ -87,8 +87,11 @@ fn checker_line(tally: &Layer0Tally) -> String {
 }
 
 /// checker 与记录核对器那一半的钉死值。只在根槽 txg 3 已持久的状态上才走得到记账树、inode 树、分配记录树与中央映射树的条目，
-/// 那 16 条只在这些状态上评估
+/// 那 22 条只在这些状态上评估
 /// （I-5.4（分配记录罩住的槽互不相交） 在其中，
+/// I-9.25（树表条目的 previous_snapshot_txg）、I-9.27（树表条目的头 ID）、I-3.13（记账行的代段）、I-9.26（inode 记录的改动计数）、
+/// I-5.5（从 mkfs 水位建树那一版的第一个数据单元） 与 I-3.5（引用区间的精确性） 同一个理由：种子根与暖机根指着的第 0 版树表是空的，
+/// 只有 txg 3 那一版有树表条目、记账树、inode 记录与数据单元；记录施加出来的那一版由 I-4.1 另起一份走读去走、判定丢掉，不进这几条的计数，
 /// 映射 key 与单元头相符（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`，实审 B2 新立、编号待定）判的是中央映射树叶里的每一条条目：
 /// 种子根与暖机根的映射根指针全零（没有文件的一版），走不到条目；txg 3 的记录已落而根槽没落时，由记录施加出来的那一版
 /// 在这条流上不走（它要实例表里有一行记着某次恢复施加到过 txg 3，实例表的行是挂载写行时写的，而这条流上唯一一次挂载在 txg 3 之前），
@@ -100,10 +103,13 @@ fn checker_line(tally: &Layer0Tally) -> String {
 /// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 单列一类：它除了回退候选集里那几版，还读下一次挂载会先施加的那一版
 /// （`.claude/kb/invariants.md` I-3.10 那一行射程 ④），txg 3 的记录至少一份已落、根槽没落的状态上也评估，
 /// 评估过的状态数由调用方按录制流逐状态现算（[`TransactionPublishInRecordedStream::allocation_generation_read_set_in`]），不写死；
-/// I-9.14（树表条目的诞生 txg 跨根不变）在这条流上一个状态都评估不到；
+/// I-9.14（树表条目的诞生 txg 跨根不变）在这条流上一个状态都评估不到，I-9.28（全部读得出的根之间同一树 ID 的诞生 txg 相同） 同一个理由；
+/// I-3.12（实例表行 T 低于引用它的根的 txg） 也评估不到：这条流上没有哪次发布重写实例表（取号只写系统配置，暖机与 A 照抄
+/// mkfs 那张只有链记录、没有行的实例表）；
 /// I-8.6（反向链算法） 要环里至少有一条自证过的记录，环还空着的那几个状态上报「不适用」；
 /// I-8.7（实例内事务号不重号）、I-8.8（前缀里的事务不被切开） 与 I-7.9（回退下界 F 不高于抬 F 的上限） 在这条流上一个状态都评估不到；
-/// 其余 23 条每个状态都评估。
+/// I-8.10、I-8.11、I-8.12、I-4.1、I-7.16 各自一类，评估过的状态数逐条写在函数里那几张表旁边；
+/// 其余 33 条每个状态都评估（2026-10-04 主工作区 checker 档日志快档那一行 `CHECKER`：69 条里 33 条是 46/0）。
 /// 违例只许出现在 I-7.7（系统配置实例代号不低于根环）：C322（取号那一步的屏障怎么放没有条款） 未还，
 /// 取号只落了一块盘的那 2 个状态里两份系统配置的实例代号不等。
 fn assert_checker_counts(
@@ -138,6 +144,12 @@ fn assert_checker_counts(
         "I-9.13",
         "I-9.15",
         MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
+        "I-9.25",
+        "I-9.27",
+        "I-3.13",
+        "I-9.26",
+        "I-5.5",
+        "I-3.5",
     ];
     // I-3.10 除了回退候选集，还读下一次挂载会先施加的那一版（射程 ④）：新根没落而 txg 3 的记录已落时，
     // 恢复要由那条记录重建 txg 3 那一版，它的分配记录树在记录的新根段指着的树表下面。
@@ -154,7 +166,10 @@ fn assert_checker_counts(
     // 违例数仍由下面那条断言钉 0）。它在哪一条流上都评估不到，判别力在 `checker_known_bad_images.rs` 的阳性对照与坏镜像上。
     // I-7.9（回退下界 F 不高于抬 F 的上限） 只判抬 F 的那一条根，这条流上没抬过 F（每条根带的 F 都是 0）⇒ 每个状态都报「不适用」。
     // 抬过 F 的流在 `crash_enumeration_fixed_script_stream.rs`（E 之前抬到 11），那里它真被评估过。
-    let never_comparable_on_this_stream = ["I-9.14", "I-8.7", "I-8.8", "I-7.9"];
+    // I-9.28 要两条根的树表里有同一个树 ID 的条目才比得出来，这条流上只有 txg 3 那条根的树表有条目 ⇒ 每个状态都没有对象。
+    // I-3.12 要根指着的实例表里有行，这条流上全部根都指着 mkfs 那张只有链记录的实例表（取号不写行、暖机与 A 照抄实例表指针）。
+    // 两条的 0 都对得上 2026-10-04 主工作区 checker 档日志快档那一行 `CHECKER` 的 `I-9.28=0/0`、`I-3.12=0/0`。
+    let never_comparable_on_this_stream = ["I-9.14", "I-8.7", "I-8.8", "I-7.9", "I-3.12", "I-9.28"];
     // I-8.6 判的是「本实例内逻辑前一条」与这一条的链值，环里一条自证过的记录都没有时判不了：
     // 取号那一段（两盘各一次系统配置写，原地覆写各三态）的 3² − 1 个状态，加上暖机第一次的记录段整段没落那一个，
     // 一共 9 个状态还没有记录落盘（补第三态之前 4 个）。
@@ -162,6 +177,18 @@ fn assert_checker_counts(
     // I-8.9（一次发布的记录序号连续且只有末条带标志） 同一个理由：环里有一条自证过的记录就判得了（记录标志其余位、序号 0 逐条判），
     // 没有就报不适用；这条流上 w1、w4、t9 各是一次只有一条记录的发布（序号 1、带末条标志）。
     let judged_once_a_record_is_on_disk = ["I-8.6", "I-8.9"];
+    // I-8.10 拿带末条标志的记录的新根段去比根环里同 (实例, txg) 的根：环里有记录还不够，还要它那次发布的根已落。
+    // 环空着的 [`STATES_WITH_AN_EMPTY_JOURNAL_RING`] 个之外，暖机第一次的记录落了而它的根没落的
+    // [`STATES_WITH_ONLY_RECORDS_OF_THE_FIRST_WARM_UP_BEFORE_ITS_ROOT`] 个也没有可比的根（mkfs 的种子根是实例 0、txg 0）。
+    let judged_once_a_record_and_the_root_of_its_publish_are_on_disk = ["I-8.10"];
+    // I-8.11 判点名项与被指单元的头：只有 A 的记录点名单元（暖机是零单元的空发布），A 的记录一份落了才有对象。
+    let judged_once_a_record_of_the_file_publish_is_on_disk = ["I-8.11"];
+    // I-8.12 比同一块盘上同一实例前后两条记录的 txg：一块盘上要有两条实例 1 的记录（暖机第二次的记录落了）才有对象。
+    let judged_once_a_device_holds_two_records_of_one_instance = ["I-8.12"];
+    // I-4.1 走重放之后那一版：所选根之上一条都不施加时报不适用。
+    let judged_only_when_the_replay_applies_a_record = ["I-4.1"];
+    // I-7.16 判可择的系统配置槽的世代号：每块盘都只剩一槽可择、而且是 mkfs 写下的世代号 1 时一格都不判。
+    let judged_unless_every_device_has_one_choosable_slot_written_by_mkfs = ["I-7.16"];
     for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
         let expected_evaluated = if never_comparable_on_this_stream.contains(&invariant) {
             0
@@ -173,6 +200,21 @@ fn assert_checker_counts(
             states_judging_allocation_generations
         } else if judged_once_a_record_is_on_disk.contains(&invariant) {
             every_state - states_with_an_empty_journal_ring
+        } else if judged_once_a_record_and_the_root_of_its_publish_are_on_disk.contains(&invariant)
+        {
+            every_state
+                - states_with_an_empty_journal_ring
+                - STATES_WITH_ONLY_RECORDS_OF_THE_FIRST_WARM_UP_BEFORE_ITS_ROOT
+        } else if judged_once_a_record_of_the_file_publish_is_on_disk.contains(&invariant) {
+            root_persisted_states + STATES_WITH_A_RECORD_OF_THE_FILE_PUBLISH_BUT_NOT_ITS_ROOT
+        } else if judged_once_a_device_holds_two_records_of_one_instance.contains(&invariant) {
+            every_state - STATES_BEFORE_A_DEVICE_HOLDS_TWO_RECORDS_OF_ONE_INSTANCE
+        } else if judged_only_when_the_replay_applies_a_record.contains(&invariant) {
+            STATES_WHERE_THE_REPLAY_APPLIES_A_RECORD
+        } else if judged_unless_every_device_has_one_choosable_slot_written_by_mkfs
+            .contains(&invariant)
+        {
+            every_state - STATES_WHERE_EVERY_DEVICE_HAS_ONE_CHOOSABLE_SLOT_WRITTEN_BY_MKFS
         } else {
             every_state
         };
@@ -454,6 +496,22 @@ const ROOT_PERSISTED_STATES: u64 = 9;
 const FILE_READ_STATES: u64 = 12;
 /// 环里一条记录都还没有的状态：取号那一段 3² − 1，加暖机第一次记录段整段没落那一个（补第三态之前 4）。
 const STATES_WITH_AN_EMPTY_JOURNAL_RING: u64 = 9;
+/// 环里只有暖机第一次（实例 1、txg 1）的记录、它的根还没落的状态：那一段两份记录只落一份的 2 个，加它的根槽段（根没落）那 1 个。
+/// 快档与全量相同（A 的单元段在它们之后）。快档 I-8.10 = 46 − 9 − 3 = 34，对得上 2026-10-04 主工作区 checker 档日志 `I-8.10=34/0`。
+const STATES_WITH_ONLY_RECORDS_OF_THE_FIRST_WARM_UP_BEFORE_ITS_ROOT: u64 = 3;
+/// A 的记录至少一份落了、A 的根没落的状态：记录段两份只落一份的 2 个，加根槽段（根没落）那 1 个。快档与全量相同（A 的单元段只在
+/// 记录之前展开，那里记录一份都没落）。I-8.11 = 9 + 3 = 12，对得上同一份日志的 `I-8.11=12/0`。
+const STATES_WITH_A_RECORD_OF_THE_FILE_PUBLISH_BUT_NOT_ITS_ROOT: u64 = 3;
+/// 哪块盘上都还没有两条实例 1 的记录的状态：取号段 8、暖机第一次的记录段 3、根槽段 1、轮换段 8，加暖机第二次记录段整段没落那 1 个。
+/// 快档与全量相同（A 的单元段在它们之后）。快档 I-8.12 = 46 − 21 = 25，对得上同一份日志的 `I-8.12=25/0`。
+const STATES_BEFORE_A_DEVICE_HOLDS_TWO_RECORDS_OF_ONE_INSTANCE: u64 = 21;
+/// 重放在所选根之上施加记录的状态：暖机第二次与 A 各 3 个（记录落了一份或两份、根没落）。暖机第一次那 3 个不算：记录是实例 1 的，
+/// 所选根是 mkfs 的实例 0 种子根，前缀只接同实例的记录，一条都不施加。快档与全量相同（全量多出来的 A 单元段状态上 A 的记录一份都没落）。
+/// 对得上同一份日志的 `I-4.1=6/0`，与 oracle 那一半的 `verification_ran_states` 6 是同一批状态（推的：恢复施加了记录才跑验证）。
+const STATES_WHERE_THE_REPLAY_APPLIES_A_RECORD: u64 = 6;
+/// 每块盘都只剩一槽可择、而且是 mkfs 写下的世代号 1 的状态：取号那一段两块盘的写都撕开的那 1 个（另一槽还是 mkfs 写的 1）。
+/// 之后每一段里每块盘都已有一槽世代号 ≥ 2。快档与全量相同。快档 I-7.16 = 46 − 1 = 45，对得上同一份日志的 `I-7.16=45/0`。
+const STATES_WHERE_EVERY_DEVICE_HAS_ONE_CHOOSABLE_SLOT_WRITTEN_BY_MKFS: u64 = 1;
 
 fn prepare(tag: &str) -> Prepared {
     let pool = build_pool(tag);
@@ -744,9 +802,10 @@ fn positive_control_root_persisted_without_each_unit_is_caught_by_the_oracle() {
 }
 
 /// 记录核对器的判别力（C6（块层语义假设写错） 那条「崩溃测试必须变红」在模型层的形态，与 E77（发布的持久顺序） b_ur 臂同形）：
-/// 摘掉新池新建文件里「journal 记录 → 根槽」那道池屏障（录制器按设备记屏障，两块盘各一步，两步都摘），两份记录与根槽同段；
+/// 摘掉新池新建文件里「journal 记录 → 根槽」那道池屏障（录制器按设备记屏障，两块盘各一步，两步都摘），两份记录、根槽与之后的
+/// 两次系统配置槽轮换写同段（段按设备记屏障：根槽 FUA 只放行它自己那块盘，另一块盘上的记录没放行，这一段开到轮换之后那道屏障才关）；
 /// 枚举里出现根在案而记录一份都不在的状态。oracle 在这里不红。只摘盘 1 那一步的判别力在
-/// `crash_segments_per_device_and_torn_in_place_overwrites.rs`。
+/// `crash_segments_per_device_and_torn_in_place_overwrites.rs`（那里同样是 `[…, 24, 5]`、同样按 `<= 5` 展开）。
 #[test]
 fn removing_the_barrier_before_the_root_slot_is_caught_by_the_record_checker() {
     let pool = build_pool("record-checker-barrier-two");
@@ -770,26 +829,29 @@ fn removing_the_barrier_before_the_root_slot_is_caught_by_the_record_checker() {
     let (writes, segments) = writes_and_segments(&operations, &geometry());
     assert_eq!(
         segments.iter().map(Vec::len).collect::<Vec<_>>(),
-        vec![2, 2, 1, 2, 2, 1, 2, 24, 3, 2],
-        "记录两份与根槽并成一段（C577 之后暖机第二次的轮换 2 写自成一段，不与 A 的 24 个单元写同段）"
+        vec![2, 2, 1, 2, 2, 1, 2, 24, 5],
+        "记录两份、根槽与 A 的两次系统配置槽轮换写并成一段（C577 之后暖机第二次的轮换 2 写自成一段，不与 A 的 24 个单元写同段）"
     );
     let root_index = writes
         .iter()
         .rposition(|write| write.kind == StepKind::RootRecordFua)
         .expect("根槽");
-    // A 的 24 个单元写那一段不展开（只以整段持久进入后面的状态），其余每段任意子集。
+    // A 的 24 个单元写那一段不展开（只以整段持久进入后面的状态），其余每段任意子集；最后那一段 5 个写，选择器放到 5 才展开得到它
+    // （留在 3 时这一段只以整段持久进来，根在案而记录不在的状态一个都摆不出来）。
     let tally = enumerate_layer0_selecting(
         &base,
         &writes,
         &segments,
         root_index,
         &file_content(),
-        &|_segment_index, segment| segment.len() <= 3,
+        &|_segment_index, segment| segment.len() <= 5,
     );
     assert_eq!(tally.violations, 0, "oracle 不红：根记录自己带着全部字段");
+    // 五个写那一段（记录 2、根槽 1、系统配置槽轮换 2）里根槽落了、两份记录都没落的状态：两次轮换写是原地覆写、各取三态（没落、落了、撕开），
+    // 3² = 9 个，都是真子集（记录没落）。记录核对器的「根在案而记录不在」只看盘上的根槽与记录写，不看系统配置槽（推的，checker 档没跑）。
     assert_eq!(
-        tally.record_root_without_record, 1,
-        "三个写那一段的真子集里只有「只有根槽」这一个是根在案而两份记录都不在"
+        tally.record_root_without_record, 9,
+        "五个写那一段的真子集里根槽在、两份记录都不在的是两次轮换写各三态的 3² 个"
     );
     assert_eq!(tally.record_claimed_state_missing_unit, 0);
 }
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
index 7f1f598..943a0e6 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
@@ -826,8 +826,7 @@ fn print_the_unsharded_enumerations_for_the_golden_comparison() {
 /// `LAYER0_PARALLEL_START` + 46 行 `LAYER0_PROGRESS` + `LAYER0_FINDINGS` + `LAYER0_PARALLEL_FINISHED` + `GOLDEN_TALLY` 共 51 行。
 /// C577（系统配置没见证到的最新根，乙罩不到）那道发布返回之前的屏障加上之后甲二那一段 54 → 46 个状态（`crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs` 文件头），
 /// 行数与进度文件行数照这个数静态改（合入后验证二，改法 B 报告第四节第 5 条推的，没跑）；池级 checker 多一条 I-7.14 之后
-/// 打印的计数行与进度文件内容也跟着变。**两个 SHA-256 与进度文件名里的计划哈希留原值，待第五步 checker-tier-layer0-replay 跑出来重录**
-/// （checker 档，子 agent 不跑）。
+/// 打印的计数行与进度文件内容也跟着变。打的行与计划哈希 2026-10-04 照日志重录（见下），进度文件内容的 SHA-256 还待重录。
 ///
 /// 钉值的来历：头一版取自加分片之前的代码（`crash.rs` 与 `layer0_progress.rs` 加分片之前那一版：38 行、SHA-256 `6deeb48f…`，
 /// 进度文件计划哈希 `a1d502a7…`、内容 `798bef3c…`），证的是加分片没改不分片那条路。实审 B3a-2 按设备记屏障、原地覆写补第三态之后
@@ -837,13 +836,19 @@ fn print_the_unsharded_enumerations_for_the_golden_comparison() {
 /// `LAYER0_FINDINGS signatures=0 red_states=0 states=54`、进度文件格式号 1 → 2 且每行片行多一个 `findings=none`，在那一版上重取
 /// （同一棵树上加发现日志之前是 65 行 `8e1ca03b…`、进度文件 `6ec51601…`，→ 下面的值）：重取之前逐行对过，打的行只多出那两行
 /// `LAYER0_FINDINGS`，进度文件去掉每行的校验和、`findings=none` 与格式号之后逐字节相同，进度文件名（计划哈希）不变。
+/// 2026-10-04 打的行与进度文件名重取自主工作区 checker 档日志（这条用例判红时把 58 行原样打在断言消息里，整段 58 行带换行的 SHA-256
+/// 与断言 `left` 里的 `7e64a883…` 逐字相同）：比 `15867e36…` 那一版不同的只有两趟的 `GOLDEN_TALLY`（池级 checker 多出的不变量各一项，
+/// I-1.12 … I-9.28）与 `LAYER0_RESUME` 里进度文件名的计划哈希（fcc4d340 那一版的 `6118958c…` → `0370ad26…`；HEAD 31e47115 的日志里
+/// 计划哈希也是 `0370ad26…`、打的行只比主工作区少 `I-9.28=0/0/46` 那两项）。
 const GOLDEN_PRINTED_LINE_COUNT: usize = 58;
 const GOLDEN_PRINTED_LINES_SHA256: &str =
-    "15867e366db44d88e7de85fafe40ab6940310fc9cfe1a75787846c39b1024212";
-/// 同一趟留下的进度文件：名字（带计划哈希）与整份内容的 SHA-256（47 行：文件头 + 46 行片行；两个值都待第五步 checker-tier-layer0-replay 重录，见上）。
-const GOLDEN_PROGRESS_FILE_NAME: &str = "layer0-progress-golden_new_pool_file_creation_stream-fingerprint0-5026593803c5654d562ab1858215ddd4a5be477116564e882947660016a1eb5e.txt";
-const GOLDEN_PROGRESS_FILE_SHA256: &str =
-    "6a153e1e268f829997ed67198ce0d125a450d0b12c4cdfbf5e6c1024ad8d5cd8";
+    "7e64a883680d9d609e033fd228790db29e8c32b3beb3798b920712f749d70ad4";
+/// 同一趟留下的进度文件：名字（带计划哈希）与整份内容的 SHA-256（47 行：文件头 + 46 行片行）。名字取自上面那一趟日志的 `LAYER0_RESUME` 行；
+/// 内容的 SHA-256 那份日志里没有（打的行那一核先红，进度文件这一核没跑到），**待重录**：checker 档跑
+/// `cargo test --release -p singlefs-checker-tier --test crash_enumeration_sharded_across_processes -- --exact unsharded_enumeration_prints_and_writes_byte_for_byte_what_it_did_before_sharding`，
+/// 取「不分片时的进度文件与钉值不同」那条断言 `left` 的第二项填进来。
+const GOLDEN_PROGRESS_FILE_NAME: &str = "layer0-progress-golden_new_pool_file_creation_stream-fingerprint0-0370ad26cad051266c57d7129a63840319034b488aefec9adebf007a9809b91e.txt";
+const GOLDEN_PROGRESS_FILE_SHA256: &str = "待重录：进度文件内容的 SHA-256，取法见上";
 
 /// 一行打印里会随机器与时刻变的两样换掉：进度目录、`elapsed_seconds=` 的值。
 fn masked_printed_line(line: &str, directory: &Path) -> String {
diff --git a/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs b/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
index 4faf102..4033ae4 100644
--- a/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
+++ b/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
@@ -1231,6 +1231,113 @@ fn allocation_and_accounting_trees_carry_the_byte_table_numbers() {
     assert_eq!(pool.output.key_order_mismatches, 0);
 }
 
+/// D3（空间分配） 已定项 10 ⑤ 那张落点表给分配记录树五个节点（t5–t9）写的出生序号 0..4（C686）：按位置先叶后根、同层按 key 升序 bump，
+/// 出生序号按同一个次序发（D19（块指针的结构与宽度预算） 已定项 9）。每个节点两块盘上的两份逐字节相同，头里的树 ID、层级、出生序号与表里那一行相同；
+/// 树表条目与父节点条目里指向它的指针，落点与出生序号也与表里那一行相同；两片叶里的记录各是表里写的那块盘的。
+#[test]
+fn allocation_record_tree_nodes_carry_birth_sequences_zero_to_four_as_the_bump_order_table_lists() {
+    let pool = build_pool("allocation-birth-sequences");
+    // (表里那一行, 落点, 层级, 出生序号, 罩哪块盘)：照 D3 已定项 10 ⑤ 的表逐行抄；根罩两块盘，记 `None`。
+    let bump_order_table: [(&str, u64, u8, u32, Option<DeviceIdentity>); 5] = [
+        (
+            "t5 分配记录树叶，设备 0",
+            50245,
+            0,
+            0,
+            Some(DeviceIdentity(0)),
+        ),
+        (
+            "t6 分配记录树叶，设备 1",
+            50246,
+            0,
+            1,
+            Some(DeviceIdentity(1)),
+        ),
+        (
+            "t7 分配记录树层级 1 节点，设备 0",
+            50247,
+            1,
+            2,
+            Some(DeviceIdentity(0)),
+        ),
+        (
+            "t8 分配记录树层级 1 节点，设备 1",
+            50248,
+            1,
+            3,
+            Some(DeviceIdentity(1)),
+        ),
+        ("t9 分配记录树根，层级 2", 50249, 2, 4, None),
+    ];
+    for (row, slot, level, birth_sequence, covered_device) in bump_order_table {
+        let copy_on_device_zero = read_unit_from(&pool, DeviceIdentity(0), SlotNumber(slot), 16384);
+        let copy_on_device_one = read_unit_from(&pool, DeviceIdentity(1), SlotNumber(slot), 16384);
+        assert_eq!(
+            copy_on_device_zero, copy_on_device_one,
+            "{row}：两块盘上的两份逐字节相同"
+        );
+        let view = index_node_view(&copy_on_device_zero).expect("分配记录树节点");
+        assert_eq!(
+            (view.tree_identifier, view.level, view.birth_sequence),
+            (
+                singlefs_format::TREE_IDENTIFIER_ALLOCATION_RECORDS,
+                level,
+                birth_sequence
+            ),
+            "{row}：头里的树 ID、层级、出生序号与 D3 已定项 10 ⑤ 的落点表相同"
+        );
+        if level == 0 {
+            let devices_of_the_records: BTreeSet<DeviceIdentity> = view
+                .entries
+                .iter()
+                .map(|bytes| {
+                    AllocationRecord::parse(bytes)
+                        .expect("分配记录树叶的条目是 20 字节的分配记录（同文件分配记录树那条用例钉过条目宽）")
+                        .device
+                })
+                .collect();
+            assert_eq!(
+                devices_of_the_records.into_iter().collect::<Vec<_>>(),
+                covered_device.into_iter().collect::<Vec<_>>(),
+                "{row}：叶里的记录全是表里写的那块盘的"
+            );
+            continue;
+        }
+        let children: Vec<(u64, u32)> = view
+            .entries
+            .iter()
+            .map(|entry| {
+                let child = NodePointer::read_from(&mut ByteReader::at(entry, 10));
+                (child.locations[0].slot.0, child.birth_sequence.0)
+            })
+            .collect();
+        let children_in_the_table: Vec<(u64, u32)> = bump_order_table
+            .iter()
+            .filter(|(_, _, child_level, _, child_device)| {
+                *child_level + 1 == level
+                    && covered_device.is_none_or(|device| *child_device == Some(device))
+            })
+            .map(|(_, child_slot, _, child_birth_sequence, _)| (*child_slot, *child_birth_sequence))
+            .collect();
+        assert_eq!(
+            children, children_in_the_table,
+            "{row}：条目里孩子指针的落点与出生序号是表里低一层、同一块盘那几行的"
+        );
+    }
+    let allocation_entry = tree_table_entries_from_disk(&pool)
+        .into_iter()
+        .find(|entry| entry.tree.0 == singlefs_format::TREE_IDENTIFIER_ALLOCATION_RECORDS)
+        .expect("分配记录树");
+    assert_eq!(
+        (
+            allocation_entry.root.locations[0].slot.0,
+            allocation_entry.root.birth_sequence.0
+        ),
+        (50249, 4),
+        "树表条目里分配记录树的根指针指着表里 t9 那一行"
+    );
+}
+
 /// 树表 11 条（D18（块里携带什么信息） 已定项 24 第 6 条：7 → 11，`deleted_inodes`、墓碑树、谱系树、logged_ops 零根注册），
 /// 树 ID、种类码、头 ID 三列逐条钉条款登记的值：树 ID 从 65536 起（D8（核心索引结构） 已定项 11），种类码 9..12 接在 8 之后（已定项 9），
 /// 头 ID 照已定项 8 / D5（快照 / 空间记账机制） 已定项 9（inode 树、extent 树、`deleted_inodes`、墓碑树写 inode 树的树 ID 65537，其余 0）；后四条根指针全零。
````

### mutations-append.tsv

````text
C577 判法：mkfs 写完一块盘的系统配置两槽之后又整段清零两槽再重写（清零落在已发过系统配置槽写的盘上，盘面终态不变）	crates/singlefs-core/src/make_filesystem.rs	        for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {\n            let offset = DeviceOffsetInBytes(\n                slot_index * u64::from(parameters.geometry.fixed_structure_slot_spacing),\n            );\n            device.write_at(offset, &slot, WriteDurability::Plain)?;\n        }	        for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {\n            let offset = DeviceOffsetInBytes(\n                slot_index * u64::from(parameters.geometry.fixed_structure_slot_spacing),\n            );\n            device.write_at(offset, &slot, WriteDurability::Plain)?;\n        }\n        device.write_zeroes_at(DeviceOffsetInBytes(0), SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE * u64::from(parameters.geometry.fixed_structure_slot_spacing))?;\n        for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {\n            let offset = DeviceOffsetInBytes(\n                slot_index * u64::from(parameters.geometry.fixed_structure_slot_spacing),\n            );\n            device.write_at(offset, &slot, WriteDurability::Plain)?;\n        }	-p singlefs-checker-tier --test a_publish_returns_only_after_the_system_configuration_rotation_is_durable -- every_crash_point_after_a_publish_returns_keeps_a_system_configuration_witness	every_crash_point_after_a_publish_returns_keeps_a_system_configuration_witness_of_its_last_record
C686：分配记录树同层节点按盘号倒序 bump（盘 1 的叶与第 1 层节点先发出生序号、先取落点，实现自洽、与 ⑤ 表对不上）	crates/singlefs-core/src/allocation_record_tree.rs	    let nodes = nodes_after\n        .iter()\n        .map(|node| {	    let mut nodes_in_the_mutated_order: Vec<&AllocationRecordTreeNode> = nodes_after.iter().collect();\n    nodes_in_the_mutated_order.sort_by_key(|node| match node { AllocationRecordTreeNode::BelowTheRoot(position) => (0u8, position.level, u32::MAX - position.device.0, position.index_in_device), AllocationRecordTreeNode::Root => (1u8, 0, 0, 0) });\n    let nodes = nodes_in_the_mutated_order\n        .into_iter()\n        .map(|node| {	-p singlefs-harness --test new_pool_file_creation_publish -- allocation_record_tree_nodes_carry_birth_sequences	allocation_record_tree_nodes_carry_birth_sequences_zero_to_four_as_the_bump_order_table_lists
````

## m3-formal-26-3

### crates.patch

````diff
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index 56faf95..728ac6e 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -35,7 +35,8 @@ pub enum InvariantVerdict {
     NotApplicable(&'static str),
     /// 部分没判（用户 2026-10-04 定，第四种判定）：判到的那一部分都对得上，射程里另有一部分这张镜像上判不了，说明逐条列出没判到的
     /// 是什么（例如 I-9.28 那几条根：盘号、txg、为什么没判）。**它不是红，也不是成立**：计红、计违例的地方算不红（层 0 与崩溃放量
-    /// 把它算成不红），计「成立」的地方不算成立。今天只有 I-9.28 报它（树表读不出、条目比字段表短的根）。
+    /// 把它算成不红），计「成立」的地方不算成立。今天 I-9.28 报它（树表读不出、条目比字段表短的根），I-5.5 报它（某一版树表读不出、
+    /// 条目窄、extent 树或分配记录树读不全、种类 1 / 3 条数不对，`research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y3）。
     PartiallyJudged(String),
 }
 
@@ -1291,6 +1292,37 @@ pub fn judge_pointer_mac_and_nonce_are_zero(
     });
 }
 
+/// 根记录里算法类型 1 + nonce 12 + MAC 16 那 29 字节（D22（单元原子性怎么合成） 已定项 7 的字段表：偏移 428、429、441，合计到 457）。
+const ROOT_RECORD_ALGORITHM_NONCE_AND_MAC: std::ops::Range<usize> = 428..457;
+
+/// I-2.4（头校验和覆盖范围） 射程里根记录那一处：根环里每一条读得出、自证过的根（`valid_roots`，不按候选集滤），根槽偏移 428..457
+/// （算法类型、nonce、MAC）全是 0。这一版 checker 只收加密类型 0 的系统配置，加密关着时它们恒 0、读者遇到非 0 判该结构损坏——
+/// 与单元头那 29 字节、指针头部 MAC / nonce 由同一条规则守（`.claude/kb/invariants.md` I-2.4 那一行；判决
+/// `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1「算法、nonce、MAC」那一行：单镜像性质，归池级 checker，
+/// 挂载透明不比）。实现今天不判这 29 字节（`crates/singlefs-core/src/root_record.rs` 的 `parse_slot` 跳过它们），checker 自己判一份。
+/// 只判不断：这条根照样进候选集、照样走。
+pub fn judge_root_record_algorithm_nonce_and_mac_are_zero(
+    judgements: &mut Judgements,
+    roots: &[(u64, u64, RootView)],
+) {
+    for (region, slot, root) in roots {
+        let reserved = &root.record_bytes[ROOT_RECORD_ALGORITHM_NONCE_AND_MAC];
+        judgements.judge("I-2.4", reserved.iter().all(|byte| *byte == 0), || {
+            let first_non_zero = reserved
+                .iter()
+                .position(|byte| *byte != 0)
+                .expect("判红时这 29 字节里有非 0 的");
+            format!(
+                "区域 {region} 槽 {slot} 的根（实例 {}、txg {}）：根槽偏移 {} 的字节是 {:#04x}，算法类型 / nonce / MAC 那 29 字节（428..457）加密关着时恒 0",
+                root.instance,
+                root.checkpoint_txg,
+                ROOT_RECORD_ALGORITHM_NONCE_AND_MAC.start + first_non_zero,
+                reserved[first_non_zero]
+            )
+        });
+    }
+}
+
 /// I-2.4（头校验和覆盖范围） 射程里指针那一半的另一格：走读跟随的一条指针，头部 extent 偏移（偏移 32 的 2 字节）为 0。
 /// 第一版指针指整个单元，读者遇到非 0 判该指针所在的结构损坏，与 MAC / nonce 同一处置、与加密开不开无关
 /// （D19（块指针的结构与宽度预算） 已定项 3，`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。只判不断。
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index bf5a31c..3b80b34 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -28,7 +28,8 @@ use crate::image::{
     chosen_system_configurations, decision_width_declared_by_the_chosen_system_configurations,
     judge_journal_ring_geometry_fields_agree, judge_location_entries_order, judge_location_order,
     judge_own_device_number_is_the_device_identity, judge_pointer_extent_offset_is_zero,
-    judge_pointer_mac_and_nonce_are_zero, judge_root_records_live_in_the_slot_their_txg_names,
+    judge_pointer_mac_and_nonce_are_zero, judge_root_record_algorithm_nonce_and_mac_are_zero,
+    judge_root_records_live_in_the_slot_their_txg_names,
     judge_root_slots_aligned_to_the_decision_width,
     judge_some_device_carries_a_self_verified_system_configuration,
     judge_system_configuration_slot_generations_follow_the_rotation,
@@ -3457,10 +3458,15 @@ impl HeadIdentifierOfATreeTableEntry {
     }
 }
 
-/// I-9.16（树表条目按树 ID 严格升序且合发号次序）：一版的树表判一次，两道任一道不成立即判红，第一处违例写明是哪一版、哪两条（下标从 0 数）。
+/// I-9.16（树表条目按树 ID 严格升序且合发号次序）：一版的树表判一次，四道任一道不成立即判红，第一处违例写明是哪一版、哪两条（下标从 0 数）
+/// 或哪一种。
 /// ① 盘上相邻两条的树 ID（条目打头 8 字节）严格升序；② 按发号次序（[`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`]），
-/// 树表里有的种类相邻两棵的树 ID 严格升序。0 条、1 条的树表两道都成立。checker 自己按字段表切一份，不与实现共用代码
-/// （D13（验证路线） 已定项 5）。
+/// 树表里有的种类相邻两棵的树 ID 严格升序；③ 同一张树表里每一种种类码（登记过的与没登记的都算）至多一条；④ 树表非空时
+/// 种类 1、2、3、4（extent 树、inode 树、分配记录树、记账树：读者重建一版要的有根的树）各恰好一条
+/// （判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y3「种类 1 不止一条；种类 3 不是恰好一条」那一行）。
+/// 0 条的树表四道都成立，1 条的 ① ② ③ 成立。checker 自己按字段表切一份，不与实现共用代码（D13（验证路线） 已定项 5；
+/// 实现那一侧是 `crates/singlefs-core/src/recovery.rs` 的 `tree_table_entries_each_kind_at_most_once` 与重建一版时缺了有根的树报的
+/// 「树表里缺一棵有根的树」）。③ ④ 先于 ② 判：同一张表两道都不成立时第一处违例写 ③ ④ 那一道。
 /// ① 与 I-1.1 在树表上判的「key 严格递增」是同一个谓词（树表的 key 就是树 ID）：只违反 ① 的镜像两条一起红，只违反 ② 的只红这一条。
 /// 条目宽不到种类那 2 字节（盘上字段，`index_node_view` 只保证 ≥ key 宽 8）时 ② 读不出种类：只判 ①，② 记成射程里判不了的那一部分。
 fn judge_tree_table_entries_ordering(
@@ -3495,7 +3501,8 @@ fn judge_tree_table_entries_ordering(
         judgements.judge("I-9.16", true, String::new);
         return;
     }
-    // (种类, 下标, 树 ID)，按发号次序；树表里没有的种类跳过。同一种有两条时取先出现的那条（核心层把它判成损坏，这里不另立判定）。
+    judge_tree_table_entry_kind_counts(judgements, tree_table_entries, version_name);
+    // (种类, 下标, 树 ID)，按发号次序；树表里没有的种类跳过。同一种有两条时取先出现的那条（那一张表 ③ 已判红）。
     let entries_in_the_issuing_order: Vec<(u16, usize, u64)> =
         TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER
             .iter()
@@ -3519,6 +3526,56 @@ fn judge_tree_table_entries_ordering(
     });
 }
 
+/// I-9.16 读者重建一版要的那四种有根的树（[`judge_tree_table_entry_kind_counts`] 的 ④）。
+const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 4] = [
+    TREE_KIND_EXTENT,
+    TREE_KIND_INODE,
+    TREE_KIND_ALLOCATION,
+    TREE_KIND_ACCOUNTING,
+];
+
+/// I-9.16 的 ③ ④（[`judge_tree_table_entries_ordering`]）：③ 每一种种类码至多一条；④ 树表非空时
+/// [`TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES`] 各恰好一条。条目都读得出种类那 2 字节（调用方判过）。
+/// 0 条的表不判（④ 的前提不成立，③ 没有对象）。第一处违例写明是哪一版、哪一种、几条。
+fn judge_tree_table_entry_kind_counts(
+    judgements: &mut Judgements,
+    tree_table_entries: &[Vec<u8>],
+    version_name: &str,
+) {
+    if tree_table_entries.is_empty() {
+        return;
+    }
+    let mut entries_of_each_kind: BTreeMap<u16, usize> = BTreeMap::new();
+    for entry in tree_table_entries {
+        *entries_of_each_kind
+            .entry(read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET))
+            .or_insert(0) += 1;
+    }
+    let first_kind_carried_more_than_once = entries_of_each_kind
+        .iter()
+        .find(|(_, entries)| **entries > 1)
+        .map(|(kind, entries)| (*kind, *entries));
+    judgements.judge(
+        "I-9.16",
+        first_kind_carried_more_than_once.is_none(),
+        || {
+            let (kind, entries) = first_kind_carried_more_than_once.expect("判红时找到过那一种");
+            format!("{version_name}的树表：种类 {kind} 有 {entries} 条（③ 每种至多一条）")
+        },
+    );
+    let first_rooted_kind_missing = TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES
+        .iter()
+        .copied()
+        .find(|kind| !entries_of_each_kind.contains_key(kind));
+    judgements.judge("I-9.16", first_rooted_kind_missing.is_none(), || {
+        let kind = first_rooted_kind_missing.expect("判红时找到过那一种");
+        format!(
+            "{version_name}的树表有 {} 条，种类 {kind} 一条都没有（④ 树表非空时种类 1、2、3、4 各恰好一条）",
+            tree_table_entries.len()
+        )
+    });
+}
+
 /// I-9.27（树表条目的头 ID 取值）：一版的树表判一次（树表被几版共享时只在第一个走到它的那一版判，同 I-9.16），每条条目按种类判一格
 /// （[`HeadIdentifierOfATreeTableEntry`]），第一处违例写明是哪一版、第几条（下标从 0 数）。树表 0 条的版本一格也不判。
 /// checker 自己按字段表切一份，不与实现共用代码（D13（验证路线） 已定项 5）。
@@ -4188,10 +4245,12 @@ struct VersionToJudgeTheFirstDataUnitOf<'record> {
 /// 只读、不判别的不变量：主走读按 `visited_units` 只走一遍共享子树，共享的 extent 树只算进第一条走到它的版本，这一条要的是
 /// 「这一版自己引用了哪些数据单元」（与 [`references_of_root`] 同一个缘故）。
 ///
-/// 下面几格 `.claude/kb/invariants.md` I-5.5 那一行没写，判不了，记成射程里判不了的那一部分（没有违例时整条报不适用、带理由），不替条款定：
-/// 树表读不出、条目宽不到字段表的 200 字节（认不出这一版是不是这一类）；种类 1 条目不止一条（① 读哪一条）；
-/// 认出来的那一版 extent 树读不全（数据单元数不全）；它有数据单元、而池里某块盘上一个都没有（那块盘上「最小的那一个」没有对象）；
-/// 种类 3（分配记录树）条目不是恰好一条、或那棵树读不全（落点算不出）。
+/// 下面几格这一版判不了，报部分没判（[`InvariantVerdict::PartiallyJudged`]，判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md`
+/// 第二节 Y3）：没有违例时整条报部分没判，逐版列出没判的那一版（实例、txg）与为什么，形态同 I-9.28 树表没读全的那几条根：
+/// 树表读不出、条目宽不到字段表的 200 字节（认不出这一版是不是这一类）；认出来的那一版 extent 树读不全（数据单元数不全）、
+/// 分配记录树读不全（落点算不出）；种类 1 条目不止一条（① 读哪一条）、种类 3（分配记录树）条目不是恰好一条（落点按哪一棵算）——
+/// 这两格的树表本身由 I-9.16 ③ ④ 判红，这里只记没判。
+/// 这一版有数据单元、而池里某块盘上一个都没有：那块盘上没有「最小的那一个」，该盘跳过、不记（多盘布局下合法；GPU 判器本来就跳过）。
 fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
     reader: &dyn ImageReader,
     versions: &[VersionToJudgeTheFirstDataUnitOf<'_>],
@@ -4201,6 +4260,8 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
     let invariant = FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK;
     let devices = reader.devices();
     let mut cache = IndexNodeCache::new();
+    // 没判的那几版：「（版本称呼，含实例与 txg）：为什么」，逐版一条，走完一起报（`Judgements::partially_judged` 只记第一次给的说明）。
+    let mut versions_not_judged: Vec<String> = Vec::new();
     for version in versions {
         let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
         if tree_table_pointer.all_zero {
@@ -4209,10 +4270,10 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
         let Some(tree_table) =
             read_index_node_without_judging(reader, &tree_table_pointer, &mut cache)
         else {
-            judgements.part_of_the_range_not_judged(
-                invariant,
-                "有一版的树表读不出：认不出它是不是从 mkfs 水位第一次建树的那一版",
-            );
+            versions_not_judged.push(format!(
+                "{}：树表读不出，认不出它是不是从 mkfs 水位第一次建树的那一版",
+                version.name
+            ));
             continue;
         };
         if tree_table.entries.is_empty() {
@@ -4223,10 +4284,10 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
             .iter()
             .any(|entry| entry.len() < tree_table_entry_bytes())
         {
-            judgements.part_of_the_range_not_judged(
-                invariant,
-                "有一版的树表条目宽不到字段表的 200 字节：认不出它是不是从 mkfs 水位第一次建树的那一版",
-            );
+            versions_not_judged.push(format!(
+                "{}：树表条目宽不到字段表的 200 字节，认不出它是不是从 mkfs 水位第一次建树的那一版",
+                version.name
+            ));
             continue;
         }
         let extent_entries: Vec<&Vec<u8>> = tree_table
@@ -4238,10 +4299,11 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
             [] => continue,
             [only_extent_entry] => *only_extent_entry,
             [_, _, ..] => {
-                judgements.part_of_the_range_not_judged(
-                    invariant,
-                    "有一版的树表里种类 1（extent 树）条目不止一条：认法 ① 读哪一条，I-5.5 那一行没写",
-                );
+                versions_not_judged.push(format!(
+                    "{}：树表里种类 1（extent 树）条目有 {} 条，认法 ① 读不出哪一条（由 I-9.16 ③ ④ 判）",
+                    version.name,
+                    extent_entries.len()
+                ));
                 continue;
             }
         };
@@ -4260,10 +4322,10 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
         let Some((_, data_pointers)) =
             extent_tree_without_judging(reader, &extent_root, &mut cache)
         else {
-            judgements.part_of_the_range_not_judged(
-                invariant,
-                "从 mkfs 水位第一次建树的那一版 extent 树读不全：它引用的数据单元数不全",
-            );
+            versions_not_judged.push(format!(
+                "{}（从 mkfs 水位第一次建树的那一版）：extent 树读不全，它引用的数据单元数不全",
+                version.name
+            ));
             continue;
         };
         let mut smallest_data_unit_slot_on_each_device: BTreeMap<u32, u64> = BTreeMap::new();
@@ -4286,10 +4348,11 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
             .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_ALLOCATION)
             .collect();
         let [allocation_entry] = allocation_entries.as_slice() else {
-            judgements.part_of_the_range_not_judged(
-                invariant,
-                "从 mkfs 水位第一次建树的那一版树表里种类 3（分配记录树）条目不是恰好一条：落点按哪一棵分配记录树算，I-5.5 那一行没写",
-            );
+            versions_not_judged.push(format!(
+                "{}（从 mkfs 水位第一次建树的那一版）：树表里种类 3（分配记录树）条目有 {} 条，落点按哪一棵算不出（由 I-9.16 ③ ④ 判）",
+                version.name,
+                allocation_entries.len()
+            ));
             continue;
         };
         let Some((_, allocation_records)) = allocation_record_tree_without_judging(
@@ -4297,39 +4360,43 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
             &parse_node_pointer(&allocation_entry[14..100]),
             &mut cache,
         ) else {
-            judgements.part_of_the_range_not_judged(
-                invariant,
-                "从 mkfs 水位第一次建树的那一版分配记录树读不全：落点算不出",
-            );
+            versions_not_judged.push(format!(
+                "{}（从 mkfs 水位第一次建树的那一版）：分配记录树读不全，落点算不出",
+                version.name
+            ));
             continue;
         };
         for device in &devices {
-            match smallest_data_unit_slot_on_each_device.get(device) {
-                Some(&smallest_slot) => {
-                    let expected_slot = first_data_unit_slot_the_allocation_records_leave_on_a_device(
-                        &allocation_records,
+            // 这一版在这块盘上一个数据单元都没有：没有「最小的那一个」，该盘跳过、不记。
+            if let Some(&smallest_slot) = smallest_data_unit_slot_on_each_device.get(device) {
+                let expected_slot = first_data_unit_slot_the_allocation_records_leave_on_a_device(
+                    &allocation_records,
+                    *device,
+                    version.checkpoint_txg,
+                    unit_area_start_slot,
+                    open_cluster_segment_named_by_the_tree_table_on_a_device(
+                        &tree_table_pointer,
                         *device,
-                        version.checkpoint_txg,
-                        unit_area_start_slot,
-                        open_cluster_segment_named_by_the_tree_table_on_a_device(
-                            &tree_table_pointer,
-                            *device,
-                        ),
-                    );
-                    judgements.judge(invariant, smallest_slot == expected_slot, || {
-                        format!(
-                            "{}（从 mkfs 水位第一次建树的那一版）：盘 {device} 上落点最小的数据单元在槽 {smallest_slot}，不是这一版的分配记录从单元区起始槽号 {unit_area_start_slot} 起算出的落点 {expected_slot}",
-                            version.name
-                        )
-                    });
-                }
-                None => judgements.part_of_the_range_not_judged(
-                    invariant,
-                    "从 mkfs 水位第一次建树的那一版有数据单元、而池里有一块盘上一个都没有：那块盘上判不判，I-5.5 那一行没写",
-                ),
+                    ),
+                );
+                judgements.judge(invariant, smallest_slot == expected_slot, || {
+                    format!(
+                        "{}（从 mkfs 水位第一次建树的那一版）：盘 {device} 上落点最小的数据单元在槽 {smallest_slot}，不是这一版的分配记录从单元区起始槽号 {unit_area_start_slot} 起算出的落点 {expected_slot}",
+                        version.name
+                    )
+                });
             }
         }
     }
+    if !versions_not_judged.is_empty() {
+        judgements.partially_judged(
+            invariant,
+            format!(
+                "判到的那几版落点都对得上；这几版没判：{}",
+                versions_not_judged.join("；")
+            ),
+        );
+    }
 }
 
 /// 一块盘上的开放段：起点是 64 的整数倍的 [`CLUSTER_SEGMENT_SLOTS`] 个槽（D3（空间分配） 已定项 10 ①）。
@@ -4760,6 +4827,12 @@ fn judge_release_generation_and_tree_table_birth(
 /// 几条根指着同一个分配记录树根时（照抄上一版的根）那棵树只判一次；每个 (树, 盘) 判一格，那块盘上只有一条记录也算判过。
 /// 树按位置寻址、可以多层（D8（核心索引结构） 已定项 14）：整棵读下来，同一块盘上跨叶的记录一起比。
 /// 走不到记录的根不判：树表或分配记录树读不出、位置对不上（由 I-2.1 / I-1.1 / I-7.2 说话）。
+///
+/// 同一条下另判一格「一条根只有一棵分配记录树」：树表非空（带树）的那一版，根记录里「树表 0 条那一版的分配记录树根」那 86 字节
+/// （偏移 342）必须全零——那一版的账住树表种类 3 那一条，根记录这一项只有树表 0 条、写过行的那一版写非零
+/// （D22（单元原子性怎么合成） 已定项 7 字段表「分配记录树根指针」那一行；判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md`
+/// 第二节 Y1 `allocation_record_tree_root` 那一行）。非零时这一版有两棵分配记录树，「这条根的分配记录树」不止一棵。
+/// 树表读不出的根这一格不判（认不出带不带树）。不变量清单里没有一条专管根记录这一项，暂挂在 I-5.4 下（归属交主 agent 定）。
 fn judge_allocation_records_disjoint(
     reader: &dyn ImageReader,
     roots: &[(u64, u64, crate::RootView)],
@@ -4781,6 +4854,20 @@ fn judge_allocation_records_disjoint(
             if let Some(tree_table) =
                 read_index_node_without_judging(reader, &tree_table_pointer, cache)
             {
+                if !tree_table.entries.is_empty() {
+                    judgements.judge(
+                        "I-5.4",
+                        allocation_roots[0].all_zero,
+                        || {
+                            format!(
+                                "根（实例 {}、txg {}）的树表有 {} 条（带树的一版），根记录里树表 0 条那一版的分配记录树根却不是全零：这一版有两棵分配记录树",
+                                root.instance,
+                                root.checkpoint_txg,
+                                tree_table.entries.len()
+                            )
+                        },
+                    );
+                }
                 allocation_roots.extend(
                     tree_table
                         .entries
@@ -7888,6 +7975,7 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
             .expect("8 字节"),
     );
     let roots = valid_roots(reader, &geometry);
+    judge_root_record_algorithm_nonce_and_mac_are_zero(&mut root_ring_judgements, &roots);
     judge_root_records_live_in_the_slot_their_txg_names(
         &roots,
         &geometry,
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index a8494c7..ebb84b0 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -1203,6 +1203,12 @@ pub struct PoolAllocator {
     /// 复用窗口置 0 时在 `release` 里当场回收掉的落点数（逐盘各算一条，与 `reclaim_released_up_to` 的返回值口径不同）：
     /// 开关走没走到，读这个数就看得出（`.claude/rules/fs-design.md` 五条硬要求第 4 条）。
     placements_reclaimed_on_release_by_the_forced_zero_reuse_window: u64,
+    /// 这个分配器实例上提交内生块走回落（`CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot`，D3（空间分配） 已定项 8 ②）的次数，
+    /// 只增、只住内存、`new` 与 `rebuild_from_records` 起步是 0。只读，产品路径不按它分支：它是回落那一支的观测
+    /// （`.claude/rules/fs-design.md` 五条硬要求第 4 条「分支必须可观测」）。harness 的看门用例在「mkfs 之后连续可写挂载、再写第一个文件」
+    /// 那一族上断言它是 0：I-5.5 的开放段认法（池级 checker 拿树表单元所在的段当开放段）假定这一族不走回落，哪天走进回落，用例红、
+    /// 回来重看那一格（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y3「开放段回落误排」那一行）。
+    commit_generated_placements_that_fell_back_to_the_lowest_free_slot: u64,
     /// 分配器眼里的根环（`RootRingOccupancy`）：挂载时装（`mount::rebuilt_allocator`），mkfs 同一个进程里由建分配器的一方装
     /// （`make_filesystem::root_ring_occupancy_after_make_filesystem`）。`None`：建分配器的一方没装（`new` 与 `rebuild_from_records`
     /// 都不装），挂载内回收与轮转清隔离位都不做，与只在挂载和抬 F 时回收的样子相同。
@@ -1238,6 +1244,7 @@ impl PoolAllocator {
             allocation_record_tree_of_the_version_without_file: None,
             reuse_window: ReuseWindow::GatedByTheRollbackFloor,
             placements_reclaimed_on_release_by_the_forced_zero_reuse_window: 0,
+            commit_generated_placements_that_fell_back_to_the_lowest_free_slot: 0,
             root_ring: None,
             frozen_publish: None,
             instance_rows_after_this_mounts_row_publish: 0,
@@ -1319,6 +1326,13 @@ impl PoolAllocator {
         self.placements_reclaimed_on_release_by_the_forced_zero_reuse_window
     }
 
+    /// 这个分配器实例上提交内生块走回落（`CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot`）的次数：每取一个回落的落点加一。
+    /// 可写挂载交回的分配器（`mount::Mounted::allocator`）带着挂载里写行、暖机、抬 F 与续做那几次发布数下的；重开之后从 0 数起。
+    #[must_use]
+    pub fn commit_generated_placements_that_fell_back_to_the_lowest_free_slot(&self) -> u64 {
+        self.commit_generated_placements_that_fell_back_to_the_lowest_free_slot
+    }
+
     /// 重开时从盘上那棵分配记录树重建分配器（D23（journal 的角色与格式） 已定项 14：defer 队列、分配器游标、记账的现行值从所选根那棵账重新载入）：
     /// 每条记录把自己那块盘上的槽标成占着，已释放的再进 defer 队列；空闲计数随 `mark_allocated` 减、与写路径同一条事件路径。
     /// 开放段与 bump 游标只在内存、盘上没有（分配记录树不记它们），重开后按「没有开放段」起步：下一个提交内生块从最低的全空段开——
@@ -1649,6 +1663,7 @@ impl PoolAllocator {
                 // 回落之后没有段可 bump 了，但刚才那一段仍是聚簇段、仍装着这次提交的内生块：它留在 `cluster_segments` 里，
                 // 用户数据照旧不许落进去（D3（空间分配） 已定项 8 第 2 条；增补 2 第 20c 行）。
                 self.open_segment = None;
+                self.commit_generated_placements_that_fell_back_to_the_lowest_free_slot += 1;
                 let placement = Placement {
                     slot,
                     span: footprint.slots(),
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index cc80b60..d354f0d 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -3224,6 +3224,68 @@ pub fn user_visible_tree_root_pointers(
     Ok(pointers)
 }
 
+/// 一条根的树表单元里每一条条目的盘上原样（每条是树表单元自述的条目宽那么多字节，D8（核心索引结构） 已定项 8 的字段表是 200），
+/// 按盘上次序；树表 0 条的一版交空表。只读、不判条目内容（种类登没登记、同一种几条都不判）：挂载透明的判官
+/// （`singlefs_harness::mount_transparency`）按字节区间比 R1 与 R2 的条目，它要的是原字节，不是解出来的几样。
+///
+/// # Errors
+/// 树表单元读不到（两份位置都读不出或校验和对不上）、解不开。
+pub fn tree_table_entry_bytes_of_root<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    root: &RootRecord,
+) -> Result<Vec<Vec<u8>>, RecoveryFailure> {
+    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
+    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
+    let tree_table =
+        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: "树表单元",
+        })?;
+    Ok(tree_table.entries)
+}
+
+/// 一版的根记录 [0, `ROOT_RECORD_BYTES`) 的字节：根环里有一槽自证过、解出来与 `root` 逐字段相同的，交那一槽的原字节
+/// （按根环槽的次序第一个）；没有的（由记录施加出来、根槽从没落盘的那一版）交 `root` 按字段表编出来的那几字节
+/// （[`RootRecord::to_slot`]：算法类型、nonce、MAC 那 29 字节与 flags 其余位写 0）。只读。
+/// 挂载透明的判官按字节区间比 R1 与 R2 的根记录用它（根记录里 `parse_slot` 不进内存的那几样——留位、flags 其余位——只有原字节里有）。
+#[must_use]
+pub fn root_record_bytes_of_the_version<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    region_devices: &[DeviceIdentity; 3],
+    immutable_sizes: &SystemImmutableSizes,
+    root: &RootRecord,
+) -> Vec<u8> {
+    let record_bytes = usize::try_from(ROOT_RECORD_BYTES).expect("457");
+    let root_slot_bytes = usize::try_from(immutable_sizes.physical_block_size).expect("根槽宽");
+    every_root_ring_slot(immutable_sizes)
+        .into_iter()
+        .find_map(|ring_slot| {
+            let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
+            let offset = slot_offset(ring_slot, immutable_sizes.fixed_structure_slot_spacing);
+            let bytes = reader.read(device, offset, root_slot_bytes)?;
+            (RootRecord::parse_slot(&bytes, &root.filesystem_identifier).as_ref() == Some(root))
+                .then(|| bytes[..record_bytes].to_vec())
+        })
+        .unwrap_or_else(|| root.to_slot(root_slot_bytes)[..record_bytes].to_vec())
+}
+
+/// 择到的系统配置（[`choose_system_configuration`]）那一槽的盘上原样（整槽 `SYSTEM_CONFIGURATION_SLOT_BYTES`）：在择到的那份自述的本盘
+/// （`immutable.this_device`）两个槽里找解出来与它相同的那一槽；择不到系统配置、或两槽都对不上时 None。只读。
+/// 挂载透明的判官按字节区间比 R1 与 R2 的系统配置用它（系统运行配置那 36 字节 `parse_slot` 不进内存，只有原字节里有）。
+#[must_use]
+pub fn chosen_system_configuration_slot_bytes(reader: &dyn PoolReader) -> Option<Vec<u8>> {
+    let chosen = choose_system_configuration(reader).ok()?;
+    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
+    let slot_spacing = u64::from(chosen.immutable.sizes.fixed_structure_slot_spacing);
+    (0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE).find_map(|slot_index| {
+        let bytes = reader.read(
+            chosen.immutable.this_device,
+            DeviceOffsetInBytes(slot_index * slot_spacing),
+            slot_bytes,
+        )?;
+        (SystemConfiguration::parse_slot(&bytes).as_ref() == Ok(&chosen)).then_some(bytes)
+    })
+}
+
 /// 根环全部自证过的根里最大的实例代号（取号的 max 里「根环里全部根记录的实例代号」那一半）；一条都没有时 None。
 #[must_use]
 pub fn highest_root_instance<Reader: PoolReader + ?Sized>(
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
index 2f4585e..493a42e 100644
--- a/crates/singlefs-harness/src/mount_transparency.rs
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -22,18 +22,27 @@ use singlefs_core::address::{
 use singlefs_core::instance_table::InstanceRow;
 use singlefs_core::journal::JournalRecordPlaceInPublish;
 use singlefs_core::mount::{MountError, MountOutput, RaisedFloor, StillUnreadableAfterOneReread};
-use singlefs_core::records::{TreeTableEntry, TREE_KIND_EXTENT, TREE_KIND_INODE};
+use singlefs_core::records::{
+    TreeTableEntry, TREE_KIND_DEADLIST, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT,
+    TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS,
+    TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
+};
 use singlefs_core::recovery::{
-    choose_root, choose_system_configuration, effective_rollback_floor, highest_root_instance,
-    highest_root_txg, instance_table_of_root, readable_roots, recover, replay_journal,
-    scan_journal, user_visible_tree_root_pointers, verified_system_configuration_slots,
-    walk_to_file, JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
-    UserVisibleTreeRootPointers,
+    choose_root, choose_system_configuration, chosen_system_configuration_slot_bytes,
+    effective_rollback_floor, highest_root_instance, highest_root_txg, instance_table_of_root,
+    readable_roots, recover, replay_journal, root_record_bytes_of_the_version, scan_journal,
+    tree_table_entry_bytes_of_root, verified_system_configuration_slots, walk_to_file,
+    JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
 };
 use singlefs_core::root_record::{RootRecord, UnmountMarker};
-use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableConfiguration};
+use singlefs_core::system_configuration::{
+    SystemConfiguration, SystemImmutableConfiguration, SystemRuntimeConfiguration,
+};
 use singlefs_core::transaction::TransactionOutput;
-use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
+use singlefs_format::{
+    SYSTEM_CONFIGURATION_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
+    SYSTEM_CONFIGURATION_SLOT_BYTES,
+};
 
 use crate::memory_pool::{root_identity_written_by, RetainedWrite};
 use crate::segments::StepKind;
@@ -76,15 +85,19 @@ pub enum ComparedVersion {
     TheVersionBeforeTheRecoveredOne,
 }
 
-/// 拿来比的那一版自己的字段：根记录、读出的内容、它指着的实例表的行、树表里两棵用户可见树的根指针。
+/// 拿来比的那一版自己的字段：根记录（解出来的与原字节）、读出的内容、它指着的实例表的行、树表条目的原字节。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct VersionFieldsRead {
     pub root: RootRecord,
+    /// 根记录 [0, `ROOT_RECORD_BYTES`) 的原字节（`recovery::root_record_bytes_of_the_version`：根环里那一槽的，
+    /// 由记录施加出来、根槽没落盘的那一版是按字段表编出来的）。[`ROOT_RECORD_FIELD_BYTE_RANGES`] 按它逐段比。
+    pub root_record_bytes: Vec<u8>,
     pub content: PosixContentRead,
     /// 实例表沿链有一片读不出、解不开时 None。
     pub instance_table_rows: Option<Vec<InstanceRow>>,
-    /// 树表读不出、解不开时 None。
-    pub user_visible_tree_root_pointers: Option<UserVisibleTreeRootPointers>,
+    /// 树表单元里每一条条目的原字节，按盘上次序（`recovery::tree_table_entry_bytes_of_root`）；树表读不出、解不开时 None。
+    /// [`TREE_TABLE_ENTRY_FIELD_BYTE_RANGES`] 按它逐条逐段比，树表条目的根指针那一样也从这里取。
+    pub tree_table_entries: Option<Vec<Vec<u8>>>,
 }
 
 /// 一个池上的一次只读恢复（看 journal）连同挂载透明要比的字段（[`read_recovery_fields`] 读出来的）。
@@ -108,6 +121,11 @@ pub struct RecoveryFieldsRead {
     pub highest_root_txg_in_the_ring: Option<CheckpointTxg>,
     /// 环里全部自证通过的 journal 记录的 checkpoint_txg 的最大值（txg 下界另一半）。
     pub highest_record_checkpoint_txg_in_the_ring: Option<CheckpointTxg>,
+    /// 环里全部自证通过的 journal 记录的计数器的最大值（系统配置 journal_tail 那一样的上界）；环里一条都没有时 None。
+    pub highest_record_counter_in_the_ring: Option<u64>,
+    /// 择到的系统配置那一槽 [0, `SYSTEM_CONFIGURATION_BYTES`) 的原字节（`recovery::chosen_system_configuration_slot_bytes`）；择不到时 None。
+    /// [`SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES`] 按它逐段比，journal_tail、journal 实例代号、系统运行配置三样也从这里取。
+    pub chosen_system_configuration_bytes: Option<Vec<u8>>,
 }
 
 /// 对 `reader` 跑一次只读恢复（看 journal），再读挂载透明要比的字段。`compared` 定拿哪一版来比（[`ComparedVersion`]）。
@@ -130,10 +148,16 @@ pub fn read_recovery_fields(
         highest_root_instance_in_the_ring: None,
         highest_root_txg_in_the_ring: None,
         highest_record_checkpoint_txg_in_the_ring: None,
+        highest_record_counter_in_the_ring: None,
+        chosen_system_configuration_bytes: None,
     };
     let Ok(system_configuration) = choose_system_configuration(reader) else {
         return fields;
     };
+    let system_configuration_bytes =
+        usize::try_from(SYSTEM_CONFIGURATION_BYTES).expect("系统配置字段表 489 字节装得进 usize");
+    fields.chosen_system_configuration_bytes = chosen_system_configuration_slot_bytes(reader)
+        .map(|slot| slot[..system_configuration_bytes].to_vec());
     let immutable = system_configuration.immutable;
     let slot_spacing = u64::from(immutable.sizes.fixed_structure_slot_spacing);
     fields.immutable_configuration = Some(immutable);
@@ -171,6 +195,7 @@ pub fn read_recovery_fields(
     let records = scan_journal(reader, &system_configuration);
     fields.highest_record_checkpoint_txg_in_the_ring =
         records.values().map(|record| record.checkpoint_txg).max();
+    fields.highest_record_counter_in_the_ring = records.values().map(|record| record.counter).max();
     let Some(chosen) = choose_root(reader, &system_configuration) else {
         return fields;
     };
@@ -191,6 +216,7 @@ pub fn read_recovery_fields(
     fields.compared_version = match compared {
         ComparedVersion::TheRecoveredVersion => Some(version_fields(
             reader,
+            &immutable,
             recovered_root,
             PosixContentRead::of_the_recovery_report(&fields.report),
         )),
@@ -213,7 +239,7 @@ pub fn read_recovery_fields(
                 Ok(None) => PosixContentRead::NoFile,
                 Err(_failure) => PosixContentRead::NothingRead,
             };
-            version_fields(reader, before, content)
+            version_fields(reader, &immutable, before, content)
         }),
     };
     fields
@@ -221,14 +247,21 @@ pub fn read_recovery_fields(
 
 fn version_fields(
     reader: &dyn PoolReader,
+    immutable: &SystemImmutableConfiguration,
     root: RootRecord,
     content: PosixContentRead,
 ) -> VersionFieldsRead {
     VersionFieldsRead {
         root,
+        root_record_bytes: root_record_bytes_of_the_version(
+            reader,
+            &immutable.region_devices,
+            &immutable.sizes,
+            &root,
+        ),
         content,
         instance_table_rows: instance_table_of_root(reader, &root).map(|table| table.rows),
-        user_visible_tree_root_pointers: user_visible_tree_root_pointers(reader, &root).ok(),
+        tree_table_entries: tree_table_entry_bytes_of_root(reader, &root).ok(),
     }
 }
 
@@ -525,15 +558,68 @@ impl FloorRaisesInTheMountWrites {
     }
 }
 
-/// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）每一步那一版树表里两棵用户可见树的根指针（用户 2026-10-03 定「续做算挂载的一部分」）：
-/// 挂载写的版本里，续做之前的那几版（写行、暖机、准入推的抬 F）照抄 R1 那一版的两棵树；续做的每一步改了它们，之后挂载写的版本
+/// 树表里只由挂载续做改根的那五种树（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1：墓碑、谱系、
+/// deleted_inodes 三根并进「树表里 inode 树、extent 树的根指针」那一行，同一条续做规则）：extent、inode、deleted_inodes、墓碑树、谱系树。
+pub const TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES: [u16; 5] = [
+    TREE_KIND_EXTENT,
+    TREE_KIND_INODE,
+    TREE_KIND_DELETED_INODES,
+    TREE_KIND_TOMBSTONE,
+    TREE_KIND_LINEAGE,
+];
+
+/// 树表里根恒全零、挂载前后必须相同的那四种树（同上判决 Y1「树表种类 6、7、8、12 的根」那一行）：livelist、稀疏旁表、deadlist、logged_ops。
+/// 种类 3、4（分配记录树、记账树）的根每一版都重写，不比，归「分配记录」那一行（池级 checker 判 R2 的镜像）。
+pub const TREE_KINDS_WHOSE_ROOTS_THE_MOUNT_KEEPS: [u16; 4] = [
+    TREE_KIND_LIVELIST,
+    TREE_KIND_SPARSE_SIDE_TABLE,
+    TREE_KIND_DEADLIST,
+    TREE_KIND_LOGGED_OPERATIONS,
+];
+
+/// 一版树表里按种类取的条目根指针（条目偏移 14 起 86 字节的盘上原样）；同一种有几条时取盘上先出现的那条（同一种两条由池级 checker 的
+/// I-9.16 ③ 判，这里不另判）。只装调用方点名的那几种。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct TreeTableEntryRootPointersByKind {
+    pub root_pointer_of_each_kind: BTreeMap<u16, Vec<u8>>,
+}
+
+impl TreeTableEntryRootPointersByKind {
+    /// 从树表条目的原字节取 `kinds` 那几种的根指针。条目宽不到种类那 2 字节或根指针那 86 字节的条目跳过（宽度由树表字段表那一段比）。
+    #[must_use]
+    pub fn of_the_entries(entries: &[Vec<u8>], kinds: &[u16]) -> Self {
+        let mut root_pointer_of_each_kind = BTreeMap::new();
+        for entry in entries {
+            let (Some(kind_bytes), Some(root_pointer)) = (
+                entry.get(TREE_TABLE_ENTRY_KIND.clone()),
+                entry.get(TREE_TABLE_ENTRY_ROOT_POINTER.clone()),
+            ) else {
+                continue;
+            };
+            let kind = u16::from_le_bytes(kind_bytes.try_into().expect("切的是种类那 2 字节"));
+            if kinds.contains(&kind) {
+                root_pointer_of_each_kind
+                    .entry(kind)
+                    .or_insert_with(|| root_pointer.to_vec());
+            }
+        }
+        TreeTableEntryRootPointersByKind {
+            root_pointer_of_each_kind,
+        }
+    }
+}
+
+/// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）每一步那一版树表里只由续做改根的那五种树
+/// （[`TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES`]）的根指针（用户 2026-10-03 定「续做算挂载的一部分」）：
+/// 挂载写的版本里，续做之前的那几版（写行、暖机、准入推的抬 F）照抄 R1 那一版的；续做的每一步改了它们，之后挂载写的版本
 /// （续做推的抬 F 的空发布）照抄最近一步的。挂载没续做时一步都没有，挂载写的每一版都比 R1 那一版。
+/// 名字里的「用户可见」是只装 inode 树、extent 树两种时起的：checker 档的崩溃注入（`crash_injection.rs`）照这个名字调它，改名随那一份。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct UserVisibleTreeRootPointersOfTheContinuation {
-    /// (那一步的实例代号与 checkpoint_txg, 两根指针)，按 txg 升序（构造时排）。
+    /// (那一步的实例代号与 checkpoint_txg, 五种树的根指针)，按 txg 升序（构造时排）。
     steps: Vec<(
         (InstanceGeneration, CheckpointTxg),
-        UserVisibleTreeRootPointers,
+        TreeTableEntryRootPointersByKind,
     )>,
 }
 
@@ -541,20 +627,20 @@ impl UserVisibleTreeRootPointersOfTheContinuation {
     /// 挂载没续做（`MountOutput::deletion_queue_continuation` 空）。
     pub const NOT_CONTINUED: Self = Self { steps: Vec::new() };
 
-    /// 两根指针取续做每一步交回的树表条目（`TransactionOutput::tree_table_entries`）里那 86 字节，与只读恢复读盘上树表的取法
-    /// （`recovery::user_visible_tree_root_pointers`）同一段字节。
+    /// 根指针取续做每一步交回的树表条目（`TransactionOutput::tree_table_entries`）编成盘上字节之后那 86 字节，与只读恢复读盘上树表条目
+    /// 的取法（[`TreeTableEntryRootPointersByKind::of_the_entries`]）同一段字节。
     #[must_use]
     pub fn of_the_mount_output(output: &MountOutput) -> Self {
         let mut steps: Vec<(
             (InstanceGeneration, CheckpointTxg),
-            UserVisibleTreeRootPointers,
+            TreeTableEntryRootPointersByKind,
         )> = output
             .deletion_queue_continuation
             .iter()
             .map(|step| {
                 (
                     (step.root.instance, step.root.checkpoint_txg),
-                    user_visible_tree_root_pointers_of_a_published_version(step),
+                    root_pointers_the_continuation_rewrites_in_a_published_version(step),
                 )
             })
             .collect();
@@ -562,12 +648,12 @@ impl UserVisibleTreeRootPointersOfTheContinuation {
         UserVisibleTreeRootPointersOfTheContinuation { steps }
     }
 
-    /// 挂载写的 `version` 那一版该带的两根指针：同一个实例里 txg 不大于它的最近一步续做那一版的；它在续做之前（或挂载没续做）时 None，
+    /// 挂载写的 `version` 那一版该带的五种树的根指针：同一个实例里 txg 不大于它的最近一步续做那一版的；它在续做之前（或挂载没续做）时 None，
     /// 调用方比 R1 那一版。
     fn of_the_last_step_at_or_before(
         &self,
         version: (InstanceGeneration, CheckpointTxg),
-    ) -> Option<&UserVisibleTreeRootPointers> {
+    ) -> Option<&TreeTableEntryRootPointersByKind> {
         self.steps
             .iter()
             .filter(|(step, _)| step.0 == version.0 && step.1 <= version.1)
@@ -576,20 +662,19 @@ impl UserVisibleTreeRootPointersOfTheContinuation {
     }
 }
 
-/// 一次发布交回的树表条目里 inode 树、extent 树两条的根指针盘上字节；树表里没有那棵树的条目时 None。
-fn user_visible_tree_root_pointers_of_a_published_version(
+/// 一次发布交回的树表条目里只由续做改根的那五种树的根指针盘上字节；树表里没有那种树的条目时那一种不在表里。
+fn root_pointers_the_continuation_rewrites_in_a_published_version(
     version: &TransactionOutput,
-) -> UserVisibleTreeRootPointers {
-    let mut pointers = UserVisibleTreeRootPointers::ABSENT;
-    for entry in &version.tree_table_entries {
-        let root_pointer = Some(TreeTableEntry::root_pointer_bytes(&entry.to_bytes()).to_vec());
-        if entry.kind == TREE_KIND_INODE {
-            pointers.inode_tree = root_pointer;
-        } else if entry.kind == TREE_KIND_EXTENT {
-            pointers.extent_tree = root_pointer;
-        }
-    }
-    pointers
+) -> TreeTableEntryRootPointersByKind {
+    let entries: Vec<Vec<u8>> = version
+        .tree_table_entries
+        .iter()
+        .map(TreeTableEntry::to_bytes)
+        .collect();
+    TreeTableEntryRootPointersByKind::of_the_entries(
+        &entries,
+        &TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES,
+    )
 }
 
 /// 做的是哪个动作。
@@ -603,7 +688,8 @@ pub enum MountTransparencyAction {
     SecondCrashInsideTheMount(FloorRaisesAtTheSecondCrash),
 }
 
-/// 比的十样（r5 判决第二节 U5 那一行的逐字段表）。
+/// 比的十八样：r5 判决第二节 U5 那一行的逐字段表十样，加判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md`
+/// 第二节 Y1、Y2 补的五样关系与三张按字节区间的字段表。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub enum MountTransparencyField {
     /// POSIX 内容与存在性。
@@ -616,11 +702,29 @@ pub enum MountTransparencyField {
     InstanceGeneration,
     InstanceTableRows,
     TreeIdentifierWatermark,
-    /// 树表里 inode 树、extent 树两个根指针。
-    UserVisibleTreeRootPointers,
+    /// 树表条目的根指针：种类 1、2、9、10、11（[`TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES`]）照续做规则，
+    /// 种类 6、7、8、12（[`TREE_KINDS_WHOSE_ROOTS_THE_MOUNT_KEEPS`]）与 R1 相同，种类 3、4 不比。
+    TreeTableEntryRootPointers,
     CheckpointTxg,
     /// 扫描报告（施加了几条、水位之上几条）。
     JournalScanReport,
+    /// 根记录里中央映射树根指针（偏移 256 起 86 字节）：挂载写的那一版 R2 全零 ⇔ R1 全零；落回 v1 时逐字节相同。
+    MappingRootEmptiness,
+    /// 根记录里树表 0 条那一版的分配记录树根（偏移 342 起 86 字节）：挂载写的那一版 R1 带树 ⇒ R2 全零，R1 树表 0 条 ⇒
+    /// R2 非零 ⇔ 这次挂载取到的新实例号 ≥ 2；落回 v1 时逐字节相同。
+    AllocationRecordTreeRoot,
+    /// 系统配置 journal_tail（偏移 469）：R1 的 ≤ R2 的 ≤ R2 环里自证记录的最大计数器，挂载做完（只挂载、挂载加一次发布）取等。
+    SystemConfigurationJournalTail,
+    /// 系统配置的 journal 实例代号（偏移 477）：挂载做完 = R2 那一版根的实例代号；二次崩溃 ∈ {R1 的, 新实例}。
+    SystemConfigurationJournalInstance,
+    /// 系统运行配置 36 字节（偏移 433 起）：R2 = 内置默认（今天没有挂载选项通道，写槽总写默认）。
+    SystemRuntimeConfiguration,
+    /// 根记录按字节区间（[`ROOT_RECORD_FIELD_BYTE_RANGES`]）：「必须相同」「必须为 0」的那几段。
+    RootRecordBytes,
+    /// 系统配置按字节区间（[`SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES`]）：「必须相同」「必须为 0」的那几段。
+    SystemConfigurationBytes,
+    /// 树表条目按字节区间（[`TREE_TABLE_ENTRY_FIELD_BYTE_RANGES`]），逐条：「必须相同」「必须为 0」的那几段。
+    TreeTableEntryBytes,
 }
 
 /// 一样字段的关系是攻方在副本装置上量过（主 agent 复跑坐实）的，还是 r5 判决标「推的」。
@@ -631,7 +735,7 @@ pub enum FieldBasis {
 }
 
 impl MountTransparencyField {
-    pub const ALL: [MountTransparencyField; 10] = [
+    pub const ALL: [MountTransparencyField; 18] = [
         MountTransparencyField::PosixContent,
         MountTransparencyField::ImmutableConfigurationWithoutThisDevice,
         MountTransparencyField::UnmountMarker,
@@ -639,13 +743,22 @@ impl MountTransparencyField {
         MountTransparencyField::InstanceGeneration,
         MountTransparencyField::InstanceTableRows,
         MountTransparencyField::TreeIdentifierWatermark,
-        MountTransparencyField::UserVisibleTreeRootPointers,
+        MountTransparencyField::TreeTableEntryRootPointers,
         MountTransparencyField::CheckpointTxg,
         MountTransparencyField::JournalScanReport,
+        MountTransparencyField::MappingRootEmptiness,
+        MountTransparencyField::AllocationRecordTreeRoot,
+        MountTransparencyField::SystemConfigurationJournalTail,
+        MountTransparencyField::SystemConfigurationJournalInstance,
+        MountTransparencyField::SystemRuntimeConfiguration,
+        MountTransparencyField::RootRecordBytes,
+        MountTransparencyField::SystemConfigurationBytes,
+        MountTransparencyField::TreeTableEntryBytes,
     ];
 
     /// 这一样在这个动作下的关系是量过的还是推的。二次崩溃那一截的字段一半整截是推的（r5 判决第二节 U6、攻方 U6-1），
     /// F 在挂载推了抬 F 时那一半（「= 挂载交回的最后一串抬到的 F」）r5 判决写的是「后半没量」。
+    /// c655 那一轮补的八样（五样关系、三张字节表）一律记推的：攻方量的是副本装置，判决第五节列它们为零轮，入库代码上还没量过。
     #[must_use]
     pub fn basis(self, action: MountTransparencyAction) -> FieldBasis {
         match (action, self) {
@@ -659,9 +772,17 @@ impl MountTransparencyField {
                 MountTransparencyAction::MountAlone(_)
                 | MountTransparencyAction::MountAndOnePublish(_),
                 MountTransparencyField::TreeIdentifierWatermark
-                | MountTransparencyField::UserVisibleTreeRootPointers
+                | MountTransparencyField::TreeTableEntryRootPointers
                 | MountTransparencyField::CheckpointTxg
-                | MountTransparencyField::JournalScanReport,
+                | MountTransparencyField::JournalScanReport
+                | MountTransparencyField::MappingRootEmptiness
+                | MountTransparencyField::AllocationRecordTreeRoot
+                | MountTransparencyField::SystemConfigurationJournalTail
+                | MountTransparencyField::SystemConfigurationJournalInstance
+                | MountTransparencyField::SystemRuntimeConfiguration
+                | MountTransparencyField::RootRecordBytes
+                | MountTransparencyField::SystemConfigurationBytes
+                | MountTransparencyField::TreeTableEntryBytes,
             ) => FieldBasis::InferredNotMeasured,
             (
                 MountTransparencyAction::MountAlone(_)
@@ -728,13 +849,13 @@ impl MountTransparencyField {
                 "挂载透明：树 ID 水位（推的）"
             }
             (
-                MountTransparencyField::UserVisibleTreeRootPointers,
+                MountTransparencyField::TreeTableEntryRootPointers,
                 FieldBasis::MeasuredByTheSecondRoundAttack,
-            ) => "挂载透明：两棵树的根指针（量过）",
+            ) => "挂载透明：树表条目的根指针（量过）",
             (
-                MountTransparencyField::UserVisibleTreeRootPointers,
+                MountTransparencyField::TreeTableEntryRootPointers,
                 FieldBasis::InferredNotMeasured,
-            ) => "挂载透明：两棵树的根指针（推的）",
+            ) => "挂载透明：树表条目的根指针（推的）",
             (MountTransparencyField::CheckpointTxg, FieldBasis::MeasuredByTheSecondRoundAttack) => {
                 "挂载透明：txg（量过）"
             }
@@ -748,10 +869,519 @@ impl MountTransparencyField {
             (MountTransparencyField::JournalScanReport, FieldBasis::InferredNotMeasured) => {
                 "挂载透明：扫描报告（推的）"
             }
+            (
+                MountTransparencyField::MappingRootEmptiness,
+                FieldBasis::MeasuredByTheSecondRoundAttack,
+            ) => "挂载透明：中央映射树根的空性（量过）",
+            (MountTransparencyField::MappingRootEmptiness, FieldBasis::InferredNotMeasured) => {
+                "挂载透明：中央映射树根的空性（推的）"
+            }
+            (
+                MountTransparencyField::AllocationRecordTreeRoot,
+                FieldBasis::MeasuredByTheSecondRoundAttack,
+            ) => "挂载透明：根记录里的分配记录树根（量过）",
+            (MountTransparencyField::AllocationRecordTreeRoot, FieldBasis::InferredNotMeasured) => {
+                "挂载透明：根记录里的分配记录树根（推的）"
+            }
+            (
+                MountTransparencyField::SystemConfigurationJournalTail,
+                FieldBasis::MeasuredByTheSecondRoundAttack,
+            ) => "挂载透明：系统配置 journal_tail（量过）",
+            (
+                MountTransparencyField::SystemConfigurationJournalTail,
+                FieldBasis::InferredNotMeasured,
+            ) => "挂载透明：系统配置 journal_tail（推的）",
+            (
+                MountTransparencyField::SystemConfigurationJournalInstance,
+                FieldBasis::MeasuredByTheSecondRoundAttack,
+            ) => "挂载透明：系统配置的 journal 实例代号（量过）",
+            (
+                MountTransparencyField::SystemConfigurationJournalInstance,
+                FieldBasis::InferredNotMeasured,
+            ) => "挂载透明：系统配置的 journal 实例代号（推的）",
+            (
+                MountTransparencyField::SystemRuntimeConfiguration,
+                FieldBasis::MeasuredByTheSecondRoundAttack,
+            ) => "挂载透明：系统运行配置（量过）",
+            (
+                MountTransparencyField::SystemRuntimeConfiguration,
+                FieldBasis::InferredNotMeasured,
+            ) => "挂载透明：系统运行配置（推的）",
+            (
+                MountTransparencyField::RootRecordBytes,
+                FieldBasis::MeasuredByTheSecondRoundAttack,
+            ) => "挂载透明：根记录按字节区间（量过）",
+            (MountTransparencyField::RootRecordBytes, FieldBasis::InferredNotMeasured) => {
+                "挂载透明：根记录按字节区间（推的）"
+            }
+            (
+                MountTransparencyField::SystemConfigurationBytes,
+                FieldBasis::MeasuredByTheSecondRoundAttack,
+            ) => "挂载透明：系统配置按字节区间（量过）",
+            (MountTransparencyField::SystemConfigurationBytes, FieldBasis::InferredNotMeasured) => {
+                "挂载透明：系统配置按字节区间（推的）"
+            }
+            (
+                MountTransparencyField::TreeTableEntryBytes,
+                FieldBasis::MeasuredByTheSecondRoundAttack,
+            ) => "挂载透明：树表条目按字节区间（量过）",
+            (MountTransparencyField::TreeTableEntryBytes, FieldBasis::InferredNotMeasured) => {
+                "挂载透明：树表条目按字节区间（推的）"
+            }
         }
     }
 }
 
+/// 一段字节区间挂载前后怎么比（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y2「字段清单从哪取」那一行：
+/// 每段带归类，判官按表比原字节）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum FieldByteRangeClass {
+    /// R2 的这几字节与 R1 的逐字节相同。
+    MustBeTheSame,
+    /// R2 的这几字节全是 0。
+    MustBeZero,
+    /// 不比：值随这次挂载合法地变，或读槽那一刻已经判过；`judged_instead_by` 写谁替它判。
+    NotCompared { judged_instead_by: &'static str },
+    /// 挂载前后有一条确定的关系，由 `judged_by` 那一样判（它从同一份原字节里取这几字节）。
+    DeterminedRelation { judged_by: MountTransparencyField },
+}
+
+/// 字段表里的一段：名字、字节区间、归类，与它引的 D13（验证路线） 已定项 1 挂载透明逐字段表那一行的行名
+/// （判决第三节第 3 条：那张表按 Y1、Y2 改行由写回批写；这里引的行名有几行是写回批才加的，见 [`D13_ROWS_ADDED_BY_THE_WRITE_BACK`]）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct FieldByteRange {
+    pub name: &'static str,
+    pub bytes: std::ops::Range<usize>,
+    pub class: FieldByteRangeClass,
+    pub row_of_the_d13_field_table: &'static str,
+}
+
+/// 系统配置字段表的四档（D22（单元原子性怎么合成） 已定项 9 / 已定项 26：系统不可变配置 389、系统可变配置 4、系统运行配置 36、系统运行量 60）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub enum SystemConfigurationTier {
+    ImmutableConfiguration,
+    MutableConfiguration,
+    RuntimeConfiguration,
+    RuntimeQuantities,
+}
+
+/// 系统配置字段表里的一段，连同它归哪一档。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct SystemConfigurationFieldByteRange {
+    pub range: FieldByteRange,
+    pub tier: SystemConfigurationTier,
+}
+
+/// D13（验证路线） 已定项 1 挂载透明逐字段表今天的行名（`.claude/kb/decisions/13-验证路线.md`）。
+const D13_IMMUTABLE_CONFIGURATION: &str = "池身份与系统配置的不可变配置（本盘设备号除外）";
+const D13_UNMOUNT_MARKER: &str = "卸载记号";
+const D13_ROLLBACK_FLOOR: &str = "F 与 F 生效值";
+const D13_INSTANCE_GENERATION: &str = "实例代号";
+const D13_INSTANCE_TABLE_ROWS: &str = "实例表行";
+const D13_CHECKPOINT_TXG: &str = "`checkpoint_txg`";
+const D13_TREE_IDENTIFIER_WATERMARK: &str = "树 ID 水位";
+const D13_TREE_ROOT_POINTERS: &str = "树表里 inode 树、extent 树的根指针";
+const D13_ALLOCATION_RECORDS: &str = "分配记录";
+/// 判决第二节 Y1、Y2 定下、写回批才加进 D13 逐字段表的几行。
+const D13_STRUCTURAL_FIELDS: &str = "magic、flags 其余位、自证校验和、整槽校验和";
+const D13_ROOT_SLOT_RESERVED_BYTES: &str = "算法、nonce、MAC（根槽偏移 428..457）";
+const D13_SYSTEM_CONFIGURATION_SLOT_GENERATION: &str = "系统配置槽世代号";
+const D13_SYSTEM_CONFIGURATION_JOURNAL_TAIL: &str = "journal_tail";
+const D13_SYSTEM_CONFIGURATION_JOURNAL_INSTANCE: &str = "系统配置的 journal 实例代号";
+const D13_MUTABLE_CONFIGURATION: &str = "系统可变配置（节点大小）";
+const D13_RUNTIME_CONFIGURATION: &str = "系统运行配置";
+const D13_TREE_TABLE_ENTRY_IDENTITY: &str = "树表条目的树 ID、种类、诞生 txg、头 ID";
+
+/// 这几张表引的 D13 行名里，今天的逐字段表有的那几行（[`D13_ROWS_ADDED_BY_THE_WRITE_BACK`] 之外的）。26-4 的对账门禁格按它与 kb 比。
+pub const D13_ROWS_OF_TODAY: [&str; 9] = [
+    D13_IMMUTABLE_CONFIGURATION,
+    D13_UNMOUNT_MARKER,
+    D13_ROLLBACK_FLOOR,
+    D13_INSTANCE_GENERATION,
+    D13_INSTANCE_TABLE_ROWS,
+    D13_CHECKPOINT_TXG,
+    D13_TREE_IDENTIFIER_WATERMARK,
+    D13_TREE_ROOT_POINTERS,
+    D13_ALLOCATION_RECORDS,
+];
+
+/// 这几张表引的 D13 行名里，写回批才加进逐字段表的那几行（判决第三节第 3 条）。
+pub const D13_ROWS_ADDED_BY_THE_WRITE_BACK: [&str; 8] = [
+    D13_STRUCTURAL_FIELDS,
+    D13_ROOT_SLOT_RESERVED_BYTES,
+    D13_SYSTEM_CONFIGURATION_SLOT_GENERATION,
+    D13_SYSTEM_CONFIGURATION_JOURNAL_TAIL,
+    D13_SYSTEM_CONFIGURATION_JOURNAL_INSTANCE,
+    D13_MUTABLE_CONFIGURATION,
+    D13_RUNTIME_CONFIGURATION,
+    D13_TREE_TABLE_ENTRY_IDENTITY,
+];
+
+/// 根记录里几样字段的字节区间（D22（单元原子性怎么合成） 已定项 7 的字段表：行序即盘上顺序，偏移按行累加）。
+const ROOT_RECORD_INSTANCE: std::ops::Range<usize> = 24..28;
+const ROOT_RECORD_MAPPING_ROOT: std::ops::Range<usize> = 256..342;
+const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT: std::ops::Range<usize> = 342..428;
+
+/// 系统配置里几样字段的字节区间（`.claude/kb/layout/01-first-txn.md`「系统配置字段表的偏移」）。
+const SYSTEM_CONFIGURATION_RUNTIME_CONFIGURATION: std::ops::Range<usize> = 433..469;
+const SYSTEM_CONFIGURATION_JOURNAL_TAIL: std::ops::Range<usize> = 469..477;
+const SYSTEM_CONFIGURATION_JOURNAL_INSTANCE: std::ops::Range<usize> = 477..481;
+
+/// 树表条目里种类与根指针的字节区间（D8（核心索引结构） 已定项 8：树 ID 8 + 条目长度 2 + 种类 2 + flags 2 + 根指针 86）。
+const TREE_TABLE_ENTRY_KIND: std::ops::Range<usize> = 10..12;
+const TREE_TABLE_ENTRY_ROOT_POINTER: std::ops::Range<usize> = 14..100;
+
+const fn must_be_the_same(
+    name: &'static str,
+    bytes: std::ops::Range<usize>,
+    row_of_the_d13_field_table: &'static str,
+) -> FieldByteRange {
+    FieldByteRange {
+        name,
+        bytes,
+        class: FieldByteRangeClass::MustBeTheSame,
+        row_of_the_d13_field_table,
+    }
+}
+
+const fn not_compared(
+    name: &'static str,
+    bytes: std::ops::Range<usize>,
+    judged_instead_by: &'static str,
+    row_of_the_d13_field_table: &'static str,
+) -> FieldByteRange {
+    FieldByteRange {
+        name,
+        bytes,
+        class: FieldByteRangeClass::NotCompared { judged_instead_by },
+        row_of_the_d13_field_table,
+    }
+}
+
+const fn determined_relation(
+    name: &'static str,
+    bytes: std::ops::Range<usize>,
+    judged_by: MountTransparencyField,
+    row_of_the_d13_field_table: &'static str,
+) -> FieldByteRange {
+    FieldByteRange {
+        name,
+        bytes,
+        class: FieldByteRangeClass::DeterminedRelation { judged_by },
+        row_of_the_d13_field_table,
+    }
+}
+
+const READ_WHEN_THE_SLOT_IS_READ: &str = "读槽时判（不对这一槽不可择）";
+
+/// 根记录 [0, `ROOT_RECORD_BYTES`) 按字段逐段（D22（单元原子性怎么合成） 已定项 7 的字段表 15 行；算法、nonce、MAC 合一段）。
+pub const ROOT_RECORD_FIELD_BYTE_RANGES: &[FieldByteRange] = &[
+    not_compared(
+        "magic",
+        0..4,
+        READ_WHEN_THE_SLOT_IS_READ,
+        D13_STRUCTURAL_FIELDS,
+    ),
+    must_be_the_same("fsid", 4..20, D13_IMMUTABLE_CONFIGURATION),
+    determined_relation(
+        "flags（位 0 是卸载记号；其余位非 0 读槽时拒收）",
+        20..24,
+        MountTransparencyField::UnmountMarker,
+        D13_UNMOUNT_MARKER,
+    ),
+    determined_relation(
+        "实例代号",
+        ROOT_RECORD_INSTANCE,
+        MountTransparencyField::InstanceGeneration,
+        D13_INSTANCE_GENERATION,
+    ),
+    determined_relation(
+        "checkpoint_txg",
+        28..36,
+        MountTransparencyField::CheckpointTxg,
+        D13_CHECKPOINT_TXG,
+    ),
+    not_compared(
+        "树表单元指针",
+        36..122,
+        "树表单元自己的落点每次发布都变、不比；它的条目按树表条目那张表逐段比",
+        D13_TREE_ROOT_POINTERS,
+    ),
+    determined_relation(
+        "树 ID 水位",
+        122..130,
+        MountTransparencyField::TreeIdentifierWatermark,
+        D13_TREE_IDENTIFIER_WATERMARK,
+    ),
+    determined_relation(
+        "回退下界 F",
+        130..138,
+        MountTransparencyField::RollbackFloor,
+        D13_ROLLBACK_FLOOR,
+    ),
+    not_compared(
+        "自证校验和",
+        138..170,
+        READ_WHEN_THE_SLOT_IS_READ,
+        D13_STRUCTURAL_FIELDS,
+    ),
+    not_compared(
+        "实例表单元指针",
+        170..256,
+        "实例表单元的落点不比；表里的行由「实例表行」那一样比",
+        D13_INSTANCE_TABLE_ROWS,
+    ),
+    determined_relation(
+        "中央映射树根指针",
+        ROOT_RECORD_MAPPING_ROOT,
+        MountTransparencyField::MappingRootEmptiness,
+        D13_ALLOCATION_RECORDS,
+    ),
+    determined_relation(
+        "分配记录树根指针",
+        ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT,
+        MountTransparencyField::AllocationRecordTreeRoot,
+        D13_ALLOCATION_RECORDS,
+    ),
+    not_compared(
+        "算法类型、nonce、MAC",
+        428..457,
+        "池级 checker 的 I-2.4 判全 0（单镜像性质）",
+        D13_ROOT_SLOT_RESERVED_BYTES,
+    ),
+];
+
+const fn system_configuration_field(
+    range: FieldByteRange,
+    tier: SystemConfigurationTier,
+) -> SystemConfigurationFieldByteRange {
+    SystemConfigurationFieldByteRange { range, tier }
+}
+
+const fn immutable_and_the_same(
+    name: &'static str,
+    bytes: std::ops::Range<usize>,
+) -> SystemConfigurationFieldByteRange {
+    system_configuration_field(
+        must_be_the_same(name, bytes, D13_IMMUTABLE_CONFIGURATION),
+        SystemConfigurationTier::ImmutableConfiguration,
+    )
+}
+
+/// 系统配置 [0, `SYSTEM_CONFIGURATION_BYTES`) 按字段逐段（`.claude/kb/layout/01-first-txn.md`「系统配置字段表」46 行，一行一段）。
+pub const SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES: &[SystemConfigurationFieldByteRange] = &[
+    system_configuration_field(
+        not_compared(
+            "magic",
+            0..4,
+            READ_WHEN_THE_SLOT_IS_READ,
+            D13_STRUCTURAL_FIELDS,
+        ),
+        SystemConfigurationTier::ImmutableConfiguration,
+    ),
+    immutable_and_the_same("格式版本", 4..6),
+    immutable_and_the_same("feature bits", 6..102),
+    immutable_and_the_same("fsid", 102..118),
+    immutable_and_the_same("写入者身份", 118..138),
+    immutable_and_the_same("校验和算法标识", 138..139),
+    system_configuration_field(
+        not_compared(
+            "本盘设备号",
+            139..143,
+            "择到哪块盘的那一槽就是哪块盘的号；池级 checker 的 I-7.14 判它等于盘在池里的身份",
+            D13_IMMUTABLE_CONFIGURATION,
+        ),
+        SystemConfigurationTier::ImmutableConfiguration,
+    ),
+    immutable_and_the_same("设备数 devs", 143..147),
+    system_configuration_field(
+        not_compared(
+            "槽世代号",
+            147..155,
+            "池级 checker 的 I-7.16 在 R2 的镜像上判",
+            D13_SYSTEM_CONFIGURATION_SLOT_GENERATION,
+        ),
+        SystemConfigurationTier::RuntimeQuantities,
+    ),
+    system_configuration_field(
+        not_compared(
+            "整槽校验和",
+            155..187,
+            READ_WHEN_THE_SLOT_IS_READ,
+            D13_STRUCTURAL_FIELDS,
+        ),
+        SystemConfigurationTier::RuntimeQuantities,
+    ),
+    immutable_and_the_same("系统配置自身 MAC", 187..203),
+    immutable_and_the_same("nonce 水位", 203..215),
+    immutable_and_the_same("KDF 标识", 215..219),
+    immutable_and_the_same("加密类型", 219..220),
+    immutable_and_the_same("MAC 长度声明", 220..221),
+    immutable_and_the_same("主密钥槽", 221..301),
+    system_configuration_field(
+        must_be_the_same("节点大小", 301..305, D13_MUTABLE_CONFIGURATION),
+        SystemConfigurationTier::MutableConfiguration,
+    ),
+    immutable_and_the_same("单元大小", 305..309),
+    immutable_and_the_same("落点粒度", 309..313),
+    immutable_and_the_same("位置条目宽度", 313..317),
+    immutable_and_the_same("mkfs 时的 physical_block_size", 317..321),
+    immutable_and_the_same("扩展点声明值 N", 321..325),
+    immutable_and_the_same("journal 环起点", 325..333),
+    immutable_and_the_same("journal 环长度", 333..341),
+    immutable_and_the_same("journal 最大记录字节", 341..345),
+    immutable_and_the_same("journal 在飞记录数上限", 345..349),
+    immutable_and_the_same("journal 最坏占用", 349..357),
+    immutable_and_the_same("journal 安全系数 F", 357..361),
+    immutable_and_the_same("根环区域数 R", 361..362),
+    immutable_and_the_same("每区槽数 S", 362..363),
+    immutable_and_the_same("素数步长 P", 363..367),
+    immutable_and_the_same("根环 chunk", 367..371),
+    immutable_and_the_same("根环起点", 371..379),
+    immutable_and_the_same("根环逐区域设备身份", 379..391),
+    immutable_and_the_same("条带宽度上界 w_max", 391..392),
+    immutable_and_the_same("组大小 g", 392..393),
+    immutable_and_the_same("映射来源", 393..417),
+    immutable_and_the_same("单元区起始槽号", 417..425),
+    immutable_and_the_same("mkfs 时的 io_min", 425..429),
+    immutable_and_the_same("固定结构槽距", 429..433),
+    system_configuration_field(
+        determined_relation(
+            "T_time",
+            433..437,
+            MountTransparencyField::SystemRuntimeConfiguration,
+            D13_RUNTIME_CONFIGURATION,
+        ),
+        SystemConfigurationTier::RuntimeConfiguration,
+    ),
+    system_configuration_field(
+        determined_relation(
+            "T_dirty",
+            437..445,
+            MountTransparencyField::SystemRuntimeConfiguration,
+            D13_RUNTIME_CONFIGURATION,
+        ),
+        SystemConfigurationTier::RuntimeConfiguration,
+    ),
+    system_configuration_field(
+        determined_relation(
+            "整理低水位 / 高水位 / 停止线",
+            445..469,
+            MountTransparencyField::SystemRuntimeConfiguration,
+            D13_RUNTIME_CONFIGURATION,
+        ),
+        SystemConfigurationTier::RuntimeConfiguration,
+    ),
+    system_configuration_field(
+        determined_relation(
+            "journal tail",
+            SYSTEM_CONFIGURATION_JOURNAL_TAIL,
+            MountTransparencyField::SystemConfigurationJournalTail,
+            D13_SYSTEM_CONFIGURATION_JOURNAL_TAIL,
+        ),
+        SystemConfigurationTier::RuntimeQuantities,
+    ),
+    system_configuration_field(
+        determined_relation(
+            "journal 实例代号",
+            SYSTEM_CONFIGURATION_JOURNAL_INSTANCE,
+            MountTransparencyField::SystemConfigurationJournalInstance,
+            D13_SYSTEM_CONFIGURATION_JOURNAL_INSTANCE,
+        ),
+        SystemConfigurationTier::RuntimeQuantities,
+    ),
+    system_configuration_field(
+        not_compared(
+            "回退下界 F",
+            481..489,
+            "写低由池级 checker 的 I-7.12 判；F 生效值由「F 与 F 生效值」那一样比",
+            D13_ROLLBACK_FLOOR,
+        ),
+        SystemConfigurationTier::RuntimeQuantities,
+    ),
+];
+
+/// 树表条目 [0, 200) 按字段逐段（D8（核心索引结构） 已定项 8 的字段表：树 ID 8 + 条目长度 2 + 种类 2 + flags 2 + 根指针 86 +
+/// `previous_snapshot_txg` 8 + 诞生 txg 8 + 头 ID 8 + 预留 76）。条目按盘上下标一一对上比（两版条数不同判红）。
+pub const TREE_TABLE_ENTRY_FIELD_BYTE_RANGES: &[FieldByteRange] = &[
+    must_be_the_same("树 ID", 0..8, D13_TREE_TABLE_ENTRY_IDENTITY),
+    must_be_the_same("条目长度", 8..10, D13_TREE_TABLE_ENTRY_IDENTITY),
+    must_be_the_same("种类", TREE_TABLE_ENTRY_KIND, D13_TREE_TABLE_ENTRY_IDENTITY),
+    must_be_the_same("flags", 12..14, D13_TREE_TABLE_ENTRY_IDENTITY),
+    determined_relation(
+        "根指针",
+        TREE_TABLE_ENTRY_ROOT_POINTER,
+        MountTransparencyField::TreeTableEntryRootPointers,
+        D13_TREE_ROOT_POINTERS,
+    ),
+    must_be_the_same(
+        "previous_snapshot_txg",
+        100..108,
+        D13_TREE_TABLE_ENTRY_IDENTITY,
+    ),
+    must_be_the_same("诞生 txg", 108..116, D13_TREE_TABLE_ENTRY_IDENTITY),
+    must_be_the_same("头 ID", 116..124, D13_TREE_TABLE_ENTRY_IDENTITY),
+    FieldByteRange {
+        name: "预留",
+        bytes: 124..200,
+        class: FieldByteRangeClass::MustBeZero,
+        row_of_the_d13_field_table: D13_TREE_TABLE_ENTRY_IDENTITY,
+    },
+];
+
+/// 三张字节表按字节比的那几段（「必须相同」「必须为 0」）里，`after` 对不上的段名。`before` / `after` 短于那一段的末尾也算对不上。
+fn byte_ranges_that_do_not_hold<'table>(
+    table: impl IntoIterator<Item = &'table FieldByteRange>,
+    before: &[u8],
+    after: &[u8],
+) -> Vec<&'static str> {
+    table
+        .into_iter()
+        .filter(|range| match range.class {
+            FieldByteRangeClass::MustBeTheSame => {
+                before.get(range.bytes.clone()) != after.get(range.bytes.clone())
+                    || after.get(range.bytes.clone()).is_none()
+            }
+            FieldByteRangeClass::MustBeZero => !after
+                .get(range.bytes.clone())
+                .is_some_and(|bytes| bytes.iter().all(|byte| *byte == 0)),
+            FieldByteRangeClass::NotCompared { .. }
+            | FieldByteRangeClass::DeterminedRelation { .. } => false,
+        })
+        .map(|range| range.name)
+        .collect()
+}
+
+/// 系统运行配置那 36 字节的内置默认值：照写槽的同一个次序（T_time 4、T_dirty 8、整理三水位 8 × 3，小端），值取
+/// `SystemRuntimeConfiguration` 交的默认值（今天没有挂载选项通道，写槽总写它们，`system_configuration.rs` 那一档的说明）。
+#[must_use]
+pub fn built_in_runtime_configuration_bytes() -> Vec<u8> {
+    let defaults = SystemRuntimeConfiguration;
+    let mut bytes = Vec::with_capacity(SYSTEM_CONFIGURATION_RUNTIME_CONFIGURATION.len());
+    bytes.extend_from_slice(&defaults.time_threshold_seconds().to_le_bytes());
+    bytes.extend_from_slice(&defaults.dirty_threshold_bytes().to_le_bytes());
+    for watermark in defaults.compaction_watermarks() {
+        bytes.extend_from_slice(&watermark.to_le_bytes());
+    }
+    bytes
+}
+
+fn read_u32_at(bytes: &[u8], range: std::ops::Range<usize>) -> Option<u32> {
+    bytes
+        .get(range)
+        .map(|field| u32::from_le_bytes(field.try_into().expect("切的是 4 字节")))
+}
+
+fn read_u64_at(bytes: &[u8], range: std::ops::Range<usize>) -> Option<u64> {
+    bytes
+        .get(range)
+        .map(|field| u64::from_le_bytes(field.try_into().expect("切的是 8 字节")))
+}
+
+fn is_all_zero(bytes: &[u8]) -> bool {
+    bytes.iter().all(|byte| *byte == 0)
+}
+
 /// 某一样为什么没判：条款没写这一格该是什么，不替它定红绿，只计数（交主 agent）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub enum FieldNotJudged {
@@ -962,11 +1592,17 @@ fn rows_sorted_by_instance(rows: &[InstanceRow]) -> Vec<InstanceRow> {
 /// 动作做成之后的那一版：POSIX 内容与存在性相同；不可变配置（除本盘设备号）相同；卸载记号恒为「不是卸载写的」、R1 的记号不进比较；
 /// 挂载没推抬 F 时 R2 的 F 生效值与 R2 根里的 F 都 = R1 的 F 生效值，推了时 R2 的 F 生效值 = 挂载交回的最后一串抬到的 F；
 /// 实例代号 = max(R1 镜像上各盘两槽自证系统配置的实例代号, 根环全部可读根的实例代号) + 1；实例表行照 D18 已定项 11「恢复写的行」；
-/// 树 ID 水位相同（挂载不发第一个文件）；两棵用户可见树的根指针相同——挂载续做过时（用户 2026-10-03 定「续做算挂载的一部分」）
-/// 比挂载交回的续做那几步里 txg 不大于 R2 那一版的最近一步的（`continuation`，[`UserVisibleTreeRootPointersOfTheContinuation`]）；
+/// 树 ID 水位相同（挂载不发第一个文件）；树表里 extent、inode、deleted_inodes、墓碑、谱系五种树的根指针相同——挂载续做过时
+/// （用户 2026-10-03 定「续做算挂载的一部分」）比挂载交回的续做那几步里 txg 不大于 R2 那一版的最近一步的
+/// （`continuation`，[`UserVisibleTreeRootPointersOfTheContinuation`]），livelist、稀疏旁表、deadlist、logged_ops 四种的根相同；
 /// txg ≥ max(根环全部根的 txg, 环里全部自证记录的 checkpoint_txg) + 1（只判下界：kb 允许跳号）；扫描报告施加 0 条、水位之上 0 条。
 /// 二次崩溃之后落回 v1 那一版自己（实例与 txg 都是 R1 那一版的）：各样与 R1 那一版相同（推的）；落到这次挂载写的某一版：照上一段，
 /// 扫描报告不判，崩在抬 F 开始之后的 F 不判。
+/// 判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1、Y2 补的几样（[`MountTransparencyField`] 的后八个成员）：
+/// 中央映射树根的空性、根记录里的分配记录树根、系统配置 journal_tail 与 journal 实例代号、系统运行配置 = 内置默认，
+/// 再加根记录、系统配置、树表条目三张按字节区间的字段表里「必须相同」「必须为 0」的那几段；这几样都从 [`read_recovery_fields`]
+/// 读进来的原字节里取（[`VersionFieldsRead::root_record_bytes`]、[`VersionFieldsRead::tree_table_entries`]、
+/// [`RecoveryFieldsRead::chosen_system_configuration_bytes`]），签名不变。
 ///
 /// # Panics
 /// 调用方交错了比的版本：R1 不是比恢复落到的那一版，或 R2 拿的那一版与动作对不上。
@@ -1199,23 +1835,8 @@ fn judge_one_field(
             first_recovery.root.tree_identifier_watermark.to_string(),
             compared.root.tree_identifier_watermark.to_string(),
         ),
-        MountTransparencyField::UserVisibleTreeRootPointers => {
-            let expected = match relation {
-                Relation::WrittenByTheMount(_) => continuation
-                    .of_the_last_step_at_or_before((
-                        compared.root.instance,
-                        compared.root.checkpoint_txg,
-                    ))
-                    .or(first_recovery.user_visible_tree_root_pointers.as_ref()),
-                Relation::TheVersionOfTheFirstRecovery(_) => {
-                    first_recovery.user_visible_tree_root_pointers.as_ref()
-                }
-            };
-            held_or_violated(
-                expected.is_some() && compared.user_visible_tree_root_pointers.as_ref() == expected,
-                format!("{expected:?}"),
-                format!("{:?}", compared.user_visible_tree_root_pointers),
-            )
+        MountTransparencyField::TreeTableEntryRootPointers => {
+            tree_table_entry_root_pointers_outcome(first_recovery, compared, relation, continuation)
         }
         MountTransparencyField::CheckpointTxg => match relation {
             Relation::WrittenByTheMount(_) => {
@@ -1257,9 +1878,326 @@ fn judge_one_field(
                 )
             }
         },
+        MountTransparencyField::MappingRootEmptiness => {
+            let before = &first_recovery.root_record_bytes[ROOT_RECORD_MAPPING_ROOT];
+            let after = &compared.root_record_bytes[ROOT_RECORD_MAPPING_ROOT];
+            match relation {
+                Relation::WrittenByTheMount(_) => held_or_violated(
+                    is_all_zero(after) == is_all_zero(before),
+                    format!("R2 全零 ⇔ R1 全零（R1 全零：{}）", is_all_zero(before)),
+                    format!("R2 全零：{}", is_all_zero(after)),
+                ),
+                Relation::TheVersionOfTheFirstRecovery(_) => held_or_violated(
+                    after == before,
+                    format!("与 R1 逐字节相同：{before:?}"),
+                    format!("{after:?}"),
+                ),
+            }
+        }
+        MountTransparencyField::AllocationRecordTreeRoot => {
+            allocation_record_tree_root_outcome(first_recovery, compared, relation, new_instance)
+        }
+        MountTransparencyField::SystemConfigurationJournalTail => {
+            system_configuration_journal_tail_outcome(before_the_mount, after_the_action, action)
+        }
+        MountTransparencyField::SystemConfigurationJournalInstance => {
+            system_configuration_journal_instance_outcome(
+                before_the_mount,
+                after_the_action,
+                compared,
+                new_instance,
+                action,
+            )
+        }
+        MountTransparencyField::SystemRuntimeConfiguration => {
+            let built_in = built_in_runtime_configuration_bytes();
+            let observed = after_the_action
+                .chosen_system_configuration_bytes
+                .as_deref()
+                .and_then(|bytes| bytes.get(SYSTEM_CONFIGURATION_RUNTIME_CONFIGURATION));
+            held_or_violated(
+                observed == Some(built_in.as_slice()),
+                format!("内置默认 {built_in:?}"),
+                format!("{observed:?}"),
+            )
+        }
+        MountTransparencyField::RootRecordBytes => {
+            let ranges_that_do_not_hold = byte_ranges_that_do_not_hold(
+                ROOT_RECORD_FIELD_BYTE_RANGES,
+                &first_recovery.root_record_bytes,
+                &compared.root_record_bytes,
+            );
+            held_or_violated(
+                ranges_that_do_not_hold.is_empty(),
+                "根记录里「必须相同」「必须为 0」的那几段都对得上".to_string(),
+                format!("这几段不符：{ranges_that_do_not_hold:?}"),
+            )
+        }
+        MountTransparencyField::SystemConfigurationBytes => match (
+            &before_the_mount.chosen_system_configuration_bytes,
+            &after_the_action.chosen_system_configuration_bytes,
+        ) {
+            (Some(before), Some(after)) => {
+                let ranges_that_do_not_hold = byte_ranges_that_do_not_hold(
+                    SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES
+                        .iter()
+                        .map(|system_configuration_field| &system_configuration_field.range),
+                    before,
+                    after,
+                );
+                held_or_violated(
+                    ranges_that_do_not_hold.is_empty(),
+                    "系统配置里「必须相同」「必须为 0」的那几段都对得上".to_string(),
+                    format!("这几段不符：{ranges_that_do_not_hold:?}"),
+                )
+            }
+            (before, after) => FieldOutcome::Violated {
+                expected: "R1、R2 都择得到系统配置".to_string(),
+                observed: format!(
+                    "R1 择到：{}；R2 择到：{}",
+                    before.is_some(),
+                    after.is_some()
+                ),
+            },
+        },
+        MountTransparencyField::TreeTableEntryBytes => {
+            tree_table_entry_bytes_outcome(first_recovery, compared)
+        }
     }
 }
 
+/// 树表条目的根指针那一样（[`MountTransparencyField::TreeTableEntryRootPointers`]）：只由续做改根的五种照续做规则
+/// （挂载写的那一版比续做里 txg 不大于它的最近一步，没有那一步比 R1），根恒全零的四种与 R1 相同，种类 3、4 不比。
+/// 两版任一版树表读不出判红。
+fn tree_table_entry_root_pointers_outcome(
+    first_recovery: &VersionFieldsRead,
+    compared: &VersionFieldsRead,
+    relation: Relation,
+    continuation: &UserVisibleTreeRootPointersOfTheContinuation,
+) -> FieldOutcome {
+    let (Some(entries_before), Some(entries_after)) = (
+        &first_recovery.tree_table_entries,
+        &compared.tree_table_entries,
+    ) else {
+        return FieldOutcome::Violated {
+            expected: "R1、R2 两版的树表都读得出".to_string(),
+            observed: format!(
+                "R1 读得出：{}；R2 读得出：{}",
+                first_recovery.tree_table_entries.is_some(),
+                compared.tree_table_entries.is_some()
+            ),
+        };
+    };
+    let rewritten_only_by_the_continuation_before =
+        TreeTableEntryRootPointersByKind::of_the_entries(
+            entries_before,
+            &TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES,
+        );
+    let expected_rewritten_only_by_the_continuation = match relation {
+        Relation::WrittenByTheMount(_) => continuation
+            .of_the_last_step_at_or_before((compared.root.instance, compared.root.checkpoint_txg))
+            .unwrap_or(&rewritten_only_by_the_continuation_before),
+        Relation::TheVersionOfTheFirstRecovery(_) => &rewritten_only_by_the_continuation_before,
+    };
+    let observed_rewritten_only_by_the_continuation =
+        TreeTableEntryRootPointersByKind::of_the_entries(
+            entries_after,
+            &TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES,
+        );
+    let kept_before = TreeTableEntryRootPointersByKind::of_the_entries(
+        entries_before,
+        &TREE_KINDS_WHOSE_ROOTS_THE_MOUNT_KEEPS,
+    );
+    let kept_after = TreeTableEntryRootPointersByKind::of_the_entries(
+        entries_after,
+        &TREE_KINDS_WHOSE_ROOTS_THE_MOUNT_KEEPS,
+    );
+    held_or_violated(
+        observed_rewritten_only_by_the_continuation == *expected_rewritten_only_by_the_continuation
+            && kept_after == kept_before,
+        format!(
+            "续做改根的五种 {expected_rewritten_only_by_the_continuation:?}；根恒全零的四种 {kept_before:?}"
+        ),
+        format!(
+            "续做改根的五种 {observed_rewritten_only_by_the_continuation:?}；根恒全零的四种 {kept_after:?}"
+        ),
+    )
+}
+
+/// 根记录里的分配记录树根那一样（[`MountTransparencyField::AllocationRecordTreeRoot`]）。R1 树表读不出（认不出带不带树）、
+/// 或 R1 树表 0 条而新实例号算不出时判红。
+fn allocation_record_tree_root_outcome(
+    first_recovery: &VersionFieldsRead,
+    compared: &VersionFieldsRead,
+    relation: Relation,
+    new_instance: Option<InstanceGeneration>,
+) -> FieldOutcome {
+    let before = &first_recovery.root_record_bytes[ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT];
+    let after = &compared.root_record_bytes[ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT];
+    match relation {
+        Relation::TheVersionOfTheFirstRecovery(_) => held_or_violated(
+            after == before,
+            format!("与 R1 逐字节相同：{before:?}"),
+            format!("{after:?}"),
+        ),
+        Relation::WrittenByTheMount(_) => {
+            match (first_recovery.tree_table_entries.as_deref(), new_instance) {
+                (Some([_, ..]), _) => held_or_violated(
+                    is_all_zero(after),
+                    "R1 带树 ⇒ R2 全零".to_string(),
+                    format!("R2 全零：{}", is_all_zero(after)),
+                ),
+                (Some([]), Some(new_instance)) => {
+                    let expected_non_zero = new_instance.0 >= 2;
+                    held_or_violated(
+                        !is_all_zero(after) == expected_non_zero,
+                        format!(
+                            "R1 树表 0 条 ⇒ R2 非零 ⇔ 新实例号 {} ≥ 2（该非零：{expected_non_zero}）",
+                            new_instance.0
+                        ),
+                        format!("R2 非零：{}", !is_all_zero(after)),
+                    )
+                }
+                (Some([]), None) => FieldOutcome::Violated {
+                    expected: "R1 树表 0 条：要新实例号定 R2 该不该非零".to_string(),
+                    observed: "新实例号算不出".to_string(),
+                },
+                (None, _) => FieldOutcome::Violated {
+                    expected: "R1 那一版的树表读得出（认得出带不带树）".to_string(),
+                    observed: "R1 那一版的树表读不出".to_string(),
+                },
+            }
+        }
+    }
+}
+
+/// 系统配置 journal_tail 那一样（[`MountTransparencyField::SystemConfigurationJournalTail`]，判决第二节 Y1 journal_tail 那一行取攻方的式子）：
+/// R1 的 ≤ R2 的 ≤ R2 环里自证记录的最大计数器（环里一条都没有按 0），挂载做完（只挂载、挂载加一次发布）取等。
+/// 二次崩溃落回 v1 那一版时照这一行判，不按与 R1 相同判（判决表下那句：系统配置的运行量不是那一版自己的）。
+/// 两次恢复任一次择不到系统配置判红。
+fn system_configuration_journal_tail_outcome(
+    before_the_mount: &RecoveryFieldsRead,
+    after_the_action: &RecoveryFieldsRead,
+    action: MountTransparencyAction,
+) -> FieldOutcome {
+    let tail_of = |fields: &RecoveryFieldsRead| {
+        fields
+            .chosen_system_configuration_bytes
+            .as_deref()
+            .and_then(|bytes| read_u64_at(bytes, SYSTEM_CONFIGURATION_JOURNAL_TAIL))
+    };
+    let (Some(tail_before), Some(tail_after)) =
+        (tail_of(before_the_mount), tail_of(after_the_action))
+    else {
+        return FieldOutcome::Violated {
+            expected: "R1、R2 都择得到系统配置".to_string(),
+            observed: format!(
+                "R1 的 tail {:?}；R2 的 tail {:?}",
+                tail_of(before_the_mount),
+                tail_of(after_the_action)
+            ),
+        };
+    };
+    let highest_record_counter = after_the_action
+        .highest_record_counter_in_the_ring
+        .unwrap_or(0);
+    let mount_completed = match action {
+        MountTransparencyAction::MountAlone(_) | MountTransparencyAction::MountAndOnePublish(_) => {
+            true
+        }
+        MountTransparencyAction::SecondCrashInsideTheMount(_) => false,
+    };
+    held_or_violated(
+        tail_before <= tail_after
+            && tail_after <= highest_record_counter
+            && (!mount_completed || tail_after == highest_record_counter),
+        format!(
+            "R1 的 tail {tail_before} ≤ R2 的 tail ≤ R2 环里自证记录的最大计数器 {highest_record_counter}{}",
+            if mount_completed { "，挂载做完取等" } else { "" }
+        ),
+        format!("R2 的 tail {tail_after}"),
+    )
+}
+
+/// 系统配置的 journal 实例代号那一样（[`MountTransparencyField::SystemConfigurationJournalInstance`]，判决第二节 Y1 那一行取攻方的式子）：
+/// 挂载做完 = R2 那一版根的实例代号（从根记录原字节取）；二次崩溃 ∈ {R1 的, 新实例}。择不到系统配置判红。
+fn system_configuration_journal_instance_outcome(
+    before_the_mount: &RecoveryFieldsRead,
+    after_the_action: &RecoveryFieldsRead,
+    compared: &VersionFieldsRead,
+    new_instance: Option<InstanceGeneration>,
+    action: MountTransparencyAction,
+) -> FieldOutcome {
+    let instance_of = |fields: &RecoveryFieldsRead| {
+        fields
+            .chosen_system_configuration_bytes
+            .as_deref()
+            .and_then(|bytes| read_u32_at(bytes, SYSTEM_CONFIGURATION_JOURNAL_INSTANCE))
+    };
+    let observed = instance_of(after_the_action);
+    match action {
+        MountTransparencyAction::MountAlone(_) | MountTransparencyAction::MountAndOnePublish(_) => {
+            let root_instance = read_u32_at(&compared.root_record_bytes, ROOT_RECORD_INSTANCE);
+            held_or_violated(
+                observed.is_some() && observed == root_instance,
+                format!("R2 那一版根的实例代号 {root_instance:?}"),
+                format!("{observed:?}"),
+            )
+        }
+        MountTransparencyAction::SecondCrashInsideTheMount(_) => {
+            let before = instance_of(before_the_mount);
+            let new_instance = new_instance.map(|instance| instance.0);
+            held_or_violated(
+                observed.is_some() && (observed == before || observed == new_instance),
+                format!("R1 的 {before:?} 或新实例 {new_instance:?}"),
+                format!("{observed:?}"),
+            )
+        }
+    }
+}
+
+/// 树表条目按字节区间那一样（[`MountTransparencyField::TreeTableEntryBytes`]）：两版条目按盘上下标一一对上，逐条按
+/// [`TREE_TABLE_ENTRY_FIELD_BYTE_RANGES`] 比「必须相同」「必须为 0」那几段。条数不同、任一版树表读不出判红。
+fn tree_table_entry_bytes_outcome(
+    first_recovery: &VersionFieldsRead,
+    compared: &VersionFieldsRead,
+) -> FieldOutcome {
+    let (Some(entries_before), Some(entries_after)) = (
+        &first_recovery.tree_table_entries,
+        &compared.tree_table_entries,
+    ) else {
+        return FieldOutcome::Violated {
+            expected: "R1、R2 两版的树表都读得出".to_string(),
+            observed: format!(
+                "R1 读得出：{}；R2 读得出：{}",
+                first_recovery.tree_table_entries.is_some(),
+                compared.tree_table_entries.is_some()
+            ),
+        };
+    };
+    if entries_before.len() != entries_after.len() {
+        return FieldOutcome::Violated {
+            expected: format!("树表 {} 条", entries_before.len()),
+            observed: format!("树表 {} 条", entries_after.len()),
+        };
+    }
+    let entries_that_do_not_hold: Vec<String> = entries_before
+        .iter()
+        .zip(entries_after)
+        .enumerate()
+        .filter_map(|(index, (before, after))| {
+            let ranges =
+                byte_ranges_that_do_not_hold(TREE_TABLE_ENTRY_FIELD_BYTE_RANGES, before, after);
+            (!ranges.is_empty()).then(|| format!("第 {index} 条：{ranges:?}"))
+        })
+        .collect();
+    held_or_violated(
+        entries_that_do_not_hold.is_empty(),
+        "树表每一条里「必须相同」「必须为 0」的那几段都对得上".to_string(),
+        format!("这几条不符：{}", entries_that_do_not_hold.join("；")),
+    )
+}
+
 /// D23（journal 的角色与格式） 已定项 14 那张单子（C554 乙-配置续，用户 2026-09-27 定；C331 判不出改取 R2，用户 2026-10-03 定）上的拒可写理由。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub enum RefusalReasonOnTheD23List {
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 56effc4..a9ddd61 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -7991,3 +7991,508 @@ fn rewriting_an_older_root_rechains_the_later_record_of_its_instance_and_every_v
         "计数器 3 两盘各一份改新根段，计数器 4 两盘各一份重算反向链"
     );
 }
+
+// ───────── I-9.16 ③ ④、I-5.5 部分没判、I-2.4 根槽留位、带树一版的分配记录树根（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md`）─────────
+
+/// 新池新建文件那一版树表里十一条按发号次序排（从 0 数）：分配记录树第 2 条、deadlist 第 6 条、logged_ops 第 10 条。
+const ALLOCATION_TREE_TABLE_ENTRY_INDEX: usize = 2;
+const DEADLIST_TREE_TABLE_ENTRY_INDEX: usize = 6;
+const LOGGED_OPERATIONS_TREE_TABLE_ENTRY_INDEX: usize = 10;
+/// 种类码登记表（D8（核心索引结构） 已定项 9）里没有的一个码：分配记录那一条改成它，树表里就一条种类 3 都没有。
+const TREE_KIND_NOT_REGISTERED: u16 = 13;
+/// 新池新建文件那一版树表里 extent 树那条的树 ID（mkfs 的水位 65536）与 inode 树那条的树 ID（65537）：
+/// 改成种类 1 的那一条头 ID 照 extent 条目写成 inode 树的号，I-9.27 不先红。
+const INODE_TREE_IDENTIFIER_OF_THE_FIRST_FILE_VERSION: u64 = 65537;
+
+/// 树表里第 `index` 条改种类：先核那一条原来的种类，再改（树 ID、根指针不动）。
+fn set_the_kind_of_tree_table_entry(
+    tree_table: &mut [u8],
+    index: usize,
+    kind_before: u16,
+    kind_after: u16,
+) {
+    let kind_offset = tree_table_entry_offset(index) + TREE_TABLE_ENTRY_KIND_OFFSET;
+    assert_eq!(
+        u16::from_le_bytes([tree_table[kind_offset], tree_table[kind_offset + 1]]),
+        kind_before,
+        "树表第 {index} 条原来的种类"
+    );
+    set_u16(tree_table, kind_offset, kind_after);
+}
+
+/// 树表里第 `index` 条改成种类 1（extent 树），头 ID 写成 inode 树的号（种类 1 的头 ID 该等于同表种类 2 条目的树 ID，I-9.27）。
+fn turn_tree_table_entry_into_a_second_extent_entry(
+    tree_table: &mut [u8],
+    index: usize,
+    kind_before: u16,
+) {
+    set_the_kind_of_tree_table_entry(tree_table, index, kind_before, TREE_KIND_EXTENT);
+    set_u64(
+        tree_table,
+        tree_table_entry_offset(index) + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
+        INODE_TREE_IDENTIFIER_OF_THE_FIRST_FILE_VERSION,
+    );
+}
+
+/// 树表里去掉第 `index` 条：它后面的条目往前挪 200、末尾 200 字节清零、条目数与声明长度各减一条
+/// （码 2 头：key 宽 1 字节在 51、条目数 2 字节在 82 + 2k、声明长度 2 字节在 8；攻方 `harness-probe-tests.rs` 的 e4 造法）。
+fn remove_tree_table_entry(tree_table: &mut [u8], index: usize) {
+    let key_width = usize::from(tree_table[51]);
+    let count_offset = 82 + 2 * key_width;
+    let count = usize::from(u16::from_le_bytes([
+        tree_table[count_offset],
+        tree_table[count_offset + 1],
+    ]));
+    let start = tree_table_entry_offset(index);
+    let end = tree_table_entry_offset(count);
+    tree_table.copy_within(start + 200..end, start);
+    tree_table[end - 200..end].fill(0);
+    set_u16(
+        tree_table,
+        count_offset,
+        u16::try_from(count - 1).expect("条目数装得进 u16"),
+    );
+    let declared = u16::from_le_bytes([tree_table[8], tree_table[9]]);
+    set_u16(tree_table, 8, declared - 200);
+}
+
+/// 一份坏镜像上判红的只有 I-9.16、第一处违例是 `expected_violation`；干净镜像上 I-9.16 真被评估过且成立。
+fn assert_only_the_tree_table_ordering_invariant_is_violated(
+    clean: &MemoryPool,
+    image: &MemoryPool,
+    expected_violation: &str,
+) {
+    assert_eq!(
+        verdict(clean, "I-9.16"),
+        InvariantVerdict::Holds,
+        "干净镜像上 I-9.16 真被评估过且成立"
+    );
+    let verdicts = check_pool_image(image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-9.16"],
+        "判红的该只有 I-9.16：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict(image, "I-9.16"),
+        InvariantVerdict::Violated(expected_violation.to_owned()),
+        "红在 ③ ④ 那一道"
+    );
+}
+
+/// I-9.16 ③（攻方 b2）：livelist 那一条改成种类 1、头 ID 照 extent 条目写成 inode 树的号——树表里种类 1 两条。
+/// 树 ID 次序、发号次序（取先出现的那条）、头 ID 都对得上，改之前整份 checker 一条不红（core 读这一版报「树表里同一种树有两条」）。
+#[test]
+fn a_tree_table_whose_livelist_entry_became_a_second_extent_entry_reddens_only_the_tree_table_ordering_invariant(
+) {
+    let clean =
+        build_pool("known-bad-tree-table-second-extent-entry-from-the-livelist").memory_pool();
+    let mut image = clean.clone();
+    mutate_unit(
+        &mut image,
+        TREE_TABLE,
+        |bytes| {
+            turn_tree_table_entry_into_a_second_extent_entry(
+                bytes,
+                LIVELIST_TREE_TABLE_ENTRY_INDEX,
+                TREE_KIND_LIVELIST,
+            );
+        },
+        true,
+    );
+    assert_only_the_tree_table_ordering_invariant_is_violated(
+        &clean,
+        &image,
+        "根（实例 1、txg 3）的树表：种类 1 有 2 条（③ 每种至多一条）",
+    );
+}
+
+/// I-9.16 ③（攻方 b3）：logged_ops 那一条（树表最末一条）改成种类 1、头 ID 写成 inode 树的号。
+#[test]
+fn a_tree_table_whose_logged_operations_entry_became_a_second_extent_entry_reddens_only_the_tree_table_ordering_invariant(
+) {
+    let clean = build_pool("known-bad-tree-table-second-extent-entry-from-the-logged-operations")
+        .memory_pool();
+    let mut image = clean.clone();
+    mutate_unit(
+        &mut image,
+        TREE_TABLE,
+        |bytes| {
+            turn_tree_table_entry_into_a_second_extent_entry(
+                bytes,
+                LOGGED_OPERATIONS_TREE_TABLE_ENTRY_INDEX,
+                TREE_KIND_LOGGED_OPERATIONS,
+            );
+        },
+        true,
+    );
+    assert_only_the_tree_table_ordering_invariant_is_violated(
+        &clean,
+        &image,
+        "根（实例 1、txg 3）的树表：种类 1 有 2 条（③ 每种至多一条）",
+    );
+}
+
+/// I-9.16 ④（正推 e1）：分配记录那一条的种类改成没登记的码——树表里一条种类 3 都没有。
+#[test]
+fn a_tree_table_whose_allocation_entry_carries_an_unregistered_kind_reddens_only_the_tree_table_ordering_invariant(
+) {
+    let clean =
+        build_pool("known-bad-tree-table-allocation-entry-of-an-unregistered-kind").memory_pool();
+    let mut image = clean.clone();
+    mutate_unit(
+        &mut image,
+        TREE_TABLE,
+        |bytes| {
+            set_the_kind_of_tree_table_entry(
+                bytes,
+                ALLOCATION_TREE_TABLE_ENTRY_INDEX,
+                TREE_KIND_ALLOCATION,
+                TREE_KIND_NOT_REGISTERED,
+            );
+        },
+        true,
+    );
+    assert_only_the_tree_table_ordering_invariant_is_violated(
+        &clean,
+        &image,
+        "根（实例 1、txg 3）的树表有 11 条，种类 3 一条都没有（④ 树表非空时种类 1、2、3、4 各恰好一条）",
+    );
+}
+
+/// I-9.16 ③（攻方 e2）：deadlist 那一条改成种类 3——树表里种类 3 两条（deadlist 的根是空根，改种类不牵动别的判定）。
+#[test]
+fn a_tree_table_whose_deadlist_entry_became_a_second_allocation_entry_reddens_only_the_tree_table_ordering_invariant(
+) {
+    let clean =
+        build_pool("known-bad-tree-table-second-allocation-entry-from-the-deadlist").memory_pool();
+    let mut image = clean.clone();
+    mutate_unit(
+        &mut image,
+        TREE_TABLE,
+        |bytes| {
+            set_the_kind_of_tree_table_entry(
+                bytes,
+                DEADLIST_TREE_TABLE_ENTRY_INDEX,
+                TREE_KIND_DEADLIST,
+                TREE_KIND_ALLOCATION,
+            );
+        },
+        true,
+    );
+    assert_only_the_tree_table_ordering_invariant_is_violated(
+        &clean,
+        &image,
+        "根（实例 1、txg 3）的树表：种类 3 有 2 条（③ 每种至多一条）",
+    );
+}
+
+/// I-9.16 ④（攻方 e4）：整条去掉分配记录那一条——树表 10 条、没有种类 3。
+#[test]
+fn a_tree_table_without_its_allocation_entry_reddens_only_the_tree_table_ordering_invariant() {
+    let clean = build_pool("known-bad-tree-table-without-the-allocation-entry").memory_pool();
+    let mut image = clean.clone();
+    mutate_unit(
+        &mut image,
+        TREE_TABLE,
+        |bytes| {
+            let kind_offset = tree_table_entry_offset(ALLOCATION_TREE_TABLE_ENTRY_INDEX)
+                + TREE_TABLE_ENTRY_KIND_OFFSET;
+            assert_eq!(
+                u16::from_le_bytes([bytes[kind_offset], bytes[kind_offset + 1]]),
+                TREE_KIND_ALLOCATION,
+                "去掉的是分配记录那一条"
+            );
+            remove_tree_table_entry(bytes, ALLOCATION_TREE_TABLE_ENTRY_INDEX);
+        },
+        true,
+    );
+    assert_only_the_tree_table_ordering_invariant_is_violated(
+        &clean,
+        &image,
+        "根（实例 1、txg 3）的树表有 10 条，种类 3 一条都没有（④ 树表非空时种类 1、2、3、4 各恰好一条）",
+    );
+}
+
+/// I-9.16 ③（攻方 f2）：覆盖写之后那一版（txg 4，不是从 mkfs 水位建树的那一版，I-5.5 不认它）的树表里 deadlist 改成种类 3。
+/// I-9.16 按每张树表第一次被走到时判，这一张是 B 自己的。
+#[test]
+fn the_tree_table_after_the_overwrite_whose_deadlist_entry_became_a_second_allocation_entry_reddens_only_the_tree_table_ordering_invariant(
+) {
+    let clean = image_after_the_overwrite(
+        "known-bad-tree-table-after-the-overwrite-second-allocation-entry",
+    );
+    let mut image = clean.clone();
+    mutate_unit_of(
+        &mut image,
+        &UNITS_AFTER_OVERWRITE,
+        TREE_TABLE_AFTER_OVERWRITE,
+        |bytes| {
+            set_the_kind_of_tree_table_entry(
+                bytes,
+                DEADLIST_TREE_TABLE_ENTRY_INDEX,
+                TREE_KIND_DEADLIST,
+                TREE_KIND_ALLOCATION,
+            );
+        },
+        true,
+    );
+    assert_only_the_tree_table_ordering_invariant_is_violated(
+        &clean,
+        &image,
+        "根（实例 1、txg 4）的树表：种类 3 有 2 条（③ 每种至多一条）",
+    );
+}
+
+/// 两块盘上同一个槽那一份单元整个写 0（不重封、不补引用链）。
+fn zero_both_copies_of_the_unit(image: &mut MemoryPool, slot: u64, length: usize) {
+    for device in DEVICES {
+        write(image, device, slot * SLOT, &vec![0u8; length]);
+    }
+}
+
+/// 新池新建文件写出的分配记录树根（`UNITS` 里 50249，先叶后根）。
+const ALLOCATION_RECORD_TREE_ROOT: u64 = 50249;
+
+/// 一份坏镜像上 I-5.5 报部分没判、说明里点名最新那条根（实例 1、txg 3）与 `why_not_judged`，判红的照旧是 `violated`。
+fn assert_the_first_data_unit_invariant_is_partially_judged(
+    image: &MemoryPool,
+    why_not_judged: &str,
+    violated: &[&str],
+) {
+    let verdicts = check_pool_image(image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        violated,
+        "判红的照改之前那几条：{verdicts:?}"
+    );
+    let expected_unjudged =
+        format!("判到的那几版落点都对得上；这几版没判：根（实例 1、txg 3）{why_not_judged}");
+    assert_eq!(
+        verdict(
+            image,
+            FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
+        ),
+        InvariantVerdict::PartiallyJudged(expected_unjudged),
+        "I-5.5 报部分没判，逐版列出没判的那一版与为什么"
+    );
+}
+
+/// 读不出的单元（两份全零、不补引用链）由 I-2.1、I-4.8、I-7.2、I-7.4 判红。
+const VIOLATED_WHEN_A_UNIT_OF_THE_NEWEST_ROOT_IS_ZEROED: [&str; 4] =
+    ["I-2.1", "I-4.8", "I-7.2", "I-7.4"];
+
+/// I-5.5（攻方 a1）：最新那条根的树表单元两份全零——认不出那一版是不是从 mkfs 水位建树的那一版，部分没判。
+#[test]
+fn a_tree_table_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged() {
+    let mut image = build_pool("known-bad-first-data-unit-tree-table-zeroed").memory_pool();
+    zero_both_copies_of_the_unit(&mut image, TREE_TABLE, 16384);
+    assert_the_first_data_unit_invariant_is_partially_judged(
+        &image,
+        "：树表读不出，认不出它是不是从 mkfs 水位第一次建树的那一版",
+        &VIOLATED_WHEN_A_UNIT_OF_THE_NEWEST_ROOT_IS_ZEROED,
+    );
+}
+
+/// I-5.5（攻方 c1）：extent 树根两份全零——那一版引用的数据单元数不全，部分没判。
+#[test]
+fn an_extent_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged() {
+    let mut image = build_pool("known-bad-first-data-unit-extent-root-zeroed").memory_pool();
+    zero_both_copies_of_the_unit(&mut image, EXTENT_ROOT, 16384);
+    assert_the_first_data_unit_invariant_is_partially_judged(
+        &image,
+        "（从 mkfs 水位第一次建树的那一版）：extent 树读不全，它引用的数据单元数不全",
+        &VIOLATED_WHEN_A_UNIT_OF_THE_NEWEST_ROOT_IS_ZEROED,
+    );
+}
+
+/// I-5.5（攻方 e3）：分配记录树根两份全零——落点算不出，部分没判。
+#[test]
+fn an_allocation_record_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged(
+) {
+    let mut image = build_pool("known-bad-first-data-unit-allocation-root-zeroed").memory_pool();
+    zero_both_copies_of_the_unit(&mut image, ALLOCATION_RECORD_TREE_ROOT, 16384);
+    assert_the_first_data_unit_invariant_is_partially_judged(
+        &image,
+        "（从 mkfs 水位第一次建树的那一版）：分配记录树读不全，落点算不出",
+        &VIOLATED_WHEN_A_UNIT_OF_THE_NEWEST_ROOT_IS_ZEROED,
+    );
+}
+
+/// I-5.5（攻方 d1）：第一个文件的数据指针第二条位置条目从盘 1 改到盘 0——盘 1 上一个数据单元都没有，该盘跳过、不记，
+/// 盘 0 上照判、成立；位置条目不按设备升序，I-2.5 照旧红。
+#[test]
+fn a_data_pointer_with_no_location_on_one_device_skips_that_device_and_the_first_data_unit_invariant_holds(
+) {
+    let clean = build_pool("known-bad-first-data-unit-no-location-on-device-one").memory_pool();
+    let data_unit_checksum = crc32_castagnoli(&read(&clean, 0, DATA_UNIT * SLOT, 32768));
+    let mut image = clean.clone();
+    mutate_unit(
+        &mut image,
+        EXTENT_ROOT,
+        |bytes| {
+            assert!(
+                replace_all(
+                    bytes,
+                    &location_pattern(1, DATA_UNIT, data_unit_checksum),
+                    &location_pattern(0, DATA_UNIT, data_unit_checksum),
+                ),
+                "extent 根里有第一个文件的数据指针"
+            );
+        },
+        true,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-2.5"],
+        "判红的只有位置条目次序那一条：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict(
+            &image,
+            FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
+        ),
+        InvariantVerdict::Holds,
+        "盘 1 上没有数据单元：该盘跳过、不记，盘 0 上判到且成立"
+    );
+}
+
+/// 改 checkpoint_txg 为 `txg` 的那条根的根槽、重算整槽自证校验和，再把每条根抄进它那几条记录的新根段（I-8.10 照旧成立）。
+fn rewrite_the_root_slot_of(
+    image: &mut MemoryPool,
+    txg: CheckpointTxg,
+    change: impl Fn(&mut [u8]),
+) {
+    let (device, offset) = root_slot_of(txg);
+    let mut bytes = read(image, device, offset, 512);
+    assert_eq!(get_u64(&bytes, 28), txg.0, "那一槽是 txg {} 的根", txg.0);
+    change(&mut bytes);
+    let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
+    bytes[138..170].copy_from_slice(&digest);
+    write(image, device, offset, &bytes);
+    copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
+}
+
+/// 根记录里算法类型（428）、nonce（429 起 12）、MAC（441 起 16）的偏移（D22（单元原子性怎么合成） 已定项 7）。
+const ROOT_RECORD_ALGORITHM_OFFSET: usize = 428;
+const ROOT_RECORD_NONCE_OFFSET: usize = 429;
+const ROOT_RECORD_MAC_OFFSET: usize = 441;
+
+/// 发布 B（txg 4）那条根的根槽留位里改一个字节：只红 I-2.4，第一处违例写明是哪一字节。
+fn assert_a_root_slot_reserved_byte_reddens_only_the_header_checksum_coverage_invariant(
+    tag: &str,
+    offset_in_the_root_record: usize,
+    value: u8,
+) {
+    let clean = image_after_the_overwrite(tag);
+    assert_eq!(
+        verdict(&clean, "I-2.4"),
+        InvariantVerdict::Holds,
+        "干净镜像上 I-2.4 真被评估过且成立"
+    );
+    let mut image = clean.clone();
+    rewrite_the_root_slot_of(&mut image, CheckpointTxg(4), |bytes| {
+        bytes[offset_in_the_root_record] = value;
+    });
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-2.4"],
+        "判红的只有 I-2.4：{verdicts:?}"
+    );
+    let InvariantVerdict::Violated(detail) = verdict(&image, "I-2.4") else {
+        panic!("I-2.4 要判红");
+    };
+    assert!(
+        detail.contains("txg 4")
+            && detail.contains(&format!(
+                "根槽偏移 {offset_in_the_root_record} 的字节是 {value:#04x}"
+            )),
+        "第一处违例写明是哪条根、哪一字节：{detail}"
+    );
+}
+
+/// I-2.4 根槽留位（攻方 m4）：算法类型写成 1。
+#[test]
+fn a_root_slot_whose_algorithm_type_is_one_reddens_only_the_header_checksum_coverage_invariant() {
+    assert_a_root_slot_reserved_byte_reddens_only_the_header_checksum_coverage_invariant(
+        "known-bad-root-slot-algorithm-type",
+        ROOT_RECORD_ALGORITHM_OFFSET,
+        1,
+    );
+}
+
+/// I-2.4 根槽留位（攻方 m5）：nonce 第二个字节写成 0x5a。
+#[test]
+fn a_root_slot_whose_nonce_has_a_non_zero_byte_reddens_only_the_header_checksum_coverage_invariant()
+{
+    assert_a_root_slot_reserved_byte_reddens_only_the_header_checksum_coverage_invariant(
+        "known-bad-root-slot-nonce",
+        ROOT_RECORD_NONCE_OFFSET + 1,
+        0x5a,
+    );
+}
+
+/// I-2.4 根槽留位（攻方 m6）：MAC 第十个字节写成 0x5a。
+#[test]
+fn a_root_slot_whose_mac_has_a_non_zero_byte_reddens_only_the_header_checksum_coverage_invariant() {
+    assert_a_root_slot_reserved_byte_reddens_only_the_header_checksum_coverage_invariant(
+        "known-bad-root-slot-mac",
+        ROOT_RECORD_MAC_OFFSET + 9,
+        0x5a,
+    );
+}
+
+/// 根记录里「树表 0 条那一版的分配记录树根」86 字节的偏移（D22（单元原子性怎么合成） 已定项 7）。
+const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET: usize = 342;
+/// 树表条目里根指针 86 字节的偏移（树 ID 8 + 条目长度 2 + 种类 2 + flags 2 之后）。
+const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET: usize = 14;
+const NODE_POINTER_BYTES: usize = 86;
+
+/// 带树的一版根记录里分配记录树根写成非零指针（判决第二节 Y1 `allocation_record_tree_root` 那一行；两腿都没造这份）：新池新建文件那条根（txg 3）
+/// 那 86 字节抄成同一版树表里分配记录那一条的根指针——指着一份合法的分配记录节点（50249），这一版就有了两棵分配记录树。
+/// 判在 I-5.4 下（这一格归属交主 agent 定，见 `walk::judge_allocation_records_disjoint`）。
+/// 走读另有两条红，照实钉住：走读把根记录这一项当「树表 0 条那一版的分配记录树」（树 0）走下去，50249 头里是树 65538，红 I-1.3；
+/// 同一版里 50249 被根记录这一项与树表种类 3 那一条各引用一次，红 I-5.1。这两条在改之前就红；改之后 I-5.4 多红一条、点名这一版有两棵分配记录树。
+#[test]
+fn a_file_version_whose_root_record_carries_a_non_zero_allocation_record_tree_root_reddens_the_disjointness_invariant_besides_the_walk(
+) {
+    let clean =
+        build_pool("known-bad-file-version-with-an-allocation-record-tree-root").memory_pool();
+    assert_eq!(
+        verdict(&clean, "I-5.4"),
+        InvariantVerdict::Holds,
+        "干净镜像上 I-5.4 真被评估过且成立"
+    );
+    let tree_table = read(&clean, 0, TREE_TABLE * SLOT, 16384);
+    let allocation_entry = tree_table_entry_offset(ALLOCATION_TREE_TABLE_ENTRY_INDEX);
+    let allocation_root_pointer =
+        tree_table[allocation_entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET
+            ..allocation_entry + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET + NODE_POINTER_BYTES]
+            .to_vec();
+    let mut image = clean.clone();
+    rewrite_the_root_slot_of(&mut image, CheckpointTxg(3), |bytes| {
+        let field = &mut bytes[ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET
+            ..ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET + NODE_POINTER_BYTES];
+        assert!(
+            field.iter().all(|byte| *byte == 0),
+            "带文件的一版这一项本来全零"
+        );
+        field.copy_from_slice(&allocation_root_pointer);
+    });
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-1.3", "I-5.1", "I-5.4"],
+        "判红的是 I-5.4 加走读带出来的 I-1.3、I-5.1：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict(&image, "I-5.4"),
+        InvariantVerdict::Violated(
+            "根（实例 1、txg 3）的树表有 11 条（带树的一版），根记录里树表 0 条那一版的分配记录树根却不是全零：这一版有两棵分配记录树"
+                .to_owned()
+        ),
+        "红在最新那条根"
+    );
+}
diff --git a/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
index 6e52a01..e798500 100644
--- a/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
+++ b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
@@ -15,6 +15,7 @@ use singlefs_core::address::{
 };
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::checksum::wide_checksum_with_field_zeroed;
 use singlefs_core::container_tree::ContainerTreeNodeEntries;
 use singlefs_core::lineage_tree::LineageContainerCapacity;
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
@@ -24,7 +25,8 @@ use singlefs_core::mount::{
     ShadowLedger, Unmounted,
 };
 use singlefs_core::mounted_session::MountedSession;
-use singlefs_core::root_record::UnmountMarker;
+use singlefs_core::records::{TREE_KIND_DELETED_INODES, TREE_KIND_INODE};
+use singlefs_core::root_record::{RootRecord, UnmountMarker, ROOT_CHECKSUM_OFFSET};
 use singlefs_core::system_configuration::SystemConfiguration;
 use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
 use singlefs_core::transaction::{
@@ -44,11 +46,15 @@ use singlefs_harness::memory_pool::{
     WrittenContents,
 };
 use singlefs_harness::mount_transparency::{
-    judge_mount_transparency, read_recovery_fields, ComparedVersion, FieldNotJudged, FieldOutcome,
-    FloorRaiseInTheMountWrites, FloorRaisedByTheMount, FloorRaisesAtTheSecondCrash,
-    FloorRaisesInTheMountWrites, FloorRaisesOfTheMount, MountTransparency, MountTransparencyAction,
-    MountTransparencyField, PosixContentRead, PublishInTheMountWrites, RecoveryFieldsRead,
-    UserVisibleTreeRootPointersOfTheContinuation, VersionFieldsRead,
+    judge_mount_transparency, read_recovery_fields, ComparedVersion, FieldByteRange,
+    FieldByteRangeClass, FieldNotJudged, FieldOutcome, FloorRaiseInTheMountWrites,
+    FloorRaisedByTheMount, FloorRaisesAtTheSecondCrash, FloorRaisesInTheMountWrites,
+    FloorRaisesOfTheMount, MountTransparency, MountTransparencyAction, MountTransparencyField,
+    PosixContentRead, PublishInTheMountWrites, RecoveryFieldsRead, SystemConfigurationTier,
+    TreeTableEntryRootPointersByKind, UserVisibleTreeRootPointersOfTheContinuation,
+    VersionFieldsRead, D13_ROWS_ADDED_BY_THE_WRITE_BACK, D13_ROWS_OF_TODAY,
+    ROOT_RECORD_FIELD_BYTE_RANGES, SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES,
+    TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES, TREE_TABLE_ENTRY_FIELD_BYTE_RANGES,
 };
 use singlefs_harness::segments::StepKind;
 use singlefs_harness::{RecordingBlockDevice, SharedStream};
@@ -178,7 +184,8 @@ fn publish_once_after_the_mount(
     .expect("挂载之后那一次发布该成");
 }
 
-/// 一次判下来要的四样：R1、R2、动作与挂载续做那几步的两根指针。
+/// 一次判下来要的四样：R1、R2、动作与挂载续做那几步的根指针。
+#[derive(Clone)]
 struct JudgedScenario {
     before_the_mount: RecoveryFieldsRead,
     after_the_action: RecoveryFieldsRead,
@@ -675,30 +682,65 @@ fn a_tree_identifier_watermark_that_moved_with_the_mount_is_red() {
 }
 
 #[test]
-fn the_two_user_visible_tree_root_pointers_before_the_publish_after_the_mount_are_those_of_the_crash_image(
+fn the_tree_table_entry_root_pointers_before_the_publish_after_the_mount_are_those_of_the_crash_image(
 ) {
     let scenario = mount_and_one_publish_on(SMALL, &pool_after_the_first_file());
     assert_holds(
         &scenario.judge(),
-        MountTransparencyField::UserVisibleTreeRootPointers,
+        MountTransparencyField::TreeTableEntryRootPointers,
     );
 }
 
+/// 树表条目里种类那 2 字节与根指针 86 字节的偏移（D8（核心索引结构） 已定项 8 的字段表）。
+const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
+const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET: usize = 14;
+
+/// R2 那一版树表里种类 `kind` 那一条的根指针第一个字节翻一下（0 变 0xff、非 0 异或 0xff）：只改原字节里的那一处。
+fn flip_the_root_pointer_of_the_tree_table_entry_of_kind(
+    fields: &mut RecoveryFieldsRead,
+    kind: u16,
+) {
+    let entries = compared_version_mut(fields)
+        .tree_table_entries
+        .as_mut()
+        .expect("R2 那一版的树表读得出");
+    let entry = entries
+        .iter_mut()
+        .find(|entry| {
+            u16::from_le_bytes([
+                entry[TREE_TABLE_ENTRY_KIND_OFFSET],
+                entry[TREE_TABLE_ENTRY_KIND_OFFSET + 1],
+            ]) == kind
+        })
+        .unwrap_or_else(|| panic!("带文件的一版树表里有种类 {kind}"));
+    entry[TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET] ^= 0xff;
+}
+
 #[test]
 fn an_inode_tree_root_pointer_that_moved_with_the_mount_is_red() {
     let mut scenario = mount_and_one_publish_on(SMALL, &pool_after_the_first_file());
-    let pointers = compared_version_mut(&mut scenario.after_the_action)
-        .user_visible_tree_root_pointers
-        .as_mut()
-        .expect("R2 那一版的树表读得出");
-    let inode_tree = pointers
-        .inode_tree
-        .as_mut()
-        .expect("带文件的一版树表里有 inode 树");
-    inode_tree[0] ^= 0xff;
+    flip_the_root_pointer_of_the_tree_table_entry_of_kind(
+        &mut scenario.after_the_action,
+        TREE_KIND_INODE,
+    );
+    assert_only_this_field_is_violated(
+        &scenario.judge(),
+        MountTransparencyField::TreeTableEntryRootPointers,
+    );
+}
+
+/// 判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1：墓碑、谱系、deleted_inodes 三根并进根指针那一行
+/// （攻方 H3）。没续做的挂载上 R2 的 deleted_inodes 根被挪了：红在树表条目的根指针那一样。判别力：种类集合退回 {1, 2} 时这一格不比、判绿。
+#[test]
+fn a_deleted_inodes_root_pointer_that_moved_with_a_mount_that_did_not_continue_is_red() {
+    let mut scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
+    flip_the_root_pointer_of_the_tree_table_entry_of_kind(
+        &mut scenario.after_the_action,
+        TREE_KIND_DELETED_INODES,
+    );
     assert_only_this_field_is_violated(
         &scenario.judge(),
-        MountTransparencyField::UserVisibleTreeRootPointers,
+        MountTransparencyField::TreeTableEntryRootPointers,
     );
 }
 
@@ -768,8 +810,16 @@ fn a_second_crash_right_after_the_acquisition_recovers_the_first_version_field_b
             | MountTransparencyField::InstanceGeneration
             | MountTransparencyField::InstanceTableRows
             | MountTransparencyField::TreeIdentifierWatermark
-            | MountTransparencyField::UserVisibleTreeRootPointers
-            | MountTransparencyField::CheckpointTxg => FieldOutcome::Holds,
+            | MountTransparencyField::TreeTableEntryRootPointers
+            | MountTransparencyField::CheckpointTxg
+            | MountTransparencyField::MappingRootEmptiness
+            | MountTransparencyField::AllocationRecordTreeRoot
+            | MountTransparencyField::SystemConfigurationJournalTail
+            | MountTransparencyField::SystemConfigurationJournalInstance
+            | MountTransparencyField::SystemRuntimeConfiguration
+            | MountTransparencyField::RootRecordBytes
+            | MountTransparencyField::SystemConfigurationBytes
+            | MountTransparencyField::TreeTableEntryBytes => FieldOutcome::Holds,
         };
         assert_eq!(
             verdict.outcome_of(field),
@@ -1128,24 +1178,29 @@ fn after_a_mount_that_continued_the_deletion_the_two_tree_root_pointers_are_thos
         PosixContentRead::NoFile,
         "前提：R1 落到意图那一版，第一个文件在删除中、读出「没有文件」"
     );
+    let deleted_inodes_root_pointer = |fields: &RecoveryFieldsRead| {
+        TreeTableEntryRootPointersByKind::of_the_entries(
+            fields
+                .compared_version
+                .as_ref()
+                .expect("落到一版")
+                .tree_table_entries
+                .as_ref()
+                .expect("那一版的树表读得出"),
+            &TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES,
+        )
+        .root_pointer_of_each_kind
+        .get(&TREE_KIND_DELETED_INODES)
+        .cloned()
+    };
     assert_ne!(
-        scenario
-            .after_the_action
-            .compared_version
-            .as_ref()
-            .expect("R2 落到一版")
-            .user_visible_tree_root_pointers,
-        scenario
-            .before_the_mount
-            .compared_version
-            .as_ref()
-            .expect("R1 落到一版")
-            .user_visible_tree_root_pointers,
-        "前提：续做改了两棵树的根指针"
+        deleted_inodes_root_pointer(&scenario.after_the_action),
+        deleted_inodes_root_pointer(&scenario.before_the_mount),
+        "前提：续做改了 deleted_inodes 的根（攻方 H3 那一形：按「必须相同」比就红）"
     );
     assert_holds(
         &scenario.judge(),
-        MountTransparencyField::UserVisibleTreeRootPointers,
+        MountTransparencyField::TreeTableEntryRootPointers,
     );
 }
 
@@ -1376,3 +1431,442 @@ fn floor_raises_pushed_by_both_the_admission_and_the_continuation_are_located_fr
         "崩在续做推的那一串里：F 不照「没推」判"
     );
 }
+
+// ───────── 判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1、Y2 补的几样与按字节区间的字段表 ─────────
+
+/// mkfs 之后可写挂载一次（实例 1，不写行）再关掉的盘面：树表 0 条，下一次可写挂载取实例 2、在树表 0 条的一版上写行。
+fn pool_after_one_mount_without_file() -> MemoryPool {
+    pool_after_the_history(
+        SMALL,
+        HistoryStartingPoint::AfterMakeFilesystem,
+        vec![HistoryOperation::CloseAndMountWritable],
+        SpaceAdmission::JudgedByTheFormula,
+    )
+}
+
+/// 根记录里中央映射树根指针与分配记录树根指针的偏移（D22（单元原子性怎么合成） 已定项 7）。
+const ROOT_RECORD_MAPPING_ROOT_OFFSET: usize = 256;
+const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET: usize = 342;
+const NODE_POINTER_BYTES: usize = 86;
+/// 系统配置里 journal tail 与 journal 实例代号的偏移（`.claude/kb/layout/01-first-txn.md`「系统配置字段表的偏移」）。
+const SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET: usize = 469;
+const SYSTEM_CONFIGURATION_JOURNAL_INSTANCE_OFFSET: usize = 477;
+
+fn root_record_bytes(fields: &RecoveryFieldsRead) -> &[u8] {
+    &fields
+        .compared_version
+        .as_ref()
+        .expect("落到一版")
+        .root_record_bytes
+}
+
+fn chosen_system_configuration_bytes_mut(fields: &mut RecoveryFieldsRead) -> &mut Vec<u8> {
+    fields
+        .chosen_system_configuration_bytes
+        .as_mut()
+        .expect("择得到系统配置")
+}
+
+#[test]
+fn the_mapping_root_stays_all_zero_after_a_mount_alone_on_a_pool_without_file() {
+    let scenario = mount_alone_on(SMALL, &pool_after_make_filesystem());
+    for fields in [&scenario.before_the_mount, &scenario.after_the_action] {
+        assert!(
+            root_record_bytes(fields)[ROOT_RECORD_MAPPING_ROOT_OFFSET
+                ..ROOT_RECORD_MAPPING_ROOT_OFFSET + NODE_POINTER_BYTES]
+                .iter()
+                .all(|byte| *byte == 0),
+            "前提：树表 0 条的一版映射根全零"
+        );
+    }
+    assert_holds(
+        &scenario.judge(),
+        MountTransparencyField::MappingRootEmptiness,
+    );
+}
+
+#[test]
+fn a_mapping_root_written_non_zero_by_a_mount_on_a_pool_without_file_is_red() {
+    let mut scenario = mount_alone_on(SMALL, &pool_after_make_filesystem());
+    compared_version_mut(&mut scenario.after_the_action).root_record_bytes
+        [ROOT_RECORD_MAPPING_ROOT_OFFSET] = 1;
+    assert_only_this_field_is_violated(
+        &scenario.judge(),
+        MountTransparencyField::MappingRootEmptiness,
+    );
+}
+
+/// 树表 0 条、实例 1 写过的盘面上再挂一次：新实例号 2 ≥ 2，写行那次发布在树表 0 条的一版上写分配记录树、根记录那一项非零。
+#[test]
+fn a_mount_taking_instance_two_on_a_pool_without_file_writes_a_non_zero_allocation_record_tree_root(
+) {
+    let scenario = mount_alone_on(SMALL, &pool_after_one_mount_without_file());
+    assert_eq!(
+        scenario
+            .after_the_action
+            .compared_version
+            .as_ref()
+            .expect("R2 落到一版")
+            .root
+            .instance,
+        InstanceGeneration(2),
+        "前提：这次挂载取实例 2"
+    );
+    assert!(
+        root_record_bytes(&scenario.after_the_action)[ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET
+            ..ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET + NODE_POINTER_BYTES]
+            .iter()
+            .any(|byte| *byte != 0),
+        "前提：R2 根记录里的分配记录树根非零"
+    );
+    assert_holds(
+        &scenario.judge(),
+        MountTransparencyField::AllocationRecordTreeRoot,
+    );
+}
+
+#[test]
+fn a_non_zero_allocation_record_tree_root_after_a_mount_on_a_file_version_is_red() {
+    let mut scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
+    assert_holds(
+        &scenario.judge(),
+        MountTransparencyField::AllocationRecordTreeRoot,
+    );
+    compared_version_mut(&mut scenario.after_the_action).root_record_bytes
+        [ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_OFFSET] = 1;
+    assert_only_this_field_is_violated(
+        &scenario.judge(),
+        MountTransparencyField::AllocationRecordTreeRoot,
+    );
+}
+
+#[test]
+fn the_journal_tail_after_a_mount_and_one_publish_is_the_highest_record_counter_in_the_ring() {
+    let scenario = mount_and_one_publish_on(SMALL, &pool_after_the_first_file());
+    assert_holds(
+        &scenario.judge(),
+        MountTransparencyField::SystemConfigurationJournalTail,
+    );
+}
+
+/// 二次崩溃只判 R1 ≤ R2 ≤ 环里最大计数器（不取等）：tail 越过环里自证记录的最大计数器，红在上界那一半（攻方 q2「tail 多写 100」）。
+#[test]
+fn a_journal_tail_past_every_record_in_the_ring_of_a_second_crash_image_is_red() {
+    let mut scenario = second_crash_after_the_acquisition(&pool_after_the_first_file());
+    let highest_record_counter = scenario
+        .after_the_action
+        .highest_record_counter_in_the_ring
+        .expect("环里有记录");
+    chosen_system_configuration_bytes_mut(&mut scenario.after_the_action)
+        [SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET..SYSTEM_CONFIGURATION_JOURNAL_TAIL_OFFSET + 8]
+        .copy_from_slice(&(highest_record_counter + 100).to_le_bytes());
+    assert_only_this_field_is_violated(
+        &scenario.judge(),
+        MountTransparencyField::SystemConfigurationJournalTail,
+    );
+}
+
+#[test]
+fn the_journal_instance_of_the_system_configuration_after_a_mount_alone_is_the_instance_of_its_root(
+) {
+    let scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
+    assert_holds(
+        &scenario.judge(),
+        MountTransparencyField::SystemConfigurationJournalInstance,
+    );
+}
+
+/// 攻方 q3「journal 实例代号写成 9」。
+#[test]
+fn a_journal_instance_of_the_system_configuration_other_than_the_instance_of_the_root_is_red() {
+    let mut scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
+    chosen_system_configuration_bytes_mut(&mut scenario.after_the_action)
+        [SYSTEM_CONFIGURATION_JOURNAL_INSTANCE_OFFSET
+            ..SYSTEM_CONFIGURATION_JOURNAL_INSTANCE_OFFSET + 4]
+        .copy_from_slice(&9u32.to_le_bytes());
+    assert_only_this_field_is_violated(
+        &scenario.judge(),
+        MountTransparencyField::SystemConfigurationJournalInstance,
+    );
+}
+
+/// 一张字节表从 0 起首尾相接、恰好盖满 `width` 字节（无洞、无重叠），每一段引的 D13 行名是登记过的那几行之一。
+fn assert_the_ranges_cover_the_width_end_to_end(
+    table_name: &str,
+    ranges: &[&FieldByteRange],
+    width: u64,
+) {
+    let mut next_byte = 0usize;
+    for range in ranges {
+        assert_eq!(
+            range.bytes.start, next_byte,
+            "{table_name}「{}」要接在上一段末尾 {next_byte}",
+            range.name
+        );
+        assert!(
+            range.bytes.end > range.bytes.start,
+            "{table_name}「{}」不是空段",
+            range.name
+        );
+        assert!(
+            D13_ROWS_OF_TODAY.contains(&range.row_of_the_d13_field_table)
+                || D13_ROWS_ADDED_BY_THE_WRITE_BACK.contains(&range.row_of_the_d13_field_table),
+            "{table_name}「{}」引的 D13 行名「{}」没登记",
+            range.name,
+            range.row_of_the_d13_field_table
+        );
+        next_byte = range.bytes.end;
+    }
+    assert_eq!(
+        u64::try_from(next_byte).expect("字节数装得进 u64"),
+        width,
+        "{table_name}要恰好盖满 {width} 字节"
+    );
+}
+
+#[test]
+fn the_three_field_byte_tables_cover_the_root_record_the_system_configuration_and_a_tree_table_entry_end_to_end(
+) {
+    assert_the_ranges_cover_the_width_end_to_end(
+        "根记录",
+        &ROOT_RECORD_FIELD_BYTE_RANGES.iter().collect::<Vec<_>>(),
+        singlefs_format::ROOT_RECORD_BYTES,
+    );
+    assert_the_ranges_cover_the_width_end_to_end(
+        "系统配置",
+        &SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES
+            .iter()
+            .map(|field| &field.range)
+            .collect::<Vec<_>>(),
+        singlefs_format::SYSTEM_CONFIGURATION_BYTES,
+    );
+    assert_the_ranges_cover_the_width_end_to_end(
+        "树表条目",
+        &TREE_TABLE_ENTRY_FIELD_BYTE_RANGES
+            .iter()
+            .collect::<Vec<_>>(),
+        singlefs_format::TREE_TABLE_ENTRY_BYTES,
+    );
+    let bytes_of_the_tier = |tier: SystemConfigurationTier| -> usize {
+        SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES
+            .iter()
+            .filter(|field| field.tier == tier)
+            .map(|field| field.range.bytes.len())
+            .sum()
+    };
+    assert_eq!(
+        [
+            bytes_of_the_tier(SystemConfigurationTier::ImmutableConfiguration),
+            bytes_of_the_tier(SystemConfigurationTier::MutableConfiguration),
+            bytes_of_the_tier(SystemConfigurationTier::RuntimeConfiguration),
+            bytes_of_the_tier(SystemConfigurationTier::RuntimeQuantities),
+        ],
+        [389, 4, 36, 60],
+        "系统配置四档之和（D22（单元原子性怎么合成） 已定项 9）"
+    );
+    assert_eq!(
+        SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES.len(),
+        46,
+        "系统配置字段表 46 行，一行一段"
+    );
+}
+
+/// 一段字节翻一个：这一段里第一个非 0 字节改成 0（数值字段这样只会变小，下界那几条才判得出），整段全 0 时第一个字节写 1。
+fn flip_one_byte_of(bytes: &mut [u8], range: std::ops::Range<usize>) {
+    match bytes[range.clone()].iter().position(|byte| *byte != 0) {
+        Some(first_non_zero) => bytes[range.start + first_non_zero] = 0,
+        None => bytes[range.start] = 1,
+    }
+}
+
+/// R2 那一版的根记录翻一字节之后重封整槽自证校验和、照新字节重解一遍（解出来的那几样与原字节一起变）。
+fn flip_one_byte_of_the_root_record_and_reseal(
+    version: &mut VersionFieldsRead,
+    range: std::ops::Range<usize>,
+) {
+    let root_slot_bytes =
+        usize::try_from(SMALL.parameters().geometry.physical_block_size).expect("根槽宽");
+    let record_bytes = version.root_record_bytes.len();
+    let mut slot = version.root_record_bytes.clone();
+    slot.resize(root_slot_bytes, 0);
+    flip_one_byte_of(&mut slot, range);
+    let digest = wide_checksum_with_field_zeroed(&slot, root_slot_bytes, ROOT_CHECKSUM_OFFSET);
+    slot[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
+    let filesystem_identifier: [u8; 16] = slot[4..20].try_into().expect("fsid 16 字节");
+    version.root =
+        RootRecord::parse_slot(&slot, &filesystem_identifier).expect("重封之后这一槽自证得过");
+    version.root_record_bytes = slot[..record_bytes].to_vec();
+}
+
+/// R2 择到的系统配置翻一字节之后重封整槽校验和（罩整槽 4096，字段表之后是补齐的 0）。
+fn flip_one_byte_of_the_system_configuration_and_reseal(
+    fields: &mut RecoveryFieldsRead,
+    range: std::ops::Range<usize>,
+) {
+    let bytes = chosen_system_configuration_bytes_mut(fields);
+    let field_table_bytes = bytes.len();
+    let slot_bytes =
+        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
+    let mut slot = bytes.clone();
+    slot.resize(slot_bytes, 0);
+    flip_one_byte_of(&mut slot, range);
+    let checksum_offset = singlefs_core::system_configuration::SYSTEM_CONFIGURATION_CHECKSUM_OFFSET;
+    let digest = wide_checksum_with_field_zeroed(&slot, slot_bytes, checksum_offset);
+    slot[checksum_offset..checksum_offset + 32].copy_from_slice(&digest);
+    *bytes = slot[..field_table_bytes].to_vec();
+}
+
+/// 字节表的一段翻一字节之后，判官红在它那一行：「必须相同」「必须为 0」的段红在那张表那一样、违例里点名这一段；
+/// 「确定关系」的段红在判它的那一样。
+fn assert_the_judge_reddens_the_row(
+    verdict: &MountTransparency,
+    table_field: MountTransparencyField,
+    range: &FieldByteRange,
+) {
+    let violated: Vec<MountTransparencyField> = verdict
+        .violated_fields()
+        .iter()
+        .map(|field_verdict| field_verdict.field)
+        .collect();
+    match range.class {
+        FieldByteRangeClass::MustBeTheSame | FieldByteRangeClass::MustBeZero => {
+            let Some(FieldOutcome::Violated { observed, .. }) = verdict.outcome_of(table_field)
+            else {
+                panic!(
+                    "「{}」翻了一字节：{table_field:?} 要红：{verdict:#?}",
+                    range.name
+                );
+            };
+            assert!(
+                observed.contains(&format!("\"{}\"", range.name)),
+                "「{}」翻了一字节：违例里点名这一段：{observed}",
+                range.name
+            );
+        }
+        FieldByteRangeClass::DeterminedRelation { judged_by } => assert!(
+            violated.contains(&judged_by),
+            "「{}」翻了一字节：判它的 {judged_by:?} 要红：{violated:?}",
+            range.name
+        ),
+        FieldByteRangeClass::NotCompared { .. } => {
+            panic!("「{}」不比，不翻", range.name)
+        }
+    }
+}
+
+fn is_compared(range: &FieldByteRange) -> bool {
+    match range.class {
+        FieldByteRangeClass::MustBeTheSame
+        | FieldByteRangeClass::MustBeZero
+        | FieldByteRangeClass::DeterminedRelation { .. } => true,
+        FieldByteRangeClass::NotCompared { .. } => false,
+    }
+}
+
+/// 中央映射树根那一段只判空性：带文件的一版上它非零，翻一字节仍非零、判不出，那一段放到树表 0 条的盘面上翻（下一条用例）。
+fn is_judged_by_the_mapping_root_emptiness(range: &FieldByteRange) -> bool {
+    range.class
+        == FieldByteRangeClass::DeterminedRelation {
+            judged_by: MountTransparencyField::MappingRootEmptiness,
+        }
+}
+
+/// 判决第二节 Y2「判别力自证」：带文件的一版上只挂载，三张字节表里每个「必须相同」「必须为 0」「确定关系」的区间各翻一字节、重封校验和，
+/// 判官红在那一行（树表条目翻的是第 0 条，extent 树那一条）。
+// harness-test-granularity:one-scenario 一份盘面上只挂载一次，逐段改的是 R2 读出来的那份原字节，每段从同一份干净的判前字段起改
+#[test]
+fn flipping_one_byte_of_each_compared_range_of_the_three_field_byte_tables_reddens_its_row() {
+    let scenario = mount_alone_on(SMALL, &pool_after_the_first_file());
+    assert!(
+        scenario.judge().violated_fields().is_empty(),
+        "前提：合法挂载上一样都不红"
+    );
+    let mut flipped_ranges = 0usize;
+    for range in ROOT_RECORD_FIELD_BYTE_RANGES
+        .iter()
+        .filter(|range| is_compared(range) && !is_judged_by_the_mapping_root_emptiness(range))
+    {
+        let mut flipped = scenario.clone();
+        flip_one_byte_of_the_root_record_and_reseal(
+            compared_version_mut(&mut flipped.after_the_action),
+            range.bytes.clone(),
+        );
+        assert_the_judge_reddens_the_row(
+            &flipped.judge(),
+            MountTransparencyField::RootRecordBytes,
+            range,
+        );
+        flipped_ranges += 1;
+    }
+    for range in SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES
+        .iter()
+        .map(|field| &field.range)
+        .filter(|range| is_compared(range))
+    {
+        let mut flipped = scenario.clone();
+        flip_one_byte_of_the_system_configuration_and_reseal(
+            &mut flipped.after_the_action,
+            range.bytes.clone(),
+        );
+        assert_the_judge_reddens_the_row(
+            &flipped.judge(),
+            MountTransparencyField::SystemConfigurationBytes,
+            range,
+        );
+        flipped_ranges += 1;
+    }
+    for range in TREE_TABLE_ENTRY_FIELD_BYTE_RANGES
+        .iter()
+        .filter(|range| is_compared(range))
+    {
+        let mut flipped = scenario.clone();
+        let entries = compared_version_mut(&mut flipped.after_the_action)
+            .tree_table_entries
+            .as_mut()
+            .expect("R2 那一版的树表读得出");
+        flip_one_byte_of(&mut entries[0], range.bytes.clone());
+        assert_the_judge_reddens_the_row(
+            &flipped.judge(),
+            MountTransparencyField::TreeTableEntryBytes,
+            range,
+        );
+        flipped_ranges += 1;
+    }
+    assert_eq!(
+        flipped_ranges,
+        ROOT_RECORD_FIELD_BYTE_RANGES
+            .iter()
+            .filter(|range| is_compared(range) && !is_judged_by_the_mapping_root_emptiness(range))
+            .count()
+            + SYSTEM_CONFIGURATION_FIELD_BYTE_RANGES
+                .iter()
+                .filter(|field| is_compared(&field.range))
+                .count()
+            + TREE_TABLE_ENTRY_FIELD_BYTE_RANGES
+                .iter()
+                .filter(|range| is_compared(range))
+                .count(),
+        "每个比的段都翻过"
+    );
+}
+
+/// 中央映射树根那一段放在树表 0 条的盘面上翻（R1、R2 都全零，翻一字节就非零）。
+#[test]
+fn flipping_one_byte_of_the_mapping_root_range_on_a_pool_without_file_reddens_its_row() {
+    let scenario = mount_alone_on(SMALL, &pool_after_make_filesystem());
+    let ranges: Vec<&FieldByteRange> = ROOT_RECORD_FIELD_BYTE_RANGES
+        .iter()
+        .filter(|range| is_judged_by_the_mapping_root_emptiness(range))
+        .collect();
+    assert_eq!(ranges.len(), 1, "根记录表里中央映射树根一段");
+    let mut flipped = scenario.clone();
+    flip_one_byte_of_the_root_record_and_reseal(
+        compared_version_mut(&mut flipped.after_the_action),
+        ranges[0].bytes.clone(),
+    );
+    assert_the_judge_reddens_the_row(
+        &flipped.judge(),
+        MountTransparencyField::RootRecordBytes,
+        ranges[0],
+    );
+}
diff --git a/crates/singlefs-harness/tests/allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file.rs b/crates/singlefs-harness/tests/allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file.rs
new file mode 100644
index 0000000..af8876a
--- /dev/null
+++ b/crates/singlefs-harness/tests/allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file.rs
@@ -0,0 +1,246 @@
+//! 分配器回落那一支的看门（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y3「开放段回落误排」那一行）：
+//! 池级 checker 的 I-5.5 拿树表单元所在的段当这一版的开放段，假定「mkfs 之后连续可写挂载（每次挂载写行、暖机两次零单元发布）、
+//! 再写第一个文件」那一族上提交内生块不走回落（`CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot`）。
+//! 单元区 240 / 256 / 384 槽的小盘、挂载 1、12、60 次，各一条用例断言分配器的回落计数
+//! （`PoolAllocator::commit_generated_placements_that_fell_back_to_the_lowest_free_slot`）是 0；哪天走进回落，用例红，回来重看 I-5.5 的开放段认法。
+//! 阳性对照照 `commit_generated_units_fall_back_when_the_segment_runs_out.rs` 的造法（没有全空段、开放段用满）断言计数大于 0。
+
+mod common;
+
+use common::{build_pool, parameters, FIXED_WRITE_TIME_SECONDS};
+use singlefs_core::address::{DeviceIdentity, InstanceGeneration, SlotNumber};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::allocator::PoolAllocator;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::make_filesystem::MakeFilesystemParameters;
+use singlefs_core::mount::{mount_writable_with_space_admission, Mounted};
+use singlefs_core::transaction::{
+    publish_first_file, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
+};
+use singlefs_format::{CLUSTER_SEGMENT_SLOTS, UNIT_AREA_START_SLOT};
+use singlefs_harness::history::{
+    execute_history_with, GeneratedHistory, HistoryDeviceWidth, HistoryExecution, HistorySeed,
+    HistoryStartingPoint, PerStepChecker,
+};
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+use singlefs_harness::SharedStream;
+
+const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
+
+/// mkfs 之后的整份盘面（随机历史的执行器，起点 `AfterMakeFilesystem`、一步都不走）。
+fn pool_after_make_filesystem(width: HistoryDeviceWidth) -> MemoryPool {
+    let stream = SharedStream::retaining_contents();
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterMakeFilesystem,
+        operations: Vec::new(),
+    };
+    execute_history_with(
+        &history,
+        HistoryExecution {
+            per_step_checker: PerStepChecker::Skipped,
+            device_width: width,
+            space_admission: SpaceAdmission::JudgedByTheFormula,
+        },
+        &stream,
+        &mut |_observation| {},
+    );
+    let mut pool = MemoryPool::with_devices(&DEVICES, width.device_bytes());
+    pool.apply(&stream.retained_operations());
+    pool
+}
+
+fn devices_of(
+    pool: &MemoryPool,
+    parameters: &MakeFilesystemParameters,
+) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
+    pool.devices
+        .iter()
+        .map(|(identity, sectors)| {
+            let mut device = SparseBlockDevice::new(
+                pool.device_size_in_bytes,
+                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
+            );
+            device.image = sectors.clone();
+            (*identity, device)
+        })
+        .collect()
+}
+
+fn pool_of(
+    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
+    device_size_in_bytes: u64,
+) -> MemoryPool {
+    MemoryPool {
+        devices: devices
+            .into_iter()
+            .map(|(identity, device)| (identity, device.image))
+            .collect(),
+        device_size_in_bytes,
+    }
+}
+
+/// 第一个文件的内容（装得进一个数据单元）。
+fn first_file_content() -> Vec<u8> {
+    (0..200u32)
+        .map(|index| u8::try_from(index % 251).expect("小于 251") ^ 0x3c)
+        .collect()
+}
+
+/// mkfs 之后连续可写挂载 `mounts` 次（每次挂载写行、暖机两次零单元发布；前几次挂上就关、不卸载），最后那次挂载之后写第一个文件：
+/// 交回每次挂载交回的分配器（带着挂载里那几次发布）与最后那次写第一个文件的分配器一共走了几次回落。
+fn fallbacks_over_mounts_then_the_first_file(width: HistoryDeviceWidth, mounts: usize) -> u64 {
+    assert!(mounts >= 1, "至少挂载一次才写得了第一个文件");
+    let parameters = width.parameters();
+    let mut pool = pool_after_make_filesystem(width);
+    let mut fallbacks = 0u64;
+    for mount_index in 1..=mounts {
+        let mut devices = devices_of(&pool, &parameters);
+        let Mounted {
+            output,
+            mut allocator,
+            current,
+        } = mount_writable_with_space_admission(
+            &parameters,
+            &mut devices,
+            SpaceAdmission::JudgedByTheFormula,
+        )
+        .unwrap_or_else(|error| panic!("第 {mount_index} 次可写挂载该成：{error:?}"));
+        if mount_index == mounts {
+            let PoolVersion::WithoutFile(current) = &current else {
+                panic!("这一族一直没写文件，现行那一版是树表 0 条的一版：{current:?}");
+            };
+            let content = first_file_content();
+            let mut writer = PoolWriter::new(&parameters, &mut devices);
+            publish_first_file(
+                &mut writer,
+                &mut allocator,
+                &current.root,
+                FirstFile {
+                    content: &content,
+                    write_time_seconds: 1_000,
+                },
+                output.instance,
+                &current.record_bytes,
+            )
+            .expect("最后那次挂载之后写第一个文件该成");
+        }
+        fallbacks += allocator.commit_generated_placements_that_fell_back_to_the_lowest_free_slot();
+        pool = pool_of(devices, pool.device_size_in_bytes);
+    }
+    fallbacks
+}
+
+fn assert_no_fallback(width: HistoryDeviceWidth, mounts: usize) {
+    assert_eq!(
+        fallbacks_over_mounts_then_the_first_file(width, mounts),
+        0,
+        "{width:?} 上挂载 {mounts} 次再写第一个文件，提交内生块一次都不该走回落（I-5.5 的开放段认法压在这上面）"
+    );
+}
+
+#[test]
+fn no_fallback_on_a_unit_area_of_240_slots_after_one_mount_and_the_first_file() {
+    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf240Slots, 1);
+}
+
+#[test]
+fn no_fallback_on_a_unit_area_of_240_slots_after_twelve_mounts_and_the_first_file() {
+    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf240Slots, 12);
+}
+
+#[test]
+fn no_fallback_on_a_unit_area_of_240_slots_after_sixty_mounts_and_the_first_file() {
+    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf240Slots, 60);
+}
+
+#[test]
+fn no_fallback_on_a_unit_area_of_256_slots_after_one_mount_and_the_first_file() {
+    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf256Slots, 1);
+}
+
+#[test]
+fn no_fallback_on_a_unit_area_of_256_slots_after_twelve_mounts_and_the_first_file() {
+    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf256Slots, 12);
+}
+
+#[test]
+fn no_fallback_on_a_unit_area_of_256_slots_after_sixty_mounts_and_the_first_file() {
+    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf256Slots, 60);
+}
+
+#[test]
+fn no_fallback_on_a_unit_area_of_384_slots_after_one_mount_and_the_first_file() {
+    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf384Slots, 1);
+}
+
+#[test]
+fn no_fallback_on_a_unit_area_of_384_slots_after_twelve_mounts_and_the_first_file() {
+    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf384Slots, 12);
+}
+
+#[test]
+fn no_fallback_on_a_unit_area_of_384_slots_after_sixty_mounts_and_the_first_file() {
+    assert_no_fallback(HistoryDeviceWidth::UnitAreaOf384Slots, 60);
+}
+
+/// 每块盘上把开放段剩下的槽占满，再给每个全空段占掉段首一槽（`commit_generated_units_fall_back_when_the_segment_runs_out.rs` 同名函数的造法）。
+fn leave_no_empty_cluster_segment(allocator: &mut PoolAllocator) {
+    let open = allocator.open_segment().expect("发布之后开放段还开着");
+    for device_map in &mut allocator.devices {
+        for slot in open.0..open.0 + CLUSTER_SEGMENT_SLOTS {
+            if device_map.is_free(SlotNumber(slot)) {
+                device_map.mark_allocated(SlotNumber(slot), 1);
+            }
+        }
+        let full_segments = device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS;
+        for segment_index in 0..full_segments {
+            if device_map.segment_is_empty(usize::try_from(segment_index).expect("段号")) {
+                device_map.mark_allocated(
+                    SlotNumber(UNIT_AREA_START_SLOT + segment_index * CLUSTER_SEGMENT_SLOTS),
+                    1,
+                );
+            }
+        }
+    }
+}
+
+/// 阳性对照：新池新建文件之后没有全空段、开放段用满，覆盖写一次——提交内生块走回落，计数大于 0（那一次落点钉在
+/// `commit_generated_units_fall_back_when_the_segment_runs_out.rs` 里）。判别力：回落那一支不加计数时这一条红。
+#[test]
+fn the_fallback_count_is_above_zero_once_no_empty_cluster_segment_is_left() {
+    let mut pool = build_pool("allocator-fallback-count-positive-control");
+    assert_eq!(
+        pool.allocator
+            .commit_generated_placements_that_fell_back_to_the_lowest_free_slot(),
+        0,
+        "前提：新池新建文件那一次没走回落"
+    );
+    leave_no_empty_cluster_segment(&mut pool.allocator);
+    let publish_parameters = parameters();
+    let previous = pool.output.clone();
+    let content: Vec<u8> = (0..4100usize)
+        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
+        .collect();
+    {
+        let devices = pool.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        publish_overwrite(
+            &mut writer,
+            &mut pool.allocator,
+            &previous,
+            FirstFile {
+                content: &content,
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
+            },
+            InstanceGeneration(1),
+        )
+        .expect("没有全空段、每块盘仍有空槽：覆盖写要成功（C369）");
+    }
+    assert!(
+        pool.allocator
+            .commit_generated_placements_that_fell_back_to_the_lowest_free_slot()
+            > 0,
+        "没有全空段时提交内生块走回落，计数要大于 0"
+    );
+}
````

### mutations-append.tsv

````text
26-3 I-9.16 ③ 每种至多一条恒判成立（判决 m3-formal-c655-c687-r1 第二节 Y3）	crates/singlefs-checker/src/walk.rs	        first_kind_carried_more_than_once.is_none(),\n	        true,\n	-p singlefs-harness --test checker_known_bad_images	a_tree_table_whose_deadlist_entry_became_a_second_allocation_entry_reddens_only_the_tree_table_ordering_invariant
26-3 I-9.16 ③ 每种至多一条恒判成立：覆盖写之后那一版的树表（攻方 f2）	crates/singlefs-checker/src/walk.rs	        first_kind_carried_more_than_once.is_none(),\n	        true,\n	-p singlefs-harness --test checker_known_bad_images	the_tree_table_after_the_overwrite_whose_deadlist_entry_became_a_second_allocation_entry_reddens_only_the_tree_table_ordering_invariant
26-3 I-9.16 ③ 每种至多一条恒判成立：livelist 改成第二条 extent（攻方 b2）	crates/singlefs-checker/src/walk.rs	        first_kind_carried_more_than_once.is_none(),\n	        true,\n	-p singlefs-harness --test checker_known_bad_images	a_tree_table_whose_livelist_entry_became_a_second_extent_entry_reddens_only_the_tree_table_ordering_invariant
26-3 I-9.16 ③ 每种至多一条恒判成立：logged_ops 改成第二条 extent（攻方 b3）	crates/singlefs-checker/src/walk.rs	        first_kind_carried_more_than_once.is_none(),\n	        true,\n	-p singlefs-harness --test checker_known_bad_images	a_tree_table_whose_logged_operations_entry_became_a_second_extent_entry_reddens_only_the_tree_table_ordering_invariant
26-3 I-9.16 ④ 种类 1–4 各恰好一条恒判成立：去掉分配记录那一条（攻方 e4）	crates/singlefs-checker/src/walk.rs	    judgements.judge("I-9.16", first_rooted_kind_missing.is_none(), || {\n	    judgements.judge("I-9.16", true, || {\n	-p singlefs-harness --test checker_known_bad_images	a_tree_table_without_its_allocation_entry_reddens_only_the_tree_table_ordering_invariant
26-3 I-9.16 ④ 种类 1–4 各恰好一条恒判成立：分配记录那一条改没登记的种类（正推 e1）	crates/singlefs-checker/src/walk.rs	    judgements.judge("I-9.16", first_rooted_kind_missing.is_none(), || {\n	    judgements.judge("I-9.16", true, || {\n	-p singlefs-harness --test checker_known_bad_images	a_tree_table_whose_allocation_entry_carries_an_unregistered_kind_reddens_only_the_tree_table_ordering_invariant
26-3 I-5.5 没判的那几版不报部分没判：树表读不出（攻方 a1）	crates/singlefs-checker/src/walk.rs	    if !versions_not_judged.is_empty() {\n	    if versions_not_judged.is_empty() && !versions_not_judged.is_empty() {\n	-p singlefs-harness --test checker_known_bad_images	a_tree_table_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged
26-3 I-5.5 没判的那几版不报部分没判：extent 树读不全（攻方 c1）	crates/singlefs-checker/src/walk.rs	    if !versions_not_judged.is_empty() {\n	    if versions_not_judged.is_empty() && !versions_not_judged.is_empty() {\n	-p singlefs-harness --test checker_known_bad_images	an_extent_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged
26-3 I-5.5 没判的那几版不报部分没判：分配记录树读不全（攻方 e3）	crates/singlefs-checker/src/walk.rs	    if !versions_not_judged.is_empty() {\n	    if versions_not_judged.is_empty() && !versions_not_judged.is_empty() {\n	-p singlefs-harness --test checker_known_bad_images	an_allocation_record_tree_that_does_not_read_back_leaves_the_first_data_unit_invariant_partially_judged
26-3 I-5.5 没有数据单元的那块盘不跳过、按槽 0 判（攻方 d1）	crates/singlefs-checker/src/walk.rs	            if let Some(&smallest_slot) = smallest_data_unit_slot_on_each_device.get(device) {\n	            if let Some(&smallest_slot) = smallest_data_unit_slot_on_each_device.get(device).or(Some(&0)) {\n	-p singlefs-harness --test checker_known_bad_images	a_data_pointer_with_no_location_on_one_device_skips_that_device_and_the_first_data_unit_invariant_holds
26-3 I-2.4 根槽 428..457 不判（攻方 m4）	crates/singlefs-checker/src/walk.rs	    judge_root_record_algorithm_nonce_and_mac_are_zero(&mut root_ring_judgements, &roots);\n		-p singlefs-harness --test checker_known_bad_images	a_root_slot_whose_algorithm_type_is_one_reddens_only_the_header_checksum_coverage_invariant
26-3 I-2.4 根槽 428..457 不判（攻方 m5）	crates/singlefs-checker/src/walk.rs	    judge_root_record_algorithm_nonce_and_mac_are_zero(&mut root_ring_judgements, &roots);\n		-p singlefs-harness --test checker_known_bad_images	a_root_slot_whose_nonce_has_a_non_zero_byte_reddens_only_the_header_checksum_coverage_invariant
26-3 I-2.4 根槽 428..457 不判（攻方 m6）	crates/singlefs-checker/src/walk.rs	    judge_root_record_algorithm_nonce_and_mac_are_zero(&mut root_ring_judgements, &roots);\n		-p singlefs-harness --test checker_known_bad_images	a_root_slot_whose_mac_has_a_non_zero_byte_reddens_only_the_header_checksum_coverage_invariant
26-3 I-5.4 带树一版的根记录分配记录树根不判全零	crates/singlefs-checker/src/walk.rs	                        allocation_roots[0].all_zero,\n	                        true,\n	-p singlefs-harness --test checker_known_bad_images	a_file_version_whose_root_record_carries_a_non_zero_allocation_record_tree_root_reddens_the_disjointness_invariant_besides_the_walk
26-3 分配器回落计数不加	crates/singlefs-core/src/allocator.rs	                self.commit_generated_placements_that_fell_back_to_the_lowest_free_slot += 1;\n	                self.commit_generated_placements_that_fell_back_to_the_lowest_free_slot += 0;\n	-p singlefs-harness --test allocator_fallback_count_on_small_devices_after_repeated_mounts_and_the_first_file	the_fallback_count_is_above_zero_once_no_empty_cluster_segment_is_left
26-3 挂载透明 根记录字节表删掉 fsid 那一行	crates/singlefs-harness/src/mount_transparency.rs	    must_be_the_same("fsid", 4..20, D13_IMMUTABLE_CONFIGURATION),\n		-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	the_three_field_byte_tables_cover_the_root_record_the_system_configuration_and_a_tree_table_entry_end_to_end
26-3 挂载透明 续做规则的种类集合退回 {1, 2}	crates/singlefs-harness/src/mount_transparency.rs	    TREE_KIND_INODE,\n    TREE_KIND_DELETED_INODES,\n    TREE_KIND_TOMBSTONE,\n    TREE_KIND_LINEAGE,\n];\n	    TREE_KIND_INODE,\n    TREE_KIND_EXTENT,\n    TREE_KIND_EXTENT,\n    TREE_KIND_EXTENT,\n];\n	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_deleted_inodes_root_pointer_that_moved_with_a_mount_that_did_not_continue_is_red
26-3 挂载透明 journal_tail 去掉上界	crates/singlefs-harness/src/mount_transparency.rs	            && tail_after <= highest_record_counter\n		-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_journal_tail_past_every_record_in_the_ring_of_a_second_crash_image_is_red
26-3 挂载透明 系统配置 journal 实例代号那一行整行跳过	crates/singlefs-harness/src/mount_transparency.rs	                observed.is_some() && observed == root_instance,\n	                true,\n	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_journal_instance_of_the_system_configuration_other_than_the_instance_of_the_root_is_red
26-3 挂载透明 中央映射树根空性恒判成立	crates/singlefs-harness/src/mount_transparency.rs	                    is_all_zero(after) == is_all_zero(before),\n	                    true,\n	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_mapping_root_written_non_zero_by_a_mount_on_a_pool_without_file_is_red
26-3 挂载透明 R1 带树时 R2 分配记录树根不判全零	crates/singlefs-harness/src/mount_transparency.rs	                    is_all_zero(after),\n                    "R1 带树 ⇒ R2 全零".to_string(),\n	                    true,\n                    "R1 带树 ⇒ R2 全零".to_string(),\n	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_non_zero_allocation_record_tree_root_after_a_mount_on_a_file_version_is_red
26-3 挂载透明 系统运行配置不判内置默认	crates/singlefs-harness/src/mount_transparency.rs	                observed == Some(built_in.as_slice()),\n	                true,\n	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	flipping_one_byte_of_each_compared_range_of_the_three_field_byte_tables_reddens_its_row
26-3 挂载透明 字节表「必须相同」那几段不比	crates/singlefs-harness/src/mount_transparency.rs	                before.get(range.bytes.clone()) != after.get(range.bytes.clone())\n                    || after.get(range.bytes.clone()).is_none()\n	                false\n	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	flipping_one_byte_of_each_compared_range_of_the_three_field_byte_tables_reddens_its_row
````

## m3-formal-26-1r

### crates.patch

````diff
diff --git a/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs b/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
index 7e383bb..d45d2e7 100644
--- a/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
+++ b/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
@@ -17914,9 +17914,10 @@ mod fourth_run_segment_two {
                 | HistoryOperation::ColdStartRecover
                 | HistoryOperation::Unlink
                 | HistoryOperation::CreateInode
-                | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(
-                    _,
-                ) => format!("{operation:?}"),
+                | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_)
+                | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
+                    format!("{operation:?}")
+                }
             });
         }
         let source_lines: Vec<&str> = body
@@ -19011,6 +19012,12 @@ mod fourth_run_segment_two {
                 HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(
                     _,
                 ) => not_applicable("OverwriteLosingPowerIsNotDrivenByThisExperiment"),
+                // 每块盘最新那一槽系统配置读回全 0 的可写挂载（C696）只进它自己那一组比重，这个装置用的几组比重不抽它：照前提不满足记。
+                HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
+                    not_applicable(
+                        "WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZerosIsNotDrivenByThisExperiment",
+                    )
+                }
             }
         }
 
diff --git a/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs b/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
index 1313b91..0fa7a50 100644
--- a/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
+++ b/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs
@@ -842,7 +842,9 @@ where
                 // 走到了也只报原样的错，窗口的账不比（推的，没在真设备上造过这一格）。
                 | MountError::FloorRaiseFailedAfterTheMountsPublishes(_)
                 // 读到更新的东西读不出、重读一次仍读不出（C554 乙）：取号之前拒，一个写都没发。
-                | MountError::NewerStateStillUnreadableAfterOneReread(_) => {
+                | MountError::NewerStateStillUnreadableAfterOneReread(_)
+                // 读阶段之后逐盘核或取号那一刻系统配置又读不出（C664）：第一个取号写之前拒，一个写都没发。
+                | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => {
                     vec![describe_run_failure("reopen_and_writable_mount", &cause)]
                 }
             };
@@ -1257,8 +1259,9 @@ fn raise_the_rollback_floor_and_describe<Inner: BlockDevice>(
                 // 现行那一版 txg 到顶：算这一串的 txg 时拒，一个写都没发；可写挂载那一处的抬 F 报错成员抬 F 自己报不出来。
                 | MountError::SequenceNumberPastTheTopOfItsRange(_)
                 | MountError::FloorRaiseFailedAfterTheMountsPublishes(_)
-                // 重读一次仍读不出只由可写挂载交回，抬 F 报不出来。
-                | MountError::NewerStateStillUnreadableAfterOneReread(_) => {
+                // 重读一次仍读不出、读阶段之后系统配置又读不出只由可写挂载交回，抬 F 报不出来。
+                | MountError::NewerStateStillUnreadableAfterOneReread(_)
+                | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => {
                     vec![describe_run_failure("raise_rollback_floor", &cause)]
                 }
             };
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index 8044166..1cef15b 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -1072,11 +1072,13 @@ fn stream_index_of_the_last_write_of_the_crash_window(
     stream_indexes[first_write_of_the_segment + last_write_within_the_crash_window]
 }
 
-/// 这一类历史步在注入的读故障下读盘：崩溃恢复抛弃根（那次可写挂载期间最新那条根的根槽与它点名的单元读回全 0，
-/// `singlefs_harness::history` 的 `DeviceReadingZerosOverHiddenRanges`）。别的步读的就是盘上的字节。
+/// 这一类历史步在注入的读故障下读盘（`singlefs_harness::history` 的 `DeviceReadingZerosOverHiddenRanges`）：崩溃恢复抛弃根（那次可写挂载期间
+/// 最新那条根的根槽与它点名的单元读回全 0），每块盘最新那一槽系统配置读回全 0 的可写挂载（C696（崩溃注入的读故障一支在随机历史里从没触发）：
+/// 随机历史里唯一真的在读故障下往盘上发写的一种步）。别的步读的就是盘上的字节。
 fn operation_reads_under_injected_faults(kind: HistoryOperationKind) -> bool {
     match kind {
-        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => true,
+        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
+        | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => true,
         HistoryOperationKind::PublishFirstFile
         | HistoryOperationKind::PublishOverwrite
         | HistoryOperationKind::PublishWithoutUnits
@@ -1579,18 +1581,27 @@ fn inject_crashes_into_history_with_every_switch(
             None => Err(RemountIdempotenceNotJudged::TheUninterruptedMountWasRefused),
             Some((_pool, fields)) => Ok(content_read_by_the_recovery_report(&fields.report)),
         };
-        judge_mount_transparency_on_the_crash_image(
-            &mut tally,
-            &mut findings,
-            &fields_read_after_the_crash,
-            &mount_run,
-            fields_after_the_uninterrupted_mount
-                .as_ref()
-                .map(|(pool, fields)| (pool, fields)),
-            history.seed,
-            crash_point,
-            image_directory,
-        );
+        // 挂载透明的射程只管不带故障的状态（D13（验证路线） 已定项 1）：注入过读故障的崩溃态不判，按理由记一次没判。
+        match read_faults_in_the_crash_state {
+            ReadFaultsInTheCrashState::NoneInjected => judge_mount_transparency_on_the_crash_image(
+                &mut tally,
+                &mut findings,
+                &fields_read_after_the_crash,
+                &mount_run,
+                fields_after_the_uninterrupted_mount
+                    .as_ref()
+                    .map(|(pool, fields)| (pool, fields)),
+                history.seed,
+                crash_point,
+                image_directory,
+            ),
+            ReadFaultsInTheCrashState::Injected => {
+                *tally
+                    .mount_transparency_not_judged
+                    .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE)
+                    .or_insert(0) += 1;
+            }
+        }
         judge_second_crashes_inside_the_writable_mount(
             &mut tally,
             &mut findings,
@@ -2278,7 +2289,8 @@ fn publishes_before_the_continuation_of_a_mount_that_failed(error: &MountError)
         | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
         | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
         | MountError::SequenceNumberPastTheTopOfItsRange(_)
-        | MountError::NewerStateStillUnreadableAfterOneReread(_) => None,
+        | MountError::NewerStateStillUnreadableAfterOneReread(_)
+        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => None,
     }
 }
 
@@ -2601,6 +2613,10 @@ const MOUNT_TRANSPARENCY_NOT_JUDGED_THE_PUBLISH_AFTER_THE_MOUNT_WAS_REFUSED: &st
 /// 二次崩溃镜像上：没崩过的那一次可写挂载被拒，没有挂载交回值（推没推抬 F）。
 const MOUNT_TRANSPARENCY_NOT_JUDGED_THE_UNINTERRUPTED_MOUNT_WAS_REFUSED: &str =
     "没崩过的那一次可写挂载被拒";
+/// 第一次崩溃那个态注入过读故障（[`StreamMarks::read_faults_in_a_crash_state`]）：挂载透明的射程只管不带故障的状态
+/// （D13（验证路线） 已定项 1），崩溃镜像上那一次挂载整个不判。
+const MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE: &str =
+    "崩溃态注入过读故障（挂载透明只管不带故障的状态）";
 
 /// 挂载透明的一判记进计数：整个动作没判的按理由、某一样没判的按理由、判红的按字段（名字里带量过或推的）；交回判红的那几样，
 /// 一样一条违例（名字是 [`singlefs_harness::mount_transparency::MountTransparencyField::violation_name`]）。
@@ -3575,6 +3591,14 @@ fn conditions_not_met_by_a_remount_refusal_with_the_witness_criterion_true(
         | WritableMountRefusalVerdict::Illegal {
             reason: RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable,
             ..
+        }
+        // 读阶段判完之后系统配置又读不出（C664 之后与判据 N-配置 分开报）：这一截只判 N-配置 那一条（C670），这一拒不判。
+        | WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
+        )
+        | WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
+            ..
         } => return None,
         WritableMountRefusalVerdict::Legal(
             RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
@@ -4799,6 +4823,109 @@ mod tests {
         );
     }
 
+    /// 二次崩溃的种子在段号过 63 之后也两两不同（攻方改法 F3-e 的 A2，判决 `research/prompts/m3-formal-code-b24-r1-main-verification.md`
+    /// 第二节「K1 改法 F3-e」与第三节第 2 条；原型 `research/prompts/m3-formal-code-b24-r1-opus-model/probe/opus-b24-probe/src/main.rs` 的
+    /// `f3_proposed`）：历史种子 0，段号 0..=63 再加 2^k、2^k + 1（k = 6..=20）共 94 个段号，各取长 12 的段上扣下集合非空的 4095 个第一次崩溃态，
+    /// 384930 个种子合成一个集合数，两两不同。真历史里的段号到 210（战役大档），段号 0..=63 那一条看不见只取段号低 6 位的种子。
+    /// 被攻过零轮：这一组取样（段号 2^k、2^k + 1）对更长的历史、别的种子基罩不罩得住没人攻过。
+    /// 判别力：段号只取低 6 位（`crates/mutations.tsv` 一行，攻方 M7d）时段号 64 与 0 撞，这一条红。
+    #[test]
+    fn first_crash_states_of_segments_past_sixty_three_seed_their_second_crashes_differently_across_segments(
+    ) {
+        let segment_indexes: Vec<usize> = (0..=63)
+            .chain((6..=20).flat_map(|exponent| [1_usize << exponent, (1_usize << exponent) + 1]))
+            .collect();
+        let seeds: Vec<SecondCrashPointSeed> = segment_indexes
+            .iter()
+            .flat_map(|segment_index| {
+                seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
+                    HistorySeed(0),
+                    *segment_index,
+                    0,
+                )
+            })
+            .collect();
+        assert_eq!(
+            seeds.len(),
+            94 * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
+            "前提：94 个段号各 4095 个第一次崩溃态，共 384930 个"
+        );
+        let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
+        assert_eq!(
+            distinct_seeds.len(),
+            seeds.len(),
+            "历史种子 0、段号 0..=63 与 2^k、2^k + 1（k = 6..=20）、长 12 的段上 384930 个第一次崩溃态算出的二次崩溃种子两两不同"
+        );
+    }
+
+    /// 长 32 的段里扣下的写落在任何位置都折进种子（攻方改法 F3-e 的 B3，同上判决与原型）：历史种子 0、段号 16、长 32 的段，扣下其中一个写
+    /// （32 种）或两个写（496 种），528 个第一次崩溃态的种子两两不同。上面长 28 的那一条只变段末 12 个写，只折段末 12 个写的指纹在它上面照绿。
+    /// 被攻过零轮：段长 32、只扣一两个写的取样对更长的段、扣得更多的子集罩不罩得住没人攻过。
+    /// 判别力：指纹只折段内末 12 个写（`crates/mutations.tsv` 一行，攻方 M7a）时扣下前 20 个写里哪一个都撞成一个种子，这一条红。
+    #[test]
+    fn first_crash_states_of_a_segment_of_thirty_two_writes_withholding_one_or_two_writes_anywhere_seed_their_second_crashes_differently(
+    ) {
+        const WRITES_OF_THE_SEGMENT: usize = 32;
+        let first_crash_with_withheld = |withheld: &[usize]| CrashPoint {
+            segment_index: 16,
+            persisted_within_the_segment: (0..WRITES_OF_THE_SEGMENT)
+                .map(|write_within_the_segment| !withheld.contains(&write_within_the_segment))
+                .collect(),
+            step: StepPosition::Operation(0),
+            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
+        };
+        let seeds: Vec<SecondCrashPointSeed> = (0..WRITES_OF_THE_SEGMENT)
+            .flat_map(|first_withheld| {
+                std::iter::once(vec![first_withheld]).chain(
+                    (first_withheld + 1..WRITES_OF_THE_SEGMENT)
+                        .map(move |second_withheld| vec![first_withheld, second_withheld]),
+                )
+            })
+            .map(|withheld| {
+                seed_of_the_second_crash_points(
+                    HistorySeed(0),
+                    &first_crash_with_withheld(&withheld),
+                )
+            })
+            .collect();
+        assert_eq!(
+            seeds.len(),
+            528,
+            "前提：长 32 的段扣下一个写 32 种、两个写 496 种"
+        );
+        let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
+        assert_eq!(
+            distinct_seeds.len(),
+            seeds.len(),
+            "历史种子 0、段号 16、长 32 的段上扣下一两个写的 528 个第一次崩溃态算出的二次崩溃种子两两不同"
+        );
+    }
+
+    /// 二次崩溃的种子随历史种子的高位也变（攻方改法 F3-e 的 C2，同上判决与原型）：段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0、1、2^63
+    /// 上各算一遍，12285 个种子两两不同。上面历史种子 0 与 1 那一条只差最低位，只取历史种子最低位的种子在它上面照绿。
+    /// 被攻过零轮：历史种子 0、1、2^63 这三个取样对别的种子基罩不罩得住没人攻过。
+    /// 判别力：历史种子只取最低位（`crates/mutations.tsv` 一行，攻方 M7f）时 0 与 2^63 撞，这一条红。
+    #[test]
+    fn first_crash_states_under_history_seeds_zero_one_and_two_to_the_sixty_third_seed_their_second_crashes_differently(
+    ) {
+        const HISTORY_SEEDS: [HistorySeed; 3] =
+            [HistorySeed(0), HistorySeed(1), HistorySeed(1 << 63)];
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
+            "段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0、1、2^63 上算出的 12285 个二次崩溃种子两两不同"
+        );
+    }
+
     /// 实例表清扫两段（D18（块里携带什么信息） 已定项 11「回收」一条）：挂载那一段的写里，第「清扫之前发了几次」个根槽写之后起是清扫抹头
     /// （FUA 写零的单元头、每块盘一道屏障），之后第一个不带 FUA 的单元写起是清扫发布（单元、屏障、记录、根槽、轮换），两段摆二次崩溃时各摆到一个。
     /// 造的写表：取号；写行那次（单元、记录、根槽、轮换）；一次暖机（单元、根槽）；抹两个头、两道屏障；清扫发布——清扫之前发了 2 次。
@@ -6385,6 +6512,353 @@ mod tests {
         );
     }
 
+    /// 每块盘最新那一槽系统配置读回全 0 时的可写挂载（C696（崩溃注入的读故障一支在随机历史里从没触发））。
+    const NEWEST_SYSTEM_CONFIGURATION_SLOT_ZEROS_MOUNT: HistoryOperationKind =
+        HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros;
+
+    /// 单元区 384 槽的小盘、不逐步跑 checker（设计调查量读故障那一支用的那一档）。
+    fn execution_on_384_slot_devices_without_the_per_step_checker() -> HistoryExecution {
+        use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};
+        HistoryExecution {
+            per_step_checker: PerStepChecker::Skipped,
+            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
+            space_admission: SpaceAdmission::JudgedByTheFormula,
+        }
+    }
+
+    /// C696 判别力（用户定「步表加一条靶向用例」之下的步表那一半；设计调查报告第三节：候选「丁」之下 2676 个注入段里 614 段按写表下标会被认成没注入）：
+    /// 比重取 `GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS`，从这个测试周期的种子基起取前 4 段「每块盘最新那一槽
+    /// 系统配置读回全 0 的可写挂载」往录制流里发过东西的历史（每段 24 步），每段按种子抽 16 个崩溃状态。用例自己再跑一遍同一段历史，照录制流下标
+    /// 独立数「崩溃窗口末尾那个写（录制流下标）之前那一步发过东西」的崩溃状态，崩溃注入的计数（`crash_points_after_a_step_reading_under_injected_faults`）
+    /// 要与它相等；另核这批崩溃状态里按录制流下标算注入过、按写表下标算没注入的至少一个（F5 那一种接错在这批上改得了计数），注入过的至少一个。
+    /// 判别力：读故障认定的崩溃窗口末尾交成写表下标（F5，`crates/mutations.tsv` 一行）时计数少了按写表下标认成没注入的那几个；认定函数恒交
+    /// NoneInjected（一行）时计数是 0——两种都与独立数的不等，这一条红。
+    #[test]
+    fn crash_states_after_a_newest_system_configuration_slot_zeros_mount_count_as_read_faulted_by_the_stream_index_of_the_crash_window(
+    ) {
+        use singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
+        const HISTORIES: usize = 4;
+        const OPERATIONS_PER_HISTORY: usize = 24;
+        const SEEDS_SEARCHED: u64 = 64;
+        let execution = execution_on_384_slot_devices_without_the_per_step_checker();
+        let mut histories_taken = 0usize;
+        let mut counted_by_the_injection = 0u64;
+        let mut read_faulted_by_the_stream_index = 0u64;
+        let mut read_faulted_by_the_stream_index_but_not_by_the_write_index = 0u64;
+        // 迭代上界是 SEEDS_SEARCHED 个种子；跨轮携带的是四样计数；取够 HISTORIES 段就停。
+        for seed_offset in 0..SEEDS_SEARCHED {
+            if histories_taken == HISTORIES {
+                break;
+            }
+            let history = generate_history_with_weights(
+                HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + seed_offset),
+                OPERATIONS_PER_HISTORY,
+                &GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS,
+            );
+            let stream = SharedStream::retaining_contents();
+            let mut operations_after_each_step: Vec<(StepPosition, usize)> = Vec::new();
+            execute_history_with(&history, execution, &stream, &mut |observation| {
+                operations_after_each_step.push((observation.position, stream.operation_count()));
+            });
+            // 读故障下发过东西的那几步各自的第一个录制流下标。
+            let mut operations_before_the_step = 0usize;
+            let mut first_stream_index_of_each_read_faulted_step: Vec<usize> = Vec::new();
+            for (position, operations_after_the_step) in &operations_after_each_step {
+                if let StepPosition::Operation(step_index) = position {
+                    if history.operations[*step_index].kind()
+                        == NEWEST_SYSTEM_CONFIGURATION_SLOT_ZEROS_MOUNT
+                        && operations_before_the_step < *operations_after_the_step
+                    {
+                        first_stream_index_of_each_read_faulted_step
+                            .push(operations_before_the_step);
+                    }
+                }
+                operations_before_the_step = *operations_after_the_step;
+            }
+            if first_stream_index_of_each_read_faulted_step.is_empty() {
+                continue;
+            }
+            let (_writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
+                &stream.retained_operations(),
+                &execution.device_width.fixed_geometry(),
+            );
+            let read_faulted_by_the_end_of_the_window = |last_index_of_the_window: usize| {
+                first_stream_index_of_each_read_faulted_step
+                    .iter()
+                    .any(|first_stream_index| *first_stream_index <= last_index_of_the_window)
+            };
+            let injection = inject_crashes_into_history(
+                &history,
+                execution,
+                CrashPointDraw::Sampled {
+                    crash_points_per_history: 16,
+                },
+                None,
+            );
+            for crash_point in &injection.crash_points {
+                let last_write_of_the_window = segments[crash_point.segment_index][0]
+                    + crash_point.persisted_within_the_segment.len()
+                    - 1;
+                if read_faulted_by_the_end_of_the_window(stream_indexes[last_write_of_the_window]) {
+                    read_faulted_by_the_stream_index += 1;
+                    if !read_faulted_by_the_end_of_the_window(last_write_of_the_window) {
+                        read_faulted_by_the_stream_index_but_not_by_the_write_index += 1;
+                    }
+                }
+            }
+            counted_by_the_injection += injection
+                .tally
+                .crash_points_after_a_step_reading_under_injected_faults;
+            histories_taken += 1;
+        }
+        assert_eq!(
+            histories_taken, HISTORIES,
+            "前提：种子基起 {SEEDS_SEARCHED} 段里有 {HISTORIES} 段的那一步往录制流里发过东西"
+        );
+        assert!(
+            read_faulted_by_the_stream_index >= 1,
+            "前提：抽到的崩溃状态里至少一个在那一步发过东西之后"
+        );
+        assert!(
+            read_faulted_by_the_stream_index_but_not_by_the_write_index >= 1,
+            "前提：至少一个崩溃状态按录制流下标算注入过、按写表下标算没注入（F5 那一种接错在这批上改得了计数）"
+        );
+        assert_eq!(
+            counted_by_the_injection, read_faulted_by_the_stream_index,
+            "崩溃注入认定的注入过读故障的崩溃状态数，与按录制流下标独立数的相等（其中 {read_faulted_by_the_stream_index_but_not_by_the_write_index} 个按写表下标会认成没注入）"
+        );
+    }
+
+    /// 起点（取号、暖机、第一个文件）之后：一次「每块盘最新那一槽系统配置读回全 0 的可写挂载」（读故障下发写，C696），再一次覆盖写。
+    fn history_of_a_newest_system_configuration_slot_zeros_mount_and_one_overwrite(
+    ) -> GeneratedHistory {
+        use singlefs_harness::history::{ContentChoice, ContentLength};
+        GeneratedHistory {
+            seed: HistorySeed(0),
+            starting_point: HistoryStartingPoint::AfterFirstFile,
+            operations: vec![
+                HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
+                HistoryOperation::PublishOverwrite(ContentChoice {
+                    length: ContentLength::InsideOneDataUnit { selector: 0 },
+                    fill_seed: 1,
+                }),
+            ],
+        }
+    }
+
+    /// 那段历史的写表与切段（与 [`inject_crashes_into_history`] 自己切的同一份），覆盖写那次发布的第一个写在写表里的下标；
+    /// 另核读回全 0 的那次可写挂载往录制流里发过东西、两步都做成。
+    fn writes_segments_and_first_write_of_the_overwrite_after_the_zeros_mount(
+        history: &GeneratedHistory,
+        execution: HistoryExecution,
+    ) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, usize) {
+        let stream = SharedStream::retaining_contents();
+        let mut operations_after_each_step: Vec<(StepPosition, usize)> = Vec::new();
+        let run = execute_history_with(history, execution, &stream, &mut |observation| {
+            operations_after_each_step.push((observation.position, stream.operation_count()));
+        });
+        assert_eq!(run.ending, HistoryEnding::Completed, "前提：这段历史跑完");
+        let [(StepPosition::StartingPoint, after_the_starting_point), (StepPosition::Operation(0), after_the_zeros_mount), (StepPosition::Operation(1), _)] =
+            operations_after_each_step[..]
+        else {
+            panic!("前提：起点与两步各登记一次：{operations_after_each_step:?}");
+        };
+        assert!(
+            after_the_starting_point < after_the_zeros_mount,
+            "前提：读回全 0 的那次可写挂载往录制流里发过东西（取号、写行、暖机）"
+        );
+        let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
+            &stream.retained_operations(),
+            &execution.device_width.fixed_geometry(),
+        );
+        let first_write_of_the_overwrite = stream_indexes
+            .iter()
+            .position(|stream_index| *stream_index >= after_the_zeros_mount)
+            .expect("覆盖写那次发布写过东西");
+        (writes, segments, first_write_of_the_overwrite)
+    }
+
+    /// C696 与 D13（验证路线） 已定项 1 挂载透明的射程（只管不带故障的状态）：第一次崩溃摆在覆盖写那次发布的第一个写上（它一个写都没落，
+    /// 盘面停在读回全 0 的那次可写挂载做成之后），这个态注入过读故障，崩溃镜像上那一次可写挂载做成，挂载透明整个不判、按理由记一次没判。
+    /// 判别力：注入态上也判挂载透明（`crates/mutations.tsv` 一行）时判过的不为空、没判的那一条理由是 0，这一条红。
+    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态（ExactlyOneCrashState），不枚举
+    #[test]
+    fn mount_transparency_is_not_judged_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount(
+    ) {
+        let history = history_of_a_newest_system_configuration_slot_zeros_mount_and_one_overwrite();
+        let execution = execution_on_384_slot_devices_without_the_per_step_checker();
+        let (_writes, segments, first_write_of_the_overwrite) =
+            writes_segments_and_first_write_of_the_overwrite_after_the_zeros_mount(
+                &history, execution,
+            );
+        let overwrite_segment_index = segments
+            .iter()
+            .position(|segment| segment[0] == first_write_of_the_overwrite)
+            .expect("覆盖写那次发布从一段的第一个写起（它前面是挂载最后那道屏障）");
+        let injection = inject_crashes_into_history(
+            &history,
+            execution,
+            CrashPointDraw::ExactlyOneCrashState {
+                segment_index: overwrite_segment_index,
+                writes_from_the_segment_on: 1,
+                withheld_among_them: &[0],
+            },
+            None,
+        );
+        let tally = &injection.tally;
+        assert_eq!(
+            (
+                tally.crash_points,
+                tally.crash_points_after_a_step_reading_under_injected_faults,
+                tally.writable_mounts_after_the_crash_succeeded
+            ),
+            (1, 1, 1),
+            "前提：只摆写死的那一个第一次崩溃态，它注入过读故障，它上面的可写挂载做成：{}",
+            tally.render()
+        );
+        assert_eq!(
+            (
+                tally.mount_transparency_judged.clone(),
+                tally
+                    .mount_transparency_not_judged
+                    .get(MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE)
+                    .copied()
+            ),
+            (BTreeMap::new(), Some(1)),
+            "注入过读故障的崩溃态上挂载透明一个动作都不判，按理由记一次：{}",
+            tally.render()
+        );
+    }
+
+    /// C696「第三截调用点写死成 NoneInjected」那一种接错的靶向用例（用户定「步表加一条靶向用例」；W 为真的阳性对照，造法同
+    /// [`a_second_crash_where_only_the_rotation_of_the_row_publish_landed_is_an_illegal_witness_refusal_through_the_call_site`]）：第一次崩溃摆在
+    /// 「每块盘最新那一槽系统配置读回全 0 的可写挂载」之后那次覆盖写的第一个写上（注入过读故障的态），第二截那次可写挂载的写行那次发布上摆一个
+    /// 二次崩溃——根槽 FUA 没落、紧跟着的两槽系统配置轮换落了。二次崩溃镜像上再挂以判据 N-配置 拒、判据复算为真；崩溃态注入过读故障，
+    /// 第三截判过一次、不判 C670 那一条（不合法 0 次、没有新发现）。
+    /// 判别力：第三截调用点把读故障的认定写死成 NoneInjected（`crates/mutations.tsv` 一行）时不合法变 1、报一条新发现，这一条红。
+    /// 随机历史上那一种接错是等价变异（设计调查报告第三节：注入态与第三截二次崩溃态上判据从没为真），等价性留档在
+    /// `crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs`。
+    // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
+    #[test]
+    fn a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount_is_not_judged_by_c670(
+    ) {
+        let history = history_of_a_newest_system_configuration_slot_zeros_mount_and_one_overwrite();
+        let execution = execution_on_384_slot_devices_without_the_per_step_checker();
+        let (writes, segments, first_write_of_the_overwrite) =
+            writes_segments_and_first_write_of_the_overwrite_after_the_zeros_mount(
+                &history, execution,
+            );
+        let overwrite_segment_index = segments
+            .iter()
+            .position(|segment| segment[0] == first_write_of_the_overwrite)
+            .expect("覆盖写那次发布从一段的第一个写起（它前面是挂载最后那道屏障）");
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
+                step: StepPosition::Operation(1),
+                operation_kind: Some(HistoryOperationKind::PublishOverwrite),
+            },
+        );
+        assert!(
+            matches!(mount_run.outcome, WritableMountOutcome::Published),
+            "前提：读回全 0 的那次挂载之后那一版上的可写挂载与之后那一次覆盖写都做成：{:?}",
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
+            (
+                tally.crash_points,
+                tally.crash_points_after_a_step_reading_under_injected_faults,
+                tally.writable_mounts_after_the_crash_succeeded
+            ),
+            (1, 1, 1),
+            "前提：只摆写死的那一个第一次崩溃态，它注入过读故障，它上面的可写挂载做成：{}",
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
+            (1, 0),
+            "二次崩溃镜像上再挂以判据 N-配置 拒、判过一次；第一次崩溃那个态注入过读故障，不判 C670 那一条：{:?}",
+            injection
+                .new_findings
+                .iter()
+                .map(CrashPointFinding::render)
+                .collect::<Vec<_>>()
+        );
+    }
+
     /// 起点（取号、暖机、第一个文件）之后一次覆盖写、不重挂的历史，与跑它用的执行（单元区 384 槽的小盘，不逐步跑 checker）。
     fn history_of_the_first_file_and_one_overwrite() -> (GeneratedHistory, HistoryExecution) {
         use singlefs_harness::history::{HistoryDeviceWidth, PerStepChecker};
diff --git a/crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs b/crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs
new file mode 100644
index 0000000..0d664b5
--- /dev/null
+++ b/crates/singlefs-checker-tier/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs
@@ -0,0 +1,226 @@
+//! checker 档模块：无（自己逐个造崩溃状态，只用 harness 档的随机历史、内存池、可写挂载与挂载透明判官）
+//! C696（崩溃注入的读故障一支在随机历史里从没触发）判别力自证里「第三截调用点写死成 NoneInjected」那一种接错，在随机历史上是等价变异
+//! （第 26 批 C696 设计调查报告第三节）：写死之后只在三样都成立的态上改判定——第一次崩溃那一态注入过读故障、
+//! 它上面那次可写挂载途中的二次崩溃镜像上再挂以判据 N-配置 拒、那个二次崩溃镜像上判据 W（`judge_writable_mount_refusal_of_a_crash_state`
+//! 没注入读故障时报的 C670 那一条）为真。这里照 `.claude/singlefs-ai-sop/rules/test-discipline.md`「变异测试证明的是断言会红，不是覆盖」
+//! 第 2 点把等价性写成一条测试留档：在「每块盘最新那一槽系统配置读回全 0 的可写挂载」那一组比重的历史上，那一步发过写之后的每个崩溃态
+//! （段内前缀）与它们上面可写挂载的每个写前缀（第三截的二次崩溃镜像），W 一个都不为真。真会改行为的那一种接错由
+//! `crash_injection::tests::a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount_is_not_judged_by_c670`
+//! 那条靶向用例钉（W 为真的阳性对照）。
+//! 推翻「等价」的观测：这里任一个态上 W 为真。量的范围：一组比重、一档盘宽（单元区 384 槽）、种子基起前 4 段那一步发过写的历史、
+//! 段内只摆前缀、第三截只摆挂载写的前缀；更宽的范围没量。
+
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::mount::{
+    mount_writable_with_space_admission, MountError, NewerPublishWitness, RollbackTarget,
+    SelectedVersionAgainstTheWitness, StillUnreadableAfterOneReread, WitnessedCounterComparison,
+};
+use singlefs_harness::history::{
+    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
+    HistoryExecution, HistoryOperationKind, HistorySeed, PerStepChecker, StepPosition,
+    SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
+};
+use singlefs_harness::memory_pool::{
+    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
+};
+use singlefs_harness::mount_transparency::{
+    judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields, ComparedVersion,
+    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage,
+    RefusalConditionNotMet, WritableMountRefusalVerdict,
+};
+use singlefs_harness::{RecordingBlockDevice, SharedStream};
+
+const WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
+const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
+const HISTORIES: usize = 4;
+const SEEDS_SEARCHED: u64 = 64;
+const OPERATIONS_PER_HISTORY: usize = 24;
+
+/// 判据 N-配置 为真、重读仍为真的一拒（判官自己在镜像上复算判据，不看这里填的读数）。
+fn witness_refusal_claimed_on(selected_version: (InstanceGeneration, CheckpointTxg)) -> MountError {
+    let reading = SelectedVersionAgainstTheWitness {
+        selected_version: RollbackTarget {
+            instance: selected_version.0,
+            checkpoint_txg: selected_version.1,
+        },
+        witness: NewerPublishWitness {
+            witnessed_journal_counter: 1,
+            comparison: WitnessedCounterComparison::Undecidable,
+        },
+    };
+    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
+        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
+            first_read: reading,
+            reread: reading,
+        },
+    ))
+}
+
+/// W：这个镜像当作没注入读故障的崩溃态，判官对一次判据 N-配置 的拒报不报 C670 那一条（判据在镜像上复算、撕裂的系统配置槽跳过仍为真）。
+/// 只读恢复落不到一版时判据无从复算，交回假。
+fn witness_criterion_holds_without_read_faults(image: &MemoryPool) -> bool {
+    let fields = read_recovery_fields(image, ComparedVersion::TheRecoveredVersion);
+    let Some(selected_version) = fields.report.effective_root else {
+        return false;
+    };
+    match judge_writable_mount_refusal_of_a_crash_state(
+        image,
+        fields.report.effective_root,
+        &witness_refusal_claimed_on(selected_version),
+        DiskAfterTheRefusal::ByteForByteUnchanged,
+        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+        ReadFaultsInTheCrashState::NoneInjected,
+    ) {
+        WritableMountRefusalVerdict::Illegal {
+            conditions_not_met, ..
+        } => conditions_not_met.iter().any(|condition| {
+            matches!(
+                condition,
+                RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(_)
+            )
+        }),
+        WritableMountRefusalVerdict::Legal(_)
+        | WritableMountRefusalVerdict::ReasonNotOnTheD23List => false,
+    }
+}
+
+/// 崩溃镜像上起一次可写挂载（与崩溃注入第二截同一种做法），交回挂载写表（做成与否都交，被拒时是空的）。
+fn writes_of_a_writable_mount_on(image: &MemoryPool) -> Vec<RetainedWrite> {
+    let parameters = WIDTH.parameters();
+    let stream = SharedStream::retaining_contents();
+    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = image
+        .devices
+        .iter()
+        .map(|(identity, sectors)| {
+            let mut device = SparseBlockDevice::new(
+                image.device_size_in_bytes,
+                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
+            );
+            device.image = sectors.clone();
+            (
+                *identity,
+                RecordingBlockDevice::with_shared_stream(*identity, device, stream.clone()),
+            )
+        })
+        .collect();
+    let _mounted = mount_writable_with_space_admission(
+        &parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+    );
+    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
+        &stream.retained_operations(),
+        &WIDTH.fixed_geometry(),
+    );
+    writes
+}
+
+/// 场景：那一组比重、种子基起前 4 段「每块盘最新那一槽系统配置读回全 0 的可写挂载」发过写的历史，每段 24 步。崩溃态取那一步发的第一个写
+/// 之后结束的每一段的每个前缀（崩溃窗口末尾在那一步之后，崩溃注入认成注入过读故障的那一类）；每段最长那个前缀上再起一次可写挂载，
+/// 挂载写的每个前缀是一个第三截的二次崩溃镜像。
+/// 预期：这些镜像上 W 一个都不为真（写死 NoneInjected 与正确接线在这批态上判出一样的东西）；崩溃态与二次崩溃镜像各至少一个（不是空判）。
+#[test]
+#[ignore = "checker 档：自己逐个摆崩溃态复算判据，release 下跑"]
+fn read_faulted_crash_states_and_their_second_crashes_never_hold_the_witness_criterion() {
+    let execution = HistoryExecution {
+        per_step_checker: PerStepChecker::Skipped,
+        device_width: WIDTH,
+        space_admission: SpaceAdmission::JudgedByTheFormula,
+    };
+    let mut histories_taken = 0usize;
+    let mut read_faulted_crash_states = 0u64;
+    let mut second_crash_images = 0u64;
+    let mut images_holding_the_witness_criterion: Vec<String> = Vec::new();
+    // 迭代上界是 SEEDS_SEARCHED 个种子；跨轮携带三样计数与 W 为真的态的描述；取够 HISTORIES 段就停。
+    for seed_offset in 0..SEEDS_SEARCHED {
+        if histories_taken == HISTORIES {
+            break;
+        }
+        let seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + seed_offset;
+        let history = generate_history_with_weights(
+            HistorySeed(seed),
+            OPERATIONS_PER_HISTORY,
+            &GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS,
+        );
+        let stream = SharedStream::retaining_contents();
+        let mut operations_after_each_step: Vec<(StepPosition, usize)> = Vec::new();
+        execute_history_with(&history, execution, &stream, &mut |observation| {
+            operations_after_each_step.push((observation.position, stream.operation_count()));
+        });
+        let mut operations_before_the_step = 0usize;
+        let mut first_stream_index_of_the_step = None;
+        for (position, operations_after_the_step) in &operations_after_each_step {
+            if let StepPosition::Operation(step_index) = position {
+                if first_stream_index_of_the_step.is_none()
+                    && history.operations[*step_index].kind()
+                        == HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
+                    && operations_before_the_step < *operations_after_the_step
+                {
+                    first_stream_index_of_the_step = Some(operations_before_the_step);
+                }
+            }
+            operations_before_the_step = *operations_after_the_step;
+        }
+        let Some(first_stream_index_of_the_step) = first_stream_index_of_the_step else {
+            continue;
+        };
+        let operations_after_the_last_step = operations_before_the_step;
+        let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
+            &stream.retained_operations(),
+            &WIDTH.fixed_geometry(),
+        );
+        let mut base = MemoryPool::with_devices(&DEVICES, WIDTH.device_bytes());
+        let mut writes_applied_to_the_base = 0usize;
+        for segment in &segments {
+            let first_write = segment[0];
+            let last_stream_index = stream_indexes[*segment.last().expect("段非空")];
+            if last_stream_index >= operations_after_the_last_step {
+                break;
+            }
+            base.apply_writes(&writes[writes_applied_to_the_base..first_write]);
+            writes_applied_to_the_base = first_write;
+            if last_stream_index < first_stream_index_of_the_step {
+                continue;
+            }
+            let mut longest_prefix = base.clone();
+            for persisted_writes in 0..segment.len() {
+                let mut image = base.clone();
+                image.apply_writes(&writes[first_write..first_write + persisted_writes]);
+                read_faulted_crash_states += 1;
+                if witness_criterion_holds_without_read_faults(&image) {
+                    images_holding_the_witness_criterion.push(format!(
+                        "种子 {seed} 第一次崩溃：段首写 {first_write}、落了 {persisted_writes} 个"
+                    ));
+                }
+                longest_prefix = image;
+            }
+            let mount_writes = writes_of_a_writable_mount_on(&longest_prefix);
+            for persisted_mount_writes in 0..mount_writes.len() {
+                let mut second_crash_image = longest_prefix.clone();
+                second_crash_image.apply_writes(&mount_writes[..persisted_mount_writes]);
+                second_crash_images += 1;
+                if witness_criterion_holds_without_read_faults(&second_crash_image) {
+                    images_holding_the_witness_criterion.push(format!(
+                        "种子 {seed} 第三截：段首写 {first_write}、挂载写落了 {persisted_mount_writes} 个"
+                    ));
+                }
+            }
+        }
+        histories_taken += 1;
+    }
+    assert_eq!(
+        histories_taken, HISTORIES,
+        "前提：种子基起 {SEEDS_SEARCHED} 段里有 {HISTORIES} 段那一步发过写"
+    );
+    assert!(
+        read_faulted_crash_states >= 1 && second_crash_images >= 1,
+        "前提：摆到了注入过读故障的崩溃态 {read_faulted_crash_states} 个、第三截二次崩溃镜像 {second_crash_images} 个"
+    );
+    assert_eq!(
+        images_holding_the_witness_criterion,
+        Vec::<String>::new(),
+        "注入过读故障的崩溃态 {read_faulted_crash_states} 个、二次崩溃镜像 {second_crash_images} 个上 W 都不为真（写死 NoneInjected 是等价变异）"
+    );
+}
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 769b294..7c8e4ba 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -311,6 +311,36 @@ pub enum MountError {
     /// 盘上逐字节不变、分配器与调用方的现行版本不动。
     /// 装箱：不装箱 `MountError` 就大过 clippy `result_large_err` 的 128 字节。
     NewerStateStillUnreadableAfterOneReread(Box<StillUnreadableAfterOneReread>),
+    /// 读阶段判完、判据 N-配置 为假之后，取号之前的逐盘核或取号那一刻读系统配置又读出判不出（R2）或某块盘一份本池自证过的都没有
+    /// （D23（journal 的角色与格式） 已定项 14「取号那一刻按同一个判不出（R2）判：判不出就在第一个取号写之前拒……这一步在某块盘一份自证过的
+    /// 系统配置都没有时也拒」）：拒可写。与读阶段判据 N-配置 那一拒（`NewerStateStillUnreadableAfterOneReread`）分开报——两遍读之间的瞬时读错
+    /// 在崩溃镜像上复算不出，并成一个拒因时挂载透明在带读故障的档里会把这一种合法拒判成不合法（C664（挂载拒绝合法复算把两种拒因报成一个）；
+    /// D13（验证路线） 已定项 1 挂载透明射程那一句）。一个字节都没写，盘上逐字节不变。
+    /// 装箱：不装箱 `MountError` 就大过 clippy `result_large_err` 的 128 字节。
+    SystemConfigurationUnreadableAfterTheReadStage(
+        Box<SystemConfigurationUnreadableAfterTheReadStage>,
+    ),
+}
+
+/// 读阶段判完之后系统配置又读不出（[`MountError::SystemConfigurationUnreadableAfterTheReadStage`]）：读阶段往下走用的那一遍读数，与之后读到的是哪一样。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct SystemConfigurationUnreadableAfterTheReadStage {
+    /// 读阶段往下走用的那一遍（判据 N-配置 在这一遍为假）：它的所选那一版与见证读数。
+    pub read_stage: SelectedVersionAgainstTheWitness,
+    pub unreadable: SystemConfigurationUnreadableWhen,
+}
+
+/// 读阶段判完之后在哪一处、读到系统配置的哪一种读不出。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum SystemConfigurationUnreadableWhen {
+    /// 取号之前的逐盘核（落后支要读每块盘自证过、世代号最大的那一份）读到 `device` 两槽里一份本池自证过的都没有。
+    CheckingEachDeviceBeforeTheAcquisition { device: DeviceIdentity },
+    /// 取号那一刻读见证值时判不出（R2：有槽设备报读错，且没有一块盘两槽都自证过）；`first_device_reporting_a_read_error` 是盘表次序第一块报读错的盘。
+    AcquisitionReadsAnUndecidableWitness {
+        first_device_reporting_a_read_error: DeviceIdentity,
+    },
+    /// 取号那一刻读见证值时 `device` 两槽里一份本池自证过的都没有。
+    AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot { device: DeviceIdentity },
 }
 
 /// 可写挂载重读一次之后仍读不出的是哪一样（[`MountError::NewerStateStillUnreadableAfterOneReread`]）。
@@ -415,8 +445,7 @@ impl SelectedVersionAgainstTheWitness {
                     )
             }
             WitnessedCounterComparison::Undecidable
-            | WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots
-            | WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot => {
+            | WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots => {
                 true
             }
         }
@@ -450,10 +479,6 @@ pub enum WitnessedCounterComparison {
     /// c_见证 可能缺了那一槽，按判据为真处置（C331（择根倒挂压过已确认的写） 改取 R2，用户 2026-10-03 定，零轮；
     /// D22（单元原子性怎么合成） 已定项 8 射程）。撕裂的系统配置轮换（读得出、自证不过，设备没报错）不落在这里，照常取自证过的槽比。
     SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots,
-    /// 读阶段判完之后，逐盘核或取号那一遍读到某块盘两槽里一份本池自证过的系统配置都没有（C554 乙-配置续 Q1：这块盘不「可见」，
-    /// D18（块里携带什么信息） 已定项 11）：只由 [`refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage`]
-    /// 报，按判据为真处置。R2 之下这一支不并进判不出（一块盘两槽都读不出、另一块盘两槽都自证过时 R2 判得出），单列一个成员。
-    SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot,
 }
 
 /// 可写挂载在「重读一次」之前做什么（D16（发布语义） 已定项 1「根槽这一次读坏」那一行：这一次读不出就重读一次，仍坏就拒）。
@@ -2810,8 +2835,10 @@ fn floor_raise_refused_for_space(refusal: &MountError) -> bool {
         // txg 到顶不是空间不够；可写挂载那一处抬 F 的错装进的那个成员只由可写挂载交回、抬 F 自己报不出来。
         | MountError::SequenceNumberPastTheTopOfItsRange(_)
         | MountError::FloorRaiseFailedAfterTheMountsPublishes(_)
-        // 重读一次仍读不出只由可写挂载的读阶段与重建分配器交回（C554 乙），抬 F 报不出来，也不是空间不够。
-        | MountError::NewerStateStillUnreadableAfterOneReread(_) => false,
+        // 重读一次仍读不出只由可写挂载的读阶段与重建分配器交回（C554 乙），读阶段之后系统配置又读不出只由可写挂载取号之前与取号交回：
+        // 抬 F 报不出来，也不是空间不够。
+        | MountError::NewerStateStillUnreadableAfterOneReread(_)
+        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => false,
     }
 }
 
@@ -3875,28 +3902,21 @@ fn devices_without_the_selected_version<Reader: PoolReader + ?Sized>(
     Ok(devices_without)
 }
 
-/// 读阶段的见证读判完之后，逐盘核或取号那一遍又读到判不出（R2）或某块盘一份自证过的系统配置都没有（两次读之间的瞬时读错）：
-/// 照判据 N-配置 为真、重读仍真的拒因报（用户 2026-09-28 定：这一支并进见证读的拒因）——前一遍是读阶段往下走用的那一遍读数，
-/// 后一遍是同一个见证值、判的是 `comparison`（[`WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots`]
-/// 或 [`WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot`]）。一个字节都没写。
+/// 读阶段的见证读判完之后，逐盘核或取号那一遍又读到判不出（R2）或某块盘一份自证过的系统配置都没有（两次读之间的瞬时读错就够）：
+/// 拒可写，报 [`MountError::SystemConfigurationUnreadableAfterTheReadStage`]，带读阶段往下走用的那一遍读数与 `unreadable`。
+/// 不并进读阶段判据 N-配置 那一拒（C664（挂载拒绝合法复算把两种拒因报成一个））。一个字节都没写。
 fn refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
     read_stage: ReadStageSettled,
-    comparison: WitnessedCounterComparison,
+    unreadable: SystemConfigurationUnreadableWhen,
 ) -> MountError {
     let settled = match read_stage {
         ReadStageSettled::OnTheFirstRead { first_read } => first_read,
         ReadStageSettled::OnTheOneReread { reread, .. } => reread,
     };
-    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
-        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
-            first_read: settled,
-            reread: SelectedVersionAgainstTheWitness {
-                selected_version: settled.selected_version,
-                witness: NewerPublishWitness {
-                    witnessed_journal_counter: settled.witness.witnessed_journal_counter,
-                    comparison,
-                },
-            },
+    MountError::SystemConfigurationUnreadableAfterTheReadStage(Box::new(
+        SystemConfigurationUnreadableAfterTheReadStage {
+            read_stage: settled,
+            unreadable,
         },
     ))
 }
@@ -4609,10 +4629,10 @@ fn establish_instance<Device: BlockDevice>(
         &units_of_the_selected_version,
         root_of_the_selected_version,
     )
-    .map_err(|_device| {
+    .map_err(|device| {
         refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
             start.rereads.read_stage,
-            WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot,
+            SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition { device },
         )
     })?;
     if !devices_without.is_empty() {
@@ -4642,18 +4662,22 @@ fn establish_instance<Device: BlockDevice>(
                 SequenceNumberAtTheTopOfItsRange::InstanceGeneration(highest_on_disk),
             ),
             // 读阶段的见证读判得出、取号那一刻读见证值时判不出（R2）或这块盘一份自证过的系统配置都没有（两次读之间的瞬时读错，
-            // C554 乙-配置续 Q1）：照见证读的拒因报（用户 2026-09-28 定）。一个字节都没写。
+            // C554 乙-配置续 Q1）：拒可写，与读阶段判据 N-配置 那一拒分开报（C664）。一个字节都没写。
             ExpectedInstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
-                first_device_reporting_a_read_error: _,
+                first_device_reporting_a_read_error,
             } => refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
                 start.rereads.read_stage,
-                WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots,
+                SystemConfigurationUnreadableWhen::AcquisitionReadsAnUndecidableWitness {
+                    first_device_reporting_a_read_error,
+                },
             ),
             ExpectedInstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
-                device: _,
+                device,
             } => refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
                 start.rereads.read_stage,
-                WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot,
+                SystemConfigurationUnreadableWhen::AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot {
+                    device,
+                },
             ),
             ExpectedInstanceAcquisitionFailed::Acquisition(acquisition) => {
                 MountError::Acquisition(acquisition)
diff --git a/crates/singlefs-harness/src/fault_injection.rs b/crates/singlefs-harness/src/fault_injection.rs
index 3306a61..bb0db51 100644
--- a/crates/singlefs-harness/src/fault_injection.rs
+++ b/crates/singlefs-harness/src/fault_injection.rs
@@ -1810,10 +1810,12 @@ fn system_configuration_instance_before_a_mount_step(
     match segment {
         FaultedSegment::TheStartingPoint => Some(MKFS_INSTANCE_GENERATION),
         FaultedSegment::Operation { operation_kind, .. } => match operation_kind {
-            // 崩溃恢复抛弃根也是一次可写挂载：取号、写行、暖机；掉电在 journal 记录之前的那次覆盖写之后也挂一次（掉电那一截不写系统配置）。
+            // 崩溃恢复抛弃根、每块盘最新那一槽系统配置读回全 0 的那一次也是可写挂载：取号、写行、暖机；
+            // 掉电在 journal 记录之前的那次覆盖写之后也挂一次（掉电那一截不写系统配置）。
             HistoryOperationKind::CloseAndMountWritable
             | HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
-            | HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable => {
+            | HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable
+            | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
                 let index = marks
                     .iter()
                     .position(|mark| mark.position == segment.position())?;
@@ -2511,7 +2513,8 @@ fn inject_one_fault(
             | HistoryOperation::CrashRecoveryAbandoningTheNewestRoot
             | HistoryOperation::Unlink
             | HistoryOperation::CreateInode
-            | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_) => {
+            | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_)
+            | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
                 None
             }
         });
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index 62404cf..2b7616b 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -36,7 +36,7 @@ use singlefs_core::make_filesystem::{
 use singlefs_core::mount::{
     mount_writable_with_space_admission, raise_rollback_floor, roll_back_by_a_forward_publish,
     MountError, ParametersAndDeviceTableOfTheMount, RollbackError, RollbackTarget, ShadowLedger,
-    StillUnreadableAfterOneReread,
+    StillUnreadableAfterOneReread, SystemConfigurationUnreadableWhen,
 };
 use singlefs_core::mounted_session::{
     MountedSession, UserChange, UserChangePublished, UserChangeRefused,
@@ -54,6 +54,7 @@ use singlefs_core::transaction::{
 use singlefs_core::unit::data_unit_payload_capacity;
 use singlefs_format::{
     DATA_UNIT_BYTES, EXTENT_TREE_LOWER_LEAF_DATA_UNITS, JOURNAL_RING_START_SLOT, SLOT_BYTES,
+    SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES,
 };
 
 use crate::fault_injection::{FaultInjectingBlockDevice, SharedFaultPlan};
@@ -506,6 +507,12 @@ pub enum HistoryOperation {
     /// 再发清扫发布删那一行（D18（块里携带什么信息） 已定项 11「回收」一条：抹头那一段只有这样的孤儿走得到）。
     /// 掉电截在第一条 journal 记录之前，那次改动没有任何一样能让恢复读到它的内容：交给模型与 RefFS 时与可写挂载相同。
     OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(ContentChoice),
+    /// 关掉这个进程的可写会话，可写挂载期间每块盘自证过、世代号最大的那一槽系统配置读回全 0（单故障：D22（单元原子性怎么合成）
+    /// 已定项 8「容错承诺是单故障：每盘一槽系统配置坏在承诺里」；`DeviceReadingZerosOverHiddenRanges`，挂载自己写到那一槽之后照常读得出）。
+    /// 读回全 0 的槽自证不过、设备不报错，可写挂载照「一槽撕裂、另一槽自证过」往下走（同条 S-辨），取号那一写落到被藏的那一槽上：
+    /// 崩溃注入里唯一一种在读故障下真的往盘上发写的步（C696（崩溃注入的读故障一支在随机历史里从没触发））。
+    /// 只进它自己那一组比重（[`GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS`]），已有各组生成的历史不变。
+    WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
 }
 
 /// 操作的种类：统计与比重用。
@@ -522,10 +529,11 @@ pub enum HistoryOperationKind {
     Unlink,
     CreateInode,
     OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable,
+    WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
 }
 
 impl HistoryOperationKind {
-    pub const ALL: [HistoryOperationKind; 11] = [
+    pub const ALL: [HistoryOperationKind; 12] = [
         HistoryOperationKind::PublishFirstFile,
         HistoryOperationKind::PublishOverwrite,
         HistoryOperationKind::PublishWithoutUnits,
@@ -537,6 +545,7 @@ impl HistoryOperationKind {
         HistoryOperationKind::Unlink,
         HistoryOperationKind::CreateInode,
         HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable,
+        HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
     ];
 }
 
@@ -559,6 +568,9 @@ impl HistoryOperation {
             HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_) => {
                 HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable
             }
+            HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
+                HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
+            }
         }
     }
 }
@@ -600,6 +612,8 @@ pub enum NamedGenerationWeights {
     DeletingFilesOfSeveralLowerLeaves,
     /// [`GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP`]。
     LeavingOrphansForTheInstanceTableSweep,
+    /// [`GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS`]。
+    NewestSystemConfigurationSlotOfEachDeviceReadingZeros,
 }
 }
 
@@ -627,6 +641,9 @@ impl NamedGenerationWeights {
             NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep => {
                 GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP
             }
+            NamedGenerationWeights::NewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
+                GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS
+            }
         }
     }
 
@@ -641,6 +658,9 @@ impl NamedGenerationWeights {
             NamedGenerationWeights::TowardTheUnitAreaWall => "unit-area-wall",
             NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves => "several-leaves",
             NamedGenerationWeights::LeavingOrphansForTheInstanceTableSweep => "orphans",
+            NamedGenerationWeights::NewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
+                "newest-system-configuration-slot-zeros"
+            }
         }
     }
 }
@@ -947,6 +967,68 @@ impl GenerationWeights {
         overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
     };
 
+    /// 崩溃注入读故障那一支的取样点（C696（崩溃注入的读故障一支在随机历史里从没触发），用户定设计调查的候选「丁」）：与
+    /// [`GenerationWeights::BROAD`] 逐格相同，只把崩溃恢复抛弃根那几格（份数照旧）换成「每块盘最新那一槽系统配置读回全 0 时的可写挂载」
+    /// （[`HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros`]）。两种步都不抽随机数、生成时对会话的
+    /// 估计都与可写挂载相同，所以同一个种子在这组比重下生成的历史与快档那一组逐步相同、只有那几步换了种类。已有各组的份数表里没有新步，
+    /// 它们生成的历史不变（`tests/random_history_generation_keeps_the_existing_weights_step_for_step.rs` 钉）。
+    pub const NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS: GenerationWeights =
+        GenerationWeights {
+            name: "每块盘最新那一槽系统配置读回全 0 的可写挂载（崩溃注入读故障那一支的取样点）",
+            named: NamedGenerationWeights::NewestSystemConfigurationSlotOfEachDeviceReadingZeros,
+            starting_points: &[
+                (HistoryStartingPoint::AfterMakeFilesystem, 1),
+                (HistoryStartingPoint::AfterFirstFile, 1),
+            ],
+            with_session_closed: &[
+                (HistoryOperationKind::CloseAndMountWritable, 92),
+                (HistoryOperationKind::ColdStartRecover, 8),
+            ],
+            with_session_open_without_file: &[
+                (HistoryOperationKind::PublishFirstFile, 60),
+                (HistoryOperationKind::PublishWithoutUnits, 20),
+                (HistoryOperationKind::CloseAndMountWritable, 4),
+                (
+                    HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
+                    2,
+                ),
+                (HistoryOperationKind::RollBackWhileMounted, 4),
+                (HistoryOperationKind::ColdStartRecover, 4),
+                (HistoryOperationKind::PublishOverwrite, 3),
+                (HistoryOperationKind::RaiseRollbackFloor, 3),
+            ],
+            with_session_open_with_file: &[
+                (HistoryOperationKind::PublishOverwrite, 52),
+                (HistoryOperationKind::RaiseRollbackFloor, 16),
+                (HistoryOperationKind::CloseAndMountWritable, 6),
+                (HistoryOperationKind::Unlink, 3),
+                (
+                    HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
+                    3,
+                ),
+                (HistoryOperationKind::RollBackWhileMounted, 10),
+                (HistoryOperationKind::ColdStartRecover, 4),
+                (HistoryOperationKind::PublishFirstFile, 4),
+                (HistoryOperationKind::PublishWithoutUnits, 2),
+            ],
+            with_session_open_after_the_file_was_deleted: Some(&[
+                (HistoryOperationKind::CreateInode, 40),
+                (HistoryOperationKind::RaiseRollbackFloor, 15),
+                (HistoryOperationKind::RollBackWhileMounted, 15),
+                (HistoryOperationKind::CloseAndMountWritable, 10),
+                (
+                    HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
+                    3,
+                ),
+                (HistoryOperationKind::ColdStartRecover, 5),
+                (HistoryOperationKind::PublishOverwrite, 4),
+                (HistoryOperationKind::PublishFirstFile, 4),
+                (HistoryOperationKind::PublishWithoutUnits, 4),
+            ]),
+            rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
+            overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
+        };
+
     /// 复现入口认得的每一组比重：按 `environment_value` 找回（[`HistorySampling::from_environment`]）。
     /// 照 [`NamedGenerationWeights::ALL`] 逐个经 [`NamedGenerationWeights::weights`] 生成，不手抄常量名：
     /// 新加一组比重要一个新成员，成员不在 `weights` 的穷举 `match` 里补一臂就编不过（代码三方 test-script-align-code-r1 判决 K4）。
@@ -1116,6 +1198,9 @@ fn draw_operation(
         }
         HistoryOperationKind::Unlink => HistoryOperation::Unlink,
         HistoryOperationKind::CreateInode => HistoryOperation::CreateInode,
+        HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
+            HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
+        }
         HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable => {
             HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(
                 match overwrite_contents {
@@ -1145,10 +1230,12 @@ fn expected_session_after(
 ) -> ExpectedSession {
     match kind {
         // 崩溃恢复抛弃根也是一次可写挂载：落到的前一条根带不带文件，生成时照「这段历史发过文件没有」估，与可写挂载同一个估法；
-        // 掉电在 journal 记录之前的那次覆盖写之后也是一次可写挂载，落到的是那次改动之前的一版。
+        // 掉电在 journal 记录之前的那次覆盖写之后也是一次可写挂载，落到的是那次改动之前的一版；
+        // 每块盘最新那一槽系统配置读回全 0 的那一次也是可写挂载，同一个估法。
         HistoryOperationKind::CloseAndMountWritable
         | HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
-        | HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable => {
+        | HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable
+        | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
             match first_file {
                 ExpectedFirstFile::Written => ExpectedSession::OpenWithFile,
                 ExpectedFirstFile::NotWrittenYet => ExpectedSession::OpenWithoutFile,
@@ -1622,6 +1709,9 @@ pub enum MissingPrecondition {
     /// 现行版本里第一个文件还活着：建 inode 只接在第一个文件已删完的一版上（`HistoryOperation::CreateInode` 的文档）；
     /// 删除中的那一版照调，会话先走空、把它走成删完的一版（代码轮 U11）。
     CurrentVersionStillHasTheFirstFile,
+    /// 每块盘最新那一槽系统配置读回全 0 的可写挂载：有一块盘两槽里本池自证过的不到两份（mkfs 之后还没取过号的盘只写过一槽）——
+    /// 再藏它自证过的那一槽，这块盘一份自证过的系统配置都不剩，那是那块盘两槽都坏，不是单故障（D22（单元原子性怎么合成） 已定项 8）。
+    SomeDeviceWithoutTwoSelfVerifiedSystemConfigurationSlots,
 }
 
 impl MissingPrecondition {
@@ -1639,6 +1729,9 @@ impl MissingPrecondition {
             MissingPrecondition::NewestPublishNamesNoUnit => "最新那次发布一个单元都没点名",
             MissingPrecondition::CurrentVersionWithoutTheFirstFile => "现行版本里第一个文件已删完",
             MissingPrecondition::CurrentVersionStillHasTheFirstFile => "现行版本里第一个文件还在",
+            MissingPrecondition::SomeDeviceWithoutTwoSelfVerifiedSystemConfigurationSlots => {
+                "有盘两槽系统配置不是都自证过"
+            }
         }
     }
 }
@@ -2493,6 +2586,7 @@ impl HistoryTally {
             | HistoryOperation::CrashRecoveryAbandoningTheNewestRoot
             | HistoryOperation::Unlink
             | HistoryOperation::CreateInode
+            | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
             // 那次改动掉电在 journal 记录之前、一次都没发布成：内容长度不记进「进过入口」「发布过」（结局是那之后可写挂载的结局）。
             | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_) => {
                 None
@@ -2961,6 +3055,22 @@ fn mount_error_member(error: &MountError) -> String {
                 still_unreadable_after_one_reread_member(still_unreadable)
             )
         }
+        MountError::SystemConfigurationUnreadableAfterTheReadStage(unreadable) => {
+            return format!(
+                "MountError::SystemConfigurationUnreadableAfterTheReadStage({})",
+                match unreadable.unreadable {
+                    SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
+                        ..
+                    } => "CheckingEachDeviceBeforeTheAcquisition",
+                    SystemConfigurationUnreadableWhen::AcquisitionReadsAnUndecidableWitness {
+                        ..
+                    } => "AcquisitionReadsAnUndecidableWitness",
+                    SystemConfigurationUnreadableWhen::AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot {
+                        ..
+                    } => "AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot",
+                }
+            )
+        }
     };
     format!("MountError::{member}")
 }
@@ -4652,6 +4762,110 @@ fn apply_overwrite_losing_power_before_its_journal_record_then_mount_writable(
     apply_mount_writable(pool, parameters)
 }
 
+/// 每块盘本池自证过（整槽校验和过、fsid 与本池相同）、世代号最大的那一槽系统配置在盘上的那一段：槽号是世代号对每盘槽数取余，
+/// 与发布末尾的轮换、取号那一写取槽同一个取法（`crates/singlefs-core/src/transaction.rs` 取号写系统配置那一处）。交回 (盘, 那一段)，
+/// 按 `devices` 的次序；有一块盘自证过的不到两份时交回 None（[`MissingPrecondition::SomeDeviceWithoutTwoSelfVerifiedSystemConfigurationSlots`]）。
+fn newest_self_verified_system_configuration_slot_of_each_device(
+    parameters: &MakeFilesystemParameters,
+    image_before_mount: &MemoryPool,
+    devices: &[DeviceIdentity],
+) -> Option<Vec<(DeviceIdentity, HiddenDeviceRange)>> {
+    let slot_spacing_in_bytes = u64::from(parameters.geometry.fixed_structure_slot_spacing);
+    devices
+        .iter()
+        .map(|device| {
+            let self_verified = singlefs_core::recovery::verified_system_configuration_slots(
+                image_before_mount,
+                *device,
+                slot_spacing_in_bytes,
+                &parameters.filesystem_identifier,
+            );
+            if u64::try_from(self_verified.len()).expect("一块盘的系统配置槽数装得进 u64")
+                < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
+            {
+                return None;
+            }
+            let newest_slot_generation = self_verified
+                .iter()
+                .map(|system_configuration| system_configuration.quantities.slot_generation)
+                .max()
+                .expect("上面判过这块盘自证过的系统配置有两份");
+            Some((
+                *device,
+                HiddenDeviceRange {
+                    offset_in_bytes: (newest_slot_generation
+                        % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
+                        * slot_spacing_in_bytes,
+                    length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
+                },
+            ))
+        })
+        .collect()
+}
+
+/// 每块盘最新那一槽系统配置读回全 0 时的可写挂载
+/// （`HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros`）：关掉这个进程的会话，每块盘本池自证过、
+/// 世代号最大的那一槽（[`newest_self_verified_system_configuration_slot_of_each_device`]）整次挂载读回全 0
+/// （[`DeviceReadingZerosOverHiddenRanges`]：取号那一写落到它上面之后照常读得出）。读回全 0 的槽自证不过、设备不报读错：读阶段的见证取另一槽，
+/// R2 判不出不成立，逐盘核与取号照「一槽撕裂、另一槽自证过」往下走（D22（单元原子性怎么合成） 已定项 8 的 S-辨）。模型照一次普通可写挂载答
+/// （`IdealModel::answer_mount_writable`：模型不比系统配置落在哪一槽、带的见证值）；结局交给与可写挂载共用的 [`settle_mount`]。
+fn apply_writable_mount_with_the_newest_system_configuration_slot_of_each_device_reading_zeros(
+    pool: &mut HistoryPool,
+    parameters: &MakeFilesystemParameters,
+) -> AppliedStep {
+    let image_before_mount = pool.image();
+    let devices: Vec<DeviceIdentity> = pool.devices.iter().map(|(identity, _)| *identity).collect();
+    let Some(hidden) = newest_self_verified_system_configuration_slot_of_each_device(
+        parameters,
+        &image_before_mount,
+        &devices,
+    ) else {
+        return AppliedStep::not_applicable(
+            MissingPrecondition::SomeDeviceWithoutTwoSelfVerifiedSystemConfigurationSlots,
+        );
+    };
+    pool.session = None;
+    pool.model.close_session();
+    pool.mount_attempts += 1;
+    let answer = pool.model.answer_mount_writable();
+    let stream_length_before = pool.stream.operation_count();
+    let mut devices_with_hidden_ranges: Vec<(
+        DeviceIdentity,
+        DeviceReadingZerosOverHiddenRanges<HistoryDevice>,
+    )> = std::mem::take(&mut pool.devices)
+        .into_iter()
+        .map(|(identity, device)| {
+            (
+                identity,
+                DeviceReadingZerosOverHiddenRanges {
+                    inner: device,
+                    hidden_ranges: hidden
+                        .iter()
+                        .filter(|(device_of_the_range, _)| *device_of_the_range == identity)
+                        .map(|(_, range)| *range)
+                        .collect(),
+                },
+            )
+        })
+        .collect();
+    let mounted = mount_writable_with_space_admission(
+        parameters,
+        &mut devices_with_hidden_ranges,
+        pool.space_admission,
+    );
+    pool.devices = devices_with_hidden_ranges
+        .into_iter()
+        .map(|(identity, device)| (identity, device.inner))
+        .collect();
+    settle_mount(
+        pool,
+        mounted,
+        &image_before_mount,
+        answer,
+        stream_length_before,
+    )
+}
+
 fn apply_rollback_while_mounted(
     pool: &mut HistoryPool,
     parameters: &MakeFilesystemParameters,
@@ -4973,6 +5187,12 @@ fn apply_operation(
                 write_time_seconds,
             )
         }
+        HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
+            apply_writable_mount_with_the_newest_system_configuration_slot_of_each_device_reading_zeros(
+                pool,
+                &parameters,
+            )
+        }
     }
 }
 
@@ -5929,7 +6149,10 @@ fn simpler_variants(operation: HistoryOperation) -> Vec<HistoryOperation> {
         | HistoryOperation::ColdStartRecover
         | HistoryOperation::CrashRecoveryAbandoningTheNewestRoot
         | HistoryOperation::Unlink
-        | HistoryOperation::CreateInode => Vec::new(),
+        | HistoryOperation::CreateInode
+        | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
+            Vec::new()
+        }
     }
 }
 
diff --git a/crates/singlefs-harness/src/model_comparison.rs b/crates/singlefs-harness/src/model_comparison.rs
index 8a6d857..5e0d252 100644
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -864,7 +864,12 @@ pub fn refusal_reason_of_mount_error(error: &MountError) -> ObservedRefusalReaso
         | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
         | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
         // 盘上的实例代号或 txg 已到顶：坏镜像才有。
-        | MountError::SequenceNumberPastTheTopOfItsRange(_) => ObservedRefusalReason::Unexplained,
+        | MountError::SequenceNumberPastTheTopOfItsRange(_)
+        // 读阶段判完之后（判据 N-配置 为假）逐盘核或取号那一刻系统配置又读不出（C664（挂载拒绝合法复算把两种拒因报成一个）之前并在 N-配置 里、
+        // 映射成模型那一条理由）：两遍读之间的瞬时读错或一块盘两槽都坏才有，模型没有这一条，不映射成「见证过比所选那一版新的发布」。
+        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => {
+            ObservedRefusalReason::Unexplained
+        }
     }
 }
 
@@ -915,7 +920,8 @@ pub fn root_ring_slot_still_bad_after_one_reread_of_mount_error(
         | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
         | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
         | MountError::SequenceNumberPastTheTopOfItsRange(_)
-        | MountError::NewerStateStillUnreadableAfterOneReread(_) => None,
+        | MountError::NewerStateStillUnreadableAfterOneReread(_)
+        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => None,
     }
 }
 
@@ -989,7 +995,8 @@ pub fn reported_ceiling_of_mount_error(error: &MountError) -> Option<ModelCheckp
         | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration { .. }
         | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
         | MountError::SequenceNumberPastTheTopOfItsRange(_)
-        | MountError::NewerStateStillUnreadableAfterOneReread(_) => None,
+        | MountError::NewerStateStillUnreadableAfterOneReread(_)
+        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => None,
     }
 }
 
@@ -1286,6 +1293,55 @@ mod tests {
         );
     }
 
+    /// C664（挂载拒绝合法复算把两种拒因报成一个）：读阶段判完之后逐盘核或取号那一刻系统配置又读不出
+    /// （`MountError::SystemConfigurationUnreadableAfterTheReadStage`，三处各一个）映射成 Unexplained，不映射成读阶段判据 N-配置 那一条模型理由——
+    /// 两种拒因分开报之后，模型那一侧也不许再把它们混成一条。
+    /// 判别力：这一成员映射成「见证过比所选那一版新的发布、重读仍判真」（`crates/mutations.tsv` 一行）时这一条红。
+    #[test]
+    fn a_system_configuration_unreadable_after_the_read_stage_does_not_map_to_the_witnessed_newer_publish(
+    ) {
+        use singlefs_core::mount::{
+            NewerPublishWitness, SelectedVersionAgainstTheWitness,
+            SystemConfigurationUnreadableAfterTheReadStage, SystemConfigurationUnreadableWhen,
+            WitnessedCounterComparison,
+        };
+        let read_stage = SelectedVersionAgainstTheWitness {
+            selected_version: RollbackTarget {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(4),
+            },
+            witness: NewerPublishWitness {
+                witnessed_journal_counter: 4,
+                comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
+                    selected_version_last_record_counter: 4,
+                },
+            },
+        };
+        for unreadable in [
+            SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
+                device: DeviceIdentity(1),
+            },
+            SystemConfigurationUnreadableWhen::AcquisitionReadsAnUndecidableWitness {
+                first_device_reporting_a_read_error: DeviceIdentity(0),
+            },
+            SystemConfigurationUnreadableWhen::AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot {
+                device: DeviceIdentity(1),
+            },
+        ] {
+            let refusal = MountError::SystemConfigurationUnreadableAfterTheReadStage(Box::new(
+                SystemConfigurationUnreadableAfterTheReadStage {
+                    read_stage,
+                    unreadable,
+                },
+            ));
+            assert_eq!(
+                refusal_reason_of_mount_error(&refusal),
+                ObservedRefusalReason::Unexplained,
+                "读阶段之后系统配置又读不出（{unreadable:?}）模型没有这一条理由，不并进判据 N-配置 那一条"
+            );
+        }
+    }
+
     /// 回退候选集的四条排除各映射到自己那一条理由、互不相同（此前「不在候选集里」映射成「低于 F、被抛弃」两条之一）；
     /// 「目标那一版树表 0 条」是第四条（D23（journal 的角色与格式） 已定项 14：候选集只收带文件的根）。原来的「落在一圈窗口外」那一条
     /// 用户 2026-10-04 定 RGall 拿掉（D16（发布语义） 已定项 1「回退候选集」）。
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
index 493a42e..d87a6cf 100644
--- a/crates/singlefs-harness/src/mount_transparency.rs
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -2201,11 +2201,13 @@ fn tree_table_entry_bytes_outcome(
 /// D23（journal 的角色与格式） 已定项 14 那张单子（C554 乙-配置续，用户 2026-09-27 定；C331 判不出改取 R2，用户 2026-10-03 定）上的拒可写理由。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub enum RefusalReasonOnTheD23List {
-    /// 读阶段判据 N-配置 为真（系统配置见证判不出按真：R2，有槽设备报读错而没有一块盘两槽都自证过；读阶段判完之后某块盘一份自证过的
-    /// 系统配置都没有也报成它，C554 乙-配置续 Q1），重读一次仍为真。
+    /// 读阶段判据 N-配置 为真（系统配置见证判不出按真：R2，有槽设备报读错而没有一块盘两槽都自证过），重读一次仍为真。
     NewerPublishWitnessedBySystemConfiguration,
     /// 重建分配器判抛弃用的最新那条根的实例表读不出，重读一次仍读不出。
     InstanceTableOfTheNewestRootUnreadable,
+    /// 读阶段判完、判据 N-配置 为假之后，取号之前的逐盘核或取号那一刻读系统配置判不出（R2）或某块盘一份自证过的都没有
+    /// （`MountError::SystemConfigurationUnreadableAfterTheReadStage`；C664（挂载拒绝合法复算把两种拒因报成一个）之前并在 N-配置 里报）。
+    SystemConfigurationUnreadableAfterTheReadStage,
 }
 
 /// 拒可写的理由在不在 D23 已定项 14 那张单子上。别的理由（单元区墙、坏盘上才有的那几种……）照崩溃注入第二截今天的写法判。
@@ -2226,6 +2228,9 @@ pub fn refusal_reason_on_the_d23_list(error: &MountError) -> Option<RefusalReaso
                 | StillUnreadableAfterOneReread::AccountOfAnAbandonedRoot { .. } => None,
             }
         }
+        MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => {
+            Some(RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage)
+        }
         MountError::Recovery(_)
         | MountError::FileVersionWithoutAnyJournalRecord
         | MountError::InstanceTableMalformed
@@ -2310,9 +2315,8 @@ fn system_configuration_witness_undecidable_on_the_crash_image(
 }
 
 /// 某块盘两槽里一份本池自证过的系统配置都没有（C554 乙-配置续 Q1：这块盘不「可见」，D18（块里携带什么信息） 已定项 11）。
-/// R2 之下这一形不并进判不出（一块盘两槽都撕裂、另一块盘两槽都自证过时 R2 判得出），实现在读阶段判完之后单列一个成员报它、
-/// 按判据为真处置（`mount::WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot`，报成判据 N-配置 的拒因，
-/// C664（挂载拒绝合法复算把两种拒因报成一个））；判法照同一个处置复算。
+/// R2 之下这一形不并进判不出（一块盘两槽都撕裂、另一块盘两槽都自证过时 R2 判得出）；实现在读阶段判完之后的逐盘核与取号里拒它
+/// （`MountError::SystemConfigurationUnreadableAfterTheReadStage`，与判据 N-配置 那一拒分开报，C664（挂载拒绝合法复算把两种拒因报成一个））。
 fn some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image(
     slots_of_every_device: &[SystemConfigurationSlotsReadOnTheCrashImage],
 ) -> bool {
@@ -2321,9 +2325,39 @@ fn some_device_without_any_self_verified_system_configuration_slot_on_the_crash_
         .any(|slots| slots.self_verified.is_empty())
 }
 
+/// 「读阶段判完之后系统配置又读不出」（D23（journal 的角色与格式） 已定项 14：取号那一刻按同一个判不出（R2）判，某块盘一份自证过的系统配置
+/// 都没有时也拒）在崩溃镜像上复算：每块盘两槽各直接读一次，判不出（[`system_configuration_witness_undecidable_on_the_crash_image`]）或某块盘
+/// 一份本池自证过的都没有（[`some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image`]）⇒ 真；择不到系统配置也按真。
+/// 两遍读之间的瞬时读错在崩溃镜像上复算不出（C664（挂载拒绝合法复算把两种拒因报成一个）），那一形怎么判见
+/// [`judge_writable_mount_refusal_of_a_crash_state`]。
+fn system_configuration_unreadable_after_the_read_stage_on_the_crash_image(
+    crash_image: &dyn PoolReader,
+) -> bool {
+    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
+        return true;
+    };
+    let immutable = system_configuration.immutable;
+    let slot_spacing = u64::from(immutable.sizes.fixed_structure_slot_spacing);
+    let slots_of_every_device: Vec<SystemConfigurationSlotsReadOnTheCrashImage> = crash_image
+        .device_identities()
+        .into_iter()
+        .map(|device| {
+            system_configuration_slots_read_on_the_crash_image(
+                crash_image,
+                device,
+                slot_spacing,
+                &immutable.filesystem_identifier,
+            )
+        })
+        .collect();
+    system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device)
+        || some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image(
+            &slots_of_every_device,
+        )
+}
+
 /// 判据 N-配置（D23 已定项 14「可写挂载读到的更新状态读不出」那一段）在崩溃镜像上自己复算一遍，不经实现的 `mount::newer_publish_witness`：
-/// 系统配置见证判不出（[`system_configuration_witness_undecidable_on_the_crash_image`]，R2）⇒ 按真；某块盘一份自证过的系统配置都没有
-/// （[`some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image`]）⇒ 按真；否则全部自证过的槽的 journal tail
+/// 系统配置见证判不出（[`system_configuration_witness_undecidable_on_the_crash_image`]，R2）⇒ 按真；否则全部自证过的槽的 journal tail
 /// 取最大为 c_见证（撕裂的槽跳过），0 ⇒ 假；所选那一版那次发布带末条标志的记录读得出 ⇒ c_见证 > 它的计数器；读不出而计数器等于 c_见证 的记录
 /// 读得出 ⇒ 它的 (实例代号, checkpoint_txg) > 所选那一版；两条都读不出按真。
 /// `selected_version` 是 R1 落到的那一版（与可写挂载读阶段同一份盘、同一套择根与重放）；没有时判据无从复算，交回假。
@@ -2348,11 +2382,7 @@ fn newer_publish_witnessed_on_the_crash_image(
             )
         })
         .collect();
-    if system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device)
-        || some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image(
-            &slots_of_every_device,
-        )
-    {
+    if system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device) {
         return true;
     }
     let witnessed_journal_counter = slots_of_every_device
@@ -2425,9 +2455,8 @@ fn witnessed_counter_holds_against_the_selected_version(
 /// C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）：判据 N-配置 在没注入读故障的崩溃镜像上复算，为真交回是哪一支。
 /// 没有读故障时设备不报读错，见证读判得出（R2，D22（单元原子性怎么合成） 已定项 8 射程：判不出 ⟺ 有槽设备报读错且没有一块盘两槽都自证过；
 /// 撕裂的系统配置轮换不算判不出）：c_见证 取全部自证过、fsid 与本池相同的槽 journal tail 的最大值，自证不过的槽（撕裂的、没写过的）跳过——
-/// 与 [`newer_publish_witnessed_on_the_crash_image`] 改取 R2 之后在撕裂的轮换上同一个取法。两边只在一形上分开：某块盘两槽都撕裂、
-/// 一份自证过的都没有（C554 乙-配置续 Q1，这块盘不「可见」；C664（挂载拒绝合法复算把两种拒因报成一个）），那一判按真、这里照常比，
-/// 那一形的拒照 U6 判。一份自证过的系统配置都择不到时 c_见证 为 0，交回 `None`。
+/// 与 [`newer_publish_witnessed_on_the_crash_image`] 改取 R2 之后在撕裂的轮换上同一个取法（某块盘两槽都撕裂、一份自证过的都没有的那一形
+/// 两边都照常比：实现拒它报的是另一个拒因，C664（挂载拒绝合法复算把两种拒因报成一个））。一份自证过的系统配置都择不到时 c_见证 为 0，交回 `None`。
 fn witnessed_counter_holds_on_a_crash_image_without_read_faults(
     crash_image: &dyn PoolReader,
     selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
@@ -2550,7 +2579,8 @@ pub fn judge_writable_mount_refusal(
 /// 判崩溃注入第二截的一次拒可写合不合法：U6 那四条（[`judge_writable_mount_refusal`]），再加 C670 那一条——崩溃态没注入读故障
 /// （`read_faults`）、理由是判据 N-配置，而判据在崩溃镜像上复算、撕裂的系统配置槽跳过之后仍为真（[`WitnessedCounterBranchThatHolds`]），
 /// 拒得不合法（[`RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults`]）。注入过读故障的崩溃态不判这一条
-/// （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。
+/// （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。理由是「读阶段判完之后系统配置又读不出」而崩溃态注入过读故障的，
+/// 判据在崩溃镜像上复算为假不算不合法（两遍读之间的瞬时读错复算不出，C664（挂载拒绝合法复算把两种拒因报成一个））。
 #[must_use]
 pub fn judge_writable_mount_refusal_of_a_crash_state(
     crash_image: &dyn PoolReader,
@@ -2586,6 +2616,15 @@ pub fn judge_writable_mount_refusal_of_a_crash_state(
                 );
             }
         }
+        // 读阶段之后系统配置又读不出、崩溃态注入过读故障：两遍读之间的瞬时读错在崩溃镜像上复算不出，判据复算为假不算拒得不合法
+        // （C664（挂载拒绝合法复算把两种拒因报成一个）的还清条件：带读故障的档里取号那一刻瞬时读错的态，分开报之后判合法）；
+        // 盘上逐字节不变与只读读出照判。没注入读故障的照 U6 复算（判据为假即不合法）。
+        (
+            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
+            ReadFaultsInTheCrashState::Injected,
+        ) => conditions_not_met.retain(|condition| {
+            *condition != RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
+        }),
         (
             RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
             ReadFaultsInTheCrashState::Injected,
@@ -2593,6 +2632,10 @@ pub fn judge_writable_mount_refusal_of_a_crash_state(
         | (
             RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable,
             ReadFaultsInTheCrashState::NoneInjected | ReadFaultsInTheCrashState::Injected,
+        )
+        | (
+            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
+            ReadFaultsInTheCrashState::NoneInjected,
         ) => {}
     }
     verdict_of_a_refusal_on_the_d23_list(reason, conditions_not_met)
@@ -2613,6 +2656,9 @@ fn conditions_not_met_by_a_refusal_on_the_d23_list(
         RefusalReasonOnTheD23List::InstanceTableOfTheNewestRootUnreadable => {
             instance_table_of_the_newest_root_unreadable_on_the_crash_image(crash_image)
         }
+        RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage => {
+            system_configuration_unreadable_after_the_read_stage_on_the_crash_image(crash_image)
+        }
     };
     let mut conditions_not_met = Vec::new();
     if !criterion_holds_on_the_crash_image {
diff --git a/crates/singlefs-harness/src/reference_filesystem_translation.rs b/crates/singlefs-harness/src/reference_filesystem_translation.rs
index 4ca58d0..39df169 100644
--- a/crates/singlefs-harness/src/reference_filesystem_translation.rs
+++ b/crates/singlefs-harness/src/reference_filesystem_translation.rs
@@ -405,10 +405,12 @@ impl ReferenceFileSystemTranslation {
             // 建 inode 只接在第一个文件删完之后（`HistoryOperation::CreateInode`），翻译到 unlink 写了盘那一步就已经停了；
             // 它也不改第一个文件的内容（新建的 inode 长度 0），照可写挂载与抬 F 那样不进操作序列。
             // 掉电在第一个 journal 记录写之前的那次覆盖写一样都没发布成（盘上没有能让恢复读到它的记录与根），之后那次可写挂载照可写挂载。
+            // 每块盘最新那一槽系统配置读回全 0 的可写挂载：根与单元照常读得出、落到的就是现行那一版，与可写挂载同样不进操作序列。
             HistoryOperation::CloseAndMountWritable
             | HistoryOperation::RaiseRollbackFloor(_)
             | HistoryOperation::CreateInode
-            | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_) => {
+            | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_)
+            | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
                 EventsOfAStep::nothing()
             }
             HistoryOperation::ColdStartRecover => match &seen.ending {
diff --git a/crates/singlefs-harness/tests/a_writable_mount_with_system_configuration_slot_zero_unreadable.rs b/crates/singlefs-harness/tests/a_writable_mount_with_system_configuration_slot_zero_unreadable.rs
index d7fc26f..979ab99 100644
--- a/crates/singlefs-harness/tests/a_writable_mount_with_system_configuration_slot_zero_unreadable.rs
+++ b/crates/singlefs-harness/tests/a_writable_mount_with_system_configuration_slot_zero_unreadable.rs
@@ -7,7 +7,8 @@
 //! 那一槽（槽 1）：见证读与取号只看每块盘的槽 1 的错实现在那几条上全过（攻方变异 slot1-only-both）。这里写坏的是最新的世代 8 那一槽（槽 0）：
 //! - 两块盘槽 0 都报读错：有槽报读错、没有一块盘两槽都自证过 ⇒ 判不出，可写挂载重读一次仍判不出、取号之前拒，取号那一刻也拒；
 //! - 盘 0 槽 0 读回全零（撕裂的轮换）、盘 1 槽 0 报读错：报读错的是第二块盘，同样判不出、拒可写；
-//! - 读阶段两槽都读得出、到取号之前的逐盘核那一遍盘 1 两槽才读不出（两次读之间的瞬时读错）：逐盘核交 `Err(盘 1)`，照见证读的拒因报。
+//! - 读阶段两槽都读得出、到取号之前的逐盘核那一遍盘 1 两槽才读不出（两次读之间的瞬时读错）：逐盘核交 `Err(盘 1)`，以「读阶段判完之后
+//!   系统配置又读不出」拒（C664（挂载拒绝合法复算把两种拒因报成一个）之后与见证读的拒因分开报）。
 mod common;
 
 use std::cell::RefCell;
@@ -23,7 +24,8 @@ use singlefs_core::block_device::{
     BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
 };
 use singlefs_core::mount::{
-    mount_writable, MountError, Mounted, StillUnreadableAfterOneReread, WitnessedCounterComparison,
+    mount_writable, MountError, Mounted, StillUnreadableAfterOneReread,
+    SystemConfigurationUnreadableWhen, WitnessedCounterComparison,
 };
 use singlefs_core::mounted_read::mount_read_only;
 use singlefs_core::recovery::PoolReader;
@@ -382,7 +384,7 @@ const SLOT_READS_OF_THE_ACQUISITION_BEFORE_ITS_BARRIER: u64 = 3;
 /// 场景：新池新建文件之后重开、可写挂载；读阶段与之前几遍读盘 1 两槽都读得出，到取号之前的逐盘核那一遍（取号写之前那道屏障那一刻
 /// 两槽各被读的次数减去取号自己的三次）两槽读坏，之后碰到别处的第一次读起读故障撤掉（两次读之间的瞬时读错）。
 /// 次数取另一份逐字节相同的池上照常挂载时数到的。
-/// 预期：逐盘核交 `Err(盘 1)`，照见证读的拒因报 `NewerStateStillUnreadableAfterOneReread`（后一遍判「有一块盘一份自证过的系统配置都没有」）；
+/// 预期：逐盘核交 `Err(盘 1)`，拒成 `SystemConfigurationUnreadableAfterTheReadStage`（逐盘核点名盘 1；C664 之后不再报成见证读的拒因）；
 /// 只读坏了逐盘核那两次；系统配置槽、根环、录制流步数都不变。
 /// 判别力：逐盘核读不出就跳过这块盘时，取号那几遍读得出，挂载做成。
 #[test]
@@ -425,19 +427,15 @@ fn a_writable_mount_is_refused_when_the_slots_of_the_second_device_turn_unreadab
     );
     let refused_by_the_check_of_every_device = matches!(
         &outcome,
-        Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable))
-            if matches!(
-                still_unreadable.as_ref(),
-                StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
-                    reread,
-                    ..
-                } if reread.witness.comparison
-                    == WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot
-            )
+        Err(MountError::SystemConfigurationUnreadableAfterTheReadStage(unreadable))
+            if unreadable.unreadable
+                == SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
+                    device: DeviceIdentity(1),
+                }
     );
     assert!(
         refused_by_the_check_of_every_device,
-        "逐盘核那一遍盘 1 两槽读不出：拒可写，后一遍判「有一块盘一份自证过的系统配置都没有」；得到 {}",
+        "逐盘核那一遍盘 1 两槽读不出：以读阶段判完之后系统配置又读不出拒、点名盘 1；得到 {}",
         describe(&outcome)
     );
     assert_eq!(
diff --git a/crates/singlefs-harness/tests/acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs b/crates/singlefs-harness/tests/acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs
index 77d4d98..be8aa81 100644
--- a/crates/singlefs-harness/tests/acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs
+++ b/crates/singlefs-harness/tests/acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs
@@ -1,9 +1,9 @@
 //! 这份文件钉的几格（实审 A3c，来历见 `research/prompts/m2-rev-a3c-implementer-report.md`）：
 //!
 //! 一、C554 乙-配置续 Q1（主 agent 定走「拒」）：取号那一刻读见证值时某块盘两槽里一份本池自证过的系统配置都读不出，
-//! 在第一个取号写之前拒——可写挂载照见证读的拒因报 `MountError::NewerStateStillUnreadableAfterOneReread`
-//! （后一遍判「有一块盘一份自证过的系统配置都没有」：C331（择根倒挂压过已确认的写） 改取 R2 之后，盘 0 两槽都自证过时 R2 判得出，
-//! 这一支不再并进判不出，单列 `WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot`），
+//! 在第一个取号写之前拒——可写挂载报 `MountError::SystemConfigurationUnreadableAfterTheReadStage`（取号那一刻这块盘一份自证过的都没有；
+//! C331（择根倒挂压过已确认的写） 改取 R2 之后，盘 0 两槽都自证过时 R2 判得出，这一支不并进判不出；C664（挂载拒绝合法复算把两种拒因报成一个）
+//! 之后也不再并进见证读的拒因），
 //! `acquire_instance` 报 `InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness`。
 //! 改之前这块盘在见证值里不出数、取号写照做：逐盘核那一遍读得出、见证值那一遍读不出（两次读之间一次瞬时读错）就走得到。
 //! 这一核在第一道屏障之前做一遍（拒的时候录制流一步不多），屏障之后取见证值时再读一遍，对不上就不写。
@@ -33,8 +33,7 @@ use singlefs_core::make_filesystem::{
     allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
 };
 use singlefs_core::mount::{
-    mount_writable, MountError, RollbackTarget, StillUnreadableAfterOneReread,
-    WitnessedCounterComparison,
+    mount_writable, MountError, RollbackTarget, SystemConfigurationUnreadableWhen,
 };
 use singlefs_core::pointer::LocationEntry;
 use singlefs_core::records::{parse_inode_internal_entry, TreeTableEntry, TREE_KIND_INODE};
@@ -254,27 +253,26 @@ fn reads_of_device_one_slots_before_the_acquisition_barrier(tag: &str) -> [u64;
     [counts[0], counts[1]]
 }
 
-/// 新池新建文件那一版可写挂载被拒时报的：读阶段判完之后取号那一刻盘 1 的槽读不出（两次读之间的瞬时读错），照见证读的拒因报
-/// （用户 2026-09-28 定并进见证读的拒因）——所选那一版就是它，后一遍判「有一块盘一份自证过的系统配置都没有」（盘 0 两槽都自证过，R2 判得出）。
+/// 新池新建文件那一版可写挂载被拒时报的：读阶段判完之后取号那一刻盘 1 的槽读不出（两次读之间的瞬时读错），以「读阶段判完之后系统配置
+/// 又读不出」拒（C664（挂载拒绝合法复算把两种拒因报成一个）之后不再并进见证读的拒因）——读阶段往下走用的那一遍所选那一版就是它，
+/// 取号那一刻点名盘 1 一份自证过的系统配置都没有（盘 0 两槽都自证过，R2 判得出）。
 fn refused_because_device_one_slots_turned_unreadable_after_the_read_stage(
     refusal: &Result<singlefs_core::mount::Mounted, MountError>,
     first: &TransactionOutput,
 ) -> bool {
-    let Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)) = refusal else {
+    let Err(MountError::SystemConfigurationUnreadableAfterTheReadStage(unreadable)) = refusal
+    else {
         return false;
     };
-    matches!(
-        still_unreadable.as_ref(),
-        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
-            first_read,
-            reread,
-        } if first_read.selected_version == RollbackTarget {
+    unreadable.read_stage.selected_version
+        == RollbackTarget {
             instance: first.root.instance,
             checkpoint_txg: first.root.checkpoint_txg,
-        } && reread.selected_version == first_read.selected_version
-            && reread.witness.comparison
-                == WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot
-    )
+        }
+        && unreadable.unreadable
+            == SystemConfigurationUnreadableWhen::AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot {
+                device: DeviceIdentity(1),
+            }
 }
 
 fn describe(refusal: &Result<singlefs_core::mount::Mounted, MountError>) -> String {
@@ -291,7 +289,7 @@ fn describe(refusal: &Result<singlefs_core::mount::Mounted, MountError>) -> Stri
 
 /// 场景：新池新建文件之后重开、可写挂载。盘 1 两个系统配置槽从「取号写之前最后一次读」起每次都读坏
 /// （次数取另一份相同的池上照常挂载时数到的：择系统配置、取号之前的逐盘核、算号那几遍都读得出）。
-/// 预期：取号拒在第一道屏障之前，照见证读的拒因报 `NewerStateStillUnreadableAfterOneReread`（后一遍判「有一块盘一份自证过的系统配置都没有」），
+/// 预期：取号拒在第一道屏障之前，报 `SystemConfigurationUnreadableAfterTheReadStage`（取号那一刻点名盘 1 一份自证过的系统配置都没有），
 /// `DiskSnapshot` 不变（系统配置槽、根环、录制流步数：一个写都没发），盘上一次屏障调用都没收到（录制流看不出取号那道屏障，
 /// 见 [`DeviceCountingBarriers`]）。
 /// 改之前：见证值只取盘 0 的、取号写照做，挂载做成。
diff --git a/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs b/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs
index f63ef0d..425919a 100644
--- a/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs
+++ b/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs
@@ -44,7 +44,8 @@ use singlefs_core::make_filesystem::{
 use singlefs_core::mount::{
     mount_writable, DeviceCount, DeviceWithoutTheSelectedVersion, MountError, NewerPublishWitness,
     RollbackTarget, SelectedVersionAgainstTheWitness, SelectedVersionLackingOnDevice,
-    StillUnreadableAfterOneReread, WitnessedCounterComparison, STRIPE_WIDTH_LOWER_BOUND,
+    StillUnreadableAfterOneReread, SystemConfigurationUnreadableAfterTheReadStage,
+    SystemConfigurationUnreadableWhen, WitnessedCounterComparison, STRIPE_WIDTH_LOWER_BOUND,
 };
 use singlefs_core::mounted_read::mount_read_only;
 use singlefs_core::records::STATISTIC_ALLOCATED_BYTES;
@@ -1707,8 +1708,8 @@ fn blank_device_refuses_the_writable_mount_by_name_before_any_write() {
         "盘 0 是空盘：拒在取号之前，两块盘逐字节不变"
     );
     // 盘 1 是空盘：没有槽报读错，R2 判得出；见证值取盘 0 读得出的最大 tail 5，所选那一版 (1, 5) 末条 5，判据 N-配置 为假。
-    // 往下逐盘核：盘 1 一份自证过的系统配置都没有（C554 乙-配置续 Q1，不「可见」），照见证读的拒因报，后一遍判
-    // 「有一块盘一份自证过的系统配置都没有」（用户 2026-09-28 定并进见证读的拒因）。
+    // 往下逐盘核：盘 1 一份自证过的系统配置都没有（C554 乙-配置续 Q1，不「可见」），以「读阶段判完之后系统配置又读不出」拒，
+    // 带读阶段往下走用的那一遍读数（C664（挂载拒绝合法复算把两种拒因报成一个）之后不再并进见证读的拒因）。
     let mut devices_with_device_one_blank = vec![
         (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
         (DEVICE_ONE, blank_device()),
@@ -1727,22 +1728,16 @@ fn blank_device_refuses_the_writable_mount_by_name_before_any_write() {
             },
         },
     };
-    let reread = SelectedVersionAgainstTheWitness {
-        selected_version: version_key(1, 5),
-        witness: NewerPublishWitness {
-            witnessed_journal_counter: 5,
-            comparison:
-                WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot,
-        },
-    };
     assert!(
         matches!(
             &refusal_with_device_one_blank,
-            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
-                if **still_unreadable
-                    == StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
-                        first_read,
-                        reread,
+            MountError::SystemConfigurationUnreadableAfterTheReadStage(unreadable)
+                if **unreadable
+                    == SystemConfigurationUnreadableAfterTheReadStage {
+                        read_stage: first_read,
+                        unreadable: SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
+                            device: DEVICE_ONE,
+                        },
                     }
         ),
         "盘 1 是空盘：读阶段判得出、为假，逐盘核点名盘 1 一份自证过的系统配置都没有：{refusal_with_device_one_blank:?}"
diff --git a/crates/singlefs-harness/tests/mount_transparency_judges_the_system_configuration_witness_by_r2.rs b/crates/singlefs-harness/tests/mount_transparency_judges_the_system_configuration_witness_by_r2.rs
index 5c463b2..c04c032 100644
--- a/crates/singlefs-harness/tests/mount_transparency_judges_the_system_configuration_witness_by_r2.rs
+++ b/crates/singlefs-harness/tests/mount_transparency_judges_the_system_configuration_witness_by_r2.rs
@@ -1,7 +1,8 @@
 //! harness 判法复算判据 N-配置 时，系统配置见证判不出改取 R2（D22（单元原子性怎么合成） 已定项 8 射程，用户 2026-10-03 定「改取 R2」；
 //! D23（journal 的角色与格式） 已定项 14 那一段；代码轮判决 `research/prompts/m3-root-ring-code-r1-main-verification.md` 第二节 RR8）：
 //! 判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过。撕裂的轮换（读得出、自证不过、设备没报错）不算判不出，c_见证 跳过撕裂的槽照常比；
-//! 某块盘两槽一份自证过的都没有（C554 乙-配置续 Q1，这块盘不「可见」）照实现那一个成员按真。
+//! 某块盘两槽一份自证过的都没有（C554 乙-配置续 Q1，这块盘不「可见」）：实现拒它报的是「读阶段判完之后系统配置又读不出」，
+//! 那一拒照它自己的判据复算为真；拿判据 N-配置 当理由拒它是拒错了（C664（挂载拒绝合法复算把两种拒因报成一个））。
 //! 小池是单元区 384 槽的两块内存盘，第一个文件之后的盘面（mkfs → 取号 → 暖机 → 第一个文件）。
 //! 判的拒都是用例自己拼的一份「判据 N-配置 为真、重读仍为真」：判法自己在崩溃镜像上复算判据，不看拒里填的读数。
 use singlefs_core::address::{
@@ -10,7 +11,8 @@ use singlefs_core::address::{
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::mount::{
     MountError, NewerPublishWitness, RollbackTarget, SelectedVersionAgainstTheWitness,
-    StillUnreadableAfterOneReread, WitnessedCounterComparison,
+    StillUnreadableAfterOneReread, SystemConfigurationUnreadableAfterTheReadStage,
+    SystemConfigurationUnreadableWhen, WitnessedCounterComparison,
 };
 use singlefs_core::recovery::verified_system_configuration_slots;
 use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
@@ -81,8 +83,34 @@ fn witness_refusal_claimed_on(selected_version: (InstanceGeneration, CheckpointT
     ))
 }
 
-/// 在崩溃镜像 `image` 上判那一拒：所选那一版取只读恢复落到的那一版，盘上逐字节不变、只读读出在允许集合里（U6 另两条都满足）。
-fn verdict_of_the_claimed_witness_refusal_on(image: &MemoryPool) -> WritableMountRefusalVerdict {
+/// 取号之前的逐盘核点名盘 0 一份自证过的系统配置都没有的一拒（判法自己复算判据，不看这里填的读数）。
+fn refusal_after_the_read_stage_claimed_on(
+    selected_version: (InstanceGeneration, CheckpointTxg),
+) -> MountError {
+    MountError::SystemConfigurationUnreadableAfterTheReadStage(Box::new(
+        SystemConfigurationUnreadableAfterTheReadStage {
+            read_stage: SelectedVersionAgainstTheWitness {
+                selected_version: RollbackTarget {
+                    instance: selected_version.0,
+                    checkpoint_txg: selected_version.1,
+                },
+                witness: NewerPublishWitness {
+                    witnessed_journal_counter: 1,
+                    comparison: WitnessedCounterComparison::NothingWitnessed,
+                },
+            },
+            unreadable: SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
+                device: DEVICES[0],
+            },
+        },
+    ))
+}
+
+/// 在崩溃镜像 `image` 上判 `claimed` 拼出的那一拒：所选那一版取只读恢复落到的那一版，盘上逐字节不变、只读读出在允许集合里（U6 另两条都满足）。
+fn verdict_of_a_claimed_refusal_on(
+    image: &MemoryPool,
+    claimed: fn((InstanceGeneration, CheckpointTxg)) -> MountError,
+) -> WritableMountRefusalVerdict {
     let fields = read_recovery_fields(image, ComparedVersion::TheRecoveredVersion);
     let selected_version = fields
         .report
@@ -91,12 +119,16 @@ fn verdict_of_the_claimed_witness_refusal_on(image: &MemoryPool) -> WritableMoun
     judge_writable_mount_refusal(
         image,
         fields.report.effective_root,
-        &witness_refusal_claimed_on(selected_version),
+        &claimed(selected_version),
         DiskAfterTheRefusal::ByteForByteUnchanged,
         ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
     )
 }
 
+fn verdict_of_the_claimed_witness_refusal_on(image: &MemoryPool) -> WritableMountRefusalVerdict {
+    verdict_of_a_claimed_refusal_on(image, witness_refusal_claimed_on)
+}
+
 /// 场景：两块盘第一个文件那次轮换写的那一槽（世代 5，槽 1）都撕裂——读得出、自证不过，设备不报错。
 /// 预期：R2 下这不是判不出，c_见证 取两块盘世代 4 那一槽、不比第一个文件那条记录的计数器大，判据复算为假：拿它当理由拒可写是拒错了，判非法。
 /// 取甲（哪块盘自证过的槽少于两份就按真）时这一拒判成合法。
@@ -126,10 +158,14 @@ fn a_witness_refusal_on_a_torn_system_configuration_rotation_is_a_refusal_for_th
 }
 
 /// 场景：盘 0 两个系统配置槽都撕裂（一份本池自证过的都没有），盘 1 两槽都自证过；设备不报错。
-/// 预期：R2 判得出（没有槽报读错），但盘 0 不「可见」（C554 乙-配置续 Q1）：实现单列一个成员报成判据 N-配置 的拒因、按真处置，
-/// 判法照同一个处置复算为真，这一拒合法。
+/// 预期：R2 判得出（没有槽报读错），但盘 0 不「可见」（C554 乙-配置续 Q1）：实现在读阶段判完之后的逐盘核里以「系统配置又读不出」拒，
+/// 判法照那一拒的判据（判不出或某块盘一份自证过的都没有）复算为真，合法；同一份盘面上拿判据 N-配置 当理由拒，判据照它自己的判法复算
+/// （c_见证 取盘 1 自证过的槽，不比第一个文件那条记录的计数器大）为假，是拒错了（C664（挂载拒绝合法复算把两种拒因报成一个）：两种拒因各按各的判据）。
+/// 判别力：判法复算判据 N-配置 时仍把「某块盘一份自证过的都没有」按真（`crates/mutations.tsv` 一行）时第二条断言红；
+/// 那一拒的判据不认「某块盘一份自证过的都没有」时第一条红。
 #[test]
-fn a_witness_refusal_on_a_device_without_any_self_verified_system_configuration_slot_is_legal() {
+fn on_a_device_without_any_self_verified_system_configuration_slot_the_refusal_after_the_read_stage_is_legal_and_a_witness_refusal_is_not(
+) {
     let mut image = pool_after_the_first_file();
     for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
         image.flip_byte(
@@ -154,10 +190,20 @@ fn a_witness_refusal_on_a_device_without_any_self_verified_system_configuration_
         "前提：盘 0 一份自证过的系统配置都没有，盘 1 两槽都自证过"
     );
     assert_eq!(
-        verdict_of_the_claimed_witness_refusal_on(&image),
+        verdict_of_a_claimed_refusal_on(&image, refusal_after_the_read_stage_claimed_on),
         WritableMountRefusalVerdict::Legal(
-            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
+            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage
         ),
-        "一块盘一份自证过的系统配置都没有：判据按真"
+        "一块盘一份自证过的系统配置都没有：读阶段之后系统配置又读不出那一拒的判据为真"
+    );
+    assert_eq!(
+        verdict_of_the_claimed_witness_refusal_on(&image),
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
+            conditions_not_met: vec![
+                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
+            ],
+        },
+        "同一份盘面上判据 N-配置 复算为假：拿它当理由拒是拒错了"
     );
 }
diff --git a/crates/singlefs-harness/tests/random_histories.rs b/crates/singlefs-harness/tests/random_histories.rs
index f6b4a88..2e138f7 100644
--- a/crates/singlefs-harness/tests/random_histories.rs
+++ b/crates/singlefs-harness/tests/random_histories.rs
@@ -204,6 +204,15 @@ fn assert_every_path_was_exercised(tally: &HistoryTally) {
                     "{kind:?} 不在快档那组比重里，却跑到了"
                 );
             }
+            // 只进它自己那一组比重（`GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS`，C696），快档那一组不抽它；
+            // 跑到了没有在那一组的取样点那条用例里核（`newest_system_configuration_slot_zeros_sampling_mounts_writable_and_ends_only_in_known_red_forms`）。
+            HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
+                assert_eq!(
+                    (operation.applied, operation.refused, operation.not_applicable),
+                    (0, 0, 0),
+                    "快档那一组比重不抽 {kind:?}"
+                );
+            }
             HistoryOperationKind::PublishFirstFile
             | HistoryOperationKind::PublishOverwrite
             | HistoryOperationKind::PublishWithoutUnits
@@ -433,6 +442,56 @@ fn rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_re
     );
 }
 
+/// 崩溃注入读故障那一支的取样点（C696（崩溃注入的读故障一支在随机历史里从没触发），用户定设计调查的候选「丁」）：比重取
+/// `GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS`（快档那一组、崩溃恢复抛弃根那几格换成「每块盘最新那一槽
+/// 系统配置读回全 0 时的可写挂载」），种子从这个测试周期的种子基起 48 段、每段 24 步，两块 384 槽单元区的窄盘（设计调查量的那一档），
+/// 每一步之后跑池级 checker、模型照判。先判清单外的失败（checker 判红、模型对不上都在其内），再核这一步真的跑到了、可写挂载做成过——
+/// 做成就是取号、写行、暖机都在读故障下发了写（D22（单元原子性怎么合成） 已定项 8：单故障在承诺里，挂载照常往下走）。
+/// 判别力：这一步藏成每块盘两槽都读回全 0（`crates/mutations.tsv` 一行：那块盘一份自证过的系统配置都不剩，双故障）时可写挂载被拒、
+/// 一次都做不成，这一条红。
+#[test]
+#[ignore = "harness 慢档用例：debug 下单条跑过 60 秒；随时跑：cargo test -p singlefs-harness -- --ignored，经内存包装"]
+fn newest_system_configuration_slot_zeros_sampling_mounts_writable_and_ends_only_in_known_red_forms(
+) {
+    let started = std::time::Instant::now();
+    let report = run_history_campaign(
+        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
+        48,
+        24,
+        &GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS,
+        HistoryExecution {
+            per_step_checker: PerStepChecker::Run,
+            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
+            space_admission: SpaceAdmission::JudgedByTheFormula,
+        },
+        worker_threads_by_default(),
+        FindingShrinking::ReportSeedsOnly,
+    );
+    let rendered = report.render();
+    print_uncaptured(&format!(
+        "── 随机历史：每块盘最新那一槽系统配置读回全 0 的取样点 ──\n{rendered}用时 {:.1} 秒\n",
+        started.elapsed().as_secs_f64()
+    ));
+    assert!(
+        report.new_findings.is_empty(),
+        "「已知红」清单外的失败：\n{rendered}"
+    );
+    let step = report
+        .tally
+        .operations_by_kind
+        .get(&HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros)
+        .copied()
+        .unwrap_or_default();
+    assert!(
+        step.applied >= 1,
+        "每块盘最新那一槽系统配置读回全 0 的可写挂载一次都没做成：{step:?}\n{rendered}"
+    );
+    assert_eq!(
+        step.refused, 0,
+        "单故障在承诺里，这一步不许被拒：{step:?}\n{rendered}"
+    );
+}
+
 /// 原分配记录墙那一格（增补 3 第 2 件代码三方第一轮判决第三节第 1 条）的取样点：比重取 `GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL`，
 /// 种子与步数见常量。分配记录树按绝对槽号按位置寻址之后（D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K1）没有「一个节点 812 条」那道墙，
 /// 模型也没有那条拒绝理由：越过 812 条之后实现因为分配记录拒一次，模型就判对不上。每一步之后照跑池级 checker、判红就停（第二轮判决第三节
diff --git a/crates/singlefs-harness/tests/random_history_generation_keeps_the_existing_weights_step_for_step.rs b/crates/singlefs-harness/tests/random_history_generation_keeps_the_existing_weights_step_for_step.rs
new file mode 100644
index 0000000..62db249
--- /dev/null
+++ b/crates/singlefs-harness/tests/random_history_generation_keeps_the_existing_weights_step_for_step.rs
@@ -0,0 +1,66 @@
+//! 随机历史的生成器加一种只进新比重的步（C696，`.claude/kb/checks-owed.md` 那一行；设计调查 `research/prompts/` 第 26 批 C696 候选「丁」）
+//! 之后，已有的每一组比重在一批种子下生成的历史与加之前逐位相同：快档、各取样点、崩溃注入与实验装置的复现都按它们生成，钉着的取样点与
+//! 变异表里「必须红」的命题才还是同一批历史。
+
+use singlefs_harness::history::{generate_history_with_weights, GenerationWeights, HistorySeed};
+
+/// 每组比重生成几段历史（种子 0 起）。
+const SEEDS_PER_WEIGHTS: u64 = 64;
+/// 每段历史几步：比快档（30 步）长，删除那一组 unlink 之后那张表也抽得到。
+const OPERATIONS_PER_HISTORY: usize = 40;
+
+/// 64 位 FNV-1a 的起始值与乘数：把一组比重生成的全部历史（`Debug` 写法：种子、起点、每一步与它抽到的参数）折成一个数。
+const FINGERPRINT_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
+const FINGERPRINT_PRIME: u64 = 0x0000_0100_0000_01b3;
+
+fn fingerprint_of_the_histories_generated_with(weights: &GenerationWeights) -> u64 {
+    (0..SEEDS_PER_WEIGHTS)
+        .map(|seed| {
+            format!(
+                "{:?}",
+                generate_history_with_weights(HistorySeed(seed), OPERATIONS_PER_HISTORY, weights)
+            )
+        })
+        .flat_map(String::into_bytes)
+        .fold(FINGERPRINT_OFFSET_BASIS, |fingerprint, byte| {
+            (fingerprint ^ u64::from(byte)).wrapping_mul(FINGERPRINT_PRIME)
+        })
+}
+
+/// 已有七组比重各自在种子 0..64、每段 40 步下生成的历史，指纹与加新步之前在同一份源码上量的逐个相同（2026-10-05 加之前先在不改动的副本里
+/// 照同一个折法把每一组打出来、抄进来）。新步只进它自己那一组比重：已有各组的份数表里没有它，抽法与份数之和不变，抽到的每一步与它的参数都不变。
+/// 判别力：把新步按崩溃恢复抛弃根的份数放进快档那一组（`crates/mutations.tsv` 一行）时快档那一组的指纹变了，这一条红。
+#[test]
+fn every_existing_weights_generate_the_same_histories_as_before_the_newest_system_configuration_slot_step(
+) {
+    let fingerprints: Vec<(&'static str, u64)> = [
+        GenerationWeights::BROAD,
+        GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
+        GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
+        GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
+        GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
+        GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES,
+        GenerationWeights::LEAVING_ORPHANS_FOR_THE_INSTANCE_TABLE_SWEEP,
+    ]
+    .iter()
+    .map(|weights| {
+        (
+            weights.environment_value(),
+            fingerprint_of_the_histories_generated_with(weights),
+        )
+    })
+    .collect();
+    assert_eq!(
+        fingerprints,
+        vec![
+            ("broad", 4_211_750_580_707_973_314),
+            ("reuse", 8_928_070_955_642_935_750),
+            ("rollback", 15_225_864_873_440_202_800),
+            ("wall", 5_861_325_111_516_291_226),
+            ("unit-area-wall", 13_275_214_430_534_233_912),
+            ("several-leaves", 1_583_708_721_650_671_726),
+            ("orphans", 10_448_260_464_455_344_558),
+        ],
+        "已有七组比重在种子 0..64、每段 40 步下生成的历史，与加新步之前逐位相同（指纹在副本里加之前量的）"
+    );
+}
diff --git a/crates/singlefs-harness/tests/writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion.rs b/crates/singlefs-harness/tests/writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion.rs
new file mode 100644
index 0000000..244b3d6
--- /dev/null
+++ b/crates/singlefs-harness/tests/writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion.rs
@@ -0,0 +1,173 @@
+//! C664（挂载拒绝合法复算把两种拒因报成一个，`.claude/kb/checks-owed.md` 那一行）的还清条件里那一句判别力自证：带读故障的档里造一个
+//! 取号那一刻瞬时读错的态，合成一个拒因时判不合法、分开后判合法。
+//! 造法：第一个文件之后的健康盘面（单元区 384 槽的两块内存盘，mkfs → 取号 → 暖机 → 第一个文件），崩溃镜像就是这份盘面本身——两遍读之间的
+//! 瞬时读错过后，盘上什么都看不出来。拒是用例自己拼的（判法自己在崩溃镜像上复算判据，不看拒里填的读数）：
+//! - 分开报的那一拒：`MountError::SystemConfigurationUnreadableAfterTheReadStage`，取号那一刻读见证值判不出（R2）；
+//! - 合成一个拒因时实现报的那一拒（C664 之前 `mount.rs` 的 `refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage`
+//!   拼的）：判据 N-配置 为真、重读仍为真，前一遍是读阶段往下走用的那一遍读数、后一遍同一个见证值判 R2 判不出。
+//!
+//! 挂载透明的射程只管不带故障的状态（D13（验证路线） 已定项 1）：崩溃态注入过读故障时，两遍读之间的瞬时读错在崩溃镜像上复算不出，
+//! 「读阶段判完之后系统配置又读不出」那一拒的判据复算为假不算拒得不合法；判据 N-配置 那一拒照旧复算（读阶段本来就判过它为假）。
+//! 没注入读故障的崩溃态上同一拒复算为假就是拒错了。
+
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::mount::{
+    MountError, NewerPublishWitness, RollbackTarget, SelectedVersionAgainstTheWitness,
+    StillUnreadableAfterOneReread, SystemConfigurationUnreadableAfterTheReadStage,
+    SystemConfigurationUnreadableWhen, WitnessedCounterComparison,
+};
+use singlefs_harness::history::{
+    execute_history_with, GeneratedHistory, HistoryDeviceWidth, HistoryExecution, HistorySeed,
+    HistoryStartingPoint, PerStepChecker,
+};
+use singlefs_harness::memory_pool::MemoryPool;
+use singlefs_harness::mount_transparency::{
+    judge_writable_mount_refusal_of_a_crash_state, read_recovery_fields, ComparedVersion,
+    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage,
+    RefusalConditionNotMet, RefusalReasonOnTheD23List, WritableMountRefusalVerdict,
+};
+use singlefs_harness::SharedStream;
+
+const SMALL: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
+const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
+
+fn pool_after_the_first_file() -> MemoryPool {
+    let stream = SharedStream::retaining_contents();
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: Vec::new(),
+    };
+    execute_history_with(
+        &history,
+        HistoryExecution {
+            per_step_checker: PerStepChecker::Skipped,
+            device_width: SMALL,
+            space_admission: SpaceAdmission::JudgedByTheFormula,
+        },
+        &stream,
+        &mut |_observation| {},
+    );
+    let mut pool = MemoryPool::with_devices(&DEVICES, SMALL.device_bytes());
+    pool.apply(&stream.retained_operations());
+    pool
+}
+
+/// 读阶段往下走用的那一遍读数：所选那一版、判据 N-配置 为假（c_见证 不比所选那一版末条的计数器大）。
+fn read_stage_settled_on(
+    selected_version: (InstanceGeneration, CheckpointTxg),
+) -> SelectedVersionAgainstTheWitness {
+    SelectedVersionAgainstTheWitness {
+        selected_version: RollbackTarget {
+            instance: selected_version.0,
+            checkpoint_txg: selected_version.1,
+        },
+        witness: NewerPublishWitness {
+            witnessed_journal_counter: 1,
+            comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
+                selected_version_last_record_counter: 1,
+            },
+        },
+    }
+}
+
+/// 拒的理由、崩溃态注没注入读故障，判一次（所选那一版取只读恢复落到的那一版，盘上逐字节不变、只读读出在允许集合里）。
+fn verdict_on(
+    crash_image: &MemoryPool,
+    refusal_on: fn(SelectedVersionAgainstTheWitness) -> MountError,
+    read_faults: ReadFaultsInTheCrashState,
+) -> WritableMountRefusalVerdict {
+    let fields = read_recovery_fields(crash_image, ComparedVersion::TheRecoveredVersion);
+    let selected_version = fields
+        .report
+        .effective_root
+        .expect("健康盘面上只读恢复落到第一个文件那一版");
+    judge_writable_mount_refusal_of_a_crash_state(
+        crash_image,
+        fields.report.effective_root,
+        &refusal_on(read_stage_settled_on(selected_version)),
+        DiskAfterTheRefusal::ByteForByteUnchanged,
+        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+        read_faults,
+    )
+}
+
+/// 分开报：取号那一刻读见证值判不出（盘 1 报读错）。
+fn refused_after_the_read_stage(read_stage: SelectedVersionAgainstTheWitness) -> MountError {
+    MountError::SystemConfigurationUnreadableAfterTheReadStage(Box::new(
+        SystemConfigurationUnreadableAfterTheReadStage {
+            read_stage,
+            unreadable: SystemConfigurationUnreadableWhen::AcquisitionReadsAnUndecidableWitness {
+                first_device_reporting_a_read_error: DEVICES[1],
+            },
+        },
+    ))
+}
+
+/// 合成一个拒因：照判据 N-配置 为真、重读仍为真报，后一遍同一个见证值判 R2 判不出。
+fn refused_as_the_witness_criterion(read_stage: SelectedVersionAgainstTheWitness) -> MountError {
+    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
+        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
+            first_read: read_stage,
+            reread: SelectedVersionAgainstTheWitness {
+                selected_version: read_stage.selected_version,
+                witness: NewerPublishWitness {
+                    witnessed_journal_counter: read_stage.witness.witnessed_journal_counter,
+                    comparison:
+                        WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots,
+                },
+            },
+        },
+    ))
+}
+
+/// 场景：第一个文件之后的健康盘面当崩溃镜像，可写挂载在取号那一刻因瞬时读错拒（盘上看不出来）。
+/// 预期：崩溃态注入过读故障时，分开报的那一拒合法（判据复算为假不算不合法，盘上不变、只读读出在允许集合里）；合成一个拒因时实现报的
+/// 判据 N-配置 那一拒复算为假、判不合法；没注入读故障时分开报的那一拒复算为假、判不合法（拒的理由在盘上不成立）。
+/// 判别力：注入过读故障时不放过那一拒的判据复算（`crates/mutations.tsv` 一行）时第一条红；没注入的也放过（一行）时第三条红；
+/// 那一成员映射回判据 N-配置 那一条理由（一行）时第一条红。
+#[test]
+fn a_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_legal_where_the_merged_witness_refusal_is_not(
+) {
+    let crash_image = pool_after_the_first_file();
+    assert_eq!(
+        verdict_on(
+            &crash_image,
+            refused_after_the_read_stage,
+            ReadFaultsInTheCrashState::Injected
+        ),
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage
+        ),
+        "注入过读故障：读阶段之后系统配置又读不出那一拒合法（瞬时读错在崩溃镜像上复算不出）"
+    );
+    assert_eq!(
+        verdict_on(
+            &crash_image,
+            refused_as_the_witness_criterion,
+            ReadFaultsInTheCrashState::Injected
+        ),
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
+            conditions_not_met: vec![
+                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
+            ],
+        },
+        "合成一个拒因（报成判据 N-配置）：判据复算为假，判不合法"
+    );
+    assert_eq!(
+        verdict_on(
+            &crash_image,
+            refused_after_the_read_stage,
+            ReadFaultsInTheCrashState::NoneInjected
+        ),
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,
+            conditions_not_met: vec![
+                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
+            ],
+        },
+        "没注入读故障：盘上两槽都读得出、都自证过，那一拒的判据复算为假，拒错了"
+    );
+}
diff --git a/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs b/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs
index a8087c6..4ed6d1c 100644
--- a/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs
+++ b/crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs
@@ -1,9 +1,9 @@
 //! C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）：崩溃注入第二截判拒可写合不合法时，崩溃态没注入读故障、拒的理由是
 //! D23（journal 的角色与格式） 已定项 14 的判据 N-配置，而判据在崩溃镜像上复算（撕裂的系统配置槽跳过）仍为真 ⇒ 拒得不合法
 //! （`singlefs_harness::mount_transparency::judge_writable_mount_refusal_of_a_crash_state`）。注入过读故障的崩溃态不判这一条
-//! （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。某块盘两槽都撕裂那一支（一份自证过的系统配置都没有，C664）：C670 那一条
-//! 在这一形上照常比（跳过撕裂的槽取 c_见证），用例那一份盘面上复算为假、不报（换一份盘面照常比可以为真）；拒照 U6 判——系统配置见证照 R2
-//! 判得出（撕裂不算判不出），判据按「某块盘一份自证过的都没有」那一支为真，合法。
+//! （D13（验证路线） 已定项 1 挂载透明的射程只管不带故障的状态）。某块盘两槽都撕裂那一支（一份自证过的系统配置都没有）：C664（挂载拒绝合法复算
+//! 把两种拒因报成一个）之后实现报的是「读阶段判完之后系统配置又读不出」那一拒，不是判据 N-配置，C670 那一条不判；拒照 U6 判——系统配置见证照 R2
+//! 判得出（撕裂不算判不出），那一拒的判据按「某块盘一份自证过的都没有」为真，合法。
 //! 小池是单元区 384 槽的两块内存盘，第一个文件之后的盘面；崩溃镜像是内存盘面本身，读不报错。
 //! 见证判据为真的那份盘面取法照 E171（三oracle合谋测试） 第 242 行（系统配置轮换先于根槽 FUA 落盘）：一次覆盖写那次发布只落了系统配置轮换，
 //! 它的单元、记录与根都没落。
@@ -13,7 +13,7 @@ use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::mount::{
     mount_writable_with_space_admission, MountError, StillUnreadableAfterOneReread,
-    WitnessedCounterComparison,
+    SystemConfigurationUnreadableWhen, WitnessedCounterComparison,
 };
 use singlefs_core::recovery::PoolReader;
 use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
@@ -285,20 +285,26 @@ fn the_same_witness_refusal_in_a_crash_state_with_read_faults_injected_is_legal(
     );
 }
 
-/// 撕裂系统配置那一支：盘 0 两槽都撕裂、没注入读故障，实现在取号之前以「这块盘一份自证过的系统配置都没有」拒（报成判据 N-配置 的拒因，C664）；
-/// C670 那一条在「某块盘两槽都撕裂」这一形上照常比（跳过撕裂的槽取 c_见证），不因盘 0 一份自证过的都没有就按真；这份盘面上从盘 1 自证过的槽取、
-/// c_见证 等于第一个文件那一版末条的计数器，复算为假，不报（换一份盘面照常比可以为真）。U6 那四条里判据在崩溃镜像上复算：R2 之下撕裂的槽
-/// 不算判不出（没有槽设备报读错），盘 0 一份自证过的系统配置都没有 ⇒ 按真：合法。
+/// 撕裂系统配置那一支：盘 0 两槽都撕裂、没注入读故障，实现在取号之前的逐盘核以「这块盘一份自证过的系统配置都没有」拒
+/// （`MountError::SystemConfigurationUnreadableAfterTheReadStage`，C664（挂载拒绝合法复算把两种拒因报成一个）之后不再报成判据 N-配置 的拒因，
+/// C670 那一条只管 N-配置、不判它）。U6 那四条里那一拒的判据在崩溃镜像上复算：R2 之下撕裂的槽不算判不出（没有槽设备报读错），
+/// 盘 0 一份自证过的系统配置都没有 ⇒ 真：合法。
 #[test]
 fn a_refusal_on_a_crash_image_without_read_faults_where_a_device_has_both_system_configuration_slots_torn_is_legal(
 ) {
     let (healthy, crash_image) =
         crash_image_where_both_system_configuration_slots_of_device_zero_are_torn();
     let (error, after) = refused_writable_mount_on(&crash_image);
-    assert_eq!(
-        witnessed_counter_comparison_on_the_reread(&error),
-        Some(WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot),
-        "前提：盘 0 一份自证过的系统配置都没有，照判据 N-配置 的拒因拒：{error:?}"
+    assert!(
+        matches!(
+            &error,
+            MountError::SystemConfigurationUnreadableAfterTheReadStage(unreadable)
+                if unreadable.unreadable
+                    == SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
+                        device: DeviceIdentity(0)
+                    }
+        ),
+        "前提：盘 0 一份自证过的系统配置都没有，取号之前的逐盘核以读阶段之后系统配置又读不出拒：{error:?}"
     );
     let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
     assert_eq!(
@@ -311,7 +317,7 @@ fn a_refusal_on_a_crash_image_without_read_faults_where_a_device_has_both_system
             ReadFaultsInTheCrashState::NoneInjected,
         ),
         WritableMountRefusalVerdict::Legal(
-            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
+            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage
         )
     );
 }
````

### mutations-append.tsv

````text
第26批 C664：读阶段之后系统配置又读不出照旧并进判据 N-配置 的拒因（两种拒因报成一个）	crates/singlefs-core/src/mount.rs	    MountError::SystemConfigurationUnreadableAfterTheReadStage(Box::new(\n        SystemConfigurationUnreadableAfterTheReadStage {\n            read_stage: settled,\n            unreadable,\n        },\n    ))	    let _ = unreadable;\n    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(\n        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {\n            first_read: settled,\n            reread: SelectedVersionAgainstTheWitness {\n                selected_version: settled.selected_version,\n                witness: NewerPublishWitness {\n                    witnessed_journal_counter: settled.witness.witnessed_journal_counter,\n                    comparison: WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots,\n                },\n            },\n        },\n    ))	-p singlefs-harness --test a_writable_mount_with_system_configuration_slot_zero_unreadable -- a_writable_mount_is_refused_when_the_slots_of_the_second_device_turn_unreadable_only_for_the_check_of_every_device	a_writable_mount_is_refused_when_the_slots_of_the_second_device_turn_unreadable_only_for_the_check_of_every_device
第26批 C664 胶水：读阶段之后系统配置又读不出映射成「见证过比所选那一版新的发布」那一条模型理由	crates/singlefs-harness/src/model_comparison.rs	        | MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => {\n            ObservedRefusalReason::Unexplained\n        }	        => ObservedRefusalReason::Unexplained,\n        MountError::SystemConfigurationUnreadableAfterTheReadStage(_) => explained(\n            ModelRefusalReason::NewerPublishWitnessedBySystemConfigurationStillUnreadableAfterOneReread,\n        ),	-p singlefs-harness --lib -- model_comparison::tests::a_system_configuration_unreadable_after_the_read_stage_does_not_map_to_the_witnessed_newer_publish	a_system_configuration_unreadable_after_the_read_stage_does_not_map_to_the_witnessed_newer_publish
第26批 C664 判法：注入过读故障时读阶段之后系统配置又读不出那一拒照样要判据在镜像上复算为真（瞬时读错判成拒错了）	crates/singlefs-harness/src/mount_transparency.rs	        ) => conditions_not_met.retain(|condition| {\n            *condition != RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage\n        }),	        ) => {}	-p singlefs-harness --test writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion	a_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_legal_where_the_merged_witness_refusal_is_not
第26批 C664 判法：没注入读故障时也放过读阶段之后系统配置又读不出那一拒的判据复算	crates/singlefs-harness/src/mount_transparency.rs	        | (\n            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,\n            ReadFaultsInTheCrashState::NoneInjected,\n        ) => {}	        => {}\n        (\n            RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage,\n            ReadFaultsInTheCrashState::NoneInjected,\n        ) => conditions_not_met.retain(|condition| {\n            *condition != RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage\n        }),	-p singlefs-harness --test writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion	a_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_legal_where_the_merged_witness_refusal_is_not
第26批 C664 判法：读阶段之后系统配置又读不出那一拒当判据 N-配置 的拒判（复算不按各自的判据）	crates/singlefs-harness/src/mount_transparency.rs	            Some(RefusalReasonOnTheD23List::SystemConfigurationUnreadableAfterTheReadStage)	            Some(RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration)	-p singlefs-harness --test writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion	a_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_legal_where_the_merged_witness_refusal_is_not
第26批 C664 判法：读阶段之后系统配置又读不出那一拒的判据不认「某块盘一份自证过的都没有」	crates/singlefs-harness/src/mount_transparency.rs	    system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device)\n        || some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image(\n            &slots_of_every_device,\n        )\n}	    system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device)\n}	-p singlefs-harness --test mount_transparency_judges_the_system_configuration_witness_by_r2 -- on_a_device_without_any_self_verified_system_configuration_slot_the_refusal_after_the_read_stage_is_legal_and_a_witness_refusal_is_not	on_a_device_without_any_self_verified_system_configuration_slot_the_refusal_after_the_read_stage_is_legal_and_a_witness_refusal_is_not
第26批 C664 判法：复算判据 N-配置 仍把「某块盘一份自证过的系统配置都没有」按真（两种拒因的判据并成一个）	crates/singlefs-harness/src/mount_transparency.rs	    if system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device) {\n        return true;\n    }\n    let witnessed_journal_counter	    if system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device)\n        || some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image(&slots_of_every_device)\n    {\n        return true;\n    }\n    let witnessed_journal_counter	-p singlefs-harness --test mount_transparency_judges_the_system_configuration_witness_by_r2 -- on_a_device_without_any_self_verified_system_configuration_slot_the_refusal_after_the_read_stage_is_legal_and_a_witness_refusal_is_not	on_a_device_without_any_self_verified_system_configuration_slot_the_refusal_after_the_read_stage_is_legal_and_a_witness_refusal_is_not
第26批 C696 生成：快档那一组比重带文件那张表里崩溃恢复抛弃根那一格换成读回全 0 的可写挂载（已有比重的步序变了）	crates/singlefs-harness/src/history.rs	            (HistoryOperationKind::Unlink, 3),\n            (\n                HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot,\n                3,\n            ),	            (HistoryOperationKind::Unlink, 3),\n            (\n                HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,\n                3,\n            ),	-p singlefs-harness --test random_history_generation_keeps_the_existing_weights_step_for_step	every_existing_weights_generate_the_same_histories_as_before_the_newest_system_configuration_slot_step
第26批 C696 执行：读回全 0 的可写挂载把每块盘两槽系统配置都藏了（双故障，不在承诺里）	crates/singlefs-harness/src/history.rs	                    offset_in_bytes: (newest_slot_generation\n                        % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)\n                        * slot_spacing_in_bytes,\n                    length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,	                    offset_in_bytes: newest_slot_generation - newest_slot_generation,\n                    length_in_bytes: SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE * slot_spacing_in_bytes,	-p singlefs-harness --test random_histories -- --include-ignored newest_system_configuration_slot_zeros_sampling_mounts_writable_and_ends_only_in_known_red_forms	newest_system_configuration_slot_zeros_sampling_mounts_writable_and_ends_only_in_known_red_forms
第26批 C696 认定：读回全 0 的可写挂载不算在读故障下读盘的步	crates/singlefs-checker-tier/src/crash_injection.rs	        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot\n        | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => true,\n        HistoryOperationKind::PublishFirstFile	        HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => true,\n        HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros\n        | HistoryOperationKind::PublishFirstFile	-p singlefs-checker-tier --lib -- crash_injection::tests::mount_transparency_is_not_judged_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount	mount_transparency_is_not_judged_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount
第26批 C696 射程：注入过读故障的崩溃态上照样判挂载透明（越出 D13 已定项 1 只管不带故障的状态）	crates/singlefs-checker-tier/src/crash_injection.rs	        match read_faults_in_the_crash_state {\n            ReadFaultsInTheCrashState::NoneInjected => judge_mount_transparency_on_the_crash_image(	        match ReadFaultsInTheCrashState::NoneInjected {\n            ReadFaultsInTheCrashState::NoneInjected => judge_mount_transparency_on_the_crash_image(	-p singlefs-checker-tier --lib -- crash_injection::tests::mount_transparency_is_not_judged_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount	mount_transparency_is_not_judged_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount
第26批 C696 判别力 F5：读故障认定的崩溃窗口末尾交成写表下标（读回全 0 的可写挂载那一组比重的取样点上量）	crates/singlefs-checker-tier/src/crash_injection.rs	    stream_indexes[first_write_of_the_segment + last_write_within_the_crash_window]	    first_write_of_the_segment + last_write_within_the_crash_window	-p singlefs-checker-tier --lib -- crash_injection::tests::crash_states_after_a_newest_system_configuration_slot_zeros_mount_count_as_read_faulted_by_the_stream_index_of_the_crash_window	crash_states_after_a_newest_system_configuration_slot_zeros_mount_count_as_read_faulted_by_the_stream_index_of_the_crash_window
第26批 C696 判别力：读故障认定函数恒交 NoneInjected（读回全 0 的可写挂载那一组比重的取样点上量）	crates/singlefs-checker-tier/src/crash_injection.rs	            if step_reads_under_injected_faults && step_sent_something_by_the_crash {	            if false && step_reads_under_injected_faults && step_sent_something_by_the_crash {	-p singlefs-checker-tier --lib -- crash_injection::tests::crash_states_after_a_newest_system_configuration_slot_zeros_mount_count_as_read_faulted_by_the_stream_index_of_the_crash_window	crash_states_after_a_newest_system_configuration_slot_zeros_mount_count_as_read_faulted_by_the_stream_index_of_the_crash_window
第26批 C696 判别力：第三截调用点把读故障的认定写死成 NoneInjected（注入过读故障的态上也判 C670）	crates/singlefs-checker-tier/src/crash_injection.rs	                read_faults: read_faults_in_the_first_crash_state,	                read_faults: ReadFaultsInTheCrashState::NoneInjected,	-p singlefs-checker-tier --lib -- crash_injection::tests::a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount_is_not_judged_by_c670	a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount_is_not_judged_by_c670
第26批 F3-e 攻方 M7a：指纹只折段内末 12 个写	crates/singlefs-checker-tier/src/crash_injection.rs	    persisted_within_the_segment.iter().fold(\n        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,	    persisted_within_the_segment[persisted_within_the_segment.len().saturating_sub(12)..].iter().fold(\n        PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS,	-p singlefs-checker-tier --lib -- crash_injection::tests::first_crash_states_of_a_segment_of_thirty_two_writes_withholding_one_or_two_writes_anywhere_seed_their_second_crashes_differently	first_crash_states_of_a_segment_of_thirty_two_writes_withholding_one_or_two_writes_anywhere_seed_their_second_crashes_differently
第26批 F3-e 攻方 M7d：段号只取低 6 位	crates/singlefs-checker-tier/src/crash_injection.rs	            ^ u64::try_from(crash_point.segment_index).expect("段号装得进 u64")	            ^ (u64::try_from(crash_point.segment_index).expect("段号装得进 u64") & 63)	-p singlefs-checker-tier --lib -- crash_injection::tests::first_crash_states_of_segments_past_sixty_three_seed_their_second_crashes_differently_across_segments	first_crash_states_of_segments_past_sixty_three_seed_their_second_crashes_differently_across_segments
第26批 F3-e 攻方 M7f：历史种子只取最低位	crates/singlefs-checker-tier/src/crash_injection.rs	        seed.0\n            ^ SECOND_CRASH_POINT_SEED_SALT	        (seed.0 & 1)\n            ^ SECOND_CRASH_POINT_SEED_SALT	-p singlefs-checker-tier --lib -- crash_injection::tests::first_crash_states_under_history_seeds_zero_one_and_two_to_the_sixty_third_seed_their_second_crashes_differently	first_crash_states_under_history_seeds_zero_one_and_two_to_the_sixty_third_seed_their_second_crashes_differently
````

### mutations-replacements.tsv

````text
C554 乙：判不出（两条记录都读不出）按判据为假	crates/singlefs-core/src/mount.rs	            WitnessedCounterComparison::Undecidable\n            | WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots => {\n                true\n            }	            WitnessedCounterComparison::Undecidable => false,\n            WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots => {\n                true\n            }	-p singlefs-harness --test a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable -- without_the_last_record_of_the_selected_version	without_the_last_record_of_the_selected_version_the_record_at_the_witnessed_counter_decides_and_without_both_the_mount_counts_it_as_witnessed
C579（用户 2026-09-27 定：乙判不出按真）：乙把「判不出」按假处置，所选根自己那条记录读不出时可写挂载照做	crates/singlefs-core/src/mount.rs	            WitnessedCounterComparison::Undecidable\n            | WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots => {\n                true\n            }	            WitnessedCounterComparison::Undecidable => false,\n            WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots => {\n                true\n            }	-p singlefs-harness --test warm_up_journal_counter -- c366_when_the_chosen_root_own_record_is_unreadable	c366_when_the_chosen_root_own_record_is_unreadable_the_writable_mount_is_refused_by_the_undecidable_witness_and_the_read_only_mount_opens_the_chosen_root
拒绝合法（11-4）：每块盘最新那一槽系统配置都报读错时 N-配置 复算成假（C331 取甲「按真」那一句丢了）	crates/singlefs-harness/src/mount_transparency.rs	    if system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device) {\n        return true;\n    }\n    let witnessed_journal_counter = slots_of_every_device	    if system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device) {\n        return false;\n    }\n    let witnessed_journal_counter = slots_of_every_device	-p singlefs-harness --test writable_mount_refusal_judged_against_the_d23_list	a_refusal_when_the_newest_system_configuration_slot_of_every_device_reports_a_read_error_is_legal
挂载透明（11-4）：两棵树的根指针只看 R1 读不读得出、不比	crates/singlefs-harness/src/mount_transparency.rs	        observed_rewritten_only_by_the_continuation == *expected_rewritten_only_by_the_continuation	        true || observed_rewritten_only_by_the_continuation == *expected_rewritten_only_by_the_continuation	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	an_inode_tree_root_pointer_that_moved_with_the_mount_is_red
C673 ⑥ I-5.5 判法：落点判成恒成立（数据单元搬走的坏镜像不红）	crates/singlefs-checker/src/walk.rs	                judgements.judge(invariant, smallest_slot == expected_slot, || {	                judgements.judge(invariant, true, || {	-p singlefs-harness --test checker_known_bad_images -- the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target	the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target
````

### mutations-delete.txt

````text
第9批 RR8：harness 判法复算判据 N-配置 不认某块盘一份自证过的系统配置都没有（C554 乙-配置续 Q1 那一拒判成拒错了）
````
