# 删除线第 59 批改动全文（2026-10-10）

## gpu-ring-turn：crates.patch

````diff
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 3205e6b..d212408 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -14023,3 +14023,195 @@ fn the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement
         KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
     );
 }
+
+// ───────────── 第 59 批 L1（判决 `research/prompts/m3-batch58-code-r1-main-verification.md` 第三节 L1；两条腿 K1-3）：少一条的那片之后还有满的一片、根环转过一圈 ─────────────
+
+/// 写的时候逐条换的旋钮值：2、2、1、1、2、1 写出 [2, 1, 2, 1]（`crates/singlefs-harness/tests/checker_known_bad_images.rs` 的
+/// `A_SHORT_CONTAINER_BEFORE_A_FULL_ONE` 那一形）。第 58 批 J3 那三条 [2, 1, 1] 上少一条的那片只会落在倒数第二位，GPU 扫描内核
+/// 「只判倒数第二片」在它们上面与正确判法逐位相同；这里少一条的容器 2 之后还有一片满的。
+const KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_CONTAINER_BEFORE_A_FULL_ONE: [u64; 6] =
+    [2, 2, 1, 1, 2, 1];
+/// 按叶序每片的记录数。
+const RECORD_COUNTS_OF_A_SHORT_CONTAINER_BEFORE_A_FULL_ONE: [usize; 4] = [2, 1, 2, 1];
+
+/// 第 59 批 L1 那三条流写的是哪一棵容器树（另两棵照产品）。
+#[derive(Clone, Copy, Debug)]
+enum ContainerTreeWrittenBeforeTheRootRingTurnsOver {
+    Tombstone,
+    Lineage,
+    Retirement,
+}
+
+impl ContainerTreeWrittenBeforeTheRootRingTurnsOver {
+    /// 新池新建文件之后，第 k 条记录写的时候只这一棵压到 `records_per_container_of_each_record[k]` 条一片：墓碑树逐个 unlink 空 inode 到底
+    /// （同 [`record_tombstone_containers_of_two_one_and_one_records_under_changing_knobs`]），谱系树挂着回退到第一个文件那一版
+    /// （[`pool_rolled_back_under_these_lineage_knobs`]），退役树每次退役独占最右那片叶的号（同
+    /// [`record_retirement_containers_of_two_one_and_one_records_under_changing_knobs`]）。
+    fn pool_written_under_these_knobs(
+        self,
+        tag: &str,
+        records_per_container_of_each_record: &[u64],
+    ) -> BuiltPool {
+        match self {
+            Self::Tombstone => {
+                let mut pool = pool_with_new_inodes(
+                    tag,
+                    u64::try_from(records_per_container_of_each_record.len()).expect("装得进 u64"),
+                );
+                // 迭代上界：切片的长度。
+                for (inode, records_per_container) in
+                    (FIRST_FILE.0 + 1..).zip(records_per_container_of_each_record)
+                {
+                    unlink_an_empty_inode_to_the_end_under_the_capacities(
+                        &mut pool,
+                        inode,
+                        capacities_of_the_tombstone_container_knob(*records_per_container),
+                    );
+                }
+                pool
+            }
+            Self::Lineage => pool_rolled_back_under_these_lineage_knobs(
+                tag,
+                records_per_container_of_each_record,
+            ),
+            Self::Retirement => {
+                let mut pool = pool_with_new_inodes(tag, INODE_RECORDS_PER_LEAF);
+                // 迭代上界：切片的长度。第 0 次删的是建出来的最后一个号（独占最右那片叶），之后每次先建一个再删它。
+                for (retirement_index, records_per_container) in
+                    records_per_container_of_each_record.iter().enumerate()
+                {
+                    if retirement_index > 0 {
+                        common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(
+                            &mut pool, 1,
+                        );
+                    }
+                    let inode = pool.output.inode_number_watermark() - 1;
+                    unlink_an_empty_inode_to_the_end_under_the_capacities(
+                        &mut pool,
+                        inode,
+                        capacities_of_the_retirement_container_knob(*records_per_container),
+                    );
+                }
+                pool
+            }
+        }
+    }
+
+    /// 这一版这一棵按叶序每片的记录数。
+    fn record_counts_of_each_container(self, pool: &BuiltPool) -> Vec<usize> {
+        match self {
+            Self::Tombstone => pool
+                .output
+                .tombstone_containers
+                .iter()
+                .map(|container| container.contents.records.len())
+                .collect(),
+            Self::Lineage => pool
+                .output
+                .lineage_containers
+                .iter()
+                .map(|container| container.contents.records.len())
+                .collect(),
+            Self::Retirement => pool
+                .output
+                .retirement_containers
+                .iter()
+                .map(|container| container.contents.records.len())
+                .collect(),
+        }
+    }
+
+    /// 判的时候编进 GPU 扫描内核、交给 CPU 新入口的旋钮值：只这一棵装 `records_per_container` 条一片。
+    fn knobs_judging(self, records_per_container: u64) -> ContainerCapacityKnobs {
+        match self {
+            Self::Tombstone => knobs_of_the_tombstone_tree(records_per_container),
+            Self::Lineage => knobs_of_the_lineage_tree(records_per_container),
+            Self::Retirement => knobs_of_the_retirement_tree(records_per_container),
+        }
+    }
+
+    /// 这一棵带旋钮值的那一条（墓碑树 I-9.22，谱系树与退役树 I-9.37）。
+    fn knob_capacity_invariant(self) -> &'static str {
+        match self {
+            Self::Tombstone => KNOB_CAPACITY_INVARIANT_OF_THE_TOMBSTONE_TREE,
+            Self::Lineage | Self::Retirement => {
+                KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES
+            }
+        }
+    }
+}
+
+/// 只 `tree` 这一棵按 [`KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_CONTAINER_BEFORE_A_FULL_ONE`] 逐条换旋钮写出 [2, 1, 2, 1]，
+/// 再建 30 次空 inode（[`turn_the_root_ring_over_by_creating_inodes`]）把根环转过一圈：写这棵树的那几版、连同写到一半时的 [2, 1, 1] 那一版
+/// （容器 2 在那一版是倒数第二片），都转出写完之后各版的回退候选集，那几版的状态上照判 [2, 1, 1]，写完之后的状态上候选集里只剩 [2, 1, 2, 1]
+/// （与 harness 慢档 `checker_known_bad_images.rs` 里 `A_SHORT_CONTAINER_BEFORE_A_FULL_ONE` 那三条同形）。流从 mkfs 之后录起，被判的是最后一次根槽写。
+fn record_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
+    tree: ContainerTreeWrittenBeforeTheRootRingTurnsOver,
+    tag: &str,
+    name: &'static str,
+) -> RecordedFlow {
+    let mut pool = tree.pool_written_under_these_knobs(
+        tag,
+        &KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
+    );
+    turn_the_root_ring_over_by_creating_inodes(&mut pool);
+    assert_eq!(
+        tree.record_counts_of_each_container(&pool),
+        RECORD_COUNTS_OF_A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
+        "{tree:?} 按叶序每片的记录数（根环转过一圈之后）"
+    );
+    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
+    recorded_flow_of_the_pool(&pool, name, versions)
+}
+
+/// L1：`tree` 这一棵 [2, 1, 2, 1]、根环转过一圈，GPU 扫描内核与 CPU 新入口都给这一棵旋钮 2：每片都不超旋钮值、容器号从 1 逐片加一、
+/// 每片至少一条，只「不是最右的容器 2 装了 1 条，不是旋钮值 2」不成立，CPU 红这一棵的那一条（I-9.22 / I-9.37），GPU 红在同样的状态、同样的位上。
+/// GPU 扫描内核只判倒数第二片（第 3 片，装满 2 条）在根环转过之后的状态上照绿，就在那里与 CPU 不一致。
+fn assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_on_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
+    test_name: &str,
+    tree: ContainerTreeWrittenBeforeTheRootRingTurnsOver,
+    tag: &str,
+    name: &'static str,
+) {
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
+        test_name,
+        &record_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
+            tree, tag, name,
+        ),
+        tree.knobs_judging(KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER),
+        tree.knob_capacity_invariant(),
+    );
+}
+
+#[test]
+fn the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
+) {
+    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_on_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
+        "the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over",
+        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Tombstone,
+        "crash-judge-gpu-tombstone-containers-of-two-one-two-and-one-records-ring-turned",
+        "tombstone-containers-of-two-one-two-and-one-records-with-the-root-ring-turned-over",
+    );
+}
+
+#[test]
+fn the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
+) {
+    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_on_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
+        "the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over",
+        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Lineage,
+        "crash-judge-gpu-lineage-containers-of-two-one-two-and-one-records-ring-turned",
+        "lineage-containers-of-two-one-two-and-one-records-with-the-root-ring-turned-over",
+    );
+}
+
+#[test]
+fn the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
+) {
+    assert_the_gpu_judge_given_the_knob_two_agrees_with_the_cpu_on_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over(
+        "the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over",
+        ContainerTreeWrittenBeforeTheRootRingTurnsOver::Retirement,
+        "crash-judge-gpu-retirement-containers-of-two-one-two-and-one-records-ring-turned",
+        "retirement-containers-of-two-one-two-and-one-records-with-the-root-ring-turned-over",
+    );
+}
````

