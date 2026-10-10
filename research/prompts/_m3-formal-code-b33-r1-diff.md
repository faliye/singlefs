# 第 32–33 批代码轮第一轮：改动全文（2026-10-05）

五份补丁照实现员交回的原样，按合入次序（32-1、32-2、33-a 与 33-b 同一趟、33-c）；底座各是当时的主工作区。另加 E156 装置（`crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs`）从第 31 批冻结副本那一版（sha256 前缀 3438ed96…）到今天（3129ec95…）的 `diff -u`：E156 第 5 次（W7 换成 PC-钉门槛、内联单测装根环表之后的改动）与第 6 次（门槛照定乙并进全零槽、U20）两次执行员的改动，变异行 M35、M44（第 5 次）与 M45–M47（第 6 次）原样在 `crates/mutations.tsv`。

## m3-formal-32-1

32-1：B3、B4、F1、F2（第 30 批判决第三节第 1 条）

### crates.patch

````diff
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index b356528..6266361 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -11,8 +11,8 @@ use singlefs_format::{
 };
 
 use crate::{
-    check_root_slot, check_system_configuration_slot, read_six_byte_unsigned, read_u32, read_u64,
-    RootView, SystemConfigurationView,
+    check_root_slot, check_system_configuration_slot, read_six_byte_unsigned, read_u16, read_u32,
+    read_u64, RootView, SystemConfigurationView,
 };
 
 /// checker 读镜像的口子。它不依赖实现的块设备抽象：harness 的崩溃镜像、宿主上的盘镜像各自接一份。
@@ -1251,6 +1251,9 @@ pub struct PointerView {
     /// 加密关着时它们恒 0，读者遇到非 0 判该指针所在的结构损坏（已定项 3 射程）；池级走读在每条跟随的指针上按 I-2.4 判
     /// （[`judge_pointer_mac_and_nonce_are_zero`]）。
     pub is_mac_and_nonce_all_zero: bool,
+    /// 头部 MAC / nonce 之后、extent 偏移之前的加密与压缩那几段（偏移 28..32）读到的原值。第一版的取值都是 0，池级走读在每条跟随的指针上
+    /// 按 I-2.4 判（[`judge_pointer_encryption_and_compression_fields_take_the_first_version_values`]）。
+    pub encryption_and_compression_fields: PointerHeadEncryptionAndCompressionFields,
     /// 头部 extent 偏移那 2 字节（偏移 32，D19（块指针的结构与宽度预算） 已定项 11 的偏移表）是 0：第一版指针指整个单元，恒 0，
     /// 读者遇到非 0 判该指针所在的结构损坏，与 MAC / nonce 同一处置、与加密开不开无关（已定项 3；`m3-inv-a-r1` 判决第二节第 11 行）。
     pub is_extent_offset_zero: bool,
@@ -1259,6 +1262,33 @@ pub struct PointerView {
 /// 指针头部 MAC 16 + nonce 12：从偏移 0 起连着的 28 字节（D19（块指针的结构与宽度预算） 已定项 11）。
 const POINTER_MAC_AND_NONCE_BYTES: usize = 16 + 12;
 
+/// 指针头部里 MAC / nonce 之后的加密与压缩那几段（D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11 的字段表：算法类型 1 在偏移 28、
+/// 压缩算法码 1 在 29、压后长度 2 在 30），读到的原值。与 core `PointerHead::read_judging_the_encryption_compression_and_extent_offset_fields_from`
+/// 判的是同一组字段，checker 自己按偏移表读一份。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct PointerHeadEncryptionAndCompressionFields {
+    pub algorithm_type: u8,
+    pub compression_algorithm_code: u8,
+    pub compressed_length: u16,
+}
+
+/// 指针头部算法类型 1 字节的位置（MAC 16 + nonce 12 之后）。
+const POINTER_ALGORITHM_TYPE_OFFSET: usize = 28;
+/// 指针头部压缩算法码 1 字节的位置（算法类型之后）。
+const POINTER_COMPRESSION_ALGORITHM_CODE_OFFSET: usize = 29;
+/// 指针头部压后长度 2 字节的位置（压缩算法码之后）。
+const POINTER_COMPRESSED_LENGTH_OFFSET: usize = 30;
+
+fn encryption_and_compression_fields_of(
+    pointer_bytes: &[u8],
+) -> PointerHeadEncryptionAndCompressionFields {
+    PointerHeadEncryptionAndCompressionFields {
+        algorithm_type: pointer_bytes[POINTER_ALGORITHM_TYPE_OFFSET],
+        compression_algorithm_code: pointer_bytes[POINTER_COMPRESSION_ALGORITHM_CODE_OFFSET],
+        compressed_length: read_u16(pointer_bytes, POINTER_COMPRESSED_LENGTH_OFFSET),
+    }
+}
+
 /// 指针头部 extent 偏移 2 字节的位置：MAC 16 + nonce 12 + 算法类型等 4 之后（D19（块指针的结构与宽度预算） 已定项 11）。
 const POINTER_EXTENT_OFFSET_FIELD: std::ops::Range<usize> = 32..34;
 
@@ -1285,6 +1315,7 @@ pub fn parse_node_pointer(bytes: &[u8]) -> PointerView {
         tail: u64::from(read_u32(bytes, 82)),
         all_zero: bytes[..86].iter().all(|byte| *byte == 0),
         is_mac_and_nonce_all_zero: mac_and_nonce_are_all_zero(bytes),
+        encryption_and_compression_fields: encryption_and_compression_fields_of(bytes),
         is_extent_offset_zero: extent_offset_is_zero(bytes),
     }
 }
@@ -1300,6 +1331,7 @@ pub fn parse_data_pointer(bytes: &[u8]) -> PointerView {
         tail: read_six_byte_unsigned(bytes, 82),
         all_zero: bytes[..88].iter().all(|byte| *byte == 0),
         is_mac_and_nonce_all_zero: mac_and_nonce_are_all_zero(bytes),
+        encryption_and_compression_fields: encryption_and_compression_fields_of(bytes),
         is_extent_offset_zero: extent_offset_is_zero(bytes),
     }
 }
@@ -1318,6 +1350,37 @@ pub fn judge_pointer_mac_and_nonce_are_zero(
     });
 }
 
