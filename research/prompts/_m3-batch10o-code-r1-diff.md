# 根环实现第 10o、10p 批合入：代码轮第一轮改动全文（2026-10-06）

合前原样取自主工作区合入那一刻（`~/.local/share/singlefs/work/m3-r1/merge-10o-10p/before/`），合后取自冻结副本（`~/.local/share/singlefs/work/m3-batch10o-code-r1/tree/`）。含主 agent 合入时删掉的那一份重名常量。

## `crates/singlefs-checker/src/image.rs`

```diff
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -91,7 +91,11 @@
 /// I-9.17、I-9.23 在主走读里判（墓碑树、谱系树的内部条目），其余在删除态扫描里每一版各判（I-9.29 跨版本比）；
 /// I-9.21、I-9.22 的旋钮值从镜像外给（`walk::check_pool_image_given_the_tombstone_container_capacity_knob`）。
 /// GPU 判器的内核这一批不置这九位（`crash_judge_gpu` 的覆盖表照登记，内核第 23 批补）。
-pub const IMPLEMENTED_INVARIANTS: [&str; 83] = [
+///
+/// 第 84 条（下标 83）是 I-3.14（实例表每片记录形态合法，用户 2026-10-06 定另立，根环线第 10j 批），同样只往末尾追加；
+/// 主走读沿实例表链每一片判（`walk` 的 `judge_instance_table_page_record_forms` 与链指针记录那一格）。
+/// 不成立的那一片解不开：行不收、链不往下走（第 10k 批）。GPU 判器的走读内核同判（`crash_judge_gpu` 的覆盖表补了下标 83，第 10k 批）。
+pub const IMPLEMENTED_INVARIANTS: [&str; 84] = [
     "I-1.1",
     "I-1.2",
     "I-1.3",
@@ -175,6 +179,7 @@
     TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
     DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
     ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
+    INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
 ];
 
 /// I-1.12：走读跟随的一条指针，偏移 34 的出生树等于被指单元头里的出生树——码 1、码 3 取头偏移 43；码 2 取哪个字段随 C289 定，
@@ -273,6 +278,11 @@
 /// I-9.29（同号只许一个出生代）：回退候选集里各版的 inode 记录与各版自己谱系上的墓碑合起来，同一棵 inode 树里同一个号只许一个对象出生代
 /// （判决第二节 T10 乙、第四节 T15「写」：D8（核心索引结构） 已定项 6「同一条时间线上 inode 号不复用」）。
 pub const ONE_OBJECT_BIRTH_PER_INODE_NUMBER: &str = "I-9.29";
+/// I-3.14（实例表每片记录形态合法，用户 2026-10-06 定另立）：走读到的实例表每一片里，每条记录的 `kind` 只有 0（行）或 1（链指针记录）；
+/// 链指针记录恰好是这一片的最后一条（合法：有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零），片中间不出现；
+/// `kind` 0 行的 flags 字节为 0（D18（块里携带什么信息） 已定项 11：flags「第一版恒 0、非 0 拒收」）。行唯一与低于挂载根归 I-3.8。
+/// 不成立的一片解不开（用户 2026-10-06 给 D18 已定项 11 补「kind 不是 0 或 1 ⇒ 这一片解不开（表不可读）」）：主走读与只读不判的读法都不收它的行。
+pub const INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED: &str = "I-3.14";
 
 /// 判定累加器：每条不变量记评估了几次、第一处违例、整条不适用的理由，以及射程里有一部分判不了的理由。
 #[derive(Default)]
@@ -1495,6 +1505,22 @@
     unit_bytes: usize,
     what: &str,
 ) -> Option<Vec<u8>> {
+    read_referenced_unit_with_the_location_it_was_read_from(
+        reader, judgements, locations, unit_bytes, what,
+    )
+    .map(|(bytes, _location_it_was_read_from)| bytes)
+}
+
+/// 同 [`read_referenced_unit`]，另交回读回的那一份（第一份对得上的）是哪一条位置条目：认「走过」要按读回、自证过的那一份的落点
+/// （实例表链上第 k 片的指针，盘 0 那条位置条目指错、盘 1 那条完好时，按盘 0 那条认会把它认成别的片，攻方 H-b，
+/// `research/prompts/m3-batch25-code-r1-opus-output.md` 第 1.3 节）。
+pub fn read_referenced_unit_with_the_location_it_was_read_from(
+    reader: &dyn ImageReader,
+    judgements: &mut Judgements,
+    locations: &[PointerLocation; 2],
+    unit_bytes: usize,
+    what: &str,
+) -> Option<(Vec<u8>, PointerLocation)> {
     let mut first_good = None;
     for location in locations {
         let bytes = reader.read(location.device, location.slot * SLOT_BYTES, unit_bytes);
@@ -1508,7 +1534,7 @@
             )
         });
         if matches && first_good.is_none() {
-            first_good = bytes;
+            first_good = bytes.map(|content| (content, *location));
         }
     }
     first_good
```

## `crates/singlefs-checker/src/walk.rs`

```diff
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -743,6 +743,131 @@
     }
 }
 
+/// 实例表一条记录的种类：首字节 `kind`（D18（块里携带什么信息） 已定项 11 只登记两种：0 行、1 链指针记录）。
+/// 别的值没登记（用户 2026-10-06 定：`kind` 不是 0 或 1 ⇒ 这一片解不开），留着原值好写进违例说明。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum InstanceTableRecordKind {
+    Row,
+    ChainRecord,
+    Unregistered(u8),
+}
+
+impl InstanceTableRecordKind {
+    fn of_record(record: &[u8]) -> Self {
+        match record
+            .first()
+            .copied()
+            .expect("实例表记录宽 88：judge_packed_container 判过记录宽等于登记的宽")
+        {
+            0 => Self::Row,
+            1 => Self::ChainRecord,
+            unregistered_kind => Self::Unregistered(unregistered_kind),
+        }
+    }
+}
+
+/// 实例表行记录（`kind` 0）里 flags 字节的偏移：kind 1 + 实例代号 4 + 所选根的 checkpoint_txg 8 + W 8（D18（块里携带什么信息） 已定项 11）。
+const INSTANCE_TABLE_ROW_FLAGS_OFFSET: usize = 21;
+/// 行记录 flags 第一版唯一合法的值（D18（块里携带什么信息） 已定项 11：「第一版恒 0、非 0 拒收」，bit0 退役也在内）。
+const INSTANCE_TABLE_ROW_FLAGS_OF_THE_FIRST_VERSION: u8 = 0;
+
+/// I-3.8（实例表行唯一且低于挂载根）在最新根的实例表第 0 片就解不开（I-3.14 不成立）时报的不适用理由：一行都没收，没有行可判
+/// （不落到 `Judgements::into_report` 的缺省理由「这个镜像上没有它判的对象」）。
+const INSTANCE_TABLE_ROWS_NOT_JUDGED_AS_THE_FIRST_PAGE_OF_THE_NEWEST_ROOTS_TABLE_DOES_NOT_DECODE:
+    &str = "最新根的实例表那一片解不开（I-3.14 不成立）、行没收全：没有行可判";
+
+/// 一片实例表的记录形态（I-3.14 那几格）合不合法。不合法的那一片解不开：用户 2026-10-06 给 D18（块里携带什么信息） 已定项 11 补
+/// 「kind 不是 0 或 1 ⇒ 这一片解不开（表不可读）」，实现的 `InstanceTablePage::parse` 对「最后一条不是合法的链指针记录」「它前面有不是行的记录」
+/// 「行的 flags 不是 0」都交 None；主走读照「片解不开」走（[`Walk::walk_instance_table_chain`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum InstanceTablePageRecordForms {
+    WellFormed,
+    IllFormedSoThePageDoesNotDecode,
+}
+
+/// I-3.14（实例表每片记录形态合法）在一片上判三件，各一格：每条记录的 `kind` 只有 0 或 1；`kind` 1（链指针记录）不出现在最后一条之前；
+/// `kind` 0 行的 flags 字节为 0。最后一条是不是合法的链指针记录是同一条不变量的第四格，在 `walk_instance_table_chain` 里判。
+/// 交回三件是不是都成立：有一件不成立，这一片解不开（[`InstanceTablePageRecordForms`]）。
+fn judge_instance_table_page_record_forms(
+    judgements: &mut Judgements,
+    records: &[Vec<u8>],
+    what: &str,
+) -> InstanceTablePageRecordForms {
+    use crate::image::INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED;
+    let kinds: Vec<InstanceTableRecordKind> = records
+        .iter()
+        .map(|record| InstanceTableRecordKind::of_record(record))
+        .collect();
+    let unregistered_kinds: Vec<(usize, u8)> = kinds
+        .iter()
+        .enumerate()
+        .filter_map(|(position, kind)| match kind {
+            InstanceTableRecordKind::Unregistered(raw_kind) => Some((position, *raw_kind)),
+            InstanceTableRecordKind::Row | InstanceTableRecordKind::ChainRecord => None,
+        })
+        .collect();
+    judgements.judge(
+        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
+        unregistered_kinds.is_empty(),
+        || {
+            format!(
+                "{what}：（第几条, kind）{unregistered_kinds:?} 的 kind 不是 0（行）或 1（链指针记录）：这一片解不开"
+            )
+        },
+    );
+    let last_position = records.len().saturating_sub(1);
+    let chain_records_before_the_last: Vec<usize> = kinds
+        .iter()
+        .enumerate()
+        .filter_map(|(position, kind)| match kind {
+            InstanceTableRecordKind::ChainRecord if position < last_position => Some(position),
+            InstanceTableRecordKind::ChainRecord
+            | InstanceTableRecordKind::Row
+            | InstanceTableRecordKind::Unregistered(_) => None,
+        })
+        .collect();
+    judgements.judge(
+        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
+        chain_records_before_the_last.is_empty(),
+        || {
+            format!(
+                "{what}：第 {chain_records_before_the_last:?} 条是链指针记录，不在这一片（{} 条）的末尾",
+                records.len()
+            )
+        },
+    );
+    let rows_with_flags_off_the_first_version: Vec<(usize, u8)> = records
+        .iter()
+        .zip(&kinds)
+        .enumerate()
+        .filter_map(|(position, (record, kind))| match kind {
+            InstanceTableRecordKind::Row => {
+                let flags = record[INSTANCE_TABLE_ROW_FLAGS_OFFSET];
+                (flags != INSTANCE_TABLE_ROW_FLAGS_OF_THE_FIRST_VERSION)
+                    .then_some((position, flags))
+            }
+            InstanceTableRecordKind::ChainRecord | InstanceTableRecordKind::Unregistered(_) => None,
+        })
+        .collect();
+    judgements.judge(
+        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
+        rows_with_flags_off_the_first_version.is_empty(),
+        || {
+            format!(
+                "{what}：（第几条, flags）{rows_with_flags_off_the_first_version:?} 的行 flags 不是 0"
+            )
+        },
+    );
+    if unregistered_kinds.is_empty()
+        && chain_records_before_the_last.is_empty()
+        && rows_with_flags_off_the_first_version.is_empty()
+    {
+        InstanceTablePageRecordForms::WellFormed
+    } else {
+        InstanceTablePageRecordForms::IllFormedSoThePageDoesNotDecode
+    }
+}
+
 /// 指针尾段装的那个量：码 1 的指针 88 尾段是写序的事务号（6 字节），码 2 / 码 3 的指针 86 尾段是出生序号（4 字节）
 /// （`image::parse_data_pointer` / `image::parse_node_pointer` 各解一份；`KEY_SCHEMA_MAPPING` 那一行的注同一个口径）。
 /// 码 3 的写序里还有一个事务号，指针不带它 ⇒ 不比它。
@@ -2934,22 +3059,27 @@
 
     /// 沿链走一张实例表（D18（块里携带什么信息） 已定项 11：根记录直接持有第 0 片，第 k 片末尾的链指针记录指着第 k + 1 片）。
     /// 每一片判 I-2.5（位置条目升序）、I-2.1（校验和）、码 3 容器那几条（`judge_packed_container`）、I-1.1（身份四元组 =
-    /// (0, 4, 片序号, 0)：链指针指到了别的单元、链成了环，都在这一格红）与 I-3.8 的链指针记录那一半（最后一条是合法的链指针记录）；
+    /// (0, 4, 片序号, 0)：链指针指到了别的单元、链成了环，都在这一格红）与 I-3.14（实例表每片记录形态合法：最后一条是合法的链指针记录，
+    /// 加 `judge_instance_table_page_record_forms` 那三件）；
     /// 挂载根那一张（`mount_root_instance_when_newest` 给了实例代号）把各片的记录交给 `judge_instance_table_rows` 判行那一半，
     /// 走到末片（链指针记录说「没有下一片」）时记下候选集滤得全（`abandoned_roots_filtering`）。
-    /// 迭代上界：第 k 片要求容器号是 k，走过的单元不再走（`visited_units`），每一轮是盘上一个不同的单元。
+    /// 迭代上界：第 k 片要求容器号是 k，走过的单元不再走（`visited_units`，按读回的那一份的落点认），每一轮是盘上一个不同的单元。
     /// 提前出口，链都不再往下走：读不出、身份不对（被引用的单元坏了或换了人，记进走读失败；头用不了由 `judge_unit_header` 记）；
-    /// 容器记录不合法、链指针记录不合法（这一片自己写错了，只红 I-1.7 / I-3.8 那一格）；走到已经走过的单元（前一条根走过这条链）。
+    /// 容器记录不合法、I-3.14 不成立（这一片自己写错了，只红 I-1.7 / I-3.14 那一格；这一片解不开，行不收）；走到已经走过的单元（前一条根走过这条链）。
+    /// I-3.14 不成立的四形（kind 不是 0 或 1、片中间有链指针记录、行的 flags 不是 0、最后一条不是合法的链指针记录）一律当这一片解不开，
+    /// 与实现 `InstanceTablePage::parse` 同一个读法（用户 2026-10-06 给 D18（块里携带什么信息） 已定项 11 补的那一句）。
     fn walk_instance_table_chain(
         &mut self,
         first_page_pointer: &PointerView,
         mount_root_instance_when_newest: Option<u32>,
     ) {
+        use crate::image::INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED;
         let mut pointer = *first_page_pointer;
         let mut page_index: u64 = 0;
         let mut pages: Vec<Vec<Vec<u8>>> = Vec::new();
         // 第 0 片由根记录里的指针指着（外面那一层的走读父单元照旧）；第 k + 1 片由第 k 片末尾的链指针记录指着，记在第 k 片下面。
         let parent_unit_when_the_chain_starts = self.walk_parent_unit;
+        let mut walk_stopped_at_a_page_that_does_not_decode = false;
         loop {
             let (what, pointer_what) = if page_index == 0 {
                 ("实例表单元".to_string(), "实例表指针".to_string())
@@ -2968,19 +3098,28 @@
             );
             judge_pointer_extent_offset_is_zero(&mut self.judgements, &pointer, &pointer_what);
             self.note_references_of_a_pointer_followed_by_the_walk(&pointer, 2, &what);
-            let Some(unit) = read_referenced_unit(
-                self.reader,
-                &mut self.judgements,
-                &pointer.locations,
-                data_unit_bytes(),
-                &what,
-            ) else {
+            let Some((unit, location_it_was_read_from)) =
+                crate::image::read_referenced_unit_with_the_location_it_was_read_from(
+                    self.reader,
+                    &mut self.judgements,
+                    &pointer.locations,
+                    data_unit_bytes(),
+                    &what,
+                )
+            else {
                 self.walk_failures
                     .push(format!("{what}两份都读不到对得上的"));
                 break;
             };
             let page_placement = (pointer.locations[0].device, pointer.locations[0].slot);
-            if !self.visited_units.insert(page_placement) {
+            // 「走过」按读回、自证过的那一份的落点认（攻方 H-b，`research/prompts/m3-batch25-code-r1-opus-output.md` 第 1.3 节）：
+            // 第 k 片指针的盘 0 位置条目指回走过的片、盘 1 那条完好时，按第 0 条位置条目认会把读到手的这一片当成走过、丢掉它的行。
+            // 别处的键（父单元、诞生代号那张表、并进前一版的落点表）照旧按第 0 条位置条目，与走读别的单元同一个读法。
+            let placement_it_was_read_from = (
+                location_it_was_read_from.device,
+                location_it_was_read_from.slot,
+            );
+            if !self.visited_units.insert(placement_it_was_read_from) {
                 self.merge_placements_below_a_unit_walked_by_an_earlier_version(page_placement);
                 break;
             }
@@ -3014,18 +3153,33 @@
                     .push(format!("{what}：指到的单元不是链上第 {page_index} 片"));
                 break;
             }
+            let record_forms =
+                judge_instance_table_page_record_forms(&mut self.judgements, &records, &what);
             let chain = records
                 .last()
                 .map_or(ChainRecordView::Malformed, |last| chain_record_view(last));
-            self.judgements
-                .judge("I-3.8", chain != ChainRecordView::Malformed, || {
+            self.judgements.judge(
+                INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
+                chain != ChainRecordView::Malformed,
+                || {
                     format!(
                         "{what}：最后一条不是合法的链指针记录（kind 1、有无下一片 0 或 1、无下一片时位置指针全零、有下一片时不全零）"
                     )
-                });
-            pages.push(records);
+                },
+            );
+            // I-3.14 不成立的一片解不开（D18（块里携带什么信息） 已定项 11 用户 2026-10-06 补句；实现 `InstanceTablePage::parse` 同样交 None），
+            // 照上面「容器记录不合法」那一处出口走：这一片的行不收、链不往下走、没走到末片（候选集滤不全，I-9.29 的门关）。
+            // 不记走读失败——走读失败驱动 I-7.4 / I-4.8「引用的单元已被复用或抹头」，这里没有哪个被引用的单元坏了，是这一片自己的记录写错了。
+            match record_forms {
+                InstanceTablePageRecordForms::WellFormed => {}
+                InstanceTablePageRecordForms::IllFormedSoThePageDoesNotDecode => {
+                    walk_stopped_at_a_page_that_does_not_decode = true;
+                    break;
+                }
+            }
             match chain {
                 ChainRecordView::LastPage => {
+                    pages.push(records);
                     if mount_root_instance_when_newest.is_some() {
                         self.abandoned_roots_filtering =
                             AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable;
@@ -3033,35 +3187,42 @@
                     break;
                 }
                 ChainRecordView::NextPage(next_page) => {
+                    pages.push(records);
                     pointer = next_page;
                     page_index += 1;
                 }
-                // 链指针记录自相矛盾：上面 I-3.8 那一格已经红了，链不往下走。不记走读失败——走读失败驱动 I-7.4 / I-4.8
-                // 「引用的单元已被复用或抹头」，这里没有哪个被引用的单元坏了，是这一片自己的记录写错了。
-                ChainRecordView::Malformed => break,
+                // 链指针记录自相矛盾：上面 I-3.14 那一格已经红了，这一片同样解不开（行不收），链不往下走。
+                ChainRecordView::Malformed => {
+                    walk_stopped_at_a_page_that_does_not_decode = true;
+                    break;
+                }
             }
         }
         self.walk_parent_unit = parent_unit_when_the_chain_starts;
         if let Some(mount_root_instance) = mount_root_instance_when_newest {
             if !pages.is_empty() {
                 self.judge_instance_table_rows(&pages, mount_root_instance);
+            } else if walk_stopped_at_a_page_that_does_not_decode {
+                self.judgements.not_applicable(
+                    "I-3.8",
+                    INSTANCE_TABLE_ROWS_NOT_JUDGED_AS_THE_FIRST_PAGE_OF_THE_NEWEST_ROOTS_TABLE_DOES_NOT_DECODE,
+                );
             }
         }
     }
 
     /// 码 3 容器：I-1.7（记录数 × 记录宽 ≤ 声明长度 ≤ 32768 − 136、记录宽 = 登记的宽）。返回记录，用不了返回 None。
-    /// I-3.8（实例表行唯一且低于挂载根）：整张表（链上各片的行接起来）里 `kind` 0 行按实例代号唯一、每行的实例代号 < 挂载根（最新根）的实例；
-    /// 链指针记录（`kind` 1）恒为每一片的最后一条（D18（块里携带什么信息） 已定项 11）。「行只在回收条件成立后删」对着一个镜像判不了。
+    /// I-3.8（实例表行唯一且低于挂载根）：整张表（链上各片的行接起来）里 `kind` 0 行按实例代号唯一、每行的实例代号 < 挂载根（最新根）的实例。
+    /// 记录形态（`kind` 只有 0 或 1、链指针记录恰好在每一片末尾、行的 flags 为 0）归 I-3.14，在 `walk_instance_table_chain` 里每一片判；
+    /// 这里只收 `kind` 0 的行，别的记录跳过。「行只在回收条件成立后删」对着一个镜像判不了。
     fn judge_instance_table_rows(&mut self, pages: &[Vec<Vec<u8>>], mount_root_instance: u32) {
         let mut instances: Vec<u32> = Vec::new();
         let mut unique = true;
         let mut below_mount_root = true;
-        let mut chain_record_last = true;
         for records in pages {
-            let mut chain_record_last_in_this_page = false;
-            for (index, row) in records.iter().enumerate() {
-                match row.first().copied() {
-                    Some(0) => {
+            for row in records {
+                match InstanceTableRecordKind::of_record(row) {
+                    InstanceTableRecordKind::Row => {
                         let instance = u32::from_le_bytes(row[1..5].try_into().expect("4 字节"));
                         if instances.contains(&instance) {
                             unique = false;
@@ -3071,20 +3232,18 @@
                         if instance >= mount_root_instance {
                             below_mount_root = false;
                         }
-                        chain_record_last_in_this_page = false;
                     }
-                    Some(1) => chain_record_last_in_this_page = index + 1 == records.len(),
-                    _ => chain_record_last_in_this_page = false,
+                    InstanceTableRecordKind::ChainRecord
+                    | InstanceTableRecordKind::Unregistered(_) => {}
                 }
             }
-            chain_record_last &= chain_record_last_in_this_page;
         }
         self.judgements.judge(
             "I-3.8",
-            unique && below_mount_root && chain_record_last,
+            unique && below_mount_root,
             || {
                 format!(
-                    "实例表（{} 片）：行的实例代号 {instances:?}（唯一 = {unique}，都低于挂载根实例 {mount_root_instance} = {below_mount_root}），每一片的链指针记录在末尾 = {chain_record_last}",
+                    "实例表（{} 片）：行的实例代号 {instances:?}（唯一 = {unique}，都低于挂载根实例 {mount_root_instance} = {below_mount_root}）",
                     pages.len()
                 )
             },
@@ -4444,8 +4603,8 @@
 }
 
 /// 沿实例表链（根记录直接持有第 0 片，第 k 片末尾的链指针记录指着第 k + 1 片，D18（块里携带什么信息） 已定项 11）把第 1 片起每一片的
-/// 指针记进这条根的引用集合。只读不判：一片读不出、解不开、身份不是 (0, 4, 片序号, 0)、最后一条不是合法的链指针记录，
-/// 都只如实记成数不全（判定归主走读：I-2.1、I-1.1、I-3.8）。
+/// 指针记进这条根的引用集合。只读不判：一片读不出、解不开、身份不是 (0, 4, 片序号, 0)、I-3.14 不成立（链指针记录之前有不是行的记录、
+/// 行的 flags 不是 0、最后一条不是合法的链指针记录），都只如实记成数不全（判定归主走读：I-2.1、I-1.1、I-3.14）。
 /// 迭代上界：第 k 片要求容器号是 k，每一轮读的是盘上一个不同的单元；跨轮带的只有下一片的指针；提前出口是走到最后一片与上面那几样读不下去。
 fn note_instance_table_pages_after_the_first(
     reader: &dyn ImageReader,
@@ -4471,6 +4630,21 @@
             references.is_complete = false;
             return;
         }
+        // I-3.14 不成立的一片解不开（D18（块里携带什么信息） 已定项 11，用户 2026-10-06 补句）：链指针记录之前有不是行的记录
+        // （kind 不是 0 或 1、片中间有链指针记录）或行的 flags 不是 0，与主走读、只读不判的读法同一个读法，数不全。
+        let records_before_the_last = page
+            .records
+            .split_last()
+            .map_or(&[][..], |(_, before)| before);
+        if records_before_the_last.iter().any(|record| {
+            record.len() != INSTANCE_TABLE_RECORD_BYTES
+                || InstanceTableRecordKind::of_record(record) != InstanceTableRecordKind::Row
+                || record[INSTANCE_TABLE_ROW_FLAGS_OFFSET]
+                    != INSTANCE_TABLE_ROW_FLAGS_OF_THE_FIRST_VERSION
+        }) {
+            references.is_complete = false;
+            return;
+        }
         match page
             .records
             .last()
@@ -5908,8 +6082,8 @@
 
 /// 一条根自己指着的那张实例表的全部行，沿链读（D18（块里携带什么信息） 已定项 11：根记录直接持有第 0 片，第 k 片末尾的链指针记录指着第 k + 1 片）。
 /// 只读不判：哪一片读不出（两份的整单元 CRC 都对不上位置条目）、解不开（头判不过、类不是码 3、记录类型不是 4、记录宽不是 88、
-/// 记录数 × 记录宽超过声明长度或声明长度超过容量、身份不是 (0, 4, 片序号, 0)、链指针记录之前有不是行的记录、最后一条不是合法的链指针记录）
-/// ⇒ None，判定归主走读那一遍（I-2.1、I-1.7、I-1.1、I-3.8）。
+/// 记录数 × 记录宽超过声明长度或声明长度超过容量、身份不是 (0, 4, 片序号, 0)、链指针记录之前有不是行的记录、行的 flags 不是 0、
+/// 最后一条不是合法的链指针记录）⇒ None，判定归主走读那一遍（I-2.1、I-1.7、I-1.1、I-3.14）。I-3.14 的四形与主走读、与实现同一个读法：解不开。
 /// 迭代上界：第 k 轮要求读到的单元身份里容器号是 k，两轮读到同一个单元就要它的容器号同时等于两个数 ⇒ 每一轮读的是盘上不同的单元，
 /// 轮数不超过盘上装得下的 32 KiB 单元数。跨轮带的是下一片的指针、片序号与已经接起来的行；提前出口只有读不出、解不开。
 fn instance_table_rows_of_root_without_judging(
@@ -5945,7 +6119,10 @@
             .collect();
         let (chain_record, rows_of_this_page) = records.split_last()?;
         for row in rows_of_this_page {
-            if row[0] != 0 {
+            if row[0] != 0
+                || row[INSTANCE_TABLE_ROW_FLAGS_OFFSET]
+                    != INSTANCE_TABLE_ROW_FLAGS_OF_THE_FIRST_VERSION
+            {
                 return None;
             }
             rows.push(parse_instance_table_row(row));
@@ -9575,7 +9752,7 @@
 /// 那张表读不全时滤不全，被抛弃时间线上的根可能进了候选集。I-9.29（同号只许一个出生代） 跨版本比，这时记判不了
 /// （`.claude/kb/invariants.md` I-9.29 那一行「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」）：被抛弃的线上同号可以是另一个对象。
 /// 读没读全看的是候选集滤法用的那一遍（主走读，`Walk::walk_instance_table_chain`），不另读一遍：另一个读法（只读不判的
-/// [`instance_table_rows_of_root_without_judging`]）拒收的比主走读多（链指针记录之前有不是行的记录、载荷 CRC 对不上），
+/// [`instance_table_rows_of_root_without_judging`]）拒收的比主走读多（载荷 CRC 对不上；I-3.14 的四形两边一样当解不开，第 10k 批），
 /// 用它当门会在候选集已经滤全时报判不了（判决 `research/prompts/m3-batch23-code-r1-main-verification.md` 第二节 K2 N5）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum AbandonedRootsFilteringOfTheCandidateSet {
```

## `crates/singlefs-checker-tier/src/crash_injection.rs`

```diff
--- a/crates/singlefs-checker-tier/src/crash_injection.rs
+++ b/crates/singlefs-checker-tier/src/crash_injection.rs
@@ -8115,4 +8115,136 @@
             tally.render()
         );
     }
+
+    /// 清扫发布那一版的根身份（实例, checkpoint_txg），两条用例共用：R2 那一版就是它。
+    const SWEEP_PUBLISH_OF_THE_C704_CASES: (InstanceGeneration, CheckpointTxg) =
+        (InstanceGeneration(3), CheckpointTxg(9));
+    /// 清扫删掉的那一行与留下的那一行。
+    const ROW_DELETED_BY_THE_SWEEP: InstanceRow = InstanceRow {
+        instance: InstanceGeneration(1),
+        selected_root_txg: CheckpointTxg(4),
+        applied_transaction_high_water: 2,
+    };
+    const ROW_KEPT_BY_THE_SWEEP: InstanceRow = InstanceRow {
+        instance: InstanceGeneration(2),
+        selected_root_txg: CheckpointTxg(7),
+        applied_transaction_high_water: 5,
+    };
+
+    /// 一次只读恢复的字段，拿来比的那一版（R2）的根身份是 `root_identity`、实例表行是 `instance_table_rows`。恢复报告与其余字段取自一对空盘上的
+    /// 只读恢复（择不到系统配置，那几样都是 None）：C704 那一臂只看 R2 的根身份与实例表行。
+    fn recovery_fields_read_comparing_a_version_with_these_instance_table_rows(
+        root_identity: (InstanceGeneration, CheckpointTxg),
+        instance_table_rows: Vec<InstanceRow>,
+    ) -> RecoveryFieldsRead {
+        let blank_pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], 4 << 30);
+        let mut read = read_recovery_fields(&blank_pool, ComparedVersion::TheRecoveredVersion);
+        let empty_pointer = singlefs_core::pointer::NodePointer::empty_root();
+        read.compared_version = Some(singlefs_harness::mount_transparency::VersionFieldsRead {
+            root: singlefs_core::root_record::RootRecord {
+                filesystem_identifier: [7u8; 16],
+                unmount_marker:
+                    singlefs_core::root_record::UnmountMarker::NotWrittenByTheUnmountSequence,
+                instance: root_identity.0,
+                checkpoint_txg: root_identity.1,
+                tree_table: empty_pointer,
+                tree_identifier_watermark: 0,
+                rollback_floor: CheckpointTxg(0),
+                instance_table: empty_pointer,
+                mapping_root: empty_pointer,
+                allocation_record_tree_root: empty_pointer,
+            },
+            root_record_bytes: Vec::new(),
+            content: singlefs_harness::mount_transparency::PosixContentRead::NoFile,
+            instance_table_rows: Some(instance_table_rows),
+            tree_table_entries: None,
+        });
+        read
+    }
+
+    fn rows_deleted_by_the_sweep_of_the_c704_cases() -> RowsDeletedByTheInstanceTableSweepPublish {
+        RowsDeletedByTheInstanceTableSweepPublish {
+            sweep_publish: SWEEP_PUBLISH_OF_THE_C704_CASES,
+            rows_deleted: vec![ROW_DELETED_BY_THE_SWEEP],
+            heads_erased: 0,
+        }
+    }
+
+    /// C704（挂载透明不认清扫发布）处置那一臂：R2 那一版是清扫发布那一版，它的实例表里却仍有清扫说删掉了的行 ⇒ 交回一条违例，
+    /// 名字是 [`MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH`]，按字段分的判红计数里它加 1；
+    /// R2 照原样交回、不记放回。判别力（`crates/mutations.tsv` 里点名这条的行）：被删的行还在也当成都不在、放回去再比时一条违例都没有；
+    /// 判红不进计数时那一格是 None。
+    #[test]
+    fn a_sweep_publish_still_holding_the_rows_it_deleted_is_a_mount_transparency_violation_counted_once(
+    ) {
+        let read = recovery_fields_read_comparing_a_version_with_these_instance_table_rows(
+            SWEEP_PUBLISH_OF_THE_C704_CASES,
+            vec![ROW_KEPT_BY_THE_SWEEP, ROW_DELETED_BY_THE_SWEEP],
+        );
+        let rows_deleted = rows_deleted_by_the_sweep_of_the_c704_cases();
+        let mut tally = CrashInjectionTally::default();
+        let (compared, violations) = after_the_action_with_the_rows_deleted_by_the_sweep_put_back(
+            &mut tally,
+            &read,
+            Some(&rows_deleted),
+        );
+        assert_eq!(
+            violations.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
+            [MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH],
+            "清扫发布那一版里仍有它删掉的那一行：判红一条：{violations:?}"
+        );
+        assert_eq!(
+            tally
+                .mount_transparency_violations_by_field
+                .get(MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH)
+                .copied(),
+            Some(1),
+            "按字段分的判红计数里这一样加 1：{}",
+            tally.render()
+        );
+        assert_eq!(
+            tally.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back, 0,
+            "被删的行还在：不放回"
+        );
+        assert_eq!(*compared, read, "R2 照原样交回给逐字段判");
+    }
+
+    /// 对照：R2 那一版是清扫发布那一版，被删的行一条都不在 ⇒ 不判红，把它们放回去之后的 R2 交给逐字段判，放回计数加 1。
+    /// 判别力（`crates/mutations.tsv` 里点名这条的行）：放回时不接上被删的行，交回的 R2 里只有留下的那一行。
+    #[test]
+    fn a_sweep_publish_without_the_rows_it_deleted_is_compared_with_them_put_back() {
+        let read = recovery_fields_read_comparing_a_version_with_these_instance_table_rows(
+            SWEEP_PUBLISH_OF_THE_C704_CASES,
+            vec![ROW_KEPT_BY_THE_SWEEP],
+        );
+        let rows_deleted = rows_deleted_by_the_sweep_of_the_c704_cases();
+        let mut tally = CrashInjectionTally::default();
+        let (compared, violations) = after_the_action_with_the_rows_deleted_by_the_sweep_put_back(
+            &mut tally,
+            &read,
+            Some(&rows_deleted),
+        );
+        assert!(
+            violations.is_empty(),
+            "被删的行都不在：不判红：{violations:?}"
+        );
+        assert!(
+            tally.mount_transparency_violations_by_field.is_empty(),
+            "按字段分的判红计数一样都没有：{}",
+            tally.render()
+        );
+        assert_eq!(
+            tally.mount_transparency_judged_with_the_rows_deleted_by_the_sweep_put_back, 1,
+            "放回一次"
+        );
+        assert_eq!(
+            compared
+                .compared_version
+                .as_ref()
+                .expect("R2 那一版在")
+                .instance_table_rows,
+            Some(vec![ROW_KEPT_BY_THE_SWEEP, ROW_DELETED_BY_THE_SWEEP]),
+            "放回之后的 R2：留下的那一行在前，被删的接在后面"
+        );
+    }
 }
```

