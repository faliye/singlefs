# m4-validate-code-r2：被判的产品代码改动全文（2026-10-24）

三份补丁里落在 `crates/singlefs-core/src/` 的部分，原样摘自各补丁目录的 `crates.patch`（逐文件整块，不删 hunk）；对此刻主工作区 `git apply --check --reverse` 都退 0。

## 标签0写者照写（`~/.local/share/singlefs/work/m4-tag-zero-writer/patch/crates.patch`）

```diff
--- a/crates/singlefs-core/src/extent_tree.rs
+++ b/crates/singlefs-core/src/extent_tree.rs
@@ -404,6 +404,19 @@
         }
     }
 
+    /// 一个数据单元都没有的文件在上段叶里那一条（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K5 与第六节 S1：
+    /// 标签 0 配 size > 0 是整个文件都是洞；`research/prompts/m4-validate-code-r1-main-verification.md` 用户定案「丑：写者接得住这一形」）：
+    /// size > 0 写标签 0，size 0 不写条目——与 `transaction::publish_new_inodes` 建出来、一次都没写过内容的 inode 同一形，
+    /// 读者按「没有条目」收（`write_request_split::DataUnitsOfTheInode::NoneWritten`）。写者装上段叶、算上段形状（有条目的号集合）都经这里。
+    #[must_use]
+    pub const fn of_a_file_without_data_units(size_in_bytes: u64) -> Option<Self> {
+        if size_in_bytes > 0 {
+            Some(ExtentUpperLeafTarget::NoDataUnit)
+        } else {
+            None
+        }
+    }
+
     /// 这个标签声明的尾部：3、4、0 有洞，1、2 没有。
     #[must_use]
     pub const fn tail(self) -> TailOfAFile {
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -3906,11 +3906,16 @@
     }
 
     /// 这一版 extent 树上段里有条目的 inode 号（上段规范形的 S，判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md`
-    /// 第三节 M2）：第一个文件有数据单元时连 inode 1，加第一个文件之外每个有数据单元的文件。写者不写标签 0（D8（核心索引结构） 已定项 14），
-    /// 有条目的号就是有数据单元的号。
+    /// 第三节 M2）：第一个文件有数据单元、或一个单元都没有而 size > 0（上段叶标签 0，整个文件都是洞：
+    /// [`ExtentUpperLeafTarget::of_a_file_without_data_units`]）时连 inode 1，加第一个文件之外每个有数据单元的文件。
+    /// 与装上段叶写不写第一个文件那一条（`build_extent_tree`）同一条判法，形状与条目才对得上。
     #[must_use]
     pub fn inodes_with_an_extent_upper_leaf_entry(&self) -> BTreeSet<u64> {
-        let first_file = (!self.data_pointers.is_empty()).then_some(FIRST_INODE_NUMBER);
+        let first_file_has_an_entry = !self.data_pointers.is_empty()
+            || self.inode_record.is_some_and(|record| {
+                ExtentUpperLeafTarget::of_a_file_without_data_units(record.size).is_some()
+            });
+        let first_file = first_file_has_an_entry.then_some(FIRST_INODE_NUMBER);
         first_file
             .into_iter()
             .chain(
@@ -11905,8 +11910,10 @@
 /// 按计划装 extent 树这一版的全部节点（D8（核心索引结构） 已定项 14 的两段）：重写的节点按 bump 次序逐个装、发出生序号、取这次的落点——
 /// 下段先叶后根（叶装它罩的那几个单元的 extent 叶记录，内部节点装「孩子那一段的起点 key + 孩子这一版的指针」；第一个文件的下段在前，
 /// 第一个文件之外各文件的下段按 inode 号接着），再上段先叶后根（重写的一片叶装它罩的那几个 inode 号上每个有数据单元的文件那一条叶条目：
-/// 一个数据单元就内联它的数据指针（标签 2），多个就放下段根这一版的指针（标签 1）；第一个文件尾部有洞的写标签 3、4。条目都取这一版内存里的指针）。
+/// 一个数据单元就内联它的数据指针（标签 2），多个就放下段根这一版的指针（标签 1）；第一个文件尾部有洞的写标签 3、4，一个单元都没有而
+/// size > 0 的写标签 0。条目都取这一版内存里的指针）。
 /// 照抄的节点字节、落点、指针全取上一版同一个位置上的。`data_pointers` 是第一个文件这一版每个数据单元的指针（第 i 项是单元 i），
+/// `size_of_the_first_file_after_this_publish` 是这一版第一个文件那条记录的 size（这一版没有那条记录时为 `None`：不写它那一条），
 /// `data_pointers_of_the_files_other_than_the_first` 是第一个文件之外各文件这一版的数据指针（与 `files_other_than_the_first.files` 同序）。
 ///
 /// # Panics
@@ -11914,7 +11921,7 @@
 /// （角色清单按同一份计划排、上一版的字节是这个进程装的或重建时核过的、两份清单由 `publish_admitted` 按同一份计划拼）。
 #[allow(
     clippy::too_many_arguments,
-    reason = "参数各是一样东西：计划、第一个文件之外各文件的计划、上一版、上一版按身份的两份索引、第一个文件的数据指针、它尾部有没有洞、别的文件的数据指针、树号、身份、落点、出生序号"
+    reason = "参数各是一样东西：计划、第一个文件之外各文件的计划、上一版、上一版按身份的两份索引、第一个文件的数据指针、它这一版的 size、别的文件的数据指针、树号、身份、落点、出生序号"
 )]
 fn build_extent_tree(
     plan: &ExtentTreePlan,
@@ -11927,7 +11934,7 @@
         &FirstEntryOfEachIdentity<'_, InodeNumber, FileOfAnInodeOtherThanTheFirst>,
     >,
     data_pointers: &[DataPointer],
-    tail_of_the_first_file: TailOfAFile,
+    size_of_the_first_file_after_this_publish: Option<u64>,
     data_pointers_of_the_files_other_than_the_first: &[(InodeNumber, Vec<DataPointer>)],
     tree: TreeIdentifier,
     context: &MultiLevelTreeBuildContext<'_>,
@@ -11941,6 +11948,10 @@
         instance: context.instance,
     };
     let data_units = u64::try_from(data_pointers.len()).expect("单元数");
+    let tail_of_the_first_file = size_of_the_first_file_after_this_publish
+        .map_or(TailOfAFile::NoHoleAfterTheLastDataUnit, |size_in_bytes| {
+            TailOfAFile::of_data_units_and_size(data_units, size_in_bytes)
+        });
     let mut units: Vec<PublishedUnit> = Vec::new();
     let mut birth_sequences_of_rewritten_nodes = BTreeMap::new();
     let mut rewritten_node = |role: TransactionUnit,
