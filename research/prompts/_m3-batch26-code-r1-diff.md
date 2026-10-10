# 删除批第 26 批合入的改动原样（2026-10-06）

## 实现员 h：crates.patch（/tmp/claude-1000/m3-batch26/h/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index 8839db6..f3be249 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -2630,6 +2630,13 @@ fn w_walk_inode_root(root: Node, tree: vec2<u32>, root_device: u32, root_slot: v
         w_note_pointer_references(child, span);
         let s = w_read_referenced_unit(child, length);
         if (s.version == NONE) {
+            // 孩子两份都读不出：I-9.2 里不看孩子就判得了的那几小句照判（walk.rs judge_an_inode_entry_whose_child_does_not_read_back，
+            // 与墓碑树、谱系树读不出孩子时同判）——类型段 0 判身份引用三段为 0，类型段不认得的判该条目损坏，类型段 2 不判
+            if (record_type == 0u) {
+                w_judge(INVARIANT_I_9_2, is_zero64(identity_birth_tree) && is_zero64(identity_container) && is_zero64(identity_birth));
+            } else if (record_type != 2u) {
+                w_judge(INVARIANT_I_9_2, false);
+            }
             w_failures = w_failures + 1u;
             continue;
         }
@@ -2945,27 +2952,29 @@ fn w_unit_verifies_at_the_length_of_its_class(s: Slice, unit_class: u32, checksu
 }
 
 // 树里一条内部条目的孩子是不是条目类型段说的那一类（walk.rs 的 child_unit_class_other_than_the_entry_type_names）：只读孩子的单元头取类标签，
-// 不按类型段定读多长。两条位置条目依次看：头按最短的单元（码 2，16 KiB）那么长读，头里的类标签就是 class_the_entry_type_names 的交回 NONE
-// （照类型段那一支走，读不出照旧归 I-2.1）；是另一类登记过的码、且按那一类的长度读回的整单元校验和对得上，交回那一类；
-// 读不出、类标签不在登记表里、校验和对不上的看下一条位置条目。class_the_entry_type_names 是 NONE（类型段不认得）时交回孩子自证过的那一类
+// 不按类型段定读多长。两条位置条目依次看：头按最短的单元（码 2，16 KiB）那么长读，头里的类标签是登记过的码、且按那一类的长度读回的
+// 整单元校验和对得上，这一份才算自证过的孩子——它的类就是 class_the_entry_type_names 的交回 NONE（照类型段那一支走），是另一类的交回那一类；
+// 读不出、类标签不在登记表里、校验和对不上的看下一条位置条目（陈旧副本、坏副本），哪一份都自证不过交回 NONE（读不出照旧归 I-2.1）。
+// 头里的类就是类型段那一类时同样要自证（判决 `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K2 N1 两故障 H1 那一行，walk.rs 同改）。
+// class_the_entry_type_names 是 NONE（类型段不认得）时交回孩子自证过的那一类
 fn w_child_unit_class_other_than_the_entry_type_names(child: Ptr, class_the_entry_type_names: u32) -> u32 {
     let first_copy = read_at(child.device0, slot_to_bytes(child.slot0), NODE_BYTES);
     if (first_copy.version != NONE) {
         let class_in_the_first_header = rd8(first_copy.base, 6u);
-        if (class_in_the_first_header == class_the_entry_type_names) {
-            return NONE;
-        }
         if (w_unit_verifies_at_the_length_of_its_class(first_copy, class_in_the_first_header, child.sum0)) {
+            if (class_in_the_first_header == class_the_entry_type_names) {
+                return NONE;
+            }
             return class_in_the_first_header;
         }
     }
     let second_copy = read_at(child.device1, slot_to_bytes(child.slot1), NODE_BYTES);
     if (second_copy.version != NONE) {
         let class_in_the_second_header = rd8(second_copy.base, 6u);
-        if (class_in_the_second_header == class_the_entry_type_names) {
-            return NONE;
-        }
         if (w_unit_verifies_at_the_length_of_its_class(second_copy, class_in_the_second_header, child.sum1)) {
+            if (class_in_the_second_header == class_the_entry_type_names) {
+                return NONE;
+            }
             return class_in_the_second_header;
         }
     }
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 3a605cb..3ab51d2 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -23,6 +23,8 @@
 //! inode 树根条目类型段 0 指码 3 叶容器（I-9.2 同形）一条同钉；
 //! I-9.29 判不了那一形（被崩溃恢复抛弃的根与现行线同号不同出生代、最新根的实例表两份翻坏）一条流整份比；
 //! 第 25 批：同一段历史、最新根的实例表链指针记录之前多一条 kind 2（攻方 N5-a，主走读走到末片、I-9.29 照比）一条流整份比。
+//! 第 26 批：inode 树根条目类型段 0、身份三段清零，叶容器盘 0 那一份换成码 2 节点的陈旧字节（攻方 H1：头里的类恰是类型段那一类的副本也要自证）
+//! 一条流钉 CPU 红 I-9.2、不红 I-7.2，GPU 逐位跟上。
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
 //! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）；这几处除实例表指针外
 //! 另各有一条 MAC 第 0 字节非 0 的坏流（core 连加密、压缩那几段一并判，GPU 恢复内核同判）；extent 上段叶内联的数据指针另有一条算法类型写 1 的坏流
@@ -5862,6 +5864,64 @@ fn record_new_pool_with_the_inode_root_entry_of_type_zero_naming_its_leaf_contai
     )
 }
 
