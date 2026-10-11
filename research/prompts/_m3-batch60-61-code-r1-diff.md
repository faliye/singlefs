# 删除线第 60、61 批改动全文（2026-10-11）

## m3-batch60/gpu-far-left

````diff
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index d212408..0119bef 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -14025,16 +14025,36 @@ fn the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement
 }
 
 // ───────────── 第 59 批 L1（判决 `research/prompts/m3-batch58-code-r1-main-verification.md` 第三节 L1；两条腿 K1-3）：少一条的那片之后还有满的一片、根环转过一圈 ─────────────
+// ───────────── 第 60 批 N1（判决 `research/prompts/m3-batch59-code-r1-main-verification.md` 第三节 N1；攻方 M1-4、正推 M1-1）：少一条的那片离最右三片 ─────────────
 
-/// 写的时候逐条换的旋钮值：2、2、1、1、2、1 写出 [2, 1, 2, 1]（`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的
+/// 只一棵容器树逐条换旋钮写、再把根环转过一圈的那几张盘怎么造：写第 k 条记录时这一棵一片几条，根环转过一圈之后这一棵按叶序每片的记录数。
+struct ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver {
+    records_per_container_of_each_record: &'static [u64],
+    record_counts_in_leaf_order: &'static [usize],
+}
+
+/// 少一条的那片之后还有一片满的（第 59 批 L1）：旋钮 2、2、1、1、2、1 写出 [2, 1, 2, 1]（`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的
 /// `A_SHORT_CONTAINER_BEFORE_A_FULL_ONE` 那一形）。第 58 批 J3 那三条 [2, 1, 1] 上少一条的那片只会落在倒数第二位，GPU 扫描内核
 /// 「只判倒数第二片」在它们上面与正确判法逐位相同；这里少一条的容器 2 之后还有一片满的。
-const KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_CONTAINER_BEFORE_A_FULL_ONE: [u64; 6] =
-    [2, 2, 1, 1, 2, 1];
-/// 按叶序每片的记录数。
-const RECORD_COUNTS_OF_A_SHORT_CONTAINER_BEFORE_A_FULL_ONE: [usize; 4] = [2, 1, 2, 1];
+const A_SHORT_CONTAINER_BEFORE_A_FULL_ONE:
+    ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver =
+    ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver {
+        records_per_container_of_each_record: &[2, 2, 1, 1, 2, 1],
+        record_counts_in_leaf_order: &[2, 1, 2, 1],
+    };
+
+/// 少一条的那片离最右三片（第 60 批 N1）：旋钮 2、2、1、1、2、2、2、1 写出 [2, 1, 2, 2, 1]（`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的
+/// `A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST` 那一形，第 59 批 L2）。上面 [2, 1, 2, 1] 与第 58 批 J3 那几条上少一条的那片离最右
+/// 都不超过两片，GPU 扫描内核「只判最右之前两片」（攻方 `research/prompts/m3-batch59-code-r1-opus-model/` 的 `w-last-two`）在它们上面与正确判法逐位相同；
+/// 这里容器 2 在倒数第三片之前。写的中途 [2, 1, 1]、[2, 1, 2]、[2, 1, 2, 1] 那几版里容器 2 离最右不超过两片，留在候选集里时那一形也红，所以要转出去。
+const A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST:
+    ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver =
+    ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver {
+        records_per_container_of_each_record: &[2, 2, 1, 1, 2, 2, 2, 1],
+        record_counts_in_leaf_order: &[2, 1, 2, 2, 1],
+    };
 
-/// 第 59 批 L1 那三条流写的是哪一棵容器树（另两棵照产品）。
+/// 第 59 批 L1、第 60 批 N1 那几条流写的是哪一棵容器树（另两棵照产品）。
 #[derive(Clone, Copy, Debug)]
 enum ContainerTreeWrittenBeforeTheRootRingTurnsOver {
     Tombstone,
@@ -14141,54 +14161,76 @@ impl ContainerTreeWrittenBeforeTheRootRingTurnsOver {
     }
 }
 
-/// 只 `tree` 这一棵按 [`KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_CONTAINER_BEFORE_A_FULL_ONE`] 逐条换旋钮写出 [2, 1, 2, 1]，
-/// 再建 30 次空 inode（[`turn_the_root_ring_over_by_creating_inodes`]）把根环转过一圈：写这棵树的那几版、连同写到一半时的 [2, 1, 1] 那一版
-/// （容器 2 在那一版是倒数第二片），都转出写完之后各版的回退候选集，那几版的状态上照判 [2, 1, 1]，写完之后的状态上候选集里只剩 [2, 1, 2, 1]
-/// （与 harness 慢档 `checker_known_bad_images.rs` 里 `A_SHORT_CONTAINER_BEFORE_A_FULL_ONE` 那三条同形）。流从 mkfs 之后录起，被判的是最后一次根槽写。
-fn record_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
+/// 只 `tree` 这一棵按 `layout` 逐条换旋钮写出三片以上的容器，再建 30 次空 inode（[`turn_the_root_ring_over_by_creating_inodes`]）把根环转过一圈：
+/// 写这棵树的那几版、连同写到一半时少一条的那片离最右更近的那几版，都转出写完之后各版的回退候选集，那几版的状态上照判写到一半的那一形，
+/// 写完之后的状态上候选集里只剩 `layout.record_counts_in_leaf_order` 那一形（与 harness `checker_known_bad_images.rs` 里同名那一形的三条同形）。
+/// 流从 mkfs 之后录起，被判的是最后一次根槽写。
+fn record_containers_written_under_changing_knobs_with_the_root_ring_turned_over(
     tree: ContainerTreeWrittenBeforeTheRootRingTurnsOver,
+    layout: &ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver,
     tag: &str,
     name: &'static str,
 ) -> RecordedFlow {
-    let mut pool = tree.pool_written_under_these_knobs(
-        tag,
-        &KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
-    );
+    let mut pool =
+        tree.pool_written_under_these_knobs(tag, layout.records_per_container_of_each_record);
     turn_the_root_ring_over_by_creating_inodes(&mut pool);
     assert_eq!(
         tree.record_counts_of_each_container(&pool),
-        RECORD_COUNTS_OF_A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
+        layout.record_counts_in_leaf_order,
         "{tree:?} 按叶序每片的记录数（根环转过一圈之后）"
     );
     let versions = versions_with_the_first_file_through_the_last_publish(&pool);
     recorded_flow_of_the_pool(&pool, name, versions)
 }
 
-/// L1：`tree` 这一棵 [2, 1, 2, 1]、根环转过一圈，GPU 扫描内核与 CPU 新入口都给这一棵旋钮 2：每片都不超旋钮值、容器号从 1 逐片加一、
-/// 每片至少一条，只「不是最右的容器 2 装了 1 条，不是旋钮值 2」不成立，CPU 红这一棵的那一条（I-9.22 / I-9.37），GPU 红在同样的状态、同样的位上。
-/// GPU 扫描内核只判倒数第二片（第 3 片，装满 2 条）在根环转过之后的状态上照绿，就在那里与 CPU 不一致。
-fn assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_on_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
+/// L1、N1：`tree` 这一棵按 `layout` 写、根环转过一圈，GPU 扫描内核与 CPU 新入口都给这一棵旋钮 2：每片都不超旋钮值、容器号从 1 逐片加一、
+/// 每片至少一条，只「不是最右的容器 2 装了 1 条，不是旋钮值 2」不成立。CPU 在这条流的全部状态上红过的不变量恰好是这一棵的那一条
+/// （I-9.22 / I-9.37；另两棵照产品、不红，别的不变量在合法写出来的盘上也不红），GPU 红在同样的状态、同样的位上。
+/// [2, 1, 2, 1] 上 GPU 扫描内核只判倒数第二片（第 3 片，装满 2 条）、[2, 1, 2, 2, 1] 上只判最右之前两片（第 3、4 片，都装满 2 条），
+/// 在根环转过之后的状态上照绿，就在那里与 CPU 不一致。
+fn assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
     test_name: &str,
     tree: ContainerTreeWrittenBeforeTheRootRingTurnsOver,
+    layout: &ContainersWrittenUnderChangingKnobsBeforeTheRootRingTurnsOver,
     tag: &str,
     name: &'static str,
 ) {
-    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
-        test_name,
-        &record_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
-            tree, tag, name,
-        ),
+    let flow = record_containers_written_under_changing_knobs_with_the_root_ring_turned_over(
+        tree, layout, tag, name,
+    );
+    let knob_capacity_invariant = tree.knob_capacity_invariant();
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
+        return;
+    };
+    let Some(comparison) = compare_on_every_state_given_the_container_capacity_knobs(
+        cards,
+        &flow,
         tree.knobs_judging(KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER),
-        tree.knob_capacity_invariant(),
+    ) else {
+        return;
+    };
+    assert_eq!(
+        comparison.cpu_checker_red_invariants,
+        std::collections::BTreeSet::from([knob_capacity_invariant]),
+        "流 {} 上 CPU 新入口带旋钮值红过的不变量恰好是 {knob_capacity_invariant}",
+        flow.name
+    );
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}（GPU 扫描内核编进旋钮值）：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
     );
 }
 
 #[test]
 fn the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
 ) {
-    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_on_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
+    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
         "the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over",
         ContainerTreeWrittenBeforeTheRootRingTurnsOver::Tombstone,
+        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
         "crash-judge-gpu-tombstone-containers-of-two-one-two-and-one-records-ring-turned",
         "tombstone-containers-of-two-one-two-and-one-records-with-the-root-ring-turned-over",
     );
@@ -14197,9 +14239,10 @@ fn the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_c
 #[test]
 fn the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
 ) {
-    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_on_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
+    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
         "the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over",
         ContainerTreeWrittenBeforeTheRootRingTurnsOver::Lineage,
+        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
         "crash-judge-gpu-lineage-containers-of-two-one-two-and-one-records-ring-turned",
         "lineage-containers-of-two-one-two-and-one-records-with-the-root-ring-turned-over",
     );
@@ -14208,10 +14251,47 @@ fn the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_conta
 #[test]
 fn the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
 ) {
-    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_on_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
+    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
         "the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over",
         ContainerTreeWrittenBeforeTheRootRingTurnsOver::Retirement,
+        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
         "crash-judge-gpu-retirement-containers-of-two-one-two-and-one-records-ring-turned",
         "retirement-containers-of-two-one-two-and-one-records-with-the-root-ring-turned-over",
     );
 }
+
+#[test]
+fn the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over(
+) {
+    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
+        "the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over",
+        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Tombstone,
+        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
+        "crash-judge-gpu-tombstone-containers-of-two-one-two-two-and-one-records-ring-turned",
+        "tombstone-containers-of-two-one-two-two-and-one-records-with-the-root-ring-turned-over",
+    );
+}
+
+#[test]
+fn the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over(
+) {
+    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
+        "the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over",
+        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Lineage,
+        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
+        "crash-judge-gpu-lineage-containers-of-two-one-two-two-and-one-records-ring-turned",
+        "lineage-containers-of-two-one-two-two-and-one-records-with-the-root-ring-turned-over",
+    );
+}
+
+#[test]
+fn the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over(
+) {
+    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_that_containers_written_under_changing_knobs_with_the_root_ring_turned_over_redden_only_the_knob_capacity_invariant(
+        "the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over",
+        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Retirement,
+        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
+        "crash-judge-gpu-retirement-containers-of-two-one-two-two-and-one-records-ring-turned",
+        "retirement-containers-of-two-one-two-two-and-one-records-with-the-root-ring-turned-over",
+    );
+}
````

### mutations-append.tsv

