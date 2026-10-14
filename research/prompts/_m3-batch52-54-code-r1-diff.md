# 删除线第 52–54 批（合后红的钉值、代码轮 m3-batch50-code-r1 判决的 F1–F5）：代码轮第一轮改动全文（2026-10-09）

三批实现员交的补丁原样，按打的次序（第 52 批 → 第 53 批 → 第 54 批），底是主工作区现在的样子（已含 W3 + 第 50 批）。

## m3-batch52/fix-reds

### crates.patch

`````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
index f516071..72055fe 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -964,14 +964,15 @@ mod tests {
     #[test]
     fn states_in_one_dispatch_follow_the_scratch_memory_up_to_the_lane_limit() {
         let bytes_per_state = u64::try_from(SCRATCH_WORDS * 4).expect("装得进 u64");
-        assert_eq!(states_per_scratch_lane(BINDING_LIMIT), 1702);
+        // 每个状态 320 688 × 4 = 1 282 752 字节：2 147 483 644 ÷ 1 282 752 = 1674 个（向下取整）
+        assert_eq!(states_per_scratch_lane(BINDING_LIMIT), 1674);
         assert_eq!(states_per_scratch_lane(1), 1, "再小的上限也带一个");
-        // 12 GiB 的草稿区：12 × 1024 × 1024 × 1024 ÷ 1 261 504 = 10 213 个（向下取整）
+        // 12 GiB 的草稿区：12 × 1024 × 1024 × 1024 ÷ 1 282 752 = 10 044 个（向下取整）
         assert_eq!(
             states_per_dispatch(BINDING_LIMIT, 12 * 1024),
             usize::try_from(12 * 1024 * 1024 * 1024 / bytes_per_state).expect("装得进 usize")
         );
-        assert_eq!(states_per_dispatch(BINDING_LIMIT, 12 * 1024), 10_213);
+        assert_eq!(states_per_dispatch(BINDING_LIMIT, 12 * 1024), 10_044);
         assert_eq!(
             states_per_dispatch(BINDING_LIMIT, 0),
             1,
@@ -979,14 +980,15 @@ mod tests {
         );
         assert_eq!(
             states_per_dispatch(BINDING_LIMIT, 1024 * 1024),
-            1702 * MAXIMUM_SCRATCH_LANES,
+            1674 * MAXIMUM_SCRATCH_LANES,
             "显存再多也只到条数上限"
         );
     }
 
     /// 卡池按 [`super::mebibytes_a_judge_needs`] 借给用例的卡（额度是借的量加驱动那一份）上，判器开的草稿区不少于借的时候算进去的状态数：
     /// 小流（45、489 个状态）照状态数、大流（131 195 个）照要的宽度（一条缓冲那么窄、三条缓冲加窄的那一档那么宽）。
-    /// 表取 1 MiB：45 个状态要 1 + ⌈45 × 1 261 616 ÷ 1 048 576⌉ = 1 + 55 = 56 MiB。
+    /// 表取 1 MiB：每个状态草稿区 320 688 × 4 加两份结论字 2 × 14 × 4，共 1 282 864 字节，
+    /// 45 个状态要 1 + ⌈45 × 1 282 864 ÷ 1 048 576⌉ = 1 + ⌈57 728 880 ÷ 1 048 576⌉ = 1 + 56 = 57 MiB。
     #[test]
     fn a_card_lent_what_a_judge_needs_opens_scratch_for_every_state_it_was_lent_for() {
         use super::{mebibytes_a_judge_needs, states_in_flight_within, PackedJudgeTables};
@@ -994,12 +996,12 @@ mod tests {
             words: vec![0; 1024 * 1024 / 4],
             state_count,
         };
-        assert_eq!(mebibytes_a_judge_needs(&tables_of(45), usize::MAX), 56);
+        assert_eq!(mebibytes_a_judge_needs(&tables_of(45), usize::MAX), 57);
         for (state_count, requested) in [
             (45, usize::MAX),
             (489, usize::MAX),
-            (131_195, 1702),
-            (131_195, 3 * 1702),
+            (131_195, 1674),
+            (131_195, 3 * 1674),
             (131_195, 256),
         ] {
             let tables = tables_of(state_count);
diff --git a/crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs b/crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs
index dde0509..76bba36 100644
--- a/crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs
+++ b/crates/singlefs-checker-tier/tests/bad_disk_input_campaign.rs
@@ -517,7 +517,8 @@ fn observation_of(base: &BaseImageForOneHistory, damaged: &DamagedImage) -> BadD
 /// [`DamageKind::NarrowedEntryWidthOfTheAccountingTreeRoot`] 原来的样子。冷走读与可写挂载的重建调同一个叶判，
 /// 两个读者都在解记账条目之前红在「根 key 区间与条目不符」上（末条的 key 与头里的最大 key 对不上）。
 /// 判别力：重建读记账树时不判节点头（`recovery.rs` 的 `rebuild_version` 那一处改回 `OnlyWhatTheShapeNeeds`），
-/// 可写挂载改报「记账条目数不是 3 + 6 × 盘数」——`crates/mutations.tsv` 里以「实审 A3d：重建读记账树不判节点头（记账根只剩一条」开头的那一行。
+/// 可写挂载改报记账条目窄于字段表（`EntryNarrowerThanItsFieldTable`：重建先核条目宽、后数条目）——`crates/mutations.tsv` 里以
+/// 「实审 A3d：重建读记账树不判节点头（记账根只剩一条」开头的那一行。
 #[test]
 fn an_accounting_root_narrowed_to_its_first_entry_is_refused_at_the_header_judgement_by_both_readers(
 ) {
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
index 4d209e6..8daa938 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
@@ -8,10 +8,15 @@ mod common;
 
 use common::{build_pool, file_content, geometry};
 use singlefs_checker::image::{
-    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY, INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
-    LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER, MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
+    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY, EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
+    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+    MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
     NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
     ONE_OBJECT_BIRTH_PER_INODE_NUMBER, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
+    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
+    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
     TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
     TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
     TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
@@ -118,8 +123,9 @@ fn checker_line(tally: &Layer0Tally) -> String {
 /// I-8.6（反向链算法） 要环里至少有一条自证过的记录，环还空着的那几个状态上报「不适用」；
 /// I-8.7（实例内事务号不重号）、I-8.8（前缀里的事务不被切开） 与 I-7.9（回退下界 F 不高于抬 F 的上限） 在这条流上一个状态都评估不到；
 /// I-8.10、I-8.11、I-8.12、I-4.1、I-7.16 各自一类，评估过的状态数逐条写在函数里那几张表旁边；
-/// 删除与回退那十条（I-4.7、I-4.6、I-4.4 与墓碑、谱系那七条 I-9.17–I-9.23）在这条流上一个状态都评估不到，理由同样写在表旁边；
-/// 其余 33 条每个状态都评估（2026-10-05 主工作区 checker 档日志快档那一行 `CHECKER`：74 条里 33 条是 46/0）。
+/// 删除与回退那十条（I-4.7、I-4.6、I-4.4 与墓碑、谱系那七条 I-9.17–I-9.23）与退役那四条（I-9.31–I-9.34）在这条流上一个状态都评估不到，
+/// I-9.35（退役完备）只在根已落的状态上评估，理由同样写在表旁边；
+/// 其余 34 条每个状态都评估（第 50 批合后 checker 档快档那一行 `CHECKER`：90 条里 34 条是 46/0）。
 /// 违例只许出现在 I-7.7（系统配置实例代号不低于根环）：C322（取号那一步的屏障怎么放没有条款） 未还，
 /// 取号只落了一块盘的那 2 个状态里两份系统配置的实例代号不等。
 fn assert_checker_counts(
@@ -171,6 +177,11 @@ fn assert_checker_counts(
         // I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，第 49 批）只在最新根下面有走全了的记账树时判：txg 3 那一版自己写的那一行
         // 就是第一级台阶 (3, 2)，① 判得到、成立；它的谱系树、墓碑树都是零根。种子根与暖机根没有记账树，报不适用。推的，没在 checker 档跑过。
         "I-9.30",
+        // I-9.35（退役完备，第 50 批）比相邻两版：删除态扫描只扫到 txg 3 那一版有 inode 树（要它的根已落），它的父版取 txg − 1 = 2
+        // （谱系树零根，没有 T_rb = 3 的记录），暖机第二次那条根比它早一段落盘、根已落的状态上恒在回退候选集里，树表 0 条、当父版时叶身份集是空集；
+        // txg 3 那一版的 inode 树一片叶、退役树零根：父版少的叶 = 空集 = 死于 txg 3 的退役记录 ⇒ 判得到、成立。
+        // 根没落的状态上没有一版有 inode 树，报不适用。对得上第 50 批合后 checker 档快档那一行 `CHECKER` 的 `I-9.35=9/0`。
+        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
     ];
     // I-3.10 除了回退候选集，还读下一次挂载会先施加的那一版（射程 ④）：新根没落而 txg 3 的记录已落时，
     // 恢复要由那条记录重建 txg 3 那一版，它的分配记录树在记录的新根段指着的树表下面。
@@ -216,6 +227,16 @@ fn assert_checker_counts(
         NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
         TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
     ];
+    // 没有删除：没有 inode 叶容器删空、被吸收，txg 3 那一版的退役树是零根（第 50 批，开池就建、零根注册）。主走读走不到退役树的内部条目
+    // （I-9.31 照 I-9.17、I-9.23 的形态：一条都没走到报不适用），删除态扫描读出的退役容器是空的（I-9.32 没有记录可判，
+    // I-9.33、I-9.34 没有死亡 txg 在谱系上的记录）。四条的 0 对得上第 50 批合后 checker 档快档那一行 `CHECKER` 的
+    // `I-9.31=0/0`、`I-9.32=0/0`、`I-9.33=0/0`、`I-9.34=0/0`。
+    let never_has_a_retirement_record_on_this_stream = [
+        RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
+        RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
+        RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+    ];
     // I-4.4（意图只描述目标态） 第一版报不适用（用户 2026-10-05 定），哪一条流上都评估不到；对得上同一份日志的 `I-4.4=0/0`。
     let not_applicable_in_the_first_version = [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE];
     // I-8.6 判的是「本实例内逻辑前一条」与这一条的链值，环里一条自证过的记录都没有时判不了：
@@ -241,6 +262,7 @@ fn assert_checker_counts(
         let expected_evaluated = if never_comparable_on_this_stream.contains(&invariant)
             || never_has_an_inode_being_deleted_on_this_stream.contains(&invariant)
             || never_has_a_tombstone_or_lineage_record_on_this_stream.contains(&invariant)
+            || never_has_a_retirement_record_on_this_stream.contains(&invariant)
             || not_applicable_in_the_first_version.contains(&invariant)
         {
             0
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
index 1f632ad..e075d97 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
@@ -840,16 +840,25 @@ fn print_the_unsharded_enumerations_for_the_golden_comparison() {
 /// 与断言 `left` 里的 `7e64a883…` 逐字相同）：比 `15867e36…` 那一版不同的只有两趟的 `GOLDEN_TALLY`（池级 checker 多出的不变量各一项，
 /// I-1.12 … I-9.28）与 `LAYER0_RESUME` 里进度文件名的计划哈希（fcc4d340 那一版的 `6118958c…` → `0370ad26…`；HEAD 31e47115 的日志里
 /// 计划哈希也是 `0370ad26…`、打的行只比主工作区少 `I-9.28=0/0/46` 那两项）。
+/// 2026-10-09 第 50 批合入（加 I-9.31–I-9.35，连同第 49 批 W3 加的 I-9.30，已实现的不变量 84 → 90 条）之后在合后副本重取：
+/// 打的行取那一趟断言 `left` 的 `4d42c7ca…`（行数仍 58）；把两趟 `GOLDEN_TALLY` 各删掉 ` I-9.30=9/0/37 I-9.31=0/0/46 I-9.32=0/0/46
+/// I-9.33=0/0/46 I-9.34=0/0/46 I-9.35=9/0/37` 六项、`LAYER0_RESUME` 里的计划哈希换回 `49878830…` 之后，整段 SHA-256 正好是上一版 `4b8b1e84…`，
+/// 别的字节不差。计划哈希 `49878830…` → `555baf1b…`：新池新建文件那次发布写下的字节变了（树表多一条退役树条目、根记录里树 ID 水位
+/// 65548 → 65549，`crash_points_tree_identifier_watermark_orphans.rs` 钉着），写表跟着变；单元写、段与状态数不变（仍 46 个状态）。
 const GOLDEN_PRINTED_LINE_COUNT: usize = 58;
 const GOLDEN_PRINTED_LINES_SHA256: &str =
-    "4b8b1e84e16b6a4ecd22a8bc28a15c75a5dbef3b99a1cfddc9e5b98703ed04b1";
+    "4d42c7ca5ca9e8e16e5c7bdde36198f576d40a98ad9a8dfb35fb33947161e354";
 /// 同一趟留下的进度文件：名字（带计划哈希）与整份内容的 SHA-256（47 行：文件头 + 46 行片行）。名字取自上面那一趟日志的 `LAYER0_RESUME` 行；
 /// 打的行与内容的 SHA-256 取自 2026-10-05 主工作区（形式化线第 34 批派出时那一版，含删除线第 22 批）：与根环线在「10g + 32-1」那棵树上重取的 58 行逐行比，只差 GOLDEN_TALLY 两行多出 I-9.17–I-9.24、I-9.29 九条新不变量的计数（删除线第 22 批把已实现的不变量从 74 条加到 83 条），已有各条计数不变；那一次比上一版只差 `LAYER0_RESUME` 那一行的计划哈希（T_time 改毫秒带来）：
 /// 打的行是那一趟断言 `left` 的 `50615e47…`。2026-10-06 删除线第 24 批合入之后（新九条各写专门的不适用理由、I-9.17 / I-9.23 改读孩子的方式）在主工作区重取：打的行取那一趟断言 `left` 的 `4fc9cde7…`；
 /// 内容是在只换了打的行钉值的副本里再跑一次、「不分片时的进度文件与钉值不同」那条断言 `left` 的第二项（2026-10-06 那一次是 `11977571…`，进度文件名不变）。2026-10-06 根环线第 10o 批合入（加 I-3.14，已实现的不变量 83 → 84 条）之后在主工作区（含删除线第 30 批）重取：打的行取断言 `left` 的 `4b8b1e84…`，把两趟 `GOLDEN_TALLY` 各一项 ` I-3.14=46/0/0` 删掉之后整段 SHA-256 正好是上一版 `4fc9cde7…`，别的字节不差；内容在只换了打的行钉值的副本里再跑一次取断言 `left` 第二项 `1d92a7ad…`，进度文件名不变。池级 checker 的判定集合再变，两个值跟着重取。
-const GOLDEN_PROGRESS_FILE_NAME: &str = "layer0-progress-golden_new_pool_file_creation_stream-fingerprint0-4987883029b866a331bd86aa1db770710eccbf4db5206676859243f01abd914c.txt";
+/// 2026-10-09 第 50 批合后重取：名字里的计划哈希取那一趟 `LAYER0_RESUME` 行的 `555baf1b…`（来历见 [`GOLDEN_PRINTED_LINE_COUNT`]）；
+/// 内容取那一趟判红之后留在临时目录里的进度文件（打的行那条断言先红，进度文件那条没跑到、目录没删），SHA-256 `1553ba8e…`。
+/// 与上一版 `1d92a7ad…` 那一份逐行比：去掉每行的校验和、每行逐条不变量的表里 I-9.30–I-9.35 那几项、计划哈希换回之后 47 行逐字相同；
+/// 那一份每行的 CRC-32C 校验和都对得上正文。
+const GOLDEN_PROGRESS_FILE_NAME: &str = "layer0-progress-golden_new_pool_file_creation_stream-fingerprint0-555baf1b8e6301e8874a473d44d0ce43260f946b695dfc0fbb3b0c5024bbbba6.txt";
 const GOLDEN_PROGRESS_FILE_SHA256: &str =
-    "1d92a7ad8c48c01f43dfe9d675d6256e110545b588d2bc5f6ad8601e63ed8cab";
+    "1553ba8eee2fd2823372ddc32820ec899effbb48376a5d34d4d8d8e2e31f7c19";
 
 /// 一行打印里会随机器与时刻变的两样换掉：进度目录、`elapsed_seconds=` 的值。
 fn masked_printed_line(line: &str, directory: &Path) -> String {
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_tree_split_streams.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_tree_split_streams.rs
index 96b66c5..f2e4d65 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_tree_split_streams.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_tree_split_streams.rs
@@ -188,26 +188,30 @@ impl TreeSplitStream {
                 shape_after: "L10",
                 units_written: 10,
             },
-            // 记账树 15 行根兼叶；空发布整批换代，叶容量 14：第 15 行插进去时从中间切（8 + 7），长出新根。
-            // 写：分配记录树五个节点、记账树两片叶与根、映射树一个节点、树表 = 10 个单元。
+            // 记账树 15 行根兼叶（第一个文件那一版自己那一行 inode 号水位就是第一级台阶，代段 3）；空发布整批换代，叶容量 14：
+            // 那一级台阶行照抄（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」，key 不变、不删不插），另 14 行先删，
+            // 叶只剩它；这次的 15 行按 key 升序插，统计量 12 排最后，前 14 行都插在台阶行之前：第 14 行插进去时 15 条从中间切（8 + 7，
+            // 台阶行在右叶），长出新根；最后插这次的水位行进右叶，8 + 8。写：分配记录树五个节点、记账树两片叶与根、映射树一个节点、树表 = 10 个单元。
             TreeSplitStream::AccountingRootSplit => StreamPlan {
                 first_file_capacities: capacities(BIG_ACCOUNTING, BIG_CENTRAL_MAPPING),
                 target_capacities: capacities((14, 3), BIG_CENTRAL_MAPPING),
                 target: TargetPublish::Empty,
                 tree: MultiLevelCodeTwoTree::Accounting,
                 shape_before: "L15",
-                shape_after: "L8 L7 I2",
+                shape_after: "L8 L8 I2",
                 units_written: 10,
             },
-            // 记账树按叶容量 14 两片叶加根；空发布叶容量压到 8：15 行插到第 9 行切一次、第 14 行再切一次，三片叶（5 + 5 + 5），
-            // 树高不变。写：分配记录树五个节点、记账树三片叶与根、映射树一个节点、树表 = 11 个单元。
+            // 记账树按叶容量 14 两片叶加根（8 + 7，第一级台阶行在右叶末尾）；空发布叶容量压到 8：台阶行照抄，另 14 行先删——
+            // 左叶删空摘掉、根只剩一个孩子降成右叶，右叶删到只剩台阶行；这次的 15 行按 key 升序插，前 14 行都在台阶行之前：
+            // 插到第 8 行时 9 条切一次（5 + 4），插到第 13 行右叶又到 9 条再切一次（5 + 4），第 14 行与这次的水位行落进最右那片，
+            // 三片叶（5 + 5 + 6），树高不变。写：分配记录树五个节点、记账树三片叶与根、映射树一个节点、树表 = 11 个单元。
             TreeSplitStream::AccountingLeafSplit => StreamPlan {
                 first_file_capacities: capacities((14, 3), BIG_CENTRAL_MAPPING),
                 target_capacities: capacities((8, 3), BIG_CENTRAL_MAPPING),
                 target: TargetPublish::Empty,
                 tree: MultiLevelCodeTwoTree::Accounting,
                 shape_before: "L8 L7 I2",
-                shape_after: "L5 L5 L5 I3",
+                shape_after: "L5 L5 L6 I3",
                 units_written: 11,
             },
         }
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs
index 74b2e2c..30f4a6c 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs
@@ -14,11 +14,11 @@
 //!   （`crash::Layer0StateObserving::OnTheWorkerThreads`），一个工作线程、四个工作线程与观察者在调用线程上三种跑法的计数逐项相同（快档对拍）。
 //!
 //! **比已有的流多罩了什么**：已有的层 0 流上没有删除类发布；这里意图那次（13 个单元：墓碑容器、`deleted_inodes` 根、inode 叶容器与根……）、
-//! 两批（12、11 个单元）、最后一批（10 个单元）各自的单元段、记录段、根槽写与之后的系统配置槽轮换第一次被枚举到；
+//! 两批（12、11 个单元）、最后一批（12 个单元：inode 叶容器删空不写，退役树开第一片容器、写根）各自的单元段、记录段、根槽写与之后的系统配置槽轮换第一次被枚举到；
 //! 多跑的一步是每个状态上的可写挂载（取号、写行、暖机、续做），与挂完的盘上那一遍池级 checker。
 //!