## `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl`

```diff
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl
@@ -180,6 +180,9 @@
 
 const INVARIANT_I_9_29: u32 = 82u;
 
+// 第 84 条：根环线第 10j 批新立的 I-3.14（实例表每片记录形态合法），走读内核在 w_walk_instance_table_chain 走到的每一片上判四格（第 10k 批）
+const INVARIANT_I_3_14: u32 = 83u;
+
 // 墓碑树、谱系树叶容器的打包记录类型（walk.rs 的 PACKED_TYPE_TOMBSTONE / PACKED_TYPE_LINEAGE；登记的记录宽墓碑 40、谱系 16）
 const PACKED_TYPE_TOMBSTONE: u32 = 1u;
 
```

## `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`

```diff
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -505,6 +505,15 @@
             r_complete = false;
             return;
         }
+        // I-3.14 不成立的一片解不开（D18 已定项 11，用户 2026-10-06 补句）：链指针记录之前有不是 kind 0 的记录（kind 不认得、
+        // 片中间有链指针记录）或行的 flags 非 0，数不全（walk.rs note_instance_table_pages_after_the_first 同判）
+        for (var r = 0u; r + 1u < packed.record_count; r = r + 1u) {
+            let o = packed_record_offset(packed, r);
+            if (rd8(packed.base, o) != 0u || rd8(packed.base, o + 21u) != 0u) {
+                r_complete = false;
+                return;
+            }
+        }
         let last = packed_record_offset(packed, packed.record_count - 1u);
         let chain = w_chain_record_kind(packed.base, last, packed.record_width);
         if (chain == 0u) {
@@ -1129,7 +1138,9 @@
         }
         for (var r = 0u; r + 1u < count; r = r + 1u) {
             let o = 136u + r * width;
-            if (rd8(s.base, o) != 0u) {
+            // 链指针记录之前每一条都是 kind 0 的行、偏移 21 的 flags 为 0（D18 已定项 11「第一版恒 0、非 0 拒收」；
+            // walk.rs instance_table_rows_of_root_without_judging 同判，I-3.14 不成立的一片解不开）
+            if (rd8(s.base, o) != 0u || rd8(s.base, o + 21u) != 0u) {
                 return false;
             }
             if (rows >= C_R_MAX_ROWS) {
```

## `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`

```diff
--- a/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker.wgsl
@@ -1584,8 +1584,8 @@
 // 否则门关、I-9.29 那张表不比，CPU 走到末片照比，两边分叉）。「条目太多」一类，站点号与别处不撞
 const UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED: u32 = UNDECIDABLE_TOO_MANY_ENTRIES | (61u << 8u);
 
-// walk.rs walk_instance_table_chain（挂载根那一张另判行：I-3.8 的行那一半，行记进 C_INSTANCE_ROWS；走到末片记进
-// w_newest_instance_table_chain_reached_its_last_page）
+// walk.rs walk_instance_table_chain（每一片判 I-3.14 四格，不成立的一片解不开：行不收、链不往下走；挂载根那一张另判行：
+// I-3.8 的行那一半，行记进 C_INSTANCE_ROWS；走到末片记进 w_newest_instance_table_chain_reached_its_last_page）
 fn w_walk_instance_table_chain(first: Ptr, judge_rows: bool, fsid_low: vec2<u32>) {
     var pointer = first;
     var page_index = vec2<u32>(0u, 0u);
@@ -1594,7 +1594,6 @@
     let parent_slot = w_parent_slot;
     var rows_unique = true;
     var rows_below = true;
-    var chain_last = true;
     var pages = 0u;
     // 循环里每个提前出口都在 page 加一之前 break：走满上限才落到 page == INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED
     var page = 0u;
@@ -1608,7 +1607,17 @@
             w_failures = w_failures + 1u;
             break;
         }
-        if (!w_visit(pointer.device0, pointer.slot0)) {
+        // 「走过」按读回、自证过的那一份的落点认（walk.rs placement_it_was_read_from，攻方 H-b）：w_read_referenced_unit 交回第一份
+        // 对得上的，第一条位置条目那一份对不上时交回的是第二条那一份。重读第一条只取它对不对得上（表里现成的校验和，不另算 CRC）
+        let first_location_copy = read_at(pointer.device0, slot_to_bytes(pointer.slot0), DATA_UNIT_BYTES);
+        let read_from_the_first_location = first_location_copy.version != NONE && slice_crc_matches(first_location_copy, DATA_UNIT_BYTES, pointer.sum0);
+        var device_read_from = pointer.device1;
+        var slot_read_from = pointer.slot1;
+        if (read_from_the_first_location) {
+            device_read_from = pointer.device0;
+            slot_read_from = pointer.slot0;
+        }
+        if (!w_visit(device_read_from, slot_read_from)) {
             w_merge_below(pointer.device0, pointer.slot0);
             break;
         }
@@ -1635,10 +1644,39 @@
         if (p.record_count > 0u) {
             chain = w_chain_record_kind(p.base, packed_record_offset(p, p.record_count - 1u), p.record_width);
         }
-        w_judge(INVARIANT_I_3_8, chain != 2u);
+        // I-3.14 四格（walk.rs judge_instance_table_page_record_forms 三格，加最后一条是合法的链指针记录）：kind 只有 0 或 1、
+        // kind 1 不在最后一条之前、kind 0 行偏移 21 的 flags 为 0；走到的每一片都判（不只挂载根那一张）
+        var kinds_are_registered = true;
+        var chain_records_only_last = true;
+        var row_flags_are_zero = true;
+        for (var r = 0u; r < p.record_count; r = r + 1u) {
+            let o = packed_record_offset(p, r);
+            let kind = rd8(p.base, o);
+            if (kind == 0u) {
+                if (rd8(p.base, o + 21u) != 0u) {
+                    row_flags_are_zero = false;
+                }
+            } else if (kind == 1u) {
+                if (r + 1u < p.record_count) {
+                    chain_records_only_last = false;
+                }
+            } else {
+                kinds_are_registered = false;
+            }
+        }
+        w_judge(INVARIANT_I_3_14, kinds_are_registered);
+        w_judge(INVARIANT_I_3_14, chain_records_only_last);
+        w_judge(INVARIANT_I_3_14, row_flags_are_zero);
+        w_judge(INVARIANT_I_3_14, chain != 2u);
+        // I-3.14 不成立的一片解不开（用户 2026-10-06 给 D18 已定项 11 补的那一句）：行不收、链不往下走、不记走读失败、没走到末片
+        if (!(kinds_are_registered && chain_records_only_last && row_flags_are_zero)) {
+            break;
+        }
+        if (chain == 2u) {
+            break;
+        }
         // 行那一半的账
         if (judge_rows) {
-            var chain_last_in_this_page = false;
             for (var r = 0u; r < p.record_count; r = r + 1u) {
                 let o = packed_record_offset(p, r);
                 let kind = rd8(p.base, o);
@@ -1664,14 +1702,8 @@
                     if (instance >= w_mount_instance) {
                         rows_below = false;
                     }
-                    chain_last_in_this_page = false;
-                } else if (kind == 1u) {
-                    chain_last_in_this_page = r + 1u == p.record_count;
-                } else {
-                    chain_last_in_this_page = false;
                 }
             }
-            chain_last = chain_last && chain_last_in_this_page;
             pages = pages + 1u;
             if (chain == 0u) {
                 w_newest_instance_table_chain_reached_its_last_page = true;
@@ -1690,7 +1722,7 @@
     w_parent_device = parent_device;
     w_parent_slot = parent_slot;
     if (judge_rows && pages > 0u) {
-        w_judge(INVARIANT_I_3_8, rows_unique && rows_below && chain_last);
+        w_judge(INVARIANT_I_3_8, rows_unique && rows_below);
     }
 }
 
```

## `crates/singlefs-checker-tier/src/crash_judge_gpu.rs`

```diff
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -60,7 +60,7 @@
     "违例位读成一个 u128"
 );
 
-/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 83 条——三批 49 条，加里程碑三二·1 那一批 16 条
+/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 84 条——三批 49 条，加里程碑三二·1 那一批 16 条
 /// （判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行），下标 65 的 I-3.5（引用区间的精确性，
 /// 走读内核在回退候选集的版本都走完之后逐对判），再加 C673（三 oracle 合谋测试的补 oracle 改法没做）⑤ ⑥ 那两条
 /// （下标 66 是 I-9.27，走读内核挨着 I-9.16 按表判；下标 67 是 I-5.5，扫描内核每一版重读树表与 extent 树判），
@@ -78,12 +78,14 @@
 /// 一棵树的容器、一版的谱系记录、跨版本那张表、两两比不重号的墓碑总数越过上限的判不了。
 /// 下标 79 的 I-9.22 照下标 73 的先例恒不置位：它只在池级 checker 带旋钮值的入口上判，层 0 与放量走旧入口（`check_pool_image`，恒报不适用），
 /// GPU 也没有旋钮值。I-9.4 第四句（同一片 inode 叶容器里记录号严格递增）走读内核一并判、并进下标 40。
+/// 下标 83 是根环线第 10j 批新立的 I-3.14（实例表每片记录形态合法）：走读内核在实例表链走到的每一片上判四格（kind 只有 0 或 1、
+/// 链指针记录只在最后一条、行的 flags 为 0、最后一条是合法的链指针记录），不成立的一片照 CPU 当解不开——行不收、链不往下走（第 10k 批）。
 /// 用户 2026-10-04 定「改回要求全覆盖」：没有「内核还没判」那一张表。
 pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
     26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
     50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
-    74, 75, 76, 77, 78, 79, 80, 81, 82,
+    74, 75, 76, 77, 78, 79, 80, 81, 82, 83,
 ];
 
 // 覆盖表就是池级 checker 的每一条，按下标升序：第 k 项是 k。新立一条不变量而内核没跟上时这里编译不过，不悄悄让 GPU 那一位恒 0。
```

## `crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl`

```diff
--- a/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl
@@ -251,7 +251,8 @@
         }
         for (var r = 0u; r + 1u < count; r = r + 1u) {
             let o = 136u + r * 88u;
-            if (rd8(base, o) != 0u) {
+            // 行的 flags（偏移 21）非 0 同样解不开（InstanceTableOfRootRecord::read 同判；D18 已定项 11「第一版恒 0、非 0 拒收」）
+            if (rd8(base, o) != 0u || rd8(base, o + 21u) != 0u) {
                 return false;
             }
             if (row_count >= MAX_ROWS) {
```

## `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`

```diff
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -22,7 +22,10 @@
 //! 旋钮压出来的层级 2 墓碑树根条目类型段 1 指码 2 节点，根环转没转过一圈各一条）钉 CPU 红 I-9.23、不红走读失败连带的那几条，GPU 逐位跟上，
 //! inode 树根条目类型段 0 指码 3 叶容器（I-9.2 同形）一条同钉；
 //! I-9.29 判不了那一形（被崩溃恢复抛弃的根与现行线同号不同出生代、最新根的实例表两份翻坏）一条流整份比；
-//! 第 25 批：同一段历史、最新根的实例表链指针记录之前多一条 kind 2（攻方 N5-a，主走读走到末片、I-9.29 照比）一条流整份比。
+//! 第 25 批：同一段历史、最新根的实例表链指针记录之前多一条 kind 2（攻方 N5-a）一条流整份比；第 10k 批起那一片解不开（I-3.14 红、
+//! 主走读走不到末片、I-9.29 不比），同一段历史再加片中间多一条链指针记录、第一行 flags 非 0 两条流，三形各一条。
+//! 第 10n 批：同一段历史再加两条，都整份比——第一行 flags 非 0 且 T 抬到最新根的 txg（只读不判的读法拒收这一片，I-3.12 两边都不判它）；
+//! 拆成两片、第 0 片链指针盘 0 那条位置条目指回第 0 片自己（攻方 H-b：「走过」按读回那一份的落点认，第 1 片照走、走到末片）。
 //! 第 26 批：inode 树根条目类型段 0、身份三段清零，叶容器盘 0 那一份换成码 2 节点的陈旧字节（攻方 H1：头里的类恰是类型段那一类的副本也要自证）
 //! 一条流钉 CPU 红 I-9.2、不红 I-7.2，GPU 逐位跟上。
 //! 第 27 批：inode 叶容器类标签改成不登记的码、不重封（孩子读不出），树根条目类型段 0 带身份、类型段 7 各一条流整份比；
@@ -6477,6 +6480,57 @@
                 &insert_a_record_of_an_unregistered_kind_before_the_chain_record,
             );
         }
+        ChangeOfTheNewestInstanceTable::SecondChainRecordInsertedInTheMiddleOfThePage => {
+            corrupt_units_and_reseal_up_to_the_root(
+                &mut flow,
+                &newest_instance_table,
+                &insert_a_copy_of_the_chain_record_before_the_chain_record,
+            );
+        }
+        ChangeOfTheNewestInstanceTable::FirstRowFlagsSetToTheRetiredBit => {
+            corrupt_units_and_reseal_up_to_the_root(
+                &mut flow,
+                &newest_instance_table,
+                &set_the_flags_of_the_first_row_to_the_retired_bit,
+            );
+        }
+        ChangeOfTheNewestInstanceTable::FirstRowFlagsSetToTheRetiredBitAndItsPublishedTxgRaisedToTheNewestRoot => {
+            let newest_root_txg = flow
+                .versions
+                .iter()
+                .map(|version| version.checkpoint_txg.0)
+                .max()
+                .expect("流里至少一版");
+            corrupt_units_and_reseal_up_to_the_root(&mut flow, &newest_instance_table, &|bytes| {
+                set_the_flags_of_the_first_row_to_the_retired_bit(bytes);
+                let first_row_published_txg =
+                    PACKED_UNIT_RECORDS_START + INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET;
+                bytes[first_row_published_txg..first_row_published_txg + 8]
+                    .copy_from_slice(&newest_root_txg.to_le_bytes());
+            });
+        }
+        ChangeOfTheNewestInstanceTable::SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage => {
+            let (_, other_rows) = rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
+                &flow,
+                &newest_instance_table,
+            );
+            rebuild_the_newest_instance_table_as_a_chain(
+                &mut flow,
+                &newest_instance_table,
+                &[other_rows, Vec::new()],
+                PagesAfterTheFirstInTheFlow::Intact,
+            );
+            let first_page_copies = last_unit_copies(
+                &flow,
+                "改造成两片之后的第 0 片",
+                &is_an_instance_table_unit,
+                false,
+            );
+            let first_page_slot = flow.writes[first_page_copies[0]].offset.0 / SLOT_BYTES;
+            corrupt_units_and_reseal_up_to_the_root(&mut flow, &first_page_copies, &|bytes| {
+                point_the_first_location_of_the_chain_record_at_slot(bytes, first_page_slot);
+            });
+        }
         ChangeOfTheNewestInstanceTable::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance => {
             let (rows_of_the_abandoned_instance, other_rows) =
                 rows_of_the_newest_instance_table_split_off_the_abandoned_instance(
@@ -6531,8 +6585,25 @@
     /// （照旧比的话 C 那条记录与现行线同号不同出生代，GPU 红 I-9.29、CPU 不红）。
     BothCopiesFlippedWithoutResealing,
     /// 链指针记录之前插一条没登记种类（kind 2）的记录、重封到根（攻方 N5-a，`research/prompts/m3-batch23-code-r1-opus-output.md`「一、N5 打中」）：
-    /// 主走读照收 kind 0 的行、沿链走到末片，候选集滤掉了被抛弃的 C，CPU 与 GPU 都比、都不红 I-9.29。
+    /// I-3.14 不成立，这一片解不开（用户 2026-10-06 给 D18（块里携带什么信息） 已定项 11 补「kind 不是 0 或 1 ⇒ 这一片解不开」，第 10k 批）：
+    /// 主走读不收这一片的行、走不到末片，候选集滤不掉被抛弃的 C，CPU 报 I-9.29 部分没判、红 I-3.14，GPU 那张表不比、置 I-3.14 那一位。
     RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord,
+    /// 链指针记录之前插一条一模一样的链指针记录（kind 1、无下一片）、重封到根：I-3.14「链指针记录只在一片末尾」不成立，处置同上一形。
+    SecondChainRecordInsertedInTheMiddleOfThePage,
+    /// 第一行（kind 0）的 flags 写成退役的 bit0、重封到根：I-3.14「行的 flags 为 0」不成立（D18（块里携带什么信息） 已定项 11「第一版恒 0、
+    /// 非 0 拒收」），处置同上两形。
+    FirstRowFlagsSetToTheRetiredBit,
+    /// 同上一形，第一行（实例 1 那一行）的 T 另抬到流里最新那条根的 txg（第 10n 批）：只读不判的读法（池级 checker 的
+    /// `instance_table_rows_of_root_without_judging`、GPU 扫描内核的 `r_instance_rows`）拒收这一片，I-3.12 两边都不判这张表；
+    /// GPU 那条读法不判 flags、照收这一行时，指着这张表的实例 3 那几条根上 GPU 红 I-3.12 而 CPU 不红。记录核对器那条读法
+    /// （`InstanceTableOfRootRecord::read`、GPU 恢复内核的 `load_instance_table_of_the_effective_root`）同样拒收：实例 2 那一行照在，
+    /// GPU 照收这张表时按它把 C 那几次发布判成被抛弃、不罩，CPU 读不出这张表、照罩。
+    FirstRowFlagsSetToTheRetiredBitAndItsPublishedTxgRaisedToTheNewestRoot,
+    /// 拆成两片（第 0 片放实例 2 之外的行，第 1 片只有末片的链指针记录），第 0 片链指针记录里盘 0 那条位置条目的槽写成第 0 片自己的槽、
+    /// 整单元校验和仍是第 1 片的，盘 1 那条完好（攻方 H-b，`research/prompts/m3-batch25-code-r1-opus-output.md` 第 1.3 节；第 10n 批）。
+    /// 读第 1 片时盘 0 那一份对不上、读回的是盘 1 那一份：「走过」按读回那一份的落点认，第 1 片没走过，主走读走到末片、门开，
+    /// 候选集里没有实例 2 那一行、滤不掉 C，CPU 红 I-9.29。GPU 按第 0 条位置条目认「走过」时第 1 片被当成走过、门关，GPU 不红而 CPU 红。
+    SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage,
     /// 唯一那一片里去掉被抛弃的实例 2 那一行，链指针记录的「有无下一片」写 1、指针照旧全零（不合法的链指针记录）、重封到根
     /// （判决 `research/prompts/m3-batch25-code-r1-main-verification.md` 第三节第 2 条，GPU 关门那一侧「链指针记录不合法」一形）：
     /// 主走读没走到末片，门关，CPU 报 I-9.29 部分没判、不红，GPU 不置位、不比。门要是开了，候选集里没有实例 2 那一行、滤不掉 C，
@@ -6556,6 +6627,18 @@
             Self::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord => {
                 "abandoned-root-naming-another-object-birth-newest-instance-table-with-an-unregistered-record-kind"
             }
+            Self::SecondChainRecordInsertedInTheMiddleOfThePage => {
+                "abandoned-root-naming-another-object-birth-newest-instance-table-with-a-chain-record-in-the-middle"
+            }
+            Self::FirstRowFlagsSetToTheRetiredBit => {
+                "abandoned-root-naming-another-object-birth-newest-instance-table-with-row-flags-not-zero"
+            }
+            Self::FirstRowFlagsSetToTheRetiredBitAndItsPublishedTxgRaisedToTheNewestRoot => {
+                "abandoned-root-naming-another-object-birth-newest-instance-table-with-row-flags-not-zero-and-its-txg-not-below-the-root"
+            }
+            Self::SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage => {
+                "abandoned-root-naming-another-object-birth-newest-instance-table-second-page-pointer-on-device-zero-naming-the-first-page"
+            }
             Self::ChainRecordSayingANextPageWithoutAPointerAndNoRowOfTheAbandonedInstance => {
                 "abandoned-root-naming-another-object-birth-newest-instance-table-with-an-invalid-chain-record"
             }
@@ -6968,6 +7051,60 @@
 
 /// 实例表一片里链指针记录（最后一条）往后挪一条，原位插一条全零、种类 2 的记录，记录数与声明长度各加一条（重封归调用方）。
 fn insert_a_record_of_an_unregistered_kind_before_the_chain_record(bytes: &mut [u8]) {
+    let mut record_of_an_unregistered_kind = vec![0u8; INSTANCE_TABLE_RECORD_BYTES];
+    record_of_an_unregistered_kind[0] = INSTANCE_TABLE_RECORD_KIND_UNREGISTERED;
+    insert_a_record_before_the_chain_record(bytes, &record_of_an_unregistered_kind);
+}
+
+/// 实例表一片里链指针记录之前插一条它自己的拷贝：片中间多一条链指针记录（重封归调用方）。
+fn insert_a_copy_of_the_chain_record_before_the_chain_record(bytes: &mut [u8]) {
+    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
+    assert!(record_count >= 1, "实例表一片里至少有链指针记录");
+    let chain_record_start =
+        PACKED_UNIT_RECORDS_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
+    let chain_record =
+        bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES].to_vec();
+    insert_a_record_before_the_chain_record(bytes, &chain_record);
+}
+
+/// 实例表行记录（kind 0）里 flags 字节的偏移（kind 1 + 实例代号 4 + T 8 + W 8，D18（块里携带什么信息） 已定项 11）与退役的 bit0。
+const INSTANCE_TABLE_ROW_FLAGS_OFFSET: usize = 21;
+const INSTANCE_TABLE_ROW_FLAGS_RETIRED_BIT: u8 = 0b1;
+
+/// 实例表一片末尾那条指下一片的链指针记录里，第 0 条位置条目的槽换成 `slot`（盘与整单元校验和不动；重封归调用方）。
+fn point_the_first_location_of_the_chain_record_at_slot(bytes: &mut [u8], slot: u64) {
+    let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
+    assert!(record_count >= 1, "实例表一片里至少有链指针记录");
+    let chain_record_start =
+        PACKED_UNIT_RECORDS_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
+    assert_eq!(
+        (
+            bytes[chain_record_start],
+            bytes[chain_record_start + INSTANCE_TABLE_CHAIN_RECORD_HAS_A_NEXT_PAGE_OFFSET]
+        ),
+        (INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD, 1),
+        "最后一条是指下一片的链指针记录"
+    );
+    let first_location_slot = chain_record_start
+        + INSTANCE_TABLE_CHAIN_RECORD_POINTER_OFFSET
+        + POINTER_LOCATION_ENTRY_OFFSETS[0]
+        + 4;
+    bytes[first_location_slot..first_location_slot + 6].copy_from_slice(&slot.to_le_bytes()[..6]);
+}
+
+/// 实例表一片的第一条（实例 3 写行那一片里是一行）flags 写成退役的 bit0（重封归调用方）。
+fn set_the_flags_of_the_first_row_to_the_retired_bit(bytes: &mut [u8]) {
+    let first_row_flags = PACKED_UNIT_RECORDS_START + INSTANCE_TABLE_ROW_FLAGS_OFFSET;
+    assert_eq!(
+        (bytes[PACKED_UNIT_RECORDS_START], bytes[first_row_flags]),
+        (0, 0),
+        "实例表这一片的第一条是行、flags 是 0"
+    );
+    bytes[first_row_flags] = INSTANCE_TABLE_ROW_FLAGS_RETIRED_BIT;
+}
+
+/// 实例表一片里链指针记录（最后一条）往后挪一条，原位放 `record`，记录数与声明长度各加一条（重封归调用方）。
+fn insert_a_record_before_the_chain_record(bytes: &mut [u8], record: &[u8]) {
     let record_count = usize::from(read_u16_at(bytes, PACKED_UNIT_RECORD_COUNT_OFFSET));
     assert!(record_count >= 1, "实例表一片里至少有链指针记录");
     let chain_record_start =
@@ -6978,8 +7115,8 @@
     );
     let chain_record =
         bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES].to_vec();
-    bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES].fill(0);
-    bytes[chain_record_start] = INSTANCE_TABLE_RECORD_KIND_UNREGISTERED;
+    bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES]
+        .copy_from_slice(record);
     bytes[chain_record_start + INSTANCE_TABLE_RECORD_BYTES
         ..chain_record_start + 2 * INSTANCE_TABLE_RECORD_BYTES]
         .copy_from_slice(&chain_record);
@@ -7602,37 +7739,169 @@
     );
 }
 
-/// 攻方 N5-a 那一形的流（[`record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed`] 取
-/// [`ChangeOfTheNewestInstanceTable::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord`]；判决
-/// `research/prompts/m3-batch23-code-r1-main-verification.md` 第三节第 1 条「GPU 同改，对拍补 N5-a」）：
-/// CPU 一侧先钉流造对了——整条落盘的镜像上主走读沿最新根的实例表链走到了末片、C 被滤掉，I-9.29 真被评估过且成立
-/// （判不了的门另用只读不判的读法时这里报部分没判）。流上 C 还是最新根的那几个状态 CPU 红 I-9.29；GPU 每个状态逐位相同。
-/// 这条流上门开不开在违例位上看不出（C 滤掉了，比也不红）：GPU 门的开向由走读内核交接段那一个字与 CPU 同一遍走读钉住，
-/// 关向由上一条（两份都读不出）的流钉住。
-#[test]
-fn the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_is_filtered_by_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind(
+/// I-3.14（实例表每片记录形态合法）不成立的三形各一条流（[`record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed`]
+/// 取 `change`）共用的判法。CPU 一侧先钉流造对了——整条落盘的镜像上最新根那一片解不开（用户 2026-10-06 给 D18（块里携带什么信息） 已定项 11
+/// 补「kind 不是 0 或 1 ⇒ 这一片解不开」，第 10k 批）：主走读不收它的行、走不到末片，I-3.14 红，I-9.29 报部分没判。
+/// 流上 C 还是最新根的那几个状态 C 的实例表读得出、C 进候选集，CPU 红 I-9.29；实例 3 的根落了的状态 CPU 红 I-3.14；GPU 每个状态逐位相同
+/// （GPU 照收这一片的行、走到末片时，实例 3 的根落了、C 写回了的状态上 GPU 比那张表而红 I-9.29，CPU 不比）。
+/// `invariants_the_cpu_never_reddens` 里的每一条，整条落盘的镜像上不是违例、流上一个状态都不红（CPU 一侧钉流造对了的另一半）。
+fn assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
+    test_name: &str,
+    tag: &str,
+    change: ChangeOfTheNewestInstanceTable,
+    invariants_the_cpu_never_reddens: &[&str],
 ) {
     let (flow, _) =
         record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
-            "crash-judge-gpu-i929-newest-instance-table-unregistered-record-kind",
-            ChangeOfTheNewestInstanceTable::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord,
+            tag, change,
+        );
+    let mut image_with_the_page_that_does_not_decode = flow.base.clone();
+    image_with_the_page_that_does_not_decode.apply_writes(&flow.writes);
+    assert!(
+        matches!(
+            pool_checker_verdict_on_the_image(&image_with_the_page_that_does_not_decode, "I-3.14"),
+            InvariantVerdict::Violated(_)
+        ),
+        "{change:?}：最新根的实例表那一片记录形态不合法，I-3.14 红"
+    );
+    match pool_checker_verdict_on_the_image(&image_with_the_page_that_does_not_decode, "I-9.29") {
+        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
+            unjudged.starts_with("最新根的实例表读不出"),
+            "{change:?}：部分没判的说明点名最新根的实例表读不出：{unjudged}"
+        ),
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::NotApplicable(_)) => panic!(
+            "{change:?}：最新根的实例表那一片解不开、候选集滤不掉被抛弃的 C：I-9.29 要报部分没判，实际 {other_verdict:?}"
+        ),
+    }
+    for invariant in invariants_the_cpu_never_reddens {
+        let verdict =
+            pool_checker_verdict_on_the_image(&image_with_the_page_that_does_not_decode, invariant);
+        assert!(
+            !matches!(verdict, InvariantVerdict::Violated(_)),
+            "{change:?}：整条落盘的镜像上 {invariant} 不是违例，实际 {verdict:?}"
+        );
+    }
+    let Some(card) = one_usable_card_or_report_not_run(test_name) else {
+        return;
+    };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    for invariant in ["I-9.29", "I-3.14"] {
+        assert!(
+            comparison.cpu_checker_red_invariants.contains(invariant),
+            "流 {} 上 CPU 红 {invariant}（红过的：{:?}）",
+            flow.name,
+            comparison.cpu_checker_red_invariants
+        );
+    }
+    for invariant in invariants_the_cpu_never_reddens {
+        assert!(
+            !comparison.cpu_checker_red_invariants.contains(*invariant),
+            "流 {} 上 CPU 一个状态都不红 {invariant}（红过的：{:?}）",
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
+/// 攻方 N5-a 那一形的流（[`ChangeOfTheNewestInstanceTable::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord`]；判决
+/// `research/prompts/m3-batch23-code-r1-main-verification.md` 第三节第 1 条「GPU 同改，对拍补 N5-a」），判法见
+/// [`assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode`]。第 10k 批之前这一片照收、C 滤掉、
+/// I-9.29 成立；kind 2 那一片解不开之后改钉部分没判。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind_does_not_decode(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind_does_not_decode",
+        "crash-judge-gpu-i929-newest-instance-table-unregistered-record-kind",
+        ChangeOfTheNewestInstanceTable::RecordOfAnUnregisteredKindInsertedBeforeTheChainRecord,
+        &[],
+    );
+}
+
+/// 片中间多一条链指针记录那一形的流（[`ChangeOfTheNewestInstanceTable::SecondChainRecordInsertedInTheMiddleOfThePage`]），判法同上。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_a_chain_record_in_the_middle_does_not_decode(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_a_chain_record_in_the_middle_does_not_decode",
+        "crash-judge-gpu-i314-newest-instance-table-chain-record-in-the-middle",
+        ChangeOfTheNewestInstanceTable::SecondChainRecordInsertedInTheMiddleOfThePage,
+        &[],
+    );
+}
+
+/// 行的 flags 不是 0 那一形的流（[`ChangeOfTheNewestInstanceTable::FirstRowFlagsSetToTheRetiredBit`]），判法同上。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_row_flags_not_zero_does_not_decode(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_row_flags_not_zero_does_not_decode",
+        "crash-judge-gpu-i314-newest-instance-table-row-flags-not-zero",
+        ChangeOfTheNewestInstanceTable::FirstRowFlagsSetToTheRetiredBit,
+        &[],
+    );
+}
+
+/// 行的 flags 不是 0、那一行的 T 抬到最新根的 txg 那一形的流
+/// （[`ChangeOfTheNewestInstanceTable::FirstRowFlagsSetToTheRetiredBitAndItsPublishedTxgRaisedToTheNewestRoot`]，第 10n 批），判法同上，
+/// 另钉 CPU 一个状态都不红 I-3.12：只读不判的读法拒收这一片，指着它的根不判。GPU 扫描内核 `r_instance_rows` 不判 flags 时照收这一行、
+/// 红 I-3.12；GPU 恢复内核 `load_instance_table_of_the_effective_root` 不判 flags 时照收这张表、按实例 2 那一行不罩 C 那几次发布，
+/// 两处都与 CPU 不一致。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_row_flags_are_not_zero_is_not_read_for_the_row_txg_invariant(
+) {
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_newest_instance_table_page_that_does_not_decode(
+        "the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_row_flags_are_not_zero_is_not_read_for_the_row_txg_invariant",
+        "crash-judge-gpu-i312-newest-instance-table-row-flags-not-zero-txg-not-below-the-root",
+        ChangeOfTheNewestInstanceTable::FirstRowFlagsSetToTheRetiredBitAndItsPublishedTxgRaisedToTheNewestRoot,
+        &["I-3.12"],
+    );
+}
+
+/// 攻方 H-b 那一形的流（[`ChangeOfTheNewestInstanceTable::SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage`]，第 10n 批）：
+/// CPU 一侧先钉流造对了——改之前整条落盘的镜像上 I-9.29 真被评估过且成立；改之后整条落盘的镜像上主走读读回第 1 片盘 1 那一份、
+/// 按它的落点认「走过」、走到末片，门开，候选集里没有实例 2 那一行、滤不掉 C，I-9.29 红（不是部分没判）。
+/// GPU 每个状态逐位相同（GPU 走读内核按第 0 条位置条目认「走过」时第 1 片被当成走过、门关，实例 3 的根落了、C 写回了的状态上
+/// GPU 不红 I-9.29 而 CPU 红）。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_that_a_second_instance_table_page_whose_pointer_on_device_zero_names_the_first_page_is_walked_from_the_copy_read_back(
+) {
+    let (flow, image_with_the_newest_instance_table_readable) =
+        record_an_abandoned_root_naming_another_object_birth_with_the_newest_instance_table_changed(
+            "crash-judge-gpu-i929-newest-instance-table-second-page-pointer-on-device-zero-naming-the-first-page",
+            ChangeOfTheNewestInstanceTable::SecondPageWhosePointerOnDeviceZeroNamesTheFirstPage,
         );
-    let mut image_with_the_record_of_an_unregistered_kind = flow.base.clone();
-    image_with_the_record_of_an_unregistered_kind.apply_writes(&flow.writes);
     assert_eq!(
-        pool_checker_verdict_on_the_image(&image_with_the_record_of_an_unregistered_kind, "I-9.29"),
+        pool_checker_verdict_on_the_image(&image_with_the_newest_instance_table_readable, "I-9.29"),
         InvariantVerdict::Holds,
-        "最新根的实例表链指针记录之前多一条 kind 2：主走读收下了全部行、走到了末片，被抛弃的 C 滤掉了，I-9.29 真被评估过且成立"
+        "改实例表之前：被抛弃的 C 按最新根的实例表滤掉，I-9.29 真被评估过且成立"
+    );
+    let mut image_with_every_write = flow.base.clone();
+    image_with_every_write.apply_writes(&flow.writes);
+    let verdict = pool_checker_verdict_on_the_image(&image_with_every_write, "I-9.29");
+    assert!(
+        matches!(verdict, InvariantVerdict::Violated(_)),
+        "流 {}：主走读按读回那一份认「走过」、走到第 1 片（末片），候选集里没有实例 2 那一行，I-9.29 红，实际 {verdict:?}",
+        flow.name
     );
     let Some(card) = one_usable_card_or_report_not_run(
-        "the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_is_filtered_by_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind",
+        "the_gpu_judge_agrees_with_the_cpu_that_a_second_instance_table_page_whose_pointer_on_device_zero_names_the_first_page_is_walked_from_the_copy_read_back",
     ) else {
         return;
     };
     let comparison = compare_on_every_state(&card, &flow, expand_up_to());
     assert!(
         comparison.cpu_checker_red_invariants.contains("I-9.29"),
-        "流 {} 上 C 还是最新根的状态 CPU 红 I-9.29（红过的：{:?}）",
+        "流 {} 上 CPU 红 I-9.29（红过的：{:?}）",
         flow.name,
         comparison.cpu_checker_red_invariants
     );
```

