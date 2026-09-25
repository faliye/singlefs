# 附录二：HEAD 之后打进主工作区的四份补丁，代码轮第一轮 diff（`git diff HEAD -- crates/` 到冻结副本；生成于 2026-09-25 01:38 JST / 2026-09-24 16:38 UTC）

基准：HEAD `e980a219f1834c638cd2fae18b25a60525a6bd52`。冻结副本：`/tmp/claude-1000/m2-final-code-r1/tree/crates/`（腿读代码一律读这份副本，不读主工作区——主工作区在腿跑着的时候还会被别的实现员改）。

diff 原始文件：`/tmp/claude-1000/m2-final-code-r1/head-to-frozen.diff`，sha256 `227e8300f9a2b3656c9202668c171508dd65cb9aec59bfbd222e5051091ad924`，4658 行；由主 agent 给出，主 agent 已排除 E158 装置（`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`——它的对错由那个实验页的变异表与单测判，不进这一轮六格）。这份 diff 是 `git diff HEAD` 到该冻结副本 `crates/` 的差，材料员未重新生成、原样落盘。

## 一、diff（crates/ 下，HEAD `e980a219` 与冻结副本之间的差，共 18 个文件；没有 `new file mode` 或 `deleted file mode` 行——全部是对既有文件的修改，没有新增或删除的文件，因此本附录没有「新文件全文」一节）

```diff
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index 44a8753..f069a73 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -34,12 +34,12 @@ pub enum InvariantVerdict {
 }
 
 /// 第一版 checker 判的不变量，按这个次序报；每次都全部报出来，没评估到的报「不适用」。
-pub const IMPLEMENTED_INVARIANTS: [&str; 41] = [
+pub const IMPLEMENTED_INVARIANTS: [&str; 44] = [
     "I-1.1", "I-1.2", "I-1.3", "I-1.4", "I-1.6", "I-1.7", "I-1.8", "I-1.10", "I-2.1", "I-2.3",
     "I-2.4", "I-2.5", "I-3.1", "I-3.8", "I-3.9", "I-3.10", "I-3.11", "I-4.2", "I-4.8", "I-5.1",
-    "I-5.2", "I-5.4", "I-7.1", "I-7.2", "I-7.3", "I-7.4", "I-7.6", "I-7.7", "I-7.8", "I-8.6",
-    "I-8.7", "I-8.8", "I-9.1", "I-9.2", "I-9.4", "I-9.6", "I-9.7", "I-9.10", "I-9.12", "I-9.13",
-    "I-9.14",
+    "I-5.2", "I-5.4", "I-7.1", "I-7.2", "I-7.3", "I-7.4", "I-7.6", "I-7.7", "I-7.8", "I-7.9",
+    "I-8.6", "I-8.7", "I-8.8", "I-8.9", "I-9.1", "I-9.2", "I-9.4", "I-9.6", "I-9.7", "I-9.10",
+    "I-9.12", "I-9.13", "I-9.14", "I-9.15",
 ];
 
 /// 判定累加器：每条不变量记评估了几次、第一处违例、以及整条不适用的理由。
diff --git a/crates/singlefs-checker/src/lib.rs b/crates/singlefs-checker/src/lib.rs
index ff1debd..6c5a433 100644
--- a/crates/singlefs-checker/src/lib.rs
+++ b/crates/singlefs-checker/src/lib.rs
@@ -497,9 +497,10 @@ pub fn packed_unit_view(unit: &[u8]) -> Result<PackedUnitView, Verdict> {
     })
 }
 
-/// journal 记录头的偏移（D23（journal 的角色与格式） 已定项 4 的字段表）：事务号 8 与提交标记 1 之后紧跟本次发布内序号 4，
-/// 再是反向链 4、载荷校验和 4、新根段 188、fsid 8、MAC 16，头到 311 为止。
+/// journal 记录头的偏移（D23（journal 的角色与格式） 已定项 4 的字段表）：magic 4 + 类型 2 + 算法类型 1 之后是记录标志 1；
+/// 事务号 8 与提交标记 1 之后紧跟本次发布内序号 4，再是反向链 4、载荷校验和 4、新根段 188、fsid 8、MAC 16，头到 311 为止。
 const JOURNAL_MAGIC: &[u8; 4] = b"SFSJ";
+const JOURNAL_RECORD_FLAGS_OFFSET: usize = 7;
 const JOURNAL_HEADER_CHECKSUM_OFFSET: usize = 46;
 const JOURNAL_TRANSACTION_OFFSET: usize = 78;
 const JOURNAL_COMMIT_MARKER_OFFSET: usize = 86;
@@ -545,7 +546,11 @@ pub struct JournalRecordView {
     /// 所以不能只留 `is_commit`——那一步把 2..=255 静默读成「不带」。
     pub commit_marker_byte: u8,
     /// 本次发布内序号（D23（journal 的角色与格式） 已定项 4）：一次发布 N 条记录依次是 1..N，只有一条时是 1。
+    /// 读到的原样：序号 0 在这里不拒，I-8.9（一次发布的记录序号连续且只有末条带标志） 判红。
     pub ordinal_within_publish: u32,
+    /// 记录标志那 1 字节原样（D23（journal 的角色与格式） 已定项 4 / 已定项 17）：位 0 = 本次发布末条，其余位只许 0。
+    /// 不在这里拒其余位非 0 的记录，I-8.9（一次发布的记录序号连续且只有末条带标志） 判红——拒了它就看不见。
+    pub record_flags_byte: u8,
     pub back_chain: u32,
     /// 新根段 188 字节原样（树表指针 86 + 映射根指针 86 + 树 ID 水位 8 + F 8）。
     pub new_root_segment: Vec<u8>,
@@ -630,6 +635,7 @@ pub fn check_journal_record(
         is_commit: record[JOURNAL_COMMIT_MARKER_OFFSET] == 1,
         commit_marker_byte: record[JOURNAL_COMMIT_MARKER_OFFSET],
         ordinal_within_publish: read_u32(record, JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET),
+        record_flags_byte: record[JOURNAL_RECORD_FLAGS_OFFSET],
         back_chain: read_u32(record, JOURNAL_BACK_CHAIN_OFFSET),
         new_tree_identifier_watermark: read_u64(record, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 86 + 86),
         new_rollback_floor: read_u64(record, JOURNAL_NEW_ROOT_SEGMENT_OFFSET + 86 + 86 + 8),
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 233a81d..1fc39d7 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -18,9 +18,10 @@ use crate::image::{
     ImageReader, InvariantVerdict, Judgements, PointerView, PoolGeometry,
 };
 use crate::{
-    back_chain_of_record_header, check_index_node_keys, check_journal_record, checksum_field_holds,
-    crc32_castagnoli_table, index_node_view, key_schema_for_tree_kind, read_six_byte_unsigned,
-    read_u16, read_u32, read_u64, KEY_SCHEMA_ALLOCATION, KEY_SCHEMA_MAPPING, KEY_SCHEMA_TREE_TABLE,
+    back_chain_of_record_header, check_index_node_keys, check_journal_record, check_unit,
+    checksum_field_holds, crc32_castagnoli_table, index_node_view, key_schema_for_tree_kind,
+    packed_unit_view, read_six_byte_unsigned, read_u16, read_u32, read_u64, KEY_SCHEMA_ALLOCATION,
+    KEY_SCHEMA_MAPPING, KEY_SCHEMA_TREE_TABLE,
 };
 
 const TREE_KIND_EXTENT: u16 = 1;
@@ -45,6 +46,12 @@ const STATISTIC_DEFER_QUEUE_BYTES: u16 = 5;
 const STATISTIC_INODE_WATERMARK: u16 = 12;
 /// 不带设备维的统计量把设备段写成这个保留值（D5（快照 / 空间记账机制） 已定项 10）。
 const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;
+/// inode 记录 140 里 `size` 与 `blocks` 两个字段的偏移（D8（核心索引结构） 已定项 6 的偏移表），按字段表另写一份、不用实现的解析。
+const INODE_RECORD_SIZE_OFFSET: usize = 40;
+const INODE_RECORD_BLOCKS_OFFSET: usize = 48;
+/// `blocks` 的计量单位：512 字节一块，`blocks` = ⌈`size` ÷ 512⌉，是逻辑长度的块数、不表示分到的空间（D8（核心索引结构） 已定项 6，
+/// C480（inode 记录的 blocks 怎么算全仓没有条款） 用户 2026-09-23 定）。
+const INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES: u64 = 512;
 
 fn data_unit_bytes() -> usize {
     usize::try_from(DATA_UNIT_BYTES).expect("32768")
@@ -699,6 +706,16 @@ impl Walk<'_> {
                     self.judgements.judge("I-9.4", inode >= container, || {
                         format!("inode {inode} 小于它所在容器的号 {container}")
                     });
+                    let size_in_bytes = read_u64(&record, INODE_RECORD_SIZE_OFFSET);
+                    let blocks = read_u64(&record, INODE_RECORD_BLOCKS_OFFSET);
+                    let logical_length_in_blocks =
+                        size_in_bytes.div_ceil(INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES);
+                    self.judgements
+                        .judge("I-9.15", blocks == logical_length_in_blocks, || {
+                            format!(
+                                "inode {inode} 的记录 blocks {blocks} 不等于 ⌈size {size_in_bytes} ÷ {INODE_RECORD_BLOCKS_FIELD_UNIT_BYTES}⌉ = {logical_length_in_blocks}"
+                            )
+                        });
                     self.inode_object_birth.insert(inode, read_u64(&record, 8));
                     smallest_inode_in_this_container = Some(
                         smallest_inode_in_this_container.map_or(inode, |seen| seen.min(inode)),
@@ -857,18 +874,7 @@ impl Walk<'_> {
                             unique = false;
                         }
                         instances.push(instance);
-                        self.instance_table_rows.push(InstanceTableRow {
-                            instance,
-                            published_checkpoint_txg: u64::from_le_bytes(
-                                row[5..13].try_into().expect("8 字节"),
-                            ),
-                            applied_transaction_high_water_mark: u64::from_le_bytes(
-                                row[13..21].try_into().expect("8 字节"),
-                            ),
-                            is_rollback: row[INSTANCE_TABLE_ROW_FLAGS_OFFSET]
-                                & INSTANCE_TABLE_ROW_FLAG_ROLLBACK
-                                != 0,
-                        });
+                        self.instance_table_rows.push(parse_instance_table_row(row));
                         if instance >= mount_root_instance {
                             below_mount_root = false;
                         }
@@ -1319,9 +1325,11 @@ fn references_of_root(
     };
     let instance_table = parse_node_pointer(&record[170..256]);
     references.note(&instance_table);
-    // ⚠️ 实例表链上第 1 片起的落点这一遍不认（D18（块里携带什么信息） 已定项 11 的链）：今天的写者只写一片
-    // （多片在 bump 次序里怎么排、行怎么分片没有条款，`MountError::InstanceTableChainLongerThanOnePageUndecided`），
-    // 带已释放记录、能让 I-3.9 看到第 1 片的镜像写不出来，认链的这一段没有用例能证它会红。写者写第二片那一次一起补。
+    // 实例表链上第 1 片起的落点（D18（块里携带什么信息） 已定项 11 的链）也是这条根引用的：写行那次发布整条链重写、逐片释放旧链，
+    // 不数它们，旧链第 1 片起那几条已释放记录在 I-3.9 这一遍里找不到引用过它的根、被当成「见证它释放的根已不在候选集里」跳过。
+    if depth == ReferenceScanDepth::EveryReferencedPlacement {
+        note_instance_table_pages_after_the_first(reader, &instance_table, &mut references);
+    }
     // 中央映射树的根住根记录（D19（块指针的结构与宽度预算） 已定项 11）。映射条目指的单元与树里引用的是同一批，不重复数——
     // 主走读的 `note_reference` 也不数它们（数了 I-3.1（已分配统计对得上） 与 I-5.1（引用不重叠） 会把同一个落点算两遍）。
     let mapping_root = parse_node_pointer(&record[256..342]);
@@ -1374,6 +1382,53 @@ fn references_of_root(
     references
 }
 
+/// 沿实例表链（根记录直接持有第 0 片，第 k 片末尾的链指针记录指着第 k + 1 片，D18（块里携带什么信息） 已定项 11）把第 1 片起每一片的
+/// 指针记进这条根的引用集合。只读不判：一片读不出、解不开、身份不是 (0, 4, 片序号, 0)、最后一条不是合法的链指针记录，
+/// 都只如实记成数不全（判定归主走读：I-2.1、I-1.1、I-3.8）。
+/// 迭代上界：第 k 片要求容器号是 k，每一轮读的是盘上一个不同的单元；跨轮带的只有下一片的指针；提前出口是走到最后一片与上面那几样读不下去。
+fn note_instance_table_pages_after_the_first(
+    reader: &dyn ImageReader,
+    first_page_pointer: &PointerView,
+    references: &mut RootReferences,
+) {
+    let mut pointer = *first_page_pointer;
+    let mut page_index: u64 = 0;
+    loop {
+        let Some(page) = read_unit_without_judging(reader, &pointer, data_unit_bytes())
+            .and_then(|unit| packed_unit_view(&unit).ok())
+        else {
+            references.is_complete = false;
+            return;
+        };
+        let identity = (
+            page.birth_tree,
+            page.record_type,
+            page.container,
+            page.container_birth,
+        );
+        if identity != (0, PACKED_TYPE_INSTANCE_TABLE, page_index, 0) {
+            references.is_complete = false;
+            return;
+        }
+        match page
+            .records
+            .last()
+            .map_or(ChainRecordView::Malformed, |last| chain_record_view(last))
+        {
+            ChainRecordView::LastPage => return,
+            ChainRecordView::NextPage(next_page) => {
+                references.note(&next_page);
+                pointer = next_page;
+                page_index += 1;
+            }
+            ChainRecordView::Malformed => {
+                references.is_complete = false;
+                return;
+            }
+        }
+    }
+}
+
 /// 一棵树的根节点下面还引用了什么。第一版只有「extent 叶的数据指针」与「inode 树内部条目的子指针」两种下探；
 /// 别的形状（另外四棵树的内部节点、条目格式没有条款的树）走不下去，如实记成数不全。
 fn collect_tree_references(
@@ -1882,6 +1937,325 @@ fn judge_allocation_generations_against_unit_births(
     }
 }
 
+/// 一条根的树表里用户可见的两棵树（inode 树、extent 树）那两条根指针的盘上原样（D16（发布语义） 已定项 1「「非空」从盘上怎么认」：
+/// 比的是树表条目里这两棵树的根指针，不比树表单元自己的落点——每一次发布（连空发布）都重写树表单元）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct UserVisibleTreeRootPointers {
+    inode_tree: Option<Vec<u8>>,
+    extent_tree: Option<Vec<u8>>,
+}
+
+impl UserVisibleTreeRootPointers {
+    /// 没有前一条有效根时拿它比：两棵树的条目都没有（mkfs 种的第 0 版树表就是这样）。
+    const ABSENT: Self = Self {
+        inode_tree: None,
+        extent_tree: None,
+    };
+}
+
+/// 一条根的树表里 inode 树与 extent 树那两条根指针（树表条目偏移 14 起的 86 字节）。只读不判。
+/// 树表指针全零、树表读不出或解不开、条目窄于字段表、同一种树出现两条 ⇒ None：这条根算不算非空判不了，不按空或非空猜。
+/// 条目格式没有条款的那几种树（livelist、稀疏旁表、deadlist 与不认识的码）与分配记录树、记账树不进「非空」的比较，跳过。
+fn user_visible_tree_root_pointers(
+    reader: &dyn ImageReader,
+    record: &[u8],
+    cache: &mut IndexNodeCache,
+) -> Option<UserVisibleTreeRootPointers> {
+    let tree_table_pointer = parse_node_pointer(&record[36..122]);
+    if tree_table_pointer.all_zero {
+        return None;
+    }
+    let tree_table = read_index_node_without_judging(reader, &tree_table_pointer, cache)?;
+    let mut pointers = UserVisibleTreeRootPointers::ABSENT;
+    for entry in &tree_table.entries {
+        if entry.len() < tree_table_entry_bytes() {
+            return None;
+        }
+        let pointer_of_this_tree = match TreeKindForReferenceScan::of(read_u16(entry, 10)) {
+            TreeKindForReferenceScan::Inode => &mut pointers.inode_tree,
+            TreeKindForReferenceScan::Extent => &mut pointers.extent_tree,
+            TreeKindForReferenceScan::Allocation
+            | TreeKindForReferenceScan::Accounting
+            | TreeKindForReferenceScan::WithoutWalkableEntryFormat(_) => continue,
+        };
+        if pointer_of_this_tree
+            .replace(entry[14..100].to_vec())
+            .is_some()
+        {
+            return None;
+        }
+    }
+    Some(pointers)
+}
+
+/// 实例表一行（`kind` 0 的记录）按字段表解出来：主走读判行那一半（`judge_instance_table_rows`）与 I-7.9 读一条根自己的实例表共用。
+fn parse_instance_table_row(row: &[u8]) -> InstanceTableRow {
+    InstanceTableRow {
+        instance: u32::from_le_bytes(row[1..5].try_into().expect("4 字节")),
+        published_checkpoint_txg: u64::from_le_bytes(row[5..13].try_into().expect("8 字节")),
+        applied_transaction_high_water_mark: u64::from_le_bytes(
+            row[13..21].try_into().expect("8 字节"),
+        ),
+        is_rollback: row[INSTANCE_TABLE_ROW_FLAGS_OFFSET] & INSTANCE_TABLE_ROW_FLAG_ROLLBACK != 0,
+    }
+}
+
+/// 一条根自己指着的那张实例表的全部行，沿链读（D18（块里携带什么信息） 已定项 11：根记录直接持有第 0 片，第 k 片末尾的链指针记录指着第 k + 1 片）。
+/// 只读不判：哪一片读不出（两份的整单元 CRC 都对不上位置条目）、解不开（头判不过、类不是码 3、记录类型不是 4、记录宽不是 88、
+/// 记录数 × 记录宽超过声明长度或声明长度超过容量、身份不是 (0, 4, 片序号, 0)、链指针记录之前有不是行的记录、最后一条不是合法的链指针记录）
+/// ⇒ None，判定归主走读那一遍（I-2.1、I-1.7、I-1.1、I-3.8）。
+/// 迭代上界：第 k 轮要求读到的单元身份里容器号是 k，两轮读到同一个单元就要它的容器号同时等于两个数 ⇒ 每一轮读的是盘上不同的单元，
+/// 轮数不超过盘上装得下的 32 KiB 单元数。跨轮带的是下一片的指针、片序号与已经接起来的行；提前出口只有读不出、解不开。
+fn instance_table_rows_of_root_without_judging(
+    reader: &dyn ImageReader,
+    record: &[u8],
+) -> Option<Vec<InstanceTableRow>> {
+    let mut pointer = parse_node_pointer(&record[170..256]);
+    let mut page_index: u64 = 0;
+    let mut rows: Vec<InstanceTableRow> = Vec::new();
+    loop {
+        let unit = read_unit_without_judging(reader, &pointer, data_unit_bytes())?;
+        if !matches!(check_unit(&unit), Ok(3)) {
+            return None;
+        }
+        let identity = (
+            read_u64(&unit, 43),
+            read_u16(&unit, 51),
+            read_u64(&unit, 53),
+            read_u64(&unit, 61),
+        );
+        let record_count = usize::from(read_u16(&unit, 69));
+        let record_width = usize::from(read_u16(&unit, 71));
+        let declared_length = usize::from(read_u16(&unit, 8));
+        if identity != (0, PACKED_TYPE_INSTANCE_TABLE, page_index, 0)
+            || record_width != INSTANCE_TABLE_RECORD_BYTES
+            || record_count * record_width > declared_length
+            || declared_length > data_unit_bytes() - 136
+        {
+            return None;
+        }
+        let records: Vec<&[u8]> = unit[136..136 + record_count * record_width]
+            .chunks(record_width)
+            .collect();
+        let (chain_record, rows_of_this_page) = records.split_last()?;
+        for row in rows_of_this_page {
+            if row[0] != 0 {
+                return None;
+            }
+            rows.push(parse_instance_table_row(row));
+        }
+        match chain_record_view(chain_record) {
+            ChainRecordView::LastPage => return Some(rows),
+            ChainRecordView::NextPage(next_page) => {
+                pointer = next_page;
+                page_index += 1;
+            }
+            ChainRecordView::Malformed => return None,
+        }
+    }
+}
+
+/// 按一张实例表，这条根在不在被抛弃的时间线上：有它那个实例的行 (i, Ti, Wi) 且它的 txg > Ti
+/// （D23（journal 的角色与格式） 已定项 14 回退段的候选集规则）。
+fn abandoned_by_instance_table_rows(
+    instance_table_rows: &[InstanceTableRow],
+    root: &crate::RootView,
+) -> bool {
+    instance_table_rows.iter().any(|row| {
+        row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg
+    })
+}
+
+/// D16（发布语义） 已定项 1 的抬 F 上限：min(每块盘上最新的有效根的 txg, 第 4 新的非空有效根的 txg)，非空有效根不足 4 个时取最旧有效根的 txg。
+/// 对非空集合单调不减：多认一条非空根，第 4 新的只会更新或不变，而最旧有效根不新于任何一条非空根。
+fn rollback_floor_ceiling_from(
+    newest_valid_root_txg_on_every_device: u64,
+    non_empty_valid_root_txgs: &[u64],
+    oldest_valid_root_txg: u64,
+) -> u64 {
+    let mut newest_first = non_empty_valid_root_txgs.to_vec();
+    newest_first.sort_unstable_by(|left, right| right.cmp(left));
+    let fourth_newest_non_empty_or_oldest_valid = newest_first
+        .get(3)
+        .copied()
+        .unwrap_or(oldest_valid_root_txg);
+    newest_valid_root_txg_on_every_device.min(fourth_newest_non_empty_or_oldest_valid)
+}
+
+/// 按一条抬 F 的根之前的根算出来的上限。有效根里有树表读不出的（它或它前一条读不出，它空不空判不了）时上限只知道一个区间：
+/// 那几条都算空是下沿、都算非空是上沿（[`rollback_floor_ceiling_from`] 对非空集合单调）；树表都读得出时两沿相等。
+struct RollbackFloorCeilingBeforeTheRaise {
+    lowest_possible: u64,
+    highest_possible: u64,
+    newest_valid_root_per_device: BTreeMap<u32, u64>,
+    non_empty_valid_root_txgs: Vec<u64>,
+    valid_root_txgs_whose_emptiness_is_undeterminable: Vec<u64>,
+    oldest_valid_root_txg: u64,
+}
+
+/// 算一条抬 F 的根 `raising_root` 那一刻的上限（I-7.9（回退下界 F 不高于抬 F 的上限），用户 2026-09-24 定）：
+/// 只用根环里 txg 比它小的根；有效 = 按**它自己**指着的实例表不被抛弃 ∧ txg ≥ `floor_before_the_raise`（抬之前的 F）；
+/// 非空 = 按 (txg, 实例) 排，跟前一条有效根比两条用户可见树的根指针，最旧的那条跟 [`UserVisibleTreeRootPointers::ABSENT`] 比。
+/// 它自己指着的实例表读不出、它之前一条有效根都没有 ⇒ None（上限无从算起：抬 F 的入口在这两格上拒，盘上出现这样一条抬 F 的根，
+/// 要么表后来坏了，要么撑它的根已被环盖掉）。
+fn rollback_floor_ceiling_before_the_raise(
+    reader: &dyn ImageReader,
+    geometry: &PoolGeometry,
+    roots: &[(u64, u64, crate::RootView)],
+    raising_root: &crate::RootView,
+    floor_before_the_raise: u64,
+    cache: &mut IndexNodeCache,
+) -> Option<RollbackFloorCeilingBeforeTheRaise> {
+    let instance_table_rows_of_the_raising_root =
+        instance_table_rows_of_root_without_judging(reader, &raising_root.record_bytes)?;
+    let mut valid_roots_before: Vec<(u32, &crate::RootView)> = roots
+        .iter()
+        .filter(|(_, _, root)| {
+            root.checkpoint_txg < raising_root.checkpoint_txg
+                && root.checkpoint_txg >= floor_before_the_raise
+                && !abandoned_by_instance_table_rows(&instance_table_rows_of_the_raising_root, root)
+        })
+        .map(|(region, _, root)| {
+            // 下标在范围内不是这里判的：`geometry_of` 已经把 R > 3 的槽拒掉，`roots` 的区域号都来自 `root_slot_positions`。
+            let device = geometry.region_devices[usize::try_from(*region).expect(
+                "区域号：geometry_of 判过 R ≤ REGION_DEVICE_FIELDS_IN_THE_SYSTEM_CONFIGURATION",
+            )];
+            (device, root)
+        })
+        .collect();
+    valid_roots_before.sort_unstable_by_key(|(_, root)| (root.checkpoint_txg, root.instance));
+    let oldest_valid_root_txg = valid_roots_before.first()?.1.checkpoint_txg;
+    let mut newest_valid_root_per_device: BTreeMap<u32, u64> = BTreeMap::new();
+    for (device, root) in &valid_roots_before {
+        let newest = newest_valid_root_per_device
+            .entry(*device)
+            .or_insert(root.checkpoint_txg);
+        *newest = (*newest).max(root.checkpoint_txg);
+    }
+    let newest_valid_root_txg_on_every_device = newest_valid_root_per_device
+        .values()
+        .copied()
+        .min()
+        .expect("上面 first() 过：至少一条有效根，它的盘在这张表里");
+    let mut non_empty_valid_root_txgs: Vec<u64> = Vec::new();
+    let mut valid_root_txgs_whose_emptiness_is_undeterminable: Vec<u64> = Vec::new();
+    let mut previous_valid_root_pointers = Some(UserVisibleTreeRootPointers::ABSENT);
+    for (_, root) in &valid_roots_before {
+        let pointers = user_visible_tree_root_pointers(reader, &root.record_bytes, cache);
+        match (&pointers, &previous_valid_root_pointers) {
+            (Some(this_root), Some(previous_root)) => {
+                if this_root != previous_root {
+                    non_empty_valid_root_txgs.push(root.checkpoint_txg);
+                }
+            }
+            (None, _) | (_, None) => {
+                valid_root_txgs_whose_emptiness_is_undeterminable.push(root.checkpoint_txg);
+            }
+        }
+        previous_valid_root_pointers = pointers;
+    }
+    let every_possibly_non_empty: Vec<u64> = non_empty_valid_root_txgs
+        .iter()
+        .chain(valid_root_txgs_whose_emptiness_is_undeterminable.iter())
+        .copied()
+        .collect();
+    Some(RollbackFloorCeilingBeforeTheRaise {
+        lowest_possible: rollback_floor_ceiling_from(
+            newest_valid_root_txg_on_every_device,
+            &non_empty_valid_root_txgs,
+            oldest_valid_root_txg,
+        ),
+        highest_possible: rollback_floor_ceiling_from(
+            newest_valid_root_txg_on_every_device,
+            &every_possibly_non_empty,
+            oldest_valid_root_txg,
+        ),
+        newest_valid_root_per_device,
+        non_empty_valid_root_txgs,
+        valid_root_txgs_whose_emptiness_is_undeterminable,
+        oldest_valid_root_txg,
+    })
+}
+
+/// I-7.9（回退下界 F 不高于抬 F 的上限）：只判**抬 F 的那一条根**，拿它之前的根算上限（用户 2026-09-24 定，
+/// 里程碑「第二个事务」收口表第 26 行；定义是实二报告 `impl-m2-checker2` 第八节那一种）。
+///
+/// 抬 F 的根：同一实例里 txg 比它小的最新那条根（它的前一条）带的 F 比它带的低。回退那次与新实例的第一条根没有同实例的前一条，
+/// 不算抬——它带的是恢复算出来的 F_生效，而按它自己的实例表，当初撑起那个 F 的非空根已在被抛弃的时间线上，拿它算上限会在合法的
+/// 「抬 F 之后回退到候选集里的根」上判红（固定用例 `rolling_back_to_the_root_at_the_effective_floor_is_accepted_and_reads_back_that_version`）。
+/// 一次抬 F 推几次空发布、每次都带新 F 时，只有第一条是抬 F 的根（后几条的前一条已带新 F）。
+///
+/// 上限只用根环里 txg 比它小的根算，不在后来每张镜像上重算：抬之后再回退，后来的根会把当初撑着 F 的非空根判成无效，而 F 不回落。
+/// 环转过之后，当初算上限用过的最旧那几条根会被盖掉，缺了它们重算只会更宽（抓不到，不会误红）。
+///
+/// 三种结局：F ≤ 上限的下沿 ⇒ 成立；F > 上限的上沿 ⇒ 违例；落在两沿之间、或上限无从算起 ⇒ 这条根不判。
+/// 一条根都没判到时整条报不适用并带理由，不报成立。
+fn judge_rollback_floor_raises_against_their_ceilings(
+    reader: &dyn ImageReader,
+    geometry: &PoolGeometry,
+    roots: &[(u64, u64, crate::RootView)],
+    cache: &mut IndexNodeCache,
+    judgements: &mut Judgements,
+) {
+    let mut raising_roots_judged = 0u64;
+    let mut raising_roots_not_judged = 0u64;
+    for (_, _, raising_root) in roots {
+        let Some(previous_root_of_the_same_instance) = roots
+            .iter()
+            .map(|(_, _, root)| root)
+            .filter(|root| {
+                root.instance == raising_root.instance
+                    && root.checkpoint_txg < raising_root.checkpoint_txg
+            })
+            .max_by_key(|root| root.checkpoint_txg)
+        else {
+            continue;
+        };
+        let floor_before_the_raise = previous_root_of_the_same_instance.rollback_floor;
+        if raising_root.rollback_floor <= floor_before_the_raise {
+            continue;
+        }
+        let Some(ceiling) = rollback_floor_ceiling_before_the_raise(
+            reader,
+            geometry,
+            roots,
+            raising_root,
+            floor_before_the_raise,
+            cache,
+        ) else {
+            raising_roots_not_judged += 1;
+            continue;
+        };
+        let raised_floor = raising_root.rollback_floor;
+        if raised_floor > ceiling.lowest_possible && raised_floor <= ceiling.highest_possible {
+            raising_roots_not_judged += 1;
+            continue;
+        }
+        raising_roots_judged += 1;
+        let (raising_instance, raising_txg) = (raising_root.instance, raising_root.checkpoint_txg);
+        judgements.judge("I-7.9", raised_floor <= ceiling.lowest_possible, || {
+            format!(
+                "实例 {raising_instance} txg {raising_txg} 那条根把回退下界 F 从 {floor_before_the_raise} 抬到 {raised_floor}，高于它之前的根算出的抬 F 上限 {}：每块盘上最新的有效根 {:?}，非空有效根 {:?}，空不空判不了的有效根 {:?}，最旧有效根 txg {}",
+                ceiling.highest_possible,
+                ceiling.newest_valid_root_per_device,
+                ceiling.non_empty_valid_root_txgs,
+                ceiling.valid_root_txgs_whose_emptiness_is_undeterminable,
+                ceiling.oldest_valid_root_txg
+            )
+        });
+    }
+    if raising_roots_judged == 0 {
+        judgements.not_applicable(
+            "I-7.9",
+            if raising_roots_not_judged == 0 {
+                "根环里没有抬 F 的根：每条根带的 F 都不高于同一实例里它前一条根带的（回退与新实例的第一条根不算抬）"
+            } else {
+                "根环里抬 F 的根都判不了：它自己指着的实例表读不出、它之前一条有效根都没有，或有效根的树表读不出、上限落在它带的 F 两边"
+            },
+        );
+    }
+}
+
 /// 一个分配记录树叶上逐盘判 I-5.4：同一块盘上的记录按起点排好，相邻两条不相交就是两两不相交（跨度非负）。
 fn judge_allocation_record_ranges_of_one_node(
     node: &crate::IndexNodeView,
@@ -2438,6 +2812,10 @@ struct ScannedJournalRecord {
     transaction: u64,
     /// 记录头里的提交标记：I-8.8（前缀里的事务不被切开） 判的就是它。
     commit_marker: CommitMarker,
+    /// 记录头里的本次发布内序号（D23（journal 的角色与格式） 已定项 4）原样：I-8.9（一次发布的记录序号连续且只有末条带标志） 判它。
+    ordinal_within_publish: u32,
+    /// 记录标志那 1 字节原样（D23（journal 的角色与格式） 已定项 4 / 已定项 17）：I-8.9 判它的位 0 与其余位。
+    record_flags_byte: u8,
     slot: u64,
     back_chain: u32,
     /// CRC32C(这一条的 311 字节头，`header_csum` 那 32 字节按零参与)——I-8.6（反向链算法） 的算式。
@@ -2473,6 +2851,8 @@ fn scanned_journal_records_of_device(
                 new_root_segment: record.new_root_segment,
                 transaction: record.transaction,
                 commit_marker: CommitMarker::of(record.commit_marker_byte),
+                ordinal_within_publish: record.ordinal_within_publish,
+                record_flags_byte: record.record_flags_byte,
                 slot,
                 back_chain: record.back_chain,
                 chain_value_the_next_record_must_carry: back_chain_of_record_header(&bytes),
@@ -2482,8 +2862,9 @@ fn scanned_journal_records_of_device(
     records
 }
 
-/// 逐盘扫一遍 journal 环，把自证过的记录按盘收齐：判 journal 的三条不变量（I-8.6（反向链算法）、
-/// I-8.7（实例内事务号不重号）、I-8.8（前缀里的事务不被切开））读的是同一批记录，扫一次三条一起判，不各扫一遍。
+/// 逐盘扫一遍 journal 环，把自证过的记录按盘收齐：判 journal 的四条不变量（I-8.6（反向链算法）、
+/// I-8.7（实例内事务号不重号）、I-8.8（前缀里的事务不被切开）、I-8.9（一次发布的记录序号连续且只有末条带标志））
+/// 读的是同一批记录，扫一次四条一起判，不各扫一遍。
 fn scanned_journal_records_by_device(
     reader: &dyn ImageReader,
     geometry: &PoolGeometry,
@@ -2766,6 +3147,181 @@ fn judge_commit_markers_per_transaction(
     }
 }
 
+/// 记录标志位 0：本次发布末条（D23（journal 的角色与格式） 已定项 17）。其余位只许 0（已定项 4）。
+const RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH: u8 = 0b0000_0001;
+
+/// I-8.9 违例说明里的判据标签，次序照这里：红了在违例说明里逐条列出是哪几条（每条只留第一处）。
+const PUBLISH_ORDINAL_CRITERIA: [&str; 7] = [
+    "标志其余位",
+    "序号 0",
+    "跳号",
+    "不从 1 起",
+    "多于一条末条",
+    "末条之后还有记录",
+    "没有末条",
+];
+
+/// I-8.9（一次发布的记录序号连续且只有末条带标志）：同一实例、同一 checkpoint_txg 的记录（一次发布写出的全部记录），
+/// 「本次发布内序号」依次是 1..N、与 jsn 同步，记录标志位 0 恰好只在序号 N 那一条上为 1，其余位为 0
+/// （D23（journal 的角色与格式） 已定项 4 / 已定项 17）。逐盘判（与 I-8.6（反向链算法）、I-8.7、I-8.8 同一个口径）：
+/// 一组 = 同一块盘上实例代号与 checkpoint_txg 都相同的自证过的记录，`records` 按计数器索引 ⇒ 组里按计数器升序。
+/// 判据各自单独判，违例说明里写明红的是哪几条（标签见 `PUBLISH_ORDINAL_CRITERIA`）：
+/// - 标志其余位：记录标志位 0 之外有位为 1；
+/// - 序号 0；
+/// - 跳号：组里两条记录的序号之差不等于计数器之差（与 jsn 同步：jsn 连号时序号也连号）；
+/// - 不从 1 起：组里计数器最小那一条的前一个计数器上坐着一条自证过的、不属于这一组的记录 ⇒ 它是这次发布的第一条，序号要是 1；
+/// - 多于一条末条：组里带末条标志的多于一条；
+/// - 末条之后还有记录：带末条标志的那一条之后，这一组还有计数器更大的记录（它不是序号 N 那一条）；
+/// - 没有末条：组里计数器最大那一条的下一个计数器上坐着**同一实例**、不属于这一组的记录（这个实例已经往下写了，
+///   这次发布写完了），这一组却一条末条标志都没有。
+///
+/// **射程**：只判读得出的记录，读不出的那条不判（I-8.9 判据原句）——
+/// 一、一组的末条可能还没落盘（崩在一次发布的记录之间）：计数器最大那一条的下一个计数器上没有自证过的记录、
+///    或坐着别的实例的记录（崩溃之后新实例从读得出的最大号 + 1 接着写，D23（journal 的角色与格式） 已定项 14 注 3），
+///    就不知道这一组写完没有，「没有末条」不判；
+/// 二、组里夹着读不出的几条：「跳号」按序号之差与计数器之差比，隔着它们照样判得了；
+/// 三、「不从 1 起」只在前一个计数器上有自证过的记录时判（环里第一条、前一条读不出，都判不了）。
+///
+/// **不适用**：环里一条自证过的记录都没有。
+fn judge_publish_ordinals_and_last_record_flags(
+    records_by_device: &[(u32, BTreeMap<u64, ScannedJournalRecord>)],
+    judgements: &mut Judgements,
+) {
+    let mut first_violation_by_criterion: BTreeMap<&'static str, String> = BTreeMap::new();
+    let mut note_violation = |criterion: &'static str, detail: String| {
+        first_violation_by_criterion
+            .entry(criterion)
+            .or_insert(detail);
+    };
+    let mut judged_records = 0u64;
+    for (device, records) in records_by_device {
+        let mut records_by_publish: BTreeMap<(u32, u64), Vec<(u64, &ScannedJournalRecord)>> =
+            BTreeMap::new();
+        for (counter, record) in records {
+            judged_records += 1;
+            if record.record_flags_byte & !RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0 {
+                note_violation(
+                    "标志其余位",
+                    format!(
+                        "盘 {device} 实例 {} 计数器 {counter}（环槽 {}）那条记录的记录标志是 {:#010b}——位 0 之外只许 0",
+                        record.instance, record.slot, record.record_flags_byte
+                    ),
+                );
+            }
+            if record.ordinal_within_publish == 0 {
+                note_violation(
+                    "序号 0",
+                    format!(
+                        "盘 {device} 实例 {} 计数器 {counter}（环槽 {}）那条记录的本次发布内序号是 0——从 1 起",
+                        record.instance, record.slot
+                    ),
+                );
+            }
+            records_by_publish
+                .entry((record.instance, record.checkpoint_txg))
+                .or_default()
+                .push((*counter, record));
+        }
+        for ((instance, checkpoint_txg), members) in &records_by_publish {
+            let member_counters: Vec<u64> = members.iter().map(|(counter, _)| *counter).collect();
+            let member_ordinals: Vec<u32> = members
+                .iter()
+                .map(|(_, member)| member.ordinal_within_publish)
+                .collect();
+            let (first_counter, first_member) = *members
+                .first()
+                .expect("每一组至少一条：组是拿记录一条条追加出来的");
+            let (last_counter, _) = *members.last().expect("同上");
+            for (counter, member) in &members[1..] {
+                let counter_distance = counter - first_counter;
+                let ordinal_distance = i128::from(member.ordinal_within_publish)
+                    - i128::from(first_member.ordinal_within_publish);
+                if ordinal_distance != i128::from(counter_distance) {
+                    note_violation(
+                        "跳号",
+                        format!(
+                            "盘 {device} 实例 {instance} checkpoint_txg {checkpoint_txg} 那次发布的记录（计数器 {member_counters:?}）序号是 {member_ordinals:?}——序号之差要等于计数器之差"
+                        ),
+                    );
+                    break;
+                }
+            }
+            let record_before_the_first_member = first_counter
+                .checked_sub(1)
+                .and_then(|previous_counter| records.get(&previous_counter));
+            if record_before_the_first_member.is_some() && first_member.ordinal_within_publish != 1
+            {
+                note_violation(
+                    "不从 1 起",
+                    format!(
+                        "盘 {device} 实例 {instance} checkpoint_txg {checkpoint_txg} 那次发布的第一条（计数器 {first_counter}，前一个计数器上坐着别的发布的记录）序号是 {}——要从 1 起",
+                        first_member.ordinal_within_publish
+                    ),
+                );
+            }
+            let flagged_counters: Vec<u64> = members
+                .iter()
+                .filter(|(_, member)| {
+                    member.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0
+                })
+                .map(|(counter, _)| *counter)
+                .collect();
+            match flagged_counters.as_slice() {
+                [] => {
+                    let this_instance_wrote_on = records
+                        .get(&(last_counter + 1))
+                        .is_some_and(|next| next.instance == *instance);
+                    if this_instance_wrote_on {
+                        note_violation(
+                            "没有末条",
+                            format!(
+                                "盘 {device} 实例 {instance} checkpoint_txg {checkpoint_txg} 那次发布的记录（计数器 {member_counters:?}）一条都不带末条标志，而同一实例在计数器 {} 上已经往下写了",
+                                last_counter + 1
+                            ),
+                        );
+                    }
+                }
+                [only_flagged] => {
+                    if *only_flagged != last_counter {
+                        note_violation(
+                            "末条之后还有记录",
+                            format!(
+                                "盘 {device} 实例 {instance} checkpoint_txg {checkpoint_txg} 那次发布带末条标志的是计数器 {only_flagged}，这一组后面还有计数器 {member_counters:?} 里更大的记录"
+                            ),
+                        );
+                    }
+                }
+                [_, _, ..] => {
+                    note_violation(
+                        "多于一条末条",
+                        format!(
+                            "盘 {device} 实例 {instance} checkpoint_txg {checkpoint_txg} 那次发布带末条标志的有计数器 {flagged_counters:?} 这几条——只许一条"
+                        ),
+                    );
+                }
+            }
+        }
+    }
+    let violated_criteria: Vec<String> = PUBLISH_ORDINAL_CRITERIA
+        .into_iter()
+        .filter_map(|criterion| {
+            first_violation_by_criterion
+                .get(criterion)
+                .map(|detail| format!("{criterion}：{detail}"))
+        })
+        .collect();
+    if !violated_criteria.is_empty() {
+        judgements.judge("I-8.9", false, || violated_criteria.join("；"));
+    } else if judged_records > 0 {
+        judgements.judge("I-8.9", true, String::new);
+    } else {
+        judgements.not_applicable(
+            "I-8.9",
+            "环里一条自证过的记录都没有：没有哪一次发布的序号与末条标志可判",
+        );
+    }
+}
+
 /// 记录新根段里两条指针的落点（D23（journal 的角色与格式） 已定项 4 的字段表：树表指针 86、映射根指针 86，再往后是树 ID 水位 8 与 F 8）。
 const NEW_ROOT_SEGMENT_TREE_TABLE_POINTER: std::ops::Range<usize> = 0..86;
 const NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER: std::ops::Range<usize> = 86..172;
@@ -2942,12 +3498,16 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
     );
     let roots = valid_roots(reader, &geometry);
     judge_instance_carriers(reader, &geometry, &roots, &mut root_ring_judgements);
-    // I-8.6、I-8.7 与 I-8.8 只读 journal 环，不读根：根环全灭的镜像上它们照样判得了（那一格归 I-7.1）。
+    // I-8.6、I-8.7、I-8.8 与 I-8.9 只读 journal 环，不读根：根环全灭的镜像上它们照样判得了（那一格归 I-7.1）。
     let journal_records_by_device =
         scanned_journal_records_by_device(reader, &geometry, filesystem_identifier_low);
     judge_journal_back_chain(&journal_records_by_device, &mut root_ring_judgements);
     judge_transaction_numbers_per_instance(&journal_records_by_device, &mut root_ring_judgements);
     judge_commit_markers_per_transaction(&journal_records_by_device, &mut root_ring_judgements);
+    judge_publish_ordinals_and_last_record_flags(
+        &journal_records_by_device,
+        &mut root_ring_judgements,
+    );
     root_ring_judgements.judge("I-7.1", !roots.is_empty(), || {
         "根环里一条自证过的根都没有".to_string()
     });
@@ -3159,6 +3719,13 @@ pub fn check_pool_image(reader: &dyn ImageReader) -> Vec<(&'static str, Invarian
         &mut index_node_cache,
         &mut judgements,
     );
+    judge_rollback_floor_raises_against_their_ceilings(
+        reader,
+        &geometry,
+        &roots,
+        &mut index_node_cache,
+        &mut judgements,
+    );
     // I-9.6（水位大于两处最大号）：记账里那条「inode 号水位」要大于遍历侧算出的 inode 树内最大 key。
     // 两条独立路径——水位是发布路径在记账树里写下的一个数，最大 key 是 checker 逐片叶容器逐条记录数出来的。
     // 另一半（> 全部已发布的墓碑记录的对象 ID）今天没有对象：墓碑是打包记录类型 1，这一版一片都不写
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 35b4468..20dc336 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -4,7 +4,7 @@
 //! 不在任何开放的聚簇段里、不在 `R` 保护期内（第一版 R 为空）。
 //! 提交内生块（D3（空间分配） 已定项 5 / 已定项 10）：从开放聚簇段 bump，开放段 = 单元区内最低的 64 槽对齐全空段，bump 只在内存；
 //! 码 3 容器按数据单元那一档取落点（起点 32768 对齐），码 2 节点取最低空槽。
-//! 开放段装不下、又没有全空段时回落（D3（空间分配） 已定项 8 ②）：该设备内槽号最小的空槽，与 bump 游标绕开同一套位（已分配、影子账隔离、释放核校验和对不上的隔离、抬 F 扣住）。
+//! 开放段装不下、又没有全空段时回落（D3（空间分配） 已定项 8 ②）：该设备内槽号最小的空槽，与 bump 游标绕开同一套位（已分配、影子账隔离、抬 F 扣住）。
 //! 落点按设备取（D3（空间分配） 已定项 8 第 1 条「在每一块被选中的设备上各自取」）：每块盘按自己的空闲图各答一个，各盘一致才分配；
 //! 有的盘答不出、或各盘答得不同，在动任何状态之前拒绝（`PlacementRefusal`）——`Placement` 两盘同槽，各盘不同槽第一版不支持。
 //! 记账（D5（快照 / 空间记账机制） 已定项 7 / 已定项 8）：已分配 = 落点之和、容量 = 单元区、runs = 空闲槽的连续段数（逐设备）、
@@ -204,14 +204,6 @@ pub struct DeviceFreeMap {
     isolated: Vec<bool>,
     isolated_per_segment: Vec<u64>,
     isolated_slots: u64,
-    /// 释放之前按位置项读盘核校验和、核出对不上的那一份所在的槽（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1：
-    /// 逻辑上照样释放，物理槽不还回空闲池，记进隔离并计数报出）。与影子账那一套位（`isolated`）分开放：那一套是
-    /// D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」、条文要它随被抛弃的根被轮转覆写而清零（C503）；
-    /// 这一套清不清、跨不跨重挂、进不进准入式子都还没有条款（C394（释放判定不核映射条目位置项里的单元校验和）），
-    /// 第一版只住内存、不清，记账照走释放与回收那条路。
-    quarantined_after_release_checksum_mismatch: Vec<bool>,
-    quarantined_after_release_checksum_mismatch_per_segment: Vec<u64>,
-    quarantined_after_release_checksum_mismatch_slots: u64,
     /// 其中已释放、还在 defer 窗口里的槽数（D5（快照 / 空间记账机制） 已定项 4 第 5 项）。
     deferred_slots: u64,
     /// 抬 F 回收、但 F 还没在每块盘上生效的槽：记账已经算它空闲，分配器却不许发出去，直到带新 F 的根落满每块盘
@@ -247,17 +239,6 @@ impl DeviceFreeMap {
             isolated: vec![false; usize::try_from(unit_area_slots).expect("单元区槽数")],
             isolated_per_segment: vec![0; usize::try_from(segments).expect("段数")],
             isolated_slots: 0,
-            quarantined_after_release_checksum_mismatch: vec![
-                false;
-                usize::try_from(unit_area_slots)
-                    .expect("单元区槽数")
-            ],
-            quarantined_after_release_checksum_mismatch_per_segment: vec![
-                0;
-                usize::try_from(segments)
-                    .expect("段数")
-            ],
-            quarantined_after_release_checksum_mismatch_slots: 0,
             held_until_floor_takes_effect: vec![
                 false;
                 usize::try_from(unit_area_slots)
@@ -286,7 +267,6 @@ impl DeviceFreeMap {
             && slot.0 < UNIT_AREA_START_SLOT + self.unit_area_slots
             && !self.allocated[Self::index(slot)]
             && !self.isolated[Self::index(slot)]
-            && !self.quarantined_after_release_checksum_mismatch[Self::index(slot)]
             && !self.held_until_floor_takes_effect[Self::index(slot)]
     }
 
@@ -341,35 +321,8 @@ impl DeviceFreeMap {
         self.isolated_slots
     }
 
-    /// 释放之前读盘核出校验和对不上的一份：分配器此后绕开这个跨度（用户数据落点、开新段、提交内生块的 bump 与回落都不落在它上面），
-    /// 记账不因它而变——释放照样把它记进 defer、回收照样把它算回空闲（I-3.1（已分配统计对得上） 按有效根的引用取并集，
-    /// 没有根引用它之后记账的「已分配」也不能再算它）。「空闲」那一行因此含着它；要不要从准入里扣掉没有条款（C394）。
-    /// 同一个槽隔离两次不重复计数。
-    pub fn quarantine_after_release_checksum_mismatch(&mut self, slot: SlotNumber, span: u64) {
-        let start = Self::index(slot);
-        let end = start + usize::try_from(span).expect("跨度");
-        assert!(
-            end <= self.allocated.len(),
-            "跨度越过单元区末尾：隔离的是释放判定路径核过的落点，它在分配记录里在册、跨度对得上"
-        );
-        for index in start..end {
-            if !self.quarantined_after_release_checksum_mismatch[index] {
-                self.quarantined_after_release_checksum_mismatch[index] = true;
-                let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
-                self.quarantined_after_release_checksum_mismatch_per_segment[segment] += 1;
-                self.quarantined_after_release_checksum_mismatch_slots += 1;
-            }
-        }
-    }
-
-    /// 释放之前读盘核出校验和对不上而隔离的槽数（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1「计数报出」的这块盘那一项）。
-    #[must_use]
-    pub fn quarantined_after_release_checksum_mismatch_slots(&self) -> u64 {
-        self.quarantined_after_release_checksum_mismatch_slots
-    }
-
     /// 清掉一个槽的影子账隔离位（`isolate` 置的那一套，D28（挂载期承诺量） 已定项 1 第九项「被抛弃的根被轮转覆写时清零」）：
-    /// 只动这一位与它的两个计数，分配位与释放核校验和对不上的那一套隔离照旧——清完之后它发不发得出去看别的位。这一位没置着就什么都不动。
+    /// 只动这一位与它的两个计数，分配位与抬 F 扣住那一位照旧——清完之后它发不发得出去看别的位。这一位没置着就什么都不动。
     pub fn clear_isolation_of_slot(&mut self, slot: SlotNumber) {
         let index = Self::index(slot);
         if self.isolated[index] {
@@ -394,14 +347,13 @@ impl DeviceFreeMap {
         }
     }
 
-    /// 提交内生块的 bump 游标要绕开的槽：已分配、影子账隔离、释放核校验和对不上的隔离、抬 F 扣住四种位任一为真
+    /// 提交内生块的 bump 游标要绕开的槽：已分配、影子账隔离、抬 F 扣住三种位任一为真
     /// （代码三方第三轮云端攻方腿打中：开段那一刻的条件担保不了开段之后才置的隔离位与扣住位）。
+    /// 释放之前核出校验和对不上的那一份不另占一种位：它那块盘上的分配记录留在已分配（D19（块指针的结构与宽度预算） 已定项 5，
+    /// 用户 2026-09-24 定案），落在「已分配」那一位上。
     #[must_use]
     pub fn is_blocked_for_commit_generated(&self, slot: SlotNumber) -> bool {
         let index = Self::index(slot);
-        if self.quarantined_after_release_checksum_mismatch[index] {
-            return true;
-        }
         self.allocated[index] || self.isolated[index] || self.held_until_floor_takes_effect[index]
     }
 
@@ -518,7 +470,6 @@ impl DeviceFreeMap {
             .find(|segment| {
                 self.used_per_segment[*segment] == 0
                     && self.isolated_per_segment[*segment] == 0
-                    && self.quarantined_after_release_checksum_mismatch_per_segment[*segment] == 0
                     && self.held_per_segment[*segment] == 0
             })
             .map(|segment| {
@@ -530,7 +481,7 @@ impl DeviceFreeMap {
     }
 
     /// 提交内生块的回落落点（D3（空间分配） 已定项 8 ②「提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`」）：
