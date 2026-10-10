# 删除批第 31 批合入的改动原样（2026-10-07）

## 实现员 n：crates.patch（/home/fy5090/.local/share/singlefs/work/m3-batch31/n/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -2447,7 +2447,8 @@
         });
     }
 
-    /// 走读不往下读的孩子（墓碑树、谱系树、inode 树内部条目里类型段不认得的，与孩子自证成类型段之外另一类的）：每一份副本照判
+    /// 走读不往下读的孩子（墓碑树、谱系树、inode 树内部条目里类型段不认得的，与孩子自证成类型段之外另一类的；墓碑树、谱系树层级 1 节点里
+    /// 类型段 0 的，第 31 批 F-L0）：每一份副本照判
     /// I-2.1（`.claude/kb/invariants.md` I-2.1「任一被引用的块，其校验和与内容匹配」；判决 `research/prompts/m3-batch27-code-r1-main-verification.md`
     /// 第二节 H-J3a、H-J3b 两行与第三节第 1 条）。一份对不对得上按
     /// [`copy_matches_its_location_checksum_at_the_length_its_header_names`] 判；两份都对不上记走读失败（I-4.8、I-7.2、I-7.4 照旧跟着）。
@@ -2479,6 +2480,8 @@
     }
 
     /// 墓碑树、谱系树里类型段 0 那条条目下面的码 2 节点：层级要是父层级 − 1，条目宽 120 才往下走（[`Self::walk_container_tree_root`]）。
+    /// 父节点层级 ≤ 1 时孩子不往下走、记走读失败；孩子每一份副本照判 I-2.1（判决 `research/prompts/m3-batch29-code-r1-main-verification.md`
+    /// 第二节 K2 L1 第三支那一行、第三节第 1 条，攻方改法 F-L0），引用照旧不登记。
     #[allow(
         clippy::too_many_arguments,
         reason = "条目、孩子指针与树的三样都是这一步要的输入，各有名字与类型，不为凑短签名收成结构体"