@@ -12157,7 +12168,9 @@
             .map(|previous_root_level| role_of_extent_upper_node(*position, previous_root_level));
         let pointer = match origin {
             ExtentTreeNodeOrigin::RewrittenThisPublish if position.level == 0 => {
-                // 第一个文件这一版的那条叶条目：下段有根就指下段根，只有一个单元就内联它；一个单元都不剩（unlink 最后一批）就不写这一条，
+                // 第一个文件这一版的那条叶条目：下段有根就指下段根，只有一个单元就内联它；一个单元都没有时按记录的 size 取
+                // （`ExtentUpperLeafTarget::of_a_file_without_data_units`）：size > 0 是整个文件都是洞，照写标签 0（判决 m4-trunc-r1 K5、S1，
+                // m4-validate-code-r1 用户定案「丑」：重写这片叶的发布不丢这一条）；记录没了（unlink 最后一批）或 size 0 就不写这一条，
                 // 只剩它一条时这片叶成了零条目根（D8（核心索引结构） 已定项 16 第 5 条）。
                 // 最后一个单元之后到 size 是洞的换成标签 3、4（判决 m4-trunc-r1 K5，`ExtentUpperLeafTarget::with_the_tail`）。
                 let target = match (lower_nodes.last(), data_pointers.first()) {
@@ -12167,7 +12180,8 @@
                     (None, Some(only_data_pointer)) => {
                         Some(ExtentUpperLeafTarget::InlineDataUnit(*only_data_pointer))
                     }
-                    (None, None) => None,
+                    (None, None) => size_of_the_first_file_after_this_publish
+                        .and_then(ExtentUpperLeafTarget::of_a_file_without_data_units),
                 }
                 .map(|target| target.with_the_tail(tail_of_the_first_file));
                 // 这片叶罩的 inode 号上第一个文件之外的文件，条目照这一版内存里的指针装（照抄的文件的指针整理之后已换成映射位置，
@@ -13335,15 +13349,7 @@
         units_of_the_previous_version.as_ref(),
         files_of_the_previous_version.as_ref(),
         &data_pointers,
-        record_of_the_first_file_after_this_publish.map_or(
-            TailOfAFile::NoHoleAfterTheLastDataUnit,
-            |record| {
-                TailOfAFile::of_data_units_and_size(
-                    u64::try_from(data_pointers.len()).expect("单元数"),
-                    record.size,
-                )
-            },
-        ),
+        record_of_the_first_file_after_this_publish.map(|record| record.size),
         &data_pointers_of_the_files_other_than_the_first,
         trees.extent,
         &multi_level_tree_context,
@@ -15214,7 +15220,7 @@
                     None,
                     None,
                     &units.data_pointers,
-                    TailOfAFile::NoHoleAfterTheLastDataUnit,
+                    Some(u64::try_from(content.len()).expect("内容长度")),
                     &[],
                     checkpoint.trees.extent,
                     &context,
--- a/crates/singlefs-core/src/truncate.rs
+++ b/crates/singlefs-core/src/truncate.rs
@@ -130,20 +130,15 @@
 }
 
 /// 变长那次要不要重写尾单元（K5）：最后一个存在的单元声明长度小于 min(N − 起点, 净荷) 时补零重写到那个长度；之后到 N 是洞、不写单元。
-///
-/// # Panics
-/// 一个单元都不存在：第一个文件至少一个数据单元（长度 0 的内容也写一个声明长度 0 的单元）。
+/// 一个单元都不存在（上段叶标签 0：整个文件都是洞，K5、S1）时没有单元可补：不重写、不加单元，变长只改记录、标签留 0
+/// （判决 `research/prompts/m4-validate-code-r1-main-verification.md` 用户定案「丑：写者接得住这一形」）。
 #[must_use]
 pub fn tail_data_unit_rewrite_of_an_extension(
     data_units_existing: u64,
     size_before_in_bytes: u64,
     target_size_in_bytes: u64,
 ) -> Option<TailDataUnitRewrite> {
-    let unit = DataUnitIndexInFile(
-        data_units_existing
-            .checked_sub(1)
-            .expect("第一个文件至少一个数据单元"),
-    );
+    let unit = DataUnitIndexInFile(data_units_existing.checked_sub(1)?);
     let declared_length_in_bytes = declared_length_within_a_size(unit, target_size_in_bytes);
     (declared_length_within_a_size(unit, size_before_in_bytes) < declared_length_in_bytes)
         .then_some(TailDataUnitRewrite {
@@ -222,8 +217,11 @@
 /// 留下的单元数正在叶边界上（144 的倍数）时也一样；被截掉那一段里的节点换下。只剩一个单元时不建下段（内联）。上段这个文件那一串
 /// 在单元数变了、有单元重写、或上段叶条目的标签变了（尾部有没有洞）时重写，否则整棵照抄、什么都不换下。
 ///
+/// 之前一个单元都没有（上段叶标签 0：整个文件都是洞，K5、S1）的，之后也是 0 个：截短、N = S、变长都不加单元、不重写单元，
+/// 标签留 0、整棵照抄（判决 `research/prompts/m4-validate-code-r1-main-verification.md` 用户定案「丑」）。
+///
 /// # Panics
-/// 这次之后的单元数是 0 或多于之前的：截断留至少一个单元、变长不加单元。
+/// 这次之后的单元数多于之前的，或之前有单元而之后是 0 个：截断留至少一个单元、变长不加单元。
 #[must_use]
 pub fn plan_the_extent_tree_after_a_size_change(
     previous: &ExtentTreeVersion,
@@ -237,9 +235,10 @@
         tail_before,
         tail_after,
     } = change;
+    let keeps_at_least_one_unit_or_had_none = data_units_after >= 1 || data_units_before == 0;
     assert!(
-        (1..=data_units_before).contains(&data_units_after),
-        "截断留至少一个单元、变长不加单元"
+        data_units_after <= data_units_before && keeps_at_least_one_unit_or_had_none,
+        "截断留至少一个单元（一个单元都没有的整文件洞照旧 0 个）、变长不加单元"
     );
     let covers_the_rewritten_unit = |position: &ExtentLowerNodePosition| {
         let (first_data_unit, last_data_unit) = position.data_unit_range();
```

## 隔离的盘上记号（`~/.local/share/singlefs/work/m4-quarantine-mark-writer/patch/crates.patch`）

```diff
--- a/crates/singlefs-core/src/allocation_record_tree.rs
+++ b/crates/singlefs-core/src/allocation_record_tree.rs
@@ -25,7 +25,9 @@ use singlefs_format::{
 use crate::address::{
     CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier,
 };
-use crate::allocator::{absolute_slot_count_of_device, AllocationRecord, PoolAllocator};
+use crate::allocator::{
+    absolute_slot_count_of_device, AllocationRecord, AllocationRecordEntryRefusal, PoolAllocator,
+};
 use crate::bytes::{ByteReader, ByteWriter};
 use crate::pointer::{
     BirthSequence, NodePointer, PointerHeadFieldGroup, PointerHeadFieldsJudgement,
@@ -725,13 +727,21 @@ impl AllocationRecordTreeReading<'_> {
         if header.level == 0 {
             let first_new_record = records.len();
             for entry in &header.entries {
-                let record = AllocationRecord::parse(entry).ok_or(
-                    RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
+                // 跨度段的隔离记号读成记号、跨度取低位（D19（块指针的结构与宽度预算） 已定项 5），下面的落叶判按去掉记号的跨度判。
+                let record = AllocationRecord::parse(entry).map_err(|refusal| match refusal {
+                    AllocationRecordEntryRefusal::EntryWidthDiffersFromTheFieldTable {
+                        entry_bytes,
+                    } => RecoveryFailure::EntryWidthDiffersFromItsFieldTable {
                         what: "分配记录",
-                        entry_bytes: entry.len(),
+                        entry_bytes,
                         field_table_bytes: usize::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
                     },
-                )?;
+                    AllocationRecordEntryRefusal::QuarantineMarkOnAReleasedRecord => {
+                        RecoveryFailure::UnitMalformed {
+                            what: ALLOCATION_RECORD_WITH_THE_QUARANTINE_MARK_ON_A_RELEASED_RECORD,
+                        }
+                    }
+                })?;
                 if self.geometry.leaf_of_slot(record.device, record.slot) != node
                     || !self.geometry.record_fits_in_its_leaf(&record)
                 {
@@ -847,6 +857,10 @@ pub fn read_allocation_record_tree(
     Ok(AllocationRecordTreeReadFromDisk { nodes, records })
 }
 
+/// 分配记录树叶里一条记录的跨度段隔离记号与已释放标志同时置（0x4000 与 0x8000，D19（块指针的结构与宽度预算） 已定项 5
+/// 「带记号的记录必须未释放」）时报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句：判那片叶损坏，整棵树读不回。
+pub const ALLOCATION_RECORD_WITH_THE_QUARANTINE_MARK_ON_A_RELEASED_RECORD: &str =
+    "分配记录树叶里一条记录的跨度段隔离记号与已释放标志同时置";
 /// 分配记录树内部条目里的子指针头部 extent 偏移非 0 时报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句
 /// （C581（extent 偏移非 0 没有判据）：判该指针所在的结构损坏）。
 pub const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO: &str =
@@ -917,6 +931,7 @@ mod tests {
             span_slots,
             generation: CheckpointTxg(1),
             is_released: false,
+            quarantine_mark: crate::allocator::AllocationRecordQuarantineMark::NotQuarantined,
         }
     }
 
@@ -993,6 +1008,28 @@ mod tests {
         );
     }
 
+    /// 读回时的落叶判按去掉隔离记号的跨度判（D19（块指针的结构与宽度预算） 已定项 5）：叶罩槽 [49532, 50344)，跨度段 0x4002 起在 50342、
+    /// 0x4001 起在 50343 的两条记录解出跨度 2、1 且带记号，末槽 50343 都不越过叶的末槽；记号当成跨度读的话末槽越到一万六千多槽之外。
+    #[test]
+    fn records_with_the_quarantine_mark_fit_in_their_leaf_by_the_span_in_the_low_bits() {
+        let geometry = geometry_of_two_four_gibibyte_devices();
+        for (slot, span_field, span_slots) in [(50342u64, 0x4002u16, 2u16), (50343, 0x4001, 1)] {
+            let mut bytes = record(0, slot, span_slots).to_bytes();
+            bytes[10..12].copy_from_slice(&span_field.to_le_bytes());
+            let parsed = AllocationRecord::parse(&bytes).expect("带隔离记号、未释放的记录解得开");
+            assert_eq!(parsed.span_slots, span_slots, "跨度取低位");
+            assert_eq!(
+                parsed.quarantine_mark,
+                crate::allocator::AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
+                "跨度段次高位读成隔离记号"
+            );
+            assert!(
+                geometry.record_fits_in_its_leaf(&parsed),
+                "带记号的记录（槽 {slot}、跨度段 {span_field:#06x}）落在它所在的叶里"
+            );
+        }
+    }
+
     /// 一个叶的记录变了：那片叶、它的父节点与根变，同一块盘上另一片叶与另一块盘上的整条路径不变。
     #[test]
     fn changed_record_changes_its_leaf_and_every_ancestor_only() {
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -32,7 +32,33 @@ use crate::unlink::DeletionSwitches;
 /// 跨度段的最高位借作已释放标志（D3（空间分配） 已定项 7 / 已定项 11）：0 = 仍分配、1 = 已释放，不占表达跨度值的位。
 pub const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
 
-/// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：key (设备 4, 槽号 6) + value (跨度 2 含已释放标志, 分配代或释放代 8)。
+/// 跨度段的次高位借作隔离记号（D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」，落点甲；D3（空间分配） 已定项 7 跨度段那一行）：
+/// 写者隔离那次发布置上，回退复活与释放时清掉；不占表达跨度值的位。与已释放标志同时置的记录是坏的（[`AllocationRecordEntryRefusal`]）。
+pub const ALLOCATION_RECORD_QUARANTINE_MARK: u16 = 0x4000;
+
+/// 跨度段里表达跨度值的那几位：去掉已释放标志与隔离记号。
+const ALLOCATION_RECORD_SPAN_VALUE_BITS: u16 =
+    !(ALLOCATION_RECORD_RELEASED_FLAG | ALLOCATION_RECORD_QUARANTINE_MARK);
+
+/// 一条分配记录跨度段次高位的读法（D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub enum AllocationRecordQuarantineMark {
+    /// 次高位 0：不是隔离记录。已释放的记录恒取这一个（释放隔离记录时记号随之清掉）。
+    NotQuarantined,
+    /// 次高位 1：释放之前读盘核出这个单元有一份对不上，隔离那次发布在每块盘上那条记录里置上；记录仍分配、分配代不变。
+    QuarantinedAfterTheReleaseChecksumCheck,
+}
+
+/// 盘上 20 字节解不成一条分配记录的样子。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum AllocationRecordEntryRefusal {
+    /// 条目宽不等于字段表的 20（窄了切不出整条，宽了同样不收：I-1.10、C867）。
+    EntryWidthDiffersFromTheFieldTable { entry_bytes: usize },
+    /// 跨度段的隔离记号与已释放标志同时置（0x4000 与 0x8000，D19（块指针的结构与宽度预算） 已定项 5「带记号的记录必须未释放」）。
+    QuarantineMarkOnAReleasedRecord,
+}
+
+/// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：key (设备 4, 槽号 6) + value (跨度 2 含已释放标志与隔离记号, 分配代或释放代 8)。
 /// 释放时条目不删、不点删：改写成已释放 + 释放代，留到该落点被重新分配时覆盖（覆盖是步 5 回收接上时的事：
 /// 回收过的落点被再分配时那条记录改写、不追加，槽号在每盘仍唯一；新落点罩住的别的已回收记录随这次分配删掉，同一块盘上的记录
 /// 罩住的槽互不相交（I-5.4（分配记录罩住的槽互不相交））；回收要 F_生效 抬到释放代之上，见 `reclaim_released_up_to`）。
@@ -40,26 +66,42 @@ pub const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
 pub struct AllocationRecord {
     pub device: DeviceIdentity,
     pub slot: SlotNumber,
-    /// 纯跨度，不含标志位。
+    /// 纯跨度，不含已释放标志与隔离记号。
     pub span_slots: u16,
     /// 仍分配时是分配代；已释放时是释放代。
     pub generation: CheckpointTxg,
     pub is_released: bool,
+    /// 已释放的记录恒是 [`AllocationRecordQuarantineMark::NotQuarantined`]（[`Self::to_bytes`] 断言）。
+    pub quarantine_mark: AllocationRecordQuarantineMark,
 }
 
 impl AllocationRecord {
+    /// # Panics
+    /// 跨度值占到已释放标志或隔离记号那一位；已释放的记录带着隔离记号（释放隔离记录时记号随之清掉，
+    /// [`PoolAllocator::release_leaving_the_record_allocated_on`]）。
     #[must_use]
     pub fn to_bytes(&self) -> Vec<u8> {
         assert_eq!(
-            self.span_slots & ALLOCATION_RECORD_RELEASED_FLAG,
+            self.span_slots & !ALLOCATION_RECORD_SPAN_VALUE_BITS,
             0,
-            "跨度值不许占到标志位"
+            "跨度值不许占到已释放标志与隔离记号那两位"
         );
-        let span_field = if self.is_released {
-            self.span_slots | ALLOCATION_RECORD_RELEASED_FLAG
+        let released_flag = if self.is_released {
+            ALLOCATION_RECORD_RELEASED_FLAG
         } else {
-            self.span_slots
+            0
         };
+        let quarantine_mark = match self.quarantine_mark {
+            AllocationRecordQuarantineMark::NotQuarantined => 0,
+            AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck => {
+                assert!(
+                    !self.is_released,
+                    "已释放的记录不带隔离记号：释放隔离记录时记号随之清掉（D19 已定项 5）"
+                );
+                ALLOCATION_RECORD_QUARANTINE_MARK
+            }
+        };
+        let span_field = self.span_slots | released_flag | quarantine_mark;
         let mut writer = ByteWriter::new(20);
         writer.put_u32(self.device.0);
         writer.put_six_byte_unsigned(self.slot.0);
@@ -74,24 +116,41 @@ impl AllocationRecord {
         (self.device.0, self.slot.0)
     }
     /// 分配记录 20 的读者。条目宽是索引节点头里的一个盘上字段（`parse_index_node` 只判了它 ≥ key 宽 10），
-    /// 不等于 20 时返回 `None`：这里是盘上字节进字段表的边界，往里就按 20 信它。窄于 20 的切不出整条记录；
+    /// 不等于 20 时拒：这里是盘上字节进字段表的边界，往里就按 20 信它。窄于 20 的切不出整条记录；
     /// 宽于 20 的同样不收（I-1.10（码 2 条目宽等于字段表宽） 写的是「等于」，用户 2026-10-18 定「宽于字段表也拒」，欠账 C867）。
-    #[must_use]
-    pub fn parse(bytes: &[u8]) -> Option<Self> {
+    /// 跨度段次高位 0x4000 读成隔离记号、跨度取去掉两个标志位之后的值（D19（块指针的结构与宽度预算） 已定项 5）。
+    ///
+    /// # Errors
+    /// 条目宽不是 20 ⇒ [`AllocationRecordEntryRefusal::EntryWidthDiffersFromTheFieldTable`]；
+    /// 隔离记号与已释放标志同时置 ⇒ [`AllocationRecordEntryRefusal::QuarantineMarkOnAReleasedRecord`]。
+    pub fn parse(bytes: &[u8]) -> Result<Self, AllocationRecordEntryRefusal> {
         if bytes.len() != usize::try_from(ALLOCATION_RECORD_BYTES).expect("20") {
-            return None;
+            return Err(
+                AllocationRecordEntryRefusal::EntryWidthDiffersFromTheFieldTable {
+                    entry_bytes: bytes.len(),
+                },
+            );
         }
         let mut reader = crate::bytes::ByteReader::at(bytes, 0);
         let device = DeviceIdentity(reader.get_u32());
         let slot = SlotNumber(reader.get_six_byte_unsigned());
         let span_field = reader.get_u16();
         let generation = CheckpointTxg(reader.get_u64());
-        Some(Self {
+        let is_released = span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0;
+        let quarantine_mark = if span_field & ALLOCATION_RECORD_QUARANTINE_MARK == 0 {
+            AllocationRecordQuarantineMark::NotQuarantined
+        } else if is_released {
+            return Err(AllocationRecordEntryRefusal::QuarantineMarkOnAReleasedRecord);
+        } else {
+            AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck
+        };
+        Ok(Self {
             device,
             slot,
-            span_slots: span_field & !ALLOCATION_RECORD_RELEASED_FLAG,
+            span_slots: span_field & ALLOCATION_RECORD_SPAN_VALUE_BITS,
             generation,
-            is_released: span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0,
+            is_released,
+            quarantine_mark,
         })
     }
 }
@@ -2087,6 +2146,7 @@ impl PoolAllocator {
                         span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),
                         generation,
                         is_released: false,
+                        quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
                     });
                 }
             }
@@ -2102,10 +2162,12 @@ impl PoolAllocator {
         self.release_leaving_the_record_allocated_on(placement, release_generation, &[]);
     }
 
-    /// 同 [`Self::release`]，只是 `devices_whose_record_stays_allocated` 那几块盘上的那条分配记录一个字节都不动、留在「已分配」：
-    /// 释放之前按位置项读盘核校验和、核出对不上（读不出也算）的那一份（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-24 定案）——
-    /// 那一份的槽不还回空闲池；记录落盘即跨重挂，重挂之后从分配记录树重建的分配器照样认它已分配，准入里照已分配算、不另进式子。
-    /// 逐盘各算：另一块盘上核得上的那一份照常释放。
+    /// 同 [`Self::release`]，只是 `devices_whose_record_stays_allocated` 那几块盘上的那条分配记录留在「已分配」、分配代不变，
+    /// 跨度段置上隔离记号：释放之前按位置项读盘核校验和、核出对不上（读不出也算）的那个单元（D19（块指针的结构与宽度预算） 已定项 5，
+    /// 用户 2026-09-24 定案；记号是同一项「隔离的盘上记号」，用户 2026-10-22 定落点甲）——那一份的槽不还回空闲池；记录与这次发布的别的改动
+    /// 一起落盘，重挂之后从分配记录树重建的分配器照样认它已分配、带着记号，准入里照已分配算、不另进式子。
+    /// 逐盘各算：不在 `devices_whose_record_stays_allocated` 里的盘上那一份照常释放；释放的记录带着隔离记号时记号随之清掉
+    /// （「释放隔离记录时改写成已释放，0x4000 随之清掉」）。
     pub fn release_leaving_the_record_allocated_on(
         &mut self,
         placement: Placement,
@@ -2113,9 +2175,6 @@ impl PoolAllocator {
         devices_whose_record_stays_allocated: &[DeviceIdentity],
     ) {
         for device in &mut self.devices {
-            if devices_whose_record_stays_allocated.contains(&device.device) {
-                continue;
-            }
             let record = self
                 .records
                 .iter_mut()
@@ -2130,7 +2189,13 @@ impl PoolAllocator {
                 placement.span,
                 "释放的跨度与分配记录不符：这块盘的记录跨度由 transaction::placement_registered_unreleased_on_every_device 逐盘判过"
             );
+            if devices_whose_record_stays_allocated.contains(&device.device) {
+                record.quarantine_mark =
+                    AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck;
+                continue;
+            }
             record.is_released = true;
+            record.quarantine_mark = AllocationRecordQuarantineMark::NotQuarantined;
             record.generation = release_generation;
             device.mark_released(placement.slot, placement.span);
         }
@@ -2188,6 +2253,36 @@ impl PoolAllocator {
         }
     }
 
+    /// 管理员回退那一次清掉隔离记号（D19（块指针的结构与宽度预算） 已定项 5「回退复活那一次，同一次发布清掉两块盘上的 0x4000」）：
+    /// 回退目标引用的单元在 `device` 上那条记录仍分配、带着记号，改成不带记号；分配代与位图都不动（隔离的槽本来就占着）。
+    ///
+    /// # Panics
+    /// 这块盘上没有起点是这个槽的记录、它已释放、不带记号或跨度对不上：调用方在任何写之前逐盘判过
+    /// （`mount::roll_back_by_a_forward_publish`）。
+    pub fn clear_the_quarantine_mark_of_a_record_the_rollback_target_references(
+        &mut self,
+        device: DeviceIdentity,
+        placement: Placement,
+    ) {
+        let record = self
+            .records
+            .iter_mut()
+            .find(|record| record.device == device && record.slot == placement.slot)
+            .expect("清记号的落点在这块盘上有记录：回退在任何写之前逐盘判过");
+        assert!(
+            !record.is_released
+                && record.quarantine_mark
+                    == AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
+            "清记号的是仍分配、带着隔离记号的记录：回退在任何写之前逐盘判过"
+        );
+        assert_eq!(
+            u64::from(record.span_slots),
+            placement.span,
+            "清记号的跨度与记录相同：回退在任何写之前逐盘判过"
+        );
+        record.quarantine_mark = AllocationRecordQuarantineMark::NotQuarantined;
+    }
+
     /// 用户数据：见 `try_allocate_user_data`；拒绝的原因在这里丢掉，只给不看原因的调用方（测试里「一定分得到」的那几处）。
     /// 发布路径调 `try_allocate_user_data`，按原因报（`PublishError::PlacementRefused`）。
     pub fn allocate_user_data(&mut self, generation: CheckpointTxg) -> Option<Placement> {
@@ -3197,10 +3292,14 @@ mod tests {
             assert_eq!(record.generation, CheckpointTxg(4), "释放代");
             assert_eq!(record.span_slots, 2, "跨度值不含标志位");
             let parsed = AllocationRecord::parse(&record.to_bytes());
-            assert_eq!(parsed, Some(*record), "标志位进跨度段最高位再读回来");
+            assert_eq!(parsed, Ok(*record), "标志位进跨度段最高位再读回来");
             assert_eq!(
                 AllocationRecord::parse(&record.to_bytes()[..19]),
-                None,
+                Err(
+                    AllocationRecordEntryRefusal::EntryWidthDiffersFromTheFieldTable {
+                        entry_bytes: 19
+                    }
+                ),
                 "窄于字段表的 20 字节一律不解（条目宽是盘上的值）"
             );
             assert_eq!(
@@ -3217,6 +3316,148 @@ mod tests {
         );
     }
 
+    /// 一条分配记录的 20 字节，跨度段换成 `span_field`（别的段照一条盘 1、槽 50180、分配代 3 的记录）。
+    fn allocation_record_bytes_with_the_span_field(span_field: u16) -> Vec<u8> {
+        let mut bytes = AllocationRecord {
+            device: DeviceIdentity(1),
+            slot: SlotNumber(50180),
+            span_slots: 2,
+            generation: CheckpointTxg(3),
+            is_released: false,
+            quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
+        }
+        .to_bytes();
+        bytes[10..12].copy_from_slice(&span_field.to_le_bytes());
+        bytes
+    }
+
+    /// 跨度段次高位 0x4000 是隔离记号（D19 已定项 5「隔离的盘上记号」）：0x4001、0x4002 读成跨度 1、2 且带记号，
+    /// 写回去逐字节相同；0x4000 与 0x8000 同时置（0xC001）拒。
+    #[test]
+    fn span_field_with_the_quarantine_mark_parses_to_the_span_in_the_low_bits_and_the_mark() {
+        for span_slots in [1u16, 2] {
+            let bytes = allocation_record_bytes_with_the_span_field(
+                ALLOCATION_RECORD_QUARANTINE_MARK | span_slots,
+            );
+            let parsed = AllocationRecord::parse(&bytes).expect("带隔离记号、未释放的记录解得开");
+            assert_eq!(parsed.span_slots, span_slots, "跨度取去掉记号之后的低位");
+            assert_eq!(
+                parsed.quarantine_mark,
+                AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
+                "0x4000 读成隔离记号"
+            );
+            assert!(!parsed.is_released, "带记号的记录仍分配");
+            assert_eq!(parsed.generation, CheckpointTxg(3), "分配代照读");
+            assert_eq!(parsed.to_bytes(), bytes, "记号写回跨度段次高位");
+        }
+        assert_eq!(
+            AllocationRecord::parse(&allocation_record_bytes_with_the_span_field(
+                ALLOCATION_RECORD_RELEASED_FLAG | ALLOCATION_RECORD_QUARANTINE_MARK | 1
+            )),
+            Err(AllocationRecordEntryRefusal::QuarantineMarkOnAReleasedRecord),
+            "0xC001：隔离记号与已释放标志同时置，判坏"
+        );
+    }
+
+    /// 隔离（`release_leaving_the_record_allocated_on` 点名两块盘）：两块盘上那条记录仍分配、分配代不变、带隔离记号，
+    /// 落盘的跨度段是 0x4002；槽不进 defer、仍占着。
+    #[test]
+    fn quarantining_a_placement_marks_the_record_on_every_named_device_and_keeps_its_generation() {
+        let mut pool = pool_after_mkfs();
+        let data = pool.allocate_user_data(CheckpointTxg(3)).expect("t1");
+        pool.release_leaving_the_record_allocated_on(
+            data,
+            CheckpointTxg(4),
+            &[DeviceIdentity(0), DeviceIdentity(1)],
+        );
+        let quarantined: Vec<&AllocationRecord> = pool
+            .records()
+            .iter()
+            .filter(|record| record.slot == data.slot)
+            .collect();
+        assert_eq!(quarantined.len(), 2, "每盘一条");
+        for record in quarantined {
+            assert!(!record.is_released, "隔离的记录留在已分配");
+            assert_eq!(record.generation, CheckpointTxg(3), "分配代不变");
+            assert_eq!(
+                record.quarantine_mark,
+                AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck,
+                "隔离那次发布置上记号"
+            );
+            assert_eq!(
+                u16::from_le_bytes([record.to_bytes()[10], record.to_bytes()[11]]),
+                2 | ALLOCATION_RECORD_QUARANTINE_MARK,
+                "落盘的跨度段是 0x4002"
+            );
+        }
+        assert_eq!(pool.devices[0].deferred_slots(), 0, "隔离的槽不进 defer");
+    }
+
+    /// 释放隔离记录时改写成已释放、记号随之清掉（D19 已定项 5）：跨度段落盘是 0x8002。
+    #[test]
+    fn releasing_a_quarantined_record_rewrites_it_released_and_clears_the_mark() {
+        let mut pool = pool_after_mkfs();
+        let data = pool.allocate_user_data(CheckpointTxg(3)).expect("t1");
+        pool.release_leaving_the_record_allocated_on(
+            data,
+            CheckpointTxg(4),
+            &[DeviceIdentity(0), DeviceIdentity(1)],
+        );
+        pool.release(data, CheckpointTxg(5));
+        for record in pool
+            .records()
+            .iter()
+            .filter(|record| record.slot == data.slot)
+        {
+            assert!(record.is_released, "改写成已释放");
+            assert_eq!(record.generation, CheckpointTxg(5), "释放代");
+            assert_eq!(
+                record.quarantine_mark,
+                AllocationRecordQuarantineMark::NotQuarantined,
+                "记号随释放清掉"
+            );
+            assert_eq!(
+                u16::from_le_bytes([record.to_bytes()[10], record.to_bytes()[11]]),
+                2 | ALLOCATION_RECORD_RELEASED_FLAG,
+                "落盘的跨度段是 0x8002"
+            );
+        }
+    }
+
+    /// 回退清记号：记录仍分配、分配代不变，记号去掉，槽照旧占着。
+    #[test]
+    fn clearing_the_quarantine_mark_keeps_the_record_allocated_under_its_generation() {
+        let mut pool = pool_after_mkfs();
+        let data = pool.allocate_user_data(CheckpointTxg(3)).expect("t1");
+        pool.release_leaving_the_record_allocated_on(
+            data,
+            CheckpointTxg(4),
+            &[DeviceIdentity(0), DeviceIdentity(1)],
+        );
+        let allocated_slots_while_quarantined = pool.devices[0].allocated_slots();
+        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
+            pool.clear_the_quarantine_mark_of_a_record_the_rollback_target_references(device, data);
+        }
+        for record in pool
+            .records()
+            .iter()
+            .filter(|record| record.slot == data.slot)
+        {
+            assert!(!record.is_released, "仍分配");
+            assert_eq!(record.generation, CheckpointTxg(3), "分配代不变");
+            assert_eq!(
+                record.quarantine_mark,
+                AllocationRecordQuarantineMark::NotQuarantined,
+                "回退那一次清掉记号"
+            );
+        }
+        assert_eq!(
+            pool.devices[0].allocated_slots(),
+            allocated_slots_while_quarantined,
+            "位图不动"
+        );
+    }
+
     /// 重建时认出还没被换下的第 0 版树表单元（分配代 0、跨度 1、未释放）；换下之后（已释放）就不认。
     #[test]
     fn rebuild_from_records_remembers_the_genesis_tree_table_until_it_is_released() {
@@ -3229,6 +3470,7 @@ mod tests {
                     span_slots: 2,
                     generation: CheckpointTxg(0),
                     is_released: false,
+                    quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
                 },
                 AllocationRecord {
                     device: DeviceIdentity(0),
@@ -3240,6 +3482,7 @@ mod tests {
                         CheckpointTxg(0)
                     },
                     is_released: released,
+                    quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
                 },
             ]
         };
@@ -3275,6 +3518,7 @@ mod tests {
                             span_slots: *span_slots,
                             generation: CheckpointTxg(*generation),
                             is_released: *is_released,
+                            quarantine_mark: AllocationRecordQuarantineMark::NotQuarantined,
                         },
                     )
             })
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -974,6 +974,7 @@ mod tests {
             span_slots,
             generation: CheckpointTxg(1),
             is_released,
+            quarantine_mark: crate::allocator::AllocationRecordQuarantineMark::NotQuarantined,
         }
     }
 
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -3516,6 +3516,7 @@ fn format_time_allocator<Device: BlockDevice>(
                 span_slots: u16::try_from(placement.span).expect("单元的跨度是 1 或 2 槽"),
                 generation: CheckpointTxg(0),
                 is_released: false,
+                quarantine_mark: crate::allocator::AllocationRecordQuarantineMark::NotQuarantined,
             })
         })
         .collect();
@@ -8871,6 +8872,7 @@ fn rollback_candidate<Reader: PoolReader>(
 /// 这一种来路（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1，`PoolAllocator::release_leaving_the_record_allocated_on`）。
 /// 隔离单元同样逐盘验（D23 已定项 14「在任何写之前拒」那一格「R_old 引用、cur 的账里仍分配而同分配代的单元里被隔离的那些……同样逐盘验」，
 /// 用户 2026-10-12 定 C572「隔离单元也进逐盘验」）；现行那一版引用着的才是新旧两版共用的单元，分配器不会发出去，不验。
+/// 仍分配、同分配代而带着隔离记号的那几份另交回清记号（D19（块指针的结构与宽度预算） 已定项 5「回退复活那一次，同一次发布清掉两块盘上的 0x4000」）。
 ///
 /// 循环：外层按目标的用户可见单元（轮数 = 单元数），内层按它的两条位置项；跨轮只往三张清单里追加；提前出口都是拒。
 ///
@@ -8896,6 +8898,7 @@ fn resurrection_against_the_current_account(
     let mut resurrected = Vec::new();
     let mut units_to_check_on_every_device = Vec::new();
     let mut isolated_units_to_check_on_every_device = Vec::new();
+    let mut quarantine_marks_to_clear = Vec::new();
     for (unit, locations) in target_units {
         refuse_locations_that_do_not_name_two_pool_devices(*unit, locations, &pool_devices)
             .map_err(|_refusal| {
@@ -8952,7 +8955,21 @@ fn resurrection_against_the_current_account(
                         },
                     );
                 }
-                Some(_still_allocated_under_the_same_generation) => {}
+                // 仍分配、同分配代而带着隔离记号：回退目标引用它，回退那次发布清掉这块盘上的记号（D19（块指针的结构与宽度预算） 已定项 5
+                // 「回退复活那一次，同一次发布清掉两块盘上的 0x4000」）。
+                Some(record)
+                    if record.quarantine_mark
+                        == crate::allocator::AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck =>
+                {
+                    quarantine_marks_to_clear.push((
+                        location.device,
+                        Placement {
+                            slot: location.slot,
+                            span: unit.span_slots(),
+                        },
+                    ));
+                }
+                Some(_still_allocated_under_the_same_generation_without_the_quarantine_mark) => {}
             }
         }
         let is_referenced_by_the_current_version = locations
@@ -8968,14 +8985,17 @@ fn resurrection_against_the_current_account(
         resurrected,
         units_to_check_on_every_device,
         isolated_units_to_check_on_every_device,
+        quarantine_marks_to_clear,
     })
 }
 
-/// 复活那一判交回的：复活集（逐盘），复活集里有份的单元（逐盘验要读它们每一份），与回退目标引用的隔离单元（同样逐盘验）。
+/// 复活那一判交回的：复活集（逐盘），复活集里有份的单元（逐盘验要读它们每一份），与回退目标引用的隔离单元（同样逐盘验），
+/// 与回退目标引用的单元里现行账带着隔离记号的那几份（逐盘：哪块盘、哪个落点；回退那次发布清掉）。
 struct Resurrection {
     resurrected: Vec<ResurrectedCopy>,
     units_to_check_on_every_device: Vec<(TransactionUnit, [LocationEntry; 2])>,
     isolated_units_to_check_on_every_device: Vec<(TransactionUnit, [LocationEntry; 2])>,
+    quarantine_marks_to_clear: Vec<(DeviceIdentity, Placement)>,
 }
 
 /// 回退目标引用的隔离单元在每块盘上的那一份都要读得出、整单元 CRC-32C 与映射条目里的对得上（D23（journal 的角色与格式） 已定项 14
@@ -9278,7 +9298,8 @@ fn mapping_keys_whose_target_locations_differ_from_the_current_mapping(
 /// - 候选集：根环里按 cur 的实例表判仍然有效 ∧ txg ≥ F_生效 ∧ 带文件（[`rollback_candidate`]）。
 /// - 释放：cur 引用、新根不引用的用户可见单元（按落点比），释放代 = 新根的 txg；释放之前照发布路径读盘核，对不上的那几份留在已分配。
 ///   cur 被这次重写换下的固定点单元由发布路径照常释放。
-/// - 复活：R_old 引用的用户可见单元里 cur 的账记成已释放的，改回已分配，分配代写回 R_old 那一版账里的。
+/// - 复活：R_old 引用的用户可见单元里 cur 的账记成已释放的，改回已分配，分配代写回 R_old 那一版账里的；cur 的账里带着隔离记号的，
+///   同一次发布清掉记号（D19（块指针的结构与宽度预算） 已定项 5）。
 /// - 水位：inode 号水位 = max(cur 那一版内存里的, 环里读得出、按 cur 的实例表没被崩溃恢复抛弃的根的)（Opt2）；
 ///   树 ID 水位 = max(cur 那一版内存里的, 环里读得出的全部根的)。台阶行照抄 cur 的；谱系记录带 H_cur = cur 的 inode 号水位（从 cur 读）。
 /// - 调用方的参数与盘表：照可写挂载同一套核（[`caller_inputs_agreeing_with_the_disk`]），写入口只按盘上择到的系统配置里那份参数建。
@@ -9416,6 +9437,7 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
         resurrected,
         units_to_check_on_every_device,
         isolated_units_to_check_on_every_device,
+        quarantine_marks_to_clear,
     } = resurrection_against_the_current_account(
         target,
         &target_version,
@@ -9503,6 +9525,11 @@ pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(
             copy.allocation_generation,
         );
     }
+    for (device, placement) in &quarantine_marks_to_clear {
+        allocator.clear_the_quarantine_mark_of_a_record_the_rollback_target_references(
+            *device, *placement,
+        );
+    }
     for (_, placement) in &released_user_visible_units {
         let devices_whose_copy_failed_the_checksum: Vec<DeviceIdentity> =
             quarantined_user_visible_copies
@@ -11341,6 +11368,7 @@ mod shadow_ledger_exempts_and_isolates_slot_by_slot_tests {
             span_slots,
             generation: ALLOCATION_GENERATION,
             is_released: false,
+            quarantine_mark: crate::allocator::AllocationRecordQuarantineMark::NotQuarantined,
         }
     }
 
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -7533,6 +7533,7 @@ mod allocation_records_per_device_tests {
             span_slots: 2,
             generation: CheckpointTxg(3),
             is_released,
+            quarantine_mark: crate::allocator::AllocationRecordQuarantineMark::NotQuarantined,
         }
     }
 