````text
第60批 N1 GPU 判器：旋钮那一支只判最右之前两片（墓碑树 [2, 1, 2, 2, 1]、根环转过一圈；攻方 M1-4）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (d_container_record_count(container_index) != knob_records) {	        if (container_index + 3u >= d_container_count && d_container_record_count(container_index) != knob_records) {	-p singlefs-checker-tier --release --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over	the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over
第60批 N1 GPU 判器：旋钮那一支只判最右之前两片（谱系树 [2, 1, 2, 2, 1]、根环转过一圈；攻方 M1-4）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (d_container_record_count(container_index) != knob_records) {	        if (container_index + 3u >= d_container_count && d_container_record_count(container_index) != knob_records) {	-p singlefs-checker-tier --release --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over	the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over
第60批 N1 GPU 判器：旋钮那一支只判最右之前两片（退役树 [2, 1, 2, 2, 1]、根环转过一圈；攻方 M1-4）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (d_container_record_count(container_index) != knob_records) {	        if (container_index + 3u >= d_container_count && d_container_record_count(container_index) != knob_records) {	-p singlefs-checker-tier --release --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over	the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_two_and_one_records_with_the_root_ring_turned_over
````

## m3-batch60/watermark-and-width-r2

````diff
diff --git a/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs b/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs
index 4d003e1..1ae0980 100644
--- a/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs
+++ b/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs
@@ -9,6 +9,7 @@
 use singlefs_checker::image::chosen_system_configurations;
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
+    TreeIdentifier,
 };
 use singlefs_core::allocator::{
     PoolAllocator, UnitAreaStartOffTheClusterSegmentBoundaryUnsupported,
@@ -26,7 +27,7 @@ use singlefs_core::mounted_read::{
 };
 use singlefs_core::pointer::NodePointer;
 use singlefs_core::records::{
-    AccountingEntry, TreeTableEntry, STATISTIC_ALLOCATED_BYTES, STATISTIC_INODE_WATERMARK,
+    AccountingEntry, TreeTableEntry, STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION,
     TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_INODE,
 };
 use singlefs_core::recovery::{
@@ -535,15 +536,22 @@ fn a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged
     );
 }
 
+/// 统计量 7：扩展点配额已用量（D5（快照 / 空间记账机制） 已定项 4 那张表第 7 行，不带设备维、不带树维）。登记过，核心层今天不写它，
+/// `singlefs_core::records` 里没有它的常量。
+const STATISTIC_EXTENSION_POINT_QUOTA_USED: u16 = 7;
+
 /// 从盘上重建一版要它自己那一行 inode 号水位（统计量 12、代段 = 根的 txg），不认台阶顶替（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」；
 /// I-3.13 记账行代段等于这一版的 txg）：第一个文件（txg 3，它那一行是第 1 级台阶）之后覆盖写一版（txg 4：池级与每盘的行换成代段 4，
-/// 照抄台阶 (12, 3)，加自己那一行 (12, 4)，16 行）。txg 4 那一版自己那一行删掉，换进上一版（txg 3）那一行盘 0 的已分配字节 (1, 0, 0, 3)，
-/// 原样带过来，那一版的根槽原地换成指着改过的树表的那一条：这一版只剩台阶 (12, 3) 那一行水位。
+/// 照抄台阶 (12, 3)，加自己那一行 (12, 4)，16 行）。txg 4 那一版自己那一行删掉，换进一行这一版的统计量 7（扩展点配额已用量）
+/// (7, 0, 0xFFFF_FFFF, 4)、seq 照删掉的那一行，那一版的根槽原地换成指着改过的树表的那一条：这一版只剩台阶 (12, 3) 那一行水位。
 /// 换进一行是为了让数记账条目照过：那一判不算台阶行（统计量 12、代段早于根的 txg），只删自己那一行时不算台阶的行剩 14 ≠ 3 + 6 × 2，
-/// 重建在数条目那一判先拒、到不了水位行那一判。数得进去、又不是这一版水位行的只能是统计量不为 12 的一行，而这一版里标签登记过、维度对、
-/// 代段等于 txg 的 14 个 key 都已经有行；换进来的这一行标签（1）、树段（0）、设备段（盘 0）都照 D5 已定项 8 那张表，代段 3 不晚于根、seq 不为 0，
-/// 记账条目宽、逐条判代与 seq、数条目三判照过，它只违反 I-3.13（非台阶行的代段等于这一版的 txg），重建今天不判那一条（池级 checker 判）。
-/// 不靠没登记的统计量标签：读者逐条判拒标签 > 12、数条目时不算标签 > 12 的行，这条照报水位行缺。
+/// 重建在数条目那一判先拒、到不了水位行那一判。数得进去、又不是这一版水位行的只能是统计量不为 12 的一行。这一版写了统计量 1–6、10、11、12
+/// （D5 已定项 8 那张表的写法），D5 已定项 4 那张表里登记过、这一版没写的还有 7、13、14，三项都不带设备维、不带树维；
+/// 换进来的统计量 7 那一行标签登记过、维度对（树段 0、设备段 0xFFFF_FFFF，D5 已定项 8 的写法）、代段等于这一版的 txg、seq 不为 0，
+/// 记账条目宽、逐条判代与 seq、数条目三判照过，I-3.13（非台阶行的代段等于这一版的 txg）也照过。它借的是 D5 已定项 8「没有对象的统计量不写行」
+/// 今天不判：这一版没有扩展点配额，这一行本不该有，重建今天不判这一条；读者将来判它，这条用例就红在那一判上，要换造法。
+/// 不换进上一版那一行已分配字节 (1, 0, 0, 3)：它违反 I-3.13，读者逐条判 I-3.13 时先报 I-3.13、到不了水位行那一判。
+/// 不靠没登记的统计量标签：读者逐条判拒标签 > 12、数条目时不算标签 > 12 的行，这条照报水位行缺；读者逐条判 I-3.13 时也照报。
 /// 从盘上重建 txg 4 那一版（`rebuild_version`，可写挂载与回退建上一版走的那一份）判它坏：记账树里没有这一版的水位行；
 /// 可写挂载建不出所选那一版，这个池系统配置里见证的 journal 计数器是 mkfs 写的 0，F2 乙不退
 /// （D23（journal 的角色与格式） 已定项 14「c_见证 = 0 … 都不退」），在任何写之前拒、报同一个失败。冷读文件内容的走读不读水位，不在这条用例里。
@@ -578,16 +586,12 @@ fn a_version_whose_own_inode_number_watermark_row_is_damaged_leaving_only_a_step
         ),
         "覆盖写那一版：16 行，水位那一统计量是台阶 (12, 3) 与它自己那一行 (12, 4)"
     );
-    let allocated_bytes_row_of_device_zero_from_the_first_version = *pool
-        .first
-        .accounting_entries
-        .iter()
-        .find(|row| row.statistic == STATISTIC_ALLOCATED_BYTES && row.device == DeviceIdentity(0))
-        .expect("第一个文件那一版每盘一行已分配字节");
-    assert_eq!(
-        allocated_bytes_row_of_device_zero_from_the_first_version.generation,
-        pool.first.root.checkpoint_txg,
-        "上一版那一行的代段是上一版的 txg（3）"
+    assert!(
+        overwrite
+            .accounting_entries
+            .iter()
+            .all(|row| row.statistic != STATISTIC_EXTENSION_POINT_QUOTA_USED),
+        "覆盖写那一版没写统计量 7：这一版没有扩展点配额（D5 已定项 8「没有对象的统计量不写行」）"
     );
     let damaged_root =
         root_whose_accounting_rows_are_rewritten(&mut pool.image, &overwrite.root, |rows| {
@@ -597,7 +601,14 @@ fn a_version_whose_own_inode_number_watermark_row_is_damaged_leaving_only_a_step
                 (STATISTIC_INODE_WATERMARK, overwrite_txg),
                 "末行是这一版自己那一行水位"
             );
-            rows.push(allocated_bytes_row_of_device_zero_from_the_first_version);
+            rows.push(AccountingEntry {
+                statistic: STATISTIC_EXTENSION_POINT_QUOTA_USED,
+                tree: TreeIdentifier(0),
+                device: DeviceIdentity(STATISTIC_NO_DEVICE_DIMENSION),
+                generation: overwrite_txg,
+                value: 0,
+                sequence: own_row.sequence,
+            });
             rows.sort_by_key(AccountingEntry::sort_key);
             assert_eq!(
                 (
@@ -610,6 +621,13 @@ fn a_version_whose_own_inode_number_watermark_row_is_damaged_leaving_only_a_step
                 (16, vec![CheckpointTxg(3)]),
                 "改过的这一版仍是 16 行，水位那一统计量只剩台阶 (12, 3)"
             );
+            assert!(
+                rows.iter()
+                    .filter(|row| !(row.statistic == STATISTIC_INODE_WATERMARK
+                        && row.generation < overwrite_txg))
+                    .all(|row| row.generation == overwrite_txg),
+                "台阶行之外每一行的代段都是这一版的 txg（I-3.13 照过）"
+            );
         });
     let ring_slot = target_for_publish(overwrite_txg, pool.geometry.root_ring_slots_per_region);
     let device_holding_the_root = DeviceIdentity(
diff --git a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
index 85407b6..af67ada 100644
--- a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
+++ b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
@@ -2036,11 +2036,30 @@ fn pool_whose_allocation_records_and_one_accounting_entry_carry_a_generation_aft
     devices
 }
 
+/// 台阶行那个池（txg 4）上两处一起坏，每一处都把指着它的校验和重算：
+/// ① 记账树根的条目宽 34 缩到 key 宽 22、16 条每条只留 key（[`pool_whose_accounting_root_carrying_a_step_row_is_narrowed_to_its_keys`]）：
+///    条目宽那一判报它，条目解不开、逐条判与数条目都轮不到。只坏这一处时只读恢复报记账条目窄于字段表，这里先核一遍；
+/// ② [`move_the_allocation_records_of_the_tree_table_slot_after_the_root`]。
+fn pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_root_is_narrowed_to_its_keys(
+) -> Devices {
+    let mut devices = pool_whose_accounting_root_carrying_a_step_row_is_narrowed_to_its_keys();
+    assert_eq!(
+        recover(&devices, JournalPolicy::Consult).outcome,
+        RecoveryOutcome::Failed {
+            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
+            failure: accounting_entry_narrower_than_its_field_table(),
+        },
+        "前提：只坏记账那一处时，只读恢复在 txg 4 那条根上报记账条目窄于字段表"
+    );
+    move_the_allocation_records_of_the_tree_table_slot_after_the_root(&mut devices);
+    devices
+}
+
 /// 场景：第一个文件之后再建一个空 inode（txg 4）；记账树根里 key 最小的那一行删掉（标签与代都不碰，记账条目数因此对不上），
 /// 罩这一版树表单元那个槽的两条分配记录代改成 5，晚于根；指着改过的节点的每一道校验和重算。
 /// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度排在数记账条目之前
 /// （先判得了自己那一条的，再数；判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节 H4）。
-/// 那一判挪到数记账条目之后，这份镜像报成记账条目数。排在数映射条目、记账条目逐条判之前那两个次序由下面两张镜像各钉一个。
+/// 那一判挪到数记账条目之后，这份镜像报成记账条目数。排在数映射条目、记账条目逐条判、记账条目宽之前那三个次序由下面三张镜像各钉一个。
 #[test]
 fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_recovery(
 ) {
@@ -2114,7 +2133,7 @@ fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_
 /// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度排在
 /// 记账条目逐条判代与 seq 之前（冷走读那一段的次序注释与重建相同：分配记录、条目宽、逐条判、数）。
 /// 那一判挪到记账条目逐条判之后（仍在两个数之前），这份镜像报成「记账条目的代晚于根或 seq 为 0」。这张盘每条记账条目宽都是 34，
-/// 条目宽那一判不坏，「分配记录排在条目宽之前」这半句它钉不到。可写挂载那一条在下面。
+/// 条目宽那一判不坏，「分配记录排在条目宽之前」这半句它钉不到，由最后两条（记账树根缩到 key 宽的那张盘）钉。可写挂载那一条在下面。
 #[test]
 fn allocation_records_whose_generation_is_after_the_root_are_refused_before_each_accounting_entry_is_judged_by_the_recovery(
 ) {
@@ -2148,6 +2167,42 @@ fn allocation_records_whose_generation_is_after_the_root_are_refused_before_each
     );
 }
 
+/// 场景：第一个文件之后再建一个空 inode（txg 4）；记账树根的条目宽缩到 key 宽、每条只留 key（条目数 16 不变），
+/// 罩这一版树表单元那个槽的两条分配记录代改成 5，晚于根；指着改过的节点的每一道校验和重算。
+/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度排在
+/// 记账条目宽那一判之前（冷走读那一段的次序：分配记录、条目宽、逐条判、数）。那一判挪到条目宽之后（逐条判与两个数之前），
+/// 这份镜像报成记账条目窄于字段表（22 对 34）；上面那张逐条判坏的盘条目宽都对，分不出这一形。可写挂载那一条在下面。
+#[test]
+fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_width_by_the_recovery(
+) {
+    let devices =
+        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_root_is_narrowed_to_its_keys();
+    let report = recover(&devices, JournalPolicy::Consult);
+    assert_eq!(
+        report.outcome,
+        RecoveryOutcome::Failed {
+            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
+            failure: allocation_record_span_zero_or_generation_after_the_root(),
+        },
+        "只读恢复在 txg 4 那条根上报分配记录的代晚于根，不报记账条目窄于字段表"
+    );
+}
+
+/// 场景：同上一条的坏镜像，交给可写挂载。
+/// 预期：可写挂载在任何写之前交回「分配记录跨度为 0，或分配代 / 释放代晚于根」：重建那一版逐条判分配记录在读记账树之前，
+/// 排在记账条目宽那一判之前，与冷走读报同一个成员。重建把那一判挪到条目宽之后（逐条判与数记账条目之前），这份镜像报成记账条目窄于字段表。
+#[test]
+fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_width_by_the_writable_mount(
+) {
+    let mut devices =
+        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_root_is_narrowed_to_its_keys();
+    let refusal = writable_mount_refused_before_any_write(&mut devices);
+    assert!(
+        matches!(&refusal, MountError::Recovery(failure) if *failure == allocation_record_span_zero_or_generation_after_the_root()),
+        "可写挂载报分配记录的代晚于根，不报记账条目窄于字段表：{refusal:?}"
+    );
+}
+
 // ─── 第 35 条续：单元区槽数那一处减法 ───
 
 /// 场景：700 MiB 的盘（单元区起点在 784 MiB）。预期：单元区槽数交回「盘末尾在单元区起点之前」，带盘的字节数与起点槽号；
````

### mutations-append.tsv

````text
第60批 N3 冷走读分配记录代与跨度挪到记账条目宽之后、逐条判之前（代晚于根的分配记录在记账树根缩到 key 宽的镜像上只读恢复报成记账条目窄于字段表）	crates/singlefs-core/src/recovery.rs	    allocation_record_generations_and_spans_are_judged(allocation_records, root)?;\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &roots.accounting.leaf_entries_in_key_order,\n    )?;\n	    every_accounting_entry_is_as_wide_as_its_field_table(\n        &roots.accounting.leaf_entries_in_key_order,\n    )?;\n    allocation_record_generations_and_spans_are_judged(allocation_records, root)?;\n	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- allocation_records_whose_generation_is_after_the_root	allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_width_by_the_recovery
第60批 N3 重建分配记录代与跨度挪到记账条目宽之后、逐条判之前（代晚于根的分配记录在记账树根缩到 key 宽的镜像上可写挂载报成记账条目窄于字段表）	crates/singlefs-core/src/recovery.rs	    allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;\n    let allocation_records: Vec<AllocationRecord> = allocation_tree_read.records.clone();\n    // 记账树可以是多层（D8（核心索引结构） 已定项 11）：从根往下整棵读回来，节点进映射、提示读不出经这一版的中央映射回退\n    // （与读别的树根同一条）；中央映射树同样整棵读回来（下面）。两棵树照冷走读判全（`code_two_tree::read_code_two_tree` 的\n    // `EveryHeaderAgainstItsReference`：节点头、层级、区间与 key 严格递增——有重复 key 的记账叶交出去，挂载之后第一次发布规划那棵树时\n    // 在 `transaction` 的断言上 panic，代码审阅第 33 条）。\n    let accounting_tree = read_code_two_tree(\n        &accounting_pointer,\n        &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),\n        CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,\n        root,\n        expected_filesystem_identifier,\n        &mut |pointer: &NodePointer| {\n            read_mapped_tree_node_via_hint_then_central_mapping(\n                reader,\n                pointer,\n                MappedTreeNodeClass::IndexNode,\n                &central_mapping_locations_of_key,\n                &mut stale_location_hint_hops_not_exposed_by_this_reader,\n            )\n        },\n    )?;\n    // 次序：条目宽、逐条判代与 seq、数条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`\n    // 第三节 G1）。数条目要按统计量与代段认台阶行，代写到根之后的台阶行认不出、被算进 3 + 6 × 盘数，「代晚于根」这一坏法就报成了条目数不对。\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &accounting_tree.leaf_entries_in_key_order,\n    )?;\n	    let allocation_records: Vec<AllocationRecord> = allocation_tree_read.records.clone();\n    // 记账树可以是多层（D8（核心索引结构） 已定项 11）：从根往下整棵读回来，节点进映射、提示读不出经这一版的中央映射回退\n    // （与读别的树根同一条）；中央映射树同样整棵读回来（下面）。两棵树照冷走读判全（`code_two_tree::read_code_two_tree` 的\n    // `EveryHeaderAgainstItsReference`：节点头、层级、区间与 key 严格递增——有重复 key 的记账叶交出去，挂载之后第一次发布规划那棵树时\n    // 在 `transaction` 的断言上 panic，代码审阅第 33 条）。\n    let accounting_tree = read_code_two_tree(\n        &accounting_pointer,\n        &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),\n        CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,\n        root,\n        expected_filesystem_identifier,\n        &mut |pointer: &NodePointer| {\n            read_mapped_tree_node_via_hint_then_central_mapping(\n                reader,\n                pointer,\n                MappedTreeNodeClass::IndexNode,\n                &central_mapping_locations_of_key,\n                &mut stale_location_hint_hops_not_exposed_by_this_reader,\n            )\n        },\n    )?;\n    // 次序：条目宽、逐条判代与 seq、数条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`\n    // 第三节 G1）。数条目要按统计量与代段认台阶行，代写到根之后的台阶行认不出、被算进 3 + 6 × 盘数，「代晚于根」这一坏法就报成了条目数不对。\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &accounting_tree.leaf_entries_in_key_order,\n    )?;\n    allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;\n	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- allocation_records_whose_generation_is_after_the_root	allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_width_by_the_writable_mount
````

## m3-batch60/core-light-rows

````diff
diff --git a/crates/singlefs-harness/tests/system_configuration_slot_is_overwritten_only_after_a_barrier.rs b/crates/singlefs-harness/tests/system_configuration_slot_is_overwritten_only_after_a_barrier.rs
index 07251d6..868b922 100644
--- a/crates/singlefs-harness/tests/system_configuration_slot_is_overwritten_only_after_a_barrier.rs
+++ b/crates/singlefs-harness/tests/system_configuration_slot_is_overwritten_only_after_a_barrier.rs
@@ -189,3 +189,60 @@ fn rolling_back_a_failed_acquisition_overwrites_the_older_slot_only_behind_a_bar
         "mkfs 之后没有一段里同一块盘写两次系统配置槽"
     );
 }
+
+/// 回卷之后再取号：盘 1 的取号写报错、盘 0 的取号写回卷成旧代号（同上一条），撤掉故障后同一个写入口再取一次号。
+/// 回卷写后面没有屏障，再取号的第一个取号写覆写的是盘 0 上回卷写之外的那一槽；两者之间要有取号自己那道屏障。
+/// 上一次发布末尾轮换之后有 C577 那道屏障，「重开取号」那一格里取号自己那道屏障是多余的；这一格里它是唯一隔开两写的那一道。
+#[test]
+fn acquisition_after_a_rolled_back_acquisition_overwrites_the_older_slot_only_behind_a_barrier() {
+    let mut pool = build_pool("a1-barrier-acquisition-after-rollback");
+    let plan = SharedFaultPlan::unarmed(geometry());
+    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<Recorded>)> = pool
+        .devices
+        .take()
+        .expect("新池新建文件写完，盘还开着")
+        .into_iter()
+        .map(|(identity, inner)| {
+            (
+                identity,
+                FaultInjectingBlockDevice::new(identity, inner, plan.clone()),
+            )
+        })
+        .collect();
+    plan.arm(FaultSchedule {
+        fault: InjectedFault::WriteFails,
+        device: FaultDeviceSelector::OnlyDevice(DeviceIdentity(1)),
+        placement: FaultPlacement::OffsetBelow(
+            singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
+                * u64::from(parameters().geometry.fixed_structure_slot_spacing),
+        ),
+        counting: FaultCounting::AcrossThePool,
+        occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
+    });
+    let publish_parameters = parameters();
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    let failure = match acquire_instance(&mut writer).expect_err("盘 1 的取号写报错，取号必须失败") {
+        InstanceAcquisitionFailed::Acquisition(acquisition) => acquisition,
+        InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
+            first_device_reporting_a_read_error,
+        } => panic!("这条用例里每块盘两槽都读得出自证过的系统配置，取号不该判不出：{first_device_reporting_a_read_error:?}"),
+        InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
+            device,
+        } => panic!("这条用例里每块盘两槽都读得出自证过的系统配置，取号不该拒在见证值那一核：{device:?}"),
+    };
+    assert!(
+        matches!(failure.rollback, AcquisitionRollback::RolledBack),
+        "盘 0 那份已写出，回卷做成：{failure:?}"
+    );
+    plan.disarm();
+    acquire_instance(&mut writer).expect("故障撤掉之后，同一个写入口再取号做成");
+    drop(writer);
+    assert_eq!(
+        second_system_configuration_writes_in_one_segment(
+            &pool.stream.operations(),
+            pool.mkfs_operation_count
+        ),
+        Vec::new(),
+        "mkfs 之后没有一段里同一块盘写两次系统配置槽"
+    );
+}
diff --git a/crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs b/crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs
index 6c219af..55d39b7 100644
--- a/crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs
+++ b/crates/singlefs-harness/tests/shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring.rs
@@ -16,7 +16,8 @@
 //! - 影子账：先造一条被抛弃的根——实例 1 最新的根 txg 24 那次发布的系统配置轮换没落、下一次可写挂载时它的根槽与数据单元一时读不出，择根落到 23、
 //!   实例 2 写行（行 (1, 23)），之后 24 读得出、按实例表被抛弃。再连崩到 txg 36 那一槽也被跳过（24 那一槽不被盖掉）、做完一次可写挂载，
 //!   再挂时影子账按候选集算：实例 1 那张实例表被被抛弃的 24 引用，也被洞里实例 1 的有效根（在窗口外）引用，豁免、不隔离；
-//!   照一圈窗口滤候选时窗口外那几条不豁免，那几槽被隔离。
+//!   照一圈窗口滤候选时窗口外那几条不豁免，而那几槽这时还在 defer 里、分配位置着，可写挂载那一遍只隔离分配位没置着的槽，豁免不豁免都不隔离；
+//!   所以判别力放在抬 F 那一串：再挂那一刻 24 的根槽读回全零（根环表里没记着它），抬 F 时照读盘取它的落点、整段隔离，不豁免就隔离那几槽。
 
 mod common_root_ring_holes;
 
@@ -29,7 +30,7 @@ use common_root_ring_holes::{
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{
-    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
 };
 use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes, WriteDurability};
 use singlefs_core::make_filesystem::make_filesystem;
@@ -390,8 +391,8 @@ fn pool_with_the_newest_root_of_instance_one_abandoned(last: u64) -> MemoryPool
 /// 一次可写挂载做完；再可写挂载一次，看重建分配器影子账隔离了几槽。
 /// 预期（前提）：盘上只有一条被抛弃的根 24；有被它引用、也只被锚在最新有效根上的一圈窗口外的有效根引用（窗口里的有效根与现行那一版的账都不引用）的槽。
 /// 影子账按候选集（按实例表有效 ∧ txg ≥ F，不按窗口滤）豁免：每块盘隔离的槽数 = 被抛弃的根引用的槽里，候选根与现行那一版的账都不引用的那几槽的跨度之和。
-/// 判别力：影子账的候选过滤改回一圈窗口（`crates/mutations.tsv`「第10g批 影子账候选过滤改回一圈窗口…」）时窗口外那几条根不豁免，
-/// 它们与被抛弃的根同引用的槽被隔离，隔离数变多，最后一项红。
+/// 这一条对影子账的候选过滤改回一圈窗口（`crates/mutations.tsv`「第10g批 影子账候选过滤改回一圈窗口…」）不敏感：窗口外那几条根不豁免时，
+/// 它们与被抛弃的根同引用的槽还在 defer 里、分配位置着，可写挂载那一遍不隔离它们，隔离数不变。那一条变异由抬 F 那一串的下一条用例抓。
 #[test]
 fn a_remount_isolates_no_slot_an_abandoned_root_shares_with_valid_roots_left_in_the_holes_outside_the_one_revolution_window(
 ) {
@@ -501,3 +502,204 @@ fn a_remount_isolates_no_slot_an_abandoned_root_shares_with_valid_roots_left_in_
         "影子账按整个环的候选集豁免：窗口外的有效根与被抛弃的根同引用的 {shared_only_with_candidates_outside_the_window:?} 不隔离"
     );
 }
