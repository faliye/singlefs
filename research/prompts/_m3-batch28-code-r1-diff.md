# 删除批第 28 批合入的改动原样（2026-10-06）

## 实现员 k：crates.patch（/tmp/claude-1000/m3-batch28/k/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 1a3f968..b4038db 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -2356,7 +2356,8 @@ impl<'reader> Walk<'reader> {
     }
 
     /// 墓碑树、谱系树里类型段 = `record_type` 那条条目下面的码 3 容器：先登记引用、读容器，按 `identity_invariant` 判条目身份
-    /// （容器头四段与身份引用逐字相等、子指针头部出生树相等、类标签 3 与打包记录类型重合 I-1.6 / I-1.7 的那几小句留作合取）；
+    /// （容器头四段与身份引用逐字相等、子指针头部出生树相等、类标签 3 与打包记录类型重合 I-1.6 / I-1.7 的那几小句留作合取；
+    /// 容器两份都读不出时只判子指针头部出生树那一小句）；
     /// 这一遍第一次走到时再判树 ID（I-1.3）、出生身份（I-1.2 / I-4.2）与容器格式（I-1.7）。
     fn walk_the_container_below_a_container_tree_entry(
         &mut self,
@@ -2376,6 +2377,19 @@ impl<'reader> Walk<'reader> {
             data_unit_bytes(),
             &what,
         ) else {
+            // 容器两份都读不出：「子指针头部出生树 == 身份引用出生树」只比父条目里的两段，照判（判决
+            // `research/prompts/m3-batch26-code-r1-main-verification.md` 第二节 K2 H3 H-B 那一行、第三节第 1 条）；比容器头的那几小句不判。
+            let identity_birth_tree = identity.0;
+            self.judgements.judge(
+                identity_invariant,
+                child.birth_tree == identity_birth_tree,
+                || {
+                    format!(
+                        "{name}内部条目类型段 {record_type}：容器两份都读不出（比容器头的那几小句不判），子指针头部的出生树 {} 不等于身份引用的出生树 {identity_birth_tree}",
+                        child.birth_tree
+                    )
+                },
+            );
             self.walk_failures
                 .push(format!("{name}的容器两份都读不到对得上的"));
             return;
@@ -2577,6 +2591,7 @@ impl<'reader> Walk<'reader> {
                     &mut self.judgements,
                     registered_entry_type,
                     identity,
+                    child.birth_tree,
                 );
                 self.walk_failures
                     .push("inode 树的孩子两份都读不到对得上的".to_string());
@@ -4133,11 +4148,14 @@ fn slots_spanned_by_a_unit_of_the_class(class: MappingKeyUnitClass) -> Option<u6
 /// 树里一条内部条目的孩子是不是条目类型段说的那一类：只读孩子的单元头取类标签，不按类型段定读多长（判决
 /// `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N4 那一行：按类型段定长度时，类型段与孩子真实的类不符，
 /// 按那个长度读回的整单元校验和对不上，身份那一判跑不到、却记了 I-2.1 与走读失败）。
-/// 逐条位置条目：头里的类标签是登记过的码、且按那一类的长度读回的整单元校验和对得上，这一份才算自证过的孩子——
-/// 自证过的孩子的类就是 `class_the_entry_type_names` 的，交回 None（照类型段那一支走）；是另一类的，交回那一类。
-/// 类标签不在登记表里、读不出、校验和对不上的看下一条位置条目（陈旧副本、坏副本），哪一份都自证不过交回 None（照类型段那一支走，读不出照旧归 I-2.1）。
-/// 头里的类就是类型段那一类时同样要自证（判决 `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K2 N1 两故障 H1 那一行：
-/// 只认第一份的头、不验，第一份是头里恰好是那一类的陈旧副本时第二份不看，结论随陈旧副本在哪块盘翻面）。
+/// 两条位置条目都看：头里的类标签是登记过的码、且按那一类的长度读回的整单元校验和对得上，这一份才算自证过的孩子；
+/// 有一份自证过的孩子是 `class_the_entry_type_names` 之外的一类，交回那一类（两份各自证成两个另一类时交回位置条目在前的那一份的）；
+/// 自证过的都是类型段那一类、或哪一份都自证不过（类标签不在登记表里、读不出、校验和对不上：陈旧副本、坏副本），交回 None
+/// （照类型段那一支走，读不出照旧归 I-2.1）。
+/// 不在第一份自证过的副本处停（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第二节 K2 H1 H-V 那一行、第三节第 2 条：
+/// 两份按各自的位置条目都自证得过、却是两类时，只看第一份的话结论随那一份在哪块盘翻面）。
+/// 头里的类就是类型段那一类的副本不影响结论（判决 `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K2 N1 两故障 H1 那一行：
+/// 它是陈旧副本时，第二份照样要看）。
 /// 类型段不认得的条目（`class_the_entry_type_names` 是 None）没有「它说的那一类」，交回孩子自证过的那一类。
 /// 只认按它自己的长度自证过的孩子：头里的类标签坏了的单元按那个长度对不上校验和，不会被当成「另一类」。只读、不判。
 fn child_unit_class_other_than_the_entry_type_names(
@@ -4145,6 +4163,7 @@ fn child_unit_class_other_than_the_entry_type_names(
     child: &PointerView,
     class_the_entry_type_names: Option<MappingKeyUnitClass>,
 ) -> Option<MappingKeyUnitClass> {
+    let mut class_other_than_the_entry_type_names = None;
     for location in &child.locations {
         let offset = location.slot * SLOT_BYTES;
         // 头按最短的单元（码 2，16 KiB）那么长读：读者只认整块的读，32 KiB 的单元头也在这前 16 KiB 里。
@@ -4161,27 +4180,42 @@ fn child_unit_class_other_than_the_entry_type_names(
         if !verifies_at_the_length_of_its_class {
             continue;
         }
-        if Some(class_in_the_header) == class_the_entry_type_names {
-            return None;
+        if Some(class_in_the_header) != class_the_entry_type_names
+            && class_other_than_the_entry_type_names.is_none()
+        {
+            class_other_than_the_entry_type_names = Some(class_in_the_header);
         }
-        return Some(class_in_the_header);
     }
-    None
+    class_other_than_the_entry_type_names
 }
 
 /// inode 树一条内部条目的孩子两份都读不出（[`child_unit_class_other_than_the_entry_type_names`] 也没认出另一类）时，I-9.2 里不看孩子就判得了的那几小句，
 /// 与墓碑树、谱系树读不出孩子时同判（`Walk::walk_the_entries_of_a_container_tree_node`；判决
 /// `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K1 N1 那一行「inode 树读不出孩子时 I-9.2 不判」）：
-/// 类型段 0 判身份引用三段为 0（「子单元是码 2」那一小句孩子读不出不判）；类型段 2 那几小句都要比孩子的头，读不出不判。
+/// 类型段 0 判身份引用三段为 0（「子单元是码 2」那一小句孩子读不出不判）；类型段 2 判「子指针头部的出生树 == 身份引用的出生树」
+/// （只比父条目里的两段，判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第二节 K2 H3 H-B 那一行、第三节第 1 条），
+/// 子头四段与身份引用相等、子容器的打包记录类型 == 2 那几小句要比孩子的头，读不出不判。
 /// 类型段不认得的条目不走到这里：调用方不读它的孩子、先判该条目损坏（第 27 批，与墓碑树、谱系树类型段不认得那一支同判）。
 /// 读不出照旧归 I-2.1 与走读失败（调用方记）。
 fn judge_an_inode_entry_whose_child_does_not_read_back(
     judgements: &mut Judgements,
     entry_type: RegisteredInodeInternalEntryType,
     identity: (u64, u16, u64, u64),
+    birth_tree_in_the_child_pointer_head: u64,
 ) {
     match entry_type {
-        RegisteredInodeInternalEntryType::LeafContainer => {}
+        RegisteredInodeInternalEntryType::LeafContainer => {
+            let identity_birth_tree = identity.0;
+            judgements.judge(
+                "I-9.2",
+                birth_tree_in_the_child_pointer_head == identity_birth_tree,
+                || {
+                    format!(
+                        "inode 内部条目类型段 2：孩子两份都读不出（比孩子头的那几小句不判），子指针头部的出生树 {birth_tree_in_the_child_pointer_head} 不等于身份引用的出生树 {identity_birth_tree}"
+                    )
+                },
+            );
+        }
         RegisteredInodeInternalEntryType::InternalNode => {
             let (birth_tree, _, container, container_birth) = identity;
             let identity_is_zero = birth_tree == 0 && container == 0 && container_birth == 0;
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index 4338e8a..2f48abc 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -2656,10 +2656,14 @@ fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: v
         let s = w_read_referenced_unit(child, length);
         if (s.version == NONE) {
             // 孩子两份都读不出：I-9.2 里不看孩子就判得了的那几小句照判（walk.rs judge_an_inode_entry_whose_child_does_not_read_back，
-            // 与墓碑树、谱系树读不出孩子时同判）——类型段 0 判身份引用三段为 0，类型段 2 不判（类型段不认得的上面已经判完）
+            // 与墓碑树、谱系树读不出孩子时同判）——类型段 0 判身份引用三段为 0，类型段 2 判子指针头部的出生树 == 身份引用的出生树
+            // （判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 1 条），比孩子头的那几小句不判（类型段不认得的上面已经判完）
             if (record_type == 0u) {
                 w_judge(INVARIANT_I_9_2, is_zero64(identity_birth_tree) && is_zero64(identity_container) && is_zero64(identity_birth));
             }
+            if (record_type == 2u) {
+                w_judge(INVARIANT_I_9_2, eq64(child.birth_tree, identity_birth_tree));
+            }
             w_failures = w_failures + 1u;
             continue;
         }
@@ -2973,33 +2977,33 @@ fn w_unit_verifies_at_the_length_of_its_class(s: Slice, unit_class: u32, checksu
 }
 
 // 树里一条内部条目的孩子是不是条目类型段说的那一类（walk.rs 的 child_unit_class_other_than_the_entry_type_names）：只读孩子的单元头取类标签，
-// 不按类型段定读多长。两条位置条目依次看：头按最短的单元（码 2，16 KiB）那么长读，头里的类标签是登记过的码、且按那一类的长度读回的
-// 整单元校验和对得上，这一份才算自证过的孩子——它的类就是 class_the_entry_type_names 的交回 NONE（照类型段那一支走），是另一类的交回那一类；
-// 读不出、类标签不在登记表里、校验和对不上的看下一条位置条目（陈旧副本、坏副本），哪一份都自证不过交回 NONE（读不出照旧归 I-2.1）。
-// 头里的类就是类型段那一类时同样要自证（判决 `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K2 N1 两故障 H1 那一行，walk.rs 同改）。
+// 不按类型段定读多长。两条位置条目都看：头按最短的单元（码 2，16 KiB）那么长读，头里的类标签是登记过的码、且按那一类的长度读回的
+// 整单元校验和对得上，这一份才算自证过的孩子；有一份自证过的是 class_the_entry_type_names 之外的一类，交回那一类（两份各自证成两个另一类时
+// 交回第一份的）；自证过的都是类型段那一类、或哪一份都自证不过（读不出、类标签不在登记表里、校验和对不上：陈旧副本、坏副本），交回 NONE
+// （照类型段那一支走，读不出照旧归 I-2.1）。不在第一份自证过的副本处停（判决 `research/prompts/m3-batch26-code-r1-main-verification.md`
+// 第二节 K2 H1 H-V 那一行、第三节第 2 条：两份都自证得过、却是两类时，只看第一份的话结论随那一份在哪块盘翻面，walk.rs 同改）。
 // class_the_entry_type_names 是 NONE（类型段不认得）时交回孩子自证过的那一类
 fn w_child_unit_class_other_than_the_entry_type_names(child: Ptr, class_the_entry_type_names: u32) -> u32 {
+    var class_other_than_the_entry_type_names = NONE;
     let first_copy = read_at(child.device0, slot_to_bytes(child.slot0), NODE_BYTES);
     if (first_copy.version != NONE) {
         let class_in_the_first_header = rd8(first_copy.base, 6u);
         if (w_unit_verifies_at_the_length_of_its_class(first_copy, class_in_the_first_header, child.sum0)) {
-            if (class_in_the_first_header == class_the_entry_type_names) {
-                return NONE;
+            if (class_in_the_first_header != class_the_entry_type_names) {
+                class_other_than_the_entry_type_names = class_in_the_first_header;
             }
-            return class_in_the_first_header;
         }
     }
     let second_copy = read_at(child.device1, slot_to_bytes(child.slot1), NODE_BYTES);
     if (second_copy.version != NONE) {
         let class_in_the_second_header = rd8(second_copy.base, 6u);
         if (w_unit_verifies_at_the_length_of_its_class(second_copy, class_in_the_second_header, child.sum1)) {
-            if (class_in_the_second_header == class_the_entry_type_names) {
-                return NONE;
+            if (class_in_the_second_header != class_the_entry_type_names && class_other_than_the_entry_type_names == NONE) {
+                class_other_than_the_entry_type_names = class_in_the_second_header;
             }
-            return class_in_the_second_header;
         }
     }
-    return NONE;
+    return class_other_than_the_entry_type_names;
 }
 
 // 一个单元按它的类占几槽：码 2 节点一槽，码 1 数据单元与码 3 打包记录单元两槽
@@ -3105,6 +3109,9 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
         w_note_pointer_references(child, 2u);
         let s = w_read_referenced_unit(child, DATA_UNIT_BYTES);
         if (s.version == NONE) {
+            // 容器两份都读不出：子指针头部出生树 == 身份引用出生树只比父条目里的两段，照判（walk.rs walk_the_container_below_a_container_tree_entry，
+            // 判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 1 条）；比容器头的那几小句不判
+            w_judge(identity_invariant, eq64(child.birth_tree, rd64(n.base, o + 8u)));
             w_failures = w_failures + 1u;
             continue;
         }
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index bdbabab..390557b 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -28,6 +28,9 @@
 //! 第 27 批：inode 叶容器类标签改成不登记的码、不重封（孩子读不出），树根条目类型段 0 带身份、类型段 7 各一条流整份比；
 //! 同一段被崩溃恢复抛弃的历史上，最新根的实例表链指针记录不合法、第 1 片读不出（GPU 关门那一侧）各一条流整份比；
 //! 链 256 片一条整份比，链 257 片一条钉 GPU 池级 checker 记判不了（实例表链过长）、回落 CPU 之后结论相同（攻方 H-c）。
+//! 第 28 批：孩子两份都读不出时照判「子指针头部的出生树 == 身份引用的出生树」（H-B）——inode 树根条目身份出生树加 1、墓碑树根条目子指针出生树加 1
+//! 各一条流钉 CPU 红 I-9.2 / I-9.23；inode 叶容器盘 1 那一份换成码 2 节点的字节、位置条目改成对得上它（H-V：两份都看）一条流钉 CPU 红 I-9.2；
+//! 同样换字节而位置条目不改（第二份副本自证不过）一条流钉 CPU 不红 I-9.2；GPU 都逐位跟上。
 //! 第 35 批：由记录造出来的两类版本取带末条标志那条记录的新根段（三条记录的发布第一条写成上一版的根：下一次挂载那一版叠两条记录罩住同一个槽、
 //! 挂载之后由记录施加出来的那一版各一条），树表指针整条全零照 core 跟过去（根与它的记录、由记录施加出来的那一版的记录各一条），都整份比。
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
@@ -7387,6 +7390,262 @@ fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered
     );
 }
 
+// ───────────────────────── 第 28 批：读不出孩子时照判出生树那一小句（H-B）、孩子两份都看（H-V）、第二份副本的自证 ─────────────────────────
+
+/// 墓碑树、谱系树码 2 节点条目里子指针的偏移：与 inode 树内部条目同形（分隔 key 8 + 身份引用 26 之后）。
+const CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET: usize = INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET;
+/// 位置条目里整单元校验和的偏移（盘 4、槽 6 之后）。
+const LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET: usize = 10;
+
+/// 一个单元的这几份副本各翻一个载荷字节、不重封、不补引用链：指着它的位置条目还是翻之前的整单元 CRC，这几份都读不出。
+fn flip_a_payload_byte_of_each_copy_without_resealing(flow: &mut RecordedFlow, copies: &[usize]) {
+    for copy in copies {
+        let WrittenContents::Bytes(bytes) = &mut flow.writes[*copy].contents else {
+            panic!("单元写是普通写");
+        };
+        let flipped_byte = bytes.len() - BYTES_FROM_THE_END_OF_THE_FLIPPED_PAYLOAD_BYTE;
+        bytes[flipped_byte] ^= 0xFF;
+    }
+}
+
+/// 节点第一条条目的子指针第 0 条位置条目指的槽就是 `copies` 那个单元的槽（钉住改的条目指着要弄坏的那个孩子）。
+fn assert_the_first_entry_names_the_unit_of(
+    flow: &RecordedFlow,
+    node_copy: usize,
+    copies: &[usize],
+    what: &str,
+) {
+    let node = flow.writes[node_copy].bytes().expect("单元写带字节");
+    assert_eq!(
+        device_and_slot_of_a_pointer_location(
+            &node[first_entry_of_a_container_number_node()
+                + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET..],
+            POINTER_LOCATION_ENTRY_OFFSETS[0],
+        )
+        .1,
+        flow.writes[copies[0]].offset.0 / SLOT_BYTES,
+        "{what}"
+    );
+}
+
+/// 节点第一条条目里 `field`（条目内偏移）起那 8 字节（出生树那两段之一）加 1。
+fn add_one_to_a_birth_tree_of_the_first_entry(bytes: &mut [u8], field: usize) {
+    let offset = first_entry_of_a_container_number_node() + field;
+    let birth_tree = read_u64_at(bytes, offset);
+    bytes[offset..offset + 8].copy_from_slice(&(birth_tree + 1).to_le_bytes());
+}
+
+/// H-B inode 那一形（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 1 条；harness 坏镜像
+/// `an_inode_tree_entry_whose_identity_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back_reddens_the_inode_entry_invariant` 同一形，录成流）：
+/// 新池新建文件的流上 inode 树根（层级 1）第一条条目身份引用的出生树加 1、重封到根，它指的叶容器两份各翻一个载荷字节、不重封（两份都读不出）。
+fn record_new_pool_with_the_inode_root_entry_identity_birth_tree_one_off_over_a_leaf_that_does_not_read_back(
+    tag: &str,
+) -> RecordedFlow {
+    let name = "new-pool-inode-root-entry-identity-birth-tree-one-off-unreadable-leaf";
+    let mut flow = record_new_pool_file_creation(tag, name);
+    let leaf_copies = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
+    let root_copies = last_unit_copies(
+        &flow,
+        name,
+        &is_the_inode_tree_root_over_leaf_containers,
+        false,
+    );
+    assert_the_first_entry_names_the_unit_of(
+        &flow,
+        root_copies[0],
+        &leaf_copies,
+        "inode 树根第一条条目指着流里最后写的那片叶容器",
+    );
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
+        add_one_to_a_birth_tree_of_the_first_entry(bytes, CONTAINER_TREE_ENTRY_BIRTH_TREE_OFFSET);
+    });
+    flip_a_payload_byte_of_each_copy_without_resealing(&mut flow, &leaf_copies);
+    flow
+}
+
+/// 第 28 批 H-B：CPU 池级 checker 读不出孩子时照判「子指针头部的出生树 == 身份引用的出生树」、红 I-9.2（改之前类型段 2 那一支整支不判），
+/// 叶容器读不出、I-2.1 红；GPU 走读内核读不出那一支同判（`w_walk_inode_root`），每个状态逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_identity_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_identity_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back",
+        &record_new_pool_with_the_inode_root_entry_identity_birth_tree_one_off_over_a_leaf_that_does_not_read_back(
+            "crash-judge-gpu-i92-inode-root-entry-identity-birth-tree-unreadable-leaf",
+        ),
+        &["I-9.2", "I-2.1"],
+    );
+}
+
+/// H-B 墓碑那一形（harness 坏镜像
+/// `a_tombstone_tree_entry_whose_child_pointer_birth_tree_is_one_off_over_a_container_that_does_not_read_back_reddens_the_tombstone_entry_invariant`
+/// 同一形，录成流）：删除中间态的流上墓碑树根（层级 1）第一条条目子指针头部的出生树加 1、重封到根，它指的墓碑容器两份各翻一个载荷字节、
+/// 不重封（两份都读不出）。
+fn record_deletion_with_the_tombstone_root_entry_pointer_birth_tree_one_off_over_a_container_that_does_not_read_back(
+    tag: &str,
+) -> RecordedFlow {
+    let name = "deletion-tombstone-root-entry-pointer-birth-tree-one-off-unreadable-container";
+    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
+    let container_copies = last_unit_copies(&flow, name, &is_a_tombstone_container, false);
+    let root_copies = last_unit_copies(&flow, name, &is_the_tombstone_tree_root, false);
+    assert_the_first_entry_names_the_unit_of(
+        &flow,
+        root_copies[0],
+        &container_copies,
+        "墓碑树根第一条条目指着流里最后写的那片墓碑容器",
+    );
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
+        add_one_to_a_birth_tree_of_the_first_entry(
+            bytes,
+            CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + POINTER_BIRTH_TREE_OFFSET,
+        );
+    });
+    flip_a_payload_byte_of_each_copy_without_resealing(&mut flow, &container_copies);
+    flow
+}
+
+/// 第 28 批 H-B：CPU 池级 checker 容器读不出时照判「子指针头部出生树 == 身份引用出生树」、红 I-9.23（改之前只记走读失败），
+/// 容器读不出、I-2.1 红；GPU 走读内核容器树读不出那一支同判（`w_walk_container_tree_root`），每个状态逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_pointer_birth_tree_is_one_off_over_a_container_that_does_not_read_back(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_pointer_birth_tree_is_one_off_over_a_container_that_does_not_read_back",
+        &record_deletion_with_the_tombstone_root_entry_pointer_birth_tree_one_off_over_a_container_that_does_not_read_back(
+            "crash-judge-gpu-i923-tombstone-root-entry-pointer-birth-tree-unreadable-container",
+        ),
+        &["I-9.23", "I-2.1"],
+    );
+}
+
+/// 新池新建文件的流上 inode 叶容器盘序大的那一份的前 16 KiB 换成改之前那个 inode 树根（码 2）的原样字节；`checksum_of_the_second_location`
+/// 说 inode 树根第一条条目子指针第 1 条位置条目的整单元校验和怎么办：换成那 16 KiB 的 CRC（重封到根，那一份自证成码 2），或不动（那一份自证不过）。
+#[derive(Clone, Copy, Debug)]
+enum ChecksumOfTheSecondLocation {
+    MatchesTheNodeBytes,
+    LeftAsTheLeafChecksum,
+}
+
+/// 见 [`ChecksumOfTheSecondLocation`]：流名 `name`，叶容器盘序小的那一份不动（码 3，按它的位置条目自证得过）。
+fn record_new_pool_with_the_second_copy_of_the_inode_leaf_holding_the_inode_root_bytes(
+    tag: &str,
+    name: &'static str,
+    checksum_of_the_second_location: ChecksumOfTheSecondLocation,
+) -> RecordedFlow {
+    let mut flow = record_new_pool_file_creation(tag, name);
+    let leaf_copies = last_unit_copies(&flow, name, &is_an_inode_leaf_container, false);
+    let second_leaf_copy =
+        if flow.writes[leaf_copies[0]].device.0 < flow.writes[leaf_copies[1]].device.0 {
+            leaf_copies[1]
+        } else {
+            leaf_copies[0]
+        };
+    let root_copies = last_unit_copies(
+        &flow,
+        name,
+        &is_the_inode_tree_root_over_leaf_containers,
+        false,
+    );
+    assert_the_first_entry_names_the_unit_of(
+        &flow,
+        root_copies[0],
+        &leaf_copies,
+        "inode 树根第一条条目指着流里最后写的那片叶容器",
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
+    let second_location = first_entry_of_a_container_number_node()
+        + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET
+        + POINTER_LOCATION_ENTRY_OFFSETS[1];
+    assert_eq!(
+        device_and_slot_of_a_pointer_location(
+            &flow.writes[root_copies[0]].bytes().expect("单元写带字节")
+                [first_entry_of_a_container_number_node()
+                    + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET..],
+            POINTER_LOCATION_ENTRY_OFFSETS[1],
+        )
+        .0,
+        flow.writes[second_leaf_copy].device.0,
+        "第 1 条位置条目是盘序大的那块盘"
+    );
+    match checksum_of_the_second_location {
+        ChecksumOfTheSecondLocation::MatchesTheNodeBytes => {
+            let checksum_of_the_node_bytes = crc32c(&node_bytes);
+            corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
+                let field = second_location + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET;
+                bytes[field..field + 4].copy_from_slice(&checksum_of_the_node_bytes.to_le_bytes());
+            });
+        }
+        ChecksumOfTheSecondLocation::LeftAsTheLeafChecksum => {}
+    }
+    let WrittenContents::Bytes(leaf) = &mut flow.writes[second_leaf_copy].contents else {
+        panic!("单元写是普通写");
+    };
+    leaf[..node_bytes.len()].copy_from_slice(&node_bytes);
+    flow
+}
+
+/// 第 28 批 H-V（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 2 条；攻方 `research/prompts/m3-batch26-code-r1-opus-output.md`
+/// 第 1.3 节那一形，条目不改、自证成另一类的那份在盘序大的那一侧）：inode 树根第一条条目类型段 2，叶容器盘 0 那一份是码 3、自证得过，
+/// 盘 1 那一份是码 2、按改过的位置条目也自证得过。CPU 两份都看、认出码 2，红 I-9.2（改之前在第一份自证过的副本处停、交回 None，I-9.2 报成立）；
+/// GPU `w_child_unit_class_other_than_the_entry_type_names` 同改，每个状态逐位相同。这条流也走 GPU 第二份副本那一支自证得过的那一侧。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_verifies_as_a_node() {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_verifies_as_a_node",
+        &record_new_pool_with_the_second_copy_of_the_inode_leaf_holding_the_inode_root_bytes(
+            "crash-judge-gpu-i92-inode-leaf-second-copy-verifying-as-a-node",
+            "new-pool-inode-leaf-second-copy-verifying-as-a-node",
+            ChecksumOfTheSecondLocation::MatchesTheNodeBytes,
+        ),
+        &["I-9.2"],
+    );
+}
+
+/// 第 28 批正推线索①（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 3 条「GPU 第二份副本那一支补对拍流与变异」）：
+/// 同上一条，位置条目不改：盘 1 那一份头里是码 2、按叶容器的整单元校验和自证不过（陈旧副本）。CPU 不把它当另一类，照类型段 2 读盘 0 那一份，
+/// I-9.2 不红，那一份对不上校验和、I-2.1 红；GPU 第二份副本那一支也要自证（只认头里的类的话 GPU 红 I-9.2、CPU 不红），每个状态逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_is_a_stale_node_that_does_not_verify(
+) {
+    let test_name = "the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_is_a_stale_node_that_does_not_verify";
+    let flow = record_new_pool_with_the_second_copy_of_the_inode_leaf_holding_the_inode_root_bytes(
+        "crash-judge-gpu-i92-inode-leaf-second-copy-stale-node",
+        "new-pool-inode-leaf-second-copy-stale-node",
+        ChecksumOfTheSecondLocation::LeftAsTheLeafChecksum,
+    );
+    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        comparison.cpu_checker_red_invariants.contains("I-2.1"),
+        "流 {} 上 CPU 池级 checker 要红 I-2.1（红过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_red_invariants
+    );
+    assert!(
+        !comparison.cpu_checker_red_invariants.contains("I-9.2"),
+        "流 {} 上自证不过的第二份不算另一类：CPU 池级 checker 不红 I-9.2（红过的：{:?}）",
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
 /// GPU 关门那一侧（判决 `research/prompts/m3-batch25-code-r1-main-verification.md` 第三节第 2 条）：最新根的实例表照 `change` 改成主走读走不到末片的样子，
 /// 门要关。CPU 一侧先钉流造对了——改之前整条落盘的镜像上 I-9.29 真被评估过且成立；改之后整条落盘的镜像上 I-9.29 报部分没判。
 /// 流上 C 还是最新根的那几个状态 CPU 红 I-9.29；GPU 每个状态逐位相同（实例 3 的根落了、C 写回了的状态上，GPU 门要是开了，
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 229f65a..a168ee0 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -11010,6 +11010,399 @@ fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read
     );
 }
 
