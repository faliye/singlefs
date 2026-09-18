# 附录二：里程碑「第二个事务」步 0 / 步 3 那批代码改动（`git diff 5f9e449 -- crates/` 原样，加两个新文件全文与变异表全文；2026-09-17）

## 一、diff（crates/ 下相对提交 5f9e449 之后的工作区）

```diff
diff --git a/crates/mutations.tsv b/crates/mutations.tsv
index 511de3a..effcc00 100644
--- a/crates/mutations.tsv
+++ b/crates/mutations.tsv
@@ -8,17 +8,23 @@
 空闲不随分配减（I-5.2 要红）	crates/singlefs-core/src/allocator.rs	        self.free_slots -= span;	        // 变异：空闲不减	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
 释放时清位图（立即复用）	crates/singlefs-core/src/allocator.rs	        self.deferred_slots += span;\n    }	        self.deferred_slots += span;\n        for index in start..end {\n            self.allocated[index] = false;\n        }\n    }	-p singlefs-harness --test second_transaction_step_one_overwrite -- released_placements	released_placements_are_not_handed_out_again_before_reclaim_exists
 分配记录树装不下不判	crates/singlefs-core/src/transaction.rs	    if records_after_this_publish > allocation_node_capacity {	    if false && records_after_this_publish > allocation_node_capacity {	-p singlefs-harness --test second_transaction_step_one_overwrite -- repeated_overwrites	repeated_overwrites_report_a_full_allocation_node_instead_of_panicking
-释放退回按提示	crates/singlefs-core/src/transaction.rs	    let release = placements_to_release_via_mapping(previous, allocator)?;	    let release = previous.placements();	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_goes_through	release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released
+释放退回按提示	crates/singlefs-core/src/transaction.rs	            placements_to_release_via_mapping(previous_version, allocator, &rewritten)?	            previous_version.placements()	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_goes_through	release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released
 映射查到 key 就算（落点指错不核记录）	crates/singlefs-core/src/transaction.rs	        let record = allocator.record_for(locations[0].device, slot).ok_or(	        let record = allocator\n            .record_for(locations[0].device, slot)\n            .or(allocator.records().first())\n            .ok_or(	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_reports_a_mapping_entry	release_reports_a_mapping_entry_whose_slot_has_no_record_or_the_wrong_span_instead_of_panicking
 失败的发布不退回分配器	crates/singlefs-core/src/transaction.rs	    if outcome.is_err() {	    if false && outcome.is_err() {	-p singlefs-harness --test second_transaction_step_one_overwrite -- publish_running_out_of_space_midway	publish_running_out_of_space_midway_leaves_the_allocator_as_it_was
-内容超长不判	crates/singlefs-core/src/transaction.rs	    if publish.content.len() > data_unit_capacity {	    if false && publish.content.len() > data_unit_capacity {	-p singlefs-harness --test second_transaction_step_one_overwrite -- content_larger_than_a_data_unit	content_larger_than_a_data_unit_payload_is_refused_before_anything_is_touched
+内容超长不判	crates/singlefs-core/src/transaction.rs	        if file.content.len() > data_unit_capacity {	        if false && file.content.len() > data_unit_capacity {	-p singlefs-harness --test second_transaction_step_one_overwrite -- content_larger_than_a_data_unit	content_larger_than_a_data_unit_payload_is_refused_before_anything_is_touched
 oracle 只比 txg 不比实例	crates/singlefs-harness/src/crash.rs	        if (effective_txg, effective_instance) < (newest_txg, newest_instance) {	        if effective_txg < newest_txg {	-p singlefs-harness --lib -- oracle_instance_tests	landing_on_the_lower_instance_of_the_same_txg_is_a_violation
 oracle 只按 txg 找版本	crates/singlefs-harness/src/crash.rs	        version.checkpoint_txg == effective_txg && version.instance == effective_instance	        version.checkpoint_txg == effective_txg	-p singlefs-harness --lib -- oracle_instance_tests	the_same_txg_from_two_instances_are_two_versions
 oracle 放过没版本的更新根	crates/singlefs-harness/src/crash.rs	                (candidate_version.checkpoint_txg, candidate_version.instance)\n                    < (effective_txg, effective_instance)	                false	-p singlefs-harness --lib -- oracle_instance_tests	newer_root_without_any_version_reporting_no_file_is_violation
 Ignore 那一遍恢复不过 oracle	crates/singlefs-harness/src/crash.rs	        tally.ignored_violations += 1;	        tally.ignored_violations += 0;	-p singlefs-harness --test second_transaction_step_zero_layer0 -- targeted_controls	targeted_controls_on_the_second_publish_go_red_where_they_should
-步 1 变异：不释放旧落点	crates/singlefs-core/src/transaction.rs	    let release = placements_to_release_via_mapping(previous, allocator)?;	    let release: Vec<Placement> = Vec::new();	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
+步 1 变异：不释放旧落点	crates/singlefs-core/src/transaction.rs	            placements_to_release_via_mapping(previous_version, allocator, &rewritten)?	            Vec::<Placement>::new()	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
 步 1 变异：defer 行写 0	crates/singlefs-core/src/transaction.rs	            device_map.deferred_slots() * SLOT_BYTES,	            0,	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
 步 1 变异：事务号不加一	crates/singlefs-core/src/transaction.rs	            transaction: previous.record.transaction + 1,	            transaction: previous.record.transaction,	-p singlefs-harness --test second_transaction_step_one_overwrite -- overwrite_publishes	overwrite_publishes_the_second_version_through_the_same_commit_shape
 步 1 变异：改动计数留 1	crates/singlefs-core/src/transaction.rs	            change_count: txg.0,	            change_count: 1,	-p singlefs-harness --test second_transaction_step_one_overwrite -- overwrite_publishes	overwrite_publishes_the_second_version_through_the_same_commit_shape
 步 2 变异：已分配行不随分配更新（I-3.1 要红）	crates/singlefs-core/src/transaction.rs	            device_map.allocated_slots() * SLOT_BYTES,	            0,	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
 步 2 变异：忘了改写分配记录	crates/singlefs-core/src/allocator.rs	            record.is_released = true;	            record.is_released = false;	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
+步 3：不写行（实例表照抄）	crates/singlefs-core/src/mount.rs	            instance_table: InstanceTablePlan::Rewrite(table_after.to_records()),	            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
+步 3：暖机只推一次	crates/singlefs-core/src/mount.rs	        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(warm_up_publishes.len()).expect("次数") < 1	-p singlefs-harness --test second_transaction_step_three_second_instance -- damaging_every_instance_two_root	damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device
+步 3：前缀跨实例边界（把别的实例的记录也接上）	crates/singlefs-core/src/recovery.rs	            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water	            (record.instance, record.checkpoint_txg) > water	-p singlefs-harness --test first_transaction_step_seven_layer0 -- layer0_partial_enumeration	layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape
+步 3：链首不接在所选根自己那条记录之后	crates/singlefs-core/src/recovery.rs	        root_own_record_counter.map(|counter| (root.instance, counter + 1));	        root_own_record_counter.map(|counter| (root.instance, counter + 1)).and(None);	-p singlefs-harness --test second_transaction_step_three_second_instance -- one_missing_record_right_after	one_missing_record_right_after_the_chosen_root_stops_the_prefix_even_when_later_records_are_intact
+步 3：I-3.8 判定恒真	crates/singlefs-checker/src/walk.rs	            unique && below_mount_root && chain_record_last,	            unique || below_mount_root || chain_record_last || true,	-p singlefs-harness --test second_transaction_step_three_second_instance -- checker_rejects_an_instance_table_row	checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root
+步 3：重建分配器时不把已释放的放进 defer 队列	crates/singlefs-core/src/allocator.rs	            if record.is_released {\n                device_map.mark_released(record.slot, span);\n            }	            if record.is_released && false {\n                device_map.mark_released(record.slot, span);\n            }	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index 6ec8977..7e4292e 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -33,10 +33,10 @@ pub enum InvariantVerdict {
 }
 
 /// 第一版 checker 判的不变量，按这个次序报；每次都全部报出来，没评估到的报「不适用」。
-pub const IMPLEMENTED_INVARIANTS: [&str; 23] = [
+pub const IMPLEMENTED_INVARIANTS: [&str; 24] = [
     "I-1.1", "I-1.3", "I-1.4", "I-1.6", "I-1.7", "I-2.1", "I-2.3", "I-2.4", "I-2.5", "I-3.1",
-    "I-5.1", "I-5.2", "I-7.1", "I-7.2", "I-7.6", "I-7.7", "I-7.8", "I-9.1", "I-9.2", "I-9.4",
-    "I-9.7", "I-9.10", "I-9.13",
+    "I-3.8", "I-5.1", "I-5.2", "I-7.1", "I-7.2", "I-7.6", "I-7.7", "I-7.8", "I-9.1", "I-9.2",
+    "I-9.4", "I-9.7", "I-9.10", "I-9.13",
 ];
 
 /// 判定累加器：每条不变量记评估了几次、第一处违例、以及整条不适用的理由。
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index d0b681c..92b2a44 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -209,7 +209,15 @@ impl Walk<'_> {
                 instance_pointer.locations[0].device,
                 instance_pointer.locations[0].slot,
             )) {
-                self.judge_packed_container(&unit, PACKED_TYPE_INSTANCE_TABLE, "实例表单元");
+                let records =
+                    self.judge_packed_container(&unit, PACKED_TYPE_INSTANCE_TABLE, "实例表单元");
+                if is_newest {
+                    if let Some(records) = records {
+                        let mount_root_instance =
+                            u32::from_le_bytes(record[24..28].try_into().expect("4 字节"));
+                        self.judge_instance_table_rows(&records, mount_root_instance);
+                    }
+                }
             }
         } else {
             self.walk_failures
@@ -397,6 +405,41 @@ impl Walk<'_> {
     }
 
     /// 码 3 容器：I-1.7（记录数 × 记录宽 ≤ 声明长度 ≤ 32768 − 136、记录宽 = 登记的宽）。返回记录，用不了返回 None。
+    /// I-3.8（实例表行唯一且低于挂载根）：`kind` 0 行按实例代号唯一、每行的实例代号 < 挂载根（最新根）的实例；
+    /// 链指针记录（`kind` 1）恒为一片的最后一条（D18（块里携带什么信息） 已定项 11）。「行只在回收条件成立后删」对着一个镜像判不了。
+    fn judge_instance_table_rows(&mut self, records: &[Vec<u8>], mount_root_instance: u32) {
+        let mut instances: Vec<u32> = Vec::new();
+        let mut unique = true;
+        let mut below_mount_root = true;
+        let mut chain_record_last = false;
+        for (index, row) in records.iter().enumerate() {
+            match row.first().copied() {
+                Some(0) => {
+                    let instance = u32::from_le_bytes(row[1..5].try_into().expect("4 字节"));
+                    if instances.contains(&instance) {
+                        unique = false;
+                    }
+                    instances.push(instance);
+                    if instance >= mount_root_instance {
+                        below_mount_root = false;
+                    }
+                    chain_record_last = false;
+                }
+                Some(1) => chain_record_last = index + 1 == records.len(),
+                _ => chain_record_last = false,
+            }
+        }
+        self.judgements.judge(
+            "I-3.8",
+            unique && below_mount_root && chain_record_last,
+            || {
+                format!(
+                    "实例表：行的实例代号 {instances:?}（唯一 = {unique}，都低于挂载根实例 {mount_root_instance} = {below_mount_root}），链指针记录在末尾 = {chain_record_last}"
+                )
+            },
+        );
+    }
+
     fn judge_packed_container(
         &mut self,
         unit: &[u8],
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 01e35e5..6858e09 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -282,6 +282,32 @@ impl PoolAllocator {
         }
     }
 
+    /// 重开时从盘上那棵分配记录树重建分配器（D23（journal 的角色与格式） 已定项 14：defer 队列、分配器游标、记账的现行值从所选根那棵账重新载入）：
+    /// 每条记录把自己那块盘上的槽标成占着，已释放的再进 defer 队列；空闲计数随 `mark_allocated` 减、与写路径同一条事件路径。
+    /// 开放段与 bump 游标只在内存、盘上没有（分配记录树不记它们），重开后按「没有开放段」起步：下一个提交内生块从最低的全空段开——
+    /// 上一段里没用完的槽先留着（C146（无空段时的回落政策全仓无定义） 之外的另一格，里程碑步 3 的决策点）。
+    #[must_use]
+    pub fn rebuild_from_records(
+        devices: Vec<DeviceFreeMap>,
+        records: Vec<AllocationRecord>,
+    ) -> Self {
+        let mut allocator = Self::new(devices);
+        for record in &records {
+            let device_map = allocator
+                .devices
+                .iter_mut()
+                .find(|device_map| device_map.device == record.device)
+                .expect("分配记录的盘在池里：走读逐盘核过");
+            let span = u64::from(record.span_slots);
+            device_map.mark_allocated(record.slot, span);
+            if record.is_released {
+                device_map.mark_released(record.slot, span);
+            }
+        }
+        allocator.records = records;
+        allocator
+    }
+
     /// mkfs 写在单元区里的两个单元：每盘一条分配代 0 的分配记录（字节表五：20 条里 m1 / m2 各两条，分配代 0）。
     /// 单元区之外的固定结构（超级块、根环、journal 环）不写分配记录、分配器不下探（D3（空间分配） 已定项 10 ④）。
     pub fn mark_format_time_units(&mut self, placements: &[Placement]) {
diff --git a/crates/singlefs-core/src/lib.rs b/crates/singlefs-core/src/lib.rs
index 58faddf..42f3e63 100644
--- a/crates/singlefs-core/src/lib.rs
+++ b/crates/singlefs-core/src/lib.rs
@@ -13,6 +13,7 @@ pub mod bytes;
 pub mod checksum;
 pub mod journal;
 pub mod make_filesystem;
+pub mod mount;
 pub mod pointer;
 pub mod records;
 pub mod recovery;
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 65d8491..5e63142 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -27,15 +27,15 @@ use crate::journal::JournalRecord;
 use crate::make_filesystem::TREE_TABLE_KEY_WIDTH;
 use crate::pointer::{DataPointer, LocationEntry, NodePointer};
 use crate::records::{
-    mapping_key_for_data, parse_extent_record, parse_inode_internal_entry, parse_mapping_entry,
-    AccountingEntry, InodeRecord, TreeTableEntry, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION,
-    TREE_KIND_DEADLIST, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LIVELIST,
-    TREE_KIND_SPARSE_SIDE_TABLE,
+    mapping_key_for_data, mapping_key_for_node, parse_extent_record, parse_inode_internal_entry,
+    parse_mapping_entry, AccountingEntry, InodeRecord, TreeTableEntry, TREE_KIND_ACCOUNTING,
+    TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_EXTENT, TREE_KIND_INODE,
+    TREE_KIND_LIVELIST, TREE_KIND_SPARSE_SIDE_TABLE,
 };
 use crate::root_record::RootRecord;
 use crate::root_ring::{slot_offset, RootRingSlot};
 use crate::superblock::{FormatTimeGeometry, Superblock};
-use crate::transaction::FIRST_INODE_NUMBER;
+use crate::transaction::{PublishedUnit, TransactionOutput, TransactionUnit, FIRST_INODE_NUMBER};
 use crate::unit::{
     data_unit_payload, parse_data_unit, parse_index_node, parse_packed_unit,
     unit_filesystem_identifier, IndexNodeHeader, PACKED_TYPE_INODE, PACKED_TYPE_INSTANCE_TABLE,
@@ -156,6 +156,9 @@ pub struct JournalScanReport {
     pub prefix_applied: usize,
     pub verification_passed: usize,
     pub verification_failed: usize,
+    /// 这次恢复施加的记录里最大的事务号（空发布的事务号 0 不进 max，D23（journal 的角色与格式） 已定项 19 ①）；
+    /// 可写挂载给上一个实例写行时 W 取它（D18（块里携带什么信息） 已定项 11 的行记录）。
+    pub maximum_applied_transaction: u64,
 }
 
 #[derive(Clone, Debug, PartialEq, Eq)]
@@ -297,6 +300,226 @@ pub fn choose_root(reader: &dyn PoolReader, superblock: &Superblock) -> Option<R
     best
 }
 
+/// 根环全部自证过的根里最大的 checkpoint_txg：新实例的第一次发布取 max(它, 环里全部自证通过的记录的 checkpoint_txg) + 1
+/// （D23（journal 的角色与格式） 已定项 14 第 3 条）；一条都没有时 None。
+#[must_use]
+pub fn highest_root_txg<Reader: PoolReader + ?Sized>(
+    reader: &Reader,
+    region_devices: &[DeviceIdentity; 3],
+    geometry: &FormatTimeGeometry,
+    filesystem_identifier: &[u8; 16],
+) -> Option<CheckpointTxg> {
+    let mut highest: Option<CheckpointTxg> = None;
+    visit_valid_roots(
+        reader,
+        region_devices,
+        geometry,
+        filesystem_identifier,
+        |root| {
+            highest = Some(highest.map_or(root.checkpoint_txg, |current| {
+                current.max(root.checkpoint_txg)
+            }));
+        },
+    );
+    highest
+}
+
+/// 从盘上按所选根重建「上一版」：全部角色的单元字节、指针、树表、分配记录、记账行与 inode 记录，交给发布路径当上一版
+/// （照抄没重写的角色、经映射释放被换下的角色都靠它；可写挂载在恢复之后调）。`record` 是所选根自己那条记录（读不出时由调用方顶一条）。
+///
+/// # Errors
+/// 树表为空（所选根下面还没有文件版本）⇒ `InvariantViolated { invariant: "挂载", … }`；单元读不到或解不开 ⇒ 走读同款的错。
+pub fn rebuild_version(
+    reader: &dyn PoolReader,
+    root: &RootRecord,
+    record: JournalRecord,
+    record_bytes: Vec<u8>,
+) -> Result<TransactionOutput, RecoveryFailure> {
+    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
+    let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
+    let instance_table_bytes =
+        read_unit_via_locations(reader, &root.instance_table.locations, data_bytes)?;
+    let tree_table_bytes = read_unit_via_locations(reader, &root.tree_table.locations, node_bytes)?;
+    let tree_table =
+        parse_index_node(&tree_table_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: "树表单元",
+        })?;
+    if tree_table.entries.is_empty() {
+        return Err(RecoveryFailure::InvariantViolated {
+            invariant: "挂载",
+            detail: "所选根下面还没有文件版本（树表为空）",
+        });
+    }
+    let mut tree_table_entries = Vec::with_capacity(tree_table.entries.len());
+    for bytes in &tree_table.entries {
+        tree_table_entries.push(TreeTableEntry::parse(bytes).ok_or(
+            RecoveryFailure::UnitMalformed {
+                what: "树表条目"
+            },
+        )?);
+    }
+    let pointer_of = |kind: u16| {
+        tree_table_entries
+            .iter()
+            .find(|entry| entry.kind == kind)
+            .map(|entry| entry.root)
+            .ok_or(RecoveryFailure::UnitMalformed {
+                what: "树表里没有那棵树",
+            })
+    };
+    let extent_pointer = pointer_of(TREE_KIND_EXTENT)?;
+    let inode_root_pointer = pointer_of(TREE_KIND_INODE)?;
+    let allocation_pointer = pointer_of(TREE_KIND_ALLOCATION)?;
+    let accounting_pointer = pointer_of(TREE_KIND_ACCOUNTING)?;
+    let read_node = |pointer: &NodePointer, what: &'static str| {
+        let bytes = read_unit_via_locations(reader, &pointer.locations, node_bytes)?;
+        let node =
+            parse_index_node(&bytes).map_err(|_error| RecoveryFailure::UnitMalformed { what })?;
+        Ok::<(Vec<u8>, IndexNodeHeader), RecoveryFailure>((bytes, node))
+    };
+    let (extent_bytes, extent_node) = read_node(&extent_pointer, "extent 树根")?;
+    let (_, data_pointer) = parse_extent_record(extent_node.entries.first().ok_or(
+        RecoveryFailure::UnitMalformed {
+            what: "extent 树根没有记录",
+        },
+    )?);
+    let data_unit_bytes = read_unit_via_locations(reader, &data_pointer.locations, data_bytes)?;
+    let (inode_root_bytes, inode_root_node) = read_node(&inode_root_pointer, "inode 树根")?;
+    let (_, _, inode_leaf_pointer) =
+        parse_inode_internal_entry(inode_root_node.entries.first().ok_or(
+            RecoveryFailure::UnitMalformed {
+                what: "inode 树根没有条目",
+            },
+        )?);
+    let inode_leaf_bytes =
+        read_unit_via_locations(reader, &inode_leaf_pointer.locations, data_bytes)?;
+    let inode_leaf =
+        parse_packed_unit(&inode_leaf_bytes).map_err(|_error| RecoveryFailure::UnitMalformed {
+            what: "inode 叶容器",
+        })?;
+    let inode_record = InodeRecord::parse(inode_leaf.records.first().ok_or(
+        RecoveryFailure::UnitMalformed {
+            what: "inode 叶容器没有记录",
+        },
+    )?)
+    .ok_or(RecoveryFailure::UnitMalformed {
+        what: "inode 记录"
+    })?;
+    let (allocation_bytes, allocation_node) = read_node(&allocation_pointer, "分配记录树根")?;
+    let allocation_records: Vec<AllocationRecord> = allocation_node
+        .entries
+        .iter()
+        .map(|bytes| AllocationRecord::parse(bytes))
+        .collect();
+    let (accounting_bytes, accounting_node) = read_node(&accounting_pointer, "记账树根")?;
+    let accounting_entries: Vec<AccountingEntry> = accounting_node
+        .entries
+        .iter()
+        .map(|bytes| AccountingEntry::parse(bytes))
+        .collect();
+    let (mapping_bytes, mapping_node) = read_node(&root.mapping_root, "映射树根")?;
+    let mapping_keys: Vec<Vec<u8>> = mapping_node
+        .entries
+        .iter()
+        .map(|entry| parse_mapping_entry(entry).0)
+        .collect();
+    let mapped_units = vec![
+        (
+            TransactionUnit::Data,
+            mapping_key_for_data(data_pointer.head, data_pointer.write_order),
+        ),
+        (
+            TransactionUnit::ExtentRoot,
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, extent_pointer),
+        ),
+        (
+            TransactionUnit::InodeLeaf,
+            mapping_key_for_node(UNIT_CLASS_PACKED, inode_leaf_pointer),
+        ),
+        (
+            TransactionUnit::InodeRoot,
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, inode_root_pointer),
+        ),
+        (
+            TransactionUnit::AllocationTree,
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, allocation_pointer),
+        ),
+        (
+            TransactionUnit::AccountingTree,
+            mapping_key_for_node(UNIT_CLASS_INDEX_NODE, accounting_pointer),
+        ),
+    ];
+    let unit =
+        |identity: TransactionUnit, locations: &[LocationEntry; 2], bytes: Vec<u8>| PublishedUnit {
+            slot: locations[0].slot,
+            identity,
+            bytes,
+        };
+    let units = vec![
+        unit(
+            TransactionUnit::Data,
+            &data_pointer.locations,
+            data_unit_bytes,
+        ),
+        unit(
+            TransactionUnit::ExtentRoot,
+            &extent_pointer.locations,
+            extent_bytes,
+        ),
+        unit(
+            TransactionUnit::InodeLeaf,
+            &inode_leaf_pointer.locations,
+            inode_leaf_bytes,
+        ),
+        unit(
+            TransactionUnit::InodeRoot,
+            &inode_root_pointer.locations,
+            inode_root_bytes,
+        ),
+        unit(
+            TransactionUnit::AllocationTree,
+            &allocation_pointer.locations,
+            allocation_bytes,
+        ),
+        unit(
+            TransactionUnit::AccountingTree,
+            &accounting_pointer.locations,
+            accounting_bytes,
+        ),
+        unit(
+            TransactionUnit::MappingTree,
+            &root.mapping_root.locations,
+            mapping_bytes,
+        ),
+        unit(
+            TransactionUnit::TreeTable,
+            &root.tree_table.locations,
+            tree_table_bytes,
+        ),
+        unit(
+            TransactionUnit::InstanceTable,
+            &root.instance_table.locations,
+            instance_table_bytes,
+        ),
+    ];
+    Ok(TransactionOutput {
+        root: *root,
+        record,
+        record_bytes,
+        units,
+        rewritten: Vec::new(),
+        data_pointer,
+        mapping_keys,
+        allocation_records,
+        accounting_entries,
+        tree_table_entries,
+        inode_record,
+        mapped_units,
+        released: Vec::new(),
+        key_order_mismatches: 0,
+    })
+}
+
 /// 根环全部自证过的根里最大的实例代号（取号的 max 里「根环里全部根记录的实例代号」那一半）；一条都没有时 None。
 #[must_use]
 pub fn highest_root_instance<Reader: PoolReader + ?Sized>(
@@ -386,18 +609,32 @@ pub fn replay_journal(
         prefix_applied: 0,
         verification_passed: 0,
         verification_failed: 0,
+        maximum_applied_transaction: 0,
     };
     let water = (root.instance, root.checkpoint_txg);
     let mut rebuilt = *root;
+    // 前缀规则不跨实例边界（D23（journal 的角色与格式） 已定项 14 第 1 条）：链从所选根覆盖的最后一条记录之后接，
+    // 下一条的实例代号与所选根不同即停——所以只有所选根自己那个实例的记录是候选；所选根是 mkfs 的第 0 代根时一条都不施加。
     let mut above: Vec<&JournalRecord> = records
         .values()
-        .filter(|record| (record.instance, record.checkpoint_txg) > water)
+        .filter(|record| {
+            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water
+        })
         .collect();
     above.sort_by_key(|record| (record.instance, record.counter));
     report.above_water = above.len();
     let in_flight_limit =
         usize::try_from(journal_in_flight_record_limit(ring_bytes)).expect("在飞上限");
-    let mut expected_next: Option<(InstanceGeneration, u64)> = None;
+    // 链首接在所选根自己那条记录（同实例、checkpoint_txg 相等）之后；那条记录读不出（两份都撕了）时不知道它的 jsn，
+    // 就从水位之上同实例最小的那条接——水位之下的记录都已经在根里，读不读得出不影响链。
+    let root_own_record_counter = records
+        .values()
+        .find(|record| {
+            record.instance == root.instance && record.checkpoint_txg == root.checkpoint_txg
+        })
+        .map(|record| record.counter);
+    let mut expected_next: Option<(InstanceGeneration, u64)> =
+        root_own_record_counter.map(|counter| (root.instance, counter + 1));
     for record in above.into_iter().take(in_flight_limit) {
         if let Some(expected_key) = expected_next {
             if (record.instance, record.counter) != expected_key {
@@ -429,6 +666,8 @@ pub fn replay_journal(
         }
         report.verification_passed += 1;
         report.prefix_applied += 1;
+        report.maximum_applied_transaction =
+            report.maximum_applied_transaction.max(record.transaction);
         rebuilt = RootRecord {
             filesystem_identifier: rebuilt.filesystem_identifier,
             instance: record.instance,
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 7eef470..97cf106 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -12,11 +12,12 @@ use std::collections::BTreeMap;
 use singlefs_format::{
     ACCOUNTING_ENTRY_BYTES, ACCOUNTING_KEY_BYTES, ALLOCATION_RECORD_BYTES,
     ALLOCATION_RECORD_KEY_BYTES, EXTENT_KEY_BYTES, EXTENT_LEAF_RECORD_BYTES, FIRST_TRANSACTION_TXG,
-    INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES, MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES, SLOT_BYTES,
-    SUPERBLOCK_SLOTS_PER_DEVICE, TREE_IDENTIFIER_ACCOUNTING, TREE_IDENTIFIER_ALLOCATION_RECORDS,
-    TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST, TREE_IDENTIFIER_EXTENT,
-    TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LIVELIST, TREE_IDENTIFIER_SPARSE_SIDE_TABLE,
-    TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH, TREE_TABLE_ENTRY_BYTES, WARM_UP_EMPTY_PUBLISHES,
+    INODE_INTERNAL_ENTRY, INODE_RECORD_BYTES, INSTANCE_ROW_BYTES, MAPPING_ENTRY_BYTES,
+    MAPPING_KEY_BYTES, SLOT_BYTES, SUPERBLOCK_SLOTS_PER_DEVICE, TREE_IDENTIFIER_ACCOUNTING,
+    TREE_IDENTIFIER_ALLOCATION_RECORDS, TREE_IDENTIFIER_CENTRAL_MAPPING, TREE_IDENTIFIER_DEADLIST,
+    TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, TREE_IDENTIFIER_LIVELIST,
+    TREE_IDENTIFIER_SPARSE_SIDE_TABLE, TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
+    TREE_TABLE_ENTRY_BYTES, WARM_UP_EMPTY_PUBLISHES,
 };
 
 use crate::address::{
@@ -33,7 +34,7 @@ use crate::pointer::{BirthSequence, DataPointer, LocationEntry, NodePointer, Poi
 use crate::records::{
     build_extent_record, build_inode_internal_entry, build_mapping_entry, data_key_tail,
     mapping_key_for_data, mapping_key_for_node, mapping_key_sort_key, node_key_tail,
-    parse_mapping_entry, AccountingEntry, InodeRecord, TreeTableEntry,
+    parse_inode_internal_entry, parse_mapping_entry, AccountingEntry, InodeRecord, TreeTableEntry,
     ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF, STATISTIC_ALLOCATED_BYTES,
     STATISTIC_COMMITTED_RESERVATION_BYTES, STATISTIC_DEFER_QUEUE_BYTES,
     STATISTIC_EMPTY_CLUSTER_SEGMENTS, STATISTIC_FRAGMENTATION_RUNS, STATISTIC_FREE_BYTES,
@@ -48,8 +49,8 @@ use crate::superblock::Superblock;
 use crate::unit::{
     build_data_unit, build_index_node, build_packed_unit, data_unit_payload_capacity,
     index_node_entry_capacity, parse_index_node, unit_filesystem_identifier, DataUnitIdentity,
-    PackedIdentity, WriteOrder, PACKED_TYPE_INODE, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE,
-    UNIT_CLASS_PACKED,
+    PackedIdentity, WriteOrder, PACKED_TYPE_INODE, PACKED_TYPE_INSTANCE_TABLE, UNIT_CLASS_DATA,
+    UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
 };
 
 /// 第一个文件的 inode 号（里程碑步 3 预想）。
@@ -392,6 +393,9 @@ pub enum TransactionUnit {
     AccountingTree,
     MappingTree,
     TreeTable,
+    /// 实例表单元（码 3 打包记录类型 4）：mkfs 种下第一片，之后每次可写挂载写行时重写（D18（块里携带什么信息） 已定项 11）；
+    /// 不在第一个事务的八个角色里，第二个事务起才进发布路径。
+    InstanceTable,
 }
 
 impl TransactionUnit {
@@ -428,6 +432,7 @@ impl TransactionUnit {
             TransactionUnit::AccountingTree => "t6",
             TransactionUnit::MappingTree => "t7",
             TransactionUnit::TreeTable => "t8",
+            TransactionUnit::InstanceTable => "ti",
         }
     }
 
@@ -435,7 +440,7 @@ impl TransactionUnit {
     pub const fn unit_class(self) -> u8 {
         match self {
             TransactionUnit::Data => UNIT_CLASS_DATA,
-            TransactionUnit::InodeLeaf => UNIT_CLASS_PACKED,
+            TransactionUnit::InodeLeaf | TransactionUnit::InstanceTable => UNIT_CLASS_PACKED,
             TransactionUnit::ExtentRoot
             | TransactionUnit::InodeRoot
             | TransactionUnit::AllocationTree
@@ -458,7 +463,9 @@ impl TransactionUnit {
             TransactionUnit::AllocationTree => TreeIdentifier(TREE_IDENTIFIER_ALLOCATION_RECORDS),
             TransactionUnit::AccountingTree => TreeIdentifier(TREE_IDENTIFIER_ACCOUNTING),
             TransactionUnit::MappingTree => TreeIdentifier(TREE_IDENTIFIER_CENTRAL_MAPPING),
-            TransactionUnit::TreeTable => TreeIdentifier(TREE_IDENTIFIER_NONE),
+            TransactionUnit::TreeTable | TransactionUnit::InstanceTable => {
+                TreeIdentifier(TREE_IDENTIFIER_NONE)
+            }
         }
     }
 
@@ -467,7 +474,7 @@ impl TransactionUnit {
     pub const fn placement(self) -> PlacementRule {
         match self {
             TransactionUnit::Data => PlacementRule::UserData,
-            TransactionUnit::InodeLeaf => {
+            TransactionUnit::InodeLeaf | TransactionUnit::InstanceTable => {
                 PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned)
             }
             TransactionUnit::ExtentRoot
@@ -500,7 +507,10 @@ pub struct TransactionOutput {
     pub root: RootRecord,
     pub record: JournalRecord,
     pub record_bytes: Vec<u8>,
+    /// 这一版全部角色的单元：这次重写的是新装的，没重写的从上一版照抄（八个文件 / 固定点角色按 bump 次序，实例表单元在末尾、mkfs 之后第一次重写之前不在）。
     pub units: Vec<PublishedUnit>,
+    /// 这次发布真正写出的角色，按写出的次序（点名项与录制流里的单元写只有这些）。
+    pub rewritten: Vec<TransactionUnit>,
     pub data_pointer: DataPointer,
     pub mapping_keys: Vec<Vec<u8>>,
     pub allocation_records: Vec<AllocationRecord>,
@@ -522,10 +532,29 @@ impl TransactionOutput {
         self.units
             .iter()
             .find(|unit| unit.identity == identity)
-            .expect("八个单元每种一个")
+            .expect("八个文件 / 固定点角色每种一个；实例表单元要先重写过一次才在")
+    }
+
+    /// 树表里某棵树的根指针（照抄没重写的角色时用）。
+    #[must_use]
+    pub fn tree_root_pointer(&self, tree: u64) -> NodePointer {
+        self.tree_table_entries
+            .iter()
+            .find(|entry| entry.tree == TreeIdentifier(tree))
+            .expect("树表里每棵登记的树一条")
+            .root
     }
 
-    /// 这次发布写出的八个落点，按写者自己记的槽号（位置提示那一路）。释放不走它、走 `placements_to_release_via_mapping`；
+    /// 树表条目的诞生 txg（七条同一个数：树建起来那次发布）。
+    #[must_use]
+    pub fn tree_birth_txg(&self) -> CheckpointTxg {
+        self.tree_table_entries
+            .first()
+            .expect("树表第 1 版起恒有七条")
+            .birth_txg
+    }
+
+    /// 这一版全部角色的落点，按写者自己记的槽号（位置提示那一路）。释放不走它、走 `placements_to_release_via_mapping`；
     /// 留着给验收拿两条路互相对。
     #[must_use]
     pub fn placements(&self) -> Vec<Placement> {
@@ -551,10 +580,11 @@ impl TransactionOutput {
 pub fn placements_to_release_via_mapping(
     previous: &TransactionOutput,
     allocator: &PoolAllocator,
+    roles: &[TransactionUnit],
 ) -> Result<Vec<Placement>, PublishError> {
     let mapping_node_bytes = &previous.unit(TransactionUnit::MappingTree).bytes;
     let mut placements = Vec::new();
-    for identity in TransactionUnit::IN_BUMP_ORDER {
+    for identity in roles.iter().copied() {
         let locations = match identity {
             TransactionUnit::Data
             | TransactionUnit::ExtentRoot
@@ -572,6 +602,7 @@ pub fn placements_to_release_via_mapping(
             }
             TransactionUnit::MappingTree => previous.root.mapping_root.locations,
             TransactionUnit::TreeTable => previous.root.tree_table.locations,
+            TransactionUnit::InstanceTable => previous.root.instance_table.locations,
         };
         assert_eq!(
             locations[0].slot, locations[1].slot,
@@ -713,24 +744,71 @@ pub struct FirstFile<'content> {
     pub write_time_seconds: u64,
 }
 
-/// 一次「文件的一个版本」的发布要的全部参数：第一个事务与覆盖写只差这些数（都是发布参数，不取系统时钟）。
+/// 这次发布要写的文件版本：四个文件角色（数据、extent 根、inode 叶、inode 根）全部重写。没有它的发布（写行、暖机）文件角色照抄上一版。
 #[derive(Clone, Copy, Debug)]
-pub struct FilePublish<'content> {
-    pub txg: CheckpointTxg,
-    /// jsn 计数器（记录落在环里的槽位）。
-    pub counter: u64,
-    /// 事务号，按实例计数从 1 起（D23（journal 的角色与格式） 已定项 7）。
-    pub transaction: u64,
+pub struct FileVersionPlan<'content> {
     pub content: &'content [u8],
     pub write_time_seconds: u64,
     /// 对象出生代 = 创建那次发布的 checkpoint_txg，覆盖写不改（D8（核心索引结构） 已定项 6；I-9.10（对象出生代与 inode 记录相符））。
     pub inode_object_birth: CheckpointTxg,
     /// 改动计数（D8（核心索引结构） 已定项 6 偏移 88）。
     pub change_count: u64,
-    /// 树表条目的诞生 txg：树建起来那次发布，覆盖写不改。
+}
+
+/// 实例表单元这次发布怎么处理：根记录照抄上一版的指针，或者重写成给定的记录（行记录在前、链指针记录最末，D18（块里携带什么信息） 已定项 11）。
+#[derive(Clone, Debug)]
+pub enum InstanceTablePlan {
+    Carry(NodePointer),
+    Rewrite(Vec<Vec<u8>>),
+}
+
+/// 一次发布的全部参数：哪些角色重写、身份字段取什么。第一个事务、覆盖写、写行、暖机都是它的一种取值，走同一条发布路径
+/// （`.claude/rules/fs-design.md`「一个事务层，所有结构共用」）。
+#[derive(Clone, Debug)]
+pub struct PublishPlan<'content> {
+    pub txg: CheckpointTxg,
+    /// jsn 计数器（记录落在环里的槽位），全池接着走、换实例不归零（D23（journal 的角色与格式） 已定项 14 第 3 条）。
+    pub counter: u64,
+    /// 事务号，按实例计数从 1 起，空发布写 0（D23（journal 的角色与格式） 已定项 7 / 已定项 19 ①）。
+    pub transaction: u64,
+    pub instance: InstanceGeneration,
+    /// 反向链：上一条记录头的 CRC；本实例的第一条恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
+    pub back_chain: u32,
+    pub file: Option<FileVersionPlan<'content>>,
+    pub instance_table: InstanceTablePlan,
+    /// 树表条目的诞生 txg：树建起来那次发布，之后每一版重写都不改。
     pub tree_birth_txg: CheckpointTxg,
-    /// 根记录照旧持有的实例表单元指针。
-    pub instance_table: NodePointer,
+    pub tree_identifier_watermark: u64,
+    pub rollback_floor: CheckpointTxg,
+}
+
+impl PublishPlan<'_> {
+    /// 这次发布重写的角色，按 bump 次序：用户数据先取（自己的政策）；提交内生块按树 ID 升序、树内先叶后根、映射树倒数第二、树表最末
+    /// （D3（空间分配） 已定项 10 ⑤）。实例表单元归树 0，排在全部提交内生块之前（mkfs 也是先实例表后树表；里程碑步 3 的决策点）。
+    #[must_use]
+    pub fn rewritten_roles(&self) -> Vec<TransactionUnit> {
+        let mut roles = Vec::new();
+        if self.file.is_some() {
+            roles.push(TransactionUnit::Data);
+        }
+        if matches!(self.instance_table, InstanceTablePlan::Rewrite(_)) {
+            roles.push(TransactionUnit::InstanceTable);
+        }
+        if self.file.is_some() {
+            roles.extend([
+                TransactionUnit::ExtentRoot,
+                TransactionUnit::InodeLeaf,
+                TransactionUnit::InodeRoot,
+            ]);
+        }
+        roles.extend([
+            TransactionUnit::AllocationTree,
+            TransactionUnit::AccountingTree,
+            TransactionUnit::MappingTree,
+            TransactionUnit::TreeTable,
+        ]);
+        roles
+    }
 }
 
 /// 第一个事务（字节表七 t1..t8 + 六 + 七）：分配八个落点、装八个单元、按 D16（发布语义） 已定项 7 的持久顺序落盘。
@@ -743,30 +821,35 @@ pub fn publish_first_file<Device: BlockDevice>(
     previous_record_bytes: &[u8],
 ) -> Result<TransactionOutput, PublishError> {
     let txg = CheckpointTxg(FIRST_TRANSACTION_TXG);
-    publish_file_version(
+    publish_version(
         pool,
         allocator,
-        FilePublish {
+        PublishPlan {
             txg,
             counter: FIRST_TRANSACTION_TXG,
             transaction: FIRST_TRANSACTION_NUMBER,
-            content: file.content,
-            write_time_seconds: file.write_time_seconds,
-            inode_object_birth: txg,
-            change_count: 1,
+            instance,
+            back_chain: back_chain_of(previous_record_bytes),
+            file: Some(FileVersionPlan {
+                content: file.content,
+                write_time_seconds: file.write_time_seconds,
+                inode_object_birth: txg,
+                change_count: 1,
+            }),
+            instance_table: InstanceTablePlan::Carry(genesis.instance_table),
             tree_birth_txg: txg,
-            instance_table: genesis.instance_table,
+            tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
+            rollback_floor: CheckpointTxg(0),
         },
-        instance,
-        previous_record_bytes,
-        &[],
+        None,
     )
 }
 
-/// 覆盖写（里程碑「第二个事务」步 1 / 步 2）：同一个实例里紧接着上一次发布，把同一个文件的内容整个换掉——
-/// 新数据单元 COW 到新落点、extent 叶记录的指针换成它、inode 记录更新（改动计数 = 这次发布的 checkpoint_txg），
-/// 六个提交内生块与树表单元各 COW 出新版本；上一版的八个单元在同一次发布里释放（释放代 = 这次的 txg）。
-/// txg、jsn、事务号各比上一次加一。
+/// 覆盖写（里程碑「第二个事务」步 1 / 步 2）：同一个实例里接在上一次发布之后再发布一版同一个文件——txg、jsn、事务号各加一，
+/// 对象出生代与容器身份不改，改动计数取这次的 txg；上一版的八个落点经映射释放（进 defer 队列）。
+///
+/// # Errors
+/// 释放判定路径查不到上一版的某个单元（`ReleaseNotInMapping` 一族）、空间不够、装不下、块设备报错，都原样交回。
 pub fn publish_overwrite<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
@@ -774,60 +857,65 @@ pub fn publish_overwrite<Device: BlockDevice>(
     file: FirstFile<'_>,
     instance: InstanceGeneration,
 ) -> Result<TransactionOutput, PublishError> {
-    assert_eq!(
-        previous.root.instance, instance,
-        "覆盖写接在同一个实例的上一次发布之后"
-    );
     let txg = CheckpointTxg(previous.root.checkpoint_txg.0 + 1);
-    let tree_birth_txg = previous
-        .tree_table_entries
-        .first()
-        .expect("树表第 1 版有七条")
-        .birth_txg;
-    let release = placements_to_release_via_mapping(previous, allocator)?;
-    publish_file_version(
+    publish_version(
         pool,
         allocator,
-        FilePublish {
+        PublishPlan {
             txg,
             counter: previous.record.counter + 1,
             transaction: previous.record.transaction + 1,
-            content: file.content,
-            write_time_seconds: file.write_time_seconds,
-            inode_object_birth: previous.inode_record.object_birth,
-            change_count: txg.0,
-            tree_birth_txg,
-            instance_table: previous.root.instance_table,
+            instance,
+            back_chain: back_chain_of(&previous.record_bytes),
+            file: Some(FileVersionPlan {
+                content: file.content,
+                write_time_seconds: file.write_time_seconds,
+                inode_object_birth: previous.inode_record.object_birth,
+                change_count: txg.0,
+            }),
+            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
+            tree_birth_txg: previous.tree_birth_txg(),
+            tree_identifier_watermark: previous.root.tree_identifier_watermark,
+            rollback_floor: previous.root.rollback_floor,
         },
-        instance,
-        &previous.record_bytes,
-        &release,
+        Some(previous),
     )
 }
 
-/// 发布一个文件版本：先做准入（内容装得进一个数据单元、分配记录树装得下这次的记录），再释放上一版的落点、分配八个落点、
-/// 装八个单元、按 D16（发布语义） 已定项 7 的持久顺序落盘。准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立，
-/// 释放与分配都不算数（第二轮攻方腿：`NoSpaceFor` 在释放之后、分配到一半才返回，留下半新的池，拿同一个上一版重试撞断言）。
-fn publish_file_version<Device: BlockDevice>(
+/// 发布一版：先做准入（内容装得进一个数据单元、分配记录树装得下这次的记录），再释放上一版被换下的角色的落点、分配、装单元、
+/// 按 D16（发布语义） 已定项 7 的持久顺序落盘。准入之后任何一步失败，分配器退回到进来时的样子——这次发布没有成立，
+/// 释放与分配都不算数（第二轮攻方腿：`NoSpaceFor` 在释放之后、分配到一半返回，留下半新的池，拿同一个上一版重试撞断言）。
+///
+/// # Errors
+/// `ContentExceedsDataUnit`、`AllocationRecordsExceedOneNode`、释放判定路径的四种错、`NoSpaceFor`、块设备错。
+pub fn publish_version<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
-    publish: FilePublish<'_>,
-    instance: InstanceGeneration,
-    previous_record_bytes: &[u8],
-    release: &[Placement],
+    plan: PublishPlan<'_>,
+    previous: Option<&TransactionOutput>,
 ) -> Result<TransactionOutput, PublishError> {
+    let rewritten = plan.rewritten_roles();
+    // 释放判定路径先于准入：它只查不改（D19（块指针的结构与宽度预算） 已定项 5 第 1 条），查不到就整次发布不做。
+    let release = match previous {
+        Some(previous_version) => {
+            placements_to_release_via_mapping(previous_version, allocator, &rewritten)?
+        }
+        None => Vec::new(),
+    };
     // 用户给的内容装不进一个数据单元是调用方能恢复的失败，不是不变量被破坏：报错，不走到 build_data_unit 的断言。
-    let data_unit_capacity = data_unit_payload_capacity();
-    if publish.content.len() > data_unit_capacity {
-        return Err(PublishError::ContentExceedsDataUnit {
-            bytes: publish.content.len(),
-            capacity: data_unit_capacity,
-        });
+    if let Some(file) = &plan.file {
+        let data_unit_capacity = data_unit_payload_capacity();
+        if file.content.len() > data_unit_capacity {
+            return Err(PublishError::ContentExceedsDataUnit {
+                bytes: file.content.len(),
+                capacity: data_unit_capacity,
+            });
+        }
     }
     // 分配记录树第一版只有一个节点：这次发布之后装不下就在动分配器之前报错，不许走到 build_index_node 的断言
-    // （三方代码第一轮攻方腿：第 50 次覆盖写 panic）。释放只改写记录、不加；这次的八个落点每盘各加一条。
+    // （三方代码第一轮攻方腿：第 50 次覆盖写 panic）。释放只改写记录、不加；这次重写的每个角色每盘各加一条。
     let records_after_this_publish =
-        allocator.records().len() + TransactionUnit::IN_BUMP_ORDER.len() * allocator.devices.len();
+        allocator.records().len() + rewritten.len() * allocator.devices.len();
     let allocation_node_capacity = index_node_entry_capacity(
         usize::try_from(ALLOCATION_RECORD_KEY_BYTES).expect("10"),
         usize::try_from(ALLOCATION_RECORD_BYTES).expect("20"),
@@ -840,37 +928,36 @@ fn publish_file_version<Device: BlockDevice>(
     }
     // 整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图），换来失败路径不留半新的池。
     let allocator_before_this_publish = allocator.clone();
-    let outcome = publish_admitted_file_version(
-        pool,
-        allocator,
-        publish,
-        instance,
-        previous_record_bytes,
-        release,
-    );
+    let outcome = publish_admitted(pool, allocator, &plan, previous, &rewritten, &release);
     if outcome.is_err() {
         *allocator = allocator_before_this_publish;
     }
     outcome
 }
 
+/// 上一版里某个角色的单元字节（照抄进这一版的 `units`）。
+fn carried_unit(previous: &TransactionOutput, identity: TransactionUnit) -> PublishedUnit {
+    previous.unit(identity).clone()
+}
+
 /// 准入之后的那一段：释放、分配、装单元、落盘。失败时分配器由调用方退回，这里不管。
 #[allow(
     clippy::too_many_lines,
-    reason = "一次发布就是一件能单独验证的事：八个单元的装法与一条持久顺序，拆开只会把顺序藏进几个函数"
+    reason = "一次发布就是一件能单独验证的事：九个角色的装法与一条持久顺序，拆开只会把顺序藏进几个函数"
 )]
-fn publish_admitted_file_version<Device: BlockDevice>(
+fn publish_admitted<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
-    publish: FilePublish<'_>,
-    instance: InstanceGeneration,
-    previous_record_bytes: &[u8],
+    plan: &PublishPlan<'_>,
+    previous: Option<&TransactionOutput>,
+    rewritten: &[TransactionUnit],
     release: &[Placement],
 ) -> Result<TransactionOutput, PublishError> {
-    let txg = publish.txg;
+    let txg = plan.txg;
+    let instance = plan.instance;
     let write_order = WriteOrder {
         instance,
-        transaction: publish.transaction,
+        transaction: plan.transaction,
     };
     let filesystem_identifier = &pool.parameters.filesystem_identifier;
     let mut sequences = BirthSequenceAllocator::default();
@@ -880,121 +967,227 @@ fn publish_admitted_file_version<Device: BlockDevice>(
         allocator.release(*placement, txg);
     }
 
-    // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以八个落点在装任何单元之前全部取定。
+    // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。
     let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
-    for identity in TransactionUnit::IN_BUMP_ORDER {
+    for identity in rewritten {
         let placement = match identity.placement() {
             PlacementRule::UserData => allocator.allocate_user_data(txg),
             PlacementRule::CommitGenerated(footprint) => {
                 allocator.allocate_commit_generated(footprint, txg)
             }
         };
-        let placement = placement.ok_or(PublishError::NoSpaceFor { unit: identity })?;
-        slots.insert(identity, placement.slot);
+        let placement = placement.ok_or(PublishError::NoSpaceFor { unit: *identity })?;
+        slots.insert(*identity, placement.slot);
     }
     let slot_of = |identity: TransactionUnit| slots[&identity];
 
-    // t1 数据单元（字节表二）。
-    let data_identity = DataUnitIdentity {
-        tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
-        object: FIRST_INODE_NUMBER,
-        object_birth: publish.inode_object_birth,
-        anchor_offset: 0,
-    };
-    let data_unit = build_data_unit(
-        data_identity,
-        txg,
-        filesystem_identifier,
-        write_order,
-        publish.content,
-    );
-    let data_pointer = DataPointer {
-        head: PointerHead {
-            birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
-            birth_txg: txg,
-        },
-        locations: pool.location_entries(slot_of(TransactionUnit::Data), &data_unit),
-        write_order,
-    };
-    let extent_record = build_extent_record(FIRST_INODE_NUMBER, 0, data_pointer);
-    let extent_key: [u8; 24] = extent_record[..usize::try_from(EXTENT_KEY_BYTES).expect("24")]
-        .try_into()
-        .expect("24");
-    let key_order_mismatches = count_key_order_mismatches(&[extent_key]);
-
-    // t3 inode 树叶容器（字节表四）：出生序号先于 t4 的根发号（树内先叶后根）。
-    // 容器身份（容器号、容器出生代）在树建起来那次定下，之后每一版重写都不变（D8（核心索引结构） 已定项 6：一片叶活很多代、反复重写）。
-    let inode_leaf_identity = PackedIdentity {
-        birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
-        record_type: PACKED_TYPE_INODE,
-        container: FIRST_INODE_NUMBER,
-        container_birth: publish.tree_birth_txg,
-    };
-    let inode_leaf_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
-    let inode_record = InodeRecord {
-        inode: FIRST_INODE_NUMBER,
-        object_birth: publish.inode_object_birth,
-        size: u64::try_from(publish.content.len()).expect("文件长度"),
-        change_count: publish.change_count,
-        write_time_seconds: publish.write_time_seconds,
+    // 文件角色：有新版本就装四个单元，没有就照抄上一版的指针与字节。
+    let carried_file = match &plan.file {
+        Some(_) => None,
+        None => Some(previous.expect("没有文件版本的发布要接在上一版之后：文件角色从它照抄")),
     };
-    let inode_leaf_unit = build_packed_unit(
-        inode_leaf_identity,
-        u16::try_from(INODE_RECORD_BYTES).expect("140"),
-        &[inode_record.to_bytes()],
-        txg,
-        filesystem_identifier,
-        write_order,
+    let (
+        data_unit,
+        data_pointer,
+        extent_unit,
+        key_order_mismatches,
+        inode_leaf_unit,
+        inode_leaf_pointer,
         inode_leaf_sequence,
-    );
-    let inode_leaf_pointer = NodePointer {
-        head: PointerHead {
-            birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
-            birth_txg: txg,
-        },
-        locations: pool.location_entries(slot_of(TransactionUnit::InodeLeaf), &inode_leaf_unit),
-        instance,
-        birth_sequence: inode_leaf_sequence,
-    };
-
-    // t2 extent 树根兼叶（字节表四·二）。
-    let extent_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_EXTENT), txg, instance);
-    let extent_unit = build_index_node(
-        TreeIdentifier(TREE_IDENTIFIER_EXTENT),
-        0,
-        usize::try_from(EXTENT_KEY_BYTES).expect("24"),
-        &extent_key,
-        &extent_key,
-        txg,
-        filesystem_identifier,
-        instance,
+        inode_record,
+        inode_root_unit,
         extent_sequence,
-        u16::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
-        std::slice::from_ref(&extent_record),
-    );
-
-    // t4 inode 树根：层级 1，key 区间 [ino, ino]，一条 120 字节条目。
-    let inode_root_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
-    let inode_key = FIRST_INODE_NUMBER.to_le_bytes();
-    let inode_root_unit = build_index_node(
-        TreeIdentifier(TREE_IDENTIFIER_INODE),
-        1,
-        8,
-        &inode_key,
-        &inode_key,
-        txg,
-        filesystem_identifier,
-        instance,
         inode_root_sequence,
-        u16::try_from(INODE_INTERNAL_ENTRY).expect("120"),
-        &[build_inode_internal_entry(
-            FIRST_INODE_NUMBER,
-            inode_leaf_identity,
-            inode_leaf_pointer,
-        )],
-    );
+    ) = match (&plan.file, carried_file) {
+        (Some(file), _) => {
+            // t1 数据单元（字节表二）。
+            let data_identity = DataUnitIdentity {
+                tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+                object: FIRST_INODE_NUMBER,
+                object_birth: file.inode_object_birth,
+                anchor_offset: 0,
+            };
+            let data_unit = build_data_unit(
+                data_identity,
+                txg,
+                filesystem_identifier,
+                write_order,
+                file.content,
+            );
+            let data_pointer = DataPointer {
+                head: PointerHead {
+                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+                    birth_txg: txg,
+                },
+                locations: pool.location_entries(slot_of(TransactionUnit::Data), &data_unit),
+                write_order,
+            };
+            let extent_record = build_extent_record(FIRST_INODE_NUMBER, 0, data_pointer);
+            let extent_key: [u8; 24] = extent_record
+                [..usize::try_from(EXTENT_KEY_BYTES).expect("24")]
+                .try_into()
+                .expect("24");
+            let key_order_mismatches = count_key_order_mismatches(&[extent_key]);
+
+            // t3 inode 树叶容器（字节表四）：出生序号先于 t4 的根发号（树内先叶后根）。
+            // 容器身份（容器号、容器出生代）在树建起来那次定下，之后每一版重写都不变（D8（核心索引结构） 已定项 6：一片叶活很多代、反复重写）。
+            let inode_leaf_identity = PackedIdentity {
+                birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
+                record_type: PACKED_TYPE_INODE,
+                container: FIRST_INODE_NUMBER,
+                container_birth: plan.tree_birth_txg,
+            };
+            let inode_leaf_sequence =
+                sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
+            let inode_record = InodeRecord {
+                inode: FIRST_INODE_NUMBER,
+                object_birth: file.inode_object_birth,
+                size: u64::try_from(file.content.len()).expect("文件长度"),
+                change_count: file.change_count,
+                write_time_seconds: file.write_time_seconds,
+            };
+            let inode_leaf_unit = build_packed_unit(
+                inode_leaf_identity,
+                u16::try_from(INODE_RECORD_BYTES).expect("140"),
+                &[inode_record.to_bytes()],
+                txg,
+                filesystem_identifier,
+                write_order,
+                inode_leaf_sequence,
+            );
+            let inode_leaf_pointer = NodePointer {
+                head: PointerHead {
+                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
+                    birth_txg: txg,
+                },
+                locations: pool
+                    .location_entries(slot_of(TransactionUnit::InodeLeaf), &inode_leaf_unit),
+                instance,
+                birth_sequence: inode_leaf_sequence,
+            };
+
+            // t2 extent 树根兼叶（字节表四·二）。
+            let extent_sequence =
+                sequences.next(TreeIdentifier(TREE_IDENTIFIER_EXTENT), txg, instance);
+            let extent_unit = build_index_node(
+                TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+                0,
+                usize::try_from(EXTENT_KEY_BYTES).expect("24"),
+                &extent_key,
+                &extent_key,
+                txg,
+                filesystem_identifier,
+                instance,
+                extent_sequence,
+                u16::try_from(EXTENT_LEAF_RECORD_BYTES).expect("112"),
+                std::slice::from_ref(&extent_record),
+            );
+
+            // t4 inode 树根：层级 1，key 区间 [ino, ino]，一条 120 字节条目。
+            let inode_root_sequence =
+                sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
+            let inode_key = FIRST_INODE_NUMBER.to_le_bytes();
+            let inode_root_unit = build_index_node(
+                TreeIdentifier(TREE_IDENTIFIER_INODE),
+                1,
+                8,
+                &inode_key,
+                &inode_key,
+                txg,
+                filesystem_identifier,
+                instance,
+                inode_root_sequence,
+                u16::try_from(INODE_INTERNAL_ENTRY).expect("120"),
+                &[build_inode_internal_entry(
+                    FIRST_INODE_NUMBER,
+                    inode_leaf_identity,
+                    inode_leaf_pointer,
+                )],
+            );
+            (
+                data_unit,
+                data_pointer,
+                extent_unit,
+                key_order_mismatches,
+                inode_leaf_unit,
+                inode_leaf_pointer,
+                inode_leaf_sequence,
+                inode_record,
+                inode_root_unit,
+                extent_sequence,
+                inode_root_sequence,
+            )
+        }
+        (None, Some(carried)) => {
+            // 照抄：数据指针、inode 记录、四个单元的字节都取上一版；inode 叶的指针从上一版 inode 根的那条条目解出来。
+            let inode_root_bytes = &carried.unit(TransactionUnit::InodeRoot).bytes;
+            let inode_root_node = parse_index_node(inode_root_bytes)
+                .expect("上一版的 inode 根是这次或上次发布装出来的，解得开");
+            let (_, _inode_leaf_identity, inode_leaf_pointer) = parse_inode_internal_entry(
+                inode_root_node
+                    .entries
+                    .first()
+                    .expect("第一版 inode 树根恒有一条条目"),
+            );
+            let extent_unit = carried.unit(TransactionUnit::ExtentRoot).bytes.clone();
+            let extent_pointer_previous = carried.tree_root_pointer(TREE_IDENTIFIER_EXTENT);
+            let inode_root_pointer_previous = carried.tree_root_pointer(TREE_IDENTIFIER_INODE);
+            (
+                carried.unit(TransactionUnit::Data).bytes.clone(),
+                carried.data_pointer,
+                extent_unit,
+                0,
+                carried.unit(TransactionUnit::InodeLeaf).bytes.clone(),
+                inode_leaf_pointer,
+                inode_leaf_pointer.birth_sequence,
+                carried.inode_record,
+                carried.unit(TransactionUnit::InodeRoot).bytes.clone(),
+                extent_pointer_previous.birth_sequence,
+                inode_root_pointer_previous.birth_sequence,
+            )
+        }
+        (None, None) => {
+            unreachable!("上面按 plan.file 分过：没有文件版本时 carried_file 一定是 Some")
+        }
+    };
+    // 实例表单元（码 3 打包记录类型 4）：重写时容器身份照 mkfs（容器 0、出生代 0），归树 0、在树表之前发号（mkfs 也是先实例表后树表）。
+    let instance_table_rewrite = match &plan.instance_table {
+        InstanceTablePlan::Rewrite(records) => {
+            let sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_NONE), txg, instance);
+            let unit = build_packed_unit(
+                PackedIdentity {
+                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
+                    record_type: PACKED_TYPE_INSTANCE_TABLE,
+                    container: 0,
+                    container_birth: CheckpointTxg(0),
+                },
+                u16::try_from(INSTANCE_ROW_BYTES).expect("88"),
+                records,
+                txg,
+                filesystem_identifier,
+                write_order,
+                sequence,
+            );
+            let pointer = NodePointer {
+                head: PointerHead {
+                    birth_tree: TreeIdentifier(TREE_IDENTIFIER_NONE),
+                    birth_txg: txg,
+                },
+                locations: pool.location_entries(slot_of(TransactionUnit::InstanceTable), &unit),
+                instance,
+                birth_sequence: sequence,
+            };
+            Some((unit, pointer, sequence))
+        }
+        InstanceTablePlan::Carry(_) => None,
+    };
+    let instance_table_pointer = match (&plan.instance_table, &instance_table_rewrite) {
+        (InstanceTablePlan::Carry(pointer), _) => *pointer,
+        (InstanceTablePlan::Rewrite(_), Some((_, pointer, _))) => *pointer,
+        (InstanceTablePlan::Rewrite(_), None) => unreachable!("重写时上面一定装了单元"),
+    };
 
-    // t5 分配记录树（字节表五）：mkfs 的两个单元分配代 0、其余是这次发布的 txg，每盘各一条，按 (设备, 槽号) 升序。
+    // t5 分配记录树（字节表五）：mkfs 的两个单元分配代 0、其余是各自分配那次发布的 txg，每盘各一条，按 (设备, 槽号) 升序。
     let mut allocation_records: Vec<AllocationRecord> = allocator.records().to_vec();
     allocation_records.sort_by_key(AllocationRecord::sort_key);
     let allocation_sequence = sequences.next(
@@ -1105,18 +1298,27 @@ fn publish_admitted_file_version<Device: BlockDevice>(
             instance,
             birth_sequence: sequence,
         };
-    let extent_pointer = node_pointer(
-        TREE_IDENTIFIER_EXTENT,
-        TransactionUnit::ExtentRoot,
-        &extent_unit,
-        extent_sequence,
-    );
-    let inode_root_pointer = node_pointer(
-        TREE_IDENTIFIER_INODE,
-        TransactionUnit::InodeRoot,
-        &inode_root_unit,
-        inode_root_sequence,
-    );
+    // 文件角色的指针：重写的按这次的落点算，照抄的取上一版。
+    let (extent_pointer, inode_root_pointer) = match carried_file {
+        None => (
+            node_pointer(
+                TREE_IDENTIFIER_EXTENT,
+                TransactionUnit::ExtentRoot,
+                &extent_unit,
+                extent_sequence,
+            ),
+            node_pointer(
+                TREE_IDENTIFIER_INODE,
+                TransactionUnit::InodeRoot,
+                &inode_root_unit,
+                inode_root_sequence,
+            ),
+        ),
+        Some(carried) => (
+            carried.tree_root_pointer(TREE_IDENTIFIER_EXTENT),
+            carried.tree_root_pointer(TREE_IDENTIFIER_INODE),
+        ),
+    };
     let allocation_pointer = node_pointer(
         TREE_IDENTIFIER_ALLOCATION_RECORDS,
         TransactionUnit::AllocationTree,
@@ -1131,6 +1333,7 @@ fn publish_admitted_file_version<Device: BlockDevice>(
     );
 
     // t7 中央映射树（字节表三·二）：码 1 一条 + 码 2 / 码 3 五条；映射树自己、树表、实例表豁免（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12）。
+    // 照抄的文件角色 key 照旧、位置照旧；重写的按这次的指针算。
     let mapped_units_with_locations: Vec<(TransactionUnit, Vec<u8>, [LocationEntry; 2])> = vec![
         (
             TransactionUnit::Data,
@@ -1200,14 +1403,14 @@ fn publish_admitted_file_version<Device: BlockDevice>(
         mapping_sequence,
     );
 
-    // t8 树表单元第 1 版：七条按树 ID 升序（D8（核心索引结构） 已定项 8）；映射树的根住根记录、不进树表（D19（块指针的结构与宽度预算） 已定项 11）；
+    // t8 树表单元：七条按树 ID 升序（D8（核心索引结构） 已定项 8）；映射树的根住根记录、不进树表（D19（块指针的结构与宽度预算） 已定项 11）；
     // 头 ID（D5（快照 / 空间记账机制） 已定项 9）：inode 树写自己、extent 树写它服务的头，其余 0。
     let table_entry =
         |kind: u16, tree: u64, root: NodePointer, head_identifier: u64| TreeTableEntry {
             kind,
             tree: TreeIdentifier(tree),
             root,
-            birth_txg: publish.tree_birth_txg,
+            birth_txg: plan.tree_birth_txg,
             head_identifier,
         };
     let tree_table_entries = vec![
@@ -1278,110 +1481,108 @@ fn publish_admitted_file_version<Device: BlockDevice>(
         tree_table_sequence,
     );
 
-    let units = vec![
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::Data),
-            identity: TransactionUnit::Data,
-            bytes: data_unit,
-        },
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::ExtentRoot),
-            identity: TransactionUnit::ExtentRoot,
-            bytes: extent_unit,
-        },
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::InodeLeaf),
-            identity: TransactionUnit::InodeLeaf,
-            bytes: inode_leaf_unit,
-        },
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::InodeRoot),
-            identity: TransactionUnit::InodeRoot,
-            bytes: inode_root_unit,
-        },
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::AllocationTree),
-            identity: TransactionUnit::AllocationTree,
-            bytes: allocation_unit,
-        },
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::AccountingTree),
-            identity: TransactionUnit::AccountingTree,
-            bytes: accounting_unit,
-        },
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::MappingTree),
-            identity: TransactionUnit::MappingTree,
-            bytes: mapping_unit,
-        },
-        PublishedUnit {
-            slot: slot_of(TransactionUnit::TreeTable),
-            identity: TransactionUnit::TreeTable,
-            bytes: tree_table_unit,
-        },
-    ];
+    // 这一版全部角色的单元：重写的是这次装的，照抄的从上一版拷（八个文件 / 固定点角色按 bump 次序，实例表单元在末尾）。
+    let rewritten_unit = |identity: TransactionUnit, bytes: Vec<u8>| PublishedUnit {
+        slot: slot_of(identity),
+        identity,
+        bytes,
+    };
+    let mut units: Vec<PublishedUnit> = Vec::new();
+    for identity in TransactionUnit::IN_BUMP_ORDER {
+        let unit = match identity {
+            TransactionUnit::Data => match carried_file {
+                None => rewritten_unit(identity, data_unit.clone()),
+                Some(carried) => carried_unit(carried, identity),
+            },
+            TransactionUnit::ExtentRoot => match carried_file {
+                None => rewritten_unit(identity, extent_unit.clone()),
+                Some(carried) => carried_unit(carried, identity),
+            },
+            TransactionUnit::InodeLeaf => match carried_file {
+                None => rewritten_unit(identity, inode_leaf_unit.clone()),
+                Some(carried) => carried_unit(carried, identity),
+            },
+            TransactionUnit::InodeRoot => match carried_file {
+                None => rewritten_unit(identity, inode_root_unit.clone()),
+                Some(carried) => carried_unit(carried, identity),
+            },
+            TransactionUnit::AllocationTree => rewritten_unit(identity, allocation_unit.clone()),
+            TransactionUnit::AccountingTree => rewritten_unit(identity, accounting_unit.clone()),
+            TransactionUnit::MappingTree => rewritten_unit(identity, mapping_unit.clone()),
+            TransactionUnit::TreeTable => rewritten_unit(identity, tree_table_unit.clone()),
+            TransactionUnit::InstanceTable => {
+                unreachable!("IN_BUMP_ORDER 只有八个文件 / 固定点角色")
+            }
+        };
+        units.push(unit);
+    }
+    match (&instance_table_rewrite, previous) {
+        (Some((unit, _, _)), _) => {
+            units.push(rewritten_unit(TransactionUnit::InstanceTable, unit.clone()))
+        }
+        (None, Some(previous_version)) => {
+            if let Some(carried) = previous_version
+                .units
+                .iter()
+                .find(|unit| unit.identity == TransactionUnit::InstanceTable)
+            {
+                units.push(carried.clone());
+            }
+        }
+        (None, None) => {}
+    }
 
-    // 点名项（D23（journal 的角色与格式） 已定项 17）：t1..t8 各一项，key 尾段与映射 key 共用。
-    let named_tails: [(TransactionUnit, [u8; 10]); 8] = [
-        (
-            TransactionUnit::Data,
-            data_key_tail(data_pointer.write_order),
-        ),
-        (
-            TransactionUnit::ExtentRoot,
-            node_key_tail(instance, extent_sequence),
-        ),
-        (
-            TransactionUnit::InodeLeaf,
-            node_key_tail(instance, inode_leaf_sequence),
-        ),
-        (
-            TransactionUnit::InodeRoot,
-            node_key_tail(instance, inode_root_sequence),
-        ),
-        (
-            TransactionUnit::AllocationTree,
-            node_key_tail(instance, allocation_sequence),
-        ),
-        (
-            TransactionUnit::AccountingTree,
-            node_key_tail(instance, accounting_sequence),
-        ),
-        (
-            TransactionUnit::MappingTree,
-            node_key_tail(instance, mapping_sequence),
-        ),
-        (
-            TransactionUnit::TreeTable,
-            node_key_tail(instance, tree_table_sequence),
-        ),
-    ];
-    let named: Vec<NamedUnit> = units
+    // 点名项（D23（journal 的角色与格式） 已定项 17）：这次重写的每个角色一项，key 尾段与映射 key 共用；照抄的不点名。
+    let key_tail_of = |identity: TransactionUnit| -> [u8; 10] {
+        match identity {
+            TransactionUnit::Data => data_key_tail(data_pointer.write_order),
+            TransactionUnit::ExtentRoot => node_key_tail(instance, extent_sequence),
+            TransactionUnit::InodeLeaf => node_key_tail(instance, inode_leaf_sequence),
+            TransactionUnit::InodeRoot => node_key_tail(instance, inode_root_sequence),
+            TransactionUnit::AllocationTree => node_key_tail(instance, allocation_sequence),
+            TransactionUnit::AccountingTree => node_key_tail(instance, accounting_sequence),
+            TransactionUnit::MappingTree => node_key_tail(instance, mapping_sequence),
+            TransactionUnit::TreeTable => node_key_tail(instance, tree_table_sequence),
+            TransactionUnit::InstanceTable => node_key_tail(
+                instance,
+                instance_table_rewrite
+                    .as_ref()
+                    .map(|(_, _, sequence)| *sequence)
+                    .expect("点名实例表单元的发布一定重写了它"),
+            ),
+        }
+    };
+    let written_units: Vec<&PublishedUnit> = rewritten
         .iter()
-        .zip(named_tails)
-        .map(|(unit, (identity, key_tail))| {
-            assert_eq!(unit.identity, identity, "点名项与单元一一对应");
-            NamedUnit {
-                locations: pool.location_entries(unit.slot, &unit.bytes),
-                unit_class: identity.unit_class(),
-                birth_tree: identity.tree(),
-                birth_txg: txg,
-                key_tail,
-            }
+        .map(|identity| {
+            units
+                .iter()
+                .find(|unit| unit.identity == *identity)
+                .expect("重写的角色都装了单元")
+        })
+        .collect();
+    let named: Vec<NamedUnit> = written_units
+        .iter()
+        .map(|unit| NamedUnit {
+            locations: pool.location_entries(unit.slot, &unit.bytes),
+            unit_class: unit.identity.unit_class(),
+            birth_tree: unit.identity.tree(),
+            birth_txg: txg,
+            key_tail: key_tail_of(unit.identity),
         })
         .collect();
     let record = JournalRecord {
         instance,
-        counter: publish.counter,
+        counter: plan.counter,
         checkpoint_txg: txg,
-        transaction: publish.transaction,
+        transaction: plan.transaction,
         is_commit: true,
-        back_chain: back_chain_of(previous_record_bytes),
+        back_chain: plan.back_chain,
         filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
         new_tree_table: tree_table_pointer,
         new_mapping_root: mapping_pointer,
-        new_tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
-        new_rollback_floor: CheckpointTxg(0),
+        new_tree_identifier_watermark: plan.tree_identifier_watermark,
+        new_rollback_floor: plan.rollback_floor,
         named,
     };
     let record_bytes = record.to_bytes();
@@ -1390,15 +1591,15 @@ fn publish_admitted_file_version<Device: BlockDevice>(
         instance,
         checkpoint_txg: txg,
         tree_table: tree_table_pointer,
-        tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
-        rollback_floor: CheckpointTxg(0),
-        instance_table: publish.instance_table,
+        tree_identifier_watermark: plan.tree_identifier_watermark,
+        rollback_floor: plan.rollback_floor,
+        instance_table: instance_table_pointer,
         mapping_root: mapping_pointer,
     };
     let root_slot = root.to_slot(pool.root_slot_bytes());
 
-    // 持久顺序（D16（发布语义） 已定项 7）：单元与节点 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 超级块槽轮换。
-    for unit in &units {
+    // 持久顺序（D16（发布语义） 已定项 7）：这次重写的单元 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 超级块槽轮换。
+    for unit in &written_units {
         pool.perform(CommitStep::WriteUnitToEveryDevice {
             slot: unit.slot,
             unit: &unit.bytes,
@@ -1406,7 +1607,7 @@ fn publish_admitted_file_version<Device: BlockDevice>(
     }
     pool.perform(CommitStep::Barrier)?;
     pool.perform(CommitStep::WriteJournalRecordToEveryDevice {
-        counter: publish.counter,
+        counter: plan.counter,
         record: &record_bytes,
     })?;
     pool.perform(CommitStep::Barrier)?;
@@ -1415,7 +1616,7 @@ fn publish_admitted_file_version<Device: BlockDevice>(
         root_slot: &root_slot,
     })?;
     pool.perform(CommitStep::RotateSuperblockSlots {
-        journal_tail: publish.counter,
+        journal_tail: plan.counter,
         journal_instance: instance,
     })?;
 
@@ -1424,6 +1625,7 @@ fn publish_admitted_file_version<Device: BlockDevice>(
         record,
         record_bytes,
         units,
+        rewritten: rewritten.to_vec(),
         data_pointer,
         mapping_keys: mapping_entries.into_iter().map(|(key, _)| key).collect(),
         allocation_records,
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
index 489d0e2..8f077bc 100644
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -27,6 +27,7 @@ const UNITS: [(u64, usize); 10] = [
     (50247, 16384),
     (50248, 16384),
 ];
+const INSTANCE_TABLE: u64 = 50176;
 const DATA_UNIT: u64 = 50180;
 const EXTENT_ROOT: u64 = 50240;
 const INODE_LEAF: u64 = 50242;
@@ -473,6 +474,27 @@ fn known_bad_images(clean: &MemoryPool) -> Vec<(&'static str, Mutation)> {
                 mutate_unit(image, INODE_LEAF, |bytes| set_u64(bytes, 136 + 8, 2), true)
             }),
         ),
+        // 实例表里多一行 (1, 3, 0)：行的实例代号等于挂载根的实例 1，不「低于」；链指针记录仍在末尾、行仍唯一，红的只有那一半。
+        (
+            "I-3.8",
+            Box::new(|image: &mut MemoryPool| {
+                mutate_unit(
+                    image,
+                    INSTANCE_TABLE,
+                    |bytes| {
+                        let chain_record: Vec<u8> = bytes[136..224].to_vec();
+                        let mut row = vec![0u8; 88];
+                        row[1..5].copy_from_slice(&1u32.to_le_bytes());
+                        row[5..13].copy_from_slice(&3u64.to_le_bytes());
+                        bytes[136..224].copy_from_slice(&row);
+                        bytes[224..312].copy_from_slice(&chain_record);
+                        set_u16(bytes, 69, 2);
+                        set_u16(bytes, 8, 176);
+                    },
+                    true,
+                );
+            }),
+        ),
         // 类型 2 容器的记录数与声明长度都改成 0。
         (
             "I-9.13",
diff --git a/crates/singlefs-harness/tests/common/mod.rs b/crates/singlefs-harness/tests/common/mod.rs
index c64073e..412b4cd 100644
--- a/crates/singlefs-harness/tests/common/mod.rs
+++ b/crates/singlefs-harness/tests/common/mod.rs
@@ -107,6 +107,28 @@ impl BuiltPool {
         pool.apply(&self.retained_operations()[..self.mkfs_operation_count]);
         pool
     }
+    /// 重开镜像并继续录进同一条流：进程重开之后的可写挂载与发布都要进层 0 的整条流（里程碑「第二个事务」步 0 / 步 3）。
+    pub fn reopen_recorded(&mut self) -> Vec<(DeviceIdentity, Recorded)> {
+        drop(self.devices.take());
+        self.paths
+            .iter()
+            .enumerate()
+            .map(|(index, path)| {
+                let file = FileBackedBlockDevice::open_or_create(
+                    path,
+                    IMAGE_BYTES,
+                    PhysicalBlockSizeInBytes(512),
+                )
+                .expect("重开镜像");
+                let identity = DeviceIdentity(u32::try_from(index).expect("设备号"));
+                (
+                    identity,
+                    RecordingBlockDevice::with_shared_stream(identity, file, self.stream.clone()),
+                )
+            })
+            .collect()
+    }
+
     /// 冷启动：丢掉进程内的设备句柄，按路径重新打开镜像。
     pub fn reopen_cold(&mut self) -> Vec<(DeviceIdentity, FileBackedBlockDevice)> {
         drop(self.devices.take());
diff --git a/crates/singlefs-harness/tests/first_transaction_step_five_publish.rs b/crates/singlefs-harness/tests/first_transaction_step_five_publish.rs
index bb89271..1ba8f67 100644
--- a/crates/singlefs-harness/tests/first_transaction_step_five_publish.rs
+++ b/crates/singlefs-harness/tests/first_transaction_step_five_publish.rs
@@ -1,5 +1,5 @@
 //! 里程碑「第一个事务」步 3 / 步 4 / 步 5 的验收：mkfs → 取号 → 暖机两次 → 第一个事务，落到两个文件镜像上，
-//! 录制流按路径切段与 E142（第一个事务的干跑） 第八次跑的产物 `research/results/e142-first-txn-dry-run-2026-09-16-tree-table-200.out`
+//! 录制流按路径切段与 E142（第一个事务的干跑） 第九次跑的产物 `research/results/e142-first-txn-dry-run-2026-09-16-instance-boundary.out`
 //! 逐字对（`name=segments` 五行、`name=root_record` 的反向链、`name=accounting` 的数），盘上的每个单元都由 checker 另一份解析判过。
 
 use std::collections::BTreeSet;
@@ -243,7 +243,9 @@ fn read_journal_record(pool: &BuiltPool, identity: DeviceIdentity, counter: u64)
 
 fn unit_length(identity: TransactionUnit) -> usize {
     match identity {
-        TransactionUnit::Data | TransactionUnit::InodeLeaf => 32768,
+        TransactionUnit::Data | TransactionUnit::InodeLeaf | TransactionUnit::InstanceTable => {
+            32768
+        }
         TransactionUnit::ExtentRoot
         | TransactionUnit::InodeRoot
         | TransactionUnit::AllocationTree
@@ -548,7 +550,9 @@ fn every_index_node_self_checks_with_tight_ascending_keys_and_merkle_checksums_h
                     .kind;
                 key_schema_for_tree_kind(kind).expect("有节点的树都有 key 形态")
             }
-            TransactionUnit::Data | TransactionUnit::InodeLeaf => unreachable!("上面按类跳过了"),
+            TransactionUnit::Data | TransactionUnit::InodeLeaf | TransactionUnit::InstanceTable => {
+                unreachable!("上面按类跳过了")
+            }
         };
         check_index_node_keys(&view, schema).expect("key 区间贴紧、条目严格递增");
         assert_eq!(
diff --git a/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs b/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
index 559232f..12aded8 100644
--- a/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
+++ b/crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs
@@ -15,7 +15,7 @@ use singlefs_harness::crash::{
 use singlefs_harness::segments::StepKind;
 use singlefs_harness::RecordedOperationKind;
 
-/// oracle 那一半：产物第 45 行逐字 `states=262165 … violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=9
+/// oracle 那一半：产物第 45 行逐字 `states=262165 … violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6
 /// verification_failed=0 first_violation=none`，第 48 行 `differing_states=3`。
 #[derive(Debug, PartialEq, Eq)]
 struct OracleCounts {
@@ -185,7 +185,7 @@ fn layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violation
             file_read_states: 7,
             failed_states: 0,
             journal_differing_states: 3,
-            verification_ran_states: 9,
+            verification_ran_states: 6,
             verification_failed_states: 0,
             first_violation: None,
         }
