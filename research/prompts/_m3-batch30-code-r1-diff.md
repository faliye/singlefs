# 删除批第 30 批合入的改动原样（2026-10-06）

## 实现员 m：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch30/m/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -2268,7 +2268,9 @@
     /// 墓碑树或谱系树的码 2 根（或根长高之后的内部节点）往下走：条目 120 字节（分隔 key 8 + 身份引用 26 + 子指针 86），
     /// 类型段 = `record_type` 的走到码 3 容器（判容器头、出生树、容器格式，单元记进这一版走到的落点），类型段 0 的走进下一层码 2 节点
     /// （层级 = 父层级 − 1），别的类型段判 `identity_invariant` 红、不往下走；孩子头里自证过的类标签与类型段说的那一类不符的同样判红、
-    /// 不往下走（[`Self::judge_a_container_tree_entry_whose_child_is_of_another_class`]，孩子读多长按它头里的类标签定、不按类型段定）；
+    /// 不往下走（[`Self::judge_a_container_tree_entry_whose_child_is_of_another_class`]，孩子读多长按它头里的类标签定、不按类型段定）——
+    /// 另有一份头里就是类型段那一类、按那一类的长度自证得过的除外：判红之后照类型段那一支走那一份（F-W，
+    /// [`some_copy_of_the_child_verifies_as_the_class_the_entry_type_names`]）；
     /// 这两支不往下走的孩子每一份副本照判 I-2.1（[`Self::judge_every_copy_of_a_child_the_walk_does_not_go_into`]）。每条条目的身份按 `identity_invariant`（墓碑树 I-9.23、
     /// 谱系树 I-9.17）单独判定，身份不符不记走读失败（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T7 取 a）。
     /// 用 checker 自己的读法，不调 `singlefs_core`（D13（验证路线） 已定项 5）。