-//! 快档只展开写数小于 10 的段（七个单元段不展开，只以整段持久进入后面的状态）；全量标 ignored（层 0 的枚举域 1448083638 个状态，
-//! 七个单元段占 1448083449；每次写只取两态时的闭式 1448083588），登记成崩溃枚举用例，带断点续跑、可双机分片（与多记录流同一个入口）。
+//! 快档只展开写数小于 10 的段（七个单元段不展开，只以整段持久进入后面的状态）；全量标 ignored（层 0 的枚举域 1463812278 个状态，
+//! 七个单元段占 1463812089；每次写只取两态时的闭式 1463812228），登记成崩溃枚举用例，带断点续跑、可双机分片（与多记录流同一个入口）。
 //! 系统配置槽写是原地覆写，取三态（`crash::TearableInPlaceOverwrites`），其余写取两态。
 
 #[path = "../../singlefs-harness/tests/common/mod.rs"]
@@ -342,10 +342,11 @@ fn prepare(tag: &str) -> Prepared {
         sizes,
         vec![
             2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2, 26, 2, 1, 2, 24, 2, 1, 2,
-            22, 2, 1, 2, 20, 2, 1, 2
+            22, 2, 1, 2, 24, 2, 1, 2
         ],
         "前十九段同多记录流；之后意图那次 26 个单元写（13 个单元两盘各一份）、记录 2、根 1、轮换 2；两批 24、22 个单元写；\
-         最后一批 20 个单元写；各自记录 2、根 1、轮换 2"
+         最后一批 24 个单元写（12 个单元：extent 根、inode 根、分配记录树五个节点、记账树、退役容器与退役树根、映射树、树表）；\
+         各自记录 2、根 1、轮换 2"
     );
     let every_write_persisted = vec![true; writes.len()];
     let publishes = publishes_in(
@@ -977,11 +978,11 @@ fn the_unlink_stream_quick_tier_counts_the_same_on_one_worker_thread_on_four_and
     );
 }
 
-/// 全量：35 段。七个单元段各 2^n − 1（n = 26、24、22、20 是删除那四次），每个状态两遍恢复 + checker，逐状态核发布边界、删除的中间态，
+/// 全量：35 段。七个单元段各 2^n − 1（n = 26、24、22、24 是删除那四次），每个状态两遍恢复 + checker，逐状态核发布边界、删除的中间态，
 /// 并在工作线程上可写挂载一次核续做走到空。登记成崩溃枚举用例，checker-tier-layer0-replay 认下面打印的
 /// `LAYER0_UNLINK_AFTER_MULTI_RECORD` 行里 `exhaustive=true`；带断点续跑、可双机分片（同多记录流）。
 #[test]
-#[ignore = "全量 1448083638 个状态（十四亿多）、每个两遍恢复 + checker + 一次可写挂载；门禁 checker-tier-layer0-replay --full 在 release 下跑，带断点续跑、可双机分片"]
+#[ignore = "全量 1463812278 个状态（十四亿多）、每个两遍恢复 + checker + 一次可写挂载；门禁 checker-tier-layer0-replay --full 在 release 下跑，带断点续跑、可双机分片"]
 fn full_enumeration_of_the_unlink_after_the_multi_record_publish_stream_is_exhaustive_and_clean() {
     let prepared = prepare("unlink-after-multi-record-layer0-full");
     let closed_form_with_two_states_per_write = closed_form_state_count(&prepared.segments);
@@ -998,10 +999,10 @@ fn full_enumeration_of_the_unlink_after_the_multi_record_publish_stream_is_exhau
             + ((1 << 26) - 1)
             + ((1 << 24) - 1)
             + ((1 << 22) - 1)
-            + ((1 << 20) - 1),
+            + ((1 << 24) - 1),
         "每次写只取两态时的闭式：1 + Σ(2^|段| − 1)，三十五段"
     );
-    assert_eq!(closed_form_with_two_states_per_write, 1_448_083_588);
+    assert_eq!(closed_form_with_two_states_per_write, 1_463_812_228);
     let closed_form = layer0_state_count_with_torn_in_place_overwrites(
         &prepared.base,
         &prepared.writes,
@@ -1021,10 +1022,10 @@ fn full_enumeration_of_the_unlink_after_the_multi_record_publish_stream_is_exhau
             + ((1 << 26) - 1)
             + ((1 << 24) - 1)
             + ((1 << 22) - 1)
-            + ((1 << 20) - 1),
+            + ((1 << 24) - 1),
         "枚举域的闭式：十个系统配置槽 2 写段各 3² − 1、七个记录 2 写段各 3、九个 1 写段、七个单元段各 2^n − 1、4 写段 15、6 写段 63"
     );
-    assert_eq!(closed_form, 1_448_083_638);
+    assert_eq!(closed_form, 1_463_812_278);
     let observe_on_the_worker_threads: &(dyn Fn(&CrashImage<'_>, &RecoveryReport, &mut Layer0ObserverCounts)
           + Sync) = &|image, report, counts| observe_every_state(&prepared, image, report, counts);
     let tally = match enumerate_layer0_against_the_version_table_or_one_shard(
@@ -1066,7 +1067,7 @@ fn full_enumeration_of_the_unlink_after_the_multi_record_publish_stream_is_exhau
         [0, 1, 2, 3]
             .map(|publish_index| counts
                 .count_named(&format!("cut_in_deletion_publish_{publish_index}"))),
-        [(1 << 26) - 1, (1 << 24) - 1, (1 << 22) - 1, (1 << 20) - 1],
+        [(1 << 26) - 1, (1 << 24) - 1, (1 << 22) - 1, (1 << 24) - 1],
         "意图、两批、最后一批各自停在中间的每个状态都核到了"
     );
     assert_eq!(
diff --git a/crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs b/crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs
index 97077ce..022d493 100644
--- a/crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs
+++ b/crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs
@@ -1,7 +1,7 @@
 //! checker 档模块：无（自己逐个造崩溃状态，只用 harness 档的内存池与池级 checker）
 //! I-7.8（根记录树 ID 水位不低于全池最大树 ID） 扫描方向的改法 D（`research/prompts/m2-wave3-code-r1-main-verification.md`
 //! 第三节 Y1、第四节第 1 条，被攻过零轮）：第一个文件版本那次发布崩在录制流的每一个前缀上 → 可写挂载 → 挂载之后六种动作，
-//! 池级 checker 一条违例都没有。流：mkfs → 取号 → 暖机 → 第一个文件版本（从水位 65536 发十二棵树）。
+//! 池级 checker 一条违例都没有。流：mkfs → 取号 → 暖机 → 第一个文件版本（从水位 65536 发十三棵树，D8（核心索引结构） 已定项 11：退役树 65548、水位 65549）。
 //! 原先的流 ②（回退到暖机根 (1, 2) 之后在回退那个会话里再发第一个文件版本）随管理员回退改成挂着时的向前发布删掉：
 //! 向前回退的候选要带文件，退不到树表 0 条的暖机根（D23（journal 的角色与格式） 已定项 14）。
 //!
@@ -110,8 +110,8 @@ fn first_file_publish_after_mkfs() -> (
                 first_file.root.checkpoint_txg,
                 first_file.root.tree_identifier_watermark
             ),
-            (CheckpointTxg(3), 65548),
-            "流 ①：第一个文件版本 (1, 3) 从 mkfs 的水位 65536 发 65536..65547、新水位 65548"
+            (CheckpointTxg(3), 65549),
+            "流 ①：第一个文件版本 (1, 3) 从 mkfs 的水位 65536 发 65536..65548（十三棵，最后一棵是退役树 65548）、新水位 65549"
         );
         (start, stream.operation_count())
     };
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index e58037e..1e497bd 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -2913,6 +2913,9 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
             )
         },
     )?;
+    every_accounting_entry_is_as_wide_as_its_field_table(
+        &accounting_tree.leaf_entries_in_key_order,
+    )?;
     accounting_entry_count_is_three_plus_six_per_device(
         &accounting_tree.leaf_entries_in_key_order,
         root.checkpoint_txg,
@@ -4762,9 +4765,21 @@ fn allocation_records_are_one_placement_per_device_on_every_device(
     Ok(())
 }
 
+/// 每条记账条目都有字段表那么宽（条目宽是盘上的值，窄于 34 报 `EntryNarrowerThanItsFieldTable`）。排在
+/// [`accounting_entry_count_is_three_plus_six_per_device`] 之前：那一判要把条目解开才认得出 inode 号水位的台阶行（统计量标签与代段都在条目里），
+/// 条目宽坏了时台阶行解不开、被算进 3 + 6 × 盘数，条目宽这一坏法就报成了条目数不对，不是它自己的成员。
+fn every_accounting_entry_is_as_wide_as_its_field_table(
+    accounting_entry_bytes: &[Vec<u8>],
+) -> Result<(), RecoveryFailure> {
+    for entry_bytes in accounting_entry_bytes {
+        accounting_entry_parsed_at_its_field_table_width(entry_bytes)?;
+    }
+    Ok(())
+}
+
 /// 记账条目数是 3 + 6 × 盘数（D5（快照 / 空间记账机制） 已定项 8：池级三行加每盘六行），inode 号水位的台阶行不算在里面：
 /// 它们是那一统计量代段早于这一版 txg（`root_txg`）的行，随根一直留着、条数随建过 inode 的发布数涨（D5 已定项 2 射程「inode 号水位的台阶行」）。
-/// 解不开的条目照算进 3 + 6 × 盘数（它在后面逐条解的时候报宽度不对）。
+/// 解不开的条目算进 3 + 6 × 盘数；调用方先过 [`every_accounting_entry_is_as_wide_as_its_field_table`]，走到这里的每条都解得开。
 fn accounting_entry_count_is_three_plus_six_per_device(
     accounting_entry_bytes: &[Vec<u8>],
     root_txg: CheckpointTxg,
@@ -4838,18 +4853,23 @@ fn allocation_record_generations_and_spans_are_judged(
     Ok(())
 }
 
+/// 解一条记账条目（条目宽是盘上的值，窄于 34 报 `EntryNarrowerThanItsFieldTable`）。
+fn accounting_entry_parsed_at_its_field_table_width(
+    entry_bytes: &[u8],
+) -> Result<AccountingEntry, RecoveryFailure> {
+    AccountingEntry::parse(entry_bytes).ok_or(RecoveryFailure::EntryNarrowerThanItsFieldTable {
+        what: "记账条目",
+        entry_bytes: entry_bytes.len(),
+        field_table_bytes: usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
+    })
+}
+
 /// 解一条记账条目（条目宽是盘上的值，窄于 34 报 `EntryNarrowerThanItsFieldTable`），判它的代不晚于根、seq 不是 0。
 fn accounting_entry_parsed_and_judged_against_the_root(
     entry_bytes: &[u8],
     root: &RootRecord,
 ) -> Result<AccountingEntry, RecoveryFailure> {
-    let entry = AccountingEntry::parse(entry_bytes).ok_or(
-        RecoveryFailure::EntryNarrowerThanItsFieldTable {
-            what: "记账条目",
-            entry_bytes: entry_bytes.len(),
-            field_table_bytes: usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
-        },
-    )?;
+    let entry = accounting_entry_parsed_at_its_field_table_width(entry_bytes)?;
     if entry.generation > root.checkpoint_txg || entry.sequence == 0 {
         return Err(RecoveryFailure::InvariantViolated {
             invariant: "E142 走读同款",
@@ -5367,6 +5387,9 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
         allocation_records,
         &device_identities,
     )?;
+    every_accounting_entry_is_as_wide_as_its_field_table(
+        &roots.accounting.leaf_entries_in_key_order,
+    )?;
     accounting_entry_count_is_three_plus_six_per_device(
         &roots.accounting.leaf_entries_in_key_order,
         root.checkpoint_txg,
diff --git a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
index 0d7c095..d4ed586 100644
--- a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
+++ b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
@@ -30,8 +30,8 @@ use singlefs_core::pointer::{
     PointerHeadFieldOutsideTheFirstVersion,
 };
 use singlefs_core::records::{
-    mapping_key_for_node, parse_mapping_entry, TreeTableEntry, TREE_KIND_ACCOUNTING,
-    TREE_KIND_INODE, TREE_KIND_RETIREMENT,
+    mapping_key_for_node, parse_mapping_entry, TreeTableEntry, STATISTIC_INODE_WATERMARK,
+    TREE_KIND_ACCOUNTING, TREE_KIND_INODE, TREE_KIND_RETIREMENT,
 };
 use singlefs_core::recovery::{
     choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, JournalPolicy,
@@ -44,7 +44,8 @@ use singlefs_core::system_configuration::{
     SystemConfiguration, SystemImmutableSizes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
 };
 use singlefs_core::transaction::{
-    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
+    acquire_instance, publish_first_file, publish_new_inodes, warm_up, FirstFile, PoolWriter,
+    TransactionOutput,
 };
 use singlefs_core::unit::{
     build_data_unit, build_index_node, build_packed_unit, parse_data_unit, parse_index_node,
@@ -53,8 +54,8 @@ use singlefs_core::unit::{
     UNIT_CLASS_INDEX_NODE,
 };
 use singlefs_format::{
-    DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, NODE_BYTES,
-    SYSTEM_CONFIGURATION_SLOT_BYTES, UNIT_AREA_START_SLOT,
+    ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES,
+    JOURNAL_RING_DEFAULT_BYTES, NODE_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES, UNIT_AREA_START_SLOT,
 };
 use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};
 
@@ -113,6 +114,12 @@ fn formatted_pool() -> Devices {
 
 /// 两块内存盘上 mkfs → 取号 1 → 暖机 → 第一个文件（txg 3）：记账树、中央映射树都是单个叶根，inode 树根下一片叶容器。
 fn pool_after_the_first_file() -> (Devices, TransactionOutput) {
+    let (devices, _allocator, first) = pool_and_its_allocator_after_the_first_file();
+    (devices, first)
+}
+
+/// [`pool_after_the_first_file`] 连同接着发布要用的分配器。
+fn pool_and_its_allocator_after_the_first_file() -> (Devices, PoolAllocator, TransactionOutput) {
     let parameters = parameters();
     let mut devices = two_sparse_devices();
     let genesis = make_filesystem(&parameters, &mut devices).expect("默认几何上 mkfs");
@@ -138,7 +145,7 @@ fn pool_after_the_first_file() -> (Devices, TransactionOutput) {
         )
         .expect("第一个文件")
     };
-    (devices, first)
+    (devices, allocator, first)
 }
 
 /// 两块盘此刻的整份镜像：拒之前、拒之后逐字节比。
@@ -1436,6 +1443,130 @@ fn the_allocation_record_tree_of_a_version_without_file_is_judged_like_the_cold_
     );
 }
 
+// ─── 记账条目宽先于记账条目数：台阶行解不开就认不出 ───
+
+/// 两块内存盘上第一个文件（txg 3）之后再建一个空 inode（txg 4）：记账树单个叶根 16 条，池级 3 行、每盘 6 行之外，
+/// 多一级第一个文件那次留下的 inode 号水位台阶行（代段 3，早于这一版的 txg；D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）。
+fn pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row() -> Devices {
+    let (mut devices, mut allocator, first) = pool_and_its_allocator_after_the_first_file();
+    let parameters = parameters();
+    let second = {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        publish_new_inodes(
+            &mut writer,
+            &mut allocator,
+            &first,
+            1,
+            WRITE_TIME_SECONDS + 60,
+            first.root.instance,
+        )
+        .expect("建一个空 inode")
+    };
+    let step_rows_older_than_this_version = second
+        .accounting_entries
+        .iter()
+        .filter(|entry| {
+            entry.statistic == STATISTIC_INODE_WATERMARK
+                && entry.generation < second.root.checkpoint_txg
+        })
+        .count();
+    assert_eq!(
+        (
+            second.root.checkpoint_txg,
+            second.accounting_entries.len(),
+            step_rows_older_than_this_version,
+            second.accounting_tree.node_count()
+        ),
+        (CheckpointTxg(4), 16, 1, 1),
+        "txg 4 那一版的记账树：单个叶根 16 条，其中 1 条是代段早于 4 的台阶行"
+    );
+    devices
+}
+
+/// 码 2 节点自述的条目宽缩到 key 宽、每条只留 key：条目数不变，条目区紧排、空出来的那一段清零，声明长度 = 条目数 × key 宽
+/// （条目数住 82 + 2k、条目宽住 84 + 2k，声明长度住偏移 8）。
+fn narrow_every_entry_of_an_index_node_to_its_key(node: &mut [u8]) {
+    let key_width = usize::from(node[51]);
+    let header_end = 86 + 2 * key_width;
+    let entry_count = usize::from(u16::from_le_bytes([
+        node[header_end - 4],
+        node[header_end - 3],
+    ]));
+    let entry_ranges: Vec<std::ops::Range<usize>> = (0..entry_count)
+        .map(|entry_index| entry_range_in_node(node, entry_index))
+        .collect();
+    let entries_start = header_end + 29;
+    let entries_end_before = entry_ranges.last().map_or(entries_start, |range| range.end);
+    // 第 i 条的 key 挪到 i × key 宽：key 宽小于原条目宽，写到的那一段止于 (i + 1) × key 宽，盖不到还没挪的那几条。
+    for (entry_index, range) in entry_ranges.iter().enumerate() {
+        node.copy_within(
+            range.start..range.start + key_width,
+            entries_start + entry_index * key_width,
+        );
+    }
+    node[entries_start + entry_count * key_width..entries_end_before].fill(0);
+    let narrowed_entry_width = u16::try_from(key_width).expect("key 宽是 1 字节的字段");
+    node[header_end - 2..header_end].copy_from_slice(&narrowed_entry_width.to_le_bytes());
+    let declared_length =
+        u16::try_from(entry_count * key_width).expect("缩之后的条目区比缩之前短，装得进 2 字节");
+    node[8..10].copy_from_slice(&declared_length.to_le_bytes());
+}
+
+/// 最新那条根（txg 4）的记账树根条目宽 34 缩到 key 宽 22、16 条每条只留 key，节点与指着它的每一道校验和重算。
+fn pool_whose_accounting_root_carrying_a_step_row_is_narrowed_to_its_keys() -> Devices {
+    let mut devices =
+        pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
+    rewrite_the_root_node_of_a_tree_of_the_newest_root(
+        &mut devices,
+        TREE_KIND_ACCOUNTING,
+        |node| narrow_every_entry_of_an_index_node_to_its_key(node),
+    );
+    devices
+}
+
+/// 记账条目窄于字段表那个成员：22 对 34。
+fn accounting_entry_narrower_than_its_field_table() -> RecoveryFailure {
+    RecoveryFailure::EntryNarrowerThanItsFieldTable {
+        what: "记账条目",
+        entry_bytes: usize::try_from(ACCOUNTING_KEY_BYTES).expect("22"),
+        field_table_bytes: usize::try_from(ACCOUNTING_ENTRY_BYTES).expect("34"),
+    }
+}
+
+/// 场景：第一个文件之后再建一个空 inode（txg 4），记账树根 16 条里有一级代段 3 的 inode 号水位台阶行；根节点自述的条目宽缩到 key 宽、
+/// 每条只留 key（条目数 16 不变），节点与指着它的每一道校验和重算。
+/// 预期：只读恢复交回 `EntryNarrowerThanItsFieldTable`（记账条目，22 对 34）。台阶行要解开才认得出（统计量标签与代段都在条目里）：
+/// 数记账条目那一判排在条目宽之前时，解不开的台阶行被算进 3 + 6 × 盘数，16 ≠ 15，报成「记账条目数（inode 号水位的台阶行不算）
+/// 不是 3 + 6 × 盘数」。checker 档坏盘战役 `every_fixed_panic_site_reports_its_own_error_member_instead_of_panicking` 的
+/// 「条目宽：记账树根」那一种是同一个坏法，这条在 harness 档钉住次序。
+#[test]
+fn a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_recovery() {
+    let devices = pool_whose_accounting_root_carrying_a_step_row_is_narrowed_to_its_keys();
+    let report = recover(&devices, JournalPolicy::Consult);
+    assert_eq!(
+        report.outcome,
+        RecoveryOutcome::Failed {
+            root: Some((InstanceGeneration(1), CheckpointTxg(4))),
+            failure: accounting_entry_narrower_than_its_field_table(),
+        },
+        "只读恢复在 txg 4 那条根上报条目宽，不报条目数"
+    );
+}
+
+/// 场景：同上一条的坏镜像，交给可写挂载。
+/// 预期：可写挂载在任何写之前交回 `EntryNarrowerThanItsFieldTable`（记账条目，22 对 34）：重建那一版读记账树时同样先核条目宽，
+/// 后数条目（判法与上一条同一个理由）。
+#[test]
+fn a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_writable_mount(
+) {
+    let mut devices = pool_whose_accounting_root_carrying_a_step_row_is_narrowed_to_its_keys();
+    let refusal = writable_mount_refused_before_any_write(&mut devices);
+    assert!(
+        matches!(&refusal, MountError::Recovery(failure) if *failure == accounting_entry_narrower_than_its_field_table()),
+        "可写挂载报条目宽，不报条目数：{refusal:?}"
+    );
+}
+
 // ─── 第 35 条续：单元区槽数那一处减法 ───
 
 /// 场景：700 MiB 的盘（单元区起点在 784 MiB）。预期：单元区槽数交回「盘末尾在单元区起点之前」，带盘的字节数与起点槽号；
`````

### mutations-append.tsv

`````text
第52批 冷走读记账条目数排回条目宽之前（台阶行解不开被算进 3 + 6 × 盘数，条目宽坏法报成条目数）	crates/singlefs-core/src/recovery.rs	    every_accounting_entry_is_as_wide_as_its_field_table(\n        &roots.accounting.leaf_entries_in_key_order,\n    )?;\n    accounting_entry_count_is_three_plus_six_per_device(\n        &roots.accounting.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_count,\n    )?;	    accounting_entry_count_is_three_plus_six_per_device(\n        &roots.accounting.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_count,\n    )?;\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &roots.accounting.leaf_entries_in_key_order,\n    )?;	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_recovery	a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_recovery
第52批 重建记账条目数排回条目宽之前（台阶行解不开被算进 3 + 6 × 盘数，条目宽坏法报成条目数）	crates/singlefs-core/src/recovery.rs	    every_accounting_entry_is_as_wide_as_its_field_table(\n        &accounting_tree.leaf_entries_in_key_order,\n    )?;\n    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_identities.len(),\n    )?;	    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_identities.len(),\n    )?;\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &accounting_tree.leaf_entries_in_key_order,\n    )?;	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_writable_mount	a_narrowed_accounting_root_carrying_a_step_row_is_refused_for_its_entry_width_by_the_writable_mount
`````

## m3-batch53/gaps

### crates.patch

`````diff
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index 947f433..0267acd 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -103,7 +103,12 @@ pub const SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY: &str =
 /// `research/prompts/m3-retire-r1-sonnet-output.md` K5.1，第 50 批）：I-9.31、I-9.32、I-9.33、I-9.34、I-9.35，同样只往末尾追加。
 /// I-9.31 在主走读里判（退役树的内部条目，与 I-9.17、I-9.23 同一个走法），其余在墓碑与谱系那一遍里每一版各判（I-9.35 跨相邻两版比）。
 /// GPU 判器同判 I-9.31，其余四条在有退役记录的状态上判不了（`crash_judge_gpu` 的覆盖表与它的文档）。
-pub const IMPLEMENTED_INVARIANTS: [&str; 90] = [
+///
+/// 第 91、92 条（下标 90、91）是谱系树与退役树的容器容量那两条（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2，
+/// 第 53 批，与 I-9.21、I-9.22 同形）：I-9.36、I-9.37，同样只往末尾追加。在墓碑与谱系那一遍里每一版各判；I-9.37 的旋钮值从镜像外给
+/// （`walk::check_pool_image_given_the_container_capacity_knobs`）。GPU 判器的扫描内核同判 I-9.36，I-9.37 照 I-9.22 恒不置位
+/// （层 0 与放量走旧入口，拿不到旋钮值）。
+pub const IMPLEMENTED_INVARIANTS: [&str; 92] = [
     "I-1.1",
     "I-1.2",
     "I-1.3",
@@ -194,6 +199,8 @@ pub const IMPLEMENTED_INVARIANTS: [&str; 90] = [
     RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
     RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
     RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+    NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+    LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
 ];
 
 /// I-1.12：走读跟随的一条指针，偏移 34 的出生树等于被指单元头里的出生树——码 1、码 3 取头偏移 43；码 2 取哪个字段随 C289 定，
@@ -320,6 +327,14 @@ pub const RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF: &str = "I-9.34";
 /// 读得出才判）：父版叶身份集 − 本版叶身份集 = 本版谱系上、死亡 txg == 本版 txg 的退役记录的身份集，两个方向各报。最新根的实例表读不出、
 /// 滤不掉被崩溃恢复抛弃的根时报部分没判（照 I-9.29 的门）。被攻过零轮（判决第四节）。
 pub const RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION: &str = "I-9.35";
+/// I-9.36（产品镜像里非最右谱系与退役容器装满，与 I-9.21 同形）：那一棵没装只供测试的容量旋钮时，每一版谱系树与退役树里容器号沿叶序严格递增、
+/// 最右那片之外每片记录数 == 产品容量（谱系 1359、退役 959；开放容器满了才关，D18（块里携带什么信息） 已定项 24）。
+pub const NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE: &str =
+    "I-9.36";
+/// I-9.37（测试镜像里谱系与退役容器不超旋钮值，与 I-9.22 同形）：那一棵装了旋钮（新入口给了旋钮值）时，每一版那棵树里容器号沿叶序严格递增、
+/// 每片记录数 ≤ 旋钮值。
+pub const LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE: &str =
+    "I-9.37";
 
 /// 判定累加器：每条不变量记评估了几次、第一处违例、整条不适用的理由，以及射程里有一部分判不了的理由。
 #[derive(Default)]
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 8a9d3e0..e95abeb 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -10,10 +10,10 @@ use singlefs_format::{
     ACCOUNTING_ENTRY_BYTES, ALLOCATION_RECORD_BYTES, ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES,
     CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES,
     EXTENT_TREE_INTERNAL_ENTRY_BYTES, EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES, INODE_INTERNAL_ENTRY,
-    JOURNAL_RECORD_BYTES, JOURNAL_SAFETY_FACTOR, MAPPING_ENTRY_BYTES, NODE_BYTES,
-    NODE_POINTER_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES, RETIREMENT_CONTAINER_RECORDS,
-    SLOT_BYTES, TOMBSTONE_CONTAINER_RECORDS, TREE_IDENTIFIER_WATERMARK_AT_MKFS,
-    TREE_TABLE_ENTRY_BYTES,
+    JOURNAL_RECORD_BYTES, JOURNAL_SAFETY_FACTOR, LINEAGE_CONTAINER_RECORDS, MAPPING_ENTRY_BYTES,
+    NODE_BYTES, NODE_POINTER_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES,
+    RETIREMENT_CONTAINER_RECORDS, SLOT_BYTES, TOMBSTONE_CONTAINER_RECORDS,
+    TREE_IDENTIFIER_WATERMARK_AT_MKFS, TREE_TABLE_ENTRY_BYTES,
 };
 
 use crate::position_addressed::{
@@ -54,8 +54,10 @@ use crate::image::{
     INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
     JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
     LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
+    LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
     LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER, MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
     NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
+    NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
     NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
     ONE_OBJECT_BIRTH_PER_INODE_NUMBER, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
     POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER, PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
@@ -10054,6 +10056,71 @@ impl TombstoneContainerCapacityKnob {
     }
 }
 
+/// 只供测试的谱系容器容量旋钮压到的每片记录数（同墓碑容器那一个：旋钮不落盘，池级 checker 从镜像外拿；判决
+/// `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2，I-9.37）。取值 1..=1359，构造时验。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct LineageContainerCapacityKnob {
+    records_per_container: u64,
+}
+
+impl LineageContainerCapacityKnob {
+    /// 1..=1359 之外交回 `None`（1359 = ⌊(32768 − 136) ÷ 24⌋，产品容量 `LINEAGE_CONTAINER_RECORDS`）。
+    #[must_use]
+    pub fn new(records_per_container: u64) -> Option<Self> {
+        (1..=LINEAGE_CONTAINER_RECORDS)
+            .contains(&records_per_container)
+            .then_some(Self {
+                records_per_container,
+            })
+    }
+
+    #[must_use]
+    pub const fn records_per_container(self) -> u64 {
+        self.records_per_container
+    }
+}
+
+/// 只供测试的退役容器容量旋钮压到的每片记录数（同上，I-9.37）。取值 1..=959，构造时验。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct RetirementContainerCapacityKnob {
+    records_per_container: u64,
+}
+
+impl RetirementContainerCapacityKnob {
+    /// 1..=959 之外交回 `None`（959 = ⌊(32768 − 136) ÷ 34⌋，产品容量 `RETIREMENT_CONTAINER_RECORDS`）。
+    #[must_use]
+    pub fn new(records_per_container: u64) -> Option<Self> {
+        (1..=RETIREMENT_CONTAINER_RECORDS)
+            .contains(&records_per_container)
+            .then_some(Self {
+                records_per_container,
+            })
+    }
+
+    #[must_use]
+    pub const fn records_per_container(self) -> u64 {
+        self.records_per_container
+    }
+}
+
+/// 写这张镜像时三棵容器树各压没压容量（从镜像外给）：`None` 的那一棵当没装旋钮，照产品容量判（墓碑树 I-9.21，谱系树、退役树 I-9.36）；
+/// `Some` 的那一棵按旋钮值判（墓碑树 I-9.22，谱系树、退役树 I-9.37）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct ContainerCapacityKnobs {
+    pub tombstone: Option<TombstoneContainerCapacityKnob>,
+    pub lineage: Option<LineageContainerCapacityKnob>,
+    pub retirement: Option<RetirementContainerCapacityKnob>,
+}
+
+impl ContainerCapacityKnobs {
+    /// 三棵都没装旋钮（产品镜像，[`check_pool_image`] 那一份）。
+    pub const PRODUCT: Self = Self {
+        tombstone: None,
+        lineage: None,
+        retirement: None,
+    };
+}
+
 /// 删除线新九条（I-9.17–I-9.23、I-9.29；I-9.24 的在 [`judge_deletion_state_of_the_versions`] 末尾）一次都没判到时各自报的不适用理由：
 /// 每条一句、说清这张镜像上为什么没有它判的对象，不落到 `Judgements::into_report` 的缺省理由「这个镜像上没有它判的对象」。
 /// 公开给「逐条判定、不在豁免里的一律要求成立」的用例逐字认（豁免只认这一句，别的不适用照红）。
@@ -10081,6 +10148,18 @@ pub const PRODUCT_TOMBSTONE_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOB: &str =
 /// I-9.22，新入口给了旋钮值：每一版墓碑树都是零根。
 pub const NO_TOMBSTONE_CONTAINER_UNDER_THE_KNOB: &str =
     "走读到的每一版墓碑树都是零根：没有墓碑容器可判";
+/// I-9.36（产品镜像里非最右谱系与退役容器装满），没装旋钮的那一棵（或两棵）上：每一版都是零根或只一片容器。
+pub const NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER: &str =
+    "走读到的每一版里没装旋钮的谱系树与退役树都是零根或只一片容器：没有非最右的容器";
+/// I-9.36，谱系树与退役树都装了旋钮（新入口给了两个旋钮值）：产品容量那一条不判。
+pub const PRODUCT_LINEAGE_AND_RETIREMENT_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOBS: &str =
+    "这张镜像的谱系与退役容量都用了只供测试的旋钮（新入口给了两个旋钮值）：产品容量那一条不判";
+/// I-9.37（测试镜像里谱系与退役容器不超旋钮值），两棵都没装旋钮（旧入口，或新入口没给这两个旋钮值）。
+pub const NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB: &str =
+    "池级 checker 没拿到只供测试的谱系与退役容量旋钮（旧入口，或新入口没给这两个旋钮值）：旋钮那一条不判";
+/// I-9.37，装了旋钮的那一棵（或两棵）在每一版都是零根。
+pub const NO_LINEAGE_OR_RETIREMENT_CONTAINER_UNDER_THE_KNOBS: &str =
+    "走读到的每一版里装了旋钮的谱系树与退役树都是零根：没有容器可判";
 /// I-9.17（谱系树条目身份与子头相符）：主走读一条谱系树内部条目都没走到。
 pub const NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
     "走读到的每一版谱系树都是零根：没有谱系树内部条目可判";
@@ -10141,6 +10220,40 @@ const fn capacity_invariant_of(
     }
 }
 
