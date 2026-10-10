# 第 19 批代码轮被判的改动全文（2026-10-04）

19-1 的补丁（过滤掉 `crash_judge_gpu.rs` 那一段之后、实际打进主仓的那一份）与它的变异追加，19-2 的补丁原样；两份都已打进主仓，合入验证全绿。

## `/tmp/claude-1000/m3-formal-19-1/patch-filtered/crates.patch`

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index 2efcda3..466bcfc 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -1771,10 +1771,81 @@ fn judge_instance_table_rows_against_the_roots_referencing_them() {
 }
 
 // 第三批的入口
+// I-5.5 那一判用的两个量（walk.rs 的 CLUSTER_SEGMENT_SLOTS 取自 singlefs_format、DATA_UNIT_SLOTS）：聚簇段 64 槽，一个数据单元占 2 槽
+const FIRST_DATA_UNIT_CLUSTER_SEGMENT_SLOTS: u32 = 64u;
+const FIRST_DATA_UNIT_DATA_UNIT_SLOTS: u32 = 2u;
+
+// C_ALLOC_RECORDS 里装着这一版的分配记录时，device 上的槽 slot 算不算被占：被「已释放、或分配代早于这一版的 txg」的记录罩着
+// （walk.rs first_data_unit_slot_the_allocation_records_leave_on_a_device 的 occupied_slots；这一版自己分配的记录当空）
+fn c_slot_is_occupied_before_the_version(device: u32, slot: vec2<u32>, txg: vec2<u32>) -> bool {
+    for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
+        let a = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
+        if (sw(a) != device) {
+            continue;
+        }
+        let generation = vec2<u32>(sw(a + 4u), sw(a + 5u));
+        if (sw(a + 6u) == 0u && !lt64(generation, txg)) {
+            continue;
+        }
+        let start = vec2<u32>(sw(a + 1u), sw(a + 2u));
+        if (le64(start, slot) && lt64(slot, add64(start, u64_of(sw(a + 3u))))) {
+            return true;
+        }
+    }
+    return false;
+}
+
+// 起点 candidate 的那一对合不合 I-5.5 的落点：不在开放段 [open_first, open_first + 64) 里（has_open 为假时不排除），两槽都不被占
+fn c_first_data_unit_pair_is_free(device: u32, candidate: vec2<u32>, txg: vec2<u32>, has_open: bool, open_first: vec2<u32>) -> bool {
+    if (has_open && le64(open_first, candidate)
+        && lt64(candidate, add64(open_first, u64_of(FIRST_DATA_UNIT_CLUSTER_SEGMENT_SLOTS)))) {
+        return false;
+    }
+    return !c_slot_is_occupied_before_the_version(device, candidate, txg)
+        && !c_slot_is_occupied_before_the_version(device, add64(candidate, u64_of(1u)), txg);
+}
+
+// I-5.5 的判据（walk.rs first_data_unit_slot_the_allocation_records_leave_on_a_device，近似在那里写明）：从 U 起，device 上起点最小、
+// 起点是 2 的整数倍、两槽都空、不在开放段里的那一对的起点。walk.rs 从 U 起一对一对往后扫；这里不扫，只试几个候选取合格里最小的：
+// 最小的合格起点要么是 U，要么它前面那一对不合格——那一对里有被占的槽（罩住它的那条记录的末端向上取到 2 的整数倍就是它），
+// 或在开放段里（开放段的末端就是它）。比每个被占的槽、比开放段都靠后的那个候选总合格，所以一定找得到，与 walk.rs 交回同一个起点
+fn c_first_data_unit_slot_the_allocation_records_leave_on_a_device(device: u32, txg: vec2<u32>, has_open: bool, open_first: vec2<u32>) -> vec2<u32> {
+    let unit_area_start = handoff_unit_area_start_slot();
+    var expected = unit_area_start;
+    var found = c_first_data_unit_pair_is_free(device, unit_area_start, txg, has_open, open_first);
+    if (has_open) {
+        let past_the_open_segment = add64(open_first, u64_of(FIRST_DATA_UNIT_CLUSTER_SEGMENT_SLOTS));
+        if (le64(unit_area_start, past_the_open_segment) && (!found || lt64(past_the_open_segment, expected))
+            && c_first_data_unit_pair_is_free(device, past_the_open_segment, txg, has_open, open_first)) {
+            expected = past_the_open_segment;
+            found = true;
+        }
+    }
+    for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
+        let a = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
+        let generation = vec2<u32>(sw(a + 4u), sw(a + 5u));
+        if (sw(a) != device || (sw(a + 6u) == 0u && !lt64(generation, txg))) {
+            continue;
+        }
+        var past_the_record = add64(vec2<u32>(sw(a + 1u), sw(a + 2u)), u64_of(sw(a + 3u)));
+        if ((past_the_record.x % FIRST_DATA_UNIT_DATA_UNIT_SLOTS) != 0u) {
+            past_the_record = add64(past_the_record, u64_of(FIRST_DATA_UNIT_DATA_UNIT_SLOTS - past_the_record.x % FIRST_DATA_UNIT_DATA_UNIT_SLOTS));
+        }
+        if (le64(unit_area_start, past_the_record) && (!found || lt64(past_the_record, expected))
+            && c_first_data_unit_pair_is_free(device, past_the_record, txg, has_open, open_first)) {
+            expected = past_the_record;
+            found = true;
+        }
+    }
+    return expected;
+}
+
 // I-5.5（walk.rs judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark）的一版：树表指针在 (base, offset)、它的 txg。
