# 第 7 批代码轮第一轮：被判的改动全文（2026-10-03）

三份补丁依次打进主工作区，冻结副本 `/tmp/claude-1000/m3-root-ring-code-r1/tree/` 是打完三份之后的样子。

## 7-2 根环实现（`/tmp/claude-1000/m3-r1/batch7-root-ring/patch/crates.patch`，报告同目录 `report.md`）

````diff
diff --git a/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs b/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
index 2ad9646..172947a 100644
--- a/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
+++ b/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs
@@ -377,6 +377,14 @@ impl RunProfile {
     }
 }
 
+/// 装置生成的每条记录的新根段后三项（实例表指针、分配记录树根、卸载记号；D23（journal 的角色与格式） 已定项 15，2026-10-02 用户定
+/// 「回到甲」之后施加一条记录时这三项取记录里的，不再抄所选根）：一律带 r1 档时间线那一份常量。r1 档的所选根投影之外的字段就是这几个常量，
+/// 施加结果与改之前逐态相同；r2 / r3 档的投影之外字段随版本变，J7 那几行判据照「投影之外抄所选根」登记，与新的施加规则对不上，
+/// 要重新登记（实现员 2026-10-03 只为让装置照新的记录格式编得过，登记与判据一个字没改）。
+fn fields_beyond_the_projection_carried_by_every_generated_record() -> FieldsBeyondTheProjection {
+    RunProfile::R1.fields_beyond_the_projection_of_version(0)
+}
+
 /// 根记录里投影之外的四个字段（登记 r2 第一节「投影之外的根字段」）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 struct FieldsBeyondTheProjection {
@@ -892,6 +900,15 @@ fn generate_history(plan: &HistoryPlan) -> History {
                     new_mapping_root: segment.mapping_root,
                     new_tree_identifier_watermark: segment.tree_identifier_watermark,
                     new_rollback_floor: CheckpointTxg(segment.rollback_floor),
+                    new_instance_table:
+                        fields_beyond_the_projection_carried_by_every_generated_record()
+                            .instance_table,
+                    new_allocation_record_tree_root:
+                        fields_beyond_the_projection_carried_by_every_generated_record()
+                            .allocation_record_tree_root,
+                    new_unmount_marker:
+                        fields_beyond_the_projection_carried_by_every_generated_record()
+                            .unmount_marker,
                     named: unit
                         .as_ref()
                         .map(|generated_unit| vec![generated_unit.named])
@@ -4789,6 +4806,13 @@ impl AdversarialEnumeration<'_> {
             new_mapping_root: chosen_fields.segment.mapping_root,
             new_tree_identifier_watermark: chosen_fields.segment.tree_identifier_watermark,
             new_rollback_floor: CheckpointTxg(chosen_fields.segment.rollback_floor),
+            new_instance_table: fields_beyond_the_projection_carried_by_every_generated_record()
+                .instance_table,
+            new_allocation_record_tree_root:
+                fields_beyond_the_projection_carried_by_every_generated_record()
+                    .allocation_record_tree_root,
+            new_unmount_marker: fields_beyond_the_projection_carried_by_every_generated_record()
+                .unmount_marker,
             named: Vec::new(),
         };
         generate_slot(
@@ -4841,6 +4865,13 @@ impl AdversarialEnumeration<'_> {
             new_mapping_root: segment.mapping_root,
             new_tree_identifier_watermark: segment.tree_identifier_watermark,
             new_rollback_floor: CheckpointTxg(segment.rollback_floor),
+            new_instance_table: fields_beyond_the_projection_carried_by_every_generated_record()
+                .instance_table,
+            new_allocation_record_tree_root:
+                fields_beyond_the_projection_carried_by_every_generated_record()
+                    .allocation_record_tree_root,
+            new_unmount_marker: fields_beyond_the_projection_carried_by_every_generated_record()
+                .unmount_marker,
             named: Vec::new(),
         };
         generate_slot(
@@ -7568,6 +7599,13 @@ fn hand_made_record(
         new_mapping_root: segment.mapping_root,
         new_tree_identifier_watermark: segment.tree_identifier_watermark,
         new_rollback_floor: CheckpointTxg(segment.rollback_floor),
+        new_instance_table: fields_beyond_the_projection_carried_by_every_generated_record()
+            .instance_table,
+        new_allocation_record_tree_root:
+            fields_beyond_the_projection_carried_by_every_generated_record()
+                .allocation_record_tree_root,
+        new_unmount_marker: fields_beyond_the_projection_carried_by_every_generated_record()
+            .unmount_marker,
         named: Vec::new(),
     }
 }
@@ -9097,6 +9135,13 @@ fn root_choice_publish_material(plan: &RootChoicePublishPlan) -> RootChoicePubli
             new_mapping_root: segment.mapping_root,
             new_tree_identifier_watermark: segment.tree_identifier_watermark,
             new_rollback_floor: CheckpointTxg(segment.rollback_floor),
+            new_instance_table: fields_beyond_the_projection_carried_by_every_generated_record()
+                .instance_table,
+            new_allocation_record_tree_root:
+                fields_beyond_the_projection_carried_by_every_generated_record()
+                    .allocation_record_tree_root,
+            new_unmount_marker: fields_beyond_the_projection_carried_by_every_generated_record()
+                .unmount_marker,
             named: Vec::new(),
         };
         let slot = generate_slot(
diff --git a/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs b/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
index a6e8b1a..c4b6b68 100644
--- a/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
+++ b/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
@@ -2562,6 +2562,9 @@ impl BadStreamFacts<'_> {
                 new_tree_identifier_watermark: template.new_tree_identifier_watermark
                     + pseudo.watermark_step,
                 new_rollback_floor: template.new_rollback_floor,
+                new_instance_table: template.new_instance_table,
+                new_allocation_record_tree_root: template.new_allocation_record_tree_root,
+                new_unmount_marker: template.new_unmount_marker,
                 named: Vec::new(),
             };
             let bytes = record.to_bytes();
diff --git a/crates/singlefs-checker/src/lib.rs b/crates/singlefs-checker/src/lib.rs
index e01fa01..3ea25df 100644
--- a/crates/singlefs-checker/src/lib.rs
+++ b/crates/singlefs-checker/src/lib.rs
@@ -696,7 +696,7 @@ pub fn packed_unit_view(unit: &[u8]) -> Result<PackedUnitView, Verdict> {
 }
 
 /// journal 记录头的偏移（D23（journal 的角色与格式） 已定项 4 的字段表）：magic 4 + 类型 2 + 算法类型 1 之后是记录标志 1；
-/// 事务号 8 与提交标记 1 之后紧跟本次发布内序号 4，再是反向链 4、载荷校验和 4、新根段 188、fsid 8、MAC 16，头到 311 为止。
+/// 事务号 8 与提交标记 1 之后紧跟本次发布内序号 4，再是反向链 4、载荷校验和 4、新根段 364、fsid 8、MAC 16，头到 487 为止。
 const JOURNAL_MAGIC: &[u8; 4] = b"SFSJ";
 const JOURNAL_RECORD_FLAGS_OFFSET: usize = 7;
 const JOURNAL_HEADER_CHECKSUM_OFFSET: usize = 46;
@@ -706,7 +706,7 @@ const JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET: usize = 87;
 const JOURNAL_BACK_CHAIN_OFFSET: usize = 91;
 const JOURNAL_PAYLOAD_CHECKSUM_OFFSET: usize = 95;
 const JOURNAL_NEW_ROOT_SEGMENT_OFFSET: usize = 99;
-const JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET: usize = 287;
+const JOURNAL_FILESYSTEM_IDENTIFIER_OFFSET: usize = 463;
 
 /// 点名项解出来的样子（D23（journal 的角色与格式） 已定项 17）。
 #[derive(Clone, Debug, PartialEq, Eq)]
@@ -750,17 +750,18 @@ pub struct JournalRecordView {
     /// 不在这里拒其余位非 0 的记录，I-8.9（一次发布的记录序号连续且只有末条带标志） 判红——拒了它就看不见。
     pub record_flags_byte: u8,
     pub back_chain: u32,
-    /// 新根段 188 字节原样（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8）。
+    /// 新根段 364 字节原样（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8 + 实例表指针 86 + 分配记录树根指针 86 + 卸载记号 4，
+    /// D23（journal 的角色与格式） 已定项 15）。
     pub new_root_segment: Vec<u8>,
     pub new_tree_identifier_watermark: u64,
     pub new_rollback_floor: u64,
     pub named: Vec<NamedEntryView>,
 }
 
-/// 反向链的口径（D23（journal 的角色与格式） 已定项 19 ②）：前一条记录 311 字节头的 CRC-32C，`header_csum` 那 32 字节按 0 参与。
+/// 反向链的口径（D23（journal 的角色与格式） 已定项 19 ②）：前一条记录 487 字节头的 CRC-32C，`header_csum` 那 32 字节按 0 参与。
 #[must_use]
 pub fn back_chain_of_record_header(record: &[u8]) -> u32 {
-    let mut header = record[..usize::try_from(JOURNAL_HEADER_BYTES).expect("311")].to_vec();
+    let mut header = record[..usize::try_from(JOURNAL_HEADER_BYTES).expect("487")].to_vec();
     header[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32].fill(0);
     crc32_castagnoli_bitwise(&header)
 }
@@ -790,7 +791,7 @@ pub fn check_journal_record(
         return Err(Verdict::FilesystemIdentifierMismatch);
     }
     let named_count = usize::try_from(read_u32(record, 12)).expect("点名项数");