@@ -2494,6 +2497,10 @@
         name: &str,
     ) {
         if node.level <= 1 {
+            self.judge_every_copy_of_a_child_the_walk_does_not_go_into(
+                child,
+                &format!("{name}内部条目的子指针"),
+            );
             self.walk_failures.push(format!(
                 "{name}层级 {} 的节点有类型段 0 的条目：孩子会是层级 0 的码 2 节点，这两棵树里没有条款定的形状",
                 node.level
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -1472,7 +1472,8 @@
 };
 
 // walk.rs read_index_node：读一个码 2 节点并判头、出生身份、树 ID（I-1.3）与 key（I-1.1）
-// key_range_reading：0 贴紧首末条目、1 多层树的子树覆盖（叶贴紧、内部只判分隔 key）、2 按位置（只判分隔 key）
+// key_range_reading：0 贴紧首末条目、1 多层树的子树覆盖（叶贴紧、内部只判分隔 key）、2 按位置（只判分隔 key）、
+// 3 按位置读 extent 树上段的根（层级 0 的零条目根只判 key 宽，其余同 2；walk.rs 的 PrescribedByThePositionOfTheExtentUpperRootJudgedByTheCaller）
 fn w_read_index_node(p: Ptr, expected_tree: vec2<u32>, schema: u32, key_range_reading: u32, registered: bool, fsid_low: vec2<u32>) -> ReadNode {
     var out: ReadNode;
     out.ok = false;
@@ -1511,6 +1512,9 @@
     var keys_hold = false;
     if (key_range_reading == 0u || (key_range_reading == 1u && n.level == 0u)) {
         keys_hold = w_check_index_node_keys(n, schema);
+    } else if (key_range_reading == 3u && n.level == 0u && n.entry_count == 0u) {
+        // extent 树上段的根兼叶删完第一个文件之后可以一条条目都没有（D8 已定项 16 第 5 条）：只判 key 宽，不判节点不空
+        keys_hold = schema_key_width(schema) == n.key_width;
     } else {
         keys_hold = w_check_internal_node_separators(n, schema);
     }
@@ -2182,7 +2186,12 @@
     var enter = true;
     loop {
         if (enter) {
-            let read = w_read_index_node(pending_pointer, tree, 3u, 2u, true, fsid_low);
+            // 读根（pending_level 还是 NONE）照 walk.rs walk_extent_upper_node 读根那一种读法（key_range_reading 3），读孩子照按位置（2）
+            var key_range_reading_of_this_node = 2u;
+            if (pending_level == NONE) {
+                key_range_reading_of_this_node = 3u;
+            }
+            let read = w_read_index_node(pending_pointer, tree, 3u, key_range_reading_of_this_node, true, fsid_low);
             if (!read.ok) {
                 if (depth == 0u) { break; }
                 extent_upper_frames[depth - 1u].child_index = extent_upper_frames[depth - 1u].child_index + 1u;
@@ -3169,6 +3178,9 @@
             w_judge(identity_invariant, is_zero64(rd64(n.base, o + 8u)) && is_zero64(rd64(n.base, o + 18u)) && is_zero64(rd64(n.base, o + 26u))
                 && child_is_a_code_two_node_or_unreadable);
             if (n.level <= 1u) {
+                // 孩子不往下走、记走读失败；每一份副本照判 I-2.1（walk.rs walk_the_internal_node_below_a_container_tree_entry 的 F-L0，
+                // 判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第三节第 1 条），引用照旧不登记
+                w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);
                 w_failures = w_failures + 1u;
                 continue;
             }
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -40,6 +40,11 @@
 //! 第 30 批（删除线）：inode 叶容器两份第一条记录 blocks 加 1、盘 1 那一份再换成码 2 节点的字节（F-W：另一份按类型段那一类自证过，照类型段走它）
 //! 一条流钉 CPU 红 I-9.2 与 I-9.15、不红 I-3.1、I-3.11、I-5.1；发布的根与它的记录里实例表指针头部诞生代号、实例、尾段各加 1（F-IT）三条流
 //! 钉 CPU 只红 I-1.2；GPU 都逐位跟上。
+//! 第 31 批（删除线）：墓碑树根条目类型段 0、身份清零、两条位置条目校验和都坏（F-L0）、类型段 7 两条都坏、类型段 0 指容器第一条坏、
+//! 层级 2 根条目类型段 1 指节点第一条坏（容器树四处 `w_judge_every_copy_of_a_child_the_walk_does_not_go_into`）各一条流；墓碑容器带记录宽违例、
+//! 另一份自证成码 2 时类型段 1 与类型段 0 两形（容器树两处 F-W）各一条流，都整份比；inode 树根条目类型段 0、另一份自证成码 2 一条钉 GPU 记判不了
+//! （类型段 0 的内部节点）、回落 CPU 之后同判；候选根自己那张实例表链 257 片、最新根的链途经它的一片一条钉 GPU 扫描内核记判不了（站点 37 / 38）、
+//! 回落 CPU 之后同判。
 //! 第 35 批：由记录造出来的两类版本取带末条标志那条记录的新根段（三条记录的发布第一条写成上一版的根：下一次挂载那一版叠两条记录罩住同一个槽、
 //! 挂载之后由记录施加出来的那一版各一条），树表指针整条全零照 core 跟过去（根与它的记录、由记录施加出来的那一版的记录各一条），都整份比。
 //! 第 36 批：下一次挂载先施加的那一版照 I-4.1 那一份重放施加的第一次发布认（改法 R1）——三条记录的发布首条没落（断号）、
@@ -9653,3 +9658,636 @@
         mismatch_listing(&comparison)
     );
 }
+
+// ─── 第 31 批（删除线）：容器树四处走读不往下读的孩子（含 F-L0）、容器树两处 F-W、inode 类型段 0 另一份自证成码 2、候选根链途经已走过的片 ───
+
+/// F-L0（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第二节 K2 L1 第三支那一行、第三节第 1 条；harness 坏镜像
+/// `a_tombstone_tree_entry_of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants`
+/// 同一形，录成流）：删除中间态的流上墓碑树根（层级 1）第一条条目类型段 1 → 0、身份三段清零、子指针两条位置条目的整单元校验和各翻一字节，
+/// 重封到根；墓碑容器不动。
+fn record_deletion_with_the_tombstone_root_entry_of_type_zero_and_no_identity_over_two_wrong_location_checksums(
+    tag: &str,
+) -> RecordedFlow {
+    record_deletion_with_a_resealed_field(
+        tag,
+        "deletion-tombstone-root-entry-type-zero-no-identity-two-location-checksums-wrong",
+        &is_the_tombstone_tree_root,
+        &|bytes| {
+            set_the_type_of_the_first_entry(bytes, 0);
+            clear_the_identity_of_the_first_entry_but_its_type(bytes);
+            for location in POINTER_LOCATION_ENTRY_OFFSETS {
+                flip_a_byte_of_a_location_checksum_in_the_first_entry(bytes, location);
+            }
+        },
+    )
+}
+
+/// 第 31 批 F-L0：孩子两份按头里的码 3 都对不上、不是另一类，走类型段 0 那一支；身份引用那一半成立，层级 1 的节点下面不往下走、记走读失败，
+/// 每一份副本照判 I-2.1：CPU 池级 checker 红 I-2.1、I-4.8、I-7.2、I-7.4（第 30 批这一支只记走读失败、I-2.1 不红）；GPU 走读内核
+/// `w_walk_container_tree_root` 类型段 0、`n.level <= 1u` 那一处同改，每个状态逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_child_whose_two_location_checksums_are_wrong(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_child_whose_two_location_checksums_are_wrong",
+        &record_deletion_with_the_tombstone_root_entry_of_type_zero_and_no_identity_over_two_wrong_location_checksums(
+            "crash-judge-gpu-i21-tombstone-root-entry-type-zero-no-identity-two-location-checksums-wrong",
+        ),
+        &["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
+    );
+}
+
+/// 墓碑树那一侧的 H-J3a（harness 坏镜像
+/// `a_tombstone_tree_entry_with_an_unregistered_type_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants`
+/// 同一形，录成流）：删除中间态的流上墓碑树根第一条条目类型段 1 → 7、子指针两条位置条目的整单元校验和各翻一字节，重封到根。
+fn record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_over_two_wrong_location_checksums(
+    tag: &str,
+) -> RecordedFlow {
+    record_deletion_with_a_resealed_field(
+        tag,
+        "deletion-tombstone-root-entry-type-seven-two-location-checksums-wrong",
+        &is_the_tombstone_tree_root,
+        &|bytes| {
+            set_the_type_of_the_first_entry(bytes, UNREGISTERED_CONTAINER_TREE_ENTRY_TYPE);
+            for location in POINTER_LOCATION_ENTRY_OFFSETS {
+                flip_a_byte_of_a_location_checksum_in_the_first_entry(bytes, location);
+            }
+        },
+    )
+}
+
+/// 容器树类型段不认得那一支（`w_walk_container_tree_root` 里 `entry_type != record_type` 那一处）：CPU 红 I-9.23、I-2.1、I-4.8、I-7.2、I-7.4，
+/// GPU 每个状态逐位相同（判决同一份第二节 K1 L1 测试尖度那一行：容器树这一处调用没有对拍流）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong",
+        &record_deletion_with_the_tombstone_root_entry_of_an_unregistered_type_over_two_wrong_location_checksums(
+            "crash-judge-gpu-i21-tombstone-root-entry-type-seven-two-location-checksums-wrong",
+        ),
+        &["I-9.23", "I-2.1", "I-4.8", "I-7.2", "I-7.4"],
+    );
+}
+
+/// 墓碑树那一侧的 H-J3b（harness 坏镜像
+/// `a_tombstone_tree_entry_of_type_zero_over_a_container_whose_first_location_checksum_is_wrong_reddens_the_checksum_invariant` 同一形，录成流）：
+/// 删除中间态的流上墓碑树根第一条条目类型段 1 → 0、子指针第一条位置条目的整单元校验和翻一字节，重封到根；墓碑容器不动。
+fn record_deletion_with_the_tombstone_root_entry_of_type_zero_over_a_wrong_first_location_checksum(
+    tag: &str,
+) -> RecordedFlow {
+    record_deletion_with_a_resealed_field(
+        tag,
+        "deletion-tombstone-root-entry-type-zero-first-location-checksum-wrong",
+        &is_the_tombstone_tree_root,
+        &|bytes| {
+            set_the_type_of_the_first_entry(bytes, 0);
+            flip_a_byte_of_a_location_checksum_in_the_first_entry(
+                bytes,
+                POINTER_LOCATION_ENTRY_OFFSETS[0],
+            );
+        },
+    )
+}
+
+/// 容器树类型段 0、孩子是另一类那一支（`class_of_the_child_of_an_internal_entry != NONE` 那一处）：第二份自证成码 3，不往下走；
+/// 第一份对不上，照判 I-2.1：CPU 红 I-9.23、I-2.1、I-4.8、I-7.4（第二份对得上、不记走读失败），GPU 每个状态逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_whose_first_location_checksum_is_wrong(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_whose_first_location_checksum_is_wrong",
+        &record_deletion_with_the_tombstone_root_entry_of_type_zero_over_a_wrong_first_location_checksum(
+            "crash-judge-gpu-i21-tombstone-root-entry-type-zero-first-location-checksum-wrong",
+        ),
+        &["I-9.23", "I-2.1", "I-4.8", "I-7.4"],
+    );
+}
+
+/// 容器树类型段 = 记录类型、孩子是另一类那一支（`class_of_the_child_of_a_container_entry != NONE` 那一处）：
+/// [`record_a_tombstone_tree_grown_to_level_two`] 的流上层级 2 的墓碑树根第一条条目类型段 0 → 1、子指针第一条位置条目的整单元校验和翻一字节，
+/// 重封到根。孩子是层级 1 的码 2 节点：第一份按 16 KiB 对不上，第二份自证成码 2、没有一份是码 3，不往下走、按一槽登记引用。
+fn record_a_tombstone_tree_of_level_two_whose_first_root_entry_names_its_node_as_a_container_over_a_wrong_first_location_checksum(
+    tag: &str,
+) -> RecordedFlow {
+    let name = "tombstone-tree-of-level-two-first-root-entry-names-its-node-as-a-container-first-location-checksum-wrong";
+    let mut flow = record_a_tombstone_tree_grown_to_level_two(tag, name, false);
+    let targets = last_unit_copies(&flow, name, &is_a_tombstone_tree_root_of_level_two, false);
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &targets, &|bytes| {
+        set_the_type_of_the_first_entry(bytes, PACKED_RECORD_TYPE_TOMBSTONE);
+        flip_a_byte_of_a_location_checksum_in_the_first_entry(
+            bytes,
+            POINTER_LOCATION_ENTRY_OFFSETS[0],
+        );
+    });
+    flow
+}
+
+/// 见上：CPU 红 I-9.23、I-2.1（第一份那一份；第二份对得上、不记走读失败），GPU 每个状态逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_whose_first_location_checksum_is_wrong(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_whose_first_location_checksum_is_wrong",
+        &record_a_tombstone_tree_of_level_two_whose_first_root_entry_names_its_node_as_a_container_over_a_wrong_first_location_checksum(
+            "crash-judge-gpu-i21-level-two-entry-names-a-node-first-location-checksum-wrong",
+        ),
+        &["I-9.23", "I-2.1"],
+    );
+}
+
+/// 码 3 打包记录单元头里记录宽（2 字节）的偏移（D18（块里携带什么信息） 已定项 11：记录数 69、记录宽 71）。
+const PACKED_UNIT_RECORD_WIDTH_OFFSET: usize = 71;
+
+/// 墓碑容器头里的记录宽 40 → 39：I-1.7 记录宽那一步判红（重封归调用方）。
+fn set_the_record_width_of_a_tombstone_container_one_below(bytes: &mut [u8]) {
+    let width = read_u16_at(bytes, PACKED_UNIT_RECORD_WIDTH_OFFSET);
+    assert_eq!(
+        usize::from(width),
+        TOMBSTONE_RECORD_BYTES,
+        "墓碑容器的记录宽是登记的 40"
+    );
+    bytes[PACKED_UNIT_RECORD_WIDTH_OFFSET..PACKED_UNIT_RECORD_WIDTH_OFFSET + 2]
+        .copy_from_slice(&(width - 1).to_le_bytes());
+}
+
+/// 删除中间态的流上墓碑容器两份的记录宽 40 → 39（重封、沿引用链补到根），再把盘序大的那一份的前 16 KiB 换成墓碑树根（码 2）的原样字节，
+/// 墓碑树根第一条条目子指针第二条位置条目的整单元校验和换成那 16 KiB 的 CRC、`change_the_first_entry` 再改一处，重封到根
+/// （攻方 `research/prompts/m3-batch28-code-r1-opus-output.md` 第 1.1 节墓碑那一形；harness 坏镜像
+/// `a_tombstone_container_with_a_wrong_record_width_beside_a_copy_verifying_as_a_node_reddens_the_packed_container_invariant_on_either_device`
+/// 位置条目 1 那一形，录成流）。盘序小的那一份是码 3、带违例、按它的位置条目自证得过。
+fn record_deletion_with_a_wrong_record_width_in_the_tombstone_container_and_its_second_copy_holding_the_root_bytes(
+    tag: &str,
+    name: &'static str,
+    change_the_first_entry: &dyn Fn(&mut [u8]),
+) -> RecordedFlow {
+    let mut flow = record_new_pool_through_the_first_deletion_batch(tag, name);
+    let container_copies = last_unit_copies(&flow, name, &is_a_tombstone_container, false);
+    corrupt_units_and_reseal_up_to_the_root(
+        &mut flow,
+        &container_copies,
+        &set_the_record_width_of_a_tombstone_container_one_below,
+    );
+    let second_container_copy =
+        if flow.writes[container_copies[0]].device.0 < flow.writes[container_copies[1]].device.0 {
+            container_copies[1]
+        } else {
+            container_copies[0]
+        };
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
+    let second_location = first_entry_of_a_container_number_node()
+        + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
+        + POINTER_LOCATION_ENTRY_OFFSETS[1];
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
+    let checksum_of_the_node_bytes = crc32c(&node_bytes);
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
+        let field = second_location + LOCATION_ENTRY_UNIT_CHECKSUM_OFFSET;
+        bytes[field..field + 4].copy_from_slice(&checksum_of_the_node_bytes.to_le_bytes());
+        change_the_first_entry(bytes);
+    });
+    let WrittenContents::Bytes(container) = &mut flow.writes[second_container_copy].contents else {
+        panic!("单元写是普通写");
+    };
+    container[..node_bytes.len()].copy_from_slice(&node_bytes);
+    flow
+}
+
+/// 墓碑树根第一条条目不再改别处（类型段照旧是 1）。
+fn leave_the_first_entry_as_it_is(_bytes: &mut [u8]) {}
+
+/// 第 31 批：容器树类型段 = 记录类型那一处的 F-W（第 30 批 M 交回第 4 条：GPU 两处没有对拍流）：孩子盘 1 那一份自证成码 2，另一份是码 3、
+/// 按 32 KiB 自证得过：CPU 认出码 2、I-9.23 红，再照类型段 1 那一支走盘 0 那一份、判出 I-1.7（见别类就停的话 I-1.7 报成立）；
+/// GPU 走读内核 `class_of_the_child_of_a_container_entry = NONE` 那一处同改，每个状态逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_container_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_container_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node",
+        &record_deletion_with_a_wrong_record_width_in_the_tombstone_container_and_its_second_copy_holding_the_root_bytes(
+            "crash-judge-gpu-i923-tombstone-container-record-width-second-copy-verifying-as-a-node",
+            "deletion-tombstone-container-record-width-second-copy-verifying-as-a-node",
+            &leave_the_first_entry_as_it_is,
+        ),
+        &["I-9.23", "I-1.7"],
+    );
+}
+
+/// 第 31 批：容器树类型段 0 那一处的 F-W：同上一条的镜像，墓碑树根第一条条目再把类型段 1 → 0。孩子盘 0 那一份自证成码 3（另一类），
+/// 盘 1 那一份就是类型段 0 那一类（码 2）、按 16 KiB 自证得过：CPU 判 I-9.23 红，照类型段 0 那一支走那一份——层级 1 的节点下面不往下走、
+/// 两份照判 I-2.1（都对得上）、记走读失败，红 I-7.2（见别类就停的话不记走读失败、I-7.2 不红）；GPU 走读内核
+/// `class_of_the_child_of_an_internal_entry = NONE` 那一处同改，每个状态逐位相同。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_with_a_copy_verifying_as_a_node(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening_each_of(
+        "the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_with_a_copy_verifying_as_a_node",
+        &record_deletion_with_a_wrong_record_width_in_the_tombstone_container_and_its_second_copy_holding_the_root_bytes(
+            "crash-judge-gpu-i923-tombstone-entry-type-zero-container-second-copy-verifying-as-a-node",
+            "deletion-tombstone-entry-type-zero-container-second-copy-verifying-as-a-node",
+            &|bytes| set_the_type_of_the_first_entry(bytes, 0),
+        ),
+        &["I-9.23", "I-7.2"],
+    );
+}
+
+/// GPU 走读内核记判不了「类型段 0 的内部节点」时交回的原因字（`crash_judge_common.wgsl` 的 `UNDECIDABLE_TOO_DEEP`）。
+const GPU_UNDECIDABLE_TOO_DEEP: u32 = 4;
+
+/// 逐状态比 GPU 与 CPU，GPU 池级 checker 只许在 `reasons_left_to_the_cpu` 这几种原因字上判不了：判了的，违例位与 CPU 逐位相同；
+/// 判不了的，原因字在 `reasons_left_to_the_cpu` 里；至少一个状态的原因字在 `reasons_one_state_must_carry` 里；照流水线那样判不了就回落 CPU
+/// （`gpu_verdict_items` 交回 None），回落之后每个状态的红项都与 CPU 相同。本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
+fn assert_the_gpu_judge_leaves_to_the_cpu_only_states_undecidable_for(
+    test_name: &str,
+    flow: &RecordedFlow,
+    reasons_left_to_the_cpu: &[u32],
+    reasons_one_state_must_carry: &[u32],
+) {
+    let Some(cards) = the_card_pool_or_report_not_run(test_name) else {
+        return;
+    };
+    let comparison = compare_on_every_state(cards, flow, expand_up_to());
+    let mut states_left_to_the_cpu = 0u64;
+    let mut states_carrying_a_required_reason = 0u64;
+    let mut disagreements: Vec<String> = Vec::new();
+    for (ordinal, gpu_words) in comparison.gpu_words_of_each_state.iter().enumerate() {
+        let cpu_checker_bits = comparison.cpu_checker_bits_of_each_state[ordinal];
+        let cpu_red_items = &comparison.cpu_red_items_of_each_state[ordinal];
+        let reason = gpu_words[POOL_CHECKER_UNDECIDABLE_WORD];
+        if reason == 0 {
+            let gpu_checker_bits = pool_checker_violated_bits(gpu_words);
+            if gpu_checker_bits != cpu_checker_bits {
+                disagreements.push(format!(
+                    "状态 {ordinal}：GPU 池级 checker 判了，违例 {:?}；CPU 违例 {:?}",
+                    invariant_names(gpu_checker_bits),
+                    invariant_names(cpu_checker_bits)
+                ));
+            }
+        } else if reasons_left_to_the_cpu.contains(&reason) {
+            states_left_to_the_cpu += 1;
+            if reasons_one_state_must_carry.contains(&reason) {
+                states_carrying_a_required_reason += 1;
+            }
+        } else {
+            disagreements.push(format!(
+                "状态 {ordinal}：GPU 池级 checker 判不了的原因字 {reason:#x} 不在 {reasons_left_to_the_cpu:#x?} 里"
+            ));
+        }
+        let concluded = gpu_verdict_items(gpu_words)
+            .map_or_else(|| cpu_red_items.clone(), |items| format!("{items:?}"));
+        if concluded != *cpu_red_items {
+            disagreements.push(format!(
+                "状态 {ordinal}：判不了就回落 CPU 之后的红项 {concluded} 与 CPU 的 {cpu_red_items} 不同"
+            ));
+        }
+    }
+    eprintln!(
+        "CRASH_JUDGE_GPU_LEFT_TO_THE_CPU flow={} states={} left_to_the_cpu={states_left_to_the_cpu} carrying_a_required_reason={states_carrying_a_required_reason}",
+        flow.name, comparison.states
+    );
+    assert!(
+        states_carrying_a_required_reason > 0,
+        "流 {}：至少一个状态上 GPU 池级 checker 要记判不了、原因字在 {reasons_one_state_must_carry:#x?} 里",
+        flow.name
+    );
+    assert!(
+        disagreements.is_empty(),
+        "流 {}：{} 处不一致（前几个）：\n{}",
+        flow.name,
+        disagreements.len(),
+        disagreements
+            .iter()
+            .take(6)
+            .cloned()
+            .collect::<Vec<_>>()
+            .join("\n")
+    );
+}
+
+/// 第 30 批 M 交回第 3 条那一形：新池新建文件的流上 inode 叶容器盘序大的那一份换成 inode 树根（码 2）的原样字节、位置条目跟着换
+/// （[`hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf`]），再把 inode 树根第一条条目类型段 2 → 0、重封到根。
+fn record_new_pool_with_the_inode_root_entry_of_type_zero_over_a_leaf_whose_second_copy_holds_the_inode_root_bytes(
+    tag: &str,
+) -> RecordedFlow {
+    let name = "new-pool-inode-root-entry-type-zero-leaf-second-copy-verifying-as-a-node";
+    let mut flow = record_new_pool_file_creation(tag, name);
+    hold_the_inode_root_bytes_in_the_second_copy_of_the_inode_leaf(
+        &mut flow,
+        name,
+        ChecksumOfTheSecondLocation::MatchesTheNodeBytes,
+    );
+    let root_copies = last_unit_copies(
+        &flow,
+        name,
+        &is_the_inode_tree_root_over_leaf_containers,
+        false,
+    );
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &root_copies, &|bytes| {
+        set_the_type_of_the_first_entry(bytes, 0);
+    });
+    flow
+}
+
+/// 第 30 批 M 交回第 3 条：inode 树根条目类型段 0，孩子盘 0 那一份自证成码 3（另一类）、盘 1 那一份就是码 2、按 16 KiB 自证得过。
+/// CPU 判 I-9.2 红、照类型段 0 那一支走那一份（F-W）；GPU 走读内核 inode 树只接层级 1、类型段 2，F-W 之后落到「类型段 0 的内部节点」那一句，
+/// 池级 checker 记判不了（[`GPU_UNDECIDABLE_TOO_DEEP`]），交回 CPU 判。逐状态：判了的与 CPU 逐位相同，判不了的只许是这一种原因字、至少一个状态是，
+/// 回落 CPU 之后红项相同。
+#[test]
+fn the_gpu_judge_leaves_to_the_cpu_an_inode_root_entry_of_type_zero_over_a_leaf_with_a_copy_verifying_as_a_node(
+) {
+    assert_the_gpu_judge_leaves_to_the_cpu_only_states_undecidable_for(
+        "the_gpu_judge_leaves_to_the_cpu_an_inode_root_entry_of_type_zero_over_a_leaf_with_a_copy_verifying_as_a_node",
+        &record_new_pool_with_the_inode_root_entry_of_type_zero_over_a_leaf_whose_second_copy_holds_the_inode_root_bytes(
+            "crash-judge-gpu-i92-inode-root-entry-type-zero-leaf-second-copy-verifying-as-a-node",
+        ),
+        &[GPU_UNDECIDABLE_TOO_DEEP],
+        &[GPU_UNDECIDABLE_TOO_DEEP],
+    );
+}
+
+/// 最新根那张实例表改造出来的第 1 片写在哪一槽：在 [`FIRST_SLOT_OF_THE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST`] 起那一截（候选根的新链占着）之前，
+/// 同样是流里没写过、基镜像里全零的那一截（写之前由 [`writes_of_instance_table_pages`] 核）。
+const SLOT_OF_THE_SECOND_PAGE_OF_THE_NEWEST_INSTANCE_TABLE: u64 = 199_000;
+/// 最新根那条链第 1 片的链指针记录指到候选根那条链的第几片（攻方 `research/prompts/m3-batch29-code-r1-opus-output.md` 第四节 `share=(1, 100)`）。
+const PAGE_OF_THE_CANDIDATE_CHAIN_THE_NEWEST_CHAIN_RUNS_INTO: usize = 100;
+/// 实例 3（崩溃恢复抛弃 C 之后那次可写挂载取的号）：它写行那次发布的根是候选根，最后一次暖机的根是最新根。
+const INSTANCE_OF_THE_MOUNT_AFTER_THE_RECOVERY: u32 = 3;
+/// 根记录里写序实例（4 字节）的偏移。
+const ROOT_RECORD_INSTANCE_OFFSET: usize = 24;
+
+/// 攻方 L2 那一形（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第二节 K1 L2 两句走不走得到那一行、第三节第 3 条；
+/// 攻方 `research/prompts/m3-batch29-code-r1-opus-output.md` 第四节 `newest=… other=257 role=candidate share=(1, 100)`）：
+/// 被崩溃恢复抛弃 C 的那一段历史（[`record_a_root_abandoned_by_a_crash_recovery`]）里，实例 3 的根都指着实例 3 写行那次写的单片实例表 T。
+/// 实例 3 写行那条根 R（回退候选集里、不是最新根）改指一条 257 片的新链：第 0 片放 T 的行，其余各片只有链指针记录（头照 T）；
+/// R 的根槽写与它那次发布的 journal 记录新根段里的实例表指针改指新链第 0 片、重封，新链各片两份另起一段插在它们之前。
+/// 再把 T 改成两片：第 0 片放原来的行、链指针记录指第 1 片，第 1 片只有一条链指针记录、指 R 那条链的第 100 片（容器号 100，
+/// 在最新根那条链上是第 2 片：身份不对，坏镜像），T 重封到根，第 1 片两份另起一段插在写 T 那一段之前。
+/// GPU 走读内核在上下文 0 里先走最新根的链、走到 R 那条链的第 100 片标成走过，再走候选根 R 的链走到第 100 片合并、停，不记站点 61；
+/// 扫描内核从头读 R 的链、读满 256 片链还没完，记判不了（站点 37 或 38）。CPU 一侧先钉流造对了：整条落盘的镜像上最新根那条链第 2 片身份不对，I-1.1 红。
+fn record_a_candidate_root_whose_own_instance_table_chain_is_longer_than_the_gpu_scans_and_the_newest_chain_runs_into_it(
+    tag: &str,
+) -> RecordedFlow {
+    let FlowWithARootAbandonedByACrashRecovery { mut flow, .. } =
+        record_a_root_abandoned_by_a_crash_recovery(
+            tag,
+            "candidate-root-own-instance-table-of-257-pages-the-newest-chain-runs-into",
+        );
+    let root_slot_writes_of_the_mount_after_the_recovery: Vec<usize> = (0..flow.writes.len())
+        .filter(|write_index| {
+            flow.writes[*write_index].kind == StepKind::RootRecordFua
+                && flow.writes[*write_index].bytes().is_some_and(|root| {
+                    read_u32_at(root, ROOT_RECORD_INSTANCE_OFFSET)
+                        == INSTANCE_OF_THE_MOUNT_AFTER_THE_RECOVERY
+                })
+        })
+        .collect();
+    let txg_of = |recorded_flow: &RecordedFlow, write_index: usize| {
+        read_u64_at(
+            recorded_flow.writes[write_index]
+                .bytes()
+                .expect("根槽写带字节"),
+            ROOT_RECORD_CHECKPOINT_TXG_OFFSET,
+        )
+    };
+    let candidate_root_txg = root_slot_writes_of_the_mount_after_the_recovery
+        .iter()
+        .map(|write_index| txg_of(&flow, *write_index))
+        .min()
+        .expect("实例 3 至少写一次根槽");
+    let newest_root_txg = root_slot_writes_of_the_mount_after_the_recovery
+        .iter()
+        .map(|write_index| txg_of(&flow, *write_index))
+        .max()
+        .expect("实例 3 至少写一次根槽");
+    assert!(
+        candidate_root_txg < newest_root_txg,
+        "实例 3 写行那条根之后还有暖机的根：R 不是最新根"
+    );
+    let pointer_of = |recorded_flow: &RecordedFlow, write_index: usize| {
+        recorded_flow.writes[write_index]
+            .bytes()
+            .expect("根槽写带字节")[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
+            ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
+            .to_vec()
+    };
+    let pointer_to_the_table_of_the_mount =
+        pointer_of(&flow, root_slot_writes_of_the_mount_after_the_recovery[0]);
+    assert!(
+        root_slot_writes_of_the_mount_after_the_recovery
+            .iter()
+            .all(|write_index| pointer_of(&flow, *write_index) == pointer_to_the_table_of_the_mount),
+        "实例 3 的根都指着同一张实例表 T"
+    );
+    let root_slot_writes_of_the_candidate: Vec<usize> =
+        root_slot_writes_of_the_mount_after_the_recovery
+            .iter()
+            .copied()
+            .filter(|write_index| txg_of(&flow, *write_index) == candidate_root_txg)
+            .collect();
+    let journal_records_of_the_candidate: Vec<usize> = (0..flow.writes.len())
+        .filter(|write_index| {
+            flow.writes[*write_index].kind == StepKind::JournalRecord
+                && flow.writes[*write_index].bytes().is_some_and(|bytes| {
+                    read_u32_at(bytes, JOURNAL_RECORD_INSTANCE_OFFSET)
+                        == INSTANCE_OF_THE_MOUNT_AFTER_THE_RECOVERY
+                        && read_u64_at(bytes, JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET)
+                            == candidate_root_txg
+                })
+        })
+        .collect();
+    assert!(
+        !journal_records_of_the_candidate.is_empty(),
+        "R 那次发布写了 journal 记录"
+    );
+    let journal_pointer_field = JOURNAL_NEW_ROOT_SEGMENT_OFFSET
+        + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
+        ..JOURNAL_NEW_ROOT_SEGMENT_OFFSET
+            + JOURNAL_NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER_OFFSET
+            + NODE_POINTER_BYTES;
+    for record in &journal_records_of_the_candidate {
+        assert_eq!(
+            &flow.writes[*record].bytes().expect("记录写带字节")[journal_pointer_field.clone()],
+            pointer_to_the_table_of_the_mount.as_slice(),
+            "R 那次发布的记录新根段里的实例表指针与 R 的根记录相同"
+        );
+    }
+    let table_copies_of_the_mount = |recorded_flow: &RecordedFlow| -> Vec<usize> {
+        let locations = POINTER_LOCATION_ENTRY_OFFSETS.map(|location| {
+            device_and_slot_of_a_pointer_location(&pointer_to_the_table_of_the_mount, location)
+        });
+        (0..recorded_flow.writes.len())
+            .filter(|write_index| {
+                let write = &recorded_flow.writes[*write_index];
+                write.kind == StepKind::UnitWrite
+                    && locations.contains(&(write.device.0, write.offset.0 / SLOT_BYTES))
+                    && write.bytes().is_some_and(|bytes| {
+                        unit_class(bytes) == 3
+                            && packed_record_type(bytes) == PACKED_RECORD_TYPE_INSTANCE_TABLE
+                    })
+            })
+            .collect()
+    };
+    let table_copies_before_the_chain = table_copies_of_the_mount(&flow);
+    assert_eq!(
+        table_copies_before_the_chain.len(),
+        2,
+        "实例 3 写行那次写了 T 两份"
+    );
+    let table_of_the_mount = flow.writes[table_copies_before_the_chain[0]]
+        .bytes()
+        .expect("单元写带字节")
+        .to_vec();
+    let mut rows = records_of_an_instance_table_page(&table_of_the_mount);
+    assert_eq!(
+        rows.pop(),
+        Some(chain_record_of_the_last_page()),
+        "T 只有一片"
+    );
+    let mut rows_of_each_page_of_the_candidate_chain = vec![rows.clone()];
+    rows_of_each_page_of_the_candidate_chain.resize(
+        InstanceTableChainPages::OneMoreThanTheGpuWalks.count(),
+        Vec::new(),
+    );
+    let candidate_pages = instance_table_pages_from(
+        &table_of_the_mount,
+        &pointer_to_the_table_of_the_mount,
+        0,
+        &rows_of_each_page_of_the_candidate_chain,
+    );
+    let (slot_of_the_page_run_into, page_run_into) =
+        candidate_pages[PAGE_OF_THE_CANDIDATE_CHAIN_THE_NEWEST_CHAIN_RUNS_INTO].clone();
+    let pointer_to_the_first_candidate_page = pointer_naming_a_page_at(
+        &pointer_to_the_table_of_the_mount,
+        candidate_pages[0].0,
+        crc32c(&candidate_pages[0].1),
+    );
+    for root_slot_write in &root_slot_writes_of_the_candidate {
+        rewrite_root_record_and_reseal(&mut flow, *root_slot_write, &|bytes| {
+            bytes[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
+                ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
+                .copy_from_slice(&pointer_to_the_first_candidate_page);
+        });
+    }
+    for record in &journal_records_of_the_candidate {
+        rewrite_journal_record_and_reseal(&mut flow, *record, &|bytes| {
+            bytes[journal_pointer_field.clone()]
+                .copy_from_slice(&pointer_to_the_first_candidate_page);
+        });
+    }
+    let devices = devices_of_an_instance_table_pointer(&pointer_to_the_table_of_the_mount);
+    let candidate_page_writes = writes_of_instance_table_pages(
+        &flow,
+        devices,
+        candidate_pages,
+        PagesAfterTheFirstInTheFlow::Intact,
+    );
+    let first_write_naming_the_candidate_chain = *root_slot_writes_of_the_candidate
+        .iter()
+        .chain(&journal_records_of_the_candidate)
+        .min()
+        .expect("至少一次写");
+    insert_a_segment_of_writes_before_the_segment_of(
+        &mut flow,
+        first_write_naming_the_candidate_chain,
+        candidate_page_writes,
+    );
+    let mut second_page_of_the_table = table_of_the_mount.clone();
+    replace_the_records_of_an_instance_table_page(
+        &mut second_page_of_the_table,
+        &[chain_record_naming_the_next_page(
+            &pointer_to_the_table_of_the_mount,
+            slot_of_the_page_run_into,
+            crc32c(&page_run_into),
+        )],
+    );
+    second_page_of_the_table
+        [PACKED_UNIT_CONTAINER_NUMBER_OFFSET..PACKED_UNIT_CONTAINER_NUMBER_OFFSET + 8]
+        .copy_from_slice(&1u64.to_le_bytes());
+    reseal_unit(&mut second_page_of_the_table);
+    let mut first_page_records = rows;
+    first_page_records.push(chain_record_naming_the_next_page(
+        &pointer_to_the_table_of_the_mount,
+        SLOT_OF_THE_SECOND_PAGE_OF_THE_NEWEST_INSTANCE_TABLE,
+        crc32c(&second_page_of_the_table),
+    ));
+    let table_copies = table_copies_of_the_mount(&flow);
+    assert_eq!(table_copies.len(), 2, "插段之后 T 还是两份");
+    corrupt_units_and_reseal_up_to_the_root(&mut flow, &table_copies, &|bytes| {
+        replace_the_records_of_an_instance_table_page(bytes, &first_page_records);
+    });
+    let second_page_writes = writes_of_instance_table_pages(
+        &flow,
+        devices,
+        vec![(
+            SLOT_OF_THE_SECOND_PAGE_OF_THE_NEWEST_INSTANCE_TABLE,
+            second_page_of_the_table,
+        )],
+        PagesAfterTheFirstInTheFlow::Intact,
+    );
+    let first_table_write = *table_copies.iter().min().expect("T 至少一份");
+    insert_a_segment_of_writes_before_the_segment_of(
+        &mut flow,
+        first_table_write,
+        second_page_writes,
+    );
+    let mut image_with_every_write = flow.base.clone();
+    image_with_every_write.apply_writes(&flow.writes);
+    assert!(
+        matches!(
+            pool_checker_verdict_on_the_image(&image_with_every_write, "I-1.1"),
+            InvariantVerdict::Violated(_)
+        ),
+        "流 {}：最新根那条链第 2 片是 R 那条链的第 100 片、容器号不是 2，CPU 判 I-1.1 红：{:?}",
+        flow.name,
+        pool_checker_verdict_on_the_image(&image_with_every_write, "I-1.1")
+    );
+    flow
+}
+
+/// 第 31 批（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第三节第 3 条：扫描内核站点 37 / 38 那两句留作兜底，补走得到它们的那一形）：
+/// [`record_a_candidate_root_whose_own_instance_table_chain_is_longer_than_the_gpu_scans_and_the_newest_chain_runs_into_it`]。
+/// CPU 读链不设片数上限；GPU 池级 checker 判不了的原因字只许是实例表链过长那三种（R 还是最新根的状态上走读内核走满记 61），
+/// 且至少一个状态落在扫描内核那两处（37 或 38，推的：读码，攻方读码模型 `l2_chain_model.py` 也是这样，没在卡上跑过）；
+/// 判了的与 CPU 逐位相同，判不了就回落 CPU 之后红项相同。
+#[test]
+fn the_gpu_judge_leaves_to_the_cpu_a_candidate_instance_table_chain_longer_than_the_pages_it_scans_once_the_newest_chain_runs_into_it(
+) {
+    assert_the_gpu_judge_leaves_to_the_cpu_only_states_undecidable_for(
+        "the_gpu_judge_leaves_to_the_cpu_a_candidate_instance_table_chain_longer_than_the_pages_it_scans_once_the_newest_chain_runs_into_it",
+        &record_a_candidate_root_whose_own_instance_table_chain_is_longer_than_the_gpu_scans_and_the_newest_chain_runs_into_it(
+            "crash-judge-gpu-candidate-instance-table-of-257-pages-the-newest-chain-runs-into",
+        ),
+        &[
+            GPU_UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED,
+            GPU_UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED,
+            GPU_UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED,
+        ],
+        &[
+            GPU_UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED,
+            GPU_UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED,
+        ],
+    );
+}
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -13065,3 +13065,203 @@
         "I-7.4 说明写明是这条根自己的树表指针全零：{detail}"
     );
 }
