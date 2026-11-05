# m4-close-code-r3 改动全文：收口修补二批五份补丁里落在产品代码 `crates/singlefs-core/src/` 的 hunk（原样抽出，照打补丁的次序）

## m4-close-code-r2-fix-sampling-writer

## m4-close-code-r2-fix-reader-writer
```diff
diff --git a/crates/singlefs-core/src/container_tree.rs b/crates/singlefs-core/src/container_tree.rs
index eb4729b..ec03aa0 100644
--- a/crates/singlefs-core/src/container_tree.rs
+++ b/crates/singlefs-core/src/container_tree.rs
@@ -3767,7 +3767,8 @@ mod tests {
     }
 
     /// 扇出压到 2：第 1、2 片挂在层级 1 的根下；第 3 片根满、长一层（旧根留在 (1, 0)、右半 (1, 1) 一个孩子、新根 (2, 0) 两个孩子）；
-    /// 第 5 片两层都满、再长一层：(1, 2) 满了开 (1, 3)、(2, 1) 也满了开 (2, 2)，根 (2, 0) 那一层原先只有两个节点、新根 (3, 0)。
+    /// 第 5 片两层都满、再长一层：层级 1 的 (1, 1) 满了开 (1, 2)；它的父亲是根 (2, 0)，也满了，旧根留在 (2, 0)、右半 (2, 1) 一个孩子 (1, 2)、
+    /// 新根 (3, 0) 两个孩子 (2, 0) 与 (2, 1)。
     #[test]
     fn a_full_root_grows_a_level_with_the_old_root_kept_in_place_and_a_new_right_half() {
         assert_eq!(shape_after_opening(0, 2).root_level(), 0);
```
```diff
diff --git a/crates/singlefs-core/src/mounted_read.rs b/crates/singlefs-core/src/mounted_read.rs
index 0e7928a..a3e2982 100644
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -78,6 +78,10 @@ pub struct ReadPathObservation {
     pub stale_location_hint_hops: u64,
     /// 向块层发了几次读（每条位置条目试一次就算一次，读不回字节也算——请求已经发出去了）。
     pub device_reads_issued: u64,
+    /// 读数据单元时只坏一份的次数：同一组位置条目里前一条读不回字节或整单元校验和对不上、后一条对上（D4（校验和位置） 已定项 4
+    /// 「读到只坏一份时计数」；两条位置提示都没对上、转去查映射的那一回不算，映射那一组里只坏一份的照算）。
+    /// 只读挂载只计数、不搬：它没有写者，搬迁要走事务层的发布（C946（只坏一份的读没有计数也没有搬迁修复））。
+    pub data_unit_reads_with_one_copy_failing: u64,
 }
 
 impl ReadPathObservation {
@@ -90,6 +94,8 @@ impl ReadPathObservation {
             stale_location_hint_hops: self.stale_location_hint_hops
                 + later.stale_location_hint_hops,
             device_reads_issued: self.device_reads_issued + later.device_reads_issued,
+            data_unit_reads_with_one_copy_failing: self.data_unit_reads_with_one_copy_failing
+                + later.data_unit_reads_with_one_copy_failing,
         }
     }
 
@@ -1167,7 +1173,8 @@ enum UnitReadFailedBecause {
 
 /// 按两条位置条目逐条试着读一个单元：整单元 CRC-32C 等于条目里那 4 字节校验和才算读到
 /// （D19（块指针的结构与宽度预算） 已定项 4 的密文校验和段，加密关时是整单元 CRC-32C）。
-/// 每试一条就记一次块层读——请求已经发出去了，读不回字节也算。
+/// 每试一条就记一次块层读——请求已经发出去了，读不回字节也算。前一条没读成、后一条对上（只坏一份）记进
+/// [`ReadPathObservation::data_unit_reads_with_one_copy_failing`]，也报给读者（[`PoolReader::note_that_a_unit_was_read_with_one_copy_failing`]）。
 fn read_unit_by_location_entries(
     reader: &dyn PoolReader,
     locations: &[LocationEntry; 2],
@@ -1175,6 +1182,7 @@ fn read_unit_by_location_entries(
     observation: &mut ReadPathObservation,
 ) -> UnitReadOutcome {
     let mut some_location_returned_bytes = false;
+    let mut an_earlier_copy_failed = false;
     for location in locations {
         observation.device_reads_issued += 1;
         let Some(unit_bytes_read) = reader.read(
@@ -1182,12 +1190,18 @@ fn read_unit_by_location_entries(
             location.slot.to_device_offset(),
             unit_bytes,
         ) else {
+            an_earlier_copy_failed = true;
             continue;
         };
         some_location_returned_bytes = true;
         if crc32_castagnoli(&unit_bytes_read) == location.unit_checksum {
+            if an_earlier_copy_failed {
+                observation.data_unit_reads_with_one_copy_failing += 1;
+                reader.note_that_a_unit_was_read_with_one_copy_failing(locations);
+            }
             return UnitReadOutcome::Read(unit_bytes_read);
         }
+        an_earlier_copy_failed = true;
     }
     if some_location_returned_bytes {
         UnitReadOutcome::EveryLocationReadButTheChecksumMismatched
@@ -1272,6 +1286,7 @@ mod tests {
             data_unit_dereferences: 8,
             stale_location_hint_hops: 1,
             device_reads_issued: 9,
+            data_unit_reads_with_one_copy_failing: 0,
         };
         assert_eq!(
             observation.stale_location_hint_hops_per_million_dereferences(),
```
```diff
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index c7eb36a..ae04342 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -8,7 +8,7 @@
 //! （I-8.6（反向链算法），前一条不在盘上不判）、提交标记齐全、在飞上限之内、点名单元逐项验过，
 //! 施加一条记录 = 把所选根的四个字段换成记录新根段里的（D23（journal 的角色与格式） 已定项 15）。
 
-use std::cell::OnceCell;
+use std::cell::{Cell, OnceCell};
 use std::collections::{BTreeMap, BTreeSet};
 
 use singlefs_format::{
@@ -184,6 +184,109 @@ pub trait PoolReader {
         _length_in_bytes: usize,
     ) {
     }
+    /// 按一组位置条目读一个单元时只坏一份（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：同一组里前一条读不回字节或整单元校验和
+    /// 对不上，后一条对上、交回了它那一份。`locations` 是那一组两条位置条目（哪个单元由它们认）。默认什么都不做；要取数的读者
+    /// 套 [`PoolReaderCountingUnitReadsWithOneCopyFailing`]。两条位置提示都没对上、转去查中央映射的那一回不算（提示过期与两份都坏分不开），
+    /// 映射那一组里只坏一份的照算。
+    fn note_that_a_unit_was_read_with_one_copy_failing(&self, _locations: &[LocationEntry; 2]) {}
+}
+
+/// 套在一个读者外面数「只坏一份的读」（[`PoolReader::note_that_a_unit_was_read_with_one_copy_failing`]）：读与别的几样报告原样交给里面那一份，
+/// 只坏一份的那一报另加一次计数。计数只在内存里，随这个套子活着：冷恢复读不挂载，只计数、报给读者，不搬
+/// （C946（只坏一份的读没有计数也没有搬迁修复）的取数接口）。
+pub struct PoolReaderCountingUnitReadsWithOneCopyFailing<'reader, Reader: PoolReader + ?Sized> {
+    reader: &'reader Reader,
+    unit_reads_with_one_copy_failing: Cell<u64>,
+}
+
+impl<'reader, Reader: PoolReader + ?Sized>
+    PoolReaderCountingUnitReadsWithOneCopyFailing<'reader, Reader>
+{
+    #[must_use]
+    pub fn counting_from_zero(reader: &'reader Reader) -> Self {
+        PoolReaderCountingUnitReadsWithOneCopyFailing {
+            reader,
+            unit_reads_with_one_copy_failing: Cell::new(0),
+        }
+    }
+
+    /// 套上之后读到只坏一份的次数。
+    #[must_use]
+    pub fn unit_reads_with_one_copy_failing(&self) -> u64 {
+        self.unit_reads_with_one_copy_failing.get()
+    }
+}
+
+impl<Reader: PoolReader + ?Sized> PoolReader
+    for PoolReaderCountingUnitReadsWithOneCopyFailing<'_, Reader>
+{
+    fn device_identities(&self) -> Vec<DeviceIdentity> {
+        self.reader.device_identities()
+    }
+    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
+        self.reader.device_size_in_bytes(device)
+    }
+    fn read(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        length: usize,
+    ) -> Option<Vec<u8>> {
+        self.reader.read(device, offset, length)
+    }
+    fn journal_record_offsets_hint(
+        &self,
+        device: DeviceIdentity,
+        ring_start: DeviceOffsetInBytes,
+        ring_bytes: u64,
+    ) -> Option<Vec<DeviceOffsetInBytes>> {
+        self.reader
+            .journal_record_offsets_hint(device, ring_start, ring_bytes)
+    }
+    fn reread_after_a_bad_read(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        length: usize,
+    ) -> Option<Vec<u8>> {
+        self.reader.reread_after_a_bad_read(device, offset, length)
+    }
+    fn keep_the_self_verified_reread(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+    ) {
+        self.reader
+            .keep_the_self_verified_reread(device, offset, bytes);
+    }
+    fn note_that_the_bytes_read_self_verify(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+    ) {
+        self.reader
+            .note_that_the_bytes_read_self_verify(device, offset, bytes);
+    }
+    fn note_that_the_bytes_read_fail_self_verification(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        length_in_bytes: usize,
+    ) {
+        self.reader.note_that_the_bytes_read_fail_self_verification(
+            device,
+            offset,
+            length_in_bytes,
+        );
+    }
+    fn note_that_a_unit_was_read_with_one_copy_failing(&self, locations: &[LocationEntry; 2]) {
+        self.reader
+            .note_that_a_unit_was_read_with_one_copy_failing(locations);
+        self.unit_reads_with_one_copy_failing
+            .set(self.unit_reads_with_one_copy_failing.get() + 1);
+    }
 }
 
 /// 一遍读是第一次读，还是读坏之后当场重读的那一遍（D19（块指针的结构与宽度预算） 已定项 5、D16（发布语义） 已定项 1「根槽这一次读坏」那一行
@@ -300,6 +403,11 @@ impl<Reader: PoolReader + ?Sized> PoolReader for PoolReaderOfOneReadPass<'_, Rea
             length_in_bytes,
         );
     }
+    /// 两遍一样转过去：只坏一份的读不管是哪一遍读到的都计一次。
+    fn note_that_a_unit_was_read_with_one_copy_failing(&self, locations: &[LocationEntry; 2]) {
+        self.reader
+            .note_that_a_unit_was_read_with_one_copy_failing(locations);
+    }
 }
 
 impl<Device: BlockDevice> PoolReader for [(DeviceIdentity, Device)] {
@@ -687,17 +795,20 @@ pub(crate) fn span_in_slots_of_a_named_unit(named: &crate::journal::NamedUnit) -
 /// 挂载态的读（`crate::mounted_read`）打开时走同一条，不另写一份。
 /// 对得上的那一份报给读者（[`PoolReader::note_that_the_bytes_read_self_verify`]），读成而对不上的那一份也报
 /// （[`PoolReader::note_that_the_bytes_read_fail_self_verification`]：可写挂载这次挂载共用的那一份记进读坏账、丢掉那几块）。
+/// 前一条没读成、后一条对上（只坏一份）另报一声（[`PoolReader::note_that_a_unit_was_read_with_one_copy_failing`]，D4（校验和位置） 已定项 4 的计数）。
 pub(crate) fn read_unit_via_locations<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     locations: &[LocationEntry; 2],
     unit_bytes: usize,
 ) -> Result<Vec<u8>, RecoveryFailure> {
+    let mut an_earlier_copy_failed = false;
     for location in locations {
         let Some(bytes) = reader.read(
             location.device,
             location.slot.to_device_offset(),
             unit_bytes,
         ) else {
+            an_earlier_copy_failed = true;
             continue;
         };
         if crc32_castagnoli(&bytes) == location.unit_checksum {
@@ -706,8 +817,12 @@ pub(crate) fn read_unit_via_locations<Reader: PoolReader + ?Sized>(
                 location.slot.to_device_offset(),
                 &bytes,
             );
+            if an_earlier_copy_failed {
+                reader.note_that_a_unit_was_read_with_one_copy_failing(locations);
+            }
             return Ok(bytes);
         }
+        an_earlier_copy_failed = true;
         reader.note_that_the_bytes_read_fail_self_verification(
             location.device,
             location.slot.to_device_offset(),
@@ -3688,6 +3803,7 @@ fn inode_tree_read_and_judged(
     inode_tree_root_level_is_one(&root_node)?;
     let mut leaf_containers: Vec<InodeLeafContainerVersion> = Vec::new();
     let mut leaf_container_bytes: Vec<Vec<u8>> = Vec::new();
+    let mut largest_inode_number_of_the_leaves_read = None;
     for entry in &root_node.entries {
         let (separator_key, identity_in_the_entry, leaf_pointer) =
             inode_internal_entry_parsed_and_judged(entry)?;
@@ -3712,6 +3828,12 @@ fn inode_tree_read_and_judged(
                 what: "inode 叶容器没有记录",
             });
         }
+        // 可写挂载拿重建出来的这一版接着写：写者的前提（全部叶的记录按叶序严格升序）在这里判，不收两片交叠的一版。
+        largest_inode_number_of_the_leaves_read =
+            inode_records_of_the_leaf_ascend_after_the_leaves_before(
+                largest_inode_number_of_the_leaves_read,
+                &records,
+            )?;
         leaf_containers.push(InodeLeafContainerVersion {
             // 身份取容器头那一份：条目里的身份引用与它逐字相等是 I-9.2（条目身份与子头相符），上面刚判过。
             contents: InodeLeafContainer {
@@ -5865,6 +5987,51 @@ fn container_tree_node_of_level_one_is_tight_over_its_entries(
 pub const INODE_TREE_ROOT_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE: &str =
     "inode 树根 key 区间不是子树覆盖区间（最小、最大记录号）";
 
+/// 一片 inode 叶容器里相邻两条记录的号不严格升序时 [`RecoveryFailure::InvariantViolated`] 的 `detail`
+/// （[`inode_records_of_the_leaf_ascend_after_the_leaves_before`]，I-9.4 第四句）。
+pub const INODE_RECORDS_OF_A_LEAF_DO_NOT_STRICTLY_ASCEND: &str =
+    "一片 inode 叶容器里相邻两条记录的号不严格升序";
+
+/// 一片 inode 叶容器的第一条记录号不大于叶序在它前面那几片的最大记录号（两片叶交叠）时 [`RecoveryFailure::InvariantViolated`] 的 `detail`
+/// （[`inode_records_of_the_leaf_ascend_after_the_leaves_before`]，I-9.12）。
+pub const INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM: &str =
+    "inode 叶容器的第一条记录号不大于叶序在它前面那几片的最大记录号（两片叶交叠）";
+
+/// 读回来的一片 inode 叶容器（按根的条目次序逐片喂）照写者的前提判：全部叶的记录按叶序连起来按 inode 号严格升序
+/// （`inode_tree::leaf_containers_are_well_formed` 的前一半，[`crate::inode_tree::write_records_into_leaf_containers`] 的 `requires`；
+/// 按叶序折半找 `TransactionOutput::inode_record_found_by_its_leaf` 也靠它）。盘上不满足这一条的一版（两片叶交叠违反 I-9.12，
+/// 一片里不升序违反 I-9.4 第四句）读者不收：收下了写者的前提就不成立，按叶序折半找漏找、维护的 n_live 与 n_记录漂走
+/// （判决 `research/prompts/m4-close-code-r2-main-verification.md` 第二节 T4，第三节取改法甲）。`largest_of_the_leaves_before` 是叶序在它前面那几片的
+/// 最大记录号（头一片交 `None`），交回读完这一片之后的最大记录号；没有记录的叶由调用方另拒，这里交回原来那个。
+///
+/// # Errors
+/// 片里相邻两条不严格升序 ⇒ [`RecoveryFailure::InvariantViolated`]（I-9.4，[`INODE_RECORDS_OF_A_LEAF_DO_NOT_STRICTLY_ASCEND`]）；
+/// 第一条不大于前面那几片的最大记录号 ⇒ 同一成员（I-9.12，[`INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM`]）。
+pub(crate) fn inode_records_of_the_leaf_ascend_after_the_leaves_before(
+    largest_of_the_leaves_before: Option<u64>,
+    records_of_the_leaf: &[InodeRecord],
+) -> Result<Option<u64>, RecoveryFailure> {
+    let Some(first) = records_of_the_leaf.first() else {
+        return Ok(largest_of_the_leaves_before);
+    };
+    if largest_of_the_leaves_before.is_some_and(|largest_before| largest_before >= first.inode) {
+        return Err(RecoveryFailure::InvariantViolated {
+            invariant: "I-9.12",
+            detail: INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM,
+        });
+    }
+    if records_of_the_leaf
+        .windows(2)
+        .any(|adjacent| adjacent[0].inode >= adjacent[1].inode)
+    {
+        return Err(RecoveryFailure::InvariantViolated {
+            invariant: "I-9.4",
+            detail: INODE_RECORDS_OF_A_LEAF_DO_NOT_STRICTLY_ASCEND,
+        });
+    }
+    Ok(records_of_the_leaf.last().map(|last| last.inode))
+}
+
 /// 容器树（墓碑树、谱系树、退役树）一个码 2 节点头里的 key 区间不是子树覆盖区间时 [`RecoveryFailure::InvariantViolated`] 的 `detail`
 /// （[`read_container_tree_index_nodes`]）。
 pub const CONTAINER_TREE_NODE_KEY_RANGE_IS_NOT_THE_SUBTREE_COVERAGE: &str =
@@ -7211,6 +7378,7 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
     inode_tree_root_level_is_one(&roots.inode)?;
     let mut inode_record: Option<InodeRecord> = None;
     let mut inode_numbers_read_back: Vec<u64> = Vec::new();
+    let mut largest_inode_number_of_the_leaves_read = None;
     for entry in &roots.inode.entries {
         let (separator_key, identity, child) = inode_internal_entry_parsed_and_judged(entry)?;
         let leaf_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
@@ -7239,6 +7407,12 @@ pub fn walk_to_file_in_the_unit_area_starting_at(
             root,
             expected_filesystem_identifier,
         )?;
+        // 冷走读与可写挂载重建同一判：全部叶的记录按叶序严格升序（写者的前提），不收两片交叠的一版。
+        largest_inode_number_of_the_leaves_read =
+            inode_records_of_the_leaf_ascend_after_the_leaves_before(
+                largest_inode_number_of_the_leaves_read,
+                &records,
+            )?;
         for record in records {
             inode_numbers_read_back.push(record.inode);
             if record.inode == FIRST_INODE_NUMBER {
@@ -8657,3 +8831,70 @@ mod back_chain_judgement_tests {
         );
     }
 }
+
+#[cfg(test)]
+mod inode_leaf_records_ascending_across_the_leaves_tests {
+    use super::{
+        inode_records_of_the_leaf_ascend_after_the_leaves_before, RecoveryFailure,
+        INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM,
+        INODE_RECORDS_OF_A_LEAF_DO_NOT_STRICTLY_ASCEND,
+    };
+    use crate::address::CheckpointTxg;
+    use crate::records::{
+        InodeRecord, InodeRecordTruncationField, NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
+    };
+
+    fn records_numbered(inode_numbers: &[u64]) -> Vec<InodeRecord> {
+        inode_numbers
+            .iter()
+            .map(|&inode| InodeRecord {
+                inode,
+                object_birth: CheckpointTxg(3),
+                nlink: NLINK_OF_A_LIVE_INODE_WITHOUT_DIRECTORIES,
+                size: 0,
+                change_count: 3,
+                write_time_seconds: 0,
+                truncation_field: InodeRecordTruncationField::NOT_WRITTEN,
+            })
+            .collect()
+    }
+
+    /// 逐片喂，交回最后的最大记录号或头一处拒。
+    fn judged_leaf_by_leaf(leaves: &[&[u64]]) -> Result<Option<u64>, RecoveryFailure> {
+        leaves.iter().try_fold(None, |largest_before, leaf| {
+            inode_records_of_the_leaf_ascend_after_the_leaves_before(
+                largest_before,
+                &records_numbered(leaf),
+            )
+        })
+    }
+
+    /// 两片叶 [1, 2, 3] 与 [4, 7]：连起来严格升序，交回最大号 7。
+    #[test]
+    fn two_leaves_ascending_across_the_boundary_are_accepted_with_the_largest_record_number() {
+        assert_eq!(judged_leaf_by_leaf(&[&[1, 2, 3], &[4, 7]]), Ok(Some(7)));
+    }
+
+    /// 攻方第二次抽样那一形缩小：左片末条改成比右片头一条大（[1, 2, 9] 与 [4, 7]），每片自己升序，两片交叠：拒在 I-9.12。
+    /// 右片头一条恰等于左片末条（[1, 2, 4] 与 [4, 7]）同样拒：严格升序，等号不放过。
+    #[test]
+    fn a_left_leaf_reaching_past_the_first_record_of_the_right_leaf_is_refused_as_overlapping() {
+        let overlapping = Err(RecoveryFailure::InvariantViolated {
+            invariant: "I-9.12",
+            detail: INODE_LEAVES_OVERLAP_THE_LEAVES_BEFORE_THEM,
+        });
+        assert_eq!(judged_leaf_by_leaf(&[&[1, 2, 9], &[4, 7]]), overlapping);
+        assert_eq!(judged_leaf_by_leaf(&[&[1, 2, 4], &[4, 7]]), overlapping);
+    }
+
+    /// 一片里两条倒序（[1, 3, 2]）或同号两条（[1, 1]）：拒在 I-9.4 第四句。
+    #[test]
+    fn records_out_of_order_within_one_leaf_are_refused_on_the_container_number_invariant() {
+        let out_of_order = Err(RecoveryFailure::InvariantViolated {
+            invariant: "I-9.4",
+            detail: INODE_RECORDS_OF_A_LEAF_DO_NOT_STRICTLY_ASCEND,
+        });
+        assert_eq!(judged_leaf_by_leaf(&[&[1, 3, 2]]), out_of_order);
+        assert_eq!(judged_leaf_by_leaf(&[&[1, 1]]), out_of_order);
+    }
+}
```
```diff
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index dc1e49f..4443afe 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -366,6 +366,9 @@ pub struct PoolWriter<'pool, Device: BlockDevice> {
     /// 这个写入口上读新槽残留的账（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节 R2：残留那一读出错记数、
     /// 不拒写，读到的字节不进新单元）。只记，发布照成；新开的写入口从 0 起。
     residue_reads: Cell<ResidueReadsOfTheWriter>,
+    /// 这个写入口上读—改—写读回旧数据单元时只坏一份的次数（D4（校验和位置） 已定项 4「读到只坏一份时计数」，
+    /// [`PoolWriter::old_data_unit_reads_back_with_one_copy_failing`]）。只在内存里、不落盘；新开的写入口从 0 起。
+    old_data_unit_reads_back_with_one_copy_failing: Cell<u64>,
 }
 
 /// 一个写入口上读新槽残留的账（[`PoolWriter::residue_reads`]）：发了几次、读不出几次。
@@ -403,6 +406,7 @@ impl<'pool, Device: BlockDevice> PoolWriter<'pool, Device> {
                 InodeNumberWatermarkSource::FromThePreviousVersionsAccounting,
             snapshot_lifetime_judgement: SnapshotLifetimeJudgement::FollowTheJudgement,
             residue_reads: Cell::new(ResidueReadsOfTheWriter::NONE),
+            old_data_unit_reads_back_with_one_copy_failing: Cell::new(0),
         }
     }
 }
@@ -423,6 +427,13 @@ impl<Device: BlockDevice> PoolWriter<'_, Device> {
         });
     }
 
+    /// 这个写入口上读—改—写读回旧数据单元时只坏一份的次数（部分写、截断与变长的尾单元；D4（校验和位置） 已定项 4「读到只坏一份时计数」）。
+    /// 读回的那个单元在同一次发布里整个重写到新槽、两份新写，旧落点照释放之前读盘核隔离（[`copies_failing_the_release_checksum_check`]）。
+    #[must_use]
+    pub fn old_data_unit_reads_back_with_one_copy_failing(&self) -> u64 {
+        self.old_data_unit_reads_back_with_one_copy_failing.get()
+    }
+
     /// 装上节点容量这个只供测试的开关（`CodeTwoTreeNodeCapacities`）。产品路径一处都不调它。
     ///
     /// # Panics
@@ -11443,13 +11454,15 @@ pub fn publish_size_change<Device: BlockDevice>(
     let payloads_of_the_rewritten_data_units: Vec<RewrittenDataUnitPayload> =
         match step.tail_data_unit_rewrite() {
             Some(rewrite) => {
-                let mut payload = read_back_the_old_payload_of_a_data_unit(
+                // 只坏一份时计数（D4（校验和位置） 已定项 4）；这个尾单元这一次就整个重写到新槽，旧落点照释放之前读盘核隔离。
+                let mut payload = read_back_the_old_payload_counting_a_failing_copy(
                     &*pool.devices,
                     previous,
                     inode,
                     record,
                     rewrite.unit,
                     &pool.parameters.filesystem_identifier,
+                    &pool.old_data_unit_reads_back_with_one_copy_failing,
                 )
                 .map_err(|read_back| {
                     PublishError::TruncationRefused(
@@ -11568,6 +11581,83 @@ pub fn read_back_the_old_payload_of_a_data_unit<Reader: PoolReader + ?Sized>(
     unit: DataUnitIndexInFile,
     filesystem_identifier: &[u8; 16],
 ) -> Result<Vec<u8>, OldDataUnitReadBackFailure> {
+    read_back_the_old_data_unit_telling_whether_a_copy_failed(
+        reader,
+        previous,
+        inode,
+        record_before,
+        unit,
+        filesystem_identifier,
+    )
+    .map(|read_back| read_back.payload)
+}
+
+/// 同 [`read_back_the_old_payload_of_a_data_unit`]，读到的是只坏一份时把 `reads_with_one_copy_failing` 加一：部分写与截断、变长读回旧单元走它，
+/// 计数交写入口那一份（[`PoolWriter::old_data_unit_reads_back_with_one_copy_failing`]）。
+///
+/// # Errors
+/// 见 [`OldDataUnitReadBackFailure`]。
+///
+/// # Panics
+/// 同 [`read_back_the_old_payload_of_a_data_unit`]。
+fn read_back_the_old_payload_counting_a_failing_copy<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    previous: &TransactionOutput,
+    inode: InodeNumber,
+    record_before: InodeRecord,
+    unit: DataUnitIndexInFile,
+    filesystem_identifier: &[u8; 16],
+    reads_with_one_copy_failing: &Cell<u64>,
+) -> Result<Vec<u8>, OldDataUnitReadBackFailure> {
+    let read_back = read_back_the_old_data_unit_telling_whether_a_copy_failed(
+        reader,
+        previous,
+        inode,
+        record_before,
+        unit,
+        filesystem_identifier,
+    )?;
+    match read_back.copies {
+        CopiesOfAUnitRead::WithoutAFailingCopy => {}
+        CopiesOfAUnitRead::FromTheOtherCopyAfterOneFailed => {
+            reads_with_one_copy_failing.set(reads_with_one_copy_failing.get() + 1);
+        }
+    }
+    Ok(read_back.payload)
+}
+
+/// 按一组两条位置条目读一个单元时读到的是哪一份（D4（校验和位置） 已定项 4「读到只坏一份时计数」）。封闭集合，`match` 不写通配臂。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CopiesOfAUnitRead {
+    /// 头一条就对上，或两条位置提示都没对上、映射那一组头一条对上（提示过期与两份都坏分不开，不算只坏一份）。
+    WithoutAFailingCopy,
+    /// 同一组里前一条读不回字节或整单元 CRC-32C 对不上，后一条对上：只坏一份。
+    FromTheOtherCopyAfterOneFailed,
+}
+
+/// 读回的旧单元：净荷与读到的是哪一份（[`read_back_the_old_data_unit_telling_whether_a_copy_failed`]）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct OldDataUnitReadBack {
+    pub payload: Vec<u8>,
+    pub copies: CopiesOfAUnitRead,
+}
+
+/// 同 [`read_back_the_old_payload_of_a_data_unit`]，另交回读到的是哪一份（[`CopiesOfAUnitRead`]）：部分写与截断、变长经
+/// [`PoolWriter`] 读回时按它给写入口记账（[`PoolWriter::old_data_unit_reads_back_with_one_copy_failing`]）。
+///
+/// # Errors
+/// 见 [`OldDataUnitReadBackFailure`]。
+///
+/// # Panics
+/// 同 [`read_back_the_old_payload_of_a_data_unit`]。
+pub fn read_back_the_old_data_unit_telling_whether_a_copy_failed<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    previous: &TransactionOutput,
+    inode: InodeNumber,
+    record_before: InodeRecord,
+    unit: DataUnitIndexInFile,
+    filesystem_identifier: &[u8; 16],
+) -> Result<OldDataUnitReadBack, OldDataUnitReadBackFailure> {
     let position = usize::try_from(unit.0).expect("单元序号");
     let pointer = if inode.0 == FIRST_INODE_NUMBER {
         previous.data_pointers.get(position)
@@ -11581,15 +11671,26 @@ pub fn read_back_the_old_payload_of_a_data_unit<Reader: PoolReader + ?Sized>(
     let role = role_of_a_data_unit_of_the_inode(inode, unit);
     let unit_bytes = usize::try_from(role.span_slots() * SLOT_BYTES)
         .expect("一个单元两槽以内，字节数装得进 usize");
+    // 读到的是哪一份：同一组里前一条没读成、后一条对上时改记只坏一份（D4（校验和位置） 已定项 4 的计数）。
+    let copies_read = Cell::new(CopiesOfAUnitRead::WithoutAFailingCopy);
     let read_by_the_locations = |locations: &[LocationEntry; 2]| {
+        let mut an_earlier_copy_failed = false;
         locations.iter().find_map(|location| {
-            reader
+            let read = reader
                 .read(
                     location.device,
                     location.slot.to_device_offset(),
                     unit_bytes,
                 )
-                .filter(|bytes| crc32_castagnoli(bytes) == location.unit_checksum)
+                .filter(|bytes| crc32_castagnoli(bytes) == location.unit_checksum);
+            match (&read, an_earlier_copy_failed) {
+                (Some(_), true) => {
+                    copies_read.set(CopiesOfAUnitRead::FromTheOtherCopyAfterOneFailed);
+                }
+                (Some(_), false) => {}
+                (None, _) => an_earlier_copy_failed = true,
+            }
+            read
         })
     };
     let bytes = match read_by_the_locations(&pointer.locations) {
@@ -11637,7 +11738,10 @@ pub fn read_back_the_old_payload_of_a_data_unit<Reader: PoolReader + ?Sized>(
     }
     let payload = data_unit_payload(&bytes, header.declared_length)
         .map_err(|_unit_error| disagrees("声明长度之后的补齐字节非零"))?;
-    Ok(payload.to_vec())
+    Ok(OldDataUnitReadBack {
+        payload: payload.to_vec(),
+        copies: copies_read.get(),
+    })
 }
 
 /// 按偏移写一个活 inode（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节；任意活 inode，R6）：把 `bytes` 写到文件偏移
@@ -11733,13 +11837,15 @@ pub fn publish_partial_write<Device: BlockDevice>(
         let unit = DataUnitIndexInFile(unit_number);
         let data_unit = write.data_unit(unit);
         let mut payload = if data_unit.reads_the_old_unit_back() {
-            read_back_the_old_payload_of_a_data_unit(
+            // 只坏一份时计数（D4（校验和位置） 已定项 4）；读回的单元这一次就整个重写到新槽，旧落点照释放之前读盘核隔离。
+            read_back_the_old_payload_counting_a_failing_copy(
                 &*pool.devices,
                 previous,
                 inode,
                 record,
                 unit,
                 &pool.parameters.filesystem_identifier,
+                &pool.old_data_unit_reads_back_with_one_copy_failing,
             )
             .map_err(|read_back| {
                 PublishError::TruncationRefused(
```

