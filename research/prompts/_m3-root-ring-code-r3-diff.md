# m3-root-ring-code-r3 被判改动（第 9 批全部合入的补丁，次序即合入次序）

## batch9-gpu/patch

实现员报告：`/tmp/claude-1000/m3-r1/batch9-gpu/patch/report.md`（sha256 b1b14ac634da83e6…），补丁 sha256 cef8902a237b69da…

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index 25d4f64..faf45d8 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -791,6 +791,207 @@ fn w_judge_unit_birth_against_an_older_candidate_root(birth_txg: vec2<u32>) {
     }
 }
 
+// ───────────────────────── I-3.5（引用区间的精确性）的出现规则：走读时记下的东西 ─────────────────────────
+// 四张表在 crash_judge_checker_common.wgsl 的 C_APPEARANCE_*，只记主走读（上下文 0）走的那几版；这几个数是各表记到第几行，w_start(0) 清零。
+var<private> appearance_placement_count: u32;
+
+var<private> appearance_version_count: u32;
+
+var<private> appearance_version_placement_count: u32;
+
+var<private> appearance_walked_tree_table_count: u32;
+
+// 正在走的这一版树表里用户可见树的根记了几棵：记在版本表第 appearance_version_count 行（这一版走完才算进那一行）
+var<private> appearance_root_count_of_this_version: u32;
+
+// 走读此刻在哪棵用户可见树里往下走（walk.rs 的 user_visible_tree_being_walked）：在不在、种类、树 ID
+var<private> appearance_user_visible_tree_is_being_walked: bool;
+
+var<private> appearance_user_visible_tree_kind: u32;
+
+var<private> appearance_user_visible_tree: vec2<u32>;
+
+// 用户可见树的种类（walk.rs 的 USER_VISIBLE_TREE_KINDS：extent、inode、deleted_inodes）
+fn appearance_tree_kind_is_user_visible(kind: u32) -> bool {
+    return kind == TREE_KIND_EXTENT || kind == TREE_KIND_INODE || kind == TREE_KIND_DELETED_INODES;
+}
+
+// 落点表里 (盘, 槽) 那一行的下标，不在就是 NONE
+fn appearance_placement_index(device: u32, slot: vec2<u32>) -> u32 {
+    for (var i = 0u; i < appearance_placement_count; i = i + 1u) {
+        let row = C_APPEARANCE_PLACEMENTS + i * C_APPEARANCE_PLACEMENT_WORDS;
+        if (sw(row) == device && sw(row + 1u) == slot.x && sw(row + 2u) == slot.y) {
+            return i;
+        }
+    }
+    return NONE;
+}
+
+// walk.rs note_references_of_a_pointer_followed_by_the_walk 里记 I-3.5 那两张表的两句：落点第一次记下时记它是哪个单元
+// （unit_of_each_placement.entry(..).or_insert），走读此刻在一棵用户可见树里、这个落点还没打标签时打上那棵树（or_insert）
+fn appearance_note_placement(device: u32, slot: vec2<u32>, unit_device: u32, unit_slot: vec2<u32>) {
+    var index = appearance_placement_index(device, slot);
+    if (index == NONE) {
+        if (appearance_placement_count >= C_APPEARANCE_MAX_PLACEMENTS) {
+            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (40u << 8u);
+            return;
+        }
+        index = appearance_placement_count;
+        let new_row = C_APPEARANCE_PLACEMENTS + index * C_APPEARANCE_PLACEMENT_WORDS;
+        sws(new_row, device);
+        sws(new_row + 1u, slot.x);
+        sws(new_row + 2u, slot.y);
+        sws(new_row + 3u, unit_device);
+        sws(new_row + 4u, unit_slot.x);
+        sws(new_row + 5u, unit_slot.y);
+        sws(new_row + 6u, 0u);
+        sws(new_row + 7u, 0u);
+        sws(new_row + 8u, 0u);
+        sws(new_row + 9u, 0u);
+        appearance_placement_count = appearance_placement_count + 1u;
+    }
+    let row = C_APPEARANCE_PLACEMENTS + index * C_APPEARANCE_PLACEMENT_WORDS;
+    if (appearance_user_visible_tree_is_being_walked && sw(row + 6u) == 0u) {
+        sws(row + 6u, 1u);
+        sws(row + 7u, appearance_user_visible_tree_kind);
+        sws(row + 8u, appearance_user_visible_tree.x);
+        sws(row + 9u, appearance_user_visible_tree.y);
+    }
+}
+
+// 实例表一片的诞生代号（walk.rs 的 birth_txg_of_walked_instance_table_pages）：记进走过的单元那一行，标记写 2——
+// I-4.9 只认标记 1（实例表单元按 I-1.2 那一行的例外不进 I-4.9），I-3.5 两种都认
+fn appearance_note_the_birth_txg_of_an_instance_table_page(device: u32, slot: vec2<u32>, birth_txg: vec2<u32>) {
+    let visited_index = w_visited_index_in_the_main_walk(device, slot);
+    if (visited_index == NONE || sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS) != 0u) {
+        return;
+    }
+    sws(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS, 2u);
+    sws(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 1u, birth_txg.x);
+    sws(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 2u, birth_txg.y);
+}
+
+// 正在走的这一版树表里一棵用户可见树的根记进版本表的待定那一行（版本表满了不记：这一版走完记它时判不了）
+fn appearance_note_a_user_visible_root_of_this_version(kind: u32, tree: vec2<u32>, root_pointer_birth_txg: vec2<u32>) {
+    if (appearance_version_count >= C_APPEARANCE_MAX_VERSIONS) {
+        return;
+    }
+    if (appearance_root_count_of_this_version >= C_APPEARANCE_MAX_USER_VISIBLE_ROOTS) {
+        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (41u << 8u);
+        return;
+    }
+    let row = C_APPEARANCE_VERSIONS + appearance_version_count * C_APPEARANCE_VERSION_WORDS
+        + C_APPEARANCE_VERSION_ROOTS_OFFSET + appearance_root_count_of_this_version * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
+    sws(row, kind);
+    sws(row + 1u, tree.x);
+    sws(row + 2u, tree.y);
+    sws(row + 3u, root_pointer_birth_txg.x);
+    sws(row + 4u, root_pointer_birth_txg.y);
+    appearance_root_count_of_this_version = appearance_root_count_of_this_version + 1u;
+}
+
+// 这一版第一个读得出的树表（walk_the_entries_of_a_tree_table_walked_first_by_this_version）：树表里每棵根指针非全零的用户可见树
+// （条目窄于 200 字节的不算，tree_table_entry_of_the_walk 交 None）记成这一版的根，连同树表落点记进走过的树表那张表
+fn appearance_note_the_user_visible_roots_of_a_tree_table_walked_first(tt: Node, tree_table_pointer: Ptr) {
+    if (appearance_walked_tree_table_count >= C_APPEARANCE_MAX_WALKED_TREE_TABLES) {
+        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (42u << 8u);
+        return;
+    }
+    let table_row = C_APPEARANCE_WALKED_TREE_TABLES + appearance_walked_tree_table_count * C_APPEARANCE_WALKED_TREE_TABLE_WORDS;
+    sws(table_row, tree_table_pointer.device0);
+    sws(table_row + 1u, tree_table_pointer.slot0.x);
+    sws(table_row + 2u, tree_table_pointer.slot0.y);
+    var roots = 0u;
+    if (tt.entry_width >= 200u) {
+        for (var e = 0u; e < tt.entry_count; e = e + 1u) {
+            let o = entry_offset(tt, e);
+            let kind = rd16(tt.base, o + 10u);
+            let root_pointer = parse_node_pointer(tt.base, o + 14u);
+            if (!appearance_tree_kind_is_user_visible(kind) || root_pointer.all_zero) {
+                continue;
+            }
+            if (roots >= C_APPEARANCE_MAX_USER_VISIBLE_ROOTS) {
+                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (46u << 8u);
+                return;
+            }
+            let root_row = table_row + C_APPEARANCE_WALKED_TREE_TABLE_ROOTS_OFFSET + roots * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
+            let tree = rd64(tt.base, o);
+            sws(root_row, kind);
+            sws(root_row + 1u, tree.x);
+            sws(root_row + 2u, tree.y);
+            sws(root_row + 3u, root_pointer.birth_txg.x);
+            sws(root_row + 4u, root_pointer.birth_txg.y);
+            roots = roots + 1u;
+            appearance_note_a_user_visible_root_of_this_version(kind, tree, root_pointer.birth_txg);
+        }
+    }
+    sws(table_row + 3u, roots);
+    appearance_walked_tree_table_count = appearance_walked_tree_table_count + 1u;
+}
+
+// 这一版的树表别的版本先走过（judge_the_tree_table_entries_an_earlier_version_walked）：这一版的用户可见树的根照先走那一版记下的取；
+// 先走那一版头用不了、没记下的，这里没有对象
+fn appearance_take_the_user_visible_roots_of_a_tree_table_walked_earlier(tree_table_pointer: Ptr) {
+    for (var t = 0u; t < appearance_walked_tree_table_count; t = t + 1u) {
+        let table_row = C_APPEARANCE_WALKED_TREE_TABLES + t * C_APPEARANCE_WALKED_TREE_TABLE_WORDS;
+        if (sw(table_row) != tree_table_pointer.device0 || sw(table_row + 1u) != tree_table_pointer.slot0.x || sw(table_row + 2u) != tree_table_pointer.slot0.y) {
+            continue;
+        }
+        let roots = sw(table_row + 3u);
+        for (var r = 0u; r < roots; r = r + 1u) {
+            let root_row = table_row + C_APPEARANCE_WALKED_TREE_TABLE_ROOTS_OFFSET + r * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
+            appearance_note_a_user_visible_root_of_this_version(sw(root_row), vec2<u32>(sw(root_row + 1u), sw(root_row + 2u)), vec2<u32>(sw(root_row + 3u), sw(root_row + 4u)));
+        }
+        return;
+    }
+}
+
+// 走树表的第 e 条之前（walk_the_entries_of_a_tree_table_walked_first_by_this_version 的循环）：条目是一棵根指针非全零的用户可见树时，
+// 之后跟着指针记下的落点打上那棵树；不是就不打
+fn appearance_begin_tree_table_entry(tt: Node, e: u32) {
+    appearance_user_visible_tree_is_being_walked = false;
+    if (tt.entry_width < 200u) {
+        return;
+    }
+    let o = entry_offset(tt, e);
+    let kind = rd16(tt.base, o + 10u);
+    if (!appearance_tree_kind_is_user_visible(kind) || parse_node_pointer(tt.base, o + 14u).all_zero) {
+        return;
+    }
+    appearance_user_visible_tree_is_being_walked = true;
+    appearance_user_visible_tree_kind = kind;
+    appearance_user_visible_tree = rd64(tt.base, o);
+}
+
+// note_the_version_just_walked_for_the_appearance_rule：一版走完，把它此刻引用过的落点（这一版的集合，w_judge_placement_in_version 记的）
+// 换成落点表的下标记进版本表，连同实例、txg、走读这一版有没有记失败
+fn appearance_note_the_version_just_walked(instance: u32, txg: vec2<u32>, references_are_complete: bool) {
+    if (appearance_version_count >= C_APPEARANCE_MAX_VERSIONS) {
+        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (43u << 8u);
+        return;
+    }
+    let row = C_APPEARANCE_VERSIONS + appearance_version_count * C_APPEARANCE_VERSION_WORDS;
+    sws(row, instance);
+    sws(row + 1u, txg.x);
+    sws(row + 2u, txg.y);
+    sws(row + 3u, select(0u, 1u, references_are_complete));
+    sws(row + 4u, appearance_version_placement_count);
+    let count = w_in_version_count[0];
+    for (var i = 0u; i < count; i = i + 1u) {
+        let placement = appearance_placement_index(sw(C_IN_VERSION0 + i * 3u), vec2<u32>(sw(C_IN_VERSION0 + i * 3u + 1u), sw(C_IN_VERSION0 + i * 3u + 2u)));
+        // 这一版引用过的每个落点都经 w_note_pointer_references 记过（这一版或先走的那一版）：不在说明落点表与这一版的集合脱节了
+        if (placement == NONE || appearance_version_placement_count >= C_APPEARANCE_MAX_VERSION_PLACEMENTS) {
+            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (44u << 8u);
+            return;
+        }
+        sws(C_APPEARANCE_VERSION_PLACEMENTS + appearance_version_placement_count, placement);
+        appearance_version_placement_count = appearance_version_placement_count + 1u;
+    }
+    sws(row + 5u, count);
+    sws(row + 6u, appearance_root_count_of_this_version);
+    appearance_version_count = appearance_version_count + 1u;
+}
+
 // 主走读里 (盘, 槽) 在走过的单元表里的下标，不在就是 NONE
 fn w_visited_index_in_the_main_walk(device: u32, slot: vec2<u32>) -> u32 {
     for (var i = 0u; i < w_visited_count[0]; i = i + 1u) {
@@ -833,6 +1034,11 @@ fn w_start(context: u32, judging: bool) {
     w_mismatches = 0u;
     w_parent_valid = false;
     if (context == 0u) {
+        appearance_placement_count = 0u;
+        appearance_version_count = 0u;
+        appearance_version_placement_count = 0u;
+        appearance_walked_tree_table_count = 0u;
+        appearance_root_count_of_this_version = 0u;
         w_leaf_count = 0u;
         w_checksum_count = 0u;
         w_tree_id_count = 0u;
@@ -850,6 +1056,10 @@ fn w_start(context: u32, judging: bool) {
 fn w_begin_version() {
     w_in_version_count[w_context] = 0u;
     w_parent_valid = false;
+    appearance_user_visible_tree_is_being_walked = false;
+    if (w_context == 0u) {
+        appearance_root_count_of_this_version = 0u;
+    }
 }
 
 // 引用集合：(盘, 起点槽, 跨度) 去重记进；位置项校验和记进主走读的表或收割表
@@ -1022,6 +1232,10 @@ fn w_merge_below(device: u32, slot: vec2<u32>) {
 
 // 走读跟着一条指针往下：两条位置条目记进引用集合，判同一版第二次引用，记到父单元下面
 fn w_note_pointer_references(p: Ptr, span: u32) {
+    if (w_context == 0u) {
+        appearance_note_placement(p.device0, p.slot0, p.device0, p.slot0);
+        appearance_note_placement(p.device1, p.slot1, p.device0, p.slot0);
+    }
     w_note_reference(p.device0, p.slot0, p.sum0, span);
     w_judge_placement_in_version(p.device0, p.slot0);
     w_record_below(p.device0, p.slot0);
@@ -1358,6 +1572,9 @@ fn w_walk_instance_table_chain(first: Ptr, judge_rows: bool, fsid_low: vec2<u32>
         if (!packed.ok) {
             break;
         }
+        if (w_context == 0u) {
+            appearance_note_the_birth_txg_of_an_instance_table_page(pointer.device0, pointer.slot0, rd64(s.base, 73u));
+        }
         let p = packed.p;
         // I-1.12：指针的出生树等于被指单元头偏移 43 的出生树（码 3）
         w_judge(INVARIANT_I_1_12, eq64(pointer.birth_tree, p.birth_tree));
@@ -2596,33 +2813,47 @@ fn w_walk_tree_table_entry(tt: Node, e: u32, fsid_low: vec2<u32>) {
 }
 
 // ── 一版的树表与中央映射树根（walk_tree_table_and_central_mapping_root）
+// 树表别的版本先走过（零单元发布、卸载与抬 F 的空发布照抄上一版的树表与映射根）：条目不重走，这一版的用户可见树的根照先走那一版记下的取，
+// 映射根照样往下走（共享的就并进这一版，I-3.5 比相邻两版的引用集合要它）；树表读不出、头用不了时映射根也不走
 fn w_walk_tree_table_and_mapping(tree_table_pointer: Ptr, mapping_root_pointer: Ptr, fsid_low: vec2<u32>) {
+    let tree_table_walked_by_an_earlier_version = !tree_table_pointer.all_zero && w_visited_contains(tree_table_pointer.device0, tree_table_pointer.slot0);
     let read = w_read_index_node(tree_table_pointer, vec2<u32>(0u, 0u), 2u, 0u, false, fsid_low);
-    if (!read.ok) {
+    if (read.ok) {
+        let tt = read.n;
+        w_judge_tree_table_entries_ordering(tt);
+        w_judge_tree_table_entry_head_identifiers(tt);
+        if (w_context == 0u) {
+            appearance_note_the_user_visible_roots_of_a_tree_table_walked_first(tt, tree_table_pointer);
+        }
+        let parent_valid = w_parent_valid;
+        let parent_device = w_parent_device;
+        let parent_slot = w_parent_slot;
+        w_parent_valid = true;
+        w_parent_device = tree_table_pointer.device0;
+        w_parent_slot = tree_table_pointer.slot0;
+        for (var e = 0u; e < tt.entry_count; e = e + 1u) {
+            appearance_begin_tree_table_entry(tt, e);
+            w_walk_tree_table_entry(tt, e, fsid_low);
+            appearance_user_visible_tree_is_being_walked = false;
+        }
+        w_parent_valid = parent_valid;
+        w_parent_device = parent_device;
+        w_parent_slot = parent_slot;
+    } else if (tree_table_walked_by_an_earlier_version) {
+        if (w_context == 0u) {
+            appearance_take_the_user_visible_roots_of_a_tree_table_walked_earlier(tree_table_pointer);
+        }
+    } else {
         return;
     }
-    let tt = read.n;
-    w_judge_tree_table_entries_ordering(tt);
-    w_judge_tree_table_entry_head_identifiers(tt);
-    let parent_valid = w_parent_valid;
-    let parent_device = w_parent_device;
-    let parent_slot = w_parent_slot;
-    w_parent_valid = true;
-    w_parent_device = tree_table_pointer.device0;
-    w_parent_slot = tree_table_pointer.slot0;
-    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
-        w_walk_tree_table_entry(tt, e, fsid_low);
-    }
-    w_parent_valid = parent_valid;
-    w_parent_device = parent_device;
-    w_parent_slot = parent_slot;
     let mapping = w_walk_code_two_tree(mapping_root_pointer, mapping_root_pointer.birth_tree, 1u, fsid_low);
     w_walk_central_mapping_entries(mapping.entries, fsid_low);
 }
 
 // ── 一条根（walk_root）
 // 走一遍：一条根（WALK_KIND_ROOT：实例表与分配记录树、树表与映射都取自这条根记录），或一版由记录施加出来、根槽从没落盘的版本
-// （WALK_KIND_VERSION：树表与映射取自那条记录，实例表与分配记录树取自它施加在其上的那条根；没有那条根就不走这两样）。
+// （WALK_KIND_VERSION：四条指针都取自那条记录的新根段——施加一条记录 = 把所选根的七个字段换成记录里的，D23（journal 的角色与格式）
+// 已定项 15，用户 2026-10-02「回到甲」；walk.rs 的 walk_version_applied_only_by_records 同一个取法）。
 // 全内核只许有一处调它（walk_and_hand_off 的循环里）：显卡上的程序没有真的函数调用，驱动的编译器把每个调用点各展开一份，
 // 这一整棵展开之后三十多万记号。原来五处走根、一处走版本各展开一份时，走读内核展开后 242 万记号，本机（驱动 595 版）冷编 523 秒、
 // 常驻 4.1 GiB，第二台（驱动 580 版）在 48G 里编不完；只留一处之后本机 18 秒、0.8 GiB。
@@ -2630,27 +2861,275 @@ fn w_walk_tree_table_and_mapping(tree_table_pointer: Ptr, mapping_root_pointer:
 const WALK_KIND_ROOT: u32 = 0u;
 const WALK_KIND_VERSION: u32 = 1u;
 
-fn w_walk_one(kind: u32, record_base: u32, applied_on_base: u32, is_newest: bool, fsid_low: vec2<u32>) {
+fn w_walk_one(kind: u32, record_base: u32, is_newest: bool, fsid_low: vec2<u32>) {
     w_begin_version();
     w_accounting_recording = is_newest;
-    var carrier_base = record_base;
     var tree_table_pointer_offset = 36u;
     var mapping_root_pointer_offset = 256u;
+    var instance_table_pointer_offset = 170u;
+    var allocation_root_pointer_offset = 342u;
     if (kind == WALK_KIND_VERSION) {
-        carrier_base = applied_on_base;
-        tree_table_pointer_offset = 99u;
-        mapping_root_pointer_offset = 185u;
+        tree_table_pointer_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET;
+        mapping_root_pointer_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 86u;
+        instance_table_pointer_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER;
+        allocation_root_pointer_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER;
     }
-    if (carrier_base != NONE) {
-        w_walk_instance_table_chain(parse_node_pointer(carrier_base, 170u), is_newest, fsid_low);
-        let allocation_root = parse_node_pointer(carrier_base, 342u);
-        if (!allocation_root.all_zero) {
-            w_walk_allocation_record_tree(allocation_root, vec2<u32>(0u, 0u), fsid_low);
-        }
+    w_walk_instance_table_chain(parse_node_pointer(record_base, instance_table_pointer_offset), is_newest, fsid_low);
+    let allocation_root = parse_node_pointer(record_base, allocation_root_pointer_offset);
+    if (!allocation_root.all_zero) {
+        w_walk_allocation_record_tree(allocation_root, vec2<u32>(0u, 0u), fsid_low);
     }
     w_walk_tree_table_and_mapping(parse_node_pointer(record_base, tree_table_pointer_offset), parse_node_pointer(record_base, mapping_root_pointer_offset), fsid_low);
 }
 
+// ───────────────────────── I-3.5（引用区间的精确性）的出现规则：版本都走完之后逐对判 ─────────────────────────
+// walk.rs 的 HoleInTheCandidateSet
+const APPEARANCE_NO_HOLE: u32 = 0u;
+const APPEARANCE_HOLE_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS: u32 = 1u;
+const APPEARANCE_HOLE_NOT_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS: u32 = 2u;
+// 判一版时它去重之后的单元（版本表一行里的落点折成单元）摊在这里：每个单元 2 个字（落点表下标、走过的单元表下标或 NONE）。
+// 借 w_merge_below 的两块（C_PENDING 与紧跟其后的 C_EXPANDED），判的时候主走读那几版已经走完、不再并入
+const APPEARANCE_UNIT_WORDS: u32 = 2u;
+const APPEARANCE_UNITS: u32 = C_PENDING;
+const APPEARANCE_MAX_UNITS: u32 = (C_SCRATCH_END - C_PENDING) / APPEARANCE_UNIT_WORDS;
+
+fn appearance_version_row(version_index: u32) -> u32 {
+    return C_APPEARANCE_VERSIONS + version_index * C_APPEARANCE_VERSION_WORDS;
+}
+
+fn appearance_version_txg(version_index: u32) -> vec2<u32> {
+    let row = appearance_version_row(version_index);
+    return vec2<u32>(sw(row + 1u), sw(row + 2u));
+}
+
+// 落点表一行的 (盘, 槽) 比另一行的大（BTreeMap 的键序：先盘、后槽）
+fn appearance_placement_is_greater(index: u32, other: u32) -> bool {
+    let row = C_APPEARANCE_PLACEMENTS + index * C_APPEARANCE_PLACEMENT_WORDS;
+    let other_row = C_APPEARANCE_PLACEMENTS + other * C_APPEARANCE_PLACEMENT_WORDS;
+    if (sw(row) != sw(other_row)) {
+        return sw(row) > sw(other_row);
+    }
+    return lt64(vec2<u32>(sw(other_row + 1u), sw(other_row + 2u)), vec2<u32>(sw(row + 1u), sw(row + 2u)));
+}
+
+// 两行落点折成的是同一个单元
+fn appearance_same_unit(index: u32, other: u32) -> bool {
+    let row = C_APPEARANCE_PLACEMENTS + index * C_APPEARANCE_PLACEMENT_WORDS;
+    let other_row = C_APPEARANCE_PLACEMENTS + other * C_APPEARANCE_PLACEMENT_WORDS;
+    return sw(row + 3u) == sw(other_row + 3u) && sw(row + 4u) == sw(other_row + 4u) && sw(row + 5u) == sw(other_row + 5u);
+}
+
+// 落点表一行打的用户可见树标签是不是 (kind, tree)
+fn appearance_placement_is_in_tree(index: u32, kind: u32, tree: vec2<u32>) -> bool {
+    let row = C_APPEARANCE_PLACEMENTS + index * C_APPEARANCE_PLACEMENT_WORDS;
+    return sw(row + 6u) == 1u && sw(row + 7u) == kind && sw(row + 8u) == tree.x && sw(row + 9u) == tree.y;
+}
+
+// 一版去重之后的单元摊进 APPEARANCE_UNITS，交回几个（判不了交回 NONE）。note_the_version_just_walked_for_the_appearance_rule 把这一版的落点
+// 按 (盘, 槽) 升序折成单元收进 BTreeMap，同一个单元后收的标签盖掉先收的：这里每个单元留它 (盘, 槽) 最大的那个落点（标签取那一个的）
+fn appearance_spread_the_units_of_a_version(version_index: u32) -> u32 {
+    let row = appearance_version_row(version_index);
+    let first = sw(row + 4u);
+    let count = sw(row + 5u);
+    var units = 0u;
+    for (var k = 0u; k < count; k = k + 1u) {
+        let placement = sw(C_APPEARANCE_VERSION_PLACEMENTS + first + k);
+        var a_greater_placement_of_the_same_unit = false;
+        for (var j = 0u; j < count; j = j + 1u) {
+            let other = sw(C_APPEARANCE_VERSION_PLACEMENTS + first + j);
+            if (j != k && appearance_same_unit(placement, other) && appearance_placement_is_greater(other, placement)) {
+                a_greater_placement_of_the_same_unit = true;
+                break;
+            }
+        }
+        if (a_greater_placement_of_the_same_unit) {
+            continue;
+        }
+        if (units >= APPEARANCE_MAX_UNITS) {
+            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (45u << 8u);
+            return NONE;
+        }
+        let placement_row = C_APPEARANCE_PLACEMENTS + placement * C_APPEARANCE_PLACEMENT_WORDS;
+        sws(APPEARANCE_UNITS + units * APPEARANCE_UNIT_WORDS, placement);
+        sws(APPEARANCE_UNITS + units * APPEARANCE_UNIT_WORDS + 1u,
+            w_visited_index_in_the_main_walk(sw(placement_row + 3u), vec2<u32>(sw(placement_row + 4u), sw(placement_row + 5u))));
+        units = units + 1u;
+    }
+    return units;
+}
+
+// 摊开的第 u 个单元有没有记下诞生代号（被引用单元那一支标记 1、实例表一片标记 2；头读不出的没有）
+fn appearance_unit_birth_is_known(u: u32) -> bool {
+    let visited_index = sw(APPEARANCE_UNITS + u * APPEARANCE_UNIT_WORDS + 1u);
+    return visited_index != NONE && sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS) != 0u;
+}
+
+fn appearance_unit_birth_txg(u: u32) -> vec2<u32> {
+    let visited_index = sw(APPEARANCE_UNITS + u * APPEARANCE_UNIT_WORDS + 1u);
+    return vec2<u32>(sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 1u), sw(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 2u));
+}
+
+// txg 为 previous_txg 的那几版（R⁻ 那一组）里有没有一版引用了与落点表第 placement 行同一个单元
+fn appearance_previous_group_references_the_unit_of(previous_txg: vec2<u32>, placement: u32) -> bool {
+    for (var other = 0u; other < appearance_version_count; other = other + 1u) {
+        if (!eq64(appearance_version_txg(other), previous_txg)) {
+            continue;
+        }
+        let row = appearance_version_row(other);
+        let first = sw(row + 4u);
+        let count = sw(row + 5u);
+        for (var k = 0u; k < count; k = k + 1u) {
+            if (appearance_same_unit(sw(C_APPEARANCE_VERSION_PLACEMENTS + first + k), placement)) {
+                return true;
+            }
+        }
+    }
+    return false;
+}
+
+// walk.rs hole_in_the_candidate_set_between：R⁻ 那一组（txg previous_txg 的那几版）与这一版 (version_instance, version_txg) 之间的洞；
+// 接得上 ⟺ R⁻ 的某个实例 i：最新根的实例表里有一行 (i, T = R⁻.txg)，这一版的实例 v > i，i 与 v 之间每个实例都有一行、每一行都是 (j, 0, 0)
+fn appearance_hole_between(previous_txg: vec2<u32>, version_txg: vec2<u32>, version_instance: u32) -> u32 {
+    let previous_txg_has_a_successor = !(previous_txg.x == NONE && previous_txg.y == NONE);
+    if (previous_txg_has_a_successor && eq64(add64(previous_txg, u64_of(1u)), version_txg)) {
+        return APPEARANCE_NO_HOLE;
+    }
+    for (var other = 0u; other < appearance_version_count; other = other + 1u) {
+        if (!eq64(appearance_version_txg(other), previous_txg)) {
+            continue;
+        }
+        let previous_instance = sw(appearance_version_row(other));
+        if (version_instance <= previous_instance) {
+            continue;
+        }
+        var row_of_the_previous_version = false;
+        var instances_between = 0u;
+        var every_row_between_published_nothing = true;
+        for (var k = 0u; k < w_instance_row_count; k = k + 1u) {
+            let row = C_INSTANCE_ROWS + k * C_ROW_WORDS;
+            let instance = sw(row);
+            let row_txg = vec2<u32>(sw(row + 1u), sw(row + 2u));
+            let row_high_water = vec2<u32>(sw(row + 3u), sw(row + 4u));
+            if (instance == previous_instance && eq64(row_txg, previous_txg)) {
+                row_of_the_previous_version = true;
+            }
+            if (instance <= previous_instance || instance >= version_instance) {
+                continue;
+            }
+            if (!is_zero64(row_txg) || !is_zero64(row_high_water)) {
+                every_row_between_published_nothing = false;
+            }
+            // 中间的实例按不同的代号数（同一个实例有两行只算一个）
+            var counted_before = false;
+            for (var j = 0u; j < k; j = j + 1u) {
+                if (sw(C_INSTANCE_ROWS + j * C_ROW_WORDS) == instance) {
+                    counted_before = true;
+                    break;
+                }
+            }
+            if (!counted_before) {
+                instances_between = instances_between + 1u;
+            }
+        }
+        if (row_of_the_previous_version && instances_between == version_instance - previous_instance - 1u && every_row_between_published_nothing) {
+            return APPEARANCE_HOLE_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS;
+        }
+    }
+    return APPEARANCE_HOLE_NOT_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS;
+}
+
+// 这一版（摊开在 APPEARANCE_UNITS 里，units 个）树表里第 root_index 棵用户可见树是不是整棵继承下来的：根指针出生 txg 严格小于 R⁻.txg，
+// 且这一版里那棵树下读得出的诞生代号都不大于它
+fn appearance_tree_is_inherited_whole(version_index: u32, root_index: u32, units: u32, previous_txg: vec2<u32>) -> bool {
+    let root_row = appearance_version_row(version_index) + C_APPEARANCE_VERSION_ROOTS_OFFSET + root_index * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
+    let kind = sw(root_row);
+    let tree = vec2<u32>(sw(root_row + 1u), sw(root_row + 2u));
+    let root_pointer_birth_txg = vec2<u32>(sw(root_row + 3u), sw(root_row + 4u));
+    if (!lt64(root_pointer_birth_txg, previous_txg)) {
+        return false;
+    }
+    for (var u = 0u; u < units; u = u + 1u) {
+        if (!appearance_placement_is_in_tree(sw(APPEARANCE_UNITS + u * APPEARANCE_UNIT_WORDS), kind, tree) || !appearance_unit_birth_is_known(u)) {
+            continue;
+        }
+        if (!le64(appearance_unit_birth_txg(u), root_pointer_birth_txg)) {
+            return false;
+        }
+    }
+    return true;
+}
+
+// walk.rs judge_the_appearance_rule_of_adjacent_candidate_versions：回退候选集（走过的每一版）按 txg 升序、同 txg 并成一组，
+// 每一版 R 与 txg 比它小的那几版里 txg 最大的那一组 R⁻ 比：R 引用而 R⁻ 不引用、诞生代号读得出的每个单元 b 满足
+// (a) birth(b) > R⁻.txg，或 (b) b 打着一棵整棵继承下来的用户可见树的标签。有洞而实例表行接不上时打着用户可见树标签的不判（N3）；
+// R 或 R⁻ 那一组有一版走读记过失败的那一对不判。一个单元都没判到时 CPU 报不适用，违例位上与成立一样是 0
+fn judge_the_appearance_rule_of_adjacent_candidate_versions() {
+    for (var version_index = 0u; version_index < appearance_version_count; version_index = version_index + 1u) {
+        let row = appearance_version_row(version_index);
+        let version_txg = appearance_version_txg(version_index);
+        var has_previous = false;
+        var previous_txg = vec2<u32>(0u, 0u);
+        for (var other = 0u; other < appearance_version_count; other = other + 1u) {
+            let other_txg = appearance_version_txg(other);
+            if (lt64(other_txg, version_txg) && (!has_previous || lt64(previous_txg, other_txg))) {
+                previous_txg = other_txg;
+                has_previous = true;
+            }
+        }
+        if (!has_previous || sw(row + 3u) != 1u) {
+            continue;
+        }
+        var previous_references_are_complete = true;
+        for (var other = 0u; other < appearance_version_count; other = other + 1u) {
+            if (eq64(appearance_version_txg(other), previous_txg) && sw(appearance_version_row(other) + 3u) != 1u) {
+                previous_references_are_complete = false;
+            }
+        }
+        if (!previous_references_are_complete) {
+            continue;
+        }
+        let hole = appearance_hole_between(previous_txg, version_txg, sw(row));
+        let units = appearance_spread_the_units_of_a_version(version_index);
+        if (units == NONE) {
+            return;
+        }
+        // 这一版里整棵继承下来的用户可见树（第 r 棵是就置位 r）
+        var trees_inherited_whole = 0u;
+        let roots = sw(row + 6u);
+        for (var r = 0u; r < roots; r = r + 1u) {
+            if (appearance_tree_is_inherited_whole(version_index, r, units, previous_txg)) {
+                trees_inherited_whole = trees_inherited_whole | (1u << r);
+            }
+        }
+        for (var u = 0u; u < units; u = u + 1u) {
+            let placement = sw(APPEARANCE_UNITS + u * APPEARANCE_UNIT_WORDS);
+            if (appearance_previous_group_references_the_unit_of(previous_txg, placement)) {
+                continue;
+            }
+            let placement_row = C_APPEARANCE_PLACEMENTS + placement * C_APPEARANCE_PLACEMENT_WORDS;
+            let in_a_user_visible_tree = sw(placement_row + 6u) == 1u;
+            if (hole == APPEARANCE_HOLE_NOT_BRIDGED_BY_THE_INSTANCE_TABLE_ROWS && in_a_user_visible_tree) {
+                continue;
+            }
+            if (!appearance_unit_birth_is_known(u)) {
+                continue;
+            }
+            let born_after_the_previous_version = lt64(previous_txg, appearance_unit_birth_txg(u));
+            var in_a_tree_inherited_whole = false;
+            for (var r = 0u; r < roots; r = r + 1u) {
+                let root_row = row + C_APPEARANCE_VERSION_ROOTS_OFFSET + r * C_APPEARANCE_USER_VISIBLE_ROOT_WORDS;
+                if ((trees_inherited_whole & (1u << r)) != 0u
+                    && appearance_placement_is_in_tree(placement, sw(root_row), vec2<u32>(sw(root_row + 1u), sw(root_row + 2u)))) {
+                    in_a_tree_inherited_whole = true;
+                }
+            }
+            if (!(born_after_the_previous_version || in_a_tree_inherited_whole)) {
+                violate(INVARIANT_I_3_5);
+            }
+        }
+    }
+}
+
 // ───────────────────────── 第二批：走读与走读之后（check_pool_image 择根之后那一段）─────────────────────────
 
 // 恢复眼里的 journal 环：每个计数器取按盘次序第一块盘上自证过的那一份（records_as_the_recovery_reads_them）
@@ -2937,7 +3416,6 @@ fn walk_and_hand_off(g: CheckerSlot, fsid_low: vec2<u32>) -> u32 {
         var found = false;
         var walk_kind = WALK_KIND_ROOT;
         var walk_base = NONE;
-        var walk_applied_on = NONE;
         var walk_is_newest = false;
         if (phase == WALK_PHASE_NEWEST_ROOT) {
             walk_base = newest_root.base;
@@ -3048,23 +3526,14 @@ fn walk_and_hand_off(g: CheckerSlot, fsid_low: vec2<u32>) -> u32 {
             if (cursor < version_count) {
                 let r = checker_record_at(sw(R_DATA + cursor));
                 cursor = cursor + 1u;
-                // 它施加在其上那一版的根：同实例、txg 比它小、txg 最大的那一条
-                var applied_on = NONE;
-                var applied_on_txg = vec2<u32>(0u, 0u);
-                for (var k = 0u; k < root_count; k = k + 1u) {
-                    let root = checker_root_of(k);
-                    if (root.instance == r.instance && lt64(root.txg, r.txg)) {
-                        if (applied_on == NONE || lt64(applied_on_txg, root.txg)) {
-                            applied_on = root.base;
-                            applied_on_txg = root.txg;
-                        }
-                    }
-                }
                 walk_kind = WALK_KIND_VERSION;
                 walk_base = r.base;
-                walk_applied_on = applied_on;
                 found = true;
             } else {
+                // I-3.5：回退候选集的每一版（最新根、候选根、由记录施加出来的版本）都走完了，逐对判（check_pool_image 走完它们之后那一处）
+                if (undecidable == 0u) {
+                    judge_the_appearance_rule_of_adjacent_candidate_versions();
+                }
                 phase = WALK_PHASE_ABANDONED_ROOTS;
                 cursor = 0u;
             }
@@ -3275,7 +3744,6 @@ fn walk_and_hand_off(g: CheckerSlot, fsid_low: vec2<u32>) -> u32 {
                 w_harvesting = false;
                 walk_kind = WALK_KIND_VERSION;
                 walk_base = checker_record_of_row(C_REPLAY_RECORDS + last_applied * C_REPLAY_RECORD_WORDS).base;
-                walk_applied_on = newest_root.base;
                 found = true;
             }
         }
@@ -3291,8 +3759,16 @@ fn walk_and_hand_off(g: CheckerSlot, fsid_low: vec2<u32>) -> u32 {
         }
         let mismatches_before = w_mismatches;
         let failures_before = w_failures;
-        w_walk_one(walk_kind, walk_base, walk_applied_on, walk_is_newest, fsid_low);
+        w_walk_one(walk_kind, walk_base, walk_is_newest, fsid_low);
         w_version_is_an_older_candidate_root = false;
+        // I-3.5：主走读走的那几版（最新根、候选根、由记录施加出来的版本）各记一份它引用了哪些单元；这一版的实例与 txg：根记录偏移 24 / 28，记录偏移 16 / 26
+        if (phase == WALK_PHASE_NEWEST_ROOT || phase == WALK_PHASE_CANDIDATE_ROOTS || phase == WALK_PHASE_VERSIONS_APPLIED_ONLY_BY_RECORDS) {
+            var walked_instance = rd32(walk_base, 24u);
+            if (walk_kind == WALK_KIND_VERSION) {
+                walked_instance = rd32(walk_base, 16u);
+            }
+            appearance_note_the_version_just_walked(walked_instance, w_version_txg, w_failures == failures_before);
+        }
         // ③ 走完之后这一阶段要做的
         if (phase == WALK_PHASE_NEWEST_ROOT) {
             c_newest_reference_count = w_reference_count[0];
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
index 9c741b7..8e612da 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
@@ -137,7 +137,10 @@ const INVARIANT_I_7_5: u32 = 63u;
 
 const INVARIANT_I_8_5: u32 = 64u;
 
-// 第 67、68 条：C673 ⑤ ⑥ 那两条（只往末尾追加；下标 65 的 I-3.5 内核还没判，这里没有它的常量）
+// 第 66 条：I-3.5（引用区间的精确性）的出现规则，走读内核在回退候选集的版本都走完之后逐对判
+const INVARIANT_I_3_5: u32 = 65u;
+
+// 第 67、68 条：C673 ⑤ ⑥ 那两条（只往末尾追加）
 const INVARIANT_I_9_27: u32 = 66u;
 
 const INVARIANT_I_5_5: u32 = 67u;
@@ -700,8 +703,47 @@ const C_REPLAY_RECORD_WORDS: u32 = 4u;
 
 const C_REPLAY_RECORDS: u32 = 253456u;
 
+// I-3.5（引用区间的精确性）的出现规则读的四张表，只记主走读（上下文 0）走的那几版（最新根、回退候选根、由记录施加出来的版本）。
+// ① 落点表（walk.rs 的 unit_of_each_placement 与 user_visible_tree_of_each_placement）：跟着指针记下的每个落点一行——
+//    落点盘、槽 lo/hi，它是哪个单元（第一次记下它的那条指针第 0 条位置条目的盘、槽 lo/hi），记下它那一刻走读在不在一棵用户可见树里
+//    （第一次在树里记下它时打上，之后不换）、那棵树的种类、树 ID lo/hi。
+const C_APPEARANCE_PLACEMENT_WORDS: u32 = 10u;
+
+const C_APPEARANCE_MAX_PLACEMENTS: u32 = 2048u;
+
+const C_APPEARANCE_PLACEMENTS: u32 = 255504u;
+
+// ② 版本表（walk.rs 的 versions_walked_for_the_appearance_rule）：走完的每一版一行——实例、txg lo/hi、走读一条失败都没记（1 / 0）、
+//    这一版引用的落点在 ③ 里的起点与条数、树表里用户可见树的根几棵，之后每棵 5 个字（种类、树 ID lo/hi、根指针出生 txg lo/hi）。
+const C_APPEARANCE_VERSION_WORDS: u32 = 48u;
+
+const C_APPEARANCE_MAX_USER_VISIBLE_ROOTS: u32 = 8u;
+
+const C_APPEARANCE_USER_VISIBLE_ROOT_WORDS: u32 = 5u;
+
+const C_APPEARANCE_VERSION_ROOTS_OFFSET: u32 = 7u;
+
+const C_APPEARANCE_MAX_VERSIONS: u32 = 64u;
+
+const C_APPEARANCE_VERSIONS: u32 = 275984u;
+
+// ③ 每一版引用的落点（walk.rs 的 placements_referenced_in_this_version 在这一版走完那一刻的样子）：每个落点一个字，是它在 ① 里的下标。
+const C_APPEARANCE_MAX_VERSION_PLACEMENTS: u32 = 8192u;
+
+const C_APPEARANCE_VERSION_PLACEMENTS: u32 = 279056u;
+
+// ④ 走过的树表（walk.rs 的 tree_table_entries_of_a_walked_tree_table 里 I-3.5 读的那一样）：第一个读得出它的那一版记下树表单元的盘、槽 lo/hi
+//    与它里面用户可见树的根几棵、每棵 5 个字（同 ②）；别的版本共享这片树表时照它取这一版的根。
+const C_APPEARANCE_WALKED_TREE_TABLE_WORDS: u32 = 44u;
+
+const C_APPEARANCE_WALKED_TREE_TABLE_ROOTS_OFFSET: u32 = 4u;
+
+const C_APPEARANCE_MAX_WALKED_TREE_TABLES: u32 = 64u;
+
+const C_APPEARANCE_WALKED_TREE_TABLES: u32 = 287248u;
+
 // 每个状态的草稿区到这里为止：crash_judge_gpu.rs 的 SCRATCH_WORDS 与它相等（那一份的单测核）
-const SCRATCH_END: u32 = 255504u;
+const SCRATCH_END: u32 = 290064u;
 
 fn handoff_region_device(region: u32) -> u32 {
     if (region == 0u) {
diff --git a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
index 56c538d..37eb55a 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -94,6 +94,11 @@ const JOURNAL_NAMED_ENTRY_BYTES: u32 = 56u;
 // 实例表指针 86、分配记录树根指针 86、卸载记号 4），fsid 8 紧跟新根段（偏移 463），头到 487 为止
 const JOURNAL_NEW_ROOT_SEGMENT_OFFSET: u32 = 99u;
 const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u32 = 364u;
+// 新根段里后三段的段内偏移（walk.rs 的 NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER、NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER、
+// NEW_ROOT_SEGMENT_UNMOUNT_MARKER）：施加一条记录取这三段（已定项 15），卸载记号只有位 0（root_record.rs 的 ROOT_RECORD_FLAG_UNMOUNT_MARKER）
+const JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER: u32 = 188u;
+const JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER: u32 = 274u;
+const JOURNAL_NEW_ROOT_SEGMENT_UNMOUNT_MARKER: u32 = 360u;
 const JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET: u32 = 463u;
 const JOURNAL_RING_START_BYTES: u32 = 16777216u;   // 1024 × 16384
 const SYSTEM_CONFIGURATION_SLOT_BYTES: u32 = 4096u;
@@ -952,7 +957,11 @@ fn invalid_record() -> Rec {
     r.named_count = 0u;
     return r;
 }
-// journal::JournalRecord::parse：magic、类型、整条校验和、标志、长度、提交标记、序号、新根段两条指针头部、fsid、载荷校验和
+// 记录新根段里一条指针的头（偏移 o 起 34 字节）是第一版的取值：head_ok 判的几段，加 extent 偏移（指针偏移 32 的 2 字节）为 0
+fn record_pointer_head_is_the_first_version(base: u32, o: u32) -> bool {
+    return parse_node_pointer(base, o).head_ok && rd16(base, o + 32u) == 0u;
+}
+// journal::JournalRecord::parse：magic、类型、整条校验和、标志、长度、提交标记、序号、新根段四条指针头部、卸载记号位 0 之外为 0、fsid、载荷校验和
 fn parse_record(s: Slice, fsid_low: vec2<u32>) -> Rec {
     if (s.version == NONE || s.length < JOURNAL_RECORD_BYTES) {
         return invalid_record();
@@ -976,9 +985,17 @@ fn parse_record(s: Slice, fsid_low: vec2<u32>) -> Rec {
     if (ordinal == 0u) {
         return invalid_record();
     }
-    let tree_table = parse_node_pointer(base, 99u);
-    let mapping_root = parse_node_pointer(base, 185u);
-    if (!tree_table.head_ok || !mapping_root.head_ok) {
+    // 新根段四条指针各按 NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from 判头：
+    // MAC / nonce、算法类型、压缩码、压缩长度、extent 偏移都是第一版的取值
+    if (!record_pointer_head_is_the_first_version(base, 99u) || !record_pointer_head_is_the_first_version(base, 185u)) {
+        return invalid_record();
+    }
+    if (!record_pointer_head_is_the_first_version(base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER)
+        || !record_pointer_head_is_the_first_version(base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER)) {
+        return invalid_record();
+    }
+    // UnmountMarker::of_unmount_marker_field_of_a_new_root_segment：位 0 之外有位为 1 当这条记录损坏
+    if ((rd32(base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_UNMOUNT_MARKER) & 0xFFFFFFFEu) != 0u) {
         return invalid_record();
     }
     if (!eq64(rd64(base, JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET), fsid_low)) {
@@ -1281,15 +1298,18 @@ fn replay_journal(chosen: RootFields, ring_bytes: vec2<u32>) -> Replay {
             continue;
         }
         open_count = 0u;
+        // 施加末条：所选根的七个字段换成记录新根段里的（recovery::replay_journal，已定项 15，用户 2026-10-02「回到甲」）；fsid 照所选根
         var next = rebuilt;
         next.valid = true;
-        next.unmount = false;
+        next.unmount = (rd32(r.base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_UNMOUNT_MARKER) & 1u) != 0u;
         next.instance = r.instance;
         next.txg = r.txg;
         next.tree_table = parse_node_pointer(r.base, 99u);
         next.watermark = rd64(r.base, 271u);
         next.floor = rd64(r.base, 279u);
+        next.instance_table = parse_node_pointer(r.base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER);
         next.mapping_root = parse_node_pointer(r.base, 185u);
+        next.allocation_root = parse_node_pointer(r.base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER);
         rebuilt = next;
         replay_stop_reason = 12u;
     }
diff --git a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
index 4100ad3..44cb400 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -879,14 +879,14 @@ mod tests {
     #[test]
     fn states_in_one_dispatch_follow_the_scratch_memory_up_to_the_lane_limit() {
         let bytes_per_state = u64::try_from(SCRATCH_WORDS * 4).expect("装得进 u64");
-        assert_eq!(states_per_scratch_lane(BINDING_LIMIT), 2170);
+        assert_eq!(states_per_scratch_lane(BINDING_LIMIT), 1850);
         assert_eq!(states_per_scratch_lane(1), 1, "再小的上限也带一个");
-        // 12 GiB 的草稿区：12 × 1024 × 1024 × 1024 ÷ 989 248 = 13 024 个（向下取整）
+        // 12 GiB 的草稿区：12 × 1024 × 1024 × 1024 ÷ 1 160 256 = 11 105 个（向下取整）
         assert_eq!(
             states_per_dispatch(BINDING_LIMIT, 12 * 1024),
             usize::try_from(12 * 1024 * 1024 * 1024 / bytes_per_state).expect("装得进 usize")
         );
-        assert_eq!(states_per_dispatch(BINDING_LIMIT, 12 * 1024), 13_024);
+        assert_eq!(states_per_dispatch(BINDING_LIMIT, 12 * 1024), 11_105);
         assert_eq!(
             states_per_dispatch(BINDING_LIMIT, 0),
             1,
@@ -894,7 +894,7 @@ mod tests {
         );
         assert_eq!(
             states_per_dispatch(BINDING_LIMIT, 1024 * 1024),
-            2170 * MAXIMUM_SCRATCH_LANES,
+            1850 * MAXIMUM_SCRATCH_LANES,
             "显存再多也只到条数上限"
         );
     }
@@ -1093,8 +1093,7 @@ mod tests {
     /// 走读内核里「走一遍根」回到六个调用点（第二台 580 版驱动 48G 里编不完的那个形态），编之前就被拒。
     #[test]
     fn the_walk_kernel_with_the_root_walk_called_from_six_sites_is_refused() {
-        const CALL: &str =
-            "w_walk_one(walk_kind, walk_base, walk_applied_on, walk_is_newest, fsid_low);";
+        const CALL: &str = "w_walk_one(walk_kind, walk_base, walk_is_newest, fsid_low);";
         let source = crate::crash_judge_gpu::checker_walk_kernel_source();
         assert_eq!(
             source.matches(CALL).count(),
diff --git a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
index 9d675b5..f267eee 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -3,7 +3,7 @@
 //! 七类 oracle、记录核对器两条判据，判法与 CPU 判器（`crate::crash::judge_crash_image`）逐条相同；GPU 与 CPU 逐状态比是它的判别力自证。
 //! 内核分五份源码：`crash_judge_common.wgsl`（各内核共用的解析与查表）、`crash_judge_recovery.wgsl`（恢复、oracle、记录核对器）、
 //! `crash_judge_checker_common.wgsl`（池级 checker 两个内核共用的读法与交接段）、`crash_judge_checker.wgsl`（走读内核：第一、二批不变量，
-//! 连同里程碑三二·1 那一批里的十五条与 I-9.27）、`crash_judge_checker_scan.wgsl`（扫描内核：第三批、I-3.12 与 I-5.5）；判不了的状态（表越过内核容量）置最高位、交 CPU 判。
+//! 连同里程碑三二·1 那一批里的十五条、I-9.27 与 I-3.5）、`crash_judge_checker_scan.wgsl`（扫描内核：第三批、I-3.12 与 I-5.5）；判不了的状态（表越过内核容量）置最高位、交 CPU 判。
 //!
 //! 一个状态的结论 [`VERDICT_WORDS`] 个 u32：第 0 字是红位（位 0..6 看 journal 那一遍 oracle 的七类、位 8..14 不看那一遍、
 //! 位 16 根在而记录都不在、位 17 恢复自称的状态少单元、位 31 判不了）；第 1 字判不了的原因（判得了时低 16 位是看 journal 那一遍
@@ -58,76 +58,32 @@ const _: () = assert!(
     "违例位读成一个 u128"
 );
 
-/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：前 65 条——三批 49 条，加里程碑三二·1 那一批 16 条
-/// （判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行），再加 C673（三 oracle 合谋测试的补 oracle 改法没做）
-/// ⑤ ⑥ 那两条（下标 66 是 I-9.27，走读内核挨着 I-9.16 按表判；下标 67 是 I-5.5，扫描内核每一版重读树表与 extent 树判），对拍逐位比、不屏蔽。
+/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 68 条——三批 49 条，加里程碑三二·1 那一批 16 条
+/// （判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行），下标 65 的 I-3.5（引用区间的精确性，
+/// 走读内核在回退候选集的版本都走完之后逐对判），再加 C673（三 oracle 合谋测试的补 oracle 改法没做）⑤ ⑥ 那两条
+/// （下标 66 是 I-9.27，走读内核挨着 I-9.16 按表判；下标 67 是 I-5.5，扫描内核每一版重读树表与 extent 树判），对拍逐位比、不屏蔽。
+/// 用户 2026-10-04 定「改回要求全覆盖」：没有「内核还没判」那一张表。
 pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
     26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
-    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67,
+    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67,
 ];
 
-/// 池级 checker 已经判、GPU 内核还没判的不变量：下标 65 是 I-3.5（引用区间的精确性，里程碑三第 8 批进池级 checker，
-/// 派发没含 GPU 内核那一半，交主 agent 定）。内核那一位恒 0：对拍逐位比照旧不屏蔽，CPU 在哪个状态上判红它，对拍就在那个状态上报不同；
-/// 崩溃放量由 GPU 判的那几趟判不到它（`crash_amplification` 按违例位取不变量）。
-pub const POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET: &[usize] = &[65];
-
-// 覆盖表与「内核还没判」那张表合起来是池级 checker 的每一条：每个下标恰好在其中一张表里、两张表各按下标升序。
-// 新立一条不变量而两张表都没跟上时这里编译不过，不悄悄让 GPU 那一位恒 0；还没判的那几条要在「内核还没判」那张表里点名。
-// 「内核还没判」那几条不必在覆盖表的末尾：I-3.5（下标 65）之后立的 I-9.27、I-5.5 内核判了，排在它后面。
+// 覆盖表就是池级 checker 的每一条，按下标升序：第 k 项是 k。新立一条不变量而内核没跟上时这里编译不过，不悄悄让 GPU 那一位恒 0。
 const _: () = {
     assert!(
         POOL_CHECKER_COVERED_INVARIANT_INDEXES.len()
-            + POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET.len()
             == singlefs_checker::image::IMPLEMENTED_INVARIANTS.len(),
-        "池级 checker 新立的不变量 GPU 内核还没判：补内核，再把下标补进覆盖表（暂不补内核的，点名进「内核还没判」那张表）"
+        "池级 checker 新立的不变量 GPU 内核还没判：补内核，再把下标补进覆盖表"
     );
-    let mut position = 1;
+    let mut position = 0;
     while position < POOL_CHECKER_COVERED_INVARIANT_INDEXES.len() {
         assert!(
-            POOL_CHECKER_COVERED_INVARIANT_INDEXES[position - 1]
-                < POOL_CHECKER_COVERED_INVARIANT_INDEXES[position],
-            "覆盖表按下标升序"
+            POOL_CHECKER_COVERED_INVARIANT_INDEXES[position] == position,
+            "覆盖表按下标升序、一条不缺一条不重：第 k 项是 k"
         );
         position += 1;
     }
-    let mut position_not_judged = 1;
-    while position_not_judged < POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET.len() {
-        assert!(
-            POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET[position_not_judged - 1]
-                < POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET[position_not_judged],
-            "「内核还没判」那张表按下标升序"
-        );
-        position_not_judged += 1;
-    }
-    // 每个下标恰好在一张表里：两张表都升序、合起来条数等于不变量条数，再要求每个下标至少在一张表里，就一条不缺、一条不重。
-    let mut invariant_index = 0;
-    while invariant_index < singlefs_checker::image::IMPLEMENTED_INVARIANTS.len() {
-        let mut is_listed = false;
-        let mut covered_position = 0;
-        while covered_position < POOL_CHECKER_COVERED_INVARIANT_INDEXES.len() {
-            if POOL_CHECKER_COVERED_INVARIANT_INDEXES[covered_position] == invariant_index {
-                is_listed = true;
-            }
-            covered_position += 1;
-        }
-        let mut not_judged_position = 0;
-        while not_judged_position
-            < POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET.len()
-        {
-            if POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET[not_judged_position]
-                == invariant_index
-            {
-                is_listed = true;
-            }
-            not_judged_position += 1;
-        }
-        assert!(
-            is_listed,
-            "池级 checker 的每一条要么在覆盖表里、要么在「内核还没判」那张表里"
-        );
-        invariant_index += 1;
-    }
 };
 
 /// 一个状态结论里池级 checker 的违例位（[`POOL_CHECKER_VIOLATED_BIT_CAPACITY`] 位，位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）。
@@ -154,8 +110,8 @@ pub fn verdict_layout_wgsl_constants() -> String {
     )
 }
 /// 每个状态的草稿区几个 u32（checker 共用那一半里 `SCRATCH_END`：共用的 `SCRATCH_WORDS_REQUIRED`、走读与扫描的集合、两个内核的交接段、
-/// 交接段之后走读内核自己用的两张表）。
-pub const SCRATCH_WORDS: usize = 255_504;
+/// 交接段之后走读内核自己用的表：两张，加 I-3.5 的出现规则读的四张）。
+pub const SCRATCH_WORDS: usize = 290_064;
 /// 内核的工作组大小（三个内核都是 `@workgroup_size(64)`）：派 n 个状态要 ⌈n ÷ 64⌉ 个工作组。
 pub const WORKGROUP_SIZE: usize = 64;
 /// 内核的四个绑定（`crash_judge_common.wgsl` 开头）：派活参数、打包的表、结论、草稿区。
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 64622ad..8d8f189 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -4,7 +4,8 @@
 //! 崩溃状态，GPU 判出的红位（两遍 oracle 各七类、记录核对器两条）要与 CPU `judge_crash_image` 的 `red_items` 逐位相同，池级 checker
 //! 每一条不变量的违例位（`IMPLEMENTED_INVARIANTS` 全部，不屏蔽）要与 CPU 池级 checker 逐位相同，一个状态都不许判不了。
 //! 改坏那条流上 CPU 有红，GPU 要红在同样的状态、同样的项上——这是 GPU 判器分得出差别的自证；里程碑三二·1 那一批的每一条不变量
-//! 另有一条只改坏它管的那个字段的流（`the_gpu_judge_agrees_with_the_cpu_when_…` 那一组）。
+//! 另有一条只改坏它管的那个字段的流（`the_gpu_judge_agrees_with_the_cpu_when_…` 那一组）；I-3.5 的出现规则红与成立各一条，
+//! 恢复施加一条记录取新根段的那几段、施加出来的那一版从记录走实例表与分配记录树、I-8.10 后三段，各有一条同样形态的流。
 //! 要本机一张空闲显存够的 NVIDIA 独显：本机一张 NVIDIA 独显适配器都没有时用卡的用例打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、
 //! 返回成功；有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）。要 `verdict-store,gpu` 特性；
 //! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（默认 8）、显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（没设不定额度）、
@@ -1504,6 +1505,145 @@ fn record_rollback_with_a_journal_record_txg_going_back(tag: &str) -> RecordedFl
     flow
 }
 
+// ───────────── 第 9 批：I-3.5 的出现规则、恢复施加一条记录取新根段的七段、施加出来的那一版从记录走实例表与分配记录树、I-8.10 后三段 ─────────────
+
+/// journal 记录新根段里实例表指针的段内偏移（D23（journal 的角色与格式） 已定项 15：树表指针 86 + 映射根指针 86 + 水位 8 + F 8）。
+const NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER: usize = 188;
+/// 新根段里分配记录树根指针的段内偏移（实例表指针之后 86 字节）。
+const NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER: usize = 274;
+/// 新根段末尾卸载记号 4 字节的段内偏移（只有位 0，与根记录 flags 同一个编码）。
+const NEW_ROOT_SEGMENT_UNMOUNT_MARKER: usize = 360;
+/// 码 2 / 码 3 指针里两条位置条目各自的整单元校验和的偏移（位置条目 50 与 64 起，盘 4 + 槽 6 之后）。
+const POINTER_LOCATION_CHECKSUM_OFFSETS: [usize; 2] = [60, 74];
+/// 指针头里 extent 偏移 2 字节的偏移（MAC / nonce 28、算法类型 1、压缩码 1、压缩长度 2 之后）。
+const POINTER_EXTENT_OFFSET_OFFSET: usize = 32;
+/// 码 2 / 码 3 指针宽。
+const NODE_POINTER_BYTES: usize = 86;
+/// 数据单元头里诞生代号的偏移（码 1 写序五元组：树 43、inode 51、对象出生代 59、偏移 67、诞生代号 75）。
+const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;
+
+/// I-3.5（引用区间的精确性）的出现规则：新池新建文件那条流上，发布那一版（txg 3）新写的数据单元头里的诞生代号 3 → 2、重封到根——
+/// 发布那一版引用、暖机最后一版（txg 2）不引用的这个单元诞生代号不大于 2，它在 extent 树里，而 extent 树的根指针生在 txg 3、不是整棵继承下来的。
+/// 指针、映射 key 与分配代还记 3，I-1.2、I-1.11、I-3.10 跟着红（与 `checker_known_bad_images.rs` 里 I-3.5 那一格同一个改法）。
+fn record_new_pool_with_a_data_unit_born_no_later_than_the_previous_version(
+    tag: &str,
+) -> RecordedFlow {
+    let data_unit = |bytes: &[u8]| unit_class(bytes) == 1;
+    record_new_pool_with_a_resealed_field(
+        tag,
+        "new-pool-data-unit-birth-txg-three-to-two",
+        &data_unit,
+        false,
+        &|bytes| {
+            let birth_txg = u64::from_le_bytes(
+                bytes[DATA_UNIT_BIRTH_TXG_OFFSET..DATA_UNIT_BIRTH_TXG_OFFSET + 8]
+                    .try_into()
+                    .expect("8"),
+            );
+            assert_eq!(birth_txg, 3, "新建的文件那个数据单元生在发布那一版 txg 3");
+            bytes[DATA_UNIT_BIRTH_TXG_OFFSET..DATA_UNIT_BIRTH_TXG_OFFSET + 8]
+                .copy_from_slice(&2u64.to_le_bytes());
+        },
+    )
+}
+
+/// 发布那几条 journal 记录新根段里的卸载记号写成 0b10（位 0 之外有位为 1）、重封：core 的读者当记录损坏（重放不施加它），
+/// CPU 池级 checker 照同一口径不收它（RR1）。
+fn record_new_pool_with_a_last_record_whose_unmount_marker_sets_another_bit(
+    tag: &str,
+) -> RecordedFlow {
+    let mut flow =
+        record_new_pool_file_creation(tag, "new-pool-last-record-unmount-marker-bit-one");
+    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
+        let marker = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_UNMOUNT_MARKER;
+        assert_eq!(
+            bytes[marker..marker + 4],
+            [0u8; 4],
+            "建文件那次发布不是卸载那一串，卸载记号是 0"
+        );
+        bytes[marker..marker + 4].copy_from_slice(&0b10u32.to_le_bytes());
+    });
+    flow
+}
+
+/// 发布那几条 journal 记录新根段里的卸载记号写成 1（只有位 0）、重封：记录照收、照施加，新根段与同代的根（flags 0）不逐字节相同，
+/// CPU 池级 checker 红 I-8.10（第七段，卸载记号）。
+fn record_new_pool_with_a_last_record_carrying_the_unmount_marker(tag: &str) -> RecordedFlow {
+    let mut flow = record_new_pool_file_creation(tag, "new-pool-last-record-unmount-marker-set");
+    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
+        let marker = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_UNMOUNT_MARKER;
+        bytes[marker..marker + 4].copy_from_slice(&1u32.to_le_bytes());
+    });
+    flow
+}
+
+/// 发布那几条 journal 记录新根段里实例表指针的两条位置条目校验和各取反、重封：记录照收、照施加，施加出来那一版的实例表指针
+/// 与所选根（暖机最后那条根）的不同、两份都读不对。core 的恢复从记录取实例表（已定项 15）走到文件之前就读不出；
+/// CPU 池级 checker 走施加出来的那一版时实例表读不出（I-4.1），根也落了的状态上新根段与同代的根不逐字节相同（I-8.10 第五段）。
+fn record_new_pool_with_a_last_record_whose_instance_table_pointer_reads_wrong(
+    tag: &str,
+) -> RecordedFlow {
+    let mut flow =
+        record_new_pool_file_creation(tag, "new-pool-last-record-instance-table-checksums-flipped");
+    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
+        let pointer = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER;
+        for location_checksum in POINTER_LOCATION_CHECKSUM_OFFSETS {
+            for byte in &mut bytes[pointer + location_checksum..pointer + location_checksum + 4] {
+                *byte ^= 0xFF;
+            }
+        }
+    });
+    flow
+}
+
+/// 发布那几条 journal 记录新根段里实例表指针头的 extent 偏移写成 1、重封：core 的读者当记录损坏（指针头不是第一版的取值，
+/// `NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from`），重放不施加它；CPU 池级 checker 不判记录里的指针头、
+/// 照收，根也落了的状态上红 I-8.10（第五段）。
+fn record_new_pool_with_a_last_record_whose_instance_table_pointer_carries_an_extent_offset(
+    tag: &str,
+) -> RecordedFlow {
+    let mut flow =
+        record_new_pool_file_creation(tag, "new-pool-last-record-instance-table-extent-offset-one");
+    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
+        let extent_offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET
+            + NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER
+            + POINTER_EXTENT_OFFSET_OFFSET;
+        bytes[extent_offset..extent_offset + 2].copy_from_slice(&1u16.to_le_bytes());
+    });
+    flow
+}
+
+/// 发布那几条 journal 记录新根段里分配记录树根指针（建文件那一版本来全零）换成外来的字节：同一条记录的实例表指针、
+/// 两条位置条目校验和各取反，重封。施加出来那一版的分配记录树根与它施加在其上的那条根（全零）不同：CPU 池级 checker 走那一版时
+/// 从记录取它、两份都读不对（I-4.1），根也落了的状态上新根段与同代的根不逐字节相同（I-8.10 第六段）。
+fn record_new_pool_with_a_last_record_whose_allocation_record_tree_root_is_foreign(
+    tag: &str,
+) -> RecordedFlow {
+    let mut flow =
+        record_new_pool_file_creation(tag, "new-pool-last-record-allocation-root-foreign-bytes");
+    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
+        let instance_table =
+            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER;
+        let allocation_root =
+            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER;
+        assert_eq!(
+            bytes[allocation_root..allocation_root + NODE_POINTER_BYTES],
+            [0u8; NODE_POINTER_BYTES],
+            "带文件的一版分配记录树根恒全零"
+        );
+        let foreign = bytes[instance_table..instance_table + NODE_POINTER_BYTES].to_vec();
+        bytes[allocation_root..allocation_root + NODE_POINTER_BYTES].copy_from_slice(&foreign);
+        for location_checksum in POINTER_LOCATION_CHECKSUM_OFFSETS {
+            for byte in &mut bytes
+                [allocation_root + location_checksum..allocation_root + location_checksum + 4]
+            {
+                *byte ^= 0xFF;
+            }
+        }
+    });
+    flow
+}
+
 /// GPU 判器与 CPU 判器在这条流的每个状态上红位与违例位逐位相同、落到同一条根，CPU 池级 checker 在这条流上红 `invariant`。
 /// 本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
 fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
@@ -1579,6 +1719,21 @@ fn expected_pool_checker_bits(judgement: &Layer0StateJudgement) -> u128 {
     bits
 }
 
+/// CPU 池级 checker 在这个状态上判过而成立的不变量（不是报不适用）：一位一条，与违例位同一个下标。
+fn expected_pool_checker_held_bits(judgement: &Layer0StateJudgement) -> u128 {
+    let mut bits = 0u128;
+    for (invariant, verdict) in &judgement.pool_checker {
+        if let InvariantVerdict::Holds = verdict {
+            let index = IMPLEMENTED_INVARIANTS
+                .iter()
+                .position(|candidate| candidate == invariant)
+                .expect("池级 checker 报的不变量在清单里");
+            bits |= 1u128 << index;
+        }
+    }
+    bits
+}
+
 fn invariant_names(bits: u128) -> Vec<&'static str> {
     IMPLEMENTED_INVARIANTS
         .iter()
@@ -1596,6 +1751,8 @@ struct Comparison {
     gpu_checker_red: u64,
     /// CPU 池级 checker 在这条流的哪几条不变量上红过。
     cpu_checker_red_invariants: std::collections::BTreeSet<&'static str>,
+    /// CPU 池级 checker 在这条流的哪几条不变量上判过而成立过（至少一个状态上不是报不适用）。
+    cpu_checker_held_invariants: std::collections::BTreeSet<&'static str>,
     /// 看 journal 那一遍落到的根或两遍的结局类别与 CPU 不同的状态数（算进不一致）。
     recovery_mismatches: u64,
     mismatches: Vec<String>,
@@ -1647,7 +1804,7 @@ fn compare_on_every_state(card: &GpuCard, flow: &RecordedFlow, limit: usize) ->
     let slice = state_count.div_ceil(threads * 8).max(1);
     let next_slice = std::sync::atomic::AtomicU64::new(0);
     let slice_count = state_count.div_ceil(slice);
-    type JudgedOnCpu = (u32, u128, String, String, [u32; 4]);
+    type JudgedOnCpu = (u32, u128, String, String, [u32; 4], u128);
     let mut slices: Vec<(u64, Vec<JudgedOnCpu>)> = std::thread::scope(|scope| {
         let handles: Vec<_> = (0..threads.min(slice_count))
             .map(|_| {
@@ -1689,6 +1846,7 @@ fn compare_on_every_state(card: &GpuCard, flow: &RecordedFlow, limit: usize) ->
                                         judgement.ignored.outcome
                                     ),
                                     recovery_words_of(judgement),
+                                    expected_pool_checker_held_bits(judgement),
                                 ));
                             },
                         );
@@ -1715,12 +1873,18 @@ fn compare_on_every_state(card: &GpuCard, flow: &RecordedFlow, limit: usize) ->
         cpu_checker_red: 0,
         gpu_checker_red: 0,
         cpu_checker_red_invariants: std::collections::BTreeSet::new(),
+        cpu_checker_held_invariants: std::collections::BTreeSet::new(),
         recovery_mismatches: 0,
         mismatches: Vec::new(),
     };
-    for (ordinal, (cpu_bits, cpu_checker_bits, cpu_items, cpu_report, cpu_recovery)) in
-        cpu.iter().enumerate()
+    for (
+        ordinal,
+        (cpu_bits, cpu_checker_bits, cpu_items, cpu_report, cpu_recovery, cpu_checker_held_bits),
+    ) in cpu.iter().enumerate()
     {
+        comparison
+            .cpu_checker_held_invariants
+            .extend(invariant_names(*cpu_checker_held_bits));
         let gpu_words: [u32; VERDICT_WORDS] = gpu[ordinal];
         let gpu_bits = gpu_words[0] & !(1 << verdict_bits::UNDECIDABLE);
         let undecidable = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
@@ -2080,7 +2244,7 @@ fn verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several(
 /// 在扫描内核读回违例位之后种第 128 位（下标 127）：
 /// 两份种过的内核判出的每个状态，违例位等于没种的那一份判出的再并上种的位（第 65 位每个状态都有，第 128 位只在扫描内核判完的状态上有），
 /// 其余的字逐字相同。违例位的高 64 位在走读内核写出、扫描内核读回又写出、Rust 读出这三处任一处被截掉，这条红。
-/// 池级 checker 今天 68 条（下标 0..=67；内核判下标 65 之外的每一条，下标 65 的 I-3.5 内核还没判，`crash_judge_gpu` 的「内核还没判」那张表），种的是第 69 位（下标 68）。
+/// 池级 checker 今天 68 条（下标 0..=67，内核每一条都判），种的是第 69 位（下标 68）。
 #[test]
 fn a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels() {
     const FIRST_BIT_OF_THE_HIGH_HALF: usize = 64;
@@ -2804,6 +2968,139 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_root_slots_are_not_aligned_to_the_
     );
 }
 
+/// 列出这条流上不一致的状态（前几个），没有就是空串。
+fn mismatch_listing(comparison: &Comparison) -> String {
+    comparison
+        .mismatches
+        .iter()
+        .filter(|line| !line.is_empty())
+        .cloned()
+        .collect::<Vec<_>>()
+        .join("\n")
+}
+
+/// I-3.5：一版引用、前一版不引用的单元诞生代号不大于前一版的 txg、也不在整棵继承下来的用户可见树里——CPU 红 I-3.5，GPU 在同样的状态上红。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_version_references_a_unit_born_no_later_than_the_previous_version(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_version_references_a_unit_born_no_later_than_the_previous_version",
+        &record_new_pool_with_a_data_unit_born_no_later_than_the_previous_version(
+            "crash-judge-gpu-i35",
+        ),
+        "I-3.5",
+    );
+}
+
+/// I-3.5 判成立的那一支：固定脚本到回退 D 的流上，回退那次发布引用回退目标那一版（txg 3）的单元，诞生代号都不大于前一版的 txg，
+/// 它们落在根指针生在 txg 3 的 extent / inode 树里、整棵继承下来（(b)）。CPU 在至少一个状态上判过 I-3.5 而成立、一个状态都不红它，
+/// GPU 每个状态逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_rollback_publish_inheriting_its_user_visible_trees_whole_keeps_the_appearance_rule(
+) {
+    let flow = record_fixed_script_through_rollback("crash-judge-gpu-i35-holds");
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_rollback_publish_inheriting_its_user_visible_trees_whole_keeps_the_appearance_rule",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        comparison.cpu_checker_held_invariants.contains("I-3.5")
+            && !comparison.cpu_checker_red_invariants.contains("I-3.5"),
+        "流 {} 上 CPU 池级 checker 判过 I-3.5 而成立、一个状态都不红它（成立过的：{:?}；红过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_held_invariants,
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
+/// 恢复内核与 core 同一口径读记录（D23（journal 的角色与格式） 已定项 15，`root_record.rs` 的 `of_unmount_marker_field_of_a_new_root_segment`）：
+/// 新根段卸载记号位 0 之外有位为 1 的记录当损坏、不施加。发布那条记录落了而根没落的状态上 CPU 恢复停在暖机那条根，
+/// GPU 收了这条记录就落到发布那一版、两遍恢复的结论对不上；池级 checker 那一边 GPU 也照 RR1 不收它。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_unmount_marker_sets_another_bit_is_not_applied(
+) {
+    let flow = record_new_pool_with_a_last_record_whose_unmount_marker_sets_another_bit(
+        "crash-judge-gpu-rr1",
+    );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_unmount_marker_sets_another_bit_is_not_applied",
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
+}
+
+/// 恢复施加一条记录取它新根段里的实例表指针（已定项 15，`recovery.rs` 的 `replay_journal`），不照抄所选根的：记录里的实例表指针读不对时
+/// CPU 恢复走到文件之前就读不出，GPU 照抄所选根就读得出；池级 checker 走施加出来那一版也从记录取（I-4.1），根落了的状态上红 I-8.10 第五段。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_applied_record_names_an_instance_table_other_than_the_chosen_root(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_applied_record_names_an_instance_table_other_than_the_chosen_root",
+        &record_new_pool_with_a_last_record_whose_instance_table_pointer_reads_wrong(
+            "crash-judge-gpu-applied-instance-table",
+        ),
+        "I-8.10",
+    );
+}
+
+/// 恢复内核判记录新根段四条指针的头与 core 同一口径：实例表指针头的 extent 偏移非 0 的记录当损坏、不施加；
+/// CPU 池级 checker 不判记录里的指针头、照收，根落了的状态上红 I-8.10 第五段。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_instance_table_pointer_carries_an_extent_offset_is_not_applied(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_instance_table_pointer_carries_an_extent_offset_is_not_applied",
+        &record_new_pool_with_a_last_record_whose_instance_table_pointer_carries_an_extent_offset(
+            "crash-judge-gpu-record-pointer-head",
+        ),
+        "I-8.10",
+    );
+}
+
+/// 走读内核走由记录施加出来的那一版时，分配记录树根从记录新根段取（walk.rs 的 `walk_version_applied_only_by_records`），
+/// 不取它施加在其上的那条根的：记录里的分配记录树根是外来的、读不对时 CPU 判 I-4.1 红，GPU 取那条根（全零、不走）就不红。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_applied_version_names_an_allocation_record_tree_root_other_than_the_root_it_is_applied_on(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_applied_version_names_an_allocation_record_tree_root_other_than_the_root_it_is_applied_on",
+        &record_new_pool_with_a_last_record_whose_allocation_record_tree_root_is_foreign(
+            "crash-judge-gpu-applied-allocation-root",
+        ),
+        "I-4.1",
+    );
+}
+
+/// I-8.10 第七段：发布那条记录的卸载记号与同代的根不同（记录 1、根 0）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_last_record_carries_another_unmount_marker_than_its_root(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_last_record_carries_another_unmount_marker_than_its_root",
+        &record_new_pool_with_a_last_record_carrying_the_unmount_marker(
+            "crash-judge-gpu-i810-unmount",
+        ),
+        "I-8.10",
+    );
+}
+
 /// 两个池级 checker 内核把违例位的每个字都写出来：结论缓冲派活前整个填全 1，没被写的字读出来是全 1。
 /// 新池新建文件那条流的每个状态（不种假位）上，违例位在池级 checker 条数之外的那些位全是 0、判得了——高 64 位那两个字只要有一个没写，这条红。
 #[test]
````

## batch9-core2/patch

实现员报告：`/tmp/claude-1000/m3-r1/batch9-core2/patch/report.md`（sha256 0beddd74369126ca…），补丁 sha256 d14762d1ad24c143…

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index bf66bc6..affcf4a 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -39,8 +39,11 @@ use singlefs_core::address::{
 };
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::instance_table_sweep::{
+    InstanceTableSweepAtTheEndOfTheMount, InstanceTableSweepOutcome,
+};
 use singlefs_core::make_filesystem::MakeFilesystemParameters;
-use singlefs_core::mount::{mount_writable_with_space_admission, MountError};
+use singlefs_core::mount::{mount_writable_with_space_admission, MountError, MountOutput};
 use singlefs_core::recovery::{
     choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryOutcome,
     RecoveryReport,
@@ -287,8 +290,8 @@ impl CrashPoint {
     }
 }
 
-/// 可写挂载的四段（D23（journal 的角色与格式） 已定项 16 取号、D18（块里携带什么信息） 已定项 11 写行、D16（发布语义） 已定项 8 暖机、
-/// D8（核心索引结构） 已定项 16 第 6 条挂载续做）：挂载那一段录制流里的一次写落在哪一段。
+/// 可写挂载的六段（D23（journal 的角色与格式） 已定项 16 取号、D18（块里携带什么信息） 已定项 11 写行、D16（发布语义） 已定项 8 暖机、
+/// D8（核心索引结构） 已定项 16 第 6 条挂载续做、D18 已定项 11「回收」一条的清扫抹头与清扫发布）：挂载那一段录制流里的一次写落在哪一段。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub enum WritableMountPhase {
     /// 取号：写行那次发布的第一个单元写或 journal 记录写之前的那几次系统配置槽写。
@@ -300,14 +303,21 @@ pub enum WritableMountPhase {
     WarmUp,
     /// 挂载续做：暖机与推的抬 F 之后、交回之前把所选那一版的 `deleted_inodes` 走空的那几步删除发布（所选那一版有删除中的文件时才有）。
     Continuation,
+    /// 实例表清扫抹头：续做之后、清扫发布之前，FUA 写零那几个未发布单元的头、每块盘一道屏障（`singlefs_core::instance_table_sweep`；
+    /// 挂载末尾清扫了才有）。
+    ErasingHeadsOfTheInstanceTableSweep,
+    /// 实例表清扫发布：删行的那一次专用发布（单元、记录、根槽、轮换）。
+    InstanceTableSweepPublish,
 }
 
 impl WritableMountPhase {
-    pub const ALL: [WritableMountPhase; 4] = [
+    pub const ALL: [WritableMountPhase; 6] = [
         WritableMountPhase::Acquisition,
         WritableMountPhase::RowPublish,
         WritableMountPhase::WarmUp,
         WritableMountPhase::Continuation,
+        WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
+        WritableMountPhase::InstanceTableSweepPublish,
     ];
 
     #[must_use]
@@ -317,6 +327,10 @@ impl WritableMountPhase {
             WritableMountPhase::RowPublish => "row_publish",
             WritableMountPhase::WarmUp => "warm_up",
             WritableMountPhase::Continuation => "continuation",
+            WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep => {
+                "erasing_heads_of_the_instance_table_sweep"
+            }
+            WritableMountPhase::InstanceTableSweepPublish => "instance_table_sweep_publish",
         }
     }
 }
@@ -1376,6 +1390,8 @@ pub fn inject_crashes_into_history(
                     .as_ref(),
                 publishes_before_the_deletion_queue_continuation: mount_run
                     .publishes_before_the_deletion_queue_continuation,
+                publishes_before_the_instance_table_sweep: mount_run
+                    .publishes_before_the_instance_table_sweep,
                 parameters: &parameters,
                 space_admission: execution.space_admission,
                 reference_filesystem_at_the_first_crash: &reference_filesystem_judgement,
@@ -1678,6 +1694,9 @@ pub struct WritableMountOnTheCrashImage {
     /// 挂载交回 `MountError::Publish` 或 `MountError::FloorRaiseFailedAfterTheMountsPublishes` 时按错里交回的已落盘发布写账划
     /// （[`publishes_before_the_continuation_of_a_mount_that_failed`]；划不出续做、别的错都是 None：写行之后都算暖机）。
     pub publishes_before_the_deletion_queue_continuation: Option<usize>,
+    /// 挂载末尾清扫写了东西（抹了头或发了清扫发布）时，清扫之前这次挂载发了几次（写行、暖机、推的抬 F、续做每一步与续做里推的抬 F）：
+    /// 挂载那一段录制流里第几个根槽写之后起是清扫抹头；没清扫、或清扫一个字节都没写、或挂载被拒时 None。
+    pub publishes_before_the_instance_table_sweep: Option<usize>,
     /// 挂载推的抬 F 从挂载那一段写表（`mount_operations` 按同一份几何切出来的写表）的哪个写起（二次崩溃上 F 判不判按它分；
     /// 挂载准入推的与续做里推的都认，界按它们在写表里的位置取，取不出的是 [`FloorRaisesInTheMountWrites::NotLocatedInTheMountWrites`]）；挂载被拒时 None。
     pub floor_raises_in_the_mount_writes: Option<FloorRaisesInTheMountWrites>,
@@ -1743,6 +1762,9 @@ pub fn mount_and_publish_once_on_the_crash_image(
         ),
         Err(error) => publishes_before_the_continuation_of_a_mount_that_failed(error),
     };
+    let publishes_before_the_instance_table_sweep = mounted.as_ref().ok().and_then(|mounted| {
+        publishes_before_the_instance_table_sweep_of_the_mount_output(&mounted.output)
+    });
     let user_visible_tree_root_pointers_of_the_continuation =
         mounted.as_ref().ok().map(|mounted| {
             UserVisibleTreeRootPointersOfTheContinuation::of_the_mount_output(&mounted.output)
@@ -1829,11 +1851,47 @@ pub fn mount_and_publish_once_on_the_crash_image(
         pool_after,
         floor_raised_by_the_mount,
         publishes_before_the_deletion_queue_continuation,
+        publishes_before_the_instance_table_sweep,
         floor_raises_in_the_mount_writes,
         user_visible_tree_root_pointers_of_the_continuation,
     }
 }
 
+/// 挂载末尾清扫写了东西时，清扫之前这次挂载发了几次（[`WritableMountOnTheCrashImage::publishes_before_the_instance_table_sweep`]）。
+#[must_use]
+pub fn publishes_before_the_instance_table_sweep_of_the_mount_output(
+    output: &MountOutput,
+) -> Option<usize> {
+    let sweep_wrote_something = match &output.instance_table_sweep {
+        InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold { .. }
+        | InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted
+        | InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::NoRowDeleted(_)) => {
+            false
+        }
+        InstanceTableSweepAtTheEndOfTheMount::Swept(
+            InstanceTableSweepOutcome::RowsDeleted { .. }
+            | InstanceTableSweepOutcome::SweepPublishFailed { .. },
+        ) => true,
+        InstanceTableSweepAtTheEndOfTheMount::Swept(
+            InstanceTableSweepOutcome::SweepPublishRefusedForSpace { heads_erased, .. }
+            | InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing {
+                heads_erased, ..
+            },
+        ) => !heads_erased.is_empty(),
+    };
+    sweep_wrote_something.then(|| {
+        1 + output.warm_up_publishes.len()
+            + output
+                .space_admission
+                .floor_raises()
+                .iter()
+                .chain(&output.floor_raises_pushed_by_the_continuation)
+                .map(|raised| raised.publishes.len())
+                .sum::<usize>()
+            + output.deletion_queue_continuation.len()
+    })
+}
+
 /// 一次发布写没写只有续做的删除步才写的那几种结构：文件的 extent 树、inode 树与 `deleted_inodes` 树（数据单元也算，挂载里没有别的发布写它）。
 /// 写行、暖机与抬 F 的空发布只重写分配记录树、记账树、中央映射树、树表、实例表与固定结构（2026-10-03 在副本上量的：崩在意图之后重挂，
 /// 写行与两次暖机写的是这几种，续做那一步写了 extent 树与 inode 树根）。
@@ -2469,22 +2527,32 @@ fn mount_transparency_violations_of_a_second_crash(
 /// 挂载那一段录制流里每一次写落在挂载的哪一段：先是取号的系统配置槽写，第一个别的写起是写行那次发布；
 /// 一次发布的根槽写与紧跟着的系统配置槽写（轮换、抬 F 那一串先写的新 F）之后的第一个别的写起是下一次发布：
 /// 写行之后、第 `publishes_before_the_deletion_queue_continuation` 个根槽写之前的那几次是暖机（连同挂载推的抬 F），
-/// 从它起是续做（没崩过的那一次挂载被拒、交不出这个数时 None：写行之后都算暖机）。
+/// 从它起是续做（没崩过的那一次挂载被拒、交不出这个数时 None：写行之后都算暖机）；第 `publishes_before_the_instance_table_sweep`
+/// 个根槽写之后起是清扫抹头（FUA 写零头所在的物理块、每块盘一道屏障），抹头之后第一个不带 FUA 的单元写或 journal 记录写起是清扫发布
+/// （没清扫时 None）。
 fn writable_mount_phase_of_each_write(
     writes: &[RetainedWrite],
     publishes_before_the_deletion_queue_continuation: Option<usize>,
+    publishes_before_the_instance_table_sweep: Option<usize>,
 ) -> Vec<WritableMountPhase> {
-    let phase_of_the_publish_after = |root_records_written: usize| match (
-        root_records_written,
-        publishes_before_the_deletion_queue_continuation,
-    ) {
-        (0, _) => WritableMountPhase::RowPublish,
-        (publishes_landed, Some(publishes_before_the_continuation))
-            if publishes_landed >= publishes_before_the_continuation =>
-        {
-            WritableMountPhase::Continuation
+    let phase_of_the_publish_after = |root_records_written: usize| {
+        if publishes_before_the_instance_table_sweep.is_some_and(|publishes_before_the_sweep| {
+            root_records_written > 0 && root_records_written >= publishes_before_the_sweep
+        }) {
+            return WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep;
+        }
+        match (
+            root_records_written,
+            publishes_before_the_deletion_queue_continuation,
+        ) {
+            (0, _) => WritableMountPhase::RowPublish,
+            (publishes_landed, Some(publishes_before_the_continuation))
+                if publishes_landed >= publishes_before_the_continuation =>
+            {
+                WritableMountPhase::Continuation
+            }
+            (_, Some(_) | None) => WritableMountPhase::WarmUp,
         }
-        (_, Some(_) | None) => WritableMountPhase::WarmUp,
     };
     let mut phase = WritableMountPhase::Acquisition;
     let mut root_records_written = 0;
@@ -2499,14 +2567,34 @@ fn writable_mount_phase_of_each_write(
                 (
                     WritableMountPhase::RowPublish
                     | WritableMountPhase::WarmUp
-                    | WritableMountPhase::Continuation,
+                    | WritableMountPhase::Continuation
+                    | WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep
+                    | WritableMountPhase::InstanceTableSweepPublish,
                     StepKind::SystemConfigurationSlot,
                 ) => phase,
+                (
+                    WritableMountPhase::InstanceTableSweepPublish,
+                    StepKind::UnitWrite
+                    | StepKind::JournalRecord
+                    | StepKind::RootRecordFua
+                    | StepKind::ZeroFill
+                    | StepKind::Barrier,
+                )
+                | (
+                    WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
+                    StepKind::JournalRecord,
+                ) => WritableMountPhase::InstanceTableSweepPublish,
+                (WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep, StepKind::UnitWrite)
+                    if !write.is_force_unit_access =>
+                {
+                    WritableMountPhase::InstanceTableSweepPublish
+                }
                 (
                     WritableMountPhase::Acquisition
                     | WritableMountPhase::RowPublish
                     | WritableMountPhase::WarmUp
-                    | WritableMountPhase::Continuation,
+                    | WritableMountPhase::Continuation
+                    | WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
                     StepKind::UnitWrite
                     | StepKind::JournalRecord
                     | StepKind::RootRecordFua
@@ -2522,6 +2610,26 @@ fn writable_mount_phase_of_each_write(
         .collect()
 }
 
+/// 清扫两段（抹头、清扫发布）在挂载那一段写表里各是哪几个写（给 checker 档清扫的用例划段用；划法同 [`writable_mount_phase_of_each_write`]）。
+#[must_use]
+pub fn writes_of_the_instance_table_sweep_phases(
+    writes: &[RetainedWrite],
+    publishes_before_the_instance_table_sweep: usize,
+) -> BTreeMap<WritableMountPhase, Vec<usize>> {
+    let mut by_phase: BTreeMap<WritableMountPhase, Vec<usize>> = BTreeMap::new();
+    for (write_index, phase) in writable_mount_phase_of_each_write(
+        writes,
+        None,
+        Some(publishes_before_the_instance_table_sweep),
+    )
+    .into_iter()
+    .enumerate()
+    {
+        by_phase.entry(phase).or_default().push(write_index);
+    }
+    by_phase
+}
+
 /// 挂载途中的一次二次崩溃：挂载那一段的第几段、段内哪几个写持久了、崩在挂载的哪一段。
 #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
 struct SecondCrashPoint {
@@ -2596,6 +2704,8 @@ struct SecondCrashInputs<'history> {
         Result<ContentReadByRecovery, RemountIdempotenceNotJudged>,
     /// 没崩过的那一次挂载在续做之前发了几次（[`WritableMountOnTheCrashImage::publishes_before_the_deletion_queue_continuation`]）。
     publishes_before_the_deletion_queue_continuation: Option<usize>,
+    /// 没崩过的那一次挂载在清扫之前发了几次（[`WritableMountOnTheCrashImage::publishes_before_the_instance_table_sweep`]）。
+    publishes_before_the_instance_table_sweep: Option<usize>,
     /// 第一次崩溃镜像上的只读恢复与挂载透明要比的字段（R1）。
     fields_read_after_the_first_crash: &'history RecoveryFieldsRead,
     /// 没崩过的那一次挂载推的抬 F 从挂载那一段写表的哪个写起（没推是 [`FloorRaisesInTheMountWrites::NotPushed`]）；挂载被拒时 None。
@@ -2634,6 +2744,7 @@ fn judge_second_crashes_inside_the_writable_mount(
         newest_persisted_before_the_mount,
         content_after_the_uninterrupted_mount,
         publishes_before_the_deletion_queue_continuation,
+        publishes_before_the_instance_table_sweep,
         fields_read_after_the_first_crash,
         floor_raises_in_the_uninterrupted_mount_writes,
         continuation_of_the_uninterrupted_mount,
@@ -2646,6 +2757,7 @@ fn judge_second_crashes_inside_the_writable_mount(
     let phase_of_each_write = writable_mount_phase_of_each_write(
         mount_writes,
         publishes_before_the_deletion_queue_continuation,
+        publishes_before_the_instance_table_sweep,
     );
     let second_crash_points =
         draw_second_crash_points(seed, crash_point, mount_segments, &phase_of_each_write);
@@ -3968,7 +4080,7 @@ mod tests {
             StepKind::SystemConfigurationSlot,
         ];
         let writes: Vec<RetainedWrite> = kinds.iter().map(|kind| write_of(*kind)).collect();
-        let phases = writable_mount_phase_of_each_write(&writes, Some(2));
+        let phases = writable_mount_phase_of_each_write(&writes, Some(2), None);
         assert_eq!(
             phases,
             [
@@ -4011,6 +4123,98 @@ mod tests {
         );
     }
 
+    /// 实例表清扫两段（D18（块里携带什么信息） 已定项 11「回收」一条）：挂载那一段的写里，第「清扫之前发了几次」个根槽写之后起是清扫抹头
+    /// （FUA 写零的单元头、每块盘一道屏障），之后第一个不带 FUA 的单元写起是清扫发布（单元、屏障、记录、根槽、轮换），两段摆二次崩溃时各摆到一个。
+    /// 造的写表：取号；写行那次（单元、记录、根槽、轮换）；一次暖机（单元、根槽）；抹两个头、两道屏障；清扫发布——清扫之前发了 2 次。
+    /// 判别力：`WritableMountPhase::ALL` 里没有这两段时一个二次崩溃都摆不到；不按 FUA 分开时清扫发布的写归抹头。
+    #[test]
+    fn the_writes_of_the_instance_table_sweep_split_into_erasing_heads_and_the_sweep_publish_and_each_gets_a_second_crash_point(
+    ) {
+        let write_of = |kind: StepKind, is_force_unit_access: bool| RetainedWrite {
+            device: DeviceIdentity(0),
+            kind,
+            is_force_unit_access,
+            offset: DeviceOffsetInBytes(0),
+            contents: singlefs_harness::memory_pool::WrittenContents::Bytes(vec![0; 512]),
+        };
+        let writes = vec![
+            write_of(StepKind::SystemConfigurationSlot, false),
+            write_of(StepKind::UnitWrite, false),
+            write_of(StepKind::JournalRecord, false),
+            write_of(StepKind::RootRecordFua, true),
+            write_of(StepKind::SystemConfigurationSlot, false),
+            write_of(StepKind::UnitWrite, false),
+            write_of(StepKind::RootRecordFua, true),
+            write_of(StepKind::UnitWrite, true),
+            write_of(StepKind::UnitWrite, true),
+            write_of(StepKind::Barrier, false),
+            write_of(StepKind::Barrier, false),
+            write_of(StepKind::UnitWrite, false),
+            write_of(StepKind::Barrier, false),
+            write_of(StepKind::JournalRecord, false),
+            write_of(StepKind::RootRecordFua, true),
+            write_of(StepKind::SystemConfigurationSlot, false),
+        ];
+        let phases = writable_mount_phase_of_each_write(&writes, None, Some(2));
+        assert_eq!(
+            phases[7..],
+            [
+                WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
+                WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
+                WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
+                WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
+                WritableMountPhase::InstanceTableSweepPublish,
+                WritableMountPhase::InstanceTableSweepPublish,
+                WritableMountPhase::InstanceTableSweepPublish,
+                WritableMountPhase::InstanceTableSweepPublish,
+                WritableMountPhase::InstanceTableSweepPublish,
+            ],
+            "抹两个头与两道屏障归抹头，清扫发布从它第一个不带 FUA 的单元写起"
+        );
+        assert_eq!(
+            phases[..7],
+            [
+                WritableMountPhase::Acquisition,
+                WritableMountPhase::RowPublish,
+                WritableMountPhase::RowPublish,
+                WritableMountPhase::RowPublish,
+                WritableMountPhase::RowPublish,
+                WritableMountPhase::WarmUp,
+                WritableMountPhase::WarmUp,
+            ],
+            "清扫之前那几段照旧"
+        );
+        let crash_point = CrashPoint {
+            segment_index: 0,
+            persisted_within_the_segment: vec![false],
+            step: StepPosition::StartingPoint,
+            operation_kind: None,
+        };
+        let one_write_per_segment: Vec<Vec<usize>> = (0..writes.len())
+            .map(|write_index| vec![write_index])
+            .collect();
+        let second_crash_points = draw_second_crash_points(
+            HistorySeed(7),
+            &crash_point,
+            &one_write_per_segment,
+            &phases,
+        );
+        for phase in [
+            WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
+            WritableMountPhase::InstanceTableSweepPublish,
+        ] {
+            assert_eq!(
+                second_crash_points
+                    .iter()
+                    .filter(|point| point.phase == phase)
+                    .count(),
+                1,
+                "{} 那一段摆一个二次崩溃：{second_crash_points:?}",
+                phase.name()
+            );
+        }
+    }
+
     /// 造写表用：一个 512 字节的写；根槽写的字节里在偏移 24、28 写上根身份（`root_identity_written_by` 的读法）。
     fn synthetic_write(kind: StepKind, root: Option<(u32, u64)>) -> RetainedWrite {
         let mut bytes = vec![0u8; 512];
@@ -4120,6 +4324,7 @@ mod tests {
                     &[row_publish, warm_up, continuation_step.clone()],
                     std::slice::from_ref(&continuation_step),
                 ),
+                None,
             );
         assert_eq!(
             phases_of_a_mount_that_failed_in_the_second_continuation_step[5..],
diff --git a/crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs b/crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs
new file mode 100644
index 0000000..5911bcf
--- /dev/null
+++ b/crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs
@@ -0,0 +1,655 @@
+//! checker 档模块：crash、layer0_progress、crash_injection
+//! 实例表清扫的崩溃点（D18（块里携带什么信息） 已定项 11「回收」一条；判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md`
+//! 第三节 F5 与第二节 Q5；C597（实例表行只增不删，删行的条件没有条款） 的判别力自证）。
+//!
+//! 流：S = 4 的 12 槽小环、两块 128 MiB 内存盘。mkfs → 实例 1 第一个文件、覆盖写两次 → 一次两个数据单元的顺序写崩在第一条 journal 记录写之前
+//! （孤儿：按实例 1 那一行判未发布）→ 实例 2 可写挂载、覆盖写到 txg 19（实例 1 的根全部轮出环）→ 实例 3 可写挂载，末尾清扫（有一行就扫）：
+//! 抹孤儿的头、每块盘一道屏障、发清扫发布删掉实例 1 那一行。基线是实例 3 那次挂载之前的盘面，枚举的是那次挂载的全部写。
+//!
+//! 比已有的流多罩的崩溃状态：已有的层 0 流与崩溃注入里没有一次挂载写过清扫的抹头与清扫发布（产品路径的阈值占位 369 行，那几条流到不了）；
+//! 这一条的挂载末尾两段（`crash_injection::WritableMountPhase` 的抹头与清扫发布）上每个崩溃状态都枚举。
+//! 多跑的一步：清扫两段上的每个崩溃状态再起一次可写挂载（也是有一行就扫），判挂载做成、池级 checker 不红、F5 照判。
+//!
+//! F5（判决第三节 F5，C597 的验收）：删行那一版生效的每个崩溃态里，被删实例按删前那一行判未发布的单元一个不剩——行一删，盘上再没有东西说它们
+//! 没发布过（池级 checker 与 oracle 都看不见），只有看得见删行之前那一行的观察者判得了。判法在这份用例里另写一份（认头只看魔数、类、头校验和，
+//! 照认领规则按整单元跳），不调实现的清扫器。
+#[path = "../../singlefs-harness/tests/common_root_ring_holes/mod.rs"]
+mod common_root_ring_holes;
+
+use std::collections::{BTreeMap, BTreeSet};
+
+use common_root_ring_holes::{
+    content_of_the_version, devices_of, fixed_geometry_of_the_ring_hole_pools,
+    parameters_of_the_ring_hole_pools, pool_of, recording_of, RingHoleDevices,
+    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, JOURNAL_RING_BYTES_OF_THE_RING_HOLE_POOLS,
+    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
+};
+use singlefs_checker_tier::crash::{
+    enumerate_layer0_in_state_slices, full_expansion, quick_tier_expansion, Layer0ObserverCounts,
+    Layer0Parallelism, Layer0SegmentExpansion, Layer0Tally,
+};
+use singlefs_checker_tier::crash_injection::{
+    materialized_crash_image, publishes_before_the_instance_table_sweep_of_the_mount_output,
+    writes_of_the_instance_table_sweep_phases, WritableMountPhase,
+};
+use singlefs_checker_tier::layer0_progress::Layer0Resume;
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::allocator::UnitAreaStart;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::instance_table::InstanceRow;
+use singlefs_core::instance_table_sweep::{
+    InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
+    InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
+    SweepPublishPlacements,
+};
+use singlefs_core::make_filesystem::make_filesystem;
+use singlefs_core::mount::{
+    mount_writable, mount_writable_with_instance_table_sweep_configuration, Mounted, ShadowLedger,
+};
+use singlefs_core::recovery::{instance_table_chain_of_root, RecoveryReport};
+use singlefs_core::transaction::{
+    publish_first_file, publish_overwrite, publish_sequential_write, FirstFile, PoolWriter,
+    TransactionOutput,
+};
+use singlefs_core::unit::{data_unit_payload_capacity, unit_filesystem_identifier};
+use singlefs_format::SLOT_BYTES;
+use singlefs_harness::memory_pool::{
+    writes_and_segments, CrashImage, MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice,
+};
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::SharedStream;
+
+/// 实例 2 覆盖写到的 txg：根环 12 槽里只剩 txg 8–19。
+const LAST_TXG_BEFORE_THE_SWEEPING_MOUNT: u64 = 19;
+/// 崩掉的那次顺序写写几个数据单元（孤儿少，抹头那一段的状态数小）。
+const ORPHAN_DATA_UNITS: usize = 2;
+/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
+const INVARIANTS_JUDGED_AFTER_THE_REMOUNT: [&str; 5] =
+    ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];
+/// 码 1 / 码 2 / 码 3 的类标签、明文头宽与 key 宽、头校验和的位置（D18 已定项 7 / 16 / 17 / 18；这份用例自己的一份，不取实现的常量）。
+const UNIT_MAGIC: &[u8; 4] = b"SFSU";
+const DATA_UNIT_CLASS: u8 = 1;
+const INDEX_NODE_CLASS: u8 = 2;
+const PACKED_UNIT_CLASS: u8 = 3;
+const DATA_UNIT_HEADER_END: usize = 105;
+const PACKED_UNIT_HEADER_END: usize = 107;
+const INDEX_NODE_HEADER_END_WITHOUT_THE_KEY_RANGE: usize = 86;
+const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
+const HEADER_CHECKSUM_OFFSET: usize = 10;
+const HEADER_CHECKSUM_FIELD_BYTES: usize = 32;
+
+/// 这份用例认出来的一个单元头：类、写序实例、诞生代号、码 1 / 码 3 的事务号、fsid 低 8 字节。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct UnitHeadSeenByTheObserver {
+    unit_class: u8,
+    writer_instance: u32,
+    birth_txg: u64,
+    transaction: u64,
+    filesystem_identifier: u64,
+}
+
+fn u64_at(bytes: &[u8], offset: usize) -> u64 {
+    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
+}
+
+/// 魔数、类（定头宽）、头校验和（CRC32C 罩 [0, 头末)，字段 10..42 置零后算，存 10..14、其余 28 字节为零）过就认；不看载荷。
+fn unit_head_seen_by_the_observer(bytes: &[u8]) -> Option<UnitHeadSeenByTheObserver> {
+    if bytes.len() <= INDEX_NODE_KEY_WIDTH_OFFSET || &bytes[..4] != UNIT_MAGIC {
+        return None;
+    }
+    let key_range_bytes = 2 * usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
+    let header_end = match bytes[6] {
+        DATA_UNIT_CLASS => DATA_UNIT_HEADER_END,
+        INDEX_NODE_CLASS => INDEX_NODE_HEADER_END_WITHOUT_THE_KEY_RANGE + key_range_bytes,
+        PACKED_UNIT_CLASS => PACKED_UNIT_HEADER_END,
+        _ => return None,
+    };
+    if bytes.len() < header_end {
+        return None;
+    }
+    let mut covered = bytes[..header_end].to_vec();
+    covered[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + HEADER_CHECKSUM_FIELD_BYTES].fill(0);
+    let expected = singlefs_checker::crc32_castagnoli_bitwise(&covered).to_le_bytes();
+    let stored =
+        &bytes[HEADER_CHECKSUM_OFFSET..HEADER_CHECKSUM_OFFSET + HEADER_CHECKSUM_FIELD_BYTES];
+    if stored[..4] != expected || stored[4..].iter().any(|byte| *byte != 0) {
+        return None;
+    }
+    let six_bytes_at = |offset: usize| {
+        let mut widened = [0u8; 8];
+        widened[..6].copy_from_slice(&bytes[offset..offset + 6]);
+        u64::from_le_bytes(widened)
+    };
+    Some(match bytes[6] {
+        DATA_UNIT_CLASS => UnitHeadSeenByTheObserver {
+            unit_class: DATA_UNIT_CLASS,
+            birth_txg: u64_at(bytes, 75),
+            filesystem_identifier: u64_at(bytes, 83),
+            writer_instance: u32::from_le_bytes(bytes[91..95].try_into().expect("4 字节")),
+            transaction: six_bytes_at(95),
+        },
+        PACKED_UNIT_CLASS => UnitHeadSeenByTheObserver {
+            unit_class: PACKED_UNIT_CLASS,
+            birth_txg: u64_at(bytes, 73),
+            filesystem_identifier: u64_at(bytes, 81),
+            writer_instance: u32::from_le_bytes(bytes[93..97].try_into().expect("4 字节")),
+            transaction: six_bytes_at(97),
+        },
+        _ => UnitHeadSeenByTheObserver {
+            unit_class: INDEX_NODE_CLASS,
+            birth_txg: u64_at(bytes, 52 + key_range_bytes),
+            filesystem_identifier: u64_at(bytes, 60 + key_range_bytes),
+            writer_instance: u32::from_le_bytes(
+                bytes[68 + key_range_bytes..72 + key_range_bytes]
+                    .try_into()
+                    .expect("4 字节"),
+            ),
+            transaction: 0,
+        },
+    })
+}
+
+/// 已发布谓词有行那一支：码 1 ⇒ b ≤ T ∨ n ≤ W，码 2 / 3 ⇒ b ≤ T；都不成立就是按这一行未发布。
+fn is_unpublished_by_the_row(head: &UnitHeadSeenByTheObserver, row: &InstanceRow) -> bool {
+    if head.unit_class == DATA_UNIT_CLASS {
+        !(head.birth_txg <= row.selected_root_txg.0
+            || head.transaction <= row.applied_transaction_high_water)
+    } else {
+        head.birth_txg > row.selected_root_txg.0
+    }
+}
+
+/// 盘上按 `row` 判未发布、写序实例是 `row` 那个实例的本池单元有几个：每块盘按槽升序走写过的槽，认出一个头就按整单元跳（码 1、码 3 两槽，码 2 一槽）。
+fn unpublished_units_of_the_row_left_on(image: &MemoryPool, row: &InstanceRow) -> u64 {
+    let unit_area_start =
+        UnitAreaStart::following_the_journal_ring(JOURNAL_RING_BYTES_OF_THE_RING_HOLE_POOLS)
+            .expect("1 MiB 环的单元区起点在段边界上")
+            .slot()
+            .0;
+    let filesystem_identifier =
+        unit_filesystem_identifier(&parameters_of_the_ring_hole_pools().filesystem_identifier);
+    let sectors_per_slot = SLOT_BYTES / 512;
+    let mut left = 0;
+    for device in image.devices.values() {
+        let written_slots: BTreeSet<u64> = device
+            .written_sectors()
+            .keys()
+            .map(|sector| sector / sectors_per_slot)
+            .filter(|slot| *slot >= unit_area_start)
+            .collect();
+        let mut claimed_until = 0;
+        for slot in written_slots {
+            if slot < claimed_until {
+                continue;
+            }
+            let bytes = device.read(
+                singlefs_core::address::DeviceOffsetInBytes(slot * SLOT_BYTES),
+                1024,
+            );
+            let Some(head) = unit_head_seen_by_the_observer(&bytes) else {
+                continue;
+            };
+            claimed_until = slot
+                + if head.unit_class == INDEX_NODE_CLASS {
+                    1
+                } else {
+                    2
+                };
+            if head.filesystem_identifier == filesystem_identifier
+                && head.writer_instance == row.instance.0
+                && is_unpublished_by_the_row(&head, row)
+            {
+                left += 1;
+            }
+        }
+    }
+    left
+}
+
+fn publish_an_overwrite(
+    devices: &mut RingHoleDevices,
+    allocator: &mut singlefs_core::allocator::PoolAllocator,
+    current: &TransactionOutput,
+    content: &[u8],
+    instance: InstanceGeneration,
+) -> TransactionOutput {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let salt = current.root.checkpoint_txg.0 + 1;
+    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+    publish_overwrite(
+        &mut writer,
+        allocator,
+        current,
+        FirstFile {
+            content,
+            write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
+        },
+        instance,
+    )
+    .expect("覆盖写")
+}
+
+/// 清扫那次挂载之前的盘面：实例 1 的孤儿留在盘上、实例 1 的根全部轮出环。
+fn pool_with_a_dead_instance() -> MemoryPool {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let mut devices: RingHoleDevices = (0..2u32)
+        .map(|number| {
+            (
+                DeviceIdentity(number),
+                SparseBlockDevice::new(
+                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
+                    PhysicalBlockSizeInBytes(512),
+                ),
+            )
+        })
+        .collect();
+    make_filesystem(&parameters, &mut devices).expect("mkfs");
+    let Mounted {
+        output,
+        mut allocator,
+        current,
+    } = mount_writable(&parameters, &mut devices).expect("实例 1 的可写挂载");
+    let instance = output.instance;
+    let mut current = {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut allocator,
+            current.root(),
+            FirstFile {
+                content: &content_of_the_version(0),
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
+            },
+            instance,
+            current.record_bytes(),
+        )
+        .expect("第一个文件")
+    };
+    // 迭代上界：两次覆盖写。
+    for _ in 0..2 {
+        current = publish_an_overwrite(
+            &mut devices,
+            &mut allocator,
+            &current,
+            &content_of_the_version(current.root.checkpoint_txg.0 + 1),
+            instance,
+        );
+    }
+    let before = pool_of(&devices);
+    let stream = SharedStream::retaining_contents();
+    let mut recording = recording_of(&before, &stream);
+    {
+        let mut writer = PoolWriter::new(&parameters, recording.as_mut_slice());
+        publish_sequential_write(
+            &mut writer,
+            &mut allocator.clone(),
+            &current,
+            FirstFile {
+                content: &vec![7u8; ORPHAN_DATA_UNITS * data_unit_payload_capacity()],
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + 100,
+            },
+            instance,
+        )
+        .expect("顺序写（录下来、之后截掉）");
+    }
+    let (writes, _segments) = writes_and_segments(
+        &stream.retained_operations(),
+        &fixed_geometry_of_the_ring_hole_pools(),
+    );
+    let first_record_write = writes
+        .iter()
+        .position(|write| write.kind == StepKind::JournalRecord)
+        .expect("顺序写有 journal 记录写");
+    let mut image = before;
+    image.apply_writes(&writes[..first_record_write]);
+    let mut devices_of_instance_two = devices_of(&image);
+    let mounted =
+        mount_writable(&parameters, &mut devices_of_instance_two).expect("实例 2 的可写挂载");
+    let mut allocator_of_instance_two = mounted.allocator;
+    let mut current_of_instance_two = mounted.current.into_file_version().expect("这一版带文件");
+    // 迭代上界：覆盖写到 txg 19。
+    while current_of_instance_two.root.checkpoint_txg.0 < LAST_TXG_BEFORE_THE_SWEEPING_MOUNT {
+        current_of_instance_two = publish_an_overwrite(
+            &mut devices_of_instance_two,
+            &mut allocator_of_instance_two,
+            &current_of_instance_two,
+            &content_of_the_version(current_of_instance_two.root.checkpoint_txg.0 + 1),
+            InstanceGeneration(2),
+        );
+    }
+    pool_of(&devices_of_instance_two)
+}
+
+fn sweeping_on_any_row() -> InstanceTableSweepConfiguration {
+    InstanceTableSweepConfiguration {
+        switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
+        row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
+        sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
+    }
+}
+
+/// 清扫那次挂载录下来：基线、写表与段、被判的根槽写（清扫发布那一次）、允许的版本、删前那一行、清扫发布那一版、清扫两段各是哪几个写。
+struct RecordedSweepingMount {
+    base: MemoryPool,
+    writes: Vec<RetainedWrite>,
+    segments: Vec<Vec<usize>>,
+    judged_root_index: usize,
+    versions: Vec<PublishedVersion>,
+    row_deleted: InstanceRow,
+    sweep_publish: (InstanceGeneration, CheckpointTxg),
+    writes_of_the_sweep_phases: BTreeSet<usize>,
+}
+
+fn record_the_sweeping_mount() -> RecordedSweepingMount {
+    let base = pool_with_a_dead_instance();
+    let stream = SharedStream::retaining_contents();
+    let mut devices = recording_of(&base, &stream);
+    let mounted = mount_writable_with_instance_table_sweep_configuration(
+        &parameters_of_the_ring_hole_pools(),
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        sweeping_on_any_row(),
+    )
+    .expect("实例 3 的可写挂载");
+    let InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
+        rows_deleted,
+        heads_erased,
+    }) = &mounted.output.instance_table_sweep
+    else {
+        panic!(
+            "前提：没崩过的那次挂载删了实例 1 的行：{:?}",
+            mounted.output.instance_table_sweep
+        );
+    };
+    assert!(!heads_erased.is_empty(), "前提：抹了孤儿的头");
+    let [row_deleted] = rows_deleted.as_slice() else {
+        panic!("前提：只删实例 1 那一行：{rows_deleted:?}");
+    };
+    let publishes_before_the_sweep =
+        publishes_before_the_instance_table_sweep_of_the_mount_output(&mounted.output)
+            .expect("清扫写了东西");
+    let (writes, segments) = writes_and_segments(
+        &stream.retained_operations(),
+        &fixed_geometry_of_the_ring_hole_pools(),
+    );
+    let judged_root_index = writes
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .expect("清扫发布写了根槽");
+    let writes_by_phase =
+        writes_of_the_instance_table_sweep_phases(&writes, publishes_before_the_sweep);
+    let writes_of_the_sweep_phases: BTreeSet<usize> = [
+        WritableMountPhase::ErasingHeadsOfTheInstanceTableSweep,
+        WritableMountPhase::InstanceTableSweepPublish,
+    ]
+    .iter()
+    .flat_map(|phase| writes_by_phase.get(phase).cloned().unwrap_or_default())
+    .collect();
+    let content = content_of_the_version(LAST_TXG_BEFORE_THE_SWEEPING_MOUNT);
+    let mut versions = vec![PublishedVersion {
+        instance: InstanceGeneration(2),
+        checkpoint_txg: CheckpointTxg(LAST_TXG_BEFORE_THE_SWEEPING_MOUNT),
+        content: content.clone(),
+    }];
+    for write in writes
+        .iter()
+        .filter(|write| write.kind == StepKind::RootRecordFua)
+    {
+        let root = singlefs_core::root_record::RootRecord::parse_slot(
+            write.bytes().expect("根槽写带字节"),
+            &parameters_of_the_ring_hole_pools().filesystem_identifier,
+        )
+        .expect("挂载写的根自证得过");
+        versions.push(PublishedVersion {
+            instance: root.instance,
+            checkpoint_txg: root.checkpoint_txg,
+            content: content.clone(),
+        });
+    }
+    RecordedSweepingMount {
+        base,
+        writes,
+        segments,
+        judged_root_index,
+        versions,
+        row_deleted: *row_deleted,
+        sweep_publish: (
+            mounted.current.root().instance,
+            mounted.current.root().checkpoint_txg,
+        ),
+        writes_of_the_sweep_phases,
+    }
+}
+
+/// F5 那一判在一个崩溃态上：恢复落到清扫发布那一版（删行那一版生效）时，删前那一行判未发布的单元一个都不许剩；交回这个态是不是删行那一版生效的。
+fn deletion_took_effect_and_no_unpublished_unit_is_left(
+    recorded: &RecordedSweepingMount,
+    image: &CrashImage<'_>,
+    report: &RecoveryReport,
+) -> bool {
+    if report.effective_root != Some(recorded.sweep_publish) {
+        return false;
+    }
+    let left = unpublished_units_of_the_row_left_on(
+        &materialized_crash_image(image),
+        &recorded.row_deleted,
+    );
+    assert_eq!(
+        left, 0,
+        "F5：删行那一版生效的崩溃态里，按删前那一行 {:?} 判未发布的单元剩 {left} 个",
+        recorded.row_deleted
+    );
+    true
+}
+
+fn assert_the_recovery_and_the_checker_are_clean(tally: &Layer0Tally) {
+    assert_eq!(
+        (tally.violations, tally.first_violation.clone()),
+        (0, None),
+        "恢复之后多版本 oracle 0 违例"
+    );
+    assert_eq!(tally.failed_states, 0, "每个状态都恢复得了");
+    assert_eq!(
+        tally.checker_violated_states.values().sum::<u64>(),
+        0,
+        "恢复之后池级 checker 0 违例：{:?}",
+        tally.checker_first_violation
+    );
+}
+
+/// 枚举清扫那次挂载的写，每个状态照 F5 判；交回 tally 与删行那一版生效的状态数。
+fn enumerate_the_sweeping_mount_judging_f5(
+    recorded: &RecordedSweepingMount,
+    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
+) -> (Layer0Tally, u64) {
+    let mut states_where_the_deletion_took_effect = 0u64;
+    let mut observe =
+        |image: &CrashImage<'_>, report: &RecoveryReport, _counts: &mut Layer0ObserverCounts| {
+            if deletion_took_effect_and_no_unpublished_unit_is_left(recorded, image, report) {
+                states_where_the_deletion_took_effect += 1;
+            }
+        };
+    let tally = enumerate_layer0_in_state_slices(
+        &recorded.base,
+        &recorded.writes,
+        &recorded.segments,
+        recorded.judged_root_index,
+        &recorded.versions,
+        expansion,
+        Layer0Parallelism::from_environment(),
+        Some(&mut observe),
+        &Layer0Resume::NoProgressFile,
+    );
+    (tally, states_where_the_deletion_took_effect)
+}
+
+/// 快档：清扫那次挂载的写按甲二展开，每个状态跑恢复、oracle、池级 checker 与 F5。
+/// 判别力：清扫发布照发、头不抹（「不抹就删」）时，删行那一版生效的态里孤儿头都还在，F5 红。
+#[test]
+fn quick_tier_of_the_sweeping_mount_leaves_no_unpublished_unit_of_the_deleted_row() {
+    let recorded = record_the_sweeping_mount();
+    assert!(
+        !recorded.writes_of_the_sweep_phases.is_empty(),
+        "前提：挂载写里有清扫两段"
+    );
+    let (tally, states_where_the_deletion_took_effect) =
+        enumerate_the_sweeping_mount_judging_f5(&recorded, &quick_tier_expansion);
+    assert_the_recovery_and_the_checker_are_clean(&tally);
+    assert!(
+        states_where_the_deletion_took_effect > 0,
+        "F5 判到了删行那一版生效的状态（{} 个状态里 0 个，说明枚举没走到清扫发布落盘之后）",
+        tally.states
+    );
+}
+
+/// 全量：清扫那次挂载的每一段都展开（D13（验证路线） 已定项 4 的全量），每个状态跑恢复、oracle、池级 checker 与 F5。
+#[test]
+#[ignore = "checker 档全量崩溃枚举：登记在 .claude/gate.d/stage-inputs.tsv 的 crash-case:instance-table-sweep-mount，由 checker-tier-layer0-replay --full 跑"]
+fn every_crash_state_of_the_sweeping_mount_leaves_no_unpublished_unit_of_the_deleted_row() {
+    let recorded = record_the_sweeping_mount();
+    let (tally, states_where_the_deletion_took_effect) =
+        enumerate_the_sweeping_mount_judging_f5(&recorded, &full_expansion);
+    assert_the_recovery_and_the_checker_are_clean(&tally);
+    assert!(
+        states_where_the_deletion_took_effect > 0,
+        "F5 判到了删行那一版生效的状态"
+    );
+}
+
+/// 清扫两段（抹头、清扫发布）上的每个崩溃状态再起一次可写挂载（有一行就扫）：挂载做成，之后的盘面上那几条不变量不红，
+/// 删行那一版在那一刻生效的照 F5 判一个不剩；挂载之后实例 1 那一行要么已删、要么这一次挂载又删了（清扫这一轮整轮重扫重抹，判决第二节 Q5）。
+#[test]
+#[ignore = "checker 档全量崩溃枚举：登记在 .claude/gate.d/stage-inputs.tsv 的 crash-case:instance-table-sweep-phases-remount，由 checker-tier-layer0-replay --full 跑"]
+fn every_crash_state_inside_the_two_sweep_phases_remounts_writable_and_ends_without_the_deleted_row(
+) {
+    let recorded = record_the_sweeping_mount();
+    let sweep_phase_writes = recorded.writes_of_the_sweep_phases.clone();
+    let expand_the_sweep_phases_only = |_segment_index: usize, segment: &[usize]| {
+        if segment
+            .iter()
+            .any(|write_index| sweep_phase_writes.contains(write_index))
+        {
+            Layer0SegmentExpansion::EveryProperSubset
+        } else {
+            Layer0SegmentExpansion::NotExpanded
+        }
+    };
+    let mut persisted_sets: Vec<Vec<bool>> = Vec::new();
+    let mut enumerated_writes: Option<Vec<RetainedWrite>> = None;
+    let mut collect =
+        |image: &CrashImage<'_>, report: &RecoveryReport, _counts: &mut Layer0ObserverCounts| {
+            deletion_took_effect_and_no_unpublished_unit_is_left(&recorded, image, report);
+            enumerated_writes.get_or_insert_with(|| image.writes.to_vec());
+            persisted_sets.push(image.persisted.clone());
+        };
+    let parallelism = Layer0Parallelism::from_environment();
+    let tally = enumerate_layer0_in_state_slices(
+        &recorded.base,
+        &recorded.writes,
+        &recorded.segments,
+        recorded.judged_root_index,
+        &recorded.versions,
+        &expand_the_sweep_phases_only,
+        parallelism,
+        Some(&mut collect),
+        &Layer0Resume::NoProgressFile,
+    );
+    assert_the_recovery_and_the_checker_are_clean(&tally);
+    let enumerated_writes = enumerated_writes.expect("观察者至少看过一个状态");
+    let worker_threads = parallelism.worker_threads.get();
+    let chunk_length = persisted_sets.len().div_ceil(worker_threads).max(1);
+    let failures: Vec<String> = std::thread::scope(|scope| {
+        let handles: Vec<_> = persisted_sets
+            .chunks(chunk_length)
+            .map(|chunk| {
+                let base = &recorded.base;
+                let writes = &enumerated_writes;
+                let row_deleted = recorded.row_deleted;
+                scope.spawn(move || {
+                    let mut failures = Vec::new();
+                    for persisted in chunk {
+                        let mut image = base.clone();
+                        let landed: Vec<RetainedWrite> = writes
+                            .iter()
+                            .zip(persisted)
+                            .filter(|(_, is_persisted)| **is_persisted)
+                            .map(|(write, _)| write.clone())
+                            .collect();
+                        image.apply_writes(&landed);
+                        let stream = SharedStream::retaining_contents();
+                        let mut devices = recording_of(&image, &stream);
+                        let remounted = mount_writable_with_instance_table_sweep_configuration(
+                            &parameters_of_the_ring_hole_pools(),
+                            &mut devices,
+                            SpaceAdmission::JudgedByTheFormula,
+                            ShadowLedger::On,
+                            sweeping_on_any_row(),
+                        );
+                        let mounted = match remounted {
+                            Ok(mounted) => mounted,
+                            Err(error) => {
+                                failures.push(format!("再挂被拒：{error:?}"));
+                                continue;
+                            }
+                        };
+                        let mut after = image.clone();
+                        after.apply(&stream.retained_operations());
+                        let violations: Vec<String> = singlefs_checker::walk::check_pool_image(
+                            &after,
+                        )
+                        .into_iter()
+                        .filter(|(invariant, _)| {
+                            INVARIANTS_JUDGED_AFTER_THE_REMOUNT.contains(invariant)
+                        })
+                        .filter_map(|(invariant, verdict)| match verdict {
+                            singlefs_checker::image::InvariantVerdict::Violated(detail) => {
+                                Some(format!("{invariant}：{detail}"))
+                            }
+                            singlefs_checker::image::InvariantVerdict::Holds
+                            | singlefs_checker::image::InvariantVerdict::NotApplicable(_) => None,
+                        })
+                        .collect();
+                        if !violations.is_empty() {
+                            failures.push(format!("再挂之后 checker 红：{violations:?}"));
+                        }
+                        let rows_after: Vec<InstanceGeneration> =
+                            instance_table_chain_of_root(&after, mounted.current.root())
+                                .map(|chain| {
+                                    chain.records.rows.iter().map(|row| row.instance).collect()
+                                })
+                                .unwrap_or_default();
+                        if rows_after.contains(&row_deleted.instance) {
+                            failures.push(format!("再挂之后实例 1 那一行还在：{rows_after:?}"));
+                        }
+                        let left = unpublished_units_of_the_row_left_on(&after, &row_deleted);
+                        if left != 0 {
+                            failures.push(format!("再挂之后删前那一行判未发布的单元剩 {left} 个"));
+                        }
+                    }
+                    failures
+                })
+            })
+            .collect();
+        handles
+            .into_iter()
+            .flat_map(|handle| handle.join().expect("工作线程没 panic"))
+            .collect()
+    });
+    let distinct: BTreeMap<String, usize> =
+        failures
+            .iter()
+            .fold(BTreeMap::new(), |mut counts, failure| {
+                *counts
+                    .entry(failure.chars().take(160).collect())
+                    .or_insert(0) += 1;
+                counts
+            });
+    assert!(
+        failures.is_empty(),
+        "清扫两段上的 {} 个崩溃态里 {} 个再挂不对：{distinct:?}",
+        persisted_sets.len(),
+        failures.len()
+    );
+}
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index ec11a84..8448274 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -957,8 +957,9 @@ pub enum RootRingOccupant {
     },
     /// 挂载那一刻读不出的槽，这个进程在它成了窗口里最小的那个时重读、读出了一条自证过而按实例表仍然有效的根
     /// （D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定，`research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表二之后那一段）：
-    /// 照读出来的 txg 进窗口。与 [`RootRingOccupant::UnreadableAtMountInsideTheWindow`] 一样不算「这个进程知道住着根的槽」：D16 已定项 1
-    /// 「根槽这一次读坏」那一行列的是挂载那一刻读得出的与这个进程写过的两种，进程内重读读出来的不在其内。
+    /// 照读出来的 txg 进窗口。它算「这个进程知道住着根的槽」（[`RootRingOccupancy::ring_slots_known_to_hold_a_root`]；D16 已定项 1
+    /// 「根槽这一次读坏」那一行，用户 2026-10-04 定，`research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1）：
+    /// 之后那一槽再读坏，算上限、判回退候选时重读一次、仍坏就拒，不按没有根往下走。
     ValidRootReadWhenTheSlotUnreadableAtMountWasReread { checkpoint_txg: CheckpointTxg },
     /// 按实例表判仍然有效的根，txg 在不在 F 之下都算（可再分配谓词取 max(F_生效, 环里最旧有效根)，F 那一半另记）。
     ValidRoot { checkpoint_txg: CheckpointTxg },
@@ -1014,19 +1015,21 @@ impl RootRingOccupancy {
     }
 
     /// 这个进程知道住着一条根的根环槽（D16（发布语义） 已定项 1「根槽这一次读坏」那一行）：挂载那一刻读得出、自证过的槽
-    /// （有效的与被抛弃的都算），与这个进程之后写过、FUA 返回过的槽——发布在落盘之前或落盘途中失败时分配器整个换回发布之前那一份，
-    /// 这张表随之退回，记着的只剩根落盘做成的那几次。挂载那一刻读不出或自证不过、之后又没写过的槽不在其内
-    /// （[`RootRingOccupant::UnreadableAtMountInsideTheWindow`] 记着的槽也不在）。
+    /// （有效的与被抛弃的都算），这个进程之后写过、FUA 返回过的槽——发布在落盘之前或落盘途中失败时分配器整个换回发布之前那一份，
+    /// 这张表随之退回，记着的只剩根落盘做成的那几次——，与进程内记根时重读读出了有效根的槽
+    /// （[`RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread`]，用户 2026-10-04 定）。挂载那一刻读不出或自证不过、
+    /// 之后又没写过也没重读出有效根的槽不在其内（[`RootRingOccupant::UnreadableAtMountInsideTheWindow`] 记着的槽也不在）。
     #[must_use]
     pub fn ring_slots_known_to_hold_a_root(&self) -> BTreeSet<RootRingSlot> {
         self.occupants
             .iter()
             .filter(|(_, occupant)| match occupant {
-                RootRingOccupant::ValidRoot { .. } | RootRingOccupant::AbandonedRoot { .. } => true,
-                RootRingOccupant::UnreadableAtMountInsideTheWindow { .. }
+                RootRingOccupant::ValidRoot { .. }
+                | RootRingOccupant::AbandonedRoot { .. }
                 | RootRingOccupant::ValidRootReadWhenTheSlotUnreadableAtMountWasReread { .. } => {
-                    false
+                    true
                 }
+                RootRingOccupant::UnreadableAtMountInsideTheWindow { .. } => false,
             })
             .map(|(ring_slot, _)| *ring_slot)
             .collect()
@@ -1101,20 +1104,22 @@ impl RootRingOccupancy {
     }
 }
 
-/// 进程内重读一个挂载那一刻读不出的根环槽读到了什么（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定）。封闭集合，`match` 不写通配臂。
+/// 进程内重读一个挂载那一刻读不出的根环槽读到了什么（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定；读得出而自证不过那一格
+/// 用户 2026-10-04 定改法 A，`research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1）。封闭集合，`match` 不写通配臂。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum RereadOfARootRingSlotUnreadableAtMount {
-    /// 读得出、自证过，按实例表仍然有效：照读出来的 txg 算（窗口里的进窗口，窗口外的不算）。
+    /// 读得出、自证过，按实例表仍然有效：照读出来的 txg 算（窗口里的进窗口，窗口外的不算）；那一槽从此算「这个进程知道住着根的槽」。
     ValidRoot { checkpoint_txg: CheckpointTxg },
-    /// 读得出而自证不过，或读出来的根按实例表被抛弃：当没有根（与挂载那一刻读得出而自证不过的槽同一个处置）。
-    NoValidRoot,
-    /// 仍读不出：照旧按它能住的最大 txg 算。
-    StillUnreadable,
+    /// 读得出、自证过，读出来的根按实例表被抛弃：当没有根。
+    AbandonedByTheInstanceTable,
+    /// 仍读不出，或读得出而自证不过（一次读坏的有效根与从没写过的槽这一刻分不开）：照旧按它能住的最大 txg 算。
+    StillUnreadableOrNotSelfVerified,
 }
 
 /// 一圈窗口里最旧有效根，挂载那一刻读不出的槽按 E2 并进来（D16（发布语义） 已定项 1，用户 2026-10-03 定）：`valid_root_txgs` 是已知
 /// 住着有效根的 txg，`unreadable_at_mount` 是挂载那一刻读不出、各按能住的最大 txg 记的槽；两者里窗口（锚 `window_anchor`）里最小的那个
-/// 若是一个读不出的槽，先重读那一槽（`reread`）：读出有效根就照它的 txg 换进去、当没有根就拿掉，再取最小；仍读不出才按最大 txg 算。
+/// 若是一个读不出的槽，先重读那一槽（`reread`）：读出有效根就照它的 txg 换进去、读出被抛弃的根就拿掉，再取最小；
+/// 仍读不出或读得出而自证不过才按最大 txg 算（自证不过那一格用户 2026-10-04 定改法 A）。
 /// 交回最小的 txg（一条都没有时 `None`）与每个重读过的槽读到了什么（按重读的次序）。
 ///
 /// 循环：每轮要么换掉或拿掉一个读不出的槽（`unreadable_at_mount` 少一个），要么在最小的是一个仍读不出的槽、或最小的不是读不出的槽时返回；
@@ -1158,10 +1163,10 @@ pub fn oldest_txg_in_the_window_rereading_slots_unreadable_at_mount(
                 unreadable_at_mount.remove(&ring_slot);
                 valid_root_txgs.push(checkpoint_txg);
             }
-            RereadOfARootRingSlotUnreadableAtMount::NoValidRoot => {
+            RereadOfARootRingSlotUnreadableAtMount::AbandonedByTheInstanceTable => {
                 unreadable_at_mount.remove(&ring_slot);
             }
-            RereadOfARootRingSlotUnreadableAtMount::StillUnreadable => {
+            RereadOfARootRingSlotUnreadableAtMount::StillUnreadableOrNotSelfVerified => {
                 return (Some(highest_txg_it_can_hold), rereads);
             }
         }
@@ -1854,8 +1859,8 @@ impl PoolAllocator {
     }
 
     /// 同 [`Self::record_root_written_by_this_process`]，记之前先照 E2 重读挂载那一刻读不出、记了这条根之后会是窗口里最小的那个槽
-    /// （D16（发布语义） 已定项 1，用户 2026-10-03 定：读得出且自证过就照读出来的根算——窗口里、按实例表有效的照它的 txg，别的当没有；
-    /// 仍读不出才按它能住的最大 txg 算）。`reread` 读盘、判自证与实例表（`mount::reread_of_a_root_ring_slot_unreadable_at_mount`）；
+    /// （D16（发布语义） 已定项 1，用户 2026-10-03 定：读得出且自证过就照读出来的根算——按实例表有效的照它的 txg（窗口外的不进窗口），
+    /// 被抛弃的当没有；仍读不出或读得出而自证不过按它能住的最大 txg 算，用户 2026-10-04 定）。`reread` 读盘、判自证与实例表（`mount::reread_of_a_root_ring_slot_unreadable_at_mount`）；
     /// 这里只定重读哪一槽、按读到的改表。读不出的槽一个都不在窗口里最小的位置上时一个都不读。没装根环时只记根（什么都不做）。
     pub fn record_root_written_by_this_process_rereading_slots_unreadable_at_mount(
         &mut self,
@@ -1903,10 +1908,10 @@ impl PoolAllocator {
                             },
                         );
                     }
-                    RereadOfARootRingSlotUnreadableAtMount::NoValidRoot => {
+                    RereadOfARootRingSlotUnreadableAtMount::AbandonedByTheInstanceTable => {
                         occupancy.occupants.remove(&ring_slot);
                     }
-                    RereadOfARootRingSlotUnreadableAtMount::StillUnreadable => {}
+                    RereadOfARootRingSlotUnreadableAtMount::StillUnreadableOrNotSelfVerified => {}
                 }
             }
         }
@@ -2769,13 +2774,13 @@ mod tests {
     fn the_slot_unreadable_at_mount_still_unreadable_on_the_reread_counts_by_its_highest_txg() {
         assert_eq!(
             oldest_in_the_window_when_the_slot_unreadable_at_mount_rereads_as(
-                RereadOfARootRingSlotUnreadableAtMount::StillUnreadable
+                RereadOfARootRingSlotUnreadableAtMount::StillUnreadableOrNotSelfVerified
             ),
             (
                 Some(CheckpointTxg(12)),
                 vec![(
                     RootRingSlot { region: 0, slot: 0 },
-                    RereadOfARootRingSlotUnreadableAtMount::StillUnreadable
+                    RereadOfARootRingSlotUnreadableAtMount::StillUnreadableOrNotSelfVerified
                 )]
             )
         );
diff --git a/crates/singlefs-core/src/instance_table_sweep.rs b/crates/singlefs-core/src/instance_table_sweep.rs
new file mode 100644
index 0000000..5e56557
--- /dev/null
+++ b/crates/singlefs-core/src/instance_table_sweep.rs
@@ -0,0 +1,902 @@
+//! 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条；判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md`
+//! 第二节 S1 与第六节用户定案 Q1、Q3、Q4）：一个实例的根全部轮出根环之后，抹掉池里按它那一行判未发布的单元头，再发一次专用的清扫发布
+//! 把那一行删掉。
+//!
+//! 次序是先判、后抹（F6）：判根环每个槽都自证过、环里没有该实例的根，判池中每一个落点（[`judge_the_instance_table_sweep`]），
+//! 判过了才抹（FUA 写零头所在的那一个物理块，之后每块盘一道屏障），抹头全部返回之后才发清扫发布
+//! （[`erase_the_heads_and_publish_the_sweep`]）。这次清扫里有一次读失败、设备不全、根环有一槽读不出或全零或重读一次仍自证不过，
+//! 这一轮一行都不删、一个头都不抹。清扫发布的落点被拒时这一轮不删行，挂载照样交回可写（Q4「不单列，被拒就这一轮不删」）。
+//!
+//! 谁跑、什么时候跑（Q1）：可写挂载末尾（写行、暖机、抬 F、删除续做之后，交回之前）判一次——开关关着时实例表行数超过阈值就在这次挂载里
+//! 立刻判、抹、发，没超过不扫；开关开着时交给挂着时的后台清扫（[`sweep_the_instance_table_while_mounted`]），后台抹之前在同一串行段里
+//! 重读那一槽的头，仍是候选行的未发布单元才抹（F1）。挂着时什么时候调度后台那一轮，这一批没做（入口在，调度器没有）。
+//!
+//! 池中每一个落点怎么读（Q3「两条都要」、S2-2「按整单元跳」）：每块盘从单元区起点按槽升序读头，先照认领规则认单元（D18 已定项 7：解出一个头
+//! 就按它的整单元跨度把后面的槽认领掉——码 1、码 3 两槽，码 2 一槽），再跳过根环里任一可读根的账里仍分配的槽；认「是单元」只看头校验和，
+//! 不要求载荷校验和过（不比任何读盘判定更严）。
+//!
+//! 第一版不做的：现行那一版树表 0 条时不清扫（[`NoRowDeletedThisRound::SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion`]：
+//! 那一版的账住根记录那一项、不在树表里，「跳过根环里可读根的账」那一半读不全；清扫发布在那一版上的形状条款也没写）。
+use std::collections::{BTreeMap, BTreeSet};
+
+use singlefs_format::{
+    DATA_UNIT_HEADER_BYTES, INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE,
+    INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT, INSTANCE_TABLE_PAGE_RECORDS, PACKED_UNIT_HEADER_BYTES,
+    SLOT_BYTES,
+};
+
+use crate::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
+};
+use crate::allocator::{absolute_slot_count_of_device, PoolAllocator};
+use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
+use crate::checksum::wide_checksum_field_holds;
+use crate::instance_table::InstanceRow;
+use crate::journal::back_chain_of;
+use crate::make_filesystem::MakeFilesystemParameters;
+use crate::mount::RollbackTarget;
+use crate::recovery::{
+    allocation_records_under_root_in_the_unit_area_starting_at, choose_system_configuration,
+    every_root_ring_slot, instance_table_chain_of_root,
+    unit_area_start_of_the_chosen_system_configuration, PoolReader, RecoveryFailure,
+};
+use crate::root_record::{RootRecord, UnmountMarker};
+use crate::root_ring::{slot_offset, RootRingSlot};
+use crate::transaction::{
+    prepare_the_version_publish, publish_version, DeletionAndLineageOfThePublish,
+    InstanceTablePlan, InstanceTableRewrite, PoolVersion, PoolWriter, PublishError, PublishPlan,
+    ReleaseChecksumCheck, TransactionOutput,
+};
+use crate::unit::{
+    unit_filesystem_identifier, HEADER_CHECKSUM_OFFSET, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE,
+    UNIT_CLASS_PACKED, UNIT_MAGIC,
+};
+
+/// 清扫在可写挂载末尾怎么走（开关，D18 已定项 11「谁跑、什么时候跑」，用户 2026-10-03 定）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum InstanceTableSweepSwitch {
+    /// 开关关着：实例表行数超过阈值就在这次挂载里立刻判、抹、发清扫发布；没超过不扫。
+    SweptAtTheEndOfTheMountAboveTheThreshold,
+    /// 开关开着：挂载末尾不扫，交给挂着时的后台清扫（[`sweep_the_instance_table_while_mounted`]）。
+    HandedToTheBackgroundSweepWhileMounted,
+}
+
+/// 实例表行数的阈值（配置项）：开关关着时行数超过它才在挂载末尾清扫。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct InstanceTableRowCountThreshold(pub u64);
+
+impl InstanceTableRowCountThreshold {
+    /// 默认值没定（C690（清扫触发阈值的默认值没定））：这里的占位取一片实例表装得下的数据行数 369（370 条记录减去链指针那一条），
+    /// 行数越过它链就长出第二片。不是定案，定了照定案换。
+    pub const PLACEHOLDER_UNTIL_C690: Self = Self(INSTANCE_TABLE_PAGE_RECORDS - 1);
+    /// 有一行就清扫（用例要在少量行上强制进入清扫那一支，`.claude/rules/fs-design.md` 五条硬要求第 2 条）。
+    pub const ANY_ROW: Self = Self(0);
+}
+
+/// 清扫发布从哪个分配器取落点：只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条）——用例要强制进入「清扫发布的落点被拒」那一支，
+/// 而一个刚做成可写挂载的池上要造出「写行与暖机落得下、清扫发布落不下」很难卡准。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum SweepPublishPlacements {
+    /// 产品路径：照这次挂载的分配器取。
+    TakenFromTheAllocatorOfTheMount,
+    /// 只供测试：在分配器的一份拷贝上把每块盘每个空闲槽都占掉再取落点（走真的取落点那一段，落点被拒），一个写都不发。
+    EveryFreeSlotTakenAwayByTheTestOnlySwitch,
+}
+
+/// 清扫的配置：开关、阈值与只供测试的落点开关。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct InstanceTableSweepConfiguration {
+    pub switch: InstanceTableSweepSwitch,
+    pub row_count_threshold: InstanceTableRowCountThreshold,
+    pub sweep_publish_placements: SweepPublishPlacements,
+}
+
+impl InstanceTableSweepConfiguration {
+    /// 产品路径（`mount::mount_writable`）：开关关着、阈值取占位（[`InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690`]）。
+    pub const PRODUCT: Self = Self {
+        switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
+        row_count_threshold: InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690,
+        sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
+    };
+}
+
+/// 一个单元头里清扫要的几样（只看头校验和认出来的）：类、写序实例、诞生代号、码 1 / 码 3 写序里的事务号、fsid 低 8 字节。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct UnitHeadReadBySweep {
+    pub unit_class: u8,
+    pub writer_instance: InstanceGeneration,
+    pub birth_txg: CheckpointTxg,
+    /// 码 1、码 3 写序里的事务号；码 2 的写序只有实例代号（D18 已定项 12），这里是 0、已发布谓词不看它。
+    pub transaction: u64,
+    pub filesystem_identifier: u64,
+}
+
+impl UnitHeadReadBySweep {
+    /// 认领规则按整单元跳的跨度（S2-2「按整单元跳」）：码 1、码 3 两槽，码 2 一槽。
+    #[must_use]
+    pub fn span_in_slots(&self) -> u64 {
+        if self.unit_class == UNIT_CLASS_INDEX_NODE {
+            NODE_SPAN_IN_SLOTS
+        } else {
+            DATA_OR_PACKED_UNIT_SPAN_IN_SLOTS
+        }
+    }
+
+    /// 已发布谓词里有行的那一支（D18 已定项 11「已发布谓词」）：有行 (i, T, W) 时，码 1 ⇒ b ≤ T ∨ n ≤ W 已发布，码 2 / 3 ⇒ b ≤ T 已发布。
+    #[must_use]
+    pub fn is_unpublished_by_the_row(&self, row: &InstanceRow) -> bool {
+        if self.unit_class == UNIT_CLASS_DATA {
+            !(self.birth_txg <= row.selected_root_txg
+                || self.transaction <= row.applied_transaction_high_water)
+        } else {
+            self.birth_txg > row.selected_root_txg
+        }
+    }
+}
+
+const NODE_SPAN_IN_SLOTS: u64 = 1;
+const DATA_OR_PACKED_UNIT_SPAN_IN_SLOTS: u64 = 2;
+/// 码 2 头里 key 宽那一个字节的偏移（D18 已定项 18 的偏移表）。
+const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
+/// 码 1 头里诞生代号、fsid、写序实例、事务号的偏移（D18 已定项 7 / 已定项 18 的偏移表；`unit::parse_data_unit` 读的同一处）。
+const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;
+const DATA_UNIT_FILESYSTEM_IDENTIFIER_OFFSET: usize = 83;
+const DATA_UNIT_WRITER_INSTANCE_OFFSET: usize = 91;
+const DATA_UNIT_TRANSACTION_OFFSET: usize = 95;
+/// 码 3 头里同几样的偏移（`unit::parse_packed_unit` 读的同一处）。
+const PACKED_UNIT_BIRTH_TXG_OFFSET: usize = 73;
+const PACKED_UNIT_FILESYSTEM_IDENTIFIER_OFFSET: usize = 81;
+const PACKED_UNIT_WRITER_INSTANCE_OFFSET: usize = 93;
+const PACKED_UNIT_TRANSACTION_OFFSET: usize = 97;
+/// 码 2 头里诞生代号、fsid、写序实例在 key 区间之后的偏移：52 + 2k 起依次 8、8、4 字节（`unit::parse_index_node` 读的同一处）。
+const INDEX_NODE_BIRTH_TXG_OFFSET_AFTER_THE_KEY_RANGE: usize = 52;
+const INDEX_NODE_FILESYSTEM_IDENTIFIER_OFFSET_AFTER_THE_KEY_RANGE: usize = 60;
+const INDEX_NODE_WRITER_INSTANCE_OFFSET_AFTER_THE_KEY_RANGE: usize = 68;
+/// 事务号在写序里占 6 字节（D18 已定项 7）。
+const TRANSACTION_BYTES: usize = 6;
+/// 一个槽读多少字节来认头：码 2 的明文头最宽 86 + 2 × 255 = 596 字节，读 1024 字节罩得住任何一类的头；不足一个物理块时读一个物理块。
+const HEAD_BYTES_READ_PER_SLOT: u64 = 1024;
+
+fn u64_at(bytes: &[u8], offset: usize) -> u64 {
+    u64::from_le_bytes(
+        bytes[offset..offset + 8]
+            .try_into()
+            .expect("切出来的是 8 字节"),
+    )
+}
+
+fn u32_at(bytes: &[u8], offset: usize) -> u32 {
+    u32::from_le_bytes(
+        bytes[offset..offset + 4]
+            .try_into()
+            .expect("切出来的是 4 字节"),
+    )
+}
+
+fn six_byte_unsigned_at(bytes: &[u8], offset: usize) -> u64 {
+    let mut widened = [0u8; 8];
+    widened[..TRANSACTION_BYTES].copy_from_slice(&bytes[offset..offset + TRANSACTION_BYTES]);
+    u64::from_le_bytes(widened)
+}
+
+/// 一个槽开头的字节认不认得出一个单元头：魔数、类（1 / 2 / 3，定头的宽）与头校验和过就认，别的（版本、flags、载荷校验和）不看
+/// ——清扫认「是单元」不比任何读盘判定更严（判决第二节 Q2 攻方补的那一句收严）。认不出交 `None`（已抹头、没写过、别的字节）。
+#[must_use]
+pub fn unit_head_read_by_sweep(head_bytes: &[u8]) -> Option<UnitHeadReadBySweep> {
+    if head_bytes.len() <= INDEX_NODE_KEY_WIDTH_OFFSET || head_bytes[..4] != UNIT_MAGIC {
+        return None;
+    }
+    let unit_class = head_bytes[6];
+    let header_end = match unit_class {
+        UNIT_CLASS_DATA => usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105"),
+        UNIT_CLASS_PACKED => usize::try_from(PACKED_UNIT_HEADER_BYTES).expect("107"),
+        UNIT_CLASS_INDEX_NODE => {
+            usize::try_from(INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE).expect("86")
+                + usize::try_from(INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT).expect("2")
+                    * usize::from(head_bytes[INDEX_NODE_KEY_WIDTH_OFFSET])
+        }
+        _ => return None,
+    };
+    if head_bytes.len() < header_end
+        || !wide_checksum_field_holds(head_bytes, header_end, HEADER_CHECKSUM_OFFSET)
+    {
+        return None;
+    }
+    Some(match unit_class {
+        UNIT_CLASS_DATA => UnitHeadReadBySweep {
+            unit_class,
+            birth_txg: CheckpointTxg(u64_at(head_bytes, DATA_UNIT_BIRTH_TXG_OFFSET)),
+            filesystem_identifier: u64_at(head_bytes, DATA_UNIT_FILESYSTEM_IDENTIFIER_OFFSET),
+            writer_instance: InstanceGeneration(u32_at(
+                head_bytes,
+                DATA_UNIT_WRITER_INSTANCE_OFFSET,
+            )),
+            transaction: six_byte_unsigned_at(head_bytes, DATA_UNIT_TRANSACTION_OFFSET),
+        },
+        UNIT_CLASS_PACKED => UnitHeadReadBySweep {
+            unit_class,
+            birth_txg: CheckpointTxg(u64_at(head_bytes, PACKED_UNIT_BIRTH_TXG_OFFSET)),
+            filesystem_identifier: u64_at(head_bytes, PACKED_UNIT_FILESYSTEM_IDENTIFIER_OFFSET),
+            writer_instance: InstanceGeneration(u32_at(
+                head_bytes,
+                PACKED_UNIT_WRITER_INSTANCE_OFFSET,
+            )),
+            transaction: six_byte_unsigned_at(head_bytes, PACKED_UNIT_TRANSACTION_OFFSET),
+        },
+        _ => {
+            let key_range_bytes = usize::try_from(INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT)
+                .expect("2")
+                * usize::from(head_bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
+            UnitHeadReadBySweep {
+                unit_class,
+                birth_txg: CheckpointTxg(u64_at(
+                    head_bytes,
+                    INDEX_NODE_BIRTH_TXG_OFFSET_AFTER_THE_KEY_RANGE + key_range_bytes,
+                )),
+                filesystem_identifier: u64_at(
+                    head_bytes,
+                    INDEX_NODE_FILESYSTEM_IDENTIFIER_OFFSET_AFTER_THE_KEY_RANGE + key_range_bytes,
+                ),
+                writer_instance: InstanceGeneration(u32_at(
+                    head_bytes,
+                    INDEX_NODE_WRITER_INSTANCE_OFFSET_AFTER_THE_KEY_RANGE + key_range_bytes,
+                )),
+                transaction: 0,
+            }
+        }
+    })
+}
+
+/// 一个落点：哪块盘、哪个槽。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub struct PlacementOfAUnitHead {
+    pub device: DeviceIdentity,
+    pub slot: SlotNumber,
+}
+
+/// 要抹的一个头：落点与判的那一刻读出来的头（后台那一支抹之前重读、与它比）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct UnitHeadToErase {
+    pub placement: PlacementOfAUnitHead,
+    pub head: UnitHeadReadBySweep,
+}
+
+/// 这一轮为什么一行都不删（判的那一半，一个头都没抹）。
+#[derive(Debug)]
+pub enum NoRowDeletedThisRound {
+    /// 实例表里每一行的实例在根环里都还有根（或表里没有行）：没有候选行。
+    NoCandidateRow,
+    /// 交进来的盘少于池的盘数（设备维：全部设备在线才删）。
+    NotEveryDeviceOfThePoolIsPresent {
+        devices_present: usize,
+        devices_of_the_pool: u32,
+    },
+    /// 根环一个槽读不出、或读成功而全零：按「可能有该实例的根」算，这一次不删。
+    ARootRingSlotIsUnreadableOrAllZero { ring_slot: RootRingSlot },
+    /// 根环一个槽读得出而自证不过，重读一次仍不过。
+    ARootRingSlotIsNotSelfVerifiedAfterOneReread { ring_slot: RootRingSlot },
+    /// 择不到系统配置：判不了池有几块盘、根环在哪。
+    SystemConfigurationNotChosen { failure: RecoveryFailure },
+    /// 现行那一版的实例表读不出：判不了哪几行是候选。
+    InstanceTableOfTheCurrentVersionUnreadable { failure: RecoveryFailure },
+    /// 根环里一条可读根的账读不出：「跳过它的账里仍分配的槽」那一半判不了。
+    AccountOfARingRootUnreadable {
+        root: RollbackTarget,
+        failure: RecoveryFailure,
+    },
+    /// 读池中一个落点的头读失败（扇区维：读不到不等于不存在）。
+    AReadOfAPlacementFailed { placement: PlacementOfAUnitHead },
+    /// 池里有一个本池的单元，写序实例大于挂载根的实例（已发布谓词判损坏）：这一轮任何行都不删（判决第二节 Q3 正推给的保守读法）。
+    AUnitWrittenByAnInstanceAboveTheMountedOne {
+        placement: PlacementOfAUnitHead,
+        writer_instance: InstanceGeneration,
+    },
+    /// 现行那一版树表 0 条：第一版不清扫（模块文档「第一版不做的」）。在任何写之前判出，一个头都不抹。
+    SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion,
+}
+
+/// 判的那一半交回的：要删的行、要抹的头，与判的那一刻池里认出来的单元个数（交给用例看扫没扫全）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct InstanceTableSweepJudgement {
+    pub rows_to_delete: Vec<InstanceRow>,
+    pub heads_to_erase: Vec<UnitHeadToErase>,
+    pub units_recognized_in_the_pool: u64,
+}
+
+/// 判一轮清扫（只读，不写任何一个字节）：根环每个槽都自证过（读不出、全零的这一次不删；自证不过的重读一次），实例表里实例在环里一条根都没有的行
+/// 是候选行；池中每一个落点照认领规则认单元、跳过根环里任一可读根的账里仍分配的槽，按候选行判未发布的记下要抹。
+///
+/// # Errors
+/// [`NoRowDeletedThisRound`] 的各个成员（`NoCandidateRow` 也走这里：没有要删的行就不扫池）。
+pub fn judge_the_instance_table_sweep(
+    reader: &dyn PoolReader,
+    current_root: &RootRecord,
+) -> Result<InstanceTableSweepJudgement, NoRowDeletedThisRound> {
+    let system_configuration = choose_system_configuration(reader)
+        .map_err(|failure| NoRowDeletedThisRound::SystemConfigurationNotChosen { failure })?;
+    let immutable = &system_configuration.immutable;
+    let devices_present = reader.device_identities();
+    if u32::try_from(devices_present.len()).expect("盘数装得进 u32") < immutable.device_count {
+        return Err(NoRowDeletedThisRound::NotEveryDeviceOfThePoolIsPresent {
+            devices_present: devices_present.len(),
+            devices_of_the_pool: immutable.device_count,
+        });
+    }
+    let roots_of_the_ring = every_root_of_the_ring_self_verified(
+        reader,
+        &immutable.region_devices,
+        &immutable.sizes,
+        &immutable.filesystem_identifier,
+    )?;
+    let rows = instance_table_chain_of_root(reader, current_root)
+        .map_err(
+            |failure| NoRowDeletedThisRound::InstanceTableOfTheCurrentVersionUnreadable { failure },
+        )?
+        .records
+        .rows;
+    let instances_with_a_root_in_the_ring: BTreeSet<InstanceGeneration> =
+        roots_of_the_ring.iter().map(|root| root.instance).collect();
+    let rows_to_delete: Vec<InstanceRow> = rows
+        .iter()
+        .copied()
+        .filter(|row| !instances_with_a_root_in_the_ring.contains(&row.instance))
+        .collect();
+    if rows_to_delete.is_empty() {
+        return Err(NoRowDeletedThisRound::NoCandidateRow);
+    }
+    let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
+    let mut slots_allocated_in_an_account_of_the_ring: BTreeSet<PlacementOfAUnitHead> =
+        BTreeSet::new();
+    for root in &roots_of_the_ring {
+        let records = allocation_records_under_root_in_the_unit_area_starting_at(
+            reader,
+            unit_area_start,
+            root,
+        )
+        .map_err(
+            |failure| NoRowDeletedThisRound::AccountOfARingRootUnreadable {
+                root: RollbackTarget {
+                    instance: root.instance,
+                    checkpoint_txg: root.checkpoint_txg,
+                },
+                failure,
+            },
+        )?;
+        for record in records.iter().filter(|record| !record.is_released) {
+            for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
+                slots_allocated_in_an_account_of_the_ring.insert(PlacementOfAUnitHead {
+                    device: record.device,
+                    slot: SlotNumber(slot),
+                });
+            }
+        }
+    }
+    let filesystem_identifier = unit_filesystem_identifier(&immutable.filesystem_identifier);
+    let head_bytes = usize::try_from(
+        HEAD_BYTES_READ_PER_SLOT.max(u64::from(immutable.sizes.physical_block_size)),
+    )
+    .expect("读头的字节数装得进 usize");
+    let rows_by_instance: BTreeMap<InstanceGeneration, InstanceRow> = rows_to_delete
+        .iter()
+        .map(|row| (row.instance, *row))
+        .collect();
+    let mut heads_to_erase = Vec::new();
+    let mut units_recognized_in_the_pool = 0u64;
+    for device in devices_present {
+        let slots_of_the_device = reader
+            .device_size_in_bytes(device)
+            .map_or(0, absolute_slot_count_of_device);
+        let mut slot = unit_area_start.slot().0;
+        // 迭代上界是这块盘的槽数；每轮至少前进一槽（认出单元就跳过它的整单元跨度）；跨轮携带的只有游标与记下的头；
+        // 提前出口是读失败与写序实例越过挂载根的单元两条。
+        while slot < slots_of_the_device {
+            let placement = PlacementOfAUnitHead {
+                device,
+                slot: SlotNumber(slot),
+            };
+            let Some(bytes) = reader.read(device, SlotNumber(slot).to_device_offset(), head_bytes)
+            else {
+                return Err(NoRowDeletedThisRound::AReadOfAPlacementFailed { placement });
+            };
+            let Some(head) = unit_head_read_by_sweep(&bytes) else {
+                slot += 1;
+                continue;
+            };
+            units_recognized_in_the_pool += 1;
+            slot += head.span_in_slots();
+            if head.filesystem_identifier != filesystem_identifier {
+                continue;
+            }
+            if head.writer_instance > current_root.instance {
+                return Err(
+                    NoRowDeletedThisRound::AUnitWrittenByAnInstanceAboveTheMountedOne {
+                        placement,
+                        writer_instance: head.writer_instance,
+                    },
+                );
+            }
+            if slots_allocated_in_an_account_of_the_ring.contains(&placement) {
+                continue;
+            }
+            if rows_by_instance
+                .get(&head.writer_instance)
+                .is_some_and(|row| head.is_unpublished_by_the_row(row))
+            {
+                heads_to_erase.push(UnitHeadToErase { placement, head });
+            }
+        }
+    }
+    Ok(InstanceTableSweepJudgement {
+        rows_to_delete,
+        heads_to_erase,
+        units_recognized_in_the_pool,
+    })
+}
+
+/// 读根环每个槽：读不出、读成功而全零 ⇒ 这一次不删；读得出而自证不过 ⇒ 重读一次，仍不过 ⇒ 这一次不删；都自证过就交回每一条根。
+fn every_root_of_the_ring_self_verified(
+    reader: &dyn PoolReader,
+    region_devices: &[DeviceIdentity; 3],
+    immutable_sizes: &crate::system_configuration::SystemImmutableSizes,
+    filesystem_identifier: &[u8; 16],
+) -> Result<Vec<RootRecord>, NoRowDeletedThisRound> {
+    let root_slot_bytes = usize::try_from(immutable_sizes.physical_block_size).expect("根槽宽");
+    let mut roots = Vec::new();
+    // 迭代上界是根环的槽数（3 × S），每槽至多读两次；提前出口是这一次不删的那几种。
+    for ring_slot in every_root_ring_slot(immutable_sizes) {
+        let device = region_devices[usize::try_from(ring_slot.region).expect("区域号")];
+        let offset = slot_offset(ring_slot, immutable_sizes.fixed_structure_slot_spacing);
+        let mut root_read = None;
+        for _reading in 0..2 {
+            let Some(bytes) = reader.read(device, offset, root_slot_bytes) else {
+                return Err(NoRowDeletedThisRound::ARootRingSlotIsUnreadableOrAllZero {
+                    ring_slot,
+                });
+            };
+            if bytes.iter().all(|byte| *byte == 0) {
+                return Err(NoRowDeletedThisRound::ARootRingSlotIsUnreadableOrAllZero {
+                    ring_slot,
+                });
+            }
+            root_read = RootRecord::parse_slot(&bytes, filesystem_identifier);
+            if root_read.is_some() {
+                break;
+            }
+        }
+        let Some(root) = root_read else {
+            return Err(
+                NoRowDeletedThisRound::ARootRingSlotIsNotSelfVerifiedAfterOneReread { ring_slot },
+            );
+        };
+        roots.push(root);
+    }
+    Ok(roots)
+}
+
+/// 抹头那一步与清扫发布走到哪儿。
+#[derive(Debug)]
+pub enum InstanceTableSweepOutcome {
+    /// 判的那一半就停了，一个头都没抹。
+    NoRowDeleted(NoRowDeletedThisRound),
+    /// 抹完头、清扫发布落盘：删了这几行。`heads_erased` 是真抹了的（后台那一支重读之后已经不是候选行未发布单元的不抹，不在里面）。
+    RowsDeleted {
+        rows_deleted: Vec<InstanceRow>,
+        heads_erased: Vec<PlacementOfAUnitHead>,
+    },
+    /// 抹完头，清扫发布的落点被拒（`PlacementRefused` / `SpaceAdmissionRefused`）：这一轮不删行，挂载照样交回可写（Q4）。一个发布的写都没发。
+    SweepPublishRefusedForSpace {
+        heads_erased: Vec<PlacementOfAUnitHead>,
+        refusal: Box<PublishError>,
+    },
+    /// 抹头的一个写或屏障报块设备错：这一轮不删行（不发清扫发布），已抹的留着——它们按那一行判本来就未发布。
+    /// 条款只写了落点被拒那一格，块设备错怎么处置没写（主 agent 交回报告里列着）；这里照落点被拒那一格不删、交回可写。
+    StoppedByABlockDeviceErrorWhileErasing {
+        heads_erased: Vec<PlacementOfAUnitHead>,
+        error: BlockDeviceError,
+    },
+    /// 清扫发布落盘途中报错（或在落点之外的理由上被拒）：这一轮不删行；落盘途中报的那一次照发布路径冻结在分配器上等原样重发
+    /// （`transaction::publish_version` 的契约）。条款同样没写这一格，处置同上一个成员。
+    SweepPublishFailed {
+        heads_erased: Vec<PlacementOfAUnitHead>,
+        cause: Box<PublishError>,
+    },
+}
+
+/// 抹头、过屏障、发清扫发布（判之后那一半）。每个要抹的头抹之前在同一串行段里重读那一槽的头：仍是判的那一刻读出来的那个单元
+/// （类、写序实例、诞生代号、事务号都相同）才抹——判完到抹之间有分配写掉了那一槽的，不抹（F1；挂载末尾那一支两步之间没有别的写，重读只多读一次）。
+/// 清扫发布接在 `current` 后面：同一实例、事务号 0、不进 W、实例表整链重写成「这一刻现行那一版的表去掉要删的行」、不判挂载准入式子
+/// （一个普通分配都没有的发布不判，`transaction::prepare_the_version_publish`）。发成了 `current` 换成清扫发布那一版。
+pub fn erase_the_heads_and_publish_the_sweep<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    allocator: &mut PoolAllocator,
+    current: &mut PoolVersion,
+    judgement: &InstanceTableSweepJudgement,
+    sweep_publish_placements: SweepPublishPlacements,
+) -> InstanceTableSweepOutcome {
+    let current_file_version = match current {
+        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
+            version.clone()
+        }
+        PoolVersion::WithoutFile(_) => {
+            return InstanceTableSweepOutcome::NoRowDeleted(
+                NoRowDeletedThisRound::SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion,
+            )
+        }
+    };
+    let physical_block_bytes = usize::try_from(parameters.geometry.physical_block_size)
+        .expect("物理块的字节数装得进 usize");
+    let head_bytes = usize::try_from(
+        HEAD_BYTES_READ_PER_SLOT.max(u64::from(parameters.geometry.physical_block_size)),
+    )
+    .expect("读头的字节数装得进 usize");
+    let rows_by_instance: BTreeMap<InstanceGeneration, InstanceRow> = judgement
+        .rows_to_delete
+        .iter()
+        .map(|row| (row.instance, *row))
+        .collect();
+    let mut heads_erased = Vec::new();
+    // 迭代上界是要抹的头数；跨轮携带的只有抹了的那几个；提前出口是块设备错。
+    for to_erase in &judgement.heads_to_erase {
+        let offset = to_erase.placement.slot.to_device_offset();
+        let reread_head =
+            PoolReader::read(&*devices, to_erase.placement.device, offset, head_bytes)
+                .as_deref()
+                .and_then(unit_head_read_by_sweep);
+        let still_the_unpublished_unit_judged = reread_head.is_some_and(|head| {
+            head == to_erase.head
+                && rows_by_instance
+                    .get(&head.writer_instance)
+                    .is_some_and(|row| head.is_unpublished_by_the_row(row))
+        });
+        if !still_the_unpublished_unit_judged {
+            continue;
+        }
+        let Some((_, device)) = devices
+            .iter_mut()
+            .find(|(identity, _)| *identity == to_erase.placement.device)
+        else {
+            continue;
+        };
+        if let Err(error) = device.write_at(
+            offset,
+            &vec![0u8; physical_block_bytes],
+            WriteDurability::ForceUnitAccess,
+        ) {
+            return InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing {
+                heads_erased,
+                error,
+            };
+        }
+        heads_erased.push(to_erase.placement);
+    }
+    // 抹头全部 FUA 返回之后每块盘一道屏障，之后才发清扫发布（D18 已定项 11「删行落在一次专用的清扫发布上」）。
+    for (_, device) in devices.iter_mut() {
+        if let Err(error) = device.barrier() {
+            return InstanceTableSweepOutcome::StoppedByABlockDeviceErrorWhileErasing {
+                heads_erased,
+                error,
+            };
+        }
+    }
+    let chain = match instance_table_chain_of_root(&*devices, &current_file_version.root) {
+        Ok(chain) => chain,
+        Err(failure) => {
+            return InstanceTableSweepOutcome::NoRowDeleted(
+                NoRowDeletedThisRound::InstanceTableOfTheCurrentVersionUnreadable { failure },
+            )
+        }
+    };
+    let instances_to_delete: BTreeSet<InstanceGeneration> =
+        rows_by_instance.keys().copied().collect();
+    let rows_kept: Vec<InstanceRow> = chain
+        .records
+        .rows
+        .iter()
+        .copied()
+        .filter(|row| !instances_to_delete.contains(&row.instance))
+        .collect();
+    let plan = sweep_publish_plan(
+        &current_file_version,
+        InstanceTableRewrite {
+            rows: rows_kept,
+            replaced_chain: chain.page_pointers,
+        },
+    );
+    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
+    let published = match sweep_publish_placements {
+        SweepPublishPlacements::TakenFromTheAllocatorOfTheMount => publish_version(
+            &mut pool,
+            allocator,
+            plan.as_publish_plan(),
+            Some(&current_file_version),
+        ),
+        SweepPublishPlacements::EveryFreeSlotTakenAwayByTheTestOnlySwitch => {
+            let mut allocator_without_a_free_slot = allocator.clone();
+            take_every_free_slot_away(&mut allocator_without_a_free_slot);
+            match prepare_the_version_publish(
+                &pool,
+                &mut allocator_without_a_free_slot,
+                &plan.as_publish_plan(),
+                Some(&current_file_version),
+                current_file_version.tree_identifiers,
+                ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt,
+                UnmountMarker::NotWrittenByTheUnmountSequence,
+                &DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,
+            ) {
+                Err(refusal) => Err(refusal),
+                Ok(_) => panic!(
+                    "只供测试的开关把每块盘每个空闲槽都占掉了，清扫发布要的实例表片取不到落点：拷贝上取到了说明这个开关没把那一支强制进去"
+                ),
+            }
+        }
+    };
+    match published {
+        Ok(sweep_publish) => {
+            *current = PoolVersion::of_a_version_of_trees(sweep_publish);
+            InstanceTableSweepOutcome::RowsDeleted {
+                rows_deleted: judgement.rows_to_delete.clone(),
+                heads_erased,
+            }
+        }
+        Err(
+            refusal @ (PublishError::PlacementRefused { .. }
+            | PublishError::SpaceAdmissionRefused(_)),
+        ) => InstanceTableSweepOutcome::SweepPublishRefusedForSpace {
+            heads_erased,
+            refusal: Box::new(refusal),
+        },
+        Err(cause) => InstanceTableSweepOutcome::SweepPublishFailed {
+            heads_erased,
+            cause: Box::new(cause),
+        },
+    }
+}
+
+/// 清扫发布的计划（它的那几样都从现行那一版来，实例表整链重写）。
+struct SweepPublishPlan {
+    txg: CheckpointTxg,
+    counter: u64,
+    highest_transaction_number_before_this_publish: u64,
+    instance: InstanceGeneration,
+    back_chain: u32,
+    instance_table: InstanceTableRewrite,
+    tree_birth_txg: CheckpointTxg,
+    tree_identifier_watermark: u64,
+    rollback_floor: CheckpointTxg,
+}
+
+impl SweepPublishPlan {
+    fn as_publish_plan(&self) -> PublishPlan<'_> {
+        PublishPlan {
+            txg: self.txg,
+            counter: self.counter,
+            // 事务号 0、不进 W（D18 已定项 11「清扫发布的形状」）。
+            transaction: 0,
+            highest_transaction_number_before_this_publish: self
+                .highest_transaction_number_before_this_publish,
+            instance: self.instance,
+            back_chain: self.back_chain,
+            file: None,
+            new_inode_records: &[],
+            instance_table: InstanceTablePlan::Rewrite(self.instance_table.clone()),
+            tree_birth_txg: self.tree_birth_txg,
+            tree_identifier_watermark: self.tree_identifier_watermark,
+            rollback_floor: self.rollback_floor,
+        }
+    }
+}
+
+fn sweep_publish_plan(
+    previous: &TransactionOutput,
+    instance_table: InstanceTableRewrite,
+) -> SweepPublishPlan {
+    SweepPublishPlan {
+        txg: CheckpointTxg(
+            previous
+                .root
+                .checkpoint_txg
+                .0
+                .checked_add(1)
+                .expect("现行那一版的 txg 不在 u64 顶上：它是这个进程刚发成的"),
+        ),
+        counter: previous.record.counter + 1,
+        highest_transaction_number_before_this_publish: previous
+            .highest_transaction_number_in_this_instance,
+        instance: previous.root.instance,
+        back_chain: back_chain_of(&previous.record_bytes),
+        instance_table,
+        tree_birth_txg: previous.tree_birth_txg(),
+        tree_identifier_watermark: previous.root.tree_identifier_watermark,
+        rollback_floor: previous.root.rollback_floor,
+    }
+}
+
+/// 只供测试的开关那一支：把分配器（一份拷贝）上每块盘每个空闲槽都占掉。
+fn take_every_free_slot_away(allocator: &mut PoolAllocator) {
+    for device_map in &mut allocator.devices {
+        let unit_area_start = device_map.unit_area_start().slot().0;
+        // 迭代上界是这块盘单元区的槽数；没有提前出口。
+        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
+            if device_map.is_free(SlotNumber(slot)) {
+                device_map.mark_allocated(SlotNumber(slot), 1);
+            }
+        }
+    }
+}
+
+/// 可写挂载末尾那一判（`mount::establish_instance` 交回之前调）的结局。
+#[derive(Debug)]
+pub enum InstanceTableSweepAtTheEndOfTheMount {
+    /// 开关关着、行数没超过阈值：不扫。
+    NotSweptAtOrBelowTheThreshold {
+        rows: u64,
+        threshold: InstanceTableRowCountThreshold,
+    },
+    /// 开关开着：交给挂着时的后台清扫。
+    HandedToTheBackgroundSweepWhileMounted,
+    /// 开关关着、行数超过阈值：这次挂载里判、抹、发的结局。
+    Swept(InstanceTableSweepOutcome),
+}
+
+/// 可写挂载末尾那一判：开关关着、行数超过阈值就立刻判、抹、发；否则不扫。`current` 是续做之后的现行那一版，清扫发布发成了换成它。
+pub fn sweep_the_instance_table_at_the_end_of_the_mount<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    allocator: &mut PoolAllocator,
+    current: &mut PoolVersion,
+    configuration: InstanceTableSweepConfiguration,
+) -> InstanceTableSweepAtTheEndOfTheMount {
+    match configuration.switch {
+        InstanceTableSweepSwitch::HandedToTheBackgroundSweepWhileMounted => {
+            return InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted
+        }
+        InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold => {}
+    }
+    let rows = match instance_table_chain_of_root(&*devices, current.root()) {
+        Ok(chain) => u64::try_from(chain.records.rows.len()).expect("行数装得进 u64"),
+        Err(failure) => {
+            return InstanceTableSweepAtTheEndOfTheMount::Swept(
+                InstanceTableSweepOutcome::NoRowDeleted(
+                    NoRowDeletedThisRound::InstanceTableOfTheCurrentVersionUnreadable { failure },
+                ),
+            )
+        }
+    };
+    if rows <= configuration.row_count_threshold.0 {
+        return InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold {
+            rows,
+            threshold: configuration.row_count_threshold,
+        };
+    }
+    InstanceTableSweepAtTheEndOfTheMount::Swept(sweep_the_instance_table_while_mounted(
+        parameters,
+        devices,
+        allocator,
+        current,
+        configuration.sweep_publish_placements,
+    ))
+}
+
+/// 一整轮清扫（后台那一支的入口，挂载末尾那一支也走它）：判、抹（抹之前重读）、发。`current` 是现行那一版，清扫发布发成了换成它。
+/// 现行那一版树表 0 条时在判之前就交回不支持（一个字节都不写）。
+pub fn sweep_the_instance_table_while_mounted<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    allocator: &mut PoolAllocator,
+    current: &mut PoolVersion,
+    sweep_publish_placements: SweepPublishPlacements,
+) -> InstanceTableSweepOutcome {
+    match current {
+        PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => {}
+        PoolVersion::WithoutFile(_) => {
+            return InstanceTableSweepOutcome::NoRowDeleted(
+                NoRowDeletedThisRound::SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion,
+            )
+        }
+    }
+    let judgement = match judge_the_instance_table_sweep(&*devices, current.root()) {
+        Ok(judgement) => judgement,
+        Err(no_row_deleted) => return InstanceTableSweepOutcome::NoRowDeleted(no_row_deleted),
+    };
+    erase_the_heads_and_publish_the_sweep(
+        parameters,
+        devices,
+        allocator,
+        current,
+        &judgement,
+        sweep_publish_placements,
+    )
+}
+
+/// 一个落点的设备内偏移（给用例读头用）。
+#[must_use]
+pub fn device_offset_of_the_head(placement: PlacementOfAUnitHead) -> DeviceOffsetInBytes {
+    DeviceOffsetInBytes(placement.slot.0 * SLOT_BYTES)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn data_unit_head(instance: u32, birth_txg: u64, transaction: u64) -> UnitHeadReadBySweep {
+        UnitHeadReadBySweep {
+            unit_class: UNIT_CLASS_DATA,
+            writer_instance: InstanceGeneration(instance),
+            birth_txg: CheckpointTxg(birth_txg),
+            transaction,
+            filesystem_identifier: 0,
+        }
+    }
+
+    fn row(
+        instance: u32,
+        selected_root_txg: u64,
+        applied_transaction_high_water: u64,
+    ) -> InstanceRow {
+        InstanceRow {
+            instance: InstanceGeneration(instance),
+            selected_root_txg: CheckpointTxg(selected_root_txg),
+            applied_transaction_high_water,
+        }
+    }
+
+    /// 已发布谓词有行那一支：码 1 看诞生代号与事务号两样，码 2 / 3 只看诞生代号（D18 已定项 11）。
+    #[test]
+    fn the_published_predicate_of_a_row_reads_birth_and_transaction_for_data_units_and_birth_only_otherwise(
+    ) {
+        let the_row = row(1, 10, 7);
+        assert!(
+            !data_unit_head(1, 10, 9).is_unpublished_by_the_row(&the_row),
+            "诞生代号 ≤ T：已发布"
+        );
+        assert!(
+            !data_unit_head(1, 11, 7).is_unpublished_by_the_row(&the_row),
+            "事务号 ≤ W：已发布"
+        );
+        assert!(
+            data_unit_head(1, 11, 8).is_unpublished_by_the_row(&the_row),
+            "两样都越过：未发布"
+        );
+        let node = UnitHeadReadBySweep {
+            unit_class: UNIT_CLASS_INDEX_NODE,
+            ..data_unit_head(1, 11, 0)
+        };
+        assert!(
+            node.is_unpublished_by_the_row(&the_row),
+            "码 2 只看诞生代号"
+        );
+    }
+
+    /// 认领规则按整单元跳：码 1、码 3 两槽，码 2 一槽（S2-2）。
+    #[test]
+    fn a_claimed_unit_spans_two_slots_for_data_and_packed_units_and_one_slot_for_nodes() {
+        assert_eq!(data_unit_head(1, 1, 1).span_in_slots(), 2);
+        assert_eq!(
+            UnitHeadReadBySweep {
+                unit_class: UNIT_CLASS_PACKED,
+                ..data_unit_head(1, 1, 1)
+            }
+            .span_in_slots(),
+            2
+        );
+        assert_eq!(
+            UnitHeadReadBySweep {
+                unit_class: UNIT_CLASS_INDEX_NODE,
+                ..data_unit_head(1, 1, 1)
+            }
+            .span_in_slots(),
+            1
+        );
+    }
+
+    /// 全零的槽、魔数不对的槽不是单元（已抹头那一形）。
+    #[test]
+    fn an_all_zero_head_and_a_head_without_the_magic_are_not_units() {
+        assert_eq!(unit_head_read_by_sweep(&[0u8; 1024]), None);
+        let mut without_magic = [0u8; 1024];
+        without_magic[..4].copy_from_slice(b"SFSX");
+        without_magic[6] = UNIT_CLASS_DATA;
+        assert_eq!(unit_head_read_by_sweep(&without_magic), None);
+    }
+}
diff --git a/crates/singlefs-core/src/lib.rs b/crates/singlefs-core/src/lib.rs
index b2ec900..d44a17e 100644
--- a/crates/singlefs-core/src/lib.rs
+++ b/crates/singlefs-core/src/lib.rs
@@ -22,6 +22,7 @@ pub mod deleted_inodes;
 pub mod extent_tree;
 pub mod inode_tree;
 pub mod instance_table;
+pub mod instance_table_sweep;
 pub mod journal;
 pub mod lineage_tree;
 pub mod make_filesystem;
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index f10150c..7bae1a9 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -23,6 +23,10 @@ use crate::allocator::{
 };
 use crate::block_device::BlockDevice;
 use crate::checksum::crc32_castagnoli;
+use crate::instance_table_sweep::{
+    sweep_the_instance_table_at_the_end_of_the_mount, InstanceTableSweepAtTheEndOfTheMount,
+    InstanceTableSweepConfiguration,
+};
 use crate::journal::back_chain_of;
 use crate::make_filesystem::MakeFilesystemParameters;
 use crate::mounted_session::refusal_is_short_of_space;
@@ -929,6 +933,10 @@ pub struct MountOutput {
     pub refusals_that_pushed_the_continuation_floor_raises: Vec<PublishError>,
     /// 挂载续做停在哪儿（[`DeletionQueueContinuationOutcome`]）：走到了空，或推满仍放不下、队列留着而挂载照样做成可写。
     pub deletion_queue_continuation_outcome: DeletionQueueContinuationOutcome,
+    /// 交回之前那一判实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条「谁跑、什么时候跑」）的结局：不扫、交给后台，
+    /// 或这次挂载里判、抹、发到了哪儿（[`crate::instance_table_sweep::sweep_the_instance_table_at_the_end_of_the_mount`]）。
+    /// 发成了清扫发布时 `Mounted::current` 是清扫发布那一版。
+    pub instance_table_sweep: InstanceTableSweepAtTheEndOfTheMount,
 }
 
 /// 挂载续做停在哪儿。成员按调用方要做的决定分：照常写；队列还留着——之后会话里每一次用户改动先走空、走不动报空间不够
@@ -1210,6 +1218,8 @@ struct InstanceStart {
     selected_version_journal_position: JournalSequenceNumber,
     /// 这次挂载重读过哪几样（C554 乙），原样交进 `MountOutput::rereads`。
     rereads: RereadsOfThisMount,
+    /// 交回之前那一判实例表清扫的开关、阈值（产品路径 `InstanceTableSweepConfiguration::PRODUCT`）。
+    instance_table_sweep: InstanceTableSweepConfiguration,
 }
 
 /// 新实例的第一次发布的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1
@@ -1272,7 +1282,9 @@ fn tree_identifier_watermark_of_the_ring<Device: BlockDevice>(
 }
 
 /// 进程内重读一个挂载那一刻读不出的根环槽（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定）：只读一次，不重试；读得出、自证过、
-/// 按 `table` 仍然有效 ⇒ 照它的 txg；读得出而自证不过、或按 `table` 被抛弃 ⇒ 当没有根；设备报读错 ⇒ 仍读不出。
+/// 按 `table` 仍然有效 ⇒ 照它的 txg；读得出、自证过而按 `table` 被抛弃 ⇒ 当没有根；设备报读错或读得出而自证不过 ⇒ 同一个处置，
+/// 按根环表记的最大 txg 算（用户 2026-10-04 定改法 A，`research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1：
+/// 这一刻读到「自证不过」既可能是一次读坏的有效根，也可能是从没写过的槽，当没有根在前一种上让回收门槛越过盘上一条有效根）。
 #[must_use]
 pub fn reread_of_a_root_ring_slot_unreadable_at_mount<Reader: PoolReader + ?Sized>(
     reader: &Reader,
@@ -1292,13 +1304,12 @@ pub fn reread_of_a_root_ring_slot_unreadable_at_mount<Reader: PoolReader + ?Size
                 checkpoint_txg: root.checkpoint_txg,
             }
         }
-        RootRingSlotReading::SelfVerified(_)
-        | RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified) => {
-            RereadOfARootRingSlotUnreadableAtMount::NoValidRoot
-        }
-        RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => {
-            RereadOfARootRingSlotUnreadableAtMount::StillUnreadable
+        RootRingSlotReading::SelfVerified(_) => {
+            RereadOfARootRingSlotUnreadableAtMount::AbandonedByTheInstanceTable
         }
+        RootRingSlotReading::Bad(
+            BadRootRingSlotReading::Unreadable | BadRootRingSlotReading::NotSelfVerified,
+        ) => RereadOfARootRingSlotUnreadableAtMount::StillUnreadableOrNotSelfVerified,
     }
 }
 
@@ -2060,7 +2071,7 @@ pub fn user_visible_trees_changed(
 /// 算抬 F 的上限时读根环（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-09-26 定）：每个槽读一次；读坏的槽按这个进程知不知道
 /// 那里住着一条根分两类——`ring_slots_known_to_hold_a_root` 里没有的（挂载那一刻就读不出或自证不过、之后也没写过）当没有根，
 /// 有的（挂载那一刻读得出、或这个进程写过且 FUA 返回过）重读一次，读出来就用重读的那一条，仍坏就拒，不按有根或没根猜。
-/// 交回读得出的每一条根，按根环槽的次序。
+/// 交回读得出的每一条根（按根环槽的次序）与这一遍读坏、当没有根的那几个槽。
 ///
 /// # Errors
 /// 知道住着根的槽重读仍坏 ⇒ `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`。
@@ -2068,7 +2079,7 @@ fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     system_configuration: &crate::system_configuration::SystemConfiguration,
     ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
-) -> Result<Vec<RootRecord>, MountError> {
+) -> Result<RootsOfTheRingForTheCeiling, MountError> {
     let read_once = |ring_slot: RootRingSlot| {
         read_root_ring_slot(
             reader,
@@ -2079,7 +2090,8 @@ fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(
         )
     };
     let mut roots = Vec::new();
-    // 迭代上界是根环的槽数（3 × S）；跨轮携带的只有交回的根；提前出口只有「知道住着根的槽重读仍坏」那一条。
+    let mut ring_slots_bad_on_this_read_taken_as_holding_no_root = BTreeSet::new();
+    // 迭代上界是根环的槽数（3 × S）；跨轮携带的只有交回的根与当没有根的槽；提前出口只有「知道住着根的槽重读仍坏」那一条。
     for ring_slot in every_root_ring_slot(&system_configuration.immutable.sizes) {
         let first_reading = match read_once(ring_slot) {
             RootRingSlotReading::SelfVerified(root) => {
@@ -2089,6 +2101,7 @@ fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(
             RootRingSlotReading::Bad(first_reading) => first_reading,
         };
         if !ring_slots_known_to_hold_a_root.contains(&ring_slot) {
+            ring_slots_bad_on_this_read_taken_as_holding_no_root.insert(ring_slot);
             continue;
         }
         match read_once(ring_slot) {
@@ -2104,7 +2117,18 @@ fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(
             }
         }
     }
-    Ok(roots)
+    Ok(RootsOfTheRingForTheCeiling {
+        roots,
+        ring_slots_bad_on_this_read_taken_as_holding_no_root,
+    })
+}
+
+/// 算上限那一遍读根环读出来的（[`roots_of_the_ring_for_the_ceiling`]）。
+struct RootsOfTheRingForTheCeiling {
+    /// 读得出、自证过的每一条根，按根环槽的次序。
+    roots: Vec<RootRecord>,
+    /// 这一遍读不出或自证不过、又不是「这个进程知道住着根的槽」的槽：当没有根。
+    ring_slots_bad_on_this_read_taken_as_holding_no_root: BTreeSet<RootRingSlot>,
 }
 
 /// 抬 F 的上限（D16（发布语义） 已定项 1）：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)；非空有效根不足 4 个时取最旧的有效根。
@@ -2117,6 +2141,8 @@ fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(
 /// 根环怎么读见 [`roots_of_the_ring_for_the_ceiling`]：`ring_slots_known_to_hold_a_root` 取这个进程分配器上那张根环表
 /// （`allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）；分配器上没装根环表的（只有用例自己拼的分配器，
 /// 可写挂载与 `make_filesystem::allocator_after_make_filesystem` 都装）给空集，读坏的槽一律当没有根。
+/// 这一份不知道哪几个槽是挂载那一刻读不出的（给手里没有这个进程的根环表的调用方：实验装置、用例）；这个进程的准入抬 F 走
+/// [`rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount`]。
 ///
 /// # Errors
 /// 一条有效根一条都没有 ⇒ `RecoveryFailure::NoValidRoot`；要比的一条有效根的树表读不出或解不开 ⇒
@@ -2133,7 +2159,44 @@ pub fn rollback_floor_ceiling<Device: BlockDevice>(
     table: &InstanceTableRecords,
     ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
 ) -> Result<CheckpointTxg, MountError> {
-    let readable = roots_of_the_ring_for_the_ceiling(
+    rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount(
+        devices,
+        system_configuration,
+        current_floor,
+        table,
+        ring_slots_known_to_hold_a_root,
+        &BTreeMap::new(),
+    )
+}
+
+/// 同 [`rollback_floor_ceiling`]，再把窗口里挂载那一刻读不出、之后这个进程没写过的槽当作可能是一条非空有效根，F 不越过它
+/// （D16（发布语义） 已定项 1「抬 F 的上限」与「根槽这一次读坏」两行，用户 2026-10-04 定，
+/// `research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS3；与「一圈窗口里最旧有效根」那一行把这些槽并进回收门槛同一个思路）：
+/// `ring_slots_unreadable_since_the_mount` 是这个进程根环表里还记着的那几个槽、各按它能住的最大 txg
+/// （`allocator::RootRingOccupancy::ring_slots_unreadable_at_mount`，[`ring_slots_unreadable_at_mount_by`]）；这一遍读那一槽又读坏
+/// （读不出或自证不过）、那个 txg 不低于今天的 F 且落在一圈窗口里，上限就取不高于它。这一遍读得出的照读出来的根算，不另封顶。
+/// 判「非空」、找「前一条」照旧只在读得出的根上取（读不出的那一条判不了空不空，F 不越过它就罩住了它是第 4 新的那一种）。
+///
+/// # Errors
+/// 同 [`rollback_floor_ceiling`]。
+#[allow(
+    clippy::ptr_arg,
+    reason = "user_visible_tree_root_pointers 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
+)]
+pub fn rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount<
+    Device: BlockDevice,
+>(
+    devices: &Vec<(DeviceIdentity, Device)>,
+    system_configuration: &crate::system_configuration::SystemConfiguration,
+    current_floor: CheckpointTxg,
+    table: &InstanceTableRecords,
+    ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
+    ring_slots_unreadable_since_the_mount: &BTreeMap<RootRingSlot, CheckpointTxg>,
+) -> Result<CheckpointTxg, MountError> {
+    let RootsOfTheRingForTheCeiling {
+        roots: readable,
+        ring_slots_bad_on_this_read_taken_as_holding_no_root,
+    } = roots_of_the_ring_for_the_ceiling(
         devices,
         system_configuration,
         ring_slots_known_to_hold_a_root,
@@ -2220,12 +2283,28 @@ pub fn rollback_floor_ceiling<Device: BlockDevice>(
             });
         }
     }
-    Ok(ceiling_from_newest_and_non_empty_roots(
+    let ceiling_from_the_roots_read = ceiling_from_newest_and_non_empty_roots(
         newest_on_every_device,
         newest_first_distinct_states(non_empty),
         valid.iter().map(|root| root.checkpoint_txg).min(),
     )
-    .ok_or(RecoveryFailure::NoValidRoot)?)
+    .ok_or(RecoveryFailure::NoValidRoot)?;
+    let lowest_txg_a_ring_slot_unreadable_since_the_mount_can_hold =
+        ring_slots_bad_on_this_read_taken_as_holding_no_root
+            .iter()
+            .filter_map(|ring_slot| ring_slots_unreadable_since_the_mount.get(ring_slot))
+            .copied()
+            .filter(|highest_txg_it_can_hold| *highest_txg_it_can_hold >= current_floor)
+            .filter(|highest_txg_it_can_hold| is_inside_the_window(*highest_txg_it_can_hold))
+            .min();
+    Ok(
+        match lowest_txg_a_ring_slot_unreadable_since_the_mount_can_hold {
+            Some(highest_txg_it_can_hold) => {
+                ceiling_from_the_roots_read.min(highest_txg_it_can_hold)
+            }
+            None => ceiling_from_the_roots_read,
+        },
+    )
 }
 
 /// 一条非空的有效根：它的 (txg, 实例代号) 与它树表里用户可见的两棵树的根指针（去重按后者比）。
@@ -2502,6 +2581,19 @@ pub fn unmount<Device: BlockDevice>(
     ))
 }
 
+/// 这个进程分配器上那张根环表里挂载那一刻读不出、之后没写过的槽，各按它能住的最大 txg
+/// （`allocator::RootRingOccupancy::ring_slots_unreadable_at_mount`）；没装根环表的分配器给空表。准入抬 F 的上限按它封顶
+/// （[`rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount`]）。
+#[must_use]
+pub fn ring_slots_unreadable_at_mount_by(
+    allocator: &PoolAllocator,
+) -> BTreeMap<RootRingSlot, CheckpointTxg> {
+    allocator.root_ring_occupancy().map_or_else(
+        BTreeMap::new,
+        RootRingOccupancy::ring_slots_unreadable_at_mount,
+    )
+}
+
 /// 这个进程分配器上那张根环表里知道住着一条根的槽（`allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`）；
 /// 没装根环表的分配器（只有用例自己拼的）给空集。算抬 F 的上限读根环按它分两类（[`rollback_floor_ceiling`]）。
 #[must_use]
@@ -2569,12 +2661,13 @@ fn raise_to_the_admission_ceiling_on_caller_inputs_agreeing_with_the_disk<Device
     let current_instance_table = instance_table_chain_of_root(&*devices, &current.root)
         .map_err(|_unreadable_or_malformed| MountError::InstanceTableMalformed)?
         .records;
-    let ceiling = rollback_floor_ceiling(
+    let ceiling = rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount(
         devices,
         &caller_inputs_agreeing_with_the_disk.system_configuration,
         current.root.rollback_floor,
         &current_instance_table,
         &ring_slots_known_to_hold_a_root_by(allocator),
+        &ring_slots_unreadable_at_mount_by(allocator),
     )?;
     if ceiling <= current.root.rollback_floor {
         return Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling {
@@ -2849,12 +2942,13 @@ fn raise_the_floor_through<Device: BlockDevice>(
     }
     let ceiling_judged_by_the_admission = match entry {
         RaiseFloorEntry::Admission => {
-            let ceiling = rollback_floor_ceiling(
+            let ceiling = rollback_floor_ceiling_not_passing_ring_slots_unreadable_since_the_mount(
                 devices,
                 &system_configuration,
                 current.root.rollback_floor,
                 &table,
                 &ring_slots_known_to_hold_a_root_by(allocator),
+                &ring_slots_unreadable_at_mount_by(allocator),
             )?;
             if new_floor > ceiling {
                 return Err(MountError::RollbackFloorAboveCeiling {
@@ -4708,6 +4802,15 @@ fn establish_instance<Device: BlockDevice>(
         writes_of_the_publishes_before_the_continuation,
         space_admission.floor_raises(),
     )?;
+    // 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条「谁跑、什么时候跑」，用户 2026-10-03 定）：写行、暖机、抬 F、删除续做之后，
+    // 交回之前判一次。清扫自己不让挂载失败（落点被拒这一轮不删、挂载照样交回可写，Q4），走到哪儿记进交回值。
+    let instance_table_sweep = sweep_the_instance_table_at_the_end_of_the_mount(
+        parameters,
+        devices,
+        &mut allocator,
+        &mut current,
+        start.instance_table_sweep,
+    );
 
     Ok(Mounted {
         output: MountOutput {
@@ -4734,6 +4837,7 @@ fn establish_instance<Device: BlockDevice>(
             floor_raises_pushed_by_the_continuation,
             refusals_that_pushed_the_continuation_floor_raises,
             deletion_queue_continuation_outcome,
+            instance_table_sweep,
         },
         allocator,
         current,
@@ -5352,6 +5456,7 @@ pub fn mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread<
         shadow_ledger,
         DeletionSwitches::PRODUCT,
         before_the_one_reread,
+        InstanceTableSweepConfiguration::PRODUCT,
     )
 }
 
@@ -5375,10 +5480,36 @@ pub fn mount_writable_with_test_only_switches_and_deletion_switches<Device: Bloc
         shadow_ledger,
         deletion_switches,
         BeforeTheOneReread::RereadImmediately,
+        InstanceTableSweepConfiguration::PRODUCT,
+    )
+}
+
+/// 同 [`mount_writable_with_test_only_switches`]，交回之前那一判实例表清扫的开关与阈值也由调用方给
+/// （D18（块里携带什么信息） 已定项 11「回收」一条：开关与阈值是配置项，阈值默认值没定，C690；
+/// [`crate::instance_table_sweep::InstanceTableSweepConfiguration`]）。产品路径走 [`mount_writable`]，取
+/// `InstanceTableSweepConfiguration::PRODUCT`。
+///
+/// # Errors
+/// 同 [`mount_writable_with_test_only_switches`]：清扫自己不报错，它走到哪儿记在 `MountOutput::instance_table_sweep`。
+pub fn mount_writable_with_instance_table_sweep_configuration<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    space_admission: SpaceAdmission,
+    shadow_ledger: ShadowLedger,
+    instance_table_sweep: InstanceTableSweepConfiguration,
+) -> Result<Mounted, MountError> {
+    mount_writable_with_every_test_only_switch(
+        parameters,
+        devices,
+        space_admission,
+        shadow_ledger,
+        DeletionSwitches::PRODUCT,
+        BeforeTheOneReread::RereadImmediately,
+        instance_table_sweep,
     )
 }
 
-/// 可写挂载的全部只供测试的开关一处收：空间准入、影子账、删除开关、「重读一次」之前的钩子。
+/// 可写挂载的全部只供测试的开关一处收：空间准入、影子账、删除开关、「重读一次」之前的钩子、实例表清扫的配置。
 fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
@@ -5386,6 +5517,7 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
     shadow_ledger: ShadowLedger,
     deletion_switches: DeletionSwitches,
     mut before_the_one_reread: BeforeTheOneReread<'_>,
+    instance_table_sweep: InstanceTableSweepConfiguration,
 ) -> Result<Mounted, MountError> {
     admit_the_writable_device_count(distinct_device_identities_handed_in(devices))?;
     refuse_device_identities_handed_in_more_than_once(devices)?;
@@ -5493,6 +5625,7 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
                 read_stage: read_stage_settled,
                 instance_table_of_the_newest_root,
             },
+            instance_table_sweep,
         },
     )
 }
diff --git a/crates/singlefs-harness/src/mount_transparency.rs b/crates/singlefs-harness/src/mount_transparency.rs
index ce4c152..2f4585e 100644
--- a/crates/singlefs-harness/src/mount_transparency.rs
+++ b/crates/singlefs-harness/src/mount_transparency.rs
@@ -16,7 +16,9 @@
 
 use std::collections::BTreeMap;
 
-use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+};
 use singlefs_core::instance_table::InstanceRow;
 use singlefs_core::journal::JournalRecordPlaceInPublish;
 use singlefs_core::mount::{MountError, MountOutput, RaisedFloor, StillUnreadableAfterOneReread};
@@ -31,7 +33,7 @@ use singlefs_core::recovery::{
 use singlefs_core::root_record::{RootRecord, UnmountMarker};
 use singlefs_core::system_configuration::{SystemConfiguration, SystemImmutableConfiguration};
 use singlefs_core::transaction::TransactionOutput;
-use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
+use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
 
 use crate::memory_pool::{root_identity_written_by, RetainedWrite};
 use crate::segments::StepKind;
@@ -1258,10 +1260,11 @@ fn judge_one_field(
     }
 }
 
-/// D23（journal 的角色与格式） 已定项 14 那张单子（C554 乙-配置续，用户 2026-09-27 定；C331 取甲，用户 2026-09-28 定）上的拒可写理由。
+/// D23（journal 的角色与格式） 已定项 14 那张单子（C554 乙-配置续，用户 2026-09-27 定；C331 判不出改取 R2，用户 2026-10-03 定）上的拒可写理由。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub enum RefusalReasonOnTheD23List {
-    /// 读阶段判据 N-配置 为真（有一槽读不出或本池自证不过按真；取号那一刻有一槽读不出也报成它），重读一次仍为真。
+    /// 读阶段判据 N-配置 为真（系统配置见证判不出按真：R2，有槽设备报读错而没有一块盘两槽都自证过；读阶段判完之后某块盘一份自证过的
+    /// 系统配置都没有也报成它，C554 乙-配置续 Q1），重读一次仍为真。
     NewerPublishWitnessedBySystemConfiguration,
     /// 重建分配器判抛弃用的最新那条根的实例表读不出，重读一次仍读不出。
     InstanceTableOfTheNewestRootUnreadable,
@@ -1314,9 +1317,77 @@ pub fn refusal_reason_on_the_d23_list(error: &MountError) -> Option<RefusalReaso
     }
 }
 
+/// 一块盘两个系统配置槽在崩溃镜像上各直接读一次：自证过、fsid 与本池相同的那几份，与设备报读错（`PoolReader::read` 交空）的槽数。
+/// 读得出而自证不过的槽（撕裂的轮换、没写过的槽）两边都不算。
+struct SystemConfigurationSlotsReadOnTheCrashImage {
+    self_verified: Vec<SystemConfiguration>,
+    slots_reporting_a_read_error: u64,
+}
+
+fn system_configuration_slots_read_on_the_crash_image(
+    crash_image: &dyn PoolReader,
+    device: DeviceIdentity,
+    slot_spacing_in_bytes: u64,
+    filesystem_identifier: &[u8; 16],
+) -> SystemConfigurationSlotsReadOnTheCrashImage {
+    let slot_bytes =
+        usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("系统配置槽 4096 字节装得进 usize");
+    let mut slots = SystemConfigurationSlotsReadOnTheCrashImage {
+        self_verified: Vec::new(),
+        slots_reporting_a_read_error: 0,
+    };
+    // 迭代上界是一块盘的两个系统配置槽；跨轮携带的只有两样计数；没有提前出口。
+    for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
+        let offset = DeviceOffsetInBytes(slot_index * slot_spacing_in_bytes);
+        match crash_image.read(device, offset, slot_bytes) {
+            None => slots.slots_reporting_a_read_error += 1,
+            Some(bytes) => {
+                if let Ok(system_configuration) = SystemConfiguration::parse_slot(&bytes) {
+                    if system_configuration.immutable.filesystem_identifier
+                        == *filesystem_identifier
+                    {
+                        slots.self_verified.push(system_configuration);
+                    }
+                }
+            }
+        }
+    }
+    slots
+}
+
+/// 系统配置见证判不出（R2，D22（单元原子性怎么合成） 已定项 8 射程，用户 2026-10-03 定「改取 R2」）在崩溃镜像上独立写的一份，
+/// 不调实现的 `recovery::system_configuration_witness_is_undecidable`：判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过。
+/// 撕裂的轮换（读得出、自证不过，设备没报错）不算判不出；一块盘报读错而另一块盘两槽都自证过也不算。
+fn system_configuration_witness_undecidable_on_the_crash_image(
+    slots_of_every_device: &[SystemConfigurationSlotsReadOnTheCrashImage],
+) -> bool {
+    let some_slot_reported_a_read_error = slots_of_every_device
+        .iter()
+        .any(|slots| slots.slots_reporting_a_read_error > 0);
+    let some_device_self_verified_both_slots = slots_of_every_device.iter().any(|slots| {
+        u64::try_from(slots.self_verified.len()).expect("一块盘的系统配置槽数装得进 u64")
+            == SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
+    });
+    some_slot_reported_a_read_error && !some_device_self_verified_both_slots
+}
+
+/// 某块盘两槽里一份本池自证过的系统配置都没有（C554 乙-配置续 Q1：这块盘不「可见」，D18（块里携带什么信息） 已定项 11）。
+/// R2 之下这一形不并进判不出（一块盘两槽都撕裂、另一块盘两槽都自证过时 R2 判得出），实现在读阶段判完之后单列一个成员报它、
+/// 按判据为真处置（`mount::WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot`，报成判据 N-配置 的拒因，
+/// C664（挂载拒绝合法复算把两种拒因报成一个））；判法照同一个处置复算。
+fn some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image(
+    slots_of_every_device: &[SystemConfigurationSlotsReadOnTheCrashImage],
+) -> bool {
+    slots_of_every_device
+        .iter()
+        .any(|slots| slots.self_verified.is_empty())
+}
+
 /// 判据 N-配置（D23 已定项 14「可写挂载读到的更新状态读不出」那一段）在崩溃镜像上自己复算一遍，不经实现的 `mount::newer_publish_witness`：
-/// 每块盘两槽有一槽读不出或本池自证不过 ⇒ 判不出、按真；全部槽的 journal tail 取最大为 c_见证，0 ⇒ 假；所选那一版那次发布带末条标志的记录读得出 ⇒
-/// c_见证 > 它的计数器；读不出而计数器等于 c_见证 的记录读得出 ⇒ 它的 (实例代号, checkpoint_txg) > 所选那一版；两条都读不出按真。
+/// 系统配置见证判不出（[`system_configuration_witness_undecidable_on_the_crash_image`]，R2）⇒ 按真；某块盘一份自证过的系统配置都没有
+/// （[`some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image`]）⇒ 按真；否则全部自证过的槽的 journal tail
+/// 取最大为 c_见证（撕裂的槽跳过），0 ⇒ 假；所选那一版那次发布带末条标志的记录读得出 ⇒ c_见证 > 它的计数器；读不出而计数器等于 c_见证 的记录
+/// 读得出 ⇒ 它的 (实例代号, checkpoint_txg) > 所选那一版；两条都读不出按真。
 /// `selected_version` 是 R1 落到的那一版（与可写挂载读阶段同一份盘、同一套择根与重放）；没有时判据无从复算，交回假。
 fn newer_publish_witnessed_on_the_crash_image(
     crash_image: &dyn PoolReader,
@@ -1327,24 +1398,31 @@ fn newer_publish_witnessed_on_the_crash_image(
     };
     let immutable = system_configuration.immutable;
     let slot_spacing = u64::from(immutable.sizes.fixed_structure_slot_spacing);
-    let mut witnessed_journal_counter = 0u64;
-    for device in crash_image.device_identities() {
-        let slots = verified_system_configuration_slots(
-            crash_image,
-            device,
-            slot_spacing,
-            &immutable.filesystem_identifier,
-        );
-        if u64::try_from(slots.len()).expect("一块盘的系统配置槽数装得进 u64")
-            < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
-        {
-            return true;
-        }
-        witnessed_journal_counter = slots
-            .iter()
-            .map(|slot| slot.quantities.journal_tail)
-            .fold(witnessed_journal_counter, u64::max);
+    let slots_of_every_device: Vec<SystemConfigurationSlotsReadOnTheCrashImage> = crash_image
+        .device_identities()
+        .into_iter()
+        .map(|device| {
+            system_configuration_slots_read_on_the_crash_image(
+                crash_image,
+                device,
+                slot_spacing,
+                &immutable.filesystem_identifier,
+            )
+        })
+        .collect();
+    if system_configuration_witness_undecidable_on_the_crash_image(&slots_of_every_device)
+        || some_device_without_any_self_verified_system_configuration_slot_on_the_crash_image(
+            &slots_of_every_device,
+        )
+    {
+        return true;
     }
+    let witnessed_journal_counter = slots_of_every_device
+        .iter()
+        .flat_map(|slots| slots.self_verified.iter())
+        .map(|slot| slot.quantities.journal_tail)
+        .max()
+        .unwrap_or(0);
     witnessed_counter_holds_against_the_selected_version(
         crash_image,
         &system_configuration,
@@ -1408,9 +1486,10 @@ fn witnessed_counter_holds_against_the_selected_version(
 
 /// C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）：判据 N-配置 在没注入读故障的崩溃镜像上复算，为真交回是哪一支。
 /// 没有读故障时设备不报读错，见证读判得出（R2，D22（单元原子性怎么合成） 已定项 8 射程：判不出 ⟺ 有槽设备报读错且没有一块盘两槽都自证过；
-/// 撕裂的系统配置轮换不算判不出）：c_见证 取全部自证过、fsid 与本池相同的槽 journal tail 的最大值，自证不过的槽（撕裂的、没写过的）跳过。
-/// 撕裂系统配置那一支因此不落在这里（C664（挂载拒绝合法复算把两种拒因报成一个）、C331（择根倒挂压过已确认的写））：
-/// [`newer_publish_witnessed_on_the_crash_image`] 照甲把它判成按真，那一支的拒照 U6 判。一份自证过的系统配置都择不到时 c_见证 为 0，交回 `None`。
+/// 撕裂的系统配置轮换不算判不出）：c_见证 取全部自证过、fsid 与本池相同的槽 journal tail 的最大值，自证不过的槽（撕裂的、没写过的）跳过——
+/// 与 [`newer_publish_witnessed_on_the_crash_image`] 改取 R2 之后在撕裂的轮换上同一个取法。两边只在一形上分开：某块盘两槽都撕裂、
+/// 一份自证过的都没有（C554 乙-配置续 Q1，这块盘不「可见」；C664（挂载拒绝合法复算把两种拒因报成一个）），那一判按真、这里照常比，
+/// 那一形的拒照 U6 判。一份自证过的系统配置都择不到时 c_见证 为 0，交回 `None`。
 fn witnessed_counter_holds_on_a_crash_image_without_read_faults(
     crash_image: &dyn PoolReader,
     selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
diff --git a/crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs b/crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs
new file mode 100644
index 0000000..90406a8
--- /dev/null
+++ b/crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs
@@ -0,0 +1,822 @@
+//! 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条；判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第二节 S1
+//! 与第六节用户定案；攻方原型 `research/prompts/m3-sweep-and-tree-id-r1-opus-model/sweep-proto/src/lib.rs` 的几形）。
+//!
+//! 盘面（S = 4 的 12 槽小环、两块 128 MiB 内存盘）：mkfs → 实例 1 第一个文件、覆盖写两次（txg 3、4、5）→ 一次十六个数据单元的顺序写崩在第一条 journal
+//! 记录写之前（单元写落了：实例 1 按它那一行判未发布的孤儿）→ 实例 2 可写挂载（写行 6、暖机 7）→ 实例 2 覆盖写到 txg 19（根环里没有实例 1 的根了）。
+//! 接着的可写挂载（实例 3）末尾判一次清扫。
+mod common;
+mod common_root_ring_holes;
+
+use std::collections::BTreeSet;
+
+use common::{
+    with_unreadable_ranges, FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange,
+    UnreadableRangeReadBack,
+};
+use common_root_ring_holes::{
+    content_of_the_version, devices_of, fixed_geometry_of_the_ring_hole_pools,
+    parameters_of_the_ring_hole_pools, pool_of, recording_of, RingHoleDevices,
+    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS, WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
+};
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{DeviceIdentity, InstanceGeneration, SlotNumber};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::instance_table::InstanceRow;
+use singlefs_core::instance_table_sweep::{
+    erase_the_heads_and_publish_the_sweep, judge_the_instance_table_sweep,
+    sweep_the_instance_table_while_mounted, unit_head_read_by_sweep,
+    InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
+    InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
+    NoRowDeletedThisRound, PlacementOfAUnitHead, SweepPublishPlacements, UnitHeadReadBySweep,
+};
+use singlefs_core::make_filesystem::make_filesystem;
+use singlefs_core::mount::{
+    mount_writable, mount_writable_with_instance_table_sweep_configuration, Mounted, ShadowLedger,
+};
+use singlefs_core::recovery::{
+    choose_system_configuration, instance_table_chain_of_root, readable_roots, recover,
+    JournalPolicy, PoolReader, RecoveryOutcome,
+};
+use singlefs_core::root_record::RootRecord;
+use singlefs_core::transaction::{
+    publish_first_file, publish_overwrite, publish_sequential_write, FirstFile, PoolVersion,
+    PoolWriter, TransactionOutput,
+};
+use singlefs_core::unit::data_unit_payload_capacity;
+use singlefs_format::{DATA_UNIT_BYTES, SLOT_BYTES};
+use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::SharedStream;
+
+/// 实例 2 覆盖写到的 txg：根环 12 槽里只剩 txg 8–19，实例 1 的根全部轮出。
+const LAST_TXG_BEFORE_THE_SWEEPING_MOUNT: u64 = 19;
+/// 崩掉的那次顺序写写几个数据单元：比实例 2 那十二次覆盖写多，实例 3 挂载时还有孤儿数据单元留着
+/// （覆盖写的数据单元取起点最小的空槽对，孤儿那几对在实例 2 的分配器里是空的，先被取走）。
+const ORPHAN_DATA_UNITS: usize = 16;
+/// 后台那一支判完之后至多覆盖写几次去占单子上的落点。
+const OVERWRITES_AFTER_THE_JUDGEMENT_AT_MOST: u64 = 128;
+/// 一个码 1 头的字节数（伪头照抄一个孤儿的头）。
+const DATA_UNIT_HEAD_BYTES: usize = 105;
+/// 带伪头那一版的内容长度：伪头落在数据单元第二个槽开头，内容要够长、又装得进一个数据单元。
+const CONTENT_BYTES_CARRYING_A_FORGED_HEAD: usize = 20000;
+/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
+const INVARIANTS_THE_SWEEP_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];
+
+fn violations_of(image: &MemoryPool) -> Vec<String> {
+    check_pool_image(image)
+        .into_iter()
+        .filter(|(invariant, _)| INVARIANTS_THE_SWEEP_KEEPS_GREEN.contains(invariant))
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
+            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+        })
+        .collect()
+}
+
+fn sweeping_at_the_end_of_the_mount_on_any_row() -> InstanceTableSweepConfiguration {
+    InstanceTableSweepConfiguration {
+        switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
+        row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
+        sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
+    }
+}
+
+/// 一个槽里的单元头字节（读清扫认头那么长）。
+fn head_bytes_at(image: &MemoryPool, placement: PlacementOfAUnitHead) -> Vec<u8> {
+    PoolReader::read(
+        image,
+        placement.device,
+        placement.slot.to_device_offset(),
+        1024,
+    )
+    .expect("内存盘上读得出")
+}
+
+/// 实例 1 的那几个孤儿：一次顺序写崩在第一条 journal 记录写之前，单元写落了的那几个（落点与头）。
+#[derive(Clone, Debug)]
+struct OrphansOfInstanceOne {
+    heads: Vec<(PlacementOfAUnitHead, UnitHeadReadBySweep)>,
+    /// 第一个孤儿数据单元的头字节（伪头照抄它）。
+    first_data_unit_head_bytes: Vec<u8>,
+}
+
+/// 带伪头的那一版：数据单元在哪（第一个槽），第二个槽开头的字节（伪头）。
+#[derive(Clone, Debug)]
+struct VersionCarryingAForgedHead {
+    data_unit: PlacementOfAUnitHead,
+    second_slot_bytes: Vec<u8>,
+}
+
+/// 清扫那次挂载之前的盘面。
+struct PoolWithADeadInstance {
+    image: MemoryPool,
+    /// 盘上还留着的孤儿（顺序写落下的、实例 2 覆盖写没盖掉的）。
+    orphans_left_on_disk: Vec<(PlacementOfAUnitHead, UnitHeadReadBySweep)>,
+    forged: Option<VersionCarryingAForgedHead>,
+}
+
+fn publish_an_overwrite<Device: singlefs_core::block_device::BlockDevice>(
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    allocator: &mut singlefs_core::allocator::PoolAllocator,
+    current: &TransactionOutput,
+    content: &[u8],
+    instance: InstanceGeneration,
+) -> TransactionOutput {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let salt = current.root.checkpoint_txg.0 + 1;
+    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+    publish_overwrite(
+        &mut writer,
+        allocator,
+        current,
+        FirstFile {
+            content,
+            write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
+        },
+        instance,
+    )
+    .expect("覆盖写")
+}
+
+/// mkfs → 实例 1 写到 txg 5 → 顺序写崩在第一条记录写之前；交回崩溃之后的盘面与孤儿。
+fn pool_with_orphans_of_instance_one() -> (MemoryPool, OrphansOfInstanceOne) {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let mut devices: RingHoleDevices = (0..2u32)
+        .map(|number| {
+            (
+                DeviceIdentity(number),
+                SparseBlockDevice::new(
+                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
+                    PhysicalBlockSizeInBytes(512),
+                ),
+            )
+        })
+        .collect();
+    make_filesystem(&parameters, &mut devices).expect("mkfs");
+    let Mounted {
+        output,
+        mut allocator,
+        current,
+    } = mount_writable(&parameters, &mut devices).expect("实例 1 的可写挂载");
+    let instance = output.instance;
+    let mut current = {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut allocator,
+            current.root(),
+            FirstFile {
+                content: &content_of_the_version(0),
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
+            },
+            instance,
+            current.record_bytes(),
+        )
+        .expect("第一个文件")
+    };
+    // 迭代上界：两次覆盖写。
+    for _ in 0..2 {
+        current = publish_an_overwrite(
+            &mut devices,
+            &mut allocator,
+            &current,
+            &content_of_the_version(current.root.checkpoint_txg.0 + 1),
+            instance,
+        );
+    }
+    let before = pool_of(&devices);
+    let stream = SharedStream::retaining_contents();
+    let mut recording = recording_of(&before, &stream);
+    {
+        let mut writer = PoolWriter::new(&parameters, recording.as_mut_slice());
+        publish_sequential_write(
+            &mut writer,
+            &mut allocator.clone(),
+            &current,
+            FirstFile {
+                content: &vec![7u8; ORPHAN_DATA_UNITS * data_unit_payload_capacity()],
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + 100,
+            },
+            instance,
+        )
+        .expect("顺序写（录下来、之后截掉）");
+    }
+    let (writes, _segments) = writes_and_segments(
+        &stream.retained_operations(),
+        &fixed_geometry_of_the_ring_hole_pools(),
+    );
+    let first_record_write = writes
+        .iter()
+        .position(|write| write.kind == StepKind::JournalRecord)
+        .expect("顺序写有 journal 记录写");
+    let mut image = before;
+    image.apply_writes(&writes[..first_record_write]);
+    let heads: Vec<(PlacementOfAUnitHead, UnitHeadReadBySweep)> = writes[..first_record_write]
+        .iter()
+        .filter(|write| write.kind == StepKind::UnitWrite)
+        .filter_map(|write| {
+            let head = unit_head_read_by_sweep(write.bytes()?)?;
+            Some((
+                PlacementOfAUnitHead {
+                    device: write.device,
+                    slot: SlotNumber(write.offset.0 / SLOT_BYTES),
+                },
+                head,
+            ))
+        })
+        .collect();
+    let first_data_unit_head_bytes = writes[..first_record_write]
+        .iter()
+        .filter(|write| write.kind == StepKind::UnitWrite)
+        .filter_map(|write| write.bytes())
+        .find(|bytes| unit_head_read_by_sweep(bytes).is_some_and(|head| head.unit_class == 1))
+        .expect("顺序写落下了数据单元")[..DATA_UNIT_HEAD_BYTES]
+        .to_vec();
+    assert!(
+        heads
+            .iter()
+            .all(|(_, head)| head.writer_instance == instance),
+        "孤儿都是实例 1 写的"
+    );
+    (
+        image,
+        OrphansOfInstanceOne {
+            heads,
+            first_data_unit_head_bytes,
+        },
+    )
+}
+
+/// 孤儿之后：实例 2 可写挂载、覆盖写到 txg 19；`forged_head_in_the_content_of_txg` 给出时，那一版的内容在数据单元第二个槽开头带一个照抄孤儿的伪头。
+fn pool_with_a_dead_instance(
+    forged_head_in_the_content_of_txg: Option<u64>,
+) -> PoolWithADeadInstance {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let (image, orphans) = pool_with_orphans_of_instance_one();
+    let mut devices = devices_of(&image);
+    let mounted = mount_writable(&parameters, &mut devices).expect("实例 2 的可写挂载");
+    assert_eq!(mounted.output.instance, InstanceGeneration(2));
+    let mut allocator = mounted.allocator;
+    let mut current = mounted.current.into_file_version().expect("这一版带文件");
+    let payload_offset_of_the_second_slot = usize::try_from(SLOT_BYTES).expect("16384")
+        - (usize::try_from(DATA_UNIT_BYTES).expect("32768") - data_unit_payload_capacity());
+    let mut forged = None;
+    // 迭代上界：覆盖写到 txg 19；跨轮携带的是现行那一版与带伪头那一版在哪。
+    while current.root.checkpoint_txg.0 < LAST_TXG_BEFORE_THE_SWEEPING_MOUNT {
+        let txg = current.root.checkpoint_txg.0 + 1;
+        let content = if forged_head_in_the_content_of_txg == Some(txg) {
+            let mut content = vec![0x5Au8; CONTENT_BYTES_CARRYING_A_FORGED_HEAD];
+            content[payload_offset_of_the_second_slot
+                ..payload_offset_of_the_second_slot + DATA_UNIT_HEAD_BYTES]
+                .copy_from_slice(&orphans.first_data_unit_head_bytes);
+            content
+        } else {
+            content_of_the_version(txg)
+        };
+        current = publish_an_overwrite(
+            &mut devices,
+            &mut allocator,
+            &current,
+            &content,
+            InstanceGeneration(2),
+        );
+        if forged_head_in_the_content_of_txg == Some(txg) {
+            let location = current.data_pointers[0].locations[0];
+            forged = Some(PlacementOfAUnitHead {
+                device: location.device,
+                slot: location.slot,
+            });
+        }
+    }
+    let image_before_the_sweeping_mount = pool_of(&devices);
+    let orphans_left_on_disk = orphans
+        .heads
+        .iter()
+        .copied()
+        .filter(|(placement, head)| {
+            unit_head_read_by_sweep(&head_bytes_at(&image_before_the_sweeping_mount, *placement))
+                == Some(*head)
+        })
+        .collect();
+    let forged = forged.map(|data_unit| {
+        let second_slot = PlacementOfAUnitHead {
+            device: data_unit.device,
+            slot: SlotNumber(data_unit.slot.0 + 1),
+        };
+        VersionCarryingAForgedHead {
+            data_unit,
+            second_slot_bytes: head_bytes_at(&image_before_the_sweeping_mount, second_slot),
+        }
+    });
+    PoolWithADeadInstance {
+        image: image_before_the_sweeping_mount,
+        orphans_left_on_disk,
+        forged,
+    }
+}
+
+/// 在 `image` 上按 `configuration` 起可写挂载（实例 3），盘外面包录制器：交回挂载、盘与录下的流。
+fn sweeping_mount_on(
+    image: &MemoryPool,
+    configuration: InstanceTableSweepConfiguration,
+) -> (
+    Mounted,
+    Vec<(
+        DeviceIdentity,
+        singlefs_harness::RecordingBlockDevice<SparseBlockDevice>,
+    )>,
+    SharedStream,
+) {
+    let stream = SharedStream::retaining_contents();
+    let mut devices = recording_of(image, &stream);
+    let mounted = mount_writable_with_instance_table_sweep_configuration(
+        &parameters_of_the_ring_hole_pools(),
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        configuration,
+    )
+    .expect("实例 3 的可写挂载");
+    (mounted, devices, stream)
+}
+
+fn image_after(image: &MemoryPool, stream: &SharedStream) -> MemoryPool {
+    let mut after = image.clone();
+    after.apply(&stream.retained_operations());
+    after
+}
+
+/// 盘上最新那条根指着的实例表里有哪几个实例的行。
+fn instances_with_a_row_on(image: &MemoryPool) -> Vec<InstanceGeneration> {
+    let system_configuration = choose_system_configuration(image).expect("系统配置");
+    let immutable = &system_configuration.immutable;
+    let newest: RootRecord = readable_roots(
+        image,
+        &immutable.region_devices,
+        &immutable.sizes,
+        &immutable.filesystem_identifier,
+    )
+    .into_iter()
+    .max_by_key(|root| (root.checkpoint_txg, root.instance))
+    .expect("根环里有根");
+    instance_table_chain_of_root(image, &newest)
+        .expect("实例表读得出")
+        .records
+        .rows
+        .iter()
+        .map(|row: &InstanceRow| row.instance)
+        .collect()
+}
+
+fn placements_of(
+    heads: &[(PlacementOfAUnitHead, UnitHeadReadBySweep)],
+) -> BTreeSet<PlacementOfAUnitHead> {
+    heads.iter().map(|(placement, _)| *placement).collect()
+}
+
+/// 场景：清扫那次挂载开关关着、阈值取「有一行就扫」。
+/// 预期：根环里没有实例 1 的根，实例 1 那一行是候选行：盘上留着的实例 1 的孤儿头全被抹掉（读不出单元头），清扫发布删掉那一行
+/// （最新那条根的实例表里只剩实例 2 的行）；挂载之后那几条不变量不红。
+#[test]
+fn the_row_of_a_dead_instance_is_deleted_after_its_roots_left_the_ring_and_its_unpublished_heads_are_erased(
+) {
+    let pool = pool_with_a_dead_instance(None);
+    assert!(
+        !pool.orphans_left_on_disk.is_empty(),
+        "前提：盘上还留着实例 1 的孤儿"
+    );
+    let (mounted, _devices, stream) =
+        sweeping_mount_on(&pool.image, sweeping_at_the_end_of_the_mount_on_any_row());
+    let InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
+        rows_deleted,
+        heads_erased,
+    }) = mounted.output.instance_table_sweep
+    else {
+        panic!("删了实例 1 的行：{:?}", mounted.output.instance_table_sweep);
+    };
+    assert_eq!(
+        rows_deleted
+            .iter()
+            .map(|row| row.instance)
+            .collect::<Vec<_>>(),
+        vec![InstanceGeneration(1)],
+        "删的是实例 1 那一行"
+    );
+    assert_eq!(
+        heads_erased.iter().copied().collect::<BTreeSet<_>>(),
+        placements_of(&pool.orphans_left_on_disk),
+        "抹的正好是盘上留着的孤儿"
+    );
+    let after = image_after(&pool.image, &stream);
+    for (placement, _) in &pool.orphans_left_on_disk {
+        assert_eq!(
+            unit_head_read_by_sweep(&head_bytes_at(&after, *placement)),
+            None,
+            "孤儿 {placement:?} 的头抹掉了"
+        );
+    }
+    assert_eq!(
+        instances_with_a_row_on(&after),
+        vec![InstanceGeneration(2)],
+        "清扫发布那一版的实例表里只剩实例 2 的行"
+    );
+    assert_eq!(violations_of(&after), Vec::<String>::new(), "清扫之后");
+}
+
+/// 场景：实例 2 覆盖写 txg 8 那一版的内容在数据单元第二个槽开头带一个照抄实例 1 孤儿的伪头（头校验和过、按实例 1 那一行判未发布）；
+/// 那一版到清扫时已经出了根环、那个数据单元已释放、没被别的根的账引用，第一个槽里它自己的头还在。
+/// 预期：认领规则按整单元跳——第一个槽认出码 1 单元，第二个槽被认领、不当探针：伪头不被抹（字节不变，不在抹了的落点里）。
+#[test]
+fn a_forged_head_inside_the_content_of_a_data_unit_is_claimed_by_the_unit_and_not_erased() {
+    let pool = pool_with_a_dead_instance(Some(8));
+    let forged = pool.forged.clone().expect("带伪头那一版写了");
+    assert!(
+        unit_head_read_by_sweep(&head_bytes_at(&pool.image, forged.data_unit))
+            .is_some_and(|head| head.writer_instance == InstanceGeneration(2)),
+        "前提：那个数据单元第一个槽里还是它自己的头（没被复用）"
+    );
+    assert!(
+        unit_head_read_by_sweep(&forged.second_slot_bytes)
+            .is_some_and(|head| head.writer_instance == InstanceGeneration(1)),
+        "前提：第二个槽开头是一个认得出的、实例 1 的伪头"
+    );
+    let second_slot = PlacementOfAUnitHead {
+        device: forged.data_unit.device,
+        slot: SlotNumber(forged.data_unit.slot.0 + 1),
+    };
+    let (mounted, _devices, stream) =
+        sweeping_mount_on(&pool.image, sweeping_at_the_end_of_the_mount_on_any_row());
+    let InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
+        heads_erased,
+        ..
+    }) = &mounted.output.instance_table_sweep
+    else {
+        panic!("这一轮删了行：{:?}", mounted.output.instance_table_sweep);
+    };
+    assert!(
+        !heads_erased.contains(&second_slot),
+        "伪头所在的槽不在抹了的落点里"
+    );
+    assert_eq!(
+        head_bytes_at(&image_after(&pool.image, &stream), second_slot),
+        forged.second_slot_bytes,
+        "伪头那一槽的字节不变"
+    );
+}
+
+/// 场景：实例 2 覆盖写 txg 14 那一版的内容带同样的伪头；清扫时那一版的根还在根环里（回退候选），它的数据单元在它的账里仍分配。
+/// 再把那个数据单元第一个槽里的头翻坏一个字节（认领规则在那一槽认不出单元，第二个槽成了探针）。
+/// 预期：跳过根环里可读根的账里仍分配的槽——伪头所在的槽在那条根的账里，不被抹（字节不变，不在抹了的落点里）。
+#[test]
+fn a_head_inside_a_unit_still_allocated_in_the_account_of_a_ring_root_is_skipped_and_not_erased() {
+    let mut pool = pool_with_a_dead_instance(Some(14));
+    let forged = pool.forged.clone().expect("带伪头那一版写了");
+    pool.image.flip_byte(
+        forged.data_unit.device,
+        forged.data_unit.slot.to_device_offset(),
+        20,
+    );
+    assert_eq!(
+        unit_head_read_by_sweep(&head_bytes_at(&pool.image, forged.data_unit)),
+        None,
+        "前提：第一个槽认不出单元头了"
+    );
+    let second_slot = PlacementOfAUnitHead {
+        device: forged.data_unit.device,
+        slot: SlotNumber(forged.data_unit.slot.0 + 1),
+    };
+    let (mounted, _devices, stream) =
+        sweeping_mount_on(&pool.image, sweeping_at_the_end_of_the_mount_on_any_row());
+    let InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
+        heads_erased,
+        ..
+    }) = &mounted.output.instance_table_sweep
+    else {
+        panic!("这一轮删了行：{:?}", mounted.output.instance_table_sweep);
+    };
+    assert!(
+        !heads_erased.contains(&second_slot),
+        "伪头所在的槽在环里那条根的账里，不在抹了的落点里"
+    );
+    assert_eq!(
+        head_bytes_at(&image_after(&pool.image, &stream), second_slot),
+        forged.second_slot_bytes,
+        "伪头那一槽的字节不变"
+    );
+}
+
+/// 场景：清扫那次挂载里，第一个孤儿所在的槽每次读都报设备错（挂载自己不读它）。
+/// 预期：这一次清扫有一次读失败：一行都不删、一个头都不抹（结局报那个落点）；挂载照样做成，实例 1 那一行还在，孤儿头都还在。
+#[test]
+fn a_read_failure_on_one_placement_deletes_no_row_and_erases_no_head() {
+    let pool = pool_with_a_dead_instance(None);
+    let (first_orphan, _) = *pool
+        .orphans_left_on_disk
+        .first()
+        .expect("前提：盘上还留着实例 1 的孤儿");
+    let unreadable = SharedUnreadableRanges::new(
+        vec![UnreadableRange {
+            device: first_orphan.device,
+            offset_in_bytes: first_orphan.slot.to_device_offset().0,
+            length_in_bytes: 512,
+            failing_reads: FailingReadsOfARange::Every,
+        }],
+        UnreadableRangeReadBack::DeviceError,
+    );
+    let stream = SharedStream::retaining_contents();
+    let mut devices = with_unreadable_ranges(recording_of(&pool.image, &stream), &unreadable);
+    let mounted = mount_writable_with_instance_table_sweep_configuration(
+        &parameters_of_the_ring_hole_pools(),
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        sweeping_at_the_end_of_the_mount_on_any_row(),
+    )
+    .expect("读故障只在清扫读的那一槽：挂载照样做成");
+    assert!(
+        matches!(
+            mounted.output.instance_table_sweep,
+            InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::NoRowDeleted(
+                NoRowDeletedThisRound::AReadOfAPlacementFailed { placement }
+            )) if placement == first_orphan
+        ),
+        "读失败这一轮不删：{:?}",
+        mounted.output.instance_table_sweep
+    );
+    let after = image_after(&pool.image, &stream);
+    assert_eq!(
+        instances_with_a_row_on(&after),
+        vec![InstanceGeneration(1), InstanceGeneration(2)],
+        "实例 1 那一行还在"
+    );
+    for (placement, head) in &pool.orphans_left_on_disk {
+        assert_eq!(
+            unit_head_read_by_sweep(&head_bytes_at(&after, *placement)),
+            Some(*head),
+            "孤儿 {placement:?} 的头还在"
+        );
+    }
+}
+
+/// 场景：清扫交给后台（开关开着），挂载之后后台那一轮只拿到池里一块盘。
+/// 预期：挂载末尾不扫；后台那一轮设备不全，一行都不删（结局报盘数）。
+#[test]
+fn the_background_sweep_with_a_device_missing_deletes_no_row() {
+    let pool = pool_with_a_dead_instance(None);
+    let (mut mounted, devices, _stream) = sweeping_mount_on(
+        &pool.image,
+        InstanceTableSweepConfiguration {
+            switch: InstanceTableSweepSwitch::HandedToTheBackgroundSweepWhileMounted,
+            row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
+            sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
+        },
+    );
+    assert!(
+        matches!(
+            mounted.output.instance_table_sweep,
+            InstanceTableSweepAtTheEndOfTheMount::HandedToTheBackgroundSweepWhileMounted
+        ),
+        "开关开着：挂载末尾不扫"
+    );
+    let mut only_the_first_device: Vec<_> = devices.into_iter().take(1).collect();
+    let outcome = sweep_the_instance_table_while_mounted(
+        &parameters_of_the_ring_hole_pools(),
+        &mut only_the_first_device,
+        &mut mounted.allocator,
+        &mut mounted.current,
+        SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
+    );
+    assert!(
+        matches!(
+            outcome,
+            InstanceTableSweepOutcome::NoRowDeleted(
+                NoRowDeletedThisRound::NotEveryDeviceOfThePoolIsPresent {
+                    devices_present: 1,
+                    devices_of_the_pool: 2
+                }
+            )
+        ),
+        "设备不全不删：{outcome:?}"
+    );
+}
+
+/// 场景：清扫交给后台。后台那一轮先判（单子上是孤儿），判完到抹之间实例 3 接着覆盖写，直到单子上有一个落点被现行那一版的数据单元占了，再抹、发。
+/// 预期：抹之前重读那一槽的头：已经不是候选行的未发布单元，不抹；单子上别的照抹，实例 1 那一行照删；现行那一版的内容读得回来、那几条不变量不红。
+#[test]
+fn the_background_sweep_rereads_each_head_before_erasing_it_and_leaves_a_slot_allocated_after_the_judgement(
+) {
+    let pool = pool_with_a_dead_instance(None);
+    let (mut mounted, mut devices, stream) = sweeping_mount_on(
+        &pool.image,
+        InstanceTableSweepConfiguration {
+            switch: InstanceTableSweepSwitch::HandedToTheBackgroundSweepWhileMounted,
+            row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
+            sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
+        },
+    );
+    let judgement =
+        judge_the_instance_table_sweep(&devices, mounted.current.root()).expect("后台那一轮判得出");
+    let judged: BTreeSet<PlacementOfAUnitHead> = judgement
+        .heads_to_erase
+        .iter()
+        .map(|to_erase| to_erase.placement)
+        .collect();
+    assert_eq!(
+        judged,
+        placements_of(&pool.orphans_left_on_disk),
+        "单子上是盘上留着的孤儿"
+    );
+    let mut current = mounted
+        .current
+        .file_version()
+        .expect("这一版带文件")
+        .clone();
+    let mut reused = None;
+    let mut last_content = Vec::new();
+    // 迭代上界：至多 OVERWRITES_AFTER_THE_JUDGEMENT_AT_MOST 次覆盖写；跨轮携带的是现行那一版；提前出口是单子上有一个落点的头被盖掉了。
+    for step in 0..OVERWRITES_AFTER_THE_JUDGEMENT_AT_MOST {
+        last_content = content_of_the_version(300 + step);
+        current = publish_an_overwrite(
+            &mut devices,
+            &mut mounted.allocator,
+            &current,
+            &last_content,
+            InstanceGeneration(3),
+        );
+        reused = judgement
+            .heads_to_erase
+            .iter()
+            .find(|to_erase| {
+                let bytes = PoolReader::read(
+                    &devices,
+                    to_erase.placement.device,
+                    to_erase.placement.slot.to_device_offset(),
+                    1024,
+                )
+                .expect("内存盘上读得出");
+                unit_head_read_by_sweep(&bytes) != Some(to_erase.head)
+            })
+            .map(|to_erase| to_erase.placement);
+        if reused.is_some() {
+            break;
+        }
+    }
+    let reused = reused.expect("前提：判完之后的覆盖写盖掉了单子上一个落点的头");
+    let mut current_version = PoolVersion::of_a_version_of_trees(current);
+    let outcome = erase_the_heads_and_publish_the_sweep(
+        &parameters_of_the_ring_hole_pools(),
+        &mut devices,
+        &mut mounted.allocator,
+        &mut current_version,
+        &judgement,
+        SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
+    );
+    let InstanceTableSweepOutcome::RowsDeleted {
+        rows_deleted,
+        heads_erased,
+    } = outcome
+    else {
+        panic!("这一轮删了行：{outcome:?}");
+    };
+    assert!(
+        !heads_erased.contains(&reused),
+        "被现行那一版占了的那一槽没抹"
+    );
+    assert_eq!(
+        rows_deleted
+            .iter()
+            .map(|row| row.instance)
+            .collect::<Vec<_>>(),
+        vec![InstanceGeneration(1)],
+        "实例 1 那一行照删"
+    );
+    let after = image_after(&pool.image, &stream);
+    assert!(
+        matches!(
+            &recover(&after, JournalPolicy::Consult).outcome,
+            RecoveryOutcome::FileRead { content, .. } if *content == last_content
+        ),
+        "现行那一版的内容读得回来"
+    );
+    assert_eq!(violations_of(&after), Vec::<String>::new(), "清扫之后");
+}
+
+/// 场景：清扫那次挂载用只供测试的落点开关：清扫发布在一份每个空闲槽都被占掉的分配器拷贝上取落点（真的取落点那一段，被拒）。
+/// 预期：清扫发布被落点拒，这一轮不删行（实例 1 那一行还在）；挂载照样交回可写——接着在交回的分配器上覆盖写一次做成。
+#[test]
+fn a_sweep_publish_refused_for_its_placement_deletes_no_row_and_the_mount_stays_writable() {
+    let pool = pool_with_a_dead_instance(None);
+    let (mut mounted, mut devices, stream) = sweeping_mount_on(
+        &pool.image,
+        InstanceTableSweepConfiguration {
+            switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
+            row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
+            sweep_publish_placements:
+                SweepPublishPlacements::EveryFreeSlotTakenAwayByTheTestOnlySwitch,
+        },
+    );
+    assert!(
+        matches!(
+            mounted.output.instance_table_sweep,
+            InstanceTableSweepAtTheEndOfTheMount::Swept(
+                InstanceTableSweepOutcome::SweepPublishRefusedForSpace { .. }
+            )
+        ),
+        "清扫发布被落点拒：{:?}",
+        mounted.output.instance_table_sweep
+    );
+    assert_eq!(
+        instances_with_a_row_on(&image_after(&pool.image, &stream)),
+        vec![InstanceGeneration(1), InstanceGeneration(2)],
+        "这一轮不删行"
+    );
+    let current = mounted
+        .current
+        .file_version()
+        .expect("这一版带文件")
+        .clone();
+    publish_an_overwrite(
+        &mut devices,
+        &mut mounted.allocator,
+        &current,
+        &content_of_the_version(400),
+        InstanceGeneration(3),
+    );
+}
+
+/// 场景：产品路径的配置（开关关着、阈值取占位 369）：实例表两行，没超过阈值。
+/// 预期：挂载末尾不扫（结局报行数与阈值），孤儿头都还在。
+#[test]
+fn the_product_configuration_does_not_sweep_a_table_at_or_below_the_threshold() {
+    let pool = pool_with_a_dead_instance(None);
+    let (mounted, _devices, stream) =
+        sweeping_mount_on(&pool.image, InstanceTableSweepConfiguration::PRODUCT);
+    assert!(
+        matches!(
+            mounted.output.instance_table_sweep,
+            InstanceTableSweepAtTheEndOfTheMount::NotSweptAtOrBelowTheThreshold {
+                rows: 2,
+                threshold: InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690
+            }
+        ),
+        "没超过阈值不扫：{:?}",
+        mounted.output.instance_table_sweep
+    );
+    let after = image_after(&pool.image, &stream);
+    for (placement, head) in &pool.orphans_left_on_disk {
+        assert_eq!(
+            unit_head_read_by_sweep(&head_bytes_at(&after, *placement)),
+            Some(*head),
+            "孤儿 {placement:?} 的头还在"
+        );
+    }
+}
+
+/// 场景：mkfs 之后一直不写文件、接连可写挂载七次（实例 1 的根轮出了 12 槽的环），第八次挂载开关关着、有一行就扫。
+/// 预期：现行那一版树表 0 条，第一版不清扫：结局报这个成员；这次挂载写下的盘面与不扫的同一次挂载逐字节相同（清扫一个字节都没写）。
+#[test]
+fn a_sweep_on_a_version_without_file_is_not_supported_and_writes_nothing() {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let mut devices: RingHoleDevices = (0..2u32)
+        .map(|number| {
+            (
+                DeviceIdentity(number),
+                SparseBlockDevice::new(
+                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
+                    PhysicalBlockSizeInBytes(512),
+                ),
+            )
+        })
+        .collect();
+    make_filesystem(&parameters, &mut devices).expect("mkfs");
+    // 迭代上界：七次可写挂载，每次写行与暖机两三条根。
+    for _ in 0..7 {
+        mount_writable(&parameters, &mut devices).expect("树表 0 条的池可写挂载");
+    }
+    let image = pool_of(&devices);
+    let (swept, _devices_of_the_swept_mount, swept_stream) =
+        sweeping_mount_on(&image, sweeping_at_the_end_of_the_mount_on_any_row());
+    assert!(
+        matches!(
+            swept.output.instance_table_sweep,
+            InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::NoRowDeleted(
+                NoRowDeletedThisRound::SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion
+            ))
+        ),
+        "树表 0 条的一版上不清扫：{:?}",
+        swept.output.instance_table_sweep
+    );
+    let (_not_swept, _devices_of_the_mount_not_swept, not_swept_stream) =
+        sweeping_mount_on(&image, InstanceTableSweepConfiguration::PRODUCT);
+    assert!(
+        image_after(&image, &swept_stream) == image_after(&image, &not_swept_stream),
+        "清扫那一判一个字节都没写"
+    );
+    assert_eq!(
+        swept_stream.operations().len(),
+        not_swept_stream.operations().len(),
+        "录制流一样长"
+    );
+}
diff --git a/crates/singlefs-harness/tests/mount_transparency_judges_the_system_configuration_witness_by_r2.rs b/crates/singlefs-harness/tests/mount_transparency_judges_the_system_configuration_witness_by_r2.rs
new file mode 100644
index 0000000..5c463b2
--- /dev/null
+++ b/crates/singlefs-harness/tests/mount_transparency_judges_the_system_configuration_witness_by_r2.rs
@@ -0,0 +1,163 @@
+//! harness 判法复算判据 N-配置 时，系统配置见证判不出改取 R2（D22（单元原子性怎么合成） 已定项 8 射程，用户 2026-10-03 定「改取 R2」；
+//! D23（journal 的角色与格式） 已定项 14 那一段；代码轮判决 `research/prompts/m3-root-ring-code-r1-main-verification.md` 第二节 RR8）：
+//! 判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过。撕裂的轮换（读得出、自证不过、设备没报错）不算判不出，c_见证 跳过撕裂的槽照常比；
+//! 某块盘两槽一份自证过的都没有（C554 乙-配置续 Q1，这块盘不「可见」）照实现那一个成员按真。
+//! 小池是单元区 384 槽的两块内存盘，第一个文件之后的盘面（mkfs → 取号 → 暖机 → 第一个文件）。
+//! 判的拒都是用例自己拼的一份「判据 N-配置 为真、重读仍为真」：判法自己在崩溃镜像上复算判据，不看拒里填的读数。
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::mount::{
+    MountError, NewerPublishWitness, RollbackTarget, SelectedVersionAgainstTheWitness,
+    StillUnreadableAfterOneReread, WitnessedCounterComparison,
+};
+use singlefs_core::recovery::verified_system_configuration_slots;
+use singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE;
+use singlefs_harness::history::{
+    execute_history_with, GeneratedHistory, HistoryDeviceWidth, HistoryExecution, HistorySeed,
+    HistoryStartingPoint, PerStepChecker,
+};
+use singlefs_harness::memory_pool::MemoryPool;
+use singlefs_harness::mount_transparency::{
+    judge_writable_mount_refusal, read_recovery_fields, ComparedVersion, DiskAfterTheRefusal,
+    ReadOnlyReadBackOfTheCrashImage, RefusalConditionNotMet, RefusalReasonOnTheD23List,
+    WritableMountRefusalVerdict,
+};
+use singlefs_harness::SharedStream;
+
+const SMALL: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
+const DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
+/// 第一个文件那次系统配置轮换写的世代号：每块盘世代 5 那一槽（槽号 5 mod 2 = 1）。
+const GENERATION_OF_THE_FIRST_FILE_ROTATION: u64 = 5;
+/// 撕裂一个系统配置槽时翻的那个字节在槽里的下标：槽中间，落在整槽校验和罩着的范围里。
+const TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT: u64 = 2048;
+
+fn pool_after_the_first_file() -> MemoryPool {
+    let stream = SharedStream::retaining_contents();
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations: Vec::new(),
+    };
+    execute_history_with(
+        &history,
+        HistoryExecution {
+            per_step_checker: PerStepChecker::Skipped,
+            device_width: SMALL,
+            space_admission: SpaceAdmission::JudgedByTheFormula,
+        },
+        &stream,
+        &mut |_observation| {},
+    );
+    let mut pool = MemoryPool::with_devices(&DEVICES, SMALL.device_bytes());
+    pool.apply(&stream.retained_operations());
+    pool
+}
+
+fn offset_of_system_configuration_slot(slot_index: u64) -> DeviceOffsetInBytes {
+    DeviceOffsetInBytes(
+        slot_index * u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing),
+    )
+}
+
+/// 判据 N-配置 为真、重读一次仍为真的一拒（判法自己复算判据，不看这里填的读数）。
+fn witness_refusal_claimed_on(selected_version: (InstanceGeneration, CheckpointTxg)) -> MountError {
+    let reading = SelectedVersionAgainstTheWitness {
+        selected_version: RollbackTarget {
+            instance: selected_version.0,
+            checkpoint_txg: selected_version.1,
+        },
+        witness: NewerPublishWitness {
+            witnessed_journal_counter: 1,
+            comparison: WitnessedCounterComparison::Undecidable,
+        },
+    };
+    MountError::NewerStateStillUnreadableAfterOneReread(Box::new(
+        StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
+            first_read: reading,
+            reread: reading,
+        },
+    ))
+}
+
+/// 在崩溃镜像 `image` 上判那一拒：所选那一版取只读恢复落到的那一版，盘上逐字节不变、只读读出在允许集合里（U6 另两条都满足）。
+fn verdict_of_the_claimed_witness_refusal_on(image: &MemoryPool) -> WritableMountRefusalVerdict {
+    let fields = read_recovery_fields(image, ComparedVersion::TheRecoveredVersion);
+    let selected_version = fields
+        .report
+        .effective_root
+        .expect("撕裂的系统配置槽上只读恢复照样落到第一个文件那一版");
+    judge_writable_mount_refusal(
+        image,
+        fields.report.effective_root,
+        &witness_refusal_claimed_on(selected_version),
+        DiskAfterTheRefusal::ByteForByteUnchanged,
+        ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
+    )
+}
+
+/// 场景：两块盘第一个文件那次轮换写的那一槽（世代 5，槽 1）都撕裂——读得出、自证不过，设备不报错。
+/// 预期：R2 下这不是判不出，c_见证 取两块盘世代 4 那一槽、不比第一个文件那条记录的计数器大，判据复算为假：拿它当理由拒可写是拒错了，判非法。
+/// 取甲（哪块盘自证过的槽少于两份就按真）时这一拒判成合法。
+#[test]
+fn a_witness_refusal_on_a_torn_system_configuration_rotation_is_a_refusal_for_the_wrong_reason() {
+    let mut image = pool_after_the_first_file();
+    let slot_of_the_first_file_rotation = offset_of_system_configuration_slot(
+        GENERATION_OF_THE_FIRST_FILE_ROTATION % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
+    );
+    for device in DEVICES {
+        image.flip_byte(
+            device,
+            slot_of_the_first_file_rotation,
+            TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT,
+        );
+    }
+    assert_eq!(
+        verdict_of_the_claimed_witness_refusal_on(&image),
+        WritableMountRefusalVerdict::Illegal {
+            reason: RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
+            conditions_not_met: vec![
+                RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage
+            ],
+        },
+        "撕裂的轮换不是判不出：判据复算为假"
+    );
+}
+
+/// 场景：盘 0 两个系统配置槽都撕裂（一份本池自证过的都没有），盘 1 两槽都自证过；设备不报错。
+/// 预期：R2 判得出（没有槽报读错），但盘 0 不「可见」（C554 乙-配置续 Q1）：实现单列一个成员报成判据 N-配置 的拒因、按真处置，
+/// 判法照同一个处置复算为真，这一拒合法。
+#[test]
+fn a_witness_refusal_on_a_device_without_any_self_verified_system_configuration_slot_is_legal() {
+    let mut image = pool_after_the_first_file();
+    for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
+        image.flip_byte(
+            DEVICES[0],
+            offset_of_system_configuration_slot(slot_index),
+            TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT,
+        );
+    }
+    let slot_spacing_in_bytes = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
+    let filesystem_identifier = SMALL.parameters().filesystem_identifier;
+    assert_eq!(
+        DEVICES.map(|device| {
+            verified_system_configuration_slots(
+                &image,
+                device,
+                slot_spacing_in_bytes,
+                &filesystem_identifier,
+            )
+            .len()
+        }),
+        [0, 2],
+        "前提：盘 0 一份自证过的系统配置都没有，盘 1 两槽都自证过"
+    );
+    assert_eq!(
+        verdict_of_the_claimed_witness_refusal_on(&image),
+        WritableMountRefusalVerdict::Legal(
+            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration
+        ),
+        "一块盘一份自证过的系统配置都没有：判据按真"
+    );
+}
diff --git a/crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs b/crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs
new file mode 100644
index 0000000..79b76d1
--- /dev/null
+++ b/crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs
@@ -0,0 +1,227 @@
+//! 准入抬 F 抬到上限时，窗口里挂载那一刻读不出、之后这个进程没写过的槽当作可能是一条非空有效根，F 不越过它
+//! （D16（发布语义） 已定项 1「抬 F 的上限」与「根槽这一次读坏」两行，用户 2026-10-04 定；代码轮第二轮判决
+//! `research/prompts/m3-root-ring-code-r2-main-verification.md` 第四节 RS3 与第六节；正推腿探针
+//! `research/prompts/m3-root-ring-code-r2-sonnet-model/tests/probe_k1_raise_to_the_ceiling_over_an_unreadable_x.rs` 那一形）。
+//!
+//! 盘面（S = 4 的 12 槽环）：实例 1 覆盖写到 txg 17；三次可写挂载崩在写行那次根槽写之前（洞 18、19、20：那几槽里留着上一圈的根，出了窗口）；
+//! 再一次可写挂载写下 X = 21（写行那一版的根）再空发布两次，正常退出；第四次可写挂载之后空发布推到锚 29，准入抬 F 抬到上限。
+//! 窗口 txg > 17 里读得出的有效根是 22–29 的空发布：X 读不出时最旧的那条 22 比不出前一条、算非空，上限按读得出的根取 22；
+//! X 读得出时 21 是非空、22 起都空，上限取 21。
+//! 抬完之后判的几条不变量不含 I-7.9：池级 checker 的 I-7.9 还按全环算上限（第 9 批池级 checker 簇在改），两种盘面上今天都红。
+mod common;
+mod common_root_ring_holes;
+
+use common::{
+    with_unreadable_ranges, DeviceWithUnreadableRanges, FailingReadsOfARange,
+    SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
+};
+use common_root_ring_holes::{
+    crash_a_mount_before_its_row_root, devices_of, parameters_of_the_ring_hole_pools, pool_of,
+    pool_overwritten_through, publish_empty, readable_root_at_txg, recording_of,
+    SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS,
+};
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity};
+use singlefs_core::allocator::PoolAllocator;
+use singlefs_core::mount::{
+    mount_writable, raise_rollback_floor_to_the_admission_ceiling, RaiseToTheAdmissionCeiling,
+    ShadowLedger,
+};
+use singlefs_core::root_ring::{slot_offset, target_for_publish};
+use singlefs_core::transaction::{PoolWriter, TransactionOutput};
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+use singlefs_harness::{RecordingBlockDevice, SharedStream};
+
+/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
+const INVARIANTS_THE_RAISE_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];
+/// 实例 1 覆盖写到的 txg。
+const LAST_TXG_OF_INSTANCE_ONE: u64 = 17;
+/// 崩在写行根槽之前的挂载次数（洞 18、19、20）。
+const HOLES: usize = 3;
+/// X 那一版之后、正常退出之前的空发布次数。
+const EMPTY_PUBLISHES_AFTER_X: u64 = 2;
+/// X：洞之后那一次可写挂载写行那一版的 txg。
+const X_TXG: u64 = 21;
+/// X 读得出时准入抬 F 的上限（探针 `PROBEREADABLE_RAISED … ceiling=CheckpointTxg(21)`）。
+const CEILING_WHEN_X_READS: CheckpointTxg = CheckpointTxg(21);
+
+type DevicesOfTheFourthMount = Vec<(
+    DeviceIdentity,
+    DeviceWithUnreadableRanges<RecordingBlockDevice<SparseBlockDevice>>,
+)>;
+
+/// 准入抬 F 之前的那一刻：第四次可写挂载的盘（X 那一槽按 `x_slot_reads` 读坏）、录下的流与它的起点盘面、分配器、现行那一版。
+struct PoolJustBeforeTheAdmissionRaise {
+    holes: Vec<u64>,
+    x_txg: u64,
+    image_before_the_fourth_mount: MemoryPool,
+    stream: SharedStream,
+    devices: DevicesOfTheFourthMount,
+    allocator: PoolAllocator,
+    current: TransactionOutput,
+}
+
+impl PoolJustBeforeTheAdmissionRaise {
+    /// 盘上此刻的样子：起点盘面加上第四次挂载以来录下的全部写（读故障只在读那一侧）。
+    fn image_now(&self) -> MemoryPool {
+        let mut image = self.image_before_the_fourth_mount.clone();
+        image.apply(&self.stream.retained_operations());
+        image
+    }
+}
+
+fn violations_of(image: &MemoryPool) -> Vec<String> {
+    check_pool_image(image)
+        .into_iter()
+        .filter(|(invariant, _)| INVARIANTS_THE_RAISE_KEEPS_GREEN.contains(invariant))
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
+            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+        })
+        .collect()
+}
+
+/// 造盘面到准入抬 F 之前（与 `raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs` 同一个做法另写一份）。
+fn pool_just_before_the_admission_raise(
+    x_slot_reads: FailingReadsOfARange,
+) -> PoolJustBeforeTheAdmissionRaise {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let slots_per_region = parameters.geometry.root_ring_slots_per_region;
+    let ring_length = 3 * SLOTS_PER_REGION_OF_THE_RING_HOLE_POOLS;
+    let mut image = pool_overwritten_through(LAST_TXG_OF_INSTANCE_ONE);
+    let mut holes = Vec::new();
+    // 迭代上界：`HOLES` 次挂载，每次一个洞；跨轮携带的是崩溃之后的盘面。
+    for _ in 0..HOLES {
+        let (crashed, row_txg) = crash_a_mount_before_its_row_root(&image);
+        holes.push(row_txg);
+        image = crashed;
+    }
+    let x_txg = {
+        let mut devices = devices_of(&image);
+        let mounted = mount_writable(&parameters, &mut devices).expect("洞之后那一次可写挂载");
+        let x_txg = mounted.output.row_publish.root().checkpoint_txg.0;
+        let instance = mounted.output.instance;
+        let mut allocator = mounted.allocator.clone();
+        let mut current = mounted.current.into_file_version().expect("这一版带文件");
+        // 迭代上界：两次空发布。
+        for _ in 0..EMPTY_PUBLISHES_AFTER_X {
+            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+            current =
+                publish_empty(&mut writer, &mut allocator, &current, instance).expect("空发布");
+        }
+        image = pool_of(&devices);
+        x_txg
+    };
+    assert!(
+        readable_root_at_txg(&image, x_txg).is_some(),
+        "X 那一版的根落盘了"
+    );
+    let x_slot = target_for_publish(CheckpointTxg(x_txg), slots_per_region);
+    let unreadable_x_slot = SharedUnreadableRanges::new(
+        vec![UnreadableRange {
+            device: parameters.region_devices[usize::try_from(x_slot.region).expect("区域号")],
+            offset_in_bytes: slot_offset(x_slot, parameters.geometry.fixed_structure_slot_spacing)
+                .0,
+            length_in_bytes: u64::from(parameters.geometry.physical_block_size),
+            failing_reads: x_slot_reads,
+        }],
+        UnreadableRangeReadBack::DeviceError,
+    );
+    let stream = SharedStream::retaining_contents();
+    let mut devices = with_unreadable_ranges(recording_of(&image, &stream), &unreadable_x_slot);
+    let mounted = mount_writable(&parameters, &mut devices).expect("第四次可写挂载");
+    let instance = mounted.output.instance;
+    let mut allocator = mounted.allocator.clone();
+    let mut current = mounted.current.into_file_version().expect("这一版带文件");
+    let anchor = holes[0] + ring_length - 1;
+    // 迭代上界：txg 逐次加一推到锚。
+    while current.root.checkpoint_txg.0 < anchor {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        current = publish_empty(&mut writer, &mut allocator, &current, instance).expect("空发布");
+    }
+    PoolJustBeforeTheAdmissionRaise {
+        holes,
+        x_txg,
+        image_before_the_fourth_mount: image,
+        stream,
+        devices,
+        allocator,
+        current,
+    }
+}
+
+/// 抬到上限，交回抬到的上限与抬完之后现行那一版带的 F。
+fn raise_to_the_admission_ceiling(
+    pool: &mut PoolJustBeforeTheAdmissionRaise,
+) -> (CheckpointTxg, CheckpointTxg) {
+    let raised = raise_rollback_floor_to_the_admission_ceiling(
+        &parameters_of_the_ring_hole_pools(),
+        &mut pool.devices,
+        &mut pool.allocator,
+        &mut pool.current,
+        ShadowLedger::On,
+    );
+    let ceiling = match raised {
+        Ok(RaiseToTheAdmissionCeiling::Raised(raised_floor)) => raised_floor.ceiling,
+        Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling }) => {
+            panic!("前提：F = {floor:?} 低于上限 {ceiling:?}，要抬")
+        }
+        Err(error) => panic!("准入抬 F 做成：{error:?}"),
+    };
+    (ceiling, pool.current.root.rollback_floor)
+}
+
+/// 场景：X 那一槽第四次挂载起一直读不出（根环表按它能住的最大 txg 21 记着），推到锚 29 之后准入抬 F 抬到上限。
+/// 预期：上限不越过 X：取 21（按读得出的根算是 22），F 抬到 21，抬完那几条不变量不红。
+#[test]
+fn admission_raising_the_floor_to_the_ceiling_stops_at_a_slot_unreadable_since_the_mount() {
+    let mut pool = pool_just_before_the_admission_raise(FailingReadsOfARange::Every);
+    assert_eq!(
+        (pool.holes.clone(), pool.x_txg),
+        (vec![18, 19, 20], X_TXG),
+        "三个洞紧跟着 X"
+    );
+    let x_slot = target_for_publish(
+        CheckpointTxg(X_TXG),
+        parameters_of_the_ring_hole_pools()
+            .geometry
+            .root_ring_slots_per_region,
+    );
+    assert_eq!(
+        pool.allocator
+            .root_ring_occupancy()
+            .expect("可写挂载装了根环表")
+            .ring_slots_unreadable_at_mount()
+            .get(&x_slot),
+        Some(&CheckpointTxg(X_TXG)),
+        "前提：根环表把 X 那一槽记成挂载时读不出、能住的最大 txg 21"
+    );
+    assert_eq!(
+        raise_to_the_admission_ceiling(&mut pool),
+        (CheckpointTxg(X_TXG), CheckpointTxg(X_TXG)),
+        "上限与抬到的 F 都不越过 X 那一槽能住的 txg 21"
+    );
+    assert_eq!(
+        violations_of(&pool.image_now()),
+        Vec::<String>::new(),
+        "抬完之后"
+    );
+}
+
+/// 场景：同一盘面，X 那一槽一直读得出。
+/// 预期：上限照读出来的根取 21（X 非空、22 起都空），F 抬到 21，抬完那几条不变量不红。
+#[test]
+fn admission_raising_the_floor_to_the_ceiling_with_x_readable_reaches_x() {
+    let mut pool = pool_just_before_the_admission_raise(FailingReadsOfARange::Never);
+    assert_eq!(
+        raise_to_the_admission_ceiling(&mut pool),
+        (CEILING_WHEN_X_READS, CEILING_WHEN_X_READS),
+        "X 读得出时上限是 21"
+    );
+    assert_eq!(
+        violations_of(&pool.image_now()),
+        Vec::<String>::new(),
+        "抬完之后"
+    );
+}
diff --git a/crates/singlefs-harness/tests/root_ring_slot_reread_that_does_not_self_verify_counts_as_still_unreadable.rs b/crates/singlefs-harness/tests/root_ring_slot_reread_that_does_not_self_verify_counts_as_still_unreadable.rs
new file mode 100644
index 0000000..26cf817
--- /dev/null
+++ b/crates/singlefs-harness/tests/root_ring_slot_reread_that_does_not_self_verify_counts_as_still_unreadable.rs
@@ -0,0 +1,866 @@
+//! 进程内记根时重读挂载那一刻读不出的根环槽（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」那一行的 E2 句与「根槽这一次读坏」那一行）：
+//! 重读那一次的几种结局各记成什么、判「按实例表有效」用哪张表、产品路径上记根的几处各自走到重读。
+//!
+//! - 读得出而自证不过，与仍读不出同一处置，按根环表记的最大 txg 算（用户 2026-10-04 定改法 A，代码轮第二轮判决
+//!   `research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1；攻方 `research/prompts/m3-root-ring-code-r2-opus-model/proto/src/bin/e2_reread_flip.rs`
+//!   那一形：一条有效根所在的槽挂载那一刻读错、它成了窗口里最小的那一刻那一次重读读坏，改之前回收门槛越过它、I-3.1 记账少）；
+//!   代价那一形（同一目录 `front_hole_eio.rs`：从没写过的槽加挂载那一刻读错）判成什么就钉什么。
+//! - 读得出、自证过、按实例表有效的照读出来的 txg 算，那一槽从此算「这个进程知道住着根的槽」（用户 2026-10-04 定）；
+//!   读得出、自证过而按实例表被抛弃的当没有根；判有效用的表写行之后是写行那一张（同一份判决第二节 RS2，正推腿探针 p1、p2）。
+//! - 记根的产品路径：带单元的发布（`transaction::publish_admitted`，已有变异行管它）、树表 0 条那一版上的暖机零单元发布
+//!   （`mount::publish_empty_after`）走到重读（探针 q1）。另两处没有会红的用例：第一次可写挂载的零单元写行发布
+//!   （`mount::establish_instance` 里 `rows_written` 为空那一支）走不到重读——那一刻根环里只有 mkfs 写的 txg 0，挂载时读不出的槽只可能是它，
+//!   而它读不出时择不到根、挂载不成；树表 0 条那一版上要写行的写行发布（`transaction::prepare_the_row_publish_on_a_version_without_file`，探针 q3）
+//!   重读的那一槽恰是紧跟着的暖机要盖掉的那一槽（写行 txg + 1），重读改的根环表与回收在暖机那一次被盖成同一个样子，那一版又没有记账树，
+//!   挂载交回之后看不出它重读没重读（交回报告里列着）。
+mod common;
+mod common_root_ring_holes;
+
+use std::cell::RefCell;
+use std::collections::BTreeSet;
+use std::rc::Rc;
+
+use common::{
+    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
+    overwrite_in_process_as_instance_one_and_advance_the_pool,
+    BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER, FIXED_WRITE_TIME_SECONDS,
+};
+use common_root_ring_holes::{
+    content_of_the_version, devices_of, fixed_geometry_of_the_ring_hole_pools,
+    parameters_of_the_ring_hole_pools, pool_of, pool_overwritten_through, recording_of,
+    RingHoleDevices, DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
+    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
+};
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+};
+use singlefs_core::allocator::PoolAllocator;
+use singlefs_core::block_device::{
+    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
+};
+use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
+use singlefs_core::mount::{
+    mount_writable, raise_rollback_floor_to_the_admission_ceiling, MountError, ShadowLedger,
+};
+use singlefs_core::recovery::instance_table_chain_of_root;
+use singlefs_core::root_record::RootRecord;
+use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
+use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
+use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::{RecordingBlockDevice, SharedStream};
+
+/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
+const INVARIANTS_THE_REREAD_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];
+/// 攻方那一形：实例 1 覆盖写到 txg 20，有效根 h = 18 所在的槽（区域 0、槽 2）挂载那一刻读错。
+const LAST_TXG_OF_INSTANCE_ONE: u64 = 20;
+const TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT: u64 = 18;
+/// h + 3S − 1：那一槽成了窗口里最小的那个、记这条根时重读它的那一版（S = 4）。
+const TXG_AT_WHICH_THE_SLOT_IS_REREAD: u64 = 29;
+/// 前洞那一形：实例 2 的 txg 9 那次覆盖写崩在根槽写之前（环还没转过来，那一槽从没写过）。
+const TXG_OF_THE_FRONT_HOLE: u64 = 9;
+/// 前洞那一形里那一槽成了窗口里最小的那个的那一版：9 + 3S − 1。
+const TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD: u64 = 20;
+
+/// 被抛弃根那一形判的几条：同上去掉 I-7.4。被抛弃的根所在的槽挂载那一刻读不出，影子账罩不到它引用的单元（挂载那一刻就读不到那条根，
+/// 与重读无关：攻方 `research/prompts/m3-root-ring-code-r2-opus-output.md` K1 第 3 小节「被抛弃」那一条），之后那几个单元被合法复用，
+/// I-7.4 在这一形上红是今天已知的那一格，不归这条用例判。
+const INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT: [&str; 4] = ["I-3.1", "I-3.11", "I-2.1", "I-4.8"];
+
+fn violations_of(image: &MemoryPool) -> Vec<String> {
+    violations_among(image, &INVARIANTS_THE_REREAD_KEEPS_GREEN)
+}
+
+fn violations_among(image: &MemoryPool, invariants_judged: &[&str]) -> Vec<String> {
+    check_pool_image(image)
+        .into_iter()
+        .filter(|(invariant, _)| invariants_judged.contains(invariant))
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
+            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+        })
+        .collect()
+}
+
+/// 一个根环槽的读这会儿怎么坏。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum ReadsOfTheRootRingSlot {
+    PassedThrough,
+    DeviceErrorOnEveryRead,
+    /// 任何一块盘上发出第一个写（取号写系统配置）之前每次读都报设备错，之后照读：只在可写挂载读阶段读错。
+    DeviceErrorUntilTheFirstWriteOfAnyDevice,
+    /// 只有下一次读坏：读得出，那一槽里一个字节取反（自证不过，盘上字节不变）；之后照读。
+    OneByteFlippedOnTheNextReadOnly,
+}
+
+struct FaultOfOneRootRingSlotState {
+    device: DeviceIdentity,
+    offset_in_bytes: u64,
+    length_in_bytes: u64,
+    reads: ReadsOfTheRootRingSlot,
+    a_write_was_sent: bool,
+    failed_reads: u64,
+    flipped_reads: u64,
+}
+
+/// 几块盘共用的、一个根环槽的读故障。
+#[derive(Clone)]
+struct FaultOfOneRootRingSlot(Rc<RefCell<FaultOfOneRootRingSlotState>>);
+
+enum ReadOutcome {
+    Read,
+    DeviceError,
+    FlipTheByteAt(usize),
+}
+
+impl FaultOfOneRootRingSlot {
+    fn of_the_root_slot_of_txg(
+        parameters: &MakeFilesystemParameters,
+        checkpoint_txg: CheckpointTxg,
+        reads: ReadsOfTheRootRingSlot,
+    ) -> Self {
+        let ring_slot = target_for_publish(
+            checkpoint_txg,
+            parameters.geometry.root_ring_slots_per_region,
+        );
+        Self(Rc::new(RefCell::new(FaultOfOneRootRingSlotState {
+            device: parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")],
+            offset_in_bytes: slot_offset(
+                ring_slot,
+                parameters.geometry.fixed_structure_slot_spacing,
+            )
+            .0,
+            length_in_bytes: u64::from(parameters.geometry.physical_block_size),
+            reads,
+            a_write_was_sent: false,
+            failed_reads: 0,
+            flipped_reads: 0,
+        })))
+    }
+
+    fn set_reads(&self, reads: ReadsOfTheRootRingSlot) {
+        self.0.borrow_mut().reads = reads;
+    }
+
+    fn failed_reads(&self) -> u64 {
+        self.0.borrow().failed_reads
+    }
+
+    fn flipped_reads(&self) -> u64 {
+        self.0.borrow().flipped_reads
+    }
+
+    fn note_a_write(&self) {
+        self.0.borrow_mut().a_write_was_sent = true;
+    }
+
+    fn note_a_read(
+        &self,
+        device: DeviceIdentity,
+        offset_in_bytes: u64,
+        length: u64,
+    ) -> ReadOutcome {
+        let mut state = self.0.borrow_mut();
+        let overlaps = state.device == device
+            && offset_in_bytes < state.offset_in_bytes + state.length_in_bytes
+            && state.offset_in_bytes < offset_in_bytes + length;
+        if !overlaps {
+            return ReadOutcome::Read;
+        }
+        match state.reads {
+            ReadsOfTheRootRingSlot::PassedThrough => ReadOutcome::Read,
+            ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead => {
+                state.failed_reads += 1;
+                ReadOutcome::DeviceError
+            }
+            ReadsOfTheRootRingSlot::DeviceErrorUntilTheFirstWriteOfAnyDevice => {
+                if state.a_write_was_sent {
+                    ReadOutcome::Read
+                } else {
+                    state.failed_reads += 1;
+                    ReadOutcome::DeviceError
+                }
+            }
+            ReadsOfTheRootRingSlot::OneByteFlippedOnTheNextReadOnly => {
+                let byte_in_the_slot =
+                    state.offset_in_bytes + BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER;
+                if byte_in_the_slot < offset_in_bytes
+                    || byte_in_the_slot >= offset_in_bytes + length
+                {
+                    return ReadOutcome::Read;
+                }
+                state.flipped_reads += 1;
+                state.reads = ReadsOfTheRootRingSlot::PassedThrough;
+                ReadOutcome::FlipTheByteAt(
+                    usize::try_from(byte_in_the_slot - offset_in_bytes).expect("落在缓冲区里"),
+                )
+            }
+        }
+    }
+}
+
+/// 读的时候按 [`FaultOfOneRootRingSlot`] 读坏一个根环槽的盘；写、写零、屏障原样交给里面那块盘（写记一笔：取号之后不再报读错的那一档要它）。
+struct DeviceWithOneRootRingSlotFaulted<Inner: BlockDevice> {
+    identity: DeviceIdentity,
+    inner: Inner,
+    fault: FaultOfOneRootRingSlot,
+}
+
+impl<Inner: BlockDevice> BlockDevice for DeviceWithOneRootRingSlotFaulted<Inner> {
+    fn read_at(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        let length = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
+        match self.fault.note_a_read(self.identity, offset.0, length) {
+            ReadOutcome::Read => self.inner.read_at(offset, buffer),
+            ReadOutcome::DeviceError => Err(BlockDeviceError::InputOutput(std::io::Error::other(
+                "用例点名的根环槽读坏",
+            ))),
+            ReadOutcome::FlipTheByteAt(index) => {
+                self.inner.read_at(offset, buffer)?;
+                buffer[index] = !buffer[index];
+                Ok(())
+            }
+        }
+    }
+
+    fn write_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+        durability: WriteDurability,
+    ) -> Result<(), BlockDeviceError> {
+        self.fault.note_a_write();
+        self.inner.write_at(offset, bytes, durability)
+    }
+
+    fn write_zeroes_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        self.fault.note_a_write();
+        self.inner.write_zeroes_at(offset, length)
+    }
+
+    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
+        self.inner.barrier()
+    }
+
+    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
+        self.inner.probe_physical_block_size()
+    }
+
+    fn size_in_bytes(&self) -> u64 {
+        self.inner.size_in_bytes()
+    }
+}
+
+fn with_the_fault<Inner: BlockDevice>(
+    devices: Vec<(DeviceIdentity, Inner)>,
+    fault: &FaultOfOneRootRingSlot,
+) -> Vec<(DeviceIdentity, DeviceWithOneRootRingSlotFaulted<Inner>)> {
+    devices
+        .into_iter()
+        .map(|(identity, inner)| {
+            (
+                identity,
+                DeviceWithOneRootRingSlotFaulted {
+                    identity,
+                    inner,
+                    fault: fault.clone(),
+                },
+            )
+        })
+        .collect()
+}
+
+type RingHoleDevicesWithTheFault = Vec<(
+    DeviceIdentity,
+    DeviceWithOneRootRingSlotFaulted<RecordingBlockDevice<SparseBlockDevice>>,
+)>;
+
+/// S = 4 的小池上一次可写挂载之后的会话：盘（一个根环槽带读故障）、录下的流与它的起点盘面、分配器、现行那一版。
+struct SessionOnARingHolePool {
+    image_before_the_mount: MemoryPool,
+    stream: SharedStream,
+    devices: RingHoleDevicesWithTheFault,
+    fault: FaultOfOneRootRingSlot,
+    ring_slot: RootRingSlot,
+    allocator: PoolAllocator,
+    current: TransactionOutput,
+    instance: InstanceGeneration,
+}
+
+impl SessionOnARingHolePool {
+    /// `image` 上可写挂载，txg 为 `txg_of_the_slot` 的那一槽挂载那一刻每次读都报设备错，挂载之后照读。
+    fn mounted_with_the_slot_of_txg_unreadable_at_mount(
+        image: MemoryPool,
+        txg_of_the_slot: u64,
+    ) -> Self {
+        let parameters = parameters_of_the_ring_hole_pools();
+        let fault = FaultOfOneRootRingSlot::of_the_root_slot_of_txg(
+            &parameters,
+            CheckpointTxg(txg_of_the_slot),
+            ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead,
+        );
+        let stream = SharedStream::retaining_contents();
+        let mut devices = with_the_fault(recording_of(&image, &stream), &fault);
+        let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
+        fault.set_reads(ReadsOfTheRootRingSlot::PassedThrough);
+        assert!(fault.failed_reads() > 0, "挂载那一次读那一槽读错了");
+        let ring_slot = target_for_publish(
+            CheckpointTxg(txg_of_the_slot),
+            parameters.geometry.root_ring_slots_per_region,
+        );
+        Self {
+            image_before_the_mount: image,
+            stream,
+            devices,
+            fault,
+            ring_slot,
+            instance: mounted.output.instance,
+            allocator: mounted.allocator,
+            current: mounted.current.into_file_version().expect("这一版带文件"),
+        }
+    }
+
+    fn image_now(&self) -> MemoryPool {
+        let mut image = self.image_before_the_mount.clone();
+        image.apply(&self.stream.retained_operations());
+        image
+    }
+
+    fn overwrite_once(&mut self) {
+        let parameters = parameters_of_the_ring_hole_pools();
+        let salt = self.current.root.checkpoint_txg.0 + 1;
+        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
+        self.current = publish_overwrite(
+            &mut writer,
+            &mut self.allocator,
+            &self.current,
+            FirstFile {
+                content: &content_of_the_version(salt),
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
+            },
+            self.instance,
+        )
+        .expect("覆盖写");
+    }
+
+    /// 根环表里这一槽按挂载时读不出记着的最大 txg（不再这样记着时 `None`）。
+    fn txg_recorded_as_unreadable_at_mount(&self) -> Option<CheckpointTxg> {
+        self.allocator
+            .root_ring_occupancy()
+            .expect("可写挂载装了根环表")
+            .ring_slots_unreadable_at_mount()
+            .get(&self.ring_slot)
+            .copied()
+    }
+
+    fn slot_is_known_to_hold_a_root(&self) -> bool {
+        self.allocator
+            .root_ring_occupancy()
+            .expect("可写挂载装了根环表")
+            .ring_slots_known_to_hold_a_root()
+            .contains(&self.ring_slot)
+    }
+}
+
+/// 攻方那一形推到重读的前一版：实例 1 覆盖写到 20；实例 2 可写挂载那一刻读 h = 18 那一槽报读错；之后覆盖写到 28，
+/// 每一版之后那几条不变量都不红。
+fn session_one_publish_before_the_reread() -> SessionOnARingHolePool {
+    let mut session = SessionOnARingHolePool::mounted_with_the_slot_of_txg_unreadable_at_mount(
+        pool_overwritten_through(LAST_TXG_OF_INSTANCE_ONE),
+        TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT,
+    );
+    assert_eq!(
+        session.txg_recorded_as_unreadable_at_mount(),
+        Some(CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)),
+        "前提：根环表把那一槽记成挂载时读不出、能住的最大 txg 18"
+    );
+    // 迭代上界：覆盖写到 txg 28；跨轮携带的是现行那一版。
+    while session.current.root.checkpoint_txg.0 < TXG_AT_WHICH_THE_SLOT_IS_REREAD - 1 {
+        session.overwrite_once();
+        assert_eq!(
+            violations_of(&session.image_now()),
+            Vec::<String>::new(),
+            "txg {} 之后",
+            session.current.root.checkpoint_txg.0
+        );
+    }
+    assert_eq!(
+        session.txg_recorded_as_unreadable_at_mount(),
+        Some(CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)),
+        "前提：txg 28 时窗口里还有比 18 旧的 17，那一槽没重读"
+    );
+    session
+}
+
+/// 场景（攻方 `e2_reread_flip 4 20 2 flip`，`FLIP_ONLY_AT_ANCHOR=1`）：发 txg 29 那一次覆盖写时只有一次读把那一槽读坏（一个字节取反，盘上不变），
+/// 那一次就是记这条根之前的重读。
+/// 预期：读得出而自证不过与仍读不出同一处置，那一槽照旧按 18 记着，回收门槛停在 18；txg 29 那一版 I-3.1 成立（改之前记账少 12 个槽）。
+#[test]
+fn a_valid_root_read_bad_once_on_the_reread_is_counted_at_the_highest_txg_its_slot_can_hold() {
+    let mut session = session_one_publish_before_the_reread();
+    session
+        .fault
+        .set_reads(ReadsOfTheRootRingSlot::OneByteFlippedOnTheNextReadOnly);
+    session.overwrite_once();
+    assert_eq!(
+        session.current.root.checkpoint_txg,
+        CheckpointTxg(TXG_AT_WHICH_THE_SLOT_IS_REREAD)
+    );
+    assert_eq!(
+        session.fault.flipped_reads(),
+        1,
+        "发 txg 29 那一次读那一槽、读坏了一次"
+    );
+    assert_eq!(
+        session.txg_recorded_as_unreadable_at_mount(),
+        Some(CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)),
+        "重读读到自证不过：照旧按能住的最大 txg 18 记着"
+    );
+    assert_eq!(
+        violations_of(&session.image_now()),
+        Vec::<String>::new(),
+        "txg 29 那一版：回收门槛没越过盘上 txg 18 那条有效根"
+    );
+}
+
+/// 场景：同一盘面，发 txg 29 那一次重读读得出（读故障只在挂载那一刻）。
+/// 预期：读出 txg 18 的根、按实例表有效、在窗口里：那一槽换成「重读读出的有效根 18」，算这个进程知道住着根的槽；txg 29 那一版 I-3.1 成立。
+#[test]
+fn a_valid_root_read_on_the_reread_inside_the_window_holds_the_reclaim_threshold_at_its_txg() {
+    let mut session = session_one_publish_before_the_reread();
+    session.overwrite_once();
+    assert_eq!(
+        session.current.root.checkpoint_txg,
+        CheckpointTxg(TXG_AT_WHICH_THE_SLOT_IS_REREAD)
+    );
+    assert_eq!(
+        (
+            session.txg_recorded_as_unreadable_at_mount(),
+            session.slot_is_known_to_hold_a_root()
+        ),
+        (None, true),
+        "重读读出有效根：不再按挂载时读不出记，那一槽算这个进程知道住着根的槽"
+    );
+    assert_eq!(
+        violations_of(&session.image_now()),
+        Vec::<String>::new(),
+        "txg 29 那一版：门槛照读出来的 18 算"
+    );
+}
+
+/// 场景：同一盘面，发 txg 29 那一次重读读出 txg 18 的有效根；之后那一槽每次读都报设备错，接着准入抬 F 抬到上限。
+/// 预期：算上限那一遍那一槽读不出、它是这个进程知道住着根的槽：重读一次仍读不出，在任何写之前拒
+/// （`RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`），盘上逐字节不变、录制流不多一步。
+#[test]
+fn a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write() {
+    let mut session = session_one_publish_before_the_reread();
+    session.overwrite_once();
+    assert!(
+        session.slot_is_known_to_hold_a_root(),
+        "前提：重读读出了有效根"
+    );
+    session
+        .fault
+        .set_reads(ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead);
+    let image_before = session.image_now();
+    let operations_before = session.stream.operations().len();
+    let refused = raise_rollback_floor_to_the_admission_ceiling(
+        &parameters_of_the_ring_hole_pools(),
+        &mut session.devices,
+        &mut session.allocator,
+        &mut session.current,
+        ShadowLedger::On,
+    );
+    assert!(
+        matches!(
+            refused,
+            Err(MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { ring_slot, .. })
+                if ring_slot == session.ring_slot
+        ),
+        "知道住着根的槽重读仍读不出就拒：{refused:?}"
+    );
+    assert_eq!(
+        session.stream.operations().len(),
+        operations_before,
+        "一个写、一道屏障都没发"
+    );
+    assert!(session.image_now() == image_before, "盘上逐字节不变");
+}
+
+/// 场景（攻方 `front_hole_eio 4 9 mount`，改法 A 的代价那一格）：实例 1 写到 3，实例 2 覆盖写到 8，txg 9 那次覆盖写崩在根槽写之前
+/// （记录已持久；环还没转过来，那一槽从没写过、读出来自证不过）；实例 3 可写挂载那一刻读那一槽报读错，之后照读；覆盖写到 9 + 2 × 3S。
+/// 钉住今天判成的样子：只有 txg 20 那一版（那一槽成了窗口里最小的那个、重读读到自证不过、照旧按 9 算）I-3.1 红——记账比遍历多，安全方向；
+/// 其余每一版那几条不变量都不红，txg 21 起那一槽出了窗口。
+#[test]
+fn a_front_hole_unreadable_at_mount_counts_more_allocated_only_on_the_version_that_rereads_it() {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let image_with_the_front_hole = {
+        let mut devices: RingHoleDevices = devices_of(&pool_overwritten_through(3));
+        let mounted = mount_writable(&parameters, &mut devices).expect("实例 2 的可写挂载");
+        let instance = mounted.output.instance;
+        let mut allocator = mounted.allocator.clone();
+        let mut current = mounted.current.into_file_version().expect("这一版带文件");
+        // 迭代上界：覆盖写到 txg 8。
+        while current.root.checkpoint_txg.0 + 1 < TXG_OF_THE_FRONT_HOLE {
+            let salt = current.root.checkpoint_txg.0 + 1;
+            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+            current = publish_overwrite(
+                &mut writer,
+                &mut allocator,
+                &current,
+                FirstFile {
+                    content: &content_of_the_version(salt),
+                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
+                },
+                instance,
+            )
+            .expect("覆盖写");
+        }
+        let before = pool_of(&devices);
+        let stream = SharedStream::retaining_contents();
+        let mut recording = recording_of(&before, &stream);
+        {
+            let mut writer = PoolWriter::new(&parameters, recording.as_mut_slice());
+            let published = publish_overwrite(
+                &mut writer,
+                &mut allocator.clone(),
+                &current,
+                FirstFile {
+                    content: &content_of_the_version(TXG_OF_THE_FRONT_HOLE),
+                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS
+                        + TXG_OF_THE_FRONT_HOLE,
+                },
+                instance,
+            )
+            .expect("洞那一次覆盖写");
+            assert_eq!(
+                published.root.checkpoint_txg,
+                CheckpointTxg(TXG_OF_THE_FRONT_HOLE)
+            );
+        }
+        let operations = stream.retained_operations();
+        let (writes, _segments) =
+            writes_and_segments(&operations, &fixed_geometry_of_the_ring_hole_pools());
+        let root_write = writes
+            .iter()
+            .position(|write| write.kind == StepKind::RootRecordFua)
+            .expect("那一次的根槽写");
+        let mut crashed = before;
+        crashed.apply_writes(&writes[..root_write]);
+        crashed
+    };
+    let mut session = SessionOnARingHolePool::mounted_with_the_slot_of_txg_unreadable_at_mount(
+        image_with_the_front_hole,
+        TXG_OF_THE_FRONT_HOLE,
+    );
+    assert_eq!(
+        session.txg_recorded_as_unreadable_at_mount(),
+        Some(CheckpointTxg(TXG_OF_THE_FRONT_HOLE)),
+        "前提：根环表把那一槽记成挂载时读不出、能住的最大 txg 9"
+    );
+    let ring_length = 3 * parameters.geometry.root_ring_slots_per_region.count();
+    let mut versions_with_a_violation = BTreeSet::new();
+    let mut invariants_violated_on_the_reread = Vec::new();
+    // 迭代上界：覆盖写到 9 + 2 × 3S；跨轮携带的是现行那一版与红过的版本。
+    while session.current.root.checkpoint_txg.0 < TXG_OF_THE_FRONT_HOLE + 2 * ring_length {
+        session.overwrite_once();
+        let violations = violations_of(&session.image_now());
+        if !violations.is_empty() {
+            versions_with_a_violation.insert(session.current.root.checkpoint_txg.0);
+        }
+        if session.current.root.checkpoint_txg.0 == TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD {
+            invariants_violated_on_the_reread = violations
+                .iter()
+                .map(|violation| violation.split('：').next().expect("不变量名").to_string())
+                .collect();
+            assert_eq!(
+                session.txg_recorded_as_unreadable_at_mount(),
+                Some(CheckpointTxg(TXG_OF_THE_FRONT_HOLE)),
+                "txg 20：重读读到自证不过（从没写过），照旧按 9 记着"
+            );
+        }
+    }
+    assert_eq!(
+        versions_with_a_violation,
+        BTreeSet::from([TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD]),
+        "只有重读那一版红"
+    );
+    assert_eq!(
+        invariants_violated_on_the_reread,
+        vec!["I-3.1".to_string()],
+        "红的是 I-3.1（记账比遍历多一截：门槛按 9 算，盘上窗口里最旧的有效根是 10）"
+    );
+}
+
+/// 场景：实例 1 覆盖写到 txg 5（最新那条根 a = 5）；崩溃恢复抛弃 a（它的根槽与数据单元一时读不出，择根落到 4，实例 2 写行 6、暖机 7），
+/// 实例 2 再覆盖写两次（8、9）；实例 3 可写挂载那一刻读 a 那一槽报读错，之后照读；覆盖写到 a + 3S − 1 = 28（S = 8 的 24 槽环）。
+/// 预期：记 txg 28 那条根之前 a 那一槽成了窗口里最小的那个，重读读出 txg 5 的根，按写行那一张实例表（实例 1 停在 4）被抛弃：当没有根，
+/// 那一槽从根环表拿掉、不算这个进程知道住着根的槽；txg 28 那一版那几条不变量（不含 I-7.4，见 [`INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT`]）不红。
+#[test]
+fn an_abandoned_root_read_on_the_reread_is_taken_as_no_root() {
+    let mut pool = build_pool("abandoned-root-read-on-the-e2-reread");
+    // 迭代上界：txg 4、5 两次覆盖写。
+    for _ in 0..2 {
+        let content = content_of_the_version(pool.output.root.checkpoint_txg.0 + 1);
+        overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
+    }
+    let newest = pool.output.clone();
+    let txg_of_the_abandoned_root = newest.root.checkpoint_txg.0;
+    let mounted_as_instance_two =
+        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &newest);
+    assert_eq!(
+        mounted_as_instance_two.output.instance,
+        InstanceGeneration(2),
+        "前提：实例 2 抛弃了 a"
+    );
+    // 迭代上界：实例 2 两次覆盖写。
+    for _ in 0..2 {
+        let previous = pool.output.clone();
+        pool.output = common::publish_overwrite_in_process(
+            &mut pool,
+            &previous,
+            &content_of_the_version(previous.root.checkpoint_txg.0 + 1),
+            FIXED_WRITE_TIME_SECONDS + previous.root.checkpoint_txg.0 + 1,
+            InstanceGeneration(2),
+        )
+        .expect("实例 2 覆盖写");
+    }
+    let parameters = common::parameters();
+    let fault = FaultOfOneRootRingSlot::of_the_root_slot_of_txg(
+        &parameters,
+        CheckpointTxg(txg_of_the_abandoned_root),
+        ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead,
+    );
+    let mut devices = with_the_fault(pool.reopen_recorded(), &fault);
+    let mounted = mount_writable(&parameters, &mut devices).expect("实例 3 的可写挂载");
+    fault.set_reads(ReadsOfTheRootRingSlot::PassedThrough);
+    assert!(fault.failed_reads() > 0, "挂载那一次读 a 那一槽读错了");
+    let ring_slot = target_for_publish(
+        CheckpointTxg(txg_of_the_abandoned_root),
+        parameters.geometry.root_ring_slots_per_region,
+    );
+    let mut allocator = mounted.allocator;
+    let occupancy_of = |allocator_of_the_mount: &PoolAllocator| {
+        let occupancy = allocator_of_the_mount
+            .root_ring_occupancy()
+            .expect("可写挂载装了根环表");
+        (
+            occupancy
+                .ring_slots_unreadable_at_mount()
+                .get(&ring_slot)
+                .copied(),
+            occupancy
+                .ring_slots_known_to_hold_a_root()
+                .contains(&ring_slot),
+        )
+    };
+    assert_eq!(
+        occupancy_of(&allocator),
+        (Some(CheckpointTxg(txg_of_the_abandoned_root)), false),
+        "前提：根环表把 a 那一槽记成挂载时读不出、能住的最大 txg 5"
+    );
+    let mut current = mounted.current.into_file_version().expect("这一版带文件");
+    let ring_length = 3 * parameters.geometry.root_ring_slots_per_region.count();
+    let txg_at_which_the_slot_is_reread = txg_of_the_abandoned_root + ring_length - 1;
+    // 迭代上界：覆盖写到 a + 3S − 1。
+    while current.root.checkpoint_txg.0 < txg_at_which_the_slot_is_reread {
+        let checkpoint_txg = current.root.checkpoint_txg.0 + 1;
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        current = publish_overwrite(
+            &mut writer,
+            &mut allocator,
+            &current,
+            FirstFile {
+                content: &content_of_the_version(checkpoint_txg),
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + checkpoint_txg,
+            },
+            InstanceGeneration(3),
+        )
+        .expect("实例 3 覆盖写");
+    }
+    assert_eq!(
+        occupancy_of(&allocator),
+        (None, false),
+        "重读读出被抛弃的根：当没有根，从根环表拿掉，不算知道住着根的槽"
+    );
+    assert_eq!(
+        violations_among(
+            &pool.memory_pool(),
+            &INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT
+        ),
+        Vec::<String>::new(),
+        "txg {txg_at_which_the_slot_is_reread} 那一版"
+    );
+}
+
+/// 场景：实例 1 覆盖写到 txg 3 之后可写挂载（实例 2 写行）。
+/// 预期：挂载交回的分配器上，进程内重读判「按实例表有效」用的那张表是写行那次发布写下的那一张（带实例 1 的行），
+/// 不是挂载时最新那条根指着的那一张——池级 checker 按最新那条根的实例表判有效，写行之后它带的就是写行那一张。
+#[test]
+fn the_instance_table_judging_the_rereads_after_the_mount_is_the_one_the_row_publish_wrote() {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let mut devices = devices_of(&pool_overwritten_through(3));
+    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
+    let table_of = |root: &RootRecord| {
+        instance_table_chain_of_root(&pool_of(&devices), root)
+            .expect("实例表读得出")
+            .records
+    };
+    let table_of_the_row_publish = table_of(mounted.output.row_publish.root());
+    assert_ne!(
+        table_of_the_row_publish,
+        table_of(&mounted.output.chosen_root),
+        "前提：写行那一张比挂载时最新那条根的多实例 1 那一行"
+    );
+    assert_eq!(
+        mounted
+            .allocator
+            .root_ring_occupancy()
+            .expect("可写挂载装了根环表")
+            .instance_table_judging_the_rereads(),
+        &table_of_the_row_publish,
+        "判有效用写行那一张"
+    );
+}
+
+/// 树表 0 条的池上一次可写挂载：录下来，崩在 txg 落在 `crash_before_the_root_of` 里的第一个根槽写之前（那次发布的单元与记录已持久、根槽没持久；
+/// 这次挂载没写那几个 txg 的根就整段落盘）。交回之后的盘面与这次挂载落盘的最后一个 txg（崩了的那一次是它的 txg）。
+fn mount_a_pool_without_file(
+    image: &MemoryPool,
+    crash_before_the_root_of: &[u64],
+) -> (MemoryPool, u64) {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let stream = SharedStream::retaining_contents();
+    let mut recording = recording_of(image, &stream);
+    mount_writable(&parameters, &mut recording).expect("树表 0 条的池可写挂载");
+    let (writes, _segments) = writes_and_segments(
+        &stream.retained_operations(),
+        &fixed_geometry_of_the_ring_hole_pools(),
+    );
+    let txg_of_the_root_written_by = |write: &singlefs_harness::memory_pool::RetainedWrite| {
+        write
+            .bytes()
+            .and_then(|bytes| RootRecord::parse_slot(bytes, &parameters.filesystem_identifier))
+            .map(|root| root.checkpoint_txg.0)
+    };
+    let cut = writes.iter().position(|write| {
+        write.kind == StepKind::RootRecordFua
+            && txg_of_the_root_written_by(write)
+                .is_some_and(|txg| crash_before_the_root_of.contains(&txg))
+    });
+    let mut after = image.clone();
+    match cut {
+        Some(cut) => {
+            let txg = txg_of_the_root_written_by(&writes[cut]).expect("根槽写带着一条根");
+            after.apply_writes(&writes[..cut]);
+            (after, txg)
+        }
+        None => {
+            after.apply_writes(&writes);
+            let last = writes
+                .iter()
+                .filter(|write| write.kind == StepKind::RootRecordFua)
+                .filter_map(txg_of_the_root_written_by)
+                .max()
+                .expect("可写挂载至少写一条根");
+            (after, last)
+        }
+    }
+}
+
+/// mkfs 之后一直不写文件，接连可写挂载、崩掉 `crash_before_the_root_of` 里那几个 txg 的根，到下一次挂载的写行是 `first_txg_of_the_next_mount` 为止。
+fn pool_without_file_before_the_mount_writing_its_row_at(
+    crash_before_the_root_of: &[u64],
+    first_txg_of_the_next_mount: u64,
+) -> MemoryPool {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let mut devices: RingHoleDevices = (0..2u32)
+        .map(|number| {
+            (
+                DeviceIdentity(number),
+                SparseBlockDevice::new(
+                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
+                    PhysicalBlockSizeInBytes(512),
+                ),
+            )
+        })
+        .collect();
+    make_filesystem(&parameters, &mut devices).expect("mkfs");
+    let mut image = pool_of(&devices);
+    let mut last_txg = 0u64;
+    // 迭代上界：每次挂载至少前进一个 txg，到下一次挂载的写行 txg 为止；跨轮携带的是盘面与落盘的最后一个 txg。
+    while last_txg + 1 < first_txg_of_the_next_mount {
+        let (after, last) = mount_a_pool_without_file(&image, crash_before_the_root_of);
+        assert!(last > last_txg, "每次挂载都往前推");
+        image = after;
+        last_txg = last;
+    }
+    assert_eq!(
+        last_txg + 1,
+        first_txg_of_the_next_mount,
+        "前提：下一次挂载的写行落在 {first_txg_of_the_next_mount}"
+    );
+    image
+}
+
+/// 场景：树表 0 条的池（S = 4），mkfs 之后接连可写挂载、从不写文件；txg 14 那次写行崩在根槽写之前（那一槽里留着上一圈 txg 2 的根），
+/// txg 23 那次写行同样崩在根槽写之前；下一次可写挂载写行 24、暖机 25（两块盘各一条）。挂载那一刻（取号之前）txg 14 那一槽读不出，之后照读。
+/// 预期：写行 24 那次窗口里还有比 14 旧的 13，不重读；暖机 25 那次零单元发布记根时那一槽成了窗口里最小的那个，重读读出 txg 2 的根：
+/// 挂载交回的根环表里那一槽换成重读读出的有效根，不再按挂载时读不出记，算这个进程知道住着根的槽。
+#[test]
+fn the_warm_up_on_a_version_without_file_rereads_the_slot_unreadable_at_mount() {
+    let image = pool_without_file_before_the_mount_writing_its_row_at(&[14, 23], 24);
+    let parameters = parameters_of_the_ring_hole_pools();
+    let fault = FaultOfOneRootRingSlot::of_the_root_slot_of_txg(
+        &parameters,
+        CheckpointTxg(14),
+        ReadsOfTheRootRingSlot::DeviceErrorUntilTheFirstWriteOfAnyDevice,
+    );
+    let stream = SharedStream::retaining_contents();
+    let mut devices = with_the_fault(recording_of(&image, &stream), &fault);
+    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
+    assert_eq!(
+        (
+            mounted.output.row_publish.root().checkpoint_txg,
+            mounted
+                .output
+                .warm_up_publishes
+                .iter()
+                .map(|version| version.root().checkpoint_txg)
+                .collect::<Vec<_>>()
+        ),
+        (CheckpointTxg(24), vec![CheckpointTxg(25)]),
+        "前提：写行 24、暖机一次 25"
+    );
+    assert!(fault.failed_reads() > 0, "前提：挂载读阶段读那一槽读错");
+    let ring_slot = target_for_publish(
+        CheckpointTxg(14),
+        parameters.geometry.root_ring_slots_per_region,
+    );
+    let occupancy = mounted
+        .allocator
+        .root_ring_occupancy()
+        .expect("可写挂载装了根环表");
+    assert_eq!(
+        (
+            occupancy
+                .ring_slots_unreadable_at_mount()
+                .get(&ring_slot)
+                .copied(),
+            occupancy
+                .ring_slots_known_to_hold_a_root()
+                .contains(&ring_slot)
+        ),
+        (None, true),
+        "暖机那次记根之前重读了那一槽"
+    );
+}
````

## batch9-checker/patch

实现员报告：`/tmp/claude-1000/m3-r1/batch9-checker/patch/report.md`（sha256 de2ada6c9b955121…），补丁 sha256 4bc0577f3cf15c6c…

````diff
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index fc3bc66..9cbf6c8 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -73,7 +73,10 @@ pub const SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY: &str =
 ///
 /// 第 67、68 条（下标 66、67）是 C673（三 oracle 合谋测试的补 oracle 改法没做） ⑤ ⑥ 那两条：I-9.27（树表条目的头 ID 取值）与
 /// I-5.5（从 mkfs 水位建树那一版的 t1 落点），同样只往末尾追加；GPU 判器内核两条都判。
-pub const IMPLEMENTED_INVARIANTS: [&str; 68] = [
+///
+/// 第 69 条（下标 68）是 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同，用户 2026-10-04 定新立，里程碑三第 9 批），同样只往末尾追加；
+/// GPU 判器内核还没判它（`crash_judge_gpu` 那一侧要补，归 GPU 那一簇）。
+pub const IMPLEMENTED_INVARIANTS: [&str; 69] = [
     "I-1.1",
     "I-1.2",
     "I-1.3",
@@ -142,6 +145,7 @@ pub const IMPLEMENTED_INVARIANTS: [&str; 68] = [
     REFERENCE_INTERVALS_ARE_EXACT,
     HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
     FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
+    TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
 ];
 
 /// I-1.12：走读跟随的一条指针，偏移 34 的出生树等于被指单元头里的出生树——码 1、码 3 取头偏移 43；码 2 取哪个字段随 C289 定，
@@ -194,6 +198,10 @@ pub const HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES: &str = "I-9.27";
 /// I-5.5：从 mkfs 水位第一次建树的那一版（树表非空、种类 1 条目的树 ID 是 65536、每条诞生 txg 都等于这一版的 `checkpoint_txg`）引用的数据单元里，
 /// 每块盘上落点槽号最小的那一个等于单元区起始槽号 U + 4（D3（空间分配） 已定项 8 射程，判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2）。
 pub const FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK: &str = "I-5.5";
+/// I-9.28：根环里全部读得出且自证合法的根（不按实例表判有效与否，含被崩溃恢复抛弃的旧线上的根），任两条根各自的树表里同一树 ID
+/// （条目偏移 0）的条目，诞生 txg 相同。与 I-9.14 的分工：I-9.14 只比同一条时间线上的根，这一条不分时间线（树 ID 被重发时
+/// 新线与老线上同一个 ID 的树诞生 txg 不同，C342（树 ID 水位在根读不出时退回去重发））。
+pub const TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT: &str = "I-9.28";
 
 /// 判定累加器：每条不变量记评估了几次、第一处违例、整条不适用的理由，以及射程里有一部分判不了的理由。
 #[derive(Default)]
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 5f9cae2..68a1237 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -47,6 +47,7 @@ use crate::image::{
     NAMED_ENTRY_MATCHES_THE_UNIT_HEADER, POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
     PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES, REFERENCE_INTERVALS_ARE_EXACT,
     TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
+    TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
     UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
 };
 use crate::{
@@ -3155,10 +3156,37 @@ fn written_after_its_instance_was_last_applied(
     })
 }
 
-/// 扫描方向：候选槽上头校验和过的码 2 节点，取它头里的树 ID。只数诞生代号不超过 `newest_published_txg` 的节点：
+/// 扫描方向认领一个单元之后跳过几槽（D18（块里携带什么信息） 已定项 7 认领规则，用户 2026-10-03 定按整单元跳）：
+/// 起点槽上解得出一个单元头——magic 对、类标签是 1 / 2 / 3、头校验和过（码 1 明文头末尾 105、码 2 86 + 2k、码 3 107，校验和都在 10）——
+/// 就认领它：码 1 数据单元、码 3 打包记录单元两槽，码 2 节点一槽；被认领的单元占着的后几槽是它的载荷，那里的探针不再参与认领。
+/// 不看 fsid：别的池的单元同样占着它那几槽（它是不是本池的、发没发布，归 I-1.4 / I-1.2）。解不出头 ⇒ None，这一槽不认领、下一槽照扫。
+fn slots_claimed_by_a_unit_header(header: &[u8]) -> Option<u64> {
+    if header.len() < 52 || &header[..4] != b"SFSU" {
+        return None;
+    }
+    let (plain_header_end, slots_of_the_unit) = match header[6] {
+        1 => (105, DATA_UNIT_BYTES / SLOT_BYTES),
+        2 => (86 + 2 * usize::from(header[51]), NODE_BYTES / SLOT_BYTES),
+        3 => (107, DATA_UNIT_BYTES / SLOT_BYTES),
+        _outside_the_three_unit_classes => return None,
+    };
+    (plain_header_end <= header.len()
+        && checksum_field_holds(header, plain_header_end, UNIT_HEADER_CHECKSUM_OFFSET))
+    .then_some(slots_of_the_unit)
+}
+
+/// 扫描方向：候选槽上头校验和过的码 2 节点，取它头里的树 ID。槽按从小到大扫、照认领规则按整单元跳
+/// （[`slots_claimed_by_a_unit_header`]）：用户文件内容里恰好对齐到槽边界、头校验和自洽的码 2 伪头落在它那个数据单元的第二槽，
+/// 不认领就被数成「出现过的」树 ID，合法镜像上 I-7.8 误红（判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md`
+/// 第二节 S2 第一行，改法 F4）。只数诞生代号不超过 `newest_published_txg` 的节点：
 /// 根环里没有一条根发布过的那一代，单元还不算「出现过」（I-7.8 的这一读法 2026-09-14 用户收尾弹窗定甲；字面读法会把
 /// 崩在发布之前的那个事务写出的孤儿节点也算进去，见 records 十一·七）。再排除最新根的实例表判得出没发布过的孤儿
 /// （[`written_after_its_instance_was_last_applied`]）。
+///
+/// 认领规则留的缝（判决同一行「F4 只修一半」，C89（单元头收口的遗留重算） 第 ② 笔）：释放了的两槽数据单元，第一槽被一槽的码 2 节点拿走、
+/// 第二槽的旧载荷还在时，那个节点只认领一槽，旧载荷里的伪头照样被数进来。
+///
+/// 循环：每块盘按槽号升序走一遍候选槽（轮数 = 候选槽数），跨轮携带「下一个没被认领的槽」，只增不减；没有提前出口。
 fn scanned_tree_identifiers(
     reader: &dyn ImageReader,
     geometry: &PoolGeometry,
@@ -3168,15 +3196,25 @@ fn scanned_tree_identifiers(
 ) -> BTreeSet<u64> {
     let mut identifiers = BTreeSet::new();
     for device in reader.devices() {
-        let slots = reader.candidate_unit_slots(device).unwrap_or_else(|| {
+        let mut slots = reader.candidate_unit_slots(device).unwrap_or_else(|| {
             let end = reader.device_bytes(device).unwrap_or(0) / SLOT_BYTES;
             (geometry.unit_area_start_slot..end).collect()
         });
+        slots.sort_unstable();
+        slots.dedup();
+        let mut first_slot_not_claimed = 0u64;
         for slot in slots {
+            if slot < first_slot_not_claimed {
+                continue;
+            }
             let Some(bytes) = reader.read(device, slot * SLOT_BYTES, node_bytes()) else {
                 continue;
             };
-            if &bytes[..4] != b"SFSU" || bytes[6] != 2 {
+            let Some(claimed_slots) = slots_claimed_by_a_unit_header(&bytes) else {
+                continue;
+            };
+            first_slot_not_claimed = slot + claimed_slots;
+            if bytes[6] != 2 {
                 continue;
             }
             let header_end = 86 + 2 * usize::from(bytes[51]);
@@ -4354,9 +4392,11 @@ fn judge_release_generations(
 /// （D23（journal 的角色与格式） 已定项 14：(i, T) 有效 ⟺ 实例表里没有实例 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti），
 /// 筛在 `check_pool_image` 里 `abandoned` 那一条上。有行 (i, Ti, Wi) 且 T > Ti 的根在被崩溃恢复切掉的那条旧线上，
 /// 一进不来，所以这里比的恒是同一条线上的根。
-/// **为什么必须收窄**：崩溃恢复落到一条没有文件的根之后（它之后带文件的根读不出），这个池只剩「再发一次第一个文件版本」这一条路，
-/// 那一次重新建树，诞生 txg 必然与旧线记的不同。不收窄就要在这一格判红，而那是一条正当历史。
+/// **为什么收窄**：同一条线上的根之间才有「这棵树只在第一次出现时设诞生 txg」这层关系可比；被崩溃恢复切掉的旧线上的根不与现行线比。
 /// 管理员回退是挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14），不切时间线、树表条目照抄，不走到这一格。
+/// 跨线（含被崩溃恢复抛弃的旧线）的诞生 txg 一致归 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）
+/// （[`judge_tree_identifier_birth_txg_across_every_readable_root`]）：这一条不再有「只有被切掉的那条线记得不同，不许红」那一格
+/// （判别力第 ③ 条与它的正向用例删掉，用户 2026-10-04 定；那份镜像今天红 I-9.28、I-9.14 照旧成立）。
 /// 没有被抛弃时间线的那一格由 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite` 盯着。
 fn judge_tree_table_birth_txg(scanned: &[ScannedCandidateRoot], judgements: &mut Judgements) {
     let mut by_tree: BTreeMap<u64, Vec<TreeTableEntrySighting>> = BTreeMap::new();
@@ -4410,6 +4450,96 @@ fn judge_tree_table_birth_txg(scanned: &[ScannedCandidateRoot], judgements: &mut
     }
 }
 
+/// 一棵树的树表条目在根环里某一条读得出的根那里的样子（I-9.28 按树 ID 归并这些；不分时间线，所以带上实例代号）。
+struct TreeTableEntrySightingInTheRing {
+    root_instance: u32,
+    root_checkpoint_txg: u64,
+    /// 这条根的树表单元落在哪 (设备身份, 起点槽)：两条根共用一个树表单元时，它们的条目不是两处独立的证据。
+    tree_table_placement: (u32, u64),
+    birth_txg: u64,
+}
+
+/// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同，用户 2026-10-04 定新立）：根环里每一条读得出且自证合法的根（`roots`，
+/// 不按实例表判有效与否、不按 F 与窗口滤，含被崩溃恢复抛弃的旧线上的根）各读一遍树表，按树 ID（条目偏移 0）归并，诞生 txg 必须相同。
+/// 判法 checker 自己写（D13（验证路线） 已定项 5），与 I-9.14 的 [`judge_tree_table_birth_txg`] 分开：那一条只比回退候选集。
+///
+/// 不适用：条款没写，这里取与 I-9.14 同一种——没有一棵树的条目出现在**两个不同的树表单元**里时整条报不适用（几条根共用一个树表单元时
+/// 拿同一份字节比自己是恒真判定，C374（释放代与树表诞生 txg 只有验收断言盯着） 那一类），不报成立。树表指针全零、树表读不出、
+/// 条目比字段表短的那几条根只是少一份证据，不进比较、不另报。
+fn judge_tree_identifier_birth_txg_across_every_readable_root(
+    reader: &dyn ImageReader,
+    roots: &[(u64, u64, crate::RootView)],
+    cache: &mut IndexNodeCache,
+    judgements: &mut Judgements,
+) {
+    let mut sightings_by_tree: BTreeMap<u64, Vec<TreeTableEntrySightingInTheRing>> =
+        BTreeMap::new();
+    for (_, _, root) in roots {
+        let references = references_of_root(
+            reader,
+            &root.record_bytes,
+            ReferenceScanDepth::TreeTableOnly,
+            cache,
+        );
+        let Some(tree_table_placement) = references.tree_table_placement else {
+            continue;
+        };
+        for (tree, birth_txg) in &references.tree_table_birth_txg {
+            sightings_by_tree
+                .entry(*tree)
+                .or_default()
+                .push(TreeTableEntrySightingInTheRing {
+                    root_instance: root.instance,
+                    root_checkpoint_txg: root.checkpoint_txg,
+                    tree_table_placement,
+                    birth_txg: *birth_txg,
+                });
+        }
+    }
+    let mut compared_trees = 0u64;
+    for (tree, sightings) in &sightings_by_tree {
+        let distinct_tree_tables: BTreeSet<(u32, u64)> = sightings
+            .iter()
+            .map(|sighting| sighting.tree_table_placement)
+            .collect();
+        if distinct_tree_tables.len() < 2 {
+            continue;
+        }
+        compared_trees += 1;
+        let first_birth_txg = sightings[0].birth_txg;
+        let birth_txg_is_the_same_in_every_readable_root = sightings
+            .iter()
+            .all(|sighting| sighting.birth_txg == first_birth_txg);
+        judgements.judge(
+            TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
+            birth_txg_is_the_same_in_every_readable_root,
+            || {
+                let seen: Vec<String> = sightings
+                    .iter()
+                    .map(|sighting| {
+                        format!(
+                            "根（实例 {}、txg {}）记 {}",
+                            sighting.root_instance,
+                            sighting.root_checkpoint_txg,
+                            sighting.birth_txg
+                        )
+                    })
+                    .collect();
+                format!(
+                    "树 {tree} 的树表条目诞生 txg 在根环全部读得出的根里不一：{}",
+                    seen.join("、")
+                )
+            },
+        );
+    }
+    if compared_trees == 0 {
+        judgements.not_applicable(
+            TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
+            "根环里读得出的根中，没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来",
+        );
+    }
+}
+
 /// I-3.9 与 I-9.14 共用的这一遍：候选集里每条根各走一遍取引用集合与树表。候选集只剩一条根时两条都报「不适用」——
 /// 「最早不再引用它的那条有效根」与「跨根相同」都要第二条根才有内容（2026-09-18 用户定案随 C374 立条时定的口径）。
 fn judge_release_generation_and_tree_table_birth(
@@ -7973,6 +8103,12 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
         &mut index_node_cache,
         &mut judgements,
     );
+    judge_tree_identifier_birth_txg_across_every_readable_root(
+        reader,
+        &roots,
+        &mut index_node_cache,
+        &mut judgements,
+    );
     judge_allocation_records_disjoint(
         reader,
         &roots,
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index a055e70..b42b20b 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -28,6 +28,7 @@ use singlefs_checker::image::{
     SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
     SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
     TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
+    TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
     UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
 };
 use singlefs_checker::walk::check_pool_image;
@@ -2591,10 +2592,18 @@ fn each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_i
             .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
             .map(|(name, _)| *name)
             .collect();
+        // 同一条线上两条根记得不同，必然也是全部可读根里记得不同：I-9.14 的坏镜像连带 I-9.28 红，别的一条不红。
+        let expected: Vec<&str> = if invariant == "I-9.14" {
+            vec![
+                "I-9.14",
+                TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
+            ]
+        } else {
+            vec![invariant]
+        };
         assert_eq!(
-            violated,
-            [invariant],
-            "判红的该只有 {invariant}：{verdicts:?}"
+            violated, expected,
+            "判红的该只有 {expected:?}：{verdicts:?}"
         );
     }
 }
@@ -2901,6 +2910,11 @@ fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target()
             "I-9.14" => InvariantVerdict::NotApplicable(
                 "没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来",
             ),
+            // 同一个理由，比的是根环里全部读得出的根（暖机那几代都指着 mkfs 种的第 0 版树表，空的）。这一条的坏镜像在「崩溃恢复抛弃了第三条根 C」
+            // 那一份上（`birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_holds_the_same_line_invariant_and_reddens_only_the_every_readable_root_invariant`）。
+            "I-9.28" => InvariantVerdict::NotApplicable(
+                "根环里读得出的根中，没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来",
+            ),
             // 这一份上实例 1 只有 A 一条非 0 事务号的记录（写行与两次暖机空发布都写 0）：一对可比的号凑不出来。
             // 这一条的坏镜像与阳性对照都在发布 B 之后那一份上
             // （`repeated_transaction_number_in_one_instance_reddens_the_uncut_transaction_invariant_and_only_when_separated_the_transaction_number_invariant` 与
@@ -2927,9 +2941,21 @@ fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target()
         assert_eq!(found, expected, "干净镜像上 {invariant} 的判定");
     }
     let cases = known_bad_images(&clean);
+    let (_, birth_txg, tree_table_slot_of_the_abandoned_root, abandoned_publish_txg) =
+        image_after_a_recovery_abandoned_the_third_root(
+            "known-bad-every-readable-root-birth-listed",
+        );
     let targets: std::collections::BTreeSet<&str> = cases
         .iter()
         .chain(known_bad_images_after_the_overwrite().iter())
+        .chain(
+            known_bad_images_of_the_birth_txg_across_every_readable_root(
+                tree_table_slot_of_the_abandoned_root,
+                birth_txg,
+                abandoned_publish_txg,
+            )
+            .iter(),
+        )
         .chain(known_bad_images_of_overlapping_allocation_records().iter())
         .chain(known_bad_images_of_the_root_ring_health().iter())
         .chain(known_bad_images_of_the_transaction_boundaries().iter())
@@ -3578,9 +3604,10 @@ fn rewrite_the_extent_tree_birth_txg_in_tree_table(
 
 /// 反向那一头（C511 收窄之后仍要红的那一格）：挂着的时候回退到 A 之后，D 那片树表把 extent 树的诞生 txg 记成 D 那次发布的 txg
 /// （在现行线上重新建树的样子），而回退目标 A 那一版仍记 3。向前回退不切掉任何一条线，A 与 D 在**同一条时间线上**
-/// ⇒ I-9.14（树表条目的诞生 txg 跨根不变） 必须红，而且只红它。把 I-9.14 的收窄做宽到把回退目标判出局的，这条红。
+/// ⇒ I-9.14（树表条目的诞生 txg 跨根不变） 必须红；同一条线上记得不同必然也是全部可读根里记得不同，I-9.28（全部可读根里同一树 ID 的
+/// 诞生 txg 相同） 跟着红，别的一条不红。把 I-9.14 的收窄做宽到把回退目标判出局的，这条红。
 #[test]
-fn birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_birth_invariant(
+fn birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollback_target_reddens_only_the_same_line_and_the_every_readable_root_birth_invariants(
 ) {
     let rolled_back =
         image_after_rolling_back_to_the_first_root("known-bad-after-rollback-same-line");
@@ -3606,43 +3633,85 @@ fn birth_txg_that_the_line_after_a_rollback_records_differently_from_the_rollbac
         .collect();
     assert_eq!(
         violated,
-        ["I-9.14"],
-        "回退目标 A 与回退那次发布 D 同在现行线上，诞生 txg 不一致只该红 I-9.14：{verdicts:?}"
+        [
+            "I-9.14",
+            TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT
+        ],
+        "回退目标 A 与回退那次发布 D 同在现行线上，诞生 txg 不一致只该红 I-9.14 与 I-9.28：{verdicts:?}"
     );
 }
 
-/// 正向那一头（C511 的收窄本身）：只有被崩溃恢复切掉的 C 把 extent 树的诞生 txg 记成它自己那次发布的 txg，
-/// 现行线上 A、B、实例 2 与实例 3 的根仍一致记 3 ⇒ I-9.14（树表条目的诞生 txg 跨根不变） 不拿 C 比，判成立；别的每一条也都成立。
+/// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 的坏镜像，从「崩溃恢复抛弃了第三条根 C」那一份干净镜像出发：只改被抛弃的 C 那片树表里
+/// extent 树（树 65536）的诞生 txg（`birth_txg` → C 那次发布的 txg `abandoned_publish_txg`），重封、沿引用链补到 C 的根槽。
+/// C 不在回退候选集里（I-9.14 不拿它比）；I-9.28 不分时间线、比根环里全部读得出的根。
+fn known_bad_images_of_the_birth_txg_across_every_readable_root(
+    tree_table_slot_of_the_abandoned_root: u64,
+    birth_txg: u64,
+    abandoned_publish_txg: u64,
+) -> Vec<(&'static str, Mutation)> {
+    vec![(
+        TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
+        Box::new(move |image: &mut MemoryPool| {
+            rewrite_the_extent_tree_birth_txg_in_tree_table(
+                image,
+                tree_table_slot_of_the_abandoned_root,
+                birth_txg,
+                abandoned_publish_txg,
+            );
+        }),
+    )]
+}
+
+/// C511 的收窄那一头加 I-9.28 那一头：只有被崩溃恢复切掉的 C 把 extent 树的诞生 txg 记成它自己那次发布的 txg，
+/// 现行线上 A、B、实例 2 与实例 3 的根仍一致记 3 ⇒ I-9.14（树表条目的诞生 txg 跨根不变） 不拿 C 比，判成立；
+/// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 比根环里全部读得出的根，C 也在里面，判红；别的每一条与干净镜像上相同。
 /// 管理员回退改成向前发布之后，被切掉的线只由崩溃恢复造出（D23（journal 的角色与格式） 已定项 14）。
-/// 把收窄去掉（I-9.14 拿根环里每一条根比，不按实例表判出局）这条就红。
+/// 这张镜像原先要求每一条都成立（I-9.14 判别力第 ③ 条的正向用例），用户 2026-10-04 定新立 I-9.28、删第 ③ 条之后改成这样。
+/// 把收窄去掉（I-9.14 拿根环里每一条根比，不按实例表判出局）这条红在 I-9.14 上；把 I-9.28 收成只比回退候选集，这条红在 I-9.28 上。
 #[test]
-fn birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_is_not_compared_and_every_invariant_holds(
+fn birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_holds_the_same_line_invariant_and_reddens_only_the_every_readable_root_invariant(
 ) {
     let (clean, birth_txg, tree_table_slot_of_the_abandoned_root, abandoned_publish_txg) =
         image_after_a_recovery_abandoned_the_third_root("known-good-after-recovery-abandoned-line");
-    let mut image = clean;
-    rewrite_the_extent_tree_birth_txg_in_tree_table(
-        &mut image,
+    for (invariant, found) in check_pool_image(&clean) {
+        assert_eq!(
+            found,
+            expected_verdict_with_one_record_per_transaction(invariant),
+            "崩溃恢复抛弃了 C 之后的干净镜像上 {invariant} 的判定（I-9.28 在这里真被评估过且成立）"
+        );
+    }
+    for (invariant, mutation) in known_bad_images_of_the_birth_txg_across_every_readable_root(
         tree_table_slot_of_the_abandoned_root,
         birth_txg,
         abandoned_publish_txg,
-    );
-    let verdicts = check_pool_image(&image);
-    let birth_invariant = verdicts
-        .iter()
-        .find(|(name, _)| *name == "I-9.14")
-        .expect("清单里有 I-9.14");
-    assert_eq!(
-        birth_invariant.1,
-        InvariantVerdict::Holds,
-        "被崩溃恢复切掉的 C 不与现行线比；现行线上的树表一致，I-9.14 要真被评估过且成立"
-    );
-    for (invariant, found) in &verdicts {
+    ) {
+        let mut image = clean.clone();
+        mutation(&mut image);
+        let verdicts = check_pool_image(&image);
+        let birth_invariant = verdicts
+            .iter()
+            .find(|(name, _)| *name == "I-9.14")
+            .expect("清单里有 I-9.14");
         assert_eq!(
-            *found,
-            expected_verdict_with_one_record_per_transaction(invariant),
-            "只改了被抛弃根 C 那片树表里的一个诞生 txg，{invariant} 的判定"
+            birth_invariant.1,
+            InvariantVerdict::Holds,
+            "被崩溃恢复切掉的 C 不与现行线比；现行线上的树表一致，I-9.14 要真被评估过且成立"
         );
+        for (name, found) in &verdicts {
+            if *name == invariant {
+                assert!(
+                    matches!(found, InvariantVerdict::Violated(detail)
+                        if detail.contains("树 65536") && detail.contains(&format!("txg {abandoned_publish_txg}）记 {abandoned_publish_txg}"))),
+                    "C 那片树表把树 65536 的诞生 txg 记成 {abandoned_publish_txg}，{invariant} 要红在它上面：{found:?}"
+                );
+            } else {
+                assert_eq!(
+                    *found,
+                    expected_verdict_with_one_record_per_transaction(name),
+                    "只改了被抛弃根 C 那片树表里的一个诞生 txg，{name} 的判定"
+                );
+            }
+        }
     }
 }
 
diff --git b/crates/singlefs-harness/tests/pool_checker_gap_row_conditions_and_incomplete_reference_pairs.rs b/crates/singlefs-harness/tests/pool_checker_gap_row_conditions_and_incomplete_reference_pairs.rs
new file mode 100644
index 0000000..e125b3b
--- /dev/null
+++ b/crates/singlefs-harness/tests/pool_checker_gap_row_conditions_and_incomplete_reference_pairs.rs
@@ -0,0 +1,598 @@
+//! 池级 checker 的 I-3.5（引用区间的精确性） 出现规则里两处判法的钉（m3-root-ring-code-r2 判决 RS4，`research/prompts/m3-root-ring-code-r2-main-verification.md`
+//! 第三节；正推腿探针 bb、bc、bd、be、bf、bg 在 `research/prompts/m3-root-ring-code-r2-sonnet-output.md` 第 2.2 节）：
+//! - gap-row：候选集里相邻两版 (R⁻, R) 之间有洞时，只有最新根的实例表里有一行 (R⁻ 的实例, T = R⁻.txg)、R 的实例比 R⁻ 的大、两者之间每个实例都有一行
+//!   且都是 (j, 0, 0)，这一对才照常判；三个子条件各缺一个，这一对都只判用户可见树之外的单元（N3）；
+//! - 引用不全：R⁻ 或 R 的走读记过失败（引用集合数不全），这一对整对不判。
+//!
+//! 每份镜像都放进同一种坏东西：洞右边（或这一对后一版）新写的一个单元诞生代号改小到 R⁻ 的 txg——按 (a) 它该红。条件成立时它红（对照），
+//! 条件缺一个时这一对不该判到它（N3 放过用户可见树里的单元、引用不全整对不判），I-3.5 不许红。把那个子条件放松的探针，这几份镜像就红。
+//! 镜像改法与 `checker_judges_reference_intervals_and_every_new_root_segment_field.rs` 同一种（改一个单元、重封、沿引用链补到根槽），在这里另写一份。
+mod common;
+
+use common::{
+    build_pool, parameters, publish_overwrite_in_process, tear_the_root_slot_written_for_txg,
+    BuiltPool, FIXED_WRITE_TIME_SECONDS,
+};
+use singlefs_checker::image::{InvariantVerdict, REFERENCE_INTERVALS_ARE_EXACT};
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
+use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
+use singlefs_core::mount::mount_writable;
+use singlefs_core::pointer::NodePointer;
+use singlefs_core::records::TREE_KIND_EXTENT;
+use singlefs_core::recovery::PoolReader;
+use singlefs_core::transaction::TransactionOutput;
+use singlefs_core::unit::{UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED, UNIT_MAGIC};
+use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES};
+use singlefs_harness::memory_pool::MemoryPool;
+
+const SLOT: u64 = 16384;
+const DEVICES: [u32; 2] = [0, 1];
+/// 单元区第一个槽（`common::build_pool` 那份 4 GiB 两盘池）。
+const FIRST_UNIT_SLOT: u64 = 50176;
+/// 找单元时每个槽读开头一个扇区看单元头。
+const SECTOR_BYTES_READ_FOR_A_UNIT_HEADER: usize = 512;
+/// 找单元时往后扫几个槽：这几份镜像上写过的单元都在这一段里。
+const UNIT_SLOTS_SCANNED: u64 = 2048;
+/// 根记录的自证校验和在 138 起 32 字节，罩整个 512 字节的根槽（D22（单元原子性怎么合成） 已定项 7）。
+const ROOT_SLOT_BYTES: usize = 512;
+const ROOT_RECORD_CHECKSUM_OFFSET: usize = 138;
+/// 码 2 节点的 key 宽在 51。
+const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
+/// 单元头里诞生代号的偏移（D18（块里携带什么信息） 已定项 7 / 已定项 11）：码 1 在 75，码 3 在 73，码 2 在 52 + 2k。
+const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;
+const PACKED_UNIT_BIRTH_TXG_OFFSET: usize = 73;
+const INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS: usize = 52;
+/// 实例表单元（码 3 打包记录单元，D18（块里携带什么信息） 已定项 11）：记录数在 69，记录从 136 起、每条 88；
+/// 一行 = 种类 1（行是 0）+ 实例代号 4 + 所选根的 checkpoint_txg 8 + W 8 + flags 1 + 预留 66。
+const PACKED_UNIT_RECORD_COUNT_OFFSET: usize = 69;
+const PACKED_UNIT_RECORDS_START: usize = 136;
+const INSTANCE_TABLE_RECORD_BYTES: usize = 88;
+const INSTANCE_TABLE_ROW_KIND: u8 = 0;
+const INSTANCE_TABLE_ROW_INSTANCE: std::ops::Range<usize> = 1..5;
+const INSTANCE_TABLE_ROW_PUBLISHED_TXG: std::ops::Range<usize> = 5..13;
+/// 改坏一个单元时翻它载荷里的哪一个字节（头之后、条目区里，任一份读出来整单元校验和都对不上位置条目）。
+const BYTE_INSIDE_THE_PAYLOAD_OF_A_NODE: u64 = 4000;
+
+fn read(image: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
+    PoolReader::read(
+        image,
+        DeviceIdentity(device),
+        DeviceOffsetInBytes(offset),
+        length,
+    )
+    .expect("这几份镜像上读的都是盘内的落点")
+}
+
+fn write(image: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
+    image
+        .devices
+        .get_mut(&DeviceIdentity(device))
+        .expect("这几份镜像都是两块盘")
+        .write(DeviceOffsetInBytes(offset), bytes);
+}
+
+fn reference_interval_verdict(image: &MemoryPool) -> InvariantVerdict {
+    check_pool_image(image)
+        .into_iter()
+        .find(|(identifier, _)| *identifier == REFERENCE_INTERVALS_ARE_EXACT)
+        .expect("池级 checker 的清单里有 I-3.5")
+        .1
+}
+
+/// I-3.5 判违例，第一处违例的说明里有 `expected_in_the_detail`。
+fn assert_the_reference_interval_invariant_is_violated_mentioning(
+    image: &MemoryPool,
+    expected_in_the_detail: &str,
+) {
+    match reference_interval_verdict(image) {
+        InvariantVerdict::Violated(detail) => assert!(
+            detail.contains(expected_in_the_detail),
+            "I-3.5 判违例，说明里该有「{expected_in_the_detail}」：{detail}"
+        ),
+        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+            panic!("I-3.5 应当判违例（说明里有「{expected_in_the_detail}」），实际 {other:?}")
+        }
+    }
+}
+
+/// 按类重封：先算载荷 CRC，再算头校验和（码 1：头 105、CRC 在 101；码 3：头 107、CRC 在 89；码 2：头 86 + 2k、CRC 在 76 + 2k）。
+fn reseal_unit(bytes: &mut [u8]) {
+    let key_width = usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET]);
+    let (header_end, payload_crc_offset) = match bytes[6] {
+        UNIT_CLASS_DATA => (105usize, 101usize),
+        UNIT_CLASS_PACKED => (107, 89),
+        UNIT_CLASS_INDEX_NODE => (86 + 2 * key_width, 76 + 2 * key_width),
+        other_class => panic!("这几份镜像上只有码 1、2、3 的单元，读到类 {other_class}"),
+    };
+    let payload_crc = crc32_castagnoli(&bytes[header_end..]);
+    bytes[payload_crc_offset..payload_crc_offset + 4].copy_from_slice(&payload_crc.to_le_bytes());
+    singlefs_core::unit::seal_header_checksum(bytes, header_end);
+}
+
+fn set_birth_txg_in_the_unit_header(bytes: &mut [u8], birth_txg: u64) {
+    let offset = match bytes[6] {
+        UNIT_CLASS_DATA => DATA_UNIT_BIRTH_TXG_OFFSET,
+        UNIT_CLASS_PACKED => PACKED_UNIT_BIRTH_TXG_OFFSET,
+        UNIT_CLASS_INDEX_NODE => {
+            INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS
+                + 2 * usize::from(bytes[INDEX_NODE_KEY_WIDTH_OFFSET])
+        }
+        other_class => panic!("这几份镜像上只有码 1、2、3 的单元，读到类 {other_class}"),
+    };
+    bytes[offset..offset + 8].copy_from_slice(&birth_txg.to_le_bytes());
+}
+
+/// 盘 0 单元区开头那一段里写过的每个单元：(起点槽, 字节数)。两盘同槽（D2（RAID 条带策略） 已定项 10），读一块盘就够。
+/// 迭代上界是扫的槽数；码 1 / 码 3 的单元占两槽，第二槽不当成单元头读。
+fn units_on_disk(image: &MemoryPool) -> Vec<(u64, usize)> {
+    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
+    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
+    let mut units = Vec::new();
+    let mut slot = FIRST_UNIT_SLOT;
+    while slot < FIRST_UNIT_SLOT + UNIT_SLOTS_SCANNED {
+        let header = read(image, 0, slot * SLOT, SECTOR_BYTES_READ_FOR_A_UNIT_HEADER);
+        if header[..4] != UNIT_MAGIC {
+            slot += 1;
+            continue;
+        }
+        if header[6] == UNIT_CLASS_INDEX_NODE {
+            units.push((slot, node_bytes));
+            slot += 1;
+        } else {
+            units.push((slot, data_unit_bytes));
+            slot += 2;
+        }
+    }
+    units
+}
+
+/// 根环全部槽：(盘, 偏移)。区域 r 在盘 [0, 1, 0][r] 上，起点 64 × 16384 + r × 3 MiB，每区 8 槽、槽距 4096。
+fn root_slots() -> Vec<(u32, u64)> {
+    (0..3u64)
+        .flat_map(|region| {
+            (0..8u64).map(move |slot| {
+                (
+                    [0u32, 1, 0][usize::try_from(region).expect("区域号")],
+                    64 * SLOT + region * 3 * (1 << 20) + slot * 4096,
+                )
+            })
+        })
+        .collect()
+}
+
+/// 一条位置条目：设备 4 + 槽号 6 + 整单元校验和 4。
+fn location_pattern(device: u32, slot: u64, checksum: u32) -> Vec<u8> {
+    [
+        device.to_le_bytes().to_vec(),
+        slot.to_le_bytes()[..6].to_vec(),
+        checksum.to_le_bytes().to_vec(),
+    ]
+    .concat()
+}
+
+fn replace_all(haystack: &mut [u8], needle: &[u8], replacement: &[u8]) -> bool {
+    let mut changed = false;
+    let mut index = 0;
+    while index + needle.len() <= haystack.len() {
+        if haystack[index..index + needle.len()] == *needle {
+            haystack[index..index + needle.len()].copy_from_slice(replacement);
+            changed = true;
+            index += needle.len();
+        } else {
+            index += 1;
+        }
+    }
+    changed
+}
+
+/// 一个单元（两盘同槽）的整单元校验和从 old 变成 new：把引用它的位置条目逐个补上（单元里的、根槽里的），被补的单元重封、再往上补，直到根槽；
+/// 最后把每条根照抄进同 (实例, txg) 的 journal 记录的新根段（`common` 的那一份）。
+/// 迭代上界：每补一个单元，它自己的新校验和进待补的表一次；引用链是有向无环的（子先于父写），补到根槽就停。
+fn propagate(image: &mut MemoryPool, units: &[(u64, usize)], mut changes: Vec<(u64, u32, u32)>) {
+    while let Some((slot, old, new)) = changes.pop() {
+        if old == new {
+            continue;
+        }
+        for (container, length) in units.iter().copied() {
+            if container == slot {
+                continue;
+            }
+            let before = read(image, 0, container * SLOT, length);
+            let mut bytes = before.clone();
+            let mut touched = false;
+            for device in DEVICES {
+                touched |= replace_all(
+                    &mut bytes,
+                    &location_pattern(device, slot, old),
+                    &location_pattern(device, slot, new),
+                );
+            }
+            if touched {
+                reseal_unit(&mut bytes);
+                for device in DEVICES {
+                    write(image, device, container * SLOT, &bytes);
+                }
+                changes.push((
+                    container,
+                    crc32_castagnoli(&before),
+                    crc32_castagnoli(&bytes),
+                ));
+            }
+        }
+        for (device, offset) in root_slots() {
+            let mut bytes = read(image, device, offset, ROOT_SLOT_BYTES);
+            let mut touched = false;
+            for location_device in DEVICES {
+                touched |= replace_all(
+                    &mut bytes,
+                    &location_pattern(location_device, slot, old),
+                    &location_pattern(location_device, slot, new),
+                );
+            }
+            if touched {
+                let digest = wide_checksum_with_field_zeroed(
+                    &bytes,
+                    ROOT_SLOT_BYTES,
+                    ROOT_RECORD_CHECKSUM_OFFSET,
+                );
+                bytes[ROOT_RECORD_CHECKSUM_OFFSET..ROOT_RECORD_CHECKSUM_OFFSET + 32]
+                    .copy_from_slice(&digest);
+                write(image, device, offset, &bytes);
+            }
+        }
+    }
+    common::copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
+}
+
+/// 改一个单元（两盘一起改）：改完重封它的两道校验和，新的整单元校验和沿引用链补到根槽。
+fn rewrite_unit_and_propagate(image: &mut MemoryPool, slot: u64, change: impl Fn(&mut Vec<u8>)) {
+    let units = units_on_disk(image);
+    let length = units
+        .iter()
+        .find(|(start, _)| *start == slot)
+        .unwrap_or_else(|| panic!("槽 {slot} 是盘上写过的一个单元的起点"))
+        .1;
+    let before = read(image, 0, slot * SLOT, length);
+    let mut bytes = before.clone();
+    change(&mut bytes);
+    reseal_unit(&mut bytes);
+    for device in DEVICES {
+        write(image, device, slot * SLOT, &bytes);
+    }
+    propagate(
+        image,
+        &units,
+        vec![(slot, crc32_castagnoli(&before), crc32_castagnoli(&bytes))],
+    );
+}
+
+/// 改坏一个单元（两盘各翻载荷里一个字节，不重封、不往上补）：引用它的位置条目里的整单元校验和对不上，走到它的那一版走读记一条失败。
+fn corrupt_the_unit_on_both_devices(image: &mut MemoryPool, slot: u64) {
+    for device in DEVICES {
+        image.flip_byte(
+            DeviceIdentity(device),
+            DeviceOffsetInBytes(slot * SLOT),
+            BYTE_INSIDE_THE_PAYLOAD_OF_A_NODE,
+        );
+    }
+}
+
+/// 两盘同槽：一条指针两条位置条目落在同一个槽号上，按一个槽号改两盘。
+fn slot_shared_by_both_copies(pointer: &NodePointer) -> u64 {
+    assert_eq!(
+        pointer.locations[0].slot, pointer.locations[1].slot,
+        "两盘同槽：这份镜像的改法按一个槽号改两盘"
+    );
+    pointer.locations[0].slot.0
+}
+
+fn root_slot_of_the_extent_tree(version: &TransactionOutput) -> u64 {
+    let entry = version
+        .tree_table_entries
+        .iter()
+        .find(|entry| entry.kind == TREE_KIND_EXTENT)
+        .expect("这一版的树表里有 extent 树");
+    slot_shared_by_both_copies(&entry.root)
+}
+
+fn data_unit_slot(version: &TransactionOutput) -> u64 {
+    let [pointer] = version.data_pointers.as_slice() else {
+        panic!(
+            "这几版文件都只有一个数据单元：{} 条数据指针",
+            version.data_pointers.len()
+        );
+    };
+    assert_eq!(
+        pointer.locations[0].slot, pointer.locations[1].slot,
+        "两盘同槽：数据单元按一个槽号改两盘"
+    );
+    pointer.locations[0].slot.0
+}
+
+/// 覆盖写那几版的内容：每版一个数据单元装得下，字节各不相同。
+fn content_of_the_overwrite(seed: usize) -> Vec<u8> {
+    (0..4100usize)
+        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
+        .collect()
+}
+
+/// 新池新建文件 A (1, 3) 之后同一个进程里再覆盖写 `overwrites` 次（B (1, 4)、C (1, 5)……）：交回池与每一版（A 在最前）。
+fn pool_through_overwrites(tag: &str, overwrites: usize) -> (BuiltPool, Vec<TransactionOutput>) {
+    let mut pool = build_pool(tag);
+    let mut versions = vec![pool.output.clone()];
+    for overwrite in 0..overwrites {
+        let previous = versions.last().expect("至少有 A").clone();
+        let next = publish_overwrite_in_process(
+            &mut pool,
+            &previous,
+            &content_of_the_overwrite(overwrite + 3),
+            FIXED_WRITE_TIME_SECONDS + 60 * (u64::try_from(overwrite).expect("次数") + 1),
+            InstanceGeneration(1),
+        )
+        .expect("覆盖写");
+        pool.output = next.clone();
+        versions.push(next);
+    }
+    (pool, versions)
+}
+
+/// 重开、可写挂载取下一个实例（写行、暖机），交回现行那一版（带文件）。
+fn remount_writable_for_the_next_instance(pool: &mut BuiltPool) -> TransactionOutput {
+    let mut devices_reopened = pool.reopen_recorded();
+    let mounted =
+        mount_writable(&parameters(), &mut devices_reopened).expect("重开取下一个实例：写行、暖机");
+    pool.devices = Some(devices_reopened);
+    pool.allocator = mounted.allocator;
+    mounted
+        .current
+        .into_file_version()
+        .expect("重开之前的现行那一版带文件")
+}
+
+/// 一段跨实例的历史：A (1, 3) 之后重开 `instances_after_the_first - 1` 次（每次写行、暖机），最后一个实例覆盖写一次 D。
+/// 交回池、A、D，与 A 到 D 之间每一条根的 txg（它们的根槽之后一起改坏，候选集里 A 与 D 之间就是一个洞）。
+struct HoleBetweenTheFirstVersionAndAnOverwriteOfALaterInstance {
+    pool: BuiltPool,
+    first: TransactionOutput,
+    overwrite: TransactionOutput,
+}
+
+impl HoleBetweenTheFirstVersionAndAnOverwriteOfALaterInstance {
+    fn build(tag: &str, instances_after_the_first: u32) -> Self {
+        let mut pool = build_pool(tag);
+        let first = pool.output.clone();
+        let mut current = first.clone();
+        for _ in 0..instances_after_the_first {
+            current = remount_writable_for_the_next_instance(&mut pool);
+        }
+        let overwrite = publish_overwrite_in_process(
+            &mut pool,
+            &current,
+            &content_of_the_overwrite(17),
+            FIXED_WRITE_TIME_SECONDS + 120,
+            InstanceGeneration(1 + instances_after_the_first),
+        )
+        .expect("最后一个实例覆盖写 D");
+        assert!(
+            overwrite.root.checkpoint_txg.0 > first.root.checkpoint_txg.0 + 1,
+            "A 与 D 之间有别的实例写过的根"
+        );
+        Self {
+            pool,
+            first,
+            overwrite,
+        }
+    }
+
+    /// 干净镜像加两处改法：D 新写的数据单元诞生代号改小到 A 的 txg；`change_the_rows` 改 D 指着的那片实例表的行（重封、补到根槽）；
+    /// 最后把 A 与 D 之间每一条根的根槽改坏。
+    fn image_with_the_overwrite_unit_born_at_the_first_txg(
+        &self,
+        change_the_rows: impl Fn(&mut [u8]),
+    ) -> MemoryPool {
+        let mut image = self.pool.memory_pool();
+        let first_txg = self.first.root.checkpoint_txg.0;
+        rewrite_unit_and_propagate(&mut image, data_unit_slot(&self.overwrite), |bytes| {
+            set_birth_txg_in_the_unit_header(bytes, first_txg);
+        });
+        rewrite_unit_and_propagate(
+            &mut image,
+            slot_shared_by_both_copies(&self.overwrite.root.instance_table),
+            |bytes| change_the_rows(bytes),
+        );
+        for txg in first_txg + 1..self.overwrite.root.checkpoint_txg.0 {
+            tear_the_root_slot_written_for_txg(&mut image, txg);
+        }
+        image
+    }
+}
+
+/// 实例表单元里第 `index` 条记录的起点。
+fn instance_table_record_offset(index: usize) -> usize {
+    PACKED_UNIT_RECORDS_START + INSTANCE_TABLE_RECORD_BYTES * index
+}
+
+/// 实例表单元里实例 `instance` 那一行的起点（这几份历史都只有一片实例表）。
+fn instance_table_row_of(table: &[u8], instance: u32) -> usize {
+    let record_count = usize::from(u16::from_le_bytes([
+        table[PACKED_UNIT_RECORD_COUNT_OFFSET],
+        table[PACKED_UNIT_RECORD_COUNT_OFFSET + 1],
+    ]));
+    (0..record_count)
+        .map(instance_table_record_offset)
+        .find(|offset| {
+            table[*offset] == INSTANCE_TABLE_ROW_KIND
+                && table[offset + INSTANCE_TABLE_ROW_INSTANCE.start
+                    ..offset + INSTANCE_TABLE_ROW_INSTANCE.end]
+                    == instance.to_le_bytes()
+        })
+        .unwrap_or_else(|| panic!("实例表里有实例 {instance} 那一行"))
+}
+
+fn published_txg_of_the_row(table: &[u8], row: usize) -> u64 {
+    u64::from_le_bytes(
+        table[row + INSTANCE_TABLE_ROW_PUBLISHED_TXG.start
+            ..row + INSTANCE_TABLE_ROW_PUBLISHED_TXG.end]
+            .try_into()
+            .expect("8 字节"),
+    )
+}
+
+/// gap-row 的「有行 (R⁻ 的实例, T = R⁻.txg)」：A (1, 3) 之后重开取号 2（写行 (1, 3)、暖机）、覆盖写 D，实例 2 在 D 之前的根槽都改坏，
+/// D 的数据单元诞生代号改小到 3。行是 (1, 3) 时接得上，这一对照常判、红在那个数据单元上（对照）；把那一行的 T 改成 4（A 仍按这一行有效：3 ≤ 4），
+/// T 不等于 R⁻ 的 txg，接不上：数据单元在用户可见树里、N3 不判它，I-3.5 成立（D 的树表这些用户可见树之外的单元照 (a) 判过）。
+/// 去掉「有行」整个条件（探针 bb）或只去掉「T = R⁻.txg」（探针 be）这一条红。
+#[test]
+fn a_hole_whose_row_for_the_previous_instance_names_another_txg_is_not_bridged_and_a_user_visible_unit_born_before_it_is_not_judged(
+) {
+    let history = HoleBetweenTheFirstVersionAndAnOverwriteOfALaterInstance::build(
+        "gap-row-row-names-another-txg",
+        1,
+    );
+    let first_txg = history.first.root.checkpoint_txg.0;
+    let data_unit = data_unit_slot(&history.overwrite);
+    let bridged = history.image_with_the_overwrite_unit_born_at_the_first_txg(|table| {
+        let row = instance_table_row_of(table, 1);
+        assert_eq!(
+            published_txg_of_the_row(table, row),
+            first_txg,
+            "写行写的是 (1, A 的 txg)"
+        );
+    });
+    assert_the_reference_interval_invariant_is_violated_mentioning(
+        &bridged,
+        &format!("槽 {data_unit}"),
+    );
+    let not_bridged = history.image_with_the_overwrite_unit_born_at_the_first_txg(|table| {
+        let row = instance_table_row_of(table, 1);
+        table[row + INSTANCE_TABLE_ROW_PUBLISHED_TXG.start
+            ..row + INSTANCE_TABLE_ROW_PUBLISHED_TXG.end]
+            .copy_from_slice(&(first_txg + 1).to_le_bytes());
+    });
+    assert_eq!(
+        reference_interval_verdict(&not_bridged),
+        InvariantVerdict::Holds,
+        "行 (1, T) 的 T 不是 A 的 txg：洞接不上，用户可见树里的数据单元不判"
+    );
+}
+
+/// gap-row 的「两者之间每个实例都有一行」：A (1, 3) 之后重开取号 2、再重开取号 3（各自写行、暖机）、实例 3 覆盖写 D，A 与 D 之间的根槽都改坏，
+/// D 的数据单元诞生代号改小到 3。D 指着的实例表里实例 2 那一行（实例 3 挂载写的，T 不是 0）的实例号改成 9：实例 1 与 3 之间的实例 2 没有行，
+/// 中间也就没有不是 (0, 0) 的行——只缺「每个实例都有一行」这一个子条件，接不上：数据单元不判，I-3.5 成立。实例 2 没有行，它那几条记录也就
+/// 不算被哪一次恢复施加过、不并进候选集，D 的前一版是 A。去掉这个子条件（探针 bc）这一条红。
+#[test]
+fn a_hole_with_an_instance_between_that_has_no_row_is_not_bridged_and_a_user_visible_unit_born_before_it_is_not_judged(
+) {
+    let history = HoleBetweenTheFirstVersionAndAnOverwriteOfALaterInstance::build(
+        "gap-row-instance-between-without-a-row",
+        2,
+    );
+    let image = history.image_with_the_overwrite_unit_born_at_the_first_txg(|table| {
+        let row = instance_table_row_of(table, 2);
+        table[row + INSTANCE_TABLE_ROW_INSTANCE.start..row + INSTANCE_TABLE_ROW_INSTANCE.end]
+            .copy_from_slice(&9u32.to_le_bytes());
+    });
+    assert_eq!(
+        reference_interval_verdict(&image),
+        InvariantVerdict::Holds,
+        "实例 1 与实例 3 之间的实例 2 没有行：洞接不上，用户可见树里的数据单元不判"
+    );
+}
+
+/// gap-row 的「中间的行都是 (j, 0, 0)」：同一段三个实例的历史，D 指着的实例表里实例 2 那一行（实例 3 挂载写的，T 是实例 2 最后那条根的 txg）的 T
+/// 改成 1——行还在、不是 (0, 0)；T 改小是为了让实例 2 那几条记录按这一行判成被抛弃、不当成由记录施加出来的版本并进候选集（不改的话
+/// 实例 2 最后那一版由记录施加出来，紧挨着 D 的前一版是它，那一对接得上、判的不是 A 与 D 之间的洞）。实例 1 与 3 之间那一行不是 (0, 0)，
+/// 接不上：数据单元不判，I-3.5 成立。去掉这个子条件（探针 bd）这一条红。
+#[test]
+fn a_hole_whose_instance_between_has_a_row_that_is_not_zero_is_not_bridged_and_a_user_visible_unit_born_before_it_is_not_judged(
+) {
+    let history = HoleBetweenTheFirstVersionAndAnOverwriteOfALaterInstance::build(
+        "gap-row-instance-between-not-zero",
+        2,
+    );
+    let image = history.image_with_the_overwrite_unit_born_at_the_first_txg(|table| {
+        let row = instance_table_row_of(table, 2);
+        assert_ne!(
+            published_txg_of_the_row(table, row),
+            0,
+            "实例 3 挂载写的实例 2 那一行记着它发布到的 txg"
+        );
+        table[row + INSTANCE_TABLE_ROW_PUBLISHED_TXG.start
+            ..row + INSTANCE_TABLE_ROW_PUBLISHED_TXG.end]
+            .copy_from_slice(&1u64.to_le_bytes());
+    });
+    assert_eq!(
+        reference_interval_verdict(&image),
+        InvariantVerdict::Holds,
+        "实例 2 那一行不是 (0, 0)：洞接不上，用户可见树里的数据单元不判"
+    );
+}
+
+/// 前一版引用不全：A (1, 3) → B (1, 4)，B 的数据单元诞生代号改小到 3。A 与 B 相邻、两版都走得完时它红在那个数据单元上（对照）；
+/// 再把 A 的 extent 树根（只有 A 引用：B 覆盖写换了它）两盘都改坏，A 的走读记一条失败、引用集合数不全，(A, B) 这一对整对不判，
+/// I-3.5 不许红。前一版引用不全不再跳过这一对（探针 bf）这一条红。
+#[test]
+fn a_pair_whose_previous_version_has_an_incomplete_reference_set_is_not_judged() {
+    let (pool, versions) = pool_through_overwrites("reference-interval-previous-incomplete", 1);
+    let (first, second) = (&versions[0], &versions[1]);
+    let data_unit = data_unit_slot(second);
+    let mut complete = pool.memory_pool();
+    rewrite_unit_and_propagate(&mut complete, data_unit, |bytes| {
+        set_birth_txg_in_the_unit_header(bytes, first.root.checkpoint_txg.0);
+    });
+    assert_the_reference_interval_invariant_is_violated_mentioning(
+        &complete,
+        &format!("槽 {data_unit}"),
+    );
+    let extent_root_of_the_first = root_slot_of_the_extent_tree(first);
+    assert_ne!(
+        extent_root_of_the_first,
+        root_slot_of_the_extent_tree(second),
+        "B 覆盖写换了 extent 树根：A 那一片只有 A 引用"
+    );
+    let mut incomplete = complete.clone();
+    corrupt_the_unit_on_both_devices(&mut incomplete, extent_root_of_the_first);
+    assert!(
+        !matches!(
+            reference_interval_verdict(&incomplete),
+            InvariantVerdict::Violated(_)
+        ),
+        "A 的引用集合数不全，(A, B) 那一对不判：{:?}",
+        reference_interval_verdict(&incomplete)
+    );
+}
+
+/// 后一版引用不全：A (1, 3) → B (1, 4) → C (1, 5)，B 的树表（不在用户可见树里）诞生代号改小到 3。A 与 B 相邻、两版都走得完时它照 (a) 红（对照）；
+/// 再把 B 的 extent 树根（只有 B 引用：C 覆盖写换了它）两盘都改坏，B 的走读记一条失败、引用集合数不全，(A, B) 这一对整对不判，
+/// I-3.5 不许红。后一版引用不全不再跳过这一对（探针 bg）这一条红。
+#[test]
+fn a_pair_whose_later_version_has_an_incomplete_reference_set_is_not_judged() {
+    let (pool, versions) = pool_through_overwrites("reference-interval-later-incomplete", 2);
+    let (first, second, third) = (&versions[0], &versions[1], &versions[2]);
+    let tree_table_of_the_second = slot_shared_by_both_copies(&second.root.tree_table);
+    let mut complete = pool.memory_pool();
+    rewrite_unit_and_propagate(&mut complete, tree_table_of_the_second, |bytes| {
+        set_birth_txg_in_the_unit_header(bytes, first.root.checkpoint_txg.0);
+    });
+    assert_the_reference_interval_invariant_is_violated_mentioning(
+        &complete,
+        &format!("槽 {tree_table_of_the_second}（用户可见树 None）"),
+    );
+    let extent_root_of_the_second = root_slot_of_the_extent_tree(second);
+    assert_ne!(
+        extent_root_of_the_second,
+        root_slot_of_the_extent_tree(third),
+        "C 覆盖写换了 extent 树根：B 那一片只有 B 引用"
+    );
+    let mut incomplete = complete.clone();
+    corrupt_the_unit_on_both_devices(&mut incomplete, extent_root_of_the_second);
+    assert!(
+        !matches!(
+            reference_interval_verdict(&incomplete),
+            InvariantVerdict::Violated(_)
+        ),
+        "B 的引用集合数不全，(A, B) 那一对不判：{:?}",
+        reference_interval_verdict(&incomplete)
+    );
+}
diff --git b/crates/singlefs-harness/tests/pool_checker_scan_of_tree_identifiers_claims_whole_units.rs b/crates/singlefs-harness/tests/pool_checker_scan_of_tree_identifiers_claims_whole_units.rs
new file mode 100644
index 0000000..2dea657
--- /dev/null
+++ b/crates/singlefs-harness/tests/pool_checker_scan_of_tree_identifiers_claims_whole_units.rs
@@ -0,0 +1,355 @@
+//! 池级 checker 的 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 扫描方向按认领规则跳单元（D18（块里携带什么信息） 已定项 7，
+//! 用户 2026-10-03 定按整单元跳：码 1、码 3 两槽，码 2 一槽；判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md`
+//! 第二节 S2 第一行，改法 F4）。合法历史 mkfs → 可写挂载 → 第一个文件，文件内容由用户给：内容里恰在槽边界上带一个头校验和自洽的码 2 伪头，
+//! 它落在那个数据单元的第二槽，被数据单元认领，不数进「出现过的」树 ID。四形照攻方原型
+//! `research/prompts/m3-sweep-and-tree-id-r1-opus-model/sweep-proto/src/bin/fake_header_tree_id.rs` 的 `high`、`foreign`、`control`、`inode`
+//! 四臂另写一份（D13（验证路线） 已定项 5：用例不与原型共用代码）；一份坏镜像：没被任何单元认领的槽上一个真孤儿节点，树 ID 高于水位，照旧红。
+
+use std::collections::BTreeMap;
+
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
+use singlefs_core::mount::mount_writable;
+use singlefs_core::root_ring::RootRingSlotsPerRegion;
+use singlefs_core::system_configuration::SystemImmutableSizes;
+use singlefs_core::transaction::{publish_first_file, FirstFile, PoolWriter};
+use singlefs_core::unit::{seal_header_checksum, UNIT_CLASS_INDEX_NODE, UNIT_MAGIC};
+use singlefs_format::{
+    DATA_UNIT_PAYLOAD_OFFSET, JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES, UNIT_AREA_START_SLOT,
+};
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+
+/// 这几份池的 fsid。
+const FILESYSTEM_IDENTIFIER_OF_THE_CLAIM_POOLS: [u8; 16] = *b"CLAIM-WHOLE-UNIT";
+/// 伪头改成的「别的池」的 fsid 低 8 字节（攻方 `foreign` 臂那个值）。
+const FOREIGN_FILESYSTEM_IDENTIFIER_LOW: u64 = 0x0123_4567_89ab_cdef;
+/// 每块盘 4 GiB 内存稀疏盘。
+const DEVICE_BYTES_OF_THE_CLAIM_POOLS: u64 = 4 << 30;
+/// 每区槽数 S = 4。
+const SLOTS_PER_REGION_OF_THE_CLAIM_POOLS: u64 = 4;
+/// 写入时间。
+const WRITE_TIME_SECONDS_OF_THE_CLAIM_POOLS: u64 = 1_788_000_000;
+/// 第一个文件的长度：一个数据单元，内容跨过它的第二槽起点（单元偏移 16384）。
+const FILE_BYTES_REACHING_THE_SECOND_SLOT: usize = 20_000;
+/// 伪头里的树 ID：2⁴⁰，远高于这几份池的水位（mkfs 水位 65536 起，第一个文件发到 65548）。
+const FORGED_TREE_IDENTIFIER: u64 = 1 << 40;
+/// 伪头里的诞生代号：1，不超过根环里最大的 checkpoint_txg（扫描方向第一道排除挡不住它）。
+const FORGED_BIRTH_TXG: u64 = 1;
+/// 码 2 头：树 ID 在 42、key 宽在 51；诞生代号 52 + 2k、fsid 60 + 2k、写序实例 68 + 2k、明文头末尾 86 + 2k
+/// （D18（块里携带什么信息） 已定项 18 的偏移表）。
+const INDEX_NODE_TREE_IDENTIFIER_OFFSET: usize = 42;
+const INDEX_NODE_KEY_WIDTH_OFFSET: usize = 51;
+const INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS: usize = 52;
+const INDEX_NODE_FILESYSTEM_IDENTIFIER_OFFSET_BEFORE_THE_KEYS: usize = 60;
+const INDEX_NODE_WRITE_ORDER_INSTANCE_OFFSET_BEFORE_THE_KEYS: usize = 68;
+const INDEX_NODE_PLAIN_HEADER_END_BEFORE_THE_KEYS: usize = 86;
+/// 内存稀疏盘按整扇区写。
+const SECTOR_BYTES: usize = 512;
+/// 找模板时每个槽读多少：码 2 头最宽 86 + 2 × 255 = 596 字节，一个 4096 罩得住。
+const BYTES_READ_FOR_A_UNIT_HEADER: usize = 4096;
+/// 真孤儿节点放在单元区起点之后这么多槽：这几份池写过的单元都在单元区开头几十槽里，那里之外的槽没有单元认领。
+const SLOTS_PAST_THE_UNIT_AREA_START_OF_THE_STRAY_NODE: u64 = 100_000;
+
+fn parameters_of_the_claim_pools() -> MakeFilesystemParameters {
+    MakeFilesystemParameters {
+        filesystem_identifier: FILESYSTEM_IDENTIFIER_OF_THE_CLAIM_POOLS,
+        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
+        geometry: SystemImmutableSizes {
+            physical_block_size: 512,
+            minimum_input_output_bytes: 512,
+            fixed_structure_slot_spacing: 4096,
+            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
+            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
+                SLOTS_PER_REGION_OF_THE_CLAIM_POOLS,
+            )
+            .expect("S = 4 在 4..16 里"),
+        },
+    }
+}
+
+fn memory_pool_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
+    MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.image.clone()))
+            .collect(),
+        device_size_in_bytes: DEVICE_BYTES_OF_THE_CLAIM_POOLS,
+    }
+}
+
+/// 用户给的文件内容：按位置取的字节，不含任何单元头。
+fn file_content_of(length: usize) -> Vec<u8> {
+    (0..length)
+        .map(|index| u8::try_from((index + 10) % 251).expect("小于 251"))
+        .collect()
+}
+
+/// 伪头放进文件内容的哪里。
+#[derive(Clone, Copy, Debug)]
+enum ForgedHeaderPlacement {
+    /// 内容偏移 = 单元偏移 16384 − 载荷起点：恰在数据单元第二槽的起点上。
+    OnTheSecondSlotOfTheFileUnit,
+    /// 内容偏移 0：在数据单元第一槽里、不在槽边界上。
+    AtTheStartOfTheContent,
+}
+
+impl ForgedHeaderPlacement {
+    fn content_offset(self) -> usize {
+        match self {
+            ForgedHeaderPlacement::OnTheSecondSlotOfTheFileUnit => {
+                usize::try_from(SLOT_BYTES - DATA_UNIT_PAYLOAD_OFFSET).expect("小于一个槽")
+            }
+            ForgedHeaderPlacement::AtTheStartOfTheContent => 0,
+        }
+    }
+}
+
+/// 伪头带哪个池的 fsid。
+#[derive(Clone, Copy, Debug)]
+enum ForgedHeaderFilesystem {
+    ThisPool,
+    AnotherPool,
+}
+
+/// mkfs 并可写挂载之后的池（第一个文件还没写）、挂载取到的实例。
+struct MountedClaimPool {
+    devices: Vec<(DeviceIdentity, SparseBlockDevice)>,
+    mounted: singlefs_core::mount::Mounted,
+}
+
+fn mounted_claim_pool() -> MountedClaimPool {
+    let parameters = parameters_of_the_claim_pools();
+    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [0u32, 1]
+        .into_iter()
+        .map(|device| {
+            (
+                DeviceIdentity(device),
+                SparseBlockDevice::new(
+                    DEVICE_BYTES_OF_THE_CLAIM_POOLS,
+                    PhysicalBlockSizeInBytes(512),
+                ),
+            )
+        })
+        .collect();
+    make_filesystem(&parameters, &mut devices).expect("两块 4 GiB 内存盘上 mkfs 做得成");
+    let mounted = mount_writable(&parameters, &mut devices).expect("只做过 mkfs 的池可写挂载");
+    MountedClaimPool { devices, mounted }
+}
+
+/// 盘 0 单元区里第一个码 2 节点的明文头（挂载之后盘上有实例表节点这类真节点），拿来当伪头的模板。
+fn header_of_the_first_index_node_on_the_first_device(image: &MemoryPool) -> Vec<u8> {
+    let device = &image.devices[&DeviceIdentity(0)];
+    let sectors_per_slot = SLOT_BYTES / 512;
+    let mut written_slots: Vec<u64> = device
+        .written_sectors()
+        .keys()
+        .map(|sector| sector / sectors_per_slot)
+        .filter(|slot| *slot >= UNIT_AREA_START_SLOT)
+        .collect();
+    written_slots.dedup();
+    written_slots
+        .into_iter()
+        .map(|slot| {
+            device.read(
+                DeviceOffsetInBytes(slot * SLOT_BYTES),
+                BYTES_READ_FOR_A_UNIT_HEADER,
+            )
+        })
+        .find(|head| head[..4] == UNIT_MAGIC && head[6] == UNIT_CLASS_INDEX_NODE)
+        .expect("挂载之后盘 0 单元区里有码 2 节点")
+}
+
+/// 照模板改出一个码 2 伪头：树 ID 2⁴⁰、诞生代号 1、写序实例 = 这次挂载的实例，fsid 按 `filesystem` 取，重封头校验和。交回明文头的字节。
+fn forged_index_node_header(
+    template: &[u8],
+    instance: InstanceGeneration,
+    filesystem: ForgedHeaderFilesystem,
+) -> Vec<u8> {
+    let key_span = 2 * usize::from(template[INDEX_NODE_KEY_WIDTH_OFFSET]);
+    let header_end = INDEX_NODE_PLAIN_HEADER_END_BEFORE_THE_KEYS + key_span;
+    let mut forged = template[..header_end].to_vec();
+    forged[INDEX_NODE_TREE_IDENTIFIER_OFFSET..INDEX_NODE_TREE_IDENTIFIER_OFFSET + 8]
+        .copy_from_slice(&FORGED_TREE_IDENTIFIER.to_le_bytes());
+    let birth_txg = INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS + key_span;
+    forged[birth_txg..birth_txg + 8].copy_from_slice(&FORGED_BIRTH_TXG.to_le_bytes());
+    let write_order_instance = INDEX_NODE_WRITE_ORDER_INSTANCE_OFFSET_BEFORE_THE_KEYS + key_span;
+    forged[write_order_instance..write_order_instance + 4]
+        .copy_from_slice(&instance.0.to_le_bytes());
+    match filesystem {
+        ForgedHeaderFilesystem::ThisPool => {}
+        ForgedHeaderFilesystem::AnotherPool => {
+            let filesystem_identifier =
+                INDEX_NODE_FILESYSTEM_IDENTIFIER_OFFSET_BEFORE_THE_KEYS + key_span;
+            forged[filesystem_identifier..filesystem_identifier + 8]
+                .copy_from_slice(&FOREIGN_FILESYSTEM_IDENTIFIER_LOW.to_le_bytes());
+        }
+    }
+    seal_header_checksum(&mut forged, header_end);
+    forged
+}
+
+/// 写完第一个文件之后的镜像：`forged` 给了就把伪头放进文件内容的 `placement` 那里（内容是用户的，什么字节都合法）。
+fn image_after_a_first_file_carrying(
+    forged: Option<(ForgedHeaderPlacement, ForgedHeaderFilesystem)>,
+) -> MemoryPool {
+    let parameters = parameters_of_the_claim_pools();
+    let MountedClaimPool {
+        mut devices,
+        mut mounted,
+    } = mounted_claim_pool();
+    let instance = mounted.output.instance;
+    let mut content = file_content_of(FILE_BYTES_REACHING_THE_SECOND_SLOT);
+    if let Some((placement, filesystem)) = forged {
+        let template =
+            header_of_the_first_index_node_on_the_first_device(&memory_pool_of(&devices));
+        let forged_header = forged_index_node_header(&template, instance, filesystem);
+        let at = placement.content_offset();
+        content[at..at + forged_header.len()].copy_from_slice(&forged_header);
+    }
+    {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut mounted.allocator,
+            mounted.current.root(),
+            FirstFile {
+                content: &content,
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_CLAIM_POOLS,
+            },
+            instance,
+            mounted.current.record_bytes(),
+        )
+        .expect("第一个文件");
+    }
+    memory_pool_of(&devices)
+}
+
+fn violated_invariants(image: &MemoryPool) -> BTreeMap<&'static str, String> {
+    check_pool_image(image)
+        .into_iter()
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some((invariant, detail)),
+            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+        })
+        .collect()
+}
+
+fn tree_identifier_watermark_verdict(image: &MemoryPool) -> InvariantVerdict {
+    check_pool_image(image)
+        .into_iter()
+        .find(|(invariant, _)| *invariant == "I-7.8")
+        .expect("池级 checker 的清单里有 I-7.8")
+        .1
+}
+
+/// 攻方 `high` 臂：伪头（本池 fsid、本实例写序、诞生代号 1）恰在数据单元第二槽起点上。数据单元在它的起点槽上被认领、跳两槽，
+/// 伪头不参与认领：I-7.8 真被评估过且成立，一条违例都没有（改法 F4 之前这一形 I-7.8 红：水位 65548、「出现过的」最大树 ID 2⁴⁰）。
+#[test]
+fn a_node_header_forged_on_the_second_slot_of_a_file_unit_is_claimed_by_that_unit_and_the_watermark_invariant_holds(
+) {
+    let image = image_after_a_first_file_carrying(Some((
+        ForgedHeaderPlacement::OnTheSecondSlotOfTheFileUnit,
+        ForgedHeaderFilesystem::ThisPool,
+    )));
+    assert_eq!(
+        tree_identifier_watermark_verdict(&image),
+        InvariantVerdict::Holds,
+        "数据单元第二槽上的伪头被数据单元认领，不数进出现过的树 ID"
+    );
+    assert_eq!(
+        violated_invariants(&image),
+        BTreeMap::new(),
+        "合法历史（用户内容里带伪头）上一条都不许红"
+    );
+}
+
+/// 攻方 `foreign` 臂：同一个伪头，fsid 改成别的池的。fsid 不是本池的码 2 节点扫描方向照旧数（I-7.8 那一行的读法），
+/// 所以挡住它的只有认领规则：I-7.8 成立，一条违例都没有。
+#[test]
+fn a_node_header_carrying_another_pools_identifier_forged_on_the_second_slot_of_a_file_unit_is_claimed_and_the_watermark_invariant_holds(
+) {
+    let image = image_after_a_first_file_carrying(Some((
+        ForgedHeaderPlacement::OnTheSecondSlotOfTheFileUnit,
+        ForgedHeaderFilesystem::AnotherPool,
+    )));
+    assert_eq!(
+        tree_identifier_watermark_verdict(&image),
+        InvariantVerdict::Holds,
+        "别的池 fsid 的伪头同样被数据单元认领"
+    );
+    assert_eq!(
+        violated_invariants(&image),
+        BTreeMap::new(),
+        "合法历史（用户内容里带别的池的伪头）上一条都不许红"
+    );
+}
+
+/// 攻方 `control` 臂：同长内容、不带伪头：I-7.8 成立，一条违例都没有。
+#[test]
+fn a_file_reaching_the_second_slot_of_its_unit_without_a_forged_header_holds_the_watermark_invariant(
+) {
+    let image = image_after_a_first_file_carrying(None);
+    assert_eq!(
+        tree_identifier_watermark_verdict(&image),
+        InvariantVerdict::Holds,
+        "不带伪头的对照"
+    );
+    assert_eq!(
+        violated_invariants(&image),
+        BTreeMap::new(),
+        "不带伪头的对照上一条都不许红"
+    );
+}
+
+/// 攻方 `inode` 臂：伪头放在内容偏移 0（数据单元第一槽里、不在槽边界上），扫描方向按槽起点读头，读不到它：I-7.8 成立，一条违例都没有。
+#[test]
+fn a_node_header_forged_off_the_slot_boundaries_inside_a_file_unit_is_not_seen_and_the_watermark_invariant_holds(
+) {
+    let image = image_after_a_first_file_carrying(Some((
+        ForgedHeaderPlacement::AtTheStartOfTheContent,
+        ForgedHeaderFilesystem::ThisPool,
+    )));
+    assert_eq!(
+        tree_identifier_watermark_verdict(&image),
+        InvariantVerdict::Holds,
+        "不在槽边界上的伪头扫描方向读不到"
+    );
+    assert_eq!(
+        violated_invariants(&image),
+        BTreeMap::new(),
+        "伪头不在槽边界上时一条都不许红"
+    );
+}
+
+/// 坏镜像：不带伪头的那一份上，把同一个伪头写到单元区里一个没有任何单元认领的槽（离写过的单元远远的）的起点：它是一个真孤儿码 2 节点，
+/// 树 ID 2⁴⁰ 高于根环水位，诞生代号 1 不超过根环最大 txg、写序实例没有实例表行排除它 ⇒ 认领规则不挡它，I-7.8 照旧红、只红它。
+#[test]
+fn a_stray_index_node_on_a_slot_no_unit_claims_with_a_tree_identifier_above_the_watermark_reddens_only_the_watermark_invariant(
+) {
+    let mut image = image_after_a_first_file_carrying(None);
+    let template = header_of_the_first_index_node_on_the_first_device(&image);
+    let instance = InstanceGeneration(1);
+    let mut stray_node =
+        forged_index_node_header(&template, instance, ForgedHeaderFilesystem::ThisPool);
+    stray_node.resize(stray_node.len().next_multiple_of(SECTOR_BYTES), 0);
+    let stray_slot = UNIT_AREA_START_SLOT + SLOTS_PAST_THE_UNIT_AREA_START_OF_THE_STRAY_NODE;
+    image
+        .devices
+        .get_mut(&DeviceIdentity(0))
+        .expect("盘 0")
+        .write(DeviceOffsetInBytes(stray_slot * SLOT_BYTES), &stray_node);
+    let violated = violated_invariants(&image);
+    assert_eq!(
+        violated.keys().copied().collect::<Vec<&str>>(),
+        ["I-7.8"],
+        "没被认领的槽上的真孤儿节点只红 I-7.8：{violated:?}"
+    );
+    assert!(
+        violated["I-7.8"].contains(&FORGED_TREE_IDENTIFIER.to_string()),
+        "红在树 ID 2⁴⁰ 上：{}",
+        violated["I-7.8"]
+    );
+}
diff --git b/crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs b/crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs
new file mode 100644
index 0000000..15089bd
--- /dev/null
+++ b/crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs
@@ -0,0 +1,473 @@
+//! 池级 checker 的 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同，用户 2026-10-04 定新立）：三段合法历史上成立、C342（树 ID 水位在根读不出时
+//! 退回去重发） 那一形上红。历史照攻方原型 `research/prompts/m3-sweep-and-tree-id-r1-opus-model/sweep-proto/src/bin/tree_id_histories.rs`
+//! 的 `orphan_ff`、`applied_ff`、`rollback`、`c342`（`C342_ROLL_BACK_SYSCONF=1`）四臂另写一份（D13（验证路线） 已定项 5：用例不与原型共用代码），
+//! 产物是同目录 `results/out/th-*.out`（判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第二节 S2）。
+
+use singlefs_checker::image::{
+    InvariantVerdict, TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
+};
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
+use singlefs_core::allocator::PoolAllocator;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
+use singlefs_core::mount::{mount_writable, roll_back_by_a_forward_publish, RollbackTarget};
+use singlefs_core::root_ring::RootRingSlotsPerRegion;
+use singlefs_core::system_configuration::SystemImmutableSizes;
+use singlefs_core::transaction::{
+    publish_first_file, publish_overwrite, FirstFile, PoolWriter, TransactionOutput,
+};
+use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+use singlefs_harness::segments::{FixedGeometry, StepKind};
+use singlefs_harness::{RecordingBlockDevice, SharedStream};
+
+/// 这几份池的 fsid。
+const FILESYSTEM_IDENTIFIER_OF_THE_BIRTH_POOLS: [u8; 16] = *b"BIRTH-EVERY-ROOT";
+/// 每块盘 4 GiB 内存稀疏盘。
+const DEVICE_BYTES_OF_THE_BIRTH_POOLS: u64 = 4 << 30;
+/// 每区槽数 S = 4：环长 12。
+const SLOTS_PER_REGION_OF_THE_BIRTH_POOLS: u64 = 4;
+/// 写入时间的起点。
+const WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS: u64 = 1_788_000_000;
+/// 每版文件的长度：一个数据单元装得下。
+const FILE_BYTES: usize = 3000;
+/// extent 树的树 ID：从 mkfs 水位 65536 发起的第一个号（D8（核心索引结构） 已定项 8）。
+const TREE_IDENTIFIER_OF_THE_FIRST_EXTENT_TREE: u64 = 65536;
+/// 第一个文件那一版的树 ID 水位：mkfs 水位 65536 之后发 12 个号（`th-c342-sysconf.out` 里实例 1 那一版与实例 2 重发那一版都是 65548）。
+const TREE_IDENTIFIER_WATERMARK_AFTER_THE_FIRST_FILE: u64 = 65548;
+
+type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;
+
+fn parameters_of_the_birth_pools() -> MakeFilesystemParameters {
+    MakeFilesystemParameters {
+        filesystem_identifier: FILESYSTEM_IDENTIFIER_OF_THE_BIRTH_POOLS,
+        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
+        geometry: SystemImmutableSizes {
+            physical_block_size: 512,
+            minimum_input_output_bytes: 512,
+            fixed_structure_slot_spacing: 4096,
+            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
+            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
+                SLOTS_PER_REGION_OF_THE_BIRTH_POOLS,
+            )
+            .expect("S = 4 在 4..16 里"),
+        },
+    }
+}
+
+fn fixed_geometry_of_the_birth_pools() -> FixedGeometry {
+    let parameters = parameters_of_the_birth_pools();
+    FixedGeometry {
+        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
+        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
+        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
+    }
+}
+
+fn fresh_devices() -> Devices {
+    [0u32, 1]
+        .into_iter()
+        .map(|device| {
+            (
+                DeviceIdentity(device),
+                SparseBlockDevice::new(
+                    DEVICE_BYTES_OF_THE_BIRTH_POOLS,
+                    PhysicalBlockSizeInBytes(512),
+                ),
+            )
+        })
+        .collect()
+}
+
+fn memory_pool_of(devices: &Devices) -> MemoryPool {
+    MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.image.clone()))
+            .collect(),
+        device_size_in_bytes: DEVICE_BYTES_OF_THE_BIRTH_POOLS,
+    }
+}
+
+fn devices_of(image: &MemoryPool) -> Devices {
+    image
+        .devices
+        .iter()
+        .map(|(identity, sectors)| {
+            let mut device = SparseBlockDevice::new(
+                DEVICE_BYTES_OF_THE_BIRTH_POOLS,
+                PhysicalBlockSizeInBytes(512),
+            );
+            device.image = sectors.clone();
+            (*identity, device)
+        })
+        .collect()
+}
+
+fn recorded_devices_of(
+    image: &MemoryPool,
+    stream: &SharedStream,
+) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
+    devices_of(image)
+        .into_iter()
+        .map(|(identity, device)| {
+            (
+                identity,
+                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
+            )
+        })
+        .collect()
+}
+
+/// 第 `version` 版文件的内容：每版字节各不相同。
+fn content_of_version(version: u64) -> Vec<u8> {
+    let salt = usize::try_from(version % 251).expect("小于 251");
+    (0..FILE_BYTES)
+        .map(|index| u8::try_from((index + salt * 7 + 3) % 251).expect("小于 251"))
+        .collect()
+}
+
+fn overwrite(
+    devices: &mut Devices,
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    version: u64,
+    instance: InstanceGeneration,
+) -> TransactionOutput {
+    let parameters = parameters_of_the_birth_pools();
+    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+    publish_overwrite(
+        &mut writer,
+        allocator,
+        previous,
+        FirstFile {
+            content: &content_of_version(version),
+            write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS + version,
+        },
+        instance,
+    )
+    .expect("覆盖写")
+}
+
+fn birth_txg_verdict(image: &MemoryPool) -> InvariantVerdict {
+    check_pool_image(image)
+        .into_iter()
+        .find(|(invariant, _)| {
+            *invariant == TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT
+        })
+        .expect("池级 checker 的清单里有 I-9.28")
+        .1
+}
+
+/// mkfs → 可写挂载实例 1 → 第一个文件那次发布录下来，切在第一次 `cut_before` 那一类写之前：交回崩溃镜像。
+fn first_file_crashed_before_the_first(cut_before: StepKind) -> MemoryPool {
+    let parameters = parameters_of_the_birth_pools();
+    let mut devices = fresh_devices();
+    make_filesystem(&parameters, &mut devices).expect("mkfs");
+    let first_mount = mount_writable(&parameters, &mut devices).expect("挂载实例 1");
+    let before = memory_pool_of(&devices);
+    let stream = SharedStream::retaining_contents();
+    let mut recorded = recorded_devices_of(&before, &stream);
+    {
+        let mut writer = PoolWriter::new(&parameters, recorded.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut first_mount.allocator.clone(),
+            first_mount.current.root(),
+            FirstFile {
+                content: &content_of_version(0),
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS,
+            },
+            first_mount.output.instance,
+            first_mount.current.record_bytes(),
+        )
+        .expect("第一个文件");
+    }
+    let operations = stream.retained_operations();
+    let geometry = fixed_geometry_of_the_birth_pools();
+    let cut = operations
+        .iter()
+        .position(|operation| geometry.classify(&operation.operation) == cut_before)
+        .expect("第一个文件那次发布写了这一类");
+    let mut crashed = before;
+    crashed.apply(&operations[..cut]);
+    crashed
+}
+
+/// 攻方 `orphan_ff` 臂：第一个文件那次发布切在第一条 journal 记录之前（它写的单元全是孤儿），实例 2 挂载落回暖机那一版、重发第一个文件
+/// （新线上从根环水位 65536 发号）、再覆盖写一次。被抛弃的那条线一条根都没落盘，全部可读根的树表里同一树 ID 只有实例 2 记的那一个诞生 txg：
+/// I-9.28 真被评估过且成立。
+#[test]
+fn a_first_file_cut_before_its_first_journal_record_and_reissued_by_the_next_instance_holds_the_every_readable_root_birth_invariant(
+) {
+    let parameters = parameters_of_the_birth_pools();
+    let crashed = first_file_crashed_before_the_first(StepKind::JournalRecord);
+    let mut devices = devices_of(&crashed);
+    let mut second_mount = mount_writable(&parameters, &mut devices).expect("挂载实例 2");
+    assert!(
+        second_mount.current.file_version().is_none(),
+        "第一个文件那次发布一条记录都没落：实例 2 落回没有文件的那一版"
+    );
+    let instance = second_mount.output.instance;
+    let reissued = {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut second_mount.allocator,
+            second_mount.current.root(),
+            FirstFile {
+                content: &content_of_version(1),
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS + 1,
+            },
+            instance,
+            second_mount.current.record_bytes(),
+        )
+        .expect("实例 2 重发第一个文件")
+    };
+    overwrite(
+        &mut devices,
+        &mut second_mount.allocator,
+        &reissued,
+        2,
+        instance,
+    );
+    assert_eq!(
+        birth_txg_verdict(&memory_pool_of(&devices)),
+        InvariantVerdict::Holds,
+        "重发第一个文件之后再覆盖写一次：两片树表记的诞生 txg 相同"
+    );
+}
+
+/// 攻方 `applied_ff` 臂：第一个文件那次发布切在根槽写之前（记录已落），实例 2 挂载由记录施加出那一版、写行、暖机：I-9.28 不许红；
+/// 之后再覆盖写一次（攻方那一臂没有这一步，加上它根环里一定有两片带条目的树表可比），两片树表里同一树 ID 的诞生 txg 相同：
+/// I-9.28 真被评估过且成立。
+#[test]
+fn a_first_file_whose_record_landed_without_its_root_applied_by_the_next_instance_and_overwritten_holds_the_every_readable_root_birth_invariant(
+) {
+    let parameters = parameters_of_the_birth_pools();
+    let crashed = first_file_crashed_before_the_first(StepKind::RootRecordFua);
+    let mut devices = devices_of(&crashed);
+    let mut second_mount = mount_writable(&parameters, &mut devices).expect("挂载实例 2");
+    let instance = second_mount.output.instance;
+    let verdict_after_the_mount = birth_txg_verdict(&memory_pool_of(&devices));
+    assert!(
+        !matches!(verdict_after_the_mount, InvariantVerdict::Violated(_)),
+        "实例 2 挂载之后：{verdict_after_the_mount:?}"
+    );
+    let applied = second_mount
+        .current
+        .into_file_version()
+        .expect("由记录施加出来的那一版带文件");
+    overwrite(
+        &mut devices,
+        &mut second_mount.allocator,
+        &applied,
+        1,
+        instance,
+    );
+    assert_eq!(
+        birth_txg_verdict(&memory_pool_of(&devices)),
+        InvariantVerdict::Holds,
+        "覆盖写之后两片树表记的诞生 txg 相同"
+    );
+}
+
+/// 攻方 `rollback` 臂：同一实例第一个文件 A、覆盖写三次、挂着时回退到 A（一次向前发布）、再覆盖写一次。管理员回退不切时间线、树表条目照抄：
+/// I-9.28 真被评估过且成立。
+#[test]
+fn a_rollback_by_a_forward_publish_and_an_overwrite_after_it_hold_the_every_readable_root_birth_invariant(
+) {
+    let parameters = parameters_of_the_birth_pools();
+    let mut devices = fresh_devices();
+    make_filesystem(&parameters, &mut devices).expect("mkfs");
+    let mut mounted = mount_writable(&parameters, &mut devices).expect("挂载实例 1");
+    let instance = mounted.output.instance;
+    let first = {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut mounted.allocator,
+            mounted.current.root(),
+            FirstFile {
+                content: &content_of_version(0),
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS,
+            },
+            instance,
+            mounted.current.record_bytes(),
+        )
+        .expect("第一个文件 A")
+    };
+    let target = RollbackTarget {
+        instance: first.root.instance,
+        checkpoint_txg: first.root.checkpoint_txg,
+    };
+    let mut current = first;
+    for version in 1..=3 {
+        current = overwrite(
+            &mut devices,
+            &mut mounted.allocator,
+            &current,
+            version,
+            instance,
+        );
+    }
+    roll_back_by_a_forward_publish(
+        &parameters,
+        &mut devices,
+        &mut mounted.allocator,
+        &mut current,
+        target,
+    )
+    .expect("挂着时回退到 A");
+    overwrite(&mut devices, &mut mounted.allocator, &current, 9, instance);
+    assert_eq!(
+        birth_txg_verdict(&memory_pool_of(&devices)),
+        InvariantVerdict::Holds,
+        "回退之前、回退那次与之后的树表记的诞生 txg 相同"
+    );
+}
+
+/// 攻方 `c342` 臂带 `C342_ROLL_BACK_SYSCONF=1`（`th-c342-sysconf.out`）：实例 1 第一个文件（extent 树 65536 生在 txg 3）、覆盖写 7 次到 txg 10；
+/// 这 8 次发布的根槽写与记录写全抹成 0（读不出），系统配置也退回实例 1 发第一个文件之前那一份（整卷回滚，D18（块里携带什么信息） 已定项 11
+/// 宣布不防；只抹根与记录时可写挂载在取号之前被拒）。实例 2 落回暖机那一版、写行、暖机、重发第一个文件（从根环水位 65536 再发一次号，
+/// extent 树 65536 生在 txg 5）；之后把没被实例 2 盖掉的根槽与记录放回去（故障消失）。根环里实例 1 那几条老根记 65536 生在 3、实例 2 的记 5：
+/// I-9.28 红，红在树 65536 上。
+#[test]
+fn tree_identifiers_reissued_after_a_whole_volume_rollback_redden_the_every_readable_root_birth_invariant_once_the_hidden_roots_come_back(
+) {
+    let parameters = parameters_of_the_birth_pools();
+    let geometry = fixed_geometry_of_the_birth_pools();
+    let mut devices = fresh_devices();
+    make_filesystem(&parameters, &mut devices).expect("mkfs");
+    let mut first_mount = mount_writable(&parameters, &mut devices).expect("挂载实例 1");
+    let first_instance = first_mount.output.instance;
+    let before = memory_pool_of(&devices);
+    let stream = SharedStream::retaining_contents();
+    let mut recorded = recorded_devices_of(&before, &stream);
+    {
+        let mut current = {
+            let mut writer = PoolWriter::new(&parameters, recorded.as_mut_slice());
+            publish_first_file(
+                &mut writer,
+                &mut first_mount.allocator,
+                first_mount.current.root(),
+                FirstFile {
+                    content: &content_of_version(0),
+                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS,
+                },
+                first_instance,
+                first_mount.current.record_bytes(),
+            )
+            .expect("实例 1 第一个文件")
+        };
+        for version in 1..=7u64 {
+            let mut writer = PoolWriter::new(&parameters, recorded.as_mut_slice());
+            current = publish_overwrite(
+                &mut writer,
+                &mut first_mount.allocator,
+                &current,
+                FirstFile {
+                    content: &content_of_version(version),
+                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS + version,
+                },
+                first_instance,
+            )
+            .expect("实例 1 覆盖写");
+        }
+    }
+    let operations = stream.retained_operations();
+    let mut image = before.clone();
+    image.apply(&operations);
+    let hidden: Vec<(DeviceIdentity, u64, Vec<u8>)> = operations
+        .iter()
+        .filter(|operation| {
+            matches!(
+                geometry.classify(&operation.operation),
+                StepKind::RootRecordFua | StepKind::JournalRecord
+            )
+        })
+        .filter_map(|operation| {
+            operation.contents.clone().map(|contents| {
+                (
+                    operation.operation.device,
+                    operation.operation.offset.0,
+                    contents,
+                )
+            })
+        })
+        .collect();
+    assert!(!hidden.is_empty(), "实例 1 那 8 次发布写过根槽与记录");
+    for (device, offset, contents) in &hidden {
+        image
+            .devices
+            .get_mut(device)
+            .expect("两块盘")
+            .write(DeviceOffsetInBytes(*offset), &vec![0u8; contents.len()]);
+    }
+    for operation in operations.iter().filter(|operation| {
+        geometry.classify(&operation.operation) == StepKind::SystemConfigurationSlot
+    }) {
+        let length = usize::try_from(operation.operation.length).expect("一个系统配置槽");
+        let earlier =
+            before.devices[&operation.operation.device].read(operation.operation.offset, length);
+        image
+            .devices
+            .get_mut(&operation.operation.device)
+            .expect("两块盘")
+            .write(operation.operation.offset, &earlier);
+    }
+    let mut devices_after_the_whole_volume_rollback = devices_of(&image);
+    let mut second_mount =
+        mount_writable(&parameters, &mut devices_after_the_whole_volume_rollback)
+            .expect("整卷回滚之后实例 2 可写挂载");
+    assert!(
+        second_mount.current.file_version().is_none(),
+        "实例 1 的根与记录都读不出：实例 2 落回没有文件的那一版"
+    );
+    let second_instance = second_mount.output.instance;
+    let reissued = {
+        let mut writer = PoolWriter::new(
+            &parameters,
+            devices_after_the_whole_volume_rollback.as_mut_slice(),
+        );
+        publish_first_file(
+            &mut writer,
+            &mut second_mount.allocator,
+            second_mount.current.root(),
+            FirstFile {
+                content: &content_of_version(40),
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS + 40,
+            },
+            second_instance,
+            second_mount.current.record_bytes(),
+        )
+        .expect("实例 2 重发第一个文件")
+    };
+    assert_eq!(
+        reissued.root.tree_identifier_watermark, TREE_IDENTIFIER_WATERMARK_AFTER_THE_FIRST_FILE,
+        "实例 2 从根环水位 65536 再发一次同一批树 ID"
+    );
+    let mut image_with_the_hidden_writes_put_back =
+        memory_pool_of(&devices_after_the_whole_volume_rollback);
+    for (device, offset, contents) in &hidden {
+        let now = image_with_the_hidden_writes_put_back.devices[device]
+            .read(DeviceOffsetInBytes(*offset), contents.len());
+        if now.iter().all(|byte| *byte == 0) {
+            image_with_the_hidden_writes_put_back
+                .devices
+                .get_mut(device)
+                .expect("两块盘")
+                .write(DeviceOffsetInBytes(*offset), contents);
+        }
+    }
+    match birth_txg_verdict(&image_with_the_hidden_writes_put_back) {
+        InvariantVerdict::Violated(detail) => assert!(
+            detail.contains(&format!("树 {TREE_IDENTIFIER_OF_THE_FIRST_EXTENT_TREE} ")),
+            "红在 extent 树 65536 上：{detail}"
+        ),
+        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+            panic!("老线与新线上同一树 ID 65536 诞生 txg 不同，I-9.28 要红，得到 {other:?}")
+        }
+    }
+}
diff --git a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
index f437dc9..3cb98f1 100644
--- a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
+++ b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
@@ -555,10 +555,12 @@ fn region_device(txg: u64) -> DeviceIdentity {
     parameters().region_devices[usize::try_from(target.region).expect("区域号")]
 }
 
-/// 最新的根下面还没有记账树、inode 树与分配记录树时（mkfs 之后、零单元发布之后）checker 报不适用的那 21 条；
-/// 其余 25 条要真被评估过且成立。映射 key 与单元头相符（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`）报不适用是因为中央映射树的根指针全零，
+/// 最新的根下面还没有记账树、inode 树与分配记录树时（mkfs 之后、零单元发布之后）checker 报不适用的那 31 条；
+/// 其余 38 条要真被评估过且成立（条数按今天的 `IMPLEMENTED_INVARIANTS` 69 条数）。映射 key 与单元头相符（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`）报不适用是因为中央映射树的根指针全零，
 /// 一条映射条目都没有。I-3.11（已分配减 defer 等于最新根走读） 与 I-3.1（已分配统计对得上） 同一个理由：最新根下面没有记账树。I-9.14（树表条目的诞生 txg 跨根不变） 在这里报不适用是因为 mkfs 种的第 0 版树表是空的：
 /// 一棵树的条目都没有，跨根比不出来（有了文件版本之后也只有一条根有条目，见 `NOT_APPLICABLE_WITH_ONE_FILE_VERSION`）；
+/// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 同一个理由，比的是根环里全部读得出的根（不按实例表判有效）：根环里每条根的树表都是
+/// mkfs 种的那一版或零单元发布照抄的，一条条目都没有——这个文件里每一步 I-9.28 与 I-9.14 同报不适用（下面五张表都同时有这两条）；
 /// I-5.4（分配记录罩住的槽互不相交） 同一个理由：第 0 版树表里没有分配记录树，候选集里一条根的记录都走不到；
 /// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 也是这个理由。
 /// I-1.10（码 2 条目宽等于字段表宽） 也是这个理由：一条树表条目都没有，就没有哪个码 2 节点的条目宽可比。
@@ -573,7 +575,7 @@ fn region_device(txg: u64) -> DeviceIdentity {
 /// 出现规则一个单元都判不到（mkfs 之后那一格的理由不同，见 `NOT_APPLICABLE_RIGHT_AFTER_MKFS`）。
 /// I-9.27（树表条目的头 ID 取值） 与 I-5.5（从 mkfs 水位建树那一版的 t1 落点） 三张没有文件的表都有：每条根的树表都是 0 条，
 /// 头 ID 一格都没有，也没有哪一版是从 mkfs 水位第一次建树的那一版（第一个文件版本之后两条都真被评估过且成立）。
-const NOT_APPLICABLE_WITHOUT_FILE: [&str; 30] = [
+const NOT_APPLICABLE_WITHOUT_FILE: [&str; 31] = [
     "I-9.27",
     "I-5.5",
     "I-1.10",
@@ -604,14 +606,15 @@ const NOT_APPLICABLE_WITHOUT_FILE: [&str; 30] = [
     "I-3.12",
     "I-4.1",
     "I-3.5",
+    "I-9.28",
 ];
 
-/// mkfs 刚写完那一份要多报两条不适用：journal 环整段是 0，一条自证过的记录都没有 ⇒ I-8.6（反向链算法） 与
-/// I-8.9（一次发布的记录序号连续且只有末条带标志） 都没有判的对象。
-/// 可写挂载之后环里有记录（写行 / 暖机的空发布），它们就真被评估过了——所以这两条只在 mkfs 之后那一格。
+/// mkfs 刚写完那一份比 `NOT_APPLICABLE_WITHOUT_FILE` 多报四条不适用：journal 环整段是 0，一条自证过的记录都没有 ⇒ I-8.6（反向链算法）、
+/// I-8.9（一次发布的记录序号连续且只有末条带标志）、I-8.10（记录新根段与同代根相同） 与 I-8.12（同实例记录的 txg 不减） 都没有判的对象。
+/// 可写挂载之后环里有记录（写行 / 暖机的空发布），它们就真被评估过了——所以这四条只在 mkfs 之后那一格。
 /// I-3.5（引用区间的精确性） 两张表都有，这一格的理由是回退候选集只剩 mkfs 的第 0 代根一版（环里没有记录、根环里只有它），
 /// 凑不出相邻的一对。
-const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 34] = [
+const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 35] = [
     "I-9.27",
     "I-5.5",
     "I-1.10",
@@ -646,35 +649,41 @@ const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 34] = [
     "I-8.12",
     "I-4.1",
     "I-3.5",
+    "I-9.28",
 ];
 
-/// 写完第一个文件版本之后仍报不适用的那五条（I-4.1 的理由见 `NOT_APPLICABLE_WITHOUT_FILE`）：
+/// 写完第一个文件版本之后仍报不适用的那六条（I-4.1 的理由见 `NOT_APPLICABLE_WITHOUT_FILE`）：
 /// I-7.9（回退下界 F 不高于抬 F 的上限）——没抬过 F，没有抬 F 的根可判（坏镜像与阳性对照在 `checker_known_bad_images.rs` 步 5 那段历史上）；
 /// I-9.14——树表条目只出现在这一条根的树表里（mkfs 种的第 0 版树表是空的、两次暖机空发布不写树表），
-/// 它要同一棵树的条目出现在两个树表单元里才比得出来；
+/// 它要同一棵树的条目出现在两个树表单元里才比得出来；I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 同一个理由（根环里别的根
+/// 都指着空树表）；
 /// I-8.7——这个实例只发过一个承载事务的版本（事务号 1），环里其余记录都是事务号 0 的空发布，
 /// 同一实例凑不出两条非 0 的号来比；
 /// I-8.8（前缀里的事务不被切开）——第一版一事务一条、每条都带提交标记，③ ④ 没有对象。
 /// 三条比得出来的镜像都在 `checker_known_bad_images.rs`（再覆盖写一次：树表两版、事务号 1 与 2；
 /// B 之后再接一事务两条记录的阳性对照与坏镜像）。
-const NOT_APPLICABLE_WITH_ONE_FILE_VERSION: [&str; 5] =
-    ["I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-4.1"];
+const NOT_APPLICABLE_WITH_ONE_FILE_VERSION: [&str; 6] =
+    ["I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-4.1", "I-9.28"];
 
 /// 只有实例 1 的池上第一个文件版本之后：比 `NOT_APPLICABLE_WITH_ONE_FILE_VERSION` 多 I-3.12（实例表行 T 低于引用它的根的 txg）——
 /// 实例表一行都没有（写过行的那一份上它真被评估过且成立）。
-const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 6] =
-    ["I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-3.12", "I-4.1"];
+const NOT_APPLICABLE_WITH_ONE_FILE_VERSION_OF_THE_ONLY_INSTANCE: [&str; 7] = [
+    "I-7.9", "I-8.7", "I-8.8", "I-9.14", "I-3.12", "I-4.1", "I-9.28",
+];
 
-/// 树表 0 条的一版上写行那次发布之后：比 `NOT_APPLICABLE_WITHOUT_FILE` 少 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）、
-/// I-1.10（码 2 条目宽等于字段表宽）、I-3.9（释放代落在停止引用它的那一格区间里） 与 I-5.4（分配记录罩住的槽互不相交） 四条。
+/// 树表 0 条的一版上写行那次发布之后：比 `NOT_APPLICABLE_WITHOUT_FILE` 少七条——I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）、
+/// I-1.10（码 2 条目宽等于字段表宽）、I-3.9（释放代落在停止引用它的那一格区间里）、I-5.4（分配记录罩住的槽互不相交）、
+/// I-8.11（点名项与被指单元头相同）、I-3.12（实例表行 T 低于引用它的根的 txg） 与 I-3.5（引用区间的精确性）。
 /// 写行那次发布写了一棵分配记录树、根指针住根记录（C512（树表 0 条的一版上被换下的单元记在哪），
 /// D16（发布语义） 已定项 9），里面未释放的那几条记录起点槽上都读得出单元头，
 /// I-3.10 在这一格真被评估过；这棵树按位置寻址（D8（核心索引结构） 已定项 14），checker 从根记录那一项按位置走下去、每个节点判条目宽，
 /// I-1.10 也真被评估过。checker 把根记录直接持有的那一棵并进候选根的分配记录（实审 B2）：被换下的 mkfs 那片实例表带释放标志，
 /// I-3.9 判它的释放代；I-5.4 判这棵树里的记录两两不相交——两条都真被评估过且成立。
+/// I-3.12：实例表里有了实例 1 那一行，判它的 T 低于引用它的根的 txg；I-8.11：写行那次发布的记录点名它写的实例表单元与分配记录树节点，
+/// 判点名项与单元头逐字段相同——两条都真被评估过且成立。
 /// I-3.5（引用区间的精确性） 也不在这张表里：写行那次发布（实例 2、txg 3）把实例表 COW 到新落点、写了自己那棵分配记录树，
 /// 这几个单元前一版（实例 1 的暖机根，txg 2）不引用、诞生代号 3 大于 2，出现规则真判到了它们且成立。
-const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 23] = [
+const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 24] = [
     "I-9.27",
     "I-5.5",
     "I-3.1",
@@ -698,6 +707,7 @@ const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 23] = [
     "I-3.13",
     "I-9.26",
     "I-4.1",
+    "I-9.28",
 ];
 
 /// 这一步的镜像上 checker 的每一条判决逐条核：`not_applicable` 里的报不适用，其余每一条都真被评估过且成立（一条违例都没有）。
@@ -1081,10 +1091,11 @@ fn plant_a_root_without_its_allocation_record_tree(
 
 /// 验收：取号 1、不写行；写行那次发布 (1, 1)、jsn 1、事务号 0、反向链 0、零单元、落盘 1；暖机一次 (1, 2)、jsn 2、反向链接 jsn 1、落盘 0；
 /// 分配器只有 mkfs 的两个单元（每盘 3 槽）；第一个文件版本 (1, 3)、jsn 3、事务号 1，数据单元落 50180、换下 mkfs 那片树表；
-/// mkfs 之后的录制流与新池新建文件那条逐字节相同、写出的那一版也相同；冷启动读回文件；checker 一条违例都没有——mkfs 之后
-/// 23 条成立（环是空的，I-8.6 也报不适用）、挂载之后 25 条成立，没有记账树 / inode 树 / 分配记录树 / 映射条目的那几条报不适用，
-/// 第一个文件版本之后 42 条成立、I-7.9、I-9.14、I-8.7 与 I-8.8 仍报不适用（树表条目只出现在这一条根的树表里；同一实例只有一条非 0 事务号的记录；
-/// 没有哪个事务号落了两条记录、每条都带提交标记）。
+/// mkfs 之后的录制流与新池新建文件那条逐字节相同、写出的那一版也相同；冷启动读回文件；checker 一条违例都没有（条数按今天的
+/// `IMPLEMENTED_INVARIANTS` 69 条数）——mkfs 之后 34 条成立（环是空的，I-8.6、I-8.9、I-8.10、I-8.12 也报不适用）、挂载之后 38 条成立，
+/// 没有记账树 / inode 树 / 分配记录树 / 映射条目的那几条报不适用，第一个文件版本之后 62 条成立、I-7.9、I-9.14、I-9.28、I-8.7、I-8.8、
+/// I-3.12 与 I-4.1 仍报不适用（树表条目只出现在这一条根的树表里；同一实例只有一条非 0 事务号的记录；没有哪个事务号落了两条记录、
+/// 每条都带提交标记；只有实例 1，实例表一行都没有；最新根之上没有记录）。
 #[test]
 fn writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold(
 ) {
````

## batch9-checker/patch-item1

实现员报告：`/tmp/claude-1000/m3-r1/batch9-checker/patch-item1/report.md`（sha256 28a552503642d450…），补丁 sha256 118649fe840e52e9…

````diff
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 68a1237..6e8cd0d 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -5062,7 +5062,10 @@ struct RollbackFloorCeilingBeforeTheRaise {
 }
 
 /// 算一条抬 F 的根 `raising_root` 那一刻的上限（I-7.9（回退下界 F 不高于抬 F 的上限），用户 2026-09-24 定）：
-/// 只用根环里 txg 比它小的根；有效 = 按**它自己**指着的实例表不被抛弃 ∧ txg ≥ `floor_before_the_raise`（抬之前的 F）；
+/// 只用根环里 txg 比它小、同时落在两个一圈窗口里的根（txg > N − 3S）：锚在它自己的 txg 上的那一个（`.claude/kb/invariants.md` I-7.9 那一行
+/// 2026-10-04 写回的字面，照 D16（发布语义） 已定项 1「抬 F 的上限」只在一圈窗口里的根上取），与锚在这张镜像根环里最新有效根上的那一个
+/// （候选过滤那一个，`window_anchored_at_the_newest_root_of_the_image`）——后一个只在镜像比抬 F 那一刻新时更窄，挡的是环上有洞时洞里留下的
+/// 上一圈的根（理由见 [`judge_rollback_floor_raises_against_their_ceilings`]）；有效 = 按**它自己**指着的实例表不被抛弃 ∧ txg ≥ `floor_before_the_raise`（抬之前的 F）；
 /// 非空 = 按 (txg, 实例) 排，跟前一条有效根比两条用户可见树的根指针，最旧的那条跟 [`UserVisibleTreeRootPointers::ABSENT`] 比。
 /// 它自己指着的实例表读不出、它之前一条有效根都没有 ⇒ None（上限无从算起：抬 F 的入口在这两格上拒，盘上出现这样一条抬 F 的根，
 /// 要么表后来坏了，要么撑它的根已被环盖掉）。
@@ -5072,14 +5075,21 @@ fn rollback_floor_ceiling_before_the_raise(
     roots: &[(u64, u64, crate::RootView)],
     raising_root: &crate::RootView,
     floor_before_the_raise: u64,
+    window_anchored_at_the_newest_root_of_the_image: OneRevolutionWindow,
     cache: &mut IndexNodeCache,
 ) -> Option<RollbackFloorCeilingBeforeTheRaise> {
     let instance_table_rows_of_the_raising_root =
         instance_table_rows_of_root_without_judging(reader, &raising_root.record_bytes)?;
+    let window_anchored_at_the_raising_root = OneRevolutionWindow {
+        anchor_txg: raising_root.checkpoint_txg,
+        root_ring_slot_count: geometry.regions * geometry.slots_per_region,
+    };
     let mut valid_roots_before: Vec<(u32, &crate::RootView)> = roots
         .iter()
         .filter(|(_, _, root)| {
             root.checkpoint_txg < raising_root.checkpoint_txg
+                && window_anchored_at_the_raising_root.contains(root.checkpoint_txg)
+                && window_anchored_at_the_newest_root_of_the_image.contains(root.checkpoint_txg)
                 && root.checkpoint_txg >= floor_before_the_raise
                 && !abandoned_by_instance_table_rows(&instance_table_rows_of_the_raising_root, root)
         })
@@ -5165,8 +5175,14 @@ fn rollback_floor_ceiling_before_the_raise(
 /// 「抬 F 之后回退到候选集里的根」上判红（固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version`）。
 /// 一次抬 F 推几次空发布、每次都带新 F 时，只有第一条是抬 F 的根（后几条的前一条已带新 F）。
 ///
-/// 上限只用根环里 txg 比它小的根算，不在后来每张镜像上重算：抬之后再回退，后来的根会把当初撑着 F 的非空根判成无效，而 F 不回落。
-/// 环转过之后，当初算上限用过的最旧那几条根会被盖掉，缺了它们重算只会更宽（抓不到，不会误红）。
+/// 上限只用根环里 txg 比它小、落在一圈窗口里的根算（[`rollback_floor_ceiling_before_the_raise`]），
+/// 不在后来每张镜像上重算：抬之后再回退，后来的根会把当初撑着 F 的非空根判成无效，而 F 不回落。
+/// ⚠️ 后来的镜像上当初撑着上限的根可以已被盖掉：环按 txg 从旧到新被盖时，盖掉的是一段最旧的前缀，重算只会更宽；环上有洞时不是这样——
+/// 洞那一槽留着上一圈的根，比它新的根反倒先被盖掉，按 [`rollback_floor_ceiling_from`] 的单调性重算出来的上限变低、在合法镜像上判红
+/// （里程碑三第 8 批调查的 inject-6 种子 1 那一形）。窗口只锚在被判那条根上挡不住它：洞里那条旧根 t − 3S（t 是洞的 txg、t 晚于抬 F 那条根）
+/// 落在那个窗口里。再按这张镜像的最新有效根 N 取一圈窗口就挡住了：洞里留下的根都 ≤ N − 3S；而窗口里 txg 比抬 F 那条根小的根在抬 F 那一刻
+/// 就在环里、之后不会被盖（盖它要一条 txg ≥ 它 + 3S > N 的根），所以留下的是抬 F 那一刻那批根去掉最旧的一段前缀，重算只会更宽。
+/// 带卸载记号那一支照条款不取窗口（I-7.9 那一行写明「根环里 txg 比它小的最新那条根」照旧），同一个机理在那一支上照样会误红。
 ///
 /// 按这条根 flags 里的卸载记号分两支（D22（单元原子性怎么合成） 已定项 7，用户 2026-09-26 定 (b)+(c) 合用）：
 /// 带记号的（正常卸载那一串）只判 F 不高于根环里 txg 比它小的最新那条根的 txg；不带的（准入抬 F）照下面的准入上限判。
@@ -5177,6 +5193,7 @@ fn judge_rollback_floor_raises_against_their_ceilings(
     reader: &dyn ImageReader,
     geometry: &PoolGeometry,
     roots: &[(u64, u64, crate::RootView)],
+    window_anchored_at_the_newest_root_of_the_image: OneRevolutionWindow,
     cache: &mut IndexNodeCache,
     judgements: &mut Judgements,
 ) {
@@ -5230,6 +5247,7 @@ fn judge_rollback_floor_raises_against_their_ceilings(
             roots,
             raising_root,
             floor_before_the_raise,
+            window_anchored_at_the_newest_root_of_the_image,
             cache,
         ) else {
             raising_roots_not_judged += 1;
@@ -6821,20 +6839,22 @@ const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;
 /// 根记录里「树表 0 条那一版的分配记录树的根」86 在 342（D22（单元原子性怎么合成） 已定项 7 的偏移表）。
 const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT: std::ops::Range<usize> = 342..428;
 
-/// 一圈窗口（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」，用户 2026-10-02 定）：N = 镜像根环里读得出的最新有效根的 txg，
-/// 窗口 = txg > N − 3S（3S 是环长，根环槽数）。同取这个窗口的五处里 checker 占两处：候选过滤（还有「由记录施加出来的那一版」
+/// 一圈窗口（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」，用户 2026-10-02 定）：窗口 = txg > N − 3S（3S 是环长，根环槽数），
+/// 锚 N 由调用方取——候选过滤与 I-3.1 丁-defer 取镜像根环里读得出的最新有效根的 txg；I-7.9（回退下界 F 不高于抬 F 的上限）
+/// 另取一个锚在被判那条抬 F 的根的 txg 上的（`.claude/kb/invariants.md` I-7.9 那一行，2026-10-04 写回），两个一起用。同取这个窗口的五处里 checker 占两处：候选过滤（还有「由记录施加出来的那一版」
 /// 判回收用的最旧有效根）与 I-3.1（已分配统计对得上） 丁-defer 的门槛。窗口之外的有效根是环上有洞时留在洞那一槽里的上一圈的根，
 /// 分配器的回收门槛不再被它压住（`crates/singlefs-core/src/allocator.rs` 的 `RootRingOccupancy::oldest_valid_root_in_the_window`）。
 #[derive(Clone, Copy, Debug)]
 struct OneRevolutionWindow {
-    newest_valid_root_txg: u64,
+    /// 锚 N。
+    anchor_txg: u64,
     root_ring_slot_count: u64,
 }
 
 impl OneRevolutionWindow {
     /// txg 落在窗口里：txg > N − 3S，写成 txg + 3S > N（N < 3S 时整个环都在窗口里，不减出负数）。
     fn contains(self, checkpoint_txg: u64) -> bool {
-        checkpoint_txg.saturating_add(self.root_ring_slot_count) > self.newest_valid_root_txg
+        checkpoint_txg.saturating_add(self.root_ring_slot_count) > self.anchor_txg
     }
 }
 
@@ -7889,7 +7909,7 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
         );
     // 一圈窗口（D16（发布语义） 已定项 1）：N 取最新根（它按自己的实例表恒有效），3S 是根环槽数。
     let window = OneRevolutionWindow {
-        newest_valid_root_txg: newest_txg,
+        anchor_txg: newest_txg,
         root_ring_slot_count: geometry.regions * geometry.slots_per_region,
     };
     // 掉出遍历的根槽按理由各记一次（几样都占的各边都记）：I-3.1 判红时这几个数就是「遍历为什么少算」的机理标识。
@@ -8138,6 +8158,7 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
         reader,
         &geometry,
         &roots,
+        window,
         &mut index_node_cache,
         &mut judgements,
     );
@@ -8523,14 +8544,14 @@ mod tests {
     #[test]
     fn the_one_revolution_window_keeps_the_txgs_above_the_newest_minus_the_ring_length() {
         let window = OneRevolutionWindow {
-            newest_valid_root_txg: 35,
+            anchor_txg: 35,
             root_ring_slot_count: 24,
         };
         assert!(!window.contains(11), "35 − 24 = 11 不在窗口里");
         assert!(window.contains(12), "12 在窗口里");
         assert!(window.contains(35), "N 自己在窗口里");
         let young_ring = OneRevolutionWindow {
-            newest_valid_root_txg: 7,
+            anchor_txg: 7,
             root_ring_slot_count: 24,
         };
         assert!(young_ring.contains(0), "N < 环长：第 0 代也在窗口里");
@@ -8618,7 +8639,7 @@ mod tests {
 
     fn small_world_window(newest_txg: u64) -> OneRevolutionWindow {
         OneRevolutionWindow {
-            newest_valid_root_txg: newest_txg,
+            anchor_txg: newest_txg,
             root_ring_slot_count: SMALL_WORLD_ROOT_RING_SLOT_COUNT,
         }
     }
diff --git b/crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs b/crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs
new file mode 100644
index 0000000..5bbe534
--- /dev/null
+++ b/crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs
@@ -0,0 +1,417 @@
+//! 池级 checker 的 I-7.9（回退下界 F 不高于抬 F 的上限） 不带卸载记号那一支：「之前的根」只取一圈窗口里的根
+//! （`.claude/kb/invariants.md` I-7.9 那一行 2026-10-04 写回，照 D16（发布语义） 已定项 1「抬 F 的上限」只在一圈窗口里的根上取）。
+//! 窗口取两个、一起用：锚在被判那条抬 F 的根上的（条款字面），与锚在这张镜像根环里最新有效根上的（候选过滤那一个）；只取前一个时下面三形
+//! 合法镜像照样红（`crates/singlefs-checker/src/walk.rs` 的 `judge_rollback_floor_raises_against_their_ceilings` 写了理由）。
+//! 环上有洞时洞那一槽留着上一圈的根，后来的发布盖掉的是比它新的根：在后来那张镜像上重算的上限不再「只会更宽」。三形合法镜像，抬 F 那一刻
+//! 实现与 checker 自己的公式算出同一个上限、F 都等于它，再挂之后洞里的旧根顶上来：
+//! - 删除与并发那条线交来的草稿：卸载那一串空发布记录已落、根没落那一形的最小复现（覆盖写切在「记录已落、根没落」再可写挂载）；
+//! - 里程碑三第 8 批调查的 inject-6 种子 1：随机历史（比重「抬 F 之后复用」、4 GiB、20 步、种子 1）里实例 5 第二次发布 txg 25 的根槽写被扣下、
+//!   记录已落（由记录施加出来那一类洞），再可写挂载加一次发布；
+//! - 同一段历史里实例 6 写行那次发布 txg 26 的根槽写被扣下（K1 类洞：那一版没被施加、取号越过它），再可写挂载加一次发布。
+//!
+//! 另一份坏镜像：洞上那张镜像里把那条抬 F 的根带的 F 改得高于窗口里的根算出的上限，I-7.9 照旧红。
+//! 这几份都不经 checker 档：崩溃状态只取一个（扣下的那一个根槽写之前全落），不枚举。
+
+mod common_admission;
+
+use common_admission::*;
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::checksum::wide_checksum_with_field_zeroed;
+use singlefs_core::mount::mount_writable_with_space_admission;
+use singlefs_core::recovery::{
+    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, RootRingSlotReading,
+};
+use singlefs_core::root_ring::slot_offset;
+use singlefs_core::transaction::{
+    publish_first_file, publish_new_inodes, publish_overwrite, FirstFile, PoolVersion, PoolWriter,
+};
+use singlefs_harness::history::{
+    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
+    HistoryExecution, HistorySeed, PerStepChecker,
+};
+use singlefs_harness::memory_pool::{
+    root_identity_of_write, root_identity_written_by, writes_and_segments, MemoryPool,
+    SparseBlockDevice,
+};
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};
+use std::collections::BTreeSet;
+
+/// 根记录里回退下界 F 的偏移：树表指针 36 起 86、映射根指针之前（D22（单元原子性怎么合成） 已定项 7 的偏移表，池级 checker 读的那一处）。
+const ROOT_RECORD_ROLLBACK_FLOOR_OFFSET: usize = 36 + 86 + 8;
+/// 根记录的自证校验和在 138 起 32 字节，罩整个 512 字节的根槽（D22（单元原子性怎么合成） 已定项 7）。
+const ROOT_RECORD_CHECKSUM_OFFSET: usize = 138;
+const ROOT_SLOT_BYTES: usize = 512;
+/// 草稿那段历史：第一个文件之后覆盖写 3 次、崩了再挂一次，再覆盖写 16 次；第 17 次覆盖写切在「记录已落、根没落」。
+const OVERWRITES_BEFORE_THE_CRASH_REMOUNT: usize = 3;
+const OVERWRITES_AFTER_THE_CRASH_REMOUNT: usize = 16;
+/// 草稿那一次切下的发布的 txg（它的根槽一直没写，留着上一圈的根）。
+const TXG_OF_THE_PUBLISH_WHOSE_ROOT_SLOT_IS_WITHHELD_IN_THE_DRAFT: u64 = 31;
+/// inject-6 那段随机历史：比重「抬 F 之后复用」、20 步、种子 1、4 GiB 两盘。
+const INJECT_SIX_HISTORY_OPERATIONS: usize = 20;
+const INJECT_SIX_HISTORY_SEED: u64 = 1;
+/// 那段历史里抬 F 的根：实例 1 txg 9 把 F 从 0 抬到 5。
+const INJECT_SIX_RAISING_ROOT_TXG: u64 = 9;
+/// 扣下的根槽写：实例 5 txg 25（由记录施加出来那一类洞，占洞的是 txg 1）、实例 6 txg 26（写行那次，K1 类洞，占洞的是 txg 2）。
+const INJECT_SIX_WITHHELD_ROOT_APPLIED_BY_ITS_RECORD: (u32, u64) = (5, 25);
+const INJECT_SIX_WITHHELD_ROOT_OF_THE_ROW_PUBLISH: (u32, u64) = (6, 26);
+/// 挂载之后那一次发布的内容长度：一个数据单元装得下。
+const CONTENT_BYTES_OF_THE_PUBLISH_AFTER_THE_MOUNT: usize = 3000;
+/// 挂载之后那一次发布的写入时间。
+const WRITE_TIME_SECONDS_OF_THE_PUBLISH_AFTER_THE_MOUNT: u64 = 1_788_000_000;
+
+fn rollback_floor_verdict(image: &MemoryPool) -> InvariantVerdict {
+    check_pool_image(image)
+        .into_iter()
+        .find(|(name, _)| *name == "I-7.9")
+        .map(|(_, verdict)| verdict)
+        .expect("池级 checker 判 I-7.9")
+}
+
+/// 镜像根环里自证过的根：txg 的集合。
+fn root_txgs_in_the_ring(image: &MemoryPool) -> BTreeSet<u64> {
+    let immutable = choose_system_configuration(image)
+        .expect("系统配置读得出")
+        .immutable;
+    every_root_ring_slot(&immutable.sizes)
+        .into_iter()
+        .filter_map(|ring_slot| {
+            match read_root_ring_slot(
+                image,
+                &immutable.region_devices,
+                &immutable.sizes,
+                &immutable.filesystem_identifier,
+                ring_slot,
+            ) {
+                RootRingSlotReading::SelfVerified(root) => Some(root.checkpoint_txg.0),
+                RootRingSlotReading::Bad(_) => None,
+            }
+        })
+        .collect()
+}
+
+/// 把根环里 txg 是 `txg` 的那条根带的 F 改成 `floor`、重封根槽自证校验和（只动根槽：同 (实例, txg) 的记录不跟着改，I-8.10 由它说话）。
+fn rewrite_the_floor_of_the_root_at(image: &mut MemoryPool, txg: u64, floor: u64) {
+    let immutable = choose_system_configuration(&*image)
+        .expect("系统配置读得出")
+        .immutable;
+    let ring_slot = every_root_ring_slot(&immutable.sizes)
+        .into_iter()
+        .find(|ring_slot| {
+            matches!(
+                read_root_ring_slot(
+                    &*image,
+                    &immutable.region_devices,
+                    &immutable.sizes,
+                    &immutable.filesystem_identifier,
+                    *ring_slot,
+                ),
+                RootRingSlotReading::SelfVerified(root) if root.checkpoint_txg.0 == txg
+            )
+        })
+        .unwrap_or_else(|| panic!("根环里有 txg {txg} 的根"));
+    let device =
+        immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号装得进 usize")];
+    let offset = slot_offset(ring_slot, immutable.sizes.fixed_structure_slot_spacing);
+    let sectors = image.devices.get_mut(&device).expect("根环区域所在的盘");
+    let mut bytes = sectors.read(offset, ROOT_SLOT_BYTES);
+    bytes[ROOT_RECORD_ROLLBACK_FLOOR_OFFSET..ROOT_RECORD_ROLLBACK_FLOOR_OFFSET + 8]
+        .copy_from_slice(&floor.to_le_bytes());
+    let digest =
+        wide_checksum_with_field_zeroed(&bytes, ROOT_SLOT_BYTES, ROOT_RECORD_CHECKSUM_OFFSET);
+    bytes[ROOT_RECORD_CHECKSUM_OFFSET..ROOT_RECORD_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
+    sectors.write(offset, &bytes);
+}
+
+/// 一张崩溃镜像上可写挂载（准入照式子判），挂上之后再发一次（有文件就覆盖写、没有就写第一个文件）：交回之后的镜像。
+fn image_after_a_writable_mount_and_one_publish(
+    crashed: &MemoryPool,
+    width: HistoryDeviceWidth,
+) -> MemoryPool {
+    let parameters = width.parameters();
+    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = crashed
+        .devices
+        .iter()
+        .map(|(identity, sectors)| {
+            let mut device = SparseBlockDevice::new(
+                crashed.device_size_in_bytes,
+                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
+            );
+            device.image = sectors.clone();
+            (*identity, device)
+        })
+        .collect();
+    let mut mounted = mount_writable_with_space_admission(
+        &parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+    .expect("崩溃镜像上可写挂载做得成");
+    let instance = mounted.output.instance;
+    let content: Vec<u8> = (0..CONTENT_BYTES_OF_THE_PUBLISH_AFTER_THE_MOUNT)
+        .map(|index| u8::try_from(index % 251).expect("小于 251"))
+        .collect();
+    let file = FirstFile {
+        content: &content,
+        write_time_seconds: WRITE_TIME_SECONDS_OF_THE_PUBLISH_AFTER_THE_MOUNT,
+    };
+    {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        match &mounted.current {
+            PoolVersion::WithFile(current) => {
+                publish_overwrite(&mut writer, &mut mounted.allocator, current, file, instance)
+                    .expect("挂载之后覆盖写一次");
+            }
+            // 删完那一版（D8（核心索引结构） 已定项 16 第 5 条）：之后只许建 inode、写行与空发布，建一个 size 0 的 inode。
+            PoolVersion::WithTreesAndNoFile(current) => {
+                publish_new_inodes(
+                    &mut writer,
+                    &mut mounted.allocator,
+                    current,
+                    1,
+                    WRITE_TIME_SECONDS_OF_THE_PUBLISH_AFTER_THE_MOUNT,
+                    instance,
+                )
+                .expect("挂载之后建一个 inode");
+            }
+            PoolVersion::WithoutFile(_) => {
+                publish_first_file(
+                    &mut writer,
+                    &mut mounted.allocator,
+                    mounted.current.root(),
+                    file,
+                    instance,
+                    mounted.current.record_bytes(),
+                )
+                .expect("挂载之后写第一个文件");
+            }
+        }
+    }
+    MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.image.clone()))
+            .collect(),
+        device_size_in_bytes: crashed.device_size_in_bytes,
+    }
+}
+
+/// inject-6 那段随机历史录下来，崩在 (实例, txg) = `withheld` 那条根的根槽写上：那一段只有这一个写、它没落，之前的写全落。
+fn inject_six_history_crashed_on_the_root_slot_write_of(withheld: (u32, u64)) -> MemoryPool {
+    let width = HistoryDeviceWidth::FourGibibytes;
+    let execution = HistoryExecution {
+        per_step_checker: PerStepChecker::Skipped,
+        device_width: width,
+        space_admission: SpaceAdmission::JudgedByTheFormula,
+    };
+    let history = generate_history_with_weights(
+        HistorySeed(INJECT_SIX_HISTORY_SEED),
+        INJECT_SIX_HISTORY_OPERATIONS,
+        &GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
+    );
+    let stream = SharedStream::retaining_contents();
+    let _run = execute_history_with(&history, execution, &stream, &mut |_| {});
+    let (writes, segments) =
+        writes_and_segments(&stream.retained_operations(), &width.fixed_geometry());
+    let segment = segments
+        .iter()
+        .find(|segment| {
+            segment.iter().any(|write| {
+                writes[*write].kind == StepKind::RootRecordFua && {
+                    let (instance, txg) = root_identity_of_write(&writes[*write]);
+                    (instance, txg) == (InstanceGeneration(withheld.0), CheckpointTxg(withheld.1))
+                }
+            })
+        })
+        .unwrap_or_else(|| panic!("这段历史写过实例 {} txg {} 的根槽", withheld.0, withheld.1));
+    assert_eq!(segment.len(), 1, "扣下的那一段只有那一个根槽写");
+    let mut crashed = MemoryPool::with_devices(
+        &[DeviceIdentity(0), DeviceIdentity(1)],
+        width.device_bytes(),
+    );
+    crashed.apply_writes(&writes[..segment[0]]);
+    crashed
+}
+
+/// 草稿用例（删除与并发那条线交来，`a_remount_over_a_root_ring_hole_reddens_an_old_floor_raise`）翻成判绿：单元区 240 槽的小盘，第一个文件之后
+/// 覆盖写 3 次、崩了再挂一次（准入判着，这次挂载里有一条根把 F 从 0 抬到 9），再覆盖写 16 次；第 17 次覆盖写切在「journal 记录已落、txg 31
+/// 那条根没落」，可写挂载一次。挂载把 txg 31 那一版当生效版（从 journal 施加），新实例的根从 txg 32 起写，txg 31 那个根槽一直没写、留着上一圈
+/// 的根，txg 32 起的根盖掉了比它新的几条。挂载前后 I-7.9 都不许红：抬 F 那一刻 F 不高于上限，后来的镜像上从中间少了的根不能把它判成越界。
+#[test]
+fn a_remount_over_a_root_ring_hole_left_by_a_journal_applied_version_keeps_an_old_floor_raise_within_its_ceiling(
+) {
+    let stream = SharedStream::retaining_contents();
+    let wrap = stream.clone();
+    let mut pool: PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>> =
+        PoolUnderTest::start_after_the_first_file(
+            HistoryDeviceWidth::UnitAreaOf240Slots,
+            move |identity, device| {
+                RecordingBlockDevice::with_shared_stream(identity, device, wrap.clone())
+            },
+        );
+    for _ in 0..OVERWRITES_BEFORE_THE_CRASH_REMOUNT {
+        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写");
+    }
+    pool.crash_and_mount_writable().expect("崩了再挂");
+    for _ in 0..OVERWRITES_AFTER_THE_CRASH_REMOUNT {
+        pool.overwrite(OVERWRITE_BYTES).expect("覆盖写");
+    }
+    let before = pool.image();
+    let first_operation = stream.retained_operations().len();
+    pool.overwrite(OVERWRITE_BYTES).expect("第 17 次覆盖写");
+    let operations = stream.retained_operations();
+    let this_publish = &operations[first_operation..];
+    let first_root = this_publish
+        .iter()
+        .position(|operation| {
+            operation.operation.kind == RecordedOperationKind::WriteForceUnitAccess
+                && operation
+                    .contents
+                    .as_ref()
+                    .is_some_and(|bytes| bytes.len() == ROOT_SLOT_BYTES)
+        })
+        .expect("这次发布写根");
+    let (_, txg_of_the_missing_root) = root_identity_written_by(
+        this_publish[first_root]
+            .contents
+            .as_ref()
+            .expect("录制流留着写的内容"),
+    );
+    assert_eq!(
+        txg_of_the_missing_root,
+        CheckpointTxg(TXG_OF_THE_PUBLISH_WHOSE_ROOT_SLOT_IS_WITHHELD_IN_THE_DRAFT)
+    );
+    let mut crashed = before.clone();
+    crashed.apply(&this_publish[..first_root]);
+    assert_eq!(
+        rollback_floor_verdict(&crashed),
+        InvariantVerdict::Holds,
+        "挂载之前 I-7.9 成立"
+    );
+    let mut devices = plain_devices_on(&crashed);
+    let mounted = mount_writable_with_space_admission(
+        &pool.parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+    .expect("再挂做得成");
+    assert_eq!(
+        mounted.output.effective_root.checkpoint_txg,
+        CheckpointTxg(TXG_OF_THE_PUBLISH_WHOSE_ROOT_SLOT_IS_WITHHELD_IN_THE_DRAFT),
+        "生效版是 journal 施加出来的 txg 31"
+    );
+    drop(mounted);
+    let after = MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.image.clone()))
+            .collect(),
+        device_size_in_bytes: crashed.device_size_in_bytes,
+    };
+    assert!(
+        !root_txgs_in_the_ring(&after)
+            .contains(&TXG_OF_THE_PUBLISH_WHOSE_ROOT_SLOT_IS_WITHHELD_IN_THE_DRAFT),
+        "txg 31 那条根一直没写：根环上有洞"
+    );
+    assert!(
+        !matches!(
+            rollback_floor_verdict(&after),
+            InvariantVerdict::Violated(_)
+        ),
+        "挂载之后洞里的旧根顶上来，I-7.9 不许红：{:?}",
+        rollback_floor_verdict(&after)
+    );
+}
+
+/// inject-6 种子 1 那一形（`r3probe ring reuse 4g 20 1 108 0`）：实例 5 txg 25 的根槽写被扣下、它的记录已落，下一次挂载由记录施加出那一版、
+/// 从 txg 26 写起；txg 25 与 txg 1 同槽，txg 1 的根留在洞里，挂载与之后那次发布的根盖掉 2–5。实例 1 txg 9 那条根当初把 F 从 0 抬到 5，
+/// 抬的那一刻上限就是 5：崩溃镜像上 I-7.9 成立，挂载加一次发布之后也不许红。
+#[test]
+fn a_remount_over_a_root_ring_hole_whose_withheld_root_its_record_applies_keeps_an_old_floor_raise_within_its_ceiling(
+) {
+    let crashed = inject_six_history_crashed_on_the_root_slot_write_of(
+        INJECT_SIX_WITHHELD_ROOT_APPLIED_BY_ITS_RECORD,
+    );
+    assert_eq!(
+        rollback_floor_verdict(&crashed),
+        InvariantVerdict::Holds,
+        "崩溃镜像上（环里 1–24 都在）I-7.9 成立"
+    );
+    let after =
+        image_after_a_writable_mount_and_one_publish(&crashed, HistoryDeviceWidth::FourGibibytes);
+    let ring = root_txgs_in_the_ring(&after);
+    assert!(
+        ring.contains(&1) && !ring.contains(&INJECT_SIX_WITHHELD_ROOT_APPLIED_BY_ITS_RECORD.1),
+        "txg 25 那一槽没写过，txg 1 的根留在洞里：{ring:?}"
+    );
+    assert!(
+        ring.contains(&INJECT_SIX_RAISING_ROOT_TXG),
+        "抬 F 的根 txg 9 还在环里：{ring:?}"
+    );
+    assert!(
+        !matches!(
+            rollback_floor_verdict(&after),
+            InvariantVerdict::Violated(_)
+        ),
+        "洞里的 txg 1 顶上来，I-7.9 不许红：{:?}",
+        rollback_floor_verdict(&after)
+    );
+}
+
+/// K1 类洞（`r3probe ring reuse 4g 20 1 113 0`）：同一段历史里扣下的是实例 6 写行那次发布 txg 26 的根槽写——下一次挂载没施加那一版、取号越过它，
+/// txg 26 那一槽留着 txg 2 的根。I-7.9 与上一形同一个机理，挂载加一次发布之后不许红。
+#[test]
+fn a_remount_over_a_root_ring_hole_left_by_a_withheld_row_publish_keeps_an_old_floor_raise_within_its_ceiling(
+) {
+    let crashed = inject_six_history_crashed_on_the_root_slot_write_of(
+        INJECT_SIX_WITHHELD_ROOT_OF_THE_ROW_PUBLISH,
+    );
+    assert_eq!(
+        rollback_floor_verdict(&crashed),
+        InvariantVerdict::Holds,
+        "崩溃镜像上 I-7.9 成立"
+    );
+    let after =
+        image_after_a_writable_mount_and_one_publish(&crashed, HistoryDeviceWidth::FourGibibytes);
+    let ring = root_txgs_in_the_ring(&after);
+    assert!(
+        ring.contains(&2) && !ring.contains(&INJECT_SIX_WITHHELD_ROOT_OF_THE_ROW_PUBLISH.1),
+        "txg 26 那一槽没写过，txg 2 的根留在洞里：{ring:?}"
+    );
+    assert!(
+        !matches!(
+            rollback_floor_verdict(&after),
+            InvariantVerdict::Violated(_)
+        ),
+        "洞里的 txg 2 顶上来，I-7.9 不许红：{:?}",
+        rollback_floor_verdict(&after)
+    );
+}
+
+/// 坏镜像：inject-6 种子 1 那一形挂载加一次发布之后的镜像上，把实例 1 txg 9 那条抬 F 的根带的 F 从 5 改成 7（重封根槽），
+/// 高于一圈窗口里它之前的根算出的上限：I-7.9 照旧红，红在「从 0 抬到 7」那一格。
+#[test]
+fn a_floor_raised_above_the_ceiling_of_the_roots_inside_the_window_still_reddens_the_rollback_floor_invariant(
+) {
+    let crashed = inject_six_history_crashed_on_the_root_slot_write_of(
+        INJECT_SIX_WITHHELD_ROOT_APPLIED_BY_ITS_RECORD,
+    );
+    let mut image =
+        image_after_a_writable_mount_and_one_publish(&crashed, HistoryDeviceWidth::FourGibibytes);
+    rewrite_the_floor_of_the_root_at(&mut image, INJECT_SIX_RAISING_ROOT_TXG, 7);
+    match rollback_floor_verdict(&image) {
+        InvariantVerdict::Violated(detail) => assert!(
+            detail.contains(&format!(
+                "实例 1 txg {INJECT_SIX_RAISING_ROOT_TXG} 那条根把回退下界 F 从 0 抬到 7"
+            )),
+            "红在 txg 9 那条根从 0 抬到 7 上：{detail}"
+        ),
+        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+            panic!("F 抬到 7 高于窗口里的上限，I-7.9 要红，得到 {other:?}")
+        }
+    }
+}
````

## batch9-repin/patch

实现员报告：`/tmp/claude-1000/m3-r1/batch9-repin/patch/report.md`（sha256 9335391966cfccdc…），补丁 sha256 f3eb3b5fb7ab1577…

````diff
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index 5bc40da..56faf95 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -1,4 +1,4 @@
-//! 池级 checker 的输入与第一层判定：读盘的口子、每条不变量的三态、系统配置与根环（I-7 类）、指针与位置条目。
+//! 池级 checker 的输入与第一层判定：读盘的口子、每条不变量的四种判定、系统配置与根环（I-7 类）、指针与位置条目。
 //! 与实现只共享 `singlefs-format` 这一个常量模块（D13（验证路线） 已定项 5）：偏移在这里按字段表各写一份。
 
 use std::collections::BTreeMap;
@@ -33,6 +33,10 @@ pub enum InvariantVerdict {
     Holds,
     Violated(String),
     NotApplicable(&'static str),
+    /// 部分没判（用户 2026-10-04 定，第四种判定）：判到的那一部分都对得上，射程里另有一部分这张镜像上判不了，说明逐条列出没判到的
+    /// 是什么（例如 I-9.28 那几条根：盘号、txg、为什么没判）。**它不是红，也不是成立**：计红、计违例的地方算不红（层 0 与崩溃放量
+    /// 把它算成不红），计「成立」的地方不算成立。今天只有 I-9.28 报它（树表读不出、条目比字段表短的根）。
+    PartiallyJudged(String),
 }
 
 /// 「中央映射条目的 key 与它指的单元头相符」那一条（用户 2026-09-27 定新立，实审 B2），
@@ -213,6 +217,7 @@ pub struct Judgements {
     first_violation: BTreeMap<&'static str, String>,
     not_applicable: BTreeMap<&'static str, &'static str>,
     part_of_the_range_not_judged: BTreeMap<&'static str, &'static str>,
+    partially_judged: BTreeMap<&'static str, String>,
 }
 
 impl Judgements {
@@ -253,6 +258,15 @@ impl Judgements {
             .entry(invariant)
             .or_insert(reason);
     }
+    /// 这条不变量射程里有一部分在这个镜像上判不了、要报「部分没判」（[`InvariantVerdict::PartiallyJudged`]）：`unjudged` 逐条列出没判到的是什么。
+    /// 判出了违例照报违例；没有违例时报部分没判，压过「成立」与「不适用」。只记第一次给的说明。
+    pub fn partially_judged(&mut self, invariant: &'static str, unjudged: String) {
+        assert!(
+            IMPLEMENTED_INVARIANTS.contains(&invariant),
+            "{invariant} 不在第一版 checker 的清单里"
+        );
+        self.partially_judged.entry(invariant).or_insert(unjudged);
+    }
     /// 这条不变量到此为止的第一处违例说明：一次走读前后各看一眼，走读里第一处对不上的是什么（I-7.4 被抛弃根那一半报它）。
     #[must_use]
     pub fn first_violation_of(&self, invariant: &'static str) -> Option<&str> {
@@ -265,6 +279,8 @@ impl Judgements {
             .map(|invariant| {
                 let verdict = if let Some(detail) = self.first_violation.get(invariant) {
                     InvariantVerdict::Violated(detail.clone())
+                } else if let Some(unjudged) = self.partially_judged.get(invariant) {
+                    InvariantVerdict::PartiallyJudged(unjudged.clone())
                 } else if let Some(reason) = self.part_of_the_range_not_judged.get(invariant) {
                     InvariantVerdict::NotApplicable(reason)
                 } else if self.evaluated.get(invariant).copied().unwrap_or(0) > 0 {
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 3967b50..bf5a31c 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -3634,6 +3634,8 @@ struct RootReferences {
     tree_table_birth_txg: BTreeMap<u64, u64>,
     /// 树表单元自己落在哪 (设备身份, 起点槽)：几条根共用同一个树表单元时，它们的条目不是两处独立的证据。
     tree_table_placement: Option<(u32, u64)>,
+    /// 树表读成什么样：I-9.28 按它列出指针非零而条目没读全的根（报「部分没判」）。
+    tree_table_reading: TreeTableReading,
     /// 这条根的分配记录树叶里的记录（I-3.9 只取最新根那一份：当前的账）。
     allocation_records: Vec<AllocationRecordView>,
     /// 这一遍按哪个深度扫的：`placements` 与 `is_complete` 只有「走到叶」那个深度上作数。
@@ -3642,6 +3644,19 @@ struct RootReferences {
     is_complete: bool,
 }
 
+/// 一条根的树表这一遍读成什么样（[`references_of_root`] 只读不判；I-9.28 拿它判「部分没判」）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum TreeTableReading {
+    /// 树表指针全零：这条根没有树表，不是没读全。
+    PointerAllZero,
+    /// 指针非零，两条位置条目都读不出、或整单元校验和对不上、或解不开成码 2 节点。
+    Unreadable,
+    /// 读出来了，有条目比字段表短（那几条不解字段）。
+    SomeEntryShorterThanTheFieldTable,
+    /// 读出来了，每条条目都按字段表解了。
+    EveryEntryRead,
+}
+
 impl RootReferences {
     fn note(&mut self, pointer: &PointerView) {
         if pointer.all_zero {
@@ -3710,6 +3725,7 @@ fn references_of_root(
         placements: BTreeSet::new(),
         tree_table_birth_txg: BTreeMap::new(),
         tree_table_placement: None,
+        tree_table_reading: TreeTableReading::PointerAllZero,
         allocation_records: Vec::new(),
         depth,
         is_complete: true,
@@ -3757,13 +3773,16 @@ fn references_of_root(
     let Some(tree_table) = read_index_node_without_judging(reader, &tree_table_pointer, cache)
     else {
         references.is_complete = false;
+        references.tree_table_reading = TreeTableReading::Unreadable;
         return references;
     };
+    references.tree_table_reading = TreeTableReading::EveryEntryRead;
     for entry in &tree_table.entries {
         // 条目宽是节点头里的一个字段：头校验和过、载荷 CRC 过而条目宽被改窄的镜像也要能判（那由 I-1.7 / I-1.1 说话），
         // 这一遍只如实记「数不全」，不按短条目去解字段。
         if entry.len() < tree_table_entry_bytes() {
             references.is_complete = false;
+            references.tree_table_reading = TreeTableReading::SomeEntryShorterThanTheFieldTable;
             continue;
         }
         let tree = read_u64(entry, 0);
@@ -4569,24 +4588,47 @@ struct TreeTableEntrySightingInTheRing {
 /// 不按实例表判有效与否、不按 F 与窗口滤，含被崩溃恢复抛弃的旧线上的根）各读一遍树表，按树 ID（条目偏移 0）归并，诞生 txg 必须相同。
 /// 判法 checker 自己写（D13（验证路线） 已定项 5），与 I-9.14 的 [`judge_tree_table_birth_txg`] 分开：那一条只比回退候选集。
 ///
-/// 不适用：条款没写，这里取与 I-9.14 同一种——没有一棵树的条目出现在**两个不同的树表单元**里时整条报不适用（几条根共用一个树表单元时
-/// 拿同一份字节比自己是恒真判定，C374（释放代与树表诞生 txg 只有验收断言盯着） 那一类），不报成立。树表指针全零、树表读不出、
-/// 条目比字段表短的那几条根只是少一份证据，不进比较、不另报。
+/// 四种结局（用户 2026-10-04 定「树表读不出算部分没判」、加第四种判定；细则主 agent 定，零轮）：
+/// - 有一棵树在两处记的诞生 txg 不一 ⇒ 违例；
+/// - 都对得上，而有读得出、自证过的根树表指针非零、树表却读不出或有条目比字段表短 ⇒ 部分没判（[`InvariantVerdict::PartiallyJudged`]），
+///   逐条列出这些根（盘号、实例、txg、为什么没判）：它们的条目没进比较，读不出的那一份可能正是对不上的那一份；
+/// - 都没有，也没有一棵树的条目出现在**两个不同的树表单元**里 ⇒ 不适用（几条根共用一个树表单元时拿同一份字节比自己是恒真判定，
+///   C374（释放代与树表诞生 txg 只有验收断言盯着） 那一类），不报成立；
+/// - 其余 ⇒ 成立。
+///
+/// 树表指针全零的根没有树表，不算没判到。
 fn judge_tree_identifier_birth_txg_across_every_readable_root(
     reader: &dyn ImageReader,
+    geometry: &PoolGeometry,
     roots: &[(u64, u64, crate::RootView)],
     cache: &mut IndexNodeCache,
     judgements: &mut Judgements,
 ) {
     let mut sightings_by_tree: BTreeMap<u64, Vec<TreeTableEntrySightingInTheRing>> =
         BTreeMap::new();
-    for (_, _, root) in roots {
+    let mut roots_whose_tree_table_was_not_read_in_full: Vec<String> = Vec::new();
+    for (region, _, root) in roots {
         let references = references_of_root(
             reader,
             &root.record_bytes,
             ReferenceScanDepth::TreeTableOnly,
             cache,
         );
+        let why_not_judged = match references.tree_table_reading {
+            TreeTableReading::PointerAllZero | TreeTableReading::EveryEntryRead => None,
+            TreeTableReading::Unreadable => Some("树表读不出"),
+            TreeTableReading::SomeEntryShorterThanTheFieldTable => Some("树表有条目比字段表短"),
+        };
+        if let Some(why_not_judged) = why_not_judged {
+            // 下标在范围内不是这里判的：`geometry_of` 已经把 R > 3 的槽拒掉，`roots` 的区域号都来自 `root_slot_positions`。
+            let device = geometry.region_devices[usize::try_from(*region).expect(
+                "区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION",
+            )];
+            roots_whose_tree_table_was_not_read_in_full.push(format!(
+                "盘 {device} 上的根（实例 {}、txg {}）：{why_not_judged}",
+                root.instance, root.checkpoint_txg
+            ));
+        }
         let Some(tree_table_placement) = references.tree_table_placement else {
             continue;
         };
@@ -4638,7 +4680,15 @@ fn judge_tree_identifier_birth_txg_across_every_readable_root(
             },
         );
     }
-    if compared_trees == 0 {
+    if !roots_whose_tree_table_was_not_read_in_full.is_empty() {
+        judgements.partially_judged(
+            TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
+            format!(
+                "比到的 {compared_trees} 棵树诞生 txg 都对得上；这几条读得出、自证过的根树表指针非零而条目没读全，没进比较：{}",
+                roots_whose_tree_table_was_not_read_in_full.join("；")
+            ),
+        );
+    } else if compared_trees == 0 {
         judgements.not_applicable(
             TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
             "根环里读得出的根中，没有一棵树的树表条目出现在两个不同的树表单元里：跨根比不出来",
@@ -5288,10 +5338,11 @@ fn rollback_floor_ceiling_before_the_raise(
 /// （里程碑三第 8 批调查的 inject-6 种子 1 那一形）。窗口只锚在被判那条根上挡不住它：洞里那条旧根 t − 3S（t 是洞的 txg、t 晚于抬 F 那条根）
 /// 落在那个窗口里。再按这张镜像的最新有效根 N 取一圈窗口就挡住了：洞里留下的根都 ≤ N − 3S；而窗口里 txg 比抬 F 那条根小的根在抬 F 那一刻
 /// 就在环里、之后不会被盖（盖它要一条 txg ≥ 它 + 3S > N 的根），所以留下的是抬 F 那一刻那批根去掉最旧的一段前缀，重算只会更宽。
-/// 带卸载记号那一支照条款不取窗口（I-7.9 那一行写明「根环里 txg 比它小的最新那条根」照旧），同一个机理在那一支上照样会误红。
+/// 带卸载记号那一支同一个机理：洞里那条旧根顶上来、比它新的前一条已被盖掉，「txg 比它小的最新那条根」变旧、在合法镜像上判红
+/// （随机历史种子 31045、1037 那两段，9-6 报告第六节 Q1）；两支都取同一个交（用户 2026-10-04 定「乙，两支都取两个窗口」）。
 ///
 /// 按这条根 flags 里的卸载记号分两支（D22（单元原子性怎么合成） 已定项 7，用户 2026-09-26 定 (b)+(c) 合用）：
-/// 带记号的（正常卸载那一串）只判 F 不高于根环里 txg 比它小的最新那条根的 txg；不带的（准入抬 F）照下面的准入上限判。
+/// 带记号的（正常卸载那一串）只判 F 不高于它之前的根（两个窗口的交）里 txg 最大那条的 txg，交是空的不判；不带的（准入抬 F）照下面的准入上限判。
 ///
 /// 不带记号那一支三种结局：F ≤ 上限的下沿 ⇒ 成立；F > 上限的上沿 ⇒ 违例；落在两沿之间、或上限无从算起 ⇒ 这条根不判。
 /// 一条根都没判到时整条报不适用并带理由，不报成立。
@@ -5325,14 +5376,27 @@ fn judge_rollback_floor_raises_against_their_ceilings(
         let raised_floor = raising_root.rollback_floor;
         match raising_root.unmount_marker {
             // 带卸载记号那一支：正常卸载把 F 抬到现行那一版的 txg、不另判上限（D16（发布语义） 已定项 1「抬 F 的上限」），
-            // 判 F 不高于根环里 txg 比它小的最新那条根的 txg。记号是不是只由卸载入口打，盘上没有别的东西核它，归层 0（C557）。
+            // 判 F 不高于它之前的根里 txg 最大的那条的 txg。「之前的根」与准入那一支取同一个交：txg 比它小、落在锚在它自己 txg 上的
+            // 一圈窗口里、也落在锚在这张镜像最新有效根上的一圈窗口里（用户 2026-10-04 定「乙，两支都取两个窗口」）；交是空的
+            // （镜像比它新出一圈以上、它前面那几条已被盖掉）这条根不判。记号是不是只由卸载入口打，盘上没有别的东西核它，归层 0（C557）。
             crate::UnmountMarkerView::WrittenByTheUnmountSequence => {
-                let newest_root_txg_before_the_raise = roots
+                let window_anchored_at_the_raising_root = OneRevolutionWindow {
+                    anchor_txg: raising_txg,
+                    root_ring_slot_count: geometry.regions * geometry.slots_per_region,
+                };
+                let Some(newest_root_txg_before_the_raise) = roots
                     .iter()
                     .map(|(_, _, root)| root.checkpoint_txg)
-                    .filter(|txg| *txg < raising_txg)
+                    .filter(|txg| {
+                        *txg < raising_txg
+                            && window_anchored_at_the_raising_root.contains(*txg)
+                            && window_anchored_at_the_newest_root_of_the_image.contains(*txg)
+                    })
                     .max()
-                    .expect("同一实例里它的前一条根就在根环里、txg 比它小（上面 let-else 找到的那一条）");
+                else {
+                    raising_roots_not_judged += 1;
+                    continue;
+                };
                 raising_roots_judged += 1;
                 judgements.judge(
                     "I-7.9",
@@ -5381,7 +5445,7 @@ fn judge_rollback_floor_raises_against_their_ceilings(
             if raising_roots_not_judged == 0 {
                 "根环里没有抬 F 的根：每条根带的 F 都不高于同一实例里它前一条根带的（回退与新实例的第一条根不算抬）"
             } else {
-                "根环里抬 F 的根都判不了：它自己指着的实例表读不出、它之前一条有效根都没有，或有效根的树表读不出、上限落在它带的 F 两边"
+                "根环里抬 F 的根都判不了：它自己指着的实例表读不出、它之前一条有效根都没有，或有效根的树表读不出、上限落在它带的 F 两边，或带卸载记号、两个一圈窗口的交里没有它之前的根"
             },
         );
     }
@@ -8231,6 +8295,7 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
     );
     judge_tree_identifier_birth_txg_across_every_readable_root(
         reader,
+        &geometry,
         &roots,
         &mut index_node_cache,
         &mut judgements,
diff --git a/crates/singlefs-checker-tier/src/bad_disk_input.rs b/crates/singlefs-checker-tier/src/bad_disk_input.rs
index 2feb4c0..a51e3c2 100644
--- a/crates/singlefs-checker-tier/src/bad_disk_input.rs
+++ b/crates/singlefs-checker-tier/src/bad_disk_input.rs
@@ -1312,12 +1312,16 @@ pub fn feed_a_damaged_image(
                 .iter()
                 .filter_map(|(invariant, verdict)| match verdict {
                     InvariantVerdict::Violated(_) => Some(*invariant),
-                    InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+                    InvariantVerdict::Holds
+                    | InvariantVerdict::NotApplicable(_)
+                    | InvariantVerdict::PartiallyJudged(_) => None,
                 })
                 .collect();
             let judged_nothing = report.iter().all(|(_, verdict)| match verdict {
                 InvariantVerdict::NotApplicable(_) => true,
-                InvariantVerdict::Holds | InvariantVerdict::Violated(_) => false,
+                InvariantVerdict::Holds
+                | InvariantVerdict::Violated(_)
+                | InvariantVerdict::PartiallyJudged(_) => false,
             });
             (
                 ReaderOutcome::Finished {
diff --git a/crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs b/crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs
index 9dd9d8b..fb6c57a 100644
--- a/crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs
+++ b/crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs
@@ -694,6 +694,7 @@ fn verdict_label(verdict: &InvariantVerdict) -> &'static str {
         InvariantVerdict::Holds => "holds",
         InvariantVerdict::Violated(_) => "violated",
         InvariantVerdict::NotApplicable(_) => "not_applicable",
+        InvariantVerdict::PartiallyJudged(_) => "partially_judged",
     }
 }
 
diff --git a/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs b/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
index 1c18240..fd1e241 100644
--- a/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
+++ b/crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs
@@ -18627,7 +18627,9 @@ mod fourth_run_segment_two {
                         }
                     }
                 }
-                InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => {}
+                InvariantVerdict::Holds
+                | InvariantVerdict::NotApplicable(_)
+                | InvariantVerdict::PartiallyJudged(_) => {}
             }
         }
         reading
diff --git a/crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs b/crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs
index 14de5a5..0e08fb2 100644
--- a/crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs
+++ b/crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs
@@ -2088,7 +2088,8 @@ fn evaluate_baseline(
                 *tally.checker_evaluated_states.entry(invariant).or_insert(0) += 1;
                 *tally.checker_violated_states.entry(invariant).or_insert(0) += 1;
             }
-            InvariantVerdict::NotApplicable(_) => {
+            // 部分没判不红、不算评估过：与层 0 那一路（`crash.rs`）同一个口径，计进不适用这一格。
+            InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => {
                 *tally
                     .checker_not_applicable_states
                     .entry(invariant)
diff --git a/crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs b/crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs
index 24c8dac..b5d2079 100644
--- a/crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs
+++ b/crates/singlefs-checker-tier/src/bin/e168_delete_ring_window_borrow.rs
@@ -1257,7 +1257,9 @@ fn checker_violations(image: &MemoryPool) -> Vec<String> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some(format!("{invariant}:{detail}")),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs b/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
index 2e231ce..1f02ecc 100644
--- a/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
+++ b/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
@@ -1086,7 +1086,9 @@ impl ImplementationVerdict {
                 .iter()
                 .filter(|(_, verdict)| match verdict {
                     InvariantVerdict::Violated(_) => true,
-                    InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => false,
+                    InvariantVerdict::Holds
+                    | InvariantVerdict::NotApplicable(_)
+                    | InvariantVerdict::PartiallyJudged(_) => false,
                 })
                 .map(|(invariant, _)| (*invariant).to_string())
                 .collect(),
diff --git a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs
index c3c34ed..354fc4f 100644
--- a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs
+++ b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs
@@ -1746,7 +1746,9 @@ impl ImplementationVerdict {
                 .iter()
                 .filter(|(_, verdict)| match verdict {
                     InvariantVerdict::Violated(_) => true,
-                    InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => false,
+                    InvariantVerdict::Holds
+                    | InvariantVerdict::NotApplicable(_)
+                    | InvariantVerdict::PartiallyJudged(_) => false,
                 })
                 .map(|(invariant, _)| (*invariant).to_string())
                 .collect(),
@@ -1818,7 +1820,9 @@ fn violated_invariant_names(verdicts: &[(&'static str, InvariantVerdict)]) -> Ve
         .iter()
         .filter(|(_, verdict)| match verdict {
             InvariantVerdict::Violated(_) => true,
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => false,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => false,
         })
         .map(|(invariant, _)| (*invariant).to_string())
         .collect()
diff --git a/crates/singlefs-checker-tier/src/crash.rs b/crates/singlefs-checker-tier/src/crash.rs
index 14e7572..dc53ef8 100644
--- a/crates/singlefs-checker-tier/src/crash.rs
+++ b/crates/singlefs-checker-tier/src/crash.rs
@@ -1166,7 +1166,9 @@ impl Layer0StateJudgement {
                 InvariantVerdict::Violated(_) => {
                     items.push(format!("{LAYER0_RED_PASS_POOL_CHECKER}:{invariant}"))
                 }
-                InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => {}
+                InvariantVerdict::Holds
+                | InvariantVerdict::NotApplicable(_)
+                | InvariantVerdict::PartiallyJudged(_) => {}
             }
         }
         if self.records.root_without_record {
@@ -1202,7 +1204,9 @@ impl Layer0StateJudgement {
             .iter()
             .filter_map(|(invariant, verdict)| match verdict {
                 InvariantVerdict::Violated(detail) => Some((*invariant, detail.clone())),
-                InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+                InvariantVerdict::Holds
+                | InvariantVerdict::NotApplicable(_)
+                | InvariantVerdict::PartiallyJudged(_) => None,
             })
             .collect();
         if !violated.is_empty() {
@@ -1484,7 +1488,9 @@ fn evaluate_state_recording_findings(
                     .or_insert_with(|| detail.clone());
                 violated_invariants.push((invariant, detail));
             }
-            InvariantVerdict::NotApplicable(_) => {
+            // 部分没判（`InvariantVerdict::PartiallyJudged`）不红、也不算评估过（判到的那一部分成立不等于整条成立）：计进不适用这一格，
+            // 「评估过 + 不适用 = 状态数」照旧成立，进度文件的格式也不多一格。
+            InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => {
                 *tally
                     .checker_not_applicable_states
                     .entry(invariant)
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index 4c526c0..d5d693c 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -502,6 +502,8 @@ pub struct CrashInjectionTally {
     pub checker_runs: u64,
     pub invariant_holds: BTreeMap<&'static str, u64>,
     pub invariant_not_applicable: BTreeMap<&'static str, u64>,
+    /// 判成「部分没判」（`InvariantVerdict::PartiallyJudged`）的次数：不红，也不算判绿。
+    pub invariant_partially_judged: BTreeMap<&'static str, u64>,
     /// 记录核对器跑过几次、两类各判红几次（与层 0 那一路同一份判据，`crash::check_records_against`）。
     pub record_checks: u64,
     pub record_root_without_record: u64,
@@ -595,6 +597,7 @@ impl CrashInjectionTally {
             checker_runs,
             invariant_holds,
             invariant_not_applicable,
+            invariant_partially_judged,
             record_checks,
             record_root_without_record,
             record_claimed_state_missing_unit,
@@ -677,6 +680,12 @@ impl CrashInjectionTally {
         for (invariant, count) in invariant_not_applicable {
             *self.invariant_not_applicable.entry(invariant).or_insert(0) += count;
         }
+        for (invariant, count) in invariant_partially_judged {
+            *self
+                .invariant_partially_judged
+                .entry(invariant)
+                .or_insert(0) += count;
+        }
         self.record_checks += record_checks;
         self.record_root_without_record += record_root_without_record;
         self.record_claimed_state_missing_unit += record_claimed_state_missing_unit;
@@ -969,9 +978,13 @@ impl CrashInjectionTally {
         for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
             let _ = writeln!(
                 text,
-                "  {invariant}：判绿 {} 次、不适用 {} 次",
+                "  {invariant}：判绿 {} 次、不适用 {} 次、部分没判 {} 次",
                 self.invariant_holds.get(invariant).copied().unwrap_or(0),
                 self.invariant_not_applicable
+                    .get(invariant)
+                    .copied()
+                    .unwrap_or(0),
+                self.invariant_partially_judged
                     .get(invariant)
                     .copied()
                     .unwrap_or(0)
@@ -3492,6 +3505,12 @@ fn checker_violations_on(
             InvariantVerdict::NotApplicable(_) => {
                 *tally.invariant_not_applicable.entry(invariant).or_insert(0) += 1;
             }
+            InvariantVerdict::PartiallyJudged(_) => {
+                *tally
+                    .invariant_partially_judged
+                    .entry(invariant)
+                    .or_insert(0) += 1;
+            }
             InvariantVerdict::Violated(detail) => violations.push((invariant, detail)),
         }
     }
@@ -3505,7 +3524,9 @@ fn checker_violations_of(image: &dyn ImageReader) -> Vec<(&'static str, String)>
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs
index 88c60f8..5bc356c 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs
@@ -260,7 +260,9 @@ fn rejudge_state(
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(_detail) => Some(invariant),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect();
     if !violated_invariants.is_empty() {
diff --git a/crates/singlefs-checker-tier/tests/crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs b/crates/singlefs-checker-tier/tests/crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs
index aed1d0c..2e0c349 100644
--- a/crates/singlefs-checker-tier/tests/crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs
+++ b/crates/singlefs-checker-tier/tests/crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs
@@ -490,7 +490,9 @@ fn checker_is_silent_on(image: &dyn singlefs_checker::image::ImageReader) -> boo
         .into_iter()
         .all(|(_, verdict)| match verdict {
             InvariantVerdict::Violated(_) => false,
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => true,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => true,
         })
 }
 
diff --git a/crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs b/crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs
index 5911bcf..8cac46f 100644
--- a/crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs
+++ b/crates/singlefs-checker-tier/tests/crash_points_of_the_instance_table_sweep.rs
@@ -596,21 +596,23 @@ fn every_crash_state_inside_the_two_sweep_phases_remounts_writable_and_ends_with
                         };
                         let mut after = image.clone();
                         after.apply(&stream.retained_operations());
-                        let violations: Vec<String> = singlefs_checker::walk::check_pool_image(
-                            &after,
-                        )
-                        .into_iter()
-                        .filter(|(invariant, _)| {
-                            INVARIANTS_JUDGED_AFTER_THE_REMOUNT.contains(invariant)
-                        })
-                        .filter_map(|(invariant, verdict)| match verdict {
-                            singlefs_checker::image::InvariantVerdict::Violated(detail) => {
-                                Some(format!("{invariant}：{detail}"))
-                            }
-                            singlefs_checker::image::InvariantVerdict::Holds
-                            | singlefs_checker::image::InvariantVerdict::NotApplicable(_) => None,
-                        })
-                        .collect();
+                        let violations: Vec<String> =
+                            singlefs_checker::walk::check_pool_image(&after)
+                                .into_iter()
+                                .filter(|(invariant, _)| {
+                                    INVARIANTS_JUDGED_AFTER_THE_REMOUNT.contains(invariant)
+                                })
+                                .filter_map(|(invariant, verdict)| match verdict {
+                                    singlefs_checker::image::InvariantVerdict::Violated(detail) => {
+                                        Some(format!("{invariant}：{detail}"))
+                                    }
+                                    singlefs_checker::image::InvariantVerdict::Holds
+                                    | singlefs_checker::image::InvariantVerdict::NotApplicable(_)
+                                    | singlefs_checker::image::InvariantVerdict::PartiallyJudged(
+                                        _,
+                                    ) => None,
+                                })
+                                .collect();
                         if !violations.is_empty() {
                             failures.push(format!("再挂之后 checker 红：{violations:?}"));
                         }
diff --git a/crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs b/crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs
index f073372..6ba68f1 100644
--- a/crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs
+++ b/crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs
@@ -277,7 +277,9 @@ fn sweep_every_crash_inside_the_publish(
                 .into_iter()
                 .filter_map(|(invariant, verdict)| match verdict {
                     InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
-                    InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+                    InvariantVerdict::Holds
+                    | InvariantVerdict::NotApplicable(_)
+                    | InvariantVerdict::PartiallyJudged(_) => None,
                 })
                 .collect();
             if !violated.is_empty() {
diff --git a/crates/singlefs-checker-tier/tests/gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs b/crates/singlefs-checker-tier/tests/gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs
index f457b2e..563608f 100644
--- a/crates/singlefs-checker-tier/tests/gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs
+++ b/crates/singlefs-checker-tier/tests/gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs
@@ -170,7 +170,9 @@ fn invariants_violated_with_every_write_landed(flow: &RecordedFlow) -> BTreeSet<
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(_) => Some(invariant),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
@@ -187,7 +189,9 @@ fn pool_checker_bits_on_the_cpu(judgement: &Layer0StateJudgement) -> u128 {
                     .position(|candidate| candidate == invariant)
                     .expect("池级 checker 报的不变量在清单里"),
             ),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .fold(0u128, |bits, index| bits | (1u128 << index))
 }
diff --git a/crates/singlefs-checker-tier/tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs b/crates/singlefs-checker-tier/tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs
index 9f39b2b..c1ee026 100644
--- a/crates/singlefs-checker-tier/tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs
+++ b/crates/singlefs-checker-tier/tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs
@@ -206,7 +206,9 @@ fn violations_with_every_write_landed(flow: &RecordedFlow) -> BTreeMap<&'static
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
@@ -223,7 +225,9 @@ fn pool_checker_bits_on_the_cpu(judgement: &Layer0StateJudgement) -> u128 {
                     .position(|candidate| candidate == invariant)
                     .expect("池级 checker 报的不变量在清单里"),
             ),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .fold(0u128, |bits, index| bits | (1u128 << index))
 }
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 8448274..a8494c7 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -1096,8 +1096,9 @@ impl RootRingOccupancy {
     }
 
     /// 进程内重读那几个槽时判「按实例表仍然有效」用的那张表（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」：按实例表判仍然有效）。
-    /// 挂载时装的是最新那条根的那一张（`mount::rebuilt_allocator`），写行那次发布的表一定下来就换成它（`mount::establish_instance`）——
-    /// 池级 checker 按镜像里最新那条根的实例表判，写行之后最新那条根带的就是写行那一张。mkfs 同一个进程里装的是空表（表里没有挂载时读不出的槽，用不到它）。
+    /// 装上根环表时是空表；可写挂载在写行那次发布的表一定下来、第一次记根之前换成它（`mount::establish_instance`）——
+    /// 池级 checker 按镜像里最新那条根的实例表判，写行之后最新那条根带的就是写行那一张。挂载装上根环表到换表之间不记根，空表没人读；
+    /// mkfs 同一个进程里一直是空表（表里没有挂载时读不出的槽，用不到它）。
     #[must_use]
     pub fn instance_table_judging_the_rereads(&self) -> &InstanceTableRecords {
         &self.instance_table_judging_the_rereads
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 7bae1a9..d5d1e3a 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -1920,8 +1920,6 @@ fn rebuilt_allocator<Device: BlockDevice>(
                 .expect("新实例第一次发布的 txg = 环里最大 txg + 1 ≥ 1"),
         ),
     ));
-    // 进程内重读挂载时读不出的槽（E2）判有效用的表：写行那次发布的表定下来之前先用最新那条根的这一张（`establish_instance` 换掉它）。
-    allocator.note_the_instance_table_judging_the_rereads_of_root_ring_slots(newest_table.clone());
     Ok(RebuiltAllocator {
         allocator,
         effective_floor,
diff --git a/crates/singlefs-harness/src/fault_injection.rs b/crates/singlefs-harness/src/fault_injection.rs
index 2ed8571..fc19fc5 100644
--- a/crates/singlefs-harness/src/fault_injection.rs
+++ b/crates/singlefs-harness/src/fault_injection.rs
@@ -1516,6 +1516,8 @@ pub struct FaultInjectionTally {
     pub checker_runs_on_the_image_after_the_fault: u64,
     pub invariant_holds: BTreeMap<&'static str, u64>,
     pub invariant_not_applicable: BTreeMap<&'static str, u64>,
+    /// 判成「部分没判」（[`InvariantVerdict::PartiallyJudged`]）的条次：不红，也不算判绿。
+    pub invariant_partially_judged: BTreeMap<&'static str, u64>,
     pub faults_ending_known_red: BTreeMap<KnownRedFormAtAFault, u64>,
     pub faults_ending_new_finding: u64,
 }
@@ -1590,6 +1592,9 @@ impl FaultInjectionTally {
         for (key, count) in &other.invariant_not_applicable {
             *self.invariant_not_applicable.entry(key).or_insert(0) += count;
         }
+        for (key, count) in &other.invariant_partially_judged {
+            *self.invariant_partially_judged.entry(key).or_insert(0) += count;
+        }
         for (form, count) in &other.faults_ending_known_red {
             *self.faults_ending_known_red.entry(*form).or_insert(0) += count;
         }
@@ -1720,10 +1725,11 @@ impl FaultInjectionTally {
         );
         let _ = writeln!(
             text,
-            "注入之后的镜像上跑池级 checker {} 次：判绿 {} 条次、不适用 {} 条次",
+            "注入之后的镜像上跑池级 checker {} 次：判绿 {} 条次、不适用 {} 条次、部分没判 {} 条次",
             self.checker_runs_on_the_image_after_the_fault,
             self.invariant_holds.values().sum::<u64>(),
-            self.invariant_not_applicable.values().sum::<u64>()
+            self.invariant_not_applicable.values().sum::<u64>(),
+            self.invariant_partially_judged.values().sum::<u64>()
         );
         let _ = writeln!(
             text,
@@ -1880,6 +1886,12 @@ fn checker_violations_on(
             InvariantVerdict::NotApplicable(_) => {
                 *tally.invariant_not_applicable.entry(invariant).or_insert(0) += 1;
             }
+            InvariantVerdict::PartiallyJudged(_) => {
+                *tally
+                    .invariant_partially_judged
+                    .entry(invariant)
+                    .or_insert(0) += 1;
+            }
             InvariantVerdict::Violated(detail) => violations.push((invariant, detail)),
         }
     }
@@ -2618,7 +2630,9 @@ fn inject_one_fault(
                 .into_iter()
                 .filter_map(|(invariant, verdict)| match verdict {
                     InvariantVerdict::Violated(_) => Some(invariant),
-                    InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+                    InvariantVerdict::Holds
+                    | InvariantVerdict::NotApplicable(_)
+                    | InvariantVerdict::PartiallyJudged(_) => None,
                 })
                 .collect();
         let swallowed_root_slot_writes: Vec<&SwallowedWrite> = swallowed_writes
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index 5542ee3..f109d09 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -2189,6 +2189,8 @@ pub struct HistoryTally {
     pub checker_runs_skipped_because_nothing_was_written: u64,
     pub invariant_holds: BTreeMap<&'static str, u64>,
     pub invariant_not_applicable: BTreeMap<&'static str, u64>,
+    /// 判成「部分没判」（[`InvariantVerdict::PartiallyJudged`]）的次数：不红，也不算判绿。
+    pub invariant_partially_judged: BTreeMap<&'static str, u64>,
     /// 理想模型判过的步数（起点那次第一个文件也算一步；前提不满足、入口没调的不算）与各格计数。
     pub model_judged_steps: u64,
     pub model_counts: ModelJudgementCounts,
@@ -2276,6 +2278,12 @@ impl HistoryTally {
         for (invariant, count) in &other.invariant_not_applicable {
             *self.invariant_not_applicable.entry(invariant).or_insert(0) += count;
         }
+        for (invariant, count) in &other.invariant_partially_judged {
+            *self
+                .invariant_partially_judged
+                .entry(invariant)
+                .or_insert(0) += count;
+        }
         self.model_judged_steps += other.model_judged_steps;
         self.model_counts.add(&other.model_counts);
     }
@@ -2534,9 +2542,13 @@ impl HistoryTally {
         for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
             let _ = writeln!(
                 text,
-                "  {invariant}：判绿 {} 次、不适用 {} 次",
+                "  {invariant}：判绿 {} 次、不适用 {} 次、部分没判 {} 次",
                 self.invariant_holds.get(invariant).copied().unwrap_or(0),
                 self.invariant_not_applicable
+                    .get(invariant)
+                    .copied()
+                    .unwrap_or(0),
+                self.invariant_partially_judged
                     .get(invariant)
                     .copied()
                     .unwrap_or(0)
@@ -4713,6 +4725,12 @@ fn violations_on(image: &MemoryPool, tally: &mut HistoryTally) -> Vec<(&'static
             InvariantVerdict::NotApplicable(_) => {
                 *tally.invariant_not_applicable.entry(invariant).or_insert(0) += 1;
             }
+            InvariantVerdict::PartiallyJudged(_) => {
+                *tally
+                    .invariant_partially_judged
+                    .entry(invariant)
+                    .or_insert(0) += 1;
+            }
             InvariantVerdict::Violated(detail) => violations.push((invariant, detail)),
         }
     }
diff --git a/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs b/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs
index 6444e89..3d16c34 100644
--- a/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs
+++ b/crates/singlefs-harness/tests/a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs
@@ -166,7 +166,9 @@ fn violated_invariants(pool: &BuiltPool) -> Vec<(&'static str, String)> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/accounting_and_mapping_tree_split.rs b/crates/singlefs-harness/tests/accounting_and_mapping_tree_split.rs
index 039058b..a9ca57a 100644
--- a/crates/singlefs-harness/tests/accounting_and_mapping_tree_split.rs
+++ b/crates/singlefs-harness/tests/accounting_and_mapping_tree_split.rs
@@ -67,7 +67,9 @@ fn violations(image: &MemoryPool) -> Vec<(&'static str, String)> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs b/crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs
index 83855e6..55728b4 100644
--- a/crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs
+++ b/crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs
@@ -564,7 +564,9 @@ fn writable_mount_short_of_its_instance_switch_reserve_raises_the_floor_after_th
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect();
     assert!(
diff --git a/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs b/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
index eb1ea48..569f87b 100644
--- a/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
+++ b/crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs
@@ -2352,12 +2352,16 @@ fn known_root_slot_still_bad_after_one_reread_refuses_the_floor_raise_before_any
     assert!(*current == current_before, "现行那一版不动");
 }
 
-/// 挂载那一刻就坏的槽照旧当没有根、不挡抬 F（C562 那一行的前半句）：同一个池进程退出，(区域 0, 槽 0) 从这时起每一次读都报错，
-/// 再可写挂载——挂载读根环时它就读不出，这个进程的根环表里没有它。之后抬 F 到上限：算上限时它读不出、当没有根，最旧的有效根是暖机那条 txg 1，
-/// 上限 1，F 抬到 1（`RaiseToTheAdmissionCeiling::Raised`），不因为这一槽被拒。
-/// 判别力：读坏的槽不分挂载时读不读得出、一律重读仍坏就拒时，这次抬 F 被拒。
+/// 挂载那一刻就读不出、这个进程之后没写过的槽，算准入抬 F 的上限时当作可能是一条非空有效根、F 不越过它
+/// （D16（发布语义） 已定项 1「抬 F 的上限」那一行，用户 2026-10-04 定 RS3；Q5：只并读不出的槽）：同一个池进程退出，(区域 0, 槽 0)
+/// 从这时起每一次读都报错，再可写挂载——挂载读根环时它就读不出，这个进程知道住着根的槽里没有它，根环表把它记成窗口里读不出的槽、
+/// 按它能住的最大 txg 记：那一槽在一圈窗口里只能住 txg 0（mkfs 那条根；下一次轮到它是 txg 3S，还没写到）。之后抬 F 到上限：
+/// 读得出的根算出的上限是 1（最旧的有效根是暖机那条 txg 1），并进那一槽的 0 之后上限是 0、不高于现行的 F 0——
+/// 报 `FloorAlreadyAtTheCeiling { floor: 0, ceiling: 0 }`，一个字节都不写；也不因为这一槽重读仍坏就拒（它不是这个进程知道住着根的槽）。
+/// 判别力：不并这一槽时上限 1、F 抬到 1；挂载那一刻读不出的槽也重读、仍坏就拒时报 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`。
 #[test]
-fn root_slot_already_unreadable_at_mount_counts_as_no_root_and_does_not_block_the_floor_raise() {
+fn root_slot_already_unreadable_at_mount_is_taken_as_a_possible_root_and_holds_the_admission_ceiling_at_its_txg(
+) {
     let (mut pool, plan) = pool_for_the_ceiling_reading();
     plan.arm(schedule_on_the_genesis_root_slot(
         &pool.parameters,
@@ -2368,8 +2372,9 @@ fn root_slot_already_unreadable_at_mount_counts_as_no_root_and_does_not_block_th
     assert!(
         !ring_slots_known_to_hold_a_root_by(&pool.session().allocator)
             .contains(&RootRingSlot { region: 0, slot: 0 }),
-        "挂载那一刻读不出的槽不在这个进程的根环表里"
+        "挂载那一刻读不出的槽不在这个进程知道住着根的槽里"
     );
+    let image_before = pool.image();
     let mut session = pool.session.take().expect("挂载的会话");
     let PoolVersion::WithFile(current) = &mut session.current else {
         panic!("挂载之后现行那一版带文件");
@@ -2382,21 +2387,15 @@ fn root_slot_already_unreadable_at_mount_counts_as_no_root_and_does_not_block_th
         ShadowLedger::On,
     );
     plan.disarm();
-    let Ok(RaiseToTheAdmissionCeiling::Raised(raised)) = raised else {
-        panic!("挂载时就坏的槽不挡抬 F：{raised:?}");
+    let Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling }) = raised else {
+        panic!("挂载时就读不出的槽按它能住的 txg 0 封顶上限、F 不越过它：{raised:?}");
     };
     assert_eq!(
-        raised.ceiling,
-        CheckpointTxg(1),
-        "最旧的有效根是暖机那条 txg 1"
-    );
-    assert!(
-        raised
-            .publishes
-            .iter()
-            .all(|publish| publish.root.rollback_floor == CheckpointTxg(1)),
-        "这一串的根都带新 F = 1"
+        (floor, ceiling),
+        (CheckpointTxg(0), CheckpointTxg(0)),
+        "上限取读得出的根算出的 1 与那一槽能住的 0 里小的那个"
     );
+    assert!(pool.image() == image_before, "没有可抬的：两块盘逐字节不变");
 }
 
 /// 实四丙的构造做成用例（`records/2026-09-24-里程碑二收尾调度.md` 第三节「实四丙交回」那一行）：故障注入的种子基 + 339 那段历史
diff --git a/crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs b/crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs
index 44ec45c..35b24f5 100644
--- a/crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs
+++ b/crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs
@@ -70,7 +70,9 @@ fn violations(image: &MemoryPool) -> Vec<(&'static str, String)> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
@@ -469,7 +471,9 @@ fn a_journal_record_whose_counter_is_zero_is_judged_without_a_panic() {
             detail.contains("计数器 0"),
             "I-8.6 的说明要点出是计数器 0 那一条：{detail}"
         ),
-        other_verdict @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("计数器 0 那一条没有本实例内逻辑前一条，I-8.6 该判违例，实际 {other_verdict:?}")
         }
     }
@@ -588,7 +592,9 @@ fn a_device_reporting_an_end_before_the_unit_area_start_is_judged_without_a_pani
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect();
     assert!(
diff --git a/crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs b/crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs
index 61cb74f..e97379f 100644
--- a/crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs
+++ b/crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs
@@ -107,7 +107,9 @@ fn assert_violated_mentioning(image: &MemoryPool, invariant: &str, expected_in_t
             detail.contains(expected_in_the_detail),
             "{invariant} 判违例，说明里该有「{expected_in_the_detail}」：{detail}"
         ),
-        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("{invariant} 应当判违例（说明里有「{expected_in_the_detail}」），实际 {other:?}")
         }
     }
diff --git a/crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs b/crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs
index 0c87f89..28a4d05 100644
--- a/crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs
+++ b/crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs
@@ -243,7 +243,9 @@ fn an_inode_tree_root_with_a_non_zero_encryption_reserved_byte_breaks_the_walk_o
             detail.contains("头用不了"),
             "最新根走不完，断在 inode 树根头用不了那一步：{detail}"
         ),
-        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("inode 树根的预留位非 0，最新根不该算走得完：{other:?}")
         }
     }
@@ -672,7 +674,9 @@ fn every_unit_class_with_an_unrecognized_format_version_or_a_non_zero_reserved_b
                     detail.contains(expected_in_the_detail),
                     "{identity:?}：I-2.4 的说明要点名{expected_in_the_detail}：{detail}"
                 ),
-                other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+                other @ (InvariantVerdict::Holds
+                | InvariantVerdict::NotApplicable(_)
+                | InvariantVerdict::PartiallyJudged(_)) => {
                     panic!("{identity:?} 的{expected_in_the_detail}改坏、两道校验和都对：池级走读要报 I-2.4，得到 {other:?}")
                 }
             }
@@ -681,7 +685,9 @@ fn every_unit_class_with_an_unrecognized_format_version_or_a_non_zero_reserved_b
                     detail.contains("头用不了"),
                     "{identity:?}：最新根走不完，断在头用不了那一步：{detail}"
                 ),
-                other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+                other @ (InvariantVerdict::Holds
+                | InvariantVerdict::NotApplicable(_)
+                | InvariantVerdict::PartiallyJudged(_)) => {
                     panic!("{identity:?} 的头用不了，最新根不该算走得完：{other:?}")
                 }
             }
@@ -826,7 +832,7 @@ fn a_followed_pointer_whose_mac_or_nonce_is_not_zero_reddens_the_header_invarian
                     detail.contains(expected_in_the_detail) && detail.contains("MAC 16 / nonce 12"),
                     "{followed_pointer:?} 头部第 {byte_in_pointer_head} 字节是 1：I-2.4 的说明要点名{expected_in_the_detail}：{detail}"
                 ),
-                other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => panic!(
+                other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_)) => panic!(
                     "{followed_pointer:?} 头部第 {byte_in_pointer_head} 字节是 1（加密关着时恒 0）：池级走读要报 I-2.4，得到 {other:?}"
                 ),
             }
@@ -867,7 +873,7 @@ fn a_node_of_a_registered_tree_whose_entry_width_is_zero_while_its_entry_count_i
             detail.contains("条目宽 0 而条目数不是 0"),
             "inode 树（登记了条目字段表 120）的根条目宽 0、条目数 3：I-1.10 的说明要点名宽 0：{detail}"
         ),
-        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => panic!(
+        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_)) => panic!(
             "登记树的节点条目宽 0 而条目数非 0：0 不是任何登记宽，要报 I-1.10，得到 {other:?}"
         ),
     }
@@ -940,7 +946,7 @@ fn a_pool_where_one_system_configuration_slot_carries_a_refused_value_is_reporte
                     detail.contains(&format!("盘 1 偏移 {offset}")),
                     "只有盘 1 偏移 {offset} 那一槽是「{what}」：违例要点名那一槽：{detail}"
                 ),
-                other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => panic!(
+                other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_)) => panic!(
                     "只有盘 1 偏移 {offset} 那一槽是「{what}」：实现整池拒绝挂载，checker 要报违例，得到 {other:?}"
                 ),
             }
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index d9da83e..315e404 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -1859,7 +1859,9 @@ fn transaction_boundary_judgement(
         .filter(|(name, _)| ["I-8.7", "I-8.8"].contains(name))
         .filter_map(|(_, found)| match found {
             InvariantVerdict::Violated(detail) => Some(detail.as_str()),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect();
     TransactionBoundaryJudgement {
@@ -2459,7 +2461,9 @@ fn publish_ordinal_judgement(
         .find(|(name, _)| *name == "I-8.9")
         .and_then(|(_, found)| match found {
             InvariantVerdict::Violated(detail) => Some(detail.as_str()),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .unwrap_or("");
     PublishOrdinalJudgement {
@@ -2800,7 +2804,7 @@ fn a_system_configuration_slot_whose_encryption_type_is_on_reddens_only_its_own_
                         )),
                         "违例要点名被改的那一槽：{detail}"
                     ),
-                    other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+                    other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_)) => {
                         panic!("加密类型 1 的槽自证过：实现整池拒绝挂载，{invariant} 要判违例，得到 {other:?}")
                     }
                 }
@@ -2848,7 +2852,9 @@ fn each_system_configuration_geometry_field_the_reader_refuses_reddens_only_its_
                             "{refused_member}：违例说明里要是这个成员：{detail}"
                         );
                     }
-                    other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+                    other @ (InvariantVerdict::Holds
+                    | InvariantVerdict::NotApplicable(_)
+                    | InvariantVerdict::PartiallyJudged(_)) => {
                         panic!("{refused_member}：这一槽自证过而带这一版读者不收的值，I-7.13 要判违例，得到 {other:?}")
                     }
                 }
@@ -2882,7 +2888,9 @@ fn devices_whose_own_device_numbers_are_swapped_redden_only_the_own_device_numbe
         SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
     ) {
         InvariantVerdict::Violated(detail) => detail,
-        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("本盘设备号互换，I-7.14 要判违例，得到 {other:?}")
         }
     };
@@ -3166,7 +3174,8 @@ fn an_empty_leaf_hung_below_the_root_of_the_allocation_record_tree_reddens_only_
             InvariantVerdict::Violated(detail) if *invariant == "I-1.1" => Some(detail.clone()),
             InvariantVerdict::Violated(_)
             | InvariantVerdict::Holds
-            | InvariantVerdict::NotApplicable(_) => None,
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .expect("I-1.1 红");
     assert!(
@@ -6735,7 +6744,9 @@ fn assert_only_the_rotation_invariant_reddens_on_the_cell(
             detail.contains(cell_in_the_detail),
             "I-7.16 红在「{cell_in_the_detail}」那一格：{detail}"
         ),
-        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("I-7.16 该判违例，实际 {other:?}")
         }
     }
@@ -7276,7 +7287,9 @@ fn a_replay_stopped_by_a_named_unit_that_does_not_verify_leaves_the_replayed_ver
             reason.contains("介质损坏"),
             "报不适用的理由要说是起点含介质损坏：{reason}"
         ),
-        other @ (InvariantVerdict::Holds | InvariantVerdict::Violated(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("点名单元一份验不过时 I-4.1 该报不适用，实际 {other:?}")
         }
     }
@@ -7515,7 +7528,9 @@ fn a_record_of_another_instance_at_the_same_counter_does_not_hide_a_broken_overw
             detail.contains("（末条计数器 4）"),
             "checker 施加的是 B 那条（计数器 4）：{detail}"
         ),
-        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("checker 施加 B 之后那一版走不完，I-4.1 该判违例，实际 {other:?}")
         }
     }
@@ -7837,7 +7852,9 @@ fn a_wrong_release_generation_is_still_reddened_when_only_a_root_not_later_than_
             detail.contains("释放代 3"),
             "红在释放代 3 那几条上：{detail}"
         ),
-        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("A 走不完而它不比 L 晚：I-3.9 照判、该红，实际 {other:?}")
         }
     }
@@ -7875,7 +7892,9 @@ fn release_generations_hidden_behind_a_later_incomplete_root_are_skipped_and_the
             reason.contains("走不完"),
             "报不适用的理由要说比 L 晚的候选根走不完：{reason}"
         ),
-        other @ (InvariantVerdict::Holds | InvariantVerdict::Violated(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("落点藏在比 L 晚、走不完的根后面：I-3.9 该报不适用，实际 {other:?}")
         }
     }
diff --git a/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs b/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
index bbfea2d..1ca3784 100644
--- a/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
+++ b/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
@@ -165,7 +165,9 @@ fn violations(image: &dyn ImageReader) -> Vec<(&'static str, String)> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
@@ -185,7 +187,9 @@ fn verdict_of(image: &dyn ImageReader, invariant: &str) -> InvariantVerdict {
 fn detail_of_the_violation(image: &dyn ImageReader, invariant: &str) -> String {
     match verdict_of(image, invariant) {
         InvariantVerdict::Violated(detail) => detail,
-        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("{invariant} 该判违例，实际 {other:?}")
         }
     }
@@ -617,7 +621,9 @@ fn an_unreadable_root_ring_slot_leaves_the_abandoned_half_of_the_reuse_invariant
             reason.contains("读不出"),
             "理由要说是根环有读不出的槽：{reason}"
         ),
-        other @ (InvariantVerdict::Holds | InvariantVerdict::Violated(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("被抛弃根那一半判不了，I-7.4 不许报成立：{other:?}")
         }
     }
diff --git a/crates/singlefs-harness/tests/common_admission_pool/mod.rs b/crates/singlefs-harness/tests/common_admission_pool/mod.rs
index 123acfc..a2fcb3e 100644
--- a/crates/singlefs-harness/tests/common_admission_pool/mod.rs
+++ b/crates/singlefs-harness/tests/common_admission_pool/mod.rs
@@ -375,7 +375,9 @@ pub fn checker_violations_on(image: &MemoryPool) -> Vec<String> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
index 37e635f..c0949ad 100644
--- a/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
+++ b/crates/singlefs-harness/tests/container_trees_grow_a_level_when_the_root_is_full.rs
@@ -132,7 +132,9 @@ fn violations(pool: &BuiltPool) -> Vec<String> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/fault_injection_fast_tier.rs b/crates/singlefs-harness/tests/fault_injection_fast_tier.rs
index 61025f4..969481e 100644
--- a/crates/singlefs-harness/tests/fault_injection_fast_tier.rs
+++ b/crates/singlefs-harness/tests/fault_injection_fast_tier.rs
@@ -1047,7 +1047,9 @@ fn publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_pub
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(_) => Some(invariant),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect();
     assert!(
diff --git a/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs b/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs
index 19daba1..f63ef0d 100644
--- a/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs
+++ b/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs
@@ -501,7 +501,9 @@ fn describe(
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(_) => Some(invariant),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect();
     let _ = writeln!(text, "    池级 checker 判红：{violated:?}");
@@ -1168,7 +1170,9 @@ fn mount_and_write_on(
                     .into_iter()
                     .filter_map(|(invariant, verdict)| match verdict {
                         InvariantVerdict::Violated(_) => Some(invariant),
-                        InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+                        InvariantVerdict::Holds
+                        | InvariantVerdict::NotApplicable(_)
+                        | InvariantVerdict::PartiallyJudged(_) => None,
                     })
                     .collect();
                 (
diff --git a/crates/singlefs-harness/tests/instance_table_chain_read.rs b/crates/singlefs-harness/tests/instance_table_chain_read.rs
index d25d2ac..54f7080 100644
--- a/crates/singlefs-harness/tests/instance_table_chain_read.rs
+++ b/crates/singlefs-harness/tests/instance_table_chain_read.rs
@@ -583,7 +583,9 @@ fn chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant()
             detail.contains("实例表第 1 片"),
             "I-1.2 红在链上第 1 片的出生身份：{detail}"
         ),
-        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("链指针记的出生身份与 mkfs 那一片头里的不等，I-1.2 该红：{other:?}")
         }
     }
@@ -592,7 +594,9 @@ fn chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant()
             detail.contains(&format!("槽 {}", INSTANCE_TABLE_SLOT.0)),
             "I-3.9 红在 mkfs 那一片（已释放、又被链指针引用）：{detail}"
         ),
-        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("被换下的 mkfs 那一片又被每一条候选根引用，I-3.9 该红：{other:?}")
         }
     }
diff --git a/crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs b/crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs
index 90406a8..4f87495 100644
--- a/crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs
+++ b/crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs
@@ -70,7 +70,9 @@ fn violations_of(image: &MemoryPool) -> Vec<String> {
         .filter(|(invariant, _)| INVARIANTS_THE_SWEEP_KEEPS_GREEN.contains(invariant))
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/multi_unit_file_write_and_read.rs b/crates/singlefs-harness/tests/multi_unit_file_write_and_read.rs
index cdc0976..922671b 100644
--- a/crates/singlefs-harness/tests/multi_unit_file_write_and_read.rs
+++ b/crates/singlefs-harness/tests/multi_unit_file_write_and_read.rs
@@ -115,7 +115,9 @@ fn assert_the_pool_checker_finds_no_violation(image: &MemoryPool, what: &str) {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect();
     assert!(
diff --git a/crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs b/crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs
index 4db7248..0f41d7d 100644
--- a/crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs
+++ b/crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs
@@ -134,7 +134,9 @@ fn reclaiming_after_a_hole_in_the_root_ring_follows_the_one_revolution_window_an
             .filter(|(invariant, _)| INVARIANTS_THE_WINDOW_KEEPS_GREEN.contains(invariant))
             .filter_map(|(invariant, verdict)| match verdict {
                 InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
-                InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+                InvariantVerdict::Holds
+                | InvariantVerdict::NotApplicable(_)
+                | InvariantVerdict::PartiallyJudged(_) => None,
             })
             .collect();
         assert_eq!(
diff --git a/crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs b/crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs
index 5bbe534..30c8074 100644
--- a/crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs
+++ b/crates/singlefs-harness/tests/pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window.rs
@@ -1,4 +1,4 @@
-//! 池级 checker 的 I-7.9（回退下界 F 不高于抬 F 的上限） 不带卸载记号那一支：「之前的根」只取一圈窗口里的根
+//! 池级 checker 的 I-7.9（回退下界 F 不高于抬 F 的上限） 两支（不带卸载记号的准入抬 F、带卸载记号的正常卸载那一串）：「之前的根」只取一圈窗口里的根
 //! （`.claude/kb/invariants.md` I-7.9 那一行 2026-10-04 写回，照 D16（发布语义） 已定项 1「抬 F 的上限」只在一圈窗口里的根上取）。
 //! 窗口取两个、一起用：锚在被判那条抬 F 的根上的（条款字面），与锚在这张镜像根环里最新有效根上的（候选过滤那一个）；只取前一个时下面三形
 //! 合法镜像照样红（`crates/singlefs-checker/src/walk.rs` 的 `judge_rollback_floor_raises_against_their_ceilings` 写了理由）。
@@ -10,6 +10,11 @@
 //! - 同一段历史里实例 6 写行那次发布 txg 26 的根槽写被扣下（K1 类洞：那一版没被施加、取号越过它），再可写挂载加一次发布。
 //!
 //! 另一份坏镜像：洞上那张镜像里把那条抬 F 的根带的 F 改得高于窗口里的根算出的上限，I-7.9 照旧红。
+//!
+//! 带卸载记号那一支取同一个交（用户 2026-10-04 定「乙，两支都取两个窗口」）：正常卸载之后再挂、覆盖写到一圈之后，
+//! 一次覆盖写切在「记录已落、根没落」，再可写挂载——洞那一槽留着卸载前两版的那条根，卸载前最后一版的根被盖掉；
+//! 只取锚在被判那条根上的窗口时「txg 比它小的最新那条根」成了那条旧根、F 高于它而误红，取两个窗口的交判绿。
+//! 坏镜像：同一段历史挂载之前那张镜像上把卸载那一串第一条根带的 F 改高一，照旧红在带卸载记号那一支。
 //! 这几份都不经 checker 档：崩溃状态只取一个（扣下的那一个根槽写之前全落），不枚举。
 
 mod common_admission;
@@ -21,7 +26,7 @@ use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::checksum::wide_checksum_with_field_zeroed;
-use singlefs_core::mount::mount_writable_with_space_admission;
+use singlefs_core::mount::{mount_writable_with_space_admission, unmount, ShadowLedger, Unmounted};
 use singlefs_core::recovery::{
     choose_system_configuration, every_root_ring_slot, read_root_ring_slot, RootRingSlotReading,
 };
@@ -59,6 +64,8 @@ const INJECT_SIX_RAISING_ROOT_TXG: u64 = 9;
 /// 扣下的根槽写：实例 5 txg 25（由记录施加出来那一类洞，占洞的是 txg 1）、实例 6 txg 26（写行那次，K1 类洞，占洞的是 txg 2）。
 const INJECT_SIX_WITHHELD_ROOT_APPLIED_BY_ITS_RECORD: (u32, u64) = (5, 25);
 const INJECT_SIX_WITHHELD_ROOT_OF_THE_ROW_PUBLISH: (u32, u64) = (6, 26);
+/// 带卸载记号那一形：卸载之前覆盖写几次。
+const OVERWRITES_BEFORE_THE_UNMOUNT: usize = 3;
 /// 挂载之后那一次发布的内容长度：一个数据单元装得下。
 const CONTENT_BYTES_OF_THE_PUBLISH_AFTER_THE_MOUNT: usize = 3000;
 /// 挂载之后那一次发布的写入时间。
@@ -410,8 +417,198 @@ fn a_floor_raised_above_the_ceiling_of_the_roots_inside_the_window_still_reddens
             )),
             "红在 txg 9 那条根从 0 抬到 7 上：{detail}"
         ),
-        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("F 抬到 7 高于窗口里的上限，I-7.9 要红，得到 {other:?}")
         }
     }
 }
+
+/// 正常卸载、再挂、洞上再挂那段历史的两张镜像与卸载那一串的几个数。
+struct UnmountThenAHoleOneRevolutionLater {
+    /// 切在洞那次覆盖写的根槽写之前（之前的写全落）：卸载前最后一版的根还在环里。
+    crashed: MemoryPool,
+    /// `crashed` 上再可写挂载一次之后。
+    after_the_remount: MemoryPool,
+    /// 卸载抬到的 F（卸载开始时现行那一版的 txg，记作 v）。
+    raised_floor: u64,
+    /// 卸载那一串带记号的空发布的 txg（第一条是 v + 1）。
+    marked_root_txgs: Vec<u64>,
+    /// 根槽一直没写的那次覆盖写的 txg：v − 1 + 3S，它那一槽留着 v − 1 的根。
+    hole_txg: u64,
+}
+
+/// 两块 4 GiB 盘：第一个文件之后覆盖写三次（现行那一版 txg 记作 v），正常卸载（F 抬到 v，推带卸载记号的空发布 v + 1 起），
+/// 再可写挂载、覆盖写到 v − 2 + 3S，下一次覆盖写（txg v − 1 + 3S）切在「journal 记录已落、根槽没落」：那一槽留着 v − 1 的根。
+/// 再可写挂载一次：由记录施加出那一版，写行与暖机从 v + 3S 写起，盖掉 v 那一槽（与之后的一两槽）。
+fn unmount_then_a_hole_one_revolution_later() -> UnmountThenAHoleOneRevolutionLater {
+    let stream = SharedStream::retaining_contents();
+    let wrap = stream.clone();
+    let mut pool: PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>> =
+        PoolUnderTest::start_after_the_first_file(
+            HistoryDeviceWidth::FourGibibytes,
+            move |identity, device| {
+                RecordingBlockDevice::with_shared_stream(identity, device, wrap.clone())
+            },
+        );
+    for _ in 0..OVERWRITES_BEFORE_THE_UNMOUNT {
+        pool.overwrite(OVERWRITE_BYTES).expect("卸载之前覆盖写");
+    }
+    let mut session = pool.session.take().expect("这一刻有可写会话");
+    let Unmounted::FloorRaisedToTheCurrentVersion(unmounted) = unmount(
+        &pool.parameters,
+        &mut pool.devices,
+        &mut session.allocator,
+        &mut session.current,
+        ShadowLedger::On,
+    )
+    .expect("正常卸载") else {
+        panic!("现行那一版带文件：卸载抬 F");
+    };
+    let raised_floor = unmounted.rollback_floor.0;
+    let marked_root_txgs: Vec<u64> = unmounted
+        .publishes
+        .iter()
+        .map(|publish| publish.root.checkpoint_txg.0)
+        .collect();
+    assert_eq!(
+        marked_root_txgs.first().copied(),
+        Some(raised_floor + 1),
+        "卸载那一串从 v + 1 起"
+    );
+    pool.crash_and_mount_writable().expect("卸载之后再挂");
+    let root_ring_slot_count = u64::try_from(pool.parameters.region_devices.len())
+        .expect("区域数装得进 u64")
+        * pool.parameters.geometry.root_ring_slots_per_region.count();
+    let hole_txg = raised_floor - 1 + root_ring_slot_count;
+    while pool.current_file_version().root.checkpoint_txg.0 + 1 < hole_txg {
+        pool.overwrite(OVERWRITE_BYTES).expect("再挂之后覆盖写");
+    }
+    assert_eq!(
+        pool.current_file_version().root.checkpoint_txg.0 + 1,
+        hole_txg,
+        "下一次覆盖写正好是洞那一次（中间没有推抬 F）"
+    );
+    let before = pool.image();
+    let first_operation = stream.retained_operations().len();
+    pool.overwrite(OVERWRITE_BYTES).expect("洞那一次覆盖写");
+    let operations = stream.retained_operations();
+    let this_publish = &operations[first_operation..];
+    let first_root = this_publish
+        .iter()
+        .position(|operation| {
+            operation.operation.kind == RecordedOperationKind::WriteForceUnitAccess
+                && operation
+                    .contents
+                    .as_ref()
+                    .is_some_and(|bytes| bytes.len() == ROOT_SLOT_BYTES)
+        })
+        .expect("这次发布写根");
+    let (_, txg_of_the_missing_root) = root_identity_written_by(
+        this_publish[first_root]
+            .contents
+            .as_ref()
+            .expect("录制流留着写的内容"),
+    );
+    assert_eq!(txg_of_the_missing_root, CheckpointTxg(hole_txg));
+    let mut crashed = before;
+    crashed.apply(&this_publish[..first_root]);
+    let mut devices = plain_devices_on(&crashed);
+    let mounted = mount_writable_with_space_admission(
+        &pool.parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+    )
+    .expect("洞上再挂做得成");
+    assert_eq!(
+        mounted.output.effective_root.checkpoint_txg,
+        CheckpointTxg(hole_txg),
+        "生效版是 journal 施加出来的洞那一版"
+    );
+    drop(mounted);
+    let after_the_remount = MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.image.clone()))
+            .collect(),
+        device_size_in_bytes: crashed.device_size_in_bytes,
+    };
+    UnmountThenAHoleOneRevolutionLater {
+        crashed,
+        after_the_remount,
+        raised_floor,
+        marked_root_txgs,
+        hole_txg,
+    }
+}
+
+/// 带卸载记号那一支的误红形（随机历史种子 31045、1037 那一类）：洞上再挂之后，根环里卸载前最后一版 v 的根被盖掉，
+/// 洞那一槽留着 v − 1 的根；卸载那一串里留在环里的带记号的根，同一实例前一条成了 v − 1、算抬 F。只取锚在它上面的窗口时
+/// 「txg 比它小的最新那条根」是 v − 1、F = v 高于它而红；两个窗口的交里没有它之前的根（镜像最新根离它不到一圈，
+/// 比它小的根都 ≤ N − 3S），这条根不判。挂载之前那张镜像（v 还在环里）I-7.9 按带记号那一支判 v ≤ v 成立。
+#[test]
+fn an_unmount_sequence_root_after_a_hole_one_revolution_later_is_not_judged_against_the_old_root_left_in_the_hole(
+) {
+    let history = unmount_then_a_hole_one_revolution_later();
+    let v = history.raised_floor;
+    let ring_before = root_txgs_in_the_ring(&history.crashed);
+    assert!(
+        ring_before.contains(&v) && ring_before.contains(&(v - 1)),
+        "挂载之前 v 与 v − 1 的根都在环里：{ring_before:?}"
+    );
+    assert_eq!(
+        rollback_floor_verdict(&history.crashed),
+        InvariantVerdict::Holds,
+        "挂载之前带记号的根按前一条根 v 判，成立"
+    );
+    let ring_after = root_txgs_in_the_ring(&history.after_the_remount);
+    assert!(
+        !ring_after.contains(&history.hole_txg)
+            && ring_after.contains(&(v - 1))
+            && !ring_after.contains(&v),
+        "洞那一槽留着 v − 1 的根，v 那一槽被盖掉：{ring_after:?}"
+    );
+    assert!(
+        history
+            .marked_root_txgs
+            .iter()
+            .any(|txg| ring_after.contains(txg)),
+        "卸载那一串还有带记号的根留在环里：{:?} / {ring_after:?}",
+        history.marked_root_txgs
+    );
+    assert!(
+        !matches!(
+            rollback_floor_verdict(&history.after_the_remount),
+            InvariantVerdict::Violated(_)
+        ),
+        "洞里的 v − 1 顶上来，带记号那一支不许红：{:?}",
+        rollback_floor_verdict(&history.after_the_remount)
+    );
+}
+
+/// 坏镜像：同一段历史挂载之前那张镜像上，把卸载那一串第一条根（txg v + 1）带的 F 从 v 改成 v + 1（重封根槽）：
+/// 两个窗口的交里它之前最新的根是 v，F 高于它，I-7.9 照旧红在带卸载记号那一支。
+#[test]
+fn an_unmount_sequence_root_raising_the_floor_above_the_previous_root_inside_both_windows_still_reddens_the_rollback_floor_invariant(
+) {
+    let history = unmount_then_a_hole_one_revolution_later();
+    let v = history.raised_floor;
+    let mut image = history.crashed;
+    rewrite_the_floor_of_the_root_at(&mut image, v + 1, v + 1);
+    match rollback_floor_verdict(&image) {
+        InvariantVerdict::Violated(detail) => assert!(
+            detail.contains(&format!(
+                "txg {} 那条带卸载记号的根把回退下界 F 从 0 抬到 {}，高于根环里 txg 比它小的最新那条根的 txg {v}",
+                v + 1,
+                v + 1
+            )),
+            "红在带卸载记号那一支、前一条根 v：{detail}"
+        ),
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
+            panic!("带记号的根把 F 抬到 v + 1、高于前一条根 v，I-7.9 要红，得到 {other:?}")
+        }
+    }
+}
diff --git a/crates/singlefs-harness/tests/pool_checker_gap_row_conditions_and_incomplete_reference_pairs.rs b/crates/singlefs-harness/tests/pool_checker_gap_row_conditions_and_incomplete_reference_pairs.rs
index e125b3b..a72f197 100644
--- a/crates/singlefs-harness/tests/pool_checker_gap_row_conditions_and_incomplete_reference_pairs.rs
+++ b/crates/singlefs-harness/tests/pool_checker_gap_row_conditions_and_incomplete_reference_pairs.rs
@@ -90,7 +90,9 @@ fn assert_the_reference_interval_invariant_is_violated_mentioning(
             detail.contains(expected_in_the_detail),
             "I-3.5 判违例，说明里该有「{expected_in_the_detail}」：{detail}"
         ),
-        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("I-3.5 应当判违例（说明里有「{expected_in_the_detail}」），实际 {other:?}")
         }
     }
diff --git a/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs b/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
index 8480788..f2a1d3e 100644
--- a/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
+++ b/crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs
@@ -92,7 +92,9 @@ fn violations_of(image: &MemoryPool) -> Vec<(&'static str, String)> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/pool_checker_scan_of_tree_identifiers_claims_whole_units.rs b/crates/singlefs-harness/tests/pool_checker_scan_of_tree_identifiers_claims_whole_units.rs
index 2dea657..86da077 100644
--- a/crates/singlefs-harness/tests/pool_checker_scan_of_tree_identifiers_claims_whole_units.rs
+++ b/crates/singlefs-harness/tests/pool_checker_scan_of_tree_identifiers_claims_whole_units.rs
@@ -232,7 +232,9 @@ fn violated_invariants(image: &MemoryPool) -> BTreeMap<&'static str, String> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs b/crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs
index 15089bd..8312260 100644
--- a/crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs
+++ b/crates/singlefs-harness/tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs
@@ -2,6 +2,10 @@
 //! 退回去重发） 那一形上红。历史照攻方原型 `research/prompts/m3-sweep-and-tree-id-r1-opus-model/sweep-proto/src/bin/tree_id_histories.rs`
 //! 的 `orphan_ff`、`applied_ff`、`rollback`、`c342`（`C342_ROLL_BACK_SYSCONF=1`）四臂另写一份（D13（验证路线） 已定项 5：用例不与原型共用代码），
 //! 产物是同目录 `results/out/th-*.out`（判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第二节 S2）。
+//!
+//! 另有三形钉第四种判定（用户 2026-10-04 定「树表读不出算部分没判」、加 `InvariantVerdict::PartiallyJudged`；细则主 agent 定）：
+//! 一条根的树表读不出、其余对得上 ⇒ 部分没判且列出那条根；没有一棵树出现在两个不同的树表单元里 ⇒ 不适用；
+//! 有一处对不上、另有一条根的树表读不出 ⇒ 照报违例。
 
 use singlefs_checker::image::{
     InvariantVerdict, TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
@@ -17,7 +21,7 @@ use singlefs_core::system_configuration::SystemImmutableSizes;
 use singlefs_core::transaction::{
     publish_first_file, publish_overwrite, FirstFile, PoolWriter, TransactionOutput,
 };
-use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
+use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES};
 use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
 use singlefs_harness::segments::{FixedGeometry, StepKind};
 use singlefs_harness::{RecordingBlockDevice, SharedStream};
@@ -34,6 +38,8 @@ const WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS: u64 = 1_788_000_000;
 const FILE_BYTES: usize = 3000;
 /// extent 树的树 ID：从 mkfs 水位 65536 发起的第一个号（D8（核心索引结构） 已定项 8）。
 const TREE_IDENTIFIER_OF_THE_FIRST_EXTENT_TREE: u64 = 65536;
+/// 改坏一片树表时翻它载荷里的哪一个字节（头之后、条目区里：两份位置条目读出来整单元校验和都对不上）。
+const BYTE_INSIDE_THE_PAYLOAD_OF_A_TREE_TABLE: u64 = 4000;
 /// 第一个文件那一版的树 ID 水位：mkfs 水位 65536 之后发 12 个号（`th-c342-sysconf.out` 里实例 1 那一版与实例 2 重发那一版都是 65548）。
 const TREE_IDENTIFIER_WATERMARK_AFTER_THE_FIRST_FILE: u64 = 65548;
 
@@ -160,6 +166,47 @@ fn birth_txg_verdict(image: &MemoryPool) -> InvariantVerdict {
         .1
 }
 
+/// 把 `version` 那条根指着的树表两份位置条目各翻一个字节（不重封、不往上补）：那条根照样自证合法，它的树表读不出。
+fn make_the_tree_table_of_the_version_unreadable(
+    image: &mut MemoryPool,
+    version: &TransactionOutput,
+) {
+    for location in &version.root.tree_table.locations {
+        image.flip_byte(
+            location.device,
+            DeviceOffsetInBytes(location.slot.0 * SLOT_BYTES),
+            BYTE_INSIDE_THE_PAYLOAD_OF_A_TREE_TABLE,
+        );
+    }
+}
+
+/// mkfs → 可写挂载实例 1 → 第一个文件 A → 覆盖写两次（B、C）：三片带条目的树表，同一批树 ID、诞生 txg 都相同。交回镜像与 B 那一版。
+fn first_file_and_two_overwrites() -> (MemoryPool, TransactionOutput) {
+    let parameters = parameters_of_the_birth_pools();
+    let mut devices = fresh_devices();
+    make_filesystem(&parameters, &mut devices).expect("mkfs");
+    let mut mounted = mount_writable(&parameters, &mut devices).expect("挂载实例 1");
+    let instance = mounted.output.instance;
+    let first = {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut mounted.allocator,
+            mounted.current.root(),
+            FirstFile {
+                content: &content_of_version(0),
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS,
+            },
+            instance,
+            mounted.current.record_bytes(),
+        )
+        .expect("第一个文件 A")
+    };
+    let second = overwrite(&mut devices, &mut mounted.allocator, &first, 1, instance);
+    overwrite(&mut devices, &mut mounted.allocator, &second, 2, instance);
+    (memory_pool_of(&devices), second)
+}
+
 /// mkfs → 可写挂载实例 1 → 第一个文件那次发布录下来，切在第一次 `cut_before` 那一类写之前：交回崩溃镜像。
 fn first_file_crashed_before_the_first(cut_before: StepKind) -> MemoryPool {
     let parameters = parameters_of_the_birth_pools();
@@ -331,11 +378,10 @@ fn a_rollback_by_a_forward_publish_and_an_overwrite_after_it_hold_the_every_read
 /// 攻方 `c342` 臂带 `C342_ROLL_BACK_SYSCONF=1`（`th-c342-sysconf.out`）：实例 1 第一个文件（extent 树 65536 生在 txg 3）、覆盖写 7 次到 txg 10；
 /// 这 8 次发布的根槽写与记录写全抹成 0（读不出），系统配置也退回实例 1 发第一个文件之前那一份（整卷回滚，D18（块里携带什么信息） 已定项 11
 /// 宣布不防；只抹根与记录时可写挂载在取号之前被拒）。实例 2 落回暖机那一版、写行、暖机、重发第一个文件（从根环水位 65536 再发一次号，
-/// extent 树 65536 生在 txg 5）；之后把没被实例 2 盖掉的根槽与记录放回去（故障消失）。根环里实例 1 那几条老根记 65536 生在 3、实例 2 的记 5：
-/// I-9.28 红，红在树 65536 上。
-#[test]
-fn tree_identifiers_reissued_after_a_whole_volume_rollback_redden_the_every_readable_root_birth_invariant_once_the_hidden_roots_come_back(
-) {
+/// extent 树 65536 生在 txg 5）；之后把没被实例 2 盖掉的根槽与记录放回去（故障消失）。根环里实例 1 那几条老根记 65536 生在 3、实例 2 的记 5。
+/// 交回放回之后的镜像，与实例 1 最后那一版（txg 10：它的根槽实例 2 没盖到，放回来了）。
+fn image_after_a_whole_volume_rollback_with_the_hidden_roots_put_back(
+) -> (MemoryPool, TransactionOutput) {
     let parameters = parameters_of_the_birth_pools();
     let geometry = fixed_geometry_of_the_birth_pools();
     let mut devices = fresh_devices();
@@ -345,7 +391,7 @@ fn tree_identifiers_reissued_after_a_whole_volume_rollback_redden_the_every_read
     let before = memory_pool_of(&devices);
     let stream = SharedStream::retaining_contents();
     let mut recorded = recorded_devices_of(&before, &stream);
-    {
+    let last_version_of_the_first_instance = {
         let mut current = {
             let mut writer = PoolWriter::new(&parameters, recorded.as_mut_slice());
             publish_first_file(
@@ -375,7 +421,8 @@ fn tree_identifiers_reissued_after_a_whole_volume_rollback_redden_the_every_read
             )
             .expect("实例 1 覆盖写");
         }
-    }
+        current
+    };
     let operations = stream.retained_operations();
     let mut image = before.clone();
     image.apply(&operations);
@@ -461,13 +508,113 @@ fn tree_identifiers_reissued_after_a_whole_volume_rollback_redden_the_every_read
                 .write(DeviceOffsetInBytes(*offset), contents);
         }
     }
+    (
+        image_with_the_hidden_writes_put_back,
+        last_version_of_the_first_instance,
+    )
+}
+
+/// 放回之后根环里实例 1 那几条老根记 65536 生在 3、实例 2 的记 5：I-9.28 红，红在树 65536 上。
+#[test]
+fn tree_identifiers_reissued_after_a_whole_volume_rollback_redden_the_every_readable_root_birth_invariant_once_the_hidden_roots_come_back(
+) {
+    let (image_with_the_hidden_writes_put_back, _) =
+        image_after_a_whole_volume_rollback_with_the_hidden_roots_put_back();
     match birth_txg_verdict(&image_with_the_hidden_writes_put_back) {
         InvariantVerdict::Violated(detail) => assert!(
             detail.contains(&format!("树 {TREE_IDENTIFIER_OF_THE_FIRST_EXTENT_TREE} ")),
             "红在 extent 树 65536 上：{detail}"
         ),
-        other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("老线与新线上同一树 ID 65536 诞生 txg 不同，I-9.28 要红，得到 {other:?}")
         }
     }
 }
+
+/// 同一张镜像上再把实例 1 最后那一版（txg 10）的树表两份都翻坏：那条根列进「没判到」，而老线与新线上别的根照样对不上——
+/// 判出了违例就报违例，不报部分没判。
+#[test]
+fn a_birth_txg_disagreement_is_reported_as_violated_even_when_another_roots_tree_table_is_unreadable(
+) {
+    let (mut image, last_version_of_the_first_instance) =
+        image_after_a_whole_volume_rollback_with_the_hidden_roots_put_back();
+    make_the_tree_table_of_the_version_unreadable(&mut image, &last_version_of_the_first_instance);
+    match birth_txg_verdict(&image) {
+        InvariantVerdict::Violated(detail) => assert!(
+            detail.contains(&format!("树 {TREE_IDENTIFIER_OF_THE_FIRST_EXTENT_TREE} ")),
+            "红在 extent 树 65536 上：{detail}"
+        ),
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => panic!(
+            "老线与新线上同一树 ID 65536 诞生 txg 不同、另有一条根的树表读不出：照报违例，得到 {other:?}"
+        ),
+    }
+}
+
+/// 第一个文件 A、覆盖写 B、C，再把 B 那条根指着的树表两份都翻坏（B 的根照样自证合法）：A 与 C 两片树表比得上、诞生 txg 相同，
+/// B 那一片没进比较——报部分没判，说明里列出 B 那条根（实例 1、它的 txg）与「树表读不出」。
+#[test]
+fn a_readable_root_whose_tree_table_is_unreadable_while_the_others_agree_is_reported_as_partially_judged_naming_that_root(
+) {
+    let (mut image, second) = first_file_and_two_overwrites();
+    assert_eq!(
+        birth_txg_verdict(&image),
+        InvariantVerdict::Holds,
+        "翻坏之前：三片树表比得上、诞生 txg 相同"
+    );
+    make_the_tree_table_of_the_version_unreadable(&mut image, &second);
+    match birth_txg_verdict(&image) {
+        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
+            unjudged.contains(&format!(
+                "（实例 {}、txg {}）：树表读不出",
+                second.root.instance.0, second.root.checkpoint_txg.0
+            )),
+            "列出 B 那条根与为什么没判：{unjudged}"
+        ),
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::Violated(_)) => {
+            panic!("B 的树表读不出、A 与 C 对得上：I-9.28 报部分没判，得到 {other:?}")
+        }
+    }
+}
+
+/// mkfs → 可写挂载实例 1 → 第一个文件：带条目的树表只有第一个文件那一片（mkfs 与暖机那几条根指着的树表 0 条），
+/// 没有一棵树的条目出现在两个不同的树表单元里——报不适用，不报成立。
+#[test]
+fn no_tree_seen_in_two_different_tree_table_units_is_reported_as_not_applicable() {
+    let parameters = parameters_of_the_birth_pools();
+    let mut devices = fresh_devices();
+    make_filesystem(&parameters, &mut devices).expect("mkfs");
+    let mut mounted = mount_writable(&parameters, &mut devices).expect("挂载实例 1");
+    let instance = mounted.output.instance;
+    {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut mounted.allocator,
+            mounted.current.root(),
+            FirstFile {
+                content: &content_of_version(0),
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_BIRTH_POOLS,
+            },
+            instance,
+            mounted.current.record_bytes(),
+        )
+        .expect("第一个文件");
+    }
+    match birth_txg_verdict(&memory_pool_of(&devices)) {
+        InvariantVerdict::NotApplicable(reason) => assert!(
+            reason.contains("两个不同的树表单元"),
+            "理由要说没有一棵树出现在两个不同的树表单元里：{reason}"
+        ),
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
+            panic!("只有一片带条目的树表：I-9.28 报不适用，得到 {other:?}")
+        }
+    }
+}
diff --git a/crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs b/crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs
index 4d263a4..c9f6489 100644
--- a/crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs
+++ b/crates/singlefs-harness/tests/raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs
@@ -76,7 +76,9 @@ fn violations_of(image: &MemoryPool, invariants_judged: &[&str]) -> Vec<String>
         .filter(|(invariant, _)| invariants_judged.contains(invariant))
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs b/crates/singlefs-harness/tests/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs
index 9fe0773..249f28e 100644
--- a/crates/singlefs-harness/tests/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs
+++ b/crates/singlefs-harness/tests/raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs
@@ -49,7 +49,9 @@ fn violations_on(image: &MemoryPool) -> Vec<(&'static str, String)> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs b/crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs
index 79b76d1..37b2ebf 100644
--- a/crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs
+++ b/crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs
@@ -77,7 +77,9 @@ fn violations_of(image: &MemoryPool) -> Vec<String> {
         .filter(|(invariant, _)| INVARIANTS_THE_RAISE_KEEPS_GREEN.contains(invariant))
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/random_histories.rs b/crates/singlefs-harness/tests/random_histories.rs
index a464475..0b54731 100644
--- a/crates/singlefs-harness/tests/random_histories.rs
+++ b/crates/singlefs-harness/tests/random_histories.rs
@@ -1241,7 +1241,9 @@ fn violations_on_the_image(image: &MemoryPool) -> Vec<(&'static str, String)> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs b/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
index 38b1631..248341e 100644
--- a/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
+++ b/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
@@ -1059,7 +1059,9 @@ fn four_overwrites_after_crash_recovery_abandoned_the_third_version_are_red_only
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect();
     assert_eq!(
diff --git a/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs b/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
index bd132e5..29033e6 100644
--- a/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
+++ b/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
@@ -1941,7 +1941,9 @@ fn plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_
         .iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((*invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect();
     assert_eq!(
diff --git a/crates/singlefs-harness/tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs b/crates/singlefs-harness/tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs
index cbd0db5..2b13f68 100644
--- a/crates/singlefs-harness/tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs
+++ b/crates/singlefs-harness/tests/rollback_lineage_record_and_its_checkpoint_cost_term.rs
@@ -71,7 +71,9 @@ fn violations(pool: &BuiltPool) -> Vec<String> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs b/crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs
new file mode 100644
index 0000000..13d43d0
--- /dev/null
+++ b/crates/singlefs-harness/tests/root_ring_slot_reread_outcomes_the_instance_table_judging_them_and_the_publishes_that_reread.rs
@@ -0,0 +1,868 @@
+//! 进程内记根时重读挂载那一刻读不出的根环槽（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」那一行的 E2 句与「根槽这一次读坏」那一行）：
+//! 重读那一次的几种结局各记成什么、判「按实例表有效」用哪张表、产品路径上记根的几处各自走到重读。
+//!
+//! - 读得出而自证不过，与仍读不出同一处置，按根环表记的最大 txg 算（用户 2026-10-04 定改法 A，代码轮第二轮判决
+//!   `research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1；攻方 `research/prompts/m3-root-ring-code-r2-opus-model/proto/src/bin/e2_reread_flip.rs`
+//!   那一形：一条有效根所在的槽挂载那一刻读错、它成了窗口里最小的那一刻那一次重读读坏，改之前回收门槛越过它、I-3.1 记账少）；
+//!   代价那一形（同一目录 `front_hole_eio.rs`：从没写过的槽加挂载那一刻读错）判成什么就钉什么。
+//! - 读得出、自证过、按实例表有效的照读出来的 txg 算，那一槽从此算「这个进程知道住着根的槽」（用户 2026-10-04 定）；
+//!   读得出、自证过而按实例表被抛弃的当没有根；判有效用的表写行之后是写行那一张（同一份判决第二节 RS2，正推腿探针 p1、p2）。
+//! - 记根的产品路径：带单元的发布（`transaction::publish_admitted`，已有变异行管它）、树表 0 条那一版上的暖机零单元发布
+//!   （`mount::publish_empty_after`）走到重读（探针 q1）。另两处没有会红的用例：第一次可写挂载的零单元写行发布
+//!   （`mount::establish_instance` 里 `rows_written` 为空那一支）走不到重读——那一刻根环里只有 mkfs 写的 txg 0，挂载时读不出的槽只可能是它，
+//!   而它读不出时择不到根、挂载不成；树表 0 条那一版上要写行的写行发布（`transaction::prepare_the_row_publish_on_a_version_without_file`，探针 q3）
+//!   重读的那一槽恰是紧跟着的暖机要盖掉的那一槽（写行 txg + 1），重读改的根环表与回收在暖机那一次被盖成同一个样子，那一版又没有记账树，
+//!   挂载交回之后看不出它重读没重读（交回报告里列着）。
+mod common;
+mod common_root_ring_holes;
+
+use std::cell::RefCell;
+use std::collections::BTreeSet;
+use std::rc::Rc;
+
+use common::{
+    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
+    overwrite_in_process_as_instance_one_and_advance_the_pool,
+    BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER, FIXED_WRITE_TIME_SECONDS,
+};
+use common_root_ring_holes::{
+    content_of_the_version, devices_of, fixed_geometry_of_the_ring_hole_pools,
+    parameters_of_the_ring_hole_pools, pool_of, pool_overwritten_through, recording_of,
+    RingHoleDevices, DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
+    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
+};
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+};
+use singlefs_core::allocator::PoolAllocator;
+use singlefs_core::block_device::{
+    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
+};
+use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
+use singlefs_core::mount::{
+    mount_writable, raise_rollback_floor_to_the_admission_ceiling, MountError, ShadowLedger,
+};
+use singlefs_core::recovery::instance_table_chain_of_root;
+use singlefs_core::root_record::RootRecord;
+use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
+use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
+use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
+use singlefs_harness::segments::StepKind;
+use singlefs_harness::{RecordingBlockDevice, SharedStream};
+
+/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
+const INVARIANTS_THE_REREAD_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];
+/// 攻方那一形：实例 1 覆盖写到 txg 20，有效根 h = 18 所在的槽（区域 0、槽 2）挂载那一刻读错。
+const LAST_TXG_OF_INSTANCE_ONE: u64 = 20;
+const TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT: u64 = 18;
+/// h + 3S − 1：那一槽成了窗口里最小的那个、记这条根时重读它的那一版（S = 4）。
+const TXG_AT_WHICH_THE_SLOT_IS_REREAD: u64 = 29;
+/// 前洞那一形：实例 2 的 txg 9 那次覆盖写崩在根槽写之前（环还没转过来，那一槽从没写过）。
+const TXG_OF_THE_FRONT_HOLE: u64 = 9;
+/// 前洞那一形里那一槽成了窗口里最小的那个的那一版：9 + 3S − 1。
+const TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD: u64 = 20;
+
+/// 被抛弃根那一形判的几条：同上去掉 I-7.4。被抛弃的根所在的槽挂载那一刻读不出，影子账罩不到它引用的单元（挂载那一刻就读不到那条根，
+/// 与重读无关：攻方 `research/prompts/m3-root-ring-code-r2-opus-output.md` K1 第 3 小节「被抛弃」那一条），之后那几个单元被合法复用，
+/// I-7.4 在这一形上红是今天已知的那一格，不归这条用例判。
+const INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT: [&str; 4] = ["I-3.1", "I-3.11", "I-2.1", "I-4.8"];
+
+fn violations_of(image: &MemoryPool) -> Vec<String> {
+    violations_among(image, &INVARIANTS_THE_REREAD_KEEPS_GREEN)
+}
+
+fn violations_among(image: &MemoryPool, invariants_judged: &[&str]) -> Vec<String> {
+    check_pool_image(image)
+        .into_iter()
+        .filter(|(invariant, _)| invariants_judged.contains(invariant))
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
+        })
+        .collect()
+}
+
+/// 一个根环槽的读这会儿怎么坏。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum ReadsOfTheRootRingSlot {
+    PassedThrough,
+    DeviceErrorOnEveryRead,
+    /// 任何一块盘上发出第一个写（取号写系统配置）之前每次读都报设备错，之后照读：只在可写挂载读阶段读错。
+    DeviceErrorUntilTheFirstWriteOfAnyDevice,
+    /// 只有下一次读坏：读得出，那一槽里一个字节取反（自证不过，盘上字节不变）；之后照读。
+    OneByteFlippedOnTheNextReadOnly,
+}
+
+struct FaultOfOneRootRingSlotState {
+    device: DeviceIdentity,
+    offset_in_bytes: u64,
+    length_in_bytes: u64,
+    reads: ReadsOfTheRootRingSlot,
+    a_write_was_sent: bool,
+    failed_reads: u64,
+    flipped_reads: u64,
+}
+
+/// 几块盘共用的、一个根环槽的读故障。
+#[derive(Clone)]
+struct FaultOfOneRootRingSlot(Rc<RefCell<FaultOfOneRootRingSlotState>>);
+
+enum ReadOutcome {
+    Read,
+    DeviceError,
+    FlipTheByteAt(usize),
+}
+
+impl FaultOfOneRootRingSlot {
+    fn of_the_root_slot_of_txg(
+        parameters: &MakeFilesystemParameters,
+        checkpoint_txg: CheckpointTxg,
+        reads: ReadsOfTheRootRingSlot,
+    ) -> Self {
+        let ring_slot = target_for_publish(
+            checkpoint_txg,
+            parameters.geometry.root_ring_slots_per_region,
+        );
+        Self(Rc::new(RefCell::new(FaultOfOneRootRingSlotState {
+            device: parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")],
+            offset_in_bytes: slot_offset(
+                ring_slot,
+                parameters.geometry.fixed_structure_slot_spacing,
+            )
+            .0,
+            length_in_bytes: u64::from(parameters.geometry.physical_block_size),
+            reads,
+            a_write_was_sent: false,
+            failed_reads: 0,
+            flipped_reads: 0,
+        })))
+    }
+
+    fn set_reads(&self, reads: ReadsOfTheRootRingSlot) {
+        self.0.borrow_mut().reads = reads;
+    }
+
+    fn failed_reads(&self) -> u64 {
+        self.0.borrow().failed_reads
+    }
+
+    fn flipped_reads(&self) -> u64 {
+        self.0.borrow().flipped_reads
+    }
+
+    fn note_a_write(&self) {
+        self.0.borrow_mut().a_write_was_sent = true;
+    }
+
+    fn note_a_read(
+        &self,
+        device: DeviceIdentity,
+        offset_in_bytes: u64,
+        length: u64,
+    ) -> ReadOutcome {
+        let mut state = self.0.borrow_mut();
+        let overlaps = state.device == device
+            && offset_in_bytes < state.offset_in_bytes + state.length_in_bytes
+            && state.offset_in_bytes < offset_in_bytes + length;
+        if !overlaps {
+            return ReadOutcome::Read;
+        }
+        match state.reads {
+            ReadsOfTheRootRingSlot::PassedThrough => ReadOutcome::Read,
+            ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead => {
+                state.failed_reads += 1;
+                ReadOutcome::DeviceError
+            }
+            ReadsOfTheRootRingSlot::DeviceErrorUntilTheFirstWriteOfAnyDevice => {
+                if state.a_write_was_sent {
+                    ReadOutcome::Read
+                } else {
+                    state.failed_reads += 1;
+                    ReadOutcome::DeviceError
+                }
+            }
+            ReadsOfTheRootRingSlot::OneByteFlippedOnTheNextReadOnly => {
+                let byte_in_the_slot =
+                    state.offset_in_bytes + BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER;
+                if byte_in_the_slot < offset_in_bytes
+                    || byte_in_the_slot >= offset_in_bytes + length
+                {
+                    return ReadOutcome::Read;
+                }
+                state.flipped_reads += 1;
+                state.reads = ReadsOfTheRootRingSlot::PassedThrough;
+                ReadOutcome::FlipTheByteAt(
+                    usize::try_from(byte_in_the_slot - offset_in_bytes).expect("落在缓冲区里"),
+                )
+            }
+        }
+    }
+}
+
+/// 读的时候按 [`FaultOfOneRootRingSlot`] 读坏一个根环槽的盘；写、写零、屏障原样交给里面那块盘（写记一笔：取号之后不再报读错的那一档要它）。
+struct DeviceWithOneRootRingSlotFaulted<Inner: BlockDevice> {
+    identity: DeviceIdentity,
+    inner: Inner,
+    fault: FaultOfOneRootRingSlot,
+}
+
+impl<Inner: BlockDevice> BlockDevice for DeviceWithOneRootRingSlotFaulted<Inner> {
+    fn read_at(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        let length = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
+        match self.fault.note_a_read(self.identity, offset.0, length) {
+            ReadOutcome::Read => self.inner.read_at(offset, buffer),
+            ReadOutcome::DeviceError => Err(BlockDeviceError::InputOutput(std::io::Error::other(
+                "用例点名的根环槽读坏",
+            ))),
+            ReadOutcome::FlipTheByteAt(index) => {
+                self.inner.read_at(offset, buffer)?;
+                buffer[index] = !buffer[index];
+                Ok(())
+            }
+        }
+    }
+
+    fn write_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+        durability: WriteDurability,
+    ) -> Result<(), BlockDeviceError> {
+        self.fault.note_a_write();
+        self.inner.write_at(offset, bytes, durability)
+    }
+
+    fn write_zeroes_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        self.fault.note_a_write();
+        self.inner.write_zeroes_at(offset, length)
+    }
+
+    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
+        self.inner.barrier()
+    }
+
+    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
+        self.inner.probe_physical_block_size()
+    }
+
+    fn size_in_bytes(&self) -> u64 {
+        self.inner.size_in_bytes()
+    }
+}
+
+fn with_the_fault<Inner: BlockDevice>(
+    devices: Vec<(DeviceIdentity, Inner)>,
+    fault: &FaultOfOneRootRingSlot,
+) -> Vec<(DeviceIdentity, DeviceWithOneRootRingSlotFaulted<Inner>)> {
+    devices
+        .into_iter()
+        .map(|(identity, inner)| {
+            (
+                identity,
+                DeviceWithOneRootRingSlotFaulted {
+                    identity,
+                    inner,
+                    fault: fault.clone(),
+                },
+            )
+        })
+        .collect()
+}
+
+type RingHoleDevicesWithTheFault = Vec<(
+    DeviceIdentity,
+    DeviceWithOneRootRingSlotFaulted<RecordingBlockDevice<SparseBlockDevice>>,
+)>;
+
+/// S = 4 的小池上一次可写挂载之后的会话：盘（一个根环槽带读故障）、录下的流与它的起点盘面、分配器、现行那一版。
+struct SessionOnARingHolePool {
+    image_before_the_mount: MemoryPool,
+    stream: SharedStream,
+    devices: RingHoleDevicesWithTheFault,
+    fault: FaultOfOneRootRingSlot,
+    ring_slot: RootRingSlot,
+    allocator: PoolAllocator,
+    current: TransactionOutput,
+    instance: InstanceGeneration,
+}
+
+impl SessionOnARingHolePool {
+    /// `image` 上可写挂载，txg 为 `txg_of_the_slot` 的那一槽挂载那一刻每次读都报设备错，挂载之后照读。
+    fn mounted_with_the_slot_of_txg_unreadable_at_mount(
+        image: MemoryPool,
+        txg_of_the_slot: u64,
+    ) -> Self {
+        let parameters = parameters_of_the_ring_hole_pools();
+        let fault = FaultOfOneRootRingSlot::of_the_root_slot_of_txg(
+            &parameters,
+            CheckpointTxg(txg_of_the_slot),
+            ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead,
+        );
+        let stream = SharedStream::retaining_contents();
+        let mut devices = with_the_fault(recording_of(&image, &stream), &fault);
+        let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
+        fault.set_reads(ReadsOfTheRootRingSlot::PassedThrough);
+        assert!(fault.failed_reads() > 0, "挂载那一次读那一槽读错了");
+        let ring_slot = target_for_publish(
+            CheckpointTxg(txg_of_the_slot),
+            parameters.geometry.root_ring_slots_per_region,
+        );
+        Self {
+            image_before_the_mount: image,
+            stream,
+            devices,
+            fault,
+            ring_slot,
+            instance: mounted.output.instance,
+            allocator: mounted.allocator,
+            current: mounted.current.into_file_version().expect("这一版带文件"),
+        }
+    }
+
+    fn image_now(&self) -> MemoryPool {
+        let mut image = self.image_before_the_mount.clone();
+        image.apply(&self.stream.retained_operations());
+        image
+    }
+
+    fn overwrite_once(&mut self) {
+        let parameters = parameters_of_the_ring_hole_pools();
+        let salt = self.current.root.checkpoint_txg.0 + 1;
+        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
+        self.current = publish_overwrite(
+            &mut writer,
+            &mut self.allocator,
+            &self.current,
+            FirstFile {
+                content: &content_of_the_version(salt),
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
+            },
+            self.instance,
+        )
+        .expect("覆盖写");
+    }
+
+    /// 根环表里这一槽按挂载时读不出记着的最大 txg（不再这样记着时 `None`）。
+    fn txg_recorded_as_unreadable_at_mount(&self) -> Option<CheckpointTxg> {
+        self.allocator
+            .root_ring_occupancy()
+            .expect("可写挂载装了根环表")
+            .ring_slots_unreadable_at_mount()
+            .get(&self.ring_slot)
+            .copied()
+    }
+
+    fn slot_is_known_to_hold_a_root(&self) -> bool {
+        self.allocator
+            .root_ring_occupancy()
+            .expect("可写挂载装了根环表")
+            .ring_slots_known_to_hold_a_root()
+            .contains(&self.ring_slot)
+    }
+}
+
+/// 攻方那一形推到重读的前一版：实例 1 覆盖写到 20；实例 2 可写挂载那一刻读 h = 18 那一槽报读错；之后覆盖写到 28，
+/// 每一版之后那几条不变量都不红。
+fn session_one_publish_before_the_reread() -> SessionOnARingHolePool {
+    let mut session = SessionOnARingHolePool::mounted_with_the_slot_of_txg_unreadable_at_mount(
+        pool_overwritten_through(LAST_TXG_OF_INSTANCE_ONE),
+        TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT,
+    );
+    assert_eq!(
+        session.txg_recorded_as_unreadable_at_mount(),
+        Some(CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)),
+        "前提：根环表把那一槽记成挂载时读不出、能住的最大 txg 18"
+    );
+    // 迭代上界：覆盖写到 txg 28；跨轮携带的是现行那一版。
+    while session.current.root.checkpoint_txg.0 < TXG_AT_WHICH_THE_SLOT_IS_REREAD - 1 {
+        session.overwrite_once();
+        assert_eq!(
+            violations_of(&session.image_now()),
+            Vec::<String>::new(),
+            "txg {} 之后",
+            session.current.root.checkpoint_txg.0
+        );
+    }
+    assert_eq!(
+        session.txg_recorded_as_unreadable_at_mount(),
+        Some(CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)),
+        "前提：txg 28 时窗口里还有比 18 旧的 17，那一槽没重读"
+    );
+    session
+}
+
+/// 场景（攻方 `e2_reread_flip 4 20 2 flip`，`FLIP_ONLY_AT_ANCHOR=1`）：发 txg 29 那一次覆盖写时只有一次读把那一槽读坏（一个字节取反，盘上不变），
+/// 那一次就是记这条根之前的重读。
+/// 预期：读得出而自证不过与仍读不出同一处置，那一槽照旧按 18 记着，回收门槛停在 18；txg 29 那一版 I-3.1 成立（改之前记账少 12 个槽）。
+#[test]
+fn a_valid_root_read_bad_once_on_the_reread_is_counted_at_the_highest_txg_its_slot_can_hold() {
+    let mut session = session_one_publish_before_the_reread();
+    session
+        .fault
+        .set_reads(ReadsOfTheRootRingSlot::OneByteFlippedOnTheNextReadOnly);
+    session.overwrite_once();
+    assert_eq!(
+        session.current.root.checkpoint_txg,
+        CheckpointTxg(TXG_AT_WHICH_THE_SLOT_IS_REREAD)
+    );
+    assert_eq!(
+        session.fault.flipped_reads(),
+        1,
+        "发 txg 29 那一次读那一槽、读坏了一次"
+    );
+    assert_eq!(
+        session.txg_recorded_as_unreadable_at_mount(),
+        Some(CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)),
+        "重读读到自证不过：照旧按能住的最大 txg 18 记着"
+    );
+    assert_eq!(
+        violations_of(&session.image_now()),
+        Vec::<String>::new(),
+        "txg 29 那一版：回收门槛没越过盘上 txg 18 那条有效根"
+    );
+}
+
+/// 场景：同一盘面，发 txg 29 那一次重读读得出（读故障只在挂载那一刻）。
+/// 预期：读出 txg 18 的根、按实例表有效、在窗口里：那一槽换成「重读读出的有效根 18」，算这个进程知道住着根的槽；txg 29 那一版 I-3.1 成立。
+#[test]
+fn a_valid_root_read_on_the_reread_inside_the_window_holds_the_reclaim_threshold_at_its_txg() {
+    let mut session = session_one_publish_before_the_reread();
+    session.overwrite_once();
+    assert_eq!(
+        session.current.root.checkpoint_txg,
+        CheckpointTxg(TXG_AT_WHICH_THE_SLOT_IS_REREAD)
+    );
+    assert_eq!(
+        (
+            session.txg_recorded_as_unreadable_at_mount(),
+            session.slot_is_known_to_hold_a_root()
+        ),
+        (None, true),
+        "重读读出有效根：不再按挂载时读不出记，那一槽算这个进程知道住着根的槽"
+    );
+    assert_eq!(
+        violations_of(&session.image_now()),
+        Vec::<String>::new(),
+        "txg 29 那一版：门槛照读出来的 18 算"
+    );
+}
+
+/// 场景：同一盘面，发 txg 29 那一次重读读出 txg 18 的有效根；之后那一槽每次读都报设备错，接着准入抬 F 抬到上限。
+/// 预期：算上限那一遍那一槽读不出、它是这个进程知道住着根的槽：重读一次仍读不出，在任何写之前拒
+/// （`RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`），盘上逐字节不变、录制流不多一步。
+#[test]
+fn a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write() {
+    let mut session = session_one_publish_before_the_reread();
+    session.overwrite_once();
+    assert!(
+        session.slot_is_known_to_hold_a_root(),
+        "前提：重读读出了有效根"
+    );
+    session
+        .fault
+        .set_reads(ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead);
+    let image_before = session.image_now();
+    let operations_before = session.stream.operations().len();
+    let refused = raise_rollback_floor_to_the_admission_ceiling(
+        &parameters_of_the_ring_hole_pools(),
+        &mut session.devices,
+        &mut session.allocator,
+        &mut session.current,
+        ShadowLedger::On,
+    );
+    assert!(
+        matches!(
+            refused,
+            Err(MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { ring_slot, .. })
+                if ring_slot == session.ring_slot
+        ),
+        "知道住着根的槽重读仍读不出就拒：{refused:?}"
+    );
+    assert_eq!(
+        session.stream.operations().len(),
+        operations_before,
+        "一个写、一道屏障都没发"
+    );
+    assert!(session.image_now() == image_before, "盘上逐字节不变");
+}
+
+/// 场景（攻方 `front_hole_eio 4 9 mount`，改法 A 的代价那一格）：实例 1 写到 3，实例 2 覆盖写到 8，txg 9 那次覆盖写崩在根槽写之前
+/// （记录已持久；环还没转过来，那一槽从没写过、读出来自证不过）；实例 3 可写挂载那一刻读那一槽报读错，之后照读；覆盖写到 9 + 2 × 3S。
+/// 钉住今天判成的样子：只有 txg 20 那一版（那一槽成了窗口里最小的那个、重读读到自证不过、照旧按 9 算）I-3.1 红——记账比遍历多，安全方向；
+/// 其余每一版那几条不变量都不红，txg 21 起那一槽出了窗口。
+#[test]
+fn a_front_hole_unreadable_at_mount_counts_more_allocated_only_on_the_version_that_rereads_it() {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let image_with_the_front_hole = {
+        let mut devices: RingHoleDevices = devices_of(&pool_overwritten_through(3));
+        let mounted = mount_writable(&parameters, &mut devices).expect("实例 2 的可写挂载");
+        let instance = mounted.output.instance;
+        let mut allocator = mounted.allocator.clone();
+        let mut current = mounted.current.into_file_version().expect("这一版带文件");
+        // 迭代上界：覆盖写到 txg 8。
+        while current.root.checkpoint_txg.0 + 1 < TXG_OF_THE_FRONT_HOLE {
+            let salt = current.root.checkpoint_txg.0 + 1;
+            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+            current = publish_overwrite(
+                &mut writer,
+                &mut allocator,
+                &current,
+                FirstFile {
+                    content: &content_of_the_version(salt),
+                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
+                },
+                instance,
+            )
+            .expect("覆盖写");
+        }
+        let before = pool_of(&devices);
+        let stream = SharedStream::retaining_contents();
+        let mut recording = recording_of(&before, &stream);
+        {
+            let mut writer = PoolWriter::new(&parameters, recording.as_mut_slice());
+            let published = publish_overwrite(
+                &mut writer,
+                &mut allocator.clone(),
+                &current,
+                FirstFile {
+                    content: &content_of_the_version(TXG_OF_THE_FRONT_HOLE),
+                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS
+                        + TXG_OF_THE_FRONT_HOLE,
+                },
+                instance,
+            )
+            .expect("洞那一次覆盖写");
+            assert_eq!(
+                published.root.checkpoint_txg,
+                CheckpointTxg(TXG_OF_THE_FRONT_HOLE)
+            );
+        }
+        let operations = stream.retained_operations();
+        let (writes, _segments) =
+            writes_and_segments(&operations, &fixed_geometry_of_the_ring_hole_pools());
+        let root_write = writes
+            .iter()
+            .position(|write| write.kind == StepKind::RootRecordFua)
+            .expect("那一次的根槽写");
+        let mut crashed = before;
+        crashed.apply_writes(&writes[..root_write]);
+        crashed
+    };
+    let mut session = SessionOnARingHolePool::mounted_with_the_slot_of_txg_unreadable_at_mount(
+        image_with_the_front_hole,
+        TXG_OF_THE_FRONT_HOLE,
+    );
+    assert_eq!(
+        session.txg_recorded_as_unreadable_at_mount(),
+        Some(CheckpointTxg(TXG_OF_THE_FRONT_HOLE)),
+        "前提：根环表把那一槽记成挂载时读不出、能住的最大 txg 9"
+    );
+    let ring_length = 3 * parameters.geometry.root_ring_slots_per_region.count();
+    let mut versions_with_a_violation = BTreeSet::new();
+    let mut invariants_violated_on_the_reread = Vec::new();
+    // 迭代上界：覆盖写到 9 + 2 × 3S；跨轮携带的是现行那一版与红过的版本。
+    while session.current.root.checkpoint_txg.0 < TXG_OF_THE_FRONT_HOLE + 2 * ring_length {
+        session.overwrite_once();
+        let violations = violations_of(&session.image_now());
+        if !violations.is_empty() {
+            versions_with_a_violation.insert(session.current.root.checkpoint_txg.0);
+        }
+        if session.current.root.checkpoint_txg.0 == TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD {
+            invariants_violated_on_the_reread = violations
+                .iter()
+                .map(|violation| violation.split('：').next().expect("不变量名").to_string())
+                .collect();
+            assert_eq!(
+                session.txg_recorded_as_unreadable_at_mount(),
+                Some(CheckpointTxg(TXG_OF_THE_FRONT_HOLE)),
+                "txg 20：重读读到自证不过（从没写过），照旧按 9 记着"
+            );
+        }
+    }
+    assert_eq!(
+        versions_with_a_violation,
+        BTreeSet::from([TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD]),
+        "只有重读那一版红"
+    );
+    assert_eq!(
+        invariants_violated_on_the_reread,
+        vec!["I-3.1".to_string()],
+        "红的是 I-3.1（记账比遍历多一截：门槛按 9 算，盘上窗口里最旧的有效根是 10）"
+    );
+}
+
+/// 场景：实例 1 覆盖写到 txg 5（最新那条根 a = 5）；崩溃恢复抛弃 a（它的根槽与数据单元一时读不出，择根落到 4，实例 2 写行 6、暖机 7），
+/// 实例 2 再覆盖写两次（8、9）；实例 3 可写挂载那一刻读 a 那一槽报读错，之后照读；覆盖写到 a + 3S − 1 = 28（S = 8 的 24 槽环）。
+/// 预期：记 txg 28 那条根之前 a 那一槽成了窗口里最小的那个，重读读出 txg 5 的根，按写行那一张实例表（实例 1 停在 4）被抛弃：当没有根，
+/// 那一槽从根环表拿掉、不算这个进程知道住着根的槽；txg 28 那一版那几条不变量（不含 I-7.4，见 [`INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT`]）不红。
+#[test]
+fn an_abandoned_root_read_on_the_reread_is_taken_as_no_root() {
+    let mut pool = build_pool("abandoned-root-read-on-the-e2-reread");
+    // 迭代上界：txg 4、5 两次覆盖写。
+    for _ in 0..2 {
+        let content = content_of_the_version(pool.output.root.checkpoint_txg.0 + 1);
+        overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
+    }
+    let newest = pool.output.clone();
+    let txg_of_the_abandoned_root = newest.root.checkpoint_txg.0;
+    let mounted_as_instance_two =
+        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &newest);
+    assert_eq!(
+        mounted_as_instance_two.output.instance,
+        InstanceGeneration(2),
+        "前提：实例 2 抛弃了 a"
+    );
+    // 迭代上界：实例 2 两次覆盖写。
+    for _ in 0..2 {
+        let previous = pool.output.clone();
+        pool.output = common::publish_overwrite_in_process(
+            &mut pool,
+            &previous,
+            &content_of_the_version(previous.root.checkpoint_txg.0 + 1),
+            FIXED_WRITE_TIME_SECONDS + previous.root.checkpoint_txg.0 + 1,
+            InstanceGeneration(2),
+        )
+        .expect("实例 2 覆盖写");
+    }
+    let parameters = common::parameters();
+    let fault = FaultOfOneRootRingSlot::of_the_root_slot_of_txg(
+        &parameters,
+        CheckpointTxg(txg_of_the_abandoned_root),
+        ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead,
+    );
+    let mut devices = with_the_fault(pool.reopen_recorded(), &fault);
+    let mounted = mount_writable(&parameters, &mut devices).expect("实例 3 的可写挂载");
+    fault.set_reads(ReadsOfTheRootRingSlot::PassedThrough);
+    assert!(fault.failed_reads() > 0, "挂载那一次读 a 那一槽读错了");
+    let ring_slot = target_for_publish(
+        CheckpointTxg(txg_of_the_abandoned_root),
+        parameters.geometry.root_ring_slots_per_region,
+    );
+    let mut allocator = mounted.allocator;
+    let occupancy_of = |allocator_of_the_mount: &PoolAllocator| {
+        let occupancy = allocator_of_the_mount
+            .root_ring_occupancy()
+            .expect("可写挂载装了根环表");
+        (
+            occupancy
+                .ring_slots_unreadable_at_mount()
+                .get(&ring_slot)
+                .copied(),
+            occupancy
+                .ring_slots_known_to_hold_a_root()
+                .contains(&ring_slot),
+        )
+    };
+    assert_eq!(
+        occupancy_of(&allocator),
+        (Some(CheckpointTxg(txg_of_the_abandoned_root)), false),
+        "前提：根环表把 a 那一槽记成挂载时读不出、能住的最大 txg 5"
+    );
+    let mut current = mounted.current.into_file_version().expect("这一版带文件");
+    let ring_length = 3 * parameters.geometry.root_ring_slots_per_region.count();
+    let txg_at_which_the_slot_is_reread = txg_of_the_abandoned_root + ring_length - 1;
+    // 迭代上界：覆盖写到 a + 3S − 1。
+    while current.root.checkpoint_txg.0 < txg_at_which_the_slot_is_reread {
+        let checkpoint_txg = current.root.checkpoint_txg.0 + 1;
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        current = publish_overwrite(
+            &mut writer,
+            &mut allocator,
+            &current,
+            FirstFile {
+                content: &content_of_the_version(checkpoint_txg),
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + checkpoint_txg,
+            },
+            InstanceGeneration(3),
+        )
+        .expect("实例 3 覆盖写");
+    }
+    assert_eq!(
+        occupancy_of(&allocator),
+        (None, false),
+        "重读读出被抛弃的根：当没有根，从根环表拿掉，不算知道住着根的槽"
+    );
+    assert_eq!(
+        violations_among(
+            &pool.memory_pool(),
+            &INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT
+        ),
+        Vec::<String>::new(),
+        "txg {txg_at_which_the_slot_is_reread} 那一版"
+    );
+}
+
+/// 场景：实例 1 覆盖写到 txg 3 之后可写挂载（实例 2 写行）。
+/// 预期：挂载交回的分配器上，进程内重读判「按实例表有效」用的那张表是写行那次发布写下的那一张（带实例 1 的行），
+/// 不是挂载时最新那条根指着的那一张——池级 checker 按最新那条根的实例表判有效，写行之后它带的就是写行那一张。
+#[test]
+fn the_instance_table_judging_the_rereads_after_the_mount_is_the_one_the_row_publish_wrote() {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let mut devices = devices_of(&pool_overwritten_through(3));
+    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
+    let table_of = |root: &RootRecord| {
+        instance_table_chain_of_root(&pool_of(&devices), root)
+            .expect("实例表读得出")
+            .records
+    };
+    let table_of_the_row_publish = table_of(mounted.output.row_publish.root());
+    assert_ne!(
+        table_of_the_row_publish,
+        table_of(&mounted.output.chosen_root),
+        "前提：写行那一张比挂载时最新那条根的多实例 1 那一行"
+    );
+    assert_eq!(
+        mounted
+            .allocator
+            .root_ring_occupancy()
+            .expect("可写挂载装了根环表")
+            .instance_table_judging_the_rereads(),
+        &table_of_the_row_publish,
+        "判有效用写行那一张"
+    );
+}
+
+/// 树表 0 条的池上一次可写挂载：录下来，崩在 txg 落在 `crash_before_the_root_of` 里的第一个根槽写之前（那次发布的单元与记录已持久、根槽没持久；
+/// 这次挂载没写那几个 txg 的根就整段落盘）。交回之后的盘面与这次挂载落盘的最后一个 txg（崩了的那一次是它的 txg）。
+fn mount_a_pool_without_file(
+    image: &MemoryPool,
+    crash_before_the_root_of: &[u64],
+) -> (MemoryPool, u64) {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let stream = SharedStream::retaining_contents();
+    let mut recording = recording_of(image, &stream);
+    mount_writable(&parameters, &mut recording).expect("树表 0 条的池可写挂载");
+    let (writes, _segments) = writes_and_segments(
+        &stream.retained_operations(),
+        &fixed_geometry_of_the_ring_hole_pools(),
+    );
+    let txg_of_the_root_written_by = |write: &singlefs_harness::memory_pool::RetainedWrite| {
+        write
+            .bytes()
+            .and_then(|bytes| RootRecord::parse_slot(bytes, &parameters.filesystem_identifier))
+            .map(|root| root.checkpoint_txg.0)
+    };
+    let cut = writes.iter().position(|write| {
+        write.kind == StepKind::RootRecordFua
+            && txg_of_the_root_written_by(write)
+                .is_some_and(|txg| crash_before_the_root_of.contains(&txg))
+    });
+    let mut after = image.clone();
+    match cut {
+        Some(cut) => {
+            let txg = txg_of_the_root_written_by(&writes[cut]).expect("根槽写带着一条根");
+            after.apply_writes(&writes[..cut]);
+            (after, txg)
+        }
+        None => {
+            after.apply_writes(&writes);
+            let last = writes
+                .iter()
+                .filter(|write| write.kind == StepKind::RootRecordFua)
+                .filter_map(txg_of_the_root_written_by)
+                .max()
+                .expect("可写挂载至少写一条根");
+            (after, last)
+        }
+    }
+}
+
+/// mkfs 之后一直不写文件，接连可写挂载、崩掉 `crash_before_the_root_of` 里那几个 txg 的根，到下一次挂载的写行是 `first_txg_of_the_next_mount` 为止。
+fn pool_without_file_before_the_mount_writing_its_row_at(
+    crash_before_the_root_of: &[u64],
+    first_txg_of_the_next_mount: u64,
+) -> MemoryPool {
+    let parameters = parameters_of_the_ring_hole_pools();
+    let mut devices: RingHoleDevices = (0..2u32)
+        .map(|number| {
+            (
+                DeviceIdentity(number),
+                SparseBlockDevice::new(
+                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
+                    PhysicalBlockSizeInBytes(512),
+                ),
+            )
+        })
+        .collect();
+    make_filesystem(&parameters, &mut devices).expect("mkfs");
+    let mut image = pool_of(&devices);
+    let mut last_txg = 0u64;
+    // 迭代上界：每次挂载至少前进一个 txg，到下一次挂载的写行 txg 为止；跨轮携带的是盘面与落盘的最后一个 txg。
+    while last_txg + 1 < first_txg_of_the_next_mount {
+        let (after, last) = mount_a_pool_without_file(&image, crash_before_the_root_of);
+        assert!(last > last_txg, "每次挂载都往前推");
+        image = after;
+        last_txg = last;
+    }
+    assert_eq!(
+        last_txg + 1,
+        first_txg_of_the_next_mount,
+        "前提：下一次挂载的写行落在 {first_txg_of_the_next_mount}"
+    );
+    image
+}
+
+/// 场景：树表 0 条的池（S = 4），mkfs 之后接连可写挂载、从不写文件；txg 14 那次写行崩在根槽写之前（那一槽里留着上一圈 txg 2 的根），
+/// txg 23 那次写行同样崩在根槽写之前；下一次可写挂载写行 24、暖机 25（两块盘各一条）。挂载那一刻（取号之前）txg 14 那一槽读不出，之后照读。
+/// 预期：写行 24 那次窗口里还有比 14 旧的 13，不重读；暖机 25 那次零单元发布记根时那一槽成了窗口里最小的那个，重读读出 txg 2 的根：
+/// 挂载交回的根环表里那一槽换成重读读出的有效根，不再按挂载时读不出记，算这个进程知道住着根的槽。
+#[test]
+fn the_warm_up_on_a_version_without_file_rereads_the_slot_unreadable_at_mount() {
+    let image = pool_without_file_before_the_mount_writing_its_row_at(&[14, 23], 24);
+    let parameters = parameters_of_the_ring_hole_pools();
+    let fault = FaultOfOneRootRingSlot::of_the_root_slot_of_txg(
+        &parameters,
+        CheckpointTxg(14),
+        ReadsOfTheRootRingSlot::DeviceErrorUntilTheFirstWriteOfAnyDevice,
+    );
+    let stream = SharedStream::retaining_contents();
+    let mut devices = with_the_fault(recording_of(&image, &stream), &fault);
+    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
+    assert_eq!(
+        (
+            mounted.output.row_publish.root().checkpoint_txg,
+            mounted
+                .output
+                .warm_up_publishes
+                .iter()
+                .map(|version| version.root().checkpoint_txg)
+                .collect::<Vec<_>>()
+        ),
+        (CheckpointTxg(24), vec![CheckpointTxg(25)]),
+        "前提：写行 24、暖机一次 25"
+    );
+    assert!(fault.failed_reads() > 0, "前提：挂载读阶段读那一槽读错");
+    let ring_slot = target_for_publish(
+        CheckpointTxg(14),
+        parameters.geometry.root_ring_slots_per_region,
+    );
+    let occupancy = mounted
+        .allocator
+        .root_ring_occupancy()
+        .expect("可写挂载装了根环表");
+    assert_eq!(
+        (
+            occupancy
+                .ring_slots_unreadable_at_mount()
+                .get(&ring_slot)
+                .copied(),
+            occupancy
+                .ring_slots_known_to_hold_a_root()
+                .contains(&ring_slot)
+        ),
+        (None, true),
+        "暖机那次记根之前重读了那一槽"
+    );
+}
diff --git a/crates/singlefs-harness/tests/root_ring_slot_reread_that_does_not_self_verify_counts_as_still_unreadable.rs b/crates/singlefs-harness/tests/root_ring_slot_reread_that_does_not_self_verify_counts_as_still_unreadable.rs
deleted file mode 100644
index 26cf817..0000000
--- a/crates/singlefs-harness/tests/root_ring_slot_reread_that_does_not_self_verify_counts_as_still_unreadable.rs
+++ /dev/null
@@ -1,866 +0,0 @@
-//! 进程内记根时重读挂载那一刻读不出的根环槽（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」那一行的 E2 句与「根槽这一次读坏」那一行）：
-//! 重读那一次的几种结局各记成什么、判「按实例表有效」用哪张表、产品路径上记根的几处各自走到重读。
-//!
-//! - 读得出而自证不过，与仍读不出同一处置，按根环表记的最大 txg 算（用户 2026-10-04 定改法 A，代码轮第二轮判决
-//!   `research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1；攻方 `research/prompts/m3-root-ring-code-r2-opus-model/proto/src/bin/e2_reread_flip.rs`
-//!   那一形：一条有效根所在的槽挂载那一刻读错、它成了窗口里最小的那一刻那一次重读读坏，改之前回收门槛越过它、I-3.1 记账少）；
-//!   代价那一形（同一目录 `front_hole_eio.rs`：从没写过的槽加挂载那一刻读错）判成什么就钉什么。
-//! - 读得出、自证过、按实例表有效的照读出来的 txg 算，那一槽从此算「这个进程知道住着根的槽」（用户 2026-10-04 定）；
-//!   读得出、自证过而按实例表被抛弃的当没有根；判有效用的表写行之后是写行那一张（同一份判决第二节 RS2，正推腿探针 p1、p2）。
-//! - 记根的产品路径：带单元的发布（`transaction::publish_admitted`，已有变异行管它）、树表 0 条那一版上的暖机零单元发布
-//!   （`mount::publish_empty_after`）走到重读（探针 q1）。另两处没有会红的用例：第一次可写挂载的零单元写行发布
-//!   （`mount::establish_instance` 里 `rows_written` 为空那一支）走不到重读——那一刻根环里只有 mkfs 写的 txg 0，挂载时读不出的槽只可能是它，
-//!   而它读不出时择不到根、挂载不成；树表 0 条那一版上要写行的写行发布（`transaction::prepare_the_row_publish_on_a_version_without_file`，探针 q3）
-//!   重读的那一槽恰是紧跟着的暖机要盖掉的那一槽（写行 txg + 1），重读改的根环表与回收在暖机那一次被盖成同一个样子，那一版又没有记账树，
-//!   挂载交回之后看不出它重读没重读（交回报告里列着）。
-mod common;
-mod common_root_ring_holes;
-
-use std::cell::RefCell;
-use std::collections::BTreeSet;
-use std::rc::Rc;
-
-use common::{
-    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
-    overwrite_in_process_as_instance_one_and_advance_the_pool,
-    BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER, FIXED_WRITE_TIME_SECONDS,
-};
-use common_root_ring_holes::{
-    content_of_the_version, devices_of, fixed_geometry_of_the_ring_hole_pools,
-    parameters_of_the_ring_hole_pools, pool_of, pool_overwritten_through, recording_of,
-    RingHoleDevices, DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
-    WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS,
-};
-use singlefs_checker::image::InvariantVerdict;
-use singlefs_checker::walk::check_pool_image;
-use singlefs_core::address::{
-    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
-};
-use singlefs_core::allocator::PoolAllocator;
-use singlefs_core::block_device::{
-    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
-};
-use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
-use singlefs_core::mount::{
-    mount_writable, raise_rollback_floor_to_the_admission_ceiling, MountError, ShadowLedger,
-};
-use singlefs_core::recovery::instance_table_chain_of_root;
-use singlefs_core::root_record::RootRecord;
-use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
-use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
-use singlefs_harness::memory_pool::{writes_and_segments, MemoryPool, SparseBlockDevice};
-use singlefs_harness::segments::StepKind;
-use singlefs_harness::{RecordingBlockDevice, SharedStream};
-
-/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
-const INVARIANTS_THE_REREAD_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];
-/// 攻方那一形：实例 1 覆盖写到 txg 20，有效根 h = 18 所在的槽（区域 0、槽 2）挂载那一刻读错。
-const LAST_TXG_OF_INSTANCE_ONE: u64 = 20;
-const TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT: u64 = 18;
-/// h + 3S − 1：那一槽成了窗口里最小的那个、记这条根时重读它的那一版（S = 4）。
-const TXG_AT_WHICH_THE_SLOT_IS_REREAD: u64 = 29;
-/// 前洞那一形：实例 2 的 txg 9 那次覆盖写崩在根槽写之前（环还没转过来，那一槽从没写过）。
-const TXG_OF_THE_FRONT_HOLE: u64 = 9;
-/// 前洞那一形里那一槽成了窗口里最小的那个的那一版：9 + 3S − 1。
-const TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD: u64 = 20;
-
-/// 被抛弃根那一形判的几条：同上去掉 I-7.4。被抛弃的根所在的槽挂载那一刻读不出，影子账罩不到它引用的单元（挂载那一刻就读不到那条根，
-/// 与重读无关：攻方 `research/prompts/m3-root-ring-code-r2-opus-output.md` K1 第 3 小节「被抛弃」那一条），之后那几个单元被合法复用，
-/// I-7.4 在这一形上红是今天已知的那一格，不归这条用例判。
-const INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT: [&str; 4] = ["I-3.1", "I-3.11", "I-2.1", "I-4.8"];
-
-fn violations_of(image: &MemoryPool) -> Vec<String> {
-    violations_among(image, &INVARIANTS_THE_REREAD_KEEPS_GREEN)
-}
-
-fn violations_among(image: &MemoryPool, invariants_judged: &[&str]) -> Vec<String> {
-    check_pool_image(image)
-        .into_iter()
-        .filter(|(invariant, _)| invariants_judged.contains(invariant))
-        .filter_map(|(invariant, verdict)| match verdict {
-            InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
-        })
-        .collect()
-}
-
-/// 一个根环槽的读这会儿怎么坏。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-enum ReadsOfTheRootRingSlot {
-    PassedThrough,
-    DeviceErrorOnEveryRead,
-    /// 任何一块盘上发出第一个写（取号写系统配置）之前每次读都报设备错，之后照读：只在可写挂载读阶段读错。
-    DeviceErrorUntilTheFirstWriteOfAnyDevice,
-    /// 只有下一次读坏：读得出，那一槽里一个字节取反（自证不过，盘上字节不变）；之后照读。
-    OneByteFlippedOnTheNextReadOnly,
-}
-
-struct FaultOfOneRootRingSlotState {
-    device: DeviceIdentity,
-    offset_in_bytes: u64,
-    length_in_bytes: u64,
-    reads: ReadsOfTheRootRingSlot,
-    a_write_was_sent: bool,
-    failed_reads: u64,
-    flipped_reads: u64,
-}
-
-/// 几块盘共用的、一个根环槽的读故障。
-#[derive(Clone)]
-struct FaultOfOneRootRingSlot(Rc<RefCell<FaultOfOneRootRingSlotState>>);
-
-enum ReadOutcome {
-    Read,
-    DeviceError,
-    FlipTheByteAt(usize),
-}
-
-impl FaultOfOneRootRingSlot {
-    fn of_the_root_slot_of_txg(
-        parameters: &MakeFilesystemParameters,
-        checkpoint_txg: CheckpointTxg,
-        reads: ReadsOfTheRootRingSlot,
-    ) -> Self {
-        let ring_slot = target_for_publish(
-            checkpoint_txg,
-            parameters.geometry.root_ring_slots_per_region,
-        );
-        Self(Rc::new(RefCell::new(FaultOfOneRootRingSlotState {
-            device: parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")],
-            offset_in_bytes: slot_offset(
-                ring_slot,
-                parameters.geometry.fixed_structure_slot_spacing,
-            )
-            .0,
-            length_in_bytes: u64::from(parameters.geometry.physical_block_size),
-            reads,
-            a_write_was_sent: false,
-            failed_reads: 0,
-            flipped_reads: 0,
-        })))
-    }
-
-    fn set_reads(&self, reads: ReadsOfTheRootRingSlot) {
-        self.0.borrow_mut().reads = reads;
-    }
-
-    fn failed_reads(&self) -> u64 {
-        self.0.borrow().failed_reads
-    }
-
-    fn flipped_reads(&self) -> u64 {
-        self.0.borrow().flipped_reads
-    }
-
-    fn note_a_write(&self) {
-        self.0.borrow_mut().a_write_was_sent = true;
-    }
-
-    fn note_a_read(
-        &self,
-        device: DeviceIdentity,
-        offset_in_bytes: u64,
-        length: u64,
-    ) -> ReadOutcome {
-        let mut state = self.0.borrow_mut();
-        let overlaps = state.device == device
-            && offset_in_bytes < state.offset_in_bytes + state.length_in_bytes
-            && state.offset_in_bytes < offset_in_bytes + length;
-        if !overlaps {
-            return ReadOutcome::Read;
-        }
-        match state.reads {
-            ReadsOfTheRootRingSlot::PassedThrough => ReadOutcome::Read,
-            ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead => {
-                state.failed_reads += 1;
-                ReadOutcome::DeviceError
-            }
-            ReadsOfTheRootRingSlot::DeviceErrorUntilTheFirstWriteOfAnyDevice => {
-                if state.a_write_was_sent {
-                    ReadOutcome::Read
-                } else {
-                    state.failed_reads += 1;
-                    ReadOutcome::DeviceError
-                }
-            }
-            ReadsOfTheRootRingSlot::OneByteFlippedOnTheNextReadOnly => {
-                let byte_in_the_slot =
-                    state.offset_in_bytes + BYTE_INSIDE_THE_ROOT_RECORDS_TREE_TABLE_POINTER;
-                if byte_in_the_slot < offset_in_bytes
-                    || byte_in_the_slot >= offset_in_bytes + length
-                {
-                    return ReadOutcome::Read;
-                }
-                state.flipped_reads += 1;
-                state.reads = ReadsOfTheRootRingSlot::PassedThrough;
-                ReadOutcome::FlipTheByteAt(
-                    usize::try_from(byte_in_the_slot - offset_in_bytes).expect("落在缓冲区里"),
-                )
-            }
-        }
-    }
-}
-
-/// 读的时候按 [`FaultOfOneRootRingSlot`] 读坏一个根环槽的盘；写、写零、屏障原样交给里面那块盘（写记一笔：取号之后不再报读错的那一档要它）。
-struct DeviceWithOneRootRingSlotFaulted<Inner: BlockDevice> {
-    identity: DeviceIdentity,
-    inner: Inner,
-    fault: FaultOfOneRootRingSlot,
-}
-
-impl<Inner: BlockDevice> BlockDevice for DeviceWithOneRootRingSlotFaulted<Inner> {
-    fn read_at(
-        &self,
-        offset: DeviceOffsetInBytes,
-        buffer: &mut [u8],
-    ) -> Result<(), BlockDeviceError> {
-        let length = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
-        match self.fault.note_a_read(self.identity, offset.0, length) {
-            ReadOutcome::Read => self.inner.read_at(offset, buffer),
-            ReadOutcome::DeviceError => Err(BlockDeviceError::InputOutput(std::io::Error::other(
-                "用例点名的根环槽读坏",
-            ))),
-            ReadOutcome::FlipTheByteAt(index) => {
-                self.inner.read_at(offset, buffer)?;
-                buffer[index] = !buffer[index];
-                Ok(())
-            }
-        }
-    }
-
-    fn write_at(
-        &mut self,
-        offset: DeviceOffsetInBytes,
-        bytes: &[u8],
-        durability: WriteDurability,
-    ) -> Result<(), BlockDeviceError> {
-        self.fault.note_a_write();
-        self.inner.write_at(offset, bytes, durability)
-    }
-
-    fn write_zeroes_at(
-        &mut self,
-        offset: DeviceOffsetInBytes,
-        length: u64,
-    ) -> Result<(), BlockDeviceError> {
-        self.fault.note_a_write();
-        self.inner.write_zeroes_at(offset, length)
-    }
-
-    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
-        self.inner.barrier()
-    }
-
-    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
-        self.inner.probe_physical_block_size()
-    }
-
-    fn size_in_bytes(&self) -> u64 {
-        self.inner.size_in_bytes()
-    }
-}
-
-fn with_the_fault<Inner: BlockDevice>(
-    devices: Vec<(DeviceIdentity, Inner)>,
-    fault: &FaultOfOneRootRingSlot,
-) -> Vec<(DeviceIdentity, DeviceWithOneRootRingSlotFaulted<Inner>)> {
-    devices
-        .into_iter()
-        .map(|(identity, inner)| {
-            (
-                identity,
-                DeviceWithOneRootRingSlotFaulted {
-                    identity,
-                    inner,
-                    fault: fault.clone(),
-                },
-            )
-        })
-        .collect()
-}
-
-type RingHoleDevicesWithTheFault = Vec<(
-    DeviceIdentity,
-    DeviceWithOneRootRingSlotFaulted<RecordingBlockDevice<SparseBlockDevice>>,
-)>;
-
-/// S = 4 的小池上一次可写挂载之后的会话：盘（一个根环槽带读故障）、录下的流与它的起点盘面、分配器、现行那一版。
-struct SessionOnARingHolePool {
-    image_before_the_mount: MemoryPool,
-    stream: SharedStream,
-    devices: RingHoleDevicesWithTheFault,
-    fault: FaultOfOneRootRingSlot,
-    ring_slot: RootRingSlot,
-    allocator: PoolAllocator,
-    current: TransactionOutput,
-    instance: InstanceGeneration,
-}
-
-impl SessionOnARingHolePool {
-    /// `image` 上可写挂载，txg 为 `txg_of_the_slot` 的那一槽挂载那一刻每次读都报设备错，挂载之后照读。
-    fn mounted_with_the_slot_of_txg_unreadable_at_mount(
-        image: MemoryPool,
-        txg_of_the_slot: u64,
-    ) -> Self {
-        let parameters = parameters_of_the_ring_hole_pools();
-        let fault = FaultOfOneRootRingSlot::of_the_root_slot_of_txg(
-            &parameters,
-            CheckpointTxg(txg_of_the_slot),
-            ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead,
-        );
-        let stream = SharedStream::retaining_contents();
-        let mut devices = with_the_fault(recording_of(&image, &stream), &fault);
-        let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
-        fault.set_reads(ReadsOfTheRootRingSlot::PassedThrough);
-        assert!(fault.failed_reads() > 0, "挂载那一次读那一槽读错了");
-        let ring_slot = target_for_publish(
-            CheckpointTxg(txg_of_the_slot),
-            parameters.geometry.root_ring_slots_per_region,
-        );
-        Self {
-            image_before_the_mount: image,
-            stream,
-            devices,
-            fault,
-            ring_slot,
-            instance: mounted.output.instance,
-            allocator: mounted.allocator,
-            current: mounted.current.into_file_version().expect("这一版带文件"),
-        }
-    }
-
-    fn image_now(&self) -> MemoryPool {
-        let mut image = self.image_before_the_mount.clone();
-        image.apply(&self.stream.retained_operations());
-        image
-    }
-
-    fn overwrite_once(&mut self) {
-        let parameters = parameters_of_the_ring_hole_pools();
-        let salt = self.current.root.checkpoint_txg.0 + 1;
-        let mut writer = PoolWriter::new(&parameters, self.devices.as_mut_slice());
-        self.current = publish_overwrite(
-            &mut writer,
-            &mut self.allocator,
-            &self.current,
-            FirstFile {
-                content: &content_of_the_version(salt),
-                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
-            },
-            self.instance,
-        )
-        .expect("覆盖写");
-    }
-
-    /// 根环表里这一槽按挂载时读不出记着的最大 txg（不再这样记着时 `None`）。
-    fn txg_recorded_as_unreadable_at_mount(&self) -> Option<CheckpointTxg> {
-        self.allocator
-            .root_ring_occupancy()
-            .expect("可写挂载装了根环表")
-            .ring_slots_unreadable_at_mount()
-            .get(&self.ring_slot)
-            .copied()
-    }
-
-    fn slot_is_known_to_hold_a_root(&self) -> bool {
-        self.allocator
-            .root_ring_occupancy()
-            .expect("可写挂载装了根环表")
-            .ring_slots_known_to_hold_a_root()
-            .contains(&self.ring_slot)
-    }
-}
-
-/// 攻方那一形推到重读的前一版：实例 1 覆盖写到 20；实例 2 可写挂载那一刻读 h = 18 那一槽报读错；之后覆盖写到 28，
-/// 每一版之后那几条不变量都不红。
-fn session_one_publish_before_the_reread() -> SessionOnARingHolePool {
-    let mut session = SessionOnARingHolePool::mounted_with_the_slot_of_txg_unreadable_at_mount(
-        pool_overwritten_through(LAST_TXG_OF_INSTANCE_ONE),
-        TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT,
-    );
-    assert_eq!(
-        session.txg_recorded_as_unreadable_at_mount(),
-        Some(CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)),
-        "前提：根环表把那一槽记成挂载时读不出、能住的最大 txg 18"
-    );
-    // 迭代上界：覆盖写到 txg 28；跨轮携带的是现行那一版。
-    while session.current.root.checkpoint_txg.0 < TXG_AT_WHICH_THE_SLOT_IS_REREAD - 1 {
-        session.overwrite_once();
-        assert_eq!(
-            violations_of(&session.image_now()),
-            Vec::<String>::new(),
-            "txg {} 之后",
-            session.current.root.checkpoint_txg.0
-        );
-    }
-    assert_eq!(
-        session.txg_recorded_as_unreadable_at_mount(),
-        Some(CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)),
-        "前提：txg 28 时窗口里还有比 18 旧的 17，那一槽没重读"
-    );
-    session
-}
-
-/// 场景（攻方 `e2_reread_flip 4 20 2 flip`，`FLIP_ONLY_AT_ANCHOR=1`）：发 txg 29 那一次覆盖写时只有一次读把那一槽读坏（一个字节取反，盘上不变），
-/// 那一次就是记这条根之前的重读。
-/// 预期：读得出而自证不过与仍读不出同一处置，那一槽照旧按 18 记着，回收门槛停在 18；txg 29 那一版 I-3.1 成立（改之前记账少 12 个槽）。
-#[test]
-fn a_valid_root_read_bad_once_on_the_reread_is_counted_at_the_highest_txg_its_slot_can_hold() {
-    let mut session = session_one_publish_before_the_reread();
-    session
-        .fault
-        .set_reads(ReadsOfTheRootRingSlot::OneByteFlippedOnTheNextReadOnly);
-    session.overwrite_once();
-    assert_eq!(
-        session.current.root.checkpoint_txg,
-        CheckpointTxg(TXG_AT_WHICH_THE_SLOT_IS_REREAD)
-    );
-    assert_eq!(
-        session.fault.flipped_reads(),
-        1,
-        "发 txg 29 那一次读那一槽、读坏了一次"
-    );
-    assert_eq!(
-        session.txg_recorded_as_unreadable_at_mount(),
-        Some(CheckpointTxg(TXG_OF_THE_VALID_ROOT_UNREADABLE_AT_MOUNT)),
-        "重读读到自证不过：照旧按能住的最大 txg 18 记着"
-    );
-    assert_eq!(
-        violations_of(&session.image_now()),
-        Vec::<String>::new(),
-        "txg 29 那一版：回收门槛没越过盘上 txg 18 那条有效根"
-    );
-}
-
-/// 场景：同一盘面，发 txg 29 那一次重读读得出（读故障只在挂载那一刻）。
-/// 预期：读出 txg 18 的根、按实例表有效、在窗口里：那一槽换成「重读读出的有效根 18」，算这个进程知道住着根的槽；txg 29 那一版 I-3.1 成立。
-#[test]
-fn a_valid_root_read_on_the_reread_inside_the_window_holds_the_reclaim_threshold_at_its_txg() {
-    let mut session = session_one_publish_before_the_reread();
-    session.overwrite_once();
-    assert_eq!(
-        session.current.root.checkpoint_txg,
-        CheckpointTxg(TXG_AT_WHICH_THE_SLOT_IS_REREAD)
-    );
-    assert_eq!(
-        (
-            session.txg_recorded_as_unreadable_at_mount(),
-            session.slot_is_known_to_hold_a_root()
-        ),
-        (None, true),
-        "重读读出有效根：不再按挂载时读不出记，那一槽算这个进程知道住着根的槽"
-    );
-    assert_eq!(
-        violations_of(&session.image_now()),
-        Vec::<String>::new(),
-        "txg 29 那一版：门槛照读出来的 18 算"
-    );
-}
-
-/// 场景：同一盘面，发 txg 29 那一次重读读出 txg 18 的有效根；之后那一槽每次读都报设备错，接着准入抬 F 抬到上限。
-/// 预期：算上限那一遍那一槽读不出、它是这个进程知道住着根的槽：重读一次仍读不出，在任何写之前拒
-/// （`RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`），盘上逐字节不变、录制流不多一步。
-#[test]
-fn a_slot_whose_reread_read_a_valid_root_staying_unreadable_refuses_the_raise_before_any_write() {
-    let mut session = session_one_publish_before_the_reread();
-    session.overwrite_once();
-    assert!(
-        session.slot_is_known_to_hold_a_root(),
-        "前提：重读读出了有效根"
-    );
-    session
-        .fault
-        .set_reads(ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead);
-    let image_before = session.image_now();
-    let operations_before = session.stream.operations().len();
-    let refused = raise_rollback_floor_to_the_admission_ceiling(
-        &parameters_of_the_ring_hole_pools(),
-        &mut session.devices,
-        &mut session.allocator,
-        &mut session.current,
-        ShadowLedger::On,
-    );
-    assert!(
-        matches!(
-            refused,
-            Err(MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { ring_slot, .. })
-                if ring_slot == session.ring_slot
-        ),
-        "知道住着根的槽重读仍读不出就拒：{refused:?}"
-    );
-    assert_eq!(
-        session.stream.operations().len(),
-        operations_before,
-        "一个写、一道屏障都没发"
-    );
-    assert!(session.image_now() == image_before, "盘上逐字节不变");
-}
-
-/// 场景（攻方 `front_hole_eio 4 9 mount`，改法 A 的代价那一格）：实例 1 写到 3，实例 2 覆盖写到 8，txg 9 那次覆盖写崩在根槽写之前
-/// （记录已持久；环还没转过来，那一槽从没写过、读出来自证不过）；实例 3 可写挂载那一刻读那一槽报读错，之后照读；覆盖写到 9 + 2 × 3S。
-/// 钉住今天判成的样子：只有 txg 20 那一版（那一槽成了窗口里最小的那个、重读读到自证不过、照旧按 9 算）I-3.1 红——记账比遍历多，安全方向；
-/// 其余每一版那几条不变量都不红，txg 21 起那一槽出了窗口。
-#[test]
-fn a_front_hole_unreadable_at_mount_counts_more_allocated_only_on_the_version_that_rereads_it() {
-    let parameters = parameters_of_the_ring_hole_pools();
-    let image_with_the_front_hole = {
-        let mut devices: RingHoleDevices = devices_of(&pool_overwritten_through(3));
-        let mounted = mount_writable(&parameters, &mut devices).expect("实例 2 的可写挂载");
-        let instance = mounted.output.instance;
-        let mut allocator = mounted.allocator.clone();
-        let mut current = mounted.current.into_file_version().expect("这一版带文件");
-        // 迭代上界：覆盖写到 txg 8。
-        while current.root.checkpoint_txg.0 + 1 < TXG_OF_THE_FRONT_HOLE {
-            let salt = current.root.checkpoint_txg.0 + 1;
-            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
-            current = publish_overwrite(
-                &mut writer,
-                &mut allocator,
-                &current,
-                FirstFile {
-                    content: &content_of_the_version(salt),
-                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS + salt,
-                },
-                instance,
-            )
-            .expect("覆盖写");
-        }
-        let before = pool_of(&devices);
-        let stream = SharedStream::retaining_contents();
-        let mut recording = recording_of(&before, &stream);
-        {
-            let mut writer = PoolWriter::new(&parameters, recording.as_mut_slice());
-            let published = publish_overwrite(
-                &mut writer,
-                &mut allocator.clone(),
-                &current,
-                FirstFile {
-                    content: &content_of_the_version(TXG_OF_THE_FRONT_HOLE),
-                    write_time_seconds: WRITE_TIME_SECONDS_OF_THE_RING_HOLE_POOLS
-                        + TXG_OF_THE_FRONT_HOLE,
-                },
-                instance,
-            )
-            .expect("洞那一次覆盖写");
-            assert_eq!(
-                published.root.checkpoint_txg,
-                CheckpointTxg(TXG_OF_THE_FRONT_HOLE)
-            );
-        }
-        let operations = stream.retained_operations();
-        let (writes, _segments) =
-            writes_and_segments(&operations, &fixed_geometry_of_the_ring_hole_pools());
-        let root_write = writes
-            .iter()
-            .position(|write| write.kind == StepKind::RootRecordFua)
-            .expect("那一次的根槽写");
-        let mut crashed = before;
-        crashed.apply_writes(&writes[..root_write]);
-        crashed
-    };
-    let mut session = SessionOnARingHolePool::mounted_with_the_slot_of_txg_unreadable_at_mount(
-        image_with_the_front_hole,
-        TXG_OF_THE_FRONT_HOLE,
-    );
-    assert_eq!(
-        session.txg_recorded_as_unreadable_at_mount(),
-        Some(CheckpointTxg(TXG_OF_THE_FRONT_HOLE)),
-        "前提：根环表把那一槽记成挂载时读不出、能住的最大 txg 9"
-    );
-    let ring_length = 3 * parameters.geometry.root_ring_slots_per_region.count();
-    let mut versions_with_a_violation = BTreeSet::new();
-    let mut invariants_violated_on_the_reread = Vec::new();
-    // 迭代上界：覆盖写到 9 + 2 × 3S；跨轮携带的是现行那一版与红过的版本。
-    while session.current.root.checkpoint_txg.0 < TXG_OF_THE_FRONT_HOLE + 2 * ring_length {
-        session.overwrite_once();
-        let violations = violations_of(&session.image_now());
-        if !violations.is_empty() {
-            versions_with_a_violation.insert(session.current.root.checkpoint_txg.0);
-        }
-        if session.current.root.checkpoint_txg.0 == TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD {
-            invariants_violated_on_the_reread = violations
-                .iter()
-                .map(|violation| violation.split('：').next().expect("不变量名").to_string())
-                .collect();
-            assert_eq!(
-                session.txg_recorded_as_unreadable_at_mount(),
-                Some(CheckpointTxg(TXG_OF_THE_FRONT_HOLE)),
-                "txg 20：重读读到自证不过（从没写过），照旧按 9 记着"
-            );
-        }
-    }
-    assert_eq!(
-        versions_with_a_violation,
-        BTreeSet::from([TXG_AT_WHICH_THE_FRONT_HOLE_IS_REREAD]),
-        "只有重读那一版红"
-    );
-    assert_eq!(
-        invariants_violated_on_the_reread,
-        vec!["I-3.1".to_string()],
-        "红的是 I-3.1（记账比遍历多一截：门槛按 9 算，盘上窗口里最旧的有效根是 10）"
-    );
-}
-
-/// 场景：实例 1 覆盖写到 txg 5（最新那条根 a = 5）；崩溃恢复抛弃 a（它的根槽与数据单元一时读不出，择根落到 4，实例 2 写行 6、暖机 7），
-/// 实例 2 再覆盖写两次（8、9）；实例 3 可写挂载那一刻读 a 那一槽报读错，之后照读；覆盖写到 a + 3S − 1 = 28（S = 8 的 24 槽环）。
-/// 预期：记 txg 28 那条根之前 a 那一槽成了窗口里最小的那个，重读读出 txg 5 的根，按写行那一张实例表（实例 1 停在 4）被抛弃：当没有根，
-/// 那一槽从根环表拿掉、不算这个进程知道住着根的槽；txg 28 那一版那几条不变量（不含 I-7.4，见 [`INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT`]）不红。
-#[test]
-fn an_abandoned_root_read_on_the_reread_is_taken_as_no_root() {
-    let mut pool = build_pool("abandoned-root-read-on-the-e2-reread");
-    // 迭代上界：txg 4、5 两次覆盖写。
-    for _ in 0..2 {
-        let content = content_of_the_version(pool.output.root.checkpoint_txg.0 + 1);
-        overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
-    }
-    let newest = pool.output.clone();
-    let txg_of_the_abandoned_root = newest.root.checkpoint_txg.0;
-    let mounted_as_instance_two =
-        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(&mut pool, &newest);
-    assert_eq!(
-        mounted_as_instance_two.output.instance,
-        InstanceGeneration(2),
-        "前提：实例 2 抛弃了 a"
-    );
-    // 迭代上界：实例 2 两次覆盖写。
-    for _ in 0..2 {
-        let previous = pool.output.clone();
-        pool.output = common::publish_overwrite_in_process(
-            &mut pool,
-            &previous,
-            &content_of_the_version(previous.root.checkpoint_txg.0 + 1),
-            FIXED_WRITE_TIME_SECONDS + previous.root.checkpoint_txg.0 + 1,
-            InstanceGeneration(2),
-        )
-        .expect("实例 2 覆盖写");
-    }
-    let parameters = common::parameters();
-    let fault = FaultOfOneRootRingSlot::of_the_root_slot_of_txg(
-        &parameters,
-        CheckpointTxg(txg_of_the_abandoned_root),
-        ReadsOfTheRootRingSlot::DeviceErrorOnEveryRead,
-    );
-    let mut devices = with_the_fault(pool.reopen_recorded(), &fault);
-    let mounted = mount_writable(&parameters, &mut devices).expect("实例 3 的可写挂载");
-    fault.set_reads(ReadsOfTheRootRingSlot::PassedThrough);
-    assert!(fault.failed_reads() > 0, "挂载那一次读 a 那一槽读错了");
-    let ring_slot = target_for_publish(
-        CheckpointTxg(txg_of_the_abandoned_root),
-        parameters.geometry.root_ring_slots_per_region,
-    );
-    let mut allocator = mounted.allocator;
-    let occupancy_of = |allocator_of_the_mount: &PoolAllocator| {
-        let occupancy = allocator_of_the_mount
-            .root_ring_occupancy()
-            .expect("可写挂载装了根环表");
-        (
-            occupancy
-                .ring_slots_unreadable_at_mount()
-                .get(&ring_slot)
-                .copied(),
-            occupancy
-                .ring_slots_known_to_hold_a_root()
-                .contains(&ring_slot),
-        )
-    };
-    assert_eq!(
-        occupancy_of(&allocator),
-        (Some(CheckpointTxg(txg_of_the_abandoned_root)), false),
-        "前提：根环表把 a 那一槽记成挂载时读不出、能住的最大 txg 5"
-    );
-    let mut current = mounted.current.into_file_version().expect("这一版带文件");
-    let ring_length = 3 * parameters.geometry.root_ring_slots_per_region.count();
-    let txg_at_which_the_slot_is_reread = txg_of_the_abandoned_root + ring_length - 1;
-    // 迭代上界：覆盖写到 a + 3S − 1。
-    while current.root.checkpoint_txg.0 < txg_at_which_the_slot_is_reread {
-        let checkpoint_txg = current.root.checkpoint_txg.0 + 1;
-        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
-        current = publish_overwrite(
-            &mut writer,
-            &mut allocator,
-            &current,
-            FirstFile {
-                content: &content_of_the_version(checkpoint_txg),
-                write_time_seconds: FIXED_WRITE_TIME_SECONDS + checkpoint_txg,
-            },
-            InstanceGeneration(3),
-        )
-        .expect("实例 3 覆盖写");
-    }
-    assert_eq!(
-        occupancy_of(&allocator),
-        (None, false),
-        "重读读出被抛弃的根：当没有根，从根环表拿掉，不算知道住着根的槽"
-    );
-    assert_eq!(
-        violations_among(
-            &pool.memory_pool(),
-            &INVARIANTS_JUDGED_ON_THE_ABANDONED_ROOT
-        ),
-        Vec::<String>::new(),
-        "txg {txg_at_which_the_slot_is_reread} 那一版"
-    );
-}
-
-/// 场景：实例 1 覆盖写到 txg 3 之后可写挂载（实例 2 写行）。
-/// 预期：挂载交回的分配器上，进程内重读判「按实例表有效」用的那张表是写行那次发布写下的那一张（带实例 1 的行），
-/// 不是挂载时最新那条根指着的那一张——池级 checker 按最新那条根的实例表判有效，写行之后它带的就是写行那一张。
-#[test]
-fn the_instance_table_judging_the_rereads_after_the_mount_is_the_one_the_row_publish_wrote() {
-    let parameters = parameters_of_the_ring_hole_pools();
-    let mut devices = devices_of(&pool_overwritten_through(3));
-    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
-    let table_of = |root: &RootRecord| {
-        instance_table_chain_of_root(&pool_of(&devices), root)
-            .expect("实例表读得出")
-            .records
-    };
-    let table_of_the_row_publish = table_of(mounted.output.row_publish.root());
-    assert_ne!(
-        table_of_the_row_publish,
-        table_of(&mounted.output.chosen_root),
-        "前提：写行那一张比挂载时最新那条根的多实例 1 那一行"
-    );
-    assert_eq!(
-        mounted
-            .allocator
-            .root_ring_occupancy()
-            .expect("可写挂载装了根环表")
-            .instance_table_judging_the_rereads(),
-        &table_of_the_row_publish,
-        "判有效用写行那一张"
-    );
-}
-
-/// 树表 0 条的池上一次可写挂载：录下来，崩在 txg 落在 `crash_before_the_root_of` 里的第一个根槽写之前（那次发布的单元与记录已持久、根槽没持久；
-/// 这次挂载没写那几个 txg 的根就整段落盘）。交回之后的盘面与这次挂载落盘的最后一个 txg（崩了的那一次是它的 txg）。
-fn mount_a_pool_without_file(
-    image: &MemoryPool,
-    crash_before_the_root_of: &[u64],
-) -> (MemoryPool, u64) {
-    let parameters = parameters_of_the_ring_hole_pools();
-    let stream = SharedStream::retaining_contents();
-    let mut recording = recording_of(image, &stream);
-    mount_writable(&parameters, &mut recording).expect("树表 0 条的池可写挂载");
-    let (writes, _segments) = writes_and_segments(
-        &stream.retained_operations(),
-        &fixed_geometry_of_the_ring_hole_pools(),
-    );
-    let txg_of_the_root_written_by = |write: &singlefs_harness::memory_pool::RetainedWrite| {
-        write
-            .bytes()
-            .and_then(|bytes| RootRecord::parse_slot(bytes, &parameters.filesystem_identifier))
-            .map(|root| root.checkpoint_txg.0)
-    };
-    let cut = writes.iter().position(|write| {
-        write.kind == StepKind::RootRecordFua
-            && txg_of_the_root_written_by(write)
-                .is_some_and(|txg| crash_before_the_root_of.contains(&txg))
-    });
-    let mut after = image.clone();
-    match cut {
-        Some(cut) => {
-            let txg = txg_of_the_root_written_by(&writes[cut]).expect("根槽写带着一条根");
-            after.apply_writes(&writes[..cut]);
-            (after, txg)
-        }
-        None => {
-            after.apply_writes(&writes);
-            let last = writes
-                .iter()
-                .filter(|write| write.kind == StepKind::RootRecordFua)
-                .filter_map(txg_of_the_root_written_by)
-                .max()
-                .expect("可写挂载至少写一条根");
-            (after, last)
-        }
-    }
-}
-
-/// mkfs 之后一直不写文件，接连可写挂载、崩掉 `crash_before_the_root_of` 里那几个 txg 的根，到下一次挂载的写行是 `first_txg_of_the_next_mount` 为止。
-fn pool_without_file_before_the_mount_writing_its_row_at(
-    crash_before_the_root_of: &[u64],
-    first_txg_of_the_next_mount: u64,
-) -> MemoryPool {
-    let parameters = parameters_of_the_ring_hole_pools();
-    let mut devices: RingHoleDevices = (0..2u32)
-        .map(|number| {
-            (
-                DeviceIdentity(number),
-                SparseBlockDevice::new(
-                    DEVICE_BYTES_OF_THE_RING_HOLE_POOLS,
-                    PhysicalBlockSizeInBytes(512),
-                ),
-            )
-        })
-        .collect();
-    make_filesystem(&parameters, &mut devices).expect("mkfs");
-    let mut image = pool_of(&devices);
-    let mut last_txg = 0u64;
-    // 迭代上界：每次挂载至少前进一个 txg，到下一次挂载的写行 txg 为止；跨轮携带的是盘面与落盘的最后一个 txg。
-    while last_txg + 1 < first_txg_of_the_next_mount {
-        let (after, last) = mount_a_pool_without_file(&image, crash_before_the_root_of);
-        assert!(last > last_txg, "每次挂载都往前推");
-        image = after;
-        last_txg = last;
-    }
-    assert_eq!(
-        last_txg + 1,
-        first_txg_of_the_next_mount,
-        "前提：下一次挂载的写行落在 {first_txg_of_the_next_mount}"
-    );
-    image
-}
-
-/// 场景：树表 0 条的池（S = 4），mkfs 之后接连可写挂载、从不写文件；txg 14 那次写行崩在根槽写之前（那一槽里留着上一圈 txg 2 的根），
-/// txg 23 那次写行同样崩在根槽写之前；下一次可写挂载写行 24、暖机 25（两块盘各一条）。挂载那一刻（取号之前）txg 14 那一槽读不出，之后照读。
-/// 预期：写行 24 那次窗口里还有比 14 旧的 13，不重读；暖机 25 那次零单元发布记根时那一槽成了窗口里最小的那个，重读读出 txg 2 的根：
-/// 挂载交回的根环表里那一槽换成重读读出的有效根，不再按挂载时读不出记，算这个进程知道住着根的槽。
-#[test]
-fn the_warm_up_on_a_version_without_file_rereads_the_slot_unreadable_at_mount() {
-    let image = pool_without_file_before_the_mount_writing_its_row_at(&[14, 23], 24);
-    let parameters = parameters_of_the_ring_hole_pools();
-    let fault = FaultOfOneRootRingSlot::of_the_root_slot_of_txg(
-        &parameters,
-        CheckpointTxg(14),
-        ReadsOfTheRootRingSlot::DeviceErrorUntilTheFirstWriteOfAnyDevice,
-    );
-    let stream = SharedStream::retaining_contents();
-    let mut devices = with_the_fault(recording_of(&image, &stream), &fault);
-    let mounted = mount_writable(&parameters, &mut devices).expect("可写挂载");
-    assert_eq!(
-        (
-            mounted.output.row_publish.root().checkpoint_txg,
-            mounted
-                .output
-                .warm_up_publishes
-                .iter()
-                .map(|version| version.root().checkpoint_txg)
-                .collect::<Vec<_>>()
-        ),
-        (CheckpointTxg(24), vec![CheckpointTxg(25)]),
-        "前提：写行 24、暖机一次 25"
-    );
-    assert!(fault.failed_reads() > 0, "前提：挂载读阶段读那一槽读错");
-    let ring_slot = target_for_publish(
-        CheckpointTxg(14),
-        parameters.geometry.root_ring_slots_per_region,
-    );
-    let occupancy = mounted
-        .allocator
-        .root_ring_occupancy()
-        .expect("可写挂载装了根环表");
-    assert_eq!(
-        (
-            occupancy
-                .ring_slots_unreadable_at_mount()
-                .get(&ring_slot)
-                .copied(),
-            occupancy
-                .ring_slots_known_to_hold_a_root()
-                .contains(&ring_slot)
-        ),
-        (None, true),
-        "暖机那次记根之前重读了那一槽"
-    );
-}
diff --git a/crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs b/crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs
index 4d54adb..689c2a9 100644
--- a/crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs
+++ b/crates/singlefs-harness/tests/root_ring_slot_unreadable_at_mount_is_reread_when_it_becomes_the_oldest_in_the_window.rs
@@ -47,7 +47,9 @@ fn violations_of(image: &MemoryPool) -> Vec<String> {
         .filter(|(invariant, _)| INVARIANTS_THE_REREAD_KEEPS_GREEN.contains(invariant))
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/row_publish_admission.rs b/crates/singlefs-harness/tests/row_publish_admission.rs
index ebf89a9..0ed1dc4 100644
--- a/crates/singlefs-harness/tests/row_publish_admission.rs
+++ b/crates/singlefs-harness/tests/row_publish_admission.rs
@@ -172,7 +172,9 @@ fn writable_mount_on_a_pool_past_the_former_allocation_record_wall_succeeds(
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect();
     assert_eq!(violated, Vec::new(), "挂载之后池级 checker 全绿");
diff --git a/crates/singlefs-harness/tests/row_publish_checks_before_acquisition.rs b/crates/singlefs-harness/tests/row_publish_checks_before_acquisition.rs
index fe50572..0e7f022 100644
--- a/crates/singlefs-harness/tests/row_publish_checks_before_acquisition.rs
+++ b/crates/singlefs-harness/tests/row_publish_checks_before_acquisition.rs
@@ -248,7 +248,9 @@ fn row_publishes_on_a_version_without_file_with_a_sixty_six_page_instance_table_
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect();
     assert_eq!(violated, Vec::new(), "十二次挂载之后池级 checker 全绿");
diff --git a/crates/singlefs-harness/tests/row_publish_record_without_its_root.rs b/crates/singlefs-harness/tests/row_publish_record_without_its_root.rs
index aee2ce9..dd2464a 100644
--- a/crates/singlefs-harness/tests/row_publish_record_without_its_root.rs
+++ b/crates/singlefs-harness/tests/row_publish_record_without_its_root.rs
@@ -251,7 +251,9 @@ fn c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_nev
             reason.contains("整单元校验和"),
             "不适用的理由要说没有校验和相符的单元可比：{reason}"
         ),
-        other @ (InvariantVerdict::Holds | InvariantVerdict::Violated(_)) => {
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
             panic!("两个孤单元改坏之后 I-8.11 没有可比的单元，该报不适用：{other:?}")
         }
     }
diff --git a/crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs b/crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs
index 025a9bd..166abb1 100644
--- a/crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs
+++ b/crates/singlefs-harness/tests/system_configuration_fields_core_and_checker_refuse_the_same_table.rs
@@ -131,7 +131,9 @@ fn core_and_checker_refuse_the_cell(
                     detail.contains(checker_names_member),
                     "{tag}：I-7.13 的违例说明里要点名 {checker_names_member}：{detail}"
                 ),
-                other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => {
+                other @ (InvariantVerdict::Holds
+                | InvariantVerdict::NotApplicable(_)
+                | InvariantVerdict::PartiallyJudged(_)) => {
                     panic!("{tag}：四槽都带这一版读者不收的值，I-7.13 要判违例，得到 {other:?}")
                 }
             }
diff --git a/crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs b/crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs
index 7a1db6b..0c53bfa 100644
--- a/crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs
+++ b/crates/singlefs-harness/tests/system_configuration_slot_one_read_at_every_spacing_by_the_checker_and_recovery.rs
@@ -180,7 +180,9 @@ fn violated_invariants(image: &MemoryPool) -> Vec<(&'static str, String)> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs b/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs
index 749e211..58c6cac 100644
--- a/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs
+++ b/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs
@@ -171,7 +171,9 @@ fn checker_violations(image: &MemoryPool) -> Vec<(&'static str, String)> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs b/crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs
index 5874aab..e32e82e 100644
--- a/crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs
+++ b/crates/singlefs-harness/tests/unlink_idempotence_criteria_and_rollback_to_the_emptied_version.rs
@@ -138,7 +138,9 @@ fn violations(image: &MemoryPool) -> Vec<String> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
index 076f3f2..310cdc4 100644
--- a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
+++ b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
@@ -162,7 +162,9 @@ fn violations(image: &MemoryPool) -> Vec<String> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs b/crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs
index 3649e9b..d71dd9f 100644
--- a/crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs
+++ b/crates/singlefs-harness/tests/unlink_of_a_file_spanning_two_extent_lower_leaves_in_product_batches.rs
@@ -36,7 +36,9 @@ fn violations(image: &MemoryPool) -> Vec<String> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/unlink_single_file_publishes.rs b/crates/singlefs-harness/tests/unlink_single_file_publishes.rs
index 1979905..56aba57 100644
--- a/crates/singlefs-harness/tests/unlink_single_file_publishes.rs
+++ b/crates/singlefs-harness/tests/unlink_single_file_publishes.rs
@@ -106,7 +106,9 @@ fn violations(image: &MemoryPool) -> Vec<String> {
         .into_iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some(format!("{invariant}: {detail}")),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect()
 }
diff --git a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
index 3cb98f1..12f0e4f 100644
--- a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
+++ b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
@@ -2148,7 +2148,9 @@ fn assert_only_the_abandoned_root_the_blind_mount_reused_reddens_the_reuse_invar
         .iter()
         .filter_map(|(invariant, verdict)| match verdict {
             InvariantVerdict::Violated(detail) => Some((*invariant, detail)),
-            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
         })
         .collect();
     assert_eq!(
````

## batch9-gpu2/patch

实现员报告：`/tmp/claude-1000/m3-r1/batch9-gpu2/patch/report.md`（sha256 9aa573320bf77cb4…），补丁 sha256 8632feb1fc93a0d5…

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -746,6 +746,22 @@
     return rd32(s.base, instance_offset);
 }
 
+// walk::slots_claimed_by_a_unit_header（D18（块里携带什么信息） 已定项 7 认领规则）：起点槽上解得出单元头——magic 对、类标签 1 / 2 / 3、
+// 头校验和按自述的类过（检查字 c2 位 0）——就认领它：码 1、码 3 两槽，码 2 一槽；解不出 ⇒ 0，这一槽不认领。不看 fsid
+fn slots_claimed_by_a_unit_header(s: Slice) -> u32 {
+    if (s.version == NONE || s.length < NODE_BYTES || rd32(s.base, 0u) != MAGIC_SFSU || (s.c2 & 1u) == 0u) {
+        return 0u;
+    }
+    let unit_class = rd8(s.base, 6u);
+    if (unit_class == 1u || unit_class == 3u) {
+        return DATA_UNIT_BYTES >> SLOT_SHIFT;
+    }
+    if (unit_class == 2u) {
+        return NODE_BYTES >> SLOT_SHIFT;
+    }
+    return 0u;
+}
+
 // 是不是有载体的实例代号高于 threshold（根、每块盘的 journal 记录、每块盘的单元头写序）
 fn some_instance_carrier_exceeds(threshold: u32, g: CheckerSlot) -> bool {
     for (var i = 0u; i < root_count; i = i + 1u) {
@@ -3622,16 +3638,31 @@
                         highest_seen = tree;
                     }
                 }
+                // 扫描方向照认领规则按整单元跳（walk.rs 的 scanned_tree_identifiers）：落点表按 (盘, 偏移) 升序，每块盘跨轮带
+                // 「下一个没被认领的槽」、只增不减；起点槽解得出单元头就认领它占的几槽，被认领的后几槽是它的载荷，不再读头
+                var claiming_device = NONE;
+                var first_slot_not_claimed = vec2<u32>(0u, 0u);
                 let location_count = hdr(6u);
                 for (var l = 0u; l < location_count; l = l + 1u) {
                     if (location_word(l, 4u) != LOCATION_UNIT_SLOT) {
                         continue;
                     }
+                    let device = location_word(l, 0u);
+                    let slot = shr64(vec2<u32>(location_word(l, 1u), location_word(l, 2u)), SLOT_SHIFT);
+                    if (device != claiming_device) {
+                        claiming_device = device;
+                        first_slot_not_claimed = vec2<u32>(0u, 0u);
+                    }
+                    if (lt64(slot, first_slot_not_claimed)) {
+                        continue;
+                    }
                     let s = read_location(l);
-                    if (s.version == NONE || s.length < NODE_BYTES || rd32(s.base, 0u) != MAGIC_SFSU || rd8(s.base, 6u) != 2u) {
+                    let claimed_slots = slots_claimed_by_a_unit_header(s);
+                    if (claimed_slots == 0u) {
                         continue;
                     }
-                    if (((s.c3 >> 4u) & 1u) == 0u) {
+                    first_slot_not_claimed = add64(slot, u64_of(claimed_slots));
+                    if (rd8(s.base, 6u) != 2u) {
                         continue;
                     }
                     let key_span = 2u * rd8(s.base, 51u);
@@ -3725,6 +3756,7 @@
                 sws(H_BASE + H_UNIT_AREA_START_LO, g.unit_area_start_slot.x);
                 sws(H_BASE + H_UNIT_AREA_START_HI, g.unit_area_start_slot.y);
                 sws(H_BASE + H_REGIONS, g.regions);
+                sws(H_BASE + H_SLOTS_PER_REGION, g.slots_per_region);
                 sws(H_BASE + H_REGION0, g.region0);
                 sws(H_BASE + H_REGION1, g.region1);
                 sws(H_BASE + H_REGION2, g.region2);
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
@@ -145,6 +145,9 @@
 
 const INVARIANT_I_5_5: u32 = 67u;
 
+// 第 69 条：I-9.28（全部可读根里同一树 ID 的诞生 txg 相同），扫描内核挨着 I-9.14 判
+const INVARIANT_I_9_28: u32 = 68u;
+
 // I-9.27 读的另外四个种类码（walk.rs 的 TREE_KIND_*；1..8 在 crash_judge_common.wgsl）
 const TREE_KIND_DELETED_INODES: u32 = 9u;
 
@@ -366,6 +369,8 @@
 
 const C_MAX_TREES_PER_TABLE: u32 = 8u;
 
+// I-9.28 在 I-3.9 与 I-9.14 判完之后借这几行，根表里第 k 条根记进第 k 行（只用 1..5 与每树那几字）：根表最多 MAX_ROOTS 条，
+// 不多于 C_MAX_CANDIDATES 行（judge_tree_identifier_birth_txg_across_every_readable_root 开头判，越过就判不了）
 const C_CANDIDATE_INFO: u32 = 166416u;
 
 const C_ALLOC_RECORD_WORDS: u32 = 8u;        // 盘、槽 lo/hi、跨度、代 lo/hi、已释放、0
@@ -684,6 +689,7 @@
 const H_FSID2: u32 = 26u;
 const H_FSID3: u32 = 27u;
 const H_BELOW_FLOOR_SLOT_COUNT: u32 = 28u;
+const H_SLOTS_PER_REGION: u32 = 29u;     // 根环每区槽数 S：I-7.9 的一圈窗口宽 3S = H_REGIONS × 它
 const H_MAX_BELOW_FLOOR_SLOTS: u32 = 4096u;
 const H_BELOW_FLOOR_SLOTS: u32 = 235024u;   // 盘、槽 lo/hi
 const H_END: u32 = 247312u;
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -1,6 +1,6 @@
 // GPU 判器的池级 checker 扫描内核（接在 crash_judge_common.wgsl 与 crash_judge_checker_common.wgsl 之后）：走读内核判完、交接段有效时，
-// 判第三批——扫描方向与记账那九条（I-1.8、I-3.9、I-9.14、I-5.4、I-3.10、I-7.9、I-3.1、I-5.2、I-3.11），再加 I-3.12（每条根各自那张实例表），
-// 违例位并进违例位那几个字，走到第几步那一字写成 5、判不了时下一字写原因。只读不判的读法照 walk.rs 的 *_without_judging 各写一份。
+// 判第三批——扫描方向与记账那九条（I-1.8、I-3.9、I-9.14、I-5.4、I-3.10、I-7.9、I-3.1、I-5.2、I-3.11），再加 I-3.12（每条根各自那张实例表）、
+// I-5.5 与 I-9.28（根表里每一条根的树表），违例位并进违例位那几个字，走到第几步那一字写成 5、判不了时下一字写原因。只读不判的读法照 walk.rs 的 *_without_judging 各写一份。
 
 
 var<private> r_complete: bool;
@@ -475,6 +475,33 @@
     r_complete = false;
 }
 
+// 树表一条条目（偏移 o 起，宽 ≥ 200）的树 ID（偏移 0）与诞生 txg（偏移 108）记进信息行 row：同一棵树在这一片里再出现时后一条盖前一条
+// （walk.rs 的 tree_table_birth_txg 是按树 ID 插入的 BTreeMap）
+fn r_note_tree_table_entry_birth_txg(row: u32, tree_table_base: u32, o: u32) {
+    let tree = rd64(tree_table_base, o);
+    let birth = rd64(tree_table_base, o + 108u);
+    let tree_count = sw(row + 5u);
+    var found = false;
+    for (var t = 0u; t < tree_count; t = t + 1u) {
+        if (sw(row + 6u + t * 4u) == tree.x && sw(row + 7u + t * 4u) == tree.y) {
+            sws(row + 8u + t * 4u, birth.x);
+            sws(row + 9u + t * 4u, birth.y);
+            found = true;
+        }
+    }
+    if (!found) {
+        if (tree_count >= C_MAX_TREES_PER_TABLE) {
+            undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (28u << 8u);
+        } else {
+            sws(row + 6u + tree_count * 4u, tree.x);
+            sws(row + 7u + tree_count * 4u, tree.y);
+            sws(row + 8u + tree_count * 4u, birth.x);
+            sws(row + 9u + tree_count * 4u, birth.y);
+            sws(row + 5u, tree_count + 1u);
+        }
+    }
+}
+
 // references_of_root：一条根引用了什么（只读不判），走全没走全、树表落点与树表条目的诞生 txg 记进候选根的信息行
 fn r_references_of_root(record_base: u32, every: bool, info: u32) {
     let row = C_CANDIDATE_INFO + info * C_CANDIDATE_WORDS;
@@ -518,28 +545,7 @@
             continue;
         }
         let o = entry_offset(tt, e);
-        let tree = rd64(tt.base, o);
-        let birth = rd64(tt.base, o + 108u);
-        var tree_count = sw(row + 5u);
-        var found = false;
-        for (var t = 0u; t < tree_count; t = t + 1u) {
-            if (sw(row + 6u + t * 4u) == tree.x && sw(row + 7u + t * 4u) == tree.y) {
-                sws(row + 8u + t * 4u, birth.x);
-                sws(row + 9u + t * 4u, birth.y);
-                found = true;
-            }
-        }
-        if (!found) {
-            if (tree_count >= C_MAX_TREES_PER_TABLE) {
-                undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (28u << 8u);
-            } else {
-                sws(row + 6u + tree_count * 4u, tree.x);
-                sws(row + 7u + tree_count * 4u, tree.y);
-                sws(row + 8u + tree_count * 4u, birth.x);
-                sws(row + 9u + tree_count * 4u, birth.y);
-                sws(row + 5u, tree_count + 1u);
-            }
-        }
+        r_note_tree_table_entry_birth_txg(row, tt.base, o);
         let tree_root = parse_node_pointer(tt.base, o + 14u);
         if (tree_root.all_zero) {
             continue;
@@ -679,8 +685,15 @@
             }
         }
     }
-    // I-9.14
-    for (var a = 0u; a < c_candidate_count; a = a + 1u) {
+    // I-9.14：候选集里的根
+    judge_tree_table_birth_txg_across_the_rows(c_candidate_count, INVARIANT_I_9_14);
+}
+
+// 按树 ID 归并 C_CANDIDATE_INFO 前 row_count 行记下的树表条目：一棵树的条目出现在两个不同的树表单元里、诞生 txg 又不一，置 invariant 的违例位。
+// 没比到两个不同树表单元的树不判；树表读不出、条目比字段表短的那几行没有条目，只是少一份证据（不适用、部分没判与成立都不置位）。
+// I-9.14 交候选根那几行，I-9.28 交根表里每一条根那几行
+fn judge_tree_table_birth_txg_across_the_rows(row_count: u32, invariant: u32) {
+    for (var a = 0u; a < row_count; a = a + 1u) {
         let row_a = C_CANDIDATE_INFO + a * C_CANDIDATE_WORDS;
         if (sw(row_a + 1u) == 0u) {
             continue;
@@ -708,7 +721,7 @@
             // 这棵树的条目出现在几个不同的树表单元里、诞生 txg 是否都相同
             var distinct_tables = 0u;
             var same_birth = true;
-            for (var c = a; c < c_candidate_count; c = c + 1u) {
+            for (var c = a; c < row_count; c = c + 1u) {
                 let row_c = C_CANDIDATE_INFO + c * C_CANDIDATE_WORDS;
                 if (sw(row_c + 1u) == 0u) {
                     continue;
@@ -746,12 +759,48 @@
                 }
             }
             if (distinct_tables >= 2u && !same_birth) {
-                violate(INVARIANT_I_9_14);
+                violate(invariant);
             }
         }
     }
 }
 
+// references_of_root 按「只要树表」那个深度（walk.rs 的 ReferenceScanDepth::TreeTableOnly）：一条根的树表落点与条目的诞生 txg 记进信息行 row。
+// 树表指针全零 ⇒ 没有落点；树表读不出、条目比字段表短 ⇒ 有落点、没有条目
+fn r_tree_table_entries_of_root(record_base: u32, row: u32) {
+    sws(row + 1u, 0u);
+    sws(row + 5u, 0u);
+    let tree_table_pointer = parse_node_pointer(record_base, 36u);
+    if (tree_table_pointer.all_zero) {
+        return;
+    }
+    sws(row + 1u, 1u);
+    sws(row + 2u, tree_table_pointer.device0);
+    sws(row + 3u, tree_table_pointer.slot0.x);
+    sws(row + 4u, tree_table_pointer.slot0.y);
+    let tt = r_read_index_node(tree_table_pointer);
+    if (!tt.ok || tt.entry_width < 200u) {
+        return;
+    }
+    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
+        r_note_tree_table_entry_birth_txg(row, tt.base, entry_offset(tt, e));
+    }
+}
+
+// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同，walk.rs 的 judge_tree_identifier_birth_txg_across_every_readable_root）：根环里每一条读得出、
+// 自证过的根（根表：不按实例表判有效与否、不按 F 与窗口滤，含被崩溃恢复抛弃的旧线上的根）各读一遍树表，按树 ID 归并，诞生 txg 不一就违例。
+// 不适用（没比到两个不同的树表单元）与部分没判（有根的树表指针非零而读不出、或条目比字段表短）都不置位，与成立一样
+fn judge_tree_identifier_birth_txg_across_every_readable_root() {
+    if (root_count > C_MAX_CANDIDATES) {
+        undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (47u << 8u);
+        return;
+    }
+    for (var index = 0u; index < root_count; index = index + 1u) {
+        r_tree_table_entries_of_root(checker_root_of(index).base, C_CANDIDATE_INFO + index * C_CANDIDATE_WORDS);
+    }
+    judge_tree_table_birth_txg_across_the_rows(root_count, INVARIANT_I_9_28);
+}
+
 // 分配记录树根节点（按第一条位置条目）去重：见过交回 false
 fn c_seen_node_insert(p: Ptr) -> bool {
     for (var i = 0u; i < c_seen_node_count; i = i + 1u) {
@@ -1146,9 +1195,17 @@
     return fourth;
 }
 
-// I-7.9：只判抬 F 的那一条根，拿它之前的根算上限
+// 一圈窗口（walk.rs 的 OneRevolutionWindow::contains）：txg > 锚 − 3S，写成 txg + 3S > 锚（锚 < 3S 时整个环都在窗口里，不减出负数）
+fn in_the_one_revolution_window(txg: vec2<u32>, anchor_txg: vec2<u32>, root_ring_slot_count: u32) -> bool {
+    return lt64(anchor_txg, add64_saturating(txg, u64_of(root_ring_slot_count)));
+}
+
+// I-7.9：只判抬 F 的那一条根，拿它之前的根算上限。两支的「之前的根」都只取 txg 比它小、同时落在两个一圈窗口里的根
+// （用户 2026-10-04 定「乙，两支都取两个窗口」）：锚在它自己的 txg 上的那一个，与锚在这张镜像根环里最新那条根的 txg 上的那一个（候选过滤那一个）
 fn judge_rollback_floor_raises_against_their_ceilings() {
     let device_count = hdr(3u);
+    let root_ring_slot_count = sw(H_BASE + H_REGIONS) * sw(H_BASE + H_SLOTS_PER_REGION);
+    let newest_txg_of_the_image = checker_root_of(c_newest_index).txg;
     for (var i = 0u; i < root_count; i = i + 1u) {
         let raising = checker_root_of(i);
         // 同一实例里 txg 比它小的最新那条根
@@ -1167,14 +1224,20 @@
             continue;
         }
         if (raising.unmount) {
+            // 带卸载记号那一支：F 不高于两个窗口里 txg 比它小的最新那条根的 txg；窗口里一条都没有 ⇒ 这条根不判
+            var have_newest_before = false;
             var newest_before = vec2<u32>(0u, 0u);
             for (var k = 0u; k < root_count; k = k + 1u) {
                 let txg = checker_root_of(k).txg;
-                if (lt64(txg, raising.txg) && lt64(newest_before, txg)) {
+                if (lt64(txg, raising.txg)
+                    && in_the_one_revolution_window(txg, raising.txg, root_ring_slot_count)
+                    && in_the_one_revolution_window(txg, newest_txg_of_the_image, root_ring_slot_count)
+                    && (!have_newest_before || lt64(newest_before, txg))) {
                     newest_before = txg;
+                    have_newest_before = true;
                 }
             }
-            if (lt64(newest_before, raising.floor)) {
+            if (have_newest_before && lt64(newest_before, raising.floor)) {
                 violate(INVARIANT_I_7_9);
             }
             continue;
@@ -1182,11 +1245,14 @@
         if (!r_instance_rows(raising.base)) {
             continue;
         }
-        // 它之前的有效根：txg < 它的、≥ 抬之前的 F、按它自己的实例表不被抛弃；按 (txg, 实例) 升序
+        // 它之前的有效根：txg < 它的、在两个一圈窗口里、≥ 抬之前的 F、按它自己的实例表不被抛弃；按 (txg, 实例) 升序
         var valid_count = 0u;
         for (var k = 0u; k < root_count; k = k + 1u) {
             let root = checker_root_of(k);
-            if (!lt64(root.txg, raising.txg) || lt64(root.txg, floor_before) || r_rows_abandon(root.instance, root.txg)) {
+            if (!lt64(root.txg, raising.txg)
+                || !in_the_one_revolution_window(root.txg, raising.txg, root_ring_slot_count)
+                || !in_the_one_revolution_window(root.txg, newest_txg_of_the_image, root_ring_slot_count)
+                || lt64(root.txg, floor_before) || r_rows_abandon(root.instance, root.txg)) {
                 continue;
             }
             var position = 0u;
@@ -1942,6 +2008,7 @@
 fn scan_stage(fsid_low: vec2<u32>) {
     judge_merged_version_total_order(fsid_low);
     judge_release_generation_and_tree_table_birth();
+    judge_tree_identifier_birth_txg_across_every_readable_root();
     judge_allocation_records_disjoint();
     judge_allocation_generations_against_unit_births();
     judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark();
diff --git a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -3,7 +3,7 @@
 //! 七类 oracle、记录核对器两条判据，判法与 CPU 判器（`crate::crash::judge_crash_image`）逐条相同；GPU 与 CPU 逐状态比是它的判别力自证。
 //! 内核分五份源码：`crash_judge_common.wgsl`（各内核共用的解析与查表）、`crash_judge_recovery.wgsl`（恢复、oracle、记录核对器）、
 //! `crash_judge_checker_common.wgsl`（池级 checker 两个内核共用的读法与交接段）、`crash_judge_checker.wgsl`（走读内核：第一、二批不变量，
-//! 连同里程碑三二·1 那一批里的十五条、I-9.27 与 I-3.5）、`crash_judge_checker_scan.wgsl`（扫描内核：第三批、I-3.12 与 I-5.5）；判不了的状态（表越过内核容量）置最高位、交 CPU 判。
+//! 连同里程碑三二·1 那一批里的十五条、I-9.27 与 I-3.5）、`crash_judge_checker_scan.wgsl`（扫描内核：第三批、I-3.12、I-5.5 与 I-9.28）；判不了的状态（表越过内核容量）置最高位、交 CPU 判。
 //!
 //! 一个状态的结论 [`VERDICT_WORDS`] 个 u32：第 0 字是红位（位 0..6 看 journal 那一遍 oracle 的七类、位 8..14 不看那一遍、
 //! 位 16 根在而记录都不在、位 17 恢复自称的状态少单元、位 31 判不了）；第 1 字判不了的原因（判得了时低 16 位是看 journal 那一遍
@@ -58,15 +58,17 @@
     "违例位读成一个 u128"
 );
 
-/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 68 条——三批 49 条，加里程碑三二·1 那一批 16 条
+/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 69 条——三批 49 条，加里程碑三二·1 那一批 16 条
 /// （判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行），下标 65 的 I-3.5（引用区间的精确性，
 /// 走读内核在回退候选集的版本都走完之后逐对判），再加 C673（三 oracle 合谋测试的补 oracle 改法没做）⑤ ⑥ 那两条
-/// （下标 66 是 I-9.27，走读内核挨着 I-9.16 按表判；下标 67 是 I-5.5，扫描内核每一版重读树表与 extent 树判），对拍逐位比、不屏蔽。
+/// （下标 66 是 I-9.27，走读内核挨着 I-9.16 按表判；下标 67 是 I-5.5，扫描内核每一版重读树表与 extent 树判），
+/// 再加下标 68 的 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同，扫描内核挨着 I-9.14 读根表里每一条根的树表判；
+/// 部分没判与不适用一样不置违例位，结论里只有违例位），对拍逐位比、不屏蔽。
 /// 用户 2026-10-04 定「改回要求全覆盖」：没有「内核还没判」那一张表。
 pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
     26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
-    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67,
+    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68,
 ];
 
 // 覆盖表就是池级 checker 的每一条，按下标升序：第 k 项是 k。新立一条不变量而内核没跟上时这里编译不过，不悄悄让 GPU 那一位恒 0。
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -5,7 +5,8 @@
 //! 每一条不变量的违例位（`IMPLEMENTED_INVARIANTS` 全部，不屏蔽）要与 CPU 池级 checker 逐位相同，一个状态都不许判不了。
 //! 改坏那条流上 CPU 有红，GPU 要红在同样的状态、同样的项上——这是 GPU 判器分得出差别的自证；里程碑三二·1 那一批的每一条不变量
 //! 另有一条只改坏它管的那个字段的流（`the_gpu_judge_agrees_with_the_cpu_when_…` 那一组）；I-3.5 的出现规则红与成立各一条，
-//! 恢复施加一条记录取新根段的那几段、施加出来的那一版从记录走实例表与分配记录树、I-8.10 后三段，各有一条同样形态的流。
+//! 恢复施加一条记录取新根段的那几段、施加出来的那一版从记录走实例表与分配记录树、I-8.10 后三段，各有一条同样形态的流；
+//! I-9.28 比到候选集之外的根与树表读不出的根、I-7.8 扫描方向的认领规则、I-7.9 两支只取两个一圈窗口里的根，也各有流。
 //! 要本机一张空闲显存够的 NVIDIA 独显：本机一张 NVIDIA 独显适配器都没有时用卡的用例打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、
 //! 返回成功；有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）。要 `verdict-store,gpu` 特性；
 //! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（默认 8）、显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（没设不定额度）、
@@ -18,7 +19,7 @@
 
 use std::time::Instant;
 
-use common::{build_pool, file_content, geometry};
+use common::{build_pool, file_content, geometry, parameters};
 use common_crash_points::{mount_writable_step, overwrite_step, record_pool_build, roll_back_step};
 use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
 use singlefs_checker_tier::crash::{
@@ -187,7 +188,17 @@
 /// 改坏发布的根指着的树表单元（两份副本各一字节）：根落了的状态走读失败，CPU 与 GPU 都要红。
 fn record_new_pool_with_a_broken_tree_table(tag: &str) -> RecordedFlow {
     let mut flow = record_new_pool_file_creation(tag, "new-pool-broken-tree-table");
-    let root_bytes = flow.writes[flow.judged_root_index]
+    let publish_root = flow.judged_root_index;
+    break_both_copies_of_the_tree_table_named_by_the_root_write(&mut flow, publish_root);
+    flow
+}
+
+/// 第 `root_write` 次写（根槽写）指着的树表单元，两份副本各改坏末字节、不重封：那条根落了的状态上它的树表读不出。
+fn break_both_copies_of_the_tree_table_named_by_the_root_write(
+    flow: &mut RecordedFlow,
+    root_write: usize,
+) {
+    let root_bytes = flow.writes[root_write]
         .bytes()
         .expect("根槽写带字节")
         .to_vec();
@@ -202,28 +213,30 @@
             (device, u64::from_le_bytes(slot))
         })
         .collect();
-    let mut broken = 0;
-    for write in &mut flow.writes {
-        if write.kind != StepKind::UnitWrite {
-            continue;
-        }
-        let slot = write.offset.0 / 16384;
-        if locations
-            .iter()
-            .any(|(device, location_slot)| *device == write.device.0 && *location_slot == slot)
-        {
-            match &mut write.contents {
-                WrittenContents::Bytes(bytes) => {
-                    let last = bytes.len() - 1;
-                    bytes[last] ^= 0x01;
-                    broken += 1;
-                }
-                WrittenContents::Zeros { .. } => panic!("单元写是普通写"),
+    // 每条位置条目取根槽写之前落在那个槽上的最后一次单元写：那一份就是这条根指着的副本（同一个槽更早被别的单元用过时不碰那一次）
+    let copies: Vec<usize> = locations
+        .iter()
+        .map(|(device, location_slot)| {
+            flow.writes[..root_write]
+                .iter()
+                .rposition(|write| {
+                    write.kind == StepKind::UnitWrite
+                        && write.device.0 == *device
+                        && write.offset.0 / 16384 == *location_slot
+                })
+                .expect("根槽写之前写过它指着的树表单元")
+        })
+        .collect();
+    assert_ne!(copies[0], copies[1], "树表单元的两份副本是两次写");
+    for copy in copies {
+        match &mut flow.writes[copy].contents {
+            WrittenContents::Bytes(bytes) => {
+                let last = bytes.len() - 1;
+                bytes[last] ^= 0x01;
             }
+            WrittenContents::Zeros { .. } => panic!("单元写是普通写"),
         }
     }
-    assert_eq!(broken, 2, "树表单元的两份副本都改坏");
-    flow
 }
 
 /// 改一次写里的几个字节之后把那个结构的整槽校验和字段重封（CRC-32C 住字段前 4 字节、后 28 字节 0）：拦住它的只剩被改的字段本身。
@@ -1828,6 +1841,8 @@
     cpu_checker_red_invariants: std::collections::BTreeSet<&'static str>,
     /// CPU 池级 checker 在这条流的哪几条不变量上判过而成立过（至少一个状态上不是报不适用）。
     cpu_checker_held_invariants: std::collections::BTreeSet<&'static str>,
+    /// CPU 池级 checker 在每个状态上的违例位（按状态序号）：要钉「同一个状态上红这一条、不红那一条」的用例读它。
+    cpu_checker_bits_of_each_state: Vec<u128>,
     /// 看 journal 那一遍落到的根或两遍的结局类别与 CPU 不同的状态数（算进不一致）。
     recovery_mismatches: u64,
     mismatches: Vec<String>,
@@ -1949,6 +1964,7 @@
         gpu_checker_red: 0,
         cpu_checker_red_invariants: std::collections::BTreeSet::new(),
         cpu_checker_held_invariants: std::collections::BTreeSet::new(),
+        cpu_checker_bits_of_each_state: Vec::with_capacity(cpu.len()),
         recovery_mismatches: 0,
         mismatches: Vec::new(),
     };
@@ -1960,6 +1976,9 @@
         comparison
             .cpu_checker_held_invariants
             .extend(invariant_names(*cpu_checker_held_bits));
+        comparison
+            .cpu_checker_bits_of_each_state
+            .push(*cpu_checker_bits);
         let gpu_words: [u32; VERDICT_WORDS] = gpu[ordinal];
         let gpu_bits = gpu_words[0] & !(1 << verdict_bits::UNDECIDABLE);
         let undecidable = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
@@ -2319,7 +2338,7 @@
 /// 在扫描内核读回违例位之后种第 128 位（下标 127）：
 /// 两份种过的内核判出的每个状态，违例位等于没种的那一份判出的再并上种的位（第 65 位每个状态都有，第 128 位只在扫描内核判完的状态上有），
 /// 其余的字逐字相同。违例位的高 64 位在走读内核写出、扫描内核读回又写出、Rust 读出这三处任一处被截掉，这条红。
-/// 池级 checker 今天 68 条（下标 0..=67，内核每一条都判），种的是第 69 位（下标 68）。
+/// 池级 checker 今天 69 条（下标 0..=68，内核每一条都判），种的是第 70 位（下标 69）。
 #[test]
 fn a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels() {
     const FIRST_BIT_OF_THE_HIGH_HALF: usize = 64;
@@ -3249,3 +3268,509 @@
             .join("\n")
     );
 }
+
+// ───────────── 第 9 批：I-9.28 比到候选集之外的根与树表读不出的根、I-7.8 扫描方向的认领规则、I-7.9 两支只取两个一圈窗口里的根 ─────────────
+
+/// 根记录里 flags（卸载记号在位 0）、checkpoint_txg、回退下界 F 的偏移（D22（单元原子性怎么合成） 已定项 7 的偏移表）。
+const ROOT_RECORD_FLAGS_OFFSET: usize = 20;
+const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;
+const ROOT_RECORD_ROLLBACK_FLOOR_OFFSET: usize = 130;
+
+/// 池级 checker 违例位里这一条的那一位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）。
+fn invariant_bit(invariant: &str) -> u128 {
+    let index = IMPLEMENTED_INVARIANTS
+        .iter()
+        .position(|candidate| *candidate == invariant)
+        .expect("这一条在池级 checker 的清单里");
+    1u128 << index
+}
+
+/// 流尾另起一段写的那条根带不带卸载记号（根记录 flags 位 0）。
+#[derive(Clone, Copy, Debug)]
+enum UnmountMarkerOfTheAppendedRoot {
+    Written,
+    NotWritten,
+}
+
+/// 流尾另起一段写一条根：抄第 `source_root_write` 次写（根槽写）的字节，checkpoint_txg 改成 `checkpoint_txg`、回退下界 F 改成
+/// `rollback_floor`、flags 位 0 照 `unmount_marker`，实例代号与各条指针照抄，重封整槽校验和，写到这个 txg 按发布规则算出的那个根槽上
+/// （`singlefs_core::root_ring::target_for_publish`，I-7.17 不因为它红）。那个槽里原来的根被它盖掉。
+fn append_a_root_at_a_later_txg(
+    flow: &mut RecordedFlow,
+    source_root_write: usize,
+    checkpoint_txg: u64,
+    rollback_floor: u64,
+    unmount_marker: UnmountMarkerOfTheAppendedRoot,
+) {
+    let source = flow.writes[source_root_write].clone();
+    assert_eq!(source.kind, StepKind::RootRecordFua, "抄的是根槽写");
+    let WrittenContents::Bytes(mut bytes) = source.contents else {
+        panic!("根槽写是普通写");
+    };
+    bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
+        .copy_from_slice(&checkpoint_txg.to_le_bytes());
+    bytes[ROOT_RECORD_ROLLBACK_FLOOR_OFFSET..ROOT_RECORD_ROLLBACK_FLOOR_OFFSET + 8]
+        .copy_from_slice(&rollback_floor.to_le_bytes());
+    let flags: u32 = match unmount_marker {
+        UnmountMarkerOfTheAppendedRoot::Written => 1,
+        UnmountMarkerOfTheAppendedRoot::NotWritten => 0,
+    };
+    bytes[ROOT_RECORD_FLAGS_OFFSET..ROOT_RECORD_FLAGS_OFFSET + 4]
+        .copy_from_slice(&flags.to_le_bytes());
+    reseal_root_record(&mut bytes);
+    let target = singlefs_core::root_ring::target_for_publish(
+        CheckpointTxg(checkpoint_txg),
+        geometry().root_ring_slots_per_region,
+    );
+    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
+    let offset =
+        singlefs_core::root_ring::slot_offset(target, geometry().fixed_structure_slot_spacing);
+    append_a_segment_of_writes(
+        flow,
+        vec![RetainedWrite {
+            device,
+            kind: StepKind::RootRecordFua,
+            is_force_unit_access: source.is_force_unit_access,
+            offset,
+            contents: WrittenContents::Bytes(bytes),
+        }],
+    );
+}
+
+/// 根环槽数 3S（三个区域、每区 S 槽）。
+fn root_ring_slot_count() -> u64 {
+    singlefs_format::ROOT_RING_REGIONS * geometry().root_ring_slots_per_region.count()
+}
+
+/// 树表单元：码 2、key 宽 8、条目宽 200。
+fn is_a_tree_table_unit(bytes: &[u8]) -> bool {
+    unit_class(bytes) == 2
+        && index_node_key_width(bytes) == 8
+        && index_node_entry_width(bytes) == 200
+}
+
+/// 树表单元里 inode 树（种类 2）那几条条目的诞生 txg（条目偏移 108）加 1。
+fn add_one_to_the_birth_txg_of_the_inode_tree_entries(bytes: &mut [u8]) {
+    for entry in 0..(bytes.len() - 131) / 200 {
+        let offset = 131 + entry * 200;
+        if u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().expect("2")) == 2 {
+            bytes[offset + 108] = bytes[offset + 108].wrapping_add(1);
+        }
+    }
+}
+
+/// 回退流上回退那次发布 D（txg 9）的树表里 inode 树条目的诞生 txg 加 1、重封到根（同 `rollback-tree-table-birth-txg-plus-one`），
+/// 流尾再在 txg 30 写一条 D 的根、回退下界 F 抬到 9（落在 txg 6 那个槽上）。这一条落了的状态上候选集只剩 D 与它（别的根都在 F 之下），
+/// 两条共用 D 那片树表，I-9.14 比不出来；I-9.28 照比根环里每一条读得出的根，A、B、C 那几片记的诞生 txg 与 D 的不同，红。
+fn record_rollback_with_another_birth_txg_recorded_only_outside_the_candidate_set(
+    tag: &str,
+) -> RecordedFlow {
+    const TXG_OF_THE_ROOT_RAISING_THE_FLOOR_TO_THE_ROLLBACK: u64 = 30;
+    const FLOOR_AT_THE_ROLLBACK: u64 = 9;
+    let mut flow = record_rollback_with_a_resealed_field(
+        tag,
+        "rollback-birth-txg-differs-only-outside-the-candidate-set",
+        &is_a_tree_table_unit,
+        &add_one_to_the_birth_txg_of_the_inode_tree_entries,
+    );
+    assert_eq!(
+        publish_root_identity(&flow),
+        (2, FLOOR_AT_THE_ROLLBACK),
+        "被判的根是回退那次发布 D（实例 2、txg 9）"
+    );
+    let rollback_root = flow.judged_root_index;
+    append_a_root_at_a_later_txg(
+        &mut flow,
+        rollback_root,
+        TXG_OF_THE_ROOT_RAISING_THE_FLOOR_TO_THE_ROLLBACK,
+        FLOOR_AT_THE_ROLLBACK,
+        UnmountMarkerOfTheAppendedRoot::NotWritten,
+    );
+    flow.versions.push(PublishedVersion {
+        instance: InstanceGeneration(2),
+        checkpoint_txg: CheckpointTxg(TXG_OF_THE_ROOT_RAISING_THE_FLOOR_TO_THE_ROLLBACK),
+        content: file_content(),
+    });
+    flow
+}
+
+/// I-9.28 比的是根环里每一条读得出、自证过的根，不只是回退候选集（I-9.14 只比候选集）：有一个状态上 CPU 红 I-9.28 而 I-9.14 不红，
+/// GPU 每个状态逐位相同。GPU 那一条只比候选根时，这个状态上它不红、与 CPU 不一致。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_only_roots_outside_the_candidate_set_record_another_birth_txg(
+) {
+    let flow = record_rollback_with_another_birth_txg_recorded_only_outside_the_candidate_set(
+        "crash-judge-gpu-i928-outside-candidates",
+    );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_when_only_roots_outside_the_candidate_set_record_another_birth_txg",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let every_readable_root = invariant_bit("I-9.28");
+    let candidate_set = invariant_bit("I-9.14");
+    assert!(
+        comparison
+            .cpu_checker_bits_of_each_state
+            .iter()
+            .any(|bits| bits & every_readable_root != 0 && bits & candidate_set == 0),
+        "流 {} 上要有一个状态 CPU 红 I-9.28 而 I-9.14 不红（红过的：{:?}）",
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
+/// 回退流上覆盖写 C（txg 8）那条根指着的树表两份副本各改坏末字节：C 那条根落了的状态上它读得出、自证过，树表指针非零而树表读不出，
+/// I-9.28 少它那一份证据（CPU 报「部分没判」，GPU 与不适用一样不置违例位）；读得出的那几片记的诞生 txg 都相同，不红。
+fn record_rollback_with_the_tree_table_of_the_overwrite_before_it_unreadable(
+    tag: &str,
+) -> RecordedFlow {
+    const TXG_OF_THE_OVERWRITE_BEFORE_THE_ROLLBACK: u64 = 8;
+    let mut flow = record_fixed_script_through_rollback(tag);
+    flow.name = "rollback-tree-table-of-c-unreadable";
+    let root_of_the_overwrite = flow
+        .writes
+        .iter()
+        .rposition(|write| {
+            write.kind == StepKind::RootRecordFua
+                && write.bytes().is_some_and(|bytes| {
+                    u64::from_le_bytes(
+                        bytes[ROOT_RECORD_CHECKPOINT_TXG_OFFSET
+                            ..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
+                            .try_into()
+                            .expect("8"),
+                    ) == TXG_OF_THE_OVERWRITE_BEFORE_THE_ROLLBACK
+                })
+        })
+        .expect("回退流上有覆盖写 C（txg 8）的根槽写");
+    break_both_copies_of_the_tree_table_named_by_the_root_write(&mut flow, root_of_the_overwrite);
+    flow
+}
+
+/// I-9.28 的四种判定在 GPU 上只有违例置位：有一条读得出、自证过的根树表读不出，而读得出的那几片都对得上时，CPU 一个状态都不红它，
+/// GPU 每个状态逐位相同。GPU 把读不出树表的根当成违例时这条红。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_root_whose_tree_table_cannot_be_read_does_not_redden_the_every_readable_root_birth_invariant(
+) {
+    let flow = record_rollback_with_the_tree_table_of_the_overwrite_before_it_unreadable(
+        "crash-judge-gpu-i928-unreadable-tree-table",
+    );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_root_whose_tree_table_cannot_be_read_does_not_redden_the_every_readable_root_birth_invariant",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        !comparison.cpu_checker_red_invariants.contains("I-9.28"),
+        "流 {} 上 CPU 一个状态都不红 I-9.28（红过的：{:?}）",
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
+/// 覆盖写那一版的文件长度：一个数据单元装得下（32768 − 134），内容跨过单元的第二槽起点（单元偏移 16384）。
+const FORGED_HEADER_FILE_BYTES: usize = 20_000;
+/// 伪头里的树 ID：2⁴⁰，远高于这条流的水位。
+const FORGED_HEADER_TREE_IDENTIFIER: u64 = 1 << 40;
+/// 伪头的 key 宽（码 2 头偏移 51）。
+const FORGED_HEADER_KEY_WIDTH: u8 = 8;
+/// 伪头里的诞生代号：1，不超过根环里最大的 checkpoint_txg（扫描方向第一道排除挡不住它）。
+const FORGED_HEADER_BIRTH_TXG: u64 = 1;
+/// 伪头带的「别的池」的 fsid 低 8 字节：写序实例按 fsid 认不出，扫描方向第二道排除也挡不住它。
+const FORGED_HEADER_FOREIGN_FILESYSTEM_IDENTIFIER_LOW: u64 = 0x0123_4567_89ab_cdef;
+
+/// 覆盖写的内容：按位置取的字节，内容偏移 16384 − 134 起放一个头校验和自洽的码 2 伪头（D18（块里携带什么信息） 已定项 18 的偏移：
+/// 树 ID 42、key 宽 51、诞生代号 52 + 2k、fsid 60 + 2k、明文头末尾 86 + 2k，头校验和在 10）。
+fn content_forging_an_index_node_header_on_the_second_slot_of_its_data_unit() -> Vec<u8> {
+    let mut content: Vec<u8> = (0..FORGED_HEADER_FILE_BYTES)
+        .map(|index| u8::try_from((index * 7 + 5) % 251).expect("小于 251"))
+        .collect();
+    let key_span = 2 * usize::from(FORGED_HEADER_KEY_WIDTH);
+    let header_end = 86 + key_span;
+    let mut header = vec![0u8; header_end];
+    header[..4].copy_from_slice(&singlefs_core::unit::UNIT_MAGIC);
+    header[6] = singlefs_core::unit::UNIT_CLASS_INDEX_NODE;
+    header[42..50].copy_from_slice(&FORGED_HEADER_TREE_IDENTIFIER.to_le_bytes());
+    header[51] = FORGED_HEADER_KEY_WIDTH;
+    header[52 + key_span..60 + key_span].copy_from_slice(&FORGED_HEADER_BIRTH_TXG.to_le_bytes());
+    header[60 + key_span..68 + key_span]
+        .copy_from_slice(&FORGED_HEADER_FOREIGN_FILESYSTEM_IDENTIFIER_LOW.to_le_bytes());
+    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(&header, header_end, 10);
+    header[10..42].copy_from_slice(&digest);
+    let forged_header_content_offset =
+        usize::try_from(singlefs_format::SLOT_BYTES - singlefs_format::DATA_UNIT_PAYLOAD_OFFSET)
+            .expect("小于一个槽");
+    content[forged_header_content_offset..forged_header_content_offset + header_end]
+        .copy_from_slice(&header);
+    content
+}
+
+/// 新池新建文件之后覆盖写一次，内容在它那个数据单元第二槽的起点上带一个码 2 伪头（[`content_forging_an_index_node_header_on_the_second_slot_of_its_data_unit`]）。
+fn record_new_pool_with_an_overwrite_forging_an_index_node_header_on_the_second_slot_of_its_data_unit(
+    tag: &str,
+) -> RecordedFlow {
+    let mut pool = build_pool(tag);
+    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
+    record_pool_build(&mut recorder, &pool);
+    let content = content_forging_an_index_node_header_on_the_second_slot_of_its_data_unit();
+    pool.output = overwrite_step(&mut recorder, &mut pool, &content, InstanceGeneration(1));
+    let base = pool.memory_pool_after_mkfs();
+    let operations = pool.retained_operations();
+    let (writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
+        &operations[pool.mkfs_operation_count..],
+        &geometry(),
+    );
+    let second_slot = usize::try_from(singlefs_format::SLOT_BYTES).expect("16384");
+    assert!(
+        writes.iter().any(|write| {
+            write.kind == StepKind::UnitWrite
+                && write.bytes().is_some_and(|bytes| {
+                    unit_class(bytes) == 1
+                        && bytes.len() > second_slot + 52
+                        && bytes[second_slot..second_slot + 4] == singlefs_core::unit::UNIT_MAGIC
+                        && bytes[second_slot + 6] == singlefs_core::unit::UNIT_CLASS_INDEX_NODE
+                })
+        }),
+        "覆盖写那一版的文件装在一个数据单元里，伪头落在它第二槽的起点上"
+    );
+    let judged_root_index = writes
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .expect("根槽写");
+    RecordedFlow {
+        name: "new-pool-overwrite-forging-an-index-node-header-on-the-second-slot",
+        base,
+        writes,
+        segments,
+        judged_root_index,
+        versions: vec![
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(3),
+                content: file_content(),
+            },
+            PublishedVersion {
+                instance: InstanceGeneration(1),
+                checkpoint_txg: CheckpointTxg(4),
+                content,
+            },
+        ],
+    }
+}
+
+/// I-7.8 扫描方向的认领规则（D18（块里携带什么信息） 已定项 7，walk.rs 的 `scanned_tree_identifiers`）：覆盖写的数据单元第二槽起点上
+/// 带一个头校验和自洽、树 ID 2⁴⁰ 的码 2 伪头。GPU 的落点表里有这个第二槽（覆盖写那次写碰过的每个 16 KiB 槽各是一个落点），
+/// CPU 的崩溃镜像只把那次写的起点槽当候选；数据单元落了的状态上，GPU 照认领规则让起点槽上的数据单元认领两槽、不读第二槽的头，
+/// 与 CPU 一样不数它，CPU 一个状态都不红 I-7.8、GPU 每个状态逐位相同。GPU 不按整单元跳时把伪头数进「出现过的」树 ID、红 I-7.8。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_data_unit_claims_its_second_slot_and_a_header_forged_there_is_not_counted(
+) {
+    let flow = record_new_pool_with_an_overwrite_forging_an_index_node_header_on_the_second_slot_of_its_data_unit(
+        "crash-judge-gpu-i78-claim",
+    );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_data_unit_claims_its_second_slot_and_a_header_forged_there_is_not_counted",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        !comparison.cpu_checker_red_invariants.contains("I-7.8"),
+        "流 {} 上 CPU 一个状态都不红 I-7.8（红过的：{:?}）",
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
+/// 新池新建文件那条流上，流尾在 txg 27 = 3 + 3S 写一条抄发布那条根 (1, 3)、回退下界 F 从 0 抬到 5 的根（落在 txg 3 那个槽上、把它盖掉）。
+/// 之后根环里是 txg 0、1、2 的根与这一条，它是镜像最新的根：锚在它上的一圈窗口（txg > 3）里 txg 比它小的根一条都没有，
+/// 两支都不判它（准入那一支上限无从算起；带卸载记号那一支没有「之前的根」）。不取窗口时 txg 1、2 的根进来，F 5 高过它们，红。
+fn record_new_pool_with_a_floor_raise_one_revolution_past_every_earlier_root(
+    tag: &str,
+    name: &'static str,
+    unmount_marker: UnmountMarkerOfTheAppendedRoot,
+) -> RecordedFlow {
+    const FLOOR_RAISED_ONE_REVOLUTION_PAST_THE_PUBLISH: u64 = 5;
+    let mut flow = record_new_pool_file_creation(tag, name);
+    let (instance, publish_txg) = publish_root_identity(&flow);
+    assert_eq!(
+        (instance, publish_txg),
+        (1, 3),
+        "被判的根是新建文件那次发布"
+    );
+    let txg_one_revolution_past_the_publish = publish_txg + root_ring_slot_count();
+    let publish_root = flow.judged_root_index;
+    append_a_root_at_a_later_txg(
+        &mut flow,
+        publish_root,
+        txg_one_revolution_past_the_publish,
+        FLOOR_RAISED_ONE_REVOLUTION_PAST_THE_PUBLISH,
+        unmount_marker,
+    );
+    flow.versions.push(PublishedVersion {
+        instance: InstanceGeneration(1),
+        checkpoint_txg: CheckpointTxg(txg_one_revolution_past_the_publish),
+        content: file_content(),
+    });
+    flow
+}
+
+/// I-7.9 准入抬 F 那一支只取两个一圈窗口里的根（用户 2026-10-04 定「乙，两支都取两个窗口」）：窗口里一条之前的根都没有，这条根不判，
+/// CPU 一个状态都不判 I-7.9（不红、也不报成立），GPU 每个状态逐位相同。GPU 那一支不取窗口时红。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_judged(
+) {
+    let flow = record_new_pool_with_a_floor_raise_one_revolution_past_every_earlier_root(
+        "crash-judge-gpu-i79-admission-window",
+        "new-pool-floor-raise-one-revolution-past-every-earlier-root",
+        UnmountMarkerOfTheAppendedRoot::NotWritten,
+    );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_judged",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        !comparison.cpu_checker_red_invariants.contains("I-7.9")
+            && !comparison.cpu_checker_held_invariants.contains("I-7.9"),
+        "流 {} 上 CPU 一个状态都不判 I-7.9（红过的：{:?}；成立过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_red_invariants,
+        comparison.cpu_checker_held_invariants
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
+/// I-7.9 带卸载记号那一支同样只取两个一圈窗口里的根：同一张根环、那条根带卸载记号，窗口里 txg 比它小的根一条都没有，CPU 一个状态都不红 I-7.9，
+/// GPU 每个状态逐位相同。GPU 那一支不取窗口时拿 txg 2 那条根比、F 5 高过它，红。CPU 那一支的窗口由同一批的兄弟实现员补，
+/// 合入之前这条在 CPU 一侧照旧红（不取窗口）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_an_unmount_marked_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_reddened(
+) {
+    let flow = record_new_pool_with_a_floor_raise_one_revolution_past_every_earlier_root(
+        "crash-judge-gpu-i79-unmount-window",
+        "new-pool-unmount-marked-floor-raise-one-revolution-past-every-earlier-root",
+        UnmountMarkerOfTheAppendedRoot::Written,
+    );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_that_an_unmount_marked_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_reddened",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        !comparison.cpu_checker_red_invariants.contains("I-7.9"),
+        "流 {} 上 CPU 一个状态都不红 I-7.9（红过的：{:?}）",
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
+/// 回退流（根环里 txg 0..9，S = 8、3S = 24）上流尾先写一段：txg 29 一条抄回退那次发布 D 的根、F 从 0 抬到 6（落在 txg 5 那个槽上）；
+/// 再写一段：txg 27 一条同样的根、F 6（落在 txg 3 那个槽上），这时抬 F 的是 txg 27 那一条（txg 29 那一条的前一条变成它、F 不再高）。
+/// 两段都落了的状态上，锚在被判那条根（27）上的窗口是 txg > 3、锚在镜像最新那条根（29）上的是 txg > 5：B 那条根 (1, 4) 只在前一个里。
+/// 只取两个窗口的交时之前的有效根是 6、7、8、9，上限 6，F 6 成立；把 (1, 4) 算进来，最旧有效根变成 4、上限 4，红。
+/// 只有前一段落了的状态上抬 F 的是 txg 29 那一条、它就是最新的根，两个窗口相同（txg > 5），上限 6，成立。
+fn record_rollback_with_a_floor_raise_whose_window_anchored_at_the_newest_root_drops_an_older_root(
+    tag: &str,
+) -> RecordedFlow {
+    const TXG_OF_THE_NEWEST_ROOT: u64 = 29;
+    const TXG_OF_THE_ROOT_RAISING_THE_FLOOR: u64 = 27;
+    const FLOOR_RAISED_PAST_THE_ROLLBACK: u64 = 6;
+    let mut flow = record_fixed_script_through_rollback(tag);
+    flow.name = "rollback-floor-raise-whose-image-window-drops-an-older-root";
+    assert_eq!(
+        publish_root_identity(&flow),
+        (2, 9),
+        "被判的根是回退那次发布 D"
+    );
+    assert_eq!(
+        TXG_OF_THE_NEWEST_ROOT - root_ring_slot_count(),
+        5,
+        "锚在最新那条根上的窗口是 txg > 5"
+    );
+    let rollback_root = flow.judged_root_index;
+    for checkpoint_txg in [TXG_OF_THE_NEWEST_ROOT, TXG_OF_THE_ROOT_RAISING_THE_FLOOR] {
+        append_a_root_at_a_later_txg(
+            &mut flow,
+            rollback_root,
+            checkpoint_txg,
+            FLOOR_RAISED_PAST_THE_ROLLBACK,
+            UnmountMarkerOfTheAppendedRoot::NotWritten,
+        );
+        flow.versions.push(PublishedVersion {
+            instance: InstanceGeneration(2),
+            checkpoint_txg: CheckpointTxg(checkpoint_txg),
+            content: file_content(),
+        });
+    }
+    flow
+}
+
+/// I-7.9 准入那一支的「之前的根」还要落在锚在镜像最新那条根上的一圈窗口里，不只是锚在被判那条根上的那一个：CPU 判过 I-7.9 而成立、
+/// 一个状态都不红它，GPU 每个状态逐位相同。GPU 只取锚在被判那条根上的窗口时把 B 那条根算进来、红。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_takes_no_root_outside_the_window_anchored_at_the_newest_root_of_the_image(
+) {
+    let flow = record_rollback_with_a_floor_raise_whose_window_anchored_at_the_newest_root_drops_an_older_root(
+        "crash-judge-gpu-i79-image-window",
+    );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_takes_no_root_outside_the_window_anchored_at_the_newest_root_of_the_image",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        comparison.cpu_checker_held_invariants.contains("I-7.9")
+            && !comparison.cpu_checker_red_invariants.contains("I-7.9"),
+        "流 {} 上 CPU 判过 I-7.9 而成立、一个状态都不红它（成立过的：{:?}；红过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_held_invariants,
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
````

## batch9-named-limit/patch-without-e171

实现员报告：`/tmp/claude-1000/m3-r1/batch9-named-limit/patch-without-e171/report.md`（sha256 3196cc8f3420ee6b…），补丁 sha256 280672fb9a259626…

````diff
diff --git a/crates/singlefs-core/src/write_request_split.rs b/crates/singlefs-core/src/write_request_split.rs
--- a/crates/singlefs-core/src/write_request_split.rs
+++ b/crates/singlefs-core/src/write_request_split.rs
@@ -198,14 +198,26 @@
         assert_eq!(transactions[1].payload_length_in_bytes(), 1);
     }
 
-    /// 跨过 67（journal 记录点名项上限那一档）与 144（一片 extent 叶装得下的记录数）两个门槛：
-    /// 单元数、区间的连续性、事务号的连号与升序都不因门槛变。整份内容被恰好一次地盖住——
+    /// 跨过 64（journal 记录点名项上限 ⌊(4096 − 487) ÷ 56⌋ 那一档）与 144（一片 extent 下段叶罩的单元数
+    /// ⌊(16384 − 163) ÷ 112⌋）两个门槛：单元数、区间的连续性、事务号的连号与升序都不因门槛变。整份内容被恰好一次地盖住——
     /// 相邻两个事务首尾相接、不重不漏。
     #[test]
-    fn the_split_covers_every_file_byte_exactly_once_across_the_sixty_seven_and_one_hundred_forty_four_unit_thresholds(
+    fn the_split_covers_every_file_byte_exactly_once_across_the_sixty_four_and_one_hundred_forty_four_unit_thresholds(
     ) {
+        // 切分本身不读这两个常量，取样点写成字面量；先钉住两个门槛今天的值，门槛一变这里先红，
+        // 取样点才会跟着挪，不会贴着旧门槛照样绿下去。
+        assert_eq!(
+            singlefs_format::JOURNAL_NAMED_ENTRIES_PER_RECORD,
+            64,
+            "取样点 63 / 64 / 65 压的是一条 journal 记录的点名项上限；上限变了要跟着挪取样点"
+        );
+        assert_eq!(
+            singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
+            144,
+            "取样点 143 / 144 / 145 压的是一片 extent 下段叶罩的单元数；它变了要跟着挪取样点"
+        );
         let payload_capacity = payload_capacity_in_bytes();
-        for expected_unit_count in [66u64, 67, 68, 143, 144, 145] {
+        for expected_unit_count in [63u64, 64, 65, 143, 144, 145] {
             for last_unit_payload in [1u64, payload_capacity] {
                 let content_length_in_bytes =
                     (expected_unit_count - 1) * payload_capacity + last_unit_payload;
diff --git a/crates/singlefs-harness/tests/multi_record_publishes_of_transaction_zero.rs b/crates/singlefs-harness/tests/multi_record_publishes_of_transaction_zero.rs
--- a/crates/singlefs-harness/tests/multi_record_publishes_of_transaction_zero.rs
+++ b/crates/singlefs-harness/tests/multi_record_publishes_of_transaction_zero.rs
@@ -2,8 +2,10 @@
 //!
 //! - 树表 0 条的一版上写行（`publish_instance_table_on_version_without_file`）点名项多于一条记录装得下时，接到带文件那一路同一个切法
 //!   （`roles_named_by_each_record_of_the_publish`）：装满一条再开下一条，只有真正的最后一条带「本次发布末条」标志与提交标记，
-//!   本次发布内序号依次 1..N（实十六接续报告 `research/prompts/m2-writepath-implementer-report.md` 第六节 Q7：片数 ≥ 67 时这一路原先只写一条、
-//!   往 4096 字节的记录里写第 68 项时越界 panic）。产品路径上要实例表长到 67 片才走得到，这里用只供测试的开关
+//!   本次发布内序号依次 1..N（实十六接续报告 `research/prompts/m2-writepath-implementer-report.md` 第六节 Q7：这一路原先只写一条；
+//!   点名项是实例表各片加分配记录树这次重写的节点，一条记录装 64 项（`JOURNAL_NAMED_ENTRIES_PER_RECORD`），多于 64 项时
+//!   往 4096 字节的记录里写第 65 项越界 panic）。产品路径上要片数加重写的节点数多于 64 才走得到（分配记录树只重写一个节点时
+//!   是实例表长到 64 片），这里用只供测试的开关
 //!   （`PoolWriter::set_journal_record_named_entry_capacity`）把一条记录装的项数压到 1，两个单元就造得出跨记录的写行。
 //! - 事务号 0 的发布跨多条记录时，前几条的提交标记写 0；读者读到提交标记 2..=255 当那条记录损坏、断链即止（
 //!   与已定项 4 读者规则同一处置），池级 checker 的 I-8.8 判「提交标记字节」违例——两边说的是同一件事。
diff --git a/crates/singlefs-harness/tests/multi_unit_file_write_and_read.rs b/crates/singlefs-harness/tests/multi_unit_file_write_and_read.rs
--- a/crates/singlefs-harness/tests/multi_unit_file_write_and_read.rs
+++ b/crates/singlefs-harness/tests/multi_unit_file_write_and_read.rs
@@ -7,7 +7,7 @@
 //! 的定案（N 个数据单元 = N 个事务 = N 条记录）；D4（校验和位置） 已定项 5（净荷 32634，文件偏移到单元做除法）。
 //!
 //! 七条用例：
-//! 1. 写 68 个单元（跨过 67）再写 144 个单元（一片 extent 叶装满），冷启动顺序读回逐字节相同；一次发布的记录条数 = 数据单元数；
+//! 1. 写 65 个单元（跨过 64）再写 144 个单元（一片 extent 叶装满），冷启动顺序读回逐字节相同；一次发布的记录条数 = 数据单元数；
 //!    extent 树节点只在打开文件时按需读、读取次数 ≤ 树高 + 叶数（顺序读期间块层一次 16 KiB 节点读都没有）；池级 checker 一条违例都没有。
 //!    多于 144 个单元时 extent 树下段长成两层（D8（核心索引结构） 已定项 14），下段几种变形走一遍的是
 //!    `the_lower_extent_segment_grows_to_two_levels_past_one_leaf_and_shrinks_back_to_inline`。
@@ -47,7 +47,9 @@
     FIRST_INODE_NUMBER,
 };
 use singlefs_core::unit::{data_unit_payload_capacity, unit_filesystem_identifier};
-use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
+use singlefs_format::{
+    JOURNAL_NAMED_ENTRIES_PER_RECORD, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES,
+};
 use singlefs_harness::memory_pool::MemoryPool;
 use singlefs_harness::read_tally::{JournalRingRegion, ReadCountingPoolReader};
 use singlefs_harness::{RecordedOperationKind, RetainedOperation};
@@ -198,33 +200,39 @@
     );
 }
 
-/// 验收第 1、2 条：68 个单元（跨过一条记录 67 个点名项那个门槛）再 144 个单元（一片 extent 叶装满），
+/// 验收第 1、2 条：65 个单元（跨过一条记录 64 个点名项那个门槛）再 144 个单元（一片 extent 叶装满），
 /// 每次冷启动读回逐字节相同，一次发布的记录条数 = 数据单元数，extent 节点读取次数 ≤ 树高 + 叶数，池级 checker 没有违例。
 #[test]
-fn units_written_across_the_sixty_seven_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte(
+fn units_written_across_the_sixty_four_threshold_and_up_to_a_full_extent_leaf_read_back_cold_byte_for_byte(
 ) {
+    // 写路径不读这个上限，单元数写成字面量；先钉住上限今天的值，上限一变这里先红，单元数才会跟着挪，不会贴着旧上限照样绿下去。
+    assert_eq!(
+        JOURNAL_NAMED_ENTRIES_PER_RECORD + 1,
+        65,
+        "第一次写的单元数是一条 journal 记录的点名项上限 ⌊(4096 − 487) ÷ 56⌋ = 64 加一；上限变了要跟着挪单元数"
+    );
     let mut pool = build_pool("parallel-line-one-large-file");
-    let sixty_eight_units = content_needing(68, 1);
-    let first = sequential_write(&mut pool, &sixty_eight_units, InstanceGeneration(1));
-    assert_eq!(first.data_pointers.len(), 68);
+    let sixty_five_units = content_needing(65, 1);
+    let first = sequential_write(&mut pool, &sixty_five_units, InstanceGeneration(1));
+    assert_eq!(first.data_pointers.len(), 65);
     let image_after_the_first = pool.memory_pool();
     assert_eq!(
         records_on_disk_with_txg(&image_after_the_first, first.root.checkpoint_txg),
-        68,
-        "一次发布的记录条数 = 数据单元数 68（C310 用户定案的口径），67 那个点名项上限够不着"
+        65,
+        "一次发布的记录条数 = 数据单元数 65（C310 用户定案的口径），64 那个点名项上限够不着"
     );
     // 144 个单元之内（D8（核心索引结构） 已定项 14）：上段一片根兼叶，下段一片根兼叶。
-    cold_sequential_read_matches(&image_after_the_first, &sixty_eight_units, 68, (2, 2));
+    cold_sequential_read_matches(&image_after_the_first, &sixty_five_units, 65, (2, 2));
     let (content, effective_root) = recovered(&image_after_the_first);
     assert!(
-        content == sixty_eight_units,
-        "冷走读（recover）读回 68 个单元"
+        content == sixty_five_units,
+        "冷走读（recover）读回 65 个单元"
     );
     assert_eq!(
         effective_root,
         (first.root.instance, first.root.checkpoint_txg)
     );
-    assert_the_pool_checker_finds_no_violation(&image_after_the_first, "68 个单元之后");
+    assert_the_pool_checker_finds_no_violation(&image_after_the_first, "65 个单元之后");
 
     let one_hundred_forty_four_units = content_needing(144, 2);
     let second = sequential_write(
@@ -233,7 +241,7 @@
         InstanceGeneration(1),
     );
     assert_eq!(second.data_pointers.len(), 144);
-    for position in 0..68u64 {
+    for position in 0..65u64 {
         let replaced_slot = first
             .unit(TransactionUnit::Data(DataUnitIndexInFile(position)))
             .slot;
diff --git a/crates/singlefs-harness/tests/sequential_write_one_record_per_data_unit.rs b/crates/singlefs-harness/tests/sequential_write_one_record_per_data_unit.rs
--- a/crates/singlefs-harness/tests/sequential_write_one_record_per_data_unit.rs
+++ b/crates/singlefs-harness/tests/sequential_write_one_record_per_data_unit.rs
@@ -3,7 +3,7 @@
 //! 三件事：
 //! ① 一次写请求按切分纪律切成若干一单元事务（D16（发布语义） 已定项 5 末段、D23（journal 的角色与格式） 已定项 7、
 //!    C310（事务切分纪律与记录数口径打架） 的定案），切出一个事务的那一档与覆盖写走同一条发布路径、写出的字节一个不变；
-//! ② 切出不止一个事务时一次发布写出一个事务一条记录，跨过 67 那个点名项上限照样走得通，一片 extent 叶装满的 144 也走得通，
+//! ② 切出不止一个事务时一次发布写出一个事务一条记录，跨过 64 那个点名项上限照样走得通，一片 extent 叶装满的 144 也走得通，
 //!    多于 144 个单元时 extent 树下段长成两层（D8（核心索引结构） 已定项 14 的 K2），同样走得通。
 
 mod common;
@@ -18,6 +18,7 @@
 use singlefs_core::write_request_split::{
     data_unit_count_of_a_sequential_write, split_sequential_write_into_one_unit_transactions,
 };
+use singlefs_format::JOURNAL_NAMED_ENTRIES_PER_RECORD;
 use singlefs_harness::RetainedOperation;
 
 /// 一份可复现的内容：长度由调用点给，字节只随下标走。
@@ -116,8 +117,8 @@
     );
 }
 
-/// 单元数跨过 67（一条 journal 记录的点名项上限）走得通：一次发布的记录条数 = 数据单元数
-/// （C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案），67 那个点名项上限够不着——每条记录只点名一个数据单元，
+/// 单元数跨过 64（一条 journal 记录的点名项上限）走得通：一次发布的记录条数 = 数据单元数
+/// （C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案），64 那个点名项上限够不着——每条记录只点名一个数据单元，
 /// 末条再加共享的提交内生块。144（一片 extent 叶装满）也走得通；145 起 extent 树下段长成两层（D8（核心索引结构） 已定项 14），
 /// 同样走得通（下段的形状钉在 `multi_unit_file_write_and_read.rs`）。
 /// 一种单元数的顺序写：切分算出的事务数 = 单元数；一次发布写出与单元数相同条数的记录、末条之前每条只点名一个数据单元。
@@ -162,15 +163,26 @@
     sequential_write_publishes_one_record_per_data_unit(2);
 }
 
-/// 67 是一条 journal 记录的点名项上限。
+/// 64 是一条 journal 记录的点名项上限 ⌊(4096 − 487) ÷ 56⌋。
 #[test]
-fn sequential_write_of_67_data_units_at_the_named_item_limit_publishes_67_records() {
-    sequential_write_publishes_one_record_per_data_unit(67);
+fn sequential_write_of_64_data_units_at_the_named_item_limit_publishes_64_records() {
+    // 写路径不读这个上限，单元数写成字面量；先钉住上限今天的值，上限一变这里先红，单元数才会跟着挪，不会贴着旧上限照样绿下去。
+    assert_eq!(
+        JOURNAL_NAMED_ENTRIES_PER_RECORD, 64,
+        "这条用例的单元数压在一条 journal 记录的点名项上限上；上限变了要跟着挪单元数"
+    );
+    sequential_write_publishes_one_record_per_data_unit(64);
 }
 
 #[test]
-fn sequential_write_of_68_data_units_past_the_named_item_limit_publishes_68_records() {
-    sequential_write_publishes_one_record_per_data_unit(68);
+fn sequential_write_of_65_data_units_past_the_named_item_limit_publishes_65_records() {
+    // 同上一条：单元数是上限加一，先钉住上限今天的值。
+    assert_eq!(
+        JOURNAL_NAMED_ENTRIES_PER_RECORD + 1,
+        65,
+        "这条用例的单元数是一条 journal 记录的点名项上限加一；上限变了要跟着挪单元数"
+    );
+    sequential_write_publishes_one_record_per_data_unit(65);
 }
 
 /// 144 装满一片 extent 叶。
````

## batch9-9-8/patch

实现员报告：`/tmp/claude-1000/m3-r1/batch9-9-8/patch/report.md`（sha256 85ee00e2517e998c…），补丁 sha256 38405091fed3123d…

````diff
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs b/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs
index 90373fb..bf9da2f 100644
--- a/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs
@@ -2,7 +2,7 @@
 //! 崩溃放量流水线在里程碑「覆盖写、释放、回退与复用」的整条固定脚本上（步 0 到步 5：取号加暖机 → 新池新建文件 A → 覆盖写 B →
 //! 重开可写挂载（取号、写行、暖机两次）→ 覆盖写 C → 挂着回退到 A（D）→ 五次覆盖写（txg 10–14）→ 抬 F 到 11（两次空发布 txg 15、16）→
 //! 复用 E（txg 17，数据单元落回 50176）→ 正常卸载（先写 F、两次带卸载记号的空发布 txg 18、19））：与层 0 全量跑的那条流同一条
-//! （`crash_enumeration_fixed_script_stream.rs` 的 `ReuseAfterRaisingFloorThenNormalUnmount`，78 段、477 次写、19 条根槽写）。
+//! （`crash_enumeration_fixed_script_stream.rs` 的 `ReuseAfterRaisingFloorThenNormalUnmount`，78 段、481 次写、19 条根槽写）。
 //! 入口都由 `common_crash_points` 设点，前 36 个点与到 D 那条测试共享。展开上限 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`（默认 6），
 //! 28 及以上就是全域（`exhaustive=true`）。要 `verdict-store` 特性。
 #![cfg(feature = "verdict-store")]
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs
index eeb2e47..88ab0d2 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs
@@ -88,23 +88,27 @@ enum Script {
 
 /// 全量那条流在进度文件名里的名字（断点续跑按「输入指纹 + 流名 + 计划哈希」分格）。
 const FULL_ENUMERATION_STREAM_NAME: &str = "file_overwrite_fixed_script_through_e_then_unmount";
-/// 到 E 再正常卸载那条流：每次写只取两态时的全量闭式 1 + Σ(2^|段| − 1)（78 段；六个 28 写段各 2^28 − 1 占 97%）。代码审阅第 19 条之后
+/// 到 E 再正常卸载那条流：每次写只取两态时的全量闭式 1 + Σ(2^|段| − 1)（78 段；六个 28 写段各 2^28 − 1 占 96%）。代码审阅第 19 条之后
 /// 取号、抬 F、卸载先写系统配置之前各多一道屏障，把上一次轮换那 2 写与它们的 2 写拆成两段：三处各 15 → 3 + 3，比改之前的 6649413746 少 27。
 /// C577（发布返回之前、系统配置轮换之后加一道屏障）之后每次轮换那 2 写都自成一段，不再与下一次发布的单元写同段：十四处 2 + n 写段
 /// 各拆成 2 与 n，每处 (2^(n+2) − 1) → 3 + (2^n − 1)、少 3 · 2^n − 3（n = 24 三处、16 四处、20 一处、28 六处——A、B、C、写行之后的两次暖机、
 /// 回退 D、五次覆盖写、抬 F 与卸载各第二次空发布、E），64 段 6649413719 → 78 段 1662648449。
-const FULL_STATES_WITH_TWO_STATES_PER_WRITE_THROUGH_THE_UNMOUNT: u64 = 1_662_648_449;
+/// 回退那次发布追加一条谱系记录之后（D18（块里携带什么信息） 已定项 24 第 5 条：零根的谱系树上第一次追加，写谱系容器与谱系树根，两块盘各一份），
+/// D 的单元写 20 → 24，那一段 2^20 − 1 → 2^24 − 1、多 15728640：1662648449 → 1678377089，段数不变。
+const FULL_STATES_WITH_TWO_STATES_PER_WRITE_THROUGH_THE_UNMOUNT: u64 = 1_678_377_089;
 /// 全量枚举的状态数（代码审阅第 4 条，用户 2026-09-27 定「原地覆写补第三态、全量也跑」）：这条流上取三态的只有 46 次系统配置槽写，
 /// 每段 3^m · 2^(n−m) − 1（m 是段里系统配置槽写的个数），再加全部持久那一个。C577 之后 46 次系统配置槽写两两自成 23 个 2 写段，
-/// 每段 3² − 1 比两态时的 2² − 1 多 5：1662648449 + 23 × 5 = 1662648564（C577 之前轮换并进 30 写段、3² · 2²⁸ − 1，14960689284）。
-const FULL_STATES_THROUGH_THE_UNMOUNT: u64 = 1_662_648_564;
+/// 每段 3² − 1 比两态时的 2² − 1 多 5：1678377089 + 23 × 5 = 1678377204（D 加谱系记录之前 1662648449 + 23 × 5 = 1662648564；
+/// C577 之前轮换并进 30 写段、3² · 2²⁸ − 1，14960689284）。
+const FULL_STATES_THROUGH_THE_UNMOUNT: u64 = 1_678_377_204;
 /// 全量的状态按发布分：每段的状态归它后面最近的那次根槽写，再加全部持久那一个。
 /// C577 之后一次带单元的普通发布：上一次轮换段 3² − 1 = 8 + 单元段 2^n − 1 + 记录段 3 + 根槽段 1 = 2^n + 11
-/// （n = 24：16777227，n = 16：65547，n = 20：1048587，n = 28：268435467；C577 之前轮换与单元同段、3² · 2^n − 1 + 4 = 9 · 2^n + 3）；
+/// （n = 24：16777227，n = 16：65547，n = 28：268435467；回退 D（txg 9）加谱系记录之后 n = 24、16777227，之前 n = 20、1048587；
+/// C577 之前轮换与单元同段、3² · 2^n − 1 + 4 = 9 · 2^n + 3）；
 /// 写行 txg 5 与抬 F、卸载各第一次空发布（txg 15、18）前面是屏障，没有变。
 const FULL_STATES_BY_PUBLISH_THROUGH_THE_UNMOUNT: &str = "instance1_txg1=12 instance1_txg2=12 \
      instance1_txg3=16777227 instance1_txg4=16777227 instance2_txg5=262163 instance2_txg6=65547 \
-     instance2_txg7=65547 instance2_txg8=16777227 instance2_txg9=1048587 instance2_txg10=268435467 \
+     instance2_txg7=65547 instance2_txg8=16777227 instance2_txg9=16777227 instance2_txg10=268435467 \
      instance2_txg11=268435467 instance2_txg12=268435467 instance2_txg13=268435467 \
      instance2_txg14=268435467 instance2_txg15=65555 instance2_txg16=65547 instance2_txg17=268435467 \
      instance2_txg18=65555 instance2_txg19=65547 after_the_last_root=8 every_write_persisted=1";
@@ -388,7 +392,9 @@ fn prepare(tag: &str, script: Script) -> Prepared {
     // 每次发布：它的单元写、两份 journal 记录、一条根槽写、之后两块盘各一次系统配置槽轮换，共「单元写数 + 5」次写；
     // C577 之后轮换之后、发布返回之前有一道屏障，上一次发布的系统配置槽轮换自成一段，不再与这一次的单元写并成一段
     // （登记表八末尾那条 ⚠️ 说的并段不再有）。单元写数：暖机 0、A 与 B 与 C 各 24、
-    // 写行发布 18、写行之后的暖机各 16、回退 D 20、覆盖写与 E 各 28、抬 F 与卸载推的空发布各 16。取号、抬 F 与卸载那一串开头各多
+    // 写行发布 18、写行之后的暖机各 16、回退 D 24（分配记录树 7 个节点 + 记账 + 映射 + 树表 = 10 个固定点单元，加谱系容器与谱系树根
+    // 2 个——D18（块里携带什么信息） 已定项 24 第 5 条，回退那次追加一条谱系记录；加谱系记录之前 20）、覆盖写与 E 各 28、
+    // 抬 F 与卸载推的空发布各 16。取号、抬 F 与卸载那一串开头各多
     // 两块盘一次系统配置写（取号写实例代号，抬 F 与卸载先写新 F，D16（发布语义） 已定项 1「抬 F 那一串」），它们之前各一道屏障
     // （代码审阅第 19 条：覆写较旧那一槽之前，上一次轮换写的那一槽先持久），与上一次的轮换各自成段（2、2；改之前并成 4 写一段）。
     match script {
@@ -418,16 +424,17 @@ fn prepare(tag: &str, script: Script) -> Prepared {
             );
         }
         Script::RollbackToFirstVersion => {
-            // 挂着时的向前回退 D：20 个单元写自成一段、C 的两个系统配置槽写在它前面自成一段（C577 之前并成 22），不取号、不写行、不暖机。
+            // 挂着时的向前回退 D：24 个单元写自成一段（10 个固定点单元加谱系容器与谱系树根，各两块盘；加谱系记录之前 20）、
+            // C 的两个系统配置槽写在它前面自成一段（C577 之前并成 22），不取号、不写行、不暖机。
             assert_eq!(
                 sizes,
                 vec![
                     2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2, 16,
-                    2, 1, 2, 24, 2, 1, 2, 20, 2, 1, 2
+                    2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2
                 ],
                 "固定脚本到 D 为止的段序列"
             );
-            assert_eq!(writes.len(), 166 + 25, "到 C 166 + D 25");
+            assert_eq!(writes.len(), 166 + 29, "到 C 166 + D 29");
         }
         Script::ReuseAfterRaisingFloorThenNormalUnmount => {
             // 五次覆盖写各 28 个单元写自成一段、上一次的两个系统配置槽写在它前面自成一段（C577 之前并成 30）；txg 14 的轮换自成一段、
@@ -437,7 +444,7 @@ fn prepare(tag: &str, script: Script) -> Prepared {
                 sizes,
                 vec![
                     2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2, 16,
-                    2, 1, 2, 24, 2, 1, 2, 20, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28,
+                    2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28,
                     2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2,
                     16, 2, 1, 2
                 ],
@@ -445,8 +452,8 @@ fn prepare(tag: &str, script: Script) -> Prepared {
             );
             assert_eq!(
                 writes.len(),
-                191 + 5 * 33 + (2 + 2 * 21) + 33 + (2 + 2 * 21),
-                "到 D 191 + 五次覆盖写 165 + 抬 F 44 + E 33 + 卸载 44"
+                195 + 5 * 33 + (2 + 2 * 21) + 33 + (2 + 2 * 21),
+                "到 D 195 + 五次覆盖写 165 + 抬 F 44 + E 33 + 卸载 44"
             );
         }
         Script::ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish => {
@@ -455,7 +462,7 @@ fn prepare(tag: &str, script: Script) -> Prepared {
                 sizes,
                 vec![
                     2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2, 16,
-                    2, 1, 2, 24, 2, 1, 2, 20, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28,
+                    2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28,
                     2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2,
                     28, 2, 1, 2
                 ],
@@ -463,8 +470,8 @@ fn prepare(tag: &str, script: Script) -> Prepared {
             );
             assert_eq!(
                 writes.len(),
-                191 + 5 * 33 + 44 + 33 + 2 * 33,
-                "到 E 433 + txg 18、19 的覆盖写各 33"
+                195 + 5 * 33 + 44 + 33 + 2 * 33,
+                "到 E 437 + txg 18、19 的覆盖写各 33"
             );
         }
     }
@@ -624,14 +631,16 @@ fn the_fixed_script_through_the_third_publish_keeps_its_registered_segment_seque
     );
 }
 
-/// 到 D 为止的固定脚本：同上，36 段、每次写只取两态时闭式 51773503、甲二 77；补第三态之后全量 51773558、甲二 132。
-/// C577 之前 30 段、206307373、95、463863878、180（到 C 那五处之外 D 的 20 个单元写也与 C 的轮换同段、22）；
+/// 到 D 为止的固定脚本：同上，36 段、每次写只取两态时闭式 67502143、甲二 77；补第三态之后全量 67502198、甲二 132。
+/// 回退那次追加谱系记录之后 D 的单元写 20 → 24（谱系容器与谱系树根各两块盘，D18（块里携带什么信息） 已定项 24 第 5 条）：
+/// 两态闭式与全量各多 2^24 − 2^20 = 15728640（之前 51773503、51773558）；只有单元写的那一段甲二只取全不落一态，77、132 不变。
+/// C577 之前 30 段、206307373、95、463863878、180（到 C 那五处之外 D 当时的 20 个单元写也与 C 的轮换同段、22）；
 /// 代码审阅第 19 条之前 29 段、206307382、104。
 #[test]
 fn the_fixed_script_through_the_rollback_publish_keeps_its_registered_segment_sequence() {
     let prepared = prepare("layer0-d-registered", Script::RollbackToFirstVersion);
     assert_eq!(prepared.segments.len(), 36);
-    assert_eq!(closed_form_state_count(&prepared.segments), 51_773_503);
+    assert_eq!(closed_form_state_count(&prepared.segments), 67_502_143);
     assert_eq!(
         layer0_state_count(&prepared.writes, &prepared.segments, &quick_tier_expansion),
         77
@@ -651,7 +660,7 @@ fn the_fixed_script_through_the_rollback_publish_keeps_its_registered_segment_se
                 &quick_tier_expansion
             )
         ),
-        (51_773_558, 132)
+        (67_502_198, 132)
     );
 }
 
````

## batch9-9-8b/patch

实现员报告：`/tmp/claude-1000/m3-r1/batch9-9-8b/patch/report.md`（sha256 49b56be18013a8f1…），补丁 sha256 5a7a756710b6a14a…

````diff
--- a/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs
@@ -125,11 +125,11 @@
         sizes,
         vec![
             2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2, 16, 2, 1,
-            2, 24, 2, 1, 2, 20, 2, 1, 2
+            2, 24, 2, 1, 2, 24, 2, 1, 2
         ],
         "固定脚本到 D 为止的段序列（与 crash_enumeration_fixed_script_stream 登记的相同）"
     );
-    assert_eq!(writes.len(), 191, "到 C 166 + D 25");
+    assert_eq!(writes.len(), 195, "到 C 166 + D 29");
     let crash_points = recorder.into_crash_points(&writes, &segments, &stream_indexes);
     assert_eq!(crash_points.len(), segments.len(), "每个屏障段一个崩溃点");
     // 按入口归拢：名字 `<入口>.<周期>.<步>` 的第一段是入口
@@ -150,9 +150,9 @@
             ("publish_overwrite", 29),
             ("mount_writable", 67),
             ("publish_overwrite", 29),
-            ("roll_back_by_a_forward_publish", 25)
+            ("roll_back_by_a_forward_publish", 29)
         ],
-        "六个入口各管的写数：取号 2 + 暖机 5 × 2；A 29；B 29；重开取号 2 + 写行 23 + 暖机 21 × 2；C 29；D 25"
+        "六个入口各管的写数：取号 2 + 暖机 5 × 2；A 29；B 29；重开取号 2 + 写行 23 + 暖机 21 × 2；C 29；D 29（单元写 24：10 个固定点单元加谱系容器与谱系树根，各两块盘）"
     );
     let names: Vec<&str> = crash_points
         .iter()
@@ -194,12 +194,12 @@
             "publish_overwrite.journal_record.1",
             "publish_overwrite.root_record_fua.1",
             "publish_overwrite.system_configuration_slot.1",
-            "roll_back_by_a_forward_publish.index_node.1",
+            "roll_back_by_a_forward_publish.index_node+packed_records.1",
             "roll_back_by_a_forward_publish.journal_record.1",
             "roll_back_by_a_forward_publish.root_record_fua.1",
             "roll_back_by_a_forward_publish.system_configuration_slot.1"
         ],
-        "36 个点的名字 <入口>.<周期：写的种类，单元写按类标签分 data_unit / index_node / packed_records>.<这个入口里这一种的第几段>：取号加暖机 7 段、A 4 段、B 4 段、重开 13 段（写行那一段是打包记录加索引节点、暖机两段只有索引节点）、C 4 段、D 4 段（回退只写索引节点）"
+        "36 个点的名字 <入口>.<周期：写的种类，单元写按类标签分 data_unit / index_node / packed_records>.<这个入口里这一种的第几段>：取号加暖机 7 段、A 4 段、B 4 段、重开 13 段（写行那一段是打包记录加索引节点、暖机两段只有索引节点）、C 4 段、D 4 段（回退那一段先写固定点的索引节点、再写谱系容器这个打包记录单元，D18（块里携带什么信息） 已定项 24 第 5 条）"
     );
     assert!(
         crash_points
--- a/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs
@@ -180,15 +180,15 @@
         sizes,
         vec![
             2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2, 16, 2, 1,
-            2, 24, 2, 1, 2, 20, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2,
+            2, 24, 2, 1, 2, 24, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2,
             1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2
         ],
         "固定脚本到 E 再正常卸载的段序列（与 crash_enumeration_fixed_script_stream 登记的相同）"
     );
     assert_eq!(
         writes.len(),
-        191 + 5 * 33 + (2 + 2 * 21) + 33 + (2 + 2 * 21),
-        "到 D 191 + 五次覆盖写 165 + 抬 F 44 + E 33 + 卸载 44"
+        195 + 5 * 33 + (2 + 2 * 21) + 33 + (2 + 2 * 21),
+        "到 D 195 + 五次覆盖写 165 + 抬 F 44 + E 33 + 卸载 44"
     );
     let crash_points = recorder.into_crash_points(&writes, &segments, &stream_indexes);
     assert_eq!(crash_points.len(), segments.len(), "每个屏障段一个崩溃点");
@@ -209,7 +209,7 @@
             ("publish_overwrite", 29),
             ("mount_writable", 67),
             ("publish_overwrite", 29),
-            ("roll_back_by_a_forward_publish", 25),
+            ("roll_back_by_a_forward_publish", 29),
             ("publish_overwrite", 33 * 5),
             ("raise_rollback_floor", 44),
             ("publish_overwrite", 33),
@@ -257,7 +257,7 @@
             "publish_overwrite.journal_record.1",
             "publish_overwrite.root_record_fua.1",
             "publish_overwrite.system_configuration_slot.1",
-            "roll_back_by_a_forward_publish.index_node.1",
+            "roll_back_by_a_forward_publish.index_node+packed_records.1",
             "roll_back_by_a_forward_publish.journal_record.1",
             "roll_back_by_a_forward_publish.root_record_fua.1",
             "roll_back_by_a_forward_publish.system_configuration_slot.1",
````