## gpu-ring-turn：mutations-append.tsv

````
第59批 L1 GPU 判器：旋钮那一支只判倒数第二片（墓碑树 [2, 1, 2, 1]、根环转过一圈；两条腿 K1-3）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (d_container_record_count(container_index) != knob_records) {	        if (container_index + 2u == d_container_count && d_container_record_count(container_index) != knob_records) {	-p singlefs-checker-tier --release --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over	the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over
第59批 L1 GPU 判器：旋钮那一支只判倒数第二片（谱系树 [2, 1, 2, 1]、根环转过一圈；两条腿 K1-3）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (d_container_record_count(container_index) != knob_records) {	        if (container_index + 2u == d_container_count && d_container_record_count(container_index) != knob_records) {	-p singlefs-checker-tier --release --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over	the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over
第59批 L1 GPU 判器：旋钮那一支只判倒数第二片（退役树 [2, 1, 2, 1]、根环转过一圈；两条腿 K1-3）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (d_container_record_count(container_index) != knob_records) {	        if (container_index + 2u == d_container_count && d_container_record_count(container_index) != knob_records) {	-p singlefs-checker-tier --release --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over	the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_two_and_one_records_with_the_root_ring_turned_over
````

## far-left-short：crates.patch

````diff
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 16368fe7..8b863346 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -17084,6 +17084,18 @@ const A_SHORT_CONTAINER_BEFORE_A_FULL_ONE: ContainersUnderChangingKnobs =
         root_ring: RootRingBeforeTheJudgement::TurnedOverByCreatingInodes,
     };
 
+/// 少一条的那片离最右三片：旋钮 2、2、1、1、2、2、2、1 写出 [2, 1, 2, 2, 1]，再把根环转过一圈（第 59 批 L2）。上面三张盘少的那片离最右
+/// 都不超过两片，只判最右之前两片（攻方 `research/prompts/m3-batch58-code-r1-opus-model/mutants.py` 的 `w-last-two`）在它们上面与正确判法同判；
+/// 这一张的容器 2 在倒数第三片之前，那一形不判它、照绿，正确判法红在容器 2。写的中途 [2, 1, 1]、[2, 1, 2]、[2, 1, 2, 1] 那几版里容器 2
+/// 离最右不超过两片，那几版留在候选集里时那一形也红，所以要转出去。
+const A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST: ContainersUnderChangingKnobs =
+    ContainersUnderChangingKnobs {
+        records_per_container_of_each_record: &[2, 2, 1, 1, 2, 2, 2, 1],
+        record_counts_in_leaf_order: &[2, 1, 2, 2, 1],
+        short_container: 2,
+        root_ring: RootRingBeforeTheJudgement::TurnedOverByCreatingInodes,
+    };
+
 /// 第 58 批 J3：只这一棵按 `layout` 逐条换旋钮写出三片以上的容器（另两棵照产品），带旋钮值的入口报这一棵旋钮 2：每片都不超旋钮值、
 /// 容器号从 1 逐片加一、每片至少一条，只「不是最右的容器 `short_container` 装了 1 条，不是旋钮值 2」不成立，只红这一棵的那一条（I-9.22 / I-9.37）。
 /// 第 57 批 H1 的会红用例都只有两片容器，两片时只判最左片、只判倒数第二片、只判叶序第一片与正确写法等价，这几张盘把它们分开。
@@ -17180,7 +17192,6 @@ fn retirement_containers_of_two_two_one_and_one_records_under_changing_knobs_red
 }
 
 #[test]
-#[ignore = "harness 慢档用例：根环转过一圈（30 次发布）再逐版判，debug 下单条跑过 60 秒"]
 fn tombstone_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
 ) {
     assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
@@ -17191,7 +17202,6 @@ fn tombstone_containers_of_two_one_two_and_one_records_under_changing_knobs_redd
 }
 
 #[test]
-#[ignore = "harness 慢档用例：根环转过一圈（30 次发布）再逐版判，debug 下单条跑过 60 秒"]
 fn lineage_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
 ) {
     assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
@@ -17202,7 +17212,6 @@ fn lineage_containers_of_two_one_two_and_one_records_under_changing_knobs_redden
 }
 
 #[test]
-#[ignore = "harness 慢档用例：根环转过一圈（30 次发布）再逐版判，debug 下单条跑过 60 秒"]
 fn retirement_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
 ) {
     assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
@@ -17212,6 +17221,36 @@ fn retirement_containers_of_two_one_two_and_one_records_under_changing_knobs_red
     );
 }
 