+/// I-9.36 / I-9.37 判的那两棵容器树：说明里怎么称呼、产品容量。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum LineageOrRetirementTree {
+    Lineage,
+    Retirement,
+}
+
+impl LineageOrRetirementTree {
+    const fn name(self) -> &'static str {
+        match self {
+            LineageOrRetirementTree::Lineage => "谱系树",
+            LineageOrRetirementTree::Retirement => "退役树",
+        }
+    }
+
+    /// 一片码 3 容器的产品容量（D18（块里携带什么信息） 已定项 24：开放容器满了才关）。
+    const fn product_records_per_container(self) -> u64 {
+        match self {
+            LineageOrRetirementTree::Lineage => LINEAGE_CONTAINER_RECORDS,
+            LineageOrRetirementTree::Retirement => RETIREMENT_CONTAINER_RECORDS,
+        }
+    }
+}
+
+/// 一棵谱系树或退役树装没装旋钮（旋钮值，每片记录数）决定它判哪一条：没装判产品那一条 I-9.36，装了判测试那一条 I-9.37（同 [`capacity_invariant_of`]）。
+const fn lineage_or_retirement_capacity_invariant_of(
+    knob_records_per_container: Option<u64>,
+) -> &'static str {
+    match knob_records_per_container {
+        None => NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+        Some(_) => LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
+    }
+}
+
 /// 一条墓碑记录的五段。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct TombstoneRecordOfTheScan {
@@ -10190,6 +10303,19 @@ struct LineageRecordOfTheScan {
     inode_number_watermark_of_the_version_rolled_back_from: u64,
 }
 
+impl LineageRecordOfTheScan {
+    fn of(record: &[u8]) -> Self {
+        Self {
+            rollback_txg: read_u64(record, LINEAGE_RECORD_ROLLBACK_TXG_OFFSET),
+            parent_txg: read_u64(record, LINEAGE_RECORD_PARENT_TXG_OFFSET),
+            inode_number_watermark_of_the_version_rolled_back_from: read_u64(
+                record,
+                LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET,
+            ),
+        }
+    }
+}
+
 /// 一版的父链在 txg 轴上占的那几段（每段闭区间）：「在谱系上」= 死亡 txg 落在其中一段里。
 struct LineageChainOfAVersion {
     segments_on_the_chain: Vec<(u64, u64)>,
@@ -10358,6 +10484,40 @@ fn tombstone_containers_without_judging(
         .collect())
 }
 