@@ -218,7 +218,7 @@ fn layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_fu
             file_read_states: 7,
             failed_states: 0,
             journal_differing_states: 3,
-            verification_ran_states: 9,
+            verification_ran_states: 6,
             verification_failed_states: 0,
             first_violation: None,
         },
diff --git a/crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs b/crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs
index 09d71ab..ca6e2e9 100644
--- a/crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs
+++ b/crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs
@@ -1,5 +1,5 @@
 //! 里程碑「第一个事务」步 6 的验收：进程外重开两个镜像走完整的挂载路径读出那个文件；坏字节探针与 E142（第一个事务的干跑）
-//! 第八次跑产物 `research/results/e142-first-txn-dry-run-2026-09-16-tree-table-200.out` 的 `name=recover_full` / `name=probe` 行逐字对。
+//! 第九次跑产物 `research/results/e142-first-txn-dry-run-2026-09-16-instance-boundary.out` 的 `name=recover_full` / `name=probe` 行逐字对。
 
 mod common;
 
@@ -39,7 +39,8 @@ fn cold_start_reopens_the_images_and_reads_the_file_back_choosing_instance_one_t
             above_water: 0,
             prefix_applied: 0,
             verification_passed: 0,
-            verification_failed: 0
+            verification_failed: 0,
+            maximum_applied_transaction: 0
         },
         "全环扫描到三条记录，没有一条高于所选根的水位"
     );
