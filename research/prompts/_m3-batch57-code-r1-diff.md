# 删除线第 57 批（代码轮 m3-batch56-code-r1 判决的 H1–H4）：代码轮第一轮改动全文（2026-10-10）

两个实现员交的补丁原样，按打的次序（甲 → 乙），底是主工作区合入第 56 批与根环线 G、G2 之后的样子。

## m3-batch57/container-judges

### crates.patch

`````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index 03f4a28..526581f 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -3342,7 +3342,7 @@ fn d_judge_container_numbers_from_one_by_one_and_no_empty_container(invariant: u
 }
 
 // 装了只供测试的容量旋钮（knob_records 条一片）的那一棵（walk.rs 那两个判法装了旋钮那一支，I-9.22 / I-9.37）：至少一片时判容器号与记录数的三句，
-// 每片记录数 ≤ knob_records；零根没有对象
+// 每片记录数 ≤ knob_records，最右那片之外每片记录数 == knob_records（开放容器满了才关，D18 已定项 24 第 3 条在旋钮下的那一半，第 57 批 H1）；零根没有对象
 fn d_judge_containers_under_the_capacity_knob(knob_records: u32, invariant: u32) {
     if (d_container_count == 0u) {
         return;
@@ -3353,6 +3353,11 @@ fn d_judge_containers_under_the_capacity_knob(knob_records: u32, invariant: u32)
             violate(invariant);
         }
     }
+    for (var container_index = 0u; container_index + 1u < d_container_count; container_index = container_index + 1u) {
+        if (d_container_record_count(container_index) != knob_records) {
+            violate(invariant);
+        }
+    }
 }
 
 // I-9.21 / I-9.22（walk.rs judge_tombstone_container_capacity_of_a_version）：墓碑旋钮 D_TOMBSTONE_CONTAINER_CAPACITY_KNOB 不为 0 时判 I-9.22
diff --git a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
index 7f44ae0..53b3f2d 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -1492,7 +1492,9 @@ mod tests {
     /// 第 36 批之前是恢复 320 015、扫描 382 259、走读 337 872）；崩溃点重放合并实现 G（段的展开方式与满 64 位的段内序号、给定状态表、
     /// 失败点对照表、不看 journal 那一遍落到的根、逐条判到位；判决 m3-crash-enumeration-merge-r2 第二节 L1-4、L1-6、L1-7、L6-2、L6-3）
     /// 恢复 332 666 → 334 042、扫描 388 815 → 395 434、走读 346 278 → 352 481（实现 G 副本上用同一个数法 `kernel_inlined_size` 量的草稿工具：
-    /// 它照库里的拼法从盘上现读五份 WGSL，拼出的与库编进去的那一份逐字相同时量出的数与它相同，三份都经 naga 解析、校验得过；箭头左边是改之前的钉值）：
+    /// 它照库里的拼法从盘上现读五份 WGSL，拼出的与库编进去的那一份逐字相同时量出的数与它相同，三份都经 naga 解析、校验得过；箭头左边是改之前的钉值）；
+    /// 删除线第 57 批 H1 扫描内核旋钮那一支加判「最右那片之外每片记录数 == 旋钮值」扫描 395 434 → 395 566（第 57 批实现员副本上用草稿工具直接调库里的
+    /// `kernel_inlined_size` 与三个 `*_kernel_source` 量的：在改之前那一份上量回 334 042 / 395 434 / 352 481，与旧钉值逐个相等）：
     /// 改了哪个内核的源码，这里先红，量过新数、确认余量还够之后再改钉值，免得余量悄悄耗光、等上卡时才撞预算。
     #[test]
     fn each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget() {
@@ -1501,8 +1503,8 @@ mod tests {
             (
                 "checker-scan",
                 crate::crash_judge_gpu::checker_scan_kernel_source(),
-                395_434,
-                4_566,
+                395_566,
+                4_434,
             ),
             (
                 "checker-walk",
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 664d256..2f28d00 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -13679,3 +13679,157 @@ fn a_judge_flow_whose_expansion_changes_between_two_asks_is_refused_for_the_segm
         "{refusal:?}"
     );
 }
+
+// ───────────── 第 57 批 H1、H2（判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节）：旋钮下非最右那片装满旋钮值；产品支墓碑那一个调用点 ─────────────
+
+/// H1（GPU 那一份，攻方 `k2_knob_bad_images` 那一形）：旋钮 1 写出的两片各 1 条，GPU 扫描内核与 CPU 新入口都给这一棵旋钮 2：每片都不超旋钮值、
+/// 容器号从 1 逐片加一，只「不是最右的那片装满旋钮值」不成立，CPU 红这一棵的那一条（I-9.22 / I-9.37），GPU 红在同样的状态、同样的位上。
+#[test]
+fn the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_two_tombstone_containers_written_under_the_knob_one(
+) {
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_two_tombstone_containers_written_under_the_knob_one",
+        &record_two_empty_inodes_unlinked_with_one_tombstone_per_container(
+            "crash-judge-gpu-tombstone-knob-two-over-containers-of-one-record",
+        ),
+        knobs_of_the_tombstone_tree(2),
+        KNOB_CAPACITY_INVARIANT_OF_THE_TOMBSTONE_TREE,
+    );
+}
+
+#[test]
+fn the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_two_lineage_containers_written_under_the_knob_one(
+) {
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_two_lineage_containers_written_under_the_knob_one",
+        &record_two_rollbacks_with_one_lineage_record_per_container(
+            "crash-judge-gpu-lineage-knob-two-over-containers-of-one-record",
+        ),
+        knobs_of_the_lineage_tree(2),
+        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
+    );
+}
+
+#[test]
+fn the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_two_retirement_containers_written_under_the_knob_one(
+) {
+    assert_the_gpu_judge_given_the_knobs_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_two_retirement_containers_written_under_the_knob_one",
+        &record_two_leaves_retired_with_one_retirement_record_per_container(
+            "crash-judge-gpu-retirement-knob-two-over-containers-of-one-record",
+        ),
+        knobs_of_the_retirement_tree(2),
+        KNOB_CAPACITY_INVARIANT_OF_THE_LINEAGE_AND_RETIREMENT_TREES,
+    );
+}
+
+/// H2 产品支墓碑那一个调用点要的流：新池新建文件、再建 816 个空 inode（一次发布），照产品容量逐个 unlink 前 815 个到底（墓碑树一片装满 815 条），
+/// 再 unlink 第 816 个（意图那次开第二片、1 条）：墓碑树 [815, 1]，产品镜像合法。交回整条流（从 mkfs 之后录）与第 816 个 unlink 的第一次写
+/// 在写表里的下标——之前的段整段持久、不展开，状态只从那一次 unlink 起取（前 815 次 unlink 约 1630 次发布，逐段展开状态数太大）。
+fn record_a_second_tombstone_container_opened_after_a_full_one_at_product_capacity(
+    tag: &str,
+) -> (RecordedFlow, usize) {
+    let tombstones_filling_one_container = singlefs_format::TOMBSTONE_CONTAINER_RECORDS;
+    let mut pool = pool_with_new_inodes(tag, tombstones_filling_one_container + 1);
+    // 迭代上界：815 个号。
+    for inode in FIRST_FILE.0 + 1..=FIRST_FILE.0 + tombstones_filling_one_container {
+        unlink_an_empty_inode_to_the_end_under_the_capacities(
+            &mut pool,
+            inode,
+            ContainerTreeCapacities::FROM_THE_FORMAT,
+        );
+    }
+    let operations_before_the_last_unlink =
+        pool.stream.operation_count() - pool.mkfs_operation_count;
+    unlink_an_empty_inode_to_the_end_under_the_capacities(
+        &mut pool,
+        FIRST_FILE.0 + tombstones_filling_one_container + 1,
+        ContainerTreeCapacities::FROM_THE_FORMAT,
+    );
+    let tombstone_record_counts: Vec<usize> = pool
+        .output
+        .tombstone_containers
+        .iter()
+        .map(|container| container.contents.records.len())
+        .collect();
+    assert_eq!(
+        tombstone_record_counts,
+        [
+            usize::try_from(tombstones_filling_one_container).expect("装得进 usize"),
+            1
+        ],
+        "墓碑树第一片装满 815、第二片 1 条"
+    );
+    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
+    let operations = pool.retained_operations();
+    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
+        &operations[pool.mkfs_operation_count..],
+        &geometry(),
+    );
+    let first_write_of_the_last_unlink = stream_indexes
+        .iter()
+        .position(|stream_index| *stream_index >= operations_before_the_last_unlink)
+        .expect("第 816 个 unlink 有写");
+    let judged_root_index = writes
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .expect("发布有根槽写");
+    (
+        RecordedFlow {
+            name: "product-tombstone-containers-after-a-full-one",
+            base: pool.memory_pool_after_mkfs(),
+            writes,
+            segments,
+            judged_root_index,
+            versions,
+        },
+        first_write_of_the_last_unlink,
+    )
+}
+
+/// H2（GPU 产品支墓碑那一个调用点，攻方 `mut-w-gap2` 那一形）：产品容量下墓碑树 [815, 1]，第二片容器与根里指它的条目的容器号 2 → 3（1、3）：
+/// 第一片装满、容量那句成立，CPU 产品入口只因「相邻两片加一」红 I-9.21，GPU 红在同样的状态、同样的位上。只展开第 816 个 unlink 的那几段
+/// （之前的段整段持久）；GPU 扫描内核产品支不调三判（`d_judge_tombstone_container_capacity` 里那一个调用点）就在这里与 CPU 不一致。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_product_tombstone_containers_after_a_full_one_skip_a_number(
+) {
+    const TEST_NAME: &str =
+        "the_gpu_judge_agrees_with_the_cpu_when_product_tombstone_containers_after_a_full_one_skip_a_number";
+    let (mut flow, first_write_of_the_last_unlink) =
+        record_a_second_tombstone_container_opened_after_a_full_one_at_product_capacity(
+            "crash-judge-gpu-i921-product-containers-skip-a-number",
+        );
+    renumber_the_last_container_of_the_flow(&mut flow, PACKED_RECORD_TYPE_TOMBSTONE, 1, 3);
+    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
+        return;
+    };
+    let limit = expand_up_to();
+    let expansion = move |_segment_index: usize, segment: &[usize]| {
+        let is_a_segment_of_the_last_unlink = segment
+            .first()
+            .is_some_and(|first_write| *first_write >= first_write_of_the_last_unlink);
+        if is_a_segment_of_the_last_unlink && segment.len() <= limit {
+            Layer0SegmentExpansion::EveryProperSubset
+        } else {
+            Layer0SegmentExpansion::NotExpanded
+        }
+    };
+    let Some(comparison) =
+        compare_on_every_state_of_these_segments(cards, &flow, &flow.segments, &expansion)
+    else {
+        return;
+    };
+    assert!(
+        comparison.cpu_checker_red_invariants.contains("I-9.21"),
+        "流 {} 上 CPU 池级 checker 要红 I-9.21（红过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_red_invariants
+    );
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
+    );
+}
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index be1b0c3..50169ad 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -11978,7 +11978,9 @@ fn judge_tombstones_on_the_lineage_of_a_version_are_distinct(
 /// （第 56 批 G2，判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md` 第三节）：沿叶序第一片容器号是 1、相邻两片加一、
 /// 每片（含最右那片）至少一条——写侧只经两个追加函数开新片，新号 = 最右号 + 1、零根时 1，有新记录才开、没有删片的路
 /// （D18（块里携带什么信息） 已定项 24 第 1、5 条）；再按旋钮：没装判 I-9.36——最右那片之外每片记录数 == 产品容量