## `crates/singlefs-core/src/allocator.rs`

```diff
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -21,6 +21,7 @@
 use crate::bytes::ByteWriter;
 use crate::instance_table::InstanceTableRecords;
 use crate::journal::slot_after_the_journal_ring;
+use crate::root_record::RootRecord;
 use crate::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};
 use crate::transaction::FrozenPublish;
 use crate::unlink::DeletionSwitches;
@@ -1007,16 +1008,15 @@
     ValidRootReadWhenTheSlotBadAtMountWasReread { checkpoint_txg: CheckpointTxg },
     /// 重建分配器读根环有一块盘没读出有效根、整环重读了一遍，两遍按槽并起来是一条自证过的根，而另一遍那一槽没自证过（读不出、读得出而
     /// 自证不过或读回全零）：并出来的照读出来的那条根算（有效的进环里最旧有效根，被抛弃的带着它引用的落点，离开根环时按它清），
-    /// 另记 `txgs_it_can_hold`——这一槽当作挂载那一刻读坏时能住的 txg（[`TxgsARingSlotBadAtMountCanHold`]，按窗口锚与新实例第一次发布的 txg 算，
-    /// 再并进读出来那条根的 txg）。抬 F 那一遍读它读坏、重读一次仍坏时照 [`RootRingOccupant::BadAtMount`] 处置：回收门槛钉在它可能住过的最旧 txg、
-    /// 准入抬 F 的上限按它能住的最大 txg 并，不拒这一次抬（用户 2026-10-06 定「另立一题改」，判决 `research/prompts/m3-batch10-code-r3-main-verification.md`
-    /// 第六节 S1：之前记成 [`RootRingOccupant::ValidRoot`] / [`RootRingOccupant::AbandonedRoot`]，算「这个进程知道住着根的槽」，抬 F 重读仍坏就拒，
-    /// 窄盘上的可写挂载在写行、暖机之后才报 `FloorRaiseFailedAfterTheMountsPublishes`）。管理员回退判候选与算 F_生效 那一遍照旧把它算进
-    /// 「这个进程知道住着根的槽」（[`RootRingOccupancy::ring_slots_known_to_hold_a_root`]：D16（发布语义） 已定项 1「根槽这一次读坏」那一行字面上
-    /// 它是「挂载那一刻读得出」的槽；这一类在那两处怎么处置条文没写，交主 agent）。
+    /// 另记读出来的那整条根记录 `root_record_self_verified_on_one_rebuild_pass`。抬 F 那一串（准入与卸载）读它读坏、重读一次仍坏时拿这条记录顶上，
+    /// 照它本身算、不拒这一次抬（用户 2026-10-06 定 T1 甲，判决 `research/prompts/m3-batch10i-code-r1-main-verification.md` 第六节）：
+    /// 有效的照它本身进准入上限与候选集，被抛弃的照它记着的落点进影子账、不进上限；有效还是被抛弃照那一遍判抛弃的实例表判。
+    /// 之前（S1，`research/prompts/m3-batch10-code-r3-main-verification.md` 第六节）照挂载那一刻读坏钉在能住的 txg：被抛弃的那一半掉出影子账（I-7.4 红），
+    /// 准入上限卡在它能住的最大 txg（同一份第 10i 批判决第二节 K1）。管理员回退判候选与算 F_生效 那一遍照旧把它算进
+    /// 「这个进程知道住着根的槽」（[`RootRingOccupancy::ring_slots_known_to_hold_a_root`]），读坏重读仍坏就拒（同一份判决第六节 T2：照今天拒）。
     RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
         root: RootReadOnOneRebuildPass,
-        txgs_it_can_hold: TxgsARingSlotBadAtMountCanHold,
+        root_record_self_verified_on_one_rebuild_pass: Box<RootRecord>,
     },
     /// 按实例表判仍然有效的根，txg 在不在 F 之下都算（可再分配谓词取 max(F_生效, 环里最旧有效根)，F 那一半另记）。
     ValidRoot { checkpoint_txg: CheckpointTxg },
@@ -1107,19 +1107,19 @@
     }
 
     /// 重建分配器两遍里只有一遍自证过、还记在表里的槽（[`RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`]），
-    /// 各带它当作挂载那一刻读坏时能住的 txg，按槽的次序：抬 F 那一遍读它读坏、重读一次仍坏时，回收门槛按其中可能住过的最旧 txg 钉、
-    /// 准入抬 F 的上限按其中能住的最大 txg 并（用户 2026-10-06 定，S1）。这个进程写掉那一槽之后就不在其内。
+    /// 各带重建那一遍自证过的那整条根记录，按槽的次序：抬 F 那一串读它读坏、重读一次仍坏时拿这条记录顶上（用户 2026-10-06 定 T1 甲）。
+    /// 这个进程写掉那一槽之后就不在其内。
     #[must_use]
     pub fn ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes(
         &self,
-    ) -> BTreeMap<RootRingSlot, TxgsARingSlotBadAtMountCanHold> {
+    ) -> BTreeMap<RootRingSlot, RootRecord> {
         self.occupants
             .iter()
             .filter_map(|(ring_slot, occupant)| match occupant {
                 RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
-                    txgs_it_can_hold,
+                    root_record_self_verified_on_one_rebuild_pass,
                     ..
-                } => Some((*ring_slot, *txgs_it_can_hold)),
+                } => Some((*ring_slot, **root_record_self_verified_on_one_rebuild_pass)),
                 RootRingOccupant::ValidRoot { .. }
                 | RootRingOccupant::BadAtMount(_)
                 | RootRingOccupant::AllZeroAtMount { .. }
```

## `crates/singlefs-core/src/mount.rs`

```diff
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -182,7 +182,7 @@
     /// `allocator::RootRingOccupancy::ring_slots_known_to_hold_a_root`），这一次读坏（`first_reading`），重读一次仍坏（`reread`）：
     /// 这条根在不在判不了，不按有根或没根猜，这次抬 F 拒绝（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-09-26 定；
     /// C562（抬 F 算上限时根槽读坏当没有根））。在回收、影子账与任何写之前返回：盘上逐字节不变、分配器不动、调用方的现行版本不动。
-    /// 这个进程的准入抬 F 里重建分配器两遍里只有一遍自证过的槽不走这一条（重读仍坏照挂载那一刻读坏并进上限，用户 2026-10-06 定 S1）。
+    /// 这个进程的准入抬 F 里重建分配器两遍里只有一遍自证过的槽不走这一条（重读仍坏拿重建那一遍读出、记在根环表里的那条根顶上，用户 2026-10-06 定 T1 甲）。
     RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {
         ring_slot: RootRingSlot,
         first_reading: BadRootRingSlotReading,
@@ -381,7 +381,7 @@
     /// 判候选、算 F_生效 读根环：`ring_slot` 是这个进程知道住着一条根的槽（挂载那一刻读得出、或这个进程写过且 FUA 返回过），这一次读坏
     /// （`first_reading`：读不出或自证不过），重读一次仍坏（`reread`）——那一槽里的根在不在判不了，不按有根或没根猜（D16（发布语义） 已定项 1
     /// 「根槽这一次读坏」那一行全句，用户 2026-09-26 定）。挂载那一刻就读坏、之后没写过的槽不走这一条，当没有根；重建分配器两遍里只有一遍
-    /// 自证过的槽在抬 F 那一遍也不走这一条（照挂载那一刻读坏钉住，用户 2026-10-06 定 S1），管理员回退那一遍照走。
+    /// 自证过的槽在抬 F 那一遍也不走这一条（拿重建那一遍读出、记在根环表里的那条根顶上，用户 2026-10-06 定 T1 甲），管理员回退那一遍照走（同日定 T2：照今天拒）。
     /// 实审 A3b 报告 Q4：与乙那一族同级，改之前是 `RecoveryFailure::RootRingSlotStillUnreadableAfterOneReread`（那时不分槽知不知道住着根）。
     RootRingSlotKnownToHoldARoot {
         ring_slot: RootRingSlot,
@@ -2076,7 +2076,7 @@
         newest_valid_root_txg_of_the_readings(&first_readings)
             .filter(|_| a_valid_root_was_read_on_every_device(&first_readings));
     // 两遍里只有一遍自证过的槽另记一类（用户 2026-10-06 定 S1，`RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses`）：
-    // 并出来照读出来的根算，抬 F 那一遍读它仍坏时照挂载那一刻读坏处置、不拒。只读了一遍的没有这一类（保持空集）。
+    // 并出来照读出来的根算，抬 F 那一串读它仍坏时拿读出来的那条根顶上、不拒（用户 2026-10-06 定 T1 甲）。只读了一遍的没有这一类（保持空集）。
     let mut ring_slots_self_verified_on_only_one_of_the_two_passes: BTreeSet<RootRingSlot> =
         BTreeSet::new();
     let (ring_slot_readings, window_anchor, root_ring_of_the_rebuilt_allocator) =
@@ -2238,36 +2238,7 @@
     // 读回全零的槽记成「挂载时读回全零」，只在回收门槛里钉在它可能住过的最旧 txg（用户 2026-10-05 定乙）。两类都钉到这个进程写到那个槽、
     // 或进程内重读读出有效根或被抛弃的根为止（用户 2026-10-04 定）。同一个槽这一遍读出了根的，读出来的那条在后面、盖掉它
     // （这一遍读的是同一个槽，几样不会同时成立，留着次序只为不依赖这一点）。
-    // 两遍里只有一遍自证过的槽当作挂载那一刻读坏时能住的 txg：与读坏的槽同一个算法（[`txgs_ring_slots_bad_at_mount_can_hold`]），
-    // 再并进读出来那条根的 txg——第 0 代根住在落点公式算不到的槽里（`make_filesystem::genesis_root_ring_slots`），公式给不出或给的不含它。
-    let txgs_ring_slots_self_verified_on_only_one_pass_can_hold: BTreeMap<
-        RootRingSlot,
-        TxgsARingSlotBadAtMountCanHold,
-    > = txgs_ring_slots_bad_at_mount_can_hold(
-        &ring_slots_self_verified_on_only_one_of_the_two_passes,
-        window_anchor,
-        first_txg,
-        slots_per_region,
-    )
-    .into_iter()
-    .collect();
-    let txgs_it_can_hold_if_read_bad = |ring_slot: RootRingSlot, root: &RootRecord| {
-        txgs_ring_slots_self_verified_on_only_one_pass_can_hold
-            .get(&ring_slot)
-            .map_or_else(
-                || TxgsARingSlotBadAtMountCanHold::new(root.checkpoint_txg, root.checkpoint_txg),
-                |txgs_it_can_hold| {
-                    TxgsARingSlotBadAtMountCanHold::new(
-                        txgs_it_can_hold
-                            .oldest_txg_it_can_have_held()
-                            .min(root.checkpoint_txg),
-                        txgs_it_can_hold
-                            .highest_txg_it_can_hold()
-                            .max(root.checkpoint_txg),
-                    )
-                },
-            )
-    };
+    // 两遍里只有一遍自证过的槽另记读出来的那整条根记录：抬 F 那一串读它读坏、重读一次仍坏时拿它顶上（用户 2026-10-06 定 T1 甲）。
     let occupants: Vec<(RootRingSlot, RootRingOccupant)> = ring_slots_bad_at_mount
         .iter()
         .map(|(ring_slot, txgs_it_can_hold)| {
@@ -2302,7 +2273,7 @@
                 (true, root_read) => {
                     RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                         root: root_read,
-                        txgs_it_can_hold: txgs_it_can_hold_if_read_bad(*ring_slot, root),
+                        root_record_self_verified_on_one_rebuild_pass: Box::new(*root),
                     }
                 }
                 (false, RootReadOnOneRebuildPass::ValidRoot { checkpoint_txg }) => {
@@ -2484,18 +2455,21 @@
 /// 算抬 F 的上限时读根环（D16（发布语义） 已定项 1「根槽这一次读坏」那一行，用户 2026-09-26 定）：每个槽读一次；读坏的槽按这个进程知不知道
 /// 那里住着一条根分两类——`ring_slots_known_to_hold_a_root` 里没有的（挂载那一刻就读不出或自证不过、之后也没写过）当没有根，
 /// 有的（挂载那一刻读得出、或这个进程写过且 FUA 返回过）重读一次，读出来就用重读的那一条，仍坏就拒，不按有根或没根猜。
-/// `ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread` 里的槽（重建分配器两遍里只有一遍自证过的，用户 2026-10-06 定 S1）
-/// 读坏也重读一次、读出来就用重读的那一条，仍坏不拒、当没有根，交给调用方照挂载那一刻读坏处置（上限按它能住的最大 txg 并）。
-/// 交回读得出的每一条根（按根环槽的次序）与这一遍读坏、当没有根的那几个槽。
+/// `root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass` 里的槽（重建分配器两遍里只有一遍自证过的，各带重建那一遍读出的整条根记录）
+/// 读坏也重读一次、读出来就用重读的那一条，仍坏不拒、拿记着的那条根顶上（用户 2026-10-06 定 T1 甲：有效的照它本身进上限，被抛弃的由调用方按实例表滤掉）。
+/// 交回读得出（或顶上）的每一条根（按根环槽的次序）与这一遍读坏、当没有根的那几个槽。
 ///
 /// # Errors
-/// 知道住着根的槽（不在 `ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread` 里的）重读仍坏 ⇒
+/// 知道住着根的槽（不在 `root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass` 里的）重读仍坏 ⇒
 /// `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`。
 fn roots_of_the_ring_for_the_ceiling<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     system_configuration: &crate::system_configuration::SystemConfiguration,
     ring_slots_known_to_hold_a_root: &BTreeSet<RootRingSlot>,
-    ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread: &BTreeSet<RootRingSlot>,
+    root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass: &BTreeMap<
+        RootRingSlot,
+        RootRecord,
+    >,
 ) -> Result<RootsOfTheRingForTheCeiling, MountError> {
     let read_once = |ring_slot: RootRingSlot| {
         read_root_ring_slot(
@@ -2517,11 +2491,13 @@
             }
             RootRingSlotReading::Bad(first_reading) => first_reading,
         };
-        if ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread.contains(&ring_slot) {
+        if let Some(root_record_self_verified_on_one_rebuild_pass) =
+            root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass.get(&ring_slot)
+        {
             match read_once(ring_slot) {
                 RootRingSlotReading::SelfVerified(root) => roots.push(root),
                 RootRingSlotReading::Bad(_still_bad) => {
-                    ring_slots_bad_on_this_read_taken_as_holding_no_root.insert(ring_slot);
+                    roots.push(*root_record_self_verified_on_one_rebuild_pass);
                 }
             }
             continue;
@@ -2551,9 +2527,9 @@
 
 /// 算上限那一遍读根环读出来的（[`roots_of_the_ring_for_the_ceiling`]）。
 struct RootsOfTheRingForTheCeiling {
-    /// 读得出、自证过的每一条根，按根环槽的次序。
+    /// 读得出、自证过的每一条根，与重建分配器两遍里只有一遍自证过、这一遍重读一次仍坏的槽记着的那条根，按根环槽的次序。
     roots: Vec<RootRecord>,
-    /// 这一遍读不出或自证不过、又不是「这个进程知道住着根的槽」的槽，与重建分配器两遍里只有一遍自证过、这一遍重读一次仍坏的槽：当没有根。
+    /// 这一遍读不出或自证不过、又不是「这个进程知道住着根的槽」的槽：当没有根。
     ring_slots_bad_on_this_read_taken_as_holding_no_root: BTreeSet<RootRingSlot>,
 }
 
@@ -2604,8 +2580,9 @@
 /// 自证不过的那一类同一个算法（区域内槽位只由 txg 定）。这一遍读那一槽又读坏（读不出或自证不过）、那个 txg 不低于今天的 F，上限就取不高于它。
 /// 这一遍读得出的照读出来的根算，不另封顶。判「非空」、找「前一条」照旧只在读得出的根上取（读不出的那一条判不了空不空，F 不越过它就罩住了它是第 4 新的那一种）。
 /// `ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes` 是这个进程根环表里重建分配器两遍里只有一遍自证过的槽，
-/// 各按它当作挂载那一刻读坏时能住的最大 txg（[`ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by`]，用户 2026-10-06 定 S1）：
-/// 这一遍读坏就重读一次，读出来照读出来的算，仍坏不拒、照挂载那一刻读坏那一类并进上限（它们也在 `ring_slots_known_to_hold_a_root` 里，这里先按它们分）。
+/// 各带重建那一遍读出的整条根记录（[`ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by`]）：这一遍读坏就重读一次，
+/// 读出来照读出来的算，仍坏不拒、拿记着的那条根顶上，与读出来的根一样按 `table` 判有效（用户 2026-10-06 定 T1 甲：有效的照它本身进上限，
+/// 被抛弃的不进；它们也在 `ring_slots_known_to_hold_a_root` 里，这里先按它们分）。
 ///
 /// # Errors
 /// 同 [`rollback_floor_ceiling`]。
@@ -2622,7 +2599,7 @@
     ring_slots_bad_since_the_mount: &BTreeMap<RootRingSlot, CheckpointTxg>,
     ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes: &BTreeMap<
         RootRingSlot,
-        CheckpointTxg,
+        RootRecord,
     >,
 ) -> Result<CheckpointTxg, MountError> {
     let RootsOfTheRingForTheCeiling {
@@ -2632,17 +2609,8 @@
         devices,
         system_configuration,
         ring_slots_known_to_hold_a_root,
-        &ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes
-            .keys()
-            .copied()
-            .collect(),
+        ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes,
     )?;
-    let ring_slots_bad_since_the_mount: BTreeMap<RootRingSlot, CheckpointTxg> =
-        ring_slots_bad_since_the_mount
-            .iter()
-            .chain(ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes)
-            .map(|(ring_slot, highest_txg_it_can_hold)| (*ring_slot, *highest_txg_it_can_hold))
-            .collect();
     let valid: Vec<RootRecord> = readable
         .iter()
         .copied()
@@ -3035,13 +3003,14 @@
     )
 }
 
-/// 这个进程分配器上那张根环表里重建分配器两遍里只有一遍自证过、之后没写过的槽，各带它当作挂载那一刻读坏时能住的 txg
-/// （`allocator::RootRingOccupancy::ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes`，用户 2026-10-06 定 S1）；
-/// 没装根环表的分配器给空表。抬 F 那一遍读它们读坏、重读一次仍坏时照挂载那一刻读坏处置、不拒（[`raise_rollback_floor`] 一族）。
+/// 这个进程分配器上那张根环表里重建分配器两遍里只有一遍自证过、之后没写过的槽，各带重建那一遍读出的整条根记录
+/// （`allocator::RootRingOccupancy::ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes`）；
+/// 没装根环表的分配器给空表。抬 F 那一串（准入算上限、抬 F 重算影子账与回收门槛，卸载同一串）读它们读坏、重读一次仍坏时拿记着的那条根顶上、
+/// 不拒（用户 2026-10-06 定 T1 甲，[`raise_rollback_floor`] 一族）。
 #[must_use]
 pub fn ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(
     allocator: &PoolAllocator,
-) -> BTreeMap<RootRingSlot, TxgsARingSlotBadAtMountCanHold> {
+) -> BTreeMap<RootRingSlot, RootRecord> {
     allocator.root_ring_occupancy().map_or_else(
         BTreeMap::new,
         RootRingOccupancy::ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes,
@@ -3049,7 +3018,7 @@
 }
 
 /// 这个进程的准入抬 F 的上限（[`rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`]），三类槽都取这个进程分配器上那张根环表：
-/// 知道住着根的槽、挂载那一刻读坏的槽、重建分配器两遍里只有一遍自证过的槽（后一类按能住的最大 txg 并，用户 2026-10-06 定 S1）。
+/// 知道住着根的槽、挂载那一刻读坏的槽、重建分配器两遍里只有一遍自证过的槽（后一类读坏重读仍坏时拿记着的那条根顶上，用户 2026-10-06 定 T1 甲）。
 ///
 /// # Errors
 /// 同 [`rollback_floor_ceiling`]。
@@ -3071,12 +3040,7 @@
         table,
         &ring_slots_known_to_hold_a_root_by(allocator),
         &ring_slots_bad_at_mount_by(allocator),
-        &ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator)
-            .into_iter()
-            .map(|(ring_slot, txgs_it_can_hold)| {
-                (ring_slot, txgs_it_can_hold.highest_txg_it_can_hold())
-            })
-            .collect(),
+        &ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator),
     )
 }
 
@@ -3310,20 +3274,12 @@
     }
 }
 
-/// 抬 F 那一遍重算影子账与回收门槛读根环读出来的（[`readable_roots_for_raising_the_floor`]）。
-struct ReadableRootsForRaisingTheFloor {
-    /// 读得出、自证过的每一条根（重读读出来的排在后面）。
-    roots: Vec<RootRecord>,
-    /// 重建分配器两遍里只有一遍自证过的槽，这一遍读坏、重读一次仍坏的：各按它当作挂载那一刻读坏时可能住过的最旧 txg。
-    ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad:
-        BTreeMap<RootRingSlot, CheckpointTxg>,
-}
-
 /// 抬 F 那一遍重算影子账与回收门槛读根环（D16（发布语义） 已定项 1「根槽这一次读坏」那一行）：每个槽读一次；读坏的槽分三类——
 /// 这个进程不知道住着根的当没有根；知道住着根的重读一次，仍坏就拒（C554 乙报告 Q6）；重建分配器两遍里只有一遍自证过的
 /// （[`ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by`]，它们也在「知道住着根的槽」里，这里先按它们分）重读一次，
-/// 读出来照读出来的算，仍坏不拒、交给调用方照挂载那一刻读坏钉在它可能住过的最旧 txg（用户 2026-10-06 定 S1：之前与知道住着根的槽同一处置，
-/// 窄盘上的可写挂载在写行、暖机之后推的抬 F 被拒）。只读盘。
+/// 读出来照读出来的算，仍坏不拒、拿重建那一遍读出、记在根环表里的那条根顶上（用户 2026-10-06 定 T1 甲：有效的照它本身进回收门槛与候选集，
+/// 被抛弃的照它引用的落点进影子账；有效还是被抛弃由调用方按同一张实例表判）。交回读得出（或顶上）的每一条根（重读读出来的与顶上的排在后面）。
+/// 只读盘。
 ///
 /// # Errors
 /// 知道住着根的槽（两遍里只有一遍自证过的除外）重读仍坏 ⇒
@@ -3332,7 +3288,7 @@
     reader: &Reader,
     system_configuration: &crate::system_configuration::SystemConfiguration,
     allocator: &PoolAllocator,
-) -> Result<ReadableRootsForRaisingTheFloor, MountError> {
+) -> Result<Vec<RootRecord>, MountError> {
     let ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass =
         ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(allocator);
     let ring_slots_whose_reread_still_bad_refuses_the_raise: BTreeSet<RootRingSlot> =
@@ -3358,10 +3314,8 @@
         .iter()
         .map(|(ring_slot, _)| *ring_slot)
         .collect();
-    let mut ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad =
-        BTreeMap::new();
-    // 迭代上界是两遍里只有一遍自证过的槽数（至多根环的槽数）；跨轮只往两张表里加；没有提前出口。上面那一遍读坏、当没有根的才在这里重读一次。
-    for (ring_slot, txgs_it_can_hold) in
+    // 迭代上界是两遍里只有一遍自证过的槽数（至多根环的槽数）；跨轮只往交回的根里加；没有提前出口。上面那一遍读坏、当没有根的才在这里重读一次。
+    for (ring_slot, root_record_self_verified_on_one_rebuild_pass) in
         &ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass
     {
         if ring_slots_read.contains(ring_slot) {
@@ -3377,37 +3331,27 @@
             RootRingSlotReading::SelfVerified(root) => {
                 roots_with_ring_slots.push((*ring_slot, root))
             }
-            RootRingSlotReading::Bad(_still_bad) => {
-                ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad
-                    .insert(*ring_slot, txgs_it_can_hold.oldest_txg_it_can_have_held());
-            }
+            RootRingSlotReading::Bad(_still_bad) => roots_with_ring_slots
+                .push((*ring_slot, *root_record_self_verified_on_one_rebuild_pass)),
         }
     }
-    Ok(ReadableRootsForRaisingTheFloor {
-        roots: roots_with_ring_slots
-            .into_iter()
-            .map(|(_, root)| root)
-            .collect(),
-        ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad,
-    })
+    Ok(roots_with_ring_slots
+        .into_iter()
+        .map(|(_, root)| root)
+        .collect())
 }
 
 /// 抬 F 那一遍的回收门槛里「环里最旧有效根」（D16（发布语义） 已定项 1，用户 2026-10-04 定 RGall：不按一圈窗口滤）：这一遍读出来的根里
 /// 按 `table` 有效的，并上分配器根环表里挂载那一刻读坏或读回全零、这一遍也没在那一槽读出根的槽（各钉在它可能住过的最旧 txg，用户 2026-10-04 定，
 /// 读回全零的 2026-10-05 定乙；
-/// 并进来是 RR4，`research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表一，攻方 B1），再并上
-/// `ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad`（重建分配器两遍里只有一遍自证过、这一遍重读一次仍坏的槽，
-/// 照挂载那一刻读坏钉在它可能住过的最旧 txg，用户 2026-10-06 定 S1），取最小；最小的是一个读坏的槽时照 E2
-/// 先重读那一槽（[`oldest_txg_in_the_ring_rereading_slots_bad_at_mount`]，与进程内记根同一取法）。只读盘，不改分配器。
+/// 并进来是 RR4，`research/prompts/m3-root-ring-code-r1-main-verification.md` 第三节表一，攻方 B1），取最小；最小的是一个读坏的槽时照 E2
+/// 先重读那一槽（[`oldest_txg_in_the_ring_rereading_slots_bad_at_mount`]，与进程内记根同一取法）。重建分配器两遍里只有一遍自证过、
+/// 这一遍重读一次仍坏的槽不另钉：顶上的那条根已在 `roots_read_from_the_ring` 里（用户 2026-10-06 定 T1 甲）。只读盘，不改分配器。
 fn oldest_valid_root_in_the_ring_for_raising_the_floor<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     parameters_of_the_pool: &MakeFilesystemParameters,
     allocator: &PoolAllocator,
     roots_read_from_the_ring: &[RootRecord],