diff --git a/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs b/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs
index 1b8b3d6..aff21e8 100644
--- a/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs
+++ b/crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs
@@ -322,7 +322,8 @@ fn cold_start_reads_the_second_content_and_the_pool_checker_stays_green() {
             above_water: 0,
             prefix_applied: 0,
             verification_passed: 0,
-            verification_failed: 0
+            verification_failed: 0,
+            maximum_applied_transaction: 0
         },
         "暖机两条 + A + B 四条记录，没有一条高于所选根"
     );
@@ -450,7 +451,8 @@ fn release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not
     let mut pool = build_pool("release-via-mapping");
     let first = pool.output.clone();
     let via_mapping =
-        placements_to_release_via_mapping(&first, &pool.allocator).expect("A 的六条映射都在");
+        placements_to_release_via_mapping(&first, &pool.allocator, &TransactionUnit::IN_BUMP_ORDER)
+            .expect("A 的六条映射都在");
     assert_eq!(
         via_mapping,
         first.placements(),
diff --git a/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs b/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
index 1d9e898..5dbaf96 100644
--- a/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
+++ b/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs
@@ -7,8 +7,9 @@ mod common;
 
 use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
 use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
+use singlefs_core::mount::mount_writable;
 use singlefs_core::recovery::RecoveryOutcome;
-use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
+use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
 use singlefs_harness::crash::{
     closed_form_state_count, enumerate_layer0_selecting_versions, enumerate_layer0_versions,
     evaluate_state_for_versions, writes_and_segments, Layer0Tally, MemoryPool, PublishedVersion,
@@ -28,16 +29,35 @@ struct Prepared {
     base: MemoryPool,
     writes: Vec<RetainedWrite>,
     segments: Vec<Vec<usize>>,
-    /// B 的根槽 FUA 写在写表里的下标。
+    /// 被判的那次根槽 FUA 写在写表里的下标：脚本最后一次发布的根（到 B 为止就是 B 的，到 C 为止是 C 的）。
     judged_root_index: usize,
     /// A 的根槽 FUA 写在写表里的下标：它之后的写都是 B 的。
     first_root_index: usize,
     versions: Vec<PublishedVersion>,
 }
 
-fn overwrite(pool: &mut BuiltPool) {
+/// 固定脚本跑到哪一步（里程碑「第二个事务」步 0）：到 B 为止是靶向对照用的两次发布流；到 C 为止多了进程重开、可写挂载
+/// （取号、写行、暖机两次）与发布 C，是层 0 全量跑的那条流。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum Script {
+    ThroughSecondPublish,
+    ThroughThirdPublish,
+}
+
+const THIRD_FILE_BYTES: usize = 2500;
+
+fn third_content() -> Vec<u8> {
+    (0..THIRD_FILE_BYTES)
+        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
+        .collect()
+}
+
+fn overwrite(
+    pool: &mut BuiltPool,
+    content: &[u8],
+    instance: InstanceGeneration,
+) -> TransactionOutput {
     let parameters = parameters();
-    let content = second_content();
     let devices = pool.devices.as_mut().expect("镜像还开着");
     let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
     publish_overwrite(
@@ -45,52 +65,101 @@ fn overwrite(pool: &mut BuiltPool) {
         &mut pool.allocator,
         &pool.output,
         FirstFile {
-            content: &content,
+            content,
             write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
         },
-        InstanceGeneration(1),
+        instance,
     )
-    .expect("覆盖写");
+    .expect("覆盖写")
 }
 
-fn prepare(tag: &str) -> Prepared {
+fn prepare(tag: &str, script: Script) -> Prepared {
     let mut pool = build_pool(tag);
-    overwrite(&mut pool);
+    let second = overwrite(&mut pool, &second_content(), InstanceGeneration(1));
+    let mut versions = vec![
+        PublishedVersion {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(3),
+            content: file_content(),
+        },
+        PublishedVersion {
+            instance: InstanceGeneration(1),
+            checkpoint_txg: CheckpointTxg(4),
+            content: second_content(),
+        },
+    ];
+    if script == Script::ThroughThirdPublish {
+        pool.output = second;
+        // 进程重开：取号 2、写行、暖机两次（txg 5 / 6 落盘 0、txg 7 落盘 1），文件还是第二次的内容。
+        let mut devices = pool.reopen_recorded();
+        let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
+        pool.devices = Some(devices);
+        pool.allocator = mounted.allocator;
+        pool.output = mounted.current;
+        for txg in 5..=7 {
+            versions.push(PublishedVersion {
+                instance: InstanceGeneration(2),
+                checkpoint_txg: CheckpointTxg(txg),
+                content: second_content(),
+            });
+        }
+        overwrite(&mut pool, &third_content(), InstanceGeneration(2));
+        versions.push(PublishedVersion {
+            instance: InstanceGeneration(2),
+            checkpoint_txg: CheckpointTxg(8),
+            content: third_content(),
+        });
+    }
     let base = pool.memory_pool_after_mkfs();
     let operations = pool.retained_operations();
     let (writes, segments) =
         writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
-    assert_eq!(
-        segments.iter().map(Vec::len).collect::<Vec<_>>(),
-        vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 2],
-        "A 的两个超级块槽写与 B 的 16 个单元写合成一段：整条流按屏障切，不按发布切"
-    );
-    assert_eq!(writes.len(), 54, "取号 2 + 暖机 10 + A 21 + B 21 次写");
+    let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
+    match script {
+        Script::ThroughSecondPublish => {
+            assert_eq!(
+                sizes,
+                vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 2],
+                "A 的两个超级块槽写与 B 的 16 个单元写合成一段：整条流按屏障切，不按发布切"
+            );
+            assert_eq!(writes.len(), 54, "取号 2 + 暖机 10 + A 21 + B 21 次写");
+        }
+        Script::ThroughThirdPublish => {
+            // B 的两个超级块槽写与取号的两个合成一段（4）；写行发布 10 个单元写（实例表 + 四个固定点单元，各两盘）；
+            // 每次暖机 8 个单元写与上一次发布的两个超级块槽写合成一段（10）；C 的 16 个单元写与 txg 7 的超级块槽写合成 18。
+            assert_eq!(
+                sizes,
+                vec![
+                    2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 4, 10, 2, 1, 10, 2, 1, 10, 2, 1, 18, 2,
+                    1, 2
+                ],
+                "固定脚本到 C 为止的段序列"
+            );
+            assert_eq!(
+                writes.len(),
+                54 + 2 + 15 + 13 + 13 + 21,
+                "取号 2 + 写行 15 + 暖机 13 × 2 + C 21"
+            );
+        }
+    }
     let root_indexes: Vec<usize> = writes
         .iter()
         .enumerate()
         .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
         .map(|(index, _)| index)
         .collect();
-    assert_eq!(root_indexes.len(), 4, "暖机两代 + A + B 四条根槽写");
+    let expected_roots = match script {
+        Script::ThroughSecondPublish => 4,
+        Script::ThroughThirdPublish => 8,
+    };
+    assert_eq!(root_indexes.len(), expected_roots, "每次发布一条根槽写");
     Prepared {
         base,
         writes,
         segments,
-        judged_root_index: root_indexes[3],
+        judged_root_index: *root_indexes.last().expect("至少一条根槽写"),
         first_root_index: root_indexes[2],
-        versions: vec![
-            PublishedVersion {
-                instance: InstanceGeneration(1),
-                checkpoint_txg: CheckpointTxg(3),
-                content: file_content(),
-            },
-            PublishedVersion {
-                instance: InstanceGeneration(1),
-                checkpoint_txg: CheckpointTxg(4),
-                content: second_content(),
-            },
-        ],
+        versions,
     }
 }
 
@@ -127,11 +196,11 @@ fn assert_checker_and_record_checker_clean(tally: &Layer0Tally) {
     }
 }
 
-/// 平时跑的那一份：两个 18 写的段不展开（只以整段持久进入后面的状态），其余每段任意子集。
+/// 平时跑的那一份：三个 18 写的段与三个 10 写的段不展开（只以整段持久进入后面的状态），其余每段任意子集。
 #[test]
 fn every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims() {
-    let prepared = prepare("layer0-b-fast");
-    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 18;
+    let prepared = prepare("layer0-c-fast", Script::ThroughThirdPublish);
+    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 10;
     let tally = enumerate_layer0_selecting_versions(
         &prepared.base,
         &prepared.writes,
@@ -143,7 +212,7 @@ fn every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_r
     let expanded: Vec<Vec<usize>> = prepared
         .segments
         .iter()
-        .filter(|segment| segment.len() < 18)
+        .filter(|segment| segment.len() < 10)
         .cloned()
         .collect();
     assert_eq!(
@@ -151,7 +220,10 @@ fn every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_r
         closed_form_state_count(&expanded),
         "展开的段按闭式数"
     );
-    assert_eq!(tally.states, 26);
+    assert_eq!(
+        tally.states, 57,
+        "1 + 十四个 2 写段各 3 + 一个 4 写段 15 + 十一个 1 写段各 1"
+    );
     assert_eq!(
         tally.violations, 0,
         "第一处违例：{:?}",
@@ -166,14 +238,14 @@ fn every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_r
     assert_checker_and_record_checker_clean(&tally);
 }
 
-/// 全量：两次发布的流 13 段、闭式 524312 个状态（第一个事务的 262165 减掉末尾那个 2 写的段、加上 B 的四段与合成的 18 写段）。
+/// 全量：固定脚本到 C 为止 26 段、闭式 789555 个状态（三个 18 写段各 262143、三个 10 写段各 1023、一个 4 写段 15，其余 2 写段各 3、1 写段各 1）。
 /// 54 号门禁在 release 下跑它，认下面打印的 `LAYER0B` 行里 `exhaustive=true`。
 #[test]
-#[ignore = "全量 524312 个状态、每个两遍恢复 + checker，debug 下几分钟；门禁 54 号在 release 下跑"]
-fn full_enumeration_of_the_two_publish_stream_is_exhaustive_and_clean() {
-    let prepared = prepare("layer0-b-full");
+#[ignore = "全量 789555 个状态、每个两遍恢复 + checker，debug 下十几分钟；门禁 54 号在 release 下跑"]
+fn full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean() {
+    let prepared = prepare("layer0-c-full", Script::ThroughThirdPublish);
     let closed_form = closed_form_state_count(&prepared.segments);
-    assert_eq!(closed_form, 524_312, "闭式：1 + Σ(2^|段| − 1)，十三段");
+    assert_eq!(closed_form, 789_555, "闭式：1 + Σ(2^|段| − 1)，二十六段");
     let tally = enumerate_layer0_versions(
         &prepared.base,
         &prepared.writes,
@@ -216,7 +288,7 @@ fn full_enumeration_of_the_two_publish_stream_is_exhaustive_and_clean() {
 /// 靶向的阳性对照：把持久集合手工摆成四个形状，oracle、journal 承重、记录核对器各要在它该红的那一格红。
 #[test]
 fn targeted_controls_on_the_second_publish_go_red_where_they_should() {
-    let prepared = prepare("layer0-b-controls");
+    let prepared = prepare("layer0-b-controls", Script::ThroughSecondPublish);
     let all = vec![true; prepared.writes.len()];
     let is_second_publish = |index: usize| index > prepared.first_root_index;
     let kinds_of_second_publish = |kind: StepKind| -> Vec<usize> {
```

## 二、新文件 `crates/singlefs-core/src/mount.rs` 全文

```rust
//! 可写挂载（里程碑「第二个事务」步 3）：进程重开镜像之后先走一次恢复，从盘上重建可写态——所选根、施加前缀之后的根、
//! 上一版全部角色的单元与指针、分配器、下一条 jsn——再取实例代号、给上一个实例写行（D18（块里携带什么信息） 已定项 11）、
//! 推空发布直到本实例的根覆盖每块盘（D16（发布语义） 已定项 8 甲′），之后本实例的发布才接在后面。
//! 第一版没有干净关闭标记，重开一律走恢复。

use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use crate::allocator::{DeviceFreeMap, PoolAllocator};
use crate::block_device::BlockDevice;
use crate::bytes::{ByteReader, ByteWriter};
use crate::journal::back_chain_of;
use crate::make_filesystem::MakeFilesystemParameters;
use crate::recovery::{
    choose_root, choose_superblock, highest_root_txg, rebuild_version, replay_journal,
    scan_journal, JournalScanReport, RecoveryFailure,
};
use crate::root_record::RootRecord;
use crate::root_ring::target_for_publish;
use crate::transaction::{
    acquire_instance, publish_version, AcquisitionFailed, InstanceTablePlan, PoolWriter,
    PublishError, PublishPlan, TransactionOutput, TransactionUnit,
};
use crate::unit::parse_packed_unit;
use singlefs_format::{INSTANCE_ROW_BYTES, ROOT_RING_REGIONS};

/// 实例表 `kind` 0 行记录（D18（块里携带什么信息） 已定项 11）：
/// `kind 1 | 实例代号 4 | 所选根的 checkpoint_txg 8 | 属于该实例的最大已施加事务号 W 8 | flags 1（bit0 = 回退行）| 预留 66`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceRow {
    pub instance: InstanceGeneration,
    pub selected_root_txg: CheckpointTxg,
    pub applied_transaction_high_water: u64,
    pub is_rollback: bool,
}

const INSTANCE_ROW_KIND_ROW: u8 = 0;
const INSTANCE_ROW_KIND_CHAIN: u8 = 1;
const INSTANCE_ROW_FLAG_ROLLBACK: u8 = 0b0000_0001;

impl InstanceRow {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(usize::try_from(INSTANCE_ROW_BYTES).expect("88"));
        writer.put_u8(INSTANCE_ROW_KIND_ROW);
        writer.put_u32(self.instance.0);
        writer.put_u64(self.selected_root_txg.0);
        writer.put_u64(self.applied_transaction_high_water);
        writer.put_u8(if self.is_rollback {
            INSTANCE_ROW_FLAG_ROLLBACK
        } else {
            0
        });
        writer.skip(66);
        writer.assert_position(INSTANCE_ROW_BYTES, "实例表行记录");
        writer.into_bytes()
    }

    /// `kind` 0 才是行；链指针记录（`kind` 1）与别的 `kind` 返回 `None`。flags 只认 bit0，别的位非 0 拒收。
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != usize::try_from(INSTANCE_ROW_BYTES).expect("88")
            || bytes[0] != INSTANCE_ROW_KIND_ROW
        {
            return None;
        }
        let mut reader = ByteReader::at(bytes, 1);
        let instance = InstanceGeneration(reader.get_u32());
        let selected_root_txg = CheckpointTxg(reader.get_u64());
        let applied_transaction_high_water = reader.get_u64();
        let flags = reader.get_u8();
        if flags & !INSTANCE_ROW_FLAG_ROLLBACK != 0 {
            return None;
        }
        Some(Self {
            instance,
            selected_root_txg,
            applied_transaction_high_water,
            is_rollback: flags & INSTANCE_ROW_FLAG_ROLLBACK != 0,
        })
    }
}

/// 一片实例表单元里的记录：行在前、链指针记录最末（D18（块里携带什么信息） 已定项 11）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceTableRecords {
    pub rows: Vec<InstanceRow>,
    pub chain_record: Vec<u8>,
}

impl InstanceTableRecords {
    /// 从一片实例表单元解出来；行与链指针之外的 `kind`、链指针不在最末、没有链指针，都是 `None`。
    #[must_use]
    pub fn parse(instance_table_unit: &[u8]) -> Option<Self> {
        let unit = parse_packed_unit(instance_table_unit).ok()?;
        let (last, rows_bytes) = unit.records.split_last()?;
        if last.first().copied() != Some(INSTANCE_ROW_KIND_CHAIN) {
            return None;
        }
        let mut rows = Vec::with_capacity(rows_bytes.len());
        for bytes in rows_bytes {
            rows.push(InstanceRow::parse(bytes)?);
        }
        Some(Self {
            rows,
            chain_record: last.clone(),
        })
    }

    #[must_use]
    pub fn to_records(&self) -> Vec<Vec<u8>> {
        let mut records: Vec<Vec<u8>> = self.rows.iter().map(InstanceRow::to_bytes).collect();
        records.push(self.chain_record.clone());
        records
    }
}

/// 可写挂载没做成。
#[derive(Debug)]
pub enum MountError {
    Recovery(RecoveryFailure),
    /// 所选根下面还没有发布过文件版本（树表为空）：第一版的可写挂载只接在有文件的池后面，刚 mkfs 的池走第一次可写挂载那条路。
    NoPublishedVersion,
    /// 所选根指着的实例表单元解不出行与链指针。
    InstanceTableMalformed,
    Acquisition(AcquisitionFailed),
    Publish(PublishError),
}

impl From<RecoveryFailure> for MountError {
    fn from(failure: RecoveryFailure) -> Self {
        MountError::Recovery(failure)
    }
}

impl From<PublishError> for MountError {
    fn from(error: PublishError) -> Self {
        MountError::Publish(error)
    }
}

/// 一次可写挂载写出的东西。
#[derive(Debug)]
pub struct MountOutput {
    pub instance: InstanceGeneration,
    /// 恢复择到的根（施加前缀之前）。
    pub chosen_root: RootRecord,
    /// 施加前缀之后的根：写行与照抄都以它为准。
    pub effective_root: RootRecord,
    pub journal: JournalScanReport,
    /// 这次挂载写进实例表的行（上一个实例那一行，中间实例各一行）。
    pub rows_written: Vec<InstanceRow>,
    /// 写行那次发布（本实例的第一次发布）。
    pub row_publish: TransactionOutput,
    /// 之后的暖机空发布，直到本实例的根覆盖每块盘。
    pub warm_up_publishes: Vec<TransactionOutput>,
}

/// 可写挂载的结果：写出的东西、重建并推进过的分配器、接下来的发布要接在后面的那一版。
#[derive(Debug)]
pub struct Mounted {
    pub output: MountOutput,
    pub allocator: PoolAllocator,
    pub current: TransactionOutput,
}

/// 可写挂载：恢复 → 重建上一版与分配器 → 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘。
///
/// # Errors
/// 恢复失败、所选根下没有文件版本、实例表解不开、取号失败、发布失败。
pub fn mount_writable<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
) -> Result<Mounted, MountError> {
    let superblock = choose_superblock(&*devices)?;
    let chosen_root = choose_root(&*devices, &superblock).ok_or(RecoveryFailure::NoValidRoot)?;
    let records = scan_journal(&*devices, &superblock);
    let (journal, effective_root) = replay_journal(
        &*devices,
        &chosen_root,
        superblock.geometry.journal_ring_bytes,
        &records,
        true,
    );
    // 所选根自己那条记录读得出就拿它当上一版的记录；读不出（两份都撕了）就拿最大 jsn 那条顶着——本实例的第一条反向链恒 0、
    // 事务号从 1 起，上一版的记录只有 jsn 会被用到，而 jsn 下面另算。
    let own_record = records
        .values()
        .find(|record| {
            record.instance == effective_root.instance
                && record.checkpoint_txg == effective_root.checkpoint_txg
        })
        .or_else(|| records.values().max_by_key(|record| record.counter))
        .cloned()
        .ok_or(MountError::NoPublishedVersion)?;
    let own_record_bytes = own_record.to_bytes();
    let previous = rebuild_version(&*devices, &effective_root, own_record, own_record_bytes)
        .map_err(|failure| match failure {
            RecoveryFailure::InvariantViolated {
                invariant: "挂载",
            ..
            } => MountError::NoPublishedVersion,
            other @ (RecoveryFailure::NoValidSuperblock { .. }
            | RecoveryFailure::SuperblocksDisagree
            | RecoveryFailure::NoValidRoot
            | RecoveryFailure::UnitUnreadable { .. }
            | RecoveryFailure::UnitMalformed { .. }
            | RecoveryFailure::InvariantViolated { .. }
            | RecoveryFailure::MappingMiss { .. }
            | RecoveryFailure::MappingStillUnreadable { .. }) => MountError::Recovery(other),
        })?;
    let table = InstanceTableRecords::parse(&previous.unit(TransactionUnit::InstanceTable).bytes)
        .ok_or(MountError::InstanceTableMalformed)?;
    let next_counter = records
        .values()
        .map(|record| record.counter)
        .max()
        .unwrap_or(0)
        + 1;
    let highest_record_txg = records
        .values()
        .map(|record| record.checkpoint_txg)
        .max()
        .unwrap_or(CheckpointTxg(0));
    // 新实例的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1
    // （D23（journal 的角色与格式） 已定项 14 第 3 条）。
    let highest_ring_txg = highest_root_txg(
        &*devices,
        &superblock.region_devices,
        &superblock.geometry,
        &superblock.filesystem_identifier,
    )
    .unwrap_or(CheckpointTxg(0));
    let first_txg = CheckpointTxg(highest_ring_txg.max(highest_record_txg).0 + 1);
    let device_maps: Vec<DeviceFreeMap> = devices
        .iter()
        .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
        .collect();
    let mut allocator =
        PoolAllocator::rebuild_from_records(device_maps, previous.allocation_records.clone());

    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut pool).map_err(MountError::Acquisition)?;

    // 写行：给 [max(所选根的实例, 1), 新实例) 里每个实例各一行——上一个实例 (i, 施加之后的根的 txg, W)，中间实例 (i, 0, 0)；
    // 实例 0（mkfs）不写行（D18（块里携带什么信息） 已定项 11）。
    let mut rows_written = Vec::new();
    let mut table_after = table.clone();
    let first_row_instance = effective_root.instance.0.max(1);
    for row_instance in first_row_instance..instance.0 {
        let row = if row_instance == effective_root.instance.0 {
            InstanceRow {
                instance: InstanceGeneration(row_instance),
                selected_root_txg: effective_root.checkpoint_txg,
                applied_transaction_high_water: journal.maximum_applied_transaction,
                is_rollback: false,
            }
        } else {
            InstanceRow {
                instance: InstanceGeneration(row_instance),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
                is_rollback: false,
            }
        };
        rows_written.push(row);
        table_after.rows.push(row);
    }
    let row_publish = publish_version(
        &mut pool,
        &mut allocator,
        PublishPlan {
            txg: first_txg,
            counter: next_counter,
            transaction: 0,
            instance,
            back_chain: 0,
            file: None,
            instance_table: InstanceTablePlan::Rewrite(table_after.to_records()),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(&previous),
    )?;

    // 暖机（D16（发布语义） 已定项 8 甲′）：本实例的根覆盖两块盘之前连推空发布，次数按落点公式现算，至多根环的区域数那么多次。
    let all_devices: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    let device_of_txg = |txg: CheckpointTxg| {
        let target = target_for_publish(txg);
        parameters.region_devices[usize::try_from(target.region).expect("区域号")]
    };
    let mut covered: Vec<DeviceIdentity> = vec![device_of_txg(row_publish.root.checkpoint_txg)];
    let mut current = row_publish.clone();
    let mut warm_up_publishes = Vec::new();
    while all_devices
        .iter()
        .any(|identity| !covered.contains(identity))
        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS
    {
        let next = publish_version(
            &mut pool,
            &mut allocator,
            PublishPlan {
                txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
                counter: current.record.counter + 1,
                transaction: 0,
                instance,
                back_chain: back_chain_of(&current.record_bytes),
                file: None,
                instance_table: InstanceTablePlan::Carry(current.root.instance_table),
                tree_birth_txg: current.tree_birth_txg(),
                tree_identifier_watermark: current.root.tree_identifier_watermark,
                rollback_floor: current.root.rollback_floor,
            },
            Some(&current),
        )?;
        let device = device_of_txg(next.root.checkpoint_txg);
        if !covered.contains(&device) {
            covered.push(device);
        }
        warm_up_publishes.push(next.clone());
        current = next;
    }

    Ok(Mounted {
        output: MountOutput {
            instance,
            chosen_root,
            effective_root,
            journal,
            rows_written,
            row_publish,
            warm_up_publishes,
        },
        allocator,
        current,
    })
}
```

## 三、新文件 `crates/singlefs-harness/tests/second_transaction_step_three_second_instance.rs` 全文

```rust
//! 里程碑「第二个事务」步 3 的验收：发布 B 之后进程退出、重开两个镜像走可写挂载——恢复、从盘上重建上一版与分配器、
//! 取实例代号 2、给实例 1 写行 (1, 4, 0)、写行发布 txg 5、暖机两次（txg 6 落盘 0 白费、txg 7 落盘 1）——再发布 C；
//! 冷启动择实例 2 的根读回第三次的内容；实例表读回那一行；池级 checker 全绿（含 I-3.8）。

mod common;

use common::{build_pool, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::journal::back_chain_of;
use singlefs_core::journal::record_offset;
use singlefs_core::mount::{mount_writable, InstanceRow, InstanceTableRecords, Mounted};
use singlefs_core::records::{
    STATISTIC_ALLOCATED_BYTES, STATISTIC_DEFER_QUEUE_BYTES, STATISTIC_FREE_BYTES,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_overwrite, publish_version, FirstFile, InstanceTablePlan, PoolWriter, PublishPlan,
    TransactionOutput, TransactionUnit,
};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_format::SLOT_BYTES;

const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn second_content() -> Vec<u8> {
    content_of(SECOND_FILE_BYTES, 3)
}

fn third_content() -> Vec<u8> {
    content_of(THIRD_FILE_BYTES, 11)
}

/// 同一个进程、同一个实例里再发布一版（发布 B 用；步 3 的「预置一条实例 1 的记录」也用它造）。
fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写")
}

/// A → B 之后进程退出、重开并可写挂载。
fn build_publish_second_version_and_remount(tag: &str) -> (BuiltPool, TransactionOutput, Mounted) {
    let mut pool = build_pool(tag);
    let second = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    pool.output = second.clone();
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    (pool, second, mounted)
}

fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(CheckpointTxg(txg));
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

fn accounting_value(output: &TransactionOutput, statistic: u16, device: DeviceIdentity) -> u64 {
    output
        .accounting_entries
        .iter()
        .find(|entry| entry.statistic == statistic && entry.device == device)
        .map(|entry| entry.value)
        .expect("带设备维的行每盘一行")
}

/// 验收第一、二条：取号 2；行 (1, 4, 0)；写行发布 txg 5 重写实例表 + 四个固定点单元、事务号 0、反向链 0；暖机两次落到两块盘；
/// 发布 C 接在后面（txg 8、事务号 1、反向链接 txg 7 那条）；冷启动择 (2, 8) 读回第三次的内容；记账与分配器对得上；checker 全绿。
#[test]
fn remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version()
{
    let (mut pool, second, mut mounted) =
        build_publish_second_version_and_remount("step-three-remount");
    let output = &mounted.output;
    assert_eq!(
        output.instance,
        InstanceGeneration(2),
        "取号 = max(超级块 1, 根环 1) + 1"
    );
    assert_eq!(
        (
            output.chosen_root.instance,
            output.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(4)),
        "B 的根持久了：所选根就是它"
    );
    assert_eq!(output.effective_root, output.chosen_root, "没有记录要施加");
    assert_eq!(output.journal.valid_records, 4);
    assert_eq!(output.journal.prefix_applied, 0);
    assert_eq!(
        output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(4),
            applied_transaction_high_water: 0,
            is_rollback: false,
        }],
        "实例 1 那一行 (1, 4, 0)；实例 0 不写行"
    );

    let row = &output.row_publish;
    assert_eq!(
        (row.root.instance, row.root.checkpoint_txg),
        (InstanceGeneration(2), CheckpointTxg(5))
    );
    assert_eq!(row.record.counter, 5, "jsn 全池接着走");
    assert_eq!(row.record.transaction, 0, "空发布的事务号 0");
    assert_eq!(row.record.back_chain, 0, "本实例的第一条反向链恒 0");
    assert_eq!(
        row.rewritten,
        [
            TransactionUnit::InstanceTable,
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ],
        "写行发布重写实例表 + 四个固定点单元（记账树已存在 ⇒ 空发布也重写，D16 已定项 9）"
    );
    assert_eq!(row.record.named.len(), 5, "点名的只有重写的五个");
    assert_eq!(row.data_pointer, second.data_pointer, "文件角色照抄 B 的");
    assert_eq!(row.inode_record, second.inode_record);
    for identity in [
        TransactionUnit::Data,
        TransactionUnit::ExtentRoot,
        TransactionUnit::InodeLeaf,
        TransactionUnit::InodeRoot,
    ] {
        assert_eq!(
            row.unit(identity).slot,
            second.unit(identity).slot,
            "{identity:?} 照抄、不换落点"
        );
    }
    assert_eq!(
        row.tree_root_pointer(singlefs_format::TREE_IDENTIFIER_EXTENT),
        second.tree_root_pointer(singlefs_format::TREE_IDENTIFIER_EXTENT),
        "树表里 extent 根指针照抄 B 的"
    );
    let table = InstanceTableRecords::parse(&row.unit(TransactionUnit::InstanceTable).bytes)
        .expect("写出的实例表单元解得开");
    assert_eq!(table.rows, output.rows_written);
    assert_eq!(
        row.released.len(),
        5,
        "写行发布释放 mkfs 的实例表单元 + B 的四个固定点单元"
    );

    assert_eq!(
        output
            .warm_up_publishes
            .iter()
            .map(|publish| publish.root.checkpoint_txg.0)
            .collect::<Vec<_>>(),
        [6, 7],
        "实例 2 从 txg 5 起要两次：txg 6 落盘 0 白费、txg 7 落盘 1"
    );
    assert_eq!(region_device(5), DeviceIdentity(0));
    assert_eq!(region_device(6), DeviceIdentity(0));
    assert_eq!(region_device(7), DeviceIdentity(1));
    for publish in &output.warm_up_publishes {
        assert_eq!(
            publish.rewritten,
            [
                TransactionUnit::AllocationTree,
                TransactionUnit::AccountingTree,
                TransactionUnit::MappingTree,
                TransactionUnit::TreeTable,
            ]
        );
        assert_eq!(publish.record.transaction, 0);
        assert_eq!(
            publish.released.len(),
            4,
            "每次暖机释放上一次的四个固定点单元"
        );
    }
    assert_eq!(
        mounted.current.root.checkpoint_txg,
        CheckpointTxg(7),
        "接下来的发布接在 txg 7 后面"
    );

    // 发布 C：实例 2 的第一次文件发布。
    let third = third_content();
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let third_publish = publish_overwrite(
        &mut writer,
        &mut mounted.allocator,
        &mounted.current,
        FirstFile {
            content: &third,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
        },
        InstanceGeneration(2),
    )
    .expect("发布 C");
    assert_eq!(
        (
            third_publish.root.instance,
            third_publish.root.checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(8))
    );
    assert_eq!(third_publish.record.counter, 8);
    assert_eq!(
        third_publish.record.transaction, 1,
        "事务号按实例计数从 1 起"
    );
    assert_eq!(
        third_publish.record.back_chain,
        back_chain_of(&mounted.current.record_bytes),
        "反向链接 txg 7 那条"
    );
    assert_eq!(third_publish.rewritten, TransactionUnit::IN_BUMP_ORDER);
    assert_eq!(
        third_publish.released.len(),
        8,
        "释放 B 的四个文件单元 + txg 7 的四个固定点单元"
    );
    assert_eq!(
        third_publish.inode_record.object_birth,
        CheckpointTxg(3),
        "对象出生代照旧"
    );
    let device_map = &mounted.allocator.devices[0];
    // 占着：mkfs 3 + A 10 + B 10 + txg5（实例表 2 + 4）+ txg6 4 + txg7 4 + C 10 = 47；
    // defer：A 10 + B 10 + mkfs 实例表 2 + txg5 的 4 + txg6 的 4 + txg7 的 4 = 34。
    assert_eq!(device_map.allocated_slots(), 47);
    assert_eq!(device_map.deferred_slots(), 34);
    assert_eq!(device_map.free_slots(), 211_968 - 47);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert_eq!(
            accounting_value(&third_publish, STATISTIC_ALLOCATED_BYTES, device),
            47 * SLOT_BYTES
        );
        assert_eq!(
            accounting_value(&third_publish, STATISTIC_DEFER_QUEUE_BYTES, device),
            34 * SLOT_BYTES
        );
        assert_eq!(
            accounting_value(&third_publish, STATISTIC_FREE_BYTES, device),
            (211_968 - 47) * SLOT_BYTES
        );
    }
    assert_eq!(
        third_publish.unit(TransactionUnit::Data).slot,
        SlotNumber(50184),
        "C 的数据单元落在 B 之后的下一对偶数空槽（A、B 的都占着）"
    );

    // 冷启动：择 (2, 8)，读回第三次的内容；八条记录都在环里、没有一条高于水位。
    let reopened = pool.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(8)),
            content: third.clone()
        }
    );
    assert_eq!(report.journal.valid_records, 8);
    assert_eq!(report.journal.above_water, 0);
    assert_eq!(report.journal.prefix_applied, 0);
    assert_eq!(
        recover(&reopened, JournalPolicy::Ignore).outcome,
        report.outcome,
        "根槽已持久：看不看 journal 一样"
    );

    // 池级 checker：全部判 Holds，I-3.8 真被评估过。
    let image = pool.memory_pool();
    let verdicts = check_pool_image(&image);
    let violated: Vec<_> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .collect();
    assert!(violated.is_empty(), "checker 判红：{violated:?}");
    for invariant in ["I-3.1", "I-3.8", "I-5.2", "I-7.2", "I-7.7"] {
        assert!(
            verdicts
                .iter()
                .any(|(name, verdict)| *name == invariant && *verdict == InvariantVerdict::Holds),
            "{invariant} 要真被评估过且成立"
        );
    }
}