+/// 节点指针里第一条位置条目的槽号（6 字节）的偏移：两条位置条目在 50 与 64（盘 4、槽 6、整单元校验和 4）。
+const POINTER_FIRST_LOCATION_SLOT_OFFSET: usize = 54;
+
+/// 攻方 H1（判决 `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K2 N1 两故障 H1 那一行，第 26 批改）的 inode 那一形：
+/// 新池新建文件的流上 inode 树根（层级 1）第一条条目的类型段 2 → 0、身份引用三段写 0、重封到根（除了孩子的类，这是一条合法的类型段 0 条目），
+/// 再把它指着的叶容器盘序小的那一份写的前 16 KiB 换成改之前那个 inode 树根的原样字节（陈旧副本：头里类标签 2，恰是类型段 0 说的那一类；
+/// 不补校验和，位置条目还是叶容器原来那一份的），盘序大的那一份不动。CPU 认孩子的类时头里是类型段那一类的副本也要按那一类的长度自证，
+/// 第一份自证不过看第二份、认出码 3，判 I-9.2 红、不往下走、不记走读失败（改之前只认第一份的头、不验，I-9.2 报成立，另连带 I-7.2 等）；
+/// GPU 走读内核同改（`w_child_unit_class_other_than_the_entry_type_names`）。
+fn record_new_pool_with_the_inode_root_entry_of_type_zero_and_no_identity_over_a_leaf_container_whose_first_copy_is_stale(
+    tag: &str,
+) -> RecordedFlow {
+    let name = "new-pool-inode-root-entry-type-zero-no-identity-first-leaf-copy-stale";
+    let mut flow = record_new_pool_file_creation(tag, name);
+    let inode_tree_root_over_leaf_containers = |bytes: &[u8]| {
+        unit_class(bytes) == 2
+            && usize::from(index_node_key_width(bytes)) == INODE_TREE_KEY_BYTES
+            && index_node_entry_width(bytes) == INODE_INTERNAL_ENTRY_BYTES
+            && read_u16_at(
+                bytes,
+                index_node_entries_start(INODE_TREE_KEY_BYTES)
+                    + CONTAINER_TREE_ENTRY_RECORD_TYPE_OFFSET,
+            ) == PACKED_RECORD_TYPE_INODE
+    };
+    let inode_leaf_container = |bytes: &[u8]| {
+        unit_class(bytes) == 3 && packed_record_type(bytes) == PACKED_RECORD_TYPE_INODE
+    };
+    let root_copies = last_unit_copies(&flow, name, &inode_tree_root_over_leaf_containers, false);
+    let stale_bytes = flow.writes[root_copies[0]]
+        .bytes()
+        .expect("单元写带字节")
+        .to_vec();
+    let first_leaf_copy = last_unit_copies(&flow, name, &inode_leaf_container, true)[0];
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
+        set_the_type_of_the_first_entry(bytes, 0);
+        clear_the_identity_of_the_first_entry_but_its_type(bytes);
+    });
+    let root_after_the_change = flow.writes[root_copies[0]].bytes().expect("单元写带字节");
+    let first_location_slot_offset = first_entry_of_a_container_number_node()
+        + INODE_INTERNAL_ENTRY_CHILD_POINTER_OFFSET
+        + POINTER_FIRST_LOCATION_SLOT_OFFSET;
+    let mut slot_named_by_the_first_entry = [0u8; 8];
+    slot_named_by_the_first_entry[..6].copy_from_slice(
+        &root_after_the_change[first_location_slot_offset..first_location_slot_offset + 6],
+    );
+    assert_eq!(
+        u64::from_le_bytes(slot_named_by_the_first_entry),
+        flow.writes[first_leaf_copy].offset.0 / 16384,
+        "inode 树根第一条条目指着的就是流里最后写的那片叶容器"
+    );
+    assert_eq!(unit_class(&stale_bytes), 2, "陈旧副本头里的类是码 2");
+    let WrittenContents::Bytes(leaf) = &mut flow.writes[first_leaf_copy].contents else {
+        panic!("单元写是普通写");
+    };
+    leaf[..stale_bytes.len()].copy_from_slice(&stale_bytes);
+    flow
+}
+
 /// I-9.18：删除中间态的流上墓碑容器里那一条墓碑的对象 ID 1 → 0、重封到根。队列 {1} 里的号 1 在墓碑树里没有墓碑：CPU 判 I-9.18 红；
 /// 号 0 没有记录，I-9.10 墓碑那一个等号与 I-9.19 没有对象，号 0 不越过水位，I-9.6 墓碑那一半照旧成立。
 fn record_deletion_with_the_tombstone_naming_an_object_without_a_record(tag: &str) -> RecordedFlow {
@@ -6787,3 +6847,39 @@ fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names
         "I-9.2",
     );
 }