-// 认法 ① 树表非空、种类 1 条目恰好一条且树 ID 是 mkfs 水位 65536；② 每条诞生 txg（条目偏移 108）都等于这一版的 txg。认出来、extent 树读得全时，
-// 盘表里每块有它数据单元的盘上最小落点要等于 U + 4（U 取交接段里所选系统配置的单元区起始槽号）。CPU 记成射程里判不了的那几格
-// （树表读不出、条目宽不到 200、种类 1 不止一条、extent 树读不全、某块盘上没有数据单元）在这里一格都不置位
+// 认法 ① 树表非空、种类 1 条目恰好一条且树 ID 是 mkfs 水位 65536；② 每条诞生 txg（条目偏移 108）都等于这一版的 txg。认出来、extent 树读得全、
+// 种类 3 条目恰好一条且那棵分配记录树读得全时，盘表里每块有它数据单元的盘上最小落点要等于这一版的分配记录在那块盘上算出的落点
+// （c_first_data_unit_slot_the_allocation_records_leave_on_a_device；U 取交接段里所选系统配置的单元区起始槽号，开放段取这一版树表单元
+// 在那块盘上那一份所在的段）。CPU 记成射程里判不了的那几格（树表读不出、条目宽不到 200、种类 1 不止一条、extent 树读不全、
+// 某块盘上没有数据单元、种类 3 不是恰好一条、分配记录树读不全）在这里一格都不置位
 fn judge_first_data_unit_of_a_version(base: u32, tree_table_pointer_offset: u32, txg: vec2<u32>) {
     let tree_table_pointer = parse_node_pointer(base, tree_table_pointer_offset);
     if (tree_table_pointer.all_zero) {
@@ -1786,6 +1857,8 @@ fn judge_first_data_unit_of_a_version(base: u32, tree_table_pointer_offset: u32,
     }
     var extent_entry_count = 0u;
     var extent_entry = 0u;
+    var allocation_entry_count = 0u;
+    var allocation_entry = 0u;
     var every_birth_is_this_version = true;
     for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
         let entry = entry_offset(tt, entry_index);
@@ -1793,6 +1866,10 @@ fn judge_first_data_unit_of_a_version(base: u32, tree_table_pointer_offset: u32,
             extent_entry_count = extent_entry_count + 1u;
             extent_entry = entry;
         }
+        if (rd16(tt.base, entry + 10u) == TREE_KIND_ALLOCATION) {
+            allocation_entry_count = allocation_entry_count + 1u;
+            allocation_entry = entry;
+        }
         if (!eq64(rd64(tt.base, entry + 108u), txg)) {
             every_birth_is_this_version = false;
         }
@@ -1814,10 +1891,37 @@ fn judge_first_data_unit_of_a_version(base: u32, tree_table_pointer_offset: u32,
     if (!read_the_whole_extent_tree) {
         return;
     }
-    let expected_slot = add64(handoff_unit_area_start_slot(), u64_of(FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START));
     let device_count = hdr(3u);
+    var has_any_data_unit = false;
     for (var device_index = 0u; device_index < device_count && device_index < MAX_DEVICES; device_index = device_index + 1u) {
-        if (r_has_data_unit_on_device[device_index] == 1u && !eq64(r_smallest_data_unit_slot_on_device[device_index], expected_slot)) {
+        if (r_has_data_unit_on_device[device_index] == 1u) {
+            has_any_data_unit = true;
+        }
+    }
+    if (!has_any_data_unit || allocation_entry_count != 1u) {
+        return;
+    }
+    c_alloc_record_count = 0u;
+    if (!r_allocation_tree(tt.base, allocation_entry + 14u)) {
+        return;
+    }
+    for (var device_index = 0u; device_index < device_count && device_index < MAX_DEVICES; device_index = device_index + 1u) {
+        if (r_has_data_unit_on_device[device_index] == 0u) {
+            continue;
+        }
+        let device = hdr(12u + device_index);
+        // 开放段：这一版树表单元在这块盘上那一份所在的 64 槽段（walk.rs open_cluster_segment_named_by_the_tree_table_on_a_device，先看第一条位置条目）
+        var has_open = false;
+        var open_first = vec2<u32>(0u, 0u);
+        if (tree_table_pointer.device0 == device) {
+            has_open = true;
+            open_first = vec2<u32>(tree_table_pointer.slot0.x & ~(FIRST_DATA_UNIT_CLUSTER_SEGMENT_SLOTS - 1u), tree_table_pointer.slot0.y);
+        } else if (tree_table_pointer.device1 == device) {
+            has_open = true;
+            open_first = vec2<u32>(tree_table_pointer.slot1.x & ~(FIRST_DATA_UNIT_CLUSTER_SEGMENT_SLOTS - 1u), tree_table_pointer.slot1.y);
+        }
+        let expected_slot = c_first_data_unit_slot_the_allocation_records_leave_on_a_device(device, txg, has_open, open_first);
+        if (!eq64(r_smallest_data_unit_slot_on_device[device_index], expected_slot)) {
             violate(INVARIANT_I_5_5);
         }
     }
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 64622ad..edb9fca 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -1185,6 +1185,81 @@ fn record_new_pool_with_the_head_identifier_of_the_extent_tree_entry_written_zer
     )
 }
 
+/// I-9.27 种类 2 那一支：发布那张树表里 inode 树那条（种类 2）的头 ID 从它自己的树 ID 改成树 ID + 1、重封到根。
+fn record_new_pool_with_the_head_identifier_of_the_inode_tree_entry_off_its_own_tree_identifier(
+    tag: &str,
+) -> RecordedFlow {
+    let tree_table = |bytes: &[u8]| {
+        unit_class(bytes) == 2
+            && index_node_key_width(bytes) == 8
+            && index_node_entry_width(bytes) == 200
+    };
+    record_new_pool_with_a_resealed_field(
+        tag,
+        "new-pool-inode-tree-entry-head-identifier-off-its-own-tree-identifier",
+        &tree_table,
+        false,
+        &|bytes| {
+            let entry = only_tree_table_entry_of_kind(bytes, 2);
+            let own_tree_identifier =
+                u64::from_le_bytes(bytes[entry..entry + 8].try_into().expect("8"));
+            assert_eq!(
+                u64::from_le_bytes(bytes[entry + 116..entry + 124].try_into().expect("8")),
+                own_tree_identifier,
+                "inode 树那条的头 ID 本来等于它自己的树 ID"
+            );
+            bytes[entry + 116..entry + 124]
+                .copy_from_slice(&(own_tree_identifier + 1).to_le_bytes());
+        },
+    )
+}
+
+/// I-9.27 写 0 那一支：发布那张树表里分配记录树那条（种类 3）的头 ID 从 0 改成 inode 树条目的树 ID、重封到根。
+fn record_new_pool_with_a_non_zero_head_identifier_on_the_allocation_record_tree_entry(
+    tag: &str,
+) -> RecordedFlow {
+    let tree_table = |bytes: &[u8]| {
+        unit_class(bytes) == 2
+            && index_node_key_width(bytes) == 8
+            && index_node_entry_width(bytes) == 200
+    };
+    record_new_pool_with_a_resealed_field(
+        tag,
+        "new-pool-allocation-record-tree-entry-head-identifier-non-zero",
+        &tree_table,
+        false,
+        &|bytes| {
+            let entry = only_tree_table_entry_of_kind(bytes, 3);
+            let inode_tree_entry = only_tree_table_entry_of_kind(bytes, 2);
+            assert_eq!(
+                bytes[entry + 116..entry + 124],
+                [0u8; 8],
+                "分配记录树那条的头 ID 本来是 0"
+            );
+            let inode_tree_identifier = bytes[inode_tree_entry..inode_tree_entry + 8].to_vec();
+            bytes[entry + 116..entry + 124].copy_from_slice(&inode_tree_identifier);
+        },
+    )
+}
+
+/// 一张树表单元（码 2、条目宽 200）里种类是 `kind` 的那一条（恰好一条）在单元里的偏移。
+fn only_tree_table_entry_of_kind(bytes: &[u8], kind: u16) -> usize {
+    let header_end = 86 + 2 * usize::from(bytes[51]);
+    let count = usize::from(u16::from_le_bytes(
+        bytes[header_end - 4..header_end - 2].try_into().expect("2"),
+    ));
+    let entries_of_the_kind: Vec<usize> = (0..count)
+        .map(|entry| 131 + entry * 200)
+        .filter(|offset| {
+            u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().expect("2")) == kind
+        })
+        .collect();
+    let [entry] = entries_of_the_kind.as_slice() else {
+        panic!("树表里种类 {kind} 的条目恰好一条：{entries_of_the_kind:?}")
+    };
+    *entry
+}
+
 /// I-5.5（从 mkfs 水位建树那一版的 t1 落点）：新池新建文件那一版唯一的数据单元（两份副本，单元区起始槽号 + 4）挪到再往后两槽
 /// （下一个 32 KiB 对齐的槽对，这条流上空着）——两次单元写的偏移、流里每一处指着它的位置条目（extent 上段叶内联的数据指针、
 /// 中央映射叶、journal 记录）的槽号一起改，单元重封到根、记录重封。树表与诞生 txg 一个字节不动，这一版照样被认出，
@@ -2639,6 +2714,28 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_extent_tree_entry_carries_a_head_i
     );
 }
 