-    ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad: &BTreeMap<
-        RootRingSlot,
-        CheckpointTxg,
-    >,
     table: &InstanceTableRecords,
 ) -> Option<CheckpointTxg> {
     let slots_per_region = parameters_of_the_pool.geometry.root_ring_slots_per_region;
@@ -3428,13 +3372,6 @@
         )
         .into_iter()
         .filter(|(ring_slot, _)| !ring_slots_read_from_the_ring.contains(ring_slot))
-        .chain(
-            ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad
-                .iter()
-                .map(|(ring_slot, oldest_txg_it_can_have_held)| {
-                    (*ring_slot, *oldest_txg_it_can_have_held)
-                }),
-        )
         .collect();
     oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
         &valid_root_txgs,
@@ -3555,12 +3492,10 @@
     // 回收门槛与影子账读的这一遍根环，照 D16（发布语义） 已定项 1「根槽这一次读坏」那一行分两类（与算上限那一遍同一个分法）：
     // 这个进程知道住着根的槽读坏就重读那一槽一次，仍坏就拒这一次抬（C554 乙报告 Q6：按「没有根」往下走，那一槽里的被抛弃根引用的槽
     // 不隔离、也不计数）；挂载那一刻就读坏、之后没写过的槽当没有根；重建分配器两遍里只有一遍自证过的槽读坏也重读一次，仍坏不拒、
-    // 照挂载那一刻读坏钉在它可能住过的最旧 txg（用户 2026-10-06 定 S1，[`readable_roots_for_raising_the_floor`]）。
-    // 在动分配器（影子账重算、回收）之前返回，分配器与盘都不动。
-    let ReadableRootsForRaisingTheFloor {
-        roots,
-        ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad,
-    } = readable_roots_for_raising_the_floor(&**devices, &system_configuration, allocator)?;
+    // 拿重建那一遍读出、记在根环表里的那条根顶上（用户 2026-10-06 定 T1 甲，[`readable_roots_for_raising_the_floor`]）：被抛弃的照它引用的落点
+    // 进下面的影子账重算（不顶上时它引用、挂载那一刻被候选根豁免的槽按新 F 回收之后发得出去，判决第二节 K1 攻方 K1-1，I-7.4 红），
+    // 有效的照它本身进回收门槛与候选集。在动分配器（影子账重算、回收）之前返回，分配器与盘都不动。
+    let roots = readable_roots_for_raising_the_floor(&**devices, &system_configuration, allocator)?;
     // 根环表里挂载那一刻读坏、这一遍也没读出根的槽，钉在它可能住过的最旧 txg 并进环里最旧有效根（RR4，攻方 B1；用户 2026-10-04 定钉在最旧 txg；与进程内记根同一取法，
     // E2 的重读照用：它成了最小的那个时先重读，`oldest_valid_root_in_the_ring_for_raising_the_floor`）。
     let oldest_valid_root_in_the_ring = oldest_valid_root_in_the_ring_for_raising_the_floor(
@@ -3568,7 +3503,6 @@
         &parameters_of_the_pool,
         allocator,
         &roots,
-        &ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass_still_bad,
         &table,
     );
     // 候选集按新 F 缩小，只被被抛弃根引用的槽会变多（一个槽此前靠一条低于新 F 的根豁免），回收之前先按新 F 重算影子账，
```

## `crates/singlefs-harness/tests/checker_known_bad_images.rs`

```diff
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -19,6 +19,7 @@
     FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
     HEAD_IDENTIFIERS_OF_THE_TREE_TABLE_ENTRIES,
     INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
+    INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
     INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
     INTENT_DESCRIBES_ONLY_THE_TARGET_STATE, INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION,
     JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
@@ -40,7 +41,7 @@
     TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT,
     UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
 };
-use singlefs_checker::walk::check_pool_image;
+use singlefs_checker::walk::{check_pool_image, InstanceTableOfRootRecord};
 use singlefs_core::address::{
     CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
 };
@@ -3017,6 +3018,25 @@
     ),
 ];
 
+/// I-3.14（实例表每片记录形态合法） 的三张坏镜像各建在攻方 N5 那一份基底上（崩溃恢复抛弃 C、C 与现行线同号不同出生代，
+/// [`image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed`]），不进只改干净镜像一处的表；
+/// 这里登记它们在哪几个用例里（各自断言判红的是 I-3.14 与这一片解不开带出来的那几条）。最后一条不是合法的链指针记录那一格的坏镜像在
+/// `instance_table_chain_read.rs` 的 `contradictory_chain_record_reddens_only_the_instance_table_invariant`。
+const INSTANCE_TABLE_RECORD_FORM_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE: [(&str, &str); 3] = [
+    (
+        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
+        "checker_known_bad_images.rs: a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set",
+    ),
+    (
+        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
+        "checker_known_bad_images.rs: a_second_chain_record_in_the_middle_of_the_newest_instance_table_page_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set",
+    ),
+    (
+        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
+        "checker_known_bad_images.rs: a_row_whose_flags_are_not_zero_in_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set",
+    ),
+];
+
 /// 清单里不要求坏镜像的几条：I-4.4（意图只描述目标态） 第一版报不适用（用户 2026-10-05 定，`.claude/kb/milestone/03-third-txn.md` 三·3 那一行
 /// 「I-4.4 不要求坏镜像与变异行」）。
 const INVARIANTS_WITHOUT_A_BAD_IMAGE: [&str; 1] = [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE];
@@ -3129,6 +3149,11 @@
                 .iter()
                 .map(|(invariant, _)| *invariant),
         )
+        .chain(
+            INSTANCE_TABLE_RECORD_FORM_INVARIANT_AND_WHERE_ITS_BAD_IMAGES_LIVE
+                .iter()
+                .map(|(invariant, _)| *invariant),
+        )
         .collect();
     let listed: std::collections::BTreeSet<&str> = singlefs_checker::image::IMPLEMENTED_INVARIANTS
         .iter()
@@ -4042,16 +4067,142 @@
 }
 
 /// 攻方 N5-a（`research/prompts/m3-batch23-code-r1-opus-output.md`「一、N5 打中」；判决 `research/prompts/m3-batch23-code-r1-main-verification.md`
-/// 第三节第 1 条）：最新根的实例表在链指针记录之前多插一条没登记种类（kind 2）的记录，重封、补到根槽。主走读照收 kind 0 的行、沿链走到末片，
-/// 候选集按这张表滤掉了被抛弃的 C：I-9.29（同号只许一个出生代） 真被评估过且成立。这张镜像别的几条红不红不在这里钉
-/// （实例表记录种类合不合法归根环线，同一判决第二节「附带：N5-a 那张镜像」）。
-/// 判别力：判不了的门另用只读不判的读法（它见到链指针记录之前不是行的记录就交回 None）时，这里报部分没判。
+/// 第三节第 1 条）：最新根的实例表在链指针记录之前多插一条没登记种类（kind 2）的记录，重封、补到根槽。用户 2026-10-06 给 D18（块里携带什么信息）
+/// 已定项 11 补「kind 不是 0 或 1 ⇒ 这一片解不开（表不可读）」（`records/2026-09-30-里程碑三开工第一轮.md`「用户 2026-10-06 弹窗定实例表 kind 那两问」）：
+/// 主走读不收这一片的行、走不到末片，候选集滤不掉被抛弃的 C，I-9.29（同号只许一个出生代） 报部分没判。第 10k 批之前这里钉的是「照收照滤、I-9.29 成立」。
+/// 判别力：这一片照收、照走到末片（`crates/mutations.tsv` 里点名这条的行）时 C 被滤掉，I-9.29 报成立。
 #[test]
-fn a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_still_filters_the_abandoned_root_so_the_one_birth_invariant_holds(
+fn a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_leaves_the_one_birth_invariant_partially_judged(
 ) {
     let (image, instance_table_slot) =
         image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
             "known-bad-one-birth-unregistered-record-kind-before-the-chain-record",
+            |bytes| insert_a_record_of_an_unregistered_kind_before_the_chain_record(bytes),
+            true,
+        );
+    match verdict(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER) {
+        InvariantVerdict::PartiallyJudged(unjudged) => assert!(
+            unjudged.starts_with("最新根的实例表读不出"),
+            "部分没判的说明点名最新根的实例表读不出：{unjudged}"
+        ),
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::NotApplicable(_)) => panic!(
+            "最新根的实例表（槽 {instance_table_slot}）链指针记录之前多一条 kind 2：这一片解不开，候选集滤不掉被抛弃的 C，I-9.29 要报部分没判，实际 {other_verdict:?}"
+        ),
+    }
+}
+
+/// 实例表一片（整个单元的字节，两道校验和由调用方重封）在末尾的链指针记录之前插一条 `record`：链指针记录往后挪一格，记录数与声明长度各加一条。
+fn insert_a_record_before_the_chain_record_of_the_instance_table_page(
+    bytes: &mut [u8],
+    record: &[u8],
+) {
+    let record_count = usize::from(u16::from_le_bytes([
+        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET],
+        bytes[PACKED_UNIT_RECORD_COUNT_OFFSET + 1],
+    ]));
+    let chain_record_start =
+        INSTANCE_TABLE_FIRST_ROW_START + (record_count - 1) * INSTANCE_TABLE_RECORD_BYTES;
+    let chain_record =
+        bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES].to_vec();
+    bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES]
+        .copy_from_slice(record);
+    bytes[chain_record_start + INSTANCE_TABLE_RECORD_BYTES
+        ..chain_record_start + 2 * INSTANCE_TABLE_RECORD_BYTES]
+        .copy_from_slice(&chain_record);
+    let new_record_count = record_count + 1;
+    set_u16(
+        bytes,
+        PACKED_UNIT_RECORD_COUNT_OFFSET,
+        u16::try_from(new_record_count).expect("一片装得下"),
+    );
+    set_u16(
+        bytes,
+        PACKED_UNIT_DECLARED_LENGTH_OFFSET,
+        u16::try_from(new_record_count * INSTANCE_TABLE_RECORD_BYTES).expect("一片装得下"),
+    );
+}
+
+/// 攻方 N5-a 的造法：链指针记录之前插一条全零、只有首字节是没登记的种类（kind 2）的记录。
+fn insert_a_record_of_an_unregistered_kind_before_the_chain_record(bytes: &mut [u8]) {
+    let mut record_of_an_unregistered_kind = vec![0u8; INSTANCE_TABLE_RECORD_BYTES];
+    record_of_an_unregistered_kind[0] = INSTANCE_TABLE_RECORD_KIND_UNREGISTERED;
+    insert_a_record_before_the_chain_record_of_the_instance_table_page(
+        bytes,
+        &record_of_an_unregistered_kind,
+    );
+}
+
+/// 实例表行记录（kind 0）里 flags 字节的偏移（kind 1 + 实例代号 4 + T 8 + W 8，D18（块里携带什么信息） 已定项 11），
+/// 与退役的 bit0（「曾标回退行，以后不给它另赋含义」）。
+const INSTANCE_TABLE_ROW_FLAGS_OFFSET: usize = 21;
+const INSTANCE_TABLE_ROW_FLAGS_RETIRED_BIT: u8 = 0b1;
+
+/// 攻方 N5 那一份基底上最新根的实例表那一片解不开（I-3.14 不成立，第 10k 批）时判红的几条，按 `IMPLEMENTED_INVARIANTS` 的次序：
+/// I-3.14 本身；走不到末片、那一片的行不收，被抛弃的 C 进了回退候选集，它那一版引用的单元算进遍历得到的已分配（I-3.1 与记账对不上）、
+/// I-9.10 连带红。I-3.14 四形一律如此：引用集合那一遍（`singlefs_checker::walk` 的 `note_instance_table_pages_after_the_first`）
+/// 第 10n 批起四形都记数不全，第 10k 批时另三形照记全、多红 I-3.9。量过：第 10n 批副本上这四张镜像 `violated_invariants` 的原样。
+const VIOLATED_ONCE_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE: [&str; 3] = [
+    "I-3.1",
+    "I-9.10",
+    INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
+];
+
+/// 最新根的实例表第 0 片就解不开时，I-3.8（实例表行唯一且低于挂载根）一行都没收、报不适用的理由（`singlefs_checker::walk` 的
+/// `walk_instance_table_chain`，第 10n 批；之前落到缺省理由「这个镜像上没有它判的对象」）。
+const INSTANCE_TABLE_ROWS_NOT_JUDGED_AS_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE: &str =
+    "最新根的实例表那一片解不开（I-3.14 不成立）、行没收全：没有行可判";
+
+/// 最新根的实例表那一片解不开的四张镜像共用的判法：判红的恰好是 [`VIOLATED_ONCE_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE`]，
+/// I-3.8 报不适用、理由是 [`INSTANCE_TABLE_ROWS_NOT_JUDGED_AS_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE`]。
+fn assert_the_newest_instance_table_page_does_not_decode(image: &MemoryPool, what: &str) {
+    let verdicts = check_pool_image(image);
+    assert_eq!(
+        violated_invariants(&verdicts),
+        VIOLATED_ONCE_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE,
+        "{what}：判红的该是 I-3.14 与 C 进候选集带出来的两条：{verdicts:?}"
+    );
+    assert_eq!(
+        verdict(image, "I-3.8"),
+        InvariantVerdict::NotApplicable(
+            INSTANCE_TABLE_ROWS_NOT_JUDGED_AS_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE
+        ),
+        "{what}：最新根的实例表第 0 片解不开、一行都没收，I-3.8 报不适用并说明为什么"
+    );
+}
+
+/// I-3.14（实例表每片记录形态合法，`records/2026-09-30-里程碑三开工第一轮.md`「用户 2026-10-06 弹窗定实例表 kind 那两问」） 的坏镜像，造法照攻方 N5-a：
+/// 最新根的实例表在链指针记录之前多插一条 kind 2，重封、补到根槽。实现读到它整张表解不开（恢复 `Failed`、可写挂载拒）；
+/// 池级 checker 同一个读法（第 10k 批）：I-3.14 红（种类那一格），这一片的行不收、走不到末片，被抛弃的 C 滤不掉、进了回退候选集，
+/// 它那一版的单元连带红 I-3.1、I-9.10（I-9.29 报部分没判，上面那一条用例钉着）；引用集合那一遍同记数不全（第 10n 批，之前多红 I-3.9）；
+/// I-3.8 一行都没收、报不适用并给理由（第 10n 批）。第 10j 批时这一片照收、只红 I-3.14。
+/// 判别力（`crates/mutations.tsv` 里点名这条的行）：种类那一格恒判成立时只红那两条；这一片照收照走时只红 I-3.14；
+/// 引用集合那一遍照记全时多红 I-3.9；I-3.8 落回缺省理由时不适用的理由对不上。
+#[test]
+fn a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set(
+) {
+    let (image, instance_table_slot) =
+        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
+            "known-bad-instance-table-record-form-unregistered-kind",
+            |bytes| insert_a_record_of_an_unregistered_kind_before_the_chain_record(bytes),
+            true,
+        );
+    assert_the_newest_instance_table_page_does_not_decode(
+        &image,
+        &format!("最新根的实例表（槽 {instance_table_slot}）链指针记录之前多一条 kind 2"),
+    );
+}
+
+/// I-3.14 的坏镜像（定案原话里的「片中间多一条链指针」）：最新根的实例表在链指针记录之前多插一条一模一样的链指针记录（kind 1、无下一片），重封、补到根槽。
+/// 最后一条仍是合法的链指针记录，I-3.14 红的是「链指针记录只许在一片末尾」那一格；这一片解不开，判红的另两条与 I-3.8 的理由同上一条用例（第 10k 批、第 10n 批）。
+/// 判别力：那一格恒判成立时只红那两条；这一片照收照走时只红 I-3.14（改之前的 checker 两臂里，中间那条 kind 1 被后面那条盖掉）。
+#[test]
+fn a_second_chain_record_in_the_middle_of_the_newest_instance_table_page_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set(
+) {
+    let (image, instance_table_slot) =
+        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
+            "known-bad-instance-table-record-form-chain-record-in-the-middle",
             |bytes| {
                 let record_count = usize::from(u16::from_le_bytes([
                     bytes[PACKED_UNIT_RECORD_COUNT_OFFSET],
@@ -4062,32 +4213,43 @@
                 let chain_record = bytes
                     [chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES]
                     .to_vec();
-                let mut record_of_an_unregistered_kind = vec![0u8; INSTANCE_TABLE_RECORD_BYTES];
-                record_of_an_unregistered_kind[0] = INSTANCE_TABLE_RECORD_KIND_UNREGISTERED;
-                bytes[chain_record_start..chain_record_start + INSTANCE_TABLE_RECORD_BYTES]
-                    .copy_from_slice(&record_of_an_unregistered_kind);
-                bytes[chain_record_start + INSTANCE_TABLE_RECORD_BYTES
-                    ..chain_record_start + 2 * INSTANCE_TABLE_RECORD_BYTES]
-                    .copy_from_slice(&chain_record);
-                let new_record_count = record_count + 1;
-                set_u16(
+                insert_a_record_before_the_chain_record_of_the_instance_table_page(
                     bytes,
-                    PACKED_UNIT_RECORD_COUNT_OFFSET,
-                    u16::try_from(new_record_count).expect("一片装得下"),
+                    &chain_record,
                 );
-                set_u16(
-                    bytes,
-                    PACKED_UNIT_DECLARED_LENGTH_OFFSET,
-                    u16::try_from(new_record_count * INSTANCE_TABLE_RECORD_BYTES)
-                        .expect("一片装得下"),
+            },
+            true,
+        );
+    assert_the_newest_instance_table_page_does_not_decode(
+        &image,
+        &format!("最新根的实例表（槽 {instance_table_slot}）片中间多一条链指针记录"),
+    );
+}
+
+/// I-3.14 的坏镜像（定案原话里的「kind 0 行 flags 非 0」）：最新根的实例表第一行的 flags 写成退役的 bit0，重封、补到根槽。
+/// 实现读行时拒收（`InstanceRow::parse` 判 flags 非 0）；池级 checker 红 I-3.14 的 flags 那一格，这一片解不开，判红的另两条与 I-3.8 的理由同上（第 10k 批、第 10n 批）。
+/// 判别力：那一格恒判成立时只红那两条；这一片照收照走时只红 I-3.14（改之前 checker 哪里都不读 flags）。
+#[test]
+fn a_row_whose_flags_are_not_zero_in_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set(
+) {
+    let (image, instance_table_slot) =
+        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
+            "known-bad-instance-table-record-form-row-flags",
+            |bytes| {
+                let first_row_flags =
+                    INSTANCE_TABLE_FIRST_ROW_START + INSTANCE_TABLE_ROW_FLAGS_OFFSET;
+                assert_eq!(
+                    (bytes[INSTANCE_TABLE_FIRST_ROW_START], bytes[first_row_flags]),
+                    (0, 0),
+                    "实例 3 的实例表第一条是行、flags 是 0"
                 );
+                bytes[first_row_flags] = INSTANCE_TABLE_ROW_FLAGS_RETIRED_BIT;
             },
             true,
         );
-    assert_eq!(
-        verdict(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER),
-        InvariantVerdict::Holds,
-        "最新根的实例表（槽 {instance_table_slot}）链指针记录之前多一条 kind 2：主走读收下了全部行、走到了末片，被抛弃的 C 滤掉了，I-9.29 要真被评估过且成立"
+    assert_the_newest_instance_table_page_does_not_decode(
+        &image,
+        &format!("最新根的实例表（槽 {instance_table_slot}）第一行 flags 写 1"),
     );
 }
 
@@ -4119,9 +4281,11 @@
     );
 }
 
-/// 攻方 N5-b（出处同上）：最新根的实例表链指针记录「有无下一片」写 2，重封、补到根槽。主走读收下这一片的行（I-3.8 红）就停，不知道后面还有没有片，
-/// 走不到末片：I-9.29 报部分没判，不报成立（这一片其实就是整张表，可镜像上分不出「只有一片」与「后面还有片」）。
-/// 判别力：把「链指针记录不合法」当成走到了末片时，候选集按这一片的行滤掉 C，I-9.29 报成立。
+/// 攻方 N5-b（出处同上）：最新根的实例表链指针记录「有无下一片」写 2，重封、补到根槽。I-3.14 红，这一片解不开（第 10k 批起行不收，
+/// 第 10j 批时照收这一片的行），不知道后面还有没有片，走不到末片：I-9.29 报部分没判，不报成立（这一片其实就是整张表，可镜像上分不出「只有一片」与
+/// 「后面还有片」）；C 进了候选集，判红的与另三形同是 [`VIOLATED_ONCE_THE_NEWEST_INSTANCE_TABLE_PAGE_DOES_NOT_DECODE`]，I-3.8 的理由同上
+/// （第 10k 批时另三形多红 I-3.9：引用集合那一遍只在这一形记数不全；第 10n 批四形同记）。
+/// 判别力：把「链指针记录不合法」当成走到了末片时 I-9.29 不报部分没判；照收这一片的行时 C 被滤掉、只红 I-3.14。
 #[test]
 fn a_malformed_chain_record_on_the_newest_instance_table_leaves_the_one_birth_invariant_partially_judged(
 ) {
@@ -4139,10 +4303,9 @@
             },
             true,
         );
-    let verdicts = check_pool_image(&image);
-    assert!(
-        violated_invariants(&verdicts).contains(&"I-3.8"),
-        "最新根的实例表（槽 {instance_table_slot}）最后一条不是合法的链指针记录：I-3.8 红：{verdicts:?}"
+    assert_the_newest_instance_table_page_does_not_decode(
+        &image,
+        &format!("最新根的实例表（槽 {instance_table_slot}）最后一条不是合法的链指针记录"),
     );
     match verdict(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER) {
         InvariantVerdict::PartiallyJudged(unjudged) => assert!(
@@ -4157,6 +4320,263 @@
     }
 }
 
+/// 根环里 (checkpoint_txg, 实例) 最大的那条根（择根序）的根记录 512 字节。
+fn newest_root_record_bytes(image: &MemoryPool) -> Vec<u8> {
+    root_slots()
+        .into_iter()
+        .map(|(device, offset)| read(image, device, offset, 512))
+        .filter(|root| &root[..4] == b"SFSR")
+        .max_by_key(|root| {
+            (
+                get_u64(root, ROOT_RECORD_CHECKPOINT_TXG_OFFSET),
+                get_u32(root, ROOT_RECORD_INSTANCE_OFFSET),
+            )
+        })
+        .expect("根环里有根")
+}
+
+/// 实例表一片（整个单元的字节，重封归调用方）的载荷换成 `records`、容器号写 `page_index`：记录数与声明长度跟着改，记录之后清零。
+fn rewrite_the_records_of_an_instance_table_page(
+    bytes: &mut [u8],
+    records: &[Vec<u8>],
+    page_index: u64,
+) {
+    bytes[INSTANCE_TABLE_FIRST_ROW_START..].fill(0);
+    for (position, record) in records.iter().enumerate() {
+        let start = INSTANCE_TABLE_FIRST_ROW_START + position * INSTANCE_TABLE_RECORD_BYTES;
+        bytes[start..start + INSTANCE_TABLE_RECORD_BYTES].copy_from_slice(record);
+    }
+    set_u16(
+        bytes,
+        PACKED_UNIT_RECORD_COUNT_OFFSET,
+        u16::try_from(records.len()).expect("一片装得下"),
+    );
+    set_u16(
+        bytes,
+        PACKED_UNIT_DECLARED_LENGTH_OFFSET,
+        u16::try_from(records.len() * INSTANCE_TABLE_RECORD_BYTES).expect("一片装得下"),
+    );
+    set_u64(bytes, PACKED_UNIT_CONTAINER_NUMBER_OFFSET, page_index);
+}
+
+/// 实例表行（kind 0）里实例代号（4 字节）与 T（8 字节）的偏移（D18（块里携带什么信息） 已定项 11：kind 1 + 实例代号 4 + T 8 + W 8 + flags 1）。
+const INSTANCE_TABLE_ROW_INSTANCE_OFFSET: usize = 1;
+const INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET: usize = 5;
+/// 被崩溃恢复抛弃的 C 那条根的实例代号（[`image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth`]：
+/// A、B 是实例 1，重开取号 2 之后写 C，恢复落到 C 之前那条根之后是实例 3）。
+const INSTANCE_OF_THE_ABANDONED_ROOT: u32 = 2;
+/// 第 1 片写在哪个槽（两盘同槽）：4 GiB 的盘上远在这段历史写到的单元之后，没人用。
+const SLOT_OF_THE_SECOND_INSTANCE_TABLE_PAGE: u64 = 200_000;
+/// 一条位置条目里整单元校验和（4 字节）的偏移（盘 4 + 槽 6）。
+const LOCATION_ENTRY_CHECKSUM_OFFSET: usize = 10;
+/// 链指针记录里位置指针的偏移（kind 1 + 有无下一片 1）与「有下一片」的值。
+const CHAIN_RECORD_POINTER_OFFSET: usize = 2;
+const CHAIN_RECORD_HAS_NEXT_PAGE: u8 = 1;
+
+/// 第 1 片是哪一形：两份完好、指着它的指针盘 0 那条位置条目指回第 0 片（攻方 H-b），或盘 0 那一份坏（攻方 P2「第 1 片盘 0 那份坏」，H-b 的对照）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum SecondInstanceTablePageShape {
+    FirstLocationOfItsPointerNamesTheFirstPage,
+    CopyOnDeviceZeroBroken,
+}
+
+/// 攻方 H-b 与它的对照的底子（`research/prompts/m3-batch25-code-r1-opus-output.md` 第 1.3 节；那一轮的 P2）：
+/// [`image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth`] 那一份上，最新根（实例 3）的实例表改成两片——
+/// 第 0 片留下别的行，被抛弃的 C 那个实例（实例 2）的行挪到第 1 片。第 1 片照第 0 片的头造（容器号 1、重封），两盘写在
+/// [`SLOT_OF_THE_SECOND_INSTANCE_TABLE_PAGE`]；第 0 片末尾的链指针记录指着它（指针头照根记录里的实例表指针，两条位置条目是第 1 片的槽与整单元校验和），
+/// 再照 `shape` 改；第 0 片重封、整单元校验和补到根槽。第 1 片的槽不在任何一版的分配记录里（I-3.1、I-3.11 那几条照红，这里不钉）。
+fn image_with_the_newest_instance_table_in_two_pages_and_the_abandoned_instance_row_on_the_second(
+    tag: &str,
+    shape: SecondInstanceTablePageShape,
+) -> MemoryPool {
+    let mut image =
+        image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth(
+            tag,
+        );
+    let first_page_slot = instance_table_slot_of_the_newest_root(&image);
+    let first_page = read(&image, 0, first_page_slot * SLOT, INSTANCE_TABLE_UNIT_BYTES);
+    let record_count = usize::from(u16::from_le_bytes([
+        first_page[PACKED_UNIT_RECORD_COUNT_OFFSET],
+        first_page[PACKED_UNIT_RECORD_COUNT_OFFSET + 1],
+    ]));
+    let records: Vec<Vec<u8>> = (0..record_count)
+        .map(|position| {
+            let start = INSTANCE_TABLE_FIRST_ROW_START + position * INSTANCE_TABLE_RECORD_BYTES;
+            first_page[start..start + INSTANCE_TABLE_RECORD_BYTES].to_vec()
+        })
+        .collect();
+    let (chain_record, rows) = records.split_last().expect("实例表一片里至少有链指针记录");
+    assert_eq!(
+        (
+            chain_record[0],
+            chain_record[CHAIN_RECORD_HAS_NEXT_PAGE_OFFSET]
+        ),
+        (INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD, 0),
+        "实例 3 的实例表只有一片"
+    );
+    let (rows_of_the_abandoned_instance, other_rows): (Vec<Vec<u8>>, Vec<Vec<u8>>) =
+        rows.iter().cloned().partition(|row| {
+            get_u32(row, INSTANCE_TABLE_ROW_INSTANCE_OFFSET) == INSTANCE_OF_THE_ABANDONED_ROOT
+        });
+    assert_eq!(
+        rows_of_the_abandoned_instance.len(),
+        1,
+        "实例 3 的实例表里被抛弃的 C 那个实例恰好一行"
+    );
+    let mut second_page = first_page.clone();
+    rewrite_the_records_of_an_instance_table_page(
+        &mut second_page,
+        &[
+            rows_of_the_abandoned_instance[0].clone(),
+            chain_record.clone(),
+        ],
+        1,
+    );
+    reseal_unit(&mut second_page);
+    let second_page_checksum = crc32_castagnoli(&second_page);
+    for device in DEVICES {
+        let mut copy = second_page.clone();
+        if device == 0 && shape == SecondInstanceTablePageShape::CopyOnDeviceZeroBroken {
+            copy[INSTANCE_TABLE_UNIT_BYTES - 100] ^= 0xff;
+        }
+        write(
+            &mut image,
+            device,
+            SLOT_OF_THE_SECOND_INSTANCE_TABLE_PAGE * SLOT,
+            &copy,
+        );
+    }
+    let root_record = newest_root_record_bytes(&image);
+    let mut pointer = root_record[ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET
+        ..ROOT_RECORD_INSTANCE_TABLE_POINTER_OFFSET + NODE_POINTER_BYTES]
+        .to_vec();
+    for (location_index, device) in DEVICES.iter().enumerate() {
+        let location = POINTER_FIRST_LOCATION_OFFSET + location_index * POINTER_LOCATION_BYTES;
+        let slot = match (shape, *device) {
+            (SecondInstanceTablePageShape::FirstLocationOfItsPointerNamesTheFirstPage, 0) => {
+                first_page_slot
+            }
+            (SecondInstanceTablePageShape::FirstLocationOfItsPointerNamesTheFirstPage, _)
+            | (SecondInstanceTablePageShape::CopyOnDeviceZeroBroken, _) => {
+                SLOT_OF_THE_SECOND_INSTANCE_TABLE_PAGE
+            }
+        };
+        pointer[location..location + 4].copy_from_slice(&device.to_le_bytes());
+        pointer[location + LOCATION_ENTRY_SLOT_OFFSET..location + LOCATION_ENTRY_SLOT_OFFSET + 6]
+            .copy_from_slice(&slot.to_le_bytes()[..6]);
+        pointer[location + LOCATION_ENTRY_CHECKSUM_OFFSET
+            ..location + LOCATION_ENTRY_CHECKSUM_OFFSET + 4]
+            .copy_from_slice(&second_page_checksum.to_le_bytes());
+    }
+    let mut chain_record_to_the_second_page = vec![0u8; INSTANCE_TABLE_RECORD_BYTES];
+    chain_record_to_the_second_page[0] = INSTANCE_TABLE_RECORD_KIND_CHAIN_RECORD;
+    chain_record_to_the_second_page[CHAIN_RECORD_HAS_NEXT_PAGE_OFFSET] = CHAIN_RECORD_HAS_NEXT_PAGE;
+    chain_record_to_the_second_page[CHAIN_RECORD_POINTER_OFFSET..].copy_from_slice(&pointer);
+    let mut first_page_records = other_rows;
+    first_page_records.push(chain_record_to_the_second_page);
+    mutate_unit_of(
+        &mut image,
+        &[(first_page_slot, INSTANCE_TABLE_UNIT_BYTES)],
+        first_page_slot,
+        |bytes| rewrite_the_records_of_an_instance_table_page(bytes, &first_page_records, 0),
+        true,
+    );
+    image
+}
+
+/// 攻方 H-b（`research/prompts/m3-batch25-code-r1-opus-output.md` 第 1.3 节）：最新根的实例表两片、被抛弃的 C 那个实例的行在第 1 片，
+/// 指着第 1 片的指针盘 0 那条位置条目指回第 0 片（校验和仍是第 1 片的）、盘 1 那条完好（底子见
+/// [`image_with_the_newest_instance_table_in_two_pages_and_the_abandoned_instance_row_on_the_second`]）。core 读得出整张表；主走读读第 1 片时
+/// 盘 0 那份对不上（I-2.1）、盘 1 那份读回、自证过，「走过」按读回的那一份的落点认（第 10k 批）：第 1 片照走、行收全、走到末片，C 滤掉，
+/// I-9.29 真被评估过且成立。判红的与对照（同一份底子、第 1 片盘 0 那一份坏）相同，只多 I-5.1：盘 0 那条位置条目与根记录里的实例表指针
+/// 占着同一个落点，是两处引用同一段物理范围。修前（按第 0 条位置条目认走过）第 1 片被当成走过、它的行丢了：I-9.29 部分没判，另多红 I-3.9。
+/// 判别力：按第 0 条位置条目认走过（`crates/mutations.tsv` 里点名这条的行）时 I-9.29 报部分没判。
+#[test]
+fn a_second_instance_table_page_whose_pointer_names_the_first_page_on_device_zero_is_walked_from_the_copy_read_back(
+) {
+    let control = image_with_the_newest_instance_table_in_two_pages_and_the_abandoned_instance_row_on_the_second(
+        "known-bad-second-instance-table-page-copy-on-device-zero-broken",
+        SecondInstanceTablePageShape::CopyOnDeviceZeroBroken,
+    );
+    let control_verdicts = check_pool_image(&control);
+    let image = image_with_the_newest_instance_table_in_two_pages_and_the_abandoned_instance_row_on_the_second(
+        "known-bad-second-instance-table-page-pointer-names-the-first-page-on-device-zero",
+        SecondInstanceTablePageShape::FirstLocationOfItsPointerNamesTheFirstPage,
+    );
+    let verdicts = check_pool_image(&image);
+    assert_eq!(
+        verdict(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER),
+        InvariantVerdict::Holds,
+        "第 1 片按读回的盘 1 那一份走：行收全、走到末片，C 滤掉，I-9.29 真被评估过且成立：{verdicts:?}"
+    );
+    let mut expected = violated_invariants(&control_verdicts);
+    expected.push("I-5.1");
+    expected.sort_unstable();
+    let mut violated = violated_invariants(&verdicts);
+    violated.sort_unstable();
+    assert_eq!(
+        violated,
+        expected,
+        "判红的与对照（第 1 片盘 0 那一份坏：{:?}）相同、只多 I-5.1：{verdicts:?}",
+        violated_invariants(&control_verdicts)
+    );
+}
+
+/// 只读不判的读法（`singlefs_checker::walk::InstanceTableOfRootRecord::read`，I-3.12、I-7.9 与记录核对器读一条根自己的实例表）与主走读、
+/// 与实现同一个读法：行的 flags 不是 0 的一片解不开（D18（块里携带什么信息） 已定项 11「第一版恒 0、非 0 拒收」，第 10k 批）。
+/// 镜像：最新根的实例表第一行 flags 写成退役的 bit0、它的 T 抬到不低于最新根的 txg，重封、补到根槽。最新根那张表这条读法读不出，
+/// I-3.12（实例表行 T 低于引用它的根的 txg）不判它、不红；第 10k 批之前这条读法照收这一行，I-3.12 红。
+/// 判别力：这条读法不看 flags（`crates/mutations.tsv` 里点名这条的行）时读得出、I-3.12 红。
+#[test]
+fn a_row_whose_flags_are_not_zero_leaves_the_newest_instance_table_unread_by_the_reader_without_judging(
+) {
+    let clean =
+        image_after_a_recovery_abandoned_the_third_root_whose_inode_record_names_another_object_birth(
+            "known-bad-instance-table-row-flags-unread-by-the-reader-without-judging-clean",
+        );
+    assert!(
+        InstanceTableOfRootRecord::read(&clean, &newest_root_record_bytes(&clean)).is_some(),
+        "前提：改之前最新根的实例表这条读法读得出"
+    );
+    let newest_root_txg = get_u64(
+        &newest_root_record_bytes(&clean),
+        ROOT_RECORD_CHECKPOINT_TXG_OFFSET,
+    );
+    let (image, instance_table_slot) =
+        image_with_an_abandoned_root_naming_another_object_birth_and_the_newest_instance_table_changed(
+            "known-bad-instance-table-row-flags-unread-by-the-reader-without-judging",
+            |bytes| {
+                let first_row = INSTANCE_TABLE_FIRST_ROW_START;
+                assert_eq!(
+                    (bytes[first_row], bytes[first_row + INSTANCE_TABLE_ROW_FLAGS_OFFSET]),
+                    (0, 0),
+                    "实例 3 的实例表第一条是行、flags 是 0"
+                );
+                bytes[first_row + INSTANCE_TABLE_ROW_FLAGS_OFFSET] =
+                    INSTANCE_TABLE_ROW_FLAGS_RETIRED_BIT;
+                set_u64(
+                    bytes,
+                    first_row + INSTANCE_TABLE_ROW_PUBLISHED_TXG_OFFSET,
+                    newest_root_txg,
+                );
+            },
+            true,
+        );
+    assert_eq!(
+        InstanceTableOfRootRecord::read(&image, &newest_root_record_bytes(&image)),
+        None,
+        "最新根的实例表（槽 {instance_table_slot}）第一行 flags 写 1：只读不判的读法读不出"
+    );
+    let found = verdict(
+        &image,
+        INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
+    );
+    assert!(
+        !matches!(found, InvariantVerdict::Violated(_)),
+        "那一行读不出、I-3.12 不判它（T {newest_root_txg} 不低于最新根的 txg 也不红），实际 {found:?}"
+    );
+}
+
 /// 发布 B 写出的数据单元：它的分配记录在 B 那一版分配记录树的两片叶（`ALLOCATION_LEAVES_AFTER_OVERWRITE`）里，未释放、分配代 4，
 /// 单元头里的诞生代号也是 4（落点由 `overwrite_in_one_instance.rs` 的验收钉住）。
 const DATA_UNIT_AFTER_OVERWRITE: u64 = 50182;
```

## `crates/singlefs-harness/tests/instance_table_chain_read.rs`

```diff
--- a/crates/singlefs-harness/tests/instance_table_chain_read.rs
+++ b/crates/singlefs-harness/tests/instance_table_chain_read.rs
@@ -12,7 +12,7 @@
 mod common;
 
 use common::{disk_snapshot, memory_pool_of_sparse_devices, parameters, DiskSnapshot, IMAGE_BYTES};
-use singlefs_checker::image::InvariantVerdict;
+use singlefs_checker::image::{InvariantVerdict, INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED};
 use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
@@ -530,14 +530,21 @@
     );
 }
 
