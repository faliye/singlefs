# 删除批第 27 批合入的改动原样（2026-10-06）

## 实现员 j：crates.patch（/tmp/claude-1000/m3-batch27/j/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index f3be249..16e903b 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -1576,6 +1576,14 @@ fn w_judge_packed_container(s: Slice, expected_type: u32, fsid_low: vec2<u32>) -
 // `research/prompts/m3-batch23-code-r1-main-verification.md` 第三节第 1 条：门与候选集的滤法用同一遍走读，不另读一遍）。择根之后、走最新根之前清掉
 var<private> w_newest_instance_table_chain_reached_its_last_page: bool;
 
+// 实例表链走读循环最多走几片（walk.rs 的链不设片数上限，走过的单元不再走）
+const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 256u;
+
+// 走满 INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED 片、最后那一片的链指针记录还说有下一片：这一张表 GPU 走不完，记判不了交 CPU
+// （判决 `research/prompts/m3-batch25-code-r1-main-verification.md` 第二节 H-c 那一行与第三节第 1 条：不当「没走到末片」，
+// 否则门关、I-9.29 那张表不比，CPU 走到末片照比，两边分叉）。「条目太多」一类，站点号与别处不撞
+const UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (61u << 8u);
+
 // walk.rs walk_instance_table_chain（挂载根那一张另判行：I-3.8 的行那一半，行记进 C_INSTANCE_ROWS；走到末片记进
 // w_newest_instance_table_chain_reached_its_last_page）
 fn w_walk_instance_table_chain(first: Ptr, judge_rows: bool, fsid_low: vec2<u32>) {
@@ -1588,7 +1596,9 @@ fn w_walk_instance_table_chain(first: Ptr, judge_rows: bool, fsid_low: vec2<u32>
     var rows_below = true;
     var chain_last = true;
     var pages = 0u;
-    for (var page = 0u; page < 256u; page = page + 1u) {
+    // 循环里每个提前出口都在 page 加一之前 break：走满上限才落到 page == INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED
+    var page = 0u;
+    for (; page < INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED; page = page + 1u) {
         w_judge_location_order(pointer);
         w_judge_pointer_head(pointer);
         w_judge_pointer_extent_offset(pointer);
@@ -1673,6 +1683,9 @@ fn w_walk_instance_table_chain(first: Ptr, judge_rows: bool, fsid_low: vec2<u32>
         pointer = parse_node_pointer(p.base, packed_record_offset(p, p.record_count - 1u) + 2u);
         page_index = add64(page_index, u64_of(1u));
     }
+    if (page == INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED) {
+        undecidable = UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED;
+    }
     w_parent_valid = parent_valid;
     w_parent_device = parent_device;
     w_parent_slot = parent_slot;
@@ -2627,15 +2640,25 @@ fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: v
             w_judge(INVARIANT_I_9_2, class_of_the_inode_child == class_the_inode_entry_type_names);
             continue;
         }
+        // 类型段不认得、上面也没认出孩子自证过的类：不按哪一种长度去读孩子（walk.rs walk_inode_internal_node 的 Unregistered 那一支，
+        // 与墓碑树、谱系树类型段不认得那一支同判；第 27 批）——孩子按这一层定的形状登记引用（层级 1 下面是叶容器两槽，层级 ≥ 2 下面是码 2 节点一槽），
+        // 判 I-9.2 红，不记 I-2.1、不记走读失败
+        if (class_the_inode_entry_type_names == NONE) {
+            var child_span_by_the_level = 2u;
+            if (root.level >= 2u) {
+                child_span_by_the_level = 1u;
+            }
+            w_note_pointer_references(child, child_span_by_the_level);
+            w_judge(INVARIANT_I_9_2, false);
+            continue;
+        }
         w_note_pointer_references(child, span);
         let s = w_read_referenced_unit(child, length);
         if (s.version == NONE) {
             // 孩子两份都读不出：I-9.2 里不看孩子就判得了的那几小句照判（walk.rs judge_an_inode_entry_whose_child_does_not_read_back，
-            // 与墓碑树、谱系树读不出孩子时同判）——类型段 0 判身份引用三段为 0，类型段不认得的判该条目损坏，类型段 2 不判
+            // 与墓碑树、谱系树读不出孩子时同判）——类型段 0 判身份引用三段为 0，类型段 2 不判（类型段不认得的上面已经判完）
             if (record_type == 0u) {
                 w_judge(INVARIANT_I_9_2, is_zero64(identity_birth_tree) && is_zero64(identity_container) && is_zero64(identity_birth));
-            } else if (record_type != 2u) {
-                w_judge(INVARIANT_I_9_2, false);
             }
             w_failures = w_failures + 1u;
             continue;
@@ -2748,12 +2771,10 @@ fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: v
                     w_separator_count = w_separator_count + 1u;
                 }
             }
-        } else if (record_type == 0u) {
-            // 类型段 0 的内部节点：这一版没有这种形状（inode_tree_root_level_is_one），GPU 判不了
+        } else {
+            // 类型段 0 的内部节点（类型段不认得的上面已经判完）：这一版没有这种形状（inode_tree_root_level_is_one），GPU 判不了
             undecidable = UNDECIDABLE_TOO_DEEP;
             return;
-        } else {
-            w_judge(INVARIANT_I_9_2, false);
         }
     }
     w_parent_valid = parent_valid;
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 3ab51d2..a4ed9bb 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -25,6 +25,9 @@
 //! 第 25 批：同一段历史、最新根的实例表链指针记录之前多一条 kind 2（攻方 N5-a，主走读走到末片、I-9.29 照比）一条流整份比。
 //! 第 26 批：inode 树根条目类型段 0、身份三段清零，叶容器盘 0 那一份换成码 2 节点的陈旧字节（攻方 H1：头里的类恰是类型段那一类的副本也要自证）
 //! 一条流钉 CPU 红 I-9.2、不红 I-7.2，GPU 逐位跟上。
+//! 第 27 批：inode 叶容器类标签改成不登记的码、不重封（孩子读不出），树根条目类型段 0 带身份、类型段 7 各一条流整份比；
+//! 同一段被崩溃恢复抛弃的历史上，最新根的实例表链指针记录不合法、第 1 片读不出（GPU 关门那一侧）各一条流整份比；
+//! 链 256 片一条整份比，链 257 片一条钉 GPU 池级 checker 记判不了（实例表链过长）、回落 CPU 之后结论相同（攻方 H-c）。
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
 //! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）；这几处除实例表指针外
 //! 另各有一条 MAC 第 0 字节非 0 的坏流（core 连加密、压缩那几段一并判，GPU 恢复内核同判）；extent 上段叶内联的数据指针另有一条算法类型写 1 的坏流
