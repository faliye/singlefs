# 第 34 批代码轮第一轮：改动全文（2026-10-06）

四份补丁照实现员交回的原样，34-1、34-2、34-3 同一趟合入，34-4 在根环线第 10h 批合入之后打；另加主 agent 写的 #9 三个钉值（`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs`，第 30 批 T_time 改毫秒与删除线第 22 批带来的重取）。

## m3-formal-34-1

34-1：池级 checker——I-5.4 罩由记录施加出来的版本与下一次挂载先施加的那一版（RV，GPU 扫描内核同改）；自证过的根树表指针全零照 core 跟过去判损坏（用户 2026-10-05 定甲）；I-5.5 专门不适用理由

### crates.patch

````diff
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 3e1dbff..4f5cc31 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -660,6 +660,17 @@ fn judge_the_appearance_rule_of_adjacent_candidate_versions(
     }
 }
 
+/// 走读遇到整条全零的树表指针时怎么读（用户 2026-10-05 定甲，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第五节第 1 条）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum AllZeroTreeTablePointerReading {
+    /// 根环里自证过的根：照 core 的根记录读者跟过去、读不出，判这条根损坏
+    /// （[`Walk::follow_an_all_zero_tree_table_pointer_of_a_root_record`]）。
+    FollowedLikeTheRootRecordReaderOfTheCore,
+    /// 由记录施加出来的那一版（树表指针在记录新根段里）：用户定的只是自证过的根，这一类照旧当这一版没有树表、走读不记失败
+    /// （要不要也判损坏交主 agent）。
+    NoTreeTable,
+}
+
 /// 走读此刻在走哪一版（D16（发布语义） 已定项 1 的回退候选集里的根，或由记录施加出来、根槽从没落盘的那一版），带它的 `checkpoint_txg`。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum VersionBeingWalked {
@@ -1398,6 +1409,7 @@ impl<'reader> Walk<'reader> {
             &record[36..122],
             &record[256..342],
             &version_name,
+            AllZeroTreeTablePointerReading::FollowedLikeTheRootRecordReaderOfTheCore,
         );
     }
 
@@ -1431,6 +1443,7 @@ impl<'reader> Walk<'reader> {
             tree_table_pointer,
             mapping_root_pointer,
             &version_name,
+            AllZeroTreeTablePointerReading::NoTreeTable,
         );
     }
 