-/// （谱系 1359、退役 959；开放容器满了才关），零根或只一片容器没有对象；装了旋钮判 I-9.37——每片记录数 ≤ 旋钮值，零根没有对象。
+/// （谱系 1359、退役 959；开放容器满了才关），零根或只一片容器没有对象；装了旋钮判 I-9.37——每片记录数 ≤ 旋钮值、最右那片之外每片记录数
+/// == 旋钮值（满了才关在旋钮下的那一半，D18（块里携带什么信息） 已定项 24 第 3 条「满了才关；关闭路径加只供测试的容量旋钮」，第 57 批 H1，
+/// 判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节），零根没有对象。
 fn judge_lineage_or_retirement_container_capacity_of_a_version(
     version_name: &str,
     tree_kind: LineageOrRetirementTree,
@@ -12048,6 +12050,22 @@ fn judge_lineage_or_retirement_container_capacity_of_a_version(
                         "{version_name}：{tree_name}（树 {tree}）的容器 {container} 装了 {record_count} 条，多于旋钮值 {records_the_knob_allows}"
                     )
                 });
+                let is_the_rightmost_under_the_knob =
+                    position_in_leaf_order + 1 == container_numbers_and_record_counts.len();
+                if is_the_rightmost_under_the_knob {
+                    continue;
+                }
+                let non_rightmost_container_holds_the_knob_value =
+                    record_count == records_the_knob_allows;
+                judgements.judge(
+                    invariant,
+                    non_rightmost_container_holds_the_knob_value,
+                    || {
+                        format!(
+                            "{version_name}：{tree_name}（树 {tree}）里不是最右的容器 {container} 装了 {record_count} 条，不是旋钮值 {records_the_knob_allows}（满了才关）"
+                        )
+                    },
+                );
             }
         }
     }
@@ -12056,7 +12074,8 @@ fn judge_lineage_or_retirement_container_capacity_of_a_version(
 /// I-9.21 / I-9.22（判决第二节 T5 取 K1、T6 每版）：这一版墓碑树按叶序的容器，读容器头、不沿根的条目读。有对象时（没装旋钮至少两片，
 /// 装了至少一片）判容器号与记录数的三句（第 56 批 G2，同 [`judge_lineage_or_retirement_container_capacity_of_a_version`]）：沿叶序第一片容器号是 1、
 /// 相邻两片加一、每片（含最右那片）至少一条；再按旋钮：没装判 I-9.21——最右那片之外每片记录数 == 815，零根或只一片容器没有对象；
-/// 装了旋钮判 I-9.22——每片记录数 ≤ 旋钮值，零根没有对象。
+/// 装了旋钮判 I-9.22——每片记录数 ≤ 旋钮值、最右那片之外每片记录数 == 旋钮值（满了才关在旋钮下的那一半，第 57 批 H1，
+/// 同 [`judge_lineage_or_retirement_container_capacity_of_a_version`]），零根没有对象。
 fn judge_tombstone_container_capacity_of_a_version(
     version_name: &str,
     tombstone_tree: u64,
@@ -12130,6 +12149,23 @@ fn judge_tombstone_container_capacity_of_a_version(
                         container.container
                     )
                 });
+                let is_the_rightmost_tombstone_container_under_the_knob =
+                    position_in_leaf_order + 1 == containers.len();
+                if is_the_rightmost_tombstone_container_under_the_knob {
+                    continue;
+                }
+                let non_rightmost_tombstone_container_holds_the_knob_value =
+                    record_count == knob_records;
+                judgements.judge(
+                    invariant,
+                    non_rightmost_tombstone_container_holds_the_knob_value,
+                    || {
+                        format!(
+                            "{version_name}：墓碑树（树 {tombstone_tree}）里不是最右的容器 {} 装了 {record_count} 条，不是旋钮值 {knob_records}（满了才关）",
+                            container.container
+                        )
+                    },
+                );
             }
         }
     }
@@ -13926,4 +13962,395 @@ mod tests {
             "extent 树条目的头 ID 不等于表里任何一条 inode 树条目的树 ID：判红"
         );
     }
