# 删除批第 29 批合入的改动原样（2026-10-06）

## 实现员 l：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch29/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index 2f48abc..75b3445 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -2637,18 +2637,20 @@ fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: v
         let class_of_the_inode_child = w_child_unit_class_other_than_the_entry_type_names(child, class_the_inode_entry_type_names);
         if (class_of_the_inode_child != NONE) {
             w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_inode_child));
+            w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
             w_judge(INVARIANT_I_9_2, class_of_the_inode_child == class_the_inode_entry_type_names);
             continue;
         }
         // 类型段不认得、上面也没认出孩子自证过的类：不按哪一种长度去读孩子（walk.rs walk_inode_internal_node 的 Unregistered 那一支，
         // 与墓碑树、谱系树类型段不认得那一支同判；第 27 批）——孩子按这一层定的形状登记引用（层级 1 下面是叶容器两槽，层级 ≥ 2 下面是码 2 节点一槽），
-        // 判 I-9.2 红，不记 I-2.1、不记走读失败
+        // 判 I-9.2 红；每一份副本照判 I-2.1、两份都对不上记走读失败（第 29 批，H-J3a）
         if (class_the_inode_entry_type_names == NONE) {
             var child_span_by_the_level = 2u;
             if (root.level >= 2u) {
                 child_span_by_the_level = 1u;
             }
             w_note_pointer_references(child, child_span_by_the_level);
+            w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
             w_judge(INVARIANT_I_9_2, false);
             continue;
         }
@@ -3006,6 +3008,40 @@ fn w_child_unit_class_other_than_the_entry_type_names(child: Ptr, class_the_entr
     return class_other_than_the_entry_type_names;
 }
 
+// 走读不往下读的孩子的一份副本与位置条目里的整单元校验和对不对得上（walk.rs 的 copy_matches_its_location_checksum_at_the_length_its_header_names）：
+// 头按最短的单元（码 2，16 KiB）那么长读，头里的类标签是登记过的码就只按那一类的长度比；没登记的码、或头读不出，16 KiB 与 32 KiB 两种长度都对不上
+// 才算对不上（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行的改法 F-α）
+fn w_copy_matches_its_location_checksum_at_the_length_its_header_names(copy: Slice, checksum: u32) -> bool {
+    if (copy.version == NONE) {
+        return false;
+    }
+    let class_in_the_header = rd8(copy.base, 6u);
+    if (class_in_the_header >= 1u && class_in_the_header <= 3u) {
+        return w_unit_verifies_at_the_length_of_its_class(copy, class_in_the_header, checksum);
+    }
+    return slice_crc_matches(copy, NODE_BYTES, checksum) || slice_crc_matches(copy, DATA_UNIT_BYTES, checksum);
+}
+
+// 走读不往下读的孩子（墓碑树、谱系树、inode 树内部条目里类型段不认得的，与孩子自证成类型段之外另一类的）：每一份副本照判 I-2.1，
+// 两份都对不上记走读失败（walk.rs 的 Walk::judge_every_copy_of_a_child_the_walk_does_not_go_into，第 29 批 H-J3a、H-J3b）。不往下走，引用由调用方先登记
+fn w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child: Ptr) {
+    let first_matches = w_copy_matches_its_location_checksum_at_the_length_its_header_names(
+        read_at(child.device0, slot_to_bytes(child.slot0), NODE_BYTES), child.sum0);
+    if (!first_matches) {
+        w_mismatches = w_mismatches + 1u;
+    }
+    w_judge(INVARIANT_I_2_1, first_matches);
+    let second_matches = w_copy_matches_its_location_checksum_at_the_length_its_header_names(
+        read_at(child.device1, slot_to_bytes(child.slot1), NODE_BYTES), child.sum1);
+    if (!second_matches) {
+        w_mismatches = w_mismatches + 1u;
+    }
+    w_judge(INVARIANT_I_2_1, second_matches);
+    if (!first_matches && !second_matches) {
+        w_failures = w_failures + 1u;
+    }
+}
+
 // 一个单元按它的类占几槽：码 2 节点一槽，码 1 数据单元与码 3 打包记录单元两槽
 fn w_slots_of_a_unit_of_class(unit_class: u32) -> u32 {
     if (unit_class == 2u) {
@@ -3057,6 +3093,7 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
             let class_of_the_child_of_an_internal_entry = w_child_unit_class_other_than_the_entry_type_names(child, 2u);
             if (class_of_the_child_of_an_internal_entry != NONE) {
                 w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_child_of_an_internal_entry));
+                w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
                 w_judge(identity_invariant, class_of_the_child_of_an_internal_entry == 2u);
                 continue;
             }
@@ -3096,6 +3133,7 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
                 span_of_the_unregistered_child = 1u;
             }
             w_note_pointer_references(child, span_of_the_unregistered_child);
+            w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
             w_judge(identity_invariant, false);
             continue;
         }
@@ -3103,6 +3141,7 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
         let class_of_the_child_of_a_container_entry = w_child_unit_class_other_than_the_entry_type_names(child, 3u);
         if (class_of_the_child_of_a_container_entry != NONE) {
             w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_child_of_a_container_entry));
+            w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
             w_judge(identity_invariant, class_of_the_child_of_a_container_entry == 3u);
             continue;
         }
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index 8a20a8e..ed398ae 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -5,6 +5,19 @@
 // （所选根的墓碑树），违例位并进违例位那几个字，走到第几步那一字写成 5、判不了时下一字写原因。只读不判的读法照 walk.rs 的 *_without_judging 各写一份。
 
 
+// 扫描内核沿实例表链最多读几片：与走读内核 crash_judge_checker.wgsl 的 INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED 同值（两个内核各自编译，
+// 共用段 crash_judge_checker_common.wgsl 里没有它，各写一份；对拍用例 crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs 钉两份相同）。
+// walk.rs 读链不设片数上限：走满这么多片、链还没完，记判不了交 CPU（判决 `research/prompts/m3-batch27-code-r1-main-verification.md`
+// 第二节 K1 J1 那一行 O-1、第三节第 2 条：不当读不出跳过 I-7.9 / I-3.12）。今天走读内核先把根环里每条自证过的根的链按同一个上限走一遍、
+// 走满先记它自己的判不了（站点号 61），扫描内核不跑；这里两处是走读内核哪天不再走某一类根时的接手（读码推的，没有一条流落到这里）
+const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 256u;
+
+// r_instance_pages_after_first（候选根的引用集合）走满 INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED 片、链还没完。「条目太多」一类，站点号与别处不撞
+const UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (37u << 8u);
+
+// r_instance_rows（一条根自己那张实例表的全部行，I-7.9 与 I-3.12 读它）走满 INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED 片、链还没完。「条目太多」一类，站点号与别处不撞
+const UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (38u << 8u);
+
 var<private> r_complete: bool;
 
 var<private> r_noting: bool;
