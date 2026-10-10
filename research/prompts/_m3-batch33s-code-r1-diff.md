# 删除批第 33 批 S 合入的改动原样（2026-10-07）

## 实现员 s：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch33/s/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 7347ef2..b50f486 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -296,6 +296,24 @@ enum KeyRangeReading {
     PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller,
 }
 
+/// 主走读读一个码 2 节点（[`Walk::read_index_node`]）的三种结局。「先走过」按读回那一份的落点认，调用方看这个结局，不在读之前另按
+/// 第 0 条位置条目问一遍（第 0 条那一份对不上、读回的是被先走过的版本读过的第 1 条那一份时，两个键会把「先走过」认成「走不下去」，
+/// 第 32 批实现员报告停下第 2 条；判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 1 条）。
+#[derive(Debug)]
+enum IndexNodeReading {
+    /// 这一遍第一次走到，头、出生身份、树 ID 与 key 都判过：节点连同读回那一份的落点（这个单元在走读里的身份，往下走时记成父单元）。
+    WalkedFirstByThisVersion {
+        view: crate::IndexNodeView,
+        placement_it_was_read_from: (u32, u64),
+    },
+    /// 读回那一份的落点别的版本先走过：它下面的落点已经并进这一版（[`Walk::merge_placements_below_a_unit_walked_by_an_earlier_version`]），不重走。
+    WalkedByAnEarlierVersion {
+        placement_it_was_read_from: (u32, u64),
+    },
+    /// 指针全零，或两份都读不出、头用不了、条目区解不开（后几种走读失败已记）：这一支不往下走。
+    NotWalkable,
+}
+
 /// 多层码 2 树一个节点的条目宽在走读里怎么对待。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum EntryWidthInTheWalk {
@@ -424,13 +442,15 @@ struct Walk<'reader> {
     referenced_units_judged_against_their_pointer: u64,
     /// 正在走的是哪一版、它的 `checkpoint_txg`：I-9.25 与 I-3.13 按这一版的 txg 判，I-4.9 只在不是最新的候选根上判。
     version_being_walked: VersionBeingWalked,
-    /// 走过的被引用单元（按位置条目第 0 条的落点认，与 `visited_units` 同一个键）→ 头里的诞生代号：别的版本再引用它时不重读，
+    /// 走过的被引用单元（按读回那一份的落点认，与 `visited_units` 同一个键）→ 头里的诞生代号：别的版本再引用它时不重读，
     /// I-4.9（非最新候选根引用的单元不晚于它） 照这张表判。
     birth_txg_of_walked_units: BTreeMap<(u32, u64), u64>,
     /// I-4.9 在不是最新的候选根上判过几个被引用单元：一个都没有（候选集只剩最新那条根）时整条报不适用。
     units_judged_against_an_older_candidate_root: u64,
     /// 跟着指针记下的每个落点 (设备, 槽) → 它是哪个单元（那条指针第 0 条位置条目的落点，与 `visited_units`、`birth_txg_of_walked_units`
-    /// 同一个键）：I-3.5 比相邻两版引用了哪些单元时，两份副本的落点折成一个单元。
+    /// 同一个键）：I-3.5 比相邻两版引用了哪些单元时，两份副本的落点折成一个单元。读回的是第 1 条那一份时，那个落点改记成它自己
+    /// （[`Walk::read_a_unit_to_walk_and_the_placement_it_was_read_from`]，判决 `research/prompts/m3-batch32-code-r1-main-verification.md`
+    /// 第三节第 1 条，攻方 allrbmap）：第 0 条落点上是别的单元或坏副本时，I-3.5 取的是读回那一份的头里的诞生代号。
     unit_of_each_placement: BTreeMap<(u32, u64), (u32, u64)>,
     /// 跟着指针记下的落点里，记的那一刻走读正在一棵用户可见树（extent、inode、`deleted_inodes`）里往下走的那些 → 那棵树
     /// （I-3.5 (b) 认「落在 R 树表里某棵用户可见树的子树里」按它）。别的版本共享这棵子树时不重走，照这张表接上。
@@ -441,7 +461,7 @@ struct Walk<'reader> {
     user_visible_tree_roots_of_this_version: Vec<UserVisibleTreeRootOfAVersion>,
     /// 走过的树表单元 (设备, 槽) → 它每一条条目里 I-9.25 与 I-3.5 读的几样：别的版本共享这片树表时照这份判，不重读盘。
     tree_table_entries_of_a_walked_tree_table: BTreeMap<(u32, u64), Vec<TreeTableEntryOfTheWalk>>,
-    /// 走过的实例表各片（按第 0 条位置条目的落点）→ 头里的诞生代号：I-3.5 要它。不进 `birth_txg_of_walked_units`——
+    /// 走过的实例表各片（按读回那一份的落点）→ 头里的诞生代号：I-3.5 要它。不进 `birth_txg_of_walked_units`——
     /// 那张表 I-4.9 也读，而实例表单元按 I-1.2 那一行的例外不进 I-4.9。
     birth_txg_of_walked_instance_table_pages: BTreeMap<(u32, u64), u64>,
     /// 已经走完的每一版（根环里的候选根与由记录施加出来的那几版）：I-3.5 在全部走完之后逐对比。