+/// I-2.4（头校验和覆盖范围） 射程里指针那一半的加密与压缩那几段：走读跟随的一条指针，头部算法类型是 0（不加密）、压缩算法码是 0（不压缩）、
+/// 压后长度是 0（码 0 时恒 0）。第一版不支持加密（D9（加密） 已定项 10），压缩算法码登记表第一版只有码 0（D9（加密） 已定项 14），
+/// 读者遇到别的取值判该指针所在的结构损坏，与 MAC / nonce 同一处置（D19（块指针的结构与宽度预算） 已定项 3；实现那一侧是
+/// `crates/singlefs-core/src/pointer.rs` 的 `PointerHeadFieldOutsideTheFirstVersion` 的算法类型、压缩算法码、压后长度三个成员；
+/// 判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2 那一行的改法 F2）。三段各判一格，违例那一句写清是哪一段。只判不断。
+pub fn judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
+    judgements: &mut Judgements,
+    pointer: &PointerView,
+    what: &str,
+) {
+    let fields = pointer.encryption_and_compression_fields;
+    judgements.judge("I-2.4", fields.algorithm_type == 0, || {
+        format!(
+            "{what}：指针头部算法类型是 {}，第一版不加密、这 1 字节恒 0",
+            fields.algorithm_type
+        )
+    });
+    judgements.judge("I-2.4", fields.compression_algorithm_code == 0, || {
+        format!(
+            "{what}：指针头部压缩算法码是 {}，第一版登记表只有码 0（不压缩）",
+            fields.compression_algorithm_code
+        )
+    });
+    judgements.judge("I-2.4", fields.compressed_length == 0, || {
+        format!(
+            "{what}：指针头部压后长度是 {}，压缩算法码 0 时这 2 字节恒 0",
+            fields.compressed_length
+        )
+    });
+}
+
 /// 根记录里算法类型 1 + nonce 12 + MAC 16 那 29 字节（D22（单元原子性怎么合成） 已定项 7 的字段表：偏移 428、429、441，合计到 457）。
 const ROOT_RECORD_ALGORITHM_NONCE_AND_MAC: std::ops::Range<usize> = 428..457;
 
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index d2af769..10c8e7a 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -27,8 +27,10 @@ use crate::position_addressed::{
 use crate::image::{
     chosen_system_configurations, decision_width_declared_by_the_chosen_system_configurations,
     judge_journal_ring_geometry_fields_agree, judge_location_entries_order, judge_location_order,
-    judge_own_device_number_is_the_device_identity, judge_pointer_extent_offset_is_zero,
-    judge_pointer_mac_and_nonce_are_zero, judge_root_record_algorithm_nonce_and_mac_are_zero,
+    judge_own_device_number_is_the_device_identity,
+    judge_pointer_encryption_and_compression_fields_take_the_first_version_values,
+    judge_pointer_extent_offset_is_zero, judge_pointer_mac_and_nonce_are_zero,
+    judge_root_record_algorithm_nonce_and_mac_are_zero,
     judge_root_records_live_in_the_slot_their_txg_names,
     judge_root_slots_aligned_to_the_decision_width,
     judge_some_device_carries_a_self_verified_system_configuration,
@@ -1268,6 +1270,11 @@ impl<'reader> Walk<'reader> {
         }
         judge_location_order(&mut self.judgements, &pointer, what);
         judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &pointer, what);
+        judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
+            &mut self.judgements,
+            &pointer,
+            what,
+        );
         judge_pointer_extent_offset_is_zero(&mut self.judgements, &pointer, what);
         self.note_references_of_a_pointer_followed_by_the_walk(&pointer, 1, what);
         let Some(unit) = read_referenced_unit(
@@ -2063,6 +2070,11 @@ impl<'reader> Walk<'reader> {
             judge_location_order(&mut self.judgements, &child, &what);
             judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &child, &what);
             judge_pointer_extent_offset_is_zero(&mut self.judgements, &child, &what);
+            judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
+                &mut self.judgements,
+                &child,
+                &what,
+            );
             if entry_type == 0 {
                 if node.level <= 1 {
                     self.walk_failures.push(format!(
@@ -2226,6 +2238,11 @@ impl<'reader> Walk<'reader> {
                 &child,
                 "inode 内部条目的子指针",
             );
+            judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
+                &mut self.judgements,
+                &child,
+                "inode 内部条目的子指针",
+            );
             judge_pointer_extent_offset_is_zero(
                 &mut self.judgements,
                 &child,
@@ -2567,6 +2584,11 @@ impl<'reader> Walk<'reader> {
             };
             judge_location_order(&mut self.judgements, &pointer, &pointer_what);
             judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &pointer, &pointer_what);
+            judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
+                &mut self.judgements,
+                &pointer,
+                &pointer_what,
+            );
             judge_pointer_extent_offset_is_zero(&mut self.judgements, &pointer, &pointer_what);
             self.note_references_of_a_pointer_followed_by_the_walk(&pointer, 2, &what);
             let Some(unit) = read_referenced_unit(
@@ -2728,6 +2750,11 @@ impl<'reader> Walk<'reader> {
             &pointer,
             "extent 记录的数据指针",
         );
+        judge_pointer_encryption_and_compression_fields_take_the_first_version_values(
+            &mut self.judgements,
+            &pointer,
+            "extent 记录的数据指针",
+        );
         judge_pointer_extent_offset_is_zero(
             &mut self.judgements,
             &pointer,
@@ -4882,7 +4909,10 @@ fn judge_release_generation_and_tree_table_birth(
 /// （偏移 342）必须全零——那一版的账住树表种类 3 那一条，根记录这一项只有树表 0 条、写过行的那一版写非零
 /// （D22（单元原子性怎么合成） 已定项 7 字段表「分配记录树根指针」那一行；判决 `research/prompts/m3-formal-c655-c687-r1-main-verification.md`
 /// 第二节 Y1 `allocation_record_tree_root` 那一行）。非零时这一版有两棵分配记录树，「这条根的分配记录树」不止一棵。
-/// 树表读不出的根这一格不判（认不出带不带树）。不变量清单里没有一条专管根记录这一项，暂挂在 I-5.4 下（归属交主 agent 定）。
+/// 不变量清单里没有一条专管根记录这一项，暂挂在 I-5.4 下（归属交主 agent 定）。
+/// 树表指针非零而树表读不出的根，这一格判不了（认不出带不带树），它那一版自己的分配记录树（树表种类 3 那一条指着的那一棵）也走不到：
+/// 不计成立，I-5.4 报部分没判、逐版列出（判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2
+/// 「I-5.4 按版本：坏盘」那一行的改法 F1：此前这种根整版不进评估，别的版本的树走到了叶就报成立）。
 /// 这一格成立的那一次评估只在**这一版自己的**分配记录树（树表种类 3 那一条指着的那一棵）走到叶时计进 I-5.4（判决
 /// `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-5.4 挂法（C2）」那一行，按版本那一层是
 /// `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2 的零轮收严）：没走到的那几版不计，I-5.4 报部分没判、逐版列出没计的那几版
@@ -4905,6 +4935,8 @@ fn judge_allocation_records_disjoint(
     // 按版本记，别的版本的树走到了叶不替这一版作保）。
     let mut versions_where_the_root_record_allocation_tree_root_is_zero: Vec<(String, bool)> =
         Vec::new();
+    // 树表指针非零而树表读不出的那几版：认不出带不带树，「根记录里分配记录树根全零」那一格与这一版自己的分配记录树都没判。
+    let mut versions_whose_tree_table_does_not_read_back: Vec<String> = Vec::new();
     for index in candidate_indexes.iter().copied() {
         let (_, _, root) = &roots[index];
         let mut this_version_carries_trees_and_a_zero_root_record_allocation_tree_root = false;
@@ -4915,9 +4947,15 @@ fn judge_allocation_records_disjoint(
         )];
         let tree_table_pointer = parse_node_pointer(&root.record_bytes[36..122]);
         if !tree_table_pointer.all_zero {
-            if let Some(tree_table) =
-                read_index_node_without_judging(reader, &tree_table_pointer, cache)
-            {
+            let tree_table_read_back =
+                read_index_node_without_judging(reader, &tree_table_pointer, cache);
+            if tree_table_read_back.is_none() {
+                versions_whose_tree_table_does_not_read_back.push(format!(
+                    "根（实例 {}、txg {}）",
+                    root.instance, root.checkpoint_txg
+                ));
+            }
+            if let Some(tree_table) = tree_table_read_back {
                 if !tree_table.entries.is_empty() {
                     if allocation_roots[0].all_zero {
                         this_version_carries_trees_and_a_zero_root_record_allocation_tree_root =
@@ -4996,8 +5034,10 @@ fn judge_allocation_records_disjoint(
             "候选集里没有一条根的分配记录树走得到叶：第 0 代树表是空的，或树表 / 分配记录树读不出、位置对不上",
         );
     }
+    // `partially_judged` 只留第一次给的说明：两种没判到的版本合成一句报。
+    let mut unjudged_parts: Vec<String> = Vec::new();
     if !versions_not_counted.is_empty() {
-        let unjudged = if some_allocation_record_tree_walked_to_its_leaves {
+        unjudged_parts.push(if some_allocation_record_tree_walked_to_its_leaves {
             format!(
                 "带树一版根记录里分配记录树根全零那一格成立而没计（那一版自己的分配记录树读不出、走不到叶，别的版本的树走到了叶不替它作保）：{}",
                 versions_not_counted.join("、")
@@ -5007,8 +5047,16 @@ fn judge_allocation_records_disjoint(
                 "候选集里没有一条根的分配记录树走得到叶，记录之间互不相交一格都没判；带树一版根记录里分配记录树根全零那一格成立而没计：{}",
                 versions_not_counted.join("、")
             )
-        };
-        judgements.partially_judged("I-5.4", unjudged);
+        });
+    }
+    if !versions_whose_tree_table_does_not_read_back.is_empty() {
+        unjudged_parts.push(format!(
+            "树表指针非零而树表读不出的版本，认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：{}",
+            versions_whose_tree_table_does_not_read_back.join("、")
+        ));
+    }
+    if !unjudged_parts.is_empty() {
+        judgements.partially_judged("I-5.4", unjudged_parts.join("；"));
     }
 }
 
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index e3f8dab..ed91749 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -1313,9 +1313,12 @@ fn w_judge_location_order(p: Ptr) {
     w_judge(INVARIANT_I_2_5, p.device0 < p.device1);
 }
 
-// I-2.4 的指针那一半：头部 MAC / nonce 全 0
+// I-2.4 的指针那一半：头部 MAC / nonce 全 0（image::judge_pointer_mac_and_nonce_are_zero），以及算法类型、压缩算法码、压后长度
+// 是第一版的取值 0（image::judge_pointer_encryption_and_compression_fields_take_the_first_version_values；head_ok 是
+// crash_judge_common.wgsl 的 pointer_encryption_and_compression_fields_are_the_first_version，连 MAC / nonce 一并判）
 fn w_judge_pointer_head(p: Ptr) {
     w_judge(INVARIANT_I_2_4, p.mac_nonce_zero);
+    w_judge(INVARIANT_I_2_4, p.head_ok);
 }
 
 // I-2.4 的指针那一半另一格：头部 extent 偏移为 0（image::judge_pointer_extent_offset_is_zero；第一版指针指整个单元，与加密开不开无关）
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 050b98b..a9aefa6 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -15,7 +15,8 @@
 //! I-9.16 ③ ④（种类重复、缺 deadlist）、根槽 428..457（I-2.4）、带树一版根记录里的分配记录树根（I-5.4）各一条坏流整份比；指针头部的
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
 //! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）；这几处除实例表指针外
-//! 另各有一条 MAC 第 0 字节非 0 的坏流（core 连加密、压缩那几段一并判，GPU 恢复内核同判）。墓碑树与谱系树里指向容器的子指针 extent 偏移各一条
+//! 另各有一条 MAC 第 0 字节非 0 的坏流（core 连加密、压缩那几段一并判，GPU 恢复内核同判）；extent 上段叶内联的数据指针另有一条算法类型写 1 的坏流
+//! （池级 checker 两个走读内核判算法类型、压缩算法码、压后长度那几段）。墓碑树与谱系树里指向容器的子指针 extent 偏移各一条
 //! （墓碑树那条在删除中间态的流上），树表里两条种类 40（≥ 32）一条，都整份比。
 //! 要本机一张空闲显存够的 NVIDIA 独显：本机一张 NVIDIA 独显适配器都没有时用卡的用例打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、
 //! 返回成功；有卡而用不上（一张都没起来，或 PCI 上有卡而 Vulkan 枚举不出）判红（`gpu_unit_checks::gpu_test_start`）。要 `verdict-store,gpu` 特性；
@@ -1623,6 +1624,8 @@ const NEW_ROOT_SEGMENT_UNMOUNT_MARKER: usize = 360;
 const POINTER_LOCATION_CHECKSUM_OFFSETS: [usize; 2] = [60, 74];
 /// 指针头里 extent 偏移 2 字节的偏移（MAC / nonce 28、算法类型 1、压缩码 1、压缩长度 2 之后）。
 const POINTER_EXTENT_OFFSET_OFFSET: usize = 32;
+/// 指针头里算法类型 1 字节的偏移（MAC / nonce 28 之后）。
+const POINTER_ALGORITHM_TYPE_OFFSET: usize = 28;
 /// 码 2 / 码 3 指针宽。
 const NODE_POINTER_BYTES: usize = 86;
 /// 数据单元头里诞生代号的偏移（码 1 写序五元组：树 43、inode 51、对象出生代 59、偏移 67、诞生代号 75）。
@@ -4209,6 +4212,9 @@ fn record_new_pool_with_a_non_first_version_head_on_the_inode_tree_root_pointer(
         PointerHeadFieldWrittenNonZero::FirstMacByte => {
             "new-pool-inode-tree-root-pointer-first-mac-byte-one"
         }
+        PointerHeadFieldWrittenNonZero::AlgorithmType => {
+            "new-pool-inode-tree-root-pointer-algorithm-type-one"
+        }
     };
     record_new_pool_with_a_resealed_field(tag, name, &is_a_tree_table_unit, false, &|bytes| {
         let entry = only_tree_table_entry_of_kind(bytes, 2);
@@ -4252,12 +4258,15 @@ fn set_the_extent_offset_of_the_pointer_at(bytes: &mut [u8], pointer: usize) {
 }
 
 /// 指针头部里写成非第一版取值的那一段（D19（块指针的结构与宽度预算） 已定项 7 的字段表）：extent 偏移（偏移 32 的 2 字节）写 1，
-/// 或 MAC 第 0 字节写 1（加密关着时 MAC 恒 0，D19 已定项 3）。core 读走读跟随的指针都经
-/// `read_judging_the_encryption_compression_and_extent_offset_fields_from` 两段一并判，GPU 恢复内核两段各一判（第 30 批 30-2 起）。
+/// 或 MAC 第 0 字节写 1（加密关着时 MAC 恒 0，D19 已定项 3），或算法类型（偏移 28 的 1 字节）写 1（第一版不加密，D9（加密） 已定项 10）。
+/// core 读走读跟随的指针都经 `read_judging_the_encryption_compression_and_extent_offset_fields_from` 几段一并判，GPU 恢复内核各段都判
+/// （第 30 批 30-2 起）；池级 checker 的 CPU 与 GPU 走读内核判算法类型那一段从第 32 批起（判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md`
+/// 第二节 M2 改法 F2）。
 #[derive(Clone, Copy, Debug)]
 enum PointerHeadFieldWrittenNonZero {
     ExtentOffset,
     FirstMacByte,
+    AlgorithmType,
 }
 
 /// 偏移 `pointer` 起那条指针头部 `field` 那一段（本来 0）写成非 0。
@@ -4274,6 +4283,14 @@ fn write_the_pointer_head_field_non_zero_at(
             assert_eq!(bytes[pointer], 0, "加密关着时指针头部的 MAC 恒 0");
             bytes[pointer] = 1;
         }
+        PointerHeadFieldWrittenNonZero::AlgorithmType => {
+            let algorithm_type = pointer + POINTER_ALGORITHM_TYPE_OFFSET;
+            assert_eq!(
+                bytes[algorithm_type], 0,
+                "第一版指针头部的算法类型恒 0（不加密）"
+            );
+            bytes[algorithm_type] = 1;
+        }
     }
 }
 
@@ -4308,6 +4325,9 @@ fn record_new_pool_with_a_non_first_version_head_on_an_inode_internal_entry_chil
         PointerHeadFieldWrittenNonZero::FirstMacByte => {
             "new-pool-inode-internal-entry-child-pointer-first-mac-byte-one"
         }
+        PointerHeadFieldWrittenNonZero::AlgorithmType => {
+            "new-pool-inode-internal-entry-child-pointer-algorithm-type-one"
+        }
     };
     let inode_tree_root = |bytes: &[u8]| {
         unit_class(bytes) == 2
@@ -4355,6 +4375,9 @@ fn record_new_pool_with_a_non_first_version_head_on_the_inline_data_pointer(
         PointerHeadFieldWrittenNonZero::FirstMacByte => {
             "new-pool-inline-data-pointer-first-mac-byte-one"
         }
+        PointerHeadFieldWrittenNonZero::AlgorithmType => {
+            "new-pool-inline-data-pointer-algorithm-type-one"
+        }
     };
     record_new_pool_with_a_resealed_field(tag, name, &is_an_extent_upper_leaf, false, &|bytes| {
         let pointer = pointer_of_the_first_extent_upper_leaf_entry(
@@ -4379,6 +4402,9 @@ fn record_new_pool_with_a_non_first_version_head_on_an_allocation_record_tree_ch
         PointerHeadFieldWrittenNonZero::FirstMacByte => {
             "new-pool-allocation-record-tree-child-pointer-first-mac-byte-one"
         }
+        PointerHeadFieldWrittenNonZero::AlgorithmType => {
+            "new-pool-allocation-record-tree-child-pointer-algorithm-type-one"
+        }
     };
     let allocation_internal_node = |bytes: &[u8]| {
         unit_class(bytes) == 2
@@ -4444,6 +4470,9 @@ fn record_three_unit_file_with_a_non_first_version_head_on_the_lower_segment_roo
         PointerHeadFieldWrittenNonZero::FirstMacByte => {
             "three-unit-file-lower-segment-root-pointer-first-mac-byte-one"
         }
+        PointerHeadFieldWrittenNonZero::AlgorithmType => {
+            "three-unit-file-lower-segment-root-pointer-algorithm-type-one"
+        }
     };
     record_three_unit_file_with_a_resealed_field(tag, name, &is_an_extent_upper_leaf, &|bytes| {
         let pointer = pointer_of_the_first_extent_upper_leaf_entry(
@@ -4468,6 +4497,9 @@ fn record_three_unit_file_with_a_non_first_version_head_on_a_lower_leaf_data_poi
         PointerHeadFieldWrittenNonZero::FirstMacByte => {
             "three-unit-file-lower-leaf-data-pointer-first-mac-byte-one"
         }
+        PointerHeadFieldWrittenNonZero::AlgorithmType => {
+            "three-unit-file-lower-leaf-data-pointer-algorithm-type-one"
+        }
     };
     let extent_lower_leaf = |bytes: &[u8]| {
         unit_class(bytes) == 2
@@ -4689,6 +4721,21 @@ fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_segment_root_pointer_c
     );
 }
 
+/// I-2.4：extent 上段叶内联的数据指针的算法类型写 1（core 解上段叶条目时拒、GPU 恢复内核同拒；池级 checker 的 CPU 与 GPU 走读内核
+/// 判算法类型那一段，第 30 批判决第二节 M2 改法 F2）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_names_an_encryption_algorithm(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_names_an_encryption_algorithm",
+        &record_new_pool_with_a_non_first_version_head_on_the_inline_data_pointer(
+            "crash-judge-gpu-i24-inline-data-pointer-algorithm-type",
+            PointerHeadFieldWrittenNonZero::AlgorithmType,
+        ),
+        "I-2.4",
+    );
+}
+
 /// I-2.4：extent 下段叶记录的数据指针的 MAC 第 0 字节非 0。
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_when_an_extent_lower_leaf_data_pointer_carries_a_non_zero_mac_byte(
diff --git a/crates/singlefs-harness/src/model.rs b/crates/singlefs-harness/src/model.rs
index a649d07..ca471f1 100644
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -4287,4 +4287,90 @@ mod tests {
             "panic 的是落点式子那一条 assert!：{message}"
         );
     }
+
+    /// 一版里最左那片之外的 inode 叶容器角色与各自的分配代。
+    fn inode_leaf_containers_right_of_the_leftmost(
+        root: &ModelRoot,
+    ) -> BTreeMap<ModelUnitRole, ModelCheckpointTxg> {
+        root.role_written_at
+            .iter()
+            .filter(|(role, _)| matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_)))
+            .map(|(role, written_at)| (*role, *written_at))
+            .collect()
+    }
+
+    /// 只供测试的一版：照 `base` 抄，inode 树换成 `leaf_containers` 片叶容器（从 1 数，最左那片算 1），最左那片之外的第 k 片记成
+    /// txg `first_written_at + k` 写的（各片分配代互不相同，回退取错一片就看得出来）。
+    fn version_with_inode_leaf_containers(
+        base: &ModelRoot,
+        leaf_containers: u64,
+        first_written_at: ModelCheckpointTxg,
+    ) -> ModelRoot {
+        let mut version = base.clone();
+        version
+            .role_written_at
+            .retain(|role, _| !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_)));
+        for position_from_the_leftmost in 1..leaf_containers {
+            version.role_written_at.insert(
+                ModelUnitRole::inode_leaf_container_at(position_from_the_leftmost),
+                ModelCheckpointTxg(first_written_at.0 + position_from_the_leftmost),
+            );
+        }
+        version
+    }
+
+    /// 模型回退那次发布对最左那片之外的叶容器的处置（`next_root_with_the_user_visible_trees` 取回退目标那一支里丢现行那几片、取目标那几片的两句），
+    /// 结构单测（第 30 批判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M1「第 k 片都对」那一行的改法 B4）：
+    /// 现行与目标各取 1、2、3、4、5、135 片（135 是一个码 2 根装得下的上界，D8（核心索引结构） 已定项 6）的每一种组合，回退写出的那一版
+    /// 最左那片之外的叶容器（角色与分配代）恰好等于目标那一版的——现行有、目标没有的那几片这一版也没有，目标的每一片都取回、分配代取目标的。
+    /// 按取样点逐片往外加历史用例追不完（回退只取回或只丢某几片，只有建到那几片之外的历史抓得到），这一条不跑历史。
+    #[test]
+    fn a_rollback_takes_exactly_the_inode_leaf_containers_right_of_the_leftmost_of_its_target_for_every_pair_of_container_counts(
+    ) {
+        const CONTAINER_COUNTS: [u64; 6] = [1, 2, 3, 4, 5, 135];
+        let model = model_after_four_overwrites_in_a_second_instance();
+        let newest = model.newest_root().clone();
+        let mut pairs_judged = 0;
+        for current_leaf_containers in CONTAINER_COUNTS {
+            for target_leaf_containers in CONTAINER_COUNTS {
+                let target = version_with_inode_leaf_containers(
+                    &newest,
+                    target_leaf_containers,
+                    ModelCheckpointTxg(1000),
+                );
+                let current = version_with_inode_leaf_containers(
+                    &newest,
+                    current_leaf_containers,
+                    ModelCheckpointTxg(2000),
+                );
+                let after_the_rollback = model.next_root_with_the_user_visible_trees(
+                    &current,
+                    current.key.instance,
+                    ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
+                    ModelJournalCounter(model.highest_journal_counter.0 + 1),
+                    ModelPublishKind::EmptyOnFileVersion,
+                    current.rollback_floor,
+                    None,
+                    Rc::clone(&current.instance_table_rows),
+                    UserVisibleTreesOfTheNewVersion::TakenFromTheRollbackTarget(&target),
+                );
+                assert_eq!(
+                    inode_leaf_containers_right_of_the_leftmost(&after_the_rollback),
+                    inode_leaf_containers_right_of_the_leftmost(&target),
+                    "现行 {current_leaf_containers} 片、目标 {target_leaf_containers} 片：回退写出的那一版最左那片之外的叶容器要恰好是目标那一版的"
+                );
+                assert_eq!(
+                    inode_leaf_containers_right_of_the_leftmost(&target).len(),
+                    usize::try_from(target_leaf_containers - 1).expect("片数小于 135"),
+                    "搭建：目标那一版最左那片之外有 {} 片",
+                    target_leaf_containers - 1
+                );
+                pairs_judged += 1;
+            }
+        }
+        assert_eq!(
+            pairs_judged,
+            CONTAINER_COUNTS.len() * CONTAINER_COUNTS.len()
+        );
+    }
 }
diff --git a/crates/singlefs-harness/src/model_comparison.rs b/crates/singlefs-harness/src/model_comparison.rs
index 5e0d252..5b62fb9 100644
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -1412,6 +1412,72 @@ mod tests {
         assert!(counts.allocation_records_compared >= 1, "{counts:?}");
     }
 
+    /// 胶水把实现交回的 inode 叶容器序号映到模型角色那一句，逐片单测（第 30 批判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md`
+    /// 第二节 M1「第 k 片都对」那一行的改法 B3）：一个码 2 根装得下的 135 片（D8（核心索引结构） 已定项 6；实现的
+    /// `leaf_containers_one_root_node_holds`，再多一片实现就拒）逐片映到这一片自己的角色——第 0 片是最左那片，第 k 片是左起第 k 片——
+    /// 而且 135 片映到 135 个互不相同的角色。按取样点逐片往外加历史用例追不完（胶水封顶在第 k 片，只有建到第 k + 1 片的历史抓得到），
+    /// 这一条不跑历史，封顶在任何一片都红。
+    #[test]
+    fn every_inode_leaf_container_one_root_holds_maps_to_its_own_model_role_and_no_two_share_one() {
+        const LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS: usize = 135;
+        assert_eq!(
+            singlefs_core::inode_tree::leaf_containers_one_root_node_holds(),
+            LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS,
+            "实现的码 2 根扇出是 135（D8（核心索引结构） 已定项 6）"
+        );
+        let mut roles_seen = BTreeSet::new();
+        for position_from_the_leftmost in 0..LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS {
+            let role = model_unit_role(TransactionUnit::InodeLeafContainer(
+                singlefs_core::inode_tree::InodeLeafContainerIndexInTree::of_position(
+                    position_from_the_leftmost,
+                ),
+            ))
+            .expect("inode 叶容器每一片都对应一个角色");
+            let position_the_role_names = match role {
+                ModelUnitRole::InodeLeaf => 0,
+                ModelUnitRole::InodeLeafRightOfTheLeftmost(leaf_container) => {
+                    usize::try_from(leaf_container.position_from_the_leftmost())
+                        .expect("位置小于 135")
+                }
+                ModelUnitRole::Data
+                | ModelUnitRole::ExtentRoot
+                | ModelUnitRole::InodeRoot
+                | ModelUnitRole::AllocationTree
+                | ModelUnitRole::AccountingTree
+                | ModelUnitRole::MappingTree
+                | ModelUnitRole::TreeTable
+                | ModelUnitRole::InstanceTable
+                | ModelUnitRole::DeletedInodesRoot
+                | ModelUnitRole::TombstoneLeaf
+                | ModelUnitRole::TombstoneRoot => {
+                    panic!(
+                        "第 {position_from_the_leftmost} 片 inode 叶容器映到了别的角色：{role:?}"
+                    )
+                }
+            };
+            assert_eq!(
+                position_the_role_names, position_from_the_leftmost,
+                "第 {position_from_the_leftmost} 片映到的角色说的是第 {position_the_role_names} 片"
+            );
+            assert_eq!(
+                role,
+                ModelUnitRole::inode_leaf_container_at(
+                    u64::try_from(position_from_the_leftmost).expect("位置小于 135")
+                ),
+                "第 {position_from_the_leftmost} 片映到模型落点函数的第 {position_from_the_leftmost} 片"
+            );
+            assert!(
+                roles_seen.insert(role),
+                "第 {position_from_the_leftmost} 片与前面某一片映到同一个角色：{role:?}"
+            );
+        }
+        assert_eq!(
+            roles_seen.len(),
+            LEAF_CONTAINERS_ONE_CODE_TWO_ROOT_HOLDS,
+            "135 片映到 135 个互不相同的角色"
+        );
+    }
+
     /// 模型模块只用格式常量那一个 crate（D13（验证路线） 已定项 5）：`model.rs` 里注释之外的每一行都不提 `singlefs_core`、`singlefs_checker`，
     /// `use` 只有 `std` 与 `singlefs_format`（测试模块的 `use super::*` 除外）。胶水（这个文件）可以用 core。
     #[test]
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 92be64f..635ba73 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -8527,6 +8527,171 @@ fn an_allocation_record_tree_of_the_newest_version_that_does_not_read_back_leave
     );
 }
 
+/// 覆盖写之后的镜像上把 `slot` 那张树表两份清零：I-5.4 报部分没判，点名树表读不出的那几版（`versions_named`），判红的照旧是读不出的单元那几条
+/// （`violated`）。改之前这几版整版不进评估，别的版本的树走到了叶就报成立（第 30 批攻方 `B30UNIT slot=… I-5.4=Holds` 那三行，
+/// 判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2「I-5.4 按版本：坏盘」那一行的改法 F1）。
+fn assert_a_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
+    tag: &str,
+    slot: u64,
+    violated: &[&str],
+    versions_named: &str,
+) {
+    let clean = image_after_the_overwrite(tag);
+    assert_eq!(
+        verdict(&clean, "I-5.4"),
+        InvariantVerdict::Holds,
+        "覆盖写之后的干净镜像上 I-5.4 成立"
+    );
+    let mut image = clean.clone();
+    zero_both_copies_of_the_unit(
+        &mut image,
+        slot,
+        usize::try_from(NODE_BYTES).expect("16384"),
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        violated,
+        "判红的照改之前那几条：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict(&image, "I-5.4"),
+        InvariantVerdict::PartiallyJudged(format!(
+            "树表指针非零而树表读不出的版本，认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：{versions_named}"
+        )),
+        "树表读不出的那几版：别的版本的树走到了叶不替它们计成立，I-5.4 报部分没判、逐版点名"
+    );
+}
+
+/// I-5.4 按版本，树表读不出（改法 F1）：覆盖写之后把 B（实例 1、txg 4）的树表两份清零。
+#[test]
+fn the_tree_table_of_the_newest_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
+) {
+    assert_a_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
+        "known-bad-disjointness-newest-tree-table-zeroed",
+        TREE_TABLE_AFTER_OVERWRITE,
+        &["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
+        "根（实例 1、txg 4）",
+    );
+}
+
+/// I-5.4 按版本，树表读不出（改法 F1）：覆盖写之后把 A（实例 1、txg 3，新池新建文件那一版）的树表两份清零。
+#[test]
+fn the_tree_table_of_the_previous_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
+) {
+    assert_a_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
+        "known-bad-disjointness-previous-tree-table-zeroed",
+        TREE_TABLE,
+        &["I-2.1", "I-3.1", "I-4.8", "I-7.4"],
+        "根（实例 1、txg 3）",
+    );
+}
+
+/// I-5.4 按版本，树表读不出（改法 F1）：覆盖写之后把第 0 代树表（mkfs 写的、树表 0 条，第 0 代根与暖机那两条根都指着它）两份清零。
+#[test]
+fn the_genesis_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
+) {
+    assert_a_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
+        "known-bad-disjointness-genesis-tree-table-zeroed",
+        GENESIS_TREE_TABLE,
+        &["I-2.1", "I-4.8", "I-7.4"],
+        "根（实例 0、txg 0）、根（实例 1、txg 1）、根（实例 1、txg 2）",
+    );
+}
+
+/// 新池新建文件那一版里一条走读跟随的指针头部 `change` 那一段改成非第一版的取值、重封、校验和沿引用链补到根槽：池级 checker 只红 I-2.4，
+/// 违例那一句是 `expected_violation`（点名是哪条指针、哪一段）。core 读这条指针时拒（判这条指针所在的结构损坏）；改之前池级 checker 只判
+/// MAC / nonce 与 extent 偏移，这三段改了一条都不红（第 30 批攻方 `B30ALG … checker_violated=[]` 那几行，判决
+/// `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2「池级 checker 不判指针头的算法类型、压缩码、压后长度」那一行的改法 F2）。
+fn assert_a_pointer_head_field_outside_the_first_version_reddens_only_the_header_coverage_invariant(
+    image: &MemoryPool,
+    expected_violation: &str,
+) {
+    let verdicts = check_pool_image(image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-2.4"],
+        "只红 I-2.4：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict(image, "I-2.4"),
+        InvariantVerdict::Violated(expected_violation.to_owned()),
+        "I-2.4 的违例那一句写清是哪条指针、哪一段"
+    );
+}
+
+/// 新池新建文件那一版的 extent 上段叶第一条条目内联的数据指针：头部 `change` 那一段改掉。
+fn pool_whose_inline_data_pointer_head_carries(
+    tag: &str,
+    change: common_pointer_head_corruption::PointerHeadFieldChange,
+) -> MemoryPool {
+    let mut image = build_pool(tag).memory_pool();
+    let singlefs_core::extent_tree::ExtentUpperLeafTarget::InlineDataUnit(data_pointer) =
+        common_pointer_head_corruption::first_upper_leaf_entry(&image).target
+    else {
+        panic!("新池新建文件那一版只有一个数据单元，上段叶第一条内联它的数据指针");
+    };
+    let extent_root =
+        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root;
+    common_pointer_head_corruption::change_a_pointer_head_field_inside_the_unit(
+        &mut image,
+        common_pointer_head_corruption::the_slot_of(&extent_root),
+        common_pointer_head_corruption::node_bytes(),
+        &common_pointer_head_corruption::data_pointer_bytes(&data_pointer),
+        change,
+    );
+    image
+}
+
+/// I-2.4 指针头部算法类型（改法 F2）：extent 上段叶内联的数据指针算法类型 0 → 1（登记过的加密算法，第一版不加密，D9（加密） 已定项 10）。
+#[test]
+fn an_inline_data_pointer_naming_an_encryption_algorithm_reddens_only_the_header_coverage_invariant(
+) {
+    let image = pool_whose_inline_data_pointer_head_carries(
+        "known-bad-pointer-head-algorithm-type",
+        common_pointer_head_corruption::PointerHeadFieldChange::AlgorithmTypeToOne,
+    );
+    assert_a_pointer_head_field_outside_the_first_version_reddens_only_the_header_coverage_invariant(
+        &image,
+        "extent 记录的数据指针：指针头部算法类型是 1，第一版不加密、这 1 字节恒 0",
+    );
+}
+
+/// I-2.4 指针头部压缩算法码（改法 F2）：树表里 inode 树那条条目的根指针压缩算法码 0 → 1（登记表第一版只有码 0，D9（加密） 已定项 14）。
+#[test]
+fn a_tree_table_root_pointer_naming_an_unregistered_compression_algorithm_reddens_only_the_header_coverage_invariant(
+) {
+    let mut image = build_pool("known-bad-pointer-head-compression-algorithm-code").memory_pool();
+    let inode_root =
+        common_pointer_head_corruption::tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
+    let tree_table = common_pointer_head_corruption::newest_root(&image).tree_table;
+    common_pointer_head_corruption::change_a_pointer_head_field_inside_the_unit(
+        &mut image,
+        common_pointer_head_corruption::the_slot_of(&tree_table),
+        common_pointer_head_corruption::node_bytes(),
+        &common_pointer_head_corruption::node_pointer_bytes(&inode_root),
+        common_pointer_head_corruption::PointerHeadFieldChange::CompressionAlgorithmCodeToOne,
+    );
+    assert_a_pointer_head_field_outside_the_first_version_reddens_only_the_header_coverage_invariant(
+        &image,
+        "树 65537（种类 2）的根：指针头部压缩算法码是 1，第一版登记表只有码 0（不压缩）",
+    );
+}
+
+/// I-2.4 指针头部压后长度（改法 F2）：extent 上段叶内联的数据指针压缩算法码仍 0、压后长度 0 → 1（码 0 时恒 0，D9（加密） 已定项 14）。
+#[test]
+fn an_inline_data_pointer_carrying_a_compressed_length_without_compression_reddens_only_the_header_coverage_invariant(
+) {
+    let image = pool_whose_inline_data_pointer_head_carries(
+        "known-bad-pointer-head-compressed-length",
+        common_pointer_head_corruption::PointerHeadFieldChange::CompressedLengthToOneWithoutCompression,
+    );
+    assert_a_pointer_head_field_outside_the_first_version_reddens_only_the_header_coverage_invariant(
+        &image,
+        "extent 记录的数据指针：指针头部压后长度是 1，压缩算法码 0 时这 2 字节恒 0",
+    );
+}
+
 /// 树表里种类 `kind` 那棵容器树（墓碑树、谱系树）的码 2 根第 0 条条目里指向容器的子指针（条目里分隔 key 8 + 身份引用 26 之后那 86 字节）：
 /// 头部 extent 偏移 0 → 1，根重封、新的整单元校验和沿引用链补到根槽
 /// （攻方 `research/prompts/m3-formal-code-b29-r1-opus-model/tests/b29_opus_tombstone_container_pointer_head.rs` 的造法）。
