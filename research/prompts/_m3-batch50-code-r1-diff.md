# 删除线 W3 + 第 50 批（M3、叶死亡退役树）：代码轮第一轮改动全文（2026-10-09）

合起来的那一份补丁原样（`~/.local/share/singlefs/work/m3-batch50/m3b/patch/`，打在根环线第二十四、二十五块合完之后的主工作区上）：`crates.patch`，变异表追加行、整行替换行、删掉的行原样。

## crates.patch

`````diff
diff --git a/crates/singlefs-checker-tier/src/bad_disk_input.rs b/crates/singlefs-checker-tier/src/bad_disk_input.rs
index 7585ef0..c2b2359 100644
--- a/crates/singlefs-checker-tier/src/bad_disk_input.rs
+++ b/crates/singlefs-checker-tier/src/bad_disk_input.rs
@@ -2630,7 +2630,8 @@ fn relabel_the_inode_watermark_row(
         return None;
     }
     let layout = IndexNodeEntryLayout::of(&chain.tree_root_node);
-    let watermark_row = (0..layout.entry_count).find(|index| {
+    // 台阶行（D5（快照 / 空间记账机制） 已定项 2 射程）与每次发布写的那一行同标签、代段更早、排在前面：改的是最后那一条（现行值那一行）。
+    let watermark_row = (0..layout.entry_count).rev().find(|index| {
         read_u16_at(
             &chain.tree_root_node,
             layout.offset_of_entry(*index) + ACCOUNTING_ENTRY_STATISTIC_OFFSET,
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index f747c83..92be941 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -2325,6 +2325,7 @@ fn first_file_is_a_live_record(version: &TransactionOutput) -> bool {
 /// 一次发布写没写只有续做的删除步才写的那几种结构：文件的 extent 树、inode 树与 `deleted_inodes` 树（数据单元也算，挂载里没有别的发布写它）。
 /// 写行、暖机与抬 F 的空发布只重写分配记录树、记账树、中央映射树、树表、实例表与固定结构（2026-10-03 在副本上量的：崩在意图之后重挂，
 /// 写行与两次暖机写的是这几种，续做那一步写了 extent 树与 inode 树根）。
+/// 退役树（第 50 批）只在身份消失的那次最后一批写，挂载里只可能是续做那一步写的，同样认。
 /// 五种里 `DataUnit` 与 `DeletedInodesTreeNode` 在那次探针上空转（续做那一步没写它们）、仍留着（代码轮第二轮判决 V6）：这一判认的是
 /// 「写行、暖机与抬 F 的空发布都不写的结构」，不是「今天的续做那一步写了的结构」，两种都属于前者——那几种发布不碰文件数据，也不碰
 /// `deleted_inodes`（队列只由删除类发布改），所以认它们不会把写行或暖机认成续做；去掉它们，划段就随续做那一步今天的写法变：
@@ -2338,7 +2339,8 @@ fn writes_a_structure_only_the_continuation_changes(writes: &WritesByStructureKi
             | WrittenStructureKind::ExtentTreeNode
             | WrittenStructureKind::InodeTreeLeafContainer
             | WrittenStructureKind::InodeTreeRoot
-            | WrittenStructureKind::DeletedInodesTreeNode => true,
+            | WrittenStructureKind::DeletedInodesTreeNode
+            | WrittenStructureKind::RetirementTreeNode => true,
             WrittenStructureKind::AllocationRecordTreeNode
             | WrittenStructureKind::AccountingTreeNode
             | WrittenStructureKind::TombstoneTreeNode
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index 43d18cd..56f0ca1 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -1600,7 +1600,9 @@ fn w_judge_packed_container(s: Slice, expected_type: u32, fsid_low: vec2<u32>) -
     } else if (record_type == PACKED_TYPE_TOMBSTONE) {
         registered_width = 40u;
     } else if (record_type == PACKED_TYPE_LINEAGE) {
-        registered_width = 16u;
+        registered_width = 24u;
+    } else if (record_type == PACKED_TYPE_RETIREMENT) {
+        registered_width = 34u;
     }
     let legal = record_type == expected_type && registered_width == width && count * width <= declared && declared <= DATA_UNIT_BYTES - 136u;
     w_judge(INVARIANT_I_1_7, legal);
@@ -1807,8 +1809,8 @@ fn w_judge_tree_table_entries_ordering(n: Node) {
                 }
             }
         }
-        // ④：树表非空时进树表的十一种（发号次序 1,2,3,4,6,7,8,9,10,11,12）各至少一条，与 ③ 合起来各恰好一条
-        // （判决 m3-formal-code-b26-r1 第二节 J1 C1：core 重建一版时十一种各要一条）
+        // ④：树表非空时进树表的十二种（发号次序 1,2,3,4,6,7,8,9,10,11,12,13）各至少一条，与 ③ 合起来各恰好一条
+        // （判决 m3-formal-code-b26-r1 第二节 J1 C1：core 重建一版时十二种各要一条）
         for (var position = 0u; position < TREE_KINDS_OF_THE_TREE_TABLE; position = position + 1u) {
             let required_kind = tree_kind_in_the_issuing_order(position);
             var required_kind_is_present = false;
@@ -1823,7 +1825,7 @@ fn w_judge_tree_table_entries_ordering(n: Node) {
             }
         }
     }
-    // ②：按发号次序 1,2,3,4,6,7,8,9,10,11,12（walk.rs 的 TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER）
+    // ②：按发号次序 1,2,3,4,6,7,8,9,10,11,12,13（walk.rs 的 TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER）
     var previous_tree = vec2<u32>(0u, 0u);
     var have_previous = false;
     for (var k = 0u; k < TREE_KINDS_OF_THE_TREE_TABLE; k = k + 1u) {
@@ -1845,7 +1847,7 @@ fn w_judge_tree_table_entries_ordering(n: Node) {
     w_judge(INVARIANT_I_9_16, true);
 }
 
-// 树表条目的头 ID（walk.rs judge_tree_table_entry_head_identifiers，I-9.27）：inode 树等于自己的树 ID；extent 树、deleted_inodes、墓碑树
+// 树表条目的头 ID（walk.rs judge_tree_table_entry_head_identifiers，I-9.27）：inode 树等于自己的树 ID；extent 树、deleted_inodes、墓碑树、退役树
 // 等于同一张表里某一条 inode 树条目的树 ID（一条都没有就红）；3、4、6、7、8、11、12 等于 0；别的种类不判。条目宽不到头 ID 那 8 字节时整张表不判
 fn w_judge_tree_table_entry_head_identifiers(n: Node) {
     if (n.entry_width < TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8u) {
@@ -1858,7 +1860,8 @@ fn w_judge_tree_table_entry_head_identifiers(n: Node) {
         let head = rd64(n.base, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
         if (kind == TREE_KIND_INODE) {
             w_judge(INVARIANT_I_9_27, eq64(head, tree));
-        } else if (kind == TREE_KIND_EXTENT || kind == TREE_KIND_DELETED_INODES || kind == TREE_KIND_TOMBSTONE) {
+        } else if (kind == TREE_KIND_EXTENT || kind == TREE_KIND_DELETED_INODES || kind == TREE_KIND_TOMBSTONE
+            || kind == TREE_KIND_RETIREMENT) {
             var names_an_inode_tree_of_this_table = false;
             for (var other_index = 0u; other_index < n.entry_count; other_index = other_index + 1u) {
                 let other = entry_offset(n, other_index);
@@ -2907,15 +2910,14 @@ fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: v
             w_judge(INVARIANT_I_9_12, lt64(previous_largest, separator));
         }
     }
-    // I-9.4 第二、三句：沿叶序相邻两片
+    // I-9.4 第二句：沿叶序相邻两片容器号严格递增（第三句「左容器的最大 key < 右容器号」随 N1 删掉：用户 2026-10-08 定案 X4，
+    // 左从右借之后右片保留身份，它的容器号可以不大于左片的最大 key；walk.rs judge_container_numbers_along_the_leaf_order 同改）
     for (var i = leaves_before + 1u; i < w_leaf_count; i = i + 1u) {
         let left = C_LEAVES + (i - 1u) * C_LEAF_WORDS;
         let right = C_LEAVES + i * C_LEAF_WORDS;
         let left_container = vec2<u32>(sw(left + 3u), sw(left + 4u));
-        let left_largest = vec2<u32>(sw(left + 7u), sw(left + 8u));
         let right_container = vec2<u32>(sw(right + 3u), sw(right + 4u));
         w_judge(INVARIANT_I_9_4, lt64(left_container, right_container));
-        w_judge(INVARIANT_I_9_4, lt64(left_largest, right_container));
     }
 }
 
@@ -2964,7 +2966,7 @@ fn w_walk_tree_table_entry(tt: Node, e: u32, fsid_low: vec2<u32>) {
     else if (kind == TREE_KIND_ACCOUNTING) { schema = 0u; registered = true; field_table_width = 34u; }
     else if (kind == 5u) { schema = 1u; }
     else if (kind == TREE_KIND_DELETED_INODES) { schema = 2u; registered = true; field_table_width = DELETED_INODES_ENTRY_BYTES; }
-    else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE) { schema = 2u; registered = true; field_table_width = CONTAINER_TREE_ENTRY_BYTES; }
+    else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) { schema = 2u; registered = true; field_table_width = CONTAINER_TREE_ENTRY_BYTES; }
     if (schema == NONE) {
         return;
     }
@@ -2977,10 +2979,20 @@ fn w_walk_tree_table_entry(tt: Node, e: u32, fsid_low: vec2<u32>) {
     }
     if (kind == TREE_KIND_ACCOUNTING) {
         let walked = w_walk_code_two_tree(pointer, tree, 0u, fsid_low);
-        // I-3.13：走全了的记账树里每一行的代段（key 偏移 14）等于这一版的 txg
+        // I-3.13：走全了的记账树里每一行的代段（key 偏移 14）等于这一版的 txg；inode 号水位那一统计量（标签 12）、不带设备维、
+        // 代段早于这一版 txg 的行是台阶行，跳过（walk.rs walk_tree_table_entry 同改，D5（快照 / 空间记账机制） 已定项 2 射程）
         if (walked.kind == 0u && walked.is_complete) {
             for (var i = 0u; i < walked.entries; i = i + 1u) {
-                w_judge(INVARIANT_I_3_13, eq64(rd64(sw(R_ACCOUNTING + i * 3u), sw(R_ACCOUNTING + i * 3u + 1u) + 14u), w_version_txg));
+                let row_base = sw(R_ACCOUNTING + i * 3u);
+                let row_offset = sw(R_ACCOUNTING + i * 3u + 1u);
+                let row_generation = rd64(row_base, row_offset + 14u);
+                let is_an_inode_number_watermark_step = rd16(row_base, row_offset) == 12u
+                    && rd32(row_base, row_offset + 10u) == NONE
+                    && lt64(row_generation, w_version_txg);
+                if (is_an_inode_number_watermark_step) {
+                    continue;
+                }
+                w_judge(INVARIANT_I_3_13, eq64(row_generation, w_version_txg));
             }
         }
         // 记账行只取最新根那一遍的（check_pool_image 在走最新根之后就把 accounting 拷走了，候选根的账不覆盖它）
@@ -3040,11 +3052,13 @@ fn w_walk_tree_table_entry(tt: Node, e: u32, fsid_low: vec2<u32>) {
     if (kind == TREE_KIND_INODE) {
         w_walk_inode_root(read.n, tree, read.unit_device, read.unit_slot, fsid_low);
     }
-    // 墓碑树、谱系树：码 2 根之下是码 3 容器（类型段墓碑 {0, 1}、谱系 {0, 5}）；走到每片容器、判它的头与容器格式
-    if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE) {
+    // 墓碑树、谱系树、退役树：码 2 根之下是码 3 容器（类型段墓碑 {0, 1}、谱系 {0, 5}、退役 {0, 3}）；走到每片容器、判它的头与容器格式
+    if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) {
         var container_record_type = PACKED_TYPE_TOMBSTONE;
         if (kind == TREE_KIND_LINEAGE) {
             container_record_type = PACKED_TYPE_LINEAGE;
+        } else if (kind == TREE_KIND_RETIREMENT) {
+            container_record_type = PACKED_TYPE_RETIREMENT;
         }
         w_walk_container_tree_root(read.n, tree, read.unit_device, read.unit_slot, container_record_type, fsid_low);
     }
@@ -3191,6 +3205,8 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
     var identity_invariant = INVARIANT_I_9_23;
     if (record_type == PACKED_TYPE_LINEAGE) {
         identity_invariant = INVARIANT_I_9_17;
+    } else if (record_type == PACKED_TYPE_RETIREMENT) {
+        identity_invariant = INVARIANT_I_9_31;
     }
     let parent_valid = w_parent_valid;
     let parent_device = w_parent_device;
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
index 1117fbd..f6a7c48 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
@@ -183,11 +183,29 @@ const INVARIANT_I_9_29: u32 = 82u;
 // 第 84 条：根环线第 10j 批新立的 I-3.14（实例表每片记录形态合法），走读内核在 w_walk_instance_table_chain 走到的每一片上判四格（第 10k 批）
 const INVARIANT_I_3_14: u32 = 83u;
 
-// 墓碑树、谱系树叶容器的打包记录类型（walk.rs 的 PACKED_TYPE_TOMBSTONE / PACKED_TYPE_LINEAGE；登记的记录宽墓碑 40、谱系 16）
+// 第 85 条：第 49 批新立的 I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上），扫描内核在最新根上判（walk.rs judge_inode_number_watermark_steps_of_the_newest_root）
+const INVARIANT_I_9_30: u32 = 84u;
+
+// 第 86–90 条：第 50 批退役树那五条（walk.rs 的 RETIREMENT_*）。走读内核在退役树的每条内部条目上判 I-9.31；扫描内核在墓碑与谱系那一遍的每一版上判
+// I-9.32、I-9.33、I-9.34，扫完各版再跨相邻两版判 I-9.35（用户 2026-09-29「配了 GPU 就一定要 GPU 判，不退回 CPU」，不记判不了）
+const INVARIANT_I_9_31: u32 = 85u;
+
+const INVARIANT_I_9_32: u32 = 86u;
+
+const INVARIANT_I_9_33: u32 = 87u;
+
+const INVARIANT_I_9_34: u32 = 88u;
+
+const INVARIANT_I_9_35: u32 = 89u;
+
+// 墓碑树、谱系树叶容器的打包记录类型（walk.rs 的 PACKED_TYPE_TOMBSTONE / PACKED_TYPE_LINEAGE；登记的记录宽墓碑 40、谱系 24）
 const PACKED_TYPE_TOMBSTONE: u32 = 1u;
 
 const PACKED_TYPE_LINEAGE: u32 = 5u;
 
+// 退役树叶容器的打包记录类型（walk.rs 的 PACKED_TYPE_RETIREMENT；登记的记录宽 34）
+const PACKED_TYPE_RETIREMENT: u32 = 3u;
+
 // deleted_inodes 根的条目：key = inode 号 8 字节、无 value（walk.rs 的 DELETED_INODES_ENTRY_BYTES）
 const DELETED_INODES_ENTRY_BYTES: u32 = 8u;
 
@@ -399,11 +417,12 @@ const C_MAX_CANDIDATES: u32 = 48u;
 
 const C_CANDIDATES: u32 = 166368u;            // 候选根在根表里的下标
 
-const C_CANDIDATE_WORDS: u32 = 50u;          // 0 走全了、1 有树表落点、2 盘、3 槽 lo、4 槽 hi、5 树数、6.. 每树 4 字（树 lo/hi、诞生 lo/hi）：6 + 4 × C_MAX_TREES_PER_TABLE
+const C_CANDIDATE_WORDS: u32 = 54u;          // 0 走全了、1 有树表落点、2 盘、3 槽 lo、4 槽 hi、5 树数、6.. 每树 4 字（树 lo/hi、诞生 lo/hi）：6 + 4 × C_MAX_TREES_PER_TABLE
 
-// 一张树表最多记几棵不同的树：进树表的种类有 11 个（recovery.rs 与 walk.rs 的 TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER，
-// 第一个文件版本那次就写满 11 条）；合法镜像上一张树表每种至多一条，更多的只出现在坏镜像上，第 12 棵起判不了、交 CPU
-const C_MAX_TREES_PER_TABLE: u32 = 11u;
+// 一张树表最多记几棵不同的树：进树表的种类有 12 个（recovery.rs 与 walk.rs 的 TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER，
+// 第一个文件版本那次就写满 12 条，退役树是第 50 批加的）；合法镜像上一张树表每种至多一条，更多的只出现在坏镜像上，第 13 棵起判不了、交 CPU。
+// 从 11 加到 12 时 C_CANDIDATE_INFO 那 48 行每行多 4 字，其后的草稿区偏移各后移 48 × 4 = 192 字
+const C_MAX_TREES_PER_TABLE: u32 = 12u;
 
 // I-9.28 在 I-3.9 与 I-9.14 判完之后借这几行，根表里第 k 条根记进第 k 行（只用 1..5 与每树那几字）：根表最多 MAX_ROOTS 条，
 // 不多于 C_MAX_CANDIDATES 行（judge_tree_identifier_birth_txg_across_every_readable_root 开头判，越过就判不了）
@@ -413,61 +432,61 @@ const C_ALLOC_RECORD_WORDS: u32 = 8u;        // 盘、槽 lo/hi、跨度、代 l
 
 const C_MAX_ALLOC_RECORDS: u32 = 2048u;
 
-const C_ALLOC_RECORDS: u32 = 168816u;
+const C_ALLOC_RECORDS: u32 = 169008u;
 
 const C_R_MAX_SEEN: u32 = 512u;
 
-const C_R_SEEN: u32 = 185200u;                // 盘、槽 lo/hi
+const C_R_SEEN: u32 = 185392u;                // 盘、槽 lo/hi
 
 const C_R_PENDING_WORDS: u32 = 10u;
 
 const C_R_MAX_PENDING: u32 = 512u;
 
-const C_R_PENDING: u32 = 186736u;
+const C_R_PENDING: u32 = 186928u;
 
 const C_MAX_SEEN_NODES: u32 = 256u;
 
-const C_SEEN_NODES: u32 = 191856u;           // 盘、槽 lo/hi、校验和
+const C_SEEN_NODES: u32 = 192048u;           // 盘、槽 lo/hi、校验和
 
 const C_MAX_EXAMINED: u32 = 2048u;
 
-const C_EXAMINED: u32 = 192880u;             // 盘、槽 lo/hi、代 lo/hi
+const C_EXAMINED: u32 = 193072u;             // 盘、槽 lo/hi、代 lo/hi
 
 const C_MAX_SCANNED: u32 = 2048u;
 
-const C_SCANNED: u32 = 203120u;              // 落点下标
+const C_SCANNED: u32 = 203312u;              // 落点下标
 
 const C_R_MAX_ROWS: u32 = 256u;
 
-const C_R_ROWS: u32 = 205168u;               // 实例、txg lo/hi
+const C_R_ROWS: u32 = 205360u;               // 实例、txg lo/hi
 
 const C_VALID_BEFORE_WORDS: u32 = 6u;        // 根下标、有指针、inode 基、inode 偏移、extent 基、extent 偏移
 
-const C_VALID_BEFORE: u32 = 205936u;
+const C_VALID_BEFORE: u32 = 206128u;
 
-const C_NON_EMPTY: u32 = 206224u;            // txg lo/hi、实例、valid_before 下标
+const C_NON_EMPTY: u32 = 206416u;            // txg lo/hi、实例、valid_before 下标
 
-const C_UNDETERMINABLE: u32 = 206416u;       // txg lo/hi、实例
+const C_UNDETERMINABLE: u32 = 206608u;       // txg lo/hi、实例
 
-const C_COUNTED: u32 = 206560u;              // txg lo/hi
+const C_COUNTED: u32 = 206752u;              // txg lo/hi
 
-const C_COUNTED_STATES: u32 = 206656u;       // valid_before 下标
+const C_COUNTED_STATES: u32 = 206848u;       // valid_before 下标
 
-const C_PICKED: u32 = 206704u;               // 选过的标记（非空 48 + 判不了 48）
+const C_PICKED: u32 = 206896u;               // 选过的标记（非空 48 + 判不了 48）
 
 const C_MAX_RELEASED_ABOVE: u32 = 2048u;
 
-const C_RELEASED_ABOVE: u32 = 206800u;       // 盘、槽 lo/hi、跨度
+const C_RELEASED_ABOVE: u32 = 206992u;       // 盘、槽 lo/hi、跨度
 
 const C_MAX_DEFERRED: u32 = 4096u;
 
-const C_DEFERRED: u32 = 214992u;             // 盘、槽 lo/hi
+const C_DEFERRED: u32 = 215184u;             // 盘、槽 lo/hi
 
 const C_MAX_GROUPS: u32 = 2048u;
 
-const C_GROUPS: u32 = 227280u;               // 槽 lo/hi、跨度、盘位掩码
+const C_GROUPS: u32 = 227472u;               // 槽 lo/hi、跨度、盘位掩码
 
-const C_SCRATCH_END_SCAN: u32 = 235472u;
+const C_SCRATCH_END_SCAN: u32 = 235664u;
 
 var<private> w_context: u32;
 
@@ -698,7 +717,7 @@ var<private> w_accounting_recording: bool;
 // ───────────────────────── 走读内核交给扫描内核的东西（草稿区里的交接段）─────────────────────────
 // 两个内核各自编（驱动编一个 4900 行的内核要 41 分钟、13 GiB；拆开各编各的、并行）；草稿区在同一次派活里两趟共用，
 // 集合都留在草稿区里，只把私有变量里的计数与几何写进这一段。
-const H_BASE: u32 = 235472u;
+const H_BASE: u32 = 235664u;
 const H_VALID: u32 = 0u;                 // 1 = 走读那一批判完、判得了；0 = 扫描内核什么都不做
 const H_ROW_COUNT: u32 = 1u;
 const H_REFERENCE_COUNT: u32 = 2u;
@@ -736,21 +755,21 @@ const H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE: u32 = 30u;
 // 没有是 NONE：扫描内核认下一次挂载先施加的那一版（I-3.10、I-5.4）读它。占原来空着的第 31 个字，交接段不变长
 const H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED: u32 = 31u;
 const H_MAX_BELOW_FLOOR_SLOTS: u32 = 4096u;
-const H_BELOW_FLOOR_SLOTS: u32 = 235504u;   // 盘、槽 lo/hi
-const H_END: u32 = 247792u;
+const H_BELOW_FLOOR_SLOTS: u32 = 235696u;   // 盘、槽 lo/hi
+const H_END: u32 = 247984u;
 
 // ───────────────────────── 交接段之后：走读内核写的两张表（扫描内核只读重放那一张）─────────────────────────
 // 主走读（上下文 0）每个走过的单元一行：有没有记下诞生代号、诞生代号 lo/hi（walk.rs 的 birth_txg_of_walked_units，与 C_VISITED0 同下标）。
 // 别的版本再引用它时按这张表判 I-4.9（非最新候选根引用的单元不晚于它）。
 const C_VISITED_BIRTH_WORDS: u32 = 3u;
 
-const C_VISITED_BIRTHS0: u32 = 247792u;
+const C_VISITED_BIRTHS0: u32 = 247984u;
 
 // I-4.1 重放读的那一份环（walk.rs 的 records_the_replay_reads）：版本、计数器 lo/hi、实例，按 (实例, 计数器) 升序。
 // 走读内核在 I-4.1 那一阶段填；扫描内核照交接段的 H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED 读其中一行、不写
 const C_REPLAY_RECORD_WORDS: u32 = 4u;
 
-const C_REPLAY_RECORDS: u32 = 253936u;
+const C_REPLAY_RECORDS: u32 = 254128u;
 
 fn replay_record_at(index: u32) -> CheckerRecord {
     return checker_record_of_row(C_REPLAY_RECORDS + index * C_REPLAY_RECORD_WORDS);
@@ -764,7 +783,7 @@ const C_APPEARANCE_PLACEMENT_WORDS: u32 = 10u;
 
 const C_APPEARANCE_MAX_PLACEMENTS: u32 = 2048u;
 
-const C_APPEARANCE_PLACEMENTS: u32 = 255984u;
+const C_APPEARANCE_PLACEMENTS: u32 = 256176u;
 
 // ② 版本表（walk.rs 的 versions_walked_for_the_appearance_rule）：走完的每一版一行——实例、txg lo/hi、走读一条失败都没记（1 / 0）、
 //    这一版引用的落点在 ③ 里的起点与条数、树表里用户可见树的根几棵，之后每棵 5 个字（种类、树 ID lo/hi、根指针出生 txg lo/hi）。
@@ -778,12 +797,12 @@ const C_APPEARANCE_VERSION_ROOTS_OFFSET: u32 = 7u;
 
 const C_APPEARANCE_MAX_VERSIONS: u32 = 64u;
 
-const C_APPEARANCE_VERSIONS: u32 = 276464u;
+const C_APPEARANCE_VERSIONS: u32 = 276656u;
 
 // ③ 每一版引用的落点（walk.rs 的 placements_referenced_in_this_version 在这一版走完那一刻的样子）：每个落点一个字，是它在 ① 里的下标。
 const C_APPEARANCE_MAX_VERSION_PLACEMENTS: u32 = 8192u;
 
-const C_APPEARANCE_VERSION_PLACEMENTS: u32 = 279536u;
+const C_APPEARANCE_VERSION_PLACEMENTS: u32 = 279728u;
 
 // ④ 走过的树表（walk.rs 的 tree_table_entries_of_a_walked_tree_table 里 I-3.5 读的那一样）：第一个读得出它的那一版记下树表单元的盘、槽 lo/hi
 //    与它里面用户可见树的根几棵、每棵 5 个字（同 ②）；别的版本共享这片树表时照它取这一版的根。
@@ -793,7 +812,7 @@ const C_APPEARANCE_WALKED_TREE_TABLE_ROOTS_OFFSET: u32 = 4u;
 
 const C_APPEARANCE_MAX_WALKED_TREE_TABLES: u32 = 64u;
 
-const C_APPEARANCE_WALKED_TREE_TABLES: u32 = 287728u;
+const C_APPEARANCE_WALKED_TREE_TABLES: u32 = 287920u;
 
 // ───────────────────────── 走读内核那几张表之后：扫描内核的删除态扫描（walk.rs judge_deletion_state_of_the_versions）与墓碑、谱系那一遍
 // （walk.rs judge_tombstones_and_the_lineage_of_the_versions）用的表 ─────────────────────────
@@ -803,14 +822,14 @@ const D_INODE_RECORD_WORDS: u32 = 7u;
 
 const D_MAX_INODE_RECORDS: u32 = 1024u;
 
-const D_INODE_RECORDS: u32 = 290544u;
+const D_INODE_RECORDS: u32 = 290736u;
 
 // ② 这一版 extent 树里每个号的数据单元数（walk.rs data_unit_count_of_each_inode_without_judging，全零数据指针不数）：号 lo/hi、单元数
 const D_DATA_UNIT_COUNT_WORDS: u32 = 3u;
 
 const D_MAX_DATA_UNIT_INODES: u32 = 1024u;
 
-const D_DATA_UNIT_COUNTS: u32 = 297712u;
+const D_DATA_UNIT_COUNTS: u32 = 297904u;
 
 // ③ 一棵墓碑树或谱系树按叶序读出的容器（walk.rs containers_of_a_container_tree_without_judging）：容器在打包表里的字节基、记录数、
 //    第一条记录的偏移、头里的容器号 lo/hi。一棵树地重填；多于 D_MAX_CONTAINERS_OF_A_CONTAINER_TREE 片判不了（交 CPU）
@@ -818,15 +837,15 @@ const D_CONTAINER_OF_A_CONTAINER_TREE_WORDS: u32 = 5u;
 
 const D_MAX_CONTAINERS_OF_A_CONTAINER_TREE: u32 = 256u;
 
-const D_CONTAINERS_OF_A_CONTAINER_TREE: u32 = 300784u;
+const D_CONTAINERS_OF_A_CONTAINER_TREE: u32 = 300976u;
 
-// ④ 这一版谱系树里的谱系记录（walk.rs lineage_records_without_judging），按 T_rb 从大到小排好：T_rb lo/hi、T_old lo/hi。
+// ④ 这一版谱系树里的谱系记录（walk.rs lineage_records_without_judging），按 T_rb 从大到小排好：T_rb lo/hi、T_old lo/hi、H_cur lo/hi。
 //    多于 D_MAX_LINEAGE_RECORDS 条判不了
-const D_LINEAGE_RECORD_WORDS: u32 = 4u;
+const D_LINEAGE_RECORD_WORDS: u32 = 6u;
 
 const D_MAX_LINEAGE_RECORDS: u32 = 256u;
 
-const D_LINEAGE_RECORDS: u32 = 302064u;
+const D_LINEAGE_RECORDS: u32 = 302256u;
 
 // ⑤ I-9.29 跨版本那张表（walk.rs ObjectBirthsOfEachInodeNumber）：inode 树的树 ID lo/hi、inode 号 lo/hi、第一次见到的对象出生代 lo/hi。
 //    回退候选集里各版读完一起判，不一版一版重填；多于 D_MAX_OBJECT_BIRTHS 个 (树, 号) 判不了
@@ -834,10 +853,29 @@ const D_OBJECT_BIRTH_WORDS: u32 = 6u;
 
 const D_MAX_OBJECT_BIRTHS: u32 = 2048u;
 
-const D_OBJECT_BIRTHS: u32 = 303088u;
+const D_OBJECT_BIRTHS: u32 = 303792u;
+
+// ⑥ I-9.30 那一判读的最新根记账里 inode 号水位那一统计量（不带设备维）的每一行（walk.rs Walk::inode_number_watermark_rows）：代段 lo/hi、值 lo/hi。
+//    最新根一棵记账树一棵树地收（读不全的那棵退回去不收）；多于 D_MAX_INODE_NUMBER_WATERMARK_ROWS 行判不了
+const D_INODE_NUMBER_WATERMARK_ROW_WORDS: u32 = 4u;
+
+const D_MAX_INODE_NUMBER_WATERMARK_ROWS: u32 = 256u;
+
+const D_INODE_NUMBER_WATERMARK_ROWS: u32 = 316080u;
+
+// ⑦ 退役那几条读的两张叶身份表（walk.rs inode_leaf_identities_without_judging）：一版一棵 inode 树里每片叶容器的身份（出生树 lo/hi、打包记录类型、
+//    容器号 lo/hi、容器出生代 lo/hi），集合、一棵树地重填。前一张装被判的那一版（I-9.34、I-9.35），后一张装 I-9.35 的父版；
+//    一棵树多于 D_MAX_LEAF_IDENTITIES 片判不了（一个码 2 根装 135 条 inode 内部条目，合法树至多 135 片）
+const D_LEAF_IDENTITY_WORDS: u32 = 7u;
+
+const D_MAX_LEAF_IDENTITIES: u32 = 256u;
+
+const D_LEAF_IDENTITIES_OF_THE_VERSION: u32 = 317104u;
+
+const D_LEAF_IDENTITIES_OF_THE_PARENT_VERSION: u32 = 318896u;
 
 // 每个状态的草稿区到这里为止：crash_judge_gpu.rs 的 SCRATCH_WORDS 与它相等（那一份的单测核）
-const SCRATCH_END: u32 = 315376u;
+const SCRATCH_END: u32 = 320688u;
 
 fn handoff_region_device(region: u32) -> u32 {
     if (region == 0u) {
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index 96004dd..6413004 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -1,7 +1,7 @@
 // GPU 判器的池级 checker 扫描内核（接在 crash_judge_common.wgsl 与 crash_judge_checker_common.wgsl 之后）：走读内核判完、交接段有效时，
 // 判第三批——扫描方向与记账那九条（I-1.8、I-3.9、I-9.14、I-5.4、I-3.10、I-7.9、I-3.1、I-5.2、I-3.11），再加 I-3.12（每条根各自那张实例表）、
 // I-5.5 与 I-9.28（根表里每一条根的树表），再加里程碑三·3 删除线的删除态扫描（I-9.5、I-3.4、I-4.7、I-4.6、I-9.24，回退候选集每一版）、
-// 墓碑与谱系那一遍（I-9.18、I-9.19、I-9.20、I-9.21、I-9.29 与 I-9.10 墓碑那一个等号，同一份版本清单）与 I-9.6 墓碑那一半
+// 墓碑与谱系那一遍（I-9.18、I-9.19、I-9.20、I-9.21、I-9.29 与 I-9.10 墓碑那一个等号，同一份版本清单；退役记录 I-9.32–I-9.35 也在这一遍）与 I-9.6 墓碑那一半
 // （所选根的墓碑树），违例位并进违例位那几个字，走到第几步那一字写成 5、判不了时下一字写原因。只读不判的读法照 walk.rs 的 *_without_judging 各写一份。
 
 
@@ -637,8 +637,8 @@ fn r_references_of_root(record_base: u32, every: bool, info: u32) {
             r_note_every_node_below(node, 0u);
         } else if (kind == TREE_KIND_DELETED_INODES) {
             // deleted_inodes 根的条目只是号，不引用别的单元（根自己上面记过）
-        } else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE) {
-            // 墓碑树、谱系树（walk.rs 的 TreeKindForReferenceScan::ContainerTree）：根是层级 1 时每条条目的子指针（条目偏移 34 起）指一片容器；
+        } else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) {
+            // 墓碑树、谱系树、退役树（walk.rs 的 TreeKindForReferenceScan::ContainerTree）：根是层级 1 时每条条目的子指针（条目偏移 34 起）指一片容器；
             // 根长高之后（层级 > 1）这一遍不往下数，记成数不全
             if (node.level == 1u && node.entry_width >= CONTAINER_TREE_ENTRY_BYTES) {
                 for (var container = 0u; container < node.entry_count; container = container + 1u) {
@@ -2165,15 +2165,18 @@ const DELETION_TOMBSTONE_RECORD_RANGE_START_OFFSET: u32 = 16u;
 const DELETION_TOMBSTONE_RECORD_RANGE_END_OFFSET: u32 = 24u;
 const DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET: u32 = 32u;
 const DELETION_TOMBSTONE_CONTAINER_RECORDS: u32 = 815u;
-// 谱系记录 16 字节（类型 5 登记 16）：T_rb 0、T_old 8，读作「T_rb 的父是 T_old」（walk.rs LINEAGE_RECORD_*_OFFSET，D18（块里携带什么信息） 已定项 24 第 5 条）
-const DELETION_LINEAGE_RECORD_BYTES: u32 = 16u;
+// 谱系记录 24 字节（类型 5 登记 24）：T_rb 0、T_old 8、H_cur 16（回退之前的现行那一版的 inode 号水位），读作「T_rb 的父是 T_old」
+// （walk.rs LINEAGE_RECORD_*_OFFSET，D18（块里携带什么信息） 已定项 24 第 5 条）
+const DELETION_LINEAGE_RECORD_BYTES: u32 = 24u;
 const DELETION_LINEAGE_RECORD_ROLLBACK_TXG_OFFSET: u32 = 0u;
 const DELETION_LINEAGE_RECORD_PARENT_TXG_OFFSET: u32 = 8u;
+const DELETION_LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET: u32 = 16u;
 // 记账树：key 22（统计量 2、树 8、设备 4、代 8），叶条目 34，内部条目 key + 子指针 86；行里设备段在 10、值在 22
 const DELETION_ACCOUNTING_KEY_BYTES: u32 = 22u;
 const DELETION_ACCOUNTING_ENTRY_BYTES: u32 = 34u;
 const DELETION_ACCOUNTING_INTERNAL_ENTRY_BYTES: u32 = 108u;
 const DELETION_ACCOUNTING_ROW_DEVICE_OFFSET: u32 = 10u;
+const DELETION_ACCOUNTING_ROW_GENERATION_OFFSET: u32 = 14u;
 const DELETION_ACCOUNTING_ROW_VALUE_OFFSET: u32 = 22u;
 // 记账统计量的行号：4 待删除但意图未完成占用（walk.rs STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES）、12 inode 号水位（STATISTIC_INODE_WATERMARK）
 const DELETION_STATISTIC_PENDING_DELETION_OCCUPANCY_BYTES: u32 = 4u;
@@ -2683,6 +2686,14 @@ var<private> d_container_count: u32;
 
 var<private> d_lineage_record_count: u32;
 
+// d_read_the_lineage_of_a_version 顺带记下的、I-9.35 找父版要的（walk.rs judge_tombstones_and_the_lineage_of_a_version 里的 parent_txg）：谱系记录读得出
+// （谱系树恰好一条、容器读得全，不要求接得成链）；叶序里第一条 T_rb 等于这一版 txg 的谱系记录有没有、它的 T_old
+var<private> d_lineage_records_are_readable: bool;
+
+var<private> d_lineage_names_the_parent_of_this_version: bool;
+
+var<private> d_parent_txg_named_by_the_lineage: vec2<u32>;
+
 var<private> d_object_birth_count: u32;
 
 // I-9.29 跨版本那张表比不比：走读内核的主走读沿最新根的实例表链走到了末片（交接段 H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE，
@@ -2791,11 +2802,213 @@ fn d_lineage_parent_txg(lineage_index: u32) -> vec2<u32> {
     return vec2<u32>(sw(row + 2u), sw(row + 3u));
 }
 
+fn d_lineage_inode_number_watermark_rolled_back_from(lineage_index: u32) -> vec2<u32> {
+    let row = D_LINEAGE_RECORDS + lineage_index * D_LINEAGE_RECORD_WORDS;
+    return vec2<u32>(sw(row + 4u), sw(row + 5u));
+}
+
+// ───────────────────────── I-9.30（walk.rs judge_inode_number_watermark_steps_of_the_newest_root）─────────────────────────
+// 最新根记账里 inode 号水位那一统计量（标签 12、不带设备维）的行收进 D_INODE_NUMBER_WATERMARK_ROWS；台阶行 (g, H) 照 walk.rs
+// inode_number_watermark_steps_of_a_version 认：代段早于最新根 txg 的行都是台阶，代段等于它的那一行值与代段最大的台阶不同（或没有更早的台阶）才是台阶。
+// ① 判在收行那一步；② 判在墓碑与谱系那一遍走到最新根那一版时（inode 记录、谱系上的墓碑）；④ 也在那里判（谱系记录带的 H_cur）。
+// CPU 记成射程里判不了的（记账树、树表、谱系、inode 树、墓碑树读不全，谱系接不成链）这里一格都不置位；行多于上限判不了
+var<private> d_inode_number_watermark_row_count: u32;
+// 收到了最新根的 inode 号水位行（CPU 不报不适用）：②④ 只在这时判
+var<private> d_inode_number_watermark_steps_are_judged: bool;
+var<private> d_newest_root_txg: vec2<u32>;
+// 代段早于最新根 txg 的行里代段最大那一行的值（台阶里最后一级），has_an_earlier_step 为假是没有这样的行
+var<private> d_latest_earlier_step_watermark: vec2<u32>;
+var<private> d_has_an_earlier_step: bool;
+
+fn d_inode_number_watermark_row_generation(row_index: u32) -> vec2<u32> {
+    let row = D_INODE_NUMBER_WATERMARK_ROWS + row_index * D_INODE_NUMBER_WATERMARK_ROW_WORDS;
+    return vec2<u32>(sw(row), sw(row + 1u));
+}
+
+fn d_inode_number_watermark_row_value(row_index: u32) -> vec2<u32> {
+    let row = D_INODE_NUMBER_WATERMARK_ROWS + row_index * D_INODE_NUMBER_WATERMARK_ROW_WORDS;
+    return vec2<u32>(sw(row + 2u), sw(row + 3u));
+}
+
+// 第 row_index 行是不是一级台阶（认法见本节开头）
+fn d_inode_number_watermark_row_is_a_step(row_index: u32) -> bool {
+    let generation = d_inode_number_watermark_row_generation(row_index);
+    if (lt64(generation, d_newest_root_txg)) {
+        return true;
+    }
+    if (!eq64(generation, d_newest_root_txg)) {
+        return false;
+    }
+    return !d_has_an_earlier_step || !eq64(d_inode_number_watermark_row_value(row_index), d_latest_earlier_step_watermark);
+}
+
+// ② 那一判：号 inode、对象出生代 object_birth 的对象，有没有一级台阶 (g, H) 使 H > inode 且 object_birth > g
+fn d_number_was_issued_before_a_step(inode: vec2<u32>, object_birth: vec2<u32>) -> bool {
+    for (var row_index = 0u; row_index < d_inode_number_watermark_row_count; row_index = row_index + 1u) {
+        if (d_inode_number_watermark_row_is_a_step(row_index)
+            && lt64(inode, d_inode_number_watermark_row_value(row_index))
+            && lt64(d_inode_number_watermark_row_generation(row_index), object_birth)) {
+            return true;
+        }
+    }
+    return false;
+}
+
+// 最新根一棵记账树（根指针在 (base, offset)，非零）里标签 12、设备段 0xFFFF_FFFF 的行接着收进 D_INODE_NUMBER_WATERMARK_ROWS：读法同
+// d_judge_pending_deletion_occupancy_of_an_accounting_tree（key 宽 22；叶条目宽 ≥ 34；内部条目宽 ≥ 108、子指针非零、孩子层级 = 父层级 − 1）；
+// 读不全交回 false、这棵树收的行退回去（CPU 只取走全了的记账树的行）
+fn d_collect_inode_number_watermark_rows_of_an_accounting_tree(base: u32, offset: u32) -> bool {
+    let rows_before_this_tree = d_inode_number_watermark_row_count;
+    var pending = 1u;
+    sws(C_R_PENDING, base);
+    sws(C_R_PENDING + 1u, offset);
+    sws(C_R_PENDING + 2u, NONE);   // 根：层级取它自己头里的
+    while (pending > 0u) {
+        pending = pending - 1u;
+        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
+        let p = parse_node_pointer(sw(row), sw(row + 1u));
+        let expected_level = sw(row + 2u);
+        let n = r_read_index_node(p);
+        if (!n.ok || (expected_level != NONE && n.level != expected_level) || n.key_width != DELETION_ACCOUNTING_KEY_BYTES) {
+            d_inode_number_watermark_row_count = rows_before_this_tree;
+            return false;
+        }
+        if (n.level == 0u) {
+            if (n.entry_width < DELETION_ACCOUNTING_ENTRY_BYTES) {
+                d_inode_number_watermark_row_count = rows_before_this_tree;
+                return false;
+            }
+            for (var e = 0u; e < n.entry_count; e = e + 1u) {
+                let o = entry_offset(n, e);
+                if (rd16(n.base, o) != DELETION_STATISTIC_INODE_WATERMARK || rd32(n.base, o + DELETION_ACCOUNTING_ROW_DEVICE_OFFSET) != NONE) {
+                    continue;
+                }
+                if (d_inode_number_watermark_row_count >= D_MAX_INODE_NUMBER_WATERMARK_ROWS) {
+                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (63u << 8u);
+                    d_inode_number_watermark_row_count = rows_before_this_tree;
+                    return false;
+                }
+                let generation = rd64(n.base, o + DELETION_ACCOUNTING_ROW_GENERATION_OFFSET);
+                let value = rd64(n.base, o + DELETION_ACCOUNTING_ROW_VALUE_OFFSET);
+                let collected = D_INODE_NUMBER_WATERMARK_ROWS + d_inode_number_watermark_row_count * D_INODE_NUMBER_WATERMARK_ROW_WORDS;
+                sws(collected, generation.x);
+                sws(collected + 1u, generation.y);
+                sws(collected + 2u, value.x);
+                sws(collected + 3u, value.y);
+                d_inode_number_watermark_row_count = d_inode_number_watermark_row_count + 1u;
+            }
+            continue;
+        }
+        if (n.entry_width < DELETION_ACCOUNTING_INTERNAL_ENTRY_BYTES) {
+            d_inode_number_watermark_row_count = rows_before_this_tree;
+            return false;
+        }
+        for (var e = 0u; e < n.entry_count; e = e + 1u) {
+            let o = entry_offset(n, e) + DELETION_ACCOUNTING_KEY_BYTES;
+            if (parse_node_pointer(n.base, o).all_zero) {
+                d_inode_number_watermark_row_count = rows_before_this_tree;
+                return false;
+            }
+            if (pending >= C_R_MAX_PENDING) {
+                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (64u << 8u);
+                d_inode_number_watermark_row_count = rows_before_this_tree;
+                return false;
+            }
+            let child = C_R_PENDING + pending * C_R_PENDING_WORDS;
+            sws(child, n.base);
+            sws(child + 1u, o);
+            sws(child + 2u, n.level - 1u);
+            pending = pending + 1u;
+        }
+    }
+    return true;
+}
+
+// 收最新根记账里 inode 号水位那一统计量的行（树表里每一棵根指针非零的记账树），定台阶，判 ①：记账里的水位（代段最大的那一行的值）
+// 不小于每一级台阶的 H。一行都没收到（没有走全了的记账树、或没有那一行）时 ②④ 不判（CPU 报不适用）
+fn judge_inode_number_watermark_steps_against_the_watermark_of_the_newest_root() {
+    d_inode_number_watermark_row_count = 0u;
+    d_inode_number_watermark_steps_are_judged = false;
+    let newest_root = checker_root_of(c_newest_index);
+    d_newest_root_txg = newest_root.txg;
+    let tree_table_pointer = parse_node_pointer(newest_root.base, 36u);
+    if (tree_table_pointer.all_zero) {
+        return;
+    }
+    let tt = r_read_index_node(tree_table_pointer);
+    if (!tt.ok || tt.entry_width < DELETION_TREE_TABLE_ENTRY_BYTES) {
+        return;
+    }
+    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
+        let entry = entry_offset(tt, entry_index);
+        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) != TREE_KIND_ACCOUNTING
+            || parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET).all_zero) {
+            continue;
+        }
+        if (!d_collect_inode_number_watermark_rows_of_an_accounting_tree(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET)
+            && undecidable != 0u) {
+            return;
+        }
+    }
+    if (d_inode_number_watermark_row_count == 0u) {
+        return;
+    }
+    d_inode_number_watermark_steps_are_judged = true;
+    var watermark_generation = d_inode_number_watermark_row_generation(0u);
+    var watermark = d_inode_number_watermark_row_value(0u);
+    d_has_an_earlier_step = false;
+    var latest_earlier_step_generation = vec2<u32>(0u, 0u);
+    d_latest_earlier_step_watermark = vec2<u32>(0u, 0u);
+    for (var row_index = 0u; row_index < d_inode_number_watermark_row_count; row_index = row_index + 1u) {
+        let generation = d_inode_number_watermark_row_generation(row_index);
+        if (lt64(watermark_generation, generation)) {
+            watermark_generation = generation;
+            watermark = d_inode_number_watermark_row_value(row_index);
+        }
+        if (lt64(generation, d_newest_root_txg) && (!d_has_an_earlier_step || lt64(latest_earlier_step_generation, generation))) {
+            d_has_an_earlier_step = true;
+            latest_earlier_step_generation = generation;
+            d_latest_earlier_step_watermark = d_inode_number_watermark_row_value(row_index);
+        }
+    }
+    for (var row_index = 0u; row_index < d_inode_number_watermark_row_count; row_index = row_index + 1u) {
+        if (d_inode_number_watermark_row_is_a_step(row_index) && lt64(watermark, d_inode_number_watermark_row_value(row_index))) {
+            violate(INVARIANT_I_9_30);
+        }
+    }
+}
+
+// ④ 那一判（谱系已读进 D_LINEAGE_RECORDS、接得成链）：每条谱系记录 (T_rb, T_old, H_cur)，代段 < T_rb 的台阶里最大的 H 等于 H_cur；
+// 没有这样的台阶判红；H_cur 为 0 时等式不成立（台阶的水位至少是 1），同 walk.rs 不另写一判
+fn d_judge_the_lineage_against_the_inode_number_watermark_steps() {
+    for (var lineage_index = 0u; lineage_index < d_lineage_record_count; lineage_index = lineage_index + 1u) {
+        let rollback_txg = d_lineage_rollback_txg(lineage_index);
+        let watermark_rolled_back_from = d_lineage_inode_number_watermark_rolled_back_from(lineage_index);
+        var has_a_step_before_the_rollback = false;
+        var largest_step_before_the_rollback = vec2<u32>(0u, 0u);
+        for (var row_index = 0u; row_index < d_inode_number_watermark_row_count; row_index = row_index + 1u) {
+            if (!d_inode_number_watermark_row_is_a_step(row_index) || !lt64(d_inode_number_watermark_row_generation(row_index), rollback_txg)) {
+                continue;
+            }
+            let step_watermark = d_inode_number_watermark_row_value(row_index);
+            if (!has_a_step_before_the_rollback || lt64(largest_step_before_the_rollback, step_watermark)) {
+                largest_step_before_the_rollback = step_watermark;
+            }
+            has_a_step_before_the_rollback = true;
+        }
+        if (!has_a_step_before_the_rollback || !eq64(largest_step_before_the_rollback, watermark_rolled_back_from)) {
+            violate(INVARIANT_I_9_30);
+        }
+    }
+}
+
 // 这一版的谱系（walk.rs judge_tombstones_and_the_lineage_of_a_version 开头与 lineage_chain_of_a_version 判链之前那两步）：树表 tt 里谱系树（种类 11）
 // 恰好一条、整棵读得出、记录接得成一条链（每条 T_old < T_rb、没有两条同一个 T_rb）才交回 true，谱系记录按 T_rb 从大到小排进 D_LINEAGE_RECORDS；
 // 否则交回 false（这一版要谱系的那几判判不了）
-fn d_read_the_lineage_of_a_version(tt: Node) -> bool {
+fn d_read_the_lineage_of_a_version(tt: Node, version_txg: vec2<u32>) -> bool {
     d_lineage_record_count = 0u;
+    d_lineage_records_are_readable = false;
+    d_lineage_names_the_parent_of_this_version = false;
     var lineage_entry_count = 0u;
     var lineage_entry = 0u;
     for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
@@ -2822,14 +3035,22 @@ fn d_read_the_lineage_of_a_version(tt: Node) -> bool {
             let record = d_container_record_offset(container_index, record_index, DELETION_LINEAGE_RECORD_BYTES);
             let rollback_txg = rd64(base, record + DELETION_LINEAGE_RECORD_ROLLBACK_TXG_OFFSET);
             let parent_txg = rd64(base, record + DELETION_LINEAGE_RECORD_PARENT_TXG_OFFSET);
+            let watermark_rolled_back_from = rd64(base, record + DELETION_LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET);
             let row = D_LINEAGE_RECORDS + d_lineage_record_count * D_LINEAGE_RECORD_WORDS;
             sws(row, rollback_txg.x);
             sws(row + 1u, rollback_txg.y);
             sws(row + 2u, parent_txg.x);
             sws(row + 3u, parent_txg.y);
+            sws(row + 4u, watermark_rolled_back_from.x);
+            sws(row + 5u, watermark_rolled_back_from.y);
             d_lineage_record_count = d_lineage_record_count + 1u;
+            if (!d_lineage_names_the_parent_of_this_version && eq64(rollback_txg, version_txg)) {
+                d_lineage_names_the_parent_of_this_version = true;
+                d_parent_txg_named_by_the_lineage = parent_txg;
+            }
         }
     }
+    d_lineage_records_are_readable = true;
     // 按 T_rb 从大到小排（选择排序：条数不过 D_MAX_LINEAGE_RECORDS）
     for (var sorted = 0u; sorted < d_lineage_record_count; sorted = sorted + 1u) {
         var latest = sorted;
@@ -3087,9 +3308,452 @@ fn d_judge_queued_inodes_have_tombstones_on_the_lineage(tt: Node, inode_tree_ent
     }
 }
 
+// ───────────────────────── 退役记录那四条（walk.rs 的 RETIREMENT_*：I-9.32–I-9.35）─────────────────────────
+// 判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条原写「GPU 判器同判 I-9.31，其余回落 CPU」；用户 2026-09-29 定「配了 GPU 就一定要
+// GPU 判，不退回 CPU」，GPU 对拍文件头「一个状态都不许判不了」，第 50 批补做把这四条也搬上 GPU，不记判不了。
+// 都在墓碑与谱系那一遍的每一版上判（d_judge_tombstones_and_the_lineage_of_a_version）：退役容器借墓碑树那一处读容器的调用点读，
+// 每版一处 d_judge_the_retirement_records_and_their_completeness 判 I-9.32、I-9.33、I-9.34，并就地跨到父版判 I-9.35；叶身份只有一个读的调用点
+// （被判那一版与父版轮流走它）。展开预算（crash_judge_dispatch.rs 的 LARGEST_INLINED_EXPRESSIONS_OF_A_KERNEL）只许这四条多占两万来个表达式，
+// 重的读法都只留一个调用点。
+// I-9.32 每条都判；I-9.33、I-9.34 只判死亡 txg 在这一版谱系上的（「在谱系上」同墓碑那几判，d_txg_is_on_the_lineage）。
+// CPU 记成射程里判不了的那几格（退役树、inode 树、谱系树读不全，谱系接不成链，头 ID 配不上，父版读不全或不止一版）这里一格都不置位；
+// 表越过上限（一版 inode 树的叶身份、两两比不重复的退役记录总数、同一个 txg 的版本数）判不了，交 CPU。
+
+// 退役记录 34 字节（类型 3 登记 34）：被退役身份的出生树 0、打包记录类型 8（2 字节）、容器号 10、容器出生代 18、死亡 txg 26
+// （walk.rs RETIREMENT_RECORD_*_OFFSET；身份四段与 inode 树内部条目里的身份引用同序同宽）；一片退役容器最多 959 条
+// （⌊(32768 − 136) ÷ 34⌋，walk.rs RETIREMENT_CONTAINER_RECORDS）
+const DELETION_RETIREMENT_RECORD_BYTES: u32 = 34u;
+const DELETION_RETIREMENT_RECORD_BIRTH_TREE_OFFSET: u32 = 0u;
+const DELETION_RETIREMENT_RECORD_RECORD_TYPE_OFFSET: u32 = 8u;
+const DELETION_RETIREMENT_RECORD_CONTAINER_OFFSET: u32 = 10u;
+const DELETION_RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET: u32 = 18u;
+const DELETION_RETIREMENT_RECORD_DEATH_TXG_OFFSET: u32 = 26u;
+const DELETION_RETIREMENT_CONTAINER_RECORDS: u32 = 959u;
+// inode 树码 2 节点条目里身份引用的起点（分隔 key 8 字节之后；类型段就是身份里的打包记录类型，在条目偏移 16）
+const DELETION_INODE_INTERNAL_ENTRY_IDENTITY_OFFSET: u32 = 8u;
+// I-9.33 两两比：一棵退役树的退役记录多于这么多条就判不了（比较次数随条数平方涨，同墓碑那一判的上限）
+const DELETION_MAX_RETIREMENT_RECORDS_COMPARED_PAIRWISE: u32 = 4096u;
+// I-9.35 找父版：同一个 txg 的版本（按 walk.rs 的去重算不同的）最多记几份树表指针，多于它判不了（合法镜像上同一个 txg 至多两三份）
+const DELETION_MAX_VERSIONS_AT_ONE_TXG: u32 = 8u;
+// 这一节判不了的站点号（「条目太多」一类，与别处不撞）：读叶身份时压 C_R_PENDING 压满、叶身份表满、两两比的退役记录太多、同一个 txg 的版本太多
+const DELETION_SITE_LEAF_IDENTITIES_PENDING: u32 = 65u;
+const DELETION_SITE_LEAF_IDENTITY_TABLE_FULL: u32 = 66u;
+const DELETION_SITE_RETIREMENT_RECORDS_COMPARED_PAIRWISE: u32 = 67u;
+const DELETION_SITE_VERSIONS_AT_ONE_TXG: u32 = 68u;
+
+// 一片打包记录容器的身份四段（walk.rs ContainerIdentityOfTheScan）
+struct ContainerIdentityOfTheScan {
+    birth_tree: vec2<u32>,
+    record_type: u32,
+    container: vec2<u32>,
+    container_birth: vec2<u32>,
+};
+
+// 从 (base, o) 起按「出生树 8 | 打包记录类型 2 | 容器号 8 | 容器出生代 8」读一个身份
+fn d_container_identity_at(base: u32, o: u32) -> ContainerIdentityOfTheScan {
+    var identity: ContainerIdentityOfTheScan;
+    identity.birth_tree = rd64(base, o + DELETION_RETIREMENT_RECORD_BIRTH_TREE_OFFSET);
+    identity.record_type = rd16(base, o + DELETION_RETIREMENT_RECORD_RECORD_TYPE_OFFSET);
+    identity.container = rd64(base, o + DELETION_RETIREMENT_RECORD_CONTAINER_OFFSET);
+    identity.container_birth = rd64(base, o + DELETION_RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET);
+    return identity;
+}
+
+fn d_same_container_identity(left: ContainerIdentityOfTheScan, right: ContainerIdentityOfTheScan) -> bool {
+    return eq64(left.birth_tree, right.birth_tree) && left.record_type == right.record_type
+        && eq64(left.container, right.container) && eq64(left.container_birth, right.container_birth);
+}
+
+// 叶身份表（D_LEAF_IDENTITIES_OF_THE_VERSION 或 D_LEAF_IDENTITIES_OF_THE_PARENT_VERSION）的第 row_index 行
+fn d_leaf_identity_at(table: u32, row_index: u32) -> ContainerIdentityOfTheScan {
+    let row = table + row_index * D_LEAF_IDENTITY_WORDS;
+    var identity: ContainerIdentityOfTheScan;
+    identity.birth_tree = vec2<u32>(sw(row), sw(row + 1u));
+    identity.record_type = sw(row + 2u);
+    identity.container = vec2<u32>(sw(row + 3u), sw(row + 4u));
+    identity.container_birth = vec2<u32>(sw(row + 5u), sw(row + 6u));
+    return identity;
+}
+
+fn d_put_leaf_identity(table: u32, row_index: u32, identity: ContainerIdentityOfTheScan) {
+    let row = table + row_index * D_LEAF_IDENTITY_WORDS;
+    sws(row, identity.birth_tree.x);
+    sws(row + 1u, identity.birth_tree.y);
+    sws(row + 2u, identity.record_type);
+    sws(row + 3u, identity.container.x);
+    sws(row + 4u, identity.container.y);
+    sws(row + 5u, identity.container_birth.x);
+    sws(row + 6u, identity.container_birth.y);
+}
+
+// 叶身份表前 identity_count 行里有没有这个身份
+fn d_leaf_identity_table_holds(table: u32, identity_count: u32, identity: ContainerIdentityOfTheScan) -> bool {
+    for (var row_index = 0u; row_index < identity_count; row_index = row_index + 1u) {
+        if (d_same_container_identity(d_leaf_identity_at(table, row_index), identity)) {
+            return true;
+        }
+    }
+    return false;
+}
+
+// walk.rs inode_leaf_identities_without_judging：一版 inode 树（根指针在 (base, root_pointer_offset)）里每片叶容器的身份——码 2 节点里类型段 2
+// 那几条条目的身份引用，不开叶——收进叶身份表 table（集合：同一个身份只记一行），交回条数。根指针全零、零条目根交回 0；根读不出、根层级 0 而有条目、
+// 码 2 节点条目宽不到 120、类型段既不是 0 也不是 2、类型段 0 的子指针全零或读不出、子层级不是父层级 − 1 或是 0，都交回 NONE（判不了）；
+// 表满、C_R_PENDING 压满记判不了、交回 NONE。根与每个孩子都经同一处读（C_R_PENDING 每行：指针所在的字节基、指针偏移、父层级，根的父层级记 NONE），
+// 次序不影响集合
+fn d_read_inode_leaf_identities(base: u32, root_pointer_offset: u32, table: u32) -> u32 {
+    var identity_count = 0u;
+    sws(C_R_PENDING, base);
+    sws(C_R_PENDING + 1u, root_pointer_offset);
+    sws(C_R_PENDING + 2u, NONE);
+    var pending = 1u;
+    while (pending > 0u) {
+        pending = pending - 1u;
+        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
+        let parent_level = sw(row + 2u);
+        let pointer = parse_node_pointer(sw(row), sw(row + 1u));
+        if (pointer.all_zero) {
+            if (parent_level == NONE) {
+                continue;
+            }
+            return NONE;
+        }
+        let node = r_read_index_node(pointer);
+        if (!node.ok) {
+            return NONE;
+        }
+        if (parent_level == NONE && node.level == 0u) {
+            if (node.entry_count == 0u) {
+                continue;
+            }
+            return NONE;
+        }
+        if ((parent_level != NONE && (node.level + 1u != parent_level || node.level == 0u)) || node.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
+            return NONE;
+        }
+        for (var entry_index = 0u; entry_index < node.entry_count; entry_index = entry_index + 1u) {
+            let entry = entry_offset(node, entry_index);
+            let entry_type = rd16(node.base, entry + DELETION_CONTAINER_TREE_ENTRY_TYPE_OFFSET);
+            if (entry_type == DELETION_PACKED_TYPE_INODE) {
+                let identity = d_container_identity_at(node.base, entry + DELETION_INODE_INTERNAL_ENTRY_IDENTITY_OFFSET);
+                if (d_leaf_identity_table_holds(table, identity_count, identity)) {
+                    continue;
+                }
+                if (identity_count >= D_MAX_LEAF_IDENTITIES) {
+                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (DELETION_SITE_LEAF_IDENTITY_TABLE_FULL << 8u);
+                    return NONE;
+                }
+                d_put_leaf_identity(table, identity_count, identity);
+                identity_count = identity_count + 1u;
+            } else if (entry_type == DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE) {
+                if (pending >= C_R_MAX_PENDING) {
+                    undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (DELETION_SITE_LEAF_IDENTITIES_PENDING << 8u);
+                    return NONE;
+                }
+                let pushed = C_R_PENDING + pending * C_R_PENDING_WORDS;
+                sws(pushed, node.base);
+                sws(pushed + 1u, entry + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
+                sws(pushed + 2u, node.level);
+                pending = pending + 1u;
+            } else {
+                return NONE;
+            }
+        }
+    }
+    return identity_count;
+}
+
+// 树表 tt 里种类 kind、条目偏移 identifier_offset 处的 8 字节（树 ID 在 0，头 ID 在 TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET）等于 identifier 的最后一条条目的偏移
+// （walk.rs 按树 ID / 头 ID 收进 BTreeMap，同一个键的后一条盖掉前一条）；没有交回 NONE
+fn d_last_tree_table_entry_of_kind_keyed_by(tt: Node, kind: u32, identifier_offset: u32, identifier: vec2<u32>) -> u32 {
+    var last_entry = NONE;
+    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
+        let entry = entry_offset(tt, entry_index);
+        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == kind && eq64(rd64(tt.base, entry + identifier_offset), identifier)) {
+            last_entry = entry;
+        }
+    }
+    return last_entry;
+}
+
+// I-9.32（walk.rs judge_every_retirement_record_of_a_version_is_well_formed）：读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的退役容器每片记录数 ≤ 959；
+// 每条被退役身份的打包记录类型 == 2、出生树 ≠ 0，容器出生代 < 死亡 txg ≤ 这一版的 txg。不按谱系滤
+fn d_judge_every_retirement_record_is_well_formed(version_txg: vec2<u32>) {
+    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
+        let base = d_container_base(container_index);
+        let record_count = d_container_record_count(container_index);
+        if (record_count > DELETION_RETIREMENT_CONTAINER_RECORDS) {
+            violate(INVARIANT_I_9_32);
+        }
+        for (var record_index = 0u; record_index < record_count; record_index = record_index + 1u) {
+            let o = d_container_record_offset(container_index, record_index, DELETION_RETIREMENT_RECORD_BYTES);
+            let retired = d_container_identity_at(base, o);
+            let death_txg = rd64(base, o + DELETION_RETIREMENT_RECORD_DEATH_TXG_OFFSET);
+            if (retired.record_type != DELETION_PACKED_TYPE_INODE || is_zero64(retired.birth_tree)
+                || !lt64(retired.container_birth, death_txg) || !le64(death_txg, version_txg)) {
+                violate(INVARIANT_I_9_32);
+            }
+        }
+    }
+}
+
+// I-9.33（walk.rs judge_retirement_records_on_the_lineage_of_a_version_are_distinct）：读进来的退役容器里死亡 txg 在这一版谱系上的退役记录按身份不重复。
+// 两两比，退役记录总数多于 DELETION_MAX_RETIREMENT_RECORDS_COMPARED_PAIRWISE 判不了
+fn d_judge_retirement_records_on_the_lineage_are_distinct(version_txg: vec2<u32>) {
+    var record_total = 0u;
+    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
+        record_total = record_total + d_container_record_count(container_index);
+    }
+    if (record_total > DELETION_MAX_RETIREMENT_RECORDS_COMPARED_PAIRWISE) {
+        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (DELETION_SITE_RETIREMENT_RECORDS_COMPARED_PAIRWISE << 8u);
+        return;
+    }
+    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
+        let base = d_container_base(container_index);
+        for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
+            let o = d_container_record_offset(container_index, record_index, DELETION_RETIREMENT_RECORD_BYTES);
+            if (!d_txg_is_on_the_lineage(rd64(base, o + DELETION_RETIREMENT_RECORD_DEATH_TXG_OFFSET), version_txg)) {
+                continue;
+            }
+            let retired = d_container_identity_at(base, o);
+            // 叶序在它之前的退役记录：前几片容器的全部、同一片里它之前的
+            for (var earlier_container = 0u; earlier_container <= container_index; earlier_container = earlier_container + 1u) {
+                let earlier_base = d_container_base(earlier_container);
+                var earlier_limit = d_container_record_count(earlier_container);
+                if (earlier_container == container_index) {
+                    earlier_limit = record_index;
+                }
+                for (var earlier_record = 0u; earlier_record < earlier_limit; earlier_record = earlier_record + 1u) {
+                    let earlier = d_container_record_offset(earlier_container, earlier_record, DELETION_RETIREMENT_RECORD_BYTES);
+                    if (d_same_container_identity(d_container_identity_at(earlier_base, earlier), retired)
+                        && d_txg_is_on_the_lineage(rd64(earlier_base, earlier + DELETION_RETIREMENT_RECORD_DEATH_TXG_OFFSET), version_txg)) {
+                        violate(INVARIANT_I_9_33);
+                    }
+                }
+            }
+        }
+    }
+}
+
+// 读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的退役容器里有没有一条死亡 txg 等于 death_txg、身份是 identity 的退役记录
+fn d_retired_at(identity: ContainerIdentityOfTheScan, death_txg: vec2<u32>) -> bool {
+    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
+        let base = d_container_base(container_index);
+        for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
+            let o = d_container_record_offset(container_index, record_index, DELETION_RETIREMENT_RECORD_BYTES);
+            if (eq64(rd64(base, o + DELETION_RETIREMENT_RECORD_DEATH_TXG_OFFSET), death_txg) && d_same_container_identity(d_container_identity_at(base, o), identity)) {
+                return true;
+            }
+        }
+    }
+    return false;
+}
+
+// 删除态扫描那一份版本清单里的一版：树表指针所在的字节基与偏移、txg（先候选集里每条根、再由记录施加出来的那几版）
+struct VersionOfTheDeletionScan {
+    base: u32,
+    tree_table_offset: u32,
+    txg: vec2<u32>,
+};
+
+fn d_version_of_the_scan(version_index: u32) -> VersionOfTheDeletionScan {
+    var version: VersionOfTheDeletionScan;
+    if (version_index < c_candidate_count) {
+        let root = checker_root_of(sw(C_CANDIDATES + version_index));
+        version.base = root.base;
+        version.tree_table_offset = 36u;
+        version.txg = root.txg;
+    } else {
+        let record = checker_record_at(sw(R_DATA + (version_index - c_candidate_count)));
+        version.base = record.base;
+        version.tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
+        version.txg = record.txg;
+    }
+    return version;
+}
+
+// I-9.35 的父版在版本清单里是哪一份：walk.rs 按 (树表指针第一条位置条目的盘、槽、校验和, txg) 去重、只记第一次（树表指针全零的版不去重，每次都记一版），
+// 去过重之后 txg 等于 txg 的恰好一份才比（没有不判、不止一份报判不了）。is_single 为真时 pointer 是那一份的树表指针
+struct SingleVersionAtATxg {
+    is_single: bool,
+    pointer: Ptr,
+};
+
+fn d_single_version_of_the_scan_at_txg(txg: vec2<u32>) -> SingleVersionAtATxg {
+    var found: SingleVersionAtATxg;
+    found.is_single = false;
+    var distinct_versions = 0u;
+    var distinct_locations: array<vec4<u32>, 8>;
+    var distinct_location_count = 0u;
+    for (var version_index = 0u; version_index < c_candidate_count + c_version_count; version_index = version_index + 1u) {
+        let version = d_version_of_the_scan(version_index);
+        if (!eq64(version.txg, txg)) {
+            continue;
+        }
+        let pointer = parse_node_pointer(version.base, version.tree_table_offset);
+        if (!pointer.all_zero) {
+            let location = vec4<u32>(pointer.device0, pointer.slot0.x, pointer.slot0.y, pointer.sum0);
+            var repeats_an_earlier_one = false;
+            for (var location_index = 0u; location_index < distinct_location_count; location_index = location_index + 1u) {
+                if (all(distinct_locations[location_index] == location)) {
+                    repeats_an_earlier_one = true;
+                }
+            }
+            if (repeats_an_earlier_one) {
+                continue;
+            }
+            if (distinct_location_count >= DELETION_MAX_VERSIONS_AT_ONE_TXG) {
+                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (DELETION_SITE_VERSIONS_AT_ONE_TXG << 8u);
+                return found;
+            }
+            distinct_locations[distinct_location_count] = location;
+            distinct_location_count = distinct_location_count + 1u;
+        }
+        distinct_versions = distinct_versions + 1u;
+        found.pointer = pointer;
+    }
+    found.is_single = distinct_versions == 1u;
+    return found;
+}
+
+// 一版的 I-9.35 要比的父版（walk.rs 里的 parent_txg 与 judge_retirement_records_account_for_every_leaf_gone_since_the_parent_version 的门）：
+// 主走读沿最新根的实例表链走到了末片（候选集滤得掉被崩溃恢复抛弃的根，同 I-9.29 那张表的门）、这一版谱系记录读得出、父版 txg 有
+// （d_read_the_lineage_of_a_version 叶序第一条 T_rb 等于这一版 txg 的 T_old，否则 txg − 1，txg 0 没有），版本清单里那一份恰好一版、
+// 树表读得出（0 条也算：每棵树是空集）、条目不窄，这一版又没有一棵退役树的头 ID 配不上这一版的任何一棵 inode 树：is_judged 为真，tree_table 是父版的树表
+struct ParentVersionToCompare {
+    is_judged: bool,
+    tree_table: Node,
+};
+
+fn d_parent_version_to_compare(tt: Node, version_txg: vec2<u32>) -> ParentVersionToCompare {
+    var parent: ParentVersionToCompare;
+    parent.is_judged = false;
+    parent.tree_table = bad_node();
+    if (!d_object_births_are_compared || !d_lineage_records_are_readable) {
+        return parent;
+    }
+    var parent_txg = d_parent_txg_named_by_the_lineage;
+    if (!d_lineage_names_the_parent_of_this_version) {
+        if (is_zero64(version_txg)) {
+            return parent;
+        }
+        parent_txg = sub64(version_txg, vec2<u32>(1u, 0u));
+    }
+    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
+        let entry = entry_offset(tt, entry_index);
+        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_RETIREMENT
+            && d_last_tree_table_entry_of_kind_keyed_by(tt, TREE_KIND_INODE, 0u, rd64(tt.base, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET)) == NONE) {
+            return parent;
+        }
+    }
+    let single = d_single_version_of_the_scan_at_txg(parent_txg);
+    if (!single.is_single || single.pointer.all_zero) {
+        return parent;
+    }
+    parent.tree_table = r_read_index_node(single.pointer);
+    parent.is_judged = parent.tree_table.ok
+        && (parent.tree_table.entry_count == 0u || parent.tree_table.entry_width >= DELETION_TREE_TABLE_ENTRY_BYTES);
+    return parent;
+}
+
+// 这一版树表 tt 里条目 entry（一棵退役树、容器已读进 D_CONTAINERS_OF_A_CONTAINER_TREE；或一棵没有退役树配它的 inode 树、d_container_count 已置 0）：
+// - 退役树：I-9.32 每条判；谱系接得成链时判 I-9.33，再经头 ID 配这张表里树 ID 等于它的最后一棵 inode 树判 I-9.34（walk.rs judge_tombstones_and_the_lineage_of_a_version 末段）；
+// - I-9.35（walk.rs judge_retirement_records_account_for_every_leaf_gone_since_the_parent_version）：父版要比（parent.is_judged）时，对一棵 inode 树 T 比一次——
+//   退役树那一支只在它是头 ID 为 T 的最后一棵时比（walk.rs 按头 ID 收进 BTreeMap，后一棵盖掉前一棵），T 是它头 ID 配上的那棵；
+//   inode 树那一支是表里同 ID 的最后一条、又没有一棵退役树的头 ID 是它（死于这一版的退役集合是空集）。
+//   父版那棵树的叶身份集 − 这一版的叶身份集 = 死于这一版（死亡 txg 等于这一版 txg）的退役记录的身份集，漏记、多记都置位；父版没有 T 那一棵是空集。
+// 叶身份按被判那一版、父版的次序轮流经同一处读（读不全的那一版那一判不判）
+fn d_judge_the_retirement_records_and_their_completeness(tt: Node, entry: u32, version_txg: vec2<u32>, lineage_holds: bool,
+    parent: ParentVersionToCompare) {
+    var inode_tree_entry = entry;
+    var judges_no_live_leaf = false;
+    var judges_completeness = parent.is_judged;
+    if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_RETIREMENT) {
+        d_judge_every_retirement_record_is_well_formed(version_txg);
+        if (lineage_holds) {
+            d_judge_retirement_records_on_the_lineage_are_distinct(version_txg);
+        }
+        let head_identifier = rd64(tt.base, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
+        inode_tree_entry = d_last_tree_table_entry_of_kind_keyed_by(tt, TREE_KIND_INODE, 0u, head_identifier);
+        judges_no_live_leaf = lineage_holds && inode_tree_entry != NONE;
+        judges_completeness = judges_completeness && inode_tree_entry != NONE
+            && d_last_tree_table_entry_of_kind_keyed_by(tt, TREE_KIND_RETIREMENT, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET, head_identifier) == entry;
+    }
+    if (!judges_no_live_leaf && !judges_completeness) {
+        return;
+    }
+    let inode_tree = rd64(tt.base, inode_tree_entry);
+    var parent_inode_tree_entry = NONE;
+    if (judges_completeness) {
+        parent_inode_tree_entry = d_last_tree_table_entry_of_kind_keyed_by(parent.tree_table, TREE_KIND_INODE, 0u, inode_tree);
+    }
+    // 第 0 轮读这一版那棵树进 D_LEAF_IDENTITIES_OF_THE_VERSION，第 1 轮读父版那棵进 D_LEAF_IDENTITIES_OF_THE_PARENT_VERSION（父版没有那一棵是空集，不读）
+    var leaf_identity_counts = vec2<u32>(0u, 0u);
+    var readings = 1u;
+    if (judges_completeness && parent_inode_tree_entry != NONE) {
+        readings = 2u;
+    }
+    for (var reading = 0u; reading < readings; reading = reading + 1u) {
+        var tree_table_base = tt.base;
+        var root_pointer_offset = inode_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
+        var table = D_LEAF_IDENTITIES_OF_THE_VERSION;
+        if (reading == 1u) {
+            tree_table_base = parent.tree_table.base;
+            root_pointer_offset = parent_inode_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
+            table = D_LEAF_IDENTITIES_OF_THE_PARENT_VERSION;
+        }
+        leaf_identity_counts[reading] = d_read_inode_leaf_identities(tree_table_base, root_pointer_offset, table);
+    }
+    let leaf_identity_count = leaf_identity_counts.x;
+    let parent_leaf_identity_count = leaf_identity_counts.y;
+    if (leaf_identity_count == NONE) {
+        return;
+    }
+    // I-9.34（walk.rs judge_retirement_records_on_the_lineage_name_no_live_leaf）：谱系上的退役记录指认的身份不在这一版那棵树的叶身份里
+    if (judges_no_live_leaf) {
+        for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
+            let base = d_container_base(container_index);
+            for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
+                let o = d_container_record_offset(container_index, record_index, DELETION_RETIREMENT_RECORD_BYTES);
+                if (d_txg_is_on_the_lineage(rd64(base, o + DELETION_RETIREMENT_RECORD_DEATH_TXG_OFFSET), version_txg)
+                    && d_leaf_identity_table_holds(D_LEAF_IDENTITIES_OF_THE_VERSION, leaf_identity_count, d_container_identity_at(base, o))) {
+                    violate(INVARIANT_I_9_34);
+                }
+            }
+        }
+    }
+    if (!judges_completeness || parent_leaf_identity_count == NONE) {
+        return;
+    }
+    // 漏记：父版有、这一版没有，又没有死于这一版的退役记录
+    for (var parent_row = 0u; parent_row < parent_leaf_identity_count; parent_row = parent_row + 1u) {
+        let gone = d_leaf_identity_at(D_LEAF_IDENTITIES_OF_THE_PARENT_VERSION, parent_row);
+        if (!d_leaf_identity_table_holds(D_LEAF_IDENTITIES_OF_THE_VERSION, leaf_identity_count, gone)
+            && !d_retired_at(gone, version_txg)) {
+            violate(INVARIANT_I_9_35);
+        }
+    }
+    // 多记：死于这一版的退役记录指认的身份要在父版里、不在这一版里
+    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
+        let base = d_container_base(container_index);
+        for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
+            let o = d_container_record_offset(container_index, record_index, DELETION_RETIREMENT_RECORD_BYTES);
+            if (!eq64(rd64(base, o + DELETION_RETIREMENT_RECORD_DEATH_TXG_OFFSET), version_txg)) {
+                continue;
+            }
+            let retired = d_container_identity_at(base, o);
+            if (!d_leaf_identity_table_holds(D_LEAF_IDENTITIES_OF_THE_PARENT_VERSION, parent_leaf_identity_count, retired)
+                || d_leaf_identity_table_holds(D_LEAF_IDENTITIES_OF_THE_VERSION, leaf_identity_count, retired)) {
+                violate(INVARIANT_I_9_35);
+            }
+        }
+    }
+}
+
 // walk.rs judge_tombstones_and_the_lineage_of_a_version 的一版：树表指针在 (base, tree_table_pointer_offset)，这一版的 txg 是 version_txg。
-// 指针全零、树表 0 条的不判；树表读不出、条目宽不到 200 的不判
-fn d_judge_tombstones_and_the_lineage_of_a_version(base: u32, tree_table_pointer_offset: u32, version_txg: vec2<u32>) {
+// 指针全零、树表 0 条的不判；树表读不出、条目宽不到 200 的不判。is_the_newest_root：这一版是最新的根，顺带判 I-9.30 的 ② ④
+// （walk.rs 在 judge_inode_number_watermark_steps_of_the_newest_root 里另读一遍最新根；这里借这一遍已经读进来的记录、墓碑与谱系，不多一个调用点）
+fn d_judge_tombstones_and_the_lineage_of_a_version(base: u32, tree_table_pointer_offset: u32, version_txg: vec2<u32>, is_the_newest_root: bool) {
     let tree_table_pointer = parse_node_pointer(base, tree_table_pointer_offset);
     if (tree_table_pointer.all_zero) {
         return;
@@ -3098,8 +3762,13 @@ fn d_judge_tombstones_and_the_lineage_of_a_version(base: u32, tree_table_pointer
     if (!tt.ok || tt.entry_count == 0u || tt.entry_width < DELETION_TREE_TABLE_ENTRY_BYTES) {
         return;
     }
-    let lineage_holds = d_read_the_lineage_of_a_version(tt);
-    // 每棵读得出的 inode 树的每条记录记进 I-9.29 那张表
+    let lineage_holds = d_read_the_lineage_of_a_version(tt, version_txg);
+    let parent = d_parent_version_to_compare(tt, version_txg);
+    let judges_the_inode_number_watermark_steps = is_the_newest_root && d_inode_number_watermark_steps_are_judged;
+    if (judges_the_inode_number_watermark_steps && lineage_holds) {
+        d_judge_the_lineage_against_the_inode_number_watermark_steps();
+    }
+    // 每棵读得出的 inode 树的每条记录记进 I-9.29 那张表（最新根那一版另判 I-9.30 ②）
     for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
         let entry = entry_offset(tt, entry_index);
         if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) != TREE_KIND_INODE
@@ -3110,14 +3779,35 @@ fn d_judge_tombstones_and_the_lineage_of_a_version(base: u32, tree_table_pointer
         for (var record_index = 0u; record_index < d_inode_record_count; record_index = record_index + 1u) {
             let row = D_INODE_RECORDS + record_index * D_INODE_RECORD_WORDS;
             d_note_object_birth(inode_tree, vec2<u32>(sw(row), sw(row + 1u)), d_inode_record_object_birth(row));
+            if (judges_the_inode_number_watermark_steps
+                && d_number_was_issued_before_a_step(vec2<u32>(sw(row), sw(row + 1u)), d_inode_record_object_birth(row))) {
+                violate(INVARIANT_I_9_30);
+            }
         }
     }
-    // 每棵墓碑树：读不全的不判；只要墓碑的那几判先判，再经头 ID 配恰好一棵 inode 树判墓碑对记录的那几判
+    // 每棵墓碑树与退役树（容器经同一处读）：读不全的不判。墓碑树只要墓碑的那几判先判，再经头 ID 配恰好一棵 inode 树判墓碑对记录的那几判；
+    // 退役树与「父版要比、表里同 ID 的最后一条、又没有退役树配它」的每棵 inode 树（死于这一版的退役集合是空集）交给
+    // d_judge_the_retirement_records_and_their_completeness（一个调用点）
     for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
         let entry = entry_offset(tt, entry_index);
-        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) != TREE_KIND_TOMBSTONE
+        let kind = rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET);
+        let is_a_retirement_tree = kind == TREE_KIND_RETIREMENT;
+        if (kind == TREE_KIND_INODE) {
+            let inode_tree = rd64(tt.base, entry);
+            if (!parent.is_judged
+                || d_last_tree_table_entry_of_kind_keyed_by(tt, TREE_KIND_INODE, 0u, inode_tree) != entry
+                || d_last_tree_table_entry_of_kind_keyed_by(tt, TREE_KIND_RETIREMENT, TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET, inode_tree) != NONE) {
+                continue;
+            }
+            d_container_count = 0u;
+        } else if ((kind != TREE_KIND_TOMBSTONE && !is_a_retirement_tree)
             || !d_read_containers_of_a_container_tree(parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET),
-                PACKED_TYPE_TOMBSTONE, DELETION_TOMBSTONE_RECORD_BYTES)) {
+                select(PACKED_TYPE_TOMBSTONE, PACKED_TYPE_RETIREMENT, is_a_retirement_tree),
+                select(DELETION_TOMBSTONE_RECORD_BYTES, DELETION_RETIREMENT_RECORD_BYTES, is_a_retirement_tree))) {
+            continue;
+        }
+        if (kind != TREE_KIND_TOMBSTONE) {
+            d_judge_the_retirement_records_and_their_completeness(tt, entry, version_txg, lineage_holds, parent);
             continue;
         }
         d_judge_tombstone_container_capacity();
@@ -3125,6 +3815,20 @@ fn d_judge_tombstones_and_the_lineage_of_a_version(base: u32, tree_table_pointer
         if (lineage_holds) {
             d_judge_tombstones_on_the_lineage_are_distinct(version_txg);
         }
+        // I-9.30 ②：最新根谱系上的墓碑
+        if (judges_the_inode_number_watermark_steps && lineage_holds) {
+            for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
+                let container_base = d_container_base(container_index);
+                for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
+                    let o = d_container_record_offset(container_index, record_index, DELETION_TOMBSTONE_RECORD_BYTES);
+                    if (d_txg_is_on_the_lineage(rd64(container_base, o + DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET), version_txg)
+                        && d_number_was_issued_before_a_step(rd64(container_base, o + DELETION_TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET),
+                            rd64(container_base, o + DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET))) {
+                        violate(INVARIANT_I_9_30);
+                    }
+                }
+            }
+        }
         let head_identifier = rd64(tt.base, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
         var paired_inode_tree_count = 0u;
         var paired_inode_tree_entry = 0u;
@@ -3160,17 +3864,19 @@ fn judge_tombstones_and_the_lineage_of_the_versions() {
         var base = 0u;
         var tree_table_offset = 36u;
         var txg = vec2<u32>(0u, 0u);
+        var is_the_newest_root = false;
         if (version_index < c_candidate_count) {
             let root = checker_root_of(sw(C_CANDIDATES + version_index));
             base = root.base;
             txg = root.txg;
+            is_the_newest_root = sw(C_CANDIDATES + version_index) == c_newest_index;
         } else {
             let record = checker_record_at(sw(R_DATA + (version_index - c_candidate_count)));
             base = record.base;
             tree_table_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
             txg = record.txg;
         }
-        d_judge_tombstones_and_the_lineage_of_a_version(base, tree_table_offset, txg);
+        d_judge_tombstones_and_the_lineage_of_a_version(base, tree_table_offset, txg, is_the_newest_root);
     }
 }
 
@@ -3182,6 +3888,7 @@ fn scan_stage(fsid_low: vec2<u32>) {
     judge_allocation_generations_against_unit_births();
     judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark();
     judge_deletion_state_of_the_versions();
+    judge_inode_number_watermark_steps_against_the_watermark_of_the_newest_root();
     judge_tombstones_and_the_lineage_of_the_versions();
     judge_inode_number_watermark_above_every_published_tombstone();
     judge_rollback_floor_raises_against_their_ceilings();
diff --git a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
index 56bd5be..9de93f9 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -135,8 +135,10 @@ const TREE_KIND_DELETED_INODES: u32 = 9u;
 const TREE_KIND_TOMBSTONE: u32 = 10u;
 const TREE_KIND_LINEAGE: u32 = 11u;
 const TREE_KIND_LOGGED_OPERATIONS: u32 = 12u;
+// 退役树（recovery.rs 与 walk.rs 的 TREE_KIND_RETIREMENT，判决 m3-retire-r1 第三节第 1 条）：码 13，排在 logged_ops 之后
+const TREE_KIND_RETIREMENT: u32 = 13u;
 // 进树表的种类有几个（中央映射树 5 不进树表）：I-9.16 ② 按发号次序数这几个，见 tree_kind_in_the_issuing_order
-const TREE_KINDS_OF_THE_TREE_TABLE: u32 = 11u;
+const TREE_KINDS_OF_THE_TREE_TABLE: u32 = 12u;
 // deleted_inodes 根的 key 是 inode 号 8 字节；墓碑树、谱系树码 2 节点的 key 是容器号 8 字节（recovery.rs 的 key_width_for_kind）
 const DELETED_INODES_KEY_WIDTH: u32 = 8u;
 const CONTAINER_NUMBER_KEY_WIDTH: u32 = 8u;
@@ -2922,10 +2924,10 @@ fn tree_kind_key_width(kind: u32) -> u32 {
     if (kind == TREE_KIND_ALLOCATION) { return 10u; }
     if (kind == TREE_KIND_ACCOUNTING) { return 22u; }
     if (kind == TREE_KIND_DELETED_INODES) { return DELETED_INODES_KEY_WIDTH; }
-    if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE) { return CONTAINER_NUMBER_KEY_WIDTH; }
+    if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) { return CONTAINER_NUMBER_KEY_WIDTH; }
     return NONE;
 }
-// 进树表的第 position 个种类（发号次序，从 0 数）：1、2、3、4，跳过不进树表的 5，再 6..12
+// 进树表的第 position 个种类（发号次序，从 0 数）：1、2、3、4，跳过不进树表的 5，再 6..13
 fn tree_kind_in_the_issuing_order(position: u32) -> u32 {
     if (position < 4u) {
         return position + 1u;
@@ -3124,7 +3126,7 @@ fn walk_to_file(root: RootFields, cfg: SysConfig, expected: u32) -> WalkResult {
             return walk_failed(25u);
         }
     }
-    // I-9.16 ②：按发号次序 1,2,3,4,6,7,8,9,10,11,12（recovery.rs 的 TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER）相邻两棵严格升序
+    // I-9.16 ②：按发号次序 1,2,3,4,6,7,8,9,10,11,12,13（recovery.rs 的 TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER）相邻两棵严格升序
     var previous_tree = vec2<u32>(0u, 0u);
     var have_previous = false;
     for (var k = 0u; k < TREE_KINDS_OF_THE_TREE_TABLE; k = k + 1u) {
@@ -3199,8 +3201,8 @@ fn walk_to_file(root: RootFields, cfg: SysConfig, expected: u32) -> WalkResult {
             have_accounting = true;
             continue;
         }
-        // 墓碑树与谱系树：从根往下读码 2 节点、数进映射的节点与容器，不读容器（它们的内容不影响读回的文件）
-        if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE) {
+        // 墓碑树、谱系树与退役树：从根往下读码 2 节点、数进映射的节点与容器，不读容器（它们的内容不影响读回的文件）
+        if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) {
             let read = read_container_tree_index_nodes(pointer, tree_table.base, o + 14u, tree, root, fsid_low);
             if (!read.ok) {
                 return walk_failed(62u);
@@ -3293,8 +3295,20 @@ fn walk_to_file(root: RootFields, cfg: SysConfig, expected: u32) -> WalkResult {
     if (devices_with_records != device_count || first_device_count < 10u) {
         return walk_failed(39u);
     }
-    // 记账条目数 3 + 6 × 盘数
-    if (accounting_count != 3u + 6u * device_count) {
+    // 记账条目数 3 + 6 × 盘数，inode 号水位的台阶行不算（recovery.rs accounting_entry_count_is_three_plus_six_per_device：
+    // 标签 12、设备段 0xFFFF_FFFF、代段早于根的 txg 的行；条目宽不到 34 的照算，后面判宽那一道拒它）
+    var accounting_entries_other_than_the_inode_number_watermark_steps = 0u;
+    for (var a = 0u; a < accounting_count; a = a + 1u) {
+        let base = sw(R_ACCOUNTING + a * 3u);
+        let o = sw(R_ACCOUNTING + a * 3u + 1u);
+        let width = sw(R_ACCOUNTING + a * 3u + 2u);
+        let is_an_inode_number_watermark_step = width >= 34u && rd16(base, o) == 12u && rd32(base, o + 10u) == NONE
+            && lt64(rd64(base, o + 14u), root.txg);
+        if (!is_an_inode_number_watermark_step) {
+            accounting_entries_other_than_the_inode_number_watermark_steps = accounting_entries_other_than_the_inode_number_watermark_steps + 1u;
+        }
+    }
+    if (accounting_entries_other_than_the_inode_number_watermark_steps != 3u + 6u * device_count) {
         return walk_failed(40u);
     }
     // 映射条目数 = 1 + extent 节点 + 分配记录树节点 + 记账树节点 + inode 叶容器数 + 数据单元数 + 删除那两棵树与谱系树的节点
diff --git a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
index e14f8f5..f516071 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -1340,24 +1340,29 @@ mod tests {
 
     /// 三个内核此刻展开之后的表达式数与离预算的余量（2026-10-08 在删除线第 39 批的仓副本上用 naga =30.0.1 现量的：恢复内核的 F2 改成 F2_NEWEST 那一形
     /// （退到 [见证到的那一版, 所选根) 里最新、不被所选根实例表抛弃的根，沿链读所选根的实例表），恢复 318 224 → 328 934，扫描与走读没变；删除线第 41 批 F2 对 c_见证 = 0 加守卫，恢复 328 934 → 328 970；删除线第 45 批 F2 不越过一次末条读得出、根读不出的发布（退到的那一条与所选根之间逐条记录、
-    /// 逐条根比一遍），恢复 328 970 → 331 792；
+    /// 逐条根比一遍），恢复 328 970 → 331 792；删除线第 49 批 W3（恢复读记账条目数不算 inode 号水位的台阶行、走读内核 I-3.13 跳过台阶行、
+    /// 扫描内核加 I-9.30 与谱系记录宽 24）恢复 331 792 → 332 638、走读 346 138 → 346 307、扫描 352 195 → 366 870（第 49 批实现员副本上用 naga =30.0.1
+    /// 照 `kernel_inlined_size` 同一个数法量的：草稿工具在旧源码上量回 331 792 / 346 138 / 352 195，与旧钉值逐个相等，三个内核都解析、校验得过）；
+    /// 删除线第 50 批退役树（恢复与走读认种类 13、走读内核判 I-9.31，扫描内核判 I-9.32–I-9.35：退役容器借墓碑树那一处读容器的调用点、叶身份只有一个读的调用点）
+    /// 恢复 332 638 → 332 666、走读 346 307 → 346 278、扫描 366 870 → 387 154（第 50 批补做实现员副本上用同一个草稿工具量的，它在主工作区那一版上量回
+    /// 331 792 / 346 138 / 352 195；四条一开始各留调用点时扫描量到 454 049、超预算，改成单调用点才落回预算里）；
     /// 第 36 批之前是恢复 320 015、扫描 382 259、走读 337 872）：
     /// 改了哪个内核的源码，这里先红，量过新数、确认余量还够之后再改钉值，免得余量悄悄耗光、等上卡时才撞预算。
     #[test]
     fn each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget() {
         for (name, source, pinned_expressions, pinned_margin) in [
-            ("recovery", recovery_kernel_source(), 331_792, 68_208),
+            ("recovery", recovery_kernel_source(), 332_666, 67_334),
             (
                 "checker-scan",
                 crate::crash_judge_gpu::checker_scan_kernel_source(),
-                352_195,
-                47_805,
+                387_154,
+                12_846,
             ),
             (
                 "checker-walk",
                 crate::crash_judge_gpu::checker_walk_kernel_source(),
-                346_138,
-                53_862,
+                346_278,
+                53_722,
             ),
         ] {
             let size = super::kernel_inlined_size(&source);
diff --git a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
index 8ef2531..6c85764 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -60,7 +60,7 @@ const _: () = assert!(
     "违例位读成一个 u128"
 );
 
-/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 84 条——三批 49 条，加里程碑三二·1 那一批 16 条
+/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 90 条——三批 49 条，加里程碑三二·1 那一批 16 条
 /// （判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行），下标 65 的 I-3.5（引用区间的精确性，
 /// 走读内核在回退候选集的版本都走完之后逐对判），再加 C673（三 oracle 合谋测试的补 oracle 改法没做）⑤ ⑥ 那两条
 /// （下标 66 是 I-9.27，走读内核挨着 I-9.16 按表判；下标 67 是 I-5.5，扫描内核每一版重读树表与 extent 树判），
@@ -80,12 +80,21 @@ const _: () = assert!(
 /// GPU 也没有旋钮值。I-9.4 第四句（同一片 inode 叶容器里记录号严格递增）走读内核一并判、并进下标 40。
 /// 下标 83 是根环线第 10j 批新立的 I-3.14（实例表每片记录形态合法）：走读内核在实例表链走到的每一片上判四格（kind 只有 0 或 1、
 /// 链指针记录只在最后一条、行的 flags 为 0、最后一条是合法的链指针记录），不成立的一片照 CPU 当解不开——行不收、链不往下走（第 10k 批）。
+/// 下标 84 是第 49 批新立的 I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上）：扫描内核先读最新根的记账树收 inode 号水位那一统计量的行、
+/// 判 ①（水位不小于每一级台阶），再在墓碑与谱系那一遍走到最新根那一版时判 ②（inode 记录与谱系上的墓碑）与 ④（谱系记录带的 H_cur）；
+/// 读不全的那一部分不置位（CPU 报射程里判不了），表越过上限判不了。
+/// 下标 85–89 是第 50 批退役树那五条（I-9.31–I-9.35，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条）：
+/// 走读内核在退役树的每条内部条目上判 I-9.31（与 I-9.17、I-9.23 同一个走法，类型段 {0, 3}）；扫描内核在墓碑与谱系那一遍的每一版上判
+/// I-9.32、I-9.33、I-9.34（后两条只判死亡 txg 在这一版谱系上的，I-9.34 读头 ID 配上的那棵 inode 树内部条目里的叶身份），扫完各版再跨相邻两版判
+/// I-9.35（父版照 walk.rs 取、在版本清单里恰好一版才比，最新根的实例表沿链读不出时整条不比）。判决同一条原写「其余回落 CPU」，
+/// 用户 2026-09-29 定「配了 GPU 就一定要 GPU 判，不退回 CPU」，第 50 批补做把这四条搬上 GPU、不记判不了；读不全的那一部分不置位（CPU 报射程里判不了），
+/// 叶身份表与两两比的退役记录越过上限判不了。
 /// 用户 2026-10-04 定「改回要求全覆盖」：没有「内核还没判」那一张表。
 pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
     26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
     50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
-    74, 75, 76, 77, 78, 79, 80, 81, 82, 83,
+    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89,
 ];
 
 // 覆盖表就是池级 checker 的每一条，按下标升序：第 k 项是 k。新立一条不变量而内核没跟上时这里编译不过，不悄悄让 GPU 那一位恒 0。
@@ -130,8 +139,9 @@ pub fn verdict_layout_wgsl_constants() -> String {
 }
 /// 每个状态的草稿区几个 u32（checker 共用那一半里 `SCRATCH_END`：共用的 `SCRATCH_WORDS_REQUIRED`、走读与扫描的集合、两个内核的交接段、
 /// 交接段之后走读内核自己用的表：两张，加 I-3.5 的出现规则读的四张；再之后是扫描内核删除态扫描与墓碑、谱系那一遍用的五张：inode 记录、每个号的数据单元数、
-/// 一棵墓碑树或谱系树的容器、一版的谱系记录、I-9.29 跨版本的号 → 对象出生代）。
-pub const SCRATCH_WORDS: usize = 315_376;
+/// 一棵墓碑树或谱系树的容器、一版的谱系记录、I-9.29 跨版本的号 → 对象出生代，与 I-9.30 读的最新根记账里 inode 号水位那一统计量的行，
+/// 再加退役那几条读的两张叶身份表：被判的那一版与 I-9.35 的父版）。
+pub const SCRATCH_WORDS: usize = 320_688;
 /// 内核的工作组大小（三个内核都是 `@workgroup_size(64)`）：派 n 个状态要 ⌈n ÷ 64⌉ 个工作组。
 pub const WORKGROUP_SIZE: usize = 64;
 /// 内核的四个绑定（`crash_judge_common.wgsl` 开头）：派活参数、打包的表、结论、草稿区。
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
index 0886398..4d209e6 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
@@ -95,7 +95,7 @@ fn checker_line(tally: &Layer0Tally) -> String {
 }
 
 /// checker 与记录核对器那一半的钉死值。只在根槽 txg 3 已持久的状态上才走得到记账树、inode 树、分配记录树与中央映射树的条目，
-/// 那 25 条只在这些状态上评估（删除态扫描那四条 I-9.5、I-3.4、I-9.24、I-9.29 的理由写在函数里那张表旁边）
+/// 那 26 条只在这些状态上评估（删除态扫描那四条 I-9.5、I-3.4、I-9.24、I-9.29 的理由写在函数里那张表旁边）
 /// （I-9.25（树表条目的 previous_snapshot_txg）、I-9.27（树表条目的头 ID）、I-3.13（记账行的代段）、I-9.26（inode 记录的改动计数）、
 /// I-5.5（从 mkfs 水位建树那一版的第一个数据单元） 与 I-3.5（引用区间的精确性） 是这个理由：种子根与暖机根指着的第 0 版树表是空的，
 /// 只有 txg 3 那一版有树表条目、记账树、inode 记录与数据单元；记录施加出来的那一版由 I-4.1 另起一份走读去走、判定丢掉，不进这几条的计数，
@@ -168,6 +168,9 @@ fn assert_checker_counts(
         "I-3.4",
         DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
         ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
+        // I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，第 49 批）只在最新根下面有走全了的记账树时判：txg 3 那一版自己写的那一行
+        // 就是第一级台阶 (3, 2)，① 判得到、成立；它的谱系树、墓碑树都是零根。种子根与暖机根没有记账树，报不适用。推的，没在 checker 档跑过。
+        "I-9.30",
     ];
     // I-3.10 除了回退候选集，还读下一次挂载会先施加的那一版（射程 ④）：新根没落而 txg 3 的记录已落时，
     // 恢复要由那条记录重建 txg 3 那一版，它的分配记录树在记录的新根段指着的树表下面。
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 2877127..faa50e2 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -79,6 +79,8 @@
 //! 照卡的先后装满前一张才用下一张，放不下就等别的用例还，比每张卡的账都大判红（几十条用例并行不再各按空闲显存的 3/4 互抢）；
 //! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（默认 8）、显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（量速度那条用例当那张卡的额度；卡池里的读法在 `gpu_unit_checks::gpu_test_start::CARD_QUOTA_ENVIRONMENT_VARIABLE`）、
 //! CPU 判器线程数 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`（没设取本机可用并行度）：设了而认不出（线程数 0 也算）就 panic，不退回没设时的那一档。
+//! 第 50 批（退役树）：退役记录那四条 GPU 同判、不记判不了——删完的流（最后一批开出退役树、一条退役记录）上退役记录死亡 txg 加一（I-9.32、I-9.35）、
+//! 同一条再追加一条（I-9.33）、死亡 txg 减一（只 I-9.35 漏记），左吸收右的流上退役记录改指幸存的左片（I-9.34、I-9.35）各一条流，都整份比。
 #![cfg(all(feature = "verdict-store", feature = "gpu"))]
 
 #[path = "../../singlefs-harness/tests/common/mod.rs"]
@@ -4441,8 +4443,8 @@ fn every_tree_table_the_new_pool_and_the_rollback_flows_write_fits_the_tree_coun
     }
 }
 
-/// 回退流上回退那次发布 D 往谱系树追加的那一片容器（码 3、记录类型 5）：记录宽从 16 改成 17、重封到根。
-/// CPU 池级 checker 走谱系树走到这片容器、按 I-1.7 判它的记录宽不是登记的 16 → 红；恢复不读容器，两遍恢复照常读到文件。
+/// 回退流上回退那次发布 D 往谱系树追加的那一片容器（码 3、记录类型 5）：记录宽从 24 改成 25、重封到根。
+/// CPU 池级 checker 走谱系树走到这片容器、按 I-1.7 判它的记录宽不是登记的 24 → 红；恢复不读容器，两遍恢复照常读到文件。
 /// GPU 走读内核不走谱系树、或不认谱系记录的登记宽时这一位对不上。
 fn record_rollback_with_the_lineage_container_record_width_off_its_registered_width(
     tag: &str,
@@ -4461,7 +4463,7 @@ fn record_rollback_with_the_lineage_container_record_width_off_its_registered_wi
                     .try_into()
                     .expect("2"),
             );
-            assert_eq!(width, 16, "谱系记录宽 16");
+            assert_eq!(width, 24, "谱系记录宽 24");
             bytes[PACKED_RECORD_WIDTH_OFFSET..PACKED_RECORD_WIDTH_OFFSET + 2]
                 .copy_from_slice(&(width + 1).to_le_bytes());
         },
@@ -4480,6 +4482,47 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_of_a_rollback_pu
     );
 }
 
+/// 回退流上回退那次发布 D 往谱系树追加的那一片容器（码 3、记录类型 5、记录宽 24）：第一条谱系记录的 H_cur（偏移 16，C 的 inode 号水位 2）
+/// 改成 3、重封到根。落在 D 上的状态 CPU 池级 checker 判 I-9.30 ④：D 记账里 T_rb − 1 以前最大的台阶是 A 留的 (3, 2)，不等于 3 → 红；
+/// 恢复不读谱系记录的内容，两遍恢复照常读到文件。GPU 扫描内核不读 H_cur、不判 I-9.30 ④，或谱系记录宽仍按 16 读时这一位对不上。
+fn record_rollback_with_the_lineage_record_carrying_another_watermark_rolled_back_from(
+    tag: &str,
+) -> RecordedFlow {
+    const LINEAGE_RECORD_TYPE: u16 = 5;
+    const PACKED_UNIT_FIRST_RECORD_OFFSET: usize = 136;
+    const LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET: usize = 16;
+    let lineage_container =
+        |bytes: &[u8]| unit_class(bytes) == 3 && packed_record_type(bytes) == LINEAGE_RECORD_TYPE;
+    record_rollback_with_a_resealed_field(
+        tag,
+        "rollback-lineage-record-watermark-rolled-back-from-three",
+        &lineage_container,
+        &|bytes| {
+            let offset = PACKED_UNIT_FIRST_RECORD_OFFSET
+                + LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET;
+            let watermark_rolled_back_from =
+                u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8"));
+            assert_eq!(
+                watermark_rolled_back_from, 2,
+                "D 的谱系记录带 C 的 inode 号水位 2"
+            );
+            bytes[offset..offset + 8].copy_from_slice(&3u64.to_le_bytes());
+        },
+    )
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from",
+        &record_rollback_with_the_lineage_record_carrying_another_watermark_rolled_back_from(
+            "crash-judge-gpu-lineage-watermark-rolled-back-from",
+        ),
+        "I-9.30",
+    );
+}
+
 /// 回退流上谱系树的根（码 2、key 宽 8、条目宽 120、第一条条目的类型段是谱系记录 5）：头里的树 ID 加 1、重封到根。
 /// 落在 D 上的状态，CPU 恢复读谱系树根时核自描述（树 ID 不是树表条目里那棵）→ 走读失败、读不到文件；CPU 池级 checker 读这个根判 I-1.3 → 红。
 /// GPU 恢复内核不读谱系树根、或读了不核树 ID，GPU 走读内核不读谱系树根时，恢复或这一位对不上。
@@ -4637,8 +4680,8 @@ const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET: usize = 14;
 /// 稀疏旁表与 deadlist 的种类码（D18（块里携带什么信息） 已定项 24 第 6 条那一组）：第一版两棵树都不建根，树表条目的根指针全零，走读与恢复都不读。
 const TREE_KIND_SPARSE_SIDE_TABLE: u16 = 7;
 const TREE_KIND_DEADLIST: u16 = 8;
-/// 一个没登记的种类码：走读与恢复都不认它、不读它的根。
-const UNREGISTERED_TREE_KIND: u16 = 13;
+/// 一个没登记的种类码：走读与恢复都不认它、不读它的根（13 第 50 批登记给了退役树，改用 14）。
+const UNREGISTERED_TREE_KIND: u16 = 14;
 /// 根记录里树表指针的第 0 条位置条目（盘 4 + 槽 6）的偏移：树表指针在 36，位置条目在指针偏移 50。
 const ROOT_RECORD_TREE_TABLE_POINTER_FIRST_LOCATION_OFFSET: usize = 86;
 /// 根记录里「树表 0 条那一版的分配记录树根」指针的偏移（D22（单元原子性怎么合成） 已定项 7 的字段表）。
@@ -6652,6 +6695,8 @@ fn record_two_empty_inodes_unlinked_with_one_tombstone_per_container(tag: &str)
             .expect("1..=815"),
         ),
         lineage_container: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     };
     for inode in [FIRST_FILE.0 + 1, FIRST_FILE.0 + 2] {
@@ -6706,6 +6751,8 @@ fn capacities_of_one_tombstone_per_container_and_two_entries_per_node() -> Conta
             .expect("1..=815"),
         ),
         lineage_container: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         node_entries: ContainerTreeNodeEntries::CappedForTests(
             ContainerTreeNodeEntriesCappedForTests::new(
                 CONTAINER_TREE_ENTRIES_PER_NODE_UNDER_THE_KNOB,
@@ -8028,6 +8075,181 @@ fn record_new_inode_unlinked_rolled_back_and_unlinked_again(tag: &str) -> Record
     )
 }
 
+/// 退役记录 34 字节（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2 条）：被退役身份的出生树 0、打包记录类型 8、容器号 10、
+/// 容器出生代 18、死亡 txg 26；退役容器是码 3、打包记录类型 3。
+const RETIREMENT_RECORD_BYTES: usize = 34;
+const RETIREMENT_RECORD_CONTAINER_OFFSET: usize = 10;
+const RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET: usize = 18;
+const RETIREMENT_RECORD_DEATH_TXG_OFFSET: usize = 26;
+const PACKED_RECORD_TYPE_RETIREMENT: u16 = 3;
+/// 一片 inode 叶容器装 233 条（`singlefs_format::INODE_LEAF_RECORDS`）：第一个文件之外再建这么多个空 inode，叶就是满的一片加一条的一片。
+const INODE_RECORDS_PER_LEAF: u64 = 233;
+
+fn is_a_retirement_container(bytes: &[u8]) -> bool {
+    unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_RETIREMENT
+}
+
+/// 退役容器第一条退役记录偏移 `field` 起的 8 字节改成 `change` 给的值。
+fn change_the_first_retirement_record_field(
+    bytes: &mut [u8],
+    field: usize,
+    change: impl Fn(u64) -> u64,
+) {
+    assert!(
+        read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET) >= 1,
+        "退役容器里至少一条退役记录"
+    );
+    let offset = PACKED_UNIT_RECORDS_START + field;
+    let value = change(read_u64_at(bytes, offset));
+    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
+}
+
+/// 删完的流（[`record_new_pool_through_the_last_deletion_batch`]：最后一批删掉唯一那片 inode 叶，同一次发布开出退役树、写一条退役记录，
+/// 死亡 txg = 最后一批那次的 txg 8）上改坏那片退役容器（两份副本）的一处、重封到根。
+fn record_retirement_with_a_resealed_field(
+    tag: &str,
+    name: &'static str,
+    mutate: &dyn Fn(&mut [u8]),
+) -> RecordedFlow {
+    let mut flow = record_new_pool_through_the_last_deletion_batch(tag, name);
+    let targets = last_unit_copies(&flow, name, &is_a_retirement_container, false);
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, mutate);
+    flow
+}
+
+/// I-9.32 与 I-9.35：删完的流上那条退役记录的死亡 txg 8 → 9、重封到根。死亡 txg 大于这一版的 txg 8：CPU 判 I-9.32 红；
+/// 死于这一版的退役记录一条都没有，父版（txg 7）那片叶的身份没记：I-9.35 漏记红。
+fn record_retirement_dying_after_its_version(tag: &str) -> RecordedFlow {
+    record_retirement_with_a_resealed_field(tag, "retirement-dies-after-its-version", &|bytes| {
+        change_the_first_retirement_record_field(
+            bytes,
+            RETIREMENT_RECORD_DEATH_TXG_OFFSET,
+            |death_txg| death_txg + 1,
+        );
+    })
+}
+
+/// I-9.33：删完的流上那条退役记录原样再追加一条（记录数加一、声明长度加 34）、重封到根。两条都在谱系上、身份相同：CPU 判 I-9.33 红；
+/// 死于这一版的身份集照旧是那一片，I-9.35 照旧成立。
+fn record_retirement_written_twice(tag: &str) -> RecordedFlow {
+    record_retirement_with_a_resealed_field(tag, "retirement-written-twice", &|bytes| {
+        let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
+        assert_eq!(record_count, 1, "删到底那一批只写一条退役记录");
+        let first = PACKED_UNIT_RECORDS_START;
+        bytes.copy_within(
+            first..first + RETIREMENT_RECORD_BYTES,
+            first + RETIREMENT_RECORD_BYTES,
+        );
+        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2]
+            .copy_from_slice(&2u16.to_le_bytes());
+        let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
+        bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
+            &(declared_length + u16::try_from(RETIREMENT_RECORD_BYTES).expect("34")).to_le_bytes(),
+        );
+    })
+}
+
+/// I-9.35 漏记那一向：删完的流上那条退役记录的死亡 txg 8 → 7、重封到根。容器出生代 3 < 7 ≤ 8、死于 7 也在谱系上，I-9.32、I-9.33 照旧成立；
+/// 死于这一版（8）的退役记录一条都没有，父版（txg 7）那片叶的身份没记：CPU 只红 I-9.35。
+fn record_retirement_dying_at_the_parent_version(tag: &str) -> RecordedFlow {
+    record_retirement_with_a_resealed_field(
+        tag,
+        "retirement-dies-at-the-parent-version",
+        &|bytes| {
+            change_the_first_retirement_record_field(
+                bytes,
+                RETIREMENT_RECORD_DEATH_TXG_OFFSET,
+                |death_txg| death_txg - 1,
+            );
+        },
+    )
+}
+
+/// I-9.34 与 I-9.35：新池新建文件之外再建 233 个空 inode（叶装成满的一片 [1..=233] 与一条的一片 [234]），unlink 2 号到底：最后一批左片吸收右片、
+/// 右片身份消失，写一条 (右片身份, 这次的 txg) 的退役记录。把这条退役记录的容器号、容器出生代改成幸存左片的、重封到根：它在谱系上、指着这一版的活叶，
+/// CPU 判 I-9.34 红；死于这一版的身份集成了左片、右片没记，I-9.35 漏记多记一起红。
+fn record_absorption_with_the_retirement_record_naming_the_surviving_left_leaf(
+    tag: &str,
+) -> RecordedFlow {
+    const NAME: &str = "absorption-retirement-record-names-the-surviving-left-leaf";
+    let mut pool = pool_with_new_inodes(tag, INODE_RECORDS_PER_LEAF);
+    let identities_before: Vec<_> = pool
+        .output
+        .inode_leaf_containers
+        .iter()
+        .map(|container| container.contents.identity)
+        .collect();
+    let [left, right] = identities_before[..] else {
+        panic!("234 条记录装成两片叶：{identities_before:?}");
+    };
+    unlink_to_the_end(&mut pool, FIRST_FILE.0 + 1);
+    let retired: Vec<_> = pool
+        .output
+        .retirement_containers
+        .iter()
+        .flat_map(|container| container.contents.records.iter())
+        .map(|record| record.retired)
+        .collect();
+    assert_eq!(retired, [right], "左吸收右：退役树里一条右片的退役记录");
+    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
+    let mut flow = recorded_flow_of_the_pool(&pool, NAME, versions);
+    let targets = last_unit_copies(&flow, NAME, &is_a_retirement_container, false);
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &move |bytes| {
+        change_the_first_retirement_record_field(
+            bytes,
+            RETIREMENT_RECORD_CONTAINER_OFFSET,
+            |_container| left.container,
+        );
+        change_the_first_retirement_record_field(
+            bytes,
+            RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET,
+            |_container_birth| left.container_birth.0,
+        );
+    });
+    flow
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_dies_after_its_version() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_dies_after_its_version",
+        &record_retirement_dying_after_its_version("crash-judge-gpu-i932-retirement-death-txg"),
+        &["I-9.32", "I-9.35"],
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_is_written_twice() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_is_written_twice",
+        &record_retirement_written_twice("crash-judge-gpu-i933-retirement-written-twice"),
+        "I-9.33",
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_names_the_surviving_left_leaf() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_names_the_surviving_left_leaf",
+        &record_absorption_with_the_retirement_record_naming_the_surviving_left_leaf(
+            "crash-judge-gpu-i934-retirement-names-a-live-leaf",
+        ),
+        &["I-9.34", "I-9.35"],
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_retirement_record_of_a_gone_leaf_dies_at_the_parent_version(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_retirement_record_of_a_gone_leaf_dies_at_the_parent_version",
+        &record_retirement_dying_at_the_parent_version(
+            "crash-judge-gpu-i935-retirement-death-txg-at-the-parent",
+        ),
+        "I-9.35",
+    );
+}
+
 /// 墓碑与谱系那九条（I-9.24 在删除态扫描里判；I-9.22 层 0 恒不适用）。
 const TOMBSTONE_AND_LINEAGE_INVARIANTS: [&str; 9] = [
     "I-9.17", "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23", "I-9.24", "I-9.29",
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index a56ef41..947f433 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -95,7 +95,15 @@ pub const SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY: &str =
 /// 第 84 条（下标 83）是 I-3.14（实例表每片记录形态合法，用户 2026-10-06 定另立，根环线第 10j 批），同样只往末尾追加；
 /// 主走读沿实例表链每一片判（`walk` 的 `judge_instance_table_page_record_forms` 与链指针记录那一格）。
 /// 不成立的那一片解不开：行不收、链不往下走（第 10k 批）。GPU 判器的走读内核同判（`crash_judge_gpu` 的覆盖表补了下标 83，第 10k 批）。
-pub const IMPLEMENTED_INVARIANTS: [&str; 84] = [
+///
+/// 第 85 条（下标 84）是 I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，W3 的 C2aL 形态，第 49 批），同样只往末尾追加；
+/// 在最新的根上判（`walk` 的 `judge_inode_number_watermark_steps_of_the_newest_root`，挨着 I-9.6）。GPU 判器的扫描内核同判（`crash_judge_gpu` 的覆盖表补了下标 84）。
+///
+/// 第 86–90 条（下标 85–89）是退役树那五条（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条，正推
+/// `research/prompts/m3-retire-r1-sonnet-output.md` K5.1，第 50 批）：I-9.31、I-9.32、I-9.33、I-9.34、I-9.35，同样只往末尾追加。
+/// I-9.31 在主走读里判（退役树的内部条目，与 I-9.17、I-9.23 同一个走法），其余在墓碑与谱系那一遍里每一版各判（I-9.35 跨相邻两版比）。
+/// GPU 判器同判 I-9.31，其余四条在有退役记录的状态上判不了（`crash_judge_gpu` 的覆盖表与它的文档）。
+pub const IMPLEMENTED_INVARIANTS: [&str; 90] = [
     "I-1.1",
     "I-1.2",
     "I-1.3",
@@ -180,6 +188,12 @@ pub const IMPLEMENTED_INVARIANTS: [&str; 84] = [
     DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
     ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
     INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
+    INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
+    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+    EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
+    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
+    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
 ];
 
 /// I-1.12：走读跟随的一条指针，偏移 34 的出生树等于被指单元头里的出生树——码 1、码 3 取头偏移 43；码 2 取哪个字段随 C289 定，
@@ -283,6 +297,29 @@ pub const ONE_OBJECT_BIRTH_PER_INODE_NUMBER: &str = "I-9.29";
 /// `kind` 0 行的 flags 字节为 0（D18（块里携带什么信息） 已定项 11：flags「第一版恒 0、非 0 拒收」）。行唯一与低于挂载根归 I-3.8。
 /// 不成立的一片解不开（用户 2026-10-06 给 D18 已定项 11 补「kind 不是 0 或 1 ⇒ 这一片解不开（表不可读）」）：主走读与只读不判的读法都不收它的行。
 pub const INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED: &str = "I-3.14";
+/// I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，拟定编号；W3 的 C2aL 形态，D5（快照 / 空间记账机制） 已定项 2 射程、
+/// D18（块里携带什么信息） 已定项 24 第 5 条）：最新根记账里 inode 号水位那一统计量的台阶行 (g, H)——①水位不小于每一级的 H；
+/// ② 最新根 inode 树里的记录与谱系上的墓碑 (号 n, 对象出生代 b) 没有一级 H > n 且 b > g；④ 每条谱系记录上，代段 ≤ T_rb − 1 的台阶里最大的 H
+/// 等于记录带的 H_cur，H_cur 为 0 也判红（正推草案 `research/prompts/m3-w3-r1-sonnet-output.md` 3.1 节，加第二轮攻方 C2aL 最强形的那一判）。
+pub const INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE: &str =
+    "I-9.30";
+
+/// I-9.31（退役树条目身份与子头相符）：与 I-9.17 同形，类型段 ∈ {0, 3}、子容器打包记录类型 3（判决
+/// `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条，逐字照 I-9.17、I-9.23）。
+pub const RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER: &str = "I-9.31";
+/// I-9.32（退役记录合法）：退役树里每条记录（不论在不在谱系上）被退役身份的打包记录类型 == 2、出生树 ≠ 0，容器出生代 < 死亡 txg ≤ 这一版的 txg；
+/// 每片退役容器的记录数 ≤ 959（正推 K5.1；记录宽 34 由 I-1.7 判）。
+pub const EVERY_RETIREMENT_RECORD_IS_WELL_FORMED: &str = "I-9.32";
+/// I-9.33（谱系上的退役记录按身份不重复）：死亡 txg 落在这一版父链上的退役记录，按（出生树、打包记录类型、容器号、容器出生代）不重复；
+/// 只判在谱系上的（不滤谱系会在「删 → 回退 → 再删」的合法历史上误报，判决第二节 K5 那一行两次抽样坐实）。
+pub const RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT: &str = "I-9.33";
+/// I-9.34（谱系上的退役记录不指本版活叶）：在谱系上的退役记录指认的身份，不出现在同一版、头 ID 配得上的那棵 inode 树的叶容器身份集合里
+/// （读内部条目的身份引用，不开叶）。
+pub const RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF: &str = "I-9.34";
+/// I-9.35（退役完备）：相邻两版（这一版的 txg 恰是这一版谱系树里某条记录的 T_rb 时父版是那条记录的 T_old，否则父版是 txg − 1；父版在回退候选集里
+/// 读得出才判）：父版叶身份集 − 本版叶身份集 = 本版谱系上、死亡 txg == 本版 txg 的退役记录的身份集，两个方向各报。最新根的实例表读不出、
+/// 滤不掉被崩溃恢复抛弃的根时报部分没判（照 I-9.29 的门）。被攻过零轮（判决第四节）。
+pub const RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION: &str = "I-9.35";
 
 /// 判定累加器：每条不变量记评估了几次、第一处违例、整条不适用的理由，以及射程里有一部分判不了的理由。
 #[derive(Default)]
diff --git a/crates/singlefs-checker/src/lib.rs b/crates/singlefs-checker/src/lib.rs
index aa2a39e..beefe78 100644
--- a/crates/singlefs-checker/src/lib.rs
+++ b/crates/singlefs-checker/src/lib.rs
@@ -598,7 +598,8 @@ pub fn key_schema_for_tree_kind(kind: u16) -> Option<KeySchema> {
         3 => Some(KEY_SCHEMA_ALLOCATION),
         4 => Some(KEY_SCHEMA_ACCOUNTING),
         5 => Some(KEY_SCHEMA_MAPPING),
-        10 | 11 => Some(KEY_SCHEMA_CONTAINER_NUMBER),
+        // 墓碑树 10、谱系树 11、退役树 13：码 2 节点的 key 是容器号 8 字节。
+        10 | 11 | 13 => Some(KEY_SCHEMA_CONTAINER_NUMBER),
         _ => None,
     }
 }
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 0648f8e..8a9d3e0 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -11,8 +11,9 @@ use singlefs_format::{
     CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES,
     EXTENT_TREE_INTERNAL_ENTRY_BYTES, EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES, INODE_INTERNAL_ENTRY,
     JOURNAL_RECORD_BYTES, JOURNAL_SAFETY_FACTOR, MAPPING_ENTRY_BYTES, NODE_BYTES,
-    NODE_POINTER_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES, SLOT_BYTES,
-    TOMBSTONE_CONTAINER_RECORDS, TREE_IDENTIFIER_WATERMARK_AT_MKFS, TREE_TABLE_ENTRY_BYTES,
+    NODE_POINTER_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES, RETIREMENT_CONTAINER_RECORDS,
+    SLOT_BYTES, TOMBSTONE_CONTAINER_RECORDS, TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+    TREE_TABLE_ENTRY_BYTES,
 };
 
 use crate::position_addressed::{
@@ -43,10 +44,12 @@ use crate::image::{
     DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
     DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
     DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY, DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
+    EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
     FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
     HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
     INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
     INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
+    INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
     INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
     INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
     JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
@@ -57,6 +60,10 @@ use crate::image::{
     ONE_OBJECT_BIRTH_PER_INODE_NUMBER, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
     POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER, PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
     QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE, REFERENCE_INTERVALS_ARE_EXACT,
+    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
+    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
     SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
     TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
     TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
@@ -87,10 +94,13 @@ const TREE_KIND_DELETED_INODES: u16 = 9;
 const TREE_KIND_TOMBSTONE: u16 = 10;
 const TREE_KIND_LINEAGE: u16 = 11;
 const TREE_KIND_LOGGED_OPERATIONS: u16 = 12;
-/// 十一棵进树表的树的种类，按发号次序（D8（核心索引结构） 已定项 8 ②「十二棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65547」：
+/// 退役树：码 13（D8（核心索引结构） 已定项 9 登记表，接在 logged_ops 之后追加；判决 `research/prompts/m3-retire-r1-main-verification.md`
+/// 第三节第 1 条），checker 按自己读的条款另写一份。
+const TREE_KIND_RETIREMENT: u16 = 13;
+/// 十二棵进树表的树的种类，按发号次序（D8（核心索引结构） 已定项 8 ②「十三棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65548」：
 /// extent、inode、分配记录、记账、中央映射、livelist、稀疏旁表、deadlist，再接 D18（块里携带什么信息） 已定项 24 第 6 条列的
-/// `deleted_inodes`、墓碑树、谱系树、logged_ops；中央映射树不进树表，这里没有它）。
-const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 11] = [
+/// `deleted_inodes`、墓碑树、谱系树、logged_ops，最后是退役树；中央映射树不进树表，这里没有它）。
+const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 12] = [
     TREE_KIND_EXTENT,
     TREE_KIND_INODE,
     TREE_KIND_ALLOCATION,
@@ -102,8 +112,11 @@ const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 11] = [
     TREE_KIND_TOMBSTONE,
     TREE_KIND_LINEAGE,
     TREE_KIND_LOGGED_OPERATIONS,
+    TREE_KIND_RETIREMENT,
 ];
 const PACKED_TYPE_TOMBSTONE: u16 = 1;
+/// 退役记录的打包记录类型（D18（块里携带什么信息） 已定项 11 登记的 3；记录宽 34，判决 m3-retire-r1 第三节第 2 条）。
+const PACKED_TYPE_RETIREMENT: u16 = 3;
 const PACKED_TYPE_INODE: u16 = 2;
 /// 谱系记录的打包记录类型（D18（块里携带什么信息） 已定项 24 第 5 条，不复用登记过的 3）。
 const PACKED_TYPE_LINEAGE: u16 = 5;
@@ -190,9 +203,11 @@ const fn registered_record_width(record_type: u16) -> Option<usize> {
     match record_type {
         PACKED_TYPE_INODE => Some(140),
         PACKED_TYPE_INSTANCE_TABLE => Some(88),
-        // 墓碑记录 40、谱系记录 16（D18（块里携带什么信息） 已定项 24 第 2、5 条）。
+        // 墓碑记录 40、谱系记录 24（D18（块里携带什么信息） 已定项 24 第 2、5 条）。
         PACKED_TYPE_TOMBSTONE => Some(40),
-        PACKED_TYPE_LINEAGE => Some(16),
+        PACKED_TYPE_LINEAGE => Some(24),
+        // 退役记录 34（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2 条）。
+        PACKED_TYPE_RETIREMENT => Some(34),
         _ => None,
     }
 }
@@ -275,7 +290,9 @@ fn field_table_width_of_tree_kind(kind: u16) -> Option<usize> {
         TREE_KIND_ACCOUNTING => Some(accounting_entry_bytes()),
         TREE_KIND_DELETED_INODES => Some(DELETED_INODES_ENTRY_BYTES),
         // 墓碑树、谱系树的根条目照 inode 树内部节点的 120 字节形态（D18（块里携带什么信息） 已定项 24 第 1、5 条）。
-        TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE => Some(inode_internal_entry_bytes()),
+        TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE | TREE_KIND_RETIREMENT => {
+            Some(inode_internal_entry_bytes())
+        }
         _ => None,
     }
 }
@@ -426,6 +443,9 @@ struct Walk<'reader> {
     /// 最新根下面记账树里的行：(统计量, 设备) → 值。
     accounting: BTreeMap<(u16, u32), u64>,
     accounting_seen: bool,
+    /// 走全了的记账树里 inode 号水位那一统计量（不带设备维）的每一行 (代段, 值)，按走到的次序（key 升序）：
+    /// 每次发布写的那一行与台阶行（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）。只取最新根那一遍（`check_pool_image` 走完最新根就拷走）。
+    inode_number_watermark_rows: Vec<(u64, u64)>,
     /// 最新根指着的实例表里的行：别的根按它判有效（回退之后被抛弃时间线的根不进 I-3.1 的并集），
     /// I-1.8（归并后版本全序） 的已发布谓词也读它。
     instance_table_rows: Vec<InstanceTableRow>,
@@ -1149,6 +1169,7 @@ impl<'reader> Walk<'reader> {
             data_unit_objects: Vec::new(),
             accounting: BTreeMap::new(),
             accounting_seen: false,
+            inode_number_watermark_rows: Vec::new(),
             instance_table_rows: Vec::new(),
             abandoned_roots_filtering:
                 AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable,
@@ -2231,9 +2252,19 @@ impl<'reader> Walk<'reader> {
             {
                 self.accounting_seen = true;
                 // I-3.13：记账代 = checkpoint 号（D16（发布语义） 已定项 6），每条根的记账树只装这条根那一代的行；只判走全了的树。
+                // inode 号水位那一统计量的台阶行例外（D5（快照 / 空间记账机制） 已定项 2 射程「I-3.13 对台阶行同样不适用」）：
+                // 那一统计量、不带设备维、代段早于这一版 txg 的行就是台阶行，跳过；它代段等于这一版 txg 的那一行与别的统计量照判。
                 let accounting_version_txg = self.version_being_walked.checkpoint_txg();
                 for row in &rows {
                     let row_generation = read_u64(row, ACCOUNTING_KEY_GENERATION_OFFSET);
+                    let is_an_inode_number_watermark_step = read_u16(row, 0)
+                        == STATISTIC_INODE_WATERMARK
+                        && read_u32(row, ACCOUNTING_ROW_DEVICE_OFFSET)
+                            == STATISTIC_NO_DEVICE_DIMENSION
+                        && row_generation < accounting_version_txg;
+                    if is_an_inode_number_watermark_step {
+                        continue;
+                    }
                     self.judgements.judge(
                         ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
                         row_generation == accounting_version_txg,
@@ -2249,6 +2280,15 @@ impl<'reader> Walk<'reader> {
                 for row in &rows {
                     self.accounting
                         .insert((read_u16(row, 0), read_u32(row, 10)), read_u64(row, 22));
+                    if read_u16(row, 0) == STATISTIC_INODE_WATERMARK
+                        && read_u32(row, ACCOUNTING_ROW_DEVICE_OFFSET)
+                            == STATISTIC_NO_DEVICE_DIMENSION
+                    {
+                        self.inode_number_watermark_rows.push((
+                            read_u64(row, ACCOUNTING_KEY_GENERATION_OFFSET),
+                            read_u64(row, ACCOUNTING_ROW_VALUE_OFFSET),
+                        ));
+                    }
                 }
             }
             return;
@@ -2326,6 +2366,18 @@ impl<'reader> Walk<'reader> {
                 "谱系树",
             );
         }
+        // 退役树（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5、8 条）：形态照墓碑树，类型段 {0, 3}，
+        // 每条内部条目按 I-9.31 判身份；记录那几条（I-9.32–I-9.35）在墓碑与谱系那一遍里每一版各判。
+        if kind == TREE_KIND_RETIREMENT {
+            self.walk_container_tree_root(
+                &node,
+                node_placement,
+                tree,
+                PACKED_TYPE_RETIREMENT,
+                RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+                "退役树",
+            );
+        }
     }
 
     /// I-9.25：`previous_snapshot_txg` 只写在 inode 树那条条目里，0 = 不适用；inode 树那条不大于这一版的 txg
@@ -2761,8 +2813,9 @@ impl<'reader> Walk<'reader> {
         self.judge_container_numbers_along_the_leaf_order(&leaves_in_leaf_order);
     }
 
-    /// I-9.4（容器号不超最小 key） 的第二、三句：沿叶序（这一版的 inode 树从左到右）逐对比，容器号严格递增，且左容器的最大 key < 右容器号
-    /// （这条逼出合并只许左吸收右 / 左从右借）。第一句（记录 inode ≥ 容器号）与第四句（同一片容器里记录号沿记录序严格递增）在走叶容器时逐条判。
+    /// I-9.4（容器号不超最小 key） 的第二句：沿叶序（这一版的 inode 树从左到右）逐对比，容器号严格递增。原来的第三句（左容器的最大 key < 右容器号）
+    /// 随 N1 删掉（用户 2026-10-08 定案 X4，判决 `research/prompts/m3-leaf-death-r3-main-verification.md` 第五节）：左从右借之后右片保留身份，
+    /// 它的容器号可以不大于左片的最大 key。第一句（记录 inode ≥ 容器号）与第四句（同一片容器里记录号沿记录序严格递增）在走叶容器时逐条判。
     /// 叶序里有别的版本已经走过的叶（缓存的容器号与 key 区间）；读不出的叶不在叶序里，它两边的叶直接相邻着比
     /// （合法镜像上容器号与 key 沿叶序单调，隔一片比照样成立）。
     fn judge_container_numbers_along_the_leaf_order(
@@ -2778,13 +2831,6 @@ impl<'reader> Walk<'reader> {
                         left.container, right.container
                     )
                 });
-            self.judgements
-                .judge("I-9.4", left.largest_inode < right.container, || {
-                    format!(
-                        "左容器 {} 的最大 key {} 不小于右容器号 {}",
-                        left.container, left.largest_inode, right.container
-                    )
-                });
         }
     }
 
@@ -4228,7 +4274,7 @@ const DATA_UNIT_SLOTS: u64 = DATA_UNIT_BYTES / SLOT_BYTES;
 enum HeadIdentifierOfATreeTableEntry {
     /// 种类 2（inode 树）：等于这条条目自己的树 ID。
     ItsOwnTreeIdentifier,
-    /// 种类 1（extent 树）、9（`deleted_inodes`）、10（墓碑树）：等于同一张树表里种类 2 条目的树 ID；
+    /// 种类 1（extent 树）、9（`deleted_inodes`）、10（墓碑树）、13（退役树）：等于同一张树表里种类 2 条目的树 ID；
     /// 种类 2 条目多于一条时放宽成「等于其中某一条的」（收严留给多头那一批），一条都没有时判红。
     TreeIdentifierOfAnInodeTreeEntryOfTheSameTable,
     /// 种类 3、4、6、7、8、11、12（无归属，D5（快照 / 空间记账机制） 已定项 9）：等于 0。
@@ -4241,9 +4287,10 @@ impl HeadIdentifierOfATreeTableEntry {
     const fn of_tree_kind(kind: u16) -> Self {
         match kind {
             TREE_KIND_INODE => Self::ItsOwnTreeIdentifier,
-            TREE_KIND_EXTENT | TREE_KIND_DELETED_INODES | TREE_KIND_TOMBSTONE => {
-                Self::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable
-            }
+            TREE_KIND_EXTENT
+            | TREE_KIND_DELETED_INODES
+            | TREE_KIND_TOMBSTONE
+            | TREE_KIND_RETIREMENT => Self::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable,
             TREE_KIND_ALLOCATION
             | TREE_KIND_ACCOUNTING
             | TREE_KIND_LIVELIST
@@ -4272,7 +4319,7 @@ impl HeadIdentifierOfATreeTableEntry {
 /// 或哪一种。
 /// ① 盘上相邻两条的树 ID（条目打头 8 字节）严格升序；② 按发号次序（[`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`]），
 /// 树表里有的种类相邻两棵的树 ID 严格升序；③ 同一张树表里每一种种类码（登记过的与没登记的都算）至多一条；④ 树表非空时
-/// 进树表的十一种（[`TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES`]）各恰好一条
+/// 进树表的十二种（[`TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES`]，第 50 批加了退役树）各恰好一条
 /// （判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y3「种类 1 不止一条；种类 3 不是恰好一条」那一行；
 /// 十一种取自判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-9.16 ④ 放过缺种类的树表（C1）」那一行）。
 /// 0 条的树表四道都成立，1 条的 ① ② ③ 成立。checker 自己按字段表切一份，不与实现共用代码（D13（验证路线） 已定项 5；
@@ -4337,12 +4384,12 @@ fn judge_tree_table_entries_ordering(
     });
 }
 
-/// I-9.16 ④（[`judge_tree_table_entry_kind_counts`]）：树表非空时各要恰好一条的种类，就是进树表的那十一种
-/// （[`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`]，同一份，不另抄）。读者重建一版时十二棵树的号逐棵从树表取：
-/// 十一棵按种类找条目、取条目里的树 ID，中央映射树不进树表、取根记录里它那条根指针的出生树；缺哪一种都重建不出这一版
+/// I-9.16 ④（[`judge_tree_table_entry_kind_counts`]）：树表非空时各要恰好一条的种类，就是进树表的那十二种
+/// （[`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`]，同一份，不另抄）。读者重建一版时十三棵树的号逐棵从树表取：
+/// 十二棵按种类找条目、取条目里的树 ID，中央映射树不进树表、取根记录里它那条根指针的出生树；缺哪一种都重建不出这一版
 /// （实现那一侧是 `crates/singlefs-core/src/recovery.rs` 的 `rebuild_version` 逐种取号时报的「树表里没有那棵树」）。
 /// 只读前四种根指针的那几行读法前提读窄了（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1 C1 那一行更正）。
-const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 11] =
+const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 12] =
     TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER;
 
 /// I-9.16 的 ③ ④（[`judge_tree_table_entries_ordering`]）：③ 每一种种类码至多一条；④ 树表非空时
@@ -4381,7 +4428,7 @@ fn judge_tree_table_entry_kind_counts(
     judgements.judge("I-9.16", first_rooted_kind_missing.is_none(), || {
         let kind = first_rooted_kind_missing.expect("判红时找到过那一种");
         format!(
-            "{version_name}的树表有 {} 条，种类 {kind} 一条都没有（④ 树表非空时进树表的十一种各恰好一条）",
+            "{version_name}的树表有 {} 条，种类 {kind} 一条都没有（④ 树表非空时进树表的十二种各恰好一条）",
             tree_table_entries.len()
         )
     });
@@ -4484,7 +4531,9 @@ impl TreeKindForReferenceScan {
             TREE_KIND_ALLOCATION => TreeKindForReferenceScan::Allocation,
             TREE_KIND_ACCOUNTING => TreeKindForReferenceScan::Accounting,
             TREE_KIND_DELETED_INODES => TreeKindForReferenceScan::DeletedInodes,
-            TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE => TreeKindForReferenceScan::ContainerTree,
+            TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE | TREE_KIND_RETIREMENT => {
+                TreeKindForReferenceScan::ContainerTree
+            }
             unregistered => TreeKindForReferenceScan::WithoutWalkableEntryFormat(unregistered),
         }
     }
@@ -9125,6 +9174,9 @@ enum DeletionScanGap {
     LineageTreeUnreadable,
     LineageRecordsInconsistent,
     QueuedInodeWithoutARecord,
+    RetirementTreeUnreadable,
+    RetirementTreeHeadIdentifierNamesNoSingleInodeTree,
+    ParentVersionNotSingle,
 }
 
 impl DeletionScanGap {
@@ -9187,6 +9239,15 @@ impl DeletionScanGap {
             DeletionScanGap::QueuedInodeWithoutARecord => {
                 "走读到的某一版 deleted_inodes 里有一个号在同一版 inode 树里没有记录：那个号要的对象出生代没有，它的谱系上墓碑判不了（这一形由 I-9.5 判红）"
             }
+            DeletionScanGap::RetirementTreeUnreadable => {
+                "走读到的某一版退役树读不全（节点或容器读不出、层级或条目宽不对、类型段不认得、容器的打包记录类型或记录宽不是登记的）：那一版的退役记录判不了"
+            }
+            DeletionScanGap::RetirementTreeHeadIdentifierNamesNoSingleInodeTree => {
+                "走读到的某一版退役树条目偏移 116 的头 ID 配不上同一张树表里恰好一条 inode 树条目：那一版退役记录对叶容器的那两判判不了"
+            }
+            DeletionScanGap::ParentVersionNotSingle => {
+                "走读到的某一版的父版（那个 txg）在回退候选集里不止一版、树表不同：拿哪一版的叶身份集比判不了"
+            }
         }
     }
 }
@@ -9963,9 +10024,11 @@ const TOMBSTONE_RECORD_DEATH_TXG_OFFSET: usize = 32;
 /// 第一版墓碑只许整对象：区间 [0, 2⁶⁴ − 1]（D18（块里携带什么信息） 已定项 24 第 2 条）。
 const TOMBSTONE_WHOLE_OBJECT_RANGE_START: u64 = 0;
 const TOMBSTONE_WHOLE_OBJECT_RANGE_END: u64 = u64::MAX;
-/// 谱系记录 16 字节：T_rb 8（回退那次发布的 txg）+ T_old 8（回退目标的 txg），读作「T_rb 的父是 T_old」（D18（块里携带什么信息） 已定项 24 第 5 条）。
+/// 谱系记录 24 字节：T_rb 8（回退那次发布的 txg）+ T_old 8（回退目标的 txg）+ H_cur 8（cur，即回退之前的现行那一版的 inode 号水位），
+/// 读作「T_rb 的父是 T_old」（D18（块里携带什么信息） 已定项 24 第 5 条；三段的逐字节次序条款没定，实现取 T_rb | T_old | H_cur，按字段表另写一份、不用实现的解析）。
 const LINEAGE_RECORD_ROLLBACK_TXG_OFFSET: usize = 0;
 const LINEAGE_RECORD_PARENT_TXG_OFFSET: usize = 8;
+const LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET: usize = 16;
 
 /// 只供测试的墓碑容器容量旋钮压到的每片记录数（D18（块里携带什么信息） 已定项 24 第 3 条：旋钮不落盘，池级 checker 从镜像外拿；
 /// 判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T5 取 K1）。取值 1..=815，构造时验。
@@ -10025,6 +10088,26 @@ pub const NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
 pub const NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
     "走读到的每一版墓碑树都是零根：没有墓碑树内部条目可判";
 
+/// 退役记录那几条（I-9.32–I-9.35）在一版上的那一组：一版的树表读不出、退役树读不全时这几条在那一版判不了。
+const RETIREMENT_SCAN_INVARIANTS: [&str; 4] = [
+    EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
+    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
+    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+];
+/// I-9.32：走读到的每一版退役树都是零根。
+pub const NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION: &str =
+    "走读到的每一版退役树都是零根：没有退役记录可判";
+/// I-9.33、I-9.34：没有一条退役记录的死亡 txg 落在它那一版的父链上。
+pub const NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION: &str =
+    "走读到的每一版里没有一条退役记录在谱系上：没有对象可判";
+/// I-9.35：没有一版的父版在回退候选集里读得出。
+pub const NO_VERSION_WHOSE_PARENT_IS_IN_THE_CANDIDATE_SET: &str =
+    "走读到的每一版的父版都不在回退候选集里（或都没有 inode 树）：没有相邻两版可比";
+/// I-9.31（退役树条目身份与子头相符）：主走读一条退役树内部条目都没走到。
+pub const NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
+    "走读到的每一版退役树都是零根：没有退役树内部条目可判";
+
 /// 墓碑与谱系那一遍在一版上判 I-9.18、I-9.19、I-9.20、I-9.29 要的那一组：一版的树表读不出、墓碑树读不全时这几条在那一版判不了
 /// （I-9.21 / I-9.22 只记装没装旋钮的那一条，[`capacity_invariant_of`]）。
 const TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS: [&str; 4] = [
@@ -10099,11 +10182,12 @@ struct TombstoneTreeOfTheScan {
     containers: Result<Vec<TombstoneContainerOfTheScan>, DeletionScanGap>,
 }
 
-/// 一条谱系记录：T_rb 的父是 T_old。
+/// 一条谱系记录：T_rb 的父是 T_old；H_cur 是回退之前的现行那一版的 inode 号水位。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct LineageRecordOfTheScan {
     rollback_txg: u64,
     parent_txg: u64,
+    inode_number_watermark_of_the_version_rolled_back_from: u64,
 }
 
 /// 一版的父链在 txg 轴上占的那几段（每段闭区间）：「在谱系上」= 死亡 txg 落在其中一段里。
@@ -10295,10 +10379,604 @@ fn lineage_records_without_judging(
         .map(|record| LineageRecordOfTheScan {
             rollback_txg: read_u64(record, LINEAGE_RECORD_ROLLBACK_TXG_OFFSET),
             parent_txg: read_u64(record, LINEAGE_RECORD_PARENT_TXG_OFFSET),
+            inode_number_watermark_of_the_version_rolled_back_from: read_u64(
+                record,
+                LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET,
+            ),
         })
         .collect())
 }
 
+/// 退役记录（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2 条，34 字节小端）各段的偏移：被退役身份的出生树 0、
+/// 打包记录类型 8（2 字节）、容器号 10、容器出生代 18，死亡 txg 26。checker 按条款另写一份，不用实现的读法。
+const RETIREMENT_RECORD_BIRTH_TREE_OFFSET: usize = 0;
+const RETIREMENT_RECORD_RECORD_TYPE_OFFSET: usize = 8;
+const RETIREMENT_RECORD_CONTAINER_OFFSET: usize = 10;
+const RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET: usize = 18;
+const RETIREMENT_RECORD_DEATH_TXG_OFFSET: usize = 26;
+
+/// 一片打包记录容器的身份四段（出生树、打包记录类型、容器号、容器出生代）：inode 树内部条目里的身份引用、退役记录里被退役的那一个都是这一形。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+struct ContainerIdentityOfTheScan {
+    birth_tree: u64,
+    record_type: u16,
+    container: u64,
+    container_birth: u64,
+}
+
+impl ContainerIdentityOfTheScan {
+    /// 从一段按「出生树 8 | 打包记录类型 2 | 容器号 8 | 容器出生代 8」排的字节里读（`offset` 是出生树那一段的起点）。
+    fn read_at(bytes: &[u8], offset: usize) -> Self {
+        Self {
+            birth_tree: read_u64(bytes, offset),
+            record_type: read_u16(
+                bytes,
+                offset + RETIREMENT_RECORD_RECORD_TYPE_OFFSET - RETIREMENT_RECORD_BIRTH_TREE_OFFSET,
+            ),
+            container: read_u64(
+                bytes,
+                offset + RETIREMENT_RECORD_CONTAINER_OFFSET - RETIREMENT_RECORD_BIRTH_TREE_OFFSET,
+            ),
+            container_birth: read_u64(
+                bytes,
+                offset + RETIREMENT_RECORD_CONTAINER_BIRTH_OFFSET
+                    - RETIREMENT_RECORD_BIRTH_TREE_OFFSET,
+            ),
+        }
+    }
+}
+
+/// 一条退役记录：被退役的身份与死亡 txg。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct RetirementRecordOfTheScan {
+    retired: ContainerIdentityOfTheScan,
+    death_txg: u64,
+}
+
+impl RetirementRecordOfTheScan {
+    fn of(record: &[u8]) -> Self {
+        Self {
+            retired: ContainerIdentityOfTheScan::read_at(
+                record,
+                RETIREMENT_RECORD_BIRTH_TREE_OFFSET,
+            ),
+            death_txg: read_u64(record, RETIREMENT_RECORD_DEATH_TXG_OFFSET),
+        }
+    }
+}
+
+/// 一片退役容器：头里的容器号与它的记录（I-9.32 数每片不多于 959 条）。
+struct RetirementContainerOfTheScan {
+    container: u64,
+    records: Vec<RetirementRecordOfTheScan>,
+}
+
+/// 一版树表里的一棵退役树：树 ID、头 ID（I-9.27：等于这张表里那棵 inode 树的树 ID）与按叶序读出的容器（读不全交回判不了的理由）。
+struct RetirementTreeOfTheScan {
+    tree: u64,
+    head_identifier: u64,
+    containers: Result<Vec<RetirementContainerOfTheScan>, DeletionScanGap>,
+}
+
+/// 一版的退役树按叶序读出的每片容器与记录，只读不判。
+fn retirement_containers_without_judging(
+    reader: &dyn ImageReader,
+    root_pointer: &PointerView,
+    index_cache: &mut IndexNodeCache,
+    packed_cache: &mut PackedUnitCache,
+) -> Result<Vec<RetirementContainerOfTheScan>, DeletionScanGap> {
+    let containers = containers_of_a_container_tree_without_judging(
+        reader,
+        root_pointer,
+        PACKED_TYPE_RETIREMENT,
+        DeletionScanGap::RetirementTreeUnreadable,
+        index_cache,
+        packed_cache,
+    )?;
+    Ok(containers
+        .iter()
+        .map(|container| RetirementContainerOfTheScan {
+            container: container.container,
+            records: container
+                .records
+                .iter()
+                .map(|record| RetirementRecordOfTheScan::of(record))
+                .collect(),
+        })
+        .collect())
+}
+
+/// 一版 inode 树里每片叶容器的身份（内部条目里类型段 2 那几条的身份引用，不开叶：判决第三节第 8 条 I-9.34「读内部条目的身份引用即可」），
+/// 只读不判。根指针全零、零条目根交回空。读不出、形状不对的交回 [`DeletionScanGap::InodeTreeUnreadable`]（主走读判 I-9.2 那些）。
+fn inode_leaf_identities_without_judging(
+    reader: &dyn ImageReader,
+    root_pointer: &PointerView,
+    index_cache: &mut IndexNodeCache,
+) -> Result<BTreeSet<ContainerIdentityOfTheScan>, DeletionScanGap> {
+    let mut identities = BTreeSet::new();
+    if root_pointer.all_zero {
+        return Ok(identities);
+    }
+    let root = read_index_node_without_judging(reader, root_pointer, index_cache)
+        .ok_or(DeletionScanGap::InodeTreeUnreadable)?;
+    if root.level == 0 {
+        return if root.entries.is_empty() {
+            Ok(identities)
+        } else {
+            Err(DeletionScanGap::InodeTreeUnreadable)
+        };
+    }
+    collect_inode_leaf_identities_below(reader, &root, index_cache, &mut identities)?;
+    Ok(identities)
+}
+
+/// [`inode_leaf_identities_without_judging`] 的一个内部节点：类型段 2 的条目收它的身份引用，类型段 0 的往下一层（层级 = 父层级 − 1 且 ≥ 1）。
+/// 递归的上界：每下一层层级减一。
+fn collect_inode_leaf_identities_below(
+    reader: &dyn ImageReader,
+    node: &crate::IndexNodeView,
+    index_cache: &mut IndexNodeCache,
+    identities: &mut BTreeSet<ContainerIdentityOfTheScan>,
+) -> Result<(), DeletionScanGap> {
+    if node.entry_width < inode_internal_entry_bytes() {
+        return Err(DeletionScanGap::InodeTreeUnreadable);
+    }
+    for entry in &node.entries {
+        match InodeInternalEntryType::of(read_u16(entry, 16)) {
+            InodeInternalEntryType::LeafContainer => {
+                identities.insert(ContainerIdentityOfTheScan::read_at(entry, 8));
+            }
+            InodeInternalEntryType::InternalNode => {
+                let child_pointer = parse_node_pointer(&entry[34..120]);
+                if child_pointer.all_zero {
+                    return Err(DeletionScanGap::InodeTreeUnreadable);
+                }
+                let child = read_index_node_without_judging(reader, &child_pointer, index_cache)
+                    .ok_or(DeletionScanGap::InodeTreeUnreadable)?;
+                if node.level.checked_sub(1) != Some(child.level) || child.level == 0 {
+                    return Err(DeletionScanGap::InodeTreeUnreadable);
+                }
+                collect_inode_leaf_identities_below(reader, &child, index_cache, identities)?;
+            }
+            InodeInternalEntryType::Unregistered(_) => {
+                return Err(DeletionScanGap::InodeTreeUnreadable);
+            }
+        }
+    }
+    Ok(())
+}
+
+/// I-9.35 跨相邻两版比要的、一版里的那几样：txg、父版的 txg（这一版谱系树里有 T_rb 等于这一版 txg 的记录时取它的 T_old，否则 txg − 1；
+/// txg 0 没有父版；谱系树读不全交回判不了的理由）、每棵 inode 树（按树 ID）的叶身份集、经头 ID 配到每棵 inode 树的退役树里死亡 txg 等于这一版
+/// txg 的退役记录的身份集。树表 0 条的一版没有 inode 树（当父版时它的每棵树都是空集）。
+struct VersionOfTheRetirementCompleteness {
+    name: String,
+    checkpoint_txg: u64,
+    parent_txg: Result<Option<u64>, DeletionScanGap>,
+    leaf_identities_of_each_inode_tree:
+        BTreeMap<u64, Result<BTreeSet<ContainerIdentityOfTheScan>, DeletionScanGap>>,
+    retired_at_this_txg_of_each_inode_tree:
+        BTreeMap<u64, Result<BTreeSet<ContainerIdentityOfTheScan>, DeletionScanGap>>,
+    /// 这一版树表读不出、条目窄：它当父版时比不了。
+    unreadable: Option<DeletionScanGap>,
+}
+
+/// I-9.32（退役记录合法）在一版的一棵退役树上：每条记录被退役身份的打包记录类型 == 2、出生树 ≠ 0，容器出生代 < 死亡 txg ≤ 这一版的 txg；
+/// 每片退役容器的记录数 ≤ 959（正推 K5.1）。不按谱系滤：每条都判。
+fn judge_every_retirement_record_of_a_version_is_well_formed(
+    version: &VersionToScanTheDeletionStateOf<'_>,
+    retirement_tree: u64,
+    containers: &[RetirementContainerOfTheScan],
+    judgements: &mut Judgements,
+) {
+    let records_at_most = usize::try_from(RETIREMENT_CONTAINER_RECORDS).expect("959 装得进 usize");
+    for container in containers {
+        let record_count = container.records.len();
+        judgements.judge(
+            EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
+            record_count <= records_at_most,
+            || {
+                format!(
+                    "{}退役树 {retirement_tree} 里容器 {} 装了 {record_count} 条退役记录，多于一片的 {records_at_most} 条",
+                    version.name, container.container
+                )
+            },
+        );
+        for record in &container.records {
+            let retired = record.retired;
+            let holds = retired.record_type == PACKED_TYPE_INODE
+                && retired.birth_tree != 0
+                && retired.container_birth < record.death_txg
+                && record.death_txg <= version.checkpoint_txg;
+            judgements.judge(EVERY_RETIREMENT_RECORD_IS_WELL_FORMED, holds, || {
+                format!(
+                    "{}退役树 {retirement_tree} 容器 {} 里的退役记录 {retired:?}（死于 txg {}）：被退役身份的类型要是 2、出生树不为 0，\
+                     容器出生代 < 死亡 txg ≤ 这一版的 txg {}",
+                    version.name, container.container, record.death_txg, version.checkpoint_txg
+                )
+            });
+        }
+    }
+}
+
+/// I-9.33（谱系上的退役记录按身份不重复）在一版的一棵退役树上：死亡 txg 在这一版父链上的那几条，按身份不重复。
+fn judge_retirement_records_on_the_lineage_of_a_version_are_distinct(
+    version_name: &str,
+    retirement_tree: u64,
+    containers: &[RetirementContainerOfTheScan],
+    lineage_chain: &LineageChainOfAVersion,
+    judgements: &mut Judgements,
+) {
+    let mut seen: BTreeMap<ContainerIdentityOfTheScan, u64> = BTreeMap::new();
+    for record in containers
+        .iter()
+        .flat_map(|container| container.records.iter())
+        .filter(|record| lineage_chain.holds_txg(record.death_txg))
+    {
+        let earlier_death_txg = seen.insert(record.retired, record.death_txg);
+        judgements.judge(
+            RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
+            earlier_death_txg.is_none(),
+            || {
+                format!(
+                    "{version_name}退役树 {retirement_tree} 里谱系上有两条退役记录指同一个身份 {:?}（死于 txg {:?} 与 {}）",
+                    record.retired, earlier_death_txg, record.death_txg
+                )
+            },
+        );
+    }
+}
+
+/// I-9.34（谱系上的退役记录不指本版活叶）在一版的一棵退役树上：死亡 txg 在这一版父链上的退役记录指认的身份不在头 ID 配得上的那棵 inode 树的叶身份集里。
+fn judge_retirement_records_on_the_lineage_name_no_live_leaf(
+    version_name: &str,
+    retirement_tree: u64,
+    containers: &[RetirementContainerOfTheScan],
+    leaf_identities: &BTreeSet<ContainerIdentityOfTheScan>,
+    lineage_chain: &LineageChainOfAVersion,
+    judgements: &mut Judgements,
+) {
+    for record in containers
+        .iter()
+        .flat_map(|container| container.records.iter())
+        .filter(|record| lineage_chain.holds_txg(record.death_txg))
+    {
+        judgements.judge(
+            RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+            !leaf_identities.contains(&record.retired),
+            || {
+                format!(
+                    "{version_name}退役树 {retirement_tree} 里谱系上的退役记录（死于 txg {}）指的身份 {:?} 是这一版 inode 树里的一片叶",
+                    record.death_txg, record.retired
+                )
+            },
+        );
+    }
+}
+
+/// I-9.35（退役完备）：每一版、它每棵 inode 树，父版（[`VersionOfTheRetirementCompleteness::parent_txg`]）在回退候选集里读得出时，
+/// 父版那棵树的叶身份集 − 这一版的叶身份集 = 这一版死亡 txg 等于自己 txg 的退役记录的身份集；两个方向（漏记、多记）各报。
+/// 父版那个 txg 不在候选集里的不判（崩溃跳号、最旧那一版）；父版读不全、同 txg 有不止一版的报射程里判不了。
+/// 候选集滤不掉被崩溃恢复抛弃的根时整条报部分没判（照 I-9.29 的门：被抛弃线上的同一个 txg 可以是另一版）。
+fn judge_retirement_records_account_for_every_leaf_gone_since_the_parent_version(
+    versions: &[VersionOfTheRetirementCompleteness],
+    abandoned_roots_filtering: AbandonedRootsFilteringOfTheCandidateSet,
+    judgements: &mut Judgements,
+) {
+    match abandoned_roots_filtering {
+        AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable => {}
+        AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable => {
+            judgements.partially_judged(
+                RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+                format!(
+                    "最新根的实例表读不出（主走读沿链没走到末片）：回退候选集里滤不掉被崩溃恢复抛弃的根，{} 版的父版一版都没比",
+                    versions.len()
+                ),
+            );
+            return;
+        }
+    }
+    let empty_leaf_set: BTreeSet<ContainerIdentityOfTheScan> = BTreeSet::new();
+    for version in versions {
+        if version.leaf_identities_of_each_inode_tree.is_empty() {
+            continue;
+        }
+        let parent_txg = match version.parent_txg {
+            Ok(Some(parent_txg)) => parent_txg,
+            Ok(None) => continue,
+            Err(gap) => {
+                judgements.part_of_the_range_not_judged(
+                    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+                    gap.reason(),
+                );
+                continue;
+            }
+        };
+        let parents: Vec<&VersionOfTheRetirementCompleteness> = versions
+            .iter()
+            .filter(|candidate| candidate.checkpoint_txg == parent_txg)
+            .collect();
+        let parent = match parents.as_slice() {
+            [] => continue,
+            [parent] => *parent,
+            [_, _, ..] => {
+                judgements.part_of_the_range_not_judged(
+                    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+                    DeletionScanGap::ParentVersionNotSingle.reason(),
+                );
+                continue;
+            }
+        };
+        if let Some(gap) = parent.unreadable {
+            judgements.part_of_the_range_not_judged(
+                RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+                gap.reason(),
+            );
+            continue;
+        }
+        for (inode_tree, leaf_identities) in &version.leaf_identities_of_each_inode_tree {
+            let parent_leaf_identities =
+                match parent.leaf_identities_of_each_inode_tree.get(inode_tree) {
+                    None => &empty_leaf_set,
+                    Some(Ok(identities)) => identities,
+                    Some(Err(gap)) => {
+                        judgements.part_of_the_range_not_judged(
+                            RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+                            gap.reason(),
+                        );
+                        continue;
+                    }
+                };
+            let leaf_identities = match leaf_identities {
+                Ok(identities) => identities,
+                Err(gap) => {
+                    judgements.part_of_the_range_not_judged(
+                        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+                        gap.reason(),
+                    );
+                    continue;
+                }
+            };
+            let retired_at_this_txg = match version
+                .retired_at_this_txg_of_each_inode_tree
+                .get(inode_tree)
+            {
+                None => &empty_leaf_set,
+                Some(Ok(identities)) => identities,
+                Some(Err(gap)) => {
+                    judgements.part_of_the_range_not_judged(
+                        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+                        gap.reason(),
+                    );
+                    continue;
+                }
+            };
+            let gone: BTreeSet<ContainerIdentityOfTheScan> = parent_leaf_identities
+                .difference(leaf_identities)
+                .copied()
+                .collect();
+            let missing: Vec<&ContainerIdentityOfTheScan> =
+                gone.difference(retired_at_this_txg).collect();
+            let extra: Vec<&ContainerIdentityOfTheScan> =
+                retired_at_this_txg.difference(&gone).collect();
+            judgements.judge(
+                RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+                missing.is_empty() && extra.is_empty(),
+                || {
+                    format!(
+                        "{}（txg {}）的 inode 树 {inode_tree} 比父版（txg {parent_txg}）少了的叶与死于这一版的退役记录对不上：漏记 {missing:?}，多记 {extra:?}",
+                        version.name, version.checkpoint_txg
+                    )
+                },
+            );
+        }
+    }
+}
+
+/// I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上）：最新的根下面没有走全了的记账树，或记账里没有 inode 号水位那一行。
+pub const NO_INODE_NUMBER_WATERMARK_ROW_UNDER_THE_NEWEST_ROOT: &str =
+    "最新的根下面没有走全了的记账树、或记账里没有 inode 号水位那一行：没有台阶行可判";
+
+/// 一版记账里 inode 号水位那一统计量的台阶行 (g, H)（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」），从那一统计量
+/// （不带设备维）的行 (代段, 值) 认：代段早于这一版 txg 的行都是台阶行；代段等于这一版 txg 的那一行（每次发布都写的那一行），值与代段最大的
+/// 台阶行不同（或没有更早的台阶行）才是台阶行——这一版那次发布本身水位变了。按代段升序。与写路径同一条认法、按字段表另写一份，
+/// 不用实现的读法。代段晚于这一版 txg 的行不认（I-3.13 判它）。
+fn inode_number_watermark_steps_of_a_version(
+    version_txg: u64,
+    inode_number_watermark_rows: &[(u64, u64)],
+) -> Vec<(u64, u64)> {
+    let mut steps: Vec<(u64, u64)> = inode_number_watermark_rows
+        .iter()
+        .copied()
+        .filter(|(row_generation, _)| *row_generation < version_txg)
+        .collect();
+    steps.sort_unstable();
+    if let Some(&(_, value_of_this_version)) = inode_number_watermark_rows
+        .iter()
+        .find(|(row_generation, _)| *row_generation == version_txg)
+    {
+        if steps.last().is_none_or(|(_, latest_step_watermark)| {
+            *latest_step_watermark != value_of_this_version
+        }) {
+            steps.push((version_txg, value_of_this_version));
+        }
+    }
+    steps
+}
+
+/// I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，W3 的 C2aL 形态：D5（快照 / 空间记账机制） 已定项 2 射程、D18（块里携带什么信息）
+/// 已定项 24 第 5 条）在最新的根上判，台阶行 (g, H) 取 [`inode_number_watermark_steps_of_a_version`]：
+/// ① 记账里的 inode 号水位（那一统计量代段最大的那一行，D8（核心索引结构） 已定项 6「读现行值」）不小于每一级台阶的 H；
+/// ② 最新根 inode 树里每条记录 (号 n, 对象出生代 b)、与最新根谱系上的每条墓碑 (n, b)：没有一级台阶 H > n 且 b > g
+///    （号 n 在 g 之前就发过，晚于 g 出生的对象占着它就是同一条时间线上的重发）；
+/// ④ 最新根谱系树里的每条谱系记录 (T_rb, T_old, H_cur)：代段 ≤ T_rb − 1 的台阶里最大的 H 等于 H_cur，没有这样的台阶判红；
+///    H_cur 为 0 也判红——不另写一判：台阶的水位至少是第一个 inode 号 1（发布路径与只供测试的压回开关都不写 0），等式在 H_cur = 0 时不成立
+///    （谱系接不成一条链时整份不信、报射程里判不了，同删除态扫描「在谱系上」那一判）。
+/// 正推草案的 ③（水位大于初值时有一级台阶 H ≥ 水位、g ≤ 最新根的 txg）不判：台阶行与每次发布写的那一行同 key 维度、只差代段，
+/// 那一行按上面的认法要么本身是台阶（H = 水位），要么前一级台阶的 H 就等于水位，③ 在单张镜像上恒成立。
+/// 树表、谱系树、inode 树、墓碑树读不全的那一部分报射程里判不了（同 I-9.5 那一行的共用判法），已判的照报；
+/// 只读不判、不进 `walk_failures`。
+// TODO(D18（块里携带什么信息） 已定项 24 第 5 条「有克隆头时这一判与台阶行都按头」)：`crates/` 今天没有克隆路径，这一判不按树段分头；
+// 做克隆时台阶行按 (统计量, 树) 分、记录与墓碑按头 ID 配那棵树的台阶。
+fn judge_inode_number_watermark_steps_of_the_newest_root(
+    reader: &dyn ImageReader,
+    tree_table_pointer_of_the_newest_root: &[u8],
+    newest_txg: u64,
+    inode_number_watermark_rows: &[(u64, u64)],
+    index_cache: &mut IndexNodeCache,
+    judgements: &mut Judgements,
+) {
+    let invariant =
+        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE;
+    let Some(&(_, inode_number_watermark)) = inode_number_watermark_rows
+        .iter()
+        .max_by_key(|(row_generation, _)| *row_generation)
+    else {
+        judgements.not_applicable(
+            invariant,
+            NO_INODE_NUMBER_WATERMARK_ROW_UNDER_THE_NEWEST_ROOT,
+        );
+        return;
+    };
+    let steps = inode_number_watermark_steps_of_a_version(newest_txg, inode_number_watermark_rows);
+    // ①
+    for (step_txg, step_watermark) in &steps {
+        judgements.judge(invariant, *step_watermark <= inode_number_watermark, || {
+            format!(
+                "最新根（txg {newest_txg}）记账里的 inode 号水位 {inode_number_watermark} 小于 txg {step_txg} 那一级台阶的水位 {step_watermark}"
+            )
+        });
+    }
+    let tree_table_pointer = parse_node_pointer(tree_table_pointer_of_the_newest_root);
+    if tree_table_pointer.all_zero {
+        judgements.part_of_the_range_not_judged(
+            invariant,
+            DeletionScanGap::TreeTablePointerAllZero.reason(),
+        );
+        return;
+    }
+    let Some(tree_table) =
+        read_index_node_without_judging(reader, &tree_table_pointer, index_cache)
+    else {
+        judgements
+            .part_of_the_range_not_judged(invariant, DeletionScanGap::TreeTableUnreadable.reason());
+        return;
+    };
+    if tree_table
+        .entries
+        .iter()
+        .any(|entry| entry.len() < tree_table_entry_bytes())
+    {
+        judgements.part_of_the_range_not_judged(
+            invariant,
+            DeletionScanGap::TreeTableEntryShorterThanTheFieldTable.reason(),
+        );
+        return;
+    }
+    let entries_of_kind = |kind: u16| -> Vec<&Vec<u8>> {
+        tree_table
+            .entries
+            .iter()
+            .filter(|entry| read_u16(entry, TREE_TABLE_ENTRY_KIND_OFFSET) == kind)
+            .collect()
+    };
+    let mut packed_cache = PackedUnitCache::new();
+    let lineage_records = match entries_of_kind(TREE_KIND_LINEAGE).as_slice() {
+        [lineage_entry] => lineage_records_without_judging(
+            reader,
+            &parse_node_pointer(&lineage_entry[14..100]),
+            index_cache,
+            &mut packed_cache,
+        ),
+        [] | [_, _, ..] => Err(DeletionScanGap::LineageTreeEntryNotSingle),
+    };
+    // ④：谱系记录读得出、接得成一条链（同删除态扫描「在谱系上」那一判的前提：接不成链的谱系整份不信）才判。
+    let lineage = lineage_records.and_then(|records| {
+        lineage_chain_of_a_version(newest_txg, &records)
+            .map(|lineage_chain| (records, lineage_chain))
+    });
+    match &lineage {
+        Ok((records, _)) => {
+            for record in records {
+                let watermark_rolled_back_from =
+                    record.inode_number_watermark_of_the_version_rolled_back_from;
+                let largest_step_before_the_rollback = steps
+                    .iter()
+                    .filter(|(step_txg, _)| *step_txg < record.rollback_txg)
+                    .map(|(_, step_watermark)| *step_watermark)
+                    .max();
+                judgements.judge(
+                    invariant,
+                    largest_step_before_the_rollback == Some(watermark_rolled_back_from),
+                    || {
+                        format!(
+                            "最新根（txg {newest_txg}）谱系记录 (T_rb {}, T_old {}) 带的 H_cur {watermark_rolled_back_from} 不等于代段 ≤ T_rb − 1 的台阶里最大的水位 {largest_step_before_the_rollback:?}（H_cur 为 0 也算不等）",
+                            record.rollback_txg, record.parent_txg
+                        )
+                    },
+                );
+            }
+        }
+        Err(gap) => judgements.part_of_the_range_not_judged(invariant, gap.reason()),
+    }
+    // ②
+    let step_the_number_was_issued_before = |number: u64, object_birth: u64| {
+        steps
+            .iter()
+            .find(|(step_txg, step_watermark)| *step_watermark > number && object_birth > *step_txg)
+            .copied()
+    };
+    for inode_tree_entry in entries_of_kind(TREE_KIND_INODE) {
+        match inode_records_without_judging(
+            reader,
+            &parse_node_pointer(&inode_tree_entry[14..100]),
+            index_cache,
+            &mut packed_cache,
+        ) {
+            Ok(records) => {
+                for (inode, record) in &records {
+                    let step = step_the_number_was_issued_before(*inode, record.object_birth);
+                    judgements.judge(invariant, step.is_none(), || {
+                        format!(
+                            "最新根（txg {newest_txg}）inode 树里 {inode} 号的对象出生代 {}，晚于台阶 {step:?}（txg, 水位）：那一级台阶说这个号在那之前就发过",
+                            record.object_birth
+                        )
+                    });
+                }
+            }
+            Err(gap) => judgements.part_of_the_range_not_judged(invariant, gap.reason()),
+        }
+    }
+    for tombstone_tree_entry in entries_of_kind(TREE_KIND_TOMBSTONE) {
+        let containers = tombstone_containers_without_judging(
+            reader,
+            &parse_node_pointer(&tombstone_tree_entry[14..100]),
+            index_cache,
+            &mut packed_cache,
+        );
+        match (containers, &lineage) {
+            (Ok(containers), Ok((_, lineage_chain))) => {
+                for tombstone in containers
+                    .iter()
+                    .flat_map(|container| container.records.iter())
+                    .filter(|tombstone| lineage_chain.holds_txg(tombstone.death_txg))
+                {
+                    let step =
+                        step_the_number_was_issued_before(tombstone.object, tombstone.object_birth);
+                    judgements.judge(invariant, step.is_none(), || {
+                        format!(
+                            "最新根（txg {newest_txg}）谱系上的墓碑（{} 号，对象出生代 {}，死于 txg {}）晚于台阶 {step:?}（txg, 水位）出生：那一级台阶说这个号在那之前就发过",
+                            tombstone.object, tombstone.object_birth, tombstone.death_txg
+                        )
+                    });
+                }
+            }
+            (Err(gap), _) => judgements.part_of_the_range_not_judged(invariant, gap.reason()),
+            (Ok(_), Err(gap)) => judgements.part_of_the_range_not_judged(invariant, gap.reason()),
+        }
+    }
+}
+
 /// 墓碑与谱系那几条（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节与第四节用户定案）：删除态扫描那一份版本清单里
 /// 每一版（树表与 txg 都相同的几版只扫一次——「在谱系上」从这一版自己的 txg 起算，txg 不同就各扫一遍）重读树表、谱系树、墓碑树、
 /// inode 树与 `deleted_inodes`，只读不判、不进 `walk_failures`：判 I-9.18、I-9.19、I-9.20、I-9.21 / I-9.22（看装没装旋钮）与 I-9.10 墓碑那一个等号，
@@ -10317,12 +10995,26 @@ fn judge_tombstones_and_the_lineage_of_the_versions(
     let mut packed_cache = PackedUnitCache::new();
     let mut tree_tables_and_txgs_scanned: BTreeSet<(u32, u64, u32, u64)> = BTreeSet::new();
     let mut object_births: ObjectBirthsOfEachInodeNumber = BTreeMap::new();
+    let mut versions_of_the_retirement_completeness: Vec<VersionOfTheRetirementCompleteness> =
+        Vec::new();
     let capacity_invariant = capacity_invariant_of(capacity_knob);
+    let unreadable_version = |version: &VersionToScanTheDeletionStateOf<'_>,
+                              gap: DeletionScanGap| {
+        VersionOfTheRetirementCompleteness {
+            name: version.name.clone(),
+            checkpoint_txg: version.checkpoint_txg,
+            parent_txg: Err(gap),
+            leaf_identities_of_each_inode_tree: BTreeMap::new(),
+            retired_at_this_txg_of_each_inode_tree: BTreeMap::new(),
+            unreadable: Some(gap),
+        }
+    };
     for version in versions {
         let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
         if tree_table_pointer.all_zero {
             for invariant in TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS
                 .into_iter()
+                .chain(RETIREMENT_SCAN_INVARIANTS)
                 .chain([capacity_invariant])
             {
                 judgements.part_of_the_range_not_judged(
@@ -10330,6 +11022,10 @@ fn judge_tombstones_and_the_lineage_of_the_versions(
                     DeletionScanGap::TreeTablePointerAllZero.reason(),
                 );
             }
+            versions_of_the_retirement_completeness.push(unreadable_version(
+                version,
+                DeletionScanGap::TreeTablePointerAllZero,
+            ));
             continue;
         }
         let first_location = tree_table_pointer.locations[0];
@@ -10355,29 +11051,47 @@ fn judge_tombstones_and_the_lineage_of_the_versions(
         if let Some(gap) = tree_table_gap {
             for invariant in TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS
                 .into_iter()
+                .chain(RETIREMENT_SCAN_INVARIANTS)
                 .chain([capacity_invariant])
             {
                 judgements.part_of_the_range_not_judged(invariant, gap.reason());
             }
+            versions_of_the_retirement_completeness.push(unreadable_version(version, gap));
             continue;
         }
         let Some(tree_table) = tree_table else {
             continue;
         };
         if tree_table.entries.is_empty() {
+            // 树表 0 条的一版没有 inode 树：当父版时它每棵树的叶身份集都是空集。
+            versions_of_the_retirement_completeness.push(VersionOfTheRetirementCompleteness {
+                name: version.name.clone(),
+                checkpoint_txg: version.checkpoint_txg,
+                parent_txg: Ok(None),
+                leaf_identities_of_each_inode_tree: BTreeMap::new(),
+                retired_at_this_txg_of_each_inode_tree: BTreeMap::new(),
+                unreadable: None,
+            });
             continue;
         }
-        judge_tombstones_and_the_lineage_of_a_version(
-            reader,
-            version,
-            &tree_table.entries,
-            capacity_knob,
-            index_cache,
-            &mut packed_cache,
-            &mut object_births,
-            judgements,
+        versions_of_the_retirement_completeness.push(
+            judge_tombstones_and_the_lineage_of_a_version(
+                reader,
+                version,
+                &tree_table.entries,
+                capacity_knob,
+                index_cache,
+                &mut packed_cache,
+                &mut object_births,
+                judgements,
+            ),
         );
     }
+    judge_retirement_records_account_for_every_leaf_gone_since_the_parent_version(
+        &versions_of_the_retirement_completeness,
+        abandoned_roots_filtering,
+        judgements,
+    );
     match abandoned_roots_filtering {
         AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable => {
             judge_one_object_birth_per_inode_number(&object_births, judgements);
@@ -10409,6 +11123,22 @@ fn judge_tombstones_and_the_lineage_of_the_versions(
         ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
         NO_INODE_RECORD_OR_TOMBSTONE_ON_THE_LINEAGE_TO_COMPARE,
     );
+    judgements.not_applicable(
+        EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
+        NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION,
+    );
+    judgements.not_applicable(
+        RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
+        NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
+    );
+    judgements.not_applicable(
+        RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+        NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
+    );
+    judgements.not_applicable(
+        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+        NO_VERSION_WHOSE_PARENT_IS_IN_THE_CANDIDATE_SET,
+    );
     match capacity_knob {
         None => {
             judgements.not_applicable(
@@ -10447,7 +11177,7 @@ fn judge_tombstones_and_the_lineage_of_a_version(
     packed_cache: &mut PackedUnitCache,
     object_births: &mut ObjectBirthsOfEachInodeNumber,
     judgements: &mut Judgements,
-) {
+) -> VersionOfTheRetirementCompleteness {
     let entries_of_kind = |kind: u16| -> Vec<&Vec<u8>> {
         tree_table_entries
             .iter()
@@ -10455,16 +11185,27 @@ fn judge_tombstones_and_the_lineage_of_a_version(
             .collect()
     };
     // 一张树表恰好一条谱系树条目（I-9.16 ④）：这一版的父链从它的记录与这一版自己的 txg 起算。
-    let lineage_chain = match entries_of_kind(TREE_KIND_LINEAGE).as_slice() {
+    let lineage_records = match entries_of_kind(TREE_KIND_LINEAGE).as_slice() {
         [lineage_entry] => lineage_records_without_judging(
             reader,
             &parse_node_pointer(&lineage_entry[14..100]),
             index_cache,
             packed_cache,
-        )
-        .and_then(|records| lineage_chain_of_a_version(version.checkpoint_txg, &records)),
+        ),
         [] | [_, _, ..] => Err(DeletionScanGap::LineageTreeEntryNotSingle),
     };
+    let lineage_chain = lineage_records
+        .clone()
+        .and_then(|records| lineage_chain_of_a_version(version.checkpoint_txg, &records));
+    // I-9.35 的父版：这一版谱系树里有 T_rb 等于这一版 txg 的记录时取它的 T_old，否则 txg − 1（正推 K5.1）。
+    let parent_txg = lineage_records.as_ref().map_err(|gap| *gap).map(|records| {
+        records
+            .iter()
+            .find(|record| record.rollback_txg == version.checkpoint_txg)
+            .map_or(version.checkpoint_txg.checked_sub(1), |record| {
+                Some(record.parent_txg)
+            })
+    });
     let mut inode_trees: Vec<(u64, InodeRecordsOfAnInodeTreeOfTheScan)> = Vec::new();
     for inode_tree_entry in entries_of_kind(TREE_KIND_INODE) {
         inode_trees.push((
@@ -10633,6 +11374,127 @@ fn judge_tombstones_and_the_lineage_of_a_version(
             judgements,
         );
     }
+    // 退役树那几条（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条）：I-9.32 每条记录都判；I-9.33、I-9.34 只判死亡 txg
+    // 在这一版父链上的；I-9.35 要的那几样收下来，扫完各版再跨相邻两版比。
+    let mut leaf_identities_of_each_inode_tree = BTreeMap::new();
+    for inode_tree_entry in entries_of_kind(TREE_KIND_INODE) {
+        leaf_identities_of_each_inode_tree.insert(
+            read_u64(inode_tree_entry, 0),
+            inode_leaf_identities_without_judging(
+                reader,
+                &parse_node_pointer(&inode_tree_entry[14..100]),
+                index_cache,
+            ),
+        );
+    }
+    let mut retired_at_this_txg_of_each_inode_tree = BTreeMap::new();
+    for retirement_tree_entry in entries_of_kind(TREE_KIND_RETIREMENT) {
+        let retirement_tree = RetirementTreeOfTheScan {
+            tree: read_u64(retirement_tree_entry, 0),
+            head_identifier: read_u64(
+                retirement_tree_entry,
+                TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
+            ),
+            containers: retirement_containers_without_judging(
+                reader,
+                &parse_node_pointer(&retirement_tree_entry[14..100]),
+                index_cache,
+                packed_cache,
+            ),
+        };
+        let containers = match &retirement_tree.containers {
+            Ok(containers) => containers,
+            Err(gap) => {
+                for invariant in RETIREMENT_SCAN_INVARIANTS {
+                    judgements.part_of_the_range_not_judged(invariant, gap.reason());
+                }
+                retired_at_this_txg_of_each_inode_tree
+                    .insert(retirement_tree.head_identifier, Err(*gap));
+                continue;
+            }
+        };
+        judge_every_retirement_record_of_a_version_is_well_formed(
+            version,
+            retirement_tree.tree,
+            containers,
+            judgements,
+        );
+        retired_at_this_txg_of_each_inode_tree.insert(
+            retirement_tree.head_identifier,
+            Ok(containers
+                .iter()
+                .flat_map(|container| container.records.iter())
+                .filter(|record| record.death_txg == version.checkpoint_txg)
+                .map(|record| record.retired)
+                .collect::<BTreeSet<ContainerIdentityOfTheScan>>()),
+        );
+        let lineage_chain = match &lineage_chain {
+            Ok(lineage_chain) => lineage_chain,
+            Err(gap) => {
+                for invariant in [
+                    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
+                    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+                ] {
+                    judgements.part_of_the_range_not_judged(invariant, gap.reason());
+                }
+                continue;
+            }
+        };
+        judge_retirement_records_on_the_lineage_of_a_version_are_distinct(
+            &version.name,
+            retirement_tree.tree,
+            containers,
+            lineage_chain,
+            judgements,
+        );
+        // 退役树经头 ID 配同一张表里树 ID 等于它的那一棵 inode 树（同墓碑树，I-9.27）：配不上、或那棵树读不全的，I-9.34 判不了。
+        let paired_leaf_identities: Vec<
+            &Result<BTreeSet<ContainerIdentityOfTheScan>, DeletionScanGap>,
+        > = leaf_identities_of_each_inode_tree
+            .iter()
+            .filter(|(inode_tree, _)| **inode_tree == retirement_tree.head_identifier)
+            .map(|(_, identities)| identities)
+            .collect();
+        match paired_leaf_identities.as_slice() {
+            [Ok(leaf_identities)] => judge_retirement_records_on_the_lineage_name_no_live_leaf(
+                &version.name,
+                retirement_tree.tree,
+                containers,
+                leaf_identities,
+                lineage_chain,
+                judgements,
+            ),
+            [Err(gap)] => judgements.part_of_the_range_not_judged(
+                RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+                gap.reason(),
+            ),
+            [] | [_, _, ..] => judgements.part_of_the_range_not_judged(
+                RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+                DeletionScanGap::RetirementTreeHeadIdentifierNamesNoSingleInodeTree.reason(),
+            ),
+        }
+    }
+    // 有一棵退役树的头 ID 配不上这张表里任何一棵 inode 树（I-9.27 判它）：它的记录算不到哪棵 inode 树头上，这一版每棵 inode 树的
+    // 退役集合都判不了——I-9.35 在这一版报射程里判不了，不按「没有退役记录」判漏记。
+    let some_retirement_tree_names_no_inode_tree = retired_at_this_txg_of_each_inode_tree
+        .keys()
+        .any(|head_identifier| !leaf_identities_of_each_inode_tree.contains_key(head_identifier));
+    if some_retirement_tree_names_no_inode_tree {
+        for inode_tree in leaf_identities_of_each_inode_tree.keys() {
+            retired_at_this_txg_of_each_inode_tree.insert(
+                *inode_tree,
+                Err(DeletionScanGap::RetirementTreeHeadIdentifierNamesNoSingleInodeTree),
+            );
+        }
+    }
+    VersionOfTheRetirementCompleteness {
+        name: version.name.clone(),
+        checkpoint_txg: version.checkpoint_txg,
+        parent_txg,
+        leaf_identities_of_each_inode_tree,
+        retired_at_this_txg_of_each_inode_tree,
+        unreadable: None,
+    }
 }
 
 /// I-9.18（删除队列里的号有谱系上的墓碑）在一版的一棵 inode 树上：经头 ID 配这棵树的 `deleted_inodes`（恰好一条）与墓碑树（恰好一条），
@@ -11259,6 +12121,7 @@ pub fn check_pool_image_given_the_tombstone_container_capacity_knob(
         });
     let accounting_seen = walk.accounting_seen;
     let accounting = walk.accounting.clone();
+    let inode_number_watermark_rows_of_the_newest_root = walk.inode_number_watermark_rows.clone();
     // 别的根只取按最新根指着的实例表仍然有效的：(i, T) 有效 ⟺ 无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti
     // （D23（journal 的角色与格式） 已定项 14 回退段的候选集规则）；被抛弃时间线的根引用的单元由影子账隔离、不在当前账里。
     // 再加一条：txg ≥ 回退下界 F 的生效值（D16（发布语义） 已定项 1 的回退候选集，`effective_rollback_floor`）；F 之下的根引用的单元可以已被回收复用，
@@ -11656,6 +12519,10 @@ pub fn check_pool_image_given_the_tombstone_container_capacity_knob(
         TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
         NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION,
     );
+    judgements.not_applicable(
+        RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION,
+    );
     // I-4.4（意图只描述目标态）：第一版的意图载体没有可装增量的字段，报不适用并写理由（用户 2026-10-05 定）。
     judgements.not_applicable(
         INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
@@ -11706,6 +12573,15 @@ pub fn check_pool_image_given_the_tombstone_container_capacity_knob(
             Err(gap) => judgements.part_of_the_range_not_judged("I-9.6", gap.reason()),
         }
     }
+    // I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上）：挨着 I-9.6，同样只在最新的根上判。
+    judge_inode_number_watermark_steps_of_the_newest_root(
+        reader,
+        &roots[newest_index].2.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
+        newest_txg,
+        &inode_number_watermark_rows_of_the_newest_root,
+        &mut index_node_cache,
+        &mut judgements,
+    );
     if !walk.inode_tree_walked {
         judgements.not_applicable(
             "I-9.12",
diff --git a/crates/singlefs-core/src/admission.rs b/crates/singlefs-core/src/admission.rs
index 5fbc17e..c239ba7 100644
--- a/crates/singlefs-core/src/admission.rs
+++ b/crates/singlefs-core/src/admission.rs
@@ -17,7 +17,8 @@
 //!
 //! 条款给的量：checkpoint 保留池的 ckpt_cost 按一次空发布至多写出的固定点计，一律按最坏情况（D28（挂载期承诺量） 已定项 4，用户 2026-09-27 定：
 //! 分配记录树取几何上整棵树（每块盘单元区在根之下每一层罩到的位置数之和加根，用户同日再定取 K1），中央映射树逐层按可能改的路径数与可能切出来的节点数，
-//! 记账树按每发布的节点数，树表一项，实例表链不进；[`checkpoint_cost_of_the_version_to_build_on`]）；挂载期承诺量暖机那一半的 c_max
+//! 记账树按每发布的节点数（这次发布写完之后那棵树的，inode 号水位台阶行跨过一片叶那次多出的节点算上），树表一项，实例表链不进；
+//! [`checkpoint_cost_of_the_version_to_build_on`]）；挂载期承诺量暖机那一半的 c_max
 //! 与保留池「按同一个现算的 c_max 取」（D28（挂载期承诺量） 已定项 4 末条）；需求 = 这次发布新写的全部槽（普通分配加这次写出的固定点，
 //! 不加这次换下的槽），没有普通分配的发布（空发布、写行、暖机、抬 F）不判（D28（挂载期承诺量） 已定项 1 接线，用户 2026-09-25 定）。
 //!
@@ -36,16 +37,21 @@ use std::collections::BTreeSet;
 use std::num::NonZeroU64;
 
 use singlefs_format::{
-    DATA_UNIT_BYTES, INSTANCE_TABLE_PAGE_RECORDS, ROOT_RING_REGIONS, SLOT_BYTES,
-    TOMBSTONE_CONTAINER_RECORDS,
+    DATA_UNIT_BYTES, INSTANCE_TABLE_PAGE_RECORDS, RETIREMENT_CONTAINER_RECORDS, ROOT_RING_REGIONS,
+    SLOT_BYTES, TOMBSTONE_CONTAINER_RECORDS,
 };
 
-use crate::address::{DataUnitIndexInFile, DeviceIdentity};
+use crate::address::{CheckpointTxg, DataUnitIndexInFile, DeviceIdentity};
 use crate::allocation_record_tree::{span_in_slots_at_level, AllocationRecordTreeGeometry};
 use crate::allocator::PoolAllocator;
-use crate::code_two_tree::{CodeTwoTreeNodeCapacity, CodeTwoTreeShape};
+use crate::code_two_tree::{
+    plan_the_tree_after_this_publish, CodeTwoKeyFieldWidths, CodeTwoTreeKey,
+    CodeTwoTreeNodeCapacity, CodeTwoTreeShape,
+};
 use crate::inode_tree::InodeLeafContainer;
-use crate::records::{NLINK_OF_AN_INODE_BEING_DELETED, STATISTIC_COMMITTED_RESERVATION_BYTES};
+use crate::records::{
+    AccountingEntry, NLINK_OF_AN_INODE_BEING_DELETED, STATISTIC_COMMITTED_RESERVATION_BYTES,
+};
 use crate::root_ring::RootRingSlotsPerRegion;
 use crate::transaction::{
     CodeTwoTreeNodeCapacities, MultiLevelCodeTwoTree, TransactionOutput, TransactionUnit,
@@ -187,6 +193,44 @@ pub fn central_mapping_entries_of_one_lineage_container_cow(
         .expect("谱系那几把装得进 u64")
 }
 
+/// ckpt_cost 里退役容器那一项的固定部分（判决 `research/prompts/m3-retire-r1-main-verification.md` 第六节 R2「两样都做」：
+/// 照谱系在 ckpt_cost 里单列一项）：一个退役容器（32 KiB 码 3，2 块）加退役树根（1 块）；退役树的内部层数另加
+/// （[`checkpoint_cost_of_one_retirement_container_cow`]）。
+pub const RETIREMENT_CONTAINER_BLOCKS_OF_ONE_COW: u64 = DATA_UNIT_BYTES / SLOT_BYTES;
+pub const RETIREMENT_TREE_ROOT_BLOCKS_OF_ONE_COW: u64 = 1;
+
+/// ckpt_cost 里退役容器那一项：一次退役容器 COW 连路径 = 容器 2 + 退役树根 1 + 退役树内部层数（根节点码 2 头里现读的层级减一，
+/// 零根与只有根时 0），同谱系那一项（[`checkpoint_cost_of_one_lineage_container_cow`]）的算法。
+///
+/// # Panics
+/// 加出来装不进 u64：内部层数至多十来层。
+#[must_use]
+pub fn checkpoint_cost_of_one_retirement_container_cow(
+    retirement_tree_internal_levels: u64,
+) -> MetadataBlocks {
+    MetadataBlocks(
+        (RETIREMENT_CONTAINER_BLOCKS_OF_ONE_COW + RETIREMENT_TREE_ROOT_BLOCKS_OF_ONE_COW)
+            .checked_add(retirement_tree_internal_levels)
+            .expect("退役那一项装得进 u64"),
+    )
+}
+
+/// 一次退役容器 COW 连祖先在中央映射树里改的条目数：退役容器、退役树根与每层内部节点各一把 key，= 1 + 1 + 内部层数
+/// （判决第六节 R2「中央映射树删插数加退役容器、退役树根各一把和内部层每层一把」，同谱系那几把）。
+///
+/// # Panics
+/// 加出来装不进 u64：内部层数至多十来层。
+#[must_use]
+pub fn central_mapping_entries_of_one_retirement_container_cow(
+    retirement_tree_internal_levels: u64,
+) -> u64 {
+    const MAPPED_UNITS_OF_THE_RETIREMENT_CONTAINER: u64 = 1;
+    const MAPPED_UNITS_OF_THE_RETIREMENT_TREE_ROOT: u64 = 1;
+    (MAPPED_UNITS_OF_THE_RETIREMENT_CONTAINER + MAPPED_UNITS_OF_THE_RETIREMENT_TREE_ROOT)
+        .checked_add(retirement_tree_internal_levels)
+        .expect("退役那几把装得进 u64")
+}
+
 /// 式子里的「不可回收」：只在 zoned 上非零（活快照钉住的是 zone 粒度、引用按块计，`invariants.md` 逐字要求另立的量）；
 /// 第一版不跑 zoned ⇒ 恒 0。记账树每块盘那一行「不可回收字节」写的就是它（`transaction` 装记账行那一段）。
 pub const UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED: BytesOnOneDevice = BytesOnOneDevice::ZERO;
@@ -231,6 +275,17 @@ pub fn live_objects_of_the_inode_tree(inode_leaf_containers: &[InodeLeafContaine
     u64::try_from(live_records).expect("一棵 inode 树的记录数装得进 u64")
 }
 
+/// n_记录（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条）：一版 inode 树里全部记录数，含 nlink = 0 的
+/// （退役记录写在最后一批，意图到最后一批之间那个对象 n_live 已不含它，退役那一份预付挂 n_live 就在用到之前释放了）。
+#[must_use]
+pub fn inode_records_of_the_inode_tree(inode_leaf_containers: &[InodeLeafContainer]) -> u64 {
+    let records: usize = inode_leaf_containers
+        .iter()
+        .map(|container| container.records.len())
+        .sum();
+    u64::try_from(records).expect("一棵 inode 树的记录数装得进 u64")
+}
+
 /// 墓碑预付里每个活对象、每份副本摊的字节：⌈一个 32 KiB 墓碑容器 ÷ 一容器装的墓碑条数 815⌉ = ⌈32768 ÷ 815⌉ = 41
 /// （`.claude/kb/layout/03-third-txn.md`「记账标签 6 的值」那一行：「41 = ⌈2 × 16384 ÷ 815⌉ 每份副本、每对象先取整再乘 n_live」）。
 pub const TOMBSTONE_PREPAY_BYTES_PER_LIVE_OBJECT_ON_ONE_REPLICA: u64 =
@@ -241,41 +296,65 @@ pub const TOMBSTONE_PREPAY_BYTES_PER_LIVE_OBJECT_ON_ONE_REPLICA: u64 =
 pub const TOMBSTONE_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA: u64 = DATA_UNIT_BYTES / SLOT_BYTES;
 pub const TOMBSTONE_PREPAY_ROOT_SLOTS_ON_ONE_REPLICA: u64 = 1;
 
-/// 算墓碑预付要读的那一版的两个量。
+/// 退役记录预付里每条 inode 记录、每份副本摊的字节：⌈一个 32 KiB 退役容器 ÷ 一容器装的退役记录条数 959⌉ = ⌈32768 ÷ 959⌉ = 35
+/// （判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条）。
+pub const RETIREMENT_PREPAY_BYTES_PER_INODE_RECORD_ON_ONE_REPLICA: u64 =
+    DATA_UNIT_BYTES.div_ceil(RETIREMENT_CONTAINER_RECORDS);
+
+/// 退役记录预付里全池常驻那一份的固定部分（每份副本、以 16 KiB 槽计）：一个开放退役容器（2 槽）加退役树根（1 槽），H_退役另算。
+pub const RETIREMENT_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA: u64 = DATA_UNIT_BYTES / SLOT_BYTES;
+pub const RETIREMENT_PREPAY_ROOT_SLOTS_ON_ONE_REPLICA: u64 = 1;
+
+/// 算墓碑与退役记录预付要读的那一版的四个量。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
-pub enum TombstonePrepayBasis {
+pub enum TombstoneAndRetirementPrepayBasis {
     /// 树表里没有 `deleted_inodes` 条目的版本（第一个文件版本之前、树表 0 条）：标签 6 取 0。
     VersionWithoutADeletedInodesTreeTableEntry,
-    /// 树表里有 `deleted_inodes` 条目的版本：`tombstone_tree_internal_levels` 是现行那一版墓碑树的内部层数 H（从墓碑树根节点码 2 头现读，
-    /// 零根与只有根时 0），`live_objects` 是这一版 inode 树里 nlink ≥ 1 的记录数 n_live。
+    /// 树表里有 `deleted_inodes` 条目的版本：`tombstone_tree_internal_levels`、`retirement_tree_internal_levels` 是现行那一版墓碑树、
+    /// 退役树的内部层数（从各自根节点码 2 头现读，零根与只有根时 0），`live_objects` 是这一版 inode 树里 nlink ≥ 1 的记录数 n_live，
+    /// `inode_records` 是这一版 inode 树里全部记录数 n_记录（含 nlink = 0 的）。
     VersionWithADeletedInodesTreeTableEntry {
         tombstone_tree_internal_levels: u64,
         live_objects: u64,
+        retirement_tree_internal_levels: u64,
+        inode_records: u64,
     },
 }
 
-/// 记账标签 6「已承诺预留」里墓碑那一份（D3（空间分配） 已定项 2，用户定案 F2）：
-/// 副本数 × ((2 + H + 1) × 16384 + 41 × n_live) 字节，按全部副本之和记（C375（待删占用与已承诺预留按份还是按和记没有条款）），
-/// 41 每对象先取整再乘 n_live（[`TOMBSTONE_PREPAY_BYTES_PER_LIVE_OBJECT_ON_ONE_REPLICA`]）；树表里没有 `deleted_inodes` 条目的版本取 0。
+/// 记账标签 6「已承诺预留」（D3（空间分配） 已定项 2，用户定案 F2；退役那一项是判决 `research/prompts/m3-retire-r1-main-verification.md`
+/// 第六节 R2「两样都做」的墓碑式预付）：
+/// 副本数 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录) 字节，按全部副本之和记（C375），
+/// 41、35 每对象先取整再乘（[`TOMBSTONE_PREPAY_BYTES_PER_LIVE_OBJECT_ON_ONE_REPLICA`]、
+/// [`RETIREMENT_PREPAY_BYTES_PER_INODE_RECORD_ON_ONE_REPLICA`]）；树表里没有 `deleted_inodes` 条目的版本取 0。
 ///
 /// # Panics
-/// 乘出来装不进 u64：H 至多十来层、n_live 至多 2^63，单份副本的量远小于 2^64 ÷ 副本数——装不下说明调用方读错了那一版。
+/// 乘出来装不进 u64：H 至多十来层、记录数至多 2^63，单份副本的量远小于 2^64 ÷ 副本数——装不下说明调用方读错了那一版。
 #[must_use]
-pub fn committed_reservation_for_tombstones(
-    basis: TombstonePrepayBasis,
+pub fn committed_reservation_for_tombstones_and_retirement_records(
+    basis: TombstoneAndRetirementPrepayBasis,
     replicas: ReplicaCount,
 ) -> BytesSummedOverAllReplicas {
     match basis {
-        TombstonePrepayBasis::VersionWithoutADeletedInodesTreeTableEntry => {
+        TombstoneAndRetirementPrepayBasis::VersionWithoutADeletedInodesTreeTableEntry => {
             BytesSummedOverAllReplicas::ZERO
         }
-        TombstonePrepayBasis::VersionWithADeletedInodesTreeTableEntry {
+        TombstoneAndRetirementPrepayBasis::VersionWithADeletedInodesTreeTableEntry {
             tombstone_tree_internal_levels,
             live_objects,
+            retirement_tree_internal_levels,
+            inode_records,
         } => {
-            let resident_slots = TOMBSTONE_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA
-                + tombstone_tree_internal_levels
-                + TOMBSTONE_PREPAY_ROOT_SLOTS_ON_ONE_REPLICA;
+            let resident_slots = [
+                TOMBSTONE_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA,
+                tombstone_tree_internal_levels,
+                TOMBSTONE_PREPAY_ROOT_SLOTS_ON_ONE_REPLICA,
+                RETIREMENT_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA,
+                retirement_tree_internal_levels,
+                RETIREMENT_PREPAY_ROOT_SLOTS_ON_ONE_REPLICA,
+            ]
+            .into_iter()
+            .try_fold(0_u64, u64::checked_add)
+            .expect("常驻槽数装得进 u64");
             let bytes_of_one_replica = resident_slots
                 .checked_mul(SLOT_BYTES)
                 .and_then(|resident_bytes| {
@@ -283,7 +362,14 @@ pub fn committed_reservation_for_tombstones(
                         .checked_mul(live_objects)
                         .and_then(|per_object_bytes| resident_bytes.checked_add(per_object_bytes))
                 })
-                .expect("一份副本的墓碑预付装得进 u64");
+                .and_then(|bytes_with_tombstones| {
+                    RETIREMENT_PREPAY_BYTES_PER_INODE_RECORD_ON_ONE_REPLICA
+                        .checked_mul(inode_records)
+                        .and_then(|per_record_bytes| {
+                            bytes_with_tombstones.checked_add(per_record_bytes)
+                        })
+                })
+                .expect("一份副本的墓碑与退役记录预付装得进 u64");
             BytesSummedOverAllReplicas(
                 bytes_of_one_replica
                     .checked_mul(replicas.replicas())
@@ -297,13 +383,20 @@ pub fn committed_reservation_for_tombstones(
 /// 普通分配 16 + 固定点根 4 + 映射树根以下 4 = 24（辩方 `research/prompts/m3-delete-r3-sonnet-output.md` 3.1 节式子那一行的拆法）。
 pub const DELETION_PUBLISH_SLOTS_OUTSIDE_THE_ALLOCATION_RECORD_TREE_AT_MOST: u64 = 24;
 
+/// W 式子里退役树那一项除内部层数之外的部分：一次删除类发布至多跨两片退役容器（一次发布的退役记录 ≤ 134 条 < 959）⇒
+/// 新写的退役单元 ≤ 2 × 2 + 1 + H_退役 + 1 = 6 + H_退役 槽（判决 `research/prompts/m3-retire-r1-main-verification.md` 第二节 K4
+/// 「一次发布至多几条退役记录」那一行与第三节第 6 条，不论 M3 连不连锁；主 agent 由两次抽样的数推的，没量）。
+pub const DELETION_PUBLISH_RETIREMENT_TREE_SLOTS_BEYOND_ITS_INTERNAL_LEVELS_AT_MOST: u64 = 6;
+
 /// W 式子里分配记录树每一层的上界 2·(K + 26)：K 是删除批大小（数据单元个数），这两个常量是式子里的 2 与 26。
 pub const DELETION_PUBLISH_ALLOCATION_RECORD_TREE_LEVEL_BOUND_FACTOR: u64 = 2;
 pub const DELETION_PUBLISH_ALLOCATION_RECORD_TREE_LEVEL_BOUND_UNITS_BEYOND_THE_BATCH: u64 = 26;
 
-/// W：一次删除类发布每盘新占槽数的上界（D28（挂载期承诺量） 已定项 1），W = 24 + Σ_ℓ min(N_ℓ, 2·(K + 26))。
+/// W：一次删除类发布每盘新占槽数的上界（D28（挂载期承诺量） 已定项 1），W = 24 + (6 + H_退役) + Σ_ℓ min(N_ℓ, 2·(K + 26))。
 /// `positions_at_each_level` 是分配记录树根之下每一层全部盘合计罩到的位置数 N_ℓ（层级 0 到根层级 − 1），
-/// `batch_data_units` 是删除批大小 K（产品取 `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`，D8（核心索引结构） 已定项 16 第 2 条）。
+/// `batch_data_units` 是删除批大小 K（产品取 `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`，D8（核心索引结构） 已定项 16 第 2 条），
+/// `retirement_tree_internal_levels` 是要接着发布的那一版退役树的内部层数 H_退役（判决 `research/prompts/m3-retire-r1-main-verification.md`
+/// 第三节第 6 条「W 加 6 + H_退役」）。
 ///
 /// # Panics
 /// 加出来装不进 u64：N_ℓ 由槽号（至多 2^48）算出、K 至多 144，远装得下。
@@ -311,6 +404,7 @@ pub const DELETION_PUBLISH_ALLOCATION_RECORD_TREE_LEVEL_BOUND_UNITS_BEYOND_THE_B
 pub fn slots_one_deletion_publish_newly_takes_on_one_device_at_most(
     positions_at_each_level: &[u64],
     batch_data_units: u64,
+    retirement_tree_internal_levels: u64,
 ) -> u64 {
     let bound_of_one_level = DELETION_PUBLISH_ALLOCATION_RECORD_TREE_LEVEL_BOUND_FACTOR
         * (batch_data_units
@@ -319,9 +413,11 @@ pub fn slots_one_deletion_publish_newly_takes_on_one_device_at_most(
         .iter()
         .map(|positions| (*positions).min(bound_of_one_level))
         .try_fold(
-            DELETION_PUBLISH_SLOTS_OUTSIDE_THE_ALLOCATION_RECORD_TREE_AT_MOST,
+            DELETION_PUBLISH_SLOTS_OUTSIDE_THE_ALLOCATION_RECORD_TREE_AT_MOST
+                + DELETION_PUBLISH_RETIREMENT_TREE_SLOTS_BEYOND_ITS_INTERNAL_LEVELS_AT_MOST,
             u64::checked_add,
         )
+        .and_then(|slots| slots.checked_add(retirement_tree_internal_levels))
         .expect("W 装得进 u64")
 }
 
@@ -329,8 +425,10 @@ pub fn slots_one_deletion_publish_newly_takes_on_one_device_at_most(
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum RingWindowOfTheReading {
     /// 每块盘扣 [`ring_window_on_one_device`]：带树的一版上的发布（第一个文件版本那次起）与可写挂载。
+    /// `retirement_tree_internal_levels` 是要接着的那一版退役树的内部层数（W 里那一项，[`slots_one_deletion_publish_newly_takes_on_one_device_at_most`]）。
     DeductedOnEachDevice {
         slots_per_region: RootRingSlotsPerRegion,
+        retirement_tree_internal_levels: u64,
     },
     /// 不扣：树表 0 条的一版（还没有第一个文件版本）上的可写挂载——环窗口还没起计。
     NotCountedBeforeTheFirstFileVersion,
@@ -350,7 +448,10 @@ impl RingWindowOfTheReading {
             Some(version) if !version.deleted_inodes.is_empty() => {
                 RingWindowOfTheReading::UsableByTheWritableMountWhileTheDeletionQueueIsNotEmpty
             }
-            Some(_) => RingWindowOfTheReading::DeductedOnEachDevice { slots_per_region },
+            Some(version) => RingWindowOfTheReading::DeductedOnEachDevice {
+                slots_per_region,
+                retirement_tree_internal_levels: version.retirement_tree_internal_levels(),
+            },
         }
     }
 
@@ -358,16 +459,18 @@ impl RingWindowOfTheReading {
     #[must_use]
     pub fn on_each_device(self, allocator: &PoolAllocator) -> BytesOnOneDevice {
         match self {
-            RingWindowOfTheReading::DeductedOnEachDevice { slots_per_region } => {
-                ring_window_on_one_device(
-                    allocator,
-                    slots_per_region,
-                    allocator
-                        .deletion_switches()
-                        .batch_size
-                        .data_units_per_batch(),
-                )
-            }
+            RingWindowOfTheReading::DeductedOnEachDevice {
+                slots_per_region,
+                retirement_tree_internal_levels,
+            } => ring_window_on_one_device(
+                allocator,
+                slots_per_region,
+                allocator
+                    .deletion_switches()
+                    .batch_size
+                    .data_units_per_batch(),
+                retirement_tree_internal_levels,
+            ),
             RingWindowOfTheReading::NotCountedBeforeTheFirstFileVersion
             | RingWindowOfTheReading::UsableByTheWritableMountWhileTheDeletionQueueIsNotEmpty => {
                 BytesOnOneDevice::ZERO
@@ -384,12 +487,14 @@ pub fn ring_window_on_one_device(
     allocator: &PoolAllocator,
     slots_per_region: RootRingSlotsPerRegion,
     batch_data_units: u64,
+    retirement_tree_internal_levels: u64,
 ) -> BytesOnOneDevice {
     let root_level = AllocationRecordTreeGeometry::of_allocator(allocator).root_level();
     let slots_one_deletion_publish_takes =
         slots_one_deletion_publish_newly_takes_on_one_device_at_most(
             &allocation_record_tree_positions_at_each_level_below_the_root(root_level, allocator),
             batch_data_units,
+            retirement_tree_internal_levels,
         );
     let roots_in_the_ring_plus_one = ROOT_RING_REGIONS * slots_per_region.count() + 1;
     BytesOnOneDevice::of_slots(
@@ -907,10 +1012,13 @@ impl SpaceAdmission {
 /// - 分配记录树：几何上整棵树（用户同日再定取 K1），1 + Σ_盘 Σ_{层级 L = 0 … 根层级 − 1} 这块盘单元区在层级 L 上罩到的位置数
 ///   （[`allocation_record_tree_positions_over_the_unit_areas`]）；
 /// - 中央映射树：逐层按这次可能改的路径数与可能切出来的节点数计（[`central_mapping_tree_nodes_an_empty_publish_rewrites_at_most`]）；
-/// - 记账树每次发布整批重写（`transaction` 装记账行那一段，条目数每次发布相同、整棵按同一个形状重建）：这一版记账树的节点数；
+/// - 记账树：这次发布写完之后那棵记账树的节点数（[`accounting_tree_nodes_after_the_publish_built_on`]：行数至多是这一版的行数加
+///   这次新留的 1 级 inode 号水位台阶，按写路径同一个规划器、按 `node_capacities` 的记账树容量现算）；
 /// - 树表 1（每次发布重写一个单元，D16（发布语义） 已定项 9）；实例表链不进（它的开销归已定项 3 的切换预留）；
 /// - 谱系容器单列一项，按一次谱系容器 COW 连路径算（[`checkpoint_cost_of_one_lineage_container_cow`]，只算带文件的一版：
-///   树表 0 条的一版没有谱系树，也走不到管理员回退）。
+///   树表 0 条的一版没有谱系树，也走不到管理员回退）；
+/// - 退役容器照谱系单列一项（[`checkpoint_cost_of_one_retirement_container_cow`]，判决 `research/prompts/m3-retire-r1-main-verification.md`
+///   第六节 R2「两样都做」，只算带文件的一版），它在中央映射树里的删插同样算上。
 ///
 /// 「当前」= 这次发布要接在后面的那一版：
 /// - 带文件的一版：分配记录树与中央映射树的高从各自根节点的码 2 头里现读（层级 + 1，不用内存里另存一份，已定项 4）——分配记录树那一项
@@ -939,7 +1047,7 @@ pub fn checkpoint_cost_of_the_version_to_build_on(
     )
 }
 
-/// 同 [`checkpoint_cost_of_the_version_to_build_on`]，中央映射树的节点容量按 `node_capacities` 取：写入口装着只供测试的压小容量时，
+/// 同 [`checkpoint_cost_of_the_version_to_build_on`]，中央映射树与记账树的节点容量按 `node_capacities` 取：写入口装着只供测试的压小容量时，
 /// 它的空发布按压小的容量切分，量它的上界要按同一个容量算。产品路径（发布与可写挂载的准入）按节点格式调上面那一个。
 #[must_use]
 pub fn checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
@@ -962,20 +1070,34 @@ pub fn checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
                 root_level_read_from_the_root_node_header,
                 allocator,
             );
-            let accounting_tree_nodes = u64::try_from(file_version.accounting_tree.node_count())
-                .expect("记账树的节点数装得进 u64");
+            let accounting_tree_nodes_of_this_version =
+                u64::try_from(file_version.accounting_tree.node_count())
+                    .expect("记账树的节点数装得进 u64");
+            let accounting_tree_nodes = accounting_tree_nodes_after_the_publish_built_on(
+                file_version,
+                node_capacities.of_tree(MultiLevelCodeTwoTree::Accounting),
+            );
             let lineage_tree_internal_levels = file_version.lineage_tree_internal_levels();
+            let retirement_tree_internal_levels = file_version.retirement_tree_internal_levels();
             // 一次空发布在中央映射树里改的条目是这两棵树的节点（别的进映射的单元照抄上一版、key 不变）：重写的每个节点删一把旧 key
             // （上一版有它时）、插一把这次的 key。再加回退那次谱系容器 COW 连祖先改的那几把：谱系那一项按一次 COW 连祖先算
             // （D28（挂载期承诺量） 已定项 4），它在中央映射树里的删插一并算上（用户 2026-10-02 定「乙 一并算上」）。
-            let changed_nodes_that_are_mapped = allocation_record_tree_nodes
-                .checked_add(accounting_tree_nodes)
-                .and_then(|nodes| {
-                    nodes.checked_add(central_mapping_entries_of_one_lineage_container_cow(
-                        lineage_tree_internal_levels,
-                    ))
-                })
-                .expect("两棵树重写的节点数与谱系那几把装得进 u64");
+            // 记账树的删至多是这一版的节点数、插至多是这次之后的节点数：两者在台阶行跨过一片叶的那次不同。
+            let mapped_changes_with_accounting_nodes = |accounting_nodes: u64| {
+                allocation_record_tree_nodes
+                    .checked_add(accounting_nodes)
+                    .and_then(|nodes| {
+                        nodes.checked_add(central_mapping_entries_of_one_lineage_container_cow(
+                            lineage_tree_internal_levels,
+                        ))
+                    })
+                    .and_then(|nodes| {
+                        nodes.checked_add(central_mapping_entries_of_one_retirement_container_cow(
+                            retirement_tree_internal_levels,
+                        ))
+                    })
+                    .expect("两棵树重写的节点数与谱系、退役那几把装得进 u64")
+            };
             let central_mapping_tree_nodes =
                 central_mapping_tree_nodes_an_empty_publish_rewrites_at_most(
                     &central_mapping_tree_nodes_at_each_level(
@@ -987,22 +1109,29 @@ pub fn checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
                     u64::try_from(file_version.mapped_units.len())
                         .expect("进映射的单元数装得进 u64"),
                     CentralMappingEntryChangesOfAnEmptyPublish {
-                        deleted_at_most: changed_nodes_that_are_mapped,
-                        inserted_at_most: changed_nodes_that_are_mapped,
+                        deleted_at_most: mapped_changes_with_accounting_nodes(
+                            accounting_tree_nodes_of_this_version,
+                        ),
+                        inserted_at_most: mapped_changes_with_accounting_nodes(
+                            accounting_tree_nodes,
+                        ),
                     },
                     node_capacities.of_tree(MultiLevelCodeTwoTree::CentralMapping),
                 );
             let lineage_container_cow =
                 checkpoint_cost_of_one_lineage_container_cow(lineage_tree_internal_levels).0;
+            let retirement_container_cow =
+                checkpoint_cost_of_one_retirement_container_cow(retirement_tree_internal_levels).0;
             [
                 allocation_record_tree_nodes,
                 central_mapping_tree_nodes,
                 accounting_tree_nodes,
                 lineage_container_cow,
+                retirement_container_cow,
             ]
             .into_iter()
             .try_fold(0u64, u64::checked_add)
-            .expect("三棵树的节点数与谱系那一项加起来装得进 u64")
+            .expect("三棵树的节点数与谱系、退役那两项加起来装得进 u64")
         }
         None => match allocator.allocation_record_tree_of_the_version_without_file() {
             Some(_) => allocation_record_tree_positions_over_the_unit_areas(
@@ -1015,6 +1144,66 @@ pub fn checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
     MetadataBlocks(record_trees_and_accounting_nodes + TREE_TABLE_UNITS_PER_PUBLISH)
 }
 
+/// ckpt_cost 里记账树那一项（D28（挂载期承诺量） 已定项 4「记账树每发布的节点数」）：接在 `version_to_build_on` 后面的那次发布写完之后
+/// 那棵记账树的节点数，记账树节点容量取 `capacity`。
+///
+/// 那次发布的记账行与它做什么无关（空发布、覆盖写、建 inode、管理员回退都一样，`transaction` 装记账行那一段）：池级三行加每块盘六行，
+/// 代段换成那次的 txg，加照抄的 inode 号水位台阶行（[`TransactionOutput::inode_number_watermark_steps`]）。行数因此至多是这一版的行数加 1：
+/// 这一版自己那一行是一级台阶时，下一次照抄它、再写自己那一行（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）。
+/// 形状按写路径同一个规划器从这一版的形状现算（`code_two_tree::plan_the_tree_after_this_publish`）：台阶行正好跨过一片叶的容量那次，
+/// 那次的树比这一版多节点（产品容量两块盘时第 463 级台阶起 +2，同一条射程），只拿这一版的节点数会少算。
+/// 那次的代段只要大于这一版的 txg 就排在同一处，取这一版的 txg + 1（D16（发布语义） 已定项 6）。
+///
+/// 规划器拒（树要长到 257 层，或这一版的形状按分隔 key 走不到它自己叶里的 key）时，接在这一版后面的发布拿同一个形状、同一组 key 规划，
+/// 同样在任何落盘之前拒、一个记账树节点都不写：这一项取这一版的节点数。
+///
+/// # Panics
+/// 这一版的 txg 加 1 装不进 u64；或算出来的行数多于这一版的行数加 1（照抄的台阶行代段都小于这一版的 txg，与这一版那一组行不重 key）。
+#[must_use]
+pub fn accounting_tree_nodes_after_the_publish_built_on(
+    version_to_build_on: &TransactionOutput,
+    capacity: CodeTwoTreeNodeCapacity,
+) -> u64 {
+    let version_txg = version_to_build_on.root.checkpoint_txg;
+    let txg_of_the_publish_built_on_it = CheckpointTxg(
+        version_txg
+            .0
+            .checked_add(1)
+            .expect("checkpoint 号加 1 装得进 u64"),
+    );
+    let rows_written_by_that_publish = version_to_build_on
+        .accounting_entries
+        .iter()
+        .filter(|entry| entry.generation == version_txg)
+        .map(|entry| AccountingEntry {
+            statistic: entry.statistic,
+            tree: entry.tree,
+            device: entry.device,
+            generation: txg_of_the_publish_built_on_it,
+            value: entry.value,
+            sequence: entry.sequence,
+        });
+    let keys_after_that_publish: BTreeSet<CodeTwoTreeKey> = rows_written_by_that_publish
+        .chain(version_to_build_on.inode_number_watermark_steps())
+        .map(|entry| CodeTwoTreeKey::new(&entry.key_bytes(), CodeTwoKeyFieldWidths::ACCOUNTING))
+        .collect();
+    assert!(
+        keys_after_that_publish.len() <= version_to_build_on.accounting_entries.len() + 1,
+        "接在这一版后面的发布至多比这一版多 1 行记账（新留的那一级台阶）：算出 {} 行，这一版 {} 行",
+        keys_after_that_publish.len(),
+        version_to_build_on.accounting_entries.len()
+    );
+    let nodes = match plan_the_tree_after_this_publish(
+        &version_to_build_on.accounting_tree.shape,
+        &keys_after_that_publish,
+        capacity,
+    ) {
+        Ok(plan) => plan.shape.nodes().len(),
+        Err(_refused_before_any_write) => version_to_build_on.accounting_tree.node_count(),
+    };
+    u64::try_from(nodes).expect("记账树的节点数装得进 u64")
+}
+
 /// 分配记录树一次空发布至多重写的节点数，取几何上整棵树（K1，用户 2026-09-27 定；D8（核心索引结构） 已定项 14：按位置寻址、根里按盘分路、
 /// 根之下每个节点只属于一块盘）：
 /// 1（根，罩整个 key 空间）+ Σ_盘 Σ_{层级 L = 0 … 根层级 − 1}（这块盘单元区 [s, e) 在层级 L 上罩到的位置数 = ⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1），
@@ -1241,6 +1430,11 @@ pub const fn space_budget_of_role(role: TransactionUnit) -> SpaceBudgetOfARole {
         | TransactionUnit::TombstoneLeafContainer(_)
         | TransactionUnit::TombstoneRoot
         | TransactionUnit::TombstoneNodeBelowTheRoot(_) => SpaceBudgetOfARole::OrdinaryAllocation,
+        // 退役树三个角色同墓碑（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条）：只在最后一批（身份消失那次）重写，
+        // 落点那一判照判，式子那一判删除类发布整次不判；它们的空间罩在标签 6 的退役那一份与环窗口 W 的 6 + H_退役 里。
+        TransactionUnit::RetirementLeafContainer(_)
+        | TransactionUnit::RetirementRoot
+        | TransactionUnit::RetirementNodeBelowTheRoot(_) => SpaceBudgetOfARole::OrdinaryAllocation,
         // 谱系容器与谱系树根归固定点、进 ckpt_cost 的 Σ 名单（D28（挂载期承诺量） 已定项 1 接线），只在管理员回退那次重写：
         // 回退不判式子，空间从 checkpoint 保留池里出（[`checkpoint_cost_of_one_lineage_container_cow`]）。
         TransactionUnit::LineageLeafContainer(_)
@@ -1369,7 +1563,11 @@ pub fn admission_reading_before_a_publish_on_a_ring_of(
         InstanceRowsOfTheSwitchReserve::OfTheNextWritableMount,
         CodeTwoTreeNodeCapacities::FromTheNodeFormat,
         committed_reservation_after_this_publish,
-        RingWindowOfTheReading::DeductedOnEachDevice { slots_per_region },
+        RingWindowOfTheReading::DeductedOnEachDevice {
+            slots_per_region,
+            retirement_tree_internal_levels: version_to_build_on
+                .map_or(0, TransactionOutput::retirement_tree_internal_levels),
+        },
     )
 }
 
@@ -1680,10 +1878,11 @@ mod tests {
     }
 
     /// 两块 4 GiB 盘（单元区 [50176, 262144)、根层级 2）：N_0 = 2 × (322 − 61 + 1) = 524、N_1 = 2 × 2 = 4，
-    /// 产品批大小 K = 144 ⇒ W = 24 + min(524, 340) + min(4, 340) = 368；S = 8 ⇒ 环窗口 = (3 × 8 + 1) × 368 = 9200 槽 / 盘
-    /// （辩方 `research/prompts/m3-delete-r3-sonnet-output.md` 第三节表里 4 GiB 那一行）。
+    /// 产品批大小 K = 144、退役树只有根（H_退役 = 0）⇒ W = 24 + (6 + 0) + min(524, 340) + min(4, 340) = 374；
+    /// S = 8 ⇒ 环窗口 = (3 × 8 + 1) × 374 = 9350 槽 / 盘（辩方 `research/prompts/m3-delete-r3-sonnet-output.md` 第三节表里 4 GiB 那一行的 368，
+    /// 加判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条的 6 + H_退役）。
     #[test]
-    fn ring_window_on_two_four_gibibyte_devices_is_twenty_five_times_three_hundred_sixty_eight_slots(
+    fn ring_window_on_two_four_gibibyte_devices_is_twenty_five_times_three_hundred_seventy_four_slots(
     ) {
         let allocator = allocator_of_two_devices_of(4 << 30);
         assert_eq!(
@@ -1702,68 +1901,107 @@ mod tests {
         assert_eq!(
             slots_one_deletion_publish_newly_takes_on_one_device_at_most(
                 &[524, 4],
-                EXTENT_TREE_LOWER_LEAF_DATA_UNITS
+                EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
+                0
+            ),
+            374,
+            "W = 24 + 6 + min(524, 340) + min(4, 340)"
+        );
+        assert_eq!(
+            slots_one_deletion_publish_newly_takes_on_one_device_at_most(
+                &[524, 4],
+                EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
+                2
             ),
-            368,
-            "W = 24 + min(524, 340) + min(4, 340)"
+            376,
+            "退役树长到内部两层：W 多 2"
         );
         assert_eq!(
             ring_window_on_one_device(
                 &allocator,
                 RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
-                EXTENT_TREE_LOWER_LEAF_DATA_UNITS
+                EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
+                0
             ),
-            slots(9200),
-            "(3 × 8 + 1) × 368"
+            slots(9350),
+            "(3 × 8 + 1) × 374"
         );
     }
 
-    /// 两块 16 GiB 盘（根层级 2）：N_1 = 2 × 8 = 16，层 0 远超 340 ⇒ W = 24 + 340 + 16 = 380；环窗口 = 25 × 380 = 9500 槽 / 盘
-    /// （辩方第三节表里 16 GiB 那一行）。
+    /// 两块 16 GiB 盘（根层级 2）：N_1 = 2 × 8 = 16，层 0 远超 340 ⇒ W = 24 + 6 + 340 + 16 = 386；环窗口 = 25 × 386 = 9650 槽 / 盘
+    /// （辩方第三节表里 16 GiB 那一行的 380 加退役树那 6 + 0）。
     #[test]
-    fn ring_window_on_two_sixteen_gibibyte_devices_is_twenty_five_times_three_hundred_eighty_slots()
-    {
+    fn ring_window_on_two_sixteen_gibibyte_devices_is_twenty_five_times_three_hundred_eighty_six_slots(
+    ) {
         assert_eq!(
             ring_window_on_one_device(
                 &allocator_of_two_devices_of(16 << 30),
                 RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
-                EXTENT_TREE_LOWER_LEAF_DATA_UNITS
+                EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
+                0
             ),
-            slots(25 * 380)
+            slots(25 * 386)
         );
     }
 
-    /// 墓碑预付（标签 6）在只有根的墓碑树上（H = 0）：两份副本 × ((2 + 0 + 1) × 16384 + 41 × n_live)。
-    /// n_live = 1 ⇒ 2 × (49152 + 41) = 98386；n_live = 3 ⇒ 2 × (49152 + 123) = 98550。
+    /// 标签 6 在只有根的墓碑树与退役树上（H_墓碑 = H_退役 = 0）、记录都是活的（n_记录 = n_live）：
+    /// 两份副本 × ((2 + 0 + 1 + 2 + 0 + 1) × 16384 + 41 × n_live + 35 × n_记录)。
+    /// n = 1 ⇒ 2 × (98304 + 41 + 35) = 196760；n = 3 ⇒ 2 × (98304 + 123 + 105) = 197064。
     #[test]
-    fn tombstone_prepay_on_a_root_only_tombstone_tree_is_three_slots_plus_forty_one_bytes_per_live_object(
+    fn tag_six_on_root_only_tombstone_and_retirement_trees_is_six_slots_plus_forty_one_and_thirty_five_bytes_per_record(
     ) {
-        let prepay_of = |live_objects: u64| {
-            committed_reservation_for_tombstones(
-                TombstonePrepayBasis::VersionWithADeletedInodesTreeTableEntry {
+        let prepay_of = |records: u64| {
+            committed_reservation_for_tombstones_and_retirement_records(
+                TombstoneAndRetirementPrepayBasis::VersionWithADeletedInodesTreeTableEntry {
                     tombstone_tree_internal_levels: 0,
-                    live_objects,
+                    live_objects: records,
+                    retirement_tree_internal_levels: 0,
+                    inode_records: records,
                 },
                 replicas_of_two_devices(),
             )
         };
         assert_eq!(TOMBSTONE_PREPAY_BYTES_PER_LIVE_OBJECT_ON_ONE_REPLICA, 41);
-        assert_eq!(prepay_of(1), BytesSummedOverAllReplicas(98_386));
-        assert_eq!(prepay_of(3), BytesSummedOverAllReplicas(98_550));
+        assert_eq!(
+            RETIREMENT_PREPAY_BYTES_PER_INODE_RECORD_ON_ONE_REPLICA, 35,
+            "⌈32768 ÷ 959⌉"
+        );
+        assert_eq!(prepay_of(1), BytesSummedOverAllReplicas(196_760));
+        assert_eq!(prepay_of(3), BytesSummedOverAllReplicas(197_064));
     }
 
-    /// 墓碑树长高一层之后（H = 1）常驻那一份多一槽：n_live = 1 ⇒ 2 × ((2 + 1 + 1) × 16384 + 41) = 131154。
+    /// 墓碑树长高一层之后（H_墓碑 = 1）常驻那一份多一槽：n = 1 ⇒ 2 × (7 × 16384 + 41 + 35) = 229528。
     #[test]
-    fn tombstone_prepay_after_the_tombstone_tree_grows_one_level_counts_one_more_slot() {
+    fn tag_six_after_the_tombstone_tree_grows_one_level_counts_one_more_slot() {
         assert_eq!(
-            committed_reservation_for_tombstones(
-                TombstonePrepayBasis::VersionWithADeletedInodesTreeTableEntry {
+            committed_reservation_for_tombstones_and_retirement_records(
+                TombstoneAndRetirementPrepayBasis::VersionWithADeletedInodesTreeTableEntry {
                     tombstone_tree_internal_levels: 1,
                     live_objects: 1,
+                    retirement_tree_internal_levels: 0,
+                    inode_records: 1,
                 },
                 replicas_of_two_devices(),
             ),
-            BytesSummedOverAllReplicas(131_154)
+            BytesSummedOverAllReplicas(229_528)
+        );
+    }
+
+    /// 退役那一项数 inode 树里全部记录（含 nlink = 0 的）、跟着退役树的内部层数：意图之后那一版 n_live = 0、n_记录 = 1、
+    /// 退役树长高一层（H_退役 = 1）⇒ 2 × ((2 + 0 + 1 + 2 + 1 + 1) × 16384 + 0 + 35) = 229446。
+    #[test]
+    fn the_retirement_share_of_tag_six_counts_every_inode_record_and_the_retirement_tree_levels() {
+        assert_eq!(
+            committed_reservation_for_tombstones_and_retirement_records(
+                TombstoneAndRetirementPrepayBasis::VersionWithADeletedInodesTreeTableEntry {
+                    tombstone_tree_internal_levels: 0,
+                    live_objects: 0,
+                    retirement_tree_internal_levels: 1,
+                    inode_records: 1,
+                },
+                replicas_of_two_devices(),
+            ),
+            BytesSummedOverAllReplicas(229_446)
         );
     }
 
@@ -1777,12 +2015,26 @@ mod tests {
         );
     }
 
+    /// ckpt_cost 退役那一项（判决 m3-retire-r1 第六节 R2「两样都做」，照谱系）：容器 2 + 根 1 + 内部层数块，零根与只有根 3 块、
+    /// 长高一层 4 块、两层 5 块；它在中央映射树里改的条目 = 容器 1 + 根 1 + 内部层数把。
+    #[test]
+    fn the_retirement_term_of_the_checkpoint_cost_is_a_container_a_root_and_every_internal_level() {
+        assert_eq!(
+            [0, 1, 2].map(checkpoint_cost_of_one_retirement_container_cow),
+            [MetadataBlocks(3), MetadataBlocks(4), MetadataBlocks(5)]
+        );
+        assert_eq!(
+            [0, 1, 2].map(central_mapping_entries_of_one_retirement_container_cow),
+            [2, 3, 4]
+        );
+    }
+
     /// 树表里没有 `deleted_inodes` 条目的版本：标签 6 取 0。
     #[test]
     fn tombstone_prepay_of_a_version_without_a_deleted_inodes_entry_is_zero() {
         assert_eq!(
-            committed_reservation_for_tombstones(
-                TombstonePrepayBasis::VersionWithoutADeletedInodesTreeTableEntry,
+            committed_reservation_for_tombstones_and_retirement_records(
+                TombstoneAndRetirementPrepayBasis::VersionWithoutADeletedInodesTreeTableEntry,
                 replicas_of_two_devices(),
             ),
             BytesSummedOverAllReplicas::ZERO
diff --git a/crates/singlefs-core/src/container_tree.rs b/crates/singlefs-core/src/container_tree.rs
index f873edd..3f2922c 100644
--- a/crates/singlefs-core/src/container_tree.rs
+++ b/crates/singlefs-core/src/container_tree.rs
@@ -1,6 +1,11 @@
-//! 墓碑树与谱系树共用的那一半（D18（块里携带什么信息） 已定项 24 第 1、5 条）：两棵树的根恒为码 2、条目照 inode 树内部节点的
+//! 墓碑树、谱系树与退役树共用的那一半（D18（块里携带什么信息） 已定项 24 第 1、5 条；退役树照墓碑树，判决
+//! `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）：三棵树的根恒为码 2、条目照 inode 树内部节点的
 //! 120 字节形态（分隔 key 容器号 8 + 身份引用 26 + 子指针 86），孩子是码 3 打包记录容器；容器号从 1 起、下一个号 = 树里最大号 + 1、
-//! 零根时 1；容器只增不删。两棵树只差打包记录类型与一片装几条，各自的模块（`crate::tombstone_tree`、`crate::lineage_tree`）把这两样填进来。
+//! 零根时 1；容器只增不删。三棵树只差打包记录类型与一片装几条，各自的模块（`crate::tombstone_tree`、`crate::lineage_tree`、
+//! `crate::retirement_tree`）把这两样填进来。
+//!
+//! 一次发布可以往开放容器里追加不止一条（退役树：一次删除连着吸收几片，[`append_records_to_the_open_container`]）：开放容器装满就开下一片，
+//! 这次重写的码 2 节点是每片重写的容器的祖先的并（[`ContainerTreeShape::ancestors_of_containers`]），只追加一条时它就是最右那一条路径。
 //!
 //! 根满时长高（D18（块里携带什么信息） 已定项 24 第 1、5 条「根满时长高，照 D8（核心索引结构） 已定项 6」，用户定案 K-R3-3）：
 //! 码 2 节点装满时末尾分裂——左半保留全部原有条目、一个字节不动，右半从触发分裂的那条新条目起；根装满时长一层，新根 1 个码 2、
@@ -17,6 +22,7 @@ use singlefs_format::INODE_INTERNAL_ENTRY;
 
 use crate::address::{CheckpointTxg, TreeIdentifier};
 use crate::lineage_tree::LineageContainerCapacity;
+use crate::retirement_tree::RetirementContainerCapacity;
 use crate::tombstone_tree::TombstoneContainerCapacity;
 use crate::unit::{index_node_entry_capacity, PackedIdentity};
 
@@ -109,19 +115,22 @@ impl ContainerTreeNodeEntries {
     }
 }
 
-/// 写入口上墓碑树与谱系树的三个容量开关（都只供测试、只住内存、不落盘）：一片墓碑容器、一片谱系容器各装满几条，一个码 2 节点装满几条。
+/// 写入口上墓碑树、谱系树与退役树的四个容量开关（都只供测试、只住内存、不落盘）：一片墓碑容器、一片谱系容器、一片退役容器各装满几条，
+/// 一个码 2 节点装满几条。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct ContainerTreeCapacities {
     pub tombstone_container: TombstoneContainerCapacity,
     pub lineage_container: LineageContainerCapacity,
+    pub retirement_container: RetirementContainerCapacity,
     pub node_entries: ContainerTreeNodeEntries,
 }
 
 impl ContainerTreeCapacities {
-    /// 产品路径：三样都按格式算。
+    /// 产品路径：四样都按格式算。
     pub const FROM_THE_FORMAT: ContainerTreeCapacities = ContainerTreeCapacities {
         tombstone_container: TombstoneContainerCapacity::FromTheContainerFormat,
         lineage_container: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container: RetirementContainerCapacity::FromTheContainerFormat,
         node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     };
 }
@@ -262,6 +271,48 @@ impl ContainerTreeShape {
         first_child..first_child + nodes[index]
     }
 
+    /// 这几片容器（叶序）的祖先的并，按 bump 次序（层级升序、同层从左到右，根最末）：这几片被重写时 COW 的就是它们（叶 + 全部祖先）。
+    /// 只有最右那一片时就是 [`Self::rightmost_path`]。没有容器时空。
+    ///
+    /// # Panics
+    /// 某个下标不小于容器数：调用方只拿这一版里的容器问。
+    #[must_use]
+    pub fn ancestors_of_containers(&self, containers: &[usize]) -> Vec<ContainerTreeNodePosition> {
+        let mut ancestors: std::collections::BTreeSet<ContainerTreeNodePosition> =
+            std::collections::BTreeSet::new();
+        for container in containers {
+            assert!(
+                *container < self.containers(),
+                "问祖先的容器 {container} 在这一版里（{} 片）",
+                self.containers()
+            );
+            // 迭代上界是层数：每一轮从这一层的孩子下标找出它在这一层的父节点，再拿父节点当上一层的孩子。
+            let mut child = *container;
+            for (level_index, nodes) in self.children_of_each_node_at_each_level.iter().enumerate()
+            {
+                let mut first_child_of_the_node = 0;
+                let parent = nodes
+                    .iter()
+                    .position(|children| {
+                        let holds_it = child < first_child_of_the_node + children;
+                        if !holds_it {
+                            first_child_of_the_node += children;
+                        }
+                        holds_it
+                    })
+                    .expect(
+                        "每一层孩子数之和等于下一层的节点数（构造时判过），孩子下标落在某个节点里",
+                    );
+                ancestors.insert(ContainerTreeNodePosition {
+                    level: u8::try_from(level_index + 1).expect("层数在构造时判过装得进一字节"),
+                    index_in_level: u32::try_from(parent).expect("同层节点数装得进 u32"),
+                });
+                child = parent;
+            }
+        }
+        ancestors.into_iter().collect()
+    }
+
     /// 最右那一条路径上的节点（每一层最右那一个），按 bump 次序（层级 1 先、根最末）：最右那片容器被重写时 COW 的祖先就是它们。
     #[must_use]
     pub fn rightmost_path(&self) -> Vec<ContainerTreeNodePosition> {
@@ -373,14 +424,22 @@ impl<Record> ContainerTreeAfterThisPublish<Record> {
             && self.shape.root_level() > self.shape_before_this_publish.root_level()
     }
 
-    /// 这次重写的码 2 节点，按 bump 次序（层级升序，根最末）。
+    /// 这次长了一层、而上一版的根一个字节不动地留在原位置当根之下的节点（它下面没有容器重写）：那一次新根在上一版里没有落点，
+    /// 上一版的根不被这次换下。一次追加好几条、上一版的开放容器先装满再长层时，旧根下面有容器重写，它跟着重写、照样被换下。
+    #[must_use]
+    pub fn previous_root_is_carried_down_unchanged(&self) -> bool {
+        self.grew_a_level_this_publish()
+            && self
+                .shape_before_this_publish
+                .root_position()
+                .is_some_and(|previous_root| !self.rewritten_nodes().contains(&previous_root))
+    }
+
+    /// 这次重写的码 2 节点，按 bump 次序（层级升序，根最末）：重写的每片容器的全部祖先（COW 叶 + 祖先）。
     #[must_use]
     pub fn rewritten_nodes(&self) -> Vec<ContainerTreeNodePosition> {
-        if self.is_the_root_rewritten() {
-            self.shape.rightmost_path()
-        } else {
-            Vec::new()
-        }
+        self.shape
+            .ancestors_of_containers(&self.rewritten_containers)
     }
 
     /// 这一版每个码 2 节点从哪来，按 bump 次序。
@@ -408,22 +467,51 @@ impl<Record> ContainerTreeAfterThisPublish<Record> {
     }
 }
 
-/// 往容器树的开放容器（最右那一片）追加一条记录：最右那片还装得下（记录数小于 `records_per_container`）就追加进去、形状不变；装满了
+/// 往容器树的开放容器（最右那一片）追加一条记录：[`append_records_to_the_open_container`] 只给一条的那一格。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：上一版的容器与形状、这条记录、txg、树、记录类型、一片装几条、一个节点装几条"
+)]
+#[must_use]
+pub fn append_record_to_the_open_container<Record: Clone>(
+    containers_before_this_publish: &[RecordContainer<Record>],
+    shape_before_this_publish: &ContainerTreeShape,
+    record: Record,
+    checkpoint_txg: CheckpointTxg,
+    tree: TreeIdentifier,
+    record_type: u16,
+    records_per_container: usize,
+    node_entries: ContainerTreeNodeEntries,
+) -> ContainerTreeAfterThisPublish<Record> {
+    append_records_to_the_open_container(
+        containers_before_this_publish,
+        shape_before_this_publish,
+        &[record],
+        checkpoint_txg,
+        tree,
+        record_type,
+        records_per_container,
+        node_entries,
+    )
+}
+
+/// 往容器树的开放容器（最右那一片）按给的次序逐条追加记录：最右那片还装得下（记录数小于 `records_per_container`）就追加进去；装满了
 /// （或树是空的）就开一片新容器：容器号 = 树里最大号 + 1（零根时 1），出生代 = 这次发布的 checkpoint_txg，出生树 = `tree`，
 /// 打包记录类型 = `record_type`，形状照 [`ContainerTreeShape::after_a_container_is_opened_at_the_right_end`] 往上挂（根满长高）。
-/// 别的容器一个字节都不动。
+/// 这次追加过的每一片进重写清单（叶序升序、各一次）；别的容器一个字节都不动。`records` 为空时整棵照抄（[`ContainerTreeAfterThisPublish::carried`]）。
 ///
 /// # Panics
-/// `shape_before_this_publish` 里的容器数与 `containers_before_this_publish` 不等：两样出自同一版。
+/// `shape_before_this_publish` 里的容器数与 `containers_before_this_publish` 不等：两样出自同一版。`records_per_container` 为 0：
+/// 三棵树的容量开关的构造函数都判过至少 1。
 #[allow(
     clippy::too_many_arguments,
-    reason = "参数各是一样东西：上一版的容器与形状、这条记录、txg、树、记录类型、一片装几条、一个节点装几条"
+    reason = "参数各是一样东西：上一版的容器与形状、这几条记录、txg、树、记录类型、一片装几条、一个节点装几条"
 )]
 #[must_use]
-pub fn append_record_to_the_open_container<Record: Clone>(
+pub fn append_records_to_the_open_container<Record: Clone>(
     containers_before_this_publish: &[RecordContainer<Record>],
     shape_before_this_publish: &ContainerTreeShape,
-    record: Record,
+    records: &[Record],
     checkpoint_txg: CheckpointTxg,
     tree: TreeIdentifier,
     record_type: u16,
@@ -435,38 +523,54 @@ pub fn append_record_to_the_open_container<Record: Clone>(
         containers_before_this_publish.len(),
         "上一版容器树的形状与它的容器出自同一版"
     );
+    assert!(
+        records_per_container >= 1,
+        "一片容器至少装一条：容量开关的构造函数判过"
+    );
     let mut containers = containers_before_this_publish.to_vec();
-    let open_container_has_room = containers
-        .last()
-        .is_some_and(|open| open.records.len() < records_per_container);
-    let shape = if open_container_has_room {
-        containers
-            .last_mut()
-            .expect("上一句判过最右那片在")
-            .records
-            .push(record);
-        shape_before_this_publish.clone()
-    } else {
-        // 容器号沿叶序严格递增、从 1 起（0 不用），下一个号 = 树里最大号 + 1：最右那片的号就是最大号。
-        let next_container_number = containers
+    let mut shape = shape_before_this_publish.clone();
+    let mut rewritten_containers: Vec<usize> = Vec::new();
+    // 迭代次数 = 这次追加的条数；跨轮携带的是容器与形状（形状里的容器数恒等于容器数）与升序不重复的重写清单。
+    for record in records {
+        let open_container_has_room = containers
             .last()
-            .map_or(1, |rightmost| rightmost.identity.container + 1);
-        containers.push(RecordContainer {
-            identity: PackedIdentity {
-                birth_tree: tree,
-                record_type,
-                container: next_container_number,
-                container_birth: checkpoint_txg,
-            },
-            records: vec![record],
-        });
-        shape_before_this_publish
-            .after_a_container_is_opened_at_the_right_end(node_entries.entries_per_node())
-    };
-    let rewritten_position = containers.len() - 1;
+            .is_some_and(|open| open.records.len() < records_per_container);
+        if open_container_has_room {
+            containers
+                .last_mut()
+                .expect("上一句判过最右那片在")
+                .records
+                .push(record.clone());
+        } else {
+            // 容器号沿叶序严格递增、从 1 起（0 不用），下一个号 = 树里最大号 + 1：最右那片的号就是最大号。
+            let next_container_number = containers
+                .last()
+                .map_or(1, |rightmost| rightmost.identity.container + 1);
+            containers.push(RecordContainer {
+                identity: PackedIdentity {
+                    birth_tree: tree,
+                    record_type,
+                    container: next_container_number,
+                    container_birth: checkpoint_txg,
+                },
+                records: vec![record.clone()],
+            });
+            shape =
+                shape.after_a_container_is_opened_at_the_right_end(node_entries.entries_per_node());
+        }
+        let rewritten_position = containers.len() - 1;
+        if rewritten_containers.last() != Some(&rewritten_position) {
+            rewritten_containers.push(rewritten_position);
+        }
+        assert_eq!(
+            shape.containers(),
+            containers.len(),
+            "形状里层级 1 的孩子数之和就是容器数"
+        );
+    }
     ContainerTreeAfterThisPublish {
         containers,
-        rewritten_containers: vec![rewritten_position],
+        rewritten_containers,
         shape,
         shape_before_this_publish: shape_before_this_publish.clone(),
     }
@@ -604,6 +708,83 @@ mod tests {
         );
     }
 
+    /// 一次追加三条、一片装两条、扇出压到 2：上一版两片（第二片装着一条、还开着）。第一条装满第二片，第二、三条开第三片（这时根满长一层）。
+    /// 这次重写的是第二、三片与它们的全部祖先：旧根 (1, 0) 下面的第二片被重写，旧根跟着重写、照样被这次换下。
+    #[test]
+    fn appending_several_records_rewrites_every_touched_container_and_all_their_ancestors() {
+        let before_shape = shape_after_opening(2, 2);
+        let identity = |number: u64| PackedIdentity {
+            birth_tree: TreeIdentifier(65548),
+            record_type: 3,
+            container: number,
+            container_birth: CheckpointTxg(5),
+        };
+        let before = vec![
+            RecordContainer {
+                identity: identity(1),
+                records: vec![10_u64, 11],
+            },
+            RecordContainer {
+                identity: identity(2),
+                records: vec![12_u64],
+            },
+        ];
+        let after = append_records_to_the_open_container(
+            &before,
+            &before_shape,
+            &[13_u64, 14, 15],
+            CheckpointTxg(9),
+            TreeIdentifier(65548),
+            3,
+            2,
+            capped(2),
+        );
+        assert_eq!(after.containers.len(), 3);
+        assert_eq!(after.containers[0], before[0], "装满了的第一片原样");
+        assert_eq!(after.containers[1].records, vec![12, 13], "第二片装满");
+        assert_eq!(
+            after.containers[2].identity,
+            PackedIdentity {
+                birth_tree: TreeIdentifier(65548),
+                record_type: 3,
+                container: 3,
+                container_birth: CheckpointTxg(9),
+            },
+            "第三片：容器号 = 最大号 + 1，出生代 = 这次的 txg"
+        );
+        assert_eq!(after.containers[2].records, vec![14, 15]);
+        assert_eq!(after.rewritten_containers, vec![1, 2]);
+        assert!(after.grew_a_level_this_publish());
+        assert!(
+            !after.previous_root_is_carried_down_unchanged(),
+            "旧根下面的第二片重写了：旧根跟着重写"
+        );
+        assert_eq!(
+            after.rewritten_nodes(),
+            vec![position(1, 0), position(1, 1), position(2, 0)]
+        );
+        assert_eq!(
+            after.shape.ancestors_of_containers(&[2]),
+            after.shape.rightmost_path(),
+            "只看最右那一片时祖先就是最右那一条路径"
+        );
+        let carried = append_records_to_the_open_container(
+            &after.containers,
+            &after.shape,
+            &[],
+            CheckpointTxg(10),
+            TreeIdentifier(65548),
+            3,
+            2,
+            capped(2),
+        );
+        assert!(carried.rewritten_containers.is_empty() && carried.rewritten_nodes().is_empty());
+        assert_eq!(
+            carried.containers, after.containers,
+            "一条都不追加时整棵照抄"
+        );
+    }
+
     /// 只供测试的扇出至少 2、至多产品扇出 135。
     #[test]
     fn the_node_entries_capped_for_tests_lie_between_two_and_the_product_fanout() {
diff --git a/crates/singlefs-core/src/inode_tree.rs b/crates/singlefs-core/src/inode_tree.rs
index f36212d..55a7a6a 100644
--- a/crates/singlefs-core/src/inode_tree.rs
+++ b/crates/singlefs-core/src/inode_tree.rs
@@ -13,9 +13,17 @@
 //! 这个模块就是那个能判的地方：它只算「这次发布之后树里是哪几片容器、各装哪些记录、哪几片要重写」，
 //! 不发一个写、不动分配器、不碰落点，所以三条纪律在这里是纯函数的后置条件，单测判得了。
 //!
-//! **删除只有一种落法**（[`remove_record_from_leaf_containers`]，unlink 最后一批那次发布）：删掉那一条；那一片删空了就整片不落盘
-//! （「记录数为 0 的类型 2 容器不落盘」），它右边的每一片位置左移一格、这次全部重写。别的合并（左从右借、两片都不空的左吸收右）
-//! 没写——合并时机是写路径策略（D8（核心索引结构） 已定项 6），这一版只做「删空就不落盘」这一种。
+//! **删除的落法**（[`remove_record_from_leaf_containers`]，unlink 最后一批那次发布）：删掉那一条；之后照 E176（inode 树叶合并三候选的容量与写代价）
+//! M3 那一臂的写路径策略合并（D8（核心索引结构） 已定项 6「左吸收右与左从右借两样都做」，合并时机是写路径策略）：
+//! - 那一片删空了就整片不落盘（「记录数为 0 的类型 2 容器不落盘」），它的左右邻都在时从左邻起按「先查右邻、再查左邻，两片之和 ≤ 233 就左吸收右」
+//!   一直并到没有一对装得下；
+//! - 没删空就从它起照同一条规则并；一次都没并、它的记录数 < 117（⌈233 ÷ 2⌉）、它有右邻时，从右邻借最小的 ⌊(右 − 左) ÷ 2⌋ 条
+//!   （均分：借完两片条数相差不超过 1），右片保留身份、分隔 key 改成它这一版的最小 key（N1），再从右片起照同一条规则并。
+//!
+//! 合并保留左半身份；被吸收的右半与删空的那一片身份消失，交回给发布路径写退役记录（[`InodeLeafContainersAfterThisPublish::retired`]）。
+//! **只重写内容变了的叶**：身份消失的那几片右边的每一片位置左移，但内容、身份、分隔 key 都没变，这次照抄上一版那一份（落点、字节、指针都不动），
+//! 只是在这一版里换到左边的叶序上（[`InodeLeafContainerOrigin::CarriedFrom`]）——删空最左一片那次新写的槽数不随右边片数涨
+//! （判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 7 条）。
 
 use singlefs_format::{INODE_INTERNAL_ENTRY, INODE_LEAF_RECORDS};
 
@@ -26,8 +34,9 @@ use crate::unit::{index_node_entry_capacity, PackedIdentity, PACKED_TYPE_INODE};
 /// inode 树里第几片叶容器，按叶序（= key 升序）从 0 数。
 ///
 /// 它是**树里的位置**，不是容器号：容器号是首次插入它的那条记录的 inode 号（D8（核心索引结构） 已定项 6），
-/// 位置随左边有没有别的容器而变，两者不是一个量。新容器只在最右端出现；一片删空了不落盘时它右边的每一片左移一格，
-/// 那一次发布把挪了位置的几片全部重写（[`remove_record_from_leaf_containers`]），所以位置照样可以当发布路径里的角色序号用。
+/// 位置随左边有没有别的容器而变，两者不是一个量。新容器只在最右端出现；一片删空或被吸收时它右边的每一片左移，
+/// 那一次发布照抄挪了位置的几片、把它们换到这一版的叶序上（[`InodeLeafContainerOrigin::CarriedFrom`] 记着上一版的叶序），
+/// 所以位置照样当这一版里的角色序号用，照抄时按上一版的叶序取上一版那一份。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub struct InodeLeafContainerIndexInTree(pub u32);
 
@@ -48,10 +57,14 @@ impl InodeLeafContainerIndexInTree {
     }
 }
 
-/// 一片 inode 叶容器这一版装的东西：身份四段（出生树、打包记录类型、容器号、容器出生代）与它的记录，记录按 inode 号升序。
+/// 一片 inode 叶容器这一版装的东西：身份四段（出生树、打包记录类型、容器号、容器出生代）、它在父节点条目里的分隔 key、
+/// 它的记录，记录按 inode 号升序。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct InodeLeafContainer {
     pub identity: PackedIdentity,
+    /// 父节点里这片容器那一条条目的分隔 key（D8（核心索引结构） 已定项 6「分隔 key = 该孩子创建时的最小 key，左从右借的那次发布改成
+    /// 被借的右孩子这一版的最小 key」）：诞生时是容器号，只有被左邻借走最小那几条的那次发布改成它这一版的最小 key，之后照抄。
+    pub separator_key_in_the_parent_entry: u64,
     pub records: Vec<InodeRecord>,
 }
 
@@ -74,13 +87,13 @@ impl InodeLeafContainer {
         self.records.last().map(|record| InodeNumber(record.inode))
     }
 
-    /// 这片容器在父节点里的分隔 key：D8（核心索引结构） 已定项 6 逐字「**分隔 key** = 该孩子创建时的最小 key」。
-    /// 创建时的最小 key 就是容器号——第一个容器的号 = 第一条记录的 inode 号，分裂出来的右半容器号 = 触发分裂那条新记录的 inode 号，
-    /// 两者都是那片容器诞生那一刻装的第一条记录。**不取今天的最小 key**：容器被 COW 重写、记录增删之后
-    /// 今天的最小 key 可以比诞生时大，而分隔 key 不许跟着变（跟着变就等于改父节点里的路由）。
+    /// 这片容器在父节点里的分隔 key：D8（核心索引结构） 已定项 6 逐字「**分隔 key** = 该孩子创建时的最小 key，左从右借的那次发布改成
+    /// 被借的右孩子这一版的最小 key」。创建时的最小 key 就是容器号——第一个容器的号 = 第一条记录的 inode 号，分裂出来的右半容器号 =
+    /// 触发分裂那条新记录的 inode 号。**别的发布不取今天的最小 key**：容器被 COW 重写、记录增删之后今天的最小 key 可以比上一次定下的大，
+    /// 而分隔 key 不许跟着变（跟着变就等于改父节点里的路由）；只有被借的那一次随借走的那几条改（[`remove_record_from_leaf_containers`]）。
     #[must_use]
     pub fn separator_key(&self) -> u64 {
-        self.identity.container
+        self.separator_key_in_the_parent_entry
     }
 
     /// 这片容器还能再装几条：一容器 233 条（D8（核心索引结构） 已定项 6，`INODE_LEAF_RECORDS`）。
@@ -110,29 +123,79 @@ pub fn leaf_containers_one_root_node_holds() -> usize {
     index_node_entry_capacity(8, usize::try_from(INODE_INTERNAL_ENTRY).expect("120"))
 }
 
-/// 这次发布之后 inode 树的叶容器，以及这次要重写哪几片。
+/// 这一版的一片叶容器从哪来。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum InodeLeafContainerOrigin {
+    /// 这次重写（内容、或分隔 key 变了，或是新分裂出来的）。
+    RewrittenThisPublish,
+    /// 上一版那一片一个字节不动，带的是它在上一版里的叶序（左边有片身份消失时比这一版的大）。
+    CarriedFrom(InodeLeafContainerIndexInTree),
+}
+
+/// 这次身份消失的一片叶容器（删空摘掉，或被左邻吸收）：它在上一版里的叶序与它的身份。发布路径给它的身份写一条退役记录、
+/// 按上一版的叶序换下它那一份（D8（核心索引结构） 已定项 6「死亡分三层」第 ② 层）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct RetiredInodeLeafContainer {
+    pub previous_index: InodeLeafContainerIndexInTree,
+    pub identity: PackedIdentity,
+}
+
+/// 这次发布之后 inode 树的叶容器，以及这次要重写哪几片、每片从哪来、哪几片身份消失。
 ///
 /// 没被点到的那几片一个字节都不变（分裂时的左半就是这样：它的记录与身份都没动 ⇒ 不进这次的重写清单，
-/// 连 COW 都不做），调用方照抄上一版的落点与指针。
+/// 连 COW 都不做），调用方按 `origins` 里上一版的叶序照抄上一版的落点与指针。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct InodeLeafContainersAfterThisPublish {
     /// 左起按 key 序的全部叶容器。
     pub containers: Vec<InodeLeafContainer>,
-    /// 这次要重写的那几片，按叶序升序。
+    /// 这次要重写的那几片，按叶序升序（就是 `origins` 里 `RewrittenThisPublish` 的那几个叶序）。
     pub rewritten: Vec<InodeLeafContainerIndexInTree>,
-    /// 这次删空、不再落盘的片数（只有 [`remove_record_from_leaf_containers`] 删空一片时是 1，别的落法都是 0）：
+    /// 每片从哪来，与 `containers` 一一对应。
+    pub origins: Vec<InodeLeafContainerOrigin>,
+    /// 这次身份消失的那几片，按它们在上一版里的叶序升序（只有 [`remove_record_from_leaf_containers`] 会有）：
     /// 删空的是最右那一片时重写清单是空的，根照样要重写（少了一条条目）。
-    pub containers_dropped: usize,
+    pub retired: Vec<RetiredInodeLeafContainer>,
 }
 
 impl InodeLeafContainersAfterThisPublish {
-    /// 这次要不要重写 inode 树根：有叶容器重写（COW 叶 + 全部祖先，D23（journal 的角色与格式） 已定项 1 甲），或有一片删空不再落盘。
+    /// 这次要不要重写 inode 树根：有叶容器重写（COW 叶 + 全部祖先，D23（journal 的角色与格式） 已定项 1 甲），或有一片身份消失、不再落盘。
     #[must_use]
     pub fn is_the_root_rewritten(&self) -> bool {
-        !self.rewritten.is_empty() || self.containers_dropped > 0
+        !self.rewritten.is_empty() || !self.retired.is_empty()
+    }
+
+    /// 这次身份消失的那几片的身份，按上一版的叶序：这次发布的退役集合（上一版的叶身份集 − 这一版的叶身份集）。
+    #[must_use]
+    pub fn retired_identities(&self) -> Vec<PackedIdentity> {
+        self.retired
+            .iter()
+            .map(|retired| retired.identity)
+            .collect()
+    }
+
+    /// 这一版第 `index` 片照抄的话，它在上一版里的叶序；这次重写的交回 `None`。
+    ///
+    /// # Panics
+    /// `index` 不在这一版里：调用方只拿这一版的叶序问。
+    #[must_use]
+    pub fn previous_index_of_a_carried_container(
+        &self,
+        index: InodeLeafContainerIndexInTree,
+    ) -> Option<InodeLeafContainerIndexInTree> {
+        match self.origins[index.position()] {
+            InodeLeafContainerOrigin::RewrittenThisPublish => None,
+            InodeLeafContainerOrigin::CarriedFrom(previous_index) => Some(previous_index),
+        }
     }
 }
 
+/// 左从右借的下限：一片叶删完一条之后记录数少于它、这次又没并过、它有右邻，就从右邻借（E176 M3 那一臂的主取样点
+/// f = ⌈233 ÷ 2⌉ = 117，`research/e7-index-bench/src/bin/e176_inode_leaf_merge_capacity_and_rewrites.rs` 的 `BORROW_FLOOR_MAIN`）。
+#[must_use]
+pub fn records_below_which_a_leaf_borrows_from_its_right_neighbour() -> usize {
+    usize::try_from(INODE_LEAF_RECORDS.div_ceil(2)).expect("117")
+}
+
 /// 往 inode 树里写记录时走得到、而条款没写的那几格。三样都在**任何落盘动作之前**交回，盘上逐字节不变。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum InodeTreeWriteRefusal {
@@ -244,6 +307,7 @@ pub fn write_records_into_leaf_containers(
                                 container: inode.0,
                                 container_birth: checkpoint_txg,
                             },
+                            separator_key_in_the_parent_entry: inode.0,
                             records: vec![*record],
                         });
                         note_rewritten(
@@ -264,19 +328,31 @@ pub fn write_records_into_leaf_containers(
             },
         );
     }
+    let origins = (0..containers.len())
+        .map(|position| {
+            let index = InodeLeafContainerIndexInTree::of_position(position);
+            if rewritten.binary_search(&index).is_ok() {
+                InodeLeafContainerOrigin::RewrittenThisPublish
+            } else {
+                InodeLeafContainerOrigin::CarriedFrom(index)
+            }
+        })
+        .collect();
     Ok(InodeLeafContainersAfterThisPublish {
         containers,
         rewritten,
-        containers_dropped: 0,
+        origins,
+        retired: Vec::new(),
     })
 }
 
-/// 从 inode 树的叶容器里删掉 `inode` 那一条记录（unlink 最后一批那次发布：D8（核心索引结构） 已定项 16 第 2 条「删 inode 记录」）。
+/// 从 inode 树的叶容器里删掉 `inode` 那一条记录（unlink 最后一批那次发布：D8（核心索引结构） 已定项 16 第 2 条「删 inode 记录」），
+/// 再照 E176 M3 那一臂合并（文件头那几条）。
 ///
-/// - 那一片删完还有记录 ⇒ 只有它进重写清单，身份不变（COW 重写身份不变，D8 已定项 6）；
-/// - 那一片删空了 ⇒ 整片不落盘（「记录数为 0 的类型 2 容器不落盘」，D8 已定项 6：左吸收右的退化形——右边那片是空的），
-///   它右边的每一片在树里的位置左移一格。位置是发布路径里的角色序号，挪了位置的那几片这次全部重写：内容与身份都不变，
-///   只是换一份落点（合法的 COW）；它们上一版的落点与删空那一片的一起换下。删空的是最右那一片时重写清单是空的、根照样重写。
+/// - 删完没并、没借、没删空 ⇒ 只有那一片进重写清单，身份不变（COW 重写身份不变，D8 已定项 6）；
+/// - 那一片删空了 ⇒ 整片不落盘（「记录数为 0 的类型 2 容器不落盘」），身份进 `retired`；左吸收右的那一对，左片进重写清单、右片身份进 `retired`；
+///   左从右借的那一对两片都进重写清单、身份都不变，右片的分隔 key 改成它这一版的最小 key；
+/// - 身份消失的那几片右边的每一片位置左移，内容没变的照抄（`origins` 记着上一版的叶序），不进重写清单。
 ///
 /// 删的是树里最后一条时交回零片容器：inode 树写零条目根（key 区间 `[0, 0]`，D8 已定项 6），那一版是「有树、文件集合为空」
 /// （D8 已定项 16 第 5 条），怎么装它归发布路径。
@@ -291,29 +367,182 @@ pub fn remove_record_from_leaf_containers(
     else {
         return Err(InodeTreeWriteRefusal::RecordToRemoveIsNotInTheTree { inode });
     };
-    let mut containers = containers_before_this_publish.to_vec();
-    let container = &mut containers[index.position()];
-    let position_in_container = container
+    let mut tree =
+        LeafContainersBeingMerged::from_the_previous_version(containers_before_this_publish);
+    let holding_position = index.position();
+    let holding = &mut tree.containers[holding_position];
+    let position_in_container = holding
+        .contents
         .records
         .binary_search_by_key(&inode.0, |existing_record| existing_record.inode)
         .expect("position_of_the_container_holding 找到的就是这条记录所在的容器");
-    container.records.remove(position_in_container);
-    if !container.records.is_empty() {
-        return Ok(InodeLeafContainersAfterThisPublish {
-            containers,
-            rewritten: vec![index],
-            containers_dropped: 0,
+    holding.contents.records.remove(position_in_container);
+    holding.is_changed_this_publish = true;
+    if holding.contents.records.is_empty() {
+        tree.retire(holding_position);
+        let has_new_neighbours = holding_position > 0 && holding_position < tree.containers.len();
+        if has_new_neighbours {
+            tree.absorb_until_no_pair_fits(holding_position - 1);
+        }
+    } else {
+        let absorbed_at_least_once = tree.absorb_until_no_pair_fits(holding_position);
+        if !absorbed_at_least_once {
+            tree.borrow_from_the_right_neighbour_if_below_the_floor(holding_position);
+        }
+    }
+    Ok(tree.after_this_publish())
+}
+
+/// 一次删除发布里合并到一半的叶容器：每片带着它在上一版里的叶序与这次有没有改过；身份消失的那几片挪进 `retired`。
+struct LeafContainersBeingMerged {
+    containers: Vec<LeafContainerBeingMerged>,
+    retired: Vec<RetiredInodeLeafContainer>,
+}
+
+struct LeafContainerBeingMerged {
+    contents: InodeLeafContainer,
+    previous_index: InodeLeafContainerIndexInTree,
+    is_changed_this_publish: bool,
+}
+
+impl LeafContainersBeingMerged {
+    fn from_the_previous_version(containers: &[InodeLeafContainer]) -> Self {
+        Self {
+            containers: containers
+                .iter()
+                .enumerate()
+                .map(|(position, contents)| LeafContainerBeingMerged {
+                    contents: contents.clone(),
+                    previous_index: InodeLeafContainerIndexInTree::of_position(position),
+                    is_changed_this_publish: false,
+                })
+                .collect(),
+            retired: Vec::new(),
+        }
+    }
+
+    fn records_at(&self, position: usize) -> usize {
+        self.containers[position].contents.records.len()
+    }
+
+    /// 这一对（`left_position` 与它右邻）合起来装得下一片（≤ 233 条）。
+    fn pair_fits_in_one_container(&self, left_position: usize) -> bool {
+        u64::try_from(self.records_at(left_position) + self.records_at(left_position + 1))
+            .expect("两片之和至多 466 条")
+            <= INODE_LEAF_RECORDS
+    }
+
+    /// 那一片的身份消失（删空摘掉或被吸收）：从这一版拿掉，身份连同上一版的叶序进 `retired`。
+    fn retire(&mut self, position: usize) -> InodeLeafContainer {
+        let retired = self.containers.remove(position);
+        self.retired.push(RetiredInodeLeafContainer {
+            previous_index: retired.previous_index,
+            identity: retired.contents.identity,
         });
+        retired.contents
+    }
+
+    /// 左片吸收它右邻：左片保留身份与分隔 key，右片的记录接在后面，右片身份消失。
+    fn absorb_the_right_neighbour(&mut self, left_position: usize) {
+        let absorbed = self.retire(left_position + 1);
+        let survivor = &mut self.containers[left_position];
+        survivor.contents.records.extend(absorbed.records);
+        survivor.is_changed_this_publish = true;
+    }
+
+    /// 从 `start_position` 那片起先查它与右邻、再查左邻与它，两片之和 ≤ 233 就左吸收右；并完从幸存那片接着查，直到没有一对装得下
+    /// （E176 M3 主取样点「先查右边」）。交回这一趟有没有并过。
+    ///
+    /// 迭代上界是片数：每一轮要么并掉一片、要么退出；跨轮携带的是「当前那片」的位置，它只往左挪（左邻吸收它时换成左邻），
+    /// 被摘掉的总在它右边或就是它被左邻吸收，所以位置一直指着同一片幸存的叶。
+    fn absorb_until_no_pair_fits(&mut self, start_position: usize) -> bool {
+        let mut current_position = start_position;
+        let mut absorbed_at_least_once = false;
+        loop {
+            let right_pair_fits = current_position + 1 < self.containers.len()
+                && self.pair_fits_in_one_container(current_position);
+            let left_pair_fits =
+                current_position > 0 && self.pair_fits_in_one_container(current_position - 1);
+            if right_pair_fits {
+                self.absorb_the_right_neighbour(current_position);
+            } else if left_pair_fits {
+                self.absorb_the_right_neighbour(current_position - 1);
+                current_position -= 1;
+            } else {
+                return absorbed_at_least_once;
+            }
+            absorbed_at_least_once = true;
+        }
+    }
+
+    /// 左从右借（D8（核心索引结构） 已定项 6；E176 M3）：`borrower_position` 那片记录数小于下限且有右邻时，把右邻最小的
+    /// ⌊(右 − 左) ÷ 2⌋ 条挪到它末尾（均分：借完两片条数相差不超过 1），两片都保留身份，右邻的分隔 key 改成它这一版的最小 key（N1）；
+    /// 借完从右邻起照 [`Self::absorb_until_no_pair_fits`] 并。它是最右那片时不借（右从左借会破「记录号 ≥ 容器号」）。
+    ///
+    /// # Panics
+    /// 要借的条数不在 [1, 右邻记录数) 里：调用方只在「从这片起一次都没并过」之后调它，那时它与右邻之和 > 233，
+    /// 它又少于 117 条 ⇒ 右邻多于 116 条、借的条数 ⌊(右 − 左) ÷ 2⌋ ≥ 117 − 左 ≥ 1，且小于右邻的条数。
+    fn borrow_from_the_right_neighbour_if_below_the_floor(&mut self, borrower_position: usize) {
+        let lender_position = borrower_position + 1;
+        if lender_position >= self.containers.len() {
+            return;
+        }
+        let records_in_the_borrower = self.records_at(borrower_position);
+        if records_in_the_borrower >= records_below_which_a_leaf_borrows_from_its_right_neighbour()
+        {
+            return;
+        }
+        let records_in_the_lender = self.records_at(lender_position);
+        let moved_count = records_in_the_lender
+            .checked_sub(records_in_the_borrower)
+            .map_or(0, |difference| difference / 2);
+        assert!(
+            moved_count >= 1 && moved_count < records_in_the_lender,
+            "借的条数要在 [1, 右邻记录数) 里：借方 {records_in_the_borrower} 条、右邻 {records_in_the_lender} 条、借 {moved_count} 条\
+             （这一对没并成说明两片之和 > 233，借方又少于 117 条）"
+        );
+        let moved: Vec<InodeRecord> = self.containers[lender_position]
+            .contents
+            .records
+            .drain(..moved_count)
+            .collect();
+        let lender = &mut self.containers[lender_position];
+        lender.contents.separator_key_in_the_parent_entry = lender
+            .contents
+            .records
+            .first()
+            .map(|record| record.inode)
+            .expect("右邻借出的条数少于它的记录数：借完至少还剩一条");
+        lender.is_changed_this_publish = true;
+        let borrower = &mut self.containers[borrower_position];
+        borrower.contents.records.extend(moved);
+        borrower.is_changed_this_publish = true;
+        self.absorb_until_no_pair_fits(lender_position);
+    }
+
+    fn after_this_publish(mut self) -> InodeLeafContainersAfterThisPublish {
+        self.retired.sort_by_key(|retired| retired.previous_index);
+        let mut rewritten = Vec::new();
+        let mut origins = Vec::with_capacity(self.containers.len());
+        let mut containers = Vec::with_capacity(self.containers.len());
+        for (position, container) in self.containers.into_iter().enumerate() {
+            if container.is_changed_this_publish {
+                rewritten.push(InodeLeafContainerIndexInTree::of_position(position));
+                origins.push(InodeLeafContainerOrigin::RewrittenThisPublish);
+            } else {
+                origins.push(InodeLeafContainerOrigin::CarriedFrom(
+                    container.previous_index,
+                ));
+            }
+            containers.push(container.contents);
+        }
+        InodeLeafContainersAfterThisPublish {
+            containers,
+            rewritten,
+            origins,
+            retired: self.retired,
+        }
     }
-    containers.remove(index.position());
-    let rewritten = (index.position()..containers.len())
-        .map(InodeLeafContainerIndexInTree::of_position)
-        .collect();
-    Ok(InodeLeafContainersAfterThisPublish {
-        containers,
-        rewritten,
-        containers_dropped: 1,
-    })
 }
 
 /// 哪一片容器装着这个 inode 号；一条都没装着是 `None`。
@@ -725,17 +954,53 @@ mod tests {
         after.containers
     }
 
-    /// 删一条、那一片还有记录：只重写那一片，身份不变，别的片不动，没有删空的片。
+    /// `full_containers` 片装满 233 条的叶容器（inode 1 起连号；容器号 = 每片第一条的号、分隔 key 同容器号）。
+    fn full_containers(full_containers: u64) -> Vec<InodeLeafContainer> {
+        let after = write_records_into_leaf_containers(
+            &[],
+            &(1..=233 * full_containers)
+                .map(|inode| record(inode, 0, 3))
+                .collect::<Vec<_>>(),
+            TREE_BIRTH_TXG,
+            INODE_TREE,
+        )
+        .expect("满的几片");
+        assert_eq!(
+            after.containers.len(),
+            usize::try_from(full_containers).expect("片数")
+        );
+        after.containers
+    }
+
+    /// 把第 `position` 片的记录留成按号升序的前 `kept` 条（直接在内容上改：这里判的是删一条之后的落法，不是怎么删到这个样子）。
+    fn keep_the_first_records(containers: &mut [InodeLeafContainer], position: usize, kept: usize) {
+        containers[position].records.truncate(kept);
+    }
+
+    fn container_numbers(containers: &[InodeLeafContainer]) -> Vec<u64> {
+        containers
+            .iter()
+            .map(|container| container.identity.container)
+            .collect()
+    }
+
+    fn record_counts(containers: &[InodeLeafContainer]) -> Vec<usize> {
+        containers
+            .iter()
+            .map(|container| container.records.len())
+            .collect()
+    }
+
+    /// 删一条、那一片还有记录、与左右邻都并不了、不少于 117 条：只重写那一片，身份不变，别的片照抄，没有身份消失。
     #[test]
-    fn removing_a_record_from_a_container_that_keeps_others_rewrites_only_that_container() {
-        let before = three_containers();
+    fn removing_a_record_that_triggers_no_merge_rewrites_only_that_container() {
+        let before = full_containers(3);
         let after = remove_record_from_leaf_containers(&before, InodeNumber(300)).expect("删 300");
-        assert_eq!(after.containers.len(), 3);
+        assert_eq!(record_counts(&after.containers), vec![233, 232, 233]);
         assert_eq!(
             after.containers[1].identity, before[1].identity,
             "COW 重写身份不变"
         );
-        assert_eq!(after.containers[1].records.len(), 232);
         assert!(
             after.containers[1]
                 .records
@@ -749,44 +1014,214 @@ mod tests {
             after.rewritten,
             vec![InodeLeafContainerIndexInTree::of_position(1)]
         );
-        assert_eq!(after.containers_dropped, 0);
+        assert_eq!(
+            after.origins,
+            vec![
+                InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(
+                    0
+                )),
+                InodeLeafContainerOrigin::RewrittenThisPublish,
+                InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(
+                    2
+                )),
+            ]
+        );
+        assert!(after.retired.is_empty());
         assert!(after.is_the_root_rewritten());
     }
 
-    /// 删空最右那一片：整片不落盘，重写清单是空的，根照样重写（少了一条条目）。
+    /// 删空最右那一片：整片不落盘、身份消失，重写清单是空的，根照样重写（少了一条条目）。它左边没有右邻可并。
     #[test]
     fn removing_the_only_record_of_the_rightmost_container_drops_it_and_still_rewrites_the_root() {
         let before = three_containers();
         let after = remove_record_from_leaf_containers(&before, InodeNumber(467)).expect("删 467");
         assert_eq!(after.containers, before[..2].to_vec(), "前两片原样");
         assert!(after.rewritten.is_empty(), "没有一片要重写");
-        assert_eq!(after.containers_dropped, 1);
+        assert_eq!(
+            after.retired,
+            vec![RetiredInodeLeafContainer {
+                previous_index: InodeLeafContainerIndexInTree::of_position(2),
+                identity: before[2].identity,
+            }]
+        );
         assert!(after.is_the_root_rewritten(), "根少一条条目，要重写");
     }
 
-    /// 删空中间一片：它右边的每一片左移一格、全部重写（身份与记录不变），左边的不动。
+    /// 删空最左一片（M1 写出来的树才有这种形状：左边一片只剩一条）：整片不落盘、身份消失；它没有左邻，不并；
+    /// 右边每一片左移一格、内容没变，照抄、不进重写清单（`origins` 记着上一版的叶序）——20 片与 40 片两种几何重写清单都是空的，
+    /// 不随右边片数涨（判决 m3-retire-r1 第三节第 7 条）。
     #[test]
-    fn removing_the_only_record_of_a_middle_container_shifts_and_rewrites_every_container_to_its_right(
+    fn dropping_the_emptied_leftmost_container_carries_every_container_to_its_right_whatever_their_number(
     ) {
+        for containers_in_the_tree in [20_u64, 40] {
+            let mut before = full_containers(containers_in_the_tree);
+            keep_the_first_records(&mut before, 0, 1);
+            let after = remove_record_from_leaf_containers(&before, InodeNumber(1)).expect("删 1");
+            assert_eq!(
+                after.containers,
+                before[1..].to_vec(),
+                "{containers_in_the_tree} 片：右边各片内容、身份、分隔 key 原样"
+            );
+            assert!(
+                after.rewritten.is_empty(),
+                "{containers_in_the_tree} 片：挪了位置的一片都不重写"
+            );
+            assert_eq!(
+                after.origins,
+                (1..before.len())
+                    .map(|previous_position| InodeLeafContainerOrigin::CarriedFrom(
+                        InodeLeafContainerIndexInTree::of_position(previous_position)
+                    ))
+                    .collect::<Vec<_>>(),
+                "{containers_in_the_tree} 片：这一版第 i 片照抄上一版第 i + 1 片"
+            );
+            assert_eq!(after.retired_identities(), vec![before[0].identity]);
+        }
+    }
+
+    /// 删空中间一片（M1 写出来的形状）：身份消失；左邻与新右邻之和 > 233 不并；新右邻左移一格照抄，左边那片不动。
+    #[test]
+    fn dropping_an_emptied_middle_container_carries_the_ones_to_its_right_without_rewriting_them() {
         let mut before = three_containers();
-        // 把第二片删到只剩 234 一条（直接在内容上改：这里判的是删空之后的落法，不是怎么删到只剩一条）。
-        before[1].records.truncate(1);
+        keep_the_first_records(&mut before, 1, 1);
         let after = remove_record_from_leaf_containers(&before, InodeNumber(234)).expect("删 234");
-        assert_eq!(after.containers.len(), 2);
-        assert_eq!(after.containers[0], before[0], "左边那片不动");
+        assert_eq!(after.containers, vec![before[0].clone(), before[2].clone()]);
+        assert!(
+            after.rewritten.is_empty(),
+            "左邻 233 + 新右邻 1 = 234 > 233，不并"
+        );
         assert_eq!(
-            after.containers[1], before[2],
-            "第三片挪到第二个位置，身份与记录原样"
+            after.origins[1],
+            InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(2))
         );
+        assert_eq!(after.retired_identities(), vec![before[1].identity]);
+    }
+
+    /// 左吸收右（M3）：第一片删完一条剩 99 条、第二片 100 条，之和 199 ≤ 233 ⇒ 第一片吸收第二片：保留左半身份与分隔 key，记录接在后面；
+    /// 第二片身份消失；第三片左移一格照抄。前后叶序：容器号 [1, 234, 467] → [1, 467]。
+    #[test]
+    fn the_left_container_absorbs_its_right_neighbour_when_the_two_fit_in_one_and_keeps_its_identity(
+    ) {
+        let mut before = full_containers(3);
+        keep_the_first_records(&mut before, 0, 100);
+        keep_the_first_records(&mut before, 1, 100);
+        let after = remove_record_from_leaf_containers(&before, InodeNumber(5)).expect("删 5");
+        assert_eq!(container_numbers(&after.containers), vec![1, 467]);
+        assert_eq!(record_counts(&after.containers), vec![199, 233]);
+        assert_eq!(
+            after.containers[0].identity, before[0].identity,
+            "保留左半身份"
+        );
+        assert_eq!(after.containers[0].separator_key(), 1, "左半分隔 key 不变");
+        assert_eq!(
+            after.containers[0]
+                .records
+                .last()
+                .map(|record| record.inode),
+            Some(333),
+            "右半的记录接在后面"
+        );
+        assert_eq!(after.containers[1], before[2], "第三片原样");
         assert_eq!(
             after.rewritten,
-            vec![InodeLeafContainerIndexInTree::of_position(1)],
-            "挪了位置的那一片重写"
+            vec![InodeLeafContainerIndexInTree::LEFTMOST]
+        );
+        assert_eq!(
+            after.origins[1],
+            InodeLeafContainerOrigin::CarriedFrom(InodeLeafContainerIndexInTree::of_position(2))
+        );
+        assert_eq!(
+            after.retired,
+            vec![RetiredInodeLeafContainer {
+                previous_index: InodeLeafContainerIndexInTree::of_position(1),
+                identity: before[1].identity,
+            }],
+            "被吸收的右半身份消失"
+        );
+    }
+
+    /// 左从右借（M3 均分借）：第一片删完一条剩 99 条、第二片 233 条，之和 > 233 不并；99 < 117 ⇒ 从第二片借最小的 ⌊(233 − 99) ÷ 2⌋ = 67 条，
+    /// 借完两片 166 / 166（相差不超过 1）；两片都保留身份，第二片的分隔 key 改成它这一版的最小 key 301（N1）；没有身份消失，第三片照抄。
+    #[test]
+    fn a_container_below_the_floor_borrows_half_the_difference_from_its_right_neighbour() {
+        let mut before = full_containers(3);
+        keep_the_first_records(&mut before, 0, 100);
+        let after = remove_record_from_leaf_containers(&before, InodeNumber(5)).expect("删 5");
+        assert_eq!(container_numbers(&after.containers), vec![1, 234, 467]);
+        assert_eq!(record_counts(&after.containers), vec![166, 166, 233]);
+        assert_eq!(after.containers[0].identity, before[0].identity);
+        assert_eq!(
+            after.containers[1].identity, before[1].identity,
+            "右片保留身份"
+        );
+        assert_eq!(
+            after
+                .containers
+                .iter()
+                .map(InodeLeafContainer::separator_key)
+                .collect::<Vec<_>>(),
+            vec![1, 301, 467],
+            "右片的分隔 key = 它这一版的最小 key（N1），别的不变"
+        );
+        assert_eq!(
+            after.containers[0]
+                .records
+                .last()
+                .map(|record| record.inode),
+            Some(300),
+            "借来的是右片最小的 67 条：234..=300"
+        );
+        assert_eq!(after.containers[2], before[2]);
+        assert_eq!(
+            after.rewritten,
+            vec![
+                InodeLeafContainerIndexInTree::of_position(0),
+                InodeLeafContainerIndexInTree::of_position(1)
+            ]
+        );
+        assert!(after.retired.is_empty(), "借不退役");
+    }
+
+    /// 借了之后再删到 116、右片又借、……：同一对一路删 233 条，最后一条让两片之和恰好 233，左吸收右。
+    /// 均分借每次都让两片条数相差不超过 1；中间每一版相邻两片之和都 ≥ 234（M3 那一臂的丙类断言）。
+    #[test]
+    fn deleting_from_the_left_of_a_pair_borrows_until_the_pair_fits_in_one_and_then_absorbs() {
+        let mut containers = full_containers(3);
+        let mut borrows = 0;
+        let mut deletions = 0;
+        let mut absorbed = false;
+        // 迭代上界：这一对一共 466 条，每一轮删一条；吸收那一轮退出。
+        while !absorbed {
+            let inode = containers[0].records[1].inode;
+            let after = remove_record_from_leaf_containers(&containers, InodeNumber(inode))
+                .expect("删左片第二条");
+            deletions += 1;
+            absorbed = !after.retired.is_empty();
+            if after.rewritten.len() == 2 {
+                borrows += 1;
+                let counts = record_counts(&after.containers);
+                assert!(counts[0].abs_diff(counts[1]) <= 1, "均分借：{counts:?}");
+            }
+            if !absorbed {
+                for pair in after.containers.windows(2) {
+                    assert!(
+                        pair[0].records.len() + pair[1].records.len() >= 234,
+                        "相邻两片之和 ≥ 234"
+                    );
+                }
+            }
+            containers = after.containers;
+        }
+        assert_eq!(deletions, 233, "两片之和 466 删到 233 才装得下一片");
+        assert_eq!(
+            borrows, 7,
+            "116 → 174 / 145 / 131 / 123 / 120 / 118 / 117 七次借"
         );
-        assert_eq!(after.containers_dropped, 1);
+        assert_eq!(container_numbers(&containers), vec![1, 467]);
+        assert_eq!(record_counts(&containers), vec![233, 233]);
     }
 
-    /// 删的是树里唯一一条（unlink 唯一的文件）：一片都不剩、没有要重写的片，根照样重写成零条目根。
+    /// 删的是树里唯一一条（unlink 唯一的文件）：一片都不剩、没有要重写的片，那片身份消失，根照样重写成零条目根。
     #[test]
     fn removing_the_last_record_of_the_tree_leaves_no_container_and_rewrites_the_root() {
         let before = write_records_into_leaf_containers(
@@ -800,7 +1235,7 @@ mod tests {
         let after = remove_record_from_leaf_containers(&before, InodeNumber(1)).expect("删 1");
         assert!(after.containers.is_empty(), "一片都不剩");
         assert!(after.rewritten.is_empty());
-        assert_eq!(after.containers_dropped, 1);
+        assert_eq!(after.retired_identities(), vec![before[0].identity]);
         assert!(after.is_the_root_rewritten(), "根重写成零条目根");
     }
 
diff --git a/crates/singlefs-core/src/lib.rs b/crates/singlefs-core/src/lib.rs
index d44a17e..4525d8f 100644
--- a/crates/singlefs-core/src/lib.rs
+++ b/crates/singlefs-core/src/lib.rs
@@ -32,6 +32,7 @@ pub mod mounted_session;
 pub mod pointer;
 pub mod records;
 pub mod recovery;
+pub mod retirement_tree;
 pub mod root_record;
 pub mod root_ring;
 pub mod system_configuration;
diff --git a/crates/singlefs-core/src/lineage_tree.rs b/crates/singlefs-core/src/lineage_tree.rs
index 8fac1e9..5cb089b 100644
--- a/crates/singlefs-core/src/lineage_tree.rs
+++ b/crates/singlefs-core/src/lineage_tree.rs
@@ -1,10 +1,10 @@
-//! 谱系树的叶容器（D18（块里携带什么信息） 已定项 24 第 5 条）：码 3 打包记录类型 5，一容器装 2039 条 16 字节谱系记录；
+//! 谱系树的叶容器（D18（块里携带什么信息） 已定项 24 第 5 条）：码 3 打包记录类型 5，一容器装 1359 条 24 字节谱系记录；
 //! 根恒为码 2，条目同墓碑树（`crate::container_tree`）。随现行那一版、只增不删；管理员回退那次发布追加一条
 //! （D23（journal 的角色与格式） 已定项 14）；容器号从 1 起，下一个号 = 树里最大号 + 1，零根时 1（规则同墓碑容器）。
 //!
 //! **根满长高**：根装满之后同样长一层（第 5 条「谱系树根满同样长高」），共用的那一半在 `crate::container_tree`。
 //! 只供测试的两个旋钮（一片装几条 [`LineageContainerCapacity`]、一个码 2 节点装几条 `ContainerTreeNodeEntries`）只住内存、不落盘，
-//! 「开下一片」与「长高」这两支按产品容量要 2039 次与 2039 × 135 次回退才走得到（`.claude/rules/fs-design.md` 五条硬要求第 2 条）。
+//! 「开下一片」与「长高」这两支按产品容量要 1359 次与 1359 × 135 次回退才走得到（`.claude/rules/fs-design.md` 五条硬要求第 2 条）。
 
 use singlefs_format::LINEAGE_CONTAINER_RECORDS;
 
@@ -41,12 +41,12 @@ impl LineageContainerIndexInTree {
 /// 一片谱系容器这一版装的东西：身份四段（出生树 = 谱系树、打包记录类型 5、容器号、容器出生代）与它的记录，按追加的先后。
 pub type LineageContainer = RecordContainer<LineageRecord>;
 
-/// 只供测试压小的每片谱系容器条数：至少 1、至多产品容量 2039；只住内存，不落盘（同墓碑容器的旋钮，D18（块里携带什么信息） 已定项 24 第 3 条）。
+/// 只供测试压小的每片谱系容器条数：至少 1、至多产品容量 1359；只住内存，不落盘（同墓碑容器的旋钮，D18（块里携带什么信息） 已定项 24 第 3 条）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct LineageRecordsPerContainerCappedForTests(u64);
 
 impl LineageRecordsPerContainerCappedForTests {
-    /// 0 条或多于产品容量 2039 交回 `None`。
+    /// 0 条或多于产品容量 1359 交回 `None`。
     #[must_use]
     pub fn new(records: u64) -> Option<Self> {
         (1..=LINEAGE_CONTAINER_RECORDS)
@@ -60,7 +60,7 @@ impl LineageRecordsPerContainerCappedForTests {
     }
 }
 
-/// 一片谱系容器装满几条就关：产品路径按容器格式算（2039），只供测试的开关把它压小。
+/// 一片谱系容器装满几条就关：产品路径按容器格式算（1359），只供测试的开关把它压小。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum LineageContainerCapacity {
     FromTheContainerFormat,
@@ -113,7 +113,7 @@ pub fn append_lineage_record_to_the_open_container(
         checkpoint_txg,
         lineage_tree,
         PACKED_TYPE_LINEAGE,
-        usize::try_from(capacity.records_per_container()).expect("至多 2039 条"),
+        usize::try_from(capacity.records_per_container()).expect("至多 1359 条"),
         node_entries,
     )
 }
@@ -127,12 +127,13 @@ mod tests {
     const LINEAGE_TREE: TreeIdentifier = TreeIdentifier(TREE_IDENTIFIER_LINEAGE);
 
     /// 零根时第一条谱系记录开第一片容器：容器号 1、出生代 = 这次回退的 txg、打包记录类型 5、出生树 = 谱系树；
-    /// 第二次回退追加进同一片（一片装 2039 条），只重写它、身份不变。
+    /// 第二次回退追加进同一片（一片装 1359 条），只重写它、身份不变。
     #[test]
     fn the_first_lineage_record_opens_container_number_one_and_the_second_is_appended_to_it() {
         let first_record = LineageRecord {
             rollback_txg: CheckpointTxg(12),
             rollback_target_txg: CheckpointTxg(9),
+            inode_number_watermark_of_the_version_rolled_back_from: 4,
         };
         let first = append_lineage_record_to_the_open_container(
             &[],
@@ -163,6 +164,7 @@ mod tests {
         let second_record = LineageRecord {
             rollback_txg: CheckpointTxg(15),
             rollback_target_txg: CheckpointTxg(12),
+            inode_number_watermark_of_the_version_rolled_back_from: 4,
         };
         let second = append_lineage_record_to_the_open_container(
             &first.containers,
@@ -184,23 +186,31 @@ mod tests {
         assert_eq!(second.rewritten_containers, vec![0]);
     }
 
-    /// 谱系记录 16 字节：T_rb 在前、T_old 在后，小端；读回逐字段相等，长度不是 16 读不出。
+    /// 谱系记录 24 字节：T_rb、T_old、H_cur 依次各 8 字节，小端；读回逐字段相等，长度不是 24 读不出（短一字节、长一字节都不收）。
     #[test]
-    fn a_lineage_record_is_sixteen_bytes_rollback_txg_then_target_txg() {
+    fn a_lineage_record_is_twenty_four_bytes_rollback_txg_then_target_txg_then_the_watermark_rolled_back_from(
+    ) {
         let record = LineageRecord {
             rollback_txg: CheckpointTxg(0x0102),
             rollback_target_txg: CheckpointTxg(0x0304),
+            inode_number_watermark_of_the_version_rolled_back_from: 0x0506,
         };
         let bytes = record.to_bytes();
         assert_eq!(
             bytes,
             [
                 [0x02, 0x01, 0, 0, 0, 0, 0, 0],
-                [0x04, 0x03, 0, 0, 0, 0, 0, 0]
+                [0x04, 0x03, 0, 0, 0, 0, 0, 0],
+                [0x06, 0x05, 0, 0, 0, 0, 0, 0]
             ]
-            .concat()
+            .concat(),
+            "T_rb 0、T_old 8、H_cur 16，各 8 字节小端"
         );
         assert_eq!(LineageRecord::parse(&bytes), Some(record));
-        assert_eq!(LineageRecord::parse(&bytes[..15]), None);
+        assert_eq!(LineageRecord::parse(&bytes[..23]), None);
+        assert_eq!(
+            LineageRecord::parse(&[bytes.as_slice(), &[0]].concat()),
+            None
+        );
     }
 }
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 3747c50..a05480e 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -34,8 +34,7 @@ use crate::mounted_session::refusal_is_short_of_space;
 use crate::pointer::LocationEntry;
 use crate::pointer::{slot_shared_by_both_location_entries, LocationEntriesOnDifferentSlots};
 use crate::records::{
-    AccountingEntry, LineageRecord, TreeTableEntry, STATISTIC_INODE_WATERMARK,
-    TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE,
+    LineageRecord, TreeTableEntry, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE,
 };
 use crate::recovery::read_unit_via_locations;
 use crate::recovery::{
@@ -7518,14 +7517,17 @@ pub const fn is_a_user_visible_unit(identity: TransactionUnit) -> bool {
         | TransactionUnit::InstanceTable
         | TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,
         // `deleted_inodes` 根不在 D23 那个记号里（新根照样取 R_old 的，见 [`is_a_unit_taken_from_the_rollback_target`]）；
-        // 墓碑树与谱系树随现行那一版。
+        // 墓碑树、谱系树与退役树随现行那一版（退役树：判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）。
         TransactionUnit::DeletedInodesRoot
         | TransactionUnit::TombstoneLeafContainer(_)
         | TransactionUnit::TombstoneRoot
         | TransactionUnit::TombstoneNodeBelowTheRoot(_)
         | TransactionUnit::LineageLeafContainer(_)
         | TransactionUnit::LineageRoot
-        | TransactionUnit::LineageNodeBelowTheRoot(_) => false,
+        | TransactionUnit::LineageNodeBelowTheRoot(_)
+        | TransactionUnit::RetirementLeafContainer(_)
+        | TransactionUnit::RetirementRoot
+        | TransactionUnit::RetirementNodeBelowTheRoot(_) => false,
     }
 }
 
@@ -7557,7 +7559,10 @@ pub const fn is_a_unit_taken_from_the_rollback_target(identity: TransactionUnit)
         | TransactionUnit::TombstoneNodeBelowTheRoot(_)
         | TransactionUnit::LineageLeafContainer(_)
         | TransactionUnit::LineageRoot
-        | TransactionUnit::LineageNodeBelowTheRoot(_) => is_a_user_visible_unit(identity),
+        | TransactionUnit::LineageNodeBelowTheRoot(_)
+        | TransactionUnit::RetirementLeafContainer(_)
+        | TransactionUnit::RetirementRoot
+        | TransactionUnit::RetirementNodeBelowTheRoot(_) => is_a_user_visible_unit(identity),
     }
 }
 
@@ -7786,8 +7791,9 @@ fn every_copy_of_the_resurrected_units_reads_back<Reader: PoolReader + ?Sized>(
     Ok(())
 }
 
-/// 根环里读得出的每一条带文件的根，它那一版记账里的 inode 号水位；从盘上重建那一版读不出的根不算（「环里读得出的根」）。
-/// 只有管理员回退读它（D23（journal 的角色与格式） 已定项 14「水位」一格）。`record_standing_for_every_root` 只是给重建顶着的一条记录
+/// `ring_roots` 里每一条带文件的根，它那一版记账里的 inode 号水位，取最大；从盘上重建那一版读不出的根不算（「环里读得出的根」）。
+/// 只有管理员回退读它（D23（journal 的角色与格式） 已定项 14「水位」一格），交进来的只是按 cur 的实例表没被崩溃恢复抛弃的那几条
+/// （Opt2，[`ring_roots_not_abandoned_by_crash_recovery`]）。`record_standing_for_every_root` 只是给重建顶着的一条记录
 /// （重建出来的版本只拿它的 jsn 与事务号，这里都不用）。
 #[allow(
     clippy::ptr_arg,
@@ -7817,17 +7823,31 @@ fn highest_inode_number_watermark_in_the_ring<Device: BlockDevice>(
         .max()
 }
 
+/// 根环里读得出的根里，按 cur 的实例表（`current_table`）没被崩溃恢复抛弃的那几条（D23（journal 的角色与格式） 已定项 14「水位」一格的 Opt2：
+/// 管理员回退取 inode 号水位只看它们）。被抛弃的根在它那条时间线上发过的号，崩溃恢复落到较旧的根之后本来就会重发（D8（核心索引结构） 已定项 6 射程），
+/// 算进来会让回退把水位抬过这条时间线没发过的号。
+fn ring_roots_not_abandoned_by_crash_recovery(
+    ring_roots: &[RootRecord],
+    current_table: &InstanceTableRecords,
+) -> Vec<RootRecord> {
+    ring_roots
+        .iter()
+        .filter(|root| !abandoned_by_table(root, current_table))
+        .cloned()
+        .collect()
+}
+
 /// 回退那次发布接在后面的「上一版」：用户可见的那一半（数据单元与指针、extent 树、inode 树的叶容器与根、第一个文件那条 inode 记录）
 /// 取回退目标那一版的，别的（分配记录树与它的记录、记账树与记账行、中央映射树、树表、实例表各片）取现行那一版的；
 /// 根记录、末条记录、事务号取现行那一版的（这次发布接着它发，反向链、计数器、txg 都接着它）。
 /// 发布路径照这一版照抄用户可见单元（inode 树根指针从树表条目取，所以树表条目里 inode 树与 extent 树那两条换成目标那一版的）、
 /// 按分配器现算固定点单元、换下现行那一版被重写的固定点单元——正是「新根的用户可见树取 R_old 的，树表单元、分配记录树、中央映射树与
 /// 记账统计量按这次发布照调整之后的分配器现算」（D23（journal 的角色与格式） 已定项 14）。
-/// 记账行里 inode 号水位那一行换成 `inode_number_watermark`（「水位」一格的 max），发布路径在它上面加这次新建的（0 个）。
+/// 记账行原样取 cur 的：inode 号水位的台阶行照抄 cur 的（D5（快照 / 空间记账机制） 已定项 2 射程「管理员回退那次从 cur 带、不取 R_old 的」），
+/// 回退那次发布的水位（「水位」一格的 max）由发布路径从谱系那一步拿（`publish_version_appending_a_lineage_record`），不改这里的行。
 fn version_to_publish_the_rollback_after(
     current: &TransactionOutput,
     target_version: &TransactionOutput,
-    inode_number_watermark: u64,
 ) -> TransactionOutput {
     let user_visible_units_of_the_target = target_version
         .units
@@ -7877,22 +7897,6 @@ fn version_to_publish_the_rollback_after(
             head_identifier: entry.head_identifier,
         })
         .collect();
-    let accounting_entries = current
-        .accounting_entries
-        .iter()
-        .map(|entry| AccountingEntry {
-            statistic: entry.statistic,
-            tree: entry.tree,
-            device: entry.device,
-            generation: entry.generation,
-            value: if entry.statistic == STATISTIC_INODE_WATERMARK {
-                inode_number_watermark
-            } else {
-                entry.value
-            },
-            sequence: entry.sequence,
-        })
-        .collect();
     TransactionOutput {
         root: current.root,
         record: current.record.clone(),
@@ -7907,7 +7911,7 @@ fn version_to_publish_the_rollback_after(
         allocation_records: current.allocation_records.clone(),
         allocation_record_tree: current.allocation_record_tree.clone(),
         extent_tree: target_version.extent_tree.clone(),
-        accounting_entries,
+        accounting_entries: current.accounting_entries.clone(),
         accounting_tree: current.accounting_tree.clone(),
         central_mapping_tree: current.central_mapping_tree.clone(),
         tree_table_entries,
@@ -7920,6 +7924,10 @@ fn version_to_publish_the_rollback_after(
         // 谱系树同样随现行那一版（D18（块里携带什么信息） 已定项 24 第 5 条「随现行那一版、只增不删」）：回退这次往它上面追加。
         lineage_containers: current.lineage_containers.clone(),
         lineage_tree_index_nodes: current.lineage_tree_index_nodes.clone(),
+        // 退役树同样随现行那一版、不取 R_old（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）：
+        // 回退那次不写退役记录，R_old 之后身份消失、回退之后又活过来的叶容器，它的退役记录死亡 txg 不在新这一版的父链上。
+        retirement_containers: current.retirement_containers.clone(),
+        retirement_tree_index_nodes: current.retirement_tree_index_nodes.clone(),
         inode_leaf_containers: target_version.inode_leaf_containers.clone(),
         mapped_units,
         released: Vec::new(),
@@ -7940,7 +7948,8 @@ fn version_to_publish_the_rollback_after(
 /// - 释放：cur 引用、新根不引用的用户可见单元（按落点比），释放代 = 新根的 txg；释放之前照发布路径读盘核，对不上的那几份留在已分配。
 ///   cur 被这次重写换下的固定点单元由发布路径照常释放。
 /// - 复活：R_old 引用的用户可见单元里 cur 的账记成已释放的，改回已分配，分配代写回 R_old 那一版账里的。
-/// - 水位：inode 号水位与树 ID 水位 = max(cur 那一版内存里的, 环里读得出的根的)。
+/// - 水位：inode 号水位 = max(cur 那一版内存里的, 环里读得出、按 cur 的实例表没被崩溃恢复抛弃的根的)（Opt2）；
+///   树 ID 水位 = max(cur 那一版内存里的, 环里读得出的全部根的)。台阶行照抄 cur 的；谱系记录带 H_cur = cur 的 inode 号水位（从 cur 读）。
 /// - 调用方的参数与盘表：照可写挂载同一套核（[`caller_inputs_agreeing_with_the_disk`]），写入口只按盘上择到的系统配置里那份参数建。
 /// - 在任何写之前拒：见 [`RollbackError`] 除 `Publish` 落盘途中失败之外的每一个成员。
 ///
@@ -8103,17 +8112,18 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
         .iter()
         .map(|root| root.tree_identifier_watermark)
         .fold(current.root.tree_identifier_watermark, u64::max);
+    // inode 号水位（Opt2，D23（journal 的角色与格式） 已定项 14「水位」一格）：环里只取按 cur 的实例表没被崩溃恢复抛弃的根；
+    // max 里 cur 那一版内存里的那一项不能去——只取环，回退那次就重发号（C143）。
     let inode_number_watermark = highest_inode_number_watermark_in_the_ring(
         devices,
         unit_area_start,
-        &ring_roots,
+        &ring_roots_not_abandoned_by_crash_recovery(&ring_roots, &current_table),
         &current.record,
     )
     .map_or(current.inode_number_watermark(), |ring| {
         ring.max(current.inode_number_watermark())
     });
-    let previous =
-        version_to_publish_the_rollback_after(current, &target_version, inode_number_watermark);
+    let previous = version_to_publish_the_rollback_after(current, &target_version);
     // 动分配器：先复活、再释放（释放代 = 新根的 txg），之后交给发布路径——它在这份账上照常释放 cur 被换下的固定点单元、取落点、
     // 装记账行。发布在落盘之前被拒时整个换回这一份之前的样子。
     let allocator_before_the_rollback = allocator.clone();
@@ -8154,10 +8164,12 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
         tree_identifier_watermark,
         rollback_floor: current.root.rollback_floor,
     };
-    // 回退那次发布追加一条谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条）：T_rb 是这次的 txg，T_old 是 R_old 的 txg。
+    // 回退那次发布追加一条谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条）：T_rb 是这次的 txg，T_old 是 R_old 的 txg，
+    // H_cur 是 cur 那一版的 inode 号水位（从 cur 读，不从交给发布路径的「上一版」现算）。
     let lineage_record = LineageRecord {
         rollback_txg,
         rollback_target_txg: target_root.checkpoint_txg,
+        inode_number_watermark_of_the_version_rolled_back_from: current.inode_number_watermark(),
     };
     let mut pool = PoolWriter::new(&parameters_of_the_pool, devices.as_mut_slice());
     // 谱系容器与容器树节点的容量取分配器上装的只供测试的开关（产品路径按格式），与会话里的删除同一个做法。
@@ -8168,6 +8180,7 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
         plan,
         &previous,
         lineage_record,
+        inode_number_watermark,
     ) {
         Ok(published) => {
             *current = published;
@@ -8867,6 +8880,7 @@ mod rollback_recognized_among_the_packed_units_named_by_a_publish_tests {
         LineageRecord {
             rollback_txg: CheckpointTxg(rollback_txg),
             rollback_target_txg: CheckpointTxg(rollback_target_txg),
+            inode_number_watermark_of_the_version_rolled_back_from: 3,
         }
         .to_bytes()
     }
@@ -8911,7 +8925,7 @@ mod rollback_recognized_among_the_packed_units_named_by_a_publish_tests {
         );
     }
 
-    /// 不是谱系片的打包单元里的记录不当谱系记录读：一片 inode 叶里的 16 字节记录恰好拼得出 (9, 5)，也不认成回退。
+    /// 不是谱系片的打包单元里的记录不当谱系记录读：一片 inode 叶里的 24 字节记录恰好拼得出 (9, 5, 3)，也不认成回退。
     #[test]
     fn records_of_a_packed_unit_that_is_not_a_lineage_unit_are_not_read_as_lineage_records() {
         assert_eq!(
@@ -8953,7 +8967,7 @@ mod rollback_recognized_among_the_packed_units_named_by_a_publish_tests {
         );
     }
 
-    /// 谱系片解开了、里面有一条记录解不开（长度不是 16），别的记录都不是这次发布的：同样不知道。
+    /// 谱系片解开了、里面有一条记录解不开（长度不是 24），别的记录都不是这次发布的：同样不知道。
     #[test]
     fn an_unparsable_lineage_record_without_a_recognized_rollback_is_unknown() {
         assert_eq!(
@@ -8961,7 +8975,7 @@ mod rollback_recognized_among_the_packed_units_named_by_a_publish_tests {
                 CHECKPOINT_TXG_OF_THE_PUBLISH,
                 &[parsed(
                     PACKED_TYPE_LINEAGE,
-                    vec![lineage_record_bytes(7, 3), vec![0; 15]]
+                    vec![lineage_record_bytes(7, 3), vec![0; 23]]
                 )],
             ),
             RollbackRecognizedAmongThePackedUnitsNamedByAPublish::UnknownWhetherARollback,
@@ -9177,11 +9191,12 @@ mod packed_units_named_by_the_records_of_a_publish_as_read_tests {
         let lineage_record = LineageRecord {
             rollback_txg: CHECKPOINT_TXG_OF_THE_PUBLISH,
             rollback_target_txg: CheckpointTxg(5),
+            inode_number_watermark_of_the_version_rolled_back_from: 3,
         }
         .to_bytes();
         let lineage_unit = packed_unit_of(
             PACKED_TYPE_LINEAGE,
-            u16::try_from(LINEAGE_RECORD_BYTES).expect("16"),
+            u16::try_from(LINEAGE_RECORD_BYTES).expect("24"),
             std::slice::from_ref(&lineage_record),
         );
         let unparsable_unit = vec![0xA5u8; unit_bytes];
diff --git a/crates/singlefs-core/src/records.rs b/crates/singlefs-core/src/records.rs
index 89d4d85..8b51a5d 100644
--- a/crates/singlefs-core/src/records.rs
+++ b/crates/singlefs-core/src/records.rs
@@ -4,7 +4,7 @@
 use singlefs_format::{
     ACCOUNTING_ENTRY_BYTES, EXTENT_LEAF_RECORD_BYTES, INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES,
     LINEAGE_RECORD_BYTES, MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES, NODE_POINTER_BYTES,
-    TOMBSTONE_RECORD_BYTES, TREE_TABLE_ENTRY_BYTES,
+    RETIREMENT_RECORD_BYTES, TOMBSTONE_RECORD_BYTES, TREE_TABLE_ENTRY_BYTES,
 };
 
 use crate::address::{
@@ -177,35 +177,80 @@ impl TombstoneRecord {
     }
 }
 
-/// 谱系记录 16（D18（块里携带什么信息） 已定项 24 第 5 条；字段表在 `.claude/kb/layout/03-third-txn.md`「三、谱系记录」）：
-/// T_rb 8 + T_old 8，小端，读作「T_rb 的父是 T_old」。管理员回退那次发布追加一条：T_rb 是回退那次发布的 checkpoint_txg，
-/// T_old 是回退目标 R_old 的 txg。
+/// 谱系记录 24（D18（块里携带什么信息） 已定项 24 第 5 条；字段表在 `.claude/kb/layout/03-third-txn.md`「三、谱系记录」）：
+/// T_rb 8 + T_old 8 + H_cur 8，小端，读作「T_rb 的父是 T_old」。管理员回退那次发布追加一条：T_rb 是回退那次发布的 checkpoint_txg，
+/// T_old 是回退目标 R_old 的 txg，H_cur 是 cur（回退之前的现行那一版）的 inode 号水位，从 cur 那一版读、不从发布路径交来的「上一版」现算。
+/// 三段的逐字节次序条款没定，这里取 T_rb | T_old | H_cur（新加的一段接在已有两段之后，已有两段的偏移不动）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct LineageRecord {
     pub rollback_txg: CheckpointTxg,
     pub rollback_target_txg: CheckpointTxg,
+    pub inode_number_watermark_of_the_version_rolled_back_from: u64,
 }
 
 impl LineageRecord {
     #[must_use]
     pub fn to_bytes(&self) -> Vec<u8> {
-        let mut writer = ByteWriter::new(usize::try_from(LINEAGE_RECORD_BYTES).expect("16"));
+        let mut writer = ByteWriter::new(usize::try_from(LINEAGE_RECORD_BYTES).expect("24"));
         writer.put_u64(self.rollback_txg.0);
         writer.put_u64(self.rollback_target_txg.0);
+        writer.put_u64(self.inode_number_watermark_of_the_version_rolled_back_from);
         writer.assert_position(LINEAGE_RECORD_BYTES, "谱系记录");
         writer.into_bytes()
     }
 
-    /// 谱系记录的读者：长度不是 16 交回 `None`（记录宽是容器头里的盘上字段，读者先核它等于 16 再切）。
+    /// 谱系记录的读者：长度不是 24 交回 `None`（记录宽是容器头里的盘上字段，读者先核它等于 24 再切）。
     #[must_use]
     pub fn parse(bytes: &[u8]) -> Option<Self> {
-        if bytes.len() != usize::try_from(LINEAGE_RECORD_BYTES).expect("16") {
+        if bytes.len() != usize::try_from(LINEAGE_RECORD_BYTES).expect("24") {
             return None;
         }
         let mut reader = ByteReader::at(bytes, 0);
         Some(Self {
             rollback_txg: CheckpointTxg(reader.get_u64()),
             rollback_target_txg: CheckpointTxg(reader.get_u64()),
+            inode_number_watermark_of_the_version_rolled_back_from: reader.get_u64(),
+        })
+    }
+}
+
+/// 退役记录 34（D18（块里携带什么信息） 已定项 11 打包记录类型 3；判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2、3 条）：
+/// 被退役身份的出生树 8 | 打包记录类型 2 | 容器号 8 | 容器出生代 8 | 死亡 txg 8，小端，无填充无预留。身份四段与 inode 内部条目里的
+/// 身份引用（[`build_inode_internal_entry`]）同序同宽。死亡 txg = 身份消失那次发布的 checkpoint_txg（删空摘掉、被左邻吸收那次）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct RetirementRecord {
+    pub retired: PackedIdentity,
+    pub death_txg: CheckpointTxg,
+}
+
+impl RetirementRecord {
+    #[must_use]
+    pub fn to_bytes(&self) -> Vec<u8> {
+        let mut writer = ByteWriter::new(usize::try_from(RETIREMENT_RECORD_BYTES).expect("34"));
+        writer.put_u64(self.retired.birth_tree.0);
+        writer.put_u16(self.retired.record_type);
+        writer.put_u64(self.retired.container);
+        writer.put_u64(self.retired.container_birth.0);
+        writer.put_u64(self.death_txg.0);
+        writer.assert_position(RETIREMENT_RECORD_BYTES, "退役记录");
+        writer.into_bytes()
+    }
+
+    /// 退役记录的读者：长度不是 34 交回 `None`（记录宽是容器头里的盘上字段，读者先核它等于 34 再切）。
+    #[must_use]
+    pub fn parse(bytes: &[u8]) -> Option<Self> {
+        if bytes.len() != usize::try_from(RETIREMENT_RECORD_BYTES).expect("34") {
+            return None;
+        }
+        let mut reader = ByteReader::at(bytes, 0);
+        Some(Self {
+            retired: PackedIdentity {
+                birth_tree: TreeIdentifier(reader.get_u64()),
+                record_type: reader.get_u16(),
+                container: reader.get_u64(),
+                container_birth: CheckpointTxg(reader.get_u64()),
+            },
+            death_txg: CheckpointTxg(reader.get_u64()),
         })
     }
 }
@@ -383,6 +428,8 @@ pub const TREE_KIND_DELETED_INODES: u16 = 9;
 pub const TREE_KIND_TOMBSTONE: u16 = 10;
 pub const TREE_KIND_LINEAGE: u16 = 11;
 pub const TREE_KIND_LOGGED_OPERATIONS: u16 = 12;
+/// 退役树：码 13，接在 logged_ops 之后追加（D8（核心索引结构） 已定项 9；判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）。
+pub const TREE_KIND_RETIREMENT: u16 = 13;
 
 /// 树表条目里根指针的起点：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2（`singlefs_format::TREE_TABLE_ENTRY_BYTES` 的字段表）。
 const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET_IN_BYTES: usize = 8 + 2 + 2 + 2;
@@ -558,7 +605,7 @@ mod tests {
         TREE_IDENTIFIER_ACCOUNTING, TREE_IDENTIFIER_ALLOCATION_RECORDS,
         TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST, TREE_IDENTIFIER_DELETED_INODES,
         TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LINEAGE,
-        TREE_IDENTIFIER_LIVELIST, TREE_IDENTIFIER_LOGGED_OPERATIONS,
+        TREE_IDENTIFIER_LIVELIST, TREE_IDENTIFIER_LOGGED_OPERATIONS, TREE_IDENTIFIER_RETIREMENT,
         TREE_IDENTIFIER_SPARSE_SIDE_TABLE, TREE_IDENTIFIER_TOMBSTONE,
         TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, TREE_IDENTIFIER_WATERMARK_AT_MKFS,
     };
@@ -580,6 +627,7 @@ mod tests {
             TREE_KIND_TOMBSTONE,
             TREE_KIND_LINEAGE,
             TREE_KIND_LOGGED_OPERATIONS,
+            TREE_KIND_RETIREMENT,
         ] {
             assert!(
                 u64::from(kind) < first_value_the_kind_field_cannot_hold,
@@ -599,6 +647,7 @@ mod tests {
             TREE_IDENTIFIER_TOMBSTONE,
             TREE_IDENTIFIER_LINEAGE,
             TREE_IDENTIFIER_LOGGED_OPERATIONS,
+            TREE_IDENTIFIER_RETIREMENT,
             TREE_IDENTIFIER_WATERMARK_AT_MKFS,
             TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
         ] {
@@ -750,6 +799,46 @@ mod tests {
         );
     }
 
+    /// 退役记录 34 字节：出生树 0、打包记录类型 8、容器号 10、容器出生代 18、死亡 txg 26，小端、无填充；读回逐字段相等，
+    /// 长度不是 34 读不出（短一字节、长一字节都不收）。身份四段与 inode 内部条目里身份引用那 26 字节逐字节相同。
+    #[test]
+    fn a_retirement_record_is_thirty_four_bytes_identity_in_entry_order_then_the_death_txg() {
+        let record = RetirementRecord {
+            retired: PackedIdentity {
+                birth_tree: TreeIdentifier(0x0102),
+                record_type: 2,
+                container: 0x0304,
+                container_birth: CheckpointTxg(0x0506),
+            },
+            death_txg: CheckpointTxg(0x0708),
+        };
+        let bytes = record.to_bytes();
+        assert_eq!(
+            bytes,
+            [
+                vec![0x02, 0x01, 0, 0, 0, 0, 0, 0],
+                vec![2, 0],
+                vec![0x04, 0x03, 0, 0, 0, 0, 0, 0],
+                vec![0x06, 0x05, 0, 0, 0, 0, 0, 0],
+                vec![0x08, 0x07, 0, 0, 0, 0, 0, 0],
+            ]
+            .concat(),
+            "出生树 0、类型 8、容器号 10、出生代 18、死亡 txg 26，小端"
+        );
+        let entry = build_inode_internal_entry(9, record.retired, NodePointer::empty_root());
+        assert_eq!(
+            &bytes[..26],
+            &entry[8..34],
+            "身份四段与 inode 内部条目里的身份引用同序同宽"
+        );
+        assert_eq!(RetirementRecord::parse(&bytes), Some(record));
+        assert_eq!(RetirementRecord::parse(&bytes[..33]), None);
+        assert_eq!(
+            RetirementRecord::parse(&[bytes.as_slice(), &[0]].concat()),
+            None
+        );
+    }
+
     /// D8（核心索引结构） 已定项 16 第 4 条：nlink 由 `to_bytes` 照结构里的值写进偏移 36、由 `parse` 读回来——
     /// 删除中的记录（nlink = 0，size 不变）写出去读回来还是 0，读者与写入口才认得出它；活着的写 1。
     #[test]
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 3e190ed..e58037e 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -15,8 +15,9 @@ use singlefs_format::{
     ACCOUNTING_ENTRY_BYTES, CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES,
     FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES, INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES,
     JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, JOURNAL_SAFETY_FACTOR, LINEAGE_RECORD_BYTES,
-    MAPPING_ENTRY_BYTES, NODE_BYTES, ROOT_RECORD_BYTES, ROOT_RING_REGIONS, SLOT_BYTES,
-    SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES, TOMBSTONE_RECORD_BYTES,
+    MAPPING_ENTRY_BYTES, NODE_BYTES, RETIREMENT_RECORD_BYTES, ROOT_RECORD_BYTES, ROOT_RING_REGIONS,
+    SLOT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES,
+    TOMBSTONE_RECORD_BYTES,
 };
 use singlefs_replay_prefix::{
     choose_root_slot, first_copy_of_each_record_key, replay_core, CoreFailure, CoreRecord,
@@ -68,11 +69,13 @@ use crate::pointer::{
 use crate::records::{
     mapping_key_for_data, mapping_key_for_node, parse_inode_internal_entry_or_refusal,
     parse_mapping_entry, AccountingEntry, InodeRecord, LineageRecord, PointerCarryingEntryRefusal,
-    TombstoneRecord, TreeTableEntry, NLINK_OF_AN_INODE_BEING_DELETED, STATISTIC_INODE_WATERMARK,
-    TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_DELETED_INODES,
-    TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LIVELIST,
-    TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
+    RetirementRecord, TombstoneRecord, TreeTableEntry, NLINK_OF_AN_INODE_BEING_DELETED,
+    STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
+    TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LINEAGE,
+    TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_RETIREMENT,
+    TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
 };
+use crate::retirement_tree::RetirementContainerIndexInTree;
 use crate::root_record::RootRecord;
 use crate::root_ring::{region_start, slot_offset, RootRingSlot, RootRingSlotsPerRegionOutOfRange};
 use crate::system_configuration::{
@@ -86,14 +89,15 @@ use crate::tombstone_tree::CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES;
 use crate::transaction::{
     role_of_allocation_record_tree_node, role_of_extent_upper_node, ContainerTreeIndexNodesVersion,
     FileVersionTreeIdentifiers, InodeLeafContainerVersion, LineageContainerVersion,
-    MultiLevelCodeTwoTree, PublishedUnit, TombstoneContainerVersion, TransactionOutput,
-    TransactionUnit, FIRST_INODE_NUMBER,
+    MultiLevelCodeTwoTree, PublishedUnit, RetirementContainerVersion, TombstoneContainerVersion,
+    TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER,
 };
 use crate::unit::{
     data_unit_payload, data_unit_payload_capacity, parse_data_unit, parse_index_node,
     parse_packed_unit, unit_filesystem_identifier, IndexNodeHeader, PackedIdentity,
     PackedUnitHeader, PACKED_TYPE_INODE, PACKED_TYPE_INSTANCE_TABLE, PACKED_TYPE_LINEAGE,
-    PACKED_TYPE_TOMBSTONE, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
+    PACKED_TYPE_RETIREMENT, PACKED_TYPE_TOMBSTONE, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE,
+    UNIT_CLASS_PACKED,
 };
 use crate::write_accounting::WritesByStructureKind;
 use crate::write_request_split::data_unit_count_of_a_sequential_write;
@@ -2367,9 +2371,10 @@ pub(crate) fn tree_table_entries_each_kind_at_most_once(
 /// 「立不变量并同步」；`invariants.md` 那一行由 kb 第八批写。
 pub const TREE_TABLE_ENTRIES_ORDERING_CONTRACT: &str = "I-9.16";
 
-/// 十一棵进树表的树的种类，按发号次序（D8（核心索引结构） 已定项 8 ②：extent、inode、分配记录、记账、中央映射、livelist、稀疏旁表、deadlist，
-/// 再接 D18（块里携带什么信息） 已定项 24 第 6 条的 `deleted_inodes`、墓碑树、谱系树、logged_ops；中央映射树不进树表，这里没有它）。
-const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 11] = [
+/// 十二棵进树表的树的种类，按发号次序（D8（核心索引结构） 已定项 8 ②：extent、inode、分配记录、记账、中央映射、livelist、稀疏旁表、deadlist，
+/// 再接 D18（块里携带什么信息） 已定项 24 第 6 条的 `deleted_inodes`、墓碑树、谱系树、logged_ops，最后是退役树（D8 已定项 9 码 13）；
+/// 中央映射树不进树表，这里没有它）。
+const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 12] = [
     TREE_KIND_EXTENT,
     TREE_KIND_INODE,
     TREE_KIND_ALLOCATION,
@@ -2381,6 +2386,7 @@ const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 11] = [
     TREE_KIND_TOMBSTONE,
     TREE_KIND_LINEAGE,
     TREE_KIND_LOGGED_OPERATIONS,
+    TREE_KIND_RETIREMENT,
 ];
 
 /// I-9.16（树表条目按树 ID 严格升序且合发号次序）：先判 ①（盘上次序），再判 ②（发号次序）。读树表取条目、要往下走的两处都在
@@ -2579,6 +2585,7 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
         tombstone: tree_of(TREE_KIND_TOMBSTONE)?,
         lineage: tree_of(TREE_KIND_LINEAGE)?,
         logged_operations: tree_of(TREE_KIND_LOGGED_OPERATIONS)?,
+        retirement: tree_of(TREE_KIND_RETIREMENT)?,
     };
     // 豁免三类之外的单元（数据单元、四棵树的根、inode 叶容器）位置提示读不出时经这一版的中央映射回退，与冷走读同一条
     // （D19（块指针的结构与宽度预算） 已定项 8）；映射树根、树表、实例表是自举豁免，只按根记录里的位置条目读。
@@ -2693,6 +2700,8 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
             // 身份取容器头那一份：条目里的身份引用与它逐字相等是 I-9.2（条目身份与子头相符），上面刚判过。
             contents: InodeLeafContainer {
                 identity: leaf.identity,
+                // 分隔 key 取条目里那一份：诞生时是容器号，被左邻借过之后是被借那一版的最小 key（D8（核心索引结构） 已定项 6，N1）。
+                separator_key_in_the_parent_entry: separator_key,
                 records,
             },
             pointer: leaf_pointer,
@@ -2823,7 +2832,7 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
             what_index_node: "谱系树的码 2 节点不是非空、层级不合父层级、或类型段 0 的条目身份引用不全 0、分隔 key 不是孩子的首 key",
             what_entry: "谱系树内部条目",
             what_container: "谱系容器",
-            what_container_against_its_entry: "谱系容器与父节点里指它的条目对不上（身份、类型 5、出生树、分隔 key = 容器号、记录宽 16、非空）",
+            what_container_against_its_entry: "谱系容器与父节点里指它的条目对不上（身份、类型 5、出生树、分隔 key = 容器号、记录宽 24、非空）",
             what_record: "谱系记录",
         },
         &lineage_root_pointer,
@@ -2831,6 +2840,27 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
         expected_filesystem_identifier,
         &mut read_container_tree_node,
     )?;
+    // 退役树（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条：形态照墓碑树，容器打包记录类型 3、记录宽 34）：
+    // 接着这一版发布要往开放容器追加、算内部层数，整棵读回来，同墓碑树、谱系树。
+    let retirement_root_pointer = pointer_of(TREE_KIND_RETIREMENT)?;
+    let retirement_tree_read = read_container_tree(
+        &ContainerTreeToRead {
+            tree: tree_identifiers.retirement,
+            kind: TREE_KIND_RETIREMENT,
+            record_type: PACKED_TYPE_RETIREMENT,
+            record_width_in_bytes: RETIREMENT_RECORD_BYTES,
+            parse_record: RetirementRecord::parse,
+            what_index_node: "退役树的码 2 节点不是非空、层级不合父层级、或类型段 0 的条目身份引用不全 0、分隔 key 不是孩子的首 key",
+            what_entry: "退役树内部条目",
+            what_container: "退役容器",
+            what_container_against_its_entry: "退役容器与父节点里指它的条目对不上（身份、类型 3、出生树、分隔 key = 容器号、记录宽 34、非空）",
+            what_record: "退役记录",
+        },
+        &retirement_root_pointer,
+        root,
+        expected_filesystem_identifier,
+        &mut read_container_tree_node,
+    )?;
     // 分配记录树按绝对槽号按位置寻址（D8（核心索引结构） 已定项 14）：整棵读回来，每个节点按位置核，节点进映射、提示读不出经映射回退。
     let allocation_tree_read = read_allocation_record_tree(
         &allocation_pointer,
@@ -2884,7 +2914,8 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
         },
     )?;
     accounting_entry_count_is_three_plus_six_per_device(
-        accounting_tree.leaf_entries_in_key_order.len(),
+        &accounting_tree.leaf_entries_in_key_order,
+        root.checkpoint_txg,
         device_identities.len(),
     )?;
     let mut accounting_entries: Vec<AccountingEntry> =
@@ -2896,10 +2927,11 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
     }
     // 记账行的统计量标签是盘上读来的 2 字节：这一版交出去之前判一次「inode 号水位那一行在」，往里就信
     // （`TransactionOutput::inode_number_watermark` 那句 expect 依赖的就是这一判；panic 面普查 R11）。
-    if !accounting_entries
-        .iter()
-        .any(|entry| entry.statistic == STATISTIC_INODE_WATERMARK)
-    {
+    // 要的是这一版自己写的那一行（代段 = 根的 txg）：那一统计量另有代段更早的台阶行（D5（快照 / 空间记账机制） 已定项 2 射程），
+    // 只剩台阶行时现行值取到的是一级台阶，不是这一版的水位。
+    if !accounting_entries.iter().any(|entry| {
+        entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation == root.checkpoint_txg
+    }) {
         return Err(RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree.into());
     }
     // 映射树是自举豁免：不经映射回退，只按父指针里的位置条目读；上面有提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份。
@@ -2917,6 +2949,9 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
                     .as_ref()
                     .map_or(0, ContainerTreeRead::mapped_units)
                 + lineage_tree_read
+                    .as_ref()
+                    .map_or(0, ContainerTreeRead::mapped_units)
+                + retirement_tree_read
                     .as_ref()
                     .map_or(0, ContainerTreeRead::mapped_units),
         },
@@ -3002,6 +3037,12 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
                 .map(ContainerTreeRead::units_in_bump_order),
             CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE,
         ),
+        (
+            retirement_tree_read
+                .as_ref()
+                .map(ContainerTreeRead::units_in_bump_order),
+            CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE,
+        ),
     ] {
         for (unit_of_the_tree, pointer, _) in tree_read.unwrap_or_default() {
             mapped_units.push((
@@ -3079,6 +3120,12 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
                 .map(ContainerTreeRead::units_in_bump_order),
             CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE,
         ),
+        (
+            retirement_tree_read
+                .as_ref()
+                .map(ContainerTreeRead::units_in_bump_order),
+            CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE,
+        ),
     ] {
         for (unit_of_the_tree, pointer, bytes) in tree_read.unwrap_or_default() {
             deletion_tree_units.push(unit(roles.of(unit_of_the_tree), &pointer.locations, bytes));
@@ -3168,6 +3215,19 @@ pub fn rebuild_version_in_the_unit_area_starting_at(
             ContainerTreeIndexNodesVersion::zero_root,
             ContainerTreeRead::index_nodes_version,
         ),
+        retirement_containers: retirement_tree_read.as_ref().map_or_else(Vec::new, |read| {
+            read.containers
+                .iter()
+                .map(|(contents, pointer, _)| RetirementContainerVersion {
+                    contents: contents.clone(),
+                    pointer: *pointer,
+                })
+                .collect()
+        }),
+        retirement_tree_index_nodes: retirement_tree_read.as_ref().map_or_else(
+            ContainerTreeIndexNodesVersion::zero_root,
+            ContainerTreeRead::index_nodes_version,
+        ),
         inode_leaf_containers,
         mapped_units,
         released: Vec::new(),
@@ -3234,7 +3294,8 @@ pub fn user_visible_tree_root_pointers(
             | TREE_KIND_DELETED_INODES
             | TREE_KIND_TOMBSTONE
             | TREE_KIND_LINEAGE
-            | TREE_KIND_LOGGED_OPERATIONS => continue,
+            | TREE_KIND_LOGGED_OPERATIONS
+            | TREE_KIND_RETIREMENT => continue,
             _ => {
                 return Err(RecoveryFailure::UnitMalformed {
                     what: "树的种类没登记",
@@ -3770,7 +3831,9 @@ fn key_width_for_kind(kind: u16) -> Option<usize> {
         // `deleted_inodes` 的 key 是 inode 号 8 字节（D8（核心索引结构） 已定项 16 第 1 条）；墓碑树与谱系树的根条目照 inode 树内部节点的
         // 120 字节形态，key 是分隔 key = 容器号 8 字节（D18（块里携带什么信息） 已定项 24 第 1、5 条）。
         TREE_KIND_DELETED_INODES => Some(DELETED_INODES_KEY_WIDTH_IN_BYTES),
-        TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE => Some(CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES),
+        TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE | TREE_KIND_RETIREMENT => {
+            Some(CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES)
+        }
         // logged_ops 的字段表留到第一个写者落地时定（D8（核心索引结构） 已定项 5、C364（整理意图记录没有字段表））。
         TREE_KIND_LIVELIST
         | TREE_KIND_SPARSE_SIDE_TABLE
@@ -3875,6 +3938,17 @@ const CONTAINER_TREE_ROLES_OF_THE_LINEAGE_TREE: ContainerTreeRolesOfOneTree =
         node_below_the_root: TransactionUnit::LineageNodeBelowTheRoot,
     };
 
+fn retirement_container_role_of_position(position: usize) -> TransactionUnit {
+    TransactionUnit::RetirementLeafContainer(RetirementContainerIndexInTree::of_position(position))
+}
+
+const CONTAINER_TREE_ROLES_OF_THE_RETIREMENT_TREE: ContainerTreeRolesOfOneTree =
+    ContainerTreeRolesOfOneTree {
+        container: retirement_container_role_of_position,
+        root: TransactionUnit::RetirementRoot,
+        node_below_the_root: TransactionUnit::RetirementNodeBelowTheRoot,
+    };
+
 /// 墓碑树、谱系树码 2 节点条目里的子指针头部 extent 偏移非 0 时报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句
 /// （C581（extent 偏移非 0 没有判据）：判该指针所在的结构损坏，与 inode 树内部条目同一条）。
 pub const CONTAINER_TREE_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO: &str =
@@ -4688,15 +4762,26 @@ fn allocation_records_are_one_placement_per_device_on_every_device(
     Ok(())
 }
 
-/// 记账条目数是 3 + 6 × 盘数（D5（快照 / 空间记账机制） 已定项 8：池级三行加每盘六行）。
+/// 记账条目数是 3 + 6 × 盘数（D5（快照 / 空间记账机制） 已定项 8：池级三行加每盘六行），inode 号水位的台阶行不算在里面：
+/// 它们是那一统计量代段早于这一版 txg（`root_txg`）的行，随根一直留着、条数随建过 inode 的发布数涨（D5 已定项 2 射程「inode 号水位的台阶行」）。
+/// 解不开的条目照算进 3 + 6 × 盘数（它在后面逐条解的时候报宽度不对）。
 fn accounting_entry_count_is_three_plus_six_per_device(
-    accounting_entries: usize,
+    accounting_entry_bytes: &[Vec<u8>],
+    root_txg: CheckpointTxg,
     device_count: usize,
 ) -> Result<(), RecoveryFailure> {
-    if accounting_entries != 3 + 6 * device_count {
+    let entries_other_than_the_inode_number_watermark_steps = accounting_entry_bytes
+        .iter()
+        .filter(|bytes| {
+            !AccountingEntry::parse(bytes).is_some_and(|entry| {
+                entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg
+            })
+        })
+        .count();
+    if entries_other_than_the_inode_number_watermark_steps != 3 + 6 * device_count {
         return Err(RecoveryFailure::InvariantViolated {
             invariant: "E142 走读同款",
-            detail: "记账条目数不是 3 + 6 × 盘数",
+            detail: "记账条目数（inode 号水位的台阶行不算）不是 3 + 6 × 盘数",
         });
     }
     Ok(())
@@ -5196,12 +5281,15 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
             continue;
         }
         // 墓碑树与谱系树：从根往下读码 2 节点（根满长高之后不止一层）、数进映射的节点与容器，不读容器（它们的内容不影响读回的文件）。
-        if entry.kind == TREE_KIND_TOMBSTONE || entry.kind == TREE_KIND_LINEAGE {
+        if entry.kind == TREE_KIND_TOMBSTONE
+            || entry.kind == TREE_KIND_LINEAGE
+            || entry.kind == TREE_KIND_RETIREMENT
+        {
             let index_nodes = read_container_tree_index_nodes(
                 entry.tree,
                 entry.kind,
-                "墓碑树或谱系树的码 2 节点不是非空、层级不合父层级、或类型段 0 的条目身份引用不全 0、分隔 key 不是孩子的首 key",
-                "墓碑树或谱系树内部条目",
+                "墓碑树、谱系树或退役树的码 2 节点不是非空、层级不合父层级、或类型段 0 的条目身份引用不全 0、分隔 key 不是孩子的首 key",
+                "墓碑树、谱系树或退役树内部条目",
                 &entry.root,
                 root,
                 expected_filesystem_identifier,
@@ -5248,7 +5336,8 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
     };
     let deletion_tree_nodes = usize::from(by_kind.contains_key(&TREE_KIND_DELETED_INODES))
         + container_tree_nodes(TREE_KIND_TOMBSTONE)
-        + container_tree_nodes(TREE_KIND_LINEAGE);
+        + container_tree_nodes(TREE_KIND_LINEAGE)
+        + container_tree_nodes(TREE_KIND_RETIREMENT);
     let mut take = |kind: u16| {
         by_kind.remove(&kind).ok_or(RecoveryFailure::UnitMalformed {
             what: "树表里缺一棵有根的树",
@@ -5279,7 +5368,8 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
         &device_identities,
     )?;
     accounting_entry_count_is_three_plus_six_per_device(
-        roots.accounting.leaf_entries_in_key_order.len(),
+        &roots.accounting.leaf_entries_in_key_order,
+        root.checkpoint_txg,
         device_count,
     )?;
     mapping_entry_count_is_one_per_mapped_unit(
@@ -6176,7 +6266,7 @@ mod tree_kind_key_width_tests {
     use super::key_width_for_kind;
     use crate::records::{
         TREE_KIND_DELETED_INODES, TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LOGGED_OPERATIONS,
-        TREE_KIND_TOMBSTONE,
+        TREE_KIND_RETIREMENT, TREE_KIND_TOMBSTONE,
     };
 
     #[test]
@@ -6197,6 +6287,11 @@ mod tree_kind_key_width_tests {
             Some(8),
             "谱系树根条目的分隔 key = 容器号 8 字节"
         );
+        assert_eq!(
+            key_width_for_kind(TREE_KIND_RETIREMENT),
+            Some(8),
+            "退役树根条目的分隔 key = 容器号 8 字节"
+        );
         assert_eq!(
             key_width_for_kind(TREE_KIND_LOGGED_OPERATIONS),
             None,
diff --git a/crates/singlefs-core/src/retirement_tree.rs b/crates/singlefs-core/src/retirement_tree.rs
new file mode 100644
index 0000000..c44d1de
--- /dev/null
+++ b/crates/singlefs-core/src/retirement_tree.rs
@@ -0,0 +1,391 @@
+//! 退役树的叶容器（D18（块里携带什么信息） 已定项 11 打包记录类型 3；判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2–5 条）：
+//! 码 3 打包记录类型 3，一容器装 959 条 34 字节退役记录；根恒为码 2，条目照 inode 树内部节点的 120 字节形态，形态照墓碑树
+//! （共用的那一半在 `crate::container_tree`）。
+//!
+//! 压着它的条款，逐字（判决第三节）：
+//! - 「死亡 txg = 身份消失那次发布的 checkpoint_txg（删空摘掉、合并被吸收那次）」（第 3 条）；
+//! - 「写者只追加：同一身份跨回退可以有多条，不去重、不覆盖」（第 4 条）；
+//! - 「容器号从 1 起、下一个号 = 树里最大号 + 1；有新记录的发布才重写开放容器；满了才关；第一版不回收；根满长高」（第 5 条）。
+//!
+//! 一次发布的退役记录条数不设上限（第二节 K4「条数不设上界」那一行）：一次删除连着吸收几片时一次追加几条，开放容器装满就开下一片
+//! （[`append_retirement_records_to_the_open_container`]）。和 `crate::tombstone_tree` 一样，这里只算「这次发布之后树里是哪几片容器、
+//! 各装哪些记录、哪几片要重写」，不发一个写、不动分配器、不碰落点。
+
+use singlefs_format::RETIREMENT_CONTAINER_RECORDS;
+
+use crate::address::{CheckpointTxg, TreeIdentifier};
+use crate::container_tree::{
+    append_records_to_the_open_container, ContainerTreeAfterThisPublish, ContainerTreeNodeEntries,
+    ContainerTreeShape, RecordContainer,
+};
+use crate::records::RetirementRecord;
+use crate::unit::{PackedIdentity, PACKED_TYPE_RETIREMENT};
+
+/// 退役树里第几片叶容器，按叶序（= 容器号升序）从 0 数。它是树里的位置，不是容器号（容器号从 1 起）。
+/// 第一版不回收 ⇒ 容器只在最右端出现、一片都不删，位置一旦定下就不再挪，可以当发布路径里的角色序号用。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub struct RetirementContainerIndexInTree(pub u32);
+
+impl RetirementContainerIndexInTree {
+    /// 当下标用。
+    #[must_use]
+    pub fn position(self) -> usize {
+        usize::try_from(self.0).expect("u32 的退役容器序号装得进 usize")
+    }
+
+    /// 第几片，从 0 数。
+    ///
+    /// # Panics
+    /// 序号装不进 u32：要 2^32 片退役容器（约 4 × 10^12 次叶容器退役），走不到。
+    #[must_use]
+    pub fn of_position(position: usize) -> Self {
+        Self(u32::try_from(position).expect("退役容器片数装得进 u32：要 2^32 片才装不下"))
+    }
+}
+
+/// 一片退役容器这一版装的东西：身份四段（出生树 = 退役树、打包记录类型 3、容器号、容器出生代）与它的记录，按追加的先后。
+pub type RetirementContainer = RecordContainer<RetirementRecord>;
+
+/// 只供测试压小的每片退役容器条数（`.claude/rules/fs-design.md` 五条硬要求第 2 条：「满了才关、开下一片」这一支按产品容量
+/// 要 959 次叶容器退役才走得到）。至少 1、至多产品容量 959；只住内存，不落盘（同墓碑容器的旋钮）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct RetirementRecordsPerContainerCappedForTests(u64);
+
+impl RetirementRecordsPerContainerCappedForTests {
+    /// 0 条或多于产品容量 959 交回 `None`。
+    #[must_use]
+    pub fn new(records: u64) -> Option<Self> {
+        (1..=RETIREMENT_CONTAINER_RECORDS)
+            .contains(&records)
+            .then_some(Self(records))
+    }
+
+    #[must_use]
+    pub const fn records(self) -> u64 {
+        self.0
+    }
+}
+
+/// 一片退役容器装满几条就关：产品路径按容器格式算（959），只供测试的开关把它压小。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RetirementContainerCapacity {
+    FromTheContainerFormat,
+    CappedForTests(RetirementRecordsPerContainerCappedForTests),
+}
+
+impl RetirementContainerCapacity {
+    /// 运行时报出走的是哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支必须可观测）。
+    #[must_use]
+    pub const fn name(self) -> &'static str {
+        match self {
+            RetirementContainerCapacity::FromTheContainerFormat => {
+                "retirement_container_capacity_from_the_container_format"
+            }
+            RetirementContainerCapacity::CappedForTests(_) => {
+                "retirement_container_capacity_capped_for_tests"
+            }
+        }
+    }
+
+    /// 一片装满几条。
+    #[must_use]
+    pub const fn records_per_container(self) -> u64 {
+        match self {
+            RetirementContainerCapacity::FromTheContainerFormat => RETIREMENT_CONTAINER_RECORDS,
+            RetirementContainerCapacity::CappedForTests(capped) => capped.records(),
+        }
+    }
+}
+
+/// 这次发布之后退役树的样子：叶容器、这次重写哪几片（叶序）、这次前后的形状（`crate::container_tree`）。
+pub type RetirementTreeAfterThisPublish = ContainerTreeAfterThisPublish<RetirementRecord>;
+
+/// 这次发布要追加的退役记录：`retired_identities` 里每个身份一条，死亡 txg 都是这次发布的 checkpoint_txg，次序照给的次序
+/// （调用方按叶容器在上一版里的叶序给，同一个参数两次发布逐字节相同）。不去重：写者只追加（判决第三节第 4 条）。
+#[must_use]
+pub fn retirement_records_of_this_publish(
+    retired_identities: &[PackedIdentity],
+    checkpoint_txg: CheckpointTxg,
+) -> Vec<RetirementRecord> {
+    retired_identities
+        .iter()
+        .map(|retired| RetirementRecord {
+            retired: *retired,
+            death_txg: checkpoint_txg,
+        })
+        .collect()
+}
+
+/// 往退役树的开放容器（最右那一片）追加这次发布的退役记录（删空摘掉、被左邻吸收的叶容器身份，各一条）：
+/// 装得下就追加进最右那片；装满了（或零根）就开一片新容器，容器号 = 树里最大号 + 1（零根时 1），出生代 = 这次发布的 checkpoint_txg，
+/// 出生树 = `retirement_tree`，打包记录类型 3；根满时长高。一条都没有时整棵照抄（这次不重写退役树）。
+#[must_use]
+pub fn append_retirement_records_to_the_open_container(
+    containers_before_this_publish: &[RetirementContainer],
+    shape_before_this_publish: &ContainerTreeShape,
+    records: &[RetirementRecord],
+    checkpoint_txg: CheckpointTxg,
+    retirement_tree: TreeIdentifier,
+    capacity: RetirementContainerCapacity,
+    node_entries: ContainerTreeNodeEntries,
+) -> RetirementTreeAfterThisPublish {
+    append_records_to_the_open_container(
+        containers_before_this_publish,
+        shape_before_this_publish,
+        records,
+        checkpoint_txg,
+        retirement_tree,
+        PACKED_TYPE_RETIREMENT,
+        usize::try_from(capacity.records_per_container()).expect("至多 959 条"),
+        node_entries,
+    )
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::container_tree::{
+        containers_one_container_tree_root_holds, ContainerTreeNodeEntriesCappedForTests,
+    };
+    use crate::unit::PACKED_TYPE_INODE;
+    use singlefs_format::{TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_RETIREMENT};
+
+    const RETIREMENT_TREE: TreeIdentifier = TreeIdentifier(TREE_IDENTIFIER_RETIREMENT);
+
+    fn retired_leaf(container: u64, death_txg: u64) -> RetirementRecord {
+        RetirementRecord {
+            retired: PackedIdentity {
+                birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
+                record_type: PACKED_TYPE_INODE,
+                container,
+                container_birth: CheckpointTxg(3),
+            },
+            death_txg: CheckpointTxg(death_txg),
+        }
+    }
+
+    fn capped(records: u64) -> RetirementContainerCapacity {
+        RetirementContainerCapacity::CappedForTests(
+            RetirementRecordsPerContainerCappedForTests::new(records).expect("1..=959"),
+        )
+    }
+
+    /// 一路追加：第 txg 次发布追加一条，交回最后一次之后的样子。
+    fn after_appending_one_per_publish(
+        publishes: u64,
+        capacity: RetirementContainerCapacity,
+        node_entries: ContainerTreeNodeEntries,
+    ) -> RetirementTreeAfterThisPublish {
+        let mut after =
+            RetirementTreeAfterThisPublish::carried(Vec::new(), ContainerTreeShape::zero_root());
+        for txg in 1..=publishes {
+            after = append_retirement_records_to_the_open_container(
+                &after.containers,
+                &after.shape,
+                &[retired_leaf(txg * 1000, txg)],
+                CheckpointTxg(txg),
+                RETIREMENT_TREE,
+                capacity,
+                node_entries,
+            );
+        }
+        after
+    }
+
+    /// 零根时第一条退役记录开第一片容器：容器号 1、出生代 = 这次发布的 txg、打包记录类型 3、出生树 = 退役树；
+    /// 下一次发布追加进同一片（开放容器），身份不变、只重写它。
+    #[test]
+    fn the_first_retirement_record_opens_container_one_and_the_next_publish_appends_to_the_open_container(
+    ) {
+        let first = append_retirement_records_to_the_open_container(
+            &[],
+            &ContainerTreeShape::zero_root(),
+            &[retired_leaf(234, 7)],
+            CheckpointTxg(7),
+            RETIREMENT_TREE,
+            RetirementContainerCapacity::FromTheContainerFormat,
+            ContainerTreeNodeEntries::FromTheNodeFormat,
+        );
+        let identity = PackedIdentity {
+            birth_tree: RETIREMENT_TREE,
+            record_type: PACKED_TYPE_RETIREMENT,
+            container: 1,
+            container_birth: CheckpointTxg(7),
+        };
+        assert_eq!(
+            first.containers,
+            vec![RetirementContainer {
+                identity,
+                records: vec![retired_leaf(234, 7)],
+            }],
+            "判决第三节第 5 条：容器号从 1 起，类型 3，出生树 = 退役树"
+        );
+        assert_eq!(first.rewritten_containers, vec![0]);
+        assert_eq!(first.shape.root_level(), 1);
+        let second = append_retirement_records_to_the_open_container(
+            &first.containers,
+            &first.shape,
+            &[retired_leaf(467, 9)],
+            CheckpointTxg(9),
+            RETIREMENT_TREE,
+            RetirementContainerCapacity::FromTheContainerFormat,
+            ContainerTreeNodeEntries::FromTheNodeFormat,
+        );
+        assert_eq!(
+            second.containers,
+            vec![RetirementContainer {
+                identity,
+                records: vec![retired_leaf(234, 7), retired_leaf(467, 9)],
+            }],
+            "有新记录的发布重写开放容器，身份（出生代 7）不变"
+        );
+        assert_eq!(second.rewritten_containers, vec![0]);
+        assert_eq!(second.shape, first.shape, "没开新容器：形状不变");
+    }
+
+    /// 没有新退役记录的发布整棵照抄：一片都不重写，根也不重写。
+    #[test]
+    fn a_publish_without_retirement_records_carries_the_whole_tree() {
+        let before = after_appending_one_per_publish(
+            2,
+            RetirementContainerCapacity::FromTheContainerFormat,
+            ContainerTreeNodeEntries::FromTheNodeFormat,
+        );
+        let after = append_retirement_records_to_the_open_container(
+            &before.containers,
+            &before.shape,
+            &[],
+            CheckpointTxg(3),
+            RETIREMENT_TREE,
+            RetirementContainerCapacity::FromTheContainerFormat,
+            ContainerTreeNodeEntries::FromTheNodeFormat,
+        );
+        assert_eq!(after.containers, before.containers);
+        assert!(!after.is_the_root_rewritten());
+        assert!(after.rewritten_nodes().is_empty());
+    }
+
+    /// 开放容器满了就关、开下一片（产品容量 959，这里压到 2）：容器号 = 最大号 + 1、出生代 = 这次的 txg；满了的那片一个字节都不动。
+    /// 一次发布追加两条、开放容器只剩一格时：第一条装满它，第二条开新片，两片都重写。
+    #[test]
+    fn a_full_open_container_is_closed_and_a_publish_with_two_records_can_span_two_containers() {
+        let two = after_appending_one_per_publish(
+            2,
+            capped(2),
+            ContainerTreeNodeEntries::FromTheNodeFormat,
+        );
+        let third = append_retirement_records_to_the_open_container(
+            &two.containers,
+            &two.shape,
+            &[retired_leaf(5000, 11)],
+            CheckpointTxg(11),
+            RETIREMENT_TREE,
+            capped(2),
+            ContainerTreeNodeEntries::FromTheNodeFormat,
+        );
+        assert_eq!(third.containers.len(), 2, "第一片满 2 条 ⇒ 开第二片");
+        assert_eq!(third.containers[0], two.containers[0], "满了的那片原样");
+        assert_eq!(third.containers[1].identity.container, 2, "最大号 1 + 1");
+        assert_eq!(
+            third.containers[1].identity.container_birth,
+            CheckpointTxg(11)
+        );
+        assert_eq!(third.rewritten_containers, vec![1], "只重写新开的那一片");
+        let spanning = append_retirement_records_to_the_open_container(
+            &third.containers,
+            &third.shape,
+            &[retired_leaf(6000, 12), retired_leaf(7000, 12)],
+            CheckpointTxg(12),
+            RETIREMENT_TREE,
+            capped(2),
+            ContainerTreeNodeEntries::FromTheNodeFormat,
+        );
+        assert_eq!(
+            spanning
+                .containers
+                .iter()
+                .map(|container| container.records.len())
+                .collect::<Vec<_>>(),
+            vec![2, 2, 1],
+            "第二片装满、第三片开出来装一条"
+        );
+        assert_eq!(spanning.rewritten_containers, vec![1, 2]);
+        assert_eq!(spanning.containers[2].identity.container, 3);
+    }
+
+    /// 根装满之后再开一片：长一层——产品扇出 135 片之后的第 136 片；长高那次旧根留在 (1, 0) 照抄，新开的容器、右半与新根重写。
+    #[test]
+    fn opening_a_container_past_what_one_root_holds_grows_the_retirement_tree_one_level() {
+        assert_eq!(containers_one_container_tree_root_holds(), 135);
+        let full_root = after_appending_one_per_publish(
+            135,
+            capped(1),
+            ContainerTreeNodeEntries::FromTheNodeFormat,
+        );
+        assert_eq!(full_root.shape.root_level(), 1);
+        let grown = append_retirement_records_to_the_open_container(
+            &full_root.containers,
+            &full_root.shape,
+            &[retired_leaf(999_999, 136)],
+            CheckpointTxg(136),
+            RETIREMENT_TREE,
+            capped(1),
+            ContainerTreeNodeEntries::FromTheNodeFormat,
+        );
+        assert_eq!(grown.shape.root_level(), 2, "第 136 片：根满长一层");
+        assert!(grown.grew_a_level_this_publish());
+        assert!(grown.previous_root_is_carried_down_unchanged());
+        assert_eq!(grown.internal_levels(), 1);
+        let node_entries_of_two = ContainerTreeNodeEntries::CappedForTests(
+            ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135"),
+        );
+        let three = after_appending_one_per_publish(3, capped(1), node_entries_of_two);
+        assert_eq!(three.shape.root_level(), 2, "扇出 2 的第 3 片长一层");
+    }
+
+    /// 写者只追加：同一身份退役两次（删 → 回退到删之前 → 再删）两条都在，不去重、不覆盖。
+    #[test]
+    fn the_same_identity_retired_twice_keeps_both_records() {
+        let first = append_retirement_records_to_the_open_container(
+            &[],
+            &ContainerTreeShape::zero_root(),
+            &[retired_leaf(234, 10)],
+            CheckpointTxg(10),
+            RETIREMENT_TREE,
+            RetirementContainerCapacity::FromTheContainerFormat,
+            ContainerTreeNodeEntries::FromTheNodeFormat,
+        );
+        let second = append_retirement_records_to_the_open_container(
+            &first.containers,
+            &first.shape,
+            &[retired_leaf(234, 12)],
+            CheckpointTxg(12),
+            RETIREMENT_TREE,
+            RetirementContainerCapacity::FromTheContainerFormat,
+            ContainerTreeNodeEntries::FromTheNodeFormat,
+        );
+        assert_eq!(
+            second.containers[0].records,
+            vec![retired_leaf(234, 10), retired_leaf(234, 12)]
+        );
+    }
+
+    /// 只供测试的容量至少 1、至多产品容量 959。
+    #[test]
+    fn the_capacity_capped_for_tests_lies_between_one_and_the_product_capacity() {
+        assert_eq!(RetirementRecordsPerContainerCappedForTests::new(0), None);
+        assert_eq!(RetirementRecordsPerContainerCappedForTests::new(960), None);
+        assert_eq!(
+            RetirementRecordsPerContainerCappedForTests::new(959).map(|capped| capped.records()),
+            Some(959)
+        );
+        assert_eq!(
+            retirement_records_of_this_publish(
+                &[retired_leaf(1, 4).retired, retired_leaf(234, 4).retired],
+                CheckpointTxg(8)
+            ),
+            vec![retired_leaf(1, 8), retired_leaf(234, 8)],
+            "死亡 txg 都取这次发布的 txg，次序照给的次序"
+        );
+    }
+}
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 2065101..b77b191 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -12,13 +12,14 @@ use std::collections::{BTreeMap, BTreeSet};
 use singlefs_format::{
     ACCOUNTING_ENTRY_BYTES, DELETED_INODES_ENTRY_BYTES, EXTENT_KEY_BYTES, INODE_INTERNAL_ENTRY,
     INODE_RECORD_BYTES, INSTANCE_ROW_BYTES, JOURNAL_NAMED_ENTRIES_PER_RECORD, LINEAGE_RECORD_BYTES,
-    MAPPING_ENTRY_BYTES, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, TOMBSTONE_RECORD_BYTES,
-    TREE_IDENTIFIER_ACCOUNTING, TREE_IDENTIFIER_ALLOCATION_RECORDS,
-    TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST, TREE_IDENTIFIER_DELETED_INODES,
-    TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LINEAGE,
-    TREE_IDENTIFIER_LIVELIST, TREE_IDENTIFIER_LOGGED_OPERATIONS, TREE_IDENTIFIER_SPARSE_SIDE_TABLE,
-    TREE_IDENTIFIER_TOMBSTONE, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
-    TREE_IDENTIFIER_WATERMARK_AT_MKFS, TREE_TABLE_ENTRY_BYTES, WARM_UP_EMPTY_PUBLISHES,
+    MAPPING_ENTRY_BYTES, RETIREMENT_RECORD_BYTES, SLOT_BYTES,
+    SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, TOMBSTONE_RECORD_BYTES, TREE_IDENTIFIER_ACCOUNTING,
+    TREE_IDENTIFIER_ALLOCATION_RECORDS, TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST,
+    TREE_IDENTIFIER_DELETED_INODES, TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE,
+    TREE_IDENTIFIER_LINEAGE, TREE_IDENTIFIER_LIVELIST, TREE_IDENTIFIER_LOGGED_OPERATIONS,
+    TREE_IDENTIFIER_RETIREMENT, TREE_IDENTIFIER_SPARSE_SIDE_TABLE, TREE_IDENTIFIER_TOMBSTONE,
+    TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+    TREE_TABLE_ENTRY_BYTES, WARM_UP_EMPTY_PUBLISHES,
 };
 
 use crate::address::{
@@ -29,10 +30,11 @@ use crate::admission::{
     admission_reading_before_a_publish_on_a_ring_of, admit_on_every_device,
     admit_the_units_landing_on_every_device,
     checkpoint_cost_of_the_version_to_build_on_with_node_capacities,
-    committed_reservation_for_tombstones, demand_of_the_roles_on_each_device,
+    committed_reservation_for_tombstones_and_retirement_records,
+    demand_of_the_roles_on_each_device, inode_records_of_the_inode_tree,
     live_objects_of_the_inode_tree, publish_has_an_ordinary_allocation, units_of_a_publish_to_land,
     AdmissionRefusedOnSomeDevices, BytesSummedOverAllReplicas, PlacementRoomOnOneDevice,
-    ReplicaCount, SpaceAdmission, TombstonePrepayBasis,
+    ReplicaCount, SpaceAdmission, TombstoneAndRetirementPrepayBasis,
 };
 use crate::allocation_record_tree::{
     build_allocation_record_tree_node, children_of, nodes_holding_records,
@@ -67,7 +69,8 @@ use crate::extent_tree::{
 };
 use crate::inode_tree::{
     remove_record_from_leaf_containers, write_records_into_leaf_containers, InodeLeafContainer,
-    InodeLeafContainerIndexInTree, InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal,
+    InodeLeafContainerIndexInTree, InodeLeafContainerOrigin, InodeLeafContainersAfterThisPublish,
+    InodeTreeWriteRefusal,
 };
 use crate::instance_table::{
     instance_table_page_records, instance_table_pages_for_rows, instance_table_rows_of_each_page,
@@ -91,7 +94,7 @@ use crate::pointer::{
 use crate::records::{
     build_extent_record, build_inode_internal_entry, build_mapping_entry, data_key_tail,
     mapping_key_for_data, mapping_key_for_node, node_key_tail, parse_mapping_entry,
-    AccountingEntry, InodeRecord, LineageRecord, TombstoneRecord, TreeTableEntry,
+    AccountingEntry, InodeRecord, LineageRecord, RetirementRecord, TombstoneRecord, TreeTableEntry,
     ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF, NLINK_OF_AN_INODE_BEING_DELETED,
     NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES, STATISTIC_ALLOCATED_BYTES,
     STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_DEFER_QUEUE_BYTES,
@@ -99,8 +102,8 @@ use crate::records::{
     STATISTIC_INODE_WATERMARK, STATISTIC_NO_DEVICE_DIMENSION, STATISTIC_PENDING_DELETE_BYTES,
     STATISTIC_UNRECLAIMABLE_BYTES, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
     TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LINEAGE,
-    TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_SPARSE_SIDE_TABLE,
-    TREE_KIND_TOMBSTONE,
+    TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_RETIREMENT,
+    TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
 };
 use crate::recovery::{
     effective_rollback_floor_rereading_unreadable_reads_once, highest_root_instance,
@@ -109,6 +112,10 @@ use crate::recovery::{
     verified_system_configuration_slots, PoolReader, RecoveryFailure,
     SystemConfigurationSlotsOfOneDevice,
 };
+use crate::retirement_tree::{
+    append_retirement_records_to_the_open_container, retirement_records_of_this_publish,
+    RetirementContainer, RetirementContainerIndexInTree, RetirementTreeAfterThisPublish,
+};
 use crate::root_record::{RootRecord, UnmountMarker};
 use crate::root_ring::{slot_offset, target_for_publish};
 use crate::system_configuration::{
@@ -1972,6 +1979,9 @@ pub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(
             | TransactionUnit::LineageLeafContainer(_)
             | TransactionUnit::LineageRoot
             | TransactionUnit::LineageNodeBelowTheRoot(_)
+            | TransactionUnit::RetirementLeafContainer(_)
+            | TransactionUnit::RetirementRoot
+            | TransactionUnit::RetirementNodeBelowTheRoot(_)
             | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
             | TransactionUnit::MappingTree
             | TransactionUnit::TreeTable => unreachable!(
@@ -2302,6 +2312,13 @@ pub enum TransactionUnit {
     LineageRoot,
     /// 谱系树根之下的一个码 2 节点（根满长高之后才有），同 `TombstoneNodeBelowTheRoot`。
     LineageNodeBelowTheRoot(ContainerTreeNodePosition),
+    /// 退役树的一片叶容器（码 3 打包记录类型 3，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）：带它在树里的叶序。
+    /// 只有身份消失（删空摘掉、被左邻吸收）的那次发布追加记录、重写它；别的发布照抄。
+    RetirementLeafContainer(RetirementContainerIndexInTree),
+    /// 退役树的根（码 2，条目同墓碑树根）：树里有容器的一版才有它。
+    RetirementRoot,
+    /// 退役树根之下的一个码 2 节点（根满长高之后才有），同 `TombstoneNodeBelowTheRoot`。
+    RetirementNodeBelowTheRoot(ContainerTreeNodePosition),
     /// 中央映射树里根之下的一个节点，同 `AccountingTreeNodeBelowTheRoot`。映射树的节点都豁免映射
     /// （D19（块指针的结构与宽度预算） 已定项 8「映射树自己的节点」）：父条目里的位置条目是权威。
     MappingTreeNodeBelowTheRoot(CodeTwoTreeNodePosition),
@@ -2506,6 +2523,12 @@ impl TransactionUnit {
             TransactionUnit::LineageNodeBelowTheRoot(position) => {
                 format!("tl^{}.{}", position.level, position.index_in_level)
             }
+            // 退役树三种角色同样没有登记：写成「tr@叶序」「tr」「tr^层级.同层序号」。
+            TransactionUnit::RetirementLeafContainer(index) => format!("tr@{}", index.0),
+            TransactionUnit::RetirementRoot => "tr".to_string(),
+            TransactionUnit::RetirementNodeBelowTheRoot(position) => {
+                format!("tr^{}.{}", position.level, position.index_in_level)
+            }
             TransactionUnit::MappingTreeNodeBelowTheRoot(position) => {
                 format!("t7@{}.{}", position.level, position.index_in_level)
             }
@@ -2551,6 +2574,9 @@ impl TransactionUnit {
             | TransactionUnit::LineageLeafContainer(_)
             | TransactionUnit::LineageRoot
             | TransactionUnit::LineageNodeBelowTheRoot(_)
+            | TransactionUnit::RetirementLeafContainer(_)
+            | TransactionUnit::RetirementRoot
+            | TransactionUnit::RetirementNodeBelowTheRoot(_)
             | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
             | TransactionUnit::MappingTree
             | TransactionUnit::TreeTable => false,
@@ -2564,6 +2590,7 @@ impl TransactionUnit {
             TransactionUnit::InodeLeafContainer(_)
             | TransactionUnit::TombstoneLeafContainer(_)
             | TransactionUnit::LineageLeafContainer(_)
+            | TransactionUnit::RetirementLeafContainer(_)
             | TransactionUnit::InstanceTable
             | TransactionUnit::InstanceTablePageAfterTheFirst(_) => UNIT_CLASS_PACKED,
             TransactionUnit::ExtentLowerNode(_)
@@ -2579,6 +2606,8 @@ impl TransactionUnit {
             | TransactionUnit::TombstoneNodeBelowTheRoot(_)
             | TransactionUnit::LineageRoot
             | TransactionUnit::LineageNodeBelowTheRoot(_)
+            | TransactionUnit::RetirementRoot
+            | TransactionUnit::RetirementNodeBelowTheRoot(_)
             | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
             | TransactionUnit::MappingTree
             | TransactionUnit::TreeTable => UNIT_CLASS_INDEX_NODE,
@@ -2606,6 +2635,9 @@ impl TransactionUnit {
             TransactionUnit::LineageLeafContainer(_)
             | TransactionUnit::LineageRoot
             | TransactionUnit::LineageNodeBelowTheRoot(_) => trees.lineage,
+            TransactionUnit::RetirementLeafContainer(_)
+            | TransactionUnit::RetirementRoot
+            | TransactionUnit::RetirementNodeBelowTheRoot(_) => trees.retirement,
             TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
                 trees.central_mapping
             }
@@ -2625,6 +2657,7 @@ impl TransactionUnit {
             TransactionUnit::InodeLeafContainer(_)
             | TransactionUnit::TombstoneLeafContainer(_)
             | TransactionUnit::LineageLeafContainer(_)
+            | TransactionUnit::RetirementLeafContainer(_)
             | TransactionUnit::InstanceTable
             | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                 PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned)
@@ -2642,6 +2675,8 @@ impl TransactionUnit {
             | TransactionUnit::TombstoneNodeBelowTheRoot(_)
             | TransactionUnit::LineageRoot
             | TransactionUnit::LineageNodeBelowTheRoot(_)
+            | TransactionUnit::RetirementRoot
+            | TransactionUnit::RetirementNodeBelowTheRoot(_)
             | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
             | TransactionUnit::MappingTree
             | TransactionUnit::TreeTable => PlacementRule::CommitGenerated(UnitFootprint::OneSlot),
@@ -2655,13 +2690,13 @@ pub enum PlacementRule {
     CommitGenerated(UnitFootprint),
 }
 
-/// 带文件的一版那十二棵树各自的树 ID，按 D8（核心索引结构） 已定项 11 的次序：extent、inode、分配记录、记账、中央映射、
+/// 带文件的一版那十三棵树各自的树 ID，按 D8（核心索引结构） 已定项 11 的次序：extent、inode、分配记录、记账、中央映射、
 /// livelist、稀疏旁表、deadlist，再接第一个文件版本那次零根注册的 `deleted_inodes`、墓碑树、谱系树、logged_ops
-/// （D18（块里携带什么信息） 已定项 24 第 6 条列的次序）。
+/// （D18（块里携带什么信息） 已定项 24 第 6 条列的次序）与退役树（D8 已定项 9 码 13，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）。
 ///
-/// 第一个文件版本那次发布从它要建在上面的那一版的树 ID 水位起，按这个次序连号发这十二个号
+/// 第一个文件版本那次发布从它要建在上面的那一版的树 ID 水位起，按这个次序连号发这十三个号
 /// （[`FileVersionTreeIdentifiers::issued_from_watermark`]）；之后每一版照抄，再不发号。
-/// mkfs 那条流上那一版的水位是 mkfs 种下的 65536，发出来就是已定项 11 登记的 65536..65547；回退到树表 0 条的一版之后再发，
+/// mkfs 那条流上那一版的水位是 mkfs 种下的 65536，发出来就是已定项 11 登记的 65536..65548；回退到树表 0 条的一版之后再发，
 /// 那一版的水位带着回退之前根环里的 max（D8（核心索引结构） 已定项 8 ②），发出来的号高于此前发过的每一个——号永不重发
 /// （里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 ④ 行；C511（回退到无文件那一版之后诞生代怎么接））。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -2678,18 +2713,19 @@ pub struct FileVersionTreeIdentifiers {
     pub tombstone: TreeIdentifier,
     pub lineage: TreeIdentifier,
     pub logged_operations: TreeIdentifier,
+    pub retirement: TreeIdentifier,
 }
 
-/// 从水位起连号发十二个树 ID 时，水位装不下：盘上读来的 8 字节水位离 `u64::MAX` 不到十二个号。
+/// 从水位起连号发十三个树 ID 时，水位装不下：盘上读来的 8 字节水位离 `u64::MAX` 不到十三个号。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees {
     pub tree_identifier_watermark: u64,
 }
 
 impl FileVersionTreeIdentifiers {
-    /// 十二棵树在发号次序里各自离水位几个号：mkfs 水位 65536 时发出来的就是 D8（核心索引结构） 已定项 11 登记的常量
-    /// （后四棵是 D18（块里携带什么信息） 已定项 24 第 6 条那四个），偏移从那几个常量现算，不另抄一份次序。
-    const OFFSETS_FROM_THE_WATERMARK_IN_ISSUE_ORDER: [u64; 12] = [
+    /// 十三棵树在发号次序里各自离水位几个号：mkfs 水位 65536 时发出来的就是 D8（核心索引结构） 已定项 11 登记的常量
+    /// （其后四棵是 D18（块里携带什么信息） 已定项 24 第 6 条那四个，最后是退役树），偏移从那几个常量现算，不另抄一份次序。
+    const OFFSETS_FROM_THE_WATERMARK_IN_ISSUE_ORDER: [u64; 13] = [
         TREE_IDENTIFIER_EXTENT - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
         TREE_IDENTIFIER_INODE - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
         TREE_IDENTIFIER_ALLOCATION_RECORDS - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
@@ -2702,13 +2738,14 @@ impl FileVersionTreeIdentifiers {
         TREE_IDENTIFIER_TOMBSTONE - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
         TREE_IDENTIFIER_LINEAGE - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
         TREE_IDENTIFIER_LOGGED_OPERATIONS - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
+        TREE_IDENTIFIER_RETIREMENT - TREE_IDENTIFIER_WATERMARK_AT_MKFS,
     ];
 
-    /// 从 `tree_identifier_watermark`（下一个可用号）起按 D8（核心索引结构） 已定项 11 的次序连号发十二个号，
+    /// 从 `tree_identifier_watermark`（下一个可用号）起按 D8（核心索引结构） 已定项 11 的次序连号发十三个号，
     /// 连同发完之后的下一个可用号（最大那个号 + 1）。
     ///
     /// # Errors
-    /// 水位加十二个号越过 `u64::MAX`（水位是盘上读来的 8 字节）⇒ `TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees`。
+    /// 水位加十三个号越过 `u64::MAX`（水位是盘上读来的 8 字节）⇒ `TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees`。
     pub fn issued_from_watermark(
         tree_identifier_watermark: u64,
     ) -> Result<(Self, u64), TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees> {
@@ -2723,7 +2760,7 @@ impl FileVersionTreeIdentifiers {
             Self::OFFSETS_FROM_THE_WATERMARK_IN_ISSUE_ORDER.map(|offset_from_the_watermark| {
                 TreeIdentifier(tree_identifier_watermark + offset_from_the_watermark)
             });
-        let [extent, inode, allocation_records, accounting, central_mapping, livelist, sparse_side_table, deadlist, deleted_inodes, tombstone, lineage, logged_operations] =
+        let [extent, inode, allocation_records, accounting, central_mapping, livelist, sparse_side_table, deadlist, deleted_inodes, tombstone, lineage, logged_operations, retirement] =
             issued;
         let trees = FileVersionTreeIdentifiers {
             extent,
@@ -2738,18 +2775,19 @@ impl FileVersionTreeIdentifiers {
             tombstone,
             lineage,
             logged_operations,
+            retirement,
         };
         assert_eq!(
             trees.highest().0 + 1,
             next_available_after_this_issue,
-            "十二个号连号发、最大的是 logged_ops：发完之后的下一个可用号就是 mkfs 那条流上第一次发布之后的水位 65548 平移过来"
+            "十三个号连号发、最大的是退役树：发完之后的下一个可用号就是 mkfs 那条流上第一次发布之后的水位 65549 平移过来"
         );
         Ok((trees, next_available_after_this_issue))
     }
 
-    /// 十二个号按发号次序。
+    /// 十三个号按发号次序。
     #[must_use]
-    pub const fn in_issue_order(&self) -> [TreeIdentifier; 12] {
+    pub const fn in_issue_order(&self) -> [TreeIdentifier; 13] {
         [
             self.extent,
             self.inode,
@@ -2763,16 +2801,17 @@ impl FileVersionTreeIdentifiers {
             self.tombstone,
             self.lineage,
             self.logged_operations,
+            self.retirement,
         ]
     }
 
-    /// 这十二个号里最大的那一个。
+    /// 这十三个号里最大的那一个。
     #[must_use]
     pub fn highest(&self) -> TreeIdentifier {
         self.in_issue_order()
             .into_iter()
             .max()
-            .expect("十二个号，数组非空")
+            .expect("十三个号，数组非空")
     }
 }
 
@@ -2844,6 +2883,11 @@ pub struct TransactionOutput {
     pub lineage_containers: Vec<LineageContainerVersion>,
     /// 这一版谱系树的形状与根之下每个码 2 节点的指针，同墓碑树那一份。
     pub lineage_tree_index_nodes: ContainerTreeIndexNodesVersion,
+    /// 这一版退役树的叶容器，按叶序（= 容器号升序）：每片的内容与这一版它的指针（判决 `research/prompts/m3-retire-r1-main-verification.md`
+    /// 第三节第 5 条）。空 ⇒ 退役树零根。随现行那一版、只增不删：身份消失的那次发布追加，管理员回退不取 R_old 的。
+    pub retirement_containers: Vec<RetirementContainerVersion>,
+    /// 这一版退役树的形状与根之下每个码 2 节点的指针，同墓碑树那一份。
+    pub retirement_tree_index_nodes: ContainerTreeIndexNodesVersion,
     /// 这一版 inode 树里的全部叶容器，左起按 key 序：每片的身份、它装的记录、这一版它的指针
     /// （这次重写的是新落点，没重写的照抄上一版）。下一次发布按它算记录落在哪一片、要不要分裂。
     pub inode_leaf_containers: Vec<InodeLeafContainerVersion>,
@@ -2921,6 +2965,13 @@ pub struct LineageContainerVersion {
     pub pointer: NodePointer,
 }
 
+/// 一版里的一片退役容器：装了什么、这一版它的指针。字节放 `TransactionOutput::units` 里那个角色的单元。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct RetirementContainerVersion {
+    pub contents: RetirementContainer,
+    pub pointer: NodePointer,
+}
+
 /// 一版里一棵容器树（墓碑树、谱系树）的码 2 节点：形状（每层每个节点几个孩子，`crate::container_tree::ContainerTreeShape`）与
 /// 根之下每个节点的位置与指针（按 bump 次序：层级升序、同层从左到右）。根的指针住树表条目，不在这里；零根与只有根时这里一个都没有。
 /// 节点的字节放 `TransactionOutput::units` 里那个角色的单元。
@@ -2961,6 +3012,12 @@ impl TransactionOutput {
         self.lineage_tree_index_nodes.shape.internal_levels()
     }
 
+    /// 这一版退役树的内部层数：根节点码 2 头里的层级减一（零根与只有根时 0）。形状的来路同谱系树那一份。
+    #[must_use]
+    pub fn retirement_tree_internal_levels(&self) -> u64 {
+        self.retirement_tree_index_nodes.shape.internal_levels()
+    }
+
     /// 这一版是不是「有树、文件集合为空」（D8（核心索引结构） 已定项 16 第 5 条）：树表有条目（`TransactionOutput` 恒有，树表 0 条的一版
     /// 是 `VersionWithoutFilePublishOutput`）∧ inode 树零条目根（一片叶容器都没有，key 区间 `[0, 0]`）∧ extent 树零条目根
     /// （上段一条叶条目都没有：这一版一个数据单元都没有）∧ `deleted_inodes` 根指针置零。不看 logged_ops、墓碑树、谱系树。
@@ -2987,6 +3044,8 @@ impl TransactionOutput {
 
     /// 记账里这一版的 inode 号水位（下一个可用号，D5（快照 / 空间记账机制） 已定项 4 第 12 项）。
     /// 读的是这一版真写进记账树的那一行，不重算——下一次发布接着它发号，I-9.6（水位大于两处最大号） 在盘上判它与树里最大 key 的关系。
+    /// 这一统计量的行里还有台阶行（[`TransactionOutput::inode_number_watermark_steps`]），现行值取代段最大的那一行
+    /// （D8（核心索引结构） 已定项 6「读现行值 = 取该维度元组下代号最大的那一行」）：每次发布写的那一行代段是这次的 txg，台阶行的代段都比它小。
     ///
     /// # Panics
     /// 这一版的记账行里没有 inode 号水位那一行。两条来路各自保证它在：这个进程发布出来的版本由 `publish_version` 写它
@@ -2996,7 +3055,8 @@ impl TransactionOutput {
     pub fn inode_number_watermark(&self) -> u64 {
         self.accounting_entries
             .iter()
-            .find(|entry| entry.statistic == STATISTIC_INODE_WATERMARK)
+            .filter(|entry| entry.statistic == STATISTIC_INODE_WATERMARK)
+            .max_by_key(|entry| entry.generation)
             .expect(
                 "inode 号水位那一行在：发布路径每次都写它（D5 已定项 8 的池级三行之一），\
                  从盘上重建的版本在 rebuild_version 的边界判上验过（盘上的标签可以是任何值）",
@@ -3004,6 +3064,40 @@ impl TransactionOutput {
             .value
     }
 
+    /// 这一版记账里 inode 号水位那一统计量的台阶行 (g, H)（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）：
+    /// 水位变了的每次发布留一行，g 是那次发布的 txg（行的代段），H 是那次发布之后的水位（行的值）；台阶行与每次发布写的那一行同统计量、
+    /// 同树段、同设备段（不带设备维的保留值），只差在代段——这一版那次发布本身水位变了时，它写的那一行就是这一次的台阶行。
+    /// 认法（按行算、不另存标记）：代段小于这一版 txg 的行都是台阶行；代段等于这一版 txg 的那一行，值与代段最大的台阶行不同（或没有更早的台阶行）
+    /// 才是台阶行。下一次发布照抄这几行（随根 COW、一直留着），管理员回退那次照抄 cur 的（`mount::roll_back_by_a_forward_publish`）。
+    /// 按代段升序。
+    // TODO(D5 已定项 2 射程「克隆头的台阶行按 (统计量, 树) 分，建克隆那次给新头留一行」)：`crates/` 今天没有克隆路径
+    // （`mount::roll_back_by_a_forward_publish` 遇到树 ID 集合不同就拒），这里不按树段分；做克隆时按树段分开取、建克隆那次给新头留一行。
+    #[must_use]
+    pub fn inode_number_watermark_steps(&self) -> Vec<AccountingEntry> {
+        let version_txg = self.root.checkpoint_txg;
+        let mut steps: Vec<AccountingEntry> = self
+            .accounting_entries
+            .iter()
+            .filter(|entry| {
+                entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < version_txg
+            })
+            .copied()
+            .collect();
+        steps.sort_by_key(|entry| entry.generation);
+        let row_of_this_version = self.accounting_entries.iter().find(|entry| {
+            entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation == version_txg
+        });
+        if let Some(row) = row_of_this_version {
+            if steps
+                .last()
+                .is_none_or(|latest_step| latest_step.value != row.value)
+            {
+                steps.push(*row);
+            }
+        }
+        steps
+    }
+
     #[must_use]
     pub fn unit(&self, identity: TransactionUnit) -> &PublishedUnit {
         self.units
@@ -3192,7 +3286,10 @@ pub fn placements_to_release_via_mapping(
             | TransactionUnit::TombstoneNodeBelowTheRoot(_)
             | TransactionUnit::LineageLeafContainer(_)
             | TransactionUnit::LineageRoot
-            | TransactionUnit::LineageNodeBelowTheRoot(_) => {
+            | TransactionUnit::LineageNodeBelowTheRoot(_)
+            | TransactionUnit::RetirementLeafContainer(_)
+            | TransactionUnit::RetirementRoot
+            | TransactionUnit::RetirementNodeBelowTheRoot(_) => {
                 if !has_a_placement_in_the_previous_version(previous, identity) {
                     continue;
                 }
@@ -3353,6 +3450,21 @@ fn has_a_placement_in_the_previous_version(
                     .shape
                     .is_the_root(position)
         }
+        // 退役树：零根的一版没有根与容器（第一次有身份消失的那次新开的那一片在上一版里没有落点）。
+        TransactionUnit::RetirementLeafContainer(index) => {
+            index.position() < previous.retirement_containers.len()
+        }
+        TransactionUnit::RetirementRoot => !previous.retirement_containers.is_empty(),
+        TransactionUnit::RetirementNodeBelowTheRoot(position) => {
+            previous
+                .retirement_tree_index_nodes
+                .shape
+                .has_a_node_at(position)
+                && !previous
+                    .retirement_tree_index_nodes
+                    .shape
+                    .is_the_root(position)
+        }
         // 多层码 2 树的节点按上一版的位置点名（`previous_roles_replaced_by_this_publish` 从计划里的「被换下的节点」取），恒在上一版里。
         TransactionUnit::ExtentRoot
         | TransactionUnit::InodeRoot
@@ -3430,7 +3542,10 @@ fn mapped_roles_released_by_this_publish(
             | TransactionUnit::TombstoneNodeBelowTheRoot(_)
             | TransactionUnit::LineageLeafContainer(_)
             | TransactionUnit::LineageRoot
-            | TransactionUnit::LineageNodeBelowTheRoot(_) => true,
+            | TransactionUnit::LineageNodeBelowTheRoot(_)
+            | TransactionUnit::RetirementLeafContainer(_)
+            | TransactionUnit::RetirementRoot
+            | TransactionUnit::RetirementNodeBelowTheRoot(_) => true,
             TransactionUnit::MappingTreeNodeBelowTheRoot(_)
             | TransactionUnit::MappingTree
             | TransactionUnit::TreeTable
@@ -4120,9 +4235,12 @@ impl DeletionStepOfThePublish {
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum LineageStepOfThePublish {
     NoLineageRecord,
-    /// 管理员回退那次发布：谱系树开放容器追加 `record`（T_rb = 这次发布的 txg，T_old = R_old 的 txg）。
+    /// 管理员回退那次发布：谱系树开放容器追加 `record`（T_rb = 这次发布的 txg，T_old = R_old 的 txg，H_cur = cur 的 inode 号水位）；
+    /// 这次发布的 inode 号水位写 `inode_number_watermark_after_the_rollback`（D23（journal 的角色与格式） 已定项 14「水位」一格的 max，
+    /// 回退那一判算好交进来），不从上一版的记账现算。
     AdministratorRollback {
         record: LineageRecord,
+        inode_number_watermark_after_the_rollback: u64,
     },
 }
 
@@ -4171,6 +4289,10 @@ fn lineage_container_role(position: usize) -> TransactionUnit {
     TransactionUnit::LineageLeafContainer(LineageContainerIndexInTree::of_position(position))
 }
 
+fn retirement_container_role(position: usize) -> TransactionUnit {
+    TransactionUnit::RetirementLeafContainer(RetirementContainerIndexInTree::of_position(position))
+}
+
 impl ContainerTreeRoles {
     const OF_THE_TOMBSTONE_TREE: ContainerTreeRoles = ContainerTreeRoles {
         container: tombstone_container_role,
@@ -4182,6 +4304,11 @@ impl ContainerTreeRoles {
         root: TransactionUnit::LineageRoot,
         node_below_the_root: TransactionUnit::LineageNodeBelowTheRoot,
     };
+    const OF_THE_RETIREMENT_TREE: ContainerTreeRoles = ContainerTreeRoles {
+        container: retirement_container_role,
+        root: TransactionUnit::RetirementRoot,
+        node_below_the_root: TransactionUnit::RetirementNodeBelowTheRoot,
+    };
 
     /// 形状为 `shape` 的那一版里，`position` 上那个码 2 节点的角色：是那一版的根就是根，别的按位置。
     fn of_node(
@@ -4259,28 +4386,58 @@ fn mapped_units_of_a_container_tree<Record>(
     mapped_units
 }
 
-/// 记账标签 6（已承诺预留里墓碑那一份，D3（空间分配） 已定项 2）在这次发布之后那一版的值：
-/// 副本数 × ((2 + H + 1) × 16384 + 41 × n_live)，H 是这次之后墓碑树的内部层数，n_live 是这次之后 inode 树里 nlink ≥ 1 的记录数
-/// （[`crate::admission::live_objects_of_the_inode_tree`]）。`TransactionOutput` 恒有树表条目（`deleted_inodes` 那一条在内），
-/// 所以走不到「没有 `deleted_inodes` 条目取 0」那一支——那一支是树表 0 条的一版，它不走这条发布路径、没有记账树。
-/// 记账行写它，发布路径判式子读它（「写入类发布按发布之后的标签判」）：一个函数算，两处读同一个数。
-/// H 照条款字面从墓碑树根节点的码 2 头现读（[`tombstone_tree_internal_levels_read_from_the_root_node_header`]，代码轮 U8）。
+/// 记账标签 6（已承诺预留里墓碑与退役记录那两份，D3（空间分配） 已定项 2）在这次发布之后那一版的值：
+/// 副本数 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)：H_墓碑、H_退役是这次之后墓碑树、退役树的内部层数，
+/// n_live 是这次之后 inode 树里 nlink ≥ 1 的记录数（[`crate::admission::live_objects_of_the_inode_tree`]），n_记录 是这次之后 inode 树里全部记录数
+/// （含 nlink = 0 的，[`crate::admission::inode_records_of_the_inode_tree`]；判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条）。
+/// `TransactionOutput` 恒有树表条目（`deleted_inodes` 那一条在内），所以走不到「没有 `deleted_inodes` 条目取 0」那一支——那一支是树表 0 条的一版，
+/// 它不走这条发布路径、没有记账树。记账行写它，发布路径判式子读它（「写入类发布按发布之后的标签判」）：一个函数算，两处读同一个数。
+/// 两个 H 照条款字面从各自根节点的码 2 头现读（[`tombstone_tree_internal_levels_read_from_the_root_node_header`]、
+/// [`retirement_tree_internal_levels_read_from_the_root_node_header`]，代码轮 U8）。
 #[must_use]
-pub fn committed_reservation_for_tombstones_after_this_publish(
+pub fn committed_reservation_for_tombstones_and_retirement_records_after_this_publish(
     resolved: &ResolvedPublish,
     previous: Option<&TransactionOutput>,
     replicas: ReplicaCount,
 ) -> BytesSummedOverAllReplicas {
-    committed_reservation_for_tombstones(
-        TombstonePrepayBasis::VersionWithADeletedInodesTreeTableEntry {
+    committed_reservation_for_tombstones_and_retirement_records(
+        TombstoneAndRetirementPrepayBasis::VersionWithADeletedInodesTreeTableEntry {
             tombstone_tree_internal_levels:
                 tombstone_tree_internal_levels_read_from_the_root_node_header(resolved, previous),
             live_objects: live_objects_of_the_inode_tree(&resolved.inode_tree.containers),
+            retirement_tree_internal_levels:
+                retirement_tree_internal_levels_read_from_the_root_node_header(resolved, previous),
+            inode_records: inode_records_of_the_inode_tree(&resolved.inode_tree.containers),
         },
         replicas,
     )
 }
 
+/// 这次发布之后退役树的内部层数（同 [`tombstone_tree_internal_levels_read_from_the_root_node_header`] 的读法）：这次重写根（有身份消失）时
+/// 新根的头由同一份形状编出，按构造就是 `resolved.retirement_tree.internal_levels()`；这次照抄退役树时读上一版那份 `RetirementRoot` 单元字节的码 2 头。
+///
+/// # Panics
+/// 上一版有退役容器而它的单元里没有 `RetirementRoot`，或那份字节解不开：单元是这个进程装出来的，或从盘上重建时读出、核过的。
+#[must_use]
+pub fn retirement_tree_internal_levels_read_from_the_root_node_header(
+    resolved: &ResolvedPublish,
+    previous: Option<&TransactionOutput>,
+) -> u64 {
+    if resolved.retirement_tree.is_the_root_rewritten() {
+        return resolved.retirement_tree.internal_levels();
+    }
+    match previous {
+        Some(previous_version) if !previous_version.retirement_containers.is_empty() => {
+            let root_header =
+                parse_index_node(&previous_version.unit(TransactionUnit::RetirementRoot).bytes)
+                    .expect("退役树根单元是这个进程装的，或重建时读出、核过的");
+            u64::from(root_header.level).saturating_sub(1)
+        }
+        // 上一版退役树零根，这次又没重写它：这次之后仍是零根。
+        Some(_) | None => 0,
+    }
+}
+
 /// 这次发布之后墓碑树的内部层数 H（D3（空间分配） 已定项 2「H 从墓碑树根节点码 2 头现读」，代码轮 U8，
 /// `research/prompts/m3-unlink-code-r1-main-verification.md` 第三节）：根节点头的层级减一（零根时 0）。
 /// 这次重写根（意图那次往开放容器追加墓碑）：新根的头由同一份形状编出（`build_index_node` 的层级取根在形状里的层级），按构造就是
@@ -4419,8 +4576,10 @@ pub struct ResolvedPublish {
     pub tombstone_tree: TombstoneTreeAfterThisPublish,
     /// 这次之后谱系树的样子（管理员回退那次追加一条，别的照抄）。
     pub lineage_tree: LineageTreeAfterThisPublish,
-    /// 上一版里有、这一版没有同名后继的角色（删除批截掉的尾部数据单元、删空的 inode 叶容器、删空的 `deleted_inodes` 根），
-    /// 照样被这次换下。
+    /// 这次之后退役树的样子（有 inode 叶容器身份消失的那次追加，别的照抄）。
+    pub retirement_tree: RetirementTreeAfterThisPublish,
+    /// 上一版里有、这一版没有同一个单元接着的角色（删除批截掉的尾部数据单元、身份消失的 inode 叶容器——按它在上一版里的叶序、
+    /// 删空的 `deleted_inodes` 根），照样被这次换下。
     pub previous_roles_without_a_successor: Vec<TransactionUnit>,
 }
 
@@ -4492,12 +4651,19 @@ impl ResolvedPublish {
                 .filter(|identity| matches!(identity, TransactionUnit::Data(_))),
         );
         replaced.extend(self.extent_tree.replaced_previous_roles.iter().copied());
-        // 容器树长高那次：新根在上一版里没有落点，上一版的根不重写、留在原位置当根之下的节点——它不被这次换下。
+        // 容器树长高那次：新根在上一版里没有落点，上一版的根不重写、留在原位置当根之下的节点——它不被这次换下
+        // （一次追加几条、旧根下面也有容器重写时旧根跟着重写，照样换下：`previous_root_is_carried_down_unchanged` 只认前一种）。
         let is_a_new_root_over_a_root_carried_down = |identity: TransactionUnit| {
             (identity == TransactionUnit::TombstoneRoot
-                && self.tombstone_tree.grew_a_level_this_publish())
+                && self
+                    .tombstone_tree
+                    .previous_root_is_carried_down_unchanged())
                 || (identity == TransactionUnit::LineageRoot
-                    && self.lineage_tree.grew_a_level_this_publish())
+                    && self.lineage_tree.previous_root_is_carried_down_unchanged())
+                || (identity == TransactionUnit::RetirementRoot
+                    && self
+                        .retirement_tree
+                        .previous_root_is_carried_down_unchanged())
         };
         replaced.extend(self.rewritten_roles.iter().copied().filter(|identity| {
             !is_a_node_of_a_tree_with_its_own_replacement_list(*identity)
@@ -4566,6 +4732,9 @@ pub const fn multi_level_tree_of_role(identity: TransactionUnit) -> Option<Multi
         | TransactionUnit::LineageLeafContainer(_)
         | TransactionUnit::LineageRoot
         | TransactionUnit::LineageNodeBelowTheRoot(_)
+        | TransactionUnit::RetirementLeafContainer(_)
+        | TransactionUnit::RetirementRoot
+        | TransactionUnit::RetirementNodeBelowTheRoot(_)
         | TransactionUnit::TreeTable
         | TransactionUnit::InstanceTable
         | TransactionUnit::InstanceTablePageAfterTheFirst(_) => None,
@@ -4610,6 +4779,9 @@ pub const fn allocation_record_tree_node_of_role(
         | TransactionUnit::LineageLeafContainer(_)
         | TransactionUnit::LineageRoot
         | TransactionUnit::LineageNodeBelowTheRoot(_)
+        | TransactionUnit::RetirementLeafContainer(_)
+        | TransactionUnit::RetirementRoot
+        | TransactionUnit::RetirementNodeBelowTheRoot(_)
         | TransactionUnit::MappingTreeNodeBelowTheRoot(_)
         | TransactionUnit::MappingTree
         | TransactionUnit::TreeTable
@@ -4874,15 +5046,24 @@ impl PerDeviceAccountingRow {
     }
 }
 
-/// 一次发布写的记账行（池级三行加每块盘六行，代 = 这次的 txg、seq 一律 1，D8（核心索引结构） 已定项 10），按 key 升序。
-/// 准入之前算记账树形状（只要 key，值给 0）与装记账树（值从分配器取）读的是这一份行表。
+/// 一次发布写的记账行（池级三行加每块盘六行，代 = 这次的 txg、seq 一律 1，D8（核心索引结构） 已定项 10），
+/// 加上从上一版照抄的 inode 号水位台阶行（[`TransactionOutput::inode_number_watermark_steps`]，代段都小于这次的 txg，原样照抄），按 key 升序。
+/// 准入之前算记账树形状（只要 key，值给 0）与装记账树（值从分配器取）读的是这一份行表：台阶行只看上一版，不看这次的水位，
+/// 所以两处的 key 集合相同。这次的水位与照抄来的最后一级台阶不同时，这次写的那一行就是这次的台阶行（下一次发布照抄它）。
 fn accounting_entries_of_this_publish(
     txg: CheckpointTxg,
     inode_tree: TreeIdentifier,
     device_identities: &[DeviceIdentity],
+    inode_number_watermark_steps_carried_from_the_previous_version: &[AccountingEntry],
     pool_wide_value: &dyn Fn(PoolWideAccountingRow) -> u64,
     per_device_value: &dyn Fn(PerDeviceAccountingRow, DeviceIdentity) -> u64,
 ) -> Vec<AccountingEntry> {
+    assert!(
+        inode_number_watermark_steps_carried_from_the_previous_version
+            .iter()
+            .all(|step| step.statistic == STATISTIC_INODE_WATERMARK && step.generation < txg),
+        "照抄来的台阶行是上一版 inode 号水位那一统计量的行，代段都早于这次的 txg {txg:?}（上一版的 txg 小于这次的）"
+    );
     let mut entries: Vec<AccountingEntry> = PoolWideAccountingRow::ALL
         .into_iter()
         .map(|row| AccountingEntry {
@@ -4906,6 +5087,7 @@ fn accounting_entries_of_this_publish(
             });
         }
     }
+    entries.extend_from_slice(inode_number_watermark_steps_carried_from_the_previous_version);
     entries.sort_by_key(AccountingEntry::sort_key);
     entries
 }
@@ -4958,6 +5140,7 @@ struct DeletionTreesOfThisPublish<'plans> {
     deleted_inodes_root: DeletedInodesRootOrigin,
     tombstone_tree: &'plans TombstoneTreeAfterThisPublish,
     lineage_tree: &'plans LineageTreeAfterThisPublish,
+    retirement_tree: &'plans RetirementTreeAfterThisPublish,
 }
 
 /// 这张计划要写的记账树与中央映射树，在一切落盘动作之前算完：记账树按这一版的记账 key，中央映射树按这一版每个进映射单元的 key。
@@ -4980,6 +5163,8 @@ struct OrdinaryAllocationPartOfThePublish {
     deleted_inodes: Vec<InodeNumber>,
     deleted_inodes_root: DeletedInodesRootOrigin,
     tombstone_tree: TombstoneTreeAfterThisPublish,
+    /// 这次之后退役树的样子：有 inode 叶容器身份消失（删空摘掉、被左邻吸收）的那次往开放容器追加，别的照抄。
+    retirement_tree: RetirementTreeAfterThisPublish,
 }
 
 impl OrdinaryAllocationPartOfThePublish {
@@ -5002,9 +5187,19 @@ impl OrdinaryAllocationPartOfThePublish {
             roles.push(TransactionUnit::InodeRoot);
         }
         roles.extend(self.deletion_roles_rewritten());
+        roles.extend(self.retirement_roles_rewritten());
         roles
     }
 
+    /// 退役树这次重写的角色（树内先叶后根）：普通分配（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条，同墓碑），
+    /// 按树 ID 排在谱系树之后、中央映射树之前（D3（空间分配） 已定项 10 ⑤）。
+    fn retirement_roles_rewritten(&self) -> Vec<TransactionUnit> {
+        container_tree_roles_rewritten(
+            &self.retirement_tree,
+            ContainerTreeRoles::OF_THE_RETIREMENT_TREE,
+        )
+    }
+
     /// 删除那两棵树这次重写的角色，按 bump 次序（树 ID 升序：`deleted_inodes` 65544、墓碑树 65545；树内先叶后根）：
     /// 排在记账树之后、中央映射树之前（D3（空间分配） 已定项 10 ⑤）。
     fn deletion_roles_rewritten(&self) -> Vec<TransactionUnit> {
@@ -5019,8 +5214,9 @@ impl OrdinaryAllocationPartOfThePublish {
         roles
     }
 
-    /// 上一版里有、这一版没有后继的角色（同名角色这次不在这一版里）：删除批截掉的尾部数据单元、删空不再落盘的 inode 叶容器
-    /// （删空一片之后最右那个叶序没有了）、队列删空之后的 `deleted_inodes` 根。它们照样被这次换下、经映射释放。
+    /// 上一版里有、这一版没有同一个单元接着的角色：删除批截掉的尾部数据单元、身份消失的 inode 叶容器（删空摘掉、被左邻吸收，
+    /// 按它们在上一版里的叶序点名：它们右边那几片照抄、换到左边的叶序上，不被换下）、队列删空之后的 `deleted_inodes` 根。
+    /// 它们照样被这次换下、经映射释放。
     fn previous_roles_without_a_successor(
         &self,
         previous: &TransactionOutput,
@@ -5039,13 +5235,18 @@ impl OrdinaryAllocationPartOfThePublish {
                 .collect()
         };
         roles.extend(
-            (self.inode_tree.containers.len()..previous.inode_leaf_containers.len()).map(
-                |position| {
-                    TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(
-                        position,
-                    ))
-                },
-            ),
+            self.inode_tree
+                .retired
+                .iter()
+                .map(|retired| TransactionUnit::InodeLeafContainer(retired.previous_index)),
+        );
+        debug_assert!(
+            self.inode_tree
+                .retired
+                .iter()
+                .all(|retired| retired.previous_index.position()
+                    < previous.inode_leaf_containers.len()),
+            "身份消失的叶容器在上一版里"
         );
         if self.deleted_inodes_root == DeletedInodesRootOrigin::NoRoot
             && !previous.deleted_inodes.is_empty()
@@ -5211,6 +5412,33 @@ impl PublishPlan<'_> {
                 tombstone_shape_before,
             ),
         };
+        // 退役树（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 3–5 条）：这次身份消失的每一片 inode 叶容器
+        // （删空摘掉、被左邻吸收）追加一条，死亡 txg = 这次发布的 checkpoint_txg，按它们在上一版里的叶序；写者只追加，不去重、不覆盖。
+        // 借不写（右片保留身份，不在 `retired` 里）。没有身份消失的发布整棵照抄。
+        let retirement_containers_before: Vec<RetirementContainer> = previous
+            .map(|version| {
+                version
+                    .retirement_containers
+                    .iter()
+                    .map(|container| container.contents.clone())
+                    .collect()
+            })
+            .unwrap_or_default();
+        let retirement_shape_before = previous
+            .map_or_else(ContainerTreeShape::zero_root, |version| {
+                version.retirement_tree_index_nodes.shape.clone()
+            });
+        let retirement_tree = append_retirement_records_to_the_open_container(
+            &retirement_containers_before,
+            &retirement_shape_before,
+            &retirement_records_of_this_publish(&inode_tree.retired_identities(), self.txg),
+            self.txg,
+            trees.retirement,
+            deletion_and_lineage
+                .container_tree_capacities
+                .retirement_container,
+            deletion_and_lineage.container_tree_capacities.node_entries,
+        );
         Ok(OrdinaryAllocationPartOfThePublish {
             inode_tree,
             extent_tree,
@@ -5219,6 +5447,7 @@ impl PublishPlan<'_> {
             deleted_inodes,
             deleted_inodes_root,
             tombstone_tree,
+            retirement_tree,
         })
     }
 
@@ -5243,6 +5472,7 @@ impl PublishPlan<'_> {
     ) -> Result<ResolvedPublish, PublishError> {
         let ordinary_part = self.ordinary_allocation_part(previous, trees, deletion_and_lineage)?;
         let deletion_roles_rewritten = ordinary_part.deletion_roles_rewritten();
+        let retirement_roles_rewritten = ordinary_part.retirement_roles_rewritten();
         let lineage_tree =
             self.lineage_tree_after_this_publish(previous, trees, deletion_and_lineage);
         let previous_roles_without_a_successor = previous
@@ -5259,6 +5489,7 @@ impl PublishPlan<'_> {
             deleted_inodes,
             deleted_inodes_root,
             tombstone_tree,
+            retirement_tree,
         } = ordinary_part;
         let mut rewritten_roles: Vec<TransactionUnit> = file_content_transactions
             .iter()
@@ -5299,6 +5530,7 @@ impl PublishPlan<'_> {
                 deleted_inodes_root,
                 tombstone_tree: &tombstone_tree,
                 lineage_tree: &lineage_tree,
+                retirement_tree: &retirement_tree,
             },
             device_identities,
             capacities,
@@ -5319,6 +5551,8 @@ impl PublishPlan<'_> {
             &lineage_tree,
             ContainerTreeRoles::OF_THE_LINEAGE_TREE,
         ));
+        // 退役树（树 ID 接在 logged_ops 之后，判决 m3-retire-r1 第三节第 1 条）：普通分配，按树 ID 排在谱系树之后。
+        rewritten_roles.extend(retirement_roles_rewritten);
         rewritten_roles.extend(central_mapping_tree.rewritten_positions().into_iter().map(
             |position| {
                 MultiLevelCodeTwoTree::CentralMapping
@@ -5339,6 +5573,7 @@ impl PublishPlan<'_> {
             deleted_inodes_root,
             tombstone_tree,
             lineage_tree,
+            retirement_tree,
             previous_roles_without_a_successor,
         })
     }
@@ -5363,7 +5598,7 @@ impl PublishPlan<'_> {
             version.lineage_tree_index_nodes.shape.clone()
         });
         match deletion_and_lineage.lineage {
-            LineageStepOfThePublish::AdministratorRollback { record } => {
+            LineageStepOfThePublish::AdministratorRollback { record, .. } => {
                 append_lineage_record_to_the_open_container(
                     &containers_before,
                     &shape_before,
@@ -5410,11 +5645,14 @@ impl PublishPlan<'_> {
                 refusal,
             }
         };
-        // 记账树：每次发布整批重写（代 = 这次的 txg），key 只看统计量、树、盘与代，不看值。
+        // 记账树：每次发布整批重写（代 = 这次的 txg），key 只看统计量、树、盘与代，不看值；inode 号水位的台阶行照抄上一版的。
+        let inode_number_watermark_steps_carried =
+            previous.map_or_else(Vec::new, TransactionOutput::inode_number_watermark_steps);
         let accounting_keys: BTreeSet<CodeTwoTreeKey> = accounting_entries_of_this_publish(
             txg,
             trees.inode,
             device_identities,
+            &inode_number_watermark_steps_carried,
             &|_| 0,
             &|_, _| 0,
         )
@@ -5523,17 +5761,25 @@ impl PublishPlan<'_> {
             }
         }
         let mut inode_tree_birth_sequence = 0;
-        for position in 0..inode_tree.containers.len() {
-            let index = InodeLeafContainerIndexInTree::of_position(position);
-            let identity = TransactionUnit::InodeLeafContainer(index);
-            if inode_tree.rewritten.contains(&index) {
-                mapped_units.push((
-                    identity,
-                    node_key(UNIT_CLASS_PACKED, trees.inode, inode_tree_birth_sequence),
-                ));
-                inode_tree_birth_sequence += 1;
-            } else {
-                mapped_units.push((identity, carried(identity)));
+        for (position, origin) in inode_tree.origins.iter().enumerate() {
+            let identity = TransactionUnit::InodeLeafContainer(
+                InodeLeafContainerIndexInTree::of_position(position),
+            );
+            match origin {
+                InodeLeafContainerOrigin::RewrittenThisPublish => {
+                    mapped_units.push((
+                        identity,
+                        node_key(UNIT_CLASS_PACKED, trees.inode, inode_tree_birth_sequence),
+                    ));
+                    inode_tree_birth_sequence += 1;
+                }
+                // 照抄的一片取它在上一版里那个叶序的 key（左边有片身份消失时它在上一版里更靠右）。
+                InodeLeafContainerOrigin::CarriedFrom(previous_index) => {
+                    mapped_units.push((
+                        identity,
+                        carried(TransactionUnit::InodeLeafContainer(*previous_index)),
+                    ));
+                }
             }
         }
         mapped_units.push(if !inode_tree.is_the_root_rewritten() {
@@ -5626,6 +5872,12 @@ impl PublishPlan<'_> {
             &|unit_class, birth_sequence| node_key(unit_class, trees.lineage, birth_sequence),
             &carried,
         ));
+        mapped_units.extend(mapped_units_of_a_container_tree(
+            deletion_trees.retirement_tree,
+            ContainerTreeRoles::OF_THE_RETIREMENT_TREE,
+            &|unit_class, birth_sequence| node_key(unit_class, trees.retirement, birth_sequence),
+            &carried,
+        ));
         let mapping_keys: BTreeSet<CodeTwoTreeKey> = mapped_units
             .iter()
             .map(|(_, key)| CodeTwoTreeKey::new(key, CodeTwoKeyFieldWidths::CENTRAL_MAPPING))
@@ -6258,7 +6510,7 @@ fn publish_version_of_trees_holding_one_data_unit<Device: BlockDevice>(
 /// `AllocationRecordTreeRewriteSetDidNotSettle`、释放判定路径的四种错、`PlacementRefused`、块设备错。
 ///
 /// # Panics
-/// 没有上一版、而计划写的新水位不是 mkfs 那条流第一次发布之后的 65548：调用方把一个水位早已推高的池当成了 mkfs 那条流，
+/// 没有上一版、而计划写的新水位不是 mkfs 那条流第一次发布之后的 65549：调用方把一个水位早已推高的池当成了 mkfs 那条流，
 /// 按 65536 发号会重发已经发过的号（D8（核心索引结构） 已定项 8 ②）。
 pub fn publish_version<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
@@ -6295,7 +6547,7 @@ pub fn publish_version_with_unmount_marker<Device: BlockDevice>(
         None => {
             assert_eq!(
                 plan.tree_identifier_watermark, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
-                "没有上一版的发布在这里按 mkfs 那条流的水位 65536 发号：计划写的新水位必须就是那条流的 65548，别的水位走 publish_first_file"
+                "没有上一版的发布在这里按 mkfs 那条流的水位 65536 发号：计划写的新水位必须就是那条流的 65549，别的水位走 publish_first_file"
             );
             FileVersionTreeIdentifiers::issued_from_watermark(TREE_IDENTIFIER_WATERMARK_AT_MKFS)
                 .expect("mkfs 水位 65536 加十二个号装得下")
@@ -6316,15 +6568,20 @@ pub fn publish_version_with_unmount_marker<Device: BlockDevice>(
 /// 管理员回退那次发布（`mount::roll_back_by_a_forward_publish`）：同 [`publish_version`]，另往谱系树开放容器追加 `lineage_record`
 /// （D18（块里携带什么信息） 已定项 24 第 5 条、D23（journal 的角色与格式） 已定项 14「回退那次发布追加一条谱系记录」）。
 /// 谱系容器与谱系树根归固定点，这次发布照旧没有普通分配、不判式子（D28（挂载期承诺量） 已定项 1 接线）。
+/// 这次的 inode 号水位写 `inode_number_watermark_after_the_rollback`（回退那一判算好的 max），记账里的台阶行照抄 `previous`（它的记账行原样是 cur 的）。
 ///
 /// # Errors
 /// 同 [`publish_version`]（谱系树根满时长高，`crate::container_tree`）。
+///
+/// # Panics
+/// `plan` 里有新 inode 记录：回退那次发布不建 inode。
 pub fn publish_version_appending_a_lineage_record<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
     plan: PublishPlan<'_>,
     previous: &TransactionOutput,
     lineage_record: LineageRecord,
+    inode_number_watermark_after_the_rollback: u64,
 ) -> Result<TransactionOutput, PublishError> {
     publish_version_of_trees(
         pool,
@@ -6338,6 +6595,7 @@ pub fn publish_version_appending_a_lineage_record<Device: BlockDevice>(
             container_tree_capacities: pool.container_tree_capacities(),
             lineage: LineageStepOfThePublish::AdministratorRollback {
                 record: lineage_record,
+                inode_number_watermark_after_the_rollback,
             },
         },
     )
@@ -6506,7 +6764,7 @@ pub fn prepare_the_version_publish<Device: BlockDevice>(
             &admission_reading_before_a_publish_on_a_ring_of(
                 allocator,
                 previous,
-                committed_reservation_for_tombstones_after_this_publish(
+                committed_reservation_for_tombstones_and_retirement_records_after_this_publish(
                     &settled.resolved,
                     previous,
                     ReplicaCount::of_every_device_in_the_pool(allocator),
@@ -6548,6 +6806,7 @@ pub fn prepare_the_version_publish<Device: BlockDevice>(
         &quarantine,
         trees,
         unmount_marker,
+        deletion_and_lineage.lineage,
     )
 }
 
@@ -7270,18 +7529,19 @@ struct InodeTreeRootUnit {
     birth_sequence: BirthSequence,
 }
 
-/// 这一版 inode 树里的一片叶容器：装了什么、这一版的指针、这一版的字节，以及它这次是不是重写的。
+/// 这一版 inode 树里的一片叶容器：装了什么、这一版的指针、这一版的字节，以及它从哪来（这次重写，或照抄上一版哪个叶序上的那一片）。
 struct InodeLeafContainerUnit {
     index: InodeLeafContainerIndexInTree,
     contents: InodeLeafContainer,
     pointer: NodePointer,
     bytes: Vec<u8>,
-    is_rewritten_this_publish: bool,
+    origin: InodeLeafContainerOrigin,
 }
 
 /// 装这次发布的 inode 树（字节表四）：先按叶序给这次重写的每一片发出生序号、装码 3 容器，再装码 2 根
 /// （树内先叶后根，D3（空间分配） 已定项 10 ⑤ 的 bump 次序与 D19（块指针的结构与宽度预算） 已定项 9 的发号次序）。
-/// 没重写的那几片一个字节都不动：字节、落点、指针、出生序号全取上一版（分裂时的左半走的就是这一支）。
+/// 没重写的那几片一个字节都不动：字节、落点、指针、出生序号全取上一版它所在的那个叶序（分裂时的左半走的就是这一支；
+/// 左边有片身份消失时，它在上一版里的叶序比这一版大，`InodeLeafContainersAfterThisPublish::origins` 记着）。
 ///
 /// # Panics
 /// 这次重写的某一片没拿到落点，或某一片既不在这次的重写清单里、上一版里也没有——两样都说明角色清单
@@ -7297,10 +7557,26 @@ fn build_inode_tree_units(
     let instance = checkpoint.write_order.instance;
     let inode_tree_identifier = checkpoint.trees.inode;
     let mut leaf_containers = Vec::with_capacity(inode_tree.containers.len());
-    for (position, contents) in inode_tree.containers.iter().enumerate() {
+    for ((position, contents), origin) in inode_tree
+        .containers
+        .iter()
+        .enumerate()
+        .zip(&inode_tree.origins)
+    {
         let index = InodeLeafContainerIndexInTree::of_position(position);
         let role = TransactionUnit::InodeLeafContainer(index);
-        if inode_tree.rewritten.contains(&index) {
+        if let InodeLeafContainerOrigin::CarriedFrom(previous_index) = origin {
+            let carried = previous_containers
+                .get(previous_index.position())
+                .expect("照抄的叶容器一定在上一版里（新建的那几片恒在重写清单里）");
+            leaf_containers.push(InodeLeafContainerUnit {
+                index,
+                contents: contents.clone(),
+                pointer: carried.pointer,
+                bytes: Vec::new(),
+                origin: *origin,
+            });
+        } else {
             let birth_sequence = sequences.next(inode_tree_identifier, txg, instance);
             let bytes = build_packed_unit(
                 contents.identity,
@@ -7333,18 +7609,7 @@ fn build_inode_tree_units(
                 contents: contents.clone(),
                 pointer,
                 bytes,
-                is_rewritten_this_publish: true,
-            });
-        } else {
-            let carried = previous_containers
-                .get(position)
-                .expect("没重写的叶容器一定在上一版里（新建的那几片恒在重写清单里）");
-            leaf_containers.push(InodeLeafContainerUnit {
-                index,
-                contents: contents.clone(),
-                pointer: carried.pointer,
-                bytes: Vec::new(),
-                is_rewritten_this_publish: false,
+                origin: *origin,
             });
         }
     }
@@ -7440,6 +7705,7 @@ struct DeletionTreeUnits {
     deleted_inodes_root: Option<DeletionTreeNodeUnit>,
     tombstone: ContainerTreeUnits<TombstoneContainerIndexInTree, TombstoneRecord>,
     lineage: ContainerTreeUnits<LineageContainerIndexInTree, LineageRecord>,
+    retirement: ContainerTreeUnits<RetirementContainerIndexInTree, RetirementRecord>,
 }
 
 /// 装一棵容器树（墓碑树、谱系树）这一版的单元要的几样：哪棵树、这次之后的样子、记录宽与记录的字节、三种角色、
@@ -7495,7 +7761,7 @@ where
             let birth_sequence = sequences.next(to_build.tree, txg, instance);
             let bytes = build_packed_unit(
                 contents.identity,
-                u16::try_from(to_build.record_width_in_bytes).expect("记录宽是 40 或 16"),
+                u16::try_from(to_build.record_width_in_bytes).expect("记录宽是 40、24 或 34"),
                 &contents
                     .records
                     .iter()
@@ -7760,10 +8026,44 @@ fn build_deletion_tree_units(
         slots,
         sequences,
     );
+    let carried_retirement_container_pointer = |position: usize| {
+        carried_previous()
+            .retirement_containers
+            .get(position)
+            .expect("没重写的退役容器一定在上一版里（新开的那几片恒在重写清单里）")
+            .pointer
+    };
+    let carried_retirement_node_pointer = |position: ContainerTreeNodePosition| {
+        carried_previous()
+            .retirement_tree_index_nodes
+            .pointer_below_the_root(position)
+            .expect(
+                "照抄的退役树根之下节点一定在上一版里（新开的节点恒是这次重写的容器的祖先、重写）",
+            )
+    };
+    let carried_retirement_root_pointer =
+        || carried_previous().tree_root_pointer(checkpoint.trees.retirement.0);
+    let retirement = build_container_tree_units(
+        &ContainerTreeToBuild {
+            tree: checkpoint.trees.retirement,
+            after_this_publish: &resolved.retirement_tree,
+            record_width_in_bytes: RETIREMENT_RECORD_BYTES,
+            record_bytes: RetirementRecord::to_bytes,
+            index_of_position: RetirementContainerIndexInTree::of_position,
+            roles: ContainerTreeRoles::OF_THE_RETIREMENT_TREE,
+            carried_container_pointer: &carried_retirement_container_pointer,
+            carried_node_below_the_root_pointer: &carried_retirement_node_pointer,
+            carried_root_pointer: &carried_retirement_root_pointer,
+        },
+        checkpoint,
+        slots,
+        sequences,
+    );
     DeletionTreeUnits {
         deleted_inodes_root,
         tombstone,
         lineage,
+        retirement,
     }
 }
 
@@ -7794,7 +8094,7 @@ fn carried_file_version_units(
     clippy::too_many_lines,
     clippy::too_many_arguments,
     reason = "一次发布就是一件能单独验证的事：九个角色的装法与一条持久顺序，拆开只会把顺序藏进几个函数；\
-              参数各是一样东西：写入口、分配器、计划、上一版、算好的树、要释放的、要隔离的、树号"
+              参数各是一样东西：写入口、分配器、计划、上一版、算好的树、要释放的、要隔离的、树号、卸载记号、谱系这一步"
 )]
 fn publish_admitted<Device: BlockDevice>(
     pool: &PoolWriter<'_, Device>,
@@ -7806,6 +8106,7 @@ fn publish_admitted<Device: BlockDevice>(
     quarantine: &[CopyQuarantinedAfterReleaseChecksumMismatch],
     trees: FileVersionTreeIdentifiers,
     unmount_marker: UnmountMarker,
+    lineage: LineageStepOfThePublish,
 ) -> Result<(PublishWrites, TransactionOutput), PublishError> {
     let rewritten = &resolved.rewritten_roles;
     let txg = plan.txg;
@@ -8023,7 +8324,7 @@ fn publish_admitted<Device: BlockDevice>(
     // 这里按「已经发出去的号」算，不按「树里现在有什么」算：号一旦发出去就不再复用（D8（核心索引结构） 已定项 6）。
     let new_inodes_of_this_publish =
         u64::try_from(plan.new_inode_records.len()).expect("这次新建的 inode 数");
-    let inode_number_watermark = match previous {
+    let inode_number_watermark_counted_from_the_previous_version = match previous {
         // 只供测试的开关装着时从压回去的那个号起算（C479），与 `publish_new_inodes` 起号读的是同一个开关、同一份上一版。
         Some(previous_version) => pool
             .inode_number_watermark_source()
@@ -8033,15 +8334,36 @@ fn publish_admitted<Device: BlockDevice>(
             .inode_number_watermark_source()
             .watermark_of_a_publish_without_a_previous_version(),
     } + new_inodes_of_this_publish;
+    let inode_number_watermark = match lineage {
+        // 管理员回退那次：水位是回退那一判算好的（D23（journal 的角色与格式） 已定项 14「水位」一格的 max），这次不建 inode；
+        // 上一版（`mount::version_to_publish_the_rollback_after`）的记账行原样是 cur 的，台阶行照抄 cur 的。
+        LineageStepOfThePublish::AdministratorRollback {
+            inode_number_watermark_after_the_rollback,
+            ..
+        } => {
+            assert_eq!(
+                new_inodes_of_this_publish, 0,
+                "管理员回退那次发布不建 inode（`mount::roll_back_by_a_forward_publish` 的计划里新 inode 记录是空的）"
+            );
+            inode_number_watermark_after_the_rollback
+        }
+        LineageStepOfThePublish::NoLineageRecord => {
+            inode_number_watermark_counted_from_the_previous_version
+        }
+    };
     let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator
         .devices
         .iter()
         .map(|device_map| device_map.device)
         .collect();
+    // inode 号水位的台阶行（D5（快照 / 空间记账机制） 已定项 2 射程）：照抄上一版的，与准入之前算记账树形状读的是同一份。
+    let inode_number_watermark_steps_carried =
+        previous.map_or_else(Vec::new, TransactionOutput::inode_number_watermark_steps);
     let accounting_entries = accounting_entries_of_this_publish(
         txg,
         trees.inode,
         &device_identities_of_the_accounting_rows,
+        &inode_number_watermark_steps_carried,
         &|row| match row {
             PoolWideAccountingRow::InodeNumberWatermark => inode_number_watermark,
             // 待删占用与准入读数读的是同一个常量（`admission`）：一处定义，两处各抄一个 0 会分叉。
@@ -8050,7 +8372,7 @@ fn publish_admitted<Device: BlockDevice>(
             }
             // 标签 6 按这一版现算（D3（空间分配） 已定项 2）：发布路径判式子时读的是同一个函数算的同一个数。
             PoolWideAccountingRow::CommittedReservationBytes => {
-                committed_reservation_for_tombstones_after_this_publish(
+                committed_reservation_for_tombstones_and_retirement_records_after_this_publish(
                     resolved,
                     previous,
                     ReplicaCount::of_every_device_in_the_pool(allocator),
@@ -8260,6 +8582,27 @@ fn publish_admitted<Device: BlockDevice>(
             root.pointer.locations,
         ));
     }
+    for container in &deletion_tree_units.retirement.containers {
+        mapped_units_with_locations.push((
+            TransactionUnit::RetirementLeafContainer(container.index),
+            mapping_key_for_node(UNIT_CLASS_PACKED, container.node.pointer),
+            container.node.pointer.locations,
+        ));
+    }
+    for node in &deletion_tree_units.retirement.nodes_below_the_root {
+        mapped_units_with_locations.push((
+            node.identity,
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, node.node.pointer),
+            node.node.pointer.locations,
+        ));
+    }
+    if let Some(root) = &deletion_tree_units.retirement.root {
+        mapped_units_with_locations.push((
+            TransactionUnit::RetirementRoot,
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, root.pointer),
+            root.pointer.locations,
+        ));
+    }
     let mapped_units: Vec<(TransactionUnit, Vec<u8>)> = mapped_units_with_locations
         .iter()
         .map(|(identity, key, _)| (*identity, key.clone()))
@@ -8386,12 +8729,24 @@ fn publish_admitted<Device: BlockDevice>(
             NodePointer::empty_root(),
             0,
         ),
+        // 退役树（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）：day-1 零根注册、排在最后，
+        // 头 ID 写 inode 树的树 ID（同墓碑树）；零根时根指针置零，有身份消失之后指着它的根，之后每一版照抄（随现行那一版）。
+        table_entry(
+            TREE_KIND_RETIREMENT,
+            trees.retirement,
+            deletion_tree_units
+                .retirement
+                .root
+                .as_ref()
+                .map_or_else(NodePointer::empty_root, |root| root.pointer),
+            trees.inode.0,
+        ),
     ];
     assert!(
         tree_table_entries
             .windows(2)
             .all(|pair| pair[0].tree < pair[1].tree),
-        "树表条目按树 ID 升序（D8（核心索引结构） 已定项 8 排序契约）：十二个号连号发、中央映射树不进树表，剩下十一条仍按发号次序升序"
+        "树表条目按树 ID 升序（D8（核心索引结构） 已定项 8 排序契约）：十三个号连号发、中央映射树不进树表，剩下十二条仍按发号次序升序"
     );
     let tree_table_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
     let tree_table_unit = build_index_node(
@@ -8399,7 +8754,7 @@ fn publish_admitted<Device: BlockDevice>(
         0,
         TREE_TABLE_KEY_WIDTH,
         &trees.extent.0.to_le_bytes(),
-        &trees.logged_operations.0.to_le_bytes(),
+        &trees.retirement.0.to_le_bytes(),
         txg,
         filesystem_identifier,
         instance,
@@ -8437,13 +8792,22 @@ fn publish_admitted<Device: BlockDevice>(
     units.extend(extent_built.units.iter().cloned());
     for container in &inode_tree_units.leaf_containers {
         let identity = TransactionUnit::InodeLeafContainer(container.index);
-        units.push(if container.is_rewritten_this_publish {
-            rewritten_unit(identity, container.bytes.clone())
-        } else {
-            carried_unit(
-                previous.expect("没重写的叶容器只会出现在接着上一版的发布里"),
-                identity,
-            )
+        units.push(match container.origin {
+            InodeLeafContainerOrigin::RewrittenThisPublish => {
+                rewritten_unit(identity, container.bytes.clone())
+            }
+            // 照抄上一版那个叶序上的那一份（字节、落点不动），装进这一版时换成这一版的角色。
+            InodeLeafContainerOrigin::CarriedFrom(previous_index) => {
+                let carried = carried_unit(
+                    previous.expect("没重写的叶容器只会出现在接着上一版的发布里"),
+                    TransactionUnit::InodeLeafContainer(previous_index),
+                );
+                PublishedUnit {
+                    slot: carried.slot,
+                    identity,
+                    bytes: carried.bytes,
+                }
+            }
         });
     }
     units.push(match &inode_tree_units.rewritten_root {
@@ -8522,6 +8886,18 @@ fn publish_admitted<Device: BlockDevice>(
     if let Some(root) = &deletion_tree_units.lineage.root {
         units.push(deletion_tree_unit(TransactionUnit::LineageRoot, root));
     }
+    for container in &deletion_tree_units.retirement.containers {
+        units.push(deletion_tree_unit(
+            TransactionUnit::RetirementLeafContainer(container.index),
+            &container.node,
+        ));
+    }
+    for node in &deletion_tree_units.retirement.nodes_below_the_root {
+        units.push(container_tree_node_below_the_root_unit(node));
+    }
+    if let Some(root) = &deletion_tree_units.retirement.root {
+        units.push(deletion_tree_unit(TransactionUnit::RetirementRoot, root));
+    }
     units.extend(mapping_built.units.iter().cloned());
     units.push(rewritten_unit(
         TransactionUnit::TreeTable,
@@ -8671,6 +9047,37 @@ fn publish_admitted<Device: BlockDevice>(
                     .expect("点名的谱系树根之下节点是这次重写的")
                     .birth_sequence,
             ),
+            TransactionUnit::RetirementLeafContainer(index) => node_key_tail(
+                instance,
+                deletion_tree_units
+                    .retirement
+                    .containers
+                    .get(index.position())
+                    .and_then(|container| container.node.rewritten.as_ref())
+                    .expect("点名的退役容器是这次重写的")
+                    .birth_sequence,
+            ),
+            TransactionUnit::RetirementRoot => node_key_tail(
+                instance,
+                deletion_tree_units
+                    .retirement
+                    .root
+                    .as_ref()
+                    .and_then(|root| root.rewritten.as_ref())
+                    .expect("点名退役树根的发布一定重写了它")
+                    .birth_sequence,
+            ),
+            TransactionUnit::RetirementNodeBelowTheRoot(_) => node_key_tail(
+                instance,
+                deletion_tree_units
+                    .retirement
+                    .nodes_below_the_root
+                    .iter()
+                    .find(|node| node.identity == identity)
+                    .and_then(|node| node.node.rewritten.as_ref())
+                    .expect("点名的退役树根之下节点是这次重写的")
+                    .birth_sequence,
+            ),
             TransactionUnit::TreeTable => node_key_tail(instance, tree_table_sequence),
             TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                 node_key_tail(
@@ -8857,6 +9264,24 @@ fn publish_admitted<Device: BlockDevice>(
                 .map(|node| (node.position, node.node.pointer))
                 .collect(),
         },
+        retirement_containers: deletion_tree_units
+            .retirement
+            .containers
+            .into_iter()
+            .map(|container| RetirementContainerVersion {
+                contents: container.contents,
+                pointer: container.node.pointer,
+            })
+            .collect(),
+        retirement_tree_index_nodes: ContainerTreeIndexNodesVersion {
+            shape: resolved.retirement_tree.shape.clone(),
+            pointers_below_the_root: deletion_tree_units
+                .retirement
+                .nodes_below_the_root
+                .iter()
+                .map(|node| (node.position, node.node.pointer))
+                .collect(),
+        },
         inode_leaf_containers: inode_tree_units
             .leaf_containers
             .into_iter()
diff --git a/crates/singlefs-core/src/unit.rs b/crates/singlefs-core/src/unit.rs
index 5a81fef..d8a5884 100644
--- a/crates/singlefs-core/src/unit.rs
+++ b/crates/singlefs-core/src/unit.rs
@@ -17,9 +17,11 @@ pub const FORMAT_VERSION: u16 = 1;
 pub const UNIT_CLASS_DATA: u8 = 1;
 pub const UNIT_CLASS_INDEX_NODE: u8 = 2;
 pub const UNIT_CLASS_PACKED: u8 = 3;
-/// 打包记录类型（第二级登记表）：1 = 墓碑（D18（块里携带什么信息） 已定项 24），2 = inode 记录，4 = 实例表。
+/// 打包记录类型（第二级登记表）：1 = 墓碑（D18（块里携带什么信息） 已定项 24），2 = inode 记录，3 = 退役记录，4 = 实例表。
 pub const PACKED_TYPE_TOMBSTONE: u16 = 1;
 pub const PACKED_TYPE_INODE: u16 = 2;
+/// 退役记录（D18（块里携带什么信息） 已定项 11 登记的类型 3；记录宽 34，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2 条）。
+pub const PACKED_TYPE_RETIREMENT: u16 = 3;
 pub const PACKED_TYPE_INSTANCE_TABLE: u16 = 4;
 /// 谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条：不复用登记过的类型 3）。
 pub const PACKED_TYPE_LINEAGE: u16 = 5;
diff --git a/crates/singlefs-core/src/unlink.rs b/crates/singlefs-core/src/unlink.rs
index 056d205..9a1d7b5 100644
--- a/crates/singlefs-core/src/unlink.rs
+++ b/crates/singlefs-core/src/unlink.rs
@@ -12,6 +12,7 @@ use crate::container_tree::{ContainerTreeCapacities, ContainerTreeNodeEntries};
 use crate::extent_tree::{lower_segment_nodes_of_a_file_without_holes, upper_path_of_inode};
 use crate::extent_tree::{ExtentLowerNodePosition, ExtentTreeVersion};
 use crate::lineage_tree::LineageContainerCapacity;
+use crate::retirement_tree::RetirementContainerCapacity;
 use crate::tombstone_tree::TombstoneContainerCapacity;
 use crate::transaction::{
     role_of_extent_upper_node, ExtentTreeNodeOrigin, ExtentTreePlan, TransactionUnit,
@@ -19,7 +20,7 @@ use crate::transaction::{
 use crate::unit::data_unit_payload_capacity;
 
 /// 删除与回退那几次发布的只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：每批几个数据单元、一片墓碑容器装满几条、
-/// 一片谱系容器装满几条（管理员回退那次追加）、容器树一个码 2 节点装满几条（墓碑树、谱系树根满长高）。
+/// 一片谱系容器装满几条（管理员回退那次追加）、一片退役容器装满几条（最后一批追加）、容器树一个码 2 节点装满几条（墓碑树、谱系树、退役树根满长高）。
 /// 产品路径是 [`DeletionSwitches::PRODUCT`]；都只住内存、不落盘（D18（块里携带什么信息） 已定项 24 第 3 条）。
 /// 经可写挂载的只供测试的开关链装在分配器上（`mount::mount_writable_with_test_only_switches_and_deletion_switches`，与空间准入开关同一个做法），
 /// 挂载续做与这次挂载之后的会话都照它。
@@ -28,6 +29,7 @@ pub struct DeletionSwitches {
     pub batch_size: UnlinkBatchSize,
     pub tombstone_container_capacity: TombstoneContainerCapacity,
     pub lineage_container_capacity: LineageContainerCapacity,
+    pub retirement_container_capacity: RetirementContainerCapacity,
     pub container_tree_node_entries: ContainerTreeNodeEntries,
 }
 
@@ -36,15 +38,17 @@ impl DeletionSwitches {
         batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
         tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
         lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container_capacity: RetirementContainerCapacity::FromTheContainerFormat,
         container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     };
 
-    /// 墓碑树与谱系树那三个容量开关，装上写入口用（`transaction::PoolWriter::set_container_tree_capacities`）。
+    /// 墓碑树、谱系树与退役树那四个容量开关，装上写入口用（`transaction::PoolWriter::set_container_tree_capacities`）。
     #[must_use]
     pub const fn container_tree_capacities(self) -> ContainerTreeCapacities {
         ContainerTreeCapacities {
             tombstone_container: self.tombstone_container_capacity,
             lineage_container: self.lineage_container_capacity,
+            retirement_container: self.retirement_container_capacity,
             node_entries: self.container_tree_node_entries,
         }
     }
diff --git a/crates/singlefs-core/src/write_accounting.rs b/crates/singlefs-core/src/write_accounting.rs
index 98f4334..5c0d50e 100644
--- a/crates/singlefs-core/src/write_accounting.rs
+++ b/crates/singlefs-core/src/write_accounting.rs
@@ -24,6 +24,8 @@ pub enum WrittenStructureKind {
     TombstoneTreeNode,
     /// 谱系树的容器与根（D18（块里携带什么信息） 已定项 24 第 5 条）：另立一种、不并进墓碑树那一种（用户 2026-10-02 定「另立一种」），按树数。
     LineageTreeNode,
+    /// 退役树的容器与根（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条）：照谱系树另立一种，按树数。
+    RetirementTreeNode,
     CentralMappingTreeNode,
     TreeTableUnit,
     InstanceTableUnit,
@@ -35,7 +37,7 @@ pub enum WrittenStructureKind {
 impl WrittenStructureKind {
     /// 报数的次序：单元按 bump 次序（实例表单元在末尾，同 `TransactionOutput::units`），固定结构按持久顺序（D16（发布语义） 已定项 7）。
     /// 合计不经过这张清单（`WritesByStructureKind::total` 只从记过的账里加）：清单漏一种，报出的各种之和就对不上合计。
-    pub const IN_REPORT_ORDER: [WrittenStructureKind; 15] = [
+    pub const IN_REPORT_ORDER: [WrittenStructureKind; 16] = [
         WrittenStructureKind::DataUnit,
         WrittenStructureKind::ExtentTreeNode,
         WrittenStructureKind::InodeTreeLeafContainer,
@@ -45,6 +47,7 @@ impl WrittenStructureKind {
         WrittenStructureKind::DeletedInodesTreeNode,
         WrittenStructureKind::TombstoneTreeNode,
         WrittenStructureKind::LineageTreeNode,
+        WrittenStructureKind::RetirementTreeNode,
         WrittenStructureKind::CentralMappingTreeNode,
         WrittenStructureKind::TreeTableUnit,
         WrittenStructureKind::InstanceTableUnit,
@@ -81,6 +84,11 @@ impl WrittenStructureKind {
                 WrittenStructureKind::TombstoneTreeNode
             }
             TransactionUnit::LineageNodeBelowTheRoot(_) => WrittenStructureKind::LineageTreeNode,
+            TransactionUnit::RetirementLeafContainer(_)
+            | TransactionUnit::RetirementRoot
+            | TransactionUnit::RetirementNodeBelowTheRoot(_) => {
+                WrittenStructureKind::RetirementTreeNode
+            }
             TransactionUnit::MappingTreeNodeBelowTheRoot(_) | TransactionUnit::MappingTree => {
                 WrittenStructureKind::CentralMappingTreeNode
             }
@@ -104,6 +112,7 @@ impl WrittenStructureKind {
             WrittenStructureKind::DeletedInodesTreeNode => "deleted_inodes_tree_node",
             WrittenStructureKind::TombstoneTreeNode => "tombstone_tree_node",
             WrittenStructureKind::LineageTreeNode => "lineage_tree_node",
+            WrittenStructureKind::RetirementTreeNode => "retirement_tree_node",
             WrittenStructureKind::CentralMappingTreeNode => "central_mapping_tree_node",
             WrittenStructureKind::TreeTableUnit => "tree_table_unit",
             WrittenStructureKind::InstanceTableUnit => "instance_table_unit",
@@ -280,6 +289,7 @@ mod tests {
             TransactionUnit::DeletedInodesRoot,
             TransactionUnit::TombstoneRoot,
             TransactionUnit::LineageRoot,
+            TransactionUnit::RetirementRoot,
             TransactionUnit::MappingTree,
             TransactionUnit::TreeTable,
             TransactionUnit::InstanceTable,
@@ -302,7 +312,7 @@ mod tests {
             assert_eq!(
                 every_kind_once.of(kind).write_calls,
                 1,
-                "{kind:?}：十二个单元角色各归一种、三种固定结构各一种，每种恰好一笔"
+                "{kind:?}：十三个单元角色各归一种、三种固定结构各一种，每种恰好一笔"
             );
         }
         let reported = WrittenStructureKind::IN_REPORT_ORDER
@@ -318,10 +328,10 @@ mod tests {
         assert_eq!(
             every_kind_once.total(),
             WriteCallsAndBytes {
-                write_calls: 15,
-                written_bytes: 32767
+                write_calls: 16,
+                written_bytes: 65535
             },
-            "十五笔，长度 1 到 16384 各一次"
+            "十六笔，长度 1 到 32768 各一次"
         );
         let mut names: Vec<&str> = WrittenStructureKind::IN_REPORT_ORDER
             .iter()
@@ -329,6 +339,6 @@ mod tests {
             .collect();
         names.sort_unstable();
         names.dedup();
-        assert_eq!(names.len(), 15, "结果行的字段名前缀互不相同");
+        assert_eq!(names.len(), 16, "结果行的字段名前缀互不相同");
     }
 }
diff --git a/crates/singlefs-format/src/lib.rs b/crates/singlefs-format/src/lib.rs
index d047b46..d8b5f33 100644
--- a/crates/singlefs-format/src/lib.rs
+++ b/crates/singlefs-format/src/lib.rs
@@ -115,11 +115,19 @@ pub const TOMBSTONE_RECORD_BYTES: u64 = 40;
 /// 一个 32 KiB 码 3 墓碑容器装几条：(32768 − 136) ÷ 40（D18（块里携带什么信息） 已定项 24 第 2 条）。
 pub const TOMBSTONE_CONTAINER_RECORDS: u64 = 815;
 
-/// 谱系记录定长 16：T_rb 8 + T_old 8（D18（块里携带什么信息） 已定项 24 第 5 条，打包记录类型 5）。
-pub const LINEAGE_RECORD_BYTES: u64 = 16;
+/// 谱系记录定长 24：T_rb 8 + T_old 8 + H_cur 8（D18（块里携带什么信息） 已定项 24 第 5 条，打包记录类型 5）。
+pub const LINEAGE_RECORD_BYTES: u64 = 24;
 
-/// 一个 32 KiB 码 3 谱系容器装几条：(32768 − 136) ÷ 16（D18（块里携带什么信息） 已定项 24 第 5 条）。
-pub const LINEAGE_CONTAINER_RECORDS: u64 = 2039;
+/// 一个 32 KiB 码 3 谱系容器装几条：(32768 − 136) ÷ 24（D18（块里携带什么信息） 已定项 24 第 5 条）。
+pub const LINEAGE_CONTAINER_RECORDS: u64 = 1359;
+
+/// 退役记录定长 34：被退役身份的出生树 8 + 打包记录类型 2 + 容器号 8 + 容器出生代 8 + 死亡 txg 8，小端，身份四段与 inode 内部条目里的
+/// 身份引用同序同宽、无填充无预留（D18（块里携带什么信息） 已定项 11 打包记录类型 3；判决 `research/prompts/m3-retire-r1-main-verification.md`
+/// 第三节第 2 条）。format-const: RETIREMENT_RECORD_BYTES
+pub const RETIREMENT_RECORD_BYTES: u64 = 34;
+
+/// 一个 32 KiB 码 3 退役容器装几条：(32768 − 136) ÷ 34（同一判决第三节第 2 条）。format-const: RETIREMENT_CONTAINER_RECORDS
+pub const RETIREMENT_CONTAINER_RECORDS: u64 = 959;
 
 /// `deleted_inodes` 树节点的条目：key = inode 号 8 字节、无 value、不带对象出生代（D8（核心索引结构） 已定项 16 第 1 条）。
 pub const DELETED_INODES_ENTRY_BYTES: u64 = 8;
@@ -184,8 +192,9 @@ pub const NEW_POOL_FILE_CREATION_ACCOUNTING_ROWS: u64 = 15;
 pub const TREE_TABLE_ENTRY_BYTES: u64 = 200;
 
 /// 新池新建文件写出的树表条目数（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist，
-/// 加 `deleted_inodes`、墓碑树、谱系树、logged_ops 四棵零根注册的树：D8（核心索引结构） 已定项 16 第 3 条、D18（块里携带什么信息） 已定项 24 第 6 条）。
-pub const NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = 11;
+/// 加 `deleted_inodes`、墓碑树、谱系树、logged_ops、退役树五棵零根注册的树：D8（核心索引结构） 已定项 9 / 已定项 16 第 3 条、
+/// D18（块里携带什么信息） 已定项 24 第 6 条；退役树见判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）。
+pub const NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = 12;
 
 /// 树 ID 从 65536 起（树的种类那 2 字节取不到的最小值），与树的种类码值域不相交（D8（核心索引结构） 已定项 11，用户 2026-10-01 定案整体挪开）。
 pub const TREE_IDENTIFIER_EXTENT: u64 = 65536;
@@ -202,9 +211,12 @@ pub const TREE_IDENTIFIER_DELETED_INODES: u64 = 65544;
 pub const TREE_IDENTIFIER_TOMBSTONE: u64 = 65545;
 pub const TREE_IDENTIFIER_LINEAGE: u64 = 65546;
 pub const TREE_IDENTIFIER_LOGGED_OPERATIONS: u64 = 65547;
-/// 树 ID 水位：mkfs 写 65536，第一个文件版本那次之后写 65548（D8（核心索引结构） 已定项 8 ② / 已定项 11）。
+/// 退役树：同样在第一个文件版本那次零根注册，接在 logged_ops 之后（D8（核心索引结构） 已定项 11；判决
+/// `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）。
+pub const TREE_IDENTIFIER_RETIREMENT: u64 = 65548;
+/// 树 ID 水位：mkfs 写 65536，第一个文件版本那次之后写 65549（D8（核心索引结构） 已定项 8 ② / 已定项 11）。
 pub const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u64 = 65536;
-pub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 65548;
+pub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 65549;
 
 /// 根记录：magic 4 + fsid 16 + flags 4 + 实例代号 4 + checkpoint_txg 8 + 树表指针 86 + 树 ID 水位 8 + 回退下界 F 8 + 自证校验和 32 + 实例表指针 86 + 中央映射树根指针 86 + 分配记录树根指针 86 + 算法类型 1 + nonce 12 + MAC 16（D22（单元原子性怎么合成） 已定项 7；加密预留 2026-09-14 用户定案；分配记录树根指针 2026-09-23 用户定案随 C512（树表 0 条的一版上被换下的单元记在哪） 加）。format-const: ROOT_RECORD_BYTES
 pub const ROOT_RECORD_BYTES: u64 = 457;
@@ -457,15 +469,33 @@ mod tests {
             TOMBSTONE_CONTAINER_RECORDS, 815,
             "D18 已定项 24 第 2 条写的 815"
         );
-        assert_eq!(LINEAGE_RECORD_BYTES, 8 + 8, "谱系记录 = T_rb + T_old");
+        assert_eq!(
+            LINEAGE_RECORD_BYTES,
+            8 + 8 + 8,
+            "谱系记录 = T_rb + T_old + H_cur"
+        );
         assert_eq!(
             LINEAGE_CONTAINER_RECORDS,
             (DATA_UNIT_BYTES - PACKED_UNIT_RECORDS_OFFSET) / LINEAGE_RECORD_BYTES,
             "一个 32 KiB 谱系容器从记录区起点 136 装到单元末尾的条数"
         );
         assert_eq!(
-            LINEAGE_CONTAINER_RECORDS, 2039,
-            "D18 已定项 24 第 5 条写的 2039"
+            LINEAGE_CONTAINER_RECORDS, 1359,
+            "D18 已定项 24 第 5 条写的 1359"
+        );
+        assert_eq!(
+            RETIREMENT_RECORD_BYTES,
+            8 + 2 + 8 + 8 + 8,
+            "退役记录 = 出生树 + 打包记录类型 + 容器号 + 容器出生代 + 死亡 txg"
+        );
+        assert_eq!(
+            RETIREMENT_CONTAINER_RECORDS,
+            (DATA_UNIT_BYTES - PACKED_UNIT_RECORDS_OFFSET) / RETIREMENT_RECORD_BYTES,
+            "一个 32 KiB 退役容器从记录区起点 136 装到单元末尾的条数"
+        );
+        assert_eq!(
+            RETIREMENT_CONTAINER_RECORDS, 959,
+            "判决 m3-retire-r1 第三节第 2 条写的 959"
         );
         assert_eq!(
             DELETED_INODES_ENTRY_BYTES, 8,
@@ -626,7 +656,7 @@ mod tests {
         );
     }
 
-    /// 十二棵树（八棵加第一个文件版本那次零根注册的四棵）的树 ID 与两个水位逐个等于 D8（核心索引结构） 已定项 11 登记的值
+    /// 十三棵树（八棵加第一个文件版本那次零根注册的五棵）的树 ID 与两个水位逐个等于 D8（核心索引结构） 已定项 11 登记的值
     /// （用户 2026-10-01 定案「树 ID 整体挪开」）：从 65536 起连号，起点是树的种类那 2 字节取不到的最小值，水位在最后一个之后。
     #[test]
     fn tree_identifiers_are_the_registered_values_in_a_row_from_the_first_value_the_kind_field_cannot_hold(
@@ -644,10 +674,11 @@ mod tests {
             (TREE_IDENTIFIER_TOMBSTONE, 65545, "墓碑树"),
             (TREE_IDENTIFIER_LINEAGE, 65546, "谱系树"),
             (TREE_IDENTIFIER_LOGGED_OPERATIONS, 65547, "logged_ops 树"),
+            (TREE_IDENTIFIER_RETIREMENT, 65548, "退役树"),
             (TREE_IDENTIFIER_WATERMARK_AT_MKFS, 65536, "mkfs 时的水位"),
             (
                 TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
-                65548,
+                65549,
                 "第一个文件版本那次之后的水位",
             ),
         ] {
@@ -674,19 +705,20 @@ mod tests {
             TREE_IDENTIFIER_TOMBSTONE,
             TREE_IDENTIFIER_LINEAGE,
             TREE_IDENTIFIER_LOGGED_OPERATIONS,
+            TREE_IDENTIFIER_RETIREMENT,
         ];
         for (position, identifier) in identifiers.iter().enumerate() {
             let expected = TREE_IDENTIFIER_WATERMARK_AT_MKFS
-                + u64::try_from(position).expect("十二棵树的序号装得进 u64");
+                + u64::try_from(position).expect("十三棵树的序号装得进 u64");
             assert_eq!(*identifier, expected, "树 ID 连号");
         }
         assert_eq!(
             TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
-            TREE_IDENTIFIER_LOGGED_OPERATIONS + 1
+            TREE_IDENTIFIER_RETIREMENT + 1
         );
         assert_eq!(
-            NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES, 11,
-            "D18 已定项 24 第 6 条：树表 7 → 11 条"
+            NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES, 12,
+            "D18 已定项 24 第 6 条树表 7 → 11 条，退役树（判决 m3-retire-r1 第三节第 1 条）11 → 12 条"
         );
         assert_eq!(
             u64::try_from(ROOT_RING_REGION_DEVICES.len()).expect("三个区域"),
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index 3150c2b..e1de598 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -143,19 +143,23 @@ const SMALLER_DEVICE_UNIT_AREA_SLOTS_BEYOND_THE_RING_WINDOW_ALLOWANCE: u64 = 256
 const NARROWEST_DEVICE_UNIT_AREA_SLOTS_BEYOND_THE_RING_WINDOW_ALLOWANCE: u64 = 240;
 
 /// 384 槽那一档小盘为环窗口补足的单元区槽数（D28（挂载期承诺量） 已定项 1 末项环窗口每块盘常驻扣；用户 2026-10-02 定「甲」：小盘放大到扣得起）。
-/// 放大之后单元区 [1408, 2607) 每块盘罩分配记录树叶 3 片（812 槽一片：⌊2606 ÷ 812⌋ − ⌊1408 ÷ 812⌋ + 1）⇒ N_0 = 6、W = 24 + 6 = 30、
-/// 环窗口 (3 × 8 + 1) × 30 = 750 槽。同一次补足还罩着 ckpt_cost 多出来的 5 块：叶多罩一片，分配记录树那一项 1 + 2 × 2 = 5 → 1 + 2 × 3 = 7
-/// （多 2 块），谱系那一项 3 块（同日定「乙」）；保留池每块盘多 5 槽、切换预留里暖机那一半多 (3 + 1) × 3 × 5 = 60 槽。
-/// 750 + 5 + 60 = 815：准入算出的可用(d) 与放大之前、ckpt_cost 加谱系那一项之前逐盘一样多（推的；每次发布改哪几片叶跟着盘宽变，实测见用例）。
-const SMALL_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS: u64 = 815;
-
-/// 256 槽、240 槽那两档小盘为环窗口补足的单元区槽数：放大之后单元区 [1408, 2403) 与 [1408, 2387) 每块盘仍只罩分配记录树叶 2 片
-/// （⌊2402 ÷ 812⌋ = ⌊2386 ÷ 812⌋ = 2）⇒ N_0 = 4、W = 28、环窗口 25 × 28 = 700 槽；ckpt_cost 只多谱系那一项 3 块：
-/// 保留池每块盘多 3 槽、切换预留多 12 × 3 = 36 槽。700 + 3 + 36 = 739。
-const SMALLER_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS: u64 = 739;
+/// 放大之后单元区 [1408, 2799) 每块盘罩分配记录树叶 3 片（812 槽一片：⌊2798 ÷ 812⌋ − ⌊1408 ÷ 812⌋ + 1）⇒ N_0 = 6、
+/// W = 24 + (6 + 0) + 6 = 36（退役树那一项 6 + H_退役，H_退役 = 0）、环窗口 (3 × 8 + 1) × 36 = 900 槽。同一次补足还罩着 ckpt_cost 多出来的 8 块：
+/// 叶多罩一片，分配记录树那一项 1 + 2 × 2 = 5 → 1 + 2 × 3 = 7（多 2 块），谱系那一项 3 块（同日定「乙」），退役那一项 3 块
+/// （判决 `research/prompts/m3-retire-r1-main-verification.md` 第六节 R2）；保留池每块盘多 8 槽、切换预留里暖机那一半多 (3 + 1) × 3 × 8 = 96 槽；
+/// 标签 6 退役那一份摊到每块盘 (2 + 0 + 1) × 16384 + 35 × n_记录 字节：补整的 3 槽，35 × n_记录 字节那一截不补（同墓碑那 41 × n_live 字节当初也没补）。
+/// 900 + 8 + 96 + 3 = 1007：准入算出的可用(d) 与放大之前、ckpt_cost 加谱系与退役两项之前逐盘一样多、只少 35 × n_记录 字节（推的；每次发布改哪几片叶
+/// 跟着盘宽变，实测见用例）。第 49 批之前是 815（没有退役那几项：750 + 5 + 60）。
+const SMALL_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS: u64 = 1007;
+
+/// 256 槽、240 槽那两档小盘为环窗口补足的单元区槽数：放大之后单元区 [1408, 2671) 与 [1408, 2655) 每块盘罩分配记录树叶 3 片
+/// （⌊2670 ÷ 812⌋ = ⌊2654 ÷ 812⌋ = 3）⇒ N_0 = 6、W = 24 + (6 + 0) + 6 = 36、环窗口 25 × 36 = 900 槽；ckpt_cost 多 8 块（同上：叶多罩一片 2、
+/// 谱系 3、退役 3）：保留池每块盘多 8 槽、切换预留多 12 × 8 = 96 槽；标签 6 退役那一份补整的 3 槽。900 + 8 + 96 + 3 = 1007，两档与 384 槽那一档
+/// 补足一样多。第 49 批之前是 739（叶只罩 2 片：700 + 3 + 36），退役那几项把单元区推过第 3 片叶的起点 2436。
+const SMALLER_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS: u64 = 1007;
 
 /// 小盘上的 journal 环字节数：6 MiB。单元区从环末尾的下一个槽 1024 + 384 = 1408（= 22 × 64，落在聚簇段边界上）起，
-/// 三档小盘的盘宽 = 1408 + 环窗口补足 + 单元区槽数，最窄那档（240 槽）是 1408 + 739 + 240 = 2387 槽，环 6 MiB 不超过它的四分之一；
+/// 三档小盘的盘宽 = 1408 + 环窗口补足 + 单元区槽数，最窄那档（240 槽）是 1408 + 1007 + 240 = 2655 槽，环 6 MiB 不超过它的四分之一；
 /// 放大之前（1648 槽），7 MiB 的环在 256 槽、240 槽那两档就超了。在飞上限 = 6 MiB ÷ 4096 ÷ 3 = 512 条。
 /// 改之前是 128 MiB、单元区从编译期常量 50176 起（盘约 790 MiB）；单元区起点改成随环长走之后，128 MiB 的环把起点拉到 9216，
 /// 同样宽的盘上单元区有四万多槽、走不到单元区墙，四分之一那一关又不许环更长。
diff --git a/crates/singlefs-harness/src/model.rs b/crates/singlefs-harness/src/model.rs
index b5f222e..56545b0 100644
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -101,6 +101,11 @@ pub enum ModelUnitRole {
     TombstoneLeaf,
     /// 墓碑树的根。
     TombstoneRoot,
+    /// 退役树的开放容器（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条，照墓碑树）：模型罩的历史里只有删第一个文件
+    /// 那次最后一批让唯一那片叶容器身份消失、追加一条退役记录，退役树恒只有容器 1 那一片。随现行那一版：管理员回退不取回退目标的。
+    RetirementLeaf,
+    /// 退役树的根。
+    RetirementRoot,
 }
 
 impl ModelUnitRole {
@@ -127,7 +132,8 @@ impl ModelUnitRole {
             | ModelUnitRole::InodeLeaf
             | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
             | ModelUnitRole::InstanceTable
-            | ModelUnitRole::TombstoneLeaf => DATA_UNIT_BYTES / SLOT_BYTES,
+            | ModelUnitRole::TombstoneLeaf
+            | ModelUnitRole::RetirementLeaf => DATA_UNIT_BYTES / SLOT_BYTES,
             ModelUnitRole::ExtentRoot
             | ModelUnitRole::InodeRoot
             | ModelUnitRole::AllocationTree
@@ -135,7 +141,8 @@ impl ModelUnitRole {
             | ModelUnitRole::MappingTree
             | ModelUnitRole::TreeTable
             | ModelUnitRole::DeletedInodesRoot
-            | ModelUnitRole::TombstoneRoot => NODE_BYTES / SLOT_BYTES,
+            | ModelUnitRole::TombstoneRoot
+            | ModelUnitRole::RetirementRoot => NODE_BYTES / SLOT_BYTES,
         }
     }
 }
@@ -171,7 +178,8 @@ enum ModelPublishKind {
     DeletionBatchKeepingDataUnits,
     /// 最后一批（D8（核心索引结构） 已定项 16 第 2 条）：剩下的单元不多于产品批大小 144 时意图之后（或上一批之后）的下一步——释放剩下的单元、
     /// 删 inode 记录（树里只有 inode 1，叶容器一片都不剩）、删 `deleted_inodes` 那条 key（删空取根指针置零、不写它的节点），
-    /// 同一次发布；重写 extent 零条目根与 inode 零条目根，加固定点单元。
+    /// 同一次发布；重写 extent 零条目根与 inode 零条目根，加固定点单元。唯一那片叶容器身份消失，退役树开放容器追加一条退役记录、
+    /// 连根重写（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 3 条）。
     FinalDeletionBatch,
     /// 建一个 size 0 的 inode（`transaction::publish_new_inodes`，count 1；D8（核心索引结构） 已定项 16 第 5 条：删完之后只许建 inode、
     /// 写行与空发布），新记录落在最左那片叶容器里（删完之后建的前 233 条，[`inode_leaf_container_position_of_a_new_record`]）：
@@ -217,6 +225,25 @@ pub fn instance_table_pages_for_rows(rows: usize) -> u64 {
 }
 
 impl ModelPublishKind {
+    /// 这次发布让 inode 号水位变了、给记账留一级台阶行（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）：
+    /// 第一个文件那次（inode 1 落下，水位从没有记账树到 2）与建 inode 那两种（每次加 1）。别的发布不发号；管理员回退那次的水位
+    /// 取环里没被崩溃恢复抛弃的根的最大值，模型罩的历史里那就是现行那一版的（D23（journal 的角色与格式） 已定项 14「水位」那一格），不留新台阶。
+    fn leaves_an_inode_number_watermark_step(self) -> bool {
+        match self {
+            ModelPublishKind::FirstFileVersion
+            | ModelPublishKind::NewInode
+            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => true,
+            ModelPublishKind::OverwriteFileVersion
+            | ModelPublishKind::RowsOnFileVersion
+            | ModelPublishKind::EmptyOnFileVersion
+            | ModelPublishKind::RowsOnVersionWithoutFile
+            | ModelPublishKind::ZeroUnit
+            | ModelPublishKind::UnlinkIntent
+            | ModelPublishKind::DeletionBatchKeepingDataUnits
+            | ModelPublishKind::FinalDeletionBatch => false,
+        }
+    }
+
     /// 这次发布重写不重写实例表：写行那两种重写整条链，别的照抄。
     fn rewrites_the_instance_table(self) -> bool {
         match self {
@@ -357,6 +384,8 @@ impl ModelPublishKind {
                 ModelUnitRole::InodeRoot,
                 ModelUnitRole::AllocationTree,
                 ModelUnitRole::AccountingTree,
+                ModelUnitRole::RetirementLeaf,
+                ModelUnitRole::RetirementRoot,
                 ModelUnitRole::MappingTree,
                 ModelUnitRole::TreeTable,
             ],
@@ -511,6 +540,18 @@ pub struct ModelRoot {
     /// 这一版中央映射树节点数的上界（[`central_mapping_tree_nodes_upper_bound`] 沿来路带下来，每盘一个节点占一个槽）；
     /// 树表 0 条的一版没有这棵树，记 0。
     pub central_mapping_tree_nodes_upper_bound: u64,
+    /// 这一版记账里 inode 号水位那一统计量的台阶行有几行（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）：
+    /// 水位变了的每次发布留一级，之后每次写记账行的发布照抄（管理员回退那次照抄现行那一版的，不取回退目标的）。
+    /// 模型不记水位的值，按发布的种类认水位变没变（[`ModelPublishKind::leaves_an_inode_number_watermark_step`]）。
+    /// 没写过记账树的一版记 0。
+    pub inode_number_watermark_steps: u64,
+    /// 这一版记账树的行数：写它的那次发布写的池级三行、每块盘六行与照抄的台阶行（[`IdealModel::accounting_rows_of_a_publish_on`]）；
+    /// 没写过记账树的一版记 0。
+    pub accounting_rows: u64,
+    /// 这一版记账树节点数的上界（[`accounting_tree_nodes_upper_bound`] 沿来路带下来，每盘一个节点占一个槽）：台阶行多到一片叶装不下时
+    /// 记账树分裂、长出根之下的节点（D8（核心索引结构） 已定项 11）。根之下的节点不对应角色，模型不记 key、不记是哪几个（同中央映射树）；
+    /// 写记账树的发布按它带占槽、按它数进映射条目。没写过记账树的一版记 0。
+    pub accounting_tree_nodes_upper_bound: u64,
     /// 写出这条根的那次发布的记录里最大的事务号：写文件内容的发布（第一个文件、覆盖写、顺序写）一事务一单元（D16（发布语义） 已定项 5）
     /// ⇒ 内容切成几个数据单元就承载几个事务、接着实例内计数往下取，这里记最后那一个号；删除类发布与建 inode 各一个事务；
     /// 空发布、写行、暖机、抬 F、回退的记录事务号 0（D23（journal 的角色与格式） 已定项 19 ①），这里记 0。
@@ -688,18 +729,83 @@ pub fn central_mapping_tree_nodes_upper_bound(nodes_before: u64, entries: u64) -
     }
 }
 
+/// 记账树一片叶装几行：(节点 − 头) ÷ 记账条目宽（D8（核心索引结构） 已定项 11；条目 34，D5（快照 / 空间记账机制） 已定项 5）。
+fn accounting_tree_leaf_entry_capacity() -> u64 {
+    index_node_entry_capacity(ACCOUNTING_KEY_BYTES, ACCOUNTING_ENTRY_BYTES)
+}
+
+/// 记账树一个内部节点装几个孩子：内部条目 = 本树 key + 子指针 86（D8（核心索引结构） 已定项 11）。
+fn accounting_tree_internal_child_capacity() -> u64 {
+    index_node_entry_capacity(
+        ACCOUNTING_KEY_BYTES,
+        ACCOUNTING_KEY_BYTES + NODE_POINTER_BYTES,
+    )
+}
+
+/// 记账树一次写记账行的发布之后至多几个节点（D8（核心索引结构） 已定项 11 ②③ 与「一次发布里先删后插」）。
+///
+/// 节点数不是行数的函数（收缩只摘空节点、不合并，形状随来路变）；模型不记 key、不照抄实现的分裂（D13（验证路线） 已定项 5），
+/// 同中央映射树沿来路带一个上界（[`central_mapping_tree_nodes_upper_bound`]），这里多数一半：
+/// - 先删：只摘空节点、根只剩一个孩子时降高，节点数不增；
+/// - 后插：设叶切了 s 次、内部节点切了 t 次、根切了 r 次（切开的根上面再长一个新根）。每切一次多一个节点，根切再多一个：
+///   N ≤ N_前 + s + t + r。
+/// - 切开的两半各至少 ⌊(容量 + 1) ÷ 2⌋（叶 239 行、内部 75 个孩子，[`smallest_half_of_a_split`]），之后这一段只插不删，到这次结束仍至少这么多。
+///   每切一次，这次结束时「切出来的」叶至少多一片（切原有的叶多两片、切切出来的叶多一片），s ≥ 1 时至少 s + 1 片、彼此不相交：
+///   (s + 1) × 239 ≤ 这次之后的行数 R。内部节点同一条：(t + 1) × 75 ≤ N − 1（根之外每个节点恰是一个内部节点的孩子）。
+///   没有叶切就没有内部节点切（内部节点只在孩子切开时多一个孩子）：s = 0 ⇒ t = 0。
+/// - 根切：根兼叶切开那一次是叶切，之后的根都是内部节点、它切开算在 t 里：r ≤ min(s + t, 1 + t)。
+///
+/// ⇒ 从一个节点长到 R = 478 行（两块盘 463 级台阶，D5 已定项 2 射程）：s ≤ 1、t = 0、r ≤ 1，N ≤ 3，恰是实现长成的根加两片叶。
+/// 右边随 N 涨得比 N 慢，满足它的 N 有个最大值：从 2 (N_前 + 2 s + 1)（那里已不满足）往下迭代，每一步都仍是 N 的上界，停在不动点。
+///
+/// `nodes_before`：上一版这棵树节点数的上界；上一版没有记账树时取 1（实现的规划从一片空的根兼叶起插）。
+///
+/// # Panics
+/// `nodes_before` 为 0：一棵有行的树至少一个节点，调用方给的是上一版的上界或 1。
+#[must_use]
+pub fn accounting_tree_nodes_upper_bound(nodes_before: u64, rows: u64) -> u64 {
+    assert!(
+        nodes_before >= 1,
+        "上一版的树至少一个节点（从空树建起也从一片空的根兼叶起插）"
+    );
+    let leaf_splits_bound =
+        (rows / smallest_half_of_a_split(accounting_tree_leaf_entry_capacity())).saturating_sub(1);
+    let nodes_after_the_splits_bound = |nodes: u64| {
+        let internal_splits_bound = if leaf_splits_bound == 0 {
+            0
+        } else {
+            ((nodes - 1) / smallest_half_of_a_split(accounting_tree_internal_child_capacity()))
+                .saturating_sub(1)
+        };
+        let root_splits_bound =
+            (leaf_splits_bound + internal_splits_bound).min(1 + internal_splits_bound);
+        nodes_before + leaf_splits_bound + internal_splits_bound + root_splits_bound
+    };
+    let mut bound = 2 * (nodes_before + 2 * leaf_splits_bound + 1);
+    // 迭代上界：bound 每一轮严格变小，不小于 nodes_before。跨轮带的是「这次之后的节点数不超过 bound」。
+    loop {
+        let next = nodes_after_the_splits_bound(bound);
+        if next >= bound {
+            return bound;
+        }
+        bound = next;
+    }
+}
+
 /// 一版的中央映射树里除分配记录树的节点之外装几条条目的上界（D19（块指针的结构与宽度预算） 已定项 8：进映射的是数据单元、extent 树每个节点、
 /// inode 叶容器与根、分配记录树每个节点、记账树每个节点、`deleted_inodes` 根、墓碑树与谱系树的容器与索引节点；
-/// 映射树自己的节点、树表、实例表豁免）。分配记录树的节点数由调用方按它的上界另加。
+/// 映射树自己的节点、树表、实例表豁免）。分配记录树的节点数由调用方按它的上界另加；记账树的节点按调用方给的这一版的上界
+/// （`accounting_tree_nodes`，[`accounting_tree_nodes_upper_bound`]）数。
 ///
 /// 谱系树模型不记（管理员回退往它上面追加一条，D18（块里携带什么信息） 已定项 24 第 5 条）：一条时间线上每次发布至多是一次回退，
-/// 回退至多 txg 次，一片容器装 2039 条，加根；没回退过的一版没有这棵树，多算的两条只让上界松一点。
+/// 回退至多 txg 次，一片容器装 1359 条，加根；没回退过的一版没有这棵树，多算的两条只让上界松一点。
 ///
 /// # Panics
-/// 谱系容器的上界多于一片：模型罩的历史一条时间线上发布不到 2039 次，谱系树根之下不长层。
+/// 谱系容器的上界多于一片：模型罩的历史一条时间线上发布不到 1359 次，谱系树根之下不长层。
 fn central_mapping_entries_other_than_the_allocation_record_tree(
     contents: &ModelVersionContents,
     role_written_at: &BTreeMap<ModelUnitRole, ModelCheckpointTxg>,
+    accounting_tree_nodes: u64,
     checkpoint_txg: ModelCheckpointTxg,
 ) -> u64 {
     let data_units = match contents {
@@ -717,15 +823,18 @@ fn central_mapping_entries_other_than_the_allocation_record_tree(
     let entries_of_one_node_roles: u64 = role_written_at
         .keys()
         .map(|role| match role {
-            // 模型罩的一版里这几个角色各一个单元（上段只有根兼叶、每片 inode 叶容器各是一个角色、记账树一个节点、墓碑树一片容器加根）。
+            // 记账树：根与根之下的每个节点各一条（台阶行多到一片叶装不下时长层），按这一版节点数的上界数。
+            ModelUnitRole::AccountingTree => accounting_tree_nodes,
+            // 模型罩的一版里这几个角色各一个单元（上段只有根兼叶、每片 inode 叶容器各是一个角色、墓碑树一片容器加根）。
             ModelUnitRole::ExtentRoot
             | ModelUnitRole::InodeLeaf
             | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
             | ModelUnitRole::InodeRoot
-            | ModelUnitRole::AccountingTree
             | ModelUnitRole::DeletedInodesRoot
             | ModelUnitRole::TombstoneLeaf
-            | ModelUnitRole::TombstoneRoot => 1,
+            | ModelUnitRole::TombstoneRoot
+            | ModelUnitRole::RetirementLeaf
+            | ModelUnitRole::RetirementRoot => 1,
             // 数据单元按这一版的单元数另数，分配记录树的节点由调用方按上界另加。
             ModelUnitRole::Data | ModelUnitRole::AllocationTree => 0,
             // 豁免映射（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
@@ -737,7 +846,7 @@ fn central_mapping_entries_other_than_the_allocation_record_tree(
     let lineage_containers = checkpoint_txg.0.div_ceil(LINEAGE_CONTAINER_RECORDS);
     assert!(
         lineage_containers <= 1,
-        "模型罩的历史一条时间线上发布不到 2039 次（谱系树至多一片容器）：txg {}",
+        "模型罩的历史一条时间线上发布不到 1359 次（谱系树至多一片容器）：txg {}",
         checkpoint_txg.0
     );
     let lineage_root = u64::from(lineage_containers > 0);
@@ -1269,6 +1378,9 @@ impl IdealModel {
             occupied_slots_upper_bound_per_device: ModelUnitRole::InstanceTable.span_in_slots()
                 + ModelUnitRole::TreeTable.span_in_slots(),
             central_mapping_tree_nodes_upper_bound: 0,
+            inode_number_watermark_steps: 0,
+            accounting_rows: 0,
+            accounting_tree_nodes_upper_bound: 0,
             highest_transaction_number_of_its_publish: 0,
             highest_transaction_number_in_its_instance: 0,
         };
@@ -1550,11 +1662,28 @@ impl IdealModel {
         }
     }
 
-    /// 一次写记账行的发布要几行、一个节点装几行（D5（快照 / 空间记账机制） 已定项 8）。
-    fn accounting_rows_exceed_one_node(&self) -> bool {
+    /// 接在 `previous` 后面、写记账行的一次发布写几行：池级三行加每块盘六行（D5（快照 / 空间记账机制） 已定项 8），加照抄的
+    /// inode 号水位台阶行（D5 已定项 2 射程「inode 号水位的台阶行」）——这次自己那一行是不是新的一级台阶不改行数（它就是池级那一行）。
+    fn accounting_rows_of_a_publish_on(&self, previous: &ModelRoot) -> u64 {
         let device_count = u64::try_from(self.geometry.devices.len()).expect("盘数");
-        POOL_WIDE_ACCOUNTING_ROWS + ACCOUNTING_ROWS_PER_DEVICE * device_count
-            > index_node_entry_capacity(ACCOUNTING_KEY_BYTES, ACCOUNTING_ENTRY_BYTES)
+        POOL_WIDE_ACCOUNTING_ROWS
+            + ACCOUNTING_ROWS_PER_DEVICE * device_count
+            + previous.inode_number_watermark_steps
+    }
+
+    /// 接在可写会话的现行版本后面、写记账行的下一次发布写几行（[`IdealModel::accounting_rows_of_a_publish_on`]）；没有可写会话时 None。
+    #[must_use]
+    pub fn accounting_rows_of_the_next_publish_in_the_session(&self) -> Option<u64> {
+        self.session
+            .as_ref()
+            .map(|session| self.accounting_rows_of_a_publish_on(&session.current))
+    }
+
+    /// 可写会话的现行版本（接下来的发布要接在它后面的那一版）；没有可写会话时 None。对拍的用例拿它的记账行数与记账树节点数的上界
+    /// 看模型在哪一步判记账树长一层、那次占几个槽。
+    #[must_use]
+    pub fn current_version_in_the_session(&self) -> Option<&ModelRoot> {
+        self.session.as_ref().map(|session| &session.current)
     }
 
     /// 单元区每块盘的槽数。
@@ -1699,7 +1828,7 @@ impl IdealModel {
                 contents.clone_from(&target_root.contents);
                 // 新根的 inode 树、extent 树与 `deleted_inodes` 取回退目标那一版的（D23（journal 的角色与格式） 已定项 14）：
                 // 目标那一版有的角色取它的分配代，没有的（删完的一版没有数据单元、叶容器与 `deleted_inodes` 根）这一版也没有；
-                // 墓碑树随现行那一版。
+                // 墓碑树与退役树随现行那一版。
                 for role in [
                     ModelUnitRole::Data,
                     ModelUnitRole::ExtentRoot,
@@ -1731,6 +1860,28 @@ impl IdealModel {
                 );
             }
         }
+        // 记账行：写记账树的发布照抄上一版的台阶行（回退那次的 `previous` 是现行那一版，台阶从它带），水位变了的再留一级；
+        // 不写记账树的发布（树表 0 条的一版上的零单元发布）这一版没有记账树。记账树节点数的上界按这次写的行数从上一版的上界往下带
+        // （回退那次实现从现行那一版的形状规划，`previous` 就是它）。
+        let (inode_number_watermark_steps, accounting_rows, accounting_tree_nodes) =
+            if rewritten.contains(&ModelUnitRole::AccountingTree) {
+                let accounting_rows = self.accounting_rows_of_a_publish_on(previous);
+                (
+                    previous.inode_number_watermark_steps
+                        + u64::from(kind.leaves_an_inode_number_watermark_step()),
+                    accounting_rows,
+                    accounting_tree_nodes_upper_bound(
+                        previous.accounting_tree_nodes_upper_bound.max(1),
+                        accounting_rows,
+                    ),
+                )
+            } else {
+                (
+                    previous.inode_number_watermark_steps,
+                    previous.accounting_rows,
+                    previous.accounting_tree_nodes_upper_bound,
+                )
+            };
         let mut slots_written = 0;
         let mut rewrites_the_allocation_record_tree = false;
         for role in &rewritten {
@@ -1740,18 +1891,23 @@ impl IdealModel {
                 ModelUnitRole::Data => {
                     slots_written += role.span_in_slots() * data_units_of_the_new_content;
                 }
+                // 记账树一次至多重写这一版的全部节点（根与根之下的，每个节点每盘占一个槽）：根之下的节点不对应角色，按上界算。
+                ModelUnitRole::AccountingTree => {
+                    slots_written += role.span_in_slots() * accounting_tree_nodes;
+                }
                 // 中央映射树在这里只记它的根：根之下的节点数按上界另算（下面）。
                 ModelUnitRole::ExtentRoot
                 | ModelUnitRole::InodeLeaf
                 | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
                 | ModelUnitRole::InodeRoot
-                | ModelUnitRole::AccountingTree
                 | ModelUnitRole::MappingTree
                 | ModelUnitRole::TreeTable
                 | ModelUnitRole::InstanceTable
                 | ModelUnitRole::DeletedInodesRoot
                 | ModelUnitRole::TombstoneLeaf
-                | ModelUnitRole::TombstoneRoot => slots_written += role.span_in_slots(),
+                | ModelUnitRole::TombstoneRoot
+                | ModelUnitRole::RetirementLeaf
+                | ModelUnitRole::RetirementRoot => slots_written += role.span_in_slots(),
             }
         }
         // extent 树下段的节点不对应角色（同分配记录树根之下的节点：分配代与位置归池级 checker 判），占槽按条款现算：
@@ -1797,6 +1953,7 @@ impl IdealModel {
                     central_mapping_entries_other_than_the_allocation_record_tree(
                         &contents,
                         &role_written_at,
+                        accounting_tree_nodes,
                         checkpoint_txg,
                     ),
                     previous.occupied_slots_upper_bound_per_device + slots_written,
@@ -1837,6 +1994,9 @@ impl IdealModel {
             occupied_slots_upper_bound_per_device: previous.occupied_slots_upper_bound_per_device
                 + slots_written,
             central_mapping_tree_nodes_upper_bound,
+            inode_number_watermark_steps,
+            accounting_rows,
+            accounting_tree_nodes_upper_bound: accounting_tree_nodes,
             highest_transaction_number_of_its_publish,
             highest_transaction_number_in_its_instance:
                 highest_transaction_number_before_this_publish
@@ -2204,17 +2364,6 @@ impl IdealModel {
         permitted_refusals: BTreeSet<ModelRefusalReason>,
         roots: Vec<ModelRoot>,
     ) -> ModelAnswer {
-        // 这一步里有一次发布重写了记账树（零单元发布不写记账行，别的都写，D16（发布语义） 已定项 9）。
-        let writes_accounting_rows = roots.iter().any(|root| {
-            root.role_written_at.get(&ModelUnitRole::AccountingTree)
-                == Some(&root.key.checkpoint_txg)
-        });
-        // 记账行装不下一个节点时记账树分裂、不拒（D8（核心索引结构） 已定项 11），分裂多出来的节点每个各占一个槽；
-        // 模型的占槽上界按「记账树一个节点」数，只罩行数装得进一个节点的池（盘数不到 80）——超出就是模型没罩到，不是实现错。
-        assert!(
-            !(writes_accounting_rows && self.accounting_rows_exceed_one_node()),
-            "模型今天只罩记账树一个节点的池：分裂多出来的节点模型的占槽上界没算"
-        );
         ModelAnswer {
             operation,
             required_refusals,
@@ -4255,6 +4404,98 @@ mod tests {
         );
     }
 
+    /// 记账树节点数的上界（D8（核心索引结构） 已定项 11：叶装 477 行、内部节点装 150 个孩子，装不下从中间切，切开的两半各至少 239 行 / 75 个孩子；
+    /// 收缩只摘空节点）：不到 2 × 239 = 478 行一次都切不了，上界照上一版的；从一个节点长到 478 行至多切一片叶、根切一次，3 个节点；
+    /// 717 = 3 × 239 行至多切两片叶，4 个；76 × 239 行 76 片叶挂一个根，77 个；151 × 239 行 151 片叶一个根装不下，根切一次、长成两个内部节点加新根，154 个。
+    /// 上一版长过的节点不往回收（3 个节点、477 行仍是 3），长过之后每多切一片叶多两个（叶一个、根一个的上界）。
+    #[test]
+    fn accounting_tree_nodes_grow_to_three_at_478_rows_and_keep_the_previous_bound() {
+        assert_eq!(
+            (
+                accounting_tree_leaf_entry_capacity(),
+                accounting_tree_internal_child_capacity(),
+                smallest_half_of_a_split(accounting_tree_leaf_entry_capacity()),
+                smallest_half_of_a_split(accounting_tree_internal_child_capacity()),
+            ),
+            (477, 150, 239, 75)
+        );
+        assert_eq!(
+            [
+                (1, 15),
+                (1, 477),
+                (1, 478),
+                (1, 3 * 239),
+                (1, 76 * 239),
+                (1, 151 * 239),
+                (3, 477),
+                (3, 479),
+            ]
+            .map(|(nodes_before, rows)| accounting_tree_nodes_upper_bound(nodes_before, rows)),
+            [1, 1, 3, 4, 77, 154, 3, 5],
+            "478 行：叶切 1 次、根切 1 次；151 × 239 行：叶切 150 次、内部节点切 1 次（2 × 75 ≤ 153）、根切 2 次之内取 1 + t = 2"
+        );
+    }
+
+    /// 两块 4 GiB 盘：第一个文件、unlink、最后一批之后一次建一个 inode，第 462 个那一版写 477 行、记账树一个节点；第 463 个那次照抄 463 级台阶、
+    /// 写 478 行，记账树长成根加两片叶（上界 3）。这次占槽上界：分配记录树与映射树之外写 8 槽（新记录落的第 2 片 inode 叶容器 2、inode 根 1、
+    /// 记账树 3 个节点各 1、映射树的根 1、树表 1），加映射树根之下的节点与分配记录树按上界算的节点。
+    #[test]
+    fn the_inode_creation_whose_watermark_step_crosses_one_accounting_leaf_counts_the_nodes_below_the_root_in_the_occupied_slots(
+    ) {
+        let mut model = two_device_model();
+        model.acquire_and_warm_up_in_the_make_filesystem_process();
+        let first = model
+            .answer_publish_first_file(&[3])
+            .expect("会话开着、现行 txg 2");
+        succeed(&mut model, &first);
+        let intent = model
+            .answer_unlink_intent()
+            .expect("会话开着、现行版本带活文件");
+        succeed(&mut model, &intent);
+        let final_batch = model
+            .answer_deletion_step()
+            .expect("会话开着、现行版本删除中");
+        succeed(&mut model, &final_batch);
+        for _ in 0..462 {
+            let created = model
+                .answer_create_inode()
+                .expect("会话开着、第一个文件已删完");
+            succeed(&mut model, &created);
+        }
+        let before = model.newest_root().clone();
+        let created = model
+            .answer_create_inode()
+            .expect("会话开着、第一个文件已删完");
+        assert!(
+            created.required_refusals.is_empty(),
+            "{:?}",
+            created.required_refusals
+        );
+        let written = &created.expected_roots[0];
+        let slots_outside_the_two_trees = 2 + 1 + 3 + 1 + 1;
+        let mapping_nodes_below_the_root = written.central_mapping_tree_nodes_upper_bound - 1;
+        assert_eq!(
+            (
+                (before.accounting_rows, before.accounting_tree_nodes_upper_bound),
+                (written.accounting_rows, written.accounting_tree_nodes_upper_bound),
+                written.occupied_slots_upper_bound_per_device
+                    - before.occupied_slots_upper_bound_per_device,
+            ),
+            (
+                (477, 1),
+                (478, 3),
+                slots_outside_the_two_trees
+                    + mapping_nodes_below_the_root
+                    + model.allocation_record_tree_nodes_rewritten_upper_bound(
+                        before.occupied_slots_upper_bound_per_device
+                            + slots_outside_the_two_trees
+                            + mapping_nodes_below_the_root
+                    ),
+            ),
+            "第 462 个那一版 477 行一个节点；第 463 个那次 478 行、根加两片叶，三个节点各占一槽进这次的占槽上界"
+        );
+    }
+
     /// 一个码 2 根装 135 片叶容器（D8（核心索引结构） 已定项 6）：删完之后建的第 233 × 135 − 1 条（从 0 数）落在最后那片（第 134 片），
     /// 第 233 × 135 条要第 136 片、根要长一层，模型的落点式子在那里 panic（判决 `research/prompts/m3-formal-code-b25-r1-main-verification.md`
     /// K1 第四行：保留 `assert!`、不答拒；core 在同一条拒，`MoreLeafContainersThanOneRootNodeHolds`）。
diff --git a/crates/singlefs-harness/src/model_comparison.rs b/crates/singlefs-harness/src/model_comparison.rs
index c123012..bc78d82 100644
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -21,6 +21,7 @@ use singlefs_core::mount::{
 use singlefs_core::pointer::LocationEntry;
 use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
 use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
+use singlefs_core::retirement_tree::RetirementContainerIndexInTree;
 use singlefs_core::root_record::RootRecord;
 use singlefs_core::root_ring::RootRingSlot;
 use singlefs_core::tombstone_tree::TombstoneContainerIndexInTree;
@@ -82,11 +83,11 @@ pub fn model_unit_role(unit: TransactionUnit) -> Option<ModelUnitRole> {
         TransactionUnit::InodeRoot => ModelUnitRole::InodeRoot,
         TransactionUnit::AllocationTree => ModelUnitRole::AllocationTree,
         TransactionUnit::AllocationTreeNodeBelowTheRoot(_) => return None,
-        // 模型罩的池两块盘、15 行记账：记账树恒只有一个节点（根兼叶），根之下的节点只在压小容量的
-        // 只供测试的开关下、或记账行装不下一个节点的池（80 块盘起）上出现，随机历史与层 0 的固定脚本都不装那个开关。
-        TransactionUnit::AccountingTreeNodeBelowTheRoot(position) => {
-            panic!("模型今天只罩记账树一个节点的池：{position:?}")
-        }
+        // 记账树根之下的节点不对应角色（同中央映射树根之下的节点）：inode 号水位的台阶行多到一片叶装不下时（两块盘第 463 级台阶起，
+        // D5（快照 / 空间记账机制） 已定项 2 射程）记账树从中间切开、长出根之下的节点（D8（核心索引结构） 已定项 11）；哪几片这次重写由
+        // 这次删插了哪些 key 定，模型不记 key、不记是哪几片。它们的分配代、位置与记录归池级 checker 判，占槽与映射条目模型沿来路带
+        // 节点数的上界（`model::accounting_tree_nodes_upper_bound`）。
+        TransactionUnit::AccountingTreeNodeBelowTheRoot(_) => return None,
         TransactionUnit::AccountingTree => ModelUnitRole::AccountingTree,
         // 中央映射树根之下的节点不对应角色（同分配记录树根之下的节点）：每个数据单元一条映射条目，数据单元加别的进映射的单元
         // 多于一片叶装的 294 条时（随机历史里是跨三片下段叶的那一档），映射树从中间切开、长出根之下的节点（D8（核心索引结构） 已定项 11）；
@@ -115,6 +116,20 @@ pub fn model_unit_role(unit: TransactionUnit) -> Option<ModelUnitRole> {
         TransactionUnit::LineageLeafContainer(_)
         | TransactionUnit::LineageRoot
         | TransactionUnit::LineageNodeBelowTheRoot(_) => return None,
+        // 退役树（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条，照墓碑树）：模型罩的历史里只有删第一个文件那次最后一批
+        // 让唯一那片叶容器身份消失，退役树恒只有容器 1 那一片与根（回退之后再删一次追加进同一片）。
+        TransactionUnit::RetirementLeafContainer(index) => {
+            assert_eq!(
+                index,
+                RetirementContainerIndexInTree(0),
+                "模型今天只罩退役树一片容器"
+            );
+            ModelUnitRole::RetirementLeaf
+        }
+        TransactionUnit::RetirementRoot => ModelUnitRole::RetirementRoot,
+        TransactionUnit::RetirementNodeBelowTheRoot(position) => {
+            panic!("模型今天只罩退役树一片容器（根之下没有节点）：{position:?}")
+        }
         // 模型把实例表链当一个角色：各片的落点与分配记录按这次之后的行数现算片数（`model::instance_table_pages_for_rows`）。
         TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
             ModelUnitRole::InstanceTable
@@ -230,6 +245,9 @@ fn instance_table_page_of_role(identity: TransactionUnit) -> Option<InstanceTabl
         | TransactionUnit::LineageLeafContainer(_)
         | TransactionUnit::LineageRoot
         | TransactionUnit::LineageNodeBelowTheRoot(_)
+        | TransactionUnit::RetirementLeafContainer(_)
+        | TransactionUnit::RetirementRoot
+        | TransactionUnit::RetirementNodeBelowTheRoot(_)
         | TransactionUnit::TreeTable => None,
     }
 }
@@ -680,7 +698,7 @@ pub fn refusal_reason_of_publish_error(error: &PublishError) -> ObservedRefusalR
         // 新记录恒追加在最右那片叶、满 233 条在末尾分裂；叶容器数到一个根装得下的 135 片之前（三万多次建 inode）不拒，
         // 它跑的那几种发布每次最多改一片叶容器、重写的角色最多九个。
         // 多层码 2 树算不出形状那一条同理：两棵树装不下一个节点时分裂、不拒，只有长到 257 层或上一版的形状按分隔 key 走不通才拒，
-        // 随机历史里两棵树恒只有一个节点；它出现就是模型与实现对不上。
+        // 随机历史里两棵树至多长一两层（记账树在建几百个 inode 之后、映射树在跨三片下段叶的文件上）；它出现就是模型与实现对不上。
         PublishError::MultiLevelCodeTwoTreeRefused { .. }
         | PublishError::InodeTreeWriteRefused(_)
         | PublishError::AllocationRecordTreeRewriteSetDidNotSettle { .. }
@@ -1460,7 +1478,9 @@ mod tests {
                 | ModelUnitRole::InstanceTable
                 | ModelUnitRole::DeletedInodesRoot
                 | ModelUnitRole::TombstoneLeaf
-                | ModelUnitRole::TombstoneRoot => {
+                | ModelUnitRole::TombstoneRoot
+                | ModelUnitRole::RetirementLeaf
+                | ModelUnitRole::RetirementRoot => {
                     panic!(
                         "第 {position_from_the_leftmost} 片 inode 叶容器映到了别的角色：{role:?}"
                     )
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
index 0c17252..f4ad637 100644
--- a/crates/singlefs-harness/src/mount_transparency.rs
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -29,7 +29,8 @@ use singlefs_core::pointer::NodePointer;
 use singlefs_core::records::{
     TreeTableEntry, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_DELETED_INODES,
     TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LIVELIST,
-    TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
+    TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_RETIREMENT, TREE_KIND_SPARSE_SIDE_TABLE,
+    TREE_KIND_TOMBSTONE,
 };
 use singlefs_core::recovery::{
     choose_root, choose_system_configuration, chosen_system_configuration_slot_bytes,
@@ -586,14 +587,17 @@ impl FloorRaisesInTheMountWrites {
     }
 }
 
-/// 树表里只由挂载续做改根的那五种树（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1：墓碑、谱系、
-/// deleted_inodes 三根并进「树表里 inode 树、extent 树的根指针」那一行，同一条续做规则）：extent、inode、deleted_inodes、墓碑树、谱系树。
-pub const TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES: [u16; 5] = [
+/// 树表里只由挂载续做改根的那六种树（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md` 第二节 Y1：墓碑、谱系、
+/// deleted_inodes 三根并进「树表里 inode 树、extent 树的根指针」那一行，同一条续做规则）：extent、inode、deleted_inodes、墓碑树、谱系树，
+/// 与退役树（续做的最后一批身份消失时追加，挂载本身不改它；判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条，
+/// 正推 K1.2 那一行）。
+pub const TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES: [u16; 6] = [
     TREE_KIND_EXTENT,
     TREE_KIND_INODE,
     TREE_KIND_DELETED_INODES,
     TREE_KIND_TOMBSTONE,
     TREE_KIND_LINEAGE,
+    TREE_KIND_RETIREMENT,
 ];
 
 /// 树表里根恒全零、挂载前后必须相同的那四种树（同上判决 Y1「树表种类 6、7、8、12 的根」那一行）：livelist、稀疏旁表、deadlist、logged_ops。
@@ -637,7 +641,7 @@ impl TreeTableEntryRootPointersByKind {
     }
 }
 
-/// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）每一步那一版树表里只由续做改根的那五种树
+/// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）每一步那一版树表里只由续做改根的那六种树
 /// （[`TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES`]）的根指针（用户 2026-10-03 定「续做算挂载的一部分」）：
 /// 挂载写的版本里，续做之前的那几版（写行、暖机、准入推的抬 F）照抄 R1 那一版的；续做的每一步改了它们，之后挂载写的版本
 /// （续做推的抬 F 的空发布）照抄最近一步的。挂载没续做时一步都没有，挂载写的每一版都比 R1 那一版。
@@ -730,7 +734,7 @@ pub enum MountTransparencyField {
     InstanceGeneration,
     InstanceTableRows,
     TreeIdentifierWatermark,
-    /// 树表条目的根指针：种类 1、2、9、10、11（[`TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES`]）照续做规则，
+    /// 树表条目的根指针：种类 1、2、9、10、11、13（[`TREE_KINDS_WHOSE_ROOTS_ONLY_THE_CONTINUATION_REWRITES`]）照续做规则，
     /// 种类 6、7、8、12（[`TREE_KINDS_WHOSE_ROOTS_THE_MOUNT_KEEPS`]）与 R1 相同，种类 3、4 不比。
     TreeTableEntryRootPointers,
     CheckpointTxg,
diff --git a/crates/singlefs-harness/src/scenario.rs b/crates/singlefs-harness/src/scenario.rs
index 2be4a94..9d0fb0f 100644
--- a/crates/singlefs-harness/src/scenario.rs
+++ b/crates/singlefs-harness/src/scenario.rs
@@ -263,6 +263,7 @@ pub fn deletion_history_switches() -> DeletionSwitches {
         batch_size: deletion_history_batch_size(),
         tombstone_container_capacity: DeletionSwitches::PRODUCT.tombstone_container_capacity,
         lineage_container_capacity: DeletionSwitches::PRODUCT.lineage_container_capacity,
+        retirement_container_capacity: DeletionSwitches::PRODUCT.retirement_container_capacity,
         container_tree_node_entries: DeletionSwitches::PRODUCT.container_tree_node_entries,
     }
 }
diff --git a/crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs b/crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs
index 7c96932..e29a100 100644
--- a/crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs
+++ b/crates/singlefs-harness/tests/a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs
@@ -57,6 +57,8 @@ fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
         ),
         tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
         lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container_capacity:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     }
 }
diff --git a/crates/singlefs-harness/tests/accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf.rs b/crates/singlefs-harness/tests/accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf.rs
new file mode 100644
index 0000000..3ba5af2
--- /dev/null
+++ b/crates/singlefs-harness/tests/accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf.rs
@@ -0,0 +1,311 @@
+//! inode 号水位的台阶行（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」，第 49 批 W3）多到一片记账树叶装不下的那次发布：
+//! 记账树从一个节点长成根加两片叶。两件事各一条用例：
+//! - ckpt_cost 记账树那一项（D28（挂载期承诺量） 已定项 4「记账树每发布的节点数」）按这次发布写完之后那棵树的节点数算，不是上一版的；
+//! - 理想模型数记账行时把台阶行算进去、带记账树节点数的上界，在台阶数跨叶的历史上与实现在同一步长一层、那次记账树占的槽数相同。
+//!
+//! 池是两块 4 GiB 盘：每次写记账行的发布写池级 3 行、每块盘 6 行，加照抄的台阶行；第一个文件那次留第 1 级台阶，之后每建一个 inode 再留一级，
+//! 第 k 个 inode 那次照抄 k 级、写 15 + k 行。
+
+mod common;
+
+use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
+use singlefs_core::address::InstanceGeneration;
+use singlefs_core::admission::{
+    accounting_tree_nodes_after_the_publish_built_on,
+    checkpoint_cost_of_the_version_to_build_on_with_node_capacities, MetadataBlocks,
+    SpaceAdmission,
+};
+use singlefs_core::code_two_tree::CodeTwoTreeNodeCapacity;
+use singlefs_core::transaction::{
+    publish_new_inodes, CodeTwoTreeNodeCapacities, MultiLevelCodeTwoTree, PoolWriter,
+    TransactionOutput, TransactionUnit,
+};
+use singlefs_format::SLOT_BYTES;
+use singlefs_harness::history::{
+    execute_history_with, AppliedEffect, GeneratedHistory, HistoryDeviceWidth, HistoryEnding,
+    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
+    StepOutcome, StepPosition,
+};
+use singlefs_harness::model::ModelUnitRole;
+use singlefs_harness::SharedStream;
+
+/// 两块盘每次写记账行的发布写的池级 3 行加每块盘 6 行。
+const ACCOUNTING_ROWS_WITHOUT_STEPS_ON_TWO_DEVICES: u64 = 15;
+
+/// 产品容量下记账树一片叶装的行数（(16384 − 头 159) ÷ 34）。
+const ACCOUNTING_ROWS_ONE_LEAF_HOLDS: u64 = 477;
+
+/// 产品容量两块盘时第一个装不下一片叶的那次：照抄 463 级台阶、写 478 行（D5 已定项 2 射程「第 463 行起 +2」）。
+const INODE_CREATIONS_UP_TO_THE_FIRST_SPLIT_OF_THE_ACCOUNTING_TREE: u64 =
+    ACCOUNTING_ROWS_ONE_LEAF_HOLDS - ACCOUNTING_ROWS_WITHOUT_STEPS_ON_TWO_DEVICES + 1;
+
+/// 在会话外经写入口建一个 size 0 的 inode（`publish_new_inodes`，count 1），节点容量按 `capacities` 装，接在 `pool.output` 后面。
+fn create_one_inode(
+    pool: &mut BuiltPool,
+    capacities: CodeTwoTreeNodeCapacities,
+) -> TransactionOutput {
+    let publish_parameters = parameters();
+    let previous = pool.output.clone();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    writer.set_code_two_tree_node_capacities(capacities);
+    let output = publish_new_inodes(
+        &mut writer,
+        &mut pool.allocator,
+        &previous,
+        1,
+        FIXED_WRITE_TIME_SECONDS + 60,
+        InstanceGeneration(1),
+    )
+    .expect("建空 inode");
+    pool.output = output.clone();
+    output
+}
+
+/// 一次发布重写的记账树节点数（根兼叶，或根与根之下的每个节点）。
+fn accounting_tree_nodes_written_by(output: &TransactionOutput) -> usize {
+    output
+        .rewritten
+        .iter()
+        .filter(|role| {
+            matches!(
+                role,
+                TransactionUnit::AccountingTree
+                    | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
+            )
+        })
+        .count()
+}
+
+/// 记账树叶压到 16 行、内部节点 4 个孩子（中央映射树照产品容量）：第一个文件那一版（txg 3）15 行、自己那一行是第 1 级台阶；
+/// 建一个 inode（txg 4）照抄 1 级、写 16 行，仍一个节点；再建一个（txg 5）照抄 2 级、写 17 行，一片叶装不下，长成根加两片叶 3 个节点、
+/// 三个都是这次写的。接在 txg 4 那一版后面的 ckpt_cost 记账树那一项按这次发布写完之后那棵树算 3，不小于 txg 5 那次实际写的 3 个；
+/// 整个 ckpt_cost 的绝对值钉住（分配记录树 529、中央映射树那一项、记账树 3、树表 1、谱系 3）。
+/// 判别力：ckpt_cost 的记账树那一项退回拿 txg 4 那一版的节点数 1 算时，红在绝对值那一句（540，少 2）；规划下一次发布的行时不照抄台阶行
+/// （只剩 15 行、一个节点），那一项算成 1，「不小于实际写的」那一句先红。
+#[test]
+fn the_checkpoint_cost_counts_the_accounting_nodes_after_the_publish_whose_watermark_step_crosses_one_leaf(
+) {
+    let capped = CodeTwoTreeNodeCapacities::CappedForTests {
+        accounting: CodeTwoTreeNodeCapacity {
+            leaf_entries: 16,
+            internal_entries: 4,
+        },
+        central_mapping: MultiLevelCodeTwoTree::CentralMapping.node_capacity_of_the_node_format(),
+    };
+    let mut pool =
+        build_pool("checkpoint-cost-counts-the-accounting-nodes-after-a-step-crosses-a-leaf");
+    let sixteen_rows = create_one_inode(&mut pool, capped);
+    let accounting_term_built_on_sixteen_rows = accounting_tree_nodes_after_the_publish_built_on(
+        &sixteen_rows,
+        capped.of_tree(MultiLevelCodeTwoTree::Accounting),
+    );
+    let checkpoint_cost_built_on_sixteen_rows =
+        checkpoint_cost_of_the_version_to_build_on_with_node_capacities(
+            Some(&sixteen_rows),
+            &pool.allocator,
+            capped,
+        );
+    let seventeen_rows = create_one_inode(&mut pool, capped);
+    assert_eq!(
+        (
+            sixteen_rows.accounting_entries.len(),
+            sixteen_rows.inode_number_watermark_steps().len(),
+            sixteen_rows.accounting_tree.node_count(),
+            seventeen_rows.accounting_entries.len(),
+            seventeen_rows.accounting_tree.node_count(),
+            accounting_tree_nodes_written_by(&seventeen_rows),
+        ),
+        (16, 2, 1, 17, 3, 3),
+        "txg 4 那一版 16 行（2 级台阶）一个节点；txg 5 那次 17 行，根加两片叶 3 个节点都是这次写的"
+    );
+    assert!(
+        accounting_term_built_on_sixteen_rows
+            >= u64::try_from(accounting_tree_nodes_written_by(&seventeen_rows))
+                .expect("节点数装得进 u64"),
+        "ckpt_cost 记账树那一项 {accounting_term_built_on_sixteen_rows} 不小于台阶跨叶那次实际写的记账树节点数 {}",
+        accounting_tree_nodes_written_by(&seventeen_rows)
+    );
+    assert_eq!(
+        (
+            accounting_term_built_on_sixteen_rows,
+            checkpoint_cost_built_on_sixteen_rows
+        ),
+        (
+            3,
+            MetadataBlocks(
+                CHECKPOINT_COST_BUILT_ON_SIXTEEN_ROWS_UNDER_A_SIXTEEN_ROW_ACCOUNTING_LEAF
+            )
+        ),
+        "接在 16 行那一版后面：记账树那一项 3、整个 ckpt_cost 的绝对值"
+    );
+}
+
+/// 副本上现跑的数（第 49 批 W3 补做）：529 + 中央映射树那一项 6 + 记账树 3 + 树表 1 + 谱系 3，第 50 批加退役 3 = 545；
+/// 记账树那一项拿上一版的节点数 1 算时是 543。
+const CHECKPOINT_COST_BUILT_ON_SIXTEEN_ROWS_UNDER_A_SIXTEEN_ROW_ACCOUNTING_LEAF: u64 = 545;
+
+/// 一次发布写的记账树单元（根兼叶，或根与根之下的每个节点）每盘占几个槽：按交回的单元字节数数。
+fn accounting_tree_slots_written_by(output: &TransactionOutput) -> u64 {
+    output
+        .units
+        .iter()
+        .filter(|unit| {
+            matches!(
+                unit.identity,
+                TransactionUnit::AccountingTree
+                    | TransactionUnit::AccountingTreeNodeBelowTheRoot(_)
+            ) && output.rewritten.contains(&unit.identity)
+        })
+        .map(|unit| u64::try_from(unit.bytes.len()).expect("单元字节数装得进 u64") / SLOT_BYTES)
+        .sum()
+}
+
+/// 产品容量两块盘、跨过第一次分裂之后再多建几个：对拍历史在分裂之后照常往下比。
+const INODE_CREATIONS_PAST_THE_FIRST_SPLIT: u64 = 2;
+
+/// 一版的记账树：行数、节点数（模型那一侧是上界）、写它的那次发布记账树每盘占几个槽。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct AccountingTreeOfAVersion {
+    rows: u64,
+    nodes: u64,
+    slots_written: u64,
+}
+
+/// 产品容量（记账树叶 477 行）、同一组台阶数，模型与实现在同一步判记账树长一层、那次记账树占的槽数相同：
+/// - 实现：第一个文件之后经写入口一次建一个 inode，第 462 个那次写 477 行、记账树一个节点占 1 槽；第 463 个那次写 478 行、
+///   长成根加两片叶，3 个节点都是这次写的、占 3 槽；
+/// - 模型：对拍历史（起点第一个文件，unlink 之后一次建一个 inode；unlink 不留台阶，第 k 个 inode 那次同样照抄 k 级）每一步都与实现对上、
+///   跑完不停；第 462 个那一版 477 行、记账树节点数上界 1，第 463 个那一版 478 行、上界 3（`model::accounting_tree_nodes_upper_bound`：
+///   一片叶切一次、根切一次），写记账树占 3 槽。分裂之后再建 2 个，对拍照常往下比（胶水不再在根之下的记账树节点上停）。
+///
+/// 每一步的池级 checker 不跑（`PerStepChecker::Skipped`）：这条用例判的是模型的行数、节点数与占槽，不判盘上。
+/// 判别力：模型数记账行时不算台阶行，第 463 个那一版模型 15 行、一个节点；多数一级，第 462 个那一版就判长层；
+/// 记账树节点数的上界不随行数长（照抄上一版的），第 463 个那一版模型 1 个节点、占 1 槽；切叶次数的上界不减一，第 462 个那一版就是 3 个节点。
+#[test]
+fn the_model_and_the_implementation_grow_the_accounting_tree_a_level_at_the_same_watermark_step() {
+    let inode_creations = INODE_CREATIONS_UP_TO_THE_FIRST_SPLIT_OF_THE_ACCOUNTING_TREE
+        + INODE_CREATIONS_PAST_THE_FIRST_SPLIT;
+    let mut pool = build_pool("model-and-implementation-grow-the-accounting-tree-at-the-same-step");
+    let mut implementation_after_each_creation = Vec::new();
+    for _ in 0..inode_creations {
+        let created = create_one_inode(&mut pool, CodeTwoTreeNodeCapacities::FromTheNodeFormat);
+        implementation_after_each_creation.push(AccountingTreeOfAVersion {
+            rows: u64::try_from(created.accounting_entries.len()).expect("行数装得进 u64"),
+            nodes: u64::try_from(created.accounting_tree.node_count()).expect("节点数装得进 u64"),
+            slots_written: accounting_tree_slots_written_by(&created),
+        });
+    }
+
+    let mut operations = vec![HistoryOperation::Unlink];
+    operations.extend(std::iter::repeat_n(
+        HistoryOperation::CreateInode,
+        usize::try_from(inode_creations).expect("几百装得进 usize"),
+    ));
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations,
+    };
+    let mut model_after_each_creation = Vec::new();
+    let run = execute_history_with(
+        &history,
+        HistoryExecution {
+            per_step_checker: PerStepChecker::Skipped,
+            device_width: HistoryDeviceWidth::FourGibibytes,
+            space_admission: SpaceAdmission::JudgedByTheFormula,
+        },
+        &SharedStream::new(),
+        &mut |observation| {
+            let is_an_inode_creation = matches!(
+                observation.position,
+                StepPosition::Operation(index) if index >= 1
+            );
+            if let (true, Some(current)) = (
+                is_an_inode_creation,
+                observation.model.current_version_in_the_session(),
+            ) {
+                model_after_each_creation.push(AccountingTreeOfAVersion {
+                    rows: current.accounting_rows,
+                    nodes: current.accounting_tree_nodes_upper_bound,
+                    slots_written: current.accounting_tree_nodes_upper_bound
+                        * ModelUnitRole::AccountingTree.span_in_slots(),
+                });
+            }
+        },
+    );
+    let published_steps = run
+        .outcomes
+        .iter()
+        .filter(|outcome| {
+            matches!(
+                outcome,
+                StepOutcome::Applied(AppliedEffect::Published { .. })
+            )
+        })
+        .count();
+    assert_eq!(
+        (
+            &run.ending,
+            published_steps,
+            model_after_each_creation.len()
+        ),
+        (
+            &HistoryEnding::Completed,
+            1 + usize::try_from(inode_creations).expect("几百装得进 usize"),
+            usize::try_from(inode_creations).expect("几百装得进 usize"),
+        ),
+        "模型：unlink 与每一次建 inode 都与实现对上，跨过记账树分裂照常跑完"
+    );
+
+    let first_split = INODE_CREATIONS_UP_TO_THE_FIRST_SPLIT_OF_THE_ACCOUNTING_TREE;
+    let at_creation = |after_each_creation: &[AccountingTreeOfAVersion], creation: u64| {
+        after_each_creation[usize::try_from(creation - 1).expect("几百装得进 usize")]
+    };
+    let one_node = AccountingTreeOfAVersion {
+        rows: ACCOUNTING_ROWS_ONE_LEAF_HOLDS,
+        nodes: 1,
+        slots_written: 1,
+    };
+    let root_and_two_leaves = AccountingTreeOfAVersion {
+        rows: ACCOUNTING_ROWS_ONE_LEAF_HOLDS + 1,
+        nodes: 3,
+        slots_written: 3,
+    };
+    assert_eq!(
+        (
+            at_creation(&implementation_after_each_creation, first_split - 1),
+            at_creation(&implementation_after_each_creation, first_split),
+        ),
+        (one_node, root_and_two_leaves),
+        "实现：第 462 个 inode 那次 477 行一个节点占 1 槽，第 463 个那次 478 行、根加两片叶都是这次写的、占 3 槽"
+    );
+    assert_eq!(
+        (
+            at_creation(&model_after_each_creation, first_split - 1),
+            at_creation(&model_after_each_creation, first_split),
+        ),
+        (
+            at_creation(&implementation_after_each_creation, first_split - 1),
+            at_creation(&implementation_after_each_creation, first_split),
+        ),
+        "模型：第 462、463 个 inode 那两版的行数、记账树节点数（上界）与写记账树占的槽数都与实现相同"
+    );
+    let first_growth = |after_each_creation: &[AccountingTreeOfAVersion]| {
+        after_each_creation
+            .iter()
+            .position(|tree| tree.nodes > 1)
+            .map(|index| index + 1)
+    };
+    assert_eq!(
+        (
+            first_growth(&model_after_each_creation),
+            first_growth(&implementation_after_each_creation)
+        ),
+        (
+            Some(usize::try_from(first_split).expect("几百装得进 usize")),
+            Some(usize::try_from(first_split).expect("几百装得进 usize"))
+        ),
+        "模型与实现都是第 463 个 inode 那次第一次长层"
+    );
+}
diff --git a/crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs b/crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs
index 585fc37..ff75fd1 100644
--- a/crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs
+++ b/crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs
@@ -124,7 +124,8 @@ fn capped(
 /// 产品路径的节点容量（按节点格式算），盘宽四档：分配记录树高 2（1 GiB）、3（4 GiB、64 GiB）、4（1 TiB），文件一个数据单元、中央映射树 1 层；
 /// 再加一格 4 GiB 两盘、文件顺序写到 300 个数据单元，中央映射树 2 层。
 /// ckpt_cost 加了谱系那一项（一次谱系容器 COW 连祖先：容器 2 + 根 1，谱系树零根，用户 2026-10-02 定「乙」）之后，这里与下一张表每格多扣的块数都多 3
-/// （空发布不写谱系树，实写的固定点不变）。
+/// （空发布不写谱系树，实写的固定点不变）。第 50 批 ckpt_cost 再加退役那一项（退役容器 2 + 根 1 + 退役树内部层数 0，判决
+/// `research/prompts/m3-retire-r1-main-verification.md` 第六节 R2），空发布不写退役树，每格再多 3。
 fn cells_with_the_node_format_capacities() -> Vec<PoolGeometryCell> {
     let one_data_unit = |name,
                          device_bytes,
@@ -147,28 +148,28 @@ fn cells_with_the_node_format_capacities() -> Vec<PoolGeometryCell> {
             GIBIBYTE,
             JOURNAL_RING_BYTES_OF_THE_ONE_GIBIBYTE_DEVICES,
             EMPTY_PUBLISHES_PER_CELL,
-            over_reserved(135, 137),
+            over_reserved(138, 140),
         ),
         one_data_unit(
             "two-4GiB",
             4 * GIBIBYTE,
             JOURNAL_RING_DEFAULT_BYTES,
             EMPTY_PUBLISHES_PER_CELL,
-            over_reserved(530, 532),
+            over_reserved(533, 535),
         ),
         one_data_unit(
             "two-64GiB",
             64 * GIBIBYTE,
             JOURNAL_RING_DEFAULT_BYTES,
             EMPTY_PUBLISHES_PER_CELL,
-            over_reserved(10340, 10342),
+            over_reserved(10343, 10345),
         ),
         one_data_unit(
             "two-1TiB",
             1024 * GIBIBYTE,
             JOURNAL_RING_DEFAULT_BYTES,
             EMPTY_PUBLISHES_OF_THE_ONE_TEBIBYTE_CELL,
-            over_reserved(167_301, 167_303),
+            over_reserved(167_304, 167_306),
         ),
         PoolGeometryCell {
             name: "two-4GiB-file-300-units-mapping-two-levels",
@@ -179,7 +180,7 @@ fn cells_with_the_node_format_capacities() -> Vec<PoolGeometryCell> {
                 DATA_UNITS_OF_THE_TWO_LEVEL_CENTRAL_MAPPING_CELL,
             ),
             empty_publishes: EMPTY_PUBLISHES_PER_CELL,
-            over_reserved_slots_of_the_empty_publishes: over_reserved(531, 533),
+            over_reserved_slots_of_the_empty_publishes: over_reserved(534, 536),
         },
     ]
 }
@@ -199,21 +200,23 @@ fn cells_with_capped_code_two_trees() -> Vec<PoolGeometryCell> {
         };
     // ckpt_cost 中央映射树那一项的删插数连谱系容器与谱系树根那 2 把一起算（用户 2026-10-02 定 Q-U「乙」）之后，压小的映射树上
     // 那一项多切几次：三格各多扣 1 / 3 / 1077 块左右（三·2 第 8 位实现员副本上现跑；之前是 880–885、1060–1070、144964–145506）。
+    // 第 50 批退役那一项（容器 2 + 根 1 + 内部层数，中央映射树删插数再加退役容器与根各一把）之后三格各多扣 4–5 / 4 / 1084–1086 块
+    // （第 50 批实现员副本上现跑；之前是 881–886、1063–1072、146041–146585）。
     vec![
         capped_cell(
             "two-4GiB-mapping-4-8",
             capped((477, 150), (4, 8)),
-            over_reserved(881, 886),
+            over_reserved(885, 891),
         ),
         capped_cell(
             "two-4GiB-mapping-3-3",
             capped((477, 150), (3, 3)),
-            over_reserved(1063, 1072),
+            over_reserved(1067, 1076),
         ),
         capped_cell(
             "two-4GiB-mapping-2-2-accounting-8-3",
             capped((8, 3), (2, 2)),
-            over_reserved(146_041, 146_585),
+            over_reserved(147_125, 147_671),
         ),
     ]
 }
@@ -446,6 +449,9 @@ fn measure(
             | TransactionUnit::TombstoneLeafContainer(_)
             | TransactionUnit::TombstoneRoot
             | TransactionUnit::TombstoneNodeBelowTheRoot(_)
+            | TransactionUnit::RetirementLeafContainer(_)
+            | TransactionUnit::RetirementRoot
+            | TransactionUnit::RetirementNodeBelowTheRoot(_)
             | TransactionUnit::InstanceTable
             | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                 measured.ordinary_allocation_slots += role.span_slots();
diff --git a/crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs b/crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs
index 162f71b..16c5af9 100644
--- a/crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs
+++ b/crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs
@@ -139,9 +139,11 @@ fn slots_of(output: &TransactionOutput, device: DeviceIdentity) -> BTreeSet<u64>
         .collect()
 }
 
-/// 记账标签 6（墓碑预付，D3（空间分配） 已定项 2）摊到每块盘扣的字节：一个活对象、墓碑树零根时
-/// 两份副本 × ((2 + 0 + 1) × 16384 + 41 × 1) = 98386，÷ 2 = 49193。准入读数从发布路径接上它起每块盘多扣这么多。
-const TAG_SIX_SHARE_OF_ONE_DEVICE_IN_BYTES: i128 = (3 * 16384 + 41) * 2 / 2;
+/// 记账标签 6（墓碑与退役预付，D3（空间分配） 已定项 2；退役那一份是判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条）
+/// 摊到每块盘扣的字节：一个活对象、一条记录、墓碑树与退役树零根时两份副本 × ((2 + 0 + 1 + 2 + 0 + 1) × 16384 + 41 × 1 + 35 × 1) = 196760，
+/// ÷ 2 = 98380（第 50 批之前 49193）。准入读数从发布路径接上它起每块盘多扣这么多。小盘的环窗口补足为退役那一份多补了 3 槽
+/// （`crates/singlefs-harness/src/history.rs` 的 `SMALLER_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS`），下面两格「可用 X 槽」各比第 50 批之前多 3。
+const TAG_SIX_SHARE_OF_ONE_DEVICE_IN_BYTES: i128 = (6 * 16384 + 41 + 35) * 2 / 2;
 
 /// 条款没给、由用例给的两个量（C363）。
 #[derive(Clone, Copy, Debug)]
@@ -475,7 +477,7 @@ fn image_of_the_devices(
 /// 可写挂载那一处（C363 (b) 判决第四节第 2 条接进来；D16（发布语义） 已定项 1「准入」那一行定了不够时怎么办，用户 2026-09-26 定）：
 /// 两块单元区 256 槽的小盘，第一个文件之后可写挂载、覆盖写 12 次的盘面上再可写挂载——取号之前按这次挂载的 rows0 算
 /// （D28（挂载期承诺量） 已定项 3），扣掉实例切换预留、checkpoint 保留池（已定项 4，按最坏情况计，用户 2026-09-27 定）与已分配之后，
-/// 两块盘的可用(d) 都是 −39 槽 < 0（A3b 把小盘改成 6 MiB 环、单元区从 1408 起之前是 −47 槽）：
+/// 两块盘的可用(d) 都是 −36 槽 < 0（第 50 批之前 −39；A3b 把小盘改成 6 MiB 环、单元区从 1408 起之前是 −47 槽）：
 /// 实例切换的预留拿不到。写行那次发布照走切换预留、写完行与暖机之后推抬 F 的空发布、再判：这里推一串（F 抬到上限 14，3 次空发布），
 /// 再判就够了，挂载做成（取号 3），交回 `MountSpaceAdmission::AdmittedAfterTheFloorRaises`，带着取号之前那一判的拒绝原样。
 /// 推的那一串的根都带新 F = 14、接在暖机后面，现行那一版就是最后一次；挂载之后的镜像池级 checker 0 违例；按可写挂载的读数再判，每块盘可用 ≥ 0。
@@ -513,13 +515,13 @@ fn writable_mount_short_of_its_instance_switch_reserve_raises_the_floor_after_th
                 .map(|device| DeviceShortOfDemand {
                     device,
                     available: AvailableBytesOnOneDevice(
-                        -39 * i128::from(SLOT_BYTES) - TAG_SIX_SHARE_OF_ONE_DEVICE_IN_BYTES,
+                        -36 * i128::from(SLOT_BYTES) - TAG_SIX_SHARE_OF_ONE_DEVICE_IN_BYTES,
                     ),
                     demand: BytesOnOneDevice::ZERO,
                 })
                 .collect(),
         },
-        "取号之前两块盘都短：可用 −39 槽再减标签 6 每盘那一份，挂载这一刻不另要需求"
+        "取号之前两块盘都短：可用 −36 槽再减标签 6 每盘那一份，挂载这一刻不另要需求"
     );
     assert_eq!(floor_raises.len(), 1, "推一串就够");
     let raised = &floor_raises[0];
@@ -595,7 +597,7 @@ fn writable_mount_short_of_its_instance_switch_reserve_raises_the_floor_after_th
 
 /// 发布路径那一处（C363 (b) 判决第四节第 2 条）：两块单元区 256 槽的小盘，第一个文件之后可写挂载、覆盖写 4 次的盘面上，进程重开、
 /// 可写挂载（取号之前就够），在这次挂载里直接调发布路径接连覆盖写三次（都放行），第四次：需求 = 这次发布新写的全部槽（D28（挂载期承诺量） 已定项 1 接线，
-/// 普通分配加固定点、不加换下的，用户 2026-09-25 定），每块盘 14 槽（普通分配 6、固定点 8），而两块盘的可用(d) 都只剩 11 槽
+/// 普通分配加固定点、不加换下的，用户 2026-09-25 定），每块盘 14 槽（普通分配 6、固定点 8），而两块盘的可用(d) 都只剩 14 槽再减标签 6 每盘那一份（第 50 批之前 11 槽，小盘补足多补了退役那 3 槽）
 /// （实例切换的预留按下一次挂载的 rows0，已定项 3；checkpoint 保留池与 c_max 按最坏情况计，已定项 4，用户 2026-09-27 定）——
 /// 在读盘核与动分配器之前返回 `PublishError::SpaceAdmissionRefused`：录制流一步没多、两块盘逐字节不变、`DiskSnapshot` 不变、
 /// 分配器与发布之前逐项相同。这一格只有把固定点算进需求才拒：普通分配只有 6 槽 ≤ 11。需求那 14 槽从同一次覆盖写真发出去的角色现数
@@ -718,12 +720,12 @@ fn an_overwrite_whose_new_slots_exceed_the_available_bytes_is_refused_by_the_spa
                 .map(|device| DeviceShortOfDemand {
                     device,
                     available: AvailableBytesOnOneDevice(
-                        11 * i128::from(SLOT_BYTES) - TAG_SIX_SHARE_OF_ONE_DEVICE_IN_BYTES,
+                        14 * i128::from(SLOT_BYTES) - TAG_SIX_SHARE_OF_ONE_DEVICE_IN_BYTES,
                     ),
                     demand: BytesOnOneDevice::of_slots(slots_written_by_the_overwrite),
                 })
                 .collect(),
         },
-        "两块盘都短：可用 11 槽再减标签 6 每盘那一份 < 需求 14 槽（普通分配的 6 槽装得下，固定点那 8 槽装不下）"
+        "两块盘都短：可用 14 槽再减标签 6 每盘那一份 < 需求 14 槽（普通分配的 6 槽装得下，固定点那 8 槽装不下）"
     );
 }
diff --git a/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs b/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
index eb62325..b26bce2 100644
--- a/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
+++ b/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
@@ -274,13 +274,18 @@ fn an_unlink_whose_last_batch_and_the_floor_raise_pushed_for_it_both_find_no_unb
 }
 
 /// 挂载续做那两条的窄盘：不为环窗口补足（准入关着造盘面，`start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off`）、
-/// 单元区 256 槽；第一个文件之后覆盖写 [`OVERWRITES_BEFORE_THE_CONTINUATION`] 次，再顺序写把它长到 [`DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION`] 个数据单元。
-/// 第十位实现员副本上的草稿探针（单元区 192 / 256 / 320 / 384 槽 × 覆盖写 0–12 次 × 长到 1–14 个单元 × 崩在意图之后或第一批之后，逐格现跑）：
-/// 256 槽、覆盖写 2 次、14 个单元那一格崩在意图之后是「取号之前那一判放行、续做里一批被落得下那一判拒、推一串之后做成」，
-/// 崩在第一批之后是「续做推不出空间、挂载交回错」。
-const UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION: u64 = 256;
-const OVERWRITES_BEFORE_THE_CONTINUATION: usize = 2;
-const DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION: usize = 14;
+/// 单元区 292 槽；第一个文件之后覆盖写 [`OVERWRITES_BEFORE_THE_CONTINUATION`] 次，再顺序写把它长到 [`DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION`] 个数据单元。
+/// 第十位实现员副本上的草稿探针（单元区 192 / 256 / 320 / 384 槽 × 覆盖写 0–12 次 × 长到 1–14 个单元 × 崩在意图之后或第一批之后，逐格现跑）定的是
+/// 256 槽、覆盖写 2 次、14 个单元；第 50 批退役树进准入之后挂载那一判每块盘多扣约 42 槽（标签 6 退役 3 槽、ckpt_cost 退役 3 块进保留池、切换预留跟着涨），
+/// 那一格崩在意图之后变成「取号之前就不够、先推一串」。第 50 批补做重扫（单元区 256–304 槽每 4 槽 × 覆盖写 0–4 次 × 长到 10–24 个单元 ×
+/// 崩在意图之后、第一批之后、第二批之后，逐格现跑）：292 槽、覆盖写 1 次、16 个单元那一格崩在意图之后是「取号之前那一判放行、续做里一批被落得下那一判拒、
+/// 推一串两次空发布之后走完」，崩在第一批之后是「取号之前放行、续做推不出空间、挂载照样做成可写」，不先覆盖写、崩在第二批之后是「续做走到空、一串都没推」——
+/// 三种场景同在一格的，256–304 槽里只有这一格。
+const UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES_FOR_THE_CONTINUATION: u64 = 292;
+const OVERWRITES_BEFORE_THE_CONTINUATION: usize = 1;
+const DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION: usize = 16;
+/// 单元区 192 槽那几条（准入与续做都推过抬 F）的文件单元数：它们的造形没随第 50 批改，仍是 14 个单元。
+const DATA_UNITS_OF_THE_FILE_ON_THE_192_SLOT_DEVICES: usize = 14;
 
 /// 删除批大小 1 个数据单元（只供测试的开关，挂载的开关链装上去，续做照它）。
 fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
@@ -290,6 +295,8 @@ fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
         ),
         tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
         lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container_capacity:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     }
 }
@@ -386,9 +393,9 @@ fn mount_after_the_crash_with_batches_of_one(
 }
 
 /// 挂载续做里删除的一批被落得下那一判拒、推一串抬 F、回收之后接着走（用户 2026-10-03 定案「挂载续做被空间拒：同样推抬 F 再试」，
-/// 与会话那一处同一个做法）：窄盘上 14 个单元的文件发完意图就崩（队列一条、单元都在），重开、准入判着、删除批大小 1。
+/// 与会话那一处同一个做法）：窄盘上 16 个单元的文件发完意图就崩（队列一条、单元都在），重开、准入判着、删除批大小 1。
 /// 取号之前那一判放行；写行与暖机之后续做，走到一半一批被落得下那一判拒（`SpaceAdmissionRefused`；删除类发布不判式子那一判），
-/// 推一串两次空发布的抬 F（插在续做的两步之间）、回收之后接着走，14 步走完：挂载交回的现行那一版有树而文件集合为空、队列空，F 抬上去了，
+/// 推一串两次空发布的抬 F（插在续做的两步之间）、回收之后接着走，16 步走完：挂载交回的现行那一版有树而文件集合为空、队列空，F 抬上去了，
 /// 池级 checker 全绿。
 /// 判别力：续做里不推（这一条定案之前的做法）时挂载交回 `MountError::Publish { cause: SpaceAdmissionRefused }`。
 #[test]
@@ -420,8 +427,8 @@ fn a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_
                 .collect::<Vec<_>>(),
             output.deletion_queue_continuation.len(),
         ),
-        (vec!["SpaceAdmissionRefused"], vec![2], 14),
-        "续做里推过一串（两次空发布），推它的是一批的落得下那一判；续做走了 14 步"
+        (vec!["SpaceAdmissionRefused"], vec![2], 16),
+        "续做里推过一串（两次空发布），推它的是一批的落得下那一判；续做走了 16 步"
     );
     let first_raise_txg = output.floor_raises_pushed_by_the_continuation[0].publishes[0]
         .root
@@ -461,11 +468,11 @@ fn a_mount_whose_deletion_continuation_finds_no_unblocked_slot_raises_the_floor_
 /// 推满仍放不下（用户 2026-10-03 定「照 D16 做成可写」：挂载照样做成可写、队列留着）：同一份窄盘造法，发完意图与第一批才崩，重开、准入判着、
 /// 删除批大小 1。续做里一批被落得下那一判拒，为它推的那一串推不出空间（一串都没推成，F 没动）：挂载照样交回可写，
 /// 续做停在 `StillShortAfterTheFloorRaises`、最后那一次被拒是落得下那一判，现行那一版的队列还留着第一个文件；
-/// 挂着时一次用户改动先走空、走不动报空间不够（一版都没发成、盘不动）。挂载写下的 13 次发布（写行、暖机与续做发成的几步）留在盘上：
+/// 挂着时一次用户改动先走空、走不动报空间不够（一版都没发成、盘不动）。挂载写下的 17 次发布（写行、暖机与续做发成的几步）留在盘上：
 /// 只读挂载走的是挂载最后落盘的那一版，第一个文件仍在删除中（打开它答「没有文件」）。
 /// 正常卸载照常做成（用户 2026-10-03 定甲 GA，`m3-unlink-code-r3-forks`）：槽被续做吃光了，卸载那一串的预演算上新 F 回收出来、当场放开的那一截
 /// （释放代不高于每块盘上最新有效根的那几个槽），
-/// 抬 F 到现行那一版、两次空发布；之后池级 checker 全绿，再挂可写、续做接着走到空（两步）。
+/// 抬 F 到现行那一版、两次空发布；之后池级 checker 全绿，再挂可写、续做接着走到空（一步）。
 /// 判别力：推满仍放不下照旧交回 `MountError::Publish`（这一条定案之前的做法）、或结局报成 `WalkedToEmpty`，挂载那一步就红；
 /// 卸载回收的槽照旧整截扣到生效（`ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect`）时卸载在预演里取不到落点，卸载那一步红。
 #[test]
@@ -532,8 +539,8 @@ fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_ma
             read_only.effective_root.checkpoint_txg.0 - root_at_the_crash.checkpoint_txg.0,
             read_only.effective_root.rollback_floor.0 - root_at_the_crash.rollback_floor.0,
         ),
-        (13, 0),
-        "挂载写下的 13 次发布（写行、暖机与续做发成的几步）留在盘上；为那一批推的抬 F 一串都没推成，F 没动"
+        (17, 0),
+        "挂载写下的 17 次发布（写行、暖机与续做发成的几步）留在盘上；为那一批推的抬 F 一串都没推成，F 没动"
     );
     assert!(
         matches!(
@@ -587,8 +594,8 @@ fn a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_ma
             remounted.output.deletion_queue_continuation.len(),
             member_name_of(&remounted.current)
         ),
-        (2, "WithTreesAndNoFile"),
-        "再挂之后续做走了两步、走到有树而文件集合为空的一版"
+        (1, "WithTreesAndNoFile"),
+        "再挂之后续做走了一步、走到有树而文件集合为空的一版"
     );
     drop(remounted);
     assert_eq!(
@@ -682,8 +689,8 @@ fn after_a_mount_whose_continuation_stays_short_the_next_mount_is_refused_by_the
 }
 
 /// 续做走到空、F 一次都没抬、可删的单元都还在 defer 里的盘面上卸载（攻方 `m3-unmount-after-short-continuation-r1` 表 A 那 9 格的形：
-/// 卸载被拒不绑队列）：单元区 256 槽、不先覆盖写、14 个单元，崩在意图与两批之后。重开、准入判着、删除批大小 1：取号之前那一判放行、
-/// 没推抬 F，续做 12 步走到空。挂载之后每块盘只剩 9 个没挡的空槽，卸载那一串三次空发布要 18 个——扣着回收出来的槽就在预演里取不到落点；
+/// 卸载被拒不绑队列）：单元区 292 槽、不先覆盖写、16 个单元，崩在意图与两批之后。重开、准入判着、删除批大小 1：取号之前那一判放行、
+/// 没推抬 F，续做 14 步走到空。挂载之后每块盘只剩 10 个没挡的空槽，卸载那一串两次空发布要 14 个——扣着回收出来的槽就在预演里取不到落点；
 /// 甲 GA 之下预演算上新 F 回收出来、当场放开的那一截，卸载做成、F 抬到现行那一版，池级 checker 全绿，再挂可写做成。
 /// 判别力：卸载回收的槽照旧扣到生效时卸载在预演里被拒，卸载那一步红。
 #[test]
@@ -713,8 +720,8 @@ fn a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmoun
             mounted.output.deletion_queue_continuation.len(),
             mounted.current.root().rollback_floor,
         ),
-        (0, 0, 12, root_at_the_crash.rollback_floor),
-        "准入与续做都没推抬 F、F 没动，续做 12 步走到空"
+        (0, 0, 14, root_at_the_crash.rollback_floor),
+        "准入与续做都没推抬 F、F 没动，续做 14 步走到空"
     );
     let free_slots_before_the_unmount: Vec<u64> = mounted
         .allocator
@@ -759,8 +766,8 @@ fn a_mount_whose_continuation_walks_to_empty_without_raising_the_floor_is_unmoun
             slots_taken_by_the_unmount_sequence.len(),
             raised.rollback_floor,
         ),
-        (vec![9, 9], 18, current_txg),
-        "卸载之前每块盘 9 个空槽，那一串要 18 个；F 抬到现行那一版"
+        (vec![10, 10], 14, current_txg),
+        "卸载之前每块盘 10 个空槽，那一串要 14 个；F 抬到现行那一版"
     );
     assert!(
         slots_taken_by_the_unmount_sequence
@@ -1034,8 +1041,8 @@ fn assert_the_continuation_publish_error_accounts_for_every_landed_write(
     );
 }
 
-/// 准入推过抬 F 的起点（代码轮 V3 第一形）：单元区 256 槽、覆盖写 2 次、14 个单元、崩在第 7 批之后；取号之前不够、写行与暖机之后推了一串
-/// （三次空发布），续做 7 步、没再推。续做第二步的第一个写报块设备错：错里 6 份（写行、暖机、推的三次、续做第一步）= 落下的 6 个根槽写，
+/// 准入推过抬 F 的起点（代码轮 V3 第一形）：单元区 292 槽、覆盖写 1 次、16 个单元、崩在第 7 批之后；取号之前不够、写行与暖机之后推了一串
+/// （三次空发布），续做 9 步、没再推。续做第二步的第一个写报块设备错：错里 7 份（写行、暖机两次、推的三次、续做第一步）= 落下的 7 个根槽写，
 /// 写账合计 = 落下的写 − 取号两次（推的那一串先写系统配置那 2 个写在内）。
 /// 判别力：续做之前的写账去掉准入推的那几次空发布（攻方 MU1）时份数少 3；先写系统配置那一步不进错（改之前）时合计少 2。
 #[test]
@@ -1062,14 +1069,14 @@ fn a_continuation_step_failing_after_the_admission_raised_the_floor_hands_back_e
             output.floor_raises_pushed_by_the_continuation.len(),
             output.deletion_queue_continuation.len(),
         ),
-        (vec![3], 0, 7),
-        "起点：准入推了一串三次空发布，续做 7 步、没推"
+        (vec![3], 0, 9),
+        "起点：准入推了一串三次空发布，续做 9 步、没推"
     );
     let (_, publishes_before_the_failing_step) =
         writes_and_publishes_of_the_mount_before(&output, failing_step);
     assert_eq!(
-        publishes_before_the_failing_step, 6,
-        "写行、暖机、推的三次、续做第一步"
+        publishes_before_the_failing_step, 7,
+        "写行、暖机两次、推的三次、续做第一步"
     );
     assert_the_continuation_publish_error_accounts_for_every_landed_write(
         &refusal,
@@ -1079,8 +1086,8 @@ fn a_continuation_step_failing_after_the_admission_raised_the_floor_hands_back_e
     );
 }
 
-/// 续做里推过抬 F 的起点（代码轮 V3 第二形）：同一种盘崩在第 0 批之后（只发了意图）；续做 14 步，第 11 步之后被落得下那一判拒、推一串两次空发布、
-/// 回收之后接着走。推的那一串之后的第一步的第一个写报块设备错：错里 16 份（写行、暖机两次、续做 11 步、推的两次）= 落下的 16 个根槽写，
+/// 续做里推过抬 F 的起点（代码轮 V3 第二形）：同一种盘崩在第 0 批之后（只发了意图）；续做 16 步，第 14 步之后被落得下那一判拒、推一串两次空发布、
+/// 回收之后接着走。推的那一串之后的第一步的第一个写报块设备错：错里 19 份（写行、暖机两次、续做 14 步、推的两次）= 落下的 16 个根槽写，
 /// 写账合计 = 落下的写 − 取号两次。
 /// 判别力：续做的写账去掉续做推的那几次空发布（攻方 MU2）时份数少 2；先写系统配置那一步不进错时合计少 2。
 #[test]
@@ -1106,14 +1113,14 @@ fn a_continuation_step_failing_after_the_continuation_raised_the_floor_hands_bac
                 .collect::<Vec<_>>(),
             output.deletion_queue_continuation.len(),
         ),
-        (0, vec![2], 14),
-        "起点：准入没推，续做 14 步、中间推了一串两次空发布"
+        (0, vec![2], 16),
+        "起点：准入没推，续做 16 步、中间推了一串两次空发布"
     );
     let (_, publishes_before_the_failing_step) =
         writes_and_publishes_of_the_mount_before(&output, failing_step);
     assert_eq!(
-        publishes_before_the_failing_step, 16,
-        "写行、暖机两次、续做 11 步、推的两次"
+        publishes_before_the_failing_step, 19,
+        "写行、暖机两次、续做 14 步、推的两次"
     );
     assert_the_continuation_publish_error_accounts_for_every_landed_write(
         &refusal,
@@ -1135,7 +1142,7 @@ fn a_continuation_step_failing_after_both_the_admission_and_the_continuation_rai
             NarrowPoolCrashedBetweenDeletionBatches {
                 unit_area_slots: 192,
                 overwrites_before_the_unlink: 0,
-                data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
+                data_units_of_the_file: DATA_UNITS_OF_THE_FILE_ON_THE_192_SLOT_DEVICES,
                 batches_published_after_the_intent: 1,
             },
             first_continuation_step_after_the_continuation_raised_the_floor,
@@ -1202,7 +1209,7 @@ fn a_floor_raise_failing_on_a_device_error_in_the_continuation_hands_back_the_wr
     let shape = NarrowPoolCrashedBetweenDeletionBatches {
         unit_area_slots: 192,
         overwrites_before_the_unlink: 0,
-        data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
+        data_units_of_the_file: DATA_UNITS_OF_THE_FILE_ON_THE_192_SLOT_DEVICES,
         batches_published_after_the_intent: 1,
     };
     let (output, failing_publish, refusal, (landed_writes, landed_root_slot_writes)) =
@@ -1293,8 +1300,8 @@ fn unit_record_and_root_slot_writes_of_the_continuation_step(
 }
 
 /// 注错点在续做推的那一串之后的第一步、它第一个写之后的某一个写上（代码轮第三轮判决第三节第 6 条 N2：之前的用例只摆在发布的第一个写上）的那一形：
-/// 起点同 [`a_continuation_step_failing_after_the_continuation_raised_the_floor_hands_back_every_landed_write`]（窄盘崩在第 0 批之后，续做 14 步、
-/// 第 11 步之后推一串两次空发布）。交回那一步之前落盘的发布数（16 份）、注错那一步写了几个单元、注错的错与真落下的写和根槽写。
+/// 起点同 [`a_continuation_step_failing_after_the_continuation_raised_the_floor_hands_back_every_landed_write`]（窄盘崩在第 0 批之后，续做 16 步、
+/// 第 14 步之后推一串两次空发布）。交回那一步之前落盘的发布数（19 份）、注错那一步写了几个单元、注错的错与真落下的写和根槽写。
 fn mount_failing_after_the_first_write_of_the_step_after_the_continuation_raised_the_floor(
     ordinal_within_the_step: impl Fn(u64, u64) -> u64,
 ) -> (usize, u64, MountError, (u64, u64)) {
@@ -1326,7 +1333,7 @@ fn mount_failing_after_the_first_write_of_the_step_after_the_continuation_raised
 }
 
 /// 续做那一步的第二个单元写报块设备错（N2，注错点在发布第一个写之后）：第一个单元写已落盘、属于失败的那一次发布。
-/// 错是 `Publish`、块设备错；已落盘份数 16 = 落下的根槽写数；写账合计（含失败那一次已记的那 1 个单元写）= 落下的写 − 取号两次。
+/// 错是 `Publish`、块设备错；已落盘份数 19 = 落下的根槽写数；写账合计（含失败那一次已记的那 1 个单元写）= 落下的写 − 取号两次。
 /// 判别力：续做那一格交回的失败账照旧是空的（不带失败那一次已落下的写）时合计少 1，那一项红。
 #[test]
 fn a_continuation_step_failing_on_its_second_unit_write_hands_back_every_landed_write() {
@@ -1340,8 +1347,8 @@ fn a_continuation_step_failing_on_its_second_unit_write_hands_back_every_landed_
     );
     assert!(unit_writes >= 2, "那一步至少写两个单元：{unit_writes}");
     assert_eq!(
-        publishes_before_the_failing_step, 16,
-        "写行、暖机两次、续做 11 步、推的两次"
+        publishes_before_the_failing_step, 19,
+        "写行、暖机两次、续做 14 步、推的两次"
     );
     assert_the_continuation_publish_error_accounts_for_every_landed_write(
         &refusal,
@@ -1352,7 +1359,7 @@ fn a_continuation_step_failing_on_its_second_unit_write_hands_back_every_landed_
 }
 
 /// 续做那一步的 journal 记录写报块设备错（N2）：那一步的单元写全落盘、属于失败的那一次发布，记录与根都没落。
-/// 判法同上一条：份数 16 = 落下的根槽写数，写账合计 = 落下的写 − 取号两次。
+/// 判法同上一条：份数 19 = 落下的根槽写数，写账合计 = 落下的写 − 取号两次。
 /// 判别力：失败账照旧是空的时合计少那一步的单元写数，那一项红。
 #[test]
 fn a_continuation_step_failing_on_its_journal_record_write_hands_back_every_landed_write() {
@@ -1370,7 +1377,7 @@ fn a_continuation_step_failing_on_its_journal_record_write_hands_back_every_land
 
 /// 续做那一步的根槽写报块设备错（N2）：那一步的单元写与 journal 记录写落盘，根没落（只读恢复会靠那条记录多走一次，见
 /// `unlink_mount_continuation_and_the_emptied_version.rs` 设备错那一条的说明；这里只比根槽写，不比只读挂载走到的版）。
-/// 判法同上：份数 16 = 落下的根槽写数，写账合计 = 落下的写 − 取号两次。
+/// 判法同上：份数 19 = 落下的根槽写数，写账合计 = 落下的写 − 取号两次。
 /// 判别力：失败账照旧是空的时合计少那一步的单元写与记录写，那一项红。
 #[test]
 fn a_continuation_step_failing_on_its_root_slot_write_hands_back_every_landed_write() {
@@ -1400,7 +1407,7 @@ fn a_floor_raise_in_the_continuation_failing_on_its_system_configuration_write_h
             NarrowPoolCrashedBetweenDeletionBatches {
                 unit_area_slots: 192,
                 overwrites_before_the_unlink: 0,
-                data_units_of_the_file: DATA_UNITS_OF_THE_FILE_DELETED_BY_THE_CONTINUATION,
+                data_units_of_the_file: DATA_UNITS_OF_THE_FILE_ON_THE_192_SLOT_DEVICES,
                 batches_published_after_the_intent: 1,
             },
             |output| {
@@ -1465,13 +1472,13 @@ fn a_floor_raise_in_the_continuation_failing_on_its_system_configuration_write_h
 /// （[`a_mount_whose_deletion_continuation_stays_short_after_raising_the_floor_is_made_writable_and_its_unmount_lets_the_next_mount_continue`]
 /// 的造法），先在一份拷贝上不注错地卸载一次，数清那一串第一次空发布写几个单元、其中落在这一串回收出来的槽上的那几个各是什么释放代；
 /// 再在另一份上从「先写系统配置那 2 个写 + 第一次空发布的单元写」之后的第一个写起每一个写都报错——单元已写进当场放开的槽，journal 记录与根都没落。
-/// GA 的结局（现行那一版 txg 21，第一版区域归属 0 / 1 / 0：盘 0 上最新的根 21、盘 1 上 19，取小 19）：第一次空发布落在回收槽上的单元，
-/// 那几个槽的释放代都不高于 19；回收出来的槽里释放代 20、21 的那几个（txg 19 那条根还引用着）扣到生效，第一次空发布一个都没落上去。
+/// GA 的结局（现行那一版 txg 24，第一版区域归属 0 / 1 / 0：盘 0 上最新的根 24、盘 1 上 22，取小 22）：第一次空发布落在回收槽上的单元，
+/// 那几个槽的释放代都不高于 22；回收出来的槽里释放代 23、24 的那几个（txg 22 那条根还引用着）扣到生效，第一次空发布一个都没落上去。
 /// 改钉这一句的理由：原来钉的「有单元落在回收出来的槽上」是丙 1 的说法（回收的槽整截当场能发）；GA 下当场能发的只有释放代不高于每块盘上最新根的那一截
 /// （判决 `research/prompts/m3-unlink-code-r3-main-verification.md` 第三节第 1 条，用户 2026-10-03 定甲 GA），钉的就改成落上去的那几个槽的释放代。
 /// 掉电之后：池级 checker 全绿；只读挂载走的是卸载之前现行那一版（根上带旧 F），而系统配置里已是新 F；再挂可写做成、续做接着走到空，池级 checker 全绿。
 /// 判别力：卸载回收的槽照旧整截扣到生效时拷贝上那一次卸载就在预演里被拒，第一步红。整截当场放开（丙 1）时这一形照样绿：这块盘面上那一串的单元
-/// 本来就没落到释放代 20、21 的槽上（判决第二节 N1「X 第 1404 行那一份盘面的交集」坐实为空；本批证红时量过，`crates/mutations.tsv` 不留这一行），
+/// 本来就没落到释放代 23、24 的槽上（判决第二节 N1「X 第 1404 行那一份盘面的交集」坐实为空；本批证红时量过，`crates/mutations.tsv` 不留这一行），
 /// 释放代那一项只钉 GA 的结局、不分辨丙 1；分辨丙 1 的是 `a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs` 那两形。
 #[test]
 fn an_unmount_losing_power_after_its_system_configuration_write_before_any_root_with_the_new_floor_recovers_to_the_current_version(
@@ -1603,11 +1610,11 @@ fn an_unmount_losing_power_after_its_system_configuration_write_before_any_root_
     );
 }
 
-/// 续做推满仍放不下的那块窄盘上，卸载之前现行那一版的 txg，与那时每块盘上最新的根的 txg 取小（第一版区域归属 0 / 1 / 0：txg 21 的根在盘 0，
-/// 盘 1 上最新的是 txg 19；这块盘的根从写行起连号写下来、没有回退，没有被抛弃的根）。
-const CURRENT_TXG_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION: CheckpointTxg = CheckpointTxg(21);
+/// 续做推满仍放不下的那块窄盘上，卸载之前现行那一版的 txg，与那时每块盘上最新的根的 txg 取小（第一版区域归属 0 / 1 / 0：txg 24 的根在盘 0，
+/// 盘 1 上最新的是 txg 22；这块盘的根从写行起连号写下来、没有回退，没有被抛弃的根）。
+const CURRENT_TXG_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION: CheckpointTxg = CheckpointTxg(24);
 const NEWEST_ROOT_ON_EVERY_DEVICE_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION: CheckpointTxg =
-    CheckpointTxg(19);
+    CheckpointTxg(22);
 
 /// 甲 GA 在续做推满仍放不下的那块窄盘上的结局（用户 2026-10-03 定，`mount::RaiseFloorEntry::reuse_of_the_reclaimed_slots`）：卸载那一串
 /// 第一次空发布落在这一串回收出来的槽上的单元至少一个（槽被续做吃光，卸载靠当场放开的那一截做成），那几个槽在卸载之前的分配记录里的释放代都不高于
@@ -1650,27 +1657,27 @@ fn assert_the_first_unmount_publish_lands_only_on_slots_released_no_later_than_t
         .collect();
     assert_eq!(
         current_root.checkpoint_txg, CURRENT_TXG_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION,
-        "卸载之前现行那一版 txg 21（盘 0 上），盘 1 上最新的根 txg 19"
+        "卸载之前现行那一版 txg 24（盘 0 上），盘 1 上最新的根 txg 22"
     );
     assert!(
         !release_generations_under_the_first_publish.is_empty()
             && release_generations_under_the_first_publish.iter().all(|generation| {
                 *generation <= NEWEST_ROOT_ON_EVERY_DEVICE_BEFORE_THE_UNMOUNT_OF_THE_SHORT_CONTINUATION
             }),
-        "第一次空发布落在回收槽上的单元有，那几个槽的释放代都不高于 19：{release_generations_under_the_first_publish:?}"
+        "第一次空发布落在回收槽上的单元有，那几个槽的释放代都不高于 22：{release_generations_under_the_first_publish:?}"
     );
     assert!(
         !reclaimed_released_after_the_newest_root_of_every_device.is_empty(),
-        "回收出来的槽里有释放代高于 19 的（txg 19 那条根还引用着，GA 下扣到生效）"
+        "回收出来的槽里有释放代高于 22 的（txg 22 那条根还引用着，GA 下扣到生效）"
     );
 }
 
 /// 同一刀（卸载那一串先写系统配置之后、第一次空发布的单元写完、journal 记录与根都没落）之后一块盘丢掉、只剩一块（攻方丙 1 那一节自列没判的第二件）：
-/// 丢掉的是住着现行那一版的根的盘 0（现行那一版 txg 21，21 mod 3 = 0，第一版区域归属 0 / 1 / 0），剩下的盘 1 上最新的根是 txg 19，
-/// 早于它系统配置里的新 F（21）。择根与重放：恢复择到盘 1 上最新的那条根、journal 没往后施加，只读挂载走 txg 19 那一版、根上带旧 F；
+/// 丢掉的是住着现行那一版的根的盘 0（现行那一版 txg 24，24 mod 3 = 0，第一版区域归属 0 / 1 / 0），剩下的盘 1 上最新的根是 txg 22，
+/// 早于它系统配置里的新 F（24）。择根与重放：恢复择到盘 1 上最新的那条根、journal 没往后施加，只读挂载走 txg 22 那一版、根上带旧 F；
 /// 可写挂载在读任何盘之前就被 w 的下限拒。
-/// GA 的结局（用户 2026-10-03 定甲 GA）：第一次空发布写下去、落在回收槽上的那几个单元，槽的释放代都不高于 19——txg 19 那条根不引用它们；
-/// 释放代 20、21 的那几个（txg 19 那条根还引用着）扣到生效、没被写。改钉这一句的理由：原来钉的「写进回收出来的槽的那几个单元没让池级 checker
+/// GA 的结局（用户 2026-10-03 定甲 GA）：第一次空发布写下去、落在回收槽上的那几个单元，槽的释放代都不高于 22——txg 22 那条根不引用它们；
+/// 释放代 23、24 的那几个（txg 22 那条根还引用着）扣到生效、没被写。改钉这一句的理由：原来钉的「写进回收出来的槽的那几个单元没让池级 checker
 /// 多判出一条」分不出 GA 与丙 1——单盘镜像上卸载之前 checker 就红六七条，条名已饱和（判决 `research/prompts/m3-unlink-code-r3-main-verification.md`
 /// 第二节 N1「第五节跑前判据的字面」那一行）；分得出的是被写的槽的释放代，与剩下那块盘的读回（`a_lone_surviving_device_reads_back_what_it_read_before_the_unmount.rs`）。
 /// checker 那一项照旧判（与「只写了系统配置、单元一个都没写」那一刀逐条相同），只记饱和、不靠它分辨。
@@ -1766,11 +1773,11 @@ fn after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_survi
             recover(&survivor, JournalPolicy::Consult).effective_root,
         ),
         (
-            CheckpointTxg(21),
-            CheckpointTxg(21),
-            Some((current_root.instance, CheckpointTxg(19)))
+            CheckpointTxg(24),
+            CheckpointTxg(24),
+            Some((current_root.instance, CheckpointTxg(22)))
         ),
-        "剩下的盘 1 系统配置里是新 F 21，恢复择到它上面最新的根 txg 19、journal 没往后施加"
+        "剩下的盘 1 系统配置里是新 F 24，恢复择到它上面最新的根 txg 22、journal 没往后施加"
     );
     let read_only = mount_read_only(&survivor).expect("剩下那块盘只读挂载");
     assert_eq!(
@@ -1778,8 +1785,8 @@ fn after_the_unmount_loses_power_before_any_root_with_the_new_floor_a_lone_survi
             read_only.effective_root.checkpoint_txg,
             read_only.effective_root.rollback_floor,
         ),
-        (CheckpointTxg(19), current_root.rollback_floor),
-        "只读挂载走 txg 19 那一版，根上带旧 F，低于系统配置里的新 F"
+        (CheckpointTxg(22), current_root.rollback_floor),
+        "只读挂载走 txg 22 那一版，根上带旧 F，低于系统配置里的新 F"
     );
     let mut survivor_devices = plain_devices_on(&survivor);
     assert!(
diff --git a/crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs b/crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs
index 944dcb6..dcf9a6b 100644
--- a/crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs
+++ b/crates/singlefs-harness/tests/admission_refuses_before_units_cannot_land_outside_the_mounts_clustered_segments.rs
@@ -510,14 +510,15 @@ fn a_write_whose_data_unit_lands_but_whose_commit_generated_units_find_no_unbloc
 }
 
 /// 第一格那一形，会话的准入用只供测试的开关关掉（`SpaceAdmission::SkippedByTheTestOnlySwitch`：式子与落得下两判都不判）：
-/// 落点那一道照样兜着。两块单元区 384 槽的小盘，崩了再挂、直接推 200 次空发布之后，经会话把文件顺序写到 2、3……25 个数据单元，都一次做成；
-/// 写到 26 个单元：走固定点时在分配器的拷贝上取不到数据单元 19 的落点，交回 `PlacementRefused { Data(19), NoFreeSlotOnAnyDevice }`，
+/// 落点那一道照样兜着。两块单元区 384 槽的小盘，崩了再挂、直接推 200 次空发布之后，经会话把文件顺序写到 2、3……29 个数据单元，都一次做成；
+/// 写到 30 个单元：走固定点时在分配器的拷贝上取不到数据单元 19 的落点，交回 `PlacementRefused { Data(19), NoFreeSlotOnAnyDevice }`，
 /// 会话照 D16（发布语义） 已定项 1「准入」那一行（准入放行而落点取不到也推）推一串抬 F、回收之后再发，做成——推过的只这一串、
 /// 推它的是那一次落点被拒；F 抬上去了，池级 checker 0 违例。
-/// 判别力：会话只在准入拒时推（落点被拒原样交回）时，写到 26 个单元报 `UserChangeRefused::Publish { PlacementRefused }`。
+/// 判别力：会话只在准入拒时推（落点被拒原样交回）时，写到 30 个单元报 `UserChangeRefused::Publish { PlacementRefused }`。
 /// A3b 把小盘改成 6 MiB 环、单元区从 1408 起之前是「80 次空发布、写到 6 个、第 7 个推」，与第一格同一次改（草稿探针在新几何上现跑的数）；
 /// 环窗口真扣、小盘放大 815 槽（用户 2026-10-02 定「甲」）之前是「写到 6 个、数据单元 0 被拒」，放大之后 200、400 次空发布都是写到 26 个、
-/// 数据单元 19 被拒（三·2 第 6 条实现员副本上的草稿探针）。
+/// 数据单元 19 被拒（三·2 第 6 条实现员副本上的草稿探针）；第 50 批补足 815 → 1007 槽（退役树那几项）之后是写到 30 个、数据单元 19 被拒
+/// （第 50 批实现员副本上现跑：29 个之前都一次做成）。
 #[test]
 fn with_the_space_admission_switched_off_the_session_raises_the_floor_after_a_placement_refusal_and_publishes_the_write(
 ) {
@@ -532,7 +533,7 @@ fn with_the_space_admission_switched_off_the_session_raises_the_floor_after_a_pl
     for _ in 0..200 {
         publish_an_empty_version_directly_on_the_session(&mut pool);
     }
-    for data_units in 2..=25 {
+    for data_units in 2..=29 {
         let published = pool
             .sequential_write(data_units * data_unit_payload_capacity())
             .unwrap_or_else(|refusal| {
@@ -545,9 +546,9 @@ fn with_the_space_admission_switched_off_the_session_raises_the_floor_after_a_pl
     }
     let floor_before = pool.rollback_floor();
     let published = pool
-        .sequential_write(26 * data_unit_payload_capacity())
+        .sequential_write(30 * data_unit_payload_capacity())
         .unwrap_or_else(|refusal| {
-            panic!("经会话顺序写到 26 个数据单元，推过抬 F 之后做成：{refusal:?}")
+            panic!("经会话顺序写到 30 个数据单元，推过抬 F 之后做成：{refusal:?}")
         });
     assert!(
         matches!(
diff --git a/crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs b/crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs
index 9cd2ad3..10e68e6 100644
--- a/crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs
+++ b/crates/singlefs-harness/tests/admission_ring_window_and_tombstone_prepay_on_the_publish_path.rs
@@ -38,8 +38,9 @@ const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);
 /// 进程内那几次发布都在新池新建文件那个实例里发。
 const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);
 
-/// 产品批大小 144、S = 8、两块 4 GiB 盘：W = 24 + min(524, 340) + min(4, 340) = 368，环窗口 = 25 × 368 槽。
-const RING_WINDOW_SLOTS_OF_THE_PRODUCT_BATCH: u64 = 25 * 368;
+/// 产品批大小 144、S = 8、两块 4 GiB 盘、退役树至多只有根：W = 24 + (6 + 0) + min(524, 340) + min(4, 340) = 374，环窗口 = 25 × 374 槽
+/// （退役树那 6 + H_退役 是判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条加的，此前 368）。
+const RING_WINDOW_SLOTS_OF_THE_PRODUCT_BATCH: u64 = 25 * 374;
 
 fn slots_per_region() -> RootRingSlotsPerRegion {
     parameters().geometry.root_ring_slots_per_region
@@ -166,11 +167,11 @@ fn publish_the_next_step_in_process(pool: &mut BuiltPool) -> TransactionOutput {
     output
 }
 
-/// 新池新建文件那一版 inode 树只有第一个文件（n_live = 1）、墓碑树零根（H = 0）：
-/// 标签 6 = 2 × ((2 + 0 + 1) × 16384 + 41 × 1) = 98386；再建两个 inode（n_live = 3）：2 × (49152 + 123) = 98550。
+/// 新池新建文件那一版 inode 树只有第一个文件（n_live = n_记录 = 1）、墓碑树与退役树零根（H_墓碑 = H_退役 = 0）：
+/// 标签 6 = 2 × ((2 + 0 + 1 + 2 + 0 + 1) × 16384 + 41 × 1 + 35 × 1) = 196760；再建两个 inode（n_live = n_记录 = 3）：
+/// 2 × (98304 + 123 + 105) = 197064（退役那一份是判决 m3-retire-r1 第三节第 6 条）。
 #[test]
-fn tag_six_of_the_first_file_version_is_three_slots_and_one_live_object_and_two_new_inodes_add_two_more(
-) {
+fn tag_six_of_the_first_file_version_is_six_slots_and_one_record_and_two_new_inodes_add_two_more() {
     let mut pool = build_pool("tag-six-first-file-and-new-inodes");
     let first_file_version = pool.output.clone();
     let with_two_new_inodes = publish_new_inodes_in_process(&mut pool, 2);
@@ -179,12 +180,12 @@ fn tag_six_of_the_first_file_version_is_three_slots_and_one_live_object_and_two_
             tag_six_of(&first_file_version),
             tag_six_of(&with_two_new_inodes)
         ),
-        (98_386, 98_550),
-        "标签 6 按每一版现算：两份副本 × (3 槽常驻 + 41 字节 × n_live)"
+        (196_760, 197_064),
+        "标签 6 按每一版现算：两份副本 × (墓碑 3 槽 + 退役 3 槽常驻 + 41 字节 × n_live + 35 字节 × n_记录)"
     );
 }
 
-/// 发布之前的读数：环窗口每块盘各扣 9200 槽，已承诺预留取那一版记账行里的标签 6。
+/// 发布之前的读数：环窗口每块盘各扣 9350 槽，已承诺预留取那一版记账行里的标签 6。
 #[test]
 fn the_publish_reading_deducts_the_ring_window_of_the_product_batch_on_each_device() {
     let pool = build_pool("ring-window-in-the-publish-reading");
@@ -200,7 +201,7 @@ fn the_publish_reading_deducts_the_ring_window_of_the_product_batch_on_each_devi
             ring_window_slots_on_each_device(&reading),
             reading.pool_wide().committed_reservation.0,
         ),
-        (vec![RING_WINDOW_SLOTS_OF_THE_PRODUCT_BATCH; 2], 98_386),
+        (vec![RING_WINDOW_SLOTS_OF_THE_PRODUCT_BATCH; 2], 196_760),
         "每块盘扣 (3S + 1) × W 槽；全池那一项读到标签 6"
     );
     assert_eq!(
@@ -210,10 +211,10 @@ fn the_publish_reading_deducts_the_ring_window_of_the_product_batch_on_each_devi
     );
 }
 
-/// 第二个几何：随机历史单元区 384 槽那一档小盘（环窗口补足 815 槽之后单元区 [1408, 2607)、分配记录树根层级 1），
-/// 每块盘罩叶 ⌊2606 ÷ 812⌋ − ⌊1408 ÷ 812⌋ + 1 = 3 片 ⇒ N_0 = 6、W = 24 + 6 = 30，环窗口 = 25 × 30 = 750 槽；可用(d) 正好少这么多字节。
+/// 第二个几何：随机历史单元区 384 槽那一档小盘（环窗口补足 1007 槽之后单元区 [1408, 2799)、分配记录树根层级 1），
+/// 每块盘罩叶 ⌊2798 ÷ 812⌋ − ⌊1408 ÷ 812⌋ + 1 = 3 片 ⇒ N_0 = 6、W = 24 + (6 + 0) + 6 = 36，环窗口 = 25 × 36 = 900 槽；可用(d) 正好少这么多字节。
 #[test]
-fn the_publish_reading_on_the_384_slot_small_devices_deducts_twenty_five_times_thirty_slots() {
+fn the_publish_reading_on_the_384_slot_small_devices_deducts_twenty_five_times_thirty_six_slots() {
     let pool = PoolUnderTest::start_after_the_first_file(
         HistoryDeviceWidth::UnitAreaOf384Slots,
         plain_device,
@@ -231,12 +232,12 @@ fn the_publish_reading_on_the_384_slot_small_devices_deducts_twenty_five_times_t
             ring_window_slots_on_each_device(&reading),
             available_bytes_the_ring_window_takes_on_each_device(&reading),
         ),
-        (vec![25 * 30; 2], vec![i128::from(25 * 30 * 16384_u64); 2]),
-        "每块盘扣 (3 × 8 + 1) × 30 槽，可用(d) 正好少这么多字节"
+        (vec![25 * 36; 2], vec![i128::from(25 * 36 * 16384_u64); 2]),
+        "每块盘扣 (3 × 8 + 1) × 36 槽，可用(d) 正好少这么多字节"
     );
 }
 
-/// 批大小 K 取分配器上装的删除开关：K = 1 时 W = 24 + min(524, 54) + min(4, 54) = 82，环窗口 = 25 × 82 = 2050 槽。
+/// 批大小 K 取分配器上装的删除开关：K = 1 时 W = 24 + (6 + 0) + min(524, 54) + min(4, 54) = 88，环窗口 = 25 × 88 = 2200 槽。
 #[test]
 fn the_ring_window_follows_the_batch_size_switch_installed_on_the_allocator() {
     let mut pool = build_pool("ring-window-follows-the-batch-switch");
@@ -244,14 +245,17 @@ fn the_ring_window_follows_the_batch_size_switch_installed_on_the_allocator() {
         batch_size: batches_of_one_data_unit(),
         tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
         lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container_capacity:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     });
     assert_eq!(
         RingWindowOfTheReading::DeductedOnEachDevice {
-            slots_per_region: slots_per_region()
+            slots_per_region: slots_per_region(),
+            retirement_tree_internal_levels: 0,
         }
         .on_each_device(&pool.allocator),
-        BytesOnOneDevice::of_slots(25 * 82)
+        BytesOnOneDevice::of_slots(25 * 88)
     );
 }
 
@@ -285,6 +289,6 @@ fn the_writable_mount_reading_counts_the_ring_window_as_usable_only_while_the_de
             vec![RING_WINDOW_SLOTS_OF_THE_PRODUCT_BATCH; 2],
             vec![0, 0],
         ),
-        "队列非空：可动用；队列空：扣 9200 槽；没有第一个文件版本：不起计"
+        "队列非空：可动用；队列空：扣 9350 槽（退役树这时只有根，H_退役 = 0）；没有第一个文件版本：不起计"
     );
 }
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index d106b90..9240663 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -18,9 +18,11 @@ use common::{
 use singlefs_checker::image::{
     InvariantVerdict, ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
     DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY, DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
+    EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
     FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
     HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
     INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
+    INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
     INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
     INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
     INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
@@ -30,7 +32,11 @@ use singlefs_checker::image::{
     NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
     ONE_OBJECT_BIRTH_PER_INODE_NUMBER, POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
     PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
-    REFERENCE_INTERVALS_ARE_EXACT, ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES,
+    REFERENCE_INTERVALS_ARE_EXACT,
+    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
+    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER, ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES,
     ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
     SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
     SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
@@ -64,8 +70,9 @@ use singlefs_core::mount::{
     MountError, RollbackTarget, ShadowLedger, Unmounted,
 };
 use singlefs_core::records::{
-    TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_DELETED_INODES,
-    TREE_KIND_LINEAGE, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_TOMBSTONE,
+    RetirementRecord, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
+    TREE_KIND_DELETED_INODES, TREE_KIND_LINEAGE, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_RETIREMENT,
+    TREE_KIND_TOMBSTONE,
 };
 use singlefs_core::recovery::{
     instance_table_chain_of_root, recover, JournalPolicy, PoolReader, RecoveryFailure,
@@ -1886,10 +1893,29 @@ fn expected_verdict_of_the_tombstone_and_lineage_invariants_without_deletion(
         TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER => {
             InvariantVerdict::NotApplicable(NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION)
         }
+        // 退役树那几条（第 50 批）：没有叶容器身份消失，退役树零根；I-9.35 在相邻两版上真被评估过且成立（叶身份集没变、没有退役记录）。
+        RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER => {
+            InvariantVerdict::NotApplicable(NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION)
+        }
+        EVERY_RETIREMENT_RECORD_IS_WELL_FORMED => {
+            InvariantVerdict::NotApplicable(NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION)
+        }
+        RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT
+        | RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF => InvariantVerdict::NotApplicable(
+            NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
+        ),
         _ => InvariantVerdict::Holds,
     }
 }
 
+/// 退役树那几条没有对象时报的理由（`walk` 的 `judge_tombstones_and_the_lineage_of_the_versions` 与 `check_pool_image` 末尾）。
+const NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
+    "走读到的每一版退役树都是零根：没有退役树内部条目可判";
+const NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION: &str =
+    "走读到的每一版退役树都是零根：没有退役记录可判";
+const NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION: &str =
+    "走读到的每一版里没有一条退役记录在谱系上：没有对象可判";
+
 /// 墓碑与谱系那几条没有对象时报的理由（`walk` 的 `judge_tombstones_and_the_lineage_of_the_versions` 与 `check_pool_image` 末尾）。
 const NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
     "走读到的每一版谱系树都是零根：没有谱系树内部条目可判";
@@ -3016,7 +3042,33 @@ const TOMBSTONE_AND_LINEAGE_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(&str,
     ),
     (
         ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
-        "inode_number_watermark_pressed_back_reissues_a_number.rs: pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_only_the_one_birth_per_number_invariant",
+        "inode_number_watermark_pressed_back_reissues_a_number.rs: pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants",
+    ),
+];
+
+/// 退役树那五条（I-9.31–I-9.35，第 50 批，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条）的坏镜像各建在
+/// 「三个单元的文件删到底」那一份上（唯一那片叶容器身份消失、退役树一条记录），或它再建 31 次 inode、删的那几版转出回退候选集的那一份上，
+/// 不进只改干净镜像一处的表。
+const RETIREMENT_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE: [(&str, &str); 5] = [
+    (
+        RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        "checker_known_bad_images.rs: a_retirement_tree_entry_whose_identity_names_another_container_reddens_only_the_retirement_entry_invariant",
+    ),
+    (
+        EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
+        "checker_known_bad_images.rs: a_retirement_record_naming_a_container_of_record_type_one_reddens_only_the_well_formed_retirement_record_invariant",
+    ),
+    (
+        RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
+        "checker_known_bad_images.rs: a_second_copy_of_the_retirement_record_reddens_only_the_distinct_retirement_records_invariant",
+    ),
+    (
+        RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+        "checker_known_bad_images.rs: a_retirement_record_naming_the_live_leaf_reddens_only_the_no_live_leaf_invariant",
+    ),
+    (
+        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+        "checker_known_bad_images.rs: a_retirement_record_moved_off_the_txg_the_leaf_died_at_reddens_only_the_retirement_completeness_invariant",
     ),
 ];
 
@@ -3039,6 +3091,28 @@ const INSTANCE_TABLE_RECORD_FORM_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE: [(&str
     ),
 ];
 
+/// I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，第 49 批） 的坏镜像各要一份自己的基底（崩溃恢复抛弃建过 inode 的根之后再建、再回退；
+/// 建过 inode 之后覆盖写），不进只改干净镜像一处的表；这里登记它们在哪几个用例里（各自断言只红它）。② 那一判（记录晚于台阶出生）的坏镜像是
+/// 压回水位再建的那几份（`inode_number_watermark_pressed_back_reissues_a_number.rs`）。
+const INODE_NUMBER_WATERMARK_STEP_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE: [(&str, &str); 4] = [
+    (
+        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
+        "checker_known_bad_images.rs: a_lineage_record_whose_watermark_rolled_back_from_is_below_the_last_step_reddens_only_the_watermark_step_invariant",
+    ),
+    (
+        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
+        "checker_known_bad_images.rs: a_lineage_record_whose_watermark_rolled_back_from_is_zero_reddens_only_the_watermark_step_invariant",
+    ),
+    (
+        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
+        "checker_known_bad_images.rs: an_inode_number_watermark_step_dropped_from_the_accounting_tree_reddens_only_the_watermark_step_invariant",
+    ),
+    (
+        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
+        "checker_known_bad_images.rs: an_inode_number_watermark_step_raised_above_the_watermark_reddens_only_the_watermark_step_invariant",
+    ),
+];
+
 /// 清单里不要求坏镜像的几条：I-4.4（意图只描述目标态） 第一版报不适用（用户 2026-10-05 定，`.claude/kb/milestone/03-third-txn.md` 三·3 那一行
 /// 「I-4.4 不要求坏镜像与变异行」）。
 const INVARIANTS_WITHOUT_A_BAD_IMAGE: [&str; 1] = [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE];
@@ -3156,6 +3230,16 @@ fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target()
                 .iter()
                 .map(|(invariant, _)| *invariant),
         )
+        .chain(
+            INODE_NUMBER_WATERMARK_STEP_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE
+                .iter()
+                .map(|(invariant, _)| *invariant),
+        )
+        .chain(
+            RETIREMENT_INVARIANTS_AND_WHERE_THEIR_BAD_IMAGES_LIVE
+                .iter()
+                .map(|(invariant, _)| *invariant),
+        )
         .collect();
     let listed: std::collections::BTreeSet<&str> = singlefs_checker::image::IMPLEMENTED_INVARIANTS
         .iter()
@@ -3590,7 +3674,8 @@ fn reused_unit_referenced_only_by_the_newest_root_makes_the_reuse_invariant_red(
 ///
 /// 跟着变的只有四条**不适用**：走读在判红的那一步停下、不按字段表解那 15 行记账条目，于是
 /// I-3.1（已分配统计对得上）、I-3.11（已分配减 defer 等于最新根走读）、I-5.2（空闲统计对得上）与 I-9.6（水位大于两处最大号） 都没有了判的对象
-/// （另有 I-3.13 与 I-3.4（可用空间扣待删占用） 两条读的也是记账行：删除态扫描里记账树条目宽不到 34 那一版报射程里判不了）。
+/// （另有 I-3.13 与 I-3.4（可用空间扣待删占用） 两条读的也是记账行：删除态扫描里记账树条目宽不到 34 那一版报射程里判不了；
+/// I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上） 的台阶行取自主走读走全了的记账树，走不全就没有行可判）。
 /// 「停下不解」正是 I-1.10（码 2 条目宽等于字段表宽） 要的判定顺序（**先于**按字段表解条目，与
 /// I-1.7（打包容器合法与判定顺序） 同一纪律），所以这四条转成不适用是这份镜像的结构决定的，照实钉下来。
 #[test]
@@ -3623,9 +3708,9 @@ fn an_entry_width_that_is_not_the_field_table_width_reddens_only_the_entry_width
         .collect();
     assert_eq!(
         turned_not_applicable,
-        ["I-3.1", "I-3.11", "I-5.2", "I-9.6", "I-3.13", "I-3.4"],
-        "跟着转成不适用的该只有这六条（走读在 I-1.10 那一步停下，记账条目不再解；I-3.13 判的就是记账行的代段；\
-         I-3.4 的删除态扫描读记账树也按条目宽 ≥ 34 才解，读不全那一版报射程里判不了）：{verdicts:?}"
+        ["I-3.1", "I-3.11", "I-5.2", "I-9.6", "I-3.13", "I-3.4", "I-9.30"],
+        "跟着转成不适用的该只有这七条（走读在 I-1.10 那一步停下，记账条目不再解；I-3.13 判的就是记账行的代段；\
+         I-3.4 的删除态扫描读记账树也按条目宽 ≥ 34 才解，读不全那一版报射程里判不了；I-9.30 的台阶行取自走全了的记账树）：{verdicts:?}"
     );
 }
 
@@ -6943,8 +7028,8 @@ fn published_nodes_behind_an_intermediate_row_still_count_against_the_tree_ident
                 .highest()
                 .0
         ),
-        (65536, 65547),
-        "实例 2 发 65536..65547，其中 65541..65547 只有树表条目、没有单元 ⇒ 盘上扫得出来最大的是映射树 65540"
+        (65536, 65548),
+        "实例 2 发 65536..65548，其中 65541..65548 只有树表条目、没有单元 ⇒ 盘上扫得出来最大的是映射树 65540"
     );
     assert_lowering_the_watermark_reddens_only_the_watermark_invariant(
         &pool.memory_pool(),
@@ -9280,12 +9365,14 @@ fn rewriting_an_older_root_rechains_the_later_record_of_its_instance_and_every_v
 
 // ───────── I-9.16 ③ ④、I-5.5 部分没判、I-2.4 根槽留位、带树一版的分配记录树根（判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md`）─────────
 
-/// 新池新建文件那一版树表里十一条按发号次序排（从 0 数）：分配记录树第 2 条、deadlist 第 6 条、logged_ops 第 10 条。
+/// 新池新建文件那一版树表里十二条按发号次序排（从 0 数）：分配记录树第 2 条、deadlist 第 6 条、logged_ops 第 10 条、退役树第 11 条（最末）。
 const ALLOCATION_TREE_TABLE_ENTRY_INDEX: usize = 2;
 const DEADLIST_TREE_TABLE_ENTRY_INDEX: usize = 6;
 const LOGGED_OPERATIONS_TREE_TABLE_ENTRY_INDEX: usize = 10;
+const RETIREMENT_TREE_TABLE_ENTRY_INDEX: usize = 11;
 /// 种类码登记表（D8（核心索引结构） 已定项 9）里没有的一个码：分配记录那一条改成它，树表里就一条种类 3 都没有。
-const TREE_KIND_NOT_REGISTERED: u16 = 13;
+/// 13 第 50 批登记给了退役树，改用 14。
+const TREE_KIND_NOT_REGISTERED: u16 = 14;
 /// 新池新建文件那一版树表里 extent 树那条的树 ID（mkfs 的水位 65536）与 inode 树那条的树 ID（65537）：
 /// 改成种类 1 的那一条头 ID 照 extent 条目写成 inode 树的号，I-9.27 不先红。
 const INODE_TREE_IDENTIFIER_OF_THE_FIRST_FILE_VERSION: u64 = 65537;
@@ -9442,7 +9529,7 @@ fn a_tree_table_whose_allocation_entry_carries_an_unregistered_kind_reddens_only
     assert_only_the_tree_table_ordering_invariant_is_violated(
         &clean,
         &image,
-        "根（实例 1、txg 3）的树表有 11 条，种类 3 一条都没有（④ 树表非空时进树表的十一种各恰好一条）",
+        "根（实例 1、txg 3）的树表有 12 条，种类 3 一条都没有（④ 树表非空时进树表的十二种各恰好一条）",
     );
 }
 
@@ -9496,7 +9583,7 @@ fn a_tree_table_without_its_allocation_entry_reddens_only_the_tree_table_orderin
     assert_only_the_tree_table_ordering_invariant_is_violated(
         &clean,
         &image,
-        "根（实例 1、txg 3）的树表有 10 条，种类 3 一条都没有（④ 树表非空时进树表的十一种各恰好一条）",
+        "根（实例 1、txg 3）的树表有 11 条，种类 3 一条都没有（④ 树表非空时进树表的十二种各恰好一条）",
     );
 }
 
@@ -9511,8 +9598,8 @@ fn remove_tree_table_entry_of_kind(tree_table: &mut [u8], index: usize, kind: u1
     remove_tree_table_entry(tree_table, index);
 }
 
-/// I-9.16 ④ 十一种（正推 C1，判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-9.16 ④ 放过缺种类的树表（C1）」
-/// 那一行）：整条去掉 deadlist 那一条——树表 10 条、没有种类 8。改之前只要种类 1–4 时整份 checker 一条不红，core 的可写挂载重建这一版时
+/// I-9.16 ④ 十二种（正推 C1，判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-9.16 ④ 放过缺种类的树表（C1）」
+/// 那一行）：整条去掉 deadlist 那一条——树表 11 条、没有种类 8。改之前只要种类 1–4 时整份 checker 一条不红，core 的可写挂载重建这一版时
 /// 逐种取号报「树表里没有那棵树」（正推探针 `research/prompts/m3-formal-code-b26-r1-sonnet-model/probe/known_bad_probe.rs.append`）。
 #[test]
 fn a_tree_table_without_its_deadlist_entry_reddens_only_the_tree_table_ordering_invariant() {
@@ -9533,17 +9620,17 @@ fn a_tree_table_without_its_deadlist_entry_reddens_only_the_tree_table_ordering_
     assert_only_the_tree_table_ordering_invariant_is_violated(
         &clean,
         &image,
-        "根（实例 1、txg 3）的树表有 10 条，种类 8 一条都没有（④ 树表非空时进树表的十一种各恰好一条）",
+        "根（实例 1、txg 3）的树表有 11 条，种类 8 一条都没有（④ 树表非空时进树表的十二种各恰好一条）",
     );
 }
 
-/// I-9.16 ④ 十一种（同上判决与探针）：整条去掉 logged_ops 那一条（树表最末一条）——树表 10 条、没有种类 12。走读另红 I-1.1，照实钉住
+/// I-9.16 ④ 十二种（同上判决与探针）：整条去掉退役树那一条（树表最末一条，第 50 批之前最末一条是 logged_ops）——树表 11 条、没有种类 13。
+/// 走读另红 I-1.1（树表头里的 key 区间末端还是退役树的号），照实钉住
 /// （正推探针改之前量到 `violated=["I-1.1"]`、I-9.16 成立：`research/prompts/m3-formal-code-b26-r1-sonnet-model/out/probe-i916-missing-kind.txt`）。
 #[test]
-fn a_tree_table_without_its_logged_operations_entry_reddens_the_tree_table_ordering_invariant_besides_the_walk(
+fn a_tree_table_without_its_retirement_entry_reddens_the_tree_table_ordering_invariant_besides_the_walk(
 ) {
-    let clean =
-        build_pool("known-bad-tree-table-without-the-logged-operations-entry").memory_pool();
+    let clean = build_pool("known-bad-tree-table-without-the-retirement-entry").memory_pool();
     assert_eq!(
         verdict(&clean, "I-9.16"),
         InvariantVerdict::Holds,
@@ -9556,8 +9643,8 @@ fn a_tree_table_without_its_logged_operations_entry_reddens_the_tree_table_order
         |bytes| {
             remove_tree_table_entry_of_kind(
                 bytes,
-                LOGGED_OPERATIONS_TREE_TABLE_ENTRY_INDEX,
-                TREE_KIND_LOGGED_OPERATIONS,
+                RETIREMENT_TREE_TABLE_ENTRY_INDEX,
+                singlefs_core::records::TREE_KIND_RETIREMENT,
             );
         },
         true,
@@ -9571,13 +9658,39 @@ fn a_tree_table_without_its_logged_operations_entry_reddens_the_tree_table_order
     assert_eq!(
         verdict(&image, "I-9.16"),
         InvariantVerdict::Violated(
-            "根（实例 1、txg 3）的树表有 10 条，种类 12 一条都没有（④ 树表非空时进树表的十一种各恰好一条）"
+            "根（实例 1、txg 3）的树表有 11 条，种类 13 一条都没有（④ 树表非空时进树表的十二种各恰好一条）"
                 .to_owned()
         ),
         "红在 ④ 那一道"
     );
 }
 
+/// I-9.16 ④ 十二种：整条去掉 logged_ops 那一条（第 50 批起它不是最末一条，树表头里的 key 区间不变）——树表 11 条、没有种类 12，只红 I-9.16。
+#[test]
+fn a_tree_table_without_its_logged_operations_entry_reddens_only_the_tree_table_ordering_invariant()
+{
+    let clean =
+        build_pool("known-bad-tree-table-without-the-logged-operations-entry").memory_pool();
+    let mut image = clean.clone();
+    mutate_unit(
+        &mut image,
+        TREE_TABLE,
+        |bytes| {
+            remove_tree_table_entry_of_kind(
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
+        "根（实例 1、txg 3）的树表有 11 条，种类 12 一条都没有（④ 树表非空时进树表的十二种各恰好一条）",
+    );
+}
+
 /// I-9.16 ③（攻方 f2）：覆盖写之后那一版（txg 4，不是从 mkfs 水位建树的那一版，I-5.5 不认它）的树表里 deadlist 改成种类 3。
 /// I-9.16 按每张树表第一次被走到时判，这一张是 B 自己的。
 #[test]
@@ -10415,7 +10528,7 @@ fn a_file_version_whose_root_record_carries_a_non_zero_allocation_record_tree_ro
     assert_eq!(
         verdict(&image, "I-5.4"),
         InvariantVerdict::Violated(
-            "根（实例 1、txg 3）的树表有 11 条（带树的一版），根记录里树表 0 条那一版的分配记录树根却不是全零：这一版有两棵分配记录树"
+            "根（实例 1、txg 3）的树表有 12 条（带树的一版），根记录里树表 0 条那一版的分配记录树根却不是全零：这一版有两棵分配记录树"
                 .to_owned()
         ),
         "红在最新那条根"
@@ -10530,6 +10643,462 @@ fn a_tombstone_tree_entry_with_an_unregistered_type_reddens_only_the_tombstone_e
     );
 }
 
+/// 退役树那五条（I-9.31–I-9.35）。
+const RETIREMENT_INVARIANTS: [&str; 5] = [
+    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+    EVERY_RETIREMENT_RECORD_IS_WELL_FORMED,
+    RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
+    RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+    RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+];
+
+/// 一份判定里 `invariant` 那一条。
+fn verdict_among(
+    verdicts: &[(&'static str, InvariantVerdict)],
+    invariant: &str,
+) -> InvariantVerdict {
+    verdicts
+        .iter()
+        .find(|(identifier, _)| *identifier == invariant)
+        .expect("清单里有")
+        .1
+        .clone()
+}
+
+/// 三个单元的文件按批大小 1 删到底（意图、三批）：最后一批删掉 inode 1 的记录，inode 树唯一那片叶容器空了、身份消失，
+/// 退役树开第一片容器、写一条 (那片叶的身份, 最后一批的 txg)（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 3 条）。
+/// 交回池、删之前那片叶的身份与最后一批那次发布。
+fn pool_with_the_three_unit_file_unlinked_to_the_end(
+    tag: &str,
+) -> (
+    BuiltPool,
+    singlefs_core::unit::PackedIdentity,
+    TransactionOutput,
+) {
+    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(tag);
+    let intent = common_deletion_in_progress::publish_the_intent(&mut pool);
+    let [leaf] = intent.inode_leaf_containers.as_slice() else {
+        panic!("意图那一版 inode 树一片叶容器")
+    };
+    let leaf_identity = leaf.contents.identity;
+    // 迭代上界：三个数据单元按批大小 1 删，三批删到底。
+    for _ in 0..2 {
+        common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool);
+    }
+    let final_batch =
+        common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool);
+    assert!(final_batch.deleted_inodes.is_empty(), "第三批是最后一批");
+    (pool, leaf_identity, final_batch)
+}
+
+/// 这一版退役树按叶序的全部退役记录。
+fn retirement_records_of(output: &TransactionOutput) -> Vec<RetirementRecord> {
+    output
+        .retirement_containers
+        .iter()
+        .flat_map(|container| container.contents.records.iter().copied())
+        .collect()
+}
+
+/// 写入那一侧与 checker 那一侧对着同一份干净镜像：唯一那片叶容器在最后一批身份消失，那次发布往退役树写恰好一条
+/// (那片叶的身份, 最后一批的 txg)；之前各版退役树零根。checker 上退役树那五条都真被评估过且成立，别的条也不红
+/// （判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 3、8 条）。
+#[test]
+fn the_final_batch_that_empties_the_only_leaf_writes_one_retirement_record_and_every_retirement_invariant_holds(
+) {
+    let (pool, leaf_identity, final_batch) =
+        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-clean");
+    assert!(
+        final_batch.inode_leaf_containers.is_empty(),
+        "最后一批之后 inode 树零条目根"
+    );
+    assert_eq!(
+        retirement_records_of(&final_batch),
+        [RetirementRecord {
+            retired: leaf_identity,
+            death_txg: final_batch.root.checkpoint_txg,
+        }],
+        "身份消失的那次发布写一条退役记录，死亡 txg = 这次发布的 txg"
+    );
+    let verdicts = check_pool_image(&pool.memory_pool());
+    let retirement_verdicts: Vec<(&str, InvariantVerdict)> = RETIREMENT_INVARIANTS
+        .iter()
+        .map(|invariant| (*invariant, verdict_among(&verdicts, invariant)))
+        .collect();
+    assert_eq!(
+        retirement_verdicts,
+        RETIREMENT_INVARIANTS
+            .iter()
+            .map(|invariant| (*invariant, InvariantVerdict::Holds))
+            .collect::<Vec<_>>(),
+        "退役树那五条都真被评估过且成立"
+    );
+    assert_eq!(
+        violated_invariants(&verdicts),
+        Vec::<&str>::new(),
+        "干净镜像上一条都不红：{verdicts:?}"
+    );
+}
+
+/// I-9.31（退役树条目身份与子头相符） 只红自己的那一形：删到底那一份上，退役树根第一条条目身份引用里的容器号加一、容器头不动。
+#[test]
+fn a_retirement_tree_entry_whose_identity_names_another_container_reddens_only_the_retirement_entry_invariant(
+) {
+    let (pool, _, _) =
+        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-entry-identity");
+    let mut image = pool.memory_pool();
+    change_the_first_entry_of_the_container_tree_root_of_kind(
+        &mut image,
+        TREE_KIND_RETIREMENT,
+        add_one_to_the_identity_container_number,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER],
+        "退役树根条目的身份与容器头不符：只红 I-9.31：{verdicts:?}"
+    );
+}
+
+/// 删到底之后建 1 个空 inode（新开一片叶容器，身份出生代 = 那次发布的 txg），再建 [`common_deletion_in_progress::PUBLISHES_THAT_TURN_THE_ROOT_RING_OVER`]
+/// 次：删的那几版与建新叶那几版都转出回退候选集（根环一圈 24 槽）。退役树那片容器在删到底那一版之后再没重写过。
+/// 交回池、新叶的身份、建新叶那次发布的 txg。
+fn pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
+    tag: &str,
+) -> (
+    BuiltPool,
+    singlefs_core::unit::PackedIdentity,
+    CheckpointTxg,
+) {
+    let (mut pool, _, _) = pool_with_the_three_unit_file_unlinked_to_the_end(tag);
+    let new_leaf_version =
+        common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+    let [new_leaf] = new_leaf_version.inode_leaf_containers.as_slice() else {
+        panic!("建 1 个空 inode 之后 inode 树一片叶容器")
+    };
+    let new_leaf_identity = new_leaf.contents.identity;
+    common_deletion_in_progress::turn_the_root_ring_over_by_creating_inodes(&mut pool);
+    (
+        pool,
+        new_leaf_identity,
+        new_leaf_version.root.checkpoint_txg,
+    )
+}
+
+/// 根环一圈 3 区 × 8 槽：最新那一版往前数这么多版之外的都不在回退候选集里。
+const ROOTS_IN_THE_RING: u64 = 24;
+
+/// I-9.32（退役记录合法） 只红自己的那一形：删的那几版转出候选集之后（I-9.35 在记录死的那一版上不再比），那条退役记录被退役身份的
+/// 打包记录类型 2 → 1。I-9.33、I-9.34 按身份比，类型 1 的身份既不重也不是活叶，照旧成立。
+#[test]
+fn a_retirement_record_naming_a_container_of_record_type_one_reddens_only_the_well_formed_retirement_record_invariant(
+) {
+    let (pool, _, _) =
+        pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
+            "known-bad-retirement-record-type",
+        );
+    let clean = pool.memory_pool();
+    let clean_verdicts = check_pool_image(&clean);
+    assert_eq!(
+        verdict_among(&clean_verdicts, EVERY_RETIREMENT_RECORD_IS_WELL_FORMED),
+        InvariantVerdict::Holds,
+        "改之前 I-9.32 真被评估过且成立"
+    );
+    assert_eq!(violated_invariants(&clean_verdicts), Vec::<&str>::new());
+    let mut image = clean.clone();
+    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
+        &mut image,
+        |record| common_deletion_in_progress::set_the_retired_record_type(record, 1),
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [EVERY_RETIREMENT_RECORD_IS_WELL_FORMED],
+        "被退役身份的类型不是 2：只红 I-9.32：{verdicts:?}"
+    );
+}
+
+/// I-9.33（谱系上的退役记录按身份不重复） 只红自己的那一形：删到底那一份上，退役容器里那条记录原样再追加一条。
+/// I-9.35 按身份集比，两条同身份收成一个，照旧成立。
+#[test]
+fn a_second_copy_of_the_retirement_record_reddens_only_the_distinct_retirement_records_invariant() {
+    let (pool, _, _) =
+        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-record-copy");
+    let mut image = pool.memory_pool();
+    common_deletion_in_progress::append_a_retirement_record_to_the_first_container_of_the_newest_root(
+        &mut image,
+        <[u8]>::to_vec,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT],
+        "谱系上两条退役记录指同一个身份：只红 I-9.33：{verdicts:?}"
+    );
+}
+
+/// I-9.34（谱系上的退役记录不指本版活叶） 只红自己的那一形：删的那几版与建新叶那几版转出候选集之后，那条退役记录改成指新叶的身份、
+/// 死亡 txg 改成建新叶那次发布的 txg + 1（在每一版父链上、大于新叶的容器出生代、不是候选集里哪一版的 txg）。I-9.32 的几道照旧成立，
+/// I-9.35 在死亡 txg 那一版与它的子版上都比不了（父版不在候选集里），只红 I-9.34。
+#[test]
+fn a_retirement_record_naming_the_live_leaf_reddens_only_the_no_live_leaf_invariant() {
+    let (pool, new_leaf_identity, new_leaf_txg) =
+        pool_with_the_file_unlinked_to_the_end_a_new_leaf_and_the_root_ring_turned_over(
+            "known-bad-retirement-record-live-leaf",
+        );
+    let death_txg = new_leaf_txg.0 + 1;
+    assert!(
+        death_txg + ROOTS_IN_THE_RING <= pool.output.root.checkpoint_txg.0,
+        "死亡 txg 那一版与它的子版都已转出候选集"
+    );
+    let new_leaf_identity_bytes = new_leaf_identity_in_record_order(new_leaf_identity);
+    let mut image = pool.memory_pool();
+    assert_eq!(
+        common_deletion_in_progress::identity_of_the_first_inode_leaf_of_the_newest_root(&image),
+        new_leaf_identity_bytes,
+        "最新那一版 inode 树唯一那片叶就是建新叶那次开的"
+    );
+    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
+        &mut image,
+        |record| {
+            record[..common_deletion_in_progress::RETIREMENT_RECORD_IDENTITY_BYTES]
+                .copy_from_slice(&new_leaf_identity_bytes);
+            common_deletion_in_progress::set_retirement_record_field(
+                record,
+                common_deletion_in_progress::RETIREMENT_RECORD_DEATH_TXG_OFFSET,
+                death_txg,
+            );
+        },
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF],
+        "谱系上的退役记录指这一版的活叶：只红 I-9.34：{verdicts:?}"
+    );
+}
+
+/// 一个打包记录容器身份按退役记录里被退役身份那 26 字节排（出生树 8、类型 2、容器号 8、容器出生代 8，小端）。
+fn new_leaf_identity_in_record_order(identity: singlefs_core::unit::PackedIdentity) -> Vec<u8> {
+    RetirementRecord {
+        retired: identity,
+        death_txg: CheckpointTxg(0),
+    }
+    .to_bytes()[..common_deletion_in_progress::RETIREMENT_RECORD_IDENTITY_BYTES]
+        .to_vec()
+}
+
+/// I-9.35（退役完备）漏写那一形，只红自己：删到底那一份上，那条退役记录的死亡 txg 往前挪一（最后一批 → 第二批的 txg）。
+/// 最后一批那一版比父版少了那片叶、死于这一版的记录一条都没有；第二批那一版看不到这片容器（退役树那时零根）；I-9.32 的几道照旧成立。
+#[test]
+fn a_retirement_record_moved_off_the_txg_the_leaf_died_at_reddens_only_the_retirement_completeness_invariant(
+) {
+    let (pool, _, final_batch) =
+        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-record-missing");
+    let mut image = pool.memory_pool();
+    common_deletion_in_progress::change_the_first_retirement_record_of_the_newest_root(
+        &mut image,
+        |record| {
+            common_deletion_in_progress::set_retirement_record_field(
+                record,
+                common_deletion_in_progress::RETIREMENT_RECORD_DEATH_TXG_OFFSET,
+                final_batch.root.checkpoint_txg.0 - 1,
+            );
+        },
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION],
+        "身份消失的那一版没记退役：只红 I-9.35：{verdicts:?}"
+    );
+}
+
+/// I-9.35（退役完备）多写那一形，只红自己：删到底那一份上，退役容器追加一条与那条同死亡 txg、容器号加一（这一版父版里没有的身份）。
+/// I-9.33 按身份比不重、I-9.34 这一版没有活叶，照旧成立。
+#[test]
+fn a_retirement_record_naming_a_leaf_the_parent_version_did_not_have_reddens_only_the_retirement_completeness_invariant(
+) {
+    let (pool, _, _) =
+        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-record-extra");
+    let mut image = pool.memory_pool();
+    common_deletion_in_progress::append_a_retirement_record_to_the_first_container_of_the_newest_root(
+        &mut image,
+        |first| {
+            let mut extra = first.to_vec();
+            let container = common_deletion_in_progress::retirement_record_field(
+                &extra,
+                common_deletion_in_progress::RETIREMENT_RECORD_CONTAINER_OFFSET,
+            );
+            common_deletion_in_progress::set_retirement_record_field(
+                &mut extra,
+                common_deletion_in_progress::RETIREMENT_RECORD_CONTAINER_OFFSET,
+                container + 1,
+            );
+            extra
+        },
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION],
+        "死于这一版的退役记录指父版里没有的身份：只红 I-9.35：{verdicts:?}"
+    );
+}
+
+/// 合法的回退不红（判决第三节第 5 条「回退照抄现行那一版的退役树」、第 8 条 I-9.33 只比谱系上的）：删到底（退役一条 (叶, F)）→
+/// 回退到意图那一版（那片叶又活了，退役树照抄、仍是那一条）→ 再删到底（同一片叶再死一次，追加 (叶, F′)）。
+/// 两条同身份，但 F 不在 F′ 那一版的父链上；每一版 checker 都一条不红，退役树那五条都成立。
+#[test]
+fn unlinking_rolling_back_and_unlinking_again_writes_a_second_record_of_the_same_leaf_and_no_invariant_reddens(
+) {
+    let mut pool = common_deletion_in_progress::pool_with_a_three_unit_file(
+        "known-bad-retirement-rollback-and-again",
+    );
+    let intent = common_deletion_in_progress::publish_the_intent(&mut pool);
+    let [leaf] = intent.inode_leaf_containers.as_slice() else {
+        panic!("意图那一版 inode 树一片叶容器")
+    };
+    let leaf_identity = leaf.contents.identity;
+    // 迭代上界：三批删到底。
+    let mut first_final_batch = intent.clone();
+    for _ in 0..3 {
+        first_final_batch =
+            common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool);
+    }
+    let rollback = common_deletion_in_progress::roll_back_to(&mut pool, &intent);
+    assert_eq!(
+        retirement_records_of(&rollback),
+        retirement_records_of(&first_final_batch),
+        "回退那一版退役树照抄现行那一版"
+    );
+    assert_eq!(
+        rollback
+            .inode_leaf_containers
+            .iter()
+            .map(|container| container.contents.identity)
+            .collect::<Vec<_>>(),
+        [leaf_identity],
+        "回退之后那片叶又在 inode 树里"
+    );
+    assert_eq!(
+        violated_invariants(&check_pool_image(&pool.memory_pool())),
+        Vec::<&str>::new(),
+        "回退那一版一条都不红"
+    );
+    // 迭代上界：三批删到底。
+    let mut second_final_batch = rollback.clone();
+    for _ in 0..3 {
+        second_final_batch =
+            common_deletion_in_progress::publish_the_next_step_in_batches_of_one(&mut pool);
+    }
+    assert!(second_final_batch.deleted_inodes.is_empty(), "再删也是三批");
+    assert_eq!(
+        retirement_records_of(&second_final_batch),
+        [
+            RetirementRecord {
+                retired: leaf_identity,
+                death_txg: first_final_batch.root.checkpoint_txg,
+            },
+            RetirementRecord {
+                retired: leaf_identity,
+                death_txg: second_final_batch.root.checkpoint_txg,
+            },
+        ],
+        "同一片叶死第二次再追加一条，第一条不动"
+    );
+    let verdicts = check_pool_image(&pool.memory_pool());
+    assert_eq!(
+        violated_invariants(&verdicts),
+        Vec::<&str>::new(),
+        "再删到底那一版一条都不红：{verdicts:?}"
+    );
+    for invariant in RETIREMENT_INVARIANTS {
+        assert_eq!(
+            verdict_among(&verdicts, invariant),
+            InvariantVerdict::Holds,
+            "{invariant} 真被评估过且成立"
+        );
+    }
+}
+
+/// I-9.35 的父版取谱系（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 8 条、正推 K5.1：这一版谱系树里有 T_rb 等于
+/// 这一版 txg 的记录时父版是那条的 T_old，否则 txg − 1）：删到底（inode 树零条目根）之后建 1 个空 inode（开一片新叶），再回退到删到底那一版，
+/// 回退那一版没有那片新叶。父版取 T_old（删到底那一版，同样没有叶），没有叶身份消失、也没有退役记录；按 txg − 1 取父版会拿建新叶那一版比、
+/// 判它漏记一片。回退那一版一条都不红，I-9.35 真被评估过且成立。
+#[test]
+fn rolling_back_past_the_birth_of_a_leaf_compares_with_the_rollback_target_and_no_invariant_reddens(
+) {
+    let (mut pool, _, final_batch) = pool_with_the_three_unit_file_unlinked_to_the_end(
+        "known-bad-retirement-rollback-past-a-leaf-birth",
+    );
+    let new_leaf_version =
+        common_deletion_in_progress::publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+    assert_eq!(
+        new_leaf_version.inode_leaf_containers.len(),
+        1,
+        "建 1 个空 inode 开一片新叶"
+    );
+    let rollback = common_deletion_in_progress::roll_back_to(&mut pool, &final_batch);
+    assert!(
+        rollback.inode_leaf_containers.is_empty(),
+        "回退那一版 inode 树零条目根"
+    );
+    assert_eq!(
+        retirement_records_of(&rollback),
+        retirement_records_of(&final_batch),
+        "回退照抄现行那一版的退役树，不写退役记录"
+    );
+    let verdicts = check_pool_image(&pool.memory_pool());
+    assert_eq!(
+        violated_invariants(&verdicts),
+        Vec::<&str>::new(),
+        "回退到新叶出生之前那一版一条都不红：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict_among(
+            &verdicts,
+            RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION
+        ),
+        InvariantVerdict::Holds,
+        "I-9.35 真被评估过且成立"
+    );
+}
+
+/// I-9.27 种类 13（退役树，第 50 批；头 ID 写 inode 树的树 ID，同墓碑树）：删到底那一份上，最新那一版树表里退役树那一条的头 ID 从 inode 树的
+/// 树 ID 改成 0、配不上任何一条 inode 树条目。只红 I-9.27；退役记录对叶容器的那两判（I-9.34、I-9.35）在那一版判不了、理由写头 ID 配不上，
+/// 不按「这棵 inode 树没有退役记录」判漏记。
+#[test]
+fn a_retirement_tree_entry_whose_head_identifier_names_no_inode_tree_reddens_only_the_head_identifier_invariant(
+) {
+    let (pool, _, _) =
+        pool_with_the_three_unit_file_unlinked_to_the_end("known-bad-retirement-head-zero");
+    let mut image = pool.memory_pool();
+    let tree_table_slot = common_deletion_in_progress::tree_table_slot_of_the_newest_root(&image);
+    common_deletion_in_progress::set_the_head_identifier_of_the_tree_table_entry_of_kind_at(
+        &mut image,
+        tree_table_slot,
+        TREE_KIND_RETIREMENT,
+        0,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES],
+        "退役树条目的头 ID 配不上 inode 树：只红 I-9.27：{verdicts:?}"
+    );
+    for invariant in [
+        RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
+        RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
+    ] {
+        let found = verdict_among(&verdicts, invariant);
+        assert!(
+            matches!(found, InvariantVerdict::NotApplicable(reason) if reason.contains("头 ID 配不上")),
+            "{invariant} 在头 ID 配不上的那一版判不了：{found:?}"
+        );
+    }
+}
+
 /// I-9.17（谱系树条目身份与子头相符） 只红自己的那一形：挂着的时候回退到 A 之后（回退那次发布写出谱系树），谱系树根第一条条目身份引用里的
 /// 容器号加一、容器头不动。
 #[test]
@@ -10573,9 +11142,9 @@ fn a_tombstone_container_whose_record_width_is_not_forty_reddens_only_the_packed
     );
 }
 
-/// I-1.7 记录宽那一步对类型 5（谱系，登记 16）：回退到 A 之后，谱系容器头里的记录宽 16 → 15。只红 I-1.7。
+/// I-1.7 记录宽那一步对类型 5（谱系，登记 24）：回退到 A 之后，谱系容器头里的记录宽 24 → 15。只红 I-1.7（I-9.30 读不出谱系，那一判报射程里判不了）。
 #[test]
-fn a_lineage_container_whose_record_width_is_not_sixteen_reddens_only_the_packed_container_invariant(
+fn a_lineage_container_whose_record_width_is_not_the_registered_one_reddens_only_the_packed_container_invariant(
 ) {
     let rolled_back = image_after_rolling_back_to_the_first_root("known-bad-lineage-record-width");
     let mut image = rolled_back.image.clone();
@@ -10588,7 +11157,7 @@ fn a_lineage_container_whose_record_width_is_not_sixteen_reddens_only_the_packed
     assert_eq!(
         violated_invariants(&verdicts),
         ["I-1.7"],
-        "谱系容器的记录宽不是 16：只红 I-1.7：{verdicts:?}"
+        "谱系容器的记录宽不是 24：只红 I-1.7：{verdicts:?}"
     );
 }
 
@@ -10867,8 +11436,10 @@ fn capacities_of_one_record_and_two_entries(
             ),
         lineage_container: singlefs_core::lineage_tree::LineageContainerCapacity::CappedForTests(
             singlefs_core::lineage_tree::LineageRecordsPerContainerCappedForTests::new(1)
-                .expect("1..=2039"),
+                .expect("1..=1359"),
         ),
+        retirement_container:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         node_entries: singlefs_core::container_tree::ContainerTreeNodeEntries::CappedForTests(
             singlefs_core::container_tree::ContainerTreeNodeEntriesCappedForTests::new(2)
                 .expect("2..=135"),
@@ -12314,7 +12885,7 @@ fn a_tombstone_container_with_a_wrong_record_width_beside_a_copy_verifying_as_a_
     );
 }
 
-/// 攻方 1.1 谱系那一形（I-1.7）：回退到 A 之后，谱系容器头里的记录宽 16 → 15。
+/// 攻方 1.1 谱系那一形（I-1.7）：回退到 A 之后，谱系容器头里的记录宽 24 → 23。
 #[test]
 fn a_lineage_container_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device(
 ) {
@@ -13552,7 +14123,7 @@ fn an_inode_leaf_with_a_wrong_record_width_reddens_the_packed_container_invarian
     );
 }
 
-/// F-KEY 谱系那一形：回退到 A 之后，谱系容器记录宽 16 → 15，谱系树根第一条条目子指针的一条位置条目指着 inode 树根（先走过，
+/// F-KEY 谱系那一形：回退到 A 之后，谱系容器记录宽 24 → 23，谱系树根第一条条目子指针的一条位置条目指着 inode 树根（先走过，
 /// 回退那一版照抄 A 的 inode 树，A 那条根也引用它）。读回来的那片容器（这一版新写的）的诞生代号记在它读回的那个落点上，不盖掉 inode 树根的
 /// （攻方 allrbmap，判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 1 条）：两种盘序红的同一批，I-4.9 不多红
 /// （改之前位置条目 0 那一形多红 I-4.9，攻方 `research/prompts/m3-batch32-code-r1-opus-output.md` 第 1.2 节 today 臂那 6 对之一）。
@@ -14171,7 +14742,7 @@ fn a_tombstone_container_whose_location_names_the_shared_extent_root_judges_both
 }
 
 /// 谱系撞 inode 树根、容器不改（攻方 `base=lineage walked=inode variant=good`）：回退到 A 之后，回退那一版照抄 A 的 inode 树。
-/// 只改诞生代号那张表、不改 I-3.5 折叠表时这一形改红 I-3.5（攻方第 1.3 节）。记录宽 16 → 15 那一形见
+/// 只改诞生代号那张表、不改 I-3.5 折叠表时这一形改红 I-3.5（攻方第 1.3 节）。记录宽 24 → 23 那一形见
 /// [`a_lineage_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before`]。
 #[test]
 fn a_lineage_container_whose_location_names_the_shared_inode_root_judges_both_device_orders_alike()
@@ -14188,7 +14759,7 @@ fn a_lineage_container_whose_location_names_the_shared_inode_root_judges_both_de
     );
 }
 
-/// 谱系撞 extent 树根、容器记录宽 16 → 15（攻方 `base=lineage walked=extent variant=bad`）。
+/// 谱系撞 extent 树根、容器记录宽 24 → 23（攻方 `base=lineage walked=extent variant=bad`）。
 #[test]
 fn a_lineage_container_with_a_wrong_record_width_whose_location_names_the_shared_extent_root_judges_both_device_orders_alike(
 ) {
@@ -15194,3 +15765,236 @@ fn a_root_pointer_whose_two_copies_do_not_read_back_and_whose_first_location_nam
         "两份照判 I-2.1 红，走读失败记下、遍历读不出红 I-4.8：{verdicts:?}"
     );
 }
+
+// ─── I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，W3 的 C2aL 形态，第 49 批） ───
+
+/// 谱系记录里 H_cur 那 8 字节的偏移（T_rb 8、T_old 8 之后）与一片打包记录单元的记录区起点（D18（块里携带什么信息） 已定项 16）。
+const LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET: usize = 16;
+const PACKED_UNIT_FIRST_RECORD_OFFSET: usize = 136;
+/// 记账树叶（key 宽 22）里条目数的偏移、第一条条目的偏移、一条条目宽与值、代段、设备段的偏移，单元头里声明长度的偏移
+/// （偏移口径同 `set_accounting`）。
+const ACCOUNTING_LEAF_ENTRY_COUNT_OFFSET: usize = 82 + 44;
+const ACCOUNTING_LEAF_FIRST_ENTRY_OFFSET: usize = 159;
+const ACCOUNTING_ENTRY_WIDTH: usize = 34;
+const ACCOUNTING_ENTRY_DEVICE_OFFSET: usize = 10;
+const ACCOUNTING_ENTRY_GENERATION_OFFSET: usize = 14;
+const ACCOUNTING_ENTRY_VALUE_OFFSET: usize = 22;
+const UNIT_DECLARED_LENGTH_OFFSET_OF_AN_INDEX_NODE: usize = 8;
+/// inode 号水位（D5（快照 / 空间记账机制） 已定项 4 第 12 项）不带设备维。
+const INODE_NUMBER_WATERMARK_STATISTIC: u16 = 12;
+const NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;
+
+/// 合法镜像：崩溃恢复抛弃建过三个 inode（水位 5）的那一版、它的根读得出（`common::rollback_after_a_recovery_abandoned_a_root_that_created_inodes`），
+/// 实例 2 再建一个（2 号重发，被抛弃那段里发过的号，D8（核心索引结构） 已定项 6 射程允许；水位 3），再回退到 A（谱系记录 H_cur 3）。
+/// 一条违例都没有：I-9.30 真被判过且成立（台阶 (3, 2)、(重发那次, 3)；谱系上 T_rb − 1 以前最大的台阶 3 等于 H_cur；被抛弃那段的台阶随它的根留在
+/// 盘上、不在最新根的记账里），I-3.13 真被判过且成立（台阶行代段早于这一版，跳过；每次发布写的那一行代段等于这一版的 txg）。
+#[test]
+fn a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_and_created_again_holds_the_watermark_step_and_generation_invariants(
+) {
+    let history = common::rollback_after_a_recovery_abandoned_a_root_that_created_inodes(
+        "known-bad-watermark-steps-legal-cross-abandon-reissue",
+    );
+    let verdicts = check_pool_image(&history.pool.memory_pool());
+    assert_eq!(
+        violated_invariants(&verdicts),
+        Vec::<&str>::new(),
+        "合法镜像一条违例都没有：{verdicts:?}"
+    );
+    for invariant in [
+        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
+        ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
+    ] {
+        assert_eq!(
+            verdicts
+                .iter()
+                .find(|(name, _)| *name == invariant)
+                .map(|(_, found)| found),
+            Some(&InvariantVerdict::Holds),
+            "{invariant} 真被判过且成立"
+        );
+    }
+}
+
+/// [`a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_and_created_again_holds_the_watermark_step_and_generation_invariants`]
+/// 那份合法镜像，最新根（回退那一版）谱系树第一片容器里第一条谱系记录的 H_cur 改成 `watermark_rolled_back_from`（重封、补到根）。
+fn image_after_the_legal_cross_abandon_rollback_with_the_watermark_rolled_back_from(
+    tag: &str,
+    watermark_rolled_back_from: u64,
+) -> MemoryPool {
+    let history = common::rollback_after_a_recovery_abandoned_a_root_that_created_inodes(tag);
+    let mut image = history.pool.memory_pool();
+    let lineage_container =
+        common_deletion_in_progress::first_container_slot_of_the_tree_of_kind_of_the_newest_root(
+            &image,
+            TREE_KIND_LINEAGE,
+        );
+    let offset = PACKED_UNIT_FIRST_RECORD_OFFSET
+        + LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET;
+    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+        &mut image,
+        lineage_container,
+        32_768,
+        |bytes| {
+            assert_eq!(
+                u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8")),
+                3,
+                "回退那一版谱系记录的 H_cur 是 cur 的水位 3"
+            );
+            bytes[offset..offset + 8].copy_from_slice(&watermark_rolled_back_from.to_le_bytes());
+        },
+    );
+    image
+}
+
+/// I-9.30 ④（谱系那一判按「等于」比，W3 判决 `research/prompts/m3-w3-r2-main-verification.md` 第二节攻方 K3）：合法镜像上谱系记录的 H_cur 3 → 2。
+/// T_rb − 1 以前最大的台阶是 3，H_cur 小于它——按「被台阶盖住」比（max ≥ H_cur）放过这一形，按「等于」比判红。只红 I-9.30。
+#[test]
+fn a_lineage_record_whose_watermark_rolled_back_from_is_below_the_last_step_reddens_only_the_watermark_step_invariant(
+) {
+    let image = image_after_the_legal_cross_abandon_rollback_with_the_watermark_rolled_back_from(
+        "known-bad-watermark-steps-lineage-watermark-below-the-step",
+        2,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE],
+        "谱系记录的 H_cur 比台阶小：只红 I-9.30：{verdicts:?}"
+    );
+}
+
+/// I-9.30 ④ 的「H_cur 为 0 也判红」（D18（块里携带什么信息） 已定项 24 第 5 条）：合法镜像上谱系记录的 H_cur 3 → 0。只红 I-9.30。
+#[test]
+fn a_lineage_record_whose_watermark_rolled_back_from_is_zero_reddens_only_the_watermark_step_invariant(
+) {
+    let image = image_after_the_legal_cross_abandon_rollback_with_the_watermark_rolled_back_from(
+        "known-bad-watermark-steps-lineage-watermark-zero",
+        0,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE],
+        "谱系记录的 H_cur 为 0：只红 I-9.30：{verdicts:?}"
+    );
+}
+
+/// 最新根记账树（根兼叶）里 inode 号水位那一统计量、代段 `generation` 的那一行：`change` 拿到那一行的 34 字节改值，`None` 把这一行从叶里拿掉
+/// （后面的行往前挪、条目数与声明长度各减一行）；重封、补到根。
+fn change_the_inode_number_watermark_row_of_the_newest_root(
+    image: &mut MemoryPool,
+    generation: u64,
+    new_value: Option<u64>,
+) {
+    let slot = common_deletion_in_progress::accounting_tree_slot_of_the_newest_root(image);
+    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+        image,
+        slot,
+        16_384,
+        |node| {
+            assert_eq!(node[50], 0, "这几份镜像上记账树只有根兼叶");
+            let read_u16_at = |unit_bytes: &[u8], offset: usize| {
+                u16::from_le_bytes(unit_bytes[offset..offset + 2].try_into().expect("2"))
+            };
+            let row_count = usize::from(read_u16_at(node, ACCOUNTING_LEAF_ENTRY_COUNT_OFFSET));
+            let row_index = (0..row_count)
+                .find(|index| {
+                    let row = ACCOUNTING_LEAF_FIRST_ENTRY_OFFSET + index * ACCOUNTING_ENTRY_WIDTH;
+                    read_u16_at(node, row) == INODE_NUMBER_WATERMARK_STATISTIC
+                        && node[row + ACCOUNTING_ENTRY_DEVICE_OFFSET
+                            ..row + ACCOUNTING_ENTRY_DEVICE_OFFSET + 4]
+                            == NO_DEVICE_DIMENSION.to_le_bytes()
+                        && node[row + ACCOUNTING_ENTRY_GENERATION_OFFSET
+                            ..row + ACCOUNTING_ENTRY_GENERATION_OFFSET + 8]
+                            == generation.to_le_bytes()
+                })
+                .unwrap_or_else(|| panic!("记账树里有 inode 号水位代段 {generation} 那一行"));
+            let row = ACCOUNTING_LEAF_FIRST_ENTRY_OFFSET + row_index * ACCOUNTING_ENTRY_WIDTH;
+            match new_value {
+                Some(value) => node
+                    [row + ACCOUNTING_ENTRY_VALUE_OFFSET..row + ACCOUNTING_ENTRY_VALUE_OFFSET + 8]
+                    .copy_from_slice(&value.to_le_bytes()),
+                None => {
+                    assert!(
+                        row_index > 0 && row_index + 1 < row_count,
+                        "这一行不在首末：拿掉它头里的 key 区间不变"
+                    );
+                    let end =
+                        ACCOUNTING_LEAF_FIRST_ENTRY_OFFSET + row_count * ACCOUNTING_ENTRY_WIDTH;
+                    node.copy_within(row + ACCOUNTING_ENTRY_WIDTH..end, row);
+                    node[end - ACCOUNTING_ENTRY_WIDTH..end].fill(0);
+                    node[ACCOUNTING_LEAF_ENTRY_COUNT_OFFSET
+                        ..ACCOUNTING_LEAF_ENTRY_COUNT_OFFSET + 2]
+                        .copy_from_slice(
+                            &u16::try_from(row_count - 1).expect("行数").to_le_bytes(),
+                        );
+                    node[UNIT_DECLARED_LENGTH_OFFSET_OF_AN_INDEX_NODE
+                        ..UNIT_DECLARED_LENGTH_OFFSET_OF_AN_INDEX_NODE + 2]
+                        .copy_from_slice(
+                            &u16::try_from((row_count - 1) * ACCOUNTING_ENTRY_WIDTH)
+                                .expect("声明长度")
+                                .to_le_bytes(),
+                        );
+                }
+            }
+        },
+    );
+}
+
+/// I-9.30 ④ 管「台阶行少了一级」：合法镜像上最新根记账里重发那次（2 号重发、水位 3）留的那级台阶拿掉。谱系记录的 H_cur 3 对不上
+/// T_rb − 1 以前最大的台阶 2，判红；台阶行不算进记账行数（恢复读得回这一版）、I-3.13 不判台阶行。只红 I-9.30。
+#[test]
+fn an_inode_number_watermark_step_dropped_from_the_accounting_tree_reddens_only_the_watermark_step_invariant(
+) {
+    let history = common::rollback_after_a_recovery_abandoned_a_root_that_created_inodes(
+        "known-bad-watermark-steps-a-step-dropped",
+    );
+    let mut image = history.pool.memory_pool();
+    change_the_inode_number_watermark_row_of_the_newest_root(
+        &mut image,
+        history.created_again.root.checkpoint_txg.0,
+        None,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE],
+        "台阶行少了重发那一级：只红 I-9.30：{verdicts:?}"
+    );
+}
+
+/// I-9.30 ① 管「水位低于台阶」：第一个文件之后建两个空 inode（txg 4，水位 4）、再覆盖写一次（txg 5）——最新根记账里台阶 (3, 2)、(4, 4) 与这一版那一行 (5, 4)。
+/// 台阶 (4, 4) 的水位抬到 5：记账里的水位 4 低于它，判红；那一级之后出生的记录一条都没有（覆盖写不建 inode），② 不红；水位 4 仍大于树里最大的 key 3，
+/// I-9.6 不红；台阶行 I-3.13 不判。只红 I-9.30。
+#[test]
+fn an_inode_number_watermark_step_raised_above_the_watermark_reddens_only_the_watermark_step_invariant(
+) {
+    let mut pool = common_deletion_in_progress::pool_with_new_inodes(
+        "known-bad-watermark-steps-a-step-above-the-watermark",
+        2,
+    );
+    let created = pool.output.clone();
+    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, b"after the new inodes");
+    assert_eq!(
+        (
+            created.root.checkpoint_txg.0,
+            created.inode_number_watermark(),
+            pool.output.inode_number_watermark()
+        ),
+        (4, 4, 4),
+        "建两个空 inode 那一版 txg 4、水位 4；覆盖写不改水位"
+    );
+    let mut image = pool.memory_pool();
+    assert_eq!(
+        violated_invariants(&check_pool_image(&image)),
+        Vec::<&str>::new(),
+        "改之前一条都不红"
+    );
+    change_the_inode_number_watermark_row_of_the_newest_root(&mut image, 4, Some(5));
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE],
+        "台阶抬到水位之上：只红 I-9.30：{verdicts:?}"
+    );
+}
diff --git a/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs b/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
index 1ec1003..d18e345 100644
--- a/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
+++ b/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
@@ -420,8 +420,10 @@ fn account_for_units_placed_by_hand(
             adjust_accounting_row(rows, STATISTIC_FREE_BYTES, device, -bytes);
         }
         if let Some(watermark) = inode_watermark {
+            // 台阶行与每次发布写的那一行同统计量、同设备段，代段更早、排在前面：改的是排在最后（代段最大）的那一行。
             let row = rows
                 .iter_mut()
+                .rev()
                 .find(|row| {
                     row[ACCOUNTING_ROW_STATISTIC_OFFSET..ACCOUNTING_ROW_STATISTIC_OFFSET + 2]
                         == STATISTIC_INODE_WATERMARK.to_le_bytes()
@@ -941,20 +943,21 @@ fn two_leaf_containers_in_key_order_hold_every_invariant() {
     assert_eq!(verdict_of(&image, "I-9.4"), InvariantVerdict::Holds);
 }
 
-/// 第二片的容器号 8：它装的 20、30 都 ≥ 8（第一句成立），容器号 1 < 8（第二句成立），而左边容器的最大 key 10 不小于右边的容器号 8
-/// （第三句不成立）——这正是审阅那一例：I-9.12 的两条不等式都过，只有 I-9.4 第三句说话。改之前 checker 只判第一句，全绿。
+/// 第二片的容器号 8：它装的 20、30 都 ≥ 8（第一句成立），容器号 1 < 8（第二句成立），左边容器的最大 key 10 不小于右边的容器号 8，
+/// 分隔 key 落在左片最大 key 之后（I-9.12 的两条不等式都过）——这正是左从右借之后右片保留身份、分隔 key 改成它这一版最小 key 的形状（N1）。
+/// 原来的第三句（左容器的最大 key < 右容器号）随 N1 删掉（用户 2026-10-08 定案 X4，判决 `research/prompts/m3-leaf-death-r3-main-verification.md`
+/// 第五节）：这张镜像一条都不红，I-9.4 照样真被评估过、成立。
 #[test]
-fn a_right_container_number_not_above_the_largest_key_on_its_left_reddens_only_the_container_number_invariant(
-) {
+fn a_borrowed_right_container_numbered_below_the_largest_key_on_its_left_holds_every_invariant() {
     let pool = product_capacity_pool();
     let image = image_with_two_leaf_containers(&pool, 8);
     let red = violations(&image);
-    assert_eq!(violated_names(&red), ["I-9.4"], "只该红 I-9.4：{red:?}");
-    assert!(
-        red[0].1.contains("最大 key"),
-        "红在「左容器的最大 key < 右容器号」那一句：{}",
-        red[0].1
+    assert_eq!(
+        red,
+        Vec::new(),
+        "借之后的两片叶是合法形状，一条都不许红：{red:?}"
     );
+    assert_eq!(verdict_of(&image, "I-9.4"), InvariantVerdict::Holds);
 }
 
 /// 第二片的容器号 0：沿叶序容器号 1 → 0 往回走（第二句不成立；第三句跟着不成立）。违例说明取第一处，要是第二句那一格。
diff --git a/crates/singlefs-harness/tests/common/mod.rs b/crates/singlefs-harness/tests/common/mod.rs
index a22e7c2..9817848 100644
--- a/crates/singlefs-harness/tests/common/mod.rs
+++ b/crates/singlefs-harness/tests/common/mod.rs
@@ -762,6 +762,171 @@ pub fn abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before_with_
     pool: &mut BuiltPool,
     newest: &TransactionOutput,
     shadow_ledger: singlefs_core::mount::ShadowLedger,
+) -> singlefs_core::mount::Mounted {
+    let data_unit_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768");
+    let data_units_of_the_newest = newest
+        .data_pointers
+        .iter()
+        .flat_map(|pointer| pointer.locations)
+        .map(|location| {
+            (
+                location.device,
+                location.slot.to_device_offset(),
+                data_unit_bytes,
+            )
+        })
+        .collect();
+    abandon_the_newest_root_by_a_recovery_hiding_its_root_slot_and(
+        pool,
+        newest,
+        data_units_of_the_newest,
+        shadow_ledger,
+    )
+}
+
+/// 同 [`abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`]，暂时读不出的是 `newest` 的根槽与它那条末条记录点名的打包记录单元
+/// （码 3，两份）：没写数据单元的发布（建空 inode 那一种）也被抛弃得了——它那条记录点名的 inode 叶容器两份都读不出，施加前验点名单元失败、不施加
+/// （D23（journal 的角色与格式） 已定项 14 前缀第四条）。影子账开着；交回那次挂载。
+pub fn abandon_the_newest_root_by_a_recovery_hiding_the_packed_units_its_record_names(
+    pool: &mut BuiltPool,
+    newest: &TransactionOutput,
+) -> singlefs_core::mount::Mounted {
+    let packed_units_named_by_the_newest = newest
+        .record
+        .named
+        .iter()
+        .filter(|named| named.unit_class == UNIT_CLASS_PACKED)
+        .flat_map(|named| {
+            named.locations.iter().map(|location| {
+                (
+                    location.device,
+                    location.slot.to_device_offset(),
+                    named_unit_bytes(named.unit_class),
+                )
+            })
+        })
+        .collect::<Vec<_>>();
+    assert!(
+        !packed_units_named_by_the_newest.is_empty(),
+        "`newest` 那条记录点名了至少一片打包记录单元（建 inode 的发布重写 inode 叶容器）"
+    );
+    abandon_the_newest_root_by_a_recovery_hiding_its_root_slot_and(
+        pool,
+        newest,
+        packed_units_named_by_the_newest,
+        singlefs_core::mount::ShadowLedger::On,
+    )
+}
+
+/// W3 那一段历史（D23（journal 的角色与格式） 已定项 14「水位」一格的 Opt2、D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」）：
+/// 第一个文件那一版 A（实例 1，inode 号水位 2）之后建三个空 inode 的那一版 `abandoned`（号 2、3、4，水位 5）；崩溃恢复抛弃它
+/// （[`abandon_the_newest_root_by_a_recovery_hiding_the_packed_units_its_record_names`]：恢复落到 A，实例 2 写行、暖机，再把它的根槽与
+/// inode 叶容器写回——它的根读得出、从盘上重建得出水位 5，按实例 2 的实例表判是被抛弃的）；实例 2 再建一个空 inode 的那一版 `created_again`
+/// （号 2 又发一次：被抛弃那段里发过的号，D8（核心索引结构） 已定项 6 射程允许重发；水位 3）；挂着的时候管理员回退到 A 的那一版 `rollback`。
+pub struct RollbackAfterARecoveryAbandonedARootThatCreatedInodes {
+    pub pool: BuiltPool,
+    pub first_file_version: TransactionOutput,
+    pub abandoned: TransactionOutput,
+    pub created_again: TransactionOutput,
+    pub rollback: TransactionOutput,
+}
+
+/// 搭 [`RollbackAfterARecoveryAbandonedARootThatCreatedInodes`] 那一段历史。搭建自证：回退之前被抛弃那一版的根在根环里读得出、
+/// 从盘上重建得出水位 5（不然「回退只取没被抛弃的根」那一判在这段历史上量不出什么）。
+pub fn rollback_after_a_recovery_abandoned_a_root_that_created_inodes(
+    tag: &str,
+) -> RollbackAfterARecoveryAbandonedARootThatCreatedInodes {
+    const NEW_INODES_OF_THE_ABANDONED_PUBLISH: u64 = 3;
+    let mut pool = build_pool(tag);
+    let first_file_version = pool.output.clone();
+    let publish_new_inodes_in_process =
+        |pool_to_publish_on: &mut BuiltPool, count: u64, instance: InstanceGeneration| {
+            let publish_parameters = parameters();
+            let previous = pool_to_publish_on.output.clone();
+            let devices = pool_to_publish_on.devices.as_mut().expect("镜像还开着");
+            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+            let output = singlefs_core::transaction::publish_new_inodes(
+                &mut writer,
+                &mut pool_to_publish_on.allocator,
+                &previous,
+                count,
+                FIXED_WRITE_TIME_SECONDS + 60,
+                instance,
+            )
+            .expect("建空 inode");
+            pool_to_publish_on.output = output.clone();
+            output
+        };
+    let abandoned = publish_new_inodes_in_process(
+        &mut pool,
+        NEW_INODES_OF_THE_ABANDONED_PUBLISH,
+        InstanceGeneration(1),
+    );
+    assert_eq!(
+        abandoned.inode_number_watermark(),
+        2 + NEW_INODES_OF_THE_ABANDONED_PUBLISH,
+        "建三个空 inode（号 2、3、4）之后水位 5"
+    );
+    let mounted = abandon_the_newest_root_by_a_recovery_hiding_the_packed_units_its_record_names(
+        &mut pool, &abandoned,
+    );
+    assert_eq!(
+        (
+            mounted.output.effective_root.instance,
+            mounted.output.effective_root.checkpoint_txg
+        ),
+        (
+            first_file_version.root.instance,
+            first_file_version.root.checkpoint_txg
+        ),
+        "崩溃恢复落到 A，建 inode 那一版的记录不施加"
+    );
+    match singlefs_core::recovery::rebuild_version(
+        &pool.memory_pool(),
+        &abandoned.root,
+        Some(abandoned.record.clone()),
+    ) {
+        Ok(
+            singlefs_core::recovery::RebuiltVersion::WithFile(version)
+            | singlefs_core::recovery::RebuiltVersion::WithTreesAndNoFile(version),
+        ) => assert_eq!(
+            version.inode_number_watermark(),
+            2 + NEW_INODES_OF_THE_ABANDONED_PUBLISH,
+            "被抛弃那一版从盘上重建得出水位 5"
+        ),
+        other => panic!("被抛弃那一版从盘上重建得出：{other:?}"),
+    }
+    let created_again = publish_new_inodes_in_process(&mut pool, 1, mounted.output.instance);
+    let publish_parameters = parameters();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    singlefs_core::mount::roll_back_by_a_forward_publish(
+        &publish_parameters,
+        devices,
+        &mut pool.allocator,
+        &mut pool.output,
+        singlefs_core::mount::RollbackTarget {
+            instance: first_file_version.root.instance,
+            checkpoint_txg: first_file_version.root.checkpoint_txg,
+        },
+    )
+    .expect("挂着的时候回退到 A");
+    let rollback = pool.output.clone();
+    RollbackAfterARecoveryAbandonedARootThatCreatedInodes {
+        pool,
+        first_file_version,
+        abandoned,
+        created_again,
+        rollback,
+    }
+}
+
+/// 崩溃恢复抛弃 `newest` 的本体：根槽与 `units_hidden_besides_the_root_slot`（盘、字节偏移、长度）暂时读不出，其余见
+/// [`abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`]。
+fn abandon_the_newest_root_by_a_recovery_hiding_its_root_slot_and(
+    pool: &mut BuiltPool,
+    newest: &TransactionOutput,
+    units_hidden_besides_the_root_slot: Vec<(DeviceIdentity, DeviceOffsetInBytes, usize)>,
+    shadow_ledger: singlefs_core::mount::ShadowLedger,
 ) -> singlefs_core::mount::Mounted {
     use singlefs_core::block_device::{BlockDevice, WriteDurability};
     let publish_parameters = parameters();
@@ -777,7 +942,6 @@ pub fn abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before_with_
     );
     let root_slot_bytes =
         usize::try_from(publish_parameters.geometry.physical_block_size).expect("根槽宽");
-    let data_unit_bytes = usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768");
     fn index_of_device(devices: &[(DeviceIdentity, Recorded)], identity: DeviceIdentity) -> usize {
         devices
             .iter()
@@ -786,19 +950,7 @@ pub fn abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before_with_
     }
     let mut devices = pool.reopen_recorded();
     let places_to_hide = std::iter::once((root_slot_device, root_slot_offset, root_slot_bytes))
-        .chain(
-            newest
-                .data_pointers
-                .iter()
-                .flat_map(|pointer| pointer.locations)
-                .map(|location| {
-                    (
-                        location.device,
-                        location.slot.to_device_offset(),
-                        data_unit_bytes,
-                    )
-                }),
-        );
+        .chain(units_hidden_besides_the_root_slot);
     let mut saved: Vec<(
         DeviceIdentity,
         singlefs_core::address::DeviceOffsetInBytes,
diff --git a/crates/singlefs-harness/tests/common_admission_pool/mod.rs b/crates/singlefs-harness/tests/common_admission_pool/mod.rs
index a2fcb3e..0d38c3e 100644
--- a/crates/singlefs-harness/tests/common_admission_pool/mod.rs
+++ b/crates/singlefs-harness/tests/common_admission_pool/mod.rs
@@ -49,10 +49,10 @@ pub const OVERWRITE_BYTES: usize = 2999;
 /// 核这里三样（参数、盘宽、几何）与 history.rs 那一档逐项相同，两边改了一边就红。
 const UNIT_AREA_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES: u64 = 384;
 
-/// 单元区 384 槽那一档小盘为环窗口补足的槽数：815（D28（挂载期承诺量） 已定项 1 末项环窗口每块盘常驻扣，用户 2026-10-02 定「甲」：小盘放大到扣得起）。
-/// 抄自 `crates/singlefs-harness/src/history.rs` 的 `SMALL_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS`（式子 750 + 5 + 60 的来历写在那边），为什么各留一份见上一条；
-/// 名字里的 384 槽是补足之外的那一段。
-const RING_WINDOW_ALLOWANCE_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES: u64 = 815;
+/// 单元区 384 槽那一档小盘为环窗口补足的槽数：1007（D28（挂载期承诺量） 已定项 1 末项环窗口每块盘常驻扣，用户 2026-10-02 定「甲」：小盘放大到扣得起；
+/// 第 50 批退役树那几项再补 192）。抄自 `crates/singlefs-harness/src/history.rs` 的 `SMALL_DEVICE_RING_WINDOW_ALLOWANCE_SLOTS`（式子的来历写在那边），
+/// 为什么各留一份见上一条；名字里的 384 槽是补足之外的那一段。
+const RING_WINDOW_ALLOWANCE_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES: u64 = 1007;
 
 /// 单元区 384 槽那一档小盘的 journal 环字节数：6 MiB。
 /// 抄自 `crates/singlefs-harness/src/history.rs` 的 `SMALL_DEVICE_JOURNAL_RING_BYTES`，为什么各留一份见上一条。
@@ -66,7 +66,7 @@ pub fn parameters_of_the_384_slot_small_devices() -> MakeFilesystemParameters {
     parameters
 }
 
-/// 单元区 384 槽那一档小盘每块盘的字节数：单元区起点（6 MiB 的环末尾的下一个槽）加环窗口补足的 815 槽、再加 384 槽。
+/// 单元区 384 槽那一档小盘每块盘的字节数：单元区起点（6 MiB 的环末尾的下一个槽）加环窗口补足的 1007 槽、再加 384 槽。
 #[must_use]
 pub fn device_bytes_of_the_384_slot_small_devices() -> u64 {
     let unit_area_start_slot =
diff --git a/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs b/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs
index 8eeea43..3f0acaf 100644
--- a/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs
+++ b/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs
@@ -20,7 +20,7 @@ use singlefs_core::mount::{roll_back_by_a_forward_publish, RollbackTarget};
 use singlefs_core::pointer::NodePointer;
 use singlefs_core::records::{
     TREE_KIND_ACCOUNTING, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE,
-    TREE_KIND_LIVELIST, TREE_KIND_TOMBSTONE,
+    TREE_KIND_LIVELIST, TREE_KIND_RETIREMENT, TREE_KIND_TOMBSTONE,
 };
 use singlefs_core::transaction::{
     publish_new_inodes, publish_unlink_intent, publish_unlink_step,
@@ -89,8 +89,9 @@ const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 116;
 /// 树表条目里根指针 86 字节占 [14, 100)（`walk.rs` 读 `entry[14..100]`）。
 const TREE_TABLE_ENTRY_ROOT_POINTER_START: usize = 14;
 const TREE_TABLE_ENTRY_ROOT_POINTER_END: usize = 100;
-/// 种类码登记表（D8（核心索引结构） 已定项 9）里没有的一个码（攻方 L1a 把 `deleted_inodes` 那一条的种类 9 改成它）。
-const TREE_KIND_NOT_REGISTERED: u16 = 13;
+/// 种类码登记表（D8（核心索引结构） 已定项 9）里没有的一个码（攻方 L1a 把 `deleted_inodes` 那一条的种类 9 改成它）：
+/// 13 第 50 批登记给了退役树，改用 14。
+const TREE_KIND_NOT_REGISTERED: u16 = 14;
 /// 码 2 节点头里条目宽 2 字节在 84 + 2k、最大 key 在 52 + k（D18（块里携带什么信息） 已定项 7、18）。
 fn index_node_entry_width_offset(key_width: usize) -> usize {
     84 + 2 * key_width
@@ -673,6 +674,97 @@ pub fn append_a_copy_of_the_first_tombstone_of_the_newest_root(image: &mut Memor
     );
 }
 
+/// 退役记录 34 字节：被退役身份（出生树 0、打包记录类型 8、容器号 10、容器出生代 18，共 26 字节）、死亡 txg 26
+/// （判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 2 条）。
+pub const RETIREMENT_RECORD_BYTES: usize = 34;
+pub const RETIREMENT_RECORD_RECORD_TYPE_OFFSET: usize = 8;
+pub const RETIREMENT_RECORD_CONTAINER_OFFSET: usize = 10;
+pub const RETIREMENT_RECORD_DEATH_TXG_OFFSET: usize = 26;
+pub const RETIREMENT_RECORD_IDENTITY_BYTES: usize = 26;
+/// inode 树内部条目里身份引用的起点（分隔 key 8 字节之后，26 字节，排法同退役记录的被退役身份）。
+const INODE_INTERNAL_ENTRY_IDENTITY_OFFSET: usize = 8;
+
+/// 最新的根那一版退役树第一片容器里第一条退役记录改一处（`change` 拿到的是那 34 字节），重封、补校验和。
+pub fn change_the_first_retirement_record_of_the_newest_root(
+    image: &mut MemoryPool,
+    change: impl FnOnce(&mut [u8]),
+) {
+    let container_slot =
+        first_container_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_RETIREMENT);
+    change_the_unit_and_propagate_its_checksum(
+        image,
+        container_slot,
+        data_unit_bytes(),
+        |container| {
+            let first = PACKED_UNIT_RECORDS_START;
+            change(&mut container[first..first + RETIREMENT_RECORD_BYTES]);
+        },
+    );
+}
+
+/// 最新的根那一版退役树第一片容器（这几份镜像上只一条记录）末尾再追加一条：`record_of` 拿第一条那 34 字节、交回要追加的那 34 字节；
+/// 记录数 1 → 2、声明长度 34 → 68，重封、补校验和。
+pub fn append_a_retirement_record_to_the_first_container_of_the_newest_root(
+    image: &mut MemoryPool,
+    record_of: impl FnOnce(&[u8]) -> Vec<u8>,
+) {
+    let container_slot =
+        first_container_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_RETIREMENT);
+    change_the_unit_and_propagate_its_checksum(
+        image,
+        container_slot,
+        data_unit_bytes(),
+        |container| {
+            assert_eq!(
+                get_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET),
+                1,
+                "退役容器里一条退役记录"
+            );
+            let first = PACKED_UNIT_RECORDS_START;
+            let second = first + RETIREMENT_RECORD_BYTES;
+            let appended = record_of(&container[first..second]);
+            assert_eq!(appended.len(), RETIREMENT_RECORD_BYTES, "退役记录 34 字节");
+            container[second..second + RETIREMENT_RECORD_BYTES].copy_from_slice(&appended);
+            set_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET, 2);
+            set_u16(
+                container,
+                UNIT_DECLARED_LENGTH_OFFSET,
+                u16::try_from(2 * RETIREMENT_RECORD_BYTES).expect("68"),
+            );
+        },
+    );
+}
+
+/// 最新的根那一版 inode 树根（层级 1，直接管叶容器）第一条条目的身份引用那 26 字节（排法同退役记录的被退役身份）。
+pub fn identity_of_the_first_inode_leaf_of_the_newest_root(image: &MemoryPool) -> Vec<u8> {
+    let inode_tree_root = tree_table_entry_of_kind(image, TREE_KIND_INODE).root;
+    let root = parse_index_node(&unit_bytes_at(
+        image,
+        the_slot_of(&inode_tree_root),
+        node_bytes(),
+    ))
+    .expect("inode 树根解得开");
+    assert_eq!(root.level, 1, "这几份镜像上 inode 树根直接管叶容器");
+    root.entries.first().expect("inode 树根至少一条条目")[INODE_INTERNAL_ENTRY_IDENTITY_OFFSET
+        ..INODE_INTERNAL_ENTRY_IDENTITY_OFFSET + RETIREMENT_RECORD_IDENTITY_BYTES]
+        .to_vec()
+}
+
+/// 退役记录（34 字节）里 `offset` 那 8 字节读成数。
+pub fn retirement_record_field(record: &[u8], offset: usize) -> u64 {
+    get_u64(record, offset)
+}
+
+/// 退役记录（34 字节）里 `offset` 那 8 字节改成 `value`。
+pub fn set_retirement_record_field(record: &mut [u8], offset: usize, value: u64) {
+    set_u64(record, offset, value);
+}
+
+/// 退役记录（34 字节）里被退役身份的打包记录类型改成 `record_type`。
+pub fn set_the_retired_record_type(record: &mut [u8], record_type: u16) {
+    set_u16(record, RETIREMENT_RECORD_RECORD_TYPE_OFFSET, record_type);
+}
+
 /// 一条墓碑（40 字节）里 `offset` 那 8 字节读成数。
 pub fn tombstone_field(tombstone: &[u8], offset: usize) -> u64 {
     get_u64(tombstone, offset)
diff --git a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
index beebcd0..3ee99ac 100644
--- a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
+++ b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
@@ -57,8 +57,10 @@ fn capacities_of_one_record_and_two_entries() -> ContainerTreeCapacities {
             TombstoneRecordsPerContainerCappedForTests::new(1).expect("1..=815"),
         ),
         lineage_container: LineageContainerCapacity::CappedForTests(
-            LineageRecordsPerContainerCappedForTests::new(1).expect("1..=2039"),
+            LineageRecordsPerContainerCappedForTests::new(1).expect("1..=1359"),
         ),
+        retirement_container:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         node_entries: ContainerTreeNodeEntries::CappedForTests(
             ContainerTreeNodeEntriesCappedForTests::new(2).expect("2..=135"),
         ),
@@ -210,10 +212,11 @@ fn the_third_tombstone_container_grows_the_tree_and_keeps_the_old_root_in_place(
         "旧根不被这次换下：{:?}",
         growth.released
     );
+    let retirement_tree_internal_levels = 0;
     assert_eq!(
         committed_reservation_of(&growth),
-        2 * ((2 + 1 + 1) * 16_384 + 41 * 3),
-        "墓碑预付按长高之后的 H = 1：inode 1、5、6 三个活对象（2、3 删完，4 删除中）"
+        2 * ((2 + 1 + 1 + 2 + retirement_tree_internal_levels + 1) * 16_384 + 41 * 3 + 35 * 4),
+        "标签 6 按长高之后的 H_墓碑 = 1：inode 1、5、6 三个活对象（2、3 删完，4 删除中），四条记录（4 的 nlink = 0 也算）；退役树零根，H_退役 = 0"
     );
     let rebuilt = rebuilt_from_disk(&pool);
     assert_eq!(
@@ -287,6 +290,8 @@ fn a_tree_grown_under_the_capped_fanout_takes_the_next_container_under_the_produ
     let product_fanout = ContainerTreeCapacities {
         tombstone_container: capped.tombstone_container,
         lineage_container: capped.lineage_container,
+        retirement_container:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     };
     let next = unlink(&mut pool, 5, product_fanout);
@@ -314,6 +319,8 @@ fn the_third_rollback_grows_the_lineage_tree_and_the_checkpoint_cost_counts_the_
         batch_size: UnlinkBatchSize::OneExtentLowerLeaf,
         tombstone_container_capacity: capacities.tombstone_container,
         lineage_container_capacity: capacities.lineage_container,
+        retirement_container_capacity:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         container_tree_node_entries: capacities.node_entries,
     });
     for _ in 0..3 {
@@ -456,6 +463,8 @@ fn a_tombstone_container_holding_more_records_than_the_knob_reddens_only_the_kno
             TombstoneRecordsPerContainerCappedForTests::new(2).expect("1..=815"),
         ),
         lineage_container: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     };
     for inode in 2..=3 {
diff --git a/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs b/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs
index b203acc..0336883 100644
--- a/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs
+++ b/crates/singlefs-harness/tests/core_review_tree_table_duplicates_and_slot_one_search.rs
@@ -1,6 +1,6 @@
 //! 代码审阅第 27、36、37 条与实审 A2a 交来的「journal 环短于三条记录」（实审 A2b）的会红用例：
 //! 树表里同一种树出现两条时，读树表的五个读者一律判损坏（不取首条也不取末条）；
-//! 所选根的 checkpoint_txg 加一溢出时恢复报损坏、不 panic；
+//! 所选根的 checkpoint_txg 加一溢出时恢复报损坏、不 panic；从盘上重建一版只剩 inode 号水位的台阶行、没有它自己那一行水位时判坏，可写挂载在任何写之前拒；
 //! 全池槽 0 都自证不过时，在格式允许的每一档槽距上找槽 1，只收自己记的槽距等于所在偏移的那一槽，几档都收得到时取世代号最大的
 //! （D22（单元原子性怎么合成） 已定项 16、D2（RAID 条带策略） 已定项 19）；
 //! journal 环装不下 F 条记录时 mkfs 在任何写之前拒掉（I-8.1（环几何够大）、D23（journal 的角色与格式） 已定项 18 / 已定项 19）。
@@ -20,11 +20,15 @@ use singlefs_core::make_filesystem::{
     allocator_after_make_filesystem, make_filesystem, MakeFilesystemError, MakeFilesystemOutput,
     MakeFilesystemParameters,
 };
+use singlefs_core::mount::{mount_writable, MountError};
 use singlefs_core::mounted_read::{
     mount_read_only, open_pool_for_read, MountReadOnlyFailure, OpenPoolForReadFailure,
 };
 use singlefs_core::pointer::NodePointer;
-use singlefs_core::records::{TreeTableEntry, TREE_KIND_ALLOCATION, TREE_KIND_INODE};
+use singlefs_core::records::{
+    AccountingEntry, TreeTableEntry, STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING,
+    TREE_KIND_ALLOCATION, TREE_KIND_INODE,
+};
 use singlefs_core::recovery::{
     allocation_records_under_root, choose_system_configuration, rebuild_version, recover,
     user_visible_tree_root_pointers, walk_to_file, JournalPolicy, PoolReader,
@@ -32,20 +36,23 @@ use singlefs_core::recovery::{
     CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR, TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE,
 };
 use singlefs_core::root_record::RootRecord;
-use singlefs_core::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegion};
+use singlefs_core::root_ring::{
+    slot_offset, target_for_publish, RootRingSlot, RootRingSlotsPerRegion,
+};
 use singlefs_core::system_configuration::{
     SystemConfiguration, SystemImmutableSizes,
     INCOMPAT_RETIRED_FIRST_SSD_LINE_WITH_ROLLBACK_WITNESS_BIT,
     SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
 };
 use singlefs_core::transaction::{
-    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
+    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
+    TransactionOutput,
 };
 use singlefs_core::unit::{build_index_node, parse_index_node};
 use singlefs_format::{
     JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT,
-    JOURNAL_SAFETY_FACTOR, NODE_BYTES, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
-    UNIT_AREA_START_SLOT,
+    JOURNAL_SAFETY_FACTOR, NODE_BYTES, ROOT_RING_REGION_DEVICES, SLOT_BYTES,
+    SYSTEM_CONFIGURATION_SLOT_BYTES, UNIT_AREA_START_SLOT,
 };
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
 use singlefs_harness::{RecordingBlockDevice, SharedStream};
@@ -135,13 +142,21 @@ struct PoolAfterTheFirstFile {
 }
 
 fn pool_after_the_first_file() -> PoolAfterTheFirstFile {
+    pool_after_the_first_file_and_the_overwrites(&[]).0
+}
+
+/// 同 [`pool_after_the_first_file`]，第一个文件之后再按次序把 `overwrite_contents` 一份一份覆盖写（txg 4 起），交回池与每次覆盖写的输出；
+/// 池里的镜像是最后一次覆盖写之后的。
+fn pool_after_the_first_file_and_the_overwrites(
+    overwrite_contents: &[Vec<u8>],
+) -> (PoolAfterTheFirstFile, Vec<TransactionOutput>) {
     let parameters = parameters_with(geometry_with_minimum_input_output_bytes(512));
     let mut devices = two_sparse_devices();
     let genesis = make_filesystem(&parameters, &mut devices).expect("默认几何上 mkfs");
     let mut allocator: PoolAllocator =
         allocator_after_make_filesystem(&parameters, &devices, &genesis);
     let first_content = content_of(3000, 0);
-    let (instance, first) = {
+    let (instance, first, overwrites) = {
         let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
         let instance = acquire_instance(&mut writer).expect("取号 1");
         let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
@@ -157,15 +172,35 @@ fn pool_after_the_first_file() -> PoolAfterTheFirstFile {
             &warmed.last_record_bytes,
         )
         .expect("第一个文件");
-        (instance, first)
+        let mut overwrites: Vec<TransactionOutput> = Vec::new();
+        for content in overwrite_contents {
+            let previous = overwrites.last().unwrap_or(&first).clone();
+            overwrites.push(
+                publish_overwrite(
+                    &mut writer,
+                    &mut allocator,
+                    &previous,
+                    FirstFile {
+                        content,
+                        write_time_seconds: WRITE_TIME_SECONDS,
+                    },
+                    instance,
+                )
+                .expect("覆盖写"),
+            );
+        }
+        (instance, first, overwrites)
     };
-    PoolAfterTheFirstFile {
-        image: image_of(&devices),
-        instance,
-        first_content,
-        first,
-        geometry: parameters.geometry,
-    }
+    (
+        PoolAfterTheFirstFile {
+            image: image_of(&devices),
+            instance,
+            first_content,
+            first,
+            geometry: parameters.geometry,
+        },
+        overwrites,
+    )
 }
 
 // ── 第 27 条：树表里同一种树出现两条 ──
@@ -343,14 +378,102 @@ fn a_tree_table_carrying_one_kind_of_tree_twice_is_refused_as_damaged_by_every_r
 
 // ── 第 36 条：所选根的 checkpoint_txg 加一溢出 ──
 
-/// 在根环区域 0 的槽 7（txg 3 之前的发布都没用到它）写一条 `root` 的根记录、checkpoint_txg 换成 `checkpoint_txg`：
-/// 它的 txg 最大，择根择中它。
+/// 一条根指着的记账树（只有根兼叶一个节点）的行交给 `rewrite_rows` 改过（改完仍按 key 升序），照原叶的头重建（key 区间取首行、末行的 key），
+/// 写回它两条位置条目指的槽；树表里记账树那一条指针的两条校验和跟着换，树表照 [`root_whose_tree_table_carries`] 重建写回，
+/// 交回一条除树表指针之外与 `root` 相同的根。原地写：别的根若指着同一片叶与同一个树表单元，它们的校验和就对不上了。
+fn root_whose_accounting_rows_are_rewritten(
+    image: &mut MemoryPool,
+    root: &RootRecord,
+    rewrite_rows: impl Fn(&mut Vec<AccountingEntry>),
+) -> RootRecord {
+    let tree_table_entries = parse_index_node(&tree_table_unit_of(image, root))
+        .expect("树表单元解得开")
+        .entries;
+    let mut rewritten_tree_table_entries = Vec::with_capacity(tree_table_entries.len());
+    for entry_bytes in &tree_table_entries {
+        let mut entry = TreeTableEntry::parse(entry_bytes).expect("写者写的条目解得开");
+        if entry.kind == TREE_KIND_ACCOUNTING {
+            let first_location = entry.root.locations[0];
+            let leaf = PoolReader::read(
+                image,
+                first_location.device,
+                first_location.slot.to_device_offset(),
+                node_bytes(),
+            )
+            .expect("记账树叶读得到");
+            let header = parse_index_node(&leaf).expect("记账树叶解得开");
+            let mut rows: Vec<AccountingEntry> = header
+                .entries
+                .iter()
+                .map(|row_bytes| AccountingEntry::parse(row_bytes).expect("写者写的记账行解得开"))
+                .collect();
+            let key_range_of = |rows_in_key_order: &[AccountingEntry]| {
+                (
+                    rows_in_key_order
+                        .first()
+                        .expect("记账树恒有池级 3 行")
+                        .key_bytes(),
+                    rows_in_key_order
+                        .last()
+                        .expect("记账树恒有池级 3 行")
+                        .key_bytes(),
+                )
+            };
+            assert_eq!(
+                (
+                    header.level,
+                    (header.smallest_key.clone(), header.largest_key.clone())
+                ),
+                (0, key_range_of(&rows)),
+                "记账树只有根兼叶一个节点，叶头的 key 区间是首行与末行的 key"
+            );
+            rewrite_rows(&mut rows);
+            let (smallest_key, largest_key) = key_range_of(&rows);
+            let rebuilt = build_index_node(
+                header.tree,
+                header.level,
+                header.key_width,
+                &smallest_key,
+                &largest_key,
+                header.birth_txg,
+                &FILESYSTEM_IDENTIFIER,
+                header.instance,
+                header.birth_sequence,
+                u16::try_from(header.entry_width).expect("条目宽 34"),
+                &rows
+                    .iter()
+                    .map(AccountingEntry::to_bytes)
+                    .collect::<Vec<_>>(),
+            );
+            let checksum = crc32_castagnoli(&rebuilt);
+            for location in &mut entry.root.locations {
+                image
+                    .devices
+                    .get_mut(&location.device)
+                    .expect("池里有这块盘")
+                    .write(location.slot.to_device_offset(), &rebuilt);
+                location.unit_checksum = checksum;
+            }
+        }
+        rewritten_tree_table_entries.push(entry.to_bytes());
+    }
+    root_whose_tree_table_carries(image, root, &rewritten_tree_table_entries)
+}
+
+/// 在根环区域 0 的槽 7（txg 3 之前的发布都没用到它）写一条 `root` 的根记录、checkpoint_txg 换成 `checkpoint_txg`，
+/// 记账行的代段跟着换成 `checkpoint_txg`（I-3.13：记账行代段等于这一版的 txg；恢复要代段 = 根 txg 的那一行 inode 号水位）：
+/// 它的 txg 最大，择根择中它。记账叶与树表原地改写，txg 3 那条根的两条校验和就对不上了，种的这条用不到它。
 fn image_whose_chosen_root_has_the_checkpoint_txg(
     pool: &PoolAfterTheFirstFile,
     checkpoint_txg: CheckpointTxg,
 ) -> MemoryPool {
     let mut image = pool.image.clone();
-    let mut planted = pool.first.root;
+    let mut planted =
+        root_whose_accounting_rows_are_rewritten(&mut image, &pool.first.root, |rows| {
+            for row in rows {
+                row.generation = checkpoint_txg;
+            }
+        });
     planted.checkpoint_txg = checkpoint_txg;
     let ring_slot = RootRingSlot { region: 0, slot: 7 };
     image
@@ -364,9 +487,9 @@ fn image_whose_chosen_root_has_the_checkpoint_txg(
     image
 }
 
-/// 所选根的 checkpoint_txg 是 u64 最大值（自证校验和对得上、别的字段都是第一个文件那一版的）：锚点读不出时链首那条的 txg 要取它加一，
-/// 改前在这一步溢出、只读挂载与恢复 panic；改后在施加任何一条记录之前报损坏。
-/// 边界：最大值减一加一不溢出，恢复照常读回第一个文件。
+/// 所选根的 checkpoint_txg 是 u64 最大值（自证校验和对得上、记账行代段跟着换成它，别的字段都是第一个文件那一版的）：锚点读不出时链首那条的 txg
+/// 要取它加一，改前在这一步溢出、只读挂载与恢复 panic；改后在施加任何一条记录之前报损坏。
+/// 边界：最大值减一加一不溢出，恢复照常读回第一个文件（读的是种的那条根：它自己的 inode 号水位那一行代段等于它的 txg）。
 #[test]
 fn a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged_instead_of_panicking(
 ) {
@@ -412,6 +535,106 @@ fn a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged
     );
 }
 
+/// 从盘上重建一版要它自己那一行 inode 号水位（统计量 12、代段 = 根的 txg），不认台阶顶替（D5（快照 / 空间记账机制） 已定项 2 射程「inode 号水位的台阶行」；
+/// I-3.13 记账行代段等于这一版的 txg）：第一个文件（txg 3，它那一行是第 1 级台阶）之后覆盖写一版（txg 4：池级与每盘的行换成代段 4，
+/// 照抄台阶 (12, 3)，加自己那一行 (12, 4)，16 行）。把 txg 4 那一版自己那一行的统计量标签改成 13（仍排在末尾；不算台阶的行仍是 3 + 6 × 2），
+/// 那一版的根槽原地换成指着改过的树表的那一条：这一版只剩台阶 (12, 3) 那一行水位。
+/// 从盘上重建 txg 4 那一版（`rebuild_version`，可写挂载与回退建上一版走的那一份）判它坏：记账树里没有这一版的水位行；
+/// 可写挂载建不出所选那一版，这个池系统配置里见证的 journal 计数器是 mkfs 写的 0，F2 乙不退
+/// （D23（journal 的角色与格式） 已定项 14「c_见证 = 0 … 都不退」），在任何写之前拒、报同一个失败。冷读文件内容的走读不读水位，不在这条用例里。
+/// 判别力：重建退回只判「水位那一统计量有一行」，台阶 (12, 3) 顶上，txg 4 那一版照建、可写挂载照挂。
+#[test]
+fn a_version_whose_own_inode_number_watermark_row_is_damaged_leaving_only_a_step_is_judged_damaged_by_the_rebuild_and_refused_by_the_writable_mount(
+) {
+    let (mut pool, overwrites) =
+        pool_after_the_first_file_and_the_overwrites(&[content_of(2000, 1)]);
+    let overwrite = &overwrites[0];
+    let overwrite_txg = overwrite.root.checkpoint_txg;
+    let rows_of = |output: &TransactionOutput| {
+        output
+            .accounting_entries
+            .iter()
+            .filter(|row| row.statistic == STATISTIC_INODE_WATERMARK)
+            .map(|row| row.generation)
+            .collect::<Vec<_>>()
+    };
+    assert_eq!(
+        (
+            pool.first.root.checkpoint_txg,
+            overwrite_txg,
+            overwrite.accounting_entries.len(),
+            rows_of(overwrite)
+        ),
+        (
+            CheckpointTxg(3),
+            CheckpointTxg(4),
+            16,
+            vec![CheckpointTxg(3), CheckpointTxg(4)]
+        ),
+        "覆盖写那一版：16 行，水位那一统计量是台阶 (12, 3) 与它自己那一行 (12, 4)"
+    );
+    let damaged_root =
+        root_whose_accounting_rows_are_rewritten(&mut pool.image, &overwrite.root, |rows| {
+            let own_row = rows.last_mut().expect("记账树恒有池级 3 行");
+            assert_eq!(
+                (own_row.statistic, own_row.generation),
+                (STATISTIC_INODE_WATERMARK, overwrite_txg),
+                "末行是这一版自己那一行水位"
+            );
+            own_row.statistic = STATISTIC_INODE_WATERMARK + 1;
+        });
+    let ring_slot = target_for_publish(overwrite_txg, pool.geometry.root_ring_slots_per_region);
+    let device_holding_the_root = DeviceIdentity(
+        ROOT_RING_REGION_DEVICES[usize::try_from(ring_slot.region).expect("区域号小于 3")],
+    );
+    pool.image
+        .devices
+        .get_mut(&device_holding_the_root)
+        .expect("区域落在池里的盘上")
+        .write(
+            slot_offset(ring_slot, pool.geometry.fixed_structure_slot_spacing),
+            &damaged_root
+                .to_slot(usize::try_from(pool.geometry.physical_block_size).expect("根槽宽")),
+        );
+    assert_eq!(
+        rebuild_version(&pool.image, &damaged_root, Some(overwrite.record.clone())).err(),
+        Some(RebuildVersionFailure::Walk(
+            RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree
+        )),
+        "从盘上重建 txg 4 那一版：只剩台阶那一行水位，判记账树里没有这一版的水位行"
+    );
+    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = pool
+        .image
+        .devices
+        .iter()
+        .map(|(identity, image)| {
+            let mut device = SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE);
+            device.image = image.clone();
+            (*identity, device)
+        })
+        .collect();
+    let refused = mount_writable(
+        &parameters_with(geometry_with_minimum_input_output_bytes(512)),
+        &mut devices,
+    )
+    .err();
+    assert!(
+        matches!(
+            refused,
+            Some(MountError::Recovery(
+                RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree
+            ))
+        ),
+        "可写挂载建上一版判 txg 4 那一版坏：系统配置见证的计数器是 0、不退（D23 已定项 14），拒的是记账树里没有这一版的水位行：{refused:?}"
+    );
+    assert!(
+        devices
+            .iter()
+            .all(|(identity, device)| pool.image.devices.get(identity) == Some(&device.image)),
+        "可写挂载在任何写之前拒：两块盘逐字节不变"
+    );
+}
+
 // ── 第 37 条：全池槽 0 都自证不过时逐档找槽 1 ──
 
 fn slot_bytes() -> usize {
diff --git a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
index e5a4be4..0d7c095 100644
--- a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
+++ b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
@@ -31,7 +31,7 @@ use singlefs_core::pointer::{
 };
 use singlefs_core::records::{
     mapping_key_for_node, parse_mapping_entry, TreeTableEntry, TREE_KIND_ACCOUNTING,
-    TREE_KIND_INODE, TREE_KIND_LOGGED_OPERATIONS,
+    TREE_KIND_INODE, TREE_KIND_RETIREMENT,
 };
 use singlefs_core::recovery::{
     choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, JournalPolicy,
@@ -427,8 +427,9 @@ fn an_accounting_leaf_with_a_repeated_key_is_refused_by_the_writable_mount_befor
     );
 }
 
-/// 场景：最新那条根的树表里 logged_ops 那条条目的树 ID 改成根记录的树 ID 水位（第一个文件那一版是 65548），树表与根记录重封。
-/// logged_ops 在发号次序里排最后、号最大（65547），改成 65548 之后树表条目仍按树 ID 严格升序、合发号次序，只违反水位那一条。
+/// 场景：最新那条根的树表里退役树那条条目的树 ID 改成根记录的树 ID 水位（第一个文件那一版是 65549），树表与根记录重封。
+/// 退役树在发号次序里排最后、号最大（65548），改成 65549 之后树表条目仍按树 ID 严格升序、合发号次序，只违反水位那一条
+/// （第 50 批之前排最后的是 logged_ops，改的是它）。
 /// （树表 7 → 11 条之前排最后的是 deadlist（那时号 18，改成水位 19），D18（块里携带什么信息） 已定项 24 第 6 条之后 deadlist 后面还有四棵，改它就先违反 I-9.16。）
 /// 预期：可写挂载在任何写之前报 I-7.8（根记录树 ID 水位不低于全池最大树 ID）「树 ID 不低于水位」。改之前重建不判它，写行那次发布照抄这张树表。
 /// 合入后验证一（2026-09-27）把被改的条目从 livelist 换成 deadlist：livelist 改成 19 同时违反 I-9.16（树表条目按树 ID 严格升序且合发号次序），
@@ -437,13 +438,9 @@ fn an_accounting_leaf_with_a_repeated_key_is_refused_by_the_writable_mount_befor
 fn a_tree_table_entry_at_or_above_the_tree_identifier_watermark_is_refused_by_the_writable_mount() {
     let (mut devices, first) = pool_after_the_first_file();
     let watermark = first.root.tree_identifier_watermark;
-    rewrite_a_tree_table_entry_of_the_newest_root(
-        &mut devices,
-        TREE_KIND_LOGGED_OPERATIONS,
-        |entry| {
-            entry[..8].copy_from_slice(&watermark.to_le_bytes());
-        },
-    );
+    rewrite_a_tree_table_entry_of_the_newest_root(&mut devices, TREE_KIND_RETIREMENT, |entry| {
+        entry[..8].copy_from_slice(&watermark.to_le_bytes());
+    });
     let refusal = writable_mount_refused_before_any_write(&mut devices);
     assert!(
         matches!(
diff --git a/crates/singlefs-harness/tests/deletion_scenario_writes.rs b/crates/singlefs-harness/tests/deletion_scenario_writes.rs
index d9264cd..d1b2587 100644
--- a/crates/singlefs-harness/tests/deletion_scenario_writes.rs
+++ b/crates/singlefs-harness/tests/deletion_scenario_writes.rs
@@ -211,7 +211,7 @@ fn spans_of_consecutive_parts_refuse_a_part_without_writes_and_accounts_that_do_
 }
 
 #[test]
-fn the_in_process_history_publishes_the_intent_then_three_batches_of_thirteen_twelve_eleven_and_ten_units_per_device(
+fn the_in_process_history_publishes_the_intent_then_three_batches_of_thirteen_twelve_eleven_and_twelve_units_per_device(
 ) {
     let (run, operations) =
         run_on_two_memory_devices(DeletionHistory::IntentAndEveryBatchInProcess);
@@ -221,9 +221,9 @@ fn the_in_process_history_publishes_the_intent_then_three_batches_of_thirteen_tw
             (DeletionHistoryPublishKind::UnlinkIntent, 13, 9, 3),
             (DeletionHistoryPublishKind::UnlinkBatchInProcess, 12, 8, 2),
             (DeletionHistoryPublishKind::UnlinkBatchInProcess, 11, 7, 1),
-            (DeletionHistoryPublishKind::UnlinkBatchInProcess, 10, 6, 0),
+            (DeletionHistoryPublishKind::UnlinkBatchInProcess, 12, 8, 0),
         ],
-        "意图 13、3 → 2 那一批 12、2 → 1 那一批 11、最后一批 10（每盘），批大小 1 走三批"
+        "意图 13、3 → 2 那一批 12、2 → 1 那一批 11、最后一批 12（每盘：此前 10，唯一那片叶容器身份消失、退役树多写容器与根两个单元），批大小 1 走三批"
     );
     let txgs: Vec<u64> = publishes(&run)
         .iter()
@@ -252,7 +252,7 @@ fn the_mount_continuation_history_publishes_its_three_batches_after_the_unmount_
             (DeletionHistoryPublishKind::UnlinkIntent, 13, 9, 3),
             (DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation, 12, 8, 2),
             (DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation, 13, 7, 1),
-            (DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation, 12, 6, 0),
+            (DeletionHistoryPublishKind::UnlinkBatchOfTheMountContinuation, 14, 8, 0),
         ],
         "意图在进程里发，三批都由挂载续做发：分配记录树根之下那几片之外与进程里那三批相同，后两批多改一对分配记录树的叶"
     );
diff --git a/crates/singlefs-harness/tests/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs b/crates/singlefs-harness/tests/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs
new file mode 100644
index 0000000..a08e2ad
--- /dev/null
+++ b/crates/singlefs-harness/tests/inode_leaf_absorb_and_borrow_on_the_unlink_path.rs
@@ -0,0 +1,276 @@
+//! unlink 最后一批删掉一条 inode 记录之后 inode 树叶容器的合并与借（E176 M3 那一臂，第 50 批；判决
+//! `research/prompts/m3-retire-r1-main-verification.md` 第三节第 3 条）在发布路径上：左吸收右那一次给被吸收的那一片写一条退役记录、
+//! 死亡 txg = 这次发布的 txg；左从右借不写退役记录，右片保留身份、分隔 key 改成它这一版的最小 key（N1）；两种发布之后池级 checker 一条都不红
+//! （I-9.4 原来的第三句「左容器的最大 key < 右容器号」随 N1 删掉，借之后右片容器号不大于左片最大 key 也不红）。
+//! 空 inode 一次建好（一次发布写进全部记录，叶从左往右装满、233 条一片），再逐个按产品批大小 unlink 到底。
+
+mod common;
+mod common_deletion_in_progress;
+mod common_pointer_head_corruption;
+
+use common::BuiltPool;
+use common_deletion_in_progress::{pool_with_new_inodes, unlink_to_the_end};
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::records::RetirementRecord;
+use singlefs_core::transaction::TransactionOutput;
+use singlefs_core::unit::PackedIdentity;
+use singlefs_format::SLOT_BYTES;
+
+/// 一片 inode 叶容器装 233 条（`singlefs_format::INODE_LEAF_RECORDS`）。
+const RECORDS_PER_LEAF: usize = 233;
+
+fn leaf_identities(version: &TransactionOutput) -> Vec<PackedIdentity> {
+    version
+        .inode_leaf_containers
+        .iter()
+        .map(|container| container.contents.identity)
+        .collect()
+}
+
+fn leaf_record_counts(version: &TransactionOutput) -> Vec<usize> {
+    version
+        .inode_leaf_containers
+        .iter()
+        .map(|container| container.contents.records.len())
+        .collect()
+}
+
+fn retirement_records_of(version: &TransactionOutput) -> Vec<RetirementRecord> {
+    version
+        .retirement_containers
+        .iter()
+        .flat_map(|container| container.contents.records.iter().copied())
+        .collect()
+}
+
+fn violated_invariants(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<&'static str> {
+    verdicts
+        .iter()
+        .filter(|(_, found)| matches!(found, InvariantVerdict::Violated(_)))
+        .map(|(name, _)| *name)
+        .collect()
+}
+
+/// 第一个文件之外再建 233 个空 inode：234 条记录装成 [1..=233]、[234] 两片。unlink 2 号：左片剩 232 条，与右邻合计 233 条装得下一片，
+/// 左片吸收右片、保留左片身份；右片身份消失，最后一批那次发布写一条 (右片身份, 这次的 txg)。池级 checker 一条都不红，退役完备（I-9.35）成立。
+#[test]
+fn the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one() {
+    let mut pool = pool_with_new_inodes("absorb-the-right-neighbour", 233);
+    let before = pool.output.clone();
+    assert_eq!(
+        leaf_record_counts(&before),
+        [RECORDS_PER_LEAF, 1],
+        "234 条记录装成满的一片与一条的一片"
+    );
+    let [left, right] = leaf_identities(&before)[..] else {
+        panic!("两片叶")
+    };
+    unlink_to_the_end(&mut pool, 2);
+    let final_batch = pool.output.clone();
+    assert_eq!(
+        (
+            leaf_identities(&final_batch),
+            leaf_record_counts(&final_batch)
+        ),
+        (vec![left], vec![RECORDS_PER_LEAF]),
+        "左片吸收右片：保留左片身份，232 + 1 = 233 条"
+    );
+    assert_eq!(
+        retirement_records_of(&final_batch),
+        [RetirementRecord {
+            retired: right,
+            death_txg: final_batch.root.checkpoint_txg,
+        }],
+        "被吸收的右片写一条退役记录，死亡 txg = 最后一批那次的 txg"
+    );
+    let verdicts = check_pool_image(&pool.memory_pool());
+    assert_eq!(
+        violated_invariants(&verdicts),
+        Vec::<&str>::new(),
+        "吸收之后一条都不红：{verdicts:?}"
+    );
+}
+
+/// 第一个文件之外再建 `new_inodes` 个空 inode（叶从左往右 233 条一片装满，前三片 [1..=233]、[234..=466]、[467..=699]，第四片从 700 起），
+/// 再把第二片最小的 117 个号逐个 unlink 到底：前 116 次第二片剩 ≥ 117 条、它与两边哪一对都合不进一片（第四片至少一条，第三片与第四片
+/// 合计 ≥ 234），不合不借；第 117 次剩 116 条，低于 ⌈233 ÷ 2⌉ = 117，与右邻合计 349 条合不进一片，从右邻借 ⌊(233 − 116) ÷ 2⌋ = 58 条最小的
+/// （467..=524）：两片 174 / 175 条。交回池、建完那一版各片的身份与第 117 次 unlink 的最后一批那一版。
+fn pool_unlinked_until_the_second_leaf_borrows(
+    tag: &str,
+    new_inodes: u64,
+) -> (BuiltPool, Vec<PackedIdentity>, TransactionOutput) {
+    let mut pool = pool_with_new_inodes(tag, new_inodes);
+    let identities_before = leaf_identities(&pool.output);
+    assert_eq!(
+        leaf_record_counts(&pool.output)[..3],
+        [RECORDS_PER_LEAF; 3],
+        "前三片满"
+    );
+    // 迭代上界：第二片最小的 116 个号，234..=349。
+    for inode in 234..=349 {
+        unlink_to_the_end(&mut pool, inode);
+        assert_eq!(
+            leaf_identities(&pool.output),
+            identities_before,
+            "unlink {inode} 之后第二片还有 ≥ 117 条：不合不借"
+        );
+    }
+    unlink_to_the_end(&mut pool, 350);
+    let borrowed = pool.output.clone();
+    (pool, identities_before, borrowed)
+}
+
+/// 第四片 60 条（[700..=759]）：借完右邻 175 条，与第四片合计 235 条合不进一片，借之后不再合。四片身份都在，右邻的分隔 key 改成它这一版的最小
+/// key 525（N1）。借不写退役记录：退役树一直零根。池级 checker 一条都不红（右邻容器号 467 不大于左片最大 key 524，原 I-9.4 第三句会红的那一形）。
+#[test]
+fn a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing()
+{
+    let (pool, identities_before, borrowed) =
+        pool_unlinked_until_the_second_leaf_borrows("borrow-from-the-right-neighbour", 758);
+    assert_eq!(
+        (leaf_identities(&borrowed), leaf_record_counts(&borrowed)),
+        (identities_before, vec![RECORDS_PER_LEAF, 174, 175, 60]),
+        "第二片 116 条从右邻借 58 条：174 / 175，四片身份都在"
+    );
+    assert_eq!(
+        borrowed.inode_leaf_containers[2]
+            .contents
+            .separator_key_in_the_parent_entry,
+        525,
+        "右邻的分隔 key 改成它这一版的最小 key（N1）"
+    );
+    assert_eq!(
+        retirement_records_of(&borrowed),
+        Vec::<RetirementRecord>::new(),
+        "借不写退役记录，退役树零根"
+    );
+    let verdicts = check_pool_image(&pool.memory_pool());
+    assert_eq!(
+        violated_invariants(&verdicts),
+        Vec::<&str>::new(),
+        "借之后一条都不红：{verdicts:?}"
+    );
+}
+
+/// 第四片 1 条（[700]）：借完右邻 175 条，与第四片合计 176 条装得下一片，借出的那一片接着吸收它的右邻（E176 M3「借完再从借出的那一片看合并」）：
+/// 剩三片 233 / 174 / 176，第四片身份消失，同一次发布写一条 (第四片身份, 这次的 txg)。池级 checker 一条都不红。
+#[test]
+fn the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it() {
+    let (pool, identities_before, borrowed) =
+        pool_unlinked_until_the_second_leaf_borrows("lender-absorbs-after-lending", 699);
+    assert_eq!(
+        (leaf_identities(&borrowed), leaf_record_counts(&borrowed)),
+        (
+            identities_before[..3].to_vec(),
+            vec![RECORDS_PER_LEAF, 174, 176]
+        ),
+        "借完之后借出的第三片吸收第四片"
+    );
+    assert_eq!(
+        retirement_records_of(&borrowed),
+        [RetirementRecord {
+            retired: identities_before[3],
+            death_txg: borrowed.root.checkpoint_txg,
+        }],
+        "被吸收的第四片写一条退役记录"
+    );
+    let verdicts = check_pool_image(&pool.memory_pool());
+    assert_eq!(
+        violated_invariants(&verdicts),
+        Vec::<&str>::new(),
+        "借之后再吸收一条都不红：{verdicts:?}"
+    );
+}
+
+/// 一次发布每块盘新写的槽：这次真正写出的角色（`rewritten`）各自的单元按槽宽取整加起来（第一版每个单元落池里每一块盘，各盘一样多）。
+fn slots_newly_written_on_each_device(publish: &TransactionOutput) -> u64 {
+    publish
+        .units
+        .iter()
+        .filter(|unit| publish.rewritten.contains(&unit.identity))
+        .map(|unit| {
+            u64::try_from(unit.bytes.len())
+                .expect("单元字节数装得进 u64")
+                .div_ceil(SLOT_BYTES)
+        })
+        .sum()
+}
+
+/// R3（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 7 条，用户定「并进 M3 改」）在发布路径上的几何：第一个文件之外再建
+/// `leaves × 233 − 1` 个空 inode，叶装成 `leaves` 片满的（第 k 片是 [233(k − 1) + 1, 233k]）。先把第一片的 2..=117、第二片的 234..=349 逐个 unlink 到底
+/// （两片各剩 117 条：不低于借的下限 117，两片之和 234 合不进一片，一次都不并不借），再 unlink 118：第一片剩 116 条、与右邻合计 233 条，
+/// 第一片吸收第二片。交回吸收之前那一版与吸收那一次（最后一批）发布。
+fn the_first_leaf_absorbing_the_second_in_a_tree_of(
+    tag: &str,
+    leaves: u64,
+) -> (TransactionOutput, TransactionOutput) {
+    let records_per_leaf = u64::try_from(RECORDS_PER_LEAF).expect("233");
+    let mut pool = pool_with_new_inodes(tag, leaves * records_per_leaf - 1);
+    assert_eq!(
+        leaf_record_counts(&pool.output),
+        vec![RECORDS_PER_LEAF; usize::try_from(leaves).expect("片数装得进 usize")],
+        "{leaves} 片满的叶"
+    );
+    // 迭代上界：两段各 116 个号。
+    for inode in (2..=117).chain(234..=349) {
+        unlink_to_the_end(&mut pool, inode);
+    }
+    let before = pool.output.clone();
+    assert_eq!(
+        leaf_record_counts(&before)[..3],
+        [117, 117, RECORDS_PER_LEAF],
+        "第一、二片各剩 117 条，一次都没并没借"
+    );
+    unlink_to_the_end(&mut pool, 118);
+    (before, pool.output.clone())
+}
+
+/// R3 验收（第 50 批补做：纯 M3 下删空最左一片走不到——低于 117 条就从右邻借，删不空——改成用左吸收右）：20 片与 40 片两种几何上，
+/// 第一片吸收第二片那一次每块盘新写的槽数相同，钉 14 槽；第二片右边那几片位置左移一格而内容没变，指针照抄上一版（不进重写清单）。
+/// 判别力：删除类发布改回按位置重写、按位置换下右边每一片（`crates/mutations.tsv` 一行「第 50 批补做 R3：…」）时第二片右边那几片都换了新落点，
+/// 先红在「指针照抄上一版」那一判（证红时量过）；只把挪了位置的片记成重写、不跟着改换下的叶序时发布路径同一个落点释放两次，红在分配器的断言上。
+#[test]
+fn the_first_leaf_absorbing_the_second_writes_as_many_slots_in_a_tree_of_twenty_leaves_as_in_a_tree_of_forty(
+) {
+    let (twenty_before, twenty) =
+        the_first_leaf_absorbing_the_second_in_a_tree_of("absorb-the-second-of-twenty-leaves", 20);
+    let (forty_before, forty) =
+        the_first_leaf_absorbing_the_second_in_a_tree_of("absorb-the-second-of-forty-leaves", 40);
+    for (leaves, before, absorbed) in [(20, &twenty_before, &twenty), (40, &forty_before, &forty)] {
+        let identities_before = leaf_identities(before);
+        assert_eq!(
+            (
+                leaf_identities(absorbed),
+                retirement_records_of(absorbed)
+                    .iter()
+                    .map(|record| record.retired)
+                    .collect::<Vec<_>>(),
+            ),
+            (
+                [&identities_before[..1], &identities_before[2..]].concat(),
+                vec![identities_before[1]],
+            ),
+            "{leaves} 片：第一片吸收第二片、保留自己的身份，第二片写一条退役记录"
+        );
+        assert_eq!(
+            absorbed.inode_leaf_containers[1..]
+                .iter()
+                .map(|container| container.pointer)
+                .collect::<Vec<_>>(),
+            before.inode_leaf_containers[2..]
+                .iter()
+                .map(|container| container.pointer)
+                .collect::<Vec<_>>(),
+            "{leaves} 片：第二片右边的每一片挪了位置、内容没变，指针照抄上一版"
+        );
+    }
+    assert_eq!(
+        (
+            slots_newly_written_on_each_device(&twenty),
+            slots_newly_written_on_each_device(&forty),
+        ),
+        (14, 14),
+        "吸收那一次每块盘新写的槽数不随右边的片数涨"
+    );
+}
diff --git a/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs b/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs
index 3461dd6..b22d816 100644
--- a/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs
+++ b/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs
@@ -12,8 +12,10 @@
 //! [`creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant`]。
 //! C479 那一问用户 2026-10-05 定「写」（D8（核心索引结构） 已定项 6「同一条时间线上 inode 号不复用」），立 I-9.29（同号只许一个出生代，
 //! 判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T10 乙）：重发的号在回退候选集里还有一版留着旧对象时它红，
-//! 下面三条用例都红它；I-9.6 拦不住的「压回的正是发过的最大号」那一形只红它，钉在
-//! [`pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_only_the_one_birth_per_number_invariant`]。
+//! 下面三条用例都红它；I-9.6 拦不住的「压回的正是发过的最大号」那一形红它与 I-9.30，钉在
+//! [`pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants`]。
+//! I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上，W3 第 49 批）三条都红：压回之前那次建 inode 的发布留了一级台阶 (g, H)，
+//! 重发的号小于 H、它的新对象晚于 g 出生（②）；水位压到那一级之下时 ① 也不成立。
 
 mod common;
 mod common_deletion_in_progress;
@@ -24,7 +26,11 @@ use common_deletion_in_progress::{
     pool_with_new_inodes, verdict_of, violated_invariants, INSTANCE_OF_THE_PUBLISHES,
     WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
 };
-use singlefs_checker::image::{InvariantVerdict, ONE_OBJECT_BIRTH_PER_INODE_NUMBER};
+use singlefs_checker::image::{
+    InvariantVerdict,
+    INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
+    ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
+};
 use singlefs_core::address::{CheckpointTxg, InodeNumber};
 use singlefs_core::mount::{
     raise_rollback_floor_to_the_admission_ceiling, RaiseToTheAdmissionCeiling, ShadowLedger,
@@ -84,7 +90,8 @@ fn object_birth_of(output: &TransactionOutput, inode: u64) -> Option<u64> {
 
 /// 新池新建文件之后建 inode 2、3、4（水位 5），再建一个 inode：开关把水位压回 3 时新建的那个拿到 3 号——
 /// inode 树里原来那条 3 号记录被原地换成新对象（对象出生代换成这次发布的 txg），记账里的水位写成 4，不大于树里最大的 key 4，
-/// I-9.6 判红；回退候选集里更早那一版还留着 3 号的旧对象，I-9.29（同号只许一个出生代） 也红。
+/// I-9.6 判红；回退候选集里更早那一版还留着 3 号的旧对象，I-9.29（同号只许一个出生代） 也红；建 2、3、4 那次留的台阶 (4, 5) 高过水位 4（①）、
+/// 3 号的新对象晚于它出生（②），I-9.30 也红。
 /// 同一段流开关关着时新建的拿 5 号、水位 6，I-9.6 成立、一条违例都没有。
 #[test]
 fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_reddens_the_watermark_invariant(
@@ -120,8 +127,12 @@ fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_r
     }
     assert_eq!(
         violated_invariants(&pressed_image),
-        vec!["I-9.6", ONE_OBJECT_BIRTH_PER_INODE_NUMBER],
-        "红 I-9.6 与 I-9.29"
+        vec![
+            "I-9.6",
+            ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
+            INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE
+        ],
+        "红 I-9.6、I-9.29 与 I-9.30"
     );
     match verdict_of(&pressed_image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER) {
         InvariantVerdict::Violated(detail) => assert!(
@@ -276,7 +287,8 @@ fn pool_whose_inode_three_is_unlinked_and_whose_floor_is_raised_past_its_record(
 /// 不抬 F 时池级 checker 还走着意图那一条根（3 号记录 nlink = 0 还在树里），树内最大 key 那一半先判出 3，说明写的是树那一半（S 的探针量过），
 /// 判不出墓碑那一半拦不拦得住。
 /// 判别力：墓碑那一半不比水位（`walk.rs` 里那一判恒真）时 I-9.6 成立，这一条红；关开关那一半由红转绿是判别力自证。
-/// 2 号被重发时回退候选集里抬 F 之后那几版（覆盖写三次的）还留着 2 号的旧对象，I-9.29（同号只许一个出生代） 也红。
+/// 2 号被重发时回退候选集里抬 F 之后那几版（覆盖写三次的）还留着 2 号的旧对象，I-9.29（同号只许一个出生代） 也红；
+/// 建 2、3 号留的台阶高过压回去的水位、2 号的新对象晚于它出生，I-9.30 也红。
 #[test]
 fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant(
 ) {
@@ -310,8 +322,12 @@ fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombs
     }
     assert_eq!(
         violated_invariants(&pressed_image),
-        vec!["I-9.6", ONE_OBJECT_BIRTH_PER_INODE_NUMBER],
-        "红 I-9.6 与 I-9.29"
+        vec![
+            "I-9.6",
+            ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
+            INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE
+        ],
+        "红 I-9.6、I-9.29 与 I-9.30"
     );
 
     let (mut control_pool, _) =
@@ -342,9 +358,10 @@ fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombs
 /// C479 变体 a（第 19 批 S 的探针：`/tmp/claude-1000/m3-batch19/s/probe/run2-probe.log` 那一形，I-9.6 拦不住的「压回的正是发过的最大号」）：
 /// 新池新建文件之后建 inode 2、3、4（水位 5），开关把水位压回 4 再建一个——4 号那条活记录被原地换成新对象（对象出生代换成这次发布的 txg），
 /// 记账的水位写成 5，仍大于树里最大的 key 4 与墓碑（没有）：I-9.6 成立。回退候选集里建 2、3、4 那一版还留着 4 号的旧对象，
-/// I-9.29（同号只许一个出生代） 判红，别的一条不红。
+/// I-9.29（同号只许一个出生代） 判红；建 2、3、4 那次留的台阶 (4, 5) 说 4 号在 txg 4 之前就发过，4 号的新对象晚于它出生，
+/// I-9.30 ② 也判红（水位 5 不低于任何一级台阶，① 成立）——I-9.6 拦不住的这一形 I-9.30 单凭最新根就判得出。别的一条不红。
 #[test]
-fn pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_only_the_one_birth_per_number_invariant(
+fn pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants(
 ) {
     let mut pressed_pool =
         pool_with_new_inodes("watermark-pressed-back-to-the-largest-issued-number", 3);
@@ -373,9 +390,26 @@ fn pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_redd
     );
     assert_eq!(
         violated_invariants(&pressed_image),
-        vec![ONE_OBJECT_BIRTH_PER_INODE_NUMBER],
-        "只红 I-9.29"
+        vec![
+            ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
+            INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE
+        ],
+        "红 I-9.29 与 I-9.30"
     );
+    match verdict_of(
+        &pressed_image,
+        INODE_NUMBER_WATERMARK_STEPS_AGREE_WITH_THE_WATERMARK_THE_RECORDS_AND_THE_LINEAGE,
+    ) {
+        InvariantVerdict::Violated(detail) => assert!(
+            detail.contains("inode 树里 4 号的对象出生代") && detail.contains("(4, 5)"),
+            "I-9.30 的说明点名重发的 4 号与它晚于的那一级台阶 (4, 5)：{detail}"
+        ),
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
+            panic!("4 号晚于台阶 (4, 5) 出生，I-9.30 应当判红，实际 {other_verdict:?}")
+        }
+    }
     match verdict_of(&pressed_image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER) {
         InvariantVerdict::Violated(detail) => assert!(
             detail.contains("inode 4 在回退候选集里对应不止一个对象出生代")
diff --git a/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
index e548e57..d9241cc 100644
--- a/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
+++ b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
@@ -1109,26 +1109,26 @@ fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
         ),
         tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
         lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container_capacity:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     }
 }
 
-/// 窄盘上 unlink 一个 14 个数据单元的文件、发完意图就崩（单元区 256 槽、不为环窗口补足，第一个文件之后覆盖写 2 次再顺序写长到 14 个单元），
-/// 与 `admission_raises_the_floor_before_refusing.rs` 里「挂载续做里一批被落得下那一判拒、推一串抬 F 之后走完」那一条同一种造法（那边第十位实现员的
-/// 草稿探针定的 256 槽、2 次、14 个单元）。交回崩溃那一刻的盘面与建池那一份参数。
-fn pool_crashed_after_the_unlink_intent_of_fourteen_data_units_on_narrow_devices(
+/// 窄盘上 unlink 一个 16 个数据单元的文件、发完意图就崩（单元区 292 槽、不为环窗口补足，第一个文件之后覆盖写 1 次再顺序写长到 16 个单元），
+/// 与 `admission_raises_the_floor_before_refusing.rs` 里「挂载续做里一批被落得下那一判拒、推一串抬 F 之后走完」那一条同一种造法（那边第 50 批补做
+/// 重扫定的 292 槽、1 次、16 个单元；此前第十位实现员定的 256 槽、2 次、14 个单元在退役树进准入之后取号之前就不够）。交回崩溃那一刻的盘面与建池那一份参数。
+fn pool_crashed_after_the_unlink_intent_of_sixteen_data_units_on_narrow_devices(
 ) -> (MemoryPool, MakeFilesystemParameters) {
     let mut pool =
         start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
             HistoryDeviceWidth::UnitAreaOf384Slots,
-            256,
+            292,
         );
-    for overwrite_index in 1..=2 {
-        pool.overwrite(OVERWRITE_BYTES)
-            .unwrap_or_else(|refusal| panic!("第 {overwrite_index} 次覆盖写做成：{refusal:?}"));
-    }
-    pool.sequential_write(13 * data_unit_payload_capacity() + 100)
-        .unwrap_or_else(|refusal| panic!("顺序写把文件长到 14 个单元：{refusal:?}"));
+    pool.overwrite(OVERWRITE_BYTES)
+        .unwrap_or_else(|refusal| panic!("那一次覆盖写做成：{refusal:?}"));
+    pool.sequential_write(15 * data_unit_payload_capacity() + 100)
+        .unwrap_or_else(|refusal| panic!("顺序写把文件长到 16 个单元：{refusal:?}"));
     let PoolUnderTest {
         session,
         devices,
@@ -1211,7 +1211,7 @@ fn after_a_mount_that_continued_the_deletion_the_two_tree_root_pointers_are_thos
 fn a_mount_whose_continuation_raised_the_floor_leaves_the_effective_floor_at_the_last_one_the_continuation_raised_to(
 ) {
     let (crash_image, parameters) =
-        pool_crashed_after_the_unlink_intent_of_fourteen_data_units_on_narrow_devices();
+        pool_crashed_after_the_unlink_intent_of_sixteen_data_units_on_narrow_devices();
     let (scenario, output) = mount_alone_with(&parameters, &crash_image, |parameters, devices| {
         mount_writable_with_test_only_switches_and_deletion_switches(
             parameters,
diff --git a/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs b/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
index 52a91ae..b49cd79 100644
--- a/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
+++ b/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
@@ -273,6 +273,7 @@ fn unit_length(identity: TransactionUnit) -> usize {
         | TransactionUnit::InodeLeafContainer(_)
         | TransactionUnit::TombstoneLeafContainer(_)
         | TransactionUnit::LineageLeafContainer(_)
+        | TransactionUnit::RetirementLeafContainer(_)
         | TransactionUnit::InstanceTable
         | TransactionUnit::InstanceTablePageAfterTheFirst(_) => 32768,
         TransactionUnit::ExtentRoot
@@ -290,6 +291,8 @@ fn unit_length(identity: TransactionUnit) -> usize {
         | TransactionUnit::TombstoneNodeBelowTheRoot(_)
         | TransactionUnit::LineageRoot
         | TransactionUnit::LineageNodeBelowTheRoot(_)
+        | TransactionUnit::RetirementRoot
+        | TransactionUnit::RetirementNodeBelowTheRoot(_)
         | TransactionUnit::TreeTable => 16384,
     }
 }
@@ -456,8 +459,8 @@ fn root_slots_system_configurations_and_journal_ring_hold_the_published_state()
         (3, 1, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, 0)
     );
     assert_eq!(
-        newest.tree_identifier_watermark, 65548,
-        "D8 已定项 11：第一个文件版本那次之后树 ID 水位是 65548"
+        newest.tree_identifier_watermark, 65549,
+        "D8 已定项 11：第一个文件版本那次之后树 ID 水位是 65549（十三棵树 65536..65548，退役树是第 50 批加的）"
     );
     assert_eq!(newest.record_bytes, pool.output.root.to_slot(512)[..457]);
     for (region, expected_txg) in [(1u64, 1u64), (2, 2)] {
@@ -566,8 +569,8 @@ fn root_slots_system_configurations_and_journal_ring_hold_the_published_state()
             );
             assert_eq!(
                 (view.new_tree_identifier_watermark, view.new_rollback_floor),
-                (65548, 0),
-                "新根段带的水位：十二棵树 65536..65547 之后是 65548（D8 已定项 11）"
+                (65549, 0),
+                "新根段带的水位：十三棵树 65536..65548 之后是 65549（D8 已定项 11）"
             );
             assert_eq!(
                 view.back_chain, E142_BACK_CHAIN_OF_NEW_POOL_FILE_CREATION,
@@ -695,8 +698,11 @@ fn every_index_node_self_checks_with_tight_ascending_keys_and_merkle_checksums_h
             | TransactionUnit::TombstoneNodeBelowTheRoot(_)
             | TransactionUnit::LineageLeafContainer(_)
             | TransactionUnit::LineageRoot
-            | TransactionUnit::LineageNodeBelowTheRoot(_) => {
-                unreachable!("新池新建文件不删除、不回退：删除那两棵树与谱系树零根")
+            | TransactionUnit::LineageNodeBelowTheRoot(_)
+            | TransactionUnit::RetirementLeafContainer(_)
+            | TransactionUnit::RetirementRoot
+            | TransactionUnit::RetirementNodeBelowTheRoot(_) => {
+                unreachable!("新池新建文件不删除、不回退：删除那两棵树、谱系树与退役树零根")
             }
         };
         match declared_key_range {
@@ -1196,8 +1202,8 @@ fn allocation_and_accounting_trees_carry_the_byte_table_numbers() {
     }
     assert_eq!(
         value_of(STATISTIC_COMMITTED_RESERVATION_BYTES),
-        2 * ((2 + 1) * 16384 + 41),
-        "已承诺预留（标签 6）按这一版现算：两份副本 × (3 槽常驻 + 41 字节 × 一个活对象)（D3 已定项 2，layout/03「记账标签 6 的值」）"
+        2 * ((2 + 1 + 2 + 1) * 16384 + 41 + 35),
+        "已承诺预留（标签 6）按这一版现算：两份副本 × (墓碑 3 槽 + 退役 3 槽常驻 + 41 字节 × 一个活对象 + 35 字节 × 一条 inode 记录)（D3 已定项 2，layout/03「记账标签 6 的值」；退役那一份是判决 m3-retire-r1 第三节第 6 条）"
     );
     assert_eq!(
         value_of(STATISTIC_DEFER_QUEUE_BYTES),
@@ -1345,36 +1351,41 @@ fn allocation_record_tree_nodes_carry_birth_sequences_zero_to_four_as_the_bump_o
     );
 }
 
-/// 树表 11 条（D18（块里携带什么信息） 已定项 24 第 6 条：7 → 11，`deleted_inodes`、墓碑树、谱系树、logged_ops 零根注册），
+/// 树表 12 条（D18（块里携带什么信息） 已定项 24 第 6 条：7 → 11，`deleted_inodes`、墓碑树、谱系树、logged_ops 零根注册；
+/// 退役树 11 → 12，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条），
 /// 树 ID、种类码、头 ID 三列逐条钉条款登记的值：树 ID 从 65536 起（D8（核心索引结构） 已定项 11），种类码 9..12 接在 8 之后（已定项 9），
 /// 头 ID 照已定项 8 / D5（快照 / 空间记账机制） 已定项 9（inode 树、extent 树、`deleted_inodes`、墓碑树写 inode 树的树 ID 65537，其余 0）；后四条根指针全零。
 #[test]
-fn tree_table_holds_eleven_entries_keyed_by_tree_identifier() {
+fn tree_table_holds_twelve_entries_keyed_by_tree_identifier() {
     let pool = build_pool("treetable");
     let entries = tree_table_entries_from_disk(&pool);
-    assert_eq!(entries.len(), 11, "D18 已定项 24 第 6 条：树表 7 → 11 条");
+    assert_eq!(
+        entries.len(),
+        12,
+        "D18 已定项 24 第 6 条树表 7 → 11 条，退役树（判决 m3-retire-r1 第三节第 1 条）11 → 12 条"
+    );
     assert_eq!(
         entries.iter().map(|entry| entry.tree.0).collect::<Vec<_>>(),
-        vec![65536, 65537, 65538, 65539, 65541, 65542, 65543, 65544, 65545, 65546, 65547],
-        "D8 已定项 11：树 ID 从 65536 起，中央映射树 65540 不进树表"
+        vec![65536, 65537, 65538, 65539, 65541, 65542, 65543, 65544, 65545, 65546, 65547, 65548],
+        "D8 已定项 11：树 ID 从 65536 起，中央映射树 65540 不进树表，退役树 65548 排最后"
     );
     assert_eq!(
         entries.iter().map(|entry| entry.kind).collect::<Vec<_>>(),
-        vec![1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12]
+        vec![1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13]
     );
     assert_eq!(
         entries
             .iter()
             .map(|entry| entry.head_identifier)
             .collect::<Vec<_>>(),
-        vec![65537, 65537, 0, 0, 0, 0, 0, 65537, 65537, 0, 0],
-        "D8 已定项 8 头 ID：inode 树、extent 树、deleted_inodes、墓碑树写 inode 树的树 ID，谱系树、logged_ops 与其余写 0"
+        vec![65537, 65537, 0, 0, 0, 0, 0, 65537, 65537, 0, 0, 65537],
+        "D8 已定项 8 头 ID：inode 树、extent 树、deleted_inodes、墓碑树、退役树写 inode 树的树 ID，谱系树、logged_ops 与其余写 0"
     );
     assert!(
         entries[7..]
             .iter()
             .all(|entry| entry.root == NodePointer::empty_root()),
-        "四棵新树零根注册：根指针全零"
+        "五棵新树零根注册：根指针全零"
     );
     assert!(entries
         .iter()
@@ -1383,8 +1394,8 @@ fn tree_table_holds_eleven_entries_keyed_by_tree_identifier() {
     let tree_table_unit = &pool.output.unit(TransactionUnit::TreeTable).bytes;
     assert_eq!(
         u16::from_le_bytes([tree_table_unit[8], tree_table_unit[9]]),
-        2200,
-        "声明长度 11 × 200"
+        2400,
+        "声明长度 12 × 200"
     );
     let view = index_node_view(tree_table_unit).expect("树表单元");
     assert_eq!(
@@ -1392,7 +1403,7 @@ fn tree_table_holds_eleven_entries_keyed_by_tree_identifier() {
         (0, 0, 0)
     );
     assert_eq!(view.smallest_key, 65536u64.to_le_bytes());
-    assert_eq!(view.largest_key, 65547u64.to_le_bytes());
+    assert_eq!(view.largest_key, 65548u64.to_le_bytes());
     // mkfs 的第 0 版树表还在原槽、0 条：COW 出新版本，不改旧的。
     let genesis_unit = read_unit_from(&pool, DeviceIdentity(0), TREE_TABLE_GENESIS_SLOT, 16384);
     assert_eq!(genesis_unit, pool.genesis.tree_table_genesis_unit);
diff --git a/crates/singlefs-harness/tests/overwrite_in_one_instance.rs b/crates/singlefs-harness/tests/overwrite_in_one_instance.rs
index 8654dd1..5a1a2a4 100644
--- a/crates/singlefs-harness/tests/overwrite_in_one_instance.rs
+++ b/crates/singlefs-harness/tests/overwrite_in_one_instance.rs
@@ -107,8 +107,8 @@ fn overwrite_publishes_the_second_version_through_the_same_commit_shape() {
     assert_eq!(second.root.checkpoint_txg, CheckpointTxg(4));
     assert_eq!(second.root.instance, InstanceGeneration(1));
     assert_eq!(
-        second.root.tree_identifier_watermark, 65548,
-        "没有建新树，水位不动（第一个文件版本发 65536..65547 之后是 65548，D8 已定项 11）"
+        second.root.tree_identifier_watermark, 65549,
+        "没有建新树，水位不动（第一个文件版本发 65536..65548 之后是 65549，D8 已定项 11）"
     );
     assert_eq!(second.root.rollback_floor, CheckpointTxg(0));
     assert_eq!(
@@ -317,10 +317,15 @@ fn release_rewrites_the_first_versions_records_and_accounting_moves_them_into_th
     let watermark = second
         .accounting_entries
         .iter()
-        .find(|entry| entry.statistic == STATISTIC_INODE_WATERMARK)
+        .filter(|entry| entry.statistic == STATISTIC_INODE_WATERMARK)
+        .max_by_key(|entry| entry.generation)
         .expect("inode 号水位");
     assert_eq!(watermark.value, 2, "没有建新 inode");
-    assert_eq!(second.accounting_entries.len(), 15);
+    assert_eq!(
+        second.accounting_entries.len(),
+        16,
+        "这一版自己的 15 行，加 A 那次留的一级 inode 号水位台阶 (A 的 txg, 2)（D5 已定项 2 射程）"
+    );
 
     assert_eq!(
         second.mapping_keys.len(),
@@ -333,7 +338,7 @@ fn release_rewrites_the_first_versions_records_and_accounting_moves_them_into_th
             "A 的映射条目一条都不留：释放走映射、条目删掉"
         );
     }
-    assert_eq!(second.tree_table_entries.len(), 11);
+    assert_eq!(second.tree_table_entries.len(), 12);
     for (entry, previous) in second
         .tree_table_entries
         .iter()
@@ -1055,10 +1060,10 @@ fn publish_running_out_of_space_midway_leaves_the_allocator_as_it_was() {
             .map(|device| DeviceShortOfDemand {
                 device,
                 available: AvailableBytesOnOneDevice(0),
-                demand: BytesOnOneDevice(8_912_896),
+                demand: BytesOnOneDevice(8_962_048),
             })
             .to_vec(),
-        "每块盘两个数据槽被数据单元取走之后没挡的槽剩 0（0 字节）< 第一个提交内生块要的 544 槽（8 912 896 字节；ckpt_cost 含谱系那一项 3 块）"
+        "每块盘两个数据槽被数据单元取走之后没挡的槽剩 0（0 字节）< 第一个提交内生块要的 547 槽（8 962 048 字节；ckpt_cost 含谱系、退役那两项各 3 块）"
     );
     assert_eq!(
         pool.allocator.records(),
diff --git a/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs b/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
index c9ddbf6..89372ad 100644
--- a/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
+++ b/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
@@ -1293,6 +1293,7 @@ fn violated_invariants_in_name_order(image: &MemoryPool) -> Vec<&'static str> {
 /// M2（判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K1 覆盖缺口那一行：I-9.29「与那一版谱系上墓碑的出生代都算」
 /// 墓碑那一侧没有用例钉）：2、3 号分两次建（对象出生代不同），3 号删完、根环转过一圈之后，那条墓碑的号 3 → 2、出生代不动（还是 3 号的）。
 /// 2 号的记录与谱系上的墓碑给出两个出生代：I-9.29 红；I-9.10 墓碑那一个等号按号认，同样红。墓碑的出生代与 2 号记录的不同，I-9.19 不判它。
+/// 建 2 号那次留的台阶说 2 号在那之前就发过，谱系上这条墓碑（2 号、3 号的对象出生代）晚于它出生：I-9.30 ② 也红（第 49 批）。
 #[test]
 fn a_tombstone_on_the_lineage_carrying_another_object_birth_for_a_live_number_reddens_the_one_birth_per_number_invariant(
 ) {
@@ -1314,8 +1315,8 @@ fn a_tombstone_on_the_lineage_carrying_another_object_birth_for_a_live_number_re
     });
     assert_eq!(
         violated_invariants_in_name_order(&image),
-        vec!["I-9.10", ONE_OBJECT_BIRTH_PER_INODE_NUMBER],
-        "2 号的记录与谱系上的墓碑两个出生代：红 I-9.29 与 I-9.10 墓碑那一个等号"
+        vec!["I-9.10", ONE_OBJECT_BIRTH_PER_INODE_NUMBER, "I-9.30"],
+        "2 号的记录与谱系上的墓碑两个出生代：红 I-9.29 与 I-9.10 墓碑那一个等号；墓碑晚于建 2 号那级台阶出生，红 I-9.30"
     );
     let detail = first_violation_of(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER);
     assert!(
diff --git a/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs b/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
index 570f025..9721873 100644
--- a/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
+++ b/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
@@ -1,7 +1,7 @@
 //! 池级 checker 的两条（C673（三 oracle 合谋测试的补 oracle 改法没做） ⑤ ⑥，判决 `research/prompts/c673-clauses-r1-main-verification.md`
 //! 第二节 K1、K2）：
 //! - I-9.27（树表条目的头 ID 取值）照同一张树表里的关系判、不拿字面 65537 比：崩溃恢复丢掉实例 2 的第一个文件版本之后，
-//!   实例 3 从根环水位 65548 发树（inode 树 65549）的那段合法历史上一条不红；
+//!   实例 3 从根环水位 65549 发树（inode 树 65550）的那段合法历史上一条不红；
 //! - I-5.5（从 mkfs 水位建树那一版的 t1 落点）在新池新建文件（默认 768 MiB 环与 128 MiB 环）、只暖机过的实例 2 写第一个文件的那一版上
 //!   真被评估过且成立；第一个文件版本离开根环之后只剩覆盖写那几版（诞生 txg 不等于这一版的 txg），一版都不判。
 //! - I-5.5 的判据按这一版的分配记录现算（用户 2026-10-04 定，判决 `research/prompts/m3-formal-code-b17-r1-main-verification.md`
@@ -174,7 +174,7 @@ fn assert_the_first_data_unit_placement_holds_and_nothing_reddens(
 /// 攻方原型 `k1_recovered_first_file_version_issues_trees_from_the_ring_watermark` 那段历史（`writable_mount_of_a_formatted_pool.rs`
 /// 那条用例的内存盘版本）：只做过 mkfs 的池可写挂载两次（实例 1 只暖机，实例 2 写行、暖机），实例 2 发第一个文件版本（txg 5，树 65536..65547），
 /// 崩在 txg 5 的根槽 FUA 之后、系统配置轮换之前；可写挂载一次，txg 5 的根槽与它那条记录点名的数据单元盘 0 那一份读不出——择根落到 (2, 4)，
-/// 实例 3 的水位按根环与环里记录取 max = 65548，从它发第一个文件版本（inode 树 65549，头 ID 都是 65549）。故障解除之后接着覆盖写，
+/// 实例 3 的水位按根环与环里记录取 max = 65549，从它发第一个文件版本（inode 树 65549，头 ID 都是 65549）。故障解除之后接着覆盖写，
 /// 直到被丢掉的 (2, 5) 那一槽被新根盖掉（不留着它：它引用的数据单元槽被实例 3 复用了，那一形是 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到））。
 /// 这份镜像合法：池级 checker 一条不红，I-9.27 真被评估过且成立（字面 65537 的读法在这里误红 4 条）；树 ID 不从 65536 起，I-5.5 一版都不判。
 #[test]
@@ -258,8 +258,8 @@ fn a_first_file_version_issuing_its_trees_from_the_ring_watermark_after_a_recove
         panic!("落到实例 2 的暖机根：现行那一版树表 0 条")
     };
     assert_eq!(
-        version_of_the_third_instance.root.tree_identifier_watermark, 65548,
-        "实例 3 那一版带根环与环里记录的水位 max：实例 2 的记录带的 65548"
+        version_of_the_third_instance.root.tree_identifier_watermark, 65549,
+        "实例 3 那一版带根环与环里记录的水位 max：实例 2 的记录带的 65549"
     );
     let mut allocator_of_the_third_instance = third_instance.allocator.clone();
     let first_file_of_the_third_instance = {
@@ -276,11 +276,11 @@ fn a_first_file_version_issuing_its_trees_from_the_ring_watermark_after_a_recove
             third_instance.output.instance,
             &version_of_the_third_instance.record_bytes,
         )
-        .expect("水位 65548、树表 0 条：照样发得出第一个文件版本")
+        .expect("水位 65549、树表 0 条：照样发得出第一个文件版本")
     };
     assert_eq!(
-        first_file_of_the_third_instance.tree_identifiers.inode.0, 65549,
-        "实例 3 从 65548 发树：extent 树 65548、inode 树 65549"
+        first_file_of_the_third_instance.tree_identifiers.inode.0, 65550,
+        "实例 3 从 65549 发树：extent 树 65549、inode 树 65550"
     );
     let mut previous = first_file_of_the_third_instance;
     while previous.root.checkpoint_txg.0 < dropped_txg.0 + ROOT_RING_SLOT_COUNT {
@@ -311,7 +311,7 @@ fn a_first_file_version_issuing_its_trees_from_the_ring_watermark_after_a_recove
     assert_eq!(
         verdict_of(&image, HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES),
         InvariantVerdict::Holds,
-        "I-9.27 在头 ID 65549 的树表上真被评估过且成立"
+        "I-9.27 在头 ID 65550 的树表上真被评估过且成立"
     );
     assert!(
         matches!(
@@ -393,8 +393,8 @@ fn the_first_file_of_an_instance_after_a_warm_up_only_mount_is_recognized_and_ho
             first_file.root.checkpoint_txg,
             first_file.tree_identifiers.highest().0
         ),
-        (InstanceGeneration(2), CheckpointTxg(5), 65547),
-        "实例 2 的第一个文件版本 txg 5，树从 mkfs 水位发 65536..65547"
+        (InstanceGeneration(2), CheckpointTxg(5), 65548),
+        "实例 2 的第一个文件版本 txg 5，树从 mkfs 水位发 65536..65548"
     );
     assert_the_first_data_unit_placement_holds_and_nothing_reddens(
         &memory_pool_of_sparse_devices(&devices),
diff --git a/crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs b/crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs
index d9b7d0c..6178f8d 100644
--- a/crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs
+++ b/crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs
@@ -48,8 +48,8 @@ const FILE_BYTES: usize = 3000;
 const TREE_IDENTIFIER_OF_THE_FIRST_EXTENT_TREE: u64 = 65536;
 /// 改坏一片树表时翻它载荷里的哪一个字节（头之后、条目区里：两份位置条目读出来整单元校验和都对不上）。
 const BYTE_INSIDE_THE_PAYLOAD_OF_A_TREE_TABLE: u64 = 4000;
-/// 第一个文件那一版的树 ID 水位：mkfs 水位 65536 之后发 12 个号（`th-c342-sysconf.out` 里实例 1 那一版与实例 2 重发那一版都是 65548）。
-const TREE_IDENTIFIER_WATERMARK_AFTER_THE_FIRST_FILE: u64 = 65548;
+/// 第一个文件那一版的树 ID 水位：mkfs 水位 65536 之后发 13 个号（第 50 批加了退役树，此前 12 个、水位 65548）（`th-c342-sysconf.out` 里实例 1 那一版与实例 2 重发那一版都是 65548）。
+const TREE_IDENTIFIER_WATERMARK_AFTER_THE_FIRST_FILE: u64 = 65549;
 
 type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;
 
diff --git a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs
index e609533..29c3d4b 100644
--- a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs
+++ b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_no_valid_root_is_read.rs
@@ -76,6 +76,8 @@ fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
         ),
         tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
         lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container_capacity:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     }
 }
diff --git a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs
index 26da3a5..1b53085 100644
--- a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs
+++ b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs
@@ -95,6 +95,8 @@ fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
         ),
         tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
         lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container_capacity:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     }
 }
diff --git a/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs b/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
index c390246..29f093d 100644
--- a/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
+++ b/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
@@ -12,7 +12,9 @@ mod common_corrupted_allocation_record;
 
 use common::{
     abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, disk_snapshot,
-    file_content, geometry, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
+    file_content, geometry, parameters,
+    rollback_after_a_recovery_abandoned_a_root_that_created_inodes, BuiltPool, Recorded,
+    RollbackAfterARecoveryAbandonedARootThatCreatedInodes, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
 };
 use common_corrupted_allocation_record::move_the_first_allocation_record_past_the_end_of_the_unit_area_and_reseal_the_chain;
 use singlefs_checker::image::InvariantVerdict;
@@ -1430,8 +1432,8 @@ fn with_the_file_version_root_slot_unreadable_the_ring_watermark_still_comes_fro
     assert!(
         records
             .values()
-            .any(|record| record.new_tree_identifier_watermark == 65548),
-        "(1, 3) 那次发布的记录还在环里，新根段带 65548（十二棵树 65536..65547 之后）"
+            .any(|record| record.new_tree_identifier_watermark == 65549),
+        "(1, 3) 那次发布的记录还在环里，新根段带 65549（十三棵树 65536..65548 之后）"
     );
     assert_eq!(
         highest_tree_identifier_watermark_in_the_ring(
@@ -1441,8 +1443,8 @@ fn with_the_file_version_root_slot_unreadable_the_ring_watermark_still_comes_fro
             &immutable.filesystem_identifier,
             &records,
         ),
-        Some(65548),
-        "根读不出、记录还在：水位按记录带的 65548 算，不退回读得出的根带的 65536"
+        Some(65549),
+        "根读不出、记录还在：水位按记录带的 65549 算，不退回读得出的根带的 65536"
     );
 }
 
@@ -2126,3 +2128,188 @@ fn plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_
         "恢复之后重开",
     );
 }
+
+/// 一版记账里 inode 号水位那一统计量的每一行 (代段, 值)，按 key 升序（与盘上那棵记账树同序）。
+fn inode_number_watermark_rows_of(version: &TransactionOutput) -> Vec<(u64, u64)> {
+    version
+        .accounting_entries
+        .iter()
+        .filter(|entry| entry.statistic == singlefs_core::records::STATISTIC_INODE_WATERMARK)
+        .map(|entry| (entry.generation.0, entry.value))
+        .collect()
+}
+
+/// 一版的台阶行 (g, H)（`TransactionOutput::inode_number_watermark_steps`）。
+fn inode_number_watermark_steps_of(version: &TransactionOutput) -> Vec<(u64, u64)> {
+    version
+        .inode_number_watermark_steps()
+        .iter()
+        .map(|step| (step.generation.0, step.value))
+        .collect()
+}
+
+/// 管理员回退取 inode 号水位只取按 cur 的实例表没被崩溃恢复抛弃的根（D23（journal 的角色与格式） 已定项 14「水位」一格的 Opt2，用户 2026-10-08 Y1）：
+/// `common::rollback_after_a_recovery_abandoned_a_root_that_created_inodes` 那段历史——建过三个 inode、水位 5 的那一版被崩溃恢复抛弃、根读得出、
+/// 从盘上重建得出水位 5（搭建里自证），实例 2 再建一个（2 号重发，水位 3），再回退到 A：回退那次发布的水位是
+/// max(cur 那一版内存里的 3, 没被抛弃的根的) = 3，不跳到被抛弃那一版的 5。环里读得出的根全都算时（改之前的取法，
+/// `crates/mutations.tsv` 里「W3 Y1 Opt2」那一行）是 5，这一条红。台阶行照抄 cur 的：(A, 2)、(重发那次, 3)，回退那次水位没变、不留新台阶；
+/// 谱系记录带 H_cur = 3。池级 checker 一条违例都没有，I-9.30（台阶行与水位、记录、谱系对得上） 与 I-3.13 真被判过且成立。
+#[test]
+fn a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root(
+) {
+    let RollbackAfterARecoveryAbandonedARootThatCreatedInodes {
+        pool,
+        first_file_version,
+        abandoned,
+        created_again,
+        rollback,
+    } = rollback_after_a_recovery_abandoned_a_root_that_created_inodes(
+        "rollback-watermark-skips-a-root-abandoned-by-a-recovery",
+    );
+    assert_eq!(
+        (
+            abandoned.inode_number_watermark(),
+            created_again.inode_number_watermark()
+        ),
+        (5, 3),
+        "被抛弃那一版水位 5；崩溃恢复落到 A 之后再建一个，2 号重发、水位 3"
+    );
+    assert_eq!(
+        rollback.inode_number_watermark(),
+        3,
+        "回退那次发布的 inode 号水位 = max(cur 内存里的 3, 没被抛弃的根的)：不取被抛弃那一版的 5"
+    );
+    assert_eq!(
+        inode_number_watermark_steps_of(&rollback),
+        vec![
+            (first_file_version.root.checkpoint_txg.0, 2),
+            (created_again.root.checkpoint_txg.0, 3)
+        ],
+        "台阶行照抄 cur 的；回退那次水位没变，不留新台阶"
+    );
+    assert_eq!(
+        rollback
+            .lineage_containers
+            .iter()
+            .flat_map(|container| container.contents.records.iter())
+            .map(|record| record.inode_number_watermark_of_the_version_rolled_back_from)
+            .collect::<Vec<u64>>(),
+        vec![3],
+        "谱系记录带 H_cur = cur 的水位 3"
+    );
+    let verdicts =
+        verdicts_without_any_violation(&pool.memory_pool(), "崩溃恢复抛弃、再建、再回退之后");
+    assert_judged_and_holding(
+        &verdicts,
+        &["I-9.30", "I-3.13"],
+        "崩溃恢复抛弃、再建、再回退之后",
+    );
+}
+
+/// inode 号水位的台阶行（D5（快照 / 空间记账机制） 已定项 2 射程，W3 的 C2aL 形态，用户 2026-10-08 Y2）：A（第一个文件，txg 3，水位 2）、
+/// B（建两个空 inode，txg 4，水位 4）、C（覆盖写，txg 5）、D（建一个，txg 6，水位 5）、挂着的时候回退到 B（E，txg 7）。
+/// 建 inode 的发布留一级台阶 (那次的 txg, 那次之后的水位)，就是它写的那一行；别的发布照抄、自己那一行不是台阶，下一次发布不带它；
+/// 回退那次照抄 cur（D）的，不取 B 的。逐版钉 inode 号水位那一统计量的每一行 (代段, 值)、台阶与记账行数（15 + 早于这一版的台阶数），
+/// 从盘上重建读回的与写出的相同（`recovery::rebuild_version` 认得台阶行：记账行数那一判不把它们算进 3 + 6 × 盘数）。
+#[test]
+fn inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback(
+) {
+    let mut pool = build_pool("inode-number-watermark-steps");
+    let first = pool.output.clone();
+    let publish_new_inodes_in_process = |pool_to_publish_on: &mut BuiltPool, count: u64| {
+        let publish_parameters = parameters();
+        let previous = pool_to_publish_on.output.clone();
+        let devices = pool_to_publish_on.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        let output = publish_new_inodes(
+            &mut writer,
+            &mut pool_to_publish_on.allocator,
+            &previous,
+            count,
+            FIXED_WRITE_TIME_SECONDS + 120,
+            InstanceGeneration(1),
+        )
+        .expect("建空 inode");
+        pool_to_publish_on.output = output.clone();
+        output
+    };
+    let second = publish_new_inodes_in_process(&mut pool, 2);
+    let third = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
+    let fourth = publish_new_inodes_in_process(&mut pool, 1);
+    roll_back(
+        &mut pool,
+        RollbackTarget {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: second.root.checkpoint_txg,
+        },
+    )
+    .expect("挂着的时候回退到 B");
+    let rollback = pool.output.clone();
+    assert_eq!(
+        [&first, &second, &third, &fourth, &rollback].map(|version| version.root.checkpoint_txg.0),
+        [3, 4, 5, 6, 7],
+        "A、B、C、D、E 的 txg"
+    );
+    assert_eq!(
+        [&first, &second, &third, &fourth, &rollback].map(inode_number_watermark_rows_of),
+        [
+            vec![(3, 2)],
+            vec![(3, 2), (4, 4)],
+            vec![(3, 2), (4, 4), (5, 4)],
+            vec![(3, 2), (4, 4), (6, 5)],
+            vec![(3, 2), (4, 4), (6, 5), (7, 5)],
+        ],
+        "inode 号水位那一统计量的行 (代段, 值)：每一版自己那一行加照抄来的台阶"
+    );
+    assert_eq!(
+        [&first, &second, &third, &fourth, &rollback].map(inode_number_watermark_steps_of),
+        [
+            vec![(3, 2)],
+            vec![(3, 2), (4, 4)],
+            vec![(3, 2), (4, 4)],
+            vec![(3, 2), (4, 4), (6, 5)],
+            vec![(3, 2), (4, 4), (6, 5)],
+        ],
+        "台阶：建 inode 的发布各留一级（第一个文件那一版从没有水位到 2 也算一级），覆盖写与回退不留"
+    );
+    assert_eq!(
+        [&first, &second, &third, &fourth, &rollback]
+            .map(|version| version.accounting_entries.len()),
+        [15, 16, 17, 17, 18],
+        "记账行数 = 每一版自己的 15 行 + 早于这一版 txg 的台阶数"
+    );
+    assert_eq!(
+        (
+            rollback.inode_number_watermark(),
+            rollback
+                .lineage_containers
+                .iter()
+                .flat_map(|container| container.contents.records.iter())
+                .map(|record| record.inode_number_watermark_of_the_version_rolled_back_from)
+                .collect::<Vec<u64>>(),
+        ),
+        (5, vec![5]),
+        "回退到 B（水位 4）那一版的水位与谱系记录的 H_cur 都是 cur（D）的 5"
+    );
+    let rebuilt = match singlefs_core::recovery::rebuild_version(
+        &pool.memory_pool(),
+        &rollback.root,
+        Some(rollback.record.clone()),
+    ) {
+        Ok(
+            singlefs_core::recovery::RebuiltVersion::WithFile(version)
+            | singlefs_core::recovery::RebuiltVersion::WithTreesAndNoFile(version),
+        ) => version,
+        other => panic!("回退那一版从盘上重建：{other:?}"),
+    };
+    assert_eq!(
+        (
+            inode_number_watermark_rows_of(&rebuilt),
+            rebuilt.inode_number_watermark()
+        ),
+        (vec![(3, 2), (4, 4), (6, 5), (7, 5)], 5),
+        "从盘上重建读回的台阶行与现行值与写出的相同"
+    );
+    let verdicts = verdicts_without_any_violation(&pool.memory_pool(), "回退到 B 之后");
+    assert_judged_and_holding(&verdicts, &["I-9.30", "I-3.13"], "回退到 B 之后");
+}
diff --git a/crates/singlefs-harness/tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs b/crates/singlefs-harness/tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs
index 2b13f68..26a1b1c 100644
--- a/crates/singlefs-harness/tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs
+++ b/crates/singlefs-harness/tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs
@@ -1,13 +1,13 @@
 //! 管理员回退那次发布追加一条谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条、D23（journal 的角色与格式） 已定项 14）：
 //! 谱系容器（码 3 打包记录类型 5，容器号同墓碑规则：零根时 1）与谱系树根（码 2 层级 1）在回退那次写出、从盘上重建读得回，
-//! 记录 (T_rb, T_old) 逐字段相等；写账另立一种 `LineageTreeNode`（用户 2026-10-02 定）；ckpt_cost 谱系那一项按一次谱系容器 COW
+//! 记录 (T_rb, T_old, H_cur) 逐字段相等（H_cur 是 cur 那一版的 inode 号水位，W3 第 49 批）；写账另立一种 `LineageTreeNode`（用户 2026-10-02 定）；ckpt_cost 谱系那一项按一次谱系容器 COW
 //! 连祖先算（容器 2 + 根 1 + 内部层数，用户 2026-10-02 定「乙」）。池是两块 4 GiB 盘。
 
 mod common;
 
 use common::{
     build_pool, file_content, overwrite_in_process_as_instance_one_and_advance_the_pool,
-    parameters, BuiltPool,
+    parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
 };
 use singlefs_checker::image::InvariantVerdict;
 use singlefs_checker::walk::check_pool_image;
@@ -24,7 +24,10 @@ use singlefs_core::records::LineageRecord;
 use singlefs_core::recovery::{
     rebuild_version, recover, JournalPolicy, RebuiltVersion, RecoveryOutcome,
 };
-use singlefs_core::transaction::{CodeTwoTreeNodeCapacities, TransactionOutput, TransactionUnit};
+use singlefs_core::transaction::{
+    publish_new_inodes, CodeTwoTreeNodeCapacities, MultiLevelCodeTwoTree, PoolWriter,
+    TransactionOutput, TransactionUnit,
+};
 use singlefs_core::unit::{PackedIdentity, PACKED_TYPE_LINEAGE};
 use singlefs_core::write_accounting::{WriteCallsAndBytes, WrittenStructureKind};
 use singlefs_format::TREE_IDENTIFIER_LINEAGE;
@@ -86,7 +89,8 @@ fn lineage_records_of(version: &TransactionOutput) -> Vec<LineageContainer> {
         .collect()
 }
 
-/// 第一次回退（零根上）：谱系树开容器号 1、出生代 = 回退那次的 txg、类型 5，装一条 (T_rb = 回退那次的 txg, T_old = R_old 的 txg)；
+/// 第一次回退（零根上）：谱系树开容器号 1、出生代 = 回退那次的 txg、类型 5，装一条 (T_rb = 回退那次的 txg, T_old = R_old 的 txg,
+/// H_cur = cur 的 inode 号水位 2)；
 /// 这次重写谱系容器与谱系树根各一个，写账记在 `LineageTreeNode` 那一种（两块盘各一份：4 次写、2 × (32768 + 16384) 字节）；
 /// 从盘上重建读回的谱系容器逐字段相等；冷启动恢复的走读（映射条目数把谱系树的容器与根也算上）读回第一个文件；池级 checker 全绿。
 #[test]
@@ -109,6 +113,7 @@ fn a_rollback_appends_one_lineage_record_that_reads_back_field_by_field() {
         records: vec![LineageRecord {
             rollback_txg,
             rollback_target_txg: first_file_version.root.checkpoint_txg,
+            inode_number_watermark_of_the_version_rolled_back_from: 2,
         }],
     }];
     assert_eq!(
@@ -159,7 +164,7 @@ fn a_rollback_appends_one_lineage_record_that_reads_back_field_by_field() {
     assert_eq!(violations(&pool), Vec::<String>::new(), "池级 checker 全绿");
 }
 
-/// 第二次回退追加进同一片开放容器（一片装 2039 条）：容器号仍 1、出生代仍是第一次回退那次，两条记录按先后排；之间的发布照抄谱系树。
+/// 第二次回退追加进同一片开放容器（一片装 1359 条）：容器号仍 1、出生代仍是第一次回退那次，两条记录按先后排；之间的发布照抄谱系树。
 #[test]
 fn a_second_rollback_appends_to_the_same_open_lineage_container() {
     let (mut pool, first_file_version) =
@@ -198,10 +203,12 @@ fn a_second_rollback_appends_to_the_same_open_lineage_container() {
                 LineageRecord {
                     rollback_txg: first_rollback.root.checkpoint_txg,
                     rollback_target_txg: first_file_version.root.checkpoint_txg,
+                    inode_number_watermark_of_the_version_rolled_back_from: 2,
                 },
                 LineageRecord {
                     rollback_txg: second_rollback_txg,
                     rollback_target_txg: first_rollback.root.checkpoint_txg,
+                    inode_number_watermark_of_the_version_rolled_back_from: 2,
                 },
             ],
         ),
@@ -210,9 +217,67 @@ fn a_second_rollback_appends_to_the_same_open_lineage_container() {
     assert_eq!(violations(&pool), Vec::<String>::new(), "池级 checker 全绿");
 }
 
+/// 谱系记录的 H_cur 从 cur（回退之前的现行那一版）读，不取回退目标的（D18（块里携带什么信息） 已定项 24 第 5 条，W3 判决
+/// `research/prompts/m3-w3-r2-main-verification.md` 第二节攻方 K3）：第一个文件那一版（txg 3，水位 2）之后建两个空 inode（txg 4，水位 4）、
+/// 再建一个（txg 5，水位 5），挂着的时候回退到建两个的那一版（水位 4）：谱系记录 (6, 4, 5)——H_cur 是 cur 的 5，不是回退目标的 4。
+/// 从盘上重建读回的同一条；池级 checker 一条违例都没有（I-9.30 谱系那一判拿它比台阶）。
+#[test]
+fn a_rollback_records_the_inode_number_watermark_of_the_version_it_rolls_back_from_not_of_its_target(
+) {
+    let mut pool = build_pool("rollback-lineage-record-watermark-of-the-current-version");
+    let mut create_inodes = |count: u64| {
+        let publish_parameters = parameters();
+        let previous = pool.output.clone();
+        let devices = pool.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        let output = publish_new_inodes(
+            &mut writer,
+            &mut pool.allocator,
+            &previous,
+            count,
+            FIXED_WRITE_TIME_SECONDS + 60,
+            InstanceGeneration(1),
+        )
+        .expect("建空 inode");
+        pool.output = output.clone();
+        output
+    };
+    let target = create_inodes(2);
+    let current = create_inodes(1);
+    assert_eq!(
+        (
+            target.root.checkpoint_txg,
+            target.inode_number_watermark(),
+            current.root.checkpoint_txg,
+            current.inode_number_watermark()
+        ),
+        (CheckpointTxg(4), 4, CheckpointTxg(5), 5),
+        "回退目标 txg 4、水位 4；cur txg 5、水位 5"
+    );
+    roll_back_to(&mut pool, &target);
+    let expected = vec![LineageRecord {
+        rollback_txg: CheckpointTxg(6),
+        rollback_target_txg: CheckpointTxg(4),
+        inode_number_watermark_of_the_version_rolled_back_from: 5,
+    }];
+    let records_of = |version: &TransactionOutput| -> Vec<LineageRecord> {
+        lineage_records_of(version)
+            .into_iter()
+            .flat_map(|container| container.records)
+            .collect()
+    };
+    assert_eq!(
+        (records_of(&pool.output), records_of(&rebuilt_from_disk(&pool))),
+        (expected.clone(), expected),
+        "谱系记录 (T_rb 6, T_old 4, H_cur 5)：H_cur 是 cur 的水位，不是回退目标的 4；从盘上重建读回同一条"
+    );
+    assert_eq!(violations(&pool), Vec::<String>::new(), "池级 checker 全绿");
+}
+
 /// ckpt_cost 谱系那一项：谱系树零根（回退之前）与只有根（回退之后）都是 3 块（容器 2 + 根 1，内部 0 层），长高之后多一层多一块
 /// （第一版的谱系树还不长高，长高那一格只在 `checkpoint_cost_of_one_lineage_container_cow` 的参数上取）。
-/// 第一个文件那一版整个 ckpt_cost 的绝对值 = 分配记录树 529 + 中央映射树那一项 + 记账树节点 + 树表 1 + 谱系 3。
+/// 第一个文件那一版整个 ckpt_cost 的绝对值 = 分配记录树 529 + 中央映射树那一项 + 记账树节点 + 树表 1 + 谱系 3 + 退役 3
+/// （退役那一项是判决 `research/prompts/m3-retire-r1-main-verification.md` 第六节 R2「两样都做」加的）。
 #[test]
 fn the_lineage_term_of_the_checkpoint_cost_is_three_blocks_on_a_zero_root_and_on_a_root_only_tree()
 {
@@ -238,7 +303,7 @@ fn the_lineage_term_of_the_checkpoint_cost_is_three_blocks_on_a_zero_root_and_on
             first_file_version.accounting_tree.node_count(),
             checkpoint_cost_of_the_version_to_build_on(Some(&first_file_version), &pool.allocator),
         ),
-        (1, MetadataBlocks(529 + 6 + 1 + 1 + 3)),
+        (1, MetadataBlocks(529 + 6 + 1 + 1 + 3 + 3)),
         "第一个文件那一版：分配记录树 529、中央映射树那一项 6、记账树 1、树表 1、谱系 3"
     );
     assert_eq!(
@@ -252,6 +317,8 @@ fn the_lineage_term_of_the_checkpoint_cost_is_three_blocks_on_a_zero_root_and_on
 /// 谱系零根（回退之前）与只有根（回退之后）两版都多算容器 1 把、根 1 把（[`central_mapping_entries_of_one_lineage_container_cow`]）。
 /// 产品容量（叶 294、内部 143）下这两版那一项的切分次数不随删插数多 2 把而变（529 + 1 + 2 把插进一片叶，切 4 次都装得下），量不出这几把；
 /// 中央映射树节点容量按只供测试的开关压到叶 4、内部节点 4（每切一次至少再插 2 把），两版的 ckpt_cost 绝对值钉在草稿探针现跑的数上。
+/// 记账树照产品容量（ckpt_cost 记账树那一项按传进去的容量规划这次发布之后的树，第 49 批 W3 补做）：压到叶 4 时两版各 15 行的记账树
+/// 那一项跟着变成多片叶，量的就不只是谱系那几把了。
 #[test]
 fn the_central_mapping_term_counts_the_lineage_entries_of_one_cow_on_a_zero_root_and_on_a_root_only_tree(
 ) {
@@ -259,10 +326,7 @@ fn the_central_mapping_term_counts_the_lineage_entries_of_one_cow_on_a_zero_root
         "central-mapping-term-counts-the-lineage-entries",
     );
     let capped = CodeTwoTreeNodeCapacities::CappedForTests {
-        accounting: CodeTwoTreeNodeCapacity {
-            leaf_entries: 4,
-            internal_entries: 4,
-        },
+        accounting: MultiLevelCodeTwoTree::Accounting.node_capacity_of_the_node_format(),
         central_mapping: CodeTwoTreeNodeCapacity {
             leaf_entries: 4,
             internal_entries: 4,
@@ -301,6 +365,7 @@ fn the_central_mapping_term_counts_the_lineage_entries_of_one_cow_on_a_zero_root
     );
 }
 
-/// 草稿探针现跑的数（三·2 第 8 位实现员副本上）：叶 4、内部 4 时两版都是 1062（叶 3 / 6 / 10 时 1064 / 803 / 670，产品容量 540）。
-const CHECKPOINT_COST_OF_THE_ZERO_ROOT_VERSION_UNDER_THE_CAPPED_MAPPING: u64 = 1062;
-const CHECKPOINT_COST_OF_THE_ROOT_ONLY_VERSION_UNDER_THE_CAPPED_MAPPING: u64 = 1062;
+/// 草稿探针现跑的数（三·2 第 8 位实现员副本上）：叶 4、内部 4 时两版都是 1062（叶 3 / 6 / 10 时 1064 / 803 / 670，产品容量 540）；
+/// 第 50 批加了退役那一项（容器 2 + 根 1 块，它在中央映射树里多删插的 2 把在叶 4 下又多切出 3 个节点），第 50 批实现员副本上现跑是 1068。
+const CHECKPOINT_COST_OF_THE_ZERO_ROOT_VERSION_UNDER_THE_CAPPED_MAPPING: u64 = 1068;
+const CHECKPOINT_COST_OF_THE_ROOT_ONLY_VERSION_UNDER_THE_CAPPED_MAPPING: u64 = 1068;
diff --git a/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs b/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
index 44d72a3..074827b 100644
--- a/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
+++ b/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
@@ -55,6 +55,8 @@ fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
         ),
         tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
         lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container_capacity:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     }
 }
diff --git a/crates/singlefs-harness/tests/unequal_devices.rs b/crates/singlefs-harness/tests/unequal_devices.rs
index 30a6bee..cacb5e0 100644
--- a/crates/singlefs-harness/tests/unequal_devices.rs
+++ b/crates/singlefs-harness/tests/unequal_devices.rs
@@ -403,10 +403,10 @@ fn devices_answering_different_places_for_a_commit_generated_block_are_refused_b
             vec![DeviceShortOfDemand {
                 device: DeviceIdentity(1),
                 available: AvailableBytesOnOneDevice(16_384),
-                demand: BytesOnOneDevice(7_602_176),
+                demand: BytesOnOneDevice(7_651_328),
             }],
-            "准入先拒（C545）：小盘数据单元取走那一对之后没挡的槽剩 1（16 384 字节）< 第一个提交内生块要的 464 槽\
-             （extent 根 + inode 叶容器 + inode 根 + ckpt_cost（含谱系那一项 3 块），7 602 176 字节），只报盘 1"
+            "准入先拒（C545）：小盘数据单元取走那一对之后没挡的槽剩 1（16 384 字节）< 第一个提交内生块要的 467 槽\
+             （extent 根 + inode 叶容器 + inode 根 + ckpt_cost（含谱系、退役那两项各 3 块），7 651 328 字节），只报盘 1"
         ),
         other => panic!("extent 树根的落点该被拒（准入先拒）：{:?}", other.as_ref().err()),
     }
diff --git a/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs b/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs
index 310c415..1ce8b20 100644
--- a/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs
+++ b/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs
@@ -311,7 +311,10 @@ fn crc32_of_the_images(devices: &[(DeviceIdentity, SparseBlockDevice)], device_b
 /// 2026-10-05 T_time 的盘上单位从秒改成毫秒（用户定案，判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2）：
 /// 两块盘每个系统配置槽偏移 [433, 437) 从 5 换成 5000，连带整槽校验和；钉值随之换成 1329175690（第 30 批 30-3 实现员副本上这条断言
 /// 红出来的 left 值；同一份副本只把 core 那一处退回秒时这一条绿，差异只来自这一改）。
-const DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32: u32 = 1_329_175_690;
+/// 第 50 批退役树 day-1 零根注册（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）：树表 11 → 12 条（声明长度 2400、
+/// 末条是退役树、key 区间末端 65548）、根记录与新根段里的树 ID 水位 65548 → 65549、记账标签 6 加退役那一份，连带各处校验和；
+/// 钉值随之换成 3497143766（第 50 批实现员副本上这条断言红出来的 left 值）。
+const DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32: u32 = 3_497_143_766;
 
 /// 默认环 768 MiB：mkfs 按环长现算的单元区起点就是 50176（`INSTANCE_TABLE_SLOT`），树表第 0 版在 50178（`TREE_TABLE_GENESIS_SLOT`），
 /// 系统配置偏移 417 的 8 字节写 50176；mkfs 到第一个文件写出的两块盘与钉值那一版（钉值说明里最后一次换钉的那一版）逐字节相同（整盘 CRC-32C 钉值）。
@@ -1601,10 +1604,10 @@ fn the_rollback_takes_the_inode_number_watermark_from_the_current_version_in_mem
 
 /// 管理员回退算水位的树 ID 那一半（D23（journal 的角色与格式） 已定项 14「水位」、D8（核心索引结构） 已定项 8 ②「号永不重发」；
 /// C586（回退取树 ID 水位的那一半没有用例），攻方 `research/prompts/m2-closeout-code-r3-opus-output.md`「X1-e」）：
-/// 第一个文件（txg 3）十二棵树发 65536..65547、水位 65548，覆盖写一次（txg 4）照带 65548；环里别的根（mkfs 与暖机）都带 mkfs 种下的 65536。
+/// 第一个文件（txg 3）十三棵树发 65536..65549、水位 65549，覆盖写一次（txg 4）照带 65549；环里别的根（mkfs 与暖机）都带 mkfs 种下的 65536。
 /// 回退到 txg 3 时这两条根的槽判候选那两遍（各自第 1、2 次读）读得出，算水位那一遍（第 3 次，`readable_roots`，不重读）都瞬时读不出：
-/// 环里读得出的根都是 65536，只有内存里的现行那一版带 65548。回退那次发布的树 ID 水位仍是 65548——树表里 65536..65547 那几棵树都在水位之下（I-7.8 不红），
-/// 之后再发树 ID 从 65548 起、不重发 65536..65547。只取环里读得出的根时是 65536——`crates/mutations.tsv` 里 C586 那一行证这条断言红。
+/// 环里读得出的根都是 65536，只有内存里的现行那一版带 65549。回退那次发布的树 ID 水位仍是 65549——树表里 65536..65549 那几棵树都在水位之下（I-7.8 不红），
+/// 之后再发树 ID 从 65549 起、不重发 65536..65549。只取环里读得出的根时是 65536——`crates/mutations.tsv` 里 C586 那一行证这条断言红。
 #[test]
 fn the_rollback_takes_the_tree_identifier_watermark_from_the_current_version_in_memory_when_the_watermark_read_misses_every_root_carrying_it(
 ) {
@@ -1626,7 +1629,7 @@ fn the_rollback_takes_the_tree_identifier_watermark_from_the_current_version_in_
         .expect("覆盖写")
     };
     assert_eq!(overwritten.root.checkpoint_txg, CheckpointTxg(4));
-    assert_eq!(overwritten.root.tree_identifier_watermark, 65548);
+    assert_eq!(overwritten.root.tree_identifier_watermark, 65549);
     let roots_in_the_ring = readable_roots_with_ring_slots(
         &pool.devices,
         &pool.parameters.region_devices,
@@ -1636,22 +1639,22 @@ fn the_rollback_takes_the_tree_identifier_watermark_from_the_current_version_in_
     let slots_of_the_roots_carrying_the_first_file_version_watermark: Vec<RootRingSlot> =
         roots_in_the_ring
             .iter()
-            .filter(|(_, root)| root.tree_identifier_watermark == 65548)
+            .filter(|(_, root)| root.tree_identifier_watermark == 65549)
             .map(|(ring_slot, _)| *ring_slot)
             .collect();
     assert_eq!(
         roots_in_the_ring
             .iter()
-            .filter(|(_, root)| root.tree_identifier_watermark == 65548)
+            .filter(|(_, root)| root.tree_identifier_watermark == 65549)
             .map(|(_, root)| root.checkpoint_txg)
             .collect::<BTreeSet<_>>(),
         BTreeSet::from([CheckpointTxg(3), CheckpointTxg(4)]),
-        "带 65548 的只有第一个文件与覆盖写那两条根"
+        "带 65549 的只有第一个文件与覆盖写那两条根"
     );
     assert!(
         roots_in_the_ring
             .iter()
-            .all(|(_, root)| root.tree_identifier_watermark == 65548
+            .all(|(_, root)| root.tree_identifier_watermark == 65549
                 || root.tree_identifier_watermark == 65536),
         "环里别的根都带 mkfs 种下的 65536"
     );
@@ -1689,8 +1692,8 @@ fn the_rollback_takes_the_tree_identifier_watermark_from_the_current_version_in_
         "回退那次发布"
     );
     assert_eq!(
-        current.root.tree_identifier_watermark, 65548,
-        "回退那次发布的树 ID 水位 = max(现行那一版内存里的 65548, 环里读得出的根的 65536)"
+        current.root.tree_identifier_watermark, 65549,
+        "回退那次发布的树 ID 水位 = max(现行那一版内存里的 65549, 环里读得出的根的 65536)"
     );
     assert_eq!(
         checker_violations(&memory_pool_of(&pool.devices, pool.device_bytes)),
diff --git a/crates/singlefs-harness/tests/unlink_crash_image_remounted_twice_continues_to_identical_disks.rs b/crates/singlefs-harness/tests/unlink_crash_image_remounted_twice_continues_to_identical_disks.rs
index 942d626..da9661f 100644
--- a/crates/singlefs-harness/tests/unlink_crash_image_remounted_twice_continues_to_identical_disks.rs
+++ b/crates/singlefs-harness/tests/unlink_crash_image_remounted_twice_continues_to_identical_disks.rs
@@ -33,6 +33,8 @@ fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
         batch_size: batches_of_one_data_unit(),
         tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
         lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container_capacity:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     }
 }
diff --git a/crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs b/crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs
index e32e82e..6fb3c29 100644
--- a/crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs
+++ b/crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs
@@ -55,6 +55,8 @@ fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
         batch_size: batches_of_one_data_unit(),
         tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
         lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container_capacity:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     }
 }
@@ -145,11 +147,13 @@ fn violations(image: &MemoryPool) -> Vec<String> {
         .collect()
 }
 
+/// 这一版那一统计量的现行值：代段最大的那一行（inode 号水位那一统计量还有台阶行，D8（核心索引结构） 已定项 6「读现行值」）。
 fn accounting_value(version: &TransactionOutput, statistic: u16) -> u64 {
     version
         .accounting_entries
         .iter()
-        .find(|entry| entry.statistic == statistic)
+        .filter(|entry| entry.statistic == statistic)
+        .max_by_key(|entry| entry.generation)
         .expect("池级三行每一版都写")
         .value
 }
@@ -314,8 +318,10 @@ fn a_crash_halfway_through_the_writes_of_the_first_batch_continues_on_remount_to
 }
 
 /// ④：删除前、意图、两批、最后一批每一版记账行里的标签 4 与 6，等于从这一版镜像现算的值：
-/// 标签 4 恒 0；标签 6 = 2 × ((2 + H + 1) × 16384 + 41 × n_live)，n_live 数重建出来的 inode 树里 nlink ≥ 1 的记录，
-/// H 从墓碑树根单元的码 2 头现读（零根时 0）。删除前 n_live = 1（98386），意图起 n_live = 0（98304）。
+/// 标签 4 恒 0；标签 6 = 2 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)，n_live 数重建出来的 inode 树里
+/// nlink ≥ 1 的记录、n_记录 数它的全部记录（含 nlink = 0 的，判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条），
+/// H_墓碑、H_退役 从墓碑树、退役树根单元的码 2 头现读（零根时 0）。删除前 n_live = n_记录 = 1（196760），意图起到最后一批之前 n_live = 0、
+/// n_记录 = 1（196678），最后一批之后 n_记录 = 0（196608）。
 #[test]
 fn tags_four_and_six_of_every_deletion_version_equal_the_values_recomputed_from_its_image() {
     let mut pool = pool_with_a_three_unit_file("idempotence-tags-four-and-six");
@@ -340,13 +346,32 @@ fn tags_four_and_six_of_every_deletion_version_equal_the_values_recomputed_from_
                     .count(),
             )
             .expect("记录数");
-            let tombstone_tree_internal_levels = rebuilt
-                .units
-                .iter()
-                .find(|unit| unit.identity == TransactionUnit::TombstoneRoot)
-                .map_or(0, |root| {
-                    u64::from(parse_index_node(&root.bytes).expect("墓碑树根解得开").level) - 1
-                });
+            // n_记录 数 inode 树里全部记录、含 nlink = 0 的（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条）。
+            let records = u64::try_from(
+                rebuilt
+                    .inode_leaf_containers
+                    .iter()
+                    .map(|container| container.contents.records.len())
+                    .sum::<usize>(),
+            )
+            .expect("记录数");
+            let internal_levels_below_the_root = |root_role: TransactionUnit, what: &str| {
+                rebuilt
+                    .units
+                    .iter()
+                    .find(|unit| unit.identity == root_role)
+                    .map_or(0, |root| {
+                        u64::from(
+                            parse_index_node(&root.bytes)
+                                .unwrap_or_else(|_| panic!("{what}根解得开"))
+                                .level,
+                        ) - 1
+                    })
+            };
+            let tombstone_tree_internal_levels =
+                internal_levels_below_the_root(TransactionUnit::TombstoneRoot, "墓碑树");
+            let retirement_tree_internal_levels =
+                internal_levels_below_the_root(TransactionUnit::RetirementRoot, "退役树");
             (
                 (
                     accounting_value(version, STATISTIC_PENDING_DELETE_BYTES),
@@ -354,7 +379,15 @@ fn tags_four_and_six_of_every_deletion_version_equal_the_values_recomputed_from_
                 ),
                 (
                     0,
-                    2 * ((2 + tombstone_tree_internal_levels + 1) * 16384 + 41 * live_objects),
+                    2 * ((2
+                        + tombstone_tree_internal_levels
+                        + 1
+                        + 2
+                        + retirement_tree_internal_levels
+                        + 1)
+                        * 16384
+                        + 41 * live_objects
+                        + 35 * records),
                 ),
             )
         })
@@ -375,8 +408,9 @@ fn tags_four_and_six_of_every_deletion_version_equal_the_values_recomputed_from_
             .iter()
             .map(|((_, recorded_tag_six), _)| *recorded_tag_six)
             .collect::<Vec<u64>>(),
-        vec![98_386, 98_304, 98_304, 98_304, 98_304],
-        "删除前一个活对象；从意图那次起第一个文件 nlink = 0、不再算活对象"
+        vec![196_760, 196_678, 196_678, 196_678, 196_608],
+        "删除前一个活对象、一条记录；从意图那次起第一个文件 nlink = 0、不再算活对象，记录还在（n_记录 含 nlink = 0 的）；最后一批记录删掉。\
+         墓碑 3 槽与退役 3 槽常驻：2 × 6 × 16384 = 196608，活对象每个 2 × 41、记录每条 2 × 35"
     );
 }
 
@@ -438,8 +472,10 @@ fn rolling_back_to_the_emptied_version_is_accepted_and_publishes_a_version_with_
 }
 
 /// 回退那次发布的标签 6 按新根现算（D23（journal 的角色与格式） 已定项 14「准入与记账」那一行）：新根的 inode 树与 `deleted_inodes`
-/// 取 R_old 的，n_live = R_old 那一版 inode 树的记录数减 `deleted_inodes` 的 key 数。删完唯一的文件（标签 6 = 98304，n_live = 0）之后
-/// 回退到意图之前那一版（第一个文件活着）：新根的标签 6 = 2 × (49152 + 41 × 1) = 98386，被带回来的对象的墓碑份额照抬。
+/// 取 R_old 的，n_live = R_old 那一版 inode 树的记录数减 `deleted_inodes` 的 key 数，n_记录 = R_old 那一版 inode 树的全部记录数。
+/// 删完唯一的文件（标签 6 = 2 × (6 × 16384) = 196608，n_live = n_记录 = 0；墓碑 3 槽与退役 3 槽常驻，判决
+/// `research/prompts/m3-retire-r1-main-verification.md` 第三节第 6 条）之后回退到意图之前那一版（第一个文件活着）：
+/// 新根的标签 6 = 2 × (98304 + 41 × 1 + 35 × 1) = 196760，被带回来的对象的墓碑与退役份额照抬。
 #[test]
 fn rolling_back_to_the_version_before_the_intent_recomputes_tag_six_from_the_rollback_targets_inode_tree(
 ) {
@@ -469,7 +505,7 @@ fn rolling_back_to_the_version_before_the_intent_recomputes_tag_six_from_the_rol
             tag_six_before_the_rollback,
             accounting_value(&pool.output, STATISTIC_COMMITTED_RESERVATION_BYTES),
         ),
-        (98_304, 98_386),
-        "删完时没有活对象；回退新根按 R_old 的 inode 树现算，第一个文件又是一个活对象"
+        (196_608, 196_760),
+        "删完时没有活对象也没有记录；回退新根按 R_old 的 inode 树现算，第一个文件又是一个活对象、一条记录"
     );
 }
diff --git a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
index e814472..8335648 100644
--- a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
+++ b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
@@ -55,6 +55,8 @@ fn deletion_switches_with_batches_of_one() -> DeletionSwitches {
         batch_size: batches_of_one_data_unit(),
         tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
         lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
+        retirement_container_capacity:
+            singlefs_core::retirement_tree::RetirementContainerCapacity::FromTheContainerFormat,
         container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
     }
 }
diff --git a/crates/singlefs-harness/tests/unlink_single_file_publishes.rs b/crates/singlefs-harness/tests/unlink_single_file_publishes.rs
index 5c6ec29..75b7742 100644
--- a/crates/singlefs-harness/tests/unlink_single_file_publishes.rs
+++ b/crates/singlefs-harness/tests/unlink_single_file_publishes.rs
@@ -216,9 +216,21 @@ fn the_final_batch_removes_the_record_the_queue_key_and_every_remaining_unit_in_
     assert_eq!(
         tags_of(&final_batch.rewritten),
         [
-            "t2", "t4", "t5@0.0.61", "t5@0.1.61", "t5@1.0.0", "t5@1.1.0", "t5", "t6", "t7", "t8"
+            "t2",
+            "t4",
+            "t5@0.0.61",
+            "t5@0.1.61",
+            "t5@1.0.0",
+            "t5@1.1.0",
+            "t5",
+            "t6",
+            "tr@0",
+            "tr",
+            "t7",
+            "t8"
         ],
-        "最后一批重写 extent 零条目根、inode 零条目根与固定点单元；deleted_inodes 删空取根指针置零、不写它的节点"
+        "最后一批重写 extent 零条目根、inode 零条目根与固定点单元；deleted_inodes 删空取根指针置零、不写它的节点；唯一那片叶容器身份消失，\
+         退役树开第一片容器、写根（判决 m3-retire-r1 第三节第 3 条）"
     );
     assert_eq!(final_batch.inode_record, None, "记录删掉");
     assert!(final_batch.deleted_inodes.is_empty(), "key 删掉");
diff --git a/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs b/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs
index 36cf9fe..c7a2ede 100644
--- a/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs
+++ b/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs
@@ -38,8 +38,8 @@ const DEVICES_OF_THE_TEST_POOL: u64 = 2;
 
 /// 任一份文件里写调用最多的那次发布，至多比一个单元的文件里写调用最多的那次多几个写调用：从代码算，不随文件的单元数长。
 /// 一次删除类发布每块盘新占的槽至多 W 个（`admission::slots_one_deletion_publish_newly_takes_on_one_device_at_most`，
-/// D28（挂载期承诺量） 已定项 1「环窗口」那一条的 W = 24 + Σ_ℓ min(N_ℓ, 2·(K + 26))，准入按它给环窗口扣空间；N_ℓ 只随盘的单元区定，
-/// K 是批大小）。这里不另抄那条式子，按这个池的分配器经 `admission::ring_window_on_one_device` 现算环窗口（= (根环槽数 + 1) × W 个槽）再除回 W。
+/// D28（挂载期承诺量） 已定项 1「环窗口」那一条的 W = 24 + (6 + H_退役) + Σ_ℓ min(N_ℓ, 2·(K + 26))，准入按它给环窗口扣空间；
+/// N_ℓ 只随盘的单元区定，K 是批大小，H_退役 是退役树的内部层数——这几条用例的池里退役树至多只有根，取 0）。这里不另抄那条式子，按这个池的分配器经 `admission::ring_window_on_one_device` 现算环窗口（= (根环槽数 + 1) × W 个槽）再除回 W。
 /// 发布是写时复制，这次写出的每个单元都是新占的落点、至少一槽，每块盘一个写调用：单元的写调用至多「盘数 × W」；
 /// 点名项一条 journal 记录装 `JOURNAL_NAMED_ENTRIES_PER_RECORD` 项，记录至多多 ⌈W ÷ 64⌉ 条、每块盘一份。
 /// 一个单元的文件那次至少也写一条记录与一个根槽，所以差至多「盘数 × (W + ⌈W ÷ 64⌉)」。
@@ -65,6 +65,7 @@ fn write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file() ->
         &pool.allocator,
         slots_per_region,
         EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
+        0,
     )
     .0 / SLOT_BYTES;
     let roots_in_the_ring_plus_one = ROOT_RING_REGIONS * slots_per_region.count() + 1;
@@ -257,13 +258,14 @@ fn unlink_of_files_of_one_to_four_batches_releases_exactly_one_batch_or_the_rest
             write_calls_of_every_publish(&three_batches_and_one_unit),
         ],
         [
-            vec![31, 25],
             vec![31, 29],
-            vec![31, 31, 29],
-            vec![33, 37, 35, 33],
-            vec![33, 39, 43, 39, 29],
+            vec![31, 33],
+            vec![31, 31, 33],
+            vec![33, 37, 35, 37],
+            vec![33, 39, 43, 39, 33],
         ],
-        "1、144、145、432、433 个数据单元的文件，每一次发布（意图、各批）的写调用"
+        "1、144、145、432、433 个数据单元的文件，每一次发布（意图、各批）的写调用：最后一批唯一那片叶容器身份消失，\
+         退役树多写容器与根两个单元、两块盘各一份，比第 49 批多 4 次"
     );
 }
 
diff --git a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
index b7700ae..bd68afb 100644
--- a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
+++ b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
@@ -556,7 +556,7 @@ fn region_device(txg: u64) -> DeviceIdentity {
 }
 
 /// 最新的根下面还没有记账树、inode 树与分配记录树时（mkfs 之后、零单元发布之后）checker 报不适用的那 45 条；
-/// 其余 39 条要真被评估过且成立（条数按今天的 `IMPLEMENTED_INVARIANTS` 84 条数）。映射 key 与单元头相符（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`）报不适用是因为中央映射树的根指针全零，
+/// 其余 39 条要真被评估过且成立（条数按今天的 `IMPLEMENTED_INVARIANTS` 85 条数）。映射 key 与单元头相符（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`）报不适用是因为中央映射树的根指针全零，
 /// 一条映射条目都没有。I-3.11（已分配减 defer 等于最新根走读） 与 I-3.1（已分配统计对得上） 同一个理由：最新根下面没有记账树。I-9.14（树表条目的诞生 txg 跨根不变） 在这里报不适用是因为 mkfs 种的第 0 版树表是空的：
 /// 一棵树的条目都没有，跨根比不出来（有了文件版本之后也只有一条根有条目，见 `NOT_APPLICABLE_WITH_ONE_FILE_VERSION`）；
 /// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 同一个理由，比的是根环里全部读得出的根（不按实例表判有效）：根环里每条根的树表都是
@@ -580,8 +580,10 @@ fn region_device(txg: u64) -> DeviceIdentity {
 /// I-4.6（中间态其余不变量成立） 没有处于删除中的号；I-4.4（意图只描述目标态） 第一版恒报不适用（下面每一张表都有它）。
 /// 墓碑与谱系那九条（里程碑三·3，第 22 批）五张没有文件的表都有：I-9.17–I-9.23 要墓碑树、谱系树（每一版树表都是 0 条，有了文件版本也是零根，
 /// 下面两张有文件的表照样有这七条），I-9.24（删除队列至多一条 key） 要 `deleted_inodes` 条目、I-9.29（同号只许一个出生代） 要 inode 记录
-/// （第一个文件版本之后两条都真被评估过且成立）。
-const NOT_APPLICABLE_WITHOUT_FILE: [&str; 45] = [
+/// （第一个文件版本之后两条都真被评估过且成立）。I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上） 要最新根下面的记账树
+/// （第一个文件版本之后真被评估过且成立：那一版自己写的那一行就是第一级台阶）。退役树那五条（I-9.31–I-9.35，第 50 批）同样没有对象：
+/// 每一版树表都是 0 条，没有退役树、没有 inode 树。
+const NOT_APPLICABLE_WITHOUT_FILE: [&str; 51] = [
     "I-9.27",
     "I-5.5",
     "I-1.10",
@@ -627,6 +629,12 @@ const NOT_APPLICABLE_WITHOUT_FILE: [&str; 45] = [
     "I-9.23",
     "I-9.24",
     "I-9.29",
+    "I-9.30",
+    "I-9.31",
+    "I-9.32",
+    "I-9.33",
+    "I-9.34",
+    "I-9.35",
 ];
 
 /// mkfs 刚写完那一份比 `NOT_APPLICABLE_WITHOUT_FILE` 多报四条不适用：journal 环整段是 0，一条自证过的记录都没有 ⇒ I-8.6（反向链算法）、
@@ -635,7 +643,7 @@ const NOT_APPLICABLE_WITHOUT_FILE: [&str; 45] = [
 /// I-3.5（引用区间的精确性） 两张表都有，这一格的理由是回退候选集只剩 mkfs 的第 0 代根一版（环里没有记录、根环里只有它），
 /// 凑不出相邻的一对。
 /// 删除线那五条与墓碑与谱系那九条的理由同 `NOT_APPLICABLE_WITHOUT_FILE`。
-const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 49] = [
+const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 55] = [
     "I-9.27",
     "I-5.5",
     "I-1.10",
@@ -685,6 +693,12 @@ const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 49] = [
     "I-9.23",
     "I-9.24",
     "I-9.29",
+    "I-9.30",
+    "I-9.31",
+    "I-9.32",
+    "I-9.33",
+    "I-9.34",
+    "I-9.35",
 ];
 
 /// 写完第一个文件版本之后仍报不适用的那六条（I-4.1 的理由见 `NOT_APPLICABLE_WITHOUT_FILE`）：
@@ -699,17 +713,20 @@ const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 49] = [
 /// B 之后再接一事务两条记录的阳性对照与坏镜像）。
 /// I-4.7（删除中 inode 有账） 与 I-4.6（中间态其余不变量成立）——没有删除，没有处于删除中的号、没有 nlink = 0 的记录
 /// （阳性对照与坏镜像在 `pool_checker_judges_the_deletion_state_of_every_walked_version.rs`）；I-4.4（意图只描述目标态） 第一版恒报不适用；
-/// I-9.17–I-9.23——没有删除、没有回退，墓碑树与谱系树都是零根。
-const NOT_APPLICABLE_WITH_ONE_FILE_VERSION: [&str; 16] = [
+/// I-9.17–I-9.23——没有删除、没有回退，墓碑树与谱系树都是零根；I-9.31–I-9.34——没有叶容器身份消失，退役树零根
+/// （I-9.35 在第一个文件版本那一版上真被评估过且成立：父版树表 0 条、叶身份集是空集）。
+const NOT_APPLICABLE_WITH_ONE_FILE_VERSION: [&str; 20] = [
     "I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-4.1", "I-9.28", "I-4.7", "I-4.6", "I-4.4", "I-9.17",
-    "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23",
+    "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23", "I-9.31", "I-9.32", "I-9.33",
+    "I-9.34",
 ];
 
 /// 只有实例 1 的池上第一个文件版本之后：比 `NOT_APPLICABLE_WITH_ONE_FILE_VERSION` 多 I-3.12（实例表行 T 低于引用它的根的 txg）——
 /// 实例表一行都没有（写过行的那一份上它真被评估过且成立）。
-const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 17] = [
+const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 21] = [
     "I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-3.12", "I-4.1", "I-9.28", "I-4.7", "I-4.6", "I-4.4",
-    "I-9.17", "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23",
+    "I-9.17", "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23", "I-9.31", "I-9.32",
+    "I-9.33", "I-9.34",
 ];
 
 /// 树表 0 条的一版上写行那次发布之后：比 `NOT_APPLICABLE_WITHOUT_FILE` 少七条——I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）、
@@ -725,7 +742,7 @@ const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 17] = [
 /// I-3.5（引用区间的精确性） 也不在这张表里：写行那次发布（实例 2、txg 3）把实例表 COW 到新落点、写了自己那棵分配记录树，
 /// 这几个单元前一版（实例 1 的暖机根，txg 2）不引用、诞生代号 3 大于 2，出现规则真判到了它们且成立。
 /// 删除线那五条与墓碑与谱系那九条的理由同 `NOT_APPLICABLE_WITHOUT_FILE`。
-const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 38] = [
+const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 44] = [
     "I-9.27",
     "I-5.5",
     "I-3.1",
@@ -764,6 +781,12 @@ const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 38] = [
     "I-9.23",
     "I-9.24",
     "I-9.29",
+    "I-9.30",
+    "I-9.31",
+    "I-9.32",
+    "I-9.33",
+    "I-9.34",
+    "I-9.35",
 ];
 
 /// 这一步的镜像上 checker 的每一条判决逐条核：`not_applicable` 里的报不适用，其余每一条都真被评估过且成立（一条违例都没有）。
@@ -1148,8 +1171,8 @@ fn plant_a_root_without_its_allocation_record_tree(
 /// 验收：取号 1、不写行；写行那次发布 (1, 1)、jsn 1、事务号 0、反向链 0、零单元、落盘 1；暖机一次 (1, 2)、jsn 2、反向链接 jsn 1、落盘 0；
 /// 分配器只有 mkfs 的两个单元（每盘 3 槽）；第一个文件版本 (1, 3)、jsn 3、事务号 1，数据单元落 50180、换下 mkfs 那片树表；
 /// mkfs 之后的录制流与新池新建文件那条逐字节相同、写出的那一版也相同；冷启动读回文件；checker 一条违例都没有（条数按今天的
-/// `IMPLEMENTED_INVARIANTS` 84 条数）——mkfs 之后 35 条成立（环是空的，I-8.6、I-8.9、I-8.10、I-8.12 也报不适用）、挂载之后 39 条成立，
-/// 没有记账树 / inode 树 / 分配记录树 / 映射条目的那几条报不适用，第一个文件版本之后 67 条成立、I-7.9、I-9.14、I-9.28、I-8.7、I-8.8、
+/// `IMPLEMENTED_INVARIANTS` 85 条数）——mkfs 之后 35 条成立（环是空的，I-8.6、I-8.9、I-8.10、I-8.12 也报不适用）、挂载之后 39 条成立，
+/// 没有记账树 / inode 树 / 分配记录树 / 映射条目的那几条报不适用，第一个文件版本之后 68 条成立、I-7.9、I-9.14、I-9.28、I-8.7、I-8.8、
 /// I-3.12、I-4.1、I-4.7、I-4.6、I-4.4 与墓碑树、谱系树那七条（I-9.17–I-9.23）仍报不适用（树表条目只出现在这一条根的树表里；同一实例只有一条非 0 事务号的记录；没有哪个事务号落了两条记录、
 /// 每条都带提交标记；只有实例 1，实例表一行都没有；最新根之上没有记录；没有删除；I-4.4 第一版恒报不适用）。
 #[test]
@@ -1531,14 +1554,14 @@ fn a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_i
 
 /// D8（核心索引结构） 已定项 8 ②「号永不重发」在崩溃恢复那一形上（原先由回退到暖机根之后再发第一个文件版本那几条用例护，
 /// 回退改成挂着时的向前发布之后那一形没有了，D23（journal 的角色与格式） 已定项 14）：只做过 mkfs 的池可写挂载（实例 1，暖机），
-/// 再挂一次（实例 2，写行 txg 3、暖机 txg 4）发第一个文件版本（txg 5，十二棵树 65536..65547），崩在 txg 5 的根槽 FUA 之后、系统配置轮换之前
+/// 再挂一次（实例 2，写行 txg 3、暖机 txg 4）发第一个文件版本（txg 5，十三棵树 65536..65548），崩在 txg 5 的根槽 FUA 之后、系统配置轮换之前
 /// （录制流切到那一写为止：系统配置只见证到 txg 4）。可写挂载一次，txg 5 的根槽与它那条记录点名的一个单元（数据单元盘 0 那一份）读不出——
 /// 择根落到实例 2 的暖机根 (2, 4)（树表 0 条、根上水位 65536），txg 5 那条记录施加前验点名单元验不过、不施加；系统配置见证到的就是 (2, 4)
 /// 那次，C554 乙不重读、照常挂载（见证过的根暂时读不出会被拒，`a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs`）。
-/// 新实例 3 的水位按根环与环里记录取 max = 65548。实例 3 接着发第一个文件版本：
-/// - 十二棵树从 65548 发 65548..65559、新水位 65560，不从那一版根上的 65536 重发（照 mkfs 水位发就与盘上实例 2 的节点撞号）；
-/// - 「树建没建」按树表条数判（0 条），不按水位是不是 65536（水位 65548、树表 0 条照样发得出第一个文件版本）；
-/// - 只读挂载沿新根读回这一版的内容：中央映射树是 65552，按根记录里映射根指针的出生树认，不写死 65540。
+/// 新实例 3 的水位按根环与环里记录取 max = 65549。实例 3 接着发第一个文件版本：
+/// - 十三棵树从 65549 发 65549..65561、新水位 65562，不从那一版根上的 65536 重发（照 mkfs 水位发就与盘上实例 2 的节点撞号）；
+/// - 「树建没建」按树表条数判（0 条），不按水位是不是 65536（水位 65549、树表 0 条照样发得出第一个文件版本）；
+/// - 只读挂载沿新根读回这一版的内容：中央映射树是 65553，按根记录里映射根指针的出生树认，不写死 65540。
 #[test]
 fn the_first_file_version_after_a_recovery_dropped_an_earlier_one_issues_its_trees_above_the_ring_watermark_and_reads_back(
 ) {
@@ -1571,8 +1594,8 @@ fn the_first_file_version_after_a_recovery_dropped_an_earlier_one_issues_its_tre
     };
     assert_eq!(
         dropped_first_file.tree_identifiers.highest().0,
-        65547,
-        "实例 2 从 mkfs 的水位 65536 发 65536..65547"
+        65548,
+        "实例 2 从 mkfs 的水位 65536 发 65536..65548"
     );
     assert_eq!(
         dropped_first_file.root.checkpoint_txg,
@@ -1638,8 +1661,8 @@ fn the_first_file_version_after_a_recovery_dropped_an_earlier_one_issues_its_tre
         panic!("落到实例 2 的暖机根：现行那一版树表 0 条")
     };
     assert_eq!(
-        version_of_the_third_instance.root.tree_identifier_watermark, 65548,
-        "实例 3 那一版带根环与环里记录的水位 max：实例 2 的记录带的 65548"
+        version_of_the_third_instance.root.tree_identifier_watermark, 65549,
+        "实例 3 那一版带根环与环里记录的水位 max：实例 2 的记录带的 65549"
     );
     let mut allocator_of_the_third_instance = third_instance.allocator.clone();
     let content = (0..3300usize)
@@ -1659,7 +1682,7 @@ fn the_first_file_version_after_a_recovery_dropped_an_earlier_one_issues_its_tre
             third_instance.output.instance,
             &version_of_the_third_instance.record_bytes,
         )
-        .expect("水位 65548、树表 0 条：照样发得出第一个文件版本")
+        .expect("水位 65549、树表 0 条：照样发得出第一个文件版本")
     };
     assert_eq!(
         (
@@ -1677,8 +1700,8 @@ fn the_first_file_version_after_a_recovery_dropped_an_earlier_one_issues_its_tre
                 .birth_tree
                 .0
         ),
-        (65559, 65560, 65552),
-        "实例 3 从 65548 发 65548..65559、新水位 65560，中央映射树是 65552"
+        (65561, 65562, 65553),
+        "实例 3 从 65549 发 65549..65561、新水位 65562，中央映射树是 65553"
     );
     let image = memory_pool_of_sparse_devices(&reopened);
     let mounted = singlefs_core::mounted_read::mount_read_only(&image).expect("只读挂载");
diff --git a/crates/singlefs-harness/tests/write_accounting_per_structure.rs b/crates/singlefs-harness/tests/write_accounting_per_structure.rs
index 7d41a53..76f05eb 100644
--- a/crates/singlefs-harness/tests/write_accounting_per_structure.rs
+++ b/crates/singlefs-harness/tests/write_accounting_per_structure.rs
@@ -140,6 +140,7 @@ fn expected_by_kind(
         WrittenStructureKind::DeletedInodesTreeNode,
         WrittenStructureKind::TombstoneTreeNode,
         WrittenStructureKind::LineageTreeNode,
+        WrittenStructureKind::RetirementTreeNode,
         WrittenStructureKind::CentralMappingTreeNode,
         WrittenStructureKind::TreeTableUnit,
         WrittenStructureKind::InstanceTableUnit,
@@ -163,6 +164,7 @@ fn expected_by_kind(
                         | WrittenStructureKind::DeletedInodesTreeNode
                         | WrittenStructureKind::TombstoneTreeNode
                         | WrittenStructureKind::LineageTreeNode
+                        | WrittenStructureKind::RetirementTreeNode
                         | WrittenStructureKind::CentralMappingTreeNode
                         | WrittenStructureKind::TreeTableUnit
                         | WrittenStructureKind::InstanceTableUnit
@@ -513,6 +515,10 @@ fn failed_midway_publish_by_kind() -> Vec<(WrittenStructureKind, WriteCallsAndBy
             WrittenStructureKind::LineageTreeNode,
             WriteCallsAndBytes::NONE,
         ),
+        (
+            WrittenStructureKind::RetirementTreeNode,
+            WriteCallsAndBytes::NONE,
+        ),
         (
             WrittenStructureKind::CentralMappingTreeNode,
             WriteCallsAndBytes::NONE,
`````

## mutations-append.tsv

`````text
第49批 W3 I-9.30 ④：谱系记录按「被台阶盖住」比（T_rb − 1 以前最大的台阶 ≥ H_cur），放过 H_cur 偏小与写 0	crates/singlefs-checker/src/walk.rs	largest_step_before_the_rollback == Some(watermark_rolled_back_from),	largest_step_before_the_rollback >= Some(watermark_rolled_back_from),	-p singlefs-harness --test checker_known_bad_images -- a_lineage_record_whose_watermark_rolled_back_from_is_	a_lineage_record_whose_watermark_rolled_back_from_is_below_the_last_step_reddens_only_the_watermark_step_invariant
第49批 W3 I-9.30 ④（H_cur 为 0）：谱系记录按「被台阶盖住」比，H_cur 写 0 也放过	crates/singlefs-checker/src/walk.rs	largest_step_before_the_rollback == Some(watermark_rolled_back_from),	largest_step_before_the_rollback >= Some(watermark_rolled_back_from),	-p singlefs-harness --test checker_known_bad_images -- a_lineage_record_whose_watermark_rolled_back_from_is_zero_reddens_only_the_watermark_step_invariant	a_lineage_record_whose_watermark_rolled_back_from_is_zero_reddens_only_the_watermark_step_invariant
第49批 W3 I-9.30 ④：比的台阶取到代段 ≤ T_rb（多算回退那一版自己写的那一行），台阶少一级时被它补上	crates/singlefs-checker/src/walk.rs	.filter(|(step_txg, _)| *step_txg < record.rollback_txg)	.filter(|(step_txg, _)| *step_txg <= record.rollback_txg)	-p singlefs-harness --test checker_known_bad_images -- an_inode_number_watermark_step_dropped_from_the_accounting_tree_reddens_only_the_watermark_step_invariant	an_inode_number_watermark_step_dropped_from_the_accounting_tree_reddens_only_the_watermark_step_invariant
第49批 W3 I-9.30 ①：水位不小于每一级台阶那一判恒成立	crates/singlefs-checker/src/walk.rs	judgements.judge(invariant, *step_watermark <= inode_number_watermark, || {	judgements.judge(invariant, true, || {	-p singlefs-harness --test checker_known_bad_images -- an_inode_number_watermark_step_raised_above_the_watermark_reddens_only_the_watermark_step_invariant	an_inode_number_watermark_step_raised_above_the_watermark_reddens_only_the_watermark_step_invariant
第49批 W3 I-3.13：inode 号水位的台阶行不跳过、照判代段等于这一版的 txg（D5 已定项 2 射程的例外撤掉）	crates/singlefs-checker/src/walk.rs	                    if is_an_inode_number_watermark_step {	                    if false && is_an_inode_number_watermark_step {	-p singlefs-harness --test checker_known_bad_images -- a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_and_created_again_holds_the_watermark_step_and_generation_invariants	a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_and_created_again_holds_the_watermark_step_and_generation_invariants
第49批 W3 I-9.30 ②：记录与谱系上的墓碑晚于台阶出生那一判恒不找到	crates/singlefs-checker/src/walk.rs	.find(|(step_txg, step_watermark)| *step_watermark > number && object_birth > *step_txg)	.find(|(step_txg, step_watermark)| *step_watermark > number && object_birth > *step_txg && false)	-p singlefs-harness --test inode_number_watermark_pressed_back_reissues_a_number -- pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants	pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants
第49批 W3 Y1 Opt2：管理员回退取 inode 号水位时环里读得出的根全都算（被崩溃恢复抛弃的也算，回退跳到被抛弃那一版的水位）	crates/singlefs-core/src/mount.rs	        &ring_roots_not_abandoned_by_crash_recovery(&ring_roots, &current_table),	        &ring_roots,	-p singlefs-harness --test rollback_by_a_forward_publish -- a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root	a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root
第49批 W3 Y2 C2aL：发布不照抄上一版的 inode 号水位台阶行	crates/singlefs-core/src/transaction.rs	    entries.extend_from_slice(inode_number_watermark_steps_carried_from_the_previous_version);	    let _ = inode_number_watermark_steps_carried_from_the_previous_version;	-p singlefs-harness --test rollback_by_a_forward_publish -- inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback	inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback
第49批 W3 Y2 C2aL：每一版自己那一行都当台阶照抄（水位没变的发布也留一级）	crates/singlefs-core/src/transaction.rs	.is_none_or(|latest_step| latest_step.value != row.value)	.is_none_or(|latest_step| latest_step.value != row.value || true)	-p singlefs-harness --test rollback_by_a_forward_publish -- inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback	inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback
第49批 W3 Y2 C2aL：管理员回退那次的记账行（连台阶行）取 R_old 的，不从 cur 带	crates/singlefs-core/src/mount.rs	        accounting_entries: current.accounting_entries.clone(),	        accounting_entries: target_version.accounting_entries.clone(),	-p singlefs-harness --test rollback_by_a_forward_publish -- inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback	inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback
第49批 W3 恢复：记账行数 3 + 6 × 盘数把 inode 号水位的台阶行也算进去（有台阶的版本从盘上重建不出）	crates/singlefs-core/src/recovery.rs	entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg	entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg && false	-p singlefs-harness --test rollback_by_a_forward_publish -- inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback	inode_number_watermark_steps_are_left_by_publishes_creating_inodes_and_carried_by_the_others_and_by_a_rollback
第49批 W3 谱系记录 H_cur：取回退目标 R_old 的水位，不取 cur 的	crates/singlefs-core/src/mount.rs	inode_number_watermark_of_the_version_rolled_back_from: current.inode_number_watermark(),	inode_number_watermark_of_the_version_rolled_back_from: target_version.inode_number_watermark(),	-p singlefs-harness --test rollback_lineage_record_and_its_checkpoint_cost_term -- a_rollback_records_the_inode_number_watermark_of_the_version_it_rolls_back_from_not_of_its_target	a_rollback_records_the_inode_number_watermark_of_the_version_it_rolls_back_from_not_of_its_target
第49批 W3 谱系记录 24 字节：H_cur 那一段写成 0	crates/singlefs-core/src/records.rs	        writer.put_u64(self.inode_number_watermark_of_the_version_rolled_back_from);	        writer.put_u64(0);	-p singlefs-core --lib -- lineage_tree::tests::a_lineage_record_is_twenty_four_bytes_rollback_txg_then_target_txg_then_the_watermark_rolled_back_from	a_lineage_record_is_twenty_four_bytes_rollback_txg_then_target_txg_then_the_watermark_rolled_back_from
第49批 W3 格式：谱系容器容量照旧写 2039（记录宽 24 之后应是 ⌊(32768 − 136) ÷ 24⌋ = 1359）	crates/singlefs-format/src/lib.rs	pub const LINEAGE_CONTAINER_RECORDS: u64 = 1359;	pub const LINEAGE_CONTAINER_RECORDS: u64 = 2039;	-p singlefs-format --lib -- tests::deletion_widths_equal_the_field_sums_and_the_container_capacity_they_stand_for	deletion_widths_equal_the_field_sums_and_the_container_capacity_they_stand_for
第49批 W3 GPU 判器：扫描内核 I-9.30 ④ 不判（谱系记录的 H_cur 改错放过）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (!has_a_step_before_the_rollback || !eq64(largest_step_before_the_rollback, watermark_rolled_back_from)) {	        if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from	the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from
第49批 W3 GPU 判器：扫描内核读谱系记录的 H_cur 读成 T_old 那一段	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	const DELETION_LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET: u32 = 16u;	const DELETION_LINEAGE_RECORD_INODE_NUMBER_WATERMARK_OF_THE_VERSION_ROLLED_BACK_FROM_OFFSET: u32 = 8u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from	the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from
第49批 W3 GPU 判器：走读内核 I-3.13 不跳过 inode 号水位的台阶行	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                if (is_an_inode_number_watermark_step) {	                if (false && is_an_inode_number_watermark_step) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from	the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from
第49批 W3 GPU 判器：恢复内核的记账条目数把 inode 号水位的台阶行也算进 3 + 6 × 盘数	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	        if (!is_an_inode_number_watermark_step) {	        if (true) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from	the_gpu_judge_agrees_with_the_cpu_when_the_lineage_record_of_a_rollback_publish_carries_another_watermark_rolled_back_from
第49批 W3 补做 ckpt_cost：记账树那一项退回拿上一版的节点数（台阶行跨过一片叶那次少算）	crates/singlefs-core/src/admission.rs	            let accounting_tree_nodes = accounting_tree_nodes_after_the_publish_built_on(\n                file_version,\n                node_capacities.of_tree(MultiLevelCodeTwoTree::Accounting),\n            );	            let accounting_tree_nodes = accounting_tree_nodes_of_this_version;	-p singlefs-harness --test accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf -- the_checkpoint_cost_counts	the_checkpoint_cost_counts_the_accounting_nodes_after_the_publish_whose_watermark_step_crosses_one_leaf
第49批 W3 补做 ckpt_cost：记账树那一项规划下一次发布的行时不照抄台阶行（只取这一版自己那一组）	crates/singlefs-core/src/admission.rs	        .chain(version_to_build_on.inode_number_watermark_steps())	        .chain(Vec::<AccountingEntry>::new())	-p singlefs-harness --test accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf -- the_checkpoint_cost_counts	the_checkpoint_cost_counts_the_accounting_nodes_after_the_publish_whose_watermark_step_crosses_one_leaf
第49批 W3 补做 模型：数记账行不算 inode 号水位台阶行（退回 3 + 6 × 盘数）	crates/singlefs-harness/src/model.rs	            + previous.inode_number_watermark_steps	            + 0	-p singlefs-harness --test accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf -- the_model_and_the_implementation	the_model_and_the_implementation_grow_the_accounting_tree_a_level_at_the_same_watermark_step
第49批 W3 补做 模型：数记账行把这次自己那一行也当照抄的台阶（多数一级，早一步判长层）	crates/singlefs-harness/src/model.rs	            + previous.inode_number_watermark_steps	            + previous.inode_number_watermark_steps\n            + 1	-p singlefs-harness --test accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf -- the_model_and_the_implementation	the_model_and_the_implementation_grow_the_accounting_tree_a_level_at_the_same_watermark_step
第49批 W3 补做 模型：建 inode 落在最左那片之外时不认水位变了（不留台阶）	crates/singlefs-harness/src/model.rs	            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => true,	            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => false,	-p singlefs-harness --test accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf -- the_model_and_the_implementation	the_model_and_the_implementation_grow_the_accounting_tree_a_level_at_the_same_watermark_step
第49批 W3 再补 模型：写记账树的发布不按这次的行数带记账树节点数的上界（行数当 0，台阶跨叶那次仍一个节点）	crates/singlefs-harness/src/model.rs	                        previous.accounting_tree_nodes_upper_bound.max(1),\n                        accounting_rows,	                        previous.accounting_tree_nodes_upper_bound.max(1),\n                        0,	-p singlefs-harness --test accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf -- the_model_and_the_implementation	the_model_and_the_implementation_grow_the_accounting_tree_a_level_at_the_same_watermark_step
第49批 W3 再补 模型：记账树切叶次数的上界不减一（不到 478 行也算切过一片叶）	crates/singlefs-harness/src/model.rs	(rows / smallest_half_of_a_split(accounting_tree_leaf_entry_capacity())).saturating_sub(1);	(rows / smallest_half_of_a_split(accounting_tree_leaf_entry_capacity())).saturating_sub(0);	-p singlefs-harness --lib -- accounting_tree_nodes_grow_to_three	accounting_tree_nodes_grow_to_three_at_478_rows_and_keep_the_previous_bound
第49批 W3 再补 模型：记账树长层时不算根切开之后多长的新根	crates/singlefs-harness/src/model.rs	(leaf_splits_bound + internal_splits_bound).min(1 + internal_splits_bound);	(leaf_splits_bound + internal_splits_bound).min(0);	-p singlefs-harness --lib -- accounting_tree_nodes_grow_to_three	accounting_tree_nodes_grow_to_three_at_478_rows_and_keep_the_previous_bound
第49批 W3 再补 模型：写记账树的发布占槽只算根（根之下的节点不进占槽上界）	crates/singlefs-harness/src/model.rs	slots_written += role.span_in_slots() * accounting_tree_nodes;	slots_written += role.span_in_slots();	-p singlefs-harness --lib -- the_inode_creation_whose_watermark_step_crosses_one_accounting_leaf	the_inode_creation_whose_watermark_step_crosses_one_accounting_leaf_counts_the_nodes_below_the_root_in_the_occupied_slots
第49批 W3 再补 胶水：记账树根之下的节点照旧 panic（模型不罩记账树分裂）	crates/singlefs-harness/src/model_comparison.rs	TransactionUnit::AccountingTreeNodeBelowTheRoot(_) => return None,	TransactionUnit::AccountingTreeNodeBelowTheRoot(position) => panic!("模型今天只罩记账树一个节点的池：{position:?}"),	-p singlefs-harness --test accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf -- the_model_and_the_implementation	the_model_and_the_implementation_grow_the_accounting_tree_a_level_at_the_same_watermark_step
第49批 W3 再补 恢复：重建一版判 inode 号水位那一行只看统计量（只剩台阶也照建）	crates/singlefs-core/src/recovery.rs	entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation == root.checkpoint_txg	entry.statistic == STATISTIC_INODE_WATERMARK	-p singlefs-harness --test core_review_tree_table_duplicates_and_slot_one_search -- a_version_whose_own_inode_number_watermark_row	a_version_whose_own_inode_number_watermark_row_is_damaged_leaving_only_a_step_is_judged_damaged_by_the_rebuild_and_refused_by_the_writable_mount
第 50 批 R1：新池新建文件那次不零根注册退役树（树表退回 11 条）	crates/singlefs-core/src/transaction.rs	        table_entry(\n            TREE_KIND_RETIREMENT,	        table_entry(\n            TREE_KIND_LOGGED_OPERATIONS + 0 * TREE_KIND_RETIREMENT,	-p singlefs-harness --test new_pool_file_creation_publish -- tree_table_holds_twelve_entries_keyed_by_tree_identifier	tree_table_holds_twelve_entries_keyed_by_tree_identifier
第 50 批 R1：退役记录死亡 txg 段与容器出生代段写反	crates/singlefs-core/src/records.rs	        writer.put_u64(self.retired.container_birth.0);\n        writer.put_u64(self.death_txg.0);	        writer.put_u64(self.death_txg.0);\n        writer.put_u64(self.retired.container_birth.0);	-p singlefs-core --lib -- records::tests	a_retirement_record_is_thirty_four_bytes_identity_in_entry_order_then_the_death_txg
第 50 批 R1：退役记录的死亡 txg 取这次发布的前一个 txg（不是身份消失那一次）	crates/singlefs-core/src/retirement_tree.rs	            death_txg: checkpoint_txg,\n        })	            death_txg: CheckpointTxg(checkpoint_txg.0 - 1),\n        })	-p singlefs-harness --test inode_leaf_absorb_and_borrow_on_the_unlink_path -- the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one	the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one
第 50 批 死亡写侧：身份消失的那次发布不写退役记录	crates/singlefs-core/src/transaction.rs	            &retirement_records_of_this_publish(&inode_tree.retired_identities(), self.txg),	            &retirement_records_of_this_publish(&[], self.txg),	-p singlefs-harness --test checker_known_bad_images -- the_final_batch_that_empties_the_only_leaf_writes_one_retirement_record_and_every_retirement_invariant_holds	the_final_batch_that_empties_the_only_leaf_writes_one_retirement_record_and_every_retirement_invariant_holds
第 50 批 死亡写侧：被吸收的右片不写退役记录（只记删空摘掉的）	crates/singlefs-core/src/inode_tree.rs	        let absorbed = self.retire(left_position + 1);	        let absorbed = self.containers.remove(left_position + 1).contents;	-p singlefs-harness --test inode_leaf_absorb_and_borrow_on_the_unlink_path -- the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one	the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one
第 50 批 M3：两片之和恰好 233 不并（合并判据少一格）	crates/singlefs-core/src/inode_tree.rs	            <= INODE_LEAF_RECORDS\n	            < INODE_LEAF_RECORDS\n	-p singlefs-harness --test inode_leaf_absorb_and_borrow_on_the_unlink_path -- the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one	the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one
第 50 批 M3：左吸收右时身份留给右片（应留左半身份）	crates/singlefs-core/src/inode_tree.rs	        let absorbed = self.retire(left_position + 1);\n        let survivor = &mut self.containers[left_position];	        let absorbed = self.retire(left_position + 1);\n        self.containers[left_position].contents.identity = absorbed.identity;\n        let survivor = &mut self.containers[left_position];	-p singlefs-core --lib -- inode_tree::tests	the_left_container_absorbs_its_right_neighbour_when_the_two_fit_in_one_and_keeps_its_identity
第 50 批 M3：借不均分（多借一条）	crates/singlefs-core/src/inode_tree.rs	            .map_or(0, |difference| difference / 2);	            .map_or(0, |difference| difference / 2 + 1);	-p singlefs-core --lib -- inode_tree::tests	a_container_below_the_floor_borrows_half_the_difference_from_its_right_neighbour
第 50 批 N1：借之后右片的分隔 key 不改成它这一版的最小 key	crates/singlefs-core/src/inode_tree.rs	            .first()\n            .map(|record| record.inode)\n            .expect("右邻借出的条数少于它的记录数：借完至少还剩一条");	            .first()\n            .map(|_| lender.contents.identity.container)\n            .expect("右邻借出的条数少于它的记录数：借完至少还剩一条");	-p singlefs-harness --test inode_leaf_absorb_and_borrow_on_the_unlink_path -- a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing	a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing
第 50 批 M3：借完不从借出的那一片接着看合并	crates/singlefs-core/src/inode_tree.rs	        borrower.is_changed_this_publish = true;\n        self.absorb_until_no_pair_fits(lender_position);	        borrower.is_changed_this_publish = true;\n        let _ = lender_position;	-p singlefs-harness --test inode_leaf_absorb_and_borrow_on_the_unlink_path -- the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it	the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it
第 50 批 M3：低于下限也不借	crates/singlefs-core/src/inode_tree.rs	        if records_in_the_borrower >= records_below_which_a_leaf_borrows_from_its_right_neighbour()\n        {	        if records_in_the_borrower < usize::MAX\n        {	-p singlefs-harness --test inode_leaf_absorb_and_borrow_on_the_unlink_path -- a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing	a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing
第 50 批 R3：删空最左一片之后右边挪了位置的片按位置重写（新写的槽随右边片数涨）	crates/singlefs-core/src/inode_tree.rs	            if container.is_changed_this_publish {\n                rewritten.push(InodeLeafContainerIndexInTree::of_position(position));	            if container.is_changed_this_publish || container.previous_index.position() != position {\n                rewritten.push(InodeLeafContainerIndexInTree::of_position(position));	-p singlefs-core --lib -- inode_tree::tests	dropping_the_emptied_leftmost_container_carries_every_container_to_its_right_whatever_their_number
第 50 批 R2：标签 6 不加退役那 35 字节每条记录	crates/singlefs-core/src/admission.rs	                    RETIREMENT_PREPAY_BYTES_PER_INODE_RECORD_ON_ONE_REPLICA\n                        .checked_mul(inode_records)	                    RETIREMENT_PREPAY_BYTES_PER_INODE_RECORD_ON_ONE_REPLICA\n                        .checked_mul(0 * inode_records)	-p singlefs-core --lib -- admission::tests	the_retirement_share_of_tag_six_counts_every_inode_record_and_the_retirement_tree_levels
第 50 批 R2：标签 6 常驻不加退役树内部层数	crates/singlefs-core/src/admission.rs	                RETIREMENT_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA,\n                retirement_tree_internal_levels,\n	                RETIREMENT_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA,\n	-p singlefs-core --lib -- admission::tests	the_retirement_share_of_tag_six_counts_every_inode_record_and_the_retirement_tree_levels
第 50 批 R2：环窗口 W 不加退役那 6 + H_退役	crates/singlefs-core/src/admission.rs	pub const DELETION_PUBLISH_RETIREMENT_TREE_SLOTS_BEYOND_ITS_INTERNAL_LEVELS_AT_MOST: u64 = 6;	pub const DELETION_PUBLISH_RETIREMENT_TREE_SLOTS_BEYOND_ITS_INTERNAL_LEVELS_AT_MOST: u64 = 0;	-p singlefs-core --lib -- admission::tests	ring_window_on_two_four_gibibyte_devices_is_twenty_five_times_three_hundred_seventy_four_slots
第 50 批 R2：ckpt_cost 退役那一项不加退役树内部层数	crates/singlefs-core/src/admission.rs	        (RETIREMENT_CONTAINER_BLOCKS_OF_ONE_COW + RETIREMENT_TREE_ROOT_BLOCKS_OF_ONE_COW)\n            .checked_add(retirement_tree_internal_levels)	        (RETIREMENT_CONTAINER_BLOCKS_OF_ONE_COW + RETIREMENT_TREE_ROOT_BLOCKS_OF_ONE_COW)\n            .checked_add(0 * retirement_tree_internal_levels)	-p singlefs-core --lib -- admission::tests	the_retirement_term_of_the_checkpoint_cost_is_a_container_a_root_and_every_internal_level
第 50 批 checker I-9.31：退役树条目身份不判	crates/singlefs-checker/src/walk.rs	        if kind == TREE_KIND_RETIREMENT {\n            self.walk_container_tree_root(	        if kind == TREE_KIND_RETIREMENT && kind == TREE_KIND_LINEAGE {\n            self.walk_container_tree_root(	-p singlefs-harness --test checker_known_bad_images -- a_retirement_tree_entry_whose_identity_names_another_container_reddens_only_the_retirement_entry_invariant	a_retirement_tree_entry_whose_identity_names_another_container_reddens_only_the_retirement_entry_invariant
第 50 批 checker I-9.32：被退役身份的类型不判	crates/singlefs-checker/src/walk.rs	            let holds = retired.record_type == PACKED_TYPE_INODE\n	            let holds = retired.record_type == retired.record_type\n	-p singlefs-harness --test checker_known_bad_images -- a_retirement_record_naming_a_container_of_record_type_one_reddens_only_the_well_formed_retirement_record_invariant	a_retirement_record_naming_a_container_of_record_type_one_reddens_only_the_well_formed_retirement_record_invariant
第 50 批 checker I-9.33：谱系上同一身份两条不判	crates/singlefs-checker/src/walk.rs	            RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,\n            earlier_death_txg.is_none(),	            RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,\n            earlier_death_txg.is_none() || earlier_death_txg.is_some(),	-p singlefs-harness --test checker_known_bad_images -- a_second_copy_of_the_retirement_record_reddens_only_the_distinct_retirement_records_invariant	a_second_copy_of_the_retirement_record_reddens_only_the_distinct_retirement_records_invariant
第 50 批 checker I-9.33：不按谱系滤（删 → 回退 → 再删的合法两条误红）	crates/singlefs-checker/src/walk.rs	    let mut seen: BTreeMap<ContainerIdentityOfTheScan, u64> = BTreeMap::new();\n    for record in containers\n        .iter()\n        .flat_map(|container| container.records.iter())\n        .filter(|record| lineage_chain.holds_txg(record.death_txg))	    let mut seen: BTreeMap<ContainerIdentityOfTheScan, u64> = BTreeMap::new();\n    for record in containers\n        .iter()\n        .flat_map(|container| container.records.iter())\n        .filter(|record| lineage_chain.holds_txg(record.death_txg) || record.death_txg > 0)	-p singlefs-harness --test checker_known_bad_images -- unlinking_rolling_back_and_unlinking_again_writes_a_second_record_of_the_same_leaf_and_no_invariant_reddens	unlinking_rolling_back_and_unlinking_again_writes_a_second_record_of_the_same_leaf_and_no_invariant_reddens
第 50 批 checker I-9.34：谱系上的退役记录指活叶不判	crates/singlefs-checker/src/walk.rs	            !leaf_identities.contains(&record.retired),	            !leaf_identities.contains(&record.retired) || record.death_txg > 0,	-p singlefs-harness --test checker_known_bad_images -- a_retirement_record_naming_the_live_leaf_reddens_only_the_no_live_leaf_invariant	a_retirement_record_naming_the_live_leaf_reddens_only_the_no_live_leaf_invariant
第 50 批 checker I-9.35：漏记不判	crates/singlefs-checker/src/walk.rs	                missing.is_empty() && extra.is_empty(),	                extra.is_empty(),	-p singlefs-harness --test checker_known_bad_images -- a_retirement_record_moved_off_the_txg_the_leaf_died_at_reddens_only_the_retirement_completeness_invariant	a_retirement_record_moved_off_the_txg_the_leaf_died_at_reddens_only_the_retirement_completeness_invariant
第 50 批 checker I-9.35：多记不判	crates/singlefs-checker/src/walk.rs	                missing.is_empty() && extra.is_empty(),	                missing.is_empty(),	-p singlefs-harness --test checker_known_bad_images -- a_retirement_record_naming_a_leaf_the_parent_version_did_not_have_reddens_only_the_retirement_completeness_invariant	a_retirement_record_naming_a_leaf_the_parent_version_did_not_have_reddens_only_the_retirement_completeness_invariant
第 50 批 checker I-9.35：父版恒取 txg − 1（回退那一版不取 T_old）	crates/singlefs-checker/src/walk.rs	            .map_or(version.checkpoint_txg.checked_sub(1), |record| {\n                Some(record.parent_txg)\n            })	            .map_or(version.checkpoint_txg.checked_sub(1), |_| {\n                version.checkpoint_txg.checked_sub(1)\n            })	-p singlefs-harness --test checker_known_bad_images -- rolling_back_past_the_birth_of_a_leaf_compares_with_the_rollback_target_and_no_invariant_reddens	rolling_back_past_the_birth_of_a_leaf_compares_with_the_rollback_target_and_no_invariant_reddens
第 50 批 checker I-9.35：退役树头 ID 配不上 inode 树时照「没有退役记录」判漏记	crates/singlefs-checker/src/walk.rs	    if some_retirement_tree_names_no_inode_tree {	    if some_retirement_tree_names_no_inode_tree && false {	-p singlefs-harness --test checker_known_bad_images -- a_retirement_tree_entry_whose_head_identifier_names_no_inode_tree_reddens_only_the_head_identifier_invariant	a_retirement_tree_entry_whose_head_identifier_names_no_inode_tree_reddens_only_the_head_identifier_invariant
第 50 批 checker I-9.27：退役树（种类 13）条目不判头 ID	crates/singlefs-checker/src/walk.rs	            TREE_KIND_EXTENT\n            | TREE_KIND_DELETED_INODES\n            | TREE_KIND_TOMBSTONE\n            | TREE_KIND_RETIREMENT => Self::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable,	            TREE_KIND_EXTENT\n            | TREE_KIND_DELETED_INODES\n            | TREE_KIND_TOMBSTONE => Self::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable,\n            TREE_KIND_RETIREMENT => Self::NotJudged,	-p singlefs-harness --test checker_known_bad_images -- a_retirement_tree_entry_whose_head_identifier_names_no_inode_tree_reddens_only_the_head_identifier_invariant	a_retirement_tree_entry_whose_head_identifier_names_no_inode_tree_reddens_only_the_head_identifier_invariant
第 50 批 checker I-9.16 ④：缺退役树那一条放过（种类退回十一种）	crates/singlefs-checker/src/walk.rs	const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 12] =\n    TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER;	const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 11] = [\n    TREE_KIND_EXTENT,\n    TREE_KIND_INODE,\n    TREE_KIND_ALLOCATION,\n    TREE_KIND_ACCOUNTING,\n    TREE_KIND_LIVELIST,\n    TREE_KIND_SPARSE_SIDE_TABLE,\n    TREE_KIND_DEADLIST,\n    TREE_KIND_DELETED_INODES,\n    TREE_KIND_TOMBSTONE,\n    TREE_KIND_LINEAGE,\n    TREE_KIND_LOGGED_OPERATIONS,\n];	-p singlefs-harness --test checker_known_bad_images -- a_tree_table_without_its_retirement_entry_reddens_the_tree_table_ordering_invariant_besides_the_walk	a_tree_table_without_its_retirement_entry_reddens_the_tree_table_ordering_invariant_besides_the_walk
第 50 批 格式：一片退役容器装的条数按 33 字节一条算	crates/singlefs-format/src/lib.rs	pub const RETIREMENT_CONTAINER_RECORDS: u64 = 959;	pub const RETIREMENT_CONTAINER_RECORDS: u64 = 988;	-p singlefs-format --lib -- tests::deletion_widths_equal_the_field_sums_and_the_container_capacity_they_stand_for	deletion_widths_equal_the_field_sums_and_the_container_capacity_they_stand_for
第 50 批 容器树多条追加：一次发布跨两片时重写清单只记最后那一片	crates/singlefs-core/src/container_tree.rs	        if rewritten_containers.last() != Some(&rewritten_position) {\n            rewritten_containers.push(rewritten_position);\n        }	        rewritten_containers.clear();\n        rewritten_containers.push(rewritten_position);	-p singlefs-core --lib -- retirement_tree::tests	a_full_open_container_is_closed_and_a_publish_with_two_records_can_span_two_containers
第 50 批 容器树多条追加：一次发布跨两片时重写清单只记最后那一片（容器树自己那条用例）	crates/singlefs-core/src/container_tree.rs	        if rewritten_containers.last() != Some(&rewritten_position) {\n            rewritten_containers.push(rewritten_position);\n        }	        rewritten_containers.clear();\n        rewritten_containers.push(rewritten_position);	-p singlefs-core --lib -- container_tree::tests	appending_several_records_rewrites_every_touched_container_and_all_their_ancestors
第 50 批 退役树：开放容器满了不关、照样往里装	crates/singlefs-core/src/container_tree.rs	            .is_some_and(|open| open.records.len() < records_per_container);	            .is_some_and(|open| open.records.len() <= records_per_container);	-p singlefs-core --lib -- retirement_tree::tests	a_full_open_container_is_closed_and_a_publish_with_two_records_can_span_two_containers
第 50 批 退役树：根满不长高（多算一格扇出）	crates/singlefs-core/src/container_tree.rs	                shape.after_a_container_is_opened_at_the_right_end(node_entries.entries_per_node());	                shape.after_a_container_is_opened_at_the_right_end(node_entries.entries_per_node() + 1);	-p singlefs-core --lib -- retirement_tree::tests	opening_a_container_past_what_one_root_holds_grows_the_retirement_tree_one_level
第 50 批 退役树：没有新记录的发布照样重写开放容器	crates/singlefs-core/src/container_tree.rs	    let mut rewritten_containers: Vec<usize> = Vec::new();	    let mut rewritten_containers: Vec<usize> = containers.len().checked_sub(1).into_iter().collect();	-p singlefs-core --lib -- retirement_tree::tests	a_publish_without_retirement_records_carries_the_whole_tree
第 50 批 退役树只追加：往开放容器里装时盖掉已有的记录	crates/singlefs-core/src/container_tree.rs	                .records\n                .push(record.clone());	                .records\n                .splice(.., [record.clone()]);	-p singlefs-core --lib -- retirement_tree::tests	the_same_identity_retired_twice_keeps_both_records
第 50 批 退役树：只供测试的容量收 0 条	crates/singlefs-core/src/retirement_tree.rs	        (1..=RETIREMENT_CONTAINER_RECORDS)	        (0..=RETIREMENT_CONTAINER_RECORDS)	-p singlefs-core --lib -- retirement_tree::tests	the_capacity_capped_for_tests_lies_between_one_and_the_product_capacity
第 50 批 M3：借不均分（多借一条），一路删到并	crates/singlefs-core/src/inode_tree.rs	            .map_or(0, |difference| difference / 2);	            .map_or(0, |difference| difference / 2 + 1);	-p singlefs-core --lib -- inode_tree::tests	deleting_from_the_left_of_a_pair_borrows_until_the_pair_fits_in_one_and_then_absorbs
第 50 批 checker I-9.4：第三句（左容器最大 key < 右容器号）又判回来，借之后的合法两片误红	crates/singlefs-checker/src/walk.rs	                .judge("I-9.4", left.container < right.container, || {	                .judge("I-9.4", left.container < right.container && left.largest_inode < right.container, || {	-p singlefs-harness --test checker_narrow_invariants_and_abandoned_roots -- a_borrowed_right_container_numbered_below_the_largest_key_on_its_left_holds_every_invariant	a_borrowed_right_container_numbered_below_the_largest_key_on_its_left_holds_every_invariant
第 50 批补做 R3：发布路径上左吸收右之后右边每一片按位置重写、按位置换下（退回改之前的写法，40 片那一次比 20 片多写）	crates/singlefs-core/src/inode_tree.rs	        self.retired.sort_by_key(|retired| retired.previous_index);\n        let mut rewritten = Vec::new();\n        let mut origins = Vec::with_capacity(self.containers.len());\n        let mut containers = Vec::with_capacity(self.containers.len());\n        for (position, container) in self.containers.into_iter().enumerate() {\n            if container.is_changed_this_publish {	        self.retired.sort_by_key(|retired| retired.previous_index);\n        let previous_tail = self.containers.len();\n        for (offset, retired) in self.retired.iter_mut().enumerate() {\n            retired.previous_index = InodeLeafContainerIndexInTree::of_position(previous_tail + offset);\n        }\n        let mut rewritten = Vec::new();\n        let mut origins = Vec::with_capacity(self.containers.len());\n        let mut containers = Vec::with_capacity(self.containers.len());\n        for (position, container) in self.containers.into_iter().enumerate() {\n            if container.is_changed_this_publish || container.previous_index.position() != position {	-p singlefs-harness --test inode_leaf_absorb_and_borrow_on_the_unlink_path -- the_first_leaf_absorbing_the_second_writes_as_many_slots_in_a_tree_of_twenty_leaves_as_in_a_tree_of_forty	the_first_leaf_absorbing_the_second_writes_as_many_slots_in_a_tree_of_twenty_leaves_as_in_a_tree_of_forty
第 50 批补做 GPU 判器 I-9.32：死亡 txg 大于这一版的 txg 不判	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	                || !lt64(retired.container_birth, death_txg) || !le64(death_txg, version_txg)) {	                || !lt64(retired.container_birth, death_txg)) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_dies_after_its_version	the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_dies_after_its_version
第 50 批补做 GPU 判器 I-9.33：谱系上同一身份两条置成 I-9.32 那一位	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	violate(INVARIANT_I_9_33);	violate(INVARIANT_I_9_32);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_is_written_twice	the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_is_written_twice
第 50 批补做 GPU 判器 I-9.34：谱系上的退役记录指活叶置成 I-9.33 那一位	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	violate(INVARIANT_I_9_34);	violate(INVARIANT_I_9_33);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_names_the_surviving_left_leaf	the_gpu_judge_agrees_with_the_cpu_when_a_retirement_record_names_the_surviving_left_leaf
第 50 批补做 GPU 判器 I-9.35：漏记那一向不判	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	&& !d_retired_at(gone, version_txg)) {	&& !d_retired_at(gone, version_txg) && false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_retirement_record_of_a_gone_leaf_dies_at_the_parent_version	the_gpu_judge_agrees_with_the_cpu_when_the_retirement_record_of_a_gone_leaf_dies_at_the_parent_version
`````

## mutations-replacements.tsv

`````text
三·2 第 1 条：第一个文件版本那次不零根注册四棵新树（树表退回 7 条）	crates/singlefs-core/src/transaction.rs	        // 第一个文件版本那次零根注册的四棵（D8（核心索引结构） 已定项 16 第 3 条、D18（块里携带什么信息） 已定项 24 第 1 / 5 / 6 条）：\n        // 头 ID（D5（快照 / 空间记账机制） 已定项 9）：`deleted_inodes` 与墓碑树写 inode 树的树 ID，谱系树写 0（全池一份），\n        // logged_ops 写 0（字段表未定）。\n        // 队列空 / 墓碑树零根时根指针置零（D8（核心索引结构） 已定项 16 第 2、5 条）。\n        table_entry(\n            TREE_KIND_DELETED_INODES,\n            trees.deleted_inodes,\n            deletion_tree_units\n                .deleted_inodes_root\n                .as_ref()\n                .map_or_else(NodePointer::empty_root, |root| root.pointer),\n            trees.inode.0,\n        ),\n        table_entry(\n            TREE_KIND_TOMBSTONE,\n            trees.tombstone,\n            deletion_tree_units\n                .tombstone\n                .root\n                .as_ref()\n                .map_or_else(NodePointer::empty_root, |root| root.pointer),\n            trees.inode.0,\n        ),\n        // 谱系树零根时根指针置零；回退那次追加之后指着它的根，之后每一版照抄（随现行那一版）。\n        table_entry(\n            TREE_KIND_LINEAGE,\n            trees.lineage,\n            deletion_tree_units\n                .lineage\n                .root\n                .as_ref()\n                .map_or_else(NodePointer::empty_root, |root| root.pointer),\n            0,\n        ),\n        table_entry(\n            TREE_KIND_LOGGED_OPERATIONS,\n            trees.logged_operations,\n            NodePointer::empty_root(),\n            0,\n        ),\n		-p singlefs-harness --test new_pool_file_creation_publish -- tree_table_holds_twelve_entries_keyed_by_tree_identifier	tree_table_holds_twelve_entries_keyed_by_tree_identifier
三·2 第 1 条：第一个文件版本那次之后的树 ID 水位退回 65544（四棵新树不占号）	crates/singlefs-format/src/lib.rs	pub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 65549;	pub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 65544;	-p singlefs-format --lib -- tests::tree_identifiers_are_the_registered_values_in_a_row_from_the_first_value_the_kind_field_cannot_hold	tree_identifiers_are_the_registered_values_in_a_row_from_the_first_value_the_kind_field_cannot_hold
三·2 第 1 条：核心层判 I-9.16 ② 的发号次序不认四棵新树（退回七棵）	crates/singlefs-core/src/recovery.rs	const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 12] = [\n    TREE_KIND_EXTENT,\n    TREE_KIND_INODE,\n    TREE_KIND_ALLOCATION,\n    TREE_KIND_ACCOUNTING,\n    TREE_KIND_LIVELIST,\n    TREE_KIND_SPARSE_SIDE_TABLE,\n    TREE_KIND_DEADLIST,\n    TREE_KIND_DELETED_INODES,\n    TREE_KIND_TOMBSTONE,\n    TREE_KIND_LINEAGE,\n    TREE_KIND_LOGGED_OPERATIONS,\n    TREE_KIND_RETIREMENT,\n];	const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 7] = [\n    TREE_KIND_EXTENT,\n    TREE_KIND_INODE,\n    TREE_KIND_ALLOCATION,\n    TREE_KIND_ACCOUNTING,\n    TREE_KIND_LIVELIST,\n    TREE_KIND_SPARSE_SIDE_TABLE,\n    TREE_KIND_DEADLIST,\n];	-p singlefs-harness --test tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker -- the_cold_walk_refuses_a_tree_table_whose_tombstone_and_lineage_entries_swapped_their_kinds_before_reading_any_tree	the_cold_walk_refuses_a_tree_table_whose_tombstone_and_lineage_entries_swapped_their_kinds_before_reading_any_tree
三·2 第 7 条：池级 checker 判 I-9.16 ② 的发号次序不认四棵新树（退回七棵）	crates/singlefs-checker/src/walk.rs	        TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER\n            .iter()\n            .filter_map(|kind| {	        TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER[..7]\n            .iter()\n            .filter_map(|kind| {	-p singlefs-harness --test tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker -- the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds	the_checker_reddens_only_the_tree_table_ordering_when_the_tombstone_and_lineage_entries_swapped_their_kinds
树 ID 整体挪开（用户 2026-10-01 定案）：起点改回 11（十二棵 11..22、水位 11 / 23），格式常量与 D8 已定项 11 登记的值不再逐个相等	crates/singlefs-format/src/lib.rs	pub const TREE_IDENTIFIER_EXTENT: u64 = 65536;\npub const TREE_IDENTIFIER_INODE: u64 = 65537;\npub const TREE_IDENTIFIER_ALLOCATION_RECORDS: u64 = 65538;\npub const TREE_IDENTIFIER_ACCOUNTING: u64 = 65539;\npub const TREE_IDENTIFIER_CENTRAL_MAPPING: u64 = 65540;\npub const TREE_IDENTIFIER_LIVELIST: u64 = 65541;\npub const TREE_IDENTIFIER_SPARSE_SIDE_TABLE: u64 = 65542;\npub const TREE_IDENTIFIER_DEADLIST: u64 = 65543;\n/// 第一个文件版本那次零根注册的四棵树（D8（核心索引结构） 已定项 5 / 已定项 16 第 3 条；D18（块里携带什么信息） 已定项 24 第 1 / 5 / 6 条），\n/// 接在 deadlist 之后，次序照 D18 已定项 24 第 6 条列的：`deleted_inodes`、墓碑树、谱系树、logged_ops（树 ID 取值见 D8（核心索引结构） 已定项 11）。\npub const TREE_IDENTIFIER_DELETED_INODES: u64 = 65544;\npub const TREE_IDENTIFIER_TOMBSTONE: u64 = 65545;\npub const TREE_IDENTIFIER_LINEAGE: u64 = 65546;\npub const TREE_IDENTIFIER_LOGGED_OPERATIONS: u64 = 65547;\n/// 退役树：同样在第一个文件版本那次零根注册，接在 logged_ops 之后（D8（核心索引结构） 已定项 11；判决\n/// `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）。\npub const TREE_IDENTIFIER_RETIREMENT: u64 = 65548;\n/// 树 ID 水位：mkfs 写 65536，第一个文件版本那次之后写 65549（D8（核心索引结构） 已定项 8 ② / 已定项 11）。\npub const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u64 = 65536;\npub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 65549;	pub const TREE_IDENTIFIER_EXTENT: u64 = 11;\npub const TREE_IDENTIFIER_INODE: u64 = 12;\npub const TREE_IDENTIFIER_ALLOCATION_RECORDS: u64 = 13;\npub const TREE_IDENTIFIER_ACCOUNTING: u64 = 14;\npub const TREE_IDENTIFIER_CENTRAL_MAPPING: u64 = 15;\npub const TREE_IDENTIFIER_LIVELIST: u64 = 16;\npub const TREE_IDENTIFIER_SPARSE_SIDE_TABLE: u64 = 17;\npub const TREE_IDENTIFIER_DEADLIST: u64 = 18;\n/// 第一个文件版本那次零根注册的四棵树（D8（核心索引结构） 已定项 5 / 已定项 16 第 3 条；D18（块里携带什么信息） 已定项 24 第 1 / 5 / 6 条），\n/// 接在 deadlist 之后，次序照 D18 已定项 24 第 6 条列的：`deleted_inodes`、墓碑树、谱系树、logged_ops（树 ID 取值见 D8（核心索引结构） 已定项 11）。\npub const TREE_IDENTIFIER_DELETED_INODES: u64 = 19;\npub const TREE_IDENTIFIER_TOMBSTONE: u64 = 20;\npub const TREE_IDENTIFIER_LINEAGE: u64 = 21;\npub const TREE_IDENTIFIER_LOGGED_OPERATIONS: u64 = 22;\n/// 退役树：同样在第一个文件版本那次零根注册，接在 logged_ops 之后（D8（核心索引结构） 已定项 11；判决\n/// `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）。\npub const TREE_IDENTIFIER_RETIREMENT: u64 = 23;\n/// 树 ID 水位：mkfs 写 11，第一个文件版本那次之后写 24（D8（核心索引结构） 已定项 8 ② / 已定项 11）。\npub const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u64 = 11;\npub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 24;	-p singlefs-format --lib -- tests::tree_identifiers_are_the_registered_values_in_a_row_from_the_first_value_the_kind_field_cannot_hold	tree_identifiers_are_the_registered_values_in_a_row_from_the_first_value_the_kind_field_cannot_hold
树 ID 整体挪开：起点改回 11 之后 extent、inode 树的树 ID 与核心层的种类码 11、12 同值（种类码与树 ID 值域不相交那一判红）	crates/singlefs-format/src/lib.rs	pub const TREE_IDENTIFIER_EXTENT: u64 = 65536;\npub const TREE_IDENTIFIER_INODE: u64 = 65537;\npub const TREE_IDENTIFIER_ALLOCATION_RECORDS: u64 = 65538;\npub const TREE_IDENTIFIER_ACCOUNTING: u64 = 65539;\npub const TREE_IDENTIFIER_CENTRAL_MAPPING: u64 = 65540;\npub const TREE_IDENTIFIER_LIVELIST: u64 = 65541;\npub const TREE_IDENTIFIER_SPARSE_SIDE_TABLE: u64 = 65542;\npub const TREE_IDENTIFIER_DEADLIST: u64 = 65543;\n/// 第一个文件版本那次零根注册的四棵树（D8（核心索引结构） 已定项 5 / 已定项 16 第 3 条；D18（块里携带什么信息） 已定项 24 第 1 / 5 / 6 条），\n/// 接在 deadlist 之后，次序照 D18 已定项 24 第 6 条列的：`deleted_inodes`、墓碑树、谱系树、logged_ops（树 ID 取值见 D8（核心索引结构） 已定项 11）。\npub const TREE_IDENTIFIER_DELETED_INODES: u64 = 65544;\npub const TREE_IDENTIFIER_TOMBSTONE: u64 = 65545;\npub const TREE_IDENTIFIER_LINEAGE: u64 = 65546;\npub const TREE_IDENTIFIER_LOGGED_OPERATIONS: u64 = 65547;\n/// 退役树：同样在第一个文件版本那次零根注册，接在 logged_ops 之后（D8（核心索引结构） 已定项 11；判决\n/// `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）。\npub const TREE_IDENTIFIER_RETIREMENT: u64 = 65548;\n/// 树 ID 水位：mkfs 写 65536，第一个文件版本那次之后写 65549（D8（核心索引结构） 已定项 8 ② / 已定项 11）。\npub const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u64 = 65536;\npub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 65549;	pub const TREE_IDENTIFIER_EXTENT: u64 = 11;\npub const TREE_IDENTIFIER_INODE: u64 = 12;\npub const TREE_IDENTIFIER_ALLOCATION_RECORDS: u64 = 13;\npub const TREE_IDENTIFIER_ACCOUNTING: u64 = 14;\npub const TREE_IDENTIFIER_CENTRAL_MAPPING: u64 = 15;\npub const TREE_IDENTIFIER_LIVELIST: u64 = 16;\npub const TREE_IDENTIFIER_SPARSE_SIDE_TABLE: u64 = 17;\npub const TREE_IDENTIFIER_DEADLIST: u64 = 18;\n/// 第一个文件版本那次零根注册的四棵树（D8（核心索引结构） 已定项 5 / 已定项 16 第 3 条；D18（块里携带什么信息） 已定项 24 第 1 / 5 / 6 条），\n/// 接在 deadlist 之后，次序照 D18 已定项 24 第 6 条列的：`deleted_inodes`、墓碑树、谱系树、logged_ops（树 ID 取值见 D8（核心索引结构） 已定项 11）。\npub const TREE_IDENTIFIER_DELETED_INODES: u64 = 19;\npub const TREE_IDENTIFIER_TOMBSTONE: u64 = 20;\npub const TREE_IDENTIFIER_LINEAGE: u64 = 21;\npub const TREE_IDENTIFIER_LOGGED_OPERATIONS: u64 = 22;\n/// 退役树：同样在第一个文件版本那次零根注册，接在 logged_ops 之后（D8（核心索引结构） 已定项 11；判决\n/// `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）。\npub const TREE_IDENTIFIER_RETIREMENT: u64 = 23;\n/// 树 ID 水位：mkfs 写 11，第一个文件版本那次之后写 24（D8（核心索引结构） 已定项 8 ② / 已定项 11）。\npub const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u64 = 11;\npub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 24;	-p singlefs-core --lib -- records::tests::every_tree_kind_code_is_below_every_tree_identifier	every_tree_kind_code_is_below_every_tree_identifier
树 ID 整体挪开：起点改回 11 之后 extent、inode 树的树 ID 与池级 checker 自己那份种类码 11、12 同值（种类码与树 ID 值域不相交那一判红）	crates/singlefs-format/src/lib.rs	pub const TREE_IDENTIFIER_EXTENT: u64 = 65536;\npub const TREE_IDENTIFIER_INODE: u64 = 65537;\npub const TREE_IDENTIFIER_ALLOCATION_RECORDS: u64 = 65538;\npub const TREE_IDENTIFIER_ACCOUNTING: u64 = 65539;\npub const TREE_IDENTIFIER_CENTRAL_MAPPING: u64 = 65540;\npub const TREE_IDENTIFIER_LIVELIST: u64 = 65541;\npub const TREE_IDENTIFIER_SPARSE_SIDE_TABLE: u64 = 65542;\npub const TREE_IDENTIFIER_DEADLIST: u64 = 65543;\n/// 第一个文件版本那次零根注册的四棵树（D8（核心索引结构） 已定项 5 / 已定项 16 第 3 条；D18（块里携带什么信息） 已定项 24 第 1 / 5 / 6 条），\n/// 接在 deadlist 之后，次序照 D18 已定项 24 第 6 条列的：`deleted_inodes`、墓碑树、谱系树、logged_ops（树 ID 取值见 D8（核心索引结构） 已定项 11）。\npub const TREE_IDENTIFIER_DELETED_INODES: u64 = 65544;\npub const TREE_IDENTIFIER_TOMBSTONE: u64 = 65545;\npub const TREE_IDENTIFIER_LINEAGE: u64 = 65546;\npub const TREE_IDENTIFIER_LOGGED_OPERATIONS: u64 = 65547;\n/// 退役树：同样在第一个文件版本那次零根注册，接在 logged_ops 之后（D8（核心索引结构） 已定项 11；判决\n/// `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）。\npub const TREE_IDENTIFIER_RETIREMENT: u64 = 65548;\n/// 树 ID 水位：mkfs 写 65536，第一个文件版本那次之后写 65549（D8（核心索引结构） 已定项 8 ② / 已定项 11）。\npub const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u64 = 65536;\npub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 65549;	pub const TREE_IDENTIFIER_EXTENT: u64 = 11;\npub const TREE_IDENTIFIER_INODE: u64 = 12;\npub const TREE_IDENTIFIER_ALLOCATION_RECORDS: u64 = 13;\npub const TREE_IDENTIFIER_ACCOUNTING: u64 = 14;\npub const TREE_IDENTIFIER_CENTRAL_MAPPING: u64 = 15;\npub const TREE_IDENTIFIER_LIVELIST: u64 = 16;\npub const TREE_IDENTIFIER_SPARSE_SIDE_TABLE: u64 = 17;\npub const TREE_IDENTIFIER_DEADLIST: u64 = 18;\n/// 第一个文件版本那次零根注册的四棵树（D8（核心索引结构） 已定项 5 / 已定项 16 第 3 条；D18（块里携带什么信息） 已定项 24 第 1 / 5 / 6 条），\n/// 接在 deadlist 之后，次序照 D18 已定项 24 第 6 条列的：`deleted_inodes`、墓碑树、谱系树、logged_ops（树 ID 取值见 D8（核心索引结构） 已定项 11）。\npub const TREE_IDENTIFIER_DELETED_INODES: u64 = 19;\npub const TREE_IDENTIFIER_TOMBSTONE: u64 = 20;\npub const TREE_IDENTIFIER_LINEAGE: u64 = 21;\npub const TREE_IDENTIFIER_LOGGED_OPERATIONS: u64 = 22;\n/// 退役树：同样在第一个文件版本那次零根注册，接在 logged_ops 之后（D8（核心索引结构） 已定项 11；判决\n/// `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）。\npub const TREE_IDENTIFIER_RETIREMENT: u64 = 23;\n/// 树 ID 水位：mkfs 写 11，第一个文件版本那次之后写 24（D8（核心索引结构） 已定项 8 ② / 已定项 11）。\npub const TREE_IDENTIFIER_WATERMARK_AT_MKFS: u64 = 11;\npub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 24;	-p singlefs-checker --lib -- walk::tests::every_tree_kind_code_the_checker_reads_is_below_every_tree_identifier	every_tree_kind_code_the_checker_reads_is_below_every_tree_identifier
D8 已定项 8 头 ID：deleted_inodes 的树表条目头 ID 退回写 0（不写它服务的 inode 树的树 ID）	crates/singlefs-core/src/transaction.rs	            trees.deleted_inodes,\n            deletion_tree_units\n                .deleted_inodes_root\n                .as_ref()\n                .map_or_else(NodePointer::empty_root, |root| root.pointer),\n            trees.inode.0,\n	            trees.deleted_inodes,\n            deletion_tree_units\n                .deleted_inodes_root\n                .as_ref()\n                .map_or_else(NodePointer::empty_root, |root| root.pointer),\n            0,\n	-p singlefs-harness --test new_pool_file_creation_publish -- tree_table_holds_twelve_entries_keyed_by_tree_identifier	tree_table_holds_twelve_entries_keyed_by_tree_identifier
D8 已定项 8 头 ID：墓碑树 的树表条目头 ID 退回写 0（不写它服务的 inode 树的树 ID）	crates/singlefs-core/src/transaction.rs	            trees.tombstone,\n            deletion_tree_units\n                .tombstone\n                .root\n                .as_ref()\n                .map_or_else(NodePointer::empty_root, |root| root.pointer),\n            trees.inode.0,\n	            trees.tombstone,\n            deletion_tree_units\n                .tombstone\n                .root\n                .as_ref()\n                .map_or_else(NodePointer::empty_root, |root| root.pointer),\n            0,\n	-p singlefs-harness --test new_pool_file_creation_publish -- tree_table_holds_twelve_entries_keyed_by_tree_identifier	tree_table_holds_twelve_entries_keyed_by_tree_identifier
第 5 批 GPU 判器：I-3.13 记账行代段不比	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                w_judge(INVARIANT_I_3_13, eq64(row_generation, w_version_txg));	                w_judge(INVARIANT_I_3_13, true);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_accounting_row_carries_another_generation_than_its_version	the_gpu_judge_agrees_with_the_cpu_when_an_accounting_row_carries_another_generation_than_its_version
第 5 批 GPU 判器：派活给每个状态的草稿区停在交接段末端、没跟上内核新加的两张表	crates/singlefs-checker-tier/src/crash_judge_gpu.rs	pub const SCRATCH_WORDS: usize = 320_688;	pub const SCRATCH_WORDS: usize = 247_792;	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_gpu::tests::scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels	scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels
三·2 inode 树删一条：那一片还有记录时不进重写清单	crates/singlefs-core/src/inode_tree.rs	    holding.is_changed_this_publish = true;\n	    holding.is_changed_this_publish = false;\n	-p singlefs-core --lib -- inode_tree::tests	removing_a_record_that_triggers_no_merge_rewrites_only_that_container
三·2 inode 树删一条：删空最右那一片不记删空的片数（根不重写）	crates/singlefs-core/src/inode_tree.rs	        !self.rewritten.is_empty() || !self.retired.is_empty()\n	        !self.rewritten.is_empty()\n	-p singlefs-core --lib -- inode_tree::tests	removing_the_only_record_of_the_rightmost_container_drops_it_and_still_rewrites_the_root
三·2 inode 树删一条：删空中间一片时右边挪了位置的片不重写	crates/singlefs-core/src/inode_tree.rs	            if container.is_changed_this_publish {\n                rewritten.push(InodeLeafContainerIndexInTree::of_position(position));	            if container.is_changed_this_publish || container.previous_index.position() != position {\n                rewritten.push(InodeLeafContainerIndexInTree::of_position(position));	-p singlefs-core --lib -- inode_tree::tests	dropping_an_emptied_middle_container_carries_the_ones_to_its_right_without_rewriting_them
三·2 inode 树删一条：删空树里最后一条不记删空的片数（根不重写成零条目根）	crates/singlefs-core/src/inode_tree.rs	        !self.rewritten.is_empty() || !self.retired.is_empty()\n	        !self.rewritten.is_empty()\n	-p singlefs-core --lib -- inode_tree::tests	removing_the_last_record_of_the_tree_leaves_no_container_and_rewrites_the_root
三·2 第 6 条：W 每层不取 min(N_ℓ, 2·(K + 26))	crates/singlefs-core/src/admission.rs	.map(|positions| (*positions).min(bound_of_one_level))	.map(|positions| *positions)	-p singlefs-core --lib -- admission::tests	ring_window_on_two_four_gibibyte_devices_is_twenty_five_times_three_hundred_seventy_four_slots
三·2 第 6 条：环窗口少乘那一格 + 1（(3 × S) × W）	crates/singlefs-core/src/admission.rs	    let roots_in_the_ring_plus_one = ROOT_RING_REGIONS * slots_per_region.count() + 1;	    let roots_in_the_ring_plus_one = ROOT_RING_REGIONS * slots_per_region.count();	-p singlefs-core --lib -- admission::tests	ring_window_on_two_sixteen_gibibyte_devices_is_twenty_five_times_three_hundred_eighty_six_slots
三·2 第 6 条：墓碑预付每对象 32768 ÷ 815 向下取整（40，不是 41）	crates/singlefs-core/src/admission.rs	DATA_UNIT_BYTES.div_ceil(TOMBSTONE_CONTAINER_RECORDS)	DATA_UNIT_BYTES / TOMBSTONE_CONTAINER_RECORDS	-p singlefs-core --lib -- admission::tests	tag_six_on_root_only_tombstone_and_retirement_trees_is_six_slots_plus_forty_one_and_thirty_five_bytes_per_record
三·2 第 6 条：墓碑预付常驻退回 3 槽（不加墓碑树内部层数 H）	crates/singlefs-core/src/admission.rs	                TOMBSTONE_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA,\n                tombstone_tree_internal_levels,\n	                TOMBSTONE_PREPAY_OPEN_CONTAINER_SLOTS_ON_ONE_REPLICA,\n	-p singlefs-core --lib -- admission::tests	tag_six_after_the_tombstone_tree_grows_one_level_counts_one_more_slot
三·2 第 6 条：树表里没有 deleted_inodes 条目的版本照样记一槽墓碑预付	crates/singlefs-core/src/admission.rs	TombstoneAndRetirementPrepayBasis::VersionWithoutADeletedInodesTreeTableEntry => {\n            BytesSummedOverAllReplicas::ZERO	TombstoneAndRetirementPrepayBasis::VersionWithoutADeletedInodesTreeTableEntry => {\n            BytesSummedOverAllReplicas(SLOT_BYTES)	-p singlefs-core --lib -- admission::tests	tombstone_prepay_of_a_version_without_a_deleted_inodes_entry_is_zero
三·2 第 7 条：读侧不认墓碑树与谱系树的 key 宽	crates/singlefs-core/src/recovery.rs	        TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE | TREE_KIND_RETIREMENT => {\n            Some(CONTAINER_NUMBER_KEY_WIDTH_IN_BYTES)\n        }	        TREE_KIND_TOMBSTONE | TREE_KIND_LINEAGE | TREE_KIND_RETIREMENT => None,	-p singlefs-core --lib -- recovery::tree_kind_key_width_tests	deleted_inodes_tombstone_and_lineage_trees_have_eight_byte_keys_and_logged_operations_none
三·2 第 6 条：标签 6 的 n_live 恒取 0（记账行与发布路径的读数不按这一版 inode 树现算）	crates/singlefs-core/src/transaction.rs	            live_objects: live_objects_of_the_inode_tree(&resolved.inode_tree.containers),	            live_objects: 0,	-p singlefs-harness --test admission_ring_window_and_tombstone_prepay_on_the_publish_path	tag_six_of_the_first_file_version_is_six_slots_and_one_record_and_two_new_inodes_add_two_more
三·2 第 6 条：环窗口的批大小不取分配器上装的删除开关（恒按产品 144）	crates/singlefs-core/src/admission.rs	                    .data_units_per_batch(),\n                retirement_tree_internal_levels,	                    .data_units_per_batch()\n                    .max(144),\n                retirement_tree_internal_levels,	-p singlefs-harness --test admission_ring_window_and_tombstone_prepay_on_the_publish_path	the_ring_window_follows_the_batch_size_switch_installed_on_the_allocator
三·2 第 6 条：发布之前的读数不记环窗口	crates/singlefs-core/src/admission.rs	        committed_reservation_after_this_publish,\n        RingWindowOfTheReading::DeductedOnEachDevice {\n            slots_per_region,\n            retirement_tree_internal_levels: version_to_build_on\n                .map_or(0, TransactionOutput::retirement_tree_internal_levels),\n        },	        committed_reservation_after_this_publish,\n        RingWindowOfTheReading::NotCountedBeforeTheFirstFileVersion,	-p singlefs-harness --test admission_ring_window_and_tombstone_prepay_on_the_publish_path	the_publish_reading_deducts_the_ring_window_of_the_product_batch_on_each_device
三·2 第 6 条：可用(d) 不扣环窗口（式子末项漏扣）	crates/singlefs-core/src/admission.rs	                    terms.isolated_for_abandoned_roots_and_unapplied_journal_records,\n                    terms.ring_window,\n	                    terms.isolated_for_abandoned_roots_and_unapplied_journal_records,\n	-p singlefs-harness --test admission_ring_window_and_tombstone_prepay_on_the_publish_path	the_publish_reading_on_the_384_slot_small_devices_deducts_twenty_five_times_thirty_six_slots
三·2 第 5 条：冷走读的映射条目数不算谱系树	crates/singlefs-core/src/recovery.rs	        + container_tree_nodes(TREE_KIND_LINEAGE)\n        + container_tree_nodes(TREE_KIND_RETIREMENT);	        + container_tree_nodes(TREE_KIND_RETIREMENT);	-p singlefs-harness --test rollback_lineage_record_and_its_checkpoint_cost_term	a_rollback_appends_one_lineage_record_that_reads_back_field_by_field
三·2 第 5 条：谱系记录 T_rb 与 T_old 两段写反	crates/singlefs-core/src/records.rs	        writer.put_u64(self.rollback_txg.0);\n        writer.put_u64(self.rollback_target_txg.0);	        writer.put_u64(self.rollback_target_txg.0);\n        writer.put_u64(self.rollback_txg.0);	-p singlefs-core --lib	a_lineage_record_is_twenty_four_bytes_rollback_txg_then_target_txg_then_the_watermark_rolled_back_from
三·2 Q-U：ckpt_cost 中央映射树那一项的删插数不算谱系容器与谱系树根那几把	crates/singlefs-core/src/admission.rs	                        nodes.checked_add(central_mapping_entries_of_one_lineage_container_cow(\n                            lineage_tree_internal_levels,\n                        ))	                        nodes.checked_add(0 * central_mapping_entries_of_one_lineage_container_cow(\n                            lineage_tree_internal_levels,\n                        ))	-p singlefs-harness --test rollback_lineage_record_and_its_checkpoint_cost_term -- the_central_mapping_term_counts_the_lineage_entries_of_one_cow_on_a_zero_root_and_on_a_root_only_tree	the_central_mapping_term_counts_the_lineage_entries_of_one_cow_on_a_zero_root_and_on_a_root_only_tree
三·2 删除批第 12 批代码轮 U8：墓碑预付的 H 照旧读内存里的墓碑树形状，不读根节点码 2 头	crates/singlefs-core/src/transaction.rs	                    .expect("墓碑树根单元是这个进程装的，或重建时读出、核过的");\n            u64::from(root_header.level).saturating_sub(1)	                    .expect("墓碑树根单元是这个进程装的，或重建时读出、核过的");\n            {\n                let _ = root_header;\n                resolved.tombstone_tree.internal_levels()\n            }	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version	the_tombstone_prepay_of_an_overwrite_reads_the_tombstone_tree_height_from_the_root_node_header
C673 ⑤ I-9.27 判法（19-1 证据缺口）：deleted_inodes（种类 9）条目不判头 ID（它的头 ID 写 0 的坏镜像不红）	crates/singlefs-checker/src/walk.rs	            TREE_KIND_EXTENT\n            | TREE_KIND_DELETED_INODES\n            | TREE_KIND_TOMBSTONE\n            | TREE_KIND_RETIREMENT => Self::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable,	            TREE_KIND_EXTENT\n            | TREE_KIND_TOMBSTONE\n            | TREE_KIND_RETIREMENT => Self::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable,	-p singlefs-harness --test checker_known_bad_images -- each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
C673 ⑤ I-9.27 判法（19-1 证据缺口）：墓碑树（种类 10）条目不判头 ID（它的头 ID 写 0 的坏镜像不红）	crates/singlefs-checker/src/walk.rs	            TREE_KIND_EXTENT\n            | TREE_KIND_DELETED_INODES\n            | TREE_KIND_TOMBSTONE\n            | TREE_KIND_RETIREMENT => Self::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable,	            TREE_KIND_EXTENT\n            | TREE_KIND_DELETED_INODES\n            | TREE_KIND_RETIREMENT => Self::TreeIdentifierOfAnInodeTreeEntryOfTheSameTable,	-p singlefs-harness --test checker_known_bad_images -- each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant	each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant
回退那次发布不追加谱系记录（D 那一段的单元写退回 20，到 D 的段序列断言红）	crates/singlefs-core/src/transaction.rs	            lineage: LineageStepOfThePublish::AdministratorRollback {\n                record: lineage_record,\n                inode_number_watermark_after_the_rollback,\n            },	            lineage: {\n                let _ = inode_number_watermark_after_the_rollback;\n                LineageStepOfThePublish::NoLineageRecord\n            },	-p singlefs-checker-tier --test crash_enumeration_fixed_script_stream -- the_fixed_script_through_the_rollback_publish_keeps_its_registered_segment_sequence	the_fixed_script_through_the_rollback_publish_keeps_its_registered_segment_sequence
第10批 谱系槽号：回退那次发布不追加谱系记录（固定脚本 D 那一段不写谱系容器 50352 与谱系树根 50354）	crates/singlefs-core/src/transaction.rs	            lineage: LineageStepOfThePublish::AdministratorRollback {\n                record: lineage_record,\n                inode_number_watermark_after_the_rollback,\n            },	            lineage: {\n                let _ = inode_number_watermark_after_the_rollback;\n                LineageStepOfThePublish::NoLineageRecord\n            },	-p singlefs-checker-tier --test crash_enumeration_fixed_script_stream -- every_crash_state_of_the_quick_tier_recovers_to_the_version_its_root_claims	every_crash_state_of_the_quick_tier_recovers_to_the_version_its_root_claims
第 10 批 10b GPU 判器：扫描内核一张树表只记 8 棵树（改回树表 7 条时的上限，新池第一个文件版本的 11 条第 9 棵起判不了）	crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl	const C_MAX_TREES_PER_TABLE: u32 = 12u;	const C_MAX_TREES_PER_TABLE: u32 = 8u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- every_tree_table_the_new_pool_and_the_rollback_flows_write_fits_the_tree_count_the_gpu_scan_kernel_records	every_tree_table_the_new_pool_and_the_rollback_flows_write_fits_the_tree_count_the_gpu_scan_kernel_records
第 10 批 10b GPU 判器：恢复内核不把谱系树当容器树读（落进只读根的那一支，映射条目数少算谱系树的节点与容器）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	        if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) {\n            let read = read_container_tree_index_nodes(	        if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_RETIREMENT) {\n            let read = read_container_tree_index_nodes(	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows	the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows
第 10 批 10b GPU 判器：走读内核谱系记录的登记宽取 17（谱系容器的记录宽不按 16 判）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    } else if (record_type == PACKED_TYPE_LINEAGE) {\n        registered_width = 24u;	    } else if (record_type == PACKED_TYPE_LINEAGE) {\n        registered_width = 17u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_of_a_rollback_publish_carries_another_record_width	the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_of_a_rollback_publish_carries_another_record_width
第 10 批 10b GPU 判器：走读内核不读谱系树根（改回 key 形态只认 1–5、别的种类不走）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) { schema = 2u;	    else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_RETIREMENT) { schema = 2u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish_names_another_tree	the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish_names_another_tree
第 10 批 10b GPU 判器：扫描内核引用集合这一遍把谱系树记成数不全（改回种类 1–4 之外一律数不全）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        } else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE || kind == TREE_KIND_RETIREMENT) {\n            // 墓碑树、谱系树、退役树（walk.rs	        } else if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_RETIREMENT) {\n            // 墓碑树、谱系树、退役树（walk.rs	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows	the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows
26-3 挂载透明 续做规则的种类集合退回 {1, 2}	crates/singlefs-harness/src/mount_transparency.rs	    TREE_KIND_INODE,\n    TREE_KIND_DELETED_INODES,\n    TREE_KIND_TOMBSTONE,\n    TREE_KIND_LINEAGE,\n    TREE_KIND_RETIREMENT,\n];\n	    TREE_KIND_INODE,\n    TREE_KIND_EXTENT,\n    TREE_KIND_EXTENT,\n    TREE_KIND_EXTENT,\n    TREE_KIND_EXTENT,\n];\n	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount	a_deleted_inodes_root_pointer_that_moved_with_a_mount_that_did_not_continue_is_red
三·6 删除历史：只供测试的批大小从 1 改成 2（H1 少走一批）	crates/singlefs-harness/src/scenario.rs	        UnlinkBatchDataUnitsCappedForTests::new(1).expect("1 在 1..=产品批大小里"),	        UnlinkBatchDataUnitsCappedForTests::new(2).expect("1 在 1..=产品批大小里"),	-p singlefs-harness --test deletion_scenario_writes	the_in_process_history_publishes_the_intent_then_three_batches_of_thirteen_twelve_eleven_and_twelve_units_per_device
第28批 I-9.16 ④ 退回只要种类 1–4：缺 deadlist 那一条放过（正推 C1）	crates/singlefs-checker/src/walk.rs	const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 12] =\n    TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER;	const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 4] = [\n    TREE_KIND_EXTENT,\n    TREE_KIND_INODE,\n    TREE_KIND_ALLOCATION,\n    TREE_KIND_ACCOUNTING,\n];	-p singlefs-harness --test checker_known_bad_images -- a_tree_table_without_its_deadlist_entry_reddens_only_the_tree_table_ordering_invariant	a_tree_table_without_its_deadlist_entry_reddens_only_the_tree_table_ordering_invariant
第28批 I-9.16 ④ 退回只要种类 1–4：缺 logged_ops 那一条放过（正推 C1）	crates/singlefs-checker/src/walk.rs	const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 12] =\n    TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER;	const TREE_KINDS_EACH_EXACTLY_ONCE_IN_A_TREE_TABLE_WITH_ENTRIES: [u16; 4] = [\n    TREE_KIND_EXTENT,\n    TREE_KIND_INODE,\n    TREE_KIND_ALLOCATION,\n    TREE_KIND_ACCOUNTING,\n];	-p singlefs-harness --test checker_known_bad_images -- a_tree_table_without_its_logged_operations_entry_reddens_only_the_tree_table_ordering_invariant	a_tree_table_without_its_logged_operations_entry_reddens_only_the_tree_table_ordering_invariant
第22批 C I-1.7：谱系记录宽登记成 15	crates/singlefs-checker/src/walk.rs	PACKED_TYPE_LINEAGE => Some(24),	PACKED_TYPE_LINEAGE => Some(15),	-p singlefs-harness --test checker_known_bad_images	a_lineage_container_whose_record_width_is_not_the_registered_one_reddens_only_the_packed_container_invariant
第22批 C I-9.29：同号几个出生代也放过	crates/singlefs-checker/src/walk.rs	births.len() == 1	!births.is_empty()	-p singlefs-harness --test inode_number_watermark_pressed_back_reissues_a_number	pressing_the_watermark_back_to_the_largest_issued_number_reissues_it_and_reddens_the_one_birth_per_number_and_the_watermark_step_invariants
第10q批 扫描内核单调用点：墓碑与谱系的一版一判加回一个调用点（展开超预算）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        d_judge_tombstones_and_the_lineage_of_a_version(base, tree_table_offset, txg, is_the_newest_root);	        d_judge_tombstones_and_the_lineage_of_a_version(base, tree_table_offset, txg, is_the_newest_root);\n        d_judge_tombstones_and_the_lineage_of_a_version(base, tree_table_offset, txg, is_the_newest_root);	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_dispatch::tests::every_kernel_stays_within_the_inlined_size_budget	every_kernel_stays_within_the_inlined_size_budget
`````

## mutations-deletions.tsv

`````text
实审 B2 第 9 条（I-9.4 第三句）：左容器的最大 key 小于右容器号不判
实审 B2 第 9 条（I-9.4 第三句）：判严一格（左最大 key + 1 < 右容器号），右容器号恰比左最大 key 大一的合法两片误红
`````

## mutations-delete.txt

`````text
实审 B2 第 9 条（I-9.4 第三句）：左容器的最大 key 小于右容器号不判
实审 B2 第 9 条（I-9.4 第三句）：判严一格（左最大 key + 1 < 右容器号），右容器号恰比左最大 key 大一的合法两片误红
`````
