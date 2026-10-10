# 第 35 批代码轮第一轮：改动全文（2026-10-06）

三份补丁照实现员交回的原样，同一趟合入（在删除线第 27 批 J 合入之后）。35-1 的 crates.patch 是主 agent 重打过两处锚点的那一份：对拍用例文件头那一段的上下文换成 J 加的三行、文件末尾追加用例那一段的上下文换成 J 合后的末三行，增删行与实现员交的原补丁逐行相同（原补丁在会话草稿 `b35-patch-backup/35-1/`）。

## m3-formal-35-1

### crates.patch

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
index f3be249..9c0fa85 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -3111,7 +3111,15 @@ fn w_walk_container_tree_root(root: Node, tree: vec2<u32>, root_device: u32, roo
 // 树表别的版本先走过（零单元发布、卸载与抬 F 的空发布照抄上一版的树表与映射根）：条目不重走，这一版的用户可见树的根照先走那一版记下的取，
 // 映射根照样往下走（共享的就并进这一版，I-3.5 比相邻两版的引用集合要它）；树表读不出、头用不了时映射根也不走
 fn w_walk_tree_table_and_mapping(tree_table_pointer: Ptr, mapping_root_pointer: Ptr, fsid_low: vec2<u32>) {
-    let tree_table_walked_by_an_earlier_version = !tree_table_pointer.all_zero && w_visited_contains(tree_table_pointer.device0, tree_table_pointer.slot0);
+    // 树表指针整条全零（walk.rs 的 follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core）：core 的根记录读者照两条位置条目
+    // 读盘 0 槽 0、读不出。照 CPU 跟过去：两条位置条目照 I-2.1 判、这一版记一次走读失败，映射根不走；不判 I-2.5（整条全零的指针豁免）、
+    // 不记引用。根环里的根、由记录施加出来的那几版、重放施加出来的那一版都经这里（用户 2026-10-06 定 Q1、Q3「甲」）
+    if (tree_table_pointer.all_zero) {
+        _ = w_read_referenced_unit(tree_table_pointer, NODE_BYTES);
+        w_failures = w_failures + 1u;
+        return;
+    }
+    let tree_table_walked_by_an_earlier_version = w_visited_contains(tree_table_pointer.device0, tree_table_pointer.slot0);
     let read = w_read_index_node(tree_table_pointer, vec2<u32>(0u, 0u), 2u, 0u, false, fsid_low);
     if (read.ok) {
         let tt = read.n;
@@ -3784,7 +3792,8 @@ fn walk_and_hand_off(g: CheckerSlot, fsid_low: vec2<u32>) -> u32 {
                         if (root_slot_is_readable || lt64(r.txg, effective_floor) || !lt64(oldest_valid_txg, r.txg)) {
                             continue;
                         }
-                        // 按 (实例, txg) 去重、升序插入；同键取计数器小的那一条（先到的）
+                        // 按 (实例, txg) 去重、升序插入；同键取带末条标志的那一条（改法 FL：core 施加末条那一份新根段），
+                        // 一条带标志的都还没遇到时先占着计数器小的那一条（先到的），之后遇到带标志的换上它（walk.rs 的 SegmentTakenForTheVersion）
                         var position = 0u;
                         var duplicate = false;
                         while (position < version_count) {
@@ -3799,6 +3808,10 @@ fn walk_and_hand_off(g: CheckerSlot, fsid_low: vec2<u32>) -> u32 {
                             position = position + 1u;
                         }
                         if (duplicate) {
+                            let existing = checker_record_at(sw(R_DATA + position));
+                            if ((existing.flags & 1u) == 0u && (r.flags & 1u) != 0u) {
+                                sws(R_DATA + position, i);
+                            }
                             continue;
                         }
                         if (version_count >= MAX_DATA_POINTERS * 2u) {
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index ada31db..8a20a8e 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -1066,8 +1066,9 @@ fn judge_allocation_generations_against_unit_births() {
     }
 }
 
-// 下一次挂载会先施加的那一版（walk.rs version_the_next_mount_applies_first）：与最新根同实例、txg = 最新根 txg + 1、带提交标记、
-// 进得了前缀、根槽读不出的第一条记录的下标；没有交回 NONE
+// 下一次挂载会先施加的那一版（walk.rs version_the_next_mount_applies_first）：与最新根同实例、txg = 最新根 txg + 1、带末条标志（改法 FL：
+// core 走到末条才整次施加、施加末条那一份新根段）、带提交标记、进得了前缀、根槽读不出的记录里计数器最小的那一条的下标；没有交回 NONE。
+// 记录数组按计数器升序排（checker_insert_record），同一实例里数组次序就是 CPU 按 (实例, 计数器) 的键序，取第一条即取计数器最小的那一条
 fn record_index_of_the_version_the_next_mount_applies_first() -> u32 {
     let newest_root = checker_root_of(c_newest_index);
     if (add64_overflows(newest_root.txg, u64_of(1u))) {
@@ -1082,7 +1083,7 @@ fn record_index_of_the_version_the_next_mount_applies_first() -> u32 {
     }
     for (var i = 0u; i < record_count; i = i + 1u) {
         let r = checker_record_at(i);
-        if (r.commit_byte == 1u && r.instance == newest_root.instance && eq64(r.txg, next_txg) && record_can_enter_a_replay_prefix(i)) {
+        if ((r.flags & 1u) != 0u && r.commit_byte == 1u && r.instance == newest_root.instance && eq64(r.txg, next_txg) && record_can_enter_a_replay_prefix(i)) {
             return i;
         }
     }
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 3ab51d2..a28644f 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -28,6 +28,8 @@
 //! 第 27 批：inode 叶容器类标签改成不登记的码、不重封（孩子读不出），树根条目类型段 0 带身份、类型段 7 各一条流整份比；
 //! 同一段被崩溃恢复抛弃的历史上，最新根的实例表链指针记录不合法、第 1 片读不出（GPU 关门那一侧）各一条流整份比；
 //! 链 256 片一条整份比，链 257 片一条钉 GPU 池级 checker 记判不了（实例表链过长）、回落 CPU 之后结论相同（攻方 H-c）。
+//! 第 35 批：由记录造出来的两类版本取带末条标志那条记录的新根段（三条记录的发布第一条写成上一版的根：下一次挂载那一版叠两条记录罩住同一个槽、
+//! 挂载之后由记录施加出来的那一版各一条），树表指针整条全零照 core 跟过去（根与它的记录、由记录施加出来的那一版的记录各一条），都整份比。
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
 //! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）；这几处除实例表指针外
 //! 另各有一条 MAC 第 0 字节非 0 的坏流（core 连加密、压缩那几段一并判，GPU 恢复内核同判）；extent 上段叶内联的数据指针另有一条算法类型写 1 的坏流
@@ -4325,7 +4327,7 @@ fn write_the_pointer_head_field_non_zero_at(
 
 /// 根记录实例表指针那一处：发布的根记录里实例表指针头部 extent 偏移写 1、重封根记录。池级 checker 走实例表链第 0 片时判它
 /// （`walk_instance_table_chain`），CPU 与 GPU 走读内核都红 I-2.4。core 择根时把这条根当损坏拒（`root_record.rs` 的 `parse_slot` 第五关），
-/// 退回上一条根（看 journal 那一遍再重放回这一版）；GPU 恢复内核择根同拒（`root_record_pointers_carry_no_extent_offset`）。
+/// 退回上一条根（看 journal 那一遍再重放回这一版）；GPU 恢复内核择根同拒（`root_record_pointer_heads_are_all_the_first_version`）。
 fn record_new_pool_with_an_extent_offset_on_the_instance_table_pointer_of_the_root_record(
     tag: &str,
 ) -> RecordedFlow {
@@ -7597,3 +7599,355 @@
             .join("\n")
     );
 }
+
+// ─── 第 35 批：由记录造出来的两类版本取带末条标志那条记录的新根段（FL）、树表指针全零照 core 跟过去（Q1、Q3） ───
+
+/// 新根段七段在记录与根记录里的偏移（新根段从记录偏移 99 起；D23（journal 的角色与格式） 已定项 15 / 已定项 4 的字段表，与 walk.rs 的
+/// `NEW_ROOT_SEGMENT_FIELDS_COMPARED_WITH_THE_ROOT` 同一张表）：(新根段里的起点, 根记录里的起点, 字节数)。
+const NEW_ROOT_SEGMENT_FIELDS_IN_THE_ROOT_RECORD: [(usize, usize, usize); 7] = [
+    (0, 36, 86),
+    (86, 256, 86),
+    (172, 122, 8),
+    (180, 130, 8),
+    (188, 170, 86),
+    (274, 342, 86),
+    (360, 20, 4),
+];
+
+/// journal 记录头里实例代号（4 字节）、计数器（6 字节）、`checkpoint_txg`（8 字节）的偏移（D23（journal 的角色与格式） 已定项 17 的字段表）。
+const JOURNAL_RECORD_INSTANCE_OFFSET: usize = 16;
+const JOURNAL_RECORD_COUNTER_OFFSET: usize = 20;
+const JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET: usize = 26;
+/// 码 2 节点头里诞生代号那 8 字节的偏移扣掉 2k（D18（块里携带什么信息） 已定项 7：诞生代号 52 + 2k）。
+const INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS: usize = 52;
+
+/// 流里 (实例 1, `checkpoint_txg`) 那次发布的每一份 journal 记录写：(写下标, 计数器)。
+fn journal_record_writes_of_the_publish(
+    flow: &RecordedFlow,
+    checkpoint_txg: u64,
+) -> Vec<(usize, u64)> {
+    flow.writes
+        .iter()
+        .enumerate()
+        .filter(|(_, write)| write.kind == StepKind::JournalRecord)
+        .filter_map(|(index, write)| {
+            let bytes = write.bytes()?;
+            let instance = u32::from_le_bytes(
+                bytes[JOURNAL_RECORD_INSTANCE_OFFSET..JOURNAL_RECORD_INSTANCE_OFFSET + 4]
+                    .try_into()
+                    .expect("4"),
+            );
+            let txg = u64::from_le_bytes(
+                bytes[JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET
+                    ..JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET + 8]
+                    .try_into()
+                    .expect("8"),
+            );
+            let mut counter = [0u8; 8];
+            counter[..6].copy_from_slice(
+                &bytes[JOURNAL_RECORD_COUNTER_OFFSET..JOURNAL_RECORD_COUNTER_OFFSET + 6],
+            );
+            (instance == 1 && txg == checkpoint_txg).then_some((index, u64::from_le_bytes(counter)))
+        })
+        .collect()
+}
+
+/// 三个数据单元那条流里第一个文件顺序写那次发布（实例 1、txg 4，切成三条记录）的**第一条**记录（两块盘各一份）新根段七段换成
+/// txg 3 那条根的（写侧违反 D23（journal 的角色与格式） 已定项 15「每条记录都带同一份新根段」的样子），重封、同实例后面的记录重链。
+fn write_the_root_of_the_first_file_into_the_first_record_of_the_three_unit_publish(
+    flow: &mut RecordedFlow,
+) {
+    let root_of_the_first_file: Vec<u8> = flow
+        .writes
+        .iter()
+        .filter(|write| write.kind == StepKind::RootRecordFua)
+        .filter_map(RetainedWrite::bytes)
+        .find(|bytes| {
+            u32::from_le_bytes(bytes[24..28].try_into().expect("4")) == 1
+                && u64::from_le_bytes(bytes[28..36].try_into().expect("8")) == 3
+        })
+        .expect("流里有第一个文件那条根 (1, 3) 的根槽写")
+        .to_vec();
+    let records = journal_record_writes_of_the_publish(flow, 4);
+    let first_counter = records
+        .iter()
+        .map(|(_, counter)| *counter)
+        .min()
+        .expect("顺序写那次发布有记录");
+    assert_eq!(
+        records.len(),
+        6,
+        "流 {}：顺序写那次发布切成三条记录、两块盘各一份",
+        flow.name
+    );
+    for (index, _) in records
+        .iter()
+        .filter(|(_, counter)| *counter == first_counter)
+    {
+        rewrite_journal_record_and_reseal(flow, *index, &|bytes| {
+            for (in_the_segment, in_the_root, length) in NEW_ROOT_SEGMENT_FIELDS_IN_THE_ROOT_RECORD
+            {
+                let segment = JOURNAL_NEW_ROOT_SEGMENT_OFFSET + in_the_segment;
+                bytes[segment..segment + length]
+                    .copy_from_slice(&root_of_the_first_file[in_the_root..in_the_root + length]);
+            }
+        });
+    }
+}
+
+/// 一片分配记录叶由顺序写那次发布（txg 4）写出、且里面有一对改得动的记录（[`widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts`]）。
+fn is_an_allocation_leaf_of_the_three_unit_publish_that_can_be_widened(bytes: &[u8]) -> bool {
+    if unit_class(bytes) != 2 || index_node_key_width(bytes) != 10 || bytes[50] != 0 {
+        return false;
+    }
+    let birth_txg_offset = INDEX_NODE_BIRTH_TXG_OFFSET_BEFORE_THE_KEYS + 2 * 10;
+    u64::from_le_bytes(
+        bytes[birth_txg_offset..birth_txg_offset + 8]
+            .try_into()
+            .expect("8"),
+    ) == 4
+        && widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
+            &mut bytes.to_vec(),
+        ) == 1
+}
+
+/// 改法 FL 下一次挂载那一支（判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P1-b，攻方报告的 s3 形）：
+/// 三个数据单元那条流上，顺序写那次发布（txg 4，三条记录）写的分配记录叶里两条记录罩住同一个槽（重封到根与三条记录），
+/// 再把第一条记录的新根段写成 txg 3 那条根的。三条记录都落了、根槽没落的状态上，CPU 认下一次挂载先施加的那一版取末条那一条，红 I-5.4；
+/// GPU 扫描内核 `record_index_of_the_version_the_next_mount_applies_first` 同取末条、红在同样的状态上（取第一条就走 txg 3 的树、不红）。
+fn record_three_unit_file_with_a_stale_first_record_over_two_allocation_records_covering_one_slot(
+    tag: &str,
+) -> RecordedFlow {
+    let name = "three-unit-file-stale-first-record-over-overlapping-allocation-records";
+    let mut flow = record_three_unit_file_with_a_resealed_field(
+        tag,
+        name,
+        &is_an_allocation_leaf_of_the_three_unit_publish_that_can_be_widened,
+        &|bytes| {
+            assert_eq!(
+                widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
+                1,
+                "挑叶时这一片改得动"
+            );
+        },
+    );
+    write_the_root_of_the_first_file_into_the_first_record_of_the_three_unit_publish(&mut flow);
+    flow
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_first_record_of_the_next_mount_version_carries_the_previous_root_over_two_allocation_records_covering_one_slot(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_first_record_of_the_next_mount_version_carries_the_previous_root_over_two_allocation_records_covering_one_slot",
+        &record_three_unit_file_with_a_stale_first_record_over_two_allocation_records_covering_one_slot(
+            "crash-judge-gpu-fl-next-mount-version",
+        ),
+        "I-5.4",
+    );
+}
+
+/// 三个数据单元的第一个文件之后可写挂载（取号 2、写行、暖机两次）、实例 2 再覆盖写一次，然后从流里拿掉顺序写那次发布（txg 4）的根槽写：
+/// txg 4 那一版只在它的三条记录里，挂载之前的状态上它是下一次挂载先施加的那一版，挂载写了实例 1 的行之后它是由记录施加出来、
+/// 根槽从没落盘的那一版（攻方报告 P1-b 的 s4 形）。`rewrite_the_records` 再改那次发布的记录（重封、重链）。
+/// 挂载在录制时读到的是 txg 4 的根；拿掉根槽写之后，恢复施加那三条记录重建出的是同一版（新根段与根同七段，D23（journal 的角色与格式）
+/// 已定项 15），挂载写出的行与之后的发布不变。
+fn record_three_unit_file_applied_only_by_its_records_after_a_mount(
+    tag: &str,
+    name: &'static str,
+    rewrite_the_records: &dyn Fn(&mut RecordedFlow),
+) -> RecordedFlow {
+    let mut pool = pool_with_a_three_unit_file(tag);
+    let mut devices_reopened_for_the_second_instance = pool.reopen_recorded();
+    let mounted = singlefs_core::mount::mount_writable(
+        &parameters(),
+        &mut devices_reopened_for_the_second_instance,
+    )
+    .expect("重开取号 2");
+    pool.devices = Some(devices_reopened_for_the_second_instance);
+    pool.allocator = mounted.allocator;
+    let current = mounted
+        .current
+        .into_file_version()
+        .expect("三个数据单元那一版之后重开，现行那一版带文件");
+    let overwrite = common::publish_overwrite_in_process(
+        &mut pool,
+        &current,
+        &third_content(),
+        common::FIXED_WRITE_TIME_SECONDS + 120,
+        InstanceGeneration(2),
+    )
+    .expect("实例 2 覆盖写");
+    let overwrite_txg = overwrite.root.checkpoint_txg.0;
+    let mut versions = vec![
+        PublishedVersion {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(3),
+            content: file_content(),
+        },
+        PublishedVersion {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(4),
+            content: content_of_three_data_units(),
+        },
+    ];
+    for txg in 5..overwrite_txg {
+        versions.push(PublishedVersion {
+            instance: InstanceGeneration(2),
+            checkpoint_txg: CheckpointTxg(txg),
+            content: content_of_three_data_units(),
+        });
+    }
+    versions.push(PublishedVersion {
+        instance: InstanceGeneration(2),
+        checkpoint_txg: CheckpointTxg(overwrite_txg),
+        content: third_content(),
+    });
+    let mut flow = recorded_flow_of_the_pool(&pool, name, versions);
+    let root_of_the_three_unit_publish = flow
+        .writes
+        .iter()
+        .position(|write| {
+            write.kind == StepKind::RootRecordFua
+                && write.bytes().is_some_and(|bytes| {
+                    u32::from_le_bytes(bytes[24..28].try_into().expect("4")) == 1
+                        && u64::from_le_bytes(bytes[28..36].try_into().expect("8")) == 4
+                })
+        })
+        .expect("流里有顺序写那次发布 (1, 4) 的根槽写");
+    flow.writes.remove(root_of_the_three_unit_publish);
+    for segment in &mut flow.segments {
+        segment.retain(|index| *index != root_of_the_three_unit_publish);
+        for index in segment.iter_mut() {
+            if *index > root_of_the_three_unit_publish {
+                *index -= 1;
+            }
+        }
+    }
+    flow.segments.retain(|segment| !segment.is_empty());
+    flow.judged_root_index = flow
+        .writes
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .expect("覆盖写那次发布有根槽写");
+    rewrite_the_records(&mut flow);
+    flow
+}
+
+/// 改法 FL 由记录施加出来的那一版那一支（攻方报告 P1-b 的 s4 形）：txg 4 只在三条记录里、第一条的新根段写成 txg 3 那条根的。
+/// 挂载之后的状态上 txg 4 是由记录施加出来的那一版：CPU 取末条那一条走它自己的树，GPU 走读内核拼 `R_DATA` 那一段同取末条、
+/// 每个状态逐位相同（取先到的那一条就走 txg 3 的树，txg 4 自己的单元没走到，I-3.1、I-3.5 在 GPU 一侧红出来）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_version_applied_only_by_its_records_is_walked_from_its_last_record_when_the_first_carries_the_previous_root(
+) {
+    let flow = record_three_unit_file_applied_only_by_its_records_after_a_mount(
+        "crash-judge-gpu-fl-version-applied-only-by-records",
+        "three-unit-file-applied-only-by-its-records-stale-first-record",
+        &write_the_root_of_the_first_file_into_the_first_record_of_the_three_unit_publish,
+    );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_version_applied_only_by_its_records_is_walked_from_its_last_record_when_the_first_carries_the_previous_root",
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
+/// Q3 甲（用户 2026-10-06 定「甲 同改」）与 Q1 甲下一次挂载那一支：新池新建文件那次发布的根记录与它的记录（两块盘各一份）里
+/// 树表指针整条写零、重封。根落了的状态上最新根照 core 跟过去读不出（CPU 红 I-2.1、I-4.8、I-7.2、I-7.4），记录落了、根没落的状态上
+/// 重放施加出来的那一版走不完（CPU 红 I-4.1）；GPU 走读内核 `w_walk_tree_table_and_mapping` 同判、每个状态逐位相同。
+fn record_new_pool_with_the_tree_table_pointer_of_the_publish_written_all_zero(
+    tag: &str,
+) -> RecordedFlow {
+    let mut flow =
+        record_new_pool_file_creation(tag, "new-pool-publish-tree-table-pointer-all-zero");
+    rewrite_every_journal_record_of_the_publish(&mut flow, &|bytes| {
+        bytes
+            [JOURNAL_NEW_ROOT_SEGMENT_OFFSET..JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NODE_POINTER_BYTES]
+            .fill(0);
+    });
+    let publish_root = flow.judged_root_index;
+    rewrite_root_record_and_reseal(&mut flow, publish_root, &|bytes| {
+        bytes[36..36 + NODE_POINTER_BYTES].fill(0);
+    });
+    flow
+}
+
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_pointer_of_the_root_and_of_its_record_is_all_zero(
+) {
+    let flow = record_new_pool_with_the_tree_table_pointer_of_the_publish_written_all_zero(
+        "crash-judge-gpu-all-zero-tree-table-pointer",
+    );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_pointer_of_the_root_and_of_its_record_is_all_zero",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    for invariant in ["I-2.1", "I-7.2", "I-4.1"] {
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
+/// Q1 甲由记录施加出来的那一版那一支：同上那条挂载之后的流，txg 4 的三条记录（两块盘各一份）新根段里的树表指针都写零、重封、重链。
+/// 挂载之后的状态上 txg 4 是由记录施加出来的那一版，CPU 照 core 跟过去、这一版走不完（红 I-2.1、I-4.8、I-7.4）；
+/// GPU 走读内核同判。挂载之前三条都落了的状态上它是下一次挂载施加的那一版，重放走不完（I-4.1）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_the_records_of_a_version_applied_only_by_them_carry_an_all_zero_tree_table_pointer(
+) {
+    let flow = record_three_unit_file_applied_only_by_its_records_after_a_mount(
+        "crash-judge-gpu-all-zero-tree-table-pointer-applied-only-by-records",
+        "three-unit-file-applied-only-by-its-records-all-zero-tree-table-pointer",
+        &|flow| {
+            let records = journal_record_writes_of_the_publish(flow, 4);
+            for (index, _) in records {
+                rewrite_journal_record_and_reseal(flow, index, &|bytes| {
+                    bytes[JOURNAL_NEW_ROOT_SEGMENT_OFFSET
+                        ..JOURNAL_NEW_ROOT_SEGMENT_OFFSET + NODE_POINTER_BYTES]
+                        .fill(0);
+                });
+            }
+        },
+    );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_when_the_records_of_a_version_applied_only_by_them_carry_an_all_zero_tree_table_pointer",
+    ) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    for invariant in ["I-2.1", "I-4.8", "I-7.4"] {
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
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index e7562dd..0f76c95 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -665,15 +665,12 @@ fn judge_the_appearance_rule_of_adjacent_candidate_versions(
     }
 }
 
-/// 走读遇到整条全零的树表指针时怎么读（用户 2026-10-05 定甲，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第五节第 1 条）。
+/// 一版的树表指针与映射根指针是从哪里读来的：根环里的根读根记录，由记录造出来的版本读那条记录的新根段（D23（journal 的角色与格式）
+/// 已定项 15）。两处的树表指针整条全零时读法相同，只在说明里分开写。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
-enum AllZeroTreeTablePointerReading {
-    /// 根环里自证过的根：照 core 的根记录读者跟过去、读不出，判这条根损坏
-    /// （[`Walk::follow_an_all_zero_tree_table_pointer_of_a_root_record`]）。
-    FollowedLikeTheRootRecordReaderOfTheCore,
-    /// 由记录施加出来的那一版（树表指针在记录新根段里）：用户定的只是自证过的根，这一类照旧当这一版没有树表、走读不记失败
-    /// （要不要也判损坏交主 agent）。
-    NoTreeTable,
+enum TreeTablePointerHolder {
+    RootRecord,
+    NewRootSegmentOfTheRecord,
 }
 
 /// 走读此刻在走哪一版（D16（发布语义） 已定项 1 的回退候选集里的根，或由记录施加出来、根槽从没落盘的那一版），带它的 `checkpoint_txg`。
@@ -1416,13 +1413,17 @@ impl<'reader> Walk<'reader> {
             &record[36..122],
             &record[256..342],
             &version_name,
-            AllZeroTreeTablePointerReading::FollowedLikeTheRootRecordReaderOfTheCore,
+            TreeTablePointerHolder::RootRecord,
         );
     }
 
     /// 由记录施加出来、根槽从没落盘的那一版（[`versions_applied_only_by_records`] 认出来的）：它没有根记录，
     /// 树表指针、映射根指针、实例表指针与「树表 0 条那一版的分配记录树的根」都在那条记录的新根段里（D23（journal 的角色与格式）
     /// 已定项 15：施加一条记录 = 把所选根的七个字段换成记录里的，用户 2026-10-02「回到甲」），从那里走下去，判定与根环里的候选根同一套。
+    /// 新根段里树表指针整条全零时与根环里的根同一读法（用户 2026-10-06 定 Q1「甲 两类都判损坏」，判决
+    /// `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第五节第 1 条）：core 施加记录之后拼出的根交给同一个根记录读者，
+    /// 照两条位置条目读盘 0 槽 0、读不出（[`Walk::follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core`]），
+    /// 主走读里这一版记一次走读失败，I-4.1 那一份另起的走读（重放施加出来的那一版也走这里）判红。
     fn walk_version_applied_only_by_records(&mut self, version: &VersionAppliedOnlyByRecords) {
         self.begin_walking_a_version();
         self.version_being_walked = VersionBeingWalked::AppliedOnlyByRecords {
@@ -1450,7 +1451,7 @@ impl<'reader> Walk<'reader> {
             tree_table_pointer,
             mapping_root_pointer,
             &version_name,
-            AllZeroTreeTablePointerReading::NoTreeTable,
+            TreeTablePointerHolder::NewRootSegmentOfTheRecord,
         );
     }
 
@@ -1458,33 +1459,31 @@ impl<'reader> Walk<'reader> {
     /// 由记录施加出来的那一版从记录的新根段里取（两处都是 86 字节的码 2 指针）。树表读不出时映射根也不走（走读已经断了）；
     /// 树表别的版本先走过时条目不重走，映射根照样往下走（共享的就并进这一版）。
     /// `version_name` 是这一版的称呼（哪条根、或由哪条记录施加出来），I-9.16 判红时写进第一处违例。
+    /// 树表指针整条全零时两类都照 core 的根记录读者跟过去（[`Walk::follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core`]），
+    /// 映射根不走；`tree_table_pointer_holder` 只用来在说明里写明这条指针是从哪里读来的。
     fn walk_tree_table_and_central_mapping_root(
         &mut self,
         tree_table_pointer: &[u8],
         mapping_root_pointer: &[u8],
         version_name: &str,
-        all_zero_tree_table_pointer_reading: AllZeroTreeTablePointerReading,
+        tree_table_pointer_holder: TreeTablePointerHolder,
     ) {
         // 树表单元：不属于任何树（树 ID 0）。
         let tree_table_pointer_view = parse_node_pointer(tree_table_pointer);
         if tree_table_pointer_view.all_zero {
-            match all_zero_tree_table_pointer_reading {
-                AllZeroTreeTablePointerReading::FollowedLikeTheRootRecordReaderOfTheCore => {
-                    self.follow_an_all_zero_tree_table_pointer_of_a_root_record(
-                        &tree_table_pointer_view,
-                        version_name,
-                    );
-                    return;
-                }
-                AllZeroTreeTablePointerReading::NoTreeTable => {}
-            }
+            self.follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core(
+                &tree_table_pointer_view,
+                version_name,
+                tree_table_pointer_holder,
+            );
+            return;
         }
         let tree_table_placement = (
             tree_table_pointer_view.locations[0].device,
             tree_table_pointer_view.locations[0].slot,
         );
         let tree_table_walked_by_an_earlier_version =
-            !tree_table_pointer_view.all_zero && self.visited_units.contains(&tree_table_placement);
+            self.visited_units.contains(&tree_table_placement);
         let tree_table = self.read_index_node(
             tree_table_pointer,
             0,
@@ -1532,18 +1531,26 @@ impl<'reader> Walk<'reader> {
         self.walk_central_mapping_entries(&leaf_entries);
     }
 
-    /// 根环里一条自证过的根，根记录里树表指针整条全零（用户 2026-10-05 定甲，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md`
-    /// 第五节第 1 条「甲 checker 判它损坏」）：core 的根记录读者不把全零当特例，照它那两条位置条目（都是盘 0 槽 0）去读、读不出
-    /// （可写挂载拒成 `Recovery(UnitUnreadable { slot: SlotNumber(0) })`）。checker 跟过去：两条位置条目照 I-2.1 判，这条根的走读记一次失败
-    /// （最新根红 I-7.2、I-4.8、I-7.4，别的候选根红 I-4.8、I-7.4），不再读成「这条根没有树表」。
-    /// 不判 I-2.5：那一行把整条全零的指针整条豁免于升序判定，豁免要不要收窄归写回批。
-    fn follow_an_all_zero_tree_table_pointer_of_a_root_record(
+    /// 一版的树表指针整条全零：根环里自证过的根（用户 2026-10-05 定甲，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md`
+    /// 第五节第 1 条「甲 checker 判它损坏」），以及由记录造出来的两类版本——由记录施加出来的那几版、重放施加出来的那一版
+    /// （用户 2026-10-06 定 Q1「甲 两类都判损坏」，判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第五节第 1 条）。
+    /// core 的根记录读者不把全零当特例，照它那两条位置条目（都是盘 0 槽 0）去读、读不出（可写挂载拒成
+    /// `Recovery(UnitUnreadable { slot: SlotNumber(0) })`；施加记录之后拼出的根交给同一个读者）。checker 跟过去：两条位置条目照 I-2.1 判，
+    /// 这一版的走读记一次失败（最新根红 I-7.2、I-4.8、I-7.4，别的候选根与由记录施加出来的那几版红 I-4.8、I-7.4，重放施加出来的那一版红 I-4.1），
+    /// 不再读成「这一版没有树表」。
+    /// 不判 I-2.5：那一行把整条全零的指针整条豁免于升序判定（用户 2026-10-06 定 Q2「乙 照今天，不红 I-2.5」）。
+    fn follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core(
         &mut self,
         tree_table_pointer: &PointerView,
         version_name: &str,
+        tree_table_pointer_holder: TreeTablePointerHolder,
     ) {
+        let holder = match tree_table_pointer_holder {
+            TreeTablePointerHolder::RootRecord => "根记录",
+            TreeTablePointerHolder::NewRootSegmentOfTheRecord => "记录新根段",
+        };
         let what = format!(
-            "{version_name}的树表（根记录里树表指针全零，照 core 的根记录读者读它那两条位置条目）"
+            "{version_name}的树表（{holder}里树表指针全零，照 core 的根记录读者读它那两条位置条目）"
         );
         let unit = read_referenced_unit(
             self.reader,
@@ -4863,16 +4870,25 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
         );
     }
     // 一版都没认出来时的专门理由（判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P3「四支豁免」那一行：
-    // 此前落到 `Judgements::into_report` 的缺省理由，与「这个函数一次都没跑」分不开）。认出来了而没有数据单元的那一形照旧落到缺省理由
-    // （派发只要了这一句）。`not_applicable` 不改已经评估过的。
+    // 此前落到 `Judgements::into_report` 的缺省理由，与「这个函数一次都没跑」分不开）。认出来了而一版都没判到（认出来的那几版都没有数据单元）
+    // 另写一句（用户 2026-10-06 定 Q4「甲 另写一句」，判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第五节第 4 条）。
+    // `not_applicable` 不改已经评估过的：认出来的版本里有一版判到了，这一句不出现。
     if versions_built_from_the_mkfs_watermark == 0 {
         judgements.not_applicable(invariant, NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK);
+    } else {
+        judgements.not_applicable(
+            invariant,
+            VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT,
+        );
     }
 }
 
 /// I-5.5 一次都没判、因为走读到的版本里没有一版满足认法 ①② 时的不适用理由。
 pub const NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK: &str = "回退候选集里每条根与由记录施加出来的那几版，没有一版是从 mkfs 水位第一次建树的那一版（树表非空、种类 1 条目的树 ID 是 65536、每条条目的诞生 txg 等于这一版的 txg）：一版都不判";
 
+/// I-5.5 一次都没判、因为认出来的从 mkfs 水位第一次建树的那几版都没有数据单元时的不适用理由（那一行「V 没有数据单元时不判」）。
+pub const VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT: &str = "认出了从 mkfs 水位第一次建树的那一版，但它们都没有数据单元（extent 树根全零或一条数据指针都没有）：一版都不判";
+
 /// 一块盘上的开放段：起点是 64 的整数倍的 [`CLUSTER_SEGMENT_SLOTS`] 个槽（D3（空间分配） 已定项 10 ①）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct OpenClusterSegment {
@@ -5229,7 +5245,7 @@ fn judge_tree_identifier_birth_txg_across_every_readable_root(
         judgements.partially_judged(
             TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
             format!(
-                "比到的 {compared_trees} 棵树诞生 txg 都对得上；这几条读得出、自证过的根树表指针非零而条目没读全，没进比较：{}",
+                "比到的 {compared_trees} 棵树诞生 txg 都对得上；这几条读得出、自证过的根树表条目没读全（树表指针全零、树表读不出或有条目比字段表短），没进比较：{}",
                 roots_whose_tree_table_was_not_read_in_full.join("；")
             ),
         );
@@ -5526,8 +5542,10 @@ fn birth_txg_in_a_usable_unit_header(header: &[u8]) -> Option<u64> {
 /// 最新根之后、下一次挂载的恢复会先施加的那一版的树表指针（改法 E，`research/prompts/m2-wave3-code-r1-main-verification.md`
 /// 第三节 Y4 与第四节第 2 条，被攻过零轮）：环里一条记录与最新根**同实例**、checkpoint_txg = 最新根的 txg + 1、带提交标记，
 /// 根环里没有 (那个实例, 那个 txg) 的根——它那次发布的记录已持久、根槽没落盘，恢复要由记录重建那一版（D23（journal 的角色与格式）
-/// 已定项 15）。树表指针取那条记录新根段里的（D23（journal 的角色与格式） 已定项 4 的字段表）；一版切成几条记录、两块盘各一份时
-/// 它们带的是同一版的指针，取先读到的那一条。
+/// 已定项 15）。树表指针取那条记录新根段里的（D23（journal 的角色与格式） 已定项 4 的字段表）。一版切成几条记录时认的是
+/// **带末条标志的那一条**（改法 FL，判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P1-b 与第三节第 1 条）：
+/// core 的恢复走到末条才整次施加、施加末条那一份新根段（`crates/singlefs-core/src/recovery.rs` 拼新根那一段）；末条还没落的那次发布
+/// core 不施加，这里也不认这一版。两块盘各一份时取先读到的那一份。I-3.10 与 I-5.4 都经这一个函数认这一版，同一个条件。
 ///
 /// **只接这一步**：恢复会连着施加几条记录时，第二条及以后那几版的分配记录这里读不到；恢复的前缀规则只复刻反向链那一道
 /// （I-8.6（反向链算法）「不等的记录不进重放前缀」，[`record_can_enter_a_replay_prefix`]），断号即止、点名单元验证、发布边界不复刻——
@@ -5542,7 +5560,7 @@ fn tree_table_pointer_of_the_version_the_next_mount_applies_first(
     })
 }
 
-/// 下一次挂载的恢复会先施加的那一版：记录已持久、根槽没落盘、还没有哪一次恢复施加过它（与 [`VersionAppliedOnlyByRecords`] 不是一类：
+/// 下一次挂载的恢复会先施加的那一版：那次发布的末条记录已持久、根槽没落盘、还没有哪一次恢复施加过它（与 [`VersionAppliedOnlyByRecords`] 不是一类：
 /// 那一类已被施加、实例表里有行）。
 struct VersionTheNextMountAppliesFirst {
     instance: u32,
@@ -5571,7 +5589,8 @@ fn version_the_next_mount_applies_first(
     records_as_the_recovery_reads_them
         .iter()
         .find(|((_, counter), record)| {
-            record.commit_marker == CommitMarker::Present
+            record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0
+                && record.commit_marker == CommitMarker::Present
                 && record.instance == newest_root.instance
                 && record.checkpoint_txg == next_txg
                 && record_can_enter_a_replay_prefix(
@@ -7728,6 +7747,13 @@ struct VersionAppliedOnlyByRecords {
     new_root_segment: Vec<u8>,
 }
 
+/// [`versions_applied_only_by_records`] 给一版取新根段时取的是哪一条记录的：带末条标志的那一条一到就换上它、之后不再换；
+/// 在它之前只读到同一次发布别的记录时先占着先到的那一条。
+enum SegmentTakenForTheVersion {
+    FromTheLastRecordOfThePublish(Vec<u8>),
+    FromTheFirstRecordReadBecauseNoLastRecordWasSeen(Vec<u8>),
+}
+
 impl VersionAppliedOnlyByRecords {
     fn tree_table_and_mapping_root_pointers(&self) -> (&[u8], &[u8]) {
         (
@@ -7755,6 +7781,11 @@ impl VersionAppliedOnlyByRecords {
 ///    再把它并进来就是拿已经不归它的槽去比。
 ///
 /// 一版由那条记录的 (实例代号, checkpoint_txg) 认；两块盘各一份镜像记录，取先读到的那一份。
+/// 一次发布切成几条记录时，新根段取带末条标志的那一条的（改法 FL，判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md`
+/// 第二节 P1-b 与第三节第 1 条）：core 的恢复走到带末条标志的那一条才整次施加，施加的是那一条的新根段
+/// （`crates/singlefs-core/src/recovery.rs` 拼新根那一段「施加末条那一份」）。合法写者每条记录都带同一份新根段（D23（journal 的角色与格式）
+/// 已定项 15），两种取法同一份字节；坏写者把前几条写成别的根时，取先到的那条就是拿 core 没施加的那一版去判。
+/// 环里读不到带末条标志的那一条时退回先到的那一条（这一版仍被实例表的行判为施加过，只是末条这一刻读不出）。
 fn versions_applied_only_by_records(
     records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
     roots: &[(u64, u64, crate::RootView)],
@@ -7774,7 +7805,7 @@ fn versions_applied_only_by_records(
     else {
         return Vec::new();
     };
-    let mut versions: BTreeMap<(u32, u64), Vec<u8>> = BTreeMap::new();
+    let mut versions: BTreeMap<(u32, u64), SegmentTakenForTheVersion> = BTreeMap::new();
     // 按恢复的读法（每个计数器取第一块盘上的那一份）：反向链对不上的记录不进重放前缀（I-8.6（反向链算法）），不算被哪一次恢复施加过。
     let records_as_the_recovery_reads_them = records_as_the_recovery_reads_them(records_by_device);
     for ((_, counter), record) in &records_as_the_recovery_reads_them {
@@ -7798,18 +7829,43 @@ fn versions_applied_only_by_records(
             && at_or_above_the_floor
             && its_units_are_not_reclaimable_yet
         {
-            versions
-                .entry((record.instance, record.checkpoint_txg))
-                .or_insert_with(|| record.new_root_segment.clone());
+            let carries_the_last_record_flag =
+                record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0;
+            let taken_so_far = versions.get(&(record.instance, record.checkpoint_txg));
+            let takes_this_record = match taken_so_far {
+                None => true,
+                Some(SegmentTakenForTheVersion::FromTheLastRecordOfThePublish(_)) => false,
+                Some(
+                    SegmentTakenForTheVersion::FromTheFirstRecordReadBecauseNoLastRecordWasSeen(_),
+                ) => carries_the_last_record_flag,
+            };
+            if takes_this_record {
+                let new_root_segment = record.new_root_segment.clone();
+                versions.insert(
+                    (record.instance, record.checkpoint_txg),
+                    if carries_the_last_record_flag {
+                        SegmentTakenForTheVersion::FromTheLastRecordOfThePublish(new_root_segment)
+                    } else {
+                        SegmentTakenForTheVersion::FromTheFirstRecordReadBecauseNoLastRecordWasSeen(
+                            new_root_segment,
+                        )
+                    },
+                );
+            }
         }
     }
     versions
         .into_iter()
         .map(
-            |((instance, checkpoint_txg), new_root_segment)| VersionAppliedOnlyByRecords {
+            |((instance, checkpoint_txg), taken)| VersionAppliedOnlyByRecords {
                 instance,
                 checkpoint_txg,
-                new_root_segment,
+                new_root_segment: match taken {
+                    SegmentTakenForTheVersion::FromTheLastRecordOfThePublish(new_root_segment)
+                    | SegmentTakenForTheVersion::FromTheFirstRecordReadBecauseNoLastRecordWasSeen(
+                        new_root_segment,
+                    ) => new_root_segment,
+                },
             },
         )
         .collect()
@@ -7869,15 +7925,27 @@ fn judge_blocks_referenced_by_abandoned_roots(
         let walked_into_a_reallocated_or_erased_unit =
             first_copy_that_does_not_match.is_some() || !walk_failures.is_empty();
         let (instance, root_txg) = (root.instance, root.checkpoint_txg);
+        // 树表指针整条全零的根：走读照 core 的根记录读者读盘 0 槽 0、读不出（`follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core`），
+        // 红的是这条根自己坏了，说明不写成「引用的单元已被重新分配或抹头」（判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md`
+        // 第二节 P2-e 与第三节第 1 条，攻方读码报的那一句）。判定不变。
+        let tree_table_pointer_is_all_zero =
+            parse_node_pointer(&root.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER]).all_zero;
         judgements.judge("I-7.4", !walked_into_a_reallocated_or_erased_unit, || {
             let read_back_wrong: Vec<String> = first_copy_that_does_not_match
                 .into_iter()
                 .chain(walk_failures)
                 .collect();
-            format!(
-                "被抛弃的根（实例 {instance}、txg {root_txg}）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：{}",
-                read_back_wrong.join("；")
-            )
+            if tree_table_pointer_is_all_zero {
+                format!(
+                    "被抛弃的根（实例 {instance}、txg {root_txg}）走不完：根记录里树表指针全零，core 的根记录读者照它读盘 0 槽 0、读不出（这条根本身损坏，不只是它引用的单元被重新分配或抹头）：{}",
+                    read_back_wrong.join("；")
+                )
+            } else {
+                format!(
+                    "被抛弃的根（实例 {instance}、txg {root_txg}）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：{}",
+                    read_back_wrong.join("；")
+                )
+            }
         });
     }
 }
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index a8c5fce..d76507f 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -5979,12 +5979,15 @@ fn widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
     0
 }
 
-/// 一次可写挂载接在「由记录施加出来、根槽从没落盘」的那一版 (1, 5) 之后的镜像上（[`image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record`]）：
-/// 改 (1, 5) 这次发布自己重写的第一片能改的分配记录叶（[`widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts`]），
-/// 叶重封、新的整单元校验和沿引用链补上去（叶 → 内部节点 → 根 → 树表），再补进 (1, 5) 那条记录（两块盘各一份）。交回改的是哪一片叶。
-fn overlap_two_records_in_the_version_applied_only_by_its_journal_record(
+/// 改 `version` 这次发布自己重写的第一片能改的分配记录叶（[`widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts`]），
+/// 叶重封、新的整单元校验和沿引用链补上去（叶 → 内部节点 → 根 → 树表），再补进那次发布的记录：`record_copies_of_the_publish` 是
+/// 那次发布的记录条数乘两块盘（一条记录的发布是 2，三条记录的是 6），只有它们引用这一版改过的单元。交回改的是哪一片叶。
+/// 例：一次可写挂载接在「由记录施加出来、根槽从没落盘」的那一版 (1, 5) 之后的镜像上
+/// （[`image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record`]）改 (1, 5) 那一版。
+fn overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(
     image: &mut MemoryPool,
     version: &TransactionOutput,
+    record_copies_of_the_publish: usize,
 ) -> u64 {
     let units = units_found_on_device_zero(image);
     let checksums_before = unit_checksums(image, &units);
@@ -6006,7 +6009,7 @@ fn overlap_two_records_in_the_version_applied_only_by_its_journal_record(
                 &mut candidate_leaf,
             ) == 1
         })
-        .expect("(1, 5) 这次重写的分配记录叶里有一对同盘、首尾相接的记录");
+        .expect("这次发布重写的分配记录叶里有一对同盘、首尾相接的记录");
     mutate_unit_of(
         image,
         &units,
@@ -6023,8 +6026,8 @@ fn overlap_two_records_in_the_version_applied_only_by_its_journal_record(
     let changes = units_whose_checksum_changed(&checksums_before, &unit_checksums(image, &units));
     assert_eq!(
         propagate_into_journal_records(image, &changes),
-        2,
-        "只有 (1, 5) 那条记录（两块盘各一份）引用这一版改过的单元"
+        record_copies_of_the_publish,
+        "只有这次发布的记录（两块盘各一份）引用这一版改过的单元"
     );
     leaf
 }
@@ -6046,9 +6049,10 @@ fn two_allocation_records_covering_one_slot_in_a_version_applied_only_by_its_jou
         "挂载之后的干净镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
     );
     let mut image = built.image.clone();
-    let leaf = overlap_two_records_in_the_version_applied_only_by_its_journal_record(
+    let leaf = overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(
         &mut image,
         &built.version_applied_only_by_its_record,
+        2,
     );
     let verdicts = check_pool_image(&image);
     assert_eq!(
@@ -9487,6 +9491,14 @@ fn an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_th
         ),
         "I-9.28 点名树表指针全零的那条根：{tree_identifier_unjudged}"
     );
+    // 单子里有树表指针全零的根，前缀不再说「树表指针非零」（判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md`
+    // 第二节 P2-e：两种都说得通的说法）。
+    assert!(
+        tree_identifier_unjudged.contains(
+            "这几条读得出、自证过的根树表条目没读全（树表指针全零、树表读不出或有条目比字段表短），没进比较："
+        ),
+        "I-9.28 部分没判的前缀把树表指针全零的根也说得通：{tree_identifier_unjudged}"
+    );
     let deletion_scan_gap = InvariantVerdict::NotApplicable(
         "走读到的某一版树表指针全零（core 照它读盘 0 槽 0、读不出）：那一版的删除态射程里判不了",
     );
@@ -10956,3 +10968,457 @@ fn an_inode_tree_entry_with_an_unregistered_type_over_a_child_that_does_not_read
         verdict(&image, "I-9.2")
     );
 }
+
+// ─── 第 35 批：由记录造出来的两类版本取哪条记录的新根段（FL）、树表指针全零（Q1、被抛弃的根）、I-5.5 认出来而没有数据单元（Q4） ───
+
+/// 恰好要 `data_units` 个数据单元的内容（最后一个单元装一半）：顺序写一次发布切成 `data_units` 条记录（一个数据单元一条）。
+fn content_needing_data_units(data_units: usize) -> Vec<u8> {
+    let payload_capacity = singlefs_core::unit::data_unit_payload_capacity();
+    (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
+        .map(|index| u8::try_from((index * 13 + 8) % 251).expect("小于 256"))
+        .collect()
+}
+
+/// 新池新建文件 A (1, 3) 之后顺序写三个数据单元的发布 B (1, 4)：B 切成三条记录（计数器 4、5、6，每条带提交标记、只有 6 带末条标志），
+/// 两块盘各一份；崩在 B 的根槽 FUA 之前（攻方报告 `research/prompts/m3-formal-code-b34-r1-opus-output.md` P1-b 那张 crafted 镜像的同形）。
+/// 最新的根还是 A 的，实例表里没有实例 1 的行：B 是下一次挂载先施加的那一版。交回镜像与 A、B 两次发布的输出。
+struct CrashBeforeTheRootOfAThreeRecordPublish {
+    image: MemoryPool,
+    previous: TransactionOutput,
+    publish: TransactionOutput,
+}
+
+fn crash_image_with_a_three_record_publish_but_not_its_root(
+    tag: &str,
+) -> CrashBeforeTheRootOfAThreeRecordPublish {
+    let mut pool = build_pool(tag);
+    let operations_through_the_first_publish = pool.retained_operations().len();
+    let previous = pool.output.clone();
+    let content = content_needing_data_units(3);
+    let publish = {
+        let publish_parameters = parameters();
+        let devices = pool.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        publish_sequential_write(
+            &mut writer,
+            &mut pool.allocator,
+            &previous,
+            FirstFile {
+                content: &content,
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
+            },
+            InstanceGeneration(1),
+        )
+        .expect("顺序写三个数据单元")
+    };
+    let counters: Vec<u64> = publish
+        .earlier_records_of_this_publish
+        .iter()
+        .map(|written| written.record.counter)
+        .chain([publish.record.counter])
+        .collect();
+    assert_eq!(
+        (counters, publish.root.checkpoint_txg),
+        (vec![4, 5, 6], CheckpointTxg(4)),
+        "B 是 txg 4、切成计数器 4、5、6 三条记录"
+    );
+    let operations = pool.retained_operations();
+    let classifier = geometry();
+    let kinds_of_the_publish: Vec<StepKind> = operations[operations_through_the_first_publish..]
+        .iter()
+        .map(|retained| classifier.classify(&retained.operation))
+        .collect();
+    let root_record_step = kinds_of_the_publish
+        .iter()
+        .position(|kind| *kind == StepKind::RootRecordFua)
+        .expect("发布 B 有一步根槽 FUA");
+    assert_eq!(
+        (
+            kinds_of_the_publish[..root_record_step]
+                .iter()
+                .filter(|kind| **kind == StepKind::JournalRecord)
+                .count(),
+            kinds_of_the_publish[root_record_step - 1]
+        ),
+        (6, StepKind::Barrier),
+        "根槽 FUA 之前：B 的三条记录两块盘各写了一份，紧挨着根槽的是记录之后那道屏障"
+    );
+    let mut image = pool.memory_pool_after_mkfs();
+    image.apply(
+        &operations
+            [pool.mkfs_operation_count..operations_through_the_first_publish + root_record_step],
+    );
+    CrashBeforeTheRootOfAThreeRecordPublish {
+        image,
+        previous,
+        publish,
+    }
+}
+
+/// 把 B 的第一条记录（计数器 4，两块盘各一份）的新根段七项换成上一版 A 那条根的（写侧违反 D23（journal 的角色与格式） 已定项 15
+/// 「每条记录都带同一份新根段」的样子），重封、后面两条重链（攻方报告 P1-b 的 s1 改法）。
+fn write_the_root_of_the_previous_version_into_the_first_record_of_the_publish(
+    image: &mut MemoryPool,
+    crash: &CrashBeforeTheRootOfAThreeRecordPublish,
+) {
+    let first_counter = crash
+        .publish
+        .earlier_records_of_this_publish
+        .first()
+        .expect("B 有三条记录")
+        .record
+        .counter;
+    let stale = &crash.previous.root;
+    rewrite_journal_record_and_rechain_the_records_after_it(image, first_counter, |record| {
+        record.new_tree_table = stale.tree_table;
+        record.new_mapping_root = stale.mapping_root;
+        record.new_tree_identifier_watermark = stale.tree_identifier_watermark;
+        record.new_rollback_floor = stale.rollback_floor;
+        record.new_instance_table = stale.instance_table;
+        record.new_allocation_record_tree_root = stale.allocation_record_tree_root;
+        record.new_unmount_marker = stale.unmount_marker;
+    });
+}
+
+/// 改法 FL（判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P1-b，攻方报告 P1-b 的 s3 形）：
+/// 下一次挂载先施加的那一版 B 切成三条记录，第一条的新根段写成上一版 A 的，B 自己重写的那片分配记录叶里两条记录罩住同一个槽
+/// （新的整单元校验和沿引用链补到 B 的树表，再补进 B 的三条记录）。core 恢复走到带末条标志的那一条才整次施加、施加的是它的新根段，
+/// 重建 B 的分配器时拒在「同一块盘上两条分配记录罩住同一个槽」上。checker 认这一版也取末条那一条：只红 I-5.4，点名下一次挂载先施加的那一版；
+/// 取先到的那一条时走的是 A 的树，这张镜像一条都不红。
+#[test]
+fn a_stale_first_record_of_the_publish_the_next_mount_applies_first_does_not_hide_two_allocation_records_covering_one_slot(
+) {
+    let crash = crash_image_with_a_three_record_publish_but_not_its_root(
+        "known-bad-stale-first-record-over-overlapping-records-of-the-next-mount-version",
+    );
+    let verdicts_on_the_clean_image = check_pool_image(&crash.image);
+    assert_eq!(
+        violated_invariants(&verdicts_on_the_clean_image),
+        Vec::<&str>::new(),
+        "B 的三条记录已落、根槽没落的干净崩溃镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
+    );
+    let mut image = crash.image.clone();
+    overlap_two_records_in_an_allocation_leaf_the_publish_rewrote(&mut image, &crash.publish, 6);
+    write_the_root_of_the_previous_version_into_the_first_record_of_the_publish(&mut image, &crash);
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-5.4"],
+        "第一条记录写旧了也不许把 B 的两条记录罩住同一个槽藏起来：只红 I-5.4：{verdicts:?}"
+    );
+    let InvariantVerdict::Violated(detail) = verdict(&image, "I-5.4") else {
+        panic!("上面判过 I-5.4 红");
+    };
+    assert!(
+        detail.starts_with("下一次挂载先施加的那一版（实例 1、txg 4）那棵分配记录树"),
+        "红在下一次挂载先施加的那一版 B 的分配记录树上：{detail}"
+    );
+
+    let stream = SharedStream::new();
+    let mut devices = crash_state_devices(&image, &[], &[], &stream);
+    let image_before_the_mount = memory_pool_of_sparse_devices(&devices);
+    let refusal = match mount_writable(&parameters(), &mut devices) {
+        Ok(mounted) => panic!(
+            "坏镜像被可写挂载收下了：择根 txg {:?}、现行版 txg {:?}、施加记录前缀 {} 条",
+            mounted.output.chosen_root.checkpoint_txg,
+            mounted.output.effective_root.checkpoint_txg,
+            mounted.output.journal.prefix_applied
+        ),
+        Err(refusal) => refusal,
+    };
+    assert!(
+        matches!(
+            refusal,
+            MountError::Recovery(RecoveryFailure::AllocationRecordOutsideThePoolGeometry {
+                what: "同一块盘上两条分配记录罩住同一个槽",
+            })
+        ),
+        "core 施加的是末条那一份新根段（B 的树），重建分配器时拒在两条记录罩住同一个槽上：{refusal:?}"
+    );
+    assert_eq!(
+        memory_pool_of_sparse_devices(&devices),
+        image_before_the_mount,
+        "拒在任何写之前：两块盘逐字节不变（{refusal:?}）"
+    );
+}
+
+/// 改法 FL 由记录施加出来的那一版那一支（攻方报告 P1-b 的 s4 形）：B 的第一条记录写成 A 的新根段（只这一处，不叠别的坏法），
+/// 可写挂载照常收下（core 施加末条那一份，现行版是 B (1, 4)），写行发布与暖机之后实例 2 再覆盖写一次：B 成了由记录施加出来、
+/// 根槽从没落盘的那一版。checker 取末条那一条走 B 自己的树，一条都不红；取先到的那一条时走的是 A 的树、B 自己的单元没走到，
+/// I-3.1 与 I-3.5 红。
+#[test]
+fn a_version_applied_only_by_its_records_whose_first_record_carries_the_previous_root_is_walked_from_its_last_record(
+) {
+    let crash = crash_image_with_a_three_record_publish_but_not_its_root(
+        "known-good-stale-first-record-of-a-version-applied-only-by-its-records",
+    );
+    let mut image = crash.image.clone();
+    write_the_root_of_the_previous_version_into_the_first_record_of_the_publish(&mut image, &crash);
+    let stream = SharedStream::retaining_contents();
+    let mut devices = crash_state_devices(&image, &[], &[], &stream);
+    let mut mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
+    assert_eq!(
+        (
+            mounted.output.chosen_root.checkpoint_txg,
+            mounted.output.effective_root.checkpoint_txg,
+            mounted.output.journal.prefix_applied
+        ),
+        (CheckpointTxg(3), CheckpointTxg(4), 3),
+        "择 A 的根 (1, 3)，施加 B 的三条记录，现行版是 (1, 4)"
+    );
+    let current = mounted
+        .current
+        .into_file_version()
+        .expect("施加了 B，现行那一版带文件");
+    let content: Vec<u8> = (0..2900usize)
+        .map(|index| u8::try_from((index * 11 + 2) % 251).expect("小于 256"))
+        .collect();
+    {
+        let publish_parameters = parameters();
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        publish_overwrite(
+            &mut writer,
+            &mut mounted.allocator,
+            &current,
+            FirstFile {
+                content: &content,
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 180,
+            },
+            InstanceGeneration(2),
+        )
+        .expect("实例 2 再覆盖写一次");
+    }
+    let image_after_the_overwrite = memory_pool_of_sparse_devices(&devices);
+    let verdicts = check_pool_image(&image_after_the_overwrite);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        Vec::<&str>::new(),
+        "B 只由记录施加出来、第一条记录写旧了：checker 与 core 同取末条，走的是 B 自己的树，一条都不红：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict(&image_after_the_overwrite, "I-3.1"),
+        InvariantVerdict::Holds,
+        "I-3.1 要真被评估过且成立（B 自己的单元并进了遍历）"
+    );
+}
+
+/// 计数器为 `counter` 的那条 journal 记录（两块盘各一份）新根段里的树表指针整条写零，经 `JournalRecord::to_bytes` 重封。
+/// 不重链后面的记录：这里改的两条记录（由记录施加出来的 (1, 5)、下一次挂载先施加的 B）在环里都是本实例最后一条，
+/// 下一条（若有）是别的实例的第一条，它的反向链不接在这一条上（`common::rewrite_journal_record_and_rechain_the_records_after_it`
+/// 不分实例照接，会把那一条改坏、I-8.6 红）。
+fn write_an_all_zero_tree_table_pointer_into_the_new_root_segment_of_journal_record(
+    image: &mut MemoryPool,
+    counter: u64,
+) {
+    let filesystem_identifier =
+        singlefs_core::unit::unit_filesystem_identifier(&parameters().filesystem_identifier);
+    let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES).0;
+    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
+    for device in DEVICES {
+        let mut record = JournalRecord::parse(
+            &read(image, device, offset, record_bytes),
+            filesystem_identifier,
+        )
+        .expect("改之前这条记录自证过");
+        assert_eq!(
+            record.counter, counter,
+            "环里那一格是计数器 {counter} 的记录"
+        );
+        record.new_tree_table = singlefs_core::pointer::NodePointer::empty_root();
+        let rewritten = record.to_bytes();
+        assert!(
+            rewritten[JOURNAL_RECORD_NEW_ROOT_SEGMENT_OFFSET
+                ..JOURNAL_RECORD_NEW_ROOT_SEGMENT_OFFSET + 86]
+                .iter()
+                .all(|byte| *byte == 0),
+            "新根段里的树表指针整条全零"
+        );
+        write(image, device, offset, &rewritten);
+    }
+}
+
+/// Q1 甲（用户 2026-10-06 定「甲 两类都判损坏」，判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第五节第 1 条），
+/// 由记录施加出来的那一版那一支（正推探针 `zz_probe_q1` 的 q1b 形）：挂载之后只由记录施加出来的那一版 (1, 5) 那条记录（两块盘各一份）
+/// 新根段里的树表指针整条写零、重封、后面的记录重链。core 施加记录之后拼出的根交给同一个根记录读者，照它读盘 0 槽 0、读不出；
+/// 主走读与根环里的根同一读法跟过去：盘 0 槽 0 那两份对不上（I-2.1），从这一版出发的遍历读不出（I-4.8、I-7.4），这一版自己的单元
+/// 没走到、遍历少算（I-3.1）。改之前读成「这一版没有树表」，只红 I-3.1、I-3.5。
+#[test]
+fn an_all_zero_tree_table_pointer_in_the_record_of_a_version_applied_only_by_its_journal_record_is_followed_like_the_core(
+) {
+    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
+        "known-bad-all-zero-tree-table-pointer-of-a-version-applied-only-by-its-record",
+    );
+    let mut image = built.image.clone();
+    write_an_all_zero_tree_table_pointer_into_the_new_root_segment_of_journal_record(
+        &mut image,
+        built.version_applied_only_by_its_record.record.counter,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-2.1", "I-3.1", "I-4.8", "I-7.4"],
+        "(1, 5) 那条记录里树表指针全零：照 core 跟过去读不出，不再读成没有树表：{verdicts:?}"
+    );
+    let InvariantVerdict::Violated(detail) = verdict(&image, "I-2.1") else {
+        panic!("上面判过 I-2.1 红");
+    };
+    assert!(
+        detail.contains(
+            "由记录施加出来的那一版（实例 1、txg 5）的树表（记录新根段里树表指针全零，照 core 的根记录读者读它那两条位置条目）"
+        ),
+        "I-2.1 写明读不对的是由记录施加出来的那一版的树表、指针从记录新根段里读来：{detail}"
+    );
+}
+
+/// Q1 甲，下一次挂载先施加的那一版那一支（正推探针 `zz_probe_q1` 的 q1 形，攻方报告 P2-b 的 z6 形）：发布 B 的记录已落、根槽没落的崩溃镜像上，
+/// B 那条记录（两块盘各一份）新根段里的树表指针整条写零、重封。两种挂载都拒成「槽 0 读不出」；checker 的重放（I-4.1）施加 B、
+/// 施加出来的那一版照 core 跟过去走不完，I-4.1 红，别的判定与干净的崩溃镜像只差 I-5.4 那一句部分没判（认不出带不带树）。
+/// 改之前这一版被读成「没有树表」，一条都不红。
+#[test]
+fn an_all_zero_tree_table_pointer_in_the_record_of_the_version_the_next_mount_applies_first_reddens_the_replayed_version_invariant(
+) {
+    let clean = crash_image_with_the_overwrite_record_but_not_its_root(
+        "known-bad-all-zero-tree-table-pointer-of-the-version-the-next-mount-applies",
+    );
+    let verdicts_on_the_clean_image = check_pool_image(&clean);
+    let mut image = clean.clone();
+    write_an_all_zero_tree_table_pointer_into_the_new_root_segment_of_journal_record(
+        &mut image,
+        COUNTER_OF_THE_OVERWRITE_RECORD,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        [TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE],
+        "B 那条记录里树表指针全零：重放施加出来的那一版照 core 跟过去走不完：{verdicts:?}"
+    );
+    assert_eq!(
+        verdicts_that_differ_outside(
+            &verdicts,
+            &verdicts_on_the_clean_image,
+            &[TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE, "I-5.4"]
+        ),
+        [],
+        "I-4.1 与 I-5.4 之外每一条的判定与干净的崩溃镜像逐项相同"
+    );
+    assert!(
+        matches!(
+            verdict(&image, "I-5.4"),
+            InvariantVerdict::PartiallyJudged(unjudged)
+                if unjudged.starts_with("树表指针全零的版本（core 照它读盘 0 槽 0、读不出），认不出带不带树")
+                    && unjudged.ends_with("：下一次挂载先施加的那一版（实例 1、txg 4）")
+        ),
+        "I-5.4 照旧把这一版记成树表指针全零、认不出带不带树的部分没判：{:?}",
+        verdict(&image, "I-5.4")
+    );
+    let InvariantVerdict::Violated(detail) =
+        verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE)
+    else {
+        panic!("上面判过 I-4.1 红");
+    };
+    assert!(
+        detail.contains("记录新根段里树表指针全零，照 core 的根记录读者读它那两条位置条目"),
+        "I-4.1 写明施加出来的那一版走不完是因为记录新根段里树表指针全零：{detail}"
+    );
+
+    let stream = SharedStream::new();
+    let mut devices = crash_state_devices(&image, &[], &[], &stream);
+    let image_before_the_mount = memory_pool_of_sparse_devices(&devices);
+    let refusal = match mount_writable(&parameters(), &mut devices) {
+        Ok(mounted) => panic!(
+            "坏镜像被可写挂载收下了：择根 txg {:?}、现行版 txg {:?}、施加记录前缀 {} 条",
+            mounted.output.chosen_root.checkpoint_txg,
+            mounted.output.effective_root.checkpoint_txg,
+            mounted.output.journal.prefix_applied
+        ),
+        Err(refusal) => refusal,
+    };
+    assert!(
+        matches!(
+            refusal,
+            MountError::Recovery(RecoveryFailure::UnitUnreadable {
+                slot: singlefs_core::address::SlotNumber(0)
+            })
+        ),
+        "core 施加 B 之后照全零的树表指针读盘 0 槽 0、读不出：{refusal:?}"
+    );
+    assert_eq!(
+        memory_pool_of_sparse_devices(&devices),
+        image_before_the_mount,
+        "拒在任何写之前：两块盘逐字节不变（{refusal:?}）"
+    );
+}
+
+/// Q4 甲（用户 2026-10-06 定「甲 另写一句」，判决同上第五节第 4 条）：新池新建文件那一版（从 mkfs 水位第一次建树、认法 ①② 都满足）
+/// 树表里 extent 树那一条的根指针整条写零、重封、沿引用链补到根槽：这一版认得出、却一个数据单元都没有，I-5.5 一版都不判，
+/// 报「认出来了而没有数据单元」那一句专门理由，不落到缺省理由、也不落到「一版都没认出来」那一句。
+#[test]
+fn a_version_building_trees_from_the_mkfs_watermark_without_a_data_unit_leaves_the_first_data_unit_invariant_not_applicable_with_its_own_reason(
+) {
+    let mut image =
+        build_pool("known-bad-first-tree-building-version-without-a-data-unit").memory_pool();
+    let zeroed_entries = std::cell::Cell::new(0usize);
+    mutate_unit(
+        &mut image,
+        TREE_TABLE,
+        |bytes| {
+            for index in 0..TREE_TABLE_ENTRIES_AFTER_THE_FIRST_FILE {
+                let entry = tree_table_entry_offset(index);
+                let kind = u16::from_le_bytes([
+                    bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET],
+                    bytes[entry + TREE_TABLE_ENTRY_KIND_OFFSET + 1],
+                ]);
+                if kind == TREE_KIND_EXTENT {
+                    bytes[entry + TREE_TABLE_ENTRY_ROOT_POINTER_RANGE.start
+                        ..entry + TREE_TABLE_ENTRY_ROOT_POINTER_RANGE.end]
+                        .fill(0);
+                    zeroed_entries.set(zeroed_entries.get() + 1);
+                }
+            }
+        },
+        true,
+    );
+    assert_eq!(zeroed_entries.get(), 1, "树表里 extent 树那一条恰一条");
+    assert_eq!(
+        verdict(
+            &image,
+            FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK
+        ),
+        InvariantVerdict::NotApplicable(
+            singlefs_checker::walk::VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT
+        ),
+        "认出了从 mkfs 水位第一次建树的那一版而它没有数据单元：I-5.5 报这一句专门理由"
+    );
+}
+
+/// 一条树表条目里树的根指针那 86 字节：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 之后（D8（核心索引结构） 已定项 8 的字段表）。
+const TREE_TABLE_ENTRY_ROOT_POINTER_RANGE: std::ops::Range<usize> = 14..100;
+
+/// 被崩溃恢复抛弃的根树表指针整条全零（判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P2-e：攻方读码报
+/// 「说明写的是单元已被重新分配或抹头」）：C (2, 8) 被实例 3 的实例表判出局之后，把 C 的根槽里树表指针写零、重封。
+/// 被抛弃根那一遍照 core 的根记录读者跟过去、读不出，I-7.4 照旧红；说明写明是这条根自己的树表指针全零，不再说成它引用的单元被重新分配或抹头。
+#[test]
+fn an_abandoned_root_whose_tree_table_pointer_is_all_zero_reddens_the_reuse_invariant_naming_the_root_itself(
+) {
+    let (mut image, _, _, abandoned_txg) = image_after_a_recovery_abandoned_the_third_root(
+        "known-bad-abandoned-root-all-zero-tree-table-pointer",
+    );
+    rewrite_the_root_slot_of(&mut image, CheckpointTxg(abandoned_txg), |bytes| {
+        bytes[ROOT_RECORD_TREE_TABLE_POINTER_RANGE].fill(0);
+    });
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-7.4"],
+        "被抛弃的根树表指针全零：只红 I-7.4（被抛弃根那一遍的判定丢掉）：{verdicts:?}"
+    );
+    let InvariantVerdict::Violated(detail) = verdict(&image, "I-7.4") else {
+        panic!("上面判过 I-7.4 红");
+    };
+    assert!(
+        detail.starts_with(&format!(
+            "被抛弃的根（实例 2、txg {abandoned_txg}）走不完：根记录里树表指针全零，core 的根记录读者照它读盘 0 槽 0、读不出"
+        )),
+        "I-7.4 说明写明是这条根自己的树表指针全零：{detail}"
+    );
+}
````

### mutations-append.tsv

````diff
第35批 35-1 FL：下一次挂载先施加的那一版照旧取先到的那条记录（s3：三条记录的发布第一条写成上一版的新根段，两条分配记录罩住同一个槽照旧 0 违例）	crates/singlefs-checker/src/walk.rs	            record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0\n                && record.commit_marker == CommitMarker::Present\n                && record.instance == newest_root.instance	            record.commit_marker == CommitMarker::Present\n                && record.instance == newest_root.instance	-p singlefs-harness --test checker_known_bad_images -- a_stale_first_record_of_the_publish_the_next_mount_applies_first_does_not_hide_two_allocation_records_covering_one_slot	a_stale_first_record_of_the_publish_the_next_mount_applies_first_does_not_hide_two_allocation_records_covering_one_slot
第35批 35-1 FL：由记录施加出来的那一版照旧取先到的那条记录（s4：第一条写成上一版的新根段，挂载之后 I-3.1、I-3.5 照旧红）	crates/singlefs-checker/src/walk.rs	                ) => carries_the_last_record_flag,	                ) => false,	-p singlefs-harness --test checker_known_bad_images -- a_version_applied_only_by_its_records_whose_first_record_carries_the_previous_root_is_walked_from_its_last_record	a_version_applied_only_by_its_records_whose_first_record_carries_the_previous_root_is_walked_from_its_last_record
第35批 35-1 Q1 甲：由记录施加出来的那一版树表指针全零照旧读成没有树表（q1b：(1, 5) 那条记录树表指针全零只红 I-3.1、I-3.5）	crates/singlefs-checker/src/walk.rs	        self.walk_tree_table_and_central_mapping_root(\n            tree_table_pointer,\n            mapping_root_pointer,	        if parse_node_pointer(tree_table_pointer).all_zero {\n            return;\n        }\n        self.walk_tree_table_and_central_mapping_root(\n            tree_table_pointer,\n            mapping_root_pointer,	-p singlefs-harness --test checker_known_bad_images -- an_all_zero_tree_table_pointer_in_the_record_of_a_version_applied_only_by_its_journal_record_is_followed_like_the_core	an_all_zero_tree_table_pointer_in_the_record_of_a_version_applied_only_by_its_journal_record_is_followed_like_the_core
第35批 35-1 Q1 甲：重放施加出来的那一版树表指针全零照旧读成没有树表（z6：下一次挂载先施加的那一版记录里树表指针全零，I-4.1 照旧不红）	crates/singlefs-checker/src/walk.rs	        self.walk_tree_table_and_central_mapping_root(\n            tree_table_pointer,\n            mapping_root_pointer,	        if parse_node_pointer(tree_table_pointer).all_zero {\n            return;\n        }\n        self.walk_tree_table_and_central_mapping_root(\n            tree_table_pointer,\n            mapping_root_pointer,	-p singlefs-harness --test checker_known_bad_images -- an_all_zero_tree_table_pointer_in_the_record_of_the_version_the_next_mount_applies_first_reddens_the_replayed_version_invariant	an_all_zero_tree_table_pointer_in_the_record_of_the_version_the_next_mount_applies_first_reddens_the_replayed_version_invariant
第35批 35-1 Q4 甲：I-5.5 认出了那一版而它没有数据单元时不写专门理由（落到缺省理由「这个镜像上没有它判的对象」）	crates/singlefs-checker/src/walk.rs	    } else {\n        judgements.not_applicable(\n            invariant,\n            VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT,\n        );\n    }	    }	-p singlefs-harness --test checker_known_bad_images -- a_version_building_trees_from_the_mkfs_watermark_without_a_data_unit_leaves_the_first_data_unit_invariant_not_applicable_with_its_own_reason	a_version_building_trees_from_the_mkfs_watermark_without_a_data_unit_leaves_the_first_data_unit_invariant_not_applicable_with_its_own_reason
第35批 35-1 被抛弃的根树表指针全零：I-7.4 说明照旧写「引用的单元已被重新分配或抹头」	crates/singlefs-checker/src/walk.rs	            if tree_table_pointer_is_all_zero {	            if tree_table_pointer_is_all_zero && instance == u32::MAX {	-p singlefs-harness --test checker_known_bad_images -- an_abandoned_root_whose_tree_table_pointer_is_all_zero_reddens_the_reuse_invariant_naming_the_root_itself	an_abandoned_root_whose_tree_table_pointer_is_all_zero_reddens_the_reuse_invariant_naming_the_root_itself
第35批 35-1 I-9.28 部分没判的前缀照旧写「树表指针非零而条目没读全」（单子里有树表指针全零的根，y1 那一句自相矛盾）	crates/singlefs-checker/src/walk.rs	根树表条目没读全（树表指针全零、树表读不出或有条目比字段表短），没进比较	根树表指针非零而条目没读全，没进比较	-p singlefs-harness --test checker_known_bad_images -- an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged	an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged
第35批 35-1 GPU 判器 FL：下一次挂载先施加的那一版照旧取记录数组里第一条合条件的（三条记录的发布第一条写成上一版的根，根槽没落的状态上 CPU 红 I-5.4、GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if ((r.flags & 1u) != 0u && r.commit_byte == 1u && r.instance == newest_root.instance	        if (r.commit_byte == 1u && r.instance == newest_root.instance	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_the_first_record_of_the_next_mount_version_carries_the_previous_root_over_two_allocation_records_covering_one_slot	the_gpu_judge_agrees_with_the_cpu_when_the_first_record_of_the_next_mount_version_carries_the_previous_root_over_two_allocation_records_covering_one_slot
第35批 35-1 GPU 判器 FL：拼 R_DATA 同键照旧取先到的那条记录（由记录施加出来的那一版第一条写成上一版的根，GPU 走上一版的树、I-3.1、I-3.5 红，CPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                            if ((existing.flags & 1u) == 0u && (r.flags & 1u) != 0u) {\n                                sws(R_DATA + position, i);\n                            }\n		-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_that_a_version_applied_only_by_its_records_is_walked_from_its_last_record_when_the_first_carries_the_previous_root	the_gpu_judge_agrees_with_the_cpu_that_a_version_applied_only_by_its_records_is_walked_from_its_last_record_when_the_first_carries_the_previous_root
第35批 35-1 GPU 判器 Q3 甲：树表指针全零照旧当没有树表（根与它的记录树表指针全零那条流上 CPU 红 I-2.1、I-7.2、I-4.1，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    if (tree_table_pointer.all_zero) {\n        _ = w_read_referenced_unit(tree_table_pointer, NODE_BYTES);\n        w_failures = w_failures + 1u;\n        return;\n    }\n	    if (tree_table_pointer.all_zero) {\n        return;\n    }\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_pointer_of_the_root_and_of_its_record_is_all_zero	the_gpu_judge_agrees_with_the_cpu_when_the_tree_table_pointer_of_the_root_and_of_its_record_is_all_zero
````

### mutations-replacements.tsv

````diff
实审 B2 第 9 条（I-5.4 / C512）：根记录持有的那棵分配记录树不判互不相交	crates/singlefs-checker/src/walk.rs	        let mut allocation_roots = vec![parse_node_pointer(\n            version.allocation_record_tree_root_held_outside_the_tree_table,\n        )];	        let mut allocation_roots = vec![parse_node_pointer(&[0u8; 86])];	-p singlefs-harness --test checker_narrow_invariants_and_abandoned_roots -- overlapping_records_in_the_allocation_record_tree_the_root_record_holds_redden_only_the_disjointness_invariant	overlapping_records_in_the_allocation_record_tree_the_root_record_holds_redden_only_the_disjointness_invariant
实审 B2b（formatted_pool 写行之后那一格 I-5.4 改判成立）：根记录持有的那棵分配记录树不判互不相交	crates/singlefs-checker/src/walk.rs	        let mut allocation_roots = vec![parse_node_pointer(\n            version.allocation_record_tree_root_held_outside_the_tree_table,\n        )];	        let mut allocation_roots = vec![parse_node_pointer(&[0u8; 86])];	-p singlefs-harness --test writable_mount_of_a_formatted_pool -- a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
第34批 34-1 全零树表指针：主走读照旧把根记录里全零的树表指针读成没有树表（y4：最新那条根树表指针全零照旧 0 违例）	crates/singlefs-checker/src/walk.rs	        self.walk_tree_table_and_central_mapping_root(\n            &record[36..122],	        if parse_node_pointer(&record[36..122]).all_zero {\n            return;\n        }\n        self.walk_tree_table_and_central_mapping_root(\n            &record[36..122],	-p singlefs-harness --test checker_known_bad_images -- an_all_zero_tree_table_pointer_in_the_newest_root_is_followed_like_the_core_and_the_newest_root_does_not_walk_through	an_all_zero_tree_table_pointer_in_the_newest_root_is_followed_like_the_core_and_the_newest_root_does_not_walk_through
````

## m3-formal-35-2

### crates.patch

````diff
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -922,12 +922,20 @@
     r.allocation_root = parse_node_pointer(base, 342u);
     return r;
 }
-// root_record.rs 的 parse_slot 第五关：树表、实例表、中央映射树、分配记录树四条指针头部的 extent 偏移任一非 0，这条根记录损坏、
+// 根记录里一条指针的头部是第一版的取值：pointer.rs 的 PointerHeadAsRead::judged 判的五段全过——MAC 16 与 nonce 12 全 0、算法类型 0、
+// 压缩算法码 0、压后长度 0（Ptr 的 head_ok），extent 偏移 0（Ptr 的 extent_offset_zero）
+fn root_record_pointer_head_is_the_first_version(pointer: Ptr) -> bool {
+    return pointer.head_ok && pointer.extent_offset_zero;
+}
+// root_record.rs 的 parse_slot：树表、实例表、中央映射树、分配记录树四条指针都经
+// NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from 读，任一条头部有一段不是第一版的取值，这条根记录损坏、
 // 与自证不过同一个处置，择根时不可择。只在恢复择根时判：池级 checker 的根环读法（image::valid_roots，走读内核 checker_collect_roots）不判它，
 // 那里红 I-2.4 的是走读跟随这几条指针的那几处
-fn root_record_pointers_carry_no_extent_offset(r: RootFields) -> bool {
-    return r.tree_table.extent_offset_zero && r.instance_table.extent_offset_zero
-        && r.mapping_root.extent_offset_zero && r.allocation_root.extent_offset_zero;
+fn root_record_pointer_heads_are_all_the_first_version(r: RootFields) -> bool {
+    return root_record_pointer_head_is_the_first_version(r.tree_table)
+        && root_record_pointer_head_is_the_first_version(r.instance_table)
+        && root_record_pointer_head_is_the_first_version(r.mapping_root)
+        && root_record_pointer_head_is_the_first_version(r.allocation_root);
 }
 fn region_device(cfg: SysConfig, region: u32) -> u32 {
     if (region == 0u) {
@@ -942,7 +950,7 @@
     found: bool,
     root: RootFields,
 };
-// 三个区域全部槽逐个验自证（连同四条指针的 extent 偏移），取 (txg, 实例) 最大的；每条自证过的根记进草稿（版本、区域）
+// 三个区域全部槽逐个验自证（连同四条指针头部的五段），取 (txg, 实例) 最大的；每条自证过的根记进草稿（版本、区域）
 fn choose_root(cfg: SysConfig) -> RootChoice {
     var out: RootChoice;
     out.found = false;
@@ -954,7 +962,7 @@
             let offset = u64_of(ROOT_RING_BASE_BYTES + region * ROOT_RING_REGION_STRIDE + slot * cfg.spacing);
             let s = read_at(device, offset, cfg.physical_block_size);
             let candidate = parse_root(s, cfg.fsid);
-            if (!candidate.valid || !root_record_pointers_carry_no_extent_offset(candidate)) {
+            if (!candidate.valid || !root_record_pointer_heads_are_all_the_first_version(candidate)) {
                 continue;
             }
             if (root_count < MAX_ROOTS) {
--- /dev/null
+++ b/crates/singlefs-checker-tier/tests/gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs
@@ -0,0 +1,417 @@
+//! checker 档模块：crash、crash_amplification、gpu_unit_checks、crash_judge_tables、crash_judge_gpu、crash_judge_dispatch
+//! GPU 恢复内核择根与 core 同判根记录四条指针的头部（第 34 批代码轮第一轮判决 `research/prompts/m3-formal-code-b34-r1-main-verification.md`
+//! 第二节 P3、第三节第 1 条）：core 读根槽经 `RootRecord::parse_slot`（`crates/singlefs-core/src/root_record.rs`），树表、实例表、中央映射树、
+//! 分配记录树四条指针都经 `NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from` 读，头部五段（`pointer.rs` 的
+//! `PointerHeadAsRead::judged`：MAC 与 nonce 全 0、算法类型 0、压缩算法码 0、压后长度 0、extent 偏移 0）任一段不是第一版的取值，这条根不可择；
+//! GPU 恢复内核择根（`crash_judge_common.wgsl` 的 `root_record_pointer_heads_are_all_the_first_version`）要同判。extent 偏移那一段另有用例
+//! （`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 根记录实例表指针那一条）；这里四条用例各把新池新建文件那条流里发布的根的一条指针
+//! 头部的另一段写成非第一版的取值、重封整槽校验和：树表指针 MAC 第 0 字节、中央映射树根指针算法类型、分配记录树根指针压缩算法码、
+//! 实例表指针压后长度（压缩算法码仍 0）。前提在取卡之前用 CPU 判：每个写都落了的镜像上，改之前 core 择到发布的根，改之后退到它之前那条根。
+//! GPU 与 CPU 在每个状态上红位、池级 checker 违例位、两遍恢复落到的根与结局逐位相同，一个状态都不许判不了。
+//! 要本机一张空闲显存够的 NVIDIA 独显：一张独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
+//! （`gpu_unit_checks::gpu_test_start`）。要 `verdict-store,gpu` 特性。
+#![cfg(all(feature = "verdict-store", feature = "gpu"))]
+
+#[path = "../../singlefs-harness/tests/common/mod.rs"]
+mod common;
+
+use common::{build_pool, file_content, geometry};
+use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
+use singlefs_checker_tier::crash::{
+    judge_layer0_state_range, layer0_plan_state_count, Layer0OracleViolationKind,
+    Layer0SegmentExpansion, Layer0StateJudgement,
+};
+use singlefs_checker_tier::crash_amplification::{gpu_recovery_words, recovery_words_of};
+use singlefs_checker_tier::crash_judge_dispatch::GpuCrashJudge;
+use singlefs_checker_tier::crash_judge_gpu::{
+    pool_checker_violated_bits, verdict_bits, POOL_CHECKER_UNDECIDABLE_WORD, VERDICT_WORDS,
+};
+use singlefs_checker_tier::crash_judge_tables::{build_judge_tables, JudgeFlow};
+use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::first_usable_card_or_report_not_run;
+use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::recovery::{choose_root, choose_system_configuration};
+use singlefs_harness::memory_pool::{
+    writes_and_segments_with_stream_indexes, MemoryPool, PublishedVersion, RetainedWrite,
+    WrittenContents,
+};
+use singlefs_harness::segments::StepKind;
+
+/// 段长不超过这个数的段逐个真子集展开（与 GPU 对拍那一份的默认展开上限同一档）。
+const EXPAND_UP_TO: usize = 8;
+/// 根记录里四条码 2 指针的起点（D22（单元原子性怎么合成） 已定项 7 的根记录布局；`crash_judge_common.wgsl` 的 `parse_root` 同一组偏移）。
+const ROOT_RECORD_TREE_TABLE_POINTER_OFFSET: usize = 36;
+const ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET: usize = 170;
+const ROOT_RECORD_MAPPING_ROOT_POINTER_OFFSET: usize = 256;
+const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_POINTER_OFFSET: usize = 342;
+/// 根记录里实例代号（4 字节）与 checkpoint_txg（8 字节）的偏移。
+const ROOT_RECORD_INSTANCE_OFFSET: usize = 24;
+const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;
+/// 根记录整槽校验和的偏移与宽（罩整个槽宽，校验和那一段按 0 算）。
+const ROOT_RECORD_CHECKSUM_OFFSET: usize = 138;
+const ROOT_RECORD_CHECKSUM_BYTES: usize = 32;
+/// 指针头部各段的偏移（D19（块指针的结构与宽度预算） 已定项 7 的字段表：MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2、extent 偏移 2）。
+const POINTER_FIRST_MAC_BYTE_OFFSET: usize = 0;
+const POINTER_ALGORITHM_TYPE_OFFSET: usize = 28;
+const POINTER_COMPRESSION_ALGORITHM_CODE_OFFSET: usize = 29;
+const POINTER_COMPRESSED_LENGTH_OFFSET: usize = 30;
+
+/// 根记录里的哪一条指针。
+#[derive(Clone, Copy, Debug)]
+enum RootRecordPointer {
+    TreeTable,
+    InstanceTable,
+    MappingRoot,
+    AllocationRecordTreeRoot,
+}
+
+impl RootRecordPointer {
+    fn offset_in_the_root_record(self) -> usize {
+        match self {
+            RootRecordPointer::TreeTable => ROOT_RECORD_TREE_TABLE_POINTER_OFFSET,
+            RootRecordPointer::InstanceTable => ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET,
+            RootRecordPointer::MappingRoot => ROOT_RECORD_MAPPING_ROOT_POINTER_OFFSET,
+            RootRecordPointer::AllocationRecordTreeRoot => {
+                ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_POINTER_OFFSET
+            }
+        }
+    }
+}
+
+/// 指针头部里写成非第一版取值的那一段（extent 偏移之外的四类，`pointer.rs` 的 `PointerHeadFieldOutsideTheFirstVersion` 前四个成员）。
+#[derive(Clone, Copy, Debug)]
+enum PointerHeadFieldWrittenOutsideTheFirstVersion {
+    /// MAC 第 0 字节写 1（加密关着时 MAC 与 nonce 恒 0，D19 已定项 3）。
+    FirstMacByteToOne,
+    /// 算法类型写 1（第一版不加密，D9（加密） 已定项 10）。
+    AlgorithmTypeToOne,
+    /// 压缩算法码写 1（登记表第一版只有码 0，D9（加密） 已定项 14）。
+    CompressionAlgorithmCodeToOne,
+    /// 压缩算法码仍 0、压后长度写 1（D9（加密） 已定项 14「码 0 时它恒 0」）。
+    CompressedLengthToOneWithoutCompression,
+}
+
+impl PointerHeadFieldWrittenOutsideTheFirstVersion {
+    /// 这一段在指针头部里的偏移与写进去的字节。
+    fn offset_in_the_pointer_and_bytes(self) -> (usize, Vec<u8>) {
+        match self {
+            PointerHeadFieldWrittenOutsideTheFirstVersion::FirstMacByteToOne => {
+                (POINTER_FIRST_MAC_BYTE_OFFSET, vec![1])
+            }
+            PointerHeadFieldWrittenOutsideTheFirstVersion::AlgorithmTypeToOne => {
+                (POINTER_ALGORITHM_TYPE_OFFSET, vec![1])
+            }
+            PointerHeadFieldWrittenOutsideTheFirstVersion::CompressionAlgorithmCodeToOne => {
+                (POINTER_COMPRESSION_ALGORITHM_CODE_OFFSET, vec![1])
+            }
+            PointerHeadFieldWrittenOutsideTheFirstVersion::CompressedLengthToOneWithoutCompression => {
+                (
+                    POINTER_COMPRESSED_LENGTH_OFFSET,
+                    1u16.to_le_bytes().to_vec(),
+                )
+            }
+        }
+    }
+}
+
+/// 一条流：mkfs 之后的基镜像、写表与段、判的那条根、模型认的版本。
+struct RecordedFlow {
+    base: MemoryPool,
+    writes: Vec<RetainedWrite>,
+    segments: Vec<Vec<usize>>,
+    judged_root_index: usize,
+    versions: Vec<PublishedVersion>,
+}
+
+/// 重封根记录的整槽校验和。
+fn reseal_root_record(bytes: &mut [u8]) {
+    let digest = singlefs_core::checksum::wide_checksum_with_field_zeroed(
+        bytes,
+        bytes.len(),
+        ROOT_RECORD_CHECKSUM_OFFSET,
+    );
+    bytes[ROOT_RECORD_CHECKSUM_OFFSET..ROOT_RECORD_CHECKSUM_OFFSET + ROOT_RECORD_CHECKSUM_BYTES]
+        .copy_from_slice(&digest);
+}
+
+/// 每个写都落了的镜像上 core 择到的根的 checkpoint_txg（core 只读恢复与挂载择的是它）。
+fn checkpoint_txg_of_the_root_the_core_chooses_with_every_write_landed(
+    base: &MemoryPool,
+    writes: &[RetainedWrite],
+) -> u64 {
+    let mut image = base.clone();
+    image.apply_writes(writes);
+    let system_configuration = choose_system_configuration(&image).expect("系统配置择得到");
+    choose_root(&image, &system_configuration)
+        .expect("择得到根")
+        .checkpoint_txg
+        .0
+}
+
+/// 新池新建文件那条流，发布的根里 `pointer` 那条指针头部的 `field` 那一段（本来是第一版的取值）写成非第一版的取值、重封。
+/// 改之前每个写都落了的镜像上 core 择到发布的根、改之后退到更早的根，在这里当场断言（取卡之前、CPU 上判）。
+fn new_pool_with_a_root_record_pointer_head_field_outside_the_first_version(
+    tag: &str,
+    pointer: RootRecordPointer,
+    field: PointerHeadFieldWrittenOutsideTheFirstVersion,
+) -> RecordedFlow {
+    let pool = build_pool(tag);
+    let operations = pool.retained_operations();
+    let (mut writes, segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
+        &operations[pool.mkfs_operation_count..],
+        &geometry(),
+    );
+    let judged_root_index = writes
+        .iter()
+        .rposition(|write| write.kind == StepKind::RootRecordFua)
+        .expect("发布有一条根槽写");
+    let base = pool.memory_pool_after_mkfs();
+    let WrittenContents::Bytes(publish_root) = &writes[judged_root_index].contents else {
+        panic!("根槽写是普通写");
+    };
+    assert_eq!(
+        u32::from_le_bytes(
+            publish_root[ROOT_RECORD_INSTANCE_OFFSET..ROOT_RECORD_INSTANCE_OFFSET + 4]
+                .try_into()
+                .expect("4 字节")
+        ),
+        1,
+        "发布的根是实例 1 的"
+    );
+    let publish_checkpoint_txg = u64::from_le_bytes(
+        publish_root[ROOT_RECORD_CHECKPOINT_TXG_OFFSET..ROOT_RECORD_CHECKPOINT_TXG_OFFSET + 8]
+            .try_into()
+            .expect("8 字节"),
+    );
+    let (field_offset, field_bytes) = field.offset_in_the_pointer_and_bytes();
+    let field_start = pointer.offset_in_the_root_record() + field_offset;
+    let field_range = field_start..field_start + field_bytes.len();
+    assert!(
+        publish_root[field_range.clone()]
+            .iter()
+            .all(|byte| *byte == 0),
+        "第一版写者把根记录 {pointer:?} 指针头部 {field:?} 那一段写成 0"
+    );
+    let mut rewritten_root = publish_root.clone();
+    rewritten_root[field_range].copy_from_slice(&field_bytes);
+    reseal_root_record(&mut rewritten_root);
+    let writes_before_the_rewrite = writes.clone();
+    writes[judged_root_index].contents = WrittenContents::Bytes(rewritten_root);
+    assert_eq!(
+        checkpoint_txg_of_the_root_the_core_chooses_with_every_write_landed(
+            &base,
+            &writes_before_the_rewrite
+        ),
+        publish_checkpoint_txg,
+        "改之前每个写都落了的镜像上 core 择到发布的根"
+    );
+    let chosen_checkpoint_txg_after_the_rewrite =
+        checkpoint_txg_of_the_root_the_core_chooses_with_every_write_landed(&base, &writes);
+    assert!(
+        chosen_checkpoint_txg_after_the_rewrite < publish_checkpoint_txg,
+        "根记录 {pointer:?} 指针头部 {field:?} 那一段不是第一版的取值，core 择根要退到发布之前那条根（择到 txg {chosen_checkpoint_txg_after_the_rewrite}，发布的根 txg {publish_checkpoint_txg}）"
+    );
+    RecordedFlow {
+        base,
+        writes,
+        segments,
+        judged_root_index,
+        versions: vec![PublishedVersion {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(publish_checkpoint_txg),
+            content: file_content(),
+        }],
+    }
+}
+
+fn oracle_index(kind: Layer0OracleViolationKind) -> u32 {
+    u32::try_from(
+        Layer0OracleViolationKind::EVERY_KIND
+            .iter()
+            .position(|candidate| *candidate == kind)
+            .expect("七类之一"),
+    )
+    .expect("装得进 u32")
+}
+
+/// CPU 判定 → GPU 结论第 0 字那一套红位（两遍 oracle 各一类、记录核对器两条）。
+fn red_bits_on_the_cpu(judgement: &Layer0StateJudgement) -> u32 {
+    let mut bits = 0u32;
+    if let Some(violation) = &judgement.consulted_oracle {
+        bits |= 1 << (verdict_bits::CONSULTED_ORACLE_FIRST + oracle_index(violation.kind));
+    }
+    if let Some(violation) = &judgement.ignored_oracle {
+        bits |= 1 << (verdict_bits::IGNORED_ORACLE_FIRST + oracle_index(violation.kind));
+    }
+    if judgement.records.root_without_record {
+        bits |= 1 << verdict_bits::ROOT_WITHOUT_RECORD;
+    }
+    if judgement.records.claimed_state_missing_unit {
+        bits |= 1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT;
+    }
+    bits
+}
+
+/// CPU 池级 checker 的违例位：位 i ↔ `IMPLEMENTED_INVARIANTS[i]`（与 GPU 判器结论里那一段同一个编号）。
+fn pool_checker_bits_on_the_cpu(judgement: &Layer0StateJudgement) -> u128 {
+    judgement
+        .pool_checker
+        .iter()
+        .filter_map(|(invariant, verdict)| match verdict {
+            InvariantVerdict::Violated(_) => Some(
+                IMPLEMENTED_INVARIANTS
+                    .iter()
+                    .position(|candidate| candidate == invariant)
+                    .expect("池级 checker 报的不变量在清单里"),
+            ),
+            InvariantVerdict::Holds
+            | InvariantVerdict::NotApplicable(_)
+            | InvariantVerdict::PartiallyJudged(_) => None,
+        })
+        .fold(0u128, |bits, index| bits | (1u128 << index))
+}
+
+fn invariant_names(bits: u128) -> Vec<&'static str> {
+    IMPLEMENTED_INVARIANTS
+        .iter()
+        .enumerate()
+        .filter(|(index, _)| bits & (1u128 << index) != 0)
+        .map(|(_, name)| *name)
+        .collect()
+}
+
+/// 一个状态上 CPU 判出的三样：红位、池级 checker 违例位、两遍恢复落到的根与结局（`recovery_words_of` 的口径）。
+struct JudgedOnTheCpu {
+    red_bits: u32,
+    pool_checker_bits: u128,
+    recovery_words: [u32; 4],
+}
+
+/// GPU 判器与 CPU 判器在这条流的每个状态上红位、池级 checker 违例位、两遍恢复落到的根与结局逐位相同，没有判不了的状态。
+/// 本机没有 NVIDIA 独显适配器时报「本次未跑」返回。
+fn assert_the_gpu_judge_agrees_with_the_cpu_on_every_state(test_name: &str, flow: &RecordedFlow) {
+    let Some(card) = first_usable_card_or_report_not_run(test_name) else {
+        return;
+    };
+    let expansion = |_segment_index: usize, segment: &[usize]| {
+        if segment.len() <= EXPAND_UP_TO {
+            Layer0SegmentExpansion::EveryProperSubset
+        } else {
+            Layer0SegmentExpansion::NotExpanded
+        }
+    };
+    let tables = build_judge_tables(&JudgeFlow {
+        base: &flow.base,
+        writes: &flow.writes,
+        segments: &flow.segments,
+        judged_root_index: flow.judged_root_index,
+        versions: &flow.versions,
+        expansion: &expansion,
+    })
+    .expect("建得出表");
+    let state_count = layer0_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
+    assert_eq!(tables.state_count, state_count);
+    let gpu: Vec<[u32; VERDICT_WORDS]> =
+        GpuCrashJudge::new(&card, &tables).judge_states(0..state_count);
+    let mut cpu: Vec<JudgedOnTheCpu> = Vec::new();
+    let judged_on_the_cpu = judge_layer0_state_range(
+        &flow.base,
+        &flow.writes,
+        &flow.segments,
+        flow.judged_root_index,
+        &flow.versions,
+        &expansion,
+        0..state_count,
+        &mut |_ordinal, _image, judgement| {
+            cpu.push(JudgedOnTheCpu {
+                red_bits: red_bits_on_the_cpu(judgement),
+                pool_checker_bits: pool_checker_bits_on_the_cpu(judgement),
+                recovery_words: recovery_words_of(judgement),
+            });
+        },
+    );
+    assert_eq!(judged_on_the_cpu, state_count);
+    assert_eq!(cpu.len(), gpu.len(), "两边判的状态数相同");
+    let mismatches: Vec<String> = cpu
+        .iter()
+        .zip(&gpu)
+        .enumerate()
+        .filter_map(|(ordinal, (cpu_judgement, gpu_words))| {
+            let undecidable = gpu_words[0] & (1 << verdict_bits::UNDECIDABLE) != 0
+                || gpu_words[POOL_CHECKER_UNDECIDABLE_WORD] != 0;
+            let gpu_red_bits = gpu_words[0] & !(1 << verdict_bits::UNDECIDABLE);
+            let gpu_pool_checker_bits = pool_checker_violated_bits(gpu_words);
+            let gpu_recovery = gpu_recovery_words(gpu_words);
+            (undecidable
+                || gpu_red_bits != cpu_judgement.red_bits
+                || gpu_pool_checker_bits != cpu_judgement.pool_checker_bits
+                || gpu_recovery != cpu_judgement.recovery_words)
+                .then(|| {
+                    format!(
+                        "状态 {ordinal}：CPU 红位 {:#x} 恢复 {:?} checker {:?}；GPU 红位 {gpu_red_bits:#x} 恢复 {gpu_recovery:?} checker {:?}（判不了：{undecidable}）",
+                        cpu_judgement.red_bits,
+                        cpu_judgement.recovery_words,
+                        invariant_names(cpu_judgement.pool_checker_bits),
+                        invariant_names(gpu_pool_checker_bits)
+                    )
+                })
+        })
+        .collect();
+    assert!(
+        mismatches.is_empty(),
+        "{} 个状态 GPU 与 CPU 不一致（GPU 恢复内核择根与 core 判根记录指针头部不同时，落到的根与两遍的结局先对不上）：\n{}",
+        mismatches.len(),
+        mismatches.iter().take(8).cloned().collect::<Vec<_>>().join("\n")
+    );
+}
+
+#[test]
+fn the_gpu_recovery_refuses_a_root_whose_tree_table_pointer_carries_a_non_zero_mac_byte_like_the_core(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state(
+        "the_gpu_recovery_refuses_a_root_whose_tree_table_pointer_carries_a_non_zero_mac_byte_like_the_core",
+        &new_pool_with_a_root_record_pointer_head_field_outside_the_first_version(
+            "gpu-recovery-root-tree-table-pointer-first-mac-byte",
+            RootRecordPointer::TreeTable,
+            PointerHeadFieldWrittenOutsideTheFirstVersion::FirstMacByteToOne,
+        ),
+    );
+}
+
+#[test]
+fn the_gpu_recovery_refuses_a_root_whose_mapping_root_pointer_carries_an_algorithm_type_like_the_core(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state(
+        "the_gpu_recovery_refuses_a_root_whose_mapping_root_pointer_carries_an_algorithm_type_like_the_core",
+        &new_pool_with_a_root_record_pointer_head_field_outside_the_first_version(
+            "gpu-recovery-root-mapping-root-pointer-algorithm-type",
+            RootRecordPointer::MappingRoot,
+            PointerHeadFieldWrittenOutsideTheFirstVersion::AlgorithmTypeToOne,
+        ),
+    );
+}
+
+#[test]
+fn the_gpu_recovery_refuses_a_root_whose_allocation_record_tree_root_pointer_carries_a_compression_algorithm_code_like_the_core(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state(
+        "the_gpu_recovery_refuses_a_root_whose_allocation_record_tree_root_pointer_carries_a_compression_algorithm_code_like_the_core",
+        &new_pool_with_a_root_record_pointer_head_field_outside_the_first_version(
+            "gpu-recovery-root-allocation-record-tree-root-pointer-compression-code",
+            RootRecordPointer::AllocationRecordTreeRoot,
+            PointerHeadFieldWrittenOutsideTheFirstVersion::CompressionAlgorithmCodeToOne,
+        ),
+    );
+}
+
+#[test]
+fn the_gpu_recovery_refuses_a_root_whose_instance_table_pointer_carries_a_compressed_length_without_compression_like_the_core(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_every_state(
+        "the_gpu_recovery_refuses_a_root_whose_instance_table_pointer_carries_a_compressed_length_without_compression_like_the_core",
+        &new_pool_with_a_root_record_pointer_head_field_outside_the_first_version(
+            "gpu-recovery-root-instance-table-pointer-compressed-length",
+            RootRecordPointer::InstanceTable,
+            PointerHeadFieldWrittenOutsideTheFirstVersion::CompressedLengthToOneWithoutCompression,
+        ),
+    );
+}
````

### mutations-append.tsv

````diff
第 35 批 35-2 GPU 判器：恢复内核择根只判根记录指针的 extent 偏移（树表指针 MAC 非 0 的根照择）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    return pointer.head_ok && pointer.extent_offset_zero;	    return pointer.extent_offset_zero;	-p singlefs-checker-tier --features verdict-store,gpu --test gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core -- the_gpu_recovery_refuses_a_root_whose_tree_table_pointer_carries_a_non_zero_mac_byte_like_the_core	the_gpu_recovery_refuses_a_root_whose_tree_table_pointer_carries_a_non_zero_mac_byte_like_the_core
第 35 批 35-2 GPU 判器：恢复内核择根只判根记录指针的 extent 偏移（映射根指针算法类型非 0 的根照择）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    return pointer.head_ok && pointer.extent_offset_zero;	    return pointer.extent_offset_zero;	-p singlefs-checker-tier --features verdict-store,gpu --test gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core -- the_gpu_recovery_refuses_a_root_whose_mapping_root_pointer_carries_an_algorithm_type_like_the_core	the_gpu_recovery_refuses_a_root_whose_mapping_root_pointer_carries_an_algorithm_type_like_the_core
第 35 批 35-2 GPU 判器：恢复内核择根只判根记录指针的 extent 偏移（分配记录树根指针压缩算法码非 0 的根照择）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    return pointer.head_ok && pointer.extent_offset_zero;	    return pointer.extent_offset_zero;	-p singlefs-checker-tier --features verdict-store,gpu --test gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core -- the_gpu_recovery_refuses_a_root_whose_allocation_record_tree_root_pointer_carries_a_compression_algorithm_code_like_the_core	the_gpu_recovery_refuses_a_root_whose_allocation_record_tree_root_pointer_carries_a_compression_algorithm_code_like_the_core
第 35 批 35-2 GPU 判器：恢复内核择根只判根记录指针的 extent 偏移（实例表指针码 0 而压后长度非 0 的根照择）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    return pointer.head_ok && pointer.extent_offset_zero;	    return pointer.extent_offset_zero;	-p singlefs-checker-tier --features verdict-store,gpu --test gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core -- the_gpu_recovery_refuses_a_root_whose_instance_table_pointer_carries_a_compressed_length_without_compression_like_the_core	the_gpu_recovery_refuses_a_root_whose_instance_table_pointer_carries_a_compressed_length_without_compression_like_the_core
````

### mutations-replacements.tsv

````diff
第 29 批 29-2 GPU 判器：恢复内核择根不判根记录四条指针的 extent 偏移	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	            if (!candidate.valid || !root_record_pointer_heads_are_all_the_first_version(candidate)) {	            if (!candidate.valid) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_of_a_root_record_carries_an_extent_offset	the_gpu_judge_agrees_with_the_cpu_when_the_instance_table_pointer_of_a_root_record_carries_an_extent_offset
````

## m3-formal-35-3

### crates.patch

````diff
diff --git a/crates/singlefs-harness/src/model.rs b/crates/singlefs-harness/src/model.rs
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -4322,15 +4322,17 @@
 
     /// 模型回退那次发布对最左那片之外的叶容器的处置（`next_root_with_the_user_visible_trees` 取回退目标那一支里丢现行那几片、取目标那几片的两句），
     /// 结构单测（第 30 批判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M1「第 k 片都对」那一行的改法 B4）：
-    /// 现行与目标各取 1、2、3、4、5、100、135 片（135 是一个码 2 根装得下的上界，D8（核心索引结构） 已定项 6）的每一种组合，回退写出的那一版
+    /// 现行与目标各取 1、2、3、4、5、30、100、120、135 片（135 是一个码 2 根装得下的上界，D8（核心索引结构） 已定项 6）的每一种组合，回退写出的那一版
     /// 最左那片之外的叶容器（角色与分配代）恰好等于目标那一版的——现行有、目标没有的那几片这一版也没有，目标的每一片都取回、分配代取目标的。
-    /// 100 罩只按片数触发、不按位置触发的错：1–5 与 135 之间没有取样点时，只在目标装 60–118 片才错的回退照绿
-    /// （第 32–33 批代码轮判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P1「B4」那一行，`.claude/rules/mutation-sampling.md` 第三类）。
+    /// 30、100、120 罩只按片数触发、不按位置触发的错（最左之外各 29、99、119 片）：1–5 与 135 之间没有取样点时，只在目标装 60–118 片才错的回退照绿
+    /// （第 32–33 批代码轮判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P1「B4」那一行，`.claude/rules/mutation-sampling.md` 第三类）；
+    /// 只有 100 时，最左之外的片数落在 6–58、5–98、100–133 里才错的回退照绿（第 34 批代码轮判决
+    /// `research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P5「B4」那一行），30 落进前两个窗口、120 落进第三个。
     /// 按取样点逐片往外加历史用例追不完（回退只取回或只丢某几片，只有建到那几片之外的历史抓得到），这一条不跑历史。
     #[test]
     fn a_rollback_takes_exactly_the_inode_leaf_containers_right_of_the_leftmost_of_its_target_for_every_pair_of_container_counts(
     ) {
-        const CONTAINER_COUNTS: [u64; 7] = [1, 2, 3, 4, 5, 100, 135];
+        const CONTAINER_COUNTS: [u64; 9] = [1, 2, 3, 4, 5, 30, 100, 120, 135];
         let model = model_after_four_overwrites_in_a_second_instance();
         let newest = model.newest_root().clone();
         let mut pairs_judged = 0;
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
@@ -826,7 +826,7 @@
 /// `LAYER0_PARALLEL_START` + 46 行 `LAYER0_PROGRESS` + `LAYER0_FINDINGS` + `LAYER0_PARALLEL_FINISHED` + `GOLDEN_TALLY` 共 51 行。
 /// C577（系统配置没见证到的最新根，乙罩不到）那道发布返回之前的屏障加上之后甲二那一段 54 → 46 个状态（`crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs` 文件头），
 /// 行数与进度文件行数照这个数静态改（合入后验证二，改法 B 报告第四节第 5 条推的，没跑）；池级 checker 多一条 I-7.14 之后
-/// 打印的计数行与进度文件内容也跟着变。打的行与进度文件内容 2026-10-05 在「第 28 批 + 根环线 10e」那棵树上重取（见下）。
+/// 打印的计数行与进度文件内容也跟着变。打的行与进度文件内容此后又重取过几次，经过与今天这组钉值取自哪一版见下。
 ///
 /// 钉值的来历：头一版取自加分片之前的代码（`crash.rs` 与 `layer0_progress.rs` 加分片之前那一版：38 行、SHA-256 `6deeb48f…`，
 /// 进度文件计划哈希 `a1d502a7…`、内容 `798bef3c…`），证的是加分片没改不分片那条路。实审 B3a-2 按设备记屏障、原地覆写补第三态之后
diff --git a/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs b/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs
@@ -95,10 +95,9 @@
 }
 
 /// checker 与记录核对器那一半的钉死值。只在根槽 txg 3 已持久的状态上才走得到记账树、inode 树、分配记录树与中央映射树的条目，
-/// 那 26 条只在这些状态上评估（删除态扫描那四条 I-9.5、I-3.4、I-9.24、I-9.29 的理由写在函数里那张表旁边）
-/// （I-5.4（分配记录罩住的槽互不相交） 在其中，
-/// I-9.25（树表条目的 previous_snapshot_txg）、I-9.27（树表条目的头 ID）、I-3.13（记账行的代段）、I-9.26（inode 记录的改动计数）、
-/// I-5.5（从 mkfs 水位建树那一版的第一个数据单元） 与 I-3.5（引用区间的精确性） 同一个理由：种子根与暖机根指着的第 0 版树表是空的，
+/// 那 25 条只在这些状态上评估（删除态扫描那四条 I-9.5、I-3.4、I-9.24、I-9.29 的理由写在函数里那张表旁边）
+/// （I-9.25（树表条目的 previous_snapshot_txg）、I-9.27（树表条目的头 ID）、I-3.13（记账行的代段）、I-9.26（inode 记录的改动计数）、
+/// I-5.5（从 mkfs 水位建树那一版的第一个数据单元） 与 I-3.5（引用区间的精确性） 是这个理由：种子根与暖机根指着的第 0 版树表是空的，
 /// 只有 txg 3 那一版有树表条目、记账树、inode 记录与数据单元；记录施加出来的那一版由 I-4.1 另起一份走读去走、判定丢掉，不进这几条的计数，
 /// 映射 key 与单元头相符（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`，实审 B2 新立、编号待定）判的是中央映射树叶里的每一条条目：
 /// 种子根与暖机根的映射根指针全零（没有文件的一版），走不到条目；txg 3 的记录已落而根槽没落时，由记录施加出来的那一版
@@ -111,6 +110,8 @@
 /// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 单列一类：它除了回退候选集里那几版，还读下一次挂载会先施加的那一版
 /// （`.claude/kb/invariants.md` I-3.10 那一行射程 ④），txg 3 的记录至少一份已落、根槽没落的状态上也评估，
 /// 评估过的状态数由调用方按录制流逐状态现算（[`TransactionPublishInRecordedStream::allocation_generation_read_set_in`]），不写死；
+/// I-5.4（分配记录罩住的槽互不相交） 在这一类里：按版本那一层也判下一次挂载先施加的那一版（第 34 批改法 RV，`.claude/kb/invariants.md`
+/// I-5.4 那一行），两条认这一版用的是池级 checker 里同一个函数，这条流上评估过的状态逐个相同，理由写在函数里那张表旁边；
 /// I-9.14（树表条目的诞生 txg 跨根不变）在这条流上一个状态都评估不到，I-9.28（全部读得出的根之间同一树 ID 的诞生 txg 相同） 同一个理由；
 /// I-3.12（实例表行 T 低于引用它的根的 txg） 也评估不到：这条流上没有哪次发布重写实例表（取号只写系统配置，暖机与 A 照抄
 /// mkfs 那张只有链记录、没有行的实例表）；
@@ -142,7 +143,6 @@
         "I-3.9",
         "I-3.11",
         "I-5.2",
-        "I-5.4",
         "I-9.1",
         "I-9.2",
         "I-9.4",
@@ -171,7 +171,14 @@
     ];
     // I-3.10 除了回退候选集，还读下一次挂载会先施加的那一版（射程 ④）：新根没落而 txg 3 的记录已落时，
     // 恢复要由那条记录重建 txg 3 那一版，它的分配记录树在记录的新根段指着的树表下面。
-    let under_the_new_root_or_through_the_record_the_next_mount_applies_first = ["I-3.10"];
+    // I-5.4 按版本那一层同样判这一版（第 34 批改法 RV）。两条在这条流上逐状态评估同一批状态（推的，读码，没在 checker 档跑过）：
+    // ① 认这一版两条都经 `walk::version_the_next_mount_applies_first`（I-3.10 经 `tree_table_pointer_of_the_version_the_next_mount_applies_first`
+    // 取它的树表指针），候选根两条都取 `candidate_indexes`，由记录施加出来的那几版两条都取、这条流上一版都没有；
+    // ② 这条流上只有 txg 3 那一版有分配记录树，A 的 24 个单元写自成一段、排在记录段与根槽段之前（`prepare` 里钉的段长），
+    // txg 3 的记录或根落了的状态上它的单元全落了：树表、分配记录树读得出、走得到叶（I-5.4 判成立），记录罩住的单元头都可用（I-3.10 判得到记录）；
+    // ③ 种子根与暖机根指着的第 0 版树表在 mkfs 的基线里、读得出、0 条，根记录里树表 0 条那一版的分配记录树根全零：
+    // I-5.4 那几格「树表读不出」「树表指针全零」「根记录直接持有的树读不出」都不出现，不报部分没判；txg 3 那一版不在读集里的状态上两条都报不适用。
+    let under_the_new_root_or_through_the_record_the_next_mount_applies_first = ["I-3.10", "I-5.4"];
     // I-9.14 要同一棵树的条目出现在两个树表单元里才比得出来，而这条流上只有新池新建文件写树表：mkfs 种的第 0 版树表是空的、
     // 两次暖机空发布不写树表 ⇒ 每个状态都报「不适用」，一个状态都评估不到。跨根比得出来的流在
     // `crash_enumeration_fixed_script_stream.rs`（发布 B 起每次发布都重写树表），那里它真被评估过。
````

### mutations-append.tsv

````diff
第 35 批 35-3 B4（理想模型自身）：回退只在目标那一版最左之外有 6–58 片叶容器时只取回前 3 片（第 34 批代码轮判决 P5「B4」存活窗口，补取样点 30）	crates/singlefs-harness/src/model.rs	                        .filter(|(role, _)| {\n                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                        })	                        .filter(|(role, _)| {\n                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                        })\n                        .take(if (6..=58).contains(&target_root.role_written_at.keys().filter(|role| matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))).count()) { 3 } else { usize::MAX })	-p singlefs-harness --lib -- model::tests::a_rollback_takes_exactly_the_inode_leaf_containers	a_rollback_takes_exactly_the_inode_leaf_containers_right_of_the_leftmost_of_its_target_for_every_pair_of_container_counts
第 35 批 35-3 B4（理想模型自身）：回退只在目标那一版最左之外有 5–98 片叶容器时只取回前 3 片（第 34 批代码轮判决 P5「B4」存活窗口，补取样点 30）	crates/singlefs-harness/src/model.rs	                        .filter(|(role, _)| {\n                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                        })	                        .filter(|(role, _)| {\n                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                        })\n                        .take(if (5..=98).contains(&target_root.role_written_at.keys().filter(|role| matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))).count()) { 3 } else { usize::MAX })	-p singlefs-harness --lib -- model::tests::a_rollback_takes_exactly_the_inode_leaf_containers	a_rollback_takes_exactly_the_inode_leaf_containers_right_of_the_leftmost_of_its_target_for_every_pair_of_container_counts
第 35 批 35-3 B4（理想模型自身）：回退只在目标那一版最左之外有 100–133 片叶容器时只取回前 3 片（第 34 批代码轮判决 P5「B4」存活窗口，补取样点 120）	crates/singlefs-harness/src/model.rs	                        .filter(|(role, _)| {\n                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                        })	                        .filter(|(role, _)| {\n                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                        })\n                        .take(if (100..=133).contains(&target_root.role_written_at.keys().filter(|role| matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))).count()) { 3 } else { usize::MAX })	-p singlefs-harness --lib -- model::tests::a_rollback_takes_exactly_the_inode_leaf_containers	a_rollback_takes_exactly_the_inode_leaf_containers_right_of_the_leftmost_of_its_target_for_every_pair_of_container_counts
````
