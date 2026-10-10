# 删除线第 58 批（代码轮 m3-batch57-code-r1 判决的 J1–J4）：代码轮第一轮改动全文（2026-10-10）

两个实现员交的补丁原样，按打的次序（甲 → 乙），底是主工作区合入第 57 批之后的样子。两份都只改测试。

## m3-batch58/knob-multi

### crates.patch

`````diff
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -84,6 +84,8 @@
 //! 第 56 批（G2、G3、G6）：三棵容器树的旋钮值编进 GPU 扫描内核（`JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`），
 //! CPU 那一侧池级 checker 的违例位改按带同一组旋钮值的新入口判：墓碑、谱系、退役各一条旋钮下的合法流（I-9.22 / I-9.37 两边判过而成立），
 //! 谱系多于旋钮值、谱系两片同号、退役两片同号、谱系跳号（1、3）、谱系从 2 起（2、3）、谱系最右多一片 0 条各一条坏流（CPU 红 I-9.37），都整份比。
+//! 第 58 批 J3：墓碑、谱系、退役各一条旋钮按 2、2、1、1 逐条换值写出 [2, 1, 1]（中间那片少一条）的流，两边都给那一棵旋钮 2，CPU 红 I-9.22 / I-9.37，整份比；
+//! J4：墓碑产品支 [815, 1] 那条流的起点镜像是第 816 个 unlink 之前的整段录制流，写表只录那一次 unlink（从 mkfs 录起时枚举写表超过持久掩码的宽、建不出表）。
 //! 崩溃点重放合并（判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 第三节 J6、J10，
 //! `research/prompts/m3-crash-enumeration-merge-r2-main-verification.md` 第二节 L1-4、L1-6、L1-7、L6-2）：每个状态比的是对拍字
 //! （`crash_judge_gpu::CrossCheckWords`：红项、两遍各自落到的根、结局、失败点、池级 checker 判到位、判不了）；甲二展开的一条流上
@@ -13121,16 +13123,32 @@
     lineage_records_per_container: u64,
     rollbacks: usize,
 ) -> RecordedFlow {
+    let pool = pool_rolled_back_under_these_lineage_knobs(
+        tag,
+        &vec![lineage_records_per_container; rollbacks],
+    );
+    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
+    recorded_flow_of_the_pool(&pool, name, versions)
+}
+
+/// 新池新建文件、建 1 个空 inode（txg 4），挂着回退到第一个文件那一版（txg 3），每次回退之前分配器换上谱系容器一片
+/// `lineage_records_per_container_of_each_rollback` 里那一项条的旋钮（回退的发布路径照分配器上的开关取容量）：旋钮可以在中途换值（第 58 批 J3）。
+fn pool_rolled_back_under_these_lineage_knobs(
+    tag: &str,
+    lineage_records_per_container_of_each_rollback: &[u64],
+) -> BuiltPool {
     let mut pool = build_pool(tag);
     let first_file_version = pool.output.clone();
     common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
-    pool.allocator
-        .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
+    // 迭代上界：切片的长度。
+    for lineage_records_per_container in lineage_records_per_container_of_each_rollback {
+        pool.allocator
+            .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
             batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
             tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
             lineage_container_capacity: LineageContainerCapacity::CappedForTests(
                 singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(
-                    lineage_records_per_container,
+                    *lineage_records_per_container,
                 )
                 .expect("1..=1359"),
             ),
@@ -13138,12 +13156,9 @@
                 singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
             container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
         });
-    // 迭代上界：`rollbacks`。
-    for _ in 0..rollbacks {
         roll_back_to(&mut pool, &first_file_version);
     }
-    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
-    recorded_flow_of_the_pool(&pool, name, versions)
+    pool
 }
 
 /// G3 ③ 在 GPU 上（I-9.37 多于旋钮值那一判）：谱系容器一片 2 条写出两条谱系记录（一片 2 条），GPU 与 CPU 新入口都给谱系旋钮 1：那片装了 2 条，
@@ -13724,11 +13739,12 @@
 }
 
 /// H2 产品支墓碑那一个调用点要的流：新池新建文件、再建 816 个空 inode（一次发布），照产品容量逐个 unlink 前 815 个到底（墓碑树一片装满 815 条），
-/// 再 unlink 第 816 个（意图那次开第二片、1 条）：墓碑树 [815, 1]，产品镜像合法。交回整条流（从 mkfs 之后录）与第 816 个 unlink 的第一次写
-/// 在写表里的下标——之前的段整段持久、不展开，状态只从那一次 unlink 起取（前 815 次 unlink 约 1630 次发布，逐段展开状态数太大）。
+/// 再 unlink 第 816 个（意图那次开第二片、1 条）：墓碑树 [815, 1]，产品镜像合法。起点镜像是 mkfs 到第 816 个 unlink 之前的整段录制流施加出来的那一份，
+/// 写表只录第 816 个 unlink 那几次发布（第 58 批 J4：前 815 次 unlink 约 1630 次发布，整条流从 mkfs 录起时它们的写也进枚举写表、
+/// 超过判器持久掩码的宽，建表交 `EnumeratedWritesPastTheMaskWidth`；之前的段在原来那一造法里本就整段持久、不展开，崩溃状态与这里相同）。
 fn record_a_second_tombstone_container_opened_after_a_full_one_at_product_capacity(
     tag: &str,
-) -> (RecordedFlow, usize) {
+) -> RecordedFlow {
     let tombstones_filling_one_container = singlefs_format::TOMBSTONE_CONTAINER_RECORDS;
     let mut pool = pool_with_new_inodes(tag, tombstones_filling_one_container + 1);
     // 迭代上界：815 个号。
@@ -13739,8 +13755,7 @@
             ContainerTreeCapacities::FROM_THE_FORMAT,
         );
     }
-    let operations_before_the_last_unlink =
-        pool.stream.operation_count() - pool.mkfs_operation_count;
+    let operations_before_the_last_unlink = pool.stream.operation_count();
     unlink_an_empty_inode_to_the_end_under_the_capacities(
         &mut pool,
         FIRST_FILE.0 + tombstones_filling_one_container + 1,
@@ -13762,61 +13777,43 @@
     );
     let versions = versions_with_the_first_file_through_the_last_publish(&pool);
     let operations = pool.retained_operations();
-    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
-        &operations[pool.mkfs_operation_count..],
+    let mut base = pool.memory_pool_after_mkfs();
+    base.apply(&operations[pool.mkfs_operation_count..operations_before_the_last_unlink]);
+    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
+        &operations[operations_before_the_last_unlink..],
         &geometry(),
     );
-    let first_write_of_the_last_unlink = stream_indexes
-        .iter()
-        .position(|stream_index| *stream_index >= operations_before_the_last_unlink)
-        .expect("第 816 个 unlink 有写");
     let judged_root_index = writes
         .iter()
         .rposition(|write| write.kind == StepKind::RootRecordFua)
-        .expect("发布有根槽写");
-    (
-        RecordedFlow {
-            name: "product-tombstone-containers-after-a-full-one",
-            base: pool.memory_pool_after_mkfs(),
-            writes,
-            segments,
-            judged_root_index,
-            versions,
-        },
-        first_write_of_the_last_unlink,
-    )
+        .expect("第 816 个 unlink 的发布有根槽写");
+    RecordedFlow {
+        name: "product-tombstone-containers-after-a-full-one",
+        base,
+        writes,
+        segments,
+        judged_root_index,
+        versions,
+    }
 }
 
 /// H2（GPU 产品支墓碑那一个调用点，攻方 `mut-w-gap2` 那一形）：产品容量下墓碑树 [815, 1]，第二片容器与根里指它的条目的容器号 2 → 3（1、3）：
-/// 第一片装满、容量那句成立，CPU 产品入口只因「相邻两片加一」红 I-9.21，GPU 红在同样的状态、同样的位上。只展开第 816 个 unlink 的那几段
-/// （之前的段整段持久）；GPU 扫描内核产品支不调三判（`d_judge_tombstone_container_capacity` 里那一个调用点）就在这里与 CPU 不一致。
+/// 第一片装满、容量那句成立，CPU 产品入口只因「相邻两片加一」红 I-9.21，GPU 红在同样的状态、同样的位上。写表只录第 816 个 unlink 那几次发布
+/// （之前的录制流施加成起点镜像，第 58 批 J4），每段照展开上限展开；GPU 扫描内核产品支不调三判（`d_judge_tombstone_container_capacity` 里那一个调用点）
+/// 就在这里与 CPU 不一致。
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_when_product_tombstone_containers_after_a_full_one_skip_a_number(
 ) {
     const TEST_NAME: &str =
         "the_gpu_judge_agrees_with_the_cpu_when_product_tombstone_containers_after_a_full_one_skip_a_number";
-    let (mut flow, first_write_of_the_last_unlink) =
-        record_a_second_tombstone_container_opened_after_a_full_one_at_product_capacity(
-            "crash-judge-gpu-i921-product-containers-skip-a-number",
-        );
+    let mut flow = record_a_second_tombstone_container_opened_after_a_full_one_at_product_capacity(
+        "crash-judge-gpu-i921-product-containers-skip-a-number",
+    );
     renumber_the_last_container_of_the_flow(&mut flow, PACKED_RECORD_TYPE_TOMBSTONE, 1, 3);
     let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
         return;
     };
-    let limit = expand_up_to();
-    let expansion = move |_segment_index: usize, segment: &[usize]| {
-        let is_a_segment_of_the_last_unlink = segment
-            .first()
-            .is_some_and(|first_write| *first_write >= first_write_of_the_last_unlink);
-        if is_a_segment_of_the_last_unlink && segment.len() <= limit {
-            Layer0SegmentExpansion::EveryProperSubset
-        } else {
-            Layer0SegmentExpansion::NotExpanded
-        }
-    };
-    let Some(comparison) =
-        compare_on_every_state_of_these_segments(cards, &flow, &flow.segments, &expansion)
-    else {
+    let Some(comparison) = compare_on_every_state(cards, &flow, expand_up_to()) else {
         return;
     };
     assert!(
@@ -13833,3 +13830,196 @@
         mismatch_listing(&comparison)
     );
 }
+
+// ───────────── 第 58 批 J3（判决 `research/prompts/m3-batch57-code-r1-main-verification.md` 第三节；两条腿 K4）：旋钮支三片、中间那片少一条 ─────────────
+
+/// 写的时候逐条换的旋钮值：2、2、1、1 写出 [2, 1, 1]（攻方 `research/prompts/m3-batch57-code-r1-opus-model/probe-crate/tests/k4_middle_short_container.rs` 那一形）。
+/// 写侧每一步都照当时的旋钮满了才关；第 57 批 H1 那几条流都只有两片容器，两片时「只判叶序第一片」与正确写法等价，三片时分得开。
+const KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER: [u64; 4] = [2, 2, 1, 1];
+/// 按叶序每片的记录数。
+const RECORD_COUNTS_OF_A_SHORT_MIDDLE_CONTAINER: [usize; 3] = [2, 1, 1];
+/// 判的时候编进 GPU 扫描内核、交给 CPU 新入口的旋钮值：容器 2 不是最右那片、只装 1 条，不是旋钮值。
+const KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER: u64 = 2;
+
+/// 只墓碑树压到 `records_per_container` 条一片的写入口（另两棵与节点照产品）。
+fn capacities_of_the_tombstone_container_knob(
+    records_per_container: u64,
+) -> ContainerTreeCapacities {
+    ContainerTreeCapacities {
+        tombstone_container: TombstoneContainerCapacity::CappedForTests(
+            TombstoneRecordsPerContainerCappedForTests::new(records_per_container)
+                .expect("1..=815"),
+        ),
+        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
+        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
+    }
+}
+
+/// 只退役树压到 `records_per_container` 条一片的写入口（另两棵与节点照产品）。
+fn capacities_of_the_retirement_container_knob(
+    records_per_container: u64,
+) -> ContainerTreeCapacities {
+    ContainerTreeCapacities {
+        tombstone_container: TombstoneContainerCapacity::FromTheContainerFormat,
+        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
+                singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(
+                    records_per_container,
+                )
+                .expect("1..=959"),
+            ),
+        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
+    }
+}
+
+/// 墓碑树那一形：新池新建文件、建 4 个空 inode（2–5 号），逐个 unlink 到底，写入口按
+/// [`KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER`] 逐个换墓碑旋钮：墓碑树 [2, 1, 1]。
+fn record_tombstone_containers_of_two_one_and_one_records_under_changing_knobs(
+    tag: &str,
+) -> RecordedFlow {
+    let mut pool = pool_with_new_inodes(
+        tag,
+        u64::try_from(KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER.len())
+            .expect("装得进 u64"),
+    );
+    // 迭代上界：4 个号。
+    for (inode, records_per_container) in
+        (FIRST_FILE.0 + 1..).zip(KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER)
+    {
+        unlink_an_empty_inode_to_the_end_under_the_capacities(
+            &mut pool,
+            inode,
+            capacities_of_the_tombstone_container_knob(records_per_container),
+        );
+    }
+    let tombstone_record_counts: Vec<usize> = pool
+        .output
+        .tombstone_containers
+        .iter()
+        .map(|container| container.contents.records.len())
+        .collect();
+    assert_eq!(
+        tombstone_record_counts, RECORD_COUNTS_OF_A_SHORT_MIDDLE_CONTAINER,
+        "墓碑树按叶序每片的记录数"
+    );
+    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
+    recorded_flow_of_the_pool(
+        &pool,
+        "tombstone-containers-of-two-one-and-one-records-under-changing-knobs",
+        versions,
+    )
+}
+
+/// 谱系树那一形：新池新建文件、建 1 个空 inode，挂着回退到第一个文件那一版 4 次，每次回退之前按
+/// [`KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER`] 换谱系旋钮：谱系树 [2, 1, 1]。
+fn record_lineage_containers_of_two_one_and_one_records_under_changing_knobs(
+    tag: &str,
+) -> RecordedFlow {
+    let pool = pool_rolled_back_under_these_lineage_knobs(
+        tag,
+        &KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER,
+    );
+    let lineage_record_counts: Vec<usize> = pool
+        .output
+        .lineage_containers
+        .iter()
+        .map(|container| container.contents.records.len())
+        .collect();
+    assert_eq!(
+        lineage_record_counts, RECORD_COUNTS_OF_A_SHORT_MIDDLE_CONTAINER,
+        "谱系树按叶序每片的记录数"
+    );
+    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
+    recorded_flow_of_the_pool(
+        &pool,
+        "lineage-containers-of-two-one-and-one-records-under-changing-knobs",
+        versions,
+    )
+}
+
+/// 退役树那一形：新池新建文件之外再建 233 个空 inode（叶 [1..=233]、[234]），unlink 234 到底（最右那片叶删空、退役一条），
+/// 之后每次建 1 个（开新叶）、unlink 它到底（又退役一条），共退役 4 次，写入口按 [`KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER`]
+/// 逐次换退役旋钮：退役树 [2, 1, 1]。
+fn record_retirement_containers_of_two_one_and_one_records_under_changing_knobs(
+    tag: &str,
+) -> RecordedFlow {
+    let mut pool = pool_with_new_inodes(tag, INODE_RECORDS_PER_LEAF);
+    // 迭代上界：4 次退役。第 0 次删的是建出来的最后一个号（独占最右那片叶），之后每次先建一个再删它。
+    for (retirement_index, records_per_container) in
+        KNOB_RECORDS_OF_EACH_RECORD_WRITING_A_SHORT_MIDDLE_CONTAINER
+            .into_iter()
+            .enumerate()
+    {
+        if retirement_index > 0 {
+            common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+        }
+        let inode = pool.output.inode_number_watermark() - 1;
+        unlink_an_empty_inode_to_the_end_under_the_capacities(
+            &mut pool,
+            inode,
+            capacities_of_the_retirement_container_knob(records_per_container),
+        );
+    }
+    let retirement_record_counts: Vec<usize> = pool
+        .output
+        .retirement_containers
+        .iter()
+        .map(|container| container.contents.records.len())
+        .collect();
+    assert_eq!(
+        retirement_record_counts, RECORD_COUNTS_OF_A_SHORT_MIDDLE_CONTAINER,
+        "退役树按叶序每片的记录数"
+    );
+    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
+    recorded_flow_of_the_pool(
+        &pool,
+        "retirement-containers-of-two-one-and-one-records-under-changing-knobs",
+        versions,
+    )
+}
+
+/// J3（GPU 那一份）：墓碑树 [2, 1, 1]，GPU 扫描内核与 CPU 新入口都给墓碑旋钮 2：每片都不超旋钮值、容器号从 1 逐片加一，
+/// 只「不是最右的容器 2 装满旋钮值」不成立，CPU 红 I-9.22，GPU 红在同样的状态、同样的位上（GPU 只判叶序第一片就在这里与 CPU 不一致）。
+#[test]
+fn the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_and_one_records_written_under_changing_knobs(
+) {
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_and_one_records_written_under_changing_knobs",
+        &record_tombstone_containers_of_two_one_and_one_records_under_changing_knobs(
+            "crash-judge-gpu-tombstone-containers-of-two-one-and-one-records",
+        ),
+        knobs_of_the_tombstone_tree(KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER),
+        KNOB_CAPACITY_INVARIANT_OF_THE_TOMBSTONE_TREE,
+    );
+}
+
+/// J3（GPU 那一份）：谱系树 [2, 1, 1]，两边都给谱系旋钮 2，CPU 红 I-9.37，GPU 同判。
+#[test]
+fn the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_and_one_records_written_under_changing_knobs(
+) {
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_lineage_containers_of_two_one_and_one_records_written_under_changing_knobs",
+        &record_lineage_containers_of_two_one_and_one_records_under_changing_knobs(
+            "crash-judge-gpu-lineage-containers-of-two-one-and-one-records",
+        ),
+        knobs_of_the_lineage_tree(KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER),
+        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
+    );
+}
+
+/// J3（GPU 那一份）：退役树 [2, 1, 1]，两边都给退役旋钮 2，CPU 红 I-9.37，GPU 同判。
+#[test]
+fn the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_and_one_records_written_under_changing_knobs(
+) {
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_retirement_containers_of_two_one_and_one_records_written_under_changing_knobs",
+        &record_retirement_containers_of_two_one_and_one_records_under_changing_knobs(
+            "crash-judge-gpu-retirement-containers-of-two-one-and-one-records",
+        ),
+        knobs_of_the_retirement_tree(KNOB_RECORDS_JUDGING_A_SHORT_MIDDLE_CONTAINER),
+        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
+    );
+}
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -16511,13 +16511,35 @@
     records_per_container: u64,
     records: u64,
 ) -> BuiltPool {
-    let capacities = tree.write_capacities(records_per_container);
+    pool_with_records_in_one_container_tree_written_under_these_knobs(
+        tree,
+        tag,
+        &vec![records_per_container; usize::try_from(records).expect("装得进 usize")],
+    )
+}
+
+/// 同 [`pool_with_records_in_one_container_tree_under_its_knob`]，第 k 条记录写的时候写入口只这一棵压到 `records_per_container_of_each_record[k]` 条一片
+/// （另两棵照产品）：旋钮在一份镜像的历史中途换值。写侧每一步都照当时的旋钮「满了才关」，换小之后下一条开新片、换大之后接着往开放的那片里装，
+/// 写得出「不是最右的那片少一条」（第 58 批 J3，攻方 `research/prompts/m3-batch57-code-r1-opus-model/probe-crate/tests/k4_middle_short_container.rs` 那一造法）。
+fn pool_with_records_in_one_container_tree_written_under_these_knobs(
+    tree: ContainerTreeOfTheCapacityInvariants,
+    tag: &str,
+    records_per_container_of_each_record: &[u64],
+) -> BuiltPool {
+    let records =
+        u64::try_from(records_per_container_of_each_record.len()).expect("记录数装得进 u64");
     let pool = match tree {
         ContainerTreeOfTheCapacityInvariants::Tombstone => {
             let mut pool = common_deletion_in_progress::pool_with_new_inodes(tag, records);
             // 迭代上界：`records`。
-            for inode in 2..2 + records {
-                unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, inode, capacities);
+            for (inode, records_per_container) in
+                (2..2 + records).zip(records_per_container_of_each_record)
+            {
+                unlink_an_empty_inode_to_the_end_under_the_capacities(
+                    &mut pool,
+                    inode,
+                    tree.write_capacities(*records_per_container),
+                );
             }
             pool
         }
@@ -16525,16 +16547,17 @@
             let mut pool = build_pool(tag);
             let first_file_version = pool.output.clone();
             common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
-            pool.allocator
-                .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
-                    batch_size: singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
-                    tombstone_container_capacity: capacities.tombstone_container,
-                    lineage_container_capacity: capacities.lineage_container,
-                    retirement_container_capacity: capacities.retirement_container,
-                    container_tree_node_entries: capacities.node_entries,
-                });
             // 迭代上界：`records`。
-            for _ in 0..records {
+            for records_per_container in records_per_container_of_each_record {
+                let capacities = tree.write_capacities(*records_per_container);
+                pool.allocator
+                    .set_deletion_switches(singlefs_core::unlink::DeletionSwitches {
+                        batch_size: singlefs_core::unlink::UnlinkBatchSize::OneExtentLowerLeaf,
+                        tombstone_container_capacity: capacities.tombstone_container,
+                        lineage_container_capacity: capacities.lineage_container,
+                        retirement_container_capacity: capacities.retirement_container,
+                        container_tree_node_entries: capacities.node_entries,
+                    });
                 common_deletion_in_progress::roll_back_to(&mut pool, &first_file_version);
             }
             pool
@@ -16543,14 +16566,20 @@
             let leaf_records = singlefs_format::INODE_LEAF_RECORDS;
             let mut pool = common_deletion_in_progress::pool_with_new_inodes(tag, leaf_records);
             // 迭代上界：`records`。第 0 次删的是建出来的最后一个号（独占最右那片叶），之后每次先建一个再删它。
-            for retirement_index in 0..records {
+            for (retirement_index, records_per_container) in
+                records_per_container_of_each_record.iter().enumerate()
+            {
                 if retirement_index > 0 {
                     common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(
                         &mut pool, 1,
                     );
                 }
                 let inode = pool.output.inode_number_watermark() - 1;
-                unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, inode, capacities);
+                unlink_an_empty_inode_to_the_end_under_the_capacities(
+                    &mut pool,
+                    inode,
+                    tree.write_capacities(*records_per_container),
+                );
             }
             pool
         }
@@ -17005,6 +17034,184 @@
     );
 }
 
+/// 第 58 批 J3 那几张盘的旋钮值：写的时候在 2 与 1 之间换，判的时候报 2。
+const KNOB_RECORDS_OF_THE_SHORT_CONTAINER_IMAGES: u64 = 2;
+
+/// 第 58 批 J3 那几张盘怎么造（判决 `research/prompts/m3-batch57-code-r1-main-verification.md` 第三节 J3；两条腿 K4）：一片几条、按叶序每片记录数、
+/// 少一条的那片的容器号、判之前要不要把根环转过一圈。
+struct ContainersUnderChangingKnobs {
+    records_per_container_of_each_record: &'static [u64],
+    record_counts_in_leaf_order: &'static [usize],
+    short_container: u64,
+    root_ring: RootRingBeforeTheJudgement,
+}
+
+/// 判之前根环转没转过一圈。
+#[derive(Clone, Copy, Debug)]
+enum RootRingBeforeTheJudgement {
+    /// 写容器树的那几版都还在回退候选集里，池级 checker 逐版判。
+    KeepsTheVersionsThatWroteTheContainerTree,
+    /// 再建 30 次空 inode（`common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes`）：留在候选集里的每一版都是写完的那棵容器树。
+    TurnedOverByCreatingInodes,
+}
+
+/// 中间那片少一条：旋钮 2、2、1、1 写出 [2, 1, 1]（攻方 `k4_middle_short_container` 那一形）。只判叶序第一片（攻方 `h1-first-only`）、
+/// 只判最左片（正推 MB2）在它上面照绿，正确判法红在容器 2。
+const MIDDLE_CONTAINER_SHORT: ContainersUnderChangingKnobs = ContainersUnderChangingKnobs {
+    records_per_container_of_each_record: &[2, 2, 1, 1],
+    record_counts_in_leaf_order: &[2, 1, 1],
+    short_container: 2,
+    root_ring: RootRingBeforeTheJudgement::KeepsTheVersionsThatWroteTheContainerTree,
+};
+
+/// 倒数第二片之前全满、倒数第二片少一条：旋钮 2、2、2、2、1、1 写出 [2, 2, 1, 1]。只判叶序第一片、只判最左片照绿，正确判法红在容器 3。
+const ALL_FULL_BEFORE_A_SHORT_SECOND_TO_LAST_CONTAINER: ContainersUnderChangingKnobs =
+    ContainersUnderChangingKnobs {
+        records_per_container_of_each_record: &[2, 2, 2, 2, 1, 1],
+        record_counts_in_leaf_order: &[2, 2, 1, 1],
+        short_container: 3,
+        root_ring: RootRingBeforeTheJudgement::KeepsTheVersionsThatWroteTheContainerTree,
+    };
+
+/// 少一条的那片之后还有一片满的：旋钮 2、2、1、1、2、1 写出 [2, 1, 2, 1]，再把根环转过一圈。只判倒数第二片（正推 MC）在它上面照绿——
+/// 写的中途有一版是 [2, 1, 1]，容器 2 在那一版是倒数第二片，那一版留在候选集里时只判倒数第二片也红，所以要转出去；
+/// 只判叶序第一片、只判最左片同样照绿，正确判法红在容器 2。
+const A_SHORT_CONTAINER_BEFORE_A_FULL_ONE: ContainersUnderChangingKnobs =
+    ContainersUnderChangingKnobs {
+        records_per_container_of_each_record: &[2, 2, 1, 1, 2, 1],
+        record_counts_in_leaf_order: &[2, 1, 2, 1],
+        short_container: 2,
+        root_ring: RootRingBeforeTheJudgement::TurnedOverByCreatingInodes,
+    };
+
+/// 第 58 批 J3：只这一棵按 `layout` 逐条换旋钮写出三片以上的容器（另两棵照产品），带旋钮值的入口报这一棵旋钮 2：每片都不超旋钮值、
+/// 容器号从 1 逐片加一、每片至少一条，只「不是最右的容器 `short_container` 装了 1 条，不是旋钮值 2」不成立，只红这一棵的那一条（I-9.22 / I-9.37）。
+/// 第 57 批 H1 的会红用例都只有两片容器，两片时只判最左片、只判倒数第二片、只判叶序第一片与正确写法等价，这几张盘把它们分开。
+fn assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+    tree: ContainerTreeOfTheCapacityInvariants,
+    tag: &str,
+    layout: &ContainersUnderChangingKnobs,
+) {
+    let mut pool = pool_with_records_in_one_container_tree_written_under_these_knobs(
+        tree,
+        tag,
+        layout.records_per_container_of_each_record,
+    );
+    match layout.root_ring {
+        RootRingBeforeTheJudgement::KeepsTheVersionsThatWroteTheContainerTree => {}
+        RootRingBeforeTheJudgement::TurnedOverByCreatingInodes => {
+            common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes(&mut pool);
+        }
+    }
+    assert_eq!(
+        tree.record_counts_of_each_container(&pool.output),
+        layout.record_counts_in_leaf_order,
+        "{tree:?} 按叶序每片的记录数"
+    );
+    assert_only_the_knob_capacity_invariant_of_the_tree_reddens_given_the_knob(
+        &pool.memory_pool(),
+        tree,
+        KNOB_RECORDS_OF_THE_SHORT_CONTAINER_IMAGES,
+        &format!(
+            "不是最右的容器 {} 装了 1 条，不是旋钮值 {KNOB_RECORDS_OF_THE_SHORT_CONTAINER_IMAGES}",
+            layout.short_container
+        ),
+    );
+}
+
+#[test]
+fn tombstone_containers_of_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+) {
+    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+        ContainerTreeOfTheCapacityInvariants::Tombstone,
+        "known-bad-tombstone-containers-of-two-one-and-one-records",
+        &MIDDLE_CONTAINER_SHORT,
+    );
+}
+
+#[test]
+fn lineage_containers_of_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+) {
+    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+        ContainerTreeOfTheCapacityInvariants::Lineage,
+        "known-bad-lineage-containers-of-two-one-and-one-records",
+        &MIDDLE_CONTAINER_SHORT,
+    );
+}
+
+#[test]
+fn retirement_containers_of_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+) {
+    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+        ContainerTreeOfTheCapacityInvariants::Retirement,
+        "known-bad-retirement-containers-of-two-one-and-one-records",
+        &MIDDLE_CONTAINER_SHORT,
+    );
+}
+
+#[test]
+fn tombstone_containers_of_two_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+) {
+    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+        ContainerTreeOfTheCapacityInvariants::Tombstone,
+        "known-bad-tombstone-containers-of-two-two-one-and-one-records",
+        &ALL_FULL_BEFORE_A_SHORT_SECOND_TO_LAST_CONTAINER,
+    );
+}
+
+#[test]
+fn lineage_containers_of_two_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+) {
+    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+        ContainerTreeOfTheCapacityInvariants::Lineage,
+        "known-bad-lineage-containers-of-two-two-one-and-one-records",
+        &ALL_FULL_BEFORE_A_SHORT_SECOND_TO_LAST_CONTAINER,
+    );
+}
+
+#[test]
+fn retirement_containers_of_two_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+) {
+    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+        ContainerTreeOfTheCapacityInvariants::Retirement,
+        "known-bad-retirement-containers-of-two-two-one-and-one-records",
+        &ALL_FULL_BEFORE_A_SHORT_SECOND_TO_LAST_CONTAINER,
+    );
+}
+
+#[test]
+#[ignore = "harness 慢档用例：根环转过一圈（30 次发布）再逐版判，debug 下单条跑过 60 秒"]
+fn tombstone_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+) {
+    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+        ContainerTreeOfTheCapacityInvariants::Tombstone,
+        "known-bad-tombstone-containers-of-two-one-two-and-one-records",
+        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
+    );
+}
+
+#[test]
+#[ignore = "harness 慢档用例：根环转过一圈（30 次发布）再逐版判，debug 下单条跑过 60 秒"]
+fn lineage_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+) {
+    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+        ContainerTreeOfTheCapacityInvariants::Lineage,
+        "known-bad-lineage-containers-of-two-one-two-and-one-records",
+        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
+    );
+}
+
+#[test]
+#[ignore = "harness 慢档用例：根环转过一圈（30 次发布）再逐版判，debug 下单条跑过 60 秒"]
+fn retirement_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+) {
+    assert_containers_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two(
+        ContainerTreeOfTheCapacityInvariants::Retirement,
+        "known-bad-retirement-containers-of-two-one-two-and-one-records",
+        &A_SHORT_CONTAINER_BEFORE_A_FULL_ONE,
+    );
+}
+
 /// G3 ③：退役容器压到一片 2 条写出两条退役记录（一片 2 条），带旋钮值的入口报退役旋钮 1：那片装了 2 条、多于旋钮值，只红 I-9.37；
 /// 报写的时候那个旋钮值 2 一条不红。
 #[test]
`````

