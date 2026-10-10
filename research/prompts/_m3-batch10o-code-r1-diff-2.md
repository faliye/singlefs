# 根环实现第 10q、10r、10s 批与 #9 钉值重取：代码轮第一轮改动全文（2026-10-06）

三份补丁照实现员交的 `crates.patch` 原样（10q、10r 合进主工作区时 `apply-writer-patch.py` 先 10q 后 10r，10s 在删除线第 30 批之后打）；#9 那一段是主 agent 自己改的，合前原样 `~/.local/share/singlefs/work/m3-r1/golden9/before-crash_enumeration_sharded_across_processes.rs`。变异表追加与替换的行在各补丁目录的 `mutations-append.tsv`、`mutations-replacements.tsv`，附在每段末尾。

## 第 10q 批

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -673,36 +673,48 @@
         }
     }
     let newest_root = checker_root_of(c_newest_index);
-    r_noting = false;
-    c_alloc_record_count = 0u;
-    r_references_of_root(newest_root.base, true, newest_slot);
-    c_released_count = 0u;
-    for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
-        let record = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
-        if (sw(record + 6u) == 0u) {
+    // 第 0 圈走最新根一遍取它的账，之后每圈走一条候选根：同一个调用点（GPU 程序把每个调用点展开成一份拷贝）
+    var every = false;
+    for (var walk_index = 0u; walk_index <= c_candidate_count; walk_index = walk_index + 1u) {
+        var record_base = newest_root.base;
+        var walks_every_node = true;
+        var candidate_slot = newest_slot;
+        if (walk_index == 0u) {
+            r_noting = false;
+        } else {
+            let candidate = walk_index - 1u;
+            let index = sw(C_CANDIDATES + candidate);
+            record_base = checker_root_of(index).base;
+            walks_every_node = every || index == c_newest_index;
+            candidate_slot = candidate;
+            r_candidate = candidate;
+            r_noting = every;
+        }
+        c_alloc_record_count = 0u;
+        r_references_of_root(record_base, walks_every_node, candidate_slot);
+        if (walk_index != 0u) {
             continue;
         }
-        if (c_released_count >= C_MAX_RELEASED) {
-            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (29u << 8u);
-            return;
+        c_released_count = 0u;
+        for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
+            let record = C_ALLOC_RECORDS + i * C_ALLOC_RECORD_WORDS;
+            if (sw(record + 6u) == 0u) {
+                continue;
+            }
+            if (c_released_count >= C_MAX_RELEASED) {
+                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (29u << 8u);
+                return;
+            }
+            sws(C_RELEASED + c_released_count * 5u, sw(record));
+            sws(C_RELEASED + c_released_count * 5u + 1u, sw(record + 1u));
+            sws(C_RELEASED + c_released_count * 5u + 2u, sw(record + 2u));
+            sws(C_RELEASED + c_released_count * 5u + 3u, sw(record + 4u));
+            sws(C_RELEASED + c_released_count * 5u + 4u, sw(record + 5u));
+            sws(C_REFERENCED_BY + c_released_count * 2u, 0u);
+            sws(C_REFERENCED_BY + c_released_count * 2u + 1u, 0u);
+            c_released_count = c_released_count + 1u;
         }
-        sws(C_RELEASED + c_released_count * 5u, sw(record));
-        sws(C_RELEASED + c_released_count * 5u + 1u, sw(record + 1u));
-        sws(C_RELEASED + c_released_count * 5u + 2u, sw(record + 2u));
-        sws(C_RELEASED + c_released_count * 5u + 3u, sw(record + 4u));
-        sws(C_RELEASED + c_released_count * 5u + 4u, sw(record + 5u));
-        sws(C_REFERENCED_BY + c_released_count * 2u, 0u);
-        sws(C_REFERENCED_BY + c_released_count * 2u + 1u, 0u);
-        c_released_count = c_released_count + 1u;
-    }
-    let every = c_released_count > 0u;
-    for (var c = 0u; c < c_candidate_count; c = c + 1u) {
-        let index = sw(C_CANDIDATES + c);
-        let root = checker_root_of(index);
-        r_candidate = c;
-        r_noting = every;
-        c_alloc_record_count = 0u;
-        r_references_of_root(root.base, every || index == c_newest_index, c);
+        every = c_released_count > 0u;
     }
     r_noting = false;
     // I-3.9
@@ -963,26 +975,26 @@
 fn judge_allocation_records_disjoint() {
     c_seen_node_count = 0u;
     r_noting = false;
-    for (var c = 0u; c < c_candidate_count; c = c + 1u) {
-        let root = checker_root_of(sw(C_CANDIDATES + c));
-        judge_allocation_records_disjoint_of_a_version(root.base, 36u, 342u);
-    }
-    for (var v = 0u; v < c_version_count; v = v + 1u) {
-        let r = checker_record_at(sw(R_DATA + v));
-        judge_allocation_records_disjoint_of_a_version(
-            r.base,
-            JOURNAL_NEW_ROOT_SEGMENT_OFFSET,
-            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER,
-        );
-    }
-    let next_version = record_row_of_the_version_the_next_mount_applies_first();
-    if (next_version != NONE) {
-        let r = checker_record_of_row(next_version);
-        judge_allocation_records_disjoint_of_a_version(
-            r.base,
-            JOURNAL_NEW_ROOT_SEGMENT_OFFSET,
-            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER,
-        );
+    // 候选集里每条根、由记录施加出来的那几版、下一次挂载先施加的那一版，次序照旧，同一个调用点；
+    // 最后那一版在前两段判完之后才找（与原先同一时点）
+    for (var version_index = 0u; version_index <= c_candidate_count + c_version_count; version_index = version_index + 1u) {
+        var base = 0u;
+        var tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
+        var allocation_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER;
+        if (version_index < c_candidate_count) {
+            base = checker_root_of(sw(C_CANDIDATES + version_index)).base;
+            tree_table_offset = 36u;
+            allocation_offset = 342u;
+        } else if (version_index < c_candidate_count + c_version_count) {
+            base = checker_record_at(sw(R_DATA + (version_index - c_candidate_count))).base;
+        } else {
+            let next_version = record_row_of_the_version_the_next_mount_applies_first();
+            if (next_version == NONE) {
+                break;
+            }
+            base = checker_record_of_row(next_version).base;
+        }
+        judge_allocation_records_disjoint_of_a_version(base, tree_table_offset, allocation_offset);
     }
 }
 
@@ -1068,24 +1080,32 @@
     c_seen_node_count = 0u;
     c_examined_count = 0u;
     r_noting = false;
-    for (var c = 0u; c < c_candidate_count; c = c + 1u) {
-        let root = checker_root_of(sw(C_CANDIDATES + c));
-        let held = parse_node_pointer(root.base, 342u);
-        if (!held.all_zero && c_seen_node_insert(held)) {
-            c_alloc_record_count = 0u;
-            if (r_allocation_tree(root.base, 342u)) {
-                judge_allocation_generations_of_the_records();
-            }
-        }
-        judge_allocation_generations_under_tree_table(parse_node_pointer(root.base, 36u));
-    }
-    for (var v = 0u; v < c_version_count; v = v + 1u) {
-        let r = checker_record_at(sw(R_DATA + v));
-        judge_allocation_generations_under_tree_table(parse_node_pointer(r.base, 99u));
-    }
-    let next_version = record_row_of_the_version_the_next_mount_applies_first();
-    if (next_version != NONE) {
-        judge_allocation_generations_under_tree_table(parse_node_pointer(checker_record_of_row(next_version).base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET));
+    // 候选集里每条根（先判它持有的那棵分配记录树）、由记录施加出来的那几版、下一次挂载先施加的那一版（在前两段判完之后才找），
+    // 次序照旧，树表那一判同一个调用点
+    for (var version_index = 0u; version_index <= c_candidate_count + c_version_count; version_index = version_index + 1u) {
+        var base = 0u;
+        var tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
+        if (version_index < c_candidate_count) {
+            let root = checker_root_of(sw(C_CANDIDATES + version_index));
+            base = root.base;
+            tree_table_offset = 36u;
+            let held = parse_node_pointer(root.base, 342u);
+            if (!held.all_zero && c_seen_node_insert(held)) {
+                c_alloc_record_count = 0u;
+                if (r_allocation_tree(root.base, 342u)) {
+                    judge_allocation_generations_of_the_records();
+                }
+            }
+        } else if (version_index < c_candidate_count + c_version_count) {
+            base = checker_record_at(sw(R_DATA + (version_index - c_candidate_count))).base;
+        } else {
+            let next_version = record_row_of_the_version_the_next_mount_applies_first();
+            if (next_version == NONE) {
+                break;
+            }
+            base = checker_record_of_row(next_version).base;
+        }
+        judge_allocation_generations_under_tree_table(parse_node_pointer(base, tree_table_offset));
     }
 }
 
@@ -1706,13 +1726,19 @@
     let newest_root = checker_root_of(c_newest_index);
     let written_under_effective_floor = eq64(newest_root.floor, c_effective_floor);
     c_harvested = false;
-    c_quarantine_exempted(newest_root.base, w_reference_count[0], fsid_low);
+    // 两遍同一个调用点：第一遍到走读的引用数为止，存下它的豁免数、算完 F 之下的推迟，第二遍到最新根自己的引用数为止
     var exempted_all: array<u32, 8>;
-    for (var d = 0u; d < 8u; d = d + 1u) {
-        exempted_all[d] = c_exempted[d];
+    for (var exemption_pass = 0u; exemption_pass < 2u; exemption_pass = exemption_pass + 1u) {
+        var reference_limit = w_reference_count[0];
+        if (exemption_pass == 1u) {
+            for (var d = 0u; d < 8u; d = d + 1u) {
+                exempted_all[d] = c_exempted[d];
+            }
+            c_deferred_below_the_floor(newest_root.base);
+            reference_limit = c_newest_reference_count;
+        }
+        c_quarantine_exempted(newest_root.base, reference_limit, fsid_low);
     }
-    c_deferred_below_the_floor(newest_root.base);
-    c_quarantine_exempted(newest_root.base, c_newest_reference_count, fsid_low);
     for (var d = 0u; d < device_count; d = d + 1u) {
         let device = hdr(12u + d);
         var allocated = vec2<u32>(0u, 0u);
@@ -2095,13 +2121,22 @@
 
 // I-5.5：回退候选集里每条根（树表指针在根记录偏移 36）与由记录施加出来的那几版（记录新根段打头的树表指针）
 fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark() {
-    for (var candidate = 0u; candidate < c_candidate_count; candidate = candidate + 1u) {
-        let root = checker_root_of(sw(C_CANDIDATES + candidate));
-        judge_first_data_unit_of_a_version(root.base, 36u, root.txg);
-    }
-    for (var version = 0u; version < c_version_count; version = version + 1u) {
-        let record = checker_record_at(sw(R_DATA + version));
-        judge_first_data_unit_of_a_version(record.base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET, record.txg);
+    // 先候选集里每条根、再由记录施加出来的那几版，同一个调用点
+    for (var version_index = 0u; version_index < c_candidate_count + c_version_count; version_index = version_index + 1u) {
+        var base = 0u;
+        var tree_table_offset = 36u;
+        var txg = vec2<u32>(0u, 0u);
+        if (version_index < c_candidate_count) {
+            let root = checker_root_of(sw(C_CANDIDATES + version_index));
+            base = root.base;
+            txg = root.txg;
+        } else {
+            let record = checker_record_at(sw(R_DATA + (version_index - c_candidate_count)));
+            base = record.base;
+            tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
+            txg = record.txg;
+        }
+        judge_first_data_unit_of_a_version(base, tree_table_offset, txg);
     }
 }
 
@@ -2530,13 +2565,17 @@
 
 // 删除态扫描：回退候选集里每条根（树表指针在根记录偏移 36）与由记录施加出来的那几版（记录新根段打头的树表指针），同 I-5.5 那一份清单
 fn judge_deletion_state_of_the_versions() {
-    for (var candidate = 0u; candidate < c_candidate_count; candidate = candidate + 1u) {
-        let root = checker_root_of(sw(C_CANDIDATES + candidate));
-        d_judge_deletion_state_of_a_version(root.base, 36u);
-    }
-    for (var version = 0u; version < c_version_count; version = version + 1u) {
-        let record = checker_record_at(sw(R_DATA + version));
-        d_judge_deletion_state_of_a_version(record.base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET);
+    // 先候选集里每条根、再由记录施加出来的那几版，同一个调用点
+    for (var version_index = 0u; version_index < c_candidate_count + c_version_count; version_index = version_index + 1u) {
+        var base = 0u;
+        var tree_table_offset = 36u;
+        if (version_index < c_candidate_count) {
+            base = checker_root_of(sw(C_CANDIDATES + version_index)).base;
+        } else {
+            base = checker_record_at(sw(R_DATA + (version_index - c_candidate_count))).base;
+            tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
+        }
+        d_judge_deletion_state_of_a_version(base, tree_table_offset);
     }
 }
 
@@ -3113,13 +3152,22 @@
 fn judge_tombstones_and_the_lineage_of_the_versions() {
     d_object_birth_count = 0u;
     d_object_births_are_compared = sw(H_BASE + H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE) == 1u;
-    for (var candidate = 0u; candidate < c_candidate_count; candidate = candidate + 1u) {
-        let root = checker_root_of(sw(C_CANDIDATES + candidate));
-        d_judge_tombstones_and_the_lineage_of_a_version(root.base, 36u, root.txg);
-    }
-    for (var version = 0u; version < c_version_count; version = version + 1u) {
-        let record = checker_record_at(sw(R_DATA + version));
-        d_judge_tombstones_and_the_lineage_of_a_version(record.base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET, record.txg);
+    // 先候选集里每条根、再由记录施加出来的那几版，同一个调用点
+    for (var version_index = 0u; version_index < c_candidate_count + c_version_count; version_index = version_index + 1u) {
+        var base = 0u;
+        var tree_table_offset = 36u;
+        var txg = vec2<u32>(0u, 0u);
+        if (version_index < c_candidate_count) {
+            let root = checker_root_of(sw(C_CANDIDATES + version_index));
+            base = root.base;
+            txg = root.txg;
+        } else {
+            let record = checker_record_at(sw(R_DATA + (version_index - c_candidate_count)));
+            base = record.base;
+            tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
+            txg = record.txg;
+        }
+        d_judge_tombstones_and_the_lineage_of_a_version(base, tree_table_offset, txg);
     }
 }
 
````

### 10q mutations-append.tsv

````text
第10q批 扫描内核单调用点：I-3.9 与 I-9.14 走根的 r_references_of_root 加回一个调用点（展开超预算）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        r_references_of_root(record_base, walks_every_node, candidate_slot);	        r_references_of_root(record_base, walks_every_node, candidate_slot);\n        r_references_of_root(record_base, walks_every_node, candidate_slot);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_dispatch::tests::every_kernel_stays_within_the_inlined_size_budget	every_kernel_stays_within_the_inlined_size_budget
第10q批 扫描内核单调用点：I-5.4 分配记录不相交的一版一判加回两个调用点（回到三处，展开超预算）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        judge_allocation_records_disjoint_of_a_version(base, tree_table_offset, allocation_offset);	        judge_allocation_records_disjoint_of_a_version(base, tree_table_offset, allocation_offset);\n        judge_allocation_records_disjoint_of_a_version(base, tree_table_offset, allocation_offset);\n        judge_allocation_records_disjoint_of_a_version(base, tree_table_offset, allocation_offset);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_dispatch::tests::every_kernel_stays_within_the_inlined_size_budget	every_kernel_stays_within_the_inlined_size_budget
第10q批 扫描内核单调用点：分配代号对单元出生的树表那一判加回两个调用点（回到三处，展开超预算）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        judge_allocation_generations_under_tree_table(parse_node_pointer(base, tree_table_offset));	        judge_allocation_generations_under_tree_table(parse_node_pointer(base, tree_table_offset));\n        judge_allocation_generations_under_tree_table(parse_node_pointer(base, tree_table_offset));\n        judge_allocation_generations_under_tree_table(parse_node_pointer(base, tree_table_offset));	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_dispatch::tests::every_kernel_stays_within_the_inlined_size_budget	every_kernel_stays_within_the_inlined_size_budget
第10q批 扫描内核单调用点：I-5.5 第一个数据单元的一版一判加回一个调用点（展开超预算）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        judge_first_data_unit_of_a_version(base, tree_table_offset, txg);	        judge_first_data_unit_of_a_version(base, tree_table_offset, txg);\n        judge_first_data_unit_of_a_version(base, tree_table_offset, txg);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_dispatch::tests::every_kernel_stays_within_the_inlined_size_budget	every_kernel_stays_within_the_inlined_size_budget
第10q批 扫描内核单调用点：删除态扫描的一版一判加回一个调用点（展开超预算）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        d_judge_deletion_state_of_a_version(base, tree_table_offset);	        d_judge_deletion_state_of_a_version(base, tree_table_offset);\n        d_judge_deletion_state_of_a_version(base, tree_table_offset);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_dispatch::tests::every_kernel_stays_within_the_inlined_size_budget	every_kernel_stays_within_the_inlined_size_budget
第10q批 扫描内核单调用点：墓碑与谱系的一版一判加回一个调用点（展开超预算）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        d_judge_tombstones_and_the_lineage_of_a_version(base, tree_table_offset, txg);	        d_judge_tombstones_and_the_lineage_of_a_version(base, tree_table_offset, txg);\n        d_judge_tombstones_and_the_lineage_of_a_version(base, tree_table_offset, txg);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_dispatch::tests::every_kernel_stays_within_the_inlined_size_budget	every_kernel_stays_within_the_inlined_size_budget
````

### 10q mutations-replacements.tsv

````text
第34批 34-1 GPU 判器 RV：I-5.4 不收下一次挂载先施加的那一版（新池新建文件那次发布的分配记录叶里两条记录罩住同一个槽，根槽没落的状态上 CPU 红、GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	            if (next_version == NONE) {\n                break;\n            }\n            base = checker_record_of_row(next_version).base;\n        }\n        judge_allocation_records_disjoint_of_a_version(base, tree_table_offset, allocation_offset);	            if (next_version == NONE || true) {\n                break;\n            }\n            base = checker_record_of_row(next_version).base;\n        }\n        judge_allocation_records_disjoint_of_a_version(base, tree_table_offset, allocation_offset);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first	the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first
````

## 第 10r 批

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
index 7338e79..8452d18 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -17,6 +17,8 @@ use crate::crash_judge_gpu::{
 };
 use crate::crash_judge_tables::JudgeTables;
 use crate::gpu_unit_checks::GpuCard;
+use singlefs_harness::gpu_memory_ledger::states_in_one_dispatch_capped_by_the_state_count;
+use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};
 
 /// 草稿区最多分几条缓冲（32 条是 64 GiB，比今天任何一张卡的显存都大：条数实际由显存额度定）。
 pub const MAXIMUM_SCRATCH_LANES: usize = 32;
@@ -134,6 +136,42 @@ pub fn states_in_flight_within(
     states_per_dispatch(max_storage_buffer_binding_size, scratch_memory_mebibytes)
 }
 
+/// 一个判器在卡上要占多少显存（MiB，向上取整），不算 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`] 那一份（那一份每张卡只付一次，归卡池的账）：
+/// 表，加一次派活带 `requested_states_per_dispatch` 与这条流状态数里小的那个（至少 1）的草稿区，
+/// 加每次派活的结论与回读缓冲（每个状态各 [`VERDICT_WORDS`] 个字）。用例向卡池借卡时按它借
+/// （`crate::gpu_unit_checks::gpu_test_start::GpuCardPool::lease_card_waiting_for_room`）；
+/// 借来的卡额度是它加那一份，判器照 [`states_in_flight_within`] 开出来的草稿区不少于这里算进去的状态数。
+#[must_use]
+pub fn mebibytes_a_judge_needs(
+    tables: &PackedJudgeTables,
+    requested_states_per_dispatch: usize,
+) -> u64 {
+    let states = u64::try_from(states_in_one_dispatch_capped_by_the_state_count(
+        usize::MAX,
+        requested_states_per_dispatch,
+        tables.state_count,
+    ))
+    .expect("装得进 u64");
+    let bytes_per_state =
+        u64::try_from(SCRATCH_WORDS * 4 + 2 * VERDICT_WORDS * 4).expect("装得进 u64");
+    tables.mebibytes() + (states * bytes_per_state).div_ceil(1024 * 1024)
+}
+
+/// 三份内核源码合起来的摘要：同一张卡上同一套源码只编一次（[`GpuCard::pipelines_compiled_once_on_this_device`]）的键。
+/// 每份前面带它的标签与字节数，三份之间的边界挪了摘要跟着变。
+fn kernel_sources_digest(kernel_sources: &JudgeKernelSources) -> [u8; DIGEST_BYTES] {
+    let mut message = Vec::new();
+    for (label, source) in [
+        ("recovery", &kernel_sources.recovery),
+        ("checker-walk", &kernel_sources.checker_walk),
+        ("checker-scan", &kernel_sources.checker_scan),
+    ] {
+        message.extend_from_slice(format!("{label}\n{}\n", source.len()).as_bytes());
+        message.extend_from_slice(source.as_bytes());
+    }
+    sha256_digest(&message)
+}
+
 /// 一张卡上装好表的判器。
 pub struct GpuCrashJudge<'card> {
     card: &'card GpuCard,
@@ -383,6 +421,57 @@ fn compute_pipeline(card: &GpuCard, label: &str, source: &str) -> wgpu::ComputeP
     })
 }
 
+/// 在这张卡上编判器的三个内核，按恢复、走读、扫描的次序交回。
+/// 三个内核各在一条线程上编（驱动的编译是单线程的，checker 两个内核冷编各要几分钟；盘上缓存装好之后几秒）。
+/// 驱动编大内核吃内存（595 版三个一起编常驻 8.6 GiB，580 版一起编撞过 40G 上限）：内存紧的机器写
+/// `SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME=1`，三个内核一个一个编。
+///
+/// # Panics
+/// 有一个内核超了展开预算（[`refuse_a_kernel_too_large_for_the_driver`]）。
+fn compile_the_three_judge_kernels(
+    card: &GpuCard,
+    kernel_sources: &JudgeKernelSources,
+) -> [wgpu::ComputePipeline; 3] {
+    let one_at_a_time =
+        std::env::var("SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME").is_ok_and(|value| value == "1");
+    if one_at_a_time {
+        return [
+            compute_pipeline(card, "crash-judge-recovery", &kernel_sources.recovery),
+            compute_pipeline(
+                card,
+                "crash-judge-checker-walk",
+                &kernel_sources.checker_walk,
+            ),
+            compute_pipeline(
+                card,
+                "crash-judge-checker-scan",
+                &kernel_sources.checker_scan,
+            ),
+        ];
+    }
+    std::thread::scope(|scope| {
+        let recovery = scope
+            .spawn(|| compute_pipeline(card, "crash-judge-recovery", &kernel_sources.recovery));
+        let scan = scope.spawn(|| {
+            compute_pipeline(
+                card,
+                "crash-judge-checker-scan",
+                &kernel_sources.checker_scan,
+            )
+        });
+        let walk = compute_pipeline(
+            card,
+            "crash-judge-checker-walk",
+            &kernel_sources.checker_walk,
+        );
+        [
+            recovery.join().expect("恢复内核的编译线程不 panic"),
+            walk,
+            scan.join().expect("扫描内核的编译线程不 panic"),
+        ]
+    })
+}
+
 impl<'card> GpuCrashJudge<'card> {
     /// 把一条流的表上卡、编内核。
     ///
@@ -393,7 +482,8 @@ impl<'card> GpuCrashJudge<'card> {
         Self::with_states_per_dispatch(card, tables, usize::MAX)
     }
 
-    /// 同 [`Self::new`]，一次派活的状态数另给（量速度、规划显卡用）：照旧被这张卡的显存额度与草稿缓冲的条数压住，至少 1。
+    /// 同 [`Self::new`]，一次派活的状态数另给（量速度、规划显卡用）：照旧被这张卡的显存额度、草稿缓冲的条数与这条流的状态数压住，至少 1
+    /// （[`states_in_one_dispatch_capped_by_the_state_count`]：草稿区不为这条流没有的状态开）。
     ///
     /// # Panics
     /// 同 [`Self::new`]。
@@ -450,9 +540,11 @@ impl<'card> GpuCrashJudge<'card> {
             "判器的表 {table_bytes} 字节超过这张卡的绑定上限 {limit}：这条流 GPU 判不了"
         );
         let states_per_lane = states_per_scratch_lane(limit);
-        let states_per_dispatch =
-            states_in_flight_within(limit, card.memory_budget_mebibytes(), tables)
-                .min(requested_states_per_dispatch.max(1));
+        let states_per_dispatch = states_in_one_dispatch_capped_by_the_state_count(
+            states_in_flight_within(limit, card.memory_budget_mebibytes(), tables),
+            requested_states_per_dispatch,
+            tables.state_count,
+        );
         let tables_buffer = wgpu::util::DeviceExt::create_buffer_init(
             device,
             &wgpu::util::BufferInitDescriptor {
@@ -474,49 +566,14 @@ impl<'card> GpuCrashJudge<'card> {
                 })
             })
             .collect();
-        // 三个内核各在一条线程上编（驱动的编译是单线程的，checker 两个内核冷编各要几分钟；盘上缓存装好之后几秒）。
-        // 驱动编大内核吃内存（595 版三个一起编常驻 8.6 GiB，580 版一起编撞过 40G 上限）：内存紧的机器写
-        // `SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME=1`，三个内核一个一个编。
-        let one_at_a_time =
-            std::env::var("SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME").is_ok_and(|value| value == "1");
-        let (recovery_pipeline, checker_walk_pipeline, checker_scan_pipeline) = if one_at_a_time {
-            (
-                compute_pipeline(card, "crash-judge-recovery", &kernel_sources.recovery),
-                compute_pipeline(
-                    card,
-                    "crash-judge-checker-walk",
-                    &kernel_sources.checker_walk,
-                ),
-                compute_pipeline(
-                    card,
-                    "crash-judge-checker-scan",
-                    &kernel_sources.checker_scan,
-                ),
-            )
-        } else {
-            std::thread::scope(|scope| {
-                let recovery = scope.spawn(|| {
-                    compute_pipeline(card, "crash-judge-recovery", &kernel_sources.recovery)
-                });
-                let scan = scope.spawn(|| {
-                    compute_pipeline(
-                        card,
-                        "crash-judge-checker-scan",
-                        &kernel_sources.checker_scan,
-                    )
-                });
-                let walk = compute_pipeline(
-                    card,
-                    "crash-judge-checker-walk",
-                    &kernel_sources.checker_walk,
-                );
-                (
-                    recovery.join().expect("恢复内核的编译线程不 panic"),
-                    walk,
-                    scan.join().expect("扫描内核的编译线程不 panic"),
-                )
+        // 同一张卡（同一个设备）上同一套源码只编一次：共用一张卡的几条用例里第一条编，别的等它编完直接拿
+        let [recovery_pipeline, checker_walk_pipeline, checker_scan_pipeline]: [wgpu::ComputePipeline;
+            3] = card
+            .pipelines_compiled_once_on_this_device(kernel_sources_digest(kernel_sources), || {
+                compile_the_three_judge_kernels(card, kernel_sources).to_vec()
             })
-        };
+            .try_into()
+            .expect("这个摘要那一格只由 compile_the_three_judge_kernels 填，里面恰好三条管线");
         Self {
             card,
             recovery_pipeline,
@@ -807,15 +864,16 @@ impl<'card> GpuCrashJudge<'card> {
                 u64::try_from(states * VERDICT_WORDS * 4).expect("装得进 u64"),
             );
         }
-        self.card.queue().submit(std::iter::once(encoder.finish()));
+        let submission = self.card.queue().submit(std::iter::once(encoder.finish()));
         let waiting_started = std::time::Instant::now();
         let slice = staging.slice(..);
         let (sender, receiver) = std::sync::mpsc::channel();
         slice.map_async(wgpu::MapMode::Read, move |result| {
             sender.send(result).expect("回调送得出去")
         });
+        // 只等这一次提交：几条用例共用这张卡的设备时，等整个设备会把别的用例的活算进这一次的用时（量宽度、超时都跟着偏）
         if let Err(error) = device.poll(wgpu::PollType::Wait {
-            submission_index: None,
+            submission_index: Some(submission),
             timeout: Some(std::time::Duration::from_secs(
                 LONGEST_WAIT_FOR_THE_CARD_SECONDS,
             )),
@@ -899,6 +957,39 @@ mod tests {
         );
     }
 
+    /// 卡池按 [`super::mebibytes_a_judge_needs`] 借给用例的卡（额度是借的量加驱动那一份）上，判器开的草稿区不少于借的时候算进去的状态数：
+    /// 小流（45、489 个状态）照状态数、大流（131 195 个）照要的宽度（一条缓冲那么窄、三条缓冲加窄的那一档那么宽）。
+    /// 表取 1 MiB：45 个状态要 1 + ⌈45 × 1 261 616 ÷ 1 048 576⌉ = 1 + 55 = 56 MiB。
+    #[test]
+    fn a_card_lent_what_a_judge_needs_opens_scratch_for_every_state_it_was_lent_for() {
+        use super::{mebibytes_a_judge_needs, states_in_flight_within, PackedJudgeTables};
+        let tables_of = |state_count: u64| PackedJudgeTables {
+            words: vec![0; 1024 * 1024 / 4],
+            state_count,
+        };
+        assert_eq!(mebibytes_a_judge_needs(&tables_of(45), usize::MAX), 56);
+        for (state_count, requested) in [
+            (45, usize::MAX),
+            (489, usize::MAX),
+            (131_195, 1702),
+            (131_195, 3 * 1702),
+            (131_195, 256),
+        ] {
+            let tables = tables_of(state_count);
+            let lent = mebibytes_a_judge_needs(&tables, requested);
+            let states_lent_for =
+                requested.min(usize::try_from(state_count).expect("装得进 usize"));
+            assert!(
+                states_in_flight_within(
+                    BINDING_LIMIT,
+                    lent + super::MEBIBYTES_LEFT_FOR_THE_DRIVER,
+                    &tables
+                ) >= states_lent_for,
+                "{state_count} 个状态、要 {requested} 个一次：借了 {lent} MiB，判器开出来的草稿区装不下借的时候算进去的 {states_lent_for} 个"
+            );
+        }
+    }
+
     /// 量宽度照 2026-09-29 两张卡上量到的数（固定脚本到回退、展开 16、流的中间，一档加一条 2170 个状态的缓冲）：
     /// 5090 到 23 870 个时每个状态的用时翻倍，停，取 21 700；5060 Ti 到 8 680 个时一次 0.324 秒，停，取 6 510。
     #[test]
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
index 3b0023f..c556a6a 100644
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -7,6 +7,7 @@
 use singlefs_checker::{
     crc32_castagnoli_table, unit_checksum_layout, UNIT_HEADER_CHECKSUM_BYTE_OFFSET,
 };
+use singlefs_harness::sha256::DIGEST_BYTES;
 
 /// 空闲显存低于这个数（MiB）的卡不用：本机有卡被别的服务占着时只剩几百 MiB，起上下文都难（E163 M2 那张卡 141 MiB 起不来）。
 pub const MINIMUM_FREE_MEMORY_MEBIBYTES: u64 = 2048;
@@ -61,7 +62,17 @@ fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
 /// 没给额度的卡，崩溃放量最多用它空闲显存的几分之几（分子、分母）：剩下的留给卡上别的进程。
 const SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA: (u64, u64) = (3, 4);
 
-/// 一张起好上下文的卡。
+/// 一个设备上按源码摘要编好的几组管线：每个摘要一格，第一个要它的线程编、同时要它的等那一次编完（[`GpuCard::pipelines_compiled_once_on_this_device`]）。
+type CompiledPipelinesBySourceDigest = std::sync::Mutex<
+    std::collections::BTreeMap<
+        [u8; DIGEST_BYTES],
+        std::sync::Arc<std::sync::OnceLock<Vec<wgpu::ComputePipeline>>>,
+    >,
+>;
+
+/// 一张起好上下文的卡。克隆出来的与原来的共用同一个设备、队列与编好的管线（wgpu 的这几样克隆的是句柄）：
+/// 一个进程里几条 GPU 用例共用一张卡就靠克隆（[`gpu_test_start::GpuCardPool`]），每张卡只起一个设备。
+#[derive(Clone)]
 pub struct GpuCard {
     pub name: String,
     pub pci_bus_identifier: String,
@@ -74,6 +85,8 @@ pub struct GpuCard {
     device: wgpu::Device,
     queue: wgpu::Queue,
     pipeline: wgpu::ComputePipeline,
+    /// 这个设备上编好的管线，几个克隆共用：同一套源码在同一个设备上只编一次。
+    compiled_pipelines_by_source_digest: std::sync::Arc<CompiledPipelinesBySourceDigest>,
 }
 
 /// `nvidia-smi` 报的一张卡：序号与空闲显存 MiB。
@@ -198,6 +211,7 @@ pub fn usable_gpu_cards() -> Vec<GpuCard> {
                     device,
                     queue,
                     pipeline,
+                    compiled_pipelines_by_source_digest: std::sync::Arc::default(),
                 })
             })
             .collect();
@@ -269,6 +283,29 @@ impl GpuCard {
         &self.queue
     }
 
+    /// 这个设备上按 `source_digest` 编好的那组管线：这个设备（这张卡与它的每个克隆）上第一次要这个摘要时调 `compile` 编，
+    /// 别的线程同时要同一个摘要就等这一次编完、拿同一组；以后再要直接拿。管线绑在设备上，另起的设备（另一次 [`usable_gpu_cards`]）另有一份。
+    /// 冷编一套判器内核要几分钟、驱动编的时候吃几 GiB 主机内存：几十条用例共用一张卡时，每条各编一遍既慢又撞主机内存上限。
+    ///
+    /// # Panics
+    /// `compile` panic：原样传出，这一格照旧空着，下一个要这个摘要的再编一次。
+    #[must_use]
+    pub fn pipelines_compiled_once_on_this_device(
+        &self,
+        source_digest: [u8; DIGEST_BYTES],
+        compile: impl FnOnce() -> Vec<wgpu::ComputePipeline>,
+    ) -> Vec<wgpu::ComputePipeline> {
+        // 持锁只为取出这个摘要那一格（不在锁里编），没有会 panic 到一半的写：别的线程 panic 留下的毒不影响这张表，照用
+        let compiled_once = {
+            let mut by_source_digest = self
+                .compiled_pipelines_by_source_digest
+                .lock()
+                .unwrap_or_else(std::sync::PoisonError::into_inner);
+            std::sync::Arc::clone(by_source_digest.entry(source_digest).or_default())
+        };
+        compiled_once.get_or_init(compile).clone()
+    }
+
     /// 一批字节段（每段不超过 32 KiB）在这张卡上算 CRC-32C，按次序交回。
     ///
     /// # Panics
@@ -376,15 +413,16 @@ impl GpuCard {
             );
         }
         encoder.copy_buffer_to_buffer(&results, 0, &staging, 0, result_bytes);
-        self.queue.submit(std::iter::once(encoder.finish()));
+        let submission = self.queue.submit(std::iter::once(encoder.finish()));
         let slice = staging.slice(..);
         let (sender, receiver) = std::sync::mpsc::channel();
         slice.map_async(wgpu::MapMode::Read, move |result| {
             sender.send(result).expect("回调送得出去")
         });
+        // 只等这一次提交：几条用例共用这张卡的设备时，不等别的用例提交的活
         device
             .poll(wgpu::PollType::Wait {
-                submission_index: None,
+                submission_index: Some(submission),
                 timeout: None,
             })
             .expect("等 GPU 算完");
@@ -814,6 +852,267 @@ pub mod gpu_test_start {
             )
         })
     }