@@ -7868,6 +7869,7 @@ mod allocation_record_pool_geometry_tests {
             span_slots,
             generation: CheckpointTxg(3),
             is_released: false,
+            quarantine_mark: crate::allocator::AllocationRecordQuarantineMark::NotQuarantined,
         }
     }
 
@@ -7944,6 +7946,35 @@ mod allocation_record_pool_geometry_tests {
             "两盘同槽是发布路径自己的写法（D2 已定项 10），不是重叠"
         );
     }
+
+    /// 盘上读来的带隔离记号的记录（跨度段 0x4002、0x4001，D19（块指针的结构与宽度预算） 已定项 5）按去掉记号的跨度判几何：
+    /// 起在单元区起点、跨度段 0x4002 的那条只罩两槽，紧挨着它的 0x4001 那条与它不重叠。记号当成跨度读的话，头一条罩到一万六千多槽之外、
+    /// 与后一条罩住同一个槽。
+    #[test]
+    fn records_read_with_the_quarantine_mark_are_judged_by_the_span_in_the_low_bits() {
+        let read_from_disk = |slot: u64, span_field: u16| {
+            let mut bytes = record(0, slot, 1).to_bytes();
+            bytes[10..12].copy_from_slice(&span_field.to_le_bytes());
+            AllocationRecord::parse(&bytes).expect("带隔离记号、未释放的记录解得开")
+        };
+        let quarantined_unit = read_from_disk(UNIT_AREA_START_SLOT, 0x4002);
+        let quarantined_node = read_from_disk(UNIT_AREA_START_SLOT + 2, 0x4001);
+        assert_eq!(
+            (quarantined_unit.span_slots, quarantined_node.span_slots),
+            (2, 1),
+            "跨度取低位"
+        );
+        assert_eq!(
+            [quarantined_unit.quarantine_mark, quarantined_node.quarantine_mark],
+            [crate::allocator::AllocationRecordQuarantineMark::QuarantinedAfterTheReleaseChecksumCheck; 2],
+            "两条都带隔离记号"
+        );
+        assert_eq!(
+            judge(&[quarantined_unit, quarantined_node]),
+            Ok(()),
+            "带记号的两条按跨度 2、1 判，挨着而不重叠"
+        );
+    }
 }
 
 /// 读侧认得删除新写出的三棵树的 key 宽（里程碑三·2 第 7 条「读侧补新种类的 key 宽」）：根指针不全零时冷走读按它读根、核根的自述 key 宽；