@@ -64,8 +67,9 @@ use singlefs_checker_tier::crash::{
     Layer0SegmentExpansion, Layer0StateJudgement,
 };
 use singlefs_checker_tier::crash_amplification::{
-    gpu_recovery_words, judge_my_blocks_on_gpu_cards, plan_crash_points, record_crash_points,
-    recovery_words_of, unjudged_blocks, CrashFlow, CrashPointRecorder, CrashPointSpan,
+    gpu_recovery_words, gpu_verdict_items, judge_my_blocks_on_gpu_cards, plan_crash_points,
+    record_crash_points, recovery_words_of, unjudged_blocks, CrashFlow, CrashPointRecorder,
+    CrashPointSpan,
 };
 use singlefs_checker_tier::crash_identity::CoverageReport;
 use singlefs_checker_tier::crash_judge_dispatch::{
@@ -555,6 +559,31 @@ fn corrupt_units_and_reseal_up_to_the_root(
     flow: &mut RecordedFlow,
     targets: &[usize],
     mutate: &dyn Fn(&mut [u8]),
+) {
+    corrupt_units_and_carry_their_whole_unit_checksums_up_to_the_root(
+        flow,
+        targets,
+        mutate,
+        OwnChecksumsOfTheChangedUnits::Resealed,
+    );
+}
+
+/// 被改的单元自己那两道校验和（载荷 CRC、头校验和）重不重封。
+#[derive(Clone, Copy, Debug)]
+enum OwnChecksumsOfTheChangedUnits {
+    /// 重封：被改的单元自证得过，坏法只剩改的那个字段。
+    Resealed,
+    /// 不重封：被改的单元按自己的类认不出、或自证不过（类标签改成不登记的码那一形），指着它的位置条目照样换成新的整单元 CRC。
+    LeftAsBeforeTheChange,
+}
+
+/// 同 [`corrupt_units_and_reseal_up_to_the_root`]，被改的单元自己那两道校验和照 `own_checksums` 重封或不动；
+/// 指着它的位置条目一律换成改后的整单元 CRC、沿引用链一路重封到根。
+fn corrupt_units_and_carry_their_whole_unit_checksums_up_to_the_root(
+    flow: &mut RecordedFlow,
+    targets: &[usize],
+    mutate: &dyn Fn(&mut [u8]),
+    own_checksums: OwnChecksumsOfTheChangedUnits,
 ) {
     let mut pending: Vec<(usize, u32)> = Vec::new();
     for target in targets {
@@ -565,7 +594,10 @@ fn corrupt_units_and_reseal_up_to_the_root(
         };
         let old_crc = crc32c(bytes);
         mutate(bytes);
-        reseal_unit(bytes);
+        match own_checksums {
+            OwnChecksumsOfTheChangedUnits::Resealed => reseal_unit(bytes),
+            OwnChecksumsOfTheChangedUnits::LeftAsBeforeTheChange => {}
+        }
         pending.push((*target, old_crc));
     }
     while let Some((index, old_crc)) = pending.pop() {
@@ -1893,6 +1925,10 @@ struct Comparison {
     cpu_checker_held_invariants: std::collections::BTreeSet<&'static str>,
     /// CPU 池级 checker 在每个状态上的违例位（按状态序号）：要钉「同一个状态上红这一条、不红那一条」的用例读它。
     cpu_checker_bits_of_each_state: Vec<u128>,
+    /// CPU 判器在每个状态上的红项（`Layer0StateJudgement::red_items` 的 `{:?}`，按状态序号）。
+    cpu_red_items_of_each_state: Vec<String>,
+    /// GPU 在每个状态上交回的结论字（按状态序号）：要按判不了的原因分开看、或照流水线那样判不了就回落 CPU 的用例读它。
+    gpu_words_of_each_state: Vec<[u32; VERDICT_WORDS]>,
     /// 看 journal 那一遍落到的根或两遍的结局类别与 CPU 不同的状态数（算进不一致）。
     recovery_mismatches: u64,
     mismatches: Vec<String>,
@@ -2021,6 +2057,8 @@ fn compare_on_every_state_of_these_segments(
         cpu_checker_red_invariants: std::collections::BTreeSet::new(),
         cpu_checker_held_invariants: std::collections::BTreeSet::new(),
         cpu_checker_bits_of_each_state: Vec::with_capacity(cpu.len()),
+        cpu_red_items_of_each_state: Vec::with_capacity(cpu.len()),
+        gpu_words_of_each_state: gpu.clone(),
         recovery_mismatches: 0,
         mismatches: Vec::new(),
         checker_mismatches: Vec::new(),
@@ -2036,6 +2074,9 @@ fn compare_on_every_state_of_these_segments(
         comparison
             .cpu_checker_bits_of_each_state
             .push(*cpu_checker_bits);
+        comparison
+            .cpu_red_items_of_each_state
+            .push(cpu_items.clone());
         let gpu_words: [u32; VERDICT_WORDS] = gpu[ordinal];
         let gpu_bits = gpu_words[0] & !(1 << verdict_bits::UNDECIDABLE);
         let undecidable = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
@@ -6394,6 +6435,48 @@ fn record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance
                 &insert_a_record_of_an_unregistered_kind_before_the_chain_record,
             );
         }
+        ChangeOfTheNewestInstanceTable::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance => {
+            let (rows_of_the_abandoned_instance, other_rows) =
+                rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
+                    &flow,
+                    &newest_instance_table,
+                );
+            assert_eq!(rows_of_the_abandoned_instance.len(), 1, "实例 2 那一行恰好一行");
+            let mut records = other_rows;
+            records.push(chain_record_saying_a_next_page_without_a_pointer());
+            corrupt_units_and_reseal_up_to_the_root(&mut flow, &newest_instance_table, &|bytes| {
+                replace_the_records_of_an_instance_table_page(bytes, &records);
+            });
+        }
+        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceUnreadable => {
+            let (rows_of_the_abandoned_instance, other_rows) =
+                rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
+                    &flow,
+                    &newest_instance_table,
+                );
+            rebuild_the_newest_instance_table_as_a_chain(
+                &mut flow,
+                &newest_instance_table,
+                &[other_rows, rows_of_the_abandoned_instance],
+                PagesAfterTheFirstInTheFlow::BothCopiesFlippedWithoutResealing,
+            );
+        }
+        ChangeOfTheNewestInstanceTable::SpreadOverPagesWithoutTheRowOfTheAbandonedInstance(
+            pages,
+        ) => {
+            let (_, other_rows) = rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
+                &flow,
+                &newest_instance_table,
+            );
+            let mut rows_of_each_page = vec![other_rows];
+            rows_of_each_page.resize(pages.count(), Vec::new());
+            rebuild_the_newest_instance_table_as_a_chain(
+                &mut flow,
+                &newest_instance_table,
+                &rows_of_each_page,
+                PagesAfterTheFirstInTheFlow::Intact,
+            );
+        }
     }
     (flow, image_with_the_newest_instance_table_readable)
 }