+
+/// H1 那一形的流（[`record_new_pool_with_the_inode_root_entry_of_type_zero_and_no_identity_over_a_leaf_container_whose_first_copy_is_stale`]）：
+/// CPU 池级 checker 红 I-9.2、一个状态都不红 I-7.2（改之前只认第一份陈旧副本的头，I-9.2 报成立，记走读失败让最新根走不完、I-7.2 连带红）；
+/// 陈旧副本自己连带的 I-2.1 那几条照红，不在这里钉；GPU 每个状态逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_without_an_identity_names_a_leaf_container_whose_first_copy_is_stale(
+) {
+    let test_name = "the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_without_an_identity_names_a_leaf_container_whose_first_copy_is_stale";
+    let flow =
+        record_new_pool_with_the_inode_root_entry_of_type_zero_and_no_identity_over_a_leaf_container_whose_first_copy_is_stale(
+            "crash-judge-gpu-i92-inode-root-entry-type-zero-first-leaf-copy-stale",
+        );
+    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        comparison.cpu_checker_red_invariants.contains("I-9.2"),
+        "流 {} 上 CPU 池级 checker 要红 I-9.2（红过的：{:?}）",
+        flow.name,
+        comparison.cpu_checker_red_invariants
+    );
+    assert!(
+        !comparison.cpu_checker_red_invariants.contains("I-7.2"),
+        "流 {} 上第一份陈旧副本不挡住第二份：CPU 池级 checker 不红 I-7.2（红过的：{:?}）",
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
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 535c799..e7562dd 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -2532,6 +2532,11 @@ impl<'reader> Walk<'reader> {
                 length,
                 "inode 树的孩子",
             ) else {
+                judge_an_inode_entry_whose_child_does_not_read_back(
+                    &mut self.judgements,
+                    entry_type,
+                    identity,
+                );
                 self.walk_failures
                     .push("inode 树的孩子两份都读不到对得上的".to_string());
                 continue;
@@ -4093,8 +4098,11 @@ fn slots_spanned_by_a_unit_of_the_class(class: MappingKeyUnitClass) -> Option<u6
 /// 树里一条内部条目的孩子是不是条目类型段说的那一类：只读孩子的单元头取类标签，不按类型段定读多长（判决
 /// `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N4 那一行：按类型段定长度时，类型段与孩子真实的类不符，
 /// 按那个长度读回的整单元校验和对不上，身份那一判跑不到、却记了 I-2.1 与走读失败）。
-/// 逐条位置条目：头里的类标签就是 `class_the_entry_type_names` 的，交回 None（照类型段那一支走，读不出照旧归 I-2.1）；
-/// 是另一类登记过的码、且按那一类的长度读回的整单元校验和对得上，交回那一类；类标签不在登记表里、读不出、校验和对不上的看下一条位置条目。
+/// 逐条位置条目：头里的类标签是登记过的码、且按那一类的长度读回的整单元校验和对得上，这一份才算自证过的孩子——
+/// 自证过的孩子的类就是 `class_the_entry_type_names` 的，交回 None（照类型段那一支走）；是另一类的，交回那一类。
+/// 类标签不在登记表里、读不出、校验和对不上的看下一条位置条目（陈旧副本、坏副本），哪一份都自证不过交回 None（照类型段那一支走，读不出照旧归 I-2.1）。
+/// 头里的类就是类型段那一类时同样要自证（判决 `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K2 N1 两故障 H1 那一行：
+/// 只认第一份的头、不验，第一份是头里恰好是那一类的陈旧副本时第二份不看，结论随陈旧副本在哪块盘翻面）。
 /// 类型段不认得的条目（`class_the_entry_type_names` 是 None）没有「它说的那一类」，交回孩子自证过的那一类。
 /// 只认按它自己的长度自证过的孩子：头里的类标签坏了的单元按那个长度对不上校验和，不会被当成「另一类」。只读、不判。
 fn child_unit_class_other_than_the_entry_type_names(
@@ -4109,22 +4117,54 @@ fn child_unit_class_other_than_the_entry_type_names(
             continue;
         };
         let class_in_the_header = MappingKeyUnitClass::of(header[UNIT_HEADER_CLASS_TAG_OFFSET]);
-        if Some(class_in_the_header) == class_the_entry_type_names {
-            return None;
-        }
         let Some(unit_bytes) = unit_bytes_of_the_class(class_in_the_header) else {
             continue;
         };
         let verifies_at_the_length_of_its_class = reader
             .read(location.device, offset, unit_bytes)
             .is_some_and(|content| crc32_castagnoli_table(&content) == location.checksum);
-        if verifies_at_the_length_of_its_class {
-            return Some(class_in_the_header);
+        if !verifies_at_the_length_of_its_class {
+            continue;
         }
+        if Some(class_in_the_header) == class_the_entry_type_names {
+            return None;
+        }
+        return Some(class_in_the_header);
     }
     None
 }
 
+/// inode 树一条内部条目的孩子两份都读不出（[`child_unit_class_other_than_the_entry_type_names`] 也没认出另一类）时，I-9.2 里不看孩子就判得了的那几小句，
+/// 与墓碑树、谱系树读不出孩子时同判（`Walk::walk_the_entries_of_a_container_tree_node`；判决
+/// `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K1 N1 那一行「inode 树读不出孩子时 I-9.2 不判」）：
+/// 类型段 0 判身份引用三段为 0（「子单元是码 2」那一小句孩子读不出不判）；类型段不认得的判该条目损坏；类型段 2 那几小句都要比孩子的头，读不出不判。
+/// 读不出照旧归 I-2.1 与走读失败（调用方记）。
+fn judge_an_inode_entry_whose_child_does_not_read_back(
+    judgements: &mut Judgements,
+    entry_type: InodeInternalEntryType,
+    identity: (u64, u16, u64, u64),
+) {
+    match entry_type {
+        InodeInternalEntryType::LeafContainer => {}
+        InodeInternalEntryType::InternalNode => {
+            let (birth_tree, _, container, container_birth) = identity;
+            let identity_is_zero = birth_tree == 0 && container == 0 && container_birth == 0;
+            judgements.judge("I-9.2", identity_is_zero, || {
+                format!(
+                    "inode 内部条目类型段 0：孩子两份都读不出（类标签那一小句不判），身份引用的出生树 / 容器号 / 出生代 {birth_tree} / {container} / {container_birth}（要三段为 0）"
+                )
+            });
+        }
+        InodeInternalEntryType::Unregistered(unregistered_type) => {
+            judgements.judge("I-9.2", false, || {
+                format!(
+                    "inode 内部条目的类型段 {unregistered_type} 不在 {{0, 2}} 里（孩子两份都读不出）"
+                )
+            });
+        }
+    }
+}
+
 /// 这一遍里读过的码 2 单元：(设备身份, 起点槽, 位置条目里的整单元校验和) → 解开的节点（解不开记 None）。
 /// 候选集里几条根常指着同一个单元（暖机那几代都指着 mkfs 种的第 0 版树表，共享子树在每条根上都要算一遍），
 /// 按位置条目记一份，省掉重复的整单元 CRC——层 0 每个崩溃状态都要跑这一遍。
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 09cb6ac..a8c5fce 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -10618,3 +10618,341 @@ fn an_inode_leaf_whose_class_tag_alone_says_index_node_is_read_at_the_length_of_
         verdict(&image, "I-3.1")
     );
 }