@@ -2337,15 +2339,31 @@
                     &child,
                     Some(class_the_entry_type_names),
                 ) {
-                    self.judge_a_container_tree_entry_whose_child_is_of_another_class(
+                    // 另一份头里就是类型段那一类、按那一类的长度自证得过（F-W，判决
+                    // `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「K2 H-V 两份都看之后判错」那一行、第三节第 1 条）：
+                    // 读者按类型段那一类的长度逐份读、用对得上的那一份，那一份的子树照类型段那一支走、照判（槽数按类型段那一类登记）；
+                    // 条目那一条照判红。没有这样一份才不往下走。
+                    if some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(
+                        self.reader,
                         &child,
-                        entry_type,
                         class_the_entry_type_names,
-                        child_class,
-                        identity_invariant,
-                        name,
-                    );
-                    continue;
+                    ) {
+                        self.judgements.judge(identity_invariant, false, || {
+                            format!(
+                                "{name}内部条目类型段 {entry_type} 说孩子是 {class_the_entry_type_names:?}，孩子头里的类标签是 {child_class:?}（按那一类的长度读回、整单元校验和对得上）；另一份头里就是类型段那一类、按那一类的长度自证得过，照类型段那一支往下走那一份"
+                            )
+                        });
+                    } else {
+                        self.judge_a_container_tree_entry_whose_child_is_of_another_class(
+                            &child,
+                            entry_type,
+                            class_the_entry_type_names,
+                            child_class,
+                            identity_invariant,
+                            name,
+                        );
+                        continue;
+                    }
                 }
             }
             match ContainerTreeEntryType::of(entry_type, record_type) {
@@ -2405,7 +2423,7 @@
 
     /// 墓碑树、谱系树里一条条目的类型段说孩子是一类（类型段 0 ⇒ 码 2，类型段 = 打包记录类型 ⇒ 码 3），孩子头里自证过的类标签是另一类：
     /// I-9.23 / I-9.17 字面「0 ⇒ 子单元是码 2 … 1（谱系 5）⇒ 子单元是码 3」不成立，按 `identity_invariant` 判红，
-    /// 「身份不符单独判定，不再记走读失败」：不往下走、不记走读失败。孩子按它自己那一类占的槽先登记引用（同类型段不认得那一支：
+    /// 「身份不符单独判定，不再记走读失败」：不往下走、不记走读失败（只在没有一份按类型段那一类自证过时走到这里，F-W）。孩子按它自己那一类占的槽先登记引用（同类型段不认得那一支：
     /// 不登记的话它占的槽不进引用并集，I-3.1（已分配统计对得上） 连带红）。每一份副本照判 I-2.1
     /// （[`Self::judge_every_copy_of_a_child_the_walk_does_not_go_into`]，第 29 批，H-J3b：另一份校验和对不上的照红）。
     fn judge_a_container_tree_entry_whose_child_is_of_another_class(
@@ -2677,6 +2695,7 @@
             // `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N4 那一行「inode 树 I-9.2 同形」）：
             // 孩子自证过的类与类型段说的不符（类型段不认得的，孩子读得出）判 I-9.2 红，按它自己那一类占的槽登记引用，不往下走、不记走读失败；
             // 每一份副本照判 I-2.1（第 29 批，H-J3b：自证成另一类的之外那一份校验和对不上的照红）。
+            // 另有一份按类型段那一类自证过的，判红之后照类型段那一支往下走那一份（第 30 批，F-W，见下）。
             let class_the_entry_type_names = match entry_type {
                 InodeInternalEntryType::LeafContainer => {
                     Some(MappingKeyUnitClass::PackedRecordUnit)
@@ -2684,11 +2703,37 @@
                 InodeInternalEntryType::InternalNode => Some(MappingKeyUnitClass::IndexNode),
                 InodeInternalEntryType::Unregistered(_) => None,
             };
-            if let Some(child_class) = child_unit_class_other_than_the_entry_type_names(
-                self.reader,
-                &child,
-                class_the_entry_type_names,
-            ) {
+            // 认出另一类、另有一份头里就是类型段那一类且按那一类的长度自证得过（F-W，同墓碑树、谱系树那一处，
+            // `Self::walk_the_entries_of_a_container_tree_node`）：I-9.2 照判红，再照类型段那一支往下走那一份（槽数按类型段那一类登记）。
+            let child_class_the_walk_does_not_go_below =
+                match child_unit_class_other_than_the_entry_type_names(
+                    self.reader,
+                    &child,
+                    class_the_entry_type_names,
+                ) {
+                    None => None,
+                    Some(child_class) => {
+                        let a_copy_of_the_class_the_entry_type_names_verifies =
+                            class_the_entry_type_names.is_some_and(|class| {
+                                some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(
+                                    self.reader,
+                                    &child,
+                                    class,
+                                )
+                            });
+                        if a_copy_of_the_class_the_entry_type_names_verifies {
+                            self.judgements.judge("I-9.2", false, || {
+                                format!(
+                                    "inode 内部条目类型段 {record_type} 说孩子是 {class_the_entry_type_names:?}，孩子头里的类标签是 {child_class:?}（按那一类的长度读回、整单元校验和对得上）；另一份头里就是类型段那一类、按那一类的长度自证得过，照类型段那一支往下走那一份"
+                                )
+                            });
+                            None
+                        } else {
+                            Some(child_class)
+                        }
+                    }
+                };
+            if let Some(child_class) = child_class_the_walk_does_not_go_below {
                 let span_in_slots = slots_spanned_by_a_unit_of_the_class(child_class).expect(
                     "child_unit_class_other_than_the_entry_type_names 只交回有长度的登记码",
                 );
@@ -4392,6 +4437,31 @@
     class_other_than_the_entry_type_names
 }
 
+/// [`child_unit_class_other_than_the_entry_type_names`] 认出另一类之后再问一句：有没有一份副本头里的类标签就是 `class_the_entry_type_names`、
+/// 按那一类的长度读回的整单元校验和对得上（判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「K2 H-V 两份都看之后判错」那一行、
+/// 第三节第 1 条，攻方改法 F-W）。有的话读者用的就是这一份（core 按条目那一类的长度逐份读、对得上就用），走读照类型段那一支走它的子树；
+/// 槽数也按这一类登记（C739 后一半）。只在认出另一类之后调：常见的条目不多读。只读、不判。
+///
+/// # Panics
+/// `class_the_entry_type_names` 没有长度：调用方只拿类型段认得的那两类（码 2、码 3）来问。
+fn some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(
+    reader: &dyn ImageReader,
+    child: &PointerView,
+    class_the_entry_type_names: MappingKeyUnitClass,
+) -> bool {
+    let unit_bytes = unit_bytes_of_the_class(class_the_entry_type_names)
+        .expect("类型段认得的那一类是码 2 或码 3，都有长度");
+    child.locations.iter().any(|location| {
+        reader
+            .read(location.device, location.slot * SLOT_BYTES, unit_bytes)
+            .is_some_and(|content| {
+                MappingKeyUnitClass::of(content[UNIT_HEADER_CLASS_TAG_OFFSET])
+                    == class_the_entry_type_names
+                    && crc32_castagnoli_table(&content) == location.checksum
+            })
+    })
+}
+
 /// 走读不往下读的孩子的一份副本（一条位置条目）与位置条目里的整单元校验和对不对得上：头按最短的单元（码 2，16 KiB）那么长读，
 /// 头里的类标签是登记过的码就只按那一类的长度比；没登记的码、或头读不出，16 KiB 与 32 KiB 两种长度都对不上才算对不上
 /// （判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行的改法 F-α）。只读、不判。
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -1634,6 +1634,12 @@
         let p = packed.p;
         // I-1.12：指针的出生树等于被指单元头偏移 43 的出生树（码 3）
         w_judge(INVARIANT_I_1_12, eq64(pointer.birth_tree, p.birth_tree));
+        // I-1.2 遍历方向那一半（walk.rs walk_instance_table_chain 每一片都调 judge_birth_identity_against_the_pointer）：头里的诞生代号（码 3 偏移 73）、
+        // 写序实例（93）与出生序号（103）与指着这一片的指针记的相同，偏移同 w_judge_birth_identity 码 3 那一支。只判 I-1.2、不判 I-4.2：
+        // 打包记录类型 4 的实例表单元按 I-1.2 那一行的例外只免谓词那一半（判决 `research/prompts/m3-batch28-code-r1-main-verification.md`
+        // 第二节「实例表链上的 I-1.2」那一行、第三节第 2 条，攻方改法 F-IT）
+        w_judge(INVARIANT_I_1_2, eq64(rd64(s.base, 73u), pointer.birth_txg) && rd32(s.base, 93u) == pointer.instance
+            && eq64(vec2<u32>(rd32(s.base, 103u), 0u), pointer.tail));
         let identity_holds = is_zero64(p.birth_tree) && p.record_type == 4u && eq64(p.container, page_index) && is_zero64(p.container_birth);
         w_judge(INVARIANT_I_1_1, identity_holds);
         if (!identity_holds) {
@@ -2666,7 +2672,15 @@
         } else if (record_type == 0u) {
             class_the_inode_entry_type_names = 2u;
         }
-        let class_of_the_inode_child = w_child_unit_class_other_than_the_entry_type_names(child, class_the_inode_entry_type_names);
+        var class_of_the_inode_child = w_child_unit_class_other_than_the_entry_type_names(child, class_the_inode_entry_type_names);
+        // 认出另一类、另有一份头里就是类型段那一类且按那一类的长度自证得过（walk.rs walk_inode_internal_node 的 F-W，判决
+        // `research/prompts/m3-batch28-code-r1-main-verification.md` 第三节第 1 条）：I-9.2 照判红，再照类型段那一支往下走那一份（槽数按类型段那一类登记）
+        if (class_of_the_inode_child != NONE && class_the_inode_entry_type_names != NONE
+            && w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child, class_the_inode_entry_type_names)) {
+            // 身份成立要孩子没有一份自证成别类：这里恒不成立
+            w_judge(INVARIANT_I_9_2, class_of_the_inode_child == NONE);
+            class_of_the_inode_child = NONE;
+        }
         if (class_of_the_inode_child != NONE) {
             w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_inode_child));
             w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
@@ -2987,7 +3001,8 @@
 // 孩子是哪一类按它头里的类标签认、不按类型段定读多长（w_child_unit_class_other_than_the_entry_type_names，判决
 // `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 N4 那一行：类型段 0 指着码 3 容器、类型段 = record_type 指着码 2 节点时，
 // 按类型段的长度读回的整单元校验和对不上，身份那一判跑不到、却记了 I-2.1 与走读失败）：头里是另一类、按那一类的长度自证得过的，
-// 判身份红、按它自己那一类占的槽登记引用（码 2 一槽、码 1 / 码 3 两槽）、不往下走、不记走读失败；别的照旧按类型段走。
+// 判身份红、按它自己那一类占的槽登记引用（码 2 一槽、码 1 / 码 3 两槽）、不往下走、不记走读失败；另有一份按类型段那一类自证得过的，
+// 判身份红之后照类型段走那一份（F-W，w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names）；别的照旧按类型段走。
 // 往下记的落点都记在正在走的那个码 2 节点下面（w_parent_*）。walk.rs 是递归，这里是同一个先序的显式栈，
 // 深度超过 w_container_tree_frames 的 12 层判不了；两个种类只经这一处调用点
 struct WalkContainerTreeFrame {
@@ -3040,6 +3055,20 @@
     return class_other_than_the_entry_type_names;
 }
 
+// w_child_unit_class_other_than_the_entry_type_names 认出另一类之后再问一句（walk.rs 的 some_copy_of_the_child_verifies_as_the_class_the_entry_type_names，
+// 判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第三节第 1 条，攻方改法 F-W）：有没有一份副本头里的类标签就是 class_the_entry_type_names、
+// 按那一类的长度读回的整单元校验和对得上。有的话读者用的就是这一份，走读照类型段那一支走它的子树。只在认出另一类之后调
+fn w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child: Ptr, class_the_entry_type_names: u32) -> bool {
+    let first_copy = read_at(child.device0, slot_to_bytes(child.slot0), NODE_BYTES);
+    if (first_copy.version != NONE && rd8(first_copy.base, 6u) == class_the_entry_type_names
+        && w_unit_verifies_at_the_length_of_its_class(first_copy, class_the_entry_type_names, child.sum0)) {
+        return true;
+    }
+    let second_copy = read_at(child.device1, slot_to_bytes(child.slot1), NODE_BYTES);
+    return second_copy.version != NONE && rd8(second_copy.base, 6u) == class_the_entry_type_names
+        && w_unit_verifies_at_the_length_of_its_class(second_copy, class_the_entry_type_names, child.sum1);
+}
+
 // 走读不往下读的孩子的一份副本与位置条目里的整单元校验和对不对得上（walk.rs 的 copy_matches_its_location_checksum_at_the_length_its_header_names）：
 // 头按最短的单元（码 2，16 KiB）那么长读，头里的类标签是登记过的码就只按那一类的长度比；没登记的码、或头读不出，16 KiB 与 32 KiB 两种长度都对不上
 // 才算对不上（判决 `research/prompts/m3-batch27-code-r1-main-verification.md` 第二节 H-J3a 那一行的改法 F-α）
@@ -3122,7 +3151,12 @@
         w_judge_pointer_extent_offset(child);
         if (entry_type == 0u) {
             // 孩子头里是码 2 之外的一类、按那一类的长度自证得过：身份不符，按它自己那一类占的槽登记引用，不往下走、不记走读失败
-            let class_of_the_child_of_an_internal_entry = w_child_unit_class_other_than_the_entry_type_names(child, 2u);
+            var class_of_the_child_of_an_internal_entry = w_child_unit_class_other_than_the_entry_type_names(child, 2u);
+            // 另有一份码 2、按 16 KiB 自证得过（walk.rs walk_the_entries_of_a_container_tree_node 的 F-W）：身份照判红，再照类型段 0 那一支往下走那一份
+            if (class_of_the_child_of_an_internal_entry != NONE && w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child, 2u)) {
+                w_judge(identity_invariant, class_of_the_child_of_an_internal_entry == NONE);
+                class_of_the_child_of_an_internal_entry = NONE;
+            }
             if (class_of_the_child_of_an_internal_entry != NONE) {
                 w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_child_of_an_internal_entry));
                 w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
