# 附录二：发布 B 那批代码改动（`git diff HEAD -- crates/` 原样，加两个新测试文件全文；2026-09-16）

## 一、diff（crates/ 下相对 HEAD d2aeb7d 之后的工作区）

```diff
diff --git a/crates/singlefs-core/src/allocator.rs b/crates/singlefs-core/src/allocator.rs
index 1e1e174..caa8e08 100644
--- a/crates/singlefs-core/src/allocator.rs
+++ b/crates/singlefs-core/src/allocator.rs
@@ -12,22 +12,39 @@ use singlefs_format::{CLUSTER_SEGMENT_SLOTS, SLOT_BYTES, UNIT_AREA_START_SLOT};
 use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
 use crate::bytes::ByteWriter;
 
-/// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：key (设备 4, 槽号 6) + value (跨度 2 含已释放标志, 分配代 8)。
+/// 跨度段的最高位借作已释放标志（D3（空间分配） 已定项 7 / 已定项 11）：0 = 仍分配、1 = 已释放，不占表达跨度值的位。
+pub const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;
+
+/// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：key (设备 4, 槽号 6) + value (跨度 2 含已释放标志, 分配代或释放代 8)。
+/// 释放时条目不删、不点删：改写成已释放 + 释放代，留到该落点被重新分配时覆盖。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
 pub struct AllocationRecord {
     pub device: DeviceIdentity,
     pub slot: SlotNumber,
+    /// 纯跨度，不含标志位。
     pub span_slots: u16,
+    /// 仍分配时是分配代；已释放时是释放代。
     pub generation: CheckpointTxg,
+    pub is_released: bool,
 }
 
 impl AllocationRecord {
     #[must_use]
     pub fn to_bytes(&self) -> Vec<u8> {
+        assert_eq!(
+            self.span_slots & ALLOCATION_RECORD_RELEASED_FLAG,
+            0,
+            "跨度值不许占到标志位"
+        );
+        let span_field = if self.is_released {
+            self.span_slots | ALLOCATION_RECORD_RELEASED_FLAG
+        } else {
+            self.span_slots
+        };
         let mut writer = ByteWriter::new(20);
         writer.put_u32(self.device.0);
         writer.put_six_byte_unsigned(self.slot.0);
-        writer.put_u16(self.span_slots);
+        writer.put_u16(span_field);
         writer.put_u64(self.generation.0);
         writer.assert_position(20, "分配记录");
         writer.into_bytes()
@@ -47,13 +64,14 @@ impl AllocationRecord {
         let mut reader = crate::bytes::ByteReader::at(bytes, 0);
         let device = DeviceIdentity(reader.get_u32());
         let slot = SlotNumber(reader.get_six_byte_unsigned());
-        let span_slots = reader.get_u16();
+        let span_field = reader.get_u16();
         let generation = CheckpointTxg(reader.get_u64());
         Self {
             device,
             slot,
-            span_slots,
+            span_slots: span_field & !ALLOCATION_RECORD_RELEASED_FLAG,
             generation,
+            is_released: span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0,
         }
     }
 }
@@ -85,7 +103,11 @@ pub struct DeviceFreeMap {
     used_per_segment: Vec<u64>,
     /// 空闲槽的连续段数（增量维护）。
     free_runs: u64,
+    /// 占着的槽数：仍分配的加上已释放、还在 defer 窗口里的——它们仍被根环里的有效根引用、仍占着空间
+    /// （I-3.1（已分配统计对得上） 的读法 2026-09-14 用户定甲：按根环里全部有效根的引用取并集）。
     allocated_slots: u64,
+    /// 其中已释放、还在 defer 窗口里的槽数（D5（快照 / 空间记账机制） 已定项 4 第 5 项）。
+    deferred_slots: u64,
 }
 
 impl DeviceFreeMap {
@@ -101,6 +123,7 @@ impl DeviceFreeMap {
             used_per_segment: vec![0; usize::try_from(segments).expect("段数")],
             free_runs: u64::from(unit_area_slots > 0),
             allocated_slots: 0,
+            deferred_slots: 0,
         }
     }
 
@@ -123,11 +146,27 @@ impl DeviceFreeMap {
     pub fn allocated_slots(&self) -> u64 {
         self.allocated_slots
     }
+    /// 空闲 = 容量 − 占着的（I-5.2（空闲统计对得上）），已释放而还在 defer 窗口里的不算空闲。
     #[must_use]
     pub fn free_slots(&self) -> u64 {
         self.unit_area_slots - self.allocated_slots
     }
     #[must_use]
+    pub fn deferred_slots(&self) -> u64 {
+        self.deferred_slots
+    }
+
+    /// 把一个仍分配的落点放进 defer 队列：槽仍占着（分配器不许再发它），只是记账上从「仍分配」挪到「待释放」。
+    pub fn mark_released(&mut self, slot: SlotNumber, span: u64) {
+        let start = Self::index(slot);
+        let end = start + usize::try_from(span).expect("跨度");
+        assert!(
+            self.allocated[start..end].iter().all(|taken| *taken),
+            "释放的跨度里有没分配的槽"
+        );
+        self.deferred_slots += span;
+    }
+    #[must_use]
     pub fn free_runs(&self) -> u64 {
         self.free_runs
     }
@@ -257,6 +296,7 @@ impl PoolAllocator {
                 slot: placement.slot,
                 span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),
                 generation,
+                is_released: false,
             });
         }
         for device in &mut self.devices {
@@ -264,6 +304,27 @@ impl PoolAllocator {
         }
     }
 
+    /// 释放一个落点（D3（空间分配） 已定项 7：「释放」= 放进 defer 队列那一刻）：每盘那条分配记录改写成已释放 + 释放代，
+    /// 条目不删；槽仍占着，要等释放代 ≤ max(F_生效, 环里最旧有效根) 才可再分配（D16（发布语义） 已定项 1）。
+    pub fn release(&mut self, placement: Placement, release_generation: CheckpointTxg) {
+        for device in &mut self.devices {
+            let record = self
+                .records
+                .iter_mut()
+                .find(|record| record.device == device.device && record.slot == placement.slot)
+                .expect("释放的落点要有分配记录");
+            assert!(!record.is_released, "同一个落点释放了两次");
+            assert_eq!(
+                u64::from(record.span_slots),
+                placement.span,
+                "释放的跨度与分配记录不符"
+            );
+            record.is_released = true;
+            record.generation = release_generation;
+            device.mark_released(placement.slot, placement.span);
+        }
+    }
+
     /// 用户数据：按政策函数取，并把「政策函数再算一遍」与实际落点比对（不一致计数）。
     pub fn allocate_user_data(&mut self, generation: CheckpointTxg) -> Option<Placement> {
         let open = self.open_segment;
@@ -437,4 +498,38 @@ mod tests {
         assert_eq!(pool.devices[0].empty_segments(), 3310);
         assert_eq!(pool.devices[1].free_runs(), 4, "两盘同构");
     }
+
+    #[test]
+    fn releasing_a_placement_keeps_the_slots_occupied_and_moves_them_into_the_defer_queue() {
+        let mut pool = pool_after_mkfs();
+        let data = pool.allocate_user_data(CheckpointTxg(3)).expect("t1");
+        pool.release(data, CheckpointTxg(4));
+        assert_eq!(pool.devices[0].allocated_slots(), 5, "占着的槽数不变");
+        assert_eq!(pool.devices[0].deferred_slots(), 2, "两槽进了 defer 队列");
+        assert_eq!(pool.devices[0].free_slots(), 211_968 - 5, "空闲不变");
+        let released: Vec<&AllocationRecord> = pool
+            .records()
+            .iter()
+            .filter(|record| record.slot == data.slot)
+            .collect();
+        assert_eq!(released.len(), 2, "每盘一条");
+        for record in released {
+            assert!(record.is_released);
+            assert_eq!(record.generation, CheckpointTxg(4), "释放代");
+            assert_eq!(record.span_slots, 2, "跨度值不含标志位");
+            let parsed = AllocationRecord::parse(&record.to_bytes());
+            assert_eq!(parsed, *record, "标志位进跨度段最高位再读回来");
+            assert_eq!(
+                u16::from_le_bytes([record.to_bytes()[10], record.to_bytes()[11]]),
+                2 | ALLOCATION_RECORD_RELEASED_FLAG
+            );
+        }
+        assert_eq!(
+            pool.allocate_user_data(CheckpointTxg(4))
+                .expect("有空槽")
+                .slot,
+            SlotNumber(50182),
+            "已释放的 50180 不许再发，下一个偶数空槽对是 50182"
+        );
+    }
 }
diff --git a/crates/singlefs-core/src/recovery.rs b/crates/singlefs-core/src/recovery.rs
index 4cb4130..ebccd78 100644
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -621,10 +621,13 @@ pub fn walk_to_file(
         )?,
     };
     let device_count = reader.device_identities().len();
-    if roots.allocation.entries.len() != 10 * device_count {
+    // 每个落点每盘一条：第一个事务 10 × 盘数，每次覆盖写再加 8 × 盘数（换下的那些改写、不删）。
+    if roots.allocation.entries.len() < 10 * device_count
+        || !roots.allocation.entries.len().is_multiple_of(device_count)
+    {
         return Err(RecoveryFailure::InvariantViolated {
             invariant: "E142 走读同款",
-            detail: "分配记录数不是 10 × 盘数",
+            detail: "分配记录数不是盘数的整数倍或少于 10 × 盘数",
         });
     }
     if roots.accounting.entries.len() != 3 + 6 * device_count {
@@ -639,15 +642,13 @@ pub fn walk_to_file(
             detail: "映射条目数不是 6",
         });
     }
+    // 已释放的记录合法（D3（空间分配） 已定项 7：改写不删），它的代是释放代，同样不许晚于根。
     for record_bytes in &roots.allocation.entries {
         let record = AllocationRecord::parse(record_bytes);
-        if record.span_slots & 0x8000 != 0
-            || record.generation > root.checkpoint_txg
-            || record.span_slots == 0
-        {
+        if record.generation > root.checkpoint_txg || record.span_slots == 0 {
             return Err(RecoveryFailure::InvariantViolated {
                 invariant: "E142 走读同款",
-                detail: "分配记录带已释放标志、跨度为 0 或分配代晚于根",
+                detail: "分配记录跨度为 0，或分配代 / 释放代晚于根",
             });
         }
     }
diff --git a/crates/singlefs-core/src/transaction.rs b/crates/singlefs-core/src/transaction.rs
index 3799416..8946c3b 100644
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -3,6 +3,8 @@
 //! 三条路径都从同一个枚举走：取号 = 逐盘一次超级块槽写 + 一道屏障（D23（journal 的角色与格式） 已定项 16，C322（取号那一步的屏障怎么放没有条款） 2026-09-14 定案）；
 //! 暖机（D16（发布语义） 已定项 8）= 屏障 → 空记录 → 屏障 → 根槽 FUA → 超级块槽轮换；
 //! 第一个事务（D16（发布语义） 已定项 7）= 单元写 × 8 → 屏障 → journal 记录 → 屏障 → 根槽 FUA → 超级块槽轮换。
+//! 覆盖写（里程碑「第二个事务」步 1 / 步 2）走同一条骨架：新数据单元 COW 到新落点、六个提交内生块与树表 COW 出新版本，
+//! 被换下的八个单元在同一次发布里释放（分配记录改写成已释放 + 释放代，条目不删，D3（空间分配） 已定项 7）。
 //! 每一步都经过块设备接口，录制器挂在那层（D17（实现分层与第三方管道） 已定项 5）。
 
 use std::collections::BTreeMap;
@@ -21,7 +23,7 @@ use crate::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
     TreeIdentifier,
 };
-use crate::allocator::{AllocationRecord, PoolAllocator, UnitFootprint};
+use crate::allocator::{AllocationRecord, Placement, PoolAllocator, UnitFootprint};
 use crate::block_device::{BlockDevice, BlockDeviceError, WriteDurability};
 use crate::journal::{back_chain_of, record_offset, JournalRecord, NamedUnit};
 use crate::make_filesystem::{
@@ -492,6 +494,10 @@ pub struct TransactionOutput {
     pub allocation_records: Vec<AllocationRecord>,
     pub accounting_entries: Vec<AccountingEntry>,
     pub tree_table_entries: Vec<TreeTableEntry>,
+    /// 这次发布写出的 inode 记录（覆盖写要接着它的对象出生代）。
+    pub inode_record: InodeRecord,
+    /// 这次发布释放的落点（覆盖写换下的上一版八个单元；第一个事务为空）。
+    pub released: Vec<Placement>,
     /// C319（请求内单元按 key 升序发出没有条款也没有检查）的运行时计数：同一请求内取号序与 key 序不一致的次数，第一个事务恒 0。
     pub key_order_mismatches: u64,
 }
@@ -504,6 +510,22 @@ impl TransactionOutput {
             .find(|unit| unit.identity == identity)
             .expect("八个单元每种一个")
     }
+
+    /// 这次发布写出的八个落点，下一次覆盖写要把它们整批释放。
+    #[must_use]
+    pub fn placements(&self) -> Vec<Placement> {
+        self.units
+            .iter()
+            .map(|unit| Placement {
+                slot: unit.slot,
+                span: match unit.identity.placement() {
+                    PlacementRule::UserData => 2,
+                    PlacementRule::CommitGenerated(UnitFootprint::TwoSlotsAligned) => 2,
+                    PlacementRule::CommitGenerated(UnitFootprint::OneSlot) => 1,
+                },
+            })
+            .collect()
+    }
 }
 
 /// 发布能出的错：单元区放不下（哪一个单元没拿到落点），或底层块设备错。
@@ -565,11 +587,27 @@ pub struct FirstFile<'content> {
     pub write_time_seconds: u64,
 }
 
+/// 一次「文件的一个版本」的发布要的全部参数：第一个事务与覆盖写只差这些数（都是发布参数，不取系统时钟）。
+#[derive(Clone, Copy, Debug)]
+pub struct FilePublish<'content> {
+    pub txg: CheckpointTxg,
+    /// jsn 计数器（记录落在环里的槽位）。
+    pub counter: u64,
+    /// 事务号，按实例计数从 1 起（D23（journal 的角色与格式） 已定项 7）。
+    pub transaction: u64,
+    pub content: &'content [u8],
+    pub write_time_seconds: u64,
+    /// 对象出生代 = 创建那次发布的 checkpoint_txg，覆盖写不改（D8（核心索引结构） 已定项 6；I-9.10（对象出生代与 inode 记录相符））。
+    pub inode_object_birth: CheckpointTxg,
+    /// 改动计数（D8（核心索引结构） 已定项 6 偏移 88）。
+    pub change_count: u64,
+    /// 树表条目的诞生 txg：树建起来那次发布，覆盖写不改。
+    pub tree_birth_txg: CheckpointTxg,
+    /// 根记录照旧持有的实例表单元指针。
+    pub instance_table: NodePointer,
+}
+
 /// 第一个事务（字节表七 t1..t8 + 六 + 七）：分配八个落点、装八个单元、按 D16（发布语义） 已定项 7 的持久顺序落盘。
-#[allow(
-    clippy::too_many_lines,
-    reason = "一次发布就是一件能单独验证的事：八个单元的装法与一条持久顺序，拆开只会把顺序藏进几个函数"
-)]
 pub fn publish_first_file<Device: BlockDevice>(
     pool: &mut PoolWriter<'_, Device>,
     allocator: &mut PoolAllocator,
@@ -579,13 +617,93 @@ pub fn publish_first_file<Device: BlockDevice>(
     previous_record_bytes: &[u8],
 ) -> Result<TransactionOutput, PublishError> {
     let txg = CheckpointTxg(FIRST_TRANSACTION_TXG);
+    publish_file_version(
+        pool,
+        allocator,
+        FilePublish {
+            txg,
+            counter: FIRST_TRANSACTION_TXG,
+            transaction: FIRST_TRANSACTION_NUMBER,
+            content: file.content,
+            write_time_seconds: file.write_time_seconds,
+            inode_object_birth: txg,
+            change_count: 1,
+            tree_birth_txg: txg,
+            instance_table: genesis.instance_table,
+        },
+        instance,
+        previous_record_bytes,
+        &[],
+    )
+}
+
+/// 覆盖写（里程碑「第二个事务」步 1 / 步 2）：同一个实例里紧接着上一次发布，把同一个文件的内容整个换掉——
+/// 新数据单元 COW 到新落点、extent 叶记录的指针换成它、inode 记录更新（改动计数 = 这次发布的 checkpoint_txg），
+/// 六个提交内生块与树表单元各 COW 出新版本；上一版的八个单元在同一次发布里释放（释放代 = 这次的 txg）。
+/// txg、jsn、事务号各比上一次加一。
+pub fn publish_overwrite<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    previous: &TransactionOutput,
+    file: FirstFile<'_>,
+    instance: InstanceGeneration,
+) -> Result<TransactionOutput, PublishError> {
+    assert_eq!(
+        previous.root.instance, instance,
+        "覆盖写接在同一个实例的上一次发布之后"
+    );
+    let txg = CheckpointTxg(previous.root.checkpoint_txg.0 + 1);
+    let tree_birth_txg = previous
+        .tree_table_entries
+        .first()
+        .expect("树表第 1 版有七条")
+        .birth_txg;
+    publish_file_version(
+        pool,
+        allocator,
+        FilePublish {
+            txg,
+            counter: previous.record.counter + 1,
+            transaction: previous.record.transaction + 1,
+            content: file.content,
+            write_time_seconds: file.write_time_seconds,
+            inode_object_birth: previous.inode_record.object_birth,
+            change_count: txg.0,
+            tree_birth_txg,
+            instance_table: previous.root.instance_table,
+        },
+        instance,
+        &previous.record_bytes,
+        &previous.placements(),
+    )
+}
+
+/// 发布一个文件版本：释放上一版的落点、分配八个落点、装八个单元、按 D16（发布语义） 已定项 7 的持久顺序落盘。
+#[allow(
+    clippy::too_many_lines,
+    reason = "一次发布就是一件能单独验证的事：八个单元的装法与一条持久顺序，拆开只会把顺序藏进几个函数"
+)]
+fn publish_file_version<Device: BlockDevice>(
+    pool: &mut PoolWriter<'_, Device>,
+    allocator: &mut PoolAllocator,
+    publish: FilePublish<'_>,
+    instance: InstanceGeneration,
+    previous_record_bytes: &[u8],
+    release: &[Placement],
+) -> Result<TransactionOutput, PublishError> {
+    let txg = publish.txg;
     let write_order = WriteOrder {
         instance,
-        transaction: FIRST_TRANSACTION_NUMBER,
+        transaction: publish.transaction,
     };
     let filesystem_identifier = &pool.parameters.filesystem_identifier;
     let mut sequences = BirthSequenceAllocator::default();
 
+    // 释放先于分配（D3（空间分配） 已定项 7）：被换下的落点进 defer 队列、槽仍占着，这次的落点不会落到它们上面。
+    for placement in release {
+        allocator.release(*placement, txg);
+    }
+
     // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以八个落点在装任何单元之前全部取定。
     let mut slots: BTreeMap<TransactionUnit, SlotNumber> = BTreeMap::new();
     for identity in TransactionUnit::IN_BUMP_ORDER {
@@ -604,7 +722,7 @@ pub fn publish_first_file<Device: BlockDevice>(
     let data_identity = DataUnitIdentity {
         tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
         object: FIRST_INODE_NUMBER,
-        object_birth: txg,
+        object_birth: publish.inode_object_birth,
         anchor_offset: 0,
     };
     let data_unit = build_data_unit(
@@ -612,7 +730,7 @@ pub fn publish_first_file<Device: BlockDevice>(
         txg,
         filesystem_identifier,
         write_order,
-        file.content,
+        publish.content,
     );
     let data_pointer = DataPointer {
         head: PointerHead {
@@ -629,19 +747,20 @@ pub fn publish_first_file<Device: BlockDevice>(
     let key_order_mismatches = count_key_order_mismatches(&[extent_key]);
 
     // t3 inode 树叶容器（字节表四）：出生序号先于 t4 的根发号（树内先叶后根）。
+    // 容器身份（容器号、容器出生代）在树建起来那次定下，之后每一版重写都不变（D8（核心索引结构） 已定项 6：一片叶活很多代、反复重写）。
     let inode_leaf_identity = PackedIdentity {
         birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
         record_type: PACKED_TYPE_INODE,
         container: FIRST_INODE_NUMBER,
-        container_birth: txg,
+        container_birth: publish.tree_birth_txg,
     };
     let inode_leaf_sequence = sequences.next(TreeIdentifier(TREE_IDENTIFIER_INODE), txg, instance);
     let inode_record = InodeRecord {
         inode: FIRST_INODE_NUMBER,
-        object_birth: txg,
-        size: u64::try_from(file.content.len()).expect("文件长度"),
-        change_count: 1,
-        write_time_seconds: file.write_time_seconds,
+        object_birth: publish.inode_object_birth,
+        size: u64::try_from(publish.content.len()).expect("文件长度"),
+        change_count: publish.change_count,
+        write_time_seconds: publish.write_time_seconds,
     };
     let inode_leaf_unit = build_packed_unit(
         inode_leaf_identity,
@@ -766,7 +885,11 @@ pub fn publish_first_file<Device: BlockDevice>(
             device_map.free_slots() * SLOT_BYTES,
         ));
         accounting_entries.push(per_device(STATISTIC_UNRECLAIMABLE_BYTES, 0));
-        accounting_entries.push(per_device(STATISTIC_DEFER_QUEUE_BYTES, 0));
+        // 第 5 项：已释放、还在 defer 窗口里的（它们仍算在已分配里：占着空间、被根环里的有效根引用）。
+        accounting_entries.push(per_device(
+            STATISTIC_DEFER_QUEUE_BYTES,
+            device_map.deferred_slots() * SLOT_BYTES,
+        ));
         accounting_entries.push(per_device(
             STATISTIC_FRAGMENTATION_RUNS,
             device_map.free_runs(),
@@ -894,7 +1017,7 @@ pub fn publish_first_file<Device: BlockDevice>(
             kind,
             tree: TreeIdentifier(tree),
             root,
-            birth_txg: txg,
+            birth_txg: publish.tree_birth_txg,
             head_identifier,
         };
     let tree_table_entries = vec![
@@ -1059,9 +1182,9 @@ pub fn publish_first_file<Device: BlockDevice>(
         .collect();
     let record = JournalRecord {
         instance,
-        counter: FIRST_TRANSACTION_TXG,
+        counter: publish.counter,
         checkpoint_txg: txg,
-        transaction: FIRST_TRANSACTION_NUMBER,
+        transaction: publish.transaction,
         is_commit: true,
         back_chain: back_chain_of(previous_record_bytes),
         filesystem_identifier: unit_filesystem_identifier(filesystem_identifier),
@@ -1079,7 +1202,7 @@ pub fn publish_first_file<Device: BlockDevice>(
         tree_table: tree_table_pointer,
         tree_identifier_watermark: TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH,
         rollback_floor: CheckpointTxg(0),
-        instance_table: genesis.instance_table,
+        instance_table: publish.instance_table,
         mapping_root: mapping_pointer,
     };
     let root_slot = root.to_slot(pool.root_slot_bytes());
@@ -1093,7 +1216,7 @@ pub fn publish_first_file<Device: BlockDevice>(
     }
     pool.perform(CommitStep::Barrier)?;
     pool.perform(CommitStep::WriteJournalRecordToEveryDevice {
-        counter: FIRST_TRANSACTION_TXG,
+        counter: publish.counter,
         record: &record_bytes,
     })?;
     pool.perform(CommitStep::Barrier)?;
@@ -1102,7 +1225,7 @@ pub fn publish_first_file<Device: BlockDevice>(
         root_slot: &root_slot,
     })?;
     pool.perform(CommitStep::RotateSuperblockSlots {
-        journal_tail: FIRST_TRANSACTION_TXG,
+        journal_tail: publish.counter,
         journal_instance: instance,
     })?;
 
@@ -1116,6 +1239,8 @@ pub fn publish_first_file<Device: BlockDevice>(
         allocation_records,
         accounting_entries,
         tree_table_entries,
+        inode_record,
+        released: release.to_vec(),
         key_order_mismatches,
     })
 }
diff --git a/crates/singlefs-harness/src/crash.rs b/crates/singlefs-harness/src/crash.rs
index 7e8a866..2562509 100644
--- a/crates/singlefs-harness/src/crash.rs
+++ b/crates/singlefs-harness/src/crash.rs
@@ -526,6 +526,7 @@ pub struct Layer0Tally {
 }
 
 /// oracle（E77（发布的持久顺序） 判据 1）：读回的内容要对；根槽已持久就不许恢复到旧态；走读不许失败。
+/// 单版本形态：整条流只有一次带文件的发布（第一个事务）。
 #[must_use]
 pub fn oracle_violation(
     outcome: &RecoveryOutcome,
@@ -543,7 +544,77 @@ pub fn oracle_violation(
     }
 }
 
-/// 评一个状态：跑两种 journal 政策的恢复，记进计数。
+/// 文件的一个已发布版本：哪次发布（checkpoint_txg）写出了什么内容。多次发布的流上 oracle 按实际走的根判该读出哪一版。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct PublishedVersion {
+    pub checkpoint_txg: CheckpointTxg,
+    pub content: Vec<u8>,
+}
+
+/// 多版本形态的 oracle：实际走的根是哪一代，读回的就得是那一代写出的内容；根下面没有文件的那几代（mkfs、暖机）只许报没有文件；
+/// 盘上已持久的最新根槽是第 T 代，恢复就不许落到比 T 旧的根上（根槽已持久而恢复到旧态）；走读不许失败。
+#[must_use]
+pub fn oracle_violation_for_versions(
+    outcome: &RecoveryOutcome,
+    effective_root: Option<(InstanceGeneration, CheckpointTxg)>,
+    newest_persisted_root_txg: Option<CheckpointTxg>,
+    versions: &[PublishedVersion],
+) -> Option<String> {
+    let Some((_, effective_txg)) = effective_root else {
+        return Some("没择到根".to_string());
+    };
+    if newest_persisted_root_txg.is_some_and(|newest| effective_txg < newest) {
+        return Some(format!(
+            "根槽已持久而恢复到旧态（盘上最新的根槽是第 {} 代，走的是第 {} 代）",
+            newest_persisted_root_txg.expect("上面判过").0,
+            effective_txg.0
+        ));
+    }
+    let version = versions
+        .iter()
+        .find(|version| version.checkpoint_txg == effective_txg);
+    match (outcome, version) {
+        (RecoveryOutcome::Failed { failure, .. }, _) => Some(format!("走读失败：{failure:?}")),
+        (RecoveryOutcome::NoFile { .. }, None) => None,
+        (RecoveryOutcome::NoFile { .. }, Some(_)) => {
+            Some(format!("第 {} 代根下面有文件却报没有", effective_txg.0))
+        }
+        (RecoveryOutcome::FileRead { .. }, None) => Some(format!(
+            "第 {} 代根下面没有文件却读出了内容",
+            effective_txg.0
+        )),
+        (RecoveryOutcome::FileRead { content, .. }, Some(version)) => (*content != version.content)
+            .then(|| format!("读回的内容不对（走的是第 {} 代根）", effective_txg.0)),
+    }
+}
+
+/// 这一状态里持久了的根槽写中最新的那一代。
+#[must_use]
+pub fn newest_persisted_root_txg(
+    writes: &[RetainedWrite],
+    persisted: &[bool],
+) -> Option<CheckpointTxg> {
+    publishes_in(writes)
+        .iter()
+        .filter(|publish| persisted[publish.root])
+        .map(|publish| CheckpointTxg(publish.checkpoint_txg))
+        .max()
+}
+
+fn root_txg_of_write(write: &RetainedWrite) -> CheckpointTxg {
+    assert_eq!(
+        write.kind,
+        StepKind::RootRecordFua,
+        "被判的那条写要是根槽 FUA 写"
+    );
+    CheckpointTxg(u64::from_le_bytes(
+        write.bytes[28..36]
+            .try_into()
+            .expect("根记录的 checkpoint_txg 在偏移 28"),
+    ))
+}
+
+/// 单版本形态：被判的那次根槽写出的那一代就是唯一带文件的版本。
 pub fn evaluate_state(
     base: &MemoryPool,
     writes: &[RetainedWrite],
@@ -552,7 +623,25 @@ pub fn evaluate_state(
     expected_content: &[u8],
     tally: &mut Layer0Tally,
 ) -> RecoveryReport {
-    let root_persisted = persisted[root_index];
+    let versions = [PublishedVersion {
+        checkpoint_txg: root_txg_of_write(&writes[root_index]),
+        content: expected_content.to_vec(),
+    }];
+    evaluate_state_for_versions(base, writes, persisted, root_index, &versions, tally)
+}
+
+/// 评一个状态：跑两种 journal 政策的恢复，记进计数。`judged_root_index` 是被判的那次根槽 FUA 写（计「根槽已持久」的状态数用），
+/// oracle 按 `versions` 判实际走的根该读出哪一版。
+pub fn evaluate_state_for_versions(
+    base: &MemoryPool,
+    writes: &[RetainedWrite],
+    persisted: Vec<bool>,
+    judged_root_index: usize,
+    versions: &[PublishedVersion],
+    tally: &mut Layer0Tally,
+) -> RecoveryReport {
+    let root_persisted = persisted[judged_root_index];
+    let newest_persisted = newest_persisted_root_txg(writes, &persisted);
     let image = CrashImage {
         base,
         writes,
@@ -578,7 +667,12 @@ pub fn evaluate_state(
         RecoveryOutcome::FileRead { .. } => tally.file_read_states += 1,
         RecoveryOutcome::Failed { .. } => tally.failed_states += 1,
     }
-    if let Some(reason) = oracle_violation(&consulted.outcome, root_persisted, expected_content) {
+    if let Some(reason) = oracle_violation_for_versions(
+        &consulted.outcome,
+        consulted.effective_root,
+        newest_persisted,
+        versions,
+    ) {
         tally.violations += 1;
         if tally.first_violation.is_none() {
             let persisted_kinds: Vec<&str> = image
@@ -620,8 +714,7 @@ pub fn evaluate_state(
     consulted
 }
 
-/// 枚举：前面的段全持久 + 当前段任意真子集，最后再加全部持久那一个状态；`expand` 决定哪一段展开子集
-/// （不展开的段只以整段持久进入后面的状态，平时 `cargo test` 里跳过 18 个写那一段就靠它）。`root_index` 是被判的那次根槽 FUA 写。
+/// 单版本形态的枚举。
 #[must_use]
 pub fn enumerate_layer0_selecting(
     base: &MemoryPool,
@@ -630,6 +723,24 @@ pub fn enumerate_layer0_selecting(
     root_index: usize,
     expected_content: &[u8],
     expand: &dyn Fn(usize, &[usize]) -> bool,
+) -> Layer0Tally {
+    let versions = [PublishedVersion {
+        checkpoint_txg: root_txg_of_write(&writes[root_index]),
+        content: expected_content.to_vec(),
+    }];
+    enumerate_layer0_selecting_versions(base, writes, segments, root_index, &versions, expand)
+}
+
+/// 枚举：前面的段全持久 + 当前段任意真子集，最后再加全部持久那一个状态；`expand` 决定哪一段展开子集
+/// （不展开的段只以整段持久进入后面的状态，平时 `cargo test` 里跳过 18 个写那一段就靠它）。`judged_root_index` 是被判的那次根槽 FUA 写。
+#[must_use]
+pub fn enumerate_layer0_selecting_versions(
+    base: &MemoryPool,
+    writes: &[RetainedWrite],
+    segments: &[Vec<usize>],
+    judged_root_index: usize,
+    versions: &[PublishedVersion],
+    expand: &dyn Fn(usize, &[usize]) -> bool,
 ) -> Layer0Tally {
     let mut tally = Layer0Tally::default();
     let mut persisted_before = vec![false; writes.len()];
@@ -643,12 +754,12 @@ pub fn enumerate_layer0_selecting(
                         persisted[*write_index] = true;
                     }
                 }
-                evaluate_state(
+                evaluate_state_for_versions(
                     base,
                     writes,
                     persisted,
-                    root_index,
-                    expected_content,
+                    judged_root_index,
+                    versions,
                     &mut tally,
                 );
             }
@@ -657,17 +768,36 @@ pub fn enumerate_layer0_selecting(
             persisted_before[*write_index] = true;
         }
     }
-    evaluate_state(
+    evaluate_state_for_versions(
         base,
         writes,
         persisted_before,
-        root_index,
-        expected_content,
+        judged_root_index,
+        versions,
         &mut tally,
     );
     tally
 }
 
+/// 全量、多版本：每一段都展开。
+#[must_use]
+pub fn enumerate_layer0_versions(
+    base: &MemoryPool,
+    writes: &[RetainedWrite],
+    segments: &[Vec<usize>],
+    judged_root_index: usize,
+    versions: &[PublishedVersion],
+) -> Layer0Tally {
+    enumerate_layer0_selecting_versions(
+        base,
+        writes,
+        segments,
+        judged_root_index,
+        versions,
+        &|_segment_index, _segment| true,
+    )
+}
+
 /// 全量：每一段都展开。
 #[must_use]
 pub fn enumerate_layer0(
```