+
+/// 码 2 节点与码 3 打包记录单元头里的类标签（单元头偏移 6）。
+const UNIT_CLASS_INDEX_NODE: u8 = 2;
+const UNIT_CLASS_PACKED_RECORD_UNIT: u8 = 3;
+/// 单元头里类标签的偏移。
+const UNIT_HEADER_CLASS_TAG_OFFSET: usize = 6;
+
+/// 最新的根那一版树表里种类 `kind` 那棵树的码 2 根第 `entry_index` 条条目的子指针。
+fn child_pointer_of_an_entry_of_the_root_of_kind(
+    image: &MemoryPool,
+    kind: u16,
+    entry_index: usize,
+) -> singlefs_core::pointer::NodePointer {
+    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
+    let root =
+        singlefs_core::unit::parse_index_node(&common_pointer_head_corruption::unit_bytes_at(
+            image,
+            common_pointer_head_corruption::the_slot_of(&root_pointer),
+            common_pointer_head_corruption::node_bytes(),
+        ))
+        .expect("根解得开");
+    singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(
+        &root.entries[entry_index],
+        CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
+    ))
+}
+
+/// 最新的根那一版树表里种类 `kind` 那棵树的码 2 根（16 KiB 原样字节）：拿来当别处一个码 2 单元的陈旧副本。
+fn root_node_bytes_of_the_tree_of_kind(image: &MemoryPool, kind: u16) -> Vec<u8> {
+    let root_pointer = common_pointer_head_corruption::tree_table_entry_of_kind(image, kind).root;
+    common_pointer_head_corruption::unit_bytes_at(
+        image,
+        common_pointer_head_corruption::the_slot_of(&root_pointer),
+        common_pointer_head_corruption::node_bytes(),
+    )
+}
+
+/// 一份副本陈旧：盘 `device` 上孩子的槽起写进 `stale_bytes`（别处一个单元的原样字节，不补任何校验和），另一块盘那一份不动。
+/// 父条目位置条目里的整单元校验和还是孩子原来那一份的：陈旧那一份按哪一类的长度都自证不过。
+fn make_the_copy_on_one_device_stale(
+    image: &mut MemoryPool,
+    device: u32,
+    child: &singlefs_core::pointer::NodePointer,
+    stale_bytes: &[u8],
+) {
+    let slot = common_pointer_head_corruption::the_slot_of(child);
+    write(image, device, slot.0 * SLOT, stale_bytes);
+}
+
+/// 违例的那几条，按名字排成集合（比两份镜像红的是不是同一批，不管清单次序）。
+fn violated_invariant_set(
+    verdicts: &[(&'static str, InvariantVerdict)],
+) -> std::collections::BTreeSet<&'static str> {
+    violated_invariants(verdicts).into_iter().collect()
+}
+
+/// 攻方 H1（判决 `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K2 N1 两故障 H1 那一行）的判法：
+/// 条目改坏（类型段与孩子的类不符）之外，孩子在一块盘上那一份陈旧、陈旧那一份头里的类恰好是类型段说的那一类，另一块盘那一份完好。
+/// 两种盘序各判一次：红的恰好是「只有这份陈旧副本」那张镜像红的那几条再加 `entry_invariant`——陈旧副本自己连带的照红，
+/// 条目那一条不随陈旧副本在哪块盘翻面，也不连带 I-7.2（改之前陈旧的在盘 0 时只认第一份的头、不验、不看第二份，
+/// `entry_invariant` 报成立，另多红 I-7.2，层级 2 与 inode 那两形再多 I-3.1、I-3.11、I-5.1）。
+fn assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(
+    clean: &MemoryPool,
+    image_with_the_entry_changed: &MemoryPool,
+    child: &singlefs_core::pointer::NodePointer,
+    stale_bytes: &[u8],
+    entry_invariant: &'static str,
+    check: impl Fn(&MemoryPool) -> Vec<(&'static str, InvariantVerdict)>,
+) {
+    let mut violated_on_each_device = Vec::new();
+    for device in DEVICES {
+        let mut stale_copy_only = clean.clone();
+        make_the_copy_on_one_device_stale(&mut stale_copy_only, device, child, stale_bytes);
+        let violated_by_the_stale_copy_only = violated_invariant_set(&check(&stale_copy_only));
+        assert!(
+            !violated_by_the_stale_copy_only.contains(entry_invariant),
+            "只有盘 {device} 上那份陈旧副本、条目不改：{entry_invariant} 不红（{violated_by_the_stale_copy_only:?}）"
+        );
+        let mut image = image_with_the_entry_changed.clone();
+        make_the_copy_on_one_device_stale(&mut image, device, child, stale_bytes);
+        let verdicts = check(&image);
+        let violated = violated_invariant_set(&verdicts);
+        let mut expected = violated_by_the_stale_copy_only.clone();
+        expected.insert(entry_invariant);
+        assert_eq!(
+            violated, expected,
+            "条目改坏、孩子在盘 {device} 上那份陈旧：红的是只有陈旧副本时那几条再加 {entry_invariant}：{verdicts:?}"
+        );
+        assert!(
+            !violated.contains("I-7.2"),
+            "条目改坏、孩子在盘 {device} 上那份陈旧：不连带 I-7.2：{verdicts:?}"
+        );
+        violated_on_each_device.push(violated);
+    }
+    assert_eq!(
+        violated_on_each_device[0], violated_on_each_device[1],
+        "同一张镜像只差陈旧副本在哪块盘：红的同一批"
+    );
+}
+
+/// H1 的 N4-i 那一形（攻方 `research/prompts/m3-batch24-code-r1-opus-model/expected-summary.txt` 第 391 / 392 行那一对）：三个单元的文件发完意图之后，
+/// 墓碑树根（层级 1）第一条条目类型段 1 → 0、身份三段清零，孩子还是那片码 3 墓碑容器；孩子在一块盘上那一份换成墓碑树根（码 2）的原样字节。
+#[test]
+fn a_tombstone_tree_entry_of_type_zero_over_a_container_with_a_stale_copy_of_the_child_reddens_the_tombstone_entry_invariant_on_either_device(
+) {
+    let clean = common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+        "known-bad-tombstone-entry-type-zero-over-a-container-stale-copy",
+    );
+    let child = child_pointer_of_an_entry_of_the_root_of_kind(&clean, TREE_KIND_TOMBSTONE, 0);
+    let stale_bytes = root_node_bytes_of_the_tree_of_kind(&clean, TREE_KIND_TOMBSTONE);
+    assert_eq!(
+        stale_bytes[UNIT_HEADER_CLASS_TAG_OFFSET], UNIT_CLASS_INDEX_NODE,
+        "陈旧副本头里的类就是类型段 0 说的码 2"
+    );
+    let mut image_with_the_entry_changed = clean.clone();
+    change_an_entry_of_the_root_of_kind(
+        &mut image_with_the_entry_changed,
+        TREE_KIND_TOMBSTONE,
+        0,
+        |entry| {
+            change_the_entry_type(
+                entry,
+                PACKED_RECORD_TYPE_TOMBSTONE,
+                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+            );
+            zero_the_three_identity_fields(entry);
+        },
+    );
+    assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(
+        &clean,
+        &image_with_the_entry_changed,
+        &child,
+        &stale_bytes,
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        |image| check_pool_image(image),
+    );
+}
+
+/// H1 的 N4-f 那一形（攻方 T2）：墓碑树长到层级 2 之后，根第一条条目类型段 0 → 1，孩子还是层级 1 的码 2 节点；
+/// 孩子在一块盘上那一份换成一片墓碑容器的前 16 KiB（头里类标签 3）。带旋钮值的入口判。
+#[test]
+fn a_tombstone_tree_entry_of_the_container_type_over_a_node_with_a_stale_copy_of_the_child_reddens_the_tombstone_entry_invariant_on_either_device(
+) {
+    let clean = image_with_a_tombstone_tree_of_level_two(
+        "known-bad-tombstone-entry-container-type-over-a-node-stale-copy",
+    );
+    let child = child_pointer_of_an_entry_of_the_root_of_kind(&clean, TREE_KIND_TOMBSTONE, 0);
+    let first_entry_of_the_child =
+        common_pointer_head_corruption::first_entry_of_the_node_at(&clean, &child);
+    let container_below_the_child =
+        singlefs_core::pointer::NodePointer::read_from(&mut singlefs_core::bytes::ByteReader::at(
+            &first_entry_of_the_child,
+            CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET,
+        ));
+    let stale_bytes = common_pointer_head_corruption::unit_bytes_at(
+        &clean,
+        common_pointer_head_corruption::the_slot_of(&container_below_the_child),
+        common_pointer_head_corruption::node_bytes(),
+    );
+    assert_eq!(
+        stale_bytes[UNIT_HEADER_CLASS_TAG_OFFSET], UNIT_CLASS_PACKED_RECORD_UNIT,
+        "陈旧副本头里的类就是类型段 1 说的码 3"
+    );
+    let mut image_with_the_entry_changed = clean.clone();
+    change_an_entry_of_the_root_of_kind(
+        &mut image_with_the_entry_changed,
+        TREE_KIND_TOMBSTONE,
+        0,
+        |entry| {
+            change_the_entry_type(
+                entry,
+                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+                PACKED_RECORD_TYPE_TOMBSTONE,
+            );
+        },
+    );
+    assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(
+        &clean,
+        &image_with_the_entry_changed,
+        &child,
+        &stale_bytes,
+        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        check_pool_image_under_the_tombstone_knob,
+    );
+}
+
+/// H1 谱系树那一侧（I-9.17 与 I-9.23 同形）：回退到 A 之后，谱系树根第一条条目类型段 5 → 0、身份三段清零，孩子还是那片码 3 谱系容器；
+/// 孩子在一块盘上那一份换成谱系树根（码 2）的原样字节。
+#[test]
+fn a_lineage_tree_entry_of_type_zero_over_a_container_with_a_stale_copy_of_the_child_reddens_the_lineage_entry_invariant_on_either_device(
+) {
+    let clean = image_after_rolling_back_to_the_first_root(
+        "known-bad-lineage-entry-type-zero-over-a-container-stale-copy",
+    )
+    .image;
+    let child = child_pointer_of_an_entry_of_the_root_of_kind(&clean, TREE_KIND_LINEAGE, 0);
+    let stale_bytes = root_node_bytes_of_the_tree_of_kind(&clean, TREE_KIND_LINEAGE);
+    assert_eq!(
+        stale_bytes[UNIT_HEADER_CLASS_TAG_OFFSET], UNIT_CLASS_INDEX_NODE,
+        "陈旧副本头里的类就是类型段 0 说的码 2"
+    );
+    let mut image_with_the_entry_changed = clean.clone();
+    change_an_entry_of_the_root_of_kind(
+        &mut image_with_the_entry_changed,
+        TREE_KIND_LINEAGE,
+        0,
+        |entry| {
+            change_the_entry_type(
+                entry,
+                PACKED_RECORD_TYPE_LINEAGE,
+                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+            );
+            zero_the_three_identity_fields(entry);
+        },
+    );
+    assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(
+        &clean,
+        &image_with_the_entry_changed,
+        &child,
+        &stale_bytes,
+        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+        |image| check_pool_image(image),
+    );
+}
+
+/// H1 的 inode 那一形（攻方 I0 type0-zero-identity）：新池新建文件那一版，inode 树根（层级 1）第一条条目类型段 2 → 0、身份三段清零，
+/// 孩子还是那片码 3 叶容器；孩子在一块盘上那一份换成 inode 树根（码 2）的原样字节。
+#[test]
+fn an_inode_tree_entry_of_type_zero_over_a_leaf_container_with_a_stale_copy_of_the_child_reddens_the_inode_entry_invariant_on_either_device(
+) {
+    let clean = build_pool("known-bad-inode-entry-type-zero-over-a-leaf-stale-copy").memory_pool();
+    let child = child_pointer_of_an_entry_of_the_root_of_kind(&clean, TREE_KIND_INODE, 0);
+    let stale_bytes = root_node_bytes_of_the_tree_of_kind(&clean, TREE_KIND_INODE);
+    assert_eq!(
+        stale_bytes[UNIT_HEADER_CLASS_TAG_OFFSET], UNIT_CLASS_INDEX_NODE,
+        "陈旧副本头里的类就是类型段 0 说的码 2"
+    );
+    let mut image_with_the_entry_changed = clean.clone();
+    change_an_entry_of_the_root_of_kind(
+        &mut image_with_the_entry_changed,
+        TREE_KIND_INODE,
+        0,
+        |entry| {
+            change_the_entry_type(
+                entry,
+                PACKED_RECORD_TYPE_INODE,
+                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+            );
+            zero_the_three_identity_fields(entry);
+        },
+    );
+    assert_only_the_entry_invariant_adds_to_the_stale_copy_on_either_device(
+        &clean,
+        &image_with_the_entry_changed,
+        &child,
+        &stale_bytes,
+        "I-9.2",
+        |image| check_pool_image(image),
+    );
+}
+
+/// 不在登记表里的类标签：孩子头里写成它之后，按哪一类的长度都自证不过，两份都读不出。
+const UNIT_CLASS_NOT_REGISTERED: u8 = 9;
+
+/// 新池新建文件那一版：inode 叶容器头里的类标签 3 → 9（不重封，整单元校验和沿引用链补上），再照 `change` 改 inode 树根第一条条目、重封到根。
+/// 孩子按自己的类标签认不出、按类型段的长度读不出：读不出归 I-2.1 与走读失败。
+fn image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
+    tag: &str,
+    change: impl FnOnce(&mut [u8]),
+) -> MemoryPool {
+    let mut image = build_pool(tag).memory_pool();
+    let child = child_pointer_of_an_entry_of_the_root_of_kind(&image, TREE_KIND_INODE, 0);
+    assert_eq!(
+        common_pointer_head_corruption::the_slot_of(&child).0,
+        INODE_LEAF,
+        "inode 树根第一条条目指着那片叶容器"
+    );
+    mutate_unit(
+        &mut image,
+        INODE_LEAF,
+        |bytes| bytes[UNIT_HEADER_CLASS_TAG_OFFSET] = UNIT_CLASS_NOT_REGISTERED,
+        false,
+    );
+    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, change);
+    image
+}
+
+/// inode 树读不出孩子时 I-9.2 的判法与墓碑树、谱系树对齐（判决 `research/prompts/m3-batch24-code-r1-main-verification.md` 第二节 K1 N1 那一行，
+/// 正推 144 格矩阵里 inode 树不判的那 9 格）：inode 树根第一条条目类型段 2 → 0、身份三段不动（容器号、出生代不为 0），孩子读不出。
+/// 「子单元是码 2」那一小句孩子读不出不判，身份三段不为 0 照判：I-9.2 红（改之前读不出就记走读失败、跳过这条条目，I-9.2 一判都没有）。
+#[test]
+fn an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
+) {
+    let image = image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
+        "known-bad-inode-entry-type-zero-identity-unreadable-child",
+        |entry| {
+            change_the_entry_type(
+                entry,
+                PACKED_RECORD_TYPE_INODE,
+                CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+            );
+        },
+    );
+    assert!(
+        matches!(
+            verdict(&image, "I-9.2"),
+            InvariantVerdict::Violated(detail)
+                if detail.contains("孩子两份都读不出") && detail.contains("（要三段为 0）")
+        ),
+        "类型段 0 的条目身份三段不为 0、孩子读不出：I-9.2 照判身份红：{:?}",
+        verdict(&image, "I-9.2")
+    );
+}
+
+/// 同上一条，条目的类型段 2 → 7（不在 {0, 2} 里）：孩子读不出也判该条目损坏，I-9.2 红（与墓碑树、谱系树类型段不认得那一支同判）。
+#[test]
+fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant(
+) {
+    let image = image_whose_inode_root_entry_names_a_child_that_does_not_read_back(
+        "known-bad-inode-entry-unregistered-type-unreadable-child",
+        |entry| {
+            change_the_entry_type(
+                entry,
+                PACKED_RECORD_TYPE_INODE,
+                CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
+            );
+        },
+    );
+    assert!(
+        matches!(
+            verdict(&image, "I-9.2"),
+            InvariantVerdict::Violated(detail)
+                if detail.contains("类型段 7 不在 {0, 2} 里（孩子两份都读不出）")
+        ),
+        "类型段不认得、孩子读不出：I-9.2 照判该条目损坏：{:?}",
+        verdict(&image, "I-9.2")
+    );
+}
````