-/// 合法的两片链上池级 checker 一条都不红：第二片的身份、校验和、位置条目次序、链指针记录都判过（I-1.1、I-2.1、I-2.5、I-3.8
+/// 合法的两片链上池级 checker 一条都不红：第二片的身份、校验和、位置条目次序、记录形态与链指针记录都判过（I-1.1、I-2.1、I-2.5、I-3.14
 /// 判成成立，不是「不适用」），两片的行接起来判 I-3.8。
 #[test]
 fn the_pool_checker_walks_both_pages_of_a_valid_chain_and_every_invariant_holds() {
     let (devices, _stream, _rows, _chain) = pool_with_a_valid_two_page_chain();
     let verdicts = verdicts(&devices);
     assert_eq!(violated(&verdicts), Vec::<&str>::new(), "{verdicts:?}");
-    for invariant in ["I-1.1", "I-2.1", "I-2.5", "I-3.8", "I-1.7"] {
+    for invariant in [
+        "I-1.1",
+        "I-2.1",
+        "I-2.5",
+        "I-3.8",
+        "I-1.7",
+        INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED,
+    ] {
         assert_eq!(
             verdict_of(&verdicts, invariant),
             InvariantVerdict::Holds,
@@ -653,7 +660,7 @@
 }
 
 /// 链指针记录自相矛盾（坏镜像）：「有无下一片」写 0 而位置指针没清零（条款：无下一片时清零占位），
-/// 与「有无下一片」写 2（字段表里只有有、无两个值）。两样都是 I-3.8 那条链指针记录的判定红；
+/// 与「有无下一片」写 2（字段表里只有有、无两个值）。两样都是 I-3.14（实例表每片记录形态合法） 那条链指针记录的判定红；
 /// 链在第 0 片就停了，第二片没人引用，别的不红。
 #[test]
 fn contradictory_chain_record_reddens_only_the_instance_table_invariant() {
@@ -668,7 +675,11 @@
             &rows[2..],
         );
         let verdicts = verdicts(&devices);
-        assert_eq!(violated(&verdicts), vec!["I-3.8"], "{what}：{verdicts:?}");
+        assert_eq!(
+            violated(&verdicts),
+            vec![INSTANCE_TABLE_PAGE_RECORDS_ARE_WELL_FORMED],
+            "{what}：{verdicts:?}"
+        );
         assert_eq!(
             instance_table_of_root(&devices, &newest_root(&devices)),
             None,
```

## `crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs`

```diff
--- a/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs
+++ b/crates/singlefs-harness/tests/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs
@@ -21,13 +21,18 @@
 //! 按槽并两遍的用例（攻方第二次抽样 `research/prompts/m3-batch10-code-r2-opus-s2-output.md` K2-2 那组历史）：384 槽两块盘，实例 1 第一个文件、
 //! 覆盖写一次，崩了再挂（实例 2）覆盖写一次得 C，再照 `tests/common` 的 `abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`
 //! 那一形抛弃 C（实例 3），进程退出；再可写挂载时第一遍（第 6 次读）盘 1 每个根环槽报读错，重读（第 7 次读）照点名的读坏。
+//!
+//! 两遍里只有一遍自证过的槽在抬 F 那一串（准入算上限、抬 F 重算影子账与回收门槛、卸载）读坏、重读仍坏时拿根环表记着的那条根顶上
+//! （用户 2026-10-06 定 T1 甲，判决 `research/prompts/m3-batch10i-code-r1-main-verification.md` 第六节）的用例照第 10i 批代码轮攻方 K1-1、K1-2、K1-3
+//! 那三形（`research/prompts/m3-batch10i-code-r1-opus-output.md`）：同一组历史（单元区 384 槽、实例 1 覆盖写 3 次），盘留着读坏接着发改动，
+//! 这个进程写过的槽不再读坏（攻方原型同一个做法）。
 
 mod common_admission;
 
 use common_admission::{
     checker_violations_on, plain_devices_on,
     start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
-    start_plain, PoolUnderTest, FIRST_WRITE_TIME_SECONDS, OVERWRITE_BYTES,
+    start_plain, DeviceOverASparseImage, PoolUnderTest, FIRST_WRITE_TIME_SECONDS, OVERWRITE_BYTES,
 };
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber, SlotNumber,
@@ -40,8 +45,9 @@
 use singlefs_core::lineage_tree::LineageContainerCapacity;
 use singlefs_core::mount::{
     mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread,
-    mount_writable_with_test_only_switches_and_deletion_switches, unmount, BeforeTheOneReread,
-    MountError, Mounted, RootRingOfTheRebuiltAllocatorRead, ShadowLedger,
+    mount_writable_with_test_only_switches_and_deletion_switches,
+    ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by, unmount,
+    BeforeTheOneReread, MountError, Mounted, RootRingOfTheRebuiltAllocatorRead, ShadowLedger,
 };
 use singlefs_core::mounted_read::{mount_read_only, open_pool_for_read};
 use singlefs_core::mounted_session::MountedSession;
@@ -65,7 +71,7 @@
     FaultSchedule, InjectedFault, SharedFaultPlan,
 };
 use singlefs_harness::history::HistoryDeviceWidth;
-use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
 use std::cell::RefCell;
 use std::collections::BTreeMap;
 use std::rc::Rc;
@@ -150,6 +156,15 @@
         fault: RootRingSlotReadFault::ReadsBackGarbage,
     };
 
+/// 这个进程写过一个根环槽之后，点名读坏它的那几次还照不照坏。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum FaultsOnARootRingSlotThisProcessWrote {
+    /// 照坏（坏道写不好）。
+    KeepFailing,
+    /// 不再坏（写把坏道换掉了；攻方原型 `research/prompts/m3-batch10i-code-r1-opus-model/main.rs` 的 `FaultDevice` 同一个做法）。
+    StopFailing,
+}
+
 /// 根环每个槽被读的次数与点名读坏的那几次（几块盘共用一份）。
 struct RootRingReadsState {
     slot_spacing_in_bytes: u64,
@@ -161,6 +176,10 @@
     slots_failing_from_a_read_onward: BTreeMap<RootRingSlot, (u64, RootRingSlotReadFault)>,
     reads_of_each_slot: BTreeMap<RootRingSlot, u64>,
     faulted_reads: u64,
+    faults_on_a_slot_this_process_wrote: FaultsOnARootRingSlotThisProcessWrote,
+    slots_this_process_wrote: std::collections::BTreeSet<RootRingSlot>,
+    /// 每一次根环槽读（槽，这个槽第几次被读）按发生的次序，中间每一段写记成一项 `None`（连着的写只记一项）。
+    reads_and_writes_in_order: Vec<Option<(RootRingSlot, u64)>>,
 }
 
 #[derive(Clone)]
@@ -187,6 +206,15 @@
             *reads += 1;
             *reads
         };
+        state
+            .reads_and_writes_in_order
+            .push(Some((ring_slot, reads_of_this_slot)));
+        if state.faults_on_a_slot_this_process_wrote
+            == FaultsOnARootRingSlotThisProcessWrote::StopFailing
+            && state.slots_this_process_wrote.contains(&ring_slot)
+        {
+            return None;
+        }
         let fault = state
             .faulted_reads_of_each_slot
             .get(&(ring_slot, reads_of_this_slot))
@@ -209,6 +237,28 @@
     fn faulted_reads(&self) -> u64 {
         self.0.borrow().faulted_reads
     }
+
+    /// 记一次写：写盖到的根环槽记成这个进程写过的；读写次序里记一项写（紧接着上一项写的不重复记）。
+    fn note_a_write(&self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: u64) {
+        let mut state = self.0.borrow_mut();
+        let spacing = state.slot_spacing_in_bytes;
+        let written: Vec<RootRingSlot> = state
+            .slots
+            .iter()
+            .filter(|(_, slot_device, start)| {
+                *slot_device == device && offset.0 < *start + spacing && offset.0 + length > *start
+            })
+            .map(|(ring_slot, _, _)| *ring_slot)
+            .collect();
+        state.slots_this_process_wrote.extend(written);
+        if state.reads_and_writes_in_order.last() != Some(&None) {
+            state.reads_and_writes_in_order.push(None);
+        }
+    }
+
+    fn reads_and_writes_in_order(&self) -> Vec<Option<(RootRingSlot, u64)>> {
+        self.0.borrow().reads_and_writes_in_order.clone()
+    }
 }
 
 /// 读的时候按 [`SharedRootRingReads`] 让点名的那几次根环槽读读坏的盘；写、写零、屏障原样交给里面那块盘。
@@ -258,6 +308,11 @@
         bytes: &[u8],
         durability: WriteDurability,
     ) -> Result<(), BlockDeviceError> {
+        self.root_ring_reads.note_a_write(
+            self.identity,
+            offset,
+            u64::try_from(bytes.len()).expect("写的长度装得进 u64"),
+        );
         self.inner.write_at(offset, bytes, durability)
     }
 
@@ -266,6 +321,8 @@
         offset: DeviceOffsetInBytes,
         length: u64,
     ) -> Result<(), BlockDeviceError> {
+        self.root_ring_reads
+            .note_a_write(self.identity, offset, length);
         self.inner.write_zeroes_at(offset, length)
     }
 
@@ -765,6 +822,21 @@
     faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
     slots_failing_from_a_read_onward: BTreeMap<RootRingSlot, (u64, RootRingSlotReadFault)>,
 ) -> SharedRootRingReads {
+    root_ring_reads_failing_with(
+        image,
+        faulted_reads_of_each_slot,
+        slots_failing_from_a_read_onward,
+        FaultsOnARootRingSlotThisProcessWrote::KeepFailing,
+    )
+}
+
+/// 同 [`root_ring_reads_failing_also_from_a_read_onward`]，另按 `faults_on_a_slot_this_process_wrote` 定写过的槽还坏不坏。
+fn root_ring_reads_failing_with(
+    image: &MemoryPool,
+    faulted_reads_of_each_slot: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault>,
+    slots_failing_from_a_read_onward: BTreeMap<RootRingSlot, (u64, RootRingSlotReadFault)>,
+    faults_on_a_slot_this_process_wrote: FaultsOnARootRingSlotThisProcessWrote,
+) -> SharedRootRingReads {
     let system_configuration = choose_system_configuration(image).expect("系统配置");
     let immutable = system_configuration.immutable;
     let slots = every_root_ring_slot(&immutable.sizes)
@@ -784,6 +856,9 @@
         slots_failing_from_a_read_onward,
         reads_of_each_slot: BTreeMap::new(),
         faulted_reads: 0,
+        faults_on_a_slot_this_process_wrote,
+        slots_this_process_wrote: std::collections::BTreeSet::new(),
+        reads_and_writes_in_order: Vec::new(),
     })))
 }
 
@@ -1224,7 +1299,8 @@
 /// `narrow-384-20-2` 上照 [`ReadsOfTheSlotFailingAfterTheRebuildPass`] 再挂时，盘 0 最新那条根住的槽第几次被读是写行与暖机之后推的
 /// 第一次抬 F 算上限读根环那一遍（`mount::roots_of_the_ring_for_the_ceiling`）：现量的（第 10i 批在副本上逐次打出读它的调用栈：
 /// 1–5 读阶段，6、7 重建分配器两遍，8、9 取号判最大实例代号，10–15 先写系统配置那几步算生效 F，16 算上限）。挂载里读根环的次序变了，
-/// 这个数跟着变；「第10i批 S1 抬 F 算上限读它读坏一次就拒、不重读」那一行变异就不再红，门禁复跑变异表时报出来。
+/// 这个数跟着变：瞬时那条用例另断言这一次读落在暖机最后一段写之后的第一遍读根环里、紧接着当场重读、写之前这一槽还被读过
+/// （[`read_opens_the_floor_raise_after_the_last_write_and_is_reread_at_once`]），读序一变那一项就红。
 const READ_OF_THE_SLOT_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE: u64 = 16;
 
 /// S1 那一形再挂之后看到的那几样（各用例自己断言）。
@@ -1247,19 +1323,73 @@
     lone_survivor_reads_what_both_devices_read_before_the_remount: [bool; 2],
     /// 丢盘 0、丢盘 1 之后，挂载之前环里还在、不低于 F 的每条自证根在剩下那块盘上整读与挂载之前相同。
     lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [bool; 2],
+    /// 那一槽第 [`READ_OF_THE_SLOT_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE`] 次读落在它之前最后一段写之后的第一遍读根环里、
+    /// 紧接着是同一槽的重读（[`read_opens_the_floor_raise_after_the_last_write_and_is_reread_at_once`]）。
+    the_named_read_opens_the_floor_raise_after_the_last_write_and_is_reread_at_once: bool,
+}
+
+/// 读写次序里（`ring_slot`，第 `read` 次读）那一次读是不是：① 落在它之前最后一段写之后的第一遍读根环里——那一段写之后的根环槽读照
+/// `every_root_ring_slot` 的次序从第一个槽起、一个槽一次地读到它；② 紧接着的一次读是同一槽的重读；③ 重读之后、下一段写之前这一槽还被读过。
+/// 挂载里暖机之后推的抬 F，第一遍读根环是准入算上限那一遍（`mount::roots_of_the_ring_for_the_ceiling`，这一类槽读坏当场重读），
+/// 之后写之前还有再算一次上限、重算影子账那几遍（③）。别的几遍对不上：先写系统配置那几步算生效 F 的那一遍也是上一次发布的写之后第一遍、
+/// 读坏也当场重读，紧接着却是这次发布的写（③ 不成立）；抬 F 那一串里第二次算上限、重算影子账那几遍不是写之后的第一遍（① 不成立）。
+fn read_opens_the_floor_raise_after_the_last_write_and_is_reread_at_once(
+    reads_and_writes_in_order: &[Option<(RootRingSlot, u64)>],
+    ring_slots_in_order: &[RootRingSlot],
+    ring_slot: RootRingSlot,
+    read: u64,
+) -> bool {
+    let Some(position) = reads_and_writes_in_order
+        .iter()
+        .position(|event| *event == Some((ring_slot, read)))
+    else {
+        return false;
+    };
+    let reread_at_once =
+        reads_and_writes_in_order.get(position + 1) == Some(&Some((ring_slot, read + 1)));
+    let first_event_after_the_last_write = reads_and_writes_in_order[..position]
+        .iter()
+        .rposition(Option::is_none)
+        .map_or(0, |last_write| last_write + 1);
+    let slots_read_since_the_last_write: Vec<RootRingSlot> = reads_and_writes_in_order
+        [first_event_after_the_last_write..=position]
+        .iter()
+        .filter_map(|event| event.map(|(slot_read, _)| slot_read))
+        .collect();
+    let Some(position_in_the_ring) = ring_slots_in_order
+        .iter()
+        .position(|slot_in_the_ring| *slot_in_the_ring == ring_slot)
+    else {
+        return false;
+    };
+    let read_again_before_the_next_write = reads_and_writes_in_order
+        .iter()
+        .skip(position + 2)
+        .take_while(|event| event.is_some())
+        .any(|event| event.is_some_and(|(slot_read, _)| slot_read == ring_slot));
+    reread_at_once
+        && read_again_before_the_next_write
+        && slots_read_since_the_last_write == ring_slots_in_order[..=position_in_the_ring]
 }
 
-/// `narrow-384-20-2` 上再可写挂载（准入判着、影子账开）：重建那一遍盘 1 每个根环槽报读错（盘 1 一条有效根都没读出，rr2 整环重读），
-/// 盘 0 最新那条根住的槽第一遍读得出、之后照 `reads_of_the_slot` 读坏；按槽并两遍时那一槽只在第一遍自证过。
-fn remount_of_the_narrow_pool_with_the_newest_root_on_device_zero_self_verified_only_on_the_rebuild_pass(
+/// `narrow-384-20-2` 上照一份读坏计划再挂一次交回的：挂载的结局、挂载之后的盘面、钩子每次调的那一刻每个根环槽读过几次、
+/// 挂载结束时每个根环槽读过几次与读写次序。
+struct NarrowRemountThroughFaultedReads {
+    mounted: Result<Mounted, MountError>,
+    image_after_the_remount: MemoryPool,
+    reads_at_each_hook_call: Vec<BTreeMap<RootRingSlot, u64>>,
+    reads_of_each_slot_by_the_end_of_the_remount: BTreeMap<RootRingSlot, u64>,
+    reads_and_writes_in_order: Vec<Option<(RootRingSlot, u64)>>,
+}
+
+/// `narrow-384-20-2` 上再可写挂载一次（准入判着、影子账开）：重建那一遍盘 1 每个根环槽报读错（盘 1 一条有效根都没读出，rr2 整环重读），
+/// 盘 0 上的 `slot` 第一遍读得出、之后照 `reads_of_the_slot` 读坏；按槽并两遍时那一槽只在第一遍自证过。
+fn remount_of_the_narrow_pool_with_device_one_failing_the_rebuild_pass_and_a_slot_after_it(
+    before: &PoolBeforeTheRemount,
+    slot: RootRingSlot,
     reads_of_the_slot: ReadsOfTheSlotFailingAfterTheRebuildPass,
-) -> RemountWithARootSelfVerifiedOnlyOnTheRebuildPass {
-    let before = pool_before_the_remount(
-        StartingPoint::UnitAreaOf384SlotsAfterTwentyOverwritesGrownToTwoUnits,
-    );
-    let slot_of_the_newest_root_on_device_zero =
-        ring_slot_of_the_newest_root_on(&before.image_to_remount, DeviceIdentity(0));
-    let device_one_failing_the_rebuild_pass: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault> =
+) -> NarrowRemountThroughFaultedReads {
+    let mut faulted_reads: BTreeMap<(RootRingSlot, u64), RootRingSlotReadFault> =
         faulted_slots_of_the_plan(&before.image_to_remount, DEVICE_ONE_REPORTS_READ_ERRORS)
             .into_iter()
             .map(|(ring_slot, fault)| {
@@ -1272,52 +1402,82 @@
                 )
             })
             .collect();
+    let mut slots_failing_from_a_read_onward = BTreeMap::new();
+    match reads_of_the_slot {
+        ReadsOfTheSlotFailingAfterTheRebuildPass::FromTheRereadOnward => {
+            slots_failing_from_a_read_onward.insert(
+                slot,
+                (
+                    READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
+                    RootRingSlotReadFault::ReadFails,
+                ),
+            );
+        }
+        ReadsOfTheSlotFailingAfterTheRebuildPass::OnTheRereadAndTheFirstReadOfTheFloorRaiseOnly => {
+            faulted_reads.extend(faulted_reads_of(
+                &[slot],
+                READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
+                RootRingSlotReadFault::ReadFails,
+            ));
+            faulted_reads.extend(faulted_reads_of(
+                &[slot],
+                READ_OF_THE_SLOT_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE,
+                RootRingSlotReadFault::ReadFails,
+            ));
+        }
+        ReadsOfTheSlotFailingAfterTheRebuildPass::OnlyOnTheReread => {
+            faulted_reads.extend(faulted_reads_of(
+                &[slot],
+                READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
+                RootRingSlotReadFault::ReadFails,
+            ));
+        }
+    }
+    let root_ring_reads = root_ring_reads_failing_also_from_a_read_onward(
+        &before.image_to_remount,
+        faulted_reads,
+        slots_failing_from_a_read_onward,
+    );
+    let (mounted, image_after_the_remount, reads_at_each_hook_call) =
+        remount_through(before, &root_ring_reads);
+    NarrowRemountThroughFaultedReads {
+        mounted,
+        image_after_the_remount,
+        reads_at_each_hook_call,
+        reads_of_each_slot_by_the_end_of_the_remount: root_ring_reads.reads_of_each_slot(),
+        reads_and_writes_in_order: root_ring_reads.reads_and_writes_in_order(),
+    }
+}
+
+/// `narrow-384-20-2` 上再可写挂载（准入判着、影子账开）：重建那一遍盘 1 每个根环槽报读错（盘 1 一条有效根都没读出，rr2 整环重读），
+/// 盘 0 最新那条根住的槽第一遍读得出、之后照 `reads_of_the_slot` 读坏；按槽并两遍时那一槽只在第一遍自证过。
+fn remount_of_the_narrow_pool_with_the_newest_root_on_device_zero_self_verified_only_on_the_rebuild_pass(
+    reads_of_the_slot: ReadsOfTheSlotFailingAfterTheRebuildPass,
+) -> RemountWithARootSelfVerifiedOnlyOnTheRebuildPass {
+    let before = pool_before_the_remount(
+        StartingPoint::UnitAreaOf384SlotsAfterTwentyOverwritesGrownToTwoUnits,
+    );
+    let slot_of_the_newest_root_on_device_zero =
+        ring_slot_of_the_newest_root_on(&before.image_to_remount, DeviceIdentity(0));
     let remount_with =
         |reads_of_the_slot_in_this_remount: ReadsOfTheSlotFailingAfterTheRebuildPass| {
-            let mut faulted_reads = device_one_failing_the_rebuild_pass.clone();
-            let mut slots_failing_from_a_read_onward = BTreeMap::new();
-            match reads_of_the_slot_in_this_remount {
-                ReadsOfTheSlotFailingAfterTheRebuildPass::FromTheRereadOnward => {
-                    slots_failing_from_a_read_onward.insert(
-                        slot_of_the_newest_root_on_device_zero,
-                        (
-                            READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
-                            RootRingSlotReadFault::ReadFails,
-                        ),
-                    );
-                }
-                ReadsOfTheSlotFailingAfterTheRebuildPass::OnTheRereadAndTheFirstReadOfTheFloorRaiseOnly => {
-                    faulted_reads.extend(faulted_reads_of(
-                        &[slot_of_the_newest_root_on_device_zero],
-                        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
-                        RootRingSlotReadFault::ReadFails,
-                    ));
-                    faulted_reads.extend(faulted_reads_of(
-                        &[slot_of_the_newest_root_on_device_zero],
-                        READ_OF_THE_SLOT_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE,
-                        RootRingSlotReadFault::ReadFails,
-                    ));
-                }
-                ReadsOfTheSlotFailingAfterTheRebuildPass::OnlyOnTheReread => {
-                    faulted_reads.extend(faulted_reads_of(
-                        &[slot_of_the_newest_root_on_device_zero],
-                        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
-                        RootRingSlotReadFault::ReadFails,
-                    ));
-                }
-            }
-            let root_ring_reads = root_ring_reads_failing_also_from_a_read_onward(
-                &before.image_to_remount,
-                faulted_reads,
-                slots_failing_from_a_read_onward,
+            let NarrowRemountThroughFaultedReads {
+                mounted,
+                image_after_the_remount,
+                reads_at_each_hook_call,
+                reads_of_each_slot_by_the_end_of_the_remount,
+                reads_and_writes_in_order,
+            } = remount_of_the_narrow_pool_with_device_one_failing_the_rebuild_pass_and_a_slot_after_it(
+                &before,
+                slot_of_the_newest_root_on_device_zero,
+                reads_of_the_slot_in_this_remount,
             );
-            let (mounted, image_after_the_remount, reads_at_each_hook_call) =
-                remount_through(&before, &root_ring_reads);
             (
                 mounted,
                 image_after_the_remount,
                 reads_at_each_hook_call,
-                root_ring_reads.reads_of_each_slot(),
+                reads_of_each_slot_by_the_end_of_the_remount,
+                reads_and_writes_in_order,
             )
         };
     let (
@@ -1325,8 +1485,9 @@
         image_after_the_remount,
         reads_at_each_hook_call,
         reads_of_each_slot_by_the_end_of_the_remount,
+        reads_and_writes_in_order,
     ) = remount_with(reads_of_the_slot);
-    let (mounted_for_reference, image_after_the_reference_remount, _, _) =
+    let (mounted_for_reference, image_after_the_reference_remount, _, _, _) =
         remount_with(ReadsOfTheSlotFailingAfterTheRebuildPass::OnlyOnTheReread);
     mounted_for_reference
         .unwrap_or_else(|error| panic!("前提：那一槽只在重读那一遍报读错的再挂做得成：{error:?}"));
@@ -1372,6 +1533,13 @@
             );
             read_differently.is_empty()
         }),
+        the_named_read_opens_the_floor_raise_after_the_last_write_and_is_reread_at_once:
+            read_opens_the_floor_raise_after_the_last_write_and_is_reread_at_once(
+                &reads_and_writes_in_order,
+                &every_root_ring_slot(&before.parameters.geometry),
+                slot_of_the_newest_root_on_device_zero,
+                READ_OF_THE_SLOT_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE,
+            ),
     }
 }
 
@@ -1415,7 +1583,7 @@
 
 /// 场景：同上，那一槽只在重读那一遍与写行、暖机之后推的抬 F 算上限读它那一次报读错，之后读得出（抬 F 那一遍瞬时读坏）。
 /// 预期：照旧——抬 F 那一遍读它读坏就重读一次、重读读出有效根就照读出来的算，与「那一槽只在重读那一遍报读错」的再挂同一份盘面；
-/// checker 不红，丢盘整读不变。
+/// checker 不红，丢盘整读不变；点名的那一次读确是暖机最后一段写之后第一遍读根环（准入算上限那一遍）读它、当场重读（正推腿 K3-e：只靠常量时读序一变照样绿）。
 /// 判别力：抬 F 算上限那一遍读它读坏一次就拒、不重读（`crates/mutations.tsv`「第10i批 S1 抬 F 算上限读它读坏一次就拒、不重读」）时
 /// 挂载在写行、暖机之后报 `FloorRaiseFailedAfterTheMountsPublishes`，第一项红。
 #[test]
@@ -1434,11 +1602,148 @@
             checker_violations_with_both_devices: Vec::new(),
             lone_survivor_reads_what_both_devices_read_before_the_remount: [true, true],
             lone_survivor_reads_every_root_still_in_the_ring_as_before_the_remount: [true, true],