+#[test]
+fn tombstone_containers_of_two_one_two_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+) {
+    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+        ContainerTreeOfTheCapacityInvariants::Tombstone,
+        "known-bad-tombstone-containers-of-two-one-two-two-and-one-records",
+        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
+    );
+}
+
+#[test]
+fn lineage_containers_of_two_one_two_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+) {
+    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+        ContainerTreeOfTheCapacityInvariants::Lineage,
+        "known-bad-lineage-containers-of-two-one-two-two-and-one-records",
+        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
+    );
+}
+
+#[test]
+fn retirement_containers_of_two_one_two_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+) {
+    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+        ContainerTreeOfTheCapacityInvariants::Retirement,
+        "known-bad-retirement-containers-of-two-one-two-two-and-one-records",
+        &A_SHORT_CONTAINER_THREE_CONTAINERS_LEFT_OF_THE_RIGHTMOST,
+    );
+}
+
 /// G3 ③：退役容器压到一片 2 条写出两条退役记录（一片 2 条），带旋钮值的入口报退役旋钮 1：那片装了 2 条、多于旋钮值，只红 I-9.37；
 /// 报写的时候那个旋钮值 2 一条不红。
 #[test]
````

## far-left-short：mutations-append.tsv

````
第59批 L2 I-9.22：旋钮支只判最右之前两片（墓碑树那一形，攻方 w-last-two；少一条的那片离最右三片 [2, 1, 2, 2, 1]、根环转过一圈）	crates/singlefs-checker/src/walk.rs	                let is_the_rightmost_tombstone_container_under_the_knob =\n                    position_in_leaf_order + 1 == containers.len();	                let is_the_rightmost_tombstone_container_under_the_knob =\n                    position_in_leaf_order + 1 == containers.len() || position_in_leaf_order + 3 < containers.len();	-p singlefs-harness --test checker_known_bad_images -- two_one_two_two_and_one_records_under_changing_knobs	tombstone_containers_of_two_one_two_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two
第59批 L2 I-9.37：旋钮支只判最右之前两片（谱系树那一形，攻方 w-last-two；少一条的那片离最右三片 [2, 1, 2, 2, 1]、根环转过一圈）	crates/singlefs-checker/src/walk.rs	                let is_the_rightmost_under_the_knob =\n                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len();	                let is_the_rightmost_under_the_knob =\n                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len() || position_in_leaf_order + 3 < container_numbers_and_record_counts.len();	-p singlefs-harness --test checker_known_bad_images -- two_one_two_two_and_one_records_under_changing_knobs	lineage_containers_of_two_one_two_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two
第59批 L2 I-9.37：旋钮支只判最右之前两片（退役树那一形，攻方 w-last-two；少一条的那片离最右三片 [2, 1, 2, 2, 1]、根环转过一圈）	crates/singlefs-checker/src/walk.rs	                let is_the_rightmost_under_the_knob =\n                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len();	                let is_the_rightmost_under_the_knob =\n                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len() || position_in_leaf_order + 3 < container_numbers_and_record_counts.len();	-p singlefs-harness --test checker_known_bad_images -- two_one_two_two_and_one_records_under_changing_knobs	retirement_containers_of_two_one_two_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two
````