@@ -6408,6 +6491,18 @@ enum ChangeOfTheNewestInstanceTable {
     /// 链指针记录之前插一条没登记种类（kind 2）的记录、重封到根（攻方 N5-a，`research/prompts/m3-batch23-code-r1-opus-output.md`「一、N5 打中」）：
     /// 主走读照收 kind 0 的行、沿链走到末片，候选集滤掉了被抛弃的 C，CPU 与 GPU 都比、都不红 I-9.29。
     RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord,
+    /// 唯一那一片里去掉被抛弃的实例 2 那一行，链指针记录的「有无下一片」写 1、指针照旧全零（不合法的链指针记录）、重封到根
+    /// （判决 `research/prompts/m3-batch25-code-r1-main-verification.md` 第三节第 2 条，GPU 关门那一侧「链指针记录不合法」一形）：
+    /// 主走读没走到末片，门关，CPU 报 I-9.29 部分没判、不红，GPU 不置位、不比。门要是开了，候选集里没有实例 2 那一行、滤不掉 C，
+    /// 比就红——GPU 走读内核把「链指针记录不合法」也当走到末片时，这条流上 GPU 红 I-9.29 而 CPU 不红。
+    ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance,
+    /// 拆成两片：第 0 片放实例 2 之外的行、链指针记录指第 1 片；第 1 片放实例 2 那一行与末片的链指针记录，两份各翻一个载荷字节、
+    /// 不重封（第 1 片两份都读不出；同一判决第 2 条「单片读不出」一形）。主走读读到第 0 片、第 1 片读不出，没走到末片，门关：
+    /// CPU 报 I-9.29 部分没判、不红，GPU 不置位、不比（门要是开了，候选集里没有实例 2 那一行，比就红）。
+    SecondPageHoldingTheRowOfTheAbandonedInstanceUnreadable,
+    /// 拆成 `pages` 片：第 0 片放实例 2 之外的行，其余各片只有链指针记录（攻方 H-c 那一形，`research/prompts/m3-batch25-code-r1-opus-output.md`
+    /// 第 2.1 节）。主走读走到末片、门开，候选集里没有实例 2 那一行、滤不掉 C，C 与现行线同号不同出生代，CPU 红 I-9.29。
+    SpreadOverPagesWithoutTheRowOfTheAbandonedInstance(InstanceTableChainPages),
 }
 
 impl ChangeOfTheNewestInstanceTable {
@@ -6419,8 +6514,320 @@ impl ChangeOfTheNewestInstanceTable {
             Self::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord => {
                 "abandoned-root-naming-another-object-birth-newest-instance-table-with-an-unregistered-record-kind"
             }
+            Self::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance => {
+                "abandoned-root-naming-another-object-birth-newest-instance-table-with-an-invalid-chain-record"
+            }
+            Self::SecondPageHoldingTheRowOfTheAbandonedInstanceUnreadable => {
+                "abandoned-root-naming-another-object-birth-newest-instance-table-second-page-unreadable"
+            }
+            Self::SpreadOverPagesWithoutTheRowOfTheAbandonedInstance(
+                InstanceTableChainPages::AsManyAsTheGpuWalks,
+            ) => "abandoned-root-naming-another-object-birth-newest-instance-table-of-256-pages",
+            Self::SpreadOverPagesWithoutTheRowOfTheAbandonedInstance(
+                InstanceTableChainPages::OneMoreThanTheGpuWalks,
+            ) => "abandoned-root-naming-another-object-birth-newest-instance-table-of-257-pages",
+        }
+    }
+}
+
+/// GPU 走读内核沿实例表链最多走几片（`crash_judge_checker.wgsl` 的 `INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED`；CPU 不设上限）。
+const GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: usize = 256;
+
+/// GPU 走读内核走满 [`GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED`] 片、链还没完时交回的判不了原因字
+/// （`crash_judge_checker.wgsl` 的 `UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED`：「条目太多」一类 3、站点号 61）。
+const GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED: u32 = 3 | (61 << 8);
+
+/// 改造出来的实例表链有几片。
+#[derive(Clone, Copy, Debug)]
+enum InstanceTableChainPages {
+    /// 恰好是 GPU 走读内核走得完的片数：两边都走到末片、都比。
+    AsManyAsTheGpuWalks,
+    /// 比 GPU 走读内核走得完的多一片：CPU 走到末片、比；GPU 走满上限链还没完，记判不了交 CPU。
+    OneMoreThanTheGpuWalks,
+}
+
+impl InstanceTableChainPages {
+    const fn count(self) -> usize {
+        match self {
+            Self::AsManyAsTheGpuWalks => GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED,
+            Self::OneMoreThanTheGpuWalks => GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED + 1,
+        }
+    }
+}
+
+/// 被崩溃恢复抛弃的 C 那一版的实例代号（流里 C 由实例 2 发布，恢复之后的实例 3 在它的实例表里给实例 2 写一行）。
+const INSTANCE_OF_THE_ABANDONED_ROOT: u32 = 2;
+/// 实例表一行里实例代号（4 字节）的偏移；第 0 字节是种类。
+const INSTANCE_TABLE_ROW_INSTANCE_OFFSET: usize = 1;
+const INSTANCE_TABLE_RECORD_KIND_ROW: u8 = 0;
+/// 链指针记录里「有无下一片」那一字节的偏移，与之后 86 字节节点指针的偏移。
+const INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET: usize = 1;
+const INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET: usize = 2;
+/// 打包记录单元头里容器号（8 字节）的偏移：实例表那一片的容器号就是它在链上的片号。
+const PACKED_UNIT_CONTAINER_NUMBER_OFFSET: usize = 53;
+/// 节点指针里两条位置条目的起点（盘 4、槽 6、整单元校验和 4）。
+const POINTER_LOCATION_ENTRY_OFFSETS: [usize; 2] = [50, 64];
+/// 一个槽的字节数（实例表一片占两槽）。
+const SLOT_BYTES: u64 = 16384;
+/// 改造实例表链时第 1 片起写在哪：从这个槽起每片两槽（4 GiB 的盘上流里没写过、基镜像里全零的那一截，写之前逐片核）。
+const FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST: u64 = 200_000;
+
+/// 实例表一片里的记录，按记录序。
+fn records_of_an_instance_table_page(bytes: &[u8]) -> Vec<Vec<u8>> {
+    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
+    (0..record_count)
+        .map(|record_index| {
+            let start = PACKED_UNIT_RECORDS_START + record_index * INSTANCE_TABLE_RECORD_BYTES;
+            bytes[start..start + INSTANCE_TABLE_RECORD_BYTES].to_vec()
+        })
+        .collect()
+}
+
+/// 实例表一片的载荷换成 `records`：记录数与声明长度跟着改，载荷里记录之后的字节写 0（重封归调用方）。
+fn replace_the_records_of_an_instance_table_page(bytes: &mut [u8], records: &[Vec<u8>]) {
+    bytes[PACKED_UNIT_RECORDS_START..].fill(0);
+    for (record_index, record) in records.iter().enumerate() {
+        let start = PACKED_UNIT_RECORDS_START + record_index * INSTANCE_TABLE_RECORD_BYTES;
+        bytes[start..start + INSTANCE_TABLE_RECORD_BYTES].copy_from_slice(record);
+    }
+    bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2].copy_from_slice(
+        &u16::try_from(records.len())
+            .expect("一片装得下")
+            .to_le_bytes(),
+    );
+    bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
+        &u16::try_from(records.len() * INSTANCE_TABLE_RECORD_BYTES)
+            .expect("一片装得下")
+            .to_le_bytes(),
+    );
+}
+
+/// 最新那张实例表（单片）里的行分成两份：被抛弃的实例 2 那几行，与其余各行（都按原来的次序）。核过它是单片、最后一条是末片的链指针记录。
+fn rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
+    flow: &RecordedFlow,
+    newest_instance_table: &[usize],
+) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
+    let page = flow.writes[newest_instance_table[0]]
+        .bytes()
+        .expect("单元写带字节");
+    let mut records = records_of_an_instance_table_page(page);
+    let chain_record = records.pop().expect("实例表一片里至少有链指针记录");
+    assert_eq!(
+        chain_record,
+        chain_record_of_the_last_page(),
+        "最新那张实例表只有一片：链指针记录说没有下一片、指针全零"
+    );
+    assert!(
+        records
+            .iter()
+            .all(|record| record[0] == INSTANCE_TABLE_RECORD_KIND_ROW),
+        "链指针记录之前都是行"
+    );
+    records.into_iter().partition(|row| {
+        read_u32_at(row, INSTANCE_TABLE_ROW_INSTANCE_OFFSET) == INSTANCE_OF_THE_ABANDONED_ROOT
+    })
+}
+
+/// 末片的链指针记录：种类 1、没有下一片、指针全零。
+fn chain_record_of_the_last_page() -> Vec<u8> {
+    let mut record = vec![0u8; INSTANCE_TABLE_RECORD_BYTES];
+    record[0] = INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD;
+    record
+}
+
+/// 不合法的链指针记录：说有下一片，指针却全零（D18（块里携带什么信息） 已定项 11 的有无下一片与指针配不上）。
+fn chain_record_saying_a_next_page_without_a_pointer() -> Vec<u8> {
+    let mut record = chain_record_of_the_last_page();
+    record[INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET] = 1;
+    record
+}
+
+/// 指下一片的链指针记录：照 `pointer_to_the_first_page`（根记录里指第 0 片的那条实例表指针）的头，两条位置条目换成下一片的槽与整单元 CRC。
+fn chain_record_naming_the_next_page(
+    pointer_to_the_first_page: &[u8],
+    next_page_slot: u64,
+    next_page_whole_unit_checksum: u32,
+) -> Vec<u8> {
+    let mut pointer = pointer_to_the_first_page.to_vec();
+    for location in POINTER_LOCATION_ENTRY_OFFSETS {
+        pointer[location + 4..location + 10].copy_from_slice(&next_page_slot.to_le_bytes()[..6]);
+        pointer[location + 10..location + 14]
+            .copy_from_slice(&next_page_whole_unit_checksum.to_le_bytes());
+    }
+    let mut record = chain_record_of_the_last_page();
+    record[INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET] = 1;
+    record[INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET
+        ..INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET + NODE_POINTER_BYTES]
+        .copy_from_slice(&pointer);
+    record
+}
+
+/// 节点指针第 `location` 条位置条目里的盘与槽。
+fn device_and_slot_of_a_pointer_location(pointer: &[u8], location: usize) -> (u32, u64) {
+    let mut slot = [0u8; 8];
+    slot[..6].copy_from_slice(&pointer[location + 4..location + 10]);
+    (read_u32_at(pointer, location), u64::from_le_bytes(slot))
+}
+
+/// 改造出来的实例表链上第 1 片起各片怎么写进流。
+#[derive(Clone, Copy, Debug)]
+enum PagesAfterTheFirstInTheFlow {
+    /// 照造出来的字节写，两份都读得出。
+    Intact,
+    /// 两份各翻一个载荷字节、不重封（前一片链指针记录里的整单元 CRC 按翻之前算）：两份都读不出。
+    BothCopiesFlippedWithoutResealing,
+}
+
+/// 最新根（实例 3）指着的那张单片实例表（`first_page_copies` 是写它的两份副本）改造成一条 `rows_of_each_page.len()` 片的链：
+/// 第 k 片放 `rows_of_each_page[k]` 再补链指针记录（末片说没有下一片，其余指下一片）。第 0 片就地改、重封到根；第 1 片起照第 0 片的头造
+/// （容器号写片号），两份写在 [`FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST`] 起每片两槽、与第 0 片同一对盘上，照 `pages_after_the_first`
+/// 写进流；这些写另起一段、插在写第 0 片那一段之前（第 0 片落了的状态上它们都已落）。
+fn rebuild_the_newest_instance_table_as_a_chain(
+    flow: &mut RecordedFlow,
+    first_page_copies: &[usize],
+    rows_of_each_page: &[Vec<Vec<u8>>],
+    pages_after_the_first: PagesAfterTheFirstInTheFlow,
+) {
+    let first_page = flow.writes[first_page_copies[0]]
+        .bytes()
+        .expect("单元写带字节")
+        .to_vec();
+    let first_page_slot = flow.writes[first_page_copies[0]].offset.0 / SLOT_BYTES;
+    let pointer_to_the_first_page = flow
+        .writes
+        .iter()
+        .filter(|write| write.kind == StepKind::RootRecordFua)
+        .filter_map(RetainedWrite::bytes)
+        .map(|root| {
+            root[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
+                ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
+                .to_vec()
+        })
+        .find(|pointer| {
+            device_and_slot_of_a_pointer_location(pointer, POINTER_LOCATION_ENTRY_OFFSETS[0]).1
+                == first_page_slot
+        })
+        .expect("流里有根记录指着最新那张实例表");
+    let devices = POINTER_LOCATION_ENTRY_OFFSETS.map(|location| {
+        singlefs_core::address::DeviceIdentity(
+            device_and_slot_of_a_pointer_location(&pointer_to_the_first_page, location).0,
+        )
+    });
+    let mut pages_after_the_first_from_the_last: Vec<(u64, Vec<u8>)> = Vec::new();
+    let mut next_page: Option<(u64, u32)> = None;
+    for page_index in (1..rows_of_each_page.len()).rev() {
+        let mut records = rows_of_each_page[page_index].clone();
+        records.push(match next_page {
+            Some((slot, whole_unit_checksum)) => chain_record_naming_the_next_page(
+                &pointer_to_the_first_page,
+                slot,
+                whole_unit_checksum,
+            ),
+            None => chain_record_of_the_last_page(),
+        });
+        let mut page = first_page.clone();
+        replace_the_records_of_an_instance_table_page(&mut page, &records);
+        page[PACKED_UNIT_CONTAINER_NUMBER_OFFSET..PACKED_UNIT_CONTAINER_NUMBER_OFFSET + 8]
+            .copy_from_slice(
+                &u64::try_from(page_index)
+                    .expect("片号装得进 u64")
+                    .to_le_bytes(),
+            );
+        reseal_unit(&mut page);
+        let slot = FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST
+            + 2 * u64::try_from(page_index).expect("片号装得进 u64");
+        next_page = Some((slot, crc32c(&page)));
+        pages_after_the_first_from_the_last.push((slot, page));
+    }
+    let mut first_page_records = rows_of_each_page[0].clone();
+    first_page_records.push(match next_page {
+        Some((slot, whole_unit_checksum)) => {
+            chain_record_naming_the_next_page(&pointer_to_the_first_page, slot, whole_unit_checksum)
+        }
+        None => chain_record_of_the_last_page(),
+    });
+    corrupt_units_and_reseal_up_to_the_root(flow, first_page_copies, &|bytes| {
+        replace_the_records_of_an_instance_table_page(bytes, &first_page_records);
+    });
+    let mut page_writes: Vec<RetainedWrite> = Vec::new();
+    for (slot, page) in pages_after_the_first_from_the_last.into_iter().rev() {
+        let page_offset = slot * SLOT_BYTES;
+        let page_end = page_offset + u64::try_from(page.len()).expect("单元长装得进 u64");
+        for device in devices {
+            assert!(
+                flow.writes.iter().all(|write| write.device != device
+                    || write.offset.0 + write.length_in_bytes() <= page_offset
+                    || write.offset.0 >= page_end),
+                "第 1 片起写的槽 {slot} 在盘 {} 上流里没写过",
+                device.0
+            );
+            assert!(
+                singlefs_core::recovery::PoolReader::read(
+                    &flow.base,
+                    device,
+                    singlefs_core::address::DeviceOffsetInBytes(page_offset),
+                    page.len(),
+                )
+                .expect("基镜像里读得到这一截")
+                .iter()
+                .all(|byte| *byte == 0),
+                "第 1 片起写的槽 {slot} 在盘 {} 的基镜像里全零",
+                device.0
+            );
+            let mut written = page.clone();
+            match pages_after_the_first {
+                PagesAfterTheFirstInTheFlow::Intact => {}
+                PagesAfterTheFirstInTheFlow::BothCopiesFlippedWithoutResealing => {
+                    let flipped = written.len() - BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE;
+                    written[flipped] ^= 0xff;
+                }
+            }
+            page_writes.push(RetainedWrite {
+                device,
+                kind: StepKind::UnitWrite,
+                is_force_unit_access: false,
+                offset: singlefs_core::address::DeviceOffsetInBytes(page_offset),
+                contents: WrittenContents::Bytes(written),
+            });
+        }
+    }
+    let first_page_write = *first_page_copies.iter().min().expect("第 0 片至少一份");
+    let segment_of_the_first_page = flow
+        .segments
+        .iter()
+        .position(|segment| segment.contains(&first_page_write))
+        .expect("写第 0 片的那次写在某一段里");
+    let insertion = *flow.segments[segment_of_the_first_page]
+        .iter()
+        .min()
+        .expect("段里至少一次写");
+    assert!(
+        flow.segments[..segment_of_the_first_page]
+            .iter()
+            .flatten()
+            .all(|write_index| *write_index < insertion)
+            && flow.segments[segment_of_the_first_page..]
+                .iter()
+                .flatten()
+                .all(|write_index| *write_index >= insertion),
+        "段按写序排：插进来的写在写第 0 片那一段之前、之前各段之后"
+    );
+    let inserted = page_writes.len();
+    flow.writes.splice(insertion..insertion, page_writes);
+    for segment in &mut flow.segments {
+        for write_index in segment.iter_mut() {
+            if *write_index >= insertion {
+                *write_index += inserted;
+            }
         }
     }
+    flow.segments.insert(
+        segment_of_the_first_page,
+        (insertion..insertion + inserted).collect(),
+    );
+    if flow.judged_root_index >= insertion {
+        flow.judged_root_index += inserted;
+    }
 }
 
 /// 实例表一条记录的字节数（D18（块里携带什么信息） 已定项 11）；第 0 字节是种类：1 链指针记录，2 没登记。
@@ -6883,3 +7290,310 @@ fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_witho
         mismatch_listing(&comparison)
     );
 }