/// 验收第二条的反面（D16 已定项 8 买的东西）：把实例 2 在盘 0 上的两个根槽（txg 5、6）都改坏，冷启动仍择到盘 1 上 txg 7 那个根、
/// 内容还是第二次的；暖机只推一次的话盘 1 上没有实例 2 的根，恢复退回实例 1。
#[test]
fn damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device() {
    let (pool, _second, mounted) =
        build_publish_second_version_and_remount("step-three-one-device-damaged");
    let mut image = pool.memory_pool();
    for txg in [5u64, 6] {
        let target = target_for_publish(CheckpointTxg(txg));
        image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
    }
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.effective_root,
        Some((InstanceGeneration(2), CheckpointTxg(7))),
        "盘 0 上实例 2 的根都坏了，盘 1 上 txg 7 那个还在"
    );
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(7)),
            content: second_content()
        }
    );
    assert_eq!(mounted.output.warm_up_publishes.len(), 2);
}

/// 验收第三条：预置一条实例 1 的记录（jsn 5、txg 5、事务号 3，点名的单元也在盘上），把它的根槽改坏 ⇒ 重开那一刻所选根是 B 的根、
/// 同一个实例、没有回退行 ⇒ 恢复必须施加它，行写 (1, 5, 3)，写行发布的 txg 从 6 起。
#[test]
fn stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row(
) {
    let mut pool = build_pool("step-three-stray-record");
    let second = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    pool.output = second;
    let stray = overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(1));
    assert_eq!(
        (
            stray.root.checkpoint_txg,
            stray.record.counter,
            stray.record.transaction
        ),
        (CheckpointTxg(5), 5, 3)
    );
    let mut devices: Vec<(DeviceIdentity, Recorded)> = pool.reopen_recorded();
    // 把 txg 5 的根槽改坏：只留记录与单元，模拟「根槽没持久」。
    let target = target_for_publish(CheckpointTxg(5));
    let device = region_device(5);
    let offset = slot_offset(target, 4096);
    let (_, damaged_device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("区域归属的盘在池里");
    let mut garbage = vec![0u8; 4096];
    garbage[0] = 0xff;
    damaged_device
        .write_at(offset, &garbage, WriteDurability::Plain)
        .expect("改坏根槽");
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    assert_eq!(
        (
            mounted.output.chosen_root.instance,
            mounted.output.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(4))
    );
    assert_eq!(mounted.output.journal.prefix_applied, 1, "jsn 5 被施加");
    assert_eq!(mounted.output.journal.maximum_applied_transaction, 3);
    assert_eq!(
        mounted.output.effective_root.checkpoint_txg,
        CheckpointTxg(5),
        "施加之后的根是第 5 代"
    );
    assert_eq!(
        mounted.output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(5),
            applied_transaction_high_water: 3,
            is_rollback: false,
        }]
    );
    assert_eq!(
        mounted.output.row_publish.root.checkpoint_txg,
        CheckpointTxg(6),
        "新实例的第一次发布 = max(根环 4, 记录 5) + 1"
    );
    assert_eq!(mounted.output.row_publish.record.counter, 6);
    pool.devices = Some(devices);
    let reopened = pool.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), mounted.current.root.checkpoint_txg),
            content: third_content()
        },
        "施加了那条记录之后照抄的文件角色就是第三次的内容"
    );
}

