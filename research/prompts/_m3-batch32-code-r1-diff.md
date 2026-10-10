# 删除批第 32 批 P 合入的改动原样（2026-10-07）

## 实现员 p：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch32/p/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index a9db071..a41caa2 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -1240,6 +1240,35 @@ impl<'reader> Walk<'reader> {
         }
     }
 
+    /// 按位置条目读一个走读要往下走的单元（每条位置条目照判 I-2.1，同 [`read_referenced_unit`]），另交回「走过」（`visited_units`）要认的落点：
+    /// 读回、自证过的那一份的落点，不是第 0 条位置条目的（攻方 F-KEY，判决 `research/prompts/m3-batch31-code-r1-main-verification.md`
+    /// 第二节 K2 观察 O3 那一行、第三节第 2 条；同实例表链那一处 `placement_it_was_read_from` 的写法）。第 0 条位置条目指着这一遍先走过的
+    /// 别的单元、第 1 条那一份完好时，按第 0 条认会把读回来的这一份当成走过、它的头与格式（I-1.3、出生身份、I-1.7）一个字节都不判。
+    /// 别处的键（父单元、诞生代号那张表、并进前一版的落点表、inode 叶序）照旧按第 0 条位置条目。
+    fn read_a_unit_to_walk_and_the_placement_it_was_read_from(
+        &mut self,
+        pointer: &PointerView,
+        unit_bytes: usize,
+        what: &str,
+    ) -> Option<(Vec<u8>, (u32, u64))> {
+        crate::image::read_referenced_unit_with_the_location_it_was_read_from(
+            self.reader,
+            &mut self.judgements,
+            &pointer.locations,
+            unit_bytes,
+            what,
+        )
+        .map(|(unit, location_it_was_read_from)| {
+            (
+                unit,
+                (
+                    location_it_was_read_from.device,
+                    location_it_was_read_from.slot,
+                ),
+            )
+        })
+    }
+
     /// 一个单元别的版本已经走过（`visited_units` 命中），这一版又引用它：不重走，把它下面的落点（孩子、孩子的孩子……，
     /// 按 `placements_referenced_below_a_walked_unit` 那张表）逐个并进这一版的集合再比交叉——老一版在共享子树之外又指了子树里面的
     /// 某个单元，就在这里红（实审 B1 留下的缺口）。一个单元的孩子只展开一次（表里有环时也走得完：每个单元至多进一次 `expanded`）。
@@ -1470,6 +1499,8 @@ impl<'reader> Walk<'reader> {
                 .push(format!("{what} 两份都读不到对得上的"));
             return None;
         };