@@ -477,7 +490,7 @@ fn r_note_every_inode_child_below(root: Node) {
 fn r_instance_pages_after_first(first: Ptr) {
     var p = first;
     var page = vec2<u32>(0u, 0u);
-    for (var step = 0u; step < 256u; step = step + 1u) {
+    for (var step = 0u; step < INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED; step = step + 1u) {
         let s = read_unit_via_locations(p, DATA_UNIT_BYTES);
         let packed = parse_packed_unit(s);
         if (!packed.ok) {
@@ -507,6 +520,7 @@ fn r_instance_pages_after_first(first: Ptr) {
         page = add64(page, u64_of(1u));
     }
     r_complete = false;
+    undecidable = UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED;
 }
 
 // 树表一条条目（偏移 o 起，宽 ≥ 200）的树 ID（偏移 0）与诞生 txg（偏移 108）记进信息行 row：同一棵树在这一片里再出现时后一条盖前一条
@@ -1095,7 +1109,7 @@ fn r_instance_rows(record_base: u32) -> bool {
     var p = parse_node_pointer(record_base, 170u);
     var page = vec2<u32>(0u, 0u);
     var rows = 0u;
-    for (var step = 0u; step < 256u; step = step + 1u) {
+    for (var step = 0u; step < INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED; step = step + 1u) {
         let s = read_unit_via_locations(p, DATA_UNIT_BYTES);
         if (s.version == NONE || s.length < DATA_UNIT_BYTES) {
             return false;
@@ -1140,6 +1154,7 @@ fn r_instance_rows(record_base: u32) -> bool {
         p = parse_node_pointer(s.base, last + 2u);
         page = add64(page, u64_of(1u));
     }
+    undecidable = UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED;
     return false;
 }
 
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 390557b..11ddd7e 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -31,6 +31,9 @@
 //! 第 28 批：孩子两份都读不出时照判「子指针头部的出生树 == 身份引用的出生树」（H-B）——inode 树根条目身份出生树加 1、墓碑树根条目子指针出生树加 1
 //! 各一条流钉 CPU 红 I-9.2 / I-9.23；inode 叶容器盘 1 那一份换成码 2 节点的字节、位置条目改成对得上它（H-V：两份都看）一条流钉 CPU 红 I-9.2；
 //! 同样换字节而位置条目不改（第二份副本自证不过）一条流钉 CPU 不红 I-9.2；GPU 都逐位跟上。
+//! 第 29 批：走读不往下读的孩子每一份副本照判 I-2.1——inode 树根条目类型段 7、子指针两条位置条目的校验和都坏（H-J3a）一条流钉 CPU 红 I-2.1、
+//! I-4.8、I-7.2、I-7.4，类型段 0、第一条位置条目的校验和坏（H-J3b）一条流钉 CPU 红 I-2.1，GPU 都逐位跟上；被崩溃恢复抛弃的根自己那张实例表链 257 片
+//! （O-1）一条钉 GPU 记判不了（实例表链过长）、回落 CPU 之后结论相同；走读内核与扫描内核的链片数上限相同一条（不用卡）。
 //! 第 35 批：由记录造出来的两类版本取带末条标志那条记录的新根段（三条记录的发布第一条写成上一版的根：下一次挂载那一版叠两条记录罩住同一个槽、
 //! 挂载之后由记录施加出来的那一版各一条），树表指针整条全零照 core 跟过去（根与它的记录、由记录施加出来的那一版的记录各一条），都整份比。
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
@@ -6283,17 +6286,25 @@ const PACKED_RECORD_TYPE_INSTANCE_TABLE: u16 = 4;
 /// 不重封，位置条目里的整单元校验和对不上，两份都读不出。
 const BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE: usize = 100;
 
-/// I-9.29（同号只许一个出生代） 判不了那一形（`.claude/kb/invariants.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」；
-/// 第 23 批 X 的 harness 坏镜像 `an_abandoned_root_naming_another_object_birth_leaves_the_one_birth_invariant_partially_judged_once_the_newest_instance_table_does_not_read_back`
-/// 同一段历史，录成流）：A (1, 3)、B (1, 4) → 重开取号 2（写行、暖机）→ C (2, 8)；崩溃恢复抛弃 C
+/// 被崩溃恢复抛弃的 C 那一段历史录成的流，与录制时 C 那次发布在录制流里占的那一截（[`record_a_root_abandoned_by_a_crash_recovery`] 交回）。
+struct FlowWithARootAbandonedByACrashRecovery {
+    flow: RecordedFlow,
+    /// `flow.writes` 每次写在录制流（扣掉 mkfs 那几步）里的下标。
+    stream_indexes: Vec<usize>,
+    /// 录制流（扣掉 mkfs 那几步）里 C 那次发布之前的步数与到它为止的步数：C 那次发布的写，录制流下标落在这两个数之间。
+    operations_before_the_abandoned_root: usize,
+    operations_through_the_abandoned_root: usize,
+    abandoned_root_checkpoint_txg: u64,
+}
+
+/// A (1, 3)、B (1, 4) → 重开取号 2（写行、暖机）→ C (2, 8)；崩溃恢复抛弃 C
 /// （`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：C 的根槽与数据单元暂时读不出、见证 C 的系统配置槽写回轮换之前那一代，
-/// 择根落到 C 之前那条根，实例 3 写行、暖机，再把暂存的字节原样写回）。录成流之后 C 那片 inode 叶容器两份里第一个文件那条记录的对象出生代加一、
-/// 重封到 C 的根（C 那一版与现行线上各版同号不同出生代），再照 `change` 改最新根（实例 3）指着的实例表（[`ChangeOfTheNewestInstanceTable`]）。
-/// 交回流，与改实例表之前这条流整条落盘之后的镜像（候选集滤得掉 C 的对照）。
-fn record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
+/// 择根落到 C 之前那条根，实例 3 写行、暖机，再把暂存的字节原样写回）。录成名叫 `name` 的流（每一版的内容都登记进 `versions`），
+/// 判的根是录制流里最后一次根槽写。
+fn record_a_root_abandoned_by_a_crash_recovery(
     tag: &str,
-    change: ChangeOfTheNewestInstanceTable,
-) -> (RecordedFlow, MemoryPool) {
+    name: &'static str,
+) -> FlowWithARootAbandonedByACrashRecovery {
     let mut pool = build_pool(tag);
     let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
     record_pool_build(&mut recorder, &pool);
@@ -6375,14 +6386,40 @@ fn record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance
         .iter()
         .rposition(|write| write.kind == StepKind::RootRecordFua)
         .expect("根槽写");
-    let mut flow = RecordedFlow {
-        name: change.flow_name(),
-        base: pool.memory_pool_after_mkfs(),
-        writes,
-        segments,
-        judged_root_index,
-        versions,
-    };
+    FlowWithARootAbandonedByACrashRecovery {
+        flow: RecordedFlow {
+            name,
+            base: pool.memory_pool_after_mkfs(),
+            writes,
+            segments,
+            judged_root_index,
+            versions,
+        },
+        stream_indexes,
+        operations_before_the_abandoned_root: operations_before_the_third,
+        operations_through_the_abandoned_root: operations_through_the_third,
+        abandoned_root_checkpoint_txg: third.root.checkpoint_txg.0,
+    }
+}
+
+/// I-9.29（同号只许一个出生代） 判不了那一形（`.claude/kb/invariants.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」；
+/// 第 23 批 X 的 harness 坏镜像 `an_abandoned_root_naming_another_object_birth_leaves_the_one_birth_invariant_partially_judged_once_the_newest_instance_table_does_not_read_back`
+/// 同一段历史，录成流）：A (1, 3)、B (1, 4) → 重开取号 2（写行、暖机）→ C (2, 8)；崩溃恢复抛弃 C
+/// （`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：C 的根槽与数据单元暂时读不出、见证 C 的系统配置槽写回轮换之前那一代，
+/// 择根落到 C 之前那条根，实例 3 写行、暖机，再把暂存的字节原样写回）。录成流之后 C 那片 inode 叶容器两份里第一个文件那条记录的对象出生代加一、
+/// 重封到 C 的根（C 那一版与现行线上各版同号不同出生代），再照 `change` 改最新根（实例 3）指着的实例表（[`ChangeOfTheNewestInstanceTable`]）。
+/// 交回流，与改实例表之前这条流整条落盘之后的镜像（候选集滤得掉 C 的对照）。
+fn record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
+    tag: &str,
+    change: ChangeOfTheNewestInstanceTable,
+) -> (RecordedFlow, MemoryPool) {
+    let FlowWithARootAbandonedByACrashRecovery {
+        mut flow,
+        stream_indexes,
+        operations_before_the_abandoned_root: operations_before_the_third,
+        operations_through_the_abandoned_root: operations_through_the_third,
+        abandoned_root_checkpoint_txg: _,
+    } = record_a_root_abandoned_by_a_crash_recovery(tag, change.flow_name());
     let inode_leaf_of_the_third: Vec<usize> = stream_indexes
         .iter()
         .enumerate()
@@ -6542,6 +6579,40 @@ const GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: usize = 256;
 /// （`crash_judge_checker.wgsl` 的 `UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED`：「条目太多」一类 3、站点号 61）。
 const GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED: u32 = 3 | (61 << 8);
 
+/// GPU 扫描内核沿实例表链读满 [`GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED`] 片、链还没完时交回的判不了原因字（`crash_judge_checker_scan.wgsl`，
+/// 「条目太多」一类 3）：候选根的引用集合那一处（`UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED`，站点号 37）、
+/// 一条根自己那张实例表的全部行那一处（`UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED`，I-7.9 与 I-3.12 读它，站点号 38）。
+const GPU_UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED: u32 =
+    3 | (37 << 8);
+const GPU_UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED: u32 = 3 | (38 << 8);
+
+/// 走读内核与扫描内核各写一份实例表链的片数上限（两个内核各自编译，共用段 `crash_judge_checker_common.wgsl` 里没有它）：两份都与
+/// [`GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED`] 相同（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第三节第 2 条
+/// 「上限用命名常量」）。只读两个内核的源码，不用卡。
+#[test]
+fn the_walk_kernel_and_the_scan_kernel_follow_instance_table_chains_up_to_the_same_number_of_pages()
+{
+    let declaration = format!(
+        "const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = {GPU_INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED}u;"
+    );
+    for (kernel, source) in [
+        (
+            "crash_judge_checker.wgsl",
+            include_str!("../src/crash_judge_checker.wgsl"),
+        ),
+        (
+            "crash_judge_checker_scan.wgsl",
+            include_str!("../src/crash_judge_checker_scan.wgsl"),
+        ),
+    ] {
+        assert_eq!(
+            source.matches(declaration.as_str()).count(),
+            1,
+            "{kernel} 里恰好一处 `{declaration}`"
+        );
+    }
+}
+
 /// 改造出来的实例表链有几片。
 #[derive(Clone, Copy, Debug)]
 enum InstanceTableChainPages {
@@ -6647,18 +6718,27 @@ fn chain_record_saying_a_next_page_without_a_pointer() -> Vec<u8> {
     record
 }
 
+/// 照 `pointer`（一条实例表指针）的头与两条位置条目的盘，两条位置条目的槽换成 `slot`、整单元校验和换成 `whole_unit_checksum`。
+fn pointer_naming_a_page_at(pointer: &[u8], slot: u64, whole_unit_checksum: u32) -> Vec<u8> {
+    let mut pointer = pointer.to_vec();
+    for location in POINTER_LOCATION_ENTRY_OFFSETS {
+        pointer[location + 4..location + 10].copy_from_slice(&slot.to_le_bytes()[..6]);
+        pointer[location + 10..location + 14].copy_from_slice(&whole_unit_checksum.to_le_bytes());
+    }
+    pointer
+}
+
 /// 指下一片的链指针记录：照 `pointer_to_the_first_page`（根记录里指第 0 片的那条实例表指针）的头，两条位置条目换成下一片的槽与整单元 CRC。
 fn chain_record_naming_the_next_page(
     pointer_to_the_first_page: &[u8],
     next_page_slot: u64,
     next_page_whole_unit_checksum: u32,
 ) -> Vec<u8> {
-    let mut pointer = pointer_to_the_first_page.to_vec();
-    for location in POINTER_LOCATION_ENTRY_OFFSETS {
-        pointer[location + 4..location + 10].copy_from_slice(&next_page_slot.to_le_bytes()[..6]);
-        pointer[location + 10..location + 14]
-            .copy_from_slice(&next_page_whole_unit_checksum.to_le_bytes());
-    }
+    let pointer = pointer_naming_a_page_at(
+        pointer_to_the_first_page,
+        next_page_slot,
+        next_page_whole_unit_checksum,
+    );
     let mut record = chain_record_of_the_last_page();
     record[INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET] = 1;
     record[INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET
@@ -6683,79 +6763,50 @@ enum PagesAfterTheFirstInTheFlow {
     BothCopiesFlippedWithoutResealing,
 }
 
-/// 最新根（实例 3）指着的那张单片实例表（`first_page_copies` 是写它的两份副本）改造成一条 `rows_of_each_page.len()` 片的链：
-/// 第 k 片放 `rows_of_each_page[k]` 再补链指针记录（末片说没有下一片，其余指下一片）。第 0 片就地改、重封到根；第 1 片起照第 0 片的头造
-/// （容器号写片号），两份写在 [`FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST`] 起每片两槽、与第 0 片同一对盘上，照 `pages_after_the_first`
-/// 写进流；这些写另起一段、插在写第 0 片那一段之前（第 0 片落了的状态上它们都已落）。
-fn rebuild_the_newest_instance_table_as_a_chain(
-    flow: &mut RecordedFlow,
-    first_page_copies: &[usize],
+/// 照 `template`（实例表的一片）的头造一段链：第 i 片（片号 `first_page_index + i`）放 `rows_of_each_page[i]` 再补链指针记录
+/// （末片说没有下一片，其余指下一片：照 `pointer_head`（一条实例表指针）的头，两条位置条目换成下一片的槽与整单元 CRC），容器号写片号、重封；
+/// 片号 k 的那一片在 [`FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST`] + 2k 槽。交回各片的 (槽, 字节)，按片号升序。
+fn instance_table_pages_from(
+    template: &[u8],
+    pointer_head: &[u8],
+    first_page_index: usize,
     rows_of_each_page: &[Vec<Vec<u8>>],
-    pages_after_the_first: PagesAfterTheFirstInTheFlow,
-) {
-    let first_page = flow.writes[first_page_copies[0]]
-        .bytes()
-        .expect("单元写带字节")
-        .to_vec();
-    let first_page_slot = flow.writes[first_page_copies[0]].offset.0 / SLOT_BYTES;
-    let pointer_to_the_first_page = flow
-        .writes
-        .iter()
-        .filter(|write| write.kind == StepKind::RootRecordFua)
-        .filter_map(RetainedWrite::bytes)
-        .map(|root| {
-            root[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
-                ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
-                .to_vec()
-        })
-        .find(|pointer| {
-            device_and_slot_of_a_pointer_location(pointer, POINTER_LOCATION_ENTRY_OFFSETS[0]).1
-                == first_page_slot
-        })
-        .expect("流里有根记录指着最新那张实例表");
-    let devices = POINTER_LOCATION_ENTRY_OFFSETS.map(|location| {
-        singlefs_core::address::DeviceIdentity(
-            device_and_slot_of_a_pointer_location(&pointer_to_the_first_page, location).0,
-        )
-    });
-    let mut pages_after_the_first_from_the_last: Vec<(u64, Vec<u8>)> = Vec::new();
+) -> Vec<(u64, Vec<u8>)> {
+    let mut pages_from_the_last: Vec<(u64, Vec<u8>)> = Vec::new();
     let mut next_page: Option<(u64, u32)> = None;
-    for page_index in (1..rows_of_each_page.len()).rev() {
-        let mut records = rows_of_each_page[page_index].clone();
+    for (index_in_the_run, rows) in rows_of_each_page.iter().enumerate().rev() {
+        let page_index =
+            u64::try_from(first_page_index + index_in_the_run).expect("片号装得进 u64");
+        let mut records = rows.clone();
         records.push(match next_page {
-            Some((slot, whole_unit_checksum)) => chain_record_naming_the_next_page(
-                &pointer_to_the_first_page,
-                slot,
-                whole_unit_checksum,
-            ),
+            Some((slot, whole_unit_checksum)) => {
+                chain_record_naming_the_next_page(pointer_head, slot, whole_unit_checksum)
+            }
             None => chain_record_of_the_last_page(),
         });
-        let mut page = first_page.clone();
+        let mut page = template.to_vec();
         replace_the_records_of_an_instance_table_page(&mut page, &records);
         page[PACKED_UNIT_CONTAINER_NUMBER_OFFSET..PACKED_UNIT_CONTAINER_NUMBER_OFFSET + 8]
-            .copy_from_slice(
-                &u64::try_from(page_index)
-                    .expect("片号装得进 u64")
-                    .to_le_bytes(),
-            );
+            .copy_from_slice(&page_index.to_le_bytes());
         reseal_unit(&mut page);
-        let slot = FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST
-            + 2 * u64::try_from(page_index).expect("片号装得进 u64");
+        let slot = FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST + 2 * page_index;
         next_page = Some((slot, crc32c(&page)));
-        pages_after_the_first_from_the_last.push((slot, page));
+        pages_from_the_last.push((slot, page));
     }
-    let mut first_page_records = rows_of_each_page[0].clone();
-    first_page_records.push(match next_page {
-        Some((slot, whole_unit_checksum)) => {
-            chain_record_naming_the_next_page(&pointer_to_the_first_page, slot, whole_unit_checksum)
-        }
-        None => chain_record_of_the_last_page(),
-    });
-    corrupt_units_and_reseal_up_to_the_root(flow, first_page_copies, &|bytes| {
-        replace_the_records_of_an_instance_table_page(bytes, &first_page_records);
-    });
+    pages_from_the_last.reverse();
+    pages_from_the_last
+}
+
+/// `pages`（[`instance_table_pages_from`] 交回的 (槽, 字节)）在 `devices` 两块盘上各写一份的单元写，照 `how` 写：核过每一片的槽在这块盘上
+/// 流里没写过、基镜像里全零。
+fn writes_of_instance_table_pages(
+    flow: &RecordedFlow,
+    devices: [singlefs_core::address::DeviceIdentity; 2],
+    pages: Vec<(u64, Vec<u8>)>,
+    how: PagesAfterTheFirstInTheFlow,
+) -> Vec<RetainedWrite> {
     let mut page_writes: Vec<RetainedWrite> = Vec::new();
-    for (slot, page) in pages_after_the_first_from_the_last.into_iter().rev() {
+    for (slot, page) in pages {
         let page_offset = slot * SLOT_BYTES;
         let page_end = page_offset + u64::try_from(page.len()).expect("单元长装得进 u64");
         for device in devices {
@@ -6763,7 +6814,7 @@ fn rebuild_the_newest_instance_table_as_a_chain(
                 flow.writes.iter().all(|write| write.device != device
                     || write.offset.0 + write.length_in_bytes() <= page_offset
                     || write.offset.0 >= page_end),
-                "第 1 片起写的槽 {slot} 在盘 {} 上流里没写过",
+                "实例表那一片写的槽 {slot} 在盘 {} 上流里没写过",
                 device.0
             );
             assert!(
@@ -6776,11 +6827,11 @@ fn rebuild_the_newest_instance_table_as_a_chain(
                 .expect("基镜像里读得到这一截")
                 .iter()
                 .all(|byte| *byte == 0),
-                "第 1 片起写的槽 {slot} 在盘 {} 的基镜像里全零",
+                "实例表那一片写的槽 {slot} 在盘 {} 的基镜像里全零",
                 device.0
             );
             let mut written = page.clone();
-            match pages_after_the_first {
+            match how {
                 PagesAfterTheFirstInTheFlow::Intact => {}
                 PagesAfterTheFirstInTheFlow::BothCopiesFlippedWithoutResealing => {
                     let flipped = written.len() - BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE;
@@ -6796,38 +6847,113 @@ fn rebuild_the_newest_instance_table_as_a_chain(
             });
         }
     }
+    page_writes
+}
+
+/// 一条实例表指针两条位置条目的盘。
+fn devices_of_an_instance_table_pointer(
+    pointer: &[u8],
+) -> [singlefs_core::address::DeviceIdentity; 2] {
+    POINTER_LOCATION_ENTRY_OFFSETS.map(|location| {
+        singlefs_core::address::DeviceIdentity(
+            device_and_slot_of_a_pointer_location(pointer, location).0,
+        )
+    })
+}
+
+/// 最新根（实例 3）指着的那张单片实例表（`first_page_copies` 是写它的两份副本）改造成一条 `rows_of_each_page.len()` 片的链：
+/// 第 k 片放 `rows_of_each_page[k]` 再补链指针记录（末片说没有下一片，其余指下一片）。第 0 片就地改、重封到根；第 1 片起照第 0 片的头造
+/// （[`instance_table_pages_from`]），两份写在与第 0 片同一对盘上，照 `pages_after_the_first` 写进流（[`writes_of_instance_table_pages`]）；
+/// 这些写另起一段、插在写第 0 片那一段之前（第 0 片落了的状态上它们都已落）。
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
+    let built_pages_after_the_first = instance_table_pages_from(
+        &first_page,
+        &pointer_to_the_first_page,
+        1,
+        &rows_of_each_page[1..],
+    );
+    let mut first_page_records = rows_of_each_page[0].clone();
+    first_page_records.push(match built_pages_after_the_first.first() {
+        Some((slot, page)) => {
+            chain_record_naming_the_next_page(&pointer_to_the_first_page, *slot, crc32c(page))
+        }
+        None => chain_record_of_the_last_page(),
+    });
+    corrupt_units_and_reseal_up_to_the_root(flow, first_page_copies, &|bytes| {
+        replace_the_records_of_an_instance_table_page(bytes, &first_page_records);
+    });
+    let page_writes = writes_of_instance_table_pages(
+        flow,
+        devices_of_an_instance_table_pointer(&pointer_to_the_first_page),
+        built_pages_after_the_first,
+        pages_after_the_first,
+    );
     let first_page_write = *first_page_copies.iter().min().expect("第 0 片至少一份");
-    let segment_of_the_first_page = flow
+    insert_a_segment_of_writes_before_the_segment_of(flow, first_page_write, page_writes);
+}
+
+/// `writes` 另起一段，插在 `write_index` 那次写所在那一段之前（那一段之前的各段之后）：之后各次写的下标与 `judged_root_index` 跟着挪。
+fn insert_a_segment_of_writes_before_the_segment_of(
+    flow: &mut RecordedFlow,
+    write_index: usize,
+    writes: Vec<RetainedWrite>,
+) {
+    let segment_of_the_write = flow
         .segments
         .iter()
-        .position(|segment| segment.contains(&first_page_write))
-        .expect("写第 0 片的那次写在某一段里");
-    let insertion = *flow.segments[segment_of_the_first_page]
+        .position(|segment| segment.contains(&write_index))
+        .expect("那次写在某一段里");
+    let insertion = *flow.segments[segment_of_the_write]
         .iter()
         .min()
         .expect("段里至少一次写");
     assert!(
-        flow.segments[..segment_of_the_first_page]
+        flow.segments[..segment_of_the_write]
             .iter()
             .flatten()
-            .all(|write_index| *write_index < insertion)
-            && flow.segments[segment_of_the_first_page..]
+            .all(|index| *index < insertion)
+            && flow.segments[segment_of_the_write..]
                 .iter()
                 .flatten()
-                .all(|write_index| *write_index >= insertion),
-        "段按写序排：插进来的写在写第 0 片那一段之前、之前各段之后"
+                .all(|index| *index >= insertion),
+        "段按写序排：插进来的写在那一段之前、之前各段之后"
     );
-    let inserted = page_writes.len();
-    flow.writes.splice(insertion..insertion, page_writes);
+    let inserted = writes.len();
+    flow.writes.splice(insertion..insertion, writes);
     for segment in &mut flow.segments {
-        for write_index in segment.iter_mut() {
-            if *write_index >= insertion {
-                *write_index += inserted;
+        for index in segment.iter_mut() {
+            if *index >= insertion {
+                *index += inserted;
             }
         }
     }
     flow.segments.insert(
-        segment_of_the_first_page,
+        segment_of_the_write,
         (insertion..insertion + inserted).collect(),
     );
     if flow.judged_root_index >= insertion {
@@ -6870,6 +6996,277 @@ fn insert_a_record_of_an_unregistered_kind_before_the_chain_record(bytes: &mut [
     );
 }
 
+/// 实例表一行里本实例最后一次发布的 `checkpoint_txg`（T，8 字节）的偏移（种类 1 字节、实例代号 4 字节之后）。
+const INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET: usize = 5;
+/// journal 记录新根段里实例表指针（86 字节）离新根段起点的偏移（walk.rs 的 `NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER`）。
+const JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET: usize = 188;
+/// C 自己那张实例表里改了 T 的那一行的实例代号：实例 2 写行那次写的表里只有实例 1 那一行。
+const INSTANCE_OF_THE_ROW_MOVED_TO_THE_ABANDONED_ROOT_TXG: u32 = 1;
+
+/// O-1 那一形（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 K1 J1 那一行、第三节第 2 条）：被崩溃恢复抛弃的 C
+/// （[`record_a_root_abandoned_by_a_crash_recovery`]；实例 3 的根落了之后 C 在回退候选集外）自己那张实例表换成一条 257 片的新链。
+/// C 的根记录原来指着实例 2 写行那次写的单片实例表（实例 2 的几条根共用），改指新链的第 0 片：第 0 片放那张表的行，实例 1 那一行的 T 改成
+/// C 自己的 txg（I-3.12「T < 这条根的 checkpoint_txg」不成立），其余各片只有链指针记录（[`instance_table_pages_from`]，头照那张表）；
+/// C 那次发布的根槽写（发布时一次、恢复之后原样写回一次）与 journal 记录新根段里的实例表指针都改指第 0 片、重封；新链各片两份另起一段，
+/// 插在 C 那次发布的记录与根槽写之前（引用它们的写落了的状态上它们都已落）。CPU 一侧先钉流造对了：整条落盘的镜像上 I-3.12 红。
+fn record_an_abandoned_root_whose_own_instance_table_is_a_chain_longer_than_the_gpu_walks(
+    tag: &str,
+) -> RecordedFlow {
+    let FlowWithARootAbandonedByACrashRecovery {
+        mut flow,
+        stream_indexes,
+        operations_before_the_abandoned_root,
+        operations_through_the_abandoned_root,
+        abandoned_root_checkpoint_txg,
+    } = record_a_root_abandoned_by_a_crash_recovery(
+        tag,
+        "abandoned-root-own-instance-table-of-257-pages-with-a-row-at-its-own-txg",
+    );
+    let root_slot_write_of_the_publish: Vec<usize> = (0..flow.writes.len())
+        .filter(|write_index| {
+            (operations_before_the_abandoned_root..operations_through_the_abandoned_root)
+                .contains(&stream_indexes[*write_index])
+                && flow.writes[*write_index].kind == StepKind::RootRecordFua
+        })
+        .collect();
+    assert_eq!(
+        root_slot_write_of_the_publish.len(),
+        1,
+        "C 那次发布写一次根槽"
+    );
+    let root_of_the_abandoned = flow.writes[root_slot_write_of_the_publish[0]]
+        .bytes()
+        .expect("根槽写带字节")
+        .to_vec();
+    assert_eq!(
+        read_u64_at(&root_of_the_abandoned, ROOT_RECORD_CHECKPOINT_TXG_OFFSET),
+        abandoned_root_checkpoint_txg,
+        "那次根槽写写的是 C"
+    );
+    let instance_of_the_abandoned = read_u32_at(&root_of_the_abandoned, 24);
+    let root_slot_writes_of_the_abandoned: Vec<usize> = (0..flow.writes.len())
+        .filter(|write_index| {
+            flow.writes[*write_index].kind == StepKind::RootRecordFua
+                && flow.writes[*write_index].bytes() == Some(root_of_the_abandoned.as_slice())
+        })
+        .collect();
+    assert_eq!(
+        root_slot_writes_of_the_abandoned.len(),
+        2,
+        "C 的根槽写两次：发布时一次，恢复之后原样写回一次"
+    );
+    let journal_records_of_the_abandoned: Vec<usize> = (0..flow.writes.len())
+        .filter(|write_index| {
+            flow.writes[*write_index].kind == StepKind::JournalRecord
+                && flow.writes[*write_index].bytes().is_some_and(|bytes| {
+                    read_u32_at(bytes, JOURNAL_RECORD_INSTANCE_OFFSET) == instance_of_the_abandoned
+                        && read_u64_at(bytes, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET)
+                            == abandoned_root_checkpoint_txg
+                })
+        })
+        .collect();
+    assert!(
+        !journal_records_of_the_abandoned.is_empty(),
+        "C 那次发布写了 journal 记录"
+    );
+    let pointer_to_the_shared_table = root_of_the_abandoned
+        [ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
+            ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
+        .to_vec();
+    let journal_pointer_field = JOURNAL_NEW_ROOT_SEGMENT_OFFSET
+        + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
+        ..JOURNAL_NEW_ROOT_SEGMENT_OFFSET
+            + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
+            + NODE_POINTER_BYTES;
+    for record in &journal_records_of_the_abandoned {
+        assert_eq!(
+            &flow.writes[*record].bytes().expect("记录写带字节")[journal_pointer_field.clone()],
+            pointer_to_the_shared_table.as_slice(),
+            "C 那次发布的记录新根段里的实例表指针与 C 的根记录相同"
+        );
+    }
+    let (shared_table_device, shared_table_slot) = device_and_slot_of_a_pointer_location(
+        &pointer_to_the_shared_table,
+        POINTER_LOCATION_ENTRY_OFFSETS[0],
+    );
+    let shared_table = flow.writes[..root_slot_write_of_the_publish[0]]
+        .iter()
+        .rev()
+        .find(|write| {
+            write.kind == StepKind::UnitWrite
+                && write.device.0 == shared_table_device
+                && write.offset.0 / SLOT_BYTES == shared_table_slot
+        })
+        .and_then(RetainedWrite::bytes)
+        .expect("C 之前流里写过 C 指着的那张实例表")
+        .to_vec();
+    assert!(
+        unit_class(&shared_table) == 3
+            && packed_record_type(&shared_table) == PACKED_RECORD_TYPE_INSTANCE_TABLE,
+        "C 的根记录指着一片实例表"
+    );
+    let mut rows = records_of_an_instance_table_page(&shared_table);
+    assert_eq!(
+        rows.pop(),
+        Some(chain_record_of_the_last_page()),
+        "C 指着的那张实例表只有一片"
+    );
+    let mut moved_rows = 0usize;
+    for row in &mut rows {
+        if row[0] == INSTANCE_TABLE_RECORD_KIND_ROW
+            && read_u32_at(row, INSTANCE_TABLE_ROW_INSTANCE_OFFSET)
+                == INSTANCE_OF_THE_ROW_MOVED_TO_THE_ABANDONED_ROOT_TXG
+        {
+            row[INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET
+                ..INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET + 8]
+                .copy_from_slice(&abandoned_root_checkpoint_txg.to_le_bytes());
+            moved_rows += 1;
+        }
+    }
+    assert_eq!(moved_rows, 1, "那张表里实例 1 恰好一行");
+    let mut rows_of_each_page = vec![rows];
+    rows_of_each_page.resize(
+        InstanceTableChainPages::OneMoreThanTheGpuWalks.count(),
+        Vec::new(),
+    );
+    let pages = instance_table_pages_from(
+        &shared_table,
+        &pointer_to_the_shared_table,
+        0,
+        &rows_of_each_page,
+    );
+    let pointer_to_the_new_first_page = pointer_naming_a_page_at(
+        &pointer_to_the_shared_table,
+        pages[0].0,
+        crc32c(&pages[0].1),
+    );
+    for root_slot_write in &root_slot_writes_of_the_abandoned {
+        rewrite_root_record_and_reseal(&mut flow, *root_slot_write, &|bytes| {
+            bytes[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
+                ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
+                .copy_from_slice(&pointer_to_the_new_first_page);
+        });
+    }
+    for record in &journal_records_of_the_abandoned {
+        rewrite_journal_record_and_reseal(&mut flow, *record, &|bytes| {
+            bytes[journal_pointer_field.clone()].copy_from_slice(&pointer_to_the_new_first_page);
+        });
+    }
+    let page_writes = writes_of_instance_table_pages(
+        &flow,
+        devices_of_an_instance_table_pointer(&pointer_to_the_shared_table),
+        pages,
+        PagesAfterTheFirstInTheFlow::Intact,
+    );
+    let first_write_naming_the_new_chain = *root_slot_writes_of_the_abandoned
+        .iter()
+        .chain(&journal_records_of_the_abandoned)
+        .min()
+        .expect("至少一次写");
+    insert_a_segment_of_writes_before_the_segment_of(
+        &mut flow,
+        first_write_naming_the_new_chain,
+        page_writes,
+    );
+    let mut image_with_every_write = flow.base.clone();
+    image_with_every_write.apply_writes(&flow.writes);
+    assert!(
+        matches!(
+            pool_checker_verdict_on_the_image(&image_with_every_write, "I-3.12"),
+            InvariantVerdict::Violated(_)
+        ),
+        "流 {}：CPU 沿 C 自己那张 257 片的实例表读到末片，实例 1 那一行的 T 等于 C 的 txg，I-3.12 红：{:?}",
+        flow.name,
+        pool_checker_verdict_on_the_image(&image_with_every_write, "I-3.12")
+    );
+    flow
+}
+
+/// O-1：回退候选集外的根（被崩溃恢复抛弃的 C）自己那张实例表链 257 片
+/// （[`record_an_abandoned_root_whose_own_instance_table_is_a_chain_longer_than_the_gpu_walks`]）。CPU 读链不设片数上限，读到末片、判 I-3.12 红；
+/// GPU 走满上限链还没完，池级 checker 记判不了（实例表链过长那三种原因字之一），不当读不出、跳过 I-3.12。逐状态：池级 checker 判了的，
+/// 违例位与 CPU 逐位相同；判不了的，原因字只能是实例表链过长，且至少一个这样的状态上 CPU 红 I-3.12；判不了就回落 CPU
+/// （`gpu_verdict_items` 交回 None），回落之后每个状态的红项都与 CPU 相同。
+/// 记哪一个原因字是推的（读码，没跑过）：走读内核把根环里每条自证过的根都另起一份走读走一遍（最新根、候选根、被抛弃的根、F 之下的根各一个阶段，
+/// `crash_judge_checker.wgsl` 的 `walk_and_hand_off`），每次先走它的实例表链，链 257 片先在那里记 61，扫描内核不跑；扫描内核那两处（37、38）
+/// 是走读内核哪天不再走某一类根时的接手，这条流上落不到。
+#[test]
+fn the_gpu_judge_leaves_to_the_cpu_an_instance_table_chain_longer_than_the_pages_it_walks_on_a_root_outside_the_candidates(
+) {
+    let test_name = "the_gpu_judge_leaves_to_the_cpu_an_instance_table_chain_longer_than_the_pages_it_walks_on_a_root_outside_the_candidates";
+    let flow =
+        record_an_abandoned_root_whose_own_instance_table_is_a_chain_longer_than_the_gpu_walks(
+            "crash-judge-gpu-i312-abandoned-root-instance-table-of-257-pages",
+        );
+    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let mut states_left_to_the_cpu_by_a_chain_limit = 0u64;
+    let mut rows_red_on_a_state_left_to_the_cpu_by_a_chain_limit = false;
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
+            GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED
+            | GPU_UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED
+            | GPU_UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED => {
+                states_left_to_the_cpu_by_a_chain_limit += 1;
+                if invariant_names(cpu_checker_bits).contains(&"I-3.12") {
+                    rows_red_on_a_state_left_to_the_cpu_by_a_chain_limit = true;
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
+        "CRASH_JUDGE_GPU_CHAIN_LIMIT flow={} states={} left_to_the_cpu_by_a_chain_limit={states_left_to_the_cpu_by_a_chain_limit}",
+        flow.name, comparison.states
+    );
+    assert!(
+        states_left_to_the_cpu_by_a_chain_limit > 0,
+        "流 {}：C 的实例表链 257 片、C 的根读得出的状态上 GPU 池级 checker 要记判不了（实例表链过长）",
+        flow.name
+    );
+    assert!(
+        rows_red_on_a_state_left_to_the_cpu_by_a_chain_limit,
+        "流 {}：GPU 因链过长判不了的状态里，至少一个 CPU 红 I-3.12（CPU 读到末片、判）",
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
+
 /// 删 → 回退 → 再删的合法历史（第 22 批 C 报告的合法历史之一）：新池新建文件、建空 inode 2（txg 4），把它 unlink 到底，挂着回退到 txg 4
 /// （谱系 (回退那次的 txg, 4)；墓碑树随现行那一版，头一条墓碑的死亡 txg 落在被回退抛弃的那一段里、不在谱系上），再把它 unlink 到底
 /// （又一条 (2, 同一个对象出生代) 的墓碑，在谱系上）。不按谱系滤的写法在这里判 I-9.20 不重号红。
@@ -7373,9 +7770,13 @@ fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_with_
     );
 }
 
-/// 第 27 批：inode 树根条目类型段 2 → 7（不在 {0, 2} 里），孩子读不出（同上一条的叶容器）：CPU 池级 checker 不按哪一种长度去读孩子，
+/// 第 27 批：inode 树根条目类型段 2 → 7（不在 {0, 2} 里），孩子读不出（同上一条的叶容器）：CPU 池级 checker 不往下走孩子，
 /// 按层级 1 下面的叶容器两槽登记引用、判 I-9.2 红，不记 I-2.1、不记走读失败（与墓碑树、谱系树类型段不认得那一支同判）；
 /// GPU 走读内核同改，每个状态逐位相同（照旧按 16 KiB 读孩子的话 GPU 红 I-2.1、记走读失败，CPU 不红）。
+/// 「不记 I-2.1」只对这一形成立（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行）：第 29 批起这一支
+/// 每一份副本照判 I-2.1，头里的类标签 9 不登记、16 KiB 与 32 KiB 两种长度都比，这片叶容器的位置条目换成了改后的整单元 CRC，按 32 KiB 对得上；
+/// 位置条目里的校验和本身坏了的那一形见
+/// [`the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong`]。
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_that_does_not_read_back(
 ) {
@@ -7390,6 +7791,90 @@ fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered
     );
 }
 
+// ───────────────────────── 第 29 批：走读不往下读的孩子每一份副本照判 I-2.1（H-J3a、H-J3b） ─────────────────────────
+
+/// 码 2 节点第一条条目子指针里第 `location` 条位置条目（[`POINTER_LOCATION_ENTRY_OFFSETS`] 之一）的整单元校验和第 0 字节翻一下：
+/// 孩子不动，那一份按哪一种长度都与位置条目里的校验和对不上。
+fn flip_a_byte_of_a_location_checksum_in_the_first_entry(bytes: &mut [u8], location: usize) {
+    let field = first_entry_of_a_container_number_node()
+        + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
+        + location
+        + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET;
+    bytes[field] ^= 0xFF;
+}
+
+/// 攻方 B1 那一形（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行；harness 坏镜像
+/// `an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants`
+/// 同一形，录成流）：新池新建文件的流上 inode 树根（层级 1）第一条条目的类型段 2 → 7、子指针两条位置条目的整单元校验和各翻一字节，重封到根；
+/// 叶容器不动。
+fn record_new_pool_with_the_inode_root_entry_of_an_unregistered_type_over_two_wrong_location_checksums(
+    tag: &str,
+) -> RecordedFlow {
+    record_new_pool_with_a_resealed_field(
+        tag,
+        "new-pool-inode-root-entry-type-seven-two-location-checksums-wrong",
+        &is_the_inode_tree_root_over_leaf_containers,
+        false,
+        &|bytes| {
+            set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
+            for location in POINTER_LOCATION_ENTRY_OFFSETS {
+                flip_a_byte_of_a_location_checksum_in_the_first_entry(bytes, location);
+            }
+        },
+    )
+}
+
+/// 第 29 批 H-J3a：类型段不认得那一支不往下走孩子，两份副本照判 I-2.1、都对不上、记走读失败：CPU 池级 checker 红 I-9.2、I-2.1、I-4.8、
+/// I-7.2、I-7.4（第 27 批这一支两样都不记，只红 I-9.2）；GPU 走读内核同改（`w_judge_every_copy_of_a_child_the_walk_does_not_go_into`），
+/// 每个状态逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong",
+        &record_new_pool_with_the_inode_root_entry_of_an_unregistered_type_over_two_wrong_location_checksums(
+            "crash-judge-gpu-i21-inode-root-entry-type-seven-two-location-checksums-wrong",
+        ),
+        &["I-9.2", "I-2.1", "I-4.8", "I-7.2", "I-7.4"],
+    );
+}
+
+/// H-J3b 那一形（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3b 那一行；harness 坏镜像
+/// `an_inode_tree_entry_of_type_zero_over_a_leaf_container_whose_first_location_checksum_is_wrong_reddens_the_checksum_invariant` 同一形，录成流）：
+/// 新池新建文件的流上 inode 树根（层级 1）第一条条目的类型段 2 → 0、子指针第一条位置条目的整单元校验和翻一字节，重封到根；叶容器不动。
+fn record_new_pool_with_the_inode_root_entry_of_type_zero_over_a_wrong_first_location_checksum(
+    tag: &str,
+) -> RecordedFlow {
+    record_new_pool_with_a_resealed_field(
+        tag,
+        "new-pool-inode-root-entry-type-zero-first-location-checksum-wrong",
+        &is_the_inode_tree_root_over_leaf_containers,
+        false,
+        &|bytes| {
+            set_the_type_of_the_first_entry(bytes, 0);
+            flip_a_byte_of_a_location_checksum_in_the_first_entry(
+                bytes,
+                POINTER_LOCATION_ENTRY_OFFSETS[0],
+            );
+        },
+    )
+}
+
+/// 第 29 批 H-J3b：第二份自证成码 3（类型段 0 说的是码 2），走「孩子是另一类」那一支，不往下走；第一份按哪一种长度都对不上，照判 I-2.1：
+/// CPU 池级 checker 红 I-9.2、I-2.1、I-4.8、I-7.4（第二份对得上、不记走读失败，I-7.2 不红；第 29 批之前这一支不判 I-2.1）；
+/// GPU 走读内核同改，每个状态逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container_whose_first_location_checksum_is_wrong(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container_whose_first_location_checksum_is_wrong",
+        &record_new_pool_with_the_inode_root_entry_of_type_zero_over_a_wrong_first_location_checksum(
+            "crash-judge-gpu-i21-inode-root-entry-type-zero-first-location-checksum-wrong",
+        ),
+        &["I-9.2", "I-2.1", "I-4.8", "I-7.4"],
+    );
+}
+
 // ───────────────────────── 第 28 批：读不出孩子时照判出生树那一小句（H-B）、孩子两份都看（H-V）、第二份副本的自证 ─────────────────────────
 
 /// 墓碑树、谱系树码 2 节点条目里子指针的偏移：与 inode 树内部条目同形（分隔 key 8 + 身份引用 26 之后）。
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index b4038db..14ad786 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -2143,7 +2143,8 @@ impl<'reader> Walk<'reader> {
     /// 墓碑树或谱系树的码 2 根（或根长高之后的内部节点）往下走：条目 120 字节（分隔 key 8 + 身份引用 26 + 子指针 86），
     /// 类型段 = `record_type` 的走到码 3 容器（判容器头、出生树、容器格式，单元记进这一版走到的落点），类型段 0 的走进下一层码 2 节点
     /// （层级 = 父层级 − 1），别的类型段判 `identity_invariant` 红、不往下走；孩子头里自证过的类标签与类型段说的那一类不符的同样判红、
-    /// 不往下走（[`Self::judge_a_container_tree_entry_whose_child_is_of_another_class`]，孩子读多长按它头里的类标签定、不按类型段定）。每条条目的身份按 `identity_invariant`（墓碑树 I-9.23、
+    /// 不往下走（[`Self::judge_a_container_tree_entry_whose_child_is_of_another_class`]，孩子读多长按它头里的类标签定、不按类型段定）；
+    /// 这两支不往下走的孩子每一份副本照判 I-2.1（[`Self::judge_every_copy_of_a_child_the_walk_does_not_go_into`]）。每条条目的身份按 `identity_invariant`（墓碑树 I-9.23、
     /// 谱系树 I-9.17）单独判定，身份不符不记走读失败（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T7 取 a）。
     /// 用 checker 自己的读法，不调 `singlefs_core`（D13（验证路线） 已定项 5）。
     /// 递归的上界：每进一层类型段 0，层级减一（层级 0 的码 2 节点在这两棵树里没有条款定的形状，记走读失败）。
@@ -2261,12 +2262,14 @@ impl<'reader> Walk<'reader> {
                 ContainerTreeEntryType::Unregistered(unregistered_type) => {
                     // 先登记引用再判红：不登记的话这个孩子占的槽不进引用并集，I-3.1（已分配统计对得上） 连带红（第二节 T7）。
                     // 孩子的形状由这一层定：层级 1 的节点下面是码 3 容器（两槽），层级 ≥ 2 的下面是码 2 节点（一槽）。
+                    // 孩子不往下走，每一份副本照判 I-2.1（第 29 批，H-J3a）。
                     let span_in_slots = if node.level >= 2 { 1 } else { 2 };
                     self.note_references_of_a_pointer_followed_by_the_walk(
                         &child,
                         span_in_slots,
                         &what,
                     );
+                    self.judge_every_copy_of_a_child_the_walk_does_not_go_into(&child, &what);
                     self.judgements.judge(identity_invariant, false, || {
                         format!("{name}内部条目的类型段 {unregistered_type} 不在 {{0, {record_type}}} 里")
                     });
@@ -2278,7 +2281,8 @@ impl<'reader> Walk<'reader> {
     /// 墓碑树、谱系树里一条条目的类型段说孩子是一类（类型段 0 ⇒ 码 2，类型段 = 打包记录类型 ⇒ 码 3），孩子头里自证过的类标签是另一类：
     /// I-9.23 / I-9.17 字面「0 ⇒ 子单元是码 2 … 1（谱系 5）⇒ 子单元是码 3」不成立，按 `identity_invariant` 判红，
     /// 「身份不符单独判定，不再记走读失败」：不往下走、不记走读失败。孩子按它自己那一类占的槽先登记引用（同类型段不认得那一支：
-    /// 不登记的话它占的槽不进引用并集，I-3.1（已分配统计对得上） 连带红）。
+    /// 不登记的话它占的槽不进引用并集，I-3.1（已分配统计对得上） 连带红）。每一份副本照判 I-2.1
+    /// （[`Self::judge_every_copy_of_a_child_the_walk_does_not_go_into`]，第 29 批，H-J3b：另一份校验和对不上的照红）。
     fn judge_a_container_tree_entry_whose_child_is_of_another_class(
         &mut self,
         child: &PointerView,
@@ -2290,11 +2294,9 @@ impl<'reader> Walk<'reader> {
     ) {
         let span_in_slots = slots_spanned_by_a_unit_of_the_class(child_class)
             .expect("child_unit_class_other_than_the_entry_type_names 只交回有长度的登记码");
-        self.note_references_of_a_pointer_followed_by_the_walk(
-            child,
-            span_in_slots,
-            &format!("{name}内部条目的子指针"),
-        );
+        let what = format!("{name}内部条目的子指针");
+        self.note_references_of_a_pointer_followed_by_the_walk(child, span_in_slots, &what);
+        self.judge_every_copy_of_a_child_the_walk_does_not_go_into(child, &what);
         self.judgements.judge(identity_invariant, false, || {
             format!(
                 "{name}内部条目类型段 {entry_type} 说孩子是 {class_the_entry_type_names:?}，孩子头里的类标签是 {child_class:?}（按那一类的长度读回、整单元校验和对得上）"
@@ -2302,6 +2304,37 @@ impl<'reader> Walk<'reader> {
         });
     }
 
+    /// 走读不往下读的孩子（墓碑树、谱系树、inode 树内部条目里类型段不认得的，与孩子自证成类型段之外另一类的）：每一份副本照判
+    /// I-2.1（`.claude/kb/invariants.md` I-2.1「任一被引用的块，其校验和与内容匹配」；判决 `research/prompts/m3-batch27-code-r1-main-verification.md`
+    /// 第二节 H-J3a、H-J3b 两行与第三节第 1 条）。一份对不对得上按
+    /// [`copy_matches_its_location_checksum_at_the_length_its_header_names`] 判；两份都对不上记走读失败（I-4.8、I-7.2、I-7.4 照旧跟着）。
+    /// 只判这两样：不往下走，引用由调用方按它定的形状先登记。
+    fn judge_every_copy_of_a_child_the_walk_does_not_go_into(
+        &mut self,
+        child: &PointerView,
+        what: &str,
+    ) {
+        let mut some_copy_matches = false;
+        for location in &child.locations {
+            let matches = copy_matches_its_location_checksum_at_the_length_its_header_names(
+                self.reader,
+                location,
+            );
+            self.judgements.judge("I-2.1", matches, || {
+                format!(
+                    "{what}（走读不往下读的孩子）在盘 {} 槽 {} 的那一份按头里那一类的长度（没登记的 16 KiB、32 KiB 都试）与位置条目里的校验和对不上",
+                    location.device, location.slot
+                )
+            });
+            some_copy_matches |= matches;
+        }
+        if !some_copy_matches {
+            self.walk_failures.push(format!(
+                "{what}（走读不往下读的孩子）两份都与位置条目里的校验和对不上"
+            ));
+        }
+    }
+
     /// 墓碑树、谱系树里类型段 0 那条条目下面的码 2 节点：层级要是父层级 − 1，条目宽 120 才往下走（[`Self::walk_container_tree_root`]）。
     #[allow(
         clippy::too_many_arguments,
@@ -2517,7 +2550,8 @@ impl<'reader> Walk<'reader> {
             );
             // I-9.2 判孩子的类按孩子头里的类标签认，不按类型段定读多长（同 I-9.17 / I-9.23，判决
             // `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N4 那一行「inode 树 I-9.2 同形」）：
-            // 孩子自证过的类与类型段说的不符（类型段不认得的，孩子读得出）判 I-9.2 红，按它自己那一类占的槽登记引用，不往下走、不记走读失败。
+            // 孩子自证过的类与类型段说的不符（类型段不认得的，孩子读得出）判 I-9.2 红，按它自己那一类占的槽登记引用，不往下走、不记走读失败；
+            // 每一份副本照判 I-2.1（第 29 批，H-J3b：自证成另一类的之外那一份校验和对不上的照红）。
             let class_the_entry_type_names = match entry_type {
                 InodeInternalEntryType::LeafContainer => {
                     Some(MappingKeyUnitClass::PackedRecordUnit)
@@ -2538,6 +2572,10 @@ impl<'reader> Walk<'reader> {
                     span_in_slots,
                     "inode 树的孩子",
                 );
+                self.judge_every_copy_of_a_child_the_walk_does_not_go_into(
+                    &child,
+                    "inode 树的孩子",
+                );
                 self.judgements.judge("I-9.2", false, || {
                     format!(
                         "inode 内部条目类型段 {record_type} 说孩子是 {class_the_entry_type_names:?}，孩子头里的类标签是 {child_class:?}（按那一类的长度读回、整单元校验和对得上）"
@@ -2553,19 +2591,25 @@ impl<'reader> Walk<'reader> {
                     RegisteredInodeInternalEntryType::InternalNode
                 }
                 InodeInternalEntryType::Unregistered(unregistered_type) => {
-                    // 类型段不认得、上面也没认出孩子自证过的类：不按哪一种长度去读孩子，与墓碑树、谱系树类型段不认得那一支同判
+                    // 类型段不认得、上面也没认出孩子自证过的类：不往下走孩子，与墓碑树、谱系树类型段不认得那一支同判
                     // （`Self::walk_the_entries_of_a_container_tree_node`；`records/2026-10-01-删除与并发事务.md` 第 27 批，
-                    // 第 26 批 H 报告「交主 agent」第 1 条）：I-9.2 红，不记 I-2.1、不记走读失败。孩子先按这一层定的形状登记引用
+                    // 第 26 批 H 报告「交主 agent」第 1 条）：I-9.2 红。孩子先按这一层定的形状登记引用
                     // （层级 1 的节点下面是码 3 叶容器两槽，层级 ≥ 2 的下面是码 2 节点一槽），不登记的话它占的槽不进引用并集，I-3.1 连带红。
+                    // 孩子不往下走，每一份副本照判 I-2.1、两份都对不上记走读失败（第 29 批，H-J3a：第 27 批这一支两样都不记，
+                    // 位置条目里的校验和坏了也报 I-2.1 成立）。
                     let child_span_in_slots_by_the_level = if node.level >= 2 { 1 } else { 2 };
                     self.note_references_of_a_pointer_followed_by_the_walk(
                         &child,
                         child_span_in_slots_by_the_level,
                         "inode 树的孩子",
                     );
+                    self.judge_every_copy_of_a_child_the_walk_does_not_go_into(
+                        &child,
+                        "inode 树的孩子",
+                    );
                     self.judgements.judge("I-9.2", false, || {
                         format!(
-                            "inode 内部条目的类型段 {unregistered_type} 不在 {{0, 2}} 里（孩子没有一份按它头里那一类的长度自证过，不读）"
+                            "inode 内部条目的类型段 {unregistered_type} 不在 {{0, 2}} 里（孩子没有一份按它头里那一类的长度自证过，不往下走）"
                         )
                     });
                     continue;
@@ -4189,13 +4233,37 @@ fn child_unit_class_other_than_the_entry_type_names(
     class_other_than_the_entry_type_names
 }
 
+/// 走读不往下读的孩子的一份副本（一条位置条目）与位置条目里的整单元校验和对不对得上：头按最短的单元（码 2，16 KiB）那么长读，
+/// 头里的类标签是登记过的码就只按那一类的长度比；没登记的码、或头读不出，16 KiB 与 32 KiB 两种长度都对不上才算对不上
+/// （判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行的改法 F-α）。只读、不判。
+fn copy_matches_its_location_checksum_at_the_length_its_header_names(
+    reader: &dyn ImageReader,
+    location: &PointerLocation,
+) -> bool {
+    let offset = location.slot * SLOT_BYTES;
+    let length_of_the_class_in_the_header = reader
+        .read(location.device, offset, node_bytes())
+        .map(|header| MappingKeyUnitClass::of(header[UNIT_HEADER_CLASS_TAG_OFFSET]))
+        .and_then(unit_bytes_of_the_class);
+    let lengths_to_compare = match length_of_the_class_in_the_header {
+        Some(unit_bytes) => vec![unit_bytes],
+        None => vec![node_bytes(), data_unit_bytes()],
+    };
+    lengths_to_compare.into_iter().any(|unit_bytes| {
+        reader
+            .read(location.device, offset, unit_bytes)
+            .is_some_and(|content| crc32_castagnoli_table(&content) == location.checksum)
+    })
+}
+
 /// inode 树一条内部条目的孩子两份都读不出（[`child_unit_class_other_than_the_entry_type_names`] 也没认出另一类）时，I-9.2 里不看孩子就判得了的那几小句，
 /// 与墓碑树、谱系树读不出孩子时同判（`Walk::walk_the_entries_of_a_container_tree_node`；判决
 /// `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K1 N1 那一行「inode 树读不出孩子时 I-9.2 不判」）：
 /// 类型段 0 判身份引用三段为 0（「子单元是码 2」那一小句孩子读不出不判）；类型段 2 判「子指针头部的出生树 == 身份引用的出生树」
 /// （只比父条目里的两段，判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第二节 K2 H3 H-B 那一行、第三节第 1 条），
 /// 子头四段与身份引用相等、子容器的打包记录类型 == 2 那几小句要比孩子的头，读不出不判。
-/// 类型段不认得的条目不走到这里：调用方不读它的孩子、先判该条目损坏（第 27 批，与墓碑树、谱系树类型段不认得那一支同判）。
+/// 类型段不认得的条目不走到这里：调用方不往下走它的孩子、判该条目损坏（第 27 批，与墓碑树、谱系树类型段不认得那一支同判），
+/// 每一份副本只判 I-2.1（第 29 批，`Walk::judge_every_copy_of_a_child_the_walk_does_not_go_into`）。
 /// 读不出照旧归 I-2.1 与走读失败（调用方记）。
 fn judge_an_inode_entry_whose_child_does_not_read_back(
     judgements: &mut Judgements,
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index a168ee0..5d5f0e1 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -10952,6 +10952,10 @@ fn an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_
 /// 孩子读不出的这一张红的不变量 = 对照所红（I-9.2）∪ 叶容器类标签改成 9 那一改自己带的 I-1.11（中央映射条目 key 里的类标签 3 与单元头不符，
 /// 与 inode 树怎么走无关）：不连带 I-2.1（改之前按 16 KiB 读孩子、校验和对不上）、不记走读失败（改之前连带 I-4.8、I-7.2、I-7.4），
 /// 也不连带 I-3.1（孩子占的两槽都进引用并集）。
+/// 「不连带 I-2.1」只对这一张镜像成立（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行）：第 29 批起这一支
+/// 每一份副本照判 I-2.1，头里的类标签 9 不登记、16 KiB 与 32 KiB 两种长度都比，这张镜像的叶容器类标签改坏、整单元校验和沿引用链补齐了，
+/// 按 32 KiB 对得上；位置条目里的校验和本身坏了的那一形见
+/// [`an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants`]。
 #[test]
 fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_only_the_inode_entry_invariant(
 ) {
@@ -11003,13 +11007,228 @@ fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read
         matches!(
             verdict(&image, "I-9.2"),
             InvariantVerdict::Violated(detail)
-                if detail.contains("类型段 7 不在 {0, 2} 里（孩子没有一份按它头里那一类的长度自证过，不读）")
+                if detail.contains("类型段 7 不在 {0, 2} 里（孩子没有一份按它头里那一类的长度自证过，不往下走）")
         ),
-        "违例说明写出不认得的类型段、孩子不读：{:?}",
+        "违例说明写出不认得的类型段、孩子不往下走：{:?}",
         verdict(&image, "I-9.2")
     );
 }
 
+/// 内部条目（120 字节）子指针里第 `location_index` 条位置条目的整单元校验和第 0 字节翻一下（位置条目 = 盘 4 + 槽 6 + 整单元校验和 4，
+/// 子指针从条目偏移 [`CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET`] 起）：孩子本身不动，那一份按哪一种长度都与位置条目里的校验和对不上。
+fn flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(
+    entry: &mut [u8],
+    location_index: usize,
+) {
+    let checksum_offset = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
+        + POINTER_FIRST_LOCATION_OFFSET
+        + location_index * POINTER_LOCATION_BYTES
+        + LOCATION_CHECKSUM_OFFSET;
+    entry[checksum_offset] ^= 0xff;
+}
+
+/// 走读不往下读的孩子每一份副本照判 I-2.1（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a、H-J3b 两行与第三节第 1 条，
+/// `.claude/kb/invariants.md` I-2.1「任一被引用的块，其校验和与内容匹配」）：`image` 红的恰好是 `expected_violations`，
+/// I-2.1 的违例说明点名「走读不往下读的孩子」（红在新判的那一处，不是别的单元）；`expected_violations` 里有 I-7.2 的，
+/// 它的违例说明点名两份都对不上（走读失败是这一处记的）。
+fn assert_the_child_the_walk_does_not_go_into_reddens_exactly(
+    image: &MemoryPool,
+    expected_violations: &[&'static str],
+    scene: &str,
+) {
+    let verdicts = check_pool_image(image);
+    let verdict_of = |invariant: &str| {
+        verdicts
+            .iter()
+            .find(|(identifier, _)| *identifier == invariant)
+            .expect("清单里有")
+            .1
+            .clone()
+    };
+    assert_eq!(
+        violated_invariant_set(&verdicts),
+        expected_violations
+            .iter()
+            .copied()
+            .collect::<std::collections::BTreeSet<&'static str>>(),
+        "{scene}：{verdicts:?}"
+    );
+    assert!(
+        matches!(
+            verdict_of("I-2.1"),
+            InvariantVerdict::Violated(detail) if detail.contains("走读不往下读的孩子")
+        ),
+        "{scene}：I-2.1 红在走读不往下读的那个孩子上：{:?}",
+        verdict_of("I-2.1")
+    );
+    if expected_violations.contains(&"I-7.2") {
+        assert!(
+            matches!(
+                verdict_of("I-7.2"),
+                InvariantVerdict::Violated(detail)
+                    if detail.contains("（走读不往下读的孩子）两份都与位置条目里的校验和对不上")
+            ),
+            "{scene}：I-7.2 红在走读不往下读的孩子两份都对不上：{:?}",
+            verdict_of("I-7.2")
+        );
+    }
+}
+
+/// 攻方 B1 那一形（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行，`research/prompts/m3-batch27-code-r1-opus-output.md`
+/// 第一节）：新池新建文件那一版，inode 树根（层级 1）第一条条目的类型段 2 → 7、子指针两条位置条目的整单元校验和各翻一字节，重封到根；
+/// 叶容器不动。孩子不往下走，两份副本照判 I-2.1、都对不上、记走读失败：红 I-9.2、I-2.1、I-4.8、I-7.2、I-7.4
+/// （第 27 批这一支两样都不记，只红 I-9.2）。
+#[test]
+fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
+) {
+    let mut image =
+        build_pool("known-bad-inode-entry-unregistered-type-both-location-checksums-wrong")
+            .memory_pool();
+    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_INODE,
+            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
+        );
+        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
+        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
+    });
+    assert_eq!(level, 1, "inode 树根直接管叶容器");
+    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
+        &image,
+        &["I-9.2", "I-2.1", "I-4.8", "I-7.2", "I-7.4"],
+        "inode 树根条目类型段 7、子指针两条位置条目的校验和都坏",
+    );
+}
+
+/// 同上，墓碑树那一侧（I-9.23；第 27 批对齐的目标那一支，同一缺口在那里早就有）：三个单元的文件发完意图之后，墓碑树根（层级 1）
+/// 第一条条目的类型段 1 → 7、子指针两条位置条目的整单元校验和各翻一字节，重封到根；墓碑容器不动。
+#[test]
+fn a_tombstone_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
+) {
+    let mut image = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+        "known-bad-tombstone-entry-unregistered-type-both-location-checksums-wrong",
+    );
+    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_TOMBSTONE,
+            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
+        );
+        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
+        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
+    });
+    assert_eq!(level, 1, "墓碑树根直接管容器");
+    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
+        &image,
+        &[
+            TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+            "I-2.1",
+            "I-4.8",
+            "I-7.2",
+            "I-7.4",
+        ],
+        "墓碑树根条目类型段 7、子指针两条位置条目的校验和都坏",
+    );
+}
+
+/// 同上，谱系树那一侧（I-9.17）：回退到 A 之后，谱系树根（层级 1）第一条条目的类型段 5 → 7、子指针两条位置条目的整单元校验和各翻一字节，
+/// 重封到根；谱系容器不动。
+#[test]
+fn a_lineage_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
+) {
+    let mut image = image_after_rolling_back_to_the_first_root(
+        "known-bad-lineage-entry-unregistered-type-both-location-checksums-wrong",
+    )
+    .image;
+    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_LINEAGE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_LINEAGE,
+            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
+        );
+        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
+        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
+    });
+    assert_eq!(level, 1, "谱系树根直接管容器");
+    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
+        &image,
+        &[
+            LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+            "I-2.1",
+            "I-4.8",
+            "I-7.2",
+            "I-7.4",
+        ],
+        "谱系树根条目类型段 7、子指针两条位置条目的校验和都坏",
+    );
+}
+
+/// H-J3b（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3b 那一行，第 27 批之前就有）：新池新建文件那一版，
+/// inode 树根（层级 1）第一条条目的类型段 2 → 0、子指针第一条位置条目的整单元校验和翻一字节，重封到根；叶容器不动。
+/// 第二份自证成码 3（类型段 0 说的是码 2），走「孩子是另一类」那一支：I-9.2 红、不往下走；第一份按哪一种长度都对不上，照判 I-2.1 红，
+/// I-4.8、I-7.4 跟着；第二份对得上，不记走读失败、I-7.2 不红（改之前这一支不判 I-2.1，映射树那一遍按映射条目的校验和读、也判不到）。
+#[test]
+fn an_inode_tree_entry_of_type_zero_over_a_leaf_container_whose_first_location_checksum_is_wrong_reddens_the_checksum_invariant(
+) {
+    let mut image =
+        build_pool("known-bad-inode-entry-type-zero-over-a-leaf-first-location-checksum-wrong")
+            .memory_pool();
+    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_INODE,
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        );
+        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
+    });
+    assert_the_violation_names_the_child_class(
+        &image,
+        "I-9.2",
+        CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        "PackedRecordUnit",
+    );
+    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
+        &image,
+        &["I-9.2", "I-2.1", "I-4.8", "I-7.4"],
+        "inode 树根条目类型段 0 指着叶容器、子指针第一条位置条目的校验和坏",
+    );
+}
+
+/// H-J3b 墓碑树那一侧（`Walk::judge_a_container_tree_entry_whose_child_is_of_another_class`，墓碑树与谱系树共用）：三个单元的文件发完意图之后，
+/// 墓碑树根（层级 1）第一条条目的类型段 1 → 0、子指针第一条位置条目的整单元校验和翻一字节，重封到根；墓碑容器不动。
+#[test]
+fn a_tombstone_tree_entry_of_type_zero_over_a_container_whose_first_location_checksum_is_wrong_reddens_the_checksum_invariant(
+) {
+    let mut image = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+        "known-bad-tombstone-entry-type-zero-over-a-container-first-location-checksum-wrong",
+    );
+    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_TOMBSTONE,
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        );
+        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
+    });
+    assert_the_violation_names_the_child_class(
+        &image,
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        "PackedRecordUnit",
+    );
+    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
+        &image,
+        &[
+            TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+            "I-2.1",
+            "I-4.8",
+            "I-7.4",
+        ],
+        "墓碑树根条目类型段 0 指着容器、子指针第一条位置条目的校验和坏",
+    );
+}
+
 /// 第 28 批正推线索②（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 3 条「读不出孩子那两条坏镜像补『只改身份一段』三形」）：
 /// inode 树根第一条条目类型段 2 → 0、身份引用三段先清零再只把 `field` 那一段写成 1，孩子读不出（同
 /// [`an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant`]）。
````

## 实现员 l：mutations-append.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch29/patch/mutations-append.tsv）

````text
第29批 H-J3a/H-J3b：走读不往下读的孩子那一份与位置条目对不上也不判 I-2.1（类型段 7、两条位置条目校验和都坏的 inode 树根条目上 I-2.1 报成立）	crates/singlefs-checker/src/walk.rs	            self.judgements.judge("I-2.1", matches, || {	            self.judgements.judge("I-2.1", true, || {	-p singlefs-harness --test checker_known_bad_images -- location_checksum	an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants
第29批 F-α：头里的类标签没登记时只按 16 KiB 比（叶容器类标签改成 9、按 32 KiB 补齐校验和的那张镜像上 I-2.1 误红）	crates/singlefs-checker/src/walk.rs	        None => vec![node_bytes(), data_unit_bytes()],	        None => vec![node_bytes()],	-p singlefs-harness --test checker_known_bad_images	an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_only_the_inode_entry_invariant
第29批 H-J3a：走读不往下读的孩子两份都对不上不记走读失败（类型段 7、两条位置条目校验和都坏的 inode 树根条目上 I-7.2 不红）	crates/singlefs-checker/src/walk.rs	        if !some_copy_matches {	        if false && !some_copy_matches {	-p singlefs-harness --test checker_known_bad_images	an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants
第29批 H-J3a：墓碑树、谱系树类型段不认得那一支不判孩子的副本（墓碑树根条目类型段 7、两条位置条目校验和都坏，I-2.1 报成立）	crates/singlefs-checker/src/walk.rs	                    self.judge_every_copy_of_a_child_the_walk_does_not_go_into(&child, &what);\n                    self.judgements.judge(identity_invariant, false, || {	                    self.judgements.judge(identity_invariant, false, || {	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong	a_tombstone_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants
第29批 H-J3b：墓碑树、谱系树孩子是另一类那一支不判孩子的副本（墓碑树根条目类型段 0、第一条位置条目校验和坏，I-2.1 报成立）	crates/singlefs-checker/src/walk.rs	        self.judge_every_copy_of_a_child_the_walk_does_not_go_into(child, &what);\n        self.judgements.judge(identity_invariant, false, || {	        self.judgements.judge(identity_invariant, false, || {	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_tree_entry_of_type_zero_over_a_container_whose_first_location_checksum_is_wrong	a_tombstone_tree_entry_of_type_zero_over_a_container_whose_first_location_checksum_is_wrong_reddens_the_checksum_invariant
第29批 H-J3b：inode 树孩子是另一类那一支不判孩子的副本（inode 树根条目类型段 0、第一条位置条目校验和坏，I-2.1 报成立）	crates/singlefs-checker/src/walk.rs	                    "inode 树的孩子",\n                );\n                self.judge_every_copy_of_a_child_the_walk_does_not_go_into(\n                    &child,\n                    "inode 树的孩子",\n                );\n                self.judgements.judge("I-9.2", false, || {	                    "inode 树的孩子",\n                );\n                self.judgements.judge("I-9.2", false, || {	-p singlefs-harness --test checker_known_bad_images -- an_inode_tree_entry_of_type_zero_over_a_leaf_container_whose_first_location_checksum_is_wrong	an_inode_tree_entry_of_type_zero_over_a_leaf_container_whose_first_location_checksum_is_wrong_reddens_the_checksum_invariant
第29批 H-J3a：inode 树类型段不认得那一支不判孩子的副本（第 27 批那样两样都不记，类型段 7、两条位置条目校验和都坏只红 I-9.2）	crates/singlefs-checker/src/walk.rs	                        "inode 树的孩子",\n                    );\n                    self.judge_every_copy_of_a_child_the_walk_does_not_go_into(\n                        &child,\n                        "inode 树的孩子",\n                    );\n                    self.judgements.judge("I-9.2", false, || {	                        "inode 树的孩子",\n                    );\n                    self.judgements.judge("I-9.2", false, || {	-p singlefs-harness --test checker_known_bad_images -- an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong	an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants
第29批 O-1：I-3.12 放过 T 等于根的 txg 的行（被抛弃的根自己那张 257 片实例表那条对拍流在 CPU 一侧钉不住 I-3.12 红）	crates/singlefs-checker/src/walk.rs	                row.published_checkpoint_txg < root.checkpoint_txg,	                row.published_checkpoint_txg <= root.checkpoint_txg,	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_leaves_to_the_cpu_an_instance_table_chain_longer_than_the_pages_it_walks_on_a_root_outside_the_candidates	the_gpu_judge_leaves_to_the_cpu_an_instance_table_chain_longer_than_the_pages_it_walks_on_a_root_outside_the_candidates
第29批 GPU 判器 H-J3a：inode 树类型段不认得那一支不判孩子的副本（类型段 7、两条位置条目校验和都坏那条流上 CPU 红 I-2.1、I-7.2，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            w_note_pointer_references(child, child_span_by_the_level);\n            w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);	            w_note_pointer_references(child, child_span_by_the_level);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong	the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong
第29批 GPU 判器 H-J3b：inode 树孩子是另一类那一支不判孩子的副本（类型段 0、第一条位置条目校验和坏那条流上 CPU 红 I-2.1，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_inode_child));\n            w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);	            w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_inode_child));	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container_whose_first_location_checksum_is_wrong	the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container_whose_first_location_checksum_is_wrong
第29批 GPU 判器 H-J3a：走读不往下读的孩子两份都对不上不记走读失败（类型段 7、两条位置条目校验和都坏那条流上 CPU 红 I-7.2，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    if (!first_matches && !second_matches) {	    if (false && !first_matches && !second_matches) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong	the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong
第29批 GPU 判器 O-1：扫描内核的实例表链上限与走读内核的不同（扫描内核读满 255 片就记判不了）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 256u;	const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 255u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_walk_kernel_and_the_scan_kernel_follow_instance_table_chains_up_to_the_same_number_of_pages	the_walk_kernel_and_the_scan_kernel_follow_instance_table_chains_up_to_the_same_number_of_pages
第29批 GPU 判器 K1 J2：实例表链某一片读不出也当走到末片（最新根实例表第 1 片两份都读不出那条关门流上门开了，GPU 红 I-9.29、CPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        let s = w_read_referenced_unit(pointer, DATA_UNIT_BYTES);\n        if (s.version == NONE) {\n            w_failures = w_failures + 1u;\n            break;\n        }	        let s = w_read_referenced_unit(pointer, DATA_UNIT_BYTES);\n        if (s.version == NONE) {\n            w_failures = w_failures + 1u;\n            if (judge_rows) {\n                w_newest_instance_table_chain_reached_its_last_page = true;\n            }\n            break;\n        }	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_second_page_of_the_newest_instance_table_does_not_read_back	the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_second_page_of_the_newest_instance_table_does_not_read_back
````