--- a/crates/singlefs-core/src/region_scan.rs
+++ b/crates/singlefs-core/src/region_scan.rs
@@ -463,6 +463,8 @@ mod tests {
                     span_slots,
                     generation: BIRTH_TXG,
                     is_released: false,
+                    quarantine_mark:
+                        crate::allocator::AllocationRecordQuarantineMark::NotQuarantined,
                 });
                 if let Some(bytes) = unit {
                     self.units.insert((device, slot), bytes.to_vec());
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -13065,7 +13065,8 @@ fn publish_admitted<Device: BlockDevice>(
 
     // 释放先于分配（D3（空间分配） 已定项 7）：被换下的落点进 defer 队列、槽仍占着，这次的落点不会落到它们上面。
     // 释放之前读盘核校验和核出对不上（读不出也算）的那几份：逻辑上照样释放（映射条目这次不再写它），那一份在它那块盘上的分配记录
-    // 留在已分配、不改成已释放（D19（块指针的结构与宽度预算） 已定项 5，用户 2026-09-24 定案），核得上的那一份照常释放。
+    // 留在已分配、不改成已释放、分配代不变，跨度段置上隔离记号 0x4000、随这次发布落盘（D19（块指针的结构与宽度预算） 已定项 5，
+    // 用户 2026-09-24 定案、记号 2026-10-22 定落点甲），核得上的那一份照常释放。
     for placement in release {
         let devices_whose_copy_failed_the_checksum: Vec<DeviceIdentity> = quarantine
             .iter()
@@ -13408,7 +13409,8 @@ fn publish_admitted<Device: BlockDevice>(
 
     // t5 分配记录树（字节表五；D8（核心索引结构） 已定项 14 按绝对槽号按位置寻址）：mkfs 的两个单元分配代 0、其余是各自分配那次发布的 txg，
     // 每盘各一条，按 (设备, 槽号) 升序分装进各自那片叶。重写哪几个节点在准入之前走到固定点定下来（`settle_the_allocation_record_tree`）：
-    // 这里取完落点之后内容变了的节点都在里面（释放之前读盘核出对不上的那几份留在已分配，只会让变了的更少）。
+    // 这里取完落点之后内容变了的节点都在里面（释放之前读盘核出对不上的那几份留在已分配、带上隔离记号：那条记录照样变了，
+    // 变的是拷贝上照常释放的同一片叶）。
     let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();
     allocation_records.sort_by_key(AllocationRecord::sort_key);
     let allocation_geometry = AllocationRecordTreeGeometry::of_allocator(allocator);
```

## 测试删自己的临时目录（`~/.local/share/singlefs/work/m4-tests-delete-temp-dirs/patch/crates.patch`）

```diff
--- a/crates/singlefs-core/src/block_device.rs
+++ b/crates/singlefs-core/src/block_device.rs
@@ -1021,26 +1021,112 @@
     static IMAGE_COUNTER: AtomicU64 = AtomicU64::new(0);
 
     /// 测试镜像一律放临时目录（`command-safety.md`），文件名带进程号与计数器免得并行测试互相盖。
-    fn temporary_image_path(tag: &str) -> std::path::PathBuf {
+    /// 交回的是认领着这条路径的守卫：建镜像之前就认领，用例做完、断言失败、panic 都删掉它。
+    fn temporary_image_path(tag: &str) -> TemporaryPathRemovedOnDrop {
         let sequence = IMAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
-        std::env::temp_dir().join(format!(
+        TemporaryPathRemovedOnDrop(std::env::temp_dir().join(format!(
             "singlefs-core-{tag}-{}-{sequence}.img",
             std::process::id()
-        ))
-    }
-
-    /// 镜像的删除守卫：用例做完、断言失败、panic 都在离开作用域时删掉镜像（`command-safety.md`「测试镜像一律放临时目录」）。
-    /// 删不掉只打一行，不在 Drop 里 panic（panic 途中再 panic 会直接中止进程）。
-    struct ImageRemovedOnDrop(std::path::PathBuf);
-
-    impl Drop for ImageRemovedOnDrop {
+        )))
+    }
+
+    /// 临时路径的删除守卫：用例做完、断言失败、panic 都在离开作用域时删掉它（文件删文件，目录连同里面的一起删，不在就算了；
+    /// `command-safety.md`「测试镜像一律放临时目录」）。核心层不依赖 harness 档，这里留一份与 `singlefs_harness::temporary_files`
+    /// 同名同删法的。删不掉只打一行，不在 Drop 里 panic（panic 途中再 panic 会直接中止进程）。
+    struct TemporaryPathRemovedOnDrop(PathBuf);
+
+    impl std::ops::Deref for TemporaryPathRemovedOnDrop {
+        type Target = Path;
+
+        fn deref(&self) -> &Path {
+            &self.0
+        }
+    }
+
+    impl AsRef<Path> for TemporaryPathRemovedOnDrop {
+        fn as_ref(&self) -> &Path {
+            &self.0
+        }
+    }
+
+    impl Drop for TemporaryPathRemovedOnDrop {
         fn drop(&mut self) {
-            if let Err(error) = std::fs::remove_file(&self.0) {
+            let removed = match std::fs::symlink_metadata(&self.0) {
+                Err(error) if error.kind() == io::ErrorKind::NotFound => return,
+                Err(error) => Err(error),
+                Ok(metadata) if metadata.is_dir() => std::fs::remove_dir_all(&self.0),
+                Ok(_not_a_directory) => std::fs::remove_file(&self.0),
+            };
+            if let Err(error) = removed {
                 if error.kind() != io::ErrorKind::NotFound {
-                    eprintln!("删不掉测试镜像 {}：{error}", self.0.display());
+                    eprintln!("删不掉临时路径 {}：{error}", self.0.display());
                 }
             }
         }
+    }
+
+    /// 场景：守卫认领一份镜像与一个带文件的目录，作用域照常走完。预期：两条路径都不在了。
+    /// 判别力（`crates/mutations.tsv`）：守卫的 Drop 不删，两条都还在，这条红。
+    #[test]
+    fn a_temporary_image_and_directory_are_gone_once_the_scope_ends_normally() {
+        let image_path;
+        let directory_path;
+        {
+            let image = temporary_image_path("guard-normal-end");
+            FileBackedBlockDevice::create_image_file_exclusively(
+                &image,
+                4096,
+                PhysicalBlockSizeInBytes(512),
+            )
+            .expect("建镜像");
+            let directory = TemporaryPathRemovedOnDrop(std::env::temp_dir().join(format!(
+                "singlefs-core-guard-directory-{}-{}",
+                std::process::id(),
+                IMAGE_COUNTER.fetch_add(1, Ordering::SeqCst)
+            )));
+            std::fs::create_dir_all(directory.join("nested")).expect("建目录");
+            std::fs::write(directory.join("nested").join("attribute"), "1\n").expect("写文件");
+            image_path = image.to_path_buf();
+            directory_path = directory.to_path_buf();
+            assert!(
+                image_path.exists() && directory_path.exists(),
+                "作用域里两条路径都在"
+            );
+        }
+        assert!(
+            !image_path.exists(),
+            "作用域走完镜像 {} 不在了",
+            image_path.display()
+        );
+        assert!(
+            !directory_path.exists(),
+            "作用域走完目录 {} 连同里面的不在了",
+            directory_path.display()
+        );
+    }
+
+    /// 场景：守卫认领的镜像建好之后作用域里 panic。预期：panic 接住之后镜像不在了。
+    /// 判别力（`crates/mutations.tsv`）：守卫的 Drop 不删，镜像还在，这条红。
+    #[test]
+    fn a_temporary_image_is_gone_after_a_panic_inside_the_scope() {
+        let image = temporary_image_path("guard-panic");
+        let image_path = image.to_path_buf();
+        let panicked = std::panic::catch_unwind(move || {
+            FileBackedBlockDevice::create_image_file_exclusively(
+                &image,
+                4096,
+                PhysicalBlockSizeInBytes(512),
+            )
+            .expect("建镜像");
+            assert!(image.exists(), "panic 之前镜像在");
+            panic!("守卫作用域里造的 panic");
+        });
+        assert!(panicked.is_err(), "作用域里真 panic 了");
+        assert!(
+            !image_path.exists(),
+            "panic 之后镜像 {} 不在了",
+            image_path.display()
+        );
     }
 
     #[test]
@@ -1073,7 +1159,6 @@
             untouched.iter().all(|byte| *byte == 0),
             "稀疏镜像没写过的地方读回全 0"
         );
-        std::fs::remove_file(&path).expect("清理镜像");
     }
 
     /// 写零把整段变成 0、段外一个字节不动；长度大于一个块时底下拆成几次写，拆的边界上不许留下没清的洞。
@@ -1136,7 +1221,6 @@
                 "偏移 {offset} 在清零段外，不该被动"
             );
         }
-        std::fs::remove_file(&path).expect("清理镜像");
     }
 
     /// 本线程经 write 一族系统调用交出去的字节数（`/proc/thread-self/io` 的 `wchar`）。
@@ -1218,7 +1302,6 @@
             IMAGE_BYTES,
             "打洞带 KEEP_SIZE，文件长度不变"
         );