+        // 「走过」照旧按第 0 条位置条目认（不跟 F-KEY 那几处改）：调用方（多层码 2 树、树表）在读之前就按第 0 条位置条目问
+        // `visited_units` 定「别的版本先走过」，读回 None 时按它分「先走过」与「走不下去」；只改这里的键，两处就按两个落点认同一件事。
         let placement = (pointer.locations[0].device, pointer.locations[0].slot);
         if !self.visited_units.insert(placement) {
             self.merge_placements_below_a_unit_walked_by_an_earlier_version(placement);
@@ -2370,9 +2401,20 @@ impl<'reader> Walk<'reader> {
                 ContainerTreeEntryType::InternalNode => {
                     // 「子单元是码 2」与 I-1.6（类标签登记且三处相等） 重合，照 I-9.2（条目身份与子头相符） 的先例留作合取
                     // （第二节 T7）：读不出的孩子这一小句不判，读不出归 I-2.1 与走读失败。
+                    // 孩子的类只认按它自己那一类的长度整份自证过的副本（F-ID-b，用户 2026-10-07 定，判决
+                    // `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K2 N1 盘序翻面 A3 那一行、第四节）：头里是码 3、
+                    // 只有前 16 KiB 与位置条目里的校验和对得上的那一份不算，不随它在哪块盘翻面。
+                    // 两份都没自证过时怎么认，条款没写（交主 agent）：这一形照改之前的读法，取第一份按 16 KiB 对得上的。
                     let (birth_tree, _, container, container_birth) = identity;
-                    let child_class = read_unit_without_judging(self.reader, &child, node_bytes())
-                        .map(|unit| unit[6]);
+                    let child_class =
+                        class_tag_of_the_first_copy_verifying_at_the_length_of_its_class(
+                            self.reader,
+                            &child,
+                        )
+                        .or_else(|| {
+                            read_unit_without_judging(self.reader, &child, node_bytes())
+                                .map(|unit| unit[UNIT_HEADER_CLASS_TAG_OFFSET])
+                        });
                     let entry_holds = birth_tree == 0
                         && container == 0
                         && container_birth == 0
@@ -2553,13 +2595,13 @@ impl<'reader> Walk<'reader> {
     ) {
         let what = format!("{name}内部条目的子指针");
         self.note_references_of_a_pointer_followed_by_the_walk(child, 2, &what);
-        let Some(unit) = read_referenced_unit(
-            self.reader,
-            &mut self.judgements,
-            &child.locations,
-            data_unit_bytes(),
-            &what,
-        ) else {
+        let Some((unit, placement_it_was_read_from)) = self
+            .read_a_unit_to_walk_and_the_placement_it_was_read_from(
+                child,
+                data_unit_bytes(),
+                &what,
+            )
+        else {
             // 容器两份都读不出：「子指针头部出生树 == 身份引用出生树」只比父条目里的两段，照判（判决
             // `research/prompts/m3-batch26-code-r1-main-verification.md` 第二节 K2 H3 H-B 那一行、第三节第 1 条）；比容器头的那几小句不判。
             let identity_birth_tree = identity.0;
@@ -2595,7 +2637,7 @@ impl<'reader> Walk<'reader> {
                 )
             });
         let child_placement = (child.locations[0].device, child.locations[0].slot);
-        if !self.visited_units.insert(child_placement) {
+        if !self.visited_units.insert(placement_it_was_read_from) {
             self.merge_placements_below_a_unit_walked_by_an_earlier_version(child_placement);
             return;
         }
@@ -2801,13 +2843,13 @@ impl<'reader> Walk<'reader> {
                 span_in_slots,
                 "inode 树的孩子",
             );
-            let Some(unit) = read_referenced_unit(
-                self.reader,
-                &mut self.judgements,
-                &child.locations,
-                length,
-                "inode 树的孩子",
-            ) else {
+            let Some((unit, placement_it_was_read_from)) = self
+                .read_a_unit_to_walk_and_the_placement_it_was_read_from(
+                    &child,
+                    length,
+                    "inode 树的孩子",
+                )
+            else {
                 judge_an_inode_entry_whose_child_does_not_read_back(
                     &mut self.judgements,
                     registered_entry_type,
@@ -2836,7 +2878,13 @@ impl<'reader> Walk<'reader> {
                             unit[6]
                         )
                     });
-                    self.walk_inode_leaf_container(&unit, &child, tree, child_placement)
+                    self.walk_inode_leaf_container(
+                        &unit,
+                        &child,
+                        tree,
+                        child_placement,
+                        placement_it_was_read_from,
+                    )
                 }
                 RegisteredInodeInternalEntryType::InternalNode => {
                     let (birth_tree, _, container, container_birth) = identity;
@@ -2848,7 +2896,14 @@ impl<'reader> Walk<'reader> {
                             unit[6]
                         )
                     });
-                    self.walk_inode_internal_child(&unit, &child, node.level, tree, child_placement)
+                    self.walk_inode_internal_child(
+                        &unit,
+                        &child,
+                        node.level,
+                        tree,
+                        child_placement,
+                        placement_it_was_read_from,
+                    )
                 }
             };
             if subtree.walked_first_by_this_version {
@@ -2879,14 +2934,17 @@ impl<'reader> Walk<'reader> {
     /// 一片 inode 叶容器（码 3 打包记录类型 2）：这一遍第一次走到时判树 ID（I-1.3）、出生身份（I-1.2 / I-4.2）、容器（I-1.7 等）与容器里
     /// 每条记录（I-9.13、I-9.7、I-9.4 第一句与第四句、I-9.15），交回它的容器号与装的最小、最大 inode 号；别的版本已经走过时不重读，
     /// 从 `inode_leaf_order_below_a_walked_unit` 接上它在叶序里的那一格，它下面的落点并进这一版（I-5.1）。
+    /// 「走过」按读回的那一份的落点 `placement_it_was_read_from` 认（F-KEY，[`Self::read_a_unit_to_walk_and_the_placement_it_was_read_from`]），
+    /// 别的键照旧按第 0 条位置条目的 `child_placement`。
     fn walk_inode_leaf_container(
         &mut self,
         unit: &[u8],
         child: &PointerView,
         tree: u64,
         child_placement: (u32, u64),
+        placement_it_was_read_from: (u32, u64),
     ) -> InodeSubtreeInTheWalk {
-        if !self.visited_units.insert(child_placement) {
+        if !self.visited_units.insert(placement_it_was_read_from) {
             self.merge_placements_below_a_unit_walked_by_an_earlier_version(child_placement);
             return InodeSubtreeInTheWalk {
                 leaves_in_leaf_order: self
@@ -2991,6 +3049,8 @@ impl<'reader> Walk<'reader> {
     /// 层级 = 父层级 − 1（I-1.1）与条目宽 120（I-1.10），再往下走（[`Self::walk_inode_internal_node`]）；
     /// 别的版本已经走过时不重读，从 `inode_leaf_order_below_a_walked_unit` 接上它下面的叶序，它下面的落点并进这一版（I-5.1）。
     /// 层级 0 的码 2 节点在 inode 树里没有条款定的形状（叶是码 3 容器，I-9.1 那一行只给根留了「空的根」）：记走读失败、不往下走。
+    /// 「走过」按读回的那一份的落点 `placement_it_was_read_from` 认（F-KEY，同 [`Self::walk_inode_leaf_container`]），
+    /// 别的键（父单元、叶序、并进前一版）照旧按第 0 条位置条目的 `child_placement`。
     fn walk_inode_internal_child(
         &mut self,
         unit: &[u8],
@@ -2998,12 +3058,13 @@ impl<'reader> Walk<'reader> {
         parent_level: u8,
         tree: u64,
         child_placement: (u32, u64),
+        placement_it_was_read_from: (u32, u64),
     ) -> InodeSubtreeInTheWalk {
         let not_walked_below = InodeSubtreeInTheWalk {
             leaves_in_leaf_order: Vec::new(),
             walked_first_by_this_version: true,
         };
-        if !self.visited_units.insert(child_placement) {
+        if !self.visited_units.insert(placement_it_was_read_from) {
             self.merge_placements_below_a_unit_walked_by_an_earlier_version(child_placement);
             return InodeSubtreeInTheWalk {
                 leaves_in_leaf_order: self
@@ -3359,19 +3420,19 @@ impl<'reader> Walk<'reader> {
             2,
             &format!("inode {inode} 偏移 {offset} 的数据单元"),
         );
-        let Some(unit) = read_referenced_unit(
-            self.reader,
-            &mut self.judgements,
-            &pointer.locations,
-            data_unit_bytes(),
-            "数据单元",
-        ) else {
+        let Some((unit, placement_it_was_read_from)) = self
+            .read_a_unit_to_walk_and_the_placement_it_was_read_from(
+                &pointer,
+                data_unit_bytes(),
+                "数据单元",
+            )
+        else {
             self.walk_failures
                 .push("数据单元两份都读不到对得上的".to_string());
             return;
         };
         let placement = (pointer.locations[0].device, pointer.locations[0].slot);
-        if !self.visited_units.insert(placement) {
+        if !self.visited_units.insert(placement_it_was_read_from) {
             self.merge_placements_below_a_unit_walked_by_an_earlier_version(placement);
             return;
         }
@@ -4444,6 +4505,26 @@ fn child_unit_class_other_than_the_entry_type_names(
     class_other_than_the_entry_type_names
 }
 
+/// 两条位置条目里第一份按它自己那一类的长度整份自证过的副本头里的类标签：头里的类标签是登记过的码、按那一类的长度读回的整单元校验和
+/// 与它的位置条目对得上（与 [`child_unit_class_other_than_the_entry_type_names`] 认「自证过的孩子」同一个判据）；哪一份都没自证过交 None。
+/// 墓碑树、谱系树类型段 0 那一支认「子单元是码 2」用它（F-ID-b，用户 2026-10-07 定）。只读、不判。
+fn class_tag_of_the_first_copy_verifying_at_the_length_of_its_class(
+    reader: &dyn ImageReader,
+    child: &PointerView,
+) -> Option<u8> {
+    child.locations.iter().find_map(|location| {
+        let offset = location.slot * SLOT_BYTES;
+        // 头按最短的单元（码 2，16 KiB）那么长读：32 KiB 的单元头也在这前 16 KiB 里。
+        let class_tag_in_the_header =
+            reader.read(location.device, offset, node_bytes())?[UNIT_HEADER_CLASS_TAG_OFFSET];
+        let unit_bytes = unit_bytes_of_the_class(MappingKeyUnitClass::of(class_tag_in_the_header))?;
+        reader
+            .read(location.device, offset, unit_bytes)
+            .filter(|content| crc32_castagnoli_table(content) == location.checksum)
+            .map(|_content| class_tag_in_the_header)
+    })
+}
+
 /// [`child_unit_class_other_than_the_entry_type_names`] 认出另一类之后再问一句：有没有一份副本头里的类标签就是 `class_the_entry_type_names`、
 /// 按那一类的长度读回的整单元校验和对得上（判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「K2 H-V 两份都看之后判错」那一行、
 /// 第三节第 1 条，攻方改法 F-W）。有的话读者用的就是这一份（core 按条目那一类的长度逐份读、对得上就用），走读照类型段那一支走它的子树；
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index 7bff84c..023ef26 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -9,7 +9,8 @@
 // 共用段 crash_judge_checker_common.wgsl 里没有它，各写一份；对拍用例 crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs 钉两份相同）。
 // walk.rs 读链不设片数上限：走满这么多片、链还没完，记判不了交 CPU（判决 `research/prompts/m3-batch27-code-r1-main-verification.md`
 // 第二节 K1 J1 那一行 O-1、第三节第 2 条：不当读不出跳过 I-7.9 / I-3.12）。今天走读内核先把根环里每条自证过的根的链按同一个上限走一遍、
-// 走满先记它自己的判不了（站点号 61），扫描内核不跑；这里两处是走读内核哪天不再走某一类根时的接手（读码推的，没有一条流落到这里）
+// 走满先记它自己的判不了（站点号 61），扫描内核不跑；这里两处落在候选根的链途经走读内核已走过的片、链长过上限的那一形（走读内核走到已走过的片
+// 就停、不记 61；坏镜像才造得出，判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第二节 K1 L2 那一行）
 const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 256u;
 
 // r_instance_pages_after_first（候选根的引用集合）走满 INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED 片、链还没完。「条目太多」一类，站点号与别处不撞
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index ed2d14e..97210d2 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -1282,9 +1282,14 @@ fn w_record_below(device: u32, slot: vec2<u32>) {
     w_below_count[w_context] = count + 1u;
 }
 
+// w_read_referenced_unit 最近一次交回的是第二条位置条目那一份（第一条那一份对不上、第二条对得上）：「走过」按读回那一份的落点认的几处读它
+// （w_visit_the_copy_read_back）
+var<private> w_read_back_from_the_second_location: bool;
+
 // image::read_referenced_unit：两条位置条目都读、都判 I-2.1，交回第一份对得上的
 fn w_read_referenced_unit(p: Ptr, unit_bytes: u32) -> Slice {
     var first = no_slice();
+    w_read_back_from_the_second_location = false;
     let s0 = read_at(p.device0, slot_to_bytes(p.slot0), unit_bytes);
     let matches0 = s0.version != NONE && slice_crc_matches(s0, unit_bytes, p.sum0);
     if (!matches0) {
@@ -1302,10 +1307,24 @@ fn w_read_referenced_unit(p: Ptr, unit_bytes: u32) -> Slice {
     w_judge(INVARIANT_I_2_1, matches1);
     if (matches1 && first.version == NONE) {
         first = s1;
+        w_read_back_from_the_second_location = true;
     }
     return first;
 }
 
+// 「走过」按 w_read_referenced_unit 刚读回、自证过的那一份的落点认（walk.rs read_a_unit_to_walk_and_the_placement_it_was_read_from，攻方 F-KEY，
+// 判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第三节第 2 条）：第一条位置条目指着这一遍先走过的别的单元、第二条那一份完好时，
+// 按第一条认会把读回来的这一份当成走过、它的头与格式一个字节都不判。并进前一版、父单元、诞生代号照旧按第一条位置条目
+fn w_visit_the_copy_read_back(p: Ptr) -> bool {
+    var device_read_from = p.device0;
+    var slot_read_from = p.slot0;
+    if (w_read_back_from_the_second_location) {
+        device_read_from = p.device1;
+        slot_read_from = p.slot1;
+    }
+    return w_visit(device_read_from, slot_read_from);
+}
+
 // I-2.5：位置条目按设备身份严格升序；整条全零的指针豁免
 fn w_judge_location_order(p: Ptr) {
     if (p.all_zero) {
@@ -1841,7 +1860,7 @@ fn w_walk_extent_data_pointer(base: u32, o: u32, inode: vec2<u32>, offset: vec2<
         w_failures = w_failures + 1u;
         return;
     }
-    if (!w_visit(p.device0, p.slot0)) {
+    if (!w_visit_the_copy_read_back(p)) {
         w_merge_below(p.device0, p.slot0);
         return;
     }
@@ -2728,8 +2747,8 @@ fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: v
             let header_identity_holds = rd8(s.base, 6u) == 3u && eq64(rd64(s.base, 43u), identity_birth_tree) && rd16(s.base, 51u) == 2u
                 && eq64(rd64(s.base, 53u), identity_container) && eq64(rd64(s.base, 61u), identity_birth) && eq64(child.birth_tree, identity_birth_tree);
             w_judge(INVARIANT_I_9_2, header_identity_holds);
-            // 叶容器
-            if (!w_visit(child.device0, child.slot0)) {
+            // 叶容器（「走过」按读回那一份的落点认，F-KEY）
+            if (!w_visit_the_copy_read_back(child)) {
                 w_merge_below(child.device0, child.slot0);
                 if (w_context == 0u) {
                     w_cached_leaves_below(child.device0, child.slot0, leaves_before);
@@ -3064,6 +3083,20 @@ fn w_child_unit_class_other_than_the_entry_type_names(child: Ptr, class_the_entr
     return class_other_than_the_entry_type_names;
 }
 
+// 两条位置条目里第一份按它自己那一类的长度整份自证过的副本头里的类标签（walk.rs 的 class_tag_of_the_first_copy_verifying_at_the_length_of_its_class）：
+// 头里的类标签是登记过的码、按那一类的长度读回的整单元校验和对得上；哪一份都没自证过交 NONE
+fn w_class_of_the_first_copy_verifying_at_the_length_of_its_class(child: Ptr) -> u32 {
+    let first_copy = read_at(child.device0, slot_to_bytes(child.slot0), NODE_BYTES);
+    if (first_copy.version != NONE && w_unit_verifies_at_the_length_of_its_class(first_copy, rd8(first_copy.base, 6u), child.sum0)) {
+        return rd8(first_copy.base, 6u);
+    }
+    let second_copy = read_at(child.device1, slot_to_bytes(child.slot1), NODE_BYTES);
+    if (second_copy.version != NONE && w_unit_verifies_at_the_length_of_its_class(second_copy, rd8(second_copy.base, 6u), child.sum1)) {
+        return rd8(second_copy.base, 6u);
+    }
+    return NONE;
+}
+
 // w_child_unit_class_other_than_the_entry_type_names 认出另一类之后再问一句（walk.rs 的 some_copy_of_the_child_verifies_as_the_class_the_entry_type_names，
 // 判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第三节第 1 条，攻方改法 F-W）：有没有一份副本头里的类标签就是 class_the_entry_type_names、
 // 按那一类的长度读回的整单元校验和对得上。有的话读者用的就是这一份，走读照类型段那一支走它的子树。只在认出另一类之后调
@@ -3172,9 +3205,17 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
                 w_judge(identity_invariant, class_of_the_child_of_an_internal_entry == 2u);
                 continue;
             }
-            // 类型段 0：身份引用的出生树、容器号、出生代三段为 0；「子单元是码 2」与 I-1.6 重合、照 I-9.2 的先例留作合取，读不出的孩子这一小句不判
-            let child_unit = read_unit_via_locations(child, NODE_BYTES);
-            let child_is_a_code_two_node_or_unreadable = child_unit.version == NONE || rd8(child_unit.base, 6u) == 2u;
+            // 类型段 0：身份引用的出生树、容器号、出生代三段为 0；「子单元是码 2」与 I-1.6 重合、照 I-9.2 的先例留作合取，读不出的孩子这一小句不判。
+            // 孩子的类只认按它自己那一类的长度整份自证过的副本（walk.rs 的 class_tag_of_the_first_copy_verifying_at_the_length_of_its_class，
+            // F-ID-b，用户 2026-10-07 定）；两份都没自证过时条款没写（交主 agent），照改之前取第一份按 16 KiB 对得上的
+            var class_of_the_child_of_a_type_zero_entry = w_class_of_the_first_copy_verifying_at_the_length_of_its_class(child);
+            if (class_of_the_child_of_a_type_zero_entry == NONE) {
+                let child_unit = read_unit_via_locations(child, NODE_BYTES);
+                if (child_unit.version != NONE) {
+                    class_of_the_child_of_a_type_zero_entry = rd8(child_unit.base, 6u);
+                }
+            }
+            let child_is_a_code_two_node_or_unreadable = class_of_the_child_of_a_type_zero_entry == NONE || class_of_the_child_of_a_type_zero_entry == 2u;
             w_judge(identity_invariant, is_zero64(rd64(n.base, o + 8u)) && is_zero64(rd64(n.base, o + 18u)) && is_zero64(rd64(n.base, o + 26u))
                 && child_is_a_code_two_node_or_unreadable);
             if (n.level <= 1u) {
@@ -3243,7 +3284,8 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
             && eq64(rd64(s.base, 53u), rd64(n.base, o + 18u)) && eq64(rd64(s.base, 61u), rd64(n.base, o + 26u))
             && eq64(child.birth_tree, rd64(n.base, o + 8u));
         w_judge(identity_invariant, identity_holds);
-        if (!w_visit(child.device0, child.slot0)) {
+        // 「走过」按读回那一份的落点认（F-KEY）
+        if (!w_visit_the_copy_read_back(child)) {
             w_merge_below(child.device0, child.slot0);
             continue;
         }
diff --git a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
index 8e1d86e..d62eee2 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -389,27 +389,6 @@ fn slot_to_bytes(slot: vec2<u32>) -> vec2<u32> {
 fn location_word(location: u32, k: u32) -> u32 {
     return tab(T_LOCATIONS, location * LOCATION_WORDS + k);
 }
-fn find_location(device: u32, offset: vec2<u32>) -> u32 {
-    var lo = 0u;
-    var hi = hdr(6u);
-    while (lo < hi) {
-        let mid = (lo + hi) >> 1u;
-        let d = location_word(mid, 0u);
-        let o = vec2<u32>(location_word(mid, 1u), location_word(mid, 2u));
-        if (d < device || (d == device && lt64(o, offset))) {
-            lo = mid + 1u;
-        } else {
-            hi = mid;
-        }
-    }
-    if (lo < hdr(6u) && location_word(lo, 0u) == device) {
-        let o = vec2<u32>(location_word(lo, 1u), location_word(lo, 2u));
-        if (eq64(o, offset)) {
-            return lo;
-        }
-    }
-    return NONE;
-}
 fn version_of_location(location: u32) -> u32 {
     let writes_first = location_word(location, 5u);
     let writes_count = location_word(location, 6u);
@@ -466,20 +445,68 @@ fn slice_of_version(version: u32, location: u32) -> Slice {
     s.c3 = tab(T_VERSIONS, version * VERSION_WORDS + 5u);
     return s;
 }
-// 读 (盘, 偏移, 长度)：起点是落点起点、长度不超过落点的才读得到
+// 读 (盘, 偏移, 长度)：起点是落点起点、长度不超过落点的才读得到；起点不是落点的，单元区里没写过的槽读出全零
+// （zero_unit_version_over_a_slot_nothing_wrote）。切片的字节与检查字照版本表那一条（slice_of_version 同一个读法），长度按落点或全零单元
 fn read_at(device: u32, offset: vec2<u32>, length: u32) -> Slice {
-    let location = find_location(device, offset);
-    if (location == NONE) {
-        return no_slice();
+    // 落点表按 (盘, 偏移) 升序二分，lo 落在排在 (device, offset) 之后的第一个落点上；lo − 1 是最后一个不排在它之后的落点，
+    // 也就是最后一次走「lo = mid + 1」的那个 mid：它在不在这块盘上、起点是不是恰为 offset，在那一次记下
+    var lo = 0u;
+    var hi = hdr(6u);
+    var before_is_on_this_device = false;
+    var before_starts_at_the_offset = false;
+    while (lo < hi) {
+        let mid = (lo + hi) >> 1u;
+        let d = location_word(mid, 0u);
+        let o = vec2<u32>(location_word(mid, 1u), location_word(mid, 2u));
+        if (d < device || (d == device && le64(o, offset))) {
+            lo = mid + 1u;
+            before_is_on_this_device = d == device;
+            before_starts_at_the_offset = before_is_on_this_device && eq64(o, offset);
+        } else {
+            hi = mid;
+        }
     }
-    if (length > location_word(location, 3u)) {
+    if (!before_is_on_this_device) {
         return no_slice();
     }
-    let version = version_of_location(location);
-    if (version == NONE) {
+    let location = lo - 1u;
+    var version = NONE;
+    var unit_length = 0u;
+    if (before_starts_at_the_offset) {
+        unit_length = location_word(location, 3u);
+        if (length <= unit_length) {
+            version = version_of_location(location);
+        }
+    } else {
+        version = zero_unit_version_over_a_slot_nothing_wrote(location, offset, length);
+        unit_length = DATA_UNIT_BYTES >> (version & 1u);
+    }
+    if (version == NONE || length > unit_length) {
         return no_slice();
     }
-    return slice_of_version(version, location);
+    var s: Slice;
+    s.version = version;
+    s.base = B[T_VERSION_BYTES] + tab(T_VERSIONS, version * VERSION_WORDS + 1u);
+    s.length = unit_length;
+    s.c0 = tab(T_VERSIONS, version * VERSION_WORDS + 2u);
+    s.c1 = tab(T_VERSIONS, version * VERSION_WORDS + 3u);
+    s.c2 = tab(T_VERSIONS, version * VERSION_WORDS + 4u);
+    s.c3 = tab(T_VERSIONS, version * VERSION_WORDS + 5u);
+    return s;
+}
+// 单元区里一个没有落点的 16 KiB 槽（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第三节第 2 条，攻方改法 F-零槽）：
+// 基线与枚举写表里每次写碰过的槽都有落点，没有落点就是这条流的每个崩溃状态里都没写过，CPU 的稀疏读者（memory_pool 的 PoolReader）读出全零。
+// before 是同一块盘上排在 offset 前面最近的落点：它的第 9、10 字是它之后、下一个落点之前在单元区里、整 16 KiB 在盘内、谁都没写过的槽 [第 9 字, 第 10 字)，
+// 第 11 字是从哪一槽起整 32 KiB 不在盘内（crash_judge_tables::UnitSlotsNothingWroteAfterALocation）。读的起点按槽对齐、读的几槽都在那一段里，
+// 交回版本表最前面那两条全零单元之一（crash_judge_tables::ZERO_UNIT_VERSIONS：第 0 条 32 KiB，盘尾只装得下 16 KiB 的槽交第 1 条 16 KiB），别的交 NONE
+fn zero_unit_version_over_a_slot_nothing_wrote(before: u32, offset: vec2<u32>, length: u32) -> u32 {
+    // 起点不按槽对齐、或槽号装不进 32 位（偏移 ≥ 2⁴⁶，盘装不下）的不认
+    let first_slot = (offset.x >> SLOT_SHIFT) | (offset.y << (32u - SLOT_SHIFT));
+    if (((offset.x & (NODE_BYTES - 1u)) | (offset.y >> (32u - SLOT_SHIFT))) != 0u || first_slot < location_word(before, 9u)
+        || location_word(before, 10u) < first_slot + ((length + NODE_BYTES - 1u) >> SLOT_SHIFT)) {
+        return NONE;
+    }
+    return select(0u, 1u, first_slot >= location_word(before, 11u));
 }
 fn read_location(location: u32) -> Slice {
     let version = version_of_location(location);
diff --git a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
index 85fa18e..ff0f322 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -1338,23 +1338,24 @@ mod tests {
         }
     }
 
-    /// 三个内核此刻展开之后的表达式数与离预算的余量（2026-10-06 在主工作区上现量的；扫描内核是第 10q 批合后的数）：
+    /// 三个内核此刻展开之后的表达式数与离预算的余量（2026-10-07 在删除线第 32 批的仓副本上用 naga =30.0.1 现量的：`read_at` 对单元区里没写过的槽读出全零、
+    /// 走读内核「走过」按读回那一份认与类型段 0 只认自证过的副本之后）：
     /// 改了哪个内核的源码，这里先红，量过新数、确认余量还够之后再改钉值，免得余量悄悄耗光、等上卡时才撞预算。
     #[test]
     fn each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget() {
         for (name, source, pinned_expressions, pinned_margin) in [
-            ("recovery", recovery_kernel_source(), 316_712, 83_288),
+            ("recovery", recovery_kernel_source(), 319_127, 80_873),
             (
                 "checker-scan",
                 crate::crash_judge_gpu::checker_scan_kernel_source(),
-                378_232,
-                21_768,
+                380_969,
+                19_031,
             ),
             (
                 "checker-walk",
                 crate::crash_judge_gpu::checker_walk_kernel_source(),
-                323_759,
-                76_241,
+                329_198,
+                70_802,
             ),
         ] {
             let size = super::kernel_inlined_size(&source);
diff --git a/crates/singlefs-checker-tier/src/crash_judge_tables.rs b/crates/singlefs-checker-tier/src/crash_judge_tables.rs
index e92b081..b5bedc4 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_tables.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_tables.rs
@@ -36,8 +36,9 @@ use crate::crash::{
 };
 use crate::crash_facts::{persisted_writes_of_state, SegmentFact, TornImageFact};
 
-/// 表的形态版本：落点、版本、检查字或发布表的布局变了就抬它。
-pub const JUDGE_TABLES_VERSION: u32 = 3;
+/// 表的形态版本：落点、版本、检查字或发布表的布局变了就抬它（4：版本表最前面两条全零单元、落点第 9–11 字记它之后没写过的槽，
+/// [`ZERO_UNIT_VERSIONS`]）。
+pub const JUDGE_TABLES_VERSION: u32 = 4;
 
 /// 一个落点最多被几次写盖住：GPU 内核里组合掩码是两个 u32（版本表里一个版本的第 0 字是低 32 位、第 7 字是高 32 位）。
 /// 错位复用那条历史把系统配置 0 号槽写了 34 次，32 位装不下。
@@ -168,6 +169,8 @@ pub struct JudgeTables {
     /// 池里的盘，按 `PoolReader::device_identities` 的次序。
     pub devices: Vec<u32>,
     pub device_bytes: u64,
+    /// 单元区起点（字节）：基线第一块盘系统配置槽 0 记的几何。单元区里没有落点的 16 KiB 槽每个崩溃状态都是全零（[`ZERO_UNIT_VERSIONS`]）。
+    pub unit_area_start_in_bytes: u64,
     pub judged_root_index: u32,
     pub writes: Vec<JudgeWrite>,
     /// 按 (盘, 偏移) 升序。
@@ -359,6 +362,7 @@ pub fn build_judge_tables(flow: &JudgeFlow<'_>) -> Result<JudgeTables, JudgeTabl
         state_count: tables.state_count,
         devices,
         device_bytes,
+        unit_area_start_in_bytes: geometry.unit_area_start_slot * slot_bytes(),
         judged_root_index: u32::try_from(flow.judged_root_index).expect("下标装得进 u32"),
         writes,
         locations,
@@ -1107,6 +1111,58 @@ impl JudgeTables {
             .find(|version| version.combination == combination)
     }
 
+    /// 每个落点之后那一段谁都没写过的槽（与 `locations` 同序，落点表第 9–11 字）。单元区里基线或枚举写表碰过的每个槽都有落点
+    /// （`enumerate_locations`），所以一个落点与同一块盘上下一个落点之间、单元区里的槽在每个崩溃状态里都没写过：
+    /// 单元槽落点从它下一槽起，别的落点从它末字节之后的头一个整槽起，都不早于单元区起点；到下一个落点起点所在的槽（不含）或盘尾为止。
+    ///
+    /// # Panics
+    /// 盘大到槽号装不进 u32（64 TiB 及以上）。
+    #[must_use]
+    pub fn unit_slots_nothing_wrote_after_each_location(
+        &self,
+    ) -> Vec<UnitSlotsNothingWroteAfterALocation> {
+        let slot_number = |bytes: u64| {
+            u32::try_from(bytes / slot_bytes()).expect("盘不到 64 TiB，槽号装得进 u32")
+        };
+        let unit_area_start_slot = self.unit_area_start_in_bytes / slot_bytes();
+        let device_end_slot = self.device_bytes / slot_bytes();
+        let first_slot_a_whole_data_unit_does_not_fit_from = self
+            .device_bytes
+            .checked_sub(DATA_UNIT_BYTES)
+            .map_or(0, |last_start| slot_number(last_start) + 1);
+        self.locations
+            .iter()
+            .enumerate()
+            .map(|(index, location)| {
+                let first_slot_after_the_location = match location.kind {
+                    JudgeLocationKind::UnitSlot => location.offset / slot_bytes() + 1,
+                    JudgeLocationKind::SystemConfiguration
+                    | JudgeLocationKind::RootSlot
+                    | JudgeLocationKind::JournalRecord => {
+                        (location.offset + u64::from(location.length)).div_ceil(slot_bytes())
+                    }
+                };
+                let first_slot = first_slot_after_the_location.max(unit_area_start_slot);
+                let end_slot = self
+                    .locations
+                    .get(index + 1)
+                    .filter(|next| next.device == location.device)
+                    .map_or(device_end_slot, |next| next.offset / slot_bytes())
+                    .min(device_end_slot);
+                let (first_slot, end_slot) = if first_slot < end_slot {
+                    (first_slot, end_slot)
+                } else {
+                    (0, 0)
+                };
+                UnitSlotsNothingWroteAfterALocation {
+                    first_slot: u32::try_from(first_slot).expect("盘不到 64 TiB，槽号装得进 u32"),
+                    end_slot: u32::try_from(end_slot).expect("盘不到 64 TiB，槽号装得进 u32"),
+                    first_slot_a_whole_data_unit_does_not_fit_from,
+                }
+            })
+            .collect()
+    }
+
     /// 全部表的摘要（连同形态版本）。
     #[must_use]
     pub fn digest(&self) -> [u8; DIGEST_BYTES] {
@@ -1212,13 +1268,26 @@ impl JudgeTables {
                 ]);
             }
         }
-        // 落点与版本
+        // 落点与版本。版本表最前面两条不是哪个落点的版本（[`ZERO_UNIT_VERSIONS`]）：第 0 条是一段 32 KiB 全零单元、第 1 条是同一段的前 16 KiB
+        // （盘尾只装得下 16 KiB 的槽），组合都是 0、字节起点都是字节表的第 0 字、检查字按单元槽算（[`version_checks`]）、第 6 字 NONE。
+        // 落点的第 9–11 字是它之后谁都没写过的槽（[`UnitSlotsNothingWroteAfterALocation`]）
         let mut locations = Vec::new();
         let mut location_writes = Vec::new();
         let mut versions = Vec::new();
         let mut version_bytes = Vec::new();
         let mut equal_masks = Vec::new();
-        for location in &self.locations {
+        let zero_unit = vec![0u8; usize::try_from(DATA_UNIT_BYTES).expect("32768")];
+        let zero_unit_lengths: [u64; ZERO_UNIT_VERSIONS] = [DATA_UNIT_BYTES, NODE_BYTES];
+        for zero_unit_bytes in zero_unit_lengths {
+            let checks = version_checks(
+                JudgeLocationKind::UnitSlot,
+                &zero_unit[..usize::try_from(zero_unit_bytes).expect("单元长装得进 usize")],
+            );
+            versions.extend([0, 0, checks[0], checks[1], checks[2], checks[3], NONE, 0]);
+        }
+        version_bytes.extend(bytes_to_words(&zero_unit));
+        let gaps = self.unit_slots_nothing_wrote_after_each_location();
+        for (location_index, location) in self.locations.iter().enumerate() {
             let (offset_lo, offset_hi) = split(location.offset);
             locations.extend([
                 location.device,
@@ -1230,9 +1299,9 @@ impl JudgeTables {
                 index(location.writes.len()),
                 index(versions.len() / VERSION_WORDS),
                 index(location.versions.len()),
-                0,
-                0,
-                0,
+                gaps[location_index].first_slot,
+                gaps[location_index].end_slot,
+                gaps[location_index].first_slot_a_whole_data_unit_does_not_fit_from,
             ]);
             location_writes.extend(location.writes.iter().copied());
             for version in &location.versions {
@@ -1338,6 +1407,21 @@ impl JudgeTables {
 
 /// 表里「没有」的写法。
 pub const NONE: u32 = u32::MAX;
+/// 版本表最前面几条是全零单元，不是哪个落点的版本：第 0 条 32 KiB、第 1 条 16 KiB（盘尾只装得下 16 KiB 的槽；布局写在 [`JudgeTables::words`]
+/// 填它们的地方）。GPU 读者 `read_at` 读单元区里一个没有落点的槽时交回它们（判决 `research/prompts/m3-batch29-code-r1-main-verification.md`
+/// 第三节第 2 条，攻方改法 F-零槽）：那一槽基线与枚举写表里哪一次写都没碰过，CPU 稀疏读者在每个崩溃状态里读出的都是零。
+/// 落点的第 9–11 字说它之后哪几槽是这样的槽（[`UnitSlotsNothingWroteAfterALocation`]）。
+pub const ZERO_UNIT_VERSIONS: usize = 2;
+
+/// 一个落点之后、同一块盘上下一个落点之前那一段谁都没写过的槽（落点表第 9–11 字）：在单元区里、整 16 KiB 在盘内，
+/// 槽号 [`Self::first_slot`, `Self::end_slot`)（没有这样的槽时两个都是 0）；`Self::first_slot_a_whole_data_unit_does_not_fit_from`
+/// 是这块盘上从哪一槽起整 32 KiB 不在盘内（GPU 从那一槽起交 16 KiB 那条全零单元，同真落点在盘尾只有 16 KiB）。槽号都装得进 u32（盘不到 64 TiB）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct UnitSlotsNothingWroteAfterALocation {
+    pub first_slot: u32,
+    pub end_slot: u32,
+    pub first_slot_a_whole_data_unit_does_not_fit_from: u32,
+}
 /// 写表每条几个 u32。
 pub const WRITE_WORDS: usize = 12;
 /// 落点表每条几个 u32。
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 1344d3f..fbd0a5e 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -45,6 +45,9 @@
 //! 另一份自证成码 2 时类型段 1 与类型段 0 两形（容器树两处 F-W）各一条流，都整份比；inode 树根条目类型段 0、另一份自证成码 2 一条钉 GPU 记判不了
 //! （类型段 0 的内部节点）、回落 CPU 之后同判；候选根自己那张实例表链 257 片、最新根的链途经它的一片一条钉 GPU 扫描内核记判不了（站点 37 / 38）、
 //! 回落 CPU 之后同判。
+//! 第 32 批（删除线）：墓碑容器带记录宽违例、墓碑树根条目第 0 条位置条目指着先走过的 inode 树根（F-KEY：「走过」按读回那一份的落点认）、
+//! 类型段 0 的条目两份副本一份只按前 16 KiB 对得上而头里是码 3、一份是码 2（F-ID-b：只认自证过的副本）、类型段 7 的条目第 1 条位置条目指着流里没写过的槽、
+//! 校验和是全零 16 KiB 的 CRC（F-零槽：GPU 读者对没写过的槽读出全零）各一条流，都整份比。
 //! 第 35 批：由记录造出来的两类版本取带末条标志那条记录的新根段（三条记录的发布第一条写成上一版的根：下一次挂载那一版叠两条记录罩住同一个槽、
 //! 挂载之后由记录施加出来的那一版各一条），树表指针整条全零照 core 跟过去（根与它的记录、由记录施加出来的那一版的记录各一条），都整份比。
 //! 第 36 批：下一次挂载先施加的那一版照 I-4.1 那一份重放施加的第一次发布认（改法 R1）——三条记录的发布首条没落（断号）、
@@ -59,7 +62,7 @@
 //! 卡经这个进程的卡池取（`gpu_test_start::gpu_card_pool_or_report_not_run`：每张卡一个设备、内核每张卡编一次、一张卡一本显存账，
 //! 卡与额度照 `GPU_CARDS_BY_PRIORITY` 里这台机器那几项，没配就是本机能用的卡）：每条用例建好表之后按它的判器实际要的显存借，
 //! 照卡的先后装满前一张才用下一张，放不下就等别的用例还，比每张卡的账都大判红（几十条用例并行不再各按空闲显存的 3/4 互抢）；
-//! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（默认 8）、显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（只对量速度那条用例作数；设了它，经卡池取卡的对拍用例判红）、
+//! 展开上限 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`（默认 8）、显存额度 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`（量速度那条用例当那张卡的额度；卡池里的读法在 `gpu_unit_checks::gpu_test_start::CARD_QUOTA_ENVIRONMENT_VARIABLE`）、
 //! CPU 判器线程数 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`（没设取本机可用并行度）：设了而认不出（线程数 0 也算）就 panic，不退回没设时的那一档。
 #![cfg(all(feature = "verdict-store", feature = "gpu"))]
 
@@ -107,7 +110,7 @@ use singlefs_checker_tier::crash_judge_gpu::{
 use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
 use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
     first_usable_card_or_report_not_run, gpu_card_pool_or_report_not_run,
-    usable_cards_or_report_not_run, GpuCardPool,
+    usable_cards_or_report_not_run, GpuCardPool, CARD_QUOTA_ENVIRONMENT_VARIABLE,
 };
 use singlefs_checker_tier::gpu_unit_checks::GpuCard;
 use singlefs_checker_tier::verdict_store::{JudgeVersion, VerdictStore};
@@ -137,7 +140,6 @@ use singlefs_harness::SharedStream;
 
 const EXPAND_UP_TO_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO";
 const DEFAULT_EXPAND_UP_TO: usize = 8;
-const CARD_QUOTA_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES";
 const CPU_THREADS_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_CPU_THREADS";
 const SECOND_FILE_BYTES: usize = 4100;
 const THIRD_FILE_BYTES: usize = 2500;
@@ -219,25 +221,6 @@ fn the_whole_emptiest_card_or_report_not_run(test_name: &str) -> Option<GpuCard>
     Some(card)
 }
 
-/// 对拍用例取这个进程的卡池（`gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run`：进程里只判一次有没有卡，
-/// 卡照 `GPU_CARDS_BY_PRIORITY` 与机器角色挑）；建好表、知道要多少显存之后再借（[`GpuCardPool::lease_card_waiting_for_room`]）。
-/// 本机没有 NVIDIA 独显适配器时打「本次未跑」那一行、交回 `None`，用例当场返回；有卡而用不上判红。
-///
-/// # Panics
-/// 设了 `SINGLEFS_CRASH_JUDGE_CARD_QUOTA_MEBIBYTES`：有了卡池之后它对借卡当什么（每张卡的账的上限，还是每条用例的上限）还没定，
-/// 不悄悄挑一种；卡的额度照 `GPU_CARDS_BY_PRIORITY` 给，这个变量只对量速度那条用例作数。
-fn the_card_pool_or_report_not_run(test_name: &str) -> Option<&'static GpuCardPool> {
-    let quota = card_quota_mebibytes_from_the_value_of_the_variable(
-        process_environment_variable(CARD_QUOTA_ENVIRONMENT_VARIABLE).as_deref(),
-    );
-    assert!(
-        quota.is_none(),
-        "{test_name}：设了 {CARD_QUOTA_ENVIRONMENT_VARIABLE}={quota:?}。有了卡池之后这个变量对借卡当什么还没定（交主 agent 定），\
-         对拍用例不认它：卡的额度写进 GPU_CARDS_BY_PRIORITY；它只对 gpu_judge_throughput_by_states_per_dispatch 作数"
-    );
-    gpu_card_pool_or_report_not_run(test_name)
-}
-
 /// 一条草稿缓冲装几个状态，按卡池里绑定上限最小的那张卡估（借卡之前还不知道借到哪张）。
 fn states_of_one_scratch_lane_on_every_card_of(cards: &GpuCardPool) -> usize {
     states_per_scratch_lane(cards.smallest_max_storage_buffer_binding_size())
@@ -1873,7 +1856,7 @@ fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
     flow: &RecordedFlow,
     invariant: &'static str,
 ) {
-    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
         return;
     };
     let comparison = compare_on_every_state(cards, flow, expand_up_to());
@@ -2225,7 +2208,7 @@ fn compare_on_every_state_of_these_segments(
 
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows() {
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_judge_on_every_state_of_three_flows",
     ) else {
         return;
@@ -2358,7 +2341,7 @@ fn the_gpu_judge_tells_apart_two_versions_of_a_root_slot_that_differ_only_past_t
             .map(|version| version.combination)
             .collect::<Vec<_>>()
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_tells_apart_two_versions_of_a_root_slot_that_differ_only_past_the_thirty_second_write",
     ) else {
         return;
@@ -2452,7 +2435,7 @@ fn resealed_publish_record_names_units_that_match_on_disk() {
 #[test]
 fn verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several() {
     const EXPAND_UP_TO: usize = 16;
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "verdicts_are_the_same_whether_one_dispatch_spans_one_scratch_lane_or_several",
     ) else {
         return;
@@ -2566,7 +2549,7 @@ fn a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both
             1,
         ),
     };
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "a_planted_violated_bit_past_the_sixty_fourth_reaches_the_verdict_through_both_checker_kernels",
     ) else {
         return;
@@ -2736,7 +2719,7 @@ fn the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest
     const EXPAND_UP_TO: usize = 16;
     const BLOCKS_BEFORE_THE_STOP: usize = 5;
     const THIS_FILE: &str = "crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs";
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest",
     ) else {
         return;
@@ -3249,7 +3232,7 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_refused_value_wins_over_a_wrong_safe
     let flow = record_new_pool_with_a_refused_slot_spacing_and_a_safety_factor_of_two(
         "crash-judge-gpu-i713-over-i81",
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_refused_value_wins_over_a_wrong_safety_factor_in_the_same_slot",
     ) else {
         return;
@@ -3330,7 +3313,7 @@ fn the_gpu_judge_agrees_with_the_cpu_when_a_version_references_a_unit_born_no_la
 fn the_gpu_judge_agrees_with_the_cpu_that_a_rollback_publish_inheriting_its_user_visible_trees_whole_keeps_the_appearance_rule(
 ) {
     let flow = record_fixed_script_through_rollback("crash-judge-gpu-i35-holds");
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_rollback_publish_inheriting_its_user_visible_trees_whole_keeps_the_appearance_rule",
     ) else {
         return;
@@ -3362,7 +3345,7 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_unmount_marker_sets_ano
     let flow = record_new_pool_with_a_last_record_whose_unmount_marker_sets_another_bit(
         "crash-judge-gpu-rr1",
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_record_whose_unmount_marker_sets_another_bit_is_not_applied",
     ) else {
         return;
@@ -3436,7 +3419,7 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_last_record_carries_another_unmoun
 /// 新池新建文件那条流的每个状态（不种假位）上，违例位在池级 checker 条数之外的那些位全是 0、判得了——高 64 位那两个字只要有一个没写，这条红。
 #[test]
 fn the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count() {
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_checker_kernels_write_every_violated_word_and_leave_no_bit_past_the_invariant_count",
     ) else {
         return;
@@ -3624,7 +3607,7 @@ fn the_gpu_judge_agrees_with_the_cpu_when_only_roots_outside_the_candidate_set_r
     let flow = record_rollback_with_another_birth_txg_recorded_only_outside_the_candidate_set(
         "crash-judge-gpu-i928-outside-candidates",
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_when_only_roots_outside_the_candidate_set_record_another_birth_txg",
     ) else {
         return;
@@ -3685,7 +3668,7 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_root_whose_tree_table_cannot_be_read
     let flow = record_rollback_with_the_tree_table_of_the_overwrite_before_it_unreadable(
         "crash-judge-gpu-i928-unreadable-tree-table",
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_root_whose_tree_table_cannot_be_read_does_not_redden_the_every_readable_root_birth_invariant",
     ) else {
         return;
@@ -3806,7 +3789,7 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_data_unit_claims_its_second_slot_and
     let flow = record_new_pool_with_an_overwrite_forging_an_index_node_header_on_the_second_slot_of_its_data_unit(
         "crash-judge-gpu-i78-claim",
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_data_unit_claims_its_second_slot_and_a_header_forged_there_is_not_counted",
     ) else {
         return;
@@ -3870,7 +3853,7 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_with_no_earlier_root_ins
         "new-pool-floor-raise-one-revolution-past-every-earlier-root",
         UnmountMarkerOfTheAppendedRoot::NotWritten,
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_judged",
     ) else {
         return;
@@ -3904,7 +3887,7 @@ fn the_gpu_judge_agrees_with_the_cpu_that_an_unmount_marked_floor_raise_with_no_
         "new-pool-unmount-marked-floor-raise-one-revolution-past-every-earlier-root",
         UnmountMarkerOfTheAppendedRoot::Written,
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_an_unmount_marked_floor_raise_with_no_earlier_root_inside_the_one_revolution_windows_is_not_reddened",
     ) else {
         return;
@@ -3974,7 +3957,7 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_takes_no_root_outside_th
     let flow = record_rollback_with_a_floor_raise_whose_window_anchored_at_the_newest_root_drops_an_older_root(
         "crash-judge-gpu-i79-image-window",
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_floor_raise_takes_no_root_outside_the_window_anchored_at_the_newest_root_of_the_image",
     ) else {
         return;
@@ -4138,7 +4121,7 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish
 ) {
     let flow =
         record_rollback_with_the_lineage_root_naming_another_tree("crash-judge-gpu-lineage-root");
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_root_of_a_rollback_publish_names_another_tree",
     ) else {
         return;
@@ -4218,7 +4201,7 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entr
     let flow = record_new_pool_with_the_lineage_and_logged_operations_kinds_swapped(
         "crash-judge-gpu-i916-issuing-order",
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_when_the_lineage_and_logged_operations_entries_break_the_issuing_order",
     ) else {
         return;
@@ -5500,7 +5483,7 @@ fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress()
         "crash-judge-gpu-deletion-in-progress",
         "deletion-through-the-first-batch",
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress",
     ) else {
         return;
@@ -5537,7 +5520,7 @@ fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_la
         "crash-judge-gpu-deletion-through-the-last-batch",
         "deletion-through-the-last-batch",
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch",
     ) else {
         return;
@@ -5659,7 +5642,7 @@ fn the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unex
             variant.missing_write_index
         );
     }
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unexpanded_segment_of_a_deletion_in_progress",
     ) else {
         return;
@@ -5890,7 +5873,7 @@ fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
     flow: &RecordedFlow,
     invariants: &[&'static str],
 ) {
-    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
         return;
     };
     let comparison = compare_on_every_state(cards, flow, expand_up_to());
@@ -7435,7 +7418,7 @@ fn the_gpu_judge_leaves_to_the_cpu_an_instance_table_chain_longer_than_the_pages
         record_an_abandoned_root_whose_own_instance_table_is_a_chain_longer_than_the_gpu_walks(
             "crash-judge-gpu-i312-abandoned-root-instance-table-of-257-pages",
         );
-    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
         return;
     };
     let comparison = compare_on_every_state(cards, &flow, expand_up_to());
@@ -7667,7 +7650,7 @@ fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_an_unlink_rolled_back_and
     let flow = record_new_inode_unlinked_rolled_back_and_unlinked_again(
         "crash-judge-gpu-unlink-rolled-back-and-unlinked-again",
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_an_unlink_rolled_back_and_unlinked_again",
     ) else {
         return;
@@ -7709,7 +7692,7 @@ fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_the_identity_wit
     flow: &RecordedFlow,
     identity_invariant: &'static str,
 ) {
-    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
         return;
     };
     let comparison = compare_on_every_state(cards, flow, expand_up_to());
@@ -7818,7 +7801,7 @@ fn the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_objec
             "最新根的实例表两份都翻坏、候选集滤不掉被抛弃的 C：I-9.29 要报部分没判，实际 {other_verdict:?}"
         ),
     }
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back",
     ) else {
         return;
@@ -7883,7 +7866,7 @@ fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that
             "{change:?}：整条落盘的镜像上 {invariant} 不是违例，实际 {verdict:?}"
         );
     }
-    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
         return;
     };
     let comparison = compare_on_every_state(cards, &flow, expand_up_to());
@@ -7993,7 +7976,7 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_second_instance_table_page_whose_poi
         "流 {}：主走读按读回那一份认「走过」、走到第 1 片（末片），候选集里没有实例 2 那一行，I-9.29 红，实际 {verdict:?}",
         flow.name
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_second_instance_table_page_whose_pointer_on_device_zero_names_the_first_page_is_walked_from_the_copy_read_back",
     ) else {
         return;
@@ -8037,7 +8020,7 @@ fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_witho
         record_new_pool_with_the_inode_root_entry_of_type_zero_and_no_identity_over_a_leaf_container_whose_first_copy_is_stale(
             "crash-judge-gpu-i92-inode-root-entry-type-zero-first-leaf-copy-stale",
         );
-    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
         return;
     };
     let comparison = compare_on_every_state(cards, &flow, expand_up_to());
@@ -8489,7 +8472,7 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_is_a_
         "new-pool-inode-leaf-second-copy-stale-node",
         ChecksumOfTheSecondLocation::LeftAsTheLeafChecksum,
     );
-    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
         return;
     };
     let comparison = compare_on_every_state(cards, &flow, expand_up_to());
@@ -8556,7 +8539,7 @@ fn the_gpu_judge_agrees_with_the_cpu_when_a_leaf_copy_of_the_entry_type_carries_
     let flow = record_new_pool_with_blocks_off_in_the_inode_leaf_and_its_second_copy_holding_the_inode_root_bytes(
         "crash-judge-gpu-i92-inode-leaf-blocks-off-second-copy-verifying-as-a-node",
     );
-    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
         return;
     };
     let comparison = compare_on_every_state(cards, &flow, expand_up_to());
@@ -8642,7 +8625,7 @@ fn assert_the_gpu_judge_agrees_with_the_cpu_on_an_instance_table_pointer_whose_b
     let flow = record_new_pool_with_the_instance_table_pointer_birth_identity_off_by_one_in(
         tag, name, field,
     );
-    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
         return;
     };
     let comparison = compare_on_every_state(cards, &flow, expand_up_to());
@@ -8726,7 +8709,7 @@ fn assert_the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_
             flow.name
         ),
     }
-    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
         return;
     };
     let comparison = compare_on_every_state(cards, &flow, expand_up_to());
@@ -8805,7 +8788,7 @@ fn the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_chain_of_as_many
             "crash-judge-gpu-i929-newest-instance-table-of-256-pages",
             InstanceTableChainPages::AsManyAsTheGpuWalks,
         );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_chain_of_as_many_pages_as_the_gpu_walks",
     ) else {
         return;
@@ -8840,7 +8823,7 @@ fn the_gpu_judge_leaves_to_the_cpu_a_newest_instance_table_chain_longer_than_the
             "crash-judge-gpu-i929-newest-instance-table-of-257-pages",
             InstanceTableChainPages::OneMoreThanTheGpuWalks,
         );
-    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
         return;
     };
     let comparison = compare_on_every_state(cards, &flow, expand_up_to());
@@ -9258,7 +9241,7 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_version_applied_only_by_its_records_
         "three-unit-file-applied-only-by-its-records-stale-first-record",
         &write_the_root_of_the_first_file_into_the_first_record_of_the_three_unit_publish,
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_version_applied_only_by_its_records_is_walked_from_its_last_record_when_the_first_carries_the_previous_root",
     ) else {
         return;
@@ -9299,7 +9282,7 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_pointer_of_the_root_and
     let flow = record_new_pool_with_the_tree_table_pointer_of_the_publish_written_all_zero(
         "crash-judge-gpu-all-zero-tree-table-pointer",
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_pointer_of_the_root_and_of_its_record_is_all_zero",
     ) else {
         return;
@@ -9338,7 +9321,7 @@ fn the_gpu_judge_agrees_with_the_cpu_when_the_records_of_a_version_applied_only_
             [JOURNAL_NEW_ROOT_SEGMENT_OFFSET..JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NODE_POINTER_BYTES]
             .fill(0);
     });
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_when_the_records_of_a_version_applied_only_by_them_carry_an_all_zero_tree_table_pointer",
     ) else {
         return;
@@ -9458,7 +9441,7 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_l
         InvariantVerdict::Holds,
         "首条两份都没落、后两条落了：core 断号不施加 txg 4，CPU 不认它，I-5.4 在 txg 3 那一版上判过而成立"
     );
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first",
     ) else {
         return;
@@ -9644,7 +9627,7 @@ fn the_gpu_judge_agrees_with_the_cpu_that_a_replay_stopped_by_a_named_unit_of_a_
             panic!("前缀停在 C 的点名单元上时 I-4.1 该报不适用，实际 {other_verdict:?}")
         }
     }
-    let Some(cards) = the_card_pool_or_report_not_run(
+    let Some(cards) = gpu_card_pool_or_report_not_run(
         "the_gpu_judge_agrees_with_the_cpu_that_a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first",
     ) else {
         return;
@@ -9926,7 +9909,7 @@ fn assert_the_gpu_judge_leaves_to_the_cpu_only_states_undecidable_for(
     reasons_left_to_the_cpu: &[u32],
     reasons_one_state_must_carry: &[u32],
 ) {
-    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
+    let Some(cards) = gpu_card_pool_or_report_not_run(test_name) else {
         return;
     };
     let comparison = compare_on_every_state(cards, flow, expand_up_to());
@@ -10291,3 +10274,266 @@ fn the_gpu_judge_leaves_to_the_cpu_a_candidate_instance_table_chain_longer_than_
         ],
     );
 }
+
+// ─── 第 32 批（删除线）：「走过」按读回那一份的落点认（F-KEY）、类型段 0 认孩子的类只认自证过的副本（F-ID-b）、单元区里没写过的槽读出全零（F-零槽） ───
+
+/// 位置条目里槽号的起点与字节数（盘 4 + 槽 6 + 整单元校验和 4）。
+const LOCATION_ENTRY_SLOT_OFFSET: usize = 4;
+const LOCATION_ENTRY_SLOT_BYTES: usize = 6;
+
+/// 码 2 节点第一条条目子指针里第 `location` 条位置条目（[`POINTER_LOCATION_ENTRY_OFFSETS`] 之一）的整单元校验和写成 `checksum`（盘与槽不动）。
+fn set_the_checksum_of_a_location_in_the_first_entry(
+    bytes: &mut [u8],
+    location: usize,
+    checksum: u32,
+) {
+    let field = first_entry_of_a_container_number_node()
+        + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
+        + location
+        + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET;
+    bytes[field..field + 4].copy_from_slice(&checksum.to_le_bytes());
+}
+
+/// 码 2 节点第一条条目子指针里第 `location` 条位置条目的槽写成 `slot`、整单元校验和写成 `checksum`（盘不动）。
+fn set_the_slot_and_checksum_of_a_location_in_the_first_entry(
+    bytes: &mut [u8],
+    location: usize,
+    slot: u64,
+    checksum: u32,
+) {
+    let field = first_entry_of_a_container_number_node()
+        + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
+        + location;
+    bytes[field + LOCATION_ENTRY_SLOT_OFFSET
+        ..field + LOCATION_ENTRY_SLOT_OFFSET + LOCATION_ENTRY_SLOT_BYTES]
+        .copy_from_slice(&slot.to_le_bytes()[..LOCATION_ENTRY_SLOT_BYTES]);
+    set_the_checksum_of_a_location_in_the_first_entry(bytes, location, checksum);
+}
+
+/// 翻进校验和里的几位：那一份按哪种长度都对不上。
+const CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY: u32 = 0x5a5a_5a5a;
+
+/// 两份副本里盘序小、盘序大的那一份（写下标）。
+fn copies_in_device_order(flow: &RecordedFlow, copies: &[usize]) -> (usize, usize) {
+    if flow.writes[copies[0]].device.0 < flow.writes[copies[1]].device.0 {
+        (copies[0], copies[1])
+    } else {
+        (copies[1], copies[0])
+    }
+}
+
+/// F-KEY（判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K2 观察 O3 那一行、第三节第 2 条；harness 坏镜像
+/// `a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before`
+/// 位置条目 0 那一形，录成流）：删除中间态的流上墓碑容器两份的记录宽 40 → 39（重封、沿引用链补到根），再把墓碑树根第一条条目子指针第 0 条位置条目的槽
+/// 改成流里最后写的 inode 树根的槽（这一遍在墓碑树之前走过）、整单元校验和写成那个根的 CRC 翻几位（这一份对不上），重封到根。
+/// 读者读回的是盘 1 那一份带违例的容器。
+fn record_deletion_with_a_wrong_record_width_in_the_tombstone_container_whose_first_location_names_the_inode_root(
+    tag: &str,
+) -> RecordedFlow {
+    let name = "deletion-tombstone-container-record-width-first-location-naming-the-inode-root";
+    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
+    let container_copies = last_unit_copies(&flow, name, &is_a_tombstone_container, false);
+    corrupt_units_and_reseal_up_to_the_root(
+        &mut flow,
+        &container_copies,
+        &set_the_record_width_of_a_tombstone_container_one_below,
+    );
+    let inode_root_copies = last_unit_copies(
+        &flow,
+        name,
+        &is_the_inode_tree_root_over_leaf_containers,
+        false,
+    );
+    let inode_root_slot = flow.writes[inode_root_copies[0]].offset.0 / SLOT_BYTES;
+    let wrong_checksum_of_the_inode_root = crc32c(
+        flow.writes[inode_root_copies[0]]
+            .bytes()
+            .expect("单元写带字节"),
+    ) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
+    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
+    assert_the_first_entry_names_the_unit_of(
+        &flow,
+        root_copies[0],
+        &container_copies,
+        "墓碑树根第一条条目指着流里最后写的那片墓碑容器",
+    );
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
+        set_the_slot_and_checksum_of_a_location_in_the_first_entry(
+            bytes,
+            POINTER_LOCATION_ENTRY_OFFSETS[0],
+            inode_root_slot,
+            wrong_checksum_of_the_inode_root,
+        );
+    });
+    flow
+}
+
+/// 第 32 批 F-KEY：CPU 池级 checker「走过」按读回那一份（盘 1）的落点认，那片容器照判，红 I-1.7（另有 I-2.1 等：位置条目 0 那一份对不上、
+/// 按两槽登记进了 inode 树根的槽）；GPU 走读内核 `w_visit_the_copy_read_back` 同改，每个状态逐位相同（按第 0 条位置条目认的话两边都不判那片容器）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_first_location_of_a_tombstone_container_with_a_wrong_record_width_names_a_node_walked_before(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_first_location_of_a_tombstone_container_with_a_wrong_record_width_names_a_node_walked_before",
+        &record_deletion_with_a_wrong_record_width_in_the_tombstone_container_whose_first_location_names_the_inode_root(
+            "crash-judge-gpu-i17-tombstone-container-record-width-first-location-naming-the-inode-root",
+        ),
+        &["I-1.7", "I-2.1"],
+    );
+}
+
+/// F-ID-b（判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K2 N1 盘序翻面 A3 那一行，用户 2026-10-07 定；harness 坏镜像
+/// `a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device`
+/// 位置条目 0 是 `c16` 那一形，录成流）：删除中间态的流上墓碑树根第一条条目类型段 1 → 0、身份三段清零，子指针第 0 条位置条目的整单元校验和换成
+/// 墓碑容器前 16 KiB 的 CRC（盘 0 那一份头里是码 3、按 32 KiB 自证不过），第 1 条换成墓碑树根原样字节的 CRC，重封到根；盘 1 那份容器的前 16 KiB
+/// 换成墓碑树根（码 2）的原样字节（按 16 KiB 自证得过）。
+fn record_deletion_with_the_tombstone_root_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_the_root_bytes(
+    tag: &str,
+) -> RecordedFlow {
+    let name = "deletion-tombstone-root-entry-type-zero-container-checked-at-sixteen-kibibytes-beside-the-root-bytes";
+    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
+    let container_copies = last_unit_copies(&flow, name, &is_a_tombstone_container, false);
+    let (first_container_copy, second_container_copy) =
+        copies_in_device_order(&flow, &container_copies);
+    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
+    assert_the_first_entry_names_the_unit_of(
+        &flow,
+        root_copies[0],
+        &container_copies,
+        "墓碑树根第一条条目指着流里最后写的那片墓碑容器",
+    );
+    let node_bytes = flow.writes[root_copies[0]]
+        .bytes()
+        .expect("单元写带字节")
+        .to_vec();
+    assert_eq!(
+        unit_class(&node_bytes),
+        2,
+        "换进去的那 16 KiB 头里的类是码 2"
+    );
+    let container = flow.writes[first_container_copy]
+        .bytes()
+        .expect("单元写带字节")
+        .to_vec();
+    assert_eq!(unit_class(&container), 3, "墓碑容器是码 3");
+    let checksum_of_the_first_sixteen_kibibytes = crc32c(&container[..node_bytes.len()]);
+    let checksum_of_the_node_bytes = crc32c(&node_bytes);
+    assert_eq!(
+        device_and_slot_of_a_pointer_location(
+            &node_bytes[first_entry_of_a_container_number_node()
+                + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET..],
+            POINTER_LOCATION_ENTRY_OFFSETS[1],
+        )
+        .0,
+        flow.writes[second_container_copy].device.0,
+        "第 1 条位置条目是盘序大的那块盘"
+    );
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
+        set_the_type_of_the_first_entry(bytes, 0);
+        clear_the_identity_of_the_first_entry_but_its_type(bytes);
+        set_the_checksum_of_a_location_in_the_first_entry(
+            bytes,
+            POINTER_LOCATION_ENTRY_OFFSETS[0],
+            checksum_of_the_first_sixteen_kibibytes,
+        );
+        set_the_checksum_of_a_location_in_the_first_entry(
+            bytes,
+            POINTER_LOCATION_ENTRY_OFFSETS[1],
+            checksum_of_the_node_bytes,
+        );
+    });
+    let WrittenContents::Bytes(second_container) = &mut flow.writes[second_container_copy].contents
+    else {
+        panic!("单元写是普通写");
+    };
+    second_container[..node_bytes.len()].copy_from_slice(&node_bytes);
+    flow
+}
+
+/// 第 32 批 F-ID-b：CPU 池级 checker 认「孩子是码 2」只看按自己那一类长度自证过的副本（盘 1 那一份码 2），I-9.23 不红；F-L0 按头里的码 3 比盘 0 那一份
+/// 对不上，红 I-2.1、I-4.8、I-7.2、I-7.4；GPU 走读内核 `w_class_of_the_first_copy_verifying_at_the_length_of_its_class` 同改，每个状态逐位相同
+/// （取第一份按 16 KiB 对得上的话两边都认成码 3、I-9.23 红）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_checked_at_sixteen_kibibytes_beside_a_node(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_checked_at_sixteen_kibibytes_beside_a_node",
+        &record_deletion_with_the_tombstone_root_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_the_root_bytes(
+            "crash-judge-gpu-i923-tombstone-root-entry-type-zero-sixteen-kibibyte-copy-beside-a-node",
+        ),
+        &["I-2.1", "I-7.2"],
+    );
+}
+
+/// 流里谁都没写过、基镜像里全零的一段：两块盘上这一槽起 32 KiB（写之前由 [`assert_nothing_in_the_flow_wrote_the_slot`] 核），在单元区里、
+/// 离 [`SLOT_OF_THE_SECOND_PAGE_OF_THE_NEWEST_INSTANCE_TABLE`] 那几条流用的那一截也远。
+const SLOT_NOTHING_IN_THE_FLOW_WROTE: u64 = 190_000;
+
+/// `slot` 起 32 KiB 在两块盘上流里没写过、基镜像里全零。
+fn assert_nothing_in_the_flow_wrote_the_slot(flow: &RecordedFlow, slot: u64) {
+    let start = slot * SLOT_BYTES;
+    let end = start + 2 * SLOT_BYTES;
+    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+        assert!(
+            flow.writes.iter().all(|write| write.device != device
+                || write.offset.0 + write.length_in_bytes() <= start
+                || write.offset.0 >= end),
+            "槽 {slot} 起 32 KiB 在盘 {} 上流里没写过",
+            device.0
+        );
+        assert!(
+            singlefs_core::recovery::PoolReader::read(
+                &flow.base,
+                device,
+                singlefs_core::address::DeviceOffsetInBytes(start),
+                usize::try_from(end - start).expect("32768"),
+            )
+            .expect("基镜像里读得到这一截")
+            .iter()
+            .all(|byte| *byte == 0),
+            "槽 {slot} 起 32 KiB 在盘 {} 的基镜像里全零",
+            device.0
+        );
+    }
+}
+
+/// F-零槽（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第二节 K2 L1 位置条目指向没写过的槽那一行、第三节第 2 条；攻方
+/// `research/prompts/m3-batch29-code-r1-opus-output.md` 第三节 `tomb-L1-loc1zero16-type7` 那一形，录成流）：删除中间态的流上墓碑树根第一条条目
+/// 类型段 1 → 7、子指针第 1 条位置条目的槽改成 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`]、整单元校验和写成 16 KiB 全零的 CRC，重封到根。
+fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_a_slot_nothing_wrote(
+    tag: &str,
+) -> RecordedFlow {
+    let name =
+        "deletion-tombstone-root-entry-type-seven-second-location-naming-a-slot-nothing-wrote";
+    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
+    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
+    let checksum_of_sixteen_zero_kibibytes =
+        crc32c(&vec![0u8; usize::try_from(SLOT_BYTES).expect("16384")]);
+    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
+        set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
+        set_the_slot_and_checksum_of_a_location_in_the_first_entry(
+            bytes,
+            POINTER_LOCATION_ENTRY_OFFSETS[1],
+            SLOT_NOTHING_IN_THE_FLOW_WROTE,
+            checksum_of_sixteen_zero_kibibytes,
+        );
+    });
+    flow
+}
+
+/// 第 32 批 F-零槽：CPU 的稀疏读者对没写过的槽读出全零，头里类标签 0 不登记、16 KiB 与 32 KiB 两种长度都比，全零 16 KiB 的 CRC 对得上，
+/// 那一份 I-2.1 成立；孩子按层级 1 的形状登记两槽引用，红 I-9.23、I-3.1、I-3.11（攻方 `RESULT … form=tomb-L1-loc1zero16-type7 red=[I-3.1,I-3.11,I-9.23]`）。
+/// GPU 读者 `read_at` 在单元区里找不到落点、整段在盘内时交回全零单元（`crash_judge_common.wgsl` 的 `zero_unit_version_over_a_slot_nothing_wrote`），
+/// 每个状态逐位相同（改之前那个槽读不出，GPU 多红 I-2.1、I-4.8、I-7.4）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_with_the_checksum_of_zeros(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_with_the_checksum_of_zeros",
+        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_a_slot_nothing_wrote(
+            "crash-judge-gpu-i21-tombstone-root-entry-type-seven-second-location-naming-a-slot-nothing-wrote",
+        ),
+        &["I-9.23", "I-3.1"],
+    );
+}
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index baf45b3..50b1bca 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -12105,6 +12105,15 @@ fn assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_h
                 "码 2 那一份在位置条目 {location_index}：槽数按类型段那一类登记，不连带 {invariant_a_wrong_slot_count_drags_in}：{verdicts:?}"
             );
         }
+        // G4（判决 `research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M3 缺口 G4 那一行、第三节第 4 条）：码 2 那一份按类型段那一类
+        // （码 3）的 32 KiB 读回比校验和，对不上，I-2.1 红，I-4.8、I-7.4 跟着（`.claude/kb/invariants.md` I-9.2 / I-9.17 / I-9.23「另一份按同一长度读回比校验和，
+        // 对不上照红 I-2.1」）。中央映射条目读那片容器时同一份也对不上，这三条不只由 F-W 这一读红。
+        for invariant_of_the_checksum_family in ["I-2.1", "I-4.8", "I-7.4"] {
+            assert!(
+                violated.contains(invariant_of_the_checksum_family),
+                "码 2 那一份在位置条目 {location_index}：按类型段那一类的长度读回对不上，{invariant_of_the_checksum_family} 红：{verdicts:?}"
+            );
+        }
         violated_on_each_device.push(violated);
     }
     assert_eq!(
@@ -13265,3 +13274,691 @@ fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_second_copy_
         "inode 树根条目类型段 7、第二条位置条目指着不在池里的盘",
     );
 }
+
+// ─── 第 32 批（删除线）：「走过」按读回那一份的落点认（F-KEY）、类型段 0 认「孩子是码 2」只认自证过的副本（F-ID-b）、层级 0 的根下 F-L0、
+// 类型段 0 那一支的 F-W 与同长度别类（G1–G3） ───
+
+/// 内部条目子指针里第 `location_index` 条位置条目的槽写成 `slot`（6 字节小端；盘与整单元校验和不动）。
+fn set_the_slot_in_a_child_location(entry: &mut [u8], location_index: usize, slot: u64) {
+    let slot_offset = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
+        + POINTER_FIRST_LOCATION_OFFSET
+        + location_index * POINTER_LOCATION_BYTES
+        + LOCATION_DEVICE_BYTES;
+    entry[slot_offset..slot_offset + LOCATION_SLOT_BYTES]
+        .copy_from_slice(&slot.to_le_bytes()[..LOCATION_SLOT_BYTES]);
+}
+
+/// 位置条目里盘身份的字节数（盘 4 + 槽 6 + 整单元校验和 4）。
+const LOCATION_DEVICE_BYTES: usize = 4;
+/// 位置条目里槽号的字节数。
+const LOCATION_SLOT_BYTES: usize = 6;
+
+/// 攻方 O3 的造法（判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K2 观察 O3 那一行、第三节第 2 条；攻方
+/// `research/prompts/m3-batch31-code-r1-opus-output.md` 第四节 `variant=bad` 那两形，模型 `research/prompts/m3-batch31-code-r1-opus-model/probe_kbi.rs`
+/// 的 `o3_probe`）：种类 `kind` 那棵树根（层级 1）第一条条目的孩子（码 3 容器或叶容器）两份头里的记录宽减 1（重封、整单元校验和沿引用链补到根，
+/// 只这一处是 I-1.7 违例）；再把那条条目子指针第 `location_index` 条位置条目的槽改成这一遍先走过的码 2 节点 X（种类 `kind_walked_before`
+/// 那棵树的根）的槽、整单元校验和写成 X 那 16 KiB 的 CRC 翻几位（这一份对不上），重封到根。另一条位置条目那一份就是带违例、
+/// 按自己的位置条目自证得过的容器，读者用的就是它。
+fn image_whose_first_root_entry_names_a_walked_node_beside_a_container_with_a_wrong_record_width(
+    clean: &MemoryPool,
+    kind: u16,
+    record_bytes: u64,
+    kind_walked_before: u16,
+    location_index: usize,
+) -> MemoryPool {
+    let mut image = clean.clone();
+    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
+        &mut image,
+        kind,
+        record_width_one_below(record_bytes),
+    );
+    let walked_node_pointer =
+        common_pointer_head_corruption::tree_table_entry_of_kind(&image, kind_walked_before).root;
+    let walked_node_slot = common_pointer_head_corruption::the_slot_of(&walked_node_pointer);
+    let wrong_checksum_of_the_walked_node =
+        crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
+            &image,
+            walked_node_slot,
+            common_pointer_head_corruption::node_bytes(),
+        )) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
+    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
+        set_the_slot_in_a_child_location(entry, location_index, walked_node_slot.0);
+        set_the_whole_unit_checksum_in_a_child_location(
+            entry,
+            location_index,
+            wrong_checksum_of_the_walked_node,
+        );
+    });
+    assert_eq!(level, 1, "根层级 1、直接管容器");
+    image
+}
+
+/// 翻进校验和里的几位：那一份按哪种长度都对不上。
+const CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY: u32 = 0x5a5a_5a5a;
+
+/// F-KEY 的判法：两种盘序（指着先走过那个节点的是位置条目 0 / 1）各判一次，I-1.7 都红、违例说明点名那片容器的记录宽；红的同一批
+/// （`allowed_to_differ` 里的几条除外，见各用例）。改之前「走过」按第 0 条位置条目认：位置条目 0 指着先走过的节点时读回来的那片容器当成走过、
+/// 合并、返回，I-1.7 报成立（攻方 `PROBE o3 … collide=loc0 … i1_7=Holds`），位置条目 1 那一形照判、红。
+fn assert_the_packed_container_invariant_reddens_alike_whichever_location_names_the_walked_node(
+    clean: &MemoryPool,
+    kind: u16,
+    record_bytes: u64,
+    kind_walked_before: u16,
+    expected_violations_on_location_one: &[&'static str],
+    allowed_to_differ: &[&'static str],
+) {
+    let mut violated_on_each_location = Vec::new();
+    for location_index in 0..2 {
+        let image = image_whose_first_root_entry_names_a_walked_node_beside_a_container_with_a_wrong_record_width(
+            clean,
+            kind,
+            record_bytes,
+            kind_walked_before,
+            location_index,
+        );
+        let verdicts = check_pool_image(&image);
+        let packed_container_verdict = verdicts
+            .iter()
+            .find(|(identifier, _)| *identifier == "I-1.7")
+            .map(|(_, found)| found.clone())
+            .expect("清单里有");
+        assert!(
+            matches!(
+                &packed_container_verdict,
+                InvariantVerdict::Violated(detail)
+                    if detail.contains(&format!("记录宽 {}", record_bytes - 1))
+            ),
+            "指着先走过的节点的是位置条目 {location_index}：读回来的那片容器照判，I-1.7 红在它的记录宽：{verdicts:?}"
+        );
+        violated_on_each_location.push(violated_invariant_set(&verdicts));
+    }
+    assert_eq!(
+        violated_on_each_location[1],
+        expected_violations_on_location_one
+            .iter()
+            .copied()
+            .collect::<std::collections::BTreeSet<&'static str>>(),
+        "位置条目 1 指着先走过的节点（攻方 `collide=loc1` 那一形）红的那一批"
+    );
+    let outside_the_allowed = |violated: &std::collections::BTreeSet<&'static str>| {
+        violated
+            .iter()
+            .copied()
+            .filter(|invariant| !allowed_to_differ.contains(invariant))
+            .collect::<std::collections::BTreeSet<&'static str>>()
+    };
+    assert_eq!(
+        outside_the_allowed(&violated_on_each_location[0]),
+        outside_the_allowed(&violated_on_each_location[1]),
+        "同一张镜像只差指着先走过的节点的是哪一条位置条目：{allowed_to_differ:?} 之外红的同一批"
+    );
+}
+
+/// F-KEY 墓碑那一形：三个单元的文件发完意图之后，墓碑容器记录宽 40 → 39，墓碑树根第一条条目子指针的一条位置条目指着 inode 树根（先走过）。
+#[test]
+fn a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before(
+) {
+    assert_the_packed_container_invariant_reddens_alike_whichever_location_names_the_walked_node(
+        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+            "known-bad-tombstone-record-width-location-naming-a-walked-node",
+        ),
+        TREE_KIND_TOMBSTONE,
+        singlefs_format::TOMBSTONE_RECORD_BYTES,
+        TREE_KIND_INODE,
+        &[
+            "I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4",
+        ],
+        &[],
+    );
+}
+
+/// F-KEY inode 那一形：新池新建文件那一版，inode 叶容器记录宽 140 → 139，inode 树根第一条条目子指针的一条位置条目指着 extent 树根（先走过）。
+#[test]
+fn an_inode_leaf_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before(
+) {
+    assert_the_packed_container_invariant_reddens_alike_whichever_location_names_the_walked_node(
+        &build_pool("known-bad-inode-leaf-record-width-location-naming-a-walked-node")
+            .memory_pool(),
+        TREE_KIND_INODE,
+        singlefs_format::INODE_RECORD_BYTES,
+        TREE_KIND_EXTENT,
+        &[
+            "I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4",
+        ],
+        &[],
+    );
+}
+
+/// F-KEY 谱系那一形：回退到 A 之后，谱系容器记录宽 16 → 15，谱系树根第一条条目子指针的一条位置条目指着 inode 树根（先走过，
+/// 回退那一版照抄 A 的 inode 树，A 那条根也引用它）。位置条目 0 那一形另多红 I-4.9：读回来的那片容器（这一版新写的）照判时，
+/// 它的诞生代号按第 0 条位置条目记在 inode 树根那个落点上（`birth_txg_of_walked_units` 照旧按第 0 条认），A 那条根再走到 inode 树根时按这个数判
+/// （推的，读码；攻方 F-KEY 臂量到同样的差，`research/prompts/m3-batch31-code-r1-opus-output.md` 第四节那张表），交主 agent 定，这里不钉它。
+#[test]
+fn a_lineage_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before(
+) {
+    assert_the_packed_container_invariant_reddens_alike_whichever_location_names_the_walked_node(
+        &image_after_rolling_back_to_the_first_root(
+            "known-bad-lineage-record-width-location-naming-a-walked-node",
+        )
+        .image,
+        TREE_KIND_LINEAGE,
+        singlefs_format::LINEAGE_RECORD_BYTES,
+        TREE_KIND_INODE,
+        &[
+            "I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4",
+        ],
+        &[UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT],
+    );
+}
+
+/// 攻方 A3 的造法（判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K2 N1 盘序翻面 A3 那一行；攻方同一份第一节 A3，
+/// 模型 `probe_kbi.rs` 的 `matrix_probe` 里 `c16` 与 `node` 两种副本）：种类 `kind` 那棵树根第一条条目类型段 `registered_type` → 0、身份三段清零；
+/// 孩子（码 3 容器）在位置条目 `container_location_index` 那块盘上不动、那条位置条目的整单元校验和换成容器前 16 KiB 的 CRC（`c16`：头里是码 3，
+/// 按 32 KiB 自证不过）；另一块盘上孩子的槽写进这棵树根（码 2）的原样字节、那条位置条目的校验和换成它的 CRC（`node`：按 16 KiB 自证得过）。
+/// `root_level` 给了就先把根头里的层级改成它（层级 0 的根，攻方 `l0root`）。
+fn image_whose_type_zero_root_entry_names_a_container_checked_at_sixteen_kibibytes_beside_a_node(
+    clean: &MemoryPool,
+    kind: u16,
+    registered_type: u16,
+    container_location_index: usize,
+    root_level: Option<u8>,
+) -> MemoryPool {
+    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, kind, 0);
+    let child_slot = common_pointer_head_corruption::the_slot_of(&child);
+    let container = common_pointer_head_corruption::unit_bytes_at(
+        clean,
+        child_slot,
+        common_pointer_head_corruption::data_unit_bytes(),
+    );
+    assert_eq!(
+        container[UNIT_HEADER_CLASS_TAG_OFFSET], 3,
+        "孩子是码 3 容器"
+    );
+    let checksum_of_the_first_sixteen_kibibytes =
+        crc32_castagnoli(&container[..common_pointer_head_corruption::node_bytes()]);
+    let node = root_node_bytes_of_the_tree_of_kind(clean, kind);
+    let checksum_of_the_node = crc32_castagnoli(&node);
+    let node_location_index = 1 - container_location_index;
+    let mut image = clean.clone();
+    write(
+        &mut image,
+        child.locations[node_location_index].device.0,
+        child_slot.0 * SLOT,
+        &node,
+    );
+    if let Some(level) = root_level {
+        set_the_level_of_the_root_of_kind(&mut image, kind, level);
+    }
+    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            registered_type,
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        );
+        zero_the_three_identity_fields(entry);
+        set_the_whole_unit_checksum_in_a_child_location(
+            entry,
+            container_location_index,
+            checksum_of_the_first_sixteen_kibibytes,
+        );
+        set_the_whole_unit_checksum_in_a_child_location(
+            entry,
+            node_location_index,
+            checksum_of_the_node,
+        );
+    });
+    assert_eq!(level, root_level.unwrap_or(1), "根的层级是这一形要的那一级");
+    image
+}
+
+/// 种类 `kind` 那棵树根头里的层级改成 `level`（重封、整单元校验和沿引用链补到根）。
+fn set_the_level_of_the_root_of_kind(image: &mut MemoryPool, kind: u16, level: u8) {
+    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
+    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+        image,
+        common_pointer_head_corruption::the_slot_of(&root_pointer),
+        common_pointer_head_corruption::node_bytes(),
+        |root| root[INDEX_NODE_LEVEL_OFFSET] = level,
+    );
+}
+
+/// F-ID-b 的判法（用户 2026-10-07 定「只认自证过的副本」）：两种盘序（`c16` 那一份在位置条目 0 / 1）各判一次，认「孩子是码 2」只看按自己那一类长度
+/// 整份自证过的副本——`c16` 不算、`node` 那一份是码 2：`identity_invariant` 两种盘序都不红；红的恰好是 I-2.1 一族与走读失败
+/// （I-2.1、I-4.8、I-7.2、I-7.4：F-L0 按头里的码 3 比 `c16` 那一份对不上，层级 ≤ 1 不往下走、记走读失败），两种盘序同一批。
+/// 改之前取第一份按 16 KiB 对得上的副本：`c16` 在位置条目 0 时认成码 3，`identity_invariant` 红，换盘就不红（攻方 `c0=c16 c1=node` 对 `c0=node c1=c16`）。
+fn assert_the_entry_invariant_holds_alike_whichever_device_holds_the_copy_checked_at_sixteen_kibibytes(
+    clean: &MemoryPool,
+    kind: u16,
+    registered_type: u16,
+    identity_invariant: &'static str,
+    root_level: Option<u8>,
+) {
+    for container_location_index in 0..2 {
+        let image = image_whose_type_zero_root_entry_names_a_container_checked_at_sixteen_kibibytes_beside_a_node(
+            clean,
+            kind,
+            registered_type,
+            container_location_index,
+            root_level,
+        );
+        let verdicts = check_pool_image(&image);
+        assert_eq!(
+            violated_invariant_set(&verdicts),
+            ["I-2.1", "I-4.8", "I-7.2", "I-7.4"]
+                .into_iter()
+                .collect::<std::collections::BTreeSet<&'static str>>(),
+            "`c16` 那一份在位置条目 {container_location_index}：{identity_invariant} 不红，红的恰好是 I-2.1 一族与走读失败：{verdicts:?}"
+        );
+    }
+}
+
+#[test]
+fn a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device(
+) {
+    assert_the_entry_invariant_holds_alike_whichever_device_holds_the_copy_checked_at_sixteen_kibibytes(
+        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+            "known-bad-tombstone-entry-type-zero-sixteen-kibibyte-copy-beside-a-node",
+        ),
+        TREE_KIND_TOMBSTONE,
+        PACKED_RECORD_TYPE_TOMBSTONE,
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        None,
+    );
+}
+
+#[test]
+fn a_lineage_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_lineage_entry_invariant_holding_on_either_device(
+) {
+    assert_the_entry_invariant_holds_alike_whichever_device_holds_the_copy_checked_at_sixteen_kibibytes(
+        &image_after_rolling_back_to_the_first_root(
+            "known-bad-lineage-entry-type-zero-sixteen-kibibyte-copy-beside-a-node",
+        )
+        .image,
+        TREE_KIND_LINEAGE,
+        PACKED_RECORD_TYPE_LINEAGE,
+        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        None,
+    );
+}
+
+/// 同上，墓碑树根头里的层级改成 0（攻方 `l0root` 那一对）。
+#[test]
+fn a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device(
+) {
+    assert_the_entry_invariant_holds_alike_whichever_device_holds_the_copy_checked_at_sixteen_kibibytes(
+        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+            "known-bad-tombstone-root-level-zero-entry-type-zero-sixteen-kibibyte-copy-beside-a-node",
+        ),
+        TREE_KIND_TOMBSTONE,
+        PACKED_RECORD_TYPE_TOMBSTONE,
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        Some(0),
+    );
+}
+
+/// 同上，谱系树根头里的层级改成 0。
+#[test]
+fn a_lineage_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_lineage_entry_invariant_holding_on_either_device(
+) {
+    assert_the_entry_invariant_holds_alike_whichever_device_holds_the_copy_checked_at_sixteen_kibibytes(
+        &image_after_rolling_back_to_the_first_root(
+            "known-bad-lineage-root-level-zero-entry-type-zero-sixteen-kibibyte-copy-beside-a-node",
+        )
+        .image,
+        TREE_KIND_LINEAGE,
+        PACKED_RECORD_TYPE_LINEAGE,
+        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        Some(0),
+    );
+}
+
+/// F-L0 在层级 0 的根上（判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K1 N1 层级 0 的根下类型段 0 那一行；
+/// 攻方 `PROBE l0root tree=tomb level=0 type=t0 c0=bad c1=bad`）：三个单元的文件发完意图之后，墓碑树根头里的层级 1 → 0，第一条条目类型段 1 → 0、
+/// 身份三段清零、子指针两条位置条目的整单元校验和各翻一字节，重封到根。层级 0 的节点同样不往下走、记走读失败，孩子每一份副本照判 I-2.1：
+/// 红 I-2.1、I-4.8、I-7.2、I-7.4（F-L0 只在层级恰为 1 时判的话，层级 0 这一形 I-2.1 不红）。
+#[test]
+fn a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
+) {
+    let mut image = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+        "known-bad-tombstone-root-level-zero-entry-type-zero-both-location-checksums-wrong",
+    );
+    set_the_level_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0);
+    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_TOMBSTONE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_TOMBSTONE,
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        );
+        zero_the_three_identity_fields(entry);
+        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
+        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
+    });
+    assert_eq!(level, 0, "墓碑树根头里的层级改成了 0");
+    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
+        &image,
+        &["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
+        "墓碑树根层级 0、条目类型段 0、身份三段清零、子指针两条位置条目的校验和都坏",
+    );
+}
+
+/// G1（判决 `research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M3 缺口 G1 那一行、第三节第 1 条）：类型段 0 那一支的 F-W。
+/// 种类 `kind` 那棵树根（层级 1）第一条条目类型段 `registered_type` → 0、身份三段清零，孩子在一块盘上那一份换成这棵树根（码 2）的原样字节、
+/// 按 16 KiB 自证得过（类型段 0 那一类），另一份是原来那片码 3 容器、自证成另一类
+/// （[`image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device`]）。两种盘序各判一次，红的恰好是 `expected_violations`
+/// （攻方 `research/prompts/m3-batch31-code-r1-opus-output.md` 那份探针 `type=t0 c0=node c1=ok` 与 `c0=ok c1=node` 两行量到的那一批），其中
+/// 只有照类型段 0 那一支往下走那一份才红的 `invariant_only_the_type_zero_branch_reddens`，违例说明含 `detail_only_the_type_zero_branch_writes`。
+/// 类型段 0 那一处见别类就停（不走 F-W，变异 S1 / S2）时那一份不往下走，那一条不红。
+fn assert_a_type_zero_entry_whose_child_has_a_copy_verifying_as_a_node_goes_down_that_copy_on_either_device(
+    clean: &MemoryPool,
+    kind: u16,
+    registered_type: u16,
+    expected_violations: &[&'static str],
+    invariant_only_the_type_zero_branch_reddens: &'static str,
+    detail_only_the_type_zero_branch_writes: &str,
+) {
+    for location_index in 0..2 {
+        let image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
+            clean,
+            kind,
+            location_index,
+            &|entry| {
+                change_the_entry_type(
+                    entry,
+                    registered_type,
+                    CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+                );
+                zero_the_three_identity_fields(entry);
+            },
+        );
+        let verdicts = check_pool_image(&image);
+        assert_eq!(
+            violated_invariant_set(&verdicts),
+            expected_violations
+                .iter()
+                .copied()
+                .collect::<std::collections::BTreeSet<&'static str>>(),
+            "码 2 那一份在位置条目 {location_index}：照类型段 0 那一支往下走那一份，红的那一批：{verdicts:?}"
+        );
+        let only_the_type_zero_branch = verdicts
+            .iter()
+            .find(|(identifier, _)| *identifier == invariant_only_the_type_zero_branch_reddens)
+            .map(|(_, found)| found.clone())
+            .expect("清单里有");
+        assert!(
+            matches!(
+                &only_the_type_zero_branch,
+                InvariantVerdict::Violated(detail) if detail.contains(detail_only_the_type_zero_branch_writes)
+            ),
+            "码 2 那一份在位置条目 {location_index}：{invariant_only_the_type_zero_branch_reddens} 红在类型段 0 那一支往下走那一份上（说明含「{detail_only_the_type_zero_branch_writes}」）：{only_the_type_zero_branch:?}"
+        );
+    }
+}
+
+/// G1 墓碑那一形：三个单元的文件发完意图之后。类型段 0 那一支往下走那一份——层级 1 的节点下面不往下读、记走读失败（F-L0），I-7.2 红在那一句。
+#[test]
+fn a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device(
+) {
+    assert_a_type_zero_entry_whose_child_has_a_copy_verifying_as_a_node_goes_down_that_copy_on_either_device(
+        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+            "known-bad-tombstone-entry-type-zero-container-copy-verifying-as-a-node-goes-down",
+        ),
+        TREE_KIND_TOMBSTONE,
+        PACKED_RECORD_TYPE_TOMBSTONE,
+        &[
+            "I-2.1",
+            "I-4.8",
+            "I-7.2",
+            "I-7.4",
+            TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        ],
+        "I-7.2",
+        "的节点有类型段 0 的条目",
+    );
+}
+
+/// G1 谱系那一形：回退到 A 之后。同墓碑那一形。
+#[test]
+fn a_lineage_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device(
+) {
+    assert_a_type_zero_entry_whose_child_has_a_copy_verifying_as_a_node_goes_down_that_copy_on_either_device(
+        &image_after_rolling_back_to_the_first_root(
+            "known-bad-lineage-entry-type-zero-container-copy-verifying-as-a-node-goes-down",
+        )
+        .image,
+        TREE_KIND_LINEAGE,
+        PACKED_RECORD_TYPE_LINEAGE,
+        &[
+            "I-2.1",
+            "I-4.8",
+            "I-7.2",
+            "I-7.4",
+            LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        ],
+        "I-7.2",
+        "的节点有类型段 0 的条目",
+    );
+}
+
+/// G1 inode 那一形：新池新建文件那一版。类型段 0 那一支往下走那一份——码 2 那一份当 inode 树的内部节点读（它就是 inode 树根，层级 1，
+/// 不是父层级减一），I-1.1 红在「inode 树的内部节点」那一句，槽数按码 2 一槽登记（叶容器另一槽没人引用：I-3.1、I-3.11、I-5.1，攻方
+/// `research/prompts/m3-batch30-code-r1-opus-output.md` 观察 O1 那几条）。
+#[test]
+fn an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device(
+) {
+    assert_a_type_zero_entry_whose_child_has_a_copy_verifying_as_a_node_goes_down_that_copy_on_either_device(
+        &build_pool("known-bad-inode-entry-type-zero-leaf-copy-verifying-as-a-node-goes-down")
+            .memory_pool(),
+        TREE_KIND_INODE,
+        PACKED_RECORD_TYPE_INODE,
+        &[
+            "I-1.1", "I-1.2", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.2", "I-7.4", "I-9.2",
+        ],
+        "I-1.1",
+        "inode 树的内部节点",
+    );
+}
+
+/// G2（判决 `research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M3 缺口 G2 那一行、第三节第 2 条）：新池新建文件那一版，
+/// inode 树根第一条条目不改（类型段 2，说孩子是码 3），叶容器盘 1 那一份换成第一个文件的数据单元（码 1、32 KiB）原样字节、位置条目 1 的整单元校验和
+/// 换成它的 CRC——按类型段那一类（码 3）的长度 32 KiB 校验和对得上、头里却是同长度的别类码 1；位置条目 0 的校验和翻一字节（盘 0 那一份自证不过）。
+/// 没有一份头里是码 3 又自证得过：不走 F-W，I-9.2 红在「孩子头里的类标签是 DataUnit」那一句、不往下走，码 1 那一份不当叶容器判（I-1.3、I-1.7 不红）。
+/// 只看整单元校验和、不看头里的类标签时（正推 S4），码 1 那一份算作类型段那一类自证过、照类型段那一支往下走，I-9.2 的说明写成往下走的那一句。
+#[test]
+fn an_inode_tree_entry_whose_only_leaf_copy_matching_at_the_container_length_is_a_data_unit_does_not_go_down_that_copy(
+) {
+    let mut image =
+        build_pool("known-bad-inode-leaf-copy-matching-at-the-container-length-is-a-data-unit")
+            .memory_pool();
+    let child = child_pointer_of_an_entry_of_the_root_of_kind(&image, TREE_KIND_INODE, 0);
+    let leaf_slot = common_pointer_head_corruption::the_slot_of(&child);
+    let data_unit = common_pointer_head_corruption::unit_bytes_at(
+        &image,
+        singlefs_core::address::SlotNumber(DATA_UNIT),
+        common_pointer_head_corruption::data_unit_bytes(),
+    );
+    assert_eq!(
+        data_unit[UNIT_HEADER_CLASS_TAG_OFFSET], 1,
+        "第一个文件的数据单元是码 1"
+    );
+    write(
+        &mut image,
+        child.locations[1].device.0,
+        leaf_slot.0 * SLOT,
+        &data_unit,
+    );
+    let checksum_of_the_data_unit = crc32_castagnoli(&data_unit);
+    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
+        set_the_whole_unit_checksum_in_a_child_location(entry, 1, checksum_of_the_data_unit);
+        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
+    });
+    assert_the_violation_names_the_child_class(
+        &image,
+        "I-9.2",
+        PACKED_RECORD_TYPE_INODE,
+        "DataUnit",
+    );
+    let verdicts = check_pool_image(&image);
+    let violated = violated_invariant_set(&verdicts);
+    for invariant_of_walking_that_copy_as_a_leaf in ["I-1.3", "I-1.7"] {
+        assert!(
+            !violated.contains(invariant_of_walking_that_copy_as_a_leaf),
+            "码 1 那一份不往下走、不当叶容器判：{invariant_of_walking_that_copy_as_a_leaf} 不红：{verdicts:?}"
+        );
+    }
+    assert!(
+        matches!(
+            verdict(&image, "I-9.2"),
+            InvariantVerdict::Violated(detail) if !detail.contains("照类型段那一支往下走那一份")
+        ),
+        "I-9.2 红在「孩子是另一类、不往下走」那一句，不是 F-W 往下走那一句：{:?}",
+        verdict(&image, "I-9.2")
+    );
+}
+
+/// 码 3 打包记录单元头里记录宽（2 字节）的偏移（D18（块里携带什么信息） 已定项 11：记录数 69、记录宽 71）。
+const PACKED_UNIT_HEADER_RECORD_WIDTH_OFFSET: usize = 71;
+
+/// 旋钮下长到层级 2 的那张镜像里两块盘上都没写过的一段（32 KiB）：远在单元区写过的那几槽之后、4 GiB 的盘之内（写之前核全零）。
+const SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE: u64 = 180_000;
+
+/// G3（判决 `research/prompts/m3-batch30-code-r1-main-verification.md` 第二节 K1 M3 缺口 G3 那一行、第三节第 3 条）：层级 2 的树里类型段 0、
+/// F-W 往下走那一份子树。墓碑树在旋钮下长到层级 2（[`image_with_a_tombstone_tree_of_level_two`]），根最后一条条目（类型段 0）指层级 1 的节点 N，
+/// N 最后一条条目指墓碑容器 C（旋钮一片只装一条墓碑，最后一次 unlink 的墓碑另起一片：C 与 N 只有最新那几版引用）。C 两份的记录宽 40 → 39
+/// （重封、补到根）；再把 C 的 32 KiB 原样字节写进位置条目 `location_index` 那块盘上一段没写过的槽 [`SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE`]、
+/// 根那条条目的那条位置条目改指它、校验和换成它的 CRC（那一份自证成码 3，另一类；不写进 N 自己的槽：N 是 16 KiB，写 32 KiB 会盖掉下一槽的单元），
+/// 另一条位置条目指的 N 不动（码 2，类型段 0 那一类，自证得过）。F-W：I-9.23 红，照类型段 0 那一支往下读 N 那一份，N 下面 C 的 I-1.7 违例照红。
+/// 类型段 0 那一处见别类就停（正推 S1）时 N 不往下走，I-1.7 不红。
+fn assert_a_wrong_record_width_below_a_level_one_node_with_a_copy_verifying_as_a_container_reddens_the_packed_container_invariant(
+    location_index: usize,
+) {
+    let mut image = image_with_a_tombstone_tree_of_level_two(&format!(
+        "known-bad-tombstone-level-two-node-copy-verifying-as-a-container-location-{location_index}"
+    ));
+    let root_pointer =
+        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_TOMBSTONE).root;
+    let root =
+        singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
+            &image,
+            common_pointer_head_corruption::the_slot_of(&root_pointer),
+            common_pointer_head_corruption::node_bytes(),
+        ))
+        .expect("根解得开");
+    assert_eq!(root.level, 2, "墓碑树根长到层级 2");
+    let last_entry_index = root.entries.len() - 1;
+    let node_pointer = child_pointer_of_an_entry_of_the_root_of_kind(
+        &image,
+        TREE_KIND_TOMBSTONE,
+        last_entry_index,
+    );
+    let node_slot = common_pointer_head_corruption::the_slot_of(&node_pointer);
+    let node =
+        singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
+            &image,
+            node_slot,
+            common_pointer_head_corruption::node_bytes(),
+        ))
+        .expect("层级 1 的节点解得开");
+    assert_eq!(node.level, 1, "根最后一条条目指层级 1 的节点");
+    let container_pointer =
+        singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(
+            node.entries.last().expect("层级 1 的节点至少一条条目"),
+            CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
+        ));
+    let container_slot = common_pointer_head_corruption::the_slot_of(&container_pointer);
+    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+        &mut image,
+        container_slot,
+        common_pointer_head_corruption::data_unit_bytes(),
+        |container| {
+            assert_eq!(
+                container[UNIT_HEADER_CLASS_TAG_OFFSET], 3,
+                "N 下面最后一个孩子是码 3 墓碑容器"
+            );
+            let width_field =
+                PACKED_UNIT_HEADER_RECORD_WIDTH_OFFSET..PACKED_UNIT_HEADER_RECORD_WIDTH_OFFSET + 2;
+            assert_eq!(
+                u64::from(u16::from_le_bytes([
+                    container[width_field.start],
+                    container[width_field.start + 1]
+                ])),
+                singlefs_format::TOMBSTONE_RECORD_BYTES,
+                "墓碑容器的记录宽是登记的 40"
+            );
+            container[width_field].copy_from_slice(
+                &record_width_one_below(singlefs_format::TOMBSTONE_RECORD_BYTES).to_le_bytes(),
+            );
+        },
+    );
+    let container = common_pointer_head_corruption::unit_bytes_at(
+        &image,
+        container_slot,
+        common_pointer_head_corruption::data_unit_bytes(),
+    );
+    let device_of_the_container_copy = node_pointer.locations[location_index].device.0;
+    assert!(
+        read(
+            &image,
+            device_of_the_container_copy,
+            SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE * SLOT,
+            common_pointer_head_corruption::data_unit_bytes(),
+        )
+        .iter()
+        .all(|byte| *byte == 0),
+        "槽 {SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE} 起 32 KiB 在盘 {device_of_the_container_copy} 上没写过"
+    );
+    write(
+        &mut image,
+        device_of_the_container_copy,
+        SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE * SLOT,
+        &container,
+    );
+    let checksum_of_the_container = crc32_castagnoli(&container);
+    let level = change_an_entry_of_the_root_of_kind(
+        &mut image,
+        TREE_KIND_TOMBSTONE,
+        last_entry_index,
+        |entry| {
+            assert_eq!(
+                entry_type_of(entry),
+                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+                "层级 2 的根条目类型段 0"
+            );
+            set_the_slot_in_a_child_location(
+                entry,
+                location_index,
+                SLOT_NOTHING_WROTE_IN_THE_LEVEL_TWO_IMAGE,
+            );
+            set_the_whole_unit_checksum_in_a_child_location(
+                entry,
+                location_index,
+                checksum_of_the_container,
+            );
+        },
+    );
+    assert_eq!(level, 2, "墓碑树根长到层级 2");
+    let verdicts = check_pool_image_under_the_tombstone_knob(&image);
+    assert!(
+        matches!(
+            verdict_under_the_tombstone_knob(&image, "I-1.7"),
+            InvariantVerdict::Violated(detail)
+                if detail.contains(&format!("记录宽 {}", singlefs_format::TOMBSTONE_RECORD_BYTES - 1))
+        ),
+        "码 3 那一份在位置条目 {location_index}：照类型段 0 那一支往下读 N 那一份，N 下面那片墓碑容器的 I-1.7 违例照红：{verdicts:?}"
+    );
+    assert!(
+        violated_invariant_set(&verdicts).contains(TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER),
+        "码 3 那一份在位置条目 {location_index}：根那条条目照判 I-9.23 红：{verdicts:?}"
+    );
+}
+
+#[test]
+fn a_tombstone_container_below_a_level_one_node_whose_first_copy_verifies_as_a_container_reddens_the_packed_container_invariant(
+) {
+    assert_a_wrong_record_width_below_a_level_one_node_with_a_copy_verifying_as_a_container_reddens_the_packed_container_invariant(0);
+}
+
+#[test]
+fn a_tombstone_container_below_a_level_one_node_whose_second_copy_verifies_as_a_container_reddens_the_packed_container_invariant(
+) {
+    assert_a_wrong_record_width_below_a_level_one_node_with_a_copy_verifying_as_a_container_reddens_the_packed_container_invariant(1);
+}
````