## rebuild-order-watermark：crates.patch

````diff
diff --git a/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs b/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs
index 0336883..4d003e1 100644
--- a/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs
+++ b/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs
@@ -26,8 +26,8 @@ use singlefs_core::mounted_read::{
 };
 use singlefs_core::pointer::NodePointer;
 use singlefs_core::records::{
-    AccountingEntry, TreeTableEntry, STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING,
-    TREE_KIND_ALLOCATION, TREE_KIND_INODE,
+    AccountingEntry, TreeTableEntry, STATISTIC_ALLOCATED_BYTES, STATISTIC_INODE_WATERMARK,
+    TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_INODE,
 };
 use singlefs_core::recovery::{
     allocation_records_under_root, choose_system_configuration, rebuild_version, recover,
@@ -537,8 +537,13 @@ fn a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged
 
 /// 从盘上重建一版要它自己那一行 inode 号水位（统计量 12、代段 = 根的 txg），不认台阶顶替（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」；
 /// I-3.13 记账行代段等于这一版的 txg）：第一个文件（txg 3，它那一行是第 1 级台阶）之后覆盖写一版（txg 4：池级与每盘的行换成代段 4，
-/// 照抄台阶 (12, 3)，加自己那一行 (12, 4)，16 行）。把 txg 4 那一版自己那一行的统计量标签改成 13（仍排在末尾；不算台阶的行仍是 3 + 6 × 2），
-/// 那一版的根槽原地换成指着改过的树表的那一条：这一版只剩台阶 (12, 3) 那一行水位。
+/// 照抄台阶 (12, 3)，加自己那一行 (12, 4)，16 行）。txg 4 那一版自己那一行删掉，换进上一版（txg 3）那一行盘 0 的已分配字节 (1, 0, 0, 3)，
+/// 原样带过来，那一版的根槽原地换成指着改过的树表的那一条：这一版只剩台阶 (12, 3) 那一行水位。
+/// 换进一行是为了让数记账条目照过：那一判不算台阶行（统计量 12、代段早于根的 txg），只删自己那一行时不算台阶的行剩 14 ≠ 3 + 6 × 2，
+/// 重建在数条目那一判先拒、到不了水位行那一判。数得进去、又不是这一版水位行的只能是统计量不为 12 的一行，而这一版里标签登记过、维度对、
+/// 代段等于 txg 的 14 个 key 都已经有行；换进来的这一行标签（1）、树段（0）、设备段（盘 0）都照 D5 已定项 8 那张表，代段 3 不晚于根、seq 不为 0，
+/// 记账条目宽、逐条判代与 seq、数条目三判照过，它只违反 I-3.13（非台阶行的代段等于这一版的 txg），重建今天不判那一条（池级 checker 判）。
+/// 不靠没登记的统计量标签：读者逐条判拒标签 > 12、数条目时不算标签 > 12 的行，这条照报水位行缺。
 /// 从盘上重建 txg 4 那一版（`rebuild_version`，可写挂载与回退建上一版走的那一份）判它坏：记账树里没有这一版的水位行；
 /// 可写挂载建不出所选那一版，这个池系统配置里见证的 journal 计数器是 mkfs 写的 0，F2 乙不退
 /// （D23（journal 的角色与格式） 已定项 14「c_见证 = 0 … 都不退」），在任何写之前拒、报同一个失败。冷读文件内容的走读不读水位，不在这条用例里。
@@ -573,15 +578,38 @@ fn a_version_whose_own_inode_number_watermark_row_is_damaged_leaving_only_a_step
         ),
         "覆盖写那一版：16 行，水位那一统计量是台阶 (12, 3) 与它自己那一行 (12, 4)"
     );