+
+fn read_bytes(
+    devices: &RingHoleDevices,
+    device: DeviceIdentity,
+    offset: DeviceOffsetInBytes,
+    length: usize,
+) -> Vec<u8> {
+    let mut bytes = vec![0u8; length];
+    devices
+        .iter()
+        .find(|(identity, _)| *identity == device)
+        .expect("池里有这块盘")
+        .1
+        .read_at(offset, &mut bytes)
+        .expect("读内存盘");
+    bytes
+}
+
+/// `image` 上被抛弃的根与它们引用、只与锚在最新有效根上的一圈窗口外的候选根（按实例表有效 ∧ txg ≥ `floor`）同引用的槽
+/// （窗口里的候选根与现行那一版的账都不引用），按 (盘, 起始槽) 认。
+struct SlotsSharedOnlyWithCandidatesOutsideTheWindow {
+    abandoned: Vec<(InstanceGeneration, CheckpointTxg)>,
+    newest_valid_root: CheckpointTxg,
+    slots: BTreeSet<(DeviceIdentity, u64)>,
+}
+
+fn slots_shared_only_with_candidates_outside_the_window(
+    image: &MemoryPool,
+    floor: CheckpointTxg,
+) -> SlotsSharedOnlyWithCandidatesOutsideTheWindow {
+    let roots = roots_of_the_ring(image);
+    let start_slots_allocated_under = |root: &RootRecord| -> Vec<(DeviceIdentity, u64)> {
+        allocation_records_under_root(image, root)
+            .expect("这一版的账读得出")
+            .into_iter()
+            .filter(|record| !record.is_released)
+            .map(|record| (record.device, record.slot.0))
+            .collect()
+    };
+    let newest_valid_root = roots
+        .valid
+        .iter()
+        .map(|root| root.checkpoint_txg)
+        .max()
+        .expect("有有效根");
+    let candidates: Vec<&RootRecord> = roots
+        .valid
+        .iter()
+        .filter(|root| root.checkpoint_txg >= floor)
+        .collect();
+    let referenced_by_candidates_inside_the_window: BTreeSet<(DeviceIdentity, u64)> = candidates
+        .iter()
+        .filter(|root| inside_the_one_revolution_window(root, newest_valid_root))
+        .flat_map(|root| start_slots_allocated_under(root))
+        .collect();
+    let referenced_by_candidates_outside_the_window: BTreeSet<(DeviceIdentity, u64)> = candidates
+        .iter()
+        .filter(|root| !inside_the_one_revolution_window(root, newest_valid_root))
+        .flat_map(|root| start_slots_allocated_under(root))
+        .collect();
+    let allocated_in_the_current_version: BTreeSet<(DeviceIdentity, u64)> =
+        start_slots_allocated_under(&roots.newest)
+            .into_iter()
+            .collect();
+    let slots = roots
+        .abandoned
+        .iter()
+        .flat_map(start_slots_allocated_under)
+        .filter(|key| {
+            referenced_by_candidates_outside_the_window.contains(key)
+                && !referenced_by_candidates_inside_the_window.contains(key)
+                && !allocated_in_the_current_version.contains(key)
+        })
+        .collect();
+    SlotsSharedOnlyWithCandidatesOutsideTheWindow {
+        abandoned: roots
+            .abandoned
+            .iter()
+            .map(|root| (root.instance, root.checkpoint_txg))
+            .collect(),
+        newest_valid_root,
+        slots,
+    }
+}
+
+/// 场景：同上一条造出被抛弃的根 24 与洞里一圈窗口外的有效根（连崩 11 次、一次可写挂载做完）；再可写挂载一次，那一刻 24 的根槽读回全零、
+/// 挂载交回之后原样写回；同一个进程里把 F 抬到盘上现算的生效值（F 不动，窗口外的有效根仍是候选）。
+/// 抬 F 那一串的影子账里 24 在根环表里没记着（挂载那一刻那一槽读回全零，C 那一圈在那一槽也没记），照读盘取它引用的落点、整段隔离
+/// （`PoolAllocator::isolate_abandoned`：回收时没有一份补得上，不看分配位）。
+/// 预期（前提）：盘上只有一条被抛弃的根 24；有被它引用、也只被窗口外的候选根引用的槽。抬完之后那几槽一个都不隔离（候选按整个环豁免）。
+/// 判别力：影子账的候选过滤改回一圈窗口（`crates/mutations.tsv`「第10g批 影子账候选过滤改回一圈窗口…」）时窗口外那几条根不豁免，
+/// 那几槽在这里整段隔离，最后一项红。上一条用例在可写挂载那一遍取样：那几槽还在 defer 里、分配位置着，只隔离分配位没置着的槽的那一路
+/// 豁免不豁免都不隔离，对这条变异不敏感。
+#[test]
+fn raising_the_floor_isolates_no_slot_an_abandoned_root_unrecorded_in_the_root_ring_table_shares_with_valid_roots_left_in_the_holes_outside_the_one_revolution_window(
+) {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let mut image = pool_with_the_newest_root_of_instance_one_abandoned(LAST_TXG_OF_INSTANCE_ONE);
+    // 迭代上界：连崩的次数，每次一个洞；跨轮携带的是崩溃之后的盘面。
+    for _ in 0..CRASHES_BEFORE_THE_ROW_ROOT_AFTER_THE_ABANDONMENT {
+        let (crashed, _row_txg) = crash_a_mount_before_its_row_root(&image);
+        image = crashed;
+    }
+    let mut devices = devices_of(&image);
+    mount_writable(&parameters, &mut devices).expect("连崩之后的可写挂载做完");
+
+    let target = target_for_publish(
+        CheckpointTxg(LAST_TXG_OF_INSTANCE_ONE),
+        parameters.geometry.root_ring_slots_per_region,
+    );
+    let root_slot_device =
+        parameters.region_devices[usize::try_from(target.region).expect("区域号")];
+    let root_slot_offset = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing);
+    let root_slot_bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
+    let root_slot_of_the_abandoned_root = read_bytes(
+        &devices,
+        root_slot_device,
+        root_slot_offset,
+        root_slot_bytes,
+    );
+    write_bytes(
+        &mut devices,
+        root_slot_device,
+        root_slot_offset,
+        &vec![0u8; root_slot_bytes],
+    );
+    let Mounted {
+        mut allocator,
+        current,
+        ..
+    } = mount_writable(&parameters, &mut devices).expect("24 的根槽读回全零时照常可写挂载");
+    let PoolVersion::WithFile(mut current) = current else {
+        panic!("现行那一版带文件");
+    };
+    assert_eq!(
+        read_bytes(
+            &devices,
+            root_slot_device,
+            root_slot_offset,
+            root_slot_bytes
+        ),
+        vec![0u8; root_slot_bytes],
+        "前提：这次挂载没写到 24 那一槽"
+    );
+    write_bytes(
+        &mut devices,
+        root_slot_device,
+        root_slot_offset,
+        &root_slot_of_the_abandoned_root,
+    );
+    let image_before_the_raise = pool_of(&devices);
+    let floor = {
+        let system_configuration =
+            choose_system_configuration(&image_before_the_raise).expect("系统配置");
+        let immutable = system_configuration.immutable;
+        effective_rollback_floor(
+            &image_before_the_raise,
+            &immutable.region_devices,
+            &immutable.sizes,
+            &immutable.filesystem_identifier,
+        )
+    };
+    let shared =
+        slots_shared_only_with_candidates_outside_the_window(&image_before_the_raise, floor);
+    assert!(
+        shared.abandoned == vec![(InstanceGeneration(1), CheckpointTxg(LAST_TXG_OF_INSTANCE_ONE))]
+            && !shared.slots.is_empty(),
+        "前提：只有被抛弃的根 txg 24；它与窗口外的候选根同引用、窗口里的候选根与现行那一版的账都不引用的槽 {:?}（最新有效根 {:?}，F {floor:?}）",
+        shared.slots,
+        shared.newest_valid_root
+    );
+
+    raise_rollback_floor(
+        &parameters,
+        &mut devices,
+        &mut allocator,
+        &mut current,
+        floor,
+        ShadowLedger::On,
+    )
+    .expect("F 抬到生效值（不高于上限）");
+    let isolated_among_the_shared: Vec<(DeviceIdentity, u64)> = shared
+        .slots
+        .iter()
+        .copied()
+        .filter(|(device, slot)| {
+            allocator
+                .devices
+                .iter()
+                .find(|device_map| device_map.device == *device)
+                .expect("池里有这块盘")
+                .is_isolated(SlotNumber(*slot))
+        })
+        .collect();
+    assert_eq!(
+        isolated_among_the_shared,
+        Vec::<(DeviceIdentity, u64)>::new(),
+        "影子账按整个环的候选集豁免：窗口外的有效根与读盘得来的被抛弃根 24 同引用的 {:?} 不隔离",
+        shared.slots
+    );
+}
````

### mutations-append.tsv