## m4-close-code-r2-fix-session-writer
```diff
diff --git a/crates/singlefs-core/src/compaction_plan.rs b/crates/singlefs-core/src/compaction_plan.rs
index 571bbe6d862a0872e7a066e6188ed18246cc79eb..a671f43a307bf63cbd433ecbb39004f2c6b7a06b 100644
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -790,7 +790,8 @@ impl CompactionSwitchesAndTheExcludedRegion {
 /// 而隔离的槽（常驻隔离集，攻方 K-c 读它）、上一次用户改动之后整理发了几次布（`B_gc`）、观测。住在挂着的会话上
 /// （`mounted_session::MountedSession::compaction`），可写挂载续做意图时先建一份、交给会话接着用（`mount::MountOutput::compaction_continuation`）。
 /// 整理挪出用户改动的调用线程之后（用户 2026-10-26 定「挪出调用线程，补齐三样保证」）另住四样：起跑判据记下的「该跑了」、上一条完成的意图
-/// （签下一条之前那一判跨趟读它）、定时兜底上一次查的钟读数、写带宽这一个窗口里整理写了多少。
+/// （签下一条之前那一判跨趟读它）、定时兜底上一次查的钟读数、写带宽这一个窗口里整理写了多少。另记这次挂载里读—改—写读回旧数据单元时只坏一份的次数
+/// （会话按挂载累计，`mounted_session::MountedSession::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct CompactionOfThisMount {
     pub segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan,
@@ -808,6 +809,10 @@ pub struct CompactionOfThisMount {
     timer_fallback_last_check: Option<CallerMonotonicClockReading>,
     /// 写带宽上限的这一个窗口（[`COMPACTION_WRITE_BANDWIDTH_WINDOW`]）：开了上限的后台入口推第一批时开、过了一个窗口长就换新的。
     write_bandwidth_window: Option<CompactionWriteBandwidthWindow>,
+    /// 这次挂载里读—改—写读回旧数据单元时只坏一份的次数（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：会话把每次用户改动里各次发布的写入口
+    /// 上的数加进来（`transaction::PoolWriter::old_data_unit_reads_back_with_one_copy_failing`）。与常驻隔离集同是这次挂载里见到的坏副本那一类观测，
+    /// 同住这一份挂载期状态；只在内存里，新挂载从 0 起（[`Self::of_a_new_mount`]）。
+    old_data_unit_reads_back_with_one_copy_failing: u64,
 }
 
 impl CompactionOfThisMount {
@@ -823,9 +828,30 @@ impl CompactionOfThisMount {
             last_completed_intent: None,
             timer_fallback_last_check: None,
             write_bandwidth_window: None,
+            old_data_unit_reads_back_with_one_copy_failing: 0,
         }
     }
 
+    /// 这次挂载里读—改—写读回旧数据单元时只坏一份的次数（各次发布之和）。
+    #[must_use]
+    pub fn old_data_unit_reads_back_with_one_copy_failing(&self) -> u64 {
+        self.old_data_unit_reads_back_with_one_copy_failing
+    }
+
+    /// 把一次用户改动里各次发布读回时只坏一份的次数加进这次挂载的累计。
+    ///
+    /// # Panics
+    /// 加起来越过 u64：每一次都是一次真读盘，一次挂载里读不到 2⁶⁴ 次。
+    pub(crate) fn count_old_data_unit_reads_back_with_one_copy_failing(
+        &mut self,
+        reads_of_the_change: u64,
+    ) {
+        self.old_data_unit_reads_back_with_one_copy_failing = self
+            .old_data_unit_reads_back_with_one_copy_failing
+            .checked_add(reads_of_the_change)
+            .expect("一次挂载里只坏一份的读回次数装得进 u64");
+    }
+
     /// 定时兜底这一次该不该再查一次起跑判据（D26（后台整理与放置回收） 已定项 13 第 5 条「另有定时兜底，每隔一段时间再查一次」）：
     /// 关着就不查；开着而还没查过、或离上一次查过了一个间隔，就该查，并把 `now` 记成上一次查的读数。调用方的钟往回走时按没过间隔算。
     pub(crate) fn check_the_timer_fallback(
```
```diff
diff --git a/crates/singlefs-core/src/mounted_read.rs b/crates/singlefs-core/src/mounted_read.rs
index a3e29827b6daf6dcb77a8403856eecf6019f2d36..fffcf7f293cd3b8ed3dcdbefde16dcd38d430d9f 100644
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -45,7 +45,9 @@ use crate::records::{
 };
 use crate::recovery::{
     choose_root, choose_system_configuration_naming_devices_without_a_mountable_slot,
-    fall_back_judged_on_the_three_readers, inode_tree_root_key_range_is_the_subtree_coverage,
+    fall_back_judged_on_the_three_readers,
+    inode_records_of_the_leaf_ascend_after_the_leaves_before,
+    inode_tree_root_key_range_is_the_subtree_coverage,
     read_mapped_tree_node_via_hint_then_central_mapping, read_mapped_tree_root,
     read_unit_via_locations, replay_journal, scan_journal,
     tree_table_entries_each_kind_at_most_once, FailureKindForTheFallBack, JournalScanReport,
@@ -502,6 +504,7 @@ pub fn open_pool_for_read(
         });
     }
     let mut inode_records: Vec<InodeRecord> = Vec::new();
+    let mut largest_inode_number_of_the_leaves_read = None;
     for entry_bytes in &inode_root.entries {
         let (_separator_key, identity_in_the_entry, child) = parse_inode_internal_entry_or_refusal(
             entry_bytes,
@@ -560,11 +563,20 @@ pub fn open_pool_for_read(
                 },
             );
         }
+        let mut records_of_the_container = Vec::with_capacity(container.records.len());
         for record_bytes in &container.records {
             let record = InodeRecord::parse(record_bytes)
                 .ok_or(OpenPoolForReadFailure::InodeRecordMalformed)?;
-            inode_records.push(record);
+            records_of_the_container.push(record);
         }
+        // 只读挂载与冷走读、可写挂载重建同一判：全部叶的记录按叶序严格升序（写者的前提，I-9.12 与 I-9.4 第四句），
+        // 同一张两片交叠的镜像读 inode 树的各个读者一个结局，不在这里收下（判决 `research/prompts/m4-close-code-r2-main-verification.md` 第三节第一簇）。
+        largest_inode_number_of_the_leaves_read =
+            inode_records_of_the_leaf_ascend_after_the_leaves_before(
+                largest_inode_number_of_the_leaves_read,
+                &records_of_the_container,
+            )?;
+        inode_records.extend(records_of_the_container);
     }
     // inode 树根头里的 key 区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2）：`read_mapped_tree_root` 对层级 ≥ 1 的根不判区间，
     // 读完叶容器在这里判，与冷走读同一个函数。