+/// I-9.27 种类 2 那一支：inode 树那条树表条目的头 ID 不等于它自己的树 ID。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_inode_tree_entry_carries_a_head_identifier_other_than_its_own_tree_identifier(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_inode_tree_entry_carries_a_head_identifier_other_than_its_own_tree_identifier",
+        &record_new_pool_with_the_head_identifier_of_the_inode_tree_entry_off_its_own_tree_identifier("crash-judge-gpu-i927-inode"),
+        "I-9.27",
+    );
+}
+
+/// I-9.27 写 0 那一支：分配记录树那条树表条目的头 ID 不是 0。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_allocation_record_tree_entry_carries_a_non_zero_head_identifier(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_allocation_record_tree_entry_carries_a_non_zero_head_identifier",
+        &record_new_pool_with_a_non_zero_head_identifier_on_the_allocation_record_tree_entry("crash-judge-gpu-i927-allocation"),
+        "I-9.27",
+    );
+}
+
 /// I-5.5：从 mkfs 水位建树的那一版，数据单元不落在单元区起始槽号 + 4。
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_when_the_first_tree_building_version_places_its_data_unit_past_the_unit_area_start_plus_four(
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 5f9cae2..c398223 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -8,11 +8,11 @@ use std::collections::{BTreeMap, BTreeSet};
 
 use singlefs_format::{
     ACCOUNTING_ENTRY_BYTES, ALLOCATION_RECORD_BYTES, ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES,
-    DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES, EXTENT_TREE_INTERNAL_ENTRY_BYTES,
-    EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES, INODE_INTERNAL_ENTRY, JOURNAL_RECORD_BYTES,
-    JOURNAL_SAFETY_FACTOR, MAPPING_ENTRY_BYTES, NODE_BYTES, NODE_POINTER_BYTES,
-    NONCE_MAC_ALGORITHM_RESERVED_BYTES, SLOT_BYTES, TREE_IDENTIFIER_WATERMARK_AT_MKFS,
-    TREE_TABLE_ENTRY_BYTES,
+    CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES,
+    EXTENT_TREE_INTERNAL_ENTRY_BYTES, EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES, INODE_INTERNAL_ENTRY,
+    JOURNAL_RECORD_BYTES, JOURNAL_SAFETY_FACTOR, MAPPING_ENTRY_BYTES, NODE_BYTES,
+    NODE_POINTER_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES, SLOT_BYTES,
+    TREE_IDENTIFIER_WATERMARK_AT_MKFS, TREE_TABLE_ENTRY_BYTES,
 };
 
 use crate::position_addressed::{
@@ -3370,9 +3370,8 @@ const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 8 + 2;
 /// 树表条目里头 ID 8 字节的偏移（树 ID 8 + 条目长度 2 + 种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8 + 诞生 txg 8，
 /// D8（核心索引结构） 已定项 8 的字段表），I-9.27 判它。
 const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8 + 8;
-/// 从 mkfs 水位第一次建树的那一版，每块盘上落点最小的数据单元比单元区起始槽号 U 多几槽（D3（空间分配） 已定项 8 射程：
-/// mkfs 的实例表占 U、U + 1，树表占 U + 2，第一个 32768 对齐的空槽对从 U + 4 起；环长不是默认值时 U 现算、仍是 U + 4），I-5.5 判它。
-const FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START: u64 = 4;
+/// 一个数据单元占几个 16 KiB 槽（32768 字节，起点 32768 对齐 ⇒ 起点槽号是它的整数倍）：I-5.5 按槽对找落点。
+const DATA_UNIT_SLOTS: u64 = DATA_UNIT_BYTES / SLOT_BYTES;
 
 /// I-9.27：树表条目的头 ID 按种类该等于什么（`.claude/kb/invariants.md` I-9.27 那一行；D8（核心索引结构） 已定项 8「头 ID 8 字节的取值」
 /// 句末的关系判据：判的是关系，不拿字面 65537 比）。
@@ -4125,13 +4124,17 @@ struct VersionToJudgeTheFirstDataUnitOf<'record> {
 /// I-5.5（从 mkfs 水位建树那一版的 t1 落点）：走读到的每一版（回退候选集里每条根，含由记录施加出来的那几版）各自重读一遍树表，
 /// ① 树表非空，且种类 1（extent 树）条目的树 ID 等于 mkfs 种下的水位 65536；② 树表里每条条目的诞生 txg（条目偏移 108）都等于这一版的
 /// `checkpoint_txg`——两样都成立就是从 mkfs 水位第一次建树的那一版：它的 extent 树引用的数据单元里，每块盘上落点槽号最小的那一个
-/// 要等于 U + 4（U 是所选系统配置偏移 417 的单元区起始槽号）。这一版没有数据单元时不判，不满足 ①② 的版本不判。只读、不判别的不变量：
-/// 主走读按 `visited_units` 只走一遍共享子树，共享的 extent 树只算进第一条走到它的版本，这一条要的是「这一版自己引用了哪些数据单元」
-/// （与 [`references_of_root`] 同一个缘故）。
+/// 要等于这一版的分配记录在那块盘上算出的落点（[`first_data_unit_slot_the_allocation_records_leave_on_a_device`]：从单元区起始槽号 U 起，
+/// 起点最小、32768 对齐、两槽都空、不在开放段里的那一对；U 是所选系统配置偏移 417）。不写死 U + 4：mkfs 的实例表（U、U + 1）被 COW 走、
+/// 过了回收门槛之后，实现照 D3（空间分配） 已定项 8 第 1 条把数据放在 U（用户 2026-10-04 定 I-5.5 按分配记录现算，判决
+/// `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K2）。这一版没有数据单元时不判，不满足 ①② 的版本不判。
+/// 只读、不判别的不变量：主走读按 `visited_units` 只走一遍共享子树，共享的 extent 树只算进第一条走到它的版本，这一条要的是
+/// 「这一版自己引用了哪些数据单元」（与 [`references_of_root`] 同一个缘故）。
 ///
 /// 下面几格 `.claude/kb/invariants.md` I-5.5 那一行没写，判不了，记成射程里判不了的那一部分（没有违例时整条报不适用、带理由），不替条款定：
 /// 树表读不出、条目宽不到字段表的 200 字节（认不出这一版是不是这一类）；种类 1 条目不止一条（① 读哪一条）；
-/// 认出来的那一版 extent 树读不全（数据单元数不全）；它有数据单元、而池里某块盘上一个都没有（那块盘上「最小的那一个」没有对象）。
+/// 认出来的那一版 extent 树读不全（数据单元数不全）；它有数据单元、而池里某块盘上一个都没有（那块盘上「最小的那一个」没有对象）；
+/// 种类 3（分配记录树）条目不是恰好一条、或那棵树读不全（落点算不出）。
 fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
     reader: &dyn ImageReader,
     versions: &[VersionToJudgeTheFirstDataUnitOf<'_>],
@@ -4139,7 +4142,6 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
     judgements: &mut Judgements,
 ) {
     let invariant = FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK;
-    let expected_slot = unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START;
     let devices = reader.devices();
     let mut cache = IndexNodeCache::new();
     for version in versions {
@@ -4221,12 +4223,45 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
         if smallest_data_unit_slot_on_each_device.is_empty() {
             continue;
         }
+        let allocation_entries: Vec<&Vec<u8>> = tree_table
+            .entries
+            .iter()
+            .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_ALLOCATION)
+            .collect();
+        let [allocation_entry] = allocation_entries.as_slice() else {
+            judgements.part_of_the_range_not_judged(
+                invariant,
+                "从 mkfs 水位第一次建树的那一版树表里种类 3（分配记录树）条目不是恰好一条：落点按哪一棵分配记录树算，I-5.5 那一行没写",
+            );
+            continue;
+        };
+        let Some((_, allocation_records)) = allocation_record_tree_without_judging(
+            reader,
+            &parse_node_pointer(&allocation_entry[14..100]),
+            &mut cache,
+        ) else {
+            judgements.part_of_the_range_not_judged(
+                invariant,
+                "从 mkfs 水位第一次建树的那一版分配记录树读不全：落点算不出",
+            );
+            continue;
+        };
         for device in &devices {
             match smallest_data_unit_slot_on_each_device.get(device) {
                 Some(&smallest_slot) => {
+                    let expected_slot = first_data_unit_slot_the_allocation_records_leave_on_a_device(
+                        &allocation_records,
+                        *device,
+                        version.checkpoint_txg,
+                        unit_area_start_slot,
+                        open_cluster_segment_named_by_the_tree_table_on_a_device(
+                            &tree_table_pointer,
+                            *device,
+                        ),
+                    );
                     judgements.judge(invariant, smallest_slot == expected_slot, || {
                         format!(
-                            "{}（从 mkfs 水位第一次建树的那一版）：盘 {device} 上落点最小的数据单元在槽 {smallest_slot}，不是单元区起始槽号 {unit_area_start_slot} + {FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START} = {expected_slot}",
+                            "{}（从 mkfs 水位第一次建树的那一版）：盘 {device} 上落点最小的数据单元在槽 {smallest_slot}，不是这一版的分配记录从单元区起始槽号 {unit_area_start_slot} 起算出的落点 {expected_slot}",
                             version.name
                         )
                     });
@@ -4240,6 +4275,77 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
     }
 }
 
+/// 一块盘上的开放段：起点是 64 的整数倍的 [`CLUSTER_SEGMENT_SLOTS`] 个槽（D3（空间分配） 已定项 10 ①）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct OpenClusterSegment {
+    first_slot: u64,
+}
+
+impl OpenClusterSegment {
+    fn holds_slot(self, slot: u64) -> bool {
+        self.first_slot <= slot && slot < self.first_slot + CLUSTER_SEGMENT_SLOTS
+    }
+}
+
+/// I-5.5 拿哪一段当这一版在这块盘上的开放段：这一版树表单元在这块盘上那一份所在的段（树表单元最末从开放段 bump，
+/// D3（空间分配） 已定项 10 ⑤）。树表指针在这块盘上没有位置条目时交回 `None`（这块盘上不排除哪一段）。
+fn open_cluster_segment_named_by_the_tree_table_on_a_device(
+    tree_table_pointer: &PointerView,
+    device: u32,
+) -> Option<OpenClusterSegment> {
+    tree_table_pointer
+        .locations
+        .iter()
+        .find(|location| location.device == device)
+        .map(|location| OpenClusterSegment {
+            first_slot: location.slot - location.slot % CLUSTER_SEGMENT_SLOTS,
+        })
+}
+
+/// I-5.5 的判据（用户 2026-10-04 定：按这一版的分配记录现算，D3（空间分配） 已定项 8 第 1 条、已定项 10 ② ③）：
+/// 从单元区起始槽号 U 起，这一版在 `device` 上起点最小、起点 32768 对齐（槽号是 [`DATA_UNIT_SLOTS`] 的整数倍；U 是 64 的整数倍，
+/// 读系统配置时判过）、两槽都空、不在开放段里的那一对槽的起点。「空」= 没有被「已释放、或分配代早于这一版」的分配记录罩着。
+/// 不看盘尾：比每一个被占的槽都靠后的那一对总是空的，交回的落点越过盘尾时与任何真落点都不等，照样判红。
+/// 迭代上界：跳过开放段至多一次（候选只增不减），其余每一轮跳过的那一对里至少有一个被占的槽，合计不超过被占槽数 + 1 轮。
+///
+/// 下面几样 checker 在这一版的盘面上复算不出，只能近似（攻方报告 `research/prompts/m3-formal-code-b17-r1-opus-output.md`
+/// 「K2-1、K2-2 打中之后的四句」第 2 句与「改法表」），都照实写进 `.claude/kb/invariants.md` I-5.5 那一行：
+/// - 回收门槛：已释放的记录一律当占着。实现按「释放代 ≤ max(F_生效, 环里最旧有效根)」回收（D3（空间分配） 已定项 7 清扫准入），
+///   这一版根里的 F、txg 与根环槽数盘上有，被抛弃的根与挂载那一刻读不出的槽盘上没有，门槛复算不出；记录在被重新分配之前一直写着已释放。
+///   ⇒ 写死从 U + 4 起扫的坏实现（攻方 H1）在「U、U + 1 已回收、还没被重新分配」的那几版上判成立，抓不到。
+/// - 这一版自己分配的记录（未释放、分配代等于这一版的 txg）当空：它们就是这一版取落点时还空着的槽（数据单元与提交内生块都在里面）。
+/// - 开放段盘上没有字段：拿这一版树表单元所在的段当它（[`open_cluster_segment_named_by_the_tree_table_on_a_device`]）；
+///   这次挂载更早开过、已经用尽的段认不出（段用尽才换段，用尽的段里剩不下起点对齐的空槽对）。
+/// - 分配准入合取（D3（空间分配） 已定项 7）里隔离位（影子账）、抬 F 扣住位、保护期内的 R 盘上都没有：没有分配记录罩着的槽一律当空。
+fn first_data_unit_slot_the_allocation_records_leave_on_a_device(
+    allocation_records: &[AllocationRecordView],
+    device: u32,
+    version_checkpoint_txg: u64,
+    unit_area_start_slot: u64,
+    open_cluster_segment: Option<OpenClusterSegment>,
+) -> u64 {
+    let occupied_slots: BTreeSet<u64> = allocation_records
+        .iter()
+        .filter(|record| {
+            record.device == device
+                && (record.is_released || record.generation < version_checkpoint_txg)
+        })
+        .flat_map(|record| record.slot..record.slot.saturating_add(record.span_slots))
+        .collect();
+    let mut candidate = unit_area_start_slot;
+    loop {
+        if let Some(segment) = open_cluster_segment.filter(|segment| segment.holds_slot(candidate))
+        {
+            candidate = segment.first_slot + CLUSTER_SEGMENT_SLOTS;
+            continue;
+        }
+        if (candidate..candidate + DATA_UNIT_SLOTS).all(|slot| !occupied_slots.contains(&slot)) {
+            return candidate;
+        }
+        candidate += DATA_UNIT_SLOTS;
+    }
+}
+
 /// 候选集里的一条根：它在 `roots` 里的下标、它的 checkpoint_txg 与这一遍取到的引用集合。
 struct ScannedCandidateRoot {
     root_index: usize,
@@ -8888,4 +8994,142 @@ mod tests {
             "盘面数、判过的（盘面、读不出的位置）数、原式并进了槽的数、双故障形下两式不同的数"
         );
     }
+
+    /// 默认 768 MiB 环的单元区起始槽号（I-5.5 那几条合成用例的 U）。
+    const UNIT_AREA_START_SLOT_OF_THE_DEFAULT_JOURNAL_RING: u64 = 50176;
+
+    /// 盘 0 上一条分配记录。
+    fn allocation_record_on_device_zero(
+        slot: u64,
+        span_slots: u64,
+        generation: u64,
+        is_released: bool,
+    ) -> AllocationRecordView {
+        AllocationRecordView {
+            device: 0,
+            slot,
+            span_slots,
+            generation,
+            is_released,
+        }
+    }
+
+    /// I-5.5 的落点不进开放段（D3（空间分配） 已定项 10 ②）：U 起的那一段 64 槽全被更早的版本占着，下一段是这一版的开放段
+    /// （树表单元在里面），落点越过开放段、落在再下一段的起点。
+    #[test]
+    fn the_first_data_unit_slot_skips_the_open_cluster_segment_of_the_version() {
+        let unit_area_start = UNIT_AREA_START_SLOT_OF_THE_DEFAULT_JOURNAL_RING;
+        let version_checkpoint_txg = 7;
+        let allocation_records = [
+            allocation_record_on_device_zero(unit_area_start, CLUSTER_SEGMENT_SLOTS, 5, false),
+            allocation_record_on_device_zero(
+                unit_area_start + CLUSTER_SEGMENT_SLOTS + 12,
+                1,
+                version_checkpoint_txg,
+                false,
+            ),
+        ];
+        assert_eq!(
+            first_data_unit_slot_the_allocation_records_leave_on_a_device(
+                &allocation_records,
+                0,
+                version_checkpoint_txg,
+                unit_area_start,
+                Some(OpenClusterSegment {
+                    first_slot: unit_area_start + CLUSTER_SEGMENT_SLOTS,
+                }),
+            ),
+            unit_area_start + 2 * CLUSTER_SEGMENT_SLOTS,
+            "U 那一段占满、下一段是开放段：落点是再下一段的起点"
+        );
+    }
+
+    /// 已释放的记录一律当占着（回收门槛复算不出的那个近似）：U、U + 1 与 U + 2 两条记录是这一版自己释放的（释放代等于这一版的 txg），
+    /// 落点不取 U，取 U + 4。
+    #[test]
+    fn a_released_allocation_record_keeps_its_slots_out_of_the_first_data_unit_slot() {
+        let unit_area_start = UNIT_AREA_START_SLOT_OF_THE_DEFAULT_JOURNAL_RING;
+        let version_checkpoint_txg = 29;
+        let allocation_records = [
+            allocation_record_on_device_zero(unit_area_start, 2, version_checkpoint_txg, true),
+            allocation_record_on_device_zero(unit_area_start + 2, 1, version_checkpoint_txg, true),
+        ];
+        assert_eq!(
+            first_data_unit_slot_the_allocation_records_leave_on_a_device(
+                &allocation_records,
+                0,
+                version_checkpoint_txg,
+                unit_area_start,
+                None,
+            ),
+            unit_area_start + 4,
+            "U、U + 1 与 U + 2 已释放（当占着）：落点是 U + 4"
+        );
+    }
+
+    /// 一条字段表宽（200 字节）的树表条目，只填 I-9.27 读的三样：树 ID（偏移 0）、种类（偏移 10）、头 ID（偏移 116）。
+    fn tree_table_entry_with_head_identifier(
+        tree: u64,
+        kind: u16,
+        head_identifier: u64,
+    ) -> Vec<u8> {
+        let mut entry = vec![0u8; tree_table_entry_bytes()];
+        entry[..8].copy_from_slice(&tree.to_le_bytes());
+        entry[TREE_TABLE_ENTRY_KIND_OFFSET..TREE_TABLE_ENTRY_KIND_OFFSET + 2]
+            .copy_from_slice(&kind.to_le_bytes());
+        entry[TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET..TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8]
+            .copy_from_slice(&head_identifier.to_le_bytes());
+        entry
+    }
+
+    /// 一张树表上 I-9.27 的判定。
+    fn head_identifier_verdict_of(tree_table_entries: &[Vec<u8>]) -> InvariantVerdict {
+        let mut judgements = Judgements::default();
+        judge_tree_table_entry_head_identifiers(
+            &mut judgements,
+            tree_table_entries,
+            "合成的那一版",
+        );
+        judgements
+            .into_report()
+            .into_iter()
+            .find(|(invariant, _)| *invariant == HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES)
+            .expect("I-9.27 在清单里")
+            .1
+    }
+
+    /// I-9.27 多于一条种类 2 条目时放宽成「等于其中某一条的树 ID」（`.claude/kb/invariants.md` I-9.27 那一行，收严留给多头那一批）：
+    /// 表里两条 inode 树（65537、65549），extent 树条目的头 ID 写第二条的树 ID 65549——成立；只认第一条的读法在这里误红。
+    #[test]
+    fn an_extent_entry_naming_the_second_of_two_inode_tree_entries_holds_the_head_identifier_relation(
+    ) {
+        let tree_table_entries = [
+            tree_table_entry_with_head_identifier(65536, TREE_KIND_EXTENT, 65549),
+            tree_table_entry_with_head_identifier(65537, TREE_KIND_INODE, 65537),
+            tree_table_entry_with_head_identifier(65549, TREE_KIND_INODE, 65549),
+        ];
+        assert_eq!(
+            head_identifier_verdict_of(&tree_table_entries),
+            InvariantVerdict::Holds,
+            "extent 树条目的头 ID 等于两条 inode 树条目里第二条的树 ID：放宽之后成立"
+        );
+    }
+
+    /// 同一张两条 inode 树的表，extent 树条目的头 ID 写成两条都不是的 65550：放宽之后照样红。
+    #[test]
+    fn an_extent_entry_naming_neither_of_two_inode_tree_entries_violates_the_head_identifier_relation(
+    ) {
+        let tree_table_entries = [
+            tree_table_entry_with_head_identifier(65536, TREE_KIND_EXTENT, 65550),
+            tree_table_entry_with_head_identifier(65537, TREE_KIND_INODE, 65537),
+            tree_table_entry_with_head_identifier(65549, TREE_KIND_INODE, 65549),
+        ];
+        assert!(
+            matches!(
+                head_identifier_verdict_of(&tree_table_entries),
+                InvariantVerdict::Violated(_)
+            ),
+            "extent 树条目的头 ID 不等于表里任何一条 inode 树条目的树 ID：判红"
+        );
+    }
 }
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index a055e70..7a534aa 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -50,6 +50,7 @@ use singlefs_core::mount::{
     mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, InstanceRow,
     MountError, RollbackTarget, ShadowLedger, Unmounted,
 };
+use singlefs_core::records::{TREE_KIND_ALLOCATION, TREE_KIND_DELETED_INODES, TREE_KIND_TOMBSTONE};
 use singlefs_core::recovery::{
     instance_table_chain_of_root, recover, JournalPolicy, PoolReader, RecoveryFailure,
     RecoveryOutcome,
@@ -6372,9 +6373,104 @@ fn known_bad_images_of_pointer_journal_and_generation_fields() -> Vec<(&'static
                 );
             }),
         ),