+/// 第 28 批正推线索②（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 3 条「读不出孩子那两条坏镜像补『只改身份一段』三形」）：
+/// inode 树根第一条条目类型段 2 → 0、身份引用三段先清零再只把 `field` 那一段写成 1，孩子读不出（同
+/// [`an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant`]）。
+/// 三段里只有一段不为 0，I-9.2 照判身份红，违例说明写出的三段就是 `expected_three_fields`（「出生树 / 容器号 / 出生代」）。
+fn assert_one_non_zero_identity_field_of_a_type_zero_inode_entry_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
+    tag: &str,
+    field: std::ops::Range<usize>,
+    expected_three_fields: &str,
+) {
+    let image = image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
+        build_pool(tag).memory_pool(),
+        |entry| {
+            change_the_entry_type(
+                entry,
+                PACKED_RECORD_TYPE_INODE,
+                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+            );
+            zero_the_three_identity_fields(entry);
+            entry[field].copy_from_slice(&1u64.to_le_bytes());
+        },
+    );
+    assert!(
+        matches!(
+            verdict(&image, "I-9.2"),
+            InvariantVerdict::Violated(detail)
+                if detail.contains("孩子两份都读不出")
+                    && detail.contains(&format!("{expected_three_fields}（要三段为 0）"))
+        ),
+        "类型段 0 的条目身份只有一段不为 0（{expected_three_fields}）、孩子读不出：I-9.2 照判身份红：{:?}",
+        verdict(&image, "I-9.2")
+    );
+}
+
+/// 条目身份引用里容器出生代那 8 字节（容器号 18..26 之后）。
+const INTERNAL_ENTRY_IDENTITY_CONTAINER_BIRTH: std::ops::Range<usize> = 26..34;
+
+#[test]
+fn an_inode_tree_entry_of_type_zero_whose_identity_carries_only_a_birth_tree_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
+) {
+    assert_one_non_zero_identity_field_of_a_type_zero_inode_entry_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
+        "known-bad-inode-entry-type-zero-only-birth-tree-unreadable-child",
+        INTERNAL_ENTRY_IDENTITY_BIRTH_TREE,
+        "1 / 0 / 0",
+    );
+}
+
+#[test]
+fn an_inode_tree_entry_of_type_zero_whose_identity_carries_only_a_container_number_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
+) {
+    assert_one_non_zero_identity_field_of_a_type_zero_inode_entry_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
+        "known-bad-inode-entry-type-zero-only-container-number-unreadable-child",
+        CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_OFFSET..INTERNAL_ENTRY_IDENTITY_CONTAINER_BIRTH.start,
+        "0 / 1 / 0",
+    );
+}
+
+#[test]
+fn an_inode_tree_entry_of_type_zero_whose_identity_carries_only_a_container_birth_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
+) {
+    assert_one_non_zero_identity_field_of_a_type_zero_inode_entry_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
+        "known-bad-inode-entry-type-zero-only-container-birth-unreadable-child",
+        INTERNAL_ENTRY_IDENTITY_CONTAINER_BIRTH,
+        "0 / 0 / 1",
+    );
+}
+
+/// 条目里子指针头部出生树那 8 字节：子指针在条目偏移 34 起，出生树在指针头部偏移 34（D19（块指针的结构与宽度预算） 已定项 11 的偏移表）。
+const INTERNAL_ENTRY_CHILD_POINTER_BIRTH_TREE: std::ops::Range<usize> =
+    CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + 34..CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + 42;
+
+/// 条目里 `field` 那 8 字节（出生树那两段之一）加 1。
+fn add_one_to_a_birth_tree_field_of_the_entry(entry: &mut [u8], field: std::ops::Range<usize>) {
+    let birth_tree = get_u64(entry, field.start);
+    entry[field].copy_from_slice(&(birth_tree + 1).to_le_bytes());
+}
+
+/// 种类 `kind` 那棵树的根（层级 1）第一条条目指着的码 3 容器两份各翻一个载荷字节（不重封、不补引用链：两份都读不出），条目不改。
+fn image_whose_first_container_of_the_root_of_kind_does_not_read_back(
+    clean: &MemoryPool,
+    kind: u16,
+) -> MemoryPool {
+    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, kind, 0);
+    let mut image = clean.clone();
+    break_both_copies_of_the_unit(
+        &mut image,
+        common_pointer_head_corruption::the_slot_of(&child).0,
+        common_pointer_head_corruption::data_unit_bytes(),
+    );
+    image
+}
+
+/// H-B（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第二节 K2 H3 H-B 那一行、第三节第 1 条；攻方
+/// `research/prompts/m3-batch26-code-r1-opus-output.md` 第 2.2 节 U 段 BT / IDB 两种改法）的判法：种类 `kind` 那棵树的根（层级 1）第一条条目
+/// 类型段是容器那一类（`container_entry_type`），它指的容器两份都读不出。对照（条目不改）：`entry_invariant` 不红——条目其余都对，
+/// 「子指针头部的出生树 == 身份引用的出生树」成立，比孩子头的那几小句读不出不判（正推线索③：不过判）。
+/// 再把条目里 `field`（子指针头部出生树或身份引用出生树）加 1：那一小句只比父条目里的两段、不看孩子，`entry_invariant` 红，
+/// 违例说明写出两段出生树（改之前读不出孩子时整支不判，`entry_invariant` 报成立）。
+fn assert_a_birth_tree_field_one_off_reddens_the_entry_invariant_over_a_container_that_does_not_read_back(
+    clean: &MemoryPool,
+    kind: u16,
+    container_entry_type: u16,
+    field: std::ops::Range<usize>,
+    entry_invariant: &'static str,
+) {
+    let unreadable_child_only =
+        image_whose_first_container_of_the_root_of_kind_does_not_read_back(clean, kind);
+    assert!(
+        !matches!(
+            verdict(&unreadable_child_only, entry_invariant),
+            InvariantVerdict::Violated(_)
+        ),
+        "对照：条目不改、容器两份都读不出，{entry_invariant} 不红：{:?}",
+        verdict(&unreadable_child_only, entry_invariant)
+    );
+    let mut image = unreadable_child_only;
+    let mut birth_trees_after_the_change = (0, 0);
+    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
+        assert_eq!(
+            entry_type_of(entry),
+            container_entry_type,
+            "根第一条条目指容器"
+        );
+        add_one_to_a_birth_tree_field_of_the_entry(entry, field);
+        birth_trees_after_the_change = (
+            get_u64(entry, INTERNAL_ENTRY_CHILD_POINTER_BIRTH_TREE.start),
+            get_u64(entry, INTERNAL_ENTRY_IDENTITY_BIRTH_TREE.start),
+        );
+    });
+    assert_eq!(level, 1, "根层级 1、直接管容器");
+    let (pointer_birth_tree, identity_birth_tree) = birth_trees_after_the_change;
+    assert!(
+        matches!(
+            verdict(&image, entry_invariant),
+            InvariantVerdict::Violated(detail)
+                if detail.contains("两份都读不出")
+                    && detail.contains(&format!(
+                        "子指针头部的出生树 {pointer_birth_tree} 不等于身份引用的出生树 {identity_birth_tree}"
+                    ))
+        ),
+        "出生树那两段不等、容器两份都读不出：{entry_invariant} 照判那一小句、红：{:?}",
+        verdict(&image, entry_invariant)
+    );
+}
+
+#[test]
+fn an_inode_tree_entry_whose_identity_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back_reddens_the_inode_entry_invariant(
+) {
+    assert_a_birth_tree_field_one_off_reddens_the_entry_invariant_over_a_container_that_does_not_read_back(
+        &build_pool("known-bad-inode-entry-identity-birth-tree-unreadable-leaf").memory_pool(),
+        TREE_KIND_INODE,
+        PACKED_RECORD_TYPE_INODE,
+        INTERNAL_ENTRY_IDENTITY_BIRTH_TREE,
+        "I-9.2",
+    );
+}
+
+#[test]
+fn a_tombstone_tree_entry_whose_child_pointer_birth_tree_is_one_off_over_a_container_that_does_not_read_back_reddens_the_tombstone_entry_invariant(
+) {
+    assert_a_birth_tree_field_one_off_reddens_the_entry_invariant_over_a_container_that_does_not_read_back(
+        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+            "known-bad-tombstone-entry-pointer-birth-tree-unreadable-container",
+        ),
+        TREE_KIND_TOMBSTONE,
+        PACKED_RECORD_TYPE_TOMBSTONE,
+        INTERNAL_ENTRY_CHILD_POINTER_BIRTH_TREE,
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+    );
+}
+
+#[test]
+fn a_lineage_tree_entry_whose_identity_birth_tree_is_one_off_over_a_container_that_does_not_read_back_reddens_the_lineage_entry_invariant(
+) {
+    assert_a_birth_tree_field_one_off_reddens_the_entry_invariant_over_a_container_that_does_not_read_back(
+        &image_after_rolling_back_to_the_first_root(
+            "known-bad-lineage-entry-identity-birth-tree-unreadable-container",
+        )
+        .image,
+        TREE_KIND_LINEAGE,
+        PACKED_RECORD_TYPE_LINEAGE,
+        INTERNAL_ENTRY_IDENTITY_BIRTH_TREE,
+        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+    );
+}
+
+/// 正推线索③（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第三节第 3 条「补一条类型段 2 读不出孩子、I-9.2 不判的坏镜像（钉住不过判）」）：
+/// 新池新建文件那一版，inode 树根第一条条目不改（类型段 2、两段出生树相等），它指的叶容器两份都读不出。
+/// 比孩子头的那几小句读不出不判，「子指针头部的出生树 == 身份引用的出生树」成立：I-9.2 不红（报成立或不判都行）。
+#[test]
+fn an_inode_tree_entry_of_the_leaf_container_type_with_agreeing_birth_trees_over_a_leaf_that_does_not_read_back_leaves_the_inode_entry_invariant_unviolated(
+) {
+    let image = image_whose_first_container_of_the_root_of_kind_does_not_read_back(
+        &build_pool("known-bad-inode-entry-agreeing-birth-trees-unreadable-leaf").memory_pool(),
+        TREE_KIND_INODE,
+    );
+    assert!(
+        matches!(verdict(&image, "I-2.1"), InvariantVerdict::Violated(_)),
+        "叶容器两份都读不出：I-2.1 红（钉住这张镜像真把孩子弄成读不出）：{:?}",
+        verdict(&image, "I-2.1")
+    );
+    assert!(
+        !matches!(verdict(&image, "I-9.2"), InvariantVerdict::Violated(_)),
+        "类型段 2、两段出生树相等、孩子读不出：I-9.2 不过判：{:?}",
+        verdict(&image, "I-9.2")
+    );
+}
+
+/// H-V（判决 `research/prompts/m3-batch26-code-r1-main-verification.md` 第二节 K2 H1 H-V 那一行、第三节第 2 条；攻方
+/// `research/prompts/m3-batch26-code-r1-opus-output.md` 第 1.3 节）的造法：种类 `kind` 那棵树的根（层级 1）第一条条目先照 `change_the_entry` 改，
+/// 再把它子指针第 `location_index` 条位置条目的整单元校验和换成这棵树根（码 2）原样 16 KiB 的 CRC、重封到根，那块盘上孩子的槽起写进这 16 KiB；
+/// 另一块盘那一份不动。两份按各自的位置条目都自证得过：一份码 2，一份是原来那片码 3 容器。
+fn image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
+    clean: &MemoryPool,
+    kind: u16,
+    location_index: usize,
+    change_the_entry: &dyn Fn(&mut [u8]),
+) -> MemoryPool {
+    let child = child_pointer_of_an_entry_of_the_root_of_kind(clean, kind, 0);
+    let root_bytes = root_node_bytes_of_the_tree_of_kind(clean, kind);
+    assert_eq!(
+        root_bytes[UNIT_HEADER_CLASS_TAG_OFFSET], UNIT_CLASS_INDEX_NODE,
+        "写进孩子槽的那 16 KiB 是码 2 节点"
+    );
+    let checksum_of_the_root_bytes = crc32_castagnoli(&root_bytes);
+    let mut image = clean.clone();
+    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
+        change_the_entry(entry);
+        let pointer_field = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
+            ..CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET + CONTAINER_TREE_ENTRY_CHILD_POINTER_BYTES;
+        let mut pointer = singlefs_core::pointer::NodePointer::read_from(
+            &mut singlefs_core::bytes::ByteReader::at(entry, pointer_field.start),
+        );
+        pointer.locations[location_index].unit_checksum = checksum_of_the_root_bytes;
+        entry[pointer_field].copy_from_slice(&common_pointer_head_corruption::node_pointer_bytes(
+            &pointer,
+        ));
+    });
+    assert_eq!(level, 1, "根层级 1、直接管容器");
+    write(
+        &mut image,
+        child.locations[location_index].device.0,
+        common_pointer_head_corruption::the_slot_of(&child).0 * SLOT,
+        &root_bytes,
+    );
+    image
+}
+
+/// H-V 的判法：两种盘序（码 2 那一份在位置条目 0 / 1）各判一次，`entry_invariant` 都红、违例说明点名孩子头里的
+/// `class_named_in_the_violation`，两种盘序红的同一批（改之前只看第一份自证过的副本：条目不改时码 2 那份在盘 1 上漏判，
+/// 条目改成类型段 0 时码 2 那份在盘 0 上漏判、另连带 I-7.2 等）。
+fn assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
+    clean: &MemoryPool,
+    kind: u16,
+    change_the_entry: &dyn Fn(&mut [u8]),
+    entry_invariant: &'static str,
+    class_named_in_the_violation: &str,
+) {
+    let mut violated_on_each_device = Vec::new();
+    for location_index in 0..2 {
+        let image = image_whose_first_root_entry_names_a_child_verifying_as_a_node_on_one_device(
+            clean,
+            kind,
+            location_index,
+            change_the_entry,
+        );
+        let verdicts = check_pool_image(&image);
+        let entry_verdict = verdicts
+            .iter()
+            .find(|(identifier, _)| *identifier == entry_invariant)
+            .map(|(_, entry_verdict)| entry_verdict)
+            .expect("清单里有");
+        assert!(
+            matches!(
+                entry_verdict,
+                InvariantVerdict::Violated(detail)
+                    if detail.contains(&format!("孩子头里的类标签是 {class_named_in_the_violation}"))
+            ),
+            "码 2 那一份在位置条目 {location_index}：{entry_invariant} 红、说明点名孩子头里的 {class_named_in_the_violation}：{verdicts:?}"
+        );
+        violated_on_each_device.push(violated_invariant_set(&verdicts));
+    }
+    assert_eq!(
+        violated_on_each_device[0], violated_on_each_device[1],
+        "同一张镜像只差码 2 那一份在哪块盘：红的同一批"
+    );
+}
+
+/// 条目不改那一形：类型段说码 3，一份自证成码 2。
+fn leave_the_entry_as_it_is(_entry: &mut [u8]) {}
+
+#[test]
+fn an_inode_tree_entry_whose_leaf_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_inode_entry_invariant_alike_on_either_device(
+) {
+    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
+        &build_pool("known-bad-inode-entry-leaf-copy-verifying-as-a-node").memory_pool(),
+        TREE_KIND_INODE,
+        &leave_the_entry_as_it_is,
+        "I-9.2",
+        "IndexNode",
+    );
+}
+
+#[test]
+fn an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_inode_entry_invariant_alike_on_either_device(
+) {
+    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
+        &build_pool("known-bad-inode-entry-type-zero-leaf-copy-verifying-as-a-node").memory_pool(),
+        TREE_KIND_INODE,
+        &|entry| {
+            change_the_entry_type(
+                entry,
+                PACKED_RECORD_TYPE_INODE,
+                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+            );
+            zero_the_three_identity_fields(entry);
+        },
+        "I-9.2",
+        "PackedRecordUnit",
+    );
+}
+
+#[test]
+fn a_tombstone_tree_entry_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_tombstone_entry_invariant_alike_on_either_device(
+) {
+    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
+        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+            "known-bad-tombstone-entry-container-copy-verifying-as-a-node",
+        ),
+        TREE_KIND_TOMBSTONE,
+        &leave_the_entry_as_it_is,
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        "IndexNode",
+    );
+}
+
+#[test]
+fn a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_tombstone_entry_invariant_alike_on_either_device(
+) {
+    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
+        &common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+            "known-bad-tombstone-entry-type-zero-container-copy-verifying-as-a-node",
+        ),
+        TREE_KIND_TOMBSTONE,
+        &|entry| {
+            change_the_entry_type(
+                entry,
+                PACKED_RECORD_TYPE_TOMBSTONE,
+                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+            );
+            zero_the_three_identity_fields(entry);
+        },
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        "PackedRecordUnit",
+    );
+}
+
+#[test]
+fn a_lineage_tree_entry_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_lineage_entry_invariant_alike_on_either_device(
+) {
+    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
+        &image_after_rolling_back_to_the_first_root(
+            "known-bad-lineage-entry-container-copy-verifying-as-a-node",
+        )
+        .image,
+        TREE_KIND_LINEAGE,
+        &leave_the_entry_as_it_is,
+        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        "IndexNode",
+    );
+}
+
+#[test]
+fn a_lineage_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_lineage_entry_invariant_alike_on_either_device(
+) {
+    assert_the_entry_invariant_reddens_alike_whichever_device_holds_the_copy_verifying_as_a_node(
+        &image_after_rolling_back_to_the_first_root(
+            "known-bad-lineage-entry-type-zero-container-copy-verifying-as-a-node",
+        )
+        .image,
+        TREE_KIND_LINEAGE,
+        &|entry| {
+            change_the_entry_type(
+                entry,
+                PACKED_RECORD_TYPE_LINEAGE,
+                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+            );
+            zero_the_three_identity_fields(entry);
+        },
+        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        "PackedRecordUnit",
+    );
+}
+
 // ─── 第 35 批：由记录造出来的两类版本取哪条记录的新根段（FL）、树表指针全零（Q1、被抛弃的根）、I-5.5 认出来而没有数据单元（Q4） ───
 
 /// 恰好要 `data_units` 个数据单元的内容（最后一个单元装一半）：顺序写一次发布切成 `data_units` 条记录（一个数据单元一条）。