+            the_named_read_opens_the_floor_raise_after_the_last_write_and_is_reread_at_once: true,
         },
         "两遍里只有一遍自证过的根、之后瞬时读坏一次：抬 F 重读读出来就照读出来的算"
     );
 }
 
+/// `narrow-384-20-2` 上第 4 新的那条根住的槽两遍里只有一遍自证过、之后一直读坏时再挂看到的那几样（用例自己断言）。
+#[derive(Debug, PartialEq, Eq)]
+struct RemountWithTheFourthNewestRootSelfVerifiedOnlyOnTheRebuildPass {
+    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
+    root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
+    /// 挂上之后那一槽记成重建分配器两遍里只有一遍自证过的那一类。
+    the_slot_is_self_verified_on_only_one_rebuild_pass: bool,
+    /// 那一槽在重读那一遍之后还被读过（挂载推的抬 F 读了它，读坏）。
+    the_slot_was_read_after_the_reread: bool,
+    /// 这次挂载写行与暖机之后推的每一串抬 F 的上限，按推的先后。
+    ceilings_of_the_floor_raises_pushed_by_the_mount: Vec<CheckpointTxg>,
+    /// 挂载之后现行那一版带的 F。
+    rollback_floor_after_the_mount: Option<CheckpointTxg>,
+    /// 交回的盘面与「那一槽只在重读那一遍报读错」（抬 F 读得出它）的再挂交回的逐字节相同。
+    same_disk_as_a_remount_failing_the_slot_only_on_the_reread: bool,
+    /// 两块盘都在时池级 checker 判违例的那几条。
+    checker_violations_with_both_devices: Vec<String>,
+}
+
+/// 场景：`narrow-384-20-2`（再挂写行、暖机之后推抬 F），重建那一遍盘 1 每个根环槽报读错（rr2 整环重读），环里第 4 新的那条根（txg 21，
+/// 住盘 0）住的槽第一遍读得出、从重读那一遍起每次都报读错：它记成两遍里只有一遍自证过、有效的那一类。环里最新的 5 条根各带一版不同的
+/// 文件内容（前提里现核），挂载自己写的根不改文件，所以这条根就是抬 F 那一刻「第 4 新的非空持久有效根」。
+/// 预期：挂载推的抬 F 算上限那一遍读它仍坏、重读一次仍坏，拿根环表记着的那条根顶上、照它本身算（用户 2026-10-06 定 T1 甲；
+/// D16（发布语义） 已定项 1「抬 F 的上限」：min(每块盘上最新的持久有效根, 第 4 新的非空持久有效根)）：上限恰是这条根的 txg，F 抬到它；
+/// 与「那一槽只在重读那一遍报读错」的再挂同一份盘面；两块盘都在时池级 checker 不判违例。
+/// 判别力：算上限那一遍读坏重读仍坏的有效根当没有根（`crates/mutations.tsv`「第10m批 T1 甲 算上限那一遍读坏重读仍坏的有效根当没有根」）时
+/// 第 4 新的换成 txg 20，上限与 F 都落到 20、盘面跟着不同。K1-3 那条根（txg 4）最旧、上一条 S1 用例那条（txg 24）最新，都罩不住这一处：
+/// 少了 txg 24，挂载写的第一条根就与 txg 23 比成非空、顶上它的位置（推的；第 10m 批在副本上逐槽量过这份盘面：
+/// 盘 1 读坏时盘 0 上只有 txg 21 这一槽、盘 0 读坏时盘 1 上只有 txg 22 那一槽，在这一变异下上限变）。
+#[test]
+fn the_fourth_newest_non_empty_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_still_sets_the_ceiling_of_the_narrow_mounts_floor_raise(
+) {
+    let before = pool_before_the_remount(
+        StartingPoint::UnitAreaOf384SlotsAfterTwentyOverwritesGrownToTwoUnits,
+    );
+    let region_devices = choose_system_configuration(&before.image_to_remount)
+        .expect("系统配置")
+        .immutable
+        .region_devices;
+    let mut roots_newest_first = self_verified_roots_in_the_ring(&before.image_to_remount);
+    roots_newest_first
+        .sort_by_key(|(_, root)| std::cmp::Reverse((root.checkpoint_txg, root.instance)));
+    let distinct_contents_of_the_five_newest_roots: std::collections::BTreeSet<
+        Vec<(u64, Vec<u8>)>,
+    > = roots_newest_first
+        .iter()
+        .take(5)
+        .map(|(_, root)| {
+            live_files_read_whole_at(&before.image_to_remount, root)
+                .unwrap_or_else(|failure| panic!("前提：最新的 5 条根整读得出：{failure}"))
+        })
+        .collect();
+    let (slot_of_the_fourth_newest_root, fourth_newest_root) = roots_newest_first[3];
+    assert_eq!(
+        (
+            fourth_newest_root.checkpoint_txg,
+            region_devices[usize::try_from(slot_of_the_fourth_newest_root.region).expect("区域号")],
+            distinct_contents_of_the_five_newest_roots.len(),
+        ),
+        (CheckpointTxg(21), DeviceIdentity(0), 5),
+        "前提：第 4 新的根是 txg 21、住盘 0，最新的 5 条根各带一版不同的文件内容"
+    );
+    let reference =
+        remount_of_the_narrow_pool_with_device_one_failing_the_rebuild_pass_and_a_slot_after_it(
+            &before,
+            slot_of_the_fourth_newest_root,
+            ReadsOfTheSlotFailingAfterTheRebuildPass::OnlyOnTheReread,
+        );
+    let mounted_for_reference = reference
+        .mounted
+        .unwrap_or_else(|error| panic!("前提：那一槽只在重读那一遍报读错的再挂做得成：{error:?}"));
+    assert_eq!(
+        mounted_for_reference
+            .output
+            .space_admission
+            .floor_raises()
+            .iter()
+            .map(|raised| raised.ceiling)
+            .collect::<Vec<_>>(),
+        vec![fourth_newest_root.checkpoint_txg],
+        "前提：抬 F 读得出那一槽的对照再挂只推一串抬 F，上限恰是第 4 新的那条根"
+    );
+    let remount =
+        remount_of_the_narrow_pool_with_device_one_failing_the_rebuild_pass_and_a_slot_after_it(
+            &before,
+            slot_of_the_fourth_newest_root,
+            ReadsOfTheSlotFailingAfterTheRebuildPass::FromTheRereadOnward,
+        );
+    let mounted = remount.mounted.as_ref();
+    assert_eq!(
+        RemountWithTheFourthNewestRootSelfVerifiedOnlyOnTheRebuildPass {
+            root_ring_of_the_rebuilt_allocator: mounted
+                .map(|mounted| mounted.output.rereads.root_ring_of_the_rebuilt_allocator)
+                .map_err(|error| format!("{error:?}")),
+            the_slot_is_self_verified_on_only_one_rebuild_pass: mounted.is_ok_and(|mounted| {
+                slot_is_self_verified_on_only_one_rebuild_pass(mounted, slot_of_the_fourth_newest_root)
+            }),
+            the_slot_was_read_after_the_reread: remount
+                .reads_of_each_slot_by_the_end_of_the_remount
+                .get(&slot_of_the_fourth_newest_root)
+                .is_some_and(|reads| *reads > READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD),
+            ceilings_of_the_floor_raises_pushed_by_the_mount: mounted.map_or_else(
+                |_| Vec::new(),
+                |mounted| {
+                    mounted
+                        .output
+                        .space_admission
+                        .floor_raises()
+                        .iter()
+                        .map(|raised| raised.ceiling)
+                        .collect()
+                }
+            ),
+            rollback_floor_after_the_mount: mounted
+                .ok()
+                .map(|mounted| mounted.current.root().rollback_floor),
+            same_disk_as_a_remount_failing_the_slot_only_on_the_reread: remount
+                .image_after_the_remount
+                == reference.image_after_the_remount,
+            checker_violations_with_both_devices: checker_violations_on(
+                &remount.image_after_the_remount
+            ),
+        },
+        RemountWithTheFourthNewestRootSelfVerifiedOnlyOnTheRebuildPass {
+            root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
+            the_slot_is_self_verified_on_only_one_rebuild_pass: true,
+            the_slot_was_read_after_the_reread: true,
+            ceilings_of_the_floor_raises_pushed_by_the_mount: vec![fourth_newest_root.checkpoint_txg],
+            rollback_floor_after_the_mount: Some(fourth_newest_root.checkpoint_txg),
+            same_disk_as_a_remount_failing_the_slot_only_on_the_reread: true,
+            checker_violations_with_both_devices: Vec::new(),
+        },
+        "第 4 新的非空有效根两遍里只有一遍自证过、之后一直读坏：抬 F 拿记着的那条根顶上，上限与 F 恰是它的 txg"
+    );
+}
+
 /// K2-2 那组历史停在进程退出那一刻：盘面、参数，被抛弃的根 C、它住的根环槽，与 C 写出的每个落点（盘、槽、那时盘上的字节）。
 struct PoolWithAnAbandonedRoot {
     image: MemoryPool,
@@ -2200,3 +2505,622 @@
         "重读整环都坏、第一遍读出了盘 0 的根：并两遍照常挂上"
     );
 }
+
+impl DeviceOverASparseImage for DeviceFailingTheRebuildPass {
+    fn sparse_image(&self) -> &SparseDevice {
+        &self.inner.image
+    }
+}
+
+/// K2-2 那组历史里盘 `device` 那几个区域上每条不被抛弃的自证根住的槽（这组历史里被抛弃的只有 C，[`pool_with_an_abandoned_root`]）。
+fn ring_slots_of_the_roots_not_abandoned_on(
+    history: &PoolWithAnAbandonedRoot,
+    device: DeviceIdentity,
+) -> Vec<RootRingSlot> {
+    self_verified_roots_in_the_ring(&history.image)
+        .into_iter()
+        .filter(|(ring_slot, _)| {
+            history.parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")]
+                == device
+                && *ring_slot != history.ring_slot_of_the_abandoned_root
+        })
+        .map(|(ring_slot, _)| ring_slot)
+        .collect()
+}
+
+/// 重建那一遍读得出的那一槽之后怎么读坏（第 10i 批代码轮攻方原型 `research/prompts/m3-batch10i-code-r1-opus-model/main.rs` 的 `s1_plans`
+/// 里 a、c 两形）。
+#[derive(Clone, Copy, Debug)]
+enum ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass {
+    /// a：从重读那一遍起每一次都读成这一样。
+    FailingFromTheRereadOnward(RootRingSlotReadFault),
+    /// c：从重读那一遍起每三次读里前两次读成这一样、第三次读得出（算上限那一遍读得出、抬 F 那一遍读它两次都读坏的那一形）。
+    FailingTwoReadsInEveryThreeFromTheRereadOn(RootRingSlotReadFault),
+}
+
+/// 交替读坏那一形点名到重建那一遍之后第几次读为止：远多于一次挂载加 [`USER_CHANGES_AFTER_THE_REMOUNT`] 次改动读那一槽的次数。
+const READS_NAMED_BY_THE_ALTERNATING_FAULT: u64 = 400;
+
+/// 再挂之后接着发几次改动（攻方 K1-1、K1-2、K1-3 同一个数）。
+const USER_CHANGES_AFTER_THE_REMOUNT: usize = 40;
+
+/// 再挂之后接着发的用户改动。
+#[derive(Clone, Copy, Debug)]
+enum UserChangesAfterTheRemount {
+    /// 每次覆盖写一次。
+    Overwrites,
+    /// 第 `first_sequential_write` 次之前覆盖写，从它起每次顺序写 8 个数据单元（攻方 K1-1 那一档）。
+    OverwritesThenSequentialWritesOfEightDataUnitsFrom { first_sequential_write: usize },
+}
+
+/// 一次可写挂载交回的盘留着读坏的计划接着用：挂载的结局、挂载之后的盘、读根环的那一份计数。
+struct RemountKeepingTheFaultedDevices {
+    mounted: Result<Mounted, MountError>,
+    devices: Vec<(DeviceIdentity, DeviceFailingTheRebuildPass)>,
+    root_ring_reads: SharedRootRingReads,
+}
+
+/// K2-2 那组历史上再可写挂载（准入判着、影子账开）：重建那一遍盘 `device_failing_the_rebuild_pass` 上每条不被抛弃的根住的槽报读错
+/// （那块盘一条有效根都没读出，rr2 整环重读），`slot` 第一遍读得出、之后照 `reads_of_the_slot` 读坏；这个进程写过的槽不再读坏。
+fn remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
+    history: &PoolWithAnAbandonedRoot,
+    device_failing_the_rebuild_pass: DeviceIdentity,
+    slot: RootRingSlot,
+    reads_of_the_slot: ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass,
+) -> RemountKeepingTheFaultedDevices {
+    let mut faulted_reads = faulted_reads_of(
+        &ring_slots_of_the_roots_not_abandoned_on(history, device_failing_the_rebuild_pass)
+            .into_iter()
+            .filter(|ring_slot| *ring_slot != slot)
+            .collect::<Vec<_>>(),
+        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
+        RootRingSlotReadFault::ReadFails,
+    );
+    let mut slots_failing_from_a_read_onward = BTreeMap::new();
+    match reads_of_the_slot {
+        ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingFromTheRereadOnward(fault) => {
+            slots_failing_from_a_read_onward
+                .insert(slot, (READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD, fault));
+        }
+        ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingTwoReadsInEveryThreeFromTheRereadOn(
+            fault,
+        ) => {
+            // 迭代上界是点名的读数；没有提前出口。
+            for read in READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD
+                ..READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD
+                    + READS_NAMED_BY_THE_ALTERNATING_FAULT
+            {
+                if !(read - READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD).is_multiple_of(3) {
+                    faulted_reads.insert((slot, read), fault);
+                }
+            }
+        }
+    }
+    let root_ring_reads = root_ring_reads_failing_with(
+        &history.image,
+        faulted_reads,
+        slots_failing_from_a_read_onward,
+        FaultsOnARootRingSlotThisProcessWrote::StopFailing,
+    );
+    let mut devices: Vec<(DeviceIdentity, DeviceFailingTheRebuildPass)> =
+        plain_devices_on(&history.image)
+            .into_iter()
+            .map(|(identity, device)| {
+                (
+                    identity,
+                    DeviceFailingTheRebuildPass {
+                        identity,
+                        inner: device,
+                        root_ring_reads: root_ring_reads.clone(),
+                    },
+                )
+            })
+            .collect();
+    let mounted = mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread(
+        &history.parameters,
+        &mut devices,
+        SpaceAdmission::JudgedByTheFormula,
+        ShadowLedger::On,
+        BeforeTheOneReread::RereadImmediately,
+    );
+    RemountKeepingTheFaultedDevices {
+        mounted,
+        devices,
+        root_ring_reads,
+    }
+}
+
+/// 挂上之后那张根环表里 `slot` 是不是记成重建分配器两遍里只有一遍自证过的那一类。
+fn slot_is_self_verified_on_only_one_rebuild_pass(mounted: &Mounted, slot: RootRingSlot) -> bool {
+    ring_slots_with_a_root_self_verified_on_only_one_of_the_two_rebuild_passes_by(
+        &mounted.allocator,
+    )
+    .contains_key(&slot)
+}
+
+/// 再挂、接着发改动之后看到的那几样（各用例自己断言）。
+#[derive(Debug, PartialEq, Eq)]
+struct ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
+    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
+    root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
+    /// 挂上之后那一槽记成重建分配器两遍里只有一遍自证过的那一类。
+    the_slot_is_self_verified_on_only_one_rebuild_pass: bool,
+    /// 挂上之后那一槽还读坏过（读坏落在抬 F 那一串上，不只落在挂载里）。
+    the_slot_read_bad_after_the_mount: bool,
+    /// 做成了几次改动。
+    changes_published: usize,
+    /// 被拒的那一次改动报的错（没被拒是 `None`）。
+    refusal: Option<String>,
+    /// 每次改动之后两块盘都在时池级 checker 判违例的第一次：（第几次改动，那几条违例）。
+    first_checker_violations: Option<(usize, Vec<String>)>,
+    /// C 还在根环里时 F 抬过了 C 的 txg（之后的回收与影子账按新 F 算，C 引用、挂载那一刻被候选根豁免的单元要补隔离）。
+    floor_raised_above_the_abandoned_root_while_it_was_in_the_ring: bool,
+}
+
+/// 照 [`remount_with_a_slot_self_verified_only_on_the_rebuild_pass`] 再挂，盘留着读坏的计划，接着发 [`USER_CHANGES_AFTER_THE_REMOUNT`] 次
+/// `user_changes`（被拒就停），每次之后两块盘都在时跑池级 checker。
+fn changes_after_a_remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
+    history: &PoolWithAnAbandonedRoot,
+    device_failing_the_rebuild_pass: DeviceIdentity,
+    slot: RootRingSlot,
+    reads_of_the_slot: ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass,
+    user_changes: UserChangesAfterTheRemount,
+) -> ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
+    let RemountKeepingTheFaultedDevices {
+        mounted,
+        devices,
+        root_ring_reads,
+    } = remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
+        history,
+        device_failing_the_rebuild_pass,
+        slot,
+        reads_of_the_slot,
+    );
+    let mounted = match mounted {
+        Ok(mounted) => mounted,
+        Err(error) => {
+            return ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
+                root_ring_of_the_rebuilt_allocator: Err(format!("{error:?}")),
+                the_slot_is_self_verified_on_only_one_rebuild_pass: false,
+                the_slot_read_bad_after_the_mount: false,
+                changes_published: 0,
+                refusal: None,
+                first_checker_violations: None,
+                floor_raised_above_the_abandoned_root_while_it_was_in_the_ring: false,
+            };
+        }
+    };
+    let faulted_reads_by_the_end_of_the_mount = root_ring_reads.faulted_reads();
+    let root_ring_of_the_rebuilt_allocator =
+        Ok(mounted.output.rereads.root_ring_of_the_rebuilt_allocator);
+    let the_slot_is_self_verified_on_only_one_rebuild_pass =
+        slot_is_self_verified_on_only_one_rebuild_pass(&mounted, slot);
+    let (_, session) = MountedSession::of_the_mount(mounted);
+    let mut pool: PoolUnderTest<DeviceFailingTheRebuildPass> = PoolUnderTest {
+        parameters: history.parameters.clone(),
+        device_bytes: history.device_bytes,
+        devices,
+        session: Some(session),
+        write_time_seconds: history.write_time_seconds + 100,
+        content_of_the_current_version: Vec::new(),
+        content_of_each_root: BTreeMap::new(),
+    };
+    let mut changes_published = 0;
+    let mut refusal = None;
+    let mut first_checker_violations = None;
+    let mut floor_raised_above_the_abandoned_root_while_it_was_in_the_ring = false;
+    // 迭代上界是 USER_CHANGES_AFTER_THE_REMOUNT；跨轮携带做成的次数与第一次违例；提前出口只有改动被拒。
+    for change in 1..=USER_CHANGES_AFTER_THE_REMOUNT {
+        let published = match user_changes {
+            UserChangesAfterTheRemount::OverwritesThenSequentialWritesOfEightDataUnitsFrom {
+                first_sequential_write,
+            } if change >= first_sequential_write => pool
+                .sequential_write(8 * data_unit_payload_capacity() + 100)
+                .map(|_| ()),
+            UserChangesAfterTheRemount::OverwritesThenSequentialWritesOfEightDataUnitsFrom {
+                ..
+            }
+            | UserChangesAfterTheRemount::Overwrites => pool.overwrite(OVERWRITE_BYTES).map(|_| ()),
+        };
+        if let Err(refused) = published {
+            refusal = Some(format!("{refused:?}"));
+            break;
+        }
+        changes_published = change;
+        let image = pool.image();
+        if pool.rollback_floor() > history.abandoned_root.checkpoint_txg
+            && abandoned_root_still_in_the_ring(&image, history)
+        {
+            floor_raised_above_the_abandoned_root_while_it_was_in_the_ring = true;
+        }
+        let violations = checker_violations_on(&image);
+        if first_checker_violations.is_none() && !violations.is_empty() {
+            first_checker_violations = Some((change, violations));
+        }
+    }
+    ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
+        root_ring_of_the_rebuilt_allocator,
+        the_slot_is_self_verified_on_only_one_rebuild_pass,
+        the_slot_read_bad_after_the_mount: root_ring_reads.faulted_reads()
+            > faulted_reads_by_the_end_of_the_mount,
+        changes_published,
+        refusal,
+        first_checker_violations,
+        floor_raised_above_the_abandoned_root_while_it_was_in_the_ring,
+    }
+}
+
+/// 场景：攻方 K1-1 那一形（`research/prompts/m3-batch10i-code-r1-opus-output.md`「K1-1」一节；原型 `abandoned_start(UnitAreaOf384Slots, 3, 0)`、
+/// 计划 `s1-A@0/3(d0)-trig0-c-Z`）：K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 3 次），再可写挂载时重建那一遍盘 0 每条不被抛弃的根住的槽报读错
+/// （rr2 整环重读），被抛弃的根 C 那一槽第一遍读得出、从重读起每三次读里前两次读回全 0：并两遍时 C 记成两遍里只有一遍自证过的那一类。
+/// 挂上之后覆盖写 5 次、从第 6 次起每次顺序写 8 个数据单元，共 40 次；准入抬 F 时算上限那一遍读 C 那一槽读得出，抬 F 那一遍读它两次都读坏。
+/// 预期：抬 F 那一遍拿根环表记着的 C 顶上（用户 2026-10-06 定 T1 甲），按新 F 重算的影子账照隔离 C 引用、挂载那一刻被候选根豁免的单元：
+/// 40 次都做成，每次之后池级 checker 都不判违例；C 还在环里时 F 就抬过了 C 的 txg（这一形的前提）。
+/// 改之前（第 10i 批：抬 F 那一遍读坏重读仍坏的这一类只钉回收门槛、不进交回的根表）在副本上实测：第 9 次改动之后（F = 17、txg 24）
+/// checker 判 I-7.4「被抛弃的根（实例 2、txg 9）引用的单元已被重新分配或抹头……实例表单元 在盘 0 槽 1536 的那一份与位置条目里的校验和对不上」。
+/// 判别力：抬 F 那一遍读坏重读仍坏的这一类不拿记着的根顶上（`crates/mutations.tsv`「第10l批 T1 甲 抬 F 重算影子账那一遍读坏重读仍坏的这一类不拿记着的根顶上」）。
+#[test]
+fn an_abandoned_root_self_verified_only_on_the_rebuild_pass_read_back_as_zeros_by_the_floor_raise_keeps_its_units_isolated(
+) {
+    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::UnitAreaOf384Slots, 3);
+    assert_eq!(
+        changes_after_a_remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
+            &history,
+            DeviceIdentity(0),
+            history.ring_slot_of_the_abandoned_root,
+            ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingTwoReadsInEveryThreeFromTheRereadOn(
+                RootRingSlotReadFault::ReadsBackAllZero,
+            ),
+            UserChangesAfterTheRemount::OverwritesThenSequentialWritesOfEightDataUnitsFrom {
+                first_sequential_write: 6,
+            },
+        ),
+        ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
+            root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
+            the_slot_is_self_verified_on_only_one_rebuild_pass: true,
+            the_slot_read_bad_after_the_mount: true,
+            changes_published: USER_CHANGES_AFTER_THE_REMOUNT,
+            refusal: None,
+            first_checker_violations: None,
+            floor_raised_above_the_abandoned_root_while_it_was_in_the_ring: true,
+        },
+        "被抛弃的根两遍里只有一遍自证过、抬 F 那一遍读它读坏：拿记着的根顶上，影子账照隔离，checker 不红"
+    );
+}
+
+/// 场景：攻方 K1-2 那一形：同一组历史，C 那一槽第一遍读得出、从重读起一直报读错（计划 a），挂上之后覆盖写 40 次。
+/// 预期：算上限那一遍拿记着的 C 顶上，按实例表判被抛弃、不进上限（用户 2026-10-06 定 T1 甲）：F 抬得过 C 的 txg，40 次都做成，checker 每次都不判违例。
+/// 改之前（第 10i 批：照挂载那一刻读坏按能住的最大 txg 并进上限）在副本上实测：F 卡在 C 的 txg 9，第 15 次覆盖写报 `NoSpaceAfterRaisingTheFloor`。
+/// 判别力：算上限那一遍读它重读仍坏照拒（`crates/mutations.tsv`「第10l批 T1 甲 算上限那一遍读坏重读仍坏的这一类照拒」）时
+/// 抬 F 报 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`、改动被拒。
+#[test]
+fn an_abandoned_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling(
+) {
+    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::UnitAreaOf384Slots, 3);
+    assert_eq!(
+        changes_after_a_remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
+            &history,
+            DeviceIdentity(0),
+            history.ring_slot_of_the_abandoned_root,
+            ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingFromTheRereadOnward(
+                RootRingSlotReadFault::ReadFails,
+            ),
+            UserChangesAfterTheRemount::Overwrites,
+        ),
+        ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
+            root_ring_of_the_rebuilt_allocator: Ok(
+                RootRingOfTheRebuiltAllocatorRead::OnTheOneReread
+            ),
+            the_slot_is_self_verified_on_only_one_rebuild_pass: true,
+            the_slot_read_bad_after_the_mount: true,
+            changes_published: USER_CHANGES_AFTER_THE_REMOUNT,
+            refusal: None,
+            first_checker_violations: None,
+            floor_raised_above_the_abandoned_root_while_it_was_in_the_ring: true,
+        },
+        "被抛弃的根两遍里只有一遍自证过、之后一直读坏：不卡上限，40 次覆盖写都做成"
+    );
+}
+
+/// 场景：攻方 K1-3 那一形：同一组历史，重建那一遍盘 0 每条不被抛弃的根住的槽报读错（rr2 整环重读），盘 1 上 txg 4 那条有效根住的槽
+/// （区域 1、槽 1）第一遍读得出、从重读起一直报读错：它记成两遍里只有一遍自证过、有效的那一类。挂上之后覆盖写 40 次。
+/// 预期：算上限、抬 F 那一串读它读坏、重读仍坏时拿记着的那条根顶上、照它本身算（用户 2026-10-06 定 T1 甲）：40 次都做成，checker 每次都不判违例。
+/// 改之前（第 10i 批：照挂载那一刻读坏按它能住的最大 txg 并进上限）在副本上实测：第 12 次覆盖写报 `NoSpaceAfterRaisingTheFloor`
+/// （`FloorAlreadyAtTheCeiling { floor: 4, ceiling: 4 }`）；同一组历史上别的 10 个住有效根的槽（盘 0 读坏时盘 1 上的、盘 1 读坏时盘 0 上的）
+/// 改之前 7 个在第 12–16 次覆盖写上停在同一种拒、txg 0–2 那 3 个做满 40 次，改之后 10 个都做满 40 次、checker 不红。
+/// 判别力：算上限那一遍读它重读仍坏照拒（`crates/mutations.tsv`「第10l批 T1 甲 算上限那一遍读坏重读仍坏的这一类照拒（有效根那一槽）」）时改动被拒。
+#[test]
+fn a_valid_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling(
+) {
+    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::UnitAreaOf384Slots, 3);
+    let slot_of_the_valid_root_of_txg_four = RootRingSlot { region: 1, slot: 1 };
+    assert_eq!(
+        self_verified_roots_in_the_ring(&history.image)
+            .into_iter()
+            .find(|(ring_slot, _)| *ring_slot == slot_of_the_valid_root_of_txg_four)
+            .map(|(_, root)| root.checkpoint_txg),
+        Some(CheckpointTxg(4)),
+        "前提：这组历史里区域 1、槽 1 住着 txg 4 那条有效根"
+    );
+    assert_eq!(
+        changes_after_a_remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
+            &history,
+            DeviceIdentity(0),
+            slot_of_the_valid_root_of_txg_four,
+            ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingFromTheRereadOnward(
+                RootRingSlotReadFault::ReadFails,
+            ),
+            UserChangesAfterTheRemount::Overwrites,
+        ),
+        ChangesAfterARemountWithASlotSelfVerifiedOnlyOnTheRebuildPass {
+            root_ring_of_the_rebuilt_allocator: Ok(RootRingOfTheRebuiltAllocatorRead::OnTheOneReread),
+            the_slot_is_self_verified_on_only_one_rebuild_pass: true,
+            the_slot_read_bad_after_the_mount: true,
+            changes_published: USER_CHANGES_AFTER_THE_REMOUNT,
+            refusal: None,
+            first_checker_violations: None,
+            floor_raised_above_the_abandoned_root_while_it_was_in_the_ring: true,
+        },
+        "有效根两遍里只有一遍自证过、之后一直读坏：照记着的那条根算上限，40 次覆盖写都做成、checker 不红"
+    );
+}
+
+/// K2-2 那组历史上再挂、覆盖写几次、再正常卸载之后看到的那几样（各用例自己断言）。
+#[derive(Debug, PartialEq, Eq)]
+struct UnmountAfterARemountWithTheAbandonedRootSelfVerifiedOnlyOnTheRebuildPass {
+    /// 挂上之后 C 那一槽记成重建分配器两遍里只有一遍自证过的那一类。
+    the_abandoned_root_is_self_verified_on_only_one_rebuild_pass: bool,
+    /// 卸载之前的覆盖写做成了几次。
+    overwrites_published: usize,
+    /// 卸载的结局（错只记成员名之前那一段）。
+    unmounted: Result<(), String>,
+    /// 卸载之后 F 高于 C 的 txg（卸载那一串把 F 抬过了 C 那一条时间线上还有效的根）。
+    floor_above_the_abandoned_root_after_the_unmount: bool,
+    /// C 还在根环里。
+    the_abandoned_root_still_in_the_ring_after_the_unmount: bool,
+    /// 卸载之后分配器里空闲的 C 引用的实例表单元那几份（盘，槽）。
+    instance_table_slots_of_the_abandoned_root_free_after_the_unmount:
+        Vec<(DeviceIdentity, SlotNumber)>,
+    /// 卸载之后两块盘都在时池级 checker 判违例的那几条。
+    checker_violations_after_the_unmount: Vec<String>,
+}
+
+/// 卸载之前覆盖写几次：抬 F 之前（攻方 K1-1 那一档准入第一次抬 F 在第 8 次改动上）。
+const OVERWRITES_BEFORE_THE_UNMOUNT: usize = 3;
+
+/// 场景：攻方 K1-1 末段那一形（「卸载那一串不判上限」那一句，判决 `research/prompts/m3-batch10i-code-r1-main-verification.md` 第五节第一行）：
+/// K2-2 那组历史（单元区 384 槽、实例 1 覆盖写 3 次），重建那一遍盘 0 每条不被抛弃的根住的槽报读错，C 那一槽第一遍读得出、从重读起一直报读错；
+/// 挂上之后覆盖写 3 次，再正常卸载，盘留着读坏。
+/// 预期：卸载那一串重算影子账时拿记着的 C 顶上（用户 2026-10-06 定 T1 甲），C 引用的实例表单元（挂载那一刻被实例 2 的有效根豁免，
+/// 卸载把 F 抬过那几条根之后只剩 C 引用）照隔离：卸载做成，之后分配器里那两份都不空闲，C 还在环里，checker 不判违例。
+/// 判别力：抬 F 重算影子账那一遍读坏重读仍坏的这一类不拿记着的根顶上（`crates/mutations.tsv`「第10l批 T1 甲 卸载那一串读坏重读仍坏的这一类不拿记着的根顶上」）
+/// 时那两份卸载之后空闲，最后一项红。
+#[test]
+fn unmounting_after_a_remount_with_an_abandoned_root_self_verified_only_on_the_rebuild_pass_keeps_its_units_isolated(
+) {
+    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::UnitAreaOf384Slots, 3);
+    let RemountKeepingTheFaultedDevices {
+        mounted, devices, ..
+    } = remount_with_a_slot_self_verified_only_on_the_rebuild_pass(
+        &history,
+        DeviceIdentity(0),
+        history.ring_slot_of_the_abandoned_root,
+        ReadsOfTheSlotSelfVerifiedOnlyOnTheRebuildPass::FailingFromTheRereadOnward(
+            RootRingSlotReadFault::ReadFails,
+        ),
+    );
+    let mounted = mounted.unwrap_or_else(|error| panic!("前提：再可写挂载做得成：{error:?}"));
+    let the_abandoned_root_is_self_verified_on_only_one_rebuild_pass =
+        slot_is_self_verified_on_only_one_rebuild_pass(
+            &mounted,
+            history.ring_slot_of_the_abandoned_root,
+        );
+    let (_, session) = MountedSession::of_the_mount(mounted);
+    let mut pool: PoolUnderTest<DeviceFailingTheRebuildPass> = PoolUnderTest {
+        parameters: history.parameters.clone(),
+        device_bytes: history.device_bytes,
+        devices,
+        session: Some(session),
+        write_time_seconds: history.write_time_seconds + 100,
+        content_of_the_current_version: Vec::new(),
+        content_of_each_root: BTreeMap::new(),
+    };
+    let mut overwrites_published = 0;
+    // 迭代上界是 OVERWRITES_BEFORE_THE_UNMOUNT；提前出口只有被拒。
+    for _ in 0..OVERWRITES_BEFORE_THE_UNMOUNT {
+        if pool.overwrite(OVERWRITE_BYTES).is_err() {
+            break;
+        }
+        overwrites_published += 1;
+    }
+    let mut session_being_unmounted = pool.session.take().expect("覆盖写之后会话还在");
+    let unmounted = unmount(
+        &pool.parameters,
+        &mut pool.devices,
+        &mut session_being_unmounted.allocator,
+        &mut session_being_unmounted.current,
+        ShadowLedger::On,
+    );
+    let image = pool.image();
+    let outcome = UnmountAfterARemountWithTheAbandonedRootSelfVerifiedOnlyOnTheRebuildPass {
+        the_abandoned_root_is_self_verified_on_only_one_rebuild_pass,
+        overwrites_published,
+        unmounted: unmounted.map(|_| ()).map_err(|error| format!("{error:?}")),
+        floor_above_the_abandoned_root_after_the_unmount: session_being_unmounted
+            .current
+            .root()
+            .rollback_floor
+            > history.abandoned_root.checkpoint_txg,
+        the_abandoned_root_still_in_the_ring_after_the_unmount: abandoned_root_still_in_the_ring(
+            &image, &history,
+        ),
+        instance_table_slots_of_the_abandoned_root_free_after_the_unmount: history
+            .abandoned_root
+            .instance_table
+            .locations
+            .iter()
+            .filter(|location| {
+                session_being_unmounted.allocator.devices
+                    [usize::try_from(location.device.0).expect("盘号")]
+                .is_free(location.slot)
+            })
+            .map(|location| (location.device, location.slot))
+            .collect(),
+        checker_violations_after_the_unmount: checker_violations_on(&image),
+    };
+    assert_eq!(
+        outcome,
+        UnmountAfterARemountWithTheAbandonedRootSelfVerifiedOnlyOnTheRebuildPass {
+            the_abandoned_root_is_self_verified_on_only_one_rebuild_pass: true,
+            overwrites_published: OVERWRITES_BEFORE_THE_UNMOUNT,
+            unmounted: Ok(()),
+            floor_above_the_abandoned_root_after_the_unmount: true,
+            the_abandoned_root_still_in_the_ring_after_the_unmount: true,
+            instance_table_slots_of_the_abandoned_root_free_after_the_unmount: Vec::new(),
+            checker_violations_after_the_unmount: Vec::new(),
+        },
+        "卸载那一串读 C 读坏、重读仍坏：拿记着的 C 顶上，C 引用的实例表单元照隔离"
+    );
+}
+
+/// K2-2 那组历史上再挂（读坏只落在挂载里）、接着覆盖写到 C 离开根环，看分配器里 C 的落点（各用例自己断言）。
+#[derive(Debug, PartialEq, Eq)]
+struct AbandonedRootWatchedInTheAllocatorUntilItLeavesTheRing {
+    /// 挂载做成时报的重建分配器读根环在哪一遍读出；拒了是拒的成员。
+    root_ring_of_the_rebuilt_allocator: Result<RootRingOfTheRebuiltAllocatorRead, String>,
+    /// 挂上之后 C 那一槽记成重建分配器两遍里只有一遍自证过的那一类。
+    the_abandoned_root_is_self_verified_on_only_one_rebuild_pass: bool,
+    /// C 还在根环里的每次覆盖写之后，C 引用的实例表单元在分配器里空闲的那几份（第几次覆盖写，盘，槽）。
+    instance_table_slots_of_the_abandoned_root_free_while_it_was_in_the_ring:
+        Vec<(usize, DeviceIdentity, SlotNumber)>,
+    /// 这一趟里环里最旧有效根越过了 C 的 txg、C 还在环里（进程内记根那一次的回收门槛越过 C，C 引用的、挂载那一刻被候选根豁免的单元被回收）。
+    oldest_valid_root_passed_the_abandoned_root_while_it_was_in_the_ring: bool,
+    /// C 离开根环那一次覆盖写之后，C 写出的落点在分配器里不空闲的那几个（盘，槽）：只被 C 引用、挂载时隔离的，C 离环时清掉。
+    places_the_abandoned_root_wrote_not_free_once_it_left_the_ring:
+        Vec<(DeviceIdentity, SlotNumber)>,
+    /// C 离开根环那一刻两块盘都在时池级 checker 判违例的那几条。
+    checker_violations_when_it_leaves_the_ring: Vec<String>,
+}
+
+/// 场景：K2-2 那组历史（4 GiB、实例 1 覆盖写 1 次：盘宽，进程内不推抬 F），第一遍盘 1 每个根环槽报读错（rr2 整环重读），重读只坏 C 的根槽（报读错）：
+/// C 记成两遍里只有一遍自证过、被抛弃的那一类。挂上之后照常读盘，覆盖写到 C 离开根环。
+/// 预期：进程内记根时这一类被抛弃根照它记着的落点算进「环里还在的被抛弃根引用着的落点」——环里最旧有效根越过 C 的 txg 那一次（C 还在环里）回收出来的
+/// C 的实例表单元照补隔离，C 在环里时那两份一次都不空闲；C 离开根环那一次按它记着的落点清隔离，C 写出的落点那一刻全空闲；checker 不红。
+/// 判别力：这一类被抛弃根不算进还被引用的落点（`crates/mutations.tsv`「第10l批 新类被抛弃那一半不算进环里还被引用的落点」）时第三项红；
+/// 被换下时不清隔离（「第10l批 新类被抛弃那一半被换下时不清隔离」）时第五项红。
+#[test]
+fn an_abandoned_root_self_verified_only_on_the_rebuild_pass_holds_its_units_in_the_allocator_until_it_leaves_the_ring(
+) {
+    let history = pool_with_an_abandoned_root(HistoryDeviceWidth::FourGibibytes, 1);
+    let mut faulted_reads = faulted_reads_of(
+        &root_ring_slots_of_device_one(&history),
+        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_ALLOCATOR_REBUILD,
+        RootRingSlotReadFault::ReadFails,
+    );
+    faulted_reads.extend(faulted_reads_of(
+        &[history.ring_slot_of_the_abandoned_root],
+        READ_OF_EACH_ROOT_RING_SLOT_BY_THE_REREAD,
+        RootRingSlotReadFault::ReadFails,
+    ));
+    let RemountThroughFaultedReads {
+        mounted,
+        image_after_the_remount,
+        ..
+    } = remount_the_pool_with_an_abandoned_root(&history, faulted_reads);
+    let mounted = mounted.unwrap_or_else(|error| panic!("前提：再可写挂载做得成：{error:?}"));
+    let root_ring_of_the_rebuilt_allocator =
+        Ok(mounted.output.rereads.root_ring_of_the_rebuilt_allocator);
+    let the_abandoned_root_is_self_verified_on_only_one_rebuild_pass =
+        slot_is_self_verified_on_only_one_rebuild_pass(
+            &mounted,
+            history.ring_slot_of_the_abandoned_root,
+        );
+    let (_, session) = MountedSession::of_the_mount(mounted);
+    let mut pool: PoolUnderTest<SparseBlockDevice> = PoolUnderTest {
+        parameters: history.parameters.clone(),
+        device_bytes: history.device_bytes,
+        devices: plain_devices_on(&image_after_the_remount),
+        session: Some(session),
+        write_time_seconds: history.write_time_seconds + 100,
+        content_of_the_current_version: Vec::new(),
+        content_of_each_root: BTreeMap::new(),
+    };
+    let root_ring_slot_count = every_root_ring_slot(&history.parameters.geometry).len();
+    let instance_table_slots_of_the_abandoned_root_free =
+        |pool_watched: &PoolUnderTest<SparseBlockDevice>| {
+            history
+                .abandoned_root
+                .instance_table
+                .locations
+                .iter()
+                .filter(|location| {
+                    pool_watched.session().allocator.devices
+                        [usize::try_from(location.device.0).expect("盘号")]
+                    .is_free(location.slot)
+                })
+                .map(|location| (location.device, location.slot))
+                .collect::<Vec<_>>()
+        };
+    let mut instance_table_slots_of_the_abandoned_root_free_while_it_was_in_the_ring = Vec::new();
+    let mut oldest_valid_root_passed_the_abandoned_root_while_it_was_in_the_ring = false;
+    let mut overwrites = 0usize;
+    // 迭代上界是根环的槽数（C 的根槽在一圈之内被盖掉）；没有提前出口。
+    while abandoned_root_still_in_the_ring(&pool.image(), &history) {
+        assert!(
+            overwrites < root_ring_slot_count,
+            "C 的根槽在一圈（{root_ring_slot_count} 次发布）之内被盖掉"
+        );
+        pool.overwrite(OVERWRITE_BYTES)
+            .unwrap_or_else(|refusal| panic!("再挂之后第 {overwrites} 次覆盖写：{refusal:?}"));
+        overwrites += 1;
+        let image = pool.image();
+        if !abandoned_root_still_in_the_ring(&image, &history) {
+            break;
+        }
+        let oldest_valid_root_txg = self_verified_roots_in_the_ring(&image)
+            .into_iter()
+            .filter(|(ring_slot, _)| *ring_slot != history.ring_slot_of_the_abandoned_root)
+            .map(|(_, root)| root.checkpoint_txg)
+            .min();
+        if oldest_valid_root_txg
+            .is_some_and(|oldest| oldest > history.abandoned_root.checkpoint_txg)
+        {
+            oldest_valid_root_passed_the_abandoned_root_while_it_was_in_the_ring = true;
+        }
+        instance_table_slots_of_the_abandoned_root_free_while_it_was_in_the_ring.extend(
+            instance_table_slots_of_the_abandoned_root_free(&pool)
+                .into_iter()
+                .map(|(device, slot)| (overwrites, device, slot)),
+        );
+    }
+    let places_the_abandoned_root_wrote_not_free_once_it_left_the_ring = history
+        .places_the_abandoned_root_wrote
+        .iter()
+        .filter(|(device, slot, _)| {
+            !pool.session().allocator.devices[usize::try_from(device.0).expect("盘号")]
+                .is_free(*slot)
+        })
+        .map(|(device, slot, _)| (*device, *slot))
+        .collect();
+    assert_eq!(
+        AbandonedRootWatchedInTheAllocatorUntilItLeavesTheRing {
+            root_ring_of_the_rebuilt_allocator,
+            the_abandoned_root_is_self_verified_on_only_one_rebuild_pass,
+            instance_table_slots_of_the_abandoned_root_free_while_it_was_in_the_ring,
+            oldest_valid_root_passed_the_abandoned_root_while_it_was_in_the_ring,
+            places_the_abandoned_root_wrote_not_free_once_it_left_the_ring,
+            checker_violations_when_it_leaves_the_ring: checker_violations_on(&pool.image()),
+        },
+        AbandonedRootWatchedInTheAllocatorUntilItLeavesTheRing {
+            root_ring_of_the_rebuilt_allocator: Ok(
+                RootRingOfTheRebuiltAllocatorRead::OnTheOneReread
+            ),
+            the_abandoned_root_is_self_verified_on_only_one_rebuild_pass: true,
+            instance_table_slots_of_the_abandoned_root_free_while_it_was_in_the_ring: Vec::new(),
+            oldest_valid_root_passed_the_abandoned_root_while_it_was_in_the_ring: true,
+            places_the_abandoned_root_wrote_not_free_once_it_left_the_ring: Vec::new(),
+            checker_violations_when_it_leaves_the_ring: Vec::new(),
+        },
+        "两遍里只有一遍自证过的被抛弃根：在环里时它引用的单元一直扣着，离环时按它记着的落点清"
+    );
+}
```

## `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs`

```diff
--- a/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
+++ b/crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs
@@ -556,7 +556,7 @@
 }
 
 /// 最新的根下面还没有记账树、inode 树与分配记录树时（mkfs 之后、零单元发布之后）checker 报不适用的那 45 条；