````

### mutations-append.tsv

````
第 32 批 32-1 B3（胶水）：inode 叶容器映射封顶在第 4 片（第 30 批攻方 glue_cap_at_fourth，判决 M1 改法 B3）	crates/singlefs-harness/src/model_comparison.rs	            ModelUnitRole::inode_leaf_container_at(u64::from(index.0))	            ModelUnitRole::inode_leaf_container_at(u64::from(index.0).min(3))	-p singlefs-harness --lib --test checker_known_bad_images -- every_inode_leaf_container_one_root_holds a_rollback_takes_exactly_the_inode_leaf_containers the_tree_table_of_the_ the_genesis_tree_table_that_does_not_read_back an_inline_data_pointer_naming_an_encryption_algorithm a_tree_table_root_pointer_naming_an_unregistered_compression_algorithm an_inline_data_pointer_carrying_a_compressed_length	every_inode_leaf_container_one_root_holds_maps_to_its_own_model_role_and_no_two_share_one
第 32 批 32-1 B4（理想模型自身）：回退只取回目标那一版的第 2–3 片叶容器（第 30 批攻方 retain_up_to_third，判决 M1 改法 B4）	crates/singlefs-harness/src/model.rs	                        .filter(|(role, _)| {\n                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                        })	                        .filter(|(role, _)| {\n                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(leaf) if leaf.position_from_the_leftmost() <= 2)\n                        })	-p singlefs-harness --lib --test checker_known_bad_images -- every_inode_leaf_container_one_root_holds a_rollback_takes_exactly_the_inode_leaf_containers the_tree_table_of_the_ the_genesis_tree_table_that_does_not_read_back an_inline_data_pointer_naming_an_encryption_algorithm a_tree_table_root_pointer_naming_an_unregistered_compression_algorithm an_inline_data_pointer_carrying_a_compressed_length	a_rollback_takes_exactly_the_inode_leaf_containers_right_of_the_leftmost_of_its_target_for_every_pair_of_container_counts
第 32 批 32-1 B4（理想模型自身）：回退只丢现行那一版的第 3–4 片叶容器（判决 M1 改法 B4）	crates/singlefs-harness/src/model.rs	                role_written_at.retain(|role, _| {\n                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                });	                role_written_at.retain(|role, _| {\n                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(leaf) if (2..=3).contains(&leaf.position_from_the_leftmost()))\n                });	-p singlefs-harness --lib -- model::tests::a_rollback_takes_exactly_the_inode_leaf_containers	a_rollback_takes_exactly_the_inode_leaf_containers_right_of_the_leftmost_of_its_target_for_every_pair_of_container_counts
第 32 批 32-1 F1 池级 checker：树表读不出的候选根不记进 I-5.4 部分没判（B 的树表清零照旧报成立，判决 M2 改法 F1）	crates/singlefs-checker/src/walk.rs	                versions_whose_tree_table_does_not_read_back.push(format!(	                Vec::<String>::new().push(format!(	-p singlefs-harness --lib --test checker_known_bad_images -- every_inode_leaf_container_one_root_holds a_rollback_takes_exactly_the_inode_leaf_containers the_tree_table_of_the_ the_genesis_tree_table_that_does_not_read_back an_inline_data_pointer_naming_an_encryption_algorithm a_tree_table_root_pointer_naming_an_unregistered_compression_algorithm an_inline_data_pointer_carrying_a_compressed_length	the_tree_table_of_the_newest_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged
第 32 批 32-1 F1 池级 checker：树表读不出的候选根不记进 I-5.4 部分没判（A 的树表清零照旧报成立）	crates/singlefs-checker/src/walk.rs	                versions_whose_tree_table_does_not_read_back.push(format!(	                Vec::<String>::new().push(format!(	-p singlefs-harness --test checker_known_bad_images -- the_tree_table_of_the_previous_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged	the_tree_table_of_the_previous_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged
第 32 批 32-1 F1 池级 checker：树表读不出的候选根不记进 I-5.4 部分没判（第 0 代树表清零照旧报成立）	crates/singlefs-checker/src/walk.rs	                versions_whose_tree_table_does_not_read_back.push(format!(	                Vec::<String>::new().push(format!(	-p singlefs-harness --test checker_known_bad_images -- the_genesis_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged	the_genesis_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged
第 32 批 32-1 F2 池级 checker：指针头部加密 / 压缩那几段一段都不判（读到的原值换成 0，判决 M2 改法 F2）	crates/singlefs-checker/src/image.rs	    let fields = pointer.encryption_and_compression_fields;	    let _ = pointer;\n    let fields = PointerHeadEncryptionAndCompressionFields {\n        algorithm_type: 0,\n        compression_algorithm_code: 0,\n        compressed_length: 0,\n    };	-p singlefs-harness --lib --test checker_known_bad_images -- every_inode_leaf_container_one_root_holds a_rollback_takes_exactly_the_inode_leaf_containers the_tree_table_of_the_ the_genesis_tree_table_that_does_not_read_back an_inline_data_pointer_naming_an_encryption_algorithm a_tree_table_root_pointer_naming_an_unregistered_compression_algorithm an_inline_data_pointer_carrying_a_compressed_length	an_inline_data_pointer_naming_an_encryption_algorithm_reddens_only_the_header_coverage_invariant
第 32 批 32-1 F2 池级 checker：指针头部算法类型放过 1	crates/singlefs-checker/src/image.rs	    judgements.judge("I-2.4", fields.algorithm_type == 0, || {	    judgements.judge("I-2.4", fields.algorithm_type <= 1, || {	-p singlefs-harness --test checker_known_bad_images -- an_inline_data_pointer_naming_an_encryption_algorithm_reddens_only_the_header_coverage_invariant	an_inline_data_pointer_naming_an_encryption_algorithm_reddens_only_the_header_coverage_invariant
第 32 批 32-1 F2 池级 checker：指针头部压缩算法码放过 1	crates/singlefs-checker/src/image.rs	    judgements.judge("I-2.4", fields.compression_algorithm_code == 0, || {	    judgements.judge("I-2.4", fields.compression_algorithm_code <= 1, || {	-p singlefs-harness --test checker_known_bad_images -- a_tree_table_root_pointer_naming_an_unregistered_compression_algorithm_reddens_only_the_header_coverage_invariant	a_tree_table_root_pointer_naming_an_unregistered_compression_algorithm_reddens_only_the_header_coverage_invariant
第 32 批 32-1 F2 池级 checker：码 0 时压后长度放过 1	crates/singlefs-checker/src/image.rs	    judgements.judge("I-2.4", fields.compressed_length == 0, || {	    judgements.judge("I-2.4", fields.compressed_length <= 1, || {	-p singlefs-harness --test checker_known_bad_images -- an_inline_data_pointer_carrying_a_compressed_length_without_compression_reddens_only_the_header_coverage_invariant	an_inline_data_pointer_carrying_a_compressed_length_without_compression_reddens_only_the_header_coverage_invariant
第 32 批 32-1 F2 GPU 判器：走读内核判指针头部只判 MAC / nonce、不判算法类型、压缩算法码、压后长度	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    w_judge(INVARIANT_I_2_4, p.head_ok);	    w_judge(INVARIANT_I_2_4, p.mac_nonce_zero);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_names_an_encryption_algorithm	the_gpu_judge_agrees_with_the_cpu_when_an_inline_extent_data_pointer_names_an_encryption_algorithm
````

## m3-formal-32-2

32-2：第 31 批判决第三节第 1 条五件，加抬 F 慢档用例四支豁免

### crates.patch

````diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index d3fbe6c..7117703 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -1293,7 +1293,9 @@ pub struct PoolAllocator {
     commit_generated_placements_that_fell_back_to_the_lowest_free_slot: u64,
     /// 分配器眼里的根环（`RootRingOccupancy`）：挂载时装（`mount::rebuilt_allocator`），mkfs 同一个进程里由建分配器的一方装
     /// （`make_filesystem::root_ring_occupancy_after_make_filesystem`）。`None`：建分配器的一方没装（`new` 与 `rebuild_from_records`
-    /// 都不装）。没装的分配器不许记根：`record_root_written_by_this_process` 断言（C551，用户 2026-10-05 定），只做不发布的事（分配、准入读数）。
+    /// 都不装）。没装的分配器不许记根：`record_root_written_by_this_process` 断言（C551，用户 2026-10-05 定），只做不发布的事：分配，
+    /// 以及调用方自己给根环每区槽数的准入读数 `admission::admission_reading_before_a_publish_on_a_ring_of`——
+    /// `admission::admission_reading_before_a_publish` 从这张表取每区槽数，没装就断言。
     root_ring: Option<RootRingOccupancy>,
     /// 落盘中途失败、冻结着等原样重发的那一次发布（`transaction::FrozenPublish`，D23（journal 的角色与格式） 已定项 14
     /// 「这一版的失败处置」）。住在分配器上是因为每一次发布都要交分配器进来：冻结着的时候发布路径在任何读写之前拒绝，
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 4efe066..d2fa7be 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -5623,9 +5623,12 @@ fn published_nodes_behind_an_intermediate_row_still_count_against_the_tree_ident
                 span: 1,
             },
         );
-        // 记根要分配器装着根环表（没装就断言，C551）：照可写挂载在实例 1 那一版上读到的样子，环里记实例 1 最新那条根
-        // （实例 2 的根槽清零了，挂载读不到它们），F 0，下一条根就是写行那次。环里最旧有效根是实例 1 那一条，写行那次释放的旧链
-        // 释放代比它新，这一次不回收，账与没装根环表时一样。
+        // 记根要分配器装着根环表（没装就断言，C551）。这张表只记实例 1 最新那一条根，F 0，下一条根就是写行那次：够这次发布记根用，
+        // 不等于可写挂载在实例 1 那一版上读到的环——挂载读到的环里还有 mkfs 写的第 0 代根与实例 1 那次挂载更早发的根
+        // （实例 2 的根槽清零了，那几条读不到），
+        // 这张表比它少几条根，回收门槛因此偏高（环里最旧有效根取成实例 1 最新那一条）。门槛偏高也不多回收，靠的是这个前提：
+        // 这个分配器上没有可回收的已释放记录——它只记着 mkfs 写在单元区里的两个单元、都未释放，写行那次释放的旧链释放代是写行那次的 txg，
+        // 比门槛新。所以这一次一个都不回收，账与没装根环表时一样；往这个分配器上加已释放记录之前，先把表换成挂载读到的环。
         let row_publish_txg =
             CheckpointTxg(first_file_of_the_second_instance.root.checkpoint_txg.0 + 1);
         allocator_rebuilt_on_the_first_instance.install_root_ring_occupancy(
diff --git a/crates/singlefs-harness/tests/overwrite_in_one_instance.rs b/crates/singlefs-harness/tests/overwrite_in_one_instance.rs
index 27dc48f..f9c06a8 100644
--- a/crates/singlefs-harness/tests/overwrite_in_one_instance.rs
+++ b/crates/singlefs-harness/tests/overwrite_in_one_instance.rs
@@ -603,14 +603,13 @@ fn release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not
     );
 }
 
-/// 同一个进程同一个实例里连着覆盖写、一个落点都不回收，分配记录只增不减：攒到多于 812 条（一片叶的条目容量，
-/// 原先「分配记录树只有一个节点」那道墙）之后照样做成——分配记录树按绝对槽号按位置寻址（D8（核心索引结构） 已定项 14，
-/// 用户 2026-09-24 定 K1），每块盘上装着记录的叶不止一片，墙拆了。越过 812 条的那一版之后池级 checker 管按位置寻址的那几条
-/// （I-1.1 节点位置、I-3.10 每个节点都有分配记录）都判过且成立，冷启动读回最后写的内容。
-/// I-3.1 在这里不判：`build_pool` 的分配器没装根环表、同一个进程里不回收（`released_placements_are_not_handed_out_again_before_reclaim_exists`），
-/// 根环转过之后被换下的落点还在 defer 里，而引用过它们的根已经被盖掉——那是这个装置的形状，不是这一轮要钉的；
-/// 装着根环表、会回收的那条会话上越过 812 条每一步都跑全部 checker，钉在随机历史
-/// `overwrites_raising_the_floor_and_rolling_back_past_812_allocation_records_all_succeed`。
+/// 同一个进程同一个实例里连着覆盖写，分配记录攒到多于 812 条（一片叶的条目容量，原先「分配记录树只有一个节点」那道墙）之后照样做成——
+/// 分配记录树按绝对槽号按位置寻址（D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K1），每块盘上装着记录的叶不止一片，墙拆了。
+/// `build_pool` 的分配器装着 mkfs 的根环（C551），根环转过一格之后照可再分配谓词回收（D16（发布语义） 已定项 1；这一形会话第一次回收
+/// 在 txg 26，第 31 批代码轮判决第二节 N2）；回收的落点被重新分配时改写原来那条记录、不追加（D3（空间分配） 已定项 7），
+/// 所以记录数只增不减，只是转环之后涨得慢。这条只钉按位置寻址那一半：越过 812 条的那一版上池级 checker 管按位置寻址的那几条
+/// （I-1.1 节点位置、I-3.10 每个节点都有分配记录）都判过且成立，冷启动读回最后写的内容。同一形会话每一版都跑全部不变量（I-3.1 在内）、
+/// 要求 0 违例的是 `allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records`。
 #[test]
 fn repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree(
 ) {
@@ -703,21 +702,40 @@ fn oldest_root_in_the_ring_after_this_process_wrote_every_txg_up_to(
 /// 被重新分配时覆盖」），回收的槽不再算已分配；还在 defer 窗口里的已释放落点仍算已分配（I-3.1 读法甲）。所以每一版每盘的「已分配」=
 /// 这一版交回的分配记录里那块盘上不是已回收的记录的跨度之和 × 16 KiB，「已回收」= 已释放且释放代 ≤ max(这一版的 F, 写到这一版时环里最旧的根)
 /// ——拿分配记录与根的 txg 现算，不看分配器的计数。逐版核，过 600 条的版本至少 5 个，其中带已回收记录的至少 1 个（判法里减掉已回收那一半真走到）。
+/// 这个判法的释放代读的是实现交回的分配记录，根环的槽位走的是实现的 `target_for_publish`：释放代记错、根环公式少一槽时它跟着实现一起偏，
+/// 照绿（第 31 批代码轮判决第二节 N3）。所以每一版另在盘上镜像上跑一遍池级 checker（全部不变量）、要求 0 违例——它自己从盘上读根环与
+/// 分配记录树、自己算谓词，不借实现的这两样（C551 推翻条件「同一条用例判全部不变量」）。镜像接着上一版只补施加这一次覆盖写发出的那一截。
 /// 每一次覆盖写接着上一版，次序本身就是被测对象，不切片并行。
 #[test]
 fn allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records(
 ) {
     let mut pool = build_pool("allocated-statistic-past-six-hundred-records");
     let genesis_txg = pool.genesis.root.checkpoint_txg;
+    let mut image = pool.memory_pool();
+    let mut operations_applied_to_the_image = pool.stream.operation_count();
     let mut versions_past_six_hundred_records = 0;
     let mut versions_carrying_reclaimed_records = 0;
     let mut overwrites = 0;
-    // 迭代上界 200 次（回收开始之前每次至少加 16 条，之后有的落点改写原来的记录）；跨轮携带的是现行那一版。
+    // 迭代上界 200 次（回收开始之前每次至少加 16 条，之后有的落点改写原来的记录）；跨轮携带的是现行那一版、
+    // 与它对应的盘上镜像和镜像已经施加到录制流第几步（不变量：等于流的长度，每一轮补施加之后重新对齐）。
     while pool.output.allocation_records.len() <= 812 && overwrites < 200 {
         let previous = pool.output.clone();
         let output = try_overwrite(&mut pool, &previous)
             .unwrap_or_else(|error| panic!("第 {} 次覆盖写：{error:?}", overwrites + 1));
         overwrites += 1;
+        let operations = pool.retained_operations();
+        image.apply(&operations[operations_applied_to_the_image..]);
+        operations_applied_to_the_image = operations.len();
+        let violated_invariants: Vec<(&str, InvariantVerdict)> = check_pool_image(&image)
+            .into_iter()
+            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
+            .collect();
+        assert!(
+            violated_invariants.is_empty(),
+            "第 {overwrites} 次覆盖写（txg {}，{} 条分配记录）之后池级 checker 判红：{violated_invariants:?}",
+            output.root.checkpoint_txg.0,
+            output.allocation_records.len()
+        );
         let reclaim_floor = output.root.rollback_floor.max(
             oldest_root_in_the_ring_after_this_process_wrote_every_txg_up_to(
                 output.root.checkpoint_txg,
diff --git a/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs b/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
index 248341e..b2f2226 100644
--- a/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
+++ b/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
@@ -12,7 +12,7 @@ use common::{
     abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, parameters,
     BuiltPool, FIXED_WRITE_TIME_SECONDS,
 };
-use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::image::{InvariantVerdict, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION};
 use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
@@ -273,6 +273,58 @@ fn raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slo
             );
             continue;
         }
+        if *invariant == "I-5.5" {
+            // E 之后的候选集（F_生效 = 11）是 txg 11–17 七条根，树表条目的诞生 txg 都是 3、不等于各自的 txg，认法 ② 不成立：
+            // 候选集里没有从 mkfs 水位第一次建树的那一版，I-5.5（从 mkfs 水位建树那一版的 t1 落点） 一版都不判、报不适用；
+            // 满足 ①② 的只有 txg 3 那一版，它在 F 之下。只认这一种理由，别的不适用照红（同一机理的先例与判别力在
+            // `pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`）。
+            assert!(
+                matches!(
+                    verdict,
+                    InvariantVerdict::NotApplicable(reason) if *reason == "这个镜像上没有它判的对象"
+                ),
+                "{invariant} 在 E 之后的镜像上报「这个镜像上没有它判的对象」的不适用：{verdict:?}"
+            );
+            continue;
+        }
+        if *invariant == "I-4.7" {
+            // 这条会话没有删除：走读到的每一版里都没有 nlink = 0 的记录，也没有有数据单元而没有 nlink ≥ 1 记录的号，
+            // I-4.7（删除中 inode 有账） 没有对象、报不适用；只认这一种理由，别的不适用照红（判别力在 `checker_known_bad_images.rs`
+            // 与 `pool_checker_judges_the_deletion_state_of_every_walked_version.rs`）。
+            assert!(
+                matches!(
+                    verdict,
+                    InvariantVerdict::NotApplicable(reason)
+                        if reason.contains("走读到的每一版里都没有处于删除中的 inode")
+                ),
+                "{invariant} 在 E 之后的镜像上报「走读到的每一版里都没有处于删除中的 inode」的不适用：{verdict:?}"
+            );
+            continue;
+        }
+        if *invariant == "I-4.6" {
+            // 同上，没有删除：I-4.6（中间态其余不变量成立） 自己判的那一格（nlink = 0 的记录单元数对 size）没有对象、报不适用；
+            // 只认这一种理由，别的不适用照红（判别力在 `checker_known_bad_images.rs` 与
+            // `pool_checker_judges_the_deletion_state_of_every_walked_version.rs`）。
+            assert!(
+                matches!(
+                    verdict,
+                    InvariantVerdict::NotApplicable(reason)
+                        if reason.contains("走读到的每一版 inode 树里都没有 nlink = 0 的记录")
+                ),
+                "{invariant} 在 E 之后的镜像上报「走读到的每一版 inode 树里都没有 nlink = 0 的记录」的不适用：{verdict:?}"
+            );
+            continue;
+        }
+        if *invariant == "I-4.4" {
+            // I-4.4（意图只描述目标态） 第一版恒报不适用（用户 2026-10-05 定）；只认池级 checker 写的那一句理由，别的不适用照红
+            // （判别力在 `checker_known_bad_images.rs` 与 `pool_checker_judges_the_deletion_state_of_every_walked_version.rs`）。
+            assert_eq!(
+                *verdict,
+                InvariantVerdict::NotApplicable(INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION),
+                "{invariant} 在 E 之后的镜像上报第一版不适用"
+            );
+            continue;
+        }
         assert_eq!(
             *verdict,
             InvariantVerdict::Holds,
````

### mutations-append.tsv

````
第 32 批 32-2 R3：释放时释放代少记一（记录与盘上都写成 txg − 1，按谓词提早一代可再分配；812 用例自己的判法从同一份记录读释放代、照绿，每一版跑的池级 checker 红）	crates/singlefs-core/src/allocator.rs	            record.generation = release_generation;	            record.generation = CheckpointTxg(release_generation.0.saturating_sub(1));	-p singlefs-harness --test overwrite_in_one_instance -- allocated_statistic_equals	allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records
第 32 批 32-2 B3：根环公式每区少用一槽（core 记根、写根与 812 用例模拟的根环一起偏，812 用例自己的判法照绿，每一版跑的池级 checker 红）	crates/singlefs-core/src/root_ring.rs	        slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),	        slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % (slots_per_region.count() - 1),	-p singlefs-harness --test overwrite_in_one_instance -- allocated_statistic_equals	allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records
第 32 批 32-2 R4：带单元的发布在取落点之前记根（这次记根回收出来的槽当场被这次发布取走，被盖掉的那条根落盘之前还引用着它们；一次挂载里转环那条已有用例红）	crates/singlefs-core/src/transaction.rs	    // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。\n    let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();\n    for identity in rewritten {\n        let placement = allocate_placement_for_role(allocator, *identity, txg)?;\n        slots.insert(*identity, placement.slot);\n    }\n    // 这次的根要盖掉根环里的一个槽：落点都取完了、这次不再分配，在装记账行之前记下它——盖掉的是被抛弃的根就清它的隔离位，\n    // 环里最旧有效根往前挪了就按谓词回收，记账行按回收之后的数写（与这条根一起说同一件事）。失败时整个分配器由调用方换回去。\n    // 记之前照 E2 重读挂载时读不出、记了这条根之后会是窗口里最小的那一槽（D16（发布语义） 已定项 1，用户 2026-10-03 定）。\n    crate::mount::record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(\n        &*pool.devices,\n        pool.parameters,\n        allocator,\n        txg,\n    );	    crate::mount::record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(\n        &*pool.devices,\n        pool.parameters,\n        allocator,\n        txg,\n    );\n    // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。\n    let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();\n    for identity in rewritten {\n        let placement = allocate_placement_for_role(allocator, *identity, txg)?;\n        slots.insert(*identity, placement.slot);\n    }\n    // 这次的根要盖掉根环里的一个槽：落点都取完了、这次不再分配，在装记账行之前记下它——盖掉的是被抛弃的根就清它的隔离位，\n    // 环里最旧有效根往前挪了就按谓词回收，记账行按回收之后的数写（与这条根一起说同一件事）。失败时整个分配器由调用方换回去。\n    // 记之前照 E2 重读挂载时读不出、记了这条根之后会是窗口里最小的那一槽（D16（发布语义） 已定项 1，用户 2026-10-03 定）。	-p singlefs-harness --test root_ring_turn_in_one_mount -- turning_the_root_ring_in_one_writable_mount	turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it
第 32 批 32-2 d：环转之前提前回收释放代 3（环里最旧有效根还是第 0 代时拿 txg 3 当回收门槛；一次挂载里转环那条已有用例红）	crates/singlefs-core/src/allocator.rs	        let reclaim_floor = crate::mount::reclaim_floor(\n            occupancy.effective_floor,\n            occupancy.oldest_valid_root_in_the_ring(),\n        );	        let reclaim_floor = crate::mount::reclaim_floor(\n            occupancy.effective_floor,\n            occupancy.oldest_valid_root_in_the_ring(),\n        );\n        let reclaim_floor = if reclaim_floor.0 == 0 {\n            CheckpointTxg(3)\n        } else {\n            reclaim_floor\n        };	-p singlefs-harness --test root_ring_turn_in_one_mount -- turning_the_root_ring_in_one_writable_mount	turning_the_root_ring_in_one_writable_mount_reclaims_what_the_predicate_releases_and_the_next_publish_takes_it
E171R1 装置 U11 那一侧：记根时环转了也不回收、不碰 core 断言（第 31 批判决第二节 N5；txg 26 起 I-3.1 红）	crates/singlefs-core/src/allocator.rs	            self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);	            {\n                let _ = (reclaim_floor, ReclaimedReuse::Immediately);\n                Vec::new()\n            };	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes -- the_third_stream_past_one_turn_of_the_root_ring_holds_i_3_1	the_third_stream_past_one_turn_of_the_root_ring_holds_i_3_1
````

## m3-formal-33-a

33-a：33-5 C551 连续覆盖写用例判全部不变量；33-6 ① 崩溃注入用例改期望

### crates.patch

````diff
diff --git a/crates/singlefs-harness/tests/overwrite_in_one_instance.rs b/crates/singlefs-harness/tests/overwrite_in_one_instance.rs
index f9c06a8b..8654dd16 100644
--- a/crates/singlefs-harness/tests/overwrite_in_one_instance.rs
+++ b/crates/singlefs-harness/tests/overwrite_in_one_instance.rs
@@ -607,16 +607,17 @@ fn release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not
 /// 分配记录树按绝对槽号按位置寻址（D8（核心索引结构） 已定项 14，用户 2026-09-24 定 K1），每块盘上装着记录的叶不止一片，墙拆了。
 /// `build_pool` 的分配器装着 mkfs 的根环（C551），根环转过一格之后照可再分配谓词回收（D16（发布语义） 已定项 1；这一形会话第一次回收
 /// 在 txg 26，第 31 批代码轮判决第二节 N2）；回收的落点被重新分配时改写原来那条记录、不追加（D3（空间分配） 已定项 7），
-/// 所以记录数只增不减，只是转环之后涨得慢。这条只钉按位置寻址那一半：越过 812 条的那一版上池级 checker 管按位置寻址的那几条
-/// （I-1.1 节点位置、I-3.10 每个节点都有分配记录）都判过且成立，冷启动读回最后写的内容。同一形会话每一版都跑全部不变量（I-3.1 在内）、
-/// 要求 0 违例的是 `allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records`。
+/// 所以记录数只增不减，只是转环之后涨得慢。越过 812 条的那一版上池级 checker 判全部不变量（I-3.1 在内）、0 违例（C551 推翻条件
+/// 「`build_pool` 装上根环表之后，同一条用例判全部不变量」），其中管按位置寻址的那两条（I-1.1 节点位置、I-3.10 每个节点都有分配记录）
+/// 还要判过且成立、不是没判；冷启动读回最后写的内容。这条只在末尾那一版跑一次 checker，每一版都跑的是
+/// `allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records`。
 #[test]
 fn repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree(
 ) {
     assert_eq!(index_node_entry_capacity(10, 20), 812);
     let mut pool = build_pool("overwrites-past-one-allocation-leaf");
     let mut overwrites = 0;
-    // 迭代上界 200 次（每次至少加 16 条，远在这之前就越过 812）；跨轮携带的是现行那一版。
+    // 迭代上界 200 次（回收开始之前每次至少加 16 条，之后有的落点改写原来的记录）；跨轮携带的是现行那一版。
     while pool.output.allocation_records.len() <= 812 && overwrites < 200 {
         let previous = pool.output.clone();
         pool.output = try_overwrite(&mut pool, &previous)
@@ -650,6 +651,16 @@ fn repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves
     }
     let image = pool.memory_pool();
     let verdicts = check_pool_image(&image);
+    let violated_invariants: Vec<&(&str, InvariantVerdict)> = verdicts
+        .iter()
+        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
+        .collect();
+    assert!(
+        violated_invariants.is_empty(),
+        "{overwrites} 次覆盖写（txg {}，{} 条分配记录）之后池级 checker 判红：{violated_invariants:?}",
+        pool.output.root.checkpoint_txg.0,
+        pool.output.allocation_records.len()
+    );
     for invariant in ["I-1.1", "I-3.10"] {
         assert_eq!(
             verdicts
diff --git a/crates/singlefs-checker-tier/src/crash_injection.rs b/crates/singlefs-checker-tier/src/crash_injection.rs
index 51ff5da2..dc76f29c 100644
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -6923,8 +6923,10 @@ mod tests {
     }
 
     /// C696 与 D13（验证路线） 已定项 1 挂载透明的射程（只管不带故障的状态）：第一次崩溃摆在覆盖写那次发布的第一个写上（它一个写都没落，
-    /// 盘面停在读回全 0 的那次可写挂载做成之后），这个态注入过读故障，崩溃镜像上那一次可写挂载做成，挂载透明整个不判、按理由记一次没判。
-    /// 判别力：注入态上也判挂载透明（`crates/mutations.tsv` 一行）时判过的不为空、没判的那一条理由是 0，这一条红。
+    /// 盘面停在读回全 0 的那次可写挂载做成之后），这个态注入过读故障，崩溃镜像上那一次可写挂载做成，挂载透明整个不判，它途中按段摆的
+    /// 三个二次崩溃也不判（第 28 批 G5），同一理由共记四次没判。
+    /// 判别力（`crates/mutations.tsv` 各一行）：第一次崩溃镜像上照判挂载透明（「第26批 C696 射程」）时没判的那一条理由只剩二次崩溃那 3 次；
+    /// 二次崩溃那一截不看标记、照判（「第28批 G5」）时它是 1；二次崩溃那一截跳过却不记没判（「第33批 G5」）时它也是 1。三种这一条都红。
     // crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态（ExactlyOneCrashState），不枚举
     #[test]
     fn mount_transparency_is_not_judged_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount(
@@ -6954,22 +6956,26 @@ mod tests {
             (
                 tally.crash_points,
                 tally.crash_points_after_a_step_reading_under_injected_faults,
-                tally.writable_mounts_after_the_crash_succeeded
+                tally.writable_mounts_after_the_crash_succeeded,
+                tally.second_crash_points
             ),
-            (1, 1, 1),
-            "前提：只摆写死的那一个第一次崩溃态，它注入过读故障，它上面的可写挂载做成：{}",
+            (1, 1, 1, 3),
+            "前提：只摆写死的那一个第一次崩溃态，它注入过读故障，它上面的可写挂载做成，挂载途中按段摆了三个二次崩溃：{}",
             tally.render()
         );
         assert_eq!(
             (
                 tally.mount_transparency_judged.clone(),
+                tally
+                    .mount_transparency_judged_on_second_crashes_by_phase
+                    .clone(),
                 tally
                     .mount_transparency_not_judged
                     .get(MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE)
                     .copied()
             ),
-            (BTreeMap::new(), Some(1)),
-            "注入过读故障的崩溃态上挂载透明一个动作都不判，按理由记一次：{}",
+            (BTreeMap::new(), BTreeMap::new(), Some(1 + 3)),
+            "注入过读故障的崩溃态上挂载透明一个动作都不判：崩溃镜像上那次挂载记一次，它途中的三个二次崩溃各记一次：{}",
             tally.render()
         );
     }
````

### mutations-append.tsv

````
第33批 C551 连续覆盖写：释放时释放代少记一（按谓词提早一代可再分配；越过 812 条那一版上池级 checker 判全部不变量红）	crates/singlefs-core/src/allocator.rs	            record.generation = release_generation;	            record.generation = CheckpointTxg(release_generation.0.saturating_sub(1));	-p singlefs-harness --test overwrite_in_one_instance	repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree
第33批 C551 连续覆盖写：根环公式每区少用一槽（越过 812 条那一版上池级 checker 判全部不变量红）	crates/singlefs-core/src/root_ring.rs	        slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),	        slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % (slots_per_region.count() - 1),	-p singlefs-harness --test overwrite_in_one_instance	repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree
第33批 G5 二次崩溃那一截跳过挂载透明却不记没判	crates/singlefs-checker-tier/src/crash_injection.rs	            ReadFaultsInTheCrashState::Injected => {\n                *tally\n                    .mount_transparency_not_judged\n                    .entry(MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE)\n                    .or_insert(0) += 1;\n                Vec::new()	            ReadFaultsInTheCrashState::Injected => {\n                Vec::new()	-p singlefs-checker-tier --lib -- crash_injection::tests::mount_transparency_is_not_judged_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount	mount_transparency_is_not_judged_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount
````

## m3-formal-33-b

33-b：33-7 ② 回退下界用例换造法、第 38 行变异改名换替换文

### crates.patch

````diff
diff --git a/crates/singlefs-checker-tier/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs b/crates/singlefs-checker-tier/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs
--- a/crates/singlefs-checker-tier/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs
+++ b/crates/singlefs-checker-tier/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs
@@ -363,23 +363,54 @@
     );
 }
 
-/// 回退候选集的下界取 F 生效值（D16（发布语义） 已定项 1「回退候选集」「生效」），不是最新根自己带的 F：同上抬到 3、两条带新 F 的根都读不出，
-/// 最新读得出的根是 txg 6（带 F 0）；挂着的时候回退到暖机那条 txg 2 的根，在任何写之前拒成 `BelowEffectiveFloor`，盘上逐字节不变、
-/// 分配器与现行版本不动。按最新根自己的 F（0）判，它过得了下界这一道，报的是「树表 0 条」（`VersionWithoutFile`）。
+/// 回退候选集的下界取 F 生效值（D16（发布语义） 已定项 1「回退候选集」「生效」），不是最新根自己带的 F、也不是目标根自己带的 F。
+/// 头几步照下面那条「先写系统配置那一步失败」：覆盖写三次到 txg 6，抬 F 到 3，第二次写（盘 1 的系统配置槽）报块设备错——
+/// 只有盘 0 的系统配置带新 F 3，一条带新 F 的根都没发；最新读得出的根仍是 txg 6（带 F 0），F 生效值是 3。
+/// 同一个进程里回退到暖机那条 txg 2 的根（实例 1，根槽 (1, 2)，带 F 0），在任何写之前拒成 `BelowEffectiveFloor`：录制流不多一步、
+/// 分配器与现行版本不动。按最新根或目标根自己的 F（都是 0）判，它过得了下界这一道，报的是「树表 0 条」（`VersionWithoutFile`）。
+///
+/// 为什么不用原来的造法（抬到 3 做成、再把 txg 7、8 那两条带新 F 的根改坏）：31-1 之后 `build_pool` 的分配器装了根环表，那两槽是
+/// 这个进程写过的、「这个进程知道住着根的槽」，判候选与算 F_生效 读根环时重读一次仍坏，就先拒成 `CandidateJudgementStillUnreadableAfterOneReread`、
+/// 到不了下界这一判（D16（发布语义） 已定项 1「根槽这一次读坏」那一行）；那一形由 `reuse_after_raising_the_floor.rs` 的
+/// `rolling_back_after_the_floor_carrier_this_process_wrote_on_device_one_reads_not_self_verified_is_refused_before_any_write` 钉成先拒。
+/// 这一造法一个根槽都不碰，下界只能从盘 0 的系统配置里来。
 #[test]
 fn rolling_back_below_a_floor_only_the_system_configuration_carries_is_refused_before_any_write() {
-    let mut pool = build_pool("rbf2-rollback-below-the-system-configuration-floor");
-    overwrite_times(&mut pool, 3);
-    raise_floor(&mut pool, CheckpointTxg(3)).expect("抬到上限 3");
-    make_the_root_unreadable(&mut pool, CheckpointTxg(7));
-    make_the_root_unreadable(&mut pool, CheckpointTxg(8));
-    let newest_readable_root = {
-        let devices = pool.devices.as_ref().expect("镜像还开着");
-        readable_roots_of(devices.as_slice())
-            .into_iter()
-            .max_by_key(|root| (root.checkpoint_txg, root.instance))
-            .expect("根环里有根")
-    };
+    let mut pool = three_overwrites_on_fault_injected_devices();
+    let roots_before_the_raise = readable_roots_of(pool.devices.as_slice());
+    pool.fault_plan
+        .arm(FaultSchedule::the_nth_call_across_the_pool(
+            InjectedFault::WriteFails,
+            2,
+        ));
+    let mut current_for_the_raise = pool.output.clone();
+    let refused_raise = raise_rollback_floor(
+        &parameters(),
+        &mut pool.devices,
+        &mut pool.allocator,
+        &mut current_for_the_raise,
+        CheckpointTxg(3),
+        ShadowLedger::On,
+    );
+    pool.fault_plan.disarm();
+    assert!(
+        matches!(
+            &refused_raise,
+            Err(MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(failed))
+                if failed.devices_carrying_the_raised_floor == vec![DeviceIdentity(0)]
+        ),
+        "盘 1 的系统配置写报错：先写系统配置那一步失败、只有盘 0 带新 F：{:?}",
+        refused_raise.as_ref().err()
+    );
+    let roots_after_the_raise = readable_roots_of(pool.devices.as_slice());
+    assert_eq!(
+        roots_after_the_raise, roots_before_the_raise,
+        "抬 F 这一串一条根都没发：根环逐条不变"
+    );
+    let newest_readable_root = roots_after_the_raise
+        .iter()
+        .max_by_key(|root| (root.checkpoint_txg, root.instance))
+        .expect("根环里有根");
     assert_eq!(
         (
             newest_readable_root.checkpoint_txg,
@@ -388,14 +419,37 @@
         (CheckpointTxg(6), CheckpointTxg(0)),
         "最新读得出的根是 txg 6、带 F 0"
     );
-    let before = disk_snapshot(&pool.memory_pool(), &pool.stream);
+    let target_root = roots_after_the_raise
+        .iter()
+        .find(|root| {
+            root.instance == InstanceGeneration(1) && root.checkpoint_txg == CheckpointTxg(2)
+        })
+        .expect("暖机那条 txg 2 的根读得出");
+    assert_eq!(
+        target_root.rollback_floor,
+        CheckpointTxg(0),
+        "目标根自己带 F 0"
+    );
+    assert_eq!(
+        (
+            system_configuration_floors_on(pool.devices.as_slice(), DeviceIdentity(0))
+                .into_iter()
+                .max(),
+            system_configuration_floors_on(pool.devices.as_slice(), DeviceIdentity(1))
+                .into_iter()
+                .max()
+        ),
+        (Some(CheckpointTxg(3)), Some(CheckpointTxg(0))),
+        "只有盘 0 的系统配置带新 F 3，盘 1 没写进"
+    );
+    // F 生效值是 3 这里不另断言：同一盘面在下面那条「先写系统配置那一步失败」里断言过；F_生效 不取系统配置的变异要红在回退判候选这一判上，
+    // 不在前面的搭建核对上。
+    let operations_before_the_rollback = pool.stream.operation_count();
     let current_before = pool.output.clone();
     let records_before = pool.allocator.records().to_vec();
-    let rollback_parameters = parameters();
-    let devices = pool.devices.as_mut().expect("镜像还开着");
     let refused = roll_back_by_a_forward_publish(
-        &rollback_parameters,
-        devices,
+        &parameters(),
+        &mut pool.devices,
         &mut pool.allocator,
         &mut pool.output,
         RollbackTarget {
@@ -407,17 +461,20 @@
         matches!(
             &refused,
             Err(RollbackError::TargetNotACandidate {
+                target: RollbackTarget {
+                    instance: InstanceGeneration(1),
+                    checkpoint_txg: CheckpointTxg(2),
+                },
                 exclusion: RollbackCandidateExclusion::BelowEffectiveFloor,
-                ..
             })
         ),
         "txg 2 低于 F 生效值 3：不是候选：{:?}",
         refused.as_ref().err()
     );
     assert_eq!(
-        disk_snapshot(&pool.memory_pool(), &pool.stream),
-        before,
-        "拒在任何写之前"
+        pool.stream.operation_count(),
+        operations_before_the_rollback,
+        "拒在任何写之前：录制流不多一步"
     );
     assert_eq!(pool.output, current_before, "现行版本不动");
     assert_eq!(
````

### mutations-append.tsv

````
步 4：回退候选集的 F 用目标根自己带的 F 而不是 F_生效	crates/singlefs-core/src/mount.rs	    if target.checkpoint_txg < effective_floor {	    if target.checkpoint_txg < target_root.rollback_floor {	-p singlefs-checker-tier --test rollback_floor_written_into_the_system_configuration_first_and_normal_unmount -- rolling_back_below_a_floor	rolling_back_below_a_floor_only_the_system_configuration_carries_is_refused_before_any_write
33-7（D16 已定项 1「生效」SysPre）：抬 F 先写系统配置那一步盘 1 失败、只有盘 0 的系统配置带新 F 时，挂着的回退判候选 F_生效 不与系统配置里读得出的取 max，低于它的目标被当成候选	crates/singlefs-core/src/recovery.rs	    highest_on_the_roots\n        .max(highest_in_the_system_configurations)\n        .unwrap_or(CheckpointTxg(0))	    highest_on_the_roots\n        .max(highest_in_the_system_configurations.filter(|_| false))\n        .unwrap_or(CheckpointTxg(0))	-p singlefs-checker-tier --test rollback_floor_written_into_the_system_configuration_first_and_normal_unmount -- rolling_back_below_a_floor_only_the_system_configuration	rolling_back_below_a_floor_only_the_system_configuration_carries_is_refused_before_any_write
````

### mutations-delete.txt

````
步 4：回退候选集的 F 用最新根自己带的 F 而不是 F_生效
````

## m3-formal-33-c

33-c：33-3 F1 同形（树表 0 条、根记录直接持有的分配记录树读不出）

### crates.patch

````diff
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -5048,6 +5048,8 @@
 /// 树表指针非零而树表读不出的根，这一格判不了（认不出带不带树），它那一版自己的分配记录树（树表种类 3 那一条指着的那一棵）也走不到：
 /// 不计成立，I-5.4 报部分没判、逐版列出（判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2
 /// 「I-5.4 按版本：坏盘」那一行的改法 F1：此前这种根整版不进评估，别的版本的树走到了叶就报成立）。
+/// 树表读得出、0 条而根记录那一项非零的根（写过行的那一版，C512（树表 0 条的一版上被换下的单元记在哪）），它那一版的分配记录树就是
+/// 根记录直接持有的那一棵：那棵树走不到叶时同样不计成立，I-5.4 报部分没判、逐版列出（F1 的同形，第 33 批造盘面证实：此前同样整版不进评估）。
 /// 这一格成立的那一次评估只在**这一版自己的**分配记录树（树表种类 3 那一条指着的那一棵）走到叶时计进 I-5.4（判决
 /// `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J1「I-5.4 挂法（C2）」那一行，按版本那一层是
 /// `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2 的零轮收严）：没走到的那几版不计，I-5.4 报部分没判、逐版列出没计的那几版
@@ -5072,9 +5074,15 @@
         Vec::new();
     // 树表指针非零而树表读不出的那几版：认不出带不带树，「根记录里分配记录树根全零」那一格与这一版自己的分配记录树都没判。
     let mut versions_whose_tree_table_does_not_read_back: Vec<String> = Vec::new();
+    // 树表读得出、0 条，分配记录树由根记录直接持有（C512（树表 0 条的一版上被换下的单元记在哪））而那棵树走不到叶的那几版：
+    // 这一版自己的分配记录树没判（判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2「I-5.4 按版本：坏盘」
+    // 那一行改法 F1 的同形，`research/prompts/m3-formal-code-b30-r1-opus-output.md` 第 201 行记下、第 33 批造盘面证实）。
+    let mut versions_whose_root_record_allocation_record_tree_does_not_read_back: Vec<String> =
+        Vec::new();
     for index in candidate_indexes.iter().copied() {
         let (_, _, root) = &roots[index];
         let mut this_version_carries_trees_and_a_zero_root_record_allocation_tree_root = false;
+        let mut this_version_holds_its_allocation_record_tree_in_the_root_record = false;
         // 这条根的分配记录树：树表里种类 3 那一条指着的根，以及树表 0 条那一版由根记录直接持有的那一棵
         // （C512（树表 0 条的一版上被换下的单元记在哪），代码审阅 6c 第 9 条：原来这一棵不读）。带文件的一版后者恒全零。
         let mut allocation_roots = vec![parse_node_pointer(
@@ -5091,6 +5099,8 @@
                 ));
             }
             if let Some(tree_table) = tree_table_read_back {
+                this_version_holds_its_allocation_record_tree_in_the_root_record =
+                    tree_table.entries.is_empty() && !allocation_roots[0].all_zero;
                 if !tree_table.entries.is_empty() {
                     if allocation_roots[0].all_zero {
                         this_version_carries_trees_and_a_zero_root_record_allocation_tree_root =
@@ -5150,6 +5160,14 @@
                 this_version_allocation_record_tree_walked_to_its_leaves,
             ));
         }
+        if this_version_holds_its_allocation_record_tree_in_the_root_record
+            && !this_version_allocation_record_tree_walked_to_its_leaves
+        {
+            versions_whose_root_record_allocation_record_tree_does_not_read_back.push(format!(
+                "根（实例 {}、txg {}）",
+                root.instance, root.checkpoint_txg
+            ));
+        }
     }
     let some_allocation_record_tree_walked_to_its_leaves =
         !allocation_roots_walked_to_their_leaves.is_empty();
@@ -5190,6 +5208,12 @@
             versions_whose_tree_table_does_not_read_back.join("、")
         ));
     }
+    if !versions_whose_root_record_allocation_record_tree_does_not_read_back.is_empty() {
+        unjudged_parts.push(format!(
+            "树表 0 条、分配记录树由根记录直接持有的版本，那棵树读不出、走不到叶，它自己的分配记录一条都没判（别的版本的树走到了叶不替它作保）：{}",
+            versions_whose_root_record_allocation_record_tree_does_not_read_back.join("、")
+        ));
+    }
     if !unjudged_parts.is_empty() {
         judgements.partially_judged("I-5.4", unjudged_parts.join("；"));
     }
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -8717,6 +8717,100 @@
     );
 }
 
+/// 只做过 mkfs 的池可写挂载两次（第二次写实例 1 那一行）、再在同一个进程里发第一个文件版本：写行那次发布（实例 2、txg 3）与暖机
+/// （实例 2、txg 4）是树表 0 条（照抄 mkfs 那片）、根记录直接持有分配记录树的那一版（C512（树表 0 条的一版上被换下的单元记在哪）），
+/// 第一个文件版本（实例 2、txg 5）的分配记录树住它自己树表种类 3 那一条。返回镜像与根记录那一项指着的分配记录树根所在的槽（两盘同槽）。
+fn image_after_the_first_file_on_the_version_whose_rows_were_written(
+    tag: &str,
+) -> (MemoryPool, u64) {
+    let mut pool = format_pool(tag);
+    let mut devices = pool.reopen_recorded();
+    let first_instance =
+        mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载：不写行");
+    drop(first_instance);
+    let row_written = mount_writable(&parameters(), &mut devices).expect("再挂一次：写行");
+    let PoolVersion::WithoutFile(version) = &row_written.current else {
+        panic!("写行之后现行那一版树表 0 条")
+    };
+    assert_eq!(
+        version.root.tree_table, pool.genesis.root.tree_table,
+        "写行那一版的树表照抄 mkfs 那片 0 条的"
+    );
+    assert_ne!(
+        version.root.allocation_record_tree_root,
+        singlefs_core::pointer::NodePointer::empty_root(),
+        "写行那一版的分配记录树根住根记录"
+    );
+    let root_record_allocation_tree_root_slot =
+        slot_shared_by_both_copies(version.root.allocation_record_tree_root.locations);
+    let mut allocator = row_written.allocator.clone();
+    {
+        let publish_parameters = parameters();
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        let first_file = publish_first_file(
+            &mut writer,
+            &mut allocator,
+            &version.root,
+            FirstFile {
+                content: &common::file_content(),
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
+            },
+            row_written.output.instance,
+            &version.record_bytes,
+        )
+        .expect("写行那一版上发第一个文件版本");
+        assert_eq!(
+            first_file.root.checkpoint_txg,
+            CheckpointTxg(5),
+            "第一个文件版本是 txg 5"
+        );
+    }
+    drop(devices);
+    (pool.memory_pool(), root_record_allocation_tree_root_slot)
+}
+
+/// I-5.4 按版本，树表 0 条、根记录直接持有的分配记录树读不出（改法 F1 的同形，`research/prompts/m3-formal-code-b30-r1-opus-output.md` 第 201 行记下、没造盘面）：
+/// 写行那一版之上发了第一个文件版本的镜像上，把写行那一版根记录那一项指着的分配记录树根两份清零。判红的是读不出的单元那几条，
+/// 与改之前相同；改之前写行那两版（txg 3、4）整版不进评估，第一个文件版本的树走到了叶，I-5.4 就报成立。
+#[test]
+fn the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
+) {
+    let (clean, root_record_allocation_tree_root_slot) =
+        image_after_the_first_file_on_the_version_whose_rows_were_written(
+            "known-bad-disjointness-root-record-allocation-tree-zeroed",
+        );
+    assert_eq!(
+        violated_invariants(&check_pool_image(&clean)),
+        Vec::<&str>::new(),
+        "写行那一版之上发了第一个文件版本的干净镜像上一条都不红"
+    );
+    assert_eq!(
+        verdict(&clean, "I-5.4"),
+        InvariantVerdict::Holds,
+        "干净镜像上 I-5.4 真被评估过且成立"
+    );
+    let mut image = clean.clone();
+    zero_both_copies_of_the_unit(
+        &mut image,
+        root_record_allocation_tree_root_slot,
+        usize::try_from(NODE_BYTES).expect("16384"),
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-2.1", "I-3.1", "I-4.8", "I-7.4"],
+        "判红的照改之前那几条：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict(&image, "I-5.4"),
+        InvariantVerdict::PartiallyJudged(
+            "树表 0 条、分配记录树由根记录直接持有的版本，那棵树读不出、走不到叶，它自己的分配记录一条都没判（别的版本的树走到了叶不替它作保）：根（实例 2、txg 3）、根（实例 2、txg 4）"
+                .to_owned()
+        ),
+        "写行那两版：第一个文件版本的树走到了叶不替它们计成立，I-5.4 报部分没判、逐版点名"
+    );
+}
+
 /// 新池新建文件那一版里一条走读跟随的指针头部 `change` 那一段改成非第一版的取值、重封、校验和沿引用链补到根槽：池级 checker 只红 I-2.4，
 /// 违例那一句是 `expected_violation`（点名是哪条指针、哪一段）。core 读这条指针时拒（判这条指针所在的结构损坏）；改之前池级 checker 只判
 /// MAC / nonce 与 extent 偏移，这三段改了一条都不红（第 30 批攻方 `B30ALG … checker_violated=[]` 那几行，判决
````

### mutations-append.tsv

````
第 33 批 33-3 F1 同形 池级 checker：树表 0 条、根记录直接持有的分配记录树读不出的版本不记进 I-5.4 部分没判（写行那一版之上发了第一个文件版本，照旧报成立）	crates/singlefs-checker/src/walk.rs	            versions_whose_root_record_allocation_record_tree_does_not_read_back.push(format!(	            Vec::<String>::new().push(format!(	-p singlefs-harness --test checker_known_bad_images -- the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged	the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged
第 33 批 33-3 F1 同形 池级 checker：认不出树表 0 条、分配记录树由根记录直接持有的版本（退回只罩树表指针非零而读不出那一支）	crates/singlefs-checker/src/walk.rs	                    tree_table.entries.is_empty() && !allocation_roots[0].all_zero;	                    false;	-p singlefs-harness --test checker_known_bad_images -- the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged	the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged
````

## E156 装置（第 5、6 次执行员）

### diff -u 第 31 批冻结副本那一版 今天那一版

````diff
--- /tmp/claude-1000/m3-formal-code-b31-r1/tree/crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs	2026-10-05 11:42:05.011494305 +0000
+++ crates/singlefs-checker-tier/src/bin/e156_allocation_basis_counts.rs	2026-10-05 13:57:20.249082118 +0000
@@ -10,7 +10,12 @@
 //!
 //! 只读驱动 `singlefs-core` / `singlefs-checker` 的公开入口，不改这两个 crate 的生产代码；产物按 `E7RESULT` 行打印，复跑登记在
 //! `research/scripts/replay.sh`。洞的 txg、挂载做完的 txg、终点 txg 从本地 S 与装置里的锚点模型（[`anchor_model`]，登记第七节 7.2
-//! 命令四 `anchors_e156_r4.py` 的逐行移植）算，不从 `crates/` 算；装置跑的时候从盘上现量，两边比。
+//! 命令四 `anchors_e156_r4.py` 的逐行移植，门槛那一处照第 6 次重跑登记 `research/prompts/e156-r6-prereg.md` 第十三节命令四
+//! `anchors_e156_r6.py` 改）算，不从 `crates/` 算；装置跑的时候从盘上现量，两边比。
+//!
+//! 第 6 次重跑（登记 `research/prompts/e156-r6-prereg.md` 第一节 W15–W18、第五节 E7–E11）：回收门槛照用户 2026-10-05 定乙读——
+//! 挂载那一刻读回全零的根环槽钉在它可能住过的最旧 txg，直到这个进程写掉那一槽、或读出一条根（[`read_hh_state`] 的
+//! `all_zero_slots_still_pinned`）；K10 守门、「每」列的旁路门槛与锚点模型三处吃同一个读数。
 //!
 //! 判定写在装置里（[`judge_peak_against_threshold`] 那一组函数，单测 U18），不另写 Python 判定脚本：执行员的写范围不含
 //! `research/scripts/` 与门禁 47 号，理由与改动记在登记第十二节。
@@ -22,7 +27,7 @@
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
 };
-use singlefs_core::allocator::{AllocationRecord, DeviceFreeMap, Placement, PoolAllocator};
+use singlefs_core::allocator::{AllocationRecord, PoolAllocator};
 use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
 use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
 use singlefs_core::journal::back_chain_of;
@@ -1069,7 +1074,8 @@
 // 只用脚本头那几句条款：根环 txg u 落槽 u mod 3S；可再分配 = 已释放 ∧ 释放代 ≤ 环里最旧有效根（Hh 里 F_生效 = 0）；
 // 每一次发布（连空发布）都重写树表单元，mkfs 之后两次暖机是零单元发布；暖机直到两块盘上各有一条本实例的根；
 // c2 之后新实例第一个 txg = max(环里最高, 记录最高) + 1。产物里 `name=anchor_dump` 那几行去掉前缀之后与 Python 脚本
-// `--dump` 的 `DUMP` 行逐行比（报告里的命令）。
+// `--dump` 的 `DUMP` 行逐行比（报告里的命令）。第 6 次重跑登记第五节 E11：门槛那一处照第十三节命令四 `anchors_e156_r6.py` 改，
+// 并进这次挂载读回全零、还没被写掉的槽（槽号即落到它的最小 txg；定乙），`h_缺`、`h_洞` 与两条闭式照旧读环里最旧的根。
 // ============================================================================================
 mod anchor_model {
     use std::collections::{BTreeMap, BTreeSet};
@@ -1197,6 +1203,8 @@
         let mut tree_tables: BTreeMap<u64, Option<u64>> = BTreeMap::from([(0, None)]);
         let mut live_tree_table = 0u64;
         let mut rows = Vec::new();
+        let mount_first_txgs: BTreeSet<u64> = holes.iter().map(|hole| hole + 1).collect();
+        let mut pins: BTreeSet<u64> = BTreeSet::new();
         for txg in 0..=end {
             let role = roles[&txg];
             if !ZERO_UNIT_WARM_UP_TXGS.contains(&txg) && txg > 0 {
@@ -1204,6 +1212,11 @@
                 tree_tables.insert(txg, None);
                 live_tree_table = txg;
             }
+            if mount_first_txgs.contains(&txg) {
+                pins = (0..ring_length.min(txg))
+                    .filter(|slot| !ring.contains_key(slot))
+                    .collect();
+            }
             if role != AnchorPublishRole::Crash {
                 ring.insert(txg % ring_length, txg);
             }
@@ -1211,7 +1224,11 @@
                 continue;
             }
             let oldest = *ring.values().min().expect("环里至少有 mkfs 那一条根");
-            let floor = oldest;
+            let floor = pins
+                .iter()
+                .copied()
+                .filter(|slot| !ring.contains_key(slot))
+                .fold(oldest, u64::min);
             let roots: BTreeSet<u64> = ring.values().copied().collect();
             let is_referenced = |allocation: u64, release: Option<u64>| match release {
                 None => true,
@@ -1409,35 +1426,12 @@
     }
 }
 
-/// W7：分配器从哪来。产品路径（mkfs 同一个进程里装根环表，之后用每次可写挂载交回的）；PC-分配器换回不装根环表的那一种。
+/// W7：分配器从哪来。只有产品路径一种（mkfs 同一个进程里装根环表，之后用每次可写挂载交回的）。第 5 次重跑登记 5.5 E1 撤了
+/// 「不装根环表」那一种（C551：没装根环表的分配器在第一次记根就断言，到不了观测点），成员名不改：`name=q1_cell` 行打
+/// `allocator={:?}`，改名会让每一格多一处不同。
 #[derive(Clone, Copy, PartialEq, Eq, Debug)]
 enum HhAllocatorSource {
     ProductPathWithRootRing,
-    WithoutRootRingPositiveControl,
-}
-
-/// 不装根环表的分配器（PC-分配器）：`PoolAllocator::new` 加 mkfs 那两个单元，与装置原有的做法相同。
-fn allocator_without_the_root_ring(
-    devices: &[(DeviceIdentity, SparseBlockDevice)],
-    genesis: &MakeFilesystemOutput,
-) -> PoolAllocator {
-    let mut allocator = PoolAllocator::new(
-        devices
-            .iter()
-            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
-            .collect(),
-    );
-    allocator.mark_format_time_units(
-        Placement {
-            slot: genesis_slot_on_device_zero(&genesis.root.instance_table.locations),
-            span: INSTANCE_TABLE_SPAN_SLOTS,
-        },
-        Placement {
-            slot: genesis_slot_on_device_zero(&genesis.root.tree_table.locations),
-            span: TransactionUnit::TreeTable.span_slots(),
-        },
-    );
-    allocator
 }
 
 fn genesis_slot_on_device_zero(
@@ -1460,9 +1454,6 @@
         HhAllocatorSource::ProductPathWithRootRing => {
             allocator_after_make_filesystem(parameters, devices, genesis)
         }
-        HhAllocatorSource::WithoutRootRingPositiveControl => {
-            allocator_without_the_root_ring(devices, genesis)
-        }
     }
 }
 
@@ -1787,6 +1778,16 @@
     isolated_slots: u64,
     not_free_beyond_allocated: u64,
     root_ring_installed: bool,
+    /// W18 (c)(d)：到这个观测点还钉着的全零槽数。
+    all_zero_slots_still_pinned: u64,
+    /// B 让门槛比只按 A（读得出的有效根）取更低（第五节 E10 的 `threshold_lowered_points`）。
+    threshold_lowered_by_all_zero_pins: bool,
+    /// W18 (a)：这个观测点看到这个进程写掉了的钉住槽。
+    all_zero_pins_released_by_write: Vec<RootRingSlot>,
+    /// W18 (b)：这个观测点看到读出了一条不是这个进程写的根的钉住槽（这一族推的是 0，不为 0 照 S18 停）。
+    all_zero_pins_released_by_reread_only: Vec<RootRingSlot>,
+    /// W18 (d)：还钉着而读坏的槽数（这一族不造读故障，不为 0 照 S17 停）。
+    bad_slots_still_pinned: u64,
 }
 
 struct HhStateInputs<'context> {
@@ -1796,6 +1797,10 @@
     versions: &'context BTreeMap<u64, PublishedVersionFacts>,
     allocation_generations: &'context HashMap<u64, u64>,
     injected_holes: &'context [u64],
+    /// 第 6 次重跑登记 E7：这次挂载钉着的槽与它可能住过的最旧 txg；mkfs 那个进程为空。
+    all_zero_pins: &'context BTreeMap<RootRingSlot, u64>,
+    /// 这次挂载新实例的第一个 txg；mkfs 那个进程为 `None`。
+    first_txg_of_this_mount: Option<u64>,
 }
 
 fn read_hh_state(inputs: &HhStateInputs<'_>) -> HhStateReading {
@@ -1840,14 +1845,47 @@
     .0;
     let newest_root_txg = newest_root.checkpoint_txg.0;
     let slots_per_region = geometry.root_ring_slots_per_region;
-    let ring_threshold =
-        reclaim_threshold_read_from_the_ring_image(effective_floor, &valid_root_txgs);
-    let oldest_valid_root_txg = valid_root_txgs.iter().min().copied().unwrap_or(0);
-    let missing_txgs = missing_txg_count(oldest_valid_root_txg, newest_root_txg, &ring_root_txgs);
     let occupant_by_ring_slot: BTreeMap<RootRingSlot, u64> = ring
         .iter()
         .map(|(ring_slot, root)| (*ring_slot, root.checkpoint_txg.0))
         .collect();
+    // W18：钉着的每一槽读这一刻那一槽里自证合法的根；「写掉」「重读读出」两句都不成立的仍钉着。
+    let mut all_zero_slots_still_pinned: BTreeSet<u64> = BTreeSet::new();
+    let mut all_zero_slots_still_pinned_count = 0u64;
+    let mut all_zero_pins_released_by_write = Vec::new();
+    let mut all_zero_pins_released_by_reread_only = Vec::new();
+    let mut bad_slots_still_pinned = 0u64;
+    if let Some(first_txg) = inputs.first_txg_of_this_mount {
+        for (ring_slot, oldest_txg_it_can_have_held) in inputs.all_zero_pins {
+            let occupant = occupant_by_ring_slot.get(ring_slot).copied();
+            let released_by_write = occupant.is_some_and(|occupant_txg| occupant_txg >= first_txg);
+            let released_by_reread = occupant.is_some_and(|occupant_txg| occupant_txg < first_txg);
+            if released_by_write {
+                all_zero_pins_released_by_write.push(*ring_slot);
+            } else if released_by_reread {
+                all_zero_pins_released_by_reread_only.push(*ring_slot);
+            } else {
+                all_zero_slots_still_pinned.insert(*oldest_txg_it_can_have_held);
+                all_zero_slots_still_pinned_count += 1;
+                if read_root_ring_slot_content(inputs.devices, inputs.parameters, *ring_slot)
+                    != RootRingSlotContent::AllZero
+                {
+                    bad_slots_still_pinned += 1;
+                }
+            }
+        }
+    }
+    let ring_threshold = reclaim_threshold_read_from_the_ring_image(
+        effective_floor,
+        &valid_root_txgs
+            .union(&all_zero_slots_still_pinned)
+            .copied()
+            .collect(),
+    );
+    let threshold_lowered_by_all_zero_pins = ring_threshold
+        < reclaim_threshold_read_from_the_ring_image(effective_floor, &valid_root_txgs);
+    let oldest_valid_root_txg = valid_root_txgs.iter().min().copied().unwrap_or(0);
+    let missing_txgs = missing_txg_count(oldest_valid_root_txg, newest_root_txg, &ring_root_txgs);
     let live_holes = live_hole_count(
         inputs.injected_holes,
         oldest_valid_root_txg,
@@ -1920,6 +1958,11 @@
             .expect("落在 u64 内")
             .saturating_sub(free_map_zero.allocated_slots()),
         root_ring_installed: inputs.allocator.root_ring_occupancy().is_some(),
+        all_zero_slots_still_pinned: all_zero_slots_still_pinned_count,
+        threshold_lowered_by_all_zero_pins,
+        all_zero_pins_released_by_write,
+        all_zero_pins_released_by_reread_only,
+        bad_slots_still_pinned,
     }
 }
 
@@ -2040,6 +2083,11 @@
     tree_table_lags_every_publish: BTreeSet<u64>,
     checker_red_on_hole_states: u64,
     checker_judged_hole_states: u64,
+    /// 第 6 次重跑登记 E8：每次挂载那一刻的定乙钉住。
+    all_zero_pin_mounts: Vec<AllZeroPinsAtMount>,
+    /// W18 (a)、(b) 解开的钉住槽数（整格累加，每槽每次挂载最多一次）。
+    all_zero_pins_released_by_write: u64,
+    all_zero_pins_released_by_reread_only: u64,
 }
 
 /// 一格要怎么跑（第 4 次重跑登记第五节 5.3、5.4）。
@@ -2109,6 +2157,93 @@
     }
 }
 
+/// 根环一个槽这一刻读到的样子（第 6 次重跑登记 W17）：读出自证过的根；读不出自证而原字节每个都是 0；
+/// 其余（读不出、读得出而不是全零又自证不过）都算读坏，这一族不造，出现了照 S17 停。
+#[derive(Clone, Copy, PartialEq, Eq, Debug)]
+enum RootRingSlotContent {
+    SelfVerifiedRoot { checkpoint_txg: u64 },
+    AllZero,
+    Bad,
+}
+
+fn read_root_ring_slot_content(
+    devices: &[(DeviceIdentity, SparseBlockDevice)],
+    parameters: &MakeFilesystemParameters,
+    ring_slot: RootRingSlot,
+) -> RootRingSlotContent {
+    match read_root_ring_slot(
+        devices,
+        &parameters.region_devices,
+        &parameters.geometry,
+        &parameters.filesystem_identifier,
+        ring_slot,
+    ) {
+        RootRingSlotReading::SelfVerified(root) => RootRingSlotContent::SelfVerifiedRoot {
+            checkpoint_txg: root.checkpoint_txg.0,
+        },
+        RootRingSlotReading::Bad(_) => {
+            let device = parameters.region_devices
+                [usize::try_from(ring_slot.region).expect("区域号落在 3 以内")];
+            let offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing);
+            let root_slot_bytes =
+                usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
+            match PoolReader::read(devices, device, offset, root_slot_bytes) {
+                Some(bytes) if bytes.iter().all(|byte| *byte == 0) => RootRingSlotContent::AllZero,
+                Some(_) | None => RootRingSlotContent::Bad,
+            }
+        }
+    }
+}
+
+/// 一次挂载那一刻的定乙钉住（第 6 次重跑登记 W16、W17，第五节 E8）：交给 `mount_writable` 之前那份切段镜像里
+/// 读回全零的槽，各钉在它可能住过的最旧 txg。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct AllZeroPinsAtMount {
+    hole_txg: u64,
+    first_txg_of_the_new_instance: u64,
+    pinned: BTreeMap<RootRingSlot, u64>,
+    bad_slots_at_mount: u64,
+}
+
+/// W17：3S 个槽逐个读，交回读回全零的槽与读坏的槽数。槽用 txg 0..3S 经 `target_for_publish` 走一遍（A2：txg 与 u mod 3S 一一对应）。
+fn root_ring_slots_reading_back_all_zero_at_mount(
+    devices: &[(DeviceIdentity, SparseBlockDevice)],
+    parameters: &MakeFilesystemParameters,
+    ring_length: u64,
+) -> (BTreeSet<RootRingSlot>, u64) {
+    let mut all_zero = BTreeSet::new();
+    let mut bad_slots = 0u64;
+    for txg in 0..ring_length {
+        let ring_slot = target_for_publish(
+            CheckpointTxg(txg),
+            parameters.geometry.root_ring_slots_per_region,
+        );
+        match read_root_ring_slot_content(devices, parameters, ring_slot) {
+            RootRingSlotContent::SelfVerifiedRoot { .. } => {}
+            RootRingSlotContent::AllZero => {
+                all_zero.insert(ring_slot);
+            }
+            RootRingSlotContent::Bad => bad_slots += 1,
+        }
+    }
+    (all_zero, bad_slots)
+}
+
+/// W16：落到这一槽、小于这次挂载新实例第一个 txg 的最小 txg，在 [0, min(3S, 第一个 txg)) 里找；一个都没有的槽不钉。
+fn oldest_txg_an_all_zero_slot_can_have_held(
+    ring_slot: RootRingSlot,
+    first_txg_of_the_new_instance: u64,
+    ring_length: u64,
+    parameters: &MakeFilesystemParameters,
+) -> Option<u64> {
+    (0..ring_length.min(first_txg_of_the_new_instance)).find(|txg| {
+        target_for_publish(
+            CheckpointTxg(*txg),
+            parameters.geometry.root_ring_slots_per_region,
+        ) == ring_slot
+    })
+}
+
 fn mount_admission_label(admission: &MountSpaceAdmission) -> &'static str {
     match admission {
         MountSpaceAdmission::AdmittedBeforeAcquisition => "admitted_before_acquisition",
@@ -2165,6 +2300,10 @@
     workload_publishes: u64,
     tree_table_lag_seen_real: BTreeSet<(u64, u64)>,
     tree_table_lag_seen_every_publish: BTreeSet<(u64, u64)>,
+    /// 第 6 次重跑登记 E7：这次挂载钉着、还没解开的槽（值是 W16 的最旧 txg）；每次挂载整张换掉，mkfs 之后为空。
+    all_zero_pins: BTreeMap<RootRingSlot, u64>,
+    /// 这次挂载新实例的第一个 txg；mkfs 之后为 `None`。
+    first_txg_of_this_mount: Option<u64>,
 }
 
 impl HhRunState {
@@ -2184,7 +2323,22 @@
             versions: &self.versions,
             allocation_generations: &self.allocation_generations,
             injected_holes: &self.injected_holes,
+            all_zero_pins: &self.all_zero_pins,
+            first_txg_of_this_mount: self.first_txg_of_this_mount,
         });
+        // W18 (a)(b)：解开的槽从这次挂载的钉住表里拿掉，各计一次。
+        for ring_slot in reading
+            .all_zero_pins_released_by_write
+            .iter()
+            .chain(&reading.all_zero_pins_released_by_reread_only)
+        {
+            self.all_zero_pins.remove(ring_slot);
+        }
+        outcome.all_zero_pins_released_by_write +=
+            u64::try_from(reading.all_zero_pins_released_by_write.len()).expect("落在 u64 内");
+        outcome.all_zero_pins_released_by_reread_only +=
+            u64::try_from(reading.all_zero_pins_released_by_reread_only.len())
+                .expect("落在 u64 内");
         let txg = self.current.root.checkpoint_txg.0;
         let free_map_zero = self
             .allocator
@@ -2399,7 +2553,11 @@
         tree_table_lags_every_publish: BTreeSet::new(),
         checker_red_on_hole_states: 0,
         checker_judged_hole_states: 0,
+        all_zero_pin_mounts: Vec::new(),
+        all_zero_pins_released_by_write: 0,
+        all_zero_pins_released_by_reread_only: 0,
     };
+    let ring_length = E156_ROOT_RING_REGIONS * shape.slots_per_region;
     let mut devices = new_pool_devices();
     let genesis = make_filesystem(&parameters, &mut devices).expect("Hh mkfs");
     outcome.genesis_root_in_ring = readable_roots(
@@ -2477,6 +2635,8 @@
         workload_publishes: 0,
         tree_table_lag_seen_real: BTreeSet::new(),
         tree_table_lag_seen_every_publish: BTreeSet::new(),
+        all_zero_pins: BTreeMap::new(),
+        first_txg_of_this_mount: None,
     };
     state.lineage.note_version(&mut state.versions, &first);
     state.observe(
@@ -2530,6 +2690,12 @@
                 .note_version(&mut state.versions, &crashed.crashed_output);
             state.injected_holes.push(next_txg);
             let mut recovered_devices = devices_from_memory_pool(&crashed.cut_image);
+            let (slots_reading_back_all_zero_at_mount, bad_slots_at_mount) =
+                root_ring_slots_reading_back_all_zero_at_mount(
+                    &recovered_devices,
+                    &state.parameters,
+                    ring_length,
+                );
             let mounted = match mount_writable(&state.parameters, &mut recovered_devices) {
                 Ok(mounted) => mounted,
                 Err(error) => {
@@ -2571,6 +2737,27 @@
                 state.lineage.note_version(&mut state.versions, version);
             }
             let first_txg_of_the_new_instance = mounted.output.row_publish.root().checkpoint_txg.0;
+            // 第 6 次重跑登记 E8：钉住表整张换成这次挂载读回全零的槽（W16、W17）。
+            let all_zero_pins: BTreeMap<RootRingSlot, u64> = slots_reading_back_all_zero_at_mount
+                .iter()
+                .filter_map(|ring_slot| {
+                    oldest_txg_an_all_zero_slot_can_have_held(
+                        *ring_slot,
+                        first_txg_of_the_new_instance,
+                        ring_length,
+                        &state.parameters,
+                    )
+                    .map(|oldest_txg| (*ring_slot, oldest_txg))
+                })
+                .collect();
+            state.all_zero_pins.clone_from(&all_zero_pins);
+            state.first_txg_of_this_mount = Some(first_txg_of_the_new_instance);
+            let all_zero_pins_at_mount = AllZeroPinsAtMount {
+                hole_txg: next_txg,
+                first_txg_of_the_new_instance,
+                pinned: all_zero_pins,
+                bad_slots_at_mount,
+            };
             let publishes_in_the_mount = 1
                 + u64::try_from(mounted.output.warm_up_publishes.len()).expect("落在 u64 内")
                 + floor_raise_sequences
@@ -2660,6 +2847,21 @@
                 recovery.mount_admission,
                 recovery.mount_floor_raise_sequences,
             ));
+            emitter.emit(&format!(
+                "name=all_zero_pins_at_mount family={} {} hole_txg={} first_txg={} pinned=[{}] bad_slots_at_mount={}",
+                request.family,
+                shape.fields(),
+                all_zero_pins_at_mount.hole_txg,
+                all_zero_pins_at_mount.first_txg_of_the_new_instance,
+                all_zero_pins_at_mount
+                    .pinned
+                    .iter()
+                    .map(|(ring_slot, txg)| format!("({},{},{txg})", ring_slot.region, ring_slot.slot))
+                    .collect::<Vec<_>>()
+                    .join(","),
+                all_zero_pins_at_mount.bad_slots_at_mount,
+            ));
+            outcome.all_zero_pin_mounts.push(all_zero_pins_at_mount);
             outcome.recoveries.push(recovery);
             workload_index_in_segment = 0;
             state.cross_check_references_against_the_disk(&mut outcome);
@@ -4339,6 +4541,78 @@
     ));
 }
 
+/// 一格的定乙钉住簿记（第 6 次重跑登记第五节 E10、第六节 R1d）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct AllZeroPinCellTotals {
+    mounts: u64,
+    pins_installed: u64,
+    pinned_points: u64,
+    pinned_slots_peak: u64,
+    pinned_slots_end: u64,
+    threshold_lowered_points: u64,
+    released_by_write: u64,
+    released_by_reread_only: u64,
+    bad_slots_at_mount: u64,
+    bad_slots_still_pinned: u64,
+}
+
+fn all_zero_pin_cell_totals(outcome: &HhRunOutcome) -> AllZeroPinCellTotals {
+    let observations = &outcome.observations;
+    AllZeroPinCellTotals {
+        mounts: u64::try_from(outcome.all_zero_pin_mounts.len()).expect("落在 u64 内"),
+        pins_installed: outcome
+            .all_zero_pin_mounts
+            .iter()
+            .map(|mount| u64::try_from(mount.pinned.len()).expect("落在 u64 内"))
+            .sum(),
+        pinned_points: count_where(observations, |observation| {
+            observation.reading.all_zero_slots_still_pinned > 0
+        }),
+        pinned_slots_peak: observations
+            .iter()
+            .map(|observation| observation.reading.all_zero_slots_still_pinned)
+            .max()
+            .unwrap_or(0),
+        pinned_slots_end: observations.last().map_or(0, |observation| {
+            observation.reading.all_zero_slots_still_pinned
+        }),
+        threshold_lowered_points: count_where(observations, |observation| {
+            observation.reading.threshold_lowered_by_all_zero_pins
+        }),
+        released_by_write: outcome.all_zero_pins_released_by_write,
+        released_by_reread_only: outcome.all_zero_pins_released_by_reread_only,
+        bad_slots_at_mount: outcome
+            .all_zero_pin_mounts
+            .iter()
+            .map(|mount| mount.bad_slots_at_mount)
+            .sum(),
+        bad_slots_still_pinned: observations
+            .iter()
+            .map(|observation| observation.reading.bad_slots_still_pinned)
+            .sum(),
+    }
+}
+
+fn emit_all_zero_pin_cell(emitter: &mut Emitter, outcome: &HhRunOutcome) -> AllZeroPinCellTotals {
+    let totals = all_zero_pin_cell_totals(outcome);
+    emitter.emit(&format!(
+        "name=all_zero_pin_cell family={} {} mounts={} pins_installed={} pinned_points={} pinned_slots_peak={} pinned_slots_end={} threshold_lowered_points={} released_by_write={} released_by_reread_only={} bad_slots_at_mount={} bad_slots_still_pinned={}",
+        outcome.family,
+        outcome.shape.fields(),
+        totals.mounts,
+        totals.pins_installed,
+        totals.pinned_points,
+        totals.pinned_slots_peak,
+        totals.pinned_slots_end,
+        totals.threshold_lowered_points,
+        totals.released_by_write,
+        totals.released_by_reread_only,
+        totals.bad_slots_at_mount,
+        totals.bad_slots_still_pinned,
+    ));
+    totals
+}
+
 /// PQ1 一段历史里「造出了洞」的判定（登记第六节 6.1：② = t + 1 且 ③ 没有）。
 fn premise_one_reproduced(outcome: &HhRunOutcome) -> bool {
     !outcome.recoveries.is_empty()
@@ -4661,6 +4935,7 @@
     let mut grid_totals = HoleHistoryVerdictTotals::default();
     let mut every_publish_tree_table_lag_mismatch_cells = 0u64;
     let mut not_fitting = 0u64;
+    let mut all_zero_pin_cells: Vec<AllZeroPinCellTotals> = Vec::new();
     for slots_per_region in E156_SLOTS_PER_REGION_SAMPLING_POINTS {
         let expected_lags = anchor_model::tree_table_reclaim_lags(
             slots_per_region,
@@ -4723,38 +4998,53 @@
                 every_publish_tree_table_lag_mismatch_cells +=
                     totals.tree_table_lag_every_publish_mismatch;
                 emit_cell_checks(&mut emitter, &outcome, &totals);
+                all_zero_pin_cells.push(emit_all_zero_pin_cell(&mut emitter, &outcome));
                 grid_totals.add(totals);
                 outcomes.push(outcome);
             }
         }
     }
 
-    // PC-分配器：Hh(0, —, S = 4, ρ = 1) 换回不装根环表的分配器，环转过一圈之后 A10 应判红。
-    let positive_control_allocator = {
-        let shape = shape_of(4, WorkloadDensity::EveryPublishOverwrites, 0, None);
-        let mut request = hh_request("PC-allocator", shape, anchor_cell_for(&anchors, 4, 0, None));
-        request.allocator_source = HhAllocatorSource::WithoutRootRingPositiveControl;
-        request.emit_positive_control_no_reclaim = false;
-        run_hh_history(&mut emitter, &request)
-    };
-    let expected_lags_four =
-        anchor_model::tree_table_reclaim_lags(4, anchors[&4].without_holes.end);
-    let positive_control_allocator_seen = positive_control_allocator.tree_table_lags_real
-        != expected_lags_four
-        && positive_control_allocator
-            .observations
-            .iter()
-            .all(|observation| !observation.reading.root_ring_installed);
+    // PC-钉门槛（第 5 次重跑登记 5.3、5.4、5.5 E2；替 PC-分配器）：格表里本来就跑的 Hh(1, 后, S, ρ) 六格，洞让 txg t − 3S 的旧根
+    // 留在 t 的槽里，真实分配器的回收门槛钉在它上面直到 t + 3S（K15、L1），释放代 6 的树表滞后 = 6S − 1。把这一格交给同一个
+    // k = 0 的 A10 判定（期望 = 同一 S 的 {3S}）：判定判「不符」只证判定不是恒绿，「含 6S − 1」证量法读到了真实分配器的推迟，两句各报各的。
+    // 六格不进 grid_totals；6S − 1 用本地的 S 现算（2 × 区域数 × S − 1）。
+    let mut pinned_floor_cells = 0u64;
+    let mut pinned_floor_seen_real_cells = 0u64;
+    let mut pinned_floor_seen_every_cells = 0u64;
+    for outcome in outcomes.iter().filter(|outcome| {
+        outcome.shape.hole_count == 1 && outcome.shape.position == Some(HolePosition::Back)
+    }) {
+        let slots_per_region = outcome.shape.slots_per_region;
+        let expected_lags = anchor_model::tree_table_reclaim_lags(
+            slots_per_region,
+            anchors[&slots_per_region].without_holes.end,
+        );
+        let totals = judge_hole_history_outcome(outcome, Some(&expected_lags));
+        let bound = 2 * E156_ROOT_RING_REGIONS * slots_per_region - 1;
+        let contains_bound_real = outcome.tree_table_lags_real.contains(&bound);
+        let contains_bound_every = outcome.tree_table_lags_every_publish.contains(&bound);
+        let seen_real = totals.tree_table_lag_real_mismatch == 1 && contains_bound_real;
+        let seen_every = totals.tree_table_lag_every_publish_mismatch == 1 && contains_bound_every;
+        pinned_floor_cells += 1;
+        pinned_floor_seen_real_cells += u64::from(seen_real);
+        pinned_floor_seen_every_cells += u64::from(seen_every);
+        emitter.emit(&format!(
+            "name=pc_pinned_floor {} expected={} lags_real={} lags_every={} judge_real_mismatch={} judge_every_mismatch={} bound={bound} contains_bound_real={contains_bound_real} contains_bound_every={contains_bound_every} seen_real={seen_real} seen_every={seen_every}",
+            outcome.shape.fields(),
+            set_text(&expected_lags),
+            set_text(&outcome.tree_table_lags_real),
+            set_text(&outcome.tree_table_lags_every_publish),
+            totals.tree_table_lag_real_mismatch,
+            totals.tree_table_lag_every_publish_mismatch,
+        ));
+    }
     emitter.emit(&format!(
-        "name=pc_allocator a10_lags_real={} expected={} root_ring_installed_points={} seen={positive_control_allocator_seen}",
-        set_text(&positive_control_allocator.tree_table_lags_real),
-        set_text(&expected_lags_four),
-        positive_control_allocator
-            .observations
-            .iter()
-            .filter(|observation| observation.reading.root_ring_installed)
-            .count()
+        "name=pc_pinned_floor_summary cells={pinned_floor_cells} seen_real_cells={pinned_floor_seen_real_cells} seen_every_cells={pinned_floor_seen_every_cells}"
     ));
+    // 登记写死六格：S ∈ {4, 8, 16} × ρ ∈ {1, 1/4}，全都排得下。
+    let pinned_floor_seen =
+        pinned_floor_cells == 6 && pinned_floor_seen_every_cells == pinned_floor_cells;
 
     // PC-多扣：每个 (S, ρ) 的 Hh(0, —) 与 Hh(2, 后)，四个组合的期末值都要 > 0。
     let mut over_withheld_controls_void = 0u64;
@@ -4963,11 +5253,28 @@
         front_one_slot.join(","),
         not_on_holes.join(","),
     ));
+    // 第 6 次重跑登记 E10：定乙钉住的汇总（逐格的和）。
+    let sum_of = |pick: fn(&AllZeroPinCellTotals) -> u64| -> u64 {
+        all_zero_pin_cells.iter().map(pick).sum()
+    };
+    emitter.emit(&format!(
+        "name=all_zero_pin_summary cells={} cells_with_pins={} pins_installed={} threshold_lowered_points={} released_by_reread_only={} bad_slots_at_mount={} bad_slots_still_pinned={}",
+        all_zero_pin_cells.len(),
+        all_zero_pin_cells
+            .iter()
+            .filter(|cell| cell.pins_installed > 0)
+            .count(),
+        sum_of(|cell| cell.pins_installed),
+        sum_of(|cell| cell.threshold_lowered_points),
+        sum_of(|cell| cell.released_by_reread_only),
+        sum_of(|cell| cell.bad_slots_at_mount),
+        sum_of(|cell| cell.bad_slots_still_pinned),
+    ));
 
     // 作废与停机（第十一节）。
     let cells_run = outcomes.len();
     emitter.emit(&format!(
-        "name=verdict stage=second cells_run={cells_run} cells_not_fitting={not_fitting} truncated_cells={} v1_over_withheld_controls_void={over_withheld_controls_void} v2_a10_every_mismatch_cells={every_publish_tree_table_lag_mismatch_cells} v2_a12_every_nonzero_points={} v2_a14_mismatch_points={} v2_a15_every_below_points={} v2_pc_allocator_seen={positive_control_allocator_seen} v2_pc_c2_seen={positive_control_crash_seen} v6_a11_mismatch_points={} v6_devices_unequal_points={} v6_pc_hole_not_seen_cells={hole_controls_not_seen} v16_judge_cases_wrong={judge_wrong} v17_a13_every_nonzero_points={} s1_failures={} s1h_root_ring_missing_points={} s11_a10_real_mismatch_cells={} s11_a12_real_nonzero_points={} s11_a15_real_below_points={} s11_k10_violation_points={} s12_g_adm_nonzero={} k8_mismatches={} referenced_but_free_points={} abandoned_or_unknown_root_points={} anchor_rows_missing={}",
+        "name=verdict stage=second cells_run={cells_run} cells_not_fitting={not_fitting} truncated_cells={} v1_over_withheld_controls_void={over_withheld_controls_void} v2_a10_every_mismatch_cells={every_publish_tree_table_lag_mismatch_cells} v2_a12_every_nonzero_points={} v2_a14_mismatch_points={} v2_a15_every_below_points={} v2_pc_pinned_floor_seen={pinned_floor_seen} v2_pc_c2_seen={positive_control_crash_seen} v6_a11_mismatch_points={} v6_devices_unequal_points={} v6_pc_hole_not_seen_cells={hole_controls_not_seen} v16_judge_cases_wrong={judge_wrong} v17_a13_every_nonzero_points={} s1_failures={} s1h_root_ring_missing_points={} s11_a10_real_mismatch_cells={} s11_a12_real_nonzero_points={} s11_a15_real_below_points={} s11_k10_violation_points={} s12_g_adm_nonzero={} k8_mismatches={} referenced_but_free_points={} abandoned_or_unknown_root_points={} anchor_rows_missing={}",
         outcomes.iter().filter(|outcome| outcome.truncation.is_some()).count(),
         grid_totals.every_publish_nonzero_without_holes,
         grid_totals.threshold_anchor_mismatches,
@@ -5004,19 +5311,20 @@
 #[cfg(test)]
 mod tests {
     //! K1（新池新建文件之后的记账三项）、G27（`referenced_slots`）、R8（U7）、R3 的判别力（U8）、R-3 闭式（U10、U12）的最小单测，
-    //! 与第 4 次重跑登记第九节的 U11、U13–U18。
+    //! 与第 4 次重跑登记第九节的 U11、U13–U18、第 5 次重跑登记第九节的 U19。
     use super::{
         accounting_row_slots, allocated_minus_deferred_matches_referenced,
         allocated_minus_deferred_mismatches_referenced, anchor_cell_for, anchor_cells_of,
         corrupt_device_zero_accounting, e156_allocation_record_tree_new_node_count,
         e156_closed_form_expected, e156_existing_leaf_positions, e156_touched_leaf_positions,
         hh_request, judge_geometry_sensitivity, judge_growth_with_holes,
-        judge_peak_against_threshold, judge_positive_control_cases, memory_pool_snapshot,
-        parameters, q7c_self_test, referenced_slots, run_forward_rollback_scenario, run_hh_history,
-        run_rollback_to_the_oldest_candidate, self_release_slots_of_this_publish,
-        synthetic_base_with_zero_deferred, AnchorCellsOfOneSlotsPerRegion, Arm, Emitter,
-        GeometrySensitivity, GrowthWithHoles, HhCellShape, HhRunOutcome, HolePosition,
-        PeakJudgement, ReclaimTiming, WorkloadDensity, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
+        judge_hole_history_outcome, judge_peak_against_threshold, judge_positive_control_cases,
+        memory_pool_snapshot, parameters, q7c_self_test, referenced_slots,
+        run_forward_rollback_scenario, run_hh_history, run_rollback_to_the_oldest_candidate,
+        self_release_slots_of_this_publish, synthetic_base_with_zero_deferred,
+        AnchorCellsOfOneSlotsPerRegion, Arm, Emitter, GeometrySensitivity, GrowthWithHoles,
+        HhCellShape, HhRunOutcome, HolePosition, PeakJudgement, ReclaimTiming, WorkloadDensity,
+        FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
     };
     use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
     use singlefs_core::allocator::{
@@ -5565,7 +5873,7 @@
     }
 
     /// U14（第 4 次重跑登记第九节）：Hh(0, —, S = 4, ρ = 1) 跑到 txg 48：实与每两个时点 A10 的滞后全是 12、A12 每个观测点
-    /// Q1a = Q1b = 0、每个观测点分配器装着根环表。M35（分配器退回不装根环表）、M42（「每」的门槛晚一次）在这里红。
+    /// Q1a = Q1b = 0、每个观测点分配器装着根环表。M35（第 5 次重跑登记第九节改写：装着根环表而记根时不回收）、M42（「每」的门槛晚一次）在这里红。
     #[test]
     fn hh_without_holes_at_four_slots_per_region_reclaims_each_tree_table_one_ring_length_after_release(
     ) {
@@ -5726,6 +6034,120 @@
         }
     }
 
+    /// U20（第 6 次重跑登记第九节；钉的值出自第七节 K16、L6）：Hh(1, 前, S = 4, ρ = 1)，与 U17 同一段历史。定乙：洞在 txg 5，
+    /// 那一槽挂载时读回全零、钉在 5；txg 16 环里最旧有效根是 6、门槛照钉住取 5；txg 17 这个进程写掉那一槽，门槛回到 6。
+    /// M45（门槛不并进全零槽）、M46（锚点模型不并进）、M47（写掉也不解开）在这里红。
+    #[test]
+    fn hh_front_hole_at_four_slots_per_region_pins_the_threshold_to_the_hole_txg_until_the_process_rewrites_its_slot(
+    ) {
+        let anchors = anchors_for_four_slots_per_region();
+        let outcome = run_the_hole_history_of_one_shape(&anchors, 1, Some(HolePosition::Front));
+        let recovery = outcome.recoveries.first().expect("造了一个洞");
+        assert_eq!(recovery.hole_txg, 5, "洞在 txg 5");
+        assert_eq!(
+            recovery.hole_slot_content, "never_written",
+            "txg 5 的槽从没写过"
+        );
+        assert_eq!(outcome.all_zero_pin_mounts.len(), 1, "一次挂载");
+        let txg_five_slot = super::RootRingSlot { region: 2, slot: 1 };
+        assert_eq!(
+            outcome.all_zero_pin_mounts[0].pinned,
+            BTreeMap::from([(txg_five_slot, 5)]),
+            "W16、W17：这次挂载恰好钉 txg 5 那一槽（区域 2、槽 1），钉在 5"
+        );
+        for (txg, expected_threshold) in [(15u64, 4u64), (16, 5), (17, 6)] {
+            assert_eq!(
+                observation_at(&outcome, txg).reading.ring_threshold,
+                expected_threshold,
+                "K16：txg {txg} 门槛 = {expected_threshold}"
+            );
+        }
+        let anchor = anchor_cell_for(&anchors, 4, 1, Some(HolePosition::Front));
+        assert_eq!(
+            anchor.observed_row(16).expect("锚点有 txg 16").floor,
+            5,
+            "L6：锚点模型 txg 16 门槛 = 5"
+        );
+        assert_eq!(
+            anchor.observed_row(17).expect("锚点有 txg 17").floor,
+            6,
+            "锚点模型 txg 17 门槛 = 6"
+        );
+        for observation in &outcome.observations {
+            assert_eq!(
+                observation.reading.device_zero.reclaim_rule_violations, 0,
+                "K10：txg {}",
+                observation.txg
+            );
+            assert_eq!(
+                observation.over_withheld(Arm::FloorRule, ReclaimTiming::Real),
+                observation.over_withheld(Arm::FloorRule, ReclaimTiming::EveryPublish),
+                "F24：实、每两列 Q1a 相等，txg {}",
+                observation.txg
+            );
+        }
+        assert_eq!(
+            outcome.all_zero_pins_released_by_write, 1,
+            "txg 17 写掉那一槽"
+        );
+        assert_eq!(
+            outcome.all_zero_pins_released_by_reread_only, 0,
+            "没有重读解开"
+        );
+        assert_eq!(
+            outcome.all_zero_pin_mounts[0].bad_slots_at_mount, 0,
+            "挂载那一刻没有读坏的槽"
+        );
+    }
+
+    /// U19（第 5 次重跑登记第九节）：Hh(1, 后, S = 4, ρ = 1)，与 U15 同一段历史。① 实、每两份滞后集合都含 6S − 1 = 23
+    /// （K15、L1：洞让 txg 4 的旧根压着门槛到 txg 28，释放代 6 的树表到那一次发布才可发，滞后 = 28 + 1 − 6）；
+    /// ② 交 k = 0 的 A10 判定、期望 {12} 时，实、每两项都判「不符」；③ 期望换成这一格自己量出来的两份集合时，两项各自判「符」
+    /// （判定不是恒红）。M44（「实」列不读真实分配器的空闲位图、按 3S 的时间表记）在 ① 红。
+    #[test]
+    fn hh_back_hole_at_four_slots_per_region_delays_the_tree_table_reclaim_to_six_ring_slots_minus_one_and_the_a10_judgement_sees_it(
+    ) {
+        let outcome = run_the_hole_history_of_one_shape(
+            &anchors_for_four_slots_per_region(),
+            1,
+            Some(HolePosition::Back),
+        );
+        assert!(
+            outcome.tree_table_lags_real.contains(&23),
+            "K15：实列滞后集合含 6S − 1 = 23，量到 {:?}",
+            outcome.tree_table_lags_real
+        );
+        assert!(
+            outcome.tree_table_lags_every_publish.contains(&23),
+            "K15：每列滞后集合含 6S − 1 = 23，量到 {:?}",
+            outcome.tree_table_lags_every_publish
+        );
+        let three_slots_per_ring_length = BTreeSet::from([12]);
+        let against_the_no_hole_expectation =
+            judge_hole_history_outcome(&outcome, Some(&three_slots_per_ring_length));
+        assert_eq!(
+            against_the_no_hole_expectation.tree_table_lag_real_mismatch, 1,
+            "期望 {{12}}：实列判不符"
+        );
+        assert_eq!(
+            against_the_no_hole_expectation.tree_table_lag_every_publish_mismatch, 1,
+            "期望 {{12}}：每列判不符"
+        );
+        let real_own = outcome.tree_table_lags_real.clone();
+        let every_own = outcome.tree_table_lags_every_publish.clone();
+        assert_eq!(
+            judge_hole_history_outcome(&outcome, Some(&real_own)).tree_table_lag_real_mismatch,
+            0,
+            "期望 = 实列自己量出来的集合：实列判符"
+        );
+        assert_eq!(
+            judge_hole_history_outcome(&outcome, Some(&every_own))
+                .tree_table_lag_every_publish_mismatch,
+            0,
+            "期望 = 每列自己量出来的集合：每列判符"
+        );
+    }
+
     /// U18（第 4 次重跑登记第九节，PC-判定器）：判定函数在合成用例上逐条判对。M43（「> 2」写成「≥ 2」）在这里红。
     #[test]
     fn judgement_of_peak_delta_against_threshold_and_geometry_sensitivity_on_synthetic_cases() {
````