## 二、新测试文件 `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs` 全文

```rust
//! 里程碑「第二个事务」步 1 / 步 2 的验收：第一个事务之后、同一个实例里对同一个文件覆盖写一次（发布 B），
//! 冷启动读回第二次的内容；录制流的段序列与第一个事务同型 `16+2+1+2`；被换下的八个单元在分配记录树里改写成
//! 已释放 + 释放代 4、记账的 defer 待释放行等于它们的字节数、已分配行仍把它们算在内（I-3.1 读法甲：根环里 A 的根还引用着它们）；
//! 池级 checker 全绿；坏字节探针：新单元两份都坏 ⇒ 恢复失败，旧单元两份都坏 ⇒ 最新根不受影响，B 的根槽坏一字节 ⇒ 由记录重建。

mod common;

use std::collections::BTreeSet;

use common::{
    build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
    IMAGE_BYTES,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::journal::{back_chain_of, record_offset};
use singlefs_core::records::{
    STATISTIC_ALLOCATED_BYTES, STATISTIC_DEFER_QUEUE_BYTES, STATISTIC_EMPTY_CLUSTER_SEGMENTS,
    STATISTIC_FRAGMENTATION_RUNS, STATISTIC_FREE_BYTES, STATISTIC_INODE_WATERMARK,
};
use singlefs_core::recovery::{
    choose_superblock, recover, JournalPolicy, JournalScanReport, PoolReader, RecoveryFailure,
    RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, TransactionOutput, TransactionUnit,
};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES, UNIT_AREA_START_SLOT};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::segments::{segment_kinds_text, segment_sizes_text, split_into_segments};

/// 第二次写的内容：与第一次不同长、不同字节。
const SECOND_FILE_BYTES: usize = 4100;

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 在第一个事务之后、同一个进程同一个实例里覆盖写一次；返回 B 的输出与它在录制流里的起点。
fn overwrite(pool: &mut BuiltPool) -> (TransactionOutput, usize) {
    let parameters = parameters();
    let operations_before = pool.stream.operations().len();
    let content = second_content();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &pool.output,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    )
    .expect("覆盖写");
    (output, operations_before)
}

fn slot_of(output: &TransactionOutput, identity: TransactionUnit) -> SlotNumber {
    output.unit(identity).slot
}

#[test]
fn overwrite_publishes_the_second_version_through_the_same_commit_shape() {
    let mut pool = build_pool("overwrite-shape");
    let (second, operations_before) = overwrite(&mut pool);
    let first = &pool.output;

    let operations = pool.stream.operations();
    let segments = split_into_segments(&operations[operations_before..], &geometry());
    assert_eq!(
        segment_sizes_text(&segments),
        "16+2+1+2",
        "覆盖写的段序列与第一个事务同型（登记表八 B 那一行的预想）"
    );
    assert_eq!(
        segment_kinds_text(&segments),
        "[unit_write×16,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]"
    );

    assert_eq!(second.root.checkpoint_txg, CheckpointTxg(4));
    assert_eq!(second.root.instance, InstanceGeneration(1));
    assert_eq!(
        second.root.tree_identifier_watermark, 19,
        "没有建新树，水位不动"
    );
    assert_eq!(second.root.rollback_floor, CheckpointTxg(0));
    assert_eq!(
        second.root.instance_table, first.root.instance_table,
        "实例表单元照旧"
    );
    assert_ne!(
        second.root.mapping_root, first.root.mapping_root,
        "映射树 COW 出新版本"
    );
    assert_eq!(second.record.counter, 4, "jsn 接着 A 的 3");
    assert_eq!(
        second.record.transaction, 2,
        "事务号按实例计数从 1 起，B 是 2"
    );
    assert_eq!(second.record.checkpoint_txg, CheckpointTxg(4));
    assert_eq!(
        second.record.back_chain,
        back_chain_of(&first.record_bytes),
        "反向链 = A 那条记录头的 CRC32C"
    );
    assert_eq!(second.record.named.len(), 8, "点名项 = 这次新写的单元数");

    let expected_slots = [
        (TransactionUnit::Data, 50182),
        (TransactionUnit::ExtentRoot, 50249),
        (TransactionUnit::InodeLeaf, 50250),
        (TransactionUnit::InodeRoot, 50252),
        (TransactionUnit::AllocationTree, 50253),
        (TransactionUnit::AccountingTree, 50254),
        (TransactionUnit::MappingTree, 50255),
        (TransactionUnit::TreeTable, 50256),
    ];
    for (identity, slot) in expected_slots {
        assert_eq!(
            slot_of(&second, identity),
            SlotNumber(slot),
            "{}：数据单元落 50180 之后最低的偶数空槽对，提交内生块接着 A 的 bump 往后",
            identity.tag()
        );
    }
    assert_eq!(
        second.released,
        first.placements(),
        "释放的正是 A 写出的八个落点"
    );
    assert_eq!(
        second.inode_record.object_birth,
        CheckpointTxg(3),
        "对象出生代不改"
    );
    assert_eq!(
        second.inode_record.change_count, 4,
        "改动计数 = 这次发布的 checkpoint_txg"
    );
    assert_eq!(
        second.inode_record.size,
        u64::try_from(SECOND_FILE_BYTES).expect("长度")
    );
    assert_eq!(
        second.inode_record.write_time_seconds,
        FIXED_WRITE_TIME_SECONDS + 60
    );
    assert_eq!(second.key_order_mismatches, 0);
    assert_eq!(pool.allocator.policy_mismatches, 0);

    // 根槽落区域 4 mod 3 = 1 的槽 (4 div 3) mod 8 = 1，区域 1 归盘 1；超级块世代号 6、tail = 4。
    let image = pool.memory_pool();
    let target = target_for_publish(CheckpointTxg(4));
    let region_device =
        parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    assert_eq!(region_device, DeviceIdentity(1));
    let root_bytes =
        PoolReader::read(&image, region_device, slot_offset(target, 4096), 512).expect("读根槽");
    let root = RootRecord::parse_slot(&root_bytes, &parameters().filesystem_identifier)
        .expect("根槽自证过");
    assert_eq!(root, second.root);
    let superblock = choose_superblock(&image).expect("超级块");
    assert_eq!(superblock.journal_tail, 4);
    assert_eq!(
        superblock.slot_generation, 6,
        "mkfs 1、取号 2、暖机 3 / 4、A 5、B 6"
    );
}

#[test]
fn release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue() {
    let mut pool = build_pool("overwrite-release");
    let first_slots: BTreeSet<SlotNumber> =
        pool.output.units.iter().map(|unit| unit.slot).collect();
    let first_mapping_keys: BTreeSet<Vec<u8>> = pool.output.mapping_keys.iter().cloned().collect();
    let (second, _) = overwrite(&mut pool);
    let second_slots: BTreeSet<SlotNumber> = second.units.iter().map(|unit| unit.slot).collect();

    assert_eq!(
        second.allocation_records.len(),
        36,
        "mkfs 2 + A 8 + B 8 个落点 × 2 盘：A 的改写不删"
    );
    let mut released_count = 0;
    let mut fresh_count = 0;
    let mut format_time_count = 0;
    for record in &second.allocation_records {
        if first_slots.contains(&record.slot) {
            assert!(
                record.is_released,
                "A 的落点 {} 改写成已释放",
                record.slot.0
            );
            assert_eq!(record.generation, CheckpointTxg(4), "释放代 = B 的 txg");
            released_count += 1;
        } else if second_slots.contains(&record.slot) {
            assert!(!record.is_released);
            assert_eq!(record.generation, CheckpointTxg(4), "分配代 = B 的 txg");
            fresh_count += 1;
        } else {
            assert_eq!(record.generation, CheckpointTxg(0), "mkfs 的单元分配代 0");
            format_time_count += 1;
        }
    }
    assert_eq!(
        (released_count, fresh_count, format_time_count),
        (16, 16, 4)
    );

    let unit_area_slots = IMAGE_BYTES / SLOT_BYTES - UNIT_AREA_START_SLOT;
    let occupied_slots = 3 + 10 + 10;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let value = |statistic: u16| {
            second
                .accounting_entries
                .iter()
                .find(|entry| entry.statistic == statistic && entry.device == device)
                .map(|entry| entry.value)
                .expect("带设备维的行每盘一行")
        };
        assert_eq!(
            value(STATISTIC_ALLOCATED_BYTES),
            occupied_slots * SLOT_BYTES,
            "已分配 = mkfs 3 槽 + A 10 槽 + B 10 槽：A 的单元仍被根环里 A 的根引用、仍占着空间（I-3.1 读法甲）"
        );
        assert_eq!(
            value(STATISTIC_DEFER_QUEUE_BYTES),
            10 * SLOT_BYTES,
            "defer 待释放 = A 的八个单元 10 槽"
        );
        assert_eq!(
            value(STATISTIC_FREE_BYTES),
            (unit_area_slots - occupied_slots) * SLOT_BYTES,
            "空闲 + 已分配 = 单元区（I-5.2），已释放的不算空闲"
        );
        assert_eq!(
            value(STATISTIC_FRAGMENTATION_RUNS),
            4,
            "[50179]、[50184, 50239]、[50241]、[50257, 末]"
        );
        assert_eq!(
            value(STATISTIC_EMPTY_CLUSTER_SEGMENTS),
            3310,
            "B 的提交内生块仍在 A 开的那个段里"
        );
    }
    let watermark = second
        .accounting_entries
        .iter()
        .find(|entry| entry.statistic == STATISTIC_INODE_WATERMARK)
        .expect("inode 号水位");
    assert_eq!(watermark.value, 2, "没有建新 inode");
    assert_eq!(second.accounting_entries.len(), 15);

    assert_eq!(second.mapping_keys.len(), 6, "码 1 一条 + 码 2 / 码 3 五条");
    for key in &second.mapping_keys {
        assert!(
            !first_mapping_keys.contains(key),
            "A 的映射条目一条都不留：释放走映射、条目删掉"
        );
    }
    assert_eq!(second.tree_table_entries.len(), 7);
    for (entry, previous) in second
        .tree_table_entries
        .iter()
        .zip(&pool.output.tree_table_entries)
    {
        assert_eq!(entry.tree, previous.tree);
        assert_eq!(entry.birth_txg, CheckpointTxg(3), "树的诞生 txg 不随重写变");
    }
    assert_eq!(
        second.tree_table_entries[0].root.head.birth_txg,
        CheckpointTxg(4),
        "extent 树根 COW 到 txg 4"
    );
}

#[test]
fn cold_start_reads_the_second_content_and_the_pool_checker_stays_green() {
    let mut pool = build_pool("overwrite-cold");
    let (_, _) = overwrite(&mut pool);
    let image = pool.memory_pool();
    let reopened = pool.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(4)),
            content: second_content()
        },
        "冷启动择 B 的根、读回第二次的内容"
    );
    assert_eq!(
        report.journal,
        JournalScanReport {
            valid_records: 4,
            above_water: 0,
            prefix_applied: 0,
            verification_passed: 0,
            verification_failed: 0
        },
        "暖机两条 + A + B 四条记录，没有一条高于所选根"
    );
    assert_eq!(report.mapping_fallbacks, 0);
    assert_eq!(
        recover(&reopened, JournalPolicy::Ignore).outcome,
        report.outcome,
        "根槽已持久：看不看 journal 结果一样"
    );
    assert_ne!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(4)),
            content: file_content()
        },
        "第一次的内容从最新根出发读不到"
    );

    let verdicts = check_pool_image(&image);
    for (invariant, verdict) in &verdicts {
        assert!(
            !matches!(verdict, InvariantVerdict::Violated(_)),
            "{invariant} 在覆盖写之后的镜像上判红：{verdict:?}"
        );
    }
    for must_hold in ["I-3.1", "I-5.1", "I-5.2", "I-7.2"] {
        let (_, verdict) = verdicts
            .iter()
            .find(|(invariant, _)| *invariant == must_hold)
            .expect("checker 报了这一条");
        assert_eq!(
            verdict,
            &InvariantVerdict::Holds,
            "{must_hold} 要真被评估过、且成立"
        );
    }
}

#[test]
fn damage_probes_after_the_overwrite_tell_the_new_unit_from_the_released_one() {
    let mut pool = build_pool("overwrite-probes");
    let (second, _) = overwrite(&mut pool);
    let full = pool.memory_pool();
    let second_data = slot_of(&second, TransactionUnit::Data).to_device_offset();
    let first_data = pool
        .output
        .unit(TransactionUnit::Data)
        .slot
        .to_device_offset();
    let both = |image: &MemoryPool, offset, byte| {
        let mut damaged = image.clone();
        damaged.flip_byte(DeviceIdentity(0), offset, byte);
        damaged.flip_byte(DeviceIdentity(1), offset, byte);
        damaged
    };
    let file_read = RecoveryOutcome::FileRead {
        root: (InstanceGeneration(1), CheckpointTxg(4)),
        content: second_content(),
    };

    let new_unit_damaged = both(&full, second_data, 200);
    let new_unit_report = recover(&new_unit_damaged, JournalPolicy::Consult);
    assert_eq!(
        new_unit_report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), CheckpointTxg(4))),
            failure: RecoveryFailure::MappingStillUnreadable {
                slot: SlotNumber(50182)
            }
        },
        "新单元两份都坏：提示读不到、经映射仍读不到，不返回坏数据"
    );
    assert_eq!(new_unit_report.mapping_fallbacks, 1);

    let released_unit_damaged = both(&full, first_data, 200);
    assert_eq!(
        recover(&released_unit_damaged, JournalPolicy::Consult).outcome,
        file_read,
        "被换下的旧单元两份都坏：最新根不引用它，读回不受影响"
    );

    let record_damaged = both(&full, record_offset(4, JOURNAL_RING_DEFAULT_BYTES), 200);
    let record_damaged_report = recover(&record_damaged, JournalPolicy::Consult);
    assert_eq!(
        record_damaged_report.outcome, file_read,
        "B 的根槽已持久，记录坏了不碍事"
    );
    assert_eq!(record_damaged_report.journal.valid_records, 3);

    let target = target_for_publish(CheckpointTxg(4));
    let region_device =
        parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let mut root_damaged = full.clone();
    root_damaged.flip_byte(region_device, slot_offset(target, 4096), 100);
    let root_damaged_report = recover(&root_damaged, JournalPolicy::Consult);
    assert_eq!(
        root_damaged_report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: second_content()
        },
        "B 的根槽坏一字节：择回 A 的根，再由 jsn 4 那条记录重建第 4 代根，内容仍是第二次的"
    );
    assert_eq!(
        root_damaged_report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4)))
    );
    assert_eq!(root_damaged_report.journal.prefix_applied, 1);
    assert_eq!(root_damaged_report.journal.verification_passed, 1);
    assert_eq!(
        recover(&root_damaged, JournalPolicy::Ignore).outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: file_content()
        },
        "不看 journal 就退回 A：第一次的内容——journal 在这一格承重"
    );
}
```