+
+    /// 卡池挂 `verdict-store`：照显卡配置挑这台机器的卡要的那几样（`GPU_CARDS_BY_PRIORITY` 的读法、机器角色、点名的卡少没少）
+    /// 住在 `crash_amplification`，它只随 `verdict-store` 编；只开 `gpu` 的测试二进制用不了卡池，照旧经
+    /// [`usable_cards_or_report_not_run`] 与 [`first_usable_card_or_report_not_run`] 取卡。
+    #[cfg(feature = "verdict-store")]
+    pub use card_pool::{gpu_card_pool_or_report_not_run, GpuCardLease, GpuCardPool};
+
+    #[cfg(feature = "verdict-store")]
+    mod card_pool {
+        use super::super::{gpu_cards_by_priority, usable_gpu_cards, GpuCard};
+        use super::{
+            decide_gpu_test_start, enumerate_vulkan_adapters, not_run_line,
+            read_pci_nvidia_display_controllers_on_the_bus, GpuTestStart, PCI_BUS_DIRECTORY,
+        };
+        use crate::crash_amplification::{
+            cards_by_priority_from_configuration_or_environment, configured_cards_not_usable,
+            CardHost,
+        };
+        use crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
+        use singlefs_harness::gpu_memory_ledger::{GpuMemoryBooking, GpuMemoryLedger};
+
+        /// 一个进程里 GPU 用例共用的卡：这台机器点名的每张卡起一次设备，各一本显存账（[`GpuMemoryLedger`]）；
+        /// 用例按自己实际要的量借（[`GpuCardPool::lease_card_waiting_for_room`]），借据丢掉时还。
+        /// 一张卡的账 = 这张卡给崩溃放量的额度（[`GpuCard::memory_budget_mebibytes`]）减一份
+        /// [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]：驱动的上下文与编好的内核每张卡（每个设备）付一次，不按用例各付一份。
+        pub struct GpuCardPool {
+            cards: Vec<GpuCard>,
+            ledger: GpuMemoryLedger,
+        }
+
+        /// 借来的一张卡：[`GpuCardLease::card`] 与池里那张共用设备、编好的内核，显存额度改成借的量加 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]
+        /// （判器照「额度 − 表 − 那一份」开草稿区，开出来正好是借的那么多）。丢掉时先让设备收掉已丢的缓冲，再销账。
+        pub struct GpuCardLease {
+            card: GpuCard,
+            booking: GpuMemoryBooking<'static>,
+        }
+
+        impl GpuCardLease {
+            #[must_use]
+            pub fn card(&self) -> &GpuCard {
+                &self.card
+            }
+
+            /// 借的显存（MiB）。
+            #[must_use]
+            pub fn mebibytes(&self) -> u64 {
+                self.booking.mebibytes()
+            }
+        }
+
+        impl Drop for GpuCardLease {
+            fn drop(&mut self) {
+                // wgpu 丢掉的缓冲要等设备下一次收拾才还给驱动：销账之前收一次，免得账上还了、卡上还占着，下一个借的建缓冲时撞满
+                if let Err(error) = self.card.device().poll(wgpu::PollType::Poll) {
+                    eprintln!(
+                        "GPU_CARD_LEASE_RETURN_POLL_FAILED index={} error={error:?}",
+                        self.card.index
+                    );
+                }
+            }
+        }
+
+        impl GpuCardPool {
+            /// 池里每张卡存储缓冲绑定上限里最小的那个（用例借之前按它估一条草稿缓冲装几个状态）。
+            #[must_use]
+            pub fn smallest_max_storage_buffer_binding_size(&self) -> u64 {
+                self.cards
+                    .iter()
+                    .map(|card| card.device().limits().max_storage_buffer_binding_size)
+                    .min()
+                    .expect(
+                        "池里至少一张卡：没有卡的机器上起不出池（gpu_card_pool_or_report_not_run）",
+                    )
+            }
+
+            /// 借 `need_mebibytes`：照卡的优先级记在第一张放得下的卡上（前一张放满了才用下一张）；此刻每张都放不下就等别的用例还。
+            /// 每借一次打一行 `GPU_CARD_LEASE test=… index=… mebibytes=… capacity_mebibytes=…`。
+            ///
+            /// # Panics
+            /// 要的比每张卡的账都大：配了 GPU 就一定用 GPU，放不下判红，不悄悄换小。
+            #[must_use]
+            pub fn lease_card_waiting_for_room(
+                &'static self,
+                test_name: &str,
+                need_mebibytes: u64,
+            ) -> GpuCardLease {
+                let booking = self
+                    .ledger
+                    .book_waiting_for_room(need_mebibytes)
+                    .unwrap_or_else(|refusal| {
+                        panic!(
+                            "{test_name}：要借 {} MiB，池里最大的一张卡的账只有 {} MiB（每张卡的账 {:?}）：这台机器的卡放不下这条用例，\
+                             腾出显存、给卡更大的额度（GPU_CARDS_BY_PRIORITY），或缩小用例",
+                            refusal.need_mebibytes,
+                            refusal.largest_capacity_mebibytes,
+                            self.ledger.card_capacities_mebibytes()
+                        )
+                    });
+                let position = booking.card_position();
+                let mut card = self.cards[position].clone();
+                card.memory_quota_mebibytes = Some(need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER);
+                eprintln!(
+                    "GPU_CARD_LEASE test={test_name} card={:?} index={} mebibytes={need_mebibytes} capacity_mebibytes={}",
+                    card.name,
+                    card.index,
+                    self.ledger.card_capacities_mebibytes()[position]
+                );
+                GpuCardLease { card, booking }
+            }
+        }
+
+        /// 起池那一次的结局：进程里只判一次，之后每条用例照它。
+        enum GpuCardPoolStart {
+            Started(GpuCardPool),
+            NotRunNoGpuAdapter { reason: String },
+            CardsPresentButNotUsable { reason: String },
+        }
+
+        /// 这台机器给卡池点名的卡。
+        struct CardsNamedForThisMachine {
+            cards: Vec<GpuCard>,
+            /// 配置点了名、本机却起不来的卡（序号，按配置的次序）。
+            named_but_not_started: Vec<u32>,
+            /// 配置读不出、写法不对、没给这台机器点卡：一张卡都不起，原因在这里。
+            configuration_refusal: Option<String>,
+        }
+
+        /// 照显卡配置挑这台机器的卡：`GPU_CARDS_BY_PRIORITY`（环境变量，没设读 `SINGLEFS_GPU_CARDS_CONFIG` 指的显卡配置里的同名键）
+        /// 里这台机器（`SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE`，没设是本机）的那几项，照配置的次序与额度起（与崩溃放量取卡同一个约定）；
+        /// 都没配就是本机能用的卡，按空闲显存从大到小、不设额度。
+        fn cards_named_for_this_machine() -> CardsNamedForThisMachine {
+            let refused = |reason: String| CardsNamedForThisMachine {
+                cards: Vec::new(),
+                named_but_not_started: Vec::new(),
+                configuration_refusal: Some(reason),
+            };
+            let host = match CardHost::of_this_process() {
+                Ok(host) => host,
+                Err(reason) => return refused(reason),
+            };
+            let configuration_file = match std::env::var_os("SINGLEFS_GPU_CARDS_CONFIG") {
+                Some(text) => {
+                    let path = std::path::PathBuf::from(text);
+                    if !path.is_file() {
+                        return refused(format!(
+                            "SINGLEFS_GPU_CARDS_CONFIG 指的配置不在：{}",
+                            path.display()
+                        ));
+                    }
+                    Some(path)
+                }
+                None => None,
+            };
+            let configured = match cards_by_priority_from_configuration_or_environment(
+                configuration_file.as_deref(),
+            ) {
+                Ok(configured) => configured,
+                Err(reason) => return refused(reason),
+            };
+            match configured {
+                Some(configured) => {
+                    let mine: Vec<(u32, u64)> = configured
+                        .iter()
+                        .filter(|card| card.host == host)
+                        .map(|card| (card.index, card.memory_quota_mebibytes))
+                        .collect();
+                    if mine.is_empty() {
+                        return refused(format!(
+                            "配置的 GPU_CARDS_BY_PRIORITY 里没有 {} 这台机器的卡",
+                            host.name()
+                        ));
+                    }
+                    let cards = gpu_cards_by_priority(&mine);
+                    let started: Vec<u32> = cards.iter().map(|card| card.index).collect();
+                    CardsNamedForThisMachine {
+                        named_but_not_started: configured_cards_not_usable(&mine, &started),
+                        cards,
+                        configuration_refusal: None,
+                    }
+                }
+                None => CardsNamedForThisMachine {
+                    cards: usable_gpu_cards(),
+                    named_but_not_started: Vec::new(),
+                    configuration_refusal: None,
+                },
+            }
+        }
+
+        fn start_the_gpu_card_pool() -> GpuCardPoolStart {
+            let named = cards_named_for_this_machine();
+            let decision = decide_gpu_test_start(
+                &enumerate_vulkan_adapters(),
+                &read_pci_nvidia_display_controllers_on_the_bus(std::path::Path::new(
+                    PCI_BUS_DIRECTORY,
+                )),
+                named.cards.len(),
+            );
+            let configuration_note = named
+                .configuration_refusal
+                .as_ref()
+                .map_or(String::new(), |reason| format!("（显卡配置：{reason}）"));
+            match decision {
+                GpuTestStart::RunOnTheUsableCards => {
+                    if named.named_but_not_started.is_empty() {
+                        let capacities = named
+                            .cards
+                            .iter()
+                            .map(|card| {
+                                card.memory_budget_mebibytes()
+                                    .saturating_sub(MEBIBYTES_LEFT_FOR_THE_DRIVER)
+                            })
+                            .collect();
+                        GpuCardPoolStart::Started(GpuCardPool {
+                            cards: named.cards,
+                            ledger: GpuMemoryLedger::with_card_capacities(capacities),
+                        })
+                    } else {
+                        // 点名的卡少一张都不跑（用户 2026-09-29 定：配了 GPU 就要它干活，不行就明确报错，不接受少用几张的默认降级）
+                        GpuCardPoolStart::CardsPresentButNotUsable {
+                            reason: format!(
+                                "配置点名给这台机器的卡里，序号 {:?} 用不了（上面 GPU_CARD_NOT_USABLE 那几行）：点名的卡每一张都要用得上",
+                                named.named_but_not_started
+                            ),
+                        }
+                    }
+                }
+                GpuTestStart::NotRunNoGpuAdapter { reason } => {
+                    GpuCardPoolStart::NotRunNoGpuAdapter { reason }
+                }
+                GpuTestStart::AdapterPresentButNoCardStarted { reason }
+                | GpuTestStart::NvidiaCardOnPciButVulkanEnumeratesNone { reason }
+                | GpuTestStart::PciBusVisibleButListingUnreadable { reason } => {
+                    GpuCardPoolStart::CardsPresentButNotUsable {
+                        reason: format!("{reason}{configuration_note}"),
+                    }
+                }
+            }
+        }
+
+        static GPU_CARD_POOL: std::sync::OnceLock<GpuCardPoolStart> = std::sync::OnceLock::new();
+
+        /// 这个进程的卡池：第一条来要的用例起池（只在这时判一次有没有卡、读一次 `nvidia-smi`、每张卡起一次设备，
+        /// 挑卡照 [`cards_named_for_this_machine`]），之后每条用例拿同一个。本机没有 NVIDIA 卡时每条来要的用例各打一行
+        /// [`not_run_line`]、交回 `None`（调用的用例当场返回）。先判有没有卡、建好表知道要多少显存之后再借（[`GpuCardPool::lease_card_waiting_for_room`]）。
+        ///
+        /// # Panics
+        /// 有卡而用不上（同 [`super::usable_cards_or_report_not_run`]）；显卡配置写错、没给这台机器点卡、点名的卡少一张起不来。
+        #[must_use]
+        pub fn gpu_card_pool_or_report_not_run(test_name: &str) -> Option<&'static GpuCardPool> {
+            match GPU_CARD_POOL.get_or_init(start_the_gpu_card_pool) {
+                GpuCardPoolStart::Started(pool) => Some(pool),
+                GpuCardPoolStart::NotRunNoGpuAdapter { reason } => {
+                    println!("{}", not_run_line(test_name, reason));
+                    None
+                }
+                GpuCardPoolStart::CardsPresentButNotUsable { reason } => {
+                    panic!("{test_name}：{reason}")
+                }
+            }
+        }
+    }
 }
 
 /// 代码三方 test-script-align-code-r2 判决 J2 G1：看不见 PCI 总线（总线目录整个不在）算没卡、报本次未跑；
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 146f39f..b8063cb 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -48,7 +48,10 @@
 //! （墓碑树那条在删除中间态的流上），树表里两条种类 40（≥ 32）一条，都整份比。
 //! 要本机一张空闲显存够的 NVIDIA 独显：本机一张 NVIDIA 独显适配器都没有时用卡的用例打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、
 //! 返回成功；有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）。要 `verdict-store,gpu` 特性；
-//! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（默认 8）、显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（没设不定额度）、
+//! 卡经这个进程的卡池取（`gpu_test_start::gpu_card_pool_or_report_not_run`：每张卡一个设备、内核每张卡编一次、一张卡一本显存账，
+//! 卡与额度照 `GPU_CARDS_BY_PRIORITY` 里这台机器那几项，没配就是本机能用的卡）：每条用例建好表之后按它的判器实际要的显存借，
+//! 照卡的先后装满前一张才用下一张，放不下就等别的用例还，比每张卡的账都大判红（几十条用例并行不再各按空闲显存的 3/4 互抢）；
+//! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（默认 8）、显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（只对量速度那条用例作数；设了它，经卡池取卡的对拍用例判红）、
 //! CPU 判器线程数 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`（没设取本机可用并行度）：设了而认不出（线程数 0 也算）就 panic，不退回没设时的那一档。
 #![cfg(all(feature = "verdict-store", feature = "gpu"))]
 
@@ -86,7 +89,7 @@ use singlefs_checker_tier::crash_amplification::{
 };
 use singlefs_checker_tier::crash_identity::CoverageReport;
 use singlefs_checker_tier::crash_judge_dispatch::{
-    states_per_scratch_lane, GpuCrashJudge, PackedJudgeTables,
+    mebibytes_a_judge_needs, states_per_scratch_lane, GpuCrashJudge, PackedJudgeTables,
 };
 use singlefs_checker_tier::crash_judge_gpu::{
     gpu_judge_digest, pool_checker_violated_bits, verdict_bits, JudgeKernelSources,
@@ -95,7 +98,8 @@ use singlefs_checker_tier::crash_judge_gpu::{
 };
 use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
 use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
-    first_usable_card_or_report_not_run, usable_cards_or_report_not_run,
+    first_usable_card_or_report_not_run, gpu_card_pool_or_report_not_run,
+    usable_cards_or_report_not_run, GpuCardPool,
 };
 use singlefs_checker_tier::gpu_unit_checks::GpuCard;
 use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
@@ -192,10 +196,11 @@ fn a_zero_crash_judge_cpu_threads_value_panics_naming_the_variable_and_the_value
     let _threads = cpu_threads_from_the_value_of_the_variable(Some("0"));
 }
 
-/// 取一张卡：空闲显存最大的那张。`SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES` 给了就当这张卡的显存额度（量速度时照配置的额度量；
+/// 量速度的那条用例取一整张卡：空闲显存最大的那张，不经卡池记账（它量的就是一张卡独占时多快）。
+/// `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES` 给了就当这张卡的显存额度（量速度时照配置的额度量；
 /// 写错就 panic，[`card_quota_mebibytes_from_the_value_of_the_variable`]）。
 /// 本机没有 NVIDIA 独显适配器时打「本次未跑」那一行、交回 `None`，用例当场返回；有适配器而一张卡都没起来判红。