-        std::fs::remove_file(&path).expect("清理镜像");
     }
 
     /// 打洞报不支持的退路（测试开关当作 `fallocate` 报了 `EOPNOTSUPP`，与真失败走同一段按 errno 分路的判定）：
@@ -1300,7 +1383,6 @@
             image_bytes,
             "退路写的全在文件里面，文件长度不变"
         );
-        std::fs::remove_file(&path).expect("清理镜像");
     }
 
     /// 打洞报别的错（测试开关当作 `fallocate` 报了 `EIO`）：照常交回 `InputOutput`、原 errno 带着，
@@ -1359,7 +1441,6 @@
             "打洞报错之后镜像该原样不动，第一个被改的偏移是 {:?}",
             whole_image.iter().position(|byte| *byte != DIRT)
         );
-        std::fs::remove_file(&path).expect("清理镜像");
     }
 
     /// 按 errno 分路：只有 `EOPNOTSUPP`（与 `ENOTSUP` 同值）与 `ENOSYS` 退回逐块写 0，`EINTR` 原样重打，
@@ -1409,7 +1490,6 @@
         const ZERO_START: u64 = 4096;
         const ZERO_LENGTH: u64 = 16384;
         let path = temporary_image_path("interrupted-punch");
-        let _image_removed_on_drop = ImageRemovedOnDrop(path.clone());
         let mut device = FileBackedBlockDevice::create_image_file_exclusively(
             &path,
             IMAGE_BYTES,
@@ -1566,7 +1646,6 @@
         ];
 
         let file_path = temporary_image_path("zero-length-file");