@@ -3170,7 +3204,12 @@
             continue;
         }
         // 类型段 = record_type：孩子头里是码 3 之外的一类、按那一类的长度自证得过，身份不符（「类标签 3」那一小句），不按数据单元长度去读、不往下走
-        let class_of_the_child_of_a_container_entry = w_child_unit_class_other_than_the_entry_type_names(child, 3u);
+        var class_of_the_child_of_a_container_entry = w_child_unit_class_other_than_the_entry_type_names(child, 3u);
+        // 另有一份码 3、按 32 KiB 自证得过（walk.rs walk_the_entries_of_a_container_tree_node 的 F-W）：身份照判红，再照这一支往下走那一份（两槽登记）
+        if (class_of_the_child_of_a_container_entry != NONE && w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(child, 3u)) {
+            w_judge(identity_invariant, class_of_the_child_of_a_container_entry == NONE);
+            class_of_the_child_of_a_container_entry = NONE;
+        }
         if (class_of_the_child_of_a_container_entry != NONE) {
             w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_child_of_a_container_entry));
             w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -12044,6 +12044,292 @@
     );
 }
 
+// ─── 第 30 批（删除线）：H-V 子树（F-W）、inode 读不出孩子时子指针出生树大一、认孩子的类时第一份坏而第二份自证成别类 ───
+
+/// F-W 的判法（判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「K2 H-V 两份都看之后判错」那一行、第三节第 1 条；
+/// 攻方 `research/prompts/m3-batch28-code-r1-opus-output.md` 第 1.1 节那几形）：`with_the_violation` 是 `clean` 上把种类 `kind` 那棵树根
+/// 第一条条目指的那片码 3 容器两份一起改坏（重封、整单元校验和沿引用链补到根）的镜像，只它就红 `subtree_invariant`。
+/// 再做 H-V 那一步（[`image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device`]，条目不改）：一份换成这棵树根的码 2 字节、
+/// 按改过的位置条目自证得过，另一份就是那片带违例、按自己的位置条目自证得过的码 3 容器（类型段说的那一类，读者用的就是它）。
+/// 两种盘序（码 2 那一份在位置条目 0 / 1）各判一次：`subtree_invariant` 红（子树照类型段那一支走、照判）、`entry_invariant` 红；
+/// 槽数按类型段那一类登记，不连带 I-3.1、I-3.11、I-5.1；两种盘序红的同一批。
+/// 对照：同一形只做 H-V、不放违例，`subtree_invariant` 不红（红的是那片容器里的违例，不是 H-V 那一步带出来的）。
+/// 改之前（第 28、29 批）「孩子是另一类」那一支见别类就不往下走：`subtree_invariant` 两种盘序都报不适用或成立。
+fn assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
+    clean: &MemoryPool,
+    with_the_violation: &MemoryPool,
+    kind: u16,
+    entry_invariant: &'static str,
+    subtree_invariant: &'static str,
+) {
+    let violated_by_the_violation_only =
+        violated_invariant_set(&check_pool_image(with_the_violation));
+    assert!(
+        violated_by_the_violation_only.contains(subtree_invariant),
+        "对照：只放违例、不做 H-V，{subtree_invariant} 红（{violated_by_the_violation_only:?}）"
+    );
+    let mut violated_on_each_device = Vec::new();
+    for location_index in 0..2 {
+        let without_the_violation =
+            image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
+                clean,
+                kind,
+                location_index,
+                &leave_the_entry_as_it_is,
+            );
+        let violated_without_the_violation =
+            violated_invariant_set(&check_pool_image(&without_the_violation));
+        assert!(
+            !violated_without_the_violation.contains(subtree_invariant),
+            "对照：码 2 那一份在位置条目 {location_index}、不放违例，{subtree_invariant} 不红（{violated_without_the_violation:?}）"
+        );
+        let image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
+            with_the_violation,
+            kind,
+            location_index,
+            &leave_the_entry_as_it_is,
+        );
+        let verdicts = check_pool_image(&image);
+        let violated = violated_invariant_set(&verdicts);
+        assert!(
+            violated.contains(subtree_invariant),
+            "码 2 那一份在位置条目 {location_index}：类型段那一类那一份的子树照走，里面的违例 {subtree_invariant} 红：{verdicts:?}"
+        );
+        assert!(
+            violated.contains(entry_invariant),
+            "码 2 那一份在位置条目 {location_index}：条目那一条 {entry_invariant} 照判红：{verdicts:?}"
+        );
+        for invariant_a_wrong_slot_count_drags_in in ["I-3.1", "I-3.11", "I-5.1"] {
+            assert!(
+                !violated.contains(invariant_a_wrong_slot_count_drags_in),
+                "码 2 那一份在位置条目 {location_index}：槽数按类型段那一类登记，不连带 {invariant_a_wrong_slot_count_drags_in}：{verdicts:?}"
+            );
+        }
+        violated_on_each_device.push(violated);
+    }
+    assert_eq!(
+        violated_on_each_device[0], violated_on_each_device[1],
+        "同一张镜像只差码 2 那一份在哪块盘：红的同一批"
+    );
+}
+
+/// inode 记录预留段（记录偏移 120 起 20 字节，I-9.7 那三段的最后一段）的最后一个字节。
+const INODE_RECORD_LAST_RESERVED_BYTE_OFFSET: usize = 139;
+
+/// 攻方 1.1 inode 那一形（I-9.7）：新池新建文件那一版，inode 叶容器第一条记录的预留段写一个非零字节（两份、重封、补到根）。
+#[test]
+fn an_inode_leaf_with_a_non_zero_reserved_byte_beside_a_copy_verifying_as_a_node_reddens_the_reserved_bytes_invariant_on_either_device(
+) {
+    let clean = build_pool("known-bad-inode-leaf-reserved-byte-beside-a-copy-verifying-as-a-node")
+        .memory_pool();
+    let mut with_the_violation = clean.clone();
+    mutate_unit(
+        &mut with_the_violation,
+        INODE_LEAF,
+        |bytes| bytes[INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_LAST_RESERVED_BYTE_OFFSET] = 1,
+        true,
+    );
+    assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
+        &clean,
+        &with_the_violation,
+        TREE_KIND_INODE,
+        "I-9.2",
+        "I-9.7",
+    );
+}
+
+/// 攻方 1.1 inode 那一形（I-9.15）：新池新建文件那一版，inode 叶容器第一条记录的 blocks 加 1（size 3000 ⇒ 要 6，写成 7）。
+#[test]
+fn an_inode_leaf_with_blocks_off_the_logical_length_beside_a_copy_verifying_as_a_node_reddens_the_blocks_invariant_on_either_device(
+) {
+    let clean =
+        build_pool("known-bad-inode-leaf-blocks-beside-a-copy-verifying-as-a-node").memory_pool();
+    let mut with_the_violation = clean.clone();
+    mutate_unit(
+        &mut with_the_violation,
+        INODE_LEAF,
+        |bytes| {
+            let blocks_offset = INODE_LEAF_FIRST_RECORD_START + INODE_RECORD_BLOCKS_OFFSET;
+            let blocks = get_u64(bytes, blocks_offset);
+            set_u64(bytes, blocks_offset, blocks + 1);
+        },
+        true,
+    );
+    assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
+        &clean,
+        &with_the_violation,
+        TREE_KIND_INODE,
+        "I-9.2",
+        "I-9.15",
+    );
+}
+
+/// 打包记录类型 `record_bytes`（登记的记录宽）少 1：I-1.7 记录宽那一步判红。
+fn record_width_one_below(record_bytes: u64) -> u16 {
+    u16::try_from(record_bytes - 1).expect("登记的记录宽装得进 u16")
+}
+
+/// 攻方 1.1 inode 那一形（I-1.7）：新池新建文件那一版，inode 叶容器头里的记录宽 140 → 139。
+#[test]
+fn an_inode_leaf_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device(
+) {
+    let clean = build_pool("known-bad-inode-leaf-record-width-beside-a-copy-verifying-as-a-node")
+        .memory_pool();
+    let mut with_the_violation = clean.clone();
+    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
+        &mut with_the_violation,
+        TREE_KIND_INODE,
+        record_width_one_below(singlefs_format::INODE_RECORD_BYTES),
+    );
+    assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
+        &clean,
+        &with_the_violation,
+        TREE_KIND_INODE,
+        "I-9.2",
+        "I-1.7",
+    );
+}
+
+/// 攻方 1.1 墓碑那一形（I-1.7）：三个单元的文件发完意图之后，墓碑容器头里的记录宽 40 → 39。
+#[test]
+fn a_tombstone_container_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device(
+) {
+    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+        "known-bad-tombstone-record-width-beside-a-copy-verifying-as-a-node",
+    );
+    let mut with_the_violation = clean.clone();
+    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
+        &mut with_the_violation,
+        TREE_KIND_TOMBSTONE,
+        record_width_one_below(singlefs_format::TOMBSTONE_RECORD_BYTES),
+    );
+    assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
+        &clean,
+        &with_the_violation,
+        TREE_KIND_TOMBSTONE,
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        "I-1.7",
+    );
+}
+
+/// 攻方 1.1 谱系那一形（I-1.7）：回退到 A 之后，谱系容器头里的记录宽 16 → 15。
+#[test]
+fn a_lineage_container_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device(
+) {
+    let clean = image_after_rolling_back_to_the_first_root(
+        "known-bad-lineage-record-width-beside-a-copy-verifying-as-a-node",
+    )
+    .image;
+    let mut with_the_violation = clean.clone();
+    common_deletion_in_progress::set_the_record_width_of_the_first_container_of_the_tree_of_kind(
+        &mut with_the_violation,
+        TREE_KIND_LINEAGE,
+        record_width_one_below(singlefs_format::LINEAGE_RECORD_BYTES),
+    );
+    assert_the_violation_in_the_copy_of_the_entry_type_reddens_whichever_device_holds_the_copy_verifying_as_a_node(
+        &clean,
+        &with_the_violation,
+        TREE_KIND_LINEAGE,
+        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        "I-1.7",
+    );
+}
+
+/// 第 28 批代码轮正推腿 X2 / X11（判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「K1 测试」那一行、第三节第 3 条；
+/// 正推 `research/prompts/m3-batch28-code-r1-sonnet-output.md` 第 3.4 节第 1 条）：inode 那一形只把身份引用出生树加 1，子指针出生树比它小，
+/// 读不出孩子时那一小句的 `==` 改成 `>=` 照样红、钉不住。这一形把子指针头部出生树加 1（比身份引用出生树大）：I-9.2 照判、红。
+#[test]
+fn an_inode_tree_entry_whose_child_pointer_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back_reddens_the_inode_entry_invariant(
+) {
+    assert_a_birth_tree_field_one_off_reddens_the_entry_invariant_over_a_container_that_does_not_read_back(
+        &build_pool("known-bad-inode-entry-pointer-birth-tree-unreadable-leaf").memory_pool(),
+        TREE_KIND_INODE,
+        PACKED_RECORD_TYPE_INODE,
+        INTERNAL_ENTRY_CHILD_POINTER_BIRTH_TREE,
+        "I-9.2",
+    );
+}
+
+/// 正推腿 X13 / X14（同上判决那一行、第三节第 3 条；正推第 3.4 节第 2 条）的判法：新池新建文件那一版，inode 树根第一条条目不改（类型段 2），
+/// 叶容器盘序大那一份换成 inode 树根的码 2 字节、按改过的位置条目自证得过（[`image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device`]），
+/// 再照 `break_the_first_copy` 把位置条目 0 那一份弄坏（自证不过）。认孩子的类时第一份那一处 `continue` 要往下看第二份：
+/// 认出码 2，I-9.2 红、违例说明点名 IndexNode（改成 `break` 的话认不出别类，照类型段 2 读、两份都读不出，两段出生树相等，I-9.2 报成立）。
+/// 两份里没有一份按码 3 自证得过：不走 F-W 那一支。
+fn assert_a_second_copy_verifying_as_a_node_behind_a_broken_first_copy_reddens_the_inode_entry_invariant(
+    tag: &str,
+    break_the_first_copy: &dyn Fn(&mut MemoryPool),
+) {
+    let clean = build_pool(tag).memory_pool();
+    let mut image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
+        &clean,
+        TREE_KIND_INODE,
+        1,
+        &leave_the_entry_as_it_is,
+    );
+    break_the_first_copy(&mut image);
+    assert_the_violation_names_the_child_class(
+        &image,
+        "I-9.2",
+        PACKED_RECORD_TYPE_INODE,
+        "IndexNode",
+    );
+}
+
+/// X13：位置条目 0 那一份头里的类标签 3 → 9（不在登记表里，也就按哪一类的长度都自证不过）。
+#[test]
+fn an_inode_leaf_whose_first_copy_carries_an_unregistered_class_tag_and_whose_second_copy_verifies_as_a_node_reddens_the_inode_entry_invariant(
+) {
+    assert_a_second_copy_verifying_as_a_node_behind_a_broken_first_copy_reddens_the_inode_entry_invariant(
+        "known-bad-inode-leaf-first-copy-unregistered-class-second-copy-node",
+        &|image| {
+            let first = child_pointer_of_an_entry_of_the_root_of_kind(image, TREE_KIND_INODE, 0)
+                .locations[0];
+            let offset = first.slot.0 * SLOT;
+            let mut header =
+                read(image, first.device.0, offset, common_pointer_head_corruption::node_bytes());
+            header[UNIT_HEADER_CLASS_TAG_OFFSET] = UNIT_CLASS_NOT_REGISTERED;
+            write(image, first.device.0, offset, &header);
+        },
+    );
+}
+
+/// X14：位置条目 0 那一份读不出（那条位置条目的槽写成盘尾之后的第一槽，读者按盘的契约拒读）。
+#[test]
+fn an_inode_leaf_whose_first_copy_does_not_read_back_and_whose_second_copy_verifies_as_a_node_reddens_the_inode_entry_invariant(
+) {
+    assert_a_second_copy_verifying_as_a_node_behind_a_broken_first_copy_reddens_the_inode_entry_invariant(
+        "known-bad-inode-leaf-first-copy-unreadable-second-copy-node",
+        &|image| {
+            let first_slot_past_the_device_end =
+                singlefs_core::address::SlotNumber(image.device_size_in_bytes / SLOT);
+            change_an_entry_of_the_root_of_kind(image, TREE_KIND_INODE, 0, |entry| {
+                let pointer_field = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
+                    ..CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
+                        + CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES;
+                let mut pointer = singlefs_core::pointer::NodePointer::read_from(
+                    &mut singlefs_core::bytes::ByteReader::at(entry, pointer_field.start),
+                );
+                pointer.locations[0].slot = first_slot_past_the_device_end;
+                entry[pointer_field].copy_from_slice(
+                    &common_pointer_head_corruption::node_pointer_bytes(&pointer),
+                );
+            });
+            let first = child_pointer_of_an_entry_of_the_root_of_kind(image, TREE_KIND_INODE, 0)
+                .locations[0];
+            assert!(
+                PoolReader::read(
+                    image,
+                    first.device,
+                    DeviceOffsetInBytes(first.slot.0 * SLOT),
+                    common_pointer_head_corruption::node_bytes()
+                )
+                .is_none(),
+                "位置条目 0 那一份的头读不出"
+            );
+        },
+    );
+}
+
 // ─── 第 35 批：由记录造出来的两类版本取哪条记录的新根段（FL）、树表指针全零（Q1、被抛弃的根）、I-5.5 认出来而没有数据单元（Q4） ───
 
 /// 恰好要 `data_units` 个数据单元的内容（最后一个单元装一半）：顺序写一次发布切成 `data_units` 条记录（一个数据单元一条）。
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -37,6 +37,9 @@
 //! 第 29 批：走读不往下读的孩子每一份副本照判 I-2.1——inode 树根条目类型段 7、子指针两条位置条目的校验和都坏（H-J3a）一条流钉 CPU 红 I-2.1、
 //! I-4.8、I-7.2、I-7.4，类型段 0、第一条位置条目的校验和坏（H-J3b）一条流钉 CPU 红 I-2.1，GPU 都逐位跟上；被崩溃恢复抛弃的根自己那张实例表链 257 片
 //! （O-1）一条钉 GPU 记判不了（实例表链过长）、回落 CPU 之后结论相同；走读内核与扫描内核的链片数上限相同一条（不用卡）。