-fn one_usable_card_or_report_not_run(test_name: &str) -> Option<GpuCard> {
+fn the_whole_emptiest_card_or_report_not_run(test_name: &str) -> Option<GpuCard> {
     let quota = card_quota_mebibytes_from_the_value_of_the_variable(
         process_environment_variable(CARD_QUOTA_ENVIRONMENT_VARIABLE).as_deref(),
     );
@@ -206,6 +211,30 @@ fn one_usable_card_or_report_not_run(test_name: &str) -> Option<GpuCard> {
     Some(card)
 }
 
+/// 对拍用例取这个进程的卡池（`gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run`：进程里只判一次有没有卡，
+/// 卡照 `GPU_CARDS_BY_PRIORITY` 与机器角色挑）；建好表、知道要多少显存之后再借（[`GpuCardPool::lease_card_waiting_for_room`]）。
+/// 本机没有 NVIDIA 独显适配器时打「本次未跑」那一行、交回 `None`，用例当场返回；有卡而用不上判红。
+///
+/// # Panics
+/// 设了 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`：有了卡池之后它对借卡当什么（每张卡的账的上限，还是每条用例的上限）还没定，
+/// 不悄悄挑一种；卡的额度照 `GPU_CARDS_BY_PRIORITY` 给，这个变量只对量速度那条用例作数。
+fn the_card_pool_or_report_not_run(test_name: &str) -> Option<&'static GpuCardPool> {
+    let quota = card_quota_mebibytes_from_the_value_of_the_variable(
+        process_environment_variable(CARD_QUOTA_ENVIRONMENT_VARIABLE).as_deref(),
+    );
+    assert!(
+        quota.is_none(),
+        "{test_name}：设了 {CARD_QUOTA_ENVIRONMENT_VARIABLE}={quota:?}。有了卡池之后这个变量对借卡当什么还没定（交主 agent 定），\
+         对拍用例不认它：卡的额度写进 GPU_CARDS_BY_PRIORITY；它只对 gpu_judge_throughput_by_states_per_dispatch 作数"
+    );
+    gpu_card_pool_or_report_not_run(test_name)
+}
+
+/// 一条草稿缓冲装几个状态，按卡池里绑定上限最小的那张卡估（借卡之前还不知道借到哪张）。
+fn states_of_one_scratch_lane_on_every_card_of(cards: &GpuCardPool) -> usize {
+    states_per_scratch_lane(cards.smallest_max_storage_buffer_binding_size())
+}
+
 struct RecordedFlow {
     name: &'static str,
     base: MemoryPool,
@@ -1836,10 +1865,10 @@ fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
     flow: &RecordedFlow,
     invariant: &'static str,
 ) {
-    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, flow, expand_up_to());
     assert!(
         comparison.cpu_checker_red_invariants.contains(invariant),
         "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
@@ -1951,7 +1980,11 @@ struct Comparison {
     checker_mismatches: Vec<String>,
 }
 
-fn compare_on_every_state(card: &GpuCard, flow: &RecordedFlow, limit: usize) -> Comparison {
+fn compare_on_every_state(
+    cards: &'static GpuCardPool,
+    flow: &RecordedFlow,
+    limit: usize,
+) -> Comparison {
     let expansion = move |_segment_index: usize, segment: &[usize]| {
         if segment.len() <= limit {
             Layer0SegmentExpansion::EveryProperSubset
@@ -1959,13 +1992,15 @@ fn compare_on_every_state(card: &GpuCard, flow: &RecordedFlow, limit: usize) ->
             Layer0SegmentExpansion::NotExpanded
         }
     };
-    compare_on_every_state_of_these_segments(card, flow, &flow.segments, &expansion)
+    compare_on_every_state_of_these_segments(cards, flow, &flow.segments, &expansion)
 }
 
 /// 同 [`compare_on_every_state`]，切段与每段怎么展开由调用方给（`segments` 与 `flow.segments` 罩同一组写、只是分法不同时，
 /// 状态仍是这条流全量枚举里的状态）。
+/// 表建好之后才向卡池借卡，借的量是这条流的判器实际要的（[`mebibytes_a_judge_needs`]，一次派活至多一条草稿缓冲那么宽）；
+/// GPU 判完就还，CPU 那一侧判的时候不占着卡。
 fn compare_on_every_state_of_these_segments(
-    card: &GpuCard,
+    cards: &'static GpuCardPool,
     flow: &RecordedFlow,
     segments: &[Vec<usize>],
     expansion: &(dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion + Sync),
@@ -1990,10 +2025,19 @@ fn compare_on_every_state_of_these_segments(
         words.version_bytes.len(),
         tables_started.elapsed()
     );
+    let packed = PackedJudgeTables::of(&tables);
+    let states_per_dispatch = states_of_one_scratch_lane_on_every_card_of(cards);
+    let lease = cards.lease_card_waiting_for_room(
+        flow.name,
+        mebibytes_a_judge_needs(&packed, states_per_dispatch),
+    );
     let gpu_started = Instant::now();
-    let judge = GpuCrashJudge::new(card, &tables);
+    let judge =
+        GpuCrashJudge::on_card_with_packed_tables(lease.card(), &packed, states_per_dispatch);
     let gpu = judge.judge_states(0..state_count);
     let gpu_elapsed = gpu_started.elapsed();
+    drop(judge);
+    drop(lease);
     assert_eq!(gpu.len(), usize::try_from(state_count).expect("装得进"));
     // CPU 那一侧按线程分段判（`SINGLEFS_CRASH_JUDGE_CPU_THREADS`，没设取本机可用并行度），合并按序号排。
     // 段切得比线程数细（每个线程八段、轮着领）：靠后的状态历史长、判得慢，按线程数等分时最后一段拖住整趟
@@ -2173,7 +2217,7 @@ fn compare_on_every_state_of_these_segments(
 
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows() {
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows",
     ) else {
         return;
@@ -2200,7 +2244,7 @@ fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows() {
         )
         .collect();
     for (flow, must_red_invariant) in every_flow {
-        let comparison = compare_on_every_state(&card, flow, limit);
+        let comparison = compare_on_every_state(cards, flow, limit);
         if let Some(invariant) = must_red_invariant {
             assert!(
                 comparison.cpu_checker_red_invariants.contains(invariant),
@@ -2306,12 +2350,12 @@ fn the_gpu_judge_tells_apart_two_versions_of_a_root_slot_that_differ_only_past_t
             .map(|version| version.combination)
             .collect::<Vec<_>>()
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_tells_apart_two_versions_of_a_root_slot_that_differ_only_past_the_thirty_second_write",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, EXPAND_UP_TO);
+    let comparison = compare_on_every_state(cards, &flow, EXPAND_UP_TO);
     assert!(
         comparison.mismatches.is_empty(),
         "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
@@ -2400,7 +2444,7 @@ fn resealed_publish_record_names_units_that_match_on_disk() {
 #[test]
 fn verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several() {
     const EXPAND_UP_TO: usize = 16;
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several",
     ) else {
         return;
@@ -2424,11 +2468,22 @@ fn verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several(
     let tables = build_judge_tables(&judge_flow).expect("建得出表");
     let state_count = tables.state_count;
     let packed = PackedJudgeTables::of(&tables);
-    let states_per_lane =
-        states_per_scratch_lane(card.device().limits().max_storage_buffer_binding_size);
-    let within_one_lane = GpuCrashJudge::on_card_with_packed_tables(&card, &packed, 256);
+    // 两个判器同时活着：借的是窄的那一档（256 个）加宽的那一档（三条缓冲）各自要的，两份表各算一份（约 6.45 GiB）
+    let states_per_lane = states_of_one_scratch_lane_on_every_card_of(cards);
+    let lease = cards.lease_card_waiting_for_room(
+        "verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several",
+        mebibytes_a_judge_needs(&packed, 256)
+            + mebibytes_a_judge_needs(&packed, 3 * states_per_lane),
+    );
+    let card = lease.card();
+    assert_eq!(
+        states_per_scratch_lane(card.device().limits().max_storage_buffer_binding_size),
+        states_per_lane,
+        "借到的卡一条草稿缓冲装的状态数与借之前按池里最小的绑定上限估的不同：借的量对不上这张卡"
+    );
+    let within_one_lane = GpuCrashJudge::on_card_with_packed_tables(card, &packed, 256);
     let across_lanes =
-        GpuCrashJudge::on_card_with_packed_tables(&card, &packed, 3 * states_per_lane);
+        GpuCrashJudge::on_card_with_packed_tables(card, &packed, 3 * states_per_lane);
     println!(
         "CRASH_JUDGE_LANES states={state_count} states_per_lane={states_per_lane} states_per_dispatch_within_one_lane={} states_per_dispatch_across_lanes={}",
         within_one_lane.states_per_dispatch(),
@@ -2503,7 +2558,7 @@ fn a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both
             1,
         ),
     };
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels",
     ) else {
         return;
@@ -2528,10 +2583,16 @@ fn a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both
     let tables = build_judge_tables(&judge_flow).expect("建得出表");
     let state_count = tables.state_count;
     let packed = PackedJudgeTables::of(&tables);
-    let unplanted = GpuCrashJudge::on_card_with_packed_tables(&card, &packed, usize::MAX)
+    // 两个判器先后建；前一个的缓冲什么时候还给驱动看 wgpu 收拾的时机，借两份
+    let lease = cards.lease_card_waiting_for_room(
+        "a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels",
+        2 * mebibytes_a_judge_needs(&packed, usize::MAX),
+    );
+    let card = lease.card();
+    let unplanted = GpuCrashJudge::on_card_with_packed_tables(card, &packed, usize::MAX)
         .judge_states(0..state_count);
     let planted = GpuCrashJudge::on_card_compiling_these_kernel_sources(
-        &card,
+        card,
         &packed,
         usize::MAX,
         &planted_sources,
@@ -2667,12 +2728,11 @@ fn the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest
     const EXPAND_UP_TO: usize = 16;
     const BLOCKS_BEFORE_THE_STOP: usize = 5;
     const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs";
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest",
     ) else {
         return;
     };
-    let card = std::sync::Arc::new(card);
     let recorded = record_fixed_script_through_rollback("crash-judge-gpu-stop");
     let expansion = move |_segment_index: usize, segment: &[usize]| {
         if segment.len() <= EXPAND_UP_TO {
@@ -2701,6 +2761,27 @@ fn the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest
             })
             .collect(),
     };
+    // 借一个定量：一条草稿缓冲那么宽的判器加表（判别力在叫停与续判，不在宽度）；放量路径照借来的卡的额度自己定一次派活带几个。
+    // 表先在这里建一次知道多大（放量路径里面照同一条流再建一次，这里没有点被忽略，两次建的是同一份）
+    let tables_of_the_flow = PackedJudgeTables::of(
+        &build_judge_tables(&JudgeFlow {
+            base: flow.base,
+            writes: flow.writes,
+            segments: flow.segments,
+            judged_root_index: flow.judged_root_index,
+            versions: flow.versions,
+            expansion: &expansion,
+        })
+        .expect("建得出表"),
+    );
+    let lease = cards.lease_card_waiting_for_room(
+        "the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest",
+        mebibytes_a_judge_needs(
+            &tables_of_the_flow,
+            states_of_one_scratch_lane_on_every_card_of(cards),
+        ),
+    );
+    let card = std::sync::Arc::new(lease.card().clone());
     let plan = plan_crash_points(&flow);
     let version = JudgeVersion([7u8; 32]);
     let directory = std::env::temp_dir().join(format!(
@@ -2882,7 +2963,7 @@ fn dispatch_time_at_each_scratch_size_with_the_whole_card_given() {
 #[ignore = "量速度用：要一张空闲显存够的独显，几种派活粒度各把整条流判一遍"]
 fn gpu_judge_throughput_by_states_per_dispatch() {
     let Some(card) =
-        one_usable_card_or_report_not_run("gpu_judge_throughput_by_states_per_dispatch")
+        the_whole_emptiest_card_or_report_not_run("gpu_judge_throughput_by_states_per_dispatch")
     else {
         return;
     };
@@ -3160,12 +3241,12 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_refused_value_wins_over_a_wrong_safe
     let flow = record_new_pool_with_a_refused_slot_spacing_and_a_safety_factor_of_two(
         "crash-judge-gpu-i713-over-i81",
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_refused_value_wins_over_a_wrong_safety_factor_in_the_same_slot",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.cpu_checker_red_invariants.contains("I-7.13")
             && !comparison.cpu_checker_red_invariants.contains("I-8.1"),
@@ -3241,12 +3322,12 @@ fn the_gpu_judge_agrees_with_the_cpu_when_a_version_references_a_unit_born_no_la
 fn the_gpu_judge_agrees_with_the_cpu_that_a_rollback_publish_inheriting_its_user_visible_trees_whole_keeps_the_appearance_rule(
 ) {
     let flow = record_fixed_script_through_rollback("crash-judge-gpu-i35-holds");
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_rollback_publish_inheriting_its_user_visible_trees_whole_keeps_the_appearance_rule",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.cpu_checker_held_invariants.contains("I-3.5")
             && !comparison.cpu_checker_red_invariants.contains("I-3.5"),
@@ -3273,12 +3354,12 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_unmount_marker_sets_ano
     let flow = record_new_pool_with_a_last_record_whose_unmount_marker_sets_another_bit(
         "crash-judge-gpu-rr1",
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_unmount_marker_sets_another_bit_is_not_applied",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.mismatches.is_empty(),
         "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
@@ -3347,7 +3428,7 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_last_record_carries_another_unmoun
 /// 新池新建文件那条流的每个状态（不种假位）上，违例位在池级 checker 条数之外的那些位全是 0、判得了——高 64 位那两个字只要有一个没写，这条红。
 #[test]
 fn the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count() {
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count",
     ) else {
         return;
@@ -3371,7 +3452,14 @@ fn the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invar
     })
     .expect("建得出表");
     let state_count = tables.state_count;
-    let verdicts = GpuCrashJudge::new(&card, &tables).judge_states(0..state_count);
+    let packed = PackedJudgeTables::of(&tables);
+    let lease = cards.lease_card_waiting_for_room(
+        "the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count",
+        mebibytes_a_judge_needs(&packed, usize::MAX),
+    );
+    let verdicts = GpuCrashJudge::on_card_with_packed_tables(lease.card(), &packed, usize::MAX)
+        .judge_states(0..state_count);
+    drop(lease);
     let invariant_count = IMPLEMENTED_INVARIANTS.len();
     let states_with_bits_past_the_invariant_count: Vec<String> = verdicts
         .iter()
@@ -3528,12 +3616,12 @@ fn the_gpu_judge_agrees_with_the_cpu_when_only_roots_outside_the_candidate_set_r
     let flow = record_rollback_with_another_birth_txg_recorded_only_outside_the_candidate_set(
         "crash-judge-gpu-i928-outside-candidates",
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_when_only_roots_outside_the_candidate_set_record_another_birth_txg",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     let every_readable_root = invariant_bit("I-9.28");
     let candidate_set = invariant_bit("I-9.14");
     assert!(
@@ -3589,12 +3677,12 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_root_whose_tree_table_cannot_be_read
     let flow = record_rollback_with_the_tree_table_of_the_overwrite_before_it_unreadable(
         "crash-judge-gpu-i928-unreadable-tree-table",
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_root_whose_tree_table_cannot_be_read_does_not_redden_the_every_readable_root_birth_invariant",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         !comparison.cpu_checker_red_invariants.contains("I-9.28"),
         "流 {} 上 CPU 一个状态都不红 I-9.28（红过的：{:?}）",
@@ -3710,12 +3798,12 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_data_unit_claims_its_second_slot_and
     let flow = record_new_pool_with_an_overwrite_forging_an_index_node_header_on_the_second_slot_of_its_data_unit(
         "crash-judge-gpu-i78-claim",
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_data_unit_claims_its_second_slot_and_a_header_forged_there_is_not_counted",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         !comparison.cpu_checker_red_invariants.contains("I-7.8"),
         "流 {} 上 CPU 一个状态都不红 I-7.8（红过的：{:?}）",
@@ -3774,12 +3862,12 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_with_no_earlier_root_ins
         "new-pool-floor-raise-one-revolution-past-every-earlier-root",
         UnmountMarkerOfTheAppendedRoot::NotWritten,
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_judged",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         !comparison.cpu_checker_red_invariants.contains("I-7.9")
             && !comparison.cpu_checker_held_invariants.contains("I-7.9"),
@@ -3808,12 +3896,12 @@ fn the_gpu_judge_agrees_with_the_cpu_that_an_unmount_marked_floor_raise_with_no_
         "new-pool-unmount-marked-floor-raise-one-revolution-past-every-earlier-root",
         UnmountMarkerOfTheAppendedRoot::Written,
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_an_unmount_marked_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_reddened",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         !comparison.cpu_checker_red_invariants.contains("I-7.9"),
         "流 {} 上 CPU 一个状态都不红 I-7.9（红过的：{:?}）",
@@ -3878,12 +3966,12 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_takes_no_root_outside_th
     let flow = record_rollback_with_a_floor_raise_whose_window_anchored_at_the_newest_root_drops_an_older_root(
         "crash-judge-gpu-i79-image-window",
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_takes_no_root_outside_the_window_anchored_at_the_newest_root_of_the_image",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.cpu_checker_held_invariants.contains("I-7.9")
             && !comparison.cpu_checker_red_invariants.contains("I-7.9"),
@@ -4042,12 +4130,12 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish
 ) {
     let flow =
         record_rollback_with_the_lineage_root_naming_another_tree("crash-judge-gpu-lineage-root");
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish_names_another_tree",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.cpu_checker_red_invariants.contains("I-1.3"),
         "流 {} 上 CPU 池级 checker 要红 I-1.3（红过的：{:?}）",
@@ -4122,12 +4210,12 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entr
     let flow = record_new_pool_with_the_lineage_and_logged_operations_kinds_swapped(
         "crash-judge-gpu-i916-issuing-order",
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entries_break_the_issuing_order",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.cpu_checker_red_invariants.contains("I-9.16"),
         "流 {} 上 CPU 池级 checker 要红 I-9.16（红过的：{:?}）",
@@ -5404,12 +5492,12 @@ fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress()
         "crash-judge-gpu-deletion-in-progress",
         "deletion-through-the-first-batch",
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.mismatches.is_empty(),
         "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
@@ -5441,12 +5529,12 @@ fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_la
         "crash-judge-gpu-deletion-through-the-last-batch",
         "deletion-through-the-last-batch",
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.mismatches.is_empty(),
         "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
@@ -5563,7 +5651,7 @@ fn the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unex
             variant.missing_write_index
         );
     }
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unexpanded_segment_of_a_deletion_in_progress",
     ) else {
         return;
@@ -5571,7 +5659,7 @@ fn the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unex
     let mut failures = Vec::new();
     for variant in &variants {
         let comparison = compare_on_every_state_of_these_segments(
-            &card,
+            cards,
             &flow,
             &variant.segments,
             &expansion_of(variant),
@@ -5794,10 +5882,10 @@ fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
     flow: &RecordedFlow,
     invariants: &[&'static str],
 ) {
-    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, flow, expand_up_to());
     for invariant in invariants {
         assert!(
             comparison.cpu_checker_red_invariants.contains(invariant),
@@ -7339,10 +7427,10 @@ fn the_gpu_judge_leaves_to_the_cpu_an_instance_table_chain_longer_than_the_pages
         record_an_abandoned_root_whose_own_instance_table_is_a_chain_longer_than_the_gpu_walks(
             "crash-judge-gpu-i312-abandoned-root-instance-table-of-257-pages",
         );
-    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     let mut states_left_to_the_cpu_by_a_chain_limit = 0u64;
     let mut rows_red_on_a_state_left_to_the_cpu_by_a_chain_limit = false;
     let mut disagreements: Vec<String> = Vec::new();
@@ -7571,12 +7659,12 @@ fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_an_unlink_rolled_back_and
     let flow = record_new_inode_unlinked_rolled_back_and_unlinked_again(
         "crash-judge-gpu-unlink-rolled-back-and-unlinked-again",
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_an_unlink_rolled_back_and_unlinked_again",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.mismatches.is_empty(),
         "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
@@ -7613,10 +7701,10 @@ fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_wit
     flow: &RecordedFlow,
     identity_invariant: &'static str,
 ) {
-    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, flow, expand_up_to());
     assert!(
         comparison
             .cpu_checker_red_invariants
@@ -7722,12 +7810,12 @@ fn the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_objec
             "最新根的实例表两份都翻坏、候选集滤不掉被抛弃的 C：I-9.29 要报部分没判，实际 {other_verdict:?}"
         ),
     }
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.cpu_checker_red_invariants.contains("I-9.29"),
         "流 {} 上 C 还是最新根的状态 CPU 红 I-9.29（红过的：{:?}）",
@@ -7787,10 +7875,10 @@ fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that
             "{change:?}：整条落盘的镜像上 {invariant} 不是违例，实际 {verdict:?}"
         );
     }
-    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     for invariant in ["I-9.29", "I-3.14"] {
         assert!(
             comparison.cpu_checker_red_invariants.contains(invariant),
@@ -7897,12 +7985,12 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_second_instance_table_page_whose_poi
         "流 {}：主走读按读回那一份认「走过」、走到第 1 片（末片），候选集里没有实例 2 那一行，I-9.29 红，实际 {verdict:?}",
         flow.name
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_second_instance_table_page_whose_pointer_on_device_zero_names_the_first_page_is_walked_from_the_copy_read_back",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.cpu_checker_red_invariants.contains("I-9.29"),
         "流 {} 上 CPU 红 I-9.29（红过的：{:?}）",
@@ -7941,10 +8029,10 @@ fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_witho
         record_new_pool_with_the_inode_root_entry_of_type_zero_and_no_identity_over_a_leaf_container_whose_first_copy_is_stale(
             "crash-judge-gpu-i92-inode-root-entry-type-zero-first-leaf-copy-stale",
         );
-    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.cpu_checker_red_invariants.contains("I-9.2"),
         "流 {} 上 CPU 池级 checker 要红 I-9.2（红过的：{:?}）",
@@ -8379,10 +8467,10 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_is_a_
         "new-pool-inode-leaf-second-copy-stale-node",
         ChecksumOfTheSecondLocation::LeftAsTheLeafChecksum,
     );
-    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.cpu_checker_red_invariants.contains("I-2.1"),
         "流 {} 上 CPU 池级 checker 要红 I-2.1（红过的：{:?}）",
@@ -8436,10 +8524,10 @@ fn assert_the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_
             flow.name
         ),
     }
-    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.cpu_checker_red_invariants.contains("I-9.29"),
         "流 {} 上 C 还是最新根的状态 CPU 红 I-9.29（红过的：{:?}）",
@@ -8515,12 +8603,12 @@ fn the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_chain_of_as_many
             "crash-judge-gpu-i929-newest-instance-table-of-256-pages",
             InstanceTableChainPages::AsManyAsTheGpuWalks,
         );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_chain_of_as_many_pages_as_the_gpu_walks",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.cpu_checker_red_invariants.contains("I-9.29"),
         "流 {} 上 CPU 红 I-9.29（红过的：{:?}）",
@@ -8550,10 +8638,10 @@ fn the_gpu_judge_leaves_to_the_cpu_a_newest_instance_table_chain_longer_than_the
             "crash-judge-gpu-i929-newest-instance-table-of-257-pages",
             InstanceTableChainPages::OneMoreThanTheGpuWalks,
         );
-    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     let mut states_left_to_the_cpu_by_the_chain_limit = 0u64;
     let mut one_birth_red_on_a_state_left_to_the_cpu_by_the_chain_limit = false;
     let mut disagreements: Vec<String> = Vec::new();
@@ -8968,12 +9056,12 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_version_applied_only_by_its_records_
         "three-unit-file-applied-only-by-its-records-stale-first-record",
         &write_the_root_of_the_first_file_into_the_first_record_of_the_three_unit_publish,
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_version_applied_only_by_its_records_is_walked_from_its_last_record_when_the_first_carries_the_previous_root",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.mismatches.is_empty(),
         "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
@@ -9009,12 +9097,12 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_pointer_of_the_root_and
     let flow = record_new_pool_with_the_tree_table_pointer_of_the_publish_written_all_zero(
         "crash-judge-gpu-all-zero-tree-table-pointer",
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_pointer_of_the_root_and_of_its_record_is_all_zero",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     for invariant in ["I-2.1", "I-7.2", "I-4.1"] {
         assert!(
             comparison.cpu_checker_red_invariants.contains(invariant),
@@ -9048,12 +9136,12 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_records_of_a_version_applied_only_
             [JOURNAL_NEW_ROOT_SEGMENT_OFFSET..JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NODE_POINTER_BYTES]
             .fill(0);
     });
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_when_the_records_of_a_version_applied_only_by_them_carry_an_all_zero_tree_table_pointer",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     for invariant in ["I-2.1", "I-4.8", "I-7.4"] {
         assert!(
             comparison.cpu_checker_red_invariants.contains(invariant),
@@ -9168,12 +9256,12 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_l
         InvariantVerdict::Holds,
         "首条两份都没落、后两条落了：core 断号不施加 txg 4，CPU 不认它，I-5.4 在 txg 3 那一版上判过而成立"
     );
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.cpu_checker_red_invariants.contains("I-5.4"),
         "流 {} 上 CPU 池级 checker 要红 I-5.4（红过的：{:?}）",
@@ -9354,12 +9442,12 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_replay_stopped_by_a_named_unit_of_a_
             panic!("前缀停在 C 的点名单元上时 I-4.1 该报不适用，实际 {other_verdict:?}")
         }
     }
-    let Some(card) = one_usable_card_or_report_not_run(
+    let Some(cards) = the_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first",
     ) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert!(
         comparison.mismatches.is_empty(),
         "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
diff --git a/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs b/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs
index 966880e..f13ca2a 100644
--- a/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs
+++ b/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs
@@ -1,6 +1,7 @@
 //! checker 档模块：无（只读这个包的源码文本，不调它的任何模块）
 //! C604（代码三方 test-script-align-code-r2 判决 J3「补一条与硬件无关的源码扫描用例」）：GPU 用例取卡一律经
-//! `gpu_unit_checks::gpu_test_start` 那一个入口（`usable_cards_or_report_not_run`、`first_usable_card_or_report_not_run`）：
+//! `gpu_unit_checks::gpu_test_start` 那一个入口（`usable_cards_or_report_not_run`、`first_usable_card_or_report_not_run`，
+//! 与进程里共用一池卡的 `gpu_card_pool_or_report_not_run`）：
 //! 没卡的机器上它报「本次未跑」，有卡而用不上它判红。绕开它直接调 `usable_gpu_cards` 或 `gpu_cards_by_priority` 起卡的用例，
 //! 在有卡的机器上与经入口走同一条路，跑用例分不出；这一条扫源码，不要显卡。
 //! 扫的是 `tests/` 下每份 `.rs`（这一份除外）整份，与 `src/` 下每份 `.rs` 里顶格 `#[cfg(test)]` 下的模块与函数；
@@ -17,9 +18,10 @@ const CARD_TAKING_CALLS_THAT_BYPASS_THE_ENTRY: [&str; 2] =
     ["usable_gpu_cards(", "gpu_cards_by_priority("];
 
 /// 经入口取卡的调用。
-const CARD_TAKING_CALLS_THROUGH_THE_ENTRY: [&str; 2] = [
+const CARD_TAKING_CALLS_THROUGH_THE_ENTRY: [&str; 3] = [
     "usable_cards_or_report_not_run(",
     "first_usable_card_or_report_not_run(",
+    "gpu_card_pool_or_report_not_run(",
 ];
 
 /// 经入口判过「有卡、照跑」之后再起卡、准许绕开的几处：(相对包根的文件, 那一行去掉前后空白的代码, 为什么准许)。
@@ -213,6 +215,6 @@ fn every_gpu_test_takes_its_cards_through_gpu_test_start() {
     assert!(
         not_allowed.is_empty(),
         "这几处绕开 gpu_unit_checks::gpu_test_start 直接起卡（没卡的机器上报不出「本次未跑」、有卡而用不上判不了红），\
-         改成经 usable_cards_or_report_not_run 或 first_usable_card_or_report_not_run 取卡：{not_allowed:#?}"
+         改成经 gpu_card_pool_or_report_not_run（共用一池卡、按量借）、usable_cards_or_report_not_run 或 first_usable_card_or_report_not_run 取卡：{not_allowed:#?}"
     );
 }
diff --git a/crates/singlefs-harness/src/lib.rs b/crates/singlefs-harness/src/lib.rs
index 08cbf14..e5f12d7 100644
--- a/crates/singlefs-harness/src/lib.rs
+++ b/crates/singlefs-harness/src/lib.rs
@@ -15,6 +15,7 @@ use singlefs_core::block_device::{
 };
 
 pub mod fault_injection;
+pub mod gpu_memory_ledger;
 pub mod hexadecimal;
 pub mod history;
 pub mod memory_pool;
diff --git a/crates/singlefs-harness/src/gpu_memory_ledger.rs b/crates/singlefs-harness/src/gpu_memory_ledger.rs
new file mode 100644
index 0000000..19b528f
--- /dev/null
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -0,0 +1,311 @@
+//! GPU 卡上的显存怎么分：一个判器一次派活最多带几个状态（草稿区按这条流的状态数封顶），与一个进程里 GPU 用例共用的卡池怎么记账。
+//! 单放在 harness 档、只用 `std`：checker 档的 `crash_judge_dispatch`（草稿区封顶）与 `gpu_unit_checks::gpu_test_start` 的卡池
+//! （按卡记账、放不下就等）都调它；这几样不碰 wgpu、不要显卡，单测在 harness 档随时跑得到（checker 档的单测只在提交时跑）。
+//! 挑卡照卡在账里的先后（配置的优先级）找第一张放得下的，前一张放满了才用下一张
+//! （用户 2026-09-29 定「一个显卡 任务满了才去分另外一个」「一边占一点绝对不合理」）；
+//! 账只拦放不下的，不限同时借几份（用户 2026-10-06 定「开到最大的并发和gpu」「仅仅管会不会对指定的电脑造成cpu或者gpu的oom」）。
+
+use std::sync::{Condvar, Mutex, PoisonError};
+
+/// 一个判器一次派活最多带几个状态：显存装得下的、调用方要的、这条流一共几个状态，三者取小，至少 1。
+/// 草稿区按它开：状态数之外开的一个状态都用不上（一条 45 个状态的流按显存开满一张空着的 5090 是约 21 GiB，用得上的约 54 MiB）。
+#[must_use]
+pub fn states_in_one_dispatch_capped_by_the_state_count(
+    states_the_memory_holds: usize,
+    requested_states_per_dispatch: usize,
+    state_count: u64,
+) -> usize {
+    let state_count_as_usize = usize::try_from(state_count).unwrap_or(usize::MAX);
+    states_the_memory_holds
+        .min(requested_states_per_dispatch)
+        .min(state_count_as_usize)
+        .max(1)
+}
+
+/// 账上借一次显存的结局（[`decide_booking_by_priority`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum BookingDecision {
+    /// 记在这张卡上：照卡在账里的先后，第一张「已记的 + 要的 ≤ 容量」的卡（值是它在账里的次序）。
+    BookOnTheCardAt { card_position: usize },
+    /// 此刻每张卡都放不下，而至少一张卡空着时放得下：等别的借据还回来。
+    WaitForMemoryToBeReturned,
+    /// 要的比每张卡的容量都大（一张卡都没有也算）：等多久都放不下。
+    NeverFits { largest_capacity_mebibytes: u64 },
+}
+
+/// 一次借 `need_mebibytes` 记在哪张卡上：`capacities_mebibytes` 是每张卡的容量、`booked_mebibytes` 是每张卡此刻已记的，次序就是优先级。
+///
+/// # Panics
+/// 两张表的卡数不同。
+#[must_use]
+pub fn decide_booking_by_priority(
+    capacities_mebibytes: &[u64],
+    booked_mebibytes: &[u64],
+    need_mebibytes: u64,
+) -> BookingDecision {
+    assert_eq!(
+        capacities_mebibytes.len(),
+        booked_mebibytes.len(),
+        "容量表与已记表要逐张卡对上"
+    );
+    let largest_capacity_mebibytes = capacities_mebibytes.iter().copied().max().unwrap_or(0);
+    if need_mebibytes > largest_capacity_mebibytes {
+        return BookingDecision::NeverFits {
+            largest_capacity_mebibytes,
+        };
+    }
+    match capacities_mebibytes
+        .iter()
+        .zip(booked_mebibytes)
+        .position(|(capacity, booked)| booked.saturating_add(need_mebibytes) <= *capacity)
+    {
+        Some(card_position) => BookingDecision::BookOnTheCardAt { card_position },
+        None => BookingDecision::WaitForMemoryToBeReturned,
+    }
+}
+
+/// 要的比每张卡的容量都大：配了 GPU 就一定用 GPU，调用方据此判红，不悄悄换小。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct NeedLargerThanEveryCard {
+    pub need_mebibytes: u64,
+    pub largest_capacity_mebibytes: u64,
+}
+
+/// 几张卡各一本显存账（MiB）：借的时候照 [`decide_booking_by_priority`] 记上，放不下就等别的借据还，借据丢掉时销账。
+pub struct GpuMemoryLedger {
+    capacities_mebibytes: Vec<u64>,
+    booked_mebibytes: Mutex<Vec<u64>>,
+    memory_returned: Condvar,
+}
+
+impl GpuMemoryLedger {
+    /// 每张卡的容量（MiB），次序就是优先级。
+    #[must_use]
+    pub fn with_card_capacities(capacities_mebibytes: Vec<u64>) -> Self {
+        let booked_mebibytes = Mutex::new(vec![0; capacities_mebibytes.len()]);
+        Self {
+            capacities_mebibytes,
+            booked_mebibytes,
+            memory_returned: Condvar::new(),
+        }
+    }
+
+    #[must_use]
+    pub fn card_capacities_mebibytes(&self) -> &[u64] {
+        &self.capacities_mebibytes
+    }
+
+    /// 借 `need_mebibytes`：记得上就交回借据（丢掉时销账）；此刻放不下就等别的借据还，还一次重判一次；永远放不下交错误、什么都不记。
+    ///
+    /// # Errors
+    /// 要的比每张卡的容量都大（[`NeedLargerThanEveryCard`]）。
+    pub fn book_waiting_for_room(
+        &self,
+        need_mebibytes: u64,
+    ) -> Result<GpuMemoryBooking<'_>, NeedLargerThanEveryCard> {
+        // 账上只做加减，持锁的那一段不会 panic 到一半：别的线程 panic 留下的毒不影响账本身，照用
+        let mut booked = self
+            .booked_mebibytes
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner);
+        loop {
+            match decide_booking_by_priority(&self.capacities_mebibytes, &booked, need_mebibytes) {
+                BookingDecision::BookOnTheCardAt { card_position } => {
+                    booked[card_position] += need_mebibytes;
+                    return Ok(GpuMemoryBooking {
+                        ledger: self,
+                        card_position,
+                        mebibytes: need_mebibytes,
+                    });
+                }
+                BookingDecision::WaitForMemoryToBeReturned => {
+                    booked = self
+                        .memory_returned
+                        .wait(booked)
+                        .unwrap_or_else(PoisonError::into_inner);
+                }
+                BookingDecision::NeverFits {
+                    largest_capacity_mebibytes,
+                } => {
+                    return Err(NeedLargerThanEveryCard {
+                        need_mebibytes,
+                        largest_capacity_mebibytes,
+                    })
+                }
+            }
+        }
+    }
+}
+
+/// 账上的一笔：记在哪张卡、多少 MiB；丢掉时销账、叫醒等着借的。
+pub struct GpuMemoryBooking<'ledger> {
+    ledger: &'ledger GpuMemoryLedger,
+    card_position: usize,
+    mebibytes: u64,
+}
+
+impl GpuMemoryBooking<'_> {
+    /// 记在账里第几张卡上。
+    #[must_use]
+    pub fn card_position(&self) -> usize {
+        self.card_position
+    }
+
+    #[must_use]
+    pub fn mebibytes(&self) -> u64 {
+        self.mebibytes
+    }
+}
+
+impl Drop for GpuMemoryBooking<'_> {
+    fn drop(&mut self) {
+        let mut booked = self
+            .ledger
+            .booked_mebibytes
+            .lock()
+            .unwrap_or_else(PoisonError::into_inner);
+        booked[self.card_position] = booked[self.card_position]
+            .checked_sub(self.mebibytes)
+            .expect("一笔只在记上的那张卡上销一次：已记的不会比这一笔还少");
+        drop(booked);
+        self.ledger.memory_returned.notify_all();
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::{
+        decide_booking_by_priority, states_in_one_dispatch_capped_by_the_state_count,
+        BookingDecision, GpuMemoryLedger, NeedLargerThanEveryCard,
+    };
+
+    /// 一张空着的 5090 照 3/4 额度装得下 17 867 个状态（2026-10-06 日志里那一行）：45 个状态的流只开 45 个，
+    /// 131 195 个状态的大流照旧按显存开，调用方要得更少就照它的，一点显存都不给、流是空的也带一个。
+    #[test]
+    fn a_small_flow_opens_scratch_for_its_own_states_and_a_large_flow_still_follows_the_memory() {
+        const STATES_A_WHOLE_RTX_5090_HOLDS: usize = 17_867;
+        assert_eq!(
+            states_in_one_dispatch_capped_by_the_state_count(
+                STATES_A_WHOLE_RTX_5090_HOLDS,
+                usize::MAX,
+                45
+            ),
+            45,
+            "45 个状态的流只开 45 个状态的草稿区"
+        );
+        assert_eq!(
+            states_in_one_dispatch_capped_by_the_state_count(
+                STATES_A_WHOLE_RTX_5090_HOLDS,
+                usize::MAX,
+                131_195
+            ),
+            STATES_A_WHOLE_RTX_5090_HOLDS,
+            "比显存装得下的还多的流照显存开"
+        );
+        assert_eq!(
+            states_in_one_dispatch_capped_by_the_state_count(
+                STATES_A_WHOLE_RTX_5090_HOLDS,
+                256,
+                131_195
+            ),
+            256,
+            "调用方要的更少就照它的"
+        );
+        assert_eq!(
+            states_in_one_dispatch_capped_by_the_state_count(0, usize::MAX, 0),
+            1,
+            "至少带一个"
+        );
+    }
+
+    /// 两张卡（容量照 5090 与 5060 Ti 的量级）：前一张放得下就一直记在前一张上，前一张放不下这一笔才记到下一张。
+    #[test]
+    fn bookings_fill_the_first_card_before_the_next_card_takes_any() {
+        let capacities = [29_000, 12_000];
+        assert_eq!(
+            decide_booking_by_priority(&capacities, &[0, 0], 600),
+            BookingDecision::BookOnTheCardAt { card_position: 0 },
+            "两张都空着：记在优先的那张"
+        );
+        assert_eq!(
+            decide_booking_by_priority(&capacities, &[28_400, 0], 600),
+            BookingDecision::BookOnTheCardAt { card_position: 0 },
+            "前一张剩的正好放得下：照旧记在前一张，不因为后一张更空就换过去"
+        );
+        assert_eq!(
+            decide_booking_by_priority(&capacities, &[28_500, 0], 600),
+            BookingDecision::BookOnTheCardAt { card_position: 1 },
+            "前一张放不下这一笔：才记到下一张"
+        );
+    }
+
+    /// 每张卡此刻都放不下而空着时放得下的等；比每张卡的容量都大的永远放不下（不等，交错误，账上什么都不记）。
+    #[test]
+    fn a_need_that_fits_an_empty_card_waits_and_one_larger_than_every_card_never_fits() {
+        let capacities = [100, 50];
+        assert_eq!(
+            decide_booking_by_priority(&capacities, &[80, 40], 30),
+            BookingDecision::WaitForMemoryToBeReturned,
+            "两张都放不下 30，而第一张空着时放得下：等"
+        );
+        assert_eq!(
+            decide_booking_by_priority(&capacities, &[0, 0], 101),
+            BookingDecision::NeverFits {
+                largest_capacity_mebibytes: 100
+            },
+            "比最大的那张还大：永远放不下"
+        );
+        assert_eq!(
+            decide_booking_by_priority(&[], &[], 1),
+            BookingDecision::NeverFits {
+                largest_capacity_mebibytes: 0
+            },
+            "一张卡都没有：永远放不下"
+        );
+        let ledger = GpuMemoryLedger::with_card_capacities(capacities.to_vec());
+        assert_eq!(
+            ledger.book_waiting_for_room(101).err(),
+            Some(NeedLargerThanEveryCard {
+                need_mebibytes: 101,
+                largest_capacity_mebibytes: 100
+            }),
+            "账上借永远放不下的：当场交错误，不等"
+        );
+        let whole_first_card = ledger
+            .book_waiting_for_room(100)
+            .expect("交错误的那一次什么都没记：第一张卡还是空的");
+        assert_eq!(whole_first_card.card_position(), 0);
+    }
+
+    /// 一张 100 MiB 的卡被一笔记满，另一条线程再借 30：还没还之前它一直等着，前一笔还回来它就记上同一张卡。
+    #[test]
+    fn a_booking_that_does_not_fit_waits_until_an_earlier_one_is_returned() {
+        // 等着借的那条线程拿的是账的借用；判红时它可能永远等着，账交给 'static、线程不回收，判红不卡住整个用例
+        let ledger: &'static GpuMemoryLedger =
+            Box::leak(Box::new(GpuMemoryLedger::with_card_capacities(vec![100])));
+        let first = ledger
+            .book_waiting_for_room(100)
+            .expect("空着的卡放得下 100");
+        let (sender, receiver) = std::sync::mpsc::channel();
+        std::thread::spawn(move || {
+            let second = ledger
+                .book_waiting_for_room(30)
+                .expect("30 比卡的容量小，放得下");
+            sender
+                .send((second.card_position(), second.mebibytes()))
+                .expect("用例还在等这一条");
+        });
+        assert!(
+            receiver
+                .recv_timeout(std::time::Duration::from_millis(300))
+                .is_err(),
+            "卡被前一笔记满了：后一笔要等，不许当场记上"
+        );
+        drop(first);
+        assert_eq!(
+            receiver.recv_timeout(std::time::Duration::from_secs(30)),
+            Ok((0, 30)),
+            "前一笔还回来之后，等着的那一笔记上同一张卡"
+        );
+    }
+}
````

### 10r mutations-append.tsv

````text
GPU 显存账 草稿区不按流的状态数封顶	crates/singlefs-harness/src/gpu_memory_ledger.rs	        .min(state_count_as_usize)	        .min(usize::MAX)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_small_flow_opens_scratch_for_its_own_states_and_a_large_flow_still_follows_the_memory	a_small_flow_opens_scratch_for_its_own_states_and_a_large_flow_still_follows_the_memory
GPU 显存账 挑卡从优先级最低的那张起	crates/singlefs-harness/src/gpu_memory_ledger.rs	        .position(|(capacity, booked)| booked.saturating_add(need_mebibytes) <= *capacity)	        .rposition(|(capacity, booked)| booked.saturating_add(need_mebibytes) <= *capacity)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::bookings_fill_the_first_card_before_the_next_card_takes_any	bookings_fill_the_first_card_before_the_next_card_takes_any
GPU 显存账 比每张卡都大的也去等	crates/singlefs-harness/src/gpu_memory_ledger.rs	    if need_mebibytes > largest_capacity_mebibytes {	    if false && need_mebibytes > largest_capacity_mebibytes {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_need_that_fits_an_empty_card_waits_and_one_larger_than_every_card_never_fits	a_need_that_fits_an_empty_card_waits_and_one_larger_than_every_card_never_fits
GPU 显存账 还了不叫醒等着借的	crates/singlefs-harness/src/gpu_memory_ledger.rs	        self.ledger.memory_returned.notify_all();	        let _ = &self.ledger.memory_returned;	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_booking_that_does_not_fit_waits_until_an_earlier_one_is_returned	a_booking_that_does_not_fit_waits_until_an_earlier_one_is_returned
GPU 派活 判器要借的显存向下取整	crates/singlefs-checker-tier/src/crash_judge_dispatch.rs	    tables.mebibytes() + (states * bytes_per_state).div_ceil(1024 * 1024)	    tables.mebibytes() + (states * bytes_per_state) / (1024 * 1024)	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_dispatch::tests::a_card_lent_what_a_judge_needs_opens_scratch_for_every_state_it_was_lent_for	a_card_lent_what_a_judge_needs_opens_scratch_for_every_state_it_was_lent_for
````

## 第 10s 批

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
index 8452d18..85fa18e 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -923,6 +923,33 @@ impl<'card> GpuCrashJudge<'card> {
     }
 }
 
+/// GPU 对拍用例在卡池里借一张卡判一条流的每个状态，按序号次序交回每个状态的结论字：表打包一次，一次派活带一条草稿缓冲那么多个状态
+/// （按池里绑定上限最小的那张卡估，与这条流的状态数取小，[`mebibytes_a_judge_needs`] 按它借），判完先丢判器、再还卡。
+/// 等满了借不到时借卡那一层打「本次未跑」那一行，这里交 `None`（调用的用例当场返回）。
+///
+/// # Panics
+/// 同 [`crate::gpu_unit_checks::gpu_test_start::GpuCardPool::lease_card_or_report_not_run`] 与 [`GpuCrashJudge::new`]。
+#[must_use]
+pub fn judge_every_state_on_a_card_leased_from_the_pool(
+    cards: &'static crate::gpu_unit_checks::gpu_test_start::GpuCardPool,
+    test_name: &str,
+    tables: &JudgeTables,
+) -> Option<Vec<[u32; VERDICT_WORDS]>> {
+    let packed = PackedJudgeTables::of(tables);
+    let states_per_dispatch =
+        states_per_scratch_lane(cards.smallest_max_storage_buffer_binding_size());
+    let lease = cards.lease_card_or_report_not_run(
+        test_name,
+        mebibytes_a_judge_needs(&packed, states_per_dispatch),
+    )?;
+    let judge =
+        GpuCrashJudge::on_card_with_packed_tables(lease.card(), &packed, states_per_dispatch);
+    let verdicts = judge.judge_states(0..tables.state_count);
+    drop(judge);
+    drop(lease);
+    Some(verdicts)
+}
+
 #[cfg(test)]
 mod tests {
     use super::{
@@ -1181,6 +1208,169 @@ mod tests {
         }
     }
 
+    /// 第 10q 批在扫描内核里把这七个调用方各自的多处调用并成了一处（调用方，它里面那个被调函数）。
+    const SCAN_KERNEL_JUDGES_MERGED_INTO_ONE_CALL_SITE: [(&str, &str); 7] = [
+        (
+            "judge_release_generation_and_tree_table_birth",
+            "r_references_of_root",
+        ),
+        (
+            "judge_allocation_records_disjoint",
+            "judge_allocation_records_disjoint_of_a_version",
+        ),
+        (
+            "judge_allocation_generations_against_unit_births",
+            "judge_allocation_generations_under_tree_table",
+        ),
+        ("judge_accounting", "c_quarantine_exempted"),
+        (
+            "judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark",
+            "judge_first_data_unit_of_a_version",
+        ),
+        (
+            "judge_deletion_state_of_the_versions",
+            "d_judge_deletion_state_of_a_version",
+        ),
+        (
+            "judge_tombstones_and_the_lineage_of_the_versions",
+            "d_judge_tombstones_and_the_lineage_of_a_version",
+        ),
+    ];
+
+    /// 一份内核源码里每个（调用方, 被调方）有几个调用点：naga 解析之后逐个函数数调用语句，入口函数记成 `entry:<名字>`。
+    /// 与 [`super::kernel_inlined_size`] 各数各的（它不交回调用点表），这一份只给钉调用点数的用例用。
+    fn call_sites_by_caller_and_callee(
+        source: &str,
+    ) -> std::collections::BTreeMap<(String, String), u64> {
+        use wgpu::naga::{Block, Function, Handle, Statement};
+        #[allow(
+            clippy::wildcard_enum_match_arm,
+            reason = "naga 的语句是外部的枚举，只有这四种里还套着语句块，其余一律不含调用"
+        )]
+        fn callees_in(block: &Block, callees: &mut Vec<Handle<Function>>) {
+            for statement in block.iter() {
+                match statement {
+                    Statement::Call { function, .. } => callees.push(*function),
+                    Statement::Block(nested) => callees_in(nested, callees),
+                    Statement::If { accept, reject, .. } => {
+                        callees_in(accept, callees);
+                        callees_in(reject, callees);
+                    }
+                    Statement::Switch { cases, .. } => {
+                        for switch_case in cases {
+                            callees_in(&switch_case.body, callees);
+                        }
+                    }
+                    Statement::Loop {
+                        body, continuing, ..
+                    } => {
+                        callees_in(body, callees);
+                        callees_in(continuing, callees);
+                    }
+                    _ => {}
+                }
+            }
+        }
+        let module = wgpu::naga::front::wgsl::parse_str(source)
+            .unwrap_or_else(|error| panic!("内核解析失败：\n{}", error.emit_to_string(source)));
+        let mut sites = std::collections::BTreeMap::new();
+        let callers = module
+            .functions
+            .iter()
+            .map(|(_, function)| (function.name.clone().unwrap_or_default(), &function.body))
+            .chain(
+                module
+                    .entry_points
+                    .iter()
+                    .map(|entry| (format!("entry:{}", entry.name), &entry.function.body)),
+            );
+        for (caller, body) in callers {
+            let mut callees = Vec::new();
+            callees_in(body, &mut callees);
+            for callee in callees {
+                *sites
+                    .entry((
+                        caller.clone(),
+                        module.functions[callee].name.clone().unwrap_or_default(),
+                    ))
+                    .or_insert(0) += 1;
+            }
+        }
+        sites
+    }
+
+    /// 第 10q 批并成一处调用的七个函数：每个被调函数在整个扫描内核里只有一个调用点、就在它的调用方里，调用方自己也只被调一处
+    /// （数是 2026-10-06 在主工作区的扫描内核上现量的）。别处再多调一次（哪怕总数还在预算里，例：`judge_accounting` 加回一处只到 392 383），
+    /// 先红在这里，不等上卡才撞展开预算；真要多一处，先量展开之后的大小，再改这里的钉值。
+    #[test]
+    fn each_judge_merged_into_one_call_site_keeps_that_single_call_site_in_the_scan_kernel() {
+        const CALLEE_SITES_INSIDE_THE_CALLER: u64 = 1;
+        const CALLEE_SITES_IN_THE_WHOLE_KERNEL: u64 = 1;
+        const CALLER_SITES_IN_THE_WHOLE_KERNEL: u64 = 1;
+        let sites =
+            call_sites_by_caller_and_callee(&crate::crash_judge_gpu::checker_scan_kernel_source());
+        let sites_calling = |callee: &str| -> u64 {
+            sites
+                .iter()
+                .filter(|((_, called), _)| called == callee)
+                .map(|(_, count)| *count)
+                .sum()
+        };
+        for (caller, callee) in SCAN_KERNEL_JUDGES_MERGED_INTO_ONE_CALL_SITE {
+            assert_eq!(
+                sites
+                    .get(&(caller.to_string(), callee.to_string()))
+                    .copied()
+                    .unwrap_or(0),
+                CALLEE_SITES_INSIDE_THE_CALLER,
+                "{caller} 里调 {callee} 的地方只许一处"
+            );
+            assert_eq!(
+                sites_calling(callee),
+                CALLEE_SITES_IN_THE_WHOLE_KERNEL,
+                "整个扫描内核里调 {callee} 的地方只许一处（就是 {caller} 里那一处）"
+            );
+            assert_eq!(
+                sites_calling(caller),
+                CALLER_SITES_IN_THE_WHOLE_KERNEL,
+                "整个扫描内核里调 {caller} 的地方只许一处"
+            );
+        }
+    }
+
+    /// 三个内核此刻展开之后的表达式数与离预算的余量（2026-10-06 在主工作区上现量的；扫描内核是第 10q 批合后的数）：
+    /// 改了哪个内核的源码，这里先红，量过新数、确认余量还够之后再改钉值，免得余量悄悄耗光、等上卡时才撞预算。
+    #[test]
+    fn each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget() {
+        for (name, source, pinned_expressions, pinned_margin) in [
+            ("recovery", recovery_kernel_source(), 316_712, 83_288),
+            (
+                "checker-scan",
+                crate::crash_judge_gpu::checker_scan_kernel_source(),
+                378_232,
+                21_768,
+            ),
+            (
+                "checker-walk",
+                crate::crash_judge_gpu::checker_walk_kernel_source(),
+                323_759,
+                76_241,
+            ),
+        ] {
+            let size = super::kernel_inlined_size(&source);
+            assert_eq!(
+                size.expressions, pinned_expressions,
+                "内核 {name} 展开之后的表达式数变了：量过、确认余量还够之后改这里的钉值"
+            );
+            assert_eq!(
+                super::LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL.checked_sub(size.expressions),
+                Some(pinned_margin),
+                "内核 {name} 离预算 {} 的余量变了",
+                super::LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL
+            );
+        }
+    }
+
     /// 走读内核里「走一遍根」回到六个调用点（第二台 580 版驱动 48G 里编不完的那个形态），编之前就被拒。
     #[test]
     fn the_walk_kernel_with_the_root_walk_called_from_six_sites_is_refused() {
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
index b63f749..7e00f3e 100644
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -830,139 +830,11 @@ impl WeightedShares {
     }
 }
 
-/// 配置里的一张卡在哪台机器上。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub enum CardHost {
-    Local,
-    Peer,
-}
-
-impl CardHost {
-    #[must_use]
-    pub fn name(self) -> &'static str {
-        match self {
-            Self::Local => "local",
-            Self::Peer => "peer",
-        }
-    }
-
-    /// 这个进程在哪台机器上：环境变量 `SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE`，没设是本机。
-    ///
-    /// # Errors
-    /// 写的不是 `local` 也不是 `peer`。
-    pub fn of_this_process() -> Result<Self, String> {
-        match std::env::var("SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE") {
-            Err(_) => Ok(Self::Local),
-            Ok(text) => Self::parse(&text),
-        }
-    }
-
-    fn parse(text: &str) -> Result<Self, String> {
-        match text.trim() {
-            "local" => Ok(Self::Local),
-            "peer" => Ok(Self::Peer),
-            other => Err(format!("机器要写 local 或 peer，实际「{other}」")),
-        }
-    }
-}
-
-/// 配置点名的一张卡（`GPU_CARDS_BY_PRIORITY` 里的一项）：在哪台机器、`nvidia-smi` 的序号、给崩溃放量的显存额度。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub struct CardByPriority {
-    pub host: CardHost,
-    pub index: u32,
-    pub memory_quota_mebibytes: u64,
-}
-
-/// 读 `GPU_CARDS_BY_PRIORITY` 的值：空白隔开的若干项，每项 `<local|peer>:<nvidia-smi 序号>:<显存额度>`，
-/// 额度是正整数加 `G` 或 `M`（1024 进制）；写在前面的优先级高，任务先装它、装满了才给后面的。
-///
-/// # Errors
-/// 一项不是三段、机器或序号或额度写法不对、同一张卡点名两次、一项都没有。
-pub fn parse_cards_by_priority(text: &str) -> Result<Vec<CardByPriority>, String> {
-    let mut cards: Vec<CardByPriority> = Vec::new();
-    for entry in text.split_whitespace() {
-        let fields: Vec<&str> = entry.split(':').collect();
-        let [host, index, quota] = fields.as_slice() else {
-            return Err(format!(
-                "显卡优先级的一项要写 <local|peer>:<序号>:<显存额度>，实际「{entry}」"
-            ));
-        };
-        let host = CardHost::parse(host)?;
-        let index: u32 = index
-            .parse()
-            .map_err(|_| format!("显卡序号不是数：「{entry}」"))?;
-        let (number, unit_mebibytes) = match quota.as_bytes().last() {
-            Some(b'G') => (&quota[..quota.len() - 1], 1024u64),
-            Some(b'M') => (&quota[..quota.len() - 1], 1u64),
-            _ => {
-                return Err(format!(
-                    "显存额度要带单位 G 或 M（1024 进制），实际「{entry}」"
-                ))
-            }
-        };
-        let number: u64 = number
-            .parse()
-            .ok()
-            .filter(|number| *number > 0)
-            .ok_or_else(|| format!("显存额度不是正整数：「{entry}」"))?;
-        if cards
-            .iter()
-            .any(|card| card.host == host && card.index == index)
-        {
-            return Err(format!("同一张卡点名了两次：「{entry}」"));
-        }
-        cards.push(CardByPriority {
-            host,
-            index,
-            memory_quota_mebibytes: number * unit_mebibytes,
-        });
-    }
-    if cards.is_empty() {
-        return Err("显卡优先级一项都没写".to_string());
-    }
-    Ok(cards)
-}
-
-/// `键=值` 配置文件里一个键的值（去掉两头的空白与一层引号）；文件不在、键没写是 `None`。
-#[must_use]
-pub fn configuration_value(path: &Path, key: &str) -> Option<String> {
-    let text = std::fs::read_to_string(path).ok()?;
-    text.lines().find_map(|line| {
-        let value = line.trim().strip_prefix(key)?.strip_prefix('=')?.trim();
-        let value = value
-            .strip_prefix('\'')
-            .and_then(|rest| rest.strip_suffix('\''))
-            .or_else(|| {
-                value
-                    .strip_prefix('"')
-                    .and_then(|rest| rest.strip_suffix('"'))
-            })
-            .unwrap_or(value);
-        Some(value.to_string())
-    })
-}
-
-/// 配置的显卡优先级：环境变量 `GPU_CARDS_BY_PRIORITY` 设了就照它（驱动脚本按清没清场从显卡配置里挑一份送过来，第二台的树副本也靠它），
-/// 没设读显卡配置文件（`SINGLEFS_GPU_CARDS_CONFIG` 指的那一份，模板是仓根的 `gpu-cards.env.example`）里的同名键；
-/// 都没有是 `None`（本机能用的卡按空闲显存从大到小用，不设额度）。
-///
-/// # Errors
-/// 写了而写法不对（[`parse_cards_by_priority`]）。
-pub fn cards_by_priority_from_configuration_or_environment(
-    configuration_file: Option<&Path>,
-) -> Result<Option<Vec<CardByPriority>>, String> {
-    let text = match std::env::var("GPU_CARDS_BY_PRIORITY") {
-        Ok(text) => Some(text),
-        Err(_) => {
-            configuration_file.and_then(|path| configuration_value(path, "GPU_CARDS_BY_PRIORITY"))
-        }
-    };
-    match text {
-        Some(text) if !text.trim().is_empty() => parse_cards_by_priority(&text).map(Some),
-        _ => Ok(None),
-    }
-}
+/// 显卡配置的读法住在 [`crate::gpu_card_configuration`]（只开 `gpu` 的卡池也照它挑卡），这里转出来，调用方与单测照旧经这一份引。
+pub use crate::gpu_card_configuration::{
+    cards_by_priority_from_configuration_or_environment, configuration_value,
+    configured_cards_not_usable, parse_cards_by_priority, CardByPriority, CardHost,
+};
 
 /// 显卡配置里登记哪几类计算走 GPU 的键（上游 `.claude/singlefs-ai-sop/rules/test-script-discipline.md`「用上现有的硬件加速」；
 /// 模板是仓根的 `gpu-cards.env.example`）。环境变量同名，设了就照它。
@@ -1645,16 +1517,6 @@ pub fn first_state_the_gpu_could_not_judge(
     })
 }
 
-/// 配置点名的卡里哪几张（序号）不在用得上的卡里，按配置的次序。
-#[must_use]
-pub fn configured_cards_not_usable(configured: &[(u32, u64)], usable: &[u32]) -> Vec<u32> {
-    configured
-        .iter()
-        .map(|(index, _)| *index)
-        .filter(|index| !usable.contains(index))
-        .collect()
-}
-
 /// 一个状态的 GPU 结论字换成与 CPU `Layer0StateJudgement::red_items` 逐项同名、同序的红项；判不了的状态交回 `None`（回落到 CPU）。
 #[cfg(feature = "gpu")]
 #[must_use]
diff --git a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
index c556a6a..e58fe8f 100644
--- a/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
+++ b/crates/singlefs-checker-tier/src/gpu_unit_checks.rs
@@ -219,6 +219,35 @@ pub fn usable_gpu_cards() -> Vec<GpuCard> {
     cards
 }
 
+/// 一张卡给崩溃放量的额度（MiB）：给了额度（`memory_quota_mebibytes`）的取额度与空闲显存里小的那个，
+/// 没给的取空闲显存的 [`SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA`]（[`GpuCard::memory_budget_mebibytes`] 照它算；卡池照它、
+/// 拿账本里记的清场后那一份空闲显存算每张卡的账）。
+#[must_use]
+pub fn memory_budget_mebibytes_of(
+    memory_quota_mebibytes: Option<u64>,
+    free_memory_mebibytes: u64,
+) -> u64 {
+    match memory_quota_mebibytes {
+        Some(quota) => quota.min(free_memory_mebibytes),
+        None => {
+            let (numerator, denominator) = SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA;
+            free_memory_mebibytes / denominator * numerator
+        }
+    }
+}
+
+/// 在卡上算 `range_count` 段字节的 CRC-32C（[`GpuCard::crc32c_of_ranges`]）一次要多少显存（MiB，向上取整）：每段一个 32 KiB 的槽、
+/// 一个长度字、一条结果与它的回读，加一份派活参数。核一批单元的两道校验和（[`check_unit_checksums_on_gpu`]）每个单元两段。
+/// 用例向卡池借卡时按它借（`gpu_test_start::GpuCardPool::lease_card_or_report_not_run`）。
+#[must_use]
+pub fn mebibytes_crc32c_of_ranges_needs(range_count: usize) -> u64 {
+    const PARAMETER_BYTES: usize = 16;
+    let bytes = range_count * (SLOT_BYTES + 4 + 2 * RESULT_ENTRY_BYTES) + PARAMETER_BYTES;
+    u64::try_from(bytes)
+        .expect("装得进 u64")
+        .div_ceil(1024 * 1024)
+}
+
 /// 驱动把编好的 Vulkan 管线存到盘上的目录：跨轮复用的缓存放 `GATE_CROSS_RUN_TMPDIR` 下；没设就放家目录 `~/.cache/singlefs`
 /// （`/tmp` 开机会清空，冷编要十几分钟的内核每次重启都得重编）；连 `HOME` 都没有才退到 `TMPDIR`、`/tmp`
 /// （`command-safety.md`「测试镜像一律放临时目录」：有意跨轮复用的缓存不算镜像）。
@@ -262,13 +291,7 @@ impl GpuCard {
     /// 没给额度的取空闲显存的 [`SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA`]。
     #[must_use]
     pub fn memory_budget_mebibytes(&self) -> u64 {
-        match self.memory_quota_mebibytes {
-            Some(quota) => quota.min(self.free_memory_mebibytes),
-            None => {
-                let (numerator, denominator) = SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA;
-                self.free_memory_mebibytes / denominator * numerator
-            }
-        }
+        memory_budget_mebibytes_of(self.memory_quota_mebibytes, self.free_memory_mebibytes)
     }
 
     /// 这张卡的设备（崩溃放量第 ② 段的核对内核在同一张卡上另起自己的管线）。
@@ -853,30 +876,44 @@ pub mod gpu_test_start {
         })
     }
 
-    /// 卡池挂 `verdict-store`：照显卡配置挑这台机器的卡要的那几样（`GPU_CARDS_BY_PRIORITY` 的读法、机器角色、点名的卡少没少）
-    /// 住在 `crash_amplification`，它只随 `verdict-store` 编；只开 `gpu` 的测试二进制用不了卡池，照旧经
-    /// [`usable_cards_or_report_not_run`] 与 [`first_usable_card_or_report_not_run`] 取卡。
-    #[cfg(feature = "verdict-store")]
-    pub use card_pool::{gpu_card_pool_or_report_not_run, GpuCardLease, GpuCardPool};
+    /// GPU 用例共用的卡池（[`card_pool::GpuCardPool`]）：只开 `gpu` 也编得进（显卡配置的读法在 `crate::gpu_card_configuration`）。
+    pub use card_pool::{
+        gpu_card_pool_or_report_not_run, GpuCardLease, GpuCardPool, CARD_QUOTA_ENVIRONMENT_VARIABLE,
+    };
 
-    #[cfg(feature = "verdict-store")]
     mod card_pool {
-        use super::super::{gpu_cards_by_priority, usable_gpu_cards, GpuCard};
+        use super::super::{
+            gpu_cards_by_priority, memory_budget_mebibytes_of, usable_gpu_cards, GpuCard,
+        };
         use super::{
             decide_gpu_test_start, enumerate_vulkan_adapters, not_run_line,
             read_pci_nvidia_display_controllers_on_the_bus, GpuTestStart, PCI_BUS_DIRECTORY,
         };
-        use crate::crash_amplification::{
+        use crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
+        use crate::gpu_card_configuration::{
             cards_by_priority_from_configuration_or_environment, configured_cards_not_usable,
             CardHost,
         };
-        use crate::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
-        use singlefs_harness::gpu_memory_ledger::{GpuMemoryBooking, GpuMemoryLedger};
+        use singlefs_harness::gpu_memory_ledger::{
+            card_ledger_capacity_mebibytes, gpu_memory_ledger_directory_of_this_process,
+            process_is_alive_by_its_entry_in_the_process_filesystem, BookingRefusal,
+            CardToRegister, GpuMemoryBooking, GpuMemoryLedger, LedgerSettings, RecordedProcess,
+            LONGEST_WAIT_FOR_GPU_MEMORY, POLL_INTERVAL_FOR_MEMORY_RETURNED_BY_OTHER_PROCESSES,
+        };
+        use singlefs_harness::numeric_environment_variable::{
+            number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
+        };
+
+        /// 每张卡账的上限（MiB）：设了，每张卡的账取它与「清场后的空闲显存减 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]」里小的那个
+        /// （`singlefs_harness::gpu_memory_ledger::card_ledger_capacity_mebibytes`）；不是每条用例的上限，借得比它多的照常判「放不下」。
+        pub const CARD_QUOTA_ENVIRONMENT_VARIABLE: &str =
+            "SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES";
 
-        /// 一个进程里 GPU 用例共用的卡：这台机器点名的每张卡起一次设备，各一本显存账（[`GpuMemoryLedger`]）；
-        /// 用例按自己实际要的量借（[`GpuCardPool::lease_card_waiting_for_room`]），借据丢掉时还。
-        /// 一张卡的账 = 这张卡给崩溃放量的额度（[`GpuCard::memory_budget_mebibytes`]）减一份
-        /// [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]：驱动的上下文与编好的内核每张卡（每个设备）付一次，不按用例各付一份。
+        /// GPU 用例共用的卡：这台机器点名的每张卡起一次设备，各一本跨进程的显存账（[`GpuMemoryLedger`]，账本在工作根下）；
+        /// 用例按自己实际要的量借（[`GpuCardPool::lease_card_or_report_not_run`]），借据丢掉时还。
+        /// 一张卡的账照 [`card_ledger_capacity_mebibytes`]：这张卡给崩溃放量的额度（[`GpuCard::memory_budget_mebibytes`] 的算法，
+        /// 空闲显存取账本里记的清场后那一份）或 [`CARD_QUOTA_ENVIRONMENT_VARIABLE`]，减一份 [`MEBIBYTES_LEFT_FOR_THE_DRIVER`]：
+        /// 驱动的上下文与编好的内核每个进程在每张卡上付一次，同一台机器上别的进程起的池各再记一份。
         pub struct GpuCardPool {
             cards: Vec<GpuCard>,
             ledger: GpuMemoryLedger,
@@ -927,29 +964,75 @@ pub mod gpu_test_start {
                     )
             }
 
-            /// 借 `need_mebibytes`：照卡的优先级记在第一张放得下的卡上（前一张放满了才用下一张）；此刻每张都放不下就等别的用例还。
-            /// 每借一次打一行 `GPU_CARD_LEASE test=… index=… mebibytes=… capacity_mebibytes=…`。
+            /// 借 `need_mebibytes`：照卡的优先级记在第一张放得下的卡上（前一张放满了才用下一张）；此刻每张都放不下就等别的用例、
+            /// 别的 GPU 测试进程还，等满 [`LONGEST_WAIT_FOR_GPU_MEMORY`] 还借不到就打 [`not_run_line`] 那一行、交回 `None`（调用的用例当场返回）。
+            /// 每借到一次打一行 `GPU_CARD_LEASE test=… index=… mebibytes=… capacity_mebibytes=…`。
             ///
             /// # Panics
-            /// 要的比每张卡的账都大：配了 GPU 就一定用 GPU，放不下判红，不悄悄换小。
+            /// 要的比每张卡的账都大（配了 GPU 就一定用 GPU，放不下判红，不悄悄换小）；账本文件读写不了。
+            #[must_use]
+            pub fn lease_card_or_report_not_run(
+                &'static self,
+                test_name: &str,
+                need_mebibytes: u64,
+            ) -> Option<GpuCardLease> {
+                match self.lease_card(test_name, need_mebibytes) {
+                    Ok(lease) => Some(lease),
+                    Err(waited) => {
+                        println!("{}", not_run_line(test_name, &waited));
+                        None
+                    }
+                }
+            }
+
+            /// 同 [`Self::lease_card_or_report_not_run`]，等满了判红而不报「本次未跑」：给还没改成报本次未跑的调用方留着
+            /// （`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`）。
+            ///
+            /// # Panics
+            /// 同 [`Self::lease_card_or_report_not_run`]；等满 [`LONGEST_WAIT_FOR_GPU_MEMORY`] 还借不到。
             #[must_use]
             pub fn lease_card_waiting_for_room(
                 &'static self,
                 test_name: &str,
                 need_mebibytes: u64,
             ) -> GpuCardLease {
-                let booking = self
-                    .ledger
-                    .book_waiting_for_room(need_mebibytes)
-                    .unwrap_or_else(|refusal| {
-                        panic!(
-                            "{test_name}：要借 {} MiB，池里最大的一张卡的账只有 {} MiB（每张卡的账 {:?}）：这台机器的卡放不下这条用例，\
-                             腾出显存、给卡更大的额度（GPU_CARDS_BY_PRIORITY），或缩小用例",
-                            refusal.need_mebibytes,
-                            refusal.largest_capacity_mebibytes,
+                self.lease_card(test_name, need_mebibytes)
+                    .unwrap_or_else(|waited| {
+                        panic!("{test_name}：{waited}（这一处还没改成经 lease_card_or_report_not_run 报本次未跑）")
+                    })
+            }
+
+            /// 借一次：借到交借来的卡，等满了交一句为什么。
+            fn lease_card(
+                &'static self,
+                test_name: &str,
+                need_mebibytes: u64,
+            ) -> Result<GpuCardLease, String> {
+                let booking = match self.ledger.book_waiting_for_room(need_mebibytes) {
+                    Ok(booking) => booking,
+                    Err(BookingRefusal::NeedLargerThanEveryCard(refusal)) => panic!(
+                        "{test_name}：要借 {} MiB，池里最大的一张卡的账只有 {} MiB（每张卡的账 {:?}）：这台机器的卡放不下这条用例，\
+                         腾出显存、给卡更大的额度（GPU_CARDS_BY_PRIORITY），或缩小用例",
+                        refusal.need_mebibytes,
+                        refusal.largest_capacity_mebibytes,
+                        self.ledger.card_capacities_mebibytes()
+                    ),
+                    Err(BookingRefusal::WaitedPastTheLongestWait {
+                        need_mebibytes: waited_for_mebibytes,
+                        waited,
+                    }) => {
+                        return Err(format!(
+                            "等了 {} 秒，池里没有一张卡空得出 {waited_for_mebibytes} MiB（每张卡的账 {:?}；这台机器上别的 GPU 用例或测试进程一直占着）",
+                            waited.as_secs(),
                             self.ledger.card_capacities_mebibytes()
-                        )
-                    });
+                        ))
+                    }
+                    Err(BookingRefusal::LedgerFileUnusable(error)) => panic!(
+                        "{test_name}：显存账本 {} 用不了：{}",
+                        error.path.display(),
+                        error.reason
+                    ),
+                };
                 let position = booking.card_position();
                 let mut card = self.cards[position].clone();
                 card.memory_quota_mebibytes = Some(need_mebibytes + MEBIBYTES_LEFT_FOR_THE_DRIVER);
@@ -959,7 +1042,7 @@ pub mod gpu_test_start {
                     card.index,
                     self.ledger.card_capacities_mebibytes()[position]
                 );
-                GpuCardLease { card, booking }
+                Ok(GpuCardLease { card, booking })
             }
         }
 
@@ -1040,6 +1123,59 @@ pub mod gpu_test_start {
             }
         }
 
+        /// 一张卡的账本文件名的主干：按 PCI 总线号（换掉 `:`），同一台机器上几个进程认得出是同一张卡。
+        fn ledger_file_stem_of(card: &GpuCard) -> String {
+            format!("card-{}", card.pci_bus_identifier.replace(':', "-"))
+        }
+
+        /// 点名的卡都起好了之后，在每张卡的账本上记这个进程，开账。
+        fn open_the_ledger_of_the_started_cards(
+            cards: &[GpuCard],
+        ) -> Result<GpuMemoryLedger, String> {
+            let card_quota = number_from_the_variable_value_panicking_when_unreadable::<u64>(
+                CARD_QUOTA_ENVIRONMENT_VARIABLE,
+                process_environment_variable(CARD_QUOTA_ENVIRONMENT_VARIABLE).as_deref(),
+            );
+            let directory = gpu_memory_ledger_directory_of_this_process().ok_or_else(|| {
+                "显存账本放在工作根下，SINGLEFS_WORK_ROOT 与 HOME 都没设，定不下工作根".to_string()
+            })?;
+            let this_process = RecordedProcess::this_process().ok_or_else(|| {
+                "读不到 /proc/self/stat：账本要按进程号与启动时刻判记账进程还在不在".to_string()
+            })?;
+            let settings = LedgerSettings {
+                directory,
+                this_process,
+                is_alive: Box::new(process_is_alive_by_its_entry_in_the_process_filesystem),
+                mebibytes_each_other_process_keeps_on_a_card: MEBIBYTES_LEFT_FOR_THE_DRIVER,
+                poll_interval_for_memory_returned_by_other_processes:
+                    POLL_INTERVAL_FOR_MEMORY_RETURNED_BY_OTHER_PROCESSES,
+                longest_wait: LONGEST_WAIT_FOR_GPU_MEMORY,
+            };
+            let to_register: Vec<CardToRegister> = cards
+                .iter()
+                .map(|card| CardToRegister {
+                    file_stem: ledger_file_stem_of(card),
+                    free_memory_read_by_this_process_mebibytes: card.free_memory_mebibytes,
+                })
+                .collect();
+            GpuMemoryLedger::register_this_process_on_the_cards(
+                settings,
+                &to_register,
+                |position, free_memory_after_quiescing_mebibytes| {
+                    card_ledger_capacity_mebibytes(
+                        card_quota,
+                        memory_budget_mebibytes_of(
+                            cards[position].memory_quota_mebibytes,
+                            free_memory_after_quiescing_mebibytes,
+                        ),
+                        free_memory_after_quiescing_mebibytes,
+                        MEBIBYTES_LEFT_FOR_THE_DRIVER,
+                    )
+                },
+            )
+            .map_err(|error| format!("显存账本 {} 用不了：{}", error.path.display(), error.reason))
+        }
+
         fn start_the_gpu_card_pool() -> GpuCardPoolStart {
             let named = cards_named_for_this_machine();
             let decision = decide_gpu_test_start(
@@ -1056,18 +1192,13 @@ pub mod gpu_test_start {
             match decision {
                 GpuTestStart::RunOnTheUsableCards => {
                     if named.named_but_not_started.is_empty() {
-                        let capacities = named
-                            .cards
-                            .iter()
-                            .map(|card| {
-                                card.memory_budget_mebibytes()
-                                    .saturating_sub(MEBIBYTES_LEFT_FOR_THE_DRIVER)
-                            })
-                            .collect();
-                        GpuCardPoolStart::Started(GpuCardPool {
-                            cards: named.cards,
-                            ledger: GpuMemoryLedger::with_card_capacities(capacities),
-                        })
+                        match open_the_ledger_of_the_started_cards(&named.cards) {
+                            Ok(ledger) => GpuCardPoolStart::Started(GpuCardPool {
+                                cards: named.cards,
+                                ledger,
+                            }),
+                            Err(reason) => GpuCardPoolStart::CardsPresentButNotUsable { reason },
+                        }
                     } else {
                         // 点名的卡少一张都不跑（用户 2026-09-29 定：配了 GPU 就要它干活，不行就明确报错，不接受少用几张的默认降级）
                         GpuCardPoolStart::CardsPresentButNotUsable {
@@ -1093,12 +1224,13 @@ pub mod gpu_test_start {
 
         static GPU_CARD_POOL: std::sync::OnceLock<GpuCardPoolStart> = std::sync::OnceLock::new();
 
-        /// 这个进程的卡池：第一条来要的用例起池（只在这时判一次有没有卡、读一次 `nvidia-smi`、每张卡起一次设备，
+        /// 这个进程的卡池：第一条来要的用例起池（只在这时判一次有没有卡、读一次 `nvidia-smi`、每张卡起一次设备、在每张卡的账本上记这个进程，
         /// 挑卡照 [`cards_named_for_this_machine`]），之后每条用例拿同一个。本机没有 NVIDIA 卡时每条来要的用例各打一行
-        /// [`not_run_line`]、交回 `None`（调用的用例当场返回）。先判有没有卡、建好表知道要多少显存之后再借（[`GpuCardPool::lease_card_waiting_for_room`]）。
+        /// [`not_run_line`]、交回 `None`（调用的用例当场返回）。先判有没有卡、建好表知道要多少显存之后再借（[`GpuCardPool::lease_card_or_report_not_run`]）。
         ///
         /// # Panics
-        /// 有卡而用不上（同 [`super::usable_cards_or_report_not_run`]）；显卡配置写错、没给这台机器点卡、点名的卡少一张起不来。
+        /// 有卡而用不上（同 [`super::usable_cards_or_report_not_run`]）；显卡配置写错、没给这台机器点卡、点名的卡少一张起不来；
+        /// [`CARD_QUOTA_ENVIRONMENT_VARIABLE`] 设了而认不出；账本目录定不下、账本读写不了。
         #[must_use]
         pub fn gpu_card_pool_or_report_not_run(test_name: &str) -> Option<&'static GpuCardPool> {
             match GPU_CARD_POOL.get_or_init(start_the_gpu_card_pool) {
diff --git a/crates/singlefs-checker-tier/src/lib.rs b/crates/singlefs-checker-tier/src/lib.rs
index e936b29..ca7baa1 100644
--- a/crates/singlefs-checker-tier/src/lib.rs
+++ b/crates/singlefs-checker-tier/src/lib.rs
@@ -2,7 +2,7 @@
 //! 崩溃注入（`crash_injection`，可写挂载的分段与崩溃镜像落池在 `writable_mount_phases`）与坏盘输入（`bad_disk_input`）两场战役、真设备一侧的设备日志比对（`device_log`）与运行模式（`on_device_modes`），
 //! 崩溃放量（里程碑二「增补 4」）：崩溃点的身份与复用键（`crash_identity`）、先录后核的流水线（`crash_amplification`：崩溃点计划、路径编号、
 //! 按块录入与核对、按权重分工、对账、门禁读一次）、判定存储（`verdict_store`，RocksDB，特性 `verdict-store`）、
-//! 单元两道校验和上 GPU 与 CPU 对拍（`gpu_unit_checks`，特性 `gpu`）、GPU 判器的判法（`crash_judge_gpu`）与它的派活（`crash_judge_dispatch`，不在判法闭包里）。
+//! 单元两道校验和上 GPU 与 CPU 对拍（`gpu_unit_checks`，特性 `gpu`）、崩溃放量取卡与 GPU 用例的卡池共用的显卡配置读法（`gpu_card_configuration`，`gpu` 与 `verdict-store` 开一个就编）、GPU 判器的判法（`crash_judge_gpu`）与它的派活（`crash_judge_dispatch`，不在判法闭包里）。
 //! 脚手架（内存池、录制器、理想模型、随机历史）在 `singlefs_harness`；池级 checker 判决器在 `singlefs_checker`。
 #![forbid(unsafe_code)]
 
@@ -21,6 +21,8 @@ pub mod crash_judge_tables;
 #[cfg(feature = "gpu")]
 pub mod crash_verify_gpu;
 pub mod device_log;
+#[cfg(any(feature = "gpu", feature = "verdict-store"))]
+pub mod gpu_card_configuration;
 #[cfg(feature = "gpu")]
 pub mod gpu_unit_checks;
 pub mod layer0_progress;
diff --git a/crates/singlefs-harness/src/gpu_memory_ledger.rs b/crates/singlefs-harness/src/gpu_memory_ledger.rs
index 19b528f..80099c4 100644
--- a/crates/singlefs-harness/src/gpu_memory_ledger.rs
+++ b/crates/singlefs-harness/src/gpu_memory_ledger.rs
@@ -1,11 +1,21 @@
-//! GPU 卡上的显存怎么分：一个判器一次派活最多带几个状态（草稿区按这条流的状态数封顶），与一个进程里 GPU 用例共用的卡池怎么记账。
+//! GPU 卡上的显存怎么分：一个判器一次派活最多带几个状态（草稿区按这条流的状态数封顶），与 GPU 用例共用的卡池怎么记账。
 //! 单放在 harness 档、只用 `std`：checker 档的 `crash_judge_dispatch`（草稿区封顶）与 `gpu_unit_checks::gpu_test_start` 的卡池
 //! （按卡记账、放不下就等）都调它；这几样不碰 wgpu、不要显卡，单测在 harness 档随时跑得到（checker 档的单测只在提交时跑）。
 //! 挑卡照卡在账里的先后（配置的优先级）找第一张放得下的，前一张放满了才用下一张
 //! （用户 2026-09-29 定「一个显卡 任务满了才去分另外一个」「一边占一点绝对不合理」）；
-//! 账只拦放不下的，不限同时借几份（用户 2026-10-06 定「开到最大的并发和gpu」「仅仅管会不会对指定的电脑造成cpu或者gpu的oom」）。
+//! 账只拦放不下的，不限同时借几份、也不限一台机器上同时跑几个 GPU 测试进程（用户 2026-10-06 定「开到最大的并发和gpu」
+//! 「仅仅管会不会对指定的电脑造成cpu或者gpu的oom」「不压并发」）。
+//!
+//! 账跨进程：每张卡一份账本文件，放在工作根下的 [`GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME`]，读写前对同名的锁文件 `flock`，
+//! 改完写到临时文件再改名换上（进程在写的半路被杀，账本还是改之前那一份）。账本里记三样：头一个起池的进程读到的空闲显存
+//! （这张卡上没有别的 GPU 测试进程时读的那一份，后起的进程照它算账的容量）、起了池的每个进程（各在卡上付一份驱动的固定开销）、
+//! 每一笔借出去的显存。每条记录带记账进程的进程号与启动时刻，读账时照 [`process_is_alive_by_its_entry_in_the_process_filesystem`] 判活，
+//! 死了的那几条当没记、下一次写账时删掉（进程被杀时它借的显存靠这个回收）。
 
+use std::path::{Path, PathBuf};
+use std::sync::atomic::{AtomicU64, Ordering};
 use std::sync::{Condvar, Mutex, PoisonError};
+use std::time::{Duration, Instant};
 
 /// 一个判器一次派活最多带几个状态：显存装得下的、调用方要的、这条流一共几个状态，三者取小，至少 1。
 /// 草稿区按它开：状态数之外开的一个状态都用不上（一条 45 个状态的流按显存开满一张空着的 5090 是约 21 GiB，用得上的约 54 MiB）。
@@ -22,6 +32,24 @@ pub fn states_in_one_dispatch_capped_by_the_state_count(
         .max(1)
 }
 
+/// 一张卡的账能借出去多少（MiB）：设了 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（`card_quota_from_the_environment_mebibytes`）
+/// 就取「这个额度」与「这张卡清场后的空闲显存减驱动的固定开销」里小的那个；没设照旧是这张卡给崩溃放量的额度（`budget_mebibytes`，
+/// 配置的额度或空闲显存的一份）减驱动的固定开销（主 agent 2026-10-06 定：它是每张卡账的上限，不是每条用例的上限，不判红）。
+#[must_use]
+pub fn card_ledger_capacity_mebibytes(
+    card_quota_from_the_environment_mebibytes: Option<u64>,
+    budget_mebibytes: u64,
+    free_memory_after_quiescing_mebibytes: u64,
+    mebibytes_left_for_the_driver: u64,
+) -> u64 {
+    match card_quota_from_the_environment_mebibytes {
+        Some(quota) => quota.min(
+            free_memory_after_quiescing_mebibytes.saturating_sub(mebibytes_left_for_the_driver),
+        ),
+        None => budget_mebibytes.saturating_sub(mebibytes_left_for_the_driver),
+    }
+}
+
 /// 账上借一次显存的结局（[`decide_booking_by_priority`]）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum BookingDecision {
@@ -71,77 +99,511 @@ pub struct NeedLargerThanEveryCard {
     pub largest_capacity_mebibytes: u64,
 }
 
-/// 几张卡各一本显存账（MiB）：借的时候照 [`decide_booking_by_priority`] 记上，放不下就等别的借据还，借据丢掉时销账。
+/// 一笔借不到、等满 [`LONGEST_WAIT_FOR_GPU_MEMORY`] 才放弃：一条 GPU 用例在卡上判一条流以分钟计，等半小时还空不出来，
+/// 说明这台机器上别的 GPU 测试进程占得太久，这条用例报「本次未跑」，不干等到整趟被外面的限时掐掉（推的，没量过用例最长占多久）。
+pub const LONGEST_WAIT_FOR_GPU_MEMORY: Duration = Duration::from_secs(30 * 60);
+/// 等别的进程还显存时隔多久重读一次账本：别的进程还了叫不醒这个进程里等着的线程（同一个进程里还的当场叫醒，不等这一下）。
+pub const POLL_INTERVAL_FOR_MEMORY_RETURNED_BY_OTHER_PROCESSES: Duration = Duration::from_secs(1);
+/// 工作根下放账本的子目录。
+pub const GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME: &str = "gpu-memory-ledgers";
+/// 工作根的环境变量与它没设时在家目录下的默认位置：定义只在 `.claude/hooks/lib_work_root.py` 一处（`WORK_ROOT_VARIABLE`、
+/// `DEFAULT_WORK_ROOT`），这里照它取，两边要一起改。
+pub const WORK_ROOT_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_WORK_ROOT";
+pub const DEFAULT_WORK_ROOT_UNDER_THE_HOME_DIRECTORY: &str = ".local/share/singlefs/work";
+
+/// 账本放在哪个目录：工作根（`work_root_variable` 是 [`WORK_ROOT_ENVIRONMENT_VARIABLE`] 的值，设了而不空就用它，开头的 `~` 照家目录展开；
+/// 没设是家目录下的 [`DEFAULT_WORK_ROOT_UNDER_THE_HOME_DIRECTORY`]）下的 [`GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME`]。
+/// 要家目录而 `home_directory` 没给，交 `None`。
+#[must_use]
+pub fn gpu_memory_ledger_directory(
+    work_root_variable: Option<&std::ffi::OsStr>,
+    home_directory: Option<&std::ffi::OsStr>,
+) -> Option<PathBuf> {
+    let work_root = match work_root_variable.filter(|value| !value.is_empty()) {
+        Some(value) => {
+            let text = value.to_string_lossy();
+            match text.strip_prefix('~') {
+                Some(rest) if rest.is_empty() || rest.starts_with('/') => {
+                    PathBuf::from(home_directory?).join(rest.trim_start_matches('/'))
+                }
+                Some(_) | None => PathBuf::from(value),
+            }
+        }
+        None => PathBuf::from(home_directory?).join(DEFAULT_WORK_ROOT_UNDER_THE_HOME_DIRECTORY),
+    };
+    Some(work_root.join(GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME))
+}
+
+/// 这个进程的账本目录（照 [`gpu_memory_ledger_directory`]，读进程的 `SINGLEFS_WORK_ROOT` 与 `HOME`）。
+#[must_use]
+pub fn gpu_memory_ledger_directory_of_this_process() -> Option<PathBuf> {
+    gpu_memory_ledger_directory(
+        std::env::var_os(WORK_ROOT_ENVIRONMENT_VARIABLE).as_deref(),
+        std::env::var_os("HOME").as_deref(),
+    )
+}
+
+/// 账本里记的一个进程：进程号加它的启动时刻（`/proc/<进程号>/stat` 第 22 项，开机以来的时钟滴答数）；
+/// 进程号会被后起的进程重用，两样都对上才是同一个进程。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct RecordedProcess {
+    pub process_identifier: u32,
+    pub start_time_in_clock_ticks: u64,
+}
+
+impl RecordedProcess {
+    /// 跑这段代码的进程；读不到 `/proc/self/stat`（不是 Linux）交 `None`。
+    #[must_use]
+    pub fn this_process() -> Option<Self> {
+        let status_line = std::fs::read_to_string("/proc/self/stat").ok()?;
+        Some(Self {
+            process_identifier: std::process::id(),
+            start_time_in_clock_ticks: start_time_in_clock_ticks_from_the_process_status_line(
+                &status_line,
+            )?,
+        })
+    }
+}
+
+/// `/proc/<进程号>/stat` 的原文里进程的启动时刻（第 22 项）。第 2 项是括号里的进程名，名字里可以有空白与括号，从最后一个 `)` 之后数。
+#[must_use]
+pub fn start_time_in_clock_ticks_from_the_process_status_line(status_line: &str) -> Option<u64> {
+    let (_, after_the_name) = status_line.rsplit_once(')')?;
+    // 最后一个 `)` 之后第一项是第 3 项（进程状态），第 22 项是之后的第 20 项
+    after_the_name.split_whitespace().nth(19)?.parse().ok()
+}
+
+/// 记账的那个进程还在不在：`/proc/<进程号>/stat` 不在、或在而启动时刻对不上（进程号被别的进程重用了）就是不在了；
+/// 读得到文件却认不出、或读的时候出了别的错，当它还在（判不了就照记的算，宁可多等，不把别人还占着的显存算成空的）。
+#[must_use]
+pub fn process_is_alive_by_its_entry_in_the_process_filesystem(process: RecordedProcess) -> bool {
+    let path = format!("/proc/{}/stat", process.process_identifier);
+    match std::fs::read_to_string(&path) {
+        Ok(status_line) => start_time_in_clock_ticks_from_the_process_status_line(&status_line)
+            .is_none_or(|start_time| start_time == process.start_time_in_clock_ticks),
+        Err(error) => error.kind() != std::io::ErrorKind::NotFound,
+    }
+}
+
+/// 账本里的一笔借出去的显存。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct RecordedLease {
+    process: RecordedProcess,
+    lease_identifier: u64,
+    mebibytes: u64,
+}
+
+/// 一张卡的账本文件里记的。
+#[derive(Clone, Debug, Default, PartialEq, Eq)]
+struct CardLedgerRecords {
+    /// 头一个起池的进程读到的空闲显存（MiB）：那时这张卡上没有别的进程起着池。
+    free_memory_after_quiescing_mebibytes: Option<u64>,
+    /// 起了池的进程：每个各在卡上起了一个设备。
+    registered_processes: Vec<RecordedProcess>,
+    leases: Vec<RecordedLease>,
+}
+
+const FREE_MEMORY_LINE_KEY: &str = "free_memory_after_quiescing_mebibytes";
+const PROCESS_LINE_KEY: &str = "process";
+const LEASE_LINE_KEY: &str = "lease";
+
+impl CardLedgerRecords {
+    /// 账本文件的原文：一行一条，`free_memory_after_quiescing_mebibytes <MiB>`、`process <进程号> <启动时刻>`、
+    /// `lease <进程号> <启动时刻> <借据号> <MiB>`。
+    fn parse(text: &str) -> Result<Self, String> {
+        let mut records = Self::default();
+        for line in text.lines().filter(|line| !line.trim().is_empty()) {
+            let fields: Vec<&str> = line.split_whitespace().collect();
+            let number = |position: usize| -> Result<u64, String> {
+                fields
+                    .get(position)
+                    .and_then(|field| field.parse().ok())
+                    .ok_or_else(|| format!("账本的这一行认不出：「{line}」"))
+            };
+            let process = |first: usize| -> Result<RecordedProcess, String> {
+                Ok(RecordedProcess {
+                    process_identifier: u32::try_from(number(first)?)
+                        .map_err(|_| format!("进程号装不进 u32：「{line}」"))?,
+                    start_time_in_clock_ticks: number(first + 1)?,
+                })
+            };
+            match (fields[0], fields.len()) {
+                (FREE_MEMORY_LINE_KEY, 2) => {
+                    records.free_memory_after_quiescing_mebibytes = Some(number(1)?);
+                }
+                (PROCESS_LINE_KEY, 3) => records.registered_processes.push(process(1)?),
+                (LEASE_LINE_KEY, 5) => records.leases.push(RecordedLease {
+                    process: process(1)?,
+                    lease_identifier: number(3)?,
+                    mebibytes: number(4)?,
+                }),
+                (_, _) => return Err(format!("账本的这一行认不出：「{line}」")),
+            }
+        }
+        Ok(records)
+    }
+
+    fn to_text(&self) -> String {
+        let mut text = String::new();
+        if let Some(free) = self.free_memory_after_quiescing_mebibytes {
+            text.push_str(&format!("{FREE_MEMORY_LINE_KEY} {free}\n"));
+        }
+        for process in &self.registered_processes {
+            text.push_str(&format!(
+                "{PROCESS_LINE_KEY} {} {}\n",
+                process.process_identifier, process.start_time_in_clock_ticks
+            ));
+        }
+        for lease in &self.leases {
+            text.push_str(&format!(
+                "{LEASE_LINE_KEY} {} {} {} {}\n",
+                lease.process.process_identifier,
+                lease.process.start_time_in_clock_ticks,
+                lease.lease_identifier,
+                lease.mebibytes
+            ));
+        }
+        text
+    }
+
+    /// 去掉记账进程已经不在了的那几条（它起的池与借的显存都当没记）。
+    fn without_the_records_of_dead_processes(
+        mut self,
+        is_alive: &dyn Fn(RecordedProcess) -> bool,
+    ) -> Self {
+        self.registered_processes
+            .retain(|process| is_alive(*process));
+        self.leases.retain(|lease| is_alive(lease.process));
+        self
+    }
+
+    /// `viewer` 这个进程看这张卡已经记了多少（MiB）：每一笔借出去的，加别的每个起了池的进程各一份驱动的固定开销
+    /// （`viewer` 自己那一份已经从它的容量里扣过）。
+    fn booked_mebibytes_seen_by(
+        &self,
+        viewer: RecordedProcess,
+        mebibytes_each_other_process_keeps_on_a_card: u64,
+    ) -> u64 {
+        let other_processes = self
+            .registered_processes
+            .iter()
+            .filter(|process| **process != viewer)
+            .count();
+        self.leases
+            .iter()
+            .map(|lease| lease.mebibytes)
+            .sum::<u64>()
+            .saturating_add(
+                mebibytes_each_other_process_keeps_on_a_card
+                    .saturating_mul(u64::try_from(other_processes).expect("进程数装得进 u64")),
+            )
+    }
+}
+
+/// 账本文件读写出错：哪个文件、什么错。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct LedgerFileError {
+    pub path: PathBuf,
+    pub reason: String,
+}
+
+/// 记账的设置：账本放哪、这个进程是谁、怎么判记账进程还在不在（测试换成假的）、别的进程各在卡上付多少驱动的固定开销、
+/// 等别的进程还显存时隔多久重读、最多等多久。
+pub struct LedgerSettings {
+    pub directory: PathBuf,
+    pub this_process: RecordedProcess,
+    pub is_alive: Box<dyn Fn(RecordedProcess) -> bool + Send + Sync>,
+    pub mebibytes_each_other_process_keeps_on_a_card: u64,
+    pub poll_interval_for_memory_returned_by_other_processes: Duration,
+    pub longest_wait: Duration,
+}
+
+/// 要记账的一张卡：账本文件名的主干（只许字母、数字、`.`、`-`）与这个进程起池时读到的空闲显存。
+pub struct CardToRegister {
+    pub file_stem: String,
+    pub free_memory_read_by_this_process_mebibytes: u64,
+}
+
+/// 借不到的几种结局。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum BookingRefusal {
+    /// 要的比每张卡的容量都大：调用方判红。
+    NeedLargerThanEveryCard(NeedLargerThanEveryCard),
+    /// 放得下而等满了 [`LedgerSettings::longest_wait`] 还没空出来：调用方报「本次未跑」。
+    WaitedPastTheLongestWait {
+        need_mebibytes: u64,
+        waited: Duration,
+    },
+    /// 账本文件读写不了：调用方判红。
+    LedgerFileUnusable(LedgerFileError),
+}
+
+/// 几张卡各一本跨进程的显存账（MiB）：借的时候照 [`decide_booking_by_priority`] 记上，放不下就等别的借据还，借据丢掉时销账。
 pub struct GpuMemoryLedger {
+    settings: LedgerSettings,
+    card_file_stems: Vec<String>,
     capacities_mebibytes: Vec<u64>,
-    booked_mebibytes: Mutex<Vec<u64>>,
+    next_lease_identifier: AtomicU64,
+    /// 这个进程里还过几次：等着的线程记下看到的次数，次数变了就不睡（还在它判完、睡下之前时不漏叫）。
+    returns_in_this_process: Mutex<u64>,
     memory_returned: Condvar,
 }
 
+/// 锁住一张卡的账本、读出来（去掉死了的进程记的）、交 `change` 改，改过的写回去；交回 `change` 的结局。
+fn with_the_card_ledger_locked<Outcome>(
+    settings: &LedgerSettings,
+    file_stem: &str,
+    change: impl FnOnce(&mut CardLedgerRecords) -> Outcome,
+) -> Result<Outcome, LedgerFileError> {
+    let unusable = |path: &Path, reason: String| LedgerFileError {
+        path: path.to_path_buf(),
+        reason,
+    };
+    std::fs::create_dir_all(&settings.directory)
+        .map_err(|error| unusable(&settings.directory, format!("建不了目录：{error}")))?;
+    let lock_path = settings.directory.join(format!("{file_stem}.lock"));
+    let ledger_path = settings.directory.join(format!("{file_stem}.ledger"));
+    let lock = std::fs::OpenOptions::new()
+        .create(true)
+        .truncate(false)
+        .read(true)
+        .write(true)
+        .open(&lock_path)
+        .map_err(|error| unusable(&lock_path, format!("打不开锁文件：{error}")))?;
+    lock.lock()
+        .map_err(|error| unusable(&lock_path, format!("锁不上：{error}")))?;
+    let text = match std::fs::read_to_string(&ledger_path) {
+        Ok(text) => text,
+        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
+        Err(error) => return Err(unusable(&ledger_path, format!("读不了：{error}"))),
+    };
+    let as_read =
+        CardLedgerRecords::parse(&text).map_err(|reason| unusable(&ledger_path, reason))?;
+    let mut records = as_read
+        .clone()
+        .without_the_records_of_dead_processes(&settings.is_alive);
+    let outcome = change(&mut records);
+    if records != as_read {
+        let temporary_path = settings.directory.join(format!(
+            "{file_stem}.ledger.{}.writing",
+            settings.this_process.process_identifier
+        ));
+        std::fs::write(&temporary_path, records.to_text())
+            .map_err(|error| unusable(&temporary_path, format!("写不了：{error}")))?;
+        std::fs::rename(&temporary_path, &ledger_path)
+            .map_err(|error| unusable(&ledger_path, format!("换不上：{error}")))?;
+    }
+    drop(lock);
+    Ok(outcome)
+}
+
 impl GpuMemoryLedger {
-    /// 每张卡的容量（MiB），次序就是优先级。
-    #[must_use]
-    pub fn with_card_capacities(capacities_mebibytes: Vec<u64>) -> Self {
-        let booked_mebibytes = Mutex::new(vec![0; capacities_mebibytes.len()]);
-        Self {
+    /// 这个进程在每张卡的账本上记一笔「起了池」，交回账。这张卡上此刻没有别的进程起着池时，账本记下这个进程读到的空闲显存
+    /// （就是清场之后那一份）；有，就照账本里记的那一份。`capacity_for_judges(卡的次序, 那一份空闲显存)` 是这个进程看这张卡的容量。
+    ///
+    /// # Errors
+    /// 账本文件读写不了、认不出。
+    ///
+    /// # Panics
+    /// 文件名主干里有字母、数字、`.`、`-` 之外的字符。
+    pub fn register_this_process_on_the_cards(
+        settings: LedgerSettings,
+        cards: &[CardToRegister],
+        capacity_for_judges: impl Fn(usize, u64) -> u64,
+    ) -> Result<Self, LedgerFileError> {
+        let mut capacities_mebibytes = Vec::with_capacity(cards.len());
+        for (position, card) in cards.iter().enumerate() {
+            assert!(
+                !card.file_stem.is_empty()
+                    && card.file_stem.chars().all(|character| {
+                        character.is_ascii_alphanumeric() || character == '.' || character == '-'
+                    }),
+                "账本文件名主干只许字母、数字、`.`、`-`：「{}」",
+                card.file_stem
+            );
+            let this_process = settings.this_process;
+            let free_after_quiescing =
+                with_the_card_ledger_locked(&settings, &card.file_stem, |records| {
+                    records
+                        .registered_processes
+                        .retain(|process| *process != this_process);
+                    if records.registered_processes.is_empty()
+                        || records.free_memory_after_quiescing_mebibytes.is_none()
+                    {
+                        records.free_memory_after_quiescing_mebibytes =
+                            Some(card.free_memory_read_by_this_process_mebibytes);
+                    }
+                    records.registered_processes.push(this_process);
+                    records
+                        .free_memory_after_quiescing_mebibytes
+                        .expect("上面刚记过")
+                })?;
+            capacities_mebibytes.push(capacity_for_judges(position, free_after_quiescing));
+        }
+        Ok(Self {
+            settings,
+            card_file_stems: cards.iter().map(|card| card.file_stem.clone()).collect(),
             capacities_mebibytes,
-            booked_mebibytes,
+            next_lease_identifier: AtomicU64::new(0),
+            returns_in_this_process: Mutex::new(0),
             memory_returned: Condvar::new(),
-        }
+        })
     }
 
+    /// 这个进程看每张卡的容量（MiB），次序就是优先级。
     #[must_use]
     pub fn card_capacities_mebibytes(&self) -> &[u64] {
         &self.capacities_mebibytes
     }
 
-    /// 借 `need_mebibytes`：记得上就交回借据（丢掉时销账）；此刻放不下就等别的借据还，还一次重判一次；永远放不下交错误、什么都不记。
+    /// 每张卡此刻已经记了多少（照 [`CardLedgerRecords::booked_mebibytes_seen_by`]）。
+    fn booked_on_every_card(&self) -> Result<Vec<u64>, LedgerFileError> {
+        self.card_file_stems
+            .iter()
+            .map(|file_stem| {
+                with_the_card_ledger_locked(&self.settings, file_stem, |records| {
+                    records.booked_mebibytes_seen_by(
+                        self.settings.this_process,
+                        self.settings.mebibytes_each_other_process_keeps_on_a_card,
+                    )
+                })
+            })
+            .collect()
+    }
+
+    /// 锁住这张卡的账本重读一遍，此刻还放得下就记上，交回借据号；读完到锁上之间别的进程记了一笔、放不下了，交 `None`。
+    fn book_on_the_card_if_it_still_fits(
+        &self,
+        card_position: usize,
+        need_mebibytes: u64,
+    ) -> Result<Option<u64>, LedgerFileError> {
+        with_the_card_ledger_locked(
+            &self.settings,
+            &self.card_file_stems[card_position],
+            |records| {
+                let booked = records.booked_mebibytes_seen_by(
+                    self.settings.this_process,
+                    self.settings.mebibytes_each_other_process_keeps_on_a_card,
+                );
+                match decide_booking_by_priority(
+                    &[self.capacities_mebibytes[card_position]],
+                    &[booked],
+                    need_mebibytes,
+                ) {
+                    BookingDecision::BookOnTheCardAt { .. } => {
+                        let lease_identifier =
+                            self.next_lease_identifier.fetch_add(1, Ordering::Relaxed);
+                        records.leases.push(RecordedLease {
+                            process: self.settings.this_process,
+                            lease_identifier,
+                            mebibytes: need_mebibytes,
+                        });
+                        Some(lease_identifier)
+                    }
+                    BookingDecision::WaitForMemoryToBeReturned
+                    | BookingDecision::NeverFits { .. } => None,
+                }
+            },
+        )
+    }
+
+    /// 借 `need_mebibytes`：记得上就交回借据（丢掉时销账）；此刻放不下就等别的借据还（这个进程里还的当场叫醒，
+    /// 别的进程还的隔 [`LedgerSettings::poll_interval_for_memory_returned_by_other_processes`] 重读一次看到），
+    /// 等满 [`LedgerSettings::longest_wait`] 还放不下就放弃；永远放不下交错误、什么都不记。
     ///
     /// # Errors
-    /// 要的比每张卡的容量都大（[`NeedLargerThanEveryCard`]）。
+    /// 见 [`BookingRefusal`]。
     pub fn book_waiting_for_room(
         &self,
         need_mebibytes: u64,
-    ) -> Result<GpuMemoryBooking<'_>, NeedLargerThanEveryCard> {
-        // 账上只做加减，持锁的那一段不会 panic 到一半：别的线程 panic 留下的毒不影响账本身，照用
-        let mut booked = self
-            .booked_mebibytes
-            .lock()
-            .unwrap_or_else(PoisonError::into_inner);
+    ) -> Result<GpuMemoryBooking<'_>, BookingRefusal> {
+        let started = Instant::now();
         loop {
+            // 计数器上只做加一，持锁的那一段不会 panic 到一半：别的线程 panic 留下的毒不影响它，照用
+            let returns_seen = *self
+                .returns_in_this_process
+                .lock()
+                .unwrap_or_else(PoisonError::into_inner);
+            let booked = self
+                .booked_on_every_card()
+                .map_err(BookingRefusal::LedgerFileUnusable)?;
             match decide_booking_by_priority(&self.capacities_mebibytes, &booked, need_mebibytes) {
                 BookingDecision::BookOnTheCardAt { card_position } => {
-                    booked[card_position] += need_mebibytes;
-                    return Ok(GpuMemoryBooking {
-                        ledger: self,
-                        card_position,
-                        mebibytes: need_mebibytes,
-                    });
+                    if let Some(lease_identifier) = self
+                        .book_on_the_card_if_it_still_fits(card_position, need_mebibytes)
+                        .map_err(BookingRefusal::LedgerFileUnusable)?
+                    {
+                        return Ok(GpuMemoryBooking {
+                            ledger: self,
+                            card_position,
+                            mebibytes: need_mebibytes,
+                            lease_identifier,
+                        });
+                    }
                 }
                 BookingDecision::WaitForMemoryToBeReturned => {
-                    booked = self
-                        .memory_returned
-                        .wait(booked)
+                    let waited = started.elapsed();
+                    if waited >= self.settings.longest_wait {
+                        return Err(BookingRefusal::WaitedPastTheLongestWait {
+                            need_mebibytes,
+                            waited,
+                        });
+                    }
+                    let returns = self
+                        .returns_in_this_process
+                        .lock()
                         .unwrap_or_else(PoisonError::into_inner);
+                    if *returns == returns_seen {
+                        let sleep = self
+                            .settings
+                            .poll_interval_for_memory_returned_by_other_processes
+                            .min(self.settings.longest_wait - waited);
+                        drop(
+                            self.memory_returned
+                                .wait_timeout(returns, sleep)
+                                .unwrap_or_else(PoisonError::into_inner),
+                        );
+                    }
                 }
                 BookingDecision::NeverFits {
                     largest_capacity_mebibytes,
                 } => {
-                    return Err(NeedLargerThanEveryCard {
-                        need_mebibytes,
-                        largest_capacity_mebibytes,
-                    })
+                    return Err(BookingRefusal::NeedLargerThanEveryCard(
+                        NeedLargerThanEveryCard {
+                            need_mebibytes,
+                            largest_capacity_mebibytes,
+                        },
+                    ))
                 }
             }
         }
     }
+
+    /// 从这张卡的账本上删掉这个进程的这一笔。
+    fn remove_the_lease(
+        &self,
+        card_position: usize,
+        lease_identifier: u64,
+    ) -> Result<(), LedgerFileError> {
+        with_the_card_ledger_locked(
+            &self.settings,
+            &self.card_file_stems[card_position],
+            |records| {
+                records.leases.retain(|lease| {
+                    lease.process != self.settings.this_process
+                        || lease.lease_identifier != lease_identifier
+                });
+            },
+        )
+    }
 }
 
-/// 账上的一笔：记在哪张卡、多少 MiB；丢掉时销账、叫醒等着借的。
+/// 账上的一笔：记在哪张卡、多少 MiB；丢掉时销账、叫醒这个进程里等着借的。
 pub struct GpuMemoryBooking<'ledger> {
     ledger: &'ledger GpuMemoryLedger,
     card_position: usize,
     mebibytes: u64,
+    lease_identifier: u64,
 }
 
 impl GpuMemoryBooking<'_> {
@@ -159,15 +621,22 @@ impl GpuMemoryBooking<'_> {
 
 impl Drop for GpuMemoryBooking<'_> {
     fn drop(&mut self) {
-        let mut booked = self
+        // 销不了账只打一行：这一笔留在账本上，等这个进程退出、判活判成不在了时回收
+        if let Err(error) = self
+            .ledger
+            .remove_the_lease(self.card_position, self.lease_identifier)
+        {
+            eprintln!(
+                "GPU_MEMORY_LEDGER_RETURN_FAILED path={} reason={}",
+                error.path.display(),
+                error.reason
+            );
+        }
+        *self
             .ledger
-            .booked_mebibytes
+            .returns_in_this_process
             .lock()
-            .unwrap_or_else(PoisonError::into_inner);
-        booked[self.card_position] = booked[self.card_position]
-            .checked_sub(self.mebibytes)
-            .expect("一笔只在记上的那张卡上销一次：已记的不会比这一笔还少");
-        drop(booked);
+            .unwrap_or_else(PoisonError::into_inner) += 1;
         self.ledger.memory_returned.notify_all();
     }
 }
@@ -175,9 +644,76 @@ impl Drop for GpuMemoryBooking<'_> {
 #[cfg(test)]
 mod tests {
     use super::{
-        decide_booking_by_priority, states_in_one_dispatch_capped_by_the_state_count,
-        BookingDecision, GpuMemoryLedger, NeedLargerThanEveryCard,
+        card_ledger_capacity_mebibytes, decide_booking_by_priority, gpu_memory_ledger_directory,
+        process_is_alive_by_its_entry_in_the_process_filesystem,
+        states_in_one_dispatch_capped_by_the_state_count, BookingDecision, BookingRefusal,
+        CardToRegister, GpuMemoryLedger, LedgerSettings, NeedLargerThanEveryCard, RecordedProcess,
+        GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME,
     };
+    use std::ffi::OsStr;
+    use std::path::{Path, PathBuf};
+    use std::time::Duration;
+
+    /// 这条用例自己的账本目录（临时目录下，带用例名与进程号）：先删掉上一趟留下的，用例收尾时删。
+    struct LedgerDirectory {
+        path: PathBuf,
+    }
+
+    impl LedgerDirectory {
+        fn for_the_test(test_name: &str) -> Self {
+            let path = std::env::temp_dir().join(format!(
+                "singlefs-gpu-memory-ledger-{test_name}-{}",
+                std::process::id()
+            ));
+            let _ = std::fs::remove_dir_all(&path);
+            Self { path }
+        }
+    }
+
+    impl Drop for LedgerDirectory {
+        fn drop(&mut self) {
+            let _ = std::fs::remove_dir_all(&self.path);
+        }
+    }
+
+    /// 一个假的记账进程（进程号就是给的数，启动时刻固定 1）。
+    fn fake_process(process_identifier: u32) -> RecordedProcess {
+        RecordedProcess {
+            process_identifier,
+            start_time_in_clock_ticks: 1,
+        }
+    }
+
+    /// 一个假进程在 `directory` 下记一张卡（文件名主干 `card-test`、容量 `capacity_mebibytes`）的账：判活只认 `alive` 里的进程号，
+    /// 别的进程各在卡上付 `mebibytes_each_other_process_keeps_on_a_card`。
+    fn ledger_of_a_fake_process(
+        directory: &Path,
+        process_identifier: u32,
+        alive: &'static [u32],
+        capacity_mebibytes: u64,
+        mebibytes_each_other_process_keeps_on_a_card: u64,
+        poll_interval_for_memory_returned_by_other_processes: Duration,
+        longest_wait: Duration,
+    ) -> GpuMemoryLedger {
+        GpuMemoryLedger::register_this_process_on_the_cards(
+            LedgerSettings {
+                directory: directory.to_path_buf(),
+                this_process: fake_process(process_identifier),
+                is_alive: Box::new(move |process: RecordedProcess| {
+                    alive.contains(&process.process_identifier)
+                }),
+                mebibytes_each_other_process_keeps_on_a_card,
+                poll_interval_for_memory_returned_by_other_processes,
+                longest_wait,
+            },
+            &[CardToRegister {
+                file_stem: "card-test".to_string(),
+                free_memory_read_by_this_process_mebibytes: 1000,
+            }],
+            |_position, _free_after_quiescing| capacity_mebibytes,
+        )
+        .expect("临时目录下的账本读写得了")
+    }
 
     /// 一张空着的 5090 照 3/4 额度装得下 17 867 个状态（2026-10-06 日志里那一行）：45 个状态的流只开 45 个，
     /// 131 195 个状态的大流照旧按显存开，调用方要得更少就照它的，一点显存都不给、流是空的也带一个。
@@ -218,6 +754,29 @@ mod tests {
         );
     }
 
+    /// 设了卡的额度变量：账的容量取额度与「清场后的空闲显存 − 驱动那一份」里小的那个（额度小取额度，额度大取空闲显存那一边）；
+    /// 没设：照旧是这张卡给崩溃放量的额度减驱动那一份，与空闲显存无关。数照一张 5090（清场后空 30 000 MiB，3/4 是 22 500）。
+    #[test]
+    fn a_card_quota_from_the_environment_caps_each_card_ledger_and_without_it_the_capacity_is_unchanged(
+    ) {
+        const DRIVER: u64 = 2560;
+        assert_eq!(
+            card_ledger_capacity_mebibytes(Some(4096), 22_500, 30_000, DRIVER),
+            4096,
+            "额度比空闲显存减驱动那一份小：取额度"
+        );
+        assert_eq!(
+            card_ledger_capacity_mebibytes(Some(40_000), 22_500, 30_000, DRIVER),
+            30_000 - DRIVER,
+            "额度比空闲显存减驱动那一份大：取空闲显存那一边"
+        );
+        assert_eq!(
+            card_ledger_capacity_mebibytes(None, 22_500, 30_000, DRIVER),
+            22_500 - DRIVER,
+            "没设额度：照旧是给崩溃放量的额度减驱动那一份"
+        );
+    }
+
     /// 两张卡（容量照 5090 与 5060 Ti 的量级）：前一张放得下就一直记在前一张上，前一张放不下这一笔才记到下一张。
     #[test]
     fn bookings_fill_the_first_card_before_the_next_card_takes_any() {
@@ -262,27 +821,47 @@ mod tests {
             },
             "一张卡都没有：永远放不下"
         );
-        let ledger = GpuMemoryLedger::with_card_capacities(capacities.to_vec());
+        let directory = LedgerDirectory::for_the_test("never-fits");
+        let ledger = ledger_of_a_fake_process(
+            &directory.path,
+            100,
+            &[100],
+            100,
+            0,
+            Duration::from_millis(20),
+            Duration::from_secs(60),
+        );
         assert_eq!(
             ledger.book_waiting_for_room(101).err(),
-            Some(NeedLargerThanEveryCard {
-                need_mebibytes: 101,
-                largest_capacity_mebibytes: 100
-            }),
+            Some(BookingRefusal::NeedLargerThanEveryCard(
+                NeedLargerThanEveryCard {
+                    need_mebibytes: 101,
+                    largest_capacity_mebibytes: 100
+                }
+            )),
             "账上借永远放不下的：当场交错误，不等"
         );
-        let whole_first_card = ledger
+        let whole_card = ledger
             .book_waiting_for_room(100)
-            .expect("交错误的那一次什么都没记：第一张卡还是空的");
-        assert_eq!(whole_first_card.card_position(), 0);
+            .expect("交错误的那一次什么都没记：卡还是空的");
+        assert_eq!(whole_card.card_position(), 0);
     }
 
-    /// 一张 100 MiB 的卡被一笔记满，另一条线程再借 30：还没还之前它一直等着，前一笔还回来它就记上同一张卡。
+    /// 同一个进程里：一张 100 MiB 的卡被一笔记满，另一条线程再借 30，还没还之前它一直等着，前一笔还回来它就记上同一张卡。
+    /// 重读账本的间隔设成 10 分钟：前一笔还的时候要当场叫醒等着的线程，不靠重读。
     #[test]
     fn a_booking_that_does_not_fit_waits_until_an_earlier_one_is_returned() {
+        let directory = LedgerDirectory::for_the_test("same-process");
         // 等着借的那条线程拿的是账的借用；判红时它可能永远等着，账交给 'static、线程不回收，判红不卡住整个用例
-        let ledger: &'static GpuMemoryLedger =
-            Box::leak(Box::new(GpuMemoryLedger::with_card_capacities(vec![100])));
+        let ledger: &'static GpuMemoryLedger = Box::leak(Box::new(ledger_of_a_fake_process(
+            &directory.path,
+            100,
+            &[100],
+            100,
+            0,
+            Duration::from_secs(600),
+            Duration::from_secs(3600),
+        )));
         let first = ledger
             .book_waiting_for_room(100)
             .expect("空着的卡放得下 100");
@@ -308,4 +887,204 @@ mod tests {
             "前一笔还回来之后，等着的那一笔记上同一张卡"
         );
     }
+
+    /// 两个进程（两本账、两个假进程号，同一个账本目录）各起一次池、各记一份：容量 100，第一个进程借 60，
+    /// 第二个进程看到的是 60 加第一个进程的驱动那一份 10，再借 40 放不下，等；第一个进程还了，第二个进程重读账本时记上。
+    #[test]
+    fn a_second_process_whose_booking_does_not_fit_waits_until_the_first_process_returns_its_booking(
+    ) {
+        let directory = LedgerDirectory::for_the_test("two-processes");
+        let first_process = ledger_of_a_fake_process(
+            &directory.path,
+            100,
+            &[100, 200],
+            100,
+            10,
+            Duration::from_millis(20),
+            Duration::from_secs(3600),
+        );
+        let second_process: &'static GpuMemoryLedger =
+            Box::leak(Box::new(ledger_of_a_fake_process(
+                &directory.path,
+                200,
+                &[100, 200],
+                100,
+                10,
+                Duration::from_millis(20),
+                Duration::from_secs(3600),
+            )));
+        let first = first_process
+            .book_waiting_for_room(60)
+            .expect("空着的卡放得下 60");
+        let (sender, receiver) = std::sync::mpsc::channel();
+        std::thread::spawn(move || {
+            let second = second_process
+                .book_waiting_for_room(40)
+                .expect("40 比卡的容量小，放得下");
+            sender
+                .send((second.card_position(), second.mebibytes()))
+                .expect("用例还在等这一条");
+        });
+        assert!(
+            receiver
+                .recv_timeout(std::time::Duration::from_millis(300))
+                .is_err(),
+            "第一个进程借的 60 与它的驱动那一份 10 加上要的 40 放不下：第二个进程要等"
+        );
+        drop(first);
+        assert_eq!(
+            receiver.recv_timeout(std::time::Duration::from_secs(30)),
+            Ok((0, 40)),
+            "第一个进程还了之后，第二个进程重读账本时记上"
+        );
+    }
+
+    /// 别的进程一直占着、等满最长的等待还空不出来：放弃，交「等满了」（调用方据此报本次未跑），账上什么都不记。
+    #[test]
+    fn a_booking_that_cannot_get_room_before_the_longest_wait_gives_up() {
+        let directory = LedgerDirectory::for_the_test("gives-up");
+        let first_process = ledger_of_a_fake_process(
+            &directory.path,
+            100,
+            &[100, 200],
+            100,
+            0,
+            Duration::from_millis(20),
+            Duration::from_secs(3600),
+        );
+        let second_process: &'static GpuMemoryLedger =
+            Box::leak(Box::new(ledger_of_a_fake_process(
+                &directory.path,
+                200,
+                &[100, 200],
+                100,
+                0,
+                Duration::from_millis(20),
+                Duration::from_millis(200),
+            )));
+        let _held = first_process
+            .book_waiting_for_room(100)
+            .expect("空着的卡放得下 100");
+        let (sender, receiver) = std::sync::mpsc::channel();
+        std::thread::spawn(move || {
+            let outcome = second_process
+                .book_waiting_for_room(30)
+                .map(|booking| booking.mebibytes());
+            sender.send(outcome).expect("用例还在等这一条");
+        });
+        match receiver.recv_timeout(std::time::Duration::from_secs(30)) {
+            Ok(Err(BookingRefusal::WaitedPastTheLongestWait {
+                need_mebibytes,
+                waited,
+            })) => {
+                assert_eq!(need_mebibytes, 30);
+                assert!(
+                    waited >= Duration::from_millis(200),
+                    "等满了才放弃：只等了 {waited:?}"
+                );
+            }
+            other => panic!("等满最长的等待要放弃，交「等满了」；实际 {other:?}"),
+        }
+    }
+
+    /// 记账进程已经不在了，它记的那一份不算：进程 300 借满 90（借据丢掉不还，就是它被杀了），还当它活着的进程 400 借 90 等到放弃；
+    /// 判活判成它不在了的进程 500 当场借到，账本里 300 记的那几行跟着删掉。
+    #[test]
+    fn a_booking_left_by_a_process_that_is_no_longer_alive_is_not_counted() {
+        let directory = LedgerDirectory::for_the_test("dead-process");
+        let killed_process = ledger_of_a_fake_process(
+            &directory.path,
+            300,
+            &[300, 400, 500],
+            100,
+            0,
+            Duration::from_millis(20),
+            Duration::from_secs(3600),
+        );
+        std::mem::forget(
+            killed_process
+                .book_waiting_for_room(90)
+                .expect("空着的卡放得下 90"),
+        );
+        let sees_it_alive = ledger_of_a_fake_process(
+            &directory.path,
+            400,
+            &[300, 400],
+            100,
+            0,
+            Duration::from_millis(20),
+            Duration::from_millis(200),
+        );
+        assert!(
+            matches!(
+                sees_it_alive.book_waiting_for_room(90).err(),
+                Some(BookingRefusal::WaitedPastTheLongestWait { .. })
+            ),
+            "进程 300 还活着时它借的 90 照算：再借 90 放不下"
+        );
+        let sees_it_dead = ledger_of_a_fake_process(
+            &directory.path,
+            500,
+            &[500],
+            100,
+            0,
+            Duration::from_millis(20),
+            Duration::from_millis(200),
+        );
+        let booking = sees_it_dead
+            .book_waiting_for_room(90)
+            .expect("进程 300 不在了，它借的那一份不算：90 当场放得下");
+        assert_eq!(booking.card_position(), 0);
+        let ledger_text =
+            std::fs::read_to_string(directory.path.join("card-test.ledger")).expect("账本在");
+        assert!(
+            !ledger_text
+                .lines()
+                .any(|line| line.split_whitespace().nth(1) == Some("300")),
+            "不在了的进程 300 记的行在下一次写账时删掉：\n{ledger_text}"
+        );
+    }
+
+    /// 跑这条用例的进程判成还在；同一个进程号配上另一个启动时刻（进程号被后起的进程重用）判成不在。
+    #[test]
+    fn the_process_running_the_test_is_alive_and_its_identifier_with_another_start_time_is_not() {
+        let this_process = RecordedProcess::this_process().expect("Linux 上读得到 /proc/self/stat");
+        assert!(
+            process_is_alive_by_its_entry_in_the_process_filesystem(this_process),
+            "跑用例的进程还在"
+        );
+        assert!(
+            !process_is_alive_by_its_entry_in_the_process_filesystem(RecordedProcess {
+                process_identifier: this_process.process_identifier,
+                start_time_in_clock_ticks: this_process.start_time_in_clock_ticks + 1,
+            }),
+            "进程号对上、启动时刻对不上：是别的进程，原来记账的那个不在了"
+        );
+    }
+
+    /// 账本目录在工作根下：设了 `SINGLEFS_WORK_ROOT` 照它（开头的 `~` 照家目录展开），设成空串与没设一样落到家目录下的默认工作根，
+    /// 要家目录而没有家目录交 `None`。
+    #[test]
+    fn the_ledger_directory_is_under_the_work_root_variable_or_the_default_work_root_under_home() {
+        let home = Some(OsStr::new("/home/someone"));
+        assert_eq!(
+            gpu_memory_ledger_directory(Some(OsStr::new("/srv/work")), home),
+            Some(PathBuf::from("/srv/work").join(GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME))
+        );
+        assert_eq!(
+            gpu_memory_ledger_directory(Some(OsStr::new("~/work")), home),
+            Some(PathBuf::from("/home/someone/work").join(GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME))
+        );
+        for unset in [None, Some(OsStr::new(""))] {
+            assert_eq!(
+                gpu_memory_ledger_directory(unset, home),
+                Some(
+                    PathBuf::from("/home/someone/.local/share/singlefs/work")
+                        .join(GPU_MEMORY_LEDGER_SUBDIRECTORY_NAME)
+                ),
+                "工作根没设（{unset:?}）：家目录下的默认工作根"
+            );
+        }
+        assert_eq!(gpu_memory_ledger_directory(None, None), None);
+    }
 }
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs b/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
index 5592d76..3365f95 100644
--- a/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
@@ -27,7 +27,7 @@ use singlefs_checker_tier::crash_amplification::{
 };
 use singlefs_checker_tier::crash_identity::CoverageReport;
 #[cfg(feature = "gpu")]
-use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run;
+use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
 use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;
 use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
 use singlefs_harness::memory_pool::{
@@ -954,16 +954,18 @@ fn a_run_importing_another_library_reports_the_states_judged_before_the_import_a
 
 /// L1：只排派活计划的那一趟照实报录入时库里已判的态数，不写 0。同一份库上第一趟（单机 CPU）把整条流判完；第二趟配置开 GPU、
 /// 登记 `crash_judge_gpu`、只排计划：一块都不判，复用数等于整条流的态数。要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」
-/// （`gpu_unit_checks::gpu_test_start`），有适配器而一张卡都起不来判红。要 `verdict-store,gpu` 两个特性。
+/// （经这个进程的卡池判有没有卡，`gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run`），有适配器而一张卡都起不来判红。
+/// 只排计划的那一趟一个状态都不上卡判，这条用例不向卡池借显存。要 `verdict-store,gpu` 两个特性。
 #[cfg(feature = "gpu")]
 #[test]
 fn a_placement_only_run_reports_the_states_judged_at_recording_as_reused() {
-    let Some(card) = first_usable_card_or_report_not_run(
+    if gpu_card_pool_or_report_not_run(
         "a_placement_only_run_reports_the_states_judged_at_recording_as_reused",
-    ) else {
+    )
+    .is_none()
+    {
         return;
-    };
-    drop(card);
+    }
     let recorded =
         record_new_pool_file_creation("crash-amplification-placement-reuse", false, file_content());
     let directory = EnvironmentRunDirectory::new("placement-reuse");
diff --git a/crates/singlefs-checker-tier/tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs b/crates/singlefs-checker-tier/tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs
index 350b013..ec42f9a 100644
--- a/crates/singlefs-checker-tier/tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs
+++ b/crates/singlefs-checker-tier/tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs
@@ -3,6 +3,8 @@
 //! ③ 改坏一次单元写的一个字节之后，两份仍逐状态相等，那次写持久的每个状态都红，而且每个红状态在 CPU 判器上也红（核对红 ⇒ 判器红）。
 //! 要本机一张空闲显存够的 NVIDIA 独显：本机没有 NVIDIA 卡时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功，
 //! 有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）；要 `verdict-store,gpu` 特性。
+//! 卡经这个进程的卡池取（`gpu_test_start::gpu_card_pool_or_report_not_run`），抽完事实表、知道要多少显存之后按量借，
+//! 放不下就等别的用例与别的 GPU 测试进程还，等满了报「本次未跑」。
 #![cfg(all(feature = "verdict-store", feature = "gpu"))]
 
 #[path = "../../singlefs-harness/tests/common/mod.rs"]
@@ -18,7 +20,7 @@ use singlefs_checker_tier::crash_facts::{
     extract_flow_facts, verdict_bits, verify_states_from_facts, FlowFacts, BASE_IMAGE_SOURCE,
 };
 use singlefs_checker_tier::crash_verify_gpu::{gpu_verifier_digest, GpuFactsVerifier};
-use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run;
+use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
 use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
 use singlefs_harness::memory_pool::{
     writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
@@ -94,6 +96,23 @@ fn judge_red_ordinals(recorded: &Recorded) -> BTreeSet<u64> {
     reds
 }
 
+/// GPU 核对器在卡上要占多少显存（MiB，向上取整）：事实表每一项上卡展成 u32 表，展开之后不超过它在事实表 KV 字节
+/// （[`FlowFacts::to_bytes`]）里那一项的 [`LARGEST_EXPANSION_OF_A_FACTS_BYTE_ON_THE_CARD`] 倍；加一次核完整条流的结论与回读缓冲
+/// （每个状态各 4 字节）；再加 1 MiB 给派活的头与空表各占的一个字。
+fn mebibytes_a_facts_verifier_needs(facts: &FlowFacts) -> u64 {
+    let table_bytes = u64::try_from(facts.to_bytes().len()).expect("装得进 u64")
+        * LARGEST_EXPANSION_OF_A_FACTS_BYTE_ON_THE_CARD;
+    let verdict_and_readback_bytes = facts.state_count * 2 * 4;
+    (table_bytes + verdict_and_readback_bytes).div_ceil(1024 * 1024) + 1
+}
+
+/// 事实表 KV 里一个字节上卡之后最多占几个字节：每次写一个字节的可撕标志上卡是一行 4 个字（16 字节），别的项展开得都比它窄。
+const LARGEST_EXPANSION_OF_A_FACTS_BYTE_ON_THE_CARD: u64 = 16;
+
+/// 同一张卡上同时活着几个核对器：健康流那一个一直留到用例末尾，改坏一份的那一个、两份都改坏的那一个各再来一个；
+/// 三条流是同一条流改了几个字节，事实表一样大。
+const FACTS_VERIFIERS_ALIVE_AT_ONCE: u64 = 3;
+
 fn facts_of(recorded: &Recorded) -> FlowFacts {
     extract_flow_facts(
         &recorded.base,
@@ -106,13 +125,19 @@ fn facts_of(recorded: &Recorded) -> FlowFacts {
 #[test]
 fn gpu_and_cpu_verifiers_agree_on_every_state_and_a_flipped_unit_byte_is_red_on_both_and_on_the_judge(
 ) {
-    let Some(card) = first_usable_card_or_report_not_run(
-        "gpu_and_cpu_verifiers_agree_on_every_state_and_a_flipped_unit_byte_is_red_on_both_and_on_the_judge",
-    ) else {
+    const TEST_NAME: &str = "gpu_and_cpu_verifiers_agree_on_every_state_and_a_flipped_unit_byte_is_red_on_both_and_on_the_judge";
+    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
         return;
     };
     let healthy = record_new_pool_file_creation("crash-facts-gpu-healthy");
     let facts = facts_of(&healthy);
+    let Some(lease) = cards.lease_card_or_report_not_run(
+        TEST_NAME,
+        FACTS_VERIFIERS_ALIVE_AT_ONCE * mebibytes_a_facts_verifier_needs(&facts),
+    ) else {
+        return;
+    };
+    let card = lease.card();
     assert!(
         facts
             .units
@@ -134,7 +159,7 @@ fn gpu_and_cpu_verifiers_agree_on_every_state_and_a_flipped_unit_byte_is_red_on_
         "事实表往返 KV 的字节不丢"
     );
     let cpu = verify_states_from_facts(&facts, 0..facts.state_count);
-    let verifier = GpuFactsVerifier::new(&card, &facts);
+    let verifier = GpuFactsVerifier::new(card, &facts);
     let gpu = verifier.verify_states(0..facts.state_count);
     assert_eq!(cpu.len(), gpu.len());
     assert_eq!(cpu, gpu, "GPU 内核与 CPU 参照逐状态相等（健康流）");
@@ -213,12 +238,12 @@ fn gpu_and_cpu_verifiers_agree_on_every_state_and_a_flipped_unit_byte_is_red_on_
         "另一份位置条目还对得上，不红"
     );
     assert_eq!(
-        GpuFactsVerifier::new(&card, &half_facts).verify_states(0..half_facts.state_count),
+        GpuFactsVerifier::new(card, &half_facts).verify_states(0..half_facts.state_count),
         half_cpu
     );
     let broken_facts = facts_of(&broken);
     let broken_cpu = verify_states_from_facts(&broken_facts, 0..broken_facts.state_count);
-    let broken_verifier = GpuFactsVerifier::new(&card, &broken_facts);
+    let broken_verifier = GpuFactsVerifier::new(card, &broken_facts);
     let broken_gpu = broken_verifier.verify_states(0..broken_facts.state_count);
     assert_eq!(broken_cpu, broken_gpu, "改坏之后两份仍逐状态相等");
     let red_states: Vec<usize> = broken_cpu
diff --git a/crates/singlefs-checker-tier/tests/gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs b/crates/singlefs-checker-tier/tests/gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs
index 563608f..af73113 100644
--- a/crates/singlefs-checker-tier/tests/gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs
+++ b/crates/singlefs-checker-tier/tests/gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs
@@ -5,7 +5,8 @@
 //! 重封，在流尾另起一段写进发布的根的下一个根槽（读根环次序在它后面）。两条根都落了的状态上择到哪一条，池级 checker 的判定不同
 //! （前提在取卡之前用 CPU 判：两条根换位的镜像判出的违例不同），GPU 与 CPU 逐状态的违例位要相同。
 //! 要本机一张空闲显存够的 NVIDIA 独显：一张独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
-//! （`gpu_unit_checks::gpu_test_start`）。要 `gpu` 特性。
+//! （`gpu_unit_checks::gpu_test_start`）。卡经这个进程的卡池取，建好表之后按判器要的显存借
+//! （`crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool`），等满了借不到报「本次未跑」。要 `gpu` 特性。
 #![cfg(feature = "gpu")]
 
 #[path = "../../singlefs-harness/tests/common/mod.rs"]
@@ -19,12 +20,12 @@ use singlefs_checker::walk::check_pool_image;
 use singlefs_checker_tier::crash::{
     judge_layer0_state_range, layer0_plan_state_count, Layer0SegmentExpansion, Layer0StateJudgement,
 };
-use singlefs_checker_tier::crash_judge_dispatch::GpuCrashJudge;
+use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
 use singlefs_checker_tier::crash_judge_gpu::{
     pool_checker_violated_bits, verdict_bits, POOL_CHECKER_UNDECIDABLE_WORD, VERDICT_WORDS,
 };
 use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
-use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run;
+use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
 use singlefs_core::address::{CheckpointTxg, DeviceOffsetInBytes, InstanceGeneration};
 use singlefs_core::recovery::PoolReader;
 use singlefs_harness::memory_pool::{
@@ -223,9 +224,9 @@ fn gpu_judge_takes_the_earlier_of_two_roots_with_the_same_key_like_the_cpu_check
         violated_with_the_publish_root_first, violated_with_the_warm_up_trees_first,
         "两条同键的根换位之后池级 checker 判出的违例要不同，这条流才分得出择根平局"
     );
-    let Some(card) = first_usable_card_or_report_not_run(
-        "gpu_judge_takes_the_earlier_of_two_roots_with_the_same_key_like_the_cpu_checker",
-    ) else {
+    const TEST_NAME: &str =
+        "gpu_judge_takes_the_earlier_of_two_roots_with_the_same_key_like_the_cpu_checker";
+    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
         return;
     };
     let expansion = |_segment_index: usize, segment: &[usize]| {
@@ -246,8 +247,11 @@ fn gpu_judge_takes_the_earlier_of_two_roots_with_the_same_key_like_the_cpu_check
     .expect("建得出表");
     let state_count = layer0_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
     assert_eq!(tables.state_count, state_count);
-    let gpu: Vec<[u32; VERDICT_WORDS]> =
-        GpuCrashJudge::new(&card, &tables).judge_states(0..state_count);
+    let Some(gpu): Option<Vec<[u32; VERDICT_WORDS]>> =
+        judge_every_state_on_a_card_leased_from_the_pool(cards, TEST_NAME, &tables)
+    else {
+        return;
+    };
     let mut cpu: Vec<u128> = Vec::new();
     let judged_on_the_cpu = judge_layer0_state_range(
         &flow.base,
diff --git a/crates/singlefs-checker-tier/tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs b/crates/singlefs-checker-tier/tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs
index c1ee026..6f88193 100644
--- a/crates/singlefs-checker-tier/tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs
+++ b/crates/singlefs-checker-tier/tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs
@@ -7,7 +7,8 @@
 //! 与 harness 档 `checker_known_bad_images.rs` 里 I-7.8 最新那条根那一格的坏镜像同一个坏法，那一份钉 CPU 判器、这一份钉 GPU 判器）。
 //! B 的根落了的状态上 CPU 判 I-7.8 红，GPU 与 CPU 逐状态的违例位要相同；GPU 只判取 max 那一格时这些状态上少一位 I-7.8。
 //! 要本机一张空闲显存够的 NVIDIA 独显：一张独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
-//! （`gpu_unit_checks::gpu_test_start`）。要 `gpu` 特性。
+//! （`gpu_unit_checks::gpu_test_start`）。卡经这个进程的卡池取，建好表之后按判器要的显存借
+//! （`crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool`），等满了借不到报「本次未跑」。要 `gpu` 特性。
 #![cfg(feature = "gpu")]
 
 #[path = "../../singlefs-harness/tests/common/mod.rs"]
@@ -24,12 +25,12 @@ use singlefs_checker::walk::check_pool_image;
 use singlefs_checker_tier::crash::{
     judge_layer0_state_range, layer0_plan_state_count, Layer0SegmentExpansion, Layer0StateJudgement,
 };
-use singlefs_checker_tier::crash_judge_dispatch::GpuCrashJudge;
+use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
 use singlefs_checker_tier::crash_judge_gpu::{
     pool_checker_violated_bits, verdict_bits, POOL_CHECKER_UNDECIDABLE_WORD, VERDICT_WORDS,
 };
 use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
-use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run;
+use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
 use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
 use singlefs_core::journal::JournalRecord;
 use singlefs_core::root_record::RootRecord;
@@ -269,9 +270,9 @@ fn gpu_judge_reddens_a_newest_root_whose_own_tree_identifier_watermark_is_low_li
         "红的是最新那条根自己那一格、取 max 那一格没红，实际说明「{}」",
         lowered["I-7.8"]
     );
-    let Some(card) = first_usable_card_or_report_not_run(
-        "gpu_judge_reddens_a_newest_root_whose_own_tree_identifier_watermark_is_low_like_the_cpu_checker",
-    ) else {
+    const TEST_NAME: &str =
+        "gpu_judge_reddens_a_newest_root_whose_own_tree_identifier_watermark_is_low_like_the_cpu_checker";
+    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
         return;
     };
     let expansion = |_segment_index: usize, segment: &[usize]| {
@@ -292,8 +293,11 @@ fn gpu_judge_reddens_a_newest_root_whose_own_tree_identifier_watermark_is_low_li
     .expect("建得出表");
     let state_count = layer0_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
     assert_eq!(tables.state_count, state_count);
-    let gpu: Vec<[u32; VERDICT_WORDS]> =
-        GpuCrashJudge::new(&card, &tables).judge_states(0..state_count);
+    let Some(gpu): Option<Vec<[u32; VERDICT_WORDS]>> =
+        judge_every_state_on_a_card_leased_from_the_pool(cards, TEST_NAME, &tables)
+    else {
+        return;
+    };
     let mut cpu: Vec<u128> = Vec::new();
     let judged_on_the_cpu = judge_layer0_state_range(
         &flow.base,
diff --git a/crates/singlefs-checker-tier/tests/gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs b/crates/singlefs-checker-tier/tests/gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs
index 9bedffa..e53fbd1 100644
--- a/crates/singlefs-checker-tier/tests/gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs
+++ b/crates/singlefs-checker-tier/tests/gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs
@@ -9,7 +9,8 @@
 //! 实例表指针压后长度（压缩算法码仍 0）。前提在取卡之前用 CPU 判：每个写都落了的镜像上，改之前 core 择到发布的根，改之后退到它之前那条根。
 //! GPU 与 CPU 在每个状态上红位、池级 checker 违例位、两遍恢复落到的根与结局逐位相同，一个状态都不许判不了。
 //! 要本机一张空闲显存够的 NVIDIA 独显：一张独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
-//! （`gpu_unit_checks::gpu_test_start`）。要 `verdict-store,gpu` 特性。
+//! （`gpu_unit_checks::gpu_test_start`）。卡经这个进程的卡池取，建好表之后按判器要的显存借
+//! （`crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool`），等满了借不到报「本次未跑」。要 `verdict-store,gpu` 特性。
 #![cfg(all(feature = "verdict-store", feature = "gpu"))]
 
 #[path = "../../singlefs-harness/tests/common/mod.rs"]
@@ -22,12 +23,12 @@ use singlefs_checker_tier::crash::{
     Layer0SegmentExpansion, Layer0StateJudgement,
 };
 use singlefs_checker_tier::crash_amplification::{gpu_recovery_words, recovery_words_of};
-use singlefs_checker_tier::crash_judge_dispatch::GpuCrashJudge;
+use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
 use singlefs_checker_tier::crash_judge_gpu::{
     pool_checker_violated_bits, verdict_bits, POOL_CHECKER_UNDECIDABLE_WORD, VERDICT_WORDS,
 };
 use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
-use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run;
+use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
 use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
 use singlefs_core::recovery::{choose_root, choose_system_configuration};
 use singlefs_harness::memory_pool::{
@@ -287,9 +288,9 @@ struct JudgedOnTheCpu {
 }
 
 /// GPU 判器与 CPU 判器在这条流的每个状态上红位、池级 checker 违例位、两遍恢复落到的根与结局逐位相同，没有判不了的状态。
-/// 本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
+/// 本机没有 NVIDIA 独显适配器、或在卡池里等满了借不到卡时报「本次未跑」返回。
 fn assert_the_gpu_judge_agrees_with_the_cpu_on_every_state(test_name: &str, flow: &RecordedFlow) {
-    let Some(card) = first_usable_card_or_report_not_run(test_name) else {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
         return;
     };
     let expansion = |_segment_index: usize, segment: &[usize]| {
@@ -310,8 +311,11 @@ fn assert_the_gpu_judge_agrees_with_the_cpu_on_every_state(test_name: &str, flow
     .expect("建得出表");
     let state_count = layer0_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
     assert_eq!(tables.state_count, state_count);
-    let gpu: Vec<[u32; VERDICT_WORDS]> =
-        GpuCrashJudge::new(&card, &tables).judge_states(0..state_count);
+    let Some(gpu): Option<Vec<[u32; VERDICT_WORDS]>> =
+        judge_every_state_on_a_card_leased_from_the_pool(cards, test_name, &tables)
+    else {
+        return;
+    };
     let mut cpu: Vec<JudgedOnTheCpu> = Vec::new();
     let judged_on_the_cpu = judge_layer0_state_range(
         &flow.base,
diff --git a/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs b/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs
index 94a938f..64e6191 100644
--- a/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs
+++ b/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs
@@ -3,6 +3,7 @@
 //! 翻载荷里一个字节报载荷 CRC 不对、翻头里一个字节报头校验和不对；整条流水线开着 GPU 核一遍 0 红、0 对不上。
 //! 要 `gpu` 与 `verdict-store` 两个特性（配置 `ENABLE_GPU=1` 的跑法才开）；开了 `gpu` 而本机一张 NVIDIA 独显适配器都没有时打一行
 //! `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功，有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）。
+//! 卡经这个进程的卡池取，知道要核几个单元之后按 `gpu_unit_checks::mebibytes_crc32c_of_ranges_needs` 借，等满了借不到报「本次未跑」。
 #![cfg(all(feature = "gpu", feature = "verdict-store"))]
 
 #[path = "../../singlefs-harness/tests/common/mod.rs"]
@@ -15,13 +16,15 @@ use common::{build_pool, file_content, geometry};
 use singlefs_checker::crc32_castagnoli_table;
 use singlefs_checker_tier::crash::Layer0SegmentExpansion;
 use singlefs_checker_tier::crash_amplification::{
-    check_recorded_blocks, plan_crash_points, record_crash_points, CrashFlow, CrashPointSpan,
-    PipelineJudgingCode, UnitChecksumsOn, WeightedShares,
+    check_recorded_blocks, checking_threads, plan_crash_points, record_crash_points, CrashFlow,
+    CrashPointSpan, PipelineJudgingCode, UnitChecksumsOn, WeightedShares,
 };
 use singlefs_checker_tier::crash_identity::CoverageReport;
-use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run;
+use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
+    gpu_card_pool_or_report_not_run, GpuCardLease, GpuCardPool,
+};
 use singlefs_checker_tier::gpu_unit_checks::{
-    check_unit_checksums_on_gpu, GpuCard, UnitChecksumVerdict,
+    check_unit_checksums_on_gpu, mebibytes_crc32c_of_ranges_needs, UnitChecksumVerdict,
 };
 use singlefs_checker_tier::layer0_progress::Layer0ToolchainIdentity;
 use singlefs_checker_tier::verdict_store::VerdictStore;
@@ -33,14 +36,27 @@ use singlefs_harness::segments::StepKind;
 
 const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs";
 
-/// 取空闲显存最大的那张卡；本机没有 NVIDIA 卡时打「本次未跑」那一行、交回 `None`，用例当场返回。
-fn one_usable_card_or_report_not_run(test_name: &str) -> Option<GpuCard> {
-    let card = first_usable_card_or_report_not_run(test_name)?;
+/// 在卡池里借一张卡，够 `checks_at_once` 批各核 `units_per_check` 个单元的两道校验和同时在卡上（每个单元两段字节）；
+/// 借到打一行借的是哪张卡，等满了借不到时借卡那一层打「本次未跑」那一行、这里交 `None`（用例当场返回）。
+fn lease_for_unit_checks_or_report_not_run(
+    cards: &'static GpuCardPool,
+    test_name: &str,
+    units_per_check: usize,
+    checks_at_once: u64,
+) -> Option<GpuCardLease> {
+    let lease = cards.lease_card_or_report_not_run(
+        test_name,
+        checks_at_once * mebibytes_crc32c_of_ranges_needs(2 * units_per_check),
+    )?;
+    let card = lease.card();
     println!(
-        "GPU_CARD name={:?} pci={} free_mebibytes={}",
-        card.name, card.pci_bus_identifier, card.free_memory_mebibytes
+        "GPU_CARD name={:?} pci={} free_mebibytes={} lent_mebibytes={}",
+        card.name,
+        card.pci_bus_identifier,
+        card.free_memory_mebibytes,
+        lease.mebibytes()
     );
-    Some(card)
+    Some(lease)
 }
 
 struct Recorded {
@@ -105,13 +121,18 @@ fn every_unit(recorded: &Recorded) -> Vec<Vec<u8>> {
 
 #[test]
 fn gpu_crc32c_matches_the_cpu_on_every_unit_and_both_checksums_of_every_healthy_unit_hold() {
-    let Some(card) = one_usable_card_or_report_not_run(
-        "gpu_crc32c_matches_the_cpu_on_every_unit_and_both_checksums_of_every_healthy_unit_hold",
-    ) else {
+    const TEST_NAME: &str =
+        "gpu_crc32c_matches_the_cpu_on_every_unit_and_both_checksums_of_every_healthy_unit_hold";
+    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
         return;
     };
     let recorded = record("gpu-unit-checks-healthy");
     let units = every_unit(&recorded);
+    let Some(lease) = lease_for_unit_checks_or_report_not_run(cards, TEST_NAME, units.len(), 1)
+    else {
+        return;
+    };
+    let card = lease.card();
     assert!(
         units.len() >= 20,
         "新池新建文件至少写 20 个单元，实际 {}",
@@ -126,7 +147,7 @@ fn gpu_crc32c_matches_the_cpu_on_every_unit_and_both_checksums_of_every_healthy_
             "GPU 与 CPU 的 CRC-32C 逐个相同"
         );
     }
-    let verdicts = check_unit_checksums_on_gpu(&card, &borrowed);
+    let verdicts = check_unit_checksums_on_gpu(card, &borrowed);
     assert!(
         verdicts
             .iter()
@@ -138,19 +159,23 @@ fn gpu_crc32c_matches_the_cpu_on_every_unit_and_both_checksums_of_every_healthy_
 
 #[test]
 fn a_flipped_payload_byte_and_a_flipped_header_byte_are_each_caught_on_the_gpu() {
-    let Some(card) = one_usable_card_or_report_not_run(
-        "a_flipped_payload_byte_and_a_flipped_header_byte_are_each_caught_on_the_gpu",
-    ) else {
+    const TEST_NAME: &str =
+        "a_flipped_payload_byte_and_a_flipped_header_byte_are_each_caught_on_the_gpu";
+    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
         return;
     };
     let recorded = record("gpu-unit-checks-flipped");
     let mut units = every_unit(&recorded);
+    let Some(lease) = lease_for_unit_checks_or_report_not_run(cards, TEST_NAME, units.len(), 1)
+    else {
+        return;
+    };
     let last = units.len() - 1;
     let last_byte_of_first_unit = units[0].len() - 1;
     units[0][last_byte_of_first_unit] ^= 0x01;
     units[last][8] ^= 0x01;
     let borrowed: Vec<&[u8]> = units.iter().map(Vec::as_slice).collect();
-    let verdicts = check_unit_checksums_on_gpu(&card, &borrowed);
+    let verdicts = check_unit_checksums_on_gpu(lease.card(), &borrowed);
     assert_eq!(
         verdicts[0],
         UnitChecksumVerdict::PayloadCrcMismatch,
@@ -168,13 +193,25 @@ fn a_flipped_payload_byte_and_a_flipped_header_byte_are_each_caught_on_the_gpu()
 
 #[test]
 fn the_pipeline_with_gpu_unit_checks_reports_no_red_and_no_disagreement_on_the_healthy_flow() {
-    let Some(card) = one_usable_card_or_report_not_run(
-        "the_pipeline_with_gpu_unit_checks_reports_no_red_and_no_disagreement_on_the_healthy_flow",
-    ) else {
+    const TEST_NAME: &str =
+        "the_pipeline_with_gpu_unit_checks_reports_no_red_and_no_disagreement_on_the_healthy_flow";
+    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
         return;
     };
-    let card = Arc::new(card);
     let recorded = record("gpu-pipeline");
+    // 流水线按核对线程并行判块，每条线程判一个状态时把这个状态里落了的单元写交卡核一批：一批最多是这条流的全部单元写
+    let unit_writes = recorded
+        .writes
+        .iter()
+        .filter(|write| write.kind == StepKind::UnitWrite)
+        .count();
+    let threads = u64::try_from(checking_threads()).expect("线程数装得进 u64");
+    let Some(lease) =
+        lease_for_unit_checks_or_report_not_run(cards, TEST_NAME, unit_writes, threads)
+    else {
+        return;
+    };
+    let card = Arc::new(lease.card().clone());
     let expansion = |_segment_index: usize, segment: &[usize]| {
         if segment.len() <= 6 {
             Layer0SegmentExpansion::EveryProperSubset
diff --git a/crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs b/crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs
index 9e8df55..54b6fae 100644
--- a/crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs
+++ b/crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs
@@ -1057,6 +1057,8 @@ fn sigma_publish_writes(recorded: &RecordedHistory) -> std::ops::Range<usize> {
 /// （它是今天唯一一条一个落点超过 32 次写的流）。只展开 σ 那次发布的段，其余段不展开；每个状态 GPU 的红项与 CPU 判器的
 /// `red_items` 逐项同名、同序，GPU 一个状态都不许判不了。要一张空闲显存够的 NVIDIA 独显：本机没有 NVIDIA 卡时打一行
 /// `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功，有卡而用不上判红（`gpu_unit_checks::gpu_test_start`）。
+/// 卡经这个进程的卡池取，建好表之后按判器要的显存借（`crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool`），
+/// 等满了借不到也报「本次未跑」。
 #[cfg(all(feature = "gpu", feature = "verdict-store"))]
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_judge_on_sigma_whose_configuration_slot_is_written_more_than_thirty_two_times(
@@ -1065,9 +1067,9 @@ fn the_gpu_judge_agrees_with_the_cpu_judge_on_sigma_whose_configuration_slot_is_
     use singlefs_checker_tier::crash_amplification::{
         gpu_recovery_words, gpu_verdict_items, recovery_words_of,
     };
-    use singlefs_checker_tier::crash_judge_dispatch::GpuCrashJudge;
+    use singlefs_checker_tier::crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool;
     use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
-    use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run;
+    use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
     let recorded = record_history("gpu-judge-sigma", &HISTORY_UOOUOMSU);
     let sigma_publish = sigma_publish_writes(&recorded);
     let expansion = |_segment_index: usize, segment: &[usize]| {
@@ -1123,13 +1125,14 @@ fn the_gpu_judge_agrees_with_the_cpu_judge_on_sigma_whose_configuration_slot_is_
         versions_told_apart_only_by_the_high_bits > 0,
         "这条对拍要有两个版本低 32 位相同、只差高位，不然只比低 32 位的内核也判得对"
     );
-    let Some(card) = first_usable_card_or_report_not_run(
-        "the_gpu_judge_agrees_with_the_cpu_judge_on_sigma_whose_configuration_slot_is_written_more_than_thirty_two_times",
-    ) else {
+    const TEST_NAME: &str = "the_gpu_judge_agrees_with_the_cpu_judge_on_sigma_whose_configuration_slot_is_written_more_than_thirty_two_times";
+    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
+        return;
+    };
+    let Some(gpu) = judge_every_state_on_a_card_leased_from_the_pool(cards, TEST_NAME, &tables)
+    else {
         return;
     };
-    let judge = GpuCrashJudge::new(&card, &tables);
-    let gpu = judge.judge_states(0..tables.state_count);
     // CPU 判器按区间切片并行判（每个状态十几到几十毫秒，单线程判 6.5 万态要一个多小时）：线程数从
     // SINGLEFS_CRASH_JUDGE_CPU_THREADS 取，没设取本机可用并行度；每线程 8 片，按片序拼回，与单线程逐状态相同
     let threads = cpu_threads_from_the_value_of_the_variable(
diff --git a/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs b/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs
index f13ca2a..1b87089 100644
--- a/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs
+++ b/crates/singlefs-checker-tier/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs
@@ -6,6 +6,8 @@
 //! 在有卡的机器上与经入口走同一条路，跑用例分不出；这一条扫源码，不要显卡。
 //! 扫的是 `tests/` 下每份 `.rs`（这一份除外）整份，与 `src/` 下每份 `.rs` 里顶格 `#[cfg(test)]` 下的模块与函数；
 //! 产品路径（崩溃放量取卡、`gpu_unit_checks` 自己）不在测试代码里，不扫。行里 `//` 之后的注释不扫。
+//! 改走卡池的几份测试文件（[`FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL`]）另判一层：只许经 `gpu_card_pool_or_report_not_run` 取卡、
+//! 按量借，不许再各自拿第一张能用的卡、拿整份卡表或直接在卡上建判器（各按空闲显存的比例开，几个用例并行时互抢显存）。
 
 use std::path::{Path, PathBuf};
 
@@ -24,6 +26,27 @@ const CARD_TAKING_CALLS_THROUGH_THE_ENTRY: [&str; 3] = [
     "gpu_card_pool_or_report_not_run(",
 ];
 
+/// 改走卡池的测试文件（相对包根）：每份至少一处经 `gpu_card_pool_or_report_not_run(` 取卡，[`CALLS_THAT_BYPASS_THE_POOL`] 一处都不许有。
+const FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL: [&str; 7] = [
+    "tests/crash_amplification_pipeline_records_checks_and_reuses.rs",
+    "tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs",
+    "tests/gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs",
+    "tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs",
+    "tests/gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs",
+    "tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs",
+    "tests/record_checker_judges_absence_by_the_persisted_set.rs",
+];
+
+/// 绕开卡池拿卡、建判器的调用：经入口拿第一张能用的卡、拿整份卡表，或拿一张卡直接建判器（判器照那张卡的空闲显存开草稿区，不记账）。
+const CALLS_THAT_BYPASS_THE_POOL: [&str; 3] = [
+    "first_usable_card_or_report_not_run(",
+    "usable_cards_or_report_not_run(",
+    "GpuCrashJudge::new(",
+];
+
+/// 改走卡池的文件里经卡池取卡的调用。
+const CARD_POOL_CALL: &str = "gpu_card_pool_or_report_not_run(";
+
 /// 经入口判过「有卡、照跑」之后再起卡、准许绕开的几处：(相对包根的文件, 那一行去掉前后空白的代码, 为什么准许)。
 /// 每一项必须恰好命中一次，命中不到也判红：准许的那一行改了或删了，这里不许还留着。
 const BYPASSES_ALLOWED_AFTER_THE_ENTRY_DECIDED: [(&str, &str, &str); 1] = [(
@@ -218,3 +241,51 @@ fn every_gpu_test_takes_its_cards_through_gpu_test_start() {
          改成经 gpu_card_pool_or_report_not_run（共用一池卡、按量借）、usable_cards_or_report_not_run 或 first_usable_card_or_report_not_run 取卡：{not_allowed:#?}"
     );
 }
+
+#[test]
+fn every_test_file_moved_onto_the_card_pool_takes_its_cards_only_from_the_pool() {
+    let package_root = Path::new(env!("CARGO_MANIFEST_DIR"));
+    let mut bypasses_of_the_pool = Vec::new();
+    let mut files_without_a_pool_call = Vec::new();
+    for file in FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL {
+        let path = package_root.join(file);
+        let source = std::fs::read_to_string(&path).unwrap_or_else(|error| {
+            panic!(
+                "{} 读不出：{error}（改了名就把这一项一起改）",
+                path.display()
+            )
+        });
+        let lines = code_lines(&source);
+        if !lines.iter().any(|(_, code)| code.contains(CARD_POOL_CALL)) {
+            files_without_a_pool_call.push(file);
+        }
+        bypasses_of_the_pool.extend(
+            lines
+                .iter()
+                .filter(|(_, code)| {
+                    CALLS_THAT_BYPASS_THE_POOL
+                        .iter()
+                        .any(|call| code.contains(call))
+                })
+                .map(|(line_number, code)| BypassingLine {
+                    file_relative_to_the_package: file.to_string(),
+                    line_number: *line_number,
+                    code: code.trim().to_string(),
+                }),
+        );
+    }
+    assert!(
+        files_without_a_pool_call.is_empty(),
+        "这几份改走卡池的文件里一处 {CARD_POOL_CALL} 都没有：{files_without_a_pool_call:?}"
+    );
+    assert!(
+        bypasses_of_the_pool.is_empty(),
+        "这几处在改走卡池的文件里绕开卡池拿卡或直接建判器（不记账，几个用例并行时互抢显存）：\
+         改成经 gpu_card_pool_or_report_not_run 取池、建好表之后按量借（GpuCardPool::lease_card_or_report_not_run，\
+         判器用 crash_judge_dispatch::judge_every_state_on_a_card_leased_from_the_pool）：{bypasses_of_the_pool:#?}"
+    );
+    println!(
+        "CARD_POOL_ONLY_FILES checked={} bypasses=0",
+        FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL.len()
+    );
+}
diff --git a/crates/singlefs-checker-tier/src/gpu_card_configuration.rs b/crates/singlefs-checker-tier/src/gpu_card_configuration.rs
new file mode 100644
index 0000000..6eb5980
--- /dev/null
+++ b/crates/singlefs-checker-tier/src/gpu_card_configuration.rs
@@ -0,0 +1,150 @@
+//! 显卡配置怎么读：`GPU_CARDS_BY_PRIORITY` 点名的卡（在哪台机器、`nvidia-smi` 序号、显存额度）、这个进程在哪台机器、`键=值` 配置文件里一个键的值、
+//! 点名的卡里哪几张用不上。崩溃放量取卡（`crate::crash_amplification`，随 `verdict-store` 编）与 GPU 用例共用的卡池
+//! （`crate::gpu_unit_checks::gpu_test_start`，随 `gpu` 编）都照它挑卡，所以单放一份、只开其中一个特性也编得进；
+//! `crate::crash_amplification` 再 `pub use` 回去，它的调用方与单测照旧经它引。
+
+use std::path::Path;
+
+/// 配置里的一张卡在哪台机器上。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CardHost {
+    Local,
+    Peer,
+}
+
+impl CardHost {
+    #[must_use]
+    pub fn name(self) -> &'static str {
+        match self {
+            Self::Local => "local",
+            Self::Peer => "peer",
+        }
+    }
+
+    /// 这个进程在哪台机器上：环境变量 `SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE`，没设是本机。
+    ///
+    /// # Errors
+    /// 写的不是 `local` 也不是 `peer`。
+    pub fn of_this_process() -> Result<Self, String> {
+        match std::env::var("SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE") {
+            Err(_) => Ok(Self::Local),
+            Ok(text) => Self::parse(&text),
+        }
+    }
+
+    fn parse(text: &str) -> Result<Self, String> {
+        match text.trim() {
+            "local" => Ok(Self::Local),
+            "peer" => Ok(Self::Peer),
+            other => Err(format!("机器要写 local 或 peer，实际「{other}」")),
+        }
+    }
+}
+
+/// 配置点名的一张卡（`GPU_CARDS_BY_PRIORITY` 里的一项）：在哪台机器、`nvidia-smi` 的序号、给崩溃放量的显存额度。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CardByPriority {
+    pub host: CardHost,
+    pub index: u32,
+    pub memory_quota_mebibytes: u64,
+}
+
+/// 读 `GPU_CARDS_BY_PRIORITY` 的值：空白隔开的若干项，每项 `<local|peer>:<nvidia-smi 序号>:<显存额度>`，
+/// 额度是正整数加 `G` 或 `M`（1024 进制）；写在前面的优先级高，任务先装它、装满了才给后面的。
+///
+/// # Errors
+/// 一项不是三段、机器或序号或额度写法不对、同一张卡点名两次、一项都没有。
+pub fn parse_cards_by_priority(text: &str) -> Result<Vec<CardByPriority>, String> {
+    let mut cards: Vec<CardByPriority> = Vec::new();
+    for entry in text.split_whitespace() {
+        let fields: Vec<&str> = entry.split(':').collect();
+        let [host, index, quota] = fields.as_slice() else {
+            return Err(format!(
+                "显卡优先级的一项要写 <local|peer>:<序号>:<显存额度>，实际「{entry}」"
+            ));
+        };
+        let host = CardHost::parse(host)?;
+        let index: u32 = index
+            .parse()
+            .map_err(|_| format!("显卡序号不是数：「{entry}」"))?;
+        let (number, unit_mebibytes) = match quota.as_bytes().last() {
+            Some(b'G') => (&quota[..quota.len() - 1], 1024u64),
+            Some(b'M') => (&quota[..quota.len() - 1], 1u64),
+            _ => {
+                return Err(format!(
+                    "显存额度要带单位 G 或 M（1024 进制），实际「{entry}」"
+                ))
+            }
+        };
+        let number: u64 = number
+            .parse()
+            .ok()
+            .filter(|number| *number > 0)
+            .ok_or_else(|| format!("显存额度不是正整数：「{entry}」"))?;
+        if cards
+            .iter()
+            .any(|card| card.host == host && card.index == index)
+        {
+            return Err(format!("同一张卡点名了两次：「{entry}」"));
+        }
+        cards.push(CardByPriority {
+            host,
+            index,
+            memory_quota_mebibytes: number * unit_mebibytes,
+        });
+    }
+    if cards.is_empty() {
+        return Err("显卡优先级一项都没写".to_string());
+    }
+    Ok(cards)
+}
+
+/// `键=值` 配置文件里一个键的值（去掉两头的空白与一层引号）；文件不在、键没写是 `None`。
+#[must_use]
+pub fn configuration_value(path: &Path, key: &str) -> Option<String> {
+    let text = std::fs::read_to_string(path).ok()?;
+    text.lines().find_map(|line| {
+        let value = line.trim().strip_prefix(key)?.strip_prefix('=')?.trim();
+        let value = value
+            .strip_prefix('\'')
+            .and_then(|rest| rest.strip_suffix('\''))
+            .or_else(|| {
+                value
+                    .strip_prefix('"')
+                    .and_then(|rest| rest.strip_suffix('"'))
+            })
+            .unwrap_or(value);
+        Some(value.to_string())
+    })
+}
+
+/// 配置的显卡优先级：环境变量 `GPU_CARDS_BY_PRIORITY` 设了就照它（驱动脚本按清没清场从显卡配置里挑一份送过来，第二台的树副本也靠它），
+/// 没设读显卡配置文件（`SINGLEFS_GPU_CARDS_CONFIG` 指的那一份，模板是仓根的 `gpu-cards.env.example`）里的同名键；
+/// 都没有是 `None`（本机能用的卡按空闲显存从大到小用，不设额度）。
+///
+/// # Errors
+/// 写了而写法不对（[`parse_cards_by_priority`]）。
+pub fn cards_by_priority_from_configuration_or_environment(
+    configuration_file: Option<&Path>,
+) -> Result<Option<Vec<CardByPriority>>, String> {
+    let text = match std::env::var("GPU_CARDS_BY_PRIORITY") {
+        Ok(text) => Some(text),
+        Err(_) => {
+            configuration_file.and_then(|path| configuration_value(path, "GPU_CARDS_BY_PRIORITY"))
+        }
+    };
+    match text {
+        Some(text) if !text.trim().is_empty() => parse_cards_by_priority(&text).map(Some),
+        _ => Ok(None),
+    }
+}
+
+/// 配置点名的卡里哪几张（序号）不在用得上的卡里，按配置的次序。
+#[must_use]
+pub fn configured_cards_not_usable(configured: &[(u32, u64)], usable: &[u32]) -> Vec<u32> {
+    configured
+        .iter()
+        .map(|(index, _)| *index)
+        .filter(|index| !usable.contains(index))
+        .collect()
+}
````

### 10s mutations-append.tsv

````text
第10s批 显存账 设了卡的额度变量却取额度与空闲显存里大的那个	crates/singlefs-harness/src/gpu_memory_ledger.rs	        Some(quota) => quota.min(	        Some(quota) => quota.max(	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_card_quota_from_the_environment_caps_each_card_ledger_and_without_it_the_capacity_is_unchanged	a_card_quota_from_the_environment_caps_each_card_ledger_and_without_it_the_capacity_is_unchanged
第10s批 显存账 只算自己这个进程借的、看不见别的进程借的	crates/singlefs-harness/src/gpu_memory_ledger.rs	            .map(|lease| lease.mebibytes)	            .filter(|lease| lease.process == viewer)\n            .map(|lease| lease.mebibytes)	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_second_process_whose_booking_does_not_fit_waits_until_the_first_process_returns_its_booking	a_second_process_whose_booking_does_not_fit_waits_until_the_first_process_returns_its_booking
第10s批 显存账 等满最长的等待也不放弃	crates/singlefs-harness/src/gpu_memory_ledger.rs	                    if waited >= self.settings.longest_wait {	                    if false && waited >= self.settings.longest_wait {	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_booking_that_cannot_get_room_before_the_longest_wait_gives_up	a_booking_that_cannot_get_room_before_the_longest_wait_gives_up
第10s批 显存账 记账进程不在了它借的那一份照算	crates/singlefs-harness/src/gpu_memory_ledger.rs	        self.leases.retain(|lease| is_alive(lease.process));	        self.leases.retain(|_lease| true);	-p singlefs-harness --lib -- gpu_memory_ledger::tests::a_booking_left_by_a_process_that_is_no_longer_alive_is_not_counted	a_booking_left_by_a_process_that_is_no_longer_alive_is_not_counted
第10s批 显存账 判活不比启动时刻（进程号被重用也当原来那个还在）	crates/singlefs-harness/src/gpu_memory_ledger.rs	            .is_none_or(|start_time| start_time == process.start_time_in_clock_ticks),	            .is_none_or(|_start_time| true),	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_process_running_the_test_is_alive_and_its_identifier_with_another_start_time_is_not	the_process_running_the_test_is_alive_and_its_identifier_with_another_start_time_is_not
第10s批 显存账 默认工作根少了 work 那一层（与 lib_work_root.py 的 DEFAULT_WORK_ROOT 对不上）	crates/singlefs-harness/src/gpu_memory_ledger.rs	pub const DEFAULT_WORK_ROOT_UNDER_THE_HOME_DIRECTORY: &str = ".local/share/singlefs/work";	pub const DEFAULT_WORK_ROOT_UNDER_THE_HOME_DIRECTORY: &str = ".local/share/singlefs";	-p singlefs-harness --lib -- gpu_memory_ledger::tests::the_ledger_directory_is_under_the_work_root_variable_or_the_default_work_root_under_home	the_ledger_directory_is_under_the_work_root_variable_or_the_default_work_root_under_home
第10s批 扫描内核调用点：judge_accounting 的 c_quarantine_exempted 加回一个调用点（展开 392 383 还在预算里，钉调用点数的单测红）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        c_quarantine_exempted(newest_root.base, reference_limit, fsid_low);	        c_quarantine_exempted(newest_root.base, reference_limit, fsid_low);\n        c_quarantine_exempted(newest_root.base, reference_limit, fsid_low);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_dispatch::tests::each_judge_merged_into_one_call_site_keeps_that_single_call_site_in_the_scan_kernel	each_judge_merged_into_one_call_site_keeps_that_single_call_site_in_the_scan_kernel
第10s批 内核展开预算抬了一个表达式（三个内核钉的余量都对不上）	crates/singlefs-checker-tier/src/crash_judge_dispatch.rs	pub const LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL: u64 = 400_000;	pub const LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL: u64 = 400_001;	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_dispatch::tests::each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget	each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget
第10s批 源码扫描：改走卡池的文件又经入口拿第一张能用的卡（不记账）	crates/singlefs-checker-tier/tests/gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs	    let Some(cards) = gpu_card_pool_or_report_not_run(TEST_NAME) else {	    let Some(cards) = first_usable_card_or_report_not_run(TEST_NAME) else {	-p singlefs-checker-tier --test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start -- every_test_file_moved_onto_the_card_pool_takes_its_cards_only_from_the_pool	every_test_file_moved_onto_the_card_pool_takes_its_cards_only_from_the_pool
````

### 10s mutations-replacements.tsv

````text
崩溃放量 显卡优先级 额度的 M 当成 G	crates/singlefs-checker-tier/src/gpu_card_configuration.rs	            Some(b'M') => (&quota[..quota.len() - 1], 1u64),	            Some(b'M') => (&quota[..quota.len() - 1], 1024u64),	-p singlefs-checker-tier --features verdict-store --lib -- crash_amplification::tests::cards_by_priority_are_read_in_the_order_written_with_their_quotas	cards_by_priority_are_read_in_the_order_written_with_their_quotas
崩溃放量 显卡优先级 同一张卡点名两次也收	crates/singlefs-checker-tier/src/gpu_card_configuration.rs	            .any(|card| card.host == host && card.index == index)	            .any(|card| card.host == host && card.index == index && card.memory_quota_mebibytes == 0)	-p singlefs-checker-tier --features verdict-store --lib -- crash_amplification::tests::cards_by_priority_are_read_in_the_order_written_with_their_quotas	cards_by_priority_are_read_in_the_order_written_with_their_quotas
崩溃放量 显卡配置 双引号的值不去引号	crates/singlefs-checker-tier/src/gpu_card_configuration.rs	                    .and_then(|rest| rest.strip_suffix('"'))	                    .and_then(|rest| rest.strip_suffix('#'))	-p singlefs-checker-tier --features verdict-store --lib -- crash_amplification::tests::the_card_priority_is_read_from_the_configuration_file_with_one_layer_of_quotes_removed	the_card_priority_is_read_from_the_configuration_file_with_one_layer_of_quotes_removed
崩溃放量 点名的卡少一张也照跑	crates/singlefs-checker-tier/src/gpu_card_configuration.rs	        .filter(|index| !usable.contains(index))	        .filter(|index| !usable.contains(index) && false)	-p singlefs-checker-tier --features verdict-store --lib -- crash_amplification::tests::configured_cards_that_are_not_usable_are_named_in_the_configured_order	configured_cards_that_are_not_usable_are_named_in_the_configured_order
test-script-align-code-r2 C604 源码扫描：tests/ 里的 GPU 对拍绕开 gpu_test_start 直接起卡	crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs	    let lease = cards.lease_card_or_report_not_run(	    let _bypass = singlefs_checker_tier::gpu_unit_checks::usable_gpu_cards();\n    let lease = cards.lease_card_or_report_not_run(	-p singlefs-checker-tier --test gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start -- every_gpu_test_takes_its_cards_through_gpu_test_start	every_gpu_test_takes_its_cards_through_gpu_test_start
````

## #9 钉值重取

````diff
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
@@ -842,14 +842,14 @@
 /// 计划哈希也是 `0370ad26…`、打的行只比主工作区少 `I-9.28=0/0/46` 那两项）。
 const GOLDEN_PRINTED_LINE_COUNT: usize = 58;
 const GOLDEN_PRINTED_LINES_SHA256: &str =
-    "4fc9cde737d618c21d064f5cd8072b63130103b8533509c2cec93c09f5216ae8";
+    "4b8b1e84e16b6a4ecd22a8bc28a15c75a5dbef3b99a1cfddc9e5b98703ed04b1";
 /// 同一趟留下的进度文件：名字（带计划哈希）与整份内容的 SHA-256（47 行：文件头 + 46 行片行）。名字取自上面那一趟日志的 `LAYER0_RESUME` 行；
 /// 打的行与内容的 SHA-256 取自 2026-10-05 主工作区（形式化线第 34 批派出时那一版，含删除线第 22 批）：与根环线在「10g + 32-1」那棵树上重取的 58 行逐行比，只差 GOLDEN_TALLY 两行多出 I-9.17–I-9.24、I-9.29 九条新不变量的计数（删除线第 22 批把已实现的不变量从 74 条加到 83 条），已有各条计数不变；那一次比上一版只差 `LAYER0_RESUME` 那一行的计划哈希（T_time 改毫秒带来）：
 /// 打的行是那一趟断言 `left` 的 `50615e47…`。2026-10-06 删除线第 24 批合入之后（新九条各写专门的不适用理由、I-9.17 / I-9.23 改读孩子的方式）在主工作区重取：打的行取那一趟断言 `left` 的 `4fc9cde7…`；
-/// 内容是在只换了打的行钉值的副本里再跑一次、「不分片时的进度文件与钉值不同」那条断言 `left` 的第二项（2026-10-06 那一次是 `11977571…`，进度文件名不变）。池级 checker 的判定集合再变，两个值跟着重取。
+/// 内容是在只换了打的行钉值的副本里再跑一次、「不分片时的进度文件与钉值不同」那条断言 `left` 的第二项（2026-10-06 那一次是 `11977571…`，进度文件名不变）。2026-10-06 根环线第 10o 批合入（加 I-3.14，已实现的不变量 83 → 84 条）之后在主工作区（含删除线第 30 批）重取：打的行取断言 `left` 的 `4b8b1e84…`，把两趟 `GOLDEN_TALLY` 各一项 ` I-3.14=46/0/0` 删掉之后整段 SHA-256 正好是上一版 `4fc9cde7…`，别的字节不差；内容在只换了打的行钉值的副本里再跑一次取断言 `left` 第二项 `1d92a7ad…`，进度文件名不变。池级 checker 的判定集合再变，两个值跟着重取。
 const GOLDEN_PROGRESS_FILE_NAME: &str = "layer0-progress-golden_new_pool_file_creation_stream-fingerprint0-4987883029b866a331bd86aa1db770710eccbf4db5206676859243f01abd914c.txt";
 const GOLDEN_PROGRESS_FILE_SHA256: &str =
-    "119775719aa569efddcf00022a0ac5f1d70a2102af181e379a70aa6e4afd6cb5";
+    "1d92a7ad8c48c01f43dfe9d675d6256e110545b588d2bc5f6ad8601e63ed8cab";
 
 /// 一行打印里会随机器与时刻变的两样换掉：进度目录、`elapsed_seconds=` 的值。
 fn masked_printed_line(line: &str, directory: &Path) -> String {
````
