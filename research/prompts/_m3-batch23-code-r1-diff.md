# 删除批第 23 批合入的改动原样（2026-10-05）

## 实现员 g：crates.patch（/tmp/claude-1000/m3-batch23/g/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index ed91749..f0f05f3 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -1,6 +1,7 @@
 // GPU 判器的池级 checker 走读内核（接在 crash_judge_common.wgsl 与 crash_judge_checker_common.wgsl 之后）：按 singlefs-checker 自己的读法
 // 判第一批（系统配置、根环、journal）与第二批（走读那 29 条），连同里程碑三二·1 那一批里不在扫描内核的十五条（I-1.12、I-8.10、I-8.11、
-// I-4.9、I-9.25、I-3.13、I-9.26、I-7.16、I-7.17、I-8.12、I-7.15、I-4.1、I-8.1、I-7.5、I-8.5），违例位写进每个状态结论的违例位那几个字
+// I-4.9、I-9.25、I-3.13、I-9.26、I-7.16、I-7.17、I-8.12、I-7.15、I-4.1、I-8.1、I-7.5、I-8.5）与里程碑三·3 墓碑树、谱系树条目身份那两条
+// （I-9.23、I-9.17），违例位写进每个状态结论的违例位那几个字
 // （IMPLEMENTED_INVARIANTS 次序），之后一字是走到的批次、再一字是判不了的原因；走完把扫描内核要的计数与几何写进草稿区的交接段
 // （第三批与 I-3.12 在 crash_judge_checker_scan.wgsl）。
 