+
+/// 单元头里类标签的偏移。
+const UNIT_HEADER_CLASS_TAG_OFFSET: usize = 6;
+/// 不在登记表里的单元类标签（登记的只有 1、2、3）：孩子头里写成它之后，按哪一类的长度都认不出它是哪一类。
+const UNIT_CLASS_NOT_REGISTERED: u8 = 9;
+
+/// inode 树根（层级 1，条目指叶容器）：码 2、key 宽 8、条目宽 120、第一条条目的类型段是 2。
+fn is_the_inode_tree_root_over_leaf_containers(bytes: &[u8]) -> bool {
+    unit_class(bytes) == 2
+        && usize::from(index_node_key_width(bytes)) == INODE_TREE_KEY_BYTES
+        && index_node_entry_width(bytes) == INODE_INTERNAL_ENTRY_BYTES
+        && read_u16_at(
+            bytes,
+            index_node_entries_start(INODE_TREE_KEY_BYTES)
+                + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
+        ) == PACKED_RECORD_TYPE_INODE
+}
+
+/// inode 树读不出孩子的那一形（第 26 批 H 报告「交主 agent」第 1、2 条，`records/2026-10-01-删除与并发事务.md` 第 27 批；
+/// harness 坏镜像 `image_whose_inode_root_entry_names_a_child_that_does_not_read_back` 同一形，录成流）：新池新建文件的流上 inode 叶容器两份
+/// 头里的类标签 3 → 9、不重封（[`corrupt_units_and_carry_their_whole_unit_checksums_up_to_the_root`] 取
+/// [`OwnChecksumsOfTheChangedUnits::LeftAsBeforeTheChange`]：指着它的位置条目换成改后的整单元 CRC、一路重封到根），
+/// 再把 inode 树根第一条条目的类型段 2 → `entry_type`、重封到根（身份引用不动）。孩子头里的类标签不登记，按哪一类都认不出；
+/// 类型段 0 按 16 KiB 去读，校验和对不上。
+fn record_new_pool_with_the_inode_root_entry_of_another_type_over_a_leaf_container_of_an_unregistered_class(
+    tag: &str,
+    name: &'static str,
+    entry_type: u16,
+) -> RecordedFlow {
+    let mut flow = record_new_pool_file_creation(tag, name);
+    let leaf_copies = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
+    let leaf_slot = flow.writes[leaf_copies[0]].offset.0 / SLOT_BYTES;
+    corrupt_units_and_carry_their_whole_unit_checksums_up_to_the_root(
+        &mut flow,
+        &leaf_copies,
+        &|bytes| bytes[UNIT_HEADER_CLASS_TAG_OFFSET] = UNIT_CLASS_NOT_REGISTERED,
+        OwnChecksumsOfTheChangedUnits::LeftAsBeforeTheChange,
+    );
+    let root_copies = last_unit_copies(
+        &flow,
+        name,
+        &is_the_inode_tree_root_over_leaf_containers,
+        false,
+    );
+    let root = flow.writes[root_copies[0]].bytes().expect("单元写带字节");
+    assert_eq!(
+        device_and_slot_of_a_pointer_location(
+            &root[first_entry_of_a_container_number_node()
+                + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET..],
+            POINTER_LOCATION_ENTRY_OFFSETS[0],
+        )
+        .1,
+        leaf_slot,
+        "inode 树根第一条条目指着的就是改了类标签的那片叶容器"
+    );
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
+        set_the_type_of_the_first_entry(bytes, entry_type);
+    });
+    flow
+}
+
+/// 第 27 批：inode 树根条目类型段 0、身份引用三段不为 0（容器号、出生代照旧），孩子读不出（类标签改成不登记的码、不重封）：
+/// CPU 池级 checker 按 16 KiB 读孩子读不出，I-9.2 里不看孩子就判得了的身份三段照判、红（`judge_an_inode_entry_whose_child_does_not_read_back`），
+/// I-2.1 红、记走读失败；GPU 走读内核读不出那一支同判（第 26 批 H 改的 `w_walk_inode_root` 那一支），每个状态逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_with_an_identity_names_a_child_that_does_not_read_back(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_with_an_identity_names_a_child_that_does_not_read_back",
+        &record_new_pool_with_the_inode_root_entry_of_another_type_over_a_leaf_container_of_an_unregistered_class(
+            "crash-judge-gpu-i92-inode-root-entry-type-zero-unreadable-child",
+            "new-pool-inode-root-entry-type-zero-over-an-unreadable-child",
+            0,
+        ),
+        &["I-9.2", "I-2.1"],
+    );
+}
+
+/// 第 27 批：inode 树根条目类型段 2 → 7（不在 {0, 2} 里），孩子读不出（同上一条的叶容器）：CPU 池级 checker 不按哪一种长度去读孩子，
+/// 按层级 1 下面的叶容器两槽登记引用、判 I-9.2 红，不记 I-2.1、不记走读失败（与墓碑树、谱系树类型段不认得那一支同判）；
+/// GPU 走读内核同改，每个状态逐位相同（照旧按 16 KiB 读孩子的话 GPU 红 I-2.1、记走读失败，CPU 不红）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_that_does_not_read_back(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_without_a_walk_failure(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_that_does_not_read_back",
+        &record_new_pool_with_the_inode_root_entry_of_another_type_over_a_leaf_container_of_an_unregistered_class(
+            "crash-judge-gpu-i92-inode-root-entry-type-seven-unreadable-child",
+            "new-pool-inode-root-entry-type-seven-over-an-unreadable-child",
+            UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE,
+        ),
+        "I-9.2",
+    );
+}
+
+/// GPU 关门那一侧（判决 `research/prompts/m3-batch25-code-r1-main-verification.md` 第三节第 2 条）：最新根的实例表照 `change` 改成主走读走不到末片的样子，
+/// 门要关。CPU 一侧先钉流造对了——改之前整条落盘的镜像上 I-9.29 真被评估过且成立；改之后整条落盘的镜像上 I-9.29 报部分没判。
+/// 流上 C 还是最新根的那几个状态 CPU 红 I-9.29；GPU 每个状态逐位相同（实例 3 的根落了、C 写回了的状态上，GPU 门要是开了，
+/// 候选集里没有实例 2 那一行、滤不掉 C，GPU 红 I-9.29 而 CPU 不红）。
+fn assert_the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_newest_instance_table_stops_before_its_last_page(
+    test_name: &str,
+    tag: &str,
+    change: ChangeOfTheNewestInstanceTable,
+) {
+    let (flow, image_with_the_newest_instance_table_readable) =
+        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
+            tag, change,
+        );
+    assert_eq!(
+        pool_checker_verdict_on_the_image(&image_with_the_newest_instance_table_readable, "I-9.29"),
+        InvariantVerdict::Holds,
+        "改实例表之前：被抛弃的 C 按最新根的实例表滤掉，I-9.29 真被评估过且成立"
+    );
+    let mut image_with_every_write = flow.base.clone();
+    image_with_every_write.apply_writes(&flow.writes);
+    match pool_checker_verdict_on_the_image(&image_with_every_write, "I-9.29") {
+        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
+            unjudged.starts_with("最新根的实例表读不出"),
+            "部分没判的说明点名最新根的实例表读不出：{unjudged}"
+        ),
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::NotApplicable(_)) => panic!(
+            "流 {}：主走读沿最新根的实例表链走不到末片，I-9.29 要报部分没判，实际 {other_verdict:?}",
+            flow.name
+        ),
+    }
+    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        comparison.cpu_checker_red_invariants.contains("I-9.29"),
+        "流 {} 上 C 还是最新根的状态 CPU 红 I-9.29（红过的：{:?}）",
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
+
+/// 链指针记录不合法那一形（[`ChangeOfTheNewestInstanceTable::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance`]）：
+/// GPU 走读内核只在链指针记录说「没有下一片」时置「走到末片」（`chain == 0u`），把不合法的也当末片时这条流红。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_chain_record_of_the_newest_instance_table_is_invalid(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_newest_instance_table_stops_before_its_last_page(
+        "the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_chain_record_of_the_newest_instance_table_is_invalid",
+        "crash-judge-gpu-i929-newest-instance-table-invalid-chain-record",
+        ChangeOfTheNewestInstanceTable::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance,
+    );
+}
+
+/// 单片读不出那一形（[`ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceUnreadable`]）：第 0 片读得出、第 1 片两份都读不出。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_second_page_of_the_newest_instance_table_does_not_read_back(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_newest_instance_table_stops_before_its_last_page(
+        "the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_second_page_of_the_newest_instance_table_does_not_read_back",
+        "crash-judge-gpu-i929-newest-instance-table-second-page-unreadable",
+        ChangeOfTheNewestInstanceTable::SecondPageHoldingTheRowOfTheAbandonedInstanceUnreadable,
+    );
+}
+
+/// 攻方 H-c 那一形的流，录 `pages` 片（[`ChangeOfTheNewestInstanceTable::SpreadOverPagesWithoutTheRowOfTheAbandonedInstance`]），
+/// CPU 一侧先钉流造对了：整条落盘的镜像上主走读走到第 `pages` 片（末片）、门开，C 滤不掉，I-9.29 红。
+fn record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_spread_over_pages(
+    tag: &str,
+    pages: InstanceTableChainPages,
+) -> RecordedFlow {
+    let (flow, _) =
+        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
+            tag,
+            ChangeOfTheNewestInstanceTable::SpreadOverPagesWithoutTheRowOfTheAbandonedInstance(
+                pages,
+            ),
+        );
+    let mut image_with_every_write = flow.base.clone();
+    image_with_every_write.apply_writes(&flow.writes);
+    assert!(
+        matches!(
+            pool_checker_verdict_on_the_image(&image_with_every_write, "I-9.29"),
+            InvariantVerdict::Violated(_)
+        ),
+        "流 {}：CPU 主走读沿 {} 片的链走到末片、候选集里没有实例 2 那一行，被抛弃的 C 与现行线同号不同出生代，I-9.29 红：{:?}",
+        flow.name,
+        pages.count(),
+        pool_checker_verdict_on_the_image(&image_with_every_write, "I-9.29")
+    );
+    flow
+}
+
+/// H-c 对照那一形：链恰好 256 片（GPU 走读内核走得完的片数），两边都走到末片、都比，每个状态逐位相同、一个状态都不判不了。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_chain_of_as_many_pages_as_the_gpu_walks(
+) {
+    let flow =
+        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_spread_over_pages(
+            "crash-judge-gpu-i929-newest-instance-table-of-256-pages",
+            InstanceTableChainPages::AsManyAsTheGpuWalks,
+        );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_chain_of_as_many_pages_as_the_gpu_walks",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        comparison.cpu_checker_red_invariants.contains("I-9.29"),
+        "流 {} 上 CPU 红 I-9.29（红过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_red_invariants
+    );
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致或判不了（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
+    );
+}
+
+/// H-c（判决 `research/prompts/m3-batch25-code-r1-main-verification.md` 第二节 H-c 那一行与第三节第 1 条）：链 257 片，比 GPU 走读内核走得完的多一片。
+/// CPU 走到末片、比、红 I-9.29；GPU 走读内核走满上限链还没完，池级 checker 记判不了、原因字是实例表链过长
+/// （[`GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED`]），不当没走到末片（当成没走到末片的话门关、不比，GPU 不红 I-9.29 而 CPU 红）。
+/// 逐状态：池级 checker 判了的，违例位与 CPU 逐位相同；判不了的，原因字只能是实例表链过长，且至少一个这样的状态上 CPU 红 I-9.29；
+/// 照流水线那样判不了就回落 CPU（`gpu_verdict_items` 交回 None），回落之后每个状态的红项都与 CPU 相同。
+#[test]
+fn the_gpu_judge_leaves_to_the_cpu_a_newest_instance_table_chain_longer_than_the_pages_it_walks() {
+    let test_name =
+        "the_gpu_judge_leaves_to_the_cpu_a_newest_instance_table_chain_longer_than_the_pages_it_walks";
+    let flow =
+        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_spread_over_pages(
+            "crash-judge-gpu-i929-newest-instance-table-of-257-pages",
+            InstanceTableChainPages::OneMoreThanTheGpuWalks,
+        );
+    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let mut states_left_to_the_cpu_by_the_chain_limit = 0u64;
+    let mut one_birth_red_on_a_state_left_to_the_cpu_by_the_chain_limit = false;
+    let mut disagreements: Vec<String> = Vec::new();
+    for (ordinal, gpu_words) in comparison.gpu_words_of_each_state.iter().enumerate() {
+        let cpu_checker_bits = comparison.cpu_checker_bits_of_each_state[ordinal];
+        let cpu_red_items = &comparison.cpu_red_items_of_each_state[ordinal];
+        match gpu_words[POOL_CHECKER_UNDECIDABLE_WORD] {
+            0 => {
+                let gpu_checker_bits = pool_checker_violated_bits(gpu_words);
+                if gpu_checker_bits != cpu_checker_bits {
+                    disagreements.push(format!(
+                        "状态 {ordinal}：GPU 池级 checker 判了，违例 {:?}；CPU 违例 {:?}",
+                        invariant_names(gpu_checker_bits),
+                        invariant_names(cpu_checker_bits)
+                    ));
+                }
+            }
+            GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED => {
+                states_left_to_the_cpu_by_the_chain_limit += 1;
+                if invariant_names(cpu_checker_bits).contains(&"I-9.29") {
+                    one_birth_red_on_a_state_left_to_the_cpu_by_the_chain_limit = true;
+                }
+            }
+            other_reason => disagreements.push(format!(
+                "状态 {ordinal}：GPU 池级 checker 判不了的原因字 {other_reason:#x} 不是实例表链过长"
+            )),
+        }
+        let concluded = gpu_verdict_items(gpu_words)
+            .map_or_else(|| cpu_red_items.clone(), |items| format!("{items:?}"));
+        if concluded != *cpu_red_items {
+            disagreements.push(format!(
+                "状态 {ordinal}：判不了就回落 CPU 之后的红项 {concluded} 与 CPU 的 {cpu_red_items} 不同"
+            ));
+        }
+    }
+    eprintln!(
+        "CRASH_JUDGE_GPU_CHAIN_LIMIT flow={} states={} left_to_the_cpu_by_the_chain_limit={states_left_to_the_cpu_by_the_chain_limit}",
+        flow.name, comparison.states
+    );
+    assert!(
+        states_left_to_the_cpu_by_the_chain_limit > 0,
+        "流 {}：最新根的实例表链 257 片的状态上 GPU 池级 checker 要记判不了（实例表链过长）",
+        flow.name
+    );
+    assert!(
+        one_birth_red_on_a_state_left_to_the_cpu_by_the_chain_limit,
+        "流 {}：GPU 因链过长判不了的状态里，至少一个 CPU 红 I-9.29（CPU 走到末片、比）",
+        flow.name
+    );
+    assert!(
+        disagreements.is_empty(),
+        "流 {}：{} 处不一致（前几个）：\n{}",
+        flow.name,
+        disagreements.len(),
+        disagreements
+            .iter()
+            .take(6)
+            .cloned()
+            .collect::<Vec<_>>()
+            .join("\n")
+    );
+}
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index e7562dd..32a03a8 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -135,6 +135,16 @@ impl InodeInternalEntryType {
     }
 }
 