+        // I-9.27 种类 2 那一支（判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md` 第二节 K1 证据缺口「种类 2 自等」）：
+        // A 的树表里 inode 树条目（种类 2）的头 ID 从它自己的树 ID 改成树 ID + 1，重封、补到根。种类 1、9、10 比的是 inode 树条目的树 ID、
+        // 不读它的头 ID，只红 I-9.27。
+        (
+            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
+            Box::new(|image: &mut MemoryPool| {
+                rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
+                    image,
+                    TREE_KIND_INODE,
+                    |own_tree_identifier, _inode_tree_identifier| own_tree_identifier,
+                    |own_tree_identifier, _inode_tree_identifier| own_tree_identifier + 1,
+                );
+            }),
+        ),
+        // I-9.27 写 0 那一支（同一节证据缺口「写 0 的七种」）：A 的树表里分配记录树条目（种类 3）的头 ID 从 0 改成 inode 树条目的树 ID。
+        (
+            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
+            Box::new(|image: &mut MemoryPool| {
+                rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
+                    image,
+                    TREE_KIND_ALLOCATION,
+                    |_own_tree_identifier, _inode_tree_identifier| 0,
+                    |_own_tree_identifier, inode_tree_identifier| inode_tree_identifier,
+                );
+            }),
+        ),
+        // I-9.27 种类 9 那一格（同一节证据缺口「种类 9 / 10」）：A 的树表里 `deleted_inodes` 条目的头 ID 从 inode 树条目的树 ID 改成 0。
+        (
+            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
+            Box::new(|image: &mut MemoryPool| {
+                rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
+                    image,
+                    TREE_KIND_DELETED_INODES,
+                    |_own_tree_identifier, inode_tree_identifier| inode_tree_identifier,
+                    |_own_tree_identifier, _inode_tree_identifier| 0,
+                );
+            }),
+        ),
+        // I-9.27 种类 10 那一格：A 的树表里墓碑树条目的头 ID 从 inode 树条目的树 ID 改成 0。
+        (
+            HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
+            Box::new(|image: &mut MemoryPool| {
+                rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
+                    image,
+                    TREE_KIND_TOMBSTONE,
+                    |_own_tree_identifier, inode_tree_identifier| inode_tree_identifier,
+                    |_own_tree_identifier, _inode_tree_identifier| 0,
+                );
+            }),
+        ),
     ]
 }
 