+
+    // ───── 第 57 批 H1、H2、H3（判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节）：容器容量的两个判法直接交 (容器号, 记录数) 表 ─────
+
+    /// 容量判法判的三棵容器树。
+    #[derive(Clone, Copy, Debug)]
+    enum ContainerTreeOfTheCapacityJudges {
+        Tombstone,
+        Lineage,
+        Retirement,
+    }
+
+    impl ContainerTreeOfTheCapacityJudges {
+        /// 一片容器的产品容量（墓碑 815、谱系 1359、退役 959）。
+        fn product_records_per_container(self) -> u64 {
+            match self {
+                ContainerTreeOfTheCapacityJudges::Tombstone => TOMBSTONE_CONTAINER_RECORDS,
+                ContainerTreeOfTheCapacityJudges::Lineage => {
+                    LineageOrRetirementTree::Lineage.product_records_per_container()
+                }
+                ContainerTreeOfTheCapacityJudges::Retirement => {
+                    LineageOrRetirementTree::Retirement.product_records_per_container()
+                }
+            }
+        }
+
+        /// 装没装旋钮时判法判的那一条（墓碑 I-9.21 / I-9.22，谱系与退役 I-9.36 / I-9.37）。
+        fn capacity_invariant(self, knob_records_per_container: Option<u64>) -> &'static str {
+            match self {
+                ContainerTreeOfTheCapacityJudges::Tombstone => capacity_invariant_of(
+                    knob_records_per_container.map(tombstone_container_capacity_knob_of),
+                ),
+                ContainerTreeOfTheCapacityJudges::Lineage
+                | ContainerTreeOfTheCapacityJudges::Retirement => {
+                    lineage_or_retirement_capacity_invariant_of(knob_records_per_container)
+                }
+            }
+        }
+    }
+
+    fn tombstone_container_capacity_knob_of(
+        records_per_container: u64,
+    ) -> TombstoneContainerCapacityKnob {
+        TombstoneContainerCapacityKnob::new(records_per_container).expect("旋钮值在 1..=815 里")
+    }
+
+    /// 一条整对象墓碑（容量判法只数条数，不读字段）。
+    fn whole_object_tombstone_for_the_capacity_judge() -> TombstoneRecordOfTheScan {
+        TombstoneRecordOfTheScan {
+            object: 2,
+            object_birth: 3,
+            range_start: TOMBSTONE_WHOLE_OBJECT_RANGE_START,
+            range_end: TOMBSTONE_WHOLE_OBJECT_RANGE_END,
+            death_txg: 4,
+        }
+    }
+
+    /// 把沿叶序的 (容器号, 记录数) 表直接交这一棵的容量判法（墓碑树交 [`judge_tombstone_container_capacity_of_a_version`]，
+    /// 谱系树与退役树交 [`judge_lineage_or_retirement_container_capacity_of_a_version`]），旋钮值 `knob_records_per_container`（`None` 是没装）。
+    fn capacity_judgements_of_the_table(
+        tree: ContainerTreeOfTheCapacityJudges,
+        container_numbers_and_record_counts: &[(u64, u64)],
+        knob_records_per_container: Option<u64>,
+    ) -> Judgements {
+        let mut judgements = Judgements::default();
+        let record_count_as_usize = |record_count: u64| {
+            usize::try_from(record_count).expect("一片容器的记录数装得进 usize")
+        };
+        let judge_the_lineage_or_retirement_table =
+            |tree_kind: LineageOrRetirementTree,
+             tree_identifier: u64,
+             judgements_of_the_table: &mut Judgements| {
+                let table: Vec<(u64, usize)> = container_numbers_and_record_counts
+                    .iter()
+                    .map(|(container, record_count)| {
+                        (*container, record_count_as_usize(*record_count))
+                    })
+                    .collect();
+                judge_lineage_or_retirement_container_capacity_of_a_version(
+                    "单测那一版",
+                    tree_kind,
+                    tree_identifier,
+                    &table,
+                    knob_records_per_container,
+                    judgements_of_the_table,
+                );
+            };
+        match tree {
+            ContainerTreeOfTheCapacityJudges::Tombstone => {
+                let containers: Vec<TombstoneContainerOfTheScan> =
+                    container_numbers_and_record_counts
+                        .iter()
+                        .map(|(container, record_count)| TombstoneContainerOfTheScan {
+                            container: *container,
+                            records: vec![
+                                whole_object_tombstone_for_the_capacity_judge();
+                                record_count_as_usize(*record_count)
+                            ],
+                        })
+                        .collect();
+                judge_tombstone_container_capacity_of_a_version(
+                    "单测那一版",
+                    TREE_IDENTIFIER_TOMBSTONE,
+                    &containers,
+                    knob_records_per_container.map(tombstone_container_capacity_knob_of),
+                    &mut judgements,
+                );
+            }
+            ContainerTreeOfTheCapacityJudges::Lineage => judge_the_lineage_or_retirement_table(
+                LineageOrRetirementTree::Lineage,
+                TREE_IDENTIFIER_LINEAGE,
+                &mut judgements,
+            ),
+            ContainerTreeOfTheCapacityJudges::Retirement => judge_the_lineage_or_retirement_table(
+                LineageOrRetirementTree::Retirement,
+                singlefs_format::TREE_IDENTIFIER_RETIREMENT,
+                &mut judgements,
+            ),
+        }
+        judgements
+    }
+
+    /// 这张表交这一棵的容量判法：那一条只红一处（只一句不成立），违例说明里有 `detail_fragment`。
+    fn assert_the_capacity_judge_violates_only_one_sentence(
+        tree: ContainerTreeOfTheCapacityJudges,
+        container_numbers_and_record_counts: &[(u64, u64)],
+        knob_records_per_container: Option<u64>,
+        detail_fragment: &str,
+    ) {
+        let invariant = tree.capacity_invariant(knob_records_per_container);
+        let judgements = capacity_judgements_of_the_table(
+            tree,
+            container_numbers_and_record_counts,
+            knob_records_per_container,
+        );
+        assert_eq!(
+            judgements.violation_count(invariant),
+            1,
+            "{tree:?} 表 {container_numbers_and_record_counts:?}（旋钮 {knob_records_per_container:?}）：{invariant} 只一句不成立（第一处：{:?}）",
+            judgements.first_violation_of(invariant)
+        );
+        assert!(
+            judgements
+                .first_violation_of(invariant)
+                .is_some_and(|detail| detail.contains(detail_fragment)),
+            "{tree:?} 表 {container_numbers_and_record_counts:?}：{invariant} 的违例说明写出「{detail_fragment}」，实际 {:?}",
+            judgements.first_violation_of(invariant)
+        );
+    }
+
+    /// 这张表交这一棵的容量判法：那一条判过、一处都不红。
+    fn assert_the_capacity_judge_holds(
+        tree: ContainerTreeOfTheCapacityJudges,
+        container_numbers_and_record_counts: &[(u64, u64)],
+        knob_records_per_container: Option<u64>,
+    ) {
+        let invariant = tree.capacity_invariant(knob_records_per_container);
+        let judgements = capacity_judgements_of_the_table(
+            tree,
+            container_numbers_and_record_counts,
+            knob_records_per_container,
+        );
+        assert_eq!(
+            judgements.violation_count(invariant),
+            0,
+            "{tree:?} 表 {container_numbers_and_record_counts:?}（旋钮 {knob_records_per_container:?}）：{invariant} 一处都不红（第一处：{:?}）",
+            judgements.first_violation_of(invariant)
+        );
+        assert!(
+            matches!(
+                judgements
+                    .into_report()
+                    .into_iter()
+                    .find(|(name, _)| *name == invariant)
+                    .map(|(_, verdict)| verdict),
+                Some(InvariantVerdict::Holds)
+            ),
+            "{tree:?} 表 {container_numbers_and_record_counts:?}：{invariant} 判过而成立"
+        );
+    }
+
+    /// H2 产品支（没装旋钮）的对照：第一片装满产品容量、号 1、2，最右那片 1 条：判过、一处不红（下面三形各只改一处）。
+    fn assert_a_full_first_container_and_one_more_hold_on_the_product_branch(
+        tree: ContainerTreeOfTheCapacityJudges,
+    ) {
+        let full = tree.product_records_per_container();
+        assert_the_capacity_judge_holds(tree, &[(1, full), (2, 1)], None);
+    }
+
+    /// H2 产品支第一形（攻方 `mut-w-start2`）：第一片装满、号 2、3——容量那句成立，只「第一片容器号是 1」那句不成立。
+    fn assert_product_containers_numbered_two_and_three_violate_only_the_first_number_sentence(
+        tree: ContainerTreeOfTheCapacityJudges,
+    ) {
+        let full = tree.product_records_per_container();
+        assert_the_capacity_judge_violates_only_one_sentence(
+            tree,
+            &[(2, full), (3, 1)],
+            None,
+            "沿叶序第一片容器的容器号是 2，不是 1",
+        );
+    }
+
+    /// H2 产品支第二形（攻方 `mut-w-gap2`）：第一片装满、号 1、3——只「相邻两片加一」那句不成立。
+    fn assert_product_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
+        tree: ContainerTreeOfTheCapacityJudges,
+    ) {
+        let full = tree.product_records_per_container();
+        assert_the_capacity_judge_violates_only_one_sentence(
+            tree,
+            &[(1, full), (3, 1)],
+            None,
+            "沿叶序容器号不是逐片加一：容器 1 之后是容器 3",
+        );
+    }
+
+    /// H2 产品支第三形（攻方 `mut-w-eager`）：第一片装满、号 1、2，最右那片 0 条——只「每片至少一条」那句不成立。
+    fn assert_product_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
+        tree: ContainerTreeOfTheCapacityJudges,
+    ) {
+        let full = tree.product_records_per_container();
+        assert_the_capacity_judge_violates_only_one_sentence(
+            tree,
+            &[(1, full), (2, 0)],
+            None,
+            "容器 2 一条记录都没有",
+        );
+    }
+
+    /// H3 产品支：第一片装满、号 0、1——相邻加一成立，只「第一片容器号是 1」那句不成立（「第一片 ≤ 1」放过它）。
+    fn assert_product_containers_numbered_zero_and_one_violate_only_the_first_number_sentence(
+        tree: ContainerTreeOfTheCapacityJudges,
+    ) {
+        let full = tree.product_records_per_container();
+        assert_the_capacity_judge_violates_only_one_sentence(
+            tree,
+            &[(0, full), (1, 1)],
+            None,
+            "沿叶序第一片容器的容器号是 0，不是 1",
+        );
+    }
+
+    /// H1 旋钮支：旋钮 1 写出的两片各 1 条交旋钮 2 判（攻方 `k2_knob_bad_images` 那一形）——容器号与「≤ 旋钮值」都成立，
+    /// 只「不是最右的那片装满旋钮值」那句不成立；同一张表交旋钮 1 判过、一处不红。
+    fn assert_two_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
+        tree: ContainerTreeOfTheCapacityJudges,
+    ) {
+        assert_the_capacity_judge_holds(tree, &[(1, 1), (2, 1)], Some(1));
+        assert_the_capacity_judge_violates_only_one_sentence(
+            tree,
+            &[(1, 1), (2, 1)],
+            Some(2),
+            "不是最右的容器 1 装了 1 条，不是旋钮值 2",
+        );
+    }
+
+    #[test]
+    fn a_full_first_tombstone_container_and_one_more_hold_on_the_product_branch() {
+        assert_a_full_first_container_and_one_more_hold_on_the_product_branch(
+            ContainerTreeOfTheCapacityJudges::Tombstone,
+        );
+    }
+
+    #[test]
+    fn a_full_first_lineage_container_and_one_more_hold_on_the_product_branch() {
+        assert_a_full_first_container_and_one_more_hold_on_the_product_branch(
+            ContainerTreeOfTheCapacityJudges::Lineage,
+        );
+    }
+
+    #[test]
+    fn a_full_first_retirement_container_and_one_more_hold_on_the_product_branch() {
+        assert_a_full_first_container_and_one_more_hold_on_the_product_branch(
+            ContainerTreeOfTheCapacityJudges::Retirement,
+        );
+    }
+
+    #[test]
+    fn product_tombstone_containers_numbered_two_and_three_violate_only_the_first_number_sentence()
+    {
+        assert_product_containers_numbered_two_and_three_violate_only_the_first_number_sentence(
+            ContainerTreeOfTheCapacityJudges::Tombstone,
+        );
+    }
+
+    #[test]
+    fn product_lineage_containers_numbered_two_and_three_violate_only_the_first_number_sentence() {
+        assert_product_containers_numbered_two_and_three_violate_only_the_first_number_sentence(
+            ContainerTreeOfTheCapacityJudges::Lineage,
+        );
+    }
+
+    #[test]
+    fn product_retirement_containers_numbered_two_and_three_violate_only_the_first_number_sentence()
+    {
+        assert_product_containers_numbered_two_and_three_violate_only_the_first_number_sentence(
+            ContainerTreeOfTheCapacityJudges::Retirement,
+        );
+    }
+
+    #[test]
+    fn product_tombstone_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
+    ) {
+        assert_product_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
+            ContainerTreeOfTheCapacityJudges::Tombstone,
+        );
+    }
+
+    #[test]
+    fn product_lineage_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence()
+    {
+        assert_product_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
+            ContainerTreeOfTheCapacityJudges::Lineage,
+        );
+    }
+
+    #[test]
+    fn product_retirement_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
+    ) {
+        assert_product_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence(
+            ContainerTreeOfTheCapacityJudges::Retirement,
+        );
+    }
+
+    #[test]
+    fn product_tombstone_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
+    ) {
+        assert_product_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
+            ContainerTreeOfTheCapacityJudges::Tombstone,
+        );
+    }
+
+    #[test]
+    fn product_lineage_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence()
+    {
+        assert_product_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
+            ContainerTreeOfTheCapacityJudges::Lineage,
+        );
+    }
+
+    #[test]
+    fn product_retirement_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
+    ) {
+        assert_product_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence(
+            ContainerTreeOfTheCapacityJudges::Retirement,
+        );
+    }
+
+    #[test]
+    fn product_tombstone_containers_numbered_zero_and_one_violate_only_the_first_number_sentence() {
+        assert_product_containers_numbered_zero_and_one_violate_only_the_first_number_sentence(
+            ContainerTreeOfTheCapacityJudges::Tombstone,
+        );
+    }
+
+    #[test]
+    fn product_lineage_containers_numbered_zero_and_one_violate_only_the_first_number_sentence() {
+        assert_product_containers_numbered_zero_and_one_violate_only_the_first_number_sentence(
+            ContainerTreeOfTheCapacityJudges::Lineage,
+        );
+    }
+
+    #[test]
+    fn product_retirement_containers_numbered_zero_and_one_violate_only_the_first_number_sentence()
+    {
+        assert_product_containers_numbered_zero_and_one_violate_only_the_first_number_sentence(
+            ContainerTreeOfTheCapacityJudges::Retirement,
+        );
+    }
+
+    #[test]
+    fn two_tombstone_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
+    ) {
+        assert_two_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
+            ContainerTreeOfTheCapacityJudges::Tombstone,
+        );
+    }
+
+    #[test]
+    fn two_lineage_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
+    ) {
+        assert_two_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
+            ContainerTreeOfTheCapacityJudges::Lineage,
+        );
+    }
+
+    #[test]
+    fn two_retirement_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
+    ) {
+        assert_two_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence(
+            ContainerTreeOfTheCapacityJudges::Retirement,
+        );
+    }
 }
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 979b679..e8dddf3 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -16705,15 +16705,30 @@ fn assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
     image: &MemoryPool,
     tree: ContainerTreeOfTheCapacityInvariants,
     detail_fragment: &str,