+/// 一片谱系容器：头里的容器号（I-9.36 / I-9.37 沿叶序比它）与它的记录。
+struct LineageContainerOfTheScan {
+    container: u64,
+    records: Vec<LineageRecordOfTheScan>,
+}
+
+/// 一版的谱系树按叶序读出的每片容器与记录，只读不判。
+fn lineage_containers_without_judging(
+    reader: &dyn ImageReader,
+    root_pointer: &PointerView,
+    index_cache: &mut IndexNodeCache,
+    packed_cache: &mut PackedUnitCache,
+) -> Result<Vec<LineageContainerOfTheScan>, DeletionScanGap> {
+    let containers = containers_of_a_container_tree_without_judging(
+        reader,
+        root_pointer,
+        PACKED_TYPE_LINEAGE,
+        DeletionScanGap::LineageTreeUnreadable,
+        index_cache,
+        packed_cache,
+    )?;
+    Ok(containers
+        .iter()
+        .map(|container| LineageContainerOfTheScan {
+            container: container.container,
+            records: container
+                .records
+                .iter()
+                .map(|record| LineageRecordOfTheScan::of(record))
+                .collect(),
+        })
+        .collect())
+}
+
 /// 一版的谱系树里的全部谱系记录，只读不判。
 fn lineage_records_without_judging(
     reader: &dyn ImageReader,
@@ -10376,14 +10536,7 @@ fn lineage_records_without_judging(
     Ok(containers
         .iter()
         .flat_map(|container| container.records.iter())
-        .map(|record| LineageRecordOfTheScan {
-            rollback_txg: read_u64(record, LINEAGE_RECORD_ROLLBACK_TXG_OFFSET),
-            parent_txg: read_u64(record, LINEAGE_RECORD_PARENT_TXG_OFFSET),
-            inode_number_watermark_of_the_version_rolled_back_from: read_u64(
-                record,
-                LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET,
-            ),
-        })
+        .map(|record| LineageRecordOfTheScan::of(record))
         .collect())
 }
 
@@ -10988,10 +11141,23 @@ fn judge_tombstones_and_the_lineage_of_the_versions(
     reader: &dyn ImageReader,
     versions: &[VersionToScanTheDeletionStateOf<'_>],
     abandoned_roots_filtering: AbandonedRootsFilteringOfTheCandidateSet,
-    capacity_knob: Option<TombstoneContainerCapacityKnob>,
+    container_capacity_knobs: ContainerCapacityKnobs,
     index_cache: &mut IndexNodeCache,
     judgements: &mut Judgements,
 ) {
+    let capacity_knob = container_capacity_knobs.tombstone;
+    let lineage_and_retirement_capacity_invariants = [
+        lineage_or_retirement_capacity_invariant_of(
+            container_capacity_knobs
+                .lineage
+                .map(LineageContainerCapacityKnob::records_per_container),
+        ),
+        lineage_or_retirement_capacity_invariant_of(
+            container_capacity_knobs
+                .retirement
+                .map(RetirementContainerCapacityKnob::records_per_container),
+        ),
+    ];
     let mut packed_cache = PackedUnitCache::new();
     let mut tree_tables_and_txgs_scanned: BTreeSet<(u32, u64, u32, u64)> = BTreeSet::new();
     let mut object_births: ObjectBirthsOfEachInodeNumber = BTreeMap::new();
@@ -11016,6 +11182,7 @@ fn judge_tombstones_and_the_lineage_of_the_versions(
                 .into_iter()
                 .chain(RETIREMENT_SCAN_INVARIANTS)
                 .chain([capacity_invariant])
+                .chain(lineage_and_retirement_capacity_invariants)
             {
                 judgements.part_of_the_range_not_judged(
                     invariant,
@@ -11053,6 +11220,7 @@ fn judge_tombstones_and_the_lineage_of_the_versions(
                 .into_iter()
                 .chain(RETIREMENT_SCAN_INVARIANTS)
                 .chain([capacity_invariant])
+                .chain(lineage_and_retirement_capacity_invariants)
             {
                 judgements.part_of_the_range_not_judged(invariant, gap.reason());
             }
@@ -11080,6 +11248,7 @@ fn judge_tombstones_and_the_lineage_of_the_versions(
                 version,
                 &tree_table.entries,
                 capacity_knob,
+                container_capacity_knobs,
                 index_cache,
                 &mut packed_cache,
                 &mut object_births,
@@ -11161,18 +11330,44 @@ fn judge_tombstones_and_the_lineage_of_the_versions(
             );
         }
     }
+    let (product_capacity_reason, knob_capacity_reason) = match (
+        container_capacity_knobs.lineage,
+        container_capacity_knobs.retirement,
+    ) {
+        (None, None) => (
+            NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER,
+            NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB,
+        ),
+        (Some(_), None) | (None, Some(_)) => (
+            NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER,
+            NO_LINEAGE_OR_RETIREMENT_CONTAINER_UNDER_THE_KNOBS,
+        ),
+        (Some(_), Some(_)) => (
+            PRODUCT_LINEAGE_AND_RETIREMENT_CONTAINER_CAPACITY_NOT_JUDGED_UNDER_THE_KNOBS,
+            NO_LINEAGE_OR_RETIREMENT_CONTAINER_UNDER_THE_KNOBS,
+        ),
+    };
+    judgements.not_applicable(
+        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+        product_capacity_reason,
+    );
+    judgements.not_applicable(
+        LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
+        knob_capacity_reason,
+    );
 }
 
 /// [`judge_tombstones_and_the_lineage_of_the_versions`] 的一版：读谱系（父链）、每棵 inode 树的记录与每棵墓碑树，再逐条判。
 #[allow(
     clippy::too_many_arguments,
-    reason = "版本、树表条目、旋钮、两份缓存、跨版本那张表与判定累加器各有名字与类型，不为凑短签名收成结构体"
+    reason = "版本、树表条目、墓碑旋钮与三棵容器树的旋钮、两份缓存、跨版本那张表与判定累加器各有名字与类型，不为凑短签名收成结构体"
 )]
 fn judge_tombstones_and_the_lineage_of_a_version(
     reader: &dyn ImageReader,
     version: &VersionToScanTheDeletionStateOf<'_>,
     tree_table_entries: &[Vec<u8>],
     capacity_knob: Option<TombstoneContainerCapacityKnob>,
+    container_capacity_knobs: ContainerCapacityKnobs,
     index_cache: &mut IndexNodeCache,
     packed_cache: &mut PackedUnitCache,
     object_births: &mut ObjectBirthsOfEachInodeNumber,
@@ -11185,15 +11380,45 @@ fn judge_tombstones_and_the_lineage_of_a_version(
             .collect()
     };
     // 一张树表恰好一条谱系树条目（I-9.16 ④）：这一版的父链从它的记录与这一版自己的 txg 起算。
-    let lineage_records = match entries_of_kind(TREE_KIND_LINEAGE).as_slice() {
-        [lineage_entry] => lineage_records_without_judging(
+    let lineage_tree = match entries_of_kind(TREE_KIND_LINEAGE).as_slice() {
+        [lineage_entry] => lineage_containers_without_judging(
             reader,
             &parse_node_pointer(&lineage_entry[14..100]),
             index_cache,
             packed_cache,
-        ),
+        )
+        .map(|containers| (read_u64(lineage_entry, 0), containers)),
         [] | [_, _, ..] => Err(DeletionScanGap::LineageTreeEntryNotSingle),
     };
+    // I-9.36 / I-9.37 在谱系树上（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2）：读不全的报射程里判不了。
+    let lineage_capacity_knob = container_capacity_knobs
+        .lineage
+        .map(LineageContainerCapacityKnob::records_per_container);
+    match &lineage_tree {
+        Ok((lineage_tree_identifier, containers)) => {
+            judge_lineage_or_retirement_container_capacity_of_a_version(
+                &version.name,
+                LineageOrRetirementTree::Lineage,
+                *lineage_tree_identifier,
+                &containers
+                    .iter()
+                    .map(|container| (container.container, container.records.len()))
+                    .collect::<Vec<(u64, usize)>>(),
+                lineage_capacity_knob,
+                judgements,
+            );
+        }
+        Err(gap) => judgements.part_of_the_range_not_judged(
+            lineage_or_retirement_capacity_invariant_of(lineage_capacity_knob),
+            gap.reason(),
+        ),
+    }
+    let lineage_records = lineage_tree.map(|(_, containers)| {
+        containers
+            .into_iter()
+            .flat_map(|container| container.records)
+            .collect::<Vec<LineageRecordOfTheScan>>()
+    });
     let lineage_chain = lineage_records
         .clone()
         .and_then(|records| lineage_chain_of_a_version(version.checkpoint_txg, &records));
@@ -11388,6 +11613,9 @@ fn judge_tombstones_and_the_lineage_of_a_version(
         );
     }
     let mut retired_at_this_txg_of_each_inode_tree = BTreeMap::new();
+    let retirement_capacity_knob = container_capacity_knobs
+        .retirement
+        .map(RetirementContainerCapacityKnob::records_per_container);
     for retirement_tree_entry in entries_of_kind(TREE_KIND_RETIREMENT) {
         let retirement_tree = RetirementTreeOfTheScan {
             tree: read_u64(retirement_tree_entry, 0),
@@ -11405,7 +11633,9 @@ fn judge_tombstones_and_the_lineage_of_a_version(
         let containers = match &retirement_tree.containers {
             Ok(containers) => containers,
             Err(gap) => {
-                for invariant in RETIREMENT_SCAN_INVARIANTS {
+                for invariant in RETIREMENT_SCAN_INVARIANTS.into_iter().chain([
+                    lineage_or_retirement_capacity_invariant_of(retirement_capacity_knob),
+                ]) {
                     judgements.part_of_the_range_not_judged(invariant, gap.reason());
                 }
                 retired_at_this_txg_of_each_inode_tree
@@ -11413,6 +11643,17 @@ fn judge_tombstones_and_the_lineage_of_a_version(
                 continue;
             }
         };
+        judge_lineage_or_retirement_container_capacity_of_a_version(
+            &version.name,
+            LineageOrRetirementTree::Retirement,
+            retirement_tree.tree,
+            &containers
+                .iter()
+                .map(|container| (container.container, container.records.len()))
+                .collect::<Vec<(u64, usize)>>(),
+            retirement_capacity_knob,
+            judgements,
+        );
         judge_every_retirement_record_of_a_version_is_well_formed(
             version,
             retirement_tree.tree,
@@ -11728,6 +11969,63 @@ fn judge_tombstones_on_the_lineage_of_a_version_are_distinct(
     }
 }
 
+/// I-9.36 / I-9.37（第 53 批，判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2，与 I-9.21 / I-9.22 同形）：这一版谱系树或
+/// 退役树按叶序的容器（容器号, 记录数），容器号严格递增（读容器头，不沿根的条目读）；没装旋钮判 I-9.36——最右那片之外每片记录数 == 产品容量
+/// （谱系 1359、退役 959；开放容器满了才关），零根或只一片容器没有对象；装了旋钮判 I-9.37——每片记录数 ≤ 旋钮值，零根没有对象。
+fn judge_lineage_or_retirement_container_capacity_of_a_version(
+    version_name: &str,
+    tree_kind: LineageOrRetirementTree,
+    tree: u64,
+    container_numbers_and_record_counts: &[(u64, usize)],
+    knob_records_per_container: Option<u64>,
+    judgements: &mut Judgements,
+) {
+    let invariant = lineage_or_retirement_capacity_invariant_of(knob_records_per_container);
+    let tree_name = tree_kind.name();
+    let judged_when_the_tree_has_at_least = match knob_records_per_container {
+        None => 2,
+        Some(_) => 1,
+    };
+    if container_numbers_and_record_counts.len() < judged_when_the_tree_has_at_least {
+        return;
+    }
+    for adjacent in container_numbers_and_record_counts.windows(2) {
+        let (left_container, right_container) = (adjacent[0].0, adjacent[1].0);
+        judgements.judge(invariant, left_container < right_container, || {
+            format!(
+                "{version_name}：{tree_name}（树 {tree}）沿叶序容器号不严格递增：容器 {left_container} 之后是容器 {right_container}"
+            )
+        });
+    }
+    for (position_in_leaf_order, (container, record_count)) in
+        container_numbers_and_record_counts.iter().enumerate()
+    {
+        let record_count = u64::try_from(*record_count).expect("一片容器的记录数装得进 u64");
+        match knob_records_per_container {
+            None => {
+                let is_the_rightmost =
+                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len();
+                if is_the_rightmost {
+                    continue;
+                }
+                let product_records = tree_kind.product_records_per_container();
+                judgements.judge(invariant, record_count == product_records, || {
+                    format!(
+                        "{version_name}：{tree_name}（树 {tree}）里不是最右的容器 {container} 装了 {record_count} 条，不是 {product_records}（满了才关）"
+                    )
+                });
+            }
+            Some(records_the_knob_allows) => {
+                judgements.judge(invariant, record_count <= records_the_knob_allows, || {
+                    format!(
+                        "{version_name}：{tree_name}（树 {tree}）的容器 {container} 装了 {record_count} 条，多于旋钮值 {records_the_knob_allows}"
+                    )
+                });
+            }
+        }
+    }
+}
+
 /// I-9.21 / I-9.22（判决第二节 T5 取 K1、T6 每版）：这一版墓碑树按叶序的容器，容器号严格递增（读容器头，不沿根的条目读）；
 /// 没装旋钮判 I-9.21——最右那片之外每片记录数 == 815，零根或只一片容器没有对象；装了旋钮判 I-9.22——每片记录数 ≤ 旋钮值，零根没有对象。
 fn judge_tombstone_container_capacity_of_a_version(
@@ -11803,11 +12101,12 @@ fn judge_one_object_birth_per_inode_number(
     }
 }
 
-/// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。当没装只供测试的墓碑容量旋钮（产品镜像）：
-/// I-9.21（产品镜像里非最右墓碑容器装满） 照产品容量判，I-9.22（测试镜像里墓碑容器不超旋钮值） 报不适用。
+/// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。当三棵容器树都没装只供测试的容量旋钮（产品镜像）：
+/// I-9.21（产品镜像里非最右墓碑容器装满） 与 I-9.36（产品镜像里非最右谱系与退役容器装满） 照产品容量判，
+/// I-9.22、I-9.37（测试镜像里容器不超旋钮值） 报不适用。
 #[must_use]
 pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, InvariantVerdict)> {
-    check_pool_image_given_the_tombstone_container_capacity_knob(reader, None)
+    check_pool_image_given_the_container_capacity_knobs(reader, ContainerCapacityKnobs::PRODUCT)
 }
 
 /// 池级 checker 带旋钮值的入口（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T5 取 K1）：旋钮不落盘
@@ -11817,6 +12116,25 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
 pub fn check_pool_image_given_the_tombstone_container_capacity_knob(
     reader: &dyn ImageReader,
     tombstone_container_capacity_knob: Option<TombstoneContainerCapacityKnob>,
+) -> Vec<(&'static str, InvariantVerdict)> {
+    check_pool_image_given_the_container_capacity_knobs(
+        reader,
+        ContainerCapacityKnobs {
+            tombstone: tombstone_container_capacity_knob,
+            lineage: None,
+            retirement: None,
+        },
+    )
+}
+
+/// 池级 checker 带三棵容器树旋钮值的入口（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2，同墓碑那一个入口的
+/// 判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T5 取 K1）：旋钮不落盘，写这张镜像时三棵容器树各压没压容量由调用方
+/// 从镜像外给。[`ContainerCapacityKnobs::PRODUCT`] 与 [`check_pool_image`] 相同；墓碑旋钮 `Some` 时 I-9.22 按旋钮值判、I-9.21 报不适用；
+/// 谱系或退役旋钮 `Some` 时那一棵按 I-9.37 判，`None` 的那一棵照 I-9.36 判。
+#[must_use]
+pub fn check_pool_image_given_the_container_capacity_knobs(
+    reader: &dyn ImageReader,
+    container_capacity_knobs: ContainerCapacityKnobs,
 ) -> Vec<(&'static str, InvariantVerdict)> {
     let system_configurations = chosen_system_configurations(reader);
     let mut root_ring_judgements = Judgements::default();
@@ -12506,7 +12824,7 @@ pub fn check_pool_image_given_the_tombstone_container_capacity_knob(
         reader,
         &versions_to_scan_the_deletion_state_of,
         abandoned_roots_filtering,
-        tombstone_container_capacity_knob,
+        container_capacity_knobs,
         &mut index_node_cache,
         &mut judgements,
     );
diff --git a/crates/singlefs-checker-tier/src/bad_disk_input.rs b/crates/singlefs-checker-tier/src/bad_disk_input.rs
index c2b2359..f6d43f7 100644
--- a/crates/singlefs-checker-tier/src/bad_disk_input.rs
+++ b/crates/singlefs-checker-tier/src/bad_disk_input.rs
@@ -178,8 +178,9 @@ pub enum DamageKind {
     NarrowedEntryWidthOfTheAllocationRecordsTreeRoot,
     /// 条目宽度那一族（普查 R4、R13）：改记账树根（key 宽 22，条目该有 34），条目宽缩到 key 宽，**每条条目只留它的 key、
     /// 条目数不变**，节点头的 key 区间照旧与首末两条对得上。只缩成 1 条（extent、inode、分配记录树根那三种的缩法）过不了判全那一道：可写挂载重建与冷走读
-    /// 都照冷走读判全（代码审阅第 24 / 33 条），先红在「根 key 区间与条目不符」与「记账条目数不是 3 + 6 × 盘数」上，
-    /// 走不到解记账条目那一步；那一种原来的样子留在 [`DamageRefusedAtTheHeaderJudgement`]。
+    /// 都照冷走读判全（代码审阅第 24 / 33 条），先红在「根 key 区间与条目不符」上；重建不判节点头时（变异「实审 A3d」那一形），
+    /// 第 52 批起重建数条目之前先核条目宽，可写挂载红在「记账条目宽不到字段表」上，不再是「记账条目数不是 3 + 6 × 盘数」；
+    /// 都走不到逐条解记账条目那一步；那一种原来的样子留在 [`DamageRefusedAtTheHeaderJudgement`]。
     NarrowedEntryWidthOfTheAccountingTreeRoot,
     /// 条目宽度那一族（普查 R2、R13）：同上，改中央映射树根（key 宽 27，条目该有 55），每条只留 key、条目数不变
     /// （映射条目数同样要对得上进映射的单元数）。映射树的根住根记录、不经树表
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
index f6a7c48..d27c6e1 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
@@ -198,6 +198,13 @@ const INVARIANT_I_9_34: u32 = 88u;
 
 const INVARIANT_I_9_35: u32 = 89u;
 
+// 第 91、92 条：第 53 批谱系树与退役树的容器容量那两条（walk.rs 的 NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE、
+// LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE）。扫描内核在墓碑与谱系那一遍判 I-9.36；I-9.37 照 I-9.22 恒不置位
+// （层 0 与放量走池级 checker 的旧入口，GPU 也没有旋钮值）
+const INVARIANT_I_9_36: u32 = 90u;
+
+const INVARIANT_I_9_37: u32 = 91u;
+
 // 墓碑树、谱系树叶容器的打包记录类型（walk.rs 的 PACKED_TYPE_TOMBSTONE / PACKED_TYPE_LINEAGE；登记的记录宽墓碑 40、谱系 24）
 const PACKED_TYPE_TOMBSTONE: u32 = 1u;
 
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index 6413004..aea78e0 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -2168,6 +2168,9 @@ const DELETION_TOMBSTONE_CONTAINER_RECORDS: u32 = 815u;
 // 谱系记录 24 字节（类型 5 登记 24）：T_rb 0、T_old 8、H_cur 16（回退之前的现行那一版的 inode 号水位），读作「T_rb 的父是 T_old」
 // （walk.rs LINEAGE_RECORD_*_OFFSET，D18（块里携带什么信息） 已定项 24 第 5 条）
 const DELETION_LINEAGE_RECORD_BYTES: u32 = 24u;
+
+// 一片谱系容器的产品容量 ⌊(32768 − 136) ÷ 24⌋ = 1359（walk.rs 的 LINEAGE_CONTAINER_RECORDS；I-9.36 判非最右那几片装满）
+const DELETION_LINEAGE_CONTAINER_RECORDS: u32 = 1359u;
 const DELETION_LINEAGE_RECORD_ROLLBACK_TXG_OFFSET: u32 = 0u;
 const DELETION_LINEAGE_RECORD_PARENT_TXG_OFFSET: u32 = 8u;
 const DELETION_LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET: u32 = 16u;
@@ -3025,6 +3028,7 @@ fn d_read_the_lineage_of_a_version(tt: Node, version_txg: vec2<u32>) -> bool {
         PACKED_TYPE_LINEAGE, DELETION_LINEAGE_RECORD_BYTES)) {
         return false;
     }
+    d_judge_lineage_or_retirement_container_capacity(DELETION_LINEAGE_CONTAINER_RECORDS);
     for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
         let base = d_container_base(container_index);
         for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
@@ -3151,6 +3155,26 @@ fn d_judge_tombstone_container_capacity() {
     }
 }
 
+// I-9.36（walk.rs judge_lineage_or_retirement_container_capacity_of_a_version 没装旋钮那一支，第 53 批，与 I-9.21 同形）：读进
+// D_CONTAINERS_OF_A_CONTAINER_TREE 的谱系或退役容器至少两片时，沿叶序容器号严格递增（读容器头，不沿根的条目读），最右那片之外每片记录数 ==
+// 产品容量 product_records（谱系 1359、退役 959）；零根或只一片没有对象。I-9.37 不置位（同 I-9.22）。谱系树在 d_read_the_lineage_of_a_version 读完容器那一处调，
+// 退役树在墓碑与谱系那一遍读完退役容器那一处调（与 walk.rs 一样：谱系树要恰好一条条目、容器读得全才判）
+fn d_judge_lineage_or_retirement_container_capacity(product_records: u32) {
+    if (d_container_count < 2u) {
+        return;
+    }
+    for (var container_index = 1u; container_index < d_container_count; container_index = container_index + 1u) {
+        if (!lt64(d_container_number(container_index - 1u), d_container_number(container_index))) {
+            violate(INVARIANT_I_9_36);
+        }
+    }
+    for (var container_index = 0u; container_index + 1u < d_container_count; container_index = container_index + 1u) {
+        if (d_container_record_count(container_index) != product_records) {
+            violate(INVARIANT_I_9_36);
+        }
+    }
+}
+
 // I-9.20 管全部墓碑的那几句（walk.rs judge_every_tombstone_of_a_version_is_well_formed）：每片容器记录数 ≤ 815（与 I-1.7 重合，留作合取）；
 // 每条整对象、对象出生代 ≤ 死亡 txg ≤ 这一版的 txg
 fn d_judge_every_tombstone_is_well_formed(version_txg: vec2<u32>) {
@@ -3807,6 +3831,9 @@ fn d_judge_tombstones_and_the_lineage_of_a_version(base: u32, tree_table_pointer
             continue;
         }
         if (kind != TREE_KIND_TOMBSTONE) {
+            if (is_a_retirement_tree) {
+                d_judge_lineage_or_retirement_container_capacity(DELETION_RETIREMENT_CONTAINER_RECORDS);
+            }
             d_judge_the_retirement_records_and_their_completeness(tt, entry, version_txg, lineage_holds, parent);
             continue;
         }
diff --git a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
index 72055fe..6f54846 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -1347,7 +1347,8 @@ mod tests {
     /// 照 `kernel_inlined_size` 同一个数法量的：草稿工具在旧源码上量回 331 792 / 346 138 / 352 195，与旧钉值逐个相等，三个内核都解析、校验得过）；
     /// 删除线第 50 批退役树（恢复与走读认种类 13、走读内核判 I-9.31，扫描内核判 I-9.32–I-9.35：退役容器借墓碑树那一处读容器的调用点、叶身份只有一个读的调用点）
     /// 恢复 332 638 → 332 666、走读 346 307 → 346 278、扫描 366 870 → 387 154（第 50 批补做实现员副本上用同一个草稿工具量的，它在主工作区那一版上量回
-    /// 331 792 / 346 138 / 352 195；四条一开始各留调用点时扫描量到 454 049、超预算，改成单调用点才落回预算里）；
+    /// 331 792 / 346 138 / 352 195；四条一开始各留调用点时扫描量到 454 049、超预算，改成单调用点才落回预算里）；第 53 批扫描内核判 I-9.36
+    /// （谱系树、退役树读完容器各一个调用点）扫描 387 154 → 387 470（第 53 批实现员副本上用同一个草稿工具量的，它在改之前那一版上量回 387 154）；
     /// 第 36 批之前是恢复 320 015、扫描 382 259、走读 337 872）：
     /// 改了哪个内核的源码，这里先红，量过新数、确认余量还够之后再改钉值，免得余量悄悄耗光、等上卡时才撞预算。
     #[test]
@@ -1357,8 +1358,8 @@ mod tests {
             (
                 "checker-scan",
                 crate::crash_judge_gpu::checker_scan_kernel_source(),
-                387_154,
-                12_846,
+                387_470,
+                12_530,
             ),
             (
                 "checker-walk",
diff --git a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
index 6c85764..bed7196 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -60,7 +60,7 @@ const _: () = assert!(
     "违例位读成一个 u128"
 );
 
-/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 90 条——三批 49 条，加里程碑三二·1 那一批 16 条
+/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 92 条——三批 49 条，加里程碑三二·1 那一批 16 条
 /// （判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行），下标 65 的 I-3.5（引用区间的精确性，
 /// 走读内核在回退候选集的版本都走完之后逐对判），再加 C673（三 oracle 合谋测试的补 oracle 改法没做）⑤ ⑥ 那两条
 /// （下标 66 是 I-9.27，走读内核挨着 I-9.16 按表判；下标 67 是 I-5.5，扫描内核每一版重读树表与 extent 树判），
@@ -89,12 +89,14 @@ const _: () = assert!(
 /// I-9.35（父版照 walk.rs 取、在版本清单里恰好一版才比，最新根的实例表沿链读不出时整条不比）。判决同一条原写「其余回落 CPU」，
 /// 用户 2026-09-29 定「配了 GPU 就一定要 GPU 判，不退回 CPU」，第 50 批补做把这四条搬上 GPU、不记判不了；读不全的那一部分不置位（CPU 报射程里判不了），
 /// 叶身份表与两两比的退役记录越过上限判不了。
+/// 下标 90、91 是第 53 批谱系树与退役树的容器容量那两条（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2）：扫描内核在
+/// 墓碑与谱系那一遍判 I-9.36（谱系树读完容器那一处、退役树读完容器那一处，各一个调用点）；下标 91 的 I-9.37 照下标 79 的先例恒不置位。
 /// 用户 2026-10-04 定「改回要求全覆盖」：没有「内核还没判」那一张表。
 pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
     26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
     50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
-    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89,
+    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91,
 ];
 
 // 覆盖表就是池级 checker 的每一条，按下标升序：第 k 项是 k。新立一条不变量而内核没跟上时这里编译不过，不悄悄让 GPU 那一位恒 0。
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
index 8daa938..3f2a91d 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
@@ -9,8 +9,10 @@ mod common;
 use common::{build_pool, file_content, geometry};
 use singlefs_checker::image::{
     DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY, EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
-    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
-    MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
+    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
+    LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
+    LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER, MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
+    NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
     NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
     ONE_OBJECT_BIRTH_PER_INODE_NUMBER, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
     RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
@@ -237,6 +239,12 @@ fn assert_checker_counts(
         RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
         RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
     ];
+    // 谱系树与退役树的容器容量那两条（第 53 批）：这条流上两棵都是零根，没有非最右的容器（I-9.36 没有对象）；层 0 走池级 checker 的旧入口，
+    // 没有谱系与退役的旋钮值（I-9.37 一格都不判）。推的：两条都是 0/0，没跑。
+    let never_has_two_lineage_or_retirement_containers_on_this_stream = [
+        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+        LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
+    ];
     // I-4.4（意图只描述目标态） 第一版报不适用（用户 2026-10-05 定），哪一条流上都评估不到；对得上同一份日志的 `I-4.4=0/0`。
     let not_applicable_in_the_first_version = [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE];
     // I-8.6 判的是「本实例内逻辑前一条」与这一条的链值，环里一条自证过的记录都没有时判不了：
@@ -263,6 +271,7 @@ fn assert_checker_counts(
             || never_has_an_inode_being_deleted_on_this_stream.contains(&invariant)
             || never_has_a_tombstone_or_lineage_record_on_this_stream.contains(&invariant)
             || never_has_a_retirement_record_on_this_stream.contains(&invariant)
+            || never_has_two_lineage_or_retirement_containers_on_this_stream.contains(&invariant)
             || not_applicable_in_the_first_version.contains(&invariant)
         {
             0
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index faa50e2..c597a1e 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -8238,6 +8238,92 @@ fn the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_names_the_survivin
     );
 }
 
+/// I-9.36 退役树那一形（第 53 批，判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2；攻方 `opus_probe_container_closed_early`）：
+/// 新池新建文件之外再建 233 个空 inode（叶 [1..=233]、[234]），写入口装退役容器一片 1 条的旋钮，unlink 234 到底（最右那片删空、退役一条），
+/// 再建 1 个（235 开新叶）、unlink 235 到底（又退役一条）：退役树两片容器各 1 条。旋钮下的合法镜像；CPU 判器走池级 checker 的旧入口
+/// （当没装旋钮），不是最右的那片只装 1 条、不是 959，判 I-9.36 红；GPU 扫描内核只有没装旋钮那一支，同判。
+fn record_two_leaves_retired_with_one_retirement_record_per_container(tag: &str) -> RecordedFlow {
+    let mut pool = pool_with_new_inodes(tag, INODE_RECORDS_PER_LEAF);
+    let capacities = ContainerTreeCapacities {
+        tombstone_container: TombstoneContainerCapacity::FromTheContainerFormat,
+        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
+                singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(1)
+                    .expect("1..=959"),
+            ),
+        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
+    };
+    let rightmost_leaf_inode = FIRST_FILE.0 + INODE_RECORDS_PER_LEAF;
+    unlink_an_empty_inode_to_the_end_under_the_capacities(
+        &mut pool,
+        rightmost_leaf_inode,
+        capacities,
+    );
+    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+    unlink_an_empty_inode_to_the_end_under_the_capacities(
+        &mut pool,
+        rightmost_leaf_inode + 1,
+        capacities,
+    );
+    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
+    recorded_flow_of_the_pool(
+        &pool,
+        "two-leaves-retired-one-retirement-record-per-container",
+        versions,
+    )
+}
+
+/// I-9.36 谱系树那一形：新池新建文件、建 1 个空 inode（txg 4），分配器装上谱系容器一片 1 条的旋钮（回退的发布路径照分配器上的开关取容量），
+/// 挂着回退到第一个文件那一版（txg 3）两次：谱系树两片容器各 1 条。旋钮下的合法镜像；CPU 判器走旧入口判 I-9.36 红，GPU 同判。
+fn record_two_rollbacks_with_one_lineage_record_per_container(tag: &str) -> RecordedFlow {
+    let mut pool = build_pool(tag);
+    let first_file_version = pool.output.clone();
+    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+    pool.allocator
+        .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
+            batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
+            tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
+            lineage_container_capacity: LineageContainerCapacity::CappedForTests(
+                singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(1)
+                    .expect("1..=1359"),
+            ),
+            retirement_container_capacity:
+                singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
+            container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
+        });
+    roll_back_to(&mut pool, &first_file_version);
+    roll_back_to(&mut pool, &first_file_version);
+    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
+    recorded_flow_of_the_pool(
+        &pool,
+        "two-rollbacks-one-lineage-record-per-container",
+        versions,
+    )
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_retirement_container_closes_before_it_is_full() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_retirement_container_closes_before_it_is_full",
+        &record_two_leaves_retired_with_one_retirement_record_per_container(
+            "crash-judge-gpu-i936-retirement-container-closed-early",
+        ),
+        "I-9.36",
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_lineage_container_closes_before_it_is_full() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_lineage_container_closes_before_it_is_full",
+        &record_two_rollbacks_with_one_lineage_record_per_container(
+            "crash-judge-gpu-i936-lineage-container-closed-early",
+        ),
+        "I-9.36",
+    );
+}
+
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_when_the_retirement_record_of_a_gone_leaf_dies_at_the_parent_version(
 ) {
diff --git a/crates/singlefs-core/src/inode_tree.rs b/crates/singlefs-core/src/inode_tree.rs
index 55a7a6a..3cfd2d1 100644
--- a/crates/singlefs-core/src/inode_tree.rs
+++ b/crates/singlefs-core/src/inode_tree.rs
@@ -1221,6 +1221,45 @@ mod tests {
         assert_eq!(record_counts(&containers), vec![233, 233]);
     }
 
+    /// 三片各 117 条、删中间那片的一条（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F1，攻方第二次抽样 K4-4）：
+    /// 中间那片剩 116 条，与左邻、右邻之和都恰好 233，两对都装得下。M3「先查右邻」⇒ 中间那片吸收右邻，退役的是右邻（容器号 467），
+    /// 左邻原样照抄；先查左邻的写法让左邻吸收中间那片、退役的是中间那片（容器号 234）。两种写法的镜像池级 checker 都不红，方向只有这里钉。
+    #[test]
+    fn removing_from_the_middle_of_three_half_full_containers_absorbs_the_right_neighbour_first() {
+        let mut before = full_containers(3);
+        for position in 0..3 {
+            keep_the_first_records(&mut before, position, 117);
+        }
+        let after = remove_record_from_leaf_containers(&before, InodeNumber(300)).expect("删 300");
+        assert_eq!(
+            container_numbers(&after.containers),
+            vec![1, 234],
+            "中间那片吸收右邻：留下左邻与中间那片"
+        );
+        assert_eq!(record_counts(&after.containers), vec![117, 233]);
+        assert_eq!(after.containers[0], before[0], "左邻一个字节都不动");
+        assert_eq!(
+            after.containers[1].identity, before[1].identity,
+            "吸收方保留中间那片的身份"
+        );
+        assert_eq!(
+            after.retired,
+            vec![RetiredInodeLeafContainer {
+                previous_index: InodeLeafContainerIndexInTree::of_position(2),
+                identity: before[2].identity,
+            }],
+            "先查右邻：退役的是右邻，不是中间那片"
+        );
+        assert_eq!(
+            after.rewritten,
+            vec![InodeLeafContainerIndexInTree::of_position(1)]
+        );
+        assert_eq!(
+            after.origins[0],
+            InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::LEFTMOST)
+        );
+    }
+
     /// 删的是树里唯一一条（unlink 唯一的文件）：一片都不剩、没有要重写的片，那片身份消失，根照样重写成零条目根。
     #[test]
     fn removing_the_last_record_of_the_tree_leaves_no_container_and_rewrites_the_root() {
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index a05480e..73d16f7 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -7837,6 +7837,26 @@ fn ring_roots_not_abandoned_by_crash_recovery(
         .collect()
 }
 
+/// 管理员回退那次发布的 inode 号水位（D23（journal 的角色与格式） 已定项 14「水位」一格的 Opt2）：max(cur 那一版内存里的水位,
+/// 根环里按 cur 的实例表没被崩溃恢复抛弃的那几条根各自那一版的水位)。环里那一项由 `highest_inode_number_watermark_of` 去盘上重建
+/// （回退路径交 [`highest_inode_number_watermark_in_the_ring`]）；取哪几条根、怎么与 cur 那一项合在这里，构造态单测直接调它
+/// （判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F3：环那一项在走得到的状态上不大于 cur，去掉它整套用例照绿）。
+/// cur 那一项不能去（只取环，回退那次就重发号，C143）。
+fn inode_number_watermark_of_the_rollback(
+    ring_roots: &[RootRecord],
+    current_table: &InstanceTableRecords,
+    inode_number_watermark_of_the_current_version: u64,
+    highest_inode_number_watermark_of: impl FnOnce(&[RootRecord]) -> Option<u64>,
+) -> u64 {
+    highest_inode_number_watermark_of(&ring_roots_not_abandoned_by_crash_recovery(
+        ring_roots,
+        current_table,
+    ))
+    .map_or(inode_number_watermark_of_the_current_version, |ring| {
+        ring.max(inode_number_watermark_of_the_current_version)
+    })
+}
+
 /// 回退那次发布接在后面的「上一版」：用户可见的那一半（数据单元与指针、extent 树、inode 树的叶容器与根、第一个文件那条 inode 记录）
 /// 取回退目标那一版的，别的（分配记录树与它的记录、记账树与记账行、中央映射树、树表、实例表各片）取现行那一版的；
 /// 根记录、末条记录、事务号取现行那一版的（这次发布接着它发，反向链、计数器、txg 都接着它）。
@@ -8114,15 +8134,19 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
         .fold(current.root.tree_identifier_watermark, u64::max);
     // inode 号水位（Opt2，D23（journal 的角色与格式） 已定项 14「水位」一格）：环里只取按 cur 的实例表没被崩溃恢复抛弃的根；
     // max 里 cur 那一版内存里的那一项不能去——只取环，回退那次就重发号（C143）。
-    let inode_number_watermark = highest_inode_number_watermark_in_the_ring(
-        devices,
-        unit_area_start,
-        &ring_roots_not_abandoned_by_crash_recovery(&ring_roots, &current_table),
-        &current.record,
-    )
-    .map_or(current.inode_number_watermark(), |ring| {
-        ring.max(current.inode_number_watermark())
-    });
+    let inode_number_watermark = inode_number_watermark_of_the_rollback(
+        &ring_roots,
+        &current_table,
+        current.inode_number_watermark(),
+        |ring_roots_not_abandoned| {
+            highest_inode_number_watermark_in_the_ring(
+                devices,
+                unit_area_start,
+                ring_roots_not_abandoned,
+                &current.record,
+            )
+        },
+    );
     let previous = version_to_publish_the_rollback_after(current, &target_version);
     // 动分配器：先复活、再释放（释放代 = 新根的 txg），之后交给发布路径——它在这份账上照常释放 cur 被换下的固定点单元、取落点、
     // 装记账行。发布在落盘之前被拒时整个换回这一份之前的样子。
@@ -9283,3 +9307,120 @@ mod packed_units_named_by_the_records_of_a_publish_as_read_tests {
         );
     }
 }
+
+/// 管理员回退取 inode 号水位那一段（Opt2，D23（journal 的角色与格式） 已定项 14「水位」一格）的构造态单测：环里那一项在走得到的历史上
+/// 不大于 cur 内存里的，去掉它回退用例与随机历史全绿（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第二节 K1、第三节 F3），
+/// 这里直接交一份环里那条根的水位高于 cur 的输入。
+#[cfg(test)]
+mod inode_number_watermark_of_the_rollback_tests {
+    use std::collections::BTreeMap;
+
+    use super::inode_number_watermark_of_the_rollback;
+    use crate::address::{CheckpointTxg, InstanceGeneration};
+    use crate::instance_table::{InstanceRow, InstanceTableRecords};
+    use crate::pointer::NodePointer;
+    use crate::root_record::{RootRecord, UnmountMarker};
+
+    /// cur 那一版内存里的 inode 号水位。
+    const INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION: u64 = 40;
+    /// 环里那条根（实例 1、txg 7）那一版的 inode 号水位：高于 cur 的。
+    const INODE_NUMBER_WATERMARK_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION: u64 = 90;
+    /// 环里另一条根（实例 1、txg 6）那一版的水位：低于 cur 的。
+    const INODE_NUMBER_WATERMARK_OF_THE_RING_ROOT_BELOW_THE_CURRENT_VERSION: u64 = 25;
+    const INSTANCE_OF_THE_RING_ROOTS: InstanceGeneration = InstanceGeneration(1);
+    const TXG_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION: CheckpointTxg = CheckpointTxg(7);
+    const TXG_OF_THE_RING_ROOT_BELOW_THE_CURRENT_VERSION: CheckpointTxg = CheckpointTxg(6);
+
+    fn ring_root(checkpoint_txg: CheckpointTxg) -> RootRecord {
+        RootRecord {
+            filesystem_identifier: [7; 16],
+            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
+            instance: INSTANCE_OF_THE_RING_ROOTS,
+            checkpoint_txg,
+            tree_table: NodePointer::empty_root(),
+            tree_identifier_watermark: 65_549,
+            rollback_floor: CheckpointTxg(0),
+            instance_table: NodePointer::empty_root(),
+            mapping_root: NodePointer::empty_root(),
+            allocation_record_tree_root: NodePointer::empty_root(),
+        }
+    }
+
+    /// cur 的实例表：实例 1 那一行的所选根 txg 是 `selected_root_txg`（txg 大于它的实例 1 的根被崩溃恢复抛弃）。
+    fn current_table_selecting(selected_root_txg: CheckpointTxg) -> InstanceTableRecords {
+        InstanceTableRecords {
+            rows: vec![InstanceRow {
+                instance: INSTANCE_OF_THE_RING_ROOTS,
+                selected_root_txg,
+                applied_transaction_high_water: 0,
+            }],
+        }
+    }
+
+    /// 替盘上重建：交进来的每条根那一版的水位（按 txg 查），取最大；一条都没交进来是 `None`。
+    fn highest_watermark_of_the_roots_handed_in(roots: &[RootRecord]) -> Option<u64> {
+        let watermark_of_each_txg = BTreeMap::from([
+            (
+                TXG_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION,
+                INODE_NUMBER_WATERMARK_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION,
+            ),
+            (
+                TXG_OF_THE_RING_ROOT_BELOW_THE_CURRENT_VERSION,
+                INODE_NUMBER_WATERMARK_OF_THE_RING_ROOT_BELOW_THE_CURRENT_VERSION,
+            ),
+        ]);
+        roots
+            .iter()
+            .map(|root| {
+                *watermark_of_each_txg
+                    .get(&root.checkpoint_txg)
+                    .expect("只交这几条构造出来的根")
+            })
+            .max()
+    }
+
+    /// 环里一条没被抛弃的根（实例表里实例 1 选到 txg 9，txg 7 不大于它）那一版的水位 90 高于 cur 的 40：取 90。
+    #[test]
+    fn a_ring_root_not_abandoned_whose_watermark_is_above_the_current_version_sets_the_watermark() {
+        assert_eq!(
+            inode_number_watermark_of_the_rollback(
+                &[ring_root(TXG_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION)],
+                &current_table_selecting(CheckpointTxg(9)),
+                INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION,
+                highest_watermark_of_the_roots_handed_in,
+            ),
+            INODE_NUMBER_WATERMARK_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION,
+            "环里那条根没被抛弃，水位取它与 cur 的最大"
+        );
+    }
+
+    /// 同一条根，实例表里实例 1 只选到 txg 5（txg 7 > 5，被崩溃恢复抛弃）：它那一版的水位不算，取 cur 的 40。
+    #[test]
+    fn a_ring_root_abandoned_by_the_instance_table_does_not_set_the_watermark() {
+        assert_eq!(
+            inode_number_watermark_of_the_rollback(
+                &[ring_root(TXG_OF_THE_RING_ROOT_ABOVE_THE_CURRENT_VERSION)],
+                &current_table_selecting(CheckpointTxg(5)),
+                INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION,
+                highest_watermark_of_the_roots_handed_in,
+            ),
+            INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION,
+            "被崩溃恢复抛弃的根那一版发过的号本来就会重发，水位不跳到它"
+        );
+    }
+
+    /// 环里没被抛弃的根那一版的水位 25 低于 cur 的 40：cur 那一项不能去（只取环，回退那次就重发号，C143），取 40。
+    #[test]
+    fn a_ring_root_whose_watermark_is_below_the_current_version_leaves_the_current_watermark() {
+        assert_eq!(
+            inode_number_watermark_of_the_rollback(
+                &[ring_root(TXG_OF_THE_RING_ROOT_BELOW_THE_CURRENT_VERSION)],
+                &current_table_selecting(CheckpointTxg(9)),
+                INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION,
+                highest_watermark_of_the_roots_handed_in,
+            ),
+            INODE_NUMBER_WATERMARK_OF_THE_CURRENT_VERSION,
+            "水位取环与 cur 的最大，不只取环"
+        );
+    }
+}
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 1e497bd..7e45cc4 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -2561,7 +2561,7 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
     let inode_root_pointer = pointer_of(TREE_KIND_INODE)?;
     let allocation_pointer = pointer_of(TREE_KIND_ALLOCATION)?;
     let accounting_pointer = pointer_of(TREE_KIND_ACCOUNTING)?;
-    // 这一版那十二棵树各自的号照盘上读回来：十一棵按树表条目的种类取条目里的树 ID，中央映射树不进树表、取根记录里
+    // 这一版那十三棵树各自的号照盘上读回来：十二棵按树表条目的种类取条目里的树 ID，中央映射树不进树表、取根记录里
     // 它那条根指针的出生树（D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11）。接着这一版的发布照抄它们，不另发。
     let tree_of = |kind: u16| {
         tree_table_entries
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index b77b191..9f249c8 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -3169,7 +3169,7 @@ impl TransactionOutput {
             .root
     }
 
-    /// 树表条目的诞生 txg（十一条同一个数：树建起来那次发布）。
+    /// 树表条目的诞生 txg（十二条同一个数：树建起来那次发布）。
     #[must_use]
     pub fn tree_birth_txg(&self) -> CheckpointTxg {
         self.tree_table_entries
@@ -8636,7 +8636,7 @@ fn publish_admitted<Device: BlockDevice>(
     );
     let mapping_pointer = mapping_built.version.root_pointer();
 
-    // t8 树表单元：十一条按树 ID 升序（D8（核心索引结构） 已定项 8）；映射树的根住根记录、不进树表（D19（块指针的结构与宽度预算） 已定项 11）；
+    // t8 树表单元：十二条按树 ID 升序（D8（核心索引结构） 已定项 8）；映射树的根住根记录、不进树表（D19（块指针的结构与宽度预算） 已定项 11）；
     // 头 ID（D5（快照 / 空间记账机制） 已定项 9）：inode 树写自己，extent 树、`deleted_inodes` 与墓碑树写它服务的头（inode 树的树 ID），其余 0。
     let table_entry =
         |kind: u16, tree: TreeIdentifier, root: NodePointer, head_identifier: u64| TreeTableEntry {
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 9240663..09b0882 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -28,7 +28,9 @@ use singlefs_checker::image::{
     INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
     JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
     LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
+    LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
     LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER, NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
+    NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
     NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
     ONE_OBJECT_BIRTH_PER_INODE_NUMBER, POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
     PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
@@ -1904,10 +1906,23 @@ fn expected_verdict_of_the_tombstone_and_lineage_invariants_without_deletion(
         | RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF => InvariantVerdict::NotApplicable(
             NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
         ),
+        // 谱系树与退役树的容器容量那两条（第 53 批）：两棵都是零根，没有非最右的容器；旧入口没有谱系与退役的旋钮值。
+        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE => {
+            InvariantVerdict::NotApplicable(NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER)
+        }
+        LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE => {
+            InvariantVerdict::NotApplicable(NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB)
+        }
         _ => InvariantVerdict::Holds,
     }
 }
 
+/// 谱系树与退役树的容器容量那两条没有对象时报的理由（`walk` 的 `judge_tombstones_and_the_lineage_of_the_versions` 末尾）。
+const NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER: &str =
+    "走读到的每一版里没装旋钮的谱系树与退役树都是零根或只一片容器：没有非最右的容器";
+const NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB: &str =
+    "池级 checker 没拿到只供测试的谱系与退役容量旋钮（旧入口，或新入口没给这两个旋钮值）：旋钮那一条不判";
+
 /// 退役树那几条没有对象时报的理由（`walk` 的 `judge_tombstones_and_the_lineage_of_the_versions` 与 `check_pool_image` 末尾）。
 const NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
     "走读到的每一版退役树都是零根：没有退役树内部条目可判";
@@ -3072,6 +3087,26 @@ const RETIREMENT_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(&str, &str); 5] =
     ),
 ];
 
+/// 谱系树与退役树的容器容量那两条（I-9.36、I-9.37，第 53 批，判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2）的坏镜像
+/// 各要一份旋钮压小写出的基底（攻方 `opus_probe_container_closed_early` 那一形：两次叶身份消失、两次回退），不进只改干净镜像一处的表。
+const LINEAGE_AND_RETIREMENT_CONTAINER_CAPACITY_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(
+    &str,
+    &str,
+); 3] = [
+    (
+        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+        "checker_known_bad_images.rs: two_leaves_retired_one_record_per_retirement_container_judged_as_a_product_image_reddens_only_the_full_container_invariant",
+    ),
+    (
+        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+        "checker_known_bad_images.rs: two_rollbacks_one_record_per_lineage_container_judged_as_a_product_image_reddens_only_the_full_container_invariant",
+    ),
+    (
+        LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
+        "checker_known_bad_images.rs: a_lineage_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant",
+    ),
+];
+
 /// I-3.14（实例表每片记录形态合法） 的三张坏镜像各建在攻方 N5 那一份基底上（崩溃恢复抛弃 C、C 与现行线同号不同出生代，
 /// [`image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed`]），不进只改干净镜像一处的表；
 /// 这里登记它们在哪几个用例里（各自断言判红的是 I-3.14 与这一片解不开带出来的那几条）。最后一条不是合法的链指针记录那一格的坏镜像在
@@ -3240,6 +3275,11 @@ fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target()
                 .iter()
                 .map(|(invariant, _)| *invariant),
         )
+        .chain(
+            LINEAGE_AND_RETIREMENT_CONTAINER_CAPACITY_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE
+                .iter()
+                .map(|(invariant, _)| *invariant),
+        )
         .collect();
     let listed: std::collections::BTreeSet<&str> = singlefs_checker::image::IMPLEMENTED_INVARIANTS
         .iter()
@@ -10946,6 +10986,338 @@ fn a_retirement_record_naming_a_leaf_the_parent_version_did_not_have_reddens_onl
     );
 }
 
+/// 退役记录里被退役身份的容器出生代那 8 字节的偏移（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2 条：出生树 0、
+/// 打包记录类型 8、容器号 10、容器出生代 18、死亡 txg 26）。
+const RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET: usize = 18;
+
+/// I-9.32（退役记录合法）「容器出生代 < 死亡 txg」那一句只红自己（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F4，
+/// 攻方第二次抽样 K7 那条变异）：删的那几版转出候选集之后（I-9.35 在记录死的那一版上不再比），那条退役记录被退役身份的容器出生代改成它自己的
+/// 死亡 txg。改了出生代的身份既不重也不是活叶，I-9.33、I-9.34 照旧成立。
+#[test]
+fn a_retirement_record_whose_container_birth_is_not_below_its_death_txg_reddens_only_the_well_formed_retirement_record_invariant(
+) {
+    let (pool, _, _) =
+        pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
+            "known-bad-retirement-record-born-at-its-death",
+        );
+    let mut image = pool.memory_pool();
+    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
+        &mut image,
+        |record| {
+            let death_txg = common_deletion_in_progress::retirement_record_field(
+                record,
+                common_deletion_in_progress::RETIREMENT_RECORD_DEATH_TXG_OFFSET,
+            );
+            common_deletion_in_progress::set_retirement_record_field(
+                record,
+                RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET,
+                death_txg,
+            );
+        },
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [EVERY_RETIREMENT_RECORD_IS_WELL_FORMED],
+        "被退役身份的容器出生代不小于死亡 txg：只红 I-9.32：{verdicts:?}"
+    );
+}
+
+/// I-9.32「死亡 txg ≤ 这一版的 txg」那一句只红自己（同上）：同一份基底上那条退役记录的死亡 txg 改成最新那一版的 txg + 1。
+/// 每一版里它都晚于那一版、不在谱系上，I-9.33、I-9.34 不判它；没有一版的 txg 等于它，I-9.35 照旧成立。
+#[test]
+fn a_retirement_record_dying_after_the_newest_version_reddens_only_the_well_formed_retirement_record_invariant(
+) {
+    let (pool, _, _) =
+        pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
+            "known-bad-retirement-record-dies-after-the-newest-version",
+        );
+    let death_txg_after_the_newest_version = pool.output.root.checkpoint_txg.0 + 1;
+    let mut image = pool.memory_pool();
+    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
+        &mut image,
+        |record| {
+            common_deletion_in_progress::set_retirement_record_field(
+                record,
+                common_deletion_in_progress::RETIREMENT_RECORD_DEATH_TXG_OFFSET,
+                death_txg_after_the_newest_version,
+            );
+        },
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [EVERY_RETIREMENT_RECORD_IS_WELL_FORMED],
+        "死亡 txg 晚于每一版：只红 I-9.32：{verdicts:?}"
+    );
+}
+
+/// I-1.7（打包容器合法与判定顺序） 记录宽那一步对类型 3（退役，登记 34；判决第三节 F4，正推 K6.3 的 C-retwidth）：三个单元的文件删到底之后，
+/// 退役容器头里的记录宽 34 → 33。墓碑与谱系那一遍读退役容器时遇到记录宽不是登记的那一个，退役那几条在那一版判不了、不判红：只红 I-1.7。
+#[test]
+fn a_retirement_container_whose_record_width_is_not_thirty_four_reddens_only_the_packed_container_invariant(
+) {
+    let (pool, _, _) =
+        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-record-width");
+    let mut image = pool.memory_pool();
+    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
+        &mut image,
+        TREE_KIND_RETIREMENT,
+        u16::try_from(singlefs_format::RETIREMENT_RECORD_BYTES - 1).expect("33"),
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-1.7"],
+        "退役容器的记录宽不是 34：只红 I-1.7：{verdicts:?}"
+    );
+}
+
+/// 写入口装 `capacities`（只供测试的容器树旋钮），把空 inode `inode` unlink 到底：意图那一次，再走空（空 inode 没有数据单元，直接是最后一批）。
+fn unlink_an_empty_inode_to_the_end_under_the_capacities(
+    pool: &mut BuiltPool,
+    inode: u64,
+    capacities: singlefs_core::container_tree::ContainerTreeCapacities,
+) {
+    let publish_parameters = parameters();
+    let previous = pool.output.clone();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    writer.set_container_tree_capacities(capacities);
+    let mut current = singlefs_core::transaction::publish_unlink_intent(
+        &mut writer,
+        &mut pool.allocator,
+        &previous,
+        singlefs_core::address::InodeNumber(inode),
+        common_deletion_in_progress::WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
+        common_deletion_in_progress::INSTANCE_OF_THE_PUBLISHES,
+    )
+    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
+    singlefs_core::transaction::publish_unlink_steps_until_the_queue_is_empty(
+        &mut writer,
+        &mut pool.allocator,
+        &mut current,
+        singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
+        common_deletion_in_progress::INSTANCE_OF_THE_PUBLISHES,
+    )
+    .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的各批：{refusal:?}"));
+    pool.output = current;
+}
+
+/// 写这几份退役树时退役容器压到的每片记录数（谱系、墓碑与节点照产品）。
+const RETIREMENT_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;
+
+/// 新池新建文件之外再建 233 个空 inode（叶 [1..=233]、[234]），写入口装退役容器一片 1 条的旋钮：unlink 234 到底（最右那片删空，退役一条），
+/// 再建 1 个（235，最左那片满了，开新叶）、unlink 235 到底（又退役一条）。退役树两片容器各 1 条——攻方 `opus_probe_container_closed_early`
+/// 退役那一形（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第二节 K3-4）。旋钮下的合法镜像。
+fn pool_with_two_leaves_retired_one_record_per_retirement_container(tag: &str) -> BuiltPool {
+    let capacities = singlefs_core::container_tree::ContainerTreeCapacities {
+        tombstone_container:
+            singlefs_core::tombstone_tree::TombstoneContainerCapacity::FromTheContainerFormat,
+        lineage_container:
+            singlefs_core::lineage_tree::LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
+                singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(
+                    RETIREMENT_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
+                )
+                .expect("1..=959"),
+            ),
+        node_entries: singlefs_core::container_tree::ContainerTreeNodeEntries::FromTheNodeFormat,
+    };
+    let mut pool = common_deletion_in_progress::pool_with_new_inodes(tag, 233);
+    unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, 234, capacities);
+    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+    unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, 235, capacities);
+    pool
+}
+
+/// I-9.36（产品镜像里非最右谱系与退役容器装满，第 53 批）退役树那一形：旋钮压小写出的两片各 1 条的退役树，交给池级 checker 的产品入口
+/// （当没装旋钮），不是最右的那片只装 1 条、不是 959：只红 I-9.36（墓碑树同形红 I-9.21）；交给带旋钮值的入口（退役一片 1 条）一条都不红、
+/// I-9.37 真被评估过且成立。
+#[test]
+fn two_leaves_retired_one_record_per_retirement_container_judged_as_a_product_image_reddens_only_the_full_container_invariant(
+) {
+    let pool = pool_with_two_leaves_retired_one_record_per_retirement_container(
+        "known-bad-retirement-container-closed-early",
+    );
+    assert_eq!(
+        pool.output
+            .retirement_containers
+            .iter()
+            .map(|container| container.contents.records.len())
+            .collect::<Vec<_>>(),
+        [1, 1],
+        "退役树两片容器各 1 条"
+    );
+    let image = pool.memory_pool();
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE],
+        "产品入口：不是最右的退役容器只装 1 条：只红 I-9.36：{verdicts:?}"
+    );
+    let verdicts_given_the_knob =
+        singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
+            &image,
+            singlefs_checker::walk::ContainerCapacityKnobs {
+                tombstone: None,
+                lineage: None,
+                retirement: Some(
+                    singlefs_checker::walk::RetirementContainerCapacityKnob::new(
+                        RETIREMENT_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
+                    )
+                    .expect("1..=959"),
+                ),
+            },
+        );
+    assert_eq!(
+        violated_invariants(&verdicts_given_the_knob),
+        Vec::<&str>::new(),
+        "带旋钮值的入口：旋钮下的合法镜像一条都不红：{verdicts_given_the_knob:?}"
+    );
+    assert_eq!(
+        verdict_among(
+            &verdicts_given_the_knob,
+            LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE
+        ),
+        InvariantVerdict::Holds,
+        "I-9.37 在退役树上真被评估过且成立"
+    );
+}
+
+/// 新池新建文件之后建 1 个空 inode（txg 4），分配器装上谱系容器一片 `lineage_records_per_container` 条的旋钮（回退的发布路径照分配器上的开关取容量），
+/// 挂着回退到第一个文件那一版两次：谱系树两条记录。攻方 `opus_probe_container_closed_early` 谱系那一形。
+fn pool_rolled_back_twice_under_the_lineage_knob(
+    tag: &str,
+    lineage_records_per_container: u64,
+) -> BuiltPool {
+    let mut pool = build_pool(tag);
+    let first_file_version = pool.output.clone();
+    common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+    pool.allocator
+        .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
+            batch_size: singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
+            tombstone_container_capacity:
+                singlefs_core::tombstone_tree::TombstoneContainerCapacity::FromTheContainerFormat,
+            lineage_container_capacity:
+                singlefs_core::lineage_tree::LineageContainerCapacity::CappedForTests(
+                    singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(
+                        lineage_records_per_container,
+                    )
+                    .expect("1..=1359"),
+                ),
+            retirement_container_capacity:
+                singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
+            container_tree_node_entries:
+                singlefs_core::container_tree::ContainerTreeNodeEntries::FromTheNodeFormat,
+        });
+    common_deletion_in_progress::roll_back_to(&mut pool, &first_file_version);
+    common_deletion_in_progress::roll_back_to(&mut pool, &first_file_version);
+    pool
+}
+
+/// 谱系容器一片 1 条时写这几份谱系树。
+const LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;
+
+fn lineage_record_counts_of_each_container(pool: &BuiltPool) -> Vec<usize> {
+    pool.output
+        .lineage_containers
+        .iter()
+        .map(|container| container.contents.records.len())
+        .collect()
+}
+
+fn check_pool_image_given_the_lineage_knob(
+    image: &MemoryPool,
+    lineage_records_per_container: u64,
+) -> Vec<(&'static str, InvariantVerdict)> {
+    singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
+        image,
+        singlefs_checker::walk::ContainerCapacityKnobs {
+            tombstone: None,
+            lineage: Some(
+                singlefs_checker::walk::LineageContainerCapacityKnob::new(
+                    lineage_records_per_container,
+                )
+                .expect("1..=1359"),
+            ),
+            retirement: None,
+        },
+    )
+}
+
+/// I-9.36 谱系树那一形：旋钮压小写出的两片各 1 条的谱系树，产品入口判不是最右的那片只装 1 条、不是 1359：只红 I-9.36；
+/// 带旋钮值的入口（谱系一片 1 条）一条都不红、I-9.37 真被评估过且成立。
+#[test]
+fn two_rollbacks_one_record_per_lineage_container_judged_as_a_product_image_reddens_only_the_full_container_invariant(
+) {
+    let pool = pool_rolled_back_twice_under_the_lineage_knob(
+        "known-bad-lineage-container-closed-early",
+        LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
+    );
+    assert_eq!(
+        lineage_record_counts_of_each_container(&pool),
+        [1, 1],
+        "谱系树两片容器各 1 条"
+    );
+    let image = pool.memory_pool();
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE],
+        "产品入口：不是最右的谱系容器只装 1 条：只红 I-9.36：{verdicts:?}"
+    );
+    let verdicts_given_the_knob = check_pool_image_given_the_lineage_knob(
+        &image,
+        LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
+    );
+    assert_eq!(
+        violated_invariants(&verdicts_given_the_knob),
+        Vec::<&str>::new(),
+        "带旋钮值的入口：旋钮下的合法镜像一条都不红：{verdicts_given_the_knob:?}"
+    );
+    assert_eq!(
+        verdict_among(
+            &verdicts_given_the_knob,
+            LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE
+        ),
+        InvariantVerdict::Holds,
+        "I-9.37 在谱系树上真被评估过且成立"
+    );
+}
+
+/// I-9.37（测试镜像里谱系与退役容器不超旋钮值） 只红自己：谱系容器一片 2 条写出的谱系树（一片 2 条），告诉带旋钮值的入口旋钮是 1：
+/// 那片装了 2 条、多于旋钮值，只红 I-9.37（照 I-9.22 那一条的坏镜像）。产品入口上只一片容器，I-9.36 没有对象，一条都不红。
+#[test]
+fn a_lineage_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant()
+{
+    let pool = pool_rolled_back_twice_under_the_lineage_knob(
+        "known-bad-lineage-container-past-the-knob",
+        2,
+    );
+    assert_eq!(
+        lineage_record_counts_of_each_container(&pool),
+        [2],
+        "谱系树一片容器 2 条"
+    );
+    let image = pool.memory_pool();
+    assert_eq!(
+        violated_invariants(&check_pool_image(&image)),
+        Vec::<&str>::new(),
+        "产品入口：只一片谱系容器，一条都不红"
+    );
+    let verdicts = check_pool_image_given_the_lineage_knob(
+        &image,
+        LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
+    );
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE],
+        "谱系容器装了 2 条、旋钮值 1：只红 I-9.37：{verdicts:?}"
+    );
+}
+
 /// 合法的回退不红（判决第三节第 5 条「回退照抄现行那一版的退役树」、第 8 条 I-9.33 只比谱系上的）：删到底（退役一条 (叶, F)）→
 /// 回退到意图那一版（那片叶又活了，退役树照抄、仍是那一条）→ 再删到底（同一片叶再死一次，追加 (叶, F_again)）。
 /// 两条同身份，但 F 不在 F_again 那一版的父链上；每一版 checker 都一条不红，退役树那五条都成立。