+/// A 的树表（50252）里种类是 `kind` 的那一条（恰好一条）：头 ID（条目偏移 116）先核等于 `head_identifier_before(那条的树 ID, inode 树条目的树 ID)`，
+/// 再改成 `head_identifier_after(…)`，重封、补到根。别的字段一个都不动。
+fn rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(
+    image: &mut MemoryPool,
+    kind: u16,
+    head_identifier_before: impl Fn(u64, u64) -> u64,
+    head_identifier_after: impl Fn(u64, u64) -> u64,
+) {
+    mutate_unit(
+        image,
+        TREE_TABLE,
+        |bytes| {
+            let entries_of_the_kind: Vec<usize> = (0..TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE)
+                .map(tree_table_entry_offset)
+                .filter(|entry| {
+                    u16::from_le_bytes([
+                        bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET],
+                        bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET + 1],
+                    ]) == kind
+                })
+                .collect();
+            let [entry] = entries_of_the_kind.as_slice() else {
+                panic!("A 的树表里种类 {kind} 的条目恰好一条：{entries_of_the_kind:?}")
+            };
+            let own_tree_identifier = get_u64(bytes, *entry);
+            let inode_tree_identifier = get_u64(
+                bytes,
+                tree_table_entry_offset(inode_tree_entry_index_in(bytes)),
+            );
+            let field = entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET;
+            assert_eq!(
+                get_u64(bytes, field),
+                head_identifier_before(own_tree_identifier, inode_tree_identifier),
+                "种类 {kind} 那条的头 ID 本来的取值"
+            );
+            set_u64(
+                bytes,
+                field,
+                head_identifier_after(own_tree_identifier, inode_tree_identifier),
+            );
+        },
+        true,
+    );
+}
+
 /// 记录新根段里实例表指针的起点：新根段 99 起，树表指针 86、映射根指针 86、树 ID 水位 8、F 8 之后（D23（journal 的角色与格式） 已定项 15）。
 const JOURNAL_RECORD_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET: usize = 99 + 188;
 
diff --git a/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs b/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
index 00e7097..8480788 100644
--- a/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
+++ b/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
@@ -4,6 +4,11 @@
 //!   实例 3 从根环水位 65548 发树（inode 树 65549）的那段合法历史上一条不红；
 //! - I-5.5（从 mkfs 水位建树那一版的 t1 落点）在新池新建文件（默认 768 MiB 环与 128 MiB 环）、只暖机过的实例 2 写第一个文件的那一版上
 //!   真被评估过且成立；第一个文件版本离开根环之后只剩覆盖写那几版（诞生 txg 不等于这一版的 txg），一版都不判。
+//! - I-5.5 的判据按这一版的分配记录现算（用户 2026-10-04 定，判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md`
+//!   第二节 K2、第五节第 1 行）：mkfs 的实例表（U、U + 1）被 COW 走、过了回收门槛之后，实现照 D3（空间分配） 已定项 8 第 1 条
+//!   把第一个文件的数据单元放在 U——只做过 mkfs 的池只暖机挂载 10 次之后写第一个文件、同一次挂载里推 22 次零单元发布之后写第一个文件，
+//!   两段合法历史上 I-5.5 成立（攻方原型 `research/prompts/m3-formal-code-b17-r1-opus-model/opus_b17_first_data_unit_attack.rs`
+//!   的形 A、形 B，写死 U + 4 的判据在这两段上误红）；一块盘上有两个数据单元的那一版，取的是每块盘上最小的那一个。
 //!
 //! 盘一律是内存稀疏盘（4 GiB × 2），不落镜像文件。
 
@@ -22,17 +27,21 @@ use singlefs_checker::image::{
 use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::journal::back_chain_of;
 use singlefs_core::make_filesystem::{
     allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
 };
-use singlefs_core::mount::mount_writable;
+use singlefs_core::mount::{mount_writable, Mounted};
 use singlefs_core::root_ring::{slot_offset, target_for_publish};
 use singlefs_core::transaction::{
-    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion,
-    PoolWriter, TransactionOutput,
+    acquire_instance, publish_first_file, publish_overwrite, publish_version,
+    publish_without_units, warm_up, FileVersionPlan, FirstFile, InstanceTablePlan, PoolVersion,
+    PoolWriter, PublishPlan, TransactionOutput, ZeroUnitPublishPlan, NEW_POOL_FILE_CREATION_NUMBER,
 };
+use singlefs_core::unit::data_unit_payload_capacity;
 use singlefs_format::{
-    DATA_UNIT_BYTES, ROOT_RING_REGIONS, ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
+    DATA_UNIT_BYTES, NEW_POOL_FILE_CREATION_TXG, ROOT_RING_REGIONS,
+    ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
 };
 use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
 use singlefs_harness::{RecordingBlockDevice, SharedStream};
@@ -45,6 +54,13 @@ const ROOT_RING_SLOT_COUNT: u64 = ROOT_RING_REGIONS * ROOT_RING_SLOTS_PER_REGION
 const FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START: u64 = 4;
 /// 非默认环长的那一种：128 MiB 环，单元区起始槽号现算（攻方在这一格上量过 9216，`research/prompts/c673-clauses-r1-opus-model/out/probe.out`）。
 const JOURNAL_RING_BYTES_OF_128_MEBIBYTES: u64 = 128 << 20;
+/// 只做过 mkfs 的池只暖机挂载这么多次，第这一次挂载上写的第一个文件版本落到 U：实例 2 写行时把 mkfs 的实例表（U、U + 1）COW 走、
+/// 释放代 3；第 10 次挂载上的第一个文件版本 txg 29，写它之前环里最旧有效根是 29 − 24 = 5 ≥ 3，U、U + 1 已回收（第 9 次是 26 − 24 = 2 < 3，还没有）。
+/// 攻方在这一格上量过翻转点 9 → 10（`research/prompts/m3-formal-code-b17-r1-opus-model/out/runA1.log` 第 15、17 行）。
+const WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED: usize = 10;
+/// 实例 2 里接连推这么多次零单元发布之后写的第一个文件版本落到 U（攻方量过翻转点 21 → 22，
+/// `research/prompts/m3-formal-code-b17-r1-opus-model/out/runB1.log` 第 28、29 行）。
+const ZERO_UNIT_PUBLISHES_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED: u64 = 22;
 
 /// 两块空的内存稀疏盘，写录进同一条流。
 fn memory_devices(stream: &SharedStream) -> Vec<(DeviceIdentity, MemoryDevice)> {
@@ -429,3 +445,223 @@ fn once_the_first_file_version_leaves_the_root_ring_the_overwrite_versions_are_n
         "覆盖写一圈之后的镜像：池级 checker 一条都不红"
     );
 }