+
+// ─── 第 31 批（删除线）：墓碑树、谱系树层级 1 节点里类型段 0 的孩子照判 I-2.1（F-L0）、走读不往下读的孩子按哪种长度比的三形（测试尖度） ───
+
+/// F-L0（判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第二节 K2 L1 第三支那一行、第三节第 1 条；攻方
+/// `research/prompts/m3-batch29-code-r1-opus-output.md` 第一节 `tomb-L1T-sum01bad-type0z` / `lin-L1T-sum01bad-type0z` 那一形）：
+/// 树根（层级 1）第一条条目类型段 `registered_type` → 0、身份三段清零，子指针两条位置条目的整单元校验和各翻一字节，重封到根；孩子不动。
+/// 两份按头里那一类（码 3）的长度都对不上，孩子不是另一类、走类型段 0 那一支，身份引用那一半成立、孩子读不出「子单元是码 2」不判，
+/// 层级 1 的节点下面不往下走、记走读失败；孩子每一份副本照判 I-2.1：红 I-2.1、I-4.8、I-7.2、I-7.4，身份那一条不红
+/// （第 30 批这一支只记走读失败、I-2.1 报成立）。
+fn assert_a_type_zero_entry_of_a_level_one_container_tree_root_over_two_wrong_location_checksums_reddens_the_checksum_invariant(
+    mut image: MemoryPool,
+    kind: u16,
+    registered_type: u16,
+    scene: &str,
+) {
+    let level = change_an_entry_of_the_root_of_kind(&mut image, kind, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            registered_type,
+            CONTAINER_TREE_ENTRY_TYPE_INTERNAL_NODE,
+        );
+        zero_the_three_identity_fields(entry);
+        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 0);
+        flip_a_byte_of_the_whole_unit_checksum_in_a_child_location(entry, 1);
+    });
+    assert_eq!(level, 1, "{scene}：树根直接管容器");
+    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
+        &image,
+        &["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
+        scene,
+    );
+}
+
+#[test]
+fn a_tombstone_tree_entry_of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
+) {
+    assert_a_type_zero_entry_of_a_level_one_container_tree_root_over_two_wrong_location_checksums_reddens_the_checksum_invariant(
+        common_deletion_in_progress::image_after_the_intent_of_a_three_unit_file(
+            "known-bad-tombstone-entry-type-zero-no-identity-both-location-checksums-wrong",
+        ),
+        TREE_KIND_TOMBSTONE,
+        PACKED_RECORD_TYPE_TOMBSTONE,
+        "墓碑树根条目类型段 0、身份三段清零、子指针两条位置条目的校验和都坏",
+    );
+}
+
+#[test]
+fn a_lineage_tree_entry_of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants(
+) {
+    assert_a_type_zero_entry_of_a_level_one_container_tree_root_over_two_wrong_location_checksums_reddens_the_checksum_invariant(
+        image_after_rolling_back_to_the_first_root(
+            "known-bad-lineage-entry-type-zero-no-identity-both-location-checksums-wrong",
+        )
+        .image,
+        TREE_KIND_LINEAGE,
+        PACKED_RECORD_TYPE_LINEAGE,
+        "谱系树根条目类型段 0、身份三段清零、子指针两条位置条目的校验和都坏",
+    );
+}
+
+/// 内部条目（120 字节）子指针里第 `location_index` 条位置条目的整单元校验和写成 `checksum`（位置条目 = 盘 4 + 槽 6 + 整单元校验和 4）。
+fn set_the_whole_unit_checksum_in_a_child_location(
+    entry: &mut [u8],
+    location_index: usize,
+    checksum: u32,
+) {
+    let checksum_offset = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
+        + POINTER_FIRST_LOCATION_OFFSET
+        + location_index * POINTER_LOCATION_BYTES
+        + LOCATION_CHECKSUM_OFFSET;
+    entry[checksum_offset..checksum_offset + 4].copy_from_slice(&checksum.to_le_bytes());
+}
+
+/// 不在池里的盘身份：两块盘是 0、1，写成 7 的位置条目照 I-2.5 仍按设备身份升序，那一份读不出。
+const DEVICE_IDENTITY_NOT_IN_THE_POOL: u32 = 7;
+
+/// 内部条目子指针里第 `location_index` 条位置条目的盘写成 `device`（槽与整单元校验和不动）。
+fn set_the_device_in_a_child_location(entry: &mut [u8], location_index: usize, device: u32) {
+    let device_offset = CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET
+        + POINTER_FIRST_LOCATION_OFFSET
+        + location_index * POINTER_LOCATION_BYTES;
+    entry[device_offset..device_offset + 4].copy_from_slice(&device.to_le_bytes());
+}
+
+/// 新池新建文件那一版 inode 树根（层级 1）第一条条目的孩子（码 3 叶容器，32 KiB）前 16 KiB 的 CRC-32C。
+fn checksum_of_the_first_sixteen_kibibytes_of_the_first_inode_leaf(image: &MemoryPool) -> u32 {
+    let child = child_pointer_of_an_entry_of_the_root_of_kind(image, TREE_KIND_INODE, 0);
+    crc32_castagnoli(&common_pointer_head_corruption::unit_bytes_at(
+        image,
+        common_pointer_head_corruption::the_slot_of(&child),
+        common_pointer_head_corruption::node_bytes(),
+    ))
+}
+
+/// 测试尖度 X3（正推 `research/prompts/m3-batch29-code-r1-sonnet-output.md` T-a 那张变异表；判决同一份第二节 K1 L1 测试尖度那一行、第三节第 4 条）：
+/// 新池新建文件那一版 inode 树根（层级 1）第一条条目类型段 2 → 7、子指针第一条位置条目的整单元校验和换成叶容器前 16 KiB 的 CRC，重封到根；
+/// 叶容器不动。孩子不往下走，第一份头里是登记过的码 3，只按 32 KiB 比、对不上：红 I-9.2、I-2.1、I-4.8、I-7.4；第二份对得上，不记走读失败、
+/// I-7.2 不红。登记过的类也按 16 KiB 比时第一份对得上，I-2.1 不红。
+#[test]
+fn an_inode_tree_entry_with_an_unregistered_type_over_a_leaf_whose_first_location_checksum_matches_only_its_first_sixteen_kibibytes_reddens_the_checksum_invariant(
+) {
+    let mut image = build_pool(
+        "known-bad-inode-entry-unregistered-type-first-location-checksum-of-sixteen-kibibytes",
+    )
+    .memory_pool();
+    let checksum_of_the_first_sixteen_kibibytes =
+        checksum_of_the_first_sixteen_kibibytes_of_the_first_inode_leaf(&image);
+    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_INODE,
+            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
+        );
+        set_the_whole_unit_checksum_in_a_child_location(
+            entry,
+            0,
+            checksum_of_the_first_sixteen_kibibytes,
+        );
+    });
+    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
+        &image,
+        &["I-9.2", "I-2.1", "I-4.8", "I-7.4"],
+        "inode 树根条目类型段 7、第一条位置条目的校验和是叶容器前 16 KiB 的 CRC",
+    );
+}
+
+/// 测试尖度 X4（同上那张变异表）：新池新建文件那一版 inode 叶容器的类标签改成没登记的 9（不重封头，新的整单元校验和沿引用链补到根），
+/// 再把 inode 树根第一条条目类型段 2 → 7、子指针第一条位置条目的整单元校验和换成改过类标签的叶容器前 16 KiB 的 CRC，重封到根。
+/// 孩子不往下走；两份头里的码都没登记，16 KiB、32 KiB 两种长度都试：第一份按 16 KiB 对得上、第二份按 32 KiB 对得上，I-2.1 不红
+/// （红的是 I-9.2 与映射树那一遍的 I-1.11）。没登记的只按 32 KiB 比时第一份对不上、只按 16 KiB 比时第二份对不上，I-2.1 都红。
+#[test]
+fn an_inode_tree_entry_with_an_unregistered_type_over_a_leaf_of_an_unregistered_class_whose_first_location_checksum_matches_its_first_sixteen_kibibytes_leaves_the_checksum_invariant_holding(
+) {
+    let mut image = build_pool(
+        "known-bad-inode-entry-unregistered-type-unregistered-class-first-location-checksum-of-sixteen-kibibytes",
+    )
+    .memory_pool();
+    let child = child_pointer_of_an_entry_of_the_root_of_kind(&image, TREE_KIND_INODE, 0);
+    let leaf_slot = common_pointer_head_corruption::the_slot_of(&child);
+    let mut leaf = common_pointer_head_corruption::unit_bytes_at(
+        &image,
+        leaf_slot,
+        common_pointer_head_corruption::data_unit_bytes(),
+    );
+    assert_eq!(leaf[UNIT_HEADER_CLASS_TAG_OFFSET], 3, "孩子是码 3 叶容器");
+    leaf[UNIT_HEADER_CLASS_TAG_OFFSET] = UNREGISTERED_UNIT_CLASS_TAG;
+    common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum(
+        &mut image, leaf_slot, &leaf,
+    );
+    let checksum_of_the_first_sixteen_kibibytes =
+        checksum_of_the_first_sixteen_kibibytes_of_the_first_inode_leaf(&image);
+    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_INODE,
+            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
+        );
+        set_the_whole_unit_checksum_in_a_child_location(
+            entry,
+            0,
+            checksum_of_the_first_sixteen_kibibytes,
+        );
+    });
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariant_set(&verdicts),
+        ["I-1.11", "I-9.2"]
+            .into_iter()
+            .collect::<std::collections::BTreeSet<&'static str>>(),
+        "inode 叶容器类标签 9、根条目类型段 7、第一条位置条目的校验和是前 16 KiB 的 CRC：{verdicts:?}"
+    );
+}
+
+/// 头里的码不在登记表里（登记的是 1、2、3）。
+const UNREGISTERED_UNIT_CLASS_TAG: u8 = 9;
+
+/// 测试尖度 X5（同上那张变异表）：新池新建文件那一版 inode 树根第一条条目类型段 2 → 7、子指针第二条位置条目的盘换成不在池里的 7，
+/// 重封到根。孩子不往下走；第二份读不出（头与整单元都读不出），算对不上：红 I-9.2、I-2.1、I-4.8、I-7.4；第一份对得上，不记走读失败。
+/// 读不出算对得上时 I-2.1 不红。
+#[test]
+fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_second_copy_does_not_read_back_reddens_the_checksum_invariant(
+) {
+    let mut image = build_pool(
+        "known-bad-inode-entry-unregistered-type-second-location-on-a-device-not-in-the-pool",
+    )
+    .memory_pool();
+    change_an_entry_of_the_root_of_kind(&mut image, TREE_KIND_INODE, 0, |entry| {
+        change_the_entry_type(
+            entry,
+            PACKED_RECORD_TYPE_INODE,
+            CONTAINER_TREE_ENTRY_TYPE_NOT_REGISTERED,
+        );
+        set_the_device_in_a_child_location(entry, 1, DEVICE_IDENTITY_NOT_IN_THE_POOL);
+    });
+    assert_the_child_the_walk_does_not_go_into_reddens_exactly(
+        &image,
+        &["I-9.2", "I-2.1", "I-4.8", "I-7.4"],
+        "inode 树根条目类型段 7、第二条位置条目指着不在池里的盘",
+    );
+}
````