diff --git a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
index 3ee99ac..637785e 100644
--- a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
+++ b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
@@ -3,6 +3,7 @@
 //! 只供测试的旋钮把容器与节点压小（一片 1 条、一个节点 2 条），旋钮不落盘；重建读层级 > 1 的根。池是两块 4 GiB 盘。
 //! 池级 checker 从镜像外拿墓碑容器的旋钮值（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T5 取 K1）：
 //! 这些镜像走带旋钮值的入口，I-9.22（测试镜像里墓碑容器不超旋钮值） 按旋钮值判；同一张镜像交给旧入口（当没装旋钮）I-9.21 红。
+//! 谱系容器同样压到一片 1 条，带旋钮值的入口一并给谱系的旋钮值（I-9.37，第 53 批；不给的话回退两次写出的两片谱系容器按产品容量判 I-9.36 红）。
 
 mod common;
 mod common_deletion_in_progress;
@@ -17,8 +18,9 @@ use singlefs_checker::image::{
     TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
 };
 use singlefs_checker::walk::{
-    check_pool_image, check_pool_image_given_the_tombstone_container_capacity_knob,
-    TombstoneContainerCapacityKnob,
+    check_pool_image, check_pool_image_given_the_container_capacity_knobs,
+    check_pool_image_given_the_tombstone_container_capacity_knob, ContainerCapacityKnobs,
+    LineageContainerCapacityKnob, TombstoneContainerCapacityKnob,
 };
 use singlefs_core::address::InodeNumber;
 use singlefs_core::admission::{
@@ -49,6 +51,8 @@ use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchSize};
 
 /// 写这些镜像时墓碑容器旋钮压到的每片记录数（`capacities_of_one_record_and_two_entries` 那一个），交给池级 checker 带旋钮值的入口。
 const TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;