@@ -2634,9 +2635,14 @@ fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: v
             for (var r = 0u; r < p.record_count; r = r + 1u) {
                 let ro = packed_record_offset(p, r);
                 let inode = rd64(p.base, ro);
+                // I-9.4 第四句：同一片打包记录类型 2 容器里记录号沿记录序严格递增（判决 `research/prompts/m3-delete-checker-r2-main-verification.md`
+                // 第二节 T11 取甲；walk.rs walk_inode_leaf_container 逐对判）
+                if (r > 0u) {
+                    w_judge(INVARIANT_I_9_4, lt64(rd64(p.base, packed_record_offset(p, r - 1u)), inode));
+                }
                 w_judge(INVARIANT_I_9_7, all_zero(p.base, ro + 108u, 32u));
                 w_judge(INVARIANT_I_9_4, le64(p.container, inode));
-                let size = rd64(p.base, ro + 40u);
+                let size = rd64(p.base, ro + INODE_RECORD_SIZE_OFFSET);
                 let blocks = rd64(p.base, ro + 48u);
                 let logical_blocks = shr64(add64_saturating(size, u64_of(511u)), 9u);
                 w_judge(INVARIANT_I_9_15, eq64(blocks, logical_blocks));
@@ -2885,7 +2891,10 @@ fn w_walk_tree_table_entry(tt: Node, e: u32, fsid_low: vec2<u32>) {
 // ── 墓碑树或谱系树的码 2 根往下走（walk.rs 的 walk_container_tree_root 与 walk_the_entries_of_a_container_tree_node）
 // 条目 120 字节（分隔 key 8 + 身份引用 26 + 子指针 86）：类型段 = record_type 的走到码 3 容器（判容器头、出生树、身份、出生身份、容器格式，
 // 单元记进这一版走到的落点），类型段 0 的走进下一层码 2 节点（层级 = 父层级 − 1、条目宽 120 才往下走；层级 ≤ 1 的节点里出现类型段 0 记走读失败），
-// 别的类型段记走读失败。往下记的落点都记在正在走的那个码 2 节点下面（w_parent_*）。walk.rs 是递归，这里是同一个先序的显式栈，
+// 别的类型段先登记引用（层级 1 的节点按码 3 容器两槽、层级 ≥ 2 的按码 2 节点一槽）再判该条目损坏、不往下走。每条条目的身份单独判定
+// （墓碑树 I-9.23、谱系树 I-9.17），身份不符不记走读失败（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T7 取 a）：
+// 类型段 0 判身份引用三段为 0 且孩子（读得出时）是码 2；类型段 = record_type 判容器头四段与身份引用逐字相等、子指针出生树相等、类标签 3。
+// 往下记的落点都记在正在走的那个码 2 节点下面（w_parent_*）。walk.rs 是递归，这里是同一个先序的显式栈，
 // 深度超过 w_container_tree_frames 的 12 层判不了；两个种类只经这一处调用点
 struct WalkContainerTreeFrame {
     n: Node,
@@ -2897,6 +2906,10 @@ struct WalkContainerTreeFrame {
 var<private> w_container_tree_frames: array<WalkContainerTreeFrame, 12>;
 
 fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: vec2<u32>, record_type: u32, fsid_low: vec2<u32>) {
+    var identity_invariant = INVARIANT_I_9_23;
+    if (record_type == PACKED_TYPE_LINEAGE) {
+        identity_invariant = INVARIANT_I_9_17;
+    }
     let parent_valid = w_parent_valid;
     let parent_device = w_parent_device;
     let parent_slot = w_parent_slot;
@@ -2931,6 +2944,11 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
         w_judge_pointer_head(child);
         w_judge_pointer_extent_offset(child);
         if (entry_type == 0u) {
+            // 类型段 0：身份引用的出生树、容器号、出生代三段为 0；「子单元是码 2」与 I-1.6 重合、照 I-9.2 的先例留作合取，读不出的孩子这一小句不判
+            let child_unit = read_unit_via_locations(child, NODE_BYTES);
+            let child_is_a_code_two_node_or_unreadable = child_unit.version == NONE || rd8(child_unit.base, 6u) == 2u;
+            w_judge(identity_invariant, is_zero64(rd64(n.base, o + 8u)) && is_zero64(rd64(n.base, o + 18u)) && is_zero64(rd64(n.base, o + 26u))
+                && child_is_a_code_two_node_or_unreadable);
             if (n.level <= 1u) {
                 w_failures = w_failures + 1u;
                 continue;
@@ -2956,7 +2974,13 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
             continue;
         }
         if (entry_type != record_type) {
-            w_failures = w_failures + 1u;
+            // 类型段不在 {0, record_type} 里：先登记引用（不登记的话这个孩子占的槽不进引用并集，I-3.1 连带红），孩子的形状由这一层定
+            var span_of_the_unregistered_child = 2u;
+            if (n.level >= 2u) {
+                span_of_the_unregistered_child = 1u;
+            }
+            w_note_pointer_references(child, span_of_the_unregistered_child);
+            w_judge(identity_invariant, false);
             continue;
         }
         w_note_pointer_references(child, 2u);
@@ -2965,17 +2989,17 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
             w_failures = w_failures + 1u;
             continue;
         }
+        // 条目里的身份引用（出生树、类型段、容器号、容器出生代）与容器头逐段相同、子指针出生树相同、类标签 3，在查这一遍走没走过之前判
+        // （别的版本共享这片容器时也按这一版的条目判）
+        let identity_holds = rd8(s.base, 6u) == 3u && eq64(rd64(s.base, 43u), rd64(n.base, o + 8u)) && rd16(s.base, 51u) == entry_type
+            && eq64(rd64(s.base, 53u), rd64(n.base, o + 18u)) && eq64(rd64(s.base, 61u), rd64(n.base, o + 26u))
+            && eq64(child.birth_tree, rd64(n.base, o + 8u));
+        w_judge(identity_invariant, identity_holds);
         if (!w_visit(child.device0, child.slot0)) {
             w_merge_below(child.device0, child.slot0);
             continue;
         }
         w_judge(INVARIANT_I_1_3, eq64(rd64(s.base, 43u), tree));
-        // 条目里的身份引用（出生树、类型段、容器号、容器出生代）与容器头逐段相同，不同记走读失败
-        let identity_holds = eq64(rd64(s.base, 43u), rd64(n.base, o + 8u)) && rd16(s.base, 51u) == entry_type
-            && eq64(rd64(s.base, 53u), rd64(n.base, o + 18u)) && eq64(rd64(s.base, 61u), rd64(n.base, o + 26u));
-        if (!identity_holds) {
-            w_failures = w_failures + 1u;
-        }
         w_judge_birth_identity(s, child);
         // 容器格式（类标签、记录类型、记录宽与声明长度）按 I-1.7 判；记录本身的不变量归 I-9.17 起那几条，走读不判
         _ = w_judge_packed_container(s, record_type, fsid_low);
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
index 33b5b85..27c91d4 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
@@ -159,6 +159,27 @@ const INVARIANT_I_4_6: u32 = 72u;
 
 const INVARIANT_I_4_4: u32 = 73u;
 
+// 第 75–83 条：里程碑三·3 墓碑与谱系那九条（只往末尾追加）。走读内核判 I-9.17、I-9.23（墓碑树、谱系树每条内部条目的身份）；
+// 扫描内核在删除态扫描那一份版本清单上判 I-9.18–I-9.21、I-9.24、I-9.29。I-9.22 恒不置位：它只在池级 checker 带旋钮值的入口上判，
+// 层 0 与放量都走旧入口（`check_pool_image`，当没装旋钮、恒报不适用），GPU 手里也没有旋钮值——照 I-4.4 的先例登记、内核不置它那一位
+const INVARIANT_I_9_17: u32 = 74u;
+
+const INVARIANT_I_9_18: u32 = 75u;
+
+const INVARIANT_I_9_19: u32 = 76u;
+
+const INVARIANT_I_9_20: u32 = 77u;
+
+const INVARIANT_I_9_21: u32 = 78u;
+
+const INVARIANT_I_9_22: u32 = 79u;
+
+const INVARIANT_I_9_23: u32 = 80u;
+
+const INVARIANT_I_9_24: u32 = 81u;
+
+const INVARIANT_I_9_29: u32 = 82u;
+
 // 墓碑树、谱系树叶容器的打包记录类型（walk.rs 的 PACKED_TYPE_TOMBSTONE / PACKED_TYPE_LINEAGE；登记的记录宽墓碑 40、谱系 16）
 const PACKED_TYPE_TOMBSTONE: u32 = 1u;
 
@@ -757,9 +778,11 @@ const C_APPEARANCE_MAX_WALKED_TREE_TABLES: u32 = 64u;
 
 const C_APPEARANCE_WALKED_TREE_TABLES: u32 = 287728u;
 
-// ───────────────────────── 走读内核那几张表之后：扫描内核的删除态扫描（walk.rs judge_deletion_state_of_the_versions）一版一版重填的两张表 ─────────────────────────
-// ① 这一版 inode 树的记录（walk.rs inode_records_without_judging）：号 lo/hi、nlink、size lo/hi；同一个号再读到时后读到的盖掉先读到的
-const D_INODE_RECORD_WORDS: u32 = 5u;
+// ───────────────────────── 走读内核那几张表之后：扫描内核的删除态扫描（walk.rs judge_deletion_state_of_the_versions）与墓碑、谱系那一遍
+// （walk.rs judge_tombstones_and_the_lineage_of_the_versions）用的表 ─────────────────────────
+// ① 这一版一棵 inode 树的记录（walk.rs inode_records_without_judging）：号 lo/hi、nlink、size lo/hi、对象出生代 lo/hi；同一个号再读到时
+//    后读到的盖掉先读到的。一版一棵树地重填
+const D_INODE_RECORD_WORDS: u32 = 7u;
 
 const D_MAX_INODE_RECORDS: u32 = 1024u;
 
@@ -770,10 +793,34 @@ const D_DATA_UNIT_COUNT_WORDS: u32 = 3u;
 
 const D_MAX_DATA_UNIT_INODES: u32 = 1024u;
 
-const D_DATA_UNIT_COUNTS: u32 = 295664u;
+const D_DATA_UNIT_COUNTS: u32 = 297712u;
+
+// ③ 一棵墓碑树或谱系树按叶序读出的容器（walk.rs containers_of_a_container_tree_without_judging）：容器在打包表里的字节基、记录数、
+//    第一条记录的偏移、头里的容器号 lo/hi。一棵树地重填；多于 D_MAX_CONTAINERS_OF_A_CONTAINER_TREE 片判不了（交 CPU）
+const D_CONTAINER_OF_A_CONTAINER_TREE_WORDS: u32 = 5u;
+
+const D_MAX_CONTAINERS_OF_A_CONTAINER_TREE: u32 = 256u;
+
+const D_CONTAINERS_OF_A_CONTAINER_TREE: u32 = 300784u;
+
+// ④ 这一版谱系树里的谱系记录（walk.rs lineage_records_without_judging），按 T_rb 从大到小排好：T_rb lo/hi、T_old lo/hi。
+//    多于 D_MAX_LINEAGE_RECORDS 条判不了
+const D_LINEAGE_RECORD_WORDS: u32 = 4u;
+
+const D_MAX_LINEAGE_RECORDS: u32 = 256u;
+
+const D_LINEAGE_RECORDS: u32 = 302064u;
+
+// ⑤ I-9.29 跨版本那张表（walk.rs ObjectBirthsOfEachInodeNumber）：inode 树的树 ID lo/hi、inode 号 lo/hi、第一次见到的对象出生代 lo/hi。
+//    回退候选集里各版读完一起判，不一版一版重填；多于 D_MAX_OBJECT_BIRTHS 个 (树, 号) 判不了
+const D_OBJECT_BIRTH_WORDS: u32 = 6u;
+
+const D_MAX_OBJECT_BIRTHS: u32 = 2048u;
+
+const D_OBJECT_BIRTHS: u32 = 303088u;
 
 // 每个状态的草稿区到这里为止：crash_judge_gpu.rs 的 SCRATCH_WORDS 与它相等（那一份的单测核）
-const SCRATCH_END: u32 = 298736u;
+const SCRATCH_END: u32 = 315376u;
 
 fn handoff_region_device(region: u32) -> u32 {
     if (region == 0u) {
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index 5aca3a7..d930706 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -1,6 +1,7 @@
 // GPU 判器的池级 checker 扫描内核（接在 crash_judge_common.wgsl 与 crash_judge_checker_common.wgsl 之后）：走读内核判完、交接段有效时，
 // 判第三批——扫描方向与记账那九条（I-1.8、I-3.9、I-9.14、I-5.4、I-3.10、I-7.9、I-3.1、I-5.2、I-3.11），再加 I-3.12（每条根各自那张实例表）、
-// I-5.5 与 I-9.28（根表里每一条根的树表），再加里程碑三·3 删除线的删除态扫描（I-9.5、I-3.4、I-4.7、I-4.6，回退候选集每一版）与 I-9.6 墓碑那一半
+// I-5.5 与 I-9.28（根表里每一条根的树表），再加里程碑三·3 删除线的删除态扫描（I-9.5、I-3.4、I-4.7、I-4.6、I-9.24，回退候选集每一版）、
+// 墓碑与谱系那一遍（I-9.18、I-9.19、I-9.20、I-9.21、I-9.29 与 I-9.10 墓碑那一个等号，同一份版本清单）与 I-9.6 墓碑那一半
 // （所选根的墓碑树），违例位并进违例位那几个字，走到第几步那一字写成 5、判不了时下一字写原因。只读不判的读法照 walk.rs 的 *_without_judging 各写一份。
 
 
@@ -2063,17 +2064,26 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
 // 头 ID 指着这棵 inode 树的 extent 树不是恰好一条）在这里一格都不置位；CPU 按树表单元去重只扫一次，违例位是并集，这里不去重。
 // I-4.4 第一版报不适用（check_pool_image 恒报），内核不置它那一位。
 
-// inode 记录里 size（8 字节）的偏移、inode 记录宽（walk.rs INODE_RECORD_SIZE_OFFSET，类型 2 登记 140）；nlink 的偏移（walk.rs INODE_RECORD_NLINK_OFFSET）
-// 用共用那一半的 INODE_RECORD_NLINK_OFFSET（crash_judge_common.wgsl 拼在三个内核前面），不另定义
-const DELETION_INODE_RECORD_SIZE_OFFSET: u32 = 40u;
+// inode 记录宽（类型 2 登记 140）；对象出生代、nlink、size 的偏移用共用那一半的 INODE_RECORD_OBJECT_BIRTH_OFFSET、INODE_RECORD_NLINK_OFFSET、
+// INODE_RECORD_SIZE_OFFSET（crash_judge_common.wgsl 拼在三个内核前面），不另定义
 const DELETION_PACKED_TYPE_INODE: u32 = 2u;
 const DELETION_INODE_RECORD_BYTES: u32 = 140u;
 // inode 树与墓碑树码 2 节点条目里的类型段（偏移 16，2 字节）：0 指下一层码 2 节点，2 指 inode 叶容器（墓碑树里 1 指墓碑容器）
 const DELETION_CONTAINER_TREE_ENTRY_TYPE_OFFSET: u32 = 16u;
 const DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE: u32 = 0u;
-// 墓碑记录宽（类型 1 登记 40）与对象 ID 的偏移（walk.rs TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET）
+// 墓碑记录 40 字节（类型 1 登记 40）：对象 ID 0、对象出生代 8、区间起 16、区间止 24、死亡 txg 32（walk.rs TOMBSTONE_RECORD_*_OFFSET，
+// D18（块里携带什么信息） 已定项 24 第 2 条）；第一版只许整对象 [0, 2⁶⁴ − 1]；一片墓碑容器最多 815 条（⌊(32768 − 136) ÷ 40⌋，walk.rs TOMBSTONE_CONTAINER_RECORDS）
 const DELETION_TOMBSTONE_RECORD_BYTES: u32 = 40u;
 const DELETION_TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET: u32 = 0u;
+const DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET: u32 = 8u;
+const DELETION_TOMBSTONE_RECORD_RANGE_START_OFFSET: u32 = 16u;
+const DELETION_TOMBSTONE_RECORD_RANGE_END_OFFSET: u32 = 24u;
+const DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET: u32 = 32u;
+const DELETION_TOMBSTONE_CONTAINER_RECORDS: u32 = 815u;
+// 谱系记录 16 字节（类型 5 登记 16）：T_rb 0、T_old 8，读作「T_rb 的父是 T_old」（walk.rs LINEAGE_RECORD_*_OFFSET，D18（块里携带什么信息） 已定项 24 第 5 条）
+const DELETION_LINEAGE_RECORD_BYTES: u32 = 16u;
+const DELETION_LINEAGE_RECORD_ROLLBACK_TXG_OFFSET: u32 = 0u;
+const DELETION_LINEAGE_RECORD_PARENT_TXG_OFFSET: u32 = 8u;
 // 记账树：key 22（统计量 2、树 8、设备 4、代 8），叶条目 34，内部条目 key + 子指针 86；行里设备段在 10、值在 22
 const DELETION_ACCOUNTING_KEY_BYTES: u32 = 22u;
 const DELETION_ACCOUNTING_ENTRY_BYTES: u32 = 34u;
@@ -2091,13 +2101,15 @@ const DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET: u32 = 14u;
 var<private> d_inode_record_count: u32;
 
 // 这一版的一条 inode 记录记进 D_INODE_RECORDS：同一个号已有一行就盖掉（walk.rs 是按号插入的 BTreeMap，后读到的盖掉先读到的）；表满交回 false
-fn d_note_inode_record(inode: vec2<u32>, nlink: u32, size_in_bytes: vec2<u32>) -> bool {
+fn d_note_inode_record(inode: vec2<u32>, nlink: u32, size_in_bytes: vec2<u32>, object_birth: vec2<u32>) -> bool {
     for (var i = 0u; i < d_inode_record_count; i = i + 1u) {
         let row = D_INODE_RECORDS + i * D_INODE_RECORD_WORDS;
         if (sw(row) == inode.x && sw(row + 1u) == inode.y) {
             sws(row + 2u, nlink);
             sws(row + 3u, size_in_bytes.x);
             sws(row + 4u, size_in_bytes.y);
+            sws(row + 5u, object_birth.x);
+            sws(row + 6u, object_birth.y);
             return true;
         }
     }
@@ -2111,10 +2123,21 @@ fn d_note_inode_record(inode: vec2<u32>, nlink: u32, size_in_bytes: vec2<u32>) -
     sws(row + 2u, nlink);
     sws(row + 3u, size_in_bytes.x);
     sws(row + 4u, size_in_bytes.y);
+    sws(row + 5u, object_birth.x);
+    sws(row + 6u, object_birth.y);
     d_inode_record_count = d_inode_record_count + 1u;
     return true;
 }
 
+// D_INODE_RECORDS 里一行的 nlink 与对象出生代
+fn d_inode_record_nlink(row: u32) -> u32 {
+    return sw(row + 2u);
+}
+
+fn d_inode_record_object_birth(row: u32) -> vec2<u32> {
+    return vec2<u32>(sw(row + 5u), sw(row + 6u));
+}
+
 // 这一版 inode 记录表里号 inode 的那一行（没有交回 NONE）
 fn d_inode_record_row_of(inode: vec2<u32>) -> u32 {
     for (var i = 0u; i < d_inode_record_count; i = i + 1u) {
@@ -2196,7 +2219,7 @@ fn d_read_inode_records(root_pointer: Ptr) -> bool {
             for (var r = 0u; r < container.record_count; r = r + 1u) {
                 let record = packed_record_offset(container, r);
                 if (!d_note_inode_record(rd64(container.base, record), rd32(container.base, record + INODE_RECORD_NLINK_OFFSET),
-                    rd64(container.base, record + DELETION_INODE_RECORD_SIZE_OFFSET))) {
+                    rd64(container.base, record + INODE_RECORD_SIZE_OFFSET), rd64(container.base, record + INODE_RECORD_OBJECT_BIRTH_OFFSET))) {
                     return false;
                 }
             }
@@ -2293,10 +2316,52 @@ fn d_judge_pending_deletion_occupancy_of_an_accounting_tree(base: u32, offset: u
     }
 }
 
-// walk.rs judge_deletion_queue_of_a_version 里一条 deleted_inodes 条目（树表 tt 的条目偏移 deleted_inodes_entry）：经偏移 116 的头 ID 配同一张表里
-// 树 ID 等于它的恰好一条 inode 树条目，读队列与记录判 I-9.5 两个方向；再配头 ID 等于这棵 inode 树的恰好一条 extent 树条目，数每个号的数据单元判
-// I-4.7（对象取丙）、I-4.6（取乙）
+// walk.rs deleted_inodes_queue_without_judging 读回的一个队列根：readable 为假是读不出或形状不对（判不了）；present 为假是根指针全零的空队列
+struct DeletionQueueRead {
+    readable: bool,
+    present: bool,
+    queue: Node,
+};
+
+// 树表 tt 的条目偏移 deleted_inodes_entry 那条 deleted_inodes 条目的队列：根指针全零是空队列；否则要读得出、层级 0、key 宽 8、条目宽 ≥ 8
+fn d_read_deletion_queue(tt: Node, deleted_inodes_entry: u32) -> DeletionQueueRead {
+    var read: DeletionQueueRead;
+    read.readable = true;
+    read.queue = bad_node();
+    let queue_pointer = parse_node_pointer(tt.base, deleted_inodes_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
+    read.present = !queue_pointer.all_zero;
+    if (read.present) {
+        read.queue = r_read_index_node(queue_pointer);
+        if (!read.queue.ok || read.queue.level != 0u || read.queue.key_width != DELETED_INODES_ENTRY_BYTES
+            || read.queue.entry_width < DELETED_INODES_ENTRY_BYTES) {
+            read.readable = false;
+        }
+    }
+    return read;
+}
+
+// I-9.24：读得出的队列里不同的号至多一个（walk.rs 把队列读成按号的集合，同一个号的几条算一条）
+fn d_deletion_queue_holds_at_most_one_key(read: DeletionQueueRead) -> bool {
+    if (!read.present) {
+        return true;
+    }
+    for (var e = 1u; e < read.queue.entry_count; e = e + 1u) {
+        if (!eq64(rd64(read.queue.base, entry_offset(read.queue, e)), rd64(read.queue.base, entry_offset(read.queue, 0u)))) {
+            return false;
+        }
+    }
+    return true;
+}
+
+// walk.rs judge_deletion_queue_of_a_version 里一条 deleted_inodes 条目（树表 tt 的条目偏移 deleted_inodes_entry）：先读队列判 I-9.24（每一条
+// deleted_inodes 条目都判、在配对之前，判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T8 取 a）；再经偏移 116 的头 ID
+// 配同一张表里树 ID 等于它的恰好一条 inode 树条目，读记录判 I-9.5 两个方向；再配头 ID 等于这棵 inode 树的恰好一条 extent 树条目，
+// 数每个号的数据单元判 I-4.7（对象取丙）、I-4.6（取乙）
 fn d_judge_deletion_queue_of_an_entry(tt: Node, deleted_inodes_entry: u32) {
+    let queue_read = d_read_deletion_queue(tt, deleted_inodes_entry);
+    if (queue_read.readable && !d_deletion_queue_holds_at_most_one_key(queue_read)) {
+        violate(INVARIANT_I_9_24);
+    }
     let inode_tree = rd64(tt.base, deleted_inodes_entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
     var inode_tree_entry_count = 0u;
     var inode_tree_entry = 0u;
@@ -2314,19 +2379,11 @@ fn d_judge_deletion_queue_of_an_entry(tt: Node, deleted_inodes_entry: u32) {
             extent_tree_entry = entry;
         }
     }
-    if (inode_tree_entry_count != 1u) {
+    if (inode_tree_entry_count != 1u || !queue_read.readable) {
         return;
     }
-    // 队列（walk.rs deleted_inodes_queue_without_judging）：根指针全零是空队列；否则要读得出、层级 0、key 宽 8、条目宽 ≥ 8
-    let queue_pointer = parse_node_pointer(tt.base, deleted_inodes_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET);
-    let queue_present = !queue_pointer.all_zero;
-    var queue = bad_node();
-    if (queue_present) {
-        queue = r_read_index_node(queue_pointer);
-        if (!queue.ok || queue.level != 0u || queue.key_width != DELETED_INODES_ENTRY_BYTES || queue.entry_width < DELETED_INODES_ENTRY_BYTES) {
-            return;
-        }
-    }
+    let queue_present = queue_read.present;
+    let queue = queue_read.queue;
     if (!d_read_inode_records(parse_node_pointer(tt.base, inode_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET))) {
         return;
     }
@@ -2525,6 +2582,487 @@ fn judge_inode_number_watermark_above_every_published_tombstone() {
     }
 }
 
+// ───────────────────────── 墓碑与谱系那一遍（walk.rs judge_tombstones_and_the_lineage_of_the_versions）─────────────────────────
+// 删除态扫描那一份版本清单里每一版重读树表、谱系树、墓碑树、inode 树与 deleted_inodes（只读，不进 w_failures）：判 I-9.18、I-9.19、I-9.20、
+// I-9.21 与 I-9.10 墓碑那一个等号，把各版的 (inode 树, 号) → 对象出生代收进跨版本那张表（D_OBJECT_BIRTHS），同一个 (树, 号) 见到第二个出生代就判 I-9.29。
+// CPU 记成射程里判不了的那几格（树表读不出、条目宽不到 200、谱系树条目不是恰好一条、谱系树或墓碑树读不全、谱系接不成一条链、头 ID 配不上恰好一棵
+// inode 树或恰好一棵墓碑树、队列根读不出、队列里的号没有记录、inode 树读不全）在这里一格都不置位；CPU 按 (树表单元, txg) 去重只扫一次，
+// 违例位是并集，这里不去重。I-9.21 只按没装旋钮那一支判（层 0 与放量走池级 checker 的旧入口），I-9.22 不置位。不按已发布与否滤墓碑（第二节 T4）。
+// 表越过上限（一棵树的容器、一版的谱系记录、跨版本那张表、两两比不重号的墓碑总数）判不了，交 CPU
+
+var<private> d_container_count: u32;
+
+var<private> d_lineage_record_count: u32;
+
+var<private> d_object_birth_count: u32;
+
+// I-9.20 不重号那一句两两比：一棵墓碑树的墓碑多于这么多条就判不了（比较次数随条数平方涨）
+const DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE: u32 = 4096u;
+
+fn d_container_row(container_index: u32) -> u32 {
+    return D_CONTAINERS_OF_A_CONTAINER_TREE + container_index * D_CONTAINER_OF_A_CONTAINER_TREE_WORDS;
+}
+
+fn d_container_base(container_index: u32) -> u32 {
+    return sw(d_container_row(container_index));
+}
+
+fn d_container_record_count(container_index: u32) -> u32 {
+    return sw(d_container_row(container_index) + 1u);
+}
+
+// 第 container_index 片容器里第 record_index 条记录的偏移（记录宽读的时候核过是登记的 record_bytes）
+fn d_container_record_offset(container_index: u32, record_index: u32, record_bytes: u32) -> u32 {
+    return sw(d_container_row(container_index) + 2u) + record_index * record_bytes;
+}
+
+fn d_container_number(container_index: u32) -> vec2<u32> {
+    let row = d_container_row(container_index);
+    return vec2<u32>(sw(row + 3u), sw(row + 4u));
+}
+
+// 一棵墓碑树或谱系树（根指针 root_pointer）按叶序读出的容器进 D_CONTAINERS_OF_A_CONTAINER_TREE（walk.rs containers_of_a_container_tree_without_judging）：
+// 根指针全零交回空（零根）；码 2 节点要读得出、层级 ≥ 1、条目宽 ≥ 120，类型段 0 的条目指下一层码 2 节点（子指针非零、层级 = 父层级 − 1），
+// 类型段 = record_type 的指码 3 容器（打包记录类型 = record_type、记录宽 = 登记的 registered_record_bytes），别的类型段、读不出、形状不对交回 false（判不了）。
+// 条目倒着压进 C_R_PENDING、弹出来是叶序（walk.rs 按条目序递归）
+fn d_read_containers_of_a_container_tree(root_pointer: Ptr, record_type: u32, registered_record_bytes: u32) -> bool {
+    d_container_count = 0u;
+    if (root_pointer.all_zero) {
+        return true;
+    }
+    let root = r_read_index_node(root_pointer);
+    if (!root.ok || root.level == 0u || root.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
+        return false;
+    }
+    var pending = 0u;
+    if (!d_push_container_tree_entries_in_reverse(root, &pending, 55u)) {
+        return false;
+    }
+    while (pending > 0u) {
+        pending = pending - 1u;
+        let row = C_R_PENDING + pending * C_R_PENDING_WORDS;
+        let base = sw(row);
+        let o = sw(row + 1u);
+        let parent_level = sw(row + 2u);
+        let child_pointer = parse_node_pointer(base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET);
+        let entry_type = rd16(base, o + DELETION_CONTAINER_TREE_ENTRY_TYPE_OFFSET);
+        if (entry_type == DELETION_CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE) {
+            if (child_pointer.all_zero) {
+                return false;
+            }
+            let child = r_read_index_node(child_pointer);
+            if (!child.ok || child.level + 1u != parent_level || child.level == 0u || child.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
+                return false;
+            }
+            if (!d_push_container_tree_entries_in_reverse(child, &pending, 56u)) {
+                return false;
+            }
+        } else if (entry_type == record_type) {
+            if (child_pointer.all_zero) {
+                return false;
+            }
+            let container = parse_packed_unit(read_unit_via_locations(child_pointer, DATA_UNIT_BYTES));
+            if (!container.ok || container.record_type != record_type || container.record_width != registered_record_bytes) {
+                return false;
+            }
+            if (d_container_count >= D_MAX_CONTAINERS_OF_A_CONTAINER_TREE) {
+                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (57u << 8u);
+                return false;
+            }
+            let container_row = d_container_row(d_container_count);
+            sws(container_row, container.base);
+            sws(container_row + 1u, container.record_count);
+            sws(container_row + 2u, packed_record_offset(container, 0u));
+            sws(container_row + 3u, container.container.x);
+            sws(container_row + 4u, container.container.y);
+            d_container_count = d_container_count + 1u;
+        } else {
+            return false;
+        }
+    }
+    return true;
+}
+
+fn d_lineage_rollback_txg(lineage_index: u32) -> vec2<u32> {
+    let row = D_LINEAGE_RECORDS + lineage_index * D_LINEAGE_RECORD_WORDS;
+    return vec2<u32>(sw(row), sw(row + 1u));
+}
+
+fn d_lineage_parent_txg(lineage_index: u32) -> vec2<u32> {
+    let row = D_LINEAGE_RECORDS + lineage_index * D_LINEAGE_RECORD_WORDS;
+    return vec2<u32>(sw(row + 2u), sw(row + 3u));
+}
+
+// 这一版的谱系（walk.rs judge_tombstones_and_the_lineage_of_a_version 开头与 lineage_chain_of_a_version 判链之前那两步）：树表 tt 里谱系树（种类 11）
+// 恰好一条、整棵读得出、记录接得成一条链（每条 T_old < T_rb、没有两条同一个 T_rb）才交回 true，谱系记录按 T_rb 从大到小排进 D_LINEAGE_RECORDS；
+// 否则交回 false（这一版要谱系的那几判判不了）
+fn d_read_the_lineage_of_a_version(tt: Node) -> bool {
+    d_lineage_record_count = 0u;
+    var lineage_entry_count = 0u;
+    var lineage_entry = 0u;
+    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
+        let entry = entry_offset(tt, entry_index);
+        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_LINEAGE) {
+            lineage_entry_count = lineage_entry_count + 1u;
+            lineage_entry = entry;
+        }
+    }
+    if (lineage_entry_count != 1u) {
+        return false;
+    }
+    if (!d_read_containers_of_a_container_tree(parse_node_pointer(tt.base, lineage_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET),
+        PACKED_TYPE_LINEAGE, DELETION_LINEAGE_RECORD_BYTES)) {
+        return false;
+    }
+    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
+        let base = d_container_base(container_index);
+        for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
+            if (d_lineage_record_count >= D_MAX_LINEAGE_RECORDS) {
+                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (58u << 8u);
+                return false;
+            }
+            let record = d_container_record_offset(container_index, record_index, DELETION_LINEAGE_RECORD_BYTES);
+            let rollback_txg = rd64(base, record + DELETION_LINEAGE_RECORD_ROLLBACK_TXG_OFFSET);
+            let parent_txg = rd64(base, record + DELETION_LINEAGE_RECORD_PARENT_TXG_OFFSET);
+            let row = D_LINEAGE_RECORDS + d_lineage_record_count * D_LINEAGE_RECORD_WORDS;
+            sws(row, rollback_txg.x);
+            sws(row + 1u, rollback_txg.y);
+            sws(row + 2u, parent_txg.x);
+            sws(row + 3u, parent_txg.y);
+            d_lineage_record_count = d_lineage_record_count + 1u;
+        }
+    }
+    // 按 T_rb 从大到小排（选择排序：条数不过 D_MAX_LINEAGE_RECORDS）
+    for (var sorted = 0u; sorted < d_lineage_record_count; sorted = sorted + 1u) {
+        var latest = sorted;
+        for (var candidate = sorted + 1u; candidate < d_lineage_record_count; candidate = candidate + 1u) {
+            if (lt64(d_lineage_rollback_txg(latest), d_lineage_rollback_txg(candidate))) {
+                latest = candidate;
+            }
+        }
+        if (latest != sorted) {
+            let sorted_row = D_LINEAGE_RECORDS + sorted * D_LINEAGE_RECORD_WORDS;
+            let latest_row = D_LINEAGE_RECORDS + latest * D_LINEAGE_RECORD_WORDS;
+            for (var word = 0u; word < D_LINEAGE_RECORD_WORDS; word = word + 1u) {
+                let kept = sw(sorted_row + word);
+                sws(sorted_row + word, sw(latest_row + word));
+                sws(latest_row + word, kept);
+            }
+        }
+    }
+    for (var lineage_index = 0u; lineage_index < d_lineage_record_count; lineage_index = lineage_index + 1u) {
+        if (!lt64(d_lineage_parent_txg(lineage_index), d_lineage_rollback_txg(lineage_index))) {
+            return false;
+        }
+        if (lineage_index > 0u && eq64(d_lineage_rollback_txg(lineage_index - 1u), d_lineage_rollback_txg(lineage_index))) {
+            return false;
+        }
+    }
+    return true;
+}
+
+// 「在谱系上」（walk.rs LineageChainOfAVersion::holds_txg 与 lineage_chain_of_a_version 的按段走）：从这一版自己的 txg 起，谱系记录按 T_rb 从大到小过一遍，
+// T_rb 大于此刻走到的 txg 的跳过，否则 [T_rb, 此刻] 整段在链上、跳到 T_old；最后 [0, 此刻] 也在链上
+fn d_txg_is_on_the_lineage(txg: vec2<u32>, version_txg: vec2<u32>) -> bool {
+    var txg_walked_to = version_txg;
+    for (var lineage_index = 0u; lineage_index < d_lineage_record_count; lineage_index = lineage_index + 1u) {
+        let rollback_txg = d_lineage_rollback_txg(lineage_index);
+        if (lt64(txg_walked_to, rollback_txg)) {
+            continue;
+        }
+        if (le64(rollback_txg, txg) && le64(txg, txg_walked_to)) {
+            return true;
+        }
+        txg_walked_to = d_lineage_parent_txg(lineage_index);
+    }
+    return le64(txg, txg_walked_to);
+}
+
+// I-9.29 跨版本那张表记一笔：(inode 树, 号) 第一次见到时记下对象出生代，再见到而出生代不同就判红（walk.rs 收齐之后每个号判一次
+// births.len() == 1，与这里见到第二个不同的出生代就红是同一判）。表满判不了
+fn d_note_object_birth(inode_tree: vec2<u32>, inode: vec2<u32>, object_birth: vec2<u32>) {
+    for (var birth_index = 0u; birth_index < d_object_birth_count; birth_index = birth_index + 1u) {
+        let row = D_OBJECT_BIRTHS + birth_index * D_OBJECT_BIRTH_WORDS;
+        if (sw(row) == inode_tree.x && sw(row + 1u) == inode_tree.y && sw(row + 2u) == inode.x && sw(row + 3u) == inode.y) {
+            if (!eq64(vec2<u32>(sw(row + 4u), sw(row + 5u)), object_birth)) {
+                violate(INVARIANT_I_9_29);
+            }
+            return;
+        }
+    }
+    if (d_object_birth_count >= D_MAX_OBJECT_BIRTHS) {
+        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (59u << 8u);
+        return;
+    }
+    let row = D_OBJECT_BIRTHS + d_object_birth_count * D_OBJECT_BIRTH_WORDS;
+    sws(row, inode_tree.x);
+    sws(row + 1u, inode_tree.y);
+    sws(row + 2u, inode.x);
+    sws(row + 3u, inode.y);
+    sws(row + 4u, object_birth.x);
+    sws(row + 5u, object_birth.y);
+    d_object_birth_count = d_object_birth_count + 1u;
+}
+
+// 一条墓碑（容器字节基 base、记录偏移 o）是不是整对象 [0, 2⁶⁴ − 1]（walk.rs TombstoneRecordOfTheScan::covers_the_whole_object：区间起 < 区间止照条款原文合取）
+fn d_tombstone_covers_the_whole_object(base: u32, o: u32) -> bool {
+    let range_start = rd64(base, o + DELETION_TOMBSTONE_RECORD_RANGE_START_OFFSET);
+    let range_end = rd64(base, o + DELETION_TOMBSTONE_RECORD_RANGE_END_OFFSET);
+    return lt64(range_start, range_end) && is_zero64(range_start) && eq64(range_end, vec2<u32>(NONE, NONE));
+}
+
+// I-9.21（walk.rs judge_tombstone_container_capacity_of_a_version 没装旋钮那一支）：读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的墓碑容器至少两片时，
+// 沿叶序容器号严格递增（读容器头，不沿根的条目读），最右那片之外每片记录数 == 815；零根或只一片没有对象
+fn d_judge_tombstone_container_capacity() {
+    if (d_container_count < 2u) {
+        return;
+    }
+    for (var container_index = 1u; container_index < d_container_count; container_index = container_index + 1u) {
+        if (!lt64(d_container_number(container_index - 1u), d_container_number(container_index))) {
+            violate(INVARIANT_I_9_21);
+        }
+    }
+    for (var container_index = 0u; container_index + 1u < d_container_count; container_index = container_index + 1u) {
+        if (d_container_record_count(container_index) != DELETION_TOMBSTONE_CONTAINER_RECORDS) {
+            violate(INVARIANT_I_9_21);
+        }
+    }
+}
+
+// I-9.20 管全部墓碑的那几句（walk.rs judge_every_tombstone_of_a_version_is_well_formed）：每片容器记录数 ≤ 815（与 I-1.7 重合，留作合取）；
+// 每条整对象、对象出生代 ≤ 死亡 txg ≤ 这一版的 txg
+fn d_judge_every_tombstone_is_well_formed(version_txg: vec2<u32>) {
+    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
+        let base = d_container_base(container_index);
+        let record_count = d_container_record_count(container_index);
+        if (record_count > DELETION_TOMBSTONE_CONTAINER_RECORDS) {
+            violate(INVARIANT_I_9_20);
+        }
+        for (var record_index = 0u; record_index < record_count; record_index = record_index + 1u) {
+            let o = d_container_record_offset(container_index, record_index, DELETION_TOMBSTONE_RECORD_BYTES);
+            if (!d_tombstone_covers_the_whole_object(base, o)) {
+                violate(INVARIANT_I_9_20);
+            }
+            let death_txg = rd64(base, o + DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET);
+            if (!le64(rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET), death_txg) || !le64(death_txg, version_txg)) {
+                violate(INVARIANT_I_9_20);
+            }
+        }
+    }
+}
+
+// I-9.20 不重号那一句（walk.rs judge_tombstones_on_the_lineage_of_a_version_are_distinct，只判在谱系上的）：这一版在谱系上的墓碑按 (号, 对象出生代)
+// 不重复。两两比，墓碑总数多于 DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE 判不了
+fn d_judge_tombstones_on_the_lineage_are_distinct(version_txg: vec2<u32>) {
+    var tombstone_count = 0u;
+    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
+        tombstone_count = tombstone_count + d_container_record_count(container_index);
+    }
+    if (tombstone_count > DELETION_MAX_TOMBSTONES_COMPARED_PAIRWISE) {
+        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (60u << 8u);
+        return;
+    }
+    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
+        let base = d_container_base(container_index);
+        for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
+            let o = d_container_record_offset(container_index, record_index, DELETION_TOMBSTONE_RECORD_BYTES);
+            if (!d_txg_is_on_the_lineage(rd64(base, o + DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET), version_txg)) {
+                continue;
+            }
+            let object = rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET);
+            let object_birth = rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET);
+            // 叶序在它之前的墓碑：前几片容器的全部、同一片里它之前的
+            for (var earlier_container = 0u; earlier_container <= container_index; earlier_container = earlier_container + 1u) {
+                let earlier_base = d_container_base(earlier_container);
+                var earlier_limit = d_container_record_count(earlier_container);
+                if (earlier_container == container_index) {
+                    earlier_limit = record_index;
+                }
+                for (var earlier_record = 0u; earlier_record < earlier_limit; earlier_record = earlier_record + 1u) {
+                    let earlier = d_container_record_offset(earlier_container, earlier_record, DELETION_TOMBSTONE_RECORD_BYTES);
+                    if (eq64(rd64(earlier_base, earlier + DELETION_TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET), object)
+                        && eq64(rd64(earlier_base, earlier + DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET), object_birth)
+                        && d_txg_is_on_the_lineage(rd64(earlier_base, earlier + DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET), version_txg)) {
+                        violate(INVARIANT_I_9_20);
+                    }
+                }
+            }
+        }
+    }
+}
+
+// 读进 D_CONTAINERS_OF_A_CONTAINER_TREE 的墓碑树对 D_INODE_RECORDS 里配上的那棵 inode 树（头 ID inode_tree）：
+// I-9.10 墓碑那一个等号（walk.rs judge_tombstones_against_the_object_births_of_the_records_by_number：按号认、不看谱系，有记录时对象出生代相等）；
+// 谱系接得成链时（lineage_holds）再判 I-9.19（walk.rs judge_tombstones_on_the_lineage_name_no_live_object：谱系上的墓碑有号与出生代都相同的记录时
+// 那条记录 nlink = 0），并把谱系上每条墓碑的 (号, 对象出生代) 记进 I-9.29 那张表
+fn d_judge_tombstones_against_the_records(inode_tree: vec2<u32>, version_txg: vec2<u32>, lineage_holds: bool) {
+    for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
+        let base = d_container_base(container_index);
+        for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
+            let o = d_container_record_offset(container_index, record_index, DELETION_TOMBSTONE_RECORD_BYTES);
+            let object = rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET);
+            let object_birth = rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET);
+            let record_row = d_inode_record_row_of(object);
+            if (record_row != NONE && !eq64(d_inode_record_object_birth(record_row), object_birth)) {
+                violate(INVARIANT_I_9_10);
+            }
+            if (!lineage_holds || !d_txg_is_on_the_lineage(rd64(base, o + DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET), version_txg)) {
+                continue;
+            }
+            if (record_row != NONE && eq64(d_inode_record_object_birth(record_row), object_birth) && d_inode_record_nlink(record_row) != 0u) {
+                violate(INVARIANT_I_9_19);
+            }
+            d_note_object_birth(inode_tree, object, object_birth);
+        }
+    }
+}
+
+// I-9.18（walk.rs judge_queued_inodes_of_an_inode_tree_have_tombstones_on_the_lineage）在树表 tt 的一棵 inode 树（条目偏移 inode_tree_entry）上：
+// 经头 ID 配这棵树的 deleted_inodes（恰好一条）与墓碑树（恰好一条），队列里每个号 N 要在墓碑树里有 (N, N 的记录的对象出生代, 整对象)、
+// 死亡 txg 在这一版的谱系上；N 没有记录的判不了、不判红（那一形只让 I-9.5 红）。队列空的没有对象
+fn d_judge_queued_inodes_have_tombstones_on_the_lineage(tt: Node, inode_tree_entry: u32, version_txg: vec2<u32>, lineage_holds: bool) {
+    let inode_tree = rd64(tt.base, inode_tree_entry);
+    var deleted_inodes_entry_count = 0u;
+    var deleted_inodes_entry = 0u;
+    var tombstone_tree_entry_count = 0u;
+    var tombstone_tree_entry = 0u;
+    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
+        let entry = entry_offset(tt, entry_index);
+        let kind = rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET);
+        let head_identifier = rd64(tt.base, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
+        if (kind == TREE_KIND_DELETED_INODES && eq64(head_identifier, inode_tree)) {
+            deleted_inodes_entry_count = deleted_inodes_entry_count + 1u;
+            deleted_inodes_entry = entry;
+        }
+        if (kind == TREE_KIND_TOMBSTONE && eq64(head_identifier, inode_tree)) {
+            tombstone_tree_entry_count = tombstone_tree_entry_count + 1u;
+            tombstone_tree_entry = entry;
+        }
+    }
+    if (deleted_inodes_entry_count != 1u) {
+        return;
+    }
+    let queue_read = d_read_deletion_queue(tt, deleted_inodes_entry);
+    if (!queue_read.readable || !queue_read.present || queue_read.queue.entry_count == 0u) {
+        return;
+    }
+    if (!d_read_inode_records(parse_node_pointer(tt.base, inode_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET))) {
+        return;
+    }
+    if (tombstone_tree_entry_count != 1u) {
+        return;
+    }
+    if (!d_read_containers_of_a_container_tree(parse_node_pointer(tt.base, tombstone_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET),
+        PACKED_TYPE_TOMBSTONE, DELETION_TOMBSTONE_RECORD_BYTES)) {
+        return;
+    }
+    if (!lineage_holds) {
+        return;
+    }
+    let queue = queue_read.queue;
+    for (var queue_index = 0u; queue_index < queue.entry_count; queue_index = queue_index + 1u) {
+        let inode = rd64(queue.base, entry_offset(queue, queue_index));
+        let record_row = d_inode_record_row_of(inode);
+        if (record_row == NONE) {
+            continue;
+        }
+        let object_birth = d_inode_record_object_birth(record_row);
+        var has_a_whole_object_tombstone_on_the_lineage = false;
+        for (var container_index = 0u; container_index < d_container_count; container_index = container_index + 1u) {
+            let base = d_container_base(container_index);
+            for (var record_index = 0u; record_index < d_container_record_count(container_index); record_index = record_index + 1u) {
+                let o = d_container_record_offset(container_index, record_index, DELETION_TOMBSTONE_RECORD_BYTES);
+                if (eq64(rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET), inode)
+                    && eq64(rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET), object_birth)
+                    && d_tombstone_covers_the_whole_object(base, o)
+                    && d_txg_is_on_the_lineage(rd64(base, o + DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET), version_txg)) {
+                    has_a_whole_object_tombstone_on_the_lineage = true;
+                }
+            }
+        }
+        if (!has_a_whole_object_tombstone_on_the_lineage) {
+            violate(INVARIANT_I_9_18);
+        }
+    }
+}
+
+// walk.rs judge_tombstones_and_the_lineage_of_a_version 的一版：树表指针在 (base, tree_table_pointer_offset)，这一版的 txg 是 version_txg。
+// 指针全零、树表 0 条的不判；树表读不出、条目宽不到 200 的不判
+fn d_judge_tombstones_and_the_lineage_of_a_version(base: u32, tree_table_pointer_offset: u32, version_txg: vec2<u32>) {
+    let tree_table_pointer = parse_node_pointer(base, tree_table_pointer_offset);
+    if (tree_table_pointer.all_zero) {
+        return;
+    }
+    let tt = r_read_index_node(tree_table_pointer);
+    if (!tt.ok || tt.entry_count == 0u || tt.entry_width < DELETION_TREE_TABLE_ENTRY_BYTES) {
+        return;
+    }
+    let lineage_holds = d_read_the_lineage_of_a_version(tt);
+    // 每棵读得出的 inode 树的每条记录记进 I-9.29 那张表
+    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
+        let entry = entry_offset(tt, entry_index);
+        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) != TREE_KIND_INODE
+            || !d_read_inode_records(parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET))) {
+            continue;
+        }
+        let inode_tree = rd64(tt.base, entry);
+        for (var record_index = 0u; record_index < d_inode_record_count; record_index = record_index + 1u) {
+            let row = D_INODE_RECORDS + record_index * D_INODE_RECORD_WORDS;
+            d_note_object_birth(inode_tree, vec2<u32>(sw(row), sw(row + 1u)), d_inode_record_object_birth(row));
+        }
+    }
+    // 每棵墓碑树：读不全的不判；只要墓碑的那几判先判，再经头 ID 配恰好一棵 inode 树判墓碑对记录的那几判
+    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
+        let entry = entry_offset(tt, entry_index);
+        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) != TREE_KIND_TOMBSTONE
+            || !d_read_containers_of_a_container_tree(parse_node_pointer(tt.base, entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET),
+                PACKED_TYPE_TOMBSTONE, DELETION_TOMBSTONE_RECORD_BYTES)) {
+            continue;
+        }
+        d_judge_tombstone_container_capacity();
+        d_judge_every_tombstone_is_well_formed(version_txg);
+        if (lineage_holds) {
+            d_judge_tombstones_on_the_lineage_are_distinct(version_txg);
+        }
+        let head_identifier = rd64(tt.base, entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET);
+        var paired_inode_tree_count = 0u;
+        var paired_inode_tree_entry = 0u;
+        for (var other_index = 0u; other_index < tt.entry_count; other_index = other_index + 1u) {
+            let other = entry_offset(tt, other_index);
+            if (rd16(tt.base, other + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_INODE && eq64(rd64(tt.base, other), head_identifier)) {
+                paired_inode_tree_count = paired_inode_tree_count + 1u;
+                paired_inode_tree_entry = other;
+            }
+        }
+        if (paired_inode_tree_count != 1u
+            || !d_read_inode_records(parse_node_pointer(tt.base, paired_inode_tree_entry + DELETION_TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET))) {
+            continue;
+        }
+        d_judge_tombstones_against_the_records(head_identifier, version_txg, lineage_holds);
+    }
+    // I-9.18：每棵 inode 树
+    for (var entry_index = 0u; entry_index < tt.entry_count; entry_index = entry_index + 1u) {
+        let entry = entry_offset(tt, entry_index);
+        if (rd16(tt.base, entry + DELETION_TREE_TABLE_ENTRY_KIND_OFFSET) == TREE_KIND_INODE) {
+            d_judge_queued_inodes_have_tombstones_on_the_lineage(tt, entry, version_txg, lineage_holds);
+        }
+    }
+}
+
+// 墓碑与谱系那一遍：删除态扫描那一份版本清单（回退候选集里每条根，与由记录施加出来的那几版），每一版从自己的 txg 与自己的谱系树起算；
+// 全部扫完时 I-9.29 那张表已经判完（d_note_object_birth 见到第二个出生代当场判）
+fn judge_tombstones_and_the_lineage_of_the_versions() {
+    d_object_birth_count = 0u;
+    for (var candidate = 0u; candidate < c_candidate_count; candidate = candidate + 1u) {
+        let root = checker_root_of(sw(C_CANDIDATES + candidate));
+        d_judge_tombstones_and_the_lineage_of_a_version(root.base, 36u, root.txg);
+    }
+    for (var version = 0u; version < c_version_count; version = version + 1u) {
+        let record = checker_record_at(sw(R_DATA + version));
+        d_judge_tombstones_and_the_lineage_of_a_version(record.base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET, record.txg);
+    }
+}
+
 fn scan_stage(fsid_low: vec2<u32>) {
     judge_merged_version_total_order(fsid_low);
     judge_release_generation_and_tree_table_birth();
@@ -2533,6 +3071,7 @@ fn scan_stage(fsid_low: vec2<u32>) {
     judge_allocation_generations_against_unit_births();
     judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark();
     judge_deletion_state_of_the_versions();
+    judge_tombstones_and_the_lineage_of_the_versions();
     judge_inode_number_watermark_above_every_published_tombstone();
     judge_rollback_floor_raises_against_their_ceilings();
     judge_accounting(fsid_low);
diff --git a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
index ac511c8..7c550f3 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -106,6 +106,8 @@ const ROOT_RING_REGION_STRIDE: u32 = 3145728u;     // P × chunk = 3 × 1 MiB
 const ROOT_RING_BASE_BYTES: u32 = 1048576u;        // 64 × 16384
 const DATA_UNIT_PAYLOAD_CAPACITY: u32 = 32634u;
 const EXTENT_UPPER_LEAF_INODES: u32 = 143u;
+// extent 上段叶条目宽（key 24 + 标签 1 + 指针 88，D8（核心索引结构） 已定项 11 字段表；extent_tree.rs 的 EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES）
+const EXTENT_UPPER_LEAF_ENTRY_BYTES: u32 = 113u;
 const EXTENT_LOWER_LEAF_DATA_UNITS: u32 = 144u;
 const EXTENT_INTERNAL_FANOUT: u32 = 147u;
 const ALLOCATION_LEAF_SLOTS: u32 = 812u;
@@ -131,9 +133,12 @@ const CONTAINER_NUMBER_KEY_WIDTH: u32 = 8u;
 const CONTAINER_TREE_ENTRY_BYTES: u32 = 120u;
 const CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET: u32 = 34u;
 const FIRST_INODE_NUMBER: u32 = 1u;
-// inode 记录里 nlink 的偏移（4 字节，D8（核心索引结构） 已定项 6 字段表；恢复内核找第一个文件与扫描内核的删除态扫描都读这一份）
-// 与删除中的 inode 的 nlink（records.rs 的 NLINK_OF_AN_INODE_BEING_DELETED）
+// inode 记录里对象出生代（8 字节）、nlink（4 字节）、size（8 字节）的偏移（D8（核心索引结构） 已定项 6 字段表；恢复内核找第一个文件、
+// 走读内核判记录、扫描内核的删除态扫描与墓碑那一遍都读这一份，walk.rs 的 INODE_RECORD_OBJECT_BIRTH_OFFSET、INODE_RECORD_NLINK_OFFSET、
+// INODE_RECORD_SIZE_OFFSET）与删除中的 inode 的 nlink（records.rs 的 NLINK_OF_AN_INODE_BEING_DELETED）
+const INODE_RECORD_OBJECT_BIRTH_OFFSET: u32 = 8u;
 const INODE_RECORD_NLINK_OFFSET: u32 = 36u;
+const INODE_RECORD_SIZE_OFFSET: u32 = 40u;
 const NLINK_OF_AN_INODE_BEING_DELETED: u32 = 0u;
 const MAGIC_SFSB: u32 = 0x42534653u;
 const MAGIC_SFSR: u32 = 0x52534653u;
@@ -2369,7 +2374,9 @@ fn read_upper_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_of
             }
             nodes = nodes + 1u;
             if (level == 0u) {
-                if (n.entry_width < 113u) {
+                // 条目宽只在有条目时核：extent_tree.rs 的 upper_leaf_entries 逐条 ExtentUpperLeafEntry::parse 时才核宽，
+                // 零条目的上段根（第一个文件删完之后）条目宽是什么都不拒
+                if (n.entry_count > 0u && n.entry_width < EXTENT_UPPER_LEAF_ENTRY_BYTES) {
                     return out;
                 }
                 var previous_inode = vec2<u32>(0u, 0u);
@@ -3218,7 +3225,9 @@ fn walk_to_file(root: RootFields, cfg: SysConfig, expected: u32) -> WalkResult {
     if (inode_root.level != 1u) {
         return walk_failed(44u);
     }
-    if (inode_root.entry_width < 120u) {
+    // 条目宽只在有条目时核：recovery.rs 逐条 inode_internal_entry_parsed_and_judged 时才核宽，零条目的根（unlink 删掉树里最后一条记录之后）
+    // 条目宽是什么都不拒
+    if (inode_root.entry_count > 0u && inode_root.entry_width < CONTAINER_TREE_ENTRY_BYTES) {
         return walk_failed(45u);
     }
     var found_inode = false;
@@ -3273,9 +3282,9 @@ fn walk_to_file(root: RootFields, cfg: SysConfig, expected: u32) -> WalkResult {
             }
             if (eq64(inode, u64_of(FIRST_INODE_NUMBER))) {
                 found_inode = true;
-                inode_object_birth = rd64(leaf.base, ro + 8u);
+                inode_object_birth = rd64(leaf.base, ro + INODE_RECORD_OBJECT_BIRTH_OFFSET);
                 inode_nlink = rd32(leaf.base, ro + INODE_RECORD_NLINK_OFFSET);
-                inode_size = rd64(leaf.base, ro + 40u);
+                inode_size = rd64(leaf.base, ro + INODE_RECORD_SIZE_OFFSET);
             }
         }
     }
diff --git a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
index 78a60e2..7338e79 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -879,14 +879,14 @@ mod tests {
     #[test]
     fn states_in_one_dispatch_follow_the_scratch_memory_up_to_the_lane_limit() {
         let bytes_per_state = u64::try_from(SCRATCH_WORDS * 4).expect("装得进 u64");
-        assert_eq!(states_per_scratch_lane(BINDING_LIMIT), 1797);
+        assert_eq!(states_per_scratch_lane(BINDING_LIMIT), 1702);
         assert_eq!(states_per_scratch_lane(1), 1, "再小的上限也带一个");
-        // 12 GiB 的草稿区：12 × 1024 × 1024 × 1024 ÷ 1 194 944 = 10 782 个（向下取整）
+        // 12 GiB 的草稿区：12 × 1024 × 1024 × 1024 ÷ 1 261 504 = 10 213 个（向下取整）
         assert_eq!(
             states_per_dispatch(BINDING_LIMIT, 12 * 1024),
             usize::try_from(12 * 1024 * 1024 * 1024 / bytes_per_state).expect("装得进 usize")
         );
-        assert_eq!(states_per_dispatch(BINDING_LIMIT, 12 * 1024), 10_782);
+        assert_eq!(states_per_dispatch(BINDING_LIMIT, 12 * 1024), 10_213);
         assert_eq!(
             states_per_dispatch(BINDING_LIMIT, 0),
             1,
@@ -894,7 +894,7 @@ mod tests {
         );
         assert_eq!(
             states_per_dispatch(BINDING_LIMIT, 1024 * 1024),
-            1797 * MAXIMUM_SCRATCH_LANES,
+            1702 * MAXIMUM_SCRATCH_LANES,
             "显存再多也只到条数上限"
         );
     }
diff --git a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
index 2f3dcff..49ebf00 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -3,7 +3,8 @@
 //! 七类 oracle、记录核对器两条判据，判法与 CPU 判器（`crate::crash::judge_crash_image`）逐条相同；GPU 与 CPU 逐状态比是它的判别力自证。
 //! 内核分五份源码：`crash_judge_common.wgsl`（各内核共用的解析与查表）、`crash_judge_recovery.wgsl`（恢复、oracle、记录核对器）、
 //! `crash_judge_checker_common.wgsl`（池级 checker 两个内核共用的读法与交接段）、`crash_judge_checker.wgsl`（走读内核：第一、二批不变量，
-//! 连同里程碑三二·1 那一批里的十五条、I-9.27 与 I-3.5）、`crash_judge_checker_scan.wgsl`（扫描内核：第三批、I-3.12、I-5.5、I-9.28，与里程碑三·3 删除线的删除态扫描、I-9.6 墓碑那一半）；判不了的状态（表越过内核容量）置最高位，
+//! 连同里程碑三二·1 那一批里的十五条、I-9.27 与 I-3.5，与里程碑三·3 墓碑树、谱系树条目身份那两条 I-9.17、I-9.23）、`crash_judge_checker_scan.wgsl`
+//! （扫描内核：第三批、I-3.12、I-5.5、I-9.28，与里程碑三·3 删除线的删除态扫描、I-9.6 墓碑那一半、墓碑与谱系那一遍）；判不了的状态（表越过内核容量）置最高位，
 //! 放量路径（`crate::crash_amplification` 收卡上结论那一段）遇到它就让整趟停下、报哪张表满了，配置开了 GPU 不退回 CPU 判器。
 //!
 //! 一个状态的结论 [`VERDICT_WORDS`] 个 u32：第 0 字是红位（位 0..6 看 journal 那一遍 oracle 的七类、位 8..14 不看那一遍、
@@ -68,9 +69,12 @@ const _: () = assert!(
 /// 72 的 I-4.6 由扫描内核的删除态扫描在回退候选集的每一版上判（判哪几版、经头 ID 配对、读不全不置位，照 `singlefs_checker::walk` 的
 /// `judge_deletion_state_of_the_versions`），73 的 I-4.4 第一版池级 checker 恒报不适用、内核不置位；I-9.6 墓碑那一半（水位 > 所选根墓碑树里
 /// 已发布墓碑的最大对象 ID）也在扫描内核里判、并进下标 41 那一位。对拍逐位比、不屏蔽。
-/// 下标 74–82 是里程碑三·3 墓碑与谱系那九条（I-9.17、I-9.18、I-9.19、I-9.20、I-9.21、I-9.22、I-9.23、I-9.24、I-9.29，第 22 批）：
-/// 照下标 73 的先例先登记、内核这一批不置这九位，GPU 侧第 23 批补（走读内核判 I-9.17、I-9.23，扫描内核判其余七条）；
-/// 补上之前 CPU 判红而 GPU 不置位的状态在对拍里会对不上。
+/// 下标 74–82 是里程碑三·3 墓碑与谱系那九条（I-9.17、I-9.18、I-9.19、I-9.20、I-9.21、I-9.22、I-9.23、I-9.24、I-9.29，CPU 第 22 批、GPU 第 23 批）：
+/// 走读内核在墓碑树、谱系树的每条内部条目上判 I-9.17、I-9.23（身份不符置位、不记走读失败）；扫描内核的删除态扫描读队列时判 I-9.24，
+/// 墓碑与谱系那一遍在同一份版本清单上判 I-9.18、I-9.19、I-9.20、I-9.21（没装旋钮那一支）与 I-9.10 墓碑那一个等号（并进下标 43），
+/// I-9.29 收各版的号 → 对象出生代进一张跨版本的表判；一棵树的容器、一版的谱系记录、跨版本那张表、两两比不重号的墓碑总数越过上限的判不了。
+/// 下标 79 的 I-9.22 照下标 73 的先例恒不置位：它只在池级 checker 带旋钮值的入口上判，层 0 与放量走旧入口（`check_pool_image`，恒报不适用），
+/// GPU 也没有旋钮值。I-9.4 第四句（同一片 inode 叶容器里记录号严格递增）走读内核一并判、并进下标 40。
 /// 用户 2026-10-04 定「改回要求全覆盖」：没有「内核还没判」那一张表。
 pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
@@ -120,8 +124,9 @@ pub fn verdict_layout_wgsl_constants() -> String {
     )
 }
 /// 每个状态的草稿区几个 u32（checker 共用那一半里 `SCRATCH_END`：共用的 `SCRATCH_WORDS_REQUIRED`、走读与扫描的集合、两个内核的交接段、
-/// 交接段之后走读内核自己用的表：两张，加 I-3.5 的出现规则读的四张；再之后是扫描内核删除态扫描一版一版重填的两张：inode 记录与每个号的数据单元数）。
-pub const SCRATCH_WORDS: usize = 298_736;
+/// 交接段之后走读内核自己用的表：两张，加 I-3.5 的出现规则读的四张；再之后是扫描内核删除态扫描与墓碑、谱系那一遍用的五张：inode 记录、每个号的数据单元数、
+/// 一棵墓碑树或谱系树的容器、一版的谱系记录、I-9.29 跨版本的号 → 对象出生代）。
+pub const SCRATCH_WORDS: usize = 315_376;
 /// 内核的工作组大小（三个内核都是 `@workgroup_size(64)`）：派 n 个状态要 ⌈n ÷ 64⌉ 个工作组。
 pub const WORKGROUP_SIZE: usize = 64;
 /// 内核的四个绑定（`crash_judge_common.wgsl` 开头）：派活参数、打包的表、结论、草稿区。
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index d73e0e0..63b584d 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -14,6 +14,10 @@
 //! 那一版起 extent 树只剩上段零条目根，同样整份比（两边都答没有文件）；删除中间态那条流上展开上限
 //! 不展开的段（新建文件、顺序写、意图、第一批那几次发布的单元写）另把「之前的段全落、这一段只差一个写」的每个状态各比一次。
 //! I-9.16 ③ ④（种类重复、缺 deadlist）、根槽 428..457（I-2.4）、带树一版根记录里的分配记录树根（I-5.4）各一条坏流整份比；指针头部的
+//! 墓碑与谱系那九条（第 23 批）：I-9.17、I-9.23 各一条改坏容器树根条目身份或类型段的流，I-9.18、I-9.19、I-9.20（两句各一条）、I-9.29
+//! （连同 I-9.10 墓碑那一个等号）各一条改坏墓碑或记录的流，I-9.24 并在队列 {1, 7} 那一条上，I-9.21 一条墓碑容器旋钮压到一片 1 条的流
+//! （旋钮下合法、CPU 判器按旧入口判就红），I-9.4 第四句一条叶里记录对调的流，删 → 回退 → 再删一条合法流钉九条不红；零条目的 extent 上段根
+//! 与 inode 树根条目宽窄于字段表各一条，恢复两边都答没有文件。I-9.22 层 0 恒不适用，没有流。
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
 //! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）；这几处除实例表指针外
 //! 另各有一条 MAC 第 0 字节非 0 的坏流（core 连加密、压缩那几段一并判，GPU 恢复内核同判）；extent 上段叶内联的数据指针另有一条算法类型写 1 的坏流
@@ -39,7 +43,8 @@ use common::{build_pool, file_content, geometry, parameters, BuiltPool};
 use common_crash_points::{mount_writable_step, overwrite_step, record_pool_build, roll_back_step};
 use common_deletion_in_progress::{
     content_of_three_data_units, pool_with_a_three_unit_file, pool_with_new_inodes,
-    publish_the_intent, publish_the_next_step_in_batches_of_one, FIRST_FILE,
+    publish_the_intent, publish_the_next_step_in_batches_of_one, roll_back_to, unlink_to_the_end,
+    FIRST_FILE, INSTANCE_OF_THE_PUBLISHES, WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
 };
 use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
 use singlefs_checker_tier::crash::{
@@ -65,8 +70,17 @@ use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
 };
 use singlefs_checker_tier::gpu_unit_checks::GpuCard;
 use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
-use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::address::{CheckpointTxg, InodeNumber, InstanceGeneration};
+use singlefs_core::container_tree::{ContainerTreeCapacities, ContainerTreeNodeEntries};
+use singlefs_core::lineage_tree::LineageContainerCapacity;
 use singlefs_core::mount::RollbackTarget;
+use singlefs_core::tombstone_tree::{
+    TombstoneContainerCapacity, TombstoneRecordsPerContainerCappedForTests,
+};
+use singlefs_core::transaction::{
+    publish_unlink_intent, publish_unlink_steps_until_the_queue_is_empty, PoolWriter,
+};
+use singlefs_core::unlink::UnlinkBatchSize;
 use singlefs_harness::memory_pool::{
     writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
     WrittenContents,
@@ -5453,6 +5467,7 @@ fn the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unex
     );
 }
 
+/// 队列 {1} → {1, 7}：7 号没有记录（I-9.5 正方向红），队列里两个不同的号（I-9.24 红，读队列在配对之前判）。
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record() {
     let flow = record_deletion_with_a_resealed_field(
@@ -5461,10 +5476,10 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_with
         &is_a_deleted_inodes_root,
         &put_an_inode_number_without_a_record_into_the_deletion_queue_root,
     );
-    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
         "the_gpu_judge_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record",
         &flow,
-        "I-9.5",
+        &["I-9.5", "I-9.24"],
     );
 }
 
@@ -5558,3 +5573,559 @@ fn the_gpu_judge_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inod
         "I-9.6",
     );
 }
+
+// ───────────────────────── 里程碑三·3 墓碑与谱系那九条（第 23 批）与第 21 批留下的两处坏盘分叉 ─────────────────────────
+
+/// 码 2 节点条目里身份引用的出生树（分隔 key 8 之后）与容器号（类型段 2 之后）。
+const CONTAINER_TREE_ENTRY_BIRTH_TREE_OFFSET: usize = 8;
+const CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET: usize = 18;
+/// 墓碑树、谱系树条目类型段条款之外的一个值（墓碑 {0, 1}、谱系 {0, 5} 之外）。
+const UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE: u16 = 7;
+/// 墓碑记录 40 字节：对象 ID 0、对象出生代 8、死亡 txg 32（D18（块里携带什么信息） 已定项 24 第 2 条）。
+const TOMBSTONE_RECORD_BYTES: usize = 40;
+const TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;
+const TOMBSTONE_RECORD_DEATH_TXG_OFFSET: usize = 32;
+/// 改坏墓碑的对象 ID 时写的号：inode 树里没有这个号的记录，也不越过 inode 号水位（I-9.6 墓碑那一半照旧成立）。
+const OBJECT_IDENTIFIER_WITHOUT_A_RECORD: u64 = 0;
+/// I-9.21 那条流里墓碑容器旋钮压到的每片记录数：两次 unlink 各占一片，最左那片不是最右的、只装 1 条。
+const TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB: u64 = 1;
+
+/// 墓碑树、谱系树、inode 树的码 2 节点里第一条条目的起点（key 是 8 字节的容器号或 inode 号）。
+fn first_entry_of_a_container_number_node() -> usize {
+    index_node_entries_start(CONTAINER_NUMBER_KEY_BYTES)
+}
+
+fn is_the_tombstone_tree_root(bytes: &[u8]) -> bool {
+    is_a_container_tree_root_over_containers_of_record_type(bytes, PACKED_RECORD_TYPE_TOMBSTONE)
+}
+
+fn is_the_lineage_tree_root(bytes: &[u8]) -> bool {
+    is_a_container_tree_root_over_containers_of_record_type(bytes, PACKED_RECORD_TYPE_LINEAGE)
+}
+
+/// 码 2 节点头里的条目数。
+fn index_node_entry_count(bytes: &[u8]) -> u16 {
+    read_u16_at(
+        bytes,
+        index_node_entry_count_offset(usize::from(index_node_key_width(bytes))),
+    )
+}
+
+/// 码 2 节点头里的条目宽改成 `entry_width`（条目数 0：声明长度 0 = 0 × 条目宽照旧成立）。
+fn set_the_entry_width_of_an_empty_index_node(bytes: &mut [u8], entry_width: u16) {
+    assert_eq!(index_node_entry_count(bytes), 0, "零条目的码 2 节点");
+    let entry_width_offset =
+        index_node_entry_count_offset(usize::from(index_node_key_width(bytes))) + 2;
+    bytes[entry_width_offset..entry_width_offset + 2].copy_from_slice(&entry_width.to_le_bytes());
+}
+
+/// 容器树根第一条条目里身份引用的容器号加 1（分隔 key 与子指针不动）。
+fn add_one_to_the_container_number_of_the_first_entry(bytes: &mut [u8]) {
+    let field =
+        first_entry_of_a_container_number_node() + CONTAINER_TREE_ENTRY_CONTAINER_NUMBER_OFFSET;
+    let container = read_u64_at(bytes, field);
+    bytes[field..field + 8].copy_from_slice(&(container + 1).to_le_bytes());
+}
+
+/// 容器树根第一条条目的类型段写成 `entry_type`。
+fn set_the_type_of_the_first_entry(bytes: &mut [u8], entry_type: u16) {
+    let field = first_entry_of_a_container_number_node() + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET;
+    bytes[field..field + 2].copy_from_slice(&entry_type.to_le_bytes());
+}
+
+/// 墓碑容器第一条墓碑偏移 `field` 起的 8 字节改成 `change` 给的值。
+fn change_the_first_tombstone_field(bytes: &mut [u8], field: usize, change: impl Fn(u64) -> u64) {
+    assert!(
+        read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET) >= 1,
+        "墓碑容器里至少一条墓碑"
+    );
+    let offset = PACKED_UNIT_RECORDS_START + field;
+    let value = change(read_u64_at(bytes, offset));
+    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
+}
+
+/// 同 [`assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening`]，CPU 池级 checker 要红的是 `invariants` 里的每一条。
+fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+    test_name: &str,
+    flow: &RecordedFlow,
+    invariants: &[&'static str],
+) {
+    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, flow, expand_up_to());
+    for invariant in invariants {
+        assert!(
+            comparison.cpu_checker_red_invariants.contains(invariant),
+            "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
+            flow.name,
+            comparison.cpu_checker_red_invariants
+        );
+    }
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
+    );
+}
+
+/// I-9.17 容器那一支：固定脚本到回退 D 的流上谱系树根第一条条目（类型段 5、指谱系容器）身份引用里的容器号加 1、重封到根。
+/// 容器头没动，条目身份与子头不符：CPU 池级 checker 判 I-9.17 红、不记走读失败（第 22 批），GPU 走读内核同判（在查这片容器走没走过之前判）。
+/// 恢复读谱系树根时层级 1 的条目只数不读，两边恢复照旧。
+fn record_rollback_with_the_lineage_root_entry_naming_another_container(tag: &str) -> RecordedFlow {
+    record_rollback_with_a_resealed_field(
+        tag,
+        "rollback-lineage-root-entry-names-another-container",
+        &is_the_lineage_tree_root,
+        &add_one_to_the_container_number_of_the_first_entry,
+    )
+}
+
+/// I-9.23 类型段不认得那一支：删除中间态的流上墓碑树根第一条条目的类型段 1 → 7、重封到根。CPU 先登记这条子指针的引用（层级 1 按容器两槽）
+/// 再判 I-9.23 红、不往下走、不记走读失败；GPU 走读内核同样先登记引用再置位。删除态扫描读这棵墓碑树判不了，I-9.18 起那几条不判。
+fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type(
+    tag: &str,
+) -> RecordedFlow {
+    record_deletion_with_a_resealed_field(
+        tag,
+        "deletion-tombstone-root-entry-type-seven",
+        &is_the_tombstone_tree_root,
+        &|bytes| set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE),
+    )
+}
+
+/// I-9.23 类型段 0 那一支：删除中间态的流上墓碑树根（层级 1）第一条条目的类型段 1 → 0、重封到根。身份引用的出生树不是 0、孩子是码 3：
+/// CPU 判 I-9.23 红，再因为层级 1 的节点里出现类型段 0 记走读失败；GPU 走读内核同判同记。
+fn record_deletion_with_the_tombstone_root_entry_of_type_zero(tag: &str) -> RecordedFlow {
+    record_deletion_with_a_resealed_field(
+        tag,
+        "deletion-tombstone-root-entry-type-zero",
+        &is_the_tombstone_tree_root,
+        &|bytes| {
+            assert_ne!(
+                read_u64_at(
+                    bytes,
+                    first_entry_of_a_container_number_node()
+                        + CONTAINER_TREE_ENTRY_BIRTH_TREE_OFFSET
+                ),
+                0,
+                "指墓碑容器的条目身份引用里出生树不是 0"
+            );
+            set_the_type_of_the_first_entry(bytes, 0);
+        },
+    )
+}
+
+/// I-9.18：删除中间态的流上墓碑容器里那一条墓碑的对象 ID 1 → 0、重封到根。队列 {1} 里的号 1 在墓碑树里没有墓碑：CPU 判 I-9.18 红；
+/// 号 0 没有记录，I-9.10 墓碑那一个等号与 I-9.19 没有对象，号 0 不越过水位，I-9.6 墓碑那一半照旧成立。
+fn record_deletion_with_the_tombstone_naming_an_object_without_a_record(tag: &str) -> RecordedFlow {
+    record_deletion_with_a_resealed_field(
+        tag,
+        "deletion-tombstone-names-an-object-without-a-record",
+        &is_a_tombstone_container,
+        &|bytes| {
+            change_the_first_tombstone_field(bytes, 0, |_object| OBJECT_IDENTIFIER_WITHOUT_A_RECORD)
+        },
+    )
+}
+
+/// I-9.19：删除中间态的流上第一批那一版 inode 叶容器里第一个文件的记录 nlink 0 → 1、重封到根。意图那一版写的墓碑在第一批那一版的谱系上、
+/// 指着号与对象出生代都相同而 nlink 1 的记录：CPU 判 I-9.19 红（队列里的号 nlink ≠ 0，I-9.5 同红）。
+fn record_deletion_with_the_queued_record_back_at_nlink_one(tag: &str) -> RecordedFlow {
+    record_deletion_with_a_resealed_field(
+        tag,
+        "deletion-queued-record-back-at-nlink-one",
+        &is_an_inode_leaf_container,
+        &|bytes| {
+            let record = offset_of_the_inode_record(bytes, FIRST_FILE.0);
+            assert_eq!(
+                read_u32_at(bytes, record + INODE_RECORD_NLINK_OFFSET),
+                0,
+                "删除中的第一个文件 nlink 0"
+            );
+            bytes[record + INODE_RECORD_NLINK_OFFSET..record + INODE_RECORD_NLINK_OFFSET + 4]
+                .copy_from_slice(&1u32.to_le_bytes());
+        },
+    )
+}
+
+/// I-9.20 管全部墓碑那几句：删除中间态的流上墓碑的死亡 txg 改成 0、重封到根。对象出生代 > 死亡 txg：CPU 判 I-9.20 红；
+/// 死亡 txg 0 仍在谱系上（没有回退的一版从 0 到自己的 txg 整段在链上），I-9.18 照旧成立。
+fn record_deletion_with_the_tombstone_dying_at_txg_zero(tag: &str) -> RecordedFlow {
+    record_deletion_with_a_resealed_field(
+        tag,
+        "deletion-tombstone-dies-at-txg-zero",
+        &is_a_tombstone_container,
+        &|bytes| {
+            change_the_first_tombstone_field(
+                bytes,
+                TOMBSTONE_RECORD_DEATH_TXG_OFFSET,
+                |_death_txg| 0,
+            )
+        },
+    )
+}
+
+/// I-9.20 不重号那一句：删除中间态的流上墓碑容器里那一条墓碑原样再追加一条（记录数加一、声明长度加 40）、重封到根。
+/// 两条都在谱系上、(号, 对象出生代) 相同：CPU 判 I-9.20 红。
+fn record_deletion_with_the_tombstone_written_twice(tag: &str) -> RecordedFlow {
+    record_deletion_with_a_resealed_field(
+        tag,
+        "deletion-tombstone-written-twice",
+        &is_a_tombstone_container,
+        &|bytes| {
+            let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
+            assert_eq!(record_count, 1, "意图那一次只写一条墓碑");
+            let first = PACKED_UNIT_RECORDS_START;
+            bytes.copy_within(
+                first..first + TOMBSTONE_RECORD_BYTES,
+                first + TOMBSTONE_RECORD_BYTES,
+            );
+            bytes[PACKED_UNIT_RECORD_COUNT_OFFSET..PACKED_UNIT_RECORD_COUNT_OFFSET + 2]
+                .copy_from_slice(&2u16.to_le_bytes());
+            let declared_length = read_u16_at(bytes, UNIT_DECLARED_LENGTH_OFFSET);
+            bytes[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
+                &(declared_length + u16::try_from(TOMBSTONE_RECORD_BYTES).expect("40"))
+                    .to_le_bytes(),
+            );
+        },
+    )
+}
+
+/// I-9.29 与 I-9.10 墓碑那一个等号：删除中间态的流上墓碑的对象出生代加 1、重封到根。号 1 的记录出生代与谱系上墓碑的不同：
+/// CPU 判 I-9.10（按号认）、I-9.29（同号两个出生代）红，I-9.18 跟着红（没有出生代对得上的墓碑）。
+fn record_deletion_with_the_tombstone_object_birth_off_its_record(tag: &str) -> RecordedFlow {
+    record_deletion_with_a_resealed_field(
+        tag,
+        "deletion-tombstone-object-birth-off-its-record",
+        &is_a_tombstone_container,
+        &|bytes| {
+            change_the_first_tombstone_field(
+                bytes,
+                TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET,
+                |object_birth| object_birth + 1,
+            )
+        },
+    )
+}
+
+/// I-9.4 第四句：新池新建文件、再建一个空 inode（2 号，与第一个文件同在最左那片叶里）的流上那片叶的两条记录对调、重封到根。
+/// 记录序 [2, 1] 不严格递增：CPU 判 I-9.4 红（号都 ≥ 容器号、叶序上的第二、三句照旧成立）；恢复按号找第一个文件，不看次序。
+fn record_new_pool_with_the_two_records_of_the_inode_leaf_swapped(tag: &str) -> RecordedFlow {
+    record_new_pool_with_a_new_inode_and_its_inode_leaf_resealed(
+        tag,
+        "new-pool-new-inode-leaf-records-swapped",
+        &|bytes| {
+            let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
+            assert_eq!(record_count, 2, "第一个文件与空 inode 两条");
+            let first = offset_of_the_inode_record(bytes, FIRST_FILE.0);
+            let second = offset_of_the_inode_record(bytes, FIRST_FILE.0 + 1);
+            assert!(first < second, "写者按号升序写记录");
+            let first_record = bytes[first..first + INODE_RECORD_BYTES].to_vec();
+            bytes.copy_within(second..second + INODE_RECORD_BYTES, first);
+            bytes[second..second + INODE_RECORD_BYTES].copy_from_slice(&first_record);
+        },
+    )
+}
+
+/// 删到最后一批的流上最后一个零条目的码 2 节点（key 宽 `key_width`）条目宽改成 `entry_width`、重封到根：第 21 批 G 留下的坏盘分叉。
+fn record_deletion_through_the_last_batch_with_an_empty_root_of_another_entry_width(
+    tag: &str,
+    name: &'static str,
+    key_width: u8,
+    entry_width: u16,
+) -> RecordedFlow {
+    let mut flow = record_new_pool_through_the_last_deletion_batch(tag, name);
+    let empty_root = move |bytes: &[u8]| {
+        unit_class(bytes) == 2
+            && index_node_key_width(bytes) == key_width
+            && index_node_entry_count(bytes) == 0
+    };
+    let targets = last_unit_copies(&flow, name, &empty_root, false);
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
+        set_the_entry_width_of_an_empty_index_node(bytes, entry_width);
+    });
+    flow
+}
+
+/// 零条目、层级 0 的 extent 上段根，条目宽 113 → 24（= key 宽）。core 冷走读逐条解条目时才核宽（`extent_tree.rs` 的 `upper_leaf_entries`），
+/// 零条目时答没有文件；GPU 恢复内核同判（`read_upper_segment` 只在有条目时核宽）。池级 checker 两边都按字段表判 I-1.10 红。
+fn record_deletion_through_the_last_batch_with_a_narrow_empty_extent_root(
+    tag: &str,
+) -> RecordedFlow {
+    const EXTENT_KEY_WIDTH: u8 = 24;
+    record_deletion_through_the_last_batch_with_an_empty_root_of_another_entry_width(
+        tag,
+        "deletion-through-the-last-batch-empty-extent-root-of-entry-width-twenty-four",
+        EXTENT_KEY_WIDTH,
+        u16::from(EXTENT_KEY_WIDTH),
+    )
+}
+
+/// 零条目的 inode 树根（层级 1），条目宽 120 → 8（= key 宽）。core 冷走读逐条 `inode_internal_entry_parsed_and_judged` 时才核宽，
+/// 零条目时答没有文件；GPU 恢复内核同判（`walk_to_file` 只在有条目时核宽）。池级 checker 两边都按字段表判 I-1.10 红。
+fn record_deletion_through_the_last_batch_with_a_narrow_empty_inode_root(
+    tag: &str,
+) -> RecordedFlow {
+    let inode_key_width = u8::try_from(INODE_TREE_KEY_BYTES).expect("8");
+    record_deletion_through_the_last_batch_with_an_empty_root_of_another_entry_width(
+        tag,
+        "deletion-through-the-last-batch-empty-inode-root-of-entry-width-eight",
+        inode_key_width,
+        u16::from(inode_key_width),
+    )
+}
+
+/// 从新池新建文件（txg 3）到 `pool` 现行那一版，每一版都读得出第一个文件（这几条流不碰它）。
+fn versions_with_the_first_file_through_the_last_publish(
+    pool: &BuiltPool,
+) -> Vec<PublishedVersion> {
+    (3..=pool.output.root.checkpoint_txg.0)
+        .map(|txg| PublishedVersion {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(txg),
+            content: file_content(),
+        })
+        .collect()
+}
+
+/// I-9.21：新池新建文件、建两个空 inode（2、3 号），写入口装上只供测试的墓碑容器旋钮（一片 1 条），unlink 2、3 号各到底：
+/// 墓碑树根两条条目各指一片容器、各装 1 条。这是旋钮下的合法镜像；CPU 判器走池级 checker 的旧入口（当没装旋钮），
+/// 不是最右的那片只装 1 条、不是 815，判 I-9.21 红；GPU 扫描内核只有没装旋钮那一支，同判。
+fn record_two_empty_inodes_unlinked_with_one_tombstone_per_container(tag: &str) -> RecordedFlow {
+    let mut pool = pool_with_new_inodes(tag, 2);
+    let capacities = ContainerTreeCapacities {
+        tombstone_container: TombstoneContainerCapacity::CappedForTests(
+            TombstoneRecordsPerContainerCappedForTests::new(
+                TOMBSTONE_RECORDS_PER_CONTAINER_UNDER_THE_KNOB,
+            )
+            .expect("1..=815"),
+        ),
+        lineage_container: LineageContainerCapacity::FromTheContainerFormat,
+        node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
+    };
+    for inode in [FIRST_FILE.0 + 1, FIRST_FILE.0 + 2] {
+        let publish_parameters = parameters();
+        let devices = pool.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        writer.set_container_tree_capacities(capacities);
+        let mut current = publish_unlink_intent(
+            &mut writer,
+            &mut pool.allocator,
+            &pool.output,
+            InodeNumber(inode),
+            WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
+            INSTANCE_OF_THE_PUBLISHES,
+        )
+        .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的意图：{refusal:?}"));
+        publish_unlink_steps_until_the_queue_is_empty(
+            &mut writer,
+            &mut pool.allocator,
+            &mut current,
+            UnlinkBatchSize::OneExtentLowerLeaf,
+            INSTANCE_OF_THE_PUBLISHES,
+        )
+        .unwrap_or_else(|refusal| panic!("unlink inode {inode} 的各批：{refusal:?}"));
+        pool.output = current;
+    }
+    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
+    recorded_flow_of_the_pool(
+        &pool,
+        "two-empty-inodes-unlinked-one-tombstone-per-container",
+        versions,
+    )
+}
+
+/// 删 → 回退 → 再删的合法历史（第 22 批 C 报告的合法历史之一）：新池新建文件、建空 inode 2（txg 4），把它 unlink 到底，挂着回退到 txg 4
+/// （谱系 (回退那次的 txg, 4)；墓碑树随现行那一版，头一条墓碑的死亡 txg 落在被回退抛弃的那一段里、不在谱系上），再把它 unlink 到底
+/// （又一条 (2, 同一个对象出生代) 的墓碑，在谱系上）。不按谱系滤的写法在这里判 I-9.20 不重号红。
+fn record_new_inode_unlinked_rolled_back_and_unlinked_again(tag: &str) -> RecordedFlow {
+    let mut pool = pool_with_new_inodes(tag, 1);
+    let before_the_intent = pool.output.clone();
+    unlink_to_the_end(&mut pool, FIRST_FILE.0 + 1);
+    roll_back_to(&mut pool, &before_the_intent);
+    unlink_to_the_end(&mut pool, FIRST_FILE.0 + 1);
+    let versions = versions_with_the_first_file_through_the_last_publish(&pool);
+    recorded_flow_of_the_pool(
+        &pool,
+        "new-inode-unlinked-rolled-back-and-unlinked-again",
+        versions,
+    )
+}
+
+/// 墓碑与谱系那九条（I-9.24 在删除态扫描里判；I-9.22 层 0 恒不适用）。
+const TOMBSTONE_AND_LINEAGE_INVARIANTS: [&str; 9] = [
+    "I-9.17", "I-9.18", "I-9.19", "I-9.20", "I-9.21", "I-9.22", "I-9.23", "I-9.24", "I-9.29",
+];
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_lineage_root_entry_names_another_container_than_its_child(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_lineage_root_entry_names_another_container_than_its_child",
+        &record_rollback_with_the_lineage_root_entry_naming_another_container(
+            "crash-judge-gpu-i917-lineage-entry-container",
+        ),
+        "I-9.17",
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_carries_an_unregistered_type() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_carries_an_unregistered_type",
+        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type(
+            "crash-judge-gpu-i923-tombstone-entry-type-seven",
+        ),
+        "I-9.23",
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_carries_an_identity()
+{
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_carries_an_identity",
+        &record_deletion_with_the_tombstone_root_entry_of_type_zero(
+            "crash-judge-gpu-i923-tombstone-entry-type-zero",
+        ),
+        "I-9.23",
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_queued_inode_has_no_tombstone_on_the_lineage() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_queued_inode_has_no_tombstone_on_the_lineage",
+        &record_deletion_with_the_tombstone_naming_an_object_without_a_record(
+            "crash-judge-gpu-i918-tombstone-object",
+        ),
+        "I-9.18",
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_names_a_live_object() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_names_a_live_object",
+        &record_deletion_with_the_queued_record_back_at_nlink_one("crash-judge-gpu-i919-nlink-one"),
+        "I-9.19",
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_dies_before_its_object_is_born() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_dies_before_its_object_is_born",
+        &record_deletion_with_the_tombstone_dying_at_txg_zero(
+            "crash-judge-gpu-i920-death-txg-zero",
+        ),
+        "I-9.20",
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_is_written_twice() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_is_written_twice",
+        &record_deletion_with_the_tombstone_written_twice("crash-judge-gpu-i920-tombstone-twice"),
+        "I-9.20",
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_names_another_object_birth_than_the_record() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_names_another_object_birth_than_the_record",
+        &record_deletion_with_the_tombstone_object_birth_off_its_record(
+            "crash-judge-gpu-i929-tombstone-object-birth",
+        ),
+        &["I-9.29", "I-9.10"],
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_non_rightmost_tombstone_container_is_not_full() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_non_rightmost_tombstone_container_is_not_full",
+        &record_two_empty_inodes_unlinked_with_one_tombstone_per_container(
+            "crash-judge-gpu-i921-one-tombstone-per-container",
+        ),
+        "I-9.21",
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_records_of_an_inode_leaf_are_out_of_order() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_records_of_an_inode_leaf_are_out_of_order",
+        &record_new_pool_with_the_two_records_of_the_inode_leaf_swapped(
+            "crash-judge-gpu-i94-records-swapped",
+        ),
+        "I-9.4",
+    );
+}
+
+/// 恢复那一半整份比（看 journal 与不看那一遍落到的根与结局）：零条目的 extent 上段根条目宽窄于字段表，两边都答没有文件。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_an_empty_extent_root_narrower_than_its_field_table_holds_no_file(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_that_an_empty_extent_root_narrower_than_its_field_table_holds_no_file",
+        &record_deletion_through_the_last_batch_with_a_narrow_empty_extent_root(
+            "crash-judge-gpu-empty-extent-root-narrow",
+        ),
+        "I-1.10",
+    );
+}
+
+/// 同上，零条目的 inode 树根。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_an_empty_inode_root_narrower_than_its_field_table_holds_no_file(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_that_an_empty_inode_root_narrower_than_its_field_table_holds_no_file",
+        &record_deletion_through_the_last_batch_with_a_narrow_empty_inode_root(
+            "crash-judge-gpu-empty-inode-root-narrow",
+        ),
+        "I-1.10",
+    );
+}
+
+/// 删 → 回退 → 再删的每个状态上 GPU 与 CPU 整份相同；CPU 池级 checker 在这条流上判过 I-9.18、I-9.19、I-9.20、I-9.24、I-9.29 而成立，
+/// 墓碑与谱系那九条一个状态都不红（谱系外那条旧墓碑不进 I-9.20 不重号、不进 I-9.29 的表）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_an_unlink_rolled_back_and_unlinked_again() {
+    let flow = record_new_inode_unlinked_rolled_back_and_unlinked_again(
+        "crash-judge-gpu-unlink-rolled-back-and-unlinked-again",
+    );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_an_unlink_rolled_back_and_unlinked_again",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
+    );
+    for invariant in ["I-9.18", "I-9.19", "I-9.20", "I-9.24", "I-9.29"] {
+        assert!(
+            comparison.cpu_checker_held_invariants.contains(invariant),
+            "删 → 回退 → 再删的流上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
+            comparison.cpu_checker_held_invariants
+        );
+    }
+    for invariant in TOMBSTONE_AND_LINEAGE_INVARIANTS {
+        assert!(
+            !comparison.cpu_checker_red_invariants.contains(invariant),
+            "删 → 回退 → 再删的流上 CPU 池级 checker 不红 {invariant}（红过的：{:?}）",
+            comparison.cpu_checker_red_invariants
+        );
+    }
+}
````