+    let allocated_bytes_row_of_device_zero_from_the_first_version = *pool
+        .first
+        .accounting_entries
+        .iter()
+        .find(|row| row.statistic == STATISTIC_ALLOCATED_BYTES && row.device == DeviceIdentity(0))
+        .expect("第一个文件那一版每盘一行已分配字节");
+    assert_eq!(
+        allocated_bytes_row_of_device_zero_from_the_first_version.generation,
+        pool.first.root.checkpoint_txg,
+        "上一版那一行的代段是上一版的 txg（3）"
+    );
     let damaged_root =
         root_whose_accounting_rows_are_rewritten(&mut pool.image, &overwrite.root, |rows| {
-            let own_row = rows.last_mut().expect("记账树恒有池级 3 行");
+            let own_row = rows.pop().expect("记账树恒有池级 3 行");
             assert_eq!(
                 (own_row.statistic, own_row.generation),
                 (STATISTIC_INODE_WATERMARK, overwrite_txg),
                 "末行是这一版自己那一行水位"
             );
-            own_row.statistic = STATISTIC_INODE_WATERMARK + 1;
+            rows.push(allocated_bytes_row_of_device_zero_from_the_first_version);
+            rows.sort_by_key(AccountingEntry::sort_key);
+            assert_eq!(
+                (
+                    rows.len(),
+                    rows.iter()
+                        .filter(|row| row.statistic == STATISTIC_INODE_WATERMARK)
+                        .map(|row| row.generation)
+                        .collect::<Vec<_>>()
+                ),
+                (16, vec![CheckpointTxg(3)]),
+                "改过的这一版仍是 16 行，水位那一统计量只剩台阶 (12, 3)"
+            );
         });
     let ring_slot = target_for_publish(overwrite_txg, pool.geometry.root_ring_slots_per_region);
     let device_holding_the_root = DeviceIdentity(
diff --git a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
index 994df9c..85407b6 100644
--- a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
+++ b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
@@ -2111,9 +2111,10 @@ fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_
 
 /// 场景：第一个文件之后再建一个空 inode（txg 4）；记账树根里这一版自己那一行 inode 号水位的代改成 5（晚于根；台阶行不碰，记账条目数照旧对得上），
 /// 罩这一版树表单元那个槽的两条分配记录代改成 5，晚于根；指着改过的节点的每一道校验和重算。
-/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度排在记账条目宽、
+/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度排在
 /// 记账条目逐条判代与 seq 之前（冷走读那一段的次序注释与重建相同：分配记录、条目宽、逐条判、数）。
-/// 那一判挪到记账条目逐条判之后（仍在两个数之前），这份镜像报成「记账条目的代晚于根或 seq 为 0」。
+/// 那一判挪到记账条目逐条判之后（仍在两个数之前），这份镜像报成「记账条目的代晚于根或 seq 为 0」。这张盘每条记账条目宽都是 34，
+/// 条目宽那一判不坏，「分配记录排在条目宽之前」这半句它钉不到。可写挂载那一条在下面。
 #[test]
 fn allocation_records_whose_generation_is_after_the_root_are_refused_before_each_accounting_entry_is_judged_by_the_recovery(
 ) {
@@ -2130,6 +2131,23 @@ fn allocation_records_whose_generation_is_after_the_root_are_refused_before_each
     );
 }
 