-/// 其余 38 条要真被评估过且成立（条数按今天的 `IMPLEMENTED_INVARIANTS` 83 条数）。映射 key 与单元头相符（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`）报不适用是因为中央映射树的根指针全零，
+/// 其余 39 条要真被评估过且成立（条数按今天的 `IMPLEMENTED_INVARIANTS` 84 条数）。映射 key 与单元头相符（`MAPPING_KEY_MATCHES_THE_UNIT_HEADER`）报不适用是因为中央映射树的根指针全零，
 /// 一条映射条目都没有。I-3.11（已分配减 defer 等于最新根走读） 与 I-3.1（已分配统计对得上） 同一个理由：最新根下面没有记账树。I-9.14（树表条目的诞生 txg 跨根不变） 在这里报不适用是因为 mkfs 种的第 0 版树表是空的：
 /// 一棵树的条目都没有，跨根比不出来（有了文件版本之后也只有一条根有条目，见 `NOT_APPLICABLE_WITH_ONE_FILE_VERSION`）；
 /// I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 同一个理由，比的是根环里全部读得出的根（不按实例表判有效）：根环里每条根的树表都是
@@ -1148,8 +1148,8 @@
 /// 验收：取号 1、不写行；写行那次发布 (1, 1)、jsn 1、事务号 0、反向链 0、零单元、落盘 1；暖机一次 (1, 2)、jsn 2、反向链接 jsn 1、落盘 0；
 /// 分配器只有 mkfs 的两个单元（每盘 3 槽）；第一个文件版本 (1, 3)、jsn 3、事务号 1，数据单元落 50180、换下 mkfs 那片树表；
 /// mkfs 之后的录制流与新池新建文件那条逐字节相同、写出的那一版也相同；冷启动读回文件；checker 一条违例都没有（条数按今天的
-/// `IMPLEMENTED_INVARIANTS` 83 条数）——mkfs 之后 34 条成立（环是空的，I-8.6、I-8.9、I-8.10、I-8.12 也报不适用）、挂载之后 38 条成立，
-/// 没有记账树 / inode 树 / 分配记录树 / 映射条目的那几条报不适用，第一个文件版本之后 66 条成立、I-7.9、I-9.14、I-9.28、I-8.7、I-8.8、
+/// `IMPLEMENTED_INVARIANTS` 84 条数）——mkfs 之后 35 条成立（环是空的，I-8.6、I-8.9、I-8.10、I-8.12 也报不适用）、挂载之后 39 条成立，
+/// 没有记账树 / inode 树 / 分配记录树 / 映射条目的那几条报不适用，第一个文件版本之后 67 条成立、I-7.9、I-9.14、I-9.28、I-8.7、I-8.8、
 /// I-3.12、I-4.1、I-4.7、I-4.6、I-4.4 与墓碑树、谱系树那七条（I-9.17–I-9.23）仍报不适用（树表条目只出现在这一条根的树表里；同一实例只有一条非 0 事务号的记录；没有哪个事务号落了两条记录、
 /// 每条都带提交标记；只有实例 1，实例表一行都没有；最新根之上没有记录；没有删除；I-4.4 第一版恒报不适用）。
 #[test]
```

## `crates/mutations.tsv`

```diff
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -22,7 +22,7 @@
 步 3：暖机只推一次	crates/singlefs-core/src/mount.rs	        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(warm_up_publishes.len()).expect("次数") < 1	-p singlefs-harness --test writable_mount_takes_a_second_instance -- damaging_every_instance_two_root	damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device
 步 3：前缀跨实例边界（把别的实例的记录也接上）	crates/singlefs-replay-prefix/src/lib.rs	    record.instance.0 == root_instance.0 && record.checkpoint_txg.0 > root_checkpoint_txg.0	    record.instance.0 > root_instance.0 || (record.instance.0 == root_instance.0 && record.checkpoint_txg.0 > root_checkpoint_txg.0)	-p singlefs-checker-tier --test crash_enumeration_new_pool_file_creation_stream -- layer0_quick_tier_matches	layer0_quick_tier_matches_the_full_tally_shape
 步 3：链首锚点错一位（接在所选根自己那条记录之后第二条）	crates/singlefs-replay-prefix/src/lib.rs	        expected_next = Some((root_instance, JournalCounter(counter.0 + 1)));	        expected_next = Some((root_instance, JournalCounter(counter.0 + 2)));	-p singlefs-harness --test writable_mount_takes_a_second_instance -- stray_record_of_the_previous_instance	stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row