## 实现员 n：mutations-append.tsv（/home/fy5090/.local/share/singlefs/work/m3-batch31/n/patch/mutations-append.tsv，追加到 `crates/mutations.tsv` 末尾，合后第 3054–3065 行）

````tsv
第31批 F-L0：墓碑树、谱系树层级 1 节点里类型段 0 的孩子不照判 I-2.1（类型段 0、身份清零、两条位置条目校验和都坏，I-2.1 报成立）	crates/singlefs-checker/src/walk.rs	        if node.level <= 1 {\n            self.judge_every_copy_of_a_child_the_walk_does_not_go_into(\n                child,\n                &format!("{name}内部条目的子指针"),\n            );\n	        if node.level <= 1 {\n	-p singlefs-harness --test checker_known_bad_images -- of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong first_location_checksum_matches_only_its_first_sixteen_kibibytes of_an_unregistered_class_whose_first_location_checksum_matches_its_first_sixteen_kibibytes over_a_child_whose_second_copy_does_not_read_back	a_tombstone_tree_entry_of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong_reddens_the_checksum_and_walk_invariants
第31批 X3：走读不往下读的孩子头里是登记过的类也按 16 KiB、32 KiB 两种长度比（inode 条目类型段 7、第一条位置条目校验和是叶容器前 16 KiB 的 CRC，I-2.1 报成立）	crates/singlefs-checker/src/walk.rs	        Some(unit_bytes) => vec![unit_bytes],\n	        Some(_unit_bytes) => vec![node_bytes(), data_unit_bytes()],\n	-p singlefs-harness --test checker_known_bad_images -- of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong first_location_checksum_matches_only_its_first_sixteen_kibibytes of_an_unregistered_class_whose_first_location_checksum_matches_its_first_sixteen_kibibytes over_a_child_whose_second_copy_does_not_read_back	an_inode_tree_entry_with_an_unregistered_type_over_a_leaf_whose_first_location_checksum_matches_only_its_first_sixteen_kibibytes_reddens_the_checksum_invariant
第31批 X4：走读不往下读的孩子头里的类没登记时只按 32 KiB 比（inode 叶容器类标签 9、第一条位置条目校验和是前 16 KiB 的 CRC，I-2.1 红）	crates/singlefs-checker/src/walk.rs	        None => vec![node_bytes(), data_unit_bytes()],\n	        None => vec![data_unit_bytes()],\n	-p singlefs-harness --test checker_known_bad_images -- of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong first_location_checksum_matches_only_its_first_sixteen_kibibytes of_an_unregistered_class_whose_first_location_checksum_matches_its_first_sixteen_kibibytes over_a_child_whose_second_copy_does_not_read_back	an_inode_tree_entry_with_an_unregistered_type_over_a_leaf_of_an_unregistered_class_whose_first_location_checksum_matches_its_first_sixteen_kibibytes_leaves_the_checksum_invariant_holding
第31批 X5：走读不往下读的孩子那一份读不出算对得上（inode 条目类型段 7、第二条位置条目指着不在池里的盘，I-2.1 报成立）	crates/singlefs-checker/src/walk.rs	    lengths_to_compare.into_iter().any(|unit_bytes| {\n        reader\n            .read(location.device, offset, unit_bytes)\n            .is_some_and(|content| crc32_castagnoli_table(&content) == location.checksum)\n    })	    lengths_to_compare.into_iter().any(|unit_bytes| {\n        reader\n            .read(location.device, offset, unit_bytes)\n            .is_none_or(|content| crc32_castagnoli_table(&content) == location.checksum)\n    })	-p singlefs-harness --test checker_known_bad_images -- of_type_zero_without_an_identity_over_a_child_whose_two_location_checksums_are_wrong first_location_checksum_matches_only_its_first_sixteen_kibibytes of_an_unregistered_class_whose_first_location_checksum_matches_its_first_sixteen_kibibytes over_a_child_whose_second_copy_does_not_read_back	an_inode_tree_entry_with_an_unregistered_type_over_a_child_whose_second_copy_does_not_read_back_reddens_the_checksum_invariant
第31批 GPU 判器 F-L0：走读内核容器树层级 1 节点里类型段 0 的孩子不照判 I-2.1（墓碑树根条目类型段 0、身份清零、两条位置条目校验和都坏那条流上 CPU 红 I-2.1，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                // 判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第三节第 1 条），引用照旧不登记\n                w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);\n	                // 判决 `research/prompts/m3-batch29-code-r1-main-verification.md` 第三节第 1 条），引用照旧不登记\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_child_whose_two_location_checksums_are_wrong	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_without_an_identity_names_a_child_whose_two_location_checksums_are_wrong
第31批 GPU 判器：走读内核容器树类型段不认得那一处不照判 I-2.1（墓碑树根条目类型段 7、两条位置条目校验和都坏那条流上 CPU 红 I-2.1，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            w_note_pointer_references(child, span_of_the_unregistered_child);\n            w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);\n	            w_note_pointer_references(child, span_of_the_unregistered_child);\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong
第31批 GPU 判器：走读内核容器树类型段 0 孩子是另一类那一处不照判 I-2.1（墓碑树根条目类型段 0 指容器、第一条位置条目校验和坏那条流上 CPU 红 I-2.1，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_child_of_an_internal_entry));\n                w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);\n	                w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_child_of_an_internal_entry));\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_whose_first_location_checksum_is_wrong	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_whose_first_location_checksum_is_wrong
第31批 GPU 判器：走读内核容器树类型段 = 记录类型、孩子是另一类那一处不照判 I-2.1（层级 2 墓碑树根条目类型段 1 指节点、第一条位置条目校验和坏那条流上 CPU 红 I-2.1，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_child_of_a_container_entry));\n            w_judge_every_copy_of_a_child_the_walk_does_not_go_into(child);\n	            w_note_pointer_references(child, w_slots_of_a_unit_of_class(class_of_the_child_of_a_container_entry));\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_whose_first_location_checksum_is_wrong	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_whose_first_location_checksum_is_wrong
第31批 GPU 判器 F-W：走读内核容器树类型段 = 记录类型那一处另一份按类型段那一类自证过也不往下走（墓碑容器记录宽 39、另一份自证成码 2 那条流上 CPU 红 I-1.7，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	            w_judge(identity_invariant, class_of_the_child_of_a_container_entry == NONE);\n            class_of_the_child_of_a_container_entry = NONE;\n	            w_judge(identity_invariant, class_of_the_child_of_a_container_entry == NONE);\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_container_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_container_copy_of_the_entry_type_carries_a_violation_beside_a_copy_verifying_as_a_node
第31批 GPU 判器 F-W：走读内核容器树类型段 0 那一处另一份按类型段那一类自证过也不往下走（墓碑树根条目类型段 0、容器另一份自证成码 2 那条流上 CPU 记走读失败红 I-7.2，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                w_judge(identity_invariant, class_of_the_child_of_an_internal_entry == NONE);\n                class_of_the_child_of_an_internal_entry = NONE;\n	                w_judge(identity_invariant, class_of_the_child_of_an_internal_entry == NONE);\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_with_a_copy_verifying_as_a_node	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_type_zero_names_a_container_with_a_copy_verifying_as_a_node
第31批 GPU 判器：走读内核读 extent 上段根没有「层级 0、零条目只判 key 宽」那条豁免（删到最后一批那条流上第 156–164 个状态 GPU 多红 I-1.1；两个窄根变体同红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    } else if (key_range_reading == 3u && n.level == 0u && n.entry_count == 0u) {\n	    } else if (false && key_range_reading == 3u && n.level == 0u && n.entry_count == 0u) {\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch	the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch
第31批 GPU 判器 L2 兜底：扫描内核站点 37 / 38 那两句不记判不了（两个原因字写成 0；候选根自己那张实例表链 257 片、最新根的链途经它第 100 片那条流上一个状态都不落 37 / 38）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	const UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (37u << 8u);\n\n// r_instance_rows（一条根自己那张实例表的全部行，I-7.9 与 I-3.12 读它）走满 INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED 片、链还没完。「条目太多」一类，站点号与别处不撞\nconst UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (38u << 8u);\n	const UNDECIDABLE_INSTANCE_TABLE_PAGES_AFTER_THE_FIRST_LONGER_THAN_THE_PAGES_SCANNED: u32 = 0u;\n\n// r_instance_rows（一条根自己那张实例表的全部行，I-7.9 与 I-3.12 读它）走满 INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED 片、链还没完。「条目太多」一类，站点号与别处不撞\nconst UNDECIDABLE_INSTANCE_TABLE_ROWS_LONGER_THAN_THE_PAGES_SCANNED: u32 = 0u;\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_leaves_to_the_cpu_a_candidate_instance_table_chain_longer_than_the_pages_it_scans_once_the_newest_chain_runs_into_it	the_gpu_judge_leaves_to_the_cpu_a_candidate_instance_table_chain_longer_than_the_pages_it_scans_once_the_newest_chain_runs_into_it
````