## 实现员 g：mutations-append.tsv（/tmp/claude-1000/m3-batch23/g/patch/mutations-append.tsv）

````text
第 23 批 GPU 判器：墓碑树、谱系树指容器的条目身份不判（谱系树根条目容器号加一那条流上 CPU 红 I-9.17、GPU 不置位）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        w_judge(identity_invariant, identity_holds);	        w_judge(identity_invariant, true);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_lineage_root_entry_names_another_container_than_its_child	the_gpu_judge_agrees_with_the_cpu_when_a_lineage_root_entry_names_another_container_than_its_child
第 23 批 GPU 判器：墓碑树、谱系树条目类型段不认得不判（墓碑树根条目类型段 7 那条流上 CPU 红 I-9.23、GPU 不置位）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            w_judge(identity_invariant, false);	            w_judge(identity_invariant, true);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_carries_an_unregistered_type	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_carries_an_unregistered_type
第 23 批 GPU 判器：类型段不认得的条目不登记引用（墓碑树根条目类型段 7 那条流上 CPU 把那片容器算进引用并集、GPU 不算，I-3.1 一侧对不上）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            w_note_pointer_references(child, span_of_the_unregistered_child);	            _ = span_of_the_unregistered_child;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_carries_an_unregistered_type	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_carries_an_unregistered_type
第 23 批 GPU 判器：类型段 0 的条目身份不判（层级 1 的墓碑树根条目类型段写 0 那条流上 CPU 红 I-9.23、GPU 不置位）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            w_judge(identity_invariant, is_zero64(rd64(n.base, o + 8u))	            w_judge(identity_invariant, true || is_zero64(rd64(n.base, o + 8u))	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_carries_an_identity	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_carries_an_identity
第 23 批 GPU 判器：I-9.4 第四句不判（inode 叶里两条记录对调那条流上 CPU 红 I-9.4、GPU 不置位）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                    w_judge(INVARIANT_I_9_4, lt64(rd64(p.base, packed_record_offset(p, r - 1u)), inode));	                    w_judge(INVARIANT_I_9_4, true || lt64(rd64(p.base, packed_record_offset(p, r - 1u)), inode));	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_records_of_an_inode_leaf_are_out_of_order	the_gpu_judge_agrees_with_the_cpu_when_the_records_of_an_inode_leaf_are_out_of_order
第 23 批 GPU 判器：零条目的 inode 树根照样按条目宽拒（条目宽 8 的零条目根那条流上 CPU 答没有文件、GPU 走读失败）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    if (inode_root.entry_count > 0u && inode_root.entry_width < CONTAINER_TREE_ENTRY_BYTES) {	    if (inode_root.entry_width < CONTAINER_TREE_ENTRY_BYTES) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_an_empty_inode_root_narrower_than_its_field_table_holds_no_file	the_gpu_judge_agrees_with_the_cpu_that_an_empty_inode_root_narrower_than_its_field_table_holds_no_file
第 23 批 GPU 判器：零条目的 extent 上段根照样按条目宽拒（条目宽 24 的零条目根那条流上 CPU 答没有文件、GPU 走读失败）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	                if (n.entry_count > 0u && n.entry_width < EXTENT_UPPER_LEAF_ENTRY_BYTES) {	                if (n.entry_width < EXTENT_UPPER_LEAF_ENTRY_BYTES) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_an_empty_extent_root_narrower_than_its_field_table_holds_no_file	the_gpu_judge_agrees_with_the_cpu_that_an_empty_extent_root_narrower_than_its_field_table_holds_no_file
第 23 批 GPU 判器：I-9.18 不判（墓碑对象 ID 改成 0 那条流上 CPU 红 I-9.18、GPU 不置位）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	            violate(INVARIANT_I_9_18);	            _ = INVARIANT_I_9_18;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_queued_inode_has_no_tombstone_on_the_lineage	the_gpu_judge_agrees_with_the_cpu_when_a_queued_inode_has_no_tombstone_on_the_lineage
第 23 批 GPU 判器：I-9.19 不判（队列里的号记录 nlink 改回 1 那条流上 CPU 红 I-9.19、GPU 不置位）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	                violate(INVARIANT_I_9_19);	                _ = INVARIANT_I_9_19;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_names_a_live_object	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_names_a_live_object
第 23 批 GPU 判器：I-9.20 不判对象出生代 ≤ 死亡 txg（墓碑死亡 txg 改成 0 那条流上 CPU 红 I-9.20、GPU 不置位）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	            if (!le64(rd64(base, o + DELETION_TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET), death_txg) || !le64(death_txg, version_txg)) {	            if (!le64(death_txg, version_txg)) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_dies_before_its_object_is_born	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_dies_before_its_object_is_born
第 23 批 GPU 判器：I-9.20 不判谱系上的墓碑不重号（同一条墓碑写两遍那条流上 CPU 红 I-9.20、GPU 不置位）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	                        && d_txg_is_on_the_lineage(rd64(earlier_base, earlier + DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET), version_txg)) {\n                        violate(INVARIANT_I_9_20);	                        && d_txg_is_on_the_lineage(rd64(earlier_base, earlier + DELETION_TOMBSTONE_RECORD_DEATH_TXG_OFFSET), version_txg)) {\n                        _ = INVARIANT_I_9_20;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_is_written_twice	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_on_the_lineage_is_written_twice
第 23 批 GPU 判器：I-9.21 不判非最右的墓碑容器装满（旋钮压到一片 1 条的两次 unlink 那条流上 CPU 按旧入口红 I-9.21、GPU 不置位）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (d_container_record_count(container_index) != DELETION_TOMBSTONE_CONTAINER_RECORDS) {	        if (false && d_container_record_count(container_index) != DELETION_TOMBSTONE_CONTAINER_RECORDS) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_non_rightmost_tombstone_container_is_not_full	the_gpu_judge_agrees_with_the_cpu_when_a_non_rightmost_tombstone_container_is_not_full
第 23 批 GPU 判器：I-9.24 不判（队列 {1, 7} 那条流上 CPU 红 I-9.24、GPU 不置位）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    if (queue_read.readable && !d_deletion_queue_holds_at_most_one_key(queue_read)) {	    if (false && queue_read.readable && !d_deletion_queue_holds_at_most_one_key(queue_read)) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record	the_gpu_judge_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record
第 23 批 GPU 判器：I-9.29 不判（墓碑对象出生代加一那条流上 CPU 红 I-9.29、GPU 不置位）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	                violate(INVARIANT_I_9_29);	                _ = INVARIANT_I_9_29;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_names_another_object_birth_than_the_record	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_names_another_object_birth_than_the_record
第 23 批 GPU 判器：I-9.10 墓碑那一个等号不判（墓碑对象出生代加一那条流上 CPU 红 I-9.10、GPU 不置位）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	                violate(INVARIANT_I_9_10);	                _ = INVARIANT_I_9_10;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_names_another_object_birth_than_the_record	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_names_another_object_birth_than_the_record
第 23 批 GPU 判器：「在谱系上」恒真、不按谱系滤（删 → 回退 → 再删那条合法流上谱系外的旧墓碑与新墓碑同号同出生代，GPU 红 I-9.20、CPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    return le64(txg, txg_walked_to);\n}	    return true;\n}	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_on_every_state_of_an_unlink_rolled_back_and_unlinked_again	the_gpu_judge_agrees_with_the_cpu_on_every_state_of_an_unlink_rolled_back_and_unlinked_again
第 23 批 size 偏移合一：共用那一半的 inode 记录 size 偏移读成 blocks（偏移 48），恢复内核按它算数据单元数、扫描内核按它判 I-4.6，删除中间态的整份比用例红	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	const INODE_RECORD_SIZE_OFFSET: u32 = 40u;	const INODE_RECORD_SIZE_OFFSET: u32 = 48u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress	the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress
第 23 批对象出生代偏移进共用那一半：读成改动计数（偏移 88），恢复内核比数据单元头的对象出生代在顺序写那一版对不上（删除中间态的整份比用例红）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	const INODE_RECORD_OBJECT_BIRTH_OFFSET: u32 = 8u;	const INODE_RECORD_OBJECT_BIRTH_OFFSET: u32 = 88u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress	the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress
````

## 实现员 g：mutations-replacements.tsv（/tmp/claude-1000/m3-batch23/g/patch/mutations-replacements.tsv）

````text
第 5 批 GPU 判器：派活给每个状态的草稿区停在交接段末端、没跟上内核新加的两张表	crates/singlefs-checker-tier/src/crash_judge_gpu.rs	pub const SCRATCH_WORDS: usize = 315_376;	pub const SCRATCH_WORDS: usize = 247_792;	-p singlefs-checker-tier --features verdict-store,gpu --lib -- crash_judge_gpu::tests::scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels	scratch_words_given_to_each_state_reach_the_scratch_end_of_the_kernels
````

## 实现员 x：crates.patch（/tmp/claude-1000/m3-batch23/x/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs b/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs
index e2e45ab..ec01f6f 100644
--- a/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs
+++ b/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs
@@ -17,10 +17,12 @@
 //! （E142 第 22 次第二段的 P8G：两块 8 GiB 盘，`research/prompts/e142-r22-prereg.md` 读法 R67），别的参数照旧，
 //! `make_filesystem` 收不收由它自己判（环不超过盘的四分之一）。`name=impl_config` 的 `image_bytes=` 打的是这一次的每盘字节数。
 //! 不读环境、不碰真设备、不写任何文件。退出码：0 = 正常；1 = 命令行参数不是上面两种之一，或每盘字节数解得开而
-//! `make_filesystem` 不收（例：`--device-bytes 0`），两种都在 stderr 打原因（后一种打 mkfs 交回的错）；
+//! 两块盘内存里的单元区空闲图在这台机器上分配不出（例：`--device-bytes 18446744073709551615`，建盘之前判），
+//! 或 `make_filesystem` 不收（例：`--device-bytes 0`），三种都在 stderr 打原因（最后一种打 mkfs 交回的错）；
 //! 101 = mkfs 收了、后面某一步（取号、暖机、新池新建文件）在内存盘上没走通，panic 带那一步与原因。
 
 use singlefs_core::address::DeviceIdentity;
+use singlefs_core::allocator::{unit_area_slots_of_device_starting_at, UnitAreaStart};
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_format::{NEW_POOL_FILE_CREATION_TXG, TEST_IMAGE_DEFAULT_BYTES};
 use singlefs_harness::hexadecimal::hexadecimal_text;
@@ -133,6 +135,48 @@ fn device_bytes_from_the_command_line(arguments: &[String]) -> Result<u64, Strin
     }
 }
 
+/// 一块盘的单元区每槽在内存空闲图（`singlefs_core::allocator::DeviceFreeMap::with_unit_area_start`）里至多占几个字节：
+/// 每槽三个布尔数组（已分配、隔离、扣到 F 生效）3 字节，每 64 槽一段的四个 `u64` 计数数组合起来每槽半个字节，向上取整。
+const FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST: u64 = 4;
+
+/// 两块盘的单元区空闲图在这台机器上分配得出：mkfs 收下之后建分配器（`allocator_after_make_filesystem`）照每盘单元区槽数分配每槽一格的空闲图，
+/// 那一步分配不出时进程当场 abort（退 134：`--device-bytes 18446744073709551615` 建盘就是这样，代码轮判决
+/// `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节 K1 那一格）。这里在建盘之前按 [`FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST`]
+/// 可失败地把两块盘那么多一起试着要一次，要到了就放掉；要不到交回给人看的一句。
+/// 单元区起点不在聚簇段边界上、盘末尾在单元区起点之前的不试，mkfs 自己拒（`UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`、
+/// `UnitAreaBeyondDevice`），照旧打 mkfs 交回的错。
+fn free_maps_of_the_two_devices_fit_in_memory(
+    device_bytes: u64,
+    journal_ring_bytes: u64,
+) -> Result<(), String> {
+    let Ok(unit_area_start) = UnitAreaStart::following_the_journal_ring(journal_ring_bytes) else {
+        return Ok(());
+    };
+    let Ok(unit_area_slots) = unit_area_slots_of_device_starting_at(device_bytes, unit_area_start)
+    else {
+        return Ok(());
+    };
+    let refusal = |why: String| {
+        format!(
+            "每盘 {device_bytes} 字节：单元区 {unit_area_slots} 槽，内存里的空闲图每盘至多要 {FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST} × {unit_area_slots} 字节，{DEVICE_COUNT} 块盘在这台机器上分配不出（{why}）"
+        )
+    };
+    let free_map_bytes_per_device = unit_area_slots
+        .checked_mul(FREE_MAP_BYTES_PER_UNIT_AREA_SLOT_AT_MOST)
+        .and_then(|bytes| usize::try_from(bytes).ok())
+        .ok_or_else(|| refusal("字节数超出这台机器的地址宽度".to_string()))?;
+    // 两块盘的空闲图同时活着：两份一起要到才算要得到。迭代上界是盘数。
+    let mut reserved_free_maps: Vec<Vec<u8>> = Vec::new();
+    for _device_number in 0..DEVICE_COUNT {
+        let mut free_map: Vec<u8> = Vec::new();
+        free_map
+            .try_reserve_exact(free_map_bytes_per_device)
+            .map_err(|error| refusal(error.to_string()))?;
+        reserved_free_maps.push(free_map);
+    }
+    Ok(())
+}
+
 fn main() {
     let arguments: Vec<String> = std::env::args().skip(1).collect();
     let device_bytes = match device_bytes_from_the_command_line(&arguments) {
@@ -161,6 +205,13 @@ fn main() {
     };
 
     let parameters = e142_parameters(PHYSICAL_BLOCK_SIZE_IN_BYTES, MINIMUM_INPUT_OUTPUT_BYTES);
+    if let Err(refusal) = free_maps_of_the_two_devices_fit_in_memory(
+        device_bytes,
+        parameters.geometry.journal_ring_bytes,
+    ) {
+        eprintln!("e142_new_pool_file_creation_write_dump {refusal}");
+        std::process::exit(EXIT_CODE_OF_A_REFUSED_COMMAND_LINE);
+    }
     let stream = SharedStream::retaining_contents();
     let mut devices = two_recording_memory_devices(device_bytes, &stream);
 
@@ -281,9 +332,10 @@ fn main() {
 mod tests {
     use super::{
         device_bytes_from_the_command_line, ending_of_an_unfinished_path,
-        two_recording_memory_devices, EndingOfAnUnfinishedPath, MINIMUM_INPUT_OUTPUT_BYTES,
-        PHYSICAL_BLOCK_SIZE_IN_BYTES,
+        free_maps_of_the_two_devices_fit_in_memory, two_recording_memory_devices,
+        EndingOfAnUnfinishedPath, MINIMUM_INPUT_OUTPUT_BYTES, PHYSICAL_BLOCK_SIZE_IN_BYTES,
     };
+    use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
     use singlefs_harness::scenario::{
         e142_parameters, run_new_pool_file_creation, NewPoolFileCreationPathFailure,
         NewPoolFileCreationPathStep,
@@ -357,6 +409,36 @@ mod tests {
         assert_eq!(stream.operation_count(), 0, "mkfs 拒在任何写之前");
     }
 
+    /// `--device-bytes 18446744073709551615` 解得开（第二种写法），两块盘的单元区空闲图分配不出：建盘之前照命令行给错了收场、打原因，
+    /// 不走到建分配器那一步 abort（退 134，代码轮判决 `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节 K1 那一格）。
+    /// 跑前登记的 4 GiB 与 P8G 的 8 GiB 两档照旧放过；每盘 0 字节不在这里拒，交给 mkfs 判（上一条）。
+    #[test]
+    fn the_largest_device_bytes_parse_but_their_free_maps_cannot_be_allocated_and_the_bin_ends_as_a_refused_command_line(
+    ) {
+        let device_bytes = device_bytes_from_the_command_line(&arguments(&[
+            "--device-bytes",
+            "18446744073709551615",
+        ]))
+        .expect("2⁶⁴ − 1 是一个十进制字节数，解得开");
+        match free_maps_of_the_two_devices_fit_in_memory(device_bytes, JOURNAL_RING_DEFAULT_BYTES) {
+            Ok(()) => panic!("每盘 2⁶⁴ − 1 字节的单元区空闲图不该分配得出"),
+            Err(refusal) => assert!(
+                refusal.contains("分配不出") && refusal.contains("每盘 18446744073709551615 字节"),
+                "原因点名每盘字节数与分配不出：{refusal}"
+            ),
+        }
+        for device_bytes_that_fit in [4_294_967_296, 8_589_934_592, 0] {
+            assert_eq!(
+                free_maps_of_the_two_devices_fit_in_memory(
+                    device_bytes_that_fit,
+                    JOURNAL_RING_DEFAULT_BYTES
+                ),
+                Ok(()),
+                "每盘 {device_bytes_that_fit} 字节不在这里拒"
+            );
+        }
+    }
+
     /// mkfs 收了而后面某一步没走通的，照旧当实现的毛病（panic），不混进命令行给错了那一种。
     #[test]
     fn a_failure_after_make_filesystem_stays_an_implementation_failure() {
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
index 102946f..e6a3faf 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
@@ -7,7 +7,15 @@
 mod common;
 
 use common::{build_pool, file_content, geometry};
-use singlefs_checker::image::MAPPING_KEY_MATCHES_THE_UNIT_HEADER;
+use singlefs_checker::image::{
+    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY, INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
+    LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER, MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
+    NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+    ONE_OBJECT_BIRTH_PER_INODE_NUMBER, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
+    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+    TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
+    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+};
 use singlefs_checker::walk::check_pool_image;
 use singlefs_checker_tier::crash::{
     check_records, enumerate_layer0_in_state_slices_or_one_shard, enumerate_layer0_selecting,
@@ -87,7 +95,7 @@ fn checker_line(tally: &Layer0Tally) -> String {
 }
 
 /// checker 与记录核对器那一半的钉死值。只在根槽 txg 3 已持久的状态上才走得到记账树、inode 树、分配记录树与中央映射树的条目，
-/// 那 22 条只在这些状态上评估
+/// 那 26 条只在这些状态上评估（删除态扫描那四条 I-9.5、I-3.4、I-9.24、I-9.29 的理由写在函数里那张表旁边）
 /// （I-5.4（分配记录罩住的槽互不相交） 在其中，
 /// I-9.25（树表条目的 previous_snapshot_txg）、I-9.27（树表条目的头 ID）、I-3.13（记账行的代段）、I-9.26（inode 记录的改动计数）、
 /// I-5.5（从 mkfs 水位建树那一版的第一个数据单元） 与 I-3.5（引用区间的精确性） 同一个理由：种子根与暖机根指着的第 0 版树表是空的，
@@ -109,7 +117,8 @@ fn checker_line(tally: &Layer0Tally) -> String {
 /// I-8.6（反向链算法） 要环里至少有一条自证过的记录，环还空着的那几个状态上报「不适用」；
 /// I-8.7（实例内事务号不重号）、I-8.8（前缀里的事务不被切开） 与 I-7.9（回退下界 F 不高于抬 F 的上限） 在这条流上一个状态都评估不到；
 /// I-8.10、I-8.11、I-8.12、I-4.1、I-7.16 各自一类，评估过的状态数逐条写在函数里那几张表旁边；
-/// 其余 33 条每个状态都评估（2026-10-04 主工作区 checker 档日志快档那一行 `CHECKER`：69 条里 33 条是 46/0）。
+/// 删除与回退那十条（I-4.7、I-4.6、I-4.4 与墓碑、谱系那七条 I-9.17–I-9.23）在这条流上一个状态都评估不到，理由同样写在表旁边；
+/// 其余 33 条每个状态都评估（2026-10-05 主工作区 checker 档日志快档那一行 `CHECKER`：74 条里 33 条是 46/0）。
 /// 违例只许出现在 I-7.7（系统配置实例代号不低于根环）：C322（取号那一步的屏障怎么放没有条款） 未还，
 /// 取号只落了一块盘的那 2 个状态里两份系统配置的实例代号不等。
 fn assert_checker_counts(
@@ -150,6 +159,15 @@ fn assert_checker_counts(
         "I-9.26",
         "I-5.5",
         "I-3.5",
+        // 删除态扫描（`walk::judge_deletion_state_of_the_versions` 与 `judge_tombstones_and_the_lineage_of_the_versions`）只扫树表有条目的那几版，
+        // 这条流上只有 txg 3 那一版：I-9.5（队列与 nlink 0 的记录同号）与 I-9.24（队列至多一条 key，空队列报成立）逐条判 `deleted_inodes` 条目，
+        // I-3.4（记账标签 4 恒 0）判记账树，I-9.29（同号只许一个出生代）比 inode 记录收出来的号——都要 txg 3 的根已落。
+        // 记录已落、根没落的那几个状态上由记录施加出来的那一版不进删除态扫描的版本清单（同 I-1.11 那一行的理由：实例表里没有行）。
+        // I-9.5、I-3.4 的 9 对得上 2026-10-05 主工作区 checker 档日志快档那一行的 `I-9.5=9/0`、`I-3.4=9/0`；I-9.24、I-9.29 是推的。
+        "I-9.5",
+        "I-3.4",
+        DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
+        ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
     ];
     // I-3.10 除了回退候选集，还读下一次挂载会先施加的那一版（射程 ④）：新根没落而 txg 3 的记录已落时，
     // 恢复要由那条记录重建 txg 3 那一版，它的分配记录树在记录的新根段指着的树表下面。
@@ -170,6 +188,26 @@ fn assert_checker_counts(
     // I-3.12 要根指着的实例表里有行，这条流上全部根都指着 mkfs 那张只有链记录的实例表（取号不写行、暖机与 A 照抄实例表指针）。
     // 两条的 0 都对得上 2026-10-04 主工作区 checker 档日志快档那一行 `CHECKER` 的 `I-9.28=0/0`、`I-3.12=0/0`。
     let never_comparable_on_this_stream = ["I-9.14", "I-8.7", "I-8.8", "I-7.9", "I-3.12", "I-9.28"];
+    // 这条流上没有删除（没有 nlink = 0 的记录、`deleted_inodes` 里没有号）：I-4.7（删除中 inode 有账） 与 I-4.6（删除中 inode 的数据单元数对得上 size）
+    // 没有对象；I-9.18（删除队列里的号有谱系上的墓碑） 队列空就不判。这三条的 0 对得上同一份日志的 `I-4.7=0/0`、`I-4.6=0/0`（I-9.18 是推的）。
+    let never_has_an_inode_being_deleted_on_this_stream = [
+        "I-4.7",
+        "I-4.6",
+        QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
+    ];
+    // 没有删除、没有回退：txg 3 那一版的墓碑树、谱系树都是零根（全部持久那个状态就是 `checker_known_bad_images.rs` 的干净镜像，
+    // 那里这几条报不适用），主走读走不到它们的内部条目（I-9.17、I-9.23），删除态扫描读出的墓碑容器是空的（I-9.19、I-9.20 没有墓碑可判，
+    // I-9.21 在旧入口上要至少两片容器）；I-9.22 只在新入口装了旋钮时判，层 0 走旧入口 `check_pool_image`。全是推的。
+    let never_has_a_tombstone_or_lineage_record_on_this_stream = [
+        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
+        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
+        NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
+        TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
+    ];
+    // I-4.4（意图只描述目标态） 第一版报不适用（用户 2026-10-05 定），哪一条流上都评估不到；对得上同一份日志的 `I-4.4=0/0`。
+    let not_applicable_in_the_first_version = [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE];
     // I-8.6 判的是「本实例内逻辑前一条」与这一条的链值，环里一条自证过的记录都没有时判不了：
     // 取号那一段（两盘各一次系统配置写，原地覆写各三态）的 3² − 1 个状态，加上暖机第一次的记录段整段没落那一个，
     // 一共 9 个状态还没有记录落盘（补第三态之前 4 个）。
@@ -190,7 +228,11 @@ fn assert_checker_counts(
     // I-7.16 判可择的系统配置槽的世代号：每块盘都只剩一槽可择、而且是 mkfs 写下的世代号 1 时一格都不判。
     let judged_unless_every_device_has_one_choosable_slot_written_by_mkfs = ["I-7.16"];
     for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
-        let expected_evaluated = if never_comparable_on_this_stream.contains(&invariant) {
+        let expected_evaluated = if never_comparable_on_this_stream.contains(&invariant)
+            || never_has_an_inode_being_deleted_on_this_stream.contains(&invariant)
+            || never_has_a_tombstone_or_lineage_record_on_this_stream.contains(&invariant)
+            || not_applicable_in_the_first_version.contains(&invariant)
+        {
             0
         } else if only_under_the_new_root.contains(&invariant) {
             root_persisted_states
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 8ed0629..df60352 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -9002,6 +9002,17 @@ const TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS: [&str; 4] = [
     ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
 ];
 
+/// 回退候选集滤没滤得掉被崩溃恢复抛弃的根：候选集按最新根指着的实例表判被抛弃的根（D23（journal 的角色与格式） 已定项 14 回退段的候选集规则），
+/// 那张表读不出时一行都滤不掉，被抛弃时间线上的根也进了候选集。I-9.29（同号只许一个出生代） 跨版本比，这时记判不了
+/// （`.claude/kb/invariants.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」）：被抛弃的线上同号可以是另一个对象。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum AbandonedRootsFilteringOfTheCandidateSet {
+    /// 最新根的实例表沿链整张读得出（[`instance_table_rows_of_root_without_judging`] 交回了行）：被抛弃的根都按它滤掉了。
+    FilteredByTheNewestRootsInstanceTable,
+    /// 最新根的实例表链上有一片读不出、解不开（同一个读法交回 None）：被抛弃的根可能留在候选集里。
+    UnfilteredAsTheNewestRootsInstanceTableDoesNotReadBack,
+}
+
 /// 装没装旋钮决定墓碑容器容量判哪一条：没装（旧入口）判产品那一条 I-9.21，装了判测试那一条 I-9.22。
 const fn capacity_invariant_of(
     capacity_knob: Option<TombstoneContainerCapacityKnob>,
@@ -9256,12 +9267,14 @@ fn lineage_records_without_judging(
 /// 墓碑与谱系那几条（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节与第四节用户定案）：删除态扫描那一份版本清单里
 /// 每一版（树表与 txg 都相同的几版只扫一次——「在谱系上」从这一版自己的 txg 起算，txg 不同就各扫一遍）重读树表、谱系树、墓碑树、
 /// inode 树与 `deleted_inodes`，只读不判、不进 `walk_failures`：判 I-9.18、I-9.19、I-9.20、I-9.21 / I-9.22（看装没装旋钮）与 I-9.10 墓碑那一个等号，
-/// 把各版的号 → 对象出生代收进一张跨版本的表，全部扫完判 I-9.29。读不全、配不上的报射程里判不了（同 I-9.5 那一行的共用判法）；
+/// 把各版的号 → 对象出生代收进一张跨版本的表，全部扫完判 I-9.29；候选集滤不掉被抛弃的根（[`AbandonedRootsFilteringOfTheCandidateSet`]）时
+/// 那张表一个号都不比、I-9.29 报部分没判。读不全、配不上的报射程里判不了（同 I-9.5 那一行的共用判法）；
 /// 不按已发布与否滤墓碑（第二节 T4）。I-9.10 墓碑那一个等号读不全、配不上时不记判不了，与它数据单元那一半同口径
 /// （C701（别的判法静默跳过读不出的部分））。
 fn judge_tombstones_and_the_lineage_of_the_versions(
     reader: &dyn ImageReader,
     versions: &[VersionToScanTheDeletionStateOf<'_>],
+    abandoned_roots_filtering: AbandonedRootsFilteringOfTheCandidateSet,
     capacity_knob: Option<TombstoneContainerCapacityKnob>,
     index_cache: &mut IndexNodeCache,
     judgements: &mut Judgements,
@@ -9321,7 +9334,20 @@ fn judge_tombstones_and_the_lineage_of_the_versions(
             judgements,
         );
     }
-    judge_one_object_birth_per_inode_number(&object_births, judgements);
+    match abandoned_roots_filtering {
+        AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable => {
+            judge_one_object_birth_per_inode_number(&object_births, judgements);
+        }
+        AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheNewestRootsInstanceTableDoesNotReadBack => {
+            judgements.partially_judged(
+                ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
+                format!(
+                    "最新根的实例表读不出（链上有一片读不出或解不开）：回退候选集里滤不掉被崩溃恢复抛弃的根，收到的 {} 个号一个都没比",
+                    object_births.len()
+                ),
+            );
+        }
+    }
     // 一次都没判到时的理由（判到过的照判定报，`Judgements::not_applicable` 不改已经评估过的）。
     judgements.not_applicable(
         QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
@@ -10199,6 +10225,15 @@ pub fn check_pool_image_given_the_tombstone_container_capacity_knob(
             row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg
         })
     };
+    // 上面那张表是主走读沿链判下来的行，链断在哪一片就只有那一片之前的行：读不全时滤不掉的根认不出来。
+    // 滤没滤全按 I-7.9 与记录核对器读一条根自己的实例表那个读法判（链上哪一片读不出、解不开 ⇒ None）。
+    let abandoned_roots_filtering = match instance_table_rows_of_root_without_judging(
+        reader,
+        &roots[newest_index].2.record_bytes,
+    ) {
+        Some(_) => AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable,
+        None => AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheNewestRootsInstanceTableDoesNotReadBack,
+    };
     // 回退候选集的下界取 F 生效值（D16（发布语义） 已定项 1「生效」，SysPre）：max(各幸存盘最新持久有效根所带 F 的最大值,
     // 池里自证过的系统配置槽带的 F)。有效 = 按最新根指着的实例表判不是被抛弃的；每块盘取落在它上面的根环区域里 (txg, 实例) 最大的那一条
     // （最新根的实例表读不出时一行都没有、不滤）；被抛弃时间线上的根带的 F 也算进来（D16（发布语义） 已定项 1，用户 2026-09-28 定「算进」）。
@@ -10527,6 +10562,7 @@ pub fn check_pool_image_given_the_tombstone_container_capacity_knob(
     judge_tombstones_and_the_lineage_of_the_versions(
         reader,
         &versions_to_scan_the_deletion_state_of,
+        abandoned_roots_filtering,
         tombstone_container_capacity_knob,
         &mut index_node_cache,
         &mut judgements,
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index b954dc0..4d81998 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -3880,6 +3880,112 @@ fn birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_hol
     }
 }
 
+/// 一条位置条目（盘 4 + 槽 6 + 整单元校验和 4，D19（块指针的结构与宽度预算） 已定项 11）里槽号那 6 字节的偏移。
+const LOCATION_ENTRY_SLOT_OFFSET: usize = 4;
+/// 实例表单元（码 3）占两槽的字节数。
+const INSTANCE_TABLE_UNIT_BYTES: usize = 32768;
+/// 根记录里实例代号（4 字节）与 checkpoint_txg（8 字节）的偏移（D22（单元原子性怎么合成） 已定项 7）。
+const ROOT_RECORD_INSTANCE_OFFSET: usize = 24;
+const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;
+
+/// 根环里 (checkpoint_txg, 实例) 最大的那条根（择根序，D22（单元原子性怎么合成） 已定项 7）指着的实例表第 0 片的槽号（第一条位置条目，两盘同槽）。
+fn instance_table_slot_of_the_newest_root(image: &MemoryPool) -> u64 {
+    let newest_root = root_slots()
+        .into_iter()
+        .map(|(device, offset)| read(image, device, offset, 512))
+        .filter(|root| &root[..4] == b"SFSR")
+        .max_by_key(|root| {
+            (
+                get_u64(root, ROOT_RECORD_CHECKPOINT_TXG_OFFSET),
+                get_u32(root, ROOT_RECORD_INSTANCE_OFFSET),
+            )
+        })
+        .expect("根环里有根");
+    let slot_start = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
+        + POINTER_FIRST_LOCATION_OFFSET
+        + LOCATION_ENTRY_SLOT_OFFSET;
+    let mut slot_bytes = [0u8; 8];
+    slot_bytes[..6].copy_from_slice(&newest_root[slot_start..slot_start + 6]);
+    u64::from_le_bytes(slot_bytes)
+}
+
+/// 崩溃恢复抛弃 C 之后的干净镜像（[`image_after_a_recovery_abandoned_the_third_root`] 那一份）上，只改被抛弃的 C 那片 inode 叶容器（两盘同槽）里
+/// 第一条记录的对象出生代（加一），重封、沿引用链补到 C 的根槽：C 那一版里这个号对着另一个对象出生代，现行线上 A、B 与实例 3 那几版仍是原来的。
+fn image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth(
+    tag: &str,
+) -> MemoryPool {
+    let (mut pool, _, third) = pool_through_the_third_version(tag);
+    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &third);
+    let mut image = pool.memory_pool();
+    let inode_leaf_of_the_abandoned_root = third
+        .unit(TransactionUnit::InodeLeafContainer(
+            singlefs_core::inode_tree::InodeLeafContainerIndexInTree::LEFTMOST,
+        ))
+        .slot
+        .0;
+    let units = units_found_on_device_zero(&image);
+    mutate_unit_of(
+        &mut image,
+        &units,
+        inode_leaf_of_the_abandoned_root,
+        |bytes| {
+            assert_eq!(
+                bytes[6], 3,
+                "槽 {inode_leaf_of_the_abandoned_root} 上还是 C 写下的 inode 叶容器（码 3）"
+            );
+            let object_birth = INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_OBJECT_BIRTH_OFFSET;
+            let another_object_birth = get_u64(bytes, object_birth) + 1;
+            set_u64(bytes, object_birth, another_object_birth);
+        },
+        true,
+    );
+    image
+}
+
+/// I-9.29（同号只许一个出生代） 判不了那一形（`.claude/kb/invariants.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」；
+/// 第 22 批 C 实现员报告「交主 agent 的设计判断」第 5 条）。镜像见
+/// [`image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth`]：被抛弃的 C 与现行线上的 A、B 同号不同出生代。
+/// ① 最新根的实例表读得出：C 按实例 3 的实例表判出局、不进候选集，I-9.29 真被评估过且成立，判红的只是干净镜像上那几条（一条都没有）。
+/// ② 再把最新根指着的实例表单元两份各翻一个载荷字节（不重封）：候选集滤不掉 C，I-9.29 报部分没判，不报违例
+/// （读不出的那一片由 I-2.1、I-7.2 这几条判红，这里不钉）。
+/// 判别力：实例表读不出时照旧跨版本比，C 那条记录的出生代与 A、B 的不同，② 红在 I-9.29 上。
+#[test]
+fn an_abandoned_root_naming_another_object_birth_leaves_the_one_birth_invariant_partially_judged_once_the_newest_instance_table_does_not_read_back(
+) {
+    let filtered = image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth(
+        "known-bad-one-birth-abandoned-root-unfiltered",
+    );
+    let verdicts_while_filtered = check_pool_image(&filtered);
+    assert_eq!(
+        violated_invariants(&verdicts_while_filtered),
+        Vec::<&str>::new(),
+        "被抛弃的 C 按实例表滤掉了，改它那片 inode 叶不红任何一条：{verdicts_while_filtered:?}"
+    );
+    assert_eq!(
+        verdict(&filtered, ONE_OBJECT_BIRTH_PER_INODE_NUMBER),
+        InvariantVerdict::Holds,
+        "C 不在候选集里，现行线上各版同号同出生代：I-9.29 真被评估过且成立"
+    );
+    let mut unfiltered = filtered.clone();
+    let instance_table_slot = instance_table_slot_of_the_newest_root(&unfiltered);
+    break_both_copies_of_the_unit(
+        &mut unfiltered,
+        instance_table_slot,
+        INSTANCE_TABLE_UNIT_BYTES,
+    );
+    match verdict(&unfiltered, ONE_OBJECT_BIRTH_PER_INODE_NUMBER) {
+        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
+            unjudged.starts_with("最新根的实例表读不出"),
+            "部分没判的说明点名最新根的实例表读不出：{unjudged}"
+        ),
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::NotApplicable(_)) => panic!(
+            "最新根的实例表（槽 {instance_table_slot}）读不出、候选集滤不掉被抛弃的 C：I-9.29 要报部分没判，实际 {other_verdict:?}"
+        ),
+    }
+}
+
 /// 发布 B 写出的数据单元：它的分配记录在 B 那一版分配记录树的两片叶（`ALLOCATION_LEAVES_AFTER_OVERWRITE`）里，未释放、分配代 4，
 /// 单元头里的诞生代号也是 4（落点由 `overwrite_in_one_instance.rs` 的验收钉住）。
 const DATA_UNIT_AFTER_OVERWRITE: u64 = 50182;
diff --git a/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs b/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
index 7c8cb40..44d72a3 100644
--- a/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
+++ b/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
@@ -607,7 +607,7 @@ fn after_a_crashed_unmount_of_a_four_gibibyte_pool_the_next_writable_mount_hands
 /// K = 1、4 就是现行那一版，K = 0、2、5 早一个，K = 3、6 早两个；覆盖写 3 次、卸载落下 15 个写那一形也早两个（`research/prompts/m3-batch21-code-r1-opus-output.md`
 /// 第二节「放开扫」与「改法 G5」两段）。早它的时候盘 1 上那条根还引用这几个槽，重建分配器时回收、扣到这次挂载的暖机落满每块盘
 /// （`mount::rebuilt_allocator`）；早两个 txg 时中间那一版换下的槽同样扣着，只看现行那一版换下的判不出「只扣现行那一版换下的」那一刀，见
-/// [`after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_in_either_warm_up`]。
+/// [`after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_released_by_any_version_after_the_root_device_one_alone_recovers_to`]。
 fn slots_released_by_the_current_version(pool: &PoolUnderTest<SparseBlockDevice>) -> Vec<u64> {
     pool.current_file_version()
         .released
@@ -623,7 +623,7 @@ fn slots_released_by_the_current_version(pool: &PoolUnderTest<SparseBlockDevice>
 /// 取号之前在分配器拷贝上预演的落点与真发的不同，挂载在预演对真发那一判上 panic，这一条红；那一判不在时，最后一项红
 /// （在副本里关掉那一判量过：暖机落在换下的槽 [1410] 上）。只读挂载丢盘的那几条都在写行那次就崩，暖机没跑到，判不出这一刀。
 /// 卸载只落下第一个写，这次挂载的暖机只推一次：「第一次暖机之后就放开」与「暖机落满之后放开」在这里是同一刻，判不出，那一刀见
-/// [`after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_in_either_warm_up`]。
+/// [`after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_released_by_any_version_after_the_root_device_one_alone_recovers_to`]。
 #[test]
 fn after_a_crashed_unmount_of_a_narrow_pool_the_next_writable_mount_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_until_its_warm_up_landed_on_every_device(
 ) {
@@ -689,9 +689,8 @@ const UNMOUNT_WRITES_LANDED_BEFORE_A_MOUNT_THAT_WARMS_UP_TWICE: u64 = 15;
 /// 重建时当场放开到 min(F_生效, max(每块盘上最新有效根的 txg 最小者, F_生效 − 1))、只扣现行那一版换下的（攻方
 /// `research/prompts/m3-batch21-code-r1-opus-model/patches/w2.patch`）时，写行那次的单元写进倒数第二次覆盖写换下的槽，最后一项红；
 /// 只比现行那一版换下的槽时这一刀照绿（攻方量过：丢盘 0 之后盘 1 读文件报 `DataUnitChecksumMismatchEverywhere`）。
-/// 测试名里「卸载之前那一版」说的是盘 1 上最新的根之后的每一版，名字没改：变异表里两行按这个名字点名。
 #[test]
-fn after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_in_either_warm_up(
+fn after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_released_by_any_version_after_the_root_device_one_alone_recovers_to(
 ) {
     let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
     let mut pool =
````

## 实现员 x：mutations-append.tsv（/tmp/claude-1000/m3-batch23/x/patch/mutations-append.tsv）

````text
第23批 X I-9.29：最新根的实例表读不出时当它滤过了被抛弃的根，照旧跨版本比（被抛弃的 C 同号不同出生代，I-9.29 判红而不是部分没判）	crates/singlefs-checker/src/walk.rs	        None => AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheNewestRootsInstanceTableDoesNotReadBack,	        None => AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable,	-p singlefs-harness --test checker_known_bad_images -- an_abandoned_root_naming_another_object_birth	an_abandoned_root_naming_another_object_birth_leaves_the_one_birth_invariant_partially_judged_once_the_newest_instance_table_does_not_read_back
第23批 X I-9.29：候选集滤不掉被抛弃的根时照样跨版本比、再记部分没判（违例压过部分没判，I-9.29 判红）	crates/singlefs-checker/src/walk.rs	            judgements.partially_judged(\n                ONE_OBJECT_BIRTH_PER_INODE_NUMBER,	            judge_one_object_birth_per_inode_number(&object_births, judgements);\n            judgements.partially_judged(\n                ONE_OBJECT_BIRTH_PER_INODE_NUMBER,	-p singlefs-harness --test checker_known_bad_images -- an_abandoned_root_naming_another_object_birth	an_abandoned_root_naming_another_object_birth_leaves_the_one_birth_invariant_partially_judged_once_the_newest_instance_table_does_not_read_back
第23批 X E142 导出：建盘之前试分配空闲图只要 0 字节（每盘 2⁶⁴ − 1 字节放过去、建分配器那一步 abort 退 134）	crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs	            .try_reserve_exact(free_map_bytes_per_device)	            .try_reserve_exact(0)	-p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump -- the_largest_device_bytes_parse	the_largest_device_bytes_parse_but_their_free_maps_cannot_be_allocated_and_the_bin_ends_as_a_refused_command_line
````

## 实现员 x：mutations-replacements.tsv（/tmp/claude-1000/m3-batch23/x/patch/mutations-replacements.tsv）

````text
删除批第 21 批 N3：可写挂载在第一次暖机之后就放开重建分配器时扣住的那一截，预演对真发那一判不比（等于取号之前的预演也在那里放开，攻方 m3-batch19-code-r1 臂 6），暖机推两次时第二次暖机写进卸载之前现行那一版换下的槽	crates/singlefs-core/src/mount.rs	        warm_up_publishes.push(next.clone());\n        current = next;\n    }\n    // 暖机走完、本实例的根在每块盘上都落了（`warm_up_publish_txgs` 按根环落点公式推到覆盖每块盘为止），重建分配器时扣住的那一截放开\n    // （`rebuilt_allocator`，用户 2026-10-04 选甲）：此后丢哪块盘，恢复择到的都不低于本实例的根，它们不引用这一截。\n    // 放开的只有那一截：扣住位只有两处会置——重建分配器那一处与抬 F 那一串（`raise_rollback_floor`），这次挂载的抬 F 都排在这一句之后\n    // （写行之后的准入 `push_floor_raises_after_the_row_publish`、续做里推的那几串），它们置的扣住位归它们自己放（`raise_rollback_floor`）；\n    // 走到这里分配器上的扣住位全是重建那一处置的。暖机没走完（取号之前被拒、写行或暖机落盘途中报错）时挂载整次报错，分配器随错丢掉，\n    // 扣住的那一截不交给任何会话。\n    allocator.release_reclaim_holds();\n    // 取号之前在拷贝上取的落点就是真发起来取的：同一个分配器、同一串分配动作。这一串里有一份换下的单元读盘核校验和对不上时不比——\n    // 真发时那一份的分配记录留在已分配，拷贝上没做那一步、照常释放，那一槽若在这一串里被回收（崩溃恢复落到环里最旧的那条根、其余全被抛弃），两边可以不同\n    // （`dry_run_of_the_publishes_after_acquisition` 的文档注释）；那一格取号之前判的与真发的不是同一串，是这一版留着的缺口。\n    let placements_taken: Vec<PlacementsTakenByOnePublish> = std::iter::once(&row_publish)\n        .chain(&warm_up_publishes)\n        .map(placements_taken_by)\n        .collect();\n    let some_copy_was_quarantined = std::iter::once(&row_publish)\n        .chain(&warm_up_publishes)\n        .filter_map(PoolVersion::version_of_trees)\n        .any(|version| {\n            !version\n                .quarantined_after_release_checksum_mismatch\n                .is_empty()\n        });\n    if let (Some(planned), false) = (&placements_planned, some_copy_was_quarantined) {	        warm_up_publishes.push(next.clone());\n        if warm_up_publishes.len() == 1 {\n            allocator.release_reclaim_holds();\n        }\n        current = next;\n    }\n    // 暖机走完、本实例的根在每块盘上都落了（`warm_up_publish_txgs` 按根环落点公式推到覆盖每块盘为止），重建分配器时扣住的那一截放开\n    // （`rebuilt_allocator`，用户 2026-10-04 选甲）：此后丢哪块盘，恢复择到的都不低于本实例的根，它们不引用这一截。\n    // 放开的只有那一截：扣住位只有两处会置——重建分配器那一处与抬 F 那一串（`raise_rollback_floor`），这次挂载的抬 F 都排在这一句之后\n    // （写行之后的准入 `push_floor_raises_after_the_row_publish`、续做里推的那几串），它们置的扣住位归它们自己放（`raise_rollback_floor`）；\n    // 走到这里分配器上的扣住位全是重建那一处置的。暖机没走完（取号之前被拒、写行或暖机落盘途中报错）时挂载整次报错，分配器随错丢掉，\n    // 扣住的那一截不交给任何会话。\n    allocator.release_reclaim_holds();\n    // 取号之前在拷贝上取的落点就是真发起来取的：同一个分配器、同一串分配动作。这一串里有一份换下的单元读盘核校验和对不上时不比——\n    // 真发时那一份的分配记录留在已分配，拷贝上没做那一步、照常释放，那一槽若在这一串里被回收（崩溃恢复落到环里最旧的那条根、其余全被抛弃），两边可以不同\n    // （`dry_run_of_the_publishes_after_acquisition` 的文档注释）；那一格取号之前判的与真发的不是同一串，是这一版留着的缺口。\n    let placements_taken: Vec<PlacementsTakenByOnePublish> = std::iter::once(&row_publish)\n        .chain(&warm_up_publishes)\n        .map(placements_taken_by)\n        .collect();\n    let some_copy_was_quarantined = std::iter::once(&row_publish)\n        .chain(&warm_up_publishes)\n        .filter_map(PoolVersion::version_of_trees)\n        .any(|version| {\n            !version\n                .quarantined_after_release_checksum_mismatch\n                .is_empty()\n        });\n    if let (Some(planned), false) = (&placements_planned, some_copy_was_quarantined || true) {	-p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads -- after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times	after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_released_by_any_version_after_the_root_device_one_alone_recovers_to
删除批第 21 批 N3：可写挂载只在真发那一处第一次暖机之后就放开、取号之前的预演照旧暖机落满才放开（攻方 m3-batch19-code-r1 臂 5），暖机推两次时挂载在预演对真发那一判上 panic	crates/singlefs-core/src/mount.rs	        warm_up_publishes.push(next.clone());\n        current = next;\n    }\n	        warm_up_publishes.push(next.clone());\n        if warm_up_publishes.len() == 1 {\n            allocator.release_reclaim_holds();\n        }\n        current = next;\n    }\n	-p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads -- after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times	after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_released_by_any_version_after_the_root_device_one_alone_recovers_to
删除批第 22 批 N5：可写挂载重建分配器时当场放开到 min(F_生效, max(每块盘上最新有效根的 txg 最小者, F_生效 − 1))、只扣现行那一版换下的（攻方 m3-batch21-code-r1 W2），覆盖写 3 次、卸载落下 15 个写那一形写行那次写进倒数第二次覆盖写换下的槽	crates/singlefs-core/src/mount.rs	            effective_floor.min(newest_valid_root_on_every_device),	            effective_floor.min(newest_valid_root_on_every_device.max(CheckpointTxg(\n                effective_floor.0.saturating_sub(1),\n            ))),	-p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads -- after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times	after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_released_by_any_version_after_the_root_device_one_alone_recovers_to
````