/// I-3.8（实例表行唯一且低于挂载根）的坏镜像：拿真写者再发一版，实例表里多写一行 (2, 7, 0)——行的实例代号等于挂载根的实例，
/// 别的都合法；池级 checker 只有 I-3.8 判红。
#[test]
fn checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root() {
    let (mut pool, _second, mut mounted) =
        build_publish_second_version_and_remount("step-three-bad-row");
    let table =
        InstanceTableRecords::parse(&mounted.current.unit(TransactionUnit::InstanceTable).bytes)
            .expect("实例表解得开");
    let mut bad_table = table.clone();
    bad_table.rows.push(InstanceRow {
        instance: InstanceGeneration(2),
        selected_root_txg: CheckpointTxg(7),
        applied_transaction_high_water: 0,
        is_rollback: false,
    });
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let current = mounted.current.clone();
    publish_version(
        &mut writer,
        &mut mounted.allocator,
        PublishPlan {
            txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
            counter: current.record.counter + 1,
            transaction: 0,
            instance: InstanceGeneration(2),
            back_chain: back_chain_of(&current.record_bytes),
            file: None,
            instance_table: InstanceTablePlan::Rewrite(bad_table.to_records()),
            tree_birth_txg: current.tree_birth_txg(),
            tree_identifier_watermark: current.root.tree_identifier_watermark,
            rollback_floor: current.root.rollback_floor,
        },
        Some(&current),
    )
    .expect("写者不拦这一行：拦它的是 checker");
    let verdicts = check_pool_image(&pool.memory_pool());
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(violated, ["I-3.8"], "只有 I-3.8 红：{verdicts:?}");
}