-        let _file_image_removed_on_drop = ImageRemovedOnDrop(file_path.clone());
         let mut file_device = FileBackedBlockDevice::create_image_file_exclusively(
             &file_path,
             IMAGE_BYTES,
@@ -1596,7 +1675,6 @@
             PageCachePolicy::GoThroughPageCache,
         ] {
             let direct_path = temporary_image_path("zero-length-direct");
-            let _direct_image_removed_on_drop = ImageRemovedOnDrop(direct_path.clone());
             std::fs::write(&direct_path, vec![DIRT; image_length]).expect("建填脏的镜像");
             let mut direct_device = DirectInputOutputBlockDevice::open_existing(
                 &direct_path,
@@ -1646,7 +1724,6 @@
             .read_at(DeviceOffsetInBytes(0), &mut read_back)
             .expect("读");
         assert_eq!(read_back, vec![0x7Eu8; 512], "被拒的写零一个字节都没落盘");
-        std::fs::remove_file(&path).expect("清理镜像");
     }
 
     /// 撞上同名文件一律报错、盘上那一份字节一个都不动（进程号会被系统复用、上一轮被杀留下的镜像不会被清：
@@ -1695,6 +1772,7 @@
             "被拒之后旧镜像逐字节不变"
         );
         drop(reopened);