## 实现员 p：mutations-append.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch32/p/patch/mutations-append.tsv，追加到 `crates/mutations.tsv` 末尾，合后第 3078–3089 行）

````tsv
第32批 F-KEY：走读认「走过」按第 0 条位置条目的落点、不按读回那一份（树根条目第 0 条位置条目指着先走过的码 2 节点、盘 1 那份容器记录宽减 1，I-1.7 报成立）	crates/singlefs-checker/src/walk.rs	        .map(|(unit, location_it_was_read_from)| {\n            (\n                unit,\n                (\n                    location_it_was_read_from.device,\n                    location_it_was_read_from.slot,\n                ),\n            )\n        })\n	        .map(|(unit, _location_it_was_read_from)| {\n            (\n                unit,\n                (pointer.locations[0].device, pointer.locations[0].slot),\n            )\n        })\n	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_whose_only_leaf_copy_matching_at_the_container_length_is_a_data_unit_does_not_go_down_that_copy	a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before
第32批 F-ID-b：墓碑树、谱系树类型段 0 那一支认「孩子是码 2」取第一份按 16 KiB 对得上的副本（c16 在位置条目 0 时 I-9.23 / I-9.17 红，换盘不红）	crates/singlefs-checker/src/walk.rs	                    let child_class =\n                        class_tag_of_the_first_copy_verifying_at_the_length_of_its_class(\n                            self.reader,\n                            &child,\n                        )\n                        .or_else(|| {\n                            read_unit_without_judging(self.reader, &child, node_bytes())\n                                .map(|unit| unit[UNIT_HEADER_CLASS_TAG_OFFSET])\n                        });\n	                    let child_class = read_unit_without_judging(self.reader, &child, node_bytes())\n                        .map(|unit| unit[UNIT_HEADER_CLASS_TAG_OFFSET]);\n	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_whose_only_leaf_copy_matching_at_the_container_length_is_a_data_unit_does_not_go_down_that_copy	a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device
第32批 M3：F-L0 那一处只在层级恰为 1 时照判 I-2.1（墓碑树根层级 0、类型段 0、身份清零、两条位置条目校验和都坏，I-2.1 报成立）	crates/singlefs-checker/src/walk.rs	        if node.level <= 1 {\n            self.judge_every_copy_of_a_child_the_walk_does_not_go_into(\n                child,\n                &format!("{name}内部条目的子指针"),\n            );\n	        if node.level <= 1 {\n            if node.level == 1 {\n                self.judge_every_copy_of_a_child_the_walk_does_not_go_into(\n                    child,\n                    &format!("{name}内部条目的子指针"),\n                );\n            }\n	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_whose_only_leaf_copy_matching_at_the_container_length_is_a_data_unit_does_not_go_down_that_copy	a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants
第32批 S1：墓碑树、谱系树那一处只在类型段说码 3 时走 F-W（类型段 0、另一份自证成码 2 时见别类就停，不记走读失败、I-7.2 不红）	crates/singlefs-checker/src/walk.rs	                    if some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(\n	                    if class_the_entry_type_names == MappingKeyUnitClass::PackedRecordUnit\n                        && some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(\n	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_whose_only_leaf_copy_matching_at_the_container_length_is_a_data_unit_does_not_go_down_that_copy below_a_level_one_node_whose	a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device
第32批 S2：inode 树那一处只在类型段 2 时走 F-W（类型段 0、另一份自证成码 2 时见别类就停，码 2 那一份不当内部节点读、I-1.1 不红）	crates/singlefs-checker/src/walk.rs	                            class_the_entry_type_names.is_some_and(|class| {\n	                            class_the_entry_type_names.filter(|class| *class == MappingKeyUnitClass::PackedRecordUnit).is_some_and(|class| {\n	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_whose_only_leaf_copy_matching_at_the_container_length_is_a_data_unit_does_not_go_down_that_copy	an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device
第32批 S4：认「另一份按类型段那一类自证过」不看头里的类标签（inode 叶容器盘 1 那份是码 1 数据单元、按 32 KiB 对得上，照类型段那一支往下走）	crates/singlefs-checker/src/walk.rs	                MappingKeyUnitClass::of(content[UNIT_HEADER_CLASS_TAG_OFFSET])\n                    == class_the_entry_type_names\n                    && crc32_castagnoli_table(&content) == location.checksum\n            })\n    })\n}\n	                crc32_castagnoli_table(&content) == location.checksum\n            })\n    })\n}\n	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_whose_only_leaf_copy_matching_at_the_container_length_is_a_data_unit_does_not_go_down_that_copy	an_inode_tree_entry_whose_only_leaf_copy_matching_at_the_container_length_is_a_data_unit_does_not_go_down_that_copy
第32批 GPU 判器 F-KEY：走读内核「走过」按第一条位置条目认（墓碑容器记录宽 39、墓碑树根条目第 0 条位置条目指着 inode 树根那条流上 CPU 红 I-1.7，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    if (w_read_back_from_the_second_location) {\n        device_read_from = p.device1;\n	    if (false && w_read_back_from_the_second_location) {\n        device_read_from = p.device1;\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_the_first_location_of_a_tombstone_container_with_a_wrong_record_width_names_a_node_walked_before	the_gpu_judge_agrees_with_the_cpu_when_the_first_location_of_a_tombstone_container_with_a_wrong_record_width_names_a_node_walked_before
第32批 GPU 判器 F-ID-b：走读内核类型段 0 认孩子的类取第一份按 16 KiB 对得上的（墓碑树根条目类型段 0、c16 在位置条目 0 那条流上 GPU 红 I-9.23，CPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            var class_of_the_child_of_a_type_zero_entry = w_class_of_the_first_copy_verifying_at_the_length_of_its_class(child);\n	            var class_of_the_child_of_a_type_zero_entry = NONE;\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_checked_at_sixteen_kibibytes_beside_a_node	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_checked_at_sixteen_kibibytes_beside_a_node
第32批 GPU 判器：走读内核 inode 树类型段 0 的内部节点不记判不了（inode 树根条目类型段 0、另一份自证成码 2 那条流上一个状态都不落原因字 4）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            // 类型段 0 的内部节点（类型段不认得的上面已经判完）：这一版没有这种形状（inode_tree_root_level_is_one），GPU 判不了\n            undecidable = UNDECIDABLE_TOO_DEEP;\n	            // 类型段 0 的内部节点（类型段不认得的上面已经判完）：这一版没有这种形状（inode_tree_root_level_is_one），GPU 判不了\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_leaves_to_the_cpu_an_inode_root_entry_of_type_zero_over_a_leaf_with_a_copy_verifying_as_a_node	the_gpu_judge_leaves_to_the_cpu_an_inode_root_entry_of_type_zero_over_a_leaf_with_a_copy_verifying_as_a_node
第32批 GPU 判器 F-IT：走读内核实例表链上 I-1.2 不比写序实例（实例表指针头部实例加 1 那条流上 CPU 红 I-1.2，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        w_judge(INVARIANT_I_1_2, eq64(rd64(s.base, 73u), pointer.birth_txg) && rd32(s.base, 93u) == pointer.instance\n	        w_judge(INVARIANT_I_1_2, eq64(rd64(s.base, 73u), pointer.birth_txg)\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_instance_than_its_page	the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_instance_than_its_page
第32批 GPU 判器 F-IT：走读内核实例表链上 I-1.2 不比出生序号（实例表指针尾段加 1 那条流上 CPU 红 I-1.2，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            && eq64(vec2<u32>(rd32(s.base, 103u), 0u), pointer.tail));\n	            );\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_sequence_than_its_page	the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_sequence_than_its_page
第32批 GPU 判器 F-零槽：读者对单元区里没写过的槽照旧读不出（墓碑树根条目类型段 7、第 1 条位置条目指着没写过的槽、校验和是全零 16 KiB 的 CRC 那条流上 GPU 多红 I-2.1）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    return select(0u, 1u, first_slot >= location_word(before, 11u));\n	    return NONE;\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_with_the_checksum_of_zeros	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_with_the_checksum_of_zeros
````