@@ -1443,9 +1456,22 @@ impl<'reader> Walk<'reader> {
         tree_table_pointer: &[u8],
         mapping_root_pointer: &[u8],
         version_name: &str,
+        all_zero_tree_table_pointer_reading: AllZeroTreeTablePointerReading,
     ) {
         // 树表单元：不属于任何树（树 ID 0）。
         let tree_table_pointer_view = parse_node_pointer(tree_table_pointer);
+        if tree_table_pointer_view.all_zero {
+            match all_zero_tree_table_pointer_reading {
+                AllZeroTreeTablePointerReading::FollowedLikeTheRootRecordReaderOfTheCore => {
+                    self.follow_an_all_zero_tree_table_pointer_of_a_root_record(
+                        &tree_table_pointer_view,
+                        version_name,
+                    );
+                    return;
+                }
+                AllZeroTreeTablePointerReading::NoTreeTable => {}
+            }
+        }
         let tree_table_placement = (
             tree_table_pointer_view.locations[0].device,
             tree_table_pointer_view.locations[0].slot,
@@ -1499,6 +1525,32 @@ impl<'reader> Walk<'reader> {
         self.walk_central_mapping_entries(&leaf_entries);
     }
 
+    /// 根环里一条自证过的根，根记录里树表指针整条全零（用户 2026-10-05 定甲，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md`
+    /// 第五节第 1 条「甲 checker 判它损坏」）：core 的根记录读者不把全零当特例，照它那两条位置条目（都是盘 0 槽 0）去读、读不出
+    /// （可写挂载拒成 `Recovery(UnitUnreadable { slot: SlotNumber(0) })`）。checker 跟过去：两条位置条目照 I-2.1 判，这条根的走读记一次失败
+    /// （最新根红 I-7.2、I-4.8、I-7.4，别的候选根红 I-4.8、I-7.4），不再读成「这条根没有树表」。
+    /// 不判 I-2.5：那一行把整条全零的指针整条豁免于升序判定，豁免要不要收窄归写回批。
+    fn follow_an_all_zero_tree_table_pointer_of_a_root_record(
+        &mut self,
+        tree_table_pointer: &PointerView,
+        version_name: &str,
+    ) {
+        let what = format!(
+            "{version_name}的树表（根记录里树表指针全零，照 core 的根记录读者读它那两条位置条目）"
+        );
+        let unit = read_referenced_unit(
+            self.reader,
+            &mut self.judgements,
+            &tree_table_pointer.locations,
+            node_bytes(),
+            &what,
+        );
+        self.walk_failures.push(match unit {
+            None => format!("{what} 两份都读不到对得上的"),
+            Some(_) => format!("{what} 读回来的那一份不在单元区里，不是一张树表"),
+        });
+    }
+
     /// 这一版第一个走到的树表：判条目次序（I-9.16）与每一条的 previous_snapshot_txg（I-9.25），把条目里 I-9.25 与 I-3.5 读的几样记下
     /// （别的版本共享这片树表时照它判），再逐条往下走；走进用户可见树时给记下的落点打上那棵树的标签（I-3.5）。
     fn walk_the_entries_of_a_tree_table_walked_first_by_this_version(
@@ -3886,7 +3938,8 @@ struct RootReferences {
 /// 一条根的树表这一遍读成什么样（[`references_of_root`] 只读不判；I-9.28 拿它判「部分没判」）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum TreeTableReading {
-    /// 树表指针全零：这条根没有树表，不是没读全。
+    /// 树表指针全零：core 的根记录读者照它读盘 0 槽 0、读不出（用户 2026-10-05 定甲），与 [`TreeTableReading::Unreadable`] 一样算没读到，
+    /// 不再算「这条根没有树表」。
     PointerAllZero,
     /// 指针非零，两条位置条目都读不出、或整单元校验和对不上、或解不开成码 2 节点。
     Unreadable,
@@ -4474,9 +4527,16 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
     let mut cache = IndexNodeCache::new();
     // 没判的那几版：「（版本称呼，含实例与 txg）：为什么」，逐版一条，走完一起报（`Judgements::partially_judged` 只记第一次给的说明）。
     let mut versions_not_judged: Vec<String> = Vec::new();
+    // 认出来的从 mkfs 水位第一次建树的那几版（不论有没有数据单元）：一版都没认出来时报专门的不适用理由。
+    let mut versions_built_from_the_mkfs_watermark = 0u64;
     for version in versions {
         let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
         if tree_table_pointer.all_zero {
+            // 用户 2026-10-05 定甲：core 照全零的树表指针读盘 0 槽 0、读不出，与树表读不出同一处置（此前当这一版没有树表、不判也不记）。
+            versions_not_judged.push(format!(
+                "{}：树表指针全零（core 照它读盘 0 槽 0、读不出），认不出它是不是从 mkfs 水位第一次建树的那一版",
+                version.name
+            ));
             continue;
         }
         let Some(tree_table) =
@@ -4527,6 +4587,7 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
         if !is_built_from_the_mkfs_watermark {
             continue;
         }
+        versions_built_from_the_mkfs_watermark += 1;
         let extent_root = parse_node_pointer(&extent_entry[14..100]);
         if extent_root.all_zero {
             continue;
@@ -4609,8 +4670,17 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
             ),
         );
     }
+    // 一版都没认出来时的专门理由（判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P3「四支豁免」那一行：
+    // 此前落到 `Judgements::into_report` 的缺省理由，与「这个函数一次都没跑」分不开）。认出来了而没有数据单元的那一形照旧落到缺省理由
+    // （派发只要了这一句）。`not_applicable` 不改已经评估过的。
+    if versions_built_from_the_mkfs_watermark == 0 {
+        judgements.not_applicable(invariant, NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK);
+    }
 }
 
+/// I-5.5 一次都没判、因为走读到的版本里没有一版满足认法 ①② 时的不适用理由。
+pub const NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK: &str = "回退候选集里每条根与由记录施加出来的那几版，没有一版是从 mkfs 水位第一次建树的那一版（树表非空、种类 1 条目的树 ID 是 65536、每条条目的诞生 txg 等于这一版的 txg）：一版都不判";
+
 /// 一块盘上的开放段：起点是 64 的整数倍的 [`CLUSTER_SEGMENT_SLOTS`] 个槽（D3（空间分配） 已定项 10 ①）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct OpenClusterSegment {
@@ -4875,7 +4945,8 @@ struct TreeTableEntrySightingInTheRing {
 ///   C374（释放代与树表诞生 txg 只有验收断言盯着） 那一类），不报成立；
 /// - 其余 ⇒ 成立。
 ///
-/// 树表指针全零的根没有树表，不算没判到。
+/// 树表指针全零的根照 core 的根记录读者读盘 0 槽 0、读不出，与树表读不出一样报部分没判（用户 2026-10-05 定甲，判决
+/// `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第五节第 1 条；此前算「这条根没有树表，不算没判到」）。
 fn judge_tree_identifier_birth_txg_across_every_readable_root(
     reader: &dyn ImageReader,
     geometry: &PoolGeometry,
@@ -4894,7 +4965,10 @@ fn judge_tree_identifier_birth_txg_across_every_readable_root(
             cache,
         );
         let why_not_judged = match references.tree_table_reading {
-            TreeTableReading::PointerAllZero | TreeTableReading::EveryEntryRead => None,
+            TreeTableReading::EveryEntryRead => None,
+            TreeTableReading::PointerAllZero => {
+                Some("树表指针全零（core 的根记录读者照它读盘 0 槽 0、读不出）")
+            }
             TreeTableReading::Unreadable => Some("树表读不出"),
             TreeTableReading::SomeEntryShorterThanTheFieldTable => Some("树表有条目比字段表短"),
         };
@@ -5033,6 +5107,15 @@ fn judge_release_generation_and_tree_table_birth(
     judge_tree_table_birth_txg(&scanned, judgements);
 }
 
+/// I-5.4 按版本那一层判的一版：称呼（判红与部分没判时点名，说清是候选根、由记录施加出来的那一版、还是下一次挂载先施加的那一版）、
+/// 树表指针与「树表 0 条那一版的分配记录树根」（根环里的根取根记录偏移 36 与 342 起的 86 字节，
+/// 另两类取记录新根段偏移 0 与 274 起的那两条：D23（journal 的角色与格式） 已定项 4 的字段表，施加一条记录 = 把所选根的这几个字段换成记录里的）。
+struct VersionToJudgeTheAllocationRecordsOf<'record> {
+    name: String,
+    tree_table_pointer: &'record [u8],
+    allocation_record_tree_root_held_outside_the_tree_table: &'record [u8],
+}
+
 /// I-5.4（分配记录罩住的槽互不相交）：候选集里每条有效根的分配记录树，同一块盘上任意两条记录（不论已分配、已释放、释放代是否已低于
 /// 回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交（`.claude/kb/invariants.md` I-5.4 那一行）。I-5.1（物理范围不重叠） 管的是
 /// 树里的引用，判不出记录之间的重叠：代码三方第二轮 Z1-d 的镜像上引用互不重叠，而两条分配记录罩住同一个槽，下一次可写挂载重建分配器时 panic。
@@ -5055,10 +5138,15 @@ fn judge_release_generation_and_tree_table_birth(
 /// `research/prompts/m3-formal-code-b29-r1-main-verification.md` 第二节 L2 的零轮收严）：没走到的那几版不计，I-5.4 报部分没判、逐版列出没计的那几版
 /// ——否则带树一版上这一格成立一次就压过不适用或别的版本没判到的那一半，这一版分配记录树两份清零的镜像上 I-5.4 报成立。
 /// 不成立照报违例（[`Judgements::partially_judged`] 的契约：判出违例照报违例）。
+///
+/// 按版本那一层判的不只是候选根：由记录施加出来的那几版（[`versions_applied_only_by_records`]）与下一次挂载先施加的那一版
+/// （[`version_the_next_mount_applies_first`]）也各判一版，树表指针与「树表 0 条那一版的分配记录树根」取记录新根段里的，判法与候选根同一套
+/// （[`VersionToJudgeTheAllocationRecordsOf`]；判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2
+/// 「由记录施加出来的版本与下一次挂载先施加的那一版」那一行的改法 RV：此前这两类版本整版不进，后一类上两条记录罩住同一个槽，
+/// 正是这一条立条时要拦的「下一次可写挂载重建分配器时」那件事）。读不出的照上面几格报部分没判，点名的称呼说清是哪一类版本。
 fn judge_allocation_records_disjoint(
     reader: &dyn ImageReader,
-    roots: &[(u64, u64, crate::RootView)],
-    candidate_indexes: &[usize],
+    versions: &[VersionToJudgeTheAllocationRecordsOf<'_>],
     cache: &mut IndexNodeCache,
     judgements: &mut Judgements,
 ) {
@@ -5074,29 +5162,29 @@ fn judge_allocation_records_disjoint(
         Vec::new();
     // 树表指针非零而树表读不出的那几版：认不出带不带树，「根记录里分配记录树根全零」那一格与这一版自己的分配记录树都没判。
     let mut versions_whose_tree_table_does_not_read_back: Vec<String> = Vec::new();
+    // 树表指针全零的那几版：core 照它读盘 0 槽 0、读不出（用户 2026-10-05 定甲），与树表读不出同一处置；此前当这一版没有树表。
+    let mut versions_whose_tree_table_pointer_is_all_zero: Vec<String> = Vec::new();
     // 树表读得出、0 条，分配记录树由根记录直接持有（C512（树表 0 条的一版上被换下的单元记在哪））而那棵树走不到叶的那几版：
     // 这一版自己的分配记录树没判（判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M2「I-5.4 按版本：坏盘」
     // 那一行改法 F1 的同形，`research/prompts/m3-formal-code-b30-r1-opus-output.md` 第 201 行记下、第 33 批造盘面证实）。
     let mut versions_whose_root_record_allocation_record_tree_does_not_read_back: Vec<String> =
         Vec::new();
-    for index in candidate_indexes.iter().copied() {
-        let (_, _, root) = &roots[index];
+    for version in versions {
         let mut this_version_carries_trees_and_a_zero_root_record_allocation_tree_root = false;
         let mut this_version_holds_its_allocation_record_tree_in_the_root_record = false;
-        // 这条根的分配记录树：树表里种类 3 那一条指着的根，以及树表 0 条那一版由根记录直接持有的那一棵
+        // 这一版的分配记录树：树表里种类 3 那一条指着的根，以及树表 0 条那一版由根记录（或记录新根段）直接持有的那一棵
         // （C512（树表 0 条的一版上被换下的单元记在哪），代码审阅 6c 第 9 条：原来这一棵不读）。带文件的一版后者恒全零。
         let mut allocation_roots = vec![parse_node_pointer(
-            &root.record_bytes[ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT],
+            version.allocation_record_tree_root_held_outside_the_tree_table,
         )];
-        let tree_table_pointer = parse_node_pointer(&root.record_bytes[36..122]);
-        if !tree_table_pointer.all_zero {
+        let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
+        if tree_table_pointer.all_zero {
+            versions_whose_tree_table_pointer_is_all_zero.push(version.name.clone());
+        } else {
             let tree_table_read_back =
                 read_index_node_without_judging(reader, &tree_table_pointer, cache);
             if tree_table_read_back.is_none() {
-                versions_whose_tree_table_does_not_read_back.push(format!(
-                    "根（实例 {}、txg {}）",
-                    root.instance, root.checkpoint_txg
-                ));
+                versions_whose_tree_table_does_not_read_back.push(version.name.clone());
             }
             if let Some(tree_table) = tree_table_read_back {
                 this_version_holds_its_allocation_record_tree_in_the_root_record =
@@ -5108,9 +5196,8 @@ fn judge_allocation_records_disjoint(
                     } else {
                         judgements.judge("I-5.4", false, || {
                             format!(
-                                "根（实例 {}、txg {}）的树表有 {} 条（带树的一版），根记录里树表 0 条那一版的分配记录树根却不是全零：这一版有两棵分配记录树",
-                                root.instance,
-                                root.checkpoint_txg,
+                                "{}的树表有 {} 条（带树的一版），根记录里树表 0 条那一版的分配记录树根却不是全零：这一版有两棵分配记录树",
+                                version.name,
                                 tree_table.entries.len()
                             )
                         });
@@ -5150,23 +5237,21 @@ fn judge_allocation_records_disjoint(
             else {
                 continue;
             };
-            judge_allocation_record_ranges(&records, root.checkpoint_txg, judgements);
+            judge_allocation_record_ranges(&records, &version.name, judgements);
             allocation_roots_walked_to_their_leaves.insert(allocation_root_key);
             this_version_allocation_record_tree_walked_to_its_leaves = true;
         }
         if this_version_carries_trees_and_a_zero_root_record_allocation_tree_root {
             versions_where_the_root_record_allocation_tree_root_is_zero.push((
-                format!("根（实例 {}、txg {}）", root.instance, root.checkpoint_txg),
+                version.name.clone(),
                 this_version_allocation_record_tree_walked_to_its_leaves,
             ));
         }
         if this_version_holds_its_allocation_record_tree_in_the_root_record
             && !this_version_allocation_record_tree_walked_to_its_leaves
         {
-            versions_whose_root_record_allocation_record_tree_does_not_read_back.push(format!(
-                "根（实例 {}、txg {}）",
-                root.instance, root.checkpoint_txg
-            ));
+            versions_whose_root_record_allocation_record_tree_does_not_read_back
+                .push(version.name.clone());
         }
     }
     let some_allocation_record_tree_walked_to_its_leaves =
@@ -5208,6 +5293,12 @@ fn judge_allocation_records_disjoint(
             versions_whose_tree_table_does_not_read_back.join("、")
         ));
     }
+    if !versions_whose_tree_table_pointer_is_all_zero.is_empty() {
+        unjudged_parts.push(format!(
+            "树表指针全零的版本（core 照它读盘 0 槽 0、读不出），认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：{}",
+            versions_whose_tree_table_pointer_is_all_zero.join("、")
+        ));
+    }
     if !versions_whose_root_record_allocation_record_tree_does_not_read_back.is_empty() {
         unjudged_parts.push(format!(
             "树表 0 条、分配记录树由根记录直接持有的版本，那棵树读不出、走不到叶，它自己的分配记录一条都没判（别的版本的树走到了叶不替它作保）：{}",
@@ -5254,6 +5345,26 @@ fn tree_table_pointer_of_the_version_the_next_mount_applies_first(
     roots: &[(u64, u64, crate::RootView)],
     newest_index: usize,
 ) -> Option<PointerView> {
+    version_the_next_mount_applies_first(records_by_device, roots, newest_index).map(|version| {
+        parse_node_pointer(&version.new_root_segment[NEW_ROOT_SEGMENT_TREE_TABLE_POINTER])
+    })
+}
+
+/// 下一次挂载的恢复会先施加的那一版：记录已持久、根槽没落盘、还没有哪一次恢复施加过它（与 [`VersionAppliedOnlyByRecords`] 不是一类：
+/// 那一类已被施加、实例表里有行）。
+struct VersionTheNextMountAppliesFirst {
+    instance: u32,
+    checkpoint_txg: u64,
+    new_root_segment: Vec<u8>,
+}
+
+/// 下一次挂载的恢复会先施加的那一版（认法见 [`tree_table_pointer_of_the_version_the_next_mount_applies_first`]）：
+/// 它的 (实例代号, txg) 与那条记录的整段新根段。I-3.10 只取树表指针，I-5.4 按版本那一层还要「树表 0 条那一版的分配记录树根」。
+fn version_the_next_mount_applies_first(
+    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
+    roots: &[(u64, u64, crate::RootView)],
+    newest_index: usize,
+) -> Option<VersionTheNextMountAppliesFirst> {
     let newest_root = &roots[newest_index].2;
     let next_txg = newest_root.checkpoint_txg.checked_add(1)?;
     let root_slot_is_readable = roots.iter().any(|(_, _, root)| {
@@ -5277,8 +5388,10 @@ fn tree_table_pointer_of_the_version_the_next_mount_applies_first(
                     record,
                 )
         })
-        .map(|(_, record)| {
-            parse_node_pointer(&record.new_root_segment[NEW_ROOT_SEGMENT_TREE_TABLE_POINTER])
+        .map(|(_, record)| VersionTheNextMountAppliesFirst {
+            instance: record.instance,
+            checkpoint_txg: record.checkpoint_txg,
+            new_root_segment: record.new_root_segment.clone(),
         })
 }
 
@@ -5889,7 +6002,7 @@ fn judge_system_configuration_floor_against_the_roots_on_each_device(
 /// 一棵分配记录树的全部记录上逐盘判 I-5.4：同一块盘上的记录按起点排好，相邻两条不相交就是两两不相交（跨度非负）。
 fn judge_allocation_record_ranges(
     records: &[AllocationRecordView],
-    root_checkpoint_txg: u64,
+    version_name: &str,
     judgements: &mut Judgements,
 ) {
     let mut ranges_per_device: BTreeMap<u32, Vec<(u64, u64)>> = BTreeMap::new();
@@ -5909,7 +6022,7 @@ fn judge_allocation_record_ranges(
                 let ((first_slot, first_span), (second_slot, second_span)) = (pair[0], pair[1]);
                 judgements.judge("I-5.4", false, || {
                     format!(
-                        "txg {root_checkpoint_txg} 的根那棵分配记录树、盘 {device}：槽 {first_slot} 跨 {first_span} 的记录与槽 {second_slot} 跨 {second_span} 的记录罩住同一个槽"
+                        "{version_name}那棵分配记录树、盘 {device}：槽 {first_slot} 跨 {first_span} 的记录与槽 {second_slot} 跨 {second_span} 的记录罩住同一个槽"
                     )
                 });
             }
@@ -8146,6 +8259,7 @@ struct InodeRecordOfTheDeletionScan {
 /// （I-2.1、I-1.1、I-1.7、I-1.10、I-9.1、I-9.27 等），这一遍只如实记「判不了」，不进 `walk_failures`。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum DeletionScanGap {
+    TreeTablePointerAllZero,
     TreeTableUnreadable,
     TreeTableEntryShorterThanTheFieldTable,
     DeletedInodesHeadIdentifierNamesNoSingleInodeTree,
@@ -8169,6 +8283,9 @@ enum DeletionScanGap {
 impl DeletionScanGap {
     fn reason(self) -> &'static str {
         match self {
+            DeletionScanGap::TreeTablePointerAllZero => {
+                "走读到的某一版树表指针全零（core 照它读盘 0 槽 0、读不出）：那一版的删除态射程里判不了"
+            }
             DeletionScanGap::TreeTableUnreadable => {
                 "走读到的某一版树表读不出（两份都对不上整单元校验和或解不开）：那一版的删除态射程里判不了"
             }
@@ -8457,7 +8574,8 @@ fn collect_accounting_rows_without_judging(
 /// 加由记录施加出来的那几版；树表单元相同的几版内容相同，只扫一次）各自重读一遍树表与几棵树，只读、不进 `walk_failures`，
 /// 判 I-3.4（[`judge_pending_deletion_occupancy_of_a_version`]）与 I-9.5、I-4.7、I-4.6（[`judge_deletion_queue_of_a_version`]）。
 /// 主走读按 `visited_units` 只走一遍共享子树，这几条要的是「这一版自己的」队列、记录与单元数，所以另走一遍（与 [`references_of_root`] 同一个缘故）。
-/// 树表指针全零、树表 0 条的那一版没有这几棵树，不判。
+/// 树表 0 条的那一版没有这几棵树，不判；树表指针全零的那一版照 core 的根记录读者读盘 0 槽 0、读不出，报射程里判不了
+/// （用户 2026-10-05 定甲，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第五节第 1 条；此前同样算没有这几棵树）。
 fn judge_deletion_state_of_the_versions(
     reader: &dyn ImageReader,
     versions: &[VersionToScanTheDeletionStateOf<'_>],
@@ -8469,6 +8587,15 @@ fn judge_deletion_state_of_the_versions(
     for version in versions {
         let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
         if tree_table_pointer.all_zero {
+            for invariant in DELETION_QUEUE_SCAN_INVARIANTS.into_iter().chain([
+                PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
+                DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
+            ]) {
+                judgements.part_of_the_range_not_judged(
+                    invariant,
+                    DeletionScanGap::TreeTablePointerAllZero.reason(),
+                );
+            }
             continue;
         }
         let first_location = tree_table_pointer.locations[0];
@@ -8521,11 +8648,11 @@ fn judge_deletion_state_of_the_versions(
     // 一次都没判到时的理由（判到过的照判定报，`Judgements::not_applicable` 不改已经评估过的）。
     judgements.not_applicable(
         DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
-        "走读到的每一版树表里都没有 deleted_inodes 条目（树表指针全零或 0 条）：没有队列可比",
+        "走读到的每一版树表里都没有 deleted_inodes 条目（树表 0 条）：没有队列可比",
     );
     judgements.not_applicable(
         PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
-        "走读到的每一版树表里都没有记账树（树表指针全零或 0 条）",
+        "走读到的每一版树表里都没有记账树（树表 0 条）",
     );
     judgements.not_applicable(
         INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
@@ -8537,7 +8664,7 @@ fn judge_deletion_state_of_the_versions(
     );
     judgements.not_applicable(
         DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
-        "走读到的每一版树表里都没有 deleted_inodes 条目（树表指针全零或 0 条）：没有队列可判",
+        "走读到的每一版树表里都没有 deleted_inodes 条目（树表 0 条）：没有队列可判",
     );
 }
 
@@ -9310,6 +9437,15 @@ fn judge_tombstones_and_the_lineage_of_the_versions(
     for version in versions {
         let tree_table_pointer = parse_node_pointer(version.tree_table_pointer);
         if tree_table_pointer.all_zero {
+            for invariant in TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS
+                .into_iter()
+                .chain([capacity_invariant])
+            {
+                judgements.part_of_the_range_not_judged(
+                    invariant,
+                    DeletionScanGap::TreeTablePointerAllZero.reason(),
+                );
+            }
             continue;
         }
         let first_location = tree_table_pointer.locations[0];
@@ -10478,7 +10614,7 @@ pub fn check_pool_image_given_the_tombstone_container_capacity_knob(
         format!("最新的根走不完：{}", newest_failures.join("；"))
     });
     // I-1.2 与 I-4.2 在遍历方向上判的是同一批单元：走读一个被引用单元都没走到时两条都没有对象
-    // （根槽读得出而树表指针全零、或每个单元的头都用不了），报「不适用」并带理由，不报成立。
+    // （根槽读得出而树表读不出、或每个单元的头都用不了），报「不适用」并带理由，不报成立。
     if referenced_units_judged_against_their_pointer == 0 {
         for invariant in ["I-1.2", "I-4.2"] {
             judgements.not_applicable(
@@ -10519,10 +10655,50 @@ pub fn check_pool_image_given_the_tombstone_container_capacity_knob(
         &mut index_node_cache,
         &mut judgements,
     );
+    // I-5.4 按版本那一层：回退候选集里每条根，加由记录施加出来的那几版与下一次挂载先施加的那一版（改法 RV，见
+    // [`judge_allocation_records_disjoint`]）。坏盘上同一版两类都认得出时判两遍：同一棵分配记录树按根节点只读一次，违例不重报。
+    let version_the_next_mount_applies_first =
+        version_the_next_mount_applies_first(&journal_records_by_device, &roots, newest_index);
+    let versions_to_judge_the_allocation_records_of: Vec<VersionToJudgeTheAllocationRecordsOf<'_>> =
+        candidate_indexes
+            .iter()
+            .map(|index| {
+                let root = &roots[*index].2;
+                VersionToJudgeTheAllocationRecordsOf {
+                    name: format!("根（实例 {}、txg {}）", root.instance, root.checkpoint_txg),
+                    tree_table_pointer: &root.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],
+                    allocation_record_tree_root_held_outside_the_tree_table: &root.record_bytes
+                        [ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT],
+                }
+            })
+            .chain(versions_applied_only_by_records.iter().map(|version| {
+                VersionToJudgeTheAllocationRecordsOf {
+                    name: format!(
+                        "由记录施加出来的那一版（实例 {}、txg {}）",
+                        version.instance, version.checkpoint_txg
+                    ),
+                    tree_table_pointer: &version.new_root_segment
+                        [NEW_ROOT_SEGMENT_TREE_TABLE_POINTER],
+                    allocation_record_tree_root_held_outside_the_tree_table: &version
+                        .new_root_segment[NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER],
+                }
+            }))
+            .chain(version_the_next_mount_applies_first.iter().map(|version| {
+                VersionToJudgeTheAllocationRecordsOf {
+                    name: format!(
+                        "下一次挂载先施加的那一版（实例 {}、txg {}）",
+                        version.instance, version.checkpoint_txg
+                    ),
+                    tree_table_pointer: &version.new_root_segment
+                        [NEW_ROOT_SEGMENT_TREE_TABLE_POINTER],
+                    allocation_record_tree_root_held_outside_the_tree_table: &version
+                        .new_root_segment[NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER],
+                }
+            }))
+            .collect();
     judge_allocation_records_disjoint(
         reader,
-        &roots,
-        &candidate_indexes,
+        &versions_to_judge_the_allocation_records_of,
         &mut index_node_cache,
         &mut judgements,
     );
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index d930706..99617f3 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -893,52 +893,76 @@ fn judge_allocation_record_ranges() {
     }
 }
 
-// I-5.4：候选集里每条根的分配记录树（树表种类 3 那一条与根记录直接持有的那一棵）；同一条下另判一格「一条根只有一棵分配记录树」：
-// 树表读得出、条目非空（带树）的那一版，根记录偏移 342 那条分配记录树根必须全零（walk.rs judge_allocation_records_disjoint，
-// 判决 m3-formal-c655-c687-r1 第二节 Y1 allocation_record_tree_root 那一行）。条目宽不到 200 的表也判，所以放在跳过窄表那一句之前
-fn judge_allocation_records_disjoint() {
-    c_seen_node_count = 0u;
-    r_noting = false;
-    for (var c = 0u; c < c_candidate_count; c = c + 1u) {
-        let root = checker_root_of(sw(C_CANDIDATES + c));
-        let held = parse_node_pointer(root.base, 342u);
-        if (!held.all_zero && c_seen_node_insert(held)) {
-            c_alloc_record_count = 0u;
-            if (r_allocation_tree(root.base, 342u)) {
-                judge_allocation_record_ranges();
-            }
-        }
-        let tree_table_pointer = parse_node_pointer(root.base, 36u);
-        if (tree_table_pointer.all_zero) {
-            continue;
+// I-5.4 的一版（walk.rs judge_allocation_records_disjoint 循环里的一轮）：base 里 tree_table_offset 起是树表指针，held_offset 起是
+// 「树表 0 条那一版的分配记录树根」（候选根取根记录偏移 36 与 342，由记录造出来的版本取记录新根段打头与新根段偏移 274）；
+// 同一条下另判一格「一条根只有一棵分配记录树」：树表读得出、条目非空（带树）的那一版，那条分配记录树根必须全零（判决 m3-formal-c655-c687-r1
+// 第二节 Y1 allocation_record_tree_root 那一行）。条目宽不到 200 的表也判，所以放在跳过窄表那一句之前。读不出的那几格 CPU 报部分没判，这里不置位
+fn judge_allocation_records_disjoint_of_a_version(base: u32, tree_table_offset: u32, held_offset: u32) {
+    let held = parse_node_pointer(base, held_offset);
+    if (!held.all_zero && c_seen_node_insert(held)) {
+        c_alloc_record_count = 0u;
+        if (r_allocation_tree(base, held_offset)) {
+            judge_allocation_record_ranges();
         }
-        let tt = r_read_index_node(tree_table_pointer);
-        if (!tt.ok) {
+    }
+    let tree_table_pointer = parse_node_pointer(base, tree_table_offset);
+    if (tree_table_pointer.all_zero) {
+        return;
+    }
+    let tt = r_read_index_node(tree_table_pointer);
+    if (!tt.ok) {
+        return;
+    }
+    if (tt.entry_count > 0u && !held.all_zero) {
+        violate(INVARIANT_I_5_4);
+    }
+    if (tt.entry_width < 200u) {
+        return;
+    }
+    for (var e = 0u; e < tt.entry_count; e = e + 1u) {
+        let o = entry_offset(tt, e);
+        if (rd16(tt.base, o + 10u) != TREE_KIND_ALLOCATION) {
             continue;
         }
-        if (tt.entry_count > 0u && !held.all_zero) {
-            violate(INVARIANT_I_5_4);
-        }
-        if (tt.entry_width < 200u) {
+        let p = parse_node_pointer(tt.base, o + 14u);
+        if (p.all_zero || !c_seen_node_insert(p)) {
             continue;
         }
-        for (var e = 0u; e < tt.entry_count; e = e + 1u) {
-            let o = entry_offset(tt, e);
-            if (rd16(tt.base, o + 10u) != TREE_KIND_ALLOCATION) {
-                continue;
-            }
-            let p = parse_node_pointer(tt.base, o + 14u);
-            if (p.all_zero || !c_seen_node_insert(p)) {
-                continue;
-            }
-            c_alloc_record_count = 0u;
-            if (r_allocation_tree(tt.base, o + 14u)) {
-                judge_allocation_record_ranges();
-            }
+        c_alloc_record_count = 0u;
+        if (r_allocation_tree(tt.base, o + 14u)) {
+            judge_allocation_record_ranges();
         }
     }
 }
 
+// I-5.4：候选集里每条根，加由记录施加出来的那几版与下一次挂载先施加的那一版（walk.rs judge_allocation_records_disjoint 的改法 RV，
+// 判决 m3-formal-code-b33-r1 第二节 P2）。后两类的两条指针在记录新根段里
+fn judge_allocation_records_disjoint() {
+    c_seen_node_count = 0u;
+    r_noting = false;
+    for (var c = 0u; c < c_candidate_count; c = c + 1u) {
+        let root = checker_root_of(sw(C_CANDIDATES + c));
+        judge_allocation_records_disjoint_of_a_version(root.base, 36u, 342u);
+    }
+    for (var v = 0u; v < c_version_count; v = v + 1u) {
+        let r = checker_record_at(sw(R_DATA + v));
+        judge_allocation_records_disjoint_of_a_version(
+            r.base,
+            JOURNAL_NEW_ROOT_SEGMENT_OFFSET,
+            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER,
+        );
+    }
+    let next_version = record_index_of_the_version_the_next_mount_applies_first();
+    if (next_version != NONE) {
+        let r = checker_record_at(next_version);
+        judge_allocation_records_disjoint_of_a_version(
+            r.base,
+            JOURNAL_NEW_ROOT_SEGMENT_OFFSET,
+            JOURNAL_NEW_ROOT_SEGMENT_OFFSET + JOURNAL_NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER,
+        );
+    }
+}
+
 // I-3.10 的一棵树：未释放的记录（按盘、槽、代去重）的分配代等于它罩住的单元头里的诞生代号
 fn judge_allocation_generations_of_the_records() {
     for (var i = 0u; i < c_alloc_record_count; i = i + 1u) {
@@ -1036,27 +1060,33 @@ fn judge_allocation_generations_against_unit_births() {
         let r = checker_record_at(sw(R_DATA + v));
         judge_allocation_generations_under_tree_table(parse_node_pointer(r.base, 99u));
     }
-    // 下一次挂载会先施加的那一版：与最新根同实例、txg = 最新根 txg + 1、带提交标记、进得了前缀、根槽读不出
+    let next_version = record_index_of_the_version_the_next_mount_applies_first();
+    if (next_version != NONE) {
+        judge_allocation_generations_under_tree_table(parse_node_pointer(checker_record_at(next_version).base, JOURNAL_NEW_ROOT_SEGMENT_OFFSET));
+    }
+}
+
+// 下一次挂载会先施加的那一版（walk.rs version_the_next_mount_applies_first）：与最新根同实例、txg = 最新根 txg + 1、带提交标记、
+// 进得了前缀、根槽读不出的第一条记录的下标；没有交回 NONE
+fn record_index_of_the_version_the_next_mount_applies_first() -> u32 {
     let newest_root = checker_root_of(c_newest_index);
-    if (!add64_overflows(newest_root.txg, u64_of(1u))) {
-        let next_txg = add64(newest_root.txg, u64_of(1u));
-        var root_slot_is_readable = false;
-        for (var i = 0u; i < root_count; i = i + 1u) {
-            let root = checker_root_of(i);
-            if (root.instance == newest_root.instance && eq64(root.txg, next_txg)) {
-                root_slot_is_readable = true;
-            }
-        }
-        if (!root_slot_is_readable) {
-            for (var i = 0u; i < record_count; i = i + 1u) {
-                let r = checker_record_at(i);
-                if (r.commit_byte == 1u && r.instance == newest_root.instance && eq64(r.txg, next_txg) && record_can_enter_a_replay_prefix(i)) {
-                    judge_allocation_generations_under_tree_table(parse_node_pointer(r.base, 99u));
-                    break;
-                }
-            }
+    if (add64_overflows(newest_root.txg, u64_of(1u))) {
+        return NONE;
+    }
+    let next_txg = add64(newest_root.txg, u64_of(1u));
+    for (var i = 0u; i < root_count; i = i + 1u) {
+        let root = checker_root_of(i);
+        if (root.instance == newest_root.instance && eq64(root.txg, next_txg)) {
+            return NONE;
         }
     }
+    for (var i = 0u; i < record_count; i = i + 1u) {
+        let r = checker_record_at(i);
+        if (r.commit_byte == 1u && r.instance == newest_root.instance && eq64(r.txg, next_txg) && record_can_enter_a_replay_prefix(i)) {
+            return i;
+        }
+    }
+    return NONE;
 }
 
 // instance_table_rows_of_root_without_judging：一条根自己的实例表沿链读出全部行，读不出交回 false
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 63b584d..6521657 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -4814,6 +4814,75 @@ fn the_gpu_judge_agrees_with_the_cpu_when_a_version_with_trees_also_holds_an_all
     );
 }
 
+/// 一片分配记录叶（码 2、层级 0、key 宽 10）里第一对「同一块盘、前一条正好接到后一条」的记录，前一条跨度加一（已释放标志照留）：
+/// 两条罩住同一个槽。返回改了几条（0 或 1）。抄自 harness 档 `checker_known_bad_images.rs` 的同名函数（两档不共用代码）。
+fn widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
+    bytes: &mut [u8],
+) -> usize {
+    const ALLOCATION_KEY_WIDTH: u8 = 10;
+    const ALLOCATION_RECORD_BYTES: usize = 20;
+    const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
+    if bytes[50] != 0 || index_node_key_width(bytes) != ALLOCATION_KEY_WIDTH {
+        return 0;
+    }
+    let header_end = 86 + 2 * usize::from(ALLOCATION_KEY_WIDTH);
+    let record_count = usize::from(u16::from_le_bytes(
+        bytes[header_end - 4..header_end - 2].try_into().expect("2"),
+    ));
+    let entries_start = 115 + 2 * usize::from(ALLOCATION_KEY_WIDTH);
+    let record_at = |leaf_bytes: &[u8], record_index: usize| {
+        let start = entries_start + ALLOCATION_RECORD_BYTES * record_index;
+        let device = u32::from_le_bytes(leaf_bytes[start..start + 4].try_into().expect("4"));
+        let mut slot_bytes = [0u8; 8];
+        slot_bytes[..6].copy_from_slice(&leaf_bytes[start + 4..start + 10]);
+        let span_field = u16::from_le_bytes([leaf_bytes[start + 10], leaf_bytes[start + 11]]);
+        (start, device, u64::from_le_bytes(slot_bytes), span_field)
+    };
+    for record_index in 0..record_count.saturating_sub(1) {
+        let (start, device, slot, span_field) = record_at(bytes, record_index);
+        let (_, next_device, next_slot, _) = record_at(bytes, record_index + 1);
+        let span_slots = span_field & !ALLOCATION_RECORD_RELEASED_FLAG;
+        if device == next_device && slot + u64::from(span_slots) == next_slot {
+            let widened = (span_field & ALLOCATION_RECORD_RELEASED_FLAG) | (span_slots + 1);
+            bytes[start + 10..start + 12].copy_from_slice(&widened.to_le_bytes());
+            return 1;
+        }
+    }
+    0
+}
+
+/// I-5.4 按版本那一层多收下一次挂载先施加的那一版（改法 RV，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2）：
+/// 新池新建文件那次发布写的最后一片分配记录叶里两条记录罩住同一个槽，重封到根与 journal 记录。发布那条记录落了、根槽没落的那几个状态上，
+/// 这一版是下一次挂载先施加的那一版，只有改法 RV 之后的 I-5.4 判得到；根槽落了的状态上它是最新的根。两边每个状态逐位比。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
+        "the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first",
+        &record_new_pool_with_a_resealed_field(
+            "crash-judge-gpu-i54-overlapping-records",
+            "new-pool-overlapping-allocation-records",
+            &|bytes| {
+                unit_class(bytes) == 2
+                    && index_node_key_width(bytes) == 10
+                    && bytes[50] == 0
+                    && widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
+                        &mut bytes.to_vec(),
+                    ) == 1
+            },
+            false,
+            &|bytes| {
+                assert_eq!(
+                    widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
+                    1,
+                    "挑叶时这一片改得动"
+                );
+            },
+        ),
+        "I-5.4",
+    );
+}
+
 /// I-2.4：树表条目里码 2 节点的指针头部 extent 偏移非 0（池级 checker 与恢复两半都比）。
 #[test]
 fn the_gpu_judge_agrees_with_the_cpu_when_a_code_two_node_pointer_carries_an_extent_offset() {
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index ccad789..9c267c9 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -4871,6 +4871,8 @@ struct ImageAfterAVersionAppliedOnlyByItsJournalRecord {
     newest_accounting_root_slot: u64,
     /// 只由记录施加出来的那一版 (1, 5) 自己的树表单元：写行发布 txg 6 把它换下放进 defer，根环里没有一条根引用它。
     tree_table_slot_of_the_version_applied_only_by_its_record: u64,
+    /// 只由记录施加出来的那一版 (1, 5) 那次发布的输出（它写了哪些单元、重写了哪几样、那条 journal 记录）。
+    version_applied_only_by_its_record: TransactionOutput,
 }
 
 fn image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
@@ -4967,6 +4969,7 @@ fn image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_recor
             .unit(TransactionUnit::TreeTable)
             .slot
             .0,
+        version_applied_only_by_its_record: applied_only_by_its_record,
     }
 }
 
@@ -5661,6 +5664,307 @@ fn an_allocation_generation_past_its_unit_birth_in_the_version_only_its_journal_
     );
 }
 
+/// 前后两份整单元校验和（[`unit_checksums`]）里变了的那几个单元：(起点槽, 旧, 新)，交给 [`propagate_into_journal_records`]。
+fn units_whose_checksum_changed(
+    before: &[(u64, u32)],
+    after: &[(u64, u32)],
+) -> Vec<(u64, u32, u32)> {
+    before
+        .iter()
+        .zip(after)
+        .filter(|((_, old), (_, new))| old != new)
+        .map(|((slot, old), (_, new))| (*slot, *old, *new))
+        .collect()
+}
+
+/// 改法 RV（判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2「由记录施加出来的版本与下一次挂载先施加的那一版」
+/// 那一行，攻方报告 `research/prompts/m3-formal-code-b33-r1-opus-output.md` P2-a 的 b3 形）：I-5.4 按版本那一层也判下一次挂载先施加的那一版。
+/// 发布 B 的记录已落、根槽没落的崩溃镜像上，B 那棵分配记录树里 A 的 inode 树根那个落点（槽 50244）的记录跨度改成 2
+/// （与 [`known_bad_images_of_overlapping_allocation_records`] 第一份同一种改法，两盘各一条），新的整单元校验和沿引用链补到 B 的树表，
+/// 再补进 B 那条记录（两块盘各一份）：
+/// ① 只红 I-5.4、别的判定与干净的崩溃镜像逐项相同，违例点名「下一次挂载先施加的那一版（实例 1、txg 4）」——改之前这一版整版不进
+///    I-5.4，这张镜像一条都不红；
+/// ② 可写挂载重建分配器时拒在「同一块盘上两条分配记录罩住同一个槽」上，两块盘逐字节不变：I-5.4 立条时要拦的
+///    「下一次可写挂载重建分配器时两条记录罩住同一个槽」就是这一版。
+#[test]
+fn two_allocation_records_covering_one_slot_in_the_version_the_next_mount_applies_first_redden_only_the_disjointness_invariant(
+) {
+    let clean = crash_image_with_the_overwrite_record_but_not_its_root(
+        "known-bad-overlapping-records-of-the-version-the-next-mount-applies",
+    );
+    let verdicts_on_the_clean_image = check_pool_image(&clean);
+    assert_eq!(
+        violated_invariants(&verdicts_on_the_clean_image),
+        Vec::<&str>::new(),
+        "B 的记录已落、根槽没落的干净崩溃镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
+    );
+    assert_eq!(
+        verdict(&clean, "I-5.4"),
+        InvariantVerdict::Holds,
+        "干净的崩溃镜像上 I-5.4 要真被评估过且成立"
+    );
+    let mut image = clean.clone();
+    let checksums_before = unit_checksums(&image, &UNITS_AFTER_OVERWRITE);
+    assert_eq!(
+        mutate_allocation_leaves(
+            &mut image,
+            &UNITS_AFTER_OVERWRITE,
+            ALLOCATION_LEAVES_AFTER_OVERWRITE,
+            |bytes| set_allocation_record_span(bytes, INODE_ROOT, 2),
+        ),
+        2,
+        "A 的 inode 树根那一个落点，B 那棵账里两盘各一条"
+    );
+    let changes = units_whose_checksum_changed(
+        &checksums_before,
+        &unit_checksums(&image, &UNITS_AFTER_OVERWRITE),
+    );
+    assert_eq!(
+        propagate_into_journal_records(&mut image, &changes),
+        2,
+        "B 那条记录两块盘各一份，新根段里的树表指针都要补"
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-5.4"],
+        "B 那一版只在记录里时，它的两条记录罩住同一个槽要在崩溃镜像上就红、只红 I-5.4：{verdicts:?}"
+    );
+    assert_eq!(
+        verdicts_that_differ_outside(&verdicts, &verdicts_on_the_clean_image, &["I-5.4"]),
+        [],
+        "I-5.4 之外每一条的判定与干净的崩溃镜像逐项相同"
+    );
+    let InvariantVerdict::Violated(detail) = verdict(&image, "I-5.4") else {
+        panic!("上面判过 I-5.4 红");
+    };
+    assert!(
+        detail.starts_with("下一次挂载先施加的那一版（实例 1、txg 4）那棵分配记录树")
+            && detail.contains(&format!("槽 {INODE_ROOT} 跨 2 的记录")),
+        "红在 B 那一版（下一次挂载先施加的那一版）的分配记录树上、槽 {INODE_ROOT} 跨 2 那一条：{detail}"
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
+        "重建 B 那一版的分配器时两条记录罩住同一个槽，可写挂载要拒在这一条上：{refusal:?}"
+    );
+    assert_eq!(
+        memory_pool_of_sparse_devices(&devices),
+        image_before_the_mount,
+        "拒在任何写之前：两块盘逐字节不变（{refusal:?}）"
+    );
+}
+
+/// 一片分配记录叶（码 2、层级 0、key 宽 10）里第一对「同一块盘、前一条正好接到后一条」的记录，前一条跨度加一（已释放标志照留）：
+/// 两条罩住同一个槽（I-5.4 那一行的 Z1-d 形态）。返回改了几条（0 或 1）。
+fn widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
+    bytes: &mut [u8],
+) -> usize {
+    if bytes[INDEX_NODE_LEVEL_OFFSET] != 0 || bytes[INDEX_NODE_KEY_WIDTH_OFFSET] != 10 {
+        return 0;
+    }
+    let record_count = usize::from(u16::from_le_bytes([
+        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
+        bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
+    ]));
+    let record_at = |leaf_bytes: &[u8], record_index: usize| {
+        let start = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * record_index;
+        let device = u32::from_le_bytes(leaf_bytes[start..start + 4].try_into().expect("4 字节"));
+        let mut slot_bytes = [0u8; 8];
+        slot_bytes[..6].copy_from_slice(&leaf_bytes[start + 4..start + 10]);
+        let span_field = u16::from_le_bytes([leaf_bytes[start + 10], leaf_bytes[start + 11]]);
+        (start, device, u64::from_le_bytes(slot_bytes), span_field)
+    };
+    for record_index in 0..record_count.saturating_sub(1) {
+        let (start, device, slot, span_field) = record_at(bytes, record_index);
+        let (_, next_device, next_slot, _) = record_at(bytes, record_index + 1);
+        let span_slots = span_field & !ALLOCATION_RECORD_RELEASED_FLAG;
+        if device == next_device && slot + u64::from(span_slots) == next_slot {
+            set_u16(
+                bytes,
+                start + 10,
+                (span_field & ALLOCATION_RECORD_RELEASED_FLAG) | (span_slots + 1),
+            );
+            return 1;
+        }
+    }
+    0
+}
+
+/// 一次可写挂载接在「由记录施加出来、根槽从没落盘」的那一版 (1, 5) 之后的镜像上（[`image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record`]）：
+/// 改 (1, 5) 这次发布自己重写的第一片能改的分配记录叶（[`widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts`]），
+/// 叶重封、新的整单元校验和沿引用链补上去（叶 → 内部节点 → 根 → 树表），再补进 (1, 5) 那条记录（两块盘各一份）。交回改的是哪一片叶。
+fn overlap_two_records_in_the_version_applied_only_by_its_journal_record(
+    image: &mut MemoryPool,
+    version: &TransactionOutput,
+) -> u64 {
+    let units = units_found_on_device_zero(image);
+    let checksums_before = unit_checksums(image, &units);
+    let leaf = version
+        .units
+        .iter()
+        .filter(|unit| {
+            version.rewritten.contains(&unit.identity)
+                && matches!(
+                    unit.identity,
+                    TransactionUnit::AllocationTree
+                        | TransactionUnit::AllocationTreeNodeBelowTheRoot(_)
+                )
+        })
+        .map(|unit| unit.slot.0)
+        .find(|slot| {
+            let mut candidate_leaf = read(image, 0, slot * SLOT, 16384);
+            widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(
+                &mut candidate_leaf,
+            ) == 1
+        })
+        .expect("(1, 5) 这次重写的分配记录叶里有一对同盘、首尾相接的记录");
+    mutate_unit_of(
+        image,
+        &units,
+        leaf,
+        |bytes| {
+            assert_eq!(
+                widen_the_first_record_that_ends_where_the_next_one_on_its_device_starts(bytes),
+                1,
+                "上面挑叶时这一片改得动"
+            );
+        },
+        true,
+    );
+    let changes = units_whose_checksum_changed(&checksums_before, &unit_checksums(image, &units));
+    assert_eq!(
+        propagate_into_journal_records(image, &changes),
+        2,
+        "只有 (1, 5) 那条记录（两块盘各一份）引用这一版改过的单元"
+    );
+    leaf
+}
+
+/// 改法 RV（判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2，攻方报告
+/// `research/prompts/m3-formal-code-b33-r1-opus-output.md` P2-a 的 a3 形）：I-5.4 按版本那一层也判由记录施加出来的那几版。
+/// 挂载之后 (1, 5) 只由记录施加出来（根环里没有它的根），它自己重写的一片分配记录叶里两条记录罩住同一个槽：只红 I-5.4，
+/// 违例点名「由记录施加出来的那一版（实例 1、txg 5）」。改之前这一版整版不进 I-5.4，这张镜像一条都不红。
+#[test]
+fn two_allocation_records_covering_one_slot_in_a_version_applied_only_by_its_journal_record_redden_only_the_disjointness_invariant(
+) {
+    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(
+        "known-bad-overlapping-records-of-a-version-applied-only-by-its-record",
+    );
+    let verdicts_on_the_clean_image = check_pool_image(&built.image);
+    assert_eq!(
+        violated_invariants(&verdicts_on_the_clean_image),
+        Vec::<&str>::new(),
+        "挂载之后的干净镜像上一条都不许红：{verdicts_on_the_clean_image:?}"
+    );
+    let mut image = built.image.clone();
+    let leaf = overlap_two_records_in_the_version_applied_only_by_its_journal_record(
+        &mut image,
+        &built.version_applied_only_by_its_record,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-5.4"],
+        "(1, 5) 的叶 {leaf} 里两条记录罩住同一个槽：只红 I-5.4：{verdicts:?}"
+    );
+    let InvariantVerdict::Violated(detail) = verdict(&image, "I-5.4") else {
+        panic!("上面判过 I-5.4 红");
+    };
+    assert!(
+        detail.starts_with("由记录施加出来的那一版（实例 1、txg 5）那棵分配记录树"),
+        "红在由记录施加出来的那一版 (1, 5) 的分配记录树上：{detail}"
+    );
+}
+
+/// 改法 RV 读不出的两形（攻方报告 `research/prompts/m3-formal-code-b33-r1-opus-output.md` P2-a 的 a1、a2 形，F1、33-c 的同形）：
+/// 挂载之后把只由记录施加出来的那一版 (1, 5) 的一个单元两份清零，判红的照改之前那几条（`violated`），I-5.4 不再报成立、
+/// 报部分没判并点名「由记录施加出来的那一版（实例 1、txg 5）」（`partially_judged_detail`）。
+fn assert_a_unit_of_the_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
+    tag: &str,
+    slot_of: impl Fn(&ImageAfterAVersionAppliedOnlyByItsJournalRecord) -> u64,
+    violated: &[&str],
+    partially_judged_detail: &str,
+) {
+    let built = image_after_a_writable_mount_over_a_version_applied_only_by_its_journal_record(tag);
+    assert_eq!(
+        verdict(&built.image, "I-5.4"),
+        InvariantVerdict::Holds,
+        "挂载之后的干净镜像上 I-5.4 成立"
+    );
+    let mut image = built.image.clone();
+    zero_both_copies_of_the_unit(
+        &mut image,
+        slot_of(&built),
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
+        InvariantVerdict::PartiallyJudged(partially_judged_detail.to_owned()),
+        "(1, 5) 那一版读不出的那一格：别的版本的树走到了叶不替它计成立，I-5.4 报部分没判、点名它是由记录施加出来的那一版"
+    );
+}
+
+/// a1 形：(1, 5) 的树表两份清零。
+#[test]
+fn the_tree_table_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
+) {
+    assert_a_unit_of_the_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
+        "known-bad-disjointness-tree-table-of-a-version-applied-only-by-its-record-zeroed",
+        |built| built.tree_table_slot_of_the_version_applied_only_by_its_record,
+        &["I-2.1", "I-3.1", "I-4.8", "I-7.4"],
+        "树表指针非零而树表读不出的版本，认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：由记录施加出来的那一版（实例 1、txg 5）",
+    );
+}
+
+/// a2 形：(1, 5) 的分配记录树根两份清零。
+#[test]
+fn the_allocation_record_tree_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
+) {
+    assert_a_unit_of_the_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged(
+        "known-bad-disjointness-allocation-root-of-a-version-applied-only-by-its-record-zeroed",
+        |built| {
+            assert!(
+                built
+                    .version_applied_only_by_its_record
+                    .rewritten
+                    .contains(&TransactionUnit::AllocationTree),
+                "(1, 5) 重写了自己的分配记录树根，不与别的版本共用"
+            );
+            built
+                .version_applied_only_by_its_record
+                .unit(TransactionUnit::AllocationTree)
+                .slot
+                .0
+        },
+        &["I-2.1", "I-4.8", "I-7.4"],
+        "带树一版根记录里分配记录树根全零那一格成立而没计（那一版自己的分配记录树读不出、走不到叶，别的版本的树走到了叶不替它作保）：由记录施加出来的那一版（实例 1、txg 5）",
+    );
+}
+
 /// 根记录里树 ID 水位那 8 字节的偏移（D22（单元原子性怎么合成） 已定项 7 的偏移表：树 ID 水位 122、自证校验和 138）。
 const ROOT_RECORD_TREE_IDENTIFIER_WATERMARK_OFFSET: usize = 122;
 
@@ -8917,6 +9221,142 @@ fn the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees
     );
 }
 
+/// 写行那一版之上发了第一个文件版本的镜像（[`image_after_the_first_file_on_the_version_whose_rows_were_written`]）上，
+/// 把 checkpoint_txg 为 `txg` 的那条根的树表指针整条写零、重封根槽（每条根照抄进它那几条记录的新根段，I-8.10 照旧成立）。
+/// 第一版没有一处写者写得出这种根（core 的根记录只从 mkfs 的树表、上一版照抄或这次发布写出的树表取这条指针）。
+/// 返回镜像与根记录那一项指着的分配记录树根所在的槽。
+fn image_with_the_tree_table_pointer_of_a_root_written_all_zero(
+    tag: &str,
+    txg: CheckpointTxg,
+) -> (MemoryPool, u64) {
+    let (mut image, root_record_allocation_tree_root_slot) =
+        image_after_the_first_file_on_the_version_whose_rows_were_written(tag);
+    rewrite_the_root_slot_of(&mut image, txg, |bytes| {
+        bytes[ROOT_RECORD_TREE_TABLE_POINTER_RANGE].fill(0);
+    });
+    (image, root_record_allocation_tree_root_slot)
+}
+
+/// 根记录里树表指针那 86 字节（D22（单元原子性怎么合成） 已定项 7 的偏移表：树表指针 36 起、树 ID 水位 122）。
+const ROOT_RECORD_TREE_TABLE_POINTER_RANGE: std::ops::Range<usize> = 36..122;
+
+/// 树表指针全零的根（用户 2026-10-05 定甲，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第五节第 1 条，
+/// 攻方报告 `research/prompts/m3-formal-code-b33-r1-opus-output.md` P2-b 的 y4 形）：最新那条根（第一个文件版本，实例 2、txg 5）的
+/// 树表指针整条写零。core 的根记录读者照它读盘 0 槽 0、读不出，可写挂载拒成 `Recovery(UnitUnreadable { slot: SlotNumber(0) })`；
+/// 改之前池级 checker 把它读成「这条根没有树表」，一条都不红、I-5.4 报成立。改之后照 core 跟过去：盘 0 槽 0 那两份对不上（I-2.1），
+/// 最新的根走不完（I-7.2），从它出发的遍历读不出（I-4.8、I-7.4），I-7.2 的说明写明是树表指针全零。
+#[test]
+fn an_all_zero_tree_table_pointer_in_the_newest_root_is_followed_like_the_core_and_the_newest_root_does_not_walk_through(
+) {
+    let (image, _) = image_with_the_tree_table_pointer_of_a_root_written_all_zero(
+        "known-bad-all-zero-tree-table-pointer-in-the-newest-root",
+        CheckpointTxg(5),
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-2.1", "I-4.8", "I-7.2", "I-7.4"],
+        "最新那条根的树表指针全零：照 core 跟过去读不出，不再一条都不红：{verdicts:?}"
+    );
+    let InvariantVerdict::Violated(detail) = verdict(&image, "I-7.2") else {
+        panic!("上面判过 I-7.2 红");
+    };
+    assert!(
+        detail.contains("根（实例 2、txg 5）的树表（根记录里树表指针全零"),
+        "I-7.2 写明走不完的是最新那条根、因为树表指针全零：{detail}"
+    );
+}
+
+/// 同上（攻方报告 P2-b 的 y1 形）：写行之后暖机那条根（实例 2、txg 4，树表 0 条、根记录直接持有分配记录树）的树表指针整条写零。
+/// 改之前池级 checker 把它读成「这条根没有树表」、一条都不红；改之后照 core 跟过去：
+/// ① 从这条候选根出发的遍历读不出（I-2.1、I-4.8、I-7.4 红），最新的根照样走得完（I-7.2 不红）；
+/// ② 只读的几处判法把它与树表读不出同一处置、点名这一版：I-5.4、I-5.5 与 I-9.28 报部分没判，删除态与墓碑谱系两遍报射程里判不了。
+#[test]
+fn an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged(
+) {
+    let (image, _) = image_with_the_tree_table_pointer_of_a_root_written_all_zero(
+        "known-bad-all-zero-tree-table-pointer-in-an-older-candidate-root",
+        CheckpointTxg(4),
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        ["I-2.1", "I-4.8", "I-7.4"],
+        "暖机那条根的树表指针全零：从它出发的遍历读不出，最新的根照样走得完：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict(&image, "I-5.4"),
+        InvariantVerdict::PartiallyJudged(
+            "树表指针全零的版本（core 照它读盘 0 槽 0、读不出），认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：根（实例 2、txg 4）"
+                .to_owned()
+        ),
+        "I-5.4 不再当这一版没有树表：与树表读不出同一处置、点名它"
+    );
+    let InvariantVerdict::PartiallyJudged(first_data_unit_unjudged) = verdict(
+        &image,
+        FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
+    ) else {
+        panic!("I-5.5 报部分没判：{verdicts:?}");
+    };
+    assert!(
+        first_data_unit_unjudged.contains(
+            "根（实例 2、txg 4）：树表指针全零（core 照它读盘 0 槽 0、读不出），认不出它是不是从 mkfs 水位第一次建树的那一版"
+        ),
+        "I-5.5 点名树表指针全零的那一版：{first_data_unit_unjudged}"
+    );
+    let InvariantVerdict::PartiallyJudged(tree_identifier_unjudged) = verdict(
+        &image,
+        TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
+    ) else {
+        panic!("I-9.28 报部分没判：{verdicts:?}");
+    };
+    assert!(
+        tree_identifier_unjudged.contains(
+            "（实例 2、txg 4）：树表指针全零（core 的根记录读者照它读盘 0 槽 0、读不出）"
+        ),
+        "I-9.28 点名树表指针全零的那条根：{tree_identifier_unjudged}"
+    );
+    let deletion_scan_gap = InvariantVerdict::NotApplicable(
+        "走读到的某一版树表指针全零（core 照它读盘 0 槽 0、读不出）：那一版的删除态射程里判不了",
+    );
+    assert_eq!(
+        verdict(&image, DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY),
+        deletion_scan_gap,
+        "删除态扫描报射程里判不了，不再当这一版没有队列"
+    );
+    assert_eq!(
+        verdict(&image, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE),
+        deletion_scan_gap,
+        "墓碑与谱系那一遍报射程里判不了，不再当这一版没有这几棵树"
+    );
+}
+
+/// 同上（攻方报告 P2-b 的 y2 形）：y1 那张镜像上再把写行那两版（txg 3、4）根记录那一项共用的分配记录树根两份清零。
+/// 改之前 I-5.4 的部分没判只点名 txg 3、不点名 txg 4（txg 4 被读成没有树表、不进「树表 0 条」那一支）；
+/// 改之后两版都点名：txg 4 在「树表指针全零」那一句里，txg 3 在「根记录直接持有的那棵树读不出」那一句里。
+#[test]
+fn an_all_zero_tree_table_pointer_beside_an_unreadable_root_record_allocation_tree_names_both_versions_in_the_disjointness_invariant(
+) {
+    let (mut image, root_record_allocation_tree_root_slot) =
+        image_with_the_tree_table_pointer_of_a_root_written_all_zero(
+            "known-bad-all-zero-tree-table-pointer-beside-an-unreadable-root-record-allocation-tree",
+            CheckpointTxg(4),
+        );
+    zero_both_copies_of_the_unit(
+        &mut image,
+        root_record_allocation_tree_root_slot,
+        usize::try_from(NODE_BYTES).expect("16384"),
+    );
+    assert_eq!(
+        verdict(&image, "I-5.4"),
+        InvariantVerdict::PartiallyJudged(
+            "树表指针全零的版本（core 照它读盘 0 槽 0、读不出），认不出带不带树：根记录里分配记录树根全零那一格与它自己的分配记录树都没判（别的版本的树走到了叶不替它作保）：根（实例 2、txg 4）；树表 0 条、分配记录树由根记录直接持有的版本，那棵树读不出、走不到叶，它自己的分配记录一条都没判（别的版本的树走到了叶不替它作保）：根（实例 2、txg 3）"
+                .to_owned()
+        ),
+        "写行那两版都点名：txg 4 树表指针全零，txg 3 根记录直接持有的那棵树读不出"
+    );
+}
+
 /// 新池新建文件那一版里一条走读跟随的指针头部 `change` 那一段改成非第一版的取值、重封、校验和沿引用链补到根槽：池级 checker 只红 I-2.4，
 /// 违例那一句是 `expected_violation`（点名是哪条指针、哪一段）。core 读这条指针时拒（判这条指针所在的结构损坏）；改之前池级 checker 只判
 /// MAC / nonce 与 extent 偏移，这三段改了一条都不红（第 30 批攻方 `B30ALG … checker_violated=[]` 那几行，判决
diff --git a/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs b/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
index b2f2226..c1437d0 100644
--- a/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
+++ b/crates/singlefs-harness/tests/reuse_after_raising_the_floor.rs
@@ -276,14 +276,17 @@ fn raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slo
         if *invariant == "I-5.5" {
             // E 之后的候选集（F_生效 = 11）是 txg 11–17 七条根，树表条目的诞生 txg 都是 3、不等于各自的 txg，认法 ② 不成立：
             // 候选集里没有从 mkfs 水位第一次建树的那一版，I-5.5（从 mkfs 水位建树那一版的 t1 落点） 一版都不判、报不适用；
-            // 满足 ①② 的只有 txg 3 那一版，它在 F 之下。只认这一种理由，别的不适用照红（同一机理的先例与判别力在
+            // 满足 ①② 的只有 txg 3 那一版，它在 F 之下。只认池级 checker 给这一形写的专门理由，不认 `Judgements::into_report` 的缺省理由
+            // （那一句与「I-5.5 的判法一次都没跑」分不开，判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P3
+            // 「四支豁免」那一行）；别的不适用照红（同一机理的先例与判别力在
             // `pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`）。
             assert!(
                 matches!(
                     verdict,
-                    InvariantVerdict::NotApplicable(reason) if *reason == "这个镜像上没有它判的对象"
+                    InvariantVerdict::NotApplicable(reason)
+                        if *reason == singlefs_checker::walk::NO_VERSION_BUILDS_TREES_FROM_THE_MKFS_WATERMARK
                 ),
-                "{invariant} 在 E 之后的镜像上报「这个镜像上没有它判的对象」的不适用：{verdict:?}"
+                "{invariant} 在 E 之后的镜像上报「候选集里没有从 mkfs 水位第一次建树的那一版」的专门不适用理由：{verdict:?}"
             );
             continue;
         }
````

### mutations-append.tsv

````
第34批 34-1 RV：I-5.4 按版本那一层不收由记录施加出来的那几版（a3：(1, 5) 的叶里两条记录罩住同一个槽照旧 0 违例）	crates/singlefs-checker/src/walk.rs	            .chain(versions_applied_only_by_records.iter().map(|version| {\n                VersionToJudgeTheAllocationRecordsOf {	            .chain(versions_applied_only_by_records.iter().take(0).map(|version| {\n                VersionToJudgeTheAllocationRecordsOf {	-p singlefs-harness --test checker_known_bad_images -- two_allocation_records_covering_one_slot_in_a_version_applied_only_by_its_journal_record_redden_only_the_disjointness_invariant	two_allocation_records_covering_one_slot_in_a_version_applied_only_by_its_journal_record_redden_only_the_disjointness_invariant
第34批 34-1 RV：I-5.4 按版本那一层不收由记录施加出来的那几版（a1：(1, 5) 的树表读不出照旧报成立）	crates/singlefs-checker/src/walk.rs	            .chain(versions_applied_only_by_records.iter().map(|version| {\n                VersionToJudgeTheAllocationRecordsOf {	            .chain(versions_applied_only_by_records.iter().take(0).map(|version| {\n                VersionToJudgeTheAllocationRecordsOf {	-p singlefs-harness --test checker_known_bad_images -- the_tree_table_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged	the_tree_table_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged
第34批 34-1 RV：I-5.4 按版本那一层不收由记录施加出来的那几版（a2：(1, 5) 的分配记录树读不出照旧报成立）	crates/singlefs-checker/src/walk.rs	            .chain(versions_applied_only_by_records.iter().map(|version| {\n                VersionToJudgeTheAllocationRecordsOf {	            .chain(versions_applied_only_by_records.iter().take(0).map(|version| {\n                VersionToJudgeTheAllocationRecordsOf {	-p singlefs-harness --test checker_known_bad_images -- the_allocation_record_tree_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged	the_allocation_record_tree_of_a_version_applied_only_by_its_journal_record_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged
第34批 34-1 RV：I-5.4 按版本那一层不收下一次挂载先施加的那一版（b3：两条记录罩住同一个槽、挂载会拒的那一版照旧 0 违例）	crates/singlefs-checker/src/walk.rs	            .chain(version_the_next_mount_applies_first.iter().map(|version| {	            .chain(version_the_next_mount_applies_first.iter().take(0).map(|version| {	-p singlefs-harness --test checker_known_bad_images -- two_allocation_records_covering_one_slot_in_the_version_the_next_mount_applies_first_redden_only_the_disjointness_invariant	two_allocation_records_covering_one_slot_in_the_version_the_next_mount_applies_first_redden_only_the_disjointness_invariant
第34批 34-1 全零树表指针：主走读照旧把根记录里全零的树表指针读成没有树表（y4：最新那条根树表指针全零照旧 0 违例）	crates/singlefs-checker/src/walk.rs	            &version_name,\n            AllZeroTreeTablePointerReading::FollowedLikeTheRootRecordReaderOfTheCore,\n        );	            &version_name,\n            AllZeroTreeTablePointerReading::NoTreeTable,\n        );	-p singlefs-harness --test checker_known_bad_images -- an_all_zero_tree_table_pointer_in_the_newest_root_is_followed_like_the_core_and_the_newest_root_does_not_walk_through	an_all_zero_tree_table_pointer_in_the_newest_root_is_followed_like_the_core_and_the_newest_root_does_not_walk_through
第34批 34-1 全零树表指针：I-5.4 不记树表指针全零的版本（y1：那一版不点名、I-5.4 报成立）	crates/singlefs-checker/src/walk.rs	        if tree_table_pointer.all_zero {\n            versions_whose_tree_table_pointer_is_all_zero.push(version.name.clone());\n        } else {	        if tree_table_pointer.all_zero {\n        } else {	-p singlefs-harness --test checker_known_bad_images -- an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged	an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged
第34批 34-1 全零树表指针：I-5.4 不记树表指针全零的版本（y2：只点名 txg 3、不点名 txg 4）	crates/singlefs-checker/src/walk.rs	        if tree_table_pointer.all_zero {\n            versions_whose_tree_table_pointer_is_all_zero.push(version.name.clone());\n        } else {	        if tree_table_pointer.all_zero {\n        } else {	-p singlefs-harness --test checker_known_bad_images -- an_all_zero_tree_table_pointer_beside_an_unreadable_root_record_allocation_tree_names_both_versions_in_the_disjointness_invariant	an_all_zero_tree_table_pointer_beside_an_unreadable_root_record_allocation_tree_names_both_versions_in_the_disjointness_invariant
第34批 34-1 全零树表指针：I-5.5 照旧跳过树表指针全零的版本、不记没判（y1：I-5.5 不报部分没判）	crates/singlefs-checker/src/walk.rs	        if tree_table_pointer.all_zero {\n            // 用户 2026-10-05 定甲：core 照全零的树表指针读盘 0 槽 0、读不出，与树表读不出同一处置（此前当这一版没有树表、不判也不记）。\n            versions_not_judged.push(format!(	        if tree_table_pointer.all_zero {\n            continue;\n            versions_not_judged.push(format!(	-p singlefs-harness --test checker_known_bad_images -- an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged	an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged
第34批 34-1 全零树表指针：I-9.28 照旧把树表指针全零的根算没有树表（y1：I-9.28 不报部分没判）	crates/singlefs-checker/src/walk.rs	                Some("树表指针全零（core 的根记录读者照它读盘 0 槽 0、读不出）")	                None	-p singlefs-harness --test checker_known_bad_images -- an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged	an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged
第34批 34-1 全零树表指针：删除态扫描照旧跳过树表指针全零的版本（y1：I-9.24 不报射程里判不了）	crates/singlefs-checker/src/walk.rs	        if tree_table_pointer.all_zero {\n            for invariant in DELETION_QUEUE_SCAN_INVARIANTS	        if tree_table_pointer.all_zero {\n            continue;\n            for invariant in DELETION_QUEUE_SCAN_INVARIANTS	-p singlefs-harness --test checker_known_bad_images -- an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged	an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged
第34批 34-1 全零树表指针：墓碑与谱系那一遍照旧跳过树表指针全零的版本（y1：I-9.18 不报射程里判不了）	crates/singlefs-checker/src/walk.rs	        if tree_table_pointer.all_zero {\n            for invariant in TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS	        if tree_table_pointer.all_zero {\n            continue;\n            for invariant in TOMBSTONE_AND_LINEAGE_SCAN_INVARIANTS	-p singlefs-harness --test checker_known_bad_images -- an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged	an_all_zero_tree_table_pointer_in_an_older_candidate_root_is_followed_like_the_core_and_every_tree_table_reading_names_it_unjudged
第34批 34-1 I-5.5 专门理由：I-5.5 的判法提前返回、一版都不判（抬 F 慢档用例只认专门理由，不认缺省理由）	crates/singlefs-checker/src/walk.rs	    let invariant = FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK;\n    let devices = reader.devices();	    return;\n    let invariant = FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK;\n    let devices = reader.devices();	-p singlefs-harness --test reuse_after_raising_the_floor -- --include-ignored --exact raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it	raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it
第34批 34-1 GPU 判器 RV：I-5.4 不收下一次挂载先施加的那一版（新池新建文件那次发布的分配记录叶里两条记录罩住同一个槽，根槽没落的状态上 CPU 红、GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    if (next_version != NONE) {\n        let r = checker_record_at(next_version);	    if (next_version != NONE && false) {\n        let r = checker_record_at(next_version);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first	the_gpu_judge_agrees_with_the_cpu_when_two_allocation_records_cover_one_slot_in_the_version_the_next_mount_applies_first
````

## m3-formal-34-2

34-2：core 根记录读者四条指针与 journal 新根段同一判法（C581 余项）；`pointer.rs` 只判 extent 偏移那一支删掉

### crates.patch

````diff
diff --git a/crates/singlefs-core/src/pointer.rs b/crates/singlefs-core/src/pointer.rs
index a255b77..06ef2c4 100644
--- a/crates/singlefs-core/src/pointer.rs
+++ b/crates/singlefs-core/src/pointer.rs
@@ -174,9 +174,8 @@ pub struct PointerHeadExtentOffsetNotZero {
     pub extent_offset: u16,
 }
 
-/// 指针头部按字段表读出来的样子，加密与压缩那几段与 extent 偏移还没判：[`PointerHead::read_from`]、
-/// [`PointerHead::read_judging_the_encryption_compression_and_extent_offset_fields_from`] 与 [`PointerHead::read_judging_the_extent_offset_from`]
-/// 共用这一份读法，字段表的偏移只写一处。
+/// 指针头部按字段表读出来的样子，加密与压缩那几段与 extent 偏移还没判：[`PointerHead::read_from`] 与
+/// [`PointerHead::read_judging_the_encryption_compression_and_extent_offset_fields_from`] 共用这一份读法，字段表的偏移只写一处。
 struct PointerHeadAsRead {
     is_every_mac_and_nonce_byte_zero: bool,
     algorithm_type: u8,
@@ -232,18 +231,12 @@ impl PointerHeadAsRead {
                 },
             );
         }
-        self.judged_extent_offset()
-            .map_err(PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero)
-    }
-
-    /// 只判 extent 偏移（C581）：今天只剩根记录（`root_record.rs`）走这一判，加密与压缩那几段在它上面还没换成判；树表条目、inode 内部条目、
-    /// extent 树的上段叶条目、内部条目与叶记录、分配记录树内部条目、实例表链指针已换成 [`Self::judged`]（判决
-    /// `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 5 行、`records/2026-09-30-里程碑三开工第一轮.md` 第 8 批 core 簇第 9 条）。
-    fn judged_extent_offset(self) -> Result<PointerHead, PointerHeadExtentOffsetNotZero> {
         if self.extent_offset != 0 {
-            return Err(PointerHeadExtentOffsetNotZero {
-                extent_offset: self.extent_offset,
-            });
+            return Err(PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(
+                PointerHeadExtentOffsetNotZero {
+                    extent_offset: self.extent_offset,
+                },
+            ));
         }
         Ok(self.head)
     }
@@ -266,8 +259,8 @@ impl PointerHead {
         );
     }
     /// 读头部，加密与压缩那几段与 extent 偏移都不判。core 读盘上指针的读者一律不用它（走
-    /// [`Self::read_judging_the_extent_offset_from`] 或 [`Self::read_judging_the_encryption_compression_and_extent_offset_fields_from`]）；
-    /// 留给测试与造坏盘的装置读自己刚写出、要改的那几个字节。
+    /// [`Self::read_judging_the_encryption_compression_and_extent_offset_fields_from`]；影子账的读者经
+    /// [`NodePointer::read_with_head_fields_judgement_from`] 不判）；留给测试与造坏盘的装置读自己刚写出、要改的那几个字节。
     pub fn read_from(reader: &mut ByteReader<'_>) -> Self {
         PointerHeadAsRead::read_from(reader).head
     }
@@ -281,16 +274,6 @@ impl PointerHead {
     ) -> Result<Self, PointerHeadFieldOutsideTheFirstVersion> {
         PointerHeadAsRead::read_from(reader).judged()
     }
-
-    /// 读头部，只判 extent 偏移（加密与压缩那几段不判）。
-    ///
-    /// # Errors
-    /// extent 偏移非 0 ⇒ [`PointerHeadExtentOffsetNotZero`]。
-    pub fn read_judging_the_extent_offset_from(
-        reader: &mut ByteReader<'_>,
-    ) -> Result<Self, PointerHeadExtentOffsetNotZero> {
-        PointerHeadAsRead::read_from(reader).judged_extent_offset()
-    }
 }
 
 /// 指向码 2 / 码 3 的指针 86。
@@ -370,17 +353,6 @@ impl NodePointer {
         Ok(Self::read_after_the_head(head, reader))
     }
 
-    /// 同 [`Self::read_from`]，头部里只判 extent 偏移（[`PointerHead::read_judging_the_extent_offset_from`]）。
-    ///
-    /// # Errors
-    /// extent 偏移非 0 ⇒ [`PointerHeadExtentOffsetNotZero`]。
-    pub fn read_judging_the_extent_offset_from(
-        reader: &mut ByteReader<'_>,
-    ) -> Result<Self, PointerHeadExtentOffsetNotZero> {
-        let head = PointerHead::read_judging_the_extent_offset_from(reader)?;
-        Ok(Self::read_after_the_head(head, reader))
-    }
-
     /// 读一条子指针，判不判头部第一版之外的取值由读者定（[`PointerHeadFieldsJudgement`]）：走读的读者判，影子账的读者不判。
     ///
     /// # Errors
@@ -456,17 +428,6 @@ impl DataPointer {
         Ok(Self::read_after_the_head(head, reader))
     }
 
-    /// 同 [`Self::read_from`]，头部里只判 extent 偏移（[`PointerHead::read_judging_the_extent_offset_from`]）。
-    ///
-    /// # Errors
-    /// extent 偏移非 0 ⇒ [`PointerHeadExtentOffsetNotZero`]。
-    pub fn read_judging_the_extent_offset_from(
-        reader: &mut ByteReader<'_>,
-    ) -> Result<Self, PointerHeadExtentOffsetNotZero> {
-        let head = PointerHead::read_judging_the_extent_offset_from(reader)?;
-        Ok(Self::read_after_the_head(head, reader))
-    }
-
     fn read_after_the_head(head: PointerHead, reader: &mut ByteReader<'_>) -> Self {
         let locations = [
             LocationEntry::read_from(reader),
diff --git a/crates/singlefs-core/src/root_record.rs b/crates/singlefs-core/src/root_record.rs
index 27244fb..11adda7 100644
--- a/crates/singlefs-core/src/root_record.rs
+++ b/crates/singlefs-core/src/root_record.rs
@@ -117,10 +117,13 @@ impl RootRecord {
         bytes
     }
 
-    /// 读者：magic、整槽校验和、fsid、flags、四条指针的 extent 偏移五关。flags 只收卸载记号那一位（[`ROOT_RECORD_FLAG_UNMOUNT_MARKER`]），
+    /// 读者：magic、整槽校验和、fsid、flags、四条指针头部五关。flags 只收卸载记号那一位（[`ROOT_RECORD_FLAG_UNMOUNT_MARKER`]），
     /// 别的位非 0 拒收；记号照实读进 [`RootRecord::unmount_marker`]（挂载与恢复不按它分支，按它分支的是池级 checker 的 I-7.9）。
-    /// 树表、实例表、中央映射树、分配记录树四条指针头部的 extent 偏移第一版恒 0，任一非 0 判这条根记录损坏、拒收
-    /// （C581（extent 偏移非 0 没有判据），`pointer::PointerHeadExtentOffsetNotZero`）：拒收的根与自证不过的根同一个处置，择根时不可择。
+    /// 树表、实例表、中央映射树、分配记录树四条指针头部里加密、压缩与 extent 偏移那几段按第一版的取值判，与 journal 新根段那四条指针
+    /// 同一判法（`NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from`）：MAC / nonce 非 0、算法类型不是未加密、
+    /// 压缩算法码不是 0、码 0 而压后长度非 0、extent 偏移非 0，任一条判这条根记录损坏、拒收（I-2.4（头校验和覆盖范围）「加密关着时恒 0、
+    /// 读者遇到非 0 一律判该结构损坏」，D19（块指针的结构与宽度预算） 已定项 3，C581（extent 偏移非 0 没有判据），
+    /// `pointer::PointerHeadFieldOutsideTheFirstVersion`）：拒收的根与自证不过的根同一个处置，择根时不可择。
     #[must_use]
     pub fn parse_slot(bytes: &[u8], expected_filesystem_identifier: &[u8; 16]) -> Option<Self> {
         if bytes.len() < usize::try_from(ROOT_RECORD_BYTES).expect("457")
@@ -143,14 +146,29 @@ impl RootRecord {
         let unmount_marker = UnmountMarker::of_flags_bits_carrying_only_the_marker(flags);
         let instance = InstanceGeneration(reader.get_u32());
         let checkpoint_txg = CheckpointTxg(reader.get_u64());
-        let tree_table = NodePointer::read_judging_the_extent_offset_from(&mut reader).ok()?;
+        let tree_table =
+            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+                &mut reader,
+            )
+            .ok()?;
         let tree_identifier_watermark = reader.get_u64();
         let rollback_floor = CheckpointTxg(reader.get_u64());
         reader.skip(usize::try_from(WIDE_CHECKSUM_BYTES).expect("32"));
-        let instance_table = NodePointer::read_judging_the_extent_offset_from(&mut reader).ok()?;
-        let mapping_root = NodePointer::read_judging_the_extent_offset_from(&mut reader).ok()?;
+        let instance_table =
+            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+                &mut reader,
+            )
+            .ok()?;
+        let mapping_root =
+            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+                &mut reader,
+            )
+            .ok()?;
         let allocation_record_tree_root =
-            NodePointer::read_judging_the_extent_offset_from(&mut reader).ok()?;
+            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+                &mut reader,
+            )
+            .ok()?;
         assert_eq!(
             reader.position(),
             ROOT_CHECKSUM_OFFSET + 32 + 3 * usize::try_from(NODE_POINTER_BYTES).expect("86")
diff --git a/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
index c01b59f..e548e57 100644
--- a/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
+++ b/crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs
@@ -1763,6 +1763,23 @@ fn flip_one_byte_of(bytes: &mut [u8], range: std::ops::Range<usize>) {
     }
 }
 
+/// 指针头部里出生 txg 的第一个字节：MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2、extent 偏移 2、出生树 8 之后
+/// （D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11 的字段表）。
+const BIRTH_TXG_IN_THE_POINTER_HEAD: usize = 16 + 12 + 1 + 1 + 2 + 2 + 8;
+
+/// 根记录里一段翻一字节：同 [`flip_one_byte_of`]，只是整段全 0 的指针段（中央映射树根、分配记录树根在没有它们的那一版上）写在头部出生 txg 那一字节，
+/// 不写在段首：头部前 34 字节（MAC、nonce、算法类型、压缩那两段与 extent 偏移）非 0 时根记录读者判这条根损坏（I-2.4（头校验和覆盖范围），
+/// `RootRecord::parse_slot`），翻在段首这一槽重封之后自证不过、到不了判官。
+fn flip_one_byte_of_a_root_record_range(bytes: &mut [u8], range: std::ops::Range<usize>) {
+    let is_an_all_zero_pointer =
+        range.len() == NODE_POINTER_BYTES && bytes[range.clone()].iter().all(|byte| *byte == 0);
+    if is_an_all_zero_pointer {
+        bytes[range.start + BIRTH_TXG_IN_THE_POINTER_HEAD] = 1;
+    } else {
+        flip_one_byte_of(bytes, range);
+    }
+}
+
 /// R2 那一版的根记录翻一字节之后重封整槽自证校验和、照新字节重解一遍（解出来的那几样与原字节一起变）。
 fn flip_one_byte_of_the_root_record_and_reseal(
     version: &mut VersionFieldsRead,
@@ -1773,7 +1790,7 @@ fn flip_one_byte_of_the_root_record_and_reseal(
     let record_bytes = version.root_record_bytes.len();
     let mut slot = version.root_record_bytes.clone();
     slot.resize(root_slot_bytes, 0);
-    flip_one_byte_of(&mut slot, range);
+    flip_one_byte_of_a_root_record_range(&mut slot, range);
     let digest = wide_checksum_with_field_zeroed(&slot, root_slot_bytes, ROOT_CHECKSUM_OFFSET);
     slot[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
     let filesystem_identifier: [u8; 16] = slot[4..20].try_into().expect("fsid 16 字节");
diff --git a/crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_both_mounts.rs b/crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_both_mounts.rs
index 0170ab6..bb7b515 100644
--- a/crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_both_mounts.rs
+++ b/crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_both_mounts.rs
@@ -4,7 +4,8 @@
 //! 坏镜像照攻方那一份造（`research/prompts/m3-inv-a-r1-opus-output.md` 五、A6）：两块内存盘上 mkfs、取号、暖机、第一个文件（txg 3），
 //! 把根环里每一条自证过的根的实例表指针 extent 偏移改成 1、整槽自证校验和重封。改之前 `recover` 读出文件、`mount_writable` 挂上
 //! （读者 `reader.skip(2)`）；改之后两个入口都择不到根、拒，盘上逐字节不变。
-//! 另一条在指针一级钉住判的那两个读法（多层码 2 树内部条目、journal 新根段指针用的、根记录用的）报的是 extent 偏移。
+//! 另一条在指针一级钉住判头部的那两个读法（多层码 2 树内部条目、journal 新根段与根记录四条指针用的；走读的读者经判不判由读者定的那个入口用的）
+//! 报的是 extent 偏移。
 
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier,
@@ -19,7 +20,7 @@ use singlefs_core::make_filesystem::{
 use singlefs_core::mount::{mount_writable, MountError};
 use singlefs_core::pointer::{
     BirthSequence, LocationEntry, NodePointer, PointerHead, PointerHeadExtentOffsetNotZero,
-    PointerHeadFieldOutsideTheFirstVersion,
+    PointerHeadFieldOutsideTheFirstVersion, PointerHeadFieldsJudgement,
 };
 use singlefs_core::recovery::{
     choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, JournalPolicy,
@@ -233,36 +234,40 @@ fn node_pointer_bytes_with_extent_offset(extent_offset: u16) -> (NodePointer, Ve
 }
 
 /// 场景：一条节点指针头部 extent 偏移写成 7，MAC、nonce、算法类型、压缩那几段都是第一版的取值。
-/// 预期：判头部的两个读法都报 extent 偏移非 0、带着读到的 7——多层码 2 树内部条目与 journal 新根段指针用的
-/// `read_judging_the_encryption_compression_and_extent_offset_fields_from`，根记录用的 `read_judging_the_extent_offset_from`；
-/// extent 偏移是 0 的同一条指针两个读法都照常读出。改之前两个读法都跳过这 2 字节、照常读出。
+/// 预期：判头部的两个读法都报 extent 偏移非 0、带着读到的 7——多层码 2 树内部条目、journal 新根段与根记录四条指针用的
+/// `read_judging_the_encryption_compression_and_extent_offset_fields_from`，走读的读者用的 `read_with_head_fields_judgement_from`
+/// 取判的那一种；extent 偏移是 0 的同一条指针两个读法都照常读出。改之前两个读法都跳过这 2 字节、照常读出。
 #[test]
 fn a_node_pointer_head_with_a_non_zero_extent_offset_is_refused_by_both_judging_readers() {
+    let judged = PointerHeadFieldsJudgement::JudgedAsTheStructureIsCorruptOutsideTheFirstVersion;
     let (pointer, clean) = node_pointer_bytes_with_extent_offset(0);
     assert_eq!(
         (
             NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                 &mut ByteReader::at(&clean, 0)
             ),
-            NodePointer::read_judging_the_extent_offset_from(&mut ByteReader::at(&clean, 0)),
+            NodePointer::read_with_head_fields_judgement_from(
+                &mut ByteReader::at(&clean, 0),
+                judged
+            ),
         ),
         (Ok(pointer), Ok(pointer)),
         "extent 偏移 0：两个读法都照常读出"
     );
     let (_, damaged) = node_pointer_bytes_with_extent_offset(7);
-    let not_zero = PointerHeadExtentOffsetNotZero { extent_offset: 7 };
+    let not_zero = PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(
+        PointerHeadExtentOffsetNotZero { extent_offset: 7 },
+    );
     assert_eq!(
         NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
             &mut ByteReader::at(&damaged, 0)
         ),
-        Err(PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(
-            not_zero
-        )),
+        Err(not_zero),
         "判加密与压缩那几段的读法也判 extent 偏移"
     );
     assert_eq!(
-        NodePointer::read_judging_the_extent_offset_from(&mut ByteReader::at(&damaged, 0)),
+        NodePointer::read_with_head_fields_judgement_from(&mut ByteReader::at(&damaged, 0), judged),
         Err(not_zero),
-        "根记录用的读法报 extent 偏移"
+        "走读的读者取判的那一种：报 extent 偏移"
     );
 }
diff --git a/crates/singlefs-harness/tests/root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts.rs b/crates/singlefs-harness/tests/root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts.rs
new file mode 100644
index 0000000..b3e1d35
--- /dev/null
+++ b/crates/singlefs-harness/tests/root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts.rs
@@ -0,0 +1,343 @@
+//! 根记录里树表、实例表、中央映射树、分配记录树四条指针的头部里，加密与压缩那几段（MAC 16 / nonce 12、算法类型 1、
+//! 压缩算法码 1、压后长度 2）第一版恒 0，读者遇到不是第一版的取值判这条根记录损坏，与 journal 新根段那四条指针同一判法
+//! （I-2.4（头校验和覆盖范围）「加密关着时恒 0、读者遇到非 0 一律判该结构损坏」，D19（块指针的结构与宽度预算） 已定项 3，
+//! D9（加密） 已定项 14；判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P2「F2 与 core」那一格）。
+//! 判不过的根与 extent 偏移非 0 的根同一个处置：自证不过、择根时不可择（`RootRecord::parse_slot`）。
+//!
+//! 坏镜像照攻方 f1–f3 那一份造（`research/prompts/m3-formal-code-b33-r1-opus-output.md` P2-c）：最新那条根的根槽里改一条指针头部的
+//! 一个字段、整槽自证校验和重封。journal 里那次发布的记录不照字节抄：抄过去的话 journal 读者（新根段四条指针早已判全）也拒那条记录，
+//! 两个读者的判混在一个结局里，分不出根记录读者判没判；这里只改根槽，记录照旧完好，可择的根退到更早那条、记录把它施加回同一版。
+//! 四条用例四条指针各改一个字段，四类字段（算法类型、压缩算法码、压后长度、MAC / nonce）各出现一次。
+//! 改之前根记录读者对这几段只判 extent 偏移：这条根照样自证过、被择，记录一条不施加（攻方 f1–f3 的 `chosen_txg=4 records_applied=0`）。
+
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use singlefs_core::allocator::PoolAllocator;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::bytes::ByteReader;
+use singlefs_core::checksum::wide_checksum_with_field_zeroed;
+use singlefs_core::make_filesystem::{
+    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
+};
+use singlefs_core::mount::mount_writable;
+use singlefs_core::pointer::NodePointer;
+use singlefs_core::recovery::{
+    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover,
+    BadRootRingSlotReading, JournalPolicy, RecoveryOutcome, RootRingSlotReading,
+};
+use singlefs_core::root_record::{RootRecord, ROOT_CHECKSUM_OFFSET};
+use singlefs_core::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegion};
+use singlefs_core::system_configuration::SystemImmutableSizes;
+use singlefs_core::transaction::{
+    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter,
+};
+use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, NODE_POINTER_BYTES};
+use singlefs_harness::memory_pool::SparseBlockDevice;
+
+const DEVICE_BYTES: u64 = 4 << 30;
+const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-m3-rrph";
+const WRITE_TIME_SECONDS: u64 = 1_788_300_000;
+/// 内存稀疏盘按 512 字节扇区存：物理块宽 512，根槽宽也是 512。
+const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes = PhysicalBlockSizeInBytes(512);
+/// 第一个文件那次发布写的根的 txg（mkfs 第 0 代、暖机 txg 1、2 之后）。
+const FIRST_FILE_TXG: CheckpointTxg = CheckpointTxg(3);
+/// 暖机最后那条根的 txg：第一个文件那条根不可择时退到它。
+const LAST_WARM_UP_TXG: CheckpointTxg = CheckpointTxg(2);
+
+/// 指针头部里各字段的偏移（D19（块指针的结构与宽度预算） 已定项 7 / 已定项 11 的字段表）：MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2。
+const MAC_IN_THE_POINTER_HEAD: usize = 0;
+const ALGORITHM_TYPE_IN_THE_POINTER_HEAD: usize = 16 + 12;
+const COMPRESSION_ALGORITHM_CODE_IN_THE_POINTER_HEAD: usize = 16 + 12 + 1;
+const COMPRESSED_LENGTH_IN_THE_POINTER_HEAD: usize = 16 + 12 + 1 + 1;
+
+/// 根槽里四条指针的偏移（`RootRecord::to_slot` 的写法；用例里拿解出来的根核一遍）：树表指针紧跟 magic 4、fsid 16、flags 4、
+/// 实例代号 4、checkpoint_txg 8；它之后是树 ID 水位 8、回退下界 F 8、自证校验和 32，再接实例表、映射根、分配记录树根三条。
+const TREE_TABLE_POINTER_IN_THE_ROOT_SLOT: usize = 4 + 16 + 4 + 4 + 8;
+const INSTANCE_TABLE_POINTER_IN_THE_ROOT_SLOT: usize = ROOT_CHECKSUM_OFFSET + 32;
+const MAPPING_ROOT_POINTER_IN_THE_ROOT_SLOT: usize =
+    INSTANCE_TABLE_POINTER_IN_THE_ROOT_SLOT + NODE_POINTER_BYTES_AS_SLOT_OFFSET;
+const ALLOCATION_RECORD_TREE_ROOT_POINTER_IN_THE_ROOT_SLOT: usize =
+    MAPPING_ROOT_POINTER_IN_THE_ROOT_SLOT + NODE_POINTER_BYTES_AS_SLOT_OFFSET;
+/// 一条节点指针的宽度，当根槽里的偏移用：与 `singlefs_format::NODE_POINTER_BYTES` 同一个数（常量里用不了 `try_from`；改槽之前核一遍）。
+const NODE_POINTER_BYTES_AS_SLOT_OFFSET: usize = 86;
+
+type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;
+
+/// 根记录里的哪一条指针。封闭集合，`match` 不写通配臂。
+#[derive(Clone, Copy, Debug)]
+enum RootRecordPointer {
+    TreeTable,
+    InstanceTable,
+    MappingRoot,
+    AllocationRecordTreeRoot,
+}
+
+impl RootRecordPointer {
+    fn offset_in_the_root_slot(self) -> usize {
+        match self {
+            RootRecordPointer::TreeTable => TREE_TABLE_POINTER_IN_THE_ROOT_SLOT,
+            RootRecordPointer::InstanceTable => INSTANCE_TABLE_POINTER_IN_THE_ROOT_SLOT,
+            RootRecordPointer::MappingRoot => MAPPING_ROOT_POINTER_IN_THE_ROOT_SLOT,
+            RootRecordPointer::AllocationRecordTreeRoot => {
+                ALLOCATION_RECORD_TREE_ROOT_POINTER_IN_THE_ROOT_SLOT
+            }
+        }
+    }
+
+    fn as_parsed_from(self, root: &RootRecord) -> NodePointer {
+        match self {
+            RootRecordPointer::TreeTable => root.tree_table,
+            RootRecordPointer::InstanceTable => root.instance_table,
+            RootRecordPointer::MappingRoot => root.mapping_root,
+            RootRecordPointer::AllocationRecordTreeRoot => root.allocation_record_tree_root,
+        }
+    }
+}
+
+fn parameters() -> MakeFilesystemParameters {
+    MakeFilesystemParameters {
+        filesystem_identifier: FILESYSTEM_IDENTIFIER,
+        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
+        geometry: SystemImmutableSizes {
+            physical_block_size: 512,
+            minimum_input_output_bytes: 512,
+            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
+            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
+            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
+        },
+    }
+}
+
+fn first_file_content() -> Vec<u8> {
+    (0..3000u32)
+        .map(|index| u8::try_from(index % 251).expect("小于 256"))
+        .collect()
+}
+
+/// 两块内存盘上 mkfs → 取号 1 → 暖机 → 第一个文件（txg 3）。
+fn pool_after_the_first_file() -> Devices {
+    let parameters = parameters();
+    let mut devices: Devices = [DeviceIdentity(0), DeviceIdentity(1)]
+        .into_iter()
+        .map(|identity| {
+            (
+                identity,
+                SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE),
+            )
+        })
+        .collect();
+    let genesis = make_filesystem(&parameters, &mut devices).expect("默认几何上 mkfs");
+    let mut allocator: PoolAllocator =
+        allocator_after_make_filesystem(&parameters, &devices, &genesis);
+    let content = first_file_content();
+    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+    let instance = acquire_instance(&mut writer).expect("取号 1");
+    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
+    publish_first_file(
+        &mut writer,
+        &mut allocator,
+        warmed.roots.last().expect("暖机写了两条根"),
+        FirstFile {
+            content: &content,
+            write_time_seconds: WRITE_TIME_SECONDS,
+        },
+        instance,
+        &warmed.last_record_bytes,
+    )
+    .expect("第一个文件");
+    devices
+}
+
+fn root_ring_slot_reading(devices: &Devices, ring_slot: RootRingSlot) -> RootRingSlotReading {
+    let system_configuration = choose_system_configuration(devices).expect("系统配置择得到");
+    read_root_ring_slot(
+        devices,
+        &system_configuration.immutable.region_devices,
+        &system_configuration.immutable.sizes,
+        &system_configuration.immutable.filesystem_identifier,
+        ring_slot,
+    )
+}
+
+/// 根环里自证过、txg 是 `checkpoint_txg` 的每一条根，连同它住的槽。
+fn self_verified_roots_of(
+    devices: &Devices,
+    checkpoint_txg: CheckpointTxg,
+) -> Vec<(RootRingSlot, RootRecord)> {
+    let system_configuration = choose_system_configuration(devices).expect("系统配置择得到");
+    every_root_ring_slot(&system_configuration.immutable.sizes)
+        .into_iter()
+        .filter_map(
+            |ring_slot| match root_ring_slot_reading(devices, ring_slot) {
+                RootRingSlotReading::SelfVerified(root)
+                    if root.checkpoint_txg == checkpoint_txg =>
+                {
+                    Some((ring_slot, root))
+                }
+                RootRingSlotReading::SelfVerified(_) | RootRingSlotReading::Bad(_) => None,
+            },
+        )
+        .collect()
+}
+
+/// 第一个文件那条根（txg 3）住的每一个根槽：`pointer` 那条指针头部偏移 `field_offset_in_the_pointer_head` 那一字节写成 `value`、
+/// 整槽自证校验和重封（攻方 f1–f3 的改法；journal 记录不动，理由在文件头）。交回改过的槽。
+fn rewrite_a_pointer_head_byte_in_the_first_file_root(
+    devices: &mut Devices,
+    pointer: RootRecordPointer,
+    field_offset_in_the_pointer_head: usize,
+    value: u8,
+) -> Vec<RootRingSlot> {
+    let parameters = parameters();
+    let root_slot_bytes =
+        usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽装得进 usize");
+    let roots = self_verified_roots_of(devices, FIRST_FILE_TXG);
+    for (ring_slot, root) in &roots {
+        let device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
+        let offset = slot_offset(*ring_slot, parameters.geometry.fixed_structure_slot_spacing);
+        let device_under_change = &mut devices
+            .iter_mut()
+            .find(|(identity, _)| *identity == device)
+            .expect("池里有这块盘")
+            .1;
+        let mut slot = device_under_change.image.read(offset, root_slot_bytes);
+        assert_eq!(
+            u64::try_from(NODE_POINTER_BYTES_AS_SLOT_OFFSET).expect("86"),
+            NODE_POINTER_BYTES,
+            "根槽偏移按节点指针宽度 {NODE_POINTER_BYTES} 累加"
+        );
+        let pointer_offset = pointer.offset_in_the_root_slot();
+        assert_eq!(
+            NodePointer::read_from(&mut ByteReader::at(&slot, pointer_offset)),
+            pointer.as_parsed_from(root),
+            "根槽偏移 {pointer_offset} 起的 {NODE_POINTER_BYTES} 字节就是 {pointer:?} 那条指针"
+        );
+        let byte_at = pointer_offset + field_offset_in_the_pointer_head;
+        assert_eq!(slot[byte_at], 0, "第一版写者在指针头部这一段写的是 0");
+        slot[byte_at] = value;
+        let digest = wide_checksum_with_field_zeroed(&slot, root_slot_bytes, ROOT_CHECKSUM_OFFSET);
+        slot[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
+        device_under_change.image.write(offset, &slot);
+    }
+    roots.into_iter().map(|(ring_slot, _)| ring_slot).collect()
+}
+
+/// 改过之后两个入口的结局：改过的每个槽读成自证不过；只读恢复择暖机最后那条根（txg 2）、施加第一个文件那条记录、读出文件；
+/// 可写挂载同样择 txg 2、施加之后走 txg 3。改之前这几样依次是：自证过、择 txg 3 一条记录不施加。
+fn assert_the_first_file_root_is_refused_and_both_mounts_fall_back_to_the_warm_up_root(
+    mut devices: Devices,
+    rewritten_slots: &[RootRingSlot],
+    what_was_rewritten: &str,
+) {
+    assert!(
+        !rewritten_slots.is_empty(),
+        "{what_was_rewritten}：根环里第一个文件那条根至少一份"
+    );
+    for ring_slot in rewritten_slots {
+        assert_eq!(
+            root_ring_slot_reading(&devices, *ring_slot),
+            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified),
+            "{what_was_rewritten}：槽 {ring_slot:?} 这条根判损坏、与自证不过同一个处置"
+        );
+    }
+    let recovered = recover(&devices, JournalPolicy::Consult);
+    assert_eq!(
+        recovered.outcome,
+        RecoveryOutcome::FileRead {
+            root: (InstanceGeneration(1), LAST_WARM_UP_TXG),
+            content: first_file_content(),
+        },
+        "{what_was_rewritten}：只读恢复择暖机最后那条根（txg 2），施加记录之后照样读出第一个文件"
+    );
+    assert_eq!(
+        (recovered.effective_root, recovered.journal.prefix_applied),
+        (Some((InstanceGeneration(1), FIRST_FILE_TXG)), 1),
+        "{what_was_rewritten}：只读恢复施加第一个文件那一条记录、回到 txg 3 那一版"
+    );
+    let mounted = mount_writable(&parameters(), &mut devices).unwrap_or_else(|error| {
+        panic!("{what_was_rewritten}：可写挂载退到更早的根照样挂上，实际 {error:?}")
+    });
+    assert_eq!(
+        (
+            mounted.output.chosen_root.checkpoint_txg,
+            mounted.output.effective_root.checkpoint_txg,
+            mounted.output.journal.prefix_applied,
+        ),
+        (LAST_WARM_UP_TXG, FIRST_FILE_TXG, 1),
+        "{what_was_rewritten}：可写挂载择 txg 2、施加一条记录、走 txg 3 那一版"
+    );
+}
+
+/// 场景：第一个文件那条根的树表指针算法类型写 1（登记过的加密算法，第一版不加密；攻方 f1）。
+/// 预期：这条根判损坏，两个入口都择暖机最后那条根、由记录施加回同一版。
+#[test]
+fn a_root_whose_tree_table_pointer_names_an_encryption_algorithm_is_not_chosen_by_either_mount() {
+    let mut devices = pool_after_the_first_file();
+    let rewritten_slots = rewrite_a_pointer_head_byte_in_the_first_file_root(
+        &mut devices,
+        RootRecordPointer::TreeTable,
+        ALGORITHM_TYPE_IN_THE_POINTER_HEAD,
+        1,
+    );
+    assert_the_first_file_root_is_refused_and_both_mounts_fall_back_to_the_warm_up_root(
+        devices,
+        &rewritten_slots,
+        "树表指针算法类型 1",
+    );
+}
+
+/// 场景：第一个文件那条根的实例表指针压缩算法码写 1（登记表第一版只有码 0）。
+/// 预期：这条根判损坏，两个入口都择暖机最后那条根、由记录施加回同一版。
+#[test]
+fn a_root_whose_instance_table_pointer_names_an_unregistered_compression_algorithm_is_not_chosen_by_either_mount(
+) {
+    let mut devices = pool_after_the_first_file();
+    let rewritten_slots = rewrite_a_pointer_head_byte_in_the_first_file_root(
+        &mut devices,
+        RootRecordPointer::InstanceTable,
+        COMPRESSION_ALGORITHM_CODE_IN_THE_POINTER_HEAD,
+        1,
+    );
+    assert_the_first_file_root_is_refused_and_both_mounts_fall_back_to_the_warm_up_root(
+        devices,
+        &rewritten_slots,
+        "实例表指针压缩算法码 1",
+    );
+}
+
+/// 场景：第一个文件那条根的映射根指针压缩算法码是 0、压后长度写 1。
+/// 预期：这条根判损坏，两个入口都择暖机最后那条根、由记录施加回同一版。
+#[test]
+fn a_root_whose_mapping_root_pointer_carries_a_compressed_length_without_compression_is_not_chosen_by_either_mount(
+) {
+    let mut devices = pool_after_the_first_file();
+    let rewritten_slots = rewrite_a_pointer_head_byte_in_the_first_file_root(
+        &mut devices,
+        RootRecordPointer::MappingRoot,
+        COMPRESSED_LENGTH_IN_THE_POINTER_HEAD,
+        1,
+    );
+    assert_the_first_file_root_is_refused_and_both_mounts_fall_back_to_the_warm_up_root(
+        devices,
+        &rewritten_slots,
+        "映射根指针压后长度 1、压缩算法码 0",
+    );
+}
+
+/// 场景：第一个文件那条根的分配记录树根指针（带文件的一版这一项整条全零）MAC 第一个字节写 1。
+/// 预期：这条根判损坏（全零指针的豁免只管 I-2.5 的升序，不管头部这几段），两个入口都择暖机最后那条根、由记录施加回同一版。
+#[test]
+fn a_root_whose_allocation_record_tree_root_pointer_carries_a_non_zero_mac_is_not_chosen_by_either_mount(
+) {
+    let mut devices = pool_after_the_first_file();
+    let rewritten_slots = rewrite_a_pointer_head_byte_in_the_first_file_root(
+        &mut devices,
+        RootRecordPointer::AllocationRecordTreeRoot,
+        MAC_IN_THE_POINTER_HEAD,
+        1,
+    );
+    assert_the_first_file_root_is_refused_and_both_mounts_fall_back_to_the_warm_up_root(
+        devices,
+        &rewritten_slots,
+        "分配记录树根指针 MAC 非 0",
+    );
+}
````

### mutations-append.tsv

````
I-2.4 根记录的树表指针只判 extent 偏移（树表指针算法类型 1 的根照择）	crates/singlefs-core/src/root_record.rs	        let tree_table =\n            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(\n                &mut reader,\n            )\n            .ok()?;	        let tree_table_bytes = reader.take(86);\n        if tree_table_bytes[32..34] != [0, 0] {\n            return None;\n        }\n        let tree_table = NodePointer::read_from(&mut ByteReader::at(tree_table_bytes, 0));	-p singlefs-harness --test root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts -- a_root_whose_tree_table_pointer_names_an_encryption_algorithm_is_not_chosen_by_either_mount	a_root_whose_tree_table_pointer_names_an_encryption_algorithm_is_not_chosen_by_either_mount
I-2.4 根记录的实例表指针只判 extent 偏移（实例表指针压缩算法码不认识的根照择）	crates/singlefs-core/src/root_record.rs	        let instance_table =\n            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(\n                &mut reader,\n            )\n            .ok()?;	        let instance_table_bytes = reader.take(86);\n        if instance_table_bytes[32..34] != [0, 0] {\n            return None;\n        }\n        let instance_table = NodePointer::read_from(&mut ByteReader::at(instance_table_bytes, 0));	-p singlefs-harness --test root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts -- a_root_whose_instance_table_pointer_names_an_unregistered_compression_algorithm_is_not_chosen_by_either_mount	a_root_whose_instance_table_pointer_names_an_unregistered_compression_algorithm_is_not_chosen_by_either_mount
I-2.4 根记录的映射根指针只判 extent 偏移（映射根指针码 0 而压后长度非 0 的根照择）	crates/singlefs-core/src/root_record.rs	        let mapping_root =\n            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(\n                &mut reader,\n            )\n            .ok()?;	        let mapping_root_bytes = reader.take(86);\n        if mapping_root_bytes[32..34] != [0, 0] {\n            return None;\n        }\n        let mapping_root = NodePointer::read_from(&mut ByteReader::at(mapping_root_bytes, 0));	-p singlefs-harness --test root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts -- a_root_whose_mapping_root_pointer_carries_a_compressed_length_without_compression_is_not_chosen_by_either_mount	a_root_whose_mapping_root_pointer_carries_a_compressed_length_without_compression_is_not_chosen_by_either_mount
I-2.4 根记录的分配记录树根指针只判 extent 偏移（分配记录树根指针 MAC 非 0 的根照择）	crates/singlefs-core/src/root_record.rs	        let allocation_record_tree_root =\n            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(\n                &mut reader,\n            )\n            .ok()?;	        let allocation_record_tree_root_bytes = reader.take(86);\n        if allocation_record_tree_root_bytes[32..34] != [0, 0] {\n            return None;\n        }\n        let allocation_record_tree_root = NodePointer::read_from(&mut ByteReader::at(allocation_record_tree_root_bytes, 0));	-p singlefs-harness --test root_record_pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_both_mounts -- a_root_whose_allocation_record_tree_root_pointer_carries_a_non_zero_mac_is_not_chosen_by_either_mount	a_root_whose_allocation_record_tree_root_pointer_carries_a_non_zero_mac_is_not_chosen_by_either_mount
I-2.4 挂载透明 翻全零指针段写出生 txg：中央映射树根空性恒判成立（树表 0 条的盘面上翻字节那条用例）	crates/singlefs-harness/src/mount_transparency.rs	                    is_all_zero(after) == is_all_zero(before),\n	                    true,\n	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount -- flipping_one_byte_of_the_mapping_root_range_on_a_pool_without_file_reddens_its_row	flipping_one_byte_of_the_mapping_root_range_on_a_pool_without_file_reddens_its_row
I-2.4 挂载透明 翻全零指针段写出生 txg：R1 带树时 R2 分配记录树根不判全零（三张字节表逐段翻那条用例）	crates/singlefs-harness/src/mount_transparency.rs	                    is_all_zero(after),\n                    "R1 带树 ⇒ R2 全零".to_string(),\n	                    true,\n                    "R1 带树 ⇒ R2 全零".to_string(),\n	-p singlefs-harness --test mount_transparency_judged_field_by_field_after_a_writable_mount -- flipping_one_byte_of_each_compared_range_of_the_three_field_byte_tables_reddens_its_row	flipping_one_byte_of_each_compared_range_of_the_three_field_byte_tables_reddens_its_row
````

### mutations-replacements.tsv

````
C581 根记录的实例表指针不判 extent 偏移（攻方 A6 那份镜像两个入口照挂）	crates/singlefs-core/src/root_record.rs	        let instance_table =\n            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(\n                &mut reader,\n            )\n            .ok()?;	        let instance_table = NodePointer::read_from(&mut reader);	-p singlefs-harness --test pointer_extent_offset_that_is_not_zero_is_refused_by_both_mounts	every_root_carrying_a_non_zero_instance_table_extent_offset_is_refused_by_recovery_and_the_writable_mount
D19 已定项 3 树表条目的根指针只判 extent 偏移（MAC 非 0 照读）	crates/singlefs-core/src/records.rs	            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(\n                &mut reader,\n            )\n            .ok()?;	            {\n                let pointer_bytes = reader.take(86);\n                if pointer_bytes[32..34] != [0, 0] {\n                    return None;\n                }\n                NodePointer::read_from(&mut ByteReader::at(pointer_bytes, 0))\n            };	-p singlefs-harness --test pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader -- a_tree_table_entry_whose_root_pointer_carries_a_non_zero_mac_is_refused	a_tree_table_entry_whose_root_pointer_carries_a_non_zero_mac_is_refused
D19 已定项 3 inode 树内部条目的子指针只判 extent 偏移（nonce 非 0 照读）	crates/singlefs-core/src/records.rs	    let child = NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(\n        &mut reader,\n    )\n    .map_err(PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion)?;	    let child_bytes = reader.take(86);\n    if child_bytes[32..34] != [0, 0] {\n        return Err(PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion(crate::pointer::PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(crate::pointer::PointerHeadExtentOffsetNotZero { extent_offset: u16::from_le_bytes([child_bytes[32], child_bytes[33]]) })));\n    }\n    let child = NodePointer::read_from(&mut ByteReader::at(child_bytes, 0));	-p singlefs-harness --test pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader -- an_inode_internal_entry_whose_child_pointer_carries_a_non_zero_nonce_is_refused	an_inode_internal_entry_whose_child_pointer_carries_a_non_zero_nonce_is_refused
D9 已定项 14 extent 叶记录的数据指针只判 extent 偏移（压缩算法码不认识照读）	crates/singlefs-core/src/records.rs	        DataPointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(\n            &mut ByteReader::at(bytes, 24),\n        )\n        .map_err(PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion)?;	        {\n            if bytes[24 + 32..24 + 34] != [0, 0] {\n                return Err(PointerCarryingEntryRefusal::PointerHeadFieldOutsideTheFirstVersion(crate::pointer::PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(crate::pointer::PointerHeadExtentOffsetNotZero { extent_offset: u16::from_le_bytes([bytes[24 + 32], bytes[24 + 33]]) })));\n            }\n            DataPointer::read_from(&mut ByteReader::at(bytes, 24))\n        };	-p singlefs-harness --test pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader -- an_extent_leaf_record_whose_data_pointer_names_an_unregistered_compression_algorithm_is_refused	an_extent_leaf_record_whose_data_pointer_names_an_unregistered_compression_algorithm_is_refused
D9 已定项 14 extent 上段叶条目内联的数据指针只判 extent 偏移（算法类型不是未加密照读）	crates/singlefs-core/src/extent_tree.rs	                DataPointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(\n                    &mut ByteReader::at(payload, 0),\n                )\n                .map_err(ExtentUpperLeafEntryMalformed::PointerHeadFieldOutsideTheFirstVersion)?,	                {\n                    if payload[32..34] != [0, 0] {\n                        return Err(ExtentUpperLeafEntryMalformed::PointerHeadFieldOutsideTheFirstVersion(crate::pointer::PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(crate::pointer::PointerHeadExtentOffsetNotZero { extent_offset: u16::from_le_bytes([payload[32], payload[33]]) })));\n                    }\n                    DataPointer::read_from(&mut ByteReader::at(payload, 0))\n                },	-p singlefs-harness --test pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader -- an_extent_upper_leaf_entry_whose_inline_data_pointer_names_an_encryption_algorithm_is_refused	an_extent_upper_leaf_entry_whose_inline_data_pointer_names_an_encryption_algorithm_is_refused
D9 已定项 14 extent 树内部条目的子指针只判 extent 偏移（码 0 而压后长度非 0 照读）	crates/singlefs-core/src/extent_tree.rs	                NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(\n                    &mut ByteReader::at(entry, key_width()),\n                )\n                .map(|child| (entry[..key_width()].to_vec(), child))	                (if entry[key_width() + 32..key_width() + 34] == [0, 0] {\n                    Ok(NodePointer::read_from(&mut ByteReader::at(entry, key_width())))\n                } else {\n                    Err(crate::pointer::PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(crate::pointer::PointerHeadExtentOffsetNotZero { extent_offset: u16::from_le_bytes([entry[key_width() + 32], entry[key_width() + 33]]) }))\n                })\n                .map(|child| (entry[..key_width()].to_vec(), child))	-p singlefs-harness --test pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader -- an_extent_internal_entry_whose_child_pointer_carries_a_compressed_length_without_compression_is_refused	an_extent_internal_entry_whose_child_pointer_carries_a_compressed_length_without_compression_is_refused
D19 已定项 3 分配记录树内部条目的子指针只判 extent 偏移（MAC 非 0 照读）	crates/singlefs-core/src/allocation_record_tree.rs	                NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(\n                    &mut reader,\n                )\n                .map_err(|field| RecoveryFailure::UnitMalformed {	                {\n                    let child_bytes = reader.take(86);\n                    if child_bytes[32..34] == [0, 0] {\n                        Ok(NodePointer::read_from(&mut ByteReader::at(child_bytes, 0)))\n                    } else {\n                        Err(crate::pointer::PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(crate::pointer::PointerHeadExtentOffsetNotZero { extent_offset: u16::from_le_bytes([child_bytes[32], child_bytes[33]]) }))\n                    }\n                }\n                .map_err(|field| RecoveryFailure::UnitMalformed {	-p singlefs-harness --test pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader -- an_allocation_record_tree_internal_entry_whose_child_pointer_carries_a_non_zero_mac_is_refused_by_the_walk	an_allocation_record_tree_internal_entry_whose_child_pointer_carries_a_non_zero_mac_is_refused_by_the_walk
D9 已定项 14 实例表链指针只判 extent 偏移（算法类型不是未加密照读）	crates/singlefs-core/src/instance_table.rs	                NodePointer::read_with_head_fields_judgement_from(\n                    &mut ByteReader::at(bytes, CHAIN_RECORD_POINTER_OFFSET),\n                    pointer_head_fields,\n                )\n                .ok()	                (bytes[CHAIN_RECORD_POINTER_OFFSET + 32..CHAIN_RECORD_POINTER_OFFSET + 34] == [0, 0])\n                    .then(|| NodePointer::read_from(&mut ByteReader::at(bytes, CHAIN_RECORD_POINTER_OFFSET)))	-p singlefs-harness --test pointer_encryption_and_compression_fields_outside_the_first_version_are_refused_by_every_tree_reader -- an_instance_table_chain_pointer_naming_an_encryption_algorithm_is_refused_by_the_walk	an_instance_table_chain_pointer_naming_an_encryption_algorithm_is_refused_by_the_walk
````

## m3-formal-34-3

34-3：B4 模型回退结构单测补取样点 100

### crates.patch

````diff
diff --git a/crates/singlefs-harness/src/model.rs b/crates/singlefs-harness/src/model.rs
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -4322,13 +4322,15 @@
 
     /// 模型回退那次发布对最左那片之外的叶容器的处置（`next_root_with_the_user_visible_trees` 取回退目标那一支里丢现行那几片、取目标那几片的两句），
     /// 结构单测（第 30 批判决 `research/prompts/m3-formal-code-b30-r1-main-verification.md` 第二节 M1「第 k 片都对」那一行的改法 B4）：
-    /// 现行与目标各取 1、2、3、4、5、135 片（135 是一个码 2 根装得下的上界，D8（核心索引结构） 已定项 6）的每一种组合，回退写出的那一版
+    /// 现行与目标各取 1、2、3、4、5、100、135 片（135 是一个码 2 根装得下的上界，D8（核心索引结构） 已定项 6）的每一种组合，回退写出的那一版
     /// 最左那片之外的叶容器（角色与分配代）恰好等于目标那一版的——现行有、目标没有的那几片这一版也没有，目标的每一片都取回、分配代取目标的。
+    /// 100 罩只按片数触发、不按位置触发的错：1–5 与 135 之间没有取样点时，只在目标装 60–118 片才错的回退照绿
+    /// （第 32–33 批代码轮判决 `research/prompts/m3-formal-code-b33-r1-main-verification.md` 第二节 P1「B4」那一行，`.claude/rules/mutation-sampling.md` 第三类）。
     /// 按取样点逐片往外加历史用例追不完（回退只取回或只丢某几片，只有建到那几片之外的历史抓得到），这一条不跑历史。
     #[test]
     fn a_rollback_takes_exactly_the_inode_leaf_containers_right_of_the_leftmost_of_its_target_for_every_pair_of_container_counts(
     ) {
-        const CONTAINER_COUNTS: [u64; 6] = [1, 2, 3, 4, 5, 135];
+        const CONTAINER_COUNTS: [u64; 7] = [1, 2, 3, 4, 5, 100, 135];
         let model = model_after_four_overwrites_in_a_second_instance();
         let newest = model.newest_root().clone();
         let mut pairs_judged = 0;
````

### mutations-append.tsv

````
第 34 批 34-3 B4（理想模型自身）：回退只在目标那一版装 60–118 片叶容器时只取回最左之外的前 3 片（第 32–33 批代码轮正推 b4-count-window-60-118 按片数改写，判决 P1 B4 补取样点 100）	crates/singlefs-harness/src/model.rs	                        .filter(|(role, _)| {\n                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                        })	                        .filter(|(role, _)| {\n                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))\n                        })\n                        .take(if (59..=117).contains(&target_root.role_written_at.keys().filter(|role| matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))).count()) { 3 } else { usize::MAX })	-p singlefs-harness --lib -- model::tests::a_rollback_takes_exactly_the_inode_leaf_containers	a_rollback_takes_exactly_the_inode_leaf_containers_right_of_the_leftmost_of_its_target_for_every_pair_of_container_counts
````

## m3-formal-34-4

34-4：34-1 改写之后 8 行变异按名重取原文

### mutations-replacements.tsv

````
I-5.4：只判最新根那棵账	crates/singlefs-checker/src/walk.rs	        candidate_indexes\n            .iter()\n            .map(|index| {\n                let root = &roots[*index].2;\n                VersionToJudgeTheAllocationRecordsOf {	        [newest_index]\n            .iter()\n            .map(|index| {\n                let root = &roots[*index].2;\n                VersionToJudgeTheAllocationRecordsOf {	-p singlefs-harness --test checker_known_bad_images -- an_allocation_record_whose_span	an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant
实审 B2 第 9 条（I-5.4 / C512）：根记录持有的那棵分配记录树不判互不相交	crates/singlefs-checker/src/walk.rs	        let mut allocation_roots = vec![parse_node_pointer(\n            version.allocation_record_tree_root_held_outside_the_tree_table,\n        )];	        let mut allocation_roots: Vec<PointerView> = Vec::new();	-p singlefs-harness --test checker_narrow_invariants_and_abandoned_roots -- overlapping_records_in_the_allocation_record_tree_the_root_record_holds_redden_only_the_disjointness_invariant	overlapping_records_in_the_allocation_record_tree_the_root_record_holds_redden_only_the_disjointness_invariant
实审 B2b（formatted_pool 写行之后那一格 I-5.4 改判成立）：根记录持有的那棵分配记录树不判互不相交	crates/singlefs-checker/src/walk.rs	        let mut allocation_roots = vec![parse_node_pointer(\n            version.allocation_record_tree_root_held_outside_the_tree_table,\n        )];	        let mut allocation_roots: Vec<PointerView> = Vec::new();	-p singlefs-harness --test writable_mount_of_a_formatted_pool -- a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold	a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold
第 26 批 26-5 GPU 判器：扫描内核不判带树一版根记录里的分配记录树根全零	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    if (tt.entry_count > 0u && !held.all_zero) {	    if (false && tt.entry_count > 0u && !held.all_zero) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_version_with_trees_also_holds_an_allocation_record_tree_in_its_root_record	the_gpu_judge_agrees_with_the_cpu_when_a_version_with_trees_also_holds_an_allocation_record_tree_in_its_root_record
第 32 批 32-1 F1 池级 checker：树表读不出的候选根不记进 I-5.4 部分没判（B 的树表清零照旧报成立，判决 M2 改法 F1）	crates/singlefs-checker/src/walk.rs	                versions_whose_tree_table_does_not_read_back.push(version.name.clone());	                Vec::<String>::new().push(version.name.clone());	-p singlefs-harness --lib --test checker_known_bad_images -- every_inode_leaf_container_one_root_holds a_rollback_takes_exactly_the_inode_leaf_containers the_tree_table_of_the_ the_genesis_tree_table_that_does_not_read_back an_inline_data_pointer_naming_an_encryption_algorithm a_tree_table_root_pointer_naming_an_unregistered_compression_algorithm an_inline_data_pointer_carrying_a_compressed_length	the_tree_table_of_the_newest_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged
第 32 批 32-1 F1 池级 checker：树表读不出的候选根不记进 I-5.4 部分没判（A 的树表清零照旧报成立）	crates/singlefs-checker/src/walk.rs	                versions_whose_tree_table_does_not_read_back.push(version.name.clone());	                Vec::<String>::new().push(version.name.clone());	-p singlefs-harness --test checker_known_bad_images -- the_tree_table_of_the_previous_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged	the_tree_table_of_the_previous_version_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged
第 32 批 32-1 F1 池级 checker：树表读不出的候选根不记进 I-5.4 部分没判（第 0 代树表清零照旧报成立）	crates/singlefs-checker/src/walk.rs	                versions_whose_tree_table_does_not_read_back.push(version.name.clone());	                Vec::<String>::new().push(version.name.clone());	-p singlefs-harness --test checker_known_bad_images -- the_genesis_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged	the_genesis_tree_table_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged
第 33 批 33-3 F1 同形 池级 checker：树表 0 条、根记录直接持有的分配记录树读不出的版本不记进 I-5.4 部分没判（写行那一版之上发了第一个文件版本，照旧报成立）	crates/singlefs-checker/src/walk.rs	            versions_whose_root_record_allocation_record_tree_does_not_read_back\n                .push(version.name.clone());	            Vec::<String>::new()\n                .push(version.name.clone());	-p singlefs-harness --test checker_known_bad_images -- the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged	the_allocation_record_tree_held_by_the_root_record_of_a_version_without_trees_that_does_not_read_back_leaves_the_disjointness_invariant_partially_judged
````

## #9 钉值（主 agent）

### diff -u 改前 今天

````diff
--- /tmp/claude-1000/-home-fy5090-code-singlefs/3c4dae2e-4622-4601-9f3b-0db4b8e8ce93/scratchpad/pin9-before.rs	2026-10-05 22:36:43.315236617 +0000
+++ crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs	2026-10-05 22:39:47.196185280 +0000
@@ -842,14 +842,14 @@
 /// 计划哈希也是 `0370ad26…`、打的行只比主工作区少 `I-9.28=0/0/46` 那两项）。
 const GOLDEN_PRINTED_LINE_COUNT: usize = 58;
 const GOLDEN_PRINTED_LINES_SHA256: &str =
-    "e1ab3f797f369a2e9f3e45195331d252306b41346bef10922c7e04e1336a8cc4";
+    "50615e47416cf01ad5f2b45692b29f03705ceeb459e0a7aa659ecae47d744066";
 /// 同一趟留下的进度文件：名字（带计划哈希）与整份内容的 SHA-256（47 行：文件头 + 46 行片行）。名字取自上面那一趟日志的 `LAYER0_RESUME` 行；
-/// 打的行与内容的 SHA-256 取自 2026-10-05「第 28 批 + 根环线 10e」那棵树（根环线点名跑这一条，树指纹开头结尾都是 `bdf78215…`）：
-/// 打的行是那一趟断言 `left` 的 `e1ab3f79…`；
+/// 打的行与内容的 SHA-256 取自 2026-10-05 主工作区（形式化线第 34 批派出时那一版，含删除线第 22 批）：与根环线在「10g + 32-1」那棵树上重取的 58 行逐行比，只差 GOLDEN_TALLY 两行多出 I-9.17–I-9.24、I-9.29 九条新不变量的计数（删除线第 22 批把已实现的不变量从 74 条加到 83 条），已有各条计数不变；那一次比上一版只差 `LAYER0_RESUME` 那一行的计划哈希（T_time 改毫秒带来）：
+/// 打的行是那一趟断言 `left` 的 `50615e47…`；
 /// 内容是在只换了打的行钉值的副本里再跑一次、「不分片时的进度文件与钉值不同」那条断言 `left` 的第二项。池级 checker 的判定集合再变，两个值跟着重取。
-const GOLDEN_PROGRESS_FILE_NAME: &str = "layer0-progress-golden_new_pool_file_creation_stream-fingerprint0-0370ad26cad051266c57d7129a63840319034b488aefec9adebf007a9809b91e.txt";
+const GOLDEN_PROGRESS_FILE_NAME: &str = "layer0-progress-golden_new_pool_file_creation_stream-fingerprint0-4987883029b866a331bd86aa1db770710eccbf4db5206676859243f01abd914c.txt";
 const GOLDEN_PROGRESS_FILE_SHA256: &str =
-    "2a77c2d767d5dadc12e55e06860327826fb9c95c04870fe15ab5cb1241bf26bc";
+    "f6f10d5d687564847462cdd987b12ce906c262712c66350915a9cc463374e288";
 
 /// 一行打印里会随机器与时刻变的两样换掉：进度目录、`elapsed_seconds=` 的值。
 fn masked_printed_line(line: &str, directory: &Path) -> String {
````