+        // 守卫丢掉时也会删；这里先删，是要看重开一份不在的镜像报什么
         std::fs::remove_file(&path).expect("清理镜像");
         let missing = FileBackedBlockDevice::open_existing_image_file(
             &path,
@@ -1745,7 +1823,6 @@
             PhysicalBlockSizeInBytes(512)
         );
         assert_eq!(device.size_in_bytes(), 8192);
-        std::fs::remove_file(&path).expect("清理镜像");
     }
 
     #[test]
@@ -1808,13 +1885,13 @@
             ),
             "普通文件在 sysfs 里没有 queue 属性：探不到就报错，不拿写死的值顶上"
         );
-        std::fs::remove_file(&path).expect("清理镜像");
     }
 
     #[test]
     fn queue_attributes_are_read_from_the_sysfs_tree_and_missing_ones_are_none() {
-        let root =
-            std::env::temp_dir().join(format!("singlefs-core-fake-sysfs-{}", std::process::id()));
+        let root = TemporaryPathRemovedOnDrop(
+            std::env::temp_dir().join(format!("singlefs-core-fake-sysfs-{}", std::process::id())),
+        );
         let queue = root.join("vdz").join("queue");
         std::fs::create_dir_all(&queue).expect("建假 sysfs");
         std::fs::write(queue.join("physical_block_size"), "4096\n").expect("写属性");
@@ -1843,7 +1920,6 @@
             probe_queue_number_under(&root, Path::new("/dev/vdy"), "physical_block_size"),
             None
         );