### mutations-append.tsv

`````text
第58批 J3 I-9.22：旋钮支只判叶序第一片（墓碑树那一形，攻方 h1-first-only；三片中间少一条 [2, 1, 1]）	crates/singlefs-checker/src/walk.rs	                if is_the_rightmost_tombstone_container_under_the_knob {\n                    continue;\n                }	                if is_the_rightmost_tombstone_container_under_the_knob || position_in_leaf_order > 0 {\n                    continue;\n                }	-p singlefs-harness --test checker_known_bad_images -- records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two	tombstone_containers_of_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two
第58批 J3 I-9.22：旋钮支只判最左片（墓碑树那一形，正推 K4-2 MB2；倒数第二片之前全满、倒数第二片少一条 [2, 2, 1, 1]）	crates/singlefs-checker/src/walk.rs	                let is_the_rightmost_tombstone_container_under_the_knob =\n                    position_in_leaf_order + 1 == containers.len();	                let is_the_rightmost_tombstone_container_under_the_knob =\n                    position_in_leaf_order != 0 || containers.len() == 1;	-p singlefs-harness --test checker_known_bad_images -- records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two	tombstone_containers_of_two_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two
第58批 J3 I-9.22：旋钮支只判倒数第二片（墓碑树那一形，正推 K4-2 MC；少一条的那片之后还有满的一片 [2, 1, 2, 1]、根环转过一圈）	crates/singlefs-checker/src/walk.rs	                let is_the_rightmost_tombstone_container_under_the_knob =\n                    position_in_leaf_order + 1 == containers.len();	                let is_the_rightmost_tombstone_container_under_the_knob =\n                    position_in_leaf_order + 2 != containers.len();	-p singlefs-harness --release --test checker_known_bad_images -- --include-ignored tombstone_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two	tombstone_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two
第58批 J3 I-9.37：旋钮支只判叶序第一片（谱系树那一形，攻方 h1-first-only；三片中间少一条 [2, 1, 1]）	crates/singlefs-checker/src/walk.rs	                if is_the_rightmost_under_the_knob {\n                    continue;\n                }	                if is_the_rightmost_under_the_knob || position_in_leaf_order > 0 {\n                    continue;\n                }	-p singlefs-harness --test checker_known_bad_images -- records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two	lineage_containers_of_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two
第58批 J3 I-9.37：旋钮支只判最左片（退役树那一形，正推 K4-2 MB2；倒数第二片之前全满、倒数第二片少一条 [2, 2, 1, 1]）	crates/singlefs-checker/src/walk.rs	                let is_the_rightmost_under_the_knob =\n                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len();	                let is_the_rightmost_under_the_knob =\n                    position_in_leaf_order != 0 || container_numbers_and_record_counts.len() == 1;	-p singlefs-harness --test checker_known_bad_images -- records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two	retirement_containers_of_two_two_one_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two
第58批 J3 I-9.37：旋钮支只判倒数第二片（谱系树那一形，正推 K4-2 MC；少一条的那片之后还有满的一片 [2, 1, 2, 1]、根环转过一圈）	crates/singlefs-checker/src/walk.rs	                let is_the_rightmost_under_the_knob =\n                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len();	                let is_the_rightmost_under_the_knob =\n                    position_in_leaf_order + 2 != container_numbers_and_record_counts.len();	-p singlefs-harness --release --test checker_known_bad_images -- --include-ignored lineage_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two	lineage_containers_of_two_one_two_and_one_records_under_changing_knobs_redden_only_the_knob_capacity_invariant_when_judged_with_the_knob_two
第58批 J3 GPU 判器：旋钮那一支只判叶序第一片（墓碑树三片中间少一条 [2, 1, 1] 那一形）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (d_container_record_count(container_index) != knob_records) {	        if (container_index == 0u && d_container_record_count(container_index) != knob_records) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_and_one_records_written_under_changing_knobs	the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_tombstone_containers_of_two_one_and_one_records_written_under_changing_knobs
`````