## 二、新测试文件 `crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` 全文

```rust
//! 里程碑「第二个事务」步 0 在发布 B 上的那一半：把「取号 → 暖机 → A → B」整条录制流按 D13（验证路线） 已定项 4 枚举全部崩溃状态，
//! 与第一个事务同一种切法（上一次发布的超级块槽写与下一次发布的单元写落在同一段，登记表八末尾那条 ⚠️），
//! 每个状态跑恢复 + 多版本 oracle（实际走的根是哪一代就得读出那一代的内容）、池级 checker、记录核对器。
//! 平时 `cargo test` 跳过两个 18 写的段；全量那条标 ignored，54 号门禁在 release 下跑它。再加四组靶向的阳性对照。

mod common;

use common::{build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::recovery::RecoveryOutcome;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter};
use singlefs_harness::crash::{
    closed_form_state_count, enumerate_layer0_selecting_versions, enumerate_layer0_versions,
    evaluate_state_for_versions, writes_and_segments, Layer0Tally, MemoryPool, PublishedVersion,
    RetainedWrite,
};
use singlefs_harness::segments::StepKind;

const SECOND_FILE_BYTES: usize = 4100;

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    /// B 的根槽 FUA 写在写表里的下标。
    judged_root_index: usize,
    /// A 的根槽 FUA 写在写表里的下标：它之后的写都是 B 的。
    first_root_index: usize,
    versions: Vec<PublishedVersion>,
}

fn overwrite(pool: &mut BuiltPool) {
    let parameters = parameters();
    let content = second_content();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &pool.output,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    )
    .expect("覆盖写");
}

fn prepare(tag: &str) -> Prepared {
    let mut pool = build_pool(tag);
    overwrite(&mut pool);
    let base = pool.memory_pool_after_mkfs();
    let operations = pool.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[pool.mkfs_operation_count..], &geometry());
    assert_eq!(
        segments.iter().map(Vec::len).collect::<Vec<_>>(),
        vec![2, 2, 1, 2, 2, 1, 18, 2, 1, 18, 2, 1, 2],
        "A 的两个超级块槽写与 B 的 16 个单元写合成一段：整条流按屏障切，不按发布切"
    );
    assert_eq!(writes.len(), 54, "取号 2 + 暖机 10 + A 21 + B 21 次写");
    let root_indexes: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indexes.len(), 4, "暖机两代 + A + B 四条根槽写");
    Prepared {
        base,
        writes,
        segments,
        judged_root_index: root_indexes[3],
        first_root_index: root_indexes[2],
        versions: vec![
            PublishedVersion {
                checkpoint_txg: CheckpointTxg(3),
                content: file_content(),
            },
            PublishedVersion {
                checkpoint_txg: CheckpointTxg(4),
                content: second_content(),
            },
        ],
    }
}

fn assert_checker_and_record_checker_clean(tally: &Layer0Tally) {
    assert_eq!(
        (
            tally.record_root_without_record,
            tally.record_claimed_state_missing_unit
        ),
        (0, 0),
        "记录核对器两条判据在已定的持久顺序下恒 0"
    );
    for invariant in singlefs_checker::image::IMPLEMENTED_INVARIANTS {
        assert_eq!(
            tally
                .checker_violated_states
                .get(invariant)
                .copied()
                .unwrap_or(0),
            0,
            "{invariant} 在两次发布的流上判违例的状态数必须是 0"
        );
    }
    for must_evaluate in ["I-3.1", "I-5.2", "I-5.1", "I-7.2"] {
        assert!(
            tally
                .checker_evaluated_states
                .get(must_evaluate)
                .copied()
                .unwrap_or(0)
                > 0,
            "{must_evaluate} 至少在一个状态上真被评估过"
        );
    }
}

/// 平时跑的那一份：两个 18 写的段不展开（只以整段持久进入后面的状态），其余每段任意子集。
#[test]
fn every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims() {
    let prepared = prepare("layer0-b-fast");
    let expand = |_segment_index: usize, segment: &[usize]| segment.len() < 18;
    let tally = enumerate_layer0_selecting_versions(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
        &expand,
    );
    let expanded: Vec<Vec<usize>> = prepared
        .segments
        .iter()
        .filter(|segment| segment.len() < 18)
        .cloned()
        .collect();
    assert_eq!(
        tally.states,
        closed_form_state_count(&expanded),
        "展开的段按闭式数"
    );
    assert_eq!(tally.states, 26);
    assert_eq!(
        tally.violations, 0,
        "第一处违例：{:?}",
        tally.first_violation
    );
    assert!(tally.file_read_states > 0 && tally.no_file_states > 0);
    assert_checker_and_record_checker_clean(&tally);
}

/// 全量：两次发布的流 13 段、闭式 524312 个状态（第一个事务的 262165 减掉末尾那个 2 写的段、加上 B 的四段与合成的 18 写段）。
/// 54 号门禁在 release 下跑它，认下面打印的 `LAYER0B` 行里 `exhaustive=true`。
#[test]
#[ignore = "全量 524312 个状态、每个两遍恢复 + checker，debug 下几分钟；门禁 54 号在 release 下跑"]
fn full_enumeration_of_the_two_publish_stream_is_exhaustive_and_clean() {
    let prepared = prepare("layer0-b-full");
    let closed_form = closed_form_state_count(&prepared.segments);
    assert_eq!(closed_form, 524_312, "闭式：1 + Σ(2^|段| − 1)，十三段");
    let tally = enumerate_layer0_versions(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        prepared.judged_root_index,
        &prepared.versions,
    );
    let checker_violations: u64 = tally.checker_violated_states.values().sum();
    println!(
        "LAYER0B states={} closed_form={closed_form} exhaustive={} violations={} root_persisted_states={} no_file={} file_read={} failed={} journal_differing={} verification_ran={} verification_failed={} record_root_without_record={} record_claimed_state_missing_unit={} checker_violations={checker_violations} first_violation={}",
        tally.states,
        tally.states == closed_form,
        tally.violations,
        tally.root_persisted_states,
        tally.no_file_states,
        tally.file_read_states,
        tally.failed_states,
        tally.journal_differing_states,
        tally.verification_ran_states,
        tally.verification_failed_states,
        tally.record_root_without_record,
        tally.record_claimed_state_missing_unit,
        tally.first_violation.as_deref().unwrap_or("none")
    );
    assert_eq!(tally.states, closed_form, "枚举到的状态数要等于闭式");
    assert_eq!(
        tally.violations, 0,
        "第一处违例：{:?}",
        tally.first_violation
    );
    assert_eq!(tally.failed_states, 0);
    assert_checker_and_record_checker_clean(&tally);
}

/// 靶向的阳性对照：把持久集合手工摆成四个形状，oracle、journal 承重、记录核对器各要在它该红的那一格红。
#[test]
fn targeted_controls_on_the_second_publish_go_red_where_they_should() {
    let prepared = prepare("layer0-b-controls");
    let all = vec![true; prepared.writes.len()];
    let is_b = |index: usize| index > prepared.first_root_index;
    let kinds_of_b = |kind: StepKind| -> Vec<usize> {
        prepared
            .writes
            .iter()
            .enumerate()
            .filter(|(index, write)| is_b(*index) && write.kind == kind)
            .map(|(index, _)| index)
            .collect()
    };
    let b_units = kinds_of_b(StepKind::UnitWrite);
    let b_records = kinds_of_b(StepKind::JournalRecord);
    assert_eq!((b_units.len(), b_records.len()), (16, 2));
    let evaluate = |persisted: Vec<bool>| {
        let mut tally = Layer0Tally::default();
        let report = evaluate_state_for_versions(
            &prepared.base,
            &prepared.writes,
            persisted,
            prepared.judged_root_index,
            &prepared.versions,
            &mut tally,
        );
        (report, tally)
    };

    // ① 全部持久：走 B 的根、读回第二次的内容，零违例。
    let (report, tally) = evaluate(all.clone());
    assert_eq!(
        report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4)))
    );
    assert!(matches!(report.outcome, RecoveryOutcome::FileRead { .. }));
    assert_eq!(tally.violations, 0);

    // ② B 的根槽已持久、B 的十六个单元写一份都没持久：oracle 必须红（走读失败），记录核对器判「自称新态而单元缺席」。
    let mut root_without_units = all.clone();
    for index in &b_units {
        root_without_units[*index] = false;
    }
    let (_, tally) = evaluate(root_without_units);
    assert_eq!(
        tally.violations, 1,
        "根槽已持久而单元不在：{:?}",
        tally.first_violation
    );
    assert_eq!(tally.record_claimed_state_missing_unit, 1);

    // ③ B 的根槽没持久、记录与单元都持久：看 journal 由记录重建第 4 代根读出第二次的内容，不看就退回 A——journal 在这一格承重。
    let mut root_missing = all.clone();
    root_missing[prepared.judged_root_index] = false;
    let (report, tally) = evaluate(root_missing);
    assert_eq!(
        report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4)))
    );
    assert_eq!(report.journal.prefix_applied, 1);
    assert_eq!(tally.journal_differing_states, 1);
    assert_eq!(tally.violations, 0);

    // ④ B 的根槽已持久、两份记录都没持久：走读没问题（根记录自带全部字段）、oracle 不红，但记录核对器判「根在案而记录缺席」。
    let mut root_without_records = all.clone();
    for index in &b_records {
        root_without_records[*index] = false;
    }
    let (_, tally) = evaluate(root_without_records);
    assert_eq!(tally.violations, 0);
    assert_eq!(tally.record_root_without_record, 1);

    // ⑤ B 一个字节都没持久：走 A 的根、读回第一次的内容，零违例——旧态在它自己的根下面是合法的。
    let mut nothing_of_b = all;
    for index in 0..prepared.writes.len() {
        if is_b(index) {
            nothing_of_b[index] = false;
        }
    }
    let (report, tally) = evaluate(nothing_of_b);
    assert_eq!(
        report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(3)))
    );
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: file_content()
        }
    );
    assert_eq!(tally.violations, 0);
}
```