````

## 实现员 k：mutations-append.tsv（/tmp/claude-1000/m3-batch28/k/patch/mutations-append.tsv）

````text
第28批 H-B inode 树读不出孩子时类型段 2 不判子指针出生树（条目身份出生树加 1、叶容器两份都读不出，I-9.2 报成立）	crates/singlefs-checker/src/walk.rs	                birth_tree_in_the_child_pointer_head == identity_birth_tree,	                true,	-p singlefs-harness --test checker_known_bad_images -- carries_only_a is_one_off_over agreeing_birth_trees	an_inode_tree_entry_whose_identity_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back_reddens_the_inode_entry_invariant
第28批 正推③ inode 树读不出孩子时类型段 2 过判（两段出生树相等也判 I-9.2 红）	crates/singlefs-checker/src/walk.rs	                birth_tree_in_the_child_pointer_head == identity_birth_tree,	                false,	-p singlefs-harness --test checker_known_bad_images -- carries_only_a is_one_off_over agreeing_birth_trees	an_inode_tree_entry_of_the_leaf_container_type_with_agreeing_birth_trees_over_a_leaf_that_does_not_read_back_leaves_the_inode_entry_invariant_unviolated
第28批 H-B 墓碑树、谱系树容器读不出时不判子指针出生树（墓碑树条目子指针出生树加 1，I-9.23 报成立）	crates/singlefs-checker/src/walk.rs	                child.birth_tree == identity_birth_tree,	                true,	-p singlefs-harness --test checker_known_bad_images -- carries_only_a is_one_off_over agreeing_birth_trees	a_tombstone_tree_entry_whose_child_pointer_birth_tree_is_one_off_over_a_container_that_does_not_read_back_reddens_the_tombstone_entry_invariant
第28批 H-B 墓碑树、谱系树容器读不出时不判子指针出生树（谱系树条目身份出生树加 1，I-9.17 报成立）	crates/singlefs-checker/src/walk.rs	                child.birth_tree == identity_birth_tree,	                true,	-p singlefs-harness --test checker_known_bad_images -- carries_only_a is_one_off_over agreeing_birth_trees	a_lineage_tree_entry_whose_identity_birth_tree_is_one_off_over_a_container_that_does_not_read_back_reddens_the_lineage_entry_invariant
第28批 正推② inode 树读不出孩子时类型段 0 身份不判出生树那一段（只有出生树不为 0 的条目 I-9.2 报成立）	crates/singlefs-checker/src/walk.rs	            let identity_is_zero = birth_tree == 0 && container == 0 && container_birth == 0;	            let identity_is_zero = (birth_tree == 0 || true) && container == 0 && container_birth == 0;	-p singlefs-harness --test checker_known_bad_images -- carries_only_a is_one_off_over agreeing_birth_trees	an_inode_tree_entry_of_type_zero_whose_identity_carries_only_a_birth_tree_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant
第28批 正推② inode 树读不出孩子时类型段 0 身份不判容器号那一段（只有容器号不为 0 的条目 I-9.2 报成立）	crates/singlefs-checker/src/walk.rs	            let identity_is_zero = birth_tree == 0 && container == 0 && container_birth == 0;	            let identity_is_zero = birth_tree == 0 && (container == 0 || true) && container_birth == 0;	-p singlefs-harness --test checker_known_bad_images -- carries_only_a is_one_off_over agreeing_birth_trees	an_inode_tree_entry_of_type_zero_whose_identity_carries_only_a_container_number_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant
第28批 正推② inode 树读不出孩子时类型段 0 身份不判出生代那一段（只有出生代不为 0 的条目 I-9.2 报成立）	crates/singlefs-checker/src/walk.rs	            let identity_is_zero = birth_tree == 0 && container == 0 && container_birth == 0;	            let identity_is_zero = birth_tree == 0 && container == 0 && (container_birth == 0 || true);	-p singlefs-harness --test checker_known_bad_images -- carries_only_a is_one_off_over agreeing_birth_trees	an_inode_tree_entry_of_type_zero_whose_identity_carries_only_a_container_birth_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant
第28批 H-V 认孩子的类在第一份自证过的副本处停（inode 树条目不改、叶容器一份自证成码 2，条目那一条随那一份在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	            class_other_than_the_entry_type_names = Some(class_in_the_header);\n        }\n    }	            class_other_than_the_entry_type_names = Some(class_in_the_header);\n        }\n        break;\n    }	-p singlefs-harness --test checker_known_bad_images -- verifying_as_a_node_on_one_device	an_inode_tree_entry_whose_leaf_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_inode_entry_invariant_alike_on_either_device
第28批 H-V 认孩子的类在第一份自证过的副本处停（inode 树条目改成类型段 0、叶容器一份自证成码 2，条目那一条随那一份在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	            class_other_than_the_entry_type_names = Some(class_in_the_header);\n        }\n    }	            class_other_than_the_entry_type_names = Some(class_in_the_header);\n        }\n        break;\n    }	-p singlefs-harness --test checker_known_bad_images -- verifying_as_a_node_on_one_device	an_inode_tree_entry_of_type_zero_whose_leaf_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_inode_entry_invariant_alike_on_either_device
第28批 H-V 认孩子的类在第一份自证过的副本处停（墓碑树条目不改、容器一份自证成码 2，条目那一条随那一份在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	            class_other_than_the_entry_type_names = Some(class_in_the_header);\n        }\n    }	            class_other_than_the_entry_type_names = Some(class_in_the_header);\n        }\n        break;\n    }	-p singlefs-harness --test checker_known_bad_images -- verifying_as_a_node_on_one_device	a_tombstone_tree_entry_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_tombstone_entry_invariant_alike_on_either_device
第28批 H-V 认孩子的类在第一份自证过的副本处停（墓碑树条目改成类型段 0、容器一份自证成码 2，条目那一条随那一份在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	            class_other_than_the_entry_type_names = Some(class_in_the_header);\n        }\n    }	            class_other_than_the_entry_type_names = Some(class_in_the_header);\n        }\n        break;\n    }	-p singlefs-harness --test checker_known_bad_images -- verifying_as_a_node_on_one_device	a_tombstone_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_tombstone_entry_invariant_alike_on_either_device
第28批 H-V 认孩子的类在第一份自证过的副本处停（谱系树条目不改、容器一份自证成码 2，条目那一条随那一份在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	            class_other_than_the_entry_type_names = Some(class_in_the_header);\n        }\n    }	            class_other_than_the_entry_type_names = Some(class_in_the_header);\n        }\n        break;\n    }	-p singlefs-harness --test checker_known_bad_images -- verifying_as_a_node_on_one_device	a_lineage_tree_entry_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_lineage_entry_invariant_alike_on_either_device
第28批 H-V 认孩子的类在第一份自证过的副本处停（谱系树条目改成类型段 0、容器一份自证成码 2，条目那一条随那一份在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	            class_other_than_the_entry_type_names = Some(class_in_the_header);\n        }\n    }	            class_other_than_the_entry_type_names = Some(class_in_the_header);\n        }\n        break;\n    }	-p singlefs-harness --test checker_known_bad_images -- verifying_as_a_node_on_one_device	a_lineage_tree_entry_of_type_zero_whose_container_has_a_copy_verifying_as_a_node_on_one_device_reddens_the_lineage_entry_invariant_alike_on_either_device
第28批 GPU 判器 H-B：inode 树根读不出孩子时类型段 2 不判子指针出生树（条目身份出生树加 1、叶容器两份都读不出那条流上 CPU 红 I-9.2，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            if (record_type == 2u) {\n                w_judge(INVARIANT_I_9_2, eq64(child.birth_tree, identity_birth_tree));\n            }\n		-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_identity_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back	the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_identity_birth_tree_is_one_off_over_a_leaf_that_does_not_read_back
第28批 GPU 判器 H-B：墓碑树、谱系树容器读不出时不判子指针出生树（墓碑树条目子指针出生树加 1、容器两份都读不出那条流上 CPU 红 I-9.23，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            w_judge(identity_invariant, eq64(child.birth_tree, rd64(n.base, o + 8u)));\n		-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_pointer_birth_tree_is_one_off_over_a_container_that_does_not_read_back	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_pointer_birth_tree_is_one_off_over_a_container_that_does_not_read_back
第28批 GPU 判器 H-V：认孩子的类在第一份自证过的副本处停（叶容器盘 1 那一份自证成码 2 那条流上 CPU 红 I-9.2，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                class_other_than_the_entry_type_names = class_in_the_first_header;\n            }\n	                class_other_than_the_entry_type_names = class_in_the_first_header;\n            }\n            return class_other_than_the_entry_type_names;\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_verifies_as_a_node	the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_verifies_as_a_node
第28批 GPU 判器 正推①：第二份副本不自证、只认头里的类（叶容器盘 1 那一份是自证不过的码 2 陈旧副本那条流上 GPU 红 I-9.2，CPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        if (w_unit_verifies_at_the_length_of_its_class(second_copy, class_in_the_second_header, child.sum1)) {	        if (true || w_unit_verifies_at_the_length_of_its_class(second_copy, class_in_the_second_header, child.sum1)) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_is_a_stale_node_that_does_not_verify	the_gpu_judge_agrees_with_the_cpu_when_the_second_copy_of_an_inode_leaf_is_a_stale_node_that_does_not_verify
````

## 实现员 k：mutations-replacements.tsv（/tmp/claude-1000/m3-batch28/k/patch/mutations-replacements.tsv）

````text
第24批 K I-9.2 同形：孩子头里的类与类型段说的不符也当相符（身份三段为 0 的 inode 条目指码 3 叶容器）	crates/singlefs-checker/src/walk.rs	        if Some(class_in_the_header) != class_the_entry_type_names	        if false && Some(class_in_the_header) != class_the_entry_type_names	-p singlefs-harness --test checker_known_bad_images -- an_inode_tree_entry_of_type_zero_with_a_zero_identity_over_a_leaf_container_reddens_only_the_inode_entry_invariant	an_inode_tree_entry_of_type_zero_with_a_zero_identity_over_a_leaf_container_reddens_only_the_inode_entry_invariant
第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（墓碑树 N4-i 加陈旧副本，I-9.23 随陈旧副本在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	        if !verifies_at_the_length_of_its_class {\n            continue;\n        }	        if Some(class_in_the_header) == class_the_entry_type_names\n            && class_other_than_the_entry_type_names.is_none()\n        {\n            break;\n        }\n        if !verifies_at_the_length_of_its_class {\n            continue;\n        }	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	a_tombstone_tree_entry_of_type_zero_over_a_container_with_a_stale_copy_of_the_child_reddens_the_tombstone_entry_invariant_on_either_device
第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（层级 2 墓碑树 N4-f 加陈旧副本，I-9.23 报成立、连带 I-3.1、I-3.11、I-5.1、I-7.2）	crates/singlefs-checker/src/walk.rs	        if !verifies_at_the_length_of_its_class {\n            continue;\n        }	        if Some(class_in_the_header) == class_the_entry_type_names\n            && class_other_than_the_entry_type_names.is_none()\n        {\n            break;\n        }\n        if !verifies_at_the_length_of_its_class {\n            continue;\n        }	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	a_tombstone_tree_entry_of_the_container_type_over_a_node_with_a_stale_copy_of_the_child_reddens_the_tombstone_entry_invariant_on_either_device
第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（谱系树类型段 0 加陈旧副本，I-9.17 随陈旧副本在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	        if !verifies_at_the_length_of_its_class {\n            continue;\n        }	        if Some(class_in_the_header) == class_the_entry_type_names\n            && class_other_than_the_entry_type_names.is_none()\n        {\n            break;\n        }\n        if !verifies_at_the_length_of_its_class {\n            continue;\n        }	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	a_lineage_tree_entry_of_type_zero_over_a_container_with_a_stale_copy_of_the_child_reddens_the_lineage_entry_invariant_on_either_device
第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（inode 树类型段 0 加陈旧副本，I-9.2 报成立、连带 I-7.2 等）	crates/singlefs-checker/src/walk.rs	        if !verifies_at_the_length_of_its_class {\n            continue;\n        }	        if Some(class_in_the_header) == class_the_entry_type_names\n            && class_other_than_the_entry_type_names.is_none()\n        {\n            break;\n        }\n        if !verifies_at_the_length_of_its_class {\n            continue;\n        }	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	an_inode_tree_entry_of_type_zero_over_a_leaf_container_with_a_stale_copy_of_the_child_reddens_the_inode_entry_invariant_on_either_device
第26批 inode 树读不出孩子时 I-9.2 不判（类型段 0 身份三段不为 0的条目只记 I-2.1 与走读失败）	crates/singlefs-checker/src/walk.rs	                judge_an_inode_entry_whose_child_does_not_read_back(\n                    &mut self.judgements,\n                    registered_entry_type,\n                    identity,\n                    child.birth_tree,\n                );	                // 变异：inode 树读不出孩子时 I-9.2 不判	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant
第26批 GPU 判器 H1：第一份副本头里的类就是类型段那一类时不验整单元校验和（inode 树根条目类型段 0、叶容器盘 0 那一份陈旧那条流上 CPU 红 I-9.2，GPU 不判、记走读失败）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        let class_in_the_first_header = rd8(first_copy.base, 6u);	        let class_in_the_first_header = rd8(first_copy.base, 6u);\n        if (class_in_the_first_header == class_the_entry_type_names) {\n            return NONE;\n        }	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_without_an_identity_names_a_leaf_container_whose_first_copy_is_stale	the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_without_an_identity_names_a_leaf_container_whose_first_copy_is_stale
````