```
```diff
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index ccc2c6b77b4c21d22b22781325e73e9689441b42..0e0c498b02724c5ed89cd33b6ed3d41476e1e790 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -89,13 +89,14 @@ use crate::transaction::{
     publish_data_changes_published_together, publish_new_inodes, publish_overwrite,
     publish_overwrite_of_an_inode, publish_partial_write, publish_sequential_write,
     publish_sequential_write_of_an_inode, publish_size_change, publish_snapshot_destruction_intent,
-    publish_unlink_intent, publish_version, walk_the_deletion_queue_until_it_is_empty,
-    CentralMappingOfAVersionReadLeafByLeaf, CompactionIntentStep, CompactionStepRefused,
-    DataChangesPublishedTogether, DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile,
-    MappingLookup, NewInodesOfOneChange, PartialWriteBytes, PlannedUnitsRewrittenAnyway,
-    PoolVersion, PoolWriter, PublishError, ReadFailedOnTheDisk, ReadFailingSelfVerification,
-    RegionCheckOfTheIntentDeletion, ReleaseChecksumCheck, RelocationBatchRefused,
-    TransactionOutput, TransactionUnit, WholeWriteOfOneInode,
+    publish_unlink_intent, publish_version, relocations_of_old_data_units_kept_for_the_snapshots,
+    walk_the_deletion_queue_until_it_is_empty, CentralMappingOfAVersionReadLeafByLeaf,
+    CompactionIntentStep, CompactionStepRefused, DataChangesPublishedTogether,
+    DeletionAndLineageOfThePublish, DeletionQueueWalk, FirstFile, MappingLookup,
+    NewInodesOfOneChange, PartialWriteBytes, PlannedUnitsRewrittenAnyway, PoolVersion, PoolWriter,
+    PublishError, ReadFailedOnTheDisk, ReadFailingSelfVerification, RegionCheckOfTheIntentDeletion,
+    ReleaseChecksumCheck, RelocationBatchRefused, TransactionOutput, TransactionUnit,
+    WholeWriteOfOneInode,
 };
 use crate::unit::{data_unit_payload_capacity, unit_filesystem_identifier, UNIT_CLASS_DATA};
 use crate::unlink::DeletionSwitches;
@@ -192,10 +193,67 @@ pub struct UserChangePublished {
     pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
     /// 这次调用里推的抬 F 之外发成的每一版，按先后（[`VersionsPublishedInThisChange`]）。整理一批都不在这里：整理挪出了用户改动的调用线程
     /// （用户 2026-10-26 定），改动发成之后起跑判据为真只记成「该跑了」（`compaction_plan::CompactionStartJudgement`），
-    /// 由后台入口 [`MountedSession::run_compaction_in_the_background`] 去跑。
+    /// 由后台入口 [`MountedSession::run_compaction_in_the_background`] 去跑。另发的那一批搬迁（不是整理，是修只坏一份的旧单元，
+    /// [`Self::relocation_of_the_old_data_units_kept_for_the_snapshots`]）发成时排在末尾。
     pub versions_published: VersionsPublishedInThisChange,
     /// 这次调用的发布都返回之后发的 discard（判决 G5，[`discard_the_segments_reusable_as_a_whole`]）。
     pub discards_after_the_change: DiscardsAfterThePublishes,
+    /// 这次改动里读—改—写读回时只坏一份、改动之后仍被活快照钉着的旧数据单元，另发的那一批搬迁做成了什么
+    /// （[`RelocationOfTheOldDataUnitsKeptForTheSnapshots`]）。
+    pub relocation_of_the_old_data_units_kept_for_the_snapshots:
+        RelocationOfTheOldDataUnitsKeptForTheSnapshots,
+}
+
+/// 一次用户改动发成之后，读—改—写读回时只坏一份、又被活快照钉着的旧数据单元另发一批搬迁（D4（校验和位置） 已定项 4「经已有的搬迁路径把这个单元
+/// 整个搬到新槽、两份新写，旧落点照释放前核隔离；走同一个事务层」；用户 2026-11-04 定有活快照时「另发一批搬迁修好」）做成了什么。封闭集合。
+/// 没有活快照钉着时读回的单元在改动那次发布里就整个重写、旧落点照释放之前读盘核隔离，不另发。
+#[derive(Debug)]
+pub enum RelocationOfTheOldDataUnitsKeptForTheSnapshots {
+    /// 不另发：这次改动读回旧单元时没有只坏一份的，或只坏一份的那几个在改动里已经释放（没有活快照钉着）。
+    NotNeeded,
+    /// 另发了一批（`transaction::publish_relocation_batch`，接在改动发成的那一版后面）：搬了这几把映射 key 的单元，按计划的次序。
+    /// 那一版排在 `versions_published` 末尾，也是会话的 `current`。
+    Relocated { mapping_keys: Vec<Vec<u8>> },
+    /// 另发的那一批被拒（`refusal` 原样）：用户改动照成，会话的 `current` 停在改动发成的那一版上，被钉着的单元坏的那一份留在旧落点上。
+    /// 被拒之后怎么办（推抬 F 再试、留给下一次改动、等快照销毁时照释放前核隔离）条款没写，第一版不再试、交回这一种。
+    /// 计划对不上与空间准入的拒绝在这一批任何写之前交回，盘上停在改动那一版；落盘中途失败的照发布路径冻结（分配器上冻结着那一次）。
+    /// 装箱：拒绝原因带着发布路径的错，不装箱这个成员远大过另两个（clippy `large_enum_variant`）。
+    RefusedWithTheFollowUpNotDecidedByTheClauses {
+        refusal: Box<RelocationBatchRefused>,
+    },
+}
+
+/// 一次用户改动里各次发布的写入口上读—改—写读回旧数据单元时只坏一份的那几次（`transaction::PoolWriter` 的两份账），按发布的先后并起来：
+/// 次数并进这次挂载的累计（[`MountedSession::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`]），映射 key 交给另发的那一批搬迁。
+/// 被拒的那几次发布读回的也算：读是真发到盘上的。
+struct OldDataUnitReadsBackWithOneCopyFailingInThisChange {
+    reads: u64,
+    mapping_keys: Vec<Vec<u8>>,
+}
+
+impl OldDataUnitReadsBackWithOneCopyFailingInThisChange {
+    fn nothing_read_yet() -> Self {
+        Self {
+            reads: 0,
+            mapping_keys: Vec::new(),
+        }
+    }
+
+    /// 并进一次发布的写入口上那两份账。
+    ///
+    /// # Panics
+    /// 次数加起来越过 u64：每一次都是一次真读盘，一次改动里读不到 2⁶⁴ 次。
+    fn add_the_reads_of_the_writer<Device: BlockDevice>(
+        &mut self,
+        writer: &PoolWriter<'_, Device>,
+    ) {
+        self.reads = self
+            .reads
+            .checked_add(writer.old_data_unit_reads_back_with_one_copy_failing())
+            .expect("一次改动里只坏一份的读回次数装得进 u64");
+        self.mapping_keys
+            .extend(writer.mapping_keys_of_old_data_units_read_back_with_one_copy_failing());
+    }
 }
 
 /// 发布返回之后发的 discard（判决 G5）：每块盘按 64 槽对齐的整段发，各段交回了哪一种结局。