+/// 场景：同上一条的坏镜像，交给可写挂载。
+/// 预期：可写挂载在任何写之前交回「分配记录跨度为 0，或分配代 / 释放代晚于根」：重建那一版逐条判分配记录在读记账树之前，
+/// 排在记账条目逐条判代与 seq 之前，与冷走读报同一个成员。重建把那一判挪到记账条目逐条判之后（仍在数记账条目之前），
+/// 这份镜像报成「记账条目的代晚于根或 seq 为 0」，而上面两张盘的可写挂载照报分配记录：那两张盘坏的是两个数，挪到两个数之前它们分不出
+/// （攻方原型 `research/prompts/m3-batch58-code-r1-opus-output.md`「K3-3」一节的 `r-rebuild-alloc-after-each-entry`）。
+#[test]
+fn allocation_records_whose_generation_is_after_the_root_are_refused_before_each_accounting_entry_is_judged_by_the_writable_mount(
+) {
+    let mut devices =
+        pool_whose_allocation_records_and_one_accounting_entry_carry_a_generation_after_the_root();
+    let refusal = writable_mount_refused_before_any_write(&mut devices);
+    assert!(
+        matches!(&refusal, MountError::Recovery(failure) if *failure == allocation_record_span_zero_or_generation_after_the_root()),
+        "可写挂载报分配记录的代晚于根，不报记账条目的代：{refusal:?}"
+    );
+}
+
 // ─── 第 35 条续：单元区槽数那一处减法 ───
 
 /// 场景：700 MiB 的盘（单元区起点在 784 MiB）。预期：单元区槽数交回「盘末尾在单元区起点之前」，带盘的字节数与起点槽号；
````

## rebuild-order-watermark：mutations-append.tsv

````
第59批 L3 重建分配记录代与跨度挪到记账条目宽与逐条判之后、紧贴数记账条目（代晚于根的分配记录在一条记账条目的代也晚于根的镜像上可写挂载报成记账条目的代）	crates/singlefs-core/src/recovery.rs	    allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;\n    let allocation_records: Vec<AllocationRecord> = allocation_tree_read.records.clone();\n    // 记账树可以是多层（D8（核心索引结构） 已定项 11）：从根往下整棵读回来，节点进映射、提示读不出经这一版的中央映射回退\n    // （与读别的树根同一条）；中央映射树同样整棵读回来（下面）。两棵树照冷走读判全（`code_two_tree::read_code_two_tree` 的\n    // `EveryHeaderAgainstItsReference`：节点头、层级、区间与 key 严格递增——有重复 key 的记账叶交出去，挂载之后第一次发布规划那棵树时\n    // 在 `transaction` 的断言上 panic，代码审阅第 33 条）。\n    let accounting_tree = read_code_two_tree(\n        &accounting_pointer,\n        &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),\n        CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,\n        root,\n        expected_filesystem_identifier,\n        &mut |pointer: &NodePointer| {\n            read_mapped_tree_node_via_hint_then_central_mapping(\n                reader,\n                pointer,\n                MappedTreeNodeClass::IndexNode,\n                &central_mapping_locations_of_key,\n                &mut stale_location_hint_hops_not_exposed_by_this_reader,\n            )\n        },\n    )?;\n    // 次序：条目宽、逐条判代与 seq、数条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`\n    // 第三节 G1）。数条目要按统计量与代段认台阶行，代写到根之后的台阶行认不出、被算进 3 + 6 × 盘数，「代晚于根」这一坏法就报成了条目数不对。\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &accounting_tree.leaf_entries_in_key_order,\n    )?;\n    let mut accounting_entries: Vec<AccountingEntry> =\n        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());\n    for bytes in &accounting_tree.leaf_entries_in_key_order {\n        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(\n            bytes, root,\n        )?);\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n	    let allocation_records: Vec<AllocationRecord> = allocation_tree_read.records.clone();\n    // 记账树可以是多层（D8（核心索引结构） 已定项 11）：从根往下整棵读回来，节点进映射、提示读不出经这一版的中央映射回退\n    // （与读别的树根同一条）；中央映射树同样整棵读回来（下面）。两棵树照冷走读判全（`code_two_tree::read_code_two_tree` 的\n    // `EveryHeaderAgainstItsReference`：节点头、层级、区间与 key 严格递增——有重复 key 的记账叶交出去，挂载之后第一次发布规划那棵树时\n    // 在 `transaction` 的断言上 panic，代码审阅第 33 条）。\n    let accounting_tree = read_code_two_tree(\n        &accounting_pointer,\n        &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),\n        CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,\n        root,\n        expected_filesystem_identifier,\n        &mut |pointer: &NodePointer| {\n            read_mapped_tree_node_via_hint_then_central_mapping(\n                reader,\n                pointer,\n                MappedTreeNodeClass::IndexNode,\n                &central_mapping_locations_of_key,\n                &mut stale_location_hint_hops_not_exposed_by_this_reader,\n            )\n        },\n    )?;\n    // 次序：条目宽、逐条判代与 seq、数条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`\n    // 第三节 G1）。数条目要按统计量与代段认台阶行，代写到根之后的台阶行认不出、被算进 3 + 6 × 盘数，「代晚于根」这一坏法就报成了条目数不对。\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &accounting_tree.leaf_entries_in_key_order,\n    )?;\n    let mut accounting_entries: Vec<AccountingEntry> =\n        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());\n    for bytes in &accounting_tree.leaf_entries_in_key_order {\n        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(\n            bytes, root,\n        )?);\n    }\n    allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;\n    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- allocation_records_whose_generation_is_after_the_root	allocation_records_whose_generation_is_after_the_root_are_refused_before_each_accounting_entry_is_judged_by_the_writable_mount
````