-    let header_bytes = usize::try_from(JOURNAL_HEADER_BYTES).expect("311");
+    let header_bytes = usize::try_from(JOURNAL_HEADER_BYTES).expect("487");
     let entry_bytes = usize::try_from(JOURNAL_NAMED_ENTRY_BYTES).expect("56");
     let payload_end = header_bytes + named_count * entry_bytes;
     if payload_end > record_bytes {
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 9268ab4..a57463a 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -1047,34 +1047,24 @@ impl<'reader> Walk<'reader> {
     }
 
     /// 由记录施加出来、根槽从没落盘的那一版（[`versions_applied_only_by_records`] 认出来的）：它没有根记录，
-    /// 树表指针与映射根指针在那条记录的新根段里，从那里走下去，判定与根环里的候选根同一套。
-    /// 根记录里另外两样——实例表指针与「树表 0 条那一版的分配记录树的根」——记录的新根段（188 字节）里没有它们的位置：
-    /// 由记录重建的根，这两样照它施加在其上那一版的根（D23（journal 的角色与格式） 已定项 15「实例表单元指针照所选根」，
-    /// 施加一条记录只换新根段那四个字段），从那条根的根记录里取来走；那条根不在回退候选集里（低于 F，或被别的条件滤掉）也照走这两样
-    /// （主 agent 2026-09-27 定，零轮；崩溃注入快档种子 7463871032432355136 第 91 段那两条 I-3.1 就是漏了它施加在其上那一版的实例表，
-    /// `research/prompts/m2-investigate-crash-injection-reds-report.md` 第三节）。
-    /// 环里找不到它施加在其上那一版的根（同实例、txg 比它小的根一条都读不出）时这两样走不到：这一格没有条款，照旧只走新根段。
+    /// 树表指针、映射根指针、实例表指针与「树表 0 条那一版的分配记录树的根」都在那条记录的新根段里（D23（journal 的角色与格式）
+    /// 已定项 15：施加一条记录 = 把所选根的七个字段换成记录里的，用户 2026-10-02「回到甲」），从那里走下去，判定与根环里的候选根同一套。
     fn walk_version_applied_only_by_records(&mut self, version: &VersionAppliedOnlyByRecords) {
         self.begin_walking_a_version();
         self.version_being_walked = VersionBeingWalked::AppliedOnlyByRecords {
             checkpoint_txg: version.checkpoint_txg,
         };
-        if let Some(record_of_the_root_it_was_applied_on) =
-            &version.root_record_of_the_version_it_was_applied_on
-        {
-            let first_page_pointer = parse_node_pointer(
-                &record_of_the_root_it_was_applied_on[ROOT_RECORD_INSTANCE_TABLE_POINTER],
+        let first_page_pointer =
+            parse_node_pointer(&version.new_root_segment[NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER]);
+        self.walk_instance_table_chain(&first_page_pointer, None);
+        let allocation_record_tree_root =
+            &version.new_root_segment[NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER];
+        if !parse_node_pointer(allocation_record_tree_root).all_zero {
+            self.walk_allocation_record_tree(
+                allocation_record_tree_root,
+                0,
+                "树表 0 条那一版的分配记录树",
             );
-            self.walk_instance_table_chain(&first_page_pointer, None);
-            let allocation_record_tree_root =
-                &record_of_the_root_it_was_applied_on[ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT];
-            if !parse_node_pointer(allocation_record_tree_root).all_zero {
-                self.walk_allocation_record_tree(
-                    allocation_record_tree_root,
-                    0,
-                    "树表 0 条那一版的分配记录树",
-                );
-            }
         }
         let (tree_table_pointer, mapping_root_pointer) =
             version.tree_table_and_mapping_root_pointers();
@@ -3943,13 +3933,13 @@ fn tree_table_pointer_of_the_version_the_next_mount_applies_first(
     let records_as_the_recovery_reads_them = records_as_the_recovery_reads_them(records_by_device);
     records_as_the_recovery_reads_them
         .iter()
-        .find(|(counter, record)| {
+        .find(|((_, counter), record)| {
             record.commit_marker == CommitMarker::Present
                 && record.instance == newest_root.instance
                 && record.checkpoint_txg == next_txg
                 && record_can_enter_a_replay_prefix(
                     &records_as_the_recovery_reads_them,
-                    **counter,
+                    *counter,
                     record,
                 )
         })
@@ -4611,7 +4601,8 @@ impl LocationEntryChecksumsForQuarantine<'_> {
 
 /// 收口表第 43 行取丁-defer（用户 2026-09-28 定，被攻过一轮；`research/prompts/abandoned-floor-r1-main-verification.md` M1）：
 /// I-3.1（已分配统计对得上） 的「实际遍历」并上这样的槽——F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的分配记录树里
-/// 罩住它的记录是已释放、释放代高于 F_生效 的（还在 defer 里，抬 F 还不许回收，记账照算已分配）。逐盘交回槽数。
+/// 罩住它的记录是已释放、释放代高于回收门槛 `reclaim_threshold` = max(F_生效, 一圈窗口里最旧有效根) 的（还在 defer 里，分配器还不许回收，
+/// 记账照算已分配；门槛 2026-10-02 起取一圈窗口，D16（发布语义） 已定项 1）。逐盘交回槽数。
 /// 最新根的树表或分配记录树读不出时一槽都不并（照旧判）。
 fn deferred_slots_referenced_only_below_the_floor_per_device(
     reader: &dyn ImageReader,
@@ -4619,7 +4610,7 @@ fn deferred_slots_referenced_only_below_the_floor_per_device(
     newest_root: &crate::RootView,
     roots_below_the_floor: &[&crate::RootView],
     walked_slots: &BTreeSet<(u32, u64)>,
-    effective_rollback_floor: u64,
+    reclaim_threshold: u64,
     cache: &mut IndexNodeCache,
 ) -> BTreeMap<u32, u64> {
     let mut deferred_per_device: BTreeMap<u32, u64> = BTreeMap::new();
@@ -4646,7 +4637,7 @@ fn deferred_slots_referenced_only_below_the_floor_per_device(
             continue;
         };
         for record in records {
-            if record.is_released && record.generation > effective_rollback_floor {
+            if record.is_released && record.generation > reclaim_threshold {
                 released_above_the_floor.extend(
                     (record.slot..record.slot + record.span_slots)
                         .map(|slot| (record.device, slot)),
@@ -5275,7 +5266,8 @@ struct ScannedJournalRecord {
     instance: u32,
     /// 记录头里的 checkpoint_txg：这条记录施加出来的是哪一版（「由记录施加出来、根槽从没落盘的那一版」按它认）。
     checkpoint_txg: u64,
-    /// 记录头里的新根段 188 字节原样（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8，D23（journal 的角色与格式） 已定项 4）：
+    /// 记录头里的新根段 364 字节原样（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8 + 实例表指针 86 + 分配记录树根指针 86 + 卸载记号 4，
+    /// D23（journal 的角色与格式） 已定项 4 / 已定项 15）：
     /// 这条记录施加出来的那一版从哪个树表、哪个映射根走下去。
     new_root_segment: Vec<u8>,
     /// 记录头里的事务号（D23（journal 的角色与格式） 已定项 7）：I-8.7（实例内事务号不重号） 判的就是它。
@@ -5288,7 +5280,7 @@ struct ScannedJournalRecord {
     record_flags_byte: u8,
     slot: u64,
     back_chain: u32,
-    /// CRC32C(这一条的 311 字节头，`header_csum` 那 32 字节按零参与)——I-8.6（反向链算法） 的算式。
+    /// CRC32C(这一条的 487 字节头，`header_csum` 那 32 字节按零参与)——I-8.6（反向链算法） 的算式。
     chain_value_the_next_record_must_carry: u32,
     /// 每个点名项的两条位置条目（D23（journal 的角色与格式） 已定项 17）：I-2.5（位置条目按设备身份升序） 逐项判。
     named_entry_locations: Vec<[PointerLocation; 2]>,
@@ -5406,16 +5398,19 @@ fn expected_back_chain_of(
     }
 }
 
-/// 恢复眼里的 journal 环：每个计数器取按盘的次序（`reader.devices()`，与恢复扫环按 `device_identities()` 的次序相同）第一块盘上
-/// 自证过的那一份。I-8.6（反向链算法） 那一判逐盘判两份（一块盘上的链坏了另一块不该跟着坏）；认「恢复施加过 / 下一次挂载会施加」
+/// 恢复眼里的 journal 环：键是 (实例代号, 计数器)，每个键取按盘的次序（`reader.devices()`，与恢复扫环按 `device_identities()` 的次序相同）
+/// 第一块盘上自证过的那一份——与实现扫环建表同一个键（`crates/singlefs-core/src/recovery.rs` 的 `scan_journal`），两块盘同一计数器坐着
+/// 不同实例的记录时两条都留（I-4.1 那一行：`records_the_replay_reads` 先改了键，这两处同改，主 agent 2026-10-03 定）。I-8.6（反向链算法） 那一判逐盘判两份（一块盘上的链坏了另一块不该跟着坏）；认「恢复施加过 / 下一次挂载会施加」
 /// 哪几条记录时照恢复的读法只看这一份（实审 A2a 报告「checker 要跟着改什么」第 1 条，`research/prompts/m2-rev-a2a-implementer-report.md`）。
 fn records_as_the_recovery_reads_them(
     records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
-) -> BTreeMap<u64, &ScannedJournalRecord> {
-    let mut first_copies: BTreeMap<u64, &ScannedJournalRecord> = BTreeMap::new();
+) -> BTreeMap<(u32, u64), &ScannedJournalRecord> {
+    let mut first_copies: BTreeMap<(u32, u64), &ScannedJournalRecord> = BTreeMap::new();
     for (_, records) in records_by_device {
         for (counter, record) in records {
-            first_copies.entry(*counter).or_insert(record);
+            first_copies
+                .entry((record.instance, *counter))
+                .or_insert(record);
         }
     }
     first_copies
@@ -5423,25 +5418,22 @@ fn records_as_the_recovery_reads_them(
 
 /// I-8.6「不等的记录不进重放前缀」（代码审阅 6c 第 25 条，用户 2026-09-27 定恢复补反向链检查、checker 的前缀口径跟着对齐）：
 /// 照恢复的判法（`crates/singlefs-core/src/recovery.rs` 的 `judge_back_chain_against_the_previous_record_on_disk`，实审 A2a）——
-/// 本实例内逻辑前一条（同一实例、计数器小 1）在盘上、它的头算出来的链值与这一条的反向链不等，这一条进不了前缀。
+/// 本实例内逻辑前一条（同一实例、计数器小 1，在 [`records_as_the_recovery_reads_them`] 里按 (实例代号, 计数器 − 1) 找）在盘上、
+/// 它的头算出来的链值与这一条的反向链不等，这一条进不了前缀。
 /// 恢复只判这一种：前缀都接在所选根覆盖的那一条之后、同一实例，计数器 1、前一条是别的实例、前一条不在盘上都不判
 /// （A2a 报告那一节第 2 条；I-8.6 那一判对前两种照条款判「链恒 0」，前缀这里不跟着判）。计数器 0 进不了前缀
 /// （jsn 从所选根覆盖的那一条 + 1 起严格连续，计数器从 1 起，D23（journal 的角色与格式） 已定项 18）。
 fn record_can_enter_a_replay_prefix(
-    records_as_the_recovery_reads_them: &BTreeMap<u64, &ScannedJournalRecord>,
+    records_as_the_recovery_reads_them: &BTreeMap<(u32, u64), &ScannedJournalRecord>,
     counter: u64,
     record: &ScannedJournalRecord,
 ) -> bool {
     let Some(previous_counter) = counter.checked_sub(1) else {
         return false;
     };
-    match records_as_the_recovery_reads_them.get(&previous_counter) {
-        Some(previous) if previous.instance == record.instance => {
-            record.back_chain == previous.chain_value_the_next_record_must_carry
-        }
-        Some(_record_of_another_instance) => true,
-        None => true,
-    }
+    records_as_the_recovery_reads_them
+        .get(&(record.instance, previous_counter))
+        .is_none_or(|previous| record.back_chain == previous.chain_value_the_next_record_must_carry)
 }
 
 /// 逐盘扫一遍 journal 环，把自证过的记录按盘收齐：判 journal 的四条不变量（I-8.6（反向链算法）、
@@ -5469,7 +5461,7 @@ fn scanned_journal_records_by_device(
         .collect()
 }
 
-/// I-8.6（反向链算法）：任一 journal 记录的反向链 = CRC32C(**本实例内逻辑前一条**记录的 311 字节头，`header_csum` 那 32 字节
+/// I-8.6（反向链算法）：任一 journal 记录的反向链 = CRC32C(**本实例内逻辑前一条**记录的 487 字节头，`header_csum` 那 32 字节
 /// 按零参与)；**本实例写出的第一条记录反向链恒 0**。逐盘判：环两盘互为镜像，一块盘上的链坏了另一块不该跟着坏。
 /// 「本实例内逻辑前一条」= 同一实例、计数器小 1 的那一条，三种情形穷举：
 /// ① 计数器 == 1 ⇒ 全池第一条，必是本实例第一条 ⇒ 反向链恒 0；
@@ -5906,25 +5898,43 @@ fn judge_publish_ordinals_and_last_record_flags(
     }
 }
 
-/// 记录新根段里两条指针的落点（D23（journal 的角色与格式） 已定项 4 的字段表：树表指针 86、映射根指针 86，再往后是树 ID 水位 8 与 F 8）。
+/// 记录新根段七段的落点（D23（journal 的角色与格式） 已定项 4 / 已定项 15 的字段表：树表指针 86、映射根指针 86、树 ID 水位 8、F 8、
+/// 实例表指针 86、分配记录树根指针 86、卸载记号 4）。
 const NEW_ROOT_SEGMENT_TREE_TABLE_POINTER: std::ops::Range<usize> = 0..86;
 const NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER: std::ops::Range<usize> = 86..172;
 const NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK: std::ops::Range<usize> = 172..180;
 const NEW_ROOT_SEGMENT_ROLLBACK_FLOOR: std::ops::Range<usize> = 180..188;
+const NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER: std::ops::Range<usize> = 188..274;
+const NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER: std::ops::Range<usize> = 274..360;
 /// 根记录里与新根段四段同名的四段（D22（单元原子性怎么合成） 已定项 7 的偏移表：树表指针 36、树 ID 水位 122、F 130、映射根指针 256）。
 const ROOT_RECORD_TREE_TABLE_POINTER: std::ops::Range<usize> = 36..122;
 const ROOT_RECORD_TREE_IDENTIFIER_WATERMARK: std::ops::Range<usize> = 122..130;
 const ROOT_RECORD_ROLLBACK_FLOOR: std::ops::Range<usize> = 130..138;
 const ROOT_RECORD_MAPPING_ROOT_POINTER: std::ops::Range<usize> = 256..342;
-/// 根记录里新根段之外、由记录重建的根照所选根的两条指针（D22（单元原子性怎么合成） 已定项 7 的偏移表：实例表指针 86 在 170，
-/// 树表 0 条那一版的分配记录树的根 86 在 342）。
-const ROOT_RECORD_INSTANCE_TABLE_POINTER: std::ops::Range<usize> = 170..256;
 /// 根记录里实例代号 4 字节与 `checkpoint_txg` 8 字节的偏移（D22（单元原子性怎么合成） 已定项 7 的偏移表，与 `check_root_slot` 读的同一处）：
 /// I-9.16 判红时拿它们说是哪一版根。
 const ROOT_RECORD_INSTANCE_OFFSET: usize = 24;
 const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;
+/// 根记录里「树表 0 条那一版的分配记录树的根」86 在 342（D22（单元原子性怎么合成） 已定项 7 的偏移表）。
 const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT: std::ops::Range<usize> = 342..428;
 
+/// 一圈窗口（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」，用户 2026-10-02 定）：N = 镜像根环里读得出的最新有效根的 txg，
+/// 窗口 = txg > N − 3S（3S 是环长，根环槽数）。同取这个窗口的五处里 checker 占两处：候选过滤（还有「由记录施加出来的那一版」
+/// 判回收用的最旧有效根）与 I-3.1（已分配统计对得上） 丁-defer 的门槛。窗口之外的有效根是环上有洞时留在洞那一槽里的上一圈的根，
+/// 分配器的回收门槛不再被它压住（`crates/singlefs-core/src/allocator.rs` 的 `RootRingOccupancy::oldest_valid_root_in_the_window`）。
+#[derive(Clone, Copy, Debug)]
+struct OneRevolutionWindow {
+    newest_valid_root_txg: u64,
+    root_ring_slot_count: u64,
+}
+
+impl OneRevolutionWindow {
+    /// txg 落在窗口里：txg > N − 3S，写成 txg + 3S > N（N < 3S 时整个环都在窗口里，不减出负数）。
+    fn contains(self, checkpoint_txg: u64) -> bool {
+        checkpoint_txg.saturating_add(self.root_ring_slot_count) > self.newest_valid_root_txg
+    }
+}
+
 /// 由记录施加出来、根槽从没落盘的那一版：一条 journal 记录自带新根段（树表指针与映射根指针），恢复把它施加在所选根之上，
 /// 这一版就成了现行版；之后写行那次发布把它换下的单元放进 defer——那些单元仍占着空间、仍算在「已分配」里，
 /// 而根环里没有任何一条根引用它们（它自己的根槽一次都没写过）。
@@ -5932,10 +5942,6 @@ struct VersionAppliedOnlyByRecords {
     instance: u32,
     checkpoint_txg: u64,
     new_root_segment: Vec<u8>,
-    /// 它施加在其上那一版的根记录：根环里同一实例、txg 比它小的根里 txg 最大的那一条（施加一条记录只换新根段那四个字段，
-    /// 实例表指针与树表 0 条那一版的分配记录树的根照所选根；一个实例写行之后它的每一版照抄同一张实例表，隔着几条也是同一份）。
-    /// 那条根在不在回退候选集里都取。环里一条都读不出时是 None。
-    root_record_of_the_version_it_was_applied_on: Option<Vec<u8>>,
 }
 
 impl VersionAppliedOnlyByRecords {
@@ -5958,8 +5964,8 @@ impl VersionAppliedOnlyByRecords {
 ///    记录要带提交标记（D23（journal 的角色与格式） 已定项 7：不带的是恢复要丢的尾巴）。
 /// ② **它的根槽读不出**：根环里自证过的根没有一条是 (i, 这个 txg)。读得出的那一版照常由根环那一路走。
 /// ③ **不低于回退下界**：txg ≥ 最新根带的 F，与根环里的候选根同一条（D16（发布语义） 已定项 1）。
-/// ④ **它换下的单元还没到回收的时候**：根环里有一条没被实例表判抛弃的根，txg 比它小。
-///    理由：一个单元只被这一版引用时，它的释放代大于这一版的 txg；回收要「释放代 ≤ max(F_生效, 环里最旧有效根)」
+/// ④ **它换下的单元还没到回收的时候**：一圈窗口（[`OneRevolutionWindow`]）里有一条没被实例表判抛弃的根，txg 比它小。
+///    理由：一个单元只被这一版引用时，它的释放代大于这一版的 txg；回收要「释放代 ≤ max(F_生效, 一圈窗口里最旧有效根)」
 ///    （D3（空间分配） 已定项 7）。环里最旧的有效根比这一版老 ⇒ 最旧有效根 < 这一版 < 释放代，③ 又让 F ≤ 这一版 ⇒ 回收不了，
 ///    它们还算在已分配里；反过来环里最旧的有效根已经比这一版新，这一版里只有它引用的单元释放代都不超过那条根，已可回收复用，
 ///    再把它并进来就是拿已经不归它的槽去比。
@@ -5970,6 +5976,7 @@ fn versions_applied_only_by_records(
     roots: &[(u64, u64, crate::RootView)],
     instance_table_rows: &[InstanceTableRow],
     rollback_floor: u64,
+    window: OneRevolutionWindow,
 ) -> Vec<VersionAppliedOnlyByRecords> {
     let is_abandoned = |instance: u32, checkpoint_txg: u64| {
         instance_table_rows
@@ -5978,7 +5985,10 @@ fn versions_applied_only_by_records(
     };
     let Some(oldest_valid_root_txg) = roots
         .iter()
-        .filter(|(_, _, root)| !is_abandoned(root.instance, root.checkpoint_txg))
+        .filter(|(_, _, root)| {
+            !is_abandoned(root.instance, root.checkpoint_txg)
+                && window.contains(root.checkpoint_txg)
+        })
         .map(|(_, _, root)| root.checkpoint_txg)
         .min()
     else {
@@ -5987,7 +5997,7 @@ fn versions_applied_only_by_records(
     let mut versions: BTreeMap<(u32, u64), Vec<u8>> = BTreeMap::new();
     // 按恢复的读法（每个计数器取第一块盘上的那一份）：反向链对不上的记录不进重放前缀（I-8.6（反向链算法）），不算被哪一次恢复施加过。
     let records_as_the_recovery_reads_them = records_as_the_recovery_reads_them(records_by_device);
-    for (counter, record) in &records_as_the_recovery_reads_them {
+    for ((_, counter), record) in &records_as_the_recovery_reads_them {
         let applied_by_a_recovery = record.commit_marker == CommitMarker::Present
             && record_can_enter_a_replay_prefix(
                 &records_as_the_recovery_reads_them,
@@ -6020,13 +6030,6 @@ fn versions_applied_only_by_records(
                 instance,
                 checkpoint_txg,
                 new_root_segment,
-                root_record_of_the_version_it_was_applied_on: roots
-                    .iter()
-                    .filter(|(_, _, root)| {
-                        root.instance == instance && root.checkpoint_txg < checkpoint_txg
-                    })
-                    .max_by_key(|(_, _, root)| root.checkpoint_txg)
-                    .map(|(_, _, root)| root.record_bytes.clone()),
             },
         )
         .collect()
@@ -6554,7 +6557,7 @@ fn replay_the_next_mount_would_do<'records>(
 
 /// [`TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE`]（I-4.1）：所选根（重放之后那一版）指向的树全可遍历。
 /// 所选根取根环里 (txg, 实例) 最大的那条（与 I-7.2 同一个择法）；重放照 [`replay_the_next_mount_would_do`]；施加出来的那一版 =
-/// 所选根换上末条记录的新根段四段，实例表指针与树表 0 条那一版的分配记录树的根照所选根（已定项 15，与实现恢复拼新根同一个读法），
+/// 所选根换上末条记录的新根段七段（实例表指针与树表 0 条那一版的分配记录树的根也从新根段来，已定项 15，与实现恢复拼新根同一个读法），
 /// 用一份另起的走读走一遍（判定丢掉，那些归别的不变量），走读断了（任一单元两份都读不对、头用不了、条目解不开）判红。
 /// 一条都不施加、锚点认不定、所选根 txg 没有后继、前缀停在点名单元验不过（起点含介质损坏）都报不适用。
 fn judge_trees_of_the_replayed_version_are_traversable(
@@ -6601,9 +6604,6 @@ fn judge_trees_of_the_replayed_version_are_traversable(
                 instance: last_record.instance,
                 checkpoint_txg: last_record.checkpoint_txg,
                 new_root_segment: last_record.new_root_segment.clone(),
-                root_record_of_the_version_it_was_applied_on: Some(
-                    chosen_root.record_bytes.clone(),
-                ),
             };
             let mut walk_of_the_replayed_version = Walk::starting_with(
                 reader,
@@ -6973,9 +6973,15 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
             highest_floor_on_the_newest_valid_and_the_abandoned_roots,
             u64::max,
         );
-    // 掉出遍历的根槽按理由各记一次（两样都占的两边都记）：I-3.1 判红时这几个数就是「遍历为什么少算」的机理标识。
+    // 一圈窗口（D16（发布语义） 已定项 1）：N 取最新根（它按自己的实例表恒有效），3S 是根环槽数。
+    let window = OneRevolutionWindow {
+        newest_valid_root_txg: newest_txg,
+        root_ring_slot_count: geometry.regions * geometry.slots_per_region,
+    };
+    // 掉出遍历的根槽按理由各记一次（几样都占的各边都记）：I-3.1 判红时这几个数就是「遍历为什么少算」的机理标识。
     let mut root_slots_dropped_as_abandoned = 0u64;
     let mut root_slots_dropped_below_floor = 0u64;
+    let mut root_slots_dropped_outside_the_window = 0u64;
     let candidate_indexes: Vec<usize> = roots
         .iter()
         .enumerate()
@@ -6983,7 +6989,10 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
             // 被抛弃：按最新根指着的实例表判（D23（journal 的角色与格式） 已定项 14 候选集）。
             let abandoned = abandoned_by_the_newest_roots_table(root);
             let below_floor = root.checkpoint_txg < effective_rollback_floor;
-            let walked = *index == newest_index || (!abandoned && !below_floor);
+            // 一圈窗口之外（D16（发布语义） 已定项 1，五处同改之一）：环上有洞时洞那一槽里留着的上一圈的根。
+            let outside_the_window = !window.contains(root.checkpoint_txg);
+            let walked =
+                *index == newest_index || (!abandoned && !below_floor && !outside_the_window);
             if !walked {
                 if abandoned {
                     root_slots_dropped_as_abandoned += 1;
@@ -6991,6 +7000,9 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
                 if below_floor {
                     root_slots_dropped_below_floor += 1;
                 }
+                if outside_the_window {
+                    root_slots_dropped_outside_the_window += 1;
+                }
             }
             walked
         })
@@ -7028,6 +7040,7 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
         &roots,
         &instance_table_rows,
         effective_rollback_floor,
+        window,
     );
     for version in &versions_applied_only_by_records {
         let mismatches_before = walk.judgements.violation_count("I-2.1");
@@ -7238,7 +7251,7 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
     // 不看各段的次序，所以插在「遍历的候选根槽」之后、与它挨着读。
     let mechanism = move || {
         format!(
-            "；机理：根环槽数 {root_ring_slot_count}、最新根 txg {newest_txg}、环里自证过的根槽 {readable_root_slot_count} 个、最老的自证过的根 txg {oldest_readable_root_txg}、遍历的候选根槽 {walked_root_slot_count} 个、并进遍历的由记录施加出来的版本 {walked_versions_applied_only_by_records} 个、被实例表判抛弃的根槽 {root_slots_dropped_as_abandoned} 个、回退下界 F {effective_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个"
+            "；机理：根环槽数 {root_ring_slot_count}、最新根 txg {newest_txg}、环里自证过的根槽 {readable_root_slot_count} 个、最老的自证过的根 txg {oldest_readable_root_txg}、遍历的候选根槽 {walked_root_slot_count} 个、并进遍历的由记录施加出来的版本 {walked_versions_applied_only_by_records} 个、被实例表判抛弃的根槽 {root_slots_dropped_as_abandoned} 个、回退下界 F {effective_rollback_floor}、低于 F 的根槽 {root_slots_dropped_below_floor} 个、一圈窗口之外的根槽 {root_slots_dropped_outside_the_window} 个"
         )
     };
     // I-3.1 / I-5.2：最新根下面记账树的「已分配」「空闲」逐盘对遍历得到的和与容量。
@@ -7274,6 +7287,23 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
                 && root.checkpoint_txg < effective_rollback_floor
         })
         .collect();
+    // 丁-defer 的门槛与分配器的回收门槛同一个：max(F_生效, 一圈窗口里最旧有效根)（D16（发布语义） 已定项 1，五处同改之一；
+    // `research/prompts/m3-ring-r3-main-verification.md` 第二节第 5 行）——释放代落在 (F_生效, 窗口里最旧有效根] 的槽分配器已经回收、
+    // 不在「已分配」里，再按 F_生效 并进来就多算。
+    let deferred_threshold = roots
+        .iter()
+        .map(|(_, _, root)| root)
+        .filter(|root| {
+            !abandoned_by_the_newest_roots_table(root) && window.contains(root.checkpoint_txg)
+        })
+        .map(|root| root.checkpoint_txg)
+        .min()
+        .map_or(
+            effective_rollback_floor,
+            |oldest_valid_root_in_the_window| {
+                oldest_valid_root_in_the_window.max(effective_rollback_floor)
+            },
+        );
     let deferred_below_the_floor_against_every_walked_version =
         deferred_slots_referenced_only_below_the_floor_per_device(
             reader,
@@ -7281,7 +7311,7 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
             newest_root_view,
             &roots_below_the_floor,
             &walked_slots_of_every_walked_version,
-            effective_rollback_floor,
+            deferred_threshold,
             &mut index_node_cache,
         );
     let quarantined_exempted_against_the_newest_root = quarantined_slots_exempted_per_device(
@@ -7454,4 +7484,85 @@ mod tests {
             );
         }
     }
+
+    /// 一条只带键、反向链与链值的扫描记录（别的字段这几条用例不读）。
+    fn scanned_record_of(
+        instance: u32,
+        back_chain: u32,
+        chain_value_the_next_record_must_carry: u32,
+    ) -> ScannedJournalRecord {
+        ScannedJournalRecord {
+            instance,
+            checkpoint_txg: 0,
+            new_root_segment: Vec::new(),
+            transaction: 0,
+            commit_marker: CommitMarker::Present,
+            ordinal_within_publish: 1,
+            record_flags_byte: RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH,
+            slot: 0,
+            back_chain,
+            chain_value_the_next_record_must_carry,
+            named_entry_locations: Vec::new(),
+            named_entries: Vec::new(),
+        }
+    }
+
+    /// 两块盘同一计数器坐着不同实例的记录（盘 0 计数器 4 是实例 2 的，盘 1 计数器 4 是实例 1 的）：恢复眼里的环按 (实例代号, 计数器)
+    /// 建键，两条都留下，不拿先读到的盘 0 那条挤掉实例 1 的（I-4.1 那一行：`records_the_replay_reads` 先改、这一处同改）。
+    #[test]
+    fn the_ring_as_the_recovery_reads_it_keeps_both_records_of_two_instances_at_the_same_counter() {
+        let records_by_device = vec![
+            (0, BTreeMap::from([(4, scanned_record_of(2, 0, 0x22))])),
+            (1, BTreeMap::from([(4, scanned_record_of(1, 0x0b, 0x11))])),
+        ];
+        let ring = records_as_the_recovery_reads_them(&records_by_device);
+        assert_eq!(
+            ring.keys().copied().collect::<Vec<_>>(),
+            vec![(1, 4), (2, 4)],
+            "两条都在，按 (实例代号, 计数器) 认"
+        );
+        assert_eq!(
+            ring[&(1, 4)].chain_value_the_next_record_must_carry,
+            0x11,
+            "实例 1 那一条是盘 1 上那一份"
+        );
+    }
+
+    /// 反向链那一关按 (实例代号, 计数器 − 1) 找本实例内逻辑前一条：计数器 3 上先读到的是实例 2 的记录、实例 1 的那一条在盘 1 上，
+    /// 实例 1 计数器 4 那一条的反向链与实例 1 计数器 3 那条的链值不等 ⇒ 进不了前缀；对得上 ⇒ 进得去。
+    /// 按计数器找时先读到的是实例 2 的那条，当成「前一条是别的实例」放过去。
+    #[test]
+    fn the_back_chain_gate_looks_up_the_predecessor_of_the_same_instance_past_another_instance_at_that_counter(
+    ) {
+        let records_by_device = vec![
+            (0, BTreeMap::from([(3, scanned_record_of(2, 0, 0x22))])),
+            (1, BTreeMap::from([(3, scanned_record_of(1, 0, 0x11))])),
+        ];
+        let ring = records_as_the_recovery_reads_them(&records_by_device);
+        assert!(
+            !record_can_enter_a_replay_prefix(&ring, 4, &scanned_record_of(1, 0x12, 0)),
+            "实例 1 计数器 4 的反向链 0x12 与实例 1 计数器 3 的链值 0x11 不等：不进前缀"
+        );
+        assert!(
+            record_can_enter_a_replay_prefix(&ring, 4, &scanned_record_of(1, 0x11, 0)),
+            "对得上：进前缀"
+        );
+    }
+
+    /// 一圈窗口（D16（发布语义） 已定项 1）：N = 35、环长 24 ⇒ 窗口 = txg > 11；N 小于环长时整个环都在窗口里。
+    #[test]
+    fn the_one_revolution_window_keeps_the_txgs_above_the_newest_minus_the_ring_length() {
+        let window = OneRevolutionWindow {
+            newest_valid_root_txg: 35,
+            root_ring_slot_count: 24,
+        };
+        assert!(!window.contains(11), "35 − 24 = 11 不在窗口里");
+        assert!(window.contains(12), "12 在窗口里");
+        assert!(window.contains(35), "N 自己在窗口里");
+        let young_ring = OneRevolutionWindow {
+            newest_valid_root_txg: 7,
+            root_ring_slot_count: 24,
+        };
+        assert!(young_ring.contains(0), "N < 环长：第 0 代也在窗口里");
+    }
 }
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 677d118..0b98097 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -20,7 +20,10 @@ use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
 use crate::admission::SpaceAdmission;
 use crate::bytes::ByteWriter;
 use crate::journal::slot_after_the_journal_ring;
-use crate::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
+use crate::root_ring::{
+    target_for_publish, txg_is_inside_the_one_revolution_window, RootRingSlot,
+    RootRingSlotsPerRegion,
+};
 use crate::transaction::FrozenPublish;
 use crate::unlink::DeletionSwitches;
 
@@ -939,10 +942,18 @@ impl PlacementOnDevice {
     }
 }
 
-/// 根环一个槽上这会儿住着哪条根。读不出、自证不过的槽不在 `RootRingOccupancy` 里：D16（发布语义） 已定项 1 的
-/// 「环里最旧有效根」只看自证合法的根，读不出的被抛弃根影子账本来就罩不到（`mount::isolate_slots_referenced_only_by_abandoned_roots`）。
+/// 根环一个槽上这会儿住着哪条根。自证不过的槽、读不出而落在一圈窗口之外的槽不在 `RootRingOccupancy` 里：D16（发布语义） 已定项 1 的
+/// 「一圈窗口里最旧有效根」只看自证合法的根与窗口里读不出的槽，读不出的被抛弃根影子账本来就罩不到（`mount::isolate_slots_referenced_only_by_abandoned_roots`）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub enum RootRingOccupant {
+    /// 挂载那一刻设备报读错的槽，落在挂载时的一圈窗口里（D16（发布语义） 已定项 1「挂载时读不出的槽在窗口里只对应一两个 txg：
+    /// 门槛按最小的算，根环表记最大的」，`research/prompts/m3-ring-r3-main-verification.md` 第二节第 4 行，零轮）：按「那里住着一条 txg 为
+    /// `highest_txg_it_can_hold` 的有效根」进这个进程的回收门槛，直到这个进程把那个槽写掉。它不算「这个进程知道住着根的槽」
+    /// （[`RootRingOccupancy::ring_slots_known_to_hold_a_root`]）：D16 已定项 1「根槽这一次读坏」那一行，挂载那一刻就读不出、之后没写过的槽
+    /// 算上限、判回退候选时当没有根，不因为这里记了它就改成「读坏就拒」。
+    UnreadableAtMountInsideTheWindow {
+        highest_txg_it_can_hold: CheckpointTxg,
+    },
     /// 按实例表判仍然有效的根，txg 在不在 F 之下都算（可再分配谓词取 max(F_生效, 环里最旧有效根)，F 那一半另记）。
     ValidRoot { checkpoint_txg: CheckpointTxg },
     /// 按实例表判被抛弃的根（D23（journal 的角色与格式） 已定项 14），带着它引用的每个落点——挂载那一刻仍被候选根或现行账引用、
@@ -995,20 +1006,40 @@ impl RootRingOccupancy {
 
     /// 这个进程知道住着一条根的根环槽（D16（发布语义） 已定项 1「根槽这一次读坏」那一行）：挂载那一刻读得出、自证过的槽
     /// （有效的与被抛弃的都算），与这个进程之后写过、FUA 返回过的槽——发布在落盘之前或落盘途中失败时分配器整个换回发布之前那一份，
-    /// 这张表随之退回，记着的只剩根落盘做成的那几次。挂载那一刻读不出或自证不过、之后又没写过的槽不在其内。
+    /// 这张表随之退回，记着的只剩根落盘做成的那几次。挂载那一刻读不出或自证不过、之后又没写过的槽不在其内
+    /// （[`RootRingOccupant::UnreadableAtMountInsideTheWindow`] 记着的槽也不在）。
     #[must_use]
     pub fn ring_slots_known_to_hold_a_root(&self) -> BTreeSet<RootRingSlot> {
-        self.occupants.keys().copied().collect()
+        self.occupants
+            .iter()
+            .filter(|(_, occupant)| match occupant {
+                RootRingOccupant::ValidRoot { .. } | RootRingOccupant::AbandonedRoot { .. } => true,
+                RootRingOccupant::UnreadableAtMountInsideTheWindow { .. } => false,
+            })
+            .map(|(ring_slot, _)| *ring_slot)
+            .collect()
     }
 
-    /// 环里最旧有效根的 txg；一条有效根都没有时 `None`。
-    fn oldest_valid_root(&self) -> Option<CheckpointTxg> {
+    /// 一圈窗口里最旧有效根的 txg（D16（发布语义） 已定项 1）：N = 这个进程记到的最新根的 txg（`latest_recorded_txg`：进程内每记一条根
+    /// 就是这条根），窗口 = txg > N − 3S（3S 是环长）；窗口里的有效根与挂载时窗口里读不出的槽（按它能住的最大 txg）取最小。一条都没有时 `None`。
+    /// 挂载那一刻的门槛不走这里（`mount::rebuilt_allocator` 锚在环里读得出的最新有效根、读不出的槽按最小 txg 算）。
+    fn oldest_valid_root_in_the_window(&self) -> Option<CheckpointTxg> {
         self.occupants
             .values()
             .filter_map(|occupant| match occupant {
                 RootRingOccupant::ValidRoot { checkpoint_txg, .. } => Some(*checkpoint_txg),
+                RootRingOccupant::UnreadableAtMountInsideTheWindow {
+                    highest_txg_it_can_hold,
+                } => Some(*highest_txg_it_can_hold),
                 RootRingOccupant::AbandonedRoot { .. } => None,
             })
+            .filter(|checkpoint_txg| {
+                txg_is_inside_the_one_revolution_window(
+                    *checkpoint_txg,
+                    self.latest_recorded_txg,
+                    self.slots_per_region,
+                )
+            })
             .min()
     }
 
@@ -1020,7 +1051,8 @@ impl RootRingOccupancy {
                 RootRingOccupant::AbandonedRoot {
                     referenced_placements,
                 } => referenced_placements.as_slice(),
-                RootRingOccupant::ValidRoot { .. } => &[],
+                RootRingOccupant::ValidRoot { .. }
+                | RootRingOccupant::UnreadableAtMountInsideTheWindow { .. } => &[],
             })
             .copied()
             .collect()
@@ -1672,9 +1704,11 @@ impl PoolAllocator {
         let displaced = occupancy
             .occupants
             .insert(ring_slot, RootRingOccupant::ValidRoot { checkpoint_txg });
-        // 与挂载时同一个谓词（一处定义）：max(F_生效, 环里最旧有效根)。
-        let reclaim_floor =
-            crate::mount::reclaim_floor(occupancy.effective_floor, occupancy.oldest_valid_root());
+        // 与挂载时同一个谓词（一处定义）：max(F_生效, 一圈窗口里最旧有效根)。
+        let reclaim_floor = crate::mount::reclaim_floor(
+            occupancy.effective_floor,
+            occupancy.oldest_valid_root_in_the_window(),
+        );
         let still_referenced_by_abandoned_roots =
             occupancy.placements_referenced_by_abandoned_roots();
         match displaced {
@@ -1684,7 +1718,11 @@ impl PoolAllocator {
                 &referenced_placements,
                 &still_referenced_by_abandoned_roots,
             ),
-            Some(RootRingOccupant::ValidRoot { .. }) | None => {}
+            Some(
+                RootRingOccupant::ValidRoot { .. }
+                | RootRingOccupant::UnreadableAtMountInsideTheWindow { .. },
+            )
+            | None => {}
         }
         let reclaimed =
             self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);
diff --git a/crates/singlefs-core/src/journal.rs b/crates/singlefs-core/src/journal.rs
index 56e5352..a3d850c 100644
--- a/crates/singlefs-core/src/journal.rs
+++ b/crates/singlefs-core/src/journal.rs
@@ -1,6 +1,6 @@
 //! journal 记录 4096（D23（journal 的角色与格式） 已定项 4 / 已定项 7 / 已定项 8 / 已定项 13 / 已定项 15 / 已定项 17 / 已定项 18 / 已定项 19）：
-//! 头 311 + 点名项 56 × N；`header_csum` 罩整条 [0, 4096) 含补齐、自身按 0 参与；`payload_csum` 罩点名项数组；
-//! 反向链 = CRC-32C(本实例内逻辑前一条记录的 311 字节头，`header_csum` 按 0 参与)，本实例第一条恒 0。
+//! 头 487 + 点名项 56 × N；`header_csum` 罩整条 [0, 4096) 含补齐、自身按 0 参与；`payload_csum` 罩点名项数组；
+//! 反向链 = CRC-32C(本实例内逻辑前一条记录的 487 字节头，`header_csum` 按 0 参与)，本实例第一条恒 0。
 
 use singlefs_format::{
     JOURNAL_HEADER_BYTES, JOURNAL_NAMED_ENTRY_BYTES, JOURNAL_NEW_ROOT_SEGMENT_BYTES,
@@ -16,6 +16,7 @@ use crate::checksum::{
     crc32_castagnoli, wide_checksum_field_holds, wide_checksum_with_field_zeroed,
 };
 use crate::pointer::{LocationEntry, NodePointer};
+use crate::root_record::UnmountMarker;
 
 pub const JOURNAL_MAGIC: [u8; 4] = *b"SFSJ";
 /// 记录类型登记表（D23（journal 的角色与格式） 已定项 1）：0 无效、1 普通记录。
@@ -160,14 +161,19 @@ pub struct JournalRecord {
     pub new_mapping_root: NodePointer,
     pub new_tree_identifier_watermark: u64,
     pub new_rollback_floor: CheckpointTxg,
+    /// 新根段后三项（D23（journal 的角色与格式） 已定项 15，用户 2026-10-02「回到甲」）：这次发布写的根上的实例表指针、
+    /// 分配记录树根指针与卸载记号。施加一条记录时这三项与前四项一样换掉所选根的（`recovery::replay_journal` 拼新根那一截），不再照抄所选根。
+    pub new_instance_table: NodePointer,
+    pub new_allocation_record_tree_root: NodePointer,
+    pub new_unmount_marker: UnmountMarker,
     pub named: Vec<NamedUnit>,
 }
 
-/// 反向链：前一条记录的整个记录头（[`JOURNAL_HEADER_BYTES`] = 311 字节），`header_csum` 那 32 字节按 0 参与。
+/// 反向链：前一条记录的整个记录头（[`JOURNAL_HEADER_BYTES`] = 487 字节），`header_csum` 那 32 字节按 0 参与。
 #[must_use]
 pub fn back_chain_of(previous_record_bytes: &[u8]) -> u32 {
     let mut header =
-        previous_record_bytes[..usize::try_from(JOURNAL_HEADER_BYTES).expect("311")].to_vec();
+        previous_record_bytes[..usize::try_from(JOURNAL_HEADER_BYTES).expect("487")].to_vec();
     header[JOURNAL_HEADER_CHECKSUM_OFFSET
         ..JOURNAL_HEADER_CHECKSUM_OFFSET + usize::try_from(WIDE_CHECKSUM_BYTES).expect("32")]
         .fill(0);
@@ -240,9 +246,13 @@ impl JournalRecord {
         self.new_mapping_root.write_to(&mut writer);
         writer.put_u64(self.new_tree_identifier_watermark);
         writer.put_u64(self.new_rollback_floor.0);
+        self.new_instance_table.write_to(&mut writer);
+        self.new_allocation_record_tree_root.write_to(&mut writer);
+        // 卸载记号 4 字节与根记录 flags 同一个编码（位 0 = 卸载那一串写的，其余位恒 0；D22（单元原子性怎么合成） 已定项 7 / 已定项 13）。
+        writer.put_u32(self.new_unmount_marker.flags_bits());
         assert_eq!(
             writer.position() - new_root_segment_start,
-            usize::try_from(JOURNAL_NEW_ROOT_SEGMENT_BYTES).expect("188")
+            usize::try_from(JOURNAL_NEW_ROOT_SEGMENT_BYTES).expect("364")
         );
         writer.put_u64(self.filesystem_identifier);
         writer.skip(16); // MAC：第一版留位全 0
@@ -253,7 +263,7 @@ impl JournalRecord {
         let payload_end = writer.position();
         let mut bytes = writer.into_bytes();
         let payload_checksum = crc32_castagnoli(
-            &bytes[usize::try_from(JOURNAL_HEADER_BYTES).expect("311")..payload_end],
+            &bytes[usize::try_from(JOURNAL_HEADER_BYTES).expect("487")..payload_end],
         );
         bytes[payload_checksum_offset..payload_checksum_offset + 4]
             .copy_from_slice(&payload_checksum.to_le_bytes());
@@ -265,8 +275,9 @@ impl JournalRecord {
     }
 
     /// 读者：magic、类型、整条校验和、fsid、载荷校验和四关，外加四条当损坏的（D23（journal 的角色与格式） 已定项 4 / 已定项 7；最后一条是代码审阅第 29 条）：
-    /// 记录标志位 0 之外有位为 1、本次发布内序号为 0、提交标记不是 0 也不是 1，另有新根段两条指针头部里加密、压缩与 extent 偏移那几段不是第一版的取值
-    /// （代码审阅第 29 条）。当损坏就是与校验和不过同一个结局——这条记录不算在，前缀在它之前断
+    /// 记录标志位 0 之外有位为 1、本次发布内序号为 0、提交标记不是 0 也不是 1，另有新根段四条指针头部里加密、压缩与 extent 偏移那几段不是第一版的取值
+    /// （代码审阅第 29 条），以及新根段的卸载记号那 4 字节位 0 之外有位为 1（与根记录 flags 同判，`root_record::parse_slot`；条款没写这一格，
+    /// 照根记录与记录标志「其余位读到非 0 当损坏」的同一个处置取）。当损坏就是与校验和不过同一个结局——这条记录不算在，前缀在它之前断
     /// （已定项 22 断号即止）。不查反向链，也不查一次发布之内跳不跳号：那两样要看别的记录，是前缀取法的事。
     #[must_use]
     pub fn parse(bytes: &[u8], expected_filesystem_identifier: u64) -> Option<Self> {
@@ -321,6 +332,18 @@ impl JournalRecord {
             .ok()?;
         let new_tree_identifier_watermark = reader.get_u64();
         let new_rollback_floor = CheckpointTxg(reader.get_u64());
+        let new_instance_table =
+            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+                &mut reader,
+            )
+            .ok()?;
+        let new_allocation_record_tree_root =
+            NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
+                &mut reader,
+            )
+            .ok()?;
+        let new_unmount_marker =
+            UnmountMarker::of_unmount_marker_field_of_a_new_root_segment(reader.get_u32())?;
         let filesystem_identifier = reader.get_u64();
         if filesystem_identifier != expected_filesystem_identifier {
             return None;
@@ -328,7 +351,7 @@ impl JournalRecord {
         reader.skip(16);
         assert_eq!(
             reader.position(),
-            usize::try_from(JOURNAL_HEADER_BYTES).expect("311")
+            usize::try_from(JOURNAL_HEADER_BYTES).expect("487")
         );
         let payload_end = reader.position()
             + named_count * usize::try_from(JOURNAL_NAMED_ENTRY_BYTES).expect("56");
@@ -354,6 +377,9 @@ impl JournalRecord {
             new_mapping_root,
             new_tree_identifier_watermark,
             new_rollback_floor,
+            new_instance_table,
+            new_allocation_record_tree_root,
+            new_unmount_marker,
             named,
         })
     }
@@ -379,12 +405,15 @@ mod tests {
             new_mapping_root: NodePointer::empty_root(),
             new_tree_identifier_watermark: singlefs_format::TREE_IDENTIFIER_WATERMARK_AT_MKFS,
             new_rollback_floor: CheckpointTxg(0),
+            new_instance_table: NodePointer::empty_root(),
+            new_allocation_record_tree_root: NodePointer::empty_root(),
+            new_unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
             named: Vec::new(),
         }
     }
 
     /// 本次发布内序号 4 字节紧跟事务号 8 与提交标记 1（D23（journal 的角色与格式） 已定项 4）：落在 [87, 91)、小端、读得回；
-    /// 它之后的反向链、载荷校验和、新根段、fsid 依次后挪 4 字节，MAC 16 占记录头最后的 [295, 311)。偏移按字段表手算、写成字面量，
+    /// 它之后的反向链、载荷校验和、新根段依次后挪 4 字节；新根段 364 字节之后是 fsid [463, 471)，MAC 16 占记录头最后的 [471, 487)。偏移按字段表手算、写成字面量，
     /// 不从写者用的常量推（已定项 26 第 1 条：覆盖范围是显式的盘上常量）。
     #[test]
     fn the_ordinal_within_publish_sits_right_after_the_commit_marker_and_round_trips() {
@@ -418,17 +447,101 @@ mod tests {
             "新根段从 99 起：两条指针 86 × 2 之后是树 ID 水位"
         );
         assert_eq!(
-            &bytes[287..295],
+            &bytes[463..471],
             &7u64.to_le_bytes(),
-            "fsid 后挪到 [287, 295)"
+            "fsid 在新根段之后，[463, 471)"
         );
         assert!(
-            bytes[295..311].iter().all(|byte| *byte == 0),
-            "MAC 16 占 [295, 311)，第一版全 0"
+            bytes[471..487].iter().all(|byte| *byte == 0),
+            "MAC 16 占 [471, 487)，第一版全 0"
         );
         assert_eq!(JournalRecord::parse(&bytes, 7), Some(record));
     }
 
+    /// 新根段后三项（D23（journal 的角色与格式） 已定项 4 / 已定项 15）：F 之后依次是实例表指针 [287, 373)、分配记录树根指针 [373, 459)、
+    /// 卸载记号 [459, 463)（与根记录 flags 同一个编码，位 0）；三项各取一个与别的字段都不同的值，写得出、读得回。
+    /// 偏移按字段表手算、写成字面量，不从写者用的常量推（已定项 26 第 1 条）。
+    #[test]
+    fn the_instance_table_allocation_record_tree_root_and_unmount_marker_follow_the_floor_in_the_new_root_segment_and_round_trip(
+    ) {
+        let instance_table_location = |device: u32| LocationEntry {
+            device: crate::address::DeviceIdentity(device),
+            slot: crate::address::SlotNumber(60_001),
+            unit_checksum: 0x0a0b_0c0d,
+        };
+        let allocation_record_tree_location = |device: u32| LocationEntry {
+            device: crate::address::DeviceIdentity(device),
+            slot: crate::address::SlotNumber(60_002),
+            unit_checksum: 0x1a1b_1c1d,
+        };
+        let pointer_with = |locations: [LocationEntry; 2], birth_txg: u64| NodePointer {
+            head: crate::pointer::PointerHead {
+                birth_tree: TreeIdentifier(0),
+                birth_txg: CheckpointTxg(birth_txg),
+            },
+            locations,
+            instance: InstanceGeneration(1),
+            birth_sequence: crate::pointer::BirthSequence(3),
+        };
+        let record = JournalRecord {
+            new_instance_table: pointer_with(
+                [instance_table_location(0), instance_table_location(1)],
+                5,
+            ),
+            new_allocation_record_tree_root: pointer_with(
+                [
+                    allocation_record_tree_location(0),
+                    allocation_record_tree_location(1),
+                ],
+                6,
+            ),
+            new_unmount_marker: UnmountMarker::WrittenByTheUnmountSequence,
+            ..empty_record(9)
+        };
+        let bytes = record.to_bytes();
+        let mut instance_table_bytes = ByteWriter::new(86);
+        record
+            .new_instance_table
+            .write_to(&mut instance_table_bytes);
+        assert_eq!(
+            &bytes[287..373],
+            instance_table_bytes.into_bytes().as_slice(),
+            "实例表指针紧跟 F（99 + 188），在 [287, 373)"
+        );
+        let mut allocation_record_tree_bytes = ByteWriter::new(86);
+        record
+            .new_allocation_record_tree_root
+            .write_to(&mut allocation_record_tree_bytes);
+        assert_eq!(
+            &bytes[373..459],
+            allocation_record_tree_bytes.into_bytes().as_slice(),
+            "分配记录树根指针在 [373, 459)"
+        );
+        assert_eq!(
+            &bytes[459..463],
+            &[0x01, 0x00, 0x00, 0x00],
+            "卸载记号在 [459, 463)：卸载那一串写的，位 0 = 1"
+        );
+        assert_eq!(JournalRecord::parse(&bytes, 7), Some(record));
+    }
+
+    /// 新根段的卸载记号那 4 字节位 0 之外有位为 1 的记录，读者当损坏（与根记录 flags、记录标志「其余位读到非 0」同一个处置）。
+    /// 头部校验和按改过的字节重封过，拦住它的只有卸载记号那一判。
+    #[test]
+    fn record_whose_new_root_segment_unmount_marker_sets_any_bit_other_than_bit_zero_is_refused_by_the_parser(
+    ) {
+        for unmount_marker_field in [0x0000_0002u32, 0x0000_0003, 0x8000_0000] {
+            let mut bytes = empty_record(4).to_bytes();
+            bytes[459..463].copy_from_slice(&unmount_marker_field.to_le_bytes());
+            reseal_header_checksum(&mut bytes);
+            assert_eq!(
+                JournalRecord::parse(&bytes, 7),
+                None,
+                "卸载记号 {unmount_marker_field:#010x}：位 0 之外有位为 1，当损坏"
+            );
+        }
+    }
+
     /// 改过记录头某几个字节之后重封头部校验和（罩整条 4096、自身按 0 参与，D23（journal 的角色与格式） 已定项 13）：
     /// 拦得住那条记录的只剩被改的字段本身。
     fn reseal_header_checksum(bytes: &mut [u8]) {
@@ -510,9 +623,9 @@ mod tests {
         );
     }
 
-    /// 一条 4096 字节记录装 67 个点名项（(4096 − 311) ÷ 56 = 3785 ÷ 56 取整）：67 项的记录写得出、读得回；头里自述 68 项的记录
+    /// 一条 4096 字节记录装 64 个点名项（(4096 − 487) ÷ 56 = 3609 ÷ 56 取整）：64 项的记录写得出、读得回；头里自述 65 项的记录
     /// （点名项区越过记录末尾）读者拒收，不按自述的项数切到 4096 之外（里程碑「覆盖写、释放、回退与复用」并行线一验收第 4 条第三个变异：
-    /// 点名项超过 67 仍塞进一条记录 ⇒ 记录解析拒收）。头校验和按改过的字节重算过，拦住它的只有项数那一判。
+    /// 点名项超过一条记录装得下的仍塞进一条记录 ⇒ 记录解析拒收）。头校验和按改过的字节重算过，拦住它的只有项数那一判。
     #[test]
     fn record_whose_header_claims_more_named_units_than_one_record_holds_is_refused_by_the_parser()
     {
@@ -535,7 +648,7 @@ mod tests {
             birth_txg: CheckpointTxg(3),
             key_tail: [0; 10],
         };
-        let capacity = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("67");
+        let capacity = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("64");
         let full = JournalRecord {
             named: vec![named_unit; capacity],
             ..empty_record(1)
@@ -544,12 +657,12 @@ mod tests {
         assert_eq!(
             JournalRecord::parse(&full_bytes, 7),
             Some(full),
-            "67 项正好装满一条记录"
+            "64 项正好装满一条记录"
         );
         let mut claims_one_more = full_bytes;
         let named_count_offset = 4 + 2 + 1 + 1 + 4;
         claims_one_more[named_count_offset..named_count_offset + 4]
-            .copy_from_slice(&u32::try_from(capacity + 1).expect("68").to_le_bytes());
+            .copy_from_slice(&u32::try_from(capacity + 1).expect("65").to_le_bytes());
         let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
         let digest = wide_checksum_with_field_zeroed(
             &claims_one_more,
@@ -561,18 +674,18 @@ mod tests {
         assert_eq!(
             JournalRecord::parse(&claims_one_more, 7),
             None,
-            "自述 68 个点名项的记录：点名项区越过 4096，读者拒收"
+            "自述 65 个点名项的记录：点名项区越过 4096，读者拒收"
         );
     }
 
     #[test]
-    fn record_is_4096_with_a_311_byte_header_and_round_trips() {
+    fn record_is_4096_with_a_487_byte_header_and_round_trips() {
         let record = empty_record(1);
         let bytes = record.to_bytes();
         assert_eq!(bytes.len(), 4096);
         assert!(
-            bytes[311..].iter().all(|byte| *byte == 0),
-            "空记录 311 之后全 0"
+            bytes[487..].iter().all(|byte| *byte == 0),
+            "空记录 487 之后全 0"
         );
         assert_eq!(JournalRecord::parse(&bytes, 7), Some(record.clone()));
         assert_eq!(
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index 39da40a..283636c 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -46,18 +46,21 @@ use crate::recovery::{
     effective_rollback_floor_under_the_newest_roots_table, highest_root_txg,
     highest_tree_identifier_watermark_in_the_ring, instance_table_chain_of_root,
     instance_table_of_root, instance_table_page_pointers_as_far_as_readable, readable_roots,
-    readable_roots_rereading_ring_slots_known_to_hold_a_root_once, readable_roots_with_ring_slots,
+    readable_roots_rereading_ring_slots_known_to_hold_a_root_once,
     rebuild_version_in_the_unit_area_starting_at, replay_journal,
-    root_is_abandoned_by_the_instance_table, scan_journal, tree_table_has_no_entries,
+    root_is_abandoned_by_the_instance_table, scan_journal,
+    system_configuration_slots_of_one_device_read_once,
+    system_configuration_witness_is_undecidable, tree_table_has_no_entries,
     user_visible_tree_root_pointers, verified_system_configuration_slots, BadRootRingSlotReading,
     EffectiveFloorReadingStillUnreadableAfterOneReread, InstanceTableChain,
     InstanceTableOfTheNewestRootStillUnreadableAfterOneReread, JournalScanReport, PoolReader,
     RebuildVersionFailure, RebuiltVersion, RecoveryFailure,
-    RootRingSlotKnownToHoldARootStillBadAfterOneReread, UserVisibleTreeRootPointers,
+    RootRingSlotKnownToHoldARootStillBadAfterOneReread, SystemConfigurationSlotsOfOneDevice,
+    UserVisibleTreeRootPointers,
 };
 use crate::recovery::{every_root_ring_slot, read_root_ring_slot, RootRingSlotReading};
 use crate::root_record::{RootRecord, UnmountMarker};
-use crate::root_ring::{target_for_publish, RootRingSlot};
+use crate::root_ring::{target_for_publish, txg_is_inside_the_one_revolution_window, RootRingSlot};
 use crate::system_configuration::SystemImmutableSizes;
 use crate::transaction::{
     acquire_expected_instance, central_mapping_locations_in_the_version,
@@ -78,10 +81,7 @@ use crate::transaction::{
 };
 use crate::unlink::DeletionSwitches;
 use crate::write_accounting::WritesByStructureKind;
-use singlefs_format::{
-    DATA_UNIT_BYTES, NODE_BYTES, ROOT_RING_REGIONS, SLOT_BYTES,
-    SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
-};
+use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES, ROOT_RING_REGIONS, SLOT_BYTES};
 use std::borrow::Cow;
 use std::collections::{BTreeMap, BTreeSet};
 
@@ -407,7 +407,10 @@ impl SelectedVersionAgainstTheWitness {
                     )
             }
             WitnessedCounterComparison::Undecidable
-            | WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified => true,
+            | WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots
+            | WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot => {
+                true
+            }
         }
     }
 }
@@ -435,10 +438,14 @@ pub enum WitnessedCounterComparison {
     AgainstTheRecordAtTheWitnessedCounter { record: RollbackTarget },
     /// 两条都读不出：判不出，按判据为真处置（登记里的标签 `undecidable`）。
     Undecidable,
-    /// 见证读里有一个系统配置槽读不出或自证不过：c_见证 缺了那一槽，判不出，按判据为真处置
-    /// （C331（择根倒挂压过已确认的写） 取甲，用户 2026-09-28 定；`research/prompts/unreadable-at-mount-r2-main-verification.md` L2）。
-    /// 撕裂的系统配置轮换也落在这里：代价是那之后只剩只读挂载（同一判决 L1）。
-    SomeSystemConfigurationSlotUnreadOrUnverified,
+    /// 见证读判不出（R2，`recovery::system_configuration_witness_is_undecidable`）：有槽设备报读错，且没有一块盘两槽都自证过——
+    /// c_见证 可能缺了那一槽，按判据为真处置（C331（择根倒挂压过已确认的写） 改取 R2，用户 2026-10-03 定，零轮；
+    /// D22（单元原子性怎么合成） 已定项 8 射程）。撕裂的系统配置轮换（读得出、自证不过，设备没报错）不落在这里，照常取自证过的槽比。
+    SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots,
+    /// 读阶段判完之后，逐盘核或取号那一遍读到某块盘两槽里一份本池自证过的系统配置都没有（C554 乙-配置续 Q1：这块盘不「可见」，
+    /// D18（块里携带什么信息） 已定项 11）：只由 [`refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage`]
+    /// 报，按判据为真处置。R2 之下这一支不并进判不出（一块盘两槽都读不出、另一块盘两槽都自证过时 R2 判得出），单列一个成员。
+    SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot,
 }
 
 /// 可写挂载在「重读一次」之前做什么（D16（发布语义） 已定项 1「根槽这一次读坏」那一行：这一次读不出就重读一次，仍坏就拒）。
@@ -1248,8 +1255,9 @@ fn tree_identifier_watermark_of_the_ring<Device: BlockDevice>(
     })
 }
 
-/// 可再分配谓词的门槛（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）：根环 R × S 槽（S 读自系统配置），第 0 代根被盖之前
-/// 环里最旧有效根恒 0、门槛就是 F_生效；盖掉之后由环里最旧的有效根接管。
+/// 可再分配谓词的门槛（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 一圈窗口里最旧有效根)」）：根环 R × S 槽（S 读自系统配置），第 0 代根被盖之前
+/// 窗口里最旧有效根恒 0、门槛就是 F_生效；盖掉之后由窗口里最旧的有效根接管。调用方交进来的 `oldest_valid_root` 已经按一圈窗口滤过
+/// （挂载时 `rebuilt_allocator`，进程内 `allocator::RootRingOccupancy`，抬 F 时 `raise_rollback_floor`）。
 #[must_use]
 pub fn reclaim_floor(
     effective_floor: CheckpointTxg,
@@ -1325,7 +1333,7 @@ struct ShadowLedgerComputed {
 #[allow(
     clippy::ptr_arg,
     clippy::too_many_arguments,
-    reason = "ptr_arg：allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、分配器、根表、判被抛弃的谓词、下界 F、当前账、重读前的快照"
+    reason = "ptr_arg：allocation_records_under_root 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn；too_many_arguments：每个参数各是一样东西：设备表、单元区起点、分配器、根表、判被抛弃的谓词、下界 F、判落在一圈窗口里的谓词、当前账、重读前的快照"
 )]
 fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
     devices: &Vec<(DeviceIdentity, Device)>,
@@ -1334,6 +1342,7 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
     roots: &[RootRecord],
     is_abandoned: &dyn Fn(&RootRecord) -> bool,
     floor: CheckpointTxg,
+    is_inside_the_window: &dyn Fn(CheckpointTxg) -> bool,
     current_records: &[AllocationRecord],
     before_the_one_reread: &mut BeforeTheOneReread<'_>,
 ) -> Result<ShadowLedgerComputed, MountError> {
@@ -1342,10 +1351,12 @@ fn isolate_slots_referenced_only_by_abandoned_roots<Device: BlockDevice>(
         .filter(|record| !record.is_released)
         .map(|record| (record.device.0, record.slot.0))
         .collect();
-    for root in roots
-        .iter()
-        .filter(|root| !is_abandoned(root) && root.checkpoint_txg >= floor)
-    {
+    // 候选 = 按实例表有效 ∧ txg ≥ F ∧ 落在一圈窗口里（D16（发布语义） 已定项 1：影子账的候选过滤是窗口五处同改之一）。
+    for root in roots.iter().filter(|root| {
+        !is_abandoned(root)
+            && root.checkpoint_txg >= floor
+            && is_inside_the_window(root.checkpoint_txg)
+    }) {
         if let Some(placements) = placements_referenced_by_root(devices, unit_area_start, root) {
             referenced_by_candidates.extend(
                 placements
@@ -1572,6 +1583,66 @@ fn placements_referenced_by_root<Device: BlockDevice>(
     }
 }
 
+/// 挂载那一刻设备报读错（`BadRootRingSlotReading::Unreadable`）、落在一圈窗口里的一个根环槽：它在 (N − 3S, first_txg − 1] 里能住的
+/// 最小与最大 txg（区域内槽位只由 txg 定，D22（单元原子性怎么合成） 已定项 2）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct UnreadableRingSlotInsideTheWindow {
+    ring_slot: RootRingSlot,
+    lowest_txg_it_can_hold: CheckpointTxg,
+    highest_txg_it_can_hold: CheckpointTxg,
+}
+
+/// 挂载时读不出的槽在窗口里只对应一两个 txg：门槛按最小的算，根环表记最大的（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」那一行，
+/// `research/prompts/m3-ring-r3-main-verification.md` 第二节第 4 行，零轮，只在攻方模型上量过）。区间取 (N − 3S, first_txg − 1]：
+/// 读不出的槽里可能住着比环里读得出的最新根还新的根（被藏住的最新根、由记录施加出来的那一版那次发布的根），上沿取新实例第一次发布之前的那个 txg。
+/// 只管设备报读错的槽；读得出而自证不过（没写过、内容坏）的槽照旧当没有根。
+///
+/// 循环：外层按区间里的 txg（轮数 = first_txg − 窗口下沿，至多 3S 加上环里只有记录、没有根的那几个 txg），跨轮只往按槽的表里写最小、最大值；没有提前出口。
+fn unreadable_ring_slots_inside_the_window(
+    ring_slot_readings: &[(RootRingSlot, RootRingSlotReading)],
+    window_anchor: CheckpointTxg,
+    first_txg: CheckpointTxg,
+    slots_per_region: crate::root_ring::RootRingSlotsPerRegion,
+) -> Vec<UnreadableRingSlotInsideTheWindow> {
+    let unreadable_ring_slots: BTreeSet<RootRingSlot> = ring_slot_readings
+        .iter()
+        .filter_map(|(ring_slot, reading)| match reading {
+            RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable) => Some(*ring_slot),
+            RootRingSlotReading::Bad(BadRootRingSlotReading::NotSelfVerified)
+            | RootRingSlotReading::SelfVerified(_) => None,
+        })
+        .collect();
+    let ring_length_in_txgs = ROOT_RING_REGIONS * slots_per_region.count();
+    let lowest_txg_inside_the_window = window_anchor
+        .0
+        .checked_sub(ring_length_in_txgs)
+        .map_or(0, |below_the_window| below_the_window + 1);
+    let mut txgs_of_each_unreadable_slot: BTreeMap<RootRingSlot, (CheckpointTxg, CheckpointTxg)> =
+        BTreeMap::new();
+    for txg in lowest_txg_inside_the_window..first_txg.0 {
+        let ring_slot = target_for_publish(CheckpointTxg(txg), slots_per_region);
+        if !unreadable_ring_slots.contains(&ring_slot) {
+            continue;
+        }
+        txgs_of_each_unreadable_slot
+            .entry(ring_slot)
+            .and_modify(|(_, highest)| *highest = CheckpointTxg(txg))
+            .or_insert((CheckpointTxg(txg), CheckpointTxg(txg)));
+    }
+    txgs_of_each_unreadable_slot
+        .into_iter()
+        .map(
+            |(ring_slot, (lowest_txg_it_can_hold, highest_txg_it_can_hold))| {
+                UnreadableRingSlotInsideTheWindow {
+                    ring_slot,
+                    lowest_txg_it_can_hold,
+                    highest_txg_it_can_hold,
+                }
+            },
+        )
+        .collect()
+}
+
 /// 重建分配器：从上一版的分配记录重建，按可再分配谓词的门槛回收，再把只被被抛弃根引用的槽隔离（影子账：被抛弃的根按最新根指着的
 /// 实例表判，D23（journal 的角色与格式） 已定项 14 射程「影子账与按实例表判抛弃留着，理由是崩溃恢复」）。
 /// 影子账只住内存，所以每次挂载都要重算（步 4 / 步 5 代码三方第一轮云端攻方腿打中：普通重开一次隔离就归零）。
@@ -1625,25 +1696,71 @@ fn rebuilt_allocator<Device: BlockDevice>(
         &system_configuration.immutable.filesystem_identifier,
         &newest_table,
     );
-    // 影子账与分配器那张根环表读的是同一遍根环：两遍之间的瞬时读错会让两边看见的根不一样。
-    let roots_with_ring_slots = readable_roots_with_ring_slots(
-        devices,
-        &system_configuration.immutable.region_devices,
-        &system_configuration.immutable.sizes,
-        &system_configuration.immutable.filesystem_identifier,
-    );
+    // 影子账、回收门槛与分配器那张根环表读的是同一遍根环（每槽读一次）：两遍之间的瞬时读错会让几边看见的根不一样。
+    let slots_per_region = system_configuration
+        .immutable
+        .sizes
+        .root_ring_slots_per_region;
+    let ring_slot_readings: Vec<(RootRingSlot, RootRingSlotReading)> =
+        every_root_ring_slot(&system_configuration.immutable.sizes)
+            .into_iter()
+            .map(|ring_slot| {
+                (
+                    ring_slot,
+                    read_root_ring_slot(
+                        devices,
+                        &system_configuration.immutable.region_devices,
+                        &system_configuration.immutable.sizes,
+                        &system_configuration.immutable.filesystem_identifier,
+                        ring_slot,
+                    ),
+                )
+            })
+            .collect();
+    let roots_with_ring_slots: Vec<(RootRingSlot, RootRecord)> = ring_slot_readings
+        .iter()
+        .filter_map(|(ring_slot, reading)| match reading {
+            RootRingSlotReading::SelfVerified(root) => Some((*ring_slot, *root)),
+            RootRingSlotReading::Bad(_) => None,
+        })
+        .collect();
     let roots: Vec<RootRecord> = roots_with_ring_slots
         .iter()
         .map(|(_, root)| *root)
         .collect();
     let is_abandoned = |root: &RootRecord| abandoned_by_table(root, &newest_table);
-    let oldest_valid_root = roots
+    // 一圈窗口（D16（发布语义） 已定项 1）：N = 环里读得出的最新有效根的 txg（挂载重建分配器时取它、不取 first_txg − 1）。
+    let window_anchor = roots
         .iter()
         .filter(|root| !is_abandoned(root))
         .map(|root| root.checkpoint_txg)
+        .max();
+    let is_inside_the_window = |checkpoint_txg: CheckpointTxg| {
+        window_anchor.is_none_or(|anchor| {
+            txg_is_inside_the_one_revolution_window(checkpoint_txg, anchor, slots_per_region)
+        })
+    };
+    let unreadable_ring_slots_inside_the_window = window_anchor.map_or_else(Vec::new, |anchor| {
+        unreadable_ring_slots_inside_the_window(
+            &ring_slot_readings,
+            anchor,
+            first_txg,
+            slots_per_region,
+        )
+    });
+    // 窗口里有效根的最小 txg；挂载时读不出的槽按它在窗口里能住的最小 txg 算（门槛只会更低）。
+    let oldest_valid_root_in_the_window = roots
+        .iter()
+        .filter(|root| !is_abandoned(root) && is_inside_the_window(root.checkpoint_txg))
+        .map(|root| root.checkpoint_txg)
+        .chain(
+            unreadable_ring_slots_inside_the_window
+                .iter()
+                .map(|unreadable| unreadable.lowest_txg_it_can_hold),
+        )
         .min();
     allocator.reclaim_released_up_to(
-        reclaim_floor(effective_floor, oldest_valid_root),
+        reclaim_floor(effective_floor, oldest_valid_root_in_the_window),
         ReclaimedReuse::Immediately,
     );
     let shadow_ledger_computed = match shadow_ledger {
@@ -1654,6 +1771,7 @@ fn rebuilt_allocator<Device: BlockDevice>(
             &roots,
             &is_abandoned,
             effective_floor,
+            &is_inside_the_window,
             &current_records,
             before_the_one_reread,
         )?,
@@ -1666,9 +1784,19 @@ fn rebuilt_allocator<Device: BlockDevice>(
         abandoned_roots_unreadable,
         placements_referenced_by_abandoned_root,
     } = shadow_ledger_computed;
-    let occupants: Vec<(RootRingSlot, RootRingOccupant)> = roots_with_ring_slots
+    // 挂载时读不出的槽在根环表里按它在窗口里能住的最大 txg 记成一条有效根（它在进程内的窗口里待到这个进程写到那个槽为止）；
+    // 同一个槽这一遍读出了根的，读出来的那条在后面、盖掉它（这一遍读的是同一个槽，两者不会同时成立，留着次序只为不依赖这一点）。
+    let occupants: Vec<(RootRingSlot, RootRingOccupant)> = unreadable_ring_slots_inside_the_window
         .iter()
-        .map(|(ring_slot, root)| {
+        .map(|unreadable| {
+            (
+                unreadable.ring_slot,
+                RootRingOccupant::UnreadableAtMountInsideTheWindow {
+                    highest_txg_it_can_hold: unreadable.highest_txg_it_can_hold,
+                },
+            )
+        })
+        .chain(roots_with_ring_slots.iter().map(|(ring_slot, root)| {
             let occupant = if is_abandoned(root) {
                 RootRingOccupant::AbandonedRoot {
                     referenced_placements: placements_referenced_by_abandoned_root
@@ -1682,7 +1810,7 @@ fn rebuilt_allocator<Device: BlockDevice>(
                 }
             };
             (*ring_slot, occupant)
-        })
+        }))
         .collect();
     allocator.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(
         system_configuration
@@ -2557,9 +2685,20 @@ fn raise_the_floor_through<Device: BlockDevice>(
     .map_err(|still_unreadable| {
         MountError::NewerStateStillUnreadableAfterOneReread(Box::new(still_unreadable.into()))
     })?;
-    let oldest_valid_root = roots
+    // 一圈窗口（D16（发布语义） 已定项 1）：进程内的锚是这个进程记到的最新根，抬 F 时就是现行那一版的根。
+    let slots_per_region = system_configuration
+        .immutable
+        .sizes
+        .root_ring_slots_per_region;
+    let window_anchor = current.root.checkpoint_txg;
+    let is_inside_the_window = |checkpoint_txg: CheckpointTxg| {
+        txg_is_inside_the_one_revolution_window(checkpoint_txg, window_anchor, slots_per_region)
+    };
+    let oldest_valid_root_in_the_window = roots
         .iter()
-        .filter(|root| !abandoned_by_table(root, &table))
+        .filter(|root| {
+            !abandoned_by_table(root, &table) && is_inside_the_window(root.checkpoint_txg)
+        })
         .map(|root| root.checkpoint_txg)
         .min();
     // 候选集按新 F 缩小，只被被抛弃根引用的槽会变多（一个槽此前靠一条低于新 F 的根豁免），回收之前先按新 F 重算影子账，
@@ -2572,6 +2711,7 @@ fn raise_the_floor_through<Device: BlockDevice>(
             &roots,
             &|root| abandoned_by_table(root, &table),
             new_floor,
+            &is_inside_the_window,
             &current.allocation_records,
             &mut BeforeTheOneReread::RereadImmediately,
         ) {
@@ -2585,7 +2725,7 @@ fn raise_the_floor_through<Device: BlockDevice>(
         0
     };
     let reclaimed = allocator.reclaim_released_up_to(
-        reclaim_floor(new_floor, oldest_valid_root),
+        reclaim_floor(new_floor, oldest_valid_root_in_the_window),
         ReclaimedReuse::HeldUntilFloorTakesEffect,
     );
     let publishes_in_the_sequence = planned_txgs.len();
@@ -3343,11 +3483,13 @@ fn devices_without_the_selected_version<Reader: PoolReader + ?Sized>(
     Ok(devices_without)
 }
 
-/// 读阶段的见证读判完之后，逐盘核或取号那一遍又读到某块盘的系统配置槽读不出（两次读之间的瞬时读错）：照 C331（择根倒挂压过已确认的写）
-/// 取甲的拒因报（用户 2026-09-28 定：这一支并进甲的拒因）——前一遍是读阶段往下走用的那一遍读数，后一遍是同一个见证值、
-/// 判的是「有一槽读不出或自证不过」。一个字节都没写。
+/// 读阶段的见证读判完之后，逐盘核或取号那一遍又读到判不出（R2）或某块盘一份自证过的系统配置都没有（两次读之间的瞬时读错）：
+/// 照判据 N-配置 为真、重读仍真的拒因报（用户 2026-09-28 定：这一支并进见证读的拒因）——前一遍是读阶段往下走用的那一遍读数，
+/// 后一遍是同一个见证值、判的是 `comparison`（[`WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots`]
+/// 或 [`WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot`]）。一个字节都没写。
 fn refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
     read_stage: ReadStageSettled,
+    comparison: WitnessedCounterComparison,
 ) -> MountError {
     let settled = match read_stage {
         ReadStageSettled::OnTheFirstRead { first_read } => first_read,
@@ -3360,7 +3502,7 @@ fn refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_
                 selected_version: settled.selected_version,
                 witness: NewerPublishWitness {
                     witnessed_journal_counter: settled.witness.witnessed_journal_counter,
-                    comparison: WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified,
+                    comparison,
                 },
             },
         },
@@ -4072,6 +4214,7 @@ fn establish_instance<Device: BlockDevice>(
     .map_err(|_device| {
         refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
             start.rereads.read_stage,
+            WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot,
         )
     })?;
     if !devices_without.is_empty() {
@@ -4100,12 +4243,19 @@ fn establish_instance<Device: BlockDevice>(
             ) => MountError::SequenceNumberPastTheTopOfItsRange(
                 SequenceNumberAtTheTopOfItsRange::InstanceGeneration(highest_on_disk),
             ),
-            // 读阶段的见证读两槽都读得出、取号那一刻读见证值时这块盘有一槽读不出或自证不过（两次读之间的瞬时读错，C554 乙-配置续 Q1）：
-            // 照 C331（择根倒挂压过已确认的写） 取甲的拒因报（用户 2026-09-28 定）。一个字节都没写。
-            ExpectedInstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness {
+            // 读阶段的见证读判得出、取号那一刻读见证值时判不出（R2）或这块盘一份自证过的系统配置都没有（两次读之间的瞬时读错，
+            // C554 乙-配置续 Q1）：照见证读的拒因报（用户 2026-09-28 定）。一个字节都没写。
+            ExpectedInstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
+                first_device_reporting_a_read_error: _,
+            } => refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
+                start.rereads.read_stage,
+                WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots,
+            ),
+            ExpectedInstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
                 device: _,
             } => refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
                 start.rereads.read_stage,
+                WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot,
             ),
             ExpectedInstanceAcquisitionFailed::Acquisition(acquisition) => {
                 MountError::Acquisition(acquisition)
@@ -4708,7 +4858,7 @@ struct ReadStage {
     against_the_witness: SelectedVersionAgainstTheWitness,
 }
 
-/// 判据 N-配置（E158 第 3 次跑登记第 347 行）的读数：系统配置槽直接读盘（`verified_system_configuration_slots`，每块盘两槽），
+/// 判据 N-配置（E158 第 3 次跑登记第 347 行）的读数：系统配置槽直接读盘（`system_configuration_slots_of_one_device_read_once`，每块盘两槽），
 /// 不经读缓存（第 4 次跑登记 5.1 第 1 条：重读那一遍也直接从盘上重读这几槽）；记录取读阶段那一遍扫出来的。
 fn newer_publish_witness<Reader: PoolReader + ?Sized>(
     devices: &Reader,
@@ -4722,35 +4872,27 @@ fn newer_publish_witness<Reader: PoolReader + ?Sized>(
             .sizes
             .fixed_structure_slot_spacing,
     );
-    let verified_slots_of_each_device: Vec<Vec<crate::system_configuration::SystemConfiguration>> =
-        devices
-            .device_identities()
-            .into_iter()
-            .map(|device| {
-                verified_system_configuration_slots(
-                    devices,
-                    device,
-                    slot_spacing_in_bytes,
-                    &system_configuration.immutable.filesystem_identifier,
-                )
-            })
-            .collect();
-    let witnessed_journal_counter = verified_slots_of_each_device
+    // 按盘读（S-辨）：每块盘两槽各读一次，自证过的进 c_见证，设备报读错的计数（D22（单元原子性怎么合成） 已定项 8 射程）。
+    let slots_of_each_device: Vec<SystemConfigurationSlotsOfOneDevice> = devices
+        .device_identities()
+        .into_iter()
+        .map(|device| {
+            system_configuration_slots_of_one_device_read_once(
+                devices,
+                device,
+                slot_spacing_in_bytes,
+                &system_configuration.immutable.filesystem_identifier,
+            )
+        })
+        .collect();
+    let witnessed_journal_counter = slots_of_each_device
         .iter()
-        .flatten()
+        .flat_map(|slots_of_this_device| slots_of_this_device.self_verified.iter())
         .map(|slot| slot.quantities.journal_tail)
         .max()
         .unwrap_or(0);
-    let some_system_configuration_slot_unread =
-        verified_slots_of_each_device
-            .iter()
-            .any(|verified_slots_of_this_device| {
-                u64::try_from(verified_slots_of_this_device.len())
-                    .expect("每块盘的系统配置槽数装得进 u64")
-                    < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
-            });
-    let comparison = if some_system_configuration_slot_unread {
-        WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified
+    let comparison = if system_configuration_witness_is_undecidable(&slots_of_each_device) {
+        WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots
     } else if witnessed_journal_counter == 0 {
         WitnessedCounterComparison::NothingWitnessed
     } else if let Some(last_record_of_the_selected_version) = records
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 22a92fb..c9e0bd3 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -15,8 +15,8 @@ use singlefs_format::{
     ACCOUNTING_ENTRY_BYTES, DATA_UNIT_BYTES, FIXED_STRUCTURE_SLOT_SPACING_MINIMUM_BYTES,
     INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT,
     JOURNAL_SAFETY_FACTOR, LINEAGE_RECORD_BYTES, MAPPING_ENTRY_BYTES, NODE_BYTES,
-    ROOT_RECORD_BYTES, ROOT_RING_REGIONS, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES,
-    TOMBSTONE_RECORD_BYTES,
+    ROOT_RECORD_BYTES, ROOT_RING_REGIONS, SLOT_BYTES, SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
+    SYSTEM_CONFIGURATION_SLOT_BYTES, TOMBSTONE_RECORD_BYTES,
 };
 use singlefs_replay_prefix::{
     choose_root_slot, first_copy_of_each_record_key, replay_core, CoreFailure, CoreRecord,
@@ -70,7 +70,7 @@ use crate::records::{
     TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LIVELIST,
     TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
 };
-use crate::root_record::{RootRecord, UnmountMarker};
+use crate::root_record::RootRecord;
 use crate::root_ring::{region_start, slot_offset, RootRingSlot, RootRingSlotsPerRegionOutOfRange};
 use crate::system_configuration::{
     journal_in_flight_record_limit, journal_in_flight_record_limit_fits_its_four_byte_field,
@@ -3235,18 +3235,80 @@ pub fn verified_system_configuration_slots<Reader: PoolReader + ?Sized>(
     slot_spacing_in_bytes: u64,
     filesystem_identifier: &[u8; 16],
 ) -> Vec<SystemConfiguration> {
+    system_configuration_slots_of_one_device_read_once(
+        reader,
+        device,
+        slot_spacing_in_bytes,
+        filesystem_identifier,
+    )
+    .self_verified
+}
+
+/// 一块盘两个系统配置槽各读一次的结局（D22（单元原子性怎么合成） 已定项 8 射程「可写挂载读系统配置的见证按盘读」，S-辨：
+/// 一槽撕裂、另一槽自证过，与设备读不出分开）：自证过、fsid 与本池相同的那几份，与设备报读错（`PoolReader::read` 交空）的槽数。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct SystemConfigurationSlotsOfOneDevice {
+    pub device: DeviceIdentity,
+    pub self_verified: Vec<SystemConfiguration>,
+    pub slots_reporting_a_read_error: u64,
+}
+
+impl SystemConfigurationSlotsOfOneDevice {
+    /// 两槽都自证过（R2 里「有一块盘两槽都自证过」那一半）。
+    #[must_use]
+    pub fn both_slots_self_verified(&self) -> bool {
+        u64::try_from(self.self_verified.len()).expect("一块盘两个槽，装得进 u64")
+            == SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
+    }
+}
+
+/// 一块盘两槽各读一次：读得出的按整槽校验和与 fsid 判（每区槽数 S 越界的槽与自证不过的槽一样丢掉：走到这一步之前
+/// `choose_system_configuration` 已经在同一个池上判过区间、越界的池根本挂不上，这里再报一次越界也没有第二种处置），读不出的计数。
+#[must_use]
+pub fn system_configuration_slots_of_one_device_read_once<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    device: DeviceIdentity,
+    slot_spacing_in_bytes: u64,
+    filesystem_identifier: &[u8; 16],
+) -> SystemConfigurationSlotsOfOneDevice {
     let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
-    [0, slot_spacing_in_bytes]
-        .into_iter()
-        .filter_map(|offset| reader.read(device, DeviceOffsetInBytes(offset), slot_bytes))
-        // 这里把每区槽数 S 越界的槽与自证不过的槽一样丢掉：走到这一步之前
-        // `choose_system_configuration` 已经在同一个池上判过区间、越界的池根本挂不上，
-        // 这个函数再报一次越界也没有第二种处置。
-        .filter_map(|bytes| SystemConfiguration::parse_slot(&bytes).ok())
-        .filter(|system_configuration| {
-            system_configuration.immutable.filesystem_identifier == *filesystem_identifier
-        })
-        .collect()
+    let mut self_verified = Vec::new();
+    let mut slots_reporting_a_read_error = 0;
+    for offset in [0, slot_spacing_in_bytes] {
+        match reader.read(device, DeviceOffsetInBytes(offset), slot_bytes) {
+            None => slots_reporting_a_read_error += 1,
+            Some(bytes) => {
+                if let Ok(system_configuration) = SystemConfiguration::parse_slot(&bytes) {
+                    if system_configuration.immutable.filesystem_identifier
+                        == *filesystem_identifier
+                    {
+                        self_verified.push(system_configuration);
+                    }
+                }
+            }
+        }
+    }
+    SystemConfigurationSlotsOfOneDevice {
+        device,
+        self_verified,
+        slots_reporting_a_read_error,
+    }
+}
+
+/// 系统配置见证判不出（R2，D22（单元原子性怎么合成） 已定项 8 射程，用户 2026-10-03 定「改取 R2」，零轮）：
+/// 判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过。可写挂载读阶段的见证读（`mount::newer_publish_witness`）与
+/// 取号那一刻读见证值（`transaction::self_verified_system_configurations_of_every_device`）都按它判，一处定义。
+/// 撕裂的系统配置轮换（读得出、自证不过，设备没报错）不算判不出；一块盘报读错而另一块盘两槽都自证过也不算。
+#[must_use]
+pub fn system_configuration_witness_is_undecidable(
+    slots_of_every_device: &[SystemConfigurationSlotsOfOneDevice],
+) -> bool {
+    slots_of_every_device
+        .iter()
+        .any(|slots| slots.slots_reporting_a_read_error > 0)
+        && !slots_of_every_device
+            .iter()
+            .any(SystemConfigurationSlotsOfOneDevice::both_slots_self_verified)
 }
 
 /// 全环扫描：每盘每个记录槽都读（有提示时只读提示的那些），两份镜像任一份自证过即算在；fsid 不符的不算数。
@@ -3315,7 +3377,7 @@ enum BackChainJudgement {
 }
 
 /// 判一条记录的反向链（I-8.6（反向链算法）；D23（journal 的角色与格式） 已定项 8 / 已定项 10 / 已定项 19 ②）：
-/// 本实例内逻辑前一条 = 同一实例、计数器小 1 的那一条；链值 = CRC-32C(它在盘上那一份的 311 字节头，`header_csum` 按 0 参与)。
+/// 本实例内逻辑前一条 = 同一实例、计数器小 1 的那一条；链值 = CRC-32C(它在盘上那一份的 487 字节头，`header_csum` 按 0 参与)。
 /// 按盘上的原样字节算、不按解出来的字段重写（重写会把头里读者不看的字节抹成 0），与池级 checker 判 I-8.6 同一个输入；
 /// 取的是 [`PoolReader::device_identities`] 次序里第一份解得出、且与扫环时收下的那一条相同的镜像——扫环按同一次序收记录。
 /// 前一条不在盘上不判：链是按实例的，跨实例边界本来不比，前缀里的记录都接在所选根覆盖的那一条之后，
@@ -3512,23 +3574,20 @@ pub fn replay_journal(
         NewRootSource::ChosenRootItself => *root,
         NewRootSource::NewRootSegmentOfTheRecordAtKeyOrderIndex(index_in_key_order) => {
             let record = records_in_key_order[index_in_key_order];
-            // 每条记录都带整次发布的新根段（已定项 15），施加末条那一份。
+            // 每条记录都带整次发布的新根段（已定项 15），施加末条那一份：把所选根的七个字段换成记录里的——实例表指针、卸载记号、
+            // 分配记录树根从新根段来，不再照抄所选根（用户 2026-10-02「回到甲」；`research/prompts/m3-ring-r3-main-verification.md`
+            // 第二节第 1、2 行：照抄时首个文件、卸载两种发布施加出错根，专用清扫发布施加出旧实例表链）。fsid 新根段里没有，照所选根。
             RootRecord {
                 filesystem_identifier: root.filesystem_identifier,
-                // 新根段（D23（journal 的角色与格式） 已定项 15）里没有 flags：由记录施加出来的这一版没有一条根槽写过它，
-                // 不是卸载那一串写下的根（D22（单元原子性怎么合成） 已定项 7「其余根写 0」）。它只当下一次发布接在后面的那一版，
-                // 发布照自己的计划写记号、不照抄这一项。
-                unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
+                unmount_marker: record.new_unmount_marker,
                 instance: record.instance,
                 checkpoint_txg: record.checkpoint_txg,
                 tree_table: record.new_tree_table,
                 tree_identifier_watermark: record.new_tree_identifier_watermark,
                 rollback_floor: record.new_rollback_floor,
-                instance_table: root.instance_table,
+                instance_table: record.new_instance_table,
                 mapping_root: record.new_mapping_root,
-                // 与实例表指针同一条理由：新根段（D23（journal 的角色与格式） 已定项 15）里没有这一项，施加记录只能照抄被施加的那条根的。
-                // 施加一条写行记录而它的根槽没落盘时，重建出来的这一版仍指着上一版的分配记录树——那正是「这次写行没有成立」该有的账。
-                allocation_record_tree_root: root.allocation_record_tree_root,
+                allocation_record_tree_root: record.new_allocation_record_tree_root,
             }
         }
     };
diff --git a/crates/singlefs-core/src/root_record.rs b/crates/singlefs-core/src/root_record.rs
index 8a93a59..67526d7 100644
--- a/crates/singlefs-core/src/root_record.rs
+++ b/crates/singlefs-core/src/root_record.rs
@@ -37,6 +37,16 @@ impl UnmountMarker {
         }
     }
 
+    /// journal 记录新根段里的卸载记号那 4 字节（D23（journal 的角色与格式） 已定项 15）：与根记录 flags 同一个编码，位 0 之外有位为 1 ⇒ `None`
+    /// （读者把那条记录当损坏，与根记录 flags、记录标志「其余位读到非 0」同一个处置）。
+    #[must_use]
+    pub const fn of_unmount_marker_field_of_a_new_root_segment(field: u32) -> Option<Self> {
+        if field & !ROOT_RECORD_FLAG_UNMOUNT_MARKER != 0 {
+            return None;
+        }
+        Some(Self::of_flags_bits_carrying_only_the_marker(field))
+    }
+
     /// 从已经判过「记号之外的位全是 0」的 flags 读回记号。
     #[must_use]
     const fn of_flags_bits_carrying_only_the_marker(flags: u32) -> Self {
diff --git a/crates/singlefs-core/src/root_ring.rs b/crates/singlefs-core/src/root_ring.rs
index 36296ad..0c65179 100644
--- a/crates/singlefs-core/src/root_ring.rs
+++ b/crates/singlefs-core/src/root_ring.rs
@@ -116,6 +116,21 @@ pub fn target_for_publish(
     }
 }
 
+/// 一圈窗口（D16（发布语义） 已定项 1「一圈窗口里最旧有效根」，用户 2026-10-02 定）：锚 N 之下的 txg 是不是落在窗口 txg > N − 3S 里
+/// （3S 是环长：R 个区域 × S 个槽）。写成 txg + 3S > N，N < 3S 时整个环都在窗口里、不减出负数。核心层判窗口的每一处
+/// （分配器的回收门槛、影子账的候选过滤、挂载时读不出的槽）都走它；锚取什么由调用方定（挂载时环里读得出的最新有效根，进程内这个进程记到的最新根）。
+#[must_use]
+pub fn txg_is_inside_the_one_revolution_window(
+    checkpoint_txg: CheckpointTxg,
+    window_anchor: CheckpointTxg,
+    slots_per_region: RootRingSlotsPerRegion,
+) -> bool {
+    checkpoint_txg
+        .0
+        .saturating_add(ROOT_RING_REGIONS * slots_per_region.count())
+        > window_anchor.0
+}
+
 /// 一个区域占多少字节：S 个槽 × 槽距。区域之间隔着 P × chunk，这一段之外的字节不归根环。
 /// mkfs 按它清根环、`ring_end` 按它算末端，两处同一个定义。
 #[must_use]
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 0aa0c1c..3d99aa1 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -104,7 +104,10 @@ use crate::records::{
 };
 use crate::recovery::{
     effective_rollback_floor_rereading_unreadable_reads_once, highest_root_instance,
-    tree_table_entry_count, verified_system_configuration_slots, PoolReader, RecoveryFailure,
+    system_configuration_slots_of_one_device_read_once,
+    system_configuration_witness_is_undecidable, tree_table_entry_count,
+    verified_system_configuration_slots, PoolReader, RecoveryFailure,
+    SystemConfigurationSlotsOfOneDevice,
 };
 use crate::root_record::{RootRecord, UnmountMarker};
 use crate::root_ring::{slot_offset, target_for_publish};
@@ -165,13 +168,13 @@ pub enum CommitStep<'publish> {
 }
 
 /// 一条 journal 记录装几个点名项，只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支必须能被测试强制进入）。
-/// 末条再跨记录（D23（journal 的角色与格式） 已定项 17）在产品路径上要一次发布点名 68 项以上才走得到——树表 0 条的一版上写行要实例表长到
-/// 67 片（两万四千多行），带文件的一版要六十几片 inode 叶容器；压小这个数就能用几个单元造出跨记录的发布。产品路径恒 `FromTheRecordFormat`。
+/// 末条再跨记录（D23（journal 的角色与格式） 已定项 17）在产品路径上要一次发布点名 65 项以上才走得到——树表 0 条的一版上写行要实例表长到
+/// 64 片（两万三千多行），带文件的一版要五十几片 inode 叶容器；压小这个数就能用几个单元造出跨记录的发布。产品路径恒 `FromTheRecordFormat`。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum JournalRecordNamedEntryCapacity {
-    /// 记录格式装得下的：(4096 − 311) ÷ 56 = 67 项（`JOURNAL_NAMED_ENTRIES_PER_RECORD`）。
+    /// 记录格式装得下的：(4096 − 487) ÷ 56 = 64 项（`JOURNAL_NAMED_ENTRIES_PER_RECORD`）。
     FromTheRecordFormat,
-    /// 压到这么多项（1..=67）。
+    /// 压到这么多项（1..=64）。
     CappedForTests { named_entries_per_record: usize },
 }
 
@@ -181,7 +184,7 @@ impl JournalRecordNamedEntryCapacity {
     pub fn named_entries_per_record(self) -> usize {
         match self {
             JournalRecordNamedEntryCapacity::FromTheRecordFormat => {
-                usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("67")
+                usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("64")
             }
             JournalRecordNamedEntryCapacity::CappedForTests {
                 named_entries_per_record,
@@ -277,12 +280,12 @@ impl<Device: BlockDevice> PoolWriter<'_, Device> {
     /// 装上「一条 journal 记录装几个点名项」这个只供测试的开关（[`JournalRecordNamedEntryCapacity`]）。产品路径一处都不调它。
     ///
     /// # Panics
-    /// 压的项数是 0 或大于记录格式装得下的 67 项（`JOURNAL_NAMED_ENTRIES_PER_RECORD`）：开关给错了。
+    /// 压的项数是 0 或大于记录格式装得下的 64 项（`JOURNAL_NAMED_ENTRIES_PER_RECORD`）：开关给错了。
     pub fn set_journal_record_named_entry_capacity(
         &mut self,
         capacity: JournalRecordNamedEntryCapacity,
     ) {
-        let of_the_format = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("67");
+        let of_the_format = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("64");
         assert!(
             (1..=of_the_format).contains(&capacity.named_entries_per_record()),
             "一条记录装的点名项压到 {capacity:?}：要在 1..={of_the_format} 之内"
@@ -669,40 +672,59 @@ fn highest_system_configuration_instance<Device: BlockDevice>(
 const JOURNAL_TAIL_WITNESSED_BY_NO_SYSTEM_CONFIGURATION: u64 = 0;
 
 /// 取号那一刻逐盘读到的两槽：盘表里每块盘两槽中本池自证过（整槽校验和过、fsid 与本池相同）的系统配置，按盘表次序、一块盘一项。
-/// 只读盘、不写。
+/// 只读盘、不写。两道拒，都在第一个取号写之前：
 ///
-/// 某块盘一份都读不出就拒（C554 乙-配置续 Q1，主 agent 2026-09-27 定走「拒」）：这块盘不「可见」（D18（块里携带什么信息） 已定项 11
-/// 「可写挂载的顺序」：「可见」= 独占打开成功且系统配置读得通），与可写挂载取号之前的逐盘核（`mount.rs` 的
-/// `devices_without_the_selected_version` 第一支，Z3-A 乙）同一判。那一道核与这一次读之间一次瞬时读错（或盘在两次读之间坏掉）就走得到这里；
-/// 不拒的话见证值取别的盘的，这块盘上自证过的槽里见证的那次发布就漏在见证值之外。
+/// - 判不出（R2，`recovery::system_configuration_witness_is_undecidable`：有槽设备报读错，且没有一块盘两槽都自证过；C331（择根倒挂压过已确认的写）
+///   改取 R2，用户 2026-10-03 定，零轮）：见证值可能缺了读错的那一槽，取不了值，与可写挂载判 N-配置 同一个判不出（D23（journal 的角色与格式） 已定项 14
+///   「取号那一刻按同一个判不出（R2……）判」）。撕裂的系统配置轮换（读得出、自证不过）不拒：取号写重写坏的那一槽。
+/// - 某块盘一份都读不出（C554 乙-配置续 Q1，主 agent 2026-09-27 定走「拒」）：这块盘不「可见」（D18（块里携带什么信息） 已定项 11
+///   「可写挂载的顺序」：「可见」= 独占打开成功且系统配置读得通），与可写挂载取号之前的逐盘核（`mount.rs` 的
+///   `devices_without_the_selected_version` 第一支，Z3-A 乙）同一判。R2 判得出的盘面上也有这一形（一块盘两槽都读不出、另一块两槽都自证过），
+///   R2 之下这一道照留（条款没写它随取甲收严之后是否还单独成立，照取甲之前的「一份都没有就拒」留着）。
 ///
 /// # Errors
-/// 按盘表次序第一块有一槽读不出或自证不过的盘 ⇒ [`InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness`]。
+/// 判不出 ⇒ [`InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness`]（带盘表次序第一块报读错的盘）；
+/// 按盘表次序第一块一份自证过的都没有的盘 ⇒ [`InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness`]。
 fn self_verified_system_configurations_of_every_device<Device: BlockDevice>(
     pool: &PoolWriter<'_, Device>,
 ) -> Result<Vec<Vec<SystemConfiguration>>, InstanceAcquisitionFailed> {
     let spacing = u64::from(pool.parameters.geometry.fixed_structure_slot_spacing);
     let filesystem_identifier = pool.parameters.filesystem_identifier;
-    pool.devices
+    let slots_of_every_device: Vec<SystemConfigurationSlotsOfOneDevice> = pool
+        .devices
         .iter()
         .map(|(identity, _)| {
-            let slots_of_this_device = verified_system_configuration_slots(
+            system_configuration_slots_of_one_device_read_once(
                 &*pool.devices,
                 *identity,
                 spacing,
                 &filesystem_identifier,
-            );
-            // C331（择根倒挂压过已确认的写） 取甲（用户 2026-09-28 定）：有一槽读不出或自证不过，见证值就缺了那一槽，取不了值。
-            if u64::try_from(slots_of_this_device.len()).expect("每块盘的系统配置槽数装得进 u64")
-                < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
-            {
+            )
+        })
+        .collect();
+    if system_configuration_witness_is_undecidable(&slots_of_every_device) {
+        let first_device_reporting_a_read_error = slots_of_every_device
+            .iter()
+            .find(|slots_of_this_device| slots_of_this_device.slots_reporting_a_read_error > 0)
+            .expect("判不出的前一半就是有一块盘报读错")
+            .device;
+        return Err(
+            InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
+                first_device_reporting_a_read_error,
+            },
+        );
+    }
+    slots_of_every_device
+        .into_iter()
+        .map(|slots_of_this_device| {
+            if slots_of_this_device.self_verified.is_empty() {
                 return Err(
-                    InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness {
-                        device: *identity,
+                    InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
+                        device: slots_of_this_device.device,
                     },
                 );
             }
-            Ok(slots_of_this_device)
+            Ok(slots_of_this_device.self_verified)
         })
         .collect()
 }
@@ -775,11 +797,12 @@ pub fn instance_generation_to_acquire<Device: BlockDevice>(
 /// ⚠️ D18（块里携带什么信息） 已定项 11 的「重试到 T_retry 用尽才算失败」这里没有做：
 /// 块设备报的第一个错就判失败（写路径上的重试全仓都还没有）。
 ///
-/// 前面没有可写挂载那一道逐盘核（mkfs 同一个进程里那条流、用例与装置在调）：取号那一刻读见证值时某块盘有一槽读不出或本池自证不过，
-/// 同样在第一个取号写之前拒（[`InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness`]）。
+/// 前面没有可写挂载那一道逐盘核（mkfs 同一个进程里那条流、用例与装置在调）：取号那一刻读见证值时判不出（R2）或某块盘一份本池自证过的都没有，
+/// 同样在第一个取号写之前拒（[`self_verified_system_configurations_of_every_device`] 的两道）。
 ///
 /// # Errors
-/// 某块盘有一槽系统配置读不出或自证不过 ⇒ `DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness`（一个字节没写）；
+/// 判不出 ⇒ `SystemConfigurationWitnessUndecidableWhenReadingTheWitness`；某块盘一份自证过的系统配置都没有 ⇒
+/// `DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness`（两样都一个字节没写）；
 /// 屏障或取号写报错 ⇒ `Acquisition`（回卷的结局在里面）。
 ///
 /// # Panics
@@ -795,9 +818,15 @@ pub fn acquire_instance<Device: BlockDevice>(
 /// 取号没做成（[`acquire_instance`]、取号的写那一半）。
 #[derive(Debug)]
 pub enum InstanceAcquisitionFailed {
-    /// 取号那一刻读见证值时，`device` 两槽里有一槽读不出或本池自证不过（C554 乙-配置续 Q1；D18（块里携带什么信息） 已定项 11
-    /// 「可见」；C331（择根倒挂压过已确认的写） 取甲，用户 2026-09-28 定，从「一份都没有」收严成「缺一槽」）：一个字节都没写。第一道屏障之前就读不出时连屏障都没发；屏障之后重读才读不出时发了那一道屏障、没有写。
-    DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness {
+    /// 取号那一刻读见证值时判不出（R2：有槽设备报读错，且没有一块盘两槽都自证过；C331（择根倒挂压过已确认的写） 改取 R2，
+    /// 用户 2026-10-03 定）：一个字节都没写。`first_device_reporting_a_read_error` 是盘表次序第一块报读错的盘。
+    /// 第一道屏障之前就判不出时连屏障都没发；屏障之后重读才判不出时发了那一道屏障、没有写。
+    SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
+        first_device_reporting_a_read_error: DeviceIdentity,
+    },
+    /// 取号那一刻读见证值时，`device` 两槽里一份本池自证过的系统配置都没有（C554 乙-配置续 Q1；D18（块里携带什么信息） 已定项 11「可见」）：
+    /// 一个字节都没写。屏障前后哪一次读到的，同上一个成员。
+    DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
         device: DeviceIdentity,
     },
     /// 屏障或取号写报错（回卷的结局在里面）。
@@ -820,9 +849,14 @@ pub enum ExpectedInstanceAcquisitionFailed {
     },
     /// 写之前重算时盘上最大的实例代号已是 `u32::MAX`（两次读之间瞬时读错才会与判定时不同）：一个字节都没写。
     InstanceGenerationPastTheTopOfItsRange(InstanceGenerationPastTheTopOfItsRange),
-    /// 同 [`InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness`]：可写挂载取号之前的逐盘核读得出、
-    /// 取号那一刻读见证值时这块盘有一槽读不出或自证不过（两次读之间一次瞬时读错就够）。一个字节都没写。
-    DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness {
+    /// 同 [`InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness`]：可写挂载读阶段的见证读判得出、
+    /// 取号那一刻读见证值时判不出（两次读之间一次瞬时读错就够）。一个字节都没写。
+    SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
+        first_device_reporting_a_read_error: DeviceIdentity,
+    },
+    /// 同 [`InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness`]：可写挂载取号之前的逐盘核读得出、
+    /// 取号那一刻读见证值时这块盘一份自证过的系统配置都没有。一个字节都没写。
+    DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
         device: DeviceIdentity,
     },
     Acquisition(AcquisitionFailed),
@@ -833,8 +867,8 @@ pub enum ExpectedInstanceAcquisitionFailed {
 /// 各报一次瞬时读错，判定看到号 1 放行、取号读到号 2 写进两盘）。
 ///
 /// # Errors
-/// 重算的号不同 ⇒ `InstanceGenerationChangedBeforeWrite`；读见证值时某块盘两槽都读不出 ⇒
-/// `DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness`；取号写或屏障报错 ⇒ `Acquisition`（与 `acquire_instance` 同）。
+/// 重算的号不同 ⇒ `InstanceGenerationChangedBeforeWrite`；读见证值时判不出 ⇒ `SystemConfigurationWitnessUndecidableWhenReadingTheWitness`，
+/// 某块盘一份自证过的都没有 ⇒ `DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness`；取号写或屏障报错 ⇒ `Acquisition`（与 `acquire_instance` 同）。
 pub fn acquire_expected_instance<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     expected: InstanceGeneration,
@@ -850,9 +884,14 @@ pub fn acquire_expected_instance<Device: BlockDevice>(
         );
     }
     write_acquired_instance(pool, recomputed).map_err(|failure| match failure {
-        InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness {
+        InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
+            first_device_reporting_a_read_error,
+        } => ExpectedInstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
+            first_device_reporting_a_read_error,
+        },
+        InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
             device,
-        } => ExpectedInstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness {
+        } => ExpectedInstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
             device,
         },
         InstanceAcquisitionFailed::Acquisition(acquisition) => {
@@ -863,7 +902,7 @@ pub fn acquire_expected_instance<Device: BlockDevice>(
 
 /// 取号的写那一半：逐盘核一遍每块盘两槽读得出自证过的系统配置，一道屏障，再读一次每块盘两槽、取见证值
 /// （[`self_verified_system_configurations_of_every_device`]、[`highest_journal_tail_of`]），逐盘写一次系统配置槽（实例代号写新号、
-/// tail 写见证值），两写之后一道屏障。两次读里有一块盘两槽都读不出就一个字节不写、报它；写之前那道屏障报错就一个字节不写、报失败，
+/// tail 写见证值），两写之后一道屏障。两次读里有一次判不出（R2）或有一块盘一份自证过的都没有就一个字节不写、报它；写之前那道屏障报错就一个字节不写、报失败，
 /// 之后任一报错就把已写出的回卷成旧代号（tail 仍写见证值）。
 fn write_acquired_instance<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
@@ -871,7 +910,7 @@ fn write_acquired_instance<Device: BlockDevice>(
 ) -> Result<InstanceGeneration, InstanceAcquisitionFailed> {
     let previous_instance = highest_system_configuration_instance(pool);
     let mut written = Vec::new();
-    // 读见证值时有一块盘两槽都读不出就拒（C554 乙-配置续 Q1 第 1 条）：这一核放在第一道屏障之前，拒的时候连屏障都没发，录制流一步不多。
+    // 读见证值时判不出（R2）或有一块盘一份自证过的都没有就拒（C554 乙-配置续 Q1 第 1 条）：这一核放在第一道屏障之前，拒的时候连屏障都没发，录制流一步不多。
     self_verified_system_configurations_of_every_device(pool)?;
     // 第一个取号写之前一道池屏障（代码审阅第 19 条）：取号写覆写的是这块盘上较旧的那一槽，另一槽可能是上一次发布末尾轮换刚写的
     // （同一个进程里，或上一个进程退出之前）、还没持久；两槽轮换要的是「覆写旧槽时新槽已持久」。这道屏障在任何取号写之前报错，
@@ -886,7 +925,7 @@ fn write_acquired_instance<Device: BlockDevice>(
     // 取号那一写带的 tail 是这一刻的见证值（C554 乙-配置续，用户 2026-09-27 定），不是 0：连着几次取号之后崩溃、中间没有
     // 发布轮换，两槽都换成了取号写，写 0 就把上一次轮换见证的那次发布抹掉，下一次可写挂载的 N-配置 判不出、会抛弃暂时读不出的最新根。
     // 第一道屏障之后、第一个取号写之前另读每块盘两槽一次（E158 第 4 次跑登记第 413 行）；逐盘取号写与回卷写都用这一次读到的。
-    // 屏障前那一核与这一次读之间又有一块盘读不出，就不写（判定与写读同一份输入：写之前重算、对不上不写）。
+    // 屏障前那一核与这一次读之间又判不出或有一块盘一份都读不出，就不写（判定与写读同一份输入：写之前重算、对不上不写）。
     // mkfs 写的 tail 是 0，mkfs 之后第一次取号读到的就是 0：新池新建文件的字节不变。
     let witnessed_journal_tail =
         highest_journal_tail_of(&self_verified_system_configurations_of_every_device(pool)?);
@@ -1322,6 +1361,21 @@ pub fn prepare_the_publish_without_units<Device: BlockDevice>(
     previous_root: &RootRecord,
     plan: ZeroUnitPublishPlan,
 ) -> (PublishWrites, VersionWithoutFilePublishOutput) {
+    let root = RootRecord {
+        filesystem_identifier: previous_root.filesystem_identifier,
+        // 零单元发布只有暖机与可写挂载取号之后那一串走：正常卸载在树表 0 条的一版上一个字节都不写
+        // （`Unmounted::NothingWrittenOnAVersionWithoutFile`），走这里的根都不是卸载那一串写的。
+        unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
+        instance: plan.instance,
+        checkpoint_txg: plan.txg,
+        tree_table: previous_root.tree_table,
+        tree_identifier_watermark: plan.tree_identifier_watermark,
+        rollback_floor: plan.rollback_floor,
+        instance_table: previous_root.instance_table,
+        mapping_root: previous_root.mapping_root,
+        // 零单元发布一个字节都不写：分配记录树照抄上一版的那一条指针，账也因此一条都没变。
+        allocation_record_tree_root: previous_root.allocation_record_tree_root,
+    };
     let record = JournalRecord {
         instance: plan.instance,
         counter: plan.counter,
@@ -1335,28 +1389,17 @@ pub fn prepare_the_publish_without_units<Device: BlockDevice>(
         place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
         back_chain: plan.back_chain,
         filesystem_identifier: unit_filesystem_identifier(&pool.parameters.filesystem_identifier),
-        new_tree_table: previous_root.tree_table,
-        new_mapping_root: previous_root.mapping_root,
-        new_tree_identifier_watermark: plan.tree_identifier_watermark,
-        new_rollback_floor: plan.rollback_floor,
+        // 新根段七项取这次发布写的根上的（D23（journal 的角色与格式） 已定项 15，用户 2026-10-02「回到甲」）。
+        new_tree_table: root.tree_table,
+        new_mapping_root: root.mapping_root,
+        new_tree_identifier_watermark: root.tree_identifier_watermark,
+        new_rollback_floor: root.rollback_floor,
+        new_instance_table: root.instance_table,
+        new_allocation_record_tree_root: root.allocation_record_tree_root,
+        new_unmount_marker: root.unmount_marker,
         named: Vec::new(),
     };
     let record_bytes = record.to_bytes();
-    let root = RootRecord {
-        filesystem_identifier: previous_root.filesystem_identifier,
-        // 零单元发布只有暖机与可写挂载取号之后那一串走：正常卸载在树表 0 条的一版上一个字节都不写
-        // （`Unmounted::NothingWrittenOnAVersionWithoutFile`），走这里的根都不是卸载那一串写的。
-        unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
-        instance: plan.instance,
-        checkpoint_txg: plan.txg,
-        tree_table: previous_root.tree_table,
-        tree_identifier_watermark: plan.tree_identifier_watermark,
-        rollback_floor: plan.rollback_floor,
-        instance_table: previous_root.instance_table,
-        mapping_root: previous_root.mapping_root,
-        // 零单元发布一个字节都不写：分配记录树照抄上一版的那一条指针，账也因此一条都没变。
-        allocation_record_tree_root: previous_root.allocation_record_tree_root,
-    };
     // 零单元：屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障（单元那一段是空的，头一道屏障前面没有写——上一次发布
     // 末尾那道或取号那道已经放行了——同一个写入口上不发它）。
     let writes = PublishWrites {
@@ -1822,7 +1865,7 @@ pub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(
         }
     };
     // 这次发布切成几条记录（D23（journal 的角色与格式） 已定项 17「末条再跨记录」，与带文件那一路同一个切法：
-    // `roles_named_by_each_record_of_the_publish`）：点名项多于一条记录装得下的（实例表长到 67 片起）就装满一条再开下一条。
+    // `roles_named_by_each_record_of_the_publish`）：点名项多于一条记录装得下的（实例表长到 64 片起）就装满一条再开下一条。
     // 写行那次发布不承载事务，这几条都是事务号 0、同属一个事务（`transaction_offset_of_each_record_of_the_publish` 全是 0）：
     // 提交标记只在它的最后一条上（已定项 7），「本次发布末条」标志只在真正的最后一条上（已定项 17），本次发布内序号依次 1..N（已定项 4），
     // 反向链第一条接 `plan.back_chain`、之后每条接这次发布里前一条的头（已定项 8），每条都带整次发布的新根段（已定项 15）。
@@ -1831,6 +1874,19 @@ pub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(
         roles_named_by_each_record_of_the_publish(&named_roles, named_entry_capacity);
     let transaction_offset_of_each_record =
         transaction_offset_of_each_record_of_the_publish(&named_roles, named_entry_capacity);
+    let root = RootRecord {
+        filesystem_identifier: previous_root.filesystem_identifier,
+        // 写行那次发布是新实例的第一次发布，不是卸载那一串。
+        unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
+        instance,
+        checkpoint_txg: txg,
+        tree_table: previous_root.tree_table,
+        tree_identifier_watermark: plan.tree_identifier_watermark,
+        rollback_floor: plan.rollback_floor,
+        instance_table: instance_table_pointer,
+        mapping_root: previous_root.mapping_root,
+        allocation_record_tree_root,
+    };
     let mut written_records: Vec<WrittenJournalRecord> =
         Vec::with_capacity(roles_named_by_each_record.len());
     // 迭代次数的上界是这次的记录条数（≥ 1）；跨轮携带的只有已经装好的记录（下一条的反向链要罩前一条的头）。
@@ -1864,10 +1920,14 @@ pub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(
             },
             back_chain,
             filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
-            new_tree_table: previous_root.tree_table,
-            new_mapping_root: previous_root.mapping_root,
-            new_tree_identifier_watermark: plan.tree_identifier_watermark,
-            new_rollback_floor: plan.rollback_floor,
+            // 新根段七项取这次发布写的根上的（D23（journal 的角色与格式） 已定项 15，用户 2026-10-02「回到甲」）。
+            new_tree_table: root.tree_table,
+            new_mapping_root: root.mapping_root,
+            new_tree_identifier_watermark: root.tree_identifier_watermark,
+            new_rollback_floor: root.rollback_floor,
+            new_instance_table: root.instance_table,
+            new_allocation_record_tree_root: root.allocation_record_tree_root,
+            new_unmount_marker: root.unmount_marker,
             named: roles_of_this_record
                 .iter()
                 .copied()
@@ -1879,19 +1939,6 @@ pub fn prepare_the_row_publish_on_a_version_without_file<Device: BlockDevice>(
     }
     let last_counter_of_this_publish =
         plan.counter + u64::try_from(written_records.len() - 1).expect("一次发布至少一条记录");
-    let root = RootRecord {
-        filesystem_identifier: previous_root.filesystem_identifier,
-        // 写行那次发布是新实例的第一次发布，不是卸载那一串。
-        unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
-        instance,
-        checkpoint_txg: txg,
-        tree_table: previous_root.tree_table,
-        tree_identifier_watermark: plan.tree_identifier_watermark,
-        rollback_floor: plan.rollback_floor,
-        instance_table: instance_table_pointer,
-        mapping_root: previous_root.mapping_root,
-        allocation_record_tree_root,
-    };
     // 单元写按 bump 次序：实例表链的各片（尾片先），分配记录树重写的节点（先叶后根）最后。
     let units: Vec<PublishedUnit> = settled
         .rewritten_roles
@@ -5697,7 +5744,7 @@ pub fn publish_sequential_write<Device: BlockDevice>(
 /// # Errors
 /// 记录落法要的条款没定（中间插入、容器数超过一个根装得下的）⇒ `InodeTreeWriteRefused`；
 /// 其余与 `publish_overwrite` 相同。两样都在任何落盘动作之前返回，盘上逐字节不变。
-/// 这次要点名的单元超过一条 journal 记录装得下的 67 项时不拒：这个事务的记录再跨几条，只有真正的最后一条带
+/// 这次要点名的单元超过一条 journal 记录装得下的 64 项时不拒：这个事务的记录再跨几条，只有真正的最后一条带
 /// 「本次发布末条」标志与提交标记（D23（journal 的角色与格式） 已定项 17 / 已定项 7）。
 pub fn publish_new_inodes<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
@@ -6504,11 +6551,11 @@ pub const ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT: usize = 64;
 /// **只在最后一个事务的记录里点名**，与第 N − 1 个数据单元一起，前面几条不重复点名它们。写了一个数据单元或一个都没写 ⇒
 /// 一个事务点名全部重写角色（新池新建文件今天的形态）。
 ///
-/// 最后一个事务要点名的项多于一条记录装得下的 67 项（[`JOURNAL_NAMED_ENTRIES_PER_RECORD`]）时**末条再跨记录**
+/// 最后一个事务要点名的项多于一条记录装得下的 64 项（[`JOURNAL_NAMED_ENTRIES_PER_RECORD`]）时**末条再跨记录**
 /// （已定项 17，用户 2026-09-24 定）：按 bump 次序装满一条再开下一条，这些记录都属于最后一个事务
 /// （[`transaction_offset_of_each_record_of_the_publish`]）；只有真正的最后一条带「本次发布末条」标志。
 ///
-/// 每一项按 `rewritten` 里的次序（bump 次序）列角色；返回至少一项，每项至多 67 个角色。落盘写出的记录读的就是它。
+/// 每一项按 `rewritten` 里的次序（bump 次序）列角色；返回至少一项，每项至多 64 个角色。落盘写出的记录读的就是它。
 #[must_use]
 pub fn roles_named_by_each_record_of_the_publish(
     rewritten: &[TransactionUnit],
@@ -8490,6 +8537,20 @@ fn publish_admitted<Device: BlockDevice>(
     // 那几条共享它的事务号）。提交标记只在一个事务的最后一条记录上（已定项 7；I-8.8（前缀里的事务不被切开） ③），
     // 「本次发布末条」标志只在这次发布真正的最后一条上（已定项 17）——恢复按它认发布边界（已定项 14 第六条）。
     // 每条都带整次发布的新根段（已定项 15）。反向链：第一条接 `plan.back_chain`（上一条记录的头），之后每条接这次发布里前一条的头（已定项 8）。
+    let root = RootRecord {
+        filesystem_identifier: *filesystem_identifier,
+        unmount_marker,
+        instance,
+        checkpoint_txg: txg,
+        tree_table: tree_table_pointer,
+        tree_identifier_watermark: plan.tree_identifier_watermark,
+        rollback_floor: plan.rollback_floor,
+        instance_table: instance_table_pointer,
+        mapping_root: mapping_pointer,
+        // 带文件的一版的分配记录树住树表条目（D8（核心索引结构） 已定项 8）：根记录这一项恒全零，
+        // 两处都写就成了同一个量的两份手抄。`root_record_of_a_file_version_leaves_the_allocation_record_tree_pointer_zero` 钉住。
+        allocation_record_tree_root: NodePointer::empty_root(),
+    };
     let mut written_records: Vec<WrittenJournalRecord> =
         Vec::with_capacity(roles_named_by_each_record.len());
     // 迭代次数的上界是这次的记录条数（≥ 1）；跨轮携带的只有已经装好的记录（下一条的反向链要罩前一条的头）。
@@ -8528,30 +8589,20 @@ fn publish_admitted<Device: BlockDevice>(
             place_in_publish,
             back_chain,
             filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
-            new_tree_table: tree_table_pointer,
-            new_mapping_root: mapping_pointer,
-            new_tree_identifier_watermark: plan.tree_identifier_watermark,
-            new_rollback_floor: plan.rollback_floor,
+            // 新根段七项取这次发布写的根上的（D23（journal 的角色与格式） 已定项 15，用户 2026-10-02「回到甲」）。
+            new_tree_table: root.tree_table,
+            new_mapping_root: root.mapping_root,
+            new_tree_identifier_watermark: root.tree_identifier_watermark,
+            new_rollback_floor: root.rollback_floor,
+            new_instance_table: root.instance_table,
+            new_allocation_record_tree_root: root.allocation_record_tree_root,
+            new_unmount_marker: root.unmount_marker,
             named: named_roles.iter().copied().map(named_unit_of).collect(),
         };
         let bytes = record.to_bytes();
         written_records.push(WrittenJournalRecord { record, bytes });
     }
     let last_counter_of_this_publish = plan.counter + (records_in_this_publish - 1);
-    let root = RootRecord {
-        filesystem_identifier: *filesystem_identifier,
-        unmount_marker,
-        instance,
-        checkpoint_txg: txg,
-        tree_table: tree_table_pointer,
-        tree_identifier_watermark: plan.tree_identifier_watermark,
-        rollback_floor: plan.rollback_floor,
-        instance_table: instance_table_pointer,
-        mapping_root: mapping_pointer,
-        // 带文件的一版的分配记录树住树表条目（D8（核心索引结构） 已定项 8）：根记录这一项恒全零，
-        // 两处都写就成了同一个量的两份手抄。`root_record_of_a_file_version_leaves_the_allocation_record_tree_pointer_zero` 钉住。
-        allocation_record_tree_root: NodePointer::empty_root(),
-    };
     // 持久顺序（D16（发布语义） 已定项 7）：这次重写的单元 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障。
     // 这里只装、不落盘：调用方拿它落盘，中途失败时这次已记的写进失败账（增补 2 第 20b 行）、这次发布冻结等原样重发。
     let writes = PublishWrites {
@@ -9109,10 +9160,10 @@ mod tests {
             .collect()
     }
 
-    /// 末条再跨记录（D23（journal 的角色与格式） 已定项 17）：最后一个事务要点名的项多于一条记录装得下的 67 项时，按 bump 次序
+    /// 末条再跨记录（D23（journal 的角色与格式） 已定项 17）：最后一个事务要点名的项多于一条记录装得下的 64 项时，按 bump 次序
     /// 装满一条再开下一条，这几条都属于最后一个事务；前 N − 1 个数据单元各一条、各是自己那个事务。
     /// 两个数据单元 + extent 根 + 130 片叶容器 + 5 个角色 = 138 个：第一条只点名数据单元 0（事务 0），
-    /// 最后一个事务要点名 137 项 ⇒ 67 + 67 + 3 三条（事务都是 1）。不写数据单元的 68 个 ⇒ 67 + 1 两条（事务都是 0）。
+    /// 最后一个事务要点名 137 项 ⇒ 64 + 64 + 9 三条（事务都是 1）。不写数据单元的 65 个 ⇒ 64 + 1 两条（事务都是 0）。
     /// 装得下的照旧：三个数据单元 ⇒ 三条三个事务，末条点名数据单元 2 与 6 个共享角色。
     #[test]
     fn the_last_transaction_spills_over_as_many_records_as_its_named_units_need_and_they_all_share_its_transaction(
@@ -9124,7 +9175,7 @@ mod tests {
         );
         assert_eq!(
             records_with_data.iter().map(Vec::len).collect::<Vec<_>>(),
-            vec![1, 67, 67, 3],
+            vec![1, 64, 64, 9],
             "数据单元 0 一条；最后一个事务 137 项装满两条再开第三条"
         );
         assert_eq!(
@@ -9145,8 +9196,8 @@ mod tests {
             "跨出去的两条属于最后一个事务"
         );
 
-        let spilling_without_data = rewritten_roles(0, 62);
-        assert_eq!(spilling_without_data.len(), 68);
+        let spilling_without_data = rewritten_roles(0, 59);
+        assert_eq!(spilling_without_data.len(), 65);
         let records_without_data = roles_named_by_each_record_of_the_publish(
             &spilling_without_data,
             JournalRecordNamedEntryCapacity::FromTheRecordFormat,
@@ -9156,7 +9207,7 @@ mod tests {
                 .iter()
                 .map(Vec::len)
                 .collect::<Vec<_>>(),
-            vec![67, 1]
+            vec![64, 1]
         );
         assert_eq!(
             transaction_offset_of_each_record_of_the_publish(
diff --git a/crates/singlefs-format/src/lib.rs b/crates/singlefs-format/src/lib.rs
index 74f332a..d047b46 100644
--- a/crates/singlefs-format/src/lib.rs
+++ b/crates/singlefs-format/src/lib.rs
@@ -215,17 +215,18 @@ pub const JOURNAL_RECORD_BYTES: u64 = 4096;
 /// journal 记录头的十个基础字段（D23（journal 的角色与格式） 已定项 4）。format-const: JOURNAL_HEADER_TEN_FIELD_BYTES
 pub const JOURNAL_HEADER_TEN_FIELD_BYTES: u64 = 78;
 
-/// journal 记录头的新根段：树表指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8（D23（journal 的角色与格式） 已定项 15）。format-const: JOURNAL_NEW_ROOT_SEGMENT_BYTES
-pub const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u64 = 188;
+/// journal 记录头的新根段：树表指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8 + 实例表指针 86 + 分配记录树根指针 86 + 卸载记号 4
+/// （D23（journal 的角色与格式） 已定项 15；后三项随用户 2026-10-02「回到甲」加，卸载记号 4 字节是判决没单独定宽、照第一轮正推腿的算法取的）。format-const: JOURNAL_NEW_ROOT_SEGMENT_BYTES
+pub const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u64 = 364;
 
-/// journal 记录头：78 + 事务号 8 + 提交标记 1 + 本次发布内序号 4 + 反向链 4 + 载荷校验和 4 + 新根段 188 + fsid 8 + MAC 16（D23（journal 的角色与格式） 已定项 4 / 已定项 7 / 已定项 8 / 已定项 13 / 已定项 14 注 1 / 已定项 15；fsid 与 MAC 2026-09-14 用户定案；本次发布内序号 2026-09-23 用户定案，随并行线一落地）。format-const: JOURNAL_HEADER_BYTES
-pub const JOURNAL_HEADER_BYTES: u64 = 311;
+/// journal 记录头：78 + 事务号 8 + 提交标记 1 + 本次发布内序号 4 + 反向链 4 + 载荷校验和 4 + 新根段 364 + fsid 8 + MAC 16（D23（journal 的角色与格式） 已定项 4 / 已定项 7 / 已定项 8 / 已定项 13 / 已定项 14 注 1 / 已定项 15；fsid 与 MAC 2026-09-14 用户定案；本次发布内序号 2026-09-23 用户定案，随并行线一落地）。format-const: JOURNAL_HEADER_BYTES
+pub const JOURNAL_HEADER_BYTES: u64 = 487;
 
 /// journal 点名项：位置条目 14 × 2 + 类标签 1 + 出生树 8 + 出生 txg 8 + key 尾段 10 + flags 1（D23（journal 的角色与格式） 已定项 17）。format-const: JOURNAL_NAMED_ENTRY_BYTES
 pub const JOURNAL_NAMED_ENTRY_BYTES: u64 = 56;
 
-/// 一条 4 KiB 记录装几个点名项：(4096 − 311) ÷ 56 的整数部分（D23（journal 的角色与格式） 已定项 4 / 已定项 17）。format-const: JOURNAL_NAMED_ENTRIES_PER_RECORD
-pub const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = 67;
+/// 一条 4 KiB 记录装几个点名项：(4096 − 487) ÷ 56 的整数部分（D23（journal 的角色与格式） 已定项 4 / 已定项 17）。format-const: JOURNAL_NAMED_ENTRIES_PER_RECORD
+pub const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = 64;
 
 /// journal 环从槽 1024（16 MiB）起（D23（journal 的角色与格式） 已定项 2）。
 pub const JOURNAL_RING_START_SLOT: u64 = 1024;
@@ -422,10 +423,10 @@ mod tests {
         assert_eq!(ACCOUNTING_ENTRY_BYTES, 34, "记账条目");
         assert_eq!(TREE_TABLE_ENTRY_BYTES, 200, "树表条目");
         assert_eq!(ROOT_RECORD_BYTES, 457, "根记录");
-        assert_eq!(JOURNAL_NEW_ROOT_SEGMENT_BYTES, 188, "journal 新根段");
-        assert_eq!(JOURNAL_HEADER_BYTES, 311, "journal 记录头");
+        assert_eq!(JOURNAL_NEW_ROOT_SEGMENT_BYTES, 364, "journal 新根段");
+        assert_eq!(JOURNAL_HEADER_BYTES, 487, "journal 记录头");
         assert_eq!(JOURNAL_NAMED_ENTRY_BYTES, 56, "journal 点名项");
-        assert_eq!(JOURNAL_NAMED_ENTRIES_PER_RECORD, 67, "一条记录装几个点名项");
+        assert_eq!(JOURNAL_NAMED_ENTRIES_PER_RECORD, 64, "一条记录装几个点名项");
         assert_eq!(
             JOURNAL_RING_DEFAULT_BYTES / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR,
             65536,
@@ -534,8 +535,8 @@ mod tests {
         );
         assert_eq!(
             JOURNAL_NEW_ROOT_SEGMENT_BYTES,
-            NODE_POINTER_BYTES + NODE_POINTER_BYTES + 8 + 8,
-            "新根段 = 树表指针 + 中央映射树根指针 + 树 ID 水位 8 + 回退下界 F 8"
+            NODE_POINTER_BYTES + NODE_POINTER_BYTES + 8 + 8 + NODE_POINTER_BYTES + NODE_POINTER_BYTES + 4,
+            "新根段 = 树表指针 + 中央映射树根指针 + 树 ID 水位 8 + 回退下界 F 8 + 实例表指针 + 分配记录树根指针 + 卸载记号 4（D23 已定项 15）"
         );
         assert_eq!(
             JOURNAL_NAMED_ENTRY_BYTES,
diff --git a/crates/singlefs-harness/tests/a_writable_mount_refuses_when_a_system_configuration_slot_does_not_verify.rs b/crates/singlefs-harness/tests/a_writable_mount_refuses_when_a_system_configuration_slot_does_not_verify.rs
deleted file mode 100644
index a9c72ad..0000000
--- a/crates/singlefs-harness/tests/a_writable_mount_refuses_when_a_system_configuration_slot_does_not_verify.rs
+++ /dev/null
@@ -1,100 +0,0 @@
-//! C331（择根倒挂压过已确认的写） 取甲（被攻过两轮；`research/prompts/unreadable-at-mount-r2-main-verification.md` L2）：
-//! 可写挂载判 N-配置 的见证读里，只要有一个系统配置槽读不出或自证不过，就判不出、按判据为真：重读一次，仍缺那一槽就在取号之前拒可写，
-//! 盘上逐字节不变；只读挂载照常（`mounted_read::mount_read_only`）。改之前读不出的槽不进 max、全读不出取 0 判「没见证」，
-//! 新实例会从较旧的根可写挂载，确认过的写之后被较旧实例的根压过（第一轮判决 V1、V2）。
-//! 这一形的代价（撕裂的系统配置轮换之后只剩只读挂载）是定案的一部分，见同一判决 L1。
-mod common;
-
-use common::{build_pool, parameters, BuiltPool};
-use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
-use singlefs_core::block_device::{BlockDevice, WriteDurability};
-use singlefs_core::mount::{
-    mount_writable, MountError, StillUnreadableAfterOneReread, WitnessedCounterComparison,
-};
-use singlefs_core::mounted_read::mount_read_only;
-use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
-
-/// 新池新建文件之后每块盘两槽：世代 4、世代 5（新池新建文件那次轮换）。把 `damaged_device` 世代 5 那一槽清零（撕裂那一形），另一槽照样自证得过。
-fn pool_with_one_system_configuration_slot_zeroed(
-    tag: &str,
-    damaged_device: DeviceIdentity,
-) -> BuiltPool {
-    let mut pool = build_pool(tag);
-    let slot_spacing_in_bytes = u64::from(parameters().geometry.fixed_structure_slot_spacing);
-    let devices = pool.devices.as_mut().expect("新池新建文件写完，盘还开着");
-    let (_, device) = devices
-        .iter_mut()
-        .find(|(identity, _)| *identity == damaged_device)
-        .expect("池里有这块盘");
-    device
-        .write_at(
-            DeviceOffsetInBytes(
-                (5 % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * slot_spacing_in_bytes,
-            ),
-            &vec![0u8; usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")],
-            WriteDurability::Plain,
-        )
-        .expect("世代 5 那一槽清零");
-    pool
-}
-
-fn a_writable_mount_refuses_before_acquisition_when_one_system_configuration_slot_does_not_verify(
-    tag: &str,
-    damaged_device: DeviceIdentity,
-) {
-    let mut pool = pool_with_one_system_configuration_slot_zeroed(tag, damaged_device);
-    let image_before = pool.memory_pool();
-    let mut devices = pool.reopen_recorded();
-    let refusal = mount_writable(&parameters(), &mut devices)
-        .expect_err("有一槽系统配置自证不过：判不出，重读一次仍缺，拒可写");
-    pool.devices = Some(devices);
-    let MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) = &refusal else {
-        panic!("拒因是「重读一次仍读不出更新的状态」：{refusal:?}");
-    };
-    let StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
-        first_read,
-        reread,
-    } = still_unreadable.as_ref()
-    else {
-        panic!("拒在判据 N-配置 那一格：{still_unreadable:?}");
-    };
-    for reading in [first_read, reread] {
-        assert_eq!(
-            reading.witness.comparison,
-            WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified,
-            "两遍读都判「有一槽系统配置读不出或自证不过」：{reading:?}"
-        );
-    }
-    assert!(
-        pool.memory_pool() == image_before,
-        "拒可写在取号之前：盘上逐字节不变"
-    );
-    mount_read_only(&pool.memory_pool()).expect("只读挂载照常");
-}
-
-#[test]
-fn a_writable_mount_refuses_before_acquisition_when_the_first_devices_newest_system_configuration_slot_does_not_verify(
-) {
-    a_writable_mount_refuses_before_acquisition_when_one_system_configuration_slot_does_not_verify(
-        "c331-jia-mount-device-zero",
-        DeviceIdentity(0),
-    );
-}
-
-#[test]
-fn a_writable_mount_refuses_before_acquisition_when_the_second_devices_newest_system_configuration_slot_does_not_verify(
-) {
-    a_writable_mount_refuses_before_acquisition_when_one_system_configuration_slot_does_not_verify(
-        "c331-jia-mount-device-one",
-        DeviceIdentity(1),
-    );
-}
-
-/// 对照：两槽都自证得过的同一个池照常可写挂载（判据 N-配置 为假，不重读）。
-#[test]
-fn a_writable_mount_with_every_system_configuration_slot_verified_goes_ahead() {
-    let mut pool = build_pool("c331-jia-mount-control");
-    let mut devices = pool.reopen_recorded();
-    mount_writable(&parameters(), &mut devices).expect("两槽都自证得过：照常可写挂载");
-    pool.devices = Some(devices);
-}
diff --git a/crates/singlefs-harness/tests/acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs b/crates/singlefs-harness/tests/acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs
index d3f396f..77d4d98 100644
--- a/crates/singlefs-harness/tests/acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs
+++ b/crates/singlefs-harness/tests/acquisition_refuses_a_device_whose_witness_slots_turn_unreadable_and_the_remaining_writer_panics.rs
@@ -1,9 +1,10 @@
 //! 这份文件钉的几格（实审 A3c，来历见 `research/prompts/m2-rev-a3c-implementer-report.md`）：
 //!
 //! 一、C554 乙-配置续 Q1（主 agent 定走「拒」）：取号那一刻读见证值时某块盘两槽里一份本池自证过的系统配置都读不出，
-//! 在第一个取号写之前拒——可写挂载照 C331（择根倒挂压过已确认的写） 取甲的拒因报 `MountError::NewerStateStillUnreadableAfterOneReread`
-//! （后一遍判「有一槽读不出或自证不过」；「这块盘没有自证过的系统配置」那一支已删、并进甲的拒因），
-//! `acquire_instance` 报 `InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness`。
+//! 在第一个取号写之前拒——可写挂载照见证读的拒因报 `MountError::NewerStateStillUnreadableAfterOneReread`
+//! （后一遍判「有一块盘一份自证过的系统配置都没有」：C331（择根倒挂压过已确认的写） 改取 R2 之后，盘 0 两槽都自证过时 R2 判得出，
+//! 这一支不再并进判不出，单列 `WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot`），
+//! `acquire_instance` 报 `InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness`。
 //! 改之前这块盘在见证值里不出数、取号写照做：逐盘核那一遍读得出、见证值那一遍读不出（两次读之间一次瞬时读错）就走得到。
 //! 这一核在第一道屏障之前做一遍（拒的时候录制流一步不多），屏障之后取见证值时再读一遍，对不上就不写。
 //!
@@ -253,8 +254,8 @@ fn reads_of_device_one_slots_before_the_acquisition_barrier(tag: &str) -> [u64;
     [counts[0], counts[1]]
 }
 
-/// 新池新建文件那一版可写挂载被拒时报的：读阶段判完之后取号那一刻盘 1 的槽读不出（两次读之间的瞬时读错），照 C331（择根倒挂压过已确认的写）
-/// 取甲的拒因报（用户 2026-09-28 定：「这块盘没有自证过的系统配置」那一支删掉、并进甲的拒因）——所选那一版就是它，后一遍判「有一槽读不出或自证不过」。
+/// 新池新建文件那一版可写挂载被拒时报的：读阶段判完之后取号那一刻盘 1 的槽读不出（两次读之间的瞬时读错），照见证读的拒因报
+/// （用户 2026-09-28 定并进见证读的拒因）——所选那一版就是它，后一遍判「有一块盘一份自证过的系统配置都没有」（盘 0 两槽都自证过，R2 判得出）。
 fn refused_because_device_one_slots_turned_unreadable_after_the_read_stage(
     refusal: &Result<singlefs_core::mount::Mounted, MountError>,
     first: &TransactionOutput,
@@ -272,7 +273,7 @@ fn refused_because_device_one_slots_turned_unreadable_after_the_read_stage(
             checkpoint_txg: first.root.checkpoint_txg,
         } && reread.selected_version == first_read.selected_version
             && reread.witness.comparison
-                == WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified
+                == WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot
     )
 }
 
@@ -290,7 +291,7 @@ fn describe(refusal: &Result<singlefs_core::mount::Mounted, MountError>) -> Stri
 
 /// 场景：新池新建文件之后重开、可写挂载。盘 1 两个系统配置槽从「取号写之前最后一次读」起每次都读坏
 /// （次数取另一份相同的池上照常挂载时数到的：择系统配置、取号之前的逐盘核、算号那几遍都读得出）。
-/// 预期：取号拒在第一道屏障之前，照甲的拒因报 `NewerStateStillUnreadableAfterOneReread`（后一遍判「有一槽读不出或自证不过」），
+/// 预期：取号拒在第一道屏障之前，照见证读的拒因报 `NewerStateStillUnreadableAfterOneReread`（后一遍判「有一块盘一份自证过的系统配置都没有」），
 /// `DiskSnapshot` 不变（系统配置槽、根环、录制流步数：一个写都没发），盘上一次屏障调用都没收到（录制流看不出取号那道屏障，
 /// 见 [`DeviceCountingBarriers`]）。
 /// 改之前：见证值只取盘 0 的、取号写照做，挂载做成。
@@ -377,7 +378,8 @@ fn a_device_whose_slots_turn_unreadable_after_the_acquisition_barrier_refuses_be
 }
 
 /// 场景：新池新建文件之后重开，不经可写挂载、直接 `acquire_instance`（前面没有逐盘核）；盘 1 两个系统配置槽每次都读坏。
-/// 预期：`InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness`（盘 1），`DiskSnapshot` 不变。
+/// 预期：`InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness`（盘 1；盘 0 两槽都自证过，R2 判得出），
+/// `DiskSnapshot` 不变。
 /// 改之前：见证值只取盘 0 的，两块盘都写了取号写，交出新号 2。
 #[test]
 fn acquire_instance_refuses_a_device_whose_slots_are_unreadable_before_any_step() {
@@ -396,7 +398,7 @@ fn acquire_instance_refuses_a_device_whose_slots_are_unreadable_before_any_step(
     assert!(
         matches!(
             acquisition,
-            Err(InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness {
+            Err(InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
                 device: DEVICE_WHOSE_SLOTS_TURN_UNREADABLE,
             })
         ),
diff --git a/crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs b/crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs
index 4814f95..7d62526 100644
--- a/crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs
+++ b/crates/singlefs-harness/tests/acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs
@@ -485,7 +485,10 @@ fn a_rolled_back_acquisition_writes_the_witnessed_tail_back_instead_of_zero() {
         let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
         match acquire_instance(&mut writer).expect_err("盘 1 的取号写报错，取号必须失败") {
             InstanceAcquisitionFailed::Acquisition(acquisition) => acquisition,
-            InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness {
+            InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
+                first_device_reporting_a_read_error,
+            } => panic!("这条用例里每块盘两槽都读得出自证过的系统配置，取号不该判不出：{first_device_reporting_a_read_error:?}"),
+            InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
                 device,
             } => panic!("这条用例里每块盘两槽都读得出自证过的系统配置，取号不该拒在见证值那一核：{device:?}"),
         }
@@ -528,14 +531,16 @@ fn a_rolled_back_acquisition_writes_the_witnessed_tail_back_instead_of_zero() {
     );
 }
 
-/// C331（择根倒挂压过已确认的写） 取甲（用户 2026-09-28 定；`research/prompts/unreadable-at-mount-r2-main-verification.md` L2 V4 那一支）：
-/// 取号那一刻读见证值，某块盘两槽里有一槽读不出或自证不过（这里清零世代 5 那一槽，另一槽照样自证得过），就在第一个取号写之前拒，
-/// 两块盘的系统配置槽逐字节不变。改之前（只在一份都没有时拒）这块盘照样取号、见证值缺了那一槽。
-fn an_acquisition_refuses_a_device_that_lost_only_its_newest_slot(
+/// C331（择根倒挂压过已确认的写） 改取 R2（用户 2026-10-03 定，零轮；D22（单元原子性怎么合成） 已定项 8 射程）：取号那一刻读见证值，
+/// 某块盘最新那一槽撕裂（这里清零世代 5 那一槽：读得出、自证不过，设备没报错），另一槽照样自证得过——R2 判得出（没有槽报读错），
+/// 取号照做：见证值取全部自证过的槽（另一块盘世代 5 那一槽见证新池新建文件那条记录，jsn 3），坏了那块盘的取号写落在撕裂的那一槽
+/// （世代 4 + 1 = 5，槽号 5 mod 2 = 1），把它重写成自证过的；另一块盘照常写世代 6。取甲时这一形在任何写之前拒、之后只剩只读挂载（C596）。
+fn an_acquisition_rewrites_the_torn_newest_slot_of_a_device(
     tag: &str,
     damaged_device: DeviceIdentity,
 ) {
     let mut pool = build_pool(tag);
+    let new_pool_file_creation_counter = pool.output.record.counter;
     let slot_spacing_in_bytes = u64::from(geometry().fixed_structure_slot_spacing);
     {
         let devices: &mut Vec<(DeviceIdentity, Recorded)> =
@@ -554,48 +559,57 @@ fn an_acquisition_refuses_a_device_that_lost_only_its_newest_slot(
             )
             .expect("世代 5 那一槽清零");
     }
-    let before_the_acquisition = pool.memory_pool();
-    let slots_before: Vec<Vec<SlotReading>> = DISKS
-        .into_iter()
-        .map(|disk| slot_readings_of(&before_the_acquisition, disk))
-        .collect();
     let pool_parameters = parameters();
     let mut devices = pool.reopen_recorded();
-    let refusal = {
+    let acquired = {
         let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
-        acquire_instance(&mut writer).expect_err("有一槽读不出：取号在任何写之前拒")
+        acquire_instance(&mut writer)
     };
     pool.devices = Some(devices);
     assert!(
-        matches!(
-            refusal,
-            InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness { device }
-                if device == damaged_device
-        ),
-        "拒因点名坏了一槽的那块盘：{refusal:?}"
+        matches!(acquired, Ok(InstanceGeneration(2))),
+        "撕裂的一槽不是读错，R2 判得出：取号照做、取到 2；得到 {acquired:?}"
     );
-    let after_the_refusal = pool.memory_pool();
-    let slots_after: Vec<Vec<SlotReading>> = DISKS
-        .into_iter()
-        .map(|disk| slot_readings_of(&after_the_refusal, disk))
-        .collect();
-    assert_eq!(slots_after, slots_before, "拒的时候一个取号写都没写");
+    let after = pool.memory_pool();
+    for disk in DISKS {
+        let expected_generations = if disk == damaged_device {
+            [4, 5]
+        } else {
+            [5, 6]
+        };
+        let readings = slot_readings_of(&after, disk);
+        assert_eq!(
+            readings
+                .iter()
+                .map(|reading| reading.slot_generation)
+                .collect::<Vec<_>>(),
+            expected_generations.to_vec(),
+            "盘 {disk:?} 两槽都自证得过：坏了的那一槽被取号写重写（{readings:?}）"
+        );
+        let acquisition_slot = readings.last().expect("两槽都在");
+        assert_eq!(
+            (
+                acquisition_slot.journal_instance,
+                acquisition_slot.journal_tail
+            ),
+            (InstanceGeneration(2), new_pool_file_creation_counter),
+            "取号写带新号 2 与见证值（全部自证过的槽 tail 的最大值 = 新池新建文件那条记录的 jsn）"
+        );
+    }
 }
 
 #[test]
-fn an_acquisition_refuses_the_first_device_that_lost_only_its_newest_slot_before_writing_anything()
-{
-    an_acquisition_refuses_a_device_that_lost_only_its_newest_slot(
-        "c331-jia-acquisition-device-zero",
+fn an_acquisition_rewrites_the_torn_newest_slot_of_the_first_device() {
+    an_acquisition_rewrites_the_torn_newest_slot_of_a_device(
+        "c331-r2-acquisition-device-zero",
         DeviceIdentity(0),
     );
 }
 
 #[test]
-fn an_acquisition_refuses_the_second_device_that_lost_only_its_newest_slot_before_writing_anything()
-{
-    an_acquisition_refuses_a_device_that_lost_only_its_newest_slot(
-        "c331-jia-acquisition-device-one",
+fn an_acquisition_rewrites_the_torn_newest_slot_of_the_second_device() {
+    an_acquisition_rewrites_the_torn_newest_slot_of_a_device(
+        "c331-r2-acquisition-device-one",
         DeviceIdentity(1),
     );
 }
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 26eedde..daff3ae 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -1108,7 +1108,7 @@ fn known_bad_images(clean: &MemoryPool) -> Vec<(&'static str, Mutation)> {
             }),
         ),
         // 盘 0 环内偏移 4096 那条记录（w4，暖机第二条空发布）的反向链字段改成 0，头校验和重封：
-        // 它不是本实例第一条（本实例第一条是 w1、计数器 1），链值必须等于 CRC32C(w1 的 311 字节头)。
+        // 它不是本实例第一条（本实例第一条是 w1、计数器 1），链值必须等于 CRC32C(w1 的 487 字节头)。
         (
             "I-8.6",
             Box::new(|image: &mut MemoryPool| {
@@ -1535,9 +1535,9 @@ fn two_committed_transactions_sharing_one_number_side_by_side(image: &mut Memory
     set_transaction_number_of_journal_record(image, 4, 2, 1);
 }
 
-/// 记录头里 fsid 那 8 字节的偏移：事务段 78 + 事务号 8 + 提交标记 1 + 本次发布内序号 4 + 反向链 4 + 载荷校验和 4 + 新根段 188
-/// （D23（journal 的角色与格式） 已定项 4 的字段表）。
-const JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET: usize = 287;
+/// 记录头里 fsid 那 8 字节的偏移：事务段 78 + 事务号 8 + 提交标记 1 + 本次发布内序号 4 + 反向链 4 + 载荷校验和 4 + 新根段 364
+/// （D23（journal 的角色与格式） 已定项 4 / 已定项 15 的字段表）。
+const JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET: usize = 463;
 
 /// 记录头里反向链那 4 字节的偏移：事务段 78 + 事务号 8 + 提交标记 1 + 本次发布内序号 4（D23（journal 的角色与格式） 已定项 4 / 已定项 8）。
 const JOURNAL_RECORD_BACK_CHAIN_OFFSET: usize = 91;
@@ -5064,7 +5064,7 @@ fn unit_checksums(pool: &MemoryPool, units: &[(u64, usize)]) -> Vec<(u64, u32)>
 /// 本次发布内序号 4 + 反向链 4 之后是载荷校验和，罩记录头之后的点名项）。
 const JOURNAL_RECORD_NAMED_COUNT_OFFSET: usize = 12;
 const JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET: usize = 95;
-const JOURNAL_RECORD_HEADER_BYTES: usize = 311;
+const JOURNAL_RECORD_HEADER_BYTES: usize = 487;
 const JOURNAL_NAMED_ENTRY_BYTES: usize = 56;
 
 /// 几个单元的整单元校验和变了（`changes`：(起点槽, 旧, 新)）：journal 环里引用它们的位置条目——记录新根段里的树表指针与映射根指针、
diff --git a/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs b/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
index b25c478..da58223 100644
--- a/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
+++ b/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
@@ -104,12 +104,12 @@ const UNIT_CLASS_INDEX_NODE: u8 = 2;
 const UNIT_CLASS_PACKED: u8 = 3;
 
 /// journal 记录（D23（journal 的角色与格式） 已定项 4 / 已定项 17）：点名项数 4 在 12、计数器 6 在 20、
-/// 自证校验和 32 在 46（罩整条 4096）、载荷校验和 4 在 95（罩头之后的点名项）、头 311、点名项 56（两条位置条目在前）。
+/// 自证校验和 32 在 46（罩整条 4096）、载荷校验和 4 在 95（罩头之后的点名项）、头 487、点名项 56（两条位置条目在前）。
 const JOURNAL_RECORD_NAMED_COUNT_OFFSET: usize = 12;
 const JOURNAL_RECORD_COUNTER_OFFSET: usize = 20;
 const JOURNAL_RECORD_HEADER_CHECKSUM_OFFSET: usize = 46;
 const JOURNAL_RECORD_PAYLOAD_CHECKSUM_OFFSET: usize = 95;
-const JOURNAL_RECORD_HEADER_BYTES: usize = 311;
+const JOURNAL_RECORD_HEADER_BYTES: usize = 487;
 const JOURNAL_NAMED_ENTRY_BYTES: usize = 56;
 const JOURNAL_RECORD_BYTES: usize = 4096;
 
diff --git a/crates/singlefs-harness/tests/common/mod.rs b/crates/singlefs-harness/tests/common/mod.rs
index eb86f3d..5ee88ce 100644
--- a/crates/singlefs-harness/tests/common/mod.rs
+++ b/crates/singlefs-harness/tests/common/mod.rs
@@ -491,8 +491,9 @@ fn journal_record_bytes_in_the_default_ring(
     .expect("环里那一格读得到")
 }
 
-/// 根环里每条自证过的根（`readable_roots`），把两块盘 journal 默认环里同 (实例, txg) 的记录的新根段改成照它的那四段
-/// （树表指针、中央映射树根指针、树 ID 水位、回退下界 F；I-8.10（记录新根段与同代根相同），D23（journal 的角色与格式） 已定项 4：
+/// 根环里每条自证过的根（`readable_roots`），把两块盘 journal 默认环里同 (实例, txg) 的记录的新根段改成照它的那七段
+/// （树表指针、中央映射树根指针、树 ID 水位、回退下界 F、实例表指针、分配记录树根指针、卸载记号；I-8.10（记录新根段与同代根相同）
+/// 判前四段，D23（journal 的角色与格式） 已定项 15 施加时七段都取记录里的，D23（journal 的角色与格式） 已定项 4：
 /// 一次发布的每条记录带同一份新根段），`JournalRecord::to_bytes` 重封；再按计数器从小到大，前一条被改过、又与这一条同实例的，
 /// 这一条的反向链按前一条改后的记录头重算、重封（I-8.6（反向链算法）只链本实例内逻辑前一条）。
 /// 坏镜像改了根（新整单元校验和沿引用链补到根槽、改树表指针）之后调它：盘上留下的就是「写者从一开始就写出了这样的根」的样子，
@@ -540,6 +541,9 @@ pub fn copy_each_root_into_the_new_root_segment_of_its_journal_records(
                 rewritten.new_mapping_root = root.mapping_root;
                 rewritten.new_tree_identifier_watermark = root.tree_identifier_watermark;
                 rewritten.new_rollback_floor = root.rollback_floor;
+                rewritten.new_instance_table = root.instance_table;
+                rewritten.new_allocation_record_tree_root = root.allocation_record_tree_root;
+                rewritten.new_unmount_marker = root.unmount_marker;
             }
             if let Some((previous_instance, previous_bytes)) = &rewritten_previous_record {
                 if *previous_instance == record.instance {
@@ -794,8 +798,8 @@ pub fn abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(
     // 见证在时可写挂载重读一次仍读不出它就拒可写，`MountError::NewerStateStillUnreadableAfterOneReread`）：崩在 `newest` 的根槽 FUA 之后、
     // 它那次系统配置轮换写之前。轮换写那一槽（每块盘世代号最大的那一槽，槽号 = 世代号 mod 2）因此还是再前一次轮换留下的内容：
     // 世代号 g − 2、自证照过。录制流只记内容哈希、拿不回那份字节，这里拿另一槽（世代号 g − 1，见证的是 `newest` 前一次发布）的内容改世代号为 g − 2
-    // 重新编码写回——见证值与那份真内容一样停在 `newest` 之前，两槽都自证得过。不清零：C331（择根倒挂压过已确认的写） 取甲之后
-    // 见证读有一槽读不出或自证不过就拒可写（`WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified`），清零造的是撕裂轮换那一形。
+    // 重新编码写回——见证值与那份真内容一样停在 `newest` 之前，两槽都自证得过。不清零：清零造的是撕裂轮换那一形（取甲时那一形拒可写；
+    // C331（择根倒挂压过已确认的写） 改取 R2 之后不拒，但见证值取的是另一槽、与真内容不同）。
     let slot_spacing_in_bytes = u64::from(publish_parameters.geometry.fixed_structure_slot_spacing);
     let rotation_not_written_of_each_device: Vec<(
         singlefs_core::address::DeviceOffsetInBytes,
diff --git a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
index c0542e9..e5a4be4 100644
--- a/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
+++ b/crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs
@@ -38,7 +38,7 @@ use singlefs_core::recovery::{
     PoolReader, RecoveryFailure, RecoveryOutcome, RootRingSlotReading,
     SystemConfigurationValueOutsideWhatThisReaderAccepts,
 };
-use singlefs_core::root_record::RootRecord;
+use singlefs_core::root_record::{RootRecord, UnmountMarker};
 use singlefs_core::root_ring::{slot_offset, RootRingSlot, RootRingSlotsPerRegion};
 use singlefs_core::system_configuration::{
     SystemConfiguration, SystemImmutableSizes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET,
@@ -1137,6 +1137,9 @@ fn a_journal_record_whose_new_tree_table_pointer_carries_compression_is_not_pars
         new_tree_identifier_watermark:
             singlefs_format::TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
         new_rollback_floor: CheckpointTxg(0),
+        new_instance_table: NodePointer::empty_root(),
+        new_allocation_record_tree_root: NodePointer::empty_root(),
+        new_unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
         named: Vec::new(),
     };
     let mut bytes = record.to_bytes();
diff --git a/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs b/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs
index f2ec223..45b8d31 100644
--- a/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs
+++ b/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs
@@ -592,28 +592,14 @@ fn a_device_left_with_one_self_verified_system_configuration_slot_is_not_refused
             Vec::<String>::new(),
             "{entry:?}：做完之后池级 checker 不红"
         );
-        // 入口的写里有一次系统配置轮换就把写坏的那一槽重写了，之后照常可写挂载；入口一次轮换都没写（抬 F 到准入上限而没有可抬的），
-        // 那一槽还自证不过，之后的可写挂载按 C331（择根倒挂压过已确认的写） 取甲拒（见证读缺一槽判不出，用户 2026-09-28 定）。
-        let slots_of_device_one_verified_afterwards = verified_system_configuration_slots(
-            &devices,
-            DeviceIdentity(1),
-            spacing,
-            &pool.parameters.filesystem_identifier,
-        )
-        .len();
+        // 入口的写里有一次系统配置轮换就把写坏的那一槽重写了；入口一次轮换都没写（抬 F 到准入上限而没有可抬的），那一槽还自证不过——
+        // 写坏的一槽读得出、自证不过，设备没报错：C331（择根倒挂压过已确认的写） 改取 R2 之后不判不出（用户 2026-10-03 定），
+        // 两种都照常可写挂载（取甲时后一种拒）。
         let outcome = writable_mount_outcome_on_a_copy_of(&pool.parameters, &devices);
-        if slots_of_device_one_verified_afterwards == 2 {
-            assert!(
-                outcome.is_ok(),
-                "{entry:?}：写坏的那一槽已被轮换重写，之后照常可写挂载：{outcome:?}"
-            );
-        } else {
-            assert!(
-                outcome.as_ref().is_err_and(|mount_refusal| mount_refusal
-                    .contains("SomeSystemConfigurationSlotUnreadOrUnverified")),
-                "{entry:?}：写坏的那一槽还自证不过，之后的可写挂载按甲拒：{outcome:?}"
-            );
-        }
+        assert!(
+            outcome.is_ok(),
+            "{entry:?}：写坏的那一槽读得出、只是自证不过，R2 判得出，之后照常可写挂载：{outcome:?}"
+        );
     }
 }
 
diff --git a/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs b/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs
index e4542ab..19daba1 100644
--- a/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs
+++ b/crates/singlefs-harness/tests/fsync_drop_and_devices_without_the_selected_version.rs
@@ -1570,8 +1570,8 @@ fn refusal_by_devices_without_the_selected_version(
 /// 读阶段交出的所选那一版是 `selected_version`、它那次发布末条记录读得出、计数器 `selected_version_last_record_counter`，另一块盘的系统配置
 /// 见证过计数器 `witnessed_journal_counter`（更大）；重读一遍读数相同。乙在读阶段之后、取号之前逐盘核之前判，所以这几格走不到逐盘核
 /// （调查 `research/prompts/m2-investigate-three-reds-report.md` 第二节）。这几格里有一块盘的系统配置槽读不出（每次读都报错、或空盘），
-/// C331（择根倒挂压过已确认的写） 取甲之后判据先按「见证读缺一槽，判不出」为真（用户 2026-09-28 定），读数里的比较记成那一支；
-/// 见证值照旧是另一块盘读得出的最大 tail，所选那一版末条的计数器 `selected_version_last_record_counter` 只进说明文字。
+/// 另一块盘两槽都自证过：C331（择根倒挂压过已确认的写） 改取 R2 之后判得出（用户 2026-10-03 定），读数里的比较是拿见证值比所选那一版末条那一支；
+/// 见证值是另一块盘读得出的最大 tail。
 fn assert_refused_by_the_witness_of_a_newer_publish(
     error: &MountError,
     what: &str,
@@ -1583,7 +1583,9 @@ fn assert_refused_by_the_witness_of_a_newer_publish(
         selected_version,
         witness: NewerPublishWitness {
             witnessed_journal_counter,
-            comparison: WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified,
+            comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
+                selected_version_last_record_counter,
+            },
         },
     };
     assert!(
@@ -1700,8 +1702,9 @@ fn blank_device_refuses_the_writable_mount_by_name_before_any_write() {
         image_before_the_mount_with_device_zero_blank,
         "盘 0 是空盘：拒在取号之前，两块盘逐字节不变"
     );
-    // 盘 1 是空盘：见证读缺盘 1 两槽，C331（择根倒挂压过已确认的写） 取甲之后判不出、按真，重读仍缺，同样在逐盘核之前拒（用户 2026-09-28 定）；
-    // 见证值取盘 0 读得出的最大 tail 5，所选那一版 (1, 5) 末条 5。
+    // 盘 1 是空盘：没有槽报读错，R2 判得出；见证值取盘 0 读得出的最大 tail 5，所选那一版 (1, 5) 末条 5，判据 N-配置 为假。
+    // 往下逐盘核：盘 1 一份自证过的系统配置都没有（C554 乙-配置续 Q1，不「可见」），照见证读的拒因报，后一遍判
+    // 「有一块盘一份自证过的系统配置都没有」（用户 2026-09-28 定并进见证读的拒因）。
     let mut devices_with_device_one_blank = vec![
         (DEVICE_ZERO, sparse_from(&pool.full, DEVICE_ZERO)),
         (DEVICE_ONE, blank_device()),
@@ -1711,12 +1714,34 @@ fn blank_device_refuses_the_writable_mount_by_name_before_any_write() {
     let refusal_with_device_one_blank =
         mount_writable(&pool.parameters, &mut devices_with_device_one_blank)
             .expect_err("盘 1 是空盘，可写挂载要拒");
-    assert_refused_by_the_witness_of_a_newer_publish(
-        &refusal_with_device_one_blank,
-        "盘 1 是空盘",
-        version_key(1, 5),
-        5,
-        5,
+    let first_read = SelectedVersionAgainstTheWitness {
+        selected_version: version_key(1, 5),
+        witness: NewerPublishWitness {
+            witnessed_journal_counter: 5,
+            comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
+                selected_version_last_record_counter: 5,
+            },
+        },
+    };
+    let reread = SelectedVersionAgainstTheWitness {
+        selected_version: version_key(1, 5),
+        witness: NewerPublishWitness {
+            witnessed_journal_counter: 5,
+            comparison:
+                WitnessedCounterComparison::SomeDeviceWithoutAnySelfVerifiedSystemConfigurationSlot,
+        },
+    };
+    assert!(
+        matches!(
+            &refusal_with_device_one_blank,
+            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
+                if **still_unreadable
+                    == StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
+                        first_read,
+                        reread,
+                    }
+        ),
+        "盘 1 是空盘：读阶段判得出、为假，逐盘核点名盘 1 一份自证过的系统配置都没有：{refusal_with_device_one_blank:?}"
     );
     assert_eq!(
         memory_pool_of(&devices_with_device_one_blank),
@@ -2112,7 +2137,7 @@ fn device_one_stale_on_the_version_without_file_refuses_the_writable_mount_namin
 /// 第一个文件（txg 3）→ 覆盖写 txg 4 之后，盘 1 的每一次读都报错、写照常。交进来的是两块盘，w 的下限那一判不拦；
 /// 所选那一版是 (1, 3)、不是 (1, 4)：txg 4 的根只落在区域 1（盘 1，读不出），txg 4 那条记录点名的单元在盘 1 上那一份验不了、
 /// 那次发布不施加（同「盘 0 是空盘」那一格所选的是 (1, 4) 而不是 (1, 5) 的道理）。盘 0 的系统配置见证过 txg 4 那次发布
-/// （jsn 4 > 所选那一版末条 3），可写挂载在取号之前被 C554 乙拒——逐盘核（点名盘 1「没有自证过的系统配置」）在乙之后不可达，
+/// （jsn 4 > 所选那一版末条 3；盘 1 两槽报读错而盘 0 两槽都自证过，R2 判得出），可写挂载在取号之前被 C554 乙拒——逐盘核（点名盘 1「没有自证过的系统配置」）在乙之后不可达，
 /// 那一判由 `blank_device_refuses_the_writable_mount_by_name_before_any_write` 的「盘 1 是空盘」那一格钉着。一个写、一道屏障都没落下。
 #[test]
 fn mount_writable_while_every_read_of_device_one_fails_is_refused_by_the_witness_before_any_write()
diff --git a/crates/singlefs-harness/tests/instance_acquisition.rs b/crates/singlefs-harness/tests/instance_acquisition.rs
index f283a4b..a0000cf 100644
--- a/crates/singlefs-harness/tests/instance_acquisition.rs
+++ b/crates/singlefs-harness/tests/instance_acquisition.rs
@@ -115,7 +115,10 @@ fn acquire(
     let mut pool = PoolWriter::new(&parameters, devices);
     acquire_instance(&mut pool).map_err(|failure| match failure {
         InstanceAcquisitionFailed::Acquisition(acquisition) => acquisition,
-        InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness {
+        InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
+            first_device_reporting_a_read_error,
+        } => panic!("这条用例里每块盘两槽都读得出自证过的系统配置，取号不该判不出：{first_device_reporting_a_read_error:?}"),
+        InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
             device,
         } => panic!("这条用例里每块盘两槽都读得出自证过的系统配置，取号不该拒在见证值那一核：{device:?}"),
     })
diff --git a/crates/singlefs-harness/tests/many_inodes_write_and_read.rs b/crates/singlefs-harness/tests/many_inodes_write_and_read.rs
index c0d8a51..0ccfc25 100644
--- a/crates/singlefs-harness/tests/many_inodes_write_and_read.rs
+++ b/crates/singlefs-harness/tests/many_inodes_write_and_read.rs
@@ -447,9 +447,9 @@ fn assert_no_invariant_is_violated(image: &MemoryPool, what: &str) {
     }
 }
 
-/// 一次建的 inode 多到这次要点名的单元超过一条 journal 记录装得下的 67 项：建 inode 是一个事务、一个数据单元都不写，
+/// 一次建的 inode 多到这次要点名的单元超过一条 journal 记录装得下的 64 项：建 inode 是一个事务、一个数据单元都不写，
 /// 它点名的全是共享的提交内生块。D23（journal 的角色与格式） 已定项 17（用户 2026-09-24 定）：**末条再跨记录**——
-/// 按 bump 次序装满 67 项再开下一条；两条都属于这一个事务（已定项 7：同一事务的记录共享事务号，提交标记只在它的最后一条），
+/// 按 bump 次序装满 64 项再开下一条；两条都属于这一个事务（已定项 7：同一事务的记录共享事务号，提交标记只在它的最后一条），
 /// 本次发布内序号 1、2（已定项 4），只有第二条带「本次发布末条」标志（记录标志位 0，已定项 17）。
 /// 盘上的字段用 checker 的独立解析从盘上读（不信写者交回的）。之后：
 /// - 发布做完：冷启动读回 1 + N 条 inode 记录；池级 checker 一条违例都没有，I-8.8（前缀里的事务不被切开） 这一回有了对象
@@ -464,10 +464,10 @@ fn publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transa
     let before = pool.output.clone();
     // 非叶容器的角色有十一个：inode 根、记账树、映射树、树表，加分配记录树按位置寻址（D8（核心索引结构） 已定项 14）重写的七个节点——
     // 这次新写的五十几片容器从 50253 往后排、越过 50344，改的记录落在两块盘各自的叶 61 与叶 62 里，
-    // 加两块盘各自的第 1 层节点 0 与根 ⇒ 容器数超过 67 − 11 = 56 就点不下一条记录。
+    // 加两块盘各自的第 1 层节点 0 与根 ⇒ 容器数超过 64 − 11 = 53 就点不下一条记录。
     let containers_that_fit = JOURNAL_NAMED_ENTRIES_PER_RECORD - 11;
     let new_inode_count = (containers_that_fit + 1) * INODE_LEAF_RECORDS - INODES_BEFORE;
-    let named_unit_capacity = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("67");
+    let named_unit_capacity = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("64");
 
     let after = create_inodes(&mut pool, new_inode_count, FIXED_WRITE_TIME_SECONDS + 120)
         .expect("点名项装不下一条记录时末条再跨记录，不拒");
@@ -475,12 +475,12 @@ fn publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transa
     assert_eq!(
         after.rewritten.len(),
         named_unit_capacity + 1,
-        "这次重写 57 片叶容器 + 11 个非叶容器角色 = 68 个单元"
+        "这次重写 54 片叶容器 + 11 个非叶容器角色 = 65 个单元"
     );
     assert_eq!(
         after.earlier_records_of_this_publish.len(),
         1,
-        "68 项 = 装满的一条 67 项 + 跨出去的一条 1 项"
+        "65 项 = 装满的一条 64 项 + 跨出去的一条 1 项"
     );
     let first_counter = after.earlier_records_of_this_publish[0].record.counter;
     assert_eq!(
@@ -522,7 +522,7 @@ fn publish_naming_more_units_than_one_journal_record_holds_spills_its_one_transa
             (txg, transaction, 0, 1, 0, named_unit_capacity),
             (txg, transaction, 1, 2, 1, 1),
         ],
-        "(txg, 事务号, 提交标记, 本次发布内序号, 记录标志, 点名项数)：同一事务、提交标记与末条标志只在第二条，第一条装满 67 项"
+        "(txg, 事务号, 提交标记, 本次发布内序号, 记录标志, 点名项数)：同一事务、提交标记与末条标志只在第二条，第一条装满 64 项"
     );
     let named_in_bump_order: Vec<_> = after.earlier_records_of_this_publish[0]
         .record
diff --git a/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs b/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
index 32a7804..4faf102 100644
--- a/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
+++ b/crates/singlefs-harness/tests/new_pool_file_creation_publish.rs
@@ -64,7 +64,7 @@ const E142_FILESYSTEM_IDENTIFIER: [u8; 16] = [
 ];
 /// E142 装置里的固定写入时间（`FIXED_WRITE_TIME_SECONDS`）。
 const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
-/// 新池新建文件那条记录的反向链 = CRC32C(暖机第二条记录的 311 字节头)。E142 产物第 37 行（`back_chain=628216162`）是 307 字节头下的值；
+/// 新池新建文件那条记录的反向链 = CRC32C(暖机第二条记录的 487 字节头)。E142 产物第 37 行（`back_chain=628216162`）是 307 字节头下的值；
 /// 记录头加本次发布内序号（D23（journal 的角色与格式） 已定项 4）之后是 1057457588（那一段独立换算：307 字节头下的三条记录在提交标记
 /// 之后插 4 字节序号 1）。记录标志位 0（原「填充 1」那个字节，偏移 7，D23（journal 的角色与格式） 已定项 4 / 已定项 17）在
 /// w1、w4、t9 上都写 1（每次只有一条记录的发布那一条也带「本次发布末条」标志）之后，这个值又由一段独立的换算得出：
@@ -73,7 +73,11 @@ const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
 /// （换算脚本与原样输出在实二十的报告里）。E142 装置按新代码重跑之后，产物里那一行应当逐字是这个数。
 /// 树 ID 整体挪到 65536 起（D8（核心索引结构） 已定项 11，用户 2026-10-01 定案）之后，暖机第二条记录头里新根段的树 ID 水位从 11 变 65536，
 /// 钉值随之从 3937788698 换成 2213468031（那一批实现员的副本上同一段流程现算，没有独立换算；E142 装置按新号重跑之后要逐字对上）。
-const E142_BACK_CHAIN_OF_NEW_POOL_FILE_CREATION: u32 = 2_213_468_031;
+/// 新根段加实例表指针、分配记录树根指针与卸载记号三项（头 311 → 487，D23（journal 的角色与格式） 已定项 4 / 已定项 15，用户 2026-10-02
+/// 「回到甲」）之后，暖机第二条记录头多出那 176 字节（实例表指针照 mkfs 那一片、分配记录树根全零、卸载记号 0），钉值从 2213468031 换成
+/// 2660362816：实现员 2026-10-03 在副本上同一段流程现算；同一条用例里每条记录的反向链另由池级 checker 自己那份逐位 CRC-32C 对盘上那 487 字节头
+/// 现算一遍再比（`back_chain_of_record_header`），两边对得上，不是只抄实现的数。E142 装置按新头宽重跑之后要逐字对上。
+const E142_BACK_CHAIN_OF_NEW_POOL_FILE_CREATION: u32 = 2_660_362_816;
 
 fn image_path(tag: &str, device: u32) -> PathBuf {
     let sequence = IMAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
@@ -526,7 +530,7 @@ fn root_slots_system_configurations_and_journal_ring_hold_the_published_state()
             .map_or(0, back_chain_of_record_header);
         assert_eq!(
             view.back_chain, expected_back_chain,
-            "反向链 = 本实例内逻辑前一条记录的 311 字节头的 CRC-32C"
+            "反向链 = 本实例内逻辑前一条记录的 487 字节头的 CRC-32C"
         );
         assert_eq!(
             view.ordinal_within_publish, 1,
@@ -560,7 +564,7 @@ fn root_slots_system_configurations_and_journal_ring_hold_the_published_state()
             );
             assert_eq!(
                 view.back_chain, E142_BACK_CHAIN_OF_NEW_POOL_FILE_CREATION,
-                "与 311 字节头、记录标志写 1 之后独立换算出的 back_chain 逐字相同（E142 第八次跑产物第 37 行是 307 字节头、偏移 7 写 0 下的值）"
+                "与 487 字节头下现算、checker 独立 CRC 核过的 back_chain 逐字相同（E142 第八次跑产物第 37 行是 307 字节头、偏移 7 写 0 下的值）"
             );
             assert_eq!(
                 &view.new_root_segment[..86],
diff --git a/crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs b/crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs
new file mode 100644
index 0000000..4db7248
--- /dev/null
+++ b/crates/singlefs-harness/tests/one_revolution_window_after_a_hole_in_the_root_ring.rs
@@ -0,0 +1,146 @@
+//! 根环上有洞之后的回收门槛取一圈窗口（D16（发布语义） 已定项 1「可再分配 = 已释放 ∧ 释放代 ≤ max(F_生效, 一圈窗口里最旧有效根)」，
+//! 用户 2026-10-02 定；`research/prompts/m3-ring-r3-main-verification.md` 第二节第 3–5 行）。
+//!
+//! 盘面（与崩溃注入种子 0 第 135 段那一形同形，`/tmp/claude-1000/inject6-findings/report.md` 第三节）：实例 1 在 24 槽的根环上发到 txg 30，
+//! txg 31 那次发布崩在「记录已持久、根槽没持久」（根槽与之后的系统配置轮换都没落：那一槽里留着上一圈的 txg 7 的根）。
+//! 可写挂载（实例 2）恢复由记录施加出 txg 31 那一版，写行 txg 32、暖机 txg 33、34 盖掉 txg 8、9、10 的槽；之后覆盖写 txg 35、36、37。
+//! txg 7 的根一直留在 txg 31 那一槽里，是环里最旧的有效根，而 txg 8–11 已经离开根环。
+//!
+//! - 分配器（进程内每记一条根，锚在这条根）：txg 35 起窗口是 txg > 11，txg 7 在窗口外，门槛跟着窗口里最旧的有效根往前走；
+//!   照「环里最旧有效根」取时门槛卡在 7，只被 txg 8–11 那几版引用、已释放的槽回收不了、仍算已分配，而根环里已没有根引用它们 ⇒ I-3.1 红。
+//! - 池级 checker 的候选过滤：txg 7 在镜像的窗口外（N = 最新根），不当候选走；照旧当候选走时，它引用的槽已被窗口门槛回收、可能已被复用 ⇒
+//!   I-3.1 / I-2.1 / I-7.4 红。
+//!
+//! 每次覆盖写之后在盘上跑池级 checker，判这几条不红。
+mod common;
+
+use common::{
+    build_pool, crash_state_devices, memory_pool_of_sparse_devices,
+    overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
+    system_configuration_slots_of_every_device, write_back_system_configuration_slots,
+    FIXED_WRITE_TIME_SECONDS,
+};
+use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::block_device::BlockDevice;
+use singlefs_core::mount::mount_writable;
+use singlefs_core::root_ring::{slot_offset, target_for_publish};
+use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolVersion, PoolWriter};
+use singlefs_harness::SharedStream;
+
+/// 根槽没落的那次发布的 txg：它的槽里留着上一圈 txg 7 的根（24 槽的环，31 − 24 = 7）。
+const TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST: u64 = 31;
+/// 洞那一槽里留着的上一圈的根。
+const TXG_OF_THE_STALE_ROOT_IN_THE_HOLE: u64 = 7;
+/// 一次覆盖写的内容长度：一个数据单元装得下。
+const OVERWRITE_BYTES: usize = 3000;
+/// 判的几条：记账与遍历（I-3.1、I-3.11）、被引用单元读回来对不对（I-2.1、I-7.4、I-4.8）。
+const INVARIANTS_THE_WINDOW_KEEPS_GREEN: [&str; 5] = ["I-3.1", "I-3.11", "I-2.1", "I-7.4", "I-4.8"];
+
+fn content_of_txg(checkpoint_txg: u64) -> Vec<u8> {
+    vec![u8::try_from(checkpoint_txg % 251).expect("小于 256"); OVERWRITE_BYTES]
+}
+
+#[test]
+fn reclaiming_after_a_hole_in_the_root_ring_follows_the_one_revolution_window_and_keeps_the_allocated_statistic(
+) {
+    let mut pool = build_pool("one-revolution-window-after-a-hole");
+    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
+    assert_eq!(
+        3 * slots_per_region.count(),
+        TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST - TXG_OF_THE_STALE_ROOT_IN_THE_HOLE,
+        "根环 24 槽：txg 31 与 txg 7 同槽"
+    );
+    // 迭代上界：txg 4 到 30，27 次覆盖写。
+    for checkpoint_txg in 4..TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST {
+        let output = overwrite_in_process_as_instance_one_and_advance_the_pool(
+            &mut pool,
+            &content_of_txg(checkpoint_txg),
+        );
+        assert_eq!(output.root.checkpoint_txg, CheckpointTxg(checkpoint_txg));
+    }
+    // txg 31 那次发布之前：它要写的那个根槽里是 txg 7 的根，每块盘两槽系统配置是 txg 30 那次轮换之后的样子。
+    let target = target_for_publish(
+        CheckpointTxg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
+        slots_per_region,
+    );
+    let device_of_the_slot =
+        parameters().region_devices[usize::try_from(target.region).expect("区域号")];
+    let offset_of_the_slot =
+        slot_offset(target, parameters().geometry.fixed_structure_slot_spacing);
+    let mut stale_root_slot =
+        vec![0u8; usize::try_from(parameters().geometry.physical_block_size).expect("根槽宽")];
+    {
+        let devices = pool.devices.as_ref().expect("镜像开着");
+        let (_, device) = devices
+            .iter()
+            .find(|(identity, _)| *identity == device_of_the_slot)
+            .expect("池里有这块盘");
+        device
+            .read_at(offset_of_the_slot, &mut stale_root_slot)
+            .expect("读那个根槽");
+    }
+    let system_configuration_before = system_configuration_slots_of_every_device(&pool);
+    overwrite_in_process_as_instance_one_and_advance_the_pool(
+        &mut pool,
+        &content_of_txg(TXG_WHOSE_ROOT_SLOT_DID_NOT_PERSIST),
+    );
+    // 崩溃状态：txg 31 的单元与记录都落了，根槽与之后的系统配置轮换没落（D16（发布语义） 已定项 7 的持久顺序）。
+    let mut crash_image = pool.memory_pool();
+    crash_image
+        .devices
+        .get_mut(&device_of_the_slot)
+        .expect("有这块盘")
+        .write(offset_of_the_slot, &stale_root_slot);
+    write_back_system_configuration_slots(&mut crash_image, &system_configuration_before);
+
+    let stream = SharedStream::retaining_contents();
+    let mut devices = crash_state_devices(&crash_image, &[], &[], &stream);
+    let mut mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
+    assert_eq!(
+        (
+            mounted.output.instance,
+            mounted.output.effective_root.checkpoint_txg,
+            mounted.current.root().checkpoint_txg
+        ),
+        (InstanceGeneration(2), CheckpointTxg(31), CheckpointTxg(34)),
+        "恢复由记录施加出 txg 31；写行 txg 32、暖机 txg 33、34"
+    );
+    let PoolVersion::WithFile(mut current) = mounted.current else {
+        panic!("txg 31 那一版带文件");
+    };
+    let publish_parameters = parameters();
+    // 迭代上界：覆盖写三次（txg 35、36、37）；跨轮携带的是现行那一版。
+    for checkpoint_txg in 35..=37u64 {
+        current = {
+            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+            publish_overwrite(
+                &mut writer,
+                &mut mounted.allocator,
+                &current,
+                FirstFile {
+                    content: &content_of_txg(checkpoint_txg),
+                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + checkpoint_txg,
+                },
+                InstanceGeneration(2),
+            )
+            .expect("覆盖写")
+        };
+        assert_eq!(current.root.checkpoint_txg, CheckpointTxg(checkpoint_txg));
+        let image = memory_pool_of_sparse_devices(&devices);
+        let violations: Vec<String> = check_pool_image(&image)
+            .into_iter()
+            .filter(|(invariant, _)| INVARIANTS_THE_WINDOW_KEEPS_GREEN.contains(invariant))
+            .filter_map(|(invariant, verdict)| match verdict {
+                InvariantVerdict::Violated(detail) => Some(format!("{invariant}：{detail}")),
+                InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => None,
+            })
+            .collect();
+        assert_eq!(
+            violations,
+            Vec::<String>::new(),
+            "txg {checkpoint_txg} 之后：洞那一槽里的 txg 7 在窗口外，回收门槛与候选集都跟着窗口走"
+        );
+    }
+}
diff --git a/crates/singlefs-harness/tests/rebuilt_root_keeps_the_chosen_roots_fields_except_the_unmount_marker.rs b/crates/singlefs-harness/tests/rebuilt_root_keeps_the_chosen_roots_fields_except_the_unmount_marker.rs
deleted file mode 100644
index a2eb6b4..0000000
--- a/crates/singlefs-harness/tests/rebuilt_root_keeps_the_chosen_roots_fields_except_the_unmount_marker.rs
+++ /dev/null
@@ -1,134 +0,0 @@
-//! 由 journal 记录重建的根（`recovery::replay_journal` 拼新根那一截）：新根段（D23（journal 的角色与格式） 已定项 15）那几项取被施加的末条记录，
-//! 其余字段等于所选根，卸载记号除外——它取「不是卸载那一串写下的」（D22（单元原子性怎么合成） 已定项 7「其余根写 0」）。
-//! 形式化题面第三轮判决 `research/prompts/m3-formal-r3-main-verification.md` 第二节 S1 后半、第四节欠账第 1 条。
-//!
-//! 判决里池上原型那一形：正常卸载那一串（`mount::unmount`）的第二次空发布记录落了、根槽没落（这里把那一槽改坏）——所选根是第一次空发布写的、
-//! 带卸载记号；第二次那条记录施加出来的根，卸载记号若照抄所选根，就成了「卸载那一串写下的」，可它没有一条根槽写过。
-
-mod common;
-
-use common::{
-    build_pool, overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
-    tear_the_root_slot_written_for_txg, CONTENT_BYTES_OF_PUBLISH_B,
-};
-use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
-use singlefs_core::mount::{unmount, ShadowLedger, Unmounted};
-use singlefs_core::mounted_read::mount_read_only;
-use singlefs_core::root_record::UnmountMarker;
-use singlefs_core::transaction::PoolVersion;
-
-/// 新池新建文件（txg 3）之后覆盖写三版（txg 4、5、6）的填充字节：现行 txg 6 除以 3 余 0，卸载那一串推两次空发布（txg 7、8）。
-const FILL_BYTES_OF_THE_THREE_OVERWRITES: [u8; 3] = [0xB0, 0xC0, 0xD0];
-
-/// 卸载那一串第二次空发布（txg 8）的根槽没落、记录落了：只读挂载择到第一次（txg 7）那条带卸载记号的根，施加 txg 8 那条记录。
-/// 重建出来的根：新根段那几项是 txg 8 那条记录的；文件系统标识、实例表指针、分配记录树根等于所选根；卸载记号是「不是卸载那一串写下的」。
-#[test]
-fn a_root_rebuilt_from_the_second_unmount_publish_record_keeps_the_chosen_roots_other_fields_and_is_not_marked_by_the_unmount_sequence(
-) {
-    let mut pool = build_pool("rebuilt-root-unmount-marker");
-    for fill_byte in FILL_BYTES_OF_THE_THREE_OVERWRITES {
-        overwrite_in_process_as_instance_one_and_advance_the_pool(
-            &mut pool,
-            &vec![fill_byte; CONTENT_BYTES_OF_PUBLISH_B],
-        );
-    }
-    assert_eq!(
-        pool.output.root.checkpoint_txg,
-        CheckpointTxg(6),
-        "新池新建文件 txg 3，覆盖写三版到 txg 6"
-    );
-    let mut current = PoolVersion::WithFile(pool.output.clone());
-    let devices = pool.devices.as_mut().expect("镜像还开着");
-    let unmounted = unmount(
-        &parameters(),
-        devices,
-        &mut pool.allocator,
-        &mut current,
-        ShadowLedger::On,
-    )
-    .expect("正常卸载");
-    let Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else {
-        panic!("带文件的一版上卸载报成「树表 0 条、一个字节都不写」");
-    };
-    let [first_unmount_publish, second_unmount_publish] =
-        <[_; 2]>::try_from(raised.publishes).expect("现行 txg 6 除以 3 余 0：卸载那一串推两次");
-    assert_eq!(
-        (
-            (
-                first_unmount_publish.root.checkpoint_txg,
-                first_unmount_publish.root.unmount_marker
-            ),
-            (
-                second_unmount_publish.root.checkpoint_txg,
-                second_unmount_publish.root.unmount_marker
-            ),
-            (
-                second_unmount_publish.record.instance,
-                second_unmount_publish.record.checkpoint_txg
-            )
-        ),
-        (
-            (CheckpointTxg(7), UnmountMarker::WrittenByTheUnmountSequence),
-            (CheckpointTxg(8), UnmountMarker::WrittenByTheUnmountSequence),
-            (InstanceGeneration(1), CheckpointTxg(8))
-        ),
-        "卸载那一串两次空发布 txg 7、8 写的根都带卸载记号；txg 8 那条记录在实例 1"
-    );
-
-    let mut image = pool.memory_pool();
-    tear_the_root_slot_written_for_txg(&mut image, 8);
-    let mounted = mount_read_only(&image).expect("只读挂载");
-    assert_eq!(
-        mounted.chosen_root, first_unmount_publish.root,
-        "txg 8 的根槽没落：所选根是卸载那一串第一次空发布写的那条，带卸载记号"
-    );
-    assert_eq!(
-        (mounted.journal.above_water, mounted.journal.prefix_applied),
-        (1, 1),
-        "水位之上只有 txg 8 那条记录，施加"
-    );
-
-    let chosen = mounted.chosen_root;
-    let rebuilt = mounted.effective_root;
-    let record = &second_unmount_publish.record;
-    assert_eq!(
-        (
-            rebuilt.instance,
-            rebuilt.checkpoint_txg,
-            rebuilt.tree_table,
-            rebuilt.tree_identifier_watermark,
-            rebuilt.rollback_floor,
-            rebuilt.mapping_root
-        ),
-        (
-            record.instance,
-            record.checkpoint_txg,
-            record.new_tree_table,
-            record.new_tree_identifier_watermark,
-            record.new_rollback_floor,
-            record.new_mapping_root
-        ),
-        "新根段那几项（D23 已定项 15）取 txg 8 那条记录的"
-    );
-    assert_eq!(
-        (
-            rebuilt.filesystem_identifier,
-            rebuilt.instance_table,
-            rebuilt.allocation_record_tree_root
-        ),
-        (
-            chosen.filesystem_identifier,
-            chosen.instance_table,
-            chosen.allocation_record_tree_root
-        ),
-        "新根段之外的字段等于所选根"
-    );
-    assert_eq!(
-        (chosen.unmount_marker, rebuilt.unmount_marker),
-        (
-            UnmountMarker::WrittenByTheUnmountSequence,
-            UnmountMarker::NotWrittenByTheUnmountSequence
-        ),
-        "卸载记号除外：所选根带着，由记录重建的根取「不是卸载那一串写下的」——没有一条根槽写过它"
-    );
-}
diff --git a/crates/singlefs-harness/tests/rebuilt_root_takes_every_new_root_segment_field_from_the_record.rs b/crates/singlefs-harness/tests/rebuilt_root_takes_every_new_root_segment_field_from_the_record.rs
new file mode 100644
index 0000000..7806ce0
--- /dev/null
+++ b/crates/singlefs-harness/tests/rebuilt_root_takes_every_new_root_segment_field_from_the_record.rs
@@ -0,0 +1,179 @@
+//! 由 journal 记录重建的根（`recovery::replay_journal` 拼新根那一截）：所选根的七个字段换成被施加的末条记录新根段里的——
+//! 树表指针、中央映射树根指针、树 ID 水位、F，加上实例表指针、分配记录树根指针、卸载记号（D23（journal 的角色与格式） 已定项 15，
+//! 用户 2026-10-02「回到甲」；`research/prompts/m3-ring-r3-main-verification.md` 第二节第 1、2 行）。只有 fsid 照所选根。
+//!
+//! 「记录已持久、根槽没持久」的两种发布各一条（根槽改坏：只留记录与单元），施加出来的根与那次发布当初写的根逐字段相同：
+//! - 首个文件：树表 0 条的一版写过行（分配记录树根住根记录），第一个文件版本把分配记录树挪进树表、根记录那一项写空；
+//!   照抄所选根时施加出来的根还指着那棵旧分配记录树（第二轮攻方 G1：差在 `allocation_record_tree_root`）。
+//! - 卸载：正常卸载那一串写的根带卸载记号；照抄时施加出来的根不带（第二轮攻方 G1：差在 `unmount_marker`）。
+//!
+//! 实例表指针那一项今天没有一种被施加的发布改它（改它的只有写行那次，那是新实例的第一次发布、从不施加；改它的专用清扫发布还没实现），
+//! 施加处那一判由 `replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs` 用改过新根段的记录钉。
+mod common;
+
+use common::{
+    build_pool, format_pool, overwrite_in_process_as_instance_one_and_advance_the_pool, parameters,
+    tear_the_root_slot_written_for_txg, CONTENT_BYTES_OF_PUBLISH_B, FIXED_WRITE_TIME_SECONDS,
+};
+use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::mount::{mount_writable, unmount, ShadowLedger, Unmounted};
+use singlefs_core::mounted_read::mount_read_only;
+use singlefs_core::pointer::NodePointer;
+use singlefs_core::root_record::UnmountMarker;
+use singlefs_core::transaction::{publish_first_file, FirstFile, PoolVersion, PoolWriter};
+
+/// 新池新建文件（txg 3）之后覆盖写三版（txg 4、5、6）的填充字节：现行 txg 6 除以 3 余 0，卸载那一串推两次空发布（txg 7、8）。
+const FILL_BYTES_OF_THE_THREE_OVERWRITES: [u8; 3] = [0xB0, 0xC0, 0xD0];
+
+/// 第一个文件版本的内容：一个数据单元装得下。
+const FIRST_FILE_BYTES: usize = 3000;
+
+/// 只做过 mkfs 的池：第一次可写挂载（取号 1、不写行，暖机 txg 1、2）→ 第二次可写挂载（取号 2，写行 txg 3 把分配记录树根写进根记录，
+/// 暖机 txg 4：写行与它两条根已经覆盖两块盘）→ 同一次挂载里发第一个文件版本（txg 5）。它的根槽没落、记录落了：只读挂载择到 txg 4 那条
+/// （分配记录树根不空），施加第一个文件版本那条记录；施加出来的根与那次写的根逐字段相同——分配记录树根是空的。
+#[test]
+fn a_root_rebuilt_from_a_first_file_record_over_a_version_that_wrote_a_row_takes_the_empty_allocation_record_tree_root_from_the_record(
+) {
+    let mut formatted = format_pool("rebuilt-root-first-file-after-a-row");
+    let mut first_mount_devices = formatted.reopen_recorded();
+    mount_writable(&parameters(), &mut first_mount_devices)
+        .expect("第一次可写挂载：取号 1、不写行");
+    formatted.devices = Some(first_mount_devices);
+    let mut second_mount_devices = formatted.reopen_recorded();
+    let mut mounted =
+        mount_writable(&parameters(), &mut second_mount_devices).expect("第二次可写挂载：写行");
+    formatted.devices = Some(second_mount_devices);
+    let (root_without_file, record_bytes_without_file) = match &mounted.current {
+        PoolVersion::WithoutFile(version_without_file) => (
+            version_without_file.root,
+            version_without_file.record_bytes.clone(),
+        ),
+        PoolVersion::WithFile(_) | PoolVersion::WithTreesAndNoFile(_) => {
+            panic!("树表 0 条的一版上写行与暖机，现行那一版仍没有文件")
+        }
+    };
+    assert_ne!(
+        root_without_file.allocation_record_tree_root,
+        NodePointer::empty_root(),
+        "写过行的一版：分配记录树根住根记录、不空"
+    );
+    let publish_parameters = parameters();
+    let first_file = {
+        let devices = formatted.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        publish_first_file(
+            &mut writer,
+            &mut mounted.allocator,
+            &root_without_file,
+            FirstFile {
+                content: &vec![0x5A; FIRST_FILE_BYTES],
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
+            },
+            mounted.output.instance,
+            &record_bytes_without_file,
+        )
+        .expect("第一个文件版本")
+    };
+    assert_eq!(
+        (
+            first_file.root.instance,
+            first_file.root.allocation_record_tree_root,
+            first_file.record.new_allocation_record_tree_root
+        ),
+        (
+            InstanceGeneration(2),
+            NodePointer::empty_root(),
+            NodePointer::empty_root()
+        ),
+        "第一个文件版本的根与它那条记录的新根段：分配记录树根都是空的（分配记录树挪进树表）"
+    );
+
+    let mut image = formatted.memory_pool();
+    tear_the_root_slot_written_for_txg(&mut image, first_file.root.checkpoint_txg.0);
+    let read_only = mount_read_only(&image).expect("只读挂载");
+    assert_eq!(
+        read_only.chosen_root, root_without_file,
+        "第一个文件版本的根槽没落：所选根是它施加在其上的那一版"
+    );
+    assert_eq!(
+        (
+            read_only.journal.above_water,
+            read_only.journal.prefix_applied
+        ),
+        (1, 1),
+        "水位之上只有第一个文件版本那条记录，施加"
+    );
+    assert_eq!(
+        read_only.effective_root, first_file.root,
+        "施加出来的根与第一个文件版本那次写的根逐字段相同（分配记录树根取记录里的空指针，不照抄所选根的）"
+    );
+}
+
+/// 卸载那一串第二次空发布（txg 8）的根槽没落、记录落了：只读挂载择到第一次（txg 7）那条，施加 txg 8 那条记录。
+/// 施加出来的根与 txg 8 那次写的根逐字段相同：带卸载记号（取记录新根段里的，不是「由记录施加的一律不带」）。
+#[test]
+fn a_root_rebuilt_from_the_second_unmount_publish_record_takes_the_unmount_marker_from_the_record()
+{
+    let mut pool = build_pool("rebuilt-root-unmount-marker-from-the-record");
+    for fill_byte in FILL_BYTES_OF_THE_THREE_OVERWRITES {
+        overwrite_in_process_as_instance_one_and_advance_the_pool(
+            &mut pool,
+            &vec![fill_byte; CONTENT_BYTES_OF_PUBLISH_B],
+        );
+    }
+    assert_eq!(
+        pool.output.root.checkpoint_txg,
+        CheckpointTxg(6),
+        "新池新建文件 txg 3，覆盖写三版到 txg 6"
+    );
+    let mut current = PoolVersion::WithFile(pool.output.clone());
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let unmounted = unmount(
+        &parameters(),
+        devices,
+        &mut pool.allocator,
+        &mut current,
+        ShadowLedger::On,
+    )
+    .expect("正常卸载");
+    let Unmounted::FloorRaisedToTheCurrentVersion(raised) = unmounted else {
+        panic!("带文件的一版上卸载报成「树表 0 条、一个字节都不写」");
+    };
+    let [first_unmount_publish, second_unmount_publish] =
+        <[_; 2]>::try_from(raised.publishes).expect("现行 txg 6 除以 3 余 0：卸载那一串推两次");
+    assert_eq!(
+        (
+            first_unmount_publish.root.checkpoint_txg,
+            second_unmount_publish.root.checkpoint_txg,
+            second_unmount_publish.root.unmount_marker,
+            second_unmount_publish.record.new_unmount_marker
+        ),
+        (
+            CheckpointTxg(7),
+            CheckpointTxg(8),
+            UnmountMarker::WrittenByTheUnmountSequence,
+            UnmountMarker::WrittenByTheUnmountSequence
+        ),
+        "卸载那一串两次空发布 txg 7、8：txg 8 的根带卸载记号，它那条记录的新根段也带"
+    );
+
+    let mut image = pool.memory_pool();
+    tear_the_root_slot_written_for_txg(&mut image, 8);
+    let read_only = mount_read_only(&image).expect("只读挂载");
+    assert_eq!(
+        read_only.chosen_root, first_unmount_publish.root,
+        "txg 8 的根槽没落：所选根是卸载那一串第一次空发布写的那条"
+    );
+    assert_eq!(
+        (
+            read_only.journal.above_water,
+            read_only.journal.prefix_applied
+        ),
+        (1, 1),
+        "水位之上只有 txg 8 那条记录，施加"
+    );
+    assert_eq!(
+        read_only.effective_root, second_unmount_publish.root,
+        "施加出来的根与 txg 8 那次写的根逐字段相同（卸载记号取记录里的）"
+    );
+}
diff --git a/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs b/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs
index c6e7d39..460c1ed 100644
--- a/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs
+++ b/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs
@@ -59,6 +59,9 @@ fn single_record_publish(counter: u64, checkpoint_txg: CheckpointTxg) -> Journal
         new_mapping_root: NodePointer::empty_root(),
         new_tree_identifier_watermark: 0,
         new_rollback_floor: CheckpointTxg(0),
+        new_instance_table: NodePointer::empty_root(),
+        new_allocation_record_tree_root: NodePointer::empty_root(),
+        new_unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
         named: Vec::new(),
     }
 }
diff --git a/crates/singlefs-harness/tests/replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs b/crates/singlefs-harness/tests/replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs
new file mode 100644
index 0000000..3d05233
--- /dev/null
+++ b/crates/singlefs-harness/tests/replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs
@@ -0,0 +1,160 @@
+//! 新根段里的实例表指针与分配记录树根指针（D23（journal 的角色与格式） 已定项 15，用户 2026-10-02「回到甲」）在施加一条记录时取记录里的：
+//! 实现恢复拼新根那一截（`recovery::replay_journal`）与池级 checker 重放那一版的走读（I-4.1（重放后树可遍历），
+//! `walk::walk_version_applied_only_by_records`）都从新根段读这两项，不照抄所选根。
+//!
+//! 今天被施加的发布没有一种改实例表指针（改它的写行是新实例的第一次发布、从不施加；专用清扫发布还没实现），所以这里的盘面是改出来的：
+//! 新池新建文件（txg 3）的根槽改坏、记录留着，再把那条记录新根段里的一项改成指着校验和对不上的落点（两份位置条目的整单元校验和各翻一位），
+//! 重封记录头——恢复照样施加它（点名单元两份都验得过），施加出来的根带着改过的那一项；checker 重放同一条记录，沿改过的那一项走不下去。
+mod common;
+
+use common::{
+    build_pool, rewrite_journal_record_and_rechain_the_records_after_it,
+    tear_the_root_slot_written_for_txg,
+};
+use singlefs_checker::image::{InvariantVerdict, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE};
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::address::CheckpointTxg;
+use singlefs_core::journal::JournalRecord;
+use singlefs_core::pointer::NodePointer;
+use singlefs_core::recovery::{
+    choose_root, choose_system_configuration, replay_journal, scan_journal,
+};
+use singlefs_harness::memory_pool::MemoryPool;
+
+/// 新池新建文件那条记录的 jsn 计数器（暖机 1、2，新池新建文件 3）。
+const COUNTER_OF_THE_NEW_POOL_FILE_CREATION_RECORD: u64 = 3;
+
+/// 一条指着与 `pointer` 同一落点、两份整单元校验和都对不上的指针：读得到单元，校验和那一关两份都过不去。
+fn pointer_whose_checksums_do_not_match(pointer: NodePointer) -> NodePointer {
+    let mut mismatching = pointer;
+    for location in &mut mismatching.locations {
+        location.unit_checksum ^= 1;
+    }
+    mismatching
+}
+
+/// 新池新建文件（txg 3）的根槽改坏、记录留着；再按 `change` 改那条记录（两盘各一份，重封）。交回改好的镜像。
+fn image_with_the_new_pool_file_creation_record_but_not_its_root(
+    tag: &str,
+    change: impl Fn(&mut JournalRecord),
+) -> MemoryPool {
+    let pool = build_pool(tag);
+    assert_eq!(
+        (pool.output.root.checkpoint_txg, pool.output.record.counter),
+        (
+            CheckpointTxg(3),
+            COUNTER_OF_THE_NEW_POOL_FILE_CREATION_RECORD
+        ),
+        "新池新建文件：txg 3、jsn 3"
+    );
+    let mut image = pool.memory_pool();
+    tear_the_root_slot_written_for_txg(&mut image, 3);
+    rewrite_journal_record_and_rechain_the_records_after_it(
+        &mut image,
+        COUNTER_OF_THE_NEW_POOL_FILE_CREATION_RECORD,
+        change,
+    );
+    image
+}
+
+fn verdict_of_the_replayed_version_invariant(image: &MemoryPool) -> InvariantVerdict {
+    check_pool_image(image)
+        .into_iter()
+        .find(|(invariant, _)| *invariant == TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE)
+        .map(|(_, verdict)| verdict)
+        .expect("池级 checker 每条已实现的不变量都报")
+}
+
+/// 实现恢复：所选根是暖机第二条（txg 2），施加 txg 3 那条记录；施加出来的根的实例表指针是那条记录新根段里改过的那一条，
+/// 不是所选根的（照抄所选根时两者相同、这一判红）。
+#[test]
+fn the_root_rebuilt_from_a_record_takes_the_instance_table_pointer_of_its_new_root_segment() {
+    let image = image_with_the_new_pool_file_creation_record_but_not_its_root(
+        "rebuilt-root-instance-table-from-the-record",
+        |record| {
+            record.new_instance_table = pointer_whose_checksums_do_not_match(record.new_tree_table);
+        },
+    );
+    let system_configuration = choose_system_configuration(&image).expect("系统配置择得出");
+    let chosen_root = choose_root(&image, &system_configuration).expect("择得出根");
+    assert_eq!(
+        chosen_root.checkpoint_txg,
+        CheckpointTxg(2),
+        "txg 3 的根槽坏了：所选根是暖机第二条"
+    );
+    let records = scan_journal(&image, &system_configuration);
+    let record = records
+        .values()
+        .find(|record| record.counter == COUNTER_OF_THE_NEW_POOL_FILE_CREATION_RECORD)
+        .expect("txg 3 那条记录改过之后照样自证得过");
+    let (journal, rebuilt_root) = replay_journal(
+        &image,
+        &chosen_root,
+        system_configuration.immutable.sizes.journal_ring_bytes,
+        &records,
+        true,
+    )
+    .expect("恢复施加 txg 3 那条记录");
+    assert_eq!(journal.prefix_applied, 1, "施加了那一条");
+    assert_ne!(
+        record.new_instance_table, chosen_root.instance_table,
+        "改过的那一项与所选根的实例表指针不同，两种取法分得开"
+    );
+    assert_eq!(
+        rebuilt_root.instance_table, record.new_instance_table,
+        "施加出来的根的实例表指针取记录新根段里的"
+    );
+}
+
+/// 对照：同一段历史、记录不改，checker 重放施加出来那一版（新根段七项都是那次发布写的根上的）走得完，I-4.1 成立。
+#[test]
+fn the_replayed_version_invariant_holds_over_the_untouched_record_without_its_root() {
+    let untouched = image_with_the_new_pool_file_creation_record_but_not_its_root(
+        "replayed-version-untouched",
+        |_record| {},
+    );
+    assert_eq!(
+        verdict_of_the_replayed_version_invariant(&untouched),
+        InvariantVerdict::Holds,
+        "没改的那条记录施加出来那一版走得完"
+    );
+}
+
+/// 池级 checker 重放改过的盘面（新根段里的实例表指针指着校验和对不上的落点）：施加出来那一版的实例表链从新根段读、走不下去 ⇒ I-4.1 判红。
+/// 照抄所选根的实例表（或不走实例表）时走得完、判成立。
+#[test]
+fn the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment() {
+    let image = image_with_the_new_pool_file_creation_record_but_not_its_root(
+        "replayed-version-instance-table-from-the-segment",
+        |record| {
+            record.new_instance_table = pointer_whose_checksums_do_not_match(record.new_tree_table);
+        },
+    );
+    assert!(
+        matches!(
+            verdict_of_the_replayed_version_invariant(&image),
+            InvariantVerdict::Violated(_)
+        ),
+        "新根段里的实例表指针走不下去：I-4.1 判红"
+    );
+}
+
+/// 同上，改的是新根段里的分配记录树根指针（这一版带文件，记录写的是空指针；改成指着校验和对不上的落点）：checker 从新根段读它、
+/// 走不下去 ⇒ I-4.1 判红。照抄所选根的分配记录树根（暖机那一版，全零、不走）时判成立。
+#[test]
+fn the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment() {
+    let image = image_with_the_new_pool_file_creation_record_but_not_its_root(
+        "replayed-version-allocation-record-tree-from-the-segment",
+        |record| {
+            record.new_allocation_record_tree_root =
+                pointer_whose_checksums_do_not_match(record.new_tree_table);
+        },
+    );
+    assert!(
+        matches!(
+            verdict_of_the_replayed_version_invariant(&image),
+            InvariantVerdict::Violated(_)
+        ),
+        "新根段里的分配记录树根指针走不下去：I-4.1 判红"
+    );
+}
diff --git a/crates/singlefs-harness/tests/system_configuration_slot_is_overwritten_only_after_a_barrier.rs b/crates/singlefs-harness/tests/system_configuration_slot_is_overwritten_only_after_a_barrier.rs
index 162b3cd..07251d6 100644
--- a/crates/singlefs-harness/tests/system_configuration_slot_is_overwritten_only_after_a_barrier.rs
+++ b/crates/singlefs-harness/tests/system_configuration_slot_is_overwritten_only_after_a_barrier.rs
@@ -167,7 +167,10 @@ fn rolling_back_a_failed_acquisition_overwrites_the_older_slot_only_behind_a_bar
         let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
         match acquire_instance(&mut writer).expect_err("盘 1 的取号写报错，取号必须失败") {
             InstanceAcquisitionFailed::Acquisition(acquisition) => acquisition,
-            InstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness {
+            InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
+                first_device_reporting_a_read_error,
+            } => panic!("这条用例里每块盘两槽都读得出自证过的系统配置，取号不该判不出：{first_device_reporting_a_read_error:?}"),
+            InstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
                 device,
             } => panic!("这条用例里每块盘两槽都读得出自证过的系统配置，取号不该拒在见证值那一核：{device:?}"),
         }
diff --git a/crates/singlefs-harness/tests/system_configuration_witness_is_undecidable_only_when_a_slot_reports_a_read_error_and_no_device_verified_both.rs b/crates/singlefs-harness/tests/system_configuration_witness_is_undecidable_only_when_a_slot_reports_a_read_error_and_no_device_verified_both.rs
new file mode 100644
index 0000000..56226d0
--- /dev/null
+++ b/crates/singlefs-harness/tests/system_configuration_witness_is_undecidable_only_when_a_slot_reports_a_read_error_and_no_device_verified_both.rs
@@ -0,0 +1,210 @@
+//! 系统配置见证读判不出取 R2（C331（择根倒挂压过已确认的写），D22（单元原子性怎么合成） 已定项 8 射程，用户 2026-10-03 定「改取 R2」，零轮）：
+//! 判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过（`recovery::system_configuration_witness_is_undecidable`）。
+//! 可写挂载读阶段判 N-配置（`mount::newer_publish_witness`）与取号那一刻读见证值（`transaction::self_verified_system_configurations_of_every_device`）
+//! 同一判；判不出按判据为真：可写挂载重读一次仍判不出就在取号之前拒、盘上逐字节不变，只读挂载照常；取号在第一个写之前拒。
+//!
+//! 盘面（新池新建文件之后每块盘两槽：世代 4、世代 5，世代 5 那一槽是新池新建文件那次轮换写的、槽号 5 mod 2 = 1），与 R2 网格的格对上：
+//! - 纯形（全池读回一个值：两块盘世代 5 那一槽都报读错）：有槽报读错、没有一块盘两槽都自证过 ⇒ 判不出、拒可写；取号也拒。
+//! - 只有一块盘世代 5 那一槽报读错、另一块盘两槽都自证过 ⇒ 判得出，照常可写挂载。
+//! - 混合形（一盘报读错、另一盘读回全零）：判不出、拒可写（取甲-读错-每盘时这一形丢写，R2 拒，网格上混合形丢写 0）。
+//! - 撕裂轮换（两块盘世代 5 那一槽都读回全零：读得出、自证不过，设备没报错）：判得出、照常可写挂载（取甲时拒、之后只剩只读挂载）。
+mod common;
+
+use common::{
+    build_pool, parameters, with_unreadable_ranges, without_unreadable_ranges, BuiltPool,
+    FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
+};
+use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
+use singlefs_core::block_device::{BlockDevice, WriteDurability};
+use singlefs_core::mount::{
+    mount_writable, MountError, StillUnreadableAfterOneReread, WitnessedCounterComparison,
+};
+use singlefs_core::mounted_read::mount_read_only;
+use singlefs_core::transaction::{acquire_instance, InstanceAcquisitionFailed, PoolWriter};
+use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
+
+/// 新池新建文件那次轮换写的世代号：每块盘世代 5 那一槽（槽号 5 mod 2 = 1）。
+const GENERATION_OF_THE_NEW_POOL_FILE_CREATION_ROTATION: u64 = 5;
+
+fn offset_of_the_newest_system_configuration_slot() -> u64 {
+    (GENERATION_OF_THE_NEW_POOL_FILE_CREATION_ROTATION % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
+        * u64::from(parameters().geometry.fixed_structure_slot_spacing)
+}
+
+/// `device` 世代 5 那一槽，每次读都报读错。
+fn newest_system_configuration_slot_reporting_a_read_error_on(
+    device: DeviceIdentity,
+) -> UnreadableRange {
+    UnreadableRange {
+        device,
+        offset_in_bytes: offset_of_the_newest_system_configuration_slot(),
+        length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
+        failing_reads: FailingReadsOfARange::Every,
+    }
+}
+
+/// 把 `device` 世代 5 那一槽在盘上清零（撕裂的轮换：读得出、自证不过）。
+fn zero_the_newest_system_configuration_slot_of(pool: &mut BuiltPool, device: DeviceIdentity) {
+    let devices = pool.devices.as_mut().expect("新池新建文件写完，盘还开着");
+    let (_, zeroed_device) = devices
+        .iter_mut()
+        .find(|(identity, _)| *identity == device)
+        .expect("池里有这块盘");
+    zeroed_device
+        .write_at(
+            DeviceOffsetInBytes(offset_of_the_newest_system_configuration_slot()),
+            &vec![0u8; usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096")],
+            WriteDurability::Plain,
+        )
+        .expect("世代 5 那一槽清零");
+}
+
+/// 重开、包上 `ranges` 的读故障可写挂载一次，交回结局；盘拆掉读故障交回池。
+fn writable_mount_with_unreadable_ranges(
+    pool: &mut BuiltPool,
+    ranges: Vec<UnreadableRange>,
+) -> Result<singlefs_core::mount::Mounted, MountError> {
+    let shared = SharedUnreadableRanges::new(ranges, UnreadableRangeReadBack::DeviceError);
+    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &shared);
+    let outcome = mount_writable(&parameters(), &mut devices);
+    pool.devices = Some(without_unreadable_ranges(devices));
+    outcome
+}
+
+/// 拒因：判据 N-配置 为真、重读仍为真，两遍读数都判 R2 的判不出。
+fn refused_as_undecidable_on_both_reads(
+    outcome: &Result<singlefs_core::mount::Mounted, MountError>,
+) -> bool {
+    let Err(MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)) = outcome else {
+        return false;
+    };
+    let StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
+        first_read,
+        reread,
+    } = still_unreadable.as_ref()
+    else {
+        return false;
+    };
+    [first_read, reread].iter().all(|reading| {
+        reading.witness.comparison
+            == WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots
+    })
+}
+
+/// 纯形：两块盘世代 5 那一槽都报读错 ⇒ 判不出，重读一次仍判不出，拒可写、盘上逐字节不变；只读挂载照常。
+#[test]
+fn a_writable_mount_is_refused_when_the_newest_slot_of_every_device_reports_a_read_error() {
+    let mut pool = build_pool("system-configuration-witness-pure-form-refused");
+    let image_before = pool.memory_pool();
+    let outcome = writable_mount_with_unreadable_ranges(
+        &mut pool,
+        vec![
+            newest_system_configuration_slot_reporting_a_read_error_on(DeviceIdentity(0)),
+            newest_system_configuration_slot_reporting_a_read_error_on(DeviceIdentity(1)),
+        ],
+    );
+    assert!(
+        refused_as_undecidable_on_both_reads(&outcome),
+        "有槽报读错、没有一块盘两槽都自证过：判不出，拒可写；得到 {:?}",
+        outcome.as_ref().err()
+    );
+    assert!(
+        pool.memory_pool() == image_before,
+        "拒在取号之前：盘上逐字节不变"
+    );
+    mount_read_only(&pool.memory_pool()).expect("只读挂载照常");
+}
+
+/// 只有盘 0 世代 5 那一槽报读错，盘 1 两槽都自证过 ⇒ 判得出（见证值取盘 1 的），照常可写挂载、取到实例 2。
+#[test]
+fn a_writable_mount_goes_ahead_when_one_device_reports_a_read_error_and_the_other_verified_both_slots(
+) {
+    let mut pool = build_pool("system-configuration-witness-one-device-read-error");
+    let outcome = writable_mount_with_unreadable_ranges(
+        &mut pool,
+        vec![newest_system_configuration_slot_reporting_a_read_error_on(
+            DeviceIdentity(0),
+        )],
+    );
+    assert_eq!(
+        outcome.as_ref().map(|mounted| mounted.output.instance).ok(),
+        Some(InstanceGeneration(2)),
+        "盘 1 两槽都自证过：判得出，照常可写挂载；得到 {:?}",
+        outcome.as_ref().err()
+    );
+}
+
+/// 混合形：盘 0 世代 5 那一槽报读错、盘 1 世代 5 那一槽读回全零 ⇒ 有槽报读错、没有一块盘两槽都自证过：判不出，拒可写、盘上逐字节不变。
+#[test]
+fn a_writable_mount_is_refused_when_one_device_reports_a_read_error_and_the_other_reads_back_zeros()
+{
+    let mut pool = build_pool("system-configuration-witness-mixed-form-refused");
+    zero_the_newest_system_configuration_slot_of(&mut pool, DeviceIdentity(1));
+    let image_before = pool.memory_pool();
+    let outcome = writable_mount_with_unreadable_ranges(
+        &mut pool,
+        vec![newest_system_configuration_slot_reporting_a_read_error_on(
+            DeviceIdentity(0),
+        )],
+    );
+    assert!(
+        refused_as_undecidable_on_both_reads(&outcome),
+        "一盘报读错、另一盘读回全零：判不出，拒可写；得到 {:?}",
+        outcome.as_ref().err()
+    );
+    assert!(
+        pool.memory_pool() == image_before,
+        "拒在取号之前：盘上逐字节不变"
+    );
+}
+
+/// 撕裂的系统配置轮换：两块盘世代 5 那一槽都读回全零（设备没报错）⇒ 判得出（见证值取两块盘世代 4 那一槽的），照常可写挂载、取到实例 2。
+#[test]
+fn a_writable_mount_goes_ahead_over_a_torn_system_configuration_rotation_on_every_device() {
+    let mut pool = build_pool("system-configuration-witness-torn-rotation");
+    zero_the_newest_system_configuration_slot_of(&mut pool, DeviceIdentity(0));
+    zero_the_newest_system_configuration_slot_of(&mut pool, DeviceIdentity(1));
+    let outcome = writable_mount_with_unreadable_ranges(&mut pool, Vec::new());
+    assert_eq!(
+        outcome.as_ref().map(|mounted| mounted.output.instance).ok(),
+        Some(InstanceGeneration(2)),
+        "撕裂的轮换不算判不出，照常可写挂载；得到 {:?}",
+        outcome.as_ref().err()
+    );
+}
+
+/// 取号那一刻同一判：两块盘世代 5 那一槽都报读错，不经可写挂载直接取号 ⇒ 判不出，在第一个取号写之前拒，报盘表次序第一块报读错的盘（盘 0）；
+/// 盘上逐字节不变。
+#[test]
+fn an_acquisition_is_refused_before_any_write_when_the_newest_slot_of_every_device_reports_a_read_error(
+) {
+    let mut pool = build_pool("system-configuration-witness-acquisition-refused");
+    let image_before = pool.memory_pool();
+    let shared = SharedUnreadableRanges::new(
+        vec![
+            newest_system_configuration_slot_reporting_a_read_error_on(DeviceIdentity(0)),
+            newest_system_configuration_slot_reporting_a_read_error_on(DeviceIdentity(1)),
+        ],
+        UnreadableRangeReadBack::DeviceError,
+    );
+    let mut devices = with_unreadable_ranges(pool.reopen_recorded(), &shared);
+    let pool_parameters = parameters();
+    let acquisition = {
+        let mut writer = PoolWriter::new(&pool_parameters, devices.as_mut_slice());
+        acquire_instance(&mut writer)
+    };
+    pool.devices = Some(without_unreadable_ranges(devices));
+    assert!(
+        matches!(
+            acquisition,
+            Err(InstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
+                first_device_reporting_a_read_error: DeviceIdentity(0),
+            })
+        ),
+        "判不出：取号在任何写之前拒，报盘 0；得到 {acquisition:?}"
+    );
+    assert!(
+        pool.memory_pool() == image_before,
+        "拒在第一个取号写之前：盘上逐字节不变"
+    );
+}
````

## 挂载时读不出槽的循环加上界（`/tmp/claude-1000/m3-r1/batch7-window-loop/patch/crates.patch`，报告同目录 `report.md`）

````diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -1609,7 +1609,8 @@
 /// 读不出的槽里可能住着比环里读得出的最新根还新的根（被藏住的最新根、由记录施加出来的那一版那次发布的根），上沿取新实例第一次发布之前的那个 txg。
 /// 只管设备报读错的槽；读得出而自证不过（没写过、内容坏）的槽照旧当没有根。
 ///
-/// 循环：外层按区间里的 txg（轮数 = first_txg − 窗口下沿，至多 3S 加上环里只有记录、没有根的那几个 txg），跨轮只往按槽的表里写最小、最大值；没有提前出口。
+/// 循环：只走区间头、区间尾两段，各 min(3S, first_txg − 窗口下沿) 个 txg（轮数至多 6S，与 first_txg 多大无关：外来记录的 txg 可以贴着
+/// `u64::MAX`），跨轮只往按槽的表里写最小、最大值；没有提前出口。
 fn unreadable_ring_slots_inside_the_window(
     ring_slot_readings: &[(RootRingSlot, RootRingSlotReading)],
     window_anchor: CheckpointTxg,
@@ -1629,9 +1630,17 @@
         .0
         .checked_sub(ring_length_in_txgs)
         .map_or(0, |below_the_window| below_the_window + 1);
+    // target_for_publish 只看 txg mod 3S（区域 = txg mod R，槽 = (txg div R) mod S），所以一个槽在区间里的最小 txg 落在区间头
+    // min(3S, 区间长) 个 txg 里、最大 txg 落在区间尾同样多个 txg 里，两段之间的 txg 改不动任何一个槽的最小、最大值。
+    // 两段在区间短于 6S 时有重叠：先走区间头、再走区间尾，各自从小到大，第一次碰到的是最小值、最后一次碰到的是最大值。
+    let interval_length_in_txgs = first_txg.0.saturating_sub(lowest_txg_inside_the_window);
+    let txgs_walked_at_each_end_of_the_interval = interval_length_in_txgs.min(ring_length_in_txgs);
+    let head_of_the_interval = lowest_txg_inside_the_window
+        ..lowest_txg_inside_the_window + txgs_walked_at_each_end_of_the_interval;
+    let tail_of_the_interval = first_txg.0 - txgs_walked_at_each_end_of_the_interval..first_txg.0;
     let mut txgs_of_each_unreadable_slot: BTreeMap<RootRingSlot, (CheckpointTxg, CheckpointTxg)> =
         BTreeMap::new();
-    for txg in lowest_txg_inside_the_window..first_txg.0 {
+    for txg in head_of_the_interval.chain(tail_of_the_interval) {
         let ring_slot = target_for_publish(CheckpointTxg(txg), slots_per_region);
         if !unreadable_ring_slots.contains(&ring_slot) {
             continue;
@@ -6180,3 +6189,187 @@
         assert_eq!(reclaim_floor(CheckpointTxg(11), None), CheckpointTxg(11));
     }
 }
+
+#[cfg(test)]
+mod unreadable_ring_slots_inside_the_window_tests {
+    use super::{unreadable_ring_slots_inside_the_window, UnreadableRingSlotInsideTheWindow};
+    use crate::address::CheckpointTxg;
+    use crate::recovery::{BadRootRingSlotReading, RootRingSlotReading};
+    use crate::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
+    use singlefs_format::ROOT_RING_REGIONS;
+    use std::collections::{BTreeMap, BTreeSet};
+
+    /// 环里每个槽都读过一次：两个槽读得出而自证不过，其余的设备报读错。
+    fn ring_slot_readings_with_two_slots_not_self_verified(
+        slots_per_region: RootRingSlotsPerRegion,
+    ) -> Vec<(RootRingSlot, RootRingSlotReading)> {
+        let not_self_verified_ring_slots = [
+            RootRingSlot { region: 0, slot: 0 },
+            RootRingSlot { region: 1, slot: 2 },
+        ];
+        (0..ROOT_RING_REGIONS)
+            .flat_map(|region| {
+                (0..slots_per_region.count()).map(move |slot| RootRingSlot { region, slot })
+            })
+            .map(|ring_slot| {
+                let bad_reading = if not_self_verified_ring_slots.contains(&ring_slot) {
+                    BadRootRingSlotReading::NotSelfVerified
+                } else {
+                    BadRootRingSlotReading::Unreadable
+                };
+                (ring_slot, RootRingSlotReading::Bad(bad_reading))
+            })
+            .collect()
+    }
+
+    /// 参照：区间 (N − 3S, first_txg − 1] 里的 txg 一个一个走（改成只走两段之前的写法），只给区间长度装得下的组用。
+    fn walk_every_txg_of_the_interval(
+        ring_slot_readings: &[(RootRingSlot, RootRingSlotReading)],
+        window_anchor: CheckpointTxg,
+        first_txg: CheckpointTxg,
+        slots_per_region: RootRingSlotsPerRegion,
+    ) -> Vec<UnreadableRingSlotInsideTheWindow> {
+        let unreadable_ring_slots: BTreeSet<RootRingSlot> = ring_slot_readings
+            .iter()
+            .filter(|(_, reading)| {
+                *reading == RootRingSlotReading::Bad(BadRootRingSlotReading::Unreadable)
+            })
+            .map(|(ring_slot, _)| *ring_slot)
+            .collect();
+        let ring_length_in_txgs = ROOT_RING_REGIONS * slots_per_region.count();
+        let lowest_txg_inside_the_window = window_anchor
+            .0
+            .checked_sub(ring_length_in_txgs)
+            .map_or(0, |below_the_window| below_the_window + 1);
+        let mut txgs_of_each_unreadable_slot: BTreeMap<
+            RootRingSlot,
+            (CheckpointTxg, CheckpointTxg),
+        > = BTreeMap::new();
+        for txg in lowest_txg_inside_the_window..first_txg.0 {
+            let ring_slot = target_for_publish(CheckpointTxg(txg), slots_per_region);
+            if unreadable_ring_slots.contains(&ring_slot) {
+                txgs_of_each_unreadable_slot
+                    .entry(ring_slot)
+                    .and_modify(|(_, highest)| *highest = CheckpointTxg(txg))
+                    .or_insert((CheckpointTxg(txg), CheckpointTxg(txg)));
+            }
+        }
+        txgs_of_each_unreadable_slot
+            .into_iter()
+            .map(
+                |(ring_slot, (lowest_txg_it_can_hold, highest_txg_it_can_hold))| {
+                    UnreadableRingSlotInsideTheWindow {
+                        ring_slot,
+                        lowest_txg_it_can_hold,
+                        highest_txg_it_can_hold,
+                    }
+                },
+            )
+            .collect()
+    }
+
+    /// 区间短于 3S、正好 3S、两段重叠、长于 3S 若干圈与空区间几组逐项比参照；first_txg 是 `u64::MAX` 那一组（外来记录的 txg 是
+    /// `u64::MAX − 1`）参照走不完，比的是按 3S 周期手算的期望。
+    #[test]
+    fn walking_only_the_two_ends_of_the_interval_matches_walking_every_txg_of_it() {
+        for declared_slots_per_region in [4, 5] {
+            let slots_per_region =
+                RootRingSlotsPerRegion::from_system_configuration_field(declared_slots_per_region)
+                    .expect("4 与 5 落在 S 的区间 [4, 16] 里");
+            let ring_length_in_txgs = ROOT_RING_REGIONS * declared_slots_per_region;
+            let ring_slot_readings =
+                ring_slot_readings_with_two_slots_not_self_verified(slots_per_region);
+            let anchor_above_one_revolution = 40;
+            let lowest_txg_inside_the_window =
+                anchor_above_one_revolution - ring_length_in_txgs + 1;
+            let anchor_and_first_txg_of_each_group = [
+                (anchor_above_one_revolution, lowest_txg_inside_the_window),
+                (
+                    anchor_above_one_revolution,
+                    lowest_txg_inside_the_window + ring_length_in_txgs - 5,
+                ),
+                (
+                    anchor_above_one_revolution,
+                    lowest_txg_inside_the_window + ring_length_in_txgs,
+                ),
+                (
+                    anchor_above_one_revolution,
+                    lowest_txg_inside_the_window + 2 * ring_length_in_txgs - 1,
+                ),
+                (
+                    anchor_above_one_revolution,
+                    lowest_txg_inside_the_window + 4 * ring_length_in_txgs + 7,
+                ),
+                (5, 9),
+                (5, 5 * ring_length_in_txgs + 3),
+            ];
+            for (window_anchor, first_txg) in anchor_and_first_txg_of_each_group {
+                assert_eq!(
+                    unreadable_ring_slots_inside_the_window(
+                        &ring_slot_readings,
+                        CheckpointTxg(window_anchor),
+                        CheckpointTxg(first_txg),
+                        slots_per_region,
+                    ),
+                    walk_every_txg_of_the_interval(
+                        &ring_slot_readings,
+                        CheckpointTxg(window_anchor),
+                        CheckpointTxg(first_txg),
+                        slots_per_region,
+                    ),
+                    "S = {declared_slots_per_region}，窗口锚 = {window_anchor}，first_txg = {first_txg}"
+                );
+            }
+        }
+
+        // S = 4，3S = 12：txg 模 12 的余数 r 落在区域 r mod 3 的槽 r div 3（D22 已定项 2 的槽位公式在一个周期里展开）。
+        // 2^64 模 12 是 4，所以 u64::MAX 模 12 是 3：区间尾 u64::MAX − 12 ..= u64::MAX − 1 里余数是 r 的那个 txg 是 u64::MAX − k，
+        // k 取 1..=12 里 (3 − k) 模 12 等于 r 的那一个。窗口锚 40 的窗口下沿是 29（模 12 是 5）。
+        let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(4)
+            .expect("4 是 S 区间 [4, 16] 的下界");
+        let ring_length_in_txgs = 12;
+        assert_eq!(u64::MAX % ring_length_in_txgs, 3);
+        let ring_slot_readings =
+            ring_slot_readings_with_two_slots_not_self_verified(slots_per_region);
+        let lowest_txg_inside_the_window = 29;
+        let mut expected: Vec<UnreadableRingSlotInsideTheWindow> = (0..ring_length_in_txgs)
+            .map(|residue| {
+                let distance_modulo_the_ring =
+                    (3 + ring_length_in_txgs - residue) % ring_length_in_txgs;
+                let distance_below_u64_maximum = if distance_modulo_the_ring == 0 {
+                    ring_length_in_txgs
+                } else {
+                    distance_modulo_the_ring
+                };
+                UnreadableRingSlotInsideTheWindow {
+                    ring_slot: RootRingSlot {
+                        region: residue % 3,
+                        slot: residue / 3,
+                    },
+                    lowest_txg_it_can_hold: CheckpointTxg(
+                        lowest_txg_inside_the_window
+                            + (residue + ring_length_in_txgs - 5) % ring_length_in_txgs,
+                    ),
+                    highest_txg_it_can_hold: CheckpointTxg(u64::MAX - distance_below_u64_maximum),
+                }
+            })
+            .filter(|slot_inside_the_window| {
+                ![
+                    RootRingSlot { region: 0, slot: 0 },
+                    RootRingSlot { region: 1, slot: 2 },
+                ]
+                .contains(&slot_inside_the_window.ring_slot)
+            })
+            .collect();
+        expected.sort_by_key(|slot_inside_the_window| slot_inside_the_window.ring_slot);
+        assert_eq!(
+            unreadable_ring_slots_inside_the_window(
+                &ring_slot_readings,
+                CheckpointTxg(40),
+                CheckpointTxg(u64::MAX),
+                slots_per_region,
+            ),
+            expected
+        );
+    }
+}
````

## 7-2 合入后改钉与前提（`/tmp/claude-1000/m3-r1/batch7-repin/patch/crates.patch`，报告同目录 `report.md`）

````diff
diff --git a/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs b/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs
index cf7075a..2d1b9f4 100644
--- a/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs
+++ b/crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs
@@ -290,9 +290,10 @@ fn crc32_of_the_images(devices: &[(DeviceIdentity, SparseBlockDevice)], device_b
 
 // ─── 一、单元区起点随环长走 ───
 
-/// 默认环 768 MiB 上 mkfs → 取号 → 暖机 → 第一个文件之后，两块盘整盘镜像的 CRC-32C。钉的是改之前那一份代码写出的字节
-/// （实审 A3b 开工时主工作区的副本上同一段流程现算，`name=a3b_default_ring_new_pool_file_creation_images_crc32 value=1783297687`）：
-/// 默认环下单元区起点仍是 50176，新池新建文件写出的字节逐字节不变。
+/// 默认环 768 MiB 上 mkfs → 取号 → 暖机 → 第一个文件之后，两块盘整盘镜像的 CRC-32C。钉值是下面最后一次换钉那一版代码写出的字节，
+/// 不是实审 A3b 改单元区起点之前那一版的：最初钉的是那一版（实审 A3b 开工时主工作区的副本上同一段流程现算，
+/// `name=a3b_default_ring_new_pool_file_creation_images_crc32 value=1783297687`），判的是 A3b 那一步默认环下单元区起点仍是 50176、
+/// 新池新建文件写出的字节逐字节不变；之后每一次有意改了这两块盘上字节的改动换一次钉值，逐次记在下面。
 /// 三·2 第 1 条（D18（块里携带什么信息） 已定项 24 第 6 条）改了新池新建文件的字节：树表 7 → 11 条、key 区间末端 18 → 22（旧号）、
 /// 根记录与 journal 新根段的树 ID 水位 19 → 23（旧号），连带树表单元的校验和、指着它的指针、根槽与记录的校验和；钉值随之从 1783297687
 /// 换成 2804593800（三·2 实现员副本上同一段流程现算，单元区起点那几格的断言不动）。
@@ -301,10 +302,14 @@ fn crc32_of_the_images(devices: &[(DeviceIdentity, SparseBlockDevice)], device_b
 /// 三·2 第 6 条把记账标签 6 从 0 换成按这一版现算的墓碑预付（D3（空间分配） 已定项 2；`.claude/kb/layout/03-third-txn.md`
 /// 「记账标签 6 的值」那一行：新池新建文件里 0 → 2 × (3 × 16384 + 41) = 98386，连带记账树单元的载荷校验和、指着它的指针与映射条目、
 /// 根槽与记录的校验和）；钉值随之换成 1733593715（m3-unlink-body 第六位实现员的副本上同一段流程现算）。
-const DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32: u32 = 1_733_593_715;
+/// 里程碑三开工第一轮 7-2（D23（journal 的角色与格式） 已定项 4、已定项 15：journal 记录头 311 → 487、新根段 188 → 364）改了三条 journal 记录
+/// （暖机两条、第一个文件一条）的字节：新根段补上实例表指针、分配记录树根指针、卸载记号，fsid 挪到 [463, 471)，点名项整体后挪 176 字节，
+/// 连带 header_csum 与反向链；系统配置槽、根环、单元区逐字节不变（7-2 之前与之后两份副本上同一段流程逐扇区比过，差异只落在 journal 环
+/// 第 0–2 个记录槽里）。钉值随之换成 4048568891（7-2 合入之后的副本上同一段流程现算，与合入验证里这条断言红出来的 left 值相同）。
+const DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32: u32 = 4_048_568_891;
 
 /// 默认环 768 MiB：mkfs 按环长现算的单元区起点就是 50176（`INSTANCE_TABLE_SLOT`），树表第 0 版在 50178（`TREE_TABLE_GENESIS_SLOT`），
-/// 系统配置偏移 417 的 8 字节写 50176；mkfs 到第一个文件写出的两块盘与改之前逐字节相同（整盘 CRC-32C 钉值）。
+/// 系统配置偏移 417 的 8 字节写 50176；mkfs 到第一个文件写出的两块盘与钉值那一版（钉值说明里最后一次换钉的那一版）逐字节相同（整盘 CRC-32C 钉值）。
 #[test]
 fn the_default_journal_ring_keeps_the_unit_area_at_slot_50176_and_the_new_pool_file_creation_bytes_unchanged(
 ) {
@@ -336,7 +341,7 @@ fn the_default_journal_ring_keeps_the_unit_area_at_slot_50176_and_the_new_pool_f
     assert_eq!(
         crc32_of_the_images(&pool.devices, pool.device_bytes),
         DEFAULT_RING_NEW_POOL_FILE_CREATION_IMAGES_CRC32,
-        "默认环下 mkfs 到第一个文件写出的两块盘与改之前逐字节相同"
+        "默认环下 mkfs 到第一个文件写出的两块盘与钉值那一版（7-2 合入之后）逐字节相同"
     );
 }
 
diff --git a/crates/singlefs-harness/tests/writable_mount_refusal_judged_against_the_d23_list.rs b/crates/singlefs-harness/tests/writable_mount_refusal_judged_against_the_d23_list.rs
index 199c330..b7635ba 100644
--- a/crates/singlefs-harness/tests/writable_mount_refusal_judged_against_the_d23_list.rs
+++ b/crates/singlefs-harness/tests/writable_mount_refusal_judged_against_the_d23_list.rs
@@ -1,16 +1,23 @@
 //! 可写挂载被拒时判拒绝本身（`singlefs_harness::mount_transparency::judge_writable_mount_refusal`，r5 判决第二节 U6）：拒绝合法 ⇔
 //! 拒的理由在 D23（journal 的角色与格式） 已定项 14 那张单子上、那一条判据在崩溃镜像上复算为真、盘上逐字节不变、只读读出在允许集合里。
 //! 判据由判法在镜像上自己复算，不经 O1 判官、不经实现判拒那一段。小池是单元区 384 槽的两块内存盘，第一个文件之后的盘面。
+//! 要拒的盘面取系统配置见证读的 R2（D22（单元原子性怎么合成） 已定项 8 射程）判不出那一形：两块盘第一个文件那次轮换写的那一槽都报读错，
+//! 有槽设备报读错、没有一块盘两槽都自证过；崩溃镜像是同一份盘面包上同一组读错（判法读到的与可写挂载读到的一样）。
+//! 撕裂的轮换（读得出、自证不过、设备没报错）R2 下照常可写挂载，不是要拒的盘面。
+mod common;
 
-use singlefs_core::address::{
-    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
+use common::{
+    with_unreadable_ranges, without_unreadable_ranges, DeviceWithUnreadableRanges,
+    FailingReadsOfARange, SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
 };
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
 use singlefs_core::admission::SpaceAdmission;
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
 use singlefs_core::mount::{
     mount_writable_with_space_admission, MountError, NewerPublishWitness, RollbackTarget,
     SelectedVersionAgainstTheWitness, StillUnreadableAfterOneReread, WitnessedCounterComparison,
 };
+use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
 use singlefs_harness::history::{
     execute_history_with, GeneratedHistory, HistoryDeviceWidth, HistoryExecution, HistorySeed,
     HistoryStartingPoint, PerStepChecker,
@@ -24,6 +31,8 @@ use singlefs_harness::mount_transparency::{
 use singlefs_harness::SharedStream;
 
 const SMALL: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;
+/// 第一个文件那次系统配置轮换写的世代号（mkfs → 取号 → 暖机 → 第一个文件，同 `build_pool` 那一段）：每块盘世代 5 那一槽（槽号 5 mod 2 = 1）。
+const GENERATION_OF_THE_FIRST_FILE_ROTATION: u64 = 5;
 
 fn pool_after_the_first_file() -> MemoryPool {
     let stream = SharedStream::retaining_contents();
@@ -50,21 +59,54 @@ fn pool_after_the_first_file() -> MemoryPool {
     pool
 }
 
-/// 在 `crash_image` 的副本上起可写挂载：拒了交回错误与拒之后的盘面，做成了 panic。
-fn refused_mount_on(crash_image: &MemoryPool) -> (MountError, MemoryPool) {
-    let parameters = SMALL.parameters();
-    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = crash_image
+/// 两块盘第一个文件那次轮换写的那一槽（世代 5）每次读都报读错：R2 判不出那一形（有槽设备报读错、没有一块盘两槽都自证过）。
+fn read_errors_on_the_newest_system_configuration_slot_of_every_device() -> SharedUnreadableRanges {
+    let offset_of_the_newest_slot = (GENERATION_OF_THE_FIRST_FILE_ROTATION
+        % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
+        * u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
+    SharedUnreadableRanges::new(
+        [DeviceIdentity(0), DeviceIdentity(1)]
+            .into_iter()
+            .map(|device| UnreadableRange {
+                device,
+                offset_in_bytes: offset_of_the_newest_slot,
+                length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
+                failing_reads: FailingReadsOfARange::Every,
+            })
+            .collect(),
+        UnreadableRangeReadBack::DeviceError,
+    )
+}
+
+/// `pool` 的盘面拷进内存稀疏盘、逐块包上 `read_errors`：既是可写挂载挂的盘，也是判法复算判据读的崩溃镜像。
+fn devices_with_read_errors(
+    pool: &MemoryPool,
+    read_errors: &SharedUnreadableRanges,
+) -> Vec<(
+    DeviceIdentity,
+    DeviceWithUnreadableRanges<SparseBlockDevice>,
+)> {
+    let physical_block_size =
+        PhysicalBlockSizeInBytes(SMALL.parameters().geometry.physical_block_size);
+    let devices = pool
         .devices
         .iter()
         .map(|(identity, sectors)| {
-            let mut device = SparseBlockDevice::new(
-                crash_image.device_size_in_bytes,
-                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
-            );
+            let mut device = SparseBlockDevice::new(pool.device_size_in_bytes, physical_block_size);
             device.image = sectors.clone();
             (*identity, device)
         })
         .collect();
+    with_unreadable_ranges(devices, read_errors)
+}
+
+/// 在 `pool` 盘面的副本上包上 `read_errors` 起可写挂载：拒了交回错误与拒之后的盘面（读错拆掉），做成了 panic。
+fn refused_mount_on(
+    pool: &MemoryPool,
+    read_errors: &SharedUnreadableRanges,
+) -> (MountError, MemoryPool) {
+    let parameters = SMALL.parameters();
+    let mut devices = devices_with_read_errors(pool, read_errors);
     let error = match mount_writable_with_space_admission(
         &parameters,
         &mut devices,
@@ -77,15 +119,33 @@ fn refused_mount_on(crash_image: &MemoryPool) -> (MountError, MemoryPool) {
         ),
     };
     let after = MemoryPool {
-        devices: devices
+        devices: without_unreadable_ranges(devices)
             .into_iter()
             .map(|(identity, device)| (identity, device.image))
             .collect(),
-        device_size_in_bytes: crash_image.device_size_in_bytes,
+        device_size_in_bytes: pool.device_size_in_bytes,
     };
     (error, after)
 }
 
+/// 拒因是 R2 的判不出：读阶段判据 N-配置 为真、重读仍为真，两遍读数的比较都是「有槽设备报读错、没有一块盘两槽都自证过」。
+fn refused_as_undecidable_under_r2_on_both_reads(error: &MountError) -> bool {
+    let MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable) = error else {
+        return false;
+    };
+    let StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion {
+        first_read,
+        reread,
+    } = still_unreadable.as_ref()
+    else {
+        return false;
+    };
+    [first_read, reread].iter().all(|reading| {
+        reading.witness.comparison
+            == WitnessedCounterComparison::SomeSlotReportedAReadErrorAndNoDeviceSelfVerifiedBothSlots
+    })
+}
+
 fn disk_after(crash_image: &MemoryPool, after: &MemoryPool) -> DiskAfterTheRefusal {
     if crash_image == after {
         DiskAfterTheRefusal::ByteForByteUnchanged
@@ -114,31 +174,28 @@ fn witness_refusal_claimed_on(selected_version: (InstanceGeneration, CheckpointT
     ))
 }
 
+/// 两块盘世代 5 那一槽都报读错：R2 判不出，可写挂载在取号之前拒、盘上逐字节不变，判法在包着同一组读错的崩溃镜像上复算判据为真 ⇒ 合法。
 #[test]
-fn a_refusal_after_a_torn_system_configuration_slot_is_legal() {
+fn a_refusal_when_the_newest_system_configuration_slot_of_every_device_reports_a_read_error_is_legal(
+) {
     let healthy = pool_after_the_first_file();
     let content_of_the_healthy_pool =
         read_recovery_fields(&healthy, ComparedVersion::TheRecoveredVersion);
-    let mut torn = healthy.clone();
-    let slot_spacing = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
-    torn.devices
-        .get_mut(&DeviceIdentity(0))
-        .expect("有盘 0")
-        .write(DeviceOffsetInBytes(slot_spacing), &[0xa5; 4096]);
-    let (error, after) = refused_mount_on(&torn);
+    let (error, after) = refused_mount_on(
+        &healthy,
+        &read_errors_on_the_newest_system_configuration_slot_of_every_device(),
+    );
     assert!(
-        matches!(
-            &error,
-            MountError::NewerStateStillUnreadableAfterOneReread(still_unreadable)
-                if matches!(
-                    **still_unreadable,
-                    StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion { .. }
-                )
-        ),
-        "前提：撕了一个系统配置槽，读阶段判据 N-配置 按真、重读仍真，拒：{error:?}"
+        refused_as_undecidable_under_r2_on_both_reads(&error),
+        "前提：两块盘世代 5 那一槽都报读错，R2 判不出，读阶段判据 N-配置 按真、重读仍真，拒：{error:?}"
     );
-    let fields_of_the_torn_pool = read_recovery_fields(&torn, ComparedVersion::TheRecoveredVersion);
-    let read_only_read_back = if fields_of_the_torn_pool
+    let crash_image = devices_with_read_errors(
+        &healthy,
+        &read_errors_on_the_newest_system_configuration_slot_of_every_device(),
+    );
+    let fields_of_the_crash_image =
+        read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
+    let read_only_read_back = if fields_of_the_crash_image
         .compared_version
         .as_ref()
         .map(|version| &version.content)
@@ -153,10 +210,10 @@ fn a_refusal_after_a_torn_system_configuration_slot_is_legal() {
     };
     assert_eq!(
         judge_writable_mount_refusal(
-            &torn,
-            fields_of_the_torn_pool.report.effective_root,
+            &crash_image,
+            fields_of_the_crash_image.report.effective_root,
             &error,
-            disk_after(&torn, &after),
+            disk_after(&healthy, &after),
             read_only_read_back,
         ),
         WritableMountRefusalVerdict::Legal(
@@ -216,20 +273,27 @@ fn an_unreadable_instance_table_refusal_on_a_healthy_pool_is_a_refusal_for_the_w
     );
 }
 
+/// 同一份 R2 判不出的盘面（两块盘世代 5 那一槽都报读错）上的拒，交进判法时报成拒之后盘变了 ⇒ 判非法，只差「盘被改了」那一条。
 #[test]
-fn a_torn_slot_refusal_that_changed_the_disk_is_illegal() {
+fn a_refusal_on_read_errors_of_every_newest_system_configuration_slot_that_changed_the_disk_is_illegal(
+) {
     let healthy = pool_after_the_first_file();
-    let mut torn = healthy.clone();
-    let slot_spacing = u64::from(SMALL.parameters().geometry.fixed_structure_slot_spacing);
-    torn.devices
-        .get_mut(&DeviceIdentity(1))
-        .expect("有盘 1")
-        .write(DeviceOffsetInBytes(slot_spacing), &[0x5a; 4096]);
-    let (error, _after) = refused_mount_on(&torn);
-    let fields = read_recovery_fields(&torn, ComparedVersion::TheRecoveredVersion);
+    let (error, _after) = refused_mount_on(
+        &healthy,
+        &read_errors_on_the_newest_system_configuration_slot_of_every_device(),
+    );
+    assert!(
+        refused_as_undecidable_under_r2_on_both_reads(&error),
+        "前提：两块盘世代 5 那一槽都报读错，R2 判不出，拒：{error:?}"
+    );
+    let crash_image = devices_with_read_errors(
+        &healthy,
+        &read_errors_on_the_newest_system_configuration_slot_of_every_device(),
+    );
+    let fields = read_recovery_fields(&crash_image, ComparedVersion::TheRecoveredVersion);
     assert_eq!(
         judge_writable_mount_refusal(
-            &torn,
+            &crash_image,
             fields.report.effective_root,
             &error,
             DiskAfterTheRefusal::Changed,
````