@@ -1650,6 +1708,7 @@ fn rehearse_the_push_then_the_remainder_of_the_change<Device: BlockDevice>(
             .collect();
     let remainder_after_the_push = match remainder {
         RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change) => {
+            // 预演里读回的那几次不并进这次挂载的累计：预演不是一次真改动（会话的分配器与现行那一版都不动）。
             MountedSession::publish_the_change_raising_the_floor_for_space(
                 parameters,
                 &mut devices_keeping_writes_in_memory,
@@ -1658,6 +1717,7 @@ fn rehearse_the_push_then_the_remainder_of_the_change<Device: BlockDevice>(
                 instance,
                 shadow_ledger,
                 change,
+                &mut OldDataUnitReadsBackWithOneCopyFailingInThisChange::nothing_read_yet(),
             )
             .map(|_published| ())
         }
@@ -1939,7 +1999,63 @@ fn publish_the_queue_until_it_is_empty<Device: BlockDevice>(
     )
 }
 
+/// 用户改动发成之后另发的那一批搬迁（[`RelocationOfTheOldDataUnitsKeptForTheSnapshots`]）：`mapping_keys` 是这次改动里读回时只坏一份的旧单元，
+/// 其中到 `current` 这一版仍被活快照钉着的（`transaction::relocations_of_old_data_units_kept_for_the_snapshots`）一个都没有就不发；有就在一个新开的
+/// 写入口上照整理的搬迁入口发一批（[`publish_the_compaction_step_once`]，意图照抄：`transaction::publish_relocation_batch` 那一支），
+/// 同一个事务层、同一条发布路径：原样字节写到新槽两份、留存的映射条目换到新槽、源落点照释放之前读盘核（只坏一份 ⇒ 两份一起隔离）。
+/// 发成就把那一版追加进 `versions_published`、换成 `current`；被拒 `current` 不动。
+///
+/// # Panics
+/// `current` 不是有树的一版：调用方只在一次用户改动发成之后调，发成的都是有树的一版（`publish_user_change` 开头拒了没树的那一形）。
+fn relocate_the_old_data_units_kept_for_the_snapshots<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut [(DeviceIdentity, Device)],
+    allocator: &mut PoolAllocator,
+    current: &mut PoolVersion,
+    instance: InstanceGeneration,
+    mapping_keys: &[Vec<u8>],
+    versions_published: &mut VersionsPublishedInThisChange,
+) -> RelocationOfTheOldDataUnitsKeptForTheSnapshots {
+    let version_after_the_change = current
+        .version_of_trees()
+        .expect("一次用户改动发成之后现行那一版是有树的一版");
+    let planned = relocations_of_old_data_units_kept_for_the_snapshots(
+        version_after_the_change,
+        mapping_keys,
+    );
+    if planned.is_empty() {
+        return RelocationOfTheOldDataUnitsKeptForTheSnapshots::NotNeeded;
+    }
+    match publish_the_compaction_step_once(
+        parameters,
+        devices,
+        allocator,
+        version_after_the_change,
+        &planned,
+        CompactionIntentStep::CarryTheIntent,
+        instance,
+    ) {
+        Ok(relocated) => {
+            versions_published.push(relocated.clone());
+            *current = PoolVersion::of_a_version_of_trees(relocated);
+            RelocationOfTheOldDataUnitsKeptForTheSnapshots::Relocated {
+                mapping_keys: planned
+                    .into_iter()
+                    .map(|relocation| relocation.mapping_key)
+                    .collect(),
+            }
+        }
+        Err(refusal) => {
+            RelocationOfTheOldDataUnitsKeptForTheSnapshots::RefusedWithTheFollowUpNotDecidedByTheClauses {
+                refusal: Box::new(refusal),
+            }
+        }
+    }
+}
+
 /// 在一个新开的写入口上发一次这个改动，接在 `previous` 后面。unlink 这里只发意图那一次，全部批由调用方接着走空。
+/// 发完（发成或被拒）把这个写入口上读—改—写读回时只坏一份的那两份账并进 `old_data_unit_reads_back`：会话里只有这一处的发布读回旧数据单元
+/// （部分写、截断与变长，`transaction::publish_partial_write`、`transaction::publish_size_change`），别的几处写入口上这两份账恒空。
 fn publish_the_change_once<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
     devices: &mut [(DeviceIdentity, Device)],