+) {
+    assert_only_the_knob_capacity_invariant_of_the_tree_reddens_given_the_knob(
+        image,
+        tree,
+        1,
+        detail_fragment,
+    );
+}
+
+/// 带旋钮值的入口（只这一棵装一片 `records_per_container` 条）上只红这一棵的那一条（I-9.22 / I-9.37），违例说明里有 `detail_fragment`。
+fn assert_only_the_knob_capacity_invariant_of_the_tree_reddens_given_the_knob(
+    image: &MemoryPool,
+    tree: ContainerTreeOfTheCapacityInvariants,
+    records_per_container: u64,
+    detail_fragment: &str,
 ) {
     let verdicts = singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
         image,
-        tree.checker_knobs(1),
+        tree.checker_knobs(records_per_container),
     );
     assert_eq!(
         violated_invariants(&verdicts),
         [tree.knob_capacity_invariant()],
-        "{tree:?}：只红 {}：{verdicts:?}",
+        "{tree:?}（旋钮值 {records_per_container}）：只红 {}：{verdicts:?}",
         tree.knob_capacity_invariant()
     );
     assert!(
@@ -16877,6 +16892,119 @@ fn retirement_containers_numbered_one_and_one_redden_only_the_knob_capacity_inva
     );
 }
 
+/// 第 57 批 H3（旋钮支那一形，判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节；正推 K2.2）：两片容器 1、2 号改成 0、1 号——
+/// 相邻加一成立、每片 1 条，只「沿叶序第一片容器号是 1」那句不成立（判成「第一片 ≤ 1」就放过它）。
+fn assert_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant(
+    tree: ContainerTreeOfTheCapacityInvariants,
+    tag: &str,
+) {
+    let mut image =
+        image_with_records_in_one_container_tree_and_the_root_ring_turned_over(tree, tag, 2);
+    renumber_the_containers_of_the_newest_tree(&mut image, tree, &[0, 1]);
+    assert_only_the_knob_capacity_invariant_of_the_tree_reddens(
+        &image,
+        tree,
+        "沿叶序第一片容器的容器号是 0，不是 1",
+    );
+}
+
+#[test]
+fn tombstone_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant() {
+    assert_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant(
+        ContainerTreeOfTheCapacityInvariants::Tombstone,
+        "known-bad-tombstone-containers-numbered-zero-and-one",
+    );
+}
+
+#[test]
+fn lineage_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant() {
+    assert_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant(
+        ContainerTreeOfTheCapacityInvariants::Lineage,
+        "known-bad-lineage-containers-numbered-zero-and-one",
+    );
+}
+
+#[test]
+fn retirement_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant() {
+    assert_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant(
+        ContainerTreeOfTheCapacityInvariants::Retirement,
+        "known-bad-retirement-containers-numbered-zero-and-one",
+    );
+}
+
+/// 第 57 批 H1（攻方 `research/prompts/m3-batch56-code-r1-opus-model/probe-crate/tests/k2_knob_bad_images.rs` 那一形，判决同上第三节）：
+/// 只这一棵在旋钮 1 下写出两片各 1 条（另两棵照产品）。写侧在旋钮 K 下装满 K 条才关（D18（块里携带什么信息） 已定项 24 第 3 条「满了才关；
+/// 关闭路径加只供测试的容量旋钮」），旋钮 2、3 写不出「不是最右的那片只装 1 条」。带旋钮值的入口：报写的时候那个旋钮值 1 一条不红、
+/// 这一棵的那一条（I-9.22 / I-9.37）判过而成立；报旋钮 2、3——每片都不超旋钮值、容器号从 1 逐片加一、每片至少一条，只「不是最右的那片装满
+/// 旋钮值」那句不成立，只红这一棵的那一条。
+fn assert_two_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
+    tree: ContainerTreeOfTheCapacityInvariants,
+    tag: &str,
+) {
+    let pool = pool_with_records_in_one_container_tree_under_its_knob(tree, tag, 1, 2);
+    assert_eq!(
+        tree.record_counts_of_each_container(&pool.output),
+        [1, 1],
+        "{tree:?} 两片各 1 条"
+    );
+    let image = pool.memory_pool();
+    let verdicts_given_the_written_knob =
+        singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(
+            &image,
+            tree.checker_knobs(1),
+        );
+    assert_eq!(
+        violated_invariants(&verdicts_given_the_written_knob),
+        Vec::<&str>::new(),
+        "{tree:?}：报写的时候那个旋钮值 1 一条不红：{verdicts_given_the_written_knob:?}"
+    );
+    assert_eq!(
+        verdict_among(
+            &verdicts_given_the_written_knob,
+            tree.knob_capacity_invariant()
+        ),
+        InvariantVerdict::Holds,
+        "{tree:?}：报旋钮值 1 时 {} 判过而成立",
+        tree.knob_capacity_invariant()
+    );
+    // 迭代上界：两个旋钮值。
+    for larger_knob in [2, 3] {
+        assert_only_the_knob_capacity_invariant_of_the_tree_reddens_given_the_knob(
+            &image,
+            tree,
+            larger_knob,
+            &format!("不是最右的容器 1 装了 1 条，不是旋钮值 {larger_knob}"),
+        );
+    }
+}
+
+#[test]
+fn two_tombstone_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
+) {
+    assert_two_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
+        ContainerTreeOfTheCapacityInvariants::Tombstone,
+        "known-bad-tombstone-containers-closed-before-a-larger-knob",
+    );
+}
+
+#[test]
+fn two_lineage_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
+) {
+    assert_two_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
+        ContainerTreeOfTheCapacityInvariants::Lineage,
+        "known-bad-lineage-containers-closed-before-a-larger-knob",
+    );
+}
+
+#[test]
+fn two_retirement_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
+) {
+    assert_two_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three(
+        ContainerTreeOfTheCapacityInvariants::Retirement,
+        "known-bad-retirement-containers-closed-before-a-larger-knob",
+    );
+}
+
 /// G3 ③：退役容器压到一片 2 条写出两条退役记录（一片 2 条），带旋钮值的入口报退役旋钮 1：那片装了 2 条、多于旋钮值，只红 I-9.37；
 /// 报写的时候那个旋钮值 2 一条不红。
 #[test]
diff --git a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
index 3a4fbfa..375060d 100644
--- a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
+++ b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
@@ -4,7 +4,8 @@
 //! 池级 checker 从镜像外拿墓碑容器的旋钮值（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T5 取 K1）：
 //! 这些镜像走带旋钮值的入口，I-9.22（测试镜像里墓碑容器不超旋钮值） 按旋钮值判；同一张镜像交给旧入口（当没装旋钮）I-9.21 红。
 //! 谱系容器同样压到一片 1 条，带旋钮值的入口一并给谱系的旋钮值（I-9.37，第 53 批；不给的话回退两次写出的两片谱系容器按产品容量判 I-9.36 红）。
-//! 第 56 批（G2）：三棵容器树都压到一片 1 条的一段固定种子随机历史（建号、unlink、回退），每一步交带旋钮值的入口一条不红（慢档）。
+//! 第 56 批（G2）：三棵容器树都压到一片 1 条的一段固定种子随机历史（建号、unlink、回退），每一步交带旋钮值的入口一条不红（慢档）；
+//! 第 57 批（H1）：同一段历史压到一片 2 条，旋钮下「不是最右的那片装满旋钮值」真判到多片（慢档）。
 
 mod common;
 mod common_deletion_in_progress;
@@ -521,19 +522,24 @@ const RANDOM_HISTORY_OPERATIONS_UNDER_EVERY_CONTAINER_KNOB: u64 = 60;
 /// 回退的目标从最近这么多版里挑：每步至多三次发布，挑到的那一版留在根环一圈里（回退候选集）。
 const ROLLBACK_TARGETS_KEPT: usize = 8;
 