## m3-batch58/cold-walk-pins

### crates.patch

`````diff
diff --git a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
--- a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
+++ b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
@@ -1687,14 +1687,12 @@
     );
 }
 
-// ─── 分配记录逐条判代与跨度先于数记账条目与数映射条目 ───
+// ─── 分配记录逐条判代与跨度先于记账条目宽与逐条判、数记账条目、数映射条目 ───
 
-/// 台阶行的统计量标签换成的值：`singlefs_core::records` 登记的统计量标签到 inode 号水位 12 为止，这是之后的一个、没有登记。
-/// 逐条判记账条目只判代与 seq，这个标签照收；数记账条目只把「统计量是 inode 号水位且代早于根」的行当台阶行不算，
-/// 换了标签的那一行被算进 3 + 6 × 盘数。
-const STATISTIC_TAG_AFTER_THE_LAST_REGISTERED_ONE: u16 = STATISTIC_INODE_WATERMARK + 1;
 /// 分配记录的代改成的值：晚于台阶行那个池最新那条根的 txg 4。
 const ALLOCATION_RECORD_GENERATION_AFTER_THE_ROOT: CheckpointTxg = CheckpointTxg(5);
+/// 这一版自己那一行 inode 号水位（代段 4）的代改成的值：晚于台阶行那个池最新那条根的 txg 4。
+const WATERMARK_ROW_GENERATION_AFTER_THE_ROOT: CheckpointTxg = CheckpointTxg(5);
 
 /// 分配记录树里 `pointer` 指着的节点与它下面整棵子树：叶里满足 `is_the_record_to_change` 的记录交给 `change` 改（只改不进 key 的字段），
 /// 改过的节点重封、写回两份落点；内部节点里指着改过的孩子的那条内部条目（key 10 + 子指针 86），子指针两条位置条目的整单元校验和
@@ -1846,12 +1844,108 @@
     }
 }
 
+/// 映射条目数那一个成员。
+fn mapping_entry_count_is_not_one_per_mapped_unit() -> RecoveryFailure {
+    RecoveryFailure::InvariantViolated {
+        invariant: "E142 走读同款",
+        detail:
+            "映射条目数不是 1 + extent 树、分配记录树、记账树的节点数 + inode 叶容器数 + 数据单元数",
+    }
+}
+
+/// 码 2 单个叶根里删掉第 `entry_index` 条条目：之后的条目前移一条、空出来的末条清零，条目数减一、声明长度（偏移 8，= 条目数 × 条目宽）跟着减，
+/// 节点头里的最小 / 最大 key（从偏移 52 起，各一个 key 宽）换成新的首末两条的 key（条目的前 key 宽字节）。条目宽不变，key 照旧严格递增，
+/// 判全那一道（节点头、key 区间与次序）照过。交回删掉的那条条目的字节；重封归调用方。
+fn drop_one_entry_of_an_index_leaf_root(node: &mut [u8], entry_index: usize) -> Vec<u8> {
+    let key_width = usize::from(node[51]);
+    let header_end = 86 + 2 * key_width;
+    let entry_count = usize::from(u16::from_le_bytes([
+        node[header_end - 4],
+        node[header_end - 3],
+    ]));
+    assert!(
+        entry_count >= 2 && entry_index < entry_count,
+        "叶根至少两条、删的那条在里面：删完还有首末两条可取 key（{entry_count} 条，删第 {entry_index} 条）"
+    );
+    let dropped_range = entry_range_in_node(node, entry_index);
+    let last_range = entry_range_in_node(node, entry_count - 1);
+    let entry_width = dropped_range.len();
+    let dropped_entry = node[dropped_range.clone()].to_vec();
+    node.copy_within(dropped_range.end..last_range.end, dropped_range.start);
+    node[last_range].fill(0);
+    let remaining_entry_count = entry_count - 1;
+    let remaining_entry_count_field =
+        u16::try_from(remaining_entry_count).expect("条目数比删之前少一条，装得进 2 字节");
+    node[header_end - 4..header_end - 2]
+        .copy_from_slice(&remaining_entry_count_field.to_le_bytes());
+    let declared_length = u16::try_from(remaining_entry_count * entry_width)
+        .expect("条目区比删之前短，装得进 2 字节");
+    node[8..10].copy_from_slice(&declared_length.to_le_bytes());
+    let first_range = entry_range_in_node(node, 0);
+    let new_last_range = entry_range_in_node(node, remaining_entry_count - 1);
+    let smallest_key = node[first_range.start..first_range.start + key_width].to_vec();
+    let largest_key = node[new_last_range.start..new_last_range.start + key_width].to_vec();
+    node[52..52 + key_width].copy_from_slice(&smallest_key);
+    node[52 + key_width..52 + 2 * key_width].copy_from_slice(&largest_key);
+    dropped_entry
+}
+
+/// 最新那条根的中央映射树根（单个叶根）里 key 最大的那一条删掉，重封、写回两份落点，根记录里映射树根那条指针两条位置条目的校验和换成新值。
+fn drop_the_entry_of_the_largest_key_from_the_central_mapping_root_of_the_newest_root(
+    devices: &mut Devices,
+) {
+    let (ring_slot, mut root) = newest_root_in_the_ring(devices);
+    let mut mapping_root = read_bytes(
+        devices,
+        root.mapping_root.locations[0].device,
+        root.mapping_root.locations[0].slot.to_device_offset(),
+        node_bytes(),
+    );
+    let mapping_header = parse_index_node(&mapping_root).expect("映射树根解得开");
+    assert_eq!(
+        mapping_header.level, 0,
+        "建空 inode 那一版的映射树是单个叶根"
+    );
+    let dropped_mapping_entry =
+        drop_one_entry_of_an_index_leaf_root(&mut mapping_root, mapping_header.entries.len() - 1);
+    assert!(
+        parse_mapping_entry(&dropped_mapping_entry).is_some(),
+        "删掉的是一条解得开的映射条目"
+    );
+    reseal_index_node(&mut mapping_root);
+    let mapping_root_checksum =
+        write_unit_at_both_locations(devices, &root.mapping_root.locations, &mapping_root);
+    for location in &mut root.mapping_root.locations {
+        location.unit_checksum = mapping_root_checksum;
+    }
+    write_root_into_the_ring_slot(devices, ring_slot, &root);
+}
+
+/// 最新那条根（txg 4）的分配记录树里罩这一版树表单元那个槽的两条记录（两盘同槽各一条，代 4、还着）代改成 5，晚于根；两盘一起改，
+/// 「每个落点每盘一条」那一判（比的是各盘的（槽, 跨度, 代, 已释放）集合）照过。指着改过的节点的每一道校验和重算。
+fn move_the_allocation_records_of_the_tree_table_slot_after_the_root(devices: &mut Devices) {
+    let (_, root) = newest_root_in_the_ring(devices);
+    let tree_table_slot = root.tree_table.locations[0].slot;
+    let changed_records = rewrite_allocation_records_of_the_newest_root(
+        devices,
+        |record| record.slot == tree_table_slot,
+        |record| {
+            assert_eq!(
+                (record.generation, record.is_released),
+                (TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW, false),
+                "罩这一版树表单元的记录是这一版分配的、还着"
+            );
+            record.generation = ALLOCATION_RECORD_GENERATION_AFTER_THE_ROOT;
+        },
+    );
+    assert_eq!(changed_records, 2, "罩树表单元那个槽的分配记录两盘各一条");
+}
+
 /// 台阶行那个池（[`pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row`]，txg 4）上两处一起坏，
 /// 每一处都把指着它的校验和重算：
-/// ① 记账树根里那一级台阶行（inode 号水位、代段 3）的统计量标签换成 [`STATISTIC_TAG_AFTER_THE_LAST_REGISTERED_ONE`]，条目按 key 重排：
-///    代 3、seq 照旧，逐条判照过；数记账条目时它不再算台阶行，16 ≠ 3 + 6 × 2。只坏这一处时只读恢复报记账条目数，这里先核一遍；
-/// ② 分配记录树里罩这一版树表单元那个槽的两条记录（两盘同槽各一条，代 4）代改成 5，晚于根；两盘一起改，
-///    「每个落点每盘一条」那一判（比的是各盘的（槽, 跨度, 代, 已释放）集合）照过。
+/// ① 记账树根里 key 最小的那一行（不是 inode 号水位的行）删掉，之后的条目前移、条目数与声明长度减一；统计量标签与代一处都不碰：
+///    逐条判照过，数记账条目 16 − 1 − 1 条台阶行 = 14 ≠ 3 + 6 × 2，与读者认不认哪个标签无关。只坏这一处时只读恢复报记账条目数，这里先核一遍；
+/// ② [`move_the_allocation_records_of_the_tree_table_slot_after_the_root`]。
 fn pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_entry_count_is_off(
 ) -> Devices {
     let mut devices =
@@ -1860,13 +1954,12 @@
         &mut devices,
         TREE_KIND_ACCOUNTING,
         |node| {
-            rewrite_one_entry_of_an_accounting_leaf_root_keeping_the_key_order(
-                node,
-                |entry| {
-                    entry.statistic == STATISTIC_INODE_WATERMARK
-                        && entry.generation < TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW
-                },
-                |step_row| step_row.statistic = STATISTIC_TAG_AFTER_THE_LAST_REGISTERED_ONE,
+            let dropped_row =
+                AccountingEntry::parse(&drop_one_entry_of_an_index_leaf_root(node, 0))
+                    .expect("记账叶根的条目宽是 34，删掉的那条解得开");
+            assert_ne!(
+                dropped_row.statistic, STATISTIC_INODE_WATERMARK,
+                "删的不是 inode 号水位的行：条目数少的这一条与认不认台阶行无关"
             );
         },
     );
@@ -1878,29 +1971,76 @@
         },
         "前提：只坏记账那一处时，只读恢复在 txg 4 那条根上报记账条目数"
     );
-    let (_, root) = newest_root_in_the_ring(&devices);
-    let tree_table_slot = root.tree_table.locations[0].slot;
-    let changed_records = rewrite_allocation_records_of_the_newest_root(
+    move_the_allocation_records_of_the_tree_table_slot_after_the_root(&mut devices);
+    devices
+}
+
+/// 台阶行那个池（txg 4）上两处一起坏，每一处都把指着它的校验和重算：
+/// ① 中央映射树根（单个叶根）里 key 最大的那一条删掉（[`drop_the_entry_of_the_largest_key_from_the_central_mapping_root_of_the_newest_root`]）：
+///    各树节点数、inode 叶容器数、数据单元数一样都没动，映射条目数比它们算出的少一条；记账树一处不碰，数记账条目照过。
+///    只坏这一处时只读恢复报映射条目数，这里先核一遍；
+/// ② [`move_the_allocation_records_of_the_tree_table_slot_after_the_root`]。
+fn pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_mapping_entry_count_is_off(
+) -> Devices {
+    let mut devices =
+        pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
+    drop_the_entry_of_the_largest_key_from_the_central_mapping_root_of_the_newest_root(
         &mut devices,
-        |record| record.slot == tree_table_slot,
-        |record| {
-            assert_eq!(
-                (record.generation, record.is_released),
-                (TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW, false),
-                "罩这一版树表单元的记录是这一版分配的、还着"
+    );
+    assert_eq!(
+        recover(&devices, JournalPolicy::Consult).outcome,
+        RecoveryOutcome::Failed {
+            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
+            failure: mapping_entry_count_is_not_one_per_mapped_unit(),
+        },
+        "前提：只坏映射那一处时，只读恢复在 txg 4 那条根上报映射条目数"
+    );
+    move_the_allocation_records_of_the_tree_table_slot_after_the_root(&mut devices);
+    devices
+}
+
+/// 台阶行那个池（txg 4）上两处一起坏，每一处都把指着它的校验和重算：
+/// ① 记账树根里这一版自己那一行 inode 号水位（代段 4，等于根的 txg；台阶行是代段 3 的那一行，不碰）的代改成 5，晚于根，条目按 key 重排：
+///    条目宽照旧 34；台阶行照旧认得出，数记账条目照旧 3 + 6 × 2，只有逐条判代与 seq 那一判报它。只坏这一处时只读恢复报记账条目的代，这里先核一遍；
+/// ② [`move_the_allocation_records_of_the_tree_table_slot_after_the_root`]。
+fn pool_whose_allocation_records_and_one_accounting_entry_carry_a_generation_after_the_root(
+) -> Devices {
+    let mut devices =
+        pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row();
+    rewrite_the_root_node_of_a_tree_of_the_newest_root(
+        &mut devices,
+        TREE_KIND_ACCOUNTING,
+        |node| {
+            rewrite_one_entry_of_an_accounting_leaf_root_keeping_the_key_order(
+                node,
+                |entry| {
+                    entry.statistic == STATISTIC_INODE_WATERMARK
+                        && entry.generation == TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW
+                },
+                |watermark_row_of_this_version| {
+                    watermark_row_of_this_version.generation =
+                        WATERMARK_ROW_GENERATION_AFTER_THE_ROOT;
+                },
             );
-            record.generation = ALLOCATION_RECORD_GENERATION_AFTER_THE_ROOT;
         },
     );
-    assert_eq!(changed_records, 2, "罩树表单元那个槽的分配记录两盘各一条");
+    assert_eq!(
+        recover(&devices, JournalPolicy::Consult).outcome,
+        RecoveryOutcome::Failed {
+            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
+            failure: accounting_entry_generation_after_the_root_or_sequence_zero(),
+        },
+        "前提：只坏记账那一处时，只读恢复在 txg 4 那条根上报记账条目的代晚于根"
+    );
+    move_the_allocation_records_of_the_tree_table_slot_after_the_root(&mut devices);
     devices
 }
 
-/// 场景：第一个文件之后再建一个空 inode（txg 4）；记账树根里那一级台阶行的统计量标签换成没登记的号（记账条目数因此对不上），
+/// 场景：第一个文件之后再建一个空 inode（txg 4）；记账树根里 key 最小的那一行删掉（标签与代都不碰，记账条目数因此对不上），
 /// 罩这一版树表单元那个槽的两条分配记录代改成 5，晚于根；指着改过的节点的每一道校验和重算。
-/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度排在数记账条目、
-/// 数映射条目之前（先判得了自己那一条的，再数；判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节 H4）。
-/// 那一判挪到两个数之后，这份镜像报成记账条目数。
+/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度排在数记账条目之前
+/// （先判得了自己那一条的，再数；判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节 H4）。
+/// 那一判挪到数记账条目之后，这份镜像报成记账条目数。排在数映射条目、记账条目逐条判之前那两个次序由下面两张镜像各钉一个。
 #[test]
 fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_recovery(
 ) {
@@ -1932,6 +2072,64 @@
     );
 }
 
+/// 场景：第一个文件之后再建一个空 inode（txg 4）；中央映射树根里 key 最大的那一条删掉（映射条目数因此对不上，记账树不碰），
+/// 罩这一版树表单元那个槽的两条分配记录代改成 5，晚于根；指着改过的节点的每一道校验和重算。
+/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度也排在数映射条目之前
+/// （判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节 H4「排在两个数之前」的另一个数；
+/// 攻方原型 `research/prompts/m3-batch57-code-r1-opus-model/probes/corrupt_on_disk_k3_append.rs` 的 ③ 那一形）。
+/// 数映射条目挪到那一判之前，这份镜像报成映射条目数，而可写挂载照报分配记录：两条路对同一张盘报不同的成员。
+#[test]
+fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_mapping_entry_count_by_the_recovery(
+) {
+    let devices =
+        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_mapping_entry_count_is_off();
+    let report = recover(&devices, JournalPolicy::Consult);
+    assert_eq!(
+        report.outcome,
+        RecoveryOutcome::Failed {
+            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
+            failure: allocation_record_span_zero_or_generation_after_the_root(),
+        },
+        "只读恢复在 txg 4 那条根上报分配记录的代晚于根，不报映射条目数"
+    );
+}
+
+/// 场景：同上一条的坏镜像，交给可写挂载。
+/// 预期：可写挂载在任何写之前交回「分配记录跨度为 0，或分配代 / 释放代晚于根」：重建那一版逐条判分配记录在读记账树之前，
+/// 数映射条目在读完记账树之后，与冷走读报同一个成员。
+#[test]
+fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_mapping_entry_count_by_the_writable_mount(
+) {
+    let mut devices =
+        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_mapping_entry_count_is_off();
+    let refusal = writable_mount_refused_before_any_write(&mut devices);
+    assert!(
+        matches!(&refusal, MountError::Recovery(failure) if *failure == allocation_record_span_zero_or_generation_after_the_root()),
+        "可写挂载报分配记录的代晚于根，不报映射条目数：{refusal:?}"
+    );
+}
+
+/// 场景：第一个文件之后再建一个空 inode（txg 4）；记账树根里这一版自己那一行 inode 号水位的代改成 5（晚于根；台阶行不碰，记账条目数照旧对得上），
+/// 罩这一版树表单元那个槽的两条分配记录代改成 5，晚于根；指着改过的节点的每一道校验和重算。
+/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度排在记账条目宽、
+/// 记账条目逐条判代与 seq 之前（冷走读那一段的次序注释与重建相同：分配记录、条目宽、逐条判、数）。
+/// 那一判挪到记账条目逐条判之后（仍在两个数之前），这份镜像报成「记账条目的代晚于根或 seq 为 0」。
+#[test]
+fn allocation_records_whose_generation_is_after_the_root_are_refused_before_each_accounting_entry_is_judged_by_the_recovery(
+) {
+    let devices =
+        pool_whose_allocation_records_and_one_accounting_entry_carry_a_generation_after_the_root();
+    let report = recover(&devices, JournalPolicy::Consult);
+    assert_eq!(
+        report.outcome,
+        RecoveryOutcome::Failed {
+            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
+            failure: allocation_record_span_zero_or_generation_after_the_root(),
+        },
+        "只读恢复在 txg 4 那条根上报分配记录的代晚于根，不报记账条目的代"
+    );
+}
+
 // ─── 第 35 条续：单元区槽数那一处减法 ───
 
 /// 场景：700 MiB 的盘（单元区起点在 784 MiB）。预期：单元区槽数交回「盘末尾在单元区起点之前」，带盘的字节数与起点槽号；
`````