+/// inode 树内部条目里条款认得的那两种类型段（[`InodeInternalEntryType`] 去掉 `Unregistered`）：类型段不认得的条目在读孩子之前就判完、
+/// 不往下读（`Walk::walk_inode_internal_node`），往下读孩子的那几步只见这两种。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum RegisteredInodeInternalEntryType {
+    /// 类型段 0：子单元是码 2 的内部节点。
+    InternalNode,
+    /// 类型段 2：子单元是码 3、打包记录类型 2 的叶容器。
+    LeafContainer,
+}
+
 /// 墓碑树、谱系树内部条目的类型段 0：子单元是下一层码 2 节点（根长高之后，D18（块里携带什么信息） 已定项 24 第 1、5 条）。
 const CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE: u16 = 0;
 
@@ -2514,11 +2524,35 @@ impl<'reader> Walk<'reader> {
                 });
                 continue;
             }
-            let (length, span_in_slots) = match entry_type {
-                InodeInternalEntryType::LeafContainer => (data_unit_bytes(), 2),
-                InodeInternalEntryType::InternalNode | InodeInternalEntryType::Unregistered(_) => {
-                    (node_bytes(), 1)
+            let registered_entry_type = match entry_type {
+                InodeInternalEntryType::LeafContainer => {
+                    RegisteredInodeInternalEntryType::LeafContainer
+                }
+                InodeInternalEntryType::InternalNode => {
+                    RegisteredInodeInternalEntryType::InternalNode
                 }
+                InodeInternalEntryType::Unregistered(unregistered_type) => {
+                    // 类型段不认得、上面也没认出孩子自证过的类：不按哪一种长度去读孩子，与墓碑树、谱系树类型段不认得那一支同判
+                    // （`Self::walk_the_entries_of_a_container_tree_node`；`records/2026-10-01-删除与并发事务.md` 第 27 批，
+                    // 第 26 批 H 报告「交主 agent」第 1 条）：I-9.2 红，不记 I-2.1、不记走读失败。孩子先按这一层定的形状登记引用
+                    // （层级 1 的节点下面是码 3 叶容器两槽，层级 ≥ 2 的下面是码 2 节点一槽），不登记的话它占的槽不进引用并集，I-3.1 连带红。
+                    let child_span_in_slots_by_the_level = if node.level >= 2 { 1 } else { 2 };
+                    self.note_references_of_a_pointer_followed_by_the_walk(
+                        &child,
+                        child_span_in_slots_by_the_level,
+                        "inode 树的孩子",
+                    );
+                    self.judgements.judge("I-9.2", false, || {
+                        format!(
+                            "inode 内部条目的类型段 {unregistered_type} 不在 {{0, 2}} 里（孩子没有一份按它头里那一类的长度自证过，不读）"
+                        )
+                    });
+                    continue;
+                }
+            };
+            let (length, span_in_slots) = match registered_entry_type {
+                RegisteredInodeInternalEntryType::LeafContainer => (data_unit_bytes(), 2),
+                RegisteredInodeInternalEntryType::InternalNode => (node_bytes(), 1),
             };
             self.note_references_of_a_pointer_followed_by_the_walk(
                 &child,
@@ -2534,7 +2568,7 @@ impl<'reader> Walk<'reader> {
             ) else {
                 judge_an_inode_entry_whose_child_does_not_read_back(
                     &mut self.judgements,
-                    entry_type,
+                    registered_entry_type,
                     identity,
                 );
                 self.walk_failures
@@ -2542,8 +2576,8 @@ impl<'reader> Walk<'reader> {
                 continue;
             };
             let child_placement = (child.locations[0].device, child.locations[0].slot);
-            let subtree = match entry_type {
-                InodeInternalEntryType::LeafContainer => {
+            let subtree = match registered_entry_type {
+                RegisteredInodeInternalEntryType::LeafContainer => {
                     let header_identity = (
                         read_u64(&unit, 43),
                         read_u16(&unit, 51),
@@ -2561,7 +2595,7 @@ impl<'reader> Walk<'reader> {
                     });
                     self.walk_inode_leaf_container(&unit, &child, tree, child_placement)
                 }
-                InodeInternalEntryType::InternalNode => {
+                RegisteredInodeInternalEntryType::InternalNode => {
                     let (birth_tree, _, container, container_birth) = identity;
                     let entry_holds =
                         unit[6] == 2 && birth_tree == 0 && container == 0 && container_birth == 0;
@@ -2573,12 +2607,6 @@ impl<'reader> Walk<'reader> {
                     });
                     self.walk_inode_internal_child(&unit, &child, node.level, tree, child_placement)
                 }
-                InodeInternalEntryType::Unregistered(unregistered_type) => {
-                    self.judgements.judge("I-9.2", false, || {
-                        format!("inode 内部条目的类型段 {unregistered_type} 不在 {{0, 2}} 里")
-                    });
-                    continue;
-                }
             };
             if subtree.walked_first_by_this_version {
                 let smallest = subtree
@@ -4137,16 +4165,17 @@ fn child_unit_class_other_than_the_entry_type_names(
 /// inode 树一条内部条目的孩子两份都读不出（[`child_unit_class_other_than_the_entry_type_names`] 也没认出另一类）时，I-9.2 里不看孩子就判得了的那几小句，
 /// 与墓碑树、谱系树读不出孩子时同判（`Walk::walk_the_entries_of_a_container_tree_node`；判决
 /// `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K1 N1 那一行「inode 树读不出孩子时 I-9.2 不判」）：
-/// 类型段 0 判身份引用三段为 0（「子单元是码 2」那一小句孩子读不出不判）；类型段不认得的判该条目损坏；类型段 2 那几小句都要比孩子的头，读不出不判。
+/// 类型段 0 判身份引用三段为 0（「子单元是码 2」那一小句孩子读不出不判）；类型段 2 那几小句都要比孩子的头，读不出不判。
+/// 类型段不认得的条目不走到这里：调用方不读它的孩子、先判该条目损坏（第 27 批，与墓碑树、谱系树类型段不认得那一支同判）。
 /// 读不出照旧归 I-2.1 与走读失败（调用方记）。
 fn judge_an_inode_entry_whose_child_does_not_read_back(
     judgements: &mut Judgements,
-    entry_type: InodeInternalEntryType,
+    entry_type: RegisteredInodeInternalEntryType,
     identity: (u64, u16, u64, u64),
 ) {
     match entry_type {
-        InodeInternalEntryType::LeafContainer => {}
-        InodeInternalEntryType::InternalNode => {
+        RegisteredInodeInternalEntryType::LeafContainer => {}
+        RegisteredInodeInternalEntryType::InternalNode => {
             let (birth_tree, _, container, container_birth) = identity;
             let identity_is_zero = birth_tree == 0 && container == 0 && container_birth == 0;
             judgements.judge("I-9.2", identity_is_zero, || {
@@ -4155,13 +4184,6 @@ fn judge_an_inode_entry_whose_child_does_not_read_back(
                 )
             });
         }
-        InodeInternalEntryType::Unregistered(unregistered_type) => {
-            judgements.judge("I-9.2", false, || {
-                format!(
-                    "inode 内部条目的类型段 {unregistered_type} 不在 {{0, 2}} 里（孩子两份都读不出）"
-                )
-            });
-        }
     }
 }
 
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index a8c5fce..3da79ee 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -10882,13 +10882,15 @@ fn an_inode_tree_entry_of_type_zero_over_a_leaf_container_with_a_stale_copy_of_t
 /// 不在登记表里的类标签：孩子头里写成它之后，按哪一类的长度都自证不过，两份都读不出。
 const UNIT_CLASS_NOT_REGISTERED: u8 = 9;
 
-/// 新池新建文件那一版：inode 叶容器头里的类标签 3 → 9（不重封，整单元校验和沿引用链补上），再照 `change` 改 inode 树根第一条条目、重封到根。
-/// 孩子按自己的类标签认不出、按类型段的长度读不出：读不出归 I-2.1 与走读失败。
+/// I-1.11：映射条目 key 里的类标签与它指的单元头里的类标签相等（叶容器头里的类标签改成 [`UNIT_CLASS_NOT_REGISTERED`] 自己就带它红）。
+const MAPPING_KEY_CLASS_TAG_MATCHES_THE_UNIT_HEADER: &str = "I-1.11";
+
+/// 新池新建文件那一版（`image`）：inode 叶容器头里的类标签 3 → 9（不重封，整单元校验和沿引用链补上），再照 `change` 改 inode 树根第一条条目、重封到根。
+/// 孩子按自己的类标签认不出、按类型段的长度读不出：类型段认得的，读不出归 I-2.1 与走读失败。
 fn image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
-    tag: &str,
+    mut image: MemoryPool,
     change: impl FnOnce(&mut [u8]),
 ) -> MemoryPool {
-    let mut image = build_pool(tag).memory_pool();
     let child = child_pointer_of_an_entry_of_the_root_of_kind(&image, TREE_KIND_INODE, 0);
     assert_eq!(
         common_pointer_head_corruption::the_slot_of(&child).0,
@@ -10912,7 +10914,7 @@ fn image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
 fn an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
 ) {
     let image = image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
-        "known-bad-inode-entry-type-zero-identity-unreadable-child",
+        build_pool("known-bad-inode-entry-type-zero-identity-unreadable-child").memory_pool(),
         |entry| {
             change_the_entry_type(
                 entry,
@@ -10932,27 +10934,66 @@ fn an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_
     );
 }
 
-/// 同上一条，条目的类型段 2 → 7（不在 {0, 2} 里）：孩子读不出也判该条目损坏，I-9.2 红（与墓碑树、谱系树类型段不认得那一支同判）。
+/// 同上一条，条目的类型段 2 → 7（不在 {0, 2} 里）、孩子读不出：判该条目损坏，与墓碑树、谱系树类型段不认得那一支同判
+/// （`records/2026-10-01-删除与并发事务.md` 第 27 批，第 26 批 H 报告「交主 agent」第 1 条）——不按 16 KiB 去读孩子，孩子按层级 1 下面的叶容器两槽登记引用。
+/// 对照：同一张新池镜像只把这条条目的类型段改成 7、孩子完好（读得出、自证出码 3，按它自己那一类两槽登记引用、判 I-9.2 红、不往下走）。
+/// 孩子读不出的这一张红的不变量 = 对照所红（I-9.2）∪ 叶容器类标签改成 9 那一改自己带的 I-1.11（中央映射条目 key 里的类标签 3 与单元头不符，
+/// 与 inode 树怎么走无关）：不连带 I-2.1（改之前按 16 KiB 读孩子、校验和对不上）、不记走读失败（改之前连带 I-4.8、I-7.2、I-7.4），
+/// 也不连带 I-3.1（孩子占的两槽都进引用并集）。
 #[test]
-fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
+fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_only_the_inode_entry_invariant(
 ) {
+    let clean =
+        build_pool("known-bad-inode-entry-unregistered-type-unreadable-child").memory_pool();
+    let change_to_an_unregistered_type = |entry: &mut [u8]| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_INODE,
+            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
+        );
+    };
+    let mut image_with_a_readable_child = clean.clone();
+    change_an_entry_of_the_root_of_kind(
+        &mut image_with_a_readable_child,
+        TREE_KIND_INODE,
+        0,
+        change_to_an_unregistered_type,
+    );
+    let verdicts_with_a_readable_child = check_pool_image(&image_with_a_readable_child);
+    assert_eq!(
+        violated_invariants(&verdicts_with_a_readable_child),
+        ["I-9.2"],
+        "对照：类型段 7 的条目指着完好的叶容器，只红 I-9.2：{verdicts_with_a_readable_child:?}"
+    );
     let image = image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
-        "known-bad-inode-entry-unregistered-type-unreadable-child",
-        |entry| {
-            change_the_entry_type(
-                entry,
-                PACKED_RECORD_TYPE_INODE,
-                CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
-            );
-        },
+        clean,
+        change_to_an_unregistered_type,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [
+            MAPPING_KEY_CLASS_TAG_MATCHES_THE_UNIT_HEADER,
+            violated_invariants(&verdicts_with_a_readable_child)[0]
+        ],
+        "类型段不认得、孩子读不出：红的 = 孩子完好的对照所红（I-9.2）∪ 类标签那一改自己带的 I-1.11，不连带 I-2.1、走读失败那几条与 I-3.1：{verdicts:?}"
+    );
+    assert!(
+        matches!(
+            verdict(&image, MAPPING_KEY_CLASS_TAG_MATCHES_THE_UNIT_HEADER),
+            InvariantVerdict::Violated(detail)
+                if detail.contains("它指的单元头里的类标签是 9")
+        ),
+        "I-1.11 红在映射条目 key 里的类标签与改成 9 的叶容器头不符（类标签那一改自己带的，不是走 inode 树带出来的）：{:?}",
+        verdict(&image, MAPPING_KEY_CLASS_TAG_MATCHES_THE_UNIT_HEADER)
     );
     assert!(
         matches!(
             verdict(&image, "I-9.2"),
             InvariantVerdict::Violated(detail)
-                if detail.contains("类型段 7 不在 {0, 2} 里（孩子两份都读不出）")
+                if detail.contains("类型段 7 不在 {0, 2} 里（孩子没有一份按它头里那一类的长度自证过，不读）")
         ),
-        "类型段不认得、孩子读不出：I-9.2 照判该条目损坏：{:?}",
+        "违例说明写出不认得的类型段、孩子不读：{:?}",
         verdict(&image, "I-9.2")
     );
 }
````

## 实现员 j：mutations-append.tsv（/tmp/claude-1000/m3-batch27/j/patch/mutations-append.tsv）

````text
第27批 inode 树类型段不认得那一支孩子按一槽登记（层级 1 下面的叶容器少登记一槽，I-3.1 连带红）	crates/singlefs-checker/src/walk.rs	                    let child_span_in_slots_by_the_level = if node.level >= 2 { 1 } else { 2 };	                    let child_span_in_slots_by_the_level = 1;	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_only_the_inode_entry_invariant
第27批 GPU 判器 H-c：实例表链走满 256 片、链还没完也不记判不了（链 257 片那条流上门关、不比，GPU 不红 I-9.29 而 CPU 红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        undecidable = UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED;	        // 变异：走满上限、链还没完也不记判不了	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_leaves_to_the_cpu_a_newest_instance_table_chain_longer_than_the_pages_it_walks	the_gpu_judge_leaves_to_the_cpu_a_newest_instance_table_chain_longer_than_the_pages_it_walks
第27批 GPU 判器关门：链指针记录不合法也当走到末片（链指针记录不合法那条流上门开、比，GPU 红 I-9.29 而 CPU 部分没判）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            if (chain == 0u) {\n                w_newest_instance_table_chain_reached_its_last_page = true;	            if (chain != 1u) {\n                w_newest_instance_table_chain_reached_its_last_page = true;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_chain_record_of_the_newest_instance_table_is_invalid	the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_chain_record_of_the_newest_instance_table_is_invalid
第27批 GPU 判器读不出孩子时 I-9.2 类型段 0 身份三段不判（inode 树根条目类型段 0 带身份、叶容器读不出那条流上 CPU 红 I-9.2，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                w_judge(INVARIANT_I_9_2, is_zero64(identity_birth_tree) && is_zero64(identity_container) && is_zero64(identity_birth));	                w_judge(INVARIANT_I_9_2, true);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_with_an_identity_names_a_child_that_does_not_read_back	the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_with_an_identity_names_a_child_that_does_not_read_back
第27批 GPU 判器 inode 树类型段不认得、孩子读不出时照旧按 16 KiB 读（GPU 红 I-2.1、记走读失败，CPU 只红 I-9.2）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        if (class_the_inode_entry_type_names == NONE) {\n            var child_span_by_the_level = 2u;	        if (false) {\n            var child_span_by_the_level = 2u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_that_does_not_read_back	the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_that_does_not_read_back
第27批 GPU 判器 inode 树类型段不认得那一支孩子按一槽登记（层级 1 下面的叶容器少登记一槽，GPU 红 I-3.1 而 CPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            if (root.level >= 2u) {\n                child_span_by_the_level = 1u;	            if (root.level >= 1u) {\n                child_span_by_the_level = 1u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_that_does_not_read_back	the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_that_does_not_read_back
````

## 实现员 j：mutations-replacements.tsv（/tmp/claude-1000/m3-batch27/j/patch/mutations-replacements.tsv）

````text
第26批 inode 树读不出孩子时 I-9.2 不判（类型段 0 身份三段不为 0的条目只记 I-2.1 与走读失败）	crates/singlefs-checker/src/walk.rs	                judge_an_inode_entry_whose_child_does_not_read_back(\n                    &mut self.judgements,\n                    registered_entry_type,\n                    identity,\n                );	                // 变异：inode 树读不出孩子时 I-9.2 不判	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant
第26批 inode 树读不出孩子时 I-9.2 不判（类型段不认得的条目只记 I-2.1 与走读失败）	crates/singlefs-checker/src/walk.rs	                    self.judgements.judge("I-9.2", false, || {\n                        format!(\n                            "inode 内部条目的类型段 {unregistered_type} 不在 {{0, 2}} 里（孩子没有一份按它头里那一类的长度自证过，不读）"\n                        )\n                    });	                    let _ = read_referenced_unit(\n                        self.reader,\n                        &mut self.judgements,\n                        &child.locations,\n                        node_bytes(),\n                        "inode 树的孩子",\n                    );\n                    self.walk_failures\n                        .push(format!("变异：类型段 {unregistered_type} 的孩子按 16 KiB 读"));	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_only_the_inode_entry_invariant
````