-/// 三棵容器树都压到一片 1 条、一个码 2 节点 2 条（第 3 片容器根满长一层）。
-fn capacities_of_one_record_in_every_container_tree_and_two_entries() -> ContainerTreeCapacities {
+/// 三棵容器树都压到一片 `records_per_container` 条、一个码 2 节点 2 条（第 3 片容器根满长一层）。
+fn capacities_of_every_container_tree_at_and_two_entries(
+    records_per_container: u64,
+) -> ContainerTreeCapacities {
     ContainerTreeCapacities {
         tombstone_container: TombstoneContainerCapacity::CappedForTests(
-            TombstoneRecordsPerContainerCappedForTests::new(1).expect("1..=815"),
+            TombstoneRecordsPerContainerCappedForTests::new(records_per_container)
+                .expect("1..=815"),
         ),
         lineage_container: LineageContainerCapacity::CappedForTests(
-            LineageRecordsPerContainerCappedForTests::new(1).expect("1..=1359"),
+            LineageRecordsPerContainerCappedForTests::new(records_per_container).expect("1..=1359"),
         ),
         retirement_container:
             singlefs_core::retirement_tree::RetirementContainerCapacity::CappedForTests(
-                singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(1)
-                    .expect("1..=959"),
+                singlefs_core::retirement_tree::RetirementRecordsPerContainerCappedForTests::new(
+                    records_per_container,
+                )
+                .expect("1..=959"),
             ),
         node_entries: ContainerTreeNodeEntries::CappedForTests(
             ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135"),
@@ -541,13 +547,15 @@ fn capacities_of_one_record_in_every_container_tree_and_two_entries() -> Contain
     }
 }
 
-/// 同一组旋钮值交给池级 checker 的新入口（三棵都是一片 1 条）。
-fn knobs_of_one_record_in_every_container_tree() -> ContainerCapacityKnobs {
+/// 同一组旋钮值交给池级 checker 的新入口（三棵都是一片 `records_per_container` 条）。
+fn knobs_of_every_container_tree_at(records_per_container: u64) -> ContainerCapacityKnobs {
     ContainerCapacityKnobs {
-        tombstone: Some(tombstone_container_capacity_knob(1)),
-        lineage: Some(LineageContainerCapacityKnob::new(1).expect("旋钮值在 1..=1359 里")),
+        tombstone: Some(tombstone_container_capacity_knob(records_per_container)),
+        lineage: Some(
+            LineageContainerCapacityKnob::new(records_per_container).expect("旋钮值在 1..=1359 里"),
+        ),
         retirement: Some(
-            singlefs_checker::walk::RetirementContainerCapacityKnob::new(1)
+            singlefs_checker::walk::RetirementContainerCapacityKnob::new(records_per_container)
                 .expect("旋钮值在 1..=959 里"),
         ),
     }
@@ -561,14 +569,40 @@ fn knobs_of_one_record_in_every_container_tree() -> ContainerCapacityKnobs {
 #[test]
 #[ignore = "harness 慢档用例：60 步随机历史、每步一次池级 checker，debug 下约两分钟"]
 fn a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step() {
+    assert_a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step(
+        1,
+        "container-trees-random-history-under-every-knob",
+        3,
+    );
+}
+
+/// 第 57 批 H1（判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节；攻方 `k2_random_histories` 那一类）：同一段固定种子
+/// 随机历史，三棵容器树都压到一片 2 条（写侧装满 2 条才关，D18（块里携带什么信息） 已定项 24 第 3 条），每一步之后带同一组旋钮值 2 交新入口，
+/// 一条都不许红；历史走完三棵树各至少两片容器，「不是最右的那片装满旋钮值」真判到了。写侧早关一片（攻方 `w-close-one-early`：装 1 条就开新片）
+/// 写出的镜像在这里红。
+#[test]
+#[ignore = "harness 慢档用例：60 步随机历史、每步一次池级 checker，debug 下约两分钟"]
+fn a_random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step() {
+    assert_a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step(
+        2,
+        "container-trees-random-history-under-every-knob-of-two-records",
+        2,
+    );
+}
+
+/// 上面两条的本体：三棵容器树都压到一片 `records_per_container` 条、一个节点 2 条，跑 [`RANDOM_HISTORY_SEED_UNDER_EVERY_CONTAINER_KNOB`] 那一段
+/// 随机历史，每一步之后带同一组旋钮值交新入口一条都不红；历史走完三棵树各至少 `fewest_containers_of_each_tree_at_the_end` 片容器，
+/// I-9.22、I-9.37 真被评估过且成立。
+fn assert_a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step(
+    records_per_container: u64,
+    tag: &str,
+    fewest_containers_of_each_tree_at_the_end: usize,
+) {
     let mut random = singlefs_harness::history::SeededRandomSource::from_seed(
         RANDOM_HISTORY_SEED_UNDER_EVERY_CONTAINER_KNOB,
     );
-    let capacities = capacities_of_one_record_in_every_container_tree_and_two_entries();
-    let mut pool = pool_with_new_inodes(
-        "container-trees-random-history-under-every-knob",
-        singlefs_format::INODE_LEAF_RECORDS,
-    );
+    let capacities = capacities_of_every_container_tree_at_and_two_entries(records_per_container);
+    let mut pool = pool_with_new_inodes(tag, singlefs_format::INODE_LEAF_RECORDS);
     pool.allocator.set_deletion_switches(DeletionSwitches {
         batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
         tombstone_container_capacity: capacities.tombstone_container,
@@ -658,7 +692,7 @@ fn a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step(
         }
         let verdicts = check_pool_image_given_the_container_capacity_knobs(
             &pool.memory_pool(),
-            knobs_of_one_record_in_every_container_tree(),
+            knobs_of_every_container_tree_at(records_per_container),
         );
         let violated: Vec<String> = verdicts
             .iter()
@@ -672,7 +706,7 @@ fn a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step(
         assert_eq!(
             violated,
             Vec::<String>::new(),
-            "第 {step} 步之后的合法镜像（带同一组旋钮值）一条都不红"
+            "第 {step} 步之后的合法镜像（带同一组旋钮值 {records_per_container}）一条都不红"
         );
         knob_invariants_held.extend(
             verdicts
@@ -692,14 +726,16 @@ fn a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step(
         pool.output.retirement_containers.len(),
     );
     eprintln!(
-        "CONTAINER_TREES_RANDOM_HISTORY seed={RANDOM_HISTORY_SEED_UNDER_EVERY_CONTAINER_KNOB} containers(tombstone, lineage, retirement)={container_counts:?} levels(tombstone, lineage, retirement)=({}, {}, {})",
+        "CONTAINER_TREES_RANDOM_HISTORY seed={RANDOM_HISTORY_SEED_UNDER_EVERY_CONTAINER_KNOB} records_per_container={records_per_container} containers(tombstone, lineage, retirement)={container_counts:?} levels(tombstone, lineage, retirement)=({}, {}, {})",
         pool.output.tombstone_tree_index_nodes.shape.root_level(),
         pool.output.lineage_tree_index_nodes.shape.root_level(),
         pool.output.retirement_tree_index_nodes.shape.root_level()
     );
     assert!(
-        container_counts.0 >= 3 && container_counts.1 >= 3 && container_counts.2 >= 3,
-        "历史走完三棵树各至少三片容器：{container_counts:?}"
+        container_counts.0 >= fewest_containers_of_each_tree_at_the_end
+            && container_counts.1 >= fewest_containers_of_each_tree_at_the_end
+            && container_counts.2 >= fewest_containers_of_each_tree_at_the_end,
+        "历史走完三棵树各至少 {fewest_containers_of_each_tree_at_the_end} 片容器：{container_counts:?}"
     );
     for invariant in [
         TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
`````

### mutations-append.tsv

`````text
第57批 H2 I-9.21：产品支不判第一片容器号是 1（墓碑树那一形，正推 c8 / 攻方 c-product-skips-three 第一句）	crates/singlefs-checker/src/walk.rs	            first_in_leaf_order.container == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	            first_in_leaf_order.container == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE\n                || capacity_knob.is_none(),	-p singlefs-checker --lib -- product_tombstone_containers_numbered_two_and_three	product_tombstone_containers_numbered_two_and_three_violate_only_the_first_number_sentence
第57批 H2 I-9.21：产品支不判相邻两片加一（墓碑树那一形，正推 c9 / 攻方 c-product-skips-three 第二句）	crates/singlefs-checker/src/walk.rs	            left.container.checked_add(1) == Some(right.container);	            left.container.checked_add(1) == Some(right.container)\n                || capacity_knob.is_none();	-p singlefs-checker --lib -- product_tombstone_containers_numbered_one_and_three	product_tombstone_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence
第57批 H2 I-9.21：产品支不判每片至少一条（墓碑树那一形，正推 c7 / 攻方 c-product-skips-three 第三句）	crates/singlefs-checker/src/walk.rs	        let tombstone_container_holds_a_record = record_count > 0;	        let tombstone_container_holds_a_record = record_count > 0 || capacity_knob.is_none();	-p singlefs-checker --lib -- product_tombstone_containers_with_an_empty_rightmost_one	product_tombstone_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence
第57批 H2 I-9.21：相邻两片要求加二（墓碑树产品支合法那一形上误报）	crates/singlefs-checker/src/walk.rs	            left.container.checked_add(1) == Some(right.container);	            left.container.checked_add(2) == Some(right.container);	-p singlefs-checker --lib -- a_full_first_tombstone_container_and_one_more_hold	a_full_first_tombstone_container_and_one_more_hold_on_the_product_branch
第57批 H3 I-9.21：第一片容器号判成 ≤ 1（墓碑树产品支那一形，正推 c3）	crates/singlefs-checker/src/walk.rs	            first_in_leaf_order.container == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	            first_in_leaf_order.container <= FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	-p singlefs-checker --lib -- product_tombstone_containers_numbered_zero_and_one	product_tombstone_containers_numbered_zero_and_one_violate_only_the_first_number_sentence
第57批 H3 I-9.22：第一片容器号判成 ≤ 1（墓碑树旋钮支坏镜像，正推 c3）	crates/singlefs-checker/src/walk.rs	            first_in_leaf_order.container == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	            first_in_leaf_order.container <= FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	-p singlefs-harness --test checker_known_bad_images -- tombstone_containers_numbered_zero_and_one	tombstone_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant
第57批 H1 I-9.22：旋钮支不判不是最右的那片装满旋钮值（墓碑树单测那一形）	crates/singlefs-checker/src/walk.rs	                    record_count == knob_records;	                    record_count <= knob_records;	-p singlefs-checker --lib -- two_tombstone_containers_of_one_record_judged_with_the_knob_two	two_tombstone_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence
第57批 H1 I-9.22：旋钮支不判不是最右的那片装满旋钮值（墓碑树旋钮 1 写出、旋钮 2 / 3 判的坏镜像，攻方 k2_knob_bad_images）	crates/singlefs-checker/src/walk.rs	                    record_count == knob_records;	                    record_count <= knob_records;	-p singlefs-harness --test checker_known_bad_images -- two_tombstone_containers_written_under_the_knob_one	two_tombstone_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three
第57批 H1 GPU 判器：旋钮那一支不判不是最右的那片装满旋钮值（墓碑树那一形）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (d_container_record_count(container_index) != knob_records) {	        if (false && d_container_record_count(container_index) != knob_records) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_two_tombstone_containers_written_under_the_knob_one	the_gpu_judge_given_the_tombstone_knob_two_agrees_with_the_cpu_on_two_tombstone_containers_written_under_the_knob_one
第57批 H2 I-9.36：产品支不判第一片容器号是 1（谱系树那一形，正推 c8 / 攻方 c-product-skips-three 第一句）	crates/singlefs-checker/src/walk.rs	            first_container_in_leaf_order == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	            first_container_in_leaf_order == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE\n                || knob_records_per_container.is_none(),	-p singlefs-checker --lib -- product_lineage_containers_numbered_two_and_three	product_lineage_containers_numbered_two_and_three_violate_only_the_first_number_sentence
第57批 H2 I-9.36：产品支不判相邻两片加一（谱系树那一形，正推 c9 / 攻方 c-product-skips-three 第二句）	crates/singlefs-checker/src/walk.rs	            left_container.checked_add(1) == Some(right_container);	            left_container.checked_add(1) == Some(right_container)\n                || knob_records_per_container.is_none();	-p singlefs-checker --lib -- product_lineage_containers_numbered_one_and_three	product_lineage_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence
第57批 H2 I-9.36：产品支不判每片至少一条（谱系树那一形，正推 c7 / 攻方 c-product-skips-three 第三句）	crates/singlefs-checker/src/walk.rs	        let lineage_or_retirement_container_holds_a_record = record_count > 0;	        let lineage_or_retirement_container_holds_a_record =\n            record_count > 0 || knob_records_per_container.is_none();	-p singlefs-checker --lib -- product_lineage_containers_with_an_empty_rightmost_one	product_lineage_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence
第57批 H2 I-9.36：相邻两片要求加二（谱系树产品支合法那一形上误报）	crates/singlefs-checker/src/walk.rs	            left_container.checked_add(1) == Some(right_container);	            left_container.checked_add(2) == Some(right_container);	-p singlefs-checker --lib -- a_full_first_lineage_container_and_one_more_hold	a_full_first_lineage_container_and_one_more_hold_on_the_product_branch
第57批 H3 I-9.36：第一片容器号判成 ≤ 1（谱系树产品支那一形，正推 c3）	crates/singlefs-checker/src/walk.rs	            first_container_in_leaf_order == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	            first_container_in_leaf_order <= FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	-p singlefs-checker --lib -- product_lineage_containers_numbered_zero_and_one	product_lineage_containers_numbered_zero_and_one_violate_only_the_first_number_sentence
第57批 H3 I-9.37：第一片容器号判成 ≤ 1（谱系树旋钮支坏镜像，正推 c3）	crates/singlefs-checker/src/walk.rs	            first_container_in_leaf_order == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	            first_container_in_leaf_order <= FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	-p singlefs-harness --test checker_known_bad_images -- lineage_containers_numbered_zero_and_one	lineage_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant
第57批 H1 I-9.37：旋钮支不判不是最右的那片装满旋钮值（谱系树单测那一形）	crates/singlefs-checker/src/walk.rs	                    record_count == records_the_knob_allows;	                    record_count <= records_the_knob_allows;	-p singlefs-checker --lib -- two_lineage_containers_of_one_record_judged_with_the_knob_two	two_lineage_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence
第57批 H1 I-9.37：旋钮支不判不是最右的那片装满旋钮值（谱系树旋钮 1 写出、旋钮 2 / 3 判的坏镜像，攻方 k2_knob_bad_images）	crates/singlefs-checker/src/walk.rs	                    record_count == records_the_knob_allows;	                    record_count <= records_the_knob_allows;	-p singlefs-harness --test checker_known_bad_images -- two_lineage_containers_written_under_the_knob_one	two_lineage_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three
第57批 H1 GPU 判器：旋钮那一支不判不是最右的那片装满旋钮值（谱系树那一形）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (d_container_record_count(container_index) != knob_records) {	        if (false && d_container_record_count(container_index) != knob_records) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_two_lineage_containers_written_under_the_knob_one	the_gpu_judge_given_the_lineage_knob_two_agrees_with_the_cpu_on_two_lineage_containers_written_under_the_knob_one
第57批 H2 I-9.36：产品支不判第一片容器号是 1（退役树那一形，正推 c8 / 攻方 c-product-skips-three 第一句）	crates/singlefs-checker/src/walk.rs	            first_container_in_leaf_order == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	            first_container_in_leaf_order == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE\n                || knob_records_per_container.is_none(),	-p singlefs-checker --lib -- product_retirement_containers_numbered_two_and_three	product_retirement_containers_numbered_two_and_three_violate_only_the_first_number_sentence
第57批 H2 I-9.36：产品支不判相邻两片加一（退役树那一形，正推 c9 / 攻方 c-product-skips-three 第二句）	crates/singlefs-checker/src/walk.rs	            left_container.checked_add(1) == Some(right_container);	            left_container.checked_add(1) == Some(right_container)\n                || knob_records_per_container.is_none();	-p singlefs-checker --lib -- product_retirement_containers_numbered_one_and_three	product_retirement_containers_numbered_one_and_three_violate_only_the_adjacent_number_sentence
第57批 H2 I-9.36：产品支不判每片至少一条（退役树那一形，正推 c7 / 攻方 c-product-skips-three 第三句）	crates/singlefs-checker/src/walk.rs	        let lineage_or_retirement_container_holds_a_record = record_count > 0;	        let lineage_or_retirement_container_holds_a_record =\n            record_count > 0 || knob_records_per_container.is_none();	-p singlefs-checker --lib -- product_retirement_containers_with_an_empty_rightmost_one	product_retirement_containers_with_an_empty_rightmost_one_violate_only_the_non_empty_sentence
第57批 H2 I-9.36：相邻两片要求加二（退役树产品支合法那一形上误报）	crates/singlefs-checker/src/walk.rs	            left_container.checked_add(1) == Some(right_container);	            left_container.checked_add(2) == Some(right_container);	-p singlefs-checker --lib -- a_full_first_retirement_container_and_one_more_hold	a_full_first_retirement_container_and_one_more_hold_on_the_product_branch
第57批 H3 I-9.36：第一片容器号判成 ≤ 1（退役树产品支那一形，正推 c3）	crates/singlefs-checker/src/walk.rs	            first_container_in_leaf_order == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	            first_container_in_leaf_order <= FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	-p singlefs-checker --lib -- product_retirement_containers_numbered_zero_and_one	product_retirement_containers_numbered_zero_and_one_violate_only_the_first_number_sentence
第57批 H3 I-9.37：第一片容器号判成 ≤ 1（退役树旋钮支坏镜像，正推 c3）	crates/singlefs-checker/src/walk.rs	            first_container_in_leaf_order == FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	            first_container_in_leaf_order <= FIRST_CONTAINER_NUMBER_OF_A_CONTAINER_TREE,	-p singlefs-harness --test checker_known_bad_images -- retirement_containers_numbered_zero_and_one	retirement_containers_numbered_zero_and_one_redden_only_the_knob_capacity_invariant
第57批 H1 I-9.37：旋钮支不判不是最右的那片装满旋钮值（退役树单测那一形）	crates/singlefs-checker/src/walk.rs	                    record_count == records_the_knob_allows;	                    record_count <= records_the_knob_allows;	-p singlefs-checker --lib -- two_retirement_containers_of_one_record_judged_with_the_knob_two	two_retirement_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence
第57批 H1 I-9.37：旋钮支不判不是最右的那片装满旋钮值（退役树旋钮 1 写出、旋钮 2 / 3 判的坏镜像，攻方 k2_knob_bad_images）	crates/singlefs-checker/src/walk.rs	                    record_count == records_the_knob_allows;	                    record_count <= records_the_knob_allows;	-p singlefs-harness --test checker_known_bad_images -- two_retirement_containers_written_under_the_knob_one	two_retirement_containers_written_under_the_knob_one_redden_only_the_knob_capacity_invariant_when_judged_with_the_knobs_two_and_three
第57批 H1 GPU 判器：旋钮那一支不判不是最右的那片装满旋钮值（退役树那一形）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (d_container_record_count(container_index) != knob_records) {	        if (false && d_container_record_count(container_index) != knob_records) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_two_retirement_containers_written_under_the_knob_one	the_gpu_judge_given_the_retirement_knob_two_agrees_with_the_cpu_on_two_retirement_containers_written_under_the_knob_one
第57批 H1 写侧早关一片（攻方 w-close-one-early：旋钮 K ≥ 2 装 K − 1 条就开新片）	crates/singlefs-core/src/container_tree.rs	            .is_some_and(|open| open.records.len() < records_per_container);	            .is_some_and(|open| open.records.len() < records_per_container.saturating_sub(1).max(1));	-p singlefs-harness --release --test container_trees_grow_a_level_when_the_root_is_full -- --include-ignored a_random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step	a_random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step
第57批 H2 GPU 判器 I-9.21：产品支墓碑那一个调用点不判容器号与记录数的三句	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    d_judge_container_numbers_from_one_by_one_and_no_empty_container(INVARIANT_I_9_21);	    // 变异：产品支墓碑那一个调用点不判三句	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_product_tombstone_containers_after_a_full_one_skip_a_number	the_gpu_judge_agrees_with_the_cpu_when_product_tombstone_containers_after_a_full_one_skip_a_number
`````

## m3-batch57/cold-walk-order

### crates.patch

`````diff
diff --git a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
--- a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
+++ b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
@@ -10,7 +10,7 @@
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
     TreeIdentifier,
 };
-use singlefs_core::allocator::PoolAllocator;
+use singlefs_core::allocator::{AllocationRecord, PoolAllocator};
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::bytes::{ByteReader, ByteWriter};
 use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
@@ -31,7 +31,8 @@
 };
 use singlefs_core::records::{
     mapping_key_for_node, parse_mapping_entry, AccountingEntry, TreeTableEntry,
-    STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING, TREE_KIND_INODE, TREE_KIND_RETIREMENT,
+    STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_INODE,
+    TREE_KIND_RETIREMENT,
 };
 use singlefs_core::recovery::{
     choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, JournalPolicy,
@@ -54,8 +55,9 @@
     UNIT_CLASS_INDEX_NODE,
 };
 use singlefs_format::{
-    ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES,
-    JOURNAL_RING_DEFAULT_BYTES, NODE_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES, UNIT_AREA_START_SLOT,
+    ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, ALLOCATION_RECORD_KEY_BYTES, DATA_UNIT_BYTES,
+    JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, NODE_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
+    UNIT_AREA_START_SLOT,
 };
 use singlefs_harness::memory_pool::{SparseBlockDevice, SparseDevice};
 
@@ -278,6 +280,34 @@
     );
 }
 
+/// 中央映射树根（单个叶根，这里改的几版都是）里码 2 节点 `node_pointer` 那一条映射条目的两条位置条目，整单元校验和换成 `node_checksum`。
+/// 改完重封映射树根自己的两道校验和（下一次在同一个根上改另一条时，解它要先过这两道）。
+fn repoint_the_mapping_entry_of_an_index_node(
+    mapping_root: &mut [u8],
+    node_pointer: NodePointer,
+    node_checksum: u32,
+) {
+    let mapping_key = mapping_key_for_node(UNIT_CLASS_INDEX_NODE, node_pointer);
+    let mapping_header = parse_index_node(mapping_root).expect("映射树根解得开");
+    assert_eq!(
+        mapping_header.level, 0,
+        "第一个文件那一版与之后建空 inode 那一版的映射树都是单个叶根"
+    );
+    let mapping_entry_index = mapping_header
+        .entries
+        .iter()
+        .position(|entry| parse_mapping_entry(entry).is_some_and(|(key, _)| key == mapping_key))
+        .expect("映射里有这个节点");
+    let mapping_entry_range = entry_range_in_node(mapping_root, mapping_entry_index);
+    for location_index in 0..2 {
+        // 映射条目：key 27 + 位置条目 14 × 2，每条的校验和住条目偏移 4 + 6。
+        let checksum_offset = mapping_entry_range.start + 27 + location_index * 14 + 10;
+        mapping_root[checksum_offset..checksum_offset + 4]
+            .copy_from_slice(&node_checksum.to_le_bytes());
+    }
+    reseal_index_node(mapping_root);
+}
+
 /// 把最新那条根的树表里种类为 `kind` 的那棵树的根节点改一下（`change` 改字节，之后这里重封它），再把指着它的每一道校验和都重算：
 /// 树表条目里那条根指针、中央映射里它那一条（单个叶根的映射树）、树表单元、映射树根、根记录。读者沿根记录读下来，走到被改的节点才判得到。
 fn rewrite_the_root_node_of_a_tree_of_the_newest_root(
@@ -324,25 +354,7 @@
         root.mapping_root.locations[0].slot.to_device_offset(),
         node_bytes(),
     );
-    let mapping_key = mapping_key_for_node(UNIT_CLASS_INDEX_NODE, child);
-    let mapping_header = parse_index_node(&mapping_root).expect("映射树根解得开");
-    assert_eq!(
-        mapping_header.level, 0,
-        "第一个文件那一版的映射树是单个叶根"
-    );
-    let mapping_entry_index = mapping_header
-        .entries
-        .iter()
-        .position(|entry| parse_mapping_entry(entry).is_some_and(|(key, _)| key == mapping_key))
-        .expect("映射里有这个节点");
-    let mapping_entry_range = entry_range_in_node(&mapping_root, mapping_entry_index);
-    for location_index in 0..2 {
-        // 映射条目：key 27 + 位置条目 14 × 2，每条的校验和住条目偏移 4 + 6。
-        let checksum_offset = mapping_entry_range.start + 27 + location_index * 14 + 10;
-        mapping_root[checksum_offset..checksum_offset + 4]
-            .copy_from_slice(&node_checksum.to_le_bytes());
-    }
-    reseal_index_node(&mut mapping_root);
+    repoint_the_mapping_entry_of_an_index_node(&mut mapping_root, child, node_checksum);
     let mapping_root_checksum =
         write_unit_at_both_locations(devices, &root.mapping_root.locations, &mapping_root);
 
@@ -1675,6 +1687,251 @@
     );
 }
 
+// ─── 分配记录逐条判代与跨度先于数记账条目与数映射条目 ───
+
+/// 台阶行的统计量标签换成的值：`singlefs_core::records` 登记的统计量标签到 inode 号水位 12 为止，这是之后的一个、没有登记。
+/// 逐条判记账条目只判代与 seq，这个标签照收；数记账条目只把「统计量是 inode 号水位且代早于根」的行当台阶行不算，
+/// 换了标签的那一行被算进 3 + 6 × 盘数。
+const STATISTIC_TAG_AFTER_THE_LAST_REGISTERED_ONE: u16 = STATISTIC_INODE_WATERMARK + 1;
+/// 分配记录的代改成的值：晚于台阶行那个池最新那条根的 txg 4。
+const ALLOCATION_RECORD_GENERATION_AFTER_THE_ROOT: CheckpointTxg = CheckpointTxg(5);
+
+/// 分配记录树里 `pointer` 指着的节点与它下面整棵子树：叶里满足 `is_the_record_to_change` 的记录交给 `change` 改（只改不进 key 的字段），
+/// 改过的节点重封、写回两份落点；内部节点里指着改过的孩子的那条内部条目（key 10 + 子指针 86），子指针两条位置条目的整单元校验和
+/// 换成孩子的新值。改过的节点连同它的新整单元校验和推进 `rewritten_nodes`，改过的记录数加进 `changed_records`；
+/// 交回这个节点改过之后的整单元校验和，没改过交回 `None`。
+fn rewrite_allocation_records_in_the_subtree(
+    devices: &mut Devices,
+    pointer: &NodePointer,
+    is_the_record_to_change: &dyn Fn(&AllocationRecord) -> bool,
+    change: &dyn Fn(&mut AllocationRecord),
+    rewritten_nodes: &mut Vec<(NodePointer, u32)>,
+    changed_records: &mut usize,
+) -> Option<u32> {
+    let mut node = read_bytes(
+        devices,
+        pointer.locations[0].device,
+        pointer.locations[0].slot.to_device_offset(),
+        node_bytes(),
+    );
+    let header = parse_index_node(&node).expect("分配记录树节点解得开");
+    let key_width = usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10");
+    let mut is_this_node_changed = false;
+    for entry_index in 0..header.entries.len() {
+        let entry_range = entry_range_in_node(&node, entry_index);
+        if header.level == 0 {
+            let mut record = AllocationRecord::parse(&node[entry_range.clone()])
+                .expect("分配记录叶的条目宽是 20");
+            if is_the_record_to_change(&record) {
+                change(&mut record);
+                node[entry_range].copy_from_slice(&record.to_bytes());
+                *changed_records += 1;
+                is_this_node_changed = true;
+            }
+            continue;
+        }
+        let child =
+            NodePointer::read_from(&mut ByteReader::at(&node[entry_range.clone()], key_width));
+        if let Some(child_checksum) = rewrite_allocation_records_in_the_subtree(
+            devices,
+            &child,
+            is_the_record_to_change,
+            change,
+            rewritten_nodes,
+            changed_records,
+        ) {
+            repoint_the_checksums_of_a_pointer(&mut node[entry_range], key_width, child_checksum);
+            is_this_node_changed = true;
+        }
+    }
+    if !is_this_node_changed {
+        return None;
+    }
+    reseal_index_node(&mut node);
+    let node_checksum = write_unit_at_both_locations(devices, &pointer.locations, &node);
+    rewritten_nodes.push((*pointer, node_checksum));
+    Some(node_checksum)
+}
+
+/// 把最新那条根的分配记录树里满足 `is_the_record_to_change` 的记录交给 `change` 改，再把指着改过的节点的每一道校验和都重算：
+/// 父节点内部条目里的子指针（逐层到根）、树表条目里那条根指针、树表单元、中央映射树根里每个改过的节点那一条、
+/// 根记录里树表、中央映射树根两条指针（带文件的那一版根记录里分配记录树根那一条是空的）。交回改了几条记录。
+fn rewrite_allocation_records_of_the_newest_root(
+    devices: &mut Devices,
+    is_the_record_to_change: impl Fn(&AllocationRecord) -> bool,
+    change: impl Fn(&mut AllocationRecord),
+) -> usize {
+    let (ring_slot, mut root) = newest_root_in_the_ring(devices);
+    let mut tree_table = read_bytes(
+        devices,
+        root.tree_table.locations[0].device,
+        root.tree_table.locations[0].slot.to_device_offset(),
+        node_bytes(),
+    );
+    let tree_table_header = parse_index_node(&tree_table).expect("树表单元解得开");
+    let entry_index = tree_table_header
+        .entries
+        .iter()
+        .position(|entry| {
+            TreeTableEntry::parse(entry).is_some_and(|parsed| parsed.kind == TREE_KIND_ALLOCATION)
+        })
+        .expect("树表里有分配记录树");
+    let allocation_tree_root_pointer =
+        TreeTableEntry::parse(&tree_table_header.entries[entry_index])
+            .expect("树表条目解得开")
+            .root;
+    assert_eq!(
+        root.allocation_record_tree_root,
+        NodePointer::empty_root(),
+        "带文件的那一版分配记录树根只在树表条目里，根记录里那一条是空的"
+    );
+    let mut rewritten_nodes: Vec<(NodePointer, u32)> = Vec::new();
+    let mut changed_records = 0;
+    let root_node_checksum = rewrite_allocation_records_in_the_subtree(
+        devices,
+        &allocation_tree_root_pointer,
+        &is_the_record_to_change,
+        &change,
+        &mut rewritten_nodes,
+        &mut changed_records,
+    )
+    .expect("至少改了一条记录：它所在的叶到根这一路都改过");
+
+    let entry_range = entry_range_in_node(&tree_table, entry_index);
+    // 树表条目里根指针从条目偏移 14 起（树 ID 8 + 条目长度 2 + 种类 2 + flags 2）。
+    repoint_the_checksums_of_a_pointer(&mut tree_table[entry_range], 14, root_node_checksum);
+    reseal_index_node(&mut tree_table);
+    let tree_table_checksum =
+        write_unit_at_both_locations(devices, &root.tree_table.locations, &tree_table);
+
+    let mut mapping_root = read_bytes(
+        devices,
+        root.mapping_root.locations[0].device,
+        root.mapping_root.locations[0].slot.to_device_offset(),
+        node_bytes(),
+    );
+    for (node_pointer, node_checksum) in &rewritten_nodes {
+        repoint_the_mapping_entry_of_an_index_node(
+            &mut mapping_root,
+            *node_pointer,
+            *node_checksum,
+        );
+    }
+    let mapping_root_checksum =
+        write_unit_at_both_locations(devices, &root.mapping_root.locations, &mapping_root);
+
+    for location in &mut root.tree_table.locations {
+        location.unit_checksum = tree_table_checksum;
+    }
+    for location in &mut root.mapping_root.locations {
+        location.unit_checksum = mapping_root_checksum;
+    }
+    write_root_into_the_ring_slot(devices, ring_slot, &root);
+    changed_records
+}
+
+/// 数记账条目那一个成员。
+fn accounting_entry_count_is_not_three_plus_six_per_device() -> RecoveryFailure {
+    RecoveryFailure::InvariantViolated {
+        invariant: "E142 走读同款",
+        detail: "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数",
+    }
+}
+
+/// 逐条判分配记录那一个成员：跨度为 0 或代晚于根。
+fn allocation_record_span_zero_or_generation_after_the_root() -> RecoveryFailure {
+    RecoveryFailure::InvariantViolated {
+        invariant: "E142 走读同款",
+        detail: "分配记录跨度为 0，或分配代 / 释放代晚于根",
+    }
+}
+
+/// 台阶行那个池（[`pool_whose_newest_accounting_root_carries_one_inode_number_watermark_step_row`]，txg 4）上两处一起坏，
+/// 每一处都把指着它的校验和重算：
+/// ① 记账树根里那一级台阶行（inode 号水位、代段 3）的统计量标签换成 [`STATISTIC_TAG_AFTER_THE_LAST_REGISTERED_ONE`]，条目按 key 重排：
+///    代 3、seq 照旧，逐条判照过；数记账条目时它不再算台阶行，16 ≠ 3 + 6 × 2。只坏这一处时只读恢复报记账条目数，这里先核一遍；
+/// ② 分配记录树里罩这一版树表单元那个槽的两条记录（两盘同槽各一条，代 4）代改成 5，晚于根；两盘一起改，
+///    「每个落点每盘一条」那一判（比的是各盘的（槽, 跨度, 代, 已释放）集合）照过。
+fn pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_entry_count_is_off(
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
+                        && entry.generation < TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW
+                },
+                |step_row| step_row.statistic = STATISTIC_TAG_AFTER_THE_LAST_REGISTERED_ONE,
+            );
+        },
+    );
+    assert_eq!(
+        recover(&devices, JournalPolicy::Consult).outcome,
+        RecoveryOutcome::Failed {
+            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
+            failure: accounting_entry_count_is_not_three_plus_six_per_device(),
+        },
+        "前提：只坏记账那一处时，只读恢复在 txg 4 那条根上报记账条目数"
+    );
+    let (_, root) = newest_root_in_the_ring(&devices);
+    let tree_table_slot = root.tree_table.locations[0].slot;
+    let changed_records = rewrite_allocation_records_of_the_newest_root(
+        &mut devices,
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
+    devices
+}
+
+/// 场景：第一个文件之后再建一个空 inode（txg 4）；记账树根里那一级台阶行的统计量标签换成没登记的号（记账条目数因此对不上），
+/// 罩这一版树表单元那个槽的两条分配记录代改成 5，晚于根；指着改过的节点的每一道校验和重算。
+/// 预期：只读恢复（冷走读）在 txg 4 那条根上报「分配记录跨度为 0，或分配代 / 释放代晚于根」：逐条判分配记录的代与跨度排在数记账条目、
+/// 数映射条目之前（先判得了自己那一条的，再数；判决 `research/prompts/m3-batch56-code-r1-main-verification.md` 第三节 H4）。
+/// 那一判挪到两个数之后，这份镜像报成记账条目数。
+#[test]
+fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_recovery(
+) {
+    let devices =
+        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_entry_count_is_off();
+    let report = recover(&devices, JournalPolicy::Consult);
+    assert_eq!(
+        report.outcome,
+        RecoveryOutcome::Failed {
+            root: Some((InstanceGeneration(1), TXG_OF_THE_ROOT_CARRYING_THE_STEP_ROW)),
+            failure: allocation_record_span_zero_or_generation_after_the_root(),
+        },
+        "只读恢复在 txg 4 那条根上报分配记录的代晚于根，不报记账条目数"
+    );
+}
+
+/// 场景：同上一条的坏镜像，交给可写挂载。
+/// 预期：可写挂载在任何写之前交回「分配记录跨度为 0，或分配代 / 释放代晚于根」：重建那一版在读记账树之前就逐条判分配记录的代与跨度
+/// （与冷走读同一个次序），那一判挪到数记账条目之后，这份镜像报成记账条目数。
+#[test]
+fn allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_writable_mount(
+) {
+    let mut devices =
+        pool_whose_allocation_records_carry_a_generation_after_the_root_and_whose_accounting_entry_count_is_off();
+    let refusal = writable_mount_refused_before_any_write(&mut devices);
+    assert!(
+        matches!(&refusal, MountError::Recovery(failure) if *failure == allocation_record_span_zero_or_generation_after_the_root()),
+        "可写挂载报分配记录的代晚于根，不报记账条目数：{refusal:?}"
+    );
+}
+
 // ─── 第 35 条续：单元区槽数那一处减法 ───
 
 /// 场景：700 MiB 的盘（单元区起点在 784 MiB）。预期：单元区槽数交回「盘末尾在单元区起点之前」，带盘的字节数与起点槽号；
`````

### mutations-append.tsv

`````text
第57批 H4 冷走读分配记录代与跨度挪回数记账条目、数映射条目之后（代晚于根的分配记录在记账条目数对不上的镜像上报成记账条目数）	crates/singlefs-core/src/recovery.rs	    allocation_record_generations_and_spans_are_judged(allocation_records, root)?;\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &roots.accounting.leaf_entries_in_key_order,\n    )?;\n    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {\n        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &roots.accounting.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_count,\n    )?;\n    mapping_entry_count_is_one_per_mapped_unit(\n        roots.mapping.leaf_entries_in_key_order.len(),\n        MappedUnitCounts {\n            extent_tree_nodes: roots.extent.nodes_in_bump_order.len(),\n            allocation_record_tree_nodes: roots.allocation.nodes.len(),\n            accounting_tree_nodes: roots.accounting.version.node_count(),\n            inode_leaf_containers: roots.inode.entries.len(),\n            data_units: roots.extent.data_pointers.len(),\n            deletion_tree_nodes,\n        },\n    )?;	    every_accounting_entry_is_as_wide_as_its_field_table(\n        &roots.accounting.leaf_entries_in_key_order,\n    )?;\n    for entry_bytes in &roots.accounting.leaf_entries_in_key_order {\n        accounting_entry_parsed_and_judged_against_the_root(entry_bytes, root)?;\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &roots.accounting.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_count,\n    )?;\n    mapping_entry_count_is_one_per_mapped_unit(\n        roots.mapping.leaf_entries_in_key_order.len(),\n        MappedUnitCounts {\n            extent_tree_nodes: roots.extent.nodes_in_bump_order.len(),\n            allocation_record_tree_nodes: roots.allocation.nodes.len(),\n            accounting_tree_nodes: roots.accounting.version.node_count(),\n            inode_leaf_containers: roots.inode.entries.len(),\n            data_units: roots.extent.data_pointers.len(),\n            deletion_tree_nodes,\n        },\n    )?;\n    allocation_record_generations_and_spans_are_judged(allocation_records, root)?;	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- allocation_records_whose_generation_is_after_the_root	allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_recovery
第57批 H4 重建分配记录代与跨度挪到数记账条目之后（代晚于根的分配记录在记账条目数对不上的镜像上报成记账条目数）	crates/singlefs-core/src/recovery.rs	    allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;\n    let allocation_records: Vec<AllocationRecord> = allocation_tree_read.records.clone();\n    // 记账树可以是多层（D8（核心索引结构） 已定项 11）：从根往下整棵读回来，节点进映射、提示读不出经这一版的中央映射回退\n    // （与读别的树根同一条）；中央映射树同样整棵读回来（下面）。两棵树照冷走读判全（`code_two_tree::read_code_two_tree` 的\n    // `EveryHeaderAgainstItsReference`：节点头、层级、区间与 key 严格递增——有重复 key 的记账叶交出去，挂载之后第一次发布规划那棵树时\n    // 在 `transaction` 的断言上 panic，代码审阅第 33 条）。\n    let accounting_tree = read_code_two_tree(\n        &accounting_pointer,\n        &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),\n        CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,\n        root,\n        expected_filesystem_identifier,\n        &mut |pointer: &NodePointer| {\n            read_mapped_tree_node_via_hint_then_central_mapping(\n                reader,\n                pointer,\n                MappedTreeNodeClass::IndexNode,\n                &central_mapping_locations_of_key,\n                &mut stale_location_hint_hops_not_exposed_by_this_reader,\n            )\n        },\n    )?;\n    // 次序：条目宽、逐条判代与 seq、数条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`\n    // 第三节 G1）。数条目要按统计量与代段认台阶行，代写到根之后的台阶行认不出、被算进 3 + 6 × 盘数，「代晚于根」这一坏法就报成了条目数不对。\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &accounting_tree.leaf_entries_in_key_order,\n    )?;\n    let mut accounting_entries: Vec<AccountingEntry> =\n        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());\n    for bytes in &accounting_tree.leaf_entries_in_key_order {\n        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(\n            bytes, root,\n        )?);\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_identities.len(),\n    )?;	    let allocation_records: Vec<AllocationRecord> = allocation_tree_read.records.clone();\n    // 记账树可以是多层（D8（核心索引结构） 已定项 11）：从根往下整棵读回来，节点进映射、提示读不出经这一版的中央映射回退\n    // （与读别的树根同一条）；中央映射树同样整棵读回来（下面）。两棵树照冷走读判全（`code_two_tree::read_code_two_tree` 的\n    // `EveryHeaderAgainstItsReference`：节点头、层级、区间与 key 严格递增——有重复 key 的记账叶交出去，挂载之后第一次发布规划那棵树时\n    // 在 `transaction` 的断言上 panic，代码审阅第 33 条）。\n    let accounting_tree = read_code_two_tree(\n        &accounting_pointer,\n        &MultiLevelCodeTwoTree::Accounting.read_expectation(tree_identifiers.accounting),\n        CodeTwoTreeHeaderJudgement::EveryHeaderAgainstItsReference,\n        root,\n        expected_filesystem_identifier,\n        &mut |pointer: &NodePointer| {\n            read_mapped_tree_node_via_hint_then_central_mapping(\n                reader,\n                pointer,\n                MappedTreeNodeClass::IndexNode,\n                &central_mapping_locations_of_key,\n                &mut stale_location_hint_hops_not_exposed_by_this_reader,\n            )\n        },\n    )?;\n    // 次序：条目宽、逐条判代与 seq、数条目——先判得了自己那一条的，再数（判决 `research/prompts/m3-batch52-54-code-r1-main-verification.md`\n    // 第三节 G1）。数条目要按统计量与代段认台阶行，代写到根之后的台阶行认不出、被算进 3 + 6 × 盘数，「代晚于根」这一坏法就报成了条目数不对。\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &accounting_tree.leaf_entries_in_key_order,\n    )?;\n    let mut accounting_entries: Vec<AccountingEntry> =\n        Vec::with_capacity(accounting_tree.leaf_entries_in_key_order.len());\n    for bytes in &accounting_tree.leaf_entries_in_key_order {\n        accounting_entries.push(accounting_entry_parsed_and_judged_against_the_root(\n            bytes, root,\n        )?);\n    }\n    accounting_entry_count_is_three_plus_six_per_device(\n        &accounting_tree.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_identities.len(),\n    )?;\n    allocation_record_generations_and_spans_are_judged(&allocation_tree_read.records, root)?;	-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking -- allocation_records_whose_generation_is_after_the_root	allocation_records_whose_generation_is_after_the_root_are_refused_before_the_accounting_entry_count_by_the_writable_mount
`````