@@ -1947,10 +2063,29 @@ fn publish_the_change_once<Device: BlockDevice>(
     previous: &TransactionOutput,
     change: UserChange<'_>,
     instance: InstanceGeneration,
+    old_data_unit_reads_back: &mut OldDataUnitReadsBackWithOneCopyFailingInThisChange,
 ) -> Result<TransactionOutput, PublishError> {
     let switches: DeletionSwitches = allocator.deletion_switches();
     let mut writer = PoolWriter::new(parameters, devices);
     writer.set_container_tree_capacities(switches.container_tree_capacities());
+    let published =
+        publish_the_change_on_the_writer(&mut writer, allocator, previous, change, instance);
+    old_data_unit_reads_back.add_the_reads_of_the_writer(&writer);
+    published
+}
+
+/// [`publish_the_change_once`] 在它新开的写入口上按改动分发的那一段。
+#[allow(
+    clippy::needless_borrow,
+    reason = "各臂照写 `&mut writer`（借一层再自动解一层）：`crates/mutations.tsv` 里有一行按这一段的原文逐字锚着"
+)]
+fn publish_the_change_on_the_writer<Device: BlockDevice>(
+    mut writer: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    change: UserChange<'_>,
+    instance: InstanceGeneration,
+) -> Result<TransactionOutput, PublishError> {
     match change {
         UserChange::Unlink {
             inode,
@@ -2074,6 +2209,15 @@ impl MountedSession {
         (output, session)
     }
 
+    /// 这次可写挂载里读—改—写读回旧数据单元时只坏一份的次数（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：每次用户改动里各次发布的
+    /// 写入口上的数之和（`transaction::PoolWriter::old_data_unit_reads_back_with_one_copy_failing`，改动被拒时那几次读也算）。只在内存里，
+    /// 不落盘；新挂载从 0 起。只读挂载那一份在挂载态上（`mounted_read::ReadPathObservation::data_unit_reads_with_one_copy_failing`）。
+    #[must_use]
+    pub fn old_data_unit_reads_back_with_one_copy_failing_in_this_mount(&self) -> u64 {
+        self.compaction
+            .old_data_unit_reads_back_with_one_copy_failing()
+    }
+
     /// 发布一次用户改动：被拒是空间不够的那两种（[`refusal_is_short_of_space`]）就推一串抬 F 的空发布、再重判，
     /// 直到发成或不再推（D16（发布语义） 已定项 1「准入」那一行）。发成时 `current` 换成新的一版；推了的那几串已落盘，
     /// 没发成时 `current` 停在最后一次落盘的空发布上。写入口只按会话持着的那份参数建（`parameters_and_device_table`），
@@ -2095,6 +2239,8 @@ impl MountedSession {
     /// 一版时只放建 inode（D8（核心索引结构） 已定项 16 第 5 条），别的改动拒成 `NoFileVersionToChange`。
     /// 发成之后查一次整理的起跑判据，为真只记成「该跑了」（`compaction_plan::CompactionStartJudgement::DueAfterAUserChange`），
     /// 返回之前一批整理都不发（整理挪出了调用线程，用户 2026-10-26 定；后台入口 [`Self::run_compaction_in_the_background`]）。
+    /// 这次改动读—改—写读回旧数据单元时只坏一份的次数并进这次挂载的累计（[`Self::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`]），
+    /// 发成之后其中仍被活快照钉着的那几个另发一批搬迁（[`RelocationOfTheOldDataUnitsKeptForTheSnapshots`]），都在 discard 之前。
     ///
     /// # Errors
     /// [`UserChangeRefused`] 的八种。
@@ -2164,7 +2310,9 @@ impl MountedSession {
                 },
             );
         }
-        let mut published = Self::publish_the_change_raising_the_floor_for_space(
+        let mut old_data_unit_reads_back =
+            OldDataUnitReadsBackWithOneCopyFailingInThisChange::nothing_read_yet();
+        let published = Self::publish_the_change_raising_the_floor_for_space(
             parameters,
             devices,
             allocator,
@@ -2172,7 +2320,24 @@ impl MountedSession {
             *instance,
             *shadow_ledger,
             change,
-        )?;
+            &mut old_data_unit_reads_back,
+        );
+        // 只坏一份的读回按一次挂载累计（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：改动被拒时那几次读也并进来。
+        compaction
+            .count_old_data_unit_reads_back_with_one_copy_failing(old_data_unit_reads_back.reads);
+        let mut published = published?;
+        // 读回时只坏一份、改动之后仍被活快照钉着的旧单元，照已有的搬迁路径另发一批（用户 2026-11-04 定「另发一批搬迁修好」）；
+        // 没有活快照钉着的那几个在改动那次发布里已经重写、旧落点照释放之前读盘核隔离，不进这一批。
+        published.relocation_of_the_old_data_units_kept_for_the_snapshots =
+            relocate_the_old_data_units_kept_for_the_snapshots(
+                parameters,
+                devices.as_mut_slice(),
+                allocator,
+                current,
+                *instance,
+                &old_data_unit_reads_back.mapping_keys,
+                &mut published.versions_published,
+            );
         // 改动发成：整理「已经过去的发布数」从这里重新数（判决 G8 `B_gc`）。起跑判据照旧在这里查一次（判决 G8 丁），整理却挪出了用户改动的
         // 调用线程（用户 2026-10-26 定「挪出调用线程」；D24（后台重活能不能卸给 GPU） 已定项 2 第 2 条「不许进前台路径」）：这里只把「该跑了」
         // 记在会话上，一批都不发，由后台入口 [`Self::run_compaction_in_the_background`] 在调用方自己的线程上去跑。这一次调用拿着 `&mut self`
@@ -2588,7 +2753,11 @@ impl MountedSession {
 
     /// [`Self::publish_user_change`] 核过盘表之后那一半：先走空、再发这次改动（unlink 连同它之后的走空），被拒是空间不够就推抬 F 再判。
     /// 要出路时的预演在只进内存的盘与分配器的拷贝上走同一段（[`rehearse_the_push_then_the_remainder_of_the_change`]），
-    /// 所以拆成不收 `self` 的一段。
+    /// 所以拆成不收 `self` 的一段。每次发这次改动的写入口上读回时只坏一份的账并进 `old_data_unit_reads_back`（发成、被拒都并）。
+    #[allow(
+        clippy::too_many_arguments,
+        reason = "参数各是一样东西：参数、盘表、分配器、现行那一版、实例代号、影子账那一臂、这次改动、只坏一份的读回账"
+    )]
     fn publish_the_change_raising_the_floor_for_space<Device: BlockDevice>(
         parameters: &MakeFilesystemParameters,
         devices: &mut Vec<(DeviceIdentity, Device)>,
@@ -2597,6 +2766,7 @@ impl MountedSession {
         instance: InstanceGeneration,
         shadow_ledger: ShadowLedger,
         change: UserChange<'_>,
+        old_data_unit_reads_back: &mut OldDataUnitReadsBackWithOneCopyFailingInThisChange,
     ) -> Result<UserChangePublished, UserChangeRefused> {
         let current_version_of_trees = match &*current {
             PoolVersion::WithFile(version_of_trees)
@@ -2675,6 +2845,7 @@ impl MountedSession {
                 version_to_change,
                 change,
                 instance,
+                old_data_unit_reads_back,
             ) {
                 Ok(published) => {
                     // unlink：意图发成之后接着把全部批走完（一次调用 = 走空 + 意图 + 全部批）；别的改动之后队列本来就空，一个写都不发。
@@ -2703,6 +2874,9 @@ impl MountedSession {
                         refusals_that_pushed_the_floor_raises,
                         versions_published,
                         discards_after_the_change: DiscardsAfterThePublishes::default(),
+                        // 另发的那一批由 `publish_user_change` 在这一段返回之后发（预演不发它），发了再换。
+                        relocation_of_the_old_data_units_kept_for_the_snapshots:
+                            RelocationOfTheOldDataUnitsKeptForTheSnapshots::NotNeeded,
                     });
                 }
                 Err(refusal) => refusal,
```
```diff
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index ae04342ddb87fa8fbea7aa641509f45cf0aa13c8..abb673c21cafdcda68c025d22e43760de3dc51e2 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -7492,10 +7492,12 @@ pub enum SnapshotFileReading {
 /// 读这条根的树表里只读快照 `snapshot`（码 14 的树号，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 5）指着的
 /// 第一个文件：从它那一对码 14 / 15 条目的两条根指针往下读 inode 树与 extent 树，节点按根指针头部的出生树核（判决 S1「读快照指着的根按根指针头部
 /// 的出生树比、不按条目自己的树号」），单元的位置提示读不出时经**这条根**的中央映射回退——快照引用、活头已换下的单元靠的就是留存在这里的
-/// 映射条目（判决 S3 的 α）。判定与冷走读那几道相同（inode 树根层级 1、叶容器与条目对得上、单元个数对得上 size、数据单元按查找路径核）。
+/// 映射条目（判决 S3 的 α）。判定与冷走读那几道相同（inode 树根层级 1、叶容器与条目对得上、全部叶的记录按叶序严格升序、单元个数对得上 size、
+/// 数据单元按查找路径核）。
 ///
 /// # Errors
-/// 树表读不出、解不开、条目不合 I-9.16 或配不成对；快照的两棵树、叶容器或数据单元读不出（提示与映射都读不出）、解不开或判红。
+/// 树表读不出、解不开、条目不合 I-9.16 或配不成对；快照的两棵树、叶容器或数据单元读不出（提示与映射都读不出）、解不开或判红
+/// （两片 inode 叶交叠拒在 I-9.12、一片里不升序拒在 I-9.4，[`inode_records_of_the_leaf_ascend_after_the_leaves_before`]）。
 pub fn read_the_first_file_of_a_snapshot(
     reader: &dyn PoolReader,
     root: &RootRecord,
@@ -7566,6 +7568,7 @@ pub fn read_the_first_file_of_a_snapshot(
     inode_tree_root_level_is_one(&inode_root)?;
     let mut inode_record: Option<InodeRecord> = None;
     let mut inode_numbers_read_back: Vec<u64> = Vec::new();
+    let mut largest_inode_number_of_the_leaves_of_the_snapshot = None;
     for entry in &inode_root.entries {
         let (separator_key, identity, child) = inode_internal_entry_parsed_and_judged(entry)?;
         let leaf_bytes = read_mapped_tree_node_via_hint_then_central_mapping(
@@ -7589,6 +7592,12 @@ pub fn read_the_first_file_of_a_snapshot(
             root,
             expected_filesystem_identifier,
         )?;
+        // 快照的冷读与冷走读、可写挂载重建、只读挂载同一判：全部叶的记录按叶序严格升序，同一张两片交叠的镜像四个读者一个结局。
+        largest_inode_number_of_the_leaves_of_the_snapshot =
+            inode_records_of_the_leaf_ascend_after_the_leaves_before(
+                largest_inode_number_of_the_leaves_of_the_snapshot,
+                &records,
+            )?;
         inode_numbers_read_back.extend(records.iter().map(|record| record.inode));
         if let Some(record) = records
             .into_iter()
```
```diff
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 4443afe0e03eaf40fb3ec435c651d57cc086bb6e..c9ee4aed65940752a0ac50a0456cf48f81874c48 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -9,7 +9,7 @@
 //! 被换下的十二个单元在同一次发布里释放（分配记录改写成已释放 + 释放代，条目不删，D3（空间分配） 已定项 7）。
 //! 每一步都经过块设备接口，录制器挂在那层（D17（实现分层与第三方管道） 已定项 5）。
 
-use std::cell::Cell;
+use std::cell::{Cell, RefCell};
 use std::collections::{BTreeMap, BTreeSet};
 
 use singlefs_format::{
@@ -369,6 +369,10 @@ pub struct PoolWriter<'pool, Device: BlockDevice> {
     /// 这个写入口上读—改—写读回旧数据单元时只坏一份的次数（D4（校验和位置） 已定项 4「读到只坏一份时计数」，
     /// [`PoolWriter::old_data_unit_reads_back_with_one_copy_failing`]）。只在内存里、不落盘；新开的写入口从 0 起。
     old_data_unit_reads_back_with_one_copy_failing: Cell<u64>,
+    /// 同一个计数里每一次读回的那个旧单元在上一版里的映射 key，按读的先后（一次一把；同一个单元读两次记两把）。会话拿它另发一批搬迁，
+    /// 把这次发布之后仍被活快照钉着（换下来进 deadlist、映射条目留存）的那几个搬到新槽（用户 2026-11-04 定「另发一批搬迁修好」，
+    /// [`relocations_of_old_data_units_kept_for_the_snapshots`]）。只在内存里；新开的写入口从空起。
+    mapping_keys_of_old_data_units_read_back_with_one_copy_failing: RefCell<Vec<Vec<u8>>>,
 }
 
 /// 一个写入口上读新槽残留的账（[`PoolWriter::residue_reads`]）：发了几次、读不出几次。
@@ -407,6 +411,7 @@ impl<'pool, Device: BlockDevice> PoolWriter<'pool, Device> {
             snapshot_lifetime_judgement: SnapshotLifetimeJudgement::FollowTheJudgement,
             residue_reads: Cell::new(ResidueReadsOfTheWriter::NONE),
             old_data_unit_reads_back_with_one_copy_failing: Cell::new(0),
+            mapping_keys_of_old_data_units_read_back_with_one_copy_failing: RefCell::new(Vec::new()),
         }
     }
 }
@@ -434,6 +439,16 @@ impl<Device: BlockDevice> PoolWriter<'_, Device> {
         self.old_data_unit_reads_back_with_one_copy_failing.get()
     }
 
+    /// 同 [`Self::old_data_unit_reads_back_with_one_copy_failing`] 那几次读回的旧单元在上一版里的映射 key，按读的先后，一次一把。
+    /// 有活快照时读回的旧单元换下来进 deadlist、不释放（判决 m4-partial-write-r1 R7），坏的那一份留在旧落点上；会话按这几把 key
+    /// 另发一批搬迁（[`relocations_of_old_data_units_kept_for_the_snapshots`]）。
+    #[must_use]
+    pub fn mapping_keys_of_old_data_units_read_back_with_one_copy_failing(&self) -> Vec<Vec<u8>> {
+        self.mapping_keys_of_old_data_units_read_back_with_one_copy_failing
+            .borrow()
+            .clone()
+    }
+
     /// 装上节点容量这个只供测试的开关（`CodeTwoTreeNodeCapacities`）。产品路径一处都不调它。
     ///
     /// # Panics
@@ -11463,6 +11478,7 @@ pub fn publish_size_change<Device: BlockDevice>(
                     rewrite.unit,
                     &pool.parameters.filesystem_identifier,
                     &pool.old_data_unit_reads_back_with_one_copy_failing,
+                    &pool.mapping_keys_of_old_data_units_read_back_with_one_copy_failing,
                 )
                 .map_err(|read_back| {
                     PublishError::TruncationRefused(
@@ -11592,14 +11608,19 @@ pub fn read_back_the_old_payload_of_a_data_unit<Reader: PoolReader + ?Sized>(
     .map(|read_back| read_back.payload)
 }
 
-/// 同 [`read_back_the_old_payload_of_a_data_unit`]，读到的是只坏一份时把 `reads_with_one_copy_failing` 加一：部分写与截断、变长读回旧单元走它，
-/// 计数交写入口那一份（[`PoolWriter::old_data_unit_reads_back_with_one_copy_failing`]）。
+/// 同 [`read_back_the_old_payload_of_a_data_unit`]，读到的是只坏一份时把 `reads_with_one_copy_failing` 加一、把这个单元在上一版里的映射 key
+/// 追加进 `mapping_keys_read_with_one_copy_failing`：部分写与截断、变长读回旧单元走它，计数与 key 交写入口那两份
+/// （[`PoolWriter::old_data_unit_reads_back_with_one_copy_failing`]、[`PoolWriter::mapping_keys_of_old_data_units_read_back_with_one_copy_failing`]）。
 ///
 /// # Errors
 /// 见 [`OldDataUnitReadBackFailure`]。
 ///
 /// # Panics
 /// 同 [`read_back_the_old_payload_of_a_data_unit`]。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：读者、上一版、哪个 inode、写之前的记录、哪个单元、本池 fsid、计数、映射 key 的账"
+)]
 fn read_back_the_old_payload_counting_a_failing_copy<Reader: PoolReader + ?Sized>(
     reader: &Reader,
     previous: &TransactionOutput,
@@ -11608,6 +11629,7 @@ fn read_back_the_old_payload_counting_a_failing_copy<Reader: PoolReader + ?Sized
     unit: DataUnitIndexInFile,
     filesystem_identifier: &[u8; 16],
     reads_with_one_copy_failing: &Cell<u64>,
+    mapping_keys_read_with_one_copy_failing: &RefCell<Vec<Vec<u8>>>,
 ) -> Result<Vec<u8>, OldDataUnitReadBackFailure> {
     let read_back = read_back_the_old_data_unit_telling_whether_a_copy_failed(
         reader,
@@ -11621,11 +11643,64 @@ fn read_back_the_old_payload_counting_a_failing_copy<Reader: PoolReader + ?Sized
         CopiesOfAUnitRead::WithoutAFailingCopy => {}
         CopiesOfAUnitRead::FromTheOtherCopyAfterOneFailed => {
             reads_with_one_copy_failing.set(reads_with_one_copy_failing.get() + 1);
+            mapping_keys_read_with_one_copy_failing
+                .borrow_mut()
+                .push(mapping_key_for_data(
+                    read_back.pointer.head,
+                    read_back.pointer.write_order,
+                ));
         }
     }
     Ok(read_back.payload)
 }
 
+/// 读—改—写读回时只坏一份的旧数据单元（`mapping_keys`，[`PoolWriter::mapping_keys_of_old_data_units_read_back_with_one_copy_failing`]）里，
+/// 到 `version` 这一版仍被活快照钉着的那几个，照已有的搬迁路径另发一批时的计划（D4（校验和位置） 已定项 4「经已有的搬迁路径把这个单元整个搬到新槽、
+/// 两份新写，旧落点照释放前核隔离」；用户 2026-11-04 定有活快照时「另发一批搬迁修好」）：被钉着的单元换下来进 deadlist、不释放（判决 m4-partial-write-r1
+/// R7），映射条目留存（`version.mapping_entries_kept_for_the_snapshots`），坏的那一份留在旧落点上，读回它的那次发布不搬它；交回的每一项就是
+/// [`publish_relocation_batch`] 收的那一种（源落点取留存条目的位置项，两盘同槽；跨度按单元类）。不在留存条目里的 key（那次发布里已经释放、
+/// 照释放之前读盘核隔离了）不进计划。按 `mapping_keys` 的次序，同一把 key 只进一次（被拒重判的那次发布读回同一个单元，记两把）。只读内存。
+///
+/// # Panics
+/// 留存条目里找得到的一把 key 是空的、或类标签不是三类用户可见单元之一：写入口记的 key 都由 [`mapping_key_for_data`] 算出，走不到。
+#[must_use]
+pub fn relocations_of_old_data_units_kept_for_the_snapshots(
+    version: &TransactionOutput,
+    mapping_keys: &[Vec<u8>],
+) -> Vec<PlannedRelocation> {
+    let mut planned: Vec<PlannedRelocation> = Vec::new();
+    // 迭代上界是 `mapping_keys` 的个数（一次改动里只坏一份的读回次数）；跨轮携带的只有已进计划的那几项。
+    for mapping_key in mapping_keys {
+        if planned
+            .iter()
+            .any(|relocation| relocation.mapping_key == *mapping_key)
+        {
+            continue;
+        }
+        let Some((_, locations)) = version
+            .mapping_entries_kept_for_the_snapshots
+            .iter()
+            .find(|(kept_key, _)| kept_key == mapping_key)
+        else {
+            continue;
+        };
+        let role = user_visible_role_standing_for_a_unit_class(
+            *mapping_key
+                .first()
+                .expect("映射 key 27 字节，首字节是类标签"),
+        )
+        .expect("这几把 key 由 mapping_key_for_data 算出、类标签是数据单元，归得到用户可见角色");
+        planned.push(PlannedRelocation {
+            mapping_key: mapping_key.clone(),
+            source: Placement {
+                slot: locations[0].slot,
+                span: role.span_slots(),
+            },
+        });
+    }
+    planned
+}
+
 /// 按一组两条位置条目读一个单元时读到的是哪一份（D4（校验和位置） 已定项 4「读到只坏一份时计数」）。封闭集合，`match` 不写通配臂。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum CopiesOfAUnitRead {
@@ -11640,6 +11715,8 @@ pub enum CopiesOfAUnitRead {
 pub struct OldDataUnitReadBack {
     pub payload: Vec<u8>,
     pub copies: CopiesOfAUnitRead,
+    /// 上一版里走到这个单元的那条数据指针（只坏一份时按它的头与写序算映射 key，[`mapping_key_for_data`]）。
+    pub pointer: DataPointer,
 }
 
 /// 同 [`read_back_the_old_payload_of_a_data_unit`]，另交回读到的是哪一份（[`CopiesOfAUnitRead`]）：部分写与截断、变长经
@@ -11741,6 +11818,7 @@ pub fn read_back_the_old_data_unit_telling_whether_a_copy_failed<Reader: PoolRea
     Ok(OldDataUnitReadBack {
         payload: payload.to_vec(),
         copies: copies_read.get(),
+        pointer,
     })
 }
 
@@ -11846,6 +11924,7 @@ pub fn publish_partial_write<Device: BlockDevice>(
                 unit,
                 &pool.parameters.filesystem_identifier,
                 &pool.old_data_unit_reads_back_with_one_copy_failing,
+                &pool.mapping_keys_of_old_data_units_read_back_with_one_copy_failing,
             )
             .map_err(|read_back| {
                 PublishError::TruncationRefused(
```

## m4-close-code-r2-fix-refusal-writer
```diff
diff --git a/crates/singlefs-core/src/compaction_plan.rs b/crates/singlefs-core/src/compaction_plan.rs
index a671f43a307bf63cbd433ecbb39004f2c6b7a06b..16a9f149903626e44f0d9600bde0c10fe74d4706 100644
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -513,6 +513,21 @@ pub enum RelocationPublishOrder {
     SourceReleasedWhileTheMappingStillNamesItForTests,
 }
 
+/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支能被测试强制进入）：会话为读回时只坏一份、又被活快照钉着的旧单元另发的那一批搬迁
+/// （`mounted_session::RelocationOfTheOldDataUnitsKeptForTheSnapshots`），在发之前当作被空间准入拒。产品路径恒 [`Self::NotForced`]。
+/// 被拒推抬 F 再试那一支（用户 2026-11-04 定「推抬 F 再试一遍」）在小盘之外的池上造不出「改动放得下、紧接着那一批放不下」，靠它进；
+/// 开关造的拒绝在这一批的写入口建起之前交回，一个写都没发。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RelocationKeptForTheSnapshotsRefusedForSpace {
+    NotForced,
+    /// 接下来这几次发这一批都当作被拒，用完之后照常发。
+    ForcedForTheNextAttemptsForTests {
+        attempts: NonZeroU64,
+    },
+    /// 每一次发这一批都当作被拒。
+    ForcedOnEveryAttemptForTests,
+}
+
 /// 整理只供测试的开关（判决 G6 `CompactionSwitches`；`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支能被测试强制进入）。
 /// 产品路径是 [`CompactionSwitches::PRODUCT`]；只住内存，装在分配器上（`allocator::PoolAllocator::set_compaction_switches`），
 /// 平常构造（`PoolAllocator::new`、挂载重建的分配器）一律是产品值；可写挂载续做意图要开关时由调用方交给挂载
@@ -791,7 +806,8 @@ impl CompactionSwitchesAndTheExcludedRegion {
 /// （`mounted_session::MountedSession::compaction`），可写挂载续做意图时先建一份、交给会话接着用（`mount::MountOutput::compaction_continuation`）。
 /// 整理挪出用户改动的调用线程之后（用户 2026-10-26 定「挪出调用线程，补齐三样保证」）另住四样：起跑判据记下的「该跑了」、上一条完成的意图
 /// （签下一条之前那一判跨趟读它）、定时兜底上一次查的钟读数、写带宽这一个窗口里整理写了多少。另记这次挂载里读—改—写读回旧数据单元时只坏一份的次数
-/// （会话按挂载累计，`mounted_session::MountedSession::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`）。
+/// （会话按挂载累计，`mounted_session::MountedSession::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`）、可写挂载读盘建上一版时只坏一份的次数
+/// （`mounted_session::MountedSession::unit_reads_with_one_copy_failing_while_mounting`），与会话另发那一批搬迁的只供测试的强制拒绝开关。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct CompactionOfThisMount {
     pub segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan,
@@ -813,6 +829,12 @@ pub struct CompactionOfThisMount {
     /// 上的数加进来（`transaction::PoolWriter::old_data_unit_reads_back_with_one_copy_failing`）。与常驻隔离集同是这次挂载里见到的坏副本那一类观测，
     /// 同住这一份挂载期状态；只在内存里，新挂载从 0 起（[`Self::of_a_new_mount`]）。
     old_data_unit_reads_back_with_one_copy_failing: u64,
+    /// 可写挂载读盘建上一版时按位置条目读一个单元只坏一份的次数（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：挂载交回之前记一次
+    /// （`mount::mount_writable` 那一串）。与上一样分开记：上一样是读—改—写读回旧数据单元，这一样是挂载读的各类单元。只在内存里，新挂载从 0 起。
+    unit_reads_with_one_copy_failing_while_mounting: u64,
+    /// 只供测试的开关：会话另发的那一批搬迁发之前当作被空间准入拒（[`RelocationKeptForTheSnapshotsRefusedForSpace`]）。新挂载是产品值。
+    relocation_kept_for_the_snapshots_refused_for_space:
+        RelocationKeptForTheSnapshotsRefusedForSpace,
 }
 
 impl CompactionOfThisMount {
@@ -829,6 +851,61 @@ impl CompactionOfThisMount {
             timer_fallback_last_check: None,
             write_bandwidth_window: None,
             old_data_unit_reads_back_with_one_copy_failing: 0,
+            unit_reads_with_one_copy_failing_while_mounting: 0,
+            relocation_kept_for_the_snapshots_refused_for_space:
+                RelocationKeptForTheSnapshotsRefusedForSpace::NotForced,
+        }
+    }
+
+    /// 可写挂载读盘建上一版时只坏一份的次数（[`Self::count_unit_reads_with_one_copy_failing_while_mounting`] 记的）。
+    #[must_use]
+    pub fn unit_reads_with_one_copy_failing_while_mounting(&self) -> u64 {
+        self.unit_reads_with_one_copy_failing_while_mounting
+    }
+
+    /// 把可写挂载读盘建上一版时只坏一份的次数加进这一份挂载期状态（挂载交回之前调一次）。
+    ///
+    /// # Panics
+    /// 加起来越过 u64：每一次都是一次真读盘，一次挂载里读不到 2⁶⁴ 次。
+    pub(crate) fn count_unit_reads_with_one_copy_failing_while_mounting(
+        &mut self,
+        reads_while_mounting: u64,
+    ) {
+        self.unit_reads_with_one_copy_failing_while_mounting = self
+            .unit_reads_with_one_copy_failing_while_mounting
+            .checked_add(reads_while_mounting)
+            .expect("一次挂载里只坏一份的读次数装得进 u64");
+    }
+
+    /// 装上只供测试的开关：会话另发的那一批搬迁发之前当作被空间准入拒（[`RelocationKeptForTheSnapshotsRefusedForSpace`]）。产品路径一处都不调它。
+    pub fn force_the_relocation_kept_for_the_snapshots_refused_for_space_for_tests(
+        &mut self,
+        forcing: RelocationKeptForTheSnapshotsRefusedForSpace,
+    ) {
+        self.relocation_kept_for_the_snapshots_refused_for_space = forcing;
+    }
+
+    /// 会话另发那一批的这一次该不该当作被空间准入拒（开关用掉一次就记下：「接下来这几次」少一次，用完回到产品值）。
+    pub(crate) fn take_a_forced_space_refusal_of_the_relocation_kept_for_the_snapshots(
+        &mut self,
+    ) -> bool {
+        match self.relocation_kept_for_the_snapshots_refused_for_space {
+            RelocationKeptForTheSnapshotsRefusedForSpace::NotForced => false,
+            RelocationKeptForTheSnapshotsRefusedForSpace::ForcedOnEveryAttemptForTests => true,
+            RelocationKeptForTheSnapshotsRefusedForSpace::ForcedForTheNextAttemptsForTests {
+                attempts,
+            } => {
+                self.relocation_kept_for_the_snapshots_refused_for_space =
+                    match NonZeroU64::new(attempts.get() - 1) {
+                        Some(attempts_left) => {
+                            RelocationKeptForTheSnapshotsRefusedForSpace::ForcedForTheNextAttemptsForTests {
+                                attempts: attempts_left,
+                            }
+                        }
+                        None => RelocationKeptForTheSnapshotsRefusedForSpace::NotForced,
+                    };
+                true
+            }
         }
     }
 
```
```diff
diff --git a/crates/singlefs-core/src/mount.rs b/crates/singlefs-core/src/mount.rs
index fdee1feb1dc33921e239be37e88cd067e84d8b3b..d4f28be3b52f78e9385313804bbbb3ae1e938d52 100644
--- a/crates/singlefs-core/src/mount.rs
+++ b/crates/singlefs-core/src/mount.rs
@@ -73,7 +73,8 @@ use crate::recovery::{
     user_visible_tree_root_pointers, verified_system_configuration_slots, BadRootRingSlotReading,
     EffectiveFloorReadingStillUnreadableAfterOneReread, InstanceTableChain,
     InstanceTableOfTheNewestRootStillUnreadableAfterOneReread, JournalScanReport,
-    NamedUnitVerification, PoolReader, RebuildVersionFailure, RebuiltVersion, RecoveryFailure,
+    NamedUnitVerification, PoolReader, PoolReaderCountingUnitReadsWithOneCopyFailing,
+    RebuildVersionFailure, RebuiltVersion, RecoveryFailure,
     RootRingSlotKnownToHoldARootStillBadAfterOneReread, SystemConfigurationSlotsOfOneDevice,
     UserVisibleTreeRootPointers,
 };
@@ -1322,25 +1323,22 @@ fn instance_table_rewrite_of_the_row_publish(
 
 /// 重建上一版：树表 0 条就留根加它指着的那张实例表；带文件的连同重建出来的单元。两臂的实例表都从根记录里那条指针沿链读
 /// （D18（块里携带什么信息） 已定项 11）——只解 `TransactionUnit::InstanceTable` 那一个单元的字节只有第 0 片，
-/// 多于一片的表会被按前半张判准入、接行。
-#[allow(
-    clippy::ptr_arg,
-    reason = "rebuild_version 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
-)]
-fn rebuild_previous_version<Device: BlockDevice>(
-    devices: &Vec<(DeviceIdentity, Device)>,
+/// 多于一片的表会被按前半张判准入、接行。读盘经 `reader`：可写挂载交的是套着数「只坏一份」的那一层
+/// （[`PoolReaderCountingUnitReadsWithOneCopyFailing`]，D4（校验和位置） 已定项 4「读到只坏一份时计数」）。
+fn rebuild_previous_version(
+    reader: &dyn PoolReader,
     unit_area_start: UnitAreaStart,
     root: &RootRecord,
     record_standing_for_root: Option<crate::journal::JournalRecord>,
 ) -> Result<PreviousVersion, MountError> {
     let rebuilt = rebuild_version_in_the_unit_area_starting_at(
-        devices,
+        reader,
         unit_area_start,
         root,
         record_standing_for_root,
     )
     .map_err(map_rebuild_failure)?;
-    let table = instance_table_chain_of_root(devices, root)
+    let table = instance_table_chain_of_root(reader, root)
         .map_err(|failure| instance_table_refusal(&failure))?;
     match rebuilt {
         RebuiltVersion::WithoutFile => Ok(PreviousVersion::WithoutFile { root: *root, table }),
@@ -1392,6 +1390,8 @@ struct InstanceStart {
     /// 这次挂载给的后台计算设置（校验过；除 [`mount_writable_with_background_compute_settings`] 之外的入口都是产品默认 `cpu`）：
     /// 挂载续做整理意图那几批照它记账，交回的会话从它起。
     background_compute: AdmittedBackgroundComputeSettings,
+    /// 建上一版按位置条目读单元时只坏一份的次数（D4（校验和位置） 已定项 4「读到只坏一份时计数」）：交回之前记进这次挂载的状态。
+    unit_reads_with_one_copy_failing_while_building_on_the_version: u64,
 }
 
 /// 新实例的第一次发布的 checkpoint_txg = max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1
@@ -7607,7 +7607,7 @@ fn establish_instance<Device: BlockDevice>(
     )?;
     let mut background_compute_of_this_mount =
         BackgroundComputeOfThisMount::of_a_mount(start.background_compute);
-    let compaction_continuation = compaction_intent_continued_at_the_mount(
+    let mut compaction_continuation = compaction_intent_continued_at_the_mount(
         parameters,
         devices.as_mut_slice(),
         &mut allocator,
@@ -7616,6 +7616,13 @@ fn establish_instance<Device: BlockDevice>(
         &deletion_queue_continuation_outcome,
         &mut background_compute_of_this_mount,
     );
+    // 建上一版读到只坏一份的次数记进这次挂载的状态（D4（校验和位置） 已定项 4「读到只坏一份时计数」），会话从它接着用。
+    // 只计数、不搬：挂载这一串里另发一批搬迁条款没写（要不要、放在哪一步、取号之前的预演罩不罩它、被拒怎么办），交主 agent。
+    compaction_continuation
+        .state_of_this_mount
+        .count_unit_reads_with_one_copy_failing_while_mounting(
+            start.unit_reads_with_one_copy_failing_while_building_on_the_version,
+        );
     // 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条「谁跑、什么时候跑」，用户 2026-10-03 定）：写行、暖机、抬 F、删除续做之后，
     // 交回之前判一次。清扫自己不让挂载失败（落点被拒这一轮不删、挂载照样交回可写，Q4），走到哪儿记进交回值。
     let instance_table_sweep = with_reads_straight_to_the_disk(&read_route, || {
@@ -9363,6 +9370,12 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
     // 判法、建退到那一版要的那条记录、F1 乙再判（`F1_AFTER_F2`）同用触发之后重扫的那一遍（D23（journal 的角色与格式） 已定项 14，用户 2026-10-20 定）。
     // 退过去的那一版建不出来，留在所选的那一版：成不成照建所选那一版。之后的一切（取号之前逐盘核、写行、影子账与 C）都按落到的那一版。
     // 退到哪一版照 `F2_NEWEST`：[见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根（判法在 `recovery` 那个函数里）。
+    // 建上一版（退到的那一版与所选的那一版两处）按位置条目读单元，经一层数「只坏一份」的读（D4（校验和位置） 已定项 4「读到只坏一份时计数」）；
+    // 读法与报给盘的几样原样交给底下这串盘，数出来的挂载交回之前记进这次挂载的状态
+    // （`CompactionOfThisMount::count_unit_reads_with_one_copy_failing_while_mounting`）。判退的三样读者不经它：它们读的是建上一版接着要读的同几个单元，
+    // 判完不留字节，经它就把同一份坏副本多数几遍。
+    let devices_counting_unit_reads_with_one_copy_failing =
+        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*devices);
     let fall_back_rebuilt = match fall_back_judged_on_the_three_readers(
         &*devices,
         &system_configuration,
@@ -9383,7 +9396,7 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
                     record_standing_for_the_version_fallen_back_to.as_ref(),
                 );
             rebuild_previous_version(
-                devices,
+                &devices_counting_unit_reads_with_one_copy_failing,
                 unit_area_start,
                 &version_fallen_back_to,
                 record_standing_for_the_version_fallen_back_to,
@@ -9434,7 +9447,7 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
             }
             None => {
                 let previous = rebuild_previous_version(
-                    devices,
+                    &devices_counting_unit_reads_with_one_copy_failing,
                     unit_area_start,
                     &effective_root,
                     own_record,
@@ -9448,6 +9461,8 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
                 )
             }
         };
+    let unit_reads_with_one_copy_failing_while_building_on_the_version =
+        devices_counting_unit_reads_with_one_copy_failing.unit_reads_with_one_copy_failing();
     // 环里没有记录时从 1 起（D23（journal 的角色与格式） 已定项 14 第 3 条：计数器全池接着走）。
     let next_counter = records
         .values()
@@ -9520,6 +9535,7 @@ fn mount_writable_with_every_test_only_switch<Device: BlockDevice>(
             },
             instance_table_sweep,
             background_compute,
+            unit_reads_with_one_copy_failing_while_building_on_the_version,
         },
     )
 }
```
```diff
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index 0e0c498b02724c5ed89cd33b6ed3d41476e1e790..a9db765a57deb72ab8fed3c2495c9b7fddfcec41 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -28,6 +28,7 @@ use crate::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber,
     InstanceGeneration, SlotNumber, TreeIdentifier,
 };
+use crate::admission::AdmissionRefusedOnSomeDevices;
 use crate::allocator::{AllocationRecord, Placement, PlacementOnDevice, PoolAllocator};
 use crate::background_compute::{
     decisions_of_one_segment, AlarmOfABackgroundTaskStoppedAfterAComputeError,
@@ -187,7 +188,8 @@ pub enum UserChange<'content> {
 /// 一次用户改动发成了：新的一版在会话的 `current` 里。
 #[derive(Debug)]
 pub struct UserChangePublished {
-    /// 这次准入里推的那几串抬 F 的空发布，按先后；一次都没推的是空的。
+    /// 这次准入里推的那几串抬 F 的空发布，按先后；一次都没推的是空的。另发的那一批搬迁被空间拒时推的不在这里，在
+    /// [`Self::relocation_of_the_old_data_units_kept_for_the_snapshots`] 里（[`FloorRaisesPushedForTheRelocation`]）。
     pub floor_raises: Vec<RaisedFloor>,
     /// 每一串是因为哪一次被拒才推的（与 `floor_raises` 一一对应）：`SpaceAdmissionRefused` 或 `PlacementRefused`。
     pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
@@ -205,24 +207,51 @@ pub struct UserChangePublished {
 }
 
 /// 一次用户改动发成之后，读—改—写读回时只坏一份、又被活快照钉着的旧数据单元另发一批搬迁（D4（校验和位置） 已定项 4「经已有的搬迁路径把这个单元
-/// 整个搬到新槽、两份新写，旧落点照释放前核隔离；走同一个事务层」；用户 2026-11-04 定有活快照时「另发一批搬迁修好」）做成了什么。封闭集合。
+/// 整个搬到新槽、两份新写，旧落点照释放前核隔离；走同一个事务层」；用户 2026-11-04 定有活快照时「另发一批搬迁修好」，另发那批被拒时
+/// 「推抬 F 再试一遍」）做成了什么。封闭集合。用户改动在每一种结局下都已发成。
 /// 没有活快照钉着时读回的单元在改动那次发布里就整个重写、旧落点照释放之前读盘核隔离，不另发。
+/// 那一批被空间拒（[`refusal_is_short_of_space`] 那两种）时与会话走空同一套（`publish_the_queue_until_it_is_empty_raising_the_floor_for_space`）：
+/// 推一串抬 F、回收之后再发同一批，预算按 `mount::push_one_floor_raise_within_the_admission_budget` 数，这一批算一次准入，推的数从 0 起。
 #[derive(Debug)]
 pub enum RelocationOfTheOldDataUnitsKeptForTheSnapshots {
     /// 不另发：这次改动读回旧单元时没有只坏一份的，或只坏一份的那几个在改动里已经释放（没有活快照钉着）。
     NotNeeded,
-    /// 另发了一批（`transaction::publish_relocation_batch`，接在改动发成的那一版后面）：搬了这几把映射 key 的单元，按计划的次序。
-    /// 那一版排在 `versions_published` 末尾，也是会话的 `current`。
-    Relocated { mapping_keys: Vec<Vec<u8>> },
-    /// 另发的那一批被拒（`refusal` 原样）：用户改动照成，会话的 `current` 停在改动发成的那一版上，被钉着的单元坏的那一份留在旧落点上。
-    /// 被拒之后怎么办（推抬 F 再试、留给下一次改动、等快照销毁时照释放前核隔离）条款没写，第一版不再试、交回这一种。
-    /// 计划对不上与空间准入的拒绝在这一批任何写之前交回，盘上停在改动那一版；落盘中途失败的照发布路径冻结（分配器上冻结着那一次）。
-    /// 装箱：拒绝原因带着发布路径的错，不装箱这个成员远大过另两个（clippy `large_enum_variant`）。
-    RefusedWithTheFollowUpNotDecidedByTheClauses {
+    /// 另发了一批（`transaction::publish_relocation_batch`，接在改动发成的那一版后面，被空间拒过的接在推的最后一串后面）：搬了这几把映射 key 的单元，
+    /// 按计划的次序。那一版排在 `versions_published` 末尾，也是会话的 `current`。
+    Relocated {
+        mapping_keys: Vec<Vec<u8>>,
+        floor_raises: FloorRaisesPushedForTheRelocation,
+    },
+    /// 被空间拒、推满仍放不下（`stop`：预算用完、F 已在上限、抬 F 那一串自己也推不出空间）：会话的 `current` 停在推的最后一串的最后一次空发布上
+    /// （一串都没推成时停在改动发成的那一版上），被钉着的单元坏的那一份留在旧落点上，这一批一个字节都没写（空间准入与落点那两拒都在任何写之前）。
+    /// `last_refusal` 是最后一次被拒的原样。装箱：拒绝原因带着发布路径的错。
+    NoSpaceAfterRaisingTheFloor {
+        last_refusal: Box<RelocationBatchRefused>,
+        stop: FloorRaiseStop,
+        floor_raises: FloorRaisesPushedForTheRelocation,
+    },
+    /// 被拒、不是空间不够（计划对不上那一版、发布路径别的错；落盘中途块设备错照发布路径冻结在分配器上，不重试）：会话的 `current` 停在最后一次落盘的
+    /// 那一版上（改动发成的那一版，或推过抬 F 时最后一次空发布）。
+    RefusedNotForSpace {
         refusal: Box<RelocationBatchRefused>,
+        floor_raises: FloorRaisesPushedForTheRelocation,
+    },
+    /// 为空间推抬 F 时抬 F 自己报了错（`mount::push_one_floor_raise_within_the_admission_budget` 的 `Err`，原样）：这一串是因为
+    /// `refusal_that_pushed_the_failed_raise` 那一次被拒才推的；之前推成的几串在 `floor_raises`。会话的 `current` 停在最后一次落盘的那一版上。
+    FloorRaiseFailedWhilePushingForSpace {
+        cause: Box<MountError>,
+        refusal_that_pushed_the_failed_raise: Box<RelocationBatchRefused>,
+        floor_raises: FloorRaisesPushedForTheRelocation,
     },
 }
 
+/// 为另发的那一批搬迁推的几串抬 F（按先后）与推它们的那几次被拒（一一对应）。一次都没被空间拒是两个空表。
+#[derive(Debug)]
+pub struct FloorRaisesPushedForTheRelocation {
+    pub floor_raises: Vec<RaisedFloor>,
+    pub refusals_that_pushed_the_floor_raises: Vec<RelocationBatchRefused>,
+}
+
 /// 一次用户改动里各次发布的写入口上读—改—写读回旧数据单元时只坏一份的那几次（`transaction::PoolWriter` 的两份账），按发布的先后并起来：
 /// 次数并进这次挂载的累计（[`MountedSession::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`]），映射 key 交给另发的那一批搬迁。
 /// 被拒的那几次发布读回的也算：读是真发到盘上的。
@@ -2000,19 +2029,27 @@ fn publish_the_queue_until_it_is_empty<Device: BlockDevice>(
 }
 
 /// 用户改动发成之后另发的那一批搬迁（[`RelocationOfTheOldDataUnitsKeptForTheSnapshots`]）：`mapping_keys` 是这次改动里读回时只坏一份的旧单元，
-/// 其中到 `current` 这一版仍被活快照钉着的（`transaction::relocations_of_old_data_units_kept_for_the_snapshots`）一个都没有就不发；有就在一个新开的
-/// 写入口上照整理的搬迁入口发一批（[`publish_the_compaction_step_once`]，意图照抄：`transaction::publish_relocation_batch` 那一支），
+/// 其中到 `current` 这一版仍被活快照钉着的（`transaction::relocations_of_old_data_units_kept_for_the_snapshots`）一个都没有就不发；有就在新开的
+/// 写入口上照整理的搬迁入口发一批（[`publish_the_relocation_batch_raising_the_floor_for_space`]，意图照抄：`transaction::publish_relocation_batch` 那一支），
 /// 同一个事务层、同一条发布路径：原样字节写到新槽两份、留存的映射条目换到新槽、源落点照释放之前读盘核（只坏一份 ⇒ 两份一起隔离）。
-/// 发成就把那一版追加进 `versions_published`、换成 `current`；被拒 `current` 不动。
+/// 被空间拒就推抬 F 再发同一批（用户 2026-11-04 定「推抬 F 再试一遍」）。推成的抬 F 已落盘：`current` 先换成推到的那一版；
+/// 那一批发成就把它追加进 `versions_published`、换成 `current`。计划只算一次：每次发之前发布路径照它接在后面的那一版核计划
+/// （计划对不上在任何写之前拒，`RelocationBatchRefused::PlannedSourceIsNotTheMappingPosition` 这一类）。
 ///
 /// # Panics
 /// `current` 不是有树的一版：调用方只在一次用户改动发成之后调，发成的都是有树的一版（`publish_user_change` 开头拒了没树的那一形）。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：参数、盘表、分配器、现行那一版、实例代号、影子账那一臂、会话这一侧的整理状态、读回只坏一份的 key、这次调用发成的各版"
+)]
 fn relocate_the_old_data_units_kept_for_the_snapshots<Device: BlockDevice>(
     parameters: &MakeFilesystemParameters,
-    devices: &mut [(DeviceIdentity, Device)],
+    devices: &mut Vec<(DeviceIdentity, Device)>,
     allocator: &mut PoolAllocator,
     current: &mut PoolVersion,
     instance: InstanceGeneration,
+    shadow_ledger: ShadowLedger,
+    compaction: &mut CompactionOfThisMount,
     mapping_keys: &[Vec<u8>],
     versions_published: &mut VersionsPublishedInThisChange,
 ) -> RelocationOfTheOldDataUnitsKeptForTheSnapshots {
@@ -2026,16 +2063,25 @@ fn relocate_the_old_data_units_kept_for_the_snapshots<Device: BlockDevice>(
     if planned.is_empty() {
         return RelocationOfTheOldDataUnitsKeptForTheSnapshots::NotNeeded;
     }
-    match publish_the_compaction_step_once(
+    let mut version_to_relocate_on = version_after_the_change.clone();
+    let batch = publish_the_relocation_batch_raising_the_floor_for_space(
         parameters,
         devices,
         allocator,
-        version_after_the_change,
-        &planned,
-        CompactionIntentStep::CarryTheIntent,
+        &mut version_to_relocate_on,
         instance,
-    ) {
-        Ok(relocated) => {
+        shadow_ledger,
+        compaction,
+        &planned,
+    );
+    // 推成的抬 F 已落盘（被拒的结局也是）：现行那一版至少是推到的那一版。
+    *current = PoolVersion::of_a_version_of_trees(version_to_relocate_on);
+    match batch {
+        RelocationBatchAfterTheFloorRaises::Published {
+            relocated,
+            floor_raises,
+        } => {
+            let relocated = *relocated;
             versions_published.push(relocated.clone());
             *current = PoolVersion::of_a_version_of_trees(relocated);
             RelocationOfTheOldDataUnitsKeptForTheSnapshots::Relocated {
@@ -2043,16 +2089,130 @@ fn relocate_the_old_data_units_kept_for_the_snapshots<Device: BlockDevice>(
                     .into_iter()
                     .map(|relocation| relocation.mapping_key)
                     .collect(),
+                floor_raises,
             }
         }
-        Err(refusal) => {
-            RelocationOfTheOldDataUnitsKeptForTheSnapshots::RefusedWithTheFollowUpNotDecidedByTheClauses {
-                refusal: Box::new(refusal),
+        RelocationBatchAfterTheFloorRaises::NotPublished(not_published) => not_published,
+    }
+}
+
+/// 另发那一批走完推抬 F 再试之后：发成（发出来的那一版与推的几串），或会话要报的那一种没发成
+/// （[`RelocationOfTheOldDataUnitsKeptForTheSnapshots`] 的三种被拒）。
+enum RelocationBatchAfterTheFloorRaises {
+    /// 装箱：一版的输出远大过另一个成员（clippy `large_enum_variant`）。
+    Published {
+        relocated: Box<TransactionOutput>,
+        floor_raises: FloorRaisesPushedForTheRelocation,
+    },
+    NotPublished(RelocationOfTheOldDataUnitsKeptForTheSnapshots),
+}
+
+/// 在新开的写入口上照 `planned` 发一批搬迁（[`publish_the_compaction_step_once`]，意图照抄），接在 `version` 后面；被空间拒
+/// （[`compaction_publish_refused_short_of_space`]：发布路径的空间准入不够、或准入放行而落点取不到）就推一串抬 F
+/// （`mount::push_one_floor_raise_within_the_admission_budget`，就地推进 `version`）、回收之后再发同一批——与会话走空被空间拒同一套
+/// （[`publish_the_queue_until_it_is_empty_raising_the_floor_for_space`]）：这一批算一次准入，推的数从 0 起，预算照那个函数数（推成的每一串的空发布数相加）。
+/// 推满仍放不下、抬 F 自己报错、被拒不是空间不够，交回会话要报的那一种；被拒不是空间不够的不再试（落盘途中块设备错照发布路径冻结在分配器上）。
+/// 只供测试的开关（`compaction_plan::RelocationKeptForTheSnapshotsRefusedForSpace`）打开时这一次不建写入口，交回一个空间准入不够的拒绝
+/// （[`space_refusal_of_the_relocation_forced_for_tests`]），照被空间拒那一支往下走。
+#[allow(
+    clippy::too_many_arguments,
+    reason = "参数各是一样东西：参数、盘表、分配器、接在后面的那一版、实例代号、影子账那一臂、会话这一侧的整理状态、这一批的计划"
+)]
+fn publish_the_relocation_batch_raising_the_floor_for_space<Device: BlockDevice>(
+    parameters: &MakeFilesystemParameters,
+    devices: &mut Vec<(DeviceIdentity, Device)>,
+    allocator: &mut PoolAllocator,
+    version: &mut TransactionOutput,
+    instance: InstanceGeneration,
+    shadow_ledger: ShadowLedger,
+    compaction: &mut CompactionOfThisMount,
+    planned: &[PlannedRelocation],
+) -> RelocationBatchAfterTheFloorRaises {
+    let mut floor_raises = FloorRaisesPushedForTheRelocation {
+        floor_raises: Vec::new(),
+        refusals_that_pushed_the_floor_raises: Vec::new(),
+    };
+    let mut publishes_pushed_in_this_admission = 0;
+    // 迭代上界：每一轮要么交回（发成、被拒不是空间不够、推满仍放不下、抬 F 报错），要么推成一串（至少一次空发布，推的数加进这次准入，
+    // 预算在 `mount::PUBLISHES_PER_ADMISSION_AT_MOST` − 1 次空发布之内停）⇒ 至多 8 轮。跨轮携带的是推到的那一版、已推的数与推成的几串。
+    loop {
+        let refusal_of_the_relocation =
+            if compaction.take_a_forced_space_refusal_of_the_relocation_kept_for_the_snapshots() {
+                space_refusal_of_the_relocation_forced_for_tests()
+            } else {
+                match publish_the_compaction_step_once(
+                    parameters,
+                    devices.as_mut_slice(),
+                    allocator,
+                    version,
+                    planned,
+                    CompactionIntentStep::CarryTheIntent,
+                    instance,
+                ) {
+                    Ok(relocated) => {
+                        return RelocationBatchAfterTheFloorRaises::Published {
+                            relocated: Box::new(relocated),
+                            floor_raises,
+                        }
+                    }
+                    Err(refusal) => refusal,
+                }
+            };
+        if !compaction_publish_refused_short_of_space(&refusal_of_the_relocation) {
+            return RelocationBatchAfterTheFloorRaises::NotPublished(
+                RelocationOfTheOldDataUnitsKeptForTheSnapshots::RefusedNotForSpace {
+                    refusal: Box::new(refusal_of_the_relocation),
+                    floor_raises,
+                },
+            );
+        }
+        match push_one_floor_raise_within_the_admission_budget(
+            parameters,
+            devices,
+            allocator,
+            version,
+            shadow_ledger,
+            publishes_pushed_in_this_admission,
+        ) {
+            Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {
+                publishes_pushed_in_this_admission += raised.publishes.len();
+                floor_raises.floor_raises.push(raised);
+                floor_raises
+                    .refusals_that_pushed_the_floor_raises
+                    .push(refusal_of_the_relocation);
+            }
+            Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => {
+                return RelocationBatchAfterTheFloorRaises::NotPublished(
+                    RelocationOfTheOldDataUnitsKeptForTheSnapshots::NoSpaceAfterRaisingTheFloor {
+                        last_refusal: Box::new(refusal_of_the_relocation),
+                        stop,
+                        floor_raises,
+                    },
+                );
+            }
+            Err(cause) => {
+                return RelocationBatchAfterTheFloorRaises::NotPublished(
+                    RelocationOfTheOldDataUnitsKeptForTheSnapshots::FloorRaiseFailedWhilePushingForSpace {
+                        cause: Box::new(cause),
+                        refusal_that_pushed_the_failed_raise: Box::new(refusal_of_the_relocation),
+                        floor_raises,
+                    },
+                );
             }
         }
     }
 }
 
+/// 只供测试的开关造的那一拒（`compaction_plan::RelocationKeptForTheSnapshotsRefusedForSpace`）：空间准入不够那一种，一块不够的盘都不点名——
+/// 没有真算准入，只为把另发那一批强制进「被空间拒」那一支；在这一批的写入口建起之前交回，一个写都没发。
+fn space_refusal_of_the_relocation_forced_for_tests() -> RelocationBatchRefused {
+    RelocationBatchRefused::Publish(PublishError::SpaceAdmissionRefused(
+        AdmissionRefusedOnSomeDevices {
+            short_devices: Vec::new(),
+        },
+    ))
+}
+
 /// 在一个新开的写入口上发一次这个改动，接在 `previous` 后面。unlink 这里只发意图那一次，全部批由调用方接着走空。
 /// 发完（发成或被拒）把这个写入口上读—改—写读回时只坏一份的那两份账并进 `old_data_unit_reads_back`：会话里只有这一处的发布读回旧数据单元
 /// （部分写、截断与变长，`transaction::publish_partial_write`、`transaction::publish_size_change`），别的几处写入口上这两份账恒空。
@@ -2218,6 +2378,15 @@ impl MountedSession {
             .old_data_unit_reads_back_with_one_copy_failing()
     }
 
+    /// 这次可写挂载读盘建上一版时（`mount::mount_writable` 那一串里建所选或退到的那一版）按位置条目读一个单元只坏一份的次数
+    /// （D4（校验和位置） 已定项 4「读到只坏一份时计数」），各类单元都算。与 [`Self::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`]
+    /// 按读的地方分开报：那一个是会话里读—改—写读回旧数据单元的。只在内存里，不落盘；没经过可写挂载的会话是 0。
+    #[must_use]
+    pub fn unit_reads_with_one_copy_failing_while_mounting(&self) -> u64 {
+        self.compaction
+            .unit_reads_with_one_copy_failing_while_mounting()
+    }
+
     /// 发布一次用户改动：被拒是空间不够的那两种（[`refusal_is_short_of_space`]）就推一串抬 F 的空发布、再重判，
     /// 直到发成或不再推（D16（发布语义） 已定项 1「准入」那一行）。发成时 `current` 换成新的一版；推了的那几串已落盘，
     /// 没发成时 `current` 停在最后一次落盘的空发布上。写入口只按会话持着的那份参数建（`parameters_and_device_table`），
@@ -2326,15 +2495,18 @@ impl MountedSession {
         compaction
             .count_old_data_unit_reads_back_with_one_copy_failing(old_data_unit_reads_back.reads);
         let mut published = published?;
-        // 读回时只坏一份、改动之后仍被活快照钉着的旧单元，照已有的搬迁路径另发一批（用户 2026-11-04 定「另发一批搬迁修好」）；
-        // 没有活快照钉着的那几个在改动那次发布里已经重写、旧落点照释放之前读盘核隔离，不进这一批。
+        // 读回时只坏一份、改动之后仍被活快照钉着的旧单元，照已有的搬迁路径另发一批（用户 2026-11-04 定「另发一批搬迁修好」，
+        // 被空间拒「推抬 F 再试一遍」）；没有活快照钉着的那几个在改动那次发布里已经重写、旧落点照释放之前读盘核隔离，不进这一批。
+        // 这一批没发成也不让改动算没发成：改动那一版已落盘。
         published.relocation_of_the_old_data_units_kept_for_the_snapshots =
             relocate_the_old_data_units_kept_for_the_snapshots(
                 parameters,
-                devices.as_mut_slice(),
+                devices,
                 allocator,
                 current,
                 *instance,
+                *shadow_ledger,
+                compaction,
                 &old_data_unit_reads_back.mapping_keys,
                 &mut published.versions_published,
             );
```

## m4-close-code-r2-fix-branch-switch-writer
```diff
diff --git a/crates/singlefs-core/src/compaction_plan.rs b/crates/singlefs-core/src/compaction_plan.rs
index 16a9f149903626e44f0d9600bde0c10fe74d4706..82f0f324e02b1fc5cbd41d7c284cc9a100e0c2b9 100644
--- a/crates/singlefs-core/src/compaction_plan.rs
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -528,6 +528,29 @@ pub enum RelocationKeptForTheSnapshotsRefusedForSpace {
     ForcedOnEveryAttemptForTests,
 }
 
+/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：会话另发的那一批搬迁（同 [`RelocationKeptForTheSnapshotsRefusedForSpace`]）在发之前
+/// 当作被拒、而不是空间不够（计划对不上那一版那一种：`transaction::RelocationBatchRefused::PlannedUnitNotInTheVersion`，点名计划里头一把 key）。
+/// 产品路径恒 [`Self::NotForced`]。被拒不是空间不够那一支（不推抬 F、不再试，`mounted_session::RelocationOfTheOldDataUnitsKeptForTheSnapshots::RefusedNotForSpace`）
+/// 在产品路径上要计划在算出来与发之间失效、或发布路径报空间之外的错才进得去，靠它进；开关造的拒绝在这一批的写入口建起之前交回，一个写都没发。
+/// 被空间拒的那个开关同时开着时先照那一个拒（它在前面判）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RelocationKeptForTheSnapshotsRefusedNotForSpace {
+    NotForced,
+    /// 每一次发这一批都当作被拒（不是空间不够：不再试，一批只发这一次）。
+    ForcedOnEveryAttemptForTests,
+}
+
+/// 只供测试（`.claude/rules/fs-design.md` 五条硬要求第 2 条）：会话为另发的那一批搬迁被空间拒而推抬 F 时，那一串抬 F 当作自己报了错
+/// （`mount::MountError::InstanceTableUnreadable`：`mount::push_one_floor_raise_within_the_admission_budget` 在任何写之前交得出的那一种），一个写都不发。
+/// 产品路径恒 [`Self::NotForced`]。抬 F 自己报错那一支（`mounted_session::RelocationOfTheOldDataUnitsKeptForTheSnapshots::FloorRaiseFailedWhilePushingForSpace`）
+/// 在产品路径上要推抬 F 时读盘或落盘出错才进得去，靠它进；要先被空间拒才推抬 F，与 [`RelocationKeptForTheSnapshotsRefusedForSpace`] 合着开。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum FloorRaiseForTheRelocationKeptForTheSnapshotsFails {
+    NotForced,
+    /// 每一次为这一批推抬 F 都当作抬 F 自己报了错。
+    ForcedOnEveryRaiseForTests,
+}
+
 /// 整理只供测试的开关（判决 G6 `CompactionSwitches`；`.claude/rules/fs-design.md` 五条硬要求第 2 条：每条分支能被测试强制进入）。
 /// 产品路径是 [`CompactionSwitches::PRODUCT`]；只住内存，装在分配器上（`allocator::PoolAllocator::set_compaction_switches`），
 /// 平常构造（`PoolAllocator::new`、挂载重建的分配器）一律是产品值；可写挂载续做意图要开关时由调用方交给挂载
@@ -807,7 +830,8 @@ impl CompactionSwitchesAndTheExcludedRegion {
 /// 整理挪出用户改动的调用线程之后（用户 2026-10-26 定「挪出调用线程，补齐三样保证」）另住四样：起跑判据记下的「该跑了」、上一条完成的意图
 /// （签下一条之前那一判跨趟读它）、定时兜底上一次查的钟读数、写带宽这一个窗口里整理写了多少。另记这次挂载里读—改—写读回旧数据单元时只坏一份的次数
 /// （会话按挂载累计，`mounted_session::MountedSession::old_data_unit_reads_back_with_one_copy_failing_in_this_mount`）、可写挂载读盘建上一版时只坏一份的次数
-/// （`mounted_session::MountedSession::unit_reads_with_one_copy_failing_while_mounting`），与会话另发那一批搬迁的只供测试的强制拒绝开关。
+/// （`mounted_session::MountedSession::unit_reads_with_one_copy_failing_while_mounting`），与会话另发那一批搬迁的三个只供测试的开关（被空间拒、
+/// 被拒不是空间不够、为它推的抬 F 自己报错）。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct CompactionOfThisMount {
     pub segments_refused_by_the_region_scan: SegmentsRefusedByTheRegionScan,
@@ -835,6 +859,12 @@ pub struct CompactionOfThisMount {
     /// 只供测试的开关：会话另发的那一批搬迁发之前当作被空间准入拒（[`RelocationKeptForTheSnapshotsRefusedForSpace`]）。新挂载是产品值。
     relocation_kept_for_the_snapshots_refused_for_space:
         RelocationKeptForTheSnapshotsRefusedForSpace,
+    /// 只供测试的开关：会话另发的那一批搬迁发之前当作被拒、不是空间不够（[`RelocationKeptForTheSnapshotsRefusedNotForSpace`]）。新挂载是产品值。
+    relocation_kept_for_the_snapshots_refused_not_for_space:
+        RelocationKeptForTheSnapshotsRefusedNotForSpace,
+    /// 只供测试的开关：为另发的那一批推的抬 F 当作自己报错（[`FloorRaiseForTheRelocationKeptForTheSnapshotsFails`]）。新挂载是产品值。
+    floor_raise_for_the_relocation_kept_for_the_snapshots_fails:
+        FloorRaiseForTheRelocationKeptForTheSnapshotsFails,
 }
 
 impl CompactionOfThisMount {
@@ -854,6 +884,10 @@ impl CompactionOfThisMount {
             unit_reads_with_one_copy_failing_while_mounting: 0,
             relocation_kept_for_the_snapshots_refused_for_space:
                 RelocationKeptForTheSnapshotsRefusedForSpace::NotForced,
+            relocation_kept_for_the_snapshots_refused_not_for_space:
+                RelocationKeptForTheSnapshotsRefusedNotForSpace::NotForced,
+            floor_raise_for_the_relocation_kept_for_the_snapshots_fails:
+                FloorRaiseForTheRelocationKeptForTheSnapshotsFails::NotForced,
         }
     }
 
@@ -909,6 +943,45 @@ impl CompactionOfThisMount {
         }
     }
 
+    /// 装上只供测试的开关：会话另发的那一批搬迁发之前当作被拒、不是空间不够（[`RelocationKeptForTheSnapshotsRefusedNotForSpace`]）。
+    /// 产品路径一处都不调它。
+    pub fn force_the_relocation_kept_for_the_snapshots_refused_not_for_space_for_tests(
+        &mut self,
+        forcing: RelocationKeptForTheSnapshotsRefusedNotForSpace,
+    ) {
+        self.relocation_kept_for_the_snapshots_refused_not_for_space = forcing;
+    }
+
+    /// 会话另发那一批的这一次该不该当作被拒、不是空间不够。
+    #[must_use]
+    pub(crate) fn is_the_relocation_kept_for_the_snapshots_forced_refused_not_for_space(
+        &self,
+    ) -> bool {
+        match self.relocation_kept_for_the_snapshots_refused_not_for_space {
+            RelocationKeptForTheSnapshotsRefusedNotForSpace::NotForced => false,
+            RelocationKeptForTheSnapshotsRefusedNotForSpace::ForcedOnEveryAttemptForTests => true,
+        }
+    }
+
+    /// 装上只供测试的开关：为另发的那一批推的抬 F 当作自己报错（[`FloorRaiseForTheRelocationKeptForTheSnapshotsFails`]）。产品路径一处都不调它。
+    pub fn force_the_floor_raise_for_the_relocation_kept_for_the_snapshots_to_fail_for_tests(
+        &mut self,
+        forcing: FloorRaiseForTheRelocationKeptForTheSnapshotsFails,
+    ) {
+        self.floor_raise_for_the_relocation_kept_for_the_snapshots_fails = forcing;
+    }
+
+    /// 为另发那一批推的这一串抬 F 该不该当作自己报了错。
+    #[must_use]
+    pub(crate) fn is_the_floor_raise_for_the_relocation_kept_for_the_snapshots_forced_to_fail(
+        &self,
+    ) -> bool {
+        match self.floor_raise_for_the_relocation_kept_for_the_snapshots_fails {
+            FloorRaiseForTheRelocationKeptForTheSnapshotsFails::NotForced => false,
+            FloorRaiseForTheRelocationKeptForTheSnapshotsFails::ForcedOnEveryRaiseForTests => true,
+        }
+    }
+
     /// 这次挂载里读—改—写读回旧数据单元时只坏一份的次数（各次发布之和）。
     #[must_use]
     pub fn old_data_unit_reads_back_with_one_copy_failing(&self) -> u64 {
```
```diff
diff --git a/crates/singlefs-core/src/mounted_session.rs b/crates/singlefs-core/src/mounted_session.rs
index a9db765a57deb72ab8fed3c2495c9b7fddfcec41..fc7df5b4bc30efa099c0b69225a8ad9d060ee4af 100644
--- a/crates/singlefs-core/src/mounted_session.rs
+++ b/crates/singlefs-core/src/mounted_session.rs
@@ -231,13 +231,15 @@ pub enum RelocationOfTheOldDataUnitsKeptForTheSnapshots {
         floor_raises: FloorRaisesPushedForTheRelocation,
     },
     /// 被拒、不是空间不够（计划对不上那一版、发布路径别的错；落盘中途块设备错照发布路径冻结在分配器上，不重试）：会话的 `current` 停在最后一次落盘的
-    /// 那一版上（改动发成的那一版，或推过抬 F 时最后一次空发布）。
+    /// 那一版上（改动发成的那一版，或推过抬 F 时最后一次空发布）。只供测试的开关 `compaction_plan::RelocationKeptForTheSnapshotsRefusedNotForSpace`
+    /// 强制进这一支（计划对不上那一形）。
     RefusedNotForSpace {
         refusal: Box<RelocationBatchRefused>,
         floor_raises: FloorRaisesPushedForTheRelocation,
     },
     /// 为空间推抬 F 时抬 F 自己报了错（`mount::push_one_floor_raise_within_the_admission_budget` 的 `Err`，原样）：这一串是因为
     /// `refusal_that_pushed_the_failed_raise` 那一次被拒才推的；之前推成的几串在 `floor_raises`。会话的 `current` 停在最后一次落盘的那一版上。
+    /// 只供测试的开关 `compaction_plan::FloorRaiseForTheRelocationKeptForTheSnapshotsFails`（连同被空间拒的那个开关）强制进这一支。
     FloorRaiseFailedWhilePushingForSpace {
         cause: Box<MountError>,
         refusal_that_pushed_the_failed_raise: Box<RelocationBatchRefused>,
@@ -2113,7 +2115,11 @@ enum RelocationBatchAfterTheFloorRaises {
 /// （[`publish_the_queue_until_it_is_empty_raising_the_floor_for_space`]）：这一批算一次准入，推的数从 0 起，预算照那个函数数（推成的每一串的空发布数相加）。
 /// 推满仍放不下、抬 F 自己报错、被拒不是空间不够，交回会话要报的那一种；被拒不是空间不够的不再试（落盘途中块设备错照发布路径冻结在分配器上）。
 /// 只供测试的开关（`compaction_plan::RelocationKeptForTheSnapshotsRefusedForSpace`）打开时这一次不建写入口，交回一个空间准入不够的拒绝
-/// （[`space_refusal_of_the_relocation_forced_for_tests`]），照被空间拒那一支往下走。
+/// （[`space_refusal_of_the_relocation_forced_for_tests`]），照被空间拒那一支往下走；另两个只供测试的开关同一种进法：
+/// `compaction_plan::RelocationKeptForTheSnapshotsRefusedNotForSpace` 打开时交回一个计划对不上的拒绝
+/// （[`refusal_not_for_space_of_the_relocation_forced_for_tests`]），照被拒不是空间不够那一支往下走；
+/// `compaction_plan::FloorRaiseForTheRelocationKeptForTheSnapshotsFails` 打开时为空间推抬 F 那一步不推、交回一个抬 F 的错
+/// （[`floor_raise_failure_of_the_relocation_forced_for_tests`]），照抬 F 自己报错那一支往下走。
 #[allow(
     clippy::too_many_arguments,
     reason = "参数各是一样东西：参数、盘表、分配器、接在后面的那一版、实例代号、影子账那一臂、会话这一侧的整理状态、这一批的计划"
@@ -2136,28 +2142,32 @@ fn publish_the_relocation_batch_raising_the_floor_for_space<Device: BlockDevice>
     // 迭代上界：每一轮要么交回（发成、被拒不是空间不够、推满仍放不下、抬 F 报错），要么推成一串（至少一次空发布，推的数加进这次准入，
     // 预算在 `mount::PUBLISHES_PER_ADMISSION_AT_MOST` − 1 次空发布之内停）⇒ 至多 8 轮。跨轮携带的是推到的那一版、已推的数与推成的几串。
     loop {
-        let refusal_of_the_relocation =
-            if compaction.take_a_forced_space_refusal_of_the_relocation_kept_for_the_snapshots() {
-                space_refusal_of_the_relocation_forced_for_tests()
-            } else {
-                match publish_the_compaction_step_once(
-                    parameters,
-                    devices.as_mut_slice(),
-                    allocator,
-                    version,
-                    planned,
-                    CompactionIntentStep::CarryTheIntent,
-                    instance,
-                ) {
-                    Ok(relocated) => {
-                        return RelocationBatchAfterTheFloorRaises::Published {
-                            relocated: Box::new(relocated),
-                            floor_raises,
-                        }
+        let refusal_of_the_relocation = if compaction
+            .take_a_forced_space_refusal_of_the_relocation_kept_for_the_snapshots()
+        {
+            space_refusal_of_the_relocation_forced_for_tests()
+        } else if compaction.is_the_relocation_kept_for_the_snapshots_forced_refused_not_for_space()
+        {
+            refusal_not_for_space_of_the_relocation_forced_for_tests(planned)
+        } else {
+            match publish_the_compaction_step_once(
+                parameters,
+                devices.as_mut_slice(),
+                allocator,
+                version,
+                planned,
+                CompactionIntentStep::CarryTheIntent,
+                instance,
+            ) {
+                Ok(relocated) => {
+                    return RelocationBatchAfterTheFloorRaises::Published {
+                        relocated: Box::new(relocated),
+                        floor_raises,
                     }
-                    Err(refusal) => refusal,
                 }
-            };
+                Err(refusal) => refusal,
+            }
+        };
         if !compaction_publish_refused_short_of_space(&refusal_of_the_relocation) {
             return RelocationBatchAfterTheFloorRaises::NotPublished(
                 RelocationOfTheOldDataUnitsKeptForTheSnapshots::RefusedNotForSpace {
@@ -2166,14 +2176,21 @@ fn publish_the_relocation_batch_raising_the_floor_for_space<Device: BlockDevice>
                 },
             );
         }
-        match push_one_floor_raise_within_the_admission_budget(
-            parameters,
-            devices,
-            allocator,
-            version,
-            shadow_ledger,
-            publishes_pushed_in_this_admission,
-        ) {
+        let floor_raise = if compaction
+            .is_the_floor_raise_for_the_relocation_kept_for_the_snapshots_forced_to_fail()
+        {
+            Err(floor_raise_failure_of_the_relocation_forced_for_tests())
+        } else {
+            push_one_floor_raise_within_the_admission_budget(
+                parameters,
+                devices,
+                allocator,
+                version,
+                shadow_ledger,
+                publishes_pushed_in_this_admission,
+            )
+        };
+        match floor_raise {
             Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {
                 publishes_pushed_in_this_admission += raised.publishes.len();
                 floor_raises.floor_raises.push(raised);
@@ -2213,6 +2230,29 @@ fn space_refusal_of_the_relocation_forced_for_tests() -> RelocationBatchRefused
     ))
 }
 
+/// 只供测试的开关造的那一拒（`compaction_plan::RelocationKeptForTheSnapshotsRefusedNotForSpace`）：计划对不上那一版那一种，点名计划里头一把 key——
+/// 没有真核计划，只为把另发那一批强制进「被拒、不是空间不够」那一支；在这一批的写入口建起之前交回，一个写都没发。
+///
+/// # Panics
+/// `planned` 是空的：调用方（[`relocate_the_old_data_units_kept_for_the_snapshots`]）计划为空就交回 `NotNeeded`、不发这一批。
+fn refusal_not_for_space_of_the_relocation_forced_for_tests(
+    planned: &[PlannedRelocation],
+) -> RelocationBatchRefused {
+    RelocationBatchRefused::PlannedUnitNotInTheVersion {
+        mapping_key: planned
+            .first()
+            .expect("另发那一批的计划不是空的：计划为空时调用方不发")
+            .mapping_key
+            .clone(),
+    }
+}
+
+/// 只供测试的开关造的那一错（`compaction_plan::FloorRaiseForTheRelocationKeptForTheSnapshotsFails`）：现行那一版的实例表读不出——
+/// [`push_one_floor_raise_within_the_admission_budget`] 在任何写之前交得出的一种；没有真读盘，只为把另发那一批强制进「抬 F 自己报错」那一支，一个写都没发。
+fn floor_raise_failure_of_the_relocation_forced_for_tests() -> MountError {
+    MountError::InstanceTableUnreadable
+}
+
 /// 在一个新开的写入口上发一次这个改动，接在 `previous` 后面。unlink 这里只发意图那一次，全部批由调用方接着走空。
 /// 发完（发成或被拒）把这个写入口上读—改—写读回时只坏一份的那两份账并进 `old_data_unit_reads_back`：会话里只有这一处的发布读回旧数据单元
 /// （部分写、截断与变长，`transaction::publish_partial_write`、`transaction::publish_size_change`），别的几处写入口上这两份账恒空。
```