## 实现员 h：mutations-append.tsv（/tmp/claude-1000/m3-batch26/h/patch/mutations-append.tsv）

````text
第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（墓碑树 N4-i 加陈旧副本，I-9.23 随陈旧副本在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	        if !verifies_at_the_length_of_its_class {\n            continue;\n        }	        if !verifies_at_the_length_of_its_class\n            && Some(class_in_the_header) != class_the_entry_type_names\n        {\n            continue;\n        }	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	a_tombstone_tree_entry_of_type_zero_over_a_container_with_a_stale_copy_of_the_child_reddens_the_tombstone_entry_invariant_on_either_device
第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（层级 2 墓碑树 N4-f 加陈旧副本，I-9.23 报成立、连带 I-3.1、I-3.11、I-5.1、I-7.2）	crates/singlefs-checker/src/walk.rs	        if !verifies_at_the_length_of_its_class {\n            continue;\n        }	        if !verifies_at_the_length_of_its_class\n            && Some(class_in_the_header) != class_the_entry_type_names\n        {\n            continue;\n        }	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	a_tombstone_tree_entry_of_the_container_type_over_a_node_with_a_stale_copy_of_the_child_reddens_the_tombstone_entry_invariant_on_either_device
第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（谱系树类型段 0 加陈旧副本，I-9.17 随陈旧副本在哪块盘翻面）	crates/singlefs-checker/src/walk.rs	        if !verifies_at_the_length_of_its_class {\n            continue;\n        }	        if !verifies_at_the_length_of_its_class\n            && Some(class_in_the_header) != class_the_entry_type_names\n        {\n            continue;\n        }	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	a_lineage_tree_entry_of_type_zero_over_a_container_with_a_stale_copy_of_the_child_reddens_the_lineage_entry_invariant_on_either_device
第26批 H1 头里的类就是类型段那一类的副本不验整单元校验和（inode 树类型段 0 加陈旧副本，I-9.2 报成立、连带 I-7.2 等）	crates/singlefs-checker/src/walk.rs	        if !verifies_at_the_length_of_its_class {\n            continue;\n        }	        if !verifies_at_the_length_of_its_class\n            && Some(class_in_the_header) != class_the_entry_type_names\n        {\n            continue;\n        }	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	an_inode_tree_entry_of_type_zero_over_a_leaf_container_with_a_stale_copy_of_the_child_reddens_the_inode_entry_invariant_on_either_device
第26批 inode 树读不出孩子时 I-9.2 不判（类型段 0 身份三段不为 0的条目只记 I-2.1 与走读失败）	crates/singlefs-checker/src/walk.rs	                judge_an_inode_entry_whose_child_does_not_read_back(\n                    &mut self.judgements,\n                    entry_type,\n                    identity,\n                );	                // 变异：inode 树读不出孩子时 I-9.2 不判	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	an_inode_tree_entry_of_type_zero_with_an_identity_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant
第26批 inode 树读不出孩子时 I-9.2 不判（类型段不认得的条目只记 I-2.1 与走读失败）	crates/singlefs-checker/src/walk.rs	                judge_an_inode_entry_whose_child_does_not_read_back(\n                    &mut self.judgements,\n                    entry_type,\n                    identity,\n                );	                // 变异：inode 树读不出孩子时 I-9.2 不判	-p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back	an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read_back_reddens_the_inode_entry_invariant
第26批 GPU 判器 H1：第一份副本头里的类就是类型段那一类时不验整单元校验和（inode 树根条目类型段 0、叶容器盘 0 那一份陈旧那条流上 CPU 红 I-9.2，GPU 不判、记走读失败）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        if (w_unit_verifies_at_the_length_of_its_class(first_copy, class_in_the_first_header, child.sum0)) {	        if (class_in_the_first_header == class_the_entry_type_names || w_unit_verifies_at_the_length_of_its_class(first_copy, class_in_the_first_header, child.sum0)) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_without_an_identity_names_a_leaf_container_whose_first_copy_is_stale	the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_without_an_identity_names_a_leaf_container_whose_first_copy_is_stale
````