-步 3：I-3.8 判定恒真	crates/singlefs-checker/src/walk.rs	            unique && below_mount_root && chain_record_last,	            unique || below_mount_root || chain_record_last || true,	-p singlefs-harness --test writable_mount_takes_a_second_instance -- checker_rejects_an_instance_table_row	checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root
+步 3：I-3.8 判定恒真	crates/singlefs-checker/src/walk.rs	            unique && below_mount_root,	            unique || below_mount_root || true,	-p singlefs-harness --test writable_mount_takes_a_second_instance -- checker_rejects_an_instance_table_row	checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root
 步 3：重建分配器时不把已释放的放进 defer 队列	crates/singlefs-core/src/allocator.rs	            if record.is_released {\n                device_map.mark_released(record.slot, span);\n            }	            if record.is_released && false {\n                device_map.mark_released(record.slot, span);\n            }	-p singlefs-harness --test writable_mount_takes_a_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
 步 4：影子账一个槽都不隔离	crates/singlefs-core/src/mount.rs	            allocator.isolate_abandoned(device, slot, span_slots);	            let _ = (device, slot, span_slots);	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored with_the_shadow_ledger_a_publish_after_a_recovery	with_the_shadow_ledger_a_publish_after_a_recovery_keeps_off_the_abandoned_data_slot_and_the_abandoned_root_reads_the_third_content
 步 4：回退候选集不按实例表判（被抛弃时间线的根也能退到）	crates/singlefs-core/src/mount.rs	    if abandoned_by_table(&target_root, current_table) {	    if false && abandoned_by_table(&target_root, current_table) {	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored rolling_back_to_a_missing_root	rolling_back_to_a_missing_root_an_abandoned_root_or_a_root_without_file_is_refused_before_any_write
@@ -377,10 +377,10 @@
 实例表第二片：冷启动走读不沿链读（第二片坏了照样走完）	crates/singlefs-core/src/recovery.rs	            instance_table_chain_of_root(reader, root)?;	            let _ = instance_table_chain_of_root(reader, root);	-p singlefs-harness --test instance_table_chain_read -- cold_start_walks	cold_start_walks_every_page_of_the_instance_table_and_an_unreadable_second_page_stops_it
 实例表第二片：取号之前的预演不按释放判定路径逐片核被换下的旧链（旧链上一片不在账里时取号写完才撞上，号烧掉；预演走的是发布路径落盘之前那一段，这一处两边共用；树表 0 条的一版上写行那一处）	crates/singlefs-core/src/transaction.rs	    let chain = instance_table_chain_to_release(&instance_table.replaced_chain, allocator)?;	    let chain: Vec<Placement> = {\n        let _ = &instance_table.replaced_chain;\n        Vec::new()\n    };	-p singlefs-harness --test instance_table_chain_read -- writable_mount_follows_the_chain	writable_mount_follows_the_chain_and_refuses_a_page_missing_from_the_allocation_records_before_acquisition
 实例表第二片：checker 拿第 0 片的身份判链上每一片	crates/singlefs-checker/src/walk.rs	            let expected_identity = (0, PACKED_TYPE_INSTANCE_TABLE, page_index, 0);	            let expected_identity = (0, PACKED_TYPE_INSTANCE_TABLE, 0, 0);	-p singlefs-harness --test instance_table_chain_read -- the_pool_checker_walks	the_pool_checker_walks_both_pages_of_a_valid_chain_and_every_invariant_holds
-实例表第二片：checker 不沿链走（链指针指到别的单元不红）	crates/singlefs-checker/src/walk.rs	                ChainRecordView::NextPage(next_page) => {\n                    pointer = next_page;\n                    page_index += 1;\n                }	                ChainRecordView::NextPage(_next_page) => break,	-p singlefs-harness --test instance_table_chain_read -- chain_pointer_to_another	chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant
-实例表第二片：checker 不沿链走（链指针校验和不对不红）	crates/singlefs-checker/src/walk.rs	                ChainRecordView::NextPage(next_page) => {\n                    pointer = next_page;\n                    page_index += 1;\n                }	                ChainRecordView::NextPage(_next_page) => break,	-p singlefs-harness --test instance_table_chain_read -- chain_pointer_whose_checksum	chain_pointer_whose_checksum_does_not_match_the_second_page_reddens_the_checksum_invariant
-实例表第二片：checker 按片各判行唯一，不按整张表	crates/singlefs-checker/src/walk.rs	        for records in pages {\n            let mut chain_record_last_in_this_page = false;	        for records in pages {\n            instances.clear();\n            let mut chain_record_last_in_this_page = false;	-p singlefs-harness --test instance_table_chain_read -- the_same_instance_on_both_pages	the_same_instance_on_both_pages_reddens_only_the_instance_table_invariant
-实例表第二片：checker 不判链指针记录合不合法	crates/singlefs-checker/src/walk.rs	                .judge("I-3.8", chain != ChainRecordView::Malformed, || {	                .judge("I-3.8", true, || {	-p singlefs-harness --test instance_table_chain_read -- contradictory_chain_record	contradictory_chain_record_reddens_only_the_instance_table_invariant
+实例表第二片：checker 不沿链走（链指针指到别的单元不红）	crates/singlefs-checker/src/walk.rs	                ChainRecordView::NextPage(next_page) => {\n                    pages.push(records);\n                    pointer = next_page;\n                    page_index += 1;\n                }	                ChainRecordView::NextPage(_next_page) => {\n                    pages.push(records);\n                    break;\n                }	-p singlefs-harness --test instance_table_chain_read -- chain_pointer_to_another	chain_pointer_to_another_instance_table_unit_reddens_the_identity_invariant
+实例表第二片：checker 不沿链走（链指针校验和不对不红）	crates/singlefs-checker/src/walk.rs	                ChainRecordView::NextPage(next_page) => {\n                    pages.push(records);\n                    pointer = next_page;\n                    page_index += 1;\n                }	                ChainRecordView::NextPage(_next_page) => {\n                    pages.push(records);\n                    break;\n                }	-p singlefs-harness --test instance_table_chain_read -- chain_pointer_whose_checksum	chain_pointer_whose_checksum_does_not_match_the_second_page_reddens_the_checksum_invariant
+实例表第二片：checker 按片各判行唯一，不按整张表	crates/singlefs-checker/src/walk.rs	        for records in pages {\n            for row in records {	        for records in pages {\n            instances.clear();\n            for row in records {	-p singlefs-harness --test instance_table_chain_read -- the_same_instance_on_both_pages	the_same_instance_on_both_pages_reddens_only_the_instance_table_invariant
+实例表第二片：checker 不判链指针记录合不合法	crates/singlefs-checker/src/walk.rs	                chain != ChainRecordView::Malformed,	                true,	-p singlefs-harness --test instance_table_chain_read -- contradictory_chain_record	contradictory_chain_record_reddens_only_the_instance_table_invariant
 实例表第二片：读者收下位置指针没清零的「无下一片」	crates/singlefs-core/src/instance_table.rs	            CHAIN_RECORD_NO_NEXT_PAGE => pointer_is_zero.then_some(Self::LastPage),	            CHAIN_RECORD_NO_NEXT_PAGE => Some(Self::LastPage),	-p singlefs-core --lib -- chain_record_tests	chain_record_parse_accepts_only_a_zeroed_last_page_or_a_real_next_page_pointer
 实例表第二片：读者收下位置指针全零的「有下一片」	crates/singlefs-core/src/instance_table.rs	                if pointer_is_zero {\n                    return None;\n                }	                if false && pointer_is_zero {\n                    return None;\n                }	-p singlefs-core --lib -- chain_record_tests	chain_record_parse_accepts_only_a_zeroed_last_page_or_a_real_next_page_pointer
 实例表第二片：读者不核一片的身份四元组（链指到别的片也收）	crates/singlefs-core/src/instance_table.rs	        if unit.identity != page_index.packed_identity() {	        if false && unit.identity != page_index.packed_identity() {	-p singlefs-core --lib -- chain_record_tests	each_page_parses_only_as_the_page_its_container_number_names_and_a_chained_first_page_is_not_a_whole_table
@@ -2901,29 +2901,13 @@
 第 24 批 GPU 判器 N4：墓碑树、谱系树类型段 = 记录类型的条目不按孩子头里的类认（根环转过一圈之后层级 2 墓碑树根条目类型段 1 指码 2 节点那条流上 GPU 按 32 KiB 读那个节点、I-2.1 红、记走读失败，CPU 不）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        if (class_of_the_child_of_a_container_entry != NONE) {	        if (false && class_of_the_child_of_a_container_entry != NONE) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root_after_the_root_ring_turned_over	the_gpu_judge_agrees_with_the_cpu_when_a_tombstone_root_entry_of_the_container_type_names_a_node_below_the_root_after_the_root_ring_turned_over
 第 24 批 GPU 判器 N4：inode 树内部条目不按孩子头里的类认（inode 树根条目类型段 0 指码 3 叶容器那条流上 CPU 红 I-9.2，GPU 照旧把类型段 0 的孩子整个状态判不了）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        if (class_of_the_inode_child != NONE) {	        if (false && class_of_the_inode_child != NONE) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container	the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_type_zero_names_a_leaf_container
 第 24 批 GPU 判器 I-9.29：最新根的实例表读不出时照旧比同号出生代（被抛弃的根与现行线同号不同出生代、最新根的实例表两份翻坏那条流上 CPU 报部分没判，GPU 红 I-9.29）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    d_object_births_are_compared = sw(H_BASE + H_NEWEST_INSTANCE_TABLE_CHAIN_REACHED_ITS_LAST_PAGE) == 1u;	    d_object_births_are_compared = true;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back	the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_naming_another_object_birth_is_not_compared_once_the_newest_instance_table_does_not_read_back
-第 25 批 I-9.29 门（CPU）：判不了的门改回另用只读不判的读法读最新根的实例表（攻方 N5-a：链指针记录之前多一条 kind 2，主走读走到了末片、候选集滤掉了被抛弃的 C，门却报部分没判）	crates/singlefs-checker/src/walk.rs	    let abandoned_roots_filtering = walk.abandoned_roots_filtering;	    let abandoned_roots_filtering = match (walk.abandoned_roots_filtering, instance_table_rows_of_root_without_judging(reader, &roots[newest_index].2.record_bytes)) { (_, Some(_)) => AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable, (_, None) => AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable };	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_still_filters_the_abandoned_root_so_the_one_birth_invariant_holds
 第 25 批 I-9.29 门（CPU）：判不了的门改回另用只读不判的读法读最新根的实例表（攻方 N5-c：载荷 CRC 对不上、整单元校验和对得上，主走读照用这一片的行，门却报部分没判）	crates/singlefs-checker/src/walk.rs	    let abandoned_roots_filtering = walk.abandoned_roots_filtering;	    let abandoned_roots_filtering = match (walk.abandoned_roots_filtering, instance_table_rows_of_root_without_judging(reader, &roots[newest_index].2.record_bytes)) { (_, Some(_)) => AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable, (_, None) => AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable };	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_payload_crc_mismatch_on_the_newest_instance_table_reddens_only_the_payload_crc_and_still_filters_the_abandoned_root_so_the_one_birth_invariant_holds
-第 25 批 I-9.29 门（CPU）：最新根的实例表链指针记录不合法也当主走读走到了末片（攻方 N5-b：分不出后面还有没有片，I-9.29 报成立而不是部分没判）	crates/singlefs-checker/src/walk.rs	                ChainRecordView::Malformed => break,\n            }\n        }\n        self.walk_parent_unit = parent_unit_when_the_chain_starts;	                ChainRecordView::Malformed => {\n                    if mount_root_instance_when_newest.is_some() {\n                        self.abandoned_roots_filtering = AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable;\n                    }\n                    break;\n                }\n            }\n        }\n        self.walk_parent_unit = parent_unit_when_the_chain_starts;	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_malformed_chain_record_on_the_newest_instance_table_leaves_the_one_birth_invariant_partially_judged
-第 25 批 GPU 判器 I-9.29 对拍 N5-a 那条流的 CPU 一侧：判不了的门改回另用只读不判的读法（整条落盘的镜像上 I-9.29 报部分没判，不是成立）	crates/singlefs-checker/src/walk.rs	    let abandoned_roots_filtering = walk.abandoned_roots_filtering;	    let abandoned_roots_filtering = match (walk.abandoned_roots_filtering, instance_table_rows_of_root_without_judging(reader, &roots[newest_index].2.record_bytes)) { (_, Some(_)) => AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable, (_, None) => AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable };	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_is_filtered_by_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind	the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_is_filtered_by_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind
-第 25 批 GPU 判器 I-9.29：走读内核沿最新根的实例表链走到末片也不记（扫描内核那张表恒不比；N5-a 那条流上 C 还是最新根的状态 CPU 红 I-9.29、GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                w_newest_instance_table_chain_reached_its_last_page = true;	                w_newest_instance_table_chain_reached_its_last_page = false;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_is_filtered_by_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind	the_gpu_judge_agrees_with_the_cpu_that_an_abandoned_root_is_filtered_by_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind
+第 25 批 I-9.29 门（CPU）：最新根的实例表链指针记录不合法也当主走读走到了末片（攻方 N5-b：分不出后面还有没有片，I-9.29 报成立而不是部分没判）	crates/singlefs-checker/src/walk.rs	                ChainRecordView::Malformed => {\n                    walk_stopped_at_a_page_that_does_not_decode = true;\n                    break;\n                }\n            }\n        }\n        self.walk_parent_unit = parent_unit_when_the_chain_starts;	                ChainRecordView::Malformed => {\n                    walk_stopped_at_a_page_that_does_not_decode = true;\n                    if mount_root_instance_when_newest.is_some() {\n                        self.abandoned_roots_filtering = AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable;\n                    }\n                    break;\n                }\n            }\n        }\n        self.walk_parent_unit = parent_unit_when_the_chain_starts;	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_malformed_chain_record_on_the_newest_instance_table_leaves_the_one_birth_invariant_partially_judged
+第 25 批 GPU 判器 I-9.29：走读内核沿最新根的实例表链走到末片也不记（扫描内核那张表恒不比；N5-a 那条流上 C 还是最新根的状态 CPU 红 I-9.29、GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	                w_newest_instance_table_chain_reached_its_last_page = true;	                w_newest_instance_table_chain_reached_its_last_page = false;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind_does_not_decode	the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind_does_not_decode
 第10i批 S1 两遍里只有一遍自证过的根改回记成知道住着根的槽（窄盘起点抬 F 重读它仍坏就拒，挂载在写行、暖机之后报 FloorRaiseFailedAfterTheMountsPublishes）	crates/singlefs-core/src/mount.rs	            let self_verified_on_only_one_of_the_two_passes =\n                ring_slots_self_verified_on_only_one_of_the_two_passes.contains(ring_slot);	            let self_verified_on_only_one_of_the_two_passes = false;	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	a_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_does_not_refuse_the_floor_raise_of_the_narrow_mount
-第10i批 S1 抬 F 算上限读它读坏一次就拒、不重读（瞬时读坏一次的那一形也在写行、暖机之后报 FloorRaiseFailedAfterTheMountsPublishes）	crates/singlefs-core/src/mount.rs	        if ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread.contains(&ring_slot) {\n            match read_once(ring_slot) {\n                RootRingSlotReading::SelfVerified(root) => roots.push(root),\n                RootRingSlotReading::Bad(_still_bad) => {\n                    ring_slots_bad_on_this_read_taken_as_holding_no_root.insert(ring_slot);\n                }\n            }\n            continue;\n        }	        if ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread.contains(&ring_slot) {\n            return Err(\n                MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {\n                    ring_slot,\n                    first_reading,\n                    reread: first_reading,\n                },\n            );\n        }	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	a_root_self_verified_only_on_the_rebuild_pass_read_bad_once_more_after_the_reread_is_taken_as_read_by_the_floor_raise
-第10i批 S1 抬 F 算上限读它重读仍坏照拒（按知道住着根的槽处置）	crates/singlefs-core/src/mount.rs	        if ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread.contains(&ring_slot) {	        if false && ring_slots_taken_as_bad_at_mount_when_still_bad_after_one_reread.contains(&ring_slot) {	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	a_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_does_not_refuse_the_floor_raise_of_the_narrow_mount
+第10i批 S1 抬 F 算上限读它读坏一次就拒、不重读（瞬时读坏一次的那一形也在写行、暖机之后报 FloorRaiseFailedAfterTheMountsPublishes）	crates/singlefs-core/src/mount.rs	        if let Some(root_record_self_verified_on_one_rebuild_pass) =\n            root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass.get(&ring_slot)\n        {\n            match read_once(ring_slot) {\n                RootRingSlotReading::SelfVerified(root) => roots.push(root),\n                RootRingSlotReading::Bad(_still_bad) => {\n                    roots.push(*root_record_self_verified_on_one_rebuild_pass);\n                }\n            }\n            continue;\n        }	        if root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass.contains_key(&ring_slot) {\n            return Err(\n                MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {\n                    ring_slot,\n                    first_reading,\n                    reread: first_reading,\n                },\n            );\n        }	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	a_root_self_verified_only_on_the_rebuild_pass_read_bad_once_more_after_the_reread_is_taken_as_read_by_the_floor_raise
+第10i批 S1 抬 F 算上限读它重读仍坏照拒（按知道住着根的槽处置）	crates/singlefs-core/src/mount.rs	            root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass.get(&ring_slot)\n        {	            root_records_of_ring_slots_self_verified_on_only_one_rebuild_pass.get(&ring_slot).filter(|_| false)\n        {	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	a_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_does_not_refuse_the_floor_raise_of_the_narrow_mount
 第10i批 S1 抬 F 重算回收门槛那一遍读它重读仍坏照拒（不从知道住着根的槽里拿掉）	crates/singlefs-core/src/mount.rs	                !ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass\n                    .contains_key(ring_slot)	                true || !ring_slots_with_a_root_self_verified_on_only_one_rebuild_pass\n                    .contains_key(ring_slot)	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	a_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_does_not_refuse_the_floor_raise_of_the_narrow_mount
-第10i批 R1 两维：重读之后的读数只剩最新那条根（单元区 384 槽、覆盖写 3 次，重读只坏 C 的根槽报读错：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites
-第10i批 R1 两维：重读之后的读数只剩最新那条根（单元区 384 槽、覆盖写 3 次，重读只坏 C 的根槽读回全 0：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites
-第10i批 R1 两维：重读之后的读数只剩最新那条根（单元区 384 槽、覆盖写 3 次，重读只坏 C 的根槽读回乱字节：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites
-第10i批 R1 两维：重读之后的读数只剩最新那条根（单元区 384 槽、覆盖写 3 次，反方向：第一遍 C 的根槽也读坏、重读读得出：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites
-第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 1 次，重读只坏 C 的根槽报读错：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite
-第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 1 次，重读只坏 C 的根槽读回全 0：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite
-第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 1 次，重读只坏 C 的根槽读回乱字节：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite
-第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 1 次，反方向：第一遍 C 的根槽也读坏、重读读得出：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite
-第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 3 次，重读只坏 C 的根槽报读错：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites
-第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 3 次，重读只坏 C 的根槽读回全 0：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites
-第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 3 次，重读只坏 C 的根槽读回乱字节：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites
-第10i批 R1 两维：重读之后的读数只剩最新那条根（4 GiB、覆盖写 3 次，反方向：第一遍 C 的根槽也读坏、重读读得出：C 掉出影子账、两块盘隔离 0）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites
-第10i批 rr2 用例丢盘整读补全零与乱字节：重读之后的读数只剩最新那条根（264 槽起点、盘 1 读回全 0：当场放开越过环里更旧的有效根）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	on_the_264_slot_pool_with_a_live_file_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before
-第10i批 rr2 用例丢盘整读补全零与乱字节：重读之后的读数只剩最新那条根（264 槽起点、盘 0 读回乱字节：当场放开越过环里更旧的有效根）	crates/singlefs-core/src/mount.rs	                (\n                    readings_of_the_two_passes_merged,\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	                (\n                    readings_of_the_two_passes_merged.into_iter().filter(|(_, reading)| matches!(reading, RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root) if root.checkpoint_txg == window_anchor)).collect::<Vec<_>>(),\n                    window_anchor,\n                    RootRingOfTheRebuiltAllocatorRead::OnTheOneReread,\n                )	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	on_the_264_slot_pool_with_a_live_file_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before
 第10i批 慢档 plain 逐步核方向：进程内回收门槛在 txg 40 那一次越过环里全部有效根（中间一步记账少于遍历，I-3.1 红在少的一边）	crates/singlefs-core/src/allocator.rs	        let reclaim_floor = crate::mount::reclaim_floor(\n            occupancy.effective_floor,\n            occupancy.oldest_valid_root_in_the_ring(),\n        );	        let reclaim_floor = if checkpoint_txg.0 == 40 {\n            checkpoint_txg\n        } else {\n            crate::mount::reclaim_floor(\n                occupancy.effective_floor,\n                occupancy.oldest_valid_root_in_the_ring(),\n            )\n        };	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored plain_overwrite_whose_root_slot_write_zeroes the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root	plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced
 第10i批 慢档 torn 逐步核方向：进程内回收门槛在 txg 40 那一次越过环里全部有效根（中间一步记账少于遍历，I-3.1 红在少的一边）	crates/singlefs-core/src/allocator.rs	        let reclaim_floor = crate::mount::reclaim_floor(\n            occupancy.effective_floor,\n            occupancy.oldest_valid_root_in_the_ring(),\n        );	        let reclaim_floor = if checkpoint_txg.0 == 40 {\n            checkpoint_txg\n        } else {\n            crate::mount::reclaim_floor(\n                occupancy.effective_floor,\n                occupancy.oldest_valid_root_in_the_ring(),\n            )\n        };	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored plain_overwrite_whose_root_slot_write_zeroes the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root	the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_torn_after_the_first_half_is_resent_in_process_or_replayed_after_a_crash
 第10i批 慢档 zeroed 逐步核方向：进程内回收门槛在 txg 40 那一次越过环里全部有效根（中间一步记账少于遍历，I-3.1 红在少的一边）	crates/singlefs-core/src/allocator.rs	        let reclaim_floor = crate::mount::reclaim_floor(\n            occupancy.effective_floor,\n            occupancy.oldest_valid_root_in_the_ring(),\n        );	        let reclaim_floor = if checkpoint_txg.0 == 40 {\n            checkpoint_txg\n        } else {\n            crate::mount::reclaim_floor(\n                occupancy.effective_floor,\n                occupancy.oldest_valid_root_in_the_ring(),\n            )\n        };	-p singlefs-harness --test rollback_by_a_forward_publish -- --include-ignored plain_overwrite_whose_root_slot_write_zeroes the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root	the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_is_resent_in_process_or_replayed_after_a_crash
@@ -2987,3 +2971,49 @@
 第29批 GPU 判器 H-J3a：走读不往下读的孩子两份都对不上不记走读失败（类型段 7、两条位置条目校验和都坏那条流上 CPU 红 I-7.2，GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	    if (!first_matches && !second_matches) {	    if (false && !first_matches && !second_matches) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong	the_gpu_judge_agrees_with_the_cpu_when_an_inode_root_entry_of_an_unregistered_type_names_a_child_whose_two_location_checksums_are_wrong
 第29批 GPU 判器 O-1：扫描内核的实例表链上限与走读内核的不同（扫描内核读满 255 片就记判不了）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 256u;	const INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED: u32 = 255u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_walk_kernel_and_the_scan_kernel_follow_instance_table_chains_up_to_the_same_number_of_pages	the_walk_kernel_and_the_scan_kernel_follow_instance_table_chains_up_to_the_same_number_of_pages
 第29批 GPU 判器 K1 J2：实例表链某一片读不出也当走到末片（最新根实例表第 1 片两份都读不出那条关门流上门开了，GPU 红 I-9.29、CPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        let s = w_read_referenced_unit(pointer, DATA_UNIT_BYTES);\n        if (s.version == NONE) {\n            w_failures = w_failures + 1u;\n            break;\n        }	        let s = w_read_referenced_unit(pointer, DATA_UNIT_BYTES);\n        if (s.version == NONE) {\n            w_failures = w_failures + 1u;\n            if (judge_rows) {\n                w_newest_instance_table_chain_reached_its_last_page = true;\n            }\n            break;\n        }	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_second_page_of_the_newest_instance_table_does_not_read_back	the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_second_page_of_the_newest_instance_table_does_not_read_back
+第 10j 批 I-3.14：实例表一片里没登记的 kind（攻方 N5-a 那一形）恒判成立	crates/singlefs-checker/src/walk.rs	unregistered_kinds.is_empty(),	true,	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set
+第 10j 批 I-3.14：链指针记录出现在一片中间恒判成立	crates/singlefs-checker/src/walk.rs	chain_records_before_the_last.is_empty(),	true,	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_second_chain_record_in_the_middle_of_the_newest_instance_table_page_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set
+第 10j 批 I-3.14：kind 0 行 flags 非 0 恒判成立	crates/singlefs-checker/src/walk.rs	rows_with_flags_off_the_first_version.is_empty(),	true,	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_row_whose_flags_are_not_zero_in_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set
+第 10k 批 I-3.14：记录形态不合法的一片照收照走、不当解不开（攻方 N5-a 那张镜像上 C 被滤掉，I-9.29 报成立）	crates/singlefs-checker/src/walk.rs	                InstanceTablePageRecordForms::IllFormedSoThePageDoesNotDecode => {\n                    walk_stopped_at_a_page_that_does_not_decode = true;\n                    break;\n                }	                InstanceTablePageRecordForms::IllFormedSoThePageDoesNotDecode => {\n                    walk_stopped_at_a_page_that_does_not_decode = true;\n                }	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_leaves_the_one_birth_invariant_partially_judged
+第 10k 批 I-3.14：没登记的 kind 那一片仍当解得开（只红 I-3.14，C 被滤掉）	crates/singlefs-checker/src/walk.rs	    if unregistered_kinds.is_empty()\n        && chain_records_before_the_last.is_empty()	    if true\n        && chain_records_before_the_last.is_empty()	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set
+第 10k 批 I-3.14：片中间有链指针记录的那一片仍当解得开（只红 I-3.14，C 被滤掉）	crates/singlefs-checker/src/walk.rs	        && chain_records_before_the_last.is_empty()\n        && rows_with_flags_off_the_first_version.is_empty()	        && true\n        && rows_with_flags_off_the_first_version.is_empty()	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_second_chain_record_in_the_middle_of_the_newest_instance_table_page_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set
+第 10k 批 I-3.14：行 flags 非 0 的那一片仍当解得开（只红 I-3.14，C 被滤掉）	crates/singlefs-checker/src/walk.rs	        && rows_with_flags_off_the_first_version.is_empty()\n    {	        && true\n    {	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_row_whose_flags_are_not_zero_in_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set
+第 10k 批 I-3.14：最后一条不是合法链指针记录的那一片照收它的行（攻方 N5-b 那张镜像上 C 被滤掉，只红 I-3.14）	crates/singlefs-checker/src/walk.rs	                ChainRecordView::Malformed => {\n                    walk_stopped_at_a_page_that_does_not_decode = true;\n                    break;\n                }	                ChainRecordView::Malformed => {\n                    pages.push(records);\n                    walk_stopped_at_a_page_that_does_not_decode = true;\n                    break;\n                }	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_malformed_chain_record_on_the_newest_instance_table_leaves_the_one_birth_invariant_partially_judged
+第 10k 批 只读不判的读法不看行的 flags（flags 非 0 的那一片照收，I-3.12 判那一行）	crates/singlefs-checker/src/walk.rs	            if row[0] != 0\n                || row[INSTANCE_TABLE_ROW_FLAGS_OFFSET]\n                    != INSTANCE_TABLE_ROW_FLAGS_OF_THE_FIRST_VERSION\n            {	            if row[0] != 0\n            {	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_row_whose_flags_are_not_zero_leaves_the_newest_instance_table_unread_by_the_reader_without_judging
+第 10k 批 攻方 H-b：实例表链「走过」按第 0 条位置条目认（盘 0 那条指回第 0 片时第 1 片被当成走过、行丢了）	crates/singlefs-checker/src/walk.rs	            if !self.visited_units.insert(placement_it_was_read_from) {	            if !self.visited_units.insert(page_placement) {	-p singlefs-harness --test checker_known_bad_images -- a_second_instance_table_page_whose_pointer_names_the_first_page_on_device_zero_is_walked_from_the_copy_read_back	a_second_instance_table_page_whose_pointer_names_the_first_page_on_device_zero_is_walked_from_the_copy_read_back
+第 10k 批 GPU 判器 I-3.14：kind 只有 0 或 1 那一格恒判成立	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        w_judge(INVARIANT_I_3_14, kinds_are_registered);	        w_judge(INVARIANT_I_3_14, true);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind_does_not_decode	the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind_does_not_decode
+第 10k 批 GPU 判器 I-3.14：链指针记录只在最后一条那一格恒判成立	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        w_judge(INVARIANT_I_3_14, chain_records_only_last);	        w_judge(INVARIANT_I_3_14, true);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_a_chain_record_in_the_middle_does_not_decode	the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_a_chain_record_in_the_middle_does_not_decode
+第 10k 批 GPU 判器 I-3.14：行 flags 为 0 那一格恒判成立	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        w_judge(INVARIANT_I_3_14, row_flags_are_zero);	        w_judge(INVARIANT_I_3_14, true);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_row_flags_not_zero_does_not_decode	the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_with_row_flags_not_zero_does_not_decode
+第 10k 批 GPU 判器 I-3.14：记录形态不合法的一片照收照走（行收进 C_INSTANCE_ROWS、走到末片）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        if (!(kinds_are_registered && chain_records_only_last && row_flags_are_zero)) {\n            break;\n        }	        if (false) {\n            break;\n        }	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind_does_not_decode	the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind_does_not_decode
+第 10k 批 C704 挂载透明：清扫发布那一版里仍有它删掉的行也当成都不在（放回再比，不判红）	crates/singlefs-checker-tier/src/crash_injection.rs	    if !still_present.is_empty() {\n        return InstanceTableRowsAgainstTheSweepPublish::DeletedRowsStillPresent(still_present);	    if false {\n        return InstanceTableRowsAgainstTheSweepPublish::DeletedRowsStillPresent(still_present);	-p singlefs-checker-tier --lib -- crash_injection::tests::a_sweep_publish_still_holding_the_rows_it_deleted_is_a_mount_transparency_violation_counted_once	a_sweep_publish_still_holding_the_rows_it_deleted_is_a_mount_transparency_violation_counted_once
+第 10k 批 C704 挂载透明：清扫发布那一版里仍有它删掉的行判红但不进按字段分的判红计数	crates/singlefs-checker-tier/src/crash_injection.rs	                .entry(MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH)\n                .or_insert(0) += 1;	                .entry(MOUNT_TRANSPARENCY_VIOLATION_ROWS_DELETED_BY_THE_SWEEP_STILL_IN_ITS_PUBLISH)\n                .or_insert(0) += 0;	-p singlefs-checker-tier --lib -- crash_injection::tests::a_sweep_publish_still_holding_the_rows_it_deleted_is_a_mount_transparency_violation_counted_once	a_sweep_publish_still_holding_the_rows_it_deleted_is_a_mount_transparency_violation_counted_once
+第 10k 批 C704 挂载透明：被删的行都不在时放回不接上被删的行（交回的 R2 只有留下的行）	crates/singlefs-checker-tier/src/crash_injection.rs	        .chain(&rows_deleted_by_the_sweep.rows_deleted)\n	        .chain(std::iter::empty())\n	-p singlefs-checker-tier --lib -- crash_injection::tests::a_sweep_publish_without_the_rows_it_deleted_is_compared_with_them_put_back	a_sweep_publish_without_the_rows_it_deleted_is_compared_with_them_put_back
+第 10n 批 引用集合那一遍：实例表一片里 kind 不认得、片中间有链指针记录、行 flags 非 0 照记引用齐全（攻方 N5 那张镜像上多红 I-3.9）	crates/singlefs-checker/src/walk.rs	        if records_before_the_last.iter().any(|record| {	        if false && records_before_the_last.iter().any(|record| {	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_record_of_an_unregistered_kind_before_the_chain_record_of_the_newest_instance_table_does_not_decode_and_leaves_the_abandoned_root_in_the_candidate_set
+第 10n 批 I-3.8：最新根的实例表第 0 片解不开时落回缺省理由（不说是哪一片解不开、行没收全）	crates/singlefs-checker/src/walk.rs	            } else if walk_stopped_at_a_page_that_does_not_decode {	            } else if false {	-p singlefs-harness --test checker_known_bad_images -- the_newest_instance_table	a_malformed_chain_record_on_the_newest_instance_table_leaves_the_one_birth_invariant_partially_judged
+第 10n 批 GPU 扫描内核 r_instance_rows 不判行的 flags（flags 非 0 的那一片照收，I-3.12 判那一行，CPU 不判）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	            if (rd8(s.base, o) != 0u || rd8(s.base, o + 21u) != 0u) {	            if (rd8(s.base, o) != 0u) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_row_flags_are_not_zero_is_not_read_for_the_row_txg_invariant	the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_row_flags_are_not_zero_is_not_read_for_the_row_txg_invariant
+第 10n 批 GPU 恢复内核 load_instance_table_of_the_effective_root 不判行的 flags（flags 非 0 的那张表照收，按它判 C 那几次发布被抛弃）	crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl	            if (rd8(base, o) != 0u || rd8(base, o + 21u) != 0u) {	            if (rd8(base, o) != 0u) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_row_flags_are_not_zero_is_not_read_for_the_row_txg_invariant	the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_whose_row_flags_are_not_zero_is_not_read_for_the_row_txg_invariant
+第 10n 批 GPU 扫描内核引用集合那一遍：实例表一片里 kind 不认得、片中间有链指针记录、行 flags 非 0 照记引用齐全	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	            if (rd8(packed.base, o) != 0u || rd8(packed.base, o + 21u) != 0u) {	            if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind_does_not_decode	the_gpu_judge_agrees_with_the_cpu_that_a_newest_instance_table_carrying_a_record_of_an_unregistered_kind_does_not_decode
+第 10n 批 GPU 判器 攻方 H-b：实例表链「走过」按第 0 条位置条目认（盘 0 那条指回第 0 片时第 1 片被当成走过、门关）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        if (!w_visit(device_read_from, slot_read_from)) {	        if (!w_visit(pointer.device0, pointer.slot0)) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_a_second_instance_table_page_whose_pointer_on_device_zero_names_the_first_page_is_walked_from_the_copy_read_back	the_gpu_judge_agrees_with_the_cpu_that_a_second_instance_table_page_whose_pointer_on_device_zero_names_the_first_page_is_walked_from_the_copy_read_back
+第 10n 批 GPU 判器 I-3.14：最后一条是合法链指针记录那一格恒判成立（第 27 批链指针记录不合法那条流上 CPU 红 I-3.14、GPU 不红）	crates/singlefs-checker-tier/src/crash_judge_checker.wgsl	        w_judge(INVARIANT_I_3_14, chain != 2u);	        w_judge(INVARIANT_I_3_14, true);	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_chain_record_of_the_newest_instance_table_is_invalid	the_gpu_judge_agrees_with_the_cpu_that_the_one_birth_invariant_is_not_compared_once_the_chain_record_of_the_newest_instance_table_is_invalid
+第10l批 T1 甲 抬 F 重算影子账那一遍读坏重读仍坏的这一类不拿记着的根顶上（被抛弃的根 C 那一槽交替读回全 0：C 掉出影子账，第 9 次改动之后 I-7.4 红）	crates/singlefs-core/src/mount.rs	            RootRingSlotReading::Bad(_still_bad) => roots_with_ring_slots\n                .push((*ring_slot, *root_record_self_verified_on_one_rebuild_pass)),	            RootRingSlotReading::Bad(_still_bad) => {}	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	an_abandoned_root_self_verified_only_on_the_rebuild_pass_read_back_as_zeros_by_the_floor_raise_keeps_its_units_isolated
+第10l批 T1 甲 卸载那一串读坏重读仍坏的这一类不拿记着的根顶上（C 那一槽一直报读错：卸载之后 C 的实例表单元两份空闲）	crates/singlefs-core/src/mount.rs	            RootRingSlotReading::Bad(_still_bad) => roots_with_ring_slots\n                .push((*ring_slot, *root_record_self_verified_on_one_rebuild_pass)),	            RootRingSlotReading::Bad(_still_bad) => {}	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	unmounting_after_a_remount_with_an_abandoned_root_self_verified_only_on_the_rebuild_pass_keeps_its_units_isolated
+第10l批 T1 甲 算上限那一遍读坏重读仍坏的这一类照拒（被抛弃的根 C 那一槽一直报读错：抬 F 报 RollbackFloorCeilingRootRingSlotStillBadAfterOneReread、改动被拒）	crates/singlefs-core/src/mount.rs	                RootRingSlotReading::Bad(_still_bad) => {\n                    roots.push(*root_record_self_verified_on_one_rebuild_pass);\n                }	                RootRingSlotReading::Bad(reread) => {\n                    return Err(\n                        MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {\n                            ring_slot,\n                            first_reading,\n                            reread,\n                        },\n                    );\n                }	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	an_abandoned_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling
+第10l批 T1 甲 算上限那一遍读坏重读仍坏的这一类照拒（有效根那一槽）	crates/singlefs-core/src/mount.rs	                RootRingSlotReading::Bad(_still_bad) => {\n                    roots.push(*root_record_self_verified_on_one_rebuild_pass);\n                }	                RootRingSlotReading::Bad(reread) => {\n                    return Err(\n                        MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread {\n                            ring_slot,\n                            first_reading,\n                            reread,\n                        },\n                    );\n                }	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	a_valid_root_self_verified_only_on_the_rebuild_pass_unreadable_from_the_reread_onward_does_not_hold_the_ceiling
+第10l批 新类被抛弃那一半不算进环里还被引用的落点（4 GiB、覆盖写 1 次：环里最旧有效根越过 C 那一次回收出来的 C 的实例表单元空闲）	crates/singlefs-core/src/allocator.rs	                RootRingOccupant::AbandonedRoot {\n                    referenced_placements,\n                }\n                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                    root:\n                        RootReadOnOneRebuildPass::AbandonedRoot {\n                            referenced_placements,\n                        },\n                    ..\n                } => referenced_placements.as_slice(),	                RootRingOccupant::AbandonedRoot {\n                    referenced_placements,\n                } => referenced_placements.as_slice(),\n                RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                    root: RootReadOnOneRebuildPass::AbandonedRoot { .. },\n                    ..\n                } => &[],	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	an_abandoned_root_self_verified_only_on_the_rebuild_pass_holds_its_units_in_the_allocator_until_it_leaves_the_ring
+第10l批 新类被抛弃那一半被换下时不清隔离（4 GiB、覆盖写 1 次：C 离开根环之后它写出的落点仍隔离、不空闲）	crates/singlefs-core/src/allocator.rs	            Some(\n                RootRingOccupant::AbandonedRoot {\n                    referenced_placements,\n                }\n                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                    root:\n                        RootReadOnOneRebuildPass::AbandonedRoot {\n                            referenced_placements,\n                        },\n                    ..\n                },\n            ) => self.clear_isolation_no_abandoned_root_in_the_ring_backs(	            Some(RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::AbandonedRoot { .. },\n                ..\n            }) => {}\n            Some(RootRingOccupant::AbandonedRoot {\n                referenced_placements,\n            }) => self.clear_isolation_no_abandoned_root_in_the_ring_backs(	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	an_abandoned_root_self_verified_only_on_the_rebuild_pass_holds_its_units_in_the_allocator_until_it_leaves_the_ring
+第10l批 读序：算上限之前多读一遍根环（瞬时那条用例点名的那一次读不再是写之后第一遍、当场重读的那一次）	crates/singlefs-core/src/mount.rs	    let ceiling = rollback_floor_ceiling_of_this_process(\n        devices,\n        &caller_inputs_agreeing_with_the_disk.system_configuration,	    let _ring_read_once_more_before_the_ceiling =\n        choose_root(&*devices, &caller_inputs_agreeing_with_the_disk.system_configuration);\n    let ceiling = rollback_floor_ceiling_of_this_process(\n        devices,\n        &caller_inputs_agreeing_with_the_disk.system_configuration,	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	a_root_self_verified_only_on_the_rebuild_pass_read_bad_once_more_after_the_reread_is_taken_as_read_by_the_floor_raise
+第10l批 R1 分得开：整份换成重读的读数（单元区 384 槽、覆盖写 3 次，重读只坏 C 的根槽报读错）	crates/singlefs-core/src/mount.rs	                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(\n                    &first_readings,\n                    readings_of_the_reread,\n                );	                let readings_of_the_two_passes_merged = readings_of_the_reread;	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites
+第10l批 R1 分得开：整份换成重读的读数（单元区 384 槽、覆盖写 3 次，重读只坏 C 的根槽读回全 0）	crates/singlefs-core/src/mount.rs	                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(\n                    &first_readings,\n                    readings_of_the_reread,\n                );	                let readings_of_the_two_passes_merged = readings_of_the_reread;	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites
+第10l批 R1 分得开：整份换成重读的读数（单元区 384 槽、覆盖写 3 次，重读只坏 C 的根槽读回乱字节）	crates/singlefs-core/src/mount.rs	                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(\n                    &first_readings,\n                    readings_of_the_reread,\n                );	                let readings_of_the_two_passes_merged = readings_of_the_reread;	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites
+第10l批 R1 分得开：第一遍读坏压过重读自证过的根（单元区 384 槽、覆盖写 3 次，反方向）	crates/singlefs-core/src/mount.rs	        (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(root_of_the_reread)) => {\n            SelfVerified(root_of_the_reread)\n        }	        (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(_)) => first_reading,	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_384_unit_slot_devices_after_three_overwrites
+第10l批 R1 分得开：整份换成重读的读数（4 GiB、覆盖写 1 次，重读只坏 C 的根槽报读错）	crates/singlefs-core/src/mount.rs	                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(\n                    &first_readings,\n                    readings_of_the_reread,\n                );	                let readings_of_the_two_passes_merged = readings_of_the_reread;	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite
+第10l批 R1 分得开：整份换成重读的读数（4 GiB、覆盖写 1 次，重读只坏 C 的根槽读回全 0）	crates/singlefs-core/src/mount.rs	                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(\n                    &first_readings,\n                    readings_of_the_reread,\n                );	                let readings_of_the_two_passes_merged = readings_of_the_reread;	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite
+第10l批 R1 分得开：整份换成重读的读数（4 GiB、覆盖写 1 次，重读只坏 C 的根槽读回乱字节）	crates/singlefs-core/src/mount.rs	                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(\n                    &first_readings,\n                    readings_of_the_reread,\n                );	                let readings_of_the_two_passes_merged = readings_of_the_reread;	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite
+第10l批 R1 分得开：第一遍读坏压过重读自证过的根（4 GiB、覆盖写 1 次，反方向）	crates/singlefs-core/src/mount.rs	        (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(root_of_the_reread)) => {\n            SelfVerified(root_of_the_reread)\n        }	        (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(_)) => first_reading,	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_one_overwrite
+第10l批 R1 分得开：整份换成重读的读数（4 GiB、覆盖写 3 次，重读只坏 C 的根槽报读错）	crates/singlefs-core/src/mount.rs	                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(\n                    &first_readings,\n                    readings_of_the_reread,\n                );	                let readings_of_the_two_passes_merged = readings_of_the_reread;	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites
+第10l批 R1 分得开：整份换成重读的读数（4 GiB、覆盖写 3 次，重读只坏 C 的根槽读回全 0）	crates/singlefs-core/src/mount.rs	                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(\n                    &first_readings,\n                    readings_of_the_reread,\n                );	                let readings_of_the_two_passes_merged = readings_of_the_reread;	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_zeros_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites
+第10l批 R1 分得开：整份换成重读的读数（4 GiB、覆盖写 3 次，重读只坏 C 的根槽读回乱字节）	crates/singlefs-core/src/mount.rs	                let readings_of_the_two_passes_merged = merged_readings_of_the_two_root_ring_passes(\n                    &first_readings,\n                    readings_of_the_reread,\n                );	                let readings_of_the_two_passes_merged = readings_of_the_reread;	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_reading_back_garbage_only_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites
+第10l批 R1 分得开：第一遍读坏压过重读自证过的根（4 GiB、覆盖写 3 次，反方向）	crates/singlefs-core/src/mount.rs	        (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(root_of_the_reread)) => {\n            SelfVerified(root_of_the_reread)\n        }	        (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(_)) => first_reading,	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_abandoned_roots_slot_failing_on_the_first_pass_and_read_on_the_reread_keeps_its_units_isolated_until_it_leaves_the_ring_on_four_gibibyte_devices_after_three_overwrites
+第10l批 R1 分得开：第一遍读坏压过重读自证过的根（rr2 用例丢盘整读，264 槽起点、盘 1 读回全 0）	crates/singlefs-core/src/mount.rs	        (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(root_of_the_reread)) => {\n            SelfVerified(root_of_the_reread)\n        }	        (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(_)) => first_reading,	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	on_the_264_slot_pool_with_a_live_file_device_one_reading_back_zeros_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before
+第10l批 R1 分得开：第一遍读坏压过重读自证过的根（rr2 用例丢盘整读，264 槽起点、盘 0 读回乱字节）	crates/singlefs-core/src/mount.rs	        (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(root_of_the_reread)) => {\n            SelfVerified(root_of_the_reread)\n        }	        (BadAndNotAllZero | AllZeroNeverWritten, SelfVerified(_)) => first_reading,	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	on_the_264_slot_pool_with_a_live_file_device_zero_reading_back_garbage_in_the_rebuild_pass_rereads_and_mounts_and_lone_survivors_read_as_before
+第10m批 T1 甲 算上限那一遍读坏重读仍坏的有效根当没有根	crates/singlefs-core/src/mount.rs	roots.push(*root_record_self_verified_on_one_rebuild_pass);	ring_slots_bad_on_this_read_taken_as_holding_no_root.insert(ring_slot);	-p singlefs-harness --test rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root	the_fourth_newest_non_empty_root_self_verified_only_on_the_rebuild_pass_and_unreadable_from_the_reread_onward_still_sets_the_ceiling_of_the_narrow_mounts_floor_raise
```