/// 前缀链首接在所选根自己那条记录之后（D23 已定项 14 第 1 条「链从所选根覆盖的最后一条记录之后接」）：同一实例里再发两版（txg 5、6），
/// 把两个根槽改坏、再把 jsn 5 的记录两份都改坏——所选根 (1, 4) 自己那条 jsn 4 读得出，链首该是 jsn 5，jsn 5 读不出 ⇒ 断号即止，
/// jsn 6 一条都不施加，读回第二次的内容；从「水位之上最小的那条」接的写法会施加 jsn 6、读出第四版。
#[test]
fn one_missing_record_right_after_the_chosen_root_stops_the_prefix_even_when_later_records_are_intact(
) {
    let mut pool = build_pool("step-three-chain-start");
    let second = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    pool.output = second;
    let third = overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(1));
    pool.output = third;
    let fourth = overwrite_in_process(&mut pool, &content_of(3300, 17), InstanceGeneration(1));
    assert_eq!(
        (fourth.root.checkpoint_txg, fourth.record.counter),
        (CheckpointTxg(6), 6)
    );
    let mut image = pool.memory_pool();
    for txg in [5u64, 6] {
        let target = target_for_publish(CheckpointTxg(txg));
        image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
    }
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        image.flip_byte(device, record_offset(5, JOURNAL_RING_DEFAULT_BYTES), 300);
    }
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(4)),
            content: second_content()
        },
        "jsn 5 读不出、jsn 6 不许接上：{:?}",
        report.journal
    );
    assert_eq!(report.journal.valid_records, 5, "jsn 1–4、6");
    assert_eq!(report.journal.prefix_applied, 0);
}
```

## 四、变异表 `crates/mutations.tsv` 全文（5 行注释 + 25 行数据 = 30 行）

```tsv
# crates 的变异表：每行六段，制表符分隔——变异名 <TAB> 文件 <TAB> 原文 <TAB> 替换文 <TAB> cargo test 的参数 <TAB> 必须红的测试名。
# 原文 / 替换文里的 \n 表示换行；原文在文件里必须恰好命中一次（锚点腐化就红，门禁 59 号）。
# 门禁 59 号把仓拷到临时目录、逐条改坏、跑点名的测试、要求那条测试判红、还原；一条没红就整道红。
# 这是 show-me-test.md「存进仓的变异清单，交给门禁反复复跑」那一条在 crates 上的形态：
# 三方对抗每一轮打中之后的改法，各留一条「改回去它就红」的变异，改法被悄悄撤回时这里先响。
逐盘核退回只核总数	crates/singlefs-core/src/recovery.rs	        && placement_sets.all(|placements| placements == first_device_placements)	        && placement_sets.all(|_placements| true)	-p singlefs-core --lib -- allocation_records_per_device	placement_recorded_on_one_device_only_is_rejected_even_when_the_total_is_even
同盘槽号不核唯一	crates/singlefs-core/src/recovery.rs	        if !slots_per_device	        if false && !slots_per_device	-p singlefs-core --lib -- allocation_records_per_device	two_records_for_the_same_slot_on_the_same_device_are_rejected
空闲不随分配减（I-5.2 要红）	crates/singlefs-core/src/allocator.rs	        self.free_slots -= span;	        // 变异：空闲不减	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
释放时清位图（立即复用）	crates/singlefs-core/src/allocator.rs	        self.deferred_slots += span;\n    }	        self.deferred_slots += span;\n        for index in start..end {\n            self.allocated[index] = false;\n        }\n    }	-p singlefs-harness --test second_transaction_step_one_overwrite -- released_placements	released_placements_are_not_handed_out_again_before_reclaim_exists
分配记录树装不下不判	crates/singlefs-core/src/transaction.rs	    if records_after_this_publish > allocation_node_capacity {	    if false && records_after_this_publish > allocation_node_capacity {	-p singlefs-harness --test second_transaction_step_one_overwrite -- repeated_overwrites	repeated_overwrites_report_a_full_allocation_node_instead_of_panicking
释放退回按提示	crates/singlefs-core/src/transaction.rs	            placements_to_release_via_mapping(previous_version, allocator, &rewritten)?	            previous_version.placements()	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_goes_through	release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released
映射查到 key 就算（落点指错不核记录）	crates/singlefs-core/src/transaction.rs	        let record = allocator.record_for(locations[0].device, slot).ok_or(	        let record = allocator\n            .record_for(locations[0].device, slot)\n            .or(allocator.records().first())\n            .ok_or(	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_reports_a_mapping_entry	release_reports_a_mapping_entry_whose_slot_has_no_record_or_the_wrong_span_instead_of_panicking
失败的发布不退回分配器	crates/singlefs-core/src/transaction.rs	    if outcome.is_err() {	    if false && outcome.is_err() {	-p singlefs-harness --test second_transaction_step_one_overwrite -- publish_running_out_of_space_midway	publish_running_out_of_space_midway_leaves_the_allocator_as_it_was
内容超长不判	crates/singlefs-core/src/transaction.rs	        if file.content.len() > data_unit_capacity {	        if false && file.content.len() > data_unit_capacity {	-p singlefs-harness --test second_transaction_step_one_overwrite -- content_larger_than_a_data_unit	content_larger_than_a_data_unit_payload_is_refused_before_anything_is_touched
oracle 只比 txg 不比实例	crates/singlefs-harness/src/crash.rs	        if (effective_txg, effective_instance) < (newest_txg, newest_instance) {	        if effective_txg < newest_txg {	-p singlefs-harness --lib -- oracle_instance_tests	landing_on_the_lower_instance_of_the_same_txg_is_a_violation
oracle 只按 txg 找版本	crates/singlefs-harness/src/crash.rs	        version.checkpoint_txg == effective_txg && version.instance == effective_instance	        version.checkpoint_txg == effective_txg	-p singlefs-harness --lib -- oracle_instance_tests	the_same_txg_from_two_instances_are_two_versions
oracle 放过没版本的更新根	crates/singlefs-harness/src/crash.rs	                (candidate_version.checkpoint_txg, candidate_version.instance)\n                    < (effective_txg, effective_instance)	                false	-p singlefs-harness --lib -- oracle_instance_tests	newer_root_without_any_version_reporting_no_file_is_violation
Ignore 那一遍恢复不过 oracle	crates/singlefs-harness/src/crash.rs	        tally.ignored_violations += 1;	        tally.ignored_violations += 0;	-p singlefs-harness --test second_transaction_step_zero_layer0 -- targeted_controls	targeted_controls_on_the_second_publish_go_red_where_they_should
步 1 变异：不释放旧落点	crates/singlefs-core/src/transaction.rs	            placements_to_release_via_mapping(previous_version, allocator, &rewritten)?	            Vec::<Placement>::new()	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
步 1 变异：defer 行写 0	crates/singlefs-core/src/transaction.rs	            device_map.deferred_slots() * SLOT_BYTES,	            0,	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
步 1 变异：事务号不加一	crates/singlefs-core/src/transaction.rs	            transaction: previous.record.transaction + 1,	            transaction: previous.record.transaction,	-p singlefs-harness --test second_transaction_step_one_overwrite -- overwrite_publishes	overwrite_publishes_the_second_version_through_the_same_commit_shape
步 1 变异：改动计数留 1	crates/singlefs-core/src/transaction.rs	            change_count: txg.0,	            change_count: 1,	-p singlefs-harness --test second_transaction_step_one_overwrite -- overwrite_publishes	overwrite_publishes_the_second_version_through_the_same_commit_shape
步 2 变异：已分配行不随分配更新（I-3.1 要红）	crates/singlefs-core/src/transaction.rs	            device_map.allocated_slots() * SLOT_BYTES,	            0,	-p singlefs-harness --test second_transaction_step_one_overwrite -- cold_start	cold_start_reads_the_second_content_and_the_pool_checker_stays_green
步 2 变异：忘了改写分配记录	crates/singlefs-core/src/allocator.rs	            record.is_released = true;	            record.is_released = false;	-p singlefs-harness --test second_transaction_step_one_overwrite -- release_rewrites	release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue
步 3：不写行（实例表照抄）	crates/singlefs-core/src/mount.rs	            instance_table: InstanceTablePlan::Rewrite(table_after.to_records()),	            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
步 3：暖机只推一次	crates/singlefs-core/src/mount.rs	        && u64::try_from(warm_up_publishes.len()).expect("次数") < ROOT_RING_REGIONS	        && u64::try_from(warm_up_publishes.len()).expect("次数") < 1	-p singlefs-harness --test second_transaction_step_three_second_instance -- damaging_every_instance_two_root	damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device
步 3：前缀跨实例边界（把别的实例的记录也接上）	crates/singlefs-core/src/recovery.rs	            record.instance == root.instance && (record.instance, record.checkpoint_txg) > water	            (record.instance, record.checkpoint_txg) > water	-p singlefs-harness --test first_transaction_step_seven_layer0 -- layer0_partial_enumeration	layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape
步 3：链首不接在所选根自己那条记录之后	crates/singlefs-core/src/recovery.rs	        root_own_record_counter.map(|counter| (root.instance, counter + 1));	        root_own_record_counter.map(|counter| (root.instance, counter + 1)).and(None);	-p singlefs-harness --test second_transaction_step_three_second_instance -- one_missing_record_right_after	one_missing_record_right_after_the_chosen_root_stops_the_prefix_even_when_later_records_are_intact
步 3：I-3.8 判定恒真	crates/singlefs-checker/src/walk.rs	            unique && below_mount_root && chain_record_last,	            unique || below_mount_root || chain_record_last || true,	-p singlefs-harness --test second_transaction_step_three_second_instance -- checker_rejects_an_instance_table_row	checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root
步 3：重建分配器时不把已释放的放进 defer 队列	crates/singlefs-core/src/allocator.rs	            if record.is_released {\n                device_map.mark_released(record.slot, span);\n            }	            if record.is_released && false {\n                device_map.mark_released(record.slot, span);\n            }	-p singlefs-harness --test second_transaction_step_three_second_instance -- remount_takes_instance_two	remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version
```