+//! 第 30 批（删除线）：inode 叶容器两份第一条记录 blocks 加 1、盘 1 那一份再换成码 2 节点的字节（F-W：另一份按类型段那一类自证过，照类型段走它）
+//! 一条流钉 CPU 红 I-9.2 与 I-9.15、不红 I-3.1、I-3.11、I-5.1；发布的根与它的记录里实例表指针头部诞生代号、实例、尾段各加 1（F-IT）三条流
+//! 钉 CPU 只红 I-1.2；GPU 都逐位跟上。
 //! 第 35 批：由记录造出来的两类版本取带末条标志那条记录的新根段（三条记录的发布第一条写成上一版的根：下一次挂载那一版叠两条记录罩住同一个槽、
 //! 挂载之后由记录施加出来的那一版各一条），树表指针整条全零照 core 跟过去（根与它的记录、由记录施加出来的那一版的记录各一条），都整份比。
 //! 第 36 批：下一次挂载先施加的那一版照 I-4.1 那一份重放施加的第一次发布认（改法 R1）——三条记录的发布首条没落（断号）、
@@ -8291,7 +8294,22 @@
     checksum_of_the_second_location: ChecksumOfTheSecondLocation,
 ) -> RecordedFlow {
     let mut flow = record_new_pool_file_creation(tag, name);
-    let leaf_copies = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
+    hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf(
+        &mut flow,
+        name,
+        checksum_of_the_second_location,
+    );
+    flow
+}
+
+/// [`record_new_pool_with_the_second_copy_of_the_inode_leaf_holding_the_inode_root_bytes`] 在一条录好的新池新建文件流上做的那一步：
+/// 流里最后那片 inode 叶容器盘序大的那一份换成最后那份 inode 树根（码 2）的字节，位置条目照 `checksum_of_the_second_location`。
+fn hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf(
+    flow: &mut RecordedFlow,
+    name: &'static str,
+    checksum_of_the_second_location: ChecksumOfTheSecondLocation,
+) {
+    let leaf_copies = last_unit_copies(flow, name, &is_an_inode_leaf_container, false);
     let second_leaf_copy =
         if flow.writes[leaf_copies[0]].device.0 < flow.writes[leaf_copies[1]].device.0 {
             leaf_copies[1]
@@ -8299,13 +8317,13 @@
             leaf_copies[0]
         };
     let root_copies = last_unit_copies(
-        &flow,
+        flow,
         name,
         &is_the_inode_tree_root_over_leaf_containers,
         false,
     );
     assert_the_first_entry_names_the_unit_of(
-        &flow,
+        flow,
         root_copies[0],
         &leaf_copies,
         "inode 树根第一条条目指着流里最后写的那片叶容器",
@@ -8336,7 +8354,7 @@
     match checksum_of_the_second_location {
         ChecksumOfTheSecondLocation::MatchesTheNodeBytes => {
             let checksum_of_the_node_bytes = crc32c(&node_bytes);
-            corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
+            corrupt_units_and_reseal_up_to_the_root(flow, &root_copies, &|bytes| {
                 let field = second_location + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET;
                 bytes[field..field + 4].copy_from_slice(&checksum_of_the_node_bytes.to_le_bytes());
             });
@@ -8347,7 +8365,6 @@
         panic!("单元写是普通写");
     };
     leaf[..node_bytes.len()].copy_from_slice(&node_bytes);
-    flow
 }
 
 /// 第 28 批 H-V（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 2 条；攻方 `research/prompts/m3-batch26-code-r1-opus-output.md`
@@ -8404,6 +8421,186 @@
     );
 }
 