+/// 写这些镜像时谱系容器旋钮压到的每片记录数（`capacities_of_one_record_and_two_entries` 那一个），同样交给带旋钮值的入口。
+const LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;
 
 /// 一片容器 1 条、一个码 2 节点 2 条：第 3 片容器根满长一层，第 5 片长到层级 3。
 fn capacities_of_one_record_and_two_entries() -> ContainerTreeCapacities {
@@ -57,7 +61,10 @@ fn capacities_of_one_record_and_two_entries() -> ContainerTreeCapacities {
             TombstoneRecordsPerContainerCappedForTests::new(1).expect("1..=815"),
         ),
         lineage_container: LineageContainerCapacity::CappedForTests(
-            LineageRecordsPerContainerCappedForTests::new(1).expect("1..=1359"),
+            LineageRecordsPerContainerCappedForTests::new(
+                LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
+            )
+            .expect("1..=1359"),
         ),
         retirement_container:
             singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
@@ -124,11 +131,19 @@ fn tombstone_container_capacity_knob(records_per_container: u64) -> TombstoneCon
     TombstoneContainerCapacityKnob::new(records_per_container).expect("旋钮值在 1..=815 里")
 }
 
-/// 池级 checker 带旋钮值的入口判出来的违例：`records_per_container` 是写这张镜像时墓碑容器旋钮压到的每片记录数。
+/// 池级 checker 带旋钮值的入口判出来的违例：`records_per_container` 是写这张镜像时墓碑容器旋钮压到的每片记录数；谱系的旋钮值一并给
+/// （[`LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB`]），退役容器照产品。
 fn violations_given_the_knob(pool: &BuiltPool, records_per_container: u64) -> Vec<String> {
-    check_pool_image_given_the_tombstone_container_capacity_knob(
+    check_pool_image_given_the_container_capacity_knobs(
         &pool.memory_pool(),
-        Some(tombstone_container_capacity_knob(records_per_container)),
+        ContainerCapacityKnobs {
+            tombstone: Some(tombstone_container_capacity_knob(records_per_container)),
+            lineage: Some(
+                LineageContainerCapacityKnob::new(LINEAGE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB)
+                    .expect("旋钮值在 1..=1359 里"),
+            ),
+            retirement: None,
+        },
     )
     .into_iter()
     .filter_map(|(invariant, verdict)| match verdict {
diff --git a/crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs b/crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs
new file mode 100644
index 0000000..f3e9882
--- /dev/null
+++ b/crates/singlefs-harness/tests/inode_leaf_merge_and_borrow_against_an_independent_reference.rs
@@ -0,0 +1,380 @@
+//! inode 树叶容器的合并与借在发布路径上逐次对一份独立参照（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F1）：
+//! 参照按条款另写一份，不引 `singlefs_core::inode_tree` 的合并、借、摘片代码，只从发布出来的那一版读记录与身份——
+//! D8（核心索引结构） 已定项 6：插入落在最右的叶、满 233 条就在末尾分裂，右半容器号 = 那条新记录的 inode 号、出生代 = 这次发布的 txg；
+//! 合并时机照 E176（inode 树叶合并三候选的容量与写代价） M3 那一臂（`scripts/experiments/e176_inode_leaf_merge_capacity_and_rewrites.rs`
+//! 的 `remove`）：删空的整片摘掉、身份消失，它的新左右邻从左邻起并；没删空就从它起并——先查它与右邻、再查左邻与它，两片之和 ≤ 233 就左吸收右、
+//! 被吸收的那一片身份消失，并完从幸存那片接着查；从它起一次都没并、它少于 ⌈233 ÷ 2⌉ 条、它有右邻时从右邻借最小的 ⌊(右 − 左) ÷ 2⌋ 条，
+//! 两片都保留身份，再从借出的那一片起并。
+//! 历史（种子固定）：随机切几批建 583–599 个空 inode（叶 [233, 233, ≥ 117]），再一次次 unlink「可删条数最多的那片叶」（并列取最左）里随机的一个号，
+//! 直到参照见到一次「删完之后与左邻、右邻两对都装得下」——M3 的方向只有这一形分得出来（攻方第二次抽样 K4-4：三片各 117 条删中间一条，
+//! 两种写法退役不同的叶、池级 checker 都不红）；再随机 unlink 几个活着的号。每次用户改动之后逐片比记录号与身份，逐次比这次退役的身份集与死亡 txg。
+
+mod common;
+mod common_deletion_in_progress;
+mod common_pointer_head_corruption;
+
+use common::{build_pool, BuiltPool};
+use common_deletion_in_progress::{publish_new_inodes_and_advance_the_pool, unlink_to_the_end};
+use singlefs_core::transaction::TransactionOutput;
+use singlefs_format::INODE_LEAF_RECORDS;
+use singlefs_harness::history::SeededRandomSource;
+
+/// 第一个文件那条 inode 记录（号 1）：历史里不 unlink 它，它在最左那片叶里。
+const FIRST_FILE_INODE: u64 = 1;
+/// inode 叶容器的打包记录类型（D8（核心索引结构） 已定项 6：类型 2）。
+const PACKED_RECORD_TYPE_INODE: u16 = 2;
+/// 这段历史的种子。
+const SEED_OF_THE_HISTORY: u64 = 53;
+/// 建 inode 那几批合起来至少建这么多个：叶装成 [233, 233, 117]，删到三片各 117 条的那一刻正是「两对都装得下」。
+const NEW_INODES_AT_LEAST: u64 = 582;
+/// 再往上随机多建 0..这么多个（叶的第三片 117..=133 条，仍删得到「两对都装得下」那一形）。
+const NEW_INODES_ABOVE_THE_LEAST_BELOW: u64 = 17;
+/// 一批至多建这么多个。
+const NEW_INODES_PER_BATCH_AT_MOST: u64 = 260;
+/// 删的那一段至多这么多次 unlink：三片从 [233, 233, 133] 删到各 117 条至多 250 次（参照的推算），见到那一形之后另删几次。
+const UNLINKS_UNTIL_BOTH_PAIRS_FIT_AT_MOST: u64 = 260;
+/// 见到「两对都装得下」之后再随机 unlink 几个活着的号。
+const RANDOM_UNLINKS_AFTER_BOTH_PAIRS_FIT: u64 = 6;
+
+/// 一片叶容器的身份四段（出生树、打包记录类型、容器号、容器出生代）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+struct ReferenceLeafIdentity {
+    birth_tree: u64,
+    record_type: u16,
+    container: u64,
+    container_birth: u64,
+}
+
+/// 参照里的一片叶：身份与它装的 inode 号（升序）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct ReferenceLeaf {
+    identity: ReferenceLeafIdentity,
+    inode_numbers: Vec<u64>,
+}
+
+/// 参照：按叶序的叶，加上这次改动里身份消失的那几片（按消失的先后）与「两对都装得下」见过几次。
+struct ReferenceLeafShape {
+    inode_tree: u64,
+    leaves: Vec<ReferenceLeaf>,
+    retired_in_this_change: Vec<ReferenceLeafIdentity>,
+    times_both_neighbour_pairs_fit: u64,
+}
+
+fn leaf_capacity() -> usize {
+    usize::try_from(INODE_LEAF_RECORDS).expect("233")
+}
+
+/// 均分借的下限 ⌈233 ÷ 2⌉（E176 M3 主取样点）。
+fn borrow_floor() -> usize {
+    leaf_capacity().div_ceil(2)
+}
+
+impl ReferenceLeafShape {
+    /// 从发布出来的那一版读起：每片叶的身份与记录号。
+    fn of_the_version(version: &TransactionOutput) -> Self {
+        let leaves = leaves_of_the_version(version);
+        let inode_tree = leaves
+            .first()
+            .expect("第一个文件之后 inode 树至少一片叶")
+            .identity
+            .birth_tree;
+        Self {
+            inode_tree,
+            leaves,
+            retired_in_this_change: Vec::new(),
+            times_both_neighbour_pairs_fit: 0,
+        }
+    }
+
+    /// 一条新号追加到最右那片；最右那片满 233 条（或树是空的）就新开一片：容器号 = 这个号、出生代 = 这次发布的 txg。
+    fn append(&mut self, inode: u64, txg_of_the_publish: u64) {
+        let rightmost_is_full = self
+            .leaves
+            .last()
+            .is_none_or(|leaf| leaf.inode_numbers.len() == leaf_capacity());
+        if rightmost_is_full {
+            self.leaves.push(ReferenceLeaf {
+                identity: ReferenceLeafIdentity {
+                    birth_tree: self.inode_tree,
+                    record_type: PACKED_RECORD_TYPE_INODE,
+                    container: inode,
+                    container_birth: txg_of_the_publish,
+                },
+                inode_numbers: Vec::new(),
+            });
+        }
+        self.leaves
+            .last_mut()
+            .expect("上面保证了至少一片")
+            .inode_numbers
+            .push(inode);
+    }
+
+    fn records_at(&self, position: usize) -> usize {
+        self.leaves[position].inode_numbers.len()
+    }
+
+    fn drop_leaf(&mut self, position: usize) -> ReferenceLeaf {
+        let dropped = self.leaves.remove(position);
+        self.retired_in_this_change.push(dropped.identity);
+        dropped
+    }
+
+    fn left_absorbs_its_right_neighbour(&mut self, left_position: usize) {
+        let absorbed = self.drop_leaf(left_position + 1);
+        self.leaves[left_position]
+            .inode_numbers
+            .extend(absorbed.inode_numbers);
+    }
+
+    /// 从 `start_position` 那片起：先查它与右邻、再查左邻与它，和 ≤ 233 就左吸收右；并完从幸存那片接着查。交回并没并过。
+    /// 迭代上界是片数：每一轮并掉一片或退出。
+    fn merge_from(&mut self, start_position: usize) -> bool {
+        let mut position = start_position;
+        let mut merged = false;
+        loop {
+            let right_pair_fits = position + 1 < self.leaves.len()
+                && self.records_at(position) + self.records_at(position + 1) <= leaf_capacity();
+            let left_pair_fits = position > 0
+                && self.records_at(position - 1) + self.records_at(position) <= leaf_capacity();
+            if right_pair_fits && left_pair_fits {
+                self.times_both_neighbour_pairs_fit += 1;
+            }
+            if right_pair_fits {
+                self.left_absorbs_its_right_neighbour(position);
+            } else if left_pair_fits {
+                self.left_absorbs_its_right_neighbour(position - 1);
+                position -= 1;
+            } else {
+                return merged;
+            }
+            merged = true;
+        }
+    }
+
+    /// 均分借：`position` 那片少于下限、有右邻时，右邻最小的 ⌊(右 − 左) ÷ 2⌋ 条挪到它末尾，再从右邻起并。
+    fn borrow_from_the_right_if_below_the_floor(&mut self, position: usize) {
+        if position + 1 >= self.leaves.len() || self.records_at(position) >= borrow_floor() {
+            return;
+        }
+        let moved_count = (self.records_at(position + 1) - self.records_at(position)) / 2;
+        let moved: Vec<u64> = self.leaves[position + 1]
+            .inode_numbers
+            .drain(..moved_count)
+            .collect();
+        self.leaves[position].inode_numbers.extend(moved);
+        self.merge_from(position + 1);
+    }
+
+    /// unlink 最后一批删掉 `inode` 那条记录。
+    fn remove(&mut self, inode: u64) {
+        let position = self
+            .leaves
+            .iter()
+            .position(|leaf| leaf.inode_numbers.contains(&inode))
+            .expect("删的号在参照里");
+        self.leaves[position]
+            .inode_numbers
+            .retain(|kept| *kept != inode);
+        if self.leaves[position].inode_numbers.is_empty() {
+            self.drop_leaf(position);
+            if position > 0 && position < self.leaves.len() {
+                self.merge_from(position - 1);
+            }
+        } else if !self.merge_from(position) {
+            self.borrow_from_the_right_if_below_the_floor(position);
+        }
+    }
+
+    /// 可删条数（不算第一个文件那条）最多的那片叶的位置，并列取最左。
+    fn position_of_the_leaf_with_the_most_removable_records(&self) -> usize {
+        let removable = |leaf: &ReferenceLeaf| {
+            leaf.inode_numbers
+                .iter()
+                .filter(|inode| **inode != FIRST_FILE_INODE)
+                .count()
+        };
+        let most = self.leaves.iter().map(removable).max().expect("至少一片");
+        self.leaves
+            .iter()
+            .position(|leaf| removable(leaf) == most)
+            .expect("取到的最大值来自其中一片")
+    }
+
+    fn live_removable_inodes(&self) -> Vec<u64> {
+        self.leaves
+            .iter()
+            .flat_map(|leaf| leaf.inode_numbers.iter().copied())
+            .filter(|inode| *inode != FIRST_FILE_INODE)
+            .collect()
+    }
+}
+
+fn identity_of(identity: &singlefs_core::unit::PackedIdentity) -> ReferenceLeafIdentity {
+    ReferenceLeafIdentity {
+        birth_tree: identity.birth_tree.0,
+        record_type: identity.record_type,
+        container: identity.container,
+        container_birth: identity.container_birth.0,
+    }
+}
+
+/// 发布出来的那一版里按叶序的每片叶：身份与记录号。
+fn leaves_of_the_version(version: &TransactionOutput) -> Vec<ReferenceLeaf> {
+    version
+        .inode_leaf_containers
+        .iter()
+        .map(|container| ReferenceLeaf {
+            identity: identity_of(&container.contents.identity),
+            inode_numbers: container
+                .contents
+                .records
+                .iter()
+                .map(|record| record.inode)
+                .collect(),
+        })
+        .collect()
+}
+
+/// 这一版退役树里按叶序的全部退役记录：(被退役的身份, 死亡 txg)。
+fn retirement_records_of(version: &TransactionOutput) -> Vec<(ReferenceLeafIdentity, u64)> {
+    version
+        .retirement_containers
+        .iter()
+        .flat_map(|container| container.contents.records.iter())
+        .map(|record| (identity_of(&record.retired), record.death_txg.0))
+        .collect()
+}
+
+/// 一次用户改动之后：发布路径那一版的每片叶（身份、记录号）与参照逐片相同；这次改动追加的退役记录（退役树只追加）与参照这次身份消失的那几片
+/// 逐个相同（身份按集合比、条数也比），死亡 txg 都是这一版的 txg。
+fn assert_the_version_matches_the_reference(
+    what_changed: &str,
+    retirement_records_before: &[(ReferenceLeafIdentity, u64)],
+    version: &TransactionOutput,
+    reference: &mut ReferenceLeafShape,
+) {
+    let leaves = leaves_of_the_version(version);
+    assert_eq!(
+        leaves
+            .iter()
+            .map(|leaf| leaf.inode_numbers.len())
+            .collect::<Vec<_>>(),
+        reference
+            .leaves
+            .iter()
+            .map(|leaf| leaf.inode_numbers.len())
+            .collect::<Vec<_>>(),
+        "{what_changed}：每片叶的记录数与参照不同"
+    );
+    assert_eq!(
+        leaves, reference.leaves,
+        "{what_changed}：叶的身份或记录号与参照不同"
+    );
+    let retirement_records_after = retirement_records_of(version);
+    assert!(
+        retirement_records_after.starts_with(retirement_records_before),
+        "{what_changed}：退役树只追加"
+    );
+    let appended = &retirement_records_after[retirement_records_before.len()..];
+    let mut retired_by_the_publish_path: Vec<ReferenceLeafIdentity> =
+        appended.iter().map(|(identity, _)| *identity).collect();
+    let mut retired_by_the_reference = std::mem::take(&mut reference.retired_in_this_change);
+    retired_by_the_publish_path.sort_unstable();
+    retired_by_the_reference.sort_unstable();
+    assert_eq!(
+        retired_by_the_publish_path, retired_by_the_reference,
+        "{what_changed}：这次退役的身份集与参照不同"
+    );
+    let txg = version.root.checkpoint_txg.0;
+    assert!(
+        appended.iter().all(|(_, death_txg)| *death_txg == txg),
+        "{what_changed}：这次追加的退役记录死亡 txg 都是这一版的 txg {txg}：{appended:?}"
+    );
+}
+
+/// 建一批 `count` 个空 inode（号从水位起），参照照条款追加，再比。
+fn create_and_compare(pool: &mut BuiltPool, count: u64, reference: &mut ReferenceLeafShape) {
+    let retirement_records_before = retirement_records_of(&pool.output);
+    let first_new_inode = pool.output.inode_number_watermark();
+    let version = publish_new_inodes_and_advance_the_pool(pool, count);
+    let txg = version.root.checkpoint_txg.0;
+    // 迭代上界：这一批的条数。
+    for inode in first_new_inode..first_new_inode + count {
+        reference.append(inode, txg);
+    }
+    assert_the_version_matches_the_reference(
+        &format!("建 {count} 个（{first_new_inode} 起，txg {txg}）"),
+        &retirement_records_before,
+        &version,
+        reference,
+    );
+}
+
+/// unlink `inode` 到底，参照在最后一批删掉那条记录，再比最后一批那一版。
+fn unlink_and_compare(pool: &mut BuiltPool, inode: u64, reference: &mut ReferenceLeafShape) {
+    let retirement_records_before = retirement_records_of(&pool.output);
+    let counts_before: Vec<usize> = reference
+        .leaves
+        .iter()
+        .map(|leaf| leaf.inode_numbers.len())
+        .collect();
+    unlink_to_the_end(pool, inode);
+    reference.remove(inode);
+    assert_the_version_matches_the_reference(
+        &format!(
+            "unlink {inode}（删之前各片 {counts_before:?}，最后一批 txg {}）",
+            pool.output.root.checkpoint_txg.0
+        ),
+        &retirement_records_before,
+        &pool.output,
+        reference,
+    );
+}
+
+#[test]
+fn every_publish_of_a_random_history_leaves_the_leaf_shape_and_the_retired_leaves_of_the_independent_reference(
+) {
+    let mut pool = build_pool("inode-leaf-merge-and-borrow-against-an-independent-reference");
+    let mut random = SeededRandomSource::from_seed(SEED_OF_THE_HISTORY);
+    let mut reference = ReferenceLeafShape::of_the_version(&pool.output);
+    let mut left_to_create = NEW_INODES_AT_LEAST + random.below(NEW_INODES_ABOVE_THE_LEAST_BELOW);
+    // 迭代上界：每一批至少建 1 个，left_to_create 至多 598。
+    while left_to_create > 0 {
+        let count = (1 + random.below(NEW_INODES_PER_BATCH_AT_MOST)).min(left_to_create);
+        create_and_compare(&mut pool, count, &mut reference);
+        left_to_create -= count;
+    }
+    let mut unlinks = 0;
+    // 迭代上界：UNLINKS_UNTIL_BOTH_PAIRS_FIT_AT_MOST 次；跨轮携带的是参照与池，每轮各删一个号。
+    while reference.times_both_neighbour_pairs_fit == 0 {
+        assert!(
+            unlinks < UNLINKS_UNTIL_BOTH_PAIRS_FIT_AT_MOST,
+            "删了 {unlinks} 次还没走到「两对都装得下」那一形：各片 {:?}",
+            reference
+                .leaves
+                .iter()
+                .map(|leaf| leaf.inode_numbers.len())
+                .collect::<Vec<_>>()
+        );
+        let position = reference.position_of_the_leaf_with_the_most_removable_records();
+        let removable: Vec<u64> = reference.leaves[position]
+            .inode_numbers
+            .iter()
+            .copied()
+            .filter(|inode| *inode != FIRST_FILE_INODE)
+            .collect();
+        let pick =
+            usize::try_from(random.below(u64::try_from(removable.len()).expect("一片至多 233 条")))
+                .expect("小于 233");
+        unlink_and_compare(&mut pool, removable[pick], &mut reference);
+        unlinks += 1;
+    }
+    // 迭代上界：常量 RANDOM_UNLINKS_AFTER_BOTH_PAIRS_FIT。
+    for _ in 0..RANDOM_UNLINKS_AFTER_BOTH_PAIRS_FIT {
+        let live = reference.live_removable_inodes();
+        let pick = usize::try_from(random.below(u64::try_from(live.len()).expect("装得进 u64")))
+            .expect("小于活着的号数");
+        unlink_and_compare(&mut pool, live[pick], &mut reference);
+    }
+}
diff --git a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
index bd68afb..1cc1bcf 100644
--- a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
+++ b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
@@ -582,8 +582,9 @@ fn region_device(txg: u64) -> DeviceIdentity {
 /// 下面两张有文件的表照样有这七条），I-9.24（删除队列至多一条 key） 要 `deleted_inodes` 条目、I-9.29（同号只许一个出生代） 要 inode 记录
 /// （第一个文件版本之后两条都真被评估过且成立）。I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上） 要最新根下面的记账树
 /// （第一个文件版本之后真被评估过且成立：那一版自己写的那一行就是第一级台阶）。退役树那五条（I-9.31–I-9.35，第 50 批）同样没有对象：
-/// 每一版树表都是 0 条，没有退役树、没有 inode 树。
-const NOT_APPLICABLE_WITHOUT_FILE: [&str; 51] = [
+/// 每一版树表都是 0 条，没有退役树、没有 inode 树。谱系树与退役树的容器容量那两条（I-9.36、I-9.37，第 53 批）同理没有对象
+/// （有了文件版本两棵也是零根，下面两张有文件的表照样有它们；旧入口没有谱系与退役的旋钮值，I-9.37 一格都不判）。
+const NOT_APPLICABLE_WITHOUT_FILE: [&str; 53] = [
     "I-9.27",
     "I-5.5",
     "I-1.10",
@@ -635,6 +636,8 @@ const NOT_APPLICABLE_WITHOUT_FILE: [&str; 51] = [
     "I-9.33",
     "I-9.34",
     "I-9.35",
+    "I-9.36",
+    "I-9.37",
 ];
 
 /// mkfs 刚写完那一份比 `NOT_APPLICABLE_WITHOUT_FILE` 多报四条不适用：journal 环整段是 0，一条自证过的记录都没有 ⇒ I-8.6（反向链算法）、
@@ -643,7 +646,7 @@ const NOT_APPLICABLE_WITHOUT_FILE: [&str; 51] = [
 /// I-3.5（引用区间的精确性） 两张表都有，这一格的理由是回退候选集只剩 mkfs 的第 0 代根一版（环里没有记录、根环里只有它），
 /// 凑不出相邻的一对。
 /// 删除线那五条与墓碑与谱系那九条的理由同 `NOT_APPLICABLE_WITHOUT_FILE`。
-const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 55] = [
+const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 57] = [
     "I-9.27",
     "I-5.5",
     "I-1.10",
@@ -699,6 +702,8 @@ const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 55] = [
     "I-9.33",
     "I-9.34",
     "I-9.35",
+    "I-9.36",
+    "I-9.37",
 ];
 
 /// 写完第一个文件版本之后仍报不适用的那六条（I-4.1 的理由见 `NOT_APPLICABLE_WITHOUT_FILE`）：
@@ -715,18 +720,18 @@ const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 55] = [
 /// （阳性对照与坏镜像在 `pool_checker_judges_the_deletion_state_of_every_walked_version.rs`）；I-4.4（意图只描述目标态） 第一版恒报不适用；
 /// I-9.17–I-9.23——没有删除、没有回退，墓碑树与谱系树都是零根；I-9.31–I-9.34——没有叶容器身份消失，退役树零根
 /// （I-9.35 在第一个文件版本那一版上真被评估过且成立：父版树表 0 条、叶身份集是空集）。
-const NOT_APPLICABLE_WITH_ONE_FILE_VERSION: [&str; 20] = [
+const NOT_APPLICABLE_WITH_ONE_FILE_VERSION: [&str; 22] = [
     "I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-4.1", "I-9.28", "I-4.7", "I-4.6", "I-4.4", "I-9.17",
     "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23", "I-9.31", "I-9.32", "I-9.33",
-    "I-9.34",
+    "I-9.34", "I-9.36", "I-9.37",
 ];
 
 /// 只有实例 1 的池上第一个文件版本之后：比 `NOT_APPLICABLE_WITH_ONE_FILE_VERSION` 多 I-3.12（实例表行 T 低于引用它的根的 txg）——
 /// 实例表一行都没有（写过行的那一份上它真被评估过且成立）。
-const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 21] = [
+const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 23] = [
     "I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-3.12", "I-4.1", "I-9.28", "I-4.7", "I-4.6", "I-4.4",
     "I-9.17", "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23", "I-9.31", "I-9.32",
-    "I-9.33", "I-9.34",
+    "I-9.33", "I-9.34", "I-9.36", "I-9.37",
 ];
 
 /// 树表 0 条的一版上写行那次发布之后：比 `NOT_APPLICABLE_WITHOUT_FILE` 少七条——I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）、
@@ -742,7 +747,7 @@ const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 21] = [
 /// I-3.5（引用区间的精确性） 也不在这张表里：写行那次发布（实例 2、txg 3）把实例表 COW 到新落点、写了自己那棵分配记录树，
 /// 这几个单元前一版（实例 1 的暖机根，txg 2）不引用、诞生代号 3 大于 2，出现规则真判到了它们且成立。
 /// 删除线那五条与墓碑与谱系那九条的理由同 `NOT_APPLICABLE_WITHOUT_FILE`。
-const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 44] = [
+const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 46] = [
     "I-9.27",
     "I-5.5",
     "I-3.1",
@@ -787,6 +792,8 @@ const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 44] = [
     "I-9.33",
     "I-9.34",
     "I-9.35",
+    "I-9.36",
+    "I-9.37",
 ];
 
 /// 这一步的镜像上 checker 的每一条判决逐条核：`not_applicable` 里的报不适用，其余每一条都真被评估过且成立（一条违例都没有）。
`````

### mutations-append.tsv

`````text
实审 A3d：重建读记账树不判节点头（记账根只剩一条：可写挂载改红在记账条目宽上，不是判全那一道的叶区间）	crates/singlefs-core/src/recovery.rs	        &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),\n        CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,	        &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),\n        CodeTwoTreeHeaderJudgement::OnlyWhatTheShapeNeeds,	-p singlefs-checker-tier --test bad_disk_input_campaign -- an_accounting_root_narrowed_to_its_first_entry	an_accounting_root_narrowed_to_its_first_entry_is_refused_at_the_header_judgement_by_both_readers
第53批 F1 M3 先查左邻：三片各 117 条删中间一条时左邻先吸收中间那片（攻方 M4，core 单测）	crates/singlefs-core/src/inode_tree.rs	            if right_pair_fits {\n                self.absorb_the_right_neighbour(current_position);\n            } else if left_pair_fits {\n                self.absorb_the_right_neighbour(current_position - 1);\n                current_position -= 1;\n            } else {	            if left_pair_fits {\n                self.absorb_the_right_neighbour(current_position - 1);\n                current_position -= 1;\n            } else if right_pair_fits {\n                self.absorb_the_right_neighbour(current_position);\n            } else {	-p singlefs-core --lib -- inode_tree::tests::removing_from_the_middle_of_three_half_full_containers_absorbs_the_right_neighbour_first	removing_from_the_middle_of_three_half_full_containers_absorbs_the_right_neighbour_first
第53批 F1 M3 先查左邻：随机历史上发布路径与按条款另写的叶形状参照逐次比（攻方 M4）	crates/singlefs-core/src/inode_tree.rs	            if right_pair_fits {\n                self.absorb_the_right_neighbour(current_position);\n            } else if left_pair_fits {\n                self.absorb_the_right_neighbour(current_position - 1);\n                current_position -= 1;\n            } else {	            if left_pair_fits {\n                self.absorb_the_right_neighbour(current_position - 1);\n                current_position -= 1;\n            } else if right_pair_fits {\n                self.absorb_the_right_neighbour(current_position);\n            } else {	-p singlefs-harness --test inode_leaf_merge_and_borrow_against_an_independent_reference -- every_publish_of_a_random_history_leaves_the_leaf_shape_and_the_retired_leaves_of_the_independent_reference	every_publish_of_a_random_history_leaves_the_leaf_shape_and_the_retired_leaves_of_the_independent_reference
第53批 F3 Opt2：回退取 inode 号水位时环那一项一条根都不交（攻方 M5，只剩 cur 内存里的）	crates/singlefs-core/src/mount.rs	    highest_inode_number_watermark_of(&ring_roots_not_abandoned_by_crash_recovery(\n        ring_roots,\n        current_table,\n    ))	    highest_inode_number_watermark_of(&ring_roots_not_abandoned_by_crash_recovery(\n        ring_roots,\n        current_table,\n    )[..0])	-p singlefs-core --lib -- inode_number_watermark_of_the_rollback_tests::a_ring_root_not_abandoned_whose_watermark_is_above_the_current_version_sets_the_watermark	a_ring_root_not_abandoned_whose_watermark_is_above_the_current_version_sets_the_watermark
第53批 F3 Opt2：回退取 inode 号水位时被崩溃恢复抛弃的根也算（构造态）	crates/singlefs-core/src/mount.rs	    highest_inode_number_watermark_of(&ring_roots_not_abandoned_by_crash_recovery(\n        ring_roots,\n        current_table,\n    ))	    highest_inode_number_watermark_of(ring_roots)	-p singlefs-core --lib -- inode_number_watermark_of_the_rollback_tests::a_ring_root_abandoned_by_the_instance_table_does_not_set_the_watermark	a_ring_root_abandoned_by_the_instance_table_does_not_set_the_watermark
第53批 F3 Opt2：回退取 inode 号水位只取环、不与 cur 内存里的取 max（构造态）	crates/singlefs-core/src/mount.rs	    .map_or(inode_number_watermark_of_the_current_version, |ring| {\n        ring.max(inode_number_watermark_of_the_current_version)\n    })	    .map_or(inode_number_watermark_of_the_current_version, |ring| ring)	-p singlefs-core --lib -- inode_number_watermark_of_the_rollback_tests::a_ring_root_whose_watermark_is_below_the_current_version_leaves_the_current_watermark	a_ring_root_whose_watermark_is_below_the_current_version_leaves_the_current_watermark
第53批 F4 I-1.7：打包容器类型 3（退役）不比记录宽（正推 C-retwidth）	crates/singlefs-checker/src/walk.rs	            && registered_record_width(record_type) == Some(width)\n            && count * width <= declared	            && (record_type == PACKED_TYPE_RETIREMENT || registered_record_width(record_type) == Some(width))\n            && count * width <= declared	-p singlefs-harness --test checker_known_bad_images -- a_retirement_container_whose_record_width_is_not_thirty_four_reddens_only_the_packed_container_invariant	a_retirement_container_whose_record_width_is_not_thirty_four_reddens_only_the_packed_container_invariant
第53批 F4 I-9.32：不判容器出生代 < 死亡 txg（攻方第二次抽样 checker-i932-no-birth-before-death）	crates/singlefs-checker/src/walk.rs	                && retired.container_birth < record.death_txg\n                && record.death_txg <= version.checkpoint_txg;	                && record.death_txg <= version.checkpoint_txg;	-p singlefs-harness --test checker_known_bad_images -- a_retirement_record_whose_container_birth_is_not_below_its_death_txg_reddens_only_the_well_formed_retirement_record_invariant	a_retirement_record_whose_container_birth_is_not_below_its_death_txg_reddens_only_the_well_formed_retirement_record_invariant
第53批 F4 I-9.32：不判死亡 txg ≤ 这一版的 txg（攻方第二次抽样 checker-i932-no-death-at-most-version）	crates/singlefs-checker/src/walk.rs	                && retired.container_birth < record.death_txg\n                && record.death_txg <= version.checkpoint_txg;	                && retired.container_birth < record.death_txg;	-p singlefs-harness --test checker_known_bad_images -- a_retirement_record_dying_after_the_newest_version_reddens_only_the_well_formed_retirement_record_invariant	a_retirement_record_dying_after_the_newest_version_reddens_only_the_well_formed_retirement_record_invariant
第53批 F2 I-9.36：非最右的退役容器只判不超产品容量、不判装满（退役树那一形）	crates/singlefs-checker/src/walk.rs	record_count == product_records	record_count <= product_records	-p singlefs-harness --test checker_known_bad_images -- two_leaves_retired_one_record_per_retirement_container_judged_as_a_product_image_reddens_only_the_full_container_invariant	two_leaves_retired_one_record_per_retirement_container_judged_as_a_product_image_reddens_only_the_full_container_invariant
第53批 F2 I-9.36：非最右的谱系容器只判不超产品容量、不判装满（谱系树那一形）	crates/singlefs-checker/src/walk.rs	record_count == product_records	record_count <= product_records	-p singlefs-harness --test checker_known_bad_images -- two_rollbacks_one_record_per_lineage_container_judged_as_a_product_image_reddens_only_the_full_container_invariant	two_rollbacks_one_record_per_lineage_container_judged_as_a_product_image_reddens_only_the_full_container_invariant
第53批 F2 I-9.37：谱系与退役容器按旋钮值加一判	crates/singlefs-checker/src/walk.rs	record_count <= records_the_knob_allows,	record_count <= records_the_knob_allows + 1,	-p singlefs-harness --test checker_known_bad_images -- a_lineage_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant	a_lineage_container_holding_more_records_than_the_knob_reddens_only_the_knob_capacity_invariant
第53批 F2 GPU 判器 I-9.36：非最右的退役容器只判不超产品容量、不判装满	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (d_container_record_count(container_index) != product_records) {	        if (d_container_record_count(container_index) > product_records) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_retirement_container_closes_before_it_is_full	the_gpu_judge_agrees_with_the_cpu_when_a_retirement_container_closes_before_it_is_full
第53批 F2 GPU 判器 I-9.36：谱系树读完容器不判容量	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        return false;\n    }\n    d_judge_lineage_or_retirement_container_capacity(DELETION_LINEAGE_CONTAINER_RECORDS);	        return false;\n    }	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_lineage_container_closes_before_it_is_full	the_gpu_judge_agrees_with_the_cpu_when_a_lineage_container_closes_before_it_is_full
`````

### mutations-replacements.tsv

`````text
实三（D23 已定项 14「水位」）：回退那次的 inode 号水位只取环里读得出的根，不与内存里现行那一版的取 max	crates/singlefs-core/src/mount.rs	    .map_or(inode_number_watermark_of_the_current_version, |ring| {\n        ring.max(inode_number_watermark_of_the_current_version)\n    })	    .map_or(inode_number_watermark_of_the_current_version, |ring| ring)	-p singlefs-harness --test unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over -- the_rollback_takes_the_inode_number_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_its_root	the_rollback_takes_the_inode_number_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_its_root
第49批 W3 Y1 Opt2：管理员回退取 inode 号水位时环里读得出的根全都算（被崩溃恢复抛弃的也算，回退跳到被抛弃那一版的水位）	crates/singlefs-core/src/mount.rs	    highest_inode_number_watermark_of(&ring_roots_not_abandoned_by_crash_recovery(\n        ring_roots,\n        current_table,\n    ))	    highest_inode_number_watermark_of(ring_roots)	-p singlefs-harness --test rollback_by_a_forward_publish -- a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root	a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root
`````

### mutations-delete.txt

`````text
实审 A3d：重建读记账树不判节点头（记账根只剩一条：可写挂载改红在记账条目数上，不是判全那一道的叶区间）
`````

## m3-batch54/repin

### crates.patch

`````diff
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
index e075d97..d016efb 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
@@ -845,9 +845,12 @@ fn print_the_unsharded_enumerations_for_the_golden_comparison() {
 /// I-9.33=0/0/46 I-9.34=0/0/46 I-9.35=9/0/37` 六项、`LAYER0_RESUME` 里的计划哈希换回 `49878830…` 之后，整段 SHA-256 正好是上一版 `4b8b1e84…`，
 /// 别的字节不差。计划哈希 `49878830…` → `555baf1b…`：新池新建文件那次发布写下的字节变了（树表多一条退役树条目、根记录里树 ID 水位
 /// 65548 → 65549，`crash_points_tree_identifier_watermark_orphans.rs` 钉着），写表跟着变；单元写、段与状态数不变（仍 46 个状态）。
+/// 2026-10-09 第 53 批合入（加 I-9.36、I-9.37，已实现的不变量 90 → 92 条）之后重取：打的行取主 agent 合后副本那一趟 checker 档快档
+/// 断言 `left` 的 `a4643c92…`（行数仍 58）；把两趟 `GOLDEN_TALLY` 各删掉 ` I-9.36=0/0/46 I-9.37=0/0/46` 两项之后，整段 SHA-256 正好是
+/// 上一版 `4d42c7ca…`，别的字节不差（`LAYER0_RESUME` 的计划哈希仍是 `555baf1b…`）。
 const GOLDEN_PRINTED_LINE_COUNT: usize = 58;
 const GOLDEN_PRINTED_LINES_SHA256: &str =
-    "4d42c7ca5ca9e8e16e5c7bdde36198f576d40a98ad9a8dfb35fb33947161e354";
+    "a4643c92e6836ef465aba245ee096127bebc0b65f7342a090fd1570d9c48bea6";
 /// 同一趟留下的进度文件：名字（带计划哈希）与整份内容的 SHA-256（47 行：文件头 + 46 行片行）。名字取自上面那一趟日志的 `LAYER0_RESUME` 行；
 /// 打的行与内容的 SHA-256 取自 2026-10-05 主工作区（形式化线第 34 批派出时那一版，含删除线第 22 批）：与根环线在「10g + 32-1」那棵树上重取的 58 行逐行比，只差 GOLDEN_TALLY 两行多出 I-9.17–I-9.24、I-9.29 九条新不变量的计数（删除线第 22 批把已实现的不变量从 74 条加到 83 条），已有各条计数不变；那一次比上一版只差 `LAYER0_RESUME` 那一行的计划哈希（T_time 改毫秒带来）：
 /// 打的行是那一趟断言 `left` 的 `50615e47…`。2026-10-06 删除线第 24 批合入之后（新九条各写专门的不适用理由、I-9.17 / I-9.23 改读孩子的方式）在主工作区重取：打的行取那一趟断言 `left` 的 `4fc9cde7…`；
@@ -856,9 +859,12 @@ const GOLDEN_PRINTED_LINES_SHA256: &str =
 /// 内容取那一趟判红之后留在临时目录里的进度文件（打的行那条断言先红，进度文件那条没跑到、目录没删），SHA-256 `1553ba8e…`。
 /// 与上一版 `1d92a7ad…` 那一份逐行比：去掉每行的校验和、每行逐条不变量的表里 I-9.30–I-9.35 那几项、计划哈希换回之后 47 行逐字相同；
 /// 那一份每行的 CRC-32C 校验和都对得上正文。
+/// 2026-10-09 第 53 批合后重取：名字不变；内容取打的行判红那一趟（断言 `left` 为 `a4643c92…` 的那一趟）留在临时目录里的进度文件，
+/// SHA-256 `92ff4595…`。与上一版 `1553ba8e…` 那一份逐行比：去掉每行的校验和、每行逐条不变量的表里 I-9.36、I-9.37 那两项之后 47 行逐字相同；
+/// 那一份每行的 CRC-32C 校验和都对得上正文。
 const GOLDEN_PROGRESS_FILE_NAME: &str = "layer0-progress-golden_new_pool_file_creation_stream-fingerprint0-555baf1b8e6301e8874a473d44d0ce43260f946b695dfc0fbb3b0c5024bbbba6.txt";
 const GOLDEN_PROGRESS_FILE_SHA256: &str =
-    "1553ba8eee2fd2823372ddc32820ec899effbb48376a5d34d4d8d8e2e31f7c19";
+    "92ff4595a87332b8a915defde97ee09f05db4066dbcd4e58ece5f9214eb4481d";
 
 /// 一行打印里会随机器与时刻变的两样换掉：进度目录、`elapsed_seconds=` 的值。
 fn masked_printed_line(line: &str, directory: &Path) -> String {
diff --git a/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs b/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
index d52703c..02b892f 100644
--- a/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
+++ b/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
@@ -13,15 +13,24 @@ use common::{
     BuiltPool, FIXED_WRITE_TIME_SECONDS,
 };
 use singlefs_checker::image::{
-    InvariantVerdict, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
+    InvariantVerdict, EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
+    INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
+    LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
+    NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
     NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
-    QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
+    QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE, RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
+    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
     TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
     TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
     TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
 };
 use singlefs_checker::walk::{
-    check_pool_image, NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER, NO_QUEUED_INODE_IN_ANY_WALKED_VERSION,
+    check_pool_image, NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB,
+    NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER, NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER,
+    NO_QUEUED_INODE_IN_ANY_WALKED_VERSION, NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION,
+    NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
+    NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION,
     NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY, NO_TOMBSTONE_IN_ANY_WALKED_VERSION,
     NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD, NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
 };
@@ -46,7 +55,12 @@ use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, Trans
 use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
 
 /// 没有删除的会话上，删除线那几条各自报的不适用理由（池级 checker `walk` 里为每条写的那一句）：不变量 → 只认的那一句。
-const DELETION_INVARIANTS_WITHOUT_AN_OBJECT_ON_A_SESSION_WITHOUT_DELETION: [(&str, &str); 6] = [
+/// 前六行是墓碑与删除队列那几条（I-9.18–I-9.23）；后六行是退役树与谱系、退役容器容量那几条（I-9.31–I-9.34、I-9.36、I-9.37），
+/// 逐条照池级 checker 在 E 之后的镜像上实际报的登记：这条会话没有 unlink，退役树每一版都是零根，I-9.31 没有内部条目、I-9.32 没有退役记录、
+/// I-9.33 与 I-9.34 没有谱系上的退役记录；回退 D 只写了一条谱系记录，谱系树只一片容器、退役树零根，I-9.36 没有非最右的容器；
+/// `check_pool_image` 是旧入口、没给谱系与退役旋钮，I-9.37 不判。I-9.35（退役完备）不登记：候选集里相邻两版的父版读得出、叶身份集没少，
+/// 真被判过、要成立。
+const DELETION_INVARIANTS_WITHOUT_AN_OBJECT_ON_A_SESSION_WITHOUT_DELETION: &[(&str, &str)] = &[
     (
         QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
         NO_QUEUED_INODE_IN_ANY_WALKED_VERSION,
@@ -71,6 +85,30 @@ const DELETION_INVARIANTS_WITHOUT_AN_OBJECT_ON_A_SESSION_WITHOUT_DELETION: [(&st
         TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
         NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
     ),
+    (
+        RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION,
+    ),
+    (
+        EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
+        NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION,
+    ),
+    (
+        RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
+        NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
+    ),
+    (
+        RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+        NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
+    ),
+    (
+        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+        NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER,
+    ),
+    (
+        LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
+        NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB,
+    ),
 ];
 
 fn content_of(length: usize, seed: usize) -> Vec<u8> {
@@ -372,10 +410,11 @@ fn raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slo
                 .iter()
                 .find(|(exempted, _)| exempted == invariant)
         {
-            // 删除线那几条（I-9.18–I-9.23）在这条没有删除的会话上没有对象：每一版的队列都空、墓碑树都是零根，报不适用；
+            // 删除线那几条（I-9.18–I-9.23、I-9.31–I-9.34、I-9.36、I-9.37）在这条没有删除的会话上没有对象：每一版的队列都空、
+            // 墓碑树与退役树都是零根、谱系树只一片容器、旧入口没给谱系与退役旋钮，报不适用；
             // 每条只认池级 checker 为它写的那一句理由，别的不适用（含缺省理由「这个镜像上没有它判的对象」）照红
             // （判别力在 `checker_known_bad_images.rs` 与 `pool_checker_judges_the_deletion_state_of_every_walked_version.rs`）。
-            // 回退 D 写出了谱系树，I-9.17 在 E 之后的镜像上真被评估过、要成立；I-9.24（空队列报成立）与 I-9.29 同样要成立。
+            // 回退 D 写出了谱系树，I-9.17 在 E 之后的镜像上真被评估过、要成立；I-9.24（空队列报成立）、I-9.29 与 I-9.35 同样要成立。
             assert_eq!(
                 *verdict,
                 InvariantVerdict::NotApplicable(reason),
`````

### mutations-append.tsv

`````text
第 54 批：抬 F 之后 E 镜像的豁免表删掉 I-9.31 那一行（没有删除的会话上它报的不适用不再被认）	crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs	    (\n        RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,\n        NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION,\n    ),\n		-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
第 54 批：抬 F 之后 E 镜像的豁免表删掉 I-9.32 那一行（没有删除的会话上它报的不适用不再被认）	crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs	    (\n        EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,\n        NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION,\n    ),\n		-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
第 54 批：抬 F 之后 E 镜像的豁免表删掉 I-9.33 那一行（没有删除的会话上它报的不适用不再被认）	crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs	    (\n        RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,\n        NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,\n    ),\n		-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
第 54 批：抬 F 之后 E 镜像的豁免表删掉 I-9.34 那一行（没有删除的会话上它报的不适用不再被认）	crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs	    (\n        RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,\n        NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,\n    ),\n		-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
第 54 批：抬 F 之后 E 镜像的豁免表删掉 I-9.36 那一行（没有删除的会话上它报的不适用不再被认）	crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs	    (\n        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,\n        NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER,\n    ),\n		-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
第 54 批：抬 F 之后 E 镜像的豁免表删掉 I-9.37 那一行（没有删除的会话上它报的不适用不再被认）	crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs	    (\n        LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,\n        NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB,\n    ),\n		-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored raising_the_floor_to_the_first	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
`````