### mutations-append.tsv

`````text
第58批 J1 冷走读数映射条目挪到分配记录代与跨度之前（代晚于根的分配记录在映射条目数对不上的镜像上报成映射条目数）	crates/singlefs-core/src/recovery.rs	    allocation_record_generations_and_spans_are_judged(allocation_records, root)?;\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &roots.accounting.leaf_entries_in_key_order,\n    )?;\n    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {\n        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &roots.accounting.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_count,\n    )?;\n    mapping_entry_count_is_one_per_mapped_unit(\n        roots.mapping.leaf_entries_in_key_order.len(),\n        MappedUnitCounts {\n            extent_tree_nodes: roots.extent.nodes_in_bump_order.len(),\n            allocation_record_tree_nodes: roots.allocation.nodes.len(),\n            accounting_tree_nodes: roots.accounting.version.node_count(),\n            inode_leaf_containers: roots.inode.entries.len(),\n            data_units: roots.extent.data_pointers.len(),\n            deletion_tree_nodes,\n        },\n    )?;	    mapping_entry_count_is_one_per_mapped_unit(\n        roots.mapping.leaf_entries_in_key_order.len(),\n        MappedUnitCounts {\n            extent_tree_nodes: roots.extent.nodes_in_bump_order.len(),\n            allocation_record_tree_nodes: roots.allocation.nodes.len(),\n            accounting_tree_nodes: roots.accounting.version.node_count(),\n            inode_leaf_containers: roots.inode.entries.len(),\n            data_units: roots.extent.data_pointers.len(),\n            deletion_tree_nodes,\n        },\n    )?;\n    allocation_record_generations_and_spans_are_judged(allocation_records, root)?;\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &roots.accounting.leaf_entries_in_key_order,\n    )?;\n    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {\n        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &roots.accounting.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_count,\n    )?;	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- allocation_records_whose_generation_is_after_the_root	allocation_records_whose_generation_is_after_the_root_are_refused_before_the_mapping_entry_count_by_the_recovery
第58批 J1 冷走读分配记录代与跨度挪到记账条目宽与逐条判之后、紧贴数记账条目（代晚于根的分配记录在一条记账条目的代也晚于根的镜像上报成记账条目的代）	crates/singlefs-core/src/recovery.rs	    allocation_record_generations_and_spans_are_judged(allocation_records, root)?;\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &roots.accounting.leaf_entries_in_key_order,\n    )?;\n    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {\n        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;\n    }\n	    every_accounting_entry_is_as_wide_as_its_field_table(\n        &roots.accounting.leaf_entries_in_key_order,\n    )?;\n    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {\n        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;\n    }\n    allocation_record_generations_and_spans_are_judged(allocation_records, root)?;\n	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- allocation_records_whose_generation_is_after_the_root	allocation_records_whose_generation_is_after_the_root_are_refused_before_each_accounting_entry_is_judged_by_the_recovery
第58批 J1 重建分配记录代与跨度挪到数映射条目之后（代晚于根的分配记录在映射条目数对不上的镜像上可写挂载报成映射条目数）	crates/singlefs-core/src/recovery.rs	    allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;\n    let allocation_records: Vec<AllocationRecord> = allocation_tree_read.records.clone();\n    // 记账树可以是多层（D8（核心索引结构） 已定项 11）：从根往下整棵读回来，节点进映射、提示读不出经这一版的中央映射回退\n    // （与读别的树根同一条）；中央映射树同样整棵读回来（下面）。两棵树照冷走读判全（`code_two_tree::read_code_two_tree` 的\n    // `EveryHeaderAgainstItsReference`：节点头、层级、区间与 key 严格递增——有重复 key 的记账叶交出去，挂载之后第一次发布规划那棵树时\n    // 在 `transaction` 的断言上 panic，代码审阅第 33 条）。\n    let accounting_tree = read_code_two_tree(\n        &accounting_pointer,\n        &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),\n        CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,\n        root,\n        expected_filesystem_identifier,\n        &mut |pointer: &NodePointer| {\n            read_mapped_tree_node_via_hint_then_central_mapping(\n                reader,\n                pointer,\n                MappedTreeNodeClass::IndexNode,\n                &central_mapping_locations_of_key,\n                &mut stale_location_hint_hops_not_exposed_by_this_reader,\n            )\n        },\n    )?;\n    // 次序：条目宽、逐条判代与 seq、数条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`\n    // 第三节 G1）。数条目要按统计量与代段认台阶行，代写到根之后的台阶行认不出、被算进 3 + 6 × 盘数，「代晚于根」这一坏法就报成了条目数不对。\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &accounting_tree.leaf_entries_in_key_order,\n    )?;\n    let mut accounting_entries: Vec<AccountingEntry> =\n        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());\n    for bytes in &accounting_tree.leaf_entries_in_key_order {\n        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(\n            bytes, root,\n        )?);\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_identities.len(),\n    )?;\n    // 记账行的统计量标签是盘上读来的 2 字节：这一版交出去之前判一次「inode 号水位那一行在」，往里就信\n    // （`TransactionOutput::inode_number_watermark` 那句 expect 依赖的就是这一判；panic 面普查 R11）。\n    // 要的是这一版自己写的那一行（代段 = 根的 txg）：那一统计量另有代段更早的台阶行（D5（快照 / 空间记账机制） 已定项 2 射程），\n    // 只剩台阶行时现行值取到的是一级台阶，不是这一版的水位。\n    if !accounting_entries.iter().any(|entry| {\n        entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation == root.checkpoint_txg\n    }) {\n        return Err(RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree.into());\n    }\n    // 映射树是自举豁免：不经映射回退，只按父指针里的位置条目读；上面有提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份。\n    let mapping_tree = central_mapping_root.into_tree()?;\n    mapping_entry_count_is_one_per_mapped_unit(\n        mapping_tree.leaf_entries_in_key_order.len(),\n        MappedUnitCounts {\n            extent_tree_nodes: first_file_extents.nodes_in_bump_order.len(),\n            allocation_record_tree_nodes: allocation_tree_read.nodes.len(),\n            accounting_tree_nodes: accounting_tree.version.node_count(),\n            inode_leaf_containers: inode_root_node.entries.len(),\n            data_units: data_pointers.len(),\n            deletion_tree_nodes: usize::from(deleted_inodes_root_bytes.is_some())\n                + tombstone_tree_read\n                    .as_ref()\n                    .map_or(0, ContainerTreeRead::mapped_units)\n                + lineage_tree_read\n                    .as_ref()\n                    .map_or(0, ContainerTreeRead::mapped_units)\n                + retirement_tree_read\n                    .as_ref()\n                    .map_or(0, ContainerTreeRead::mapped_units),\n        },\n    )?;	    let allocation_records: Vec<AllocationRecord> = allocation_tree_read.records.clone();\n    // 记账树可以是多层（D8（核心索引结构） 已定项 11）：从根往下整棵读回来，节点进映射、提示读不出经这一版的中央映射回退\n    // （与读别的树根同一条）；中央映射树同样整棵读回来（下面）。两棵树照冷走读判全（`code_two_tree::read_code_two_tree` 的\n    // `EveryHeaderAgainstItsReference`：节点头、层级、区间与 key 严格递增——有重复 key 的记账叶交出去，挂载之后第一次发布规划那棵树时\n    // 在 `transaction` 的断言上 panic，代码审阅第 33 条）。\n    let accounting_tree = read_code_two_tree(\n        &accounting_pointer,\n        &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),\n        CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,\n        root,\n        expected_filesystem_identifier,\n        &mut |pointer: &NodePointer| {\n            read_mapped_tree_node_via_hint_then_central_mapping(\n                reader,\n                pointer,\n                MappedTreeNodeClass::IndexNode,\n                &central_mapping_locations_of_key,\n                &mut stale_location_hint_hops_not_exposed_by_this_reader,\n            )\n        },\n    )?;\n    // 次序：条目宽、逐条判代与 seq、数条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`\n    // 第三节 G1）。数条目要按统计量与代段认台阶行，代写到根之后的台阶行认不出、被算进 3 + 6 × 盘数，「代晚于根」这一坏法就报成了条目数不对。\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &accounting_tree.leaf_entries_in_key_order,\n    )?;\n    let mut accounting_entries: Vec<AccountingEntry> =\n        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());\n    for bytes in &accounting_tree.leaf_entries_in_key_order {\n        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(\n            bytes, root,\n        )?);\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_identities.len(),\n    )?;\n    // 记账行的统计量标签是盘上读来的 2 字节：这一版交出去之前判一次「inode 号水位那一行在」，往里就信\n    // （`TransactionOutput::inode_number_watermark` 那句 expect 依赖的就是这一判；panic 面普查 R11）。\n    // 要的是这一版自己写的那一行（代段 = 根的 txg）：那一统计量另有代段更早的台阶行（D5（快照 / 空间记账机制） 已定项 2 射程），\n    // 只剩台阶行时现行值取到的是一级台阶，不是这一版的水位。\n    if !accounting_entries.iter().any(|entry| {\n        entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation == root.checkpoint_txg\n    }) {\n        return Err(RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree.into());\n    }\n    // 映射树是自举豁免：不经映射回退，只按父指针里的位置条目读；上面有提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份。\n    let mapping_tree = central_mapping_root.into_tree()?;\n    mapping_entry_count_is_one_per_mapped_unit(\n        mapping_tree.leaf_entries_in_key_order.len(),\n        MappedUnitCounts {\n            extent_tree_nodes: first_file_extents.nodes_in_bump_order.len(),\n            allocation_record_tree_nodes: allocation_tree_read.nodes.len(),\n            accounting_tree_nodes: accounting_tree.version.node_count(),\n            inode_leaf_containers: inode_root_node.entries.len(),\n            data_units: data_pointers.len(),\n            deletion_tree_nodes: usize::from(deleted_inodes_root_bytes.is_some())\n                + tombstone_tree_read\n                    .as_ref()\n                    .map_or(0, ContainerTreeRead::mapped_units)\n                + lineage_tree_read\n                    .as_ref()\n                    .map_or(0, ContainerTreeRead::mapped_units)\n                + retirement_tree_read\n                    .as_ref()\n                    .map_or(0, ContainerTreeRead::mapped_units),\n        },\n    )?;\n    allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- allocation_records_whose_generation_is_after_the_root	allocation_records_whose_generation_is_after_the_root_are_refused_before_the_mapping_entry_count_by_the_writable_mount
`````