-    /// 这块盘单元区里起点槽号最小、整个跨度都不被挡的落点——挡的是 bump 游标绕开的同一套位（已分配、影子账隔离、释放核校验和对不上的隔离、抬 F 扣住），
+    /// 这块盘单元区里起点槽号最小、整个跨度都不被挡的落点——挡的是 bump 游标绕开的同一套位（已分配、影子账隔离、抬 F 扣住），
     /// 回落不是绕开影子账或扣住的第二条路；两槽的码 3 容器照数据单元那一档起点 32768 对齐（D3（空间分配） 已定项 10 ⑤）。第一版 `R` 为空。
     #[must_use]
     pub fn lowest_commit_generated_fallback_slot(
@@ -775,10 +726,6 @@ pub struct PoolAllocator {
     /// 那一段仍装着这次提交的内生块，D3（空间分配） 已定项 8 第 2 条「聚簇段只给提交内生块」照样压着它（增补 2 第 20c 行，
     /// 代码三方第一轮打中）。段里的块被回收空了也仍算聚簇段：它还能被 `lowest_empty_segment` 重新开来装提交内生块。
     cluster_segments: BTreeSet<SlotNumber>,
-    /// C146（无空段时的回落政策全仓无定义） ② 的运行时计数：实际落点与政策函数不一致的次数，第一个事务恒 0。
-    /// 落点按设备取之后，发出去的落点就是每块盘各自的政策答案（答得不同在动状态之前拒绝），这个数按构造恒 0；
-    /// 此前它比的是「盘 0 的答案」与「各盘答案的最小值」。字段留着是因为发布结果与真设备二进制照报它；删还是改成别的口径，没有定。
-    pub policy_mismatches: u64,
     records: Vec<AllocationRecord>,
     /// 已回收、还没被复用的落点（盘上那条记录仍写着已释放；复用时那条记录被改写）。只住内存。
     reclaimed: BTreeSet<(DeviceIdentity, SlotNumber)>,
@@ -808,7 +755,6 @@ impl PoolAllocator {
             open_segment: None,
             bump_cursor: 0,
             cluster_segments: BTreeSet::new(),
-            policy_mismatches: 0,
             records: Vec::new(),
             reclaimed: BTreeSet::new(),
             format_time_tree_table: None,
@@ -953,7 +899,23 @@ impl PoolAllocator {
     /// 释放一个落点（D3（空间分配） 已定项 7：「释放」= 放进 defer 队列那一刻）：每盘那条分配记录改写成已释放 + 释放代，
     /// 条目不删；槽仍占着，要等释放代 ≤ max(F_生效, 环里最旧有效根) 才可再分配（D16（发布语义） 已定项 1）。
     pub fn release(&mut self, placement: Placement, release_generation: CheckpointTxg) {
+        self.release_leaving_the_record_allocated_on(placement, release_generation, &[]);
+    }
+
+    /// 同 [`Self::release`]，只是 `devices_whose_record_stays_allocated` 那几块盘上的那条分配记录一个字节都不动、留在「已分配」：
+    /// 释放之前按位置项读盘核校验和、核出对不上（读不出也算）的那一份（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-24 定案）——
+    /// 那一份的槽不还回空闲池；记录落盘即跨重挂，重挂之后从分配记录树重建的分配器照样认它已分配，准入里照已分配算、不另进式子。
+    /// 逐盘各算：另一块盘上核得上的那一份照常释放。
+    pub fn release_leaving_the_record_allocated_on(
+        &mut self,
+        placement: Placement,
+        release_generation: CheckpointTxg,
+        devices_whose_record_stays_allocated: &[DeviceIdentity],
+    ) {
         for device in &mut self.devices {
+            if devices_whose_record_stays_allocated.contains(&device.device) {
+                continue;
+            }
             let record = self
                 .records
                 .iter_mut()
@@ -1228,21 +1190,6 @@ impl PoolAllocator {
         device_map.isolate(slot, span);
     }
 
-    /// 释放之前按位置项读盘核校验和、这块盘上那一份核出对不上：把它在这块盘上隔离（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）。
-    /// 调用方先照常 `release` 这个落点（逻辑上照样释放），再隔离对不上的那几块盘。
-    pub fn quarantine_after_release_checksum_mismatch(
-        &mut self,
-        device: DeviceIdentity,
-        placement: Placement,
-    ) {
-        let device_map = self
-            .devices
-            .iter_mut()
-            .find(|device_map| device_map.device == device)
-            .expect("核出对不上的那一份是从这块盘上读出来的：读得到就说明池里有这块盘，而分配器按池里的盘建");
-        device_map.quarantine_after_release_checksum_mismatch(placement.slot, placement.span);
-    }
-
     /// 装上根环表（`RootRingOccupancy`：挂载时从盘上读出来的，或 mkfs 刚写下的），替掉原来那一张。
     pub fn install_root_ring_occupancy(&mut self, occupancy: RootRingOccupancy) {
         self.root_ring = Some(occupancy);
@@ -1470,7 +1417,6 @@ mod tests {
             6,
             "mkfs 两个单元 × 2 盘（分配代 0）+ t1 × 2 盘"
         );
-        assert_eq!(pool.policy_mismatches, 0);
         let extent_root = pool
             .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
             .expect("开放段");
diff --git a/crates/singlefs-core/src/instance_table.rs b/crates/singlefs-core/src/instance_table.rs
index 09847d5..f75a4c4 100644
--- a/crates/singlefs-core/src/instance_table.rs
+++ b/crates/singlefs-core/src/instance_table.rs
@@ -2,9 +2,9 @@
 //! 恢复（回退行的 W）、可写挂载与回退（写行）都从这里解；checker 是独立解析器，不共用这份。
 //!
 //! 一张实例表是一条链：根记录指着第 0 片，第 k 片的链指针记录指着第 k + 1 片，最后一片写「无下一片」
-//! （沿链读盘在 `recovery::instance_table_chain_of_root`）。**写者今天只写一片**：多于一片时各片在 bump 次序里怎么排
-//! （D3（空间分配） 已定项 10 ⑤ 只写了「实例表单元最前」，排序规则管的是「其余」）、行怎么分到各片，都没有条款，
-//! 可写挂载与回退在取号之前就拒（`mount::MountError::InstanceTableChainLongerThanOnePageUndecided`）。
+//! （沿链读盘在 `recovery::instance_table_chain_of_root`）。写者按用户 2026-09-24 的两条定案写多片：行一片写满 369 行再开下一片、
+//! 最后一片装剩下的（D18（块里携带什么信息） 已定项 11，[`instance_table_rows_of_each_page`]）；在提交内生块的 bump 次序里尾片先
+//! （D3（空间分配） 已定项 10 ⑤，装链在 `transaction::build_instance_table_chain`）。
 
 use crate::address::{CheckpointTxg, InstanceGeneration, TreeIdentifier};
 use crate::bytes::{ByteReader, ByteWriter};
@@ -86,7 +86,8 @@ const _CHAIN_RECORD_IS_KIND_FLAG_AND_POINTER_WITHOUT_RESERVE: () = assert!(
 );
 
 /// 实例表链上的第几片：身份四元组里的容器号就是它（D18（块里携带什么信息） 已定项 11「容器号 = 片序号从 0 起」）。
-#[derive(Clone, Copy)]
+/// 比较与排序那几样是它进 `transaction::TransactionUnit` 的角色（第 1 片起那一类）逼出来的，那个枚举要它们。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub struct InstanceTablePageIndex(pub u64);
 
 impl InstanceTablePageIndex {
@@ -127,6 +128,42 @@ pub fn instance_table_pages_for_rows(rows: usize) -> usize {
     rows.div_ceil(rows_per_page).max(1)
 }
 
+/// 一片装几行：370 条记录减去末尾那条链指针记录。
+fn instance_rows_per_page() -> usize {
+    usize::try_from(INSTANCE_TABLE_PAGE_RECORDS).expect("370") - 1
+}
+
+/// 一张表的行分到链上各片，按链上的次序（第 0 片在前）：一片写满 369 行再开下一片，最后一片装剩下的
+/// （D18（块里携带什么信息） 已定项 11，用户 2026-09-24 定案）；0 行也是一片（空的那一片）。片数恒等于
+/// [`instance_table_pages_for_rows`]。
+#[must_use]
+pub fn instance_table_rows_of_each_page(rows: &[InstanceRow]) -> Vec<&[InstanceRow]> {
+    if rows.is_empty() {
+        return vec![rows];
+    }
+    rows.chunks(instance_rows_per_page()).collect()
+}
+
+/// 链上一片的全部记录：这一片的行在前、链指针记录最末（D18（块里携带什么信息） 已定项 11「链指针记录恒为一片的最后一条」）。
+///
+/// # Panics
+/// 行数多于一片装得下的 369：调用方按 [`instance_table_rows_of_each_page`] 切的片，每片至多 369 行。
+#[must_use]
+pub fn instance_table_page_records(
+    rows: &[InstanceRow],
+    chain: &InstanceTableChainRecord,
+) -> Vec<Vec<u8>> {
+    assert!(
+        rows.len() <= instance_rows_per_page(),
+        "一片至多 {} 行：调用方按 instance_table_rows_of_each_page 切片（{} 行）",
+        instance_rows_per_page(),
+        rows.len()
+    );
+    let mut records: Vec<Vec<u8>> = rows.iter().map(InstanceRow::to_bytes).collect();
+    records.push(chain.to_bytes());
+    records
+}
+
 /// 链指针记录（`kind` 1，恒为一片的最后一条）：`kind 1 | 有无下一片 1 | 位置指针 86（无下一片时清零占位）| 预留 0`
 /// （D18（块里携带什么信息） 已定项 11；位置指针与根记录里的实例表单元指针同型，D19（块指针的结构与宽度预算） 已定项 7 / 8）。
 #[derive(Debug, PartialEq)]
@@ -138,6 +175,23 @@ pub enum InstanceTableChainRecord {
 }
 
 impl InstanceTableChainRecord {
+    /// 这条链指针记录的 88 字节：`kind 1 | 有无下一片 1 | 位置指针 86`。最后一片那一条与 mkfs 写的那一条逐字节相同
+    /// （`make_filesystem::instance_table_chain_record`，位置指针清零占位）。
+    #[must_use]
+    pub fn to_bytes(&self) -> Vec<u8> {
+        match self {
+            Self::LastPage => instance_table_chain_record(),
+            Self::NextPage(pointer) => {
+                let mut writer = ByteWriter::new(usize::try_from(INSTANCE_ROW_BYTES).expect("88"));
+                writer.put_u8(INSTANCE_ROW_KIND_CHAIN);
+                writer.put_u8(CHAIN_RECORD_HAS_NEXT_PAGE);
+                pointer.write_to(&mut writer);
+                writer.assert_position(INSTANCE_ROW_BYTES, "实例表链指针记录");
+                writer.into_bytes()
+            }
+        }
+    }
+
     /// 解一条链指针记录。宽不是 88、`kind` 不是 1、「有无下一片」不是 0 或 1、无下一片而位置指针不全零（条款：清零占位）、
     /// 有下一片而位置指针全零（指不到任何地方），都是 `None`。
     #[must_use]
@@ -210,31 +264,13 @@ impl InstanceTableRecords {
             InstanceTableChainRecord::NextPage(_) => None,
         }
     }
-
-    /// 写行那次发布重写出去的那一片的记录：行在前、「无下一片」的链指针记录最末（D18（块里携带什么信息） 已定项 11）。
-    ///
-    /// # Panics
-    /// 行数 + 1（链指针记录）多于一片的 370 条。产品路径走不到：写行只有 `mount` 的可写挂载与回退两条路，
-    /// 都经 `establish_instance` 在取号之前算这一版的行数加这次要写的行数，多于一片就拒
-    /// （`MountError::InstanceTableChainLongerThanOnePageUndecided`：第二片在 bump 次序里排第几、行怎么分片没有条款）。
-    #[must_use]
-    pub fn to_records(&self) -> Vec<Vec<u8>> {
-        let records_per_page = usize::try_from(INSTANCE_TABLE_PAGE_RECORDS).expect("370");
-        assert!(
-            self.rows.len() < records_per_page,
-            "{} 行加链指针记录装不进一片 {records_per_page} 条：可写挂载与回退在取号之前就该按片数拒掉",
-            self.rows.len()
-        );
-        let mut records: Vec<Vec<u8>> = self.rows.iter().map(InstanceRow::to_bytes).collect();
-        records.push(instance_table_chain_record());
-        records
-    }
 }
 
 #[cfg(test)]
 mod chain_record_tests {
     use super::{
-        instance_table_pages_for_rows, InstanceTableChainRecord, InstanceTablePage,
+        instance_table_page_records, instance_table_pages_for_rows,
+        instance_table_rows_of_each_page, InstanceRow, InstanceTableChainRecord, InstanceTablePage,
         InstanceTablePageIndex, InstanceTableRecords,
     };
     use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
@@ -363,4 +399,55 @@ mod chain_record_tests {
         assert_eq!(instance_table_pages_for_rows(738), 2);
         assert_eq!(instance_table_pages_for_rows(739), 3);
     }
+
+    /// 行分到各片（用户 2026-09-24 定：一片写满 369 行再开下一片，最后一片装剩下的）：各片的行数，片数与
+    /// `instance_table_pages_for_rows` 同一个数（写者按后者取落点、按前者装片，两者不等就有一片没有落点）；
+    /// 一片的记录 = 这一片的行 + 末尾一条链指针记录，「有下一片」那一条解回来就是写进去的那个指针。
+    #[test]
+    fn rows_fill_a_page_of_three_hundred_sixty_nine_before_the_next_page_opens() {
+        let rows_of = |count: u32| -> Vec<InstanceRow> {
+            (1..=count)
+                .map(|instance| InstanceRow {
+                    instance: InstanceGeneration(instance),
+                    selected_root_txg: CheckpointTxg(0),
+                    applied_transaction_high_water: 0,
+                    is_rollback: false,
+                })
+                .collect()
+        };
+        for (count, expected_rows_per_page) in [
+            (0, vec![0]),
+            (1, vec![1]),
+            (369, vec![369]),
+            (370, vec![369, 1]),
+            (738, vec![369, 369]),
+            (739, vec![369, 369, 1]),
+        ] {
+            let rows = rows_of(count);
+            let pages = instance_table_rows_of_each_page(&rows);
+            assert_eq!(
+                pages.iter().map(|page| page.len()).collect::<Vec<_>>(),
+                expected_rows_per_page,
+                "{count} 行"
+            );
+            assert_eq!(
+                pages.len(),
+                instance_table_pages_for_rows(rows.len()),
+                "{count} 行：片数与 instance_table_pages_for_rows 同一个数"
+            );
+        }
+        let pointer = next_page_pointer();
+        let records =
+            instance_table_page_records(&rows_of(2), &InstanceTableChainRecord::NextPage(pointer));
+        assert_eq!(records.len(), 3, "两行加末尾一条链指针记录");
+        assert_eq!(
+            InstanceTableChainRecord::parse(&records[2]),
+            Some(InstanceTableChainRecord::NextPage(pointer))
+        );
+        assert_eq!(
+            InstanceTableChainRecord::LastPage.to_bytes(),
+            instance_table_chain_record(),
+            "最后一片那一条与 mkfs 写的那一条逐字节相同"
+        );
+    }
 }
diff --git a/crates/singlefs-core/src/journal.rs b/crates/singlefs-core/src/journal.rs
index 5cf7206..a571d96 100644
--- a/crates/singlefs-core/src/journal.rs
+++ b/crates/singlefs-core/src/journal.rs
@@ -18,8 +18,48 @@ use crate::pointer::{LocationEntry, NodePointer};
 pub const JOURNAL_MAGIC: [u8; 4] = *b"SFSJ";
 /// 记录类型登记表（D23（journal 的角色与格式） 已定项 1）：0 无效、1 普通记录。
 pub const JOURNAL_RECORD_TYPE_ORDINARY: u16 = 1;
-/// magic 4 + 类型 2 + 算法类型 1 + 填充 1 + 记录长度 4 + 点名项数 4 + jsn 10 + checkpoint_txg 8 + nonce 12。
+/// magic 4 + 类型 2 + 算法类型 1 + 记录标志 1 + 记录长度 4 + 点名项数 4 + jsn 10 + checkpoint_txg 8 + nonce 12。
 pub const JOURNAL_HEADER_CHECKSUM_OFFSET: usize = 4 + 2 + 1 + 1 + 4 + 4 + 10 + 8 + 12;
+/// 记录标志 1 字节在 magic 4 + 类型 2 + 算法类型 1 之后（D23（journal 的角色与格式） 已定项 4，原「填充 1」那个字节）。
+pub const JOURNAL_RECORD_FLAGS_OFFSET: usize = 4 + 2 + 1;
+/// 记录标志位 0：本次发布末条（D23（journal 的角色与格式） 已定项 17）。其余位写 0，读到非 0 当损坏（已定项 4）。
+pub const JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH: u8 = 0b0000_0001;
+
+/// 这条记录在它那次发布里是不是末条：记录标志位 0（D23（journal 的角色与格式） 已定项 4 / 已定项 17）。
+/// 一次发布的末条之外的记录写 0；每次只有一条记录的发布（含空发布记录）那一条也写 1。
+/// 恢复按它认一次发布的边界（已定项 14 第六条），所选根覆盖的最后一条也按它认（已定项 14 注 1）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum JournalRecordPlaceInPublish {
+    /// 位 0 = 1：本次发布末条。
+    LastRecordOfThePublish,
+    /// 位 0 = 0：这次发布在它之后还有记录。
+    MoreRecordsOfThePublishFollow,
+}
+
+impl JournalRecordPlaceInPublish {
+    /// 写进记录标志那 1 字节的值：只用位 0，其余位恒 0。
+    #[must_use]
+    pub const fn record_flags_byte(self) -> u8 {
+        match self {
+            JournalRecordPlaceInPublish::LastRecordOfThePublish => {
+                JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH
+            }
+            JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => 0,
+        }
+    }
+
+    /// 从盘上读来的记录标志那 1 字节：位 0 之外有位为 1 ⇒ `None`（已定项 4「其余位写 0，读到非 0 当损坏」）。
+    #[must_use]
+    pub const fn from_record_flags_byte(record_flags_byte: u8) -> Option<Self> {
+        match record_flags_byte {
+            0 => Some(JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow),
+            JOURNAL_RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH => {
+                Some(JournalRecordPlaceInPublish::LastRecordOfThePublish)
+            }
+            _other_bits_set => None,
+        }
+    }
+}
 /// 本次发布内序号紧跟事务号 8 与提交标记 1（D23（journal 的角色与格式） 已定项 4）：头部校验和 32 之后。
 pub const JOURNAL_ORDINAL_WITHIN_PUBLISH_OFFSET: usize =
     JOURNAL_HEADER_CHECKSUM_OFFSET + 32 + 8 + 1;
@@ -111,6 +151,8 @@ pub struct JournalRecord {
     pub transaction: u64,
     pub is_commit: bool,
     pub ordinal_within_publish: JournalRecordOrdinalWithinPublish,
+    /// 记录标志位 0（D23（journal 的角色与格式） 已定项 4 / 已定项 17）。
+    pub place_in_publish: JournalRecordPlaceInPublish,
     pub back_chain: u32,
     pub filesystem_identifier: u64,
     pub new_tree_table: NodePointer,
@@ -148,7 +190,11 @@ impl JournalRecord {
         writer.put(&JOURNAL_MAGIC);
         writer.put_u16(JOURNAL_RECORD_TYPE_ORDINARY);
         writer.put_u8(0); // 算法类型
-        writer.put_u8(0); // 对齐填充
+        writer.assert_position(
+            u64::try_from(JOURNAL_RECORD_FLAGS_OFFSET).expect("偏移"),
+            "记录标志",
+        );
+        writer.put_u8(self.place_in_publish.record_flags_byte());
         writer.put_u32(u32::try_from(JOURNAL_RECORD_BYTES).expect("记录长度"));
         writer.put_u32(u32::try_from(self.named.len()).expect("点名项数 4 字节"));
         writer.put_u32(self.instance.0);
@@ -199,7 +245,9 @@ impl JournalRecord {
         bytes
     }
 
-    /// 读者：magic、类型、整条校验和、fsid、载荷校验和四关；不查反向链（那是前缀取法的事）。
+    /// 读者：magic、类型、整条校验和、fsid、载荷校验和四关，外加两条当损坏的（D23（journal 的角色与格式） 已定项 4）：
+    /// 记录标志位 0 之外有位为 1、本次发布内序号为 0。当损坏就是与校验和不过同一个结局——这条记录不算在，前缀在它之前断
+    /// （已定项 22 断号即止）。不查反向链，也不查一次发布之内跳不跳号：那两样要看别的记录，是前缀取法的事。
     #[must_use]
     pub fn parse(bytes: &[u8], expected_filesystem_identifier: u64) -> Option<Self> {
         let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
@@ -213,7 +261,9 @@ impl JournalRecord {
         if reader.get_u16() != JOURNAL_RECORD_TYPE_ORDINARY {
             return None;
         }
-        reader.skip(1 + 1);
+        reader.skip(1); // 算法类型
+        let place_in_publish =
+            JournalRecordPlaceInPublish::from_record_flags_byte(reader.get_u8())?;
         if u64::from(reader.get_u32()) != JOURNAL_RECORD_BYTES {
             return None;
         }
@@ -225,6 +275,10 @@ impl JournalRecord {
         let transaction = reader.get_u64();
         let is_commit = reader.get_u8() == 1;
         let ordinal_within_publish = JournalRecordOrdinalWithinPublish(reader.get_u32());
+        // 序号从 1 起（已定项 4）：读到 0 当这条记录损坏。
+        if ordinal_within_publish.0 == 0 {
+            return None;
+        }
         let back_chain = reader.get_u32();
         let payload_checksum = reader.get_u32();
         let new_tree_table = NodePointer::read_from(&mut reader);
@@ -257,6 +311,7 @@ impl JournalRecord {
             transaction,
             is_commit,
             ordinal_within_publish,
+            place_in_publish,
             back_chain,
             filesystem_identifier,
             new_tree_table,
@@ -281,6 +336,7 @@ mod tests {
             transaction: 0,
             is_commit: true,
             ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,
+            place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
             back_chain: 0,
             filesystem_identifier: 7,
             new_tree_table: NodePointer::empty_root(),
@@ -337,6 +393,73 @@ mod tests {
         assert_eq!(JournalRecord::parse(&bytes, 7), Some(record));
     }
 
+    /// 改过记录头某几个字节之后重封头部校验和（罩整条 4096、自身按 0 参与，D23（journal 的角色与格式） 已定项 13）：
+    /// 拦得住那条记录的只剩被改的字段本身。
+    fn reseal_header_checksum(bytes: &mut [u8]) {
+        let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
+        let digest =
+            wide_checksum_with_field_zeroed(bytes, record_bytes, JOURNAL_HEADER_CHECKSUM_OFFSET);
+        bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
+            .copy_from_slice(&digest);
+    }
+
+    /// 记录标志 1 字节在偏移 7（magic 4 + 类型 2 + 算法类型 1，原「填充 1」那个字节，D23（journal 的角色与格式） 已定项 4）：
+    /// 本次发布末条写 1、不是末条写 0，两种都读得回。偏移按字段表手算、写成字面量，不从写者用的常量推（已定项 26 第 1 条）。
+    #[test]
+    fn the_record_flags_byte_at_offset_seven_carries_bit_zero_for_the_last_record_of_the_publish_and_round_trips(
+    ) {
+        let last = empty_record(3);
+        let last_bytes = last.to_bytes();
+        assert_eq!(last_bytes[7], 1, "本次发布末条：记录标志位 0 = 1，其余位 0");
+        assert_eq!(JournalRecord::parse(&last_bytes, 7), Some(last));
+        let not_last = JournalRecord {
+            place_in_publish: JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,
+            ..empty_record(3)
+        };
+        let not_last_bytes = not_last.to_bytes();
+        assert_eq!(not_last_bytes[7], 0, "不是末条：记录标志整字节 0");
+        assert_eq!(JournalRecord::parse(&not_last_bytes, 7), Some(not_last));
+        let mut differing: Vec<usize> = (0..last_bytes.len())
+            .filter(|offset| last_bytes[*offset] != not_last_bytes[*offset])
+            .collect();
+        differing.retain(|offset| !(46..78).contains(offset));
+        assert_eq!(
+            differing,
+            vec![7],
+            "两条记录只差记录标志那 1 字节（与头部校验和）"
+        );
+    }
+
+    /// 记录标志位 0 之外有位为 1 的记录，读者当损坏（D23（journal 的角色与格式） 已定项 4「其余位写 0，读到非 0 当损坏」）：
+    /// 位 0 本身是不是 1 都一样拒。头部校验和按改过的字节重封过，拦住它的只有记录标志那一判。
+    #[test]
+    fn a_record_whose_flags_byte_sets_any_bit_other_than_bit_zero_is_refused_by_the_parser() {
+        for record_flags_byte in [0b0000_0010u8, 0b0000_0011, 0b1000_0001, 0xff] {
+            let mut bytes = empty_record(4).to_bytes();
+            bytes[7] = record_flags_byte;
+            reseal_header_checksum(&mut bytes);
+            assert_eq!(
+                JournalRecord::parse(&bytes, 7),
+                None,
+                "记录标志 {record_flags_byte:#010b}：位 0 之外有位为 1，当损坏"
+            );
+        }
+    }
+
+    /// 本次发布内序号为 0 的记录，读者当损坏（D23（journal 的角色与格式） 已定项 4：序号从 1 起，读到 0 当那条记录损坏、断链即止）。
+    /// 头部校验和按改过的字节重封过，拦住它的只有序号那一判。
+    #[test]
+    fn a_record_whose_ordinal_within_publish_is_zero_is_refused_by_the_parser() {
+        let mut bytes = empty_record(5).to_bytes();
+        bytes[87..91].copy_from_slice(&0u32.to_le_bytes());
+        reseal_header_checksum(&mut bytes);
+        assert_eq!(
+            JournalRecord::parse(&bytes, 7),
+            None,
+            "序号 0：当这条记录损坏"
+        );
+    }
+
     /// 一次发布里从 0 数第 k 条记录的序号是 k + 1：第一条是 1（D23（journal 的角色与格式） 已定项 4「从 1 起」）。
     #[test]
     fn the_ordinal_of_the_record_at_offset_k_of_a_publish_is_k_plus_one() {
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index d7bd568..7fee6bb 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -3,10 +3,10 @@
 //! 推空发布直到本实例的根覆盖每块盘（D16（发布语义） 已定项 8 戊），之后本实例的发布才接在后面。
 //! 第一版没有干净关闭标记，重开一律走恢复。
 
-use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
+use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
 use crate::allocator::{
-    AllocationRecord, DeviceFreeMap, Placement, PlacementOnDevice, PoolAllocator, ReclaimedReuse,
-    RootRingOccupancy, RootRingOccupant,
+    AllocationRecord, DeviceFreeMap, Placement, PlacementOnDevice, PlacementRefusal, PoolAllocator,
+    ReclaimedReuse, RootRingOccupancy, RootRingOccupant,
 };
 use crate::block_device::BlockDevice;
 use crate::journal::back_chain_of;
@@ -25,17 +25,18 @@ use crate::recovery::{
 use crate::root_record::RootRecord;
 use crate::root_ring::{target_for_publish, RootRingSlot};
 use crate::transaction::{
-    acquire_expected_instance, instance_generation_to_acquire,
+    acquire_expected_instance, instance_generation_to_acquire, instance_table_chain_to_release,
+    instance_table_page_roles_in_bump_order, placements_to_release_via_mapping,
     publish_instance_table_on_version_without_file, publish_sequence_admission, publish_version,
     publish_without_units, AcquisitionFailed, ExpectedInstanceAcquisitionFailed,
-    InstanceTableOnlyPublishPlan, InstanceTablePlan, PoolVersion, PoolWriter, PublishError,
-    PublishPlan, PublishShape, TransactionOutput, TransactionUnit, ZeroUnitPublishPlan,
+    InstanceTableOnlyPublishPlan, InstanceTablePlan, InstanceTableRewrite, PlacementRule,
+    PoolVersion, PoolWriter, PublishError, PublishPlan, PublishShape, TransactionOutput,
+    TransactionUnit, ZeroUnitPublishPlan,
 };
 use crate::write_accounting::WritesByStructureKind;
-use singlefs_format::{INSTANCE_TABLE_PAGE_RECORDS, ROOT_RING_REGIONS};
+use singlefs_format::ROOT_RING_REGIONS;
 use std::collections::{BTreeMap, BTreeSet};
 
-use crate::instance_table::instance_table_pages_for_rows;
 pub use crate::instance_table::{InstanceRow, InstanceTableRecords};
 
 /// 可写挂载没做成。
@@ -49,16 +50,14 @@ pub enum MountError {
     /// 判不了候选集，在任何写之前拒绝。
     InstanceTableMalformed,
     Acquisition(AcquisitionFailed),
-    /// 取号之后那一串发布（写行一次、暖机至多 R 次）里有一次没做成：错，连同这次挂载的写入口交得出的写账（[`PublishAfterAcquisitionFailed`]）。
-    Publish(PublishAfterAcquisitionFailed),
-    /// 抬 F 那一串空发布（D16（发布语义） 已定项 1：推到每块盘上都有一条带新 F 的根才生效）里有一次发不出去：
-    /// 前面 `publishes_persisted` 次已经落盘——带新 F 的根在盘上、调用方的现行版本已经是最后落盘的那一版，F 还没在每块盘上生效，
-    /// 抬 F 回收的槽照旧扣着；`cause` 是第 `publishes_persisted + 1` 次（从 1 数）的错。`cause` 自己说的「在任何写之前」
+    /// 取号之后那一串发布（写行一次、暖机至多 R 次）里有一次没做成：错，连同这次挂载的写入口交得出的写账（[`PublishSequenceFailed`]）。
+    Publish(PublishSequenceFailed),
+    /// 抬 F 那一串空发布（D16（发布语义） 已定项 1：推到每块盘上都有一条带新 F 的根才生效）里有一次发不出去：错，连同抬 F 自己开的
+    /// 写入口交得出的写账（[`PublishSequenceFailed`]，增补 2 收口表第 58 行，与可写挂载那一条同一个形态）。
+    /// `writes_of_persisted_publishes` 有几份，前面就有几次已经落盘——带新 F 的根在盘上、调用方的现行版本已经是最后落盘的那一版，
+    /// F 还没在每块盘上生效，抬 F 回收的槽照旧扣着；`cause` 是下一次（份数 + 1，从 1 数）的错。`cause` 自己说的「在任何写之前」
     /// （例如 `PublishError::PlacementRefused`）只对出错的这一次成立，不对整串成立（C516（抬 F 那一串发布被拒时前面几次已落盘））。
-    RaiseFloorSequencePublishFailed {
-        publishes_persisted: usize,
-        cause: PublishError,
-    },
+    RaiseFloorSequencePublishFailed(PublishSequenceFailed),
     /// 回退的目标根不在回退候选集里，`exclusion` 说是哪一条（管理员要做的决定都是换一条目标；调用方按这个字段分流，不看给人看的文字——
     /// 增补 3 第 2 件代码三方第一轮判决第三节第 2 条：此前只带一句理由文字，胶水分不出是哪一条）。在任何写之前拒绝。
     RollbackTargetNotACandidate {
@@ -97,7 +96,8 @@ pub enum MountError {
         expected: InstanceGeneration,
         recomputed: InstanceGeneration,
     },
-    /// 写行那次发布的准入（这次之后的分配记录条数、这次要写的记账行数）算不过：在**取号之前**拒绝，盘上一个字节都不动、
+    /// 写行那次发布的准入（这次之后的分配记录条数、这次要写的记账行数；这次要换下的那条实例表旧链逐片核不过）算不过：
+    /// 在**取号之前**拒绝，盘上一个字节都不动、
     /// 两块盘系统配置里的实例代号不动（增补 2 第 20a 行，代码三方第一轮打中）。改之前这两条只在发布路径里算，
     /// 取号（两次系统配置槽写 + 一道屏障）已经写完才报出来，而取号的回卷只管取号自己那几次写报错、管不到之后的发布失败 ⇒
     /// 分配记录树满了的池此后每试一次可写挂载就再烧一个实例代号。
@@ -116,34 +116,35 @@ pub enum MountError {
         warm_up_publishes_planned: usize,
         cause: PublishError,
     },
-    /// 写行那次发布要写（或要 COW 重写）一条多于一片的实例表链，而多于一片的写法有两处条款没写：
-    /// ① 各片在提交内生块的 bump 次序里怎么排——D3（空间分配） 已定项 10 ⑤ 只写「实例表单元最前」，树 ID 升序、先叶后根、
-    /// 同层按 key 升序那几条管的是「其余」；这个次序同时是各片的出生序号（D19（块指针的结构与宽度预算） 已定项 9），第二片的落点、
-    /// 它头里与第一片链指针里的出生序号都随它定；② 行怎么分到各片（每片数据行 369 是上限，写满一片再开下一片没有逐字条款）。
-    /// 两处定之前第一版不写第二片；在**取号之前**拒绝，盘上一个字节都不动、两块盘系统配置里的实例代号不动
-    /// （改之前取号写完才在装实例表单元时越界 panic，池此后每试一次可写挂载就再烧一个实例代号，代码三方第二轮 Z1-a）。
-    /// 两种情形都拒：这次之后要多于一片（`pages_after_this_publish` > 1，这一版的行数 + 这次要写的行数 + 1 > 370）；
-    /// 这一版的表已经多于一片（`pages_in_version` > 1，只有别的实现写的或坏的镜像上有——把它整条重写成一片要逐片释放旧链，
-    /// 这一格条款写了，第一版不写多片，也就不重写多片，一并拒）。
-    InstanceTableChainLongerThanOnePageUndecided {
+    /// 取号之前在分配器的一份拷贝上把这次挂载取号之后要发的那一串（写行一次、暖机 `warm_up_publishes_planned` 次）逐次取落点，
+    /// 第 `publish_index` 次（从 0 数，0 是写行那次）的 `unit` 取不到，`refusal` 是分配器给的原因
+    /// （`placements_of_the_publishes_after_acquisition_on_a_copy`）。**取号之前的准入怎么把这次挂载要写的落点算进去，条款没定**：
+    /// 可写挂载的准入要「实例切换的预留拿得到」（D2（RAID 条带策略） 已定项 13；D23（journal 的角色与格式） 已定项 14「切换要用的块在挂载准入时预留」），
+    /// 预留按 D28（挂载期承诺量） 已定项 3 算，其中暖机那一半的 c_max 与已定项 4 的 checkpoint 保留池从哪读没有条款
+    /// （C363（现算保留池时树高从哪读没有条款）），D28（挂载期承诺量） 已定项 1 那条式子另一边的「需求」怎么摊到每块盘也没有
+    /// （C370（需求、可用与 df 没有共同单位））。第一版不算那条式子，只把「这一串自己的落点取不到」挪到取号之前：在任何写之前返回，
+    /// 盘上逐字节不变、两块盘系统配置里的实例代号不动。改之前取号写完、写行或暖机才被落点拒绝（`Publish`），实例代号一去不回
+    /// （里程碑「第二个事务」增补 2 收口表第 39 行那一族：单元区 240 槽的小盘上走得到）。
+    PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
         instance_to_acquire: InstanceGeneration,
-        rows_in_version: usize,
-        pages_in_version: usize,
-        rows_to_write: usize,
-        pages_after_this_publish: usize,
+        publish_index: usize,
+        warm_up_publishes_planned: usize,
+        unit: TransactionUnit,
+        refusal: PlacementRefusal,
     },
 }
 
-/// 可写挂载（或回退）取号之后那一串发布——写行一次、暖机推到本实例的根覆盖每块盘（D16（发布语义） 已定项 8 戊）——里有一次没做成。
-/// 这次挂载的写入口是挂载自己开的、随错一起丢掉，所以它交得出的写账都在这里：已经落盘的那几次发布各自的写，
+/// 自己开写入口、接连推的一串发布里有一次没做成：可写挂载（或回退）取号之后那一串——写行一次、暖机推到本实例的根覆盖每块盘
+/// （D16（发布语义） 已定项 8 戊）；抬 F 那一串空发布——推到每块盘上都有一条带新 F 的根（D16（发布语义） 已定项 1）。
+/// 写入口是这一串自己开的、随错一起丢掉，所以它交得出的写账都在这里：已经落盘的那几次发布各自的写，
 /// 与失败那一次落盘阶段已记的写（增补 2 收口表第 58 行：不交出来，设备一层数到的写与程序交得出的账对不上）。
-/// 取号那两次系统配置槽写不是发布，不在其内（取号的回卷另由 `AcquisitionFailed` 报）。
+/// 不是发布的写不在其内：可写挂载取号那两次系统配置槽写（取号的回卷另由 `AcquisitionFailed` 报）。
 #[derive(Debug)]
-pub struct PublishAfterAcquisitionFailed {
+pub struct PublishSequenceFailed {
     pub cause: PublishError,
-    /// 失败之前这次挂载里已经落盘的发布各自的写，按先后（写行那次在最前）；写行那次就失败时是空的。
+    /// 失败之前这一串里已经落盘的发布各自的写，按先后（可写挂载写行那次在最前）；第一次就失败时是空的。
     pub writes_of_persisted_publishes: Vec<WritesByStructureKind>,
-    /// 这次挂载的写入口的失败账（`PoolWriter::writes_of_failed_publishes` 原样）：落盘阶段失败的那一次一份；
+    /// 这一串的写入口的失败账（`PoolWriter::writes_of_failed_publishes` 原样）：落盘阶段失败的那一次一份；
     /// 落盘之前就被拒的（准入、释放判定、落点）一个写都没发，不记，这里是空的。
     pub writes_of_failed_publishes: Vec<WritesByStructureKind>,
 }
@@ -257,21 +258,28 @@ enum PreviousVersion {
 }
 
 impl PreviousVersion {
-    /// 这一版的实例表的全部行（写行时要在它后面接上这次的行，取号之前的准入也按它的行数算）。两臂都有表：
-    /// 树表 0 条那一版的表是根记录直接指着的那条链（mkfs 种下的，或上一次挂载写行重写的）。
-    fn instance_table(&self) -> &InstanceTableRecords {
+    /// 这一版的实例表链：全部行（写行时要在它后面接上这次的行，取号之前的准入也按它的行数算）与每一片的指针
+    /// （写行那次发布整条链重写，这几片逐片释放）。两臂都有表：树表 0 条那一版的表是根记录直接指着的那条链
+    /// （mkfs 种下的，或上一次挂载写行重写的）。
+    fn instance_table_chain(&self) -> &InstanceTableChain {
         match self {
             PreviousVersion::WithoutFile { table, .. }
-            | PreviousVersion::WithFile { table, .. } => &table.records,
+            | PreviousVersion::WithFile { table, .. } => table,
         }
     }
+}
 
-    /// 这一版的实例表链有几片（取号之前的准入按它判：多于一片的链第一版不重写）。
-    fn instance_table_pages(&self) -> usize {
-        match self {
-            PreviousVersion::WithoutFile { table, .. }
-            | PreviousVersion::WithFile { table, .. } => table.page_pointers.len(),
-        }
+/// 写行那次发布的实例表链怎么重写：这一版那张表的行后面接上这次写的行，被换下的是这一版的整条链
+/// （D18（块里携带什么信息） 已定项 11「每次写行 COW 重写整条链」）。取号之前的准入与写行那次发布读的都是它，两处同一份输入。
+fn instance_table_rewrite_of_the_row_publish(
+    table: &InstanceTableChain,
+    rows_written: &[InstanceRow],
+) -> InstanceTableRewrite {
+    let mut rows = table.records.rows.clone();
+    rows.extend_from_slice(rows_written);
+    InstanceTableRewrite {
+        rows,
+        replaced_chain: table.page_pointers.clone(),
     }
 }
 
@@ -883,7 +891,7 @@ pub struct RaisedFloor {
 ///
 /// # Errors
 /// 现行那一版的根指着的实例表读不出或解不开（`InstanceTableMalformed`）；`new_floor` 超过上限；
-/// 那一串空发布里有一次发不出去（`RaiseFloorSequencePublishFailed`，带着前面已经落盘了几次）。
+/// 那一串空发布里有一次发不出去（`RaiseFloorSequencePublishFailed`，带着前面已经落盘的那几次与失败那一次的写账）。
 pub fn raise_rollback_floor<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
@@ -963,7 +971,6 @@ pub fn raise_rollback_floor<Device: BlockDevice>(
         .any(|identity| !covered.contains(identity))
         && u64::try_from(publishes.len()).expect("次数") < ROOT_RING_REGIONS
     {
-        let publishes_persisted = publishes.len();
         let next = publish_version(
             &mut pool,
             allocator,
@@ -986,9 +993,16 @@ pub fn raise_rollback_floor<Device: BlockDevice>(
             },
             Some(&*current),
         )
-        .map_err(|cause| MountError::RaiseFloorSequencePublishFailed {
-            publishes_persisted,
-            cause,
+        // 抬 F 的写入口是这里开的、随错一起丢掉：已经落盘的那几次各自的写与失败那一次已记的写都随错交出（增补 2 收口表第 58 行）。
+        .map_err(|cause| {
+            MountError::RaiseFloorSequencePublishFailed(publish_sequence_failed(
+                &pool,
+                publishes
+                    .iter()
+                    .map(|persisted: &TransactionOutput| persisted.writes.clone())
+                    .collect(),
+                cause,
+            ))
         })?;
         let device = device_of_txg(next.root.checkpoint_txg);
         if !covered.contains(&device) {
@@ -1017,18 +1031,15 @@ struct RowPublishIdentity {
     tree_identifier_watermark: u64,
 }
 
-/// 写行那次发布、上一版带文件：在上一版那张实例表后面接上这次写的行，重写实例表与四个固定点单元（D18（块里携带什么信息） 已定项 11；
+/// 写行那次发布、上一版带文件：在上一版那张实例表后面接上这次写的行，重写整条实例表链与四个固定点单元（D18（块里携带什么信息） 已定项 11；
 /// 记账树已经存在 ⇒ 空发布也重写固定点单元，D16（发布语义） 已定项 9）。事务号 0、本实例第一条反向链 0。
 fn publish_rows_on_file_version<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
     previous: &TransactionOutput,
-    table: &InstanceTableRecords,
-    rows_written: &[InstanceRow],
+    instance_table: InstanceTableRewrite,
     identity: RowPublishIdentity,
 ) -> Result<TransactionOutput, PublishError> {
-    let mut table_after = table.clone();
-    table_after.rows.extend_from_slice(rows_written);
     publish_version(
         pool,
         allocator,
@@ -1043,7 +1054,7 @@ fn publish_rows_on_file_version<Device: BlockDevice>(
             file: None,
             // 写行那次发布不碰 inode 树。
             new_inode_records: &[],
-            instance_table: InstanceTablePlan::Rewrite(table_after.to_records()),
+            instance_table: InstanceTablePlan::Rewrite(instance_table),
             tree_birth_txg: previous.tree_birth_txg(),
             tree_identifier_watermark: identity.tree_identifier_watermark,
             rollback_floor: identity.rollback_floor,
@@ -1129,25 +1140,42 @@ fn publish_empty_after<Device: BlockDevice>(
 /// 在每一次之后都要装得下，前面几次要新增的分配记录算进后面几次的基数；算不过就在任何写之前返回——取号写出去的实例代号一去不回，
 /// 回卷只管取号自己那几次写报错，管不到取号之后的发布失败。
 /// 读的是内存里这个分配器，与发布路径那一遍同一份输入（取号不碰它），发布路径那一遍仍在、是动分配器之前的最后一道。
-/// 再加一项实例表（代码三方第二轮 Z1-a），按片数判：写行那次发布重写的实例表 = 这一版的行 + `rows_to_write` 行，每片末尾一条链指针记录
-/// （`INSTANCE_TABLE_PAGE_RECORDS`，D18（块里携带什么信息） 已定项 11）；这一版的链已经多于一片、或这次之后要多于一片，都拒
-/// （多于一片怎么写没有条款，`InstanceTableChainLongerThanOnePageUndecided`）。行数与片数读的是 `start.previous` 里那一版实例表
-/// （从根记录沿链读的整条链）、要写的行由调用方按这次要取的号列出来——写行那次发布拼的正是这两样（`publish_rows_on_file_version`
-/// 在那一版后面接上这几行），号在取号写之前重算、不等就不写（`acquire_expected_instance`），所以这里判的与写出去的是同一张表。
+/// 实例表按写行那次发布要写的那条链算（`instance_table_rewrite`：这一版的行接上这次写的行、被换下的是这一版的整条链；
+/// 写行那次发布拼的正是它，号在取号写之前重算、不等就不写（`acquire_expected_instance`），所以这里判的与写出去的是同一张表）：
+/// 这次之后几片，写行那次发布就重写几个实例表角色（行一片写满 369 行再开下一片，D18（块里携带什么信息） 已定项 11）；
+/// 被换下的那条旧链逐片按释放判定路径核三样（`instance_table_chain_to_release`，只查不改），核不过同样在取号之前拒——
+/// 旧链上有一片不在这一版的分配记录里（只有别的实现写的或坏镜像上有），取号之后才在发布路径里报出来，号就烧了。
 /// 可写挂载与回退各按自己的 `start` 算：回退的那一版是 R_old 指着的表、要写的是 [max(r_old, 1), 新实例)。
-/// **实例表那一项两臂都判**：树表 0 条的一版上写行同样重写整张实例表（`publish_instance_table_on_version_without_file`），
-/// 多于一片同样走不下去；那一版上要写的行数没有上界——每次挂载崩在取号之后、写行之前，下一次要写的区间就多一个实例。
+/// **旧链那一项两臂都判**：树表 0 条的一版上写行同样重写整条链（`publish_instance_table_on_version_without_file`）；
+/// 那一版上要写的行为空时写行那次是零单元发布、不碰实例表，不判。
 /// 分配记录树、记账树与中央映射树那三条只在带文件的一版上判：树表 0 条 ⇒ 这一版没有这三棵树，写行与暖机都不写它们
 /// （D16（发布语义） 已定项 9 那五样一样都不写）。
 fn refuse_publishes_before_acquisition_that_do_not_pass_admission(
     allocator: &PoolAllocator,
     start: &InstanceStart,
     instance_to_acquire: InstanceGeneration,
+    instance_table_rewrite: &InstanceTableRewrite,
     rows_to_write: usize,
     warm_up_publishes_planned: usize,
 ) -> Result<(), MountError> {
+    let row_publish_rewrites_the_instance_table = match &start.previous {
+        PreviousVersion::WithFile { .. } => true,
+        PreviousVersion::WithoutFile { .. } => rows_to_write > 0,
+    };
+    if row_publish_rewrites_the_instance_table {
+        instance_table_chain_to_release(&instance_table_rewrite.replaced_chain, allocator)
+            .map_err(
+                |cause| MountError::RowPublishAdmissionRefusedBeforeAcquisition {
+                    instance_to_acquire,
+                    cause,
+                },
+            )?;
+    }
     if let PreviousVersion::WithFile { output, .. } = &start.previous {
-        let shapes: Vec<PublishShape> = std::iter::once(PublishShape::ROW_PUBLISH)
+        let shapes: Vec<PublishShape> =
+            std::iter::once(PublishShape::row_publish_rewriting_instance_table_pages(
+                instance_table_rewrite.pages_after_this_publish(),
+            ))
             .chain(std::iter::repeat_n(
                 PublishShape::EMPTY_PUBLISH,
                 warm_up_publishes_planned,
@@ -1172,27 +1200,183 @@ fn refuse_publishes_before_acquisition_that_do_not_pass_admission(
             },
         )?;
     }
-    // 实例表按片数算（D18（块里携带什么信息） 已定项 11：一片 370 条，链指针记录恒为最后一条）。多于一片的链怎么写有两处条款没写
-    // （`InstanceTableChainLongerThanOnePageUndecided` 的文档注释），第一版只写一片：这一版的链已经多于一片、或这次之后要多于一片，
-    // 都在这里拒。只有一片时写行那次发布只重写一个实例表单元，上面那串准入按一个角色数，与发布路径一致。
-    let records_per_page = usize::try_from(INSTANCE_TABLE_PAGE_RECORDS).expect("370");
-    let rows_in_version = start.previous.instance_table().rows.len();
-    let pages_in_version = start.previous.instance_table_pages();
-    let chain_longer_than_one_page = || MountError::InstanceTableChainLongerThanOnePageUndecided {
-        instance_to_acquire,
-        rows_in_version,
-        pages_in_version,
-        rows_to_write,
-        pages_after_this_publish: instance_table_pages_for_rows(rows_in_version + rows_to_write),
-    };
-    if pages_in_version > 1 {
-        return Err(chain_longer_than_one_page());
+    Ok(())
+}
+
+/// 取号之后那一串里一次发布取到的落点：这次重写的角色，按取落点的次序（bump 次序）各一个槽。零单元的发布一个都不取。
+type PlacementsTakenByOnePublish = Vec<(TransactionUnit, SlotNumber)>;
+
+/// 取号之后那一串里第几次发布（从 0 数：0 是写行那次，之后是暖机）的哪个角色在分配器的拷贝上取不到落点、分配器给的原因。
+struct PlacementRefusedOnTheCopy {
+    publish_index: usize,
+    unit: TransactionUnit,
+    refusal: PlacementRefusal,
+}
+
+/// 在分配器的拷贝上走那一串的结局。
+enum PlacementsOnTheCopy {
+    /// 每一次的每个角色都取到了：按次序交回每一次取到的。
+    TakenByEveryPublish(Vec<PlacementsTakenByOnePublish>),
+    Refused(PlacementRefusedOnTheCopy),
+    /// 写行那次（带文件的一版上）经映射查换下的落点就报错了（`transaction::placements_to_release_via_mapping` 的那几种）：
+    /// 发布路径走到同一处报同一个错（它只读内存里的上一版与分配器，取号不碰这两样），这里不判、不拦，照旧交给发布路径报。
+    ReleaseCheckFailedBeforeTheFirstPlacement,
+}
+
+/// 一个角色在分配器上取落点：与发布路径同一条政策（`TransactionUnit::placement`：用户数据按政策函数，其余是提交内生块）。
+fn take_placement_for_role(
+    allocator: &mut PoolAllocator,
+    role: TransactionUnit,
+    txg: CheckpointTxg,
+) -> Result<Placement, PlacementRefusal> {
+    match role.placement() {
+        PlacementRule::UserData => allocator.try_allocate_user_data(txg),
+        PlacementRule::CommitGenerated(footprint) => {
+            allocator.try_allocate_commit_generated(footprint, txg)
+        }
     }
-    // 这次之后多于一片 ⟺ 行数 + 链指针记录 > 一片的记录数（`instance_table_pages_for_rows` 的 ⌈行数 / 369⌉ > 1 同一个判定）。
-    if rows_in_version + rows_to_write + 1 > records_per_page {
-        return Err(chain_longer_than_one_page());
+}
+
+/// 取号之前把这次挂载取号之后要发的那一串——写行一次、暖机 `warm_up_publish_txgs` 那几次——在分配器的一份拷贝上逐次取一遍落点。
+/// 只动拷贝、不碰盘、不读盘。
+///
+/// 动分配器的步骤与发布路径逐步相同（`transaction::publish_version` 与 `transaction::publish_instance_table_on_version_without_file`）：
+/// 先释放这一次换下的落点（进 defer、槽仍占着），再按 bump 次序给这次重写的每个角色取落点，取完记这条根盖掉的根环槽
+/// （`PoolAllocator::record_root_written_by_this_process`：根环转过就按谓词回收、清离开根环的被抛弃根的隔离位）。重写的角色：
+/// 带文件的一版上写行是 `PublishShape::row_publish_rewriting_instance_table_pages`（这次之后实例表链几片就重写几个实例表角色，
+/// 尾片先）、暖机是 `PublishShape::EMPTY_PUBLISH`（`publish_empty_after` 那条断言钉住暖机与它相同），
+/// 暖机换下的就是上一次在拷贝上取到的那几个；写行换下的是这一版的整条实例表链（`transaction::instance_table_chain_to_release`，
+/// 排最前）加经映射查到的那几个，与 `transaction::publish_version` 同序。树表 0 条那一版上写行是实例表链各片（尾片先）加分配记录树节点
+/// （这次没有行要写时是零单元发布），暖机是零单元发布。那一版上写行换下的旧链与分配记录树节点不在这里释放：它之后只有零单元发布，
+/// 释放不释放，取到的落点都一样。
+///
+/// 发布路径在取落点之前还读盘核换下的每一份的校验和，对不上（读不出也算）的那一份在它那块盘上的分配记录留在已分配、不释放
+/// （`transaction::copies_failing_the_release_checksum_check`，D19（块指针的结构与宽度预算） 已定项 5），这里不做（它要读盘）、照常释放。
+/// 两边只在那一槽这一串里被回收时分叉：拷贝上它回收了、可能再发出去，真发时它一直占着；这一串里回收它，得这一串自己的根把根环里
+/// 比它旧的有效根全盖掉（回退到环里最旧的那条根、其余全被抛弃时走得到）。所以这一串里没有一份核出对不上时，这里取到的与真发起来取到的逐项相同
+/// （`establish_instance` 在这一串发完之后断言）；有一份核出对不上时可能不同，那一格见 `establish_instance` 的注释。
+fn placements_of_the_publishes_after_acquisition_on_a_copy(
+    allocator: &PoolAllocator,
+    previous: &PreviousVersion,
+    instance_table_rewrite: &InstanceTableRewrite,
+    rows_to_write: usize,
+    row_publish_txg: CheckpointTxg,
+    warm_up_publish_txgs: &[CheckpointTxg],
+) -> PlacementsOnTheCopy {
+    let mut copy = allocator.clone();
+    let (roles_of_the_row_publish, released_by_the_row_publish, roles_of_a_warm_up_publish) =
+        match previous {
+            PreviousVersion::WithFile { output, .. } => {
+                let roles = PublishShape::row_publish_rewriting_instance_table_pages(
+                    instance_table_rewrite.pages_after_this_publish(),
+                )
+                .rewritten_roles();
+                let Ok(mut released) =
+                    instance_table_chain_to_release(&instance_table_rewrite.replaced_chain, &copy)
+                else {
+                    return PlacementsOnTheCopy::ReleaseCheckFailedBeforeTheFirstPlacement;
+                };
+                let Ok(released_via_mapping) =
+                    placements_to_release_via_mapping(output, &copy, &roles)
+                else {
+                    return PlacementsOnTheCopy::ReleaseCheckFailedBeforeTheFirstPlacement;
+                };
+                released.extend(released_via_mapping);
+                (
+                    roles,
+                    released,
+                    PublishShape::EMPTY_PUBLISH.rewritten_roles(),
+                )
+            }
+            PreviousVersion::WithoutFile { .. } if rows_to_write == 0 => {
+                (Vec::new(), Vec::new(), Vec::new())
+            }
+            PreviousVersion::WithoutFile { .. } => (
+                instance_table_page_roles_in_bump_order(
+                    instance_table_rewrite.pages_after_this_publish(),
+                )
+                .into_iter()
+                .chain(std::iter::once(TransactionUnit::AllocationTree))
+                .collect(),
+                Vec::new(),
+                Vec::new(),
+            ),
+        };
+    let txgs = std::iter::once(row_publish_txg).chain(warm_up_publish_txgs.iter().copied());
+    let mut released_by_the_next_publish = released_by_the_row_publish;
+    let mut taken_by_every_publish: Vec<PlacementsTakenByOnePublish> = Vec::new();
+    for (publish_index, txg) in txgs.enumerate() {
+        let roles = if publish_index == 0 {
+            &roles_of_the_row_publish
+        } else {
+            &roles_of_a_warm_up_publish
+        };
+        for placement in std::mem::take(&mut released_by_the_next_publish) {
+            copy.release(placement, txg);
+        }
+        let mut taken_by_this_publish: PlacementsTakenByOnePublish = Vec::new();
+        for role in roles {
+            match take_placement_for_role(&mut copy, *role, txg) {
+                Ok(placement) => taken_by_this_publish.push((*role, placement.slot)),
+                Err(refusal) => {
+                    return PlacementsOnTheCopy::Refused(PlacementRefusedOnTheCopy {
+                        publish_index,
+                        unit: *role,
+                        refusal,
+                    })
+                }
+            }
+        }
+        copy.record_root_written_by_this_process(txg);
+        // 下一次（暖机）换下的是这一次取到的那几个同角色的落点：这一次重写了下一次要重写的每个角色。
+        released_by_the_next_publish = roles_of_a_warm_up_publish
+            .iter()
+            .filter_map(|role| {
+                taken_by_this_publish
+                    .iter()
+                    .find(|(taken_role, _)| taken_role == role)
+                    .map(|(_, slot)| Placement {
+                        slot: *slot,
+                        span: role.span_slots(),
+                    })
+            })
+            .collect();
+        taken_by_every_publish.push(taken_by_this_publish);
+    }
+    PlacementsOnTheCopy::TakenByEveryPublish(taken_by_every_publish)
+}
+
+/// 一次发布真取到的落点，按取落点的次序：带文件的一版是这次重写的每个角色（`TransactionOutput::rewritten`）；
+/// 树表 0 条的一版是这次那条记录的点名项：写行那次按取落点的次序点名实例表链各片（尾片先）与分配记录树节点
+/// （`transaction::publish_instance_table_on_version_without_file`；第 1 片起的指针不在根记录里，只在记录的点名项与上一片的链指针里），
+/// 零单元发布一项都不点名、一个都没有。
+fn placements_taken_by(version: &PoolVersion) -> PlacementsTakenByOnePublish {
+    match version {
+        PoolVersion::WithFile(output) => output
+            .rewritten
+            .iter()
+            .map(|role| (*role, output.unit(*role).slot))
+            .collect(),
+        PoolVersion::WithoutFile(version_without_file) => {
+            let named = &version_without_file.record.named;
+            let Some(instance_table_pages) = named.len().checked_sub(1) else {
+                return Vec::new();
+            };
+            instance_table_page_roles_in_bump_order(instance_table_pages)
+                .into_iter()
+                .chain(std::iter::once(TransactionUnit::AllocationTree))
+                .zip(named)
+                .map(|(role, named_unit)| {
+                (
+                    role,
+                    slot_shared_by_both_location_entries(&named_unit.locations).expect(
+                        "这个进程这次发布刚写的指针，两条位置条目按一个落点写（两盘同槽，`PoolWriter::location_entries`）",
+                    ),
+                )
+            })
+            .collect()
+        }
     }
-    Ok(())
 }
 
 /// 暖机要推的那几次空发布，按它们的 checkpoint_txg 列出来（D16（发布语义） 已定项 8 戊）：从写行那次发布的 txg 起逐个加一，
@@ -1258,16 +1442,29 @@ fn instance_rows_to_write(
         .collect()
 }
 
-/// 取号之后的一次发布失败了：错，连同这次挂载的写入口交得出的账——`persisted` 是这次挂载里已经落盘的那几次（按先后），
-/// 失败那一次落盘阶段已记的写从写入口的失败账取。
+/// 一串发布里有一次失败了：错，连同这一串的写入口交得出的账——`writes_of_persisted_publishes` 是这一串里已经落盘的那几次各自的写
+/// （按先后），失败那一次落盘阶段已记的写从写入口的失败账取。可写挂载与抬 F 共用这一处。
+fn publish_sequence_failed<Device: BlockDevice>(
+    pool: &PoolWriter<'_, Device>,
+    writes_of_persisted_publishes: Vec<WritesByStructureKind>,
+    cause: PublishError,
+) -> PublishSequenceFailed {
+    PublishSequenceFailed {
+        cause,
+        writes_of_persisted_publishes,
+        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),
+    }
+}
+
+/// 取号之后的一次发布失败了：错，连同这次挂载的写入口交得出的账——`persisted` 是这次挂载里已经落盘的那几次（按先后）。
 fn publish_failed_after_acquisition<Device: BlockDevice>(
     pool: &PoolWriter<'_, Device>,
     persisted: &[&PoolVersion],
     cause: PublishError,
 ) -> MountError {
-    MountError::Publish(PublishAfterAcquisitionFailed {
-        cause,
-        writes_of_persisted_publishes: persisted
+    MountError::Publish(publish_sequence_failed(
+        pool,
+        persisted
             .iter()
             .map(|version| match version {
                 PoolVersion::WithoutFile(version_without_file) => {
@@ -1276,8 +1473,8 @@ fn publish_failed_after_acquisition<Device: BlockDevice>(
                 PoolVersion::WithFile(file_version) => file_version.writes.clone(),
             })
             .collect(),
-        writes_of_failed_publishes: pool.writes_of_failed_publishes().to_vec(),
-    })
+        cause,
+    ))
 }
 
 /// 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘：可写挂载与回退共用的后半段。
@@ -1300,13 +1497,42 @@ fn establish_instance<Device: BlockDevice>(
     let instance_to_acquire = instance_generation_to_acquire(&pool);
     // 要写的行按要取的号先列出来：实例表那一项准入按它算，取号之后写行也用这一份（号在写之前重算、不等就不写）。
     let rows_written = instance_rows_to_write(&start.previous_row, instance_to_acquire);
+    let instance_table_rewrite = instance_table_rewrite_of_the_row_publish(
+        start.previous.instance_table_chain(),
+        &rows_written,
+    );
     refuse_publishes_before_acquisition_that_do_not_pass_admission(
         &allocator,
         &start,
         instance_to_acquire,
+        &instance_table_rewrite,
         rows_written.len(),
         warm_up_publishes_planned.len(),
     )?;
+    // 这一串自己的落点也在取号之前先取一遍（分配器的拷贝上，与后面真发读同一个分配器——取号不碰它）：取不到就在任何写之前返回。
+    // 怎么把它们算进取号之前的准入，条款没定（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided` 的文档注释）。
+    let placements_planned = match placements_of_the_publishes_after_acquisition_on_a_copy(
+        &allocator,
+        &start.previous,
+        &instance_table_rewrite,
+        rows_written.len(),
+        start.first_txg,
+        &warm_up_publishes_planned,
+    ) {
+        PlacementsOnTheCopy::TakenByEveryPublish(taken) => Some(taken),
+        PlacementsOnTheCopy::Refused(refused) => {
+            return Err(
+                MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
+                    instance_to_acquire,
+                    publish_index: refused.publish_index,
+                    warm_up_publishes_planned: warm_up_publishes_planned.len(),
+                    unit: refused.unit,
+                    refusal: refused.refusal,
+                },
+            )
+        }
+        PlacementsOnTheCopy::ReleaseCheckFailedBeforeTheFirstPlacement => None,
+    };
     let instance = acquire_expected_instance(&mut pool, instance_to_acquire).map_err(
         |failure| match failure {
             ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite {
@@ -1326,15 +1552,14 @@ fn establish_instance<Device: BlockDevice>(
         "acquire_expected_instance 写之前重算、与判定时的号不等就不写：交回的就是列行与判准入用的那个号"
     );
 
-    // 取号之后的每一次发布失败都带着这次挂载的写入口交得出的账返回（`PublishAfterAcquisitionFailed`）：写入口随错一起丢掉。
+    // 取号之后的每一次发布失败都带着这次挂载的写入口交得出的账返回（`PublishSequenceFailed`）：写入口随错一起丢掉。
     let row_publish = match &start.previous {
-        PreviousVersion::WithFile { output, table } => {
+        PreviousVersion::WithFile { output, .. } => {
             let published = publish_rows_on_file_version(
                 &mut pool,
                 &mut allocator,
                 output,
-                &table.records,
-                &rows_written,
+                instance_table_rewrite,
                 RowPublishIdentity {
                     txg: start.first_txg,
                     counter: start.next_counter,
@@ -1369,10 +1594,7 @@ fn establish_instance<Device: BlockDevice>(
                 PoolVersion::WithoutFile(published)
             })
         }
-        PreviousVersion::WithoutFile { root, table } => {
-            let mut table_after = table.records.clone();
-            table_after.rows.extend_from_slice(&rows_written);
-            let instance_table_records = table_after.to_records();
+        PreviousVersion::WithoutFile { root, .. } => {
             publish_instance_table_on_version_without_file(
                 &mut pool,
                 &mut allocator,
@@ -1384,7 +1606,7 @@ fn establish_instance<Device: BlockDevice>(
                     // 新实例的第一条记录，反向链恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
                     back_chain: 0,
                     rollback_floor: start.effective_floor,
-                    instance_table_records: &instance_table_records,
+                    instance_table: &instance_table_rewrite,
                     tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                 },
             )
@@ -1419,6 +1641,28 @@ fn establish_instance<Device: BlockDevice>(
         warm_up_publishes.push(next.clone());
         current = next;
     }
+    // 取号之前在拷贝上取的落点就是真发起来取的：同一个分配器、同一串分配动作。这一串里有一份换下的单元读盘核校验和对不上时不比——
+    // 真发时那一份的分配记录留在已分配，拷贝上没做那一步、照常释放，那一槽若在这一串里被回收（回退到环里最旧的那条根、其余全被抛弃），两边可以不同
+    // （`placements_of_the_publishes_after_acquisition_on_a_copy` 的文档注释）；那一格取号之前判的与真发的不是同一串，是这一版留着的缺口。
+    let placements_taken: Vec<PlacementsTakenByOnePublish> = std::iter::once(&row_publish)
+        .chain(&warm_up_publishes)
+        .map(placements_taken_by)
+        .collect();
+    let some_copy_was_quarantined = std::iter::once(&row_publish)
+        .chain(&warm_up_publishes)
+        .filter_map(PoolVersion::file_version)
+        .any(|version| {
+            !version
+                .quarantined_after_release_checksum_mismatch
+                .is_empty()
+        });
+    if let (Some(planned), false) = (&placements_planned, some_copy_was_quarantined) {
+        assert_eq!(
+            &placements_taken, planned,
+            "取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；\
+             不同说明两处的角色、次序或记根分叉了，取号之前那一判判的不是真发的那一串"
+        );
+    }
 
     Ok(Mounted {
         output: MountOutput {
@@ -1443,7 +1687,8 @@ fn establish_instance<Device: BlockDevice>(
 ///
 /// # Errors
 /// 恢复失败、所选根下有文件而环里一条记录都没有、实例表沿链读不出或解不开、取号之前的准入不过（分配记录树、记账树、
-/// 实例表多于一片）、取号失败、发布失败。
+/// 实例表多于一片）、取号之后那一串在分配器的拷贝上取不到落点（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`）、
+/// 取号失败、发布失败。
 pub fn mount_writable<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
@@ -1459,7 +1704,7 @@ pub fn mount_writable<Device: BlockDevice>(
         &records,
         true,
         rollback_high_water_of_root(&*devices, &chosen_root),
-    );
+    )?;
     // 所选根覆盖的最后一条记录读得出就拿它当上一版的记录：同实例、同 checkpoint_txg 的记录里 jsn 最大的那条
     // （D23（journal 的角色与格式） 已定项 14 注 1，P6 2026-09-23 定；一次发布切成多条记录时那次发布的末条）。
     // 读不出（两份都撕了）就拿最大 jsn 那条顶着——本实例的第一条反向链恒 0、
@@ -1533,7 +1778,8 @@ pub fn mount_writable<Device: BlockDevice>(
 /// 新实例的第一条 jsn 接在环里最大的 jsn 之后（C340（回退之后记录链从哪条之后接没有定义） 取 P2，预想、等用户定）。
 ///
 /// # Errors
-/// 目标根不在根环、不在候选集、它自己那条记录读不出、取号之前的准入不过（同可写挂载，实例表按 R_old 那一版算）、取号或发布失败。
+/// 目标根不在根环、不在候选集、它自己那条记录读不出、取号之前的准入不过（同可写挂载，实例表按 R_old 那一版算）、
+/// 取号之后那一串在分配器的拷贝上取不到落点（同可写挂载）、取号或发布失败。
 pub fn mount_rollback<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut Vec<(DeviceIdentity, Device)>,
diff --git a/crates/singlefs-core/src/mounted_read.rs b/crates/singlefs-core/src/mounted_read.rs
index a152aa9..315a437 100644
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -586,7 +586,8 @@ pub fn mount_read_only(reader: &dyn PoolReader) -> Result<MountedReadOnly, Mount
         &records,
         true,
         rollback_high_water_of_root(reader, &chosen_root),
-    );
+    )
+    .map_err(MountReadOnlyFailure::Recovery)?;
     let mounted =
         open_pool_for_read(reader, &effective_root).map_err(MountReadOnlyFailure::Open)?;
     Ok(MountedReadOnly {
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 4722501..f8fed26 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -29,7 +29,9 @@ use crate::inode_tree::{InodeLeafContainer, InodeLeafContainerIndexInTree};
 use crate::instance_table::{
     InstanceTableChainRecord, InstanceTablePage, InstanceTablePageIndex, InstanceTableRecords,
 };
-use crate::journal::{JournalRecord, JournalRecordOrdinalWithinPublish};
+use crate::journal::{
+    JournalRecord, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish,
+};
 use crate::make_filesystem::TREE_TABLE_KEY_WIDTH;
 use crate::pointer::{DataPointer, LocationEntry, NodePointer};
 use crate::records::{
@@ -188,6 +190,16 @@ pub enum RecoveryFailure {
     MappingMiss { slot: SlotNumber },
     /// 位置提示读不到、经映射仍读不到。
     MappingStillUnreadable { slot: SlotNumber },
+    /// 所选根那次发布（与所选根同实例、同 checkpoint_txg 的记录）读得出的几条里，带「本次发布末条」标志的多于一条：
+    /// 链首的锚点「所选根覆盖的最后一条」按末条标志认（D23（journal 的角色与格式） 已定项 14 注 1，读法乙），
+    /// 两条以上都带时认哪一条**条款没有写** ⇒ 第一版不支持：恢复在施加任何一条记录之前停下（挂载因此在任何落盘动作之前返回）。
+    /// 写者每次发布只给真正的最后一条带标志（已定项 17），走到这里要一条记录坏了而校验和恰好仍对得上，或者镜像是改出来的。
+    /// `counters` 是带标志的那几条的 jsn 计数器，按升序。
+    RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+        instance: InstanceGeneration,
+        checkpoint_txg: CheckpointTxg,
+        counters: Vec<u64>,
+    },
 }
 
 /// 恢复的结果：择到的根下面没有文件（第 0 代）、读回文件、或走不下去。`root` 恒是**所选**的那条根。
@@ -224,7 +236,8 @@ pub struct JournalScanReport {
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct RecoveryReport {
     pub outcome: RecoveryOutcome,
-    /// 施加 journal 之后实际走的那条根（所选根，或由记录重建的根）；择不到根时是 None。记录核对器拿它判「恢复自称的状态」。
+    /// 施加 journal 之后实际走的那条根（所选根，或由记录重建的根）；择不到根、或恢复在施加任何记录之前停下
+    /// （`RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`）时是 None。记录核对器拿它判「恢复自称的状态」。
     pub effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
     pub journal: JournalScanReport,
     /// 位置提示读不到、转去查映射的次数。
@@ -360,6 +373,126 @@ fn central_mapping_locations_among_entries(
     Ok(mapped)
 }
 
+/// 按数据指针里的位置提示读一个码 1 数据单元；两条提示都读不出（读不回字节，或整单元校验和对不上）时，按它的码 1 映射 key
+/// 查中央映射、照映射里的两条位置条目再读一次（D19（块指针的结构与宽度预算） 已定项 8：豁免三类之外的单元位置提示读不出时一律经映射回退）。
+/// 冷启动走读与从盘上重建上一版走这同一条；每回退一次 `data_unit_stale_location_hint_hops` 加一（已定项 5 硬规则 3 的观测点）。
+/// 数据单元读不出不是这里的错：交回 [`DataUnitReadThroughTheCentralMapping`] 的两种读不出，由调用方按各自的条款处置。
+///
+/// # Errors
+/// 查映射本身判红的（映射树根读不出、条目窄于字段表），原样交回。
+fn read_data_unit_via_hint_then_central_mapping(
+    reader: &dyn PoolReader,
+    data_pointer: &DataPointer,
+    central_mapping_locations_of_key: &CentralMappingLocationsOfKey<'_>,
+    data_unit_stale_location_hint_hops: &mut usize,
+) -> Result<DataUnitReadThroughTheCentralMapping, RecoveryFailure> {
+    let data_unit_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
+    if let Ok(bytes) = read_unit_via_locations(reader, &data_pointer.locations, data_unit_bytes) {
+        return Ok(DataUnitReadThroughTheCentralMapping::Content(bytes));
+    }
+    *data_unit_stale_location_hint_hops += 1;
+    let mapping_key = mapping_key_for_data(data_pointer.head, data_pointer.write_order);
+    let Some(mapped_data_unit_locations) = central_mapping_locations_of_key(&mapping_key)? else {
+        return Ok(
+            DataUnitReadThroughTheCentralMapping::MissingFromTheMapping {
+                hint_slot: data_pointer.locations[0].slot,
+            },
+        );
+    };
+    Ok(
+        match read_unit_via_locations(reader, &mapped_data_unit_locations, data_unit_bytes) {
+            Ok(bytes) => DataUnitReadThroughTheCentralMapping::Content(bytes),
+            Err(_still_unreadable) => {
+                DataUnitReadThroughTheCentralMapping::UnreadableAtTheMappedLocation {
+                    mapped_slot: mapped_data_unit_locations[0].slot,
+                }
+            }
+        },
+    )
+}
+
+/// 一个数据单元按位置提示、再经中央映射读下来的结局（[`read_data_unit_via_hint_then_central_mapping`]）。
+/// 读不出的两种各自带着冷走读报错要点名的那个槽；冷走读把它们报成错，从盘上重建上一版照抄位置项、不读内容。
+enum DataUnitReadThroughTheCentralMapping {
+    Content(Vec<u8>),
+    /// 提示读不出，映射里没有它的 key。
+    MissingFromTheMapping {
+        hint_slot: SlotNumber,
+    },
+    /// 提示读不出，映射落点也读不出。
+    UnreadableAtTheMappedLocation {
+        mapped_slot: SlotNumber,
+    },
+}
+
+impl DataUnitReadThroughTheCentralMapping {
+    /// 冷走读的读法：读不出就是这一步走不下去。
+    ///
+    /// # Errors
+    /// 映射里没有 ⇒ `MappingMiss`（带提示的槽）；映射落点也读不出 ⇒ `MappingStillUnreadable`（带映射落点的槽）。
+    fn into_content(self) -> Result<Vec<u8>, RecoveryFailure> {
+        match self {
+            DataUnitReadThroughTheCentralMapping::Content(bytes) => Ok(bytes),
+            DataUnitReadThroughTheCentralMapping::MissingFromTheMapping { hint_slot } => {
+                Err(RecoveryFailure::MappingMiss { slot: hint_slot })
+            }
+            DataUnitReadThroughTheCentralMapping::UnreadableAtTheMappedLocation { mapped_slot } => {
+                Err(RecoveryFailure::MappingStillUnreadable { slot: mapped_slot })
+            }
+        }
+    }
+}
+
+/// 从盘上重建上一版、读一条根的分配记录时用的中央映射树根（自举豁免，只按根记录里的位置条目读，D19（块指针的结构与宽度预算） 已定项 8）：
+/// 到第一次有提示读不出、要经映射回退时，或重建走到映射树根那一步时才读，读过一次就留着——盘上字节连同解开的节点，重建要把字节照抄进上一版。
+/// 不提前读：提示都读得出的镜像上，读序与报错的次序照旧。读法同重建原来读映射树根那一步（只解节点、不核自描述），
+/// 冷走读那一份（`CentralMappingRootReadOnFirstUse`）另核树 ID、key 宽、出生身份与 fsid。
+struct CentralMappingRootWithBytesReadOnFirstUse<'reader> {
+    reader: &'reader dyn PoolReader,
+    mapping_root: NodePointer,
+    bytes_and_node: OnceCell<(Vec<u8>, IndexNodeHeader)>,
+}
+
+impl<'reader> CentralMappingRootWithBytesReadOnFirstUse<'reader> {
+    fn new(reader: &'reader dyn PoolReader, mapping_root: NodePointer) -> Self {
+        Self {
+            reader,
+            mapping_root,
+            bytes_and_node: OnceCell::new(),
+        }
+    }
+
+    fn bytes_and_node(&self) -> Result<&(Vec<u8>, IndexNodeHeader), RecoveryFailure> {
+        if let Some(read) = self.bytes_and_node.get() {
+            return Ok(read);
+        }
+        let bytes = read_unit_via_locations(
+            self.reader,
+            &self.mapping_root.locations,
+            usize::try_from(NODE_BYTES).expect("16384"),
+        )?;
+        let node = parse_index_node(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: "映射树根",
+        })?;
+        Ok(self.bytes_and_node.get_or_init(|| (bytes, node)))
+    }
+
+    fn locations_of_key(
+        &self,
+        mapping_key: &[u8],
+    ) -> Result<Option<[LocationEntry; 2]>, RecoveryFailure> {
+        central_mapping_locations_among_entries(&self.bytes_and_node()?.1.entries, mapping_key)
+    }
+
+    fn into_bytes_and_node(self) -> Result<(Vec<u8>, IndexNodeHeader), RecoveryFailure> {
+        self.bytes_and_node()?;
+        Ok(self
+            .bytes_and_node
+            .into_inner()
+            .expect("上一行刚把映射树根读进来，读不出已经返回了"))
+    }
+}
+
 /// 一个槽读回来之后分三路：读不到 / 自证不过 ⇒ `None`（这一槽不可择，换一槽换一盘还可以试）；
 /// 自述的每区槽数 S 越界 ⇒ 整池拒绝挂载，把点名的成员交回去；三关都过 ⇒ `Some`。
 ///
@@ -732,9 +865,13 @@ fn allocation_records_fit_the_pool_geometry(
 }
 
 /// 一条根引用的分配记录（树表 → 分配记录树根节点）：回退的影子账要读每条被抛弃根的账。第 0 代树表（没有分配记录树）给空。
+/// 分配记录树根的位置提示读不出时经这条根的中央映射回退（D19（块指针的结构与宽度预算） 已定项 8，与冷走读同一条
+/// `read_mapped_tree_node_via_hint_then_central_mapping`）；树表是自举豁免，只按根记录里的位置条目读。
+/// 回退的次数这里不交出去：调用方（影子账、重建分配器）今天没有接多跳观测点的口子。
 ///
 /// # Errors
-/// 树表或分配记录树根读不到、解不开；条目宽或结构值判红（见 [`allocation_records_of_node`]）。
+/// 树表读不到、解不开；分配记录树根提示读不出且映射里没有它（`MappingMiss`）、映射落点也读不出（`MappingStillUnreadable`）、
+/// 映射树根读不出或解不开、分配记录树根解不开；条目宽或结构值判红（见 [`allocation_records_of_node`]）。
 pub fn allocation_records_under_root(
     reader: &dyn PoolReader,
     root: &RootRecord,
@@ -757,8 +894,16 @@ pub fn allocation_records_under_root(
     let Some(allocation_pointer) = allocation_pointer else {
         return Ok(Vec::new());
     };
-    let allocation_bytes =
-        read_unit_via_locations(reader, &allocation_pointer.locations, node_bytes)?;
+    let central_mapping_root =
+        CentralMappingRootWithBytesReadOnFirstUse::new(reader, root.mapping_root);
+    let mut stale_location_hint_hops_not_exposed_by_this_reader = 0usize;
+    let allocation_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
+        reader,
+        &allocation_pointer,
+        MappedTreeNodeClass::IndexNode,
+        &|mapping_key: &[u8]| central_mapping_root.locations_of_key(mapping_key),
+        &mut stale_location_hint_hops_not_exposed_by_this_reader,
+    )?;
     let allocation_node =
         parse_index_node(&allocation_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
             what: "分配记录树根",
@@ -978,9 +1123,12 @@ impl From<RecoveryFailure> for RebuildVersionFailure {
 /// （照抄没重写的角色、经映射释放被换下的角色都靠它；可写挂载在恢复之后调）。`record_standing_for_root` 是所选根覆盖的最后一条记录
 /// （同实例、同 checkpoint_txg 里 jsn 最大的那条，D23（journal 的角色与格式） 已定项 14 注 1；读不出时由调用方顶一条；树表 0 条时用不到）。
 /// extent 根兼叶里的每条记录指的数据单元都读回来：一个文件跨多个单元时（并行线一）每个单元一个角色。
+/// 豁免三类之外的单元位置提示读不出时经这一版的中央映射回退（D19（块指针的结构与宽度预算） 已定项 8）；
+/// 提示与映射都读不出的**数据单元**不算失败：照抄它的位置项、不读内容，那一项的字节是空的（D19 已定项 5，用户 2026-09-24 定 N2）。
 ///
 /// # Errors
-/// 树表不是 0 条而 `record_standing_for_root` 是 `None` ⇒ `NoRecordStandingForFileVersion`；单元读不到或解不开 ⇒ 走读同款的错；
+/// 树表不是 0 条而 `record_standing_for_root` 是 `None` ⇒ `NoRecordStandingForFileVersion`；实例表、树表、映射树根读不到，
+/// 树根或 inode 叶容器提示读不出且经映射也读不回（`MappingMiss` / `MappingStillUnreadable`），单元解不开 ⇒ 走读同款的错；
 /// 记账行里没有 inode 号水位那一行 ⇒ `InodeNumberWatermarkRowMissingFromTheAccountingTree`。
 pub fn rebuild_version(
     reader: &dyn PoolReader,
@@ -1044,13 +1192,33 @@ pub fn rebuild_version(
         sparse_side_table: tree_of(TREE_KIND_SPARSE_SIDE_TABLE)?,
         deadlist: tree_of(TREE_KIND_DEADLIST)?,
     };
-    let read_node = |pointer: &NodePointer, what: &'static str| {
-        let bytes = read_unit_via_locations(reader, &pointer.locations, node_bytes)?;
-        let node =
-            parse_index_node(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed { what })?;
-        Ok::<(Vec<u8>, IndexNodeHeader), RecoveryFailure>((bytes, node))
-    };
-    let (extent_bytes, extent_node) = read_node(&extent_pointer, "extent 树根")?;
+    // 豁免三类之外的单元（数据单元、四棵树的根、inode 叶容器）位置提示读不出时经这一版的中央映射回退，与冷走读同一条
+    // （D19（块指针的结构与宽度预算） 已定项 8）；映射树根、树表、实例表是自举豁免，只按根记录里的位置条目读。
+    // 映射树根到第一次要回退、或走到它那一步时才读：提示都读得出的镜像上读序与报错次序照旧。
+    let central_mapping_root =
+        CentralMappingRootWithBytesReadOnFirstUse::new(reader, root.mapping_root);
+    let central_mapping_locations_of_key =
+        |mapping_key: &[u8]| central_mapping_root.locations_of_key(mapping_key);
+    // 回退的次数这里不交出去：`RebuiltVersion` 与可写挂载今天没有接多跳观测点的口子（挂载态的读有，`MountedPoolForRead`）。
+    let mut stale_location_hint_hops_not_exposed_by_this_reader = 0usize;
+    let read_node =
+        |pointer: &NodePointer, what: &'static str, stale_location_hint_hops: &mut usize| {
+            let bytes = read_mapped_tree_node_via_hint_then_central_mapping(
+                reader,
+                pointer,
+                MappedTreeNodeClass::IndexNode,
+                &central_mapping_locations_of_key,
+                stale_location_hint_hops,
+            )?;
+            let node = parse_index_node(&bytes)
+                .map_err(|_error| RecoveryFailure::UnitMalformed { what })?;
+            Ok::<(Vec<u8>, IndexNodeHeader), RecoveryFailure>((bytes, node))
+        };
+    let (extent_bytes, extent_node) = read_node(
+        &extent_pointer,
+        "extent 树根",
+        &mut stale_location_hint_hops_not_exposed_by_this_reader,
+    )?;
     if extent_node.entries.is_empty() {
         return Err(RecoveryFailure::UnitMalformed {
             what: "extent 树根没有记录",
@@ -1059,8 +1227,11 @@ pub fn rebuild_version(
     }
     // extent 根兼叶里的记录按 key 升序，第 i 条就是文件第 i 个数据单元（一个文件跨多个单元，并行线一）；
     // 每条指的数据单元都读回来，照抄进这一版的角色（覆盖写经映射释放它们要这几个 key，写行与暖机照抄它们的字节）。
+    // 提示与映射都读不出的数据单元照抄它的位置项、不读内容，挂载照常，读到那个文件时才报错（D19（块指针的结构与宽度预算）
+    // 已定项 5，用户 2026-09-24 定 N2）：它在这一版 `units` 里那一项的字节是空的。照抄它的发布只搬这一项、不写它的字节，
+    // 重写它的发布按新内容装；释放它的发布照映射条目读盘核（已定项 5 硬规则 1），不读这里的字节。
     let mut data_pointers: Vec<DataPointer> = Vec::with_capacity(extent_node.entries.len());
-    let mut data_unit_bytes_in_file_order: Vec<Vec<u8>> =
+    let mut data_unit_contents_in_file_order: Vec<Vec<u8>> =
         Vec::with_capacity(extent_node.entries.len());
     for extent_record_bytes in &extent_node.entries {
         let (_, data_pointer) = parse_extent_record(extent_record_bytes).ok_or(
@@ -1070,14 +1241,26 @@ pub fn rebuild_version(
                 field_table_bytes: usize::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
             },
         )?;
-        data_unit_bytes_in_file_order.push(read_unit_via_locations(
+        let content_or_nothing_when_unreadable = match read_data_unit_via_hint_then_central_mapping(
             reader,
-            &data_pointer.locations,
-            data_bytes,
-        )?);
+            &data_pointer,
+            &central_mapping_locations_of_key,
+            &mut stale_location_hint_hops_not_exposed_by_this_reader,
+        )? {
+            DataUnitReadThroughTheCentralMapping::Content(bytes) => bytes,
+            DataUnitReadThroughTheCentralMapping::MissingFromTheMapping { .. }
+            | DataUnitReadThroughTheCentralMapping::UnreadableAtTheMappedLocation { .. } => {
+                Vec::new()
+            }
+        };
+        data_unit_contents_in_file_order.push(content_or_nothing_when_unreadable);
         data_pointers.push(data_pointer);
     }
-    let (inode_root_bytes, inode_root_node) = read_node(&inode_root_pointer, "inode 树根")?;
+    let (inode_root_bytes, inode_root_node) = read_node(
+        &inode_root_pointer,
+        "inode 树根",
+        &mut stale_location_hint_hops_not_exposed_by_this_reader,
+    )?;
     if inode_root_node.entries.is_empty() {
         return Err(RecoveryFailure::UnitMalformed {
             what: "inode 树根没有条目",
@@ -1096,7 +1279,13 @@ pub fn rebuild_version(
             entry_bytes: entry.len(),
             field_table_bytes: usize::try_from(INODE_INTERNAL_ENTRY).expect("120"),
         })?;
-        let leaf_bytes = read_unit_via_locations(reader, &leaf_pointer.locations, data_bytes)?;
+        let leaf_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
+            reader,
+            &leaf_pointer,
+            MappedTreeNodeClass::PackedRecordUnit,
+            &central_mapping_locations_of_key,
+            &mut stale_location_hint_hops_not_exposed_by_this_reader,
+        )?;
         let leaf =
             parse_packed_unit(&leaf_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
                 what: "inode 叶容器",
@@ -1133,10 +1322,18 @@ pub fn rebuild_version(
         .ok_or(RecoveryFailure::UnitMalformed {
             what: "inode 树里没有第一个文件那条记录",
         })?;
-    let (allocation_bytes, allocation_node) = read_node(&allocation_pointer, "分配记录树根")?;
+    let (allocation_bytes, allocation_node) = read_node(
+        &allocation_pointer,
+        "分配记录树根",
+        &mut stale_location_hint_hops_not_exposed_by_this_reader,
+    )?;
     let allocation_records: Vec<AllocationRecord> =
         allocation_records_of_node(reader, &allocation_node)?;
-    let (accounting_bytes, accounting_node) = read_node(&accounting_pointer, "记账树根")?;
+    let (accounting_bytes, accounting_node) = read_node(
+        &accounting_pointer,
+        "记账树根",
+        &mut stale_location_hint_hops_not_exposed_by_this_reader,
+    )?;
     let mut accounting_entries: Vec<AccountingEntry> =
         Vec::with_capacity(accounting_node.entries.len());
     for bytes in &accounting_node.entries {
@@ -1156,7 +1353,8 @@ pub fn rebuild_version(
     {
         return Err(RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree.into());
     }
-    let (mapping_bytes, mapping_node) = read_node(&root.mapping_root, "映射树根")?;
+    // 映射树根是自举豁免：不经映射回退，只按根记录里的位置条目读；上面有提示读不出、经映射回退过的，这里拿的就是那时读进来的那一份。
+    let (mapping_bytes, mapping_node) = central_mapping_root.into_bytes_and_node()?;
     let mut mapping_keys: Vec<Vec<u8>> = Vec::with_capacity(mapping_node.entries.len());
     for entry in &mapping_node.entries {
         let (key, _locations) =
@@ -1216,7 +1414,7 @@ pub fn rebuild_version(
         };
     let mut units: Vec<PublishedUnit> = data_pointers
         .iter()
-        .zip(data_unit_bytes_in_file_order)
+        .zip(data_unit_contents_in_file_order)
         .enumerate()
         .map(|(position, (data_pointer, bytes))| {
             unit(
@@ -1451,24 +1649,60 @@ pub fn scan_journal(
     records
 }
 
-/// 一条记录是不是它那次发布的末条（D23（journal 的角色与格式） 已定项 14 第六条「发布边界怎么认」：一次发布的末条 = 点名了
-/// 这次发布共享的提交内生块的那一条，已定项 17：共享内生块只在最后一条点名）。写者的切法（`transaction::roles_named_by_each_record_of_the_publish`）
-/// 让末条之外的每条恰只点名一个数据单元（码 1）⇒ 点名了任何码 1 之外的单元的那条就是末条；一个单元都不点名的记录
-/// （树表 0 条那一版的零单元发布、空发布）不可能是末条之外的那几条，它一条就是一次发布。
+/// 一条记录是不是它那次发布的末条：记录标志位 0（D23（journal 的角色与格式） 已定项 14 第六条「发布边界按记录标志位 0 认」、
+/// 已定项 17：只有真正的最后一条带「本次发布末条」标志，每次只有一条记录的发布——含空发布记录——那一条也带）。
 fn record_ends_its_publish(record: &JournalRecord) -> bool {
-    record.named.is_empty()
-        || record
-            .named
-            .iter()
-            .any(|named| named.unit_class != UNIT_CLASS_DATA)
+    match record.place_in_publish {
+        JournalRecordPlaceInPublish::LastRecordOfThePublish => true,
+        JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => false,
+    }
+}
+
+/// 所选根覆盖的最后一条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 14 注 1，读法乙，用户 2026-09-24 定）：
+/// 「那条」按末条标志认——所选根那次发布（与所选根同实例、同 checkpoint_txg）读得出的几条里带「本次发布末条」标志的那一条。
+/// 一条都不带 ⇒ `Ok(None)`，就算「那条读不出」，链首走「序号为 1 的第一条可读记录」那一支；**不取**读得出的同 txg 记录里
+/// jsn 最大的那条（读法甲：末条读不出而前几条读得出时，它锚在前几条上，下一次发布一条都接不上）。
+///
+/// # Errors
+/// 带标志的多于一条 ⇒ `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`：认哪一条条款没有写。
+fn counter_of_the_last_record_the_root_covers(
+    root: &RootRecord,
+    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
+) -> Result<Option<u64>, RecoveryFailure> {
+    // `records` 按 (实例代号, 计数器) 排序 ⇒ 同一实例里按计数器升序。
+    let counters_carrying_the_last_record_flag: Vec<u64> = records
+        .values()
+        .filter(|record| {
+            record.instance == root.instance
+                && record.checkpoint_txg == root.checkpoint_txg
+                && record_ends_its_publish(record)
+        })
+        .map(|record| record.counter)
+        .collect();
+    if counters_carrying_the_last_record_flag.len() > 1 {
+        return Err(
+            RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+                instance: root.instance,
+                checkpoint_txg: root.checkpoint_txg,
+                counters: counters_carrying_the_last_record_flag,
+            },
+        );
+    }
+    // 至多一条：带标志的那一条就是锚点，一条都没有就是「那条读不出」。
+    Ok(counters_carrying_the_last_record_flag.first().copied())
 }
 
 /// 取前缀并施加（D23（journal 的角色与格式） 已定项 14 / 已定项 15）；返回扫描报告与施加之后的根。
 ///
-/// 链首锚在所选根覆盖的最后一条：与所选根同实例、同 checkpoint_txg 的记录里 jsn 最大的那条（已定项 14 注 1，P6 2026-09-23 定；
-/// 一次发布切成多条记录时它们共享一个 checkpoint_txg，那次发布的末条才是根覆盖到的末端）。
-/// 施加的单位是一次发布（第六条）：前五条判出来的前缀里，一次发布的记录要一直走到它的末条（[`record_ends_its_publish`]）
-/// 才整体施加；前缀停在一次发布中间（末条没到、断号、校验不过、回退行的 W 截在中间、下一条换了 txg）⇒ 那次发布整体不施加。
+/// 链首锚在所选根覆盖的最后一条：所选根那次发布里带「本次发布末条」标志的那一条（已定项 14 注 1，读法乙，
+/// [`counter_of_the_last_record_the_root_covers`]）。
+/// 施加的单位是一次发布（第六条）：前五条判出来的前缀里，一次发布的记录要一直走到带末条标志的那一条（[`record_ends_its_publish`]）
+/// 才整体施加；前缀停在一次发布中间（末条没到、断号、校验不过、回退行的 W 截在中间、下一条换了 txg、一次发布之内跳号、
+/// 一个事务的提交标记没出现）⇒ 那次发布整体不施加。
+///
+/// # Errors
+/// 所选根那次发布读得出的记录里带末条标志的多于一条（锚点认哪一条条款没写）⇒
+/// `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided`，一条记录都没施加。
 pub fn replay_journal(
     reader: &dyn PoolReader,
     root: &RootRecord,
@@ -1476,7 +1710,7 @@ pub fn replay_journal(
     records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
     verify_named_units: bool,
     rollback_high_water: Option<u64>,
-) -> (JournalScanReport, RootRecord) {
+) -> Result<(JournalScanReport, RootRecord), RecoveryFailure> {
     let mut report = JournalScanReport {
         valid_records: records.len(),
         above_water: 0,
@@ -1499,21 +1733,15 @@ pub fn replay_journal(
     report.above_water = above.len();
     let in_flight_limit =
         usize::try_from(journal_in_flight_record_limit(ring_bytes)).expect("在飞上限");
-    // 链首锚点 = 所选根覆盖的最后一条：同实例、checkpoint_txg 相等的记录里 jsn 最大的那条（已定项 14 注 1）。
-    // 取最小那条时，一次发布切成多条记录的那一版上链首落在那次发布自己的第二条（它不在水位之上），
+    // 链首锚点 = 所选根覆盖的最后一条：那次发布里带末条标志的那一条（已定项 14 注 1，读法乙）。
+    // 取那次发布 jsn 最小那条时，一次发布切成多条记录的那一版上链首落在那次发布自己的第二条（它不在水位之上），
     // 水位之上的第一条对不上号、一条都不施加（三方第一轮 K3：零故障少施加）。
-    // 那条读不出（两份都撕了）时不知道它的 jsn，链首只能是水位之上第一条可读记录，前提是它的 checkpoint_txg = 根的 txg + 1、
+    // 带标志的那条读不出（两份都撕了）时不知道它的 jsn，链首只能是水位之上第一条可读记录，前提是它的 checkpoint_txg = 根的 txg + 1、
     // 本次发布内序号为 1（已定项 14 注 1 / 已定项 4）。txg 更大 ⇒ 中间少了一次发布，断号即止（里程碑「第二个事务」步 3
     // 三方第一轮攻方腿打中：无锚点时无条件接上会跳过撕掉的一条）；序号不是 1 ⇒ 下一次发布的开头缺了，同样断号即止——
     // 只看 txg 时，下一次发布的第一条也读不出、第二条读得出，链首就接在第二条上，缺了第一条的那次发布照样整体施加
     // （三方第一轮 K4-b：多接）。
-    let root_own_record_counter = records
-        .values()
-        .filter(|record| {
-            record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg
-        })
-        .map(|record| record.counter)
-        .max();
+    let root_own_record_counter = counter_of_the_last_record_the_root_covers(root, records)?;
     let mut expected_next: Option<(InstanceGeneration, u64)> =
         root_own_record_counter.map(|counter| (root.instance, counter + 1));
     let chain_start_txg_without_anchor = CheckpointTxg(root.checkpoint_txg.0 + 1);
@@ -1536,17 +1764,33 @@ pub fn replay_journal(
                 break;
             }
         }
-        // 一次发布的记录共享一个 checkpoint_txg（D16（发布语义） 已定项 6）：末条还没到、下一条已经换了 txg
-        // ⇒ 这次发布缺了末条，断在这里，它整体不施加。
-        if let Some(first_record_of_the_open_publish) = records_of_the_open_publish.first() {
-            if record.checkpoint_txg != first_record_of_the_open_publish.checkpoint_txg {
+        if let Some(previous_record_of_the_open_publish) = records_of_the_open_publish.last() {
+            // 一次发布的记录共享一个 checkpoint_txg（D16（发布语义） 已定项 6）：末条还没到、下一条已经换了 txg
+            // ⇒ 这次发布缺了末条，断在这里，它整体不施加。
+            if record.checkpoint_txg != previous_record_of_the_open_publish.checkpoint_txg {
+                break;
+            }
+            // 一次发布的 N 条记录序号依次 1..N、与 jsn 同步（D23（journal 的角色与格式） 已定项 4）：一次发布之内跳号，
+            // 当这条记录损坏、断链即止——这次发布走不到末条，整体不施加。
+            if u64::from(record.ordinal_within_publish.0)
+                != u64::from(previous_record_of_the_open_publish.ordinal_within_publish.0) + 1
+            {
+                break;
+            }
+            // 提交标记（D23（journal 的角色与格式） 已定项 7）：一个事务可以跨多条记录，只有它的最后一条带提交标记
+            // （最后一个事务装不下一条记录时末条再跨记录，已定项 17）。上一条没带提交标记而这一条换了事务号 ⇒
+            // 那个事务的提交标记没出现，它被丢掉（已定项 7「丢掉提交标记还没出现的那个事务的全部记录」），
+            // 它所在的这次发布因此不完整、整体不施加（第六条）。
+            if !previous_record_of_the_open_publish.is_commit
+                && record.transaction != previous_record_of_the_open_publish.transaction
+            {
                 break;
             }
         }
         expected_next = Some((record.instance, record.counter + 1));
-        // 提交标记（D23（journal 的角色与格式） 已定项 7）：一条记录一个事务（C310（事务切分纪律与记录数口径打架）
-        // 2026-09-16 用户定案），写者给每条都带上它；不带的那条是没写完的事务，停在这里——它所在的那次发布因此也走不到末条、整体不施加。
-        if !record.is_commit {
+        // 末条不带提交标记 ⇒ 这次发布最后一个事务没提交，这次发布整体不施加。末条之外不带提交标记的，是一个跨多条记录的事务
+        // 还没写到它的最后一条，接着往下走（它换了事务号还没等到提交标记，上面那一判断链）。
+        if !record.is_commit && record_ends_its_publish(record) {
             break;
         }
         let all_verified = !verify_named_units
@@ -1600,7 +1844,7 @@ pub fn replay_journal(
             allocation_record_tree_root: rebuilt.allocation_record_tree_root,
         };
     }
-    (report, rebuilt)
+    Ok((report, rebuilt))
 }
 
 /// 每棵树的 key 宽（码 2 头里的自述 key 宽要与它相符）；day-1 只注册的三棵没有节点要解。
@@ -1853,7 +2097,7 @@ pub fn walk_to_file(
         });
     }
     // 实例表是一条链（D18（块里携带什么信息） 已定项 11）：第 0 片的链指针记录说还有下一片，就沿链读到最后一片，
-    // 任一片读不出、解不开，整张表就不可读，与第 0 片读不出同一个结局。只有一片的表（今天写者只写一片）不多读一次盘。
+    // 任一片读不出、解不开，整张表就不可读，与第 0 片读不出同一个结局。只有一片的表（行数不超过 369）不多读一次盘。
     let first_page = InstanceTablePage::parse(&instance_table_bytes, InstanceTablePageIndex::FIRST)
         .ok_or(RecoveryFailure::InvariantViolated {
             invariant: "E142 走读同款",
@@ -2120,25 +2364,15 @@ pub fn walk_to_file(
                 detail: "文件第 i 条 extent 记录的 key 不是 (0, inode, i × 净荷容量)：offset 段不是文件字节偏移，或记录有洞、错位",
             });
         }
-        let bytes = match read_unit_via_locations(reader, &pointer.locations, data_unit_bytes) {
-            Ok(bytes) => bytes,
-            Err(_hint_error) => {
-                *mapping_fallbacks += 1;
-                let mapping_key = mapping_key_for_data(pointer.head, pointer.write_order);
-                let Some(locations) =
-                    central_mapping_locations_among_entries(&roots.mapping.entries, &mapping_key)?
-                else {
-                    return Err(RecoveryFailure::MappingMiss {
-                        slot: pointer.locations[0].slot,
-                    });
-                };
-                read_unit_via_locations(reader, &locations, data_unit_bytes).map_err(|_error| {
-                    RecoveryFailure::MappingStillUnreadable {
-                        slot: locations[0].slot,
-                    }
-                })?
-            }
-        };
+        let bytes = read_data_unit_via_hint_then_central_mapping(
+            reader,
+            pointer,
+            &|mapping_key: &[u8]| {
+                central_mapping_locations_among_entries(&roots.mapping.entries, mapping_key)
+            },
+            mapping_fallbacks,
+        )?
+        .into_content()?;
         let header = parse_data_unit(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
             what: "数据单元",
         })?;
@@ -2218,7 +2452,7 @@ pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport
         };
     };
     let root_key = (root.instance, root.checkpoint_txg);
-    let (journal, effective_root) = match policy {
+    let replayed = match policy {
         JournalPolicy::Consult | JournalPolicy::ConsultWithoutNamedVerification => {
             let records = scan_journal(reader, &system_configuration);
             replay_journal(
@@ -2230,7 +2464,22 @@ pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport
                 rollback_high_water_of_root(reader, &root),
             )
         }
-        JournalPolicy::Ignore => (JournalScanReport::default(), root),
+        JournalPolicy::Ignore => Ok((JournalScanReport::default(), root)),
+    };
+    // 恢复在施加任何一条记录之前停下（锚点认哪一条条款没写）：没有「实际走的那条根」可报。
+    let (journal, effective_root) = match replayed {
+        Ok(replayed) => replayed,
+        Err(failure) => {
+            return RecoveryReport {
+                outcome: RecoveryOutcome::Failed {
+                    root: Some(root_key),
+                    failure,
+                },
+                effective_root: None,
+                journal: JournalScanReport::default(),
+                mapping_fallbacks,
+            }
+        }
     };
     let outcome = match walk_to_file(reader, &effective_root, &mut mapping_fallbacks) {
         Ok(Some(content)) => RecoveryOutcome::FileRead {
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 9f54bdc..14ec3f4 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -34,8 +34,13 @@ use crate::inode_tree::{
     write_records_into_leaf_containers, InodeLeafContainer, InodeLeafContainerIndexInTree,
     InodeLeafContainersAfterThisPublish, InodeTreeWriteRefusal,
 };
+use crate::instance_table::{
+    instance_table_page_records, instance_table_pages_for_rows, instance_table_rows_of_each_page,
+    InstanceRow, InstanceTableChainRecord, InstanceTablePageIndex,
+};
 use crate::journal::{
-    back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish, NamedUnit,
+    back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish,
+    JournalRecordPlaceInPublish, NamedUnit,
 };
 use crate::make_filesystem::{
     location_entries, MakeFilesystemParameters, MKFS_INSTANCE_GENERATION, TREE_TABLE_KEY_WIDTH,
@@ -68,8 +73,7 @@ use crate::system_configuration::{
 use crate::unit::{
     build_data_unit, build_index_node, build_packed_unit, data_unit_payload_capacity,
     index_node_entry_capacity, parse_index_node, unit_filesystem_identifier, DataUnitIdentity,
-    PackedIdentity, WriteOrder, PACKED_TYPE_INSTANCE_TABLE, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE,
-    UNIT_CLASS_PACKED,
+    WriteOrder, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
 };
 use crate::write_accounting::{WritesByStructureKind, WrittenStructureKind};
 use crate::write_request_split::{
@@ -609,6 +613,9 @@ pub fn publish_without_units<Device: BlockDevice>(
         is_commit: true,
         // 空发布记录也写 1（D23（journal 的角色与格式） 已定项 4）：它一条就是一次发布。
         ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,
+        // 它一条就是一次发布，也就是这次发布的末条（D23（journal 的角色与格式） 已定项 17：每次只有一条记录的发布，
+        // 含空发布记录，那一条记录标志位 0 也写 1）。
+        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
         back_chain: plan.back_chain,
         filesystem_identifier: unit_filesystem_identifier(&pool.parameters.filesystem_identifier),
         new_tree_table: previous_root.tree_table,
@@ -663,7 +670,7 @@ pub fn publish_without_units<Device: BlockDevice>(
 }
 
 /// 写行那次发布的参数，树表 0 条的一版上（D18（块里携带什么信息） 已定项 11「每次可写挂载都写行」）：身份字段同零单元发布，
-/// 另带这次要写出去的整片实例表记录（行在前、链指针记录最末，由调用方在上一版那张表后面接上这次的行拼好）。
+/// 另带这次整条实例表链怎么重写（这次之后整张表的行——调用方在上一版那张表后面接上这次的行——与被换下的那条旧链）。
 #[derive(Clone, Copy, Debug)]
 pub struct InstanceTableOnlyPublishPlan<'records> {
     pub txg: CheckpointTxg,
@@ -673,14 +680,14 @@ pub struct InstanceTableOnlyPublishPlan<'records> {
     /// 反向链：本实例的第一条恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
     pub back_chain: u32,
     pub rollback_floor: CheckpointTxg,
-    pub instance_table_records: &'records [Vec<u8>],
+    pub instance_table: &'records InstanceTableRewrite,
     /// 这次写进根记录与记录新根段的树 ID 水位，同 `ZeroUnitPublishPlan::tree_identifier_watermark`：写行那次发布一个树 ID 都不发，
     /// 就是根环里全部根记录的 max（D8（核心索引结构） 已定项 8 ②）——回退到树表 0 条的一版时它高于那一版自己带的。
     pub tree_identifier_watermark: u64,
 }
 
-/// 写行那次发布，上一版树表 0 条：**重写实例表与分配记录树两个单元**，落盘顺序同别的发布（D16（发布语义） 已定项 7）——
-/// 两个单元写 → 屏障 → journal 记录（点名这两个单元）→ 屏障 → 根槽 FUA → 系统配置槽轮换。
+/// 写行那次发布，上一版树表 0 条：**重写实例表链与分配记录树**（链有几片就几个实例表单元，加一个分配记录树节点），
+/// 落盘顺序同别的发布（D16（发布语义） 已定项 7）——单元写 → 屏障 → journal 记录（点名这几个单元）→ 屏障 → 根槽 FUA → 系统配置槽轮换。
 ///
 /// 为什么是这两个单元：树表 0 条 ⇒ 这一版没有记账树，D16（发布语义） 已定项 9 那五样（记账行、记账树节点、映射条目、
 /// 树表单元、树表条目）一样都不写；而 D18（块里携带什么信息） 已定项 11 要求每次可写挂载都写行、写行 COW 重写整条链 ⇒ 实例表自己那一个单元
@@ -688,28 +695,44 @@ pub struct InstanceTableOnlyPublishPlan<'records> {
 /// 那一片就成了空闲槽，而根环里 txg 更低的候选根还指着它（2026-09-23 用户定案随 C512（树表 0 条的一版上被换下的单元记在哪））。
 /// 它的根指针住**根记录**新加的那一项，不进树表——进树表 `tree_table_has_no_entries` 当场翻面，
 /// 按 `PreviousVersion::WithoutFile` / `WithFile` 分流的每一处跟着变。
-/// 落点照 D3（空间分配） 已定项 5 从聚簇段 bump、按已定项 10 ⑤ 各自那一档取，与两个角色在别的发布路径上走的是同一条规则。
+/// 落点照 D3（空间分配） 已定项 5 从聚簇段 bump、按已定项 10 ⑤ 各自那一档取（实例表各片最前、尾片先，分配记录树节点在后），
+/// 与这几个角色在别的发布路径上走的是同一条规则。
 ///
-/// 被换下的那两片（上一版的实例表、上一版的分配记录树节点）在同一次发布里释放（释放先于分配，D3（空间分配） 已定项 7）：
+/// 被换下的（上一版的整条实例表链、上一版的分配记录树节点）在同一次发布里释放（释放先于分配，D3（空间分配） 已定项 7）：
 /// 两者都豁免映射（实例表见 D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12；分配记录树的根这一版住根记录），
-/// 落点从上一版根记录里那两条指针取，三样逐盘核过（`placement_to_release_after_checking_every_device`）。
-/// 上一版是 mkfs 的第 0 代（分配记录树根指针全零）时只释放实例表那一片。
+/// 实例表链的各片取计划带着的那条旧链（`instance_table_chain_to_release`），分配记录树节点取上一版根记录里那一条指针，
+/// 三样逐盘核过（`placement_to_release_after_checking_every_device`）。上一版是 mkfs 的第 0 代（分配记录树根指针全零）时只释放实例表。
 ///
 /// 新根照上一版的根，只换 checkpoint_txg、实例代号、回退下界、实例表指针、分配记录树根指针与树 ID 水位（取计划里给的，
 /// D8（核心索引结构） 已定项 8 ②）：树表、映射树根照抄（这一版的树表仍是 mkfs 那片 0 条的）。
 ///
 /// # Errors
-/// 被换下的那两片任一片的三样核不过（`ReleaseTarget*` / `ReleaseSpanMismatch`）、这次之后的分配记录装不进一个节点
+/// 被换下的任一片的三样核不过（`ReleaseTarget*` / `ReleaseSpanMismatch`）、这次之后的分配记录装不进一个节点
 /// （`AllocationRecordsExceedOneNode`）、落点取不到（`PlacementRefused`）、块设备报错。
 /// 前三样在任何写之前返回，盘上逐字节不变；块设备错交回时分配器回到发布之前的样子。
+///
+/// # Panics
+/// 计划带着的旧链第 0 片不是上一版根记录指着的那一片：调用方拼行与拼旧链读的是同一条根。
 pub fn publish_instance_table_on_version_without_file<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
     previous_root: &RootRecord,
     plan: InstanceTableOnlyPublishPlan<'_>,
 ) -> Result<VersionWithoutFilePublishOutput, PublishError> {
-    let swapped_out = placements_to_release_on_a_version_without_file(previous_root, allocator)?;
-    version_without_file_row_publish_admission(allocator)?;
+    assert_eq!(
+        plan.instance_table.replaced_chain.first(),
+        Some(&previous_root.instance_table),
+        "计划带着的旧链是上一版根记录指着的那一条：调用方拼行与拼旧链读的是同一条根"
+    );
+    let swapped_out = placements_to_release_on_a_version_without_file(
+        previous_root,
+        &plan.instance_table.replaced_chain,
+        allocator,
+    )?;
+    version_without_file_row_publish_admission(
+        allocator,
+        plan.instance_table.pages_after_this_publish(),
+    )?;
     // 失败就换回去：取落点会动分配器（bump 指针、位图、记录），中途报错不留半新的池。
     let allocator_before_this_publish = allocator.clone();
     let published = publish_instance_table_after_the_release_check(
@@ -725,27 +748,26 @@ pub fn publish_instance_table_on_version_without_file<Device: BlockDevice>(
     published
 }
 
-/// 写行那次发布换下的两片：上一版的实例表，与上一版那片分配记录树节点
+/// 写行那次发布换下的：上一版的整条实例表链，与上一版那片分配记录树节点
 /// （上一版是 mkfs 的第 0 代时根记录那一项全零、没有这一片）。
 struct PlacementsReleasedByTheRowPublish {
-    instance_table: Placement,
+    /// 旧链每一片的落点，按片序号。
+    instance_table_chain: Vec<Placement>,
     allocation_record_node: Option<Placement>,
 }
 
-/// 写行那次发布换下的落点：上一版的实例表，加上一版的分配记录树节点（上一版是 mkfs 的第 0 代时那一条指针全零、没有这一片）。
+/// 写行那次发布换下的落点：上一版的整条实例表链（`replaced_chain`，计划带着的），加上一版的分配记录树节点
+/// （上一版是 mkfs 的第 0 代时那一条指针全零、没有这一片）。
 /// 只查不改（D19（块指针的结构与宽度预算） 已定项 5 第 1 条），查不齐就一个落点都不释放。
 ///
 /// # Errors
-/// 两条指针任一条的三样（在册、没释放过、跨度对得上）核不过 ⇒ `ReleaseTarget*` / `ReleaseSpanMismatch`。
+/// 任一条指针的三样（在册、没释放过、跨度对得上）核不过 ⇒ `ReleaseTarget*` / `ReleaseSpanMismatch`。
 fn placements_to_release_on_a_version_without_file(
     previous_root: &RootRecord,
+    replaced_chain: &[NodePointer],
     allocator: &PoolAllocator,
 ) -> Result<PlacementsReleasedByTheRowPublish, PublishError> {
-    let instance_table = placement_to_release_after_checking_every_device(
-        TransactionUnit::InstanceTable,
-        &previous_root.instance_table.locations,
-        allocator,
-    )?;
+    let instance_table_chain = instance_table_chain_to_release(replaced_chain, allocator)?;
     let allocation_record_node =
         if previous_root.allocation_record_tree_root == NodePointer::empty_root() {
             None
@@ -757,7 +779,7 @@ fn placements_to_release_on_a_version_without_file(
             )?)
         };
     Ok(PlacementsReleasedByTheRowPublish {
-        instance_table,
+        instance_table_chain,
         allocation_record_node,
     })
 }
@@ -765,19 +787,17 @@ fn placements_to_release_on_a_version_without_file(
 /// 写行那次发布的准入：这次之后的分配记录条数装不进一个节点就在动分配器之前拒掉，不许走到 `build_index_node` 的断言。
 /// 与 `admission_of_one_publish` 的第一条共用 `refuse_when_the_allocation_records_do_not_fit_one_node`；
 /// 记账与映射那两条在这一格没有对象（树表 0 条 ⇒ 这一版没有记账树、没有映射条目），所以不调那一整道。
-/// 释放只改写记录、不加条数，所以基数是分配器此刻的记录数；这次新增的是重写的那两个角色每盘各一条。
+/// 释放只改写记录、不加条数，所以基数是分配器此刻的记录数；这次新增的是重写的那几个角色（实例表链每一片、分配记录树节点）每盘各一条。
 ///
 /// # Errors
 /// `AllocationRecordsExceedOneNode`。
 fn version_without_file_row_publish_admission(
     allocator: &PoolAllocator,
+    instance_table_pages_after_this_publish: usize,
 ) -> Result<(), PublishError> {
-    let rewritten = [
-        TransactionUnit::InstanceTable,
-        TransactionUnit::AllocationTree,
-    ];
+    let rewritten_roles = instance_table_pages_after_this_publish + 1;
     refuse_when_the_allocation_records_do_not_fit_one_node(
-        allocator.records().len() + rewritten.len() * allocator.devices.len(),
+        allocator.records().len() + rewritten_roles * allocator.devices.len(),
     )
 }
 
@@ -850,6 +870,78 @@ fn build_allocation_record_node(
     )
 }
 
+/// 这次重写出去的实例表链上的一片：角色、单元字节、指着它的指针（第 0 片的进根记录，第 k 片的进第 k − 1 片的链指针记录）。
+struct InstanceTablePageUnit {
+    role: TransactionUnit,
+    bytes: Vec<u8>,
+    pointer: NodePointer,
+}
+
+/// 装一条实例表链（D18（块里携带什么信息） 已定项 11）：行按链上的次序一片写满 369 行再开下一片、最后一片装剩下的
+/// （`instance_table_rows_of_each_page`，用户 2026-09-24 定案），每片末尾一条链指针记录——最后一片写「无下一片」，别的片写下一片的指针。
+/// **尾片先装、先发出生序号**（D3（空间分配） 已定项 10 ⑤，用户 2026-09-24 定案：第 k 片当第 k + 1 片的父，照先叶后根——第 k 片的
+/// 链指针记录要第 k + 1 片的落点、整单元校验和与出生序号，第 k + 1 片装好了它才装得出来）。落点由调用方先按同一个次序取好
+/// （`instance_table_page_roles_in_bump_order`），这里按角色查。交回的各片按链上的次序，第 0 片在前。
+/// 带文件的一版上写行（`publish_admitted`）与树表 0 条的一版上写行（`publish_instance_table_after_the_release_check`）共用这一处：
+/// 两处各装一份，片的切法与发号次序会分叉。
+///
+/// # Panics
+/// 某一片的角色没取落点（`slot_of` 里的断言）：调用方按同一张行表的片数取的落点。
+fn build_instance_table_chain(
+    rows: &[InstanceRow],
+    slot_of: &dyn Fn(TransactionUnit) -> SlotNumber,
+    txg: CheckpointTxg,
+    write_order: WriteOrder,
+    filesystem_identifier: &[u8; 16],
+    device_identities: &[DeviceIdentity],
+    sequences: &mut BirthSequenceAllocator,
+) -> Vec<InstanceTablePageUnit> {
+    let rows_of_each_page = instance_table_rows_of_each_page(rows);
+    let mut pages_tail_first: Vec<InstanceTablePageUnit> =
+        Vec::with_capacity(rows_of_each_page.len());
+    // 迭代次数的上界是片数；跨轮带的只有后一片的指针（前一片的链指针记录要它），没有提前出口。
+    let mut pointer_to_the_next_page: Option<NodePointer> = None;
+    for (position, page_rows) in rows_of_each_page.iter().enumerate().rev() {
+        let page = InstanceTablePageIndex(u64::try_from(position).expect("片序号"));
+        let role = TransactionUnit::of_instance_table_page(page);
+        let chain = match pointer_to_the_next_page {
+            None => InstanceTableChainRecord::LastPage,
+            Some(next_page) => InstanceTableChainRecord::NextPage(next_page),
+        };
+        let birth_sequence = sequences.next(
+            TreeIdentifier(TREE_IDENTIFIER_NONE),
+            txg,
+            write_order.instance,
+        );
+        let bytes = build_packed_unit(
+            page.packed_identity(),
+            u16::try_from(INSTANCE_ROW_BYTES).expect("88"),
+            &instance_table_page_records(page_rows, &chain),
+            txg,
+            filesystem_identifier,
+            write_order,
+            birth_sequence,
+        );
+        let pointer = NodePointer {
+            head: PointerHead {
+                birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
+                birth_txg: txg,
+            },
+            locations: location_entries(device_identities, slot_of(role), &bytes),
+            instance: write_order.instance,
+            birth_sequence,
+        };
+        pointer_to_the_next_page = Some(pointer);
+        pages_tail_first.push(InstanceTablePageUnit {
+            role,
+            bytes,
+            pointer,
+        });
+    }
+    pages_tail_first.reverse();
+    pages_tail_first
+}
+
 /// `publish_instance_table_on_version_without_file` 的后半段：释放、取落点、装单元、落盘。分出来只为把「失败就把分配器换回去」
 /// 收在一处——中间每一步都可能提前返回，散在调用点上就会漏掉某一条路径。
 fn publish_instance_table_after_the_release_check<Device: BlockDevice>(
@@ -859,7 +951,6 @@ fn publish_instance_table_after_the_release_check<Device: BlockDevice>(
     plan: InstanceTableOnlyPublishPlan<'_>,
     swapped_out: PlacementsReleasedByTheRowPublish,
 ) -> Result<VersionWithoutFilePublishOutput, PublishError> {
-    let identity = TransactionUnit::InstanceTable;
     let txg = plan.txg;
     let instance = plan.instance;
     let filesystem_identifier = &pool.parameters.filesystem_identifier;
@@ -868,13 +959,21 @@ fn publish_instance_table_after_the_release_check<Device: BlockDevice>(
         // 写行那次发布不承载事务（D23（journal 的角色与格式） 已定项 19 ①：空发布与写行的记录写事务号 0）。
         transaction: 0,
     };
-    let released = swapped_out.instance_table;
-    allocator.release(released, txg);
+    for released in swapped_out.instance_table_chain.iter().copied() {
+        allocator.release(released, txg);
+    }
     if let Some(previous_allocation_record_node) = swapped_out.allocation_record_node {
         allocator.release(previous_allocation_record_node, txg);
     }
-    let placement = allocate_placement_for_role(allocator, identity, txg)?;
-    // 分配记录树那一片的落点在实例表之后取：这一片自己的分配记录也要进它自己那个节点，所以两个落点都取完才装节点。
+    // 实例表链的各片最前、尾片先（D3（空间分配） 已定项 10 ⑤），分配记录树那一片在它们之后取：
+    // 这一片自己的分配记录也要进它自己那个节点，所以落点都取完才装节点。
+    let instance_table_roles =
+        instance_table_page_roles_in_bump_order(plan.instance_table.pages_after_this_publish());
+    let mut instance_table_slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
+    for identity in &instance_table_roles {
+        let placement = allocate_placement_for_role(allocator, *identity, txg)?;
+        instance_table_slots.insert(*identity, placement.slot);
+    }
     let allocation_placement =
         allocate_placement_for_role(allocator, TransactionUnit::AllocationTree, txg)?;
     // 这一版的分配记录树节点换成了刚取的这一片：下一次发布（再写一次行，或在这一版上发第一个文件版本）要换下它，
@@ -883,33 +982,24 @@ fn publish_instance_table_after_the_release_check<Device: BlockDevice>(
     allocator.note_allocation_record_node_of_the_version_without_file(allocation_placement);
     // 两个落点取完、这次不再分配：记下这条根盖掉的根环槽（同 `publish_admitted` 那一处；失败时分配器由调用方整个换回去）。
     allocator.record_root_written_by_this_process(txg);
-    // 这一次发布有实例表与分配记录树两个提交内生块：实例表归树 0，分配记录树归树 13，各自在 (txg, 实例) 上发自己的出生序号。
+    // 这一次发布的提交内生块（实例表链的各片、分配记录树那一片）都归树 0（这一版还没登记过任何树），在 (txg, 实例) 上连着发出生序号：
+    // 实例表各片按 bump 次序（尾片先）在前，分配记录树那一片最后。
     let mut sequences = BirthSequenceAllocator::default();
-    let birth_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
-    let unit = build_packed_unit(
-        PackedIdentity {
-            birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
-            record_type: PACKED_TYPE_INSTANCE_TABLE,
-            container: 0,
-            container_birth: CheckpointTxg(0),
-        },
-        u16::try_from(INSTANCE_ROW_BYTES).expect("88"),
-        plan.instance_table_records,
+    let device_identities: Vec<DeviceIdentity> =
+        pool.devices.iter().map(|(identity, _)| *identity).collect();
+    let instance_table_pages = build_instance_table_chain(
+        &plan.instance_table.rows,
+        &|identity| instance_table_slots[&identity],
         txg,
-        filesystem_identifier,
         write_order,
-        birth_sequence,
+        filesystem_identifier,
+        &device_identities,
+        &mut sequences,
     );
-    let locations = pool.location_entries(placement.slot, &unit);
-    let instance_table_pointer = NodePointer {
-        head: PointerHead {
-            birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
-            birth_txg: txg,
-        },
-        locations,
-        instance,
-        birth_sequence,
-    };
+    let instance_table_pointer = instance_table_pages
+        .first()
+        .expect("一条链至少一片：0 行也写出空的那一片")
+        .pointer;
     let allocation_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
     let allocation_unit = build_allocation_record_node(
         allocator,
@@ -937,31 +1027,40 @@ fn publish_instance_table_after_the_release_check<Device: BlockDevice>(
         is_commit: true,
         // 这次发布只有这一条记录（D23（journal 的角色与格式） 已定项 4：只有一条时是 1）。
         ordinal_within_publish: JournalRecordOrdinalWithinPublish::FIRST,
+        // 只有这一条 ⇒ 它就是这次发布的末条（D23（journal 的角色与格式） 已定项 17）。
+        place_in_publish: JournalRecordPlaceInPublish::LastRecordOfThePublish,
         back_chain: plan.back_chain,
         filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
         new_tree_table: previous_root.tree_table,
         new_mapping_root: previous_root.mapping_root,
         new_tree_identifier_watermark: plan.tree_identifier_watermark,
         new_rollback_floor: plan.rollback_floor,
-        // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的角色各一项——实例表与分配记录树。
-        named: vec![
-            NamedUnit {
-                locations,
-                unit_class: identity.unit_class(),
-                // 实例表单元不属于任何一棵树（树 0）；这一版还没发过树 ID，不按 `TransactionUnit::tree` 取。
-                birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
-                birth_txg: txg,
-                key_tail: node_key_tail(instance, birth_sequence),
-            },
-            NamedUnit {
+        // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的角色各一项，按 bump 次序——实例表链的各片（尾片先），分配记录树。
+        named: instance_table_roles
+            .iter()
+            .map(|identity| {
+                let page = instance_table_pages
+                    .iter()
+                    .find(|page| page.role == *identity)
+                    .expect("每个取了落点的实例表角色都装了一片");
+                NamedUnit {
+                    locations: page.pointer.locations,
+                    unit_class: identity.unit_class(),
+                    // 实例表单元不属于任何一棵树（树 0）；这一版还没发过树 ID，不按 `TransactionUnit::tree` 取。
+                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
+                    birth_txg: txg,
+                    key_tail: node_key_tail(instance, page.pointer.birth_sequence),
+                }
+            })
+            .chain([NamedUnit {
                 locations: allocation_locations,
                 unit_class: TransactionUnit::AllocationTree.unit_class(),
                 // 树 ID 0：这一版还没登记过任何树，那一片由根记录独占持有（见 `build_allocation_record_node` 的文档注释）。
                 birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
                 birth_txg: txg,
                 key_tail: node_key_tail(instance, allocation_sequence),
-            },
-        ],
+            }])
+            .collect(),
     };
     let record_bytes = record.to_bytes();
     let root = RootRecord {
@@ -979,11 +1078,18 @@ fn publish_instance_table_after_the_release_check<Device: BlockDevice>(
     let writes_before_this_row_publish = pool.writes_by_structure_kind.clone();
     // 中途失败时这次已记的写要交出去（增补 2 第 20b 行）：六步收在一个闭包里，失败在这里记账再把错原样交回。
     let persist = |writer: &mut PoolWriter<'_, Device>| -> Result<(), BlockDeviceError> {
-        writer.perform(CommitStep::WriteUnitToEveryDevice {
-            slot: placement.slot,
-            unit: &unit,
-            identity,
-        })?;
+        // 单元写按 bump 次序：实例表链的各片（尾片先），分配记录树那一片最后。
+        for identity in &instance_table_roles {
+            let page = instance_table_pages
+                .iter()
+                .find(|page| page.role == *identity)
+                .expect("每个取了落点的实例表角色都装了一片");
+            writer.perform(CommitStep::WriteUnitToEveryDevice {
+                slot: instance_table_slots[identity],
+                unit: &page.bytes,
+                identity: *identity,
+            })?;
+        }
         writer.perform(CommitStep::WriteUnitToEveryDevice {
             slot: allocation_placement.slot,
             unit: &allocation_unit,
@@ -1108,8 +1214,14 @@ pub enum TransactionUnit {
     MappingTree,
     TreeTable,
     /// 实例表单元（码 3 打包记录类型 4）：mkfs 种下第一片，之后每次可写挂载写行时重写（D18（块里携带什么信息） 已定项 11）；
-    /// 不在第一个事务的八个角色里，第二个事务起才进发布路径。
+    /// 不在第一个事务的八个角色里，第二个事务起才进发布路径。它是链上第 0 片——根记录直接持有的那一片；
+    /// 多于一片时第 1 片起是 [`TransactionUnit::InstanceTablePageAfterTheFirst`]。
     InstanceTable,
+    /// 实例表链上第 1 片起的一片（码 3 打包记录类型 4，身份 (0, 4, 片序号, 0)，D18（块里携带什么信息） 已定项 11）：它的指针不在根记录里，
+    /// 在上一片末尾那条链指针记录里。带的是片序号，恒 ≥ 1（第 0 片是 [`TransactionUnit::InstanceTable`]，
+    /// 两者由 [`TransactionUnit::of_instance_table_page`] 按片序号分）。表的行多于一片装得下的 369 行时写行那次发布才写出它们，
+    /// 在 bump 次序里尾片先（D3（空间分配） 已定项 10 ⑤，用户 2026-09-24 定案）。
+    InstanceTablePageAfterTheFirst(InstanceTablePageIndex),
 }
 
 impl TransactionUnit {
@@ -1153,6 +1265,36 @@ impl TransactionUnit {
             TransactionUnit::MappingTree => "t7".to_string(),
             TransactionUnit::TreeTable => "t8".to_string(),
             TransactionUnit::InstanceTable => "ti".to_string(),
+            TransactionUnit::InstanceTablePageAfterTheFirst(page) => format!("ti+{}", page.0),
+        }
+    }
+
+    /// 实例表链上第 `page` 片的角色：第 0 片是根记录持有的 [`TransactionUnit::InstanceTable`]，第 1 片起是
+    /// [`TransactionUnit::InstanceTablePageAfterTheFirst`]。
+    #[must_use]
+    pub fn of_instance_table_page(page: InstanceTablePageIndex) -> Self {
+        if page == InstanceTablePageIndex::FIRST {
+            TransactionUnit::InstanceTable
+        } else {
+            TransactionUnit::InstanceTablePageAfterTheFirst(page)
+        }
+    }
+
+    /// 这个角色是不是实例表链上的一片（第 0 片或第 1 片起）。
+    #[must_use]
+    pub const fn is_a_page_of_the_instance_table(self) -> bool {
+        match self {
+            TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
+                true
+            }
+            TransactionUnit::Data(_)
+            | TransactionUnit::ExtentRoot
+            | TransactionUnit::InodeLeafContainer(_)
+            | TransactionUnit::InodeRoot
+            | TransactionUnit::AllocationTree
+            | TransactionUnit::AccountingTree
+            | TransactionUnit::MappingTree
+            | TransactionUnit::TreeTable => false,
         }
     }
 
@@ -1160,9 +1302,9 @@ impl TransactionUnit {
     pub const fn unit_class(self) -> u8 {
         match self {
             TransactionUnit::Data(_) => UNIT_CLASS_DATA,
-            TransactionUnit::InodeLeafContainer(_) | TransactionUnit::InstanceTable => {
-                UNIT_CLASS_PACKED
-            }
+            TransactionUnit::InodeLeafContainer(_)
+            | TransactionUnit::InstanceTable
+            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => UNIT_CLASS_PACKED,
             TransactionUnit::ExtentRoot
             | TransactionUnit::InodeRoot
             | TransactionUnit::AllocationTree
@@ -1182,7 +1324,9 @@ impl TransactionUnit {
             TransactionUnit::AllocationTree => trees.allocation_records,
             TransactionUnit::AccountingTree => trees.accounting,
             TransactionUnit::MappingTree => trees.central_mapping,
-            TransactionUnit::TreeTable | TransactionUnit::InstanceTable => {
+            TransactionUnit::TreeTable
+            | TransactionUnit::InstanceTable
+            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                 TreeIdentifier(TREE_IDENTIFIER_NONE)
             }
         }
@@ -1193,7 +1337,9 @@ impl TransactionUnit {
     pub const fn placement(self) -> PlacementRule {
         match self {
             TransactionUnit::Data(_) => PlacementRule::UserData,
-            TransactionUnit::InodeLeafContainer(_) | TransactionUnit::InstanceTable => {
+            TransactionUnit::InodeLeafContainer(_)
+            | TransactionUnit::InstanceTable
+            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
                 PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned)
             }
             TransactionUnit::ExtentRoot
@@ -1335,15 +1481,16 @@ pub struct WrittenJournalRecord {
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct TransactionOutput {
     pub root: RootRecord,
-    /// 这次发布的末条记录：点名了这次发布共享的提交内生块的那一条（D23（journal 的角色与格式） 已定项 17），
+    /// 这次发布的末条记录：带「本次发布末条」标志的那一条（D23（journal 的角色与格式） 已定项 17），
     /// 也是所选根覆盖到的最后一条（已定项 14 注 1）——下一次发布的 jsn 与反向链接在它后面。只有一条记录的发布就是那一条。
     pub record: JournalRecord,
     pub record_bytes: Vec<u8>,
-    /// 这次发布末条之前的那几条，按 jsn 升序：一次发布切成 N 条记录时的前 N − 1 条，各只点名自己那个数据单元
-    /// （D23（journal 的角色与格式） 已定项 17）。只有一条记录的发布、从盘上重建出来的版本都为空。
+    /// 这次发布末条之前的那几条，按 jsn 升序：一次发布切成 N 个事务时前 N − 1 个事务各一条、各只点名自己那个数据单元，
+    /// 最后一个事务装不下一条记录时再跨出去的那几条（末条除外）也在这里（D23（journal 的角色与格式） 已定项 17）。
+    /// 只有一条记录的发布、从盘上重建出来的版本都为空。
     pub earlier_records_of_this_publish: Vec<WrittenJournalRecord>,
     /// 这一版全部角色的单元：这次重写的是新装的，没重写的从上一版照抄（文件 / 固定点角色按 bump 次序——每个数据单元、extent 根、
-    /// 每片 inode 叶容器、inode 根、四个固定点单元——实例表单元在末尾、mkfs 之后第一次重写之前不在）。
+    /// 每片 inode 叶容器、inode 根、四个固定点单元——实例表链的各片在末尾（按链上的次序，第 0 片在前）、mkfs 之后第一次重写之前不在）。
     pub units: Vec<PublishedUnit>,
     /// 这次发布真正写出的角色，按写出的次序（点名项与录制流里的单元写只有这些）。
     pub rewritten: Vec<TransactionUnit>,
@@ -1364,10 +1511,12 @@ pub struct TransactionOutput {
     /// 进映射的单元各自的映射 key（每个数据单元、extent 根、每片 inode 叶容器、inode 根、分配记录树、记账树；映射树与树表豁免）；
     /// 下一次覆盖写按它经映射取落点释放（D19（块指针的结构与宽度预算） 已定项 5 第 1 条）。
     pub mapped_units: Vec<(TransactionUnit, Vec<u8>)>,
-    /// 这次发布释放的落点（覆盖写换下的上一版八个单元；第一个事务为空）。
+    /// 这次发布释放的落点（覆盖写换下的上一版八个单元；第一个事务为空）：逻辑上释放了的都在，
+    /// 其中有一份核出对不上、那块盘上的记录留在已分配的也在（那几份另列在下一个字段）。
     pub released: Vec<Placement>,
-    /// 这次发布释放的落点里，释放之前读盘核校验和核出对不上、随即隔离的那几份（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1
-    /// 「记进隔离并计数报出」的这一次那一份）；从盘上重建的版本不是这个进程发布的，是空的。
+    /// 这次发布释放的落点里，释放之前读盘核校验和核出对不上（读不出也算）、隔离了的那几份（D19（块指针的结构与宽度预算） 已定项 5
+    /// 硬规则 1「记进隔离并计数报出」的这一次那一份）：隔离 = 那一份在它那块盘上的分配记录留在已分配（用户 2026-09-24 定案），
+    /// 落盘即跨重挂。从盘上重建的版本不是这个进程发布的，是空的。
     pub quarantined_after_release_checksum_mismatch:
         Vec<CopyQuarantinedAfterReleaseChecksumMismatch>,
     /// C319（请求内单元按 key 升序发出没有条款也没有检查）的运行时计数：同一请求内取号序与 key 序不一致的次数，第一个事务恒 0。
@@ -1379,7 +1528,8 @@ pub struct TransactionOutput {
     pub highest_transaction_number_in_this_instance: u64,
 }
 
-/// 释放之前按位置项读盘核校验和、核出对不上而隔离的一份：哪个角色、哪块盘、那一份的落点（槽取位置项里的，跨度取角色的）。
+/// 释放之前按位置项读盘核校验和、核出对不上（读不出也算）而隔离的一份：哪个角色、哪块盘、那一份的落点（槽取位置项里的，跨度取角色的）。
+/// 隔离就是那块盘上这个落点的分配记录留在已分配。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct CopyQuarantinedAfterReleaseChecksumMismatch {
     pub unit: TransactionUnit,
@@ -1501,6 +1651,8 @@ fn format_time_tree_table_to_release(
 /// 这次重写文件内容时，上一版比这一版多出来的那几个数据单元一并释放（文件变短），这一版新多出来的数据单元与叶容器在上一版里没有落点、跳过。
 /// 映射树与树表豁免映射，各从根记录里指着它们的那条指针取
 /// （D19（块指针的结构与宽度预算） 已定项 11：映射树的根住根记录）。两盘同槽（D2（RAID 条带策略） 已定项 10）。
+/// 实例表链不在这里释放：整条旧链由 [`instance_table_chain_to_release`] 按计划带着的那条链逐片释放（第 1 片起的指针只在
+/// 上一片的链指针记录里，不在上一版的内存态里）。
 /// 映射只给槽号；跨度取分配记录里的，并与单元种类该有的跨度互核——查出来的槽在册、没释放过、跨度对得上，三样有一样不对就报错、
 /// 一个落点都不释放（第二轮攻方腿：此前查得到 key 就直接交给 `release`，落点指错时在断言上 panic）。
 /// **那三样对池里每块盘各核一遍**：`PoolAllocator::release` 对每块盘都要求一条对得上的记录，而两块盘的分配记录树
@@ -1518,9 +1670,6 @@ pub fn placements_to_release_via_mapping(
 ) -> Result<Vec<Placement>, PublishError> {
     let mut placements = Vec::new();
     for identity in roles_replaced_via_mapping(previous, roles) {
-        if !has_a_placement_in_the_previous_version(previous, identity) {
-            continue;
-        }
         let locations = match identity {
             TransactionUnit::Data(_)
             | TransactionUnit::ExtentRoot
@@ -1528,6 +1677,9 @@ pub fn placements_to_release_via_mapping(
             | TransactionUnit::InodeRoot
             | TransactionUnit::AllocationTree
             | TransactionUnit::AccountingTree => {
+                if !has_a_placement_in_the_previous_version(previous, identity) {
+                    continue;
+                }
                 match mapping_locations_of_a_mapped_unit(previous, identity) {
                     MappingLookup::Found(locations) => locations,
                     MappingLookup::NodeMalformedOrNoEntryWithThisKey => {
@@ -1544,7 +1696,10 @@ pub fn placements_to_release_via_mapping(
             }
             TransactionUnit::MappingTree => previous.root.mapping_root.locations,
             TransactionUnit::TreeTable => previous.root.tree_table.locations,
-            TransactionUnit::InstanceTable => previous.root.instance_table.locations,
+            // 整条旧链由 `instance_table_chain_to_release` 按计划带着的那条链释放。
+            TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
+                continue
+            }
         };
         placements.push(placement_to_release_after_checking_every_device(
             identity, &locations, allocator,
@@ -1553,6 +1708,35 @@ pub fn placements_to_release_via_mapping(
     Ok(placements)
 }
 
+/// 整条实例表链重写时被换下的那条旧链，逐片的落点（D18（块里携带什么信息） 已定项 11「每次写行 COW 重写整条链」）：
+/// `replaced_chain` 是那条旧链每一片的指针，按片序号（第 0 片是上一版根记录里那一条，第 k 片是第 k − 1 片链指针记录里那一条）。
+/// 实例表豁免映射（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12），不经映射、不做释放之前的读盘核；每一片照样按
+/// `placement_to_release_after_checking_every_device` 核三样（两条位置条目同槽、池里每块盘在册、没释放过、跨度对得上）。
+/// 只查不改：带文件的一版上写行（`publish_version`）、树表 0 条的一版上写行（`publish_instance_table_on_version_without_file`）
+/// 与可写挂载取号之前的准入（`mount`）都调它，三处判的是同一件事。
+///
+/// # Errors
+/// 某一片的三样核不过 ⇒ `ReleaseTargetLocationsOnDifferentSlots` / `ReleaseTargetNotAllocated` /
+/// `ReleaseTargetAlreadyReleased` / `ReleaseSpanMismatch`，带着是第几片（角色）。
+pub fn instance_table_chain_to_release(
+    replaced_chain: &[NodePointer],
+    allocator: &PoolAllocator,
+) -> Result<Vec<Placement>, PublishError> {
+    replaced_chain
+        .iter()
+        .enumerate()
+        .map(|(position, page_pointer)| {
+            placement_to_release_after_checking_every_device(
+                TransactionUnit::of_instance_table_page(InstanceTablePageIndex(
+                    u64::try_from(position).expect("片序号"),
+                )),
+                &page_pointer.locations,
+                allocator,
+            )
+        })
+        .collect()
+}
+
 /// 这次发布经映射换下的角色：这次重写的角色，加上文件内容重写时上一版比这一版多出来的那几个数据单元（文件变短）——
 /// 它们这次没有对应的新角色，却同样被换下，不释放它们，它们就一直占着槽、再也没有树引用。释放判定路径
 /// （`placements_to_release_via_mapping`）与释放之前读盘核校验和（`copies_failing_the_release_checksum_check`）读的是这同一张清单
@@ -1606,6 +1790,9 @@ fn has_a_placement_in_the_previous_version(
         | TransactionUnit::MappingTree
         | TransactionUnit::TreeTable
         | TransactionUnit::InstanceTable => true,
+        // 第 1 片起的实例表页在不在上一版里，上一版的内存态答不出（指针只在上一片的链指针记录里）；两个调用方都只拿进映射的角色问，
+        // 旧链由 `instance_table_chain_to_release` 按计划带着的那条链释放。这一臂只为穷举。
+        TransactionUnit::InstanceTablePageAfterTheFirst(_) => false,
     }
 }
 
@@ -1628,14 +1815,23 @@ fn mapping_locations_of_a_mapped_unit(
 /// 释放之前按映射条目的位置项读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1，用户 2026-09-23 定案）：
 /// 这次重写的角色里经映射释放的那几个（数据、extent 根、inode 叶容器与根、分配记录树、记账树），映射条目的每条位置项指的那一份
 /// 整单元读出来算 CRC-32C、与位置项里的比（写的时候位置项的校验和就是整单元 CRC-32C，`make_filesystem::location_entries`）。
-/// 交回比不上的那几份，按角色次序、位置项次序，不重复；发布照常释放、随即把它们各自在那块盘上隔离，发布照成。
+/// **读盘本身失败先重读一次，还读不出就按对不上处置**（用户 2026-09-24 定案，C394 三问的第一问）：只重读一次，不退避、不多次重试。
+/// 交回对不上的那几份，按角色次序、位置项次序，不重复；发布照常释放（映射条目去掉），只是对不上的那一份在它那块盘上的分配记录
+/// 留在「已分配」、不改成已释放（`PoolAllocator::release_leaving_the_record_allocated_on`，同一次定案的第三问），发布照成。
 /// 映射树、树表、实例表豁免映射，硬规则 1 的读盘核挂在「经映射核到那条映射条目之后」，它们不核。
 ///
+/// **一个单元只有一部分副本对不上时停在这一格**（`ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported`）：
+/// 照定案只把对不上的那几块盘的记录留在已分配、别的盘照常释放，两块盘的分配记录从此不对称——第一版的冷启动走读要求每块盘的
+/// （槽, 跨度, 代, 已释放）集合相同（`recovery::allocation_records_are_one_per_device`，不相同就判整池走读失败），分配器的落点也按
+/// 「各盘一致才分配」（`PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported`），这两处怎么容下不对称的账没有条款。
+/// 每一份都对不上（或都读不出）时两块盘的记录一起留在已分配，账仍对称，照定案做。
+///
 /// 只读，在动分配器、发任何一个写之前调；调用方先走 `placements_to_release_via_mapping`、它核过了才走到这里。
 ///
 /// # Errors
-/// 某一份读不到（`PoolReader::read` 交回 `None`：读盘报错、位置项指的盘不在池里或越界）⇒
-/// `ReleaseChecksumReadFailedWhoseHandlingIsUndecided`，带着是哪一份；读到之前的几份不算数。
+/// 某条位置项指的盘不在这个池里 ⇒ `ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided`：
+/// 那一份读不到不是读盘失败，「在它那块盘上的分配记录留在已分配」也没有那块盘可留，条款没写这一格；读到之前的几份不算数。
+/// 一个单元有的副本对不上、有的对得上 ⇒ `ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported`（见上）。
 ///
 /// # Panics
 /// 某个经映射释放的角色在上一版的映射里查不到：`placements_to_release_via_mapping` 刚按同一个上一版、同一串角色查过，
@@ -1645,11 +1841,9 @@ pub fn copies_failing_the_release_checksum_check<Reader: PoolReader + ?Sized>(
     roles: &[TransactionUnit],
     reader: &Reader,
 ) -> Result<Vec<CopyQuarantinedAfterReleaseChecksumMismatch>, PublishError> {
+    let pool_devices = reader.device_identities();
     let mut failing = Vec::new();
     for identity in roles_replaced_via_mapping(previous, roles) {
-        if !has_a_placement_in_the_previous_version(previous, identity) {
-            continue;
-        }
         match identity {
             TransactionUnit::Data(_)
             | TransactionUnit::ExtentRoot
@@ -1659,7 +1853,11 @@ pub fn copies_failing_the_release_checksum_check<Reader: PoolReader + ?Sized>(
             | TransactionUnit::AccountingTree => {}
             TransactionUnit::MappingTree
             | TransactionUnit::TreeTable
-            | TransactionUnit::InstanceTable => continue,
+            | TransactionUnit::InstanceTable
+            | TransactionUnit::InstanceTablePageAfterTheFirst(_) => continue,
+        }
+        if !has_a_placement_in_the_previous_version(previous, identity) {
+            continue;
         }
         let MappingLookup::Found(locations) =
             mapping_locations_of_a_mapped_unit(previous, identity)
@@ -1670,20 +1868,17 @@ pub fn copies_failing_the_release_checksum_check<Reader: PoolReader + ?Sized>(
         };
         let unit_bytes = usize::try_from(identity.span_slots() * SLOT_BYTES)
             .expect("一个单元两槽以内，字节数装得进 usize");
+        let failing_before_this_unit = failing.len();
         for location in &locations {
-            let copy = reader
-                .read(
-                    location.device,
-                    location.slot.to_device_offset(),
-                    unit_bytes,
-                )
-                .ok_or(
-                    PublishError::ReleaseChecksumReadFailedWhoseHandlingIsUndecided {
+            if !pool_devices.contains(&location.device) {
+                return Err(
+                    PublishError::ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided {
                         unit: identity,
                         device: location.device,
                         slot: location.slot,
                     },
-                )?;
+                );
+            }
             let quarantined = CopyQuarantinedAfterReleaseChecksumMismatch {
                 unit: identity,
                 device: location.device,
@@ -1692,11 +1887,40 @@ pub fn copies_failing_the_release_checksum_check<Reader: PoolReader + ?Sized>(
                     span: identity.span_slots(),
                 },
             };
+            let read_the_copy = || {
+                reader.read(
+                    location.device,
+                    location.slot.to_device_offset(),
+                    unit_bytes,
+                )
+            };
+            // 读不出先重读一次（只一次）；还读不出就按对不上处置。
+            let Some(copy) = read_the_copy().or_else(read_the_copy) else {
+                if !failing.contains(&quarantined) {
+                    failing.push(quarantined);
+                }
+                continue;
+            };
             if crc32_castagnoli(&copy) != location.unit_checksum && !failing.contains(&quarantined)
             {
                 failing.push(quarantined);
             }
         }
+        let devices_whose_copy_failed: Vec<DeviceIdentity> = failing[failing_before_this_unit..]
+            .iter()
+            .map(|copy| copy.device)
+            .collect();
+        let every_copy_failed = pool_devices
+            .iter()
+            .all(|device| devices_whose_copy_failed.contains(device));
+        if !devices_whose_copy_failed.is_empty() && !every_copy_failed {
+            return Err(
+                PublishError::ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported {
+                    unit: identity,
+                    devices_whose_copy_failed,
+                },
+            );
+        }
     }
     Ok(failing)
 }
@@ -1863,15 +2087,25 @@ pub enum PublishError {
         entry_bytes: usize,
         field_table_bytes: usize,
     },
-    /// 释放之前按映射条目的位置项读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1），这一份没读到：读盘报错，
-    /// 或位置项指的盘不在池里、槽越过盘尾（`PoolReader::read` 把这几样都交成 `None`，分不开）。
-    /// **读盘本身失败归哪个错误成员、之后是拒掉这次发布还是照「对不上」隔离，条款没有写**（C394（释放判定不核映射条目位置项里的单元校验和）
-    /// 前置那三件之一）⇒ 第一版不支持：在动分配器、发任何一个写之前返回，盘上逐字节不变。
-    ReleaseChecksumReadFailedWhoseHandlingIsUndecided {
+    /// 释放之前按映射条目的位置项读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1），这条位置项指的盘不在这个池里：
+    /// 映射条目是盘上读来的字节（坏镜像、外来镜像才有），这一份读不到不是「读盘本身失败」——用户 2026-09-24 定的「先重读一次、
+    /// 还读不出就按对不上处置、那一份在它那块盘上的分配记录留在已分配」三句都没有对象（池里没有那块盘、也就没有那条记录可留），
+    /// 条款没写这一格 ⇒ 第一版不支持：在动分配器、发任何一个写之前返回，盘上逐字节不变。
+    ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided {
         unit: TransactionUnit,
         device: DeviceIdentity,
         slot: SlotNumber,
     },
+    /// 释放之前按映射条目的位置项读盘核校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1），这个单元只有一部分副本对不上
+    /// （或读不出，先重读一次之后）、别的副本对得上。用户 2026-09-24 定「对不上的那一份在它那块盘上的分配记录留在已分配」，照做的话
+    /// 另一块盘的那一份照常释放，两块盘的分配记录从此不对称；而第一版的冷启动走读要求每块盘的记录集合相同
+    /// （`recovery::allocation_records_are_one_per_device`，不同就判整池走读失败——2026-09-24 故障注入快档打中），
+    /// 用户数据落点也按「各盘一致才分配」，这两处怎么容下不对称的账没有条款 ⇒ 第一版不支持：在动分配器、发任何一个写之前返回，
+    /// 盘上逐字节不变。`devices_whose_copy_failed` 是对不上的那几块盘。
+    ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported {
+        unit: TransactionUnit,
+        devices_whose_copy_failed: Vec<DeviceIdentity>,
+    },
     /// 用户给的内容装不进一个数据单元（32768 − 头 − 预留）：`publish_first_file` 与 `publish_overwrite` 按一个数据单元写
     /// （它们的契约），多单元的内容走 `publish_sequential_write`。
     ContentExceedsDataUnit {
@@ -1891,16 +2125,6 @@ pub enum PublishError {
     /// 这次发布要往 inode 树里写的记录，落法要的条款仓里还没定（[`InodeTreeWriteRefusal`] 的三格）：
     /// 在分配器、块设备都还没被碰过的时候返回，盘上逐字节不变。
     InodeTreeWriteRefused(InodeTreeWriteRefusal),
-    /// 这次发布末条记录要点名的单元多于一条 journal 记录装得下的 67 项（D23（journal 的角色与格式） 已定项 12 的 4 KiB 定长记录 +
-    /// 已定项 17 的 56 字节点名项）。一次发布切成 N 个事务、N 条记录时，前 N − 1 条各只点名自己那个数据单元，这次发布共享的
-    /// 提交内生块**只在最后一条点名**（已定项 17，2026-09-23 定）⇒ 末条要点名的是它自己的数据单元加全部共享内生块；
-    /// 这些装不进一条记录时，「只在最后一条点名」这一句就做不到，而装不下时怎么办（末条再跨几条记录、
-    /// 发布边界那时怎么认，已定项 14 第六条）仓里没有条款 ⇒ 在任何落盘动作之前返回，盘上逐字节不变。
-    /// `named_units` 是末条要点名的项数，`capacity` 是一条记录装得下的项数。
-    MoreNamedUnitsThanOneJournalRecordHolds {
-        named_units: usize,
-        capacity: usize,
-    },
     /// `publish_first_file` 要建在上面的那一版的根与交给它的上一条记录说的不是同一版：记录解不出本池的记录（`None`），
     /// 或记录的 checkpoint_txg 与那条根的不同。新根的 txg 从那条根接着算、jsn 从那条记录接着算，两者对不上时接上去会盖在别的发布上。
     FirstFileVersionDoesNotFollowTheVersionItBuildsOn {
@@ -2004,11 +2228,46 @@ pub struct FileVersionPlan<'content> {
     pub change_count: u64,
 }
 
-/// 实例表单元这次发布怎么处理：根记录照抄上一版的指针，或者重写成给定的记录（行记录在前、链指针记录最末，D18（块里携带什么信息） 已定项 11）。
+/// 实例表单元这次发布怎么处理：根记录照抄上一版的指针，或者整条链重写（D18（块里携带什么信息） 已定项 11「每次写行 COW 重写整条链」）。
 #[derive(Clone, Debug)]
 pub enum InstanceTablePlan {
     Carry(NodePointer),
-    Rewrite(Vec<Vec<u8>>),
+    Rewrite(InstanceTableRewrite),
+}
+
+/// 整条实例表链重写：这次之后整张表的行，与被这次换下的那条旧链。
+#[derive(Clone, Debug)]
+pub struct InstanceTableRewrite {
+    /// 这次之后整张表的行，按链上的次序（上一版那张表的行在前、这次写的接在后面）。怎么分到各片由
+    /// `instance_table_rows_of_each_page` 定（一片写满 369 行再开下一片）。
+    pub rows: Vec<InstanceRow>,
+    /// 被这次重写换下的那条旧链每一片的指针，按片序号：第 0 片是上一版根记录里那一条，第 k 片是第 k − 1 片链指针记录里那一条
+    /// （`recovery::instance_table_chain_of_root` 读出来的 `page_pointers`）。发布逐片释放它们。
+    /// 第 1 片起的指针不在上一版的内存态（`TransactionOutput`）里，所以由拼这张计划的调用方带进来：它拼行读的就是这同一条链，
+    /// 接行与释放读的是同一份输入。
+    pub replaced_chain: Vec<NodePointer>,
+}
+
+impl InstanceTableRewrite {
+    /// 这次之后这条链有几片（D28（挂载期承诺量） 已定项 3 的 max(1, ⌈行数 ÷ 369⌉)）。
+    #[must_use]
+    pub fn pages_after_this_publish(&self) -> usize {
+        instance_table_pages_for_rows(self.rows.len())
+    }
+}
+
+/// 重写一条 `pages` 片的实例表链时各片的角色，按 bump 次序：尾片先（D3（空间分配） 已定项 10 ⑤，用户 2026-09-24 定案：
+/// 第 k 片当第 k + 1 片的父，照先叶后根），第 0 片最后。这个次序同时是各片取落点与发出生序号的次序（D19（块指针的结构与宽度预算） 已定项 9）。
+#[must_use]
+pub fn instance_table_page_roles_in_bump_order(pages: usize) -> Vec<TransactionUnit> {
+    (0..pages)
+        .rev()
+        .map(|position| {
+            TransactionUnit::of_instance_table_page(InstanceTablePageIndex(
+                u64::try_from(position).expect("片序号"),
+            ))
+        })
+        .collect()
 }
 
 /// 一次发布的全部参数：哪些角色重写、身份字段取什么。第一个事务、覆盖写、写行、暖机都是它的一种取值，走同一条发布路径
@@ -2059,27 +2318,38 @@ pub struct PublishShape {
     pub rewritten_data_units: usize,
     /// 这次重写几片 inode 叶容器；0 = 这次一个字节都不碰 inode 树（连根都不重写）。
     pub rewritten_inode_leaf_containers: usize,
-    pub rewrites_instance_table: bool,
+    /// 这次重写的实例表链有几片；0 = 实例表照抄。
+    pub rewritten_instance_table_pages: usize,
 }
 
 impl PublishShape {
-    /// 写行那次发布的形状：不写文件内容、不碰 inode 树、重写实例表（`publish_rows_on_file_version` 交给 `publish_version` 的那张计划同形）。
-    pub const ROW_PUBLISH: PublishShape = PublishShape {
-        rewritten_data_units: 0,
-        rewritten_inode_leaf_containers: 0,
-        rewrites_instance_table: true,
-    };
+    /// 写行那次发布、实例表链这次之后只有一片时的形状（行数不超过 369，第一个事务之后的多数挂载都是它）：
+    /// 多于一片的见 [`PublishShape::row_publish_rewriting_instance_table_pages`]。
+    pub const ROW_PUBLISH: PublishShape =
+        PublishShape::row_publish_rewriting_instance_table_pages(1);
+
+    /// 写行那次发布的形状：不写文件内容、不碰 inode 树、实例表整条链重写成 `instance_table_pages` 片
+    /// （`publish_rows_on_file_version` 交给 `publish_version` 的那张计划同形）。
+    #[must_use]
+    pub const fn row_publish_rewriting_instance_table_pages(instance_table_pages: usize) -> Self {
+        PublishShape {
+            rewritten_data_units: 0,
+            rewritten_inode_leaf_containers: 0,
+            rewritten_instance_table_pages: instance_table_pages,
+        }
+    }
 
     /// 暖机那几次空发布的形状：不写文件内容、不碰 inode 树、实例表照抄（`publish_empty_after` 交给 `publish_version` 的那张计划同形，
     /// 那里有一条断言钉住两者相等）。记账树已经存在 ⇒ 照样重写四个固定点单元（D16（发布语义） 已定项 9）。
     pub const EMPTY_PUBLISH: PublishShape = PublishShape {
         rewritten_data_units: 0,
         rewritten_inode_leaf_containers: 0,
-        rewrites_instance_table: false,
+        rewritten_instance_table_pages: 0,
     };
 
     /// 这次发布重写的角色，按 bump 次序：用户数据先取（自己的政策）；提交内生块按树 ID 升序、树内先叶后根、映射树倒数第二、树表最末
-    /// （D3（空间分配） 已定项 10 ⑤）。实例表单元归树 0，排在全部提交内生块之前（mkfs 也是先实例表后树表；里程碑步 3 的决策点）。
+    /// （D3（空间分配） 已定项 10 ⑤）。实例表单元归树 0，排在全部提交内生块之前（mkfs 也是先实例表后树表；里程碑步 3 的决策点），
+    /// 多于一片时尾片先（[`instance_table_page_roles_in_bump_order`]）。
     ///
     /// ⚠️ **叶容器在这里按 0 起的连号排，不是它们在树里的真实叶序**：形状只答「几个角色」（准入按条数算，
     /// `publish_sequence_admission` 在取号之前要的就是这个数），答不了「改的是哪几片」——那要看上一版的树长什么样。
@@ -2092,9 +2362,9 @@ impl PublishShape {
                 u64::try_from(position).expect("单元序号"),
             )));
         }
-        if self.rewrites_instance_table {
-            roles.push(TransactionUnit::InstanceTable);
-        }
+        roles.extend(instance_table_page_roles_in_bump_order(
+            self.rewritten_instance_table_pages,
+        ));
         if self.rewritten_data_units > 0 {
             roles.push(TransactionUnit::ExtentRoot);
         }
@@ -2135,9 +2405,11 @@ impl ResolvedPublish {
                 .filter(|identity| matches!(identity, TransactionUnit::Data(_)))
                 .count(),
             rewritten_inode_leaf_containers: self.inode_tree.rewritten.len(),
-            rewrites_instance_table: self
+            rewritten_instance_table_pages: self
                 .rewritten_roles
-                .contains(&TransactionUnit::InstanceTable),
+                .iter()
+                .filter(|identity| identity.is_a_page_of_the_instance_table())
+                .count(),
         }
     }
 }
@@ -2192,8 +2464,10 @@ impl PublishPlan<'_> {
             .iter()
             .map(|transaction| TransactionUnit::Data(transaction.unit_index_in_file))
             .collect();
-        if matches!(self.instance_table, InstanceTablePlan::Rewrite(_)) {
-            rewritten_roles.push(TransactionUnit::InstanceTable);
+        if let InstanceTablePlan::Rewrite(rewrite) = &self.instance_table {
+            rewritten_roles.extend(instance_table_page_roles_in_bump_order(
+                rewrite.pages_after_this_publish(),
+            ));
         }
         if self.file.is_some() {
             rewritten_roles.push(TransactionUnit::ExtentRoot);
@@ -2392,8 +2666,8 @@ pub fn publish_overwrite<Device: BlockDevice>(
 ///
 /// # Errors
 /// 切出的单元多于一片 extent 叶装得下的记录数 ⇒ `ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided`
-/// （extent 内部条目的格式没有条款）；末条要点名的项多于一条记录装得下的 ⇒ `MoreNamedUnitsThanOneJournalRecordHolds`。
-/// 两样都在分配器、块设备都还没被碰过的时候返回，盘上逐字节不变。其余与 `publish_overwrite` 相同
+/// （extent 内部条目的格式没有条款），在分配器、块设备都还没被碰过的时候返回，盘上逐字节不变。
+/// 最后一个事务要点名的项多于一条记录装得下的，末条再跨记录（D23（journal 的角色与格式） 已定项 17），不拒。其余与 `publish_overwrite` 相同
 /// （不报 `ContentExceedsDataUnit`：多个数据单元正是这条路径要写的）。
 pub fn publish_sequential_write<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
@@ -2450,8 +2724,9 @@ pub fn publish_sequential_write<Device: BlockDevice>(
 ///
 /// # Errors
 /// 记录落法要的条款没定（中间插入、容器数超过一个根装得下的）⇒ `InodeTreeWriteRefused`；
-/// 这次要点名的单元超过一条 journal 记录装得下的 67 项 ⇒ `MoreNamedUnitsThanOneJournalRecordHolds`；
-/// 其余与 `publish_overwrite` 相同。三样都在任何落盘动作之前返回，盘上逐字节不变。
+/// 其余与 `publish_overwrite` 相同。两样都在任何落盘动作之前返回，盘上逐字节不变。
+/// 这次要点名的单元超过一条 journal 记录装得下的 67 项时不拒：这个事务的记录再跨几条，只有真正的最后一条带
+/// 「本次发布末条」标志与提交标记（D23（journal 的角色与格式） 已定项 17 / 已定项 7）。
 pub fn publish_new_inodes<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
@@ -2545,7 +2820,7 @@ fn publish_version_of_trees_holding_one_data_unit<Device: BlockDevice>(
     publish_version_of_trees(pool, allocator, plan, previous, trees)
 }
 
-/// 发布一版：先做准入（extent 树装得下这个文件的数据单元、末条记录装得下要点名的项，再加 `publish_admission` 的三条），
+/// 发布一版：先做准入（extent 树装得下这个文件的数据单元，再加 `publish_admission` 的三条），
 /// 再释放上一版被换下的角色的落点、分配、装单元、
 /// 按 D16（发布语义） 已定项 7 的持久顺序落盘。准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立，
 /// 释放与分配都不算数（第二轮攻方腿：落点被拒（当时叫 `NoSpaceFor`）在释放之后、分配到一半返回，留下半新的池，拿同一个上一版重试撞断言）；
@@ -2556,7 +2831,7 @@ fn publish_version_of_trees_holding_one_data_unit<Device: BlockDevice>(
 /// （回退到树表 0 条的一版之后）走 `publish_first_file`，它按要建在上面的那一版的水位发。
 ///
 /// # Errors
-/// `InodeTreeWriteRefused`、`MoreNamedUnitsThanOneJournalRecordHolds`、`ContentExceedsDataUnit`、
+/// `InodeTreeWriteRefused`、`ContentExceedsDataUnit`、
 /// `AllocationRecordsExceedOneNode`、`AccountingEntriesExceedOneNode`、`MappingEntriesExceedOneNode`、
 /// 释放判定路径的四种错、`PlacementRefused`、块设备错。
 ///
@@ -2615,19 +2890,8 @@ fn publish_version_of_trees<Device: BlockDevice>(
             },
         );
     }
-    // 点名项一条记录装 67 个（D23（journal 的角色与格式） 已定项 12 / 已定项 17）：前几条各只点名一个数据单元，
-    // 共享的提交内生块只在末条点名 ⇒ 末条装不下时「只在最后一条点名」做不到，装不下怎么办没有条款 ⇒ 在动分配器之前拒掉。
-    // 切法与落盘时写出的记录读同一个函数（`roles_named_by_each_record_of_the_publish`）。
-    let named_unit_capacity = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("67");
-    let named_units_of_the_last_record = roles_named_by_each_record_of_the_publish(&rewritten)
-        .last()
-        .map_or(0, Vec::len);
-    if named_units_of_the_last_record > named_unit_capacity {
-        return Err(PublishError::MoreNamedUnitsThanOneJournalRecordHolds {
-            named_units: named_units_of_the_last_record,
-            capacity: named_unit_capacity,
-        });
-    }
+    // 点名项一条记录装 67 个（D23（journal 的角色与格式） 已定项 12 / 已定项 17）：最后一个事务要点名的项装不下一条记录时
+    // 末条再跨记录（`roles_named_by_each_record_of_the_publish`），这里不拒。
     // 释放判定路径先于准入：它只查不改（D19（块指针的结构与宽度预算） 已定项 5 第 1 条），查不到就整次发布不做。
     let release = match previous {
         Some(previous_version) => {
@@ -2637,6 +2901,24 @@ fn publish_version_of_trees<Device: BlockDevice>(
         // （D3（空间分配） 已定项 7；不释放它，txg 0 的根离开候选集之后这一槽就永远占着，I-3.1（已分配统计对得上） 在抬 F 之后红）。
         None => format_time_tree_table_to_release(allocator, &rewritten),
     };
+    // 整条实例表链重写（写行、回退那一次）：被换下的那条旧链逐片释放，指针取计划带着的那条链（第 1 片起的不在上一版的内存态里）。
+    // 旧链排在别的角色前面，与实例表在 bump 次序里排最前同一个次序。
+    let release = match &plan.instance_table {
+        InstanceTablePlan::Rewrite(rewrite) => {
+            if let Some(previous_version) = previous {
+                assert_eq!(
+                    rewrite.replaced_chain.first(),
+                    Some(&previous_version.root.instance_table),
+                    "计划带着的旧链是上一版根记录指着的那一条：调用方拼行与拼旧链读的是同一条根"
+                );
+            }
+            let mut chain_then_the_rest =
+                instance_table_chain_to_release(&rewrite.replaced_chain, allocator)?;
+            chain_then_the_rest.extend(release);
+            chain_then_the_rest
+        }
+        InstanceTablePlan::Carry(_) => release,
+    };
     // 经映射核到的那几个单元，释放之前再按位置项读盘核一次校验和（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）：只读，
     // 读不到就在动分配器、发任何一个写之前返回。第一个文件版本换下的 mkfs 那片树表与树表 0 条那一版的分配记录树节点不经映射，不核。
     let quarantine = match previous {
@@ -2834,12 +3116,16 @@ fn mapping_entry_count(
 }
 
 /// 一次发布切成几条 journal 记录、每条点名哪几个角色（D23（journal 的角色与格式） 已定项 17，C491（多条记录时共享内生块在哪条点名没定）
-/// 2026-09-23 定）：这次写了 N 个数据单元（N ≥ 2）⇒ N 个事务、N 条记录（C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案），
-/// 第 k 条（k < N − 1）只点名文件第 k 个数据单元；这次发布共享的提交内生块（数据单元之外的每个重写角色）**只在最后一条点名**，
-/// 与第 N − 1 个数据单元一起，前面几条不重复点名它们。写了一个数据单元或一个都没写 ⇒ 一条记录点名全部重写角色
-/// （第一个事务今天的形态，字节一个不变）。
+/// 2026-09-23 定）：这次写了 N 个数据单元（N ≥ 2）⇒ N 个事务（C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案），
+/// 第 k 个事务（k < N − 1）一条记录、只点名文件第 k 个数据单元；这次发布共享的提交内生块（数据单元之外的每个重写角色）
+/// **只在最后一个事务的记录里点名**，与第 N − 1 个数据单元一起，前面几条不重复点名它们。写了一个数据单元或一个都没写 ⇒
+/// 一个事务点名全部重写角色（第一个事务今天的形态）。
+///
+/// 最后一个事务要点名的项多于一条记录装得下的 67 项（[`JOURNAL_NAMED_ENTRIES_PER_RECORD`]）时**末条再跨记录**
+/// （已定项 17，用户 2026-09-24 定）：按 bump 次序装满一条再开下一条，这些记录都属于最后一个事务
+/// （[`transaction_offset_of_each_record_of_the_publish`]）；只有真正的最后一条带「本次发布末条」标志。
 ///
-/// 每一项按 `rewritten` 里的次序（bump 次序）列角色；返回至少一项。准入（末条装不装得下）与落盘写出的记录读的都是它。
+/// 每一项按 `rewritten` 里的次序（bump 次序）列角色；返回至少一项，每项至多 67 个角色。落盘写出的记录读的就是它。
 #[must_use]
 pub fn roles_named_by_each_record_of_the_publish(
     rewritten: &[TransactionUnit],
@@ -2862,9 +3148,43 @@ pub fn roles_named_by_each_record_of_the_publish(
             .filter(|identity| !data_roles_named_before_the_last_record.contains(identity))
             .collect(),
     );
+    // 末条再跨记录（已定项 17）：最后一个事务那一项装不下一条记录时切成几条，装满一条再开下一条。
+    let roles_named_by_the_last_transaction = roles_named_by_each_record
+        .pop()
+        .expect("上一句刚推进去最后一个事务那一项");
+    if roles_named_by_the_last_transaction.is_empty() {
+        // 一个角色都不点名的发布照样写一条记录（它一条就是一次发布）。
+        roles_named_by_each_record.push(roles_named_by_the_last_transaction);
+    } else {
+        let named_unit_capacity = usize::try_from(JOURNAL_NAMED_ENTRIES_PER_RECORD).expect("67");
+        roles_named_by_each_record.extend(
+            roles_named_by_the_last_transaction
+                .chunks(named_unit_capacity)
+                .map(<[TransactionUnit]>::to_vec),
+        );
+    }
     roles_named_by_each_record
 }
 
+/// 一次发布里从 0 数第 k 条记录属于这次发布从 0 数的第几个事务（与 [`roles_named_by_each_record_of_the_publish`] 同一种切法）：
+/// 前 N − 1 条（N = 这次写的数据单元数，N ≥ 2）各是自己那个数据单元的事务，第 k 条就是第 k 个事务；
+/// 其余的——最后一个事务那条记录与它装不下再跨出去的几条（D23（journal 的角色与格式） 已定项 17）——都属于最后一个事务，
+/// 共享它的事务号（已定项 7「同一事务的全部记录共享它」）。
+#[must_use]
+pub fn transaction_offset_of_each_record_of_the_publish(rewritten: &[TransactionUnit]) -> Vec<u64> {
+    let transactions_named_one_record_each = rewritten
+        .iter()
+        .filter(|identity| matches!(identity, TransactionUnit::Data(_)))
+        .count()
+        .saturating_sub(1);
+    (0..roles_named_by_each_record_of_the_publish(rewritten).len())
+        .map(|record_offset_in_this_publish| {
+            u64::try_from(record_offset_in_this_publish.min(transactions_named_one_record_each))
+                .expect("一次发布的事务数装得进 u64")
+        })
+        .collect()
+}
+
 /// 上一版里某个角色的单元字节（照抄进这一版的 `units`）。
 fn carried_unit(previous: &TransactionOutput, identity: TransactionUnit) -> PublishedUnit {
     previous.unit(identity).clone()
@@ -3193,13 +3513,20 @@ fn publish_admitted<Device: BlockDevice>(
     let rewritten = &resolved.rewritten_roles;
     let txg = plan.txg;
     let instance = plan.instance;
-    // 这次发布切成几条记录、每条点名哪几个角色（D23（journal 的角色与格式） 已定项 17）：准入按同一个函数判过末条装得下。
+    // 这次发布切成几条记录、每条点名哪几个角色、属于第几个事务（D23（journal 的角色与格式） 已定项 17：最后一个事务装不下一条
+    // 记录时末条再跨记录）。
     let roles_named_by_each_record = roles_named_by_each_record_of_the_publish(rewritten);
+    let transaction_offset_of_each_record =
+        transaction_offset_of_each_record_of_the_publish(rewritten);
     let records_in_this_publish =
         u64::try_from(roles_named_by_each_record.len()).expect("记录条数");
-    // 一条记录一个事务（C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案）：第 k 条是第 `plan.transaction + k` 号事务，
-    // 共享的提交内生块在末条点名 ⇒ 它们头里的写序取末条那个事务的号（数据单元各带自己那个事务的号）。
-    let last_transaction_of_this_publish = plan.transaction + (records_in_this_publish - 1);
+    // 一个数据单元一个事务（C310（事务切分纪律与记录数口径打架） 2026-09-16 用户定案）：第 k 个事务是第 `plan.transaction + k` 号，
+    // 共享的提交内生块在最后一个事务的记录里点名 ⇒ 它们头里的写序取最后那个事务的号（数据单元各带自己那个事务的号）。
+    let last_transaction_of_this_publish = plan.transaction
+        + transaction_offset_of_each_record
+            .last()
+            .copied()
+            .expect("一次发布至少一条记录");
     let write_order = WriteOrder {
         instance,
         transaction: last_transaction_of_this_publish,
@@ -3209,13 +3536,19 @@ fn publish_admitted<Device: BlockDevice>(
     let mut sequences = BirthSequenceAllocator::default();
 
     // 释放先于分配（D3（空间分配） 已定项 7）：被换下的落点进 defer 队列、槽仍占着，这次的落点不会落到它们上面。
+    // 释放之前读盘核校验和核出对不上（读不出也算）的那几份：逻辑上照样释放（映射条目这次不再写它），那一份在它那块盘上的分配记录
+    // 留在已分配、不改成已释放（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-24 定案），核得上的那一份照常释放。
     for placement in release {
-        allocator.release(*placement, txg);
-    }
-    // 释放之前读盘核校验和核出对不上的那几份：逻辑上照样释放了（上面那一圈），物理槽随即在各自那块盘上隔离、不还回空闲池
-    // （D19（块指针的结构与宽度预算） 已定项 5 硬规则 1）。隔离在分配这次的落点之前，复用窗口置 0 时当场回收的槽也发不回它们上面。
-    for copy in quarantine {
-        allocator.quarantine_after_release_checksum_mismatch(copy.device, copy.placement);
+        let devices_whose_copy_failed_the_checksum: Vec<DeviceIdentity> = quarantine
+            .iter()
+            .filter(|copy| copy.placement == *placement)
+            .map(|copy| copy.device)
+            .collect();
+        allocator.release_leaving_the_record_allocated_on(
+            *placement,
+            txg,
+            &devices_whose_copy_failed_the_checksum,
+        );
     }
 
     // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。
@@ -3292,41 +3625,28 @@ fn publish_admitted<Device: BlockDevice>(
         .copied()
         .expect("inode 树里恒有第一个文件那条记录（inode 号 1）");
 
-    // 实例表单元（码 3 打包记录类型 4）：重写时容器身份照 mkfs（容器 0、出生代 0），归树 0、在树表之前发号（mkfs 也是先实例表后树表）。
-    let instance_table_rewrite = match &plan.instance_table {
-        InstanceTablePlan::Rewrite(records) => {
-            let sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
-            let unit = build_packed_unit(
-                PackedIdentity {
-                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
-                    record_type: PACKED_TYPE_INSTANCE_TABLE,
-                    container: 0,
-                    container_birth: CheckpointTxg(0),
-                },
-                u16::try_from(INSTANCE_ROW_BYTES).expect("88"),
-                records,
-                txg,
-                filesystem_identifier,
-                write_order,
-                sequence,
-            );
-            let pointer = NodePointer {
-                head: PointerHead {
-                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
-                    birth_txg: txg,
-                },
-                locations: pool.location_entries(slot_of(TransactionUnit::InstanceTable), &unit),
-                instance,
-                birth_sequence: sequence,
-            };
-            Some((unit, pointer, sequence))
-        }
-        InstanceTablePlan::Carry(_) => None,
+    // 实例表链（码 3 打包记录类型 4）：重写时每一片的容器身份是 (0, 4, 片序号, 0)（第 0 片照 mkfs），归树 0、在树表之前发号
+    // （mkfs 也是先实例表后树表）；多于一片时尾片先装、先发号（`build_instance_table_chain`）。
+    let instance_table_rewrite: Vec<InstanceTablePageUnit> = match &plan.instance_table {
+        InstanceTablePlan::Rewrite(rewrite) => build_instance_table_chain(
+            &rewrite.rows,
+            &slot_of,
+            txg,
+            write_order,
+            filesystem_identifier,
+            &device_identities,
+            &mut sequences,
+        ),
+        InstanceTablePlan::Carry(_) => Vec::new(),
     };
-    let instance_table_pointer = match (&plan.instance_table, &instance_table_rewrite) {
-        (InstanceTablePlan::Carry(pointer), _) => *pointer,
-        (InstanceTablePlan::Rewrite(_), Some((_, pointer, _))) => *pointer,
-        (InstanceTablePlan::Rewrite(_), None) => unreachable!("重写时上面一定装了单元"),
+    let instance_table_pointer = match &plan.instance_table {
+        InstanceTablePlan::Carry(pointer) => *pointer,
+        InstanceTablePlan::Rewrite(_) => {
+            instance_table_rewrite
+                .first()
+                .expect("一条链至少一片：0 行也写出空的那一片")
+                .pointer
+        }
     };
 
     // t5 分配记录树（字节表五）：mkfs 的两个单元分配代 0、其余是各自分配那次发布的 txg，每盘各一条，按 (设备, 槽号) 升序。
@@ -3658,7 +3978,7 @@ fn publish_admitted<Device: BlockDevice>(
     );
 
     // 这一版全部角色的单元：重写的是这次装的，照抄的从上一版拷（按 bump 次序：每个数据单元、extent 根、每片 inode 叶容器、
-    // inode 根、四个固定点单元，实例表单元在末尾）。
+    // inode 根、四个固定点单元，实例表链的各片在末尾）。
     let rewritten_unit = |identity: TransactionUnit, bytes: Vec<u8>| PublishedUnit {
         slot: slot_of(identity),
         identity,
@@ -3712,20 +4032,23 @@ fn publish_admitted<Device: BlockDevice>(
         TransactionUnit::TreeTable,
         tree_table_unit.clone(),
     ));
-    match (&instance_table_rewrite, previous) {
-        (Some((unit, _, _)), _) => {
-            units.push(rewritten_unit(TransactionUnit::InstanceTable, unit.clone()))
-        }
-        (None, Some(previous_version)) => {
-            if let Some(carried) = previous_version
-                .units
-                .iter()
-                .find(|unit| unit.identity == TransactionUnit::InstanceTable)
-            {
-                units.push(carried.clone());
+    // 实例表链上的各片按链上的次序（第 0 片在前）：重写的是这次装的，照抄的是上一版带着的那几片。
+    match (&plan.instance_table, previous) {
+        (InstanceTablePlan::Rewrite(_), _) => {
+            for page in &instance_table_rewrite {
+                units.push(rewritten_unit(page.role, page.bytes.clone()));
             }
         }
-        (None, None) => {}
+        (InstanceTablePlan::Carry(_), Some(previous_version)) => {
+            units.extend(
+                previous_version
+                    .units
+                    .iter()
+                    .filter(|unit| unit.identity.is_a_page_of_the_instance_table())
+                    .cloned(),
+            );
+        }
+        (InstanceTablePlan::Carry(_), None) => {}
     }
 
     // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的每个角色一项，key 尾段与映射 key 共用；照抄的不点名。
@@ -3759,13 +4082,17 @@ fn publish_admitted<Device: BlockDevice>(
             TransactionUnit::AccountingTree => node_key_tail(instance, accounting_sequence),
             TransactionUnit::MappingTree => node_key_tail(instance, mapping_sequence),
             TransactionUnit::TreeTable => node_key_tail(instance, tree_table_sequence),
-            TransactionUnit::InstanceTable => node_key_tail(
-                instance,
-                instance_table_rewrite
-                    .as_ref()
-                    .map(|(_, _, sequence)| *sequence)
-                    .expect("点名实例表单元的发布一定重写了它"),
-            ),
+            TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
+                node_key_tail(
+                    instance,
+                    instance_table_rewrite
+                        .iter()
+                        .find(|page| page.role == identity)
+                        .expect("点名实例表那一片的发布一定重写了整条链")
+                        .pointer
+                        .birth_sequence,
+                )
+            }
         }
     };
     let written_units: Vec<&PublishedUnit> = rewritten
@@ -3791,12 +4118,13 @@ fn publish_admitted<Device: BlockDevice>(
         }
     };
     // 这次发布的记录（D23（journal 的角色与格式） 已定项 7 / 已定项 17）：第 k 条的 jsn 计数器是 `plan.counter + k`、
-    // 事务号是 `plan.transaction + k`，一条记录一个事务、各带提交标记（一个事务不跨记录，C310（事务切分纪律与记录数口径打架）
-    // 2026-09-16 用户定案）；每条都带整次发布的新根段（已定项 15），发布边界由「点名了共享内生块的末条」认（已定项 14 第六条）。
-    // 反向链：第一条接 `plan.back_chain`（上一条记录的头），之后每条接这次发布里前一条的头（已定项 8）。
+    // 事务号是 `plan.transaction` 加它属于的那个事务的序号（前 N − 1 条一条一个事务；最后一个事务装不下一条记录时再跨几条，
+    // 那几条共享它的事务号）。提交标记只在一个事务的最后一条记录上（已定项 7；I-8.8（前缀里的事务不被切开） ③），
+    // 「本次发布末条」标志只在这次发布真正的最后一条上（已定项 17）——恢复按它认发布边界（已定项 14 第六条）。
+    // 每条都带整次发布的新根段（已定项 15）。反向链：第一条接 `plan.back_chain`（上一条记录的头），之后每条接这次发布里前一条的头（已定项 8）。
     let mut written_records: Vec<WrittenJournalRecord> =
         Vec::with_capacity(roles_named_by_each_record.len());
-    // 迭代次数的上界是这次的记录条数（= 数据单元数，至少 1）；跨轮携带的只有已经装好的记录（下一条的反向链要罩前一条的头）。
+    // 迭代次数的上界是这次的记录条数（≥ 1）；跨轮携带的只有已经装好的记录（下一条的反向链要罩前一条的头）。
     for (record_offset_in_this_publish, named_roles) in
         roles_named_by_each_record.iter().enumerate()
     {
@@ -3807,17 +4135,29 @@ fn publish_admitted<Device: BlockDevice>(
                 back_chain_of(&previous_record_of_this_publish.bytes)
             }
         };
+        let transaction_offset = transaction_offset_of_each_record[record_offset_in_this_publish];
+        let transaction_of_the_next_record = transaction_offset_of_each_record
+            .get(record_offset_in_this_publish + 1)
+            .copied();
+        let is_the_last_record_of_its_transaction =
+            transaction_of_the_next_record != Some(transaction_offset);
+        let place_in_publish = if transaction_of_the_next_record.is_none() {
+            JournalRecordPlaceInPublish::LastRecordOfThePublish
+        } else {
+            JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
+        };
         let record = JournalRecord {
             instance,
             counter: plan.counter + offset,
             checkpoint_txg: txg,
-            transaction: plan.transaction + offset,
-            is_commit: true,
+            transaction: plan.transaction + transaction_offset,
+            is_commit: is_the_last_record_of_its_transaction,
             // 本次发布内序号依次是 1..N（D23（journal 的角色与格式） 已定项 4）：所选根那条记录读不出时，
             // 恢复只从序号 1 那条接链首（已定项 14 注 1）。
             ordinal_within_publish: JournalRecordOrdinalWithinPublish::of_record_at_offset(
                 record_offset_in_this_publish,
             ),
+            place_in_publish,
             back_chain,
             filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
             new_tree_table: tree_table_pointer,
@@ -4166,4 +4506,95 @@ mod tests {
             ["t1", "t2", "t3", "t4", "t5", "t6", "t7", "t8"]
         );
     }
+
+    /// 一次发布的重写角色：`data_units` 个数据单元、extent 根、`leaf_containers` 片 inode 叶容器、inode 根与四个固定点单元，按 bump 次序。
+    fn rewritten_roles(data_units: u64, leaf_containers: usize) -> Vec<TransactionUnit> {
+        (0..data_units)
+            .map(|index| TransactionUnit::Data(DataUnitIndexInFile(index)))
+            .chain([TransactionUnit::ExtentRoot])
+            .chain((0..leaf_containers).map(|position| {
+                TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::of_position(
+                    position,
+                ))
+            }))
+            .chain([
+                TransactionUnit::InodeRoot,
+                TransactionUnit::AllocationTree,
+                TransactionUnit::AccountingTree,
+                TransactionUnit::MappingTree,
+                TransactionUnit::TreeTable,
+            ])
+            .collect()
+    }
+
+    /// 末条再跨记录（D23（journal 的角色与格式） 已定项 17）：最后一个事务要点名的项多于一条记录装得下的 67 项时，按 bump 次序
+    /// 装满一条再开下一条，这几条都属于最后一个事务；前 N − 1 个数据单元各一条、各是自己那个事务。
+    /// 两个数据单元 + extent 根 + 130 片叶容器 + 5 个角色 = 138 个：第一条只点名数据单元 0（事务 0），
+    /// 最后一个事务要点名 137 项 ⇒ 67 + 67 + 3 三条（事务都是 1）。不写数据单元的 68 个 ⇒ 67 + 1 两条（事务都是 0）。
+    /// 装得下的照旧：三个数据单元 ⇒ 三条三个事务，末条点名数据单元 2 与 6 个共享角色。
+    #[test]
+    fn the_last_transaction_spills_over_as_many_records_as_its_named_units_need_and_they_all_share_its_transaction(
+    ) {
+        let spilling_with_data = rewritten_roles(2, 130);
+        let records_with_data = roles_named_by_each_record_of_the_publish(&spilling_with_data);
+        assert_eq!(
+            records_with_data.iter().map(Vec::len).collect::<Vec<_>>(),
+            vec![1, 67, 67, 3],
+            "数据单元 0 一条；最后一个事务 137 项装满两条再开第三条"
+        );
+        assert_eq!(
+            records_with_data[0],
+            vec![TransactionUnit::Data(DataUnitIndexInFile(0))]
+        );
+        assert_eq!(
+            records_with_data.concat(),
+            spilling_with_data,
+            "几条合起来按 bump 次序恰好点名每个重写角色一次"
+        );
+        assert_eq!(
+            transaction_offset_of_each_record_of_the_publish(&spilling_with_data),
+            vec![0, 1, 1, 1],
+            "跨出去的两条属于最后一个事务"
+        );
+
+        let spilling_without_data = rewritten_roles(0, 62);
+        assert_eq!(spilling_without_data.len(), 68);
+        let records_without_data =
+            roles_named_by_each_record_of_the_publish(&spilling_without_data);
+        assert_eq!(
+            records_without_data
+                .iter()
+                .map(Vec::len)
+                .collect::<Vec<_>>(),
+            vec![67, 1]
+        );
+        assert_eq!(
+            transaction_offset_of_each_record_of_the_publish(&spilling_without_data),
+            vec![0, 0],
+            "一个事务跨两条"
+        );
+
+        let fitting = rewritten_roles(3, 1);
+        assert_eq!(
+            roles_named_by_each_record_of_the_publish(&fitting)
+                .iter()
+                .map(Vec::len)
+                .collect::<Vec<_>>(),
+            vec![1, 1, 8]
+        );
+        assert_eq!(
+            transaction_offset_of_each_record_of_the_publish(&fitting),
+            vec![0, 1, 2]
+        );
+
+        assert_eq!(
+            roles_named_by_each_record_of_the_publish(&[]),
+            vec![Vec::<TransactionUnit>::new()],
+            "一个角色都不重写的发布照样一条记录"
+        );
+        assert_eq!(
+            transaction_offset_of_each_record_of_the_publish(&[]),
+            vec![0]
+        );
+    }
 }
diff --git a/crates/singlefs-core/src/write_accounting.rs b/crates/singlefs-core/src/write_accounting.rs
index 15b67bc..8dd4ab6 100644
--- a/crates/singlefs-core/src/write_accounting.rs
+++ b/crates/singlefs-core/src/write_accounting.rs
@@ -56,7 +56,9 @@ impl WrittenStructureKind {
             TransactionUnit::AccountingTree => WrittenStructureKind::AccountingTreeNode,
             TransactionUnit::MappingTree => WrittenStructureKind::CentralMappingTreeNode,
             TransactionUnit::TreeTable => WrittenStructureKind::TreeTableUnit,
-            TransactionUnit::InstanceTable => WrittenStructureKind::InstanceTableUnit,
+            TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
+                WrittenStructureKind::InstanceTableUnit
+            }
         }
     }
 
diff --git a/crates/singlefs-harness/src/bin/first_transaction_on_device.rs b/crates/singlefs-harness/src/bin/first_transaction_on_device.rs
index 30a3984..f41d64b 100644
--- a/crates/singlefs-harness/src/bin/first_transaction_on_device.rs
+++ b/crates/singlefs-harness/src/bin/first_transaction_on_device.rs
@@ -714,7 +714,7 @@ where
                 | MountError::FileVersionWithoutAnyJournalRecord
                 | MountError::InstanceTableMalformed
                 | MountError::Acquisition(_)
-                | MountError::RaiseFloorSequencePublishFailed { .. }
+                | MountError::RaiseFloorSequencePublishFailed(_)
                 | MountError::RollbackTargetNotACandidate { .. }
                 | MountError::RollbackFloorAboveCeiling { .. }
                 | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
@@ -723,7 +723,7 @@ where
                 | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
                 | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
                 | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
-                | MountError::InstanceTableChainLongerThanOnePageUndecided { .. } => {
+                | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. } => {
                     vec![describe_run_failure("reopen_and_writable_mount", &cause)]
                 }
             };
@@ -1111,8 +1111,7 @@ fn main() {
     emitter.emit(&first_transaction_line);
     let mut every_window_matches_device = warm_up_matches && first_transaction_matches;
     emitter.emit(&format!(
-        "name=transaction policy_mismatches={} key_order_mismatches={} root_txg={} back_chain={}",
-        run.policy_mismatches,
+        "name=transaction key_order_mismatches={} root_txg={} back_chain={}",
         run.output.key_order_mismatches,
         run.output.root.checkpoint_txg.0,
         run.output.record.back_chain
@@ -1640,10 +1639,11 @@ mod tests {
     }
 
     use super::{
-        describe_first_transaction_path_failure, device_call_counts,
+        describe_failed_window, describe_first_transaction_path_failure, device_call_counts,
         publish_the_second_version_and_describe, publish_the_third_version_and_describe,
-        reopen_and_mount_writable, FailedRun,
+        reopen_and_mount_writable, FailedRun, WritableMountRun,
     };
+    use singlefs_core::mount::{raise_rollback_floor, MountError, Mounted, ShadowLedger};
     use singlefs_harness::device_log::{expected_device_events, DeviceEvent};
     use singlefs_harness::fault_injection::{FaultSchedule, InjectedFault};
     use singlefs_harness::scenario::{FirstTransactionPathStep, ScenarioPoint};
@@ -2023,6 +2023,89 @@ mod tests {
         }
     }
 
+    /// 抬 F 失败（增补 2 收口表第 58 行「`raise_rollback_floor` 同形」）：抬 F 的写入口是它自己开的、随错丢掉，
+    /// 已经落盘的那几次空发布与失败那一次已记的写随 `MountError::RaiseFloorSequencePublishFailed` 交回，照可写挂载那一段的判法
+    /// （`describe_failed_window`）与设备一层逐项相等。二进制的五个模式里没有抬 F，这一段只在这里走：`second-instance` 那条路走到
+    /// 可写挂载做完（实例 2，现行 txg 7），装上注入，抬到现行的 F（0）——推空发布直到两块盘上都有一条带这个 F 的根（txg 8、9）。
+    /// 注入摆在之后整池第 16 次写：第一次空发布 13 次写已经落盘（四个固定点单元与 journal 记录每盘一份、根槽一次、系统配置每盘一次），
+    /// 第二次的第 3 个写报错 ⇒ 失败账 2 次写，两样相加 15 次与设备一层逐项相等。
+    #[test]
+    fn failed_raise_of_the_rollback_floor_reports_every_publish_it_wrote_and_they_equal_the_device_layer_count(
+    ) {
+        let parameters = e142_parameters(512, 512);
+        let geometry = geometry_of(&parameters);
+        let stream = SharedStream::new();
+        let plan = SharedFaultPlan::unarmed(geometry);
+        let mut devices = counted_sparse_devices(&stream, &plan);
+        let run = run_first_transaction(&parameters, &mut devices, &stream, |_point, _devices| {})
+            .expect("第一个事务");
+        let mut allocator = allocator_rebuilt_from_the_records_of(&devices, &run.output);
+        publish_the_second_version(&parameters, &mut devices, &mut allocator, &run.output)
+            .expect("发布 B");
+        let mut clock = SegmentClock::start();
+        let WritableMountRun {
+            devices: mut mounted_devices,
+            mounted:
+                Mounted {
+                    allocator: mut mounted_allocator,
+                    current,
+                    ..
+                },
+            plan: mounted_plan,
+            ..
+        } = match reopen_and_mount_writable(
+            &parameters,
+            devices,
+            &stream,
+            &geometry,
+            SharedFaultPlan::unarmed(geometry),
+            &mut clock,
+            reopen_sparse_devices,
+        ) {
+            Ok(mounted) => mounted,
+            Err(failed) => panic!("可写挂载：{failed:?}"),
+        };
+        let mut current = current
+            .into_file_version()
+            .expect("发布 B 之后重开，现行那一版带文件");
+        assert_eq!(current.root.checkpoint_txg, CheckpointTxg(7));
+        let counts_before_raise = device_call_counts(&mounted_devices, &mounted_plan);
+        mounted_plan.arm(FaultSchedule::the_nth_call_across_the_pool(
+            InjectedFault::WriteFails,
+            16,
+        ));
+        let raised = raise_rollback_floor(
+            &parameters,
+            &mut mounted_devices,
+            &mut mounted_allocator,
+            &mut current,
+            CheckpointTxg(0),
+            ShadowLedger::On,
+        );
+        let Err(MountError::RaiseFloorSequencePublishFailed(failed)) = raised else {
+            panic!(
+                "注入的写错该让抬 F 那一串停在第二次空发布：{:?}",
+                raised.as_ref().err()
+            );
+        };
+        assert_eq!(
+            current.root.checkpoint_txg,
+            CheckpointTxg(8),
+            "调用方的现行版本已经是落盘的第一次空发布"
+        );
+        let (lines, _) = describe_failed_window(
+            "raise_rollback_floor",
+            &format!("{:?}", failed.cause),
+            &failed.writes_of_persisted_publishes,
+            &failed.writes_of_failed_publishes,
+            pool_writes_between(
+                &counts_before_raise,
+                &device_call_counts(&mounted_devices, &mounted_plan),
+            ),
+        );
+        assert_the_failed_window_is_reconciled(&lines, "raise_rollback_floor", &["13"], "2", "15");
+    }
+
     /// 宿主检查（`first_transaction_device_log_check`）拿录制流投到每块盘上的事件当「程序的信念」：每个写一件、FUA 写之后一个 FLUSH、
     /// 每道池屏障在每块盘上各一个 FLUSH（录制器把连续几道并成一道）。这个投法在发布 B、可写挂载、发布 C 三段上是否就是设备收到的，
     /// 拿录制器外面那一层（注入包装，与录制器不共享计数）数到的核：每块盘、每段，写的件数相等，FLUSH 件数 = 转发的屏障 + FUA 写。
diff --git a/crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs b/crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs
index d7e75b6..3aefab2 100644
--- a/crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs
+++ b/crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs
@@ -112,10 +112,9 @@ fn main() {
 
     let mut emitter = Emitter { emitted: 0 };
     emitter.emit(&format!(
-        "name=impl_config devices={DEVICE_COUNT} image_bytes={TEST_IMAGE_DEFAULT_BYTES} physical_block_size={PHYSICAL_BLOCK_SIZE_IN_BYTES} minimum_input_output_bytes={MINIMUM_INPUT_OUTPUT_BYTES} file_bytes={FIRST_FILE_BYTES} write_time_seconds={FIXED_WRITE_TIME_SECONDS} filesystem_identifier={} checkpoint_txg={FIRST_TRANSACTION_TXG} mkfs_operations={} policy_mismatches={}",
+        "name=impl_config devices={DEVICE_COUNT} image_bytes={TEST_IMAGE_DEFAULT_BYTES} physical_block_size={PHYSICAL_BLOCK_SIZE_IN_BYTES} minimum_input_output_bytes={MINIMUM_INPUT_OUTPUT_BYTES} file_bytes={FIRST_FILE_BYTES} write_time_seconds={FIXED_WRITE_TIME_SECONDS} filesystem_identifier={} checkpoint_txg={FIRST_TRANSACTION_TXG} mkfs_operations={}",
         hexadecimal_text(&E142_FILESYSTEM_IDENTIFIER),
-        run.mkfs_operation_count,
-        run.policy_mismatches
+        run.mkfs_operation_count
     ));
     for line in region_result_lines(&image) {
         emitter.emit(&line);
diff --git a/crates/singlefs-harness/src/fault_injection.rs b/crates/singlefs-harness/src/fault_injection.rs
index 855c94a..ef80217 100644
--- a/crates/singlefs-harness/src/fault_injection.rs
+++ b/crates/singlefs-harness/src/fault_injection.rs
@@ -1906,7 +1906,9 @@ fn the_fault_surfaced_as_an_error(
 ///
 /// **白名单之外的签名照报成新发现**（用户 2026-09-21 定的收严）：别的签名（I-3.9、I-5.4、I-9.14 这一类）意味着
 /// 丢一份内容之外还发生了别的事，那一格不默认豁免——那才可能是实现的缺口。
-/// 白名单少一组，被它罩住的那几次当场变成新发现、快档判红（`crates/mutations.tsv` 里钉着 `["I-3.1"]` 那一组）。
+/// 白名单少一组，被它罩住的那几次当场变成新发现。快档那 24 段里说谎的设备留下的不一致全落在前一组，`["I-3.1"]` 那一组一次都没有；
+/// 那一组的取样点是故障注入那个二进制里的 `a_swallowed_write_after_which_the_checker_flags_only_i_3_1_is_excused_as_what_a_lying_device_may_leave`
+/// （`crates/mutations.tsv` 里钉着把那一组去掉的变异）。
 const INCONSISTENCIES_A_LYING_DEVICE_MAY_LEAVE: &[&[&str]] =
     &[&["I-2.1", "I-4.8", "I-7.4"], &["I-3.1"]];
 
diff --git a/crates/singlefs-harness/src/history.rs b/crates/singlefs-harness/src/history.rs
index 7321220..415bc6f 100644
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -78,6 +78,10 @@ pub enum HistoryDeviceWidth {
     /// （`make_filesystem::allocator_after_make_filesystem`），每条会话在根环转过之后都按谓词回收，稳态占用约是根环里那 24 版的账
     /// （每版每盘约 10 槽），384 槽那一档一次落点拒绝都走不到了；256 槽走得到用户数据那一处的拒绝。
     UnitAreaOf256Slots,
+    /// 单元区 240 槽的小盘（journal 环同 384 槽那一档）：可写挂载自己那一串（写行与暖机）在这么窄的盘上拿不到落点，
+    /// 走得到取号之后才被落点拒绝的那一格（增补 2 收口表第 39 行那一族：取号之前的准入不算落点）；256 槽那一档的单元区墙取样点上
+    /// 一次都没走到过。
+    UnitAreaOf240Slots,
 }
 
 /// 小盘的单元区槽数（`HistoryDeviceWidth::UnitAreaOf384Slots`）：六个 64 槽的聚簇段。
@@ -86,6 +90,9 @@ const SMALL_DEVICE_UNIT_AREA_SLOTS: u64 = 384;
 /// 更小一档小盘的单元区槽数（`HistoryDeviceWidth::UnitAreaOf256Slots`）：四个 64 槽的聚簇段。
 const SMALLER_DEVICE_UNIT_AREA_SLOTS: u64 = 256;
 
+/// 再窄一档小盘的单元区槽数（`HistoryDeviceWidth::UnitAreaOf240Slots`）：三个整 64 槽的聚簇段加 48 槽的尾巴。
+const NARROWEST_DEVICE_UNIT_AREA_SLOTS: u64 = 240;
+
 /// 小盘上的 journal 环字节数：不超过设备容量的四分之一（设备约 790 MiB）。
 const SMALL_DEVICE_JOURNAL_RING_BYTES: u64 = 128 << 20;
 
@@ -101,6 +108,9 @@ impl HistoryDeviceWidth {
             HistoryDeviceWidth::UnitAreaOf256Slots => {
                 (UNIT_AREA_START_SLOT + SMALLER_DEVICE_UNIT_AREA_SLOTS) * SLOT_BYTES
             }
+            HistoryDeviceWidth::UnitAreaOf240Slots => {
+                (UNIT_AREA_START_SLOT + NARROWEST_DEVICE_UNIT_AREA_SLOTS) * SLOT_BYTES
+            }
         }
     }
 
@@ -110,7 +120,9 @@ impl HistoryDeviceWidth {
         let mut parameters = e142_parameters(512, 512);
         match self {
             HistoryDeviceWidth::FourGibibytes => {}
-            HistoryDeviceWidth::UnitAreaOf384Slots | HistoryDeviceWidth::UnitAreaOf256Slots => {
+            HistoryDeviceWidth::UnitAreaOf384Slots
+            | HistoryDeviceWidth::UnitAreaOf256Slots
+            | HistoryDeviceWidth::UnitAreaOf240Slots => {
                 parameters.geometry.journal_ring_bytes = SMALL_DEVICE_JOURNAL_RING_BYTES;
             }
         }
@@ -139,6 +151,9 @@ impl HistoryDeviceWidth {
             HistoryDeviceWidth::UnitAreaOf256Slots => {
                 "两块单元区 256 槽的小盘（journal 环 128 MiB）"
             }
+            HistoryDeviceWidth::UnitAreaOf240Slots => {
+                "两块单元区 240 槽的小盘（journal 环 128 MiB）"
+            }
         }
     }
 }
@@ -1925,17 +1940,17 @@ fn publish_error_member(error: &PublishError) -> String {
         PublishError::MappingEntryNarrowerThanItsFieldTable { .. } => {
             "MappingEntryNarrowerThanItsFieldTable"
         }
-        PublishError::ReleaseChecksumReadFailedWhoseHandlingIsUndecided { .. } => {
-            "ReleaseChecksumReadFailedWhoseHandlingIsUndecided"
-        }
+        PublishError::ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided {
+            ..
+        } => "ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided",
+        PublishError::ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported {
+            ..
+        } => "ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported",
         PublishError::ContentExceedsDataUnit { .. } => "ContentExceedsDataUnit",
         PublishError::ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided { .. } => {
             "ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided"
         }
         PublishError::InodeTreeWriteRefused(_) => "InodeTreeWriteRefused",
-        PublishError::MoreNamedUnitsThanOneJournalRecordHolds { .. } => {
-            "MoreNamedUnitsThanOneJournalRecordHolds"
-        }
         PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn { .. } => {
             "FirstFileVersionDoesNotFollowTheVersionItBuildsOn"
         }
@@ -2006,6 +2021,10 @@ fn recovery_failure_member(failure: &RecoveryFailure) -> String {
         RecoveryFailure::MappingStillUnreadable { .. } => {
             "RecoveryFailure::MappingStillUnreadable".to_string()
         }
+        RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
+            ..
+        } => "RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided"
+            .to_string(),
     }
 }
 
@@ -2028,13 +2047,11 @@ fn mount_error_member(error: &MountError) -> String {
                 publish_error_member(&failed.cause)
             )
         }
-        MountError::RaiseFloorSequencePublishFailed {
-            publishes_persisted,
-            cause,
-        } => {
+        MountError::RaiseFloorSequencePublishFailed(failed) => {
             return format!(
-                "MountError::RaiseFloorSequencePublishFailed(publishes_persisted = {publishes_persisted}, {})",
-                publish_error_member(cause)
+                "MountError::RaiseFloorSequencePublishFailed(publishes_persisted = {}, {})",
+                failed.writes_of_persisted_publishes.len(),
+                publish_error_member(&failed.cause)
             )
         }
         MountError::RollbackTargetNotACandidate { exclusion, .. } => {
@@ -2068,8 +2085,13 @@ fn mount_error_member(error: &MountError) -> String {
                 publish_error_member(cause)
             )
         }
-        MountError::InstanceTableChainLongerThanOnePageUndecided { .. } => {
-            "InstanceTableChainLongerThanOnePageUndecided"
+        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
+            refusal, ..
+        } => {
+            return format!(
+                "MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided({})",
+                placement_refusal_member(refusal)
+            )
         }
     };
     format!("MountError::{member}")
diff --git a/crates/singlefs-harness/src/model.rs b/crates/singlefs-harness/src/model.rs
index 65118b4..a97ec6f 100644
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -110,7 +110,31 @@ enum ModelPublishKind {
     ZeroUnit,
 }
 
+/// 装得下 `rows` 行要几片（D18（块里携带什么信息） 已定项 11：一片 370 条记录，链指针记录恒为一片的最后一条、数据行 369；
+/// 0 行也是一片）。模型按条款自己算，不调实现的那一份。
+#[must_use]
+pub fn instance_table_pages_for_rows(rows: usize) -> u64 {
+    let rows_per_page = INSTANCE_TABLE_PAGE_RECORDS - 1;
+    u64::try_from(rows)
+        .expect("行数装得进 u64")
+        .div_ceil(rows_per_page)
+        .max(1)
+}
+
 impl ModelPublishKind {
+    /// 这次发布重写不重写实例表：写行那两种重写整条链，别的照抄。
+    fn rewrites_the_instance_table(self) -> bool {
+        match self {
+            ModelPublishKind::RowsOnFileVersion | ModelPublishKind::RowsOnVersionWithoutFile => {
+                true
+            }
+            ModelPublishKind::FirstFileVersion
+            | ModelPublishKind::OverwriteFileVersion
+            | ModelPublishKind::EmptyOnFileVersion
+            | ModelPublishKind::ZeroUnit => false,
+        }
+    }
+
     /// 这次发布往分配记录树里加几条（每块盘一条）：重写的每个角色各一条。
     /// 零单元发布一个字节都不写，一条不加；树表 0 条的一版上写行那次**建起这一版自己的分配记录树**
     /// （C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案），实例表与那片节点各一条。
@@ -271,9 +295,6 @@ pub enum ModelRefusalReason {
     AccountingNodeWall,
     /// 单元区装不下（D28（挂载期承诺量） 已定项 1 的准入；模型答允许拒绝的区间）。
     UnitAreaWall,
-    /// 实例表这次之后要多于一片（D18（块里携带什么信息） 已定项 11：一片 370 条含链指针）：第二片在 bump 次序里怎么排、
-    /// 行怎么分片没有条款（D3（空间分配） 已定项 10 ⑤ 只写「实例表单元最前」），第一版不写第二片。
-    InstanceTableChainLongerThanOnePageUndecided,
     /// 回退目标不在根环里（D23（journal 的角色与格式） 已定项 14：候选集是根环里的根）。
     RollbackTargetNotInRing,
     /// 回退目标的 txg 低于 F_生效（D16（发布语义） 已定项 1「回退候选集」）。
@@ -302,9 +323,6 @@ impl ModelRefusalReason {
             ModelRefusalReason::AllocationRecordNodeWall => "分配记录树一个节点装不下",
             ModelRefusalReason::AccountingNodeWall => "记账树一个节点装不下",
             ModelRefusalReason::UnitAreaWall => "单元区装不下",
-            ModelRefusalReason::InstanceTableChainLongerThanOnePageUndecided => {
-                "实例表要多于一片（第二片怎么写没有条款）"
-            }
             ModelRefusalReason::RollbackTargetNotInRing => "回退目标不在根环里",
             ModelRefusalReason::RollbackTargetBelowEffectiveFloor => "回退目标低于 F_生效",
             ModelRefusalReason::RollbackTargetOnAbandonedTimeline => "回退目标在被抛弃的时间线上",
@@ -323,7 +341,6 @@ impl ModelRefusalReason {
             | ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn
             | ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile
             | ModelRefusalReason::AccountingNodeWall
-            | ModelRefusalReason::InstanceTableChainLongerThanOnePageUndecided
             | ModelRefusalReason::RollbackTargetNotInRing
             | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
             | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
@@ -834,7 +851,17 @@ impl IdealModel {
             role_written_at.insert(*role, checkpoint_txg);
             slots_written += role.span_in_slots();
         }
-        let allocation_records_added = kind.allocation_records_added_per_device() * device_count;
+        // 实例表链多于一片时第 1 片起每一片也是一个重写的单元：`rewritten_roles` 里实例表只记一个角色，多出来的几片在这里补上。
+        let instance_table_pages_after_the_first = if kind.rewrites_the_instance_table() {
+            instance_table_pages_for_rows(instance_table_rows.len()) - 1
+        } else {
+            0
+        };
+        slots_written +=
+            instance_table_pages_after_the_first * ModelUnitRole::InstanceTable.span_in_slots();
+        let allocation_records_added = (kind.allocation_records_added_per_device()
+            + instance_table_pages_after_the_first)
+            * device_count;
         let file = match new_file_content {
             Some(content) => Some(ModelFileVersion {
                 written_by: key,
@@ -1126,7 +1153,7 @@ impl IdealModel {
         let first_journal_counter = ModelJournalCounter(self.highest_journal_counter.0 + 1);
         // 新实例的根带的 F = 恢复后生效的 F（D16（发布语义） 已定项 1「生效」）：预想，跟收口表第 ② 行。
         let rollback_floor = self.effective_rollback_floor();
-        let mut required_refusals = BTreeSet::new();
+        let required_refusals = BTreeSet::new();
         let has_file = base.file.is_some();
         // 树表 0 条、而这一版的实例表已经不是 mkfs 那一片（上一次挂载在这一版上写过行）**此前是必拒的一格**：
         // 重建账时只剩根记录那两条指针，被换下的那一片成了空闲槽。C512（树表 0 条的一版上被换下的单元记在哪）
@@ -1134,14 +1161,8 @@ impl IdealModel {
         // 模型因此**一条都不列**：实现要是还在这一格上拒，对拍当场报「模型说该成、实现拒了」——
         // 那正是这条定案要盯住的回退面。`MountError::VersionWithoutFileNotWrittenByMakeFilesystem` 今天只剩
         // 「树表或实例表不是 mkfs 写的那一版、而这一版又没有自己的分配记录树」那种手造镜像走得到，随机历史里造不出来。
-        let rows_after =
-            u64::try_from(base.instance_table_rows.len() + rows_to_write.len()).expect("行数");
-        // 一片 370 条，链指针记录恒为一片的最后一条（D18（块里携带什么信息） 已定项 11）；这次之后要多于一片时，第二片怎么写没有条款，
-        // 第一版不写第二片。两臂都判：树表 0 条的一版上写行同样重写整张实例表。
-        if rows_after + 1 > INSTANCE_TABLE_PAGE_RECORDS {
-            required_refusals
-                .insert(ModelRefusalReason::InstanceTableChainLongerThanOnePageUndecided);
-        }
+        // 实例表多于一片不再拒（用户 2026-09-24 定尾片先、一片写满 369 行再开下一片）：写行那次发布整条链重写，
+        // 有几片就多几个实例表单元（`next_root` 按这次之后的行数现算片数）。
         let table_after: Rc<Vec<ModelInstanceRow>> = if rows_to_write.is_empty() {
             Rc::clone(&base.instance_table_rows)
         } else {
@@ -1374,7 +1395,6 @@ impl IdealModel {
             | ModelRefusalReason::FirstFileVersionDoesNotFollowTheVersionItBuildsOn
             | ModelRefusalReason::FirstFileVersionOnAVersionThatAlreadyHasAFile
             | ModelRefusalReason::AccountingNodeWall
-            | ModelRefusalReason::InstanceTableChainLongerThanOnePageUndecided
             | ModelRefusalReason::RollbackTargetNotInRing
             | ModelRefusalReason::RollbackTargetBelowEffectiveFloor
             | ModelRefusalReason::RollbackTargetOnAbandonedTimeline
@@ -1947,6 +1967,41 @@ mod tests {
         model
     }
 
+    /// 实例表多于一片（用户 2026-09-24 定尾片先、一片写满 369 行再开下一片）：可写挂载不再拒，写行那次发布整条链重写，
+    /// 链上每一片都是一个重写的单元、每盘各加一条分配记录。第一个文件之后号推到 370（等于连着 369 次取号之后崩溃），
+    /// 下一次可写挂载取 371、写 [1, 371) 共 370 行 ⇒ 两片：写行那次发布加 (两片 + 四个固定点单元) × 2 盘 = 12 条、占 2 × 2 + 4 = 8 槽。
+    #[test]
+    fn a_mount_that_writes_more_rows_than_one_page_holds_counts_every_page_of_the_instance_table_chain(
+    ) {
+        let mut model = two_device_model();
+        model.acquire_and_warm_up_in_the_make_filesystem_process();
+        let first = model
+            .answer_publish_first_file(&[3])
+            .expect("会话开着、现行 txg 2");
+        succeed(&mut model, &first);
+        model.close_session();
+        model.highest_acquired_instance = ModelInstanceGeneration(370);
+        let mount = model.answer_mount_writable();
+        assert!(mount.required_refusals.is_empty(), "{mount:?}");
+        let (instance, rows_written) = mount.expected_mount.clone().expect("挂载做成");
+        assert_eq!(
+            (instance, rows_written.len()),
+            (ModelInstanceGeneration(371), 370)
+        );
+        let row_publish = &mount.expected_roots[0];
+        assert_eq!(row_publish.instance_table_rows.len(), 370);
+        assert_eq!(
+            row_publish.allocation_records_added_by_its_publish, 12,
+            "两片实例表加四个固定点单元，每盘一条"
+        );
+        let slots_before = model.newest_root().occupied_slots_upper_bound_per_device;
+        assert_eq!(
+            row_publish.occupied_slots_upper_bound_per_device - slots_before,
+            8,
+            "两片实例表各 2 槽、四个固定点单元各 1 槽"
+        );
+    }
+
     /// 内容装不装得下按 32768 含头与预留位算（D4（校验和位置） 已定项 5）：正好装满不拒，多一个字节才拒。
     #[test]
     fn content_of_exactly_the_payload_capacity_is_accepted_and_one_byte_more_is_refused() {
diff --git a/crates/singlefs-harness/src/model_comparison.rs b/crates/singlefs-harness/src/model_comparison.rs
index e447312..0f0dca9 100644
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -11,7 +11,7 @@ use singlefs_core::allocator::PlacementRefusal;
 use singlefs_core::block_device::BlockDeviceError;
 use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
 use singlefs_core::mount::{
-    InstanceRow, MountError, Mounted, PublishAfterAcquisitionFailed, RollbackCandidateExclusion,
+    InstanceRow, MountError, Mounted, PublishSequenceFailed, RollbackCandidateExclusion,
 };
 use singlefs_core::recovery::{RecoveryOutcome, RecoveryReport};
 use singlefs_core::transaction::{
@@ -72,7 +72,10 @@ pub fn model_unit_role(unit: TransactionUnit) -> ModelUnitRole {
         TransactionUnit::AccountingTree => ModelUnitRole::AccountingTree,
         TransactionUnit::MappingTree => ModelUnitRole::MappingTree,
         TransactionUnit::TreeTable => ModelUnitRole::TreeTable,
-        TransactionUnit::InstanceTable => ModelUnitRole::InstanceTable,
+        // 模型把实例表链当一个角色：各片的落点与分配记录按这次之后的行数现算片数（`model::instance_table_pages_for_rows`）。
+        TransactionUnit::InstanceTable | TransactionUnit::InstanceTablePageAfterTheFirst(_) => {
+            ModelUnitRole::InstanceTable
+        }
     }
 }
 
@@ -183,13 +186,12 @@ pub fn refusal_reason_of_publish_error(error: &PublishError) -> ObservedRefusalR
         }
         // 释放判定路径的五种：上一版的映射或分配记录与上一版对不上、盘上那条指针的两条位置条目不同槽，健康的历史里不该出现。
         // extent 树要长内部节点那一条同理：随机历史一次都不调 `publish_sequential_write`，写的文件恒一个数据单元，它出现就是对不上。
-        // inode 树写入被拒与点名项装不下那两条同样：随机历史一次都不调 `publish_new_inodes`，
+        // inode 树写入被拒那一条同样：随机历史一次都不调 `publish_new_inodes`，
         // 而它跑的那几种发布每次最多改一片叶容器、重写的角色最多九个。
         // 映射节点装不下那一条同理：条目数 = 五个固定角色 + 叶容器数，随机历史里恒是 1 片叶 ⇒ 恒 6 条，
         // 离一个节点的 294 条差得远；它出现就是模型与实现对不上。
         PublishError::MappingEntriesExceedOneNode { .. }
         | PublishError::InodeTreeWriteRefused(_)
-        | PublishError::MoreNamedUnitsThanOneJournalRecordHolds { .. }
         | PublishError::ExtentTreeNeedsAnInternalNodeWhoseEntryFormatIsUndecided { .. }
         | PublishError::ReleaseNotInMapping { .. }
         | PublishError::ReleaseTargetNotAllocated { .. }
@@ -197,8 +199,12 @@ pub fn refusal_reason_of_publish_error(error: &PublishError) -> ObservedRefusalR
         | PublishError::ReleaseTargetLocationsOnDifferentSlots { .. }
         | PublishError::ReleaseSpanMismatch { .. }
         | PublishError::MappingEntryNarrowerThanItsFieldTable { .. }
-        // 释放之前读盘核校验和那一读没读到：健康的内存盘上读不会失败，出现就是对不上（读失败怎么办条款没定，模型里没有它的理由）。
-        | PublishError::ReleaseChecksumReadFailedWhoseHandlingIsUndecided { .. }
+        // 释放之前读盘核校验和时位置项指的盘不在池里、或只有一部分副本对不上：前者只有坏镜像、外来镜像上有，后者要盘上那一份坏了
+        // 或读出坏字节才走得到；健康的内存盘上都不该出现，模型里没有它们的理由（两格条款都没写）。
+        | PublishError::ReleaseChecksumLocationOnADeviceOutsideThePoolWhoseHandlingIsUndecided {
+            ..
+        }
+        | PublishError::ReleaseChecksumFailedOnSomeCopiesOnlyAsymmetricRecordsUnsupported { .. }
         // 第一个文件版本读不出那一版的树表、水位离 u64::MAX 不到八个号：健康的内存盘上都不该出现。
         | PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. }
         | PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_)
@@ -249,8 +255,8 @@ pub fn refusal_reason_of_block_device_error(_error: &BlockDeviceError) -> Observ
 #[must_use]
 pub fn refusal_reason_of_mount_error(error: &MountError) -> ObservedRefusalReason {
     match error {
-        MountError::Publish(PublishAfterAcquisitionFailed { cause, .. })
-        | MountError::RaiseFloorSequencePublishFailed { cause, .. }
+        MountError::Publish(PublishSequenceFailed { cause, .. })
+        | MountError::RaiseFloorSequencePublishFailed(PublishSequenceFailed { cause, .. })
         | MountError::RowPublishAdmissionRefusedBeforeAcquisition { cause, .. }
         | MountError::WarmUpAdmissionRefusedBeforeAcquisition { cause, .. } => {
             refusal_reason_of_publish_error(cause)
@@ -261,9 +267,10 @@ pub fn refusal_reason_of_mount_error(error: &MountError) -> ObservedRefusalReaso
         MountError::RollbackFloorAboveCeiling { .. } => {
             explained(ModelRefusalReason::FloorAboveCeiling)
         }
-        MountError::InstanceTableChainLongerThanOnePageUndecided { .. } => {
-            explained(ModelRefusalReason::InstanceTableChainLongerThanOnePageUndecided)
-        }
+        // 取号之后那一串自己的落点在取号之前就取不到：说的是分配器那一条原因（每块盘上都没有 = 单元区墙）。
+        MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
+            refusal, ..
+        } => refusal_reason_of_placement_refusal(refusal),
         // 树表 0 条、而实例表已经不是 mkfs 那一片：零故障走得到（写过行的那一版上再挂载一次），模型照代码今天的读法划进必须拒。
         MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. } => {
             explained(ModelRefusalReason::VersionWithoutFileNotWrittenByMakeFilesystem)
@@ -291,7 +298,7 @@ pub fn reported_ceiling_of_mount_error(error: &MountError) -> Option<ModelCheckp
         | MountError::InstanceTableMalformed
         | MountError::Acquisition(_)
         | MountError::Publish(_)
-        | MountError::RaiseFloorSequencePublishFailed { .. }
+        | MountError::RaiseFloorSequencePublishFailed(_)
         | MountError::RollbackTargetNotACandidate { .. }
         | MountError::VersionWithoutFileNotWrittenByMakeFilesystem { .. }
         | MountError::FormatTimeUnitLocationsOnDifferentSlots { .. }
@@ -299,7 +306,7 @@ pub fn reported_ceiling_of_mount_error(error: &MountError) -> Option<ModelCheckp
         | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
         | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
         | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
-        | MountError::InstanceTableChainLongerThanOnePageUndecided { .. } => None,
+        | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. } => None,
     }
 }
 
diff --git a/crates/singlefs-harness/src/scenario.rs b/crates/singlefs-harness/src/scenario.rs
index f77e1ba..ef60f83 100644
--- a/crates/singlefs-harness/src/scenario.rs
+++ b/crates/singlefs-harness/src/scenario.rs
@@ -58,7 +58,6 @@ pub struct FirstTransactionRun {
     pub mkfs_operation_count: usize,
     pub warm_up: WarmUpOutput,
     pub output: TransactionOutput,
-    pub policy_mismatches: u64,
 }
 
 /// 整条路上调用方被叫到的三处。
@@ -190,6 +189,5 @@ pub fn run_first_transaction<
         mkfs_operation_count,
         warm_up: warm_up_output,
         output,
-        policy_mismatches: allocator.policy_mismatches,
     })
 }
```

## 二、新文件全文

这批 diff 里没有 `new file mode` 行，18 个文件全部是对既有文件的修改，没有新文件需要附全文。