-        std::fs::remove_dir_all(&root).expect("清理假 sysfs");
     }
 
     #[test]
@@ -1867,7 +1943,7 @@
     fn dirty_image_for_discards(
         tag: &str,
         landing: DiscardLanding,
-    ) -> (FileBackedBlockDevice, ImageRemovedOnDrop) {
+    ) -> (FileBackedBlockDevice, TemporaryPathRemovedOnDrop) {
         let path = temporary_image_path(tag);
         let mut device = FileBackedBlockDevice::create_image_file_exclusively(
             &path,
@@ -1879,10 +1955,7 @@
         device
             .write_at(DeviceOffsetInBytes(0), &dirt, WriteDurability::Plain)
             .expect("填脏");
-        (
-            device.with_test_only_discard_landing(landing),
-            ImageRemovedOnDrop(path),
-        )
+        (device.with_test_only_discard_landing(landing), path)
     }
 
     fn sector_at(device: &FileBackedBlockDevice, offset: u64) -> Vec<u8> {
@@ -2051,7 +2124,6 @@
             PageCachePolicy::GoThroughPageCache,
         ] {
             let path = temporary_image_path("direct-discard-zero-length");
-            let _removed = ImageRemovedOnDrop(path.clone());
             let mut file = std::fs::File::create(&path).expect("建文件");
             std::io::Write::write_all(&mut file, &[DISCARD_DIRT_BYTE; 65536]).expect("填脏");
             drop(file);
@@ -2084,7 +2156,6 @@
             PageCachePolicy::GoThroughPageCache,
         ] {
             let path = temporary_image_path("direct-discard");
-            let _removed = ImageRemovedOnDrop(path.clone());
             let mut file = std::fs::File::create(&path).expect("建文件");
             std::io::Write::write_all(&mut file, &[DISCARD_DIRT_BYTE; 65536]).expect("填脏");
             drop(file);
```