+// ───────────────── 第 30 批：H-V 子树（F-W）、实例表链上每一片判 I-1.2 的出生身份那一半（F-IT） ─────────────────
+
+/// 叶容器第一条记录（新池新建文件那一版只有它一条）的 blocks 加 1：size 3000 要 6，写成 7，I-9.15 红。
+fn add_one_to_the_blocks_of_the_first_record_of_an_inode_leaf(bytes: &mut [u8]) {
+    let blocks_offset = PACKED_UNIT_RECORDS_START + INODE_RECORD_BLOCKS_OFFSET;
+    let blocks = read_u64_at(bytes, blocks_offset);
+    bytes[blocks_offset..blocks_offset + 8].copy_from_slice(&(blocks + 1).to_le_bytes());
+}
+
+/// 第 30 批 F-W（判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「K2 H-V 两份都看之后判错」那一行、第三节第 1 条；
+/// harness 坏镜像 `an_inode_leaf_with_blocks_off_the_logical_length_beside_a_copy_verifying_as_a_node_reddens_the_blocks_invariant_on_either_device`
+/// 同一形，录成流）：新池新建文件的流上 inode 叶容器两份第一条记录的 blocks 加 1（重封、沿引用链补到根），
+/// 再照 [`hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf`] 把盘序大的那一份换成 inode 树根的码 2 字节、位置条目跟着换。
+fn record_new_pool_with_blocks_off_in_the_inode_leaf_and_its_second_copy_holding_the_inode_root_bytes(
+    tag: &str,
+) -> RecordedFlow {
+    let name = "new-pool-inode-leaf-blocks-off-second-copy-verifying-as-a-node";
+    let mut flow = record_new_pool_file_creation(tag, name);
+    let leaf_copies = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
+    corrupt_units_and_reseal_up_to_the_root(
+        &mut flow,
+        &leaf_copies,
+        &add_one_to_the_blocks_of_the_first_record_of_an_inode_leaf,
+    );
+    hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf(
+        &mut flow,
+        name,
+        ChecksumOfTheSecondLocation::MatchesTheNodeBytes,
+    );
+    flow
+}
+
+/// F-W 那一形：叶容器盘 0 那一份是码 3、带 blocks 的违例、自证得过，盘 1 那一份自证成码 2。CPU 认出码 2、I-9.2 红，另一份按类型段那一类自证过，
+/// 照类型段 2 那一支走盘 0 那一份、判出 I-9.15（改之前见别类就不往下走，I-9.15 报不适用）；槽数按类型段那一类登记，不连带 I-3.1、I-3.11、I-5.1。
+/// GPU 走读内核 `w_walk_inode_root` 同改（`w_some_copy_of_the_child_verifies_as_the_class_the_entry_type_names`），每个状态逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_leaf_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node(
+) {
+    let test_name = "the_gpu_judge_agrees_with_the_cpu_when_a_leaf_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node";
+    let flow = record_new_pool_with_blocks_off_in_the_inode_leaf_and_its_second_copy_holding_the_inode_root_bytes(
+        "crash-judge-gpu-i92-inode-leaf-blocks-off-second-copy-verifying-as-a-node",
+    );
+    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    for invariant in ["I-9.2", "I-9.15"] {
+        assert!(
+            comparison.cpu_checker_red_invariants.contains(invariant),
+            "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
+            flow.name,
+            comparison.cpu_checker_red_invariants
+        );
+    }
+    for invariant_a_wrong_slot_count_drags_in in ["I-3.1", "I-3.11", "I-5.1"] {
+        assert!(
+            !comparison
+                .cpu_checker_red_invariants
+                .contains(invariant_a_wrong_slot_count_drags_in),
+            "流 {} 上槽数按类型段那一类登记：CPU 池级 checker 不红 {invariant_a_wrong_slot_count_drags_in}（红过的：{:?}）",
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
+/// 码 2 / 码 3 指针头里出生身份的三段（D19（块指针的结构与宽度预算） 已定项 11 的偏移表）：诞生代号 8 字节在 42、写序实例 4 字节在 78、
+/// 尾段（码 2 / 码 3 是出生序号）4 字节在 82。
+#[derive(Clone, Copy, Debug)]
+enum BirthIdentityFieldOfANodePointer {
+    BirthTxg,
+    Instance,
+    Tail,
+}
+
+impl BirthIdentityFieldOfANodePointer {
+    fn offset_in_the_pointer(self) -> usize {
+        match self {
+            BirthIdentityFieldOfANodePointer::BirthTxg => 42,
+            BirthIdentityFieldOfANodePointer::Instance => 78,
+            BirthIdentityFieldOfANodePointer::Tail => 82,
+        }
+    }
+}
+
+/// 攻方 F-IT 那三形（`research/prompts/m3-batch28-code-r1-opus-output.md` 第 4.2 节）：新池新建文件的流上，发布的根记录里实例表指针头部出生身份的
+/// `field` 那一段最低字节加 1、重封根记录，发布那几条 journal 记录新根段里同一条指针的同一段跟着改、重封（新根段与同代的根逐字节相同，I-8.10 不红）。
+/// 实例表那一片头里的诞生代号、写序实例、出生序号都是 0：指针记的不再与它相同。
+fn record_new_pool_with_the_instance_table_pointer_birth_identity_off_by_one_in(
+    tag: &str,
+    name: &'static str,
+    field: BirthIdentityFieldOfANodePointer,
+) -> RecordedFlow {
+    let mut flow = record_new_pool_file_creation(tag, name);
+    let field_offset = field.offset_in_the_pointer();
+    let publish_root = flow.judged_root_index;
+    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
+        let offset = ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + field_offset;
+        bytes[offset] = bytes[offset].wrapping_add(1);
+    });
+    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
+        let offset = JOURNAL_NEW_ROOT_SEGMENT_OFFSET
+            + NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER
+            + field_offset;
+        bytes[offset] = bytes[offset].wrapping_add(1);
+    });
+    flow
+}
+
+/// F-IT 的判法（判决 `research/prompts/m3-batch28-code-r1-main-verification.md` 第二节「实例表链上的 I-1.2」那一行、第三节第 2 条）：CPU 池级 checker
+/// 走实例表链每一片都判指针与头的出生身份（`walk.rs` `walk_instance_table_chain`），这条流上只红 I-1.2；GPU 走读内核
+/// `w_walk_instance_table_chain` 每一片同判（改之前链循环不调 `w_judge_birth_identity`，I-1.2 报成立），每个状态逐位相同。
+fn assert_the_gpu_judge_agrees_with_the_cpu_on_an_instance_table_pointer_whose_birth_identity_is_off_by_one_in(
+    test_name: &str,
+    tag: &str,
+    name: &'static str,
+    field: BirthIdentityFieldOfANodePointer,
+) {
+    let flow = record_new_pool_with_the_instance_table_pointer_birth_identity_off_by_one_in(
+        tag, name, field,
+    );
+    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert_eq!(
+        comparison.cpu_checker_red_invariants,
+        std::collections::BTreeSet::from(["I-1.2"]),
+        "流 {} 上 CPU 池级 checker 只红 I-1.2（实例表指针头部 {field:?} 那一段与那一片头里的不符）",
+        flow.name
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
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_txg_than_its_page(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_an_instance_table_pointer_whose_birth_identity_is_off_by_one_in(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_txg_than_its_page",
+        "crash-judge-gpu-i12-instance-table-pointer-birth-txg",
+        "new-pool-instance-table-pointer-birth-txg-plus-one",
+        BirthIdentityFieldOfANodePointer::BirthTxg,
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_instance_than_its_page(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_an_instance_table_pointer_whose_birth_identity_is_off_by_one_in(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_instance_than_its_page",
+        "crash-judge-gpu-i12-instance-table-pointer-instance",
+        "new-pool-instance-table-pointer-instance-plus-one",
+        BirthIdentityFieldOfANodePointer::Instance,
+    );
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_sequence_than_its_page(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_an_instance_table_pointer_whose_birth_identity_is_off_by_one_in(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_sequence_than_its_page",
+        "crash-judge-gpu-i12-instance-table-pointer-birth-sequence",
+        "new-pool-instance-table-pointer-birth-sequence-plus-one",
+        BirthIdentityFieldOfANodePointer::Tail,
+    );
+}
+
 /// GPU 关门那一侧（判决 `research/prompts/m3-batch25-code-r1-main-verification.md` 第三节第 2 条）：最新根的实例表照 `change` 改成主走读走不到末片的样子，
 /// 门要关。CPU 一侧先钉流造对了——改之前整条落盘的镜像上 I-9.29 真被评估过且成立；改之后整条落盘的镜像上 I-9.29 报部分没判。
 /// 流上 C 还是最新根的那几个状态 CPU 红 I-9.29；GPU 每个状态逐位相同（实例 3 的根落了、C 写回了的状态上，GPU 门要是开了，
````

## 实现员 m：mutations-append.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch30/m/patch/mutations-append.tsv）

````text
第30批 F-W：另一份按类型段那一类自证过也不往下走（三棵树都改回见别类就停；inode 叶容器带 blocks 违例、另一份自证成码 2，I-9.15 两种盘序都报不适用）	crates/singlefs-checker/src/walk.rs	    child.locations.iter().any(|location| {\n        reader\n            .read(location.device, location.slot * SLOT_BYTES, unit_bytes)	    child.locations.iter().take(0).any(|location| {\n        reader\n            .read(location.device, location.slot * SLOT_BYTES, unit_bytes)	-p singlefs-harness --test checker_known_bad_images -- beside_a_copy_verifying_as_a_node	an_inode_leaf_with_blocks_off_the_logical_length_beside_a_copy_verifying_as_a_node_reddens_the_blocks_invariant_on_either_device
第30批 F-W：inode 树那一处另一份按类型段那一类自证过也不往下走（inode 叶容器预留段非 0、另一份自证成码 2，I-9.7 两种盘序都报不适用）	crates/singlefs-checker/src/walk.rs	                        if a_copy_of_the_class_the_entry_type_names_verifies {	                        if false && a_copy_of_the_class_the_entry_type_names_verifies {	-p singlefs-harness --test checker_known_bad_images -- beside_a_copy_verifying_as_a_node	an_inode_leaf_with_a_non_zero_reserved_byte_beside_a_copy_verifying_as_a_node_reddens_the_reserved_bytes_invariant_on_either_device
第30批 F-W：inode 树那一处另一份按类型段那一类自证过也不往下走（inode 叶容器记录宽 139、另一份自证成码 2，I-1.7 两种盘序都报成立）	crates/singlefs-checker/src/walk.rs	                        if a_copy_of_the_class_the_entry_type_names_verifies {	                        if false && a_copy_of_the_class_the_entry_type_names_verifies {	-p singlefs-harness --test checker_known_bad_images -- beside_a_copy_verifying_as_a_node	an_inode_leaf_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device
第30批 F-W：墓碑树、谱系树那一处另一份按类型段那一类自证过也不往下走（墓碑容器记录宽 39、另一份自证成码 2，I-1.7 两种盘序都报成立）	crates/singlefs-checker/src/walk.rs	                    if some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(	                    if false && some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(	-p singlefs-harness --test checker_known_bad_images -- beside_a_copy_verifying_as_a_node	a_tombstone_container_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device
第30批 F-W：墓碑树、谱系树那一处另一份按类型段那一类自证过也不往下走（谱系容器记录宽 15、另一份自证成码 2，I-1.7 两种盘序都报成立）	crates/singlefs-checker/src/walk.rs	                    if some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(	                    if false && some_copy_of_the_child_verifies_as_the_class_the_entry_type_names(	-p singlefs-harness --test checker_known_bad_images -- beside_a_copy_verifying_as_a_node	a_lineage_container_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device
第30批 X2：inode 树读不出孩子时类型段 2 子指针出生树按 >= 比（子指针头部出生树加 1、叶容器两份都读不出，I-9.2 报成立）	crates/singlefs-checker/src/walk.rs	                birth_tree_in_the_child_pointer_head == identity_birth_tree,	                birth_tree_in_the_child_pointer_head >= identity_birth_tree,	-p singlefs-harness --test checker_known_bad_images -- child_pointer_birth_tree_is_one_off_over_a_leaf whose_second_copy_verifies_as_a_node	an_inode_tree_entry_whose_child_pointer_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back_reddens_the_inode_entry_invariant
第30批 X13：认孩子的类时第一份头里的类标签不在登记表里就不看第二份（第二份自证成码 2，I-9.2 报成立）	crates/singlefs-checker/src/walk.rs	        let Some(unit_bytes) = unit_bytes_of_the_class(class_in_the_header) else {\n            continue;\n        };	        let Some(unit_bytes) = unit_bytes_of_the_class(class_in_the_header) else {\n            break;\n        };	-p singlefs-harness --test checker_known_bad_images -- child_pointer_birth_tree_is_one_off_over_a_leaf whose_second_copy_verifies_as_a_node	an_inode_leaf_whose_first_copy_carries_an_unregistered_class_tag_and_whose_second_copy_verifies_as_a_node_reddens_the_inode_entry_invariant
第30批 X14：认孩子的类时第一份头读不出就不看第二份（第二份自证成码 2，I-9.2 报成立）	crates/singlefs-checker/src/walk.rs	        let Some(header) = reader.read(location.device, offset, node_bytes()) else {\n            continue;\n        };	        let Some(header) = reader.read(location.device, offset, node_bytes()) else {\n            break;\n        };	-p singlefs-harness --test checker_known_bad_images -- child_pointer_birth_tree_is_one_off_over_a_leaf whose_second_copy_verifies_as_a_node	an_inode_leaf_whose_first_copy_does_not_read_back_and_whose_second_copy_verifies_as_a_node_reddens_the_inode_entry_invariant
第30批 GPU 判器 F-W：走读内核 inode 树另一份按类型段那一类自证过也不往下走（叶容器带 blocks 违例、盘 1 那一份自证成码 2 那条流上 CPU 红 I-9.15，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            w_judge(INVARIANT_I_9_2, class_of_the_inode_child == NONE);\n            class_of_the_inode_child = NONE;	            w_judge(INVARIANT_I_9_2, class_of_the_inode_child == NONE);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_leaf_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node	the_gpu_judge_agrees_with_the_cpu_when_a_leaf_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node
第30批 GPU 判器 F-IT：走读内核实例表链上每一片不判 I-1.2（实例表指针头部诞生代号加 1 那条流上 CPU 红 I-1.2，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        w_judge(INVARIANT_I_1_2, eq64(rd64(s.base, 73u), pointer.birth_txg) && rd32(s.base, 93u) == pointer.instance\n            && eq64(vec2<u32>(rd32(s.base, 103u), 0u), pointer.tail));	        // 变异：实例表链上每一片不判 I-1.2	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_txg_than_its_page	the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_names_another_birth_txg_than_its_page
````

## 实现员 m：mutations-replacements.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch30/m/patch/mutations-replacements.tsv）

````text
第26批 inode 树读不出孩子时 I-9.2 不判（类型段不认得的条目只记 I-2.1 与走读失败）	crates/singlefs-checker/src/walk.rs	                    self.judgements.judge("I-9.2", false, || {\n                        format!(\n                            "inode 内部条目的类型段 {unregistered_type} 不在 {{0, 2}} 里（孩子没有一份按它头里那一类的长度自证过，不往下走）"\n                        )\n                    });	                    let _ = read_referenced_unit(\n                        self.reader,\n                        &mut self.judgements,\n                        &child.locations,\n                        node_bytes(),\n                        "inode 树的孩子",\n                    );\n                    self.walk_failures\n                        .push(format!("变异：类型段 {unregistered_type} 的孩子按 16 KiB 读"));	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_only_the_inode_entry_invariant
````

## 实现员 fix：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch30/fix/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -8551,10 +8551,10 @@
     let flow = record_new_pool_with_blocks_off_in_the_inode_leaf_and_its_second_copy_holding_the_inode_root_bytes(
         "crash-judge-gpu-i92-inode-leaf-blocks-off-second-copy-verifying-as-a-node",
     );
-    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     for invariant in ["I-9.2", "I-9.15"] {
         assert!(
             comparison.cpu_checker_red_invariants.contains(invariant),
@@ -8637,10 +8637,10 @@
     let flow = record_new_pool_with_the_instance_table_pointer_birth_identity_off_by_one_in(
         tag, name, field,
     );
-    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
         return;
     };
-    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    let comparison = compare_on_every_state(cards, &flow, expand_up_to());
     assert_eq!(
         comparison.cpu_checker_red_invariants,
         std::collections::BTreeSet::from(["I-1.2"]),
````