@@ -1024,10 +1044,14 @@ impl<'reader> Walk<'reader> {
     /// **打包记录类型 4 的实例表单元不进这个函数**：I-1.2 那一行的「**例外**」写着它只免谓词那一半，
     /// 它的现行版本由根记录持有的物理指针唯一确定；它的出生身份与指针相等那一格在 `walk_instance_table_chain` 里单判
     /// （[`Self::judge_birth_identity_against_the_pointer`]），不进 I-4.2、也不进 I-4.9 与「判过几个被引用单元」那个数。
+    ///
+    /// 诞生代号记在 `placement_it_was_read_from` 下面：读回、自证过的那一份的落点，就是这个单元在走读里的身份
+    /// （[`Self::read_a_unit_to_walk_and_the_placement_it_was_read_from`]）；I-4.9 那一行「按落点认，读落点上单元头里的诞生代号」。
     fn judge_birth_identity_of_a_referenced_unit(
         &mut self,
         unit: &[u8],
         pointer: &PointerView,
+        placement_it_was_read_from: (u32, u64),
         what: &str,
     ) {
         let Some(identity) = birth_identity_in_the_header(unit) else {
@@ -1035,10 +1059,8 @@ impl<'reader> Walk<'reader> {
         };
         self.referenced_units_judged_against_their_pointer += 1;
         let birth_txg = identity.write_order.birth_txg();
-        self.birth_txg_of_walked_units.insert(
-            (pointer.locations[0].device, pointer.locations[0].slot),
-            birth_txg,
-        );
+        self.birth_txg_of_walked_units
+            .insert(placement_it_was_read_from, birth_txg);
         self.judge_unit_birth_against_an_older_candidate_root(birth_txg, what);
         self.judge_birth_identity_against_the_pointer(identity, pointer, what);
         let mount_root_instance = self.mount_root_instance;
@@ -1261,29 +1283,37 @@ impl<'reader> Walk<'reader> {
     /// 读回、自证过的那一份的落点，不是第 0 条位置条目的（攻方 F-KEY，判决 `research/prompts/m3-batch31-code-r1-main-verification.md`
     /// 第二节 K2 观察 O3 那一行、第三节第 2 条；同实例表链那一处 `placement_it_was_read_from` 的写法）。第 0 条位置条目指着这一遍先走过的
     /// 别的单元、第 1 条那一份完好时，按第 0 条认会把读回来的这一份当成走过、它的头与格式（I-1.3、出生身份、I-1.7）一个字节都不判。
-    /// 别处的键（父单元、诞生代号那张表、并进前一版的落点表、inode 叶序）照旧按第 0 条位置条目。
+    /// 交回的落点就是这个单元在走读里的身份：走过、诞生代号那张表、并进前一版、父单元（它下面记的落点）、inode 叶序与 I-3.5 的折叠表
+    /// 一律按它认（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 1 条，攻方 allrbmap：少改一把键就有一组形翻面）。
+    /// 折叠表在这里改：读回的是第 1 条那一份、那个落点此刻还折在第 0 条落点那个单元上时，改记成它自己。调用方先记过引用
+    /// （[`Self::note_references_of_a_pointer_followed_by_the_walk`]）再调它。
     fn read_a_unit_to_walk_and_the_placement_it_was_read_from(
         &mut self,
         pointer: &PointerView,
         unit_bytes: usize,
         what: &str,
     ) -> Option<(Vec<u8>, (u32, u64))> {
-        crate::image::read_referenced_unit_with_the_location_it_was_read_from(
-            self.reader,
-            &mut self.judgements,
-            &pointer.locations,
-            unit_bytes,
-            what,
-        )
-        .map(|(unit, location_it_was_read_from)| {
-            (
-                unit,
-                (
-                    location_it_was_read_from.device,
-                    location_it_was_read_from.slot,
-                ),
-            )
-        })
+        let (unit, location_it_was_read_from) =
+            crate::image::read_referenced_unit_with_the_location_it_was_read_from(
+                self.reader,
+                &mut self.judgements,
+                &pointer.locations,
+                unit_bytes,
+                what,
+            )?;
+        let placement_it_was_read_from = (
+            location_it_was_read_from.device,
+            location_it_was_read_from.slot,
+        );
+        let first_placement = (pointer.locations[0].device, pointer.locations[0].slot);
+        if placement_it_was_read_from != first_placement
+            && self.unit_of_each_placement.get(&placement_it_was_read_from)
+                == Some(&first_placement)
+        {
+            self.unit_of_each_placement
+                .insert(placement_it_was_read_from, placement_it_was_read_from);
+        }
+        Some((unit, placement_it_was_read_from))
     }
 
     /// 一个单元别的版本已经走过（`visited_units` 命中），这一版又引用它：不重走，把它下面的落点（孩子、孩子的孩子……，
@@ -1483,6 +1513,8 @@ impl<'reader> Walk<'reader> {
 
     /// 读一个码 2 节点并判头、树 ID（I-1.3）与 key 区间（I-1.1）。`key_range_reading` 说内部节点的 key 区间怎么判：
     /// 首末两条条目的 key（inode 树根、树表等今天的写法），或多层码 2 树的子树覆盖区间（孩子读回来之后由走读那一方判）。
+    /// 「走过」、诞生代号与并进前一版都按读回那一份的落点认（同 [`Self::read_a_unit_to_walk_and_the_placement_it_was_read_from`]），
+    /// 「先走过」作为一种结局交回调用方（[`IndexNodeReading`]）。
     fn read_index_node(
         &mut self,
         pointer_bytes: &[u8],
@@ -1491,10 +1523,10 @@ impl<'reader> Walk<'reader> {
         what: &str,
         key_range_reading: KeyRangeReading,
         entry_field_table_registration: EntryFieldTableRegistration,
-    ) -> Option<crate::IndexNodeView> {
+    ) -> IndexNodeReading {
         let pointer = parse_node_pointer(pointer_bytes);
         if pointer.all_zero {
-            return None;
+            return IndexNodeReading::NotWalkable;
         }
         judge_location_order(&mut self.judgements, &pointer, what);
         judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &pointer, what);
@@ -1505,33 +1537,37 @@ impl<'reader> Walk<'reader> {
         );
         judge_pointer_extent_offset_is_zero(&mut self.judgements, &pointer, what);
         self.note_references_of_a_pointer_followed_by_the_walk(&pointer, 1, what);
-        let Some(unit) = read_referenced_unit(
-            self.reader,
-            &mut self.judgements,
-            &pointer.locations,
-            node_bytes(),
-            what,
-        ) else {
+        let Some((unit, placement_it_was_read_from)) = self
+            .read_a_unit_to_walk_and_the_placement_it_was_read_from(&pointer, node_bytes(), what)
+        else {
             self.walk_failures
                 .push(format!("{what} 两份都读不到对得上的"));
-            return None;
+            return IndexNodeReading::NotWalkable;
         };
-        // 「走过」照旧按第 0 条位置条目认（不跟 F-KEY 那几处改）：调用方（多层码 2 树、树表）在读之前就按第 0 条位置条目问
-        // `visited_units` 定「别的版本先走过」，读回 None 时按它分「先走过」与「走不下去」；只改这里的键，两处就按两个落点认同一件事。
-        let placement = (pointer.locations[0].device, pointer.locations[0].slot);
-        if !self.visited_units.insert(placement) {
-            self.merge_placements_below_a_unit_walked_by_an_earlier_version(placement);
-            return None;
+        if !self.visited_units.insert(placement_it_was_read_from) {
+            self.merge_placements_below_a_unit_walked_by_an_earlier_version(
+                placement_it_was_read_from,
+            );
+            return IndexNodeReading::WalkedByAnEarlierVersion {
+                placement_it_was_read_from,
+            };
         }
         if !self.judge_unit_header(&unit, 2, what) {
-            return None;
+            return IndexNodeReading::NotWalkable;
         }
-        self.judge_birth_identity_of_a_referenced_unit(&unit, &pointer, what);
-        let view = self.index_node_view_judging_a_zero_entry_width(
+        self.judge_birth_identity_of_a_referenced_unit(
+            &unit,
+            &pointer,
+            placement_it_was_read_from,
+            what,
+        );
+        let Some(view) = self.index_node_view_judging_a_zero_entry_width(
             &unit,
             entry_field_table_registration,
             what,
-        )?;
+        ) else {
+            return IndexNodeReading::NotWalkable;
+        };
         self.judgements
             .judge("I-1.3", view.tree_identifier == expected_tree, || {
                 format!(
@@ -1562,7 +1598,10 @@ impl<'reader> Walk<'reader> {
         self.judgements.judge("I-1.1", keys.is_ok(), || {
             format!("{what}：key 宽 / key 区间与条目不符（{keys:?}）")
         });
-        Some(view)
+        IndexNodeReading::WalkedFirstByThisVersion {
+            view,
+            placement_it_was_read_from,
+        }
     }
 
     fn walk_root(&mut self, record: &[u8], is_newest: bool) {
@@ -1661,12 +1700,6 @@ impl<'reader> Walk<'reader> {
             );
             return;
         }
-        let tree_table_placement = (
-            tree_table_pointer_view.locations[0].device,
-            tree_table_pointer_view.locations[0].slot,
-        );
-        let tree_table_walked_by_an_earlier_version =
-            self.visited_units.contains(&tree_table_placement);
         let tree_table = self.read_index_node(
             tree_table_pointer,
             0,
@@ -1676,7 +1709,10 @@ impl<'reader> Walk<'reader> {
             EntryFieldTableRegistration::NotRegistered,
         );
         match tree_table {
-            Some(tree_table) => self.walk_the_entries_of_a_tree_table_walked_first_by_this_version(
+            IndexNodeReading::WalkedFirstByThisVersion {
+                view: tree_table,
+                placement_it_was_read_from: tree_table_placement,
+            } => self.walk_the_entries_of_a_tree_table_walked_first_by_this_version(
                 &tree_table,
                 tree_table_placement,
                 version_name,
@@ -1684,11 +1720,14 @@ impl<'reader> Walk<'reader> {
             // 别的版本先走过这片树表（零单元发布、卸载与抬 F 的空发布照抄上一版的树表与映射根）：它下面的落点已经并进这一版
             // （`read_index_node` 里的 `merge_placements_below_a_unit_walked_by_an_earlier_version`），条目照先走那一版记下的判；
             // 映射根照样往下并（这一版也引用它，I-3.5 比相邻两版的引用集合要它，`research/prompts/m3-birth-floor-r3-main-verification.md`）。
-            None if tree_table_walked_by_an_earlier_version => {
+            // 「先走过」按读回那一份的落点认，由 `read_index_node` 交回（第 0 条那一份对不上、第 1 条那一份被先走那一版读过时也认得出）。
+            IndexNodeReading::WalkedByAnEarlierVersion {
+                placement_it_was_read_from: tree_table_placement,
+            } => {
                 self.judge_the_tree_table_entries_an_earlier_version_walked(tree_table_placement);
             }
             // 树表读不出、头用不了：走读已经断了，映射根也不走。
-            None => return,
+            IndexNodeReading::NotWalkable => return,
         }
         // 中央映射树不进树表，引用它的是那条映射根指针（根记录里的，或由记录施加出来那一版的新根段里的）：
         // I-1.3（块头树 ID 一致） 的「实际引用它的树」按那条指针头部的出生树取（D19（块指针的结构与宽度预算） 已定项 7），
@@ -1941,22 +1980,22 @@ impl<'reader> Walk<'reader> {
             }
             return SubtreeInTheWalk::NotWalkable;
         }
-        let already_walked_from_another_root = self
-            .visited_units
-            .contains(&(pointer.locations[0].device, pointer.locations[0].slot));
-        let Some(view) = self.read_index_node(
+        let (view, node_placement) = match self.read_index_node(
             pointer_bytes,
             tree.tree,
             tree.schema,
             &what,
             KeyRangeReading::SubtreeCoverageOfInternalNodesJudgedByTheCaller,
             tree.leaf_entry_width.entry_field_table_registration(),
-        ) else {
-            return if already_walked_from_another_root {
-                SubtreeInTheWalk::AlreadyWalkedFromAnotherRoot
-            } else {
-                SubtreeInTheWalk::NotWalkable
-            };
+        ) {
+            IndexNodeReading::WalkedFirstByThisVersion {
+                view,
+                placement_it_was_read_from,
+            } => (view, placement_it_was_read_from),
+            IndexNodeReading::WalkedByAnEarlierVersion { .. } => {
+                return SubtreeInTheWalk::AlreadyWalkedFromAnotherRoot;
+            }
+            IndexNodeReading::NotWalkable => return SubtreeInTheWalk::NotWalkable,
         };
         // 头里自述的 key 宽是盘上读来的一字节：与这棵树的 key 形态不等时 `read_index_node` 已经按 I-1.1 判红，这里不往下走——
         // 下面按它切子指针（`entry[key_width..key_width + 86]`）、把头里的 key 交回父节点按形态逐字段读，宽了越过条目、窄了越过 key，
@@ -2002,8 +2041,7 @@ impl<'reader> Walk<'reader> {
         let fields = |key: &[u8]| tree.schema.fields(key);
         let mut is_complete = true;
         let mut children: Vec<(Vec<u8>, SubtreeInTheWalk)> = Vec::with_capacity(view.entries.len());
-        let previous_parent_unit =
-            self.start_walking_below((pointer.locations[0].device, pointer.locations[0].slot));
+        let previous_parent_unit = self.start_walking_below(node_placement);
         for entry in &view.entries {
             let separator_key = entry[..key_width].to_vec();
             let child_pointer_bytes = &entry[key_width..key_width + node_pointer_bytes()];
@@ -2195,7 +2233,10 @@ impl<'reader> Walk<'reader> {
             }
             return;
         }
-        let Some(node) = self.read_index_node(
+        let IndexNodeReading::WalkedFirstByThisVersion {
+            view: node,
+            placement_it_was_read_from: node_placement,
+        } = self.read_index_node(
             &entry[14..100],
             tree,
             schema,
@@ -2205,7 +2246,8 @@ impl<'reader> Walk<'reader> {
                 Some(_) => EntryFieldTableRegistration::Registered,
                 None => EntryFieldTableRegistration::NotRegistered,
             },
-        ) else {
+        )
+        else {
             return;
         };
         // I-1.10：头里自述的条目宽等于这棵树登记的条目字段表宽度，**先于**按字段表解条目判
@@ -2226,18 +2268,11 @@ impl<'reader> Walk<'reader> {
             }
         }
         if kind == TREE_KIND_INODE {
-            let root_locations = parse_node_pointer(&entry[14..100]).locations;
-            self.walk_inode_root(
-                &node,
-                tree,
-                (root_locations[0].device, root_locations[0].slot),
-            );
+            self.walk_inode_root(&node, tree, node_placement);
         }
         // 墓碑树、谱系树：码 2 根之下是码 3 容器（类型段墓碑 {0, 1}、谱系 {0, 5}，D18（块里携带什么信息） 已定项 24）。
         // 这里走到每片容器、判每条内部条目的身份（墓碑树 I-9.23、谱系树 I-9.17）、容器的头与容器格式（单元算进可达与已有统计）；
         // 记录本身的那几条（I-9.18–I-9.22、I-9.24、I-9.29 与 I-9.10 墓碑那一个等号）在删除态扫描里每一版各判。
-        let node_locations = parse_node_pointer(&entry[14..100]).locations;
-        let node_placement = (node_locations[0].device, node_locations[0].slot);
         if kind == TREE_KIND_TOMBSTONE {
             self.walk_container_tree_root(
                 &node,
@@ -2421,24 +2456,19 @@ impl<'reader> Walk<'reader> {
                     // 孩子的类只认按它自己那一类的长度整份自证过的副本（F-ID-b，用户 2026-10-07 定，判决
                     // `research/prompts/m3-batch31-code-r1-main-verification.md` 第二节 K2 N1 盘序翻面 A3 那一行、第四节）：头里是码 3、
                     // 只有前 16 KiB 与位置条目里的校验和对得上的那一份不算，不随它在哪块盘翻面。
-                    // 两份都没自证过时怎么认，条款没写（交主 agent）：这一形照改之前的读法，取第一份按 16 KiB 对得上的。
+                    // 两份都没自证过的当读不出，「子单元是码 2」这一小句不判（用户 2026-10-07 定，`records/2026-10-01-删除与并发事务.md` 同日那一条）。
                     let (birth_tree, _, container, container_birth) = identity;
-                    let child_class =
-                        class_tag_of_the_first_copy_verifying_at_the_length_of_its_class(
-                            self.reader,
-                            &child,
-                        )
-                        .or_else(|| {
-                            read_unit_without_judging(self.reader, &child, node_bytes())
-                                .map(|unit| unit[UNIT_HEADER_CLASS_TAG_OFFSET])
-                        });
+                    let child_class = class_of_the_first_copy_verifying_at_the_length_of_its_class(
+                        self.reader,
+                        &child,
+                    );
                     let entry_holds = birth_tree == 0
                         && container == 0
                         && container_birth == 0
-                        && child_class.is_none_or(|class| class == 2);
+                        && child_class.is_none_or(|class| class == MappingKeyUnitClass::IndexNode);
                     self.judgements.judge(identity_invariant, entry_holds, || {
                         format!(
-                            "{name}内部条目类型段 0：孩子的类标签 {child_class:?}（要 2），身份引用的出生树 / 容器号 / 出生代 {birth_tree} / {container} / {container_birth}（要三段为 0）"
+                            "{name}内部条目类型段 0：孩子自证过的类 {child_class:?}（要码 2；两份都没自证过的不判），身份引用的出生树 / 容器号 / 出生代 {birth_tree} / {container} / {container_birth}（要三段为 0）"
                         )
                     });
                     self.walk_the_internal_node_below_a_container_tree_entry(
@@ -2566,14 +2596,18 @@ impl<'reader> Walk<'reader> {
             ));
             return;
         }
-        let Some(child_node) = self.read_index_node(
+        let IndexNodeReading::WalkedFirstByThisVersion {
+            view: child_node,
+            placement_it_was_read_from: child_placement,
+        } = self.read_index_node(
             &entry[34..120],
             tree,
             KEY_SCHEMA_CONTAINER_NUMBER,
             &format!("{name}根之下的内部节点"),
             KeyRangeReading::FirstAndLastEntries,
             EntryFieldTableRegistration::Registered,
-        ) else {
+        )
+        else {
             return;
         };
         self.judgements
@@ -2588,7 +2622,7 @@ impl<'reader> Walk<'reader> {
         {
             self.walk_container_tree_root(
                 &child_node,
-                (child.locations[0].device, child.locations[0].slot),
+                child_placement,
                 tree,
                 record_type,
                 identity_invariant,
@@ -2653,9 +2687,10 @@ impl<'reader> Walk<'reader> {
                     unit[6], child.birth_tree
                 )
             });
-        let child_placement = (child.locations[0].device, child.locations[0].slot);
         if !self.visited_units.insert(placement_it_was_read_from) {
-            self.merge_placements_below_a_unit_walked_by_an_earlier_version(child_placement);
+            self.merge_placements_below_a_unit_walked_by_an_earlier_version(
+                placement_it_was_read_from,
+            );
             return;
         }
         let container_name = format!("{name}的容器");
@@ -2666,7 +2701,12 @@ impl<'reader> Walk<'reader> {
                     read_u64(&unit, 43)
                 )
             });
-        self.judge_birth_identity_of_a_referenced_unit(&unit, child, &container_name);
+        self.judge_birth_identity_of_a_referenced_unit(
+            &unit,
+            child,
+            placement_it_was_read_from,
+            &container_name,
+        );
         // 容器格式（类标签、记录类型、记录宽与声明长度）按 I-1.7 那一套判；记录本身的不变量归删除态扫描（I-9.18 起那几条）。
         let _records = self.judge_packed_container(&unit, record_type, &container_name);
     }
@@ -2877,7 +2917,6 @@ impl<'reader> Walk<'reader> {
                     .push("inode 树的孩子两份都读不到对得上的".to_string());
                 continue;
             };
-            let child_placement = (child.locations[0].device, child.locations[0].slot);
             let subtree = match registered_entry_type {
                 RegisteredInodeInternalEntryType::LeafContainer => {
                     let header_identity = (
@@ -2895,13 +2934,7 @@ impl<'reader> Walk<'reader> {
                             unit[6]
                         )
                     });
-                    self.walk_inode_leaf_container(
-                        &unit,
-                        &child,
-                        tree,
-                        child_placement,
-                        placement_it_was_read_from,
-                    )
+                    self.walk_inode_leaf_container(&unit, &child, tree, placement_it_was_read_from)
                 }
                 RegisteredInodeInternalEntryType::InternalNode => {
                     let (birth_tree, _, container, container_birth) = identity;
@@ -2918,7 +2951,6 @@ impl<'reader> Walk<'reader> {
                         &child,
                         node.level,
                         tree,
-                        child_placement,
                         placement_it_was_read_from,
                     )
                 }
@@ -2951,22 +2983,23 @@ impl<'reader> Walk<'reader> {
     /// 一片 inode 叶容器（码 3 打包记录类型 2）：这一遍第一次走到时判树 ID（I-1.3）、出生身份（I-1.2 / I-4.2）、容器（I-1.7 等）与容器里
     /// 每条记录（I-9.13、I-9.7、I-9.4 第一句与第四句、I-9.15），交回它的容器号与装的最小、最大 inode 号；别的版本已经走过时不重读，
     /// 从 `inode_leaf_order_below_a_walked_unit` 接上它在叶序里的那一格，它下面的落点并进这一版（I-5.1）。
-    /// 「走过」按读回的那一份的落点 `placement_it_was_read_from` 认（F-KEY，[`Self::read_a_unit_to_walk_and_the_placement_it_was_read_from`]），
-    /// 别的键照旧按第 0 条位置条目的 `child_placement`。
+    /// 走过、并进前一版、叶序与诞生代号都按读回的那一份的落点 `placement_it_was_read_from` 认（F-KEY 与攻方 allrbmap，
+    /// [`Self::read_a_unit_to_walk_and_the_placement_it_was_read_from`]）。
     fn walk_inode_leaf_container(
         &mut self,
         unit: &[u8],
         child: &PointerView,
         tree: u64,
-        child_placement: (u32, u64),
         placement_it_was_read_from: (u32, u64),
     ) -> InodeSubtreeInTheWalk {
         if !self.visited_units.insert(placement_it_was_read_from) {
-            self.merge_placements_below_a_unit_walked_by_an_earlier_version(child_placement);
+            self.merge_placements_below_a_unit_walked_by_an_earlier_version(
+                placement_it_was_read_from,
+            );
             return InodeSubtreeInTheWalk {
                 leaves_in_leaf_order: self
                     .inode_leaf_order_below_a_walked_unit
-                    .get(&child_placement)
+                    .get(&placement_it_was_read_from)
                     .cloned()
                     .unwrap_or_default(),
                 walked_first_by_this_version: false,
@@ -2977,7 +3010,12 @@ impl<'reader> Walk<'reader> {
             .judge("I-1.3", read_u64(unit, 43) == tree, || {
                 format!("inode 树叶的出生树 {} 不是 {tree}", read_u64(unit, 43))
             });
-        self.judge_birth_identity_of_a_referenced_unit(unit, child, "inode 树的叶容器");
+        self.judge_birth_identity_of_a_referenced_unit(
+            unit,
+            child,
+            placement_it_was_read_from,
+            "inode 树的叶容器",
+        );
         if let Some(records) =
             self.judge_packed_container(unit, PACKED_TYPE_INODE, "inode 树的叶容器")
         {
@@ -3054,7 +3092,7 @@ impl<'reader> Walk<'reader> {
             }
         }
         self.inode_leaf_order_below_a_walked_unit
-            .insert(child_placement, leaves_in_leaf_order.clone());
+            .insert(placement_it_was_read_from, leaves_in_leaf_order.clone());
         InodeSubtreeInTheWalk {
             leaves_in_leaf_order,
             walked_first_by_this_version: true,
@@ -3066,15 +3104,14 @@ impl<'reader> Walk<'reader> {
     /// 层级 = 父层级 − 1（I-1.1）与条目宽 120（I-1.10），再往下走（[`Self::walk_inode_internal_node`]）；
     /// 别的版本已经走过时不重读，从 `inode_leaf_order_below_a_walked_unit` 接上它下面的叶序，它下面的落点并进这一版（I-5.1）。
     /// 层级 0 的码 2 节点在 inode 树里没有条款定的形状（叶是码 3 容器，I-9.1 那一行只给根留了「空的根」）：记走读失败、不往下走。
-    /// 「走过」按读回的那一份的落点 `placement_it_was_read_from` 认（F-KEY，同 [`Self::walk_inode_leaf_container`]），
-    /// 别的键（父单元、叶序、并进前一版）照旧按第 0 条位置条目的 `child_placement`。
+    /// 走过、父单元、叶序、并进前一版与诞生代号都按读回的那一份的落点 `placement_it_was_read_from` 认（F-KEY 与攻方 allrbmap，
+    /// 同 [`Self::walk_inode_leaf_container`]）。
     fn walk_inode_internal_child(
         &mut self,
         unit: &[u8],
         child: &PointerView,
         parent_level: u8,
         tree: u64,
-        child_placement: (u32, u64),
         placement_it_was_read_from: (u32, u64),
     ) -> InodeSubtreeInTheWalk {
         let not_walked_below = InodeSubtreeInTheWalk {
@@ -3082,11 +3119,13 @@ impl<'reader> Walk<'reader> {
             walked_first_by_this_version: true,
         };
         if !self.visited_units.insert(placement_it_was_read_from) {
-            self.merge_placements_below_a_unit_walked_by_an_earlier_version(child_placement);
+            self.merge_placements_below_a_unit_walked_by_an_earlier_version(
+                placement_it_was_read_from,
+            );
             return InodeSubtreeInTheWalk {
                 leaves_in_leaf_order: self
                     .inode_leaf_order_below_a_walked_unit
-                    .get(&child_placement)
+                    .get(&placement_it_was_read_from)
                     .cloned()
                     .unwrap_or_default(),
                 walked_first_by_this_version: false,
@@ -3096,7 +3135,12 @@ impl<'reader> Walk<'reader> {
         if !self.judge_unit_header(unit, 2, what) {
             return not_walked_below;
         }
-        self.judge_birth_identity_of_a_referenced_unit(unit, child, what);
+        self.judge_birth_identity_of_a_referenced_unit(
+            unit,
+            child,
+            placement_it_was_read_from,
+            what,
+        );
         let Some(view) = self.index_node_view_judging_a_zero_entry_width(
             unit,
             EntryFieldTableRegistration::Registered,
@@ -3149,7 +3193,12 @@ impl<'reader> Walk<'reader> {
             return not_walked_below;
         }
         let mut leaves_in_leaf_order = Vec::new();
-        self.walk_inode_internal_node(&view, tree, child_placement, &mut leaves_in_leaf_order);
+        self.walk_inode_internal_node(
+            &view,
+            tree,
+            placement_it_was_read_from,
+            &mut leaves_in_leaf_order,
+        );
         InodeSubtreeInTheWalk {
             leaves_in_leaf_order,
             walked_first_by_this_version: true,
@@ -3229,11 +3278,9 @@ impl<'reader> Walk<'reader> {
             );
             judge_pointer_extent_offset_is_zero(&mut self.judgements, &pointer, &pointer_what);
             self.note_references_of_a_pointer_followed_by_the_walk(&pointer, 2, &what);
-            let Some((unit, location_it_was_read_from)) =
-                crate::image::read_referenced_unit_with_the_location_it_was_read_from(
-                    self.reader,
-                    &mut self.judgements,
-                    &pointer.locations,
+            let Some((unit, placement_it_was_read_from)) = self
+                .read_a_unit_to_walk_and_the_placement_it_was_read_from(
+                    &pointer,
                     data_unit_bytes(),
                     &what,
                 )
@@ -3242,19 +3289,16 @@ impl<'reader> Walk<'reader> {
                     .push(format!("{what}两份都读不到对得上的"));
                 break;
             };
-            let page_placement = (pointer.locations[0].device, pointer.locations[0].slot);
             // 「走过」按读回、自证过的那一份的落点认（攻方 H-b，`research/prompts/m3-batch25-code-r1-opus-output.md` 第 1.3 节）：
             // 第 k 片指针的盘 0 位置条目指回走过的片、盘 1 那条完好时，按第 0 条位置条目认会把读到手的这一片当成走过、丢掉它的行。
-            // 别处的键（父单元、诞生代号那张表、并进前一版的落点表）照旧按第 0 条位置条目，与走读别的单元同一个读法。
-            let placement_it_was_read_from = (
-                location_it_was_read_from.device,
-                location_it_was_read_from.slot,
-            );
+            // 别处的键（父单元、诞生代号那张表、并进前一版的落点表）同一个落点（与走读别的单元同一个读法，攻方 allrbmap）。
             if !self.visited_units.insert(placement_it_was_read_from) {
-                self.merge_placements_below_a_unit_walked_by_an_earlier_version(page_placement);
+                self.merge_placements_below_a_unit_walked_by_an_earlier_version(
+                    placement_it_was_read_from,
+                );
                 break;
             }
-            self.walk_parent_unit = Some(page_placement);
+            self.walk_parent_unit = Some(placement_it_was_read_from);
             let Some(records) =
                 self.judge_packed_container(&unit, PACKED_TYPE_INSTANCE_TABLE, &what)
             else {
@@ -3262,8 +3306,10 @@ impl<'reader> Walk<'reader> {
             };
             self.judge_pointer_birth_tree_against_the_unit_header(&unit, &pointer, &what);
             if let Some(birth_identity) = birth_identity_in_the_header(&unit) {
-                self.birth_txg_of_walked_instance_table_pages
-                    .insert(page_placement, birth_identity.write_order.birth_txg());
+                self.birth_txg_of_walked_instance_table_pages.insert(
+                    placement_it_was_read_from,
+                    birth_identity.write_order.birth_txg(),
+                );
                 self.judge_birth_identity_against_the_pointer(birth_identity, &pointer, &what);
             }
             let identity = (
@@ -3449,15 +3495,21 @@ impl<'reader> Walk<'reader> {
                 .push("数据单元两份都读不到对得上的".to_string());
             return;
         };
-        let placement = (pointer.locations[0].device, pointer.locations[0].slot);
         if !self.visited_units.insert(placement_it_was_read_from) {
-            self.merge_placements_below_a_unit_walked_by_an_earlier_version(placement);
+            self.merge_placements_below_a_unit_walked_by_an_earlier_version(
+                placement_it_was_read_from,
+            );
             return;
         }
         if !self.judge_unit_header(&unit, 1, "数据单元") {
             return;
         }
-        self.judge_birth_identity_of_a_referenced_unit(&unit, &pointer, "数据单元");
+        self.judge_birth_identity_of_a_referenced_unit(
+            &unit,
+            &pointer,
+            placement_it_was_read_from,
+            "数据单元",
+        );
         self.judge_pointer_birth_tree_against_the_unit_header(&unit, &pointer, "数据单元");
         self.judgements
             .judge("I-1.3", read_u64(&unit, 43) == tree, || {
@@ -3560,14 +3612,18 @@ impl<'reader> Walk<'reader> {
                 index,
             } => format!("{name}（树 {tree}）层级 {cell_level} 盘 {device} 第 {index} 格的节点"),
         };
-        let Some(view) = self.read_index_node(
+        let IndexNodeReading::WalkedFirstByThisVersion {
+            view,
+            placement_it_was_read_from: node_placement,
+        } = self.read_index_node(
             pointer_bytes,
             tree,
             KEY_SCHEMA_ALLOCATION,
             &what,
             KeyRangeReading::PrescribedByThePositionJudgedByTheCaller,
             EntryFieldTableRegistration::Registered,
-        ) else {
+        )
+        else {
             return;
         };
         let entry_width = if level == 0 {
@@ -3607,9 +3663,7 @@ impl<'reader> Walk<'reader> {
         }
         let key_width = KEY_SCHEMA_ALLOCATION.width();
         let mut previous_child: Option<AllocationRecordTreeCell> = None;
-        let node_locations = parse_node_pointer(pointer_bytes).locations;
-        let previous_parent_unit =
-            self.start_walking_below((node_locations[0].device, node_locations[0].slot));
+        let previous_parent_unit = self.start_walking_below(node_placement);
         for entry in &view.entries {
             let child = allocation_record_tree_child_of_entry_key(
                 cell,
@@ -3661,7 +3715,10 @@ impl<'reader> Walk<'reader> {
                 format!("extent 树（树 {tree}）上段层级 {level} 第 {index} 格的节点")
             }
         };
-        let Some(view) = self.read_index_node(
+        let IndexNodeReading::WalkedFirstByThisVersion {
+            view,
+            placement_it_was_read_from: node_placement,
+        } = self.read_index_node(
             pointer_bytes,
             tree,
             KEY_SCHEMA_EXTENT,
@@ -3673,7 +3730,8 @@ impl<'reader> Walk<'reader> {
                 Some(_) => KeyRangeReading::PrescribedByThePositionJudgedByTheCaller,
             },
             EntryFieldTableRegistration::Registered,
-        ) else {
+        )
+        else {
             return;
         };
         let (level, index) = position.unwrap_or((view.level, 0));
@@ -3694,9 +3752,7 @@ impl<'reader> Walk<'reader> {
         let span = extent_upper_span_in_inodes(level);
         let first_inode = index.saturating_mul(span);
         let last_inode = first_inode.saturating_add(span - 1);
-        let node_locations = parse_node_pointer(pointer_bytes).locations;
-        let previous_parent_unit =
-            self.start_walking_below((node_locations[0].device, node_locations[0].slot));
+        let previous_parent_unit = self.start_walking_below(node_placement);
         if level == 0 {
             for entry in &view.entries {
                 let entry_view = extent_upper_leaf_entry_view(entry);
@@ -3775,14 +3831,18 @@ impl<'reader> Walk<'reader> {
                 format!("extent 树（树 {tree}）inode {inode} 下段层级 {level} 第 {index} 格的节点")
             }
         };
-        let Some(view) = self.read_index_node(
+        let IndexNodeReading::WalkedFirstByThisVersion {
+            view,
+            placement_it_was_read_from: node_placement,
+        } = self.read_index_node(
             pointer_bytes,
             tree,
             KEY_SCHEMA_EXTENT,
             &what,
             KeyRangeReading::PrescribedByThePositionJudgedByTheCaller,
             EntryFieldTableRegistration::Registered,
-        ) else {
+        )
+        else {
             return;
         };
         let (level, index) = position.unwrap_or((view.level, 0));
@@ -3800,9 +3860,7 @@ impl<'reader> Walk<'reader> {
         ) {
             return;
         }
-        let node_locations = parse_node_pointer(pointer_bytes).locations;
-        let previous_parent_unit =
-            self.start_walking_below((node_locations[0].device, node_locations[0].slot));
+        let previous_parent_unit = self.start_walking_below(node_placement);
         if level == 0 {
             let span = extent_lower_span_in_data_units(0);
             let first_unit = index.saturating_mul(span);
@@ -4497,50 +4555,44 @@ fn child_unit_class_other_than_the_entry_type_names(
     child: &PointerView,
     class_the_entry_type_names: Option<MappingKeyUnitClass>,
 ) -> Option<MappingKeyUnitClass> {
-    let mut class_other_than_the_entry_type_names = None;
-    for location in &child.locations {
-        let offset = location.slot * SLOT_BYTES;
-        // 头按最短的单元（码 2，16 KiB）那么长读：读者只认整块的读，32 KiB 的单元头也在这前 16 KiB 里。
-        let Some(header) = reader.read(location.device, offset, node_bytes()) else {
-            continue;
-        };
-        let class_in_the_header = MappingKeyUnitClass::of(header[UNIT_HEADER_CLASS_TAG_OFFSET]);
-        let Some(unit_bytes) = unit_bytes_of_the_class(class_in_the_header) else {
-            continue;
-        };
-        let verifies_at_the_length_of_its_class = reader
-            .read(location.device, offset, unit_bytes)
-            .is_some_and(|content| crc32_castagnoli_table(&content) == location.checksum);
-        if !verifies_at_the_length_of_its_class {
-            continue;
-        }
-        if Some(class_in_the_header) != class_the_entry_type_names
-            && class_other_than_the_entry_type_names.is_none()
-        {
-            class_other_than_the_entry_type_names = Some(class_in_the_header);
-        }
-    }
-    class_other_than_the_entry_type_names
+    child.locations.iter().find_map(|location| {
+        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)
+            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)
+    })
+}
+
+/// 一份副本（一条位置条目）按它自己那一类的长度整份自证过时交回那一类：头按最短的单元（码 2，16 KiB）那么长读取类标签
+/// （读者只认整块的读，32 KiB 的单元头也在这前 16 KiB 里），类标签是登记过的码、按那一类的长度读回的整单元校验和与位置条目对得上；
+/// 类标签没登记、读不出、校验和对不上交 None。走读里「自证过」只有这一份判据（判决 `research/prompts/m3-batch32-code-r1-main-verification.md`
+/// 第二节 K1 P2 那一行、第三节第 3 条：先前三处各抄一遍）：认孩子是别类（[`child_unit_class_other_than_the_entry_type_names`]）、
+/// 认另一份是类型段那一类（[`some_copy_of_the_child_verifies_as_the_class_the_entry_type_names`]，F-W）、
+/// 类型段 0 认「子单元是码 2」（[`class_of_the_first_copy_verifying_at_the_length_of_its_class`]，F-ID-b）都调它。只读、不判。
+fn class_of_a_copy_verifying_at_the_length_of_its_class(
+    reader: &dyn ImageReader,
+    location: &PointerLocation,
+) -> Option<MappingKeyUnitClass> {
+    let offset = location.slot * SLOT_BYTES;
+    let class_in_the_header = MappingKeyUnitClass::of(
+        reader.read(location.device, offset, node_bytes())?[UNIT_HEADER_CLASS_TAG_OFFSET],
+    );
+    let unit_bytes = unit_bytes_of_the_class(class_in_the_header)?;
+    reader
+        .read(location.device, offset, unit_bytes)
+        .filter(|content| crc32_castagnoli_table(content) == location.checksum)
+        .map(|_content| class_in_the_header)
 }
 
-/// 两条位置条目里第一份按它自己那一类的长度整份自证过的副本头里的类标签：头里的类标签是登记过的码、按那一类的长度读回的整单元校验和
-/// 与它的位置条目对得上（与 [`child_unit_class_other_than_the_entry_type_names`] 认「自证过的孩子」同一个判据）；哪一份都没自证过交 None。
+/// 两条位置条目里第一份按它自己那一类的长度整份自证过的副本是哪一类（[`class_of_a_copy_verifying_at_the_length_of_its_class`]）；
+/// 哪一份都没自证过交 None，调用方当读不出（用户 2026-10-07 定）。
 /// 墓碑树、谱系树类型段 0 那一支认「子单元是码 2」用它（F-ID-b，用户 2026-10-07 定）。只读、不判。
-fn class_tag_of_the_first_copy_verifying_at_the_length_of_its_class(
+fn class_of_the_first_copy_verifying_at_the_length_of_its_class(
     reader: &dyn ImageReader,
     child: &PointerView,
-) -> Option<u8> {
-    child.locations.iter().find_map(|location| {
-        let offset = location.slot * SLOT_BYTES;
-        // 头按最短的单元（码 2，16 KiB）那么长读：32 KiB 的单元头也在这前 16 KiB 里。
-        let class_tag_in_the_header =
-            reader.read(location.device, offset, node_bytes())?[UNIT_HEADER_CLASS_TAG_OFFSET];
-        let unit_bytes = unit_bytes_of_the_class(MappingKeyUnitClass::of(class_tag_in_the_header))?;
-        reader
-            .read(location.device, offset, unit_bytes)
-            .filter(|content| crc32_castagnoli_table(content) == location.checksum)
-            .map(|_content| class_tag_in_the_header)
-    })
+) -> Option<MappingKeyUnitClass> {
+    child
+        .locations
+        .iter()
+        .find_map(|location| class_of_a_copy_verifying_at_the_length_of_its_class(reader, location))
 }
 
 /// [`child_unit_class_other_than_the_entry_type_names`] 认出另一类之后再问一句：有没有一份副本头里的类标签就是 `class_the_entry_type_names`、
@@ -4555,16 +4607,13 @@ fn some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(
     child: &PointerView,
     class_the_entry_type_names: MappingKeyUnitClass,
 ) -> bool {
-    let unit_bytes = unit_bytes_of_the_class(class_the_entry_type_names)
-        .expect("类型段认得的那一类是码 2 或码 3，都有长度");
+    assert!(
+        unit_bytes_of_the_class(class_the_entry_type_names).is_some(),
+        "类型段认得的那一类是码 2 或码 3，都有长度"
+    );
     child.locations.iter().any(|location| {
-        reader
-            .read(location.device, location.slot * SLOT_BYTES, unit_bytes)
-            .is_some_and(|content| {
-                MappingKeyUnitClass::of(content[UNIT_HEADER_CLASS_TAG_OFFSET])
-                    == class_the_entry_type_names
-                    && crc32_castagnoli_table(&content) == location.checksum
-            })
+        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)
+            == Some(class_the_entry_type_names)
     })
 }
 
diff --git a/crates/singlefs-checker-tier/src/crash_amplification.rs b/crates/singlefs-checker-tier/src/crash_amplification.rs
index facac84..b25ad07 100644
--- a/crates/singlefs-checker-tier/src/crash_amplification.rs
+++ b/crates/singlefs-checker-tier/src/crash_amplification.rs
@@ -60,7 +60,7 @@ use singlefs_harness::gpu_memory_ledger::{
 use singlefs_harness::hexadecimal::hexadecimal_text;
 use singlefs_harness::memory_pool::{
     root_identity_of_write, root_identity_written_by, CrashImage, MemoryPool, PublishedVersion,
-    RetainedWrite, WrittenContents,
+    PublishedVersionWithoutAFile, RetainedWrite, WrittenContents,
 };
 use singlefs_harness::numeric_environment_variable::{
     number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
@@ -69,9 +69,9 @@ use singlefs_harness::segments::StepKind;
 use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};
 
 use crate::crash::{
-    judge_layer0_state_range, layer0_state_ranges_by_segment,
+    judge_layer0_state_range_against_the_version_table, layer0_state_ranges_by_segment,
     receive_until_the_senders_are_gone_with_wall_clock_lines, Layer0SegmentExpansion,
-    ReceivedOrWallClockLineDue, WallClockProgressLines,
+    Layer0VersionTable, ReceivedOrWallClockLineDue, WallClockProgressLines,
 };
 #[cfg(feature = "gpu")]
 use crate::crash::{
@@ -161,16 +161,28 @@ pub struct CrashPointSpan {
 }
 
 /// 一条录好的测试流程：崩溃枚举要的五样，加上各点（名字、设它的文件、写区间；按流程次序、首尾相接）。
+/// 版本表是两样：带文件的那几版（`versions`）与文件从那一版起没有了的那几版（`versions_without_a_file`，删除流的意图那一版，
+/// D8（核心索引结构） 已定项 16 第 4 条；文件从不消失的流是空的），两个判器都按 [`Layer0VersionTable`] 认（第 33 批 R 报告「停下交主 agent」第一项）。
+/// 同一个根两边都有是用例写错了：判的时候 [`Layer0VersionTable::new`] 停下。
 pub struct CrashFlow<'flow> {
     pub base: &'flow MemoryPool,
     pub writes: &'flow [RetainedWrite],
     pub segments: &'flow [Vec<usize>],
     pub judged_root_index: usize,
     pub versions: &'flow [PublishedVersion],
+    pub versions_without_a_file: &'flow [PublishedVersionWithoutAFile],
     pub expansion: &'flow (dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion + Sync),
     pub crash_points: Vec<CrashPointSpan>,
 }
 
+impl CrashFlow<'_> {
+    /// 判器读的版本表（[`Layer0VersionTable::new`]）。
+    #[must_use]
+    pub fn version_table(&self) -> Layer0VersionTable<'_> {
+        Layer0VersionTable::new(self.versions, self.versions_without_a_file)
+    }
+}
+
 /// 一个崩溃点在这一趟里的计划：它管哪几段、这几段的状态在整条流里的序号区间、它的复用键与路径编号、分成几块。
 pub struct PlannedCrashPoint {
     pub name: String,
@@ -431,7 +443,9 @@ enum WritesAfterThePoint {
 /// - 这个点之前的写表（不只是扇区终值）：记录核对器按写表分发布、按写的种类找根槽写与系统配置槽写
 ///   （`crash::check_records_against`、`crash::reuse_is_not_proven_illegal_by_the_reclaim_predicate`），种类、次序变了而扇区终值相同时判定会变。
 ///   按（写、整条流里的段号）逐行链起来取摘要。
-/// - 版本表：oracle 只按恢复落到的根（实例，txg）找那一版、找比它旧的版本（`crash::classified_oracle_violation_for_versions`）。
+/// - 版本表：oracle 只按恢复落到的根（实例，txg）找那一版、找比它旧的版本（`crash::classified_oracle_violation_for_the_version_table`）；
+///   「没有文件的那几版」（`CrashFlow::versions_without_a_file`）与带文件的那几版按同一个上界进键，一行都没有时一个字都不加
+///   （文件从不消失的流键不变）。
 ///   落到的根要么是基镜像里本来就有的根（基镜像里带根 magic 的扇区；基镜像自己的 journal 不越过其中最新那条，排计划时断言：
 ///   `assert_journal_recovery_of_the_base_image_lands_on_its_newest_root`），要么是写表里到这个点为止写过的根槽写，要么是 journal 重放出来的
 ///   那一版——没做完的那次发布，它的根槽写在这个点之后、是下一条。够得着的版本是（txg，实例）不超过这几样根里最大那一条的行：
@@ -649,6 +663,16 @@ impl JudgeInputsOfTheFlow {
                 hexadecimal_text(&sha256_digest(&version.content))
             ));
         }
+        for version in flow.versions_without_a_file.iter().filter(|version| {
+            highest_reachable_root.is_none_or(|highest_root| {
+                (version.checkpoint_txg, version.instance) <= highest_root
+            })
+        }) {
+            text.push_str(&format!(
+                "version_without_a_file instance={} txg={}\n",
+                version.instance.0, version.checkpoint_txg.0
+            ));
+        }
         let judged_root_slot_write = if flow.judged_root_index < first_write {
             "persisted_in_every_state".to_string()
         } else if flow.judged_root_index < last_write {
@@ -1178,12 +1202,12 @@ fn judge_one_block(
     let mut red_texts = Vec::new();
     let mut gpu_cpu_disagreements = 0u64;
     let ordinals = point.ordinals.start + local.start..point.ordinals.start + local.end;
-    let stream_state_count = judge_layer0_state_range(
+    let stream_state_count = judge_layer0_state_range_against_the_version_table(
         flow.base,
         flow.writes,
         flow.segments,
         flow.judged_root_index,
-        flow.versions,
+        flow.version_table(),
         &expansion,
         ordinals,
         &mut |ordinal, image, judgement| {
@@ -2353,7 +2377,7 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
     cards_taken_from: GpuCardsSource,
     stop_is_requested: &(dyn Fn() -> bool + Sync),
 ) -> Result<GpuJudgingOfMyBlocks, String> {
-    use crate::crash_judge_tables::{build_judge_tables, JudgeFlow};
+    use crate::crash_judge_tables::{build_judge_tables_against_the_version_table, JudgeFlow};
     let pending = pending_blocks(store, plan, version, is_mine)
         .map_err(|error| format!("读判定块失败：{error:?}"))?;
     let expansion = |segment_index: usize, segment: &[usize]| {
@@ -2367,7 +2391,10 @@ fn judge_my_blocks_on_gpu_cards_booking_the_scratch_by_where_the_cards_came_from
         versions: flow.versions,
         expansion: &expansion,
     };
-    let tables = match build_judge_tables(&judge_flow) {
+    let tables = match build_judge_tables_against_the_version_table(
+        &judge_flow,
+        flow.versions_without_a_file,
+    ) {
         Ok(tables) => tables,
         Err(refusal) => {
             return Err(format!(
@@ -5203,7 +5230,7 @@ fn print_the_placement(
     states_reused: u64,
 ) -> Result<PlacementOfThisRun, String> {
     use crate::crash_judge_dispatch::{states_in_flight_within, GpuCrashJudge, PackedJudgeTables};
-    use crate::crash_judge_tables::{build_judge_tables, JudgeFlow};
+    use crate::crash_judge_tables::{build_judge_tables_against_the_version_table, JudgeFlow};
     /// 量完宽度之后计时判几次派活那么多的状态。
     const CALIBRATION_DISPATCHES: u64 = 8;
     let pending = pending_blocks(store, plan, version, is_mine)
@@ -5223,7 +5250,10 @@ fn print_the_placement(
         versions: flow.versions,
         expansion: &expansion,
     };
-    let (states_of_the_flow, tables) = match build_judge_tables(&judge_flow) {
+    let (states_of_the_flow, tables) = match build_judge_tables_against_the_version_table(
+        &judge_flow,
+        flow.versions_without_a_file,
+    ) {
         Ok(tables) => (tables.state_count, PackedJudgeTables::of(&tables)),
         Err(refusal) => {
             return Err(format!(
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index 299bf97..1af794a 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -909,15 +909,15 @@ fn appearance_note_a_user_visible_root_of_this_version(kind: u32, tree: vec2<u32
 
 // 这一版第一个读得出的树表（walk_the_entries_of_a_tree_table_walked_first_by_this_version）：树表里每棵根指针非全零的用户可见树
 // （条目窄于 200 字节的不算，tree_table_entry_of_the_walk 交 None）记成这一版的根，连同树表落点记进走过的树表那张表
-fn appearance_note_the_user_visible_roots_of_a_tree_table_walked_first(tt: Node, tree_table_pointer: Ptr) {
+fn appearance_note_the_user_visible_roots_of_a_tree_table_walked_first(tt: Node, tree_table_device: u32, tree_table_slot: vec2<u32>) {
     if (appearance_walked_tree_table_count >= C_APPEARANCE_MAX_WALKED_TREE_TABLES) {
         undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (42u << 8u);
         return;
     }
     let table_row = C_APPEARANCE_WALKED_TREE_TABLES + appearance_walked_tree_table_count * C_APPEARANCE_WALKED_TREE_TABLE_WORDS;
-    sws(table_row, tree_table_pointer.device0);
-    sws(table_row + 1u, tree_table_pointer.slot0.x);
-    sws(table_row + 2u, tree_table_pointer.slot0.y);
+    sws(table_row, tree_table_device);
+    sws(table_row + 1u, tree_table_slot.x);
+    sws(table_row + 2u, tree_table_slot.y);
     var roots = 0u;
     if (tt.entry_width >= 200u) {
         for (var e = 0u; e < tt.entry_count; e = e + 1u) {
@@ -948,10 +948,10 @@ fn appearance_note_the_user_visible_roots_of_a_tree_table_walked_first(tt: Node,
 
 // 这一版的树表别的版本先走过（judge_the_tree_table_entries_an_earlier_version_walked）：这一版的用户可见树的根照先走那一版记下的取；
 // 先走那一版头用不了、没记下的，这里没有对象
-fn appearance_take_the_user_visible_roots_of_a_tree_table_walked_earlier(tree_table_pointer: Ptr) {
+fn appearance_take_the_user_visible_roots_of_a_tree_table_walked_earlier(tree_table_device: u32, tree_table_slot: vec2<u32>) {
     for (var t = 0u; t < appearance_walked_tree_table_count; t = t + 1u) {
         let table_row = C_APPEARANCE_WALKED_TREE_TABLES + t * C_APPEARANCE_WALKED_TREE_TABLE_WORDS;
-        if (sw(table_row) != tree_table_pointer.device0 || sw(table_row + 1u) != tree_table_pointer.slot0.x || sw(table_row + 2u) != tree_table_pointer.slot0.y) {
+        if (sw(table_row) != tree_table_device || sw(table_row + 1u) != tree_table_slot.x || sw(table_row + 2u) != tree_table_slot.y) {
             continue;
         }
         let roots = sw(table_row + 3u);
@@ -1286,6 +1286,12 @@ fn w_record_below(device: u32, slot: vec2<u32>) {
 // （w_visit_the_copy_read_back）
 var<private> w_read_back_from_the_second_location: bool;
 
+// w_read_unit_to_walk 最近一次读回、自证过的那一份的落点：这个单元在走读里的身份（walk.rs read_a_unit_to_walk_and_the_placement_it_was_read_from
+// 交回的落点；判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 1 条，攻方 allrbmap）。走过、诞生代号、并进前一版、
+// 父单元、叶序与 I-3.5 的折叠表都按它认。两份都读不出时是第一条位置条目的落点（调用方不用它）
+var<private> w_unit_device_read_back: u32;
+var<private> w_unit_slot_read_back: vec2<u32>;
+
 // image::read_referenced_unit：两条位置条目都读、都判 I-2.1，交回第一份对得上的
 fn w_read_referenced_unit(p: Ptr, unit_bytes: u32) -> Slice {
     var first = no_slice();
@@ -1312,17 +1318,35 @@ fn w_read_referenced_unit(p: Ptr, unit_bytes: u32) -> Slice {
     return first;
 }
 
-// 「走过」按 w_read_referenced_unit 刚读回、自证过的那一份的落点认（walk.rs read_a_unit_to_walk_and_the_placement_it_was_read_from，攻方 F-KEY，
-// 判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第三节第 2 条）：第一条位置条目指着这一遍先走过的别的单元、第二条那一份完好时，
-// 按第一条认会把读回来的这一份当成走过、它的头与格式一个字节都不判。并进前一版、父单元、诞生代号照旧按第一条位置条目
-fn w_visit_the_copy_read_back(p: Ptr) -> bool {
-    var device_read_from = p.device0;
-    var slot_read_from = p.slot0;
-    if (w_read_back_from_the_second_location) {
-        device_read_from = p.device1;
-        slot_read_from = p.slot1;
-    }
-    return w_visit(device_read_from, slot_read_from);
+// walk.rs read_a_unit_to_walk_and_the_placement_it_was_read_from：按位置条目读一个走读要往下走的单元（同 w_read_referenced_unit），读回、自证过的那一份的
+// 落点记进 w_unit_device_read_back / w_unit_slot_read_back；读回的是第二条那一份、I-3.5 的落点表此刻把那个落点折在第一条落点那个单元上时，
+// 改记成它自己（appearance_note_placement 记的是第一条位置条目的落点）。调用方先 w_note_pointer_references 再调它
+fn w_read_unit_to_walk(p: Ptr, unit_bytes: u32) -> Slice {
+    let s = w_read_referenced_unit(p, unit_bytes);
+    w_unit_device_read_back = p.device0;
+    w_unit_slot_read_back = p.slot0;
+    if (s.version != NONE && w_read_back_from_the_second_location) {
+        w_unit_device_read_back = p.device1;
+        w_unit_slot_read_back = p.slot1;
+        if (w_context == 0u && !(p.device1 == p.device0 && eq64(p.slot1, p.slot0))) {
+            let index = appearance_placement_index(p.device1, p.slot1);
+            if (index != NONE) {
+                let row = C_APPEARANCE_PLACEMENTS + index * C_APPEARANCE_PLACEMENT_WORDS;
+                if (sw(row + 3u) == p.device0 && sw(row + 4u) == p.slot0.x && sw(row + 5u) == p.slot0.y) {
+                    sws(row + 3u, p.device1);
+                    sws(row + 4u, p.slot1.x);
+                    sws(row + 5u, p.slot1.y);
+                }
+            }
+        }
+    }
+    return s;
+}
+
+// 「走过」按 w_read_unit_to_walk 刚读回、自证过的那一份的落点认（攻方 F-KEY，判决 `research/prompts/m3-batch31-code-r1-main-verification.md` 第三节第 2 条）：
+// 第一条位置条目指着这一遍先走过的别的单元、第二条那一份完好时，按第一条认会把读回来的这一份当成走过、它的头与格式一个字节都不判
+fn w_visit_the_copy_read_back() -> bool {
+    return w_visit(w_unit_device_read_back, w_unit_slot_read_back);
 }
 
 // I-2.5：位置条目按设备身份严格升序；整条全零的指针豁免
@@ -1392,8 +1416,8 @@ fn w_judge_unit_header(s: Slice, expected_class: u32, fsid_low: vec2<u32>) -> bo
     return true;
 }
 
-// I-1.2 / I-4.2：头里的出生身份与引用它的指针记的相同；按已发布谓词判成已发布
-fn w_judge_birth_identity(s: Slice, p: Ptr) {
+// I-1.2 / I-4.2：头里的出生身份与引用它的指针记的相同；按已发布谓词判成已发布。诞生代号记在 (unit_device, unit_slot)：读回那一份的落点
+fn w_judge_birth_identity(s: Slice, p: Ptr, unit_device: u32, unit_slot: vec2<u32>) {
     let unit_class = rd8(s.base, 6u);
     var birth_txg = vec2<u32>(0u, 0u);
     var instance = 0u;
@@ -1416,9 +1440,9 @@ fn w_judge_birth_identity(s: Slice, p: Ptr) {
     } else {
         return;
     }
-    // 记下这个单元的诞生代号（按位置条目第 0 条的落点认，与走过的单元表同一个键），再判 I-4.9
+    // 记下这个单元的诞生代号（按读回那一份的落点认，与走过的单元表同一个键），再判 I-4.9
     if (w_context == 0u) {
-        let visited_index = w_visited_index_in_the_main_walk(p.device0, p.slot0);
+        let visited_index = w_visited_index_in_the_main_walk(unit_device, unit_slot);
         if (visited_index != NONE) {
             sws(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS, 1u);
             sws(C_VISITED_BIRTHS0 + visited_index * C_VISITED_BIRTH_WORDS + 1u, birth_txg.x);
@@ -1488,6 +1512,9 @@ struct ReadNode {
     ok: bool,
     already_walked: bool,
     n: Node,
+    // 读回那一份的落点（这个节点在走读里的身份；往下走时记成父单元）。读不出时是第一条位置条目的落点
+    unit_device: u32,
+    unit_slot: vec2<u32>,
 };
 
 // walk.rs read_index_node：读一个码 2 节点并判头、出生身份、树 ID（I-1.3）与 key（I-1.1）
@@ -1498,6 +1525,8 @@ fn w_read_index_node(p: Ptr, expected_tree: vec2<u32>, schema: u32, key_range_re
     out.ok = false;
     out.already_walked = false;
     out.n = bad_node();
+    out.unit_device = p.device0;
+    out.unit_slot = p.slot0;
     if (p.all_zero) {
         return out;
     }
@@ -1505,20 +1534,23 @@ fn w_read_index_node(p: Ptr, expected_tree: vec2<u32>, schema: u32, key_range_re
     w_judge_pointer_head(p);
     w_judge_pointer_extent_offset(p);
     w_note_pointer_references(p, 1u);
-    let s = w_read_referenced_unit(p, NODE_BYTES);
+    let s = w_read_unit_to_walk(p, NODE_BYTES);
     if (s.version == NONE) {
         w_failures = w_failures + 1u;
         return out;
     }
-    if (!w_visit(p.device0, p.slot0)) {
+    // 「走过」、并进前一版、诞生代号按读回那一份的落点认，「先走过」交回调用方看（walk.rs IndexNodeReading）
+    out.unit_device = w_unit_device_read_back;
+    out.unit_slot = w_unit_slot_read_back;
+    if (!w_visit(out.unit_device, out.unit_slot)) {
         out.already_walked = true;
-        w_merge_below(p.device0, p.slot0);
+        w_merge_below(out.unit_device, out.unit_slot);
         return out;
     }
     if (!w_judge_unit_header(s, 2u, fsid_low)) {
         return out;
     }
-    w_judge_birth_identity(s, p);
+    w_judge_birth_identity(s, p, out.unit_device, out.unit_slot);
     let n = parse_index_node(s);
     if (!n.ok) {
         if (registered && entry_width_zero_with_entries(s)) {
@@ -1625,34 +1657,28 @@ fn w_walk_instance_table_chain(first: Ptr, judge_rows: bool, fsid_low: vec2<u32>
         w_judge_pointer_head(pointer);
         w_judge_pointer_extent_offset(pointer);
         w_note_pointer_references(pointer, 2u);
-        let s = w_read_referenced_unit(pointer, DATA_UNIT_BYTES);
+        let s = w_read_unit_to_walk(pointer, DATA_UNIT_BYTES);
         if (s.version == NONE) {
             w_failures = w_failures + 1u;
             break;
         }
-        // 「走过」按读回、自证过的那一份的落点认（walk.rs placement_it_was_read_from，攻方 H-b）：w_read_referenced_unit 交回第一份
-        // 对得上的，第一条位置条目那一份对不上时交回的是第二条那一份。重读第一条只取它对不对得上（表里现成的校验和，不另算 CRC）
-        let first_location_copy = read_at(pointer.device0, slot_to_bytes(pointer.slot0), DATA_UNIT_BYTES);
-        let read_from_the_first_location = first_location_copy.version != NONE && slice_crc_matches(first_location_copy, DATA_UNIT_BYTES, pointer.sum0);
-        var device_read_from = pointer.device1;
-        var slot_read_from = pointer.slot1;
-        if (read_from_the_first_location) {
-            device_read_from = pointer.device0;
-            slot_read_from = pointer.slot0;
-        }
+        // 「走过」按读回、自证过的那一份的落点认（walk.rs placement_it_was_read_from，攻方 H-b）；并进前一版、父单元、这一片的诞生代号同一个落点
+        // （攻方 allrbmap）
+        let device_read_from = w_unit_device_read_back;
+        let slot_read_from = w_unit_slot_read_back;
         if (!w_visit(device_read_from, slot_read_from)) {
-            w_merge_below(pointer.device0, pointer.slot0);
+            w_merge_below(device_read_from, slot_read_from);
             break;
         }
         w_parent_valid = true;
-        w_parent_device = pointer.device0;
-        w_parent_slot = pointer.slot0;
+        w_parent_device = device_read_from;
+        w_parent_slot = slot_read_from;
         let packed = w_judge_packed_container(s, 4u, fsid_low);
         if (!packed.ok) {
             break;
         }
         if (w_context == 0u) {
-            appearance_note_the_birth_txg_of_an_instance_table_page(pointer.device0, pointer.slot0, rd64(s.base, 73u));
+            appearance_note_the_birth_txg_of_an_instance_table_page(device_read_from, slot_read_from, rd64(s.base, 73u));
         }
         let p = packed.p;
         // I-1.12：指针的出生树等于被指单元头偏移 43 的出生树（码 3）
@@ -1856,19 +1882,21 @@ fn w_walk_extent_data_pointer(base: u32, o: u32, inode: vec2<u32>, offset: vec2<
     w_judge_pointer_head(p);
     w_judge_pointer_extent_offset(p);
     w_note_pointer_references(p, 2u);
-    let s = w_read_referenced_unit(p, DATA_UNIT_BYTES);
+    let s = w_read_unit_to_walk(p, DATA_UNIT_BYTES);
     if (s.version == NONE) {
         w_failures = w_failures + 1u;
         return;
     }
-    if (!w_visit_the_copy_read_back(p)) {
-        w_merge_below(p.device0, p.slot0);
+    let unit_device = w_unit_device_read_back;
+    let unit_slot = w_unit_slot_read_back;
+    if (!w_visit_the_copy_read_back()) {
+        w_merge_below(unit_device, unit_slot);
         return;
     }
     if (!w_judge_unit_header(s, 1u, fsid_low)) {
         return;
     }
-    w_judge_birth_identity(s, p);
+    w_judge_birth_identity(s, p, unit_device, unit_slot);
     // I-1.12：指针的出生树等于被指单元头偏移 43 的出生树（码 1 五元组里的树）
     w_judge(INVARIANT_I_1_12, eq64(p.birth_tree, rd64(s.base, 43u)));
     w_judge(INVARIANT_I_1_3, eq64(rd64(s.base, 43u), tree));
@@ -1991,8 +2019,8 @@ fn w_walk_allocation_record_tree(root_pointer: Ptr, tree: vec2<u32>, fsid_low: v
             allocation_walk_frames[depth].parent_device = w_parent_device;
             allocation_walk_frames[depth].parent_slot = w_parent_slot;
             w_parent_valid = true;
-            w_parent_device = pending_pointer.device0;
-            w_parent_slot = pending_pointer.slot0;
+            w_parent_device = read.unit_device;
+            w_parent_slot = read.unit_slot;
             depth = depth + 1u;
             enter = false;
             continue;
@@ -2116,8 +2144,8 @@ fn w_walk_extent_lower(root_pointer: Ptr, inode: vec2<u32>, tree: vec2<u32>, fsi
                 let parent_device = w_parent_device;
                 let parent_slot = w_parent_slot;
                 w_parent_valid = true;
-                w_parent_device = pending_pointer.device0;
-                w_parent_slot = pending_pointer.slot0;
+                w_parent_device = read.unit_device;
+                w_parent_slot = read.unit_slot;
                 for (var e = 0u; e < n.entry_count; e = e + 1u) {
                     let o = entry_offset(n, e);
                     let record_inode = extent_key_inode(n.base, o);
@@ -2149,8 +2177,8 @@ fn w_walk_extent_lower(root_pointer: Ptr, inode: vec2<u32>, tree: vec2<u32>, fsi
             extent_lower_frames[depth].parent_device = w_parent_device;
             extent_lower_frames[depth].parent_slot = w_parent_slot;
             w_parent_valid = true;
-            w_parent_device = pending_pointer.device0;
-            w_parent_slot = pending_pointer.slot0;
+            w_parent_device = read.unit_device;
+            w_parent_slot = read.unit_slot;
             depth = depth + 1u;
             enter = false;
             continue;
@@ -2242,8 +2270,8 @@ fn w_walk_extent_tree(root_pointer: Ptr, tree: vec2<u32>, fsid_low: vec2<u32>) {
                 let parent_device = w_parent_device;
                 let parent_slot = w_parent_slot;
                 w_parent_valid = true;
-                w_parent_device = pending_pointer.device0;
-                w_parent_slot = pending_pointer.slot0;
+                w_parent_device = read.unit_device;
+                w_parent_slot = read.unit_slot;
                 for (var e = 0u; e < n.entry_count; e = e + 1u) {
                     let o = entry_offset(n, e);
                     let entry_inode = extent_key_inode(n.base, o);
@@ -2263,8 +2291,8 @@ fn w_walk_extent_tree(root_pointer: Ptr, tree: vec2<u32>, fsid_low: vec2<u32>) {
                         w_walk_extent_lower(lower_root, entry_inode, tree, fsid_low);
                         // 下段走完把父单元换回这片叶
                         w_parent_valid = true;
-                        w_parent_device = pending_pointer.device0;
-                        w_parent_slot = pending_pointer.slot0;
+                        w_parent_device = read.unit_device;
+                        w_parent_slot = read.unit_slot;
                     } else if (well_formed && tag == 2u) {
                         w_walk_extent_data_pointer(n.base, o + 25u, entry_inode, vec2<u32>(0u, 0u), tree, fsid_low);
                     }
@@ -2290,8 +2318,8 @@ fn w_walk_extent_tree(root_pointer: Ptr, tree: vec2<u32>, fsid_low: vec2<u32>) {
             extent_upper_frames[depth].parent_device = w_parent_device;
             extent_upper_frames[depth].parent_slot = w_parent_slot;
             w_parent_valid = true;
-            w_parent_device = pending_pointer.device0;
-            w_parent_slot = pending_pointer.slot0;
+            w_parent_device = read.unit_device;
+            w_parent_slot = read.unit_slot;
             depth = depth + 1u;
             enter = false;
             continue;
@@ -2416,10 +2444,9 @@ fn w_walk_code_two_tree(root_pointer: Ptr, tree: vec2<u32>, schema: u32, fsid_lo
             if (pending_pointer.all_zero && pending_expected_level != NONE) {
                 w_failures = w_failures + 1u;
             } else {
-                let already = w_visited_contains(pending_pointer.device0, pending_pointer.slot0);
                 let read = w_read_index_node(pending_pointer, tree, schema, 1u, registered, fsid_low);
                 if (!read.ok) {
-                    if (already) {
+                    if (read.already_walked) {
                         result_kind = 1u;
                     }
                 } else {
@@ -2482,8 +2509,8 @@ fn w_walk_code_two_tree(root_pointer: Ptr, tree: vec2<u32>, schema: u32, fsid_lo
                             code2_walk_frames[depth].parent_device = w_parent_device;
                             code2_walk_frames[depth].parent_slot = w_parent_slot;
                             w_parent_valid = true;
-                            w_parent_device = pending_pointer.device0;
-                            w_parent_slot = pending_pointer.slot0;
+                            w_parent_device = read.unit_device;
+                            w_parent_slot = read.unit_slot;
                             depth = depth + 1u;
                             descended = true;
                         }
@@ -2730,7 +2757,7 @@ fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: v
             continue;
         }
         w_note_pointer_references(child, span);
-        let s = w_read_referenced_unit(child, length);
+        let s = w_read_unit_to_walk(child, length);
         if (s.version == NONE) {
             // 孩子两份都读不出：I-9.2 里不看孩子就判得了的那几小句照判（walk.rs judge_an_inode_entry_whose_child_does_not_read_back，
             // 与墓碑树、谱系树读不出孩子时同判）——类型段 0 判身份引用三段为 0，类型段 2 判子指针头部的出生树 == 身份引用的出生树
@@ -2748,16 +2775,18 @@ fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: v
             let header_identity_holds = rd8(s.base, 6u) == 3u && eq64(rd64(s.base, 43u), identity_birth_tree) && rd16(s.base, 51u) == 2u
                 && eq64(rd64(s.base, 53u), identity_container) && eq64(rd64(s.base, 61u), identity_birth) && eq64(child.birth_tree, identity_birth_tree);
             w_judge(INVARIANT_I_9_2, header_identity_holds);
-            // 叶容器（「走过」按读回那一份的落点认，F-KEY）
-            if (!w_visit_the_copy_read_back(child)) {
-                w_merge_below(child.device0, child.slot0);
+            // 叶容器（走过、并进前一版、叶序、诞生代号都按读回那一份的落点认，F-KEY 与攻方 allrbmap）
+            let leaf_device = w_unit_device_read_back;
+            let leaf_slot = w_unit_slot_read_back;
+            if (!w_visit_the_copy_read_back()) {
+                w_merge_below(leaf_device, leaf_slot);
                 if (w_context == 0u) {
-                    w_cached_leaves_below(child.device0, child.slot0, leaves_before);
+                    w_cached_leaves_below(leaf_device, leaf_slot, leaves_before);
                 }
                 continue;
             }
             w_judge(INVARIANT_I_1_3, eq64(rd64(s.base, 43u), tree));
-            w_judge_birth_identity(s, child);
+            w_judge_birth_identity(s, child, leaf_device, leaf_slot);
             let packed = w_judge_packed_container(s, 2u, fsid_low);
             if (!packed.ok) {
                 continue;
@@ -2826,9 +2855,9 @@ fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: v
                 } else {
                     // 叶序里的一格，父 = 这片叶自己（别的版本再引用它时按父找）；这一趟的叶序按写入次序读
                     let row = C_LEAVES + w_leaf_count * C_LEAF_WORDS;
-                    sws(row, child.device0);
-                    sws(row + 1u, child.slot0.x);
-                    sws(row + 2u, child.slot0.y);
+                    sws(row, leaf_device);
+                    sws(row + 1u, leaf_slot.x);
+                    sws(row + 2u, leaf_slot.y);
                     sws(row + 3u, p.container.x);
                     sws(row + 4u, p.container.y);
                     sws(row + 5u, smallest.x);
@@ -3009,7 +3038,7 @@ fn w_walk_tree_table_entry(tt: Node, e: u32, fsid_low: vec2<u32>) {
         }
     }
     if (kind == TREE_KIND_INODE) {
-        w_walk_inode_root(read.n, tree, pointer.device0, pointer.slot0, fsid_low);
+        w_walk_inode_root(read.n, tree, read.unit_device, read.unit_slot, fsid_low);
     }
     // 墓碑树、谱系树：码 2 根之下是码 3 容器（类型段墓碑 {0, 1}、谱系 {0, 5}）；走到每片容器、判它的头与容器格式
     if (kind == TREE_KIND_TOMBSTONE || kind == TREE_KIND_LINEAGE) {
@@ -3017,7 +3046,7 @@ fn w_walk_tree_table_entry(tt: Node, e: u32, fsid_low: vec2<u32>) {
         if (kind == TREE_KIND_LINEAGE) {
             container_record_type = PACKED_TYPE_LINEAGE;
         }
-        w_walk_container_tree_root(read.n, tree, pointer.device0, pointer.slot0, container_record_type, fsid_low);
+        w_walk_container_tree_root(read.n, tree, read.unit_device, read.unit_slot, container_record_type, fsid_low);
     }
 }
 
@@ -3084,7 +3113,7 @@ fn w_child_unit_class_other_than_the_entry_type_names(child: Ptr, class_the_entr
     return class_other_than_the_entry_type_names;
 }
 
-// 两条位置条目里第一份按它自己那一类的长度整份自证过的副本头里的类标签（walk.rs 的 class_tag_of_the_first_copy_verifying_at_the_length_of_its_class）：
+// 两条位置条目里第一份按它自己那一类的长度整份自证过的副本头里的类标签（walk.rs 的 class_of_the_first_copy_verifying_at_the_length_of_its_class）：
 // 头里的类标签是登记过的码、按那一类的长度读回的整单元校验和对得上；哪一份都没自证过交 NONE
 fn w_class_of_the_first_copy_verifying_at_the_length_of_its_class(child: Ptr) -> u32 {
     let first_copy = read_at(child.device0, slot_to_bytes(child.slot0), NODE_BYTES);
@@ -3112,10 +3141,13 @@ fn w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child: Pt
         && w_unit_verifies_at_the_length_of_its_class(second_copy, class_the_entry_type_names, child.sum1);
 }
 
-// 走读不往下读的孩子的一份副本与位置条目里的整单元校验和对不对得上（walk.rs 的 copy_matches_its_location_checksum_at_the_length_its_header_names）：
+// 走读不往下读的孩子的一份副本（盘 device、槽 slot）与位置条目里的整单元校验和对不对得上（walk.rs 的 copy_matches_its_location_checksum_at_the_length_its_header_names）：
 // 头按最短的单元（码 2，16 KiB）那么长读，头里的类标签是登记过的码就只按那一类的长度比；没登记的码、或头读不出，16 KiB 与 32 KiB 两种长度都对不上
-// 才算对不上（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行的改法 F-α）
-fn w_copy_matches_its_location_checksum_at_the_length_its_header_names(copy: Slice, checksum: u32) -> bool {
+// 才算对不上（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行的改法 F-α）。比 32 KiB 时另按 32 KiB 读一次
+// （CPU 那一边按每种长度各读一次）：没写过的槽 read_at 按 16 KiB 交回的全零单元只按它自己那一槽的长度用，下一槽有落点时不拿它的 32 KiB 检查字去比
+// （判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第二节 K1 / K2 P4 F-零槽那一行、第三节第 2 条，攻方改法 Z3）
+fn w_copy_matches_its_location_checksum_at_the_length_its_header_names(device: u32, slot: vec2<u32>, checksum: u32) -> bool {
+    let copy = read_at(device, slot_to_bytes(slot), NODE_BYTES);
     if (copy.version == NONE) {
         return false;
     }
@@ -3123,20 +3155,19 @@ fn w_copy_matches_its_location_checksum_at_the_length_its_header_names(copy: Sli
     if (class_in_the_header >= 1u && class_in_the_header <= 3u) {
         return w_unit_verifies_at_the_length_of_its_class(copy, class_in_the_header, checksum);
     }
-    return slice_crc_matches(copy, NODE_BYTES, checksum) || slice_crc_matches(copy, DATA_UNIT_BYTES, checksum);
+    return slice_crc_matches(copy, NODE_BYTES, checksum)
+        || slice_crc_matches(read_at(device, slot_to_bytes(slot), DATA_UNIT_BYTES), DATA_UNIT_BYTES, checksum);
 }
 
 // 走读不往下读的孩子（墓碑树、谱系树、inode 树内部条目里类型段不认得的，与孩子自证成类型段之外另一类的）：每一份副本照判 I-2.1，
 // 两份都对不上记走读失败（walk.rs 的 Walk::judge_every_copy_of_a_child_the_walk_does_not_go_into，第 29 批 H-J3a、H-J3b）。不往下走，引用由调用方先登记
 fn w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child: Ptr) {
-    let first_matches = w_copy_matches_its_location_checksum_at_the_length_its_header_names(
-        read_at(child.device0, slot_to_bytes(child.slot0), NODE_BYTES), child.sum0);
+    let first_matches = w_copy_matches_its_location_checksum_at_the_length_its_header_names(child.device0, child.slot0, child.sum0);
     if (!first_matches) {
         w_mismatches = w_mismatches + 1u;
     }
     w_judge(INVARIANT_I_2_1, first_matches);
-    let second_matches = w_copy_matches_its_location_checksum_at_the_length_its_header_names(
-        read_at(child.device1, slot_to_bytes(child.slot1), NODE_BYTES), child.sum1);
+    let second_matches = w_copy_matches_its_location_checksum_at_the_length_its_header_names(child.device1, child.slot1, child.sum1);
     if (!second_matches) {
         w_mismatches = w_mismatches + 1u;
     }
@@ -3207,15 +3238,9 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
                 continue;
             }
             // 类型段 0：身份引用的出生树、容器号、出生代三段为 0；「子单元是码 2」与 I-1.6 重合、照 I-9.2 的先例留作合取，读不出的孩子这一小句不判。
-            // 孩子的类只认按它自己那一类的长度整份自证过的副本（walk.rs 的 class_tag_of_the_first_copy_verifying_at_the_length_of_its_class，
-            // F-ID-b，用户 2026-10-07 定）；两份都没自证过时条款没写（交主 agent），照改之前取第一份按 16 KiB 对得上的
-            var class_of_the_child_of_a_type_zero_entry = w_class_of_the_first_copy_verifying_at_the_length_of_its_class(child);
-            if (class_of_the_child_of_a_type_zero_entry == NONE) {
-                let child_unit = read_unit_via_locations(child, NODE_BYTES);
-                if (child_unit.version != NONE) {
-                    class_of_the_child_of_a_type_zero_entry = rd8(child_unit.base, 6u);
-                }
-            }
+            // 孩子的类只认按它自己那一类的长度整份自证过的副本（walk.rs 的 class_of_the_first_copy_verifying_at_the_length_of_its_class，
+            // F-ID-b，用户 2026-10-07 定）；两份都没自证过的当读不出、这一小句不判（用户 2026-10-07 定）
+            let class_of_the_child_of_a_type_zero_entry = w_class_of_the_first_copy_verifying_at_the_length_of_its_class(child);
             let child_is_a_code_two_node_or_unreadable = class_of_the_child_of_a_type_zero_entry == NONE || class_of_the_child_of_a_type_zero_entry == 2u;
             w_judge(identity_invariant, is_zero64(rd64(n.base, o + 8u)) && is_zero64(rd64(n.base, o + 18u)) && is_zero64(rd64(n.base, o + 26u))
                 && child_is_a_code_two_node_or_unreadable);
@@ -3238,11 +3263,11 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
                 }
                 w_container_tree_frames[depth].n = read.n;
                 w_container_tree_frames[depth].next_entry = 0u;
-                w_container_tree_frames[depth].device = child.device0;
-                w_container_tree_frames[depth].slot = child.slot0;
+                w_container_tree_frames[depth].device = read.unit_device;
+                w_container_tree_frames[depth].slot = read.unit_slot;
                 depth = depth + 1u;
-                w_parent_device = child.device0;
-                w_parent_slot = child.slot0;
+                w_parent_device = read.unit_device;
+                w_parent_slot = read.unit_slot;
             }
             continue;
         }
@@ -3271,7 +3296,7 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
             continue;
         }
         w_note_pointer_references(child, 2u);
-        let s = w_read_referenced_unit(child, DATA_UNIT_BYTES);
+        let s = w_read_unit_to_walk(child, DATA_UNIT_BYTES);
         if (s.version == NONE) {
             // 容器两份都读不出：子指针头部出生树 == 身份引用出生树只比父条目里的两段，照判（walk.rs walk_the_container_below_a_container_tree_entry，
             // 判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 1 条）；比容器头的那几小句不判
@@ -3285,13 +3310,15 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
             && eq64(rd64(s.base, 53u), rd64(n.base, o + 18u)) && eq64(rd64(s.base, 61u), rd64(n.base, o + 26u))
             && eq64(child.birth_tree, rd64(n.base, o + 8u));
         w_judge(identity_invariant, identity_holds);
-        // 「走过」按读回那一份的落点认（F-KEY）
-        if (!w_visit_the_copy_read_back(child)) {
-            w_merge_below(child.device0, child.slot0);
+        // 走过、并进前一版、诞生代号都按读回那一份的落点认（F-KEY 与攻方 allrbmap）
+        let container_device = w_unit_device_read_back;
+        let container_slot = w_unit_slot_read_back;
+        if (!w_visit_the_copy_read_back()) {
+            w_merge_below(container_device, container_slot);
             continue;
         }
         w_judge(INVARIANT_I_1_3, eq64(rd64(s.base, 43u), tree));
-        w_judge_birth_identity(s, child);
+        w_judge_birth_identity(s, child, container_device, container_slot);
         // 容器格式（类标签、记录类型、记录宽与声明长度）按 I-1.7 判；记录本身的不变量归 I-9.17 起那几条，走读不判
         _ = w_judge_packed_container(s, record_type, fsid_low);
     }
@@ -3312,21 +3339,20 @@ fn w_walk_tree_table_and_mapping(tree_table_pointer: Ptr, mapping_root_pointer:
         w_failures = w_failures + 1u;
         return;
     }
-    let tree_table_walked_by_an_earlier_version = w_visited_contains(tree_table_pointer.device0, tree_table_pointer.slot0);
     let read = w_read_index_node(tree_table_pointer, vec2<u32>(0u, 0u), 2u, 0u, false, fsid_low);
     if (read.ok) {
         let tt = read.n;
         w_judge_tree_table_entries_ordering(tt);
         w_judge_tree_table_entry_head_identifiers(tt);
         if (w_context == 0u) {
-            appearance_note_the_user_visible_roots_of_a_tree_table_walked_first(tt, tree_table_pointer);
+            appearance_note_the_user_visible_roots_of_a_tree_table_walked_first(tt, read.unit_device, read.unit_slot);
         }
         let parent_valid = w_parent_valid;
         let parent_device = w_parent_device;
         let parent_slot = w_parent_slot;
         w_parent_valid = true;
-        w_parent_device = tree_table_pointer.device0;
-        w_parent_slot = tree_table_pointer.slot0;
+        w_parent_device = read.unit_device;
+        w_parent_slot = read.unit_slot;
         for (var e = 0u; e < tt.entry_count; e = e + 1u) {
             appearance_begin_tree_table_entry(tt, e);
             w_walk_tree_table_entry(tt, e, fsid_low);
@@ -3335,9 +3361,9 @@ fn w_walk_tree_table_and_mapping(tree_table_pointer: Ptr, mapping_root_pointer:
         w_parent_valid = parent_valid;
         w_parent_device = parent_device;
         w_parent_slot = parent_slot;
-    } else if (tree_table_walked_by_an_earlier_version) {
+    } else if (read.already_walked) {
         if (w_context == 0u) {
-            appearance_take_the_user_visible_roots_of_a_tree_table_walked_earlier(tree_table_pointer);
+            appearance_take_the_user_visible_roots_of_a_tree_table_walked_earlier(read.unit_device, read.unit_slot);
         }
     } else {
         return;
diff --git a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
index d62eee2..075eb51 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -500,10 +500,15 @@ fn read_at(device: u32, offset: vec2<u32>, length: u32) -> Slice {
 // 第 11 字是从哪一槽起整 32 KiB 不在盘内（crash_judge_tables::UnitSlotsNothingWroteAfterALocation）。读的起点按槽对齐、读的几槽都在那一段里，
 // 交回版本表最前面那两条全零单元之一（crash_judge_tables::ZERO_UNIT_VERSIONS：第 0 条 32 KiB，盘尾只装得下 16 KiB 的槽交第 1 条 16 KiB），别的交 NONE
 fn zero_unit_version_over_a_slot_nothing_wrote(before: u32, offset: vec2<u32>, length: u32) -> u32 {
-    // 起点不按槽对齐、或槽号装不进 32 位（偏移 ≥ 2⁴⁶，盘装不下）的不认
+    // 起点不按槽对齐、或槽号装不进 32 位（偏移 ≥ 2⁴⁶，盘装不下）的不认：槽号 = 偏移 >> SLOT_SHIFT 装进 32 位要偏移的高 32 位 < 2^(32 − SLOT_SHIFT)，
+    // 也就是 offset.y >> (32 − SLOT_SHIFT) 那几位之外再往下 SLOT_SHIFT 位都是 0（判决 `research/prompts/m3-batch32-code-r1-main-verification.md`
+    // 第二节 K1 / K2 P4 回绕那一行：按 offset.y >> (32 − SLOT_SHIFT) 判时槽号在 [2³², 2³⁶) 回绕成低 32 位）
     let first_slot = (offset.x >> SLOT_SHIFT) | (offset.y << (32u - SLOT_SHIFT));
-    if (((offset.x & (NODE_BYTES - 1u)) | (offset.y >> (32u - SLOT_SHIFT))) != 0u || first_slot < location_word(before, 9u)
-        || location_word(before, 10u) < first_slot + ((length + NODE_BYTES - 1u) >> SLOT_SHIFT)) {
+    let slots_read = (length + NODE_BYTES - 1u) >> SLOT_SHIFT;
+    let first_slot_past_the_gap = location_word(before, 10u);
+    // 读的几槽都在空段 [第 9 字, 第 10 字) 里：先判起点不越过空段的终点，再拿差比，不做会回绕的 first_slot + slots_read
+    if (((offset.x & (NODE_BYTES - 1u)) | (offset.y >> SLOT_SHIFT)) != 0u || first_slot < location_word(before, 9u)
+        || first_slot > first_slot_past_the_gap || first_slot_past_the_gap - first_slot < slots_read) {
         return NONE;
     }
     return select(0u, 1u, first_slot >= location_word(before, 11u));
diff --git a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
index 74cc8c9..b8f0305 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_dispatch.rs
@@ -1338,24 +1338,24 @@ mod tests {
         }
     }
 
-    /// 三个内核此刻展开之后的表达式数与离预算的余量（2026-10-07 在删除线第 32 批的仓副本上用 naga =30.0.1 现量的：`read_at` 对单元区里没写过的槽读出全零、
-    /// 走读内核「走过」按读回那一份认与类型段 0 只认自证过的副本之后）：
+    /// 三个内核此刻展开之后的表达式数与离预算的余量（2026-10-07 在删除线第 33 批的仓副本上用 naga =30.0.1 现量的：走读内核一个单元的各把键
+    /// 按读回那一份的落点认、全零单元守卫按槽号装不装得进 32 位判、F-α 那一处比 32 KiB 时另按 32 KiB 读、恢复内核的 oracle 认「没有文件」的那几版之后）：
     /// 改了哪个内核的源码，这里先红，量过新数、确认余量还够之后再改钉值，免得余量悄悄耗光、等上卡时才撞预算。
     #[test]
     fn each_kernel_keeps_its_pinned_inlined_size_and_margin_under_the_budget() {
         for (name, source, pinned_expressions, pinned_margin) in [
-            ("recovery", recovery_kernel_source(), 319_137, 80_863),
+            ("recovery", recovery_kernel_source(), 320_015, 79_985),
             (
                 "checker-scan",
                 crate::crash_judge_gpu::checker_scan_kernel_source(),
-                381_545,
-                18_455,
+                382_259,
+                17_741,
             ),
             (
                 "checker-walk",
                 crate::crash_judge_gpu::checker_walk_kernel_source(),
-                329_390,
-                70_610,
+                337_872,
+                62_128,
             ),
         ] {
             let size = super::kernel_inlined_size(&source);
diff --git a/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl b/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
index 7d0b706..15791c4 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
@@ -74,7 +74,13 @@ fn newest_persisted_root() -> NewestRoot {
     }
     return out;
 }
-// crash::classified_oracle_violation_for_versions
+// 期望版本表第 5 字（crash_judge_tables::EXPECTED_VERSION_WITH_A_FILE / EXPECTED_VERSION_WITHOUT_A_FILE）：这一版有文件，还是文件从这一版起没有了
+const EXPECTED_VERSION_WITH_A_FILE: u32 = 0u;
+const EXPECTED_VERSION_WITHOUT_A_FILE: u32 = 1u;
+
+// crash::classified_oracle_violation_for_the_version_table：落到的根是带文件的一版，读回的得是它的内容；是「文件从这一版起没有了」的一版，
+// 只许报没有文件；不在表里，报没有文件时看比它旧的最新那一项（按 (txg, 实例)）——那一项有文件就是违例，没有文件、或没有更旧的项不是
+// （文件从不消失的表上与「任一更旧版有文件」相同）；不在表里却读出了内容一律是违例
 fn classify_oracle(recovery: Recovery, newest: NewestRoot) -> u32 {
     if (!recovery.has_effective) {
         return ORACLE_NO_ROOT_CHOSEN;
@@ -85,25 +91,43 @@ fn classify_oracle(recovery: Recovery, newest: NewestRoot) -> u32 {
         }
     }
     let version = expected_version_index(recovery.effective_instance, recovery.effective_txg);
+    var version_has_a_file = false;
+    if (version != NONE) {
+        version_has_a_file = tab(T_EXPECTED, version * EXPECTED_WORDS + 5u) == EXPECTED_VERSION_WITH_A_FILE;
+    }
     if (recovery.outcome == OUTCOME_FAILED) {
         return ORACLE_READ_FAILED;
     }
     if (recovery.outcome == OUTCOME_NO_FILE) {
         if (version == NONE) {
-            // 更旧的根下面有文件？
+            // 比它旧的最新那一项下面有文件？
             let count = hdr(8u);
+            var newest_older = NONE;
+            var newest_older_txg = vec2<u32>(0u, 0u);
+            var newest_older_instance = 0u;
             for (var i = 0u; i < count; i = i + 1u) {
                 let instance = tab(T_EXPECTED, i * EXPECTED_WORDS);
                 let txg = vec2<u32>(tab(T_EXPECTED, i * EXPECTED_WORDS + 1u), tab(T_EXPECTED, i * EXPECTED_WORDS + 2u));
-                if (lt64(txg, recovery.effective_txg) || (eq64(txg, recovery.effective_txg) && instance < recovery.effective_instance)) {
-                    return ORACLE_NO_FILE_WHERE_AN_OLDER_ROOT_HAS_ONE;
+                let is_older = lt64(txg, recovery.effective_txg) || (eq64(txg, recovery.effective_txg) && instance < recovery.effective_instance);
+                let is_newer_than_the_newest_older = newest_older == NONE || lt64(newest_older_txg, txg)
+                    || (eq64(newest_older_txg, txg) && newest_older_instance < instance);
+                if (is_older && is_newer_than_the_newest_older) {
+                    newest_older = i;
+                    newest_older_txg = txg;
+                    newest_older_instance = instance;
                 }
             }
+            if (newest_older != NONE && tab(T_EXPECTED, newest_older * EXPECTED_WORDS + 5u) == EXPECTED_VERSION_WITH_A_FILE) {
+                return ORACLE_NO_FILE_WHERE_AN_OLDER_ROOT_HAS_ONE;
+            }
+            return ORACLE_NONE;
+        }
+        if (!version_has_a_file) {
             return ORACLE_NONE;
         }
         return ORACLE_NO_FILE_WHERE_THIS_ROOT_HAS_ONE;
     }
-    if (version == NONE) {
+    if (version == NONE || !version_has_a_file) {
         return ORACLE_FILE_READ_WHERE_THIS_ROOT_HAS_NONE;
     }
     if (!recovery.content_matches) {
diff --git a/crates/singlefs-checker-tier/src/crash_judge_tables.rs b/crates/singlefs-checker-tier/src/crash_judge_tables.rs
index b5bedc4..65d2c7e 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_tables.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_tables.rs
@@ -24,21 +24,22 @@ use singlefs_format::{
     ROOT_RING_REGIONS, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
 };
 use singlefs_harness::memory_pool::{
-    publishes_in, root_identity_of_write, MemoryPool, PublishedVersion, RecordStreamContinuity,
-    RetainedWrite, WrittenContents, SECTOR_BYTES,
+    publishes_in, root_identity_of_write, MemoryPool, PublishedVersion,
+    PublishedVersionWithoutAFile, RecordStreamContinuity, RetainedWrite, WrittenContents,
+    SECTOR_BYTES,
 };
 use singlefs_harness::segments::StepKind;
 use singlefs_harness::sha256::{sha256_digest, DIGEST_BYTES};
 
 use crate::crash::{
     layer0_enumeration_tables, reuse_is_not_proven_illegal_by_the_reclaim_predicate,
-    Layer0EnumerationTables, Layer0SegmentExpansion,
+    Layer0EnumerationTables, Layer0SegmentExpansion, Layer0VersionTable,
 };
 use crate::crash_facts::{persisted_writes_of_state, SegmentFact, TornImageFact};
 
 /// 表的形态版本：落点、版本、检查字或发布表的布局变了就抬它（4：版本表最前面两条全零单元、落点第 9–11 字记它之后没写过的槽，
-/// [`ZERO_UNIT_VERSIONS`]）。
-pub const JUDGE_TABLES_VERSION: u32 = 4;
+/// [`ZERO_UNIT_VERSIONS`]；5：期望版本表第 5 字记这一版有没有文件，[`EXPECTED_VERSION_WITHOUT_A_FILE`]）。
+pub const JUDGE_TABLES_VERSION: u32 = 5;
 
 /// 一个落点最多被几次写盖住：GPU 内核里组合掩码是两个 u32（版本表里一个版本的第 0 字是低 32 位、第 7 字是高 32 位）。
 /// 错位复用那条历史把系统配置 0 号槽写了 34 次，32 位装不下。
@@ -148,11 +149,20 @@ pub struct JudgePublish {
     pub unit_copy_groups: Vec<Vec<u32>>,
 }
 
-/// 一个已发布版本的内容按数据单元切开之后，每个单元该有的载荷 CRC 与声明长度（与走读读出的内容比）。
+/// 期望版本表里的一版：有文件（读回的要是 `chunks` 那几个单元），还是文件从这一版起没有了（`crash::Layer0VersionTable` 的两样，
+/// D8（核心索引结构） 已定项 16 第 4 条）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum FileOfTheExpectedVersion {
+    WithAFile,
+    WithoutAFile,
+}
+
+/// 一个已发布版本的内容按数据单元切开之后，每个单元该有的载荷 CRC 与声明长度（与走读读出的内容比）；没有文件的那一版不切，`chunks` 是空的。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct ExpectedVersionChunks {
     pub instance: u32,
     pub checkpoint_txg: u64,
+    pub file: FileOfTheExpectedVersion,
     /// （载荷区 CRC-32C，与数据单元头里偏移 101 的那 4 字节同一个覆盖范围；声明长度）。
     pub chunks: Vec<(u32, u32)>,
 }
@@ -222,7 +232,7 @@ enum Landing {
     Persisted,
 }
 
-/// 从一条流建全部表。
+/// 从一条流建全部表（文件从不消失的流：没有「没有文件的那几版」）。
 ///
 /// # Errors
 /// [`JudgeTablesRefusal`] 的各成员。
@@ -230,6 +240,23 @@ enum Landing {
 /// # Panics
 /// 枚举写表里的写越过盘尾（录制器只录落下了的写）。
 pub fn build_judge_tables(flow: &JudgeFlow<'_>) -> Result<JudgeTables, JudgeTablesRefusal> {
+    build_judge_tables_against_the_version_table(flow, &[])
+}
+
+/// 同 [`build_judge_tables`]，期望版本表另带文件从那一版起没有了的那几版（删除流，`crash_amplification::CrashFlow::versions_without_a_file`）：
+/// 恢复内核的 oracle 照 `crash::classified_oracle_violation_for_the_version_table` 认它们（`crash_judge_recovery.wgsl` 的 `classify_oracle`）。
+///
+/// # Errors
+/// [`JudgeTablesRefusal`] 的各成员。
+///
+/// # Panics
+/// 枚举写表里的写越过盘尾；同一个根既在 `flow.versions` 里又在 `versions_without_a_file` 里（[`Layer0VersionTable::new`]）。
+pub fn build_judge_tables_against_the_version_table(
+    flow: &JudgeFlow<'_>,
+    versions_without_a_file: &[PublishedVersionWithoutAFile],
+) -> Result<JudgeTables, JudgeTablesRefusal> {
+    let _version_table_without_contradictions =
+        Layer0VersionTable::new(flow.versions, versions_without_a_file);
     let tables: Layer0EnumerationTables =
         layer0_enumeration_tables(flow.base, flow.writes, flow.segments, flow.expansion);
     let enumerated = &tables.enumerated_writes;
@@ -324,7 +351,21 @@ pub fn build_judge_tables(flow: &JudgeFlow<'_>) -> Result<JudgeTables, JudgeTabl
         &location_index_by_start,
     )?;
     let publishes = judge_publishes(enumerated);
-    let expected_versions = flow.versions.iter().map(expected_version_chunks).collect();
+    let expected_versions = flow
+        .versions
+        .iter()
+        .map(expected_version_chunks)
+        .chain(
+            versions_without_a_file
+                .iter()
+                .map(|version| ExpectedVersionChunks {
+                    instance: version.instance.0,
+                    checkpoint_txg: version.checkpoint_txg.0,
+                    file: FileOfTheExpectedVersion::WithoutAFile,
+                    chunks: Vec::new(),
+                }),
+        )
+        .collect();
     Ok(JudgeTables {
         recorded_write_count: u32::try_from(flow.writes.len()).expect("写数装得进 u32"),
         enumerated_write_count: u32::try_from(enumerated.len()).expect("写数装得进 u32"),
@@ -1073,6 +1114,7 @@ fn expected_version_chunks(version: &PublishedVersion) -> ExpectedVersionChunks
     ExpectedVersionChunks {
         instance: version.instance.0,
         checkpoint_txg: version.checkpoint_txg.0,
+        file: FileOfTheExpectedVersion::WithAFile,
         chunks,
     }
 }
@@ -1357,7 +1399,10 @@ impl JudgeTables {
                 txg_hi,
                 index(chunks.len() / 2),
                 index(version.chunks.len()),
-                0,
+                match version.file {
+                    FileOfTheExpectedVersion::WithAFile => EXPECTED_VERSION_WITH_A_FILE,
+                    FileOfTheExpectedVersion::WithoutAFile => EXPECTED_VERSION_WITHOUT_A_FILE,
+                },
             ]);
             for (crc, length) in &version.chunks {
                 chunks.extend([*crc, *length]);
@@ -1432,8 +1477,12 @@ pub const VERSION_WORDS: usize = 8;
 pub const LATER_WORDS: usize = 4;
 /// 发布表每条几个 u32。
 pub const PUBLISH_WORDS: usize = 8;
-/// 期望版本表每条几个 u32。
+/// 期望版本表每条几个 u32：实例、txg 低高、单元 CRC 表起点、几个单元、有没有文件（第 5 字）。
 pub const EXPECTED_WORDS: usize = 6;
+/// 期望版本表第 5 字：这一版有文件（`crash_judge_recovery.wgsl` 同名常量）。
+pub const EXPECTED_VERSION_WITH_A_FILE: u32 = 0;
+/// 期望版本表第 5 字：文件从这一版起没有了（`crash_judge_recovery.wgsl` 同名常量）。
+pub const EXPECTED_VERSION_WITHOUT_A_FILE: u32 = 1;
 /// 头：12 个计数字段之后是盘的身份，共 32 个 u32（盘不超过 20 块）。
 pub const HEADER_WORDS: usize = 32;
 
diff --git a/crates/singlefs-checker-tier/src/crash.rs b/crates/singlefs-checker-tier/src/crash.rs
index d9609bf..f32c9a4 100644
--- a/crates/singlefs-checker-tier/src/crash.rs
+++ b/crates/singlefs-checker-tier/src/crash.rs
@@ -1418,6 +1418,38 @@ pub fn judge_layer0_state_range(
     expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
     ordinals: Range<u64>,
     judge_state: &mut dyn FnMut(u64, &CrashImage<'_>, &Layer0StateJudgement),
+) -> u64 {
+    judge_layer0_state_range_against_the_version_table(
+        base,
+        writes,
+        segments,
+        judged_root_index,
+        Layer0VersionTable::with_a_file_only(versions),
+        expansion,
+        ordinals,
+        judge_state,
+    )
+}
+
+/// 同 [`judge_layer0_state_range`]，oracle 对一张版本表判（[`classified_oracle_violation_for_the_version_table`]）：文件会消失的流
+/// （删除流：意图那一版起没有文件）经崩溃放量判的时候走这里（`crash_amplification::CrashFlow` 带「没有文件的那几版」）。
+///
+/// # Panics
+/// `ordinals` 越过整条流的状态数。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "与 judge_layer0_state_range 同一组参数，版本表换成带「没有文件的那几版」的那一张"
+)]
+#[must_use]
+pub fn judge_layer0_state_range_against_the_version_table(
+    base: &MemoryPool,
+    writes: &[RetainedWrite],
+    segments: &[Vec<usize>],
+    judged_root_index: usize,
+    versions: Layer0VersionTable<'_>,
+    expansion: &dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion,
+    ordinals: Range<u64>,
+    judge_state: &mut dyn FnMut(u64, &CrashImage<'_>, &Layer0StateJudgement),
 ) -> u64 {
     let tearable = TearableInPlaceOverwrites::of(base, writes);
     let writes_with_torn_images = WritesWithTornImages::of(base, writes, segments, &tearable);
@@ -1440,7 +1472,8 @@ pub fn judge_layer0_state_range(
             writes: enumerated_writes,
             persisted: plan.persisted_writes_of_state(ordinal),
         };
-        let judgement = judge_crash_image(&image, judged_root_index, versions);
+        let judgement =
+            judge_crash_image_against_the_version_table(&image, judged_root_index, versions);
         judge_state(ordinal, &image, &judgement);
     }
     plan.state_count
diff --git a/crates/singlefs-checker-tier/tests/common_crash_points/mod.rs b/crates/singlefs-checker-tier/tests/common_crash_points/mod.rs
index 5635f30..36abf3e 100644
--- a/crates/singlefs-checker-tier/tests/common_crash_points/mod.rs
+++ b/crates/singlefs-checker-tier/tests/common_crash_points/mod.rs
@@ -191,6 +191,36 @@ pub fn amplify_prepared_flow(
     .0
 }
 
+/// 同 [`amplify_prepared_flow`]，版本表另带文件从那一版起没有了的那几版（删除流的意图那一版，`CrashFlow::versions_without_a_file`）。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "一条流的八样各自独立，收成一个结构体只是把八个字段挪个地方"
+)]
+pub fn amplify_prepared_flow_against_the_version_table(
+    flow_name: &str,
+    base: &singlefs_harness::memory_pool::MemoryPool,
+    writes: &[singlefs_harness::memory_pool::RetainedWrite],
+    segments: &[Vec<usize>],
+    judged_root_index: usize,
+    versions: &[singlefs_harness::memory_pool::PublishedVersion],
+    versions_without_a_file: &[singlefs_harness::memory_pool::PublishedVersionWithoutAFile],
+    crash_points: Vec<singlefs_checker_tier::crash_amplification::CrashPointSpan>,
+    default_expand_up_to: usize,
+) -> singlefs_checker_tier::crash_amplification::EnvironmentRun {
+    amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
+        flow_name,
+        base,
+        writes,
+        segments,
+        judged_root_index,
+        versions,
+        versions_without_a_file,
+        crash_points,
+        default_expand_up_to,
+    )
+    .0
+}
+
 /// 同 [`amplify_prepared_flow`]，另交回打出去的那一行 `CRASH_AMPLIFICATION flow=<名> …`（只排派活计划的那一趟不打，是 `None`）。
 /// 行末接 `reuse_and_scale_fields_of_the_line` 的几个字段（复用数、全域与否、展开上限、ignore 的跳过数），被叫停的那一行也接。
 #[allow(
@@ -209,6 +239,38 @@ pub fn amplify_prepared_flow_and_hand_back_its_line(
 ) -> (
     singlefs_checker_tier::crash_amplification::EnvironmentRun,
     Option<String>,
+) {
+    amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
+        flow_name,
+        base,
+        writes,
+        segments,
+        judged_root_index,
+        versions,
+        &[],
+        crash_points,
+        default_expand_up_to,
+    )
+}
+
+/// 同 [`amplify_prepared_flow_and_hand_back_its_line`]，版本表另带「没有文件的那几版」。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "一条流的八样各自独立，收成一个结构体只是把八个字段挪个地方"
+)]
+fn amplify_prepared_flow_against_the_version_table_and_hand_back_its_line(
+    flow_name: &str,
+    base: &singlefs_harness::memory_pool::MemoryPool,
+    writes: &[singlefs_harness::memory_pool::RetainedWrite],
+    segments: &[Vec<usize>],
+    judged_root_index: usize,
+    versions: &[singlefs_harness::memory_pool::PublishedVersion],
+    versions_without_a_file: &[singlefs_harness::memory_pool::PublishedVersionWithoutAFile],
+    crash_points: Vec<singlefs_checker_tier::crash_amplification::CrashPointSpan>,
+    default_expand_up_to: usize,
+) -> (
+    singlefs_checker_tier::crash_amplification::EnvironmentRun,
+    Option<String>,
 ) {
     use singlefs_checker_tier::crash_amplification::{
         reuse_and_scale_fields_of_the_line, run_from_environment, CrashFlow,
@@ -223,6 +285,7 @@ pub fn amplify_prepared_flow_and_hand_back_its_line(
         segments,
         judged_root_index,
         versions,
+        versions_without_a_file,
         expansion: &expansion,
         crash_points,
     };
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs b/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs
index 51954c4..6ded1b8 100644
--- a/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_rollback.rs
@@ -220,6 +220,7 @@ fn the_fixed_script_through_the_rollback_is_recorded_checked_in_parallel_and_reu
         segments: &segments,
         judged_root_index,
         versions: &versions,
+        versions_without_a_file: &[],
         expansion: &expansion,
         crash_points,
     };
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs b/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs
index 25a02ec..1d4a86a 100644
--- a/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_fixed_script_through_unmount.rs
@@ -333,6 +333,7 @@ fn the_whole_fixed_script_through_the_unmount_is_recorded_checked_in_parallel_an
         segments: &segments,
         judged_root_index,
         versions: &versions,
+        versions_without_a_file: &[],
         expansion: &expansion,
         crash_points,
     };
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs b/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
index 1beb82a..c242770 100644
--- a/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_pipeline_records_checks_and_reuses.rs
@@ -76,6 +76,7 @@ impl RecordedFlow {
             segments: &self.segments,
             judged_root_index: self.judged_root_index,
             versions: &self.versions,
+            versions_without_a_file: &[],
             expansion: &expansion,
             crash_points: self.crash_points.clone(),
         }
@@ -847,6 +848,7 @@ fn a_driver_line_reports_reuse_exhaustiveness_and_depth_and_its_reuse_equals_the
         segments: &recorded.segments,
         judged_root_index: recorded.judged_root_index,
         versions: &recorded.versions,
+        versions_without_a_file: &[],
         expansion: &expansion_of_the_run,
         crash_points: recorded.crash_points.clone(),
     };
diff --git a/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs b/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs
index 3cc67fb..aed7b81 100644
--- a/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs
+++ b/crates/singlefs-checker-tier/tests/crash_amplification_two_flows_share_their_prefix.rs
@@ -79,6 +79,7 @@ impl RecordedFlow {
             segments: &self.segments,
             judged_root_index: self.judged_root_index,
             versions: &self.versions,
+            versions_without_a_file: &[],
             expansion: &expansion,
             crash_points: self.crash_points.clone(),
         }
@@ -989,6 +990,7 @@ fn root_slot_writes_whose_txgs_do_not_strictly_increase_put_every_later_write_in
             segments: &recorded.segments,
             judged_root_index: recorded.judged_root_index,
             versions: &recorded.versions,
+            versions_without_a_file: &[],
             expansion: &expansion,
             crash_points: recorded.crash_points.clone(),
         })
@@ -1111,6 +1113,7 @@ fn a_flow_whose_base_image_is_a_crash_state_with_a_journal_record_past_its_newes
         segments: &segments,
         judged_root_index,
         versions: &versions,
+        versions_without_a_file: &[],
         expansion: &expansion,
         crash_points,
     });
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs
index cec9f74..f668c53 100644
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs
@@ -1,4 +1,4 @@
-//! checker 档模块：crash、layer0_progress、writable_mount_phases
+//! checker 档模块：crash、layer0_progress、writable_mount_phases、crash_amplification
 //! 删除类发布崩在中间（K-U6 F1：接在多记录流后面 unlink 那个文件）：多记录流「取号 → 暖机 × 2 → A → B → C」之后
 //! unlink 第一个文件（C 之后三个数据单元）：意图那次发布 → 按只供测试的批大小 1 删两批 → 最后一批（D8（核心索引结构） 已定项 16），
 //! 整条录制流按 D13（验证路线） 已定项 4 枚举崩溃状态。每个状态跑恢复 + 多版本 oracle、池级 checker、记录核对器；
@@ -21,6 +21,8 @@
 
 #[path = "../../singlefs-harness/tests/common/mod.rs"]
 mod common;
+#[cfg(feature = "verdict-store")]
+mod common_crash_points;
 
 use std::num::NonZeroUsize;
 
@@ -931,3 +933,79 @@ fn full_enumeration_of_the_unlink_after_the_multi_record_publish_stream_is_exhau
     );
     assert_clean(&tally);
 }
+
+/// 崩溃放量的形态（照多记录流：取号加暖机、新池新建文件两个点由公共模块设，B、C 两次顺序写，意图与每一批各一个点；起点是第 32 批 Q 的草稿）：
+/// 按环境跑一趟，健康的流零红；配置开了 GPU 就是 GPU 判器判。版本表带「没有文件的那几版」（意图那一版，`CrashFlow::versions_without_a_file`，
+/// 第 33 批 R 报告「停下交主 agent」第一项），两批与最后一批不进表、照比它旧的最新那一项判：不带它时各批报没有文件判成违例、放量红。
+/// 展开上限默认 6。
+#[cfg(feature = "verdict-store")]
+#[test]
+fn crash_amplification_of_the_unlink_after_the_multi_record_publish_stream_is_clean() {
+    use singlefs_checker_tier::crash_amplification::CrashPointRecorder;
+    let mut pool = build_pool("crash-amplification-unlink-after-multi-record");
+    let mut recorder = CrashPointRecorder::new(pool.stream.clone(), pool.mkfs_operation_count);
+    common_crash_points::record_pool_build(&mut recorder, &pool);
+    let first = pool.output.clone();
+    let second_content = content_needing(2, 1);
+    let second = recorder.step(file!(), "sequential_write", || {
+        sequential_write(&mut pool, &second_content)
+    });
+    let third_content = content_needing(3, 2);
+    let third = recorder.step(file!(), "sequential_write", || {
+        sequential_write(&mut pool, &third_content)
+    });
+    let mut deletion_steps = vec![recorder.step(file!(), "publish_unlink_intent", || {
+        publish_the_unlink_intent(&mut pool)
+    })];
+    for _batch_index in 0..=DELETION_BATCHES_BEFORE_THE_FINAL_BATCH {
+        deletion_steps.push(recorder.step(file!(), "publish_unlink_step", || {
+            publish_the_next_unlink_step(&mut pool)
+        }));
+    }
+    assert_the_shape_of_the_deletion_steps(&deletion_steps);
+    let base = pool.memory_pool_after_mkfs();
+    let operations = pool.retained_operations();
+    let (writes, segments, stream_indexes) =
+        singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes(
+            &operations[pool.mkfs_operation_count..],
+            &geometry(),
+        );
+    let crash_points = recorder.into_crash_points(&writes, &segments, &stream_indexes);
+    let judged_root_index = writes
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .expect("根槽写");
+    let versions = vec![
+        PublishedVersion {
+            instance: first.root.instance,
+            checkpoint_txg: first.root.checkpoint_txg,
+            content: file_content(),
+        },
+        PublishedVersion {
+            instance: second.root.instance,
+            checkpoint_txg: second.root.checkpoint_txg,
+            content: second_content,
+        },
+        PublishedVersion {
+            instance: third.root.instance,
+            checkpoint_txg: third.root.checkpoint_txg,
+            content: third_content,
+        },
+    ];
+    let intent = deletion_steps.first().expect("意图那一步");
+    let versions_without_a_file = vec![PublishedVersionWithoutAFile {
+        instance: intent.root.instance,
+        checkpoint_txg: intent.root.checkpoint_txg,
+    }];
+    common_crash_points::amplify_prepared_flow_against_the_version_table(
+        "unlink-after-multi-record-publish",
+        &base,
+        &writes,
+        &segments,
+        judged_root_index,
+        &versions,
+        &versions_without_a_file,
+        crash_points,
+        6,
+    );
+}
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 6043d65..6ce3719 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -49,6 +49,10 @@
 //! 第 32 批（删除线）：墓碑容器带记录宽违例、墓碑树根条目第 0 条位置条目指着先走过的 inode 树根（F-KEY：「走过」按读回那一份的落点认）、
 //! 类型段 0 的条目两份副本一份只按前 16 KiB 对得上而头里是码 3、一份是码 2（F-ID-b：只认自证过的副本）、类型段 7 的条目第 1 条位置条目指着流里没写过的槽、
 //! 校验和是全零 16 KiB 的 CRC（F-零槽：GPU 读者对没写过的槽读出全零）各一条流，都整份比。
+//! 第 33 批（删除线）：回退之后共享的叶容器诞生代号抬到回退那一版、盘 0 那一份写坏（一个单元的各把键按读回那一份的落点认，allrbmap）、
+//! 类型段 7 的条目第 1 条位置条目的槽号加 2³² 与槽 2³² − 1（全零单元守卫按槽号装不装得进 32 位判）、指着没写过而下一槽写过的槽、校验和是 32 KiB
+//! 全零的 CRC（F-α 比 32 KiB 时另按 32 KiB 读）、类型段 0 的条目两份都没自证过（当读不出）、删完的流带「没有文件的那几版」（恢复内核的 oracle）、
+//! 由记录施加出来的版与下一次挂载先施加的那一版同在（C744 ①）各一条流，都整份比。
 //! 第 35 批：由记录造出来的两类版本取带末条标志那条记录的新根段（三条记录的发布第一条写成上一版的根：下一次挂载那一版叠两条记录罩住同一个槽、
 //! 挂载之后由记录施加出来的那一版各一条），树表指针整条全零照 core 跟过去（根与它的记录、由记录施加出来的那一版的记录各一条），都整份比。
 //! 第 36 批：下一次挂载先施加的那一版照 I-4.1 那一份重放施加的第一次发布认（改法 R1）——三条记录的发布首条没落（断号）、
@@ -91,8 +95,8 @@ use common_deletion_in_progress::{
 use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
 use singlefs_checker::walk::check_pool_image;
 use singlefs_checker_tier::crash::{
-    judge_layer0_state_range, layer0_plan_state_count, Layer0OracleViolationKind,
-    Layer0SegmentExpansion, Layer0StateJudgement,
+    judge_layer0_state_range_against_the_version_table, layer0_plan_state_count,
+    Layer0OracleViolationKind, Layer0SegmentExpansion, Layer0StateJudgement, Layer0VersionTable,
 };
 use singlefs_checker_tier::crash_amplification::{
     gpu_recovery_words, gpu_verdict_items, judge_my_blocks_on_gpu_cards, plan_crash_points,
@@ -108,7 +112,9 @@ use singlefs_checker_tier::crash_judge_gpu::{
     POOL_CHECKER_REACHED_WORD, POOL_CHECKER_UNDECIDABLE_WORD, POOL_CHECKER_VIOLATED_BIT_CAPACITY,
     POOL_CHECKER_VIOLATED_FIRST_WORD, POOL_CHECKER_VIOLATED_WORDS, VERDICT_WORDS,
 };
-use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
+use singlefs_checker_tier::crash_judge_tables::{
+    build_judge_tables, build_judge_tables_against_the_version_table, JudgeFlow,
+};
 use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::{
     first_usable_card_or_report_not_run, gpu_card_pool_or_report_not_run,
     usable_cards_or_report_not_run, GpuCardPool, CARD_QUOTA_ENVIRONMENT_VARIABLE,
@@ -130,8 +136,8 @@ use singlefs_core::transaction::{
 };
 use singlefs_core::unlink::UnlinkBatchSize;
 use singlefs_harness::memory_pool::{
-    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
-    WrittenContents,
+    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion,
+    PublishedVersionWithoutAFile, RetainedWrite, WrittenContents,
 };
 use singlefs_harness::numeric_environment_variable::{
     number_from_the_variable_value_panicking_when_unreadable, process_environment_variable,
@@ -1996,6 +2002,47 @@ fn compare_on_every_state_of_these_segments(
     flow: &RecordedFlow,
     segments: &[Vec<usize>],
     expansion: &(dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion + Sync),
+) -> Comparison {
+    compare_on_every_state_of_these_segments_against_the_version_table(
+        cards,
+        flow,
+        &[],
+        segments,
+        expansion,
+    )
+}
+
+/// 同 [`compare_on_every_state`]，版本表另带文件从那一版起没有了的那几版（删除流的意图那一版）：GPU 的期望版本表
+/// （`crash_judge_tables::build_judge_tables_against_the_version_table`）与 CPU 的 oracle（`crash::Layer0VersionTable`）都认它们。
+fn compare_on_every_state_against_the_version_table(
+    cards: &'static GpuCardPool,
+    flow: &RecordedFlow,
+    versions_without_a_file: &[PublishedVersionWithoutAFile],
+    limit: usize,
+) -> Comparison {
+    let expansion = move |_segment_index: usize, segment: &[usize]| {
+        if segment.len() <= limit {
+            Layer0SegmentExpansion::EveryProperSubset
+        } else {
+            Layer0SegmentExpansion::NotExpanded
+        }
+    };
+    compare_on_every_state_of_these_segments_against_the_version_table(
+        cards,
+        flow,
+        versions_without_a_file,
+        &flow.segments,
+        &expansion,
+    )
+}
+
+/// [`compare_on_every_state_of_these_segments`] 的本体，版本表照 `versions_without_a_file` 另带「没有文件的那几版」。
+fn compare_on_every_state_of_these_segments_against_the_version_table(
+    cards: &'static GpuCardPool,
+    flow: &RecordedFlow,
+    versions_without_a_file: &[PublishedVersionWithoutAFile],
+    segments: &[Vec<usize>],
+    expansion: &(dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion + Sync),
 ) -> Comparison {
     let judge_flow = JudgeFlow {
         base: &flow.base,
@@ -2006,7 +2053,8 @@ fn compare_on_every_state_of_these_segments(
         expansion,
     };
     let tables_started = Instant::now();
-    let tables = build_judge_tables(&judge_flow).expect("建得出表");
+    let tables = build_judge_tables_against_the_version_table(&judge_flow, versions_without_a_file)
+        .expect("建得出表");
     let state_count = layer0_plan_state_count(&flow.base, &flow.writes, segments, expansion);
     assert_eq!(tables.state_count, state_count);
     let words = tables.words();
@@ -2059,12 +2107,15 @@ fn compare_on_every_state_of_these_segments(
                         }
                         let range = index * slice..((index + 1) * slice).min(state_count);
                         let mut judged: Vec<JudgedOnCpu> = Vec::new();
-                        let _state_count = judge_layer0_state_range(
+                        let _state_count = judge_layer0_state_range_against_the_version_table(
                             &flow_reference.base,
                             &flow_reference.writes,
                             segments,
                             flow_reference.judged_root_index,
-                            &flow_reference.versions,
+                            Layer0VersionTable::new(
+                                &flow_reference.versions,
+                                versions_without_a_file,
+                            ),
                             expansion,
                             range,
                             &mut |_ordinal, _image, judgement| {
@@ -2671,6 +2722,7 @@ fn a_flow_whose_judge_tables_cannot_be_built_is_refused_and_not_judged_on_the_cp
         segments: &recorded.segments,
         judged_root_index: recorded.judged_root_index,
         versions: &recorded.versions,
+        versions_without_a_file: &[],
         expansion: &expansion,
         crash_points: vec![CrashPointSpan {
             name: "every_write".to_string(),
@@ -2739,6 +2791,7 @@ fn the_card_stops_taking_blocks_when_asked_and_the_next_run_judges_only_the_rest
         segments: &recorded.segments,
         judged_root_index: recorded.judged_root_index,
         versions: &recorded.versions,
+        versions_without_a_file: &[],
         expansion: &expansion,
         crash_points: recorded
             .segments
@@ -10613,3 +10666,299 @@ fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregiste
         &["I-9.23", "I-3.1"],
     );
 }
+
+/// 回退流上 A（txg 3）那片 inode 叶容器（码 3、打包记录类型 2、头里诞生代号 3：回退那一版 D 照抄 A 的 inode 树，两版经同一个树根引用它）。
+fn is_the_inode_leaf_container_born_at_txg_three(bytes: &[u8]) -> bool {
+    const PACKED_UNIT_BIRTH_TXG_OFFSET: usize = 73;
+    unit_class(bytes) == 3
+        && packed_record_type(bytes) == 2
+        && u64::from_le_bytes(
+            bytes[PACKED_UNIT_BIRTH_TXG_OFFSET..PACKED_UNIT_BIRTH_TXG_OFFSET + 8]
+                .try_into()
+                .expect("8 字节"),
+        ) == 3
+}
+
+/// 分叉形（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第二节 K1 / K2 P1 那一行③、第三节第 1 条；攻方
+/// `research/prompts/m3-batch32-code-r1-opus-output.md` 第 1.4 节 `raise_birth=true corrupt_loc0=true direct=false`，录成流）：
+/// 固定脚本到回退 D 的流上，A 那片 inode 叶容器 L 两份头里的诞生代号 3 → 9（D 的 txg，重封、沿引用链补到根：A、D 两版照旧经同一个树根引用 L），
+/// 再把盘序小的那一份写成别的字节（不补校验和，那个槽上没有别的单元）。harness 坏镜像
+/// `a_shared_leaf_container_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device` 同一形。
+fn record_rollback_whose_shared_leaf_container_is_born_after_the_target_with_its_first_copy_garbled(
+    tag: &str,
+) -> RecordedFlow {
+    const ROLLBACK_PUBLISH_TXG: u64 = 9;
+    let mut flow = record_fixed_script_through_rollback(tag);
+    flow.name = "rollback-shared-leaf-container-born-after-the-target-first-copy-garbled";
+    let name = flow.name;
+    let leaf_copies = last_unit_copies(
+        &flow,
+        name,
+        &is_the_inode_leaf_container_born_at_txg_three,
+        false,
+    );
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &leaf_copies, &|bytes| {
+        bytes[73..81].copy_from_slice(&ROLLBACK_PUBLISH_TXG.to_le_bytes());
+    });
+    let (first_copy, _second_copy) = copies_in_device_order(&flow, &leaf_copies);
+    let WrittenContents::Bytes(first) = &mut flow.writes[first_copy].contents else {
+        panic!("单元写是普通写");
+    };
+    first.fill(0xa5);
+    flow
+}
+
+/// 第 33 批 allrbmap：L 盘 0 那一份读不出、读回的是盘 1 那一份。CPU 池级 checker 的诞生代号、并进前一版与 I-3.5 的折叠表都按读回那一份的落点认，
+/// A 那条根（txg 3，不是最新的候选根）引用 L、L 读得出的那一份诞生代号 9 > 3，红 I-4.9；D 引用、C 不引用的 inode 树根那一格子树里 L 的诞生代号
+/// 大于树根指针的出生 txg，红 I-3.5。GPU 走读内核同改（`w_read_unit_to_walk` 记下读回那一份的落点），每个状态逐位相同
+/// （改之前 GPU 把诞生代号记在盘 0 那个落点的走过表行上、那一行不存在，两条都漏）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_shared_leaf_container_born_after_the_rollback_target_reads_back_from_its_second_copy(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_shared_leaf_container_born_after_the_rollback_target_reads_back_from_its_second_copy",
+        &record_rollback_whose_shared_leaf_container_is_born_after_the_target_with_its_first_copy_garbled(
+            "crash-judge-gpu-i49-shared-leaf-container-born-after-the-target-first-copy-garbled",
+        ),
+        &["I-4.9", "I-3.5"],
+    );
+}
+
+/// 两份副本里离盘尾近、槽号装不进 32 位的位置：槽号加 2³²（`zero_unit_version_over_a_slot_nothing_wrote` 的守卫按 `offset.y >> (32 − SLOT_SHIFT)` 判时
+/// 落在 [2³², 2³⁶) 的槽号回绕成低 32 位）。
+const SLOTS_IN_THIRTY_TWO_BITS: u64 = 1 << 32;
+
+/// P4 回绕（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第二节 K1 / K2 P4 回绕那一行、第三节第 2 条；正推
+/// `research/prompts/m3-batch32-code-r1-sonnet-output.md` 第八节第 1 条）：同 F-零槽那条流
+/// （[`record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_a_slot_nothing_wrote`]），
+/// 第 1 条位置条目的槽写成 `slot`、整单元校验和写成 16 KiB 全零的 CRC。
+fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_the_slot(
+    tag: &str,
+    name: &'static str,
+    slot: u64,
+) -> RecordedFlow {
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
+            slot,
+            checksum_of_sixteen_zero_kibibytes,
+        );
+    });
+    flow
+}
+
+/// P4 回绕：槽号是 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`] 加 2³²。CPU 的稀疏读者越过盘尾读不到，那一份 I-2.1 红（I-4.8、I-7.4 跟着）；
+/// GPU `read_at` 的全零单元守卫按 `SLOT_SHIFT` 判、拒槽号 ≥ 2³²，同样读不到，每个状态逐位相同（守卫按 `offset.y >> (32 − SLOT_SHIFT)` 判时
+/// 回绕成槽 190000、交回全零单元，I-2.1 成立，少红三条）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_past_thirty_two_bits_with_the_checksum_of_zeros(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_past_thirty_two_bits_with_the_checksum_of_zeros",
+        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_the_slot(
+            "crash-judge-gpu-i21-tombstone-root-entry-type-seven-second-location-past-thirty-two-bits",
+            "deletion-tombstone-root-entry-type-seven-second-location-naming-a-slot-past-thirty-two-bits",
+            SLOTS_IN_THIRTY_TWO_BITS + SLOT_NOTHING_IN_THE_FLOW_WROTE,
+        ),
+        &["I-2.1", "I-9.23"],
+    );
+}
+
+/// P4 回绕：槽号是 2³² − 1（装得进 32 位，`first_slot + 读的槽数` 那一加在 32 位里回绕成 0）。CPU 越过盘尾读不到，I-2.1 红；GPU 先判起点
+/// 不越过空段终点、再拿差比，同样读不到，每个状态逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_in_thirty_two_bits_with_the_checksum_of_zeros(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_in_thirty_two_bits_with_the_checksum_of_zeros",
+        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_whose_second_location_names_the_slot(
+            "crash-judge-gpu-i21-tombstone-root-entry-type-seven-second-location-last-slot-in-thirty-two-bits",
+            "deletion-tombstone-root-entry-type-seven-second-location-naming-the-last-slot-in-thirty-two-bits",
+            SLOTS_IN_THIRTY_TWO_BITS - 1,
+        ),
+        &["I-2.1", "I-9.23"],
+    );
+}
+
+/// 攻方那一形（`research/prompts/m3-batch32-code-r1-opus-output.md` 第二节 `P4 p4 type=7 next_written=true sum=zero32`，录成流）：
+/// 同 F-零槽那条流，第 1 条位置条目指着 [`SLOT_NOTHING_IN_THE_FLOW_WROTE`]、整单元校验和写成 32 KiB 全零的 CRC；它的下一槽在两块盘的基镜像里
+/// 写进 16 KiB 非零字节（流里谁都没写过，每个状态都在）。
+fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_naming_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes(
+    tag: &str,
+) -> RecordedFlow {
+    let name = "deletion-tombstone-root-entry-type-seven-second-location-naming-a-slot-nothing-wrote-before-a-written-slot";
+    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
+    assert_nothing_in_the_flow_wrote_the_slot(&flow, SLOT_NOTHING_IN_THE_FLOW_WROTE);
+    let next_slot_offset = (SLOT_NOTHING_IN_THE_FLOW_WROTE + 1) * SLOT_BYTES;
+    let bytes_of_the_next_slot = vec![0x5au8; usize::try_from(SLOT_BYTES).expect("16384")];
+    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+        flow.base.devices.get_mut(&device).expect("盘在池里").write(
+            singlefs_core::address::DeviceOffsetInBytes(next_slot_offset),
+            &bytes_of_the_next_slot,
+        );
+    }
+    let checksum_of_thirty_two_zero_kibibytes =
+        crc32c(&vec![0u8; usize::try_from(2 * SLOT_BYTES).expect("32768")]);
+    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
+        set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
+        set_the_slot_and_checksum_of_a_location_in_the_first_entry(
+            bytes,
+            POINTER_LOCATION_ENTRY_OFFSETS[1],
+            SLOT_NOTHING_IN_THE_FLOW_WROTE,
+            checksum_of_thirty_two_zero_kibibytes,
+        );
+    });
+    flow
+}
+
+/// 攻方那一形：CPU 头里类标签 0 不登记，按 16 KiB 读出全零、按 32 KiB 读出「16 KiB 零 + 下一槽的字节」，两种长度都对不上 32 KiB 全零的 CRC，I-2.1 红。
+/// GPU F-α 那一处比 32 KiB 时另按 32 KiB 读一次（空段只装得下一槽，读不到），同样对不上，每个状态逐位相同（拿按 16 KiB 读回的全零切片的 32 KiB
+/// 检查字去比时判对得上，少红 I-2.1、I-4.8、I-7.4）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes",
+        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_naming_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes(
+            "crash-judge-gpu-i21-tombstone-root-entry-type-seven-slot-nothing-wrote-before-a-written-slot",
+        ),
+        &["I-2.1", "I-9.23"],
+    );
+}
+
+fn leave_the_records_before_the_mount_as_they_are(_flow: &mut RecordedFlow) {}
+
+/// C744 ①（`.claude/kb/checks-owed.md` C744；正推 `research/prompts/m3-batch10o-code-r1-sonnet-output.md` K3-2 第 1 类）：三个数据单元那次发布
+/// （txg 4）只落了三条记录、根槽没落，新进程可写挂载施加它、取号 2、写行、暖机，再以实例 2 覆盖写一次（记录照旧，
+/// [`record_three_unit_file_applied_only_by_its_records_after_a_mount`]）。挂载写了实例 2 的行、覆盖写的记录落了而根槽没落的状态上，
+/// txg 4 是由记录施加出来的那一版、覆盖写那一版是下一次挂载先施加的那一版，两样同在（推的，读录制的次序）：扫描内核
+/// `judge_allocation_records_disjoint` 与 `judge_allocation_generations_against_unit_births` 的末圈与前两段都走到。每个状态 GPU 与 CPU 逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_on_states_holding_a_version_applied_by_its_records_beside_the_version_the_next_mount_applies_first(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_on_states_holding_a_version_applied_by_its_records_beside_the_version_the_next_mount_applies_first",
+        &record_three_unit_file_applied_only_by_its_records_after_a_mount(
+            "crash-judge-gpu-c744-version-applied-by-records-beside-the-next-mount-version",
+            "three-unit-file-applied-only-by-its-records-beside-the-version-the-next-mount-applies-first",
+            &leave_the_records_before_the_mount_as_they_are,
+        ),
+        &[],
+    );
+}
+
+/// F-ID-b 两份都没自证过（用户 2026-10-07 定「当读不出、这一小句不判」；判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 3 条；
+/// harness 坏镜像 `a_tombstone_root_entry_of_type_zero_over_two_copies_that_do_not_verify_leaves_the_tombstone_entry_invariant_holding_on_either_device`
+/// 位置条目 0 是 `C16`、1 是 `BAD` 那一格，录成流）：删除中间态的流上墓碑树根第一条条目类型段 1 → 0、身份三段清零，子指针第 0 条位置条目的
+/// 整单元校验和换成墓碑容器前 16 KiB 的 CRC（头里是码 3、按 32 KiB 自证不过），第 1 条换成容器整单元 CRC 翻几位，重封到根。
+fn record_deletion_with_the_tombstone_root_entry_of_type_zero_over_two_copies_that_do_not_verify(
+    tag: &str,
+) -> RecordedFlow {
+    let name = "deletion-tombstone-root-entry-type-zero-two-copies-that-do-not-verify";
+    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
+    let container_copies = last_unit_copies(&flow, name, &is_a_tombstone_container, false);
+    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
+    assert_the_first_entry_names_the_unit_of(
+        &flow,
+        root_copies[0],
+        &container_copies,
+        "墓碑树根第一条条目指着流里最后写的那片墓碑容器",
+    );
+    let container = flow.writes[container_copies[0]]
+        .bytes()
+        .expect("单元写带字节")
+        .to_vec();
+    assert_eq!(unit_class(&container), 3, "墓碑容器是码 3");
+    let checksum_of_the_first_sixteen_kibibytes =
+        crc32c(&container[..usize::try_from(SLOT_BYTES).expect("16384")]);
+    let wrong_checksum_of_the_container =
+        crc32c(&container) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
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
+            wrong_checksum_of_the_container,
+        );
+    });
+    flow
+}
+
+/// 第 33 批 F-ID-b 两份都没自证过：CPU 池级 checker 当读不出、「子单元是码 2」那一小句不判，I-9.23 不红；F-L0 每份照判 I-2.1（码 3 按 32 KiB 对不上、
+/// 翻过位的对不上），红 I-2.1。GPU 走读内核去掉「照改之前取第一份按 16 KiB 对得上的」那一读，每个状态逐位相同（留着它的话 GPU 读出码 3、红 I-9.23）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_child_whose_two_copies_do_not_verify(
+) {
+    let Some(cards) = gpu_card_pool_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_child_whose_two_copies_do_not_verify",
+    ) else {
+        return;
+    };
+    let flow = record_deletion_with_the_tombstone_root_entry_of_type_zero_over_two_copies_that_do_not_verify(
+        "crash-judge-gpu-i923-tombstone-root-entry-type-zero-two-copies-that-do-not-verify",
+    );
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
+    assert!(
+        comparison.cpu_checker_red_invariants.contains("I-2.1")
+            && !comparison.cpu_checker_red_invariants.contains("I-9.23"),
+        "流 {} 上 CPU 红 I-2.1、一个状态都不红 I-9.23（红过的：{:?}）",
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
+/// 删完的流（[`record_new_pool_through_the_last_deletion_batch`]）带「没有文件的那几版」：意图那一版（txg 5）起没有文件，三批（txg 6–8）不进表、
+/// 照比它旧的最新那一项（意图那一版）判（D8（核心索引结构） 已定项 16 第 4 条；第 33 批 R 报告「停下交主 agent」第二项）。
+/// 恢复落到 txg 5–8 报没有文件的状态上 CPU 的 oracle 一个违例都不记；GPU 恢复内核的期望版本表第 5 字记这一版有没有文件、`classify_oracle` 照认，
+/// 每个状态逐位相同（不认的话这几版报没有文件判成「更旧的根下面有文件」）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_on_the_deleted_flow_against_a_version_table_without_a_file_from_the_intent_on(
+) {
+    let Some(cards) = gpu_card_pool_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_on_the_deleted_flow_against_a_version_table_without_a_file_from_the_intent_on",
+    ) else {
+        return;
+    };
+    let flow = record_new_pool_through_the_last_deletion_batch(
+        "crash-judge-gpu-oracle-deleted-flow-without-a-file-from-the-intent-on",
+        "deleted-flow-against-a-version-table-without-a-file-from-the-intent-on",
+    );
+    let versions_without_a_file = [PublishedVersionWithoutAFile {
+        instance: InstanceGeneration(1),
+        checkpoint_txg: CheckpointTxg(5),
+    }];
+    let comparison = compare_on_every_state_against_the_version_table(
+        cards,
+        &flow,
+        &versions_without_a_file,
+        expand_up_to(),
+    );
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
+    );
+}
diff --git a/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs b/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs
index 64e6191..d033ba8 100644
--- a/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs
+++ b/crates/singlefs-checker-tier/tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs
@@ -225,6 +225,7 @@ fn the_pipeline_with_gpu_unit_checks_reports_no_red_and_no_disagreement_on_the_h
         segments: &recorded.segments,
         judged_root_index: recorded.judged_root_index,
         versions: &recorded.versions,
+        versions_without_a_file: &[],
         expansion: &expansion,
         crash_points: vec![
             CrashPointSpan {
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 88d4b50..75357e4 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -13399,12 +13399,42 @@ fn image_whose_first_root_entry_names_a_walked_node_beside_a_container_with_a_wr
     kind_walked_before: u16,
     location_index: usize,
 ) -> MemoryPool {
-    let mut image = clean.clone();
-    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
-        &mut image,
+    image_whose_first_root_entry_names_a_walked_node_beside_a_container(
+        clean,
         kind,
-        record_width_one_below(record_bytes),
-    );
+        RecordWidthOfTheFirstContainer::OneBelowTheRegistered { record_bytes },
+        kind_walked_before,
+        location_index,
+    )
+}
+
+/// 种类 `kind` 那棵树根第一条条目指的那片容器头里的记录宽：照登记的不动，或比登记的 `record_bytes` 少 1（I-1.7 违例）。
+#[derive(Clone, Copy, Debug)]
+enum RecordWidthOfTheFirstContainer {
+    AsRegistered,
+    OneBelowTheRegistered { record_bytes: u64 },
+}
+
+/// 同 [`image_whose_first_root_entry_names_a_walked_node_beside_a_container_with_a_wrong_record_width`]，容器的记录宽照 `record_width`
+/// （攻方 `research/prompts/m3-batch32-code-r1-opus-output.md` 第 1.1 节 O3 族的 `variant=bad` / `variant=good`）。
+fn image_whose_first_root_entry_names_a_walked_node_beside_a_container(
+    clean: &MemoryPool,
+    kind: u16,
+    record_width: RecordWidthOfTheFirstContainer,
+    kind_walked_before: u16,
+    location_index: usize,
+) -> MemoryPool {
+    let mut image = clean.clone();
+    match record_width {
+        RecordWidthOfTheFirstContainer::AsRegistered => {}
+        RecordWidthOfTheFirstContainer::OneBelowTheRegistered { record_bytes } => {
+            common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
+                &mut image,
+                kind,
+                record_width_one_below(record_bytes),
+            );
+        }
+    }
     let walked_node_pointer =
         common_pointer_head_corruption::tree_table_entry_of_kind(&image, kind_walked_before).root;
     let walked_node_slot = common_pointer_head_corruption::the_slot_of(&walked_node_pointer);
@@ -13523,9 +13553,9 @@ fn an_inode_leaf_with_a_wrong_record_width_reddens_the_packed_container_invarian
 }
 
 /// F-KEY 谱系那一形：回退到 A 之后，谱系容器记录宽 16 → 15，谱系树根第一条条目子指针的一条位置条目指着 inode 树根（先走过，
-/// 回退那一版照抄 A 的 inode 树，A 那条根也引用它）。位置条目 0 那一形另多红 I-4.9：读回来的那片容器（这一版新写的）照判时，
-/// 它的诞生代号按第 0 条位置条目记在 inode 树根那个落点上（`birth_txg_of_walked_units` 照旧按第 0 条认），A 那条根再走到 inode 树根时按这个数判
-/// （推的，读码；攻方 F-KEY 臂量到同样的差，`research/prompts/m3-batch31-code-r1-opus-output.md` 第四节那张表），交主 agent 定，这里不钉它。
+/// 回退那一版照抄 A 的 inode 树，A 那条根也引用它）。读回来的那片容器（这一版新写的）的诞生代号记在它读回的那个落点上，不盖掉 inode 树根的
+/// （攻方 allrbmap，判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 1 条）：两种盘序红的同一批，I-4.9 不多红
+/// （改之前位置条目 0 那一形多红 I-4.9，攻方 `research/prompts/m3-batch32-code-r1-opus-output.md` 第 1.2 节 today 臂那 6 对之一）。
 #[test]
 fn a_lineage_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before(
 ) {
@@ -13540,7 +13570,7 @@ fn a_lineage_container_with_a_wrong_record_width_reddens_the_packed_container_in
         &[
             "I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4",
         ],
-        &[UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT],
+        &[],
     );
 }
 
@@ -14055,3 +14085,828 @@ fn a_tombstone_container_below_a_level_one_node_whose_second_copy_verifies_as_a_
 ) {
     assert_a_wrong_record_width_below_a_level_one_node_with_a_copy_verifying_as_a_container_reddens_the_packed_container_invariant(1);
 }
+
+// ─── 第 33 批（删除线）：一个单元的各把键按读回那一份的落点认（allrbmap）、F-ID-b 两份都没自证过、几处 F-KEY 与 G4 补的坏镜像 ───
+
+/// allrbmap 的判法（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第二节 K1 / K2 P1 没改的几把键那一行、第三节第 1 条；攻方
+/// `research/prompts/m3-batch32-code-r1-opus-output.md` 第 1.2、1.3 节 today 臂换盘判得不同的那 6 对）：种类 `kind` 那棵树根第一条条目子指针的一条
+/// 位置条目指着这一遍先走过、又被更早的候选根共享的码 2 节点（种类 `kind_walked_before` 那棵树的根）、整单元校验和对不上，另一条位置条目那一份是
+/// 按自己的位置条目自证得过的容器（记录宽照 `record_width`）。两种盘序（指着先走过的节点的是位置条目 0 / 1）各判一次：红的都是 `expected_violations`，
+/// I-4.9 与 I-3.5 都成立——读回那片容器的诞生代号记在它自己读回的落点上、不盖掉先走过那个节点的，I-3.5 的折叠表把那个落点记成它自己
+/// （改之前位置条目 0 那一形多红 I-4.9，只改诞生代号那张表时谱系那几形改红 I-3.5）。
+fn assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
+    clean: &MemoryPool,
+    kind: u16,
+    record_width: RecordWidthOfTheFirstContainer,
+    kind_walked_before: u16,
+    expected_violations: &[&'static str],
+) {
+    for location_index in 0..2 {
+        let image = image_whose_first_root_entry_names_a_walked_node_beside_a_container(
+            clean,
+            kind,
+            record_width,
+            kind_walked_before,
+            location_index,
+        );
+        let verdicts = check_pool_image(&image);
+        assert_eq!(
+            violated_invariant_set(&verdicts),
+            expected_violations
+                .iter()
+                .copied()
+                .collect::<std::collections::BTreeSet<&'static str>>(),
+            "指着先走过的节点的是位置条目 {location_index}：两种盘序红的同一批：{verdicts:?}"
+        );
+        for invariant_judged_by_the_placement in [
+            UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
+            REFERENCE_INTERVALS_ARE_EXACT,
+        ] {
+            let found = verdicts
+                .iter()
+                .find(|(identifier, _)| *identifier == invariant_judged_by_the_placement)
+                .map(|(_, found)| found.clone())
+                .expect("清单里有");
+            assert_eq!(
+                found,
+                InvariantVerdict::Holds,
+                "指着先走过的节点的是位置条目 {location_index}：{invariant_judged_by_the_placement} 按落点认、读落点上单元头里的诞生代号，成立"
+            );
+        }
+    }
+}
+
+/// 墓碑撞 extent 树根、容器记录宽 40 → 39（攻方 `base=tomb walked=extent variant=bad`）：三个单元的文件发完意图之后，extent 树根被更早那条候选根共享。
+#[test]
+fn a_tombstone_container_with_a_wrong_record_width_whose_location_names_the_shared_extent_root_judges_both_device_orders_alike(
+) {
+    assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
+        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+            "known-bad-tombstone-record-width-location-naming-the-shared-extent-root",
+        ),
+        TREE_KIND_TOMBSTONE,
+        RecordWidthOfTheFirstContainer::OneBelowTheRegistered {
+            record_bytes: singlefs_format::TOMBSTONE_RECORD_BYTES,
+        },
+        TREE_KIND_EXTENT,
+        &[
+            "I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4",
+        ],
+    );
+}
+
+/// 墓碑撞 extent 树根、容器不改（攻方 `base=tomb walked=extent variant=good`）。
+#[test]
+fn a_tombstone_container_whose_location_names_the_shared_extent_root_judges_both_device_orders_alike(
+) {
+    assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
+        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+            "known-bad-tombstone-location-naming-the-shared-extent-root",
+        ),
+        TREE_KIND_TOMBSTONE,
+        RecordWidthOfTheFirstContainer::AsRegistered,
+        TREE_KIND_EXTENT,
+        &["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"],
+    );
+}
+
+/// 谱系撞 inode 树根、容器不改（攻方 `base=lineage walked=inode variant=good`）：回退到 A 之后，回退那一版照抄 A 的 inode 树。
+/// 只改诞生代号那张表、不改 I-3.5 折叠表时这一形改红 I-3.5（攻方第 1.3 节）。记录宽 16 → 15 那一形见
+/// [`a_lineage_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before`]。
+#[test]
+fn a_lineage_container_whose_location_names_the_shared_inode_root_judges_both_device_orders_alike()
+{
+    assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
+        &image_after_rolling_back_to_the_first_root(
+            "known-bad-lineage-location-naming-the-shared-inode-root",
+        )
+        .image,
+        TREE_KIND_LINEAGE,
+        RecordWidthOfTheFirstContainer::AsRegistered,
+        TREE_KIND_INODE,
+        &["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"],
+    );
+}
+
+/// 谱系撞 extent 树根、容器记录宽 16 → 15（攻方 `base=lineage walked=extent variant=bad`）。
+#[test]
+fn a_lineage_container_with_a_wrong_record_width_whose_location_names_the_shared_extent_root_judges_both_device_orders_alike(
+) {
+    assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
+        &image_after_rolling_back_to_the_first_root(
+            "known-bad-lineage-record-width-location-naming-the-shared-extent-root",
+        )
+        .image,
+        TREE_KIND_LINEAGE,
+        RecordWidthOfTheFirstContainer::OneBelowTheRegistered {
+            record_bytes: singlefs_format::LINEAGE_RECORD_BYTES,
+        },
+        TREE_KIND_EXTENT,
+        &[
+            "I-1.7", "I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4",
+        ],
+    );
+}
+
+/// 谱系撞 extent 树根、容器不改（攻方 `base=lineage walked=extent variant=good`）。
+#[test]
+fn a_lineage_container_whose_location_names_the_shared_extent_root_judges_both_device_orders_alike()
+{
+    assert_both_device_orders_judge_alike_with_the_unit_birth_and_appearance_rules_holding(
+        &image_after_rolling_back_to_the_first_root(
+            "known-bad-lineage-location-naming-the-shared-extent-root",
+        )
+        .image,
+        TREE_KIND_LINEAGE,
+        RecordWidthOfTheFirstContainer::AsRegistered,
+        TREE_KIND_EXTENT,
+        &["I-2.1", "I-3.1", "I-3.11", "I-4.8", "I-5.1", "I-7.4"],
+    );
+}
+
+/// 回退那一版（D）的 inode 树根：照抄回退目标 A 的那一个（两版经同一个树根共享叶容器），或原样另写一份到没写过的槽、D 的树表改指它
+/// （两版各走自己的树根，叶容器由两个树根直接共享）。
+#[derive(Clone, Copy, Debug)]
+enum InodeRootOfTheRollbackPublish {
+    SharedWithTheRollbackTarget,
+    CopiedToASlotNothingWrote,
+}
+
+/// 两块盘上自己与前后各一槽都没写过的一个槽：单元区里写过的最后一槽再往后 16 槽。
+fn a_slot_after_every_written_slot_of_the_unit_area(image: &MemoryPool) -> u64 {
+    let last_written_slot = DEVICES
+        .iter()
+        .filter_map(|device| {
+            image
+                .devices
+                .get(&DeviceIdentity(*device))
+                .expect("盘")
+                .written_sectors()
+                .keys()
+                .map(|sector| sector * 512 / SLOT)
+                .filter(|slot| *slot >= UNIT_AREA_START_SLOT_OF_THE_POOL)
+                .max()
+        })
+        .max()
+        .expect("单元区写过");
+    last_written_slot + 16
+}
+
+/// 分叉形（攻方 `research/prompts/m3-batch32-code-r1-opus-output.md` 第 1.4 节，`raise_birth=true corrupt_loc0=true`；第 32 批实现员报告停下第 1 条
+/// 第二段推的那一形）：回退到 A（txg 3）之后的镜像，D（txg 9）照抄 A 的 inode 树。inode 树根第一条条目指的叶容器 L 头里的诞生代号抬到 D 的 txg
+/// （重封、沿引用链补到根：A、D 两版照旧引用同一个 L），再把 L 在位置条目 `garbled_location_index` 那块盘上的那一份写成别的字节
+/// （不补校验和，那个槽上没有别的单元）。`inode_root` 定两版经同一个树根共享 L，还是各走自己的树根、直接共享 L。
+fn image_after_the_rollback_whose_shared_leaf_container_is_born_after_the_target_with_one_copy_garbled(
+    rolled: &ImageAfterTheRollbackToTheFirstRoot,
+    garbled_location_index: usize,
+    inode_root: InodeRootOfTheRollbackPublish,
+) -> MemoryPool {
+    let mut image = rolled.image.clone();
+    let leaf = child_pointer_of_an_entry_of_the_root_of_kind(&image, TREE_KIND_INODE, 0);
+    let leaf_slot = common_pointer_head_corruption::the_slot_of(&leaf);
+    let data_unit_bytes = common_pointer_head_corruption::data_unit_bytes();
+    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+        &mut image,
+        leaf_slot,
+        data_unit_bytes,
+        |bytes| {
+            let birth_txg = u64::from_le_bytes(bytes[73..81].try_into().expect("8 字节"));
+            assert!(
+                birth_txg <= rolled.rollback_target_txg,
+                "L 的诞生代号 {birth_txg} 本来不晚于 A"
+            );
+            bytes[73..81].copy_from_slice(&rolled.rollback_publish_txg.to_le_bytes());
+        },
+    );
+    match inode_root {
+        InodeRootOfTheRollbackPublish::SharedWithTheRollbackTarget => {}
+        InodeRootOfTheRollbackPublish::CopiedToASlotNothingWrote => {
+            let inode_root_pointer =
+                common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_INODE)
+                    .root;
+            let inode_root_bytes = common_pointer_head_corruption::unit_bytes_at(
+                &image,
+                common_pointer_head_corruption::the_slot_of(&inode_root_pointer),
+                common_pointer_head_corruption::node_bytes(),
+            );
+            let copy_slot = a_slot_after_every_written_slot_of_the_unit_area(&image);
+            for device in DEVICES {
+                write(&mut image, device, copy_slot * SLOT, &inode_root_bytes);
+            }
+            let pointer_bytes =
+                common_pointer_head_corruption::node_pointer_bytes(&inode_root_pointer);
+            common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+                &mut image,
+                singlefs_core::address::SlotNumber(rolled.tree_table_slot_of_the_rollback_publish),
+                common_pointer_head_corruption::node_bytes(),
+                |table| {
+                    let starts: Vec<usize> = (0..=table.len() - pointer_bytes.len())
+                        .filter(|start| {
+                            table[*start..*start + pointer_bytes.len()] == pointer_bytes[..]
+                        })
+                        .collect();
+                    assert_eq!(starts.len(), 1, "D 的树表里 inode 树根指针恰好一处");
+                    for location_index in 0..2 {
+                        let slot_at = starts[0]
+                            + POINTER_FIRST_LOCATION_OFFSET
+                            + location_index * POINTER_LOCATION_BYTES
+                            + LOCATION_DEVICE_BYTES;
+                        table[slot_at..slot_at + LOCATION_SLOT_BYTES]
+                            .copy_from_slice(&copy_slot.to_le_bytes()[..LOCATION_SLOT_BYTES]);
+                    }
+                },
+            );
+        }
+    }
+    write(
+        &mut image,
+        leaf.locations[garbled_location_index].device.0,
+        leaf_slot.0 * SLOT,
+        &vec![0xa5u8; data_unit_bytes],
+    );
+    image
+}
+
+/// 分叉形的判法：L 坏的那一份在位置条目 0 / 1 两种盘序各判一次，红的都是 `expected_violations`，I-4.9（A 引用 L、L 读得出的那一份诞生代号 9 > 3）
+/// 与 I-3.5（D 引用、C 不引用的 inode 树根那一格：子树里 L 读得出、诞生代号 9 大于树根指针的出生 txg）都红。坏的是位置条目 0 那一份时，
+/// 诞生代号与 I-3.5 的折叠表都按读回的盘 1 那一份的落点记（攻方 allrbmap），GPU 判器同改（对拍流
+/// `the_gpu_judge_agrees_with_the_cpu_when_a_shared_leaf_container_born_after_the_rollback_target_reads_back_from_its_second_copy`）。
+fn assert_the_shared_leaf_container_born_after_the_rollback_target_reddens_alike_whichever_copy_is_garbled(
+    tag: &str,
+    inode_root: InodeRootOfTheRollbackPublish,
+    expected_violations: &[&'static str],
+) {
+    let rolled = image_after_rolling_back_to_the_first_root(tag);
+    for garbled_location_index in 0..2 {
+        let image = image_after_the_rollback_whose_shared_leaf_container_is_born_after_the_target_with_one_copy_garbled(
+            &rolled,
+            garbled_location_index,
+            inode_root,
+        );
+        let verdicts = check_pool_image(&image);
+        assert_eq!(
+            violated_invariant_set(&verdicts),
+            expected_violations
+                .iter()
+                .copied()
+                .collect::<std::collections::BTreeSet<&'static str>>(),
+            "L 坏的是位置条目 {garbled_location_index} 那一份：两种盘序红的同一批（I-4.9、I-3.5 都在里面）：{verdicts:?}"
+        );
+    }
+}
+
+/// 分叉形、两版经同一个 inode 树根共享叶容器（攻方 `direct=false`）：A 走到树根时「走过」命中，按树根下面记下的落点并进来。
+#[test]
+fn a_shared_leaf_container_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device(
+) {
+    assert_the_shared_leaf_container_born_after_the_rollback_target_reddens_alike_whichever_copy_is_garbled(
+        "known-bad-shared-leaf-container-born-after-the-target-garbled-copy",
+        InodeRootOfTheRollbackPublish::SharedWithTheRollbackTarget,
+        &[
+            "I-1.11",
+            "I-1.2",
+            "I-2.1",
+            "I-3.10",
+            REFERENCE_INTERVALS_ARE_EXACT,
+            "I-4.2",
+            "I-4.8",
+            UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
+            "I-7.4",
+        ],
+    );
+}
+
+/// 分叉形、两版各走自己的 inode 树根、直接共享叶容器（攻方 `direct=true`）：A 走到叶容器时「走过」命中，按叶容器读回那一份的落点并进来。
+/// 树根另写一份的那两槽没有分配记录（I-3.1、I-3.11 跟着红）。
+#[test]
+fn a_leaf_container_directly_shared_and_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device(
+) {
+    assert_the_shared_leaf_container_born_after_the_rollback_target_reddens_alike_whichever_copy_is_garbled(
+        "known-bad-directly-shared-leaf-container-born-after-the-target-garbled-copy",
+        InodeRootOfTheRollbackPublish::CopiedToASlotNothingWrote,
+        &[
+            "I-1.11",
+            "I-1.2",
+            "I-2.1",
+            "I-3.1",
+            "I-3.10",
+            "I-3.11",
+            REFERENCE_INTERVALS_ARE_EXACT,
+            "I-4.2",
+            "I-4.8",
+            UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
+            "I-7.4",
+        ],
+    );
+}
+
+/// 类型段 0 那条条目孩子的一份副本：这几种都不是按自己那一类的长度自证过的（正推 `research/prompts/m3-batch32-code-r1-sonnet-output.md`
+/// 第 2.2 节那张网格里两份都没自证过的那几种）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum CopyThatDoesNotVerifyAtTheLengthOfItsClass {
+    /// 头里是码 3、位置条目里的校验和只对得上前 16 KiB（`C16`）。
+    ContainerCheckedAtSixteenKibibytes,
+    /// 位置条目里的校验和翻几位（`BAD`）。
+    ChecksumBitsFlipped,
+    /// 位置条目里的盘不在池里（`UNREAD`）。
+    DeviceOutsideThePool,
+    /// 位置条目指着没写过的槽、校验和是 16 KiB 全零的 CRC（`ZERO`：读出全零，头里类标签 0 不登记）。
+    SlotNothingWroteWithTheChecksumOfZeros,
+}
+
+const COPIES_THAT_DO_NOT_VERIFY_AT_THE_LENGTH_OF_THEIR_CLASS:
+    [CopyThatDoesNotVerifyAtTheLengthOfItsClass; 4] = [
+    CopyThatDoesNotVerifyAtTheLengthOfItsClass::ContainerCheckedAtSixteenKibibytes,
+    CopyThatDoesNotVerifyAtTheLengthOfItsClass::ChecksumBitsFlipped,
+    CopyThatDoesNotVerifyAtTheLengthOfItsClass::DeviceOutsideThePool,
+    CopyThatDoesNotVerifyAtTheLengthOfItsClass::SlotNothingWroteWithTheChecksumOfZeros,
+];
+
+/// 回退到 A 之后的镜像与三个单元的文件发完意图之后的镜像里两块盘上都没写过的一槽（造之前核过全零）。
+const SLOT_NOTHING_WROTE_IN_THE_DELETION_AND_ROLLBACK_IMAGES: u64 = 190_000;
+
+/// 种类 `kind` 那棵树根（层级 1）第一条条目类型段 `registered_type` → 0、身份三段清零，孩子（码 3 容器）的两条位置条目照 `copies` 各改成一种
+/// 自证不过的副本。
+fn image_whose_type_zero_root_entry_names_a_child_with_two_copies_that_do_not_verify(
+    clean: &MemoryPool,
+    kind: u16,
+    registered_type: u16,
+    copies: [CopyThatDoesNotVerifyAtTheLengthOfItsClass; 2],
+) -> MemoryPool {
+    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, kind, 0);
+    let container = common_pointer_head_corruption::unit_bytes_at(
+        clean,
+        common_pointer_head_corruption::the_slot_of(&child),
+        common_pointer_head_corruption::data_unit_bytes(),
+    );
+    let checksum_of_the_container = crc32_castagnoli(&container);
+    let checksum_of_the_first_sixteen_kibibytes =
+        crc32_castagnoli(&container[..common_pointer_head_corruption::node_bytes()]);
+    let checksum_of_sixteen_zero_kibibytes =
+        crc32_castagnoli(&vec![0u8; common_pointer_head_corruption::node_bytes()]);
+    let mut image = clean.clone();
+    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            registered_type,
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        );
+        zero_the_three_identity_fields(entry);
+        for (location_index, copy) in copies.iter().enumerate() {
+            match copy {
+                CopyThatDoesNotVerifyAtTheLengthOfItsClass::ContainerCheckedAtSixteenKibibytes => {
+                    set_the_whole_unit_checksum_in_a_child_location(
+                        entry,
+                        location_index,
+                        checksum_of_the_first_sixteen_kibibytes,
+                    );
+                }
+                CopyThatDoesNotVerifyAtTheLengthOfItsClass::ChecksumBitsFlipped => {
+                    set_the_whole_unit_checksum_in_a_child_location(
+                        entry,
+                        location_index,
+                        checksum_of_the_container ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY,
+                    );
+                }
+                CopyThatDoesNotVerifyAtTheLengthOfItsClass::DeviceOutsideThePool => {
+                    set_the_device_in_a_child_location(
+                        entry,
+                        location_index,
+                        DEVICE_IDENTITY_NOT_IN_THE_POOL,
+                    );
+                }
+                CopyThatDoesNotVerifyAtTheLengthOfItsClass::SlotNothingWroteWithTheChecksumOfZeros => {
+                    set_the_slot_in_a_child_location(
+                        entry,
+                        location_index,
+                        SLOT_NOTHING_WROTE_IN_THE_DELETION_AND_ROLLBACK_IMAGES,
+                    );
+                    set_the_whole_unit_checksum_in_a_child_location(
+                        entry,
+                        location_index,
+                        checksum_of_sixteen_zero_kibibytes,
+                    );
+                }
+            }
+        }
+    });
+    assert_eq!(level, 1, "根层级 1、直接管容器");
+    image
+}
+
+/// F-ID-b 两份都没自证过（用户 2026-10-07 定「当读不出、这一小句不判」，`records/2026-10-01-删除与并发事务.md` 同日那一条；判决
+/// `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 3 条）：两条位置条目各取一种自证不过的副本、两两组合，身份那一条
+/// （`identity_invariant`）一格都不红；互换两条位置条目，红的同一批（盘不在池里那一种放在位置条目 0 时位置条目的盘序不升序，I-2.5 那一条不比）。
+/// 改之前取第一份按 16 KiB 对得上的头里的类标签：组合里有 `C16`（码 3）或 `ZERO`（码 0）的那 12 格判身份红（正推第 2.2 节第 3 点）。
+fn assert_a_type_zero_entry_over_two_copies_that_do_not_verify_leaves_the_identity_unjudged_on_either_device(
+    clean: &MemoryPool,
+    kind: u16,
+    registered_type: u16,
+    identity_invariant: &'static str,
+) {
+    for device in DEVICES {
+        assert!(
+            read(
+                clean,
+                device,
+                SLOT_NOTHING_WROTE_IN_THE_DELETION_AND_ROLLBACK_IMAGES * SLOT,
+                common_pointer_head_corruption::data_unit_bytes(),
+            )
+            .iter()
+            .all(|byte| *byte == 0),
+            "槽 {SLOT_NOTHING_WROTE_IN_THE_DELETION_AND_ROLLBACK_IMAGES} 起 32 KiB 在盘 {device} 上没写过"
+        );
+    }
+    let mut violated_of_each_pair = std::collections::BTreeMap::new();
+    for first in COPIES_THAT_DO_NOT_VERIFY_AT_THE_LENGTH_OF_THEIR_CLASS {
+        for second in COPIES_THAT_DO_NOT_VERIFY_AT_THE_LENGTH_OF_THEIR_CLASS {
+            let image =
+                image_whose_type_zero_root_entry_names_a_child_with_two_copies_that_do_not_verify(
+                    clean,
+                    kind,
+                    registered_type,
+                    [first, second],
+                );
+            let verdicts = check_pool_image(&image);
+            let violated = violated_invariant_set(&verdicts);
+            assert!(
+                !violated.contains(identity_invariant),
+                "位置条目 0 是 {first:?}、1 是 {second:?}：两份都没自证过，「子单元是码 2」这一小句不判，{identity_invariant} 不红：{verdicts:?}"
+            );
+            violated_of_each_pair.insert(
+                (format!("{first:?}"), format!("{second:?}")),
+                violated
+                    .into_iter()
+                    .filter(|invariant| *invariant != "I-2.5")
+                    .collect::<std::collections::BTreeSet<&'static str>>(),
+            );
+        }
+    }
+    for ((first, second), violated) in &violated_of_each_pair {
+        assert_eq!(
+            Some(violated),
+            violated_of_each_pair.get(&(second.clone(), first.clone())),
+            "位置条目 0 是 {first}、1 是 {second}，与互换之后红的同一批（I-2.5 之外）"
+        );
+    }
+}
+
+#[test]
+fn a_tombstone_root_entry_of_type_zero_over_two_copies_that_do_not_verify_leaves_the_tombstone_entry_invariant_holding_on_either_device(
+) {
+    assert_a_type_zero_entry_over_two_copies_that_do_not_verify_leaves_the_identity_unjudged_on_either_device(
+        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+            "known-bad-tombstone-entry-type-zero-two-copies-that-do-not-verify",
+        ),
+        TREE_KIND_TOMBSTONE,
+        PACKED_RECORD_TYPE_TOMBSTONE,
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+    );
+}
+
+#[test]
+fn a_lineage_root_entry_of_type_zero_over_two_copies_that_do_not_verify_leaves_the_lineage_entry_invariant_holding_on_either_device(
+) {
+    assert_a_type_zero_entry_over_two_copies_that_do_not_verify_leaves_the_identity_unjudged_on_either_device(
+        &image_after_rolling_back_to_the_first_root(
+            "known-bad-lineage-entry-type-zero-two-copies-that-do-not-verify",
+        )
+        .image,
+        TREE_KIND_LINEAGE,
+        PACKED_RECORD_TYPE_LINEAGE,
+        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+    );
+}
+
+/// 一条位置条目的盘上原样：盘 4 + 槽 6 + 整单元校验和 4，小端。
+fn bytes_of_a_location_entry(device: u32, slot: u64, checksum: u32) -> Vec<u8> {
+    let mut bytes = device.to_le_bytes().to_vec();
+    bytes.extend_from_slice(&slot.to_le_bytes()[..LOCATION_SLOT_BYTES]);
+    bytes.extend_from_slice(&checksum.to_le_bytes());
+    bytes
+}
+
+/// G4 那一读另造的一形（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第二节 K1 P3 那一行、第三节第 4 条；正推变异 E1；
+/// 第 32 批实现员报告停下第 4 条）：三个单元的文件发完意图之后，H-V 那一步（墓碑树根第一条条目指的容器在位置条目 `location_index` 那块盘上的
+/// 一份换成墓碑树根的码 2 字节、那条位置条目的校验和换成它的 CRC），再把中央映射里那片容器那一条在同一块盘上的位置条目校验和换成那块盘上
+/// 此刻 32 KiB 的 CRC（重封、补到根）：映射那一读对得上，I-2.1 只剩 F-W 往下走时按类型段那一类（码 3）的 32 KiB 读那一份那一读来红。
+fn image_whose_container_copy_verifying_as_a_node_matches_its_central_mapping_entry(
+    clean: &MemoryPool,
+    location_index: usize,
+) -> MemoryPool {
+    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, TREE_KIND_TOMBSTONE, 0);
+    let container_slot = common_pointer_head_corruption::the_slot_of(&child);
+    let checksum_of_the_container =
+        crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
+            clean,
+            container_slot,
+            common_pointer_head_corruption::data_unit_bytes(),
+        ));
+    let mut image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
+        clean,
+        TREE_KIND_TOMBSTONE,
+        location_index,
+        &leave_the_entry_as_it_is,
+    );
+    let device = child.locations[location_index].device.0;
+    let checksum_of_the_copy_now_on_the_device = crc32_castagnoli(&read(
+        &image,
+        device,
+        container_slot.0 * SLOT,
+        common_pointer_head_corruption::data_unit_bytes(),
+    ));
+    let mapping_root = common_pointer_head_corruption::newest_root(&image).mapping_root;
+    let before = bytes_of_a_location_entry(device, container_slot.0, checksum_of_the_container);
+    let after = bytes_of_a_location_entry(
+        device,
+        container_slot.0,
+        checksum_of_the_copy_now_on_the_device,
+    );
+    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+        &mut image,
+        common_pointer_head_corruption::the_slot_of(&mapping_root),
+        common_pointer_head_corruption::node_bytes(),
+        |bytes| {
+            let starts: Vec<usize> = (0..=bytes.len() - before.len())
+                .filter(|start| bytes[*start..*start + before.len()] == before[..])
+                .collect();
+            assert_eq!(
+                starts.len(),
+                1,
+                "中央映射树（根兼叶）里那片容器那一条在盘 {device} 上的位置条目恰好一处"
+            );
+            bytes[starts[0]..starts[0] + after.len()].copy_from_slice(&after);
+        },
+    );
+    image
+}
+
+/// G4 那一读：两种盘序（码 2 那一份在位置条目 0 / 1）各判一次，I-2.1 红、第一处违例说的是墓碑树内部条目的子指针那一读（中央映射那一读对得上）。
+/// 去掉 F-W 往下走时那一读的 I-2.1（正推变异 E1）这一条红。
+#[test]
+fn a_tombstone_container_copy_verifying_as_a_node_whose_mapping_entry_matches_it_reddens_the_checksum_invariant_on_either_device(
+) {
+    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+        "known-bad-tombstone-container-copy-verifying-as-a-node-matching-its-mapping-entry",
+    );
+    for location_index in 0..2 {
+        let image =
+            image_whose_container_copy_verifying_as_a_node_matches_its_central_mapping_entry(
+                &clean,
+                location_index,
+            );
+        let verdicts = check_pool_image(&image);
+        assert!(
+            matches!(
+                verdict(&image, "I-2.1"),
+                InvariantVerdict::Violated(detail) if detail.contains("墓碑树内部条目的子指针")
+            ),
+            "码 2 那一份在位置条目 {location_index}：F-W 往下走时按码 3 的 32 KiB 读回那一份对不上，I-2.1 红在这一读：{verdicts:?}"
+        );
+    }
+}
+
+/// F-KEY 数据单元那一处（正推变异 E2）：新池新建文件那一版，第一个文件的数据单元头里五元组的对象号加 1（重封、补到根，I-1.1 违例），
+/// 再把 extent 树根（上段叶）里内联它的那条数据指针第 `location_index` 条位置条目的槽改成 extent 树根自己的槽（这一遍刚走过）、
+/// 校验和写成它那 16 KiB 的 CRC 翻几位（这一份对不上）。另一条位置条目那一份就是带违例的数据单元。
+fn image_whose_inline_data_pointer_names_the_walked_extent_root_beside_a_data_unit_with_another_object(
+    clean: &MemoryPool,
+    location_index: usize,
+) -> MemoryPool {
+    const DATA_UNIT_OBJECT_OFFSET: usize = 51;
+    let mut image = clean.clone();
+    let singlefs_core::extent_tree::ExtentUpperLeafTarget::InlineDataUnit(data_pointer) =
+        common_pointer_head_corruption::first_upper_leaf_entry(&image).target
+    else {
+        panic!("新池新建文件那一版只有一个数据单元，上段叶第一条内联它的数据指针");
+    };
+    assert_eq!(
+        data_pointer.locations[0].slot.0, DATA_UNIT,
+        "新池新建文件那一版唯一的数据单元在登记过的那一槽"
+    );
+    // 码 1 的重封：`mutate_unit` 认三种码（`common_deletion_in_progress` 那一份只认码 2 与码 3）。
+    mutate_unit(
+        &mut image,
+        DATA_UNIT,
+        |bytes| {
+            let object = get_u64(bytes, DATA_UNIT_OBJECT_OFFSET);
+            set_u64(bytes, DATA_UNIT_OBJECT_OFFSET, object + 1);
+        },
+        true,
+    );
+    let singlefs_core::extent_tree::ExtentUpperLeafTarget::InlineDataUnit(resealed_pointer) =
+        common_pointer_head_corruption::first_upper_leaf_entry(&image).target
+    else {
+        panic!("改了数据单元之后上段叶第一条照旧内联它的数据指针");
+    };
+    let extent_root =
+        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root;
+    let extent_root_slot = common_pointer_head_corruption::the_slot_of(&extent_root);
+    let before = common_pointer_head_corruption::data_pointer_bytes(&resealed_pointer);
+    let mut pointer_naming_the_extent_root = resealed_pointer;
+    pointer_naming_the_extent_root.locations[location_index].slot = extent_root_slot;
+    pointer_naming_the_extent_root.locations[location_index].unit_checksum =
+        crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
+            &image,
+            extent_root_slot,
+            common_pointer_head_corruption::node_bytes(),
+        )) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
+    let after = common_pointer_head_corruption::data_pointer_bytes(&pointer_naming_the_extent_root);
+    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+        &mut image,
+        extent_root_slot,
+        common_pointer_head_corruption::node_bytes(),
+        |bytes| {
+            let starts: Vec<usize> = (0..=bytes.len() - before.len())
+                .filter(|start| bytes[*start..*start + before.len()] == before[..])
+                .collect();
+            assert_eq!(starts.len(), 1, "上段叶里那条数据指针恰好一处");
+            bytes[starts[0]..starts[0] + after.len()].copy_from_slice(&after);
+        },
+    );
+    image
+}
+
+/// F-KEY 数据单元那一处：两种盘序（指着先走过的 extent 树根的是位置条目 0 / 1）各判一次，读回的那份数据单元照判，I-1.1 红在它的五元组、
+/// 红的同一批。「走过」按第 0 条位置条目认的话，位置条目 0 那一形把读回来的数据单元当成走过（它的五元组一个字节都不判）。
+#[test]
+fn a_data_unit_with_another_object_whose_location_names_the_walked_extent_root_reddens_its_five_tuple_on_either_device(
+) {
+    let clean = build_pool("known-bad-data-unit-object-location-naming-the-walked-extent-root")
+        .memory_pool();
+    let mut violated_on_each_location = Vec::new();
+    for location_index in 0..2 {
+        let image =
+            image_whose_inline_data_pointer_names_the_walked_extent_root_beside_a_data_unit_with_another_object(
+                &clean,
+                location_index,
+            );
+        let verdicts = check_pool_image(&image);
+        assert!(
+            matches!(
+                verdict(&image, "I-1.1"),
+                InvariantVerdict::Violated(detail) if detail.contains("数据单元五元组")
+            ),
+            "指着先走过的节点的是位置条目 {location_index}：读回来的那份数据单元照判，I-1.1 红在它的五元组：{verdicts:?}"
+        );
+        violated_on_each_location.push(violated_invariant_set(&verdicts));
+    }
+    assert_eq!(
+        violated_on_each_location[0], violated_on_each_location[1],
+        "同一张镜像只差指着先走过的节点的是哪一条位置条目：红的同一批"
+    );
+}
+
+/// F-KEY inode 内部孩子那一处（正推变异 E4）：新池新建文件那一版，inode 树根第一条条目类型段 2 → 0、身份三段清零；位置条目 `location_index`
+/// 的槽改成 extent 树根的槽（这一遍先走过）、校验和写成它那 16 KiB 的 CRC 翻几位；另一条位置条目那块盘上叶容器的槽换成 inode 树根的码 2 字节、
+/// 那条位置条目的校验和换成它的 CRC（按 16 KiB 自证得过）。读回的就是那份码 2，当 inode 树的内部节点读（层级 1，不是父层级减一）。
+fn image_whose_type_zero_inode_entry_names_the_walked_extent_root_beside_a_copy_verifying_as_a_node(
+    clean: &MemoryPool,
+    location_index: usize,
+) -> MemoryPool {
+    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, TREE_KIND_INODE, 0);
+    let inode_root_bytes = root_node_bytes_of_the_tree_of_kind(clean, TREE_KIND_INODE);
+    let extent_root =
+        common_pointer_head_corruption::tree_table_entry_of_kind(clean, TREE_KIND_EXTENT).root;
+    let extent_root_slot = common_pointer_head_corruption::the_slot_of(&extent_root);
+    let wrong_checksum_of_the_extent_root =
+        crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
+            clean,
+            extent_root_slot,
+            common_pointer_head_corruption::node_bytes(),
+        )) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
+    let other_location_index = 1 - location_index;
+    let mut image = clean.clone();
+    let level = change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_INODE,
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        );
+        zero_the_three_identity_fields(entry);
+        set_the_slot_in_a_child_location(entry, location_index, extent_root_slot.0);
+        set_the_whole_unit_checksum_in_a_child_location(
+            entry,
+            location_index,
+            wrong_checksum_of_the_extent_root,
+        );
+        set_the_whole_unit_checksum_in_a_child_location(
+            entry,
+            other_location_index,
+            crc32_castagnoli(&inode_root_bytes),
+        );
+    });
+    assert_eq!(level, 1, "新池新建文件那一版的 inode 树根层级 1");
+    write(
+        &mut image,
+        child.locations[other_location_index].device.0,
+        common_pointer_head_corruption::the_slot_of(&child).0 * SLOT,
+        &inode_root_bytes,
+    );
+    image
+}
+
+/// F-KEY inode 内部孩子那一处：两种盘序各判一次，读回的那份码 2 照判，I-1.1 红在「inode 树的内部节点」那一句，红的同一批。
+#[test]
+fn a_type_zero_inode_entry_whose_location_names_the_walked_extent_root_walks_the_copy_verifying_as_a_node_on_either_device(
+) {
+    let clean =
+        build_pool("known-bad-inode-entry-type-zero-location-naming-the-walked-extent-root")
+            .memory_pool();
+    let mut violated_on_each_location = Vec::new();
+    for location_index in 0..2 {
+        let image =
+            image_whose_type_zero_inode_entry_names_the_walked_extent_root_beside_a_copy_verifying_as_a_node(
+                &clean,
+                location_index,
+            );
+        let verdicts = check_pool_image(&image);
+        assert!(
+            matches!(
+                verdict(&image, "I-1.1"),
+                InvariantVerdict::Violated(detail) if detail.contains("inode 树的内部节点")
+            ),
+            "指着先走过的节点的是位置条目 {location_index}：读回来的那份码 2 照判，I-1.1 红在「inode 树的内部节点」：{verdicts:?}"
+        );
+        violated_on_each_location.push(violated_invariant_set(&verdicts));
+    }
+    assert_eq!(
+        violated_on_each_location[0], violated_on_each_location[1],
+        "同一张镜像只差指着先走过的节点的是哪一条位置条目：红的同一批"
+    );
+}
+
+/// `read_index_node` 那一处（判决 `research/prompts/m3-batch32-code-r1-main-verification.md` 第三节第 1 条，第 32 批实现员报告停下第 2 条）：
+/// 新池新建文件那一版，inode 叶容器第一条记录的预留段写一个非零字节（两份、重封、补到根，I-9.7 违例）；再把树表里 inode 树那条条目的根指针
+/// 第 `location_index` 条位置条目的槽改成树表单元自己的槽（这一遍先走过）、校验和写成它那 16 KiB 的 CRC 翻几位（这一份对不上）。
+/// 另一条位置条目那一份就是 inode 树根，读者用的就是它。
+fn image_whose_inode_root_pointer_names_the_walked_tree_table_above_a_leaf_with_a_non_zero_reserved_byte(
+    clean: &MemoryPool,
+    location_index: usize,
+) -> MemoryPool {
+    let mut image = clean.clone();
+    mutate_unit(
+        &mut image,
+        INODE_LEAF,
+        |bytes| bytes[INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_LAST_RESERVED_BYTE_OFFSET] = 1,
+        true,
+    );
+    let inode_root =
+        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
+    let tree_table = common_pointer_head_corruption::newest_root(&image).tree_table;
+    let tree_table_slot = common_pointer_head_corruption::the_slot_of(&tree_table);
+    let before = common_pointer_head_corruption::node_pointer_bytes(&inode_root);
+    let mut pointer_naming_the_tree_table = inode_root;
+    pointer_naming_the_tree_table.locations[location_index].slot = tree_table_slot;
+    pointer_naming_the_tree_table.locations[location_index].unit_checksum =
+        crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
+            &image,
+            tree_table_slot,
+            common_pointer_head_corruption::node_bytes(),
+        )) ^ CHECKSUM_BITS_FLIPPED_SO_THE_COPY_DOES_NOT_VERIFY;
+    let after = common_pointer_head_corruption::node_pointer_bytes(&pointer_naming_the_tree_table);
+    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
+        &mut image,
+        tree_table_slot,
+        common_pointer_head_corruption::node_bytes(),
+        |bytes| {
+            let starts: Vec<usize> = (0..=bytes.len() - before.len())
+                .filter(|start| bytes[*start..*start + before.len()] == before[..])
+                .collect();
+            assert_eq!(starts.len(), 1, "树表里 inode 树根指针恰好一处");
+            bytes[starts[0]..starts[0] + after.len()].copy_from_slice(&after);
+        },
+    );
+    image
+}
+
+/// `read_index_node` 那一处：两种盘序（指着先走过的树表单元的是位置条目 0 / 1）各判一次，读回的那份 inode 树根照走，它下面那片叶容器的
+/// I-9.7 违例照红，红的同一批。「走过」按第 0 条位置条目认的话，位置条目 0 那一形把 inode 树根当成别的版本先走过（并进树表下面的落点、不往下走），
+/// 叶容器一个字节都不判。
+#[test]
+fn an_inode_leaf_with_a_non_zero_reserved_byte_below_a_root_pointer_naming_the_walked_tree_table_reddens_the_reserved_bytes_invariant_on_either_device(
+) {
+    let clean = build_pool(
+        "known-bad-inode-leaf-reserved-byte-below-a-root-pointer-naming-the-walked-tree-table",
+    )
+    .memory_pool();
+    let mut violated_on_each_location = Vec::new();
+    for location_index in 0..2 {
+        let image =
+            image_whose_inode_root_pointer_names_the_walked_tree_table_above_a_leaf_with_a_non_zero_reserved_byte(
+                &clean,
+                location_index,
+            );
+        let verdicts = check_pool_image(&image);
+        let violated = violated_invariant_set(&verdicts);
+        assert!(
+            violated.contains("I-9.7"),
+            "指着先走过的树表单元的是位置条目 {location_index}：读回的那份 inode 树根照走，叶容器里的 I-9.7 违例照红：{verdicts:?}"
+        );
+        violated_on_each_location.push(violated);
+    }
+    assert_eq!(
+        violated_on_each_location[0], violated_on_each_location[1],
+        "同一张镜像只差指着先走过的节点的是哪一条位置条目：红的同一批"
+    );
+}
````

## 追加的变异行（/home/fy5090/.local/share/singlefs/work/m3-batch33/s/patch/mutations-append.tsv，合后在 `crates/mutations.tsv` 末尾）

````tsv
第33批 allrbmap：诞生代号按第 0 条位置条目的落点记（盖掉先走过的单元的），墓碑撞被更早候选根共享的 extent 树根那一形位置条目 0 多红 I-4.9	crates/singlefs-checker/src/walk.rs	        self.birth_txg_of_walked_units\n            .insert(placement_it_was_read_from, birth_txg);\n	        self.birth_txg_of_walked_units.insert(\n            (pointer.locations[0].device, pointer.locations[0].slot),\n            birth_txg,\n        );\n	-p singlefs-harness --test checker_known_bad_images -- judges_both_device_orders_alike whichever_location_names_a_node_walked_before born_after_the_rollback_target_with_a_garbled_copy over_two_copies_that_do_not_verify whose_mapping_entry_matches_it whose_location_names_the_walked_extent_root naming_the_walked_tree_table	a_tombstone_container_whose_location_names_the_shared_extent_root_judges_both_device_orders_alike
第33批 allrbmap：读回第 1 条那一份时 I-3.5 的折叠表不把那个落点记成它自己（照旧折在第 0 条落点上），分叉形盘 0 那份坏时漏 I-3.5	crates/singlefs-checker/src/walk.rs	        if placement_it_was_read_from != first_placement\n            && self.unit_of_each_placement.get(&placement_it_was_read_from)\n                == Some(&first_placement)\n        {\n	        if placement_it_was_read_from != first_placement\n            && self.unit_of_each_placement.get(&placement_it_was_read_from)\n                == Some(&first_placement)\n            && placement_it_was_read_from == first_placement\n        {\n	-p singlefs-harness --test checker_known_bad_images -- judges_both_device_orders_alike whichever_location_names_a_node_walked_before born_after_the_rollback_target_with_a_garbled_copy over_two_copies_that_do_not_verify whose_mapping_entry_matches_it whose_location_names_the_walked_extent_root naming_the_walked_tree_table	a_shared_leaf_container_born_after_the_rollback_target_with_a_garbled_copy_reddens_the_birth_and_appearance_rules_on_either_device
第33批 F-ID-b 两份都没自证过：照改之前取第一份按 16 KiB 对得上的头里的类标签（C16、ZERO 那几格 I-9.23 / I-9.17 红）	crates/singlefs-checker/src/walk.rs	                    let child_class = class_of_the_first_copy_verifying_at_the_length_of_its_class(\n                        self.reader,\n                        &child,\n                    );\n	                    let child_class = class_of_the_first_copy_verifying_at_the_length_of_its_class(\n                        self.reader,\n                        &child,\n                    )\n                    .or_else(|| {\n                        read_unit_without_judging(self.reader, &child, node_bytes())\n                            .map(|unit| MappingKeyUnitClass::of(unit[UNIT_HEADER_CLASS_TAG_OFFSET]))\n                    });\n	-p singlefs-harness --test checker_known_bad_images -- judges_both_device_orders_alike whichever_location_names_a_node_walked_before born_after_the_rollback_target_with_a_garbled_copy over_two_copies_that_do_not_verify whose_mapping_entry_matches_it whose_location_names_the_walked_extent_root naming_the_walked_tree_table	a_tombstone_root_entry_of_type_zero_over_two_copies_that_do_not_verify_leaves_the_tombstone_entry_invariant_holding_on_either_device
第33批 G4 那一读（正推 E1）：走读往下读的单元不判 I-2.1（只取第一份校验和对得上的），中央映射那一读对得上时 I-2.1 不红	crates/singlefs-checker/src/walk.rs	        let (unit, location_it_was_read_from) =\n            crate::image::read_referenced_unit_with_the_location_it_was_read_from(\n                self.reader,\n                &mut self.judgements,\n                &pointer.locations,\n                unit_bytes,\n                what,\n            )?;\n	        let _ = what;\n        let (unit, location_it_was_read_from) = pointer.locations.iter().find_map(|location| {\n            self.reader\n                .read(location.device, location.slot * SLOT_BYTES, unit_bytes)\n                .filter(|content| crc32_castagnoli_table(content) == location.checksum)\n                .map(|content| (content, *location))\n        })?;\n	-p singlefs-harness --test checker_known_bad_images -- judges_both_device_orders_alike whichever_location_names_a_node_walked_before born_after_the_rollback_target_with_a_garbled_copy over_two_copies_that_do_not_verify whose_mapping_entry_matches_it whose_location_names_the_walked_extent_root naming_the_walked_tree_table	a_tombstone_container_copy_verifying_as_a_node_whose_mapping_entry_matches_it_reddens_the_checksum_invariant_on_either_device
第33批 F-KEY 几处一起退：走读认的落点交回第 0 条位置条目的（走过、诞生代号、父单元都按它），数据单元一形位置条目 0 漏判五元组	crates/singlefs-checker/src/walk.rs	        Some((unit, placement_it_was_read_from))\n    }\n	        Some((unit, first_placement))\n    }\n	-p singlefs-harness --test checker_known_bad_images -- judges_both_device_orders_alike whichever_location_names_a_node_walked_before born_after_the_rollback_target_with_a_garbled_copy over_two_copies_that_do_not_verify whose_mapping_entry_matches_it whose_location_names_the_walked_extent_root naming_the_walked_tree_table	a_data_unit_with_another_object_whose_location_names_the_walked_extent_root_reddens_its_five_tuple_on_either_device
第33批 F-KEY 只退数据单元那一处（正推 E2）：「走过」按第 0 条位置条目的落点认	crates/singlefs-checker/src/walk.rs	        if !self.visited_units.insert(placement_it_was_read_from) {\n            self.merge_placements_below_a_unit_walked_by_an_earlier_version(\n                placement_it_was_read_from,\n            );\n            return;\n        }\n        if !self.judge_unit_header(&unit, 1, "数据单元") {\n	        if !self\n            .visited_units\n            .insert((pointer.locations[0].device, pointer.locations[0].slot))\n        {\n            self.merge_placements_below_a_unit_walked_by_an_earlier_version(\n                placement_it_was_read_from,\n            );\n            return;\n        }\n        if !self.judge_unit_header(&unit, 1, "数据单元") {\n	-p singlefs-harness --test checker_known_bad_images -- judges_both_device_orders_alike whichever_location_names_a_node_walked_before born_after_the_rollback_target_with_a_garbled_copy over_two_copies_that_do_not_verify whose_mapping_entry_matches_it whose_location_names_the_walked_extent_root naming_the_walked_tree_table	a_data_unit_with_another_object_whose_location_names_the_walked_extent_root_reddens_its_five_tuple_on_either_device
第33批 F-KEY 只退 inode 内部孩子那一处（正推 E4）：「走过」按第 0 条位置条目的落点认	crates/singlefs-checker/src/walk.rs	            walked_first_by_this_version: true,\n        };\n        if !self.visited_units.insert(placement_it_was_read_from) {\n	            walked_first_by_this_version: true,\n        };\n        if !self\n            .visited_units\n            .insert((child.locations[0].device, child.locations[0].slot))\n        {\n	-p singlefs-harness --test checker_known_bad_images -- judges_both_device_orders_alike whichever_location_names_a_node_walked_before born_after_the_rollback_target_with_a_garbled_copy over_two_copies_that_do_not_verify whose_mapping_entry_matches_it whose_location_names_the_walked_extent_root naming_the_walked_tree_table	a_type_zero_inode_entry_whose_location_names_the_walked_extent_root_walks_the_copy_verifying_as_a_node_on_either_device
第33批 read_index_node 的「走过」按第 0 条位置条目的落点认：树表里 inode 树根指针第 0 条指着先走过的树表单元时 inode 树不往下走	crates/singlefs-checker/src/walk.rs	        if !self.visited_units.insert(placement_it_was_read_from) {\n            self.merge_placements_below_a_unit_walked_by_an_earlier_version(\n                placement_it_was_read_from,\n            );\n            return IndexNodeReading::WalkedByAnEarlierVersion {\n	        if !self\n            .visited_units\n            .insert((pointer.locations[0].device, pointer.locations[0].slot))\n        {\n            self.merge_placements_below_a_unit_walked_by_an_earlier_version(\n                placement_it_was_read_from,\n            );\n            return IndexNodeReading::WalkedByAnEarlierVersion {\n	-p singlefs-harness --test checker_known_bad_images -- judges_both_device_orders_alike whichever_location_names_a_node_walked_before born_after_the_rollback_target_with_a_garbled_copy over_two_copies_that_do_not_verify whose_mapping_entry_matches_it whose_location_names_the_walked_extent_root naming_the_walked_tree_table	an_inode_leaf_with_a_non_zero_reserved_byte_below_a_root_pointer_naming_the_walked_tree_table_reddens_the_reserved_bytes_invariant_on_either_device
第33批 「自证过」的那一份判据只按 16 KiB 比校验和（不按头里那一类的长度）：C16 那一份算成自证过的码 3，I-9.17 红	crates/singlefs-checker/src/walk.rs	    let unit_bytes = unit_bytes_of_the_class(class_in_the_header)?;\n    reader\n        .read(location.device, offset, unit_bytes)\n	    let _unit_bytes = unit_bytes_of_the_class(class_in_the_header)?;\n    reader\n        .read(location.device, offset, node_bytes())\n	-p singlefs-harness --test checker_known_bad_images -- judges_both_device_orders_alike whichever_location_names_a_node_walked_before born_after_the_rollback_target_with_a_garbled_copy over_two_copies_that_do_not_verify whose_mapping_entry_matches_it whose_location_names_the_walked_extent_root naming_the_walked_tree_table	a_lineage_root_entry_of_type_zero_over_two_copies_that_do_not_verify_leaves_the_lineage_entry_invariant_holding_on_either_device
第33批 崩溃放量 CPU 判器不带「没有文件的那几版」（版本表只认带文件的），删除流各批报没有文件判违例、放量红	crates/singlefs-checker-tier/src/crash_amplification.rs	        flow.judged_root_index,\n        flow.version_table(),\n        &expansion,\n	        flow.judged_root_index,\n        Layer0VersionTable::with_a_file_only(flow.versions),\n        &expansion,\n	-p singlefs-checker-tier --features verdict-store --test crash_enumeration_unlink_after_the_multi_record_publish_stream -- crash_amplification_of_the_unlink_after_the_multi_record_publish_stream_is_clean	crash_amplification_of_the_unlink_after_the_multi_record_publish_stream_is_clean
第33批 GPU 期望版本表第 5 字一律记成有文件，恢复内核的 oracle 把删完那条流的几版报没有文件判成违例、与 CPU 不一致	crates/singlefs-checker-tier/src/crash_judge_tables.rs	                    FileOfTheExpectedVersion::WithoutAFile => EXPECTED_VERSION_WITHOUT_A_FILE,\n	                    FileOfTheExpectedVersion::WithoutAFile => EXPECTED_VERSION_WITH_A_FILE,\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_on_the_deleted_flow_against_a_version_table_without_a_file_from_the_intent_on	the_gpu_judge_agrees_with_the_cpu_on_the_deleted_flow_against_a_version_table_without_a_file_from_the_intent_on
第33批 GPU allrbmap：诞生代号记在第一条位置条目落点的走过表行上（那一行不存在就记不下），分叉形 I-4.9 漏判、与 CPU 不一致	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    // 记下这个单元的诞生代号（按读回那一份的落点认，与走过的单元表同一个键），再判 I-4.9\n    if (w_context == 0u) {\n        let visited_index = w_visited_index_in_the_main_walk(unit_device, unit_slot);\n	    // 记下这个单元的诞生代号（按读回那一份的落点认，与走过的单元表同一个键），再判 I-4.9\n    if (w_context == 0u) {\n        let visited_index = w_visited_index_in_the_main_walk(p.device0, p.slot0);\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_shared_leaf_container_born_after_the_rollback_target_reads_back_from_its_second_copy	the_gpu_judge_agrees_with_the_cpu_when_a_shared_leaf_container_born_after_the_rollback_target_reads_back_from_its_second_copy
第33批 GPU allrbmap：读回第二条那一份时 I-3.5 的落点表不改记成它自己，分叉形 I-3.5 漏判、与 CPU 不一致	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                if (sw(row + 3u) == p.device0 && sw(row + 4u) == p.slot0.x && sw(row + 5u) == p.slot0.y) {\n                    sws(row + 3u, p.device1);\n	                if (sw(row + 3u) == p.device0 && sw(row + 4u) == p.slot0.x && sw(row + 5u) == p.slot0.y && p.device1 == p.device0) {\n                    sws(row + 3u, p.device1);\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_shared_leaf_container_born_after_the_rollback_target_reads_back_from_its_second_copy	the_gpu_judge_agrees_with_the_cpu_when_a_shared_leaf_container_born_after_the_rollback_target_reads_back_from_its_second_copy
第33批 GPU 全零单元守卫照改之前按 offset.y >> (32 − SLOT_SHIFT) 判，槽号 2³² + 190000 回绕成 190000、交回全零单元	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    if (((offset.x & (NODE_BYTES - 1u)) | (offset.y >> SLOT_SHIFT)) != 0u || first_slot < location_word(before, 9u)\n	    if (((offset.x & (NODE_BYTES - 1u)) | (offset.y >> (32u - SLOT_SHIFT))) != 0u || first_slot < location_word(before, 9u)\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_past_thirty_two_bits_with_the_checksum_of_zeros	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_past_thirty_two_bits_with_the_checksum_of_zeros
第33批 GPU 全零单元守卫照改之前拿 first_slot + 读的槽数 比空段终点（32 位加法），槽 2³² − 1 回绕、交回全零单元	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	        || first_slot > first_slot_past_the_gap || first_slot_past_the_gap - first_slot < slots_read) {\n	        || first_slot_past_the_gap < first_slot + slots_read) {\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_in_thirty_two_bits_with_the_checksum_of_zeros	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_the_last_slot_in_thirty_two_bits_with_the_checksum_of_zeros
第33批 GPU F-α 那一处拿按 16 KiB 读回的切片的 32 KiB 检查字去比（不另按 32 KiB 读），没写过而下一槽写过的槽判对得上 32 KiB 全零的 CRC	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        || slice_crc_matches(read_at(device, slot_to_bytes(slot), DATA_UNIT_BYTES), DATA_UNIT_BYTES, checksum);\n	        || slice_crc_matches(copy, DATA_UNIT_BYTES, checksum);\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_slot_nothing_wrote_before_a_written_slot_with_the_checksum_of_thirty_two_zero_kibibytes
第33批 GPU F-ID-b 两份都没自证过时照改之前取第一份按 16 KiB 对得上的头里的类标签（C16 读成码 3、I-9.23 红，与 CPU 不一致）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            let class_of_the_child_of_a_type_zero_entry = w_class_of_the_first_copy_verifying_at_the_length_of_its_class(child);\n	            var class_of_the_child_of_a_type_zero_entry = w_class_of_the_first_copy_verifying_at_the_length_of_its_class(child);\n            if (class_of_the_child_of_a_type_zero_entry == NONE) {\n                let child_unit = read_unit_via_locations(child, NODE_BYTES);\n                if (child_unit.version != NONE) {\n                    class_of_the_child_of_a_type_zero_entry = rd8(child_unit.base, 6u);\n                }\n            }\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_child_whose_two_copies_do_not_verify	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_child_whose_two_copies_do_not_verify
第33批 GPU 恢复内核的 oracle 不认「没有文件」的那一版（落在它上面报没有文件判成这一版有文件），删完那条流与 CPU 不一致	crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl	        if (!version_has_a_file) {\n            return ORACLE_NONE;\n        }\n	        if (!version_has_a_file) {\n            return ORACLE_NO_FILE_WHERE_THIS_ROOT_HAS_ONE;\n        }\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_on_the_deleted_flow_against_a_version_table_without_a_file_from_the_intent_on	the_gpu_judge_agrees_with_the_cpu_on_the_deleted_flow_against_a_version_table_without_a_file_from_the_intent_on
第33批 GPU 恢复内核的 oracle 不在表里的根照「任一更旧版」判（不看比它旧的最新那一项有没有文件），删完那条流各批判成违例	crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl	            if (newest_older != NONE && tab(T_EXPECTED, newest_older * EXPECTED_WORDS + 5u) == EXPECTED_VERSION_WITH_A_FILE) {\n	            if (newest_older != NONE) {\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_on_the_deleted_flow_against_a_version_table_without_a_file_from_the_intent_on	the_gpu_judge_agrees_with_the_cpu_on_the_deleted_flow_against_a_version_table_without_a_file_from_the_intent_on
````

## 按名字整行替换的已有变异行（/home/fy5090/.local/share/singlefs/work/m3-batch33/s/patch/mutations-replacements.tsv）

````tsv
增补 2 收口第 26 行：走读数据单元时不判出生身份（I-1.2 / I-4.2 的调用点被摘掉）	crates/singlefs-checker/src/walk.rs	        self.judge_birth_identity_of_a_referenced_unit(\n            &unit,\n            &pointer,\n            placement_it_was_read_from,\n            "数据单元",\n        );\n	        let _ = &pointer;\n	-p singlefs-harness --test checker_known_bad_images	the_birth_identity_bad_images_redden_only_their_own_invariant
实审 B2 第 9 条（I-9.2 类型段 0）：类型段 0 指着的码 2 节点判完头不往下走	crates/singlefs-checker/src/walk.rs	        let mut leaves_in_leaf_order = Vec::new();\n        self.walk_inode_internal_node(\n            &view,\n            tree,\n            placement_it_was_read_from,\n            &mut leaves_in_leaf_order,\n        );\n	        let leaves_in_leaf_order = Vec::new();\n	-p singlefs-harness --test checker_narrow_invariants_and_abandoned_roots -- a_legal_type_zero_inode_entry_is_walked_down_to_the_leaf_and_every_invariant_holds	a_legal_type_zero_inode_entry_is_walked_down_to_the_leaf_and_every_invariant_holds
第 10 批 10b GPU 判器：走读内核读了谱系树根不往下走到容器（改回别的种类不走）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        w_walk_container_tree_root(read.n, tree, read.unit_device, read.unit_slot, container_record_type, fsid_low);	        // 变异：墓碑树、谱系树不往下走	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_of_a_rollback_publish_carries_another_record_width	the_gpu_judge_agrees_with_the_cpu_when_the_lineage_container_of_a_rollback_publish_carries_another_record_width
第24批 K I-9.2 同形：孩子头里的类与类型段说的不符也当相符（身份三段为 0 的 inode 条目指码 3 叶容器）	crates/singlefs-checker/src/walk.rs	            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n    })\n}\n	            .filter(|class_in_the_header| false && Some(*class_in_the_header) != class_the_entry_type_names)\n    })\n}\n	-p singlefs-harness --test checker_known_bad_images -- an_inode_tree_entry_of_type_zero_with_a_zero_identity_over_a_leaf_container_reddens_only_the_inode_entry_invariant	an_inode_tree_entry_of_type_zero_with_a_zero_identity_over_a_leaf_container_reddens_only_the_inode_entry_invariant
第24批 K 孩子的类不验整单元校验和（只改了类标签的叶容器被当成码 2 孩子、按一槽登记）	crates/singlefs-checker/src/walk.rs	        .read(location.device, offset, unit_bytes)\n        .filter(|content| crc32_castagnoli_table(content) == location.checksum)\n        .map(|_content| class_in_the_header)\n	        .read(location.device, offset, unit_bytes)\n        .map(|_content| class_in_the_header)\n	-p singlefs-harness --test checker_known_bad_images -- an_inode_leaf_whose_class_tag_alone_says_index_node_is_read_at_the_length_of_its_entry_type	an_inode_leaf_whose_class_tag_alone_says_index_node_is_read_at_the_length_of_its_entry_type
第24批 K M1：容器树类型段 0 那一支身份判定恒真	crates/singlefs-checker/src/walk.rs	                        && child_class.is_none_or(|class| class == MappingKeyUnitClass::IndexNode);\n	                        && child_class.is_none_or(|class| class == MappingKeyUnitClass::IndexNode)\n                        || true;\n	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_tree_entry_of_type_zero_with_a_container_number_reddens_only_the_tombstone_entry_invariant	a_tombstone_tree_entry_of_type_zero_with_a_container_number_reddens_only_the_tombstone_entry_invariant
第29批 GPU 判器 K1 J2：实例表链某一片读不出也当走到末片（最新根实例表第 1 片两份都读不出那条关门流上门开了，GPU 红 I-9.29、CPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        let s = w_read_unit_to_walk(pointer, DATA_UNIT_BYTES);\n        if (s.version == NONE) {\n            w_failures = w_failures + 1u;\n            break;\n        }	        let s = w_read_unit_to_walk(pointer, DATA_UNIT_BYTES);\n        if (s.version == NONE) {\n            w_failures = w_failures + 1u;\n            if (judge_rows) {\n                w_newest_instance_table_chain_reached_its_last_page = true;\n            }\n            break;\n        }	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_second_page_of_the_newest_instance_table_does_not_read_back	the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_second_page_of_the_newest_instance_table_does_not_read_back
第30批 X13：认孩子的类时第一份头里的类标签不在登记表里就不看第二份（第二份自证成码 2，I-9.2 报成立）	crates/singlefs-checker/src/walk.rs	    child.locations.iter().find_map(|location| {\n        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n    })\n}\n	    for location in &child.locations {\n        let Some(header) = reader.read(location.device, location.slot * SLOT_BYTES, node_bytes()) else {\n            continue;\n        };\n        let class_in_the_header = MappingKeyUnitClass::of(header[UNIT_HEADER_CLASS_TAG_OFFSET]);\n        if unit_bytes_of_the_class(class_in_the_header).is_none() {\n            break;\n        }\n        if let Some(class_verified) = class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_verified| Some(*class_verified) != class_the_entry_type_names)\n        {\n            return Some(class_verified);\n        }\n    }\n    None\n}\n	-p singlefs-harness --test checker_known_bad_images -- child_pointer_birth_tree_is_one_off_over_a_leaf whose_second_copy_verifies_as_a_node	an_inode_leaf_whose_first_copy_carries_an_unregistered_class_tag_and_whose_second_copy_verifies_as_a_node_reddens_the_inode_entry_invariant
第30批 X14：认孩子的类时第一份头读不出就不看第二份（第二份自证成码 2，I-9.2 报成立）	crates/singlefs-checker/src/walk.rs	    child.locations.iter().find_map(|location| {\n        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n    })\n}\n	    for location in &child.locations {\n        let Some(header) = reader.read(location.device, location.slot * SLOT_BYTES, node_bytes()) else {\n            break;\n        };\n        let class_in_the_header = MappingKeyUnitClass::of(header[UNIT_HEADER_CLASS_TAG_OFFSET]);\n        if unit_bytes_of_the_class(class_in_the_header).is_none() {\n            continue;\n        }\n        if let Some(class_verified) = class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_verified| Some(*class_verified) != class_the_entry_type_names)\n        {\n            return Some(class_verified);\n        }\n    }\n    None\n}\n	-p singlefs-harness --test checker_known_bad_images -- child_pointer_birth_tree_is_one_off_over_a_leaf whose_second_copy_verifies_as_a_node	an_inode_leaf_whose_first_copy_does_not_read_back_and_whose_second_copy_verifies_as_a_node_reddens_the_inode_entry_invariant
第30批 F-W：另一份按类型段那一类自证过也不往下走（三棵树都改回见别类就停；inode 叶容器带 blocks 违例、另一份自证成码 2，I-9.15 两种盘序都报不适用）	crates/singlefs-checker/src/walk.rs	    child.locations.iter().any(|location| {\n        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            == Some(class_the_entry_type_names)\n    })\n}\n	    child.locations.iter().take(0).any(|location| {\n        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            == Some(class_the_entry_type_names)\n    })\n}\n	-p singlefs-harness --test checker_known_bad_images -- beside_a_copy_verifying_as_a_node	an_inode_leaf_with_blocks_off_the_logical_length_beside_a_copy_verifying_as_a_node_reddens_the_blocks_invariant_on_either_device
第32批 F-ID-b：墓碑树、谱系树类型段 0 那一支认「孩子是码 2」取第一份按 16 KiB 对得上的副本（c16 在位置条目 0 时 I-9.23 / I-9.17 红，换盘不红）	crates/singlefs-checker/src/walk.rs	                    let child_class = class_of_the_first_copy_verifying_at_the_length_of_its_class(\n                        self.reader,\n                        &child,\n                    );\n	                    let child_class = read_unit_without_judging(self.reader, &child, node_bytes())\n                        .map(|unit| MappingKeyUnitClass::of(unit[UNIT_HEADER_CLASS_TAG_OFFSET]));\n	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_whose_only_leaf_copy_matching_at_the_container_length_is_a_data_unit_does_not_go_down_that_copy	a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device
第32批 S4：认「另一份按类型段那一类自证过」不看头里的类标签（inode 叶容器盘 1 那份是码 1 数据单元、按 32 KiB 对得上，照类型段那一支往下走）	crates/singlefs-checker/src/walk.rs	    child.locations.iter().any(|location| {\n        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            == Some(class_the_entry_type_names)\n    })\n}\n	    child.locations.iter().any(|location| {\n        reader\n            .read(\n                location.device,\n                location.slot * SLOT_BYTES,\n                unit_bytes_of_the_class(class_the_entry_type_names).expect("码 2 或码 3"),\n            )\n            .is_some_and(|content| crc32_castagnoli_table(&content) == location.checksum)\n    })\n}\n	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_whose_only_leaf_copy_matching_at_the_container_length_is_a_data_unit_does_not_go_down_that_copy	an_inode_tree_entry_whose_only_leaf_copy_matching_at_the_container_length_is_a_data_unit_does_not_go_down_that_copy
第32批 GPU 判器 F-KEY：走读内核「走过」按第一条位置条目认（墓碑容器记录宽 39、墓碑树根条目第 0 条位置条目指着 inode 树根那条流上 CPU 红 I-1.7，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    if (s.version != NONE && w_read_back_from_the_second_location) {\n	    if (false && s.version != NONE && w_read_back_from_the_second_location) {\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_the_first_location_of_a_tombstone_container_with_a_wrong_record_width_names_a_node_walked_before	the_gpu_judge_agrees_with_the_cpu_when_the_first_location_of_a_tombstone_container_with_a_wrong_record_width_names_a_node_walked_before
第32批 GPU 判器 F-ID-b：走读内核类型段 0 认孩子的类取第一份按 16 KiB 对得上的（墓碑树根条目类型段 0、c16 在位置条目 0 那条流上 GPU 红 I-9.23，CPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            let class_of_the_child_of_a_type_zero_entry = w_class_of_the_first_copy_verifying_at_the_length_of_its_class(child);\n	            let class_of_the_child_of_a_type_zero_entry = NONE;\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_checked_at_sixteen_kibibytes_beside_a_node	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_checked_at_sixteen_kibibytes_beside_a_node
第32批 F-KEY：走读认「走过」按第 0 条位置条目的落点、不按读回那一份（树根条目第 0 条位置条目指着先走过的码 2 节点、盘 1 那份容器记录宽减 1，I-1.7 报成立）	crates/singlefs-checker/src/walk.rs	        Some((unit, placement_it_was_read_from))\n    }\n	        Some((unit, first_placement))\n    }\n	-p singlefs-harness --test checker_known_bad_images -- a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before a_tombstone_tree_entry_of_type_zero_over_a_container_checked_at_sixteen_kibibytes_beside_a_node_leaves_the_tombstone_entry_invariant_holding_on_either_device a_tombstone_tree_entry_of_type_zero_in_a_root_of_level_zero_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_goes_down_the_type_zero_branch_on_either_device an_inode_tree_entry_whose_only_leaf_copy_matching_at_the_container_length_is_a_data_unit_does_not_go_down_that_copy	a_tombstone_container_with_a_wrong_record_width_reddens_the_packed_container_invariant_whichever_location_names_a_node_walked_before
第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（墓碑树 N4-i 加陈旧副本，I-9.23 随陈旧副本在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	    child.locations.iter().find_map(|location| {\n        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n    })\n}\n	    for location in &child.locations {\n        let Some(header) = reader.read(location.device, location.slot * SLOT_BYTES, node_bytes()) else {\n            continue;\n        };\n        let class_in_the_header = MappingKeyUnitClass::of(header[UNIT_HEADER_CLASS_TAG_OFFSET]);\n        if unit_bytes_of_the_class(class_in_the_header).is_none() {\n            continue;\n        }\n        if Some(class_in_the_header) == class_the_entry_type_names {\n            break;\n        }\n        if let Some(class_verified) = class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_verified| Some(*class_verified) != class_the_entry_type_names)\n        {\n            return Some(class_verified);\n        }\n    }\n    None\n}\n	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	a_tombstone_tree_entry_of_type_zero_over_a_container_with_a_stale_copy_of_the_child_reddens_the_tombstone_entry_invariant_on_either_device
第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（层级 2 墓碑树 N4-f 加陈旧副本，I-9.23 报成立、连带 I-3.1、I-3.11、I-5.1、I-7.2）	crates/singlefs-checker/src/walk.rs	    child.locations.iter().find_map(|location| {\n        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n    })\n}\n	    for location in &child.locations {\n        let Some(header) = reader.read(location.device, location.slot * SLOT_BYTES, node_bytes()) else {\n            continue;\n        };\n        let class_in_the_header = MappingKeyUnitClass::of(header[UNIT_HEADER_CLASS_TAG_OFFSET]);\n        if unit_bytes_of_the_class(class_in_the_header).is_none() {\n            continue;\n        }\n        if Some(class_in_the_header) == class_the_entry_type_names {\n            break;\n        }\n        if let Some(class_verified) = class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_verified| Some(*class_verified) != class_the_entry_type_names)\n        {\n            return Some(class_verified);\n        }\n    }\n    None\n}\n	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	a_tombstone_tree_entry_of_the_container_type_over_a_node_with_a_stale_copy_of_the_child_reddens_the_tombstone_entry_invariant_on_either_device
第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（谱系树类型段 0 加陈旧副本，I-9.17 随陈旧副本在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	    child.locations.iter().find_map(|location| {\n        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n    })\n}\n	    for location in &child.locations {\n        let Some(header) = reader.read(location.device, location.slot * SLOT_BYTES, node_bytes()) else {\n            continue;\n        };\n        let class_in_the_header = MappingKeyUnitClass::of(header[UNIT_HEADER_CLASS_TAG_OFFSET]);\n        if unit_bytes_of_the_class(class_in_the_header).is_none() {\n            continue;\n        }\n        if Some(class_in_the_header) == class_the_entry_type_names {\n            break;\n        }\n        if let Some(class_verified) = class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_verified| Some(*class_verified) != class_the_entry_type_names)\n        {\n            return Some(class_verified);\n        }\n    }\n    None\n}\n	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	a_lineage_tree_entry_of_type_zero_over_a_container_with_a_stale_copy_of_the_child_reddens_the_lineage_entry_invariant_on_either_device
第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（inode 树类型段 0 加陈旧副本，I-9.2 报成立、连带 I-7.2 等）	crates/singlefs-checker/src/walk.rs	    child.locations.iter().find_map(|location| {\n        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n    })\n}\n	    for location in &child.locations {\n        let Some(header) = reader.read(location.device, location.slot * SLOT_BYTES, node_bytes()) else {\n            continue;\n        };\n        let class_in_the_header = MappingKeyUnitClass::of(header[UNIT_HEADER_CLASS_TAG_OFFSET]);\n        if unit_bytes_of_the_class(class_in_the_header).is_none() {\n            continue;\n        }\n        if Some(class_in_the_header) == class_the_entry_type_names {\n            break;\n        }\n        if let Some(class_verified) = class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_verified| Some(*class_verified) != class_the_entry_type_names)\n        {\n            return Some(class_verified);\n        }\n    }\n    None\n}\n	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	an_inode_tree_entry_of_type_zero_over_a_leaf_container_with_a_stale_copy_of_the_child_reddens_the_inode_entry_invariant_on_either_device
第28批 H-V 认孩子的类在第一份自证过的副本处停（inode 树条目不改、叶容器一份自证成码 2，条目那一条随那一份在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	    child.locations.iter().find_map(|location| {\n        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n    })\n}\n	    child\n        .locations\n        .iter()\n        .find_map(|location| class_of_a_copy_verifying_at_the_length_of_its_class(reader, location))\n        .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n}\n	-p singlefs-harness --test checker_known_bad_images -- verifying_as_a_node_on_one_device	an_inode_tree_entry_whose_leaf_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_inode_entry_invariant_alike_on_either_device
第28批 H-V 认孩子的类在第一份自证过的副本处停（inode 树条目改成类型段 0、叶容器一份自证成码 2，条目那一条随那一份在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	    child.locations.iter().find_map(|location| {\n        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n    })\n}\n	    child\n        .locations\n        .iter()\n        .find_map(|location| class_of_a_copy_verifying_at_the_length_of_its_class(reader, location))\n        .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n}\n	-p singlefs-harness --test checker_known_bad_images -- verifying_as_a_node_on_one_device	an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_inode_entry_invariant_alike_on_either_device
第28批 H-V 认孩子的类在第一份自证过的副本处停（墓碑树条目不改、容器一份自证成码 2，条目那一条随那一份在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	    child.locations.iter().find_map(|location| {\n        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n    })\n}\n	    child\n        .locations\n        .iter()\n        .find_map(|location| class_of_a_copy_verifying_at_the_length_of_its_class(reader, location))\n        .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n}\n	-p singlefs-harness --test checker_known_bad_images -- verifying_as_a_node_on_one_device	a_tombstone_tree_entry_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_tombstone_entry_invariant_alike_on_either_device
第28批 H-V 认孩子的类在第一份自证过的副本处停（墓碑树条目改成类型段 0、容器一份自证成码 2，条目那一条随那一份在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	    child.locations.iter().find_map(|location| {\n        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n    })\n}\n	    child\n        .locations\n        .iter()\n        .find_map(|location| class_of_a_copy_verifying_at_the_length_of_its_class(reader, location))\n        .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n}\n	-p singlefs-harness --test checker_known_bad_images -- verifying_as_a_node_on_one_device	a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_tombstone_entry_invariant_alike_on_either_device
第28批 H-V 认孩子的类在第一份自证过的副本处停（谱系树条目不改、容器一份自证成码 2，条目那一条随那一份在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	    child.locations.iter().find_map(|location| {\n        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n    })\n}\n	    child\n        .locations\n        .iter()\n        .find_map(|location| class_of_a_copy_verifying_at_the_length_of_its_class(reader, location))\n        .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n}\n	-p singlefs-harness --test checker_known_bad_images -- verifying_as_a_node_on_one_device	a_lineage_tree_entry_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_lineage_entry_invariant_alike_on_either_device
第28批 H-V 认孩子的类在第一份自证过的副本处停（谱系树条目改成类型段 0、容器一份自证成码 2，条目那一条随那一份在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	    child.locations.iter().find_map(|location| {\n        class_of_a_copy_verifying_at_the_length_of_its_class(reader, location)\n            .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n    })\n}\n	    child\n        .locations\n        .iter()\n        .find_map(|location| class_of_a_copy_verifying_at_the_length_of_its_class(reader, location))\n        .filter(|class_in_the_header| Some(*class_in_the_header) != class_the_entry_type_names)\n}\n	-p singlefs-harness --test checker_known_bad_images -- verifying_as_a_node_on_one_device	a_lineage_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_lineage_entry_invariant_alike_on_either_device
````