+
+/// 一个文件版本的数据单元在每块盘上的落点槽号，按 (盘, 槽) 升序。
+fn data_unit_slots_on_each_device(first_file: &TransactionOutput) -> Vec<(u32, u64)> {
+    let mut slots: Vec<(u32, u64)> = first_file
+        .data_pointers
+        .iter()
+        .flat_map(|pointer| pointer.locations)
+        .map(|location| (location.device.0, location.slot.0))
+        .collect();
+    slots.sort_unstable();
+    slots
+}
+
+/// 只做过 mkfs 的池接连可写挂载 `mounts` 次（每次挂上只写行、暖机，不写文件），交回最后那一次挂载。
+fn mount_writable_repeatedly(
+    devices: &mut Vec<(DeviceIdentity, MemoryDevice)>,
+    mounts: usize,
+) -> Mounted {
+    let mut last_mount = mount_writable(&parameters(), devices).expect("第 1 次可写挂载");
+    for _ in 1..mounts {
+        last_mount = mount_writable(&parameters(), devices).expect("再一次可写挂载");
+    }
+    last_mount
+}
+
+/// 在一次挂载现行的那一版（树表 0 条）上接着发第一个文件版本，写进 `devices`。
+fn publish_the_first_file_on_the_mount(
+    devices: &mut [(DeviceIdentity, MemoryDevice)],
+    mounted: &Mounted,
+    version_root: &singlefs_core::root_record::RootRecord,
+    version_record_bytes: &[u8],
+) -> TransactionOutput {
+    let mut allocator = mounted.allocator.clone();
+    let publish_parameters = parameters();
+    let mut writer = PoolWriter::new(&publish_parameters, devices);
+    publish_first_file(
+        &mut writer,
+        &mut allocator,
+        version_root,
+        FirstFile {
+            content: &file_content(),
+            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
+        },
+        mounted.output.instance,
+        version_record_bytes,
+    )
+    .expect("树表 0 条的一版上发第一个文件版本")
+}
+
+/// 攻方原型形 A（k = 10）：只做过 mkfs 的池只暖机挂载 10 次，第 10 次挂载上写第一个文件。实例 2 写行时 mkfs 的实例表（U、U + 1）
+/// 被 COW 走，到这一版之前已过回收门槛：实现照 D3（空间分配） 已定项 8 第 1 条（起点最小、32768 对齐、两槽都空的那一对）把数据单元放在 U。
+/// 这一版仍从 mkfs 水位第一次建树（认法 ①②），I-5.5 按这一版的分配记录现算：U、U + 1 是这一版自己分配的、算空，该在 U，成立；
+/// 写死 U + 4 的判据在这里误红。池级 checker 一条都不红。
+#[test]
+fn the_first_file_after_ten_warm_up_only_mounts_lands_on_the_reclaimed_unit_area_start_and_holds_the_first_data_unit_placement(
+) {
+    let stream = SharedStream::retaining_contents();
+    let mut devices = memory_devices(&stream);
+    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
+    let unit_area_start_slot = genesis.unit_area_start.slot().0;
+    let mounted = mount_writable_repeatedly(
+        &mut devices,
+        WARM_UP_ONLY_MOUNTS_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED,
+    );
+    let PoolVersion::WithoutFile(version) = &mounted.current else {
+        panic!("只暖机过的池上现行那一版树表 0 条")
+    };
+    let first_file = publish_the_first_file_on_the_mount(
+        &mut devices,
+        &mounted,
+        &version.root,
+        &version.record_bytes,
+    );
+    assert_eq!(
+        (
+            first_file.root.checkpoint_txg,
+            first_file.tree_identifiers.inode.0
+        ),
+        (CheckpointTxg(29), 65537),
+        "第 10 次挂载上的第一个文件版本 txg 29，树仍从 mkfs 水位发"
+    );
+    assert_eq!(
+        data_unit_slots_on_each_device(&first_file),
+        vec![(0, unit_area_start_slot), (1, unit_area_start_slot)],
+        "mkfs 的实例表那一对已回收：实现把数据单元放在单元区起始槽号 U"
+    );
+    assert_the_first_data_unit_placement_holds_and_nothing_reddens(
+        &memory_pool_of_sparse_devices(&devices),
+        "只暖机挂载 10 次之后写第一个文件",
+    );
+}
+
+/// 攻方原型形 B（z = 22）：mkfs 之后可写挂载两次（实例 2），在实例 2 里接连推 22 次零单元发布，再写第一个文件——
+/// mkfs 的实例表那一对已过回收门槛，数据单元落在 U；I-5.5 按这一版的分配记录现算，成立，池级 checker 一条都不红。
+#[test]
+fn the_first_file_after_twenty_two_zero_unit_publishes_in_one_mount_lands_on_the_reclaimed_unit_area_start_and_holds_the_first_data_unit_placement(
+) {
+    let stream = SharedStream::retaining_contents();
+    let mut devices = memory_devices(&stream);
+    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
+    let unit_area_start_slot = genesis.unit_area_start.slot().0;
+    let mounted = mount_writable_repeatedly(&mut devices, 2);
+    let PoolVersion::WithoutFile(version_of_the_second_instance) = &mounted.current else {
+        panic!("只暖机过的池上现行那一版树表 0 条")
+    };
+    let mut current = version_of_the_second_instance.clone();
+    for _ in 0..ZERO_UNIT_PUBLISHES_UNTIL_THE_MKFS_INSTANCE_TABLE_IS_RECLAIMED {
+        let publish_parameters = parameters();
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        current = publish_without_units(
+            &mut writer,
+            &current.root,
+            ZeroUnitPublishPlan {
+                txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
+                counter: current.record.counter + 1,
+                instance: mounted.output.instance,
+                back_chain: back_chain_of(&current.record_bytes),
+                rollback_floor: current.root.rollback_floor,
+                tree_identifier_watermark: current.root.tree_identifier_watermark,
+            },
+        )
+        .expect("零单元发布");
+    }
+    let first_file = publish_the_first_file_on_the_mount(
+        &mut devices,
+        &mounted,
+        &current.root,
+        &current.record_bytes,
+    );
+    assert_eq!(
+        first_file.tree_identifiers.inode.0, 65537,
+        "第一个文件版本的树仍从 mkfs 水位发"
+    );
+    assert_eq!(
+        data_unit_slots_on_each_device(&first_file),
+        vec![(0, unit_area_start_slot), (1, unit_area_start_slot)],
+        "mkfs 的实例表那一对已回收：实现把数据单元放在单元区起始槽号 U"
+    );
+    assert_the_first_data_unit_placement_holds_and_nothing_reddens(
+        &memory_pool_of_sparse_devices(&devices),
+        "实例 2 里推 22 次零单元发布之后写第一个文件",
+    );
+}
+
+/// mkfs 同一个进程里那条流，第一个文件版本的内容比一个数据单元多一个字节：切成两个数据单元，每块盘上两个（U + 4 与 U + 6）。
+/// I-5.5 取的是每块盘上落点最小的那一个（U + 4），等于按分配记录现算的那一对，成立；取最大的那一个就误红。
+#[test]
+fn a_new_pool_file_of_two_data_units_holds_the_first_data_unit_placement_on_the_smallest_slot_of_each_device(
+) {
+    let stream = SharedStream::retaining_contents();
+    let mut devices = memory_devices(&stream);
+    let genesis = make_filesystem(&parameters(), &mut devices).expect("mkfs");
+    let unit_area_start_slot = genesis.unit_area_start.slot().0;
+    let mut allocator = allocator_after_make_filesystem(&parameters(), &devices, &genesis);
+    let content: Vec<u8> = file_content()
+        .into_iter()
+        .cycle()
+        .take(data_unit_payload_capacity() + 1)
+        .collect();
+    let first_file = {
+        let publish_parameters = parameters();
+        let mut writer = PoolWriter::new(&publish_parameters, &mut devices);
+        let instance = acquire_instance(&mut writer).expect("取号");
+        let warmed_up = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
+        let warmed_up_root = warmed_up.roots.last().expect("暖机两代根");
+        allocator.record_zero_unit_roots_leading_to(warmed_up_root.checkpoint_txg);
+        let txg = CheckpointTxg(NEW_POOL_FILE_CREATION_TXG);
+        publish_version(
+            &mut writer,
+            &mut allocator,
+            PublishPlan {
+                txg,
+                counter: warmed_up.records.last().expect("暖机两条记录").counter + 1,
+                transaction: NEW_POOL_FILE_CREATION_NUMBER,
+                highest_transaction_number_before_this_publish: 0,
+                instance,
+                back_chain: back_chain_of(&warmed_up.last_record_bytes),
+                file: Some(FileVersionPlan {
+                    content: &content,
+                    write_time_seconds: FIXED_WRITE_TIME_SECONDS,
+                    inode_object_birth: txg,
+                    change_count: txg.0,
+                }),
+                new_inode_records: &[],
+                instance_table: InstanceTablePlan::Carry(warmed_up_root.instance_table),
+                tree_birth_txg: txg,
+                tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
+                rollback_floor: warmed_up_root.rollback_floor,
+            },
+            None,
+        )
+        .expect("两个数据单元的新池新建文件")
+    };
+    assert_eq!(
+        data_unit_slots_on_each_device(&first_file),
+        vec![
+            (
+                0,
+                unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START
+            ),
+            (
+                0,
+                unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START + 2
+            ),
+            (
+                1,
+                unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START
+            ),
+            (
+                1,
+                unit_area_start_slot + FIRST_DATA_UNIT_SLOTS_PAST_THE_UNIT_AREA_START + 2
+            ),
+        ],
+        "两个数据单元落在 U + 4 与 U + 6，每块盘上各两个"
+    );
+    assert_the_first_data_unit_placement_holds_and_nothing_reddens(
+        &memory_pool_of_sparse_devices(&devices),
+        "两个数据单元的新池新建文件",
+    );
+}
````

变异追加：