## rebuild-order-watermark：mutations-replacements.tsv

````
第58批 J1 重建分配记录代与跨度挪到数映射条目之后（代晚于根的分配记录在映射条目数对不上的镜像上可写挂载报成映射条目数）	crates/singlefs-core/src/recovery.rs	    allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;\n    let allocation_records: Vec<AllocationRecord> = allocation_tree_read.records.clone();\n    // 记账树可以是多层（D8（核心索引结构） 已定项 11）：从根往下整棵读回来，节点进映射、提示读不出经这一版的中央映射回退\n    // （与读别的树根同一条）；中央映射树同样整棵读回来（下面）。两棵树照冷走读判全（`code_two_tree::read_code_two_tree` 的\n    // `EveryHeaderAgainstItsReference`：节点头、层级、区间与 key 严格递增——有重复 key 的记账叶交出去，挂载之后第一次发布规划那棵树时\n    // 在 `transaction` 的断言上 panic，代码审阅第 33 条）。\n    let accounting_tree = read_code_two_tree(\n        &accounting_pointer,\n        &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),\n        CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,\n        root,\n        expected_filesystem_identifier,\n        &mut |pointer: &NodePointer| {\n            read_mapped_tree_node_via_hint_then_central_mapping(\n                reader,\n                pointer,\n                MappedTreeNodeClass::IndexNode,\n                &central_mapping_locations_of_key,\n                &mut stale_location_hint_hops_not_exposed_by_this_reader,\n            )\n        },\n    )?;\n    // 次序：条目宽、逐条判代与 seq、数条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`\n    // 第三节 G1）。数条目要按统计量与代段认台阶行，代写到根之后的台阶行认不出、被算进 3 + 6 × 盘数，「代晚于根」这一坏法就报成了条目数不对。\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &accounting_tree.leaf_entries_in_key_order,\n    )?;\n    let mut accounting_entries: Vec<AccountingEntry> =\n        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());\n    for bytes in &accounting_tree.leaf_entries_in_key_order {\n        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(\n            bytes, root,\n        )?);\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_identities.len(),\n    )?;\n    // 记账行的统计量标签是盘上读来的 2 字节：这一版交出去之前判一次「inode 号水位那一行在」，往里就信\n    // （`TransactionOutput::inode_number_watermark` 那句 expect 依赖的就是这一判；panic 面普查 R11）。\n    // 要的是这一版自己写的那一行（代段 = 根的 txg）：那一统计量另有代段更早的台阶行（D5（快照 / 空间记账机制） 已定项 2 射程），\n    // 只剩台阶行时现行值取到的是一级台阶，不是这一版的水位。\n    if !accounting_entries.iter().any(|entry| {\n        entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation == root.checkpoint_txg\n    }) {\n        return Err(RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree.into());\n    }\n    // 映射树是自举豁免：不经映射回退，只按父指针里的位置条目读；上面有提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份。\n    let mapping_tree = central_mapping_root.into_tree()?;\n    mapping_entry_count_is_one_per_mapped_unit(\n        mapping_tree.leaf_entries_in_key_order.len(),\n        MappedUnitCounts {\n            extent_tree_nodes: first_file_extents.nodes_in_bump_order.len(),\n            allocation_record_tree_nodes: allocation_tree_read.nodes.len(),\n            accounting_tree_nodes: accounting_tree.version.node_count(),\n            inode_leaf_containers: inode_root_node.entries.len(),\n            data_units: data_pointers.len(),\n            deletion_tree_nodes: usize::from(deleted_inodes_root_bytes.is_some())\n                + tombstone_tree_read\n                    .as_ref()\n                    .map_or(0, ContainerTreeRead::mapped_units)\n                + lineage_tree_read\n                    .as_ref()\n                    .map_or(0, ContainerTreeRead::mapped_units)\n                + retirement_tree_read\n                    .as_ref()\n                    .map_or(0, ContainerTreeRead::mapped_units),\n        },\n    )?;\n	    let allocation_records: Vec<AllocationRecord> = allocation_tree_read.records.clone();\n    // 记账树可以是多层（D8（核心索引结构） 已定项 11）：从根往下整棵读回来，节点进映射、提示读不出经这一版的中央映射回退\n    // （与读别的树根同一条）；中央映射树同样整棵读回来（下面）。两棵树照冷走读判全（`code_two_tree::read_code_two_tree` 的\n    // `EveryHeaderAgainstItsReference`：节点头、层级、区间与 key 严格递增——有重复 key 的记账叶交出去，挂载之后第一次发布规划那棵树时\n    // 在 `transaction` 的断言上 panic，代码审阅第 33 条）。\n    let accounting_tree = read_code_two_tree(\n        &accounting_pointer,\n        &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),\n        CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,\n        root,\n        expected_filesystem_identifier,\n        &mut |pointer: &NodePointer| {\n            read_mapped_tree_node_via_hint_then_central_mapping(\n                reader,\n                pointer,\n                MappedTreeNodeClass::IndexNode,\n                &central_mapping_locations_of_key,\n                &mut stale_location_hint_hops_not_exposed_by_this_reader,\n            )\n        },\n    )?;\n    // 映射树是自举豁免：不经映射回退，只按父指针里的位置条目读；上面有提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份。\n    let mapping_tree = central_mapping_root.into_tree()?;\n    mapping_entry_count_is_one_per_mapped_unit(\n        mapping_tree.leaf_entries_in_key_order.len(),\n        MappedUnitCounts {\n            extent_tree_nodes: first_file_extents.nodes_in_bump_order.len(),\n            allocation_record_tree_nodes: allocation_tree_read.nodes.len(),\n            accounting_tree_nodes: accounting_tree.version.node_count(),\n            inode_leaf_containers: inode_root_node.entries.len(),\n            data_units: data_pointers.len(),\n            deletion_tree_nodes: usize::from(deleted_inodes_root_bytes.is_some())\n                + tombstone_tree_read\n                    .as_ref()\n                    .map_or(0, ContainerTreeRead::mapped_units)\n                + lineage_tree_read\n                    .as_ref()\n                    .map_or(0, ContainerTreeRead::mapped_units)\n                + retirement_tree_read\n                    .as_ref()\n                    .map_or(0, ContainerTreeRead::mapped_units),\n        },\n    )?;\n    allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;\n    // 次序：条目宽、逐条判代与 seq、数条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`\n    // 第三节 G1）。数条目要按统计量与代段认台阶行，代写到根之后的台阶行认不出、被算进 3 + 6 × 盘数，「代晚于根」这一坏法就报成了条目数不对。\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &accounting_tree.leaf_entries_in_key_order,\n    )?;\n    let mut accounting_entries: Vec<AccountingEntry> =\n        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());\n    for bytes in &accounting_tree.leaf_entries_in_key_order {\n        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(\n            bytes, root,\n        )?);\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_identities.len(),\n    )?;\n    // 记账行的统计量标签是盘上读来的 2 字节：这一版交出去之前判一次「inode 号水位那一行在」，往里就信\n    // （`TransactionOutput::inode_number_watermark` 那句 expect 依赖的就是这一判；panic 面普查 R11）。\n    // 要的是这一版自己写的那一行（代段 = 根的 txg）：那一统计量另有代段更早的台阶行（D5（快照 / 空间记账机制） 已定项 2 射程），\n    // 只剩台阶行时现行值取到的是一级台阶，不是这一版的水位。\n    if !accounting_entries.iter().any(|entry| {\n        entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation == root.checkpoint_txg\n    }) {\n        return Err(RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree.into());\n    }\n	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- allocation_records_whose_generation_is_after_the_root	allocation_records_whose_generation_is_after_the_root_are_refused_before_the_mapping_entry_count_by_the_writable_mount
````