````text
并行线一：层 0：一次多条记录的发布每条记录都带末条标志（里程碑并行线一验收第 4 条第一个变异「提交标记提前到第一条记录」：崩在记录之间时半次发布施加上去）	crates/singlefs-core/src/transaction.rs	        let place_in_publish = if transaction_of_the_next_record.is_none() {	        let place_in_publish = if true {	-p singlefs-checker-tier --test crash_enumeration_multi_record_publish_stream -- every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it	every_crash_state_between_the_records_of_a_multi_record_publish_keeps_the_version_before_it
````

### mutations-replacements.tsv

````text
增补 1：发布路径漏计数据单元的写（新池新建文件按种类的合计对不上录制器在设备一层记下的写）	crates/singlefs-core/src/transaction.rs	                    self.writes_by_structure_kind.count_write_call(kind, unit);	                    if !matches!(identity, TransactionUnit::Data(_)) { self.writes_by_structure_kind.count_write_call(kind, unit); }	-p singlefs-harness --test write_accounting_per_structure -- new_pool_file_creation_and_overwrite_writes_by_kind	new_pool_file_creation_and_overwrite_writes_by_kind_add_up_to_the_recorded_writes_and_the_byte_table
步 6：恢复从系统配置 tail 起逐条验证记录的点名单元、失配即中止（E78 的自我中止形态）	crates/singlefs-core/src/recovery.rs	            let records = scan_journal(reader, &system_configuration);	            let records = scan_journal(reader, &system_configuration);\n            let trusted_tail_mismatch = records\n                .values()\n                .filter(|record| record.counter > system_configuration.quantities.journal_tail)\n                .any(|record| {\n                    replay_journal(\n                        reader,\n                        &RootRecord {\n                            instance: record.instance,\n                            checkpoint_txg: CheckpointTxg(record.checkpoint_txg.0 - 1),\n                            ..root\n                        },\n                        system_configuration.immutable.sizes.journal_ring_bytes,\n                        &BTreeMap::from([((record.instance, record.counter), record.clone())]),\n                        true,\n                    )\n                    .is_ok_and(|(report, _)| report.verification_failed > 0)\n                });\n            if trusted_tail_mismatch {\n                return RecoveryReport {\n                    outcome: RecoveryOutcome::Failed {\n                        root: Some(root_key),\n                        failure: RecoveryFailure::NoValidRoot,\n                    },\n                    effective_root: None,\n                    journal: JournalScanReport::default(),\n                    mapping_fallbacks,\n                    devices_without_a_mountable_system_configuration_slot,\n                };\n            }	-p singlefs-checker-tier --test crash_enumeration_fixed_script_stream -- stale_tail	stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state
并行线一：C490：extent 叶记录 key 的 offset 段写成单元序号（并行线一验收第 4 条第二个变异：读回错位判红）	crates/singlefs-core/src/extent_tree.rs	        .map(|(unit, pointer)| build_extent_record(inode, unit * payload, *pointer))	        .map(|(unit, pointer)| build_extent_record(inode, unit, *pointer))	-p singlefs-harness --test sequential_write_split_into_one_unit_transactions -- the_second_extent_leaf_record_key_is_the_file_byte_offset_of_the_second_data_unit	the_second_extent_leaf_record_key_is_the_file_byte_offset_of_the_second_data_unit
实二七（fsync 失败掉盘的核查，调度记录 2026-09-24 第三节；D2 已定项 13 挂载准入、D18 已定项 11 可写挂载的顺序与前提）：有盘不带所选那一版也照样取号（拒可写那一句关掉），盘 1 是空盘时挂成可写	crates/singlefs-core/src/mount.rs	    if !devices_without.is_empty() {\n        return Err(\n            MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion {	    if false && !devices_without.is_empty() {\n        return Err(\n            MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion {	-p singlefs-harness --test fsync_drop_and_devices_without_the_selected_version -- mount_writable_with_a_stale_device_one_refuses_or_keeps_two_copies	mount_writable_with_a_stale_device_one_refuses_or_keeps_two_copies
C673 ⑤ I-9.27 判别力（攻方 M1）：发布写树表时非 0 的头 ID 一律写字面 65537，树 ID 从根环水位发的那一版上头 ID 与 inode 树 65549 对不上	crates/singlefs-core/src/transaction.rs	            birth_txg: plan.tree_birth_txg,\n            head_identifier,\n        };	            birth_txg: plan.tree_birth_txg,\n            head_identifier: if head_identifier == 0 { 0 } else { singlefs_format::TREE_IDENTIFIER_INODE },\n        };	-p singlefs-harness --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement --test checker_known_bad_images -- a_first_file_version_issuing_its_trees_from_the_ring_watermark_after_a_recovery_keeps_the_head_identifier_relation	a_first_file_version_issuing_its_trees_from_the_ring_watermark_after_a_recovery_keeps_the_head_identifier_relation
E171R1 装置 U12 那一侧：第 31 批 core 记根断言删掉、没装根环表的分配器静默返回（登记修订 6 第 6.7 节第 4 条）	crates/singlefs-core/src/allocator.rs	        let occupancy = self.root_ring.as_mut().expect(	        let Some(occupancy) = self.root_ring.as_mut() else {\n            return;\n        };\n        let _message_of_the_removed_assertion = (	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes -- a_live_stream_without_the_root_ring_panics_in_core_at_its_first_root	a_live_stream_without_the_root_ring_panics_in_core_at_its_first_root
实审A1 第19条：取号写之前不过屏障（上一次轮换与取号写同段）	crates/singlefs-core/src/transaction.rs	    if let Err(cause) = pool.perform_commit_step(CommitStep::Barrier) {\n        return Err(AcquisitionFailed {	    if let Err(cause) = Ok::<(), BlockDeviceError>(()) {\n        return Err(AcquisitionFailed {	-p singlefs-harness --test system_configuration_slot_is_overwritten_only_after_a_barrier -- acquisition_after_a_rolled_back_acquisition_overwrites_the_older_slot_only_behind_a_barrier	acquisition_after_a_rolled_back_acquisition_overwrites_the_older_slot_only_behind_a_barrier
第10g批 影子账候选过滤改回一圈窗口（锚取这一遍读出的最新有效根、长 3S：洞里窗口外的有效根不豁免它与被抛弃的根同引用的槽，隔离数变多）	crates/singlefs-core/src/mount.rs	    for root in roots\n        .iter()\n        .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)\n    {	    let window_anchor_of_the_candidates = roots\n        .iter()\n        .filter(|root| !is_abandoned(root))\n        .map(|root| root.checkpoint_txg.0)\n        .max()\n        .unwrap_or(0);\n    let ring_length_of_the_candidates = ROOT_RING_REGIONS\n        * crate::recovery::choose_system_configuration(devices)\n            .expect("系统配置")\n            .immutable\n            .sizes\n            .root_ring_slots_per_region\n            .count();\n    for root in roots.iter().filter(|root| {\n        !is_abandoned(root)\n            && root.checkpoint_txg >= floor\n            && root.checkpoint_txg.0 + ring_length_of_the_candidates > window_anchor_of_the_candidates\n    }) {	-p singlefs-harness --test shadow_ledger_candidates_and_the_deferred_reclaim_threshold_take_the_whole_ring -- raising_the_floor_isolates_no_slot_an_abandoned_root_unrecorded_in_the_root_ring_table_shares_with_valid_roots_left_in_the_holes_outside_the_one_revolution_window	raising_the_floor_isolates_no_slot_an_abandoned_root_unrecorded_in_the_root_ring_table_shares_with_valid_roots_left_in_the_holes_outside_the_one_revolution_window
````

### mutations-delete.txt

````text
并行线一：层 0：共享内生块挪到第一条记录里点名（里程碑并行线一验收第 4 条第一个变异「提交标记提前到第一条记录」在一事务一记录下的形态：半次发布施加上去）
````

## m3-batch61/import-planned-blocks-2

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -14,7 +14,8 @@
 //! 3. **对账**：一轮收尾，录入表里有、判定块表里没有的块就是没核完的（卡掉线、进程被杀），重新分给还在的一方补跑；
 //!    领块只在内存里，库里不记谁领了，不会有块被一张死卡占着。
 //! 4. **分工与跑法**：块按权重分给几方（一方一台机器的 CPU，或一张卡，权重取空闲显存），各写各的库，最后
-//!    [`crate::verdict_store::VerdictStore::import_from`] 并成一个；跑法由配置的两个开关定：`ENABLE_ACROSS_MACHINES`、`ENABLE_GPU`。
+//!    导入那几方的库时只点这一趟要的块（流水线是计划里不归本机的块，给定状态表是判到的、归别的机器的那一张，
+//!    [`crate::verdict_store::VerdictStore::import_the_planned_rows_from`]）；跑法由配置的两个开关定：`ENABLE_ACROSS_MACHINES`、`ENABLE_GPU`。
 //! 5. **显存账**（`singlefs_harness::gpu_memory_ledger`，只防这台机器的显存 OOM、不压并发：用户 2026-10-06 定；只记这个进程借的、
 //!    每次借现读卡上此刻的显存：用户 2026-10-07 定第 10t 批代码轮判决岔路 G1「二」，口径在那份文件头）：这个进程起过 GPU 用例共用的卡池
 //!    （[`GpuCardsSource::TheCardPoolOfThisProcess`]）时，取卡照池里的卡、不登记也不起设备，借到哪张卡才在哪张上起（第 10t 批代码轮判决 K2-3）；
@@ -7705,15 +7706,85 @@
     let states_skipped_by_ignore = states_skipped_by_ignore(flow, &plan);
     let mut imported_blocks = 0u64;
     if let Some(list) = &import_list {
+        // 按这一趟的计划导：只点名这条流里不归本机这一份的块（别的机器判的就在那一份里），每块连同两种判器与挂在这个点上的观察者的结论版本、
+        // 两种核对代码的核对行，路径取计划里每个点的；对方库里没有的跳过。导入的读写随那一份的块数线性，不随库里累计的别的流、别的版本的行涨
+        #[cfg(feature = "gpu")]
+        let verifier_digests = [
+            FactsVerifierOn::Cpu.verifier_digest(),
+            crate::crash_verify_gpu::gpu_verifier_digest(),
+        ];
+        #[cfg(not(feature = "gpu"))]
+        let verifier_digests = [FactsVerifierOn::Cpu.verifier_digest()];
+        let mut planned = crate::verdict_store::PlannedImportKeys {
+            conclusions: Vec::new(),
+            paths: plan
+                .points
+                .iter()
+                .map(|point| point.path_number.clone())
+                .collect(),
+            verifier_blocks: Vec::new(),
+        };
+        let mut block_sequence = 0u64;
+        for (point_index, point) in plan.points.iter().enumerate() {
+            let version_of_the_point = |version_of_each_point: &[ConclusionVersion]| {
+                *version_of_each_point.get(point_index).expect(
+                    "每个点各一把版本、与计划的点同序：ConclusionVersionsOfThePlan::of_the_plan 照计划的点建",
+                )
+            };
+            let mut conclusions_of_the_point = vec![
+                (
+                    JudgedBy::GpuJudge.conclusion_kind(),
+                    version_of_the_point(&conclusion_versions.gpu_judge_of_each_point),
+                ),
+                (
+                    JudgedBy::CpuJudge.conclusion_kind(),
+                    version_of_the_point(&conclusion_versions.cpu_judge_of_each_point),
+                ),
+            ];
+            for (observer, observer_version) in observers.iter().zip(&observer_versions) {
+                if observer
+                    .nodes
+                    .iter()
+                    .any(|node| node.holds_the_point(point))
+                {
+                    conclusions_of_the_point.push((
+                        ConclusionKind::Observer {
+                            observer_name: observer.observer_name.clone(),
+                        },
+                        *observer_version,
+                    ));
+                }
+            }
+            for local in point.blocks() {
+                let this_block = block_sequence;
+                block_sequence += 1;
+                if selected.contains(&this_block) {
+                    continue;
+                }
+                let key = point.block_key(&local);
+                for (kind, conclusion_version) in &conclusions_of_the_point {
+                    planned
+                        .conclusions
+                        .push((key.clone(), kind.clone(), *conclusion_version));
+                }
+                for verifier_digest in &verifier_digests {
+                    planned
+                        .verifier_blocks
+                        .push((key.clone(), *verifier_digest));
+                }
+            }
+        }
         for directory in list.split(':').filter(|directory| !directory.is_empty()) {
             let (other, _) = VerdictStore::open_existing(&path_relative_to_repository_root(
                 repository_root,
                 directory,
             ))
             .map_err(|error| format!("开不了要导入的库 {directory}：{error:?}"))?;
-            imported_blocks += store
-                .import_from(&other)
+            let tally = store
+                .import_the_planned_rows_from(&other, &planned)
                 .map_err(|error| format!("导入 {directory} 失败：{error:?}"))?;
+            imported_blocks +=
+                tally.conclusion_blocks_written + tally.conclusion_blocks_already_identical;
         }
     }
     let seconds_planning_and_recording = planning_started.elapsed().as_secs_f64();
@@ -8298,7 +8369,8 @@
     /// 两台分片时归别的机器判、这一趟库里又没有的表（第三轮正文 H5「配了双机就两台分片」）：这一趟交回的是占位的结论，不作数，
     /// 要等那一台判完、库取回来导入之后再跑一趟（[`GivenStatesRun::left_tables_to_the_other_hosts`]）。
     pub tables_left_to_the_other_hosts: u64,
-    /// 开跑时从别的机器取回的库导入了几块（`SINGLEFS_CRASH_AMPLIFICATION_IMPORT`）。
+    /// 判到归别的机器的表时，从别的机器取回的库（`SINGLEFS_CRASH_AMPLIFICATION_IMPORT`）点名导入了几块结论：一张表一种判器一块，
+    /// 写下的加本库已有且相同的，对方库里没有的不算（[`GivenStatesEngine::import_the_table_from_the_other_hosts`]）。
     pub imported_blocks: u64,
 }
 
@@ -8355,7 +8427,8 @@
 
 /// 给定状态表交崩溃点重放的引擎判（崩溃点重放合并第三轮判决 J3、J5、J6、J8）：照本机多机配置的 GPU 开关选路（没配走 CPU 判器、配了走 GPU 判器，
 /// 卡用不上、借不到显存都报错、不退回 CPU）；两台分片照流水线那一侧的做法（`SINGLEFS_CRASH_AMPLIFICATION_SHARE` 领哪一份、
-/// `…_HOST_WEIGHTS` 权重，一张表一块、按块键的输入指纹归一台；`…_IMPORT` 开跑时导入别的机器取回的库）；叫停照流水线那一侧的叫停层
+/// `…_HOST_WEIGHTS` 权重，一张表一块、按块键的输入指纹归一台；`…_IMPORT` 是别的机器取回的库，开跑时打开，判到归别的机器的表时
+/// 从那几份库里点名导入这一张表的两种判器结论，[`GivenStatesEngine::import_the_table_from_the_other_hosts`]）；叫停照流水线那一侧的叫停层
 /// （[`stop_is_requested_by_the_environment`]，见过一次就记住，崩溃注入照它不再领新的一片）；配了 GPU 时 CPU 判器同一批状态也判、逐状态比红项与恢复字（两遍各自落到的根、结局、失败点；红项是池级 checker 违例逐条、
 /// 记录核对器两条；两遍 oracle 的位两侧都不判不比），比不上判红；结论按（块键，结论种类，版本）存进判定库，一张表一块（H8「判定按段存」），
 /// 同一把版本下判过的表照库里的复用。GPU 判不了的表（建不出表：掩码装不下、落点版本太多……）照 H6 记一条不一致、判红，这几个状态的结论取 CPU 判器的
@@ -8381,8 +8454,9 @@
     gpu_cards_taken_from: GpuCardsSource,
     share: HostShare,
     host_shares: WeightedShares,
-    /// 开跑时导入过别的机器取回的库：归别的机器判的表库里没有就判红（对账不为空），不再交占位的结论。
-    imported_the_other_hosts: bool,
+    /// 别的机器取回的库（`SINGLEFS_CRASH_AMPLIFICATION_IMPORT`，开跑时打开、这个进程里同一个目录共用一份）：不是空的时，归别的机器判的表
+    /// 先从这几份库点名导入，导入之后库里还没有就判红（对账不为空），不再交占位的结论。
+    libraries_to_import_from: Vec<GivenStatesLibraryOpenInThisProcess>,
     stop_request: StopRequestRememberedInThisRun<'static>,
     /// 崩溃注入问叫停时这一趟答过「叫停了」（之后它不再领新的一片）：收尾照它报 `stopped_early`，不在收尾那一刻再读一次叫停文件。
     answered_a_stop_request: std::sync::atomic::AtomicBool,
@@ -8424,14 +8498,30 @@
     store: Option<std::sync::Arc<std::sync::Mutex<VerdictStore>>>,
 }
 
+/// 引擎开的一份库是哪一种：本机这一份（不在就建、旧布局拒绝），还是别的机器取回、要从里面导入的那一份（不在报错、不建，布局不挑：
+/// 按计划导入自己认这一套之前的库，[`VerdictStore::import_the_planned_rows_from`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum GivenStatesLibraryRole {
+    ThisHostsLibrary,
+    LibraryToImportFrom,
+}
+
 impl GivenStatesLibraryOpenInThisProcess {
     /// 开 `library_directory` 那一份库，这个进程里已经开着就共用。
     ///
     /// # Errors
-    /// 建不了上层目录；库在而开不了（按结论种类存之前的旧库拒绝，不迁移）；库不在而建不了。
-    fn open_or_share(library_directory: &Path) -> Result<Self, String> {
+    /// 本机这一份：建不了上层目录；库在而开不了（按结论种类存之前的旧库拒绝，不迁移）；库不在而建不了。
+    /// 要导入的那一份：上层目录取不了规范路径（目录不在）；库不在、开不了。
+    fn open_or_share(
+        library_directory: &Path,
+        role: GivenStatesLibraryRole,
+    ) -> Result<Self, String> {
         let parent = library_directory.parent().unwrap_or(Path::new("."));
-        std::fs::create_dir_all(parent).map_err(|error| format!("建不了库的上层目录：{error}"))?;
+        match role {
+            GivenStatesLibraryRole::ThisHostsLibrary => std::fs::create_dir_all(parent)
+                .map_err(|error| format!("建不了库的上层目录：{error}"))?,
+            GivenStatesLibraryRole::LibraryToImportFrom => {}
+        }
         let key = std::fs::canonicalize(parent)
             .map_err(|error| {
                 format!(
@@ -8449,18 +8539,34 @@
                 store: Some(store),
             });
         }
-        let store = if library_directory.join("CURRENT").exists() {
-            VerdictStore::open_existing_with_conclusion_kinds(library_directory)
-                .map(|(store, _)| store)
-                .map_err(|error| {
-                    format!(
-                        "开不了库 {}（按结论种类存之前的旧库拒绝，不迁移：换一个新目录建库）：{error:?}",
-                        library_directory.display()
-                    )
+        let store = match role {
+            GivenStatesLibraryRole::ThisHostsLibrary
+                if library_directory.join("CURRENT").exists() =>
+            {
+                VerdictStore::open_existing_with_conclusion_kinds(library_directory)
+                    .map(|(store, _)| store)
+                    .map_err(|error| {
+                        format!(
+                            "开不了库 {}（按结论种类存之前的旧库拒绝，不迁移：换一个新目录建库）：{error:?}",
+                            library_directory.display()
+                        )
+                    })?
+            }
+            GivenStatesLibraryRole::ThisHostsLibrary => {
+                VerdictStore::create_empty(library_directory).map_err(|error| {
+                    format!("建不了库 {}：{error:?}", library_directory.display())
                 })?
-        } else {
-            VerdictStore::create_empty(library_directory)
-                .map_err(|error| format!("建不了库 {}：{error:?}", library_directory.display()))?
+            }
+            GivenStatesLibraryRole::LibraryToImportFrom => {
+                VerdictStore::open_existing(library_directory)
+                    .map(|(store, _)| store)
+                    .map_err(|error| {
+                        format!(
+                            "开不了要导入的库 {}：{error:?}",
+                            library_directory.display()
+                        )
+                    })?
+            }
         };
         let store = std::sync::Arc::new(std::sync::Mutex::new(store));
         open_libraries.insert(key.clone(), std::sync::Arc::downgrade(&store));
@@ -8535,7 +8641,8 @@
     /// # Errors
     /// 多机配置读不了或写坏了；配置开了 GPU 而这份二进制没有 `gpu` 特性、或这一趟上不了卡（卡用不上不退回 CPU）；结论的版本与编译时对不上
     /// （打 [`CONCLUSION_VERSION_NOT_WRITTEN_LINE_PREFIX`] 那一行，整趟不写库）；库目录落在开机会清空的地方、库打不开；分片与权重写法错
-    /// （[`host_share_and_weights_from_the_environment`]）；要导入的库打不开、导入失败。
+    /// （[`host_share_and_weights_from_the_environment`]）；要导入的库打不开、就是本机这一份库（导入失败在判到那张表时报，
+    /// [`GivenStatesEngine::import_the_table_from_the_other_hosts`]）。
     pub fn start_from_environment_putting_a_fault_on_the_gpu_side(
         item: GivenStatesItem,
         toolchain: &Layer0ToolchainIdentity,
@@ -8626,22 +8733,27 @@
         if !delete_the_library_after {
             refuse_a_library_that_a_reboot_would_erase(&library_directory)?;
         }
-        let library = GivenStatesLibraryOpenInThisProcess::open_or_share(&library_directory)?;
-        // 别的机器判完、拷回来的库（同流水线那一侧的 SINGLEFS_CRASH_AMPLIFICATION_IMPORT）：判之前导入，归那一台的表照库里的读
-        let mut imported_the_other_hosts = false;
-        let mut tally = GivenStatesTally::default();
+        let library = GivenStatesLibraryOpenInThisProcess::open_or_share(
+            &library_directory,
+            GivenStatesLibraryRole::ThisHostsLibrary,
+        )?;
+        // 别的机器判完、拷回来的库（同流水线那一侧的 SINGLEFS_CRASH_AMPLIFICATION_IMPORT）：开跑时只打开，不整库导入；判到归那一台的表时
+        // 才点名导入那一张（这一趟要判哪几张表开跑时不知道，崩溃注入跑到哪张问哪张）
+        let mut libraries_to_import_from = Vec::new();
         if let Ok(list) = std::env::var("SINGLEFS_CRASH_AMPLIFICATION_IMPORT") {
             for directory in list.split(':').filter(|directory| !directory.is_empty()) {
-                let (other, _) = VerdictStore::open_existing(&path_relative_to_repository_root(
-                    repository_root,
-                    directory,
-                ))
-                .map_err(|error| format!("开不了要导入的库 {directory}：{error:?}"))?;
-                tally.imported_blocks += library
-                    .lock()
-                    .import_from(&other)
-                    .map_err(|error| format!("导入 {directory} 失败：{error:?}"))?;
-                imported_the_other_hosts = true;
+                let other = GivenStatesLibraryOpenInThisProcess::open_or_share(
+                    &path_relative_to_repository_root(repository_root, directory),
+                    GivenStatesLibraryRole::LibraryToImportFrom,
+                )?;
+                // 同一份库在这个进程里只开一次、几个引擎共用：要导入的就是本机这一份时，导入那一刻两把锁是同一把，拒绝（整库导入时开第二次撞 RocksDB 的锁，也是开跑时报错）
+                if other.key == library.key {
+                    return Err(format!(
+                        "要导入的库 {directory} 就是本机这一份库 {}",
+                        library_directory.display()
+                    ));
+                }
+                libraries_to_import_from.push(other);
             }
         }
         let (cards, gpu_cards_taken_from) = if routes.crash_judge == ComputedOn::Gpu {
@@ -8665,14 +8777,14 @@
             gpu_cards_taken_from,
             share,
             host_shares: WeightedShares::new(host_weights),
-            imported_the_other_hosts,
+            libraries_to_import_from,
             stop_request: StopRequestRememberedInThisRun::new(
                 &stop_is_requested_by_the_environment,
             ),
             answered_a_stop_request: std::sync::atomic::AtomicBool::new(false),
             fault_put_on_the_gpu_side: fault,
             next_card: std::sync::atomic::AtomicUsize::new(0),
-            tally: std::sync::Mutex::new(tally),
+            tally: std::sync::Mutex::new(GivenStatesTally::default()),
         })
     }
 
@@ -8686,6 +8798,7 @@
         let stopped_early = self
             .answered_a_stop_request
             .load(std::sync::atomic::Ordering::SeqCst);
+        drop(self.libraries_to_import_from);
         drop(self.library);
         if self.delete_the_library_after {
             let _ = std::fs::remove_dir_all(&self.library_directory);
@@ -8759,6 +8872,48 @@
         self.library.lock()
     }
 
+    /// 一张归别的机器判的表：从开跑时打开的那几份别的机器取回的库里点名导入它的两种判器结论（GPU 判器、CPU 判器，各在这一趟的那一把版本下；
+    /// 对方库里没有的跳过，本库已有且相同的不写，不同的报错），导入的块数记进账的 `imported_blocks`。只点这一张表的块键，导入的读写随这一趟
+    /// 问到的、归别的机器的表数线性，不随那几份库里累计的别的项、别的版本的行涨（[`VerdictStore::import_the_planned_rows_from`]）。
+    /// 先拿本机库的锁、再逐份拿要导入的库的锁（几个引擎都是这个次序）。
+    ///
+    /// # Errors
+    /// 导入失败：同一（块键，种类，版本）两边结论不同、库读写失败这类。
+    fn import_the_table_from_the_other_hosts(&self, key: &VerdictBlockKey) -> Result<(), String> {
+        let planned = crate::verdict_store::PlannedImportKeys {
+            conclusions: [JudgedBy::GpuJudge, JudgedBy::CpuJudge]
+                .into_iter()
+                .map(|judged_by| {
+                    (
+                        key.clone(),
+                        judged_by.conclusion_kind(),
+                        self.version_of(judged_by),
+                    )
+                })
+                .collect(),
+            paths: Vec::new(),
+            verifier_blocks: Vec::new(),
+        };
+        let mut imported_blocks = 0u64;
+        {
+            let mut store = self.store();
+            for other in &self.libraries_to_import_from {
+                let tally = store
+                    .import_the_planned_rows_from(&other.lock(), &planned)
+                    .map_err(|error| {
+                        format!(
+                            "从 {} 导入给定状态表的结论失败：{error:?}",
+                            other.key.display()
+                        )
+                    })?;
+                imported_blocks +=
+                    tally.conclusion_blocks_written + tally.conclusion_blocks_already_identical;
+            }
+        }
+        self.note(|tally| tally.imported_blocks += imported_blocks);
+        Ok(())
+    }
+
     /// 一张给定状态表的块键：输入指纹是项（名字、种子与规模）、表名、基线镜像、写表（逐个写的盘、种类、FUA、偏移、长度、内容摘要与段号）、
     /// 接缝与每个持久集合的摘要——表的输入一样才复用；节点名是 `<设点文件>::<项名>::<表名>`；计划哈希写状态数、写数与接缝数；一张表一块。
     ///
@@ -9239,6 +9394,10 @@
             ));
         }
         let key = self.block_key_of_the_table(table)?;
+        // 归别的机器判的表，先从那一台取回的库点名导入这一张（判之前；导过来的照库里的复用），与流水线那一侧只点不归本机的块同一个分法
+        if !self.libraries_to_import_from.is_empty() && !self.owns_the_table(&key) {
+            self.import_the_table_from_the_other_hosts(&key)?;
+        }
         let cross_check_runs = self.cross_check == CrossCheckOfTheGivenStates::Compared;
         let stored = self.stored_conclusions(&key, self.judged_by, state_count)?;
         let cross_check_is_stored = !cross_check_runs
@@ -9253,7 +9412,7 @@
             return Ok(stored);
         }
         if !self.owns_the_table(&key) {
-            if self.imported_the_other_hosts {
+            if !self.libraries_to_import_from.is_empty() {
                 return Err(format!(
                     "{GIVEN_STATES_TABLE_JUDGED_BY_NO_HOST_PREFIX} table={} share={}/{}：这张表归别的机器判，导入了那一台取回的库之后库里还是没有它的结论（那一台没判完，或库没取回全）：对账不为空，判红",
                     table.table_name, self.share.index, self.share.count
diff --git a/crates/singlefs-checker-tier/src/verdict_store.rs b/crates/singlefs-checker-tier/src/verdict_store.rs
--- a/crates/singlefs-checker-tier/src/verdict_store.rs
+++ b/crates/singlefs-checker-tier/src/verdict_store.rs
@@ -40,7 +40,7 @@
 //! [`VerdictStore::open_existing_with_conclusion_kinds`] 打开时拒绝、不迁移：那时的判定块按（块键，两个判器合起来的判法版本）存，
 //! 三种结论的版本都与它不同，迁过来一块也复用不上；换一个新目录建库，就是一趟清空复用的全量。
 
-use std::collections::{BTreeMap, BTreeSet, HashMap};
+use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
 use std::path::{Path, PathBuf};
 
 use crate::crash_identity::{ConclusionKind, ConclusionVersion, ObserverName};
@@ -930,6 +930,114 @@
     })
 }
 
+/// 一块一种结论在库里的全部行（结论块、违例、计数向量、块属性，给了恢复字的连同恢复字）放进一批。存块与按计划导入都经它，行的写法只在这一处。
+fn put_conclusion_rows_into_the_batch(
+    batch: &mut rocksdb::WriteBatch,
+    block_key: &VerdictBlockKey,
+    kind: &ConclusionKind,
+    conclusion_version: &ConclusionVersion,
+    conclusion: &StoredConclusionBlock,
+) {
+    batch.put(
+        VerdictStoreKey::ConclusionBlock(block_key, kind, conclusion_version)
+            .to_key_bytes()
+            .0,
+        conclusion.block.to_block_value_bytes(),
+    );
+    for violation in &conclusion.violations {
+        batch.put(
+            VerdictStoreKey::ConclusionViolation(
+                block_key,
+                kind,
+                conclusion_version,
+                violation.offset_in_block,
+            )
+            .to_key_bytes()
+            .0,
+            vec![violation.verdict_vector_number.as_u8()],
+        );
+    }
+    batch.put(
+        VerdictStoreKey::ConclusionCountVector(block_key, kind, conclusion_version)
+            .to_key_bytes()
+            .0,
+        count_vector_bytes(&conclusion.count_vector),
+    );
+    batch.put(
+        VerdictStoreKey::ConclusionBlockAttributes(block_key, kind, conclusion_version)
+            .to_key_bytes()
+            .0,
+        conclusion.judged_on.to_attribute_bytes(),
+    );
+    if let Some(recovery_words) = &conclusion.recovery_words {
+        batch.put(
+            VerdictStoreKey::RecoveryWords(block_key, kind, conclusion_version)
+                .to_key_bytes()
+                .0,
+            recovery_words_value_bytes(recovery_words),
+        );
+    }
+}
+
+/// 要存的结论与库里同键那一份比出来的结局（不同的两种是错误，见 [`VerdictStore::stored_conclusion_compared_with`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum StoredConclusionComparison {
+    NotStored,
+    StoredIdentically,
+}
+
+/// 按计划导入点名的键（[`VerdictStore::import_the_planned_rows_from`]）：这一趟这条流要从别的机器取回的几样，别的一行都不看。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct PlannedImportKeys {
+    /// 结论：一项是一块一种结论一把版本。
+    pub conclusions: Vec<(VerdictBlockKey, ConclusionKind, ConclusionVersion)>,
+    /// 路径索引里的路径。
+    pub paths: Vec<String>,
+    /// 核对表的行：块键连同核对代码摘要。导过来的核对行点名的事实表（值的头 32 字节）随它们一起导。
+    pub verifier_blocks: Vec<(VerdictBlockKey, [u8; SHA256_DIGEST_BYTES])>,
+}
+
+/// 按计划导入的账：点名的结论各落在哪一格（写了、本库已有且相同而跳过、对方库里没有），路径、核对行与事实行各写了几行，
+/// 一共写了几个键值行（连同新登记的判定向量那两张表）、同步写了几批。
+#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
+pub struct PlannedImportTally {
+    pub conclusion_blocks_written: u64,
+    pub conclusion_blocks_already_identical: u64,
+    pub conclusion_blocks_missing_in_the_other_library: u64,
+    pub paths_written: u64,
+    pub verifier_blocks_written: u64,
+    pub flow_facts_written: u64,
+    pub rows_written: u64,
+    pub synced_writes: u64,
+}
+
+impl PlannedImportTally {
+    /// 记一批同步写：写下了几行。
+    fn count_a_written_batch(&mut self, rows_written_in_the_batch: u64) {
+        self.rows_written += rows_written_in_the_batch;
+        self.synced_writes += 1;
+    }
+}
+
+/// 按计划导入时一批攒到这么多字节就同步写一次（一块一种结论的行不拆开，块内原子）：一块 GPU 结论连恢复字约 2 MiB
+/// （65536 个状态每个 1 字节编号加 32 字节恢复字），一批装几十块；同步写的次数随写下的字节数线性，攒批的内存有界。数是推的，没量过。
+const PLANNED_IMPORT_BATCH_BYTES: usize = 64 << 20;
+
+/// 按计划导入时正在攒的一批：要写的行，与这一批里新登记的判定向量（按登记次序；这一批写下去之后才进库的内存表）。
+struct PlannedImportBatch {
+    batch: rocksdb::WriteBatch,
+    verdict_vectors_registered_in_the_batch: Vec<EncodedVerdictVector>,
+}
+
+impl PlannedImportBatch {
+    fn empty() -> Self {
+        Self {
+            batch: rocksdb::WriteBatch::default(),
+            verdict_vectors_registered_in_the_batch: Vec::new(),
+        }
+    }
+}
+
 /// 一个打开着的判定库。同一时刻一个库只许一个进程开（RocksDB 的 LOCK 文件），进程里改库的方法都要 `&mut self`，
 /// 所以「先读后写」的两步之间没有别的写者。
 pub struct VerdictStore {
@@ -1268,13 +1376,20 @@
             .map_err(VerdictStoreError::RocksDatabase)
     }
 
-    /// 路径索引登一条：这条路径上的节点是这个身份。同一路径再登、身份相同不算错；身份不同也照写（同名不同数据的分叉在路径这一级并成一段，
-    /// 最后写的那个留在索引里；两边都在块表里，一个不丢）。
+    /// 路径索引登一条：这条路径上的节点是这个身份。同一路径再登、身份相同不算错，什么都不写（与 [`Self::record_block`] 一样先比再写）；
+    /// 身份不同也照写（同名不同数据的分叉在路径这一级并成一段，最后写的那个留在索引里；两边都在块表里，一个不丢）。
     ///
     /// # Errors
-    /// RocksDB 写失败。
+    /// RocksDB 读写失败。
     pub fn record_path(&mut self, path: &str, identity: &str) -> Result<(), VerdictStoreError> {
         let key_bytes = VerdictStoreKey::Path(path).to_key_bytes();
+        let stored_identity = self
+            .database
+            .get(&key_bytes.0)
+            .map_err(VerdictStoreError::RocksDatabase)?;
+        if stored_identity.as_deref() == Some(identity.as_bytes()) {
+            return Ok(());
+        }
         let mut batch = rocksdb::WriteBatch::default();
         batch.put(&key_bytes.0, identity.as_bytes());
         self.database
@@ -1640,6 +1755,308 @@
         Ok(imported)
     }
 
+    /// 按计划导入（[`PlannedImportKeys`]）：只读点名的键，不扫两边的整张表。每一项先在对方库里读，没有就跳过（第二台没判到的块）；
+    /// 结论按判定向量的内容换成这个库的编号（这个库没登记过的判定向量随行一起登记），再照 [`Self::store_conclusion_block`] 的规矩与本库同键的比：
+    /// 逐项相同跳过、不写，结论不同或块属性不同报错；本库没有的连同新登记的判定向量放进一批。路径、核对行与事实行本库已有且逐字节相同的跳过，
+    /// 不同的照写（与 [`Self::record_path`]、[`Self::store_verifier_block`]、[`Self::store_flow_facts`] 同规矩）。一块一种结论的行在同一批里
+    /// （块内原子），一批攒到 [`PLANNED_IMPORT_BATCH_BYTES`] 字节同步写一次，最后一批收尾时写。中途出错时已写下的批留着、正在攒的那一批不写，
+    /// 重跑同一条命令补齐（写下的都是对方库里有的行，重跑逐项相同就跳过）。
+    ///
+    /// 读写次数（N 是点名的项数，与两边库里别的流、别的版本的行数无关）：每项结论对方库四次点读加一次只扫这一块的违例前缀扫、本库一次点读
+    /// （本库有这一块时再三次点读加一次前缀扫，没有时再一次点读看恢复字旁表）；每条路径、每个核对行两边各一次点读；核对行点名的每份事实
+    /// 两边各一次点读；同步写的批数是写下的字节数除以批长向上取整。每个状态换编号查一张 256 格的表，每种判定向量只按内容查一次。
+    ///
+    /// # Errors
+    /// 点名了结论而这个库是这一套之前的布局；RocksDB 读写失败；存着的行不合布局（违例的块内序号不在块里、核对行不到 32 字节这类）；
+    /// 同一（块键，种类，版本）两边结论不同或块属性不同；判定向量超过 256 种。
+    pub fn import_the_planned_rows_from(
+        &mut self,
+        other: &VerdictStore,
+        planned: &PlannedImportKeys,
+    ) -> Result<PlannedImportTally, VerdictStoreError> {
+        if !planned.conclusions.is_empty() {
+            self.refuse_unless_the_conclusion_kinds_layout()?;
+        }
+        let mut tally = PlannedImportTally::default();
+        let mut pending = PlannedImportBatch::empty();
+        // 对方库的编号换成这个库的编号：一次导入里两边的编号都不变（新登记的照次序接在后面），按对方的编号记一次就够
+        let mut this_number_of_each_other_number: Vec<Option<VerdictVectorNumber>> =
+            vec![None; usize::from(u8::MAX) + 1];
+        let mut conclusion_keys_seen: HashSet<Vec<u8>> = HashSet::new();
+        for (block_key, kind, conclusion_version) in &planned.conclusions {
+            let conclusion_key_bytes =
+                VerdictStoreKey::ConclusionBlock(block_key, kind, conclusion_version)
+                    .to_key_bytes();
+            if !conclusion_keys_seen.insert(conclusion_key_bytes.0.clone()) {
+                continue;
+            }
+            let other_conclusion = match other.layout {
+                VerdictStoreLayout::ConclusionKinds => {
+                    other.read_conclusion_block(block_key, kind, conclusion_version)?
+                }
+                VerdictStoreLayout::BeforeConclusionKinds => None,
+            };
+            let Some(other_conclusion) = other_conclusion else {
+                tally.conclusion_blocks_missing_in_the_other_library += 1;
+                continue;
+            };
+            let requested = self.conclusion_renumbered_for_the_planned_import(
+                other,
+                other_conclusion,
+                &conclusion_key_bytes,
+                &mut this_number_of_each_other_number,
+                &mut pending,
+            )?;
+            match self.stored_conclusion_compared_with(
+                block_key,
+                kind,
+                conclusion_version,
+                &requested,
+            )? {
+                StoredConclusionComparison::StoredIdentically => {
+                    tally.conclusion_blocks_already_identical += 1;
+                }
+                StoredConclusionComparison::NotStored => {
+                    put_conclusion_rows_into_the_batch(
+                        &mut pending.batch,
+                        block_key,
+                        kind,
+                        conclusion_version,
+                        &requested,
+                    );
+                    tally.conclusion_blocks_written += 1;
+                    if let Some(rows) =
+                        self.write_the_planned_import_batch_when_full(&mut pending)?
+                    {
+                        tally.count_a_written_batch(rows);
+                    }
+                }
+            }
+        }
+        let mut paths_seen: HashSet<&str> = HashSet::new();
+        for path in &planned.paths {
+            if !paths_seen.insert(path.as_str()) {
+                continue;
+            }
+            let Some(identity) = other.identity_of_path(path)? else {
+                continue;
+            };
+            if self.identity_of_path(path)?.as_deref() == Some(identity.as_str()) {
+                continue;
+            }
+            pending.batch.put(
+                VerdictStoreKey::Path(path).to_key_bytes().0,
+                identity.as_bytes(),
+            );
+            tally.paths_written += 1;
+        }
+        let mut verifier_keys_seen: HashSet<Vec<u8>> = HashSet::new();
+        let mut facts_digests_named: BTreeSet<[u8; SHA256_DIGEST_BYTES]> = BTreeSet::new();
+        for (block_key, verifier_digest) in &planned.verifier_blocks {
+            let key_bytes =
+                VerdictStoreKey::VerifierBlock(block_key, verifier_digest).to_key_bytes();
+            if !verifier_keys_seen.insert(key_bytes.0.clone()) {
+                continue;
+            }
+            let Some(value_bytes) = other
+                .database
+                .get(&key_bytes.0)
+                .map_err(VerdictStoreError::RocksDatabase)?
+            else {
+                continue;
+            };
+            let facts_digest = value_bytes
+                .get(..SHA256_DIGEST_BYTES)
+                .and_then(|digest_bytes| <[u8; SHA256_DIGEST_BYTES]>::try_from(digest_bytes).ok())
+                .ok_or_else(|| {
+                    stored_bytes_mismatch(
+                        VerdictStoreTable::VerifierVerdicts,
+                        &key_bytes.0,
+                        "核对行的值要以 32 字节事实摘要起头",
+                    )
+                })?;
+            facts_digests_named.insert(facts_digest);
+            let stored_value = self
+                .database
+                .get(&key_bytes.0)
+                .map_err(VerdictStoreError::RocksDatabase)?;
+            if stored_value.as_deref() == Some(value_bytes.as_slice()) {
+                continue;
+            }
+            pending.batch.put(&key_bytes.0, &value_bytes);
+            tally.verifier_blocks_written += 1;
+            if let Some(rows) = self.write_the_planned_import_batch_when_full(&mut pending)? {
+                tally.count_a_written_batch(rows);
+            }
+        }
+        for facts_digest in &facts_digests_named {
+            let Some(facts_bytes) = other.read_flow_facts(facts_digest)? else {
+                continue;
+            };
+            if self.read_flow_facts(facts_digest)?.as_deref() == Some(facts_bytes.as_slice()) {
+                continue;
+            }
+            pending.batch.put(
+                VerdictStoreKey::FlowFacts(facts_digest).to_key_bytes().0,
+                &facts_bytes,
+            );
+            tally.flow_facts_written += 1;
+            if let Some(rows) = self.write_the_planned_import_batch_when_full(&mut pending)? {
+                tally.count_a_written_batch(rows);
+            }
+        }
+        if !pending.batch.is_empty() {
+            tally.count_a_written_batch(self.write_the_planned_import_batch(pending)?);
+        }
+        Ok(tally)
+    }
+
+    /// 攒着的那一批到了 [`PLANNED_IMPORT_BATCH_BYTES`] 字节就同步写下去、换一批空的接着攒；交回写下的行数，没到交回 None。
+    ///
+    /// # Errors
+    /// RocksDB 写失败。
+    fn write_the_planned_import_batch_when_full(
+        &mut self,
+        pending: &mut PlannedImportBatch,
+    ) -> Result<Option<u64>, VerdictStoreError> {
+        if pending.batch.size_in_bytes() < PLANNED_IMPORT_BATCH_BYTES {
+            return Ok(None);
+        }
+        let full_batch = std::mem::replace(pending, PlannedImportBatch::empty());
+        self.write_the_planned_import_batch(full_batch).map(Some)
+    }
+
+    /// 对方库读回的一块一种结论换成这个库的编号：每种判定向量按内容找这个库的编号，这个库没有的在攒着的那一批里登记（编号接在已登记的
+    /// 与这一批已登记的后面）；违例的编号取换过编号的块里那个状态的（同 [`Self::store_conclusion_block`]）。
+    ///
+    /// # Errors
+    /// 判定向量超过 256 种（`MoreThan256VerdictVectorsNotDecidedInTheFirstVersion`，这一批没写）；对方库的违例的块内序号不在块里。
+    fn conclusion_renumbered_for_the_planned_import(
+        &self,
+        other: &VerdictStore,
+        other_conclusion: StoredConclusionBlock,
+        conclusion_key_bytes: &VerdictStoreKeyBytes,
+        this_number_of_each_other_number: &mut [Option<VerdictVectorNumber>],
+        pending: &mut PlannedImportBatch,
+    ) -> Result<StoredConclusionBlock, VerdictStoreError> {
+        let other_numbers = other_conclusion.block.verdict_vector_number_of_each_state();
+        let mut this_numbers = Vec::with_capacity(other_numbers.len());
+        for other_number in other_numbers {
+            let slot = this_number_of_each_other_number
+                .get_mut(usize::from(other_number.as_u8()))
+                .expect("换编号的表有 256 格，编号是一个字节");
+            let this_number = match *slot {
+                Some(this_number) => this_number,
+                None => {
+                    let vector = other.verdict_vector_of_number(*other_number).expect(
+                        "对方库读回的结论块里的编号在它自己的库里登记过（read_conclusion_block 逐个核过）",
+                    );
+                    let this_number =
+                        self.verdict_vector_number_for_the_planned_import(vector, pending)?;
+                    *slot = Some(this_number);
+                    this_number
+                }
+            };
+            this_numbers.push(this_number);
+        }
+        let block = VerdictBlock::new(this_numbers).expect("读回的结论块至少一个状态");
+        let mut violations = Vec::with_capacity(other_conclusion.violations.len());
+        for violation in &other_conclusion.violations {
+            let Some(verdict_vector_number) =
+                block.verdict_vector_number_at(violation.offset_in_block)
+            else {
+                return Err(stored_bytes_mismatch(
+                    VerdictStoreTable::ConclusionViolations,
+                    &conclusion_key_bytes.0,
+                    "对方库这一块这一种结论有一处违例的块内序号不在块里",
+                ));
+            };
+            violations.push(StoredViolation {
+                offset_in_block: violation.offset_in_block,
+                verdict_vector_number,
+            });
+        }
+        Ok(StoredConclusionBlock {
+            block,
+            violations,
+            count_vector: other_conclusion.count_vector,
+            judged_on: other_conclusion.judged_on,
+            recovery_words: other_conclusion.recovery_words,
+        })
+    }
+
+    /// 一种判定向量在这个库里的编号：登记过的照旧；这一批里已登记的照它在批里的次序；都没有的放进这一批登记（两张表各一行），编号接在后面。
+    ///
+    /// # Errors
+    /// 加上这一种就超过 256 种（`MoreThan256VerdictVectorsNotDecidedInTheFirstVersion`，这一批没写）。
+    fn verdict_vector_number_for_the_planned_import(
+        &self,
+        verdict_vector: &EncodedVerdictVector,
+        pending: &mut PlannedImportBatch,
+    ) -> Result<VerdictVectorNumber, VerdictStoreError> {
+        if let Some(registered_number) =
+            self.verdict_vector_number_by_content.get(&verdict_vector.0)
+        {
+            return Ok(*registered_number);
+        }
+        let registered_count = self.verdict_vector_by_number.len();
+        if let Some(position_in_the_batch) = pending
+            .verdict_vectors_registered_in_the_batch
+            .iter()
+            .position(|registered_in_the_batch| registered_in_the_batch == verdict_vector)
+        {
+            let number = u8::try_from(registered_count + position_in_the_batch)
+                .expect("放进这一批时核过编号装得进一个字节");
+            return Ok(VerdictVectorNumber(number));
+        }
+        let Ok(next_number) =
+            u8::try_from(registered_count + pending.verdict_vectors_registered_in_the_batch.len())
+        else {
+            return Err(VerdictStoreError::MoreThan256VerdictVectorsNotDecidedInTheFirstVersion);
+        };
+        let number = VerdictVectorNumber(next_number);
+        pending.batch.put(
+            VerdictStoreKey::VerdictVectorNumberByContent(verdict_vector)
+                .to_key_bytes()
+                .0,
+            [number.0],
+        );
+        pending.batch.put(
+            VerdictStoreKey::VerdictVectorContentByNumber(number)
+                .to_key_bytes()
+                .0,
+            &verdict_vector.0,
+        );
+        pending
+            .verdict_vectors_registered_in_the_batch
+            .push(verdict_vector.clone());
+        Ok(number)
+    }
+
+    /// 把攒好的一批同步写下去，写下去之后这一批登记的判定向量才进内存表；交回写下的键值行数。
+    ///
+    /// # Errors
+    /// RocksDB 写失败（这一批一行都没进库，内存表也不动）。
+    fn write_the_planned_import_batch(
+        &mut self,
+        pending: PlannedImportBatch,
+    ) -> Result<u64, VerdictStoreError> {
+        let rows = u64::try_from(pending.batch.len()).expect("一批的行数装得进 u64");
+        self.database
+            .write_opt(pending.batch, &synced_write_options())
+            .map_err(VerdictStoreError::RocksDatabase)?;
+        for verdict_vector in pending.verdict_vectors_registered_in_the_batch {
+            let number = VerdictVectorNumber(
+                u8::try_from(self.verdict_vector_by_number.len())
+                    .expect("放进这一批时核过编号装得进一个字节"),
+            );
+            self.verdict_vector_number_by_content
+                .insert(verdict_vector.0.clone(), number);
+            self.verdict_vector_by_number.push(verdict_vector);
+        }
+        Ok(rows)
+    }
+
     /// 库的布局：建库时写的布局行（[`VerdictStore::create_empty`] 写），这一套之前的库没有这一行。
     #[must_use]
     pub fn layout(&self) -> VerdictStoreLayout {
@@ -1711,6 +2128,45 @@
             judged_on: record.judged_on.clone(),
             recovery_words: record.recovery_words.clone(),
         };
+        match self.stored_conclusion_compared_with(
+            block_key,
+            kind,
+            conclusion_version,
+            &requested,
+        )? {
+            StoredConclusionComparison::StoredIdentically => {
+                return Ok(VerdictBlockStoring::AlreadyStoredIdentically)
+            }
+            StoredConclusionComparison::NotStored => {}
+        }
+        let mut batch = rocksdb::WriteBatch::default();
+        put_conclusion_rows_into_the_batch(
+            &mut batch,
+            block_key,
+            kind,
+            conclusion_version,
+            &requested,
+        );
+        self.database
+            .write_opt(batch, &synced_write_options())
+            .map_err(VerdictStoreError::RocksDatabase)?;
+        Ok(VerdictBlockStoring::WrittenAndSynced)
+    }
+
+    /// 要存的一块一种结论（编号已是这个库的）与库里这个（块键，种类，版本）下存着的比：没存过（恢复字旁表也没有这一行）交回
+    /// `NotStored`；块、违例、计数向量、恢复字与块属性都相同交回 `StoredIdentically`。存块与按计划导入都经它，规矩只在这一处。
+    ///
+    /// # Errors
+    /// 结论不同（`DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion`）；结论相同、块属性不同
+    /// （`DifferentBlockAttributesUnderAStoredConclusionNotDecidedInTheFirstVersion`）；恢复字旁表有这一行而结论块表没有；
+    /// 已存的行读回来不合布局；RocksDB 读失败。
+    fn stored_conclusion_compared_with(
+        &self,
+        block_key: &VerdictBlockKey,
+        kind: &ConclusionKind,
+        conclusion_version: &ConclusionVersion,
+        requested: &StoredConclusionBlock,
+    ) -> Result<StoredConclusionComparison, VerdictStoreError> {
         if let Some(stored) = self.read_conclusion_block(block_key, kind, conclusion_version)? {
             let is_the_same_conclusion = stored.block == requested.block
                 && stored.violations == requested.violations
@@ -1730,7 +2186,7 @@
                     kind: kind.clone(),
                 });
             }
-            return Ok(VerdictBlockStoring::AlreadyStoredIdentically);
+            return Ok(StoredConclusionComparison::StoredIdentically);
         }
         let recovery_words_key =
             VerdictStoreKey::RecoveryWords(block_key, kind, conclusion_version).to_key_bytes();
@@ -1745,48 +2201,7 @@
                 "恢复字旁表有这一块这一种结论这一版本，结论块表没有",
             ));
         }
-        let mut batch = rocksdb::WriteBatch::default();
-        batch.put(
-            VerdictStoreKey::ConclusionBlock(block_key, kind, conclusion_version)
-                .to_key_bytes()
-                .0,
-            requested.block.to_block_value_bytes(),
-        );
-        for violation in &requested.violations {
-            batch.put(
-                VerdictStoreKey::ConclusionViolation(
-                    block_key,
-                    kind,
-                    conclusion_version,
-                    violation.offset_in_block,
-                )
-                .to_key_bytes()
-                .0,
-                vec![violation.verdict_vector_number.as_u8()],
-            );
-        }
-        batch.put(
-            VerdictStoreKey::ConclusionCountVector(block_key, kind, conclusion_version)
-                .to_key_bytes()
-                .0,
-            count_vector_bytes(&requested.count_vector),
-        );
-        batch.put(
-            VerdictStoreKey::ConclusionBlockAttributes(block_key, kind, conclusion_version)
-                .to_key_bytes()
-                .0,
-            requested.judged_on.to_attribute_bytes(),
-        );
-        if let Some(recovery_words) = &requested.recovery_words {
-            batch.put(
-                &recovery_words_key.0,
-                recovery_words_value_bytes(recovery_words),
-            );
-        }
-        self.database
-            .write_opt(batch, &synced_write_options())
-            .map_err(VerdictStoreError::RocksDatabase)?;
-        Ok(VerdictBlockStoring::WrittenAndSynced)
+        Ok(StoredConclusionComparison::NotStored)
     }
 
     /// 读回一块一种结论，连同违例、计数向量、块属性与恢复字；这个（块键，种类，版本）没存过时是 `None`（别的种类、别的版本存过也是 `None`）。
@@ -3216,4 +3631,498 @@
         assert_eq!(stored.judged_on, expected.judged_on);
         assert_eq!(stored.recovery_words, expected.recovery_words);
     }
+
+    /// 按计划导入的对拍用的流 F 与另一条流 G 的节点名、F 的块。
+    const FLOW_F_NODE: &str = "flow-f::node-f::0011";
+    const FLOW_G_NODE: &str = "flow-g::node-g::0022";
+    /// 本库 A 里流 F 在一把旧版本下的历史行（判法改过、旧行留着）。
+    const OLD_GPU_CONCLUSION_VERSION: ConclusionVersion =
+        ConclusionVersion([0x54; SHA256_DIGEST_BYTES]);
+    const CPU_VERIFIER_DIGEST: [u8; SHA256_DIGEST_BYTES] = [0x81; SHA256_DIGEST_BYTES];
+    const GPU_VERIFIER_DIGEST: [u8; SHA256_DIGEST_BYTES] = [0x82; SHA256_DIGEST_BYTES];
+    const OLD_FACTS_DIGEST: [u8; SHA256_DIGEST_BYTES] = [0x71; SHA256_DIGEST_BYTES];
+    const FLOW_F_FACTS_DIGEST: [u8; SHA256_DIGEST_BYTES] = [0x72; SHA256_DIGEST_BYTES];
+    const FLOW_G_FACTS_DIGEST: [u8; SHA256_DIGEST_BYTES] = [0x73; SHA256_DIGEST_BYTES];
+    const MOUNT_OBSERVER_NAME: &str = "mount";
+
+    fn flow_f_block(block_index: u64) -> VerdictBlockKey {
+        block_key_named(FLOW_F_NODE, block_index * STATES_PER_FLOW_F_TEST_BLOCK)
+    }
+
+    fn flow_g_block(block_index: u64) -> VerdictBlockKey {
+        block_key_named(FLOW_G_NODE, block_index * STATES_PER_FLOW_F_TEST_BLOCK)
+    }
+
+    /// 对拍里块起点的步长（块里的状态数各块自己定，不超过它）。
+    const STATES_PER_FLOW_F_TEST_BLOCK: u64 = 4;
+
+    fn observer_record(block: VerdictBlock, violating_states: &[u64]) -> ConclusionBlockRecord {
+        let states = u64::try_from(block.verdict_vector_number_of_each_state().len())
+            .expect("测试里的块不长");
+        ConclusionBlockRecord {
+            block,
+            violating_states: violating_states
+                .iter()
+                .copied()
+                .map(StateOffsetInBlock)
+                .collect(),
+            count_vector: count_vector_of(states),
+            judged_on: JudgingDevice::Cpu,
+            recovery_words: None,
+        }
+    }
+
+    fn store_conclusion_in_the_test_library(
+        store: &mut VerdictStore,
+        block_key: &VerdictBlockKey,
+        kind: &ConclusionKind,
+        conclusion_version: &ConclusionVersion,
+        record: &ConclusionBlockRecord,
+    ) {
+        store
+            .store_conclusion_block(block_key, kind, conclusion_version, record)
+            .expect("测试库里存得进结论");
+    }
+
+    /// 本库 A（导入那一趟开跑时本机的库）：流 F 四块都录了，第 0 块是本机这一份、两种判器都判过；第 1 块的 CPU 结论早先导入过一次；
+    /// 第 0、1 块在旧 GPU 版本下有历史行；流 G 第 0 块判过；三条路径；一份旧事实行。同一个目录名之外逐字节相同，两份 A 各调一次。
+    fn build_the_local_library_a(directory: &Path) -> VerdictStore {
+        let mut store = VerdictStore::create_empty(directory).expect("建库");
+        let clean = store
+            .register_verdict_vector(&verdict_vector(b""))
+            .expect("登记");
+        let red = store
+            .register_verdict_vector(&verdict_vector(b"red-f"))
+            .expect("登记");
+        for (block_index, state_count) in [(0, 2), (1, 2), (2, 3), (3, 2)] {
+            store
+                .record_block(&flow_f_block(block_index), state_count)
+                .expect("录入");
+        }
+        store.record_block(&flow_g_block(0), 2).expect("录入");
+        let judged_by_this_host = conclusion_record(block_of(&[clean, red]), &[1]);
+        for (kind, conclusion_version) in [
+            (ConclusionKind::GpuJudge, GPU_CONCLUSION_VERSION),
+            (ConclusionKind::CpuJudge, CPU_CONCLUSION_VERSION),
+        ] {
+            store_conclusion_in_the_test_library(
+                &mut store,
+                &flow_f_block(0),
+                &kind,
+                &conclusion_version,
+                &judged_by_this_host,
+            );
+        }
+        store_conclusion_in_the_test_library(
+            &mut store,
+            &flow_f_block(1),
+            &ConclusionKind::CpuJudge,
+            &CPU_CONCLUSION_VERSION,
+            &conclusion_record(block_of(&[clean, red]), &[1]),
+        );
+        store_conclusion_in_the_test_library(
+            &mut store,
+            &flow_f_block(0),
+            &ConclusionKind::GpuJudge,
+            &OLD_GPU_CONCLUSION_VERSION,
+            &conclusion_record(block_of(&[clean, clean]), &[]),
+        );
+        store_conclusion_in_the_test_library(
+            &mut store,
+            &flow_f_block(1),
+            &ConclusionKind::GpuJudge,
+            &OLD_GPU_CONCLUSION_VERSION,
+            &conclusion_record(block_of(&[red, clean]), &[0]),
+        );
+        store_conclusion_in_the_test_library(
+            &mut store,
+            &flow_g_block(0),
+            &ConclusionKind::GpuJudge,
+            &GPU_CONCLUSION_VERSION,
+            &conclusion_record(block_of(&[clean, clean]), &[]),
+        );
+        for (path, identity) in [
+            ("f.0", "flow-f-identity-0"),
+            ("f.0/f.1", "flow-f-identity-1-before-the-fork"),
+            ("g.0", "flow-g-identity-0"),
+        ] {
+            store.record_path(path, identity).expect("登路径");
+        }
+        store
+            .store_flow_facts(&OLD_FACTS_DIGEST, b"facts of an older extractor")
+            .expect("登事实");
+        store
+    }
+
+    /// 对方库 B（第二台取回的库）：A 的快照加上第二台这一趟判的——流 F 第 1 块的 GPU 结论、第 2 块的 GPU 与观察者结论（用到一种 A 没登记过的
+    /// 判定向量）、这两块在 CPU 核对代码下的核对行与 F 的事实行、F 的两条路径（一条与 A 同路径不同身份，是同名不同数据并成一段的那种）；
+    /// 第 3 块没判到。另一条流 G 第 1 块也判了、核了、登了路径与事实（不属于 F）。
+    fn build_the_other_library_b(directory: &Path) -> VerdictStore {
+        let mut store = build_the_local_library_a(directory);
+        let clean = store
+            .register_verdict_vector(&verdict_vector(b""))
+            .expect("登记");
+        let red = store
+            .register_verdict_vector(&verdict_vector(b"red-f"))
+            .expect("登记");
+        let red_only_in_the_other_library = store
+            .register_verdict_vector(&verdict_vector(b"red-f-judged-on-the-other-host"))
+            .expect("登记");
+        store_conclusion_in_the_test_library(
+            &mut store,
+            &flow_f_block(1),
+            &ConclusionKind::GpuJudge,
+            &GPU_CONCLUSION_VERSION,
+            &conclusion_record(block_of(&[clean, red]), &[1]),
+        );
+        let block_two = block_of(&[red_only_in_the_other_library, clean, clean]);
+        store_conclusion_in_the_test_library(
+            &mut store,
+            &flow_f_block(2),
+            &ConclusionKind::GpuJudge,
+            &GPU_CONCLUSION_VERSION,
+            &conclusion_record(block_two.clone(), &[0]),
+        );
+        store_conclusion_in_the_test_library(
+            &mut store,
+            &flow_f_block(2),
+            &observer_kind(MOUNT_OBSERVER_NAME),
+            &OBSERVER_CONCLUSION_VERSION,
+            &observer_record(block_two, &[0]),
+        );
+        store_conclusion_in_the_test_library(
+            &mut store,
+            &flow_g_block(1),
+            &ConclusionKind::GpuJudge,
+            &GPU_CONCLUSION_VERSION,
+            &conclusion_record(block_of(&[red, clean]), &[0]),
+        );
+        for (block_key, facts_digest, bits) in [
+            (flow_f_block(1), FLOW_F_FACTS_DIGEST, vec![0, 1]),
+            (flow_f_block(2), FLOW_F_FACTS_DIGEST, vec![1, 0, 0]),
+            (flow_g_block(1), FLOW_G_FACTS_DIGEST, vec![1, 0]),
+        ] {
+            store
+                .store_verifier_block(&block_key, &CPU_VERIFIER_DIGEST, &facts_digest, &bits)
+                .expect("写核对行");
+        }
+        store
+            .store_flow_facts(&FLOW_F_FACTS_DIGEST, b"facts of flow f")
+            .expect("登事实");
+        store
+            .store_flow_facts(&FLOW_G_FACTS_DIGEST, b"facts of flow g")
+            .expect("登事实");
+        for (path, identity) in [
+            ("f.0/f.1", "flow-f-identity-1-after-the-fork"),
+            ("f.0/f.1/f.2", "flow-f-identity-2"),
+            ("g.0/g.1", "flow-g-identity-1"),
+        ] {
+            store.record_path(path, identity).expect("登路径");
+        }
+        store
+    }
+
+    /// 流水线导入那一趟照计划点名的键（`crash_amplification` 导入那一段的形态）：流 F 里不归本机的第 1–3 块，每块两种判器各一把当前版本、
+    /// 观察者一把，两种核对代码各一行；路径是计划里 F 的每个点。
+    fn planned_keys_of_flow_f_beyond_this_host_s_share() -> PlannedImportKeys {
+        let mut planned = PlannedImportKeys {
+            conclusions: Vec::new(),
+            paths: vec![
+                "f.0".to_string(),
+                "f.0/f.1".to_string(),
+                "f.0/f.1/f.2".to_string(),
+            ],
+            verifier_blocks: Vec::new(),
+        };
+        for block_index in 1..4 {
+            for (kind, conclusion_version) in [
+                (ConclusionKind::GpuJudge, GPU_CONCLUSION_VERSION),
+                (ConclusionKind::CpuJudge, CPU_CONCLUSION_VERSION),
+                (
+                    observer_kind(MOUNT_OBSERVER_NAME),
+                    OBSERVER_CONCLUSION_VERSION,
+                ),
+            ] {
+                planned
+                    .conclusions
+                    .push((flow_f_block(block_index), kind, conclusion_version));
+            }
+            for verifier_digest in [CPU_VERIFIER_DIGEST, GPU_VERIFIER_DIGEST] {
+                planned
+                    .verifier_blocks
+                    .push((flow_f_block(block_index), verifier_digest));
+            }
+        }
+        planned
+    }
+
+    /// 一块一种结论读回来、换成判定向量的内容（两份库的编号各是各的，按内容比才是「读回相同」）。
+    #[derive(Debug, PartialEq, Eq)]
+    struct ConclusionReadBackByContent {
+        verdict_vector_of_each_state: Vec<Vec<u8>>,
+        violations: Vec<(u64, Vec<u8>)>,
+        count_vector_bytes: Vec<u8>,
+        judged_on: JudgingDevice,
+        recovery_words: Option<Vec<RecoveryWordsOfState>>,
+    }
+
+    fn conclusion_read_back_by_content(
+        store: &VerdictStore,
+        block_key: &VerdictBlockKey,
+        kind: &ConclusionKind,
+        conclusion_version: &ConclusionVersion,
+    ) -> Option<ConclusionReadBackByContent> {
+        let vector_bytes = |number: VerdictVectorNumber| {
+            store
+                .verdict_vector_of_number(number)
+                .expect("读回的编号在这个库里登记过")
+                .0
+                .clone()
+        };
+        store
+            .read_conclusion_block(block_key, kind, conclusion_version)
+            .expect("读结论")
+            .map(|stored| ConclusionReadBackByContent {
+                verdict_vector_of_each_state: stored
+                    .block
+                    .verdict_vector_number_of_each_state()
+                    .iter()
+                    .map(|number| vector_bytes(*number))
+                    .collect(),
+                violations: stored
+                    .violations
+                    .iter()
+                    .map(|violation| {
+                        (
+                            violation.offset_in_block.0,
+                            vector_bytes(violation.verdict_vector_number),
+                        )
+                    })
+                    .collect(),
+                count_vector_bytes: count_vector_bytes(&stored.count_vector),
+                judged_on: stored.judged_on,
+                recovery_words: stored.recovery_words,
+            })
+    }
+
+    /// B 里属于流 G 的行：键里带 G 的节点名的（G 的块的结论、核对行），G 的路径与事实行。
+    fn is_a_row_of_flow_g(key_bytes: &[u8]) -> bool {
+        let flow_g_node_bytes = FLOW_G_NODE.as_bytes();
+        key_bytes
+            .windows(flow_g_node_bytes.len())
+            .any(|window| window == flow_g_node_bytes)
+            || key_bytes == VerdictStoreKey::Path("g.0/g.1").to_key_bytes().0.as_slice()
+            || key_bytes
+                == VerdictStoreKey::FlowFacts(&FLOW_G_FACTS_DIGEST)
+                    .to_key_bytes()
+                    .0
+                    .as_slice()
+    }
+
+    /// 优化前后对拍（`.claude/singlefs-ai-sop/rules/test-script-discipline.md`「编译与算法都要快」：优化前后在同一组输入上判定逐项相同）：
+    /// 两份逐字节相同的本库 A，一份整库导入对方库 B（[`VerdictStore::import_from`]，改之前流水线的导法），一份按计划只导流 F 不归本机的那几块；
+    /// 流 F 的每个块键在每种结论、每把版本下读回相同，F 的路径、核对行、事实行读回相同。按计划导入写下的行数钉成绝对值，
+    /// 写下的正是 B 比 A 多出、属于 F 的那几行；B 里流 G 的新行不写。
+    #[test]
+    fn importing_the_planned_rows_of_a_flow_reads_back_like_importing_the_whole_library_and_writes_only_that_flow_s_new_rows(
+    ) {
+        let other_directory = TemporaryLibraryDirectory::new("planned-import-other");
+        let other = build_the_other_library_b(&other_directory.path);
+        let whole_directory = TemporaryLibraryDirectory::new("planned-import-whole");
+        let mut imported_whole = build_the_local_library_a(&whole_directory.path);
+        let planned_directory = TemporaryLibraryDirectory::new("planned-import-planned");
+        let mut imported_by_plan = build_the_local_library_a(&planned_directory.path);
+        let entries_before = imported_by_plan.every_stored_entry();
+        assert_eq!(
+            entries_before,
+            imported_whole.every_stored_entry(),
+            "两份本库 A 开头逐字节相同"
+        );
+        imported_whole.import_from(&other).expect("整库导入");
+        let tally = imported_by_plan
+            .import_the_planned_rows_from(
+                &other,
+                &planned_keys_of_flow_f_beyond_this_host_s_share(),
+            )
+            .expect("按计划导入");
+        for block_index in 0..4 {
+            for (kind, conclusion_version) in [
+                (ConclusionKind::GpuJudge, GPU_CONCLUSION_VERSION),
+                (ConclusionKind::GpuJudge, OLD_GPU_CONCLUSION_VERSION),
+                (ConclusionKind::CpuJudge, CPU_CONCLUSION_VERSION),
+                (
+                    observer_kind(MOUNT_OBSERVER_NAME),
+                    OBSERVER_CONCLUSION_VERSION,
+                ),
+            ] {
+                let block_key = flow_f_block(block_index);
+                assert_eq!(
+                    conclusion_read_back_by_content(
+                        &imported_by_plan,
+                        &block_key,
+                        &kind,
+                        &conclusion_version
+                    ),
+                    conclusion_read_back_by_content(
+                        &imported_whole,
+                        &block_key,
+                        &kind,
+                        &conclusion_version
+                    ),
+                    "流 F 第 {block_index} 块的 {kind:?} 在版本 {:02x}… 下两种导法读回不同",
+                    conclusion_version.0[0]
+                );
+            }
+            for verifier_digest in [CPU_VERIFIER_DIGEST, GPU_VERIFIER_DIGEST] {
+                assert_eq!(
+                    imported_by_plan
+                        .read_verifier_block(&flow_f_block(block_index), &verifier_digest)
+                        .expect("读核对行"),
+                    imported_whole
+                        .read_verifier_block(&flow_f_block(block_index), &verifier_digest)
+                        .expect("读核对行"),
+                    "流 F 第 {block_index} 块在核对代码 {:02x}… 下的核对行两种导法读回不同",
+                    verifier_digest[0]
+                );
+            }
+        }
+        for path in ["f.0", "f.0/f.1", "f.0/f.1/f.2"] {
+            assert_eq!(
+                imported_by_plan.identity_of_path(path).expect("读路径"),
+                imported_whole.identity_of_path(path).expect("读路径"),
+                "流 F 的路径 {path} 两种导法读回不同"
+            );
+        }
+        assert_eq!(
+            imported_by_plan
+                .read_flow_facts(&FLOW_F_FACTS_DIGEST)
+                .expect("读事实"),
+            imported_whole
+                .read_flow_facts(&FLOW_F_FACTS_DIGEST)
+                .expect("读事实"),
+            "流 F 的事实行两种导法读回不同"
+        );
+        // 绝对值：结论 3 块写下（F1 的 GPU 5 行、F2 的 GPU 5 行、F2 的观察者 4 行），F1 的 CPU 本库已有、相同；
+        // F1 的观察者、F2 的 CPU、F3 的三种对方库里没有。另：新判定向量 2 行、路径 2 行（f.0 相同跳过）、核对 2 行、事实 1 行
+        assert_eq!(
+            tally,
+            PlannedImportTally {
+                conclusion_blocks_written: 3,
+                conclusion_blocks_already_identical: 1,
+                conclusion_blocks_missing_in_the_other_library: 5,
+                paths_written: 2,
+                verifier_blocks_written: 2,
+                flow_facts_written: 1,
+                rows_written: 21,
+                synced_writes: 1,
+            },
+            "按计划导入的账"
+        );
+        let entries_before: BTreeSet<StoredEntry> = entries_before.into_iter().collect();
+        let written_by_plan: BTreeSet<StoredEntry> = imported_by_plan
+            .every_stored_entry()
+            .into_iter()
+            .filter(|entry| !entries_before.contains(entry))
+            .collect();
+        let other_beyond_a_of_flow_f: BTreeSet<StoredEntry> = other
+            .every_stored_entry()
+            .into_iter()
+            .filter(|entry| !entries_before.contains(entry) && !is_a_row_of_flow_g(&entry.0))
+            .collect();
+        assert_eq!(written_by_plan.len(), 21, "按计划导入写下 21 行");
+        assert_eq!(
+            written_by_plan, other_beyond_a_of_flow_f,
+            "按计划导入写下的正是 B 比 A 多出、属于流 F 的那几行"
+        );
+        assert!(
+            conclusion_read_back_by_content(
+                &imported_whole,
+                &flow_g_block(1),
+                &ConclusionKind::GpuJudge,
+                &GPU_CONCLUSION_VERSION
+            )
+            .is_some()
+                && conclusion_read_back_by_content(
+                    &imported_by_plan,
+                    &flow_g_block(1),
+                    &ConclusionKind::GpuJudge,
+                    &GPU_CONCLUSION_VERSION
+                )
+                .is_none(),
+            "流 G 的新结论整库导入带过来了，按计划导入不碰它"
+        );
+    }
+
+    /// 按计划导入时本库同键已有不同的结论（流 F 第 2 块的 GPU 结论两边判得不同）：照存块的规矩报错，点名那一块那一种；
+    /// 排在它前面已攒进这一批的行（F1 的 GPU 结论）一行都不写，本库逐字节不变。
+    #[test]
+    fn planned_import_refuses_a_different_conclusion_under_a_stored_key_and_leaves_the_library_unchanged(
+    ) {
+        let other_directory = TemporaryLibraryDirectory::new("planned-import-conflict-other");
+        let other = build_the_other_library_b(&other_directory.path);
+        let local_directory = TemporaryLibraryDirectory::new("planned-import-conflict-local");
+        let mut local = build_the_local_library_a(&local_directory.path);
+        let red = local
+            .register_verdict_vector(&verdict_vector(b"red-f"))
+            .expect("登记");
+        store_conclusion_in_the_test_library(
+            &mut local,
+            &flow_f_block(2),
+            &ConclusionKind::GpuJudge,
+            &GPU_CONCLUSION_VERSION,
+            &conclusion_record(block_of(&[red, red, red]), &[0, 1, 2]),
+        );
+        let entries_before = local.every_stored_entry();
+        match local.import_the_planned_rows_from(
+            &other,
+            &planned_keys_of_flow_f_beyond_this_host_s_share(),
+        ) {
+            Err(
+                VerdictStoreError::DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion {
+                    block_key,
+                    kind,
+                },
+            ) => {
+                assert_eq!(block_key, flow_f_block(2), "点名判得不同的那一块");
+                assert_eq!(kind, ConclusionKind::GpuJudge, "点名判得不同的那一种");
+            }
+            other_outcome => panic!("本库同键已有不同的结论应当报错，得到 {other_outcome:?}"),
+        }
+        assert_eq!(
+            local.every_stored_entry(),
+            entries_before,
+            "报错之前攒着的那一批一行都没写"
+        );
+    }
+
+    /// 同一条路径再登同一个身份：什么都不写（库的序号不动）；换一个身份照写。
+    #[test]
+    fn recording_a_path_again_with_the_same_identity_writes_nothing() {
+        let directory = TemporaryLibraryDirectory::new("path-recorded-twice");
+        let mut store = VerdictStore::create_empty(&directory.path).expect("建库");
+        store
+            .record_path("a.unit_write.1", "f::a.unit_write.1::00")
+            .expect("登路径");
+        let sequence_after_the_first = store.database.latest_sequence_number();
+        store
+            .record_path("a.unit_write.1", "f::a.unit_write.1::00")
+            .expect("再登同一个身份");
+        assert_eq!(
+            store.database.latest_sequence_number(),
+            sequence_after_the_first,
+            "同一个身份再登不写库"
+        );
+        store
+            .record_path("a.unit_write.1", "f::a.unit_write.1::11")
+            .expect("换一个身份");
+        assert!(
+            store.database.latest_sequence_number() > sequence_after_the_first,
+            "换了身份照写"
+        );
+        assert_eq!(
+            store.identity_of_path("a.unit_write.1").expect("读路径"),
+            Some("f::a.unit_write.1::11".to_string()),
+            "最后登的身份留在索引里"
+        );
+    }
 }
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs b/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
--- a/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
@@ -1,7 +1,8 @@
-//! checker 档模块：crash、layer0_progress、crash_identity、crash_amplification、verdict_store、gpu_unit_checks
+//! checker 档模块：crash、layer0_progress、crash_given_states、crash_identity、crash_amplification、verdict_store、gpu_unit_checks
 //! 崩溃放量流水线在新池新建文件那条流（取号与暖机、发布两个崩溃点）上的端到端用例：录入 → 核对 → 第二趟复用 → 两方分工与对账 →
 //! 两个库并成一个 → ignore 的点不出状态 → 版本表写错要红；经公共模块按环境真跑两趟，驱动行带复用数、全域与否、展开上限；
-//! 被叫停、导入别的库、只排派活计划的那一趟照实报复用数（L1）。经环境跑的用例自己清、自己还原环境（K5，公共模块的环境守卫）。
+//! 被叫停、导入别的库、只排派活计划的那一趟照实报复用数（L1）；两台分片之后按计划点名导入，流水线那一侧与给定状态表那一侧
+//! 各与一台判完、与旧的整库导入逐项相同。经环境跑的用例自己清、自己还原环境（K5，公共模块的环境守卫）。
 //! 要 `verdict-store` 特性（RocksDB），不带特性时这份文件是空的。
 #![cfg(feature = "verdict-store")]
 
@@ -15,25 +16,37 @@
 
 use common::{build_pool, file_content, geometry};
 use singlefs_checker_tier::crash::{
-    layer0_plan_state_count, Layer0SegmentExpansion, LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE,
-    LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE,
+    layer0_plan_state_count, Layer0ObserverCounts, Layer0SegmentExpansion,
+    LAYER0_RED_PASS_JOURNAL_CONSULTED_ORACLE, LAYER0_RED_PASS_JOURNAL_IGNORED_ORACLE,
 };
 use singlefs_checker_tier::crash_amplification::{
     check_blocks_where_until, check_recorded_blocks, compare_judge_and_verifier, plan_crash_points,
     read_violations, record_crash_points, unjudged_blocks, unjudged_blocks_of_the_flow,
     verify_blocks_from_facts, ConclusionsStoredUnder, CrashFlow, CrashPlan, CrashPointSpan,
-    EnvironmentRun, ExpandUpToWritesPerSegment, FactsVerifierOn, PipelineJudgingCode,
+    EnvironmentRun, ExpandUpToWritesPerSegment, FactsVerifierOn, GivenStatesEngine,
+    GivenStatesItem, GivenStatesRun, NodeOfAnObserver, ObserverOnTheNodes, PipelineJudgingCode,
     UnitChecksumsOn, WeightedShares, END_OF_FLOW_NODE_NAME, ENVIRONMENT_VARIABLES_READ_BY_A_RUN,
 };
-use singlefs_checker_tier::crash_identity::{ConclusionKind, ConclusionVersion, CoverageReport};
+use singlefs_checker_tier::crash_given_states::{
+    GivenStatesConclusionsOfATable, GivenStatesJudge, GivenStatesOfATable,
+};
+use singlefs_checker_tier::crash_identity::{
+    ConclusionKind, ConclusionVersion, CoverageReport, ObserverName,
+};
 #[cfg(feature = "gpu")]
 use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
 use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;
-use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
+use singlefs_checker_tier::verdict_store::{
+    BlockCountVector, EncodedVerdictVector, JudgeVersion, JudgingDevice, RecoveryWordsOfState,
+    StateOffsetInBlock, VerdictBlockKey, VerdictStore, VerdictVectorNumber,
+};
+use singlefs_core::recovery::RecoveryReport;
 use singlefs_harness::memory_pool::{
-    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
+    writes_and_segments_with_stream_indexes, CrashImage, MemoryPool, PublishedVersion,
+    RetainedWrite,
 };
 use singlefs_harness::segments::StepKind;
+use singlefs_harness::sha256::DIGEST_BYTES;
 
 const THIS_FILE: &str =
     "crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs";
@@ -1117,6 +1130,438 @@
     drop(directory);
 }
 
+/// 一块一种结论按内容读回（两份库各有各的判定向量编号，按编号比没有意义）：逐状态的判定向量、违例（块内序号与判定向量）、计数向量、
+/// 块属性、恢复字。
+#[derive(Debug, PartialEq, Eq)]
+struct ConclusionReadBackByContent {
+    verdict_vector_of_each_state: Vec<EncodedVerdictVector>,
+    violations: Vec<(StateOffsetInBlock, EncodedVerdictVector)>,
+    count_vector: BlockCountVector,
+    judged_on: JudgingDevice,
+    recovery_words: Option<Vec<RecoveryWordsOfState>>,
+}
+
+/// `store` 里这一块这一种结论在这一把版本下按内容读回；没存过是 `None`。
+fn conclusion_read_back_by_content(
+    store: &VerdictStore,
+    block_key: &VerdictBlockKey,
+    kind: &ConclusionKind,
+    conclusion_version: &ConclusionVersion,
+) -> Option<ConclusionReadBackByContent> {
+    let stored = store
+        .read_conclusion_block(block_key, kind, conclusion_version)
+        .expect("读得了结论块")?;
+    let vector_of_number = |number: VerdictVectorNumber| {
+        store
+            .verdict_vector_of_number(number)
+            .expect("结论块里的编号在它自己的库里登记过")
+            .clone()
+    };
+    Some(ConclusionReadBackByContent {
+        verdict_vector_of_each_state: stored
+            .block
+            .verdict_vector_number_of_each_state()
+            .iter()
+            .map(|number| vector_of_number(*number))
+            .collect(),
+        violations: stored
+            .violations
+            .iter()
+            .map(|violation| {
+                (
+                    violation.offset_in_block,
+                    vector_of_number(violation.verdict_vector_number),
+                )
+            })
+            .collect(),
+        count_vector: stored.count_vector,
+        judged_on: stored.judged_on,
+        recovery_words: stored.recovery_words,
+    })
+}
+
+/// 导入用例里挂在发布那个点上的观察者：只数它看过几个状态，不判红。
+const STATES_SEEN_BY_THE_IMPORT_PROBE_COUNTER: &str = "states_seen_by_the_import_probe";
+
+/// 一条录好的流带着挂在节点上的观察者经公共模块按环境跑一趟（展开上限默认 6），交回这一趟的结果。环境由调用方的守卫定。
+fn amplify_with_observers(
+    recorded: &RecordedFlow,
+    flow_name: &str,
+    observers: &[ObserverOnTheNodes<'_>],
+) -> EnvironmentRun {
+    common_crash_points::amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
+        flow_name,
+        &recorded.base,
+        &recorded.writes,
+        &recorded.segments,
+        recorded.judged_root_index,
+        &recorded.versions,
+        &[],
+        recorded.crash_points.clone(),
+        LARGEST_EXPANDED_SEGMENT_WRITES,
+        observers,
+    )
+    .0
+}
+
+/// 两台分片之后的导入（`scripts/crash/crash-amplification-two-hosts.sh` 那几步：本机 0/2、第二台 1/2 各写一份库，本机再带
+/// `SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<第二台的库>` 跑一趟 0/2）按这一趟的计划只点不归本机的块。新池新建文件那条流 3 块，平滑加权轮转
+/// 把第 0、2 块分给 0/2、第 1 块（发布那个点）分给 1/2；发布那个点挂一个只数状态的观察者。导入之后这条流一块没判的都没有；
+/// 本机库里每块每种结论（两种判器、观察者）按内容读回，与一台把整条流判完的那份库逐项相同；导入写的结论块数等于第二台那一份库里的
+/// 结论块数，钉成绝对值 2（CPU 判器 1 块、观察者 1 块；没配 GPU，GPU 判器那一种两边都没有）。
+/// 判别力（`crates/mutations.tsv`）：排键时选块条件反过来（点的是本机那一份），第二台那一块导不进来、对账剩 1 块；
+/// 排键漏掉观察者那一种，导入只写 1 块、本机库里发布那一块没有观察者的结论。
+#[test]
+fn importing_the_other_share_names_its_blocks_and_reads_back_like_one_host_judging_the_whole_flow()
+{
+    let recorded =
+        record_new_pool_file_creation("crash-amplification-import-planned", false, file_content());
+    let observe_by_counting =
+        |_image: &CrashImage<'_>, _report: &RecoveryReport, counts: &mut Layer0ObserverCounts| {
+            counts.add(STATES_SEEN_BY_THE_IMPORT_PROBE_COUNTER, 1);
+        };
+    let observers = [ObserverOnTheNodes {
+        observer_name: ObserverName::new("import_probe").expect("观察者的名字非空、不长"),
+        observer_code_file: THIS_FILE.to_string(),
+        extra_inputs_digest: [0; DIGEST_BYTES],
+        nodes: vec![NodeOfAnObserver {
+            setting_file: THIS_FILE.to_string(),
+            entry_name: "publish_first_file".to_string(),
+        }],
+        observe: &observe_by_counting,
+    }];
+    let plan = plan_crash_points(&recorded.flow());
+    assert_eq!(total_blocks(&plan), 3, "前提：两个点与末节点各一块");
+    let shares = WeightedShares::new(vec![1, 1]);
+    let blocks_of_the_other_share = (0..total_blocks(&plan))
+        .filter(|block| shares.owner_of_block(*block) == 1)
+        .count();
+    assert_eq!(
+        blocks_of_the_other_share, 1,
+        "前提：1/2 那一份只有第 1 块（发布那个点）"
+    );
+    let directory = EnvironmentRunDirectory::new("import-planned");
+    let one_host_library = directory.temporary.directory.join("one-host-library");
+    let other_share_library = directory.temporary.directory.join("other-share-library");
+    let this_share_library = directory.library.clone();
+    let mut environment = directory.environment();
+    environment.set(
+        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
+        one_host_library.as_os_str(),
+    );
+    let one_host_run = amplify_with_observers(&recorded, "import-planned", &observers);
+    environment.set(
+        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
+        other_share_library.as_os_str(),
+    );
+    environment.set("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("1/2"));
+    let other_share_run = amplify_with_observers(&recorded, "import-planned", &observers);
+    environment.set(
+        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
+        this_share_library.as_os_str(),
+    );
+    environment.set("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("0/2"));
+    let this_share_run = amplify_with_observers(&recorded, "import-planned", &observers);
+    environment.set(
+        "SINGLEFS_CRASH_AMPLIFICATION_IMPORT",
+        other_share_library.as_os_str(),
+    );
+    let importing_run = amplify_with_observers(&recorded, "import-planned", &observers);
+    drop(environment);
+    assert_eq!(one_host_run.unjudged_blocks, 0, "一台判完整条流");
+    assert_eq!(
+        (
+            other_share_run.checking.blocks_checked,
+            this_share_run.checking.blocks_checked
+        ),
+        (1, 2),
+        "1/2 判第 1 块，0/2 判第 0、2 块"
+    );
+    assert_eq!(
+        this_share_run.unjudged_blocks, 1,
+        "导入之前 0/2 的库里第二台那一块没判"
+    );
+    assert_eq!(
+        importing_run.unjudged_blocks, 0,
+        "导入之后这条流一块没判的都没有"
+    );
+    assert_eq!(
+        importing_run.checking.blocks_checked, 0,
+        "导入那一趟本机那一份全复用、一块不判"
+    );
+    let (one_host_store, _) =
+        VerdictStore::open_existing(&one_host_library).expect("开得了一台判完的库");
+    let (other_share_store, _) =
+        VerdictStore::open_existing(&other_share_library).expect("开得了第二台那一份库");
+    let (this_share_store, _) =
+        VerdictStore::open_existing(&this_share_library).expect("开得了导入之后的本机库");
+    let other_share_conclusions = other_share_store
+        .stored_conclusion_keys()
+        .expect("列得出第二台那一份的结论");
+    assert_eq!(
+        other_share_conclusions.len(),
+        2,
+        "第二台那一份：发布那一块的 CPU 判器结论与观察者结论"
+    );
+    assert_eq!(
+        importing_run.imported_blocks, 2,
+        "导入写的结论块数等于第二台那一份的结论块数"
+    );
+    let one_host_conclusions = one_host_store
+        .stored_conclusion_keys()
+        .expect("列得出一台判完的结论");
+    let this_share_conclusions = this_share_store
+        .stored_conclusion_keys()
+        .expect("列得出导入之后的结论");
+    assert_eq!(
+        one_host_conclusions.len(),
+        4,
+        "一台判完：3 块 CPU 判器结论、发布那一块的观察者结论"
+    );
+    assert_eq!(
+        this_share_conclusions, one_host_conclusions,
+        "导入之后本机库里的（块键，种类，版本）与一台判完的逐项相同"
+    );
+    for (block_key, kind, conclusion_version) in &one_host_conclusions {
+        let read_back_from_one_host =
+            conclusion_read_back_by_content(&one_host_store, block_key, kind, conclusion_version);
+        assert!(read_back_from_one_host.is_some(), "刚列出来的结论读得到");
+        assert_eq!(
+            conclusion_read_back_by_content(&this_share_store, block_key, kind, conclusion_version),
+            read_back_from_one_host,
+            "块 {} 的 {kind:?} 结论：导入之后的本机库与一台判完的按内容读回不同",
+            block_key.stream_or_node_name.as_str()
+        );
+    }
+    drop((one_host_store, other_share_store, this_share_store));
+    drop(directory);
+}
+
+/// 给定状态表导入用例的两项：要导入的这一项，与第二台库里另判的另一项（按名导入不该带过来的）。
+const GIVEN_STATES_IMPORTED_ITEM: &str = "given_states_import_probe";
+const GIVEN_STATES_OTHER_ITEM: &str = "given_states_import_probe_other_item";
+const GIVEN_STATES_TABLES_OF_THE_IMPORTED_ITEM: usize = 4;
+const GIVEN_STATES_TABLES_OF_THE_OTHER_ITEM: usize = 2;
+
+/// 新池新建文件那条流上的几张给定状态表：第 k 张两个状态，持久了前 k·写数/张数 次写的与全部持久的（一条录制流的前缀都是合法的崩溃状态），
+/// 表名带 k。
+fn given_states_tables_of_the_flow(
+    recorded: &RecordedFlow,
+    table_count: usize,
+) -> Vec<GivenStatesOfATable<'_>> {
+    let write_count = recorded.writes.len();
+    (0..table_count)
+        .map(|table_index| {
+            let persisted_prefix_writes = table_index * write_count / table_count;
+            GivenStatesOfATable {
+                table_name: format!("import-probe-table-{table_index}"),
+                base: &recorded.base,
+                writes: &recorded.writes,
+                segments: &recorded.segments,
+                seams: &[],
+                persisted_sets: vec![
+                    (0..write_count)
+                        .map(|write_index| write_index < persisted_prefix_writes)
+                        .collect(),
+                    vec![true; write_count],
+                ],
+            }
+        })
+        .collect()
+}
+
+/// 照此刻的环境起一个给定状态表的引擎，判完 `tables` 每一张、收尾，交回每张的结论与这一趟。
+fn judge_every_given_states_table(
+    item_name: &str,
+    tables: &[GivenStatesOfATable<'_>],
+) -> (Vec<GivenStatesConclusionsOfATable>, GivenStatesRun) {
+    let engine = GivenStatesEngine::start_from_environment(
+        GivenStatesItem {
+            item_name: item_name.to_string(),
+            setting_file: THIS_FILE.to_string(),
+            seeds_and_scale: "new_pool_file_creation import_probe".to_string(),
+        },
+        &Layer0ToolchainIdentity::of_the_cargo_running_this_test(),
+        &repository_root(),
+    )
+    .expect("CPU 配置的给定状态表引擎起得来");
+    let conclusions = tables
+        .iter()
+        .map(|table| {
+            engine
+                .judge_the_given_states(table)
+                .unwrap_or_else(|error| {
+                    panic!("项 {item_name} 判得了表 {}：{error}", table.table_name)
+                })
+        })
+        .collect();
+    (conclusions, engine.finish())
+}
+
+/// 给定状态表那一路的导入按这一趟问到的表点名导：权重 0,1 让每张表都归 1/2 那一台；那一台判这一项的 4 张表与另一项的 2 张表、写一份库 B。
+/// 本机这一份（0/2，一张表都不归它）带 `SINGLEFS_CRASH_AMPLIFICATION_IMPORT=B` 判这一项的 4 张表：每张都从 B 点名导入、照库里的复用，
+/// 一张都不现判。与旧的整库导入对拍（`VerdictStore::import_from` 把 B 整个导进一份空库，再在那份库上起本机这一份、不设导入，判同一批）：
+/// 两边交回的结论逐张相同；按名导入的库里这一项的（块键，种类，版本）与整库导入的那份里这一项的逐项相同、按内容读回逐项相同，
+/// 另一项的一行都没有。账钉成绝对值：导入 4 块（每张表一块 CPU 判器结论；没配 GPU，GPU 判器那一种 B 里没有）、复用 4 张、现判 0 张。
+/// 判别力（`crates/mutations.tsv`）：归别的机器的表不导（导入条件里的归属反过来），引擎判第一张表就报「导入之后库里还是没有」；
+/// 点名漏掉 CPU 判器那一种，同样报错。
+#[test]
+fn given_states_tables_of_another_host_are_imported_by_name_and_read_back_like_the_whole_library_import(
+) {
+    let recorded = record_new_pool_file_creation(
+        "crash-amplification-given-states-import",
+        false,
+        file_content(),
+    );
+    let tables =
+        given_states_tables_of_the_flow(&recorded, GIVEN_STATES_TABLES_OF_THE_IMPORTED_ITEM);
+    let other_item_tables =
+        given_states_tables_of_the_flow(&recorded, GIVEN_STATES_TABLES_OF_THE_OTHER_ITEM);
+    let directory = EnvironmentRunDirectory::new("given-states-import");
+    let other_host_library = directory.temporary.directory.join("other-host-library");
+    let whole_import_library = directory.temporary.directory.join("whole-import-library");
+    let named_import_library = directory.library.clone();
+    let mut environment = directory.environment();
+    environment.set(
+        "SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS",
+        OsStr::new("0,1"),
+    );
+    environment.set("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("1/2"));
+    environment.set(
+        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
+        other_host_library.as_os_str(),
+    );
+    let (_, other_host_run) = judge_every_given_states_table(GIVEN_STATES_IMPORTED_ITEM, &tables);
+    let (_, other_host_other_item_run) =
+        judge_every_given_states_table(GIVEN_STATES_OTHER_ITEM, &other_item_tables);
+    {
+        let (other_host_store, _) =
+            VerdictStore::open_existing(&other_host_library).expect("开得了第二台的库");
+        let mut whole_import_store =
+            VerdictStore::create_empty(&whole_import_library).expect("建得了整库导入那一份");
+        whole_import_store
+            .import_from(&other_host_store)
+            .expect("整库导得进");
+    }
+    environment.set("SINGLEFS_CRASH_AMPLIFICATION_SHARE", OsStr::new("0/2"));
+    environment.set(
+        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
+        whole_import_library.as_os_str(),
+    );
+    let (after_the_whole_import, whole_import_run) =
+        judge_every_given_states_table(GIVEN_STATES_IMPORTED_ITEM, &tables);
+    environment.set(
+        "SINGLEFS_CRASH_AMPLIFICATION_LIBRARY",
+        named_import_library.as_os_str(),
+    );
+    environment.set(
+        "SINGLEFS_CRASH_AMPLIFICATION_IMPORT",
+        other_host_library.as_os_str(),
+    );
+    let (after_the_named_import, named_import_run) =
+        judge_every_given_states_table(GIVEN_STATES_IMPORTED_ITEM, &tables);
+    drop(environment);
+    assert_eq!(
+        (
+            other_host_run.tally.tables_judged_here,
+            other_host_other_item_run.tally.tables_judged_here
+        ),
+        (4, 2),
+        "前提：权重 0,1 下每张表都归 1/2，那一台现判了两项的每一张"
+    );
+    let tally_counts = |run: &GivenStatesRun| {
+        (
+            run.tally.imported_blocks,
+            run.tally.tables_reused,
+            run.tally.tables_judged_here,
+            run.tally.tables_left_to_the_other_hosts,
+        )
+    };
+    assert_eq!(
+        tally_counts(&whole_import_run),
+        (0, 4, 0, 0),
+        "整库导入那一份：不设导入，4 张表全照库里的复用"
+    );
+    assert_eq!(
+        tally_counts(&named_import_run),
+        (4, 4, 0, 0),
+        "按名导入：导入 4 块、复用 4 张、现判 0 张、留给别的机器 0 张"
+    );
+    assert_eq!(
+        after_the_named_import, after_the_whole_import,
+        "两种导法之后引擎交回的结论逐张相同"
+    );
+    let (other_host_store, _) =
+        VerdictStore::open_existing(&other_host_library).expect("开得了第二台的库");
+    let (whole_import_store, _) =
+        VerdictStore::open_existing(&whole_import_library).expect("开得了整库导入那一份");
+    let (named_import_store, _) =
+        VerdictStore::open_existing(&named_import_library).expect("开得了按名导入那一份");
+    let node_name_prefix_of_the_imported_item =
+        format!("{THIS_FILE}::{GIVEN_STATES_IMPORTED_ITEM}::");
+    let whole_import_conclusions = whole_import_store
+        .stored_conclusion_keys()
+        .expect("列得出整库导入那一份的结论");
+    assert_eq!(
+        whole_import_conclusions.len(),
+        6,
+        "前提：整库导入把另一项的 2 张也带过来了"
+    );
+    let named_import_conclusions = named_import_store
+        .stored_conclusion_keys()
+        .expect("列得出按名导入那一份的结论");
+    let whole_import_conclusions_of_the_imported_item: Vec<_> = whole_import_conclusions
+        .iter()
+        .filter(|(block_key, _, _)| {
+            block_key
+                .stream_or_node_name
+                .as_str()
+                .starts_with(&node_name_prefix_of_the_imported_item)
+        })
+        .cloned()
+        .collect();
+    assert_eq!(
+        whole_import_conclusions_of_the_imported_item.len(),
+        4,
+        "这一项 4 张表各一块 CPU 判器结论"
+    );
+    assert_eq!(
+        named_import_conclusions, whole_import_conclusions_of_the_imported_item,
+        "按名导入的库里只有这一项的结论，与整库导入那一份里这一项的逐项相同"
+    );
+    for (block_key, kind, conclusion_version) in &named_import_conclusions {
+        let read_back_from_the_whole_import = conclusion_read_back_by_content(
+            &whole_import_store,
+            block_key,
+            kind,
+            conclusion_version,
+        );
+        assert!(
+            read_back_from_the_whole_import.is_some(),
+            "刚列出来的结论读得到"
+        );
+        assert_eq!(
+            conclusion_read_back_by_content(
+                &named_import_store,
+                block_key,
+                kind,
+                conclusion_version
+            ),
+            read_back_from_the_whole_import,
+            "表 {} 的 {kind:?} 结论：两种导法按内容读回不同",
+            block_key.stream_or_node_name.as_str()
+        );
+        assert_eq!(
+            conclusion_read_back_by_content(&other_host_store, block_key, kind, conclusion_version),
+            read_back_from_the_whole_import,
+            "表 {} 的 {kind:?} 结论：导过来的与第二台库里的按内容读回不同",
+            block_key.stream_or_node_name.as_str()
+        );
+    }
+    drop((other_host_store, whole_import_store, named_import_store));
+    drop(directory);
+}
+
 /// L1：只排派活计划的那一趟照实报录入时库里已判的态数，不写 0。同一份库上第一趟（单机 CPU）把整条流判完；第二趟配置开 GPU、
 /// 登记 `crash_judge_gpu`、只排计划：一块都不判，复用数等于整条流的态数。要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」
 /// （经这个进程的卡池判有没有卡，`gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run`），有适配器而点名的卡找不到判红。
````

### mutations-append.tsv

````text
按计划导入 键表里漏掉第一项结论	crates/singlefs-checker-tier/src/verdict_store.rs	        for (block_key, kind, conclusion_version) in &planned.conclusions {	        for (block_key, kind, conclusion_version) in planned.conclusions.iter().skip(1) {	-p singlefs-checker-tier --features verdict-store --lib -- verdict_store::tests::importing_the_planned_rows_of_a_flow_reads_back_like_importing_the_whole_library_and_writes_only_that_flow_s_new_rows	importing_the_planned_rows_of_a_flow_reads_back_like_importing_the_whole_library_and_writes_only_that_flow_s_new_rows
按计划导入 路径本库有就跳过、不比身份	crates/singlefs-checker-tier/src/verdict_store.rs	            if self.identity_of_path(path)?.as_deref() == Some(identity.as_str()) {	            if self.identity_of_path(path)?.is_some() {	-p singlefs-checker-tier --features verdict-store --lib -- verdict_store::tests::importing_the_planned_rows_of_a_flow_reads_back_like_importing_the_whole_library_and_writes_only_that_flow_s_new_rows	importing_the_planned_rows_of_a_flow_reads_back_like_importing_the_whole_library_and_writes_only_that_flow_s_new_rows
按计划导入 核对行点名的事实行不随着导	crates/singlefs-checker-tier/src/verdict_store.rs	            facts_digests_named.insert(facts_digest);	            let _ = facts_digest;	-p singlefs-checker-tier --features verdict-store --lib -- verdict_store::tests::importing_the_planned_rows_of_a_flow_reads_back_like_importing_the_whole_library_and_writes_only_that_flow_s_new_rows	importing_the_planned_rows_of_a_flow_reads_back_like_importing_the_whole_library_and_writes_only_that_flow_s_new_rows
按计划导入 本库有同键结论就跳过、不比内容	crates/singlefs-checker-tier/src/verdict_store.rs	            let requested = self.conclusion_renumbered_for_the_planned_import(	            if self.read_conclusion_block(block_key, kind, conclusion_version)?.is_some() {\n                tally.conclusion_blocks_already_identical += 1;\n                continue;\n            }\n            let requested = self.conclusion_renumbered_for_the_planned_import(	-p singlefs-checker-tier --features verdict-store --lib -- verdict_store::tests::planned_import_refuses_a_different_conclusion_under_a_stored_key_and_leaves_the_library_unchanged	planned_import_refuses_a_different_conclusion_under_a_stored_key_and_leaves_the_library_unchanged
判定存储 路径再登同一身份照写	crates/singlefs-checker-tier/src/verdict_store.rs	        if stored_identity.as_deref() == Some(identity.as_bytes()) {	        if false && stored_identity.as_deref() == Some(identity.as_bytes()) {	-p singlefs-checker-tier --features verdict-store --lib -- verdict_store::tests::recording_a_path_again_with_the_same_identity_writes_nothing	recording_a_path_again_with_the_same_identity_writes_nothing
按计划导入 批长判据反过来（每项都同步写一批）	crates/singlefs-checker-tier/src/verdict_store.rs	        if pending.batch.size_in_bytes() < PLANNED_IMPORT_BATCH_BYTES {	        if pending.batch.size_in_bytes() >= PLANNED_IMPORT_BATCH_BYTES {	-p singlefs-checker-tier --features verdict-store --lib -- verdict_store::tests::importing_the_planned_rows_of_a_flow_reads_back_like_importing_the_whole_library_and_writes_only_that_flow_s_new_rows	importing_the_planned_rows_of_a_flow_reads_back_like_importing_the_whole_library_and_writes_only_that_flow_s_new_rows
按计划导入 排键时选块条件反过来（点了本机那一份）	crates/singlefs-checker-tier/src/crash_amplification.rs	                if selected.contains(&this_block) {	                if !selected.contains(&this_block) {	-p singlefs-checker-tier --features verdict-store --test crash_amplification_pipeline_records_checks_and_reuses -- importing_the_other_share_names_its_blocks_and_reads_back_like_one_host_judging_the_whole_flow	importing_the_other_share_names_its_blocks_and_reads_back_like_one_host_judging_the_whole_flow
按计划导入 排键漏掉观察者那一种结论	crates/singlefs-checker-tier/src/crash_amplification.rs	                    conclusions_of_the_point.push((	                    drop((	-p singlefs-checker-tier --features verdict-store --test crash_amplification_pipeline_records_checks_and_reuses -- importing_the_other_share_names_its_blocks_and_reads_back_like_one_host_judging_the_whole_flow	importing_the_other_share_names_its_blocks_and_reads_back_like_one_host_judging_the_whole_flow
给定状态表按名导入 归别的机器的表不导（导入条件里的归属反过来）	crates/singlefs-checker-tier/src/crash_amplification.rs	        if !self.libraries_to_import_from.is_empty() && !self.owns_the_table(&key) {	        if !self.libraries_to_import_from.is_empty() && self.owns_the_table(&key) {	-p singlefs-checker-tier --features verdict-store --test crash_amplification_pipeline_records_checks_and_reuses -- given_states_tables_of_another_host_are_imported_by_name_and_read_back_like_the_whole_library_import	given_states_tables_of_another_host_are_imported_by_name_and_read_back_like_the_whole_library_import
给定状态表按名导入 点名漏掉 CPU 判器那一种结论	crates/singlefs-checker-tier/src/crash_amplification.rs	            conclusions: [JudgedBy::GpuJudge, JudgedBy::CpuJudge]	            conclusions: [JudgedBy::GpuJudge]	-p singlefs-checker-tier --features verdict-store --test crash_amplification_pipeline_records_checks_and_reuses -- given_states_tables_of_another_host_are_imported_by_name_and_read_back_like_the_whole_library_import	given_states_tables_of_another_host_are_imported_by_name_and_read_back_like_the_whole_library_import
````

### mutations-replacements.tsv

````text
判定库 恢复字不写进旁表	crates/singlefs-checker-tier/src/verdict_store.rs	    if let Some(recovery_words) = &conclusion.recovery_words {	    if let Some(recovery_words) = &None::<Vec<RecoveryWordsOfState>> {	-p singlefs-checker-tier --features verdict-store --lib -- verdict_store::tests::a_conclusion_block_reads_back_with_its_count_vector_attributes_and_recovery_words	a_conclusion_block_reads_back_with_its_count_vector_attributes_and_recovery_words
E4 给定状态引擎 导入之后还缺别的机器的表照占位的结论往下走	crates/singlefs-checker-tier/src/crash_amplification.rs	            if !self.libraries_to_import_from.is_empty() {	            if false && !self.libraries_to_import_from.is_empty() {	-p singlefs-checker-tier --features verdict-store --test crash_injection_campaign -- two_shares_judged_apart_and_then_imported_judge_every_table_once_and_render_the_report_of_one_host	two_shares_judged_apart_and_then_imported_judge_every_table_once_and_render_the_report_of_one_host
````