````
C673 ⑥ I-5.5 判法（按分配记录现算，19-1）：落点改回从 U + 4 起扫，只暖机挂载 10 次之后第一个文件落在 U 的合法历史上误红	crates/singlefs-checker/src/walk.rs	    let mut candidate = unit_area_start_slot;\n    loop {	    let mut candidate = unit_area_start_slot + 4;\n    loop {	-p singlefs-harness --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement	the_first_file_after_ten_warm_up_only_mounts_lands_on_the_reclaimed_unit_area_start_and_holds_the_first_data_unit_placement
C673 ⑥ I-5.5 判法（按分配记录现算，19-1）：落点改回从 U + 4 起扫，实例 2 推 22 次零单元发布之后第一个文件落在 U 的合法历史上误红	crates/singlefs-checker/src/walk.rs	    let mut candidate = unit_area_start_slot;\n    loop {	    let mut candidate = unit_area_start_slot + 4;\n    loop {	-p singlefs-harness --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement	the_first_file_after_twenty_two_zero_unit_publishes_in_one_mount_lands_on_the_reclaimed_unit_area_start_and_holds_the_first_data_unit_placement
C673 ⑥ I-5.5 判法（按分配记录现算，19-1）：这一版自己分配的记录也当占着，新池新建文件的落点被推到 U + 6、误红	crates/singlefs-checker/src/walk.rs	                && (record.is_released || record.generation < version_checkpoint_txg)	                && (record.is_released || record.generation <= version_checkpoint_txg)	-p singlefs-harness --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement	a_new_pool_file_on_the_default_journal_ring_holds_the_first_data_unit_placement
C673 ⑥ I-5.5 判法（19-1 取样点）：每盘取落点最大的数据单元而不是最小的，两个数据单元的新池新建文件误红	crates/singlefs-checker/src/walk.rs	                .and_modify(|smallest_slot| *smallest_slot = (*smallest_slot).min(location.slot))	                .and_modify(|smallest_slot| *smallest_slot = (*smallest_slot).max(location.slot))	-p singlefs-harness --test pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement	a_new_pool_file_of_two_data_units_holds_the_first_data_unit_placement_on_the_smallest_slot_of_each_device
C673 ⑥ I-5.5 判法（按分配记录现算，19-1）：不排除开放段，U 那一段占满时落点算进这一版的开放段	crates/singlefs-checker/src/walk.rs	        if let Some(segment) = open_cluster_segment.filter(|segment| segment.holds_slot(candidate))	        if let Some(segment) = open_cluster_segment.filter(|segment| false && segment.holds_slot(candidate))	-p singlefs-checker --lib	the_first_data_unit_slot_skips_the_open_cluster_segment_of_the_version
C673 ⑥ I-5.5 判法（按分配记录现算，19-1）：已释放的记录不当占着，这一版自己释放的 U、U + 1 被算成落点	crates/singlefs-checker/src/walk.rs	                && (record.is_released || record.generation < version_checkpoint_txg)	                && (record.generation < version_checkpoint_txg)	-p singlefs-checker --lib	a_released_allocation_record_keeps_its_slots_out_of_the_first_data_unit_slot
C673 ⑤ I-9.27 判法（19-1 证据缺口）：种类 2 的头 ID 不比自己的树 ID（inode 树条目头 ID 改成树 ID + 1 的坏镜像不红）	crates/singlefs-checker/src/walk.rs	            HeadIdentifierOfATreeTableEntry::ItsOwnTreeIdentifier => head_identifier == tree,	            HeadIdentifierOfATreeTableEntry::ItsOwnTreeIdentifier => true,	-p singlefs-harness --test checker_known_bad_images -- each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
C673 ⑤ I-9.27 判法（19-1 证据缺口）：种类 3、4、6、7、8、11、12 的头 ID 不比 0（分配记录树条目头 ID 非 0 的坏镜像不红）	crates/singlefs-checker/src/walk.rs	            HeadIdentifierOfATreeTableEntry::Zero => head_identifier == 0,	            HeadIdentifierOfATreeTableEntry::Zero => true,	-p singlefs-harness --test checker_known_bad_images -- each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
C673 ⑤ I-9.27 判法（19-1 证据缺口）：deleted_inodes（种类 9）条目不判头 ID（它的头 ID 写 0 的坏镜像不红）	crates/singlefs-checker/src/walk.rs	            TREE_KIND_EXTENT | TREE_KIND_DELETED_INODES | TREE_KIND_TOMBSTONE => {	            TREE_KIND_EXTENT | TREE_KIND_TOMBSTONE => {	-p singlefs-harness --test checker_known_bad_images -- each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
C673 ⑤ I-9.27 判法（19-1 证据缺口）：墓碑树（种类 10）条目不判头 ID（它的头 ID 写 0 的坏镜像不红）	crates/singlefs-checker/src/walk.rs	            TREE_KIND_EXTENT | TREE_KIND_DELETED_INODES | TREE_KIND_TOMBSTONE => {	            TREE_KIND_EXTENT | TREE_KIND_DELETED_INODES => {	-p singlefs-harness --test checker_known_bad_images -- each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
C673 ⑤ I-9.27 判法（19-1 证据缺口）：多条种类 2 时不放宽、只认第一条（extent 树条目头 ID 写第二条 inode 树的树 ID 的合法表误红）	crates/singlefs-checker/src/walk.rs	inode_tree_identifiers.contains(&head_identifier)	inode_tree_identifiers.first() == Some(&head_identifier)	-p singlefs-checker --lib	an_extent_entry_naming_the_second_of_two_inode_tree_entries_holds_the_head_identifier_relation
C673 ⑤ I-9.27 判法（19-1）：种类 1、9、10 的头 ID 不比 inode 树的树 ID（两条 inode 树都对不上的合成表不红）	crates/singlefs-checker/src/walk.rs	inode_tree_identifiers.contains(&head_identifier)	true	-p singlefs-checker --lib	an_extent_entry_naming_neither_of_two_inode_tree_entries_violates_the_head_identifier_relation
````

## `/tmp/claude-1000/m3-formal-19-2/patch/crates.patch`

````diff
diff --git a/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs b/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
--- a/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
+++ b/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
@@ -15578,15 +15578,15 @@
         mount_two_of(FaultDomain::GateOn),
         mount_two_of(FaultDomain::GateOff),
     ) {
-        let mut passed = true;
-        let mut detail = Vec::new();
+        let mut gate_refusal_proof_passed = true;
+        let mut gate_refusal_proof_detail = Vec::new();
         for label in ["SN0-TP", "SN1-TP", "SB-TP", "RNSB-TP"] {
             let with_the_gate = gate_on.mount_two[label_index(label)];
             let without_the_gate = gate_off.mount_two[label_index(label)];
-            passed &= with_the_gate.histories > 0
+            gate_refusal_proof_passed &= with_the_gate.histories > 0
                 && with_the_gate.refused == with_the_gate.histories
                 && without_the_gate.refused == 0;
-            detail.push(format!(
+            gate_refusal_proof_detail.push(format!(
                 "{label}:KF_histories={}_KF_refused={}_KG_refused={}",
                 with_the_gate.histories, with_the_gate.refused, without_the_gate.refused
             ));
@@ -15594,20 +15594,20 @@
         proofs.push((
             "gate_refuses_every_history_of_the_four_tp_faults_and_the_gate_off_domain_refuses_none"
                 .to_string(),
-            passed,
-            detail.join(","),
+            gate_refusal_proof_passed,
+            gate_refusal_proof_detail.join(","),
         ));
-        let mut passed = true;
-        let mut detail = Vec::new();
+        let mut second_read_proof_passed = true;
+        let mut second_read_proof_detail = Vec::new();
         for label in ["SN0-T1", "SN1-T1", "SB-T1"] {
             let with_the_gate = gate_on.mount_two[label_index(label)];
-            passed &= with_the_gate.reread > 0;
-            detail.push(format!("{label}:KF_reread={}", with_the_gate.reread));
+            second_read_proof_passed &= with_the_gate.reread > 0;
+            second_read_proof_detail.push(format!("{label}:KF_reread={}", with_the_gate.reread));
         }
         proofs.push((
             "t1_takes_the_second_read_in_the_gate_on_domain".to_string(),
-            passed,
-            detail.join(","),
+            second_read_proof_passed,
+            second_read_proof_detail.join(","),
         ));
     }
     if let Some([kj1, kj1t]) = witnesses.kg_kf4_at_two_mounts_and_three_user_publishes {
@@ -17468,20 +17468,22 @@
     #[test]
     fn a_system_configuration_write_takes_the_generation_after_the_largest_one_it_can_read() {
         let mut slots = system_slots_right_after_mkfs();
-        let (slot, content) = system_slot_write_of(&slots, 0, 7, 3, FaultLocations::NOWHERE);
-        assert_eq!(slot, 0);
+        let (first_write_slot, first_write_content) =
+            system_slot_write_of(&slots, 0, 7, 3, FaultLocations::NOWHERE);
+        assert_eq!(first_write_slot, 0);
         assert_eq!(
-            content,
+            first_write_content,
             SystemSlotContent::Verified {
                 generation: 2,
                 tail: 7,
                 instance: 3
             }
         );
-        slots[0][0] = content;
-        let (slot, content) = system_slot_write_of(&slots, 0, 8, 3, FaultLocations::NOWHERE);
+        slots[0][0] = first_write_content;
+        let (second_write_slot, second_write_content) =
+            system_slot_write_of(&slots, 0, 8, 3, FaultLocations::NOWHERE);
         assert_eq!(
-            (slot, content),
+            (second_write_slot, second_write_content),
             (
                 1,
                 SystemSlotContent::Verified {
@@ -17492,9 +17494,13 @@
             )
         );
         let unreadable_slot_zero = FaultLocations::NOWHERE.with_system_slot(0, 0);
-        let (slot, content) = system_slot_write_of(&slots, 0, 9, 4, unreadable_slot_zero);
+        let (write_slot_beside_the_unreadable_one, write_content_beside_the_unreadable_one) =
+            system_slot_write_of(&slots, 0, 9, 4, unreadable_slot_zero);
         assert_eq!(
-            (slot, content),
+            (
+                write_slot_beside_the_unreadable_one,
+                write_content_beside_the_unreadable_one
+            ),
             (
                 0,
                 SystemSlotContent::Verified {
@@ -17507,9 +17513,10 @@
         );
         slots[1][1] = SystemSlotContent::Torn;
         slots[1][0] = SystemSlotContent::Torn;
-        let (_slot, content) = system_slot_write_of(&slots, 1, 1, 1, FaultLocations::NOWHERE);
+        let (_slot, write_content_when_no_slot_reads) =
+            system_slot_write_of(&slots, 1, 1, 1, FaultLocations::NOWHERE);
         assert_eq!(
-            content,
+            write_content_when_no_slot_reads,
             SystemSlotContent::Verified {
                 generation: 1,
                 tail: 1,
diff --git a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs
--- a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs
+++ b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs
@@ -234,7 +234,7 @@
 /// 理想模型对一次调用的判定（O1-对拍，登记 5.3「调用与模型的对应」那张表）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 enum ModelVerdictOfCall {
-    /// 模型这一步不交答：取号、暖机（两次都返回之后一起推一步）与卸载（`model.rs` 里没有 `answer_unmount` 一族）。
+    /// 模型这一步不交答：取号、暖机（两次都返回之后一起推一步）与卸载（`crates/singlefs-harness/src/model.rs` 里已有 `answer_unmount`，本装置还没改调它，卸载这一步照旧不问模型；改调归 E171 第 1 次重跑）。
     NotModelled,
     /// 对得上，模型往前走。
     Agrees,
@@ -1987,7 +1987,7 @@
                         .filter_map(|(_, call)| *call)
                         .max();
                     let verdict = ImplementationVerdict::of(judgement);
-                    let started = Instant::now();
+                    let crash_judge_started = Instant::now();
                     let crash_judge = if !model_is_enabled {
                         CrashJudgeVerdict::Off
                     } else if !crash_judge_is_applicable(touched_call, stream.model_stopped_at_call)
@@ -2002,8 +2002,8 @@
                             &mut controls,
                         )
                     };
-                    let o1_nanoseconds = started.elapsed().as_nanos();
-                    let started = Instant::now();
+                    let o1_nanoseconds = crash_judge_started.elapsed().as_nanos();
+                    let pool_checker_started = Instant::now();
                     let pool_checker_red = pool_checker_is_enabled.then(|| {
                         let direct = if controls.switches.positive_control
                             == Some(PositiveControl::PoolChecker)
@@ -2016,17 +2016,17 @@
                         };
                         violated_invariant_names(&direct)
                     });
-                    let o2_nanoseconds = started.elapsed().as_nanos();
+                    let o2_nanoseconds = pool_checker_started.elapsed().as_nanos();
                     let pool_checker_matches_the_judgement = pool_checker_red
                         .as_ref()
                         .is_none_or(|direct| *direct == verdict.pool_checker_red)
                         || controls.switches.positive_control == Some(PositiveControl::PoolChecker);
-                    let started = Instant::now();
+                    let recovery_started = Instant::now();
                     let _ = singlefs_core::recovery::recover(
                         image,
                         singlefs_core::recovery::JournalPolicy::Consult,
                     );
-                    let recovery_nanoseconds = started.elapsed().as_nanos();
+                    let recovery_nanoseconds = recovery_started.elapsed().as_nanos();
                     observations.push(StateObservation {
                         batch,
                         ordinal,
@@ -5737,11 +5737,13 @@
             0,
             RunSwitches::default(),
         );
-        let refused: Result<(), RefusedForTheTest> = Err(
+        let refused_acquire: Result<(), RefusedForTheTest> = Err(
             RefusedForTheTest::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness,
         );
-        let failure = driver.call("acquire", || refused).expect_err("取号被拒");
-        driver.record_a_refusal_the_model_only_goes_forward_on(failure);
+        let acquire_failure = driver
+            .call("acquire", || refused_acquire)
+            .expect_err("取号被拒");
+        driver.record_a_refusal_the_model_only_goes_forward_on(acquire_failure);
         assert_eq!(
             driver.calls[0].model,
             ModelVerdictOfCall::Disagrees {
@@ -5764,13 +5766,13 @@
         );
         let accepted: Result<(), RefusedForTheTest> = Ok(());
         assert!(after_a_good_acquire.call("acquire", || accepted).is_ok());
-        let refused: Result<(), RefusedForTheTest> = Err(
+        let refused_warm_up: Result<(), RefusedForTheTest> = Err(
             RefusedForTheTest::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness,
         );
-        let failure = after_a_good_acquire
-            .call("warm_up", || refused)
+        let warm_up_failure = after_a_good_acquire
+            .call("warm_up", || refused_warm_up)
             .expect_err("暖机被拒");
-        after_a_good_acquire.record_a_refusal_the_model_only_goes_forward_on(failure);
+        after_a_good_acquire.record_a_refusal_the_model_only_goes_forward_on(warm_up_failure);
         assert_eq!(
             after_a_good_acquire.calls[0].model,
             ModelVerdictOfCall::NotModelled
@@ -5901,9 +5903,9 @@
             errored("X"),
             ModelVerdictOfCall::NotModelled,
         )];
-        let checks = s5_checks_of_a_bad_implementation_row("s", &silent, Some(0), 0);
+        let silent_acquire_checks = s5_checks_of_a_bad_implementation_row("s", &silent, Some(0), 0);
         assert_eq!(
-            checks
+            silent_acquire_checks
                 .iter()
                 .map(|check| check.name.as_str())
                 .collect::<Vec<_>>(),
@@ -5913,7 +5915,7 @@
             ]
         );
         assert_eq!(
-            checks
+            silent_acquire_checks
                 .iter()
                 .map(|check| (check.local.as_str(), check.crates.as_str(), check.holds()))
                 .collect::<Vec<_>>(),
@@ -5925,8 +5927,8 @@
             errored("X"),
             disagreement_for_the_test(),
         )];
-        let checks = s5_checks_of_a_bad_implementation_row("s", &red, Some(0), 9);
-        assert!(checks.iter().all(StopCheck::holds));
+        let recorded_red_checks = s5_checks_of_a_bad_implementation_row("s", &red, Some(0), 9);
+        assert!(recorded_red_checks.iter().all(StopCheck::holds));
         // 对拍在发布上判出不同、模型停在那里，之后的可写挂载出错（`skipped_after_the_model_stopped`，第 108 行那一形）：登记读法 6 的合规来源，
         // 两样都 0（登记修订 4）。
         let error_after_the_stop = [
@@ -5941,9 +5943,10 @@
                 ModelVerdictOfCall::SkippedAfterTheModelStopped,
             ),
         ];
-        let checks = s5_checks_of_a_bad_implementation_row("s", &error_after_the_stop, Some(0), 6);
+        let error_after_the_stop_checks =
+            s5_checks_of_a_bad_implementation_row("s", &error_after_the_stop, Some(0), 6);
         assert_eq!(
-            checks
+            error_after_the_stop_checks
                 .iter()
                 .map(|check| (check.local.as_str(), check.crates.as_str(), check.holds()))
                 .collect::<Vec<_>>(),
@@ -5963,9 +5966,10 @@
             ),
             recorded_call_for_the_test("publish", errored("Y"), ModelVerdictOfCall::Agrees),
         ];
-        let checks = s5_checks_of_a_bad_implementation_row("s", &allowed_refusal, Some(2), 5);
+        let allowed_refusal_checks =
+            s5_checks_of_a_bad_implementation_row("s", &allowed_refusal, Some(2), 5);
         assert_eq!(
-            checks
+            allowed_refusal_checks
                 .iter()
                 .map(|check| (check.local.as_str(), check.crates.as_str(), check.holds()))
                 .collect::<Vec<_>>(),
@@ -5984,16 +5988,17 @@
                 ModelVerdictOfCall::NotModelled,
             ),
         ];
-        let checks = s5_checks_of_a_bad_implementation_row("s", &unmount_stop, Some(1), 7);
-        assert!(checks.iter().all(StopCheck::holds));
+        let unmount_stop_checks =
+            s5_checks_of_a_bad_implementation_row("s", &unmount_stop, Some(1), 7);
+        assert!(unmount_stop_checks.iter().all(StopCheck::holds));
         // 模型没停：没有不适用的态，合规。
         let no_stop = [recorded_call_for_the_test(
             "publish",
             CallOutcome::Succeeded,
             ModelVerdictOfCall::Agrees,
         )];
-        let checks = s5_checks_of_a_bad_implementation_row("s", &no_stop, None, 0);
-        assert!(checks.iter().all(StopCheck::holds));
+        let no_stop_checks = s5_checks_of_a_bad_implementation_row("s", &no_stop, None, 0);
+        assert!(no_stop_checks.iter().all(StopCheck::holds));
     }
 
     /// 哪一趟判哪一份 S5（代码轮第三轮判决 K4-c 丙）：逐行那一趟判「返回了错误的调用都记了对拍的红」那一份，基线那一趟判原来那一份；
````
