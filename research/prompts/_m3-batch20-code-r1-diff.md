# 删除批第 20 批合入的改动原样（2026-10-05）

## 实现员 a：crates.patch（/tmp/claude-1000/m3-batch20/a/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
index 01e9b2f..b356528 100644
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -83,8 +83,8 @@ pub const SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY: &str =
 /// GPU 判器内核还没判它（`crash_judge_gpu` 那一侧要补，归 GPU 那一簇）。
 ///
 /// 第 70–74 条（下标 69–73）是里程碑三·3 删除线那五条（判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节，
-/// 用户 2026-10-05 定 I-4.4、I-4.6 取乙）：I-9.5、I-3.4、I-4.7、I-4.6、I-4.4，同样只往末尾追加；GPU 判器内核还没判它们
-/// （`crash_judge_gpu` 那一侧要补，归 GPU 那一簇）。
+/// 用户 2026-10-05 定 I-4.4、I-4.6 取乙）：I-9.5、I-3.4、I-4.7、I-4.6、I-4.4，同样只往末尾追加；GPU 判器的扫描内核判前四条
+/// （下标 69–72，删除态扫描，第 18 批 P2），下标 73 的 I-4.4 第一版恒报不适用、内核不置位（`crash_judge_gpu` 的覆盖表）。
 pub const IMPLEMENTED_INVARIANTS: [&str; 74] = [
     "I-1.1",
     "I-1.2",
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
index 4045711..797ddc1 100644
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -7908,6 +7908,7 @@ enum DeletionScanGap {
     TreeTableUnreadable,
     TreeTableEntryShorterThanTheFieldTable,
     DeletedInodesHeadIdentifierNamesNoSingleInodeTree,
+    InodeTreeNamedByNoSingleDeletedInodesEntry,
     DeletedInodesRootUnreadable,
     InodeTreeUnreadable,
     ExtentTreeOfTheInodeTreeNotSingle,
@@ -7928,6 +7929,9 @@ impl DeletionScanGap {
             DeletionScanGap::DeletedInodesHeadIdentifierNamesNoSingleInodeTree => {
                 "走读到的某一版 deleted_inodes 条目偏移 116 的头 ID 配不上同一张树表里恰好一条 inode 树条目：那一版判不了"
             }
+            DeletionScanGap::InodeTreeNamedByNoSingleDeletedInodesEntry => {
+                "走读到的某一版树表里有一棵 inode 树，同一张表里偏移 116 的头 ID 指着它的 deleted_inodes 条目不是恰好一条：那一版这棵树的队列与记录判不了"
+            }
             DeletionScanGap::DeletedInodesRootUnreadable => {
                 "走读到的某一版 deleted_inodes 根读不出，或不是层级 0、key 宽与条目宽 8 的码 2 根：那一版的队列判不了"
             }
@@ -8305,7 +8309,8 @@ fn judge_pending_deletion_occupancy_of_a_version(
 }
 
 /// I-9.5（两个方向）、I-4.7（对象取丙）、I-4.6（取乙）在一版上判：这一版树表里每一条 `deleted_inodes` 条目经偏移 116 的头 ID 配同一张表里
-/// 树 ID 等于它的那一条 inode 树条目（判决第二节 F4-b；配不上、配到不止一条记判不了），extent 树同样按头 ID 配这棵 inode 树。
+/// 树 ID 等于它的那一条 inode 树条目（判决第二节 F4-b；配不上、配到不止一条记判不了），extent 树同样按头 ID 配这棵 inode 树；
+/// 反方向也配：每一棵 inode 树都要恰好一条 `deleted_inodes` 条目指着它，不是恰好一条的记判不了。
 /// I-9.5 每一版每个方向各判一次（队列与 nlink = 0 的号集合两两比，干净的一版也真比过）；I-4.7 每个处于删除中的号判一次、
 /// I-4.6 每条 nlink = 0 的记录判一次（一个都没有时没有对象）。
 fn judge_deletion_queue_of_a_version(
@@ -8403,6 +8408,31 @@ fn judge_deletion_queue_of_a_version(
             }
         }
     }
+    // 反方向也配（代码轮判决 `research/prompts/m3-batch18-code-r1-main-verification.md` 第二节「K2 漏判 L1」，改法 A）：上面那一遍只从
+    // deleted_inodes 条目一侧去配，树表里没有指着某棵 inode 树的 deleted_inodes 条目时那棵树一次都不判、也不记判不了。这里每一棵 inode 树
+    // 都要恰好一条 deleted_inodes 条目的头 ID 指着它，不是恰好一条的记判不了（判决 F4-b「配不上记判不了」）。放在上面那一遍之后：
+    // deleted_inodes 头 ID 配不上 inode 树的那一形同时也是这一形，先记上面那一句更近的理由。
+    for inode_tree_entry in tree_table_entries
+        .iter()
+        .filter(|entry| kind_of(entry) == TREE_KIND_INODE)
+    {
+        let inode_tree = read_u64(inode_tree_entry, 0);
+        let deleted_inodes_entries_naming_the_inode_tree = tree_table_entries
+            .iter()
+            .filter(|entry| {
+                kind_of(entry) == TREE_KIND_DELETED_INODES
+                    && head_identifier_of(entry) == inode_tree
+            })
+            .count();
+        if deleted_inodes_entries_naming_the_inode_tree != 1 {
+            for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
+                judgements.part_of_the_range_not_judged(
+                    invariant,
+                    DeletionScanGap::InodeTreeNamedByNoSingleDeletedInodesEntry.reason(),
+                );
+            }
+        }
+    }
 }
 
 /// I-9.5 两个方向，一版各判一次：① 队列里的每个号在这棵 inode 树里有记录且 nlink = 0；② nlink = 0 的每条记录，号在队列里。
diff --git a/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs b/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs
index bfab0bf..c400df7 100644
--- a/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs
+++ b/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs
@@ -32,8 +32,8 @@ use crate::common::{
     BuiltPool, FIXED_WRITE_TIME_SECONDS,
 };
 use crate::common_pointer_head_corruption::{
-    data_unit_bytes, node_bytes, replace_the_unit_and_propagate_its_checksum, the_slot_of,
-    tree_table_entry_of_kind, unit_bytes_at,
+    data_unit_bytes, newest_root, node_bytes, replace_the_unit_and_propagate_its_checksum,
+    the_slot_of, tree_table_entry_of_kind, unit_bytes_at,
 };
 
 /// 删除与建 inode 都在新池新建文件那个实例里发。
@@ -77,6 +77,24 @@ const ACCOUNTING_ROW_VALUE_OFFSET: usize = 22;
 /// 待删除但意图未完成占用（D5（快照 / 空间记账机制） 已定项 4 第 4 项），不带设备维的设备段。
 pub const STATISTIC_PENDING_DELETION_OCCUPANCY: u16 = 4;
 const STATISTIC_NO_DEVICE_DIMENSION: u32 = 0xFFFF_FFFF;
+/// 树表（码 2，key = 树 ID 8 字节）：条目 200 字节，树的种类在条目偏移 10、头 ID 在 116（D8（核心索引结构） 已定项 8 的字段表）。
+const TREE_TABLE_KEY_BYTES: usize = 8;
+const TREE_TABLE_ENTRY_BYTES: usize = 200;
+const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
+const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 116;
+/// 种类码登记表（D8（核心索引结构） 已定项 9）里没有的一个码（攻方 L1a 把 `deleted_inodes` 那一条的种类 9 改成它）。
+const TREE_KIND_NOT_REGISTERED: u16 = 13;
+/// 码 2 节点头里条目宽 2 字节在 84 + 2k、最大 key 在 52 + k（D18（块里携带什么信息） 已定项 7、18）。
+fn index_node_entry_width_offset(key_width: usize) -> usize {
+    84 + 2 * key_width
+}
+fn index_node_largest_key_offset(key_width: usize) -> usize {
+    INDEX_NODE_SMALLEST_KEY_OFFSET + key_width
+}
+/// 码 3 单元头里写序的实例代号（4 字节，偏移 93）：已发布谓词按它与诞生代号判（I-1.2）。
+const PACKED_UNIT_WRITE_ORDER_INSTANCE_OFFSET: usize = 93;
+/// 墓碑记录 40 字节，对象 ID 在 0（D18（块里携带什么信息） 已定项 24）。
+const TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET: usize = 0;
 
 pub fn batches_of_one_data_unit() -> UnlinkBatchSize {
     UnlinkBatchSize::CappedForTests(
@@ -162,11 +180,20 @@ pub fn image_after_the_first_batch_of_a_three_unit_file(tag: &str) -> MemoryPool
 /// 新池新建文件之后建 `count` 个空 inode（号从 2 起，size 0、没有数据单元，与第一个文件同在最左那片叶里）。
 pub fn pool_with_new_inodes(tag: &str, count: u64) -> BuiltPool {
     let mut pool = build_pool(tag);
+    publish_new_inodes_and_advance_the_pool(&mut pool, count);
+    pool
+}
+
+/// 在池上再发一次建 `count` 个空 inode（号从记账水位起），交回这一次发布。
+pub fn publish_new_inodes_and_advance_the_pool(
+    pool: &mut BuiltPool,
+    count: u64,
+) -> TransactionOutput {
     let publish_parameters = parameters();
     let previous = pool.output.clone();
     let devices = pool.devices.as_mut().expect("镜像还开着");
     let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
-    pool.output = publish_new_inodes(
+    let output = publish_new_inodes(
         &mut writer,
         &mut pool.allocator,
         &previous,
@@ -175,6 +202,20 @@ pub fn pool_with_new_inodes(tag: &str, count: u64) -> BuiltPool {
         INSTANCE_OF_THE_PUBLISHES,
     )
     .expect("建空 inode");
+    pool.output = output.clone();
+    output
+}
+
+/// 新池新建文件之后把第一个文件顺序写成空内容：长度 0 的顺序写照样写一个数据单元（`write_request_split` 的
+/// `data_unit_count_of_a_sequential_write`，长度 0 时是 1），inode 1 size 0、一个数据单元。
+pub fn pool_with_a_first_file_of_size_zero_and_one_data_unit(tag: &str) -> BuiltPool {
+    let mut pool = build_pool(tag);
+    let grown = sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &[]);
+    assert_eq!(
+        grown.data_pointers.len(),
+        1,
+        "长度 0 的顺序写写一个数据单元"
+    );
     pool
 }
 
@@ -320,7 +361,17 @@ pub fn change_the_inode_record_in_the_leaf_at(
 
 /// I-9.5 反方向（C708 那一形，也是判决里说的僵尸记录：nlink 0、单元全放、不在队列）：`inode` 的记录 nlink 1 → 0，队列不动。
 pub fn set_the_nlink_of_an_inode_record_to_zero(image: &mut MemoryPool, inode: u64) {
-    change_the_inode_record(image, inode, |record| {
+    let (leaf_slot, _) = inode_leaf_holding(image, inode);
+    set_the_nlink_of_an_inode_record_in_the_leaf_at_to_zero(image, leaf_slot, inode);
+}
+
+/// [`set_the_nlink_of_an_inode_record_to_zero`] 改的是 `leaf_slot` 那片叶（哪一版的都行）。
+pub fn set_the_nlink_of_an_inode_record_in_the_leaf_at_to_zero(
+    image: &mut MemoryPool,
+    leaf_slot: SlotNumber,
+    inode: u64,
+) {
+    change_the_inode_record_in_the_leaf_at(image, leaf_slot, inode, |record| {
         assert_eq!(
             get_u32(record, INODE_RECORD_NLINK_OFFSET),
             1,
@@ -330,6 +381,19 @@ pub fn set_the_nlink_of_an_inode_record_to_zero(image: &mut MemoryPool, inode: u
     });
 }
 
+/// I-9.5 正方向「队列里的号在树里 nlink ≠ 0」那一形（C708（树里 nlink = 0 的 inode 不在删除队列里，池级 checker 判不出） 还差的那一形）：
+/// 意图之后队列 {1} 不动，1 号那条记录的 nlink 0 → 1（size、单元都不动）。
+pub fn set_the_nlink_of_the_queued_first_file_back_to_one(image: &mut MemoryPool) {
+    change_the_inode_record(image, FIRST_INODE_NUMBER, |record| {
+        assert_eq!(
+            get_u32(record, INODE_RECORD_NLINK_OFFSET),
+            0,
+            "意图之后 1 号的 nlink 是 0"
+        );
+        set_u32(record, INODE_RECORD_NLINK_OFFSET, 1);
+    });
+}
+
 /// I-9.5 正方向「队列里的号没有记录」那一形（判决第二节最后一行：只红 I-9.5）：意图之后的 `deleted_inodes` 根 {1} 改成 {1, 7}
 /// （条目数 1 → 2、声明长度 8 → 16、头里最大 key 1 → 7），7 号在 inode 树里没有记录、extent 树里也没有单元。
 pub fn put_an_inode_number_without_a_record_into_the_deletion_queue(image: &mut MemoryPool) {
@@ -364,8 +428,20 @@ pub fn put_an_inode_number_without_a_record_into_the_deletion_queue(image: &mut
 /// 与空 inode 2 同在最左那片叶里，把 1 号那条记录从叶里拿掉（2 号挪到第 0 条、记录数 2 → 1、声明长度 280 → 140），extent 树不动：
 /// 1 号在 extent 树里还有数据单元、inode 树里没有它的记录、号也不在队列里。
 pub fn drop_the_record_of_the_first_file_and_keep_its_data_units(image: &mut MemoryPool) {
-    let (leaf_slot, index) = inode_leaf_holding(image, FIRST_INODE_NUMBER);
-    assert_eq!(index, 0, "1 号是叶里第 0 条");
+    let (leaf_slot, _) = inode_leaf_holding(image, FIRST_INODE_NUMBER);
+    drop_the_record_of_the_first_file_in_the_leaf_at_and_keep_its_data_units(image, leaf_slot);
+}
+
+/// [`drop_the_record_of_the_first_file_and_keep_its_data_units`] 改的是 `leaf_slot` 那片叶（哪一版的都行，叶里恰好 1 号与 2 号两条）。
+pub fn drop_the_record_of_the_first_file_in_the_leaf_at_and_keep_its_data_units(
+    image: &mut MemoryPool,
+    leaf_slot: SlotNumber,
+) {
+    assert_eq!(
+        index_of_the_inode_record_in_the_leaf_at(image, leaf_slot, FIRST_INODE_NUMBER),
+        Some(0),
+        "1 号是叶里第 0 条"
+    );
     change_the_unit_and_propagate_its_checksum(image, leaf_slot, data_unit_bytes(), |leaf| {
         assert_eq!(
             get_u16(leaf, PACKED_UNIT_RECORD_COUNT_OFFSET),
@@ -480,7 +556,36 @@ pub fn drop_the_pending_deletion_occupancy_row(image: &mut MemoryPool) {
 /// I-9.6 墓碑那一半：意图之后墓碑树第一片容器里那条墓碑的对象 ID 1 → 水位（记账里的 inode 号水位 2）：水位不再大于全部墓碑的对象 ID，
 /// inode 树里最大的 key 仍是 1。
 pub fn raise_the_tombstone_object_identifier_to_the_inode_number_watermark(image: &mut MemoryPool) {
-    const INODE_NUMBER_WATERMARK_AFTER_THE_FIRST_FILE: u64 = 2;
+    let container_slot = first_tombstone_container_slot_of_the_newest_root(image);
+    change_the_unit_and_propagate_its_checksum(
+        image,
+        container_slot,
+        data_unit_bytes(),
+        |container| {
+            assert_eq!(
+                get_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET),
+                1,
+                "意图之后墓碑容器里一条墓碑"
+            );
+            assert_eq!(
+                get_u64(container, PACKED_UNIT_RECORDS_START),
+                FIRST_INODE_NUMBER,
+                "那条墓碑的对象 ID 是 1"
+            );
+            set_u64(
+                container,
+                PACKED_UNIT_RECORDS_START,
+                INODE_NUMBER_WATERMARK_AFTER_THE_FIRST_FILE,
+            );
+        },
+    );
+}
+
+/// 意图之后记账里的 inode 号水位（第一个文件用掉 1 号）。
+const INODE_NUMBER_WATERMARK_AFTER_THE_FIRST_FILE: u64 = 2;
+
+/// 最新的根那一版墓碑树根第一条条目指的那片墓碑容器的槽（这几份镜像上墓碑树根是层级 1、孩子是容器）。
+pub fn first_tombstone_container_slot_of_the_newest_root(image: &MemoryPool) -> SlotNumber {
     let tombstone_root = tree_table_entry_of_kind(image, TREE_KIND_TOMBSTONE).root;
     let first_entry = parse_index_node(&unit_bytes_at(
         image,
@@ -492,31 +597,211 @@ pub fn raise_the_tombstone_object_identifier_to_the_inode_number_watermark(image
     .first()
     .expect("墓碑树根至少一条条目")
     .clone();
-    let container_slot = child_slot_of_a_container_tree_entry(&first_entry);
+    child_slot_of_a_container_tree_entry(&first_entry)
+}
+
+/// I-9.6 墓碑那一半「已发布」那一支（`walk.rs` 的 `largest_published_tombstone_object_identifier` 里按已发布谓词滤掉的容器）：
+/// 意图之后那片墓碑容器头里写序的实例代号 1 → 2（大于挂载根的实例 1，已发布谓词判它不是已发布，I-1.2 那一行状态列），
+/// 那条墓碑的对象 ID 也抬到水位 2。这片容器不算进墓碑那一半，I-9.6 照旧成立；容器被所选根引用而写序不是已发布，由别的条判红。
+pub fn move_the_tombstone_container_to_an_unpublished_instance_and_raise_its_object_identifier(
+    image: &mut MemoryPool,
+) {
+    let container_slot = first_tombstone_container_slot_of_the_newest_root(image);
     change_the_unit_and_propagate_its_checksum(
         image,
         container_slot,
         data_unit_bytes(),
         |container| {
             assert_eq!(
-                get_u16(container, PACKED_UNIT_RECORD_COUNT_OFFSET),
-                1,
-                "意图之后墓碑容器里一条墓碑"
+                get_u32(container, PACKED_UNIT_WRITE_ORDER_INSTANCE_OFFSET),
+                INSTANCE_OF_THE_PUBLISHES.0,
+                "墓碑容器是挂载根那个实例写的"
+            );
+            set_u32(
+                container,
+                PACKED_UNIT_WRITE_ORDER_INSTANCE_OFFSET,
+                INSTANCE_OF_THE_PUBLISHES.0 + 1,
             );
+            let object_identifier =
+                PACKED_UNIT_RECORDS_START + TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET;
             assert_eq!(
-                get_u64(container, PACKED_UNIT_RECORDS_START),
+                get_u64(container, object_identifier),
                 FIRST_INODE_NUMBER,
                 "那条墓碑的对象 ID 是 1"
             );
             set_u64(
                 container,
-                PACKED_UNIT_RECORDS_START,
+                object_identifier,
                 INODE_NUMBER_WATERMARK_AFTER_THE_FIRST_FILE,
             );
         },
     );
 }
 
+/// 一个单元两份全零（两盘同槽，不补引用链）：那个单元读不出。
+pub fn zero_both_copies_of_the_unit_at(
+    image: &mut MemoryPool,
+    slot: SlotNumber,
+    length_in_bytes: usize,
+) {
+    let zeroed = vec![0_u8; length_in_bytes];
+    for device in image.devices.values_mut() {
+        device.write(slot.to_device_offset(), &zeroed);
+    }
+}
+
+/// 最新的根那一版树表单元的槽。
+pub fn tree_table_slot_of_the_newest_root(image: &MemoryPool) -> SlotNumber {
+    the_slot_of(&newest_root(image).tree_table)
+}
+
+fn tree_table_entry_count(tree_table: &[u8]) -> usize {
+    usize::from(get_u16(
+        tree_table,
+        index_node_entry_count_offset(TREE_TABLE_KEY_BYTES),
+    ))
+}
+
+fn tree_table_entry_start(index: usize) -> usize {
+    index_node_entries_start(TREE_TABLE_KEY_BYTES) + index * TREE_TABLE_ENTRY_BYTES
+}
+
+/// 树表里种类是 `kind` 的那一条（恰好一条）的下标。
+fn index_of_the_only_tree_table_entry_of_kind(tree_table: &[u8], kind: u16) -> usize {
+    let indexes: Vec<usize> = (0..tree_table_entry_count(tree_table))
+        .filter(|index| {
+            get_u16(
+                tree_table,
+                tree_table_entry_start(*index) + TREE_TABLE_ENTRY_KIND_OFFSET,
+            ) == kind
+        })
+        .collect();
+    let [index] = indexes.as_slice() else {
+        panic!("树表里种类 {kind} 的条目恰好一条：{indexes:?}")
+    };
+    *index
+}
+
+/// L1a（代码轮判决 `research/prompts/m3-batch18-code-r1-main-verification.md` 第二节「K2 漏判 L1」，攻方报告第二节）：
+/// `tree_table_slot` 那张树表（哪一版的都行）里 `deleted_inodes` 那一条的种类码 9 改成没登记的 13，树 ID、头 ID、根指针都不动：
+/// 那一版树表里没有指着 inode 树的 `deleted_inodes` 条目。
+pub fn give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at(
+    image: &mut MemoryPool,
+    tree_table_slot: SlotNumber,
+) {
+    change_the_unit_and_propagate_its_checksum(
+        image,
+        tree_table_slot,
+        node_bytes(),
+        |tree_table| {
+            let index =
+                index_of_the_only_tree_table_entry_of_kind(tree_table, TREE_KIND_DELETED_INODES);
+            set_u16(
+                tree_table,
+                tree_table_entry_start(index) + TREE_TABLE_ENTRY_KIND_OFFSET,
+                TREE_KIND_NOT_REGISTERED,
+            );
+        },
+    );
+}
+
+/// L1b：`tree_table_slot` 那张树表里整条拿掉 `deleted_inodes` 那一条（后面的条目往前挪、末尾清零、条目数与声明长度各减一条；
+/// 那一条不在首末，头里的 key 区间不变）。
+pub fn remove_the_deleted_inodes_entry_from_the_tree_table_at(
+    image: &mut MemoryPool,
+    tree_table_slot: SlotNumber,
+) {
+    change_the_unit_and_propagate_its_checksum(
+        image,
+        tree_table_slot,
+        node_bytes(),
+        |tree_table| {
+            let count = tree_table_entry_count(tree_table);
+            let index =
+                index_of_the_only_tree_table_entry_of_kind(tree_table, TREE_KIND_DELETED_INODES);
+            assert!(
+                index > 0 && index + 1 < count,
+                "deleted_inodes 那一条不在首末：拿掉它头里的 key 区间不变"
+            );
+            let start = tree_table_entry_start(index);
+            let end = tree_table_entry_start(count);
+            tree_table.copy_within(start + TREE_TABLE_ENTRY_BYTES..end, start);
+            tree_table[end - TREE_TABLE_ENTRY_BYTES..end].fill(0);
+            set_u16(
+                tree_table,
+                index_node_entry_count_offset(TREE_TABLE_KEY_BYTES),
+                u16::try_from(count - 1).expect("条目数"),
+            );
+            let declared_length = get_u16(tree_table, UNIT_DECLARED_LENGTH_OFFSET);
+            set_u16(
+                tree_table,
+                UNIT_DECLARED_LENGTH_OFFSET,
+                declared_length - u16::try_from(TREE_TABLE_ENTRY_BYTES).expect("200"),
+            );
+        },
+    );
+}
+
+/// `tree_table_slot` 那张树表里种类是 `kind` 的那一条的头 ID（条目偏移 116）改成 `head_identifier`，别的字段不动。
+pub fn set_the_head_identifier_of_the_tree_table_entry_of_kind_at(
+    image: &mut MemoryPool,
+    tree_table_slot: SlotNumber,
+    kind: u16,
+    head_identifier: u64,
+) {
+    change_the_unit_and_propagate_its_checksum(
+        image,
+        tree_table_slot,
+        node_bytes(),
+        |tree_table| {
+            let index = index_of_the_only_tree_table_entry_of_kind(tree_table, kind);
+            set_u64(
+                tree_table,
+                tree_table_entry_start(index) + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
+                head_identifier,
+            );
+        },
+    );
+}
+
+/// `tree_table_slot` 那张树表的条目宽收窄到 key 宽（8）：条目数改 1、声明长度改 8、最大 key 贴到剩下那一条的 key 上、
+/// 条目区之后清零（照 `checker_known_bad_images.rs` 的 `narrow_the_entry_width_to_the_key_width`）。别的自描述字段自洽，
+/// 只是条目比字段表的 200 字节窄。
+pub fn narrow_the_entries_of_the_tree_table_at_to_the_key_width(
+    image: &mut MemoryPool,
+    tree_table_slot: SlotNumber,
+) {
+    change_the_unit_and_propagate_its_checksum(
+        image,
+        tree_table_slot,
+        node_bytes(),
+        |tree_table| {
+            let key_width = TREE_TABLE_KEY_BYTES;
+            let key_width_field = u16::try_from(key_width).expect("8");
+            let entries_start = index_node_entries_start(key_width);
+            let first_key = tree_table[entries_start..entries_start + key_width].to_vec();
+            set_u16(tree_table, UNIT_DECLARED_LENGTH_OFFSET, key_width_field);
+            set_u16(tree_table, index_node_entry_count_offset(key_width), 1);
+            set_u16(
+                tree_table,
+                index_node_entry_width_offset(key_width),
+                key_width_field,
+            );
+            let largest_key = index_node_largest_key_offset(key_width);
+            tree_table[largest_key..largest_key + key_width].copy_from_slice(&first_key);
+            tree_table[entries_start + key_width..].fill(0);
+        },
+    );
+}
+
+/// 最新的根那一版树表里种类是 `kind` 的那一条指的根（`deleted_inodes`、extent 树）的槽。
+pub fn root_slot_of_the_tree_of_kind_of_the_newest_root(
+    image: &MemoryPool,
+    kind: u16,
+) -> SlotNumber {
+    the_slot_of(&tree_table_entry_of_kind(image, kind).root)
+}
+
 /// 意图之后那一版上的坏镜像（`image_after_the_intent_of_a_three_unit_file`）。
 pub fn known_bad_images_after_the_intent_of_a_three_unit_file() -> Vec<(&'static str, Mutation)> {
     vec![
@@ -524,6 +809,10 @@ pub fn known_bad_images_after_the_intent_of_a_three_unit_file() -> Vec<(&'static
             DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
             Box::new(put_an_inode_number_without_a_record_into_the_deletion_queue),
         ),
+        (
+            DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+            Box::new(set_the_nlink_of_the_queued_first_file_back_to_one),
+        ),
         (
             "I-9.6",
             Box::new(raise_the_tombstone_object_identifier_to_the_inode_number_watermark),
diff --git a/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs b/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
index 190be19..37852bc 100644
--- a/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
+++ b/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
@@ -13,17 +13,28 @@ use common_deletion_in_progress::{
     change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at,
     drop_the_pending_deletion_occupancy_row,
     drop_the_record_of_the_first_file_and_keep_its_data_units,
+    drop_the_record_of_the_first_file_in_the_leaf_at_and_keep_its_data_units,
+    first_tombstone_container_slot_of_the_newest_root,
+    give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at,
     image_after_the_first_batch_of_a_three_unit_file, image_after_the_intent_of_a_three_unit_file,
-    pool_with_a_three_unit_file, pool_with_new_inodes, publish_the_intent,
+    move_the_tombstone_container_to_an_unpublished_instance_and_raise_its_object_identifier,
+    narrow_the_entries_of_the_tree_table_at_to_the_key_width,
+    pool_with_a_first_file_of_size_zero_and_one_data_unit, pool_with_a_three_unit_file,
+    pool_with_new_inodes, publish_new_inodes_and_advance_the_pool, publish_the_intent,
     publish_the_next_step_in_batches_of_one,
     put_an_inode_number_without_a_record_into_the_deletion_queue,
     raise_the_size_of_the_first_file_by_one_data_unit_payload,
     raise_the_size_of_the_first_file_in_the_leaf_at,
     raise_the_tombstone_object_identifier_to_the_inode_number_watermark,
-    set_the_nlink_of_an_inode_record_to_zero, set_the_pending_deletion_occupancy_row_to_one,
-    verdict_of, violated_invariants,
+    remove_the_deleted_inodes_entry_from_the_tree_table_at,
+    root_slot_of_the_tree_of_kind_of_the_newest_root,
+    set_the_head_identifier_of_the_tree_table_entry_of_kind_at,
+    set_the_nlink_of_an_inode_record_in_the_leaf_at_to_zero,
+    set_the_nlink_of_an_inode_record_to_zero, set_the_nlink_of_the_queued_first_file_back_to_one,
+    set_the_pending_deletion_occupancy_row_to_one, tree_table_slot_of_the_newest_root, verdict_of,
+    violated_invariants, zero_both_copies_of_the_unit_at,
 };
-use common_pointer_head_corruption::{data_unit_bytes, unit_bytes_at};
+use common_pointer_head_corruption::{data_unit_bytes, node_bytes, unit_bytes_at};
 use singlefs_checker::image::{
     InvariantVerdict, DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
     DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
@@ -31,6 +42,7 @@ use singlefs_checker::image::{
     INTENT_IS_NOT_APPLICABLE_IN_THE_FIRST_VERSION, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
 };
 use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
+use singlefs_core::records::{TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT};
 use singlefs_core::transaction::TransactionUnit;
 use singlefs_harness::memory_pool::MemoryPool;
 
@@ -46,6 +58,38 @@ fn first_violation_of(image: &MemoryPool, invariant: &str) -> String {
     }
 }
 
+/// 这一条报不适用时的理由（射程里判不了的那一版也报这个，`Judgements::part_of_the_range_not_judged`）；报了别的判定就当场失败。
+fn reason_not_judged(image: &MemoryPool, invariant: &str) -> &'static str {
+    match verdict_of(image, invariant) {
+        InvariantVerdict::NotApplicable(reason) => reason,
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
+            panic!("{invariant} 应当报射程里判不了（不适用带理由），实际 {other_verdict:?}")
+        }
+    }
+}
+
+/// 删除态扫描在一版上读 I-9.5、I-4.7、I-4.6 三条要的那一组（`walk.rs` 的 `DELETION_QUEUE_SCAN_INVARIANTS`）。
+const DELETION_QUEUE_SCAN_INVARIANTS: [&str; 3] = [
+    DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+];
+/// 各种判不了的理由里点得出是哪一形的那一段（`walk.rs` 的 `DeletionScanGap::reason`）。
+const NOT_JUDGED_INODE_TREE_NAMED_BY_NO_SINGLE_DELETED_INODES_ENTRY: &str =
+    "头 ID 指着它的 deleted_inodes 条目不是恰好一条";
+const NOT_JUDGED_DELETED_INODES_HEAD_IDENTIFIER_NAMES_NO_SINGLE_INODE_TREE: &str =
+    "deleted_inodes 条目偏移 116 的头 ID 配不上同一张树表里恰好一条 inode 树条目";
+const NOT_JUDGED_EXTENT_TREE_OF_THE_INODE_TREE_NOT_SINGLE: &str =
+    "头 ID 指着这棵 inode 树的 extent 树条目不是恰好一条";
+const NOT_JUDGED_EXTENT_TREE_UNREADABLE: &str = "extent 树读不全：那一版每个号的数据单元数判不了";
+const NOT_JUDGED_DELETED_INODES_ROOT_UNREADABLE: &str = "deleted_inodes 根读不出";
+const NOT_JUDGED_TREE_TABLE_UNREADABLE: &str = "树表读不出";
+const NOT_JUDGED_TREE_TABLE_ENTRY_SHORTER_THAN_THE_FIELD_TABLE: &str =
+    "树表条目宽不到字段表的 200 字节";
+const NOT_JUDGED_TOMBSTONE_TREE_UNREADABLE: &str = "所选根的树表或墓碑树读不全";
+
 /// 干净的那一版上一条违例都没有：坏镜像「只红自己」从这里比起。
 fn assert_the_base_image_is_clean(image: &MemoryPool, base: &str) {
     assert_eq!(
@@ -375,3 +419,453 @@ fn every_published_step_of_unlinking_a_three_unit_file_holds_the_deletion_invari
         );
     }
 }
+
+/// L1a（代码轮判决 `research/prompts/m3-batch18-code-r1-main-verification.md` 第二节「K2 漏判 L1」，攻方报告第二节）：意图之后那一版树表里
+/// `deleted_inodes` 那一条的种类码 9 改成没登记的 13——那一版的 inode 树（1 号 nlink 0、三个单元）没有任何 `deleted_inodes` 条目指着它。
+/// 改法 A 之前只从 `deleted_inodes` 条目一侧去配，这一版一次都不判、也不记判不了，I-9.5 报成立；现在三条在那一版判不了、不报成立。
+/// 判红的只有 I-9.16（树表里十一种各恰好一条，④）。
+#[test]
+fn an_inode_tree_whose_deleted_inodes_entry_carries_an_unregistered_kind_leaves_the_deletion_queue_invariants_not_judged(
+) {
+    let clean =
+        image_after_the_intent_of_a_three_unit_file("deletion-checker-l1a-unregistered-kind");
+    assert_the_base_image_is_clean(&clean, "意图之后");
+    let mut image = clean.clone();
+    let tree_table_slot = tree_table_slot_of_the_newest_root(&image);
+    give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at(
+        &mut image,
+        tree_table_slot,
+    );
+    assert_eq!(
+        violated_invariants(&image),
+        vec!["I-9.16"],
+        "判红的只有树表种类那一条"
+    );
+    for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
+        let reason = reason_not_judged(&image, invariant);
+        assert!(
+            reason.contains(NOT_JUDGED_INODE_TREE_NAMED_BY_NO_SINGLE_DELETED_INODES_ENTRY),
+            "{invariant} 在没有 deleted_inodes 条目指着 inode 树的那一版判不了：{reason}"
+        );
+    }
+}
+
+/// L1b：同上，`deleted_inodes` 那一条整条从意图之后那一版的树表里拿掉（树表 10 条）。
+#[test]
+fn an_inode_tree_without_a_deleted_inodes_entry_in_the_tree_table_leaves_the_deletion_queue_invariants_not_judged(
+) {
+    let clean = image_after_the_intent_of_a_three_unit_file(
+        "deletion-checker-l1b-deleted-inodes-entry-removed",
+    );
+    assert_the_base_image_is_clean(&clean, "意图之后");
+    let mut image = clean.clone();
+    let tree_table_slot = tree_table_slot_of_the_newest_root(&image);
+    remove_the_deleted_inodes_entry_from_the_tree_table_at(&mut image, tree_table_slot);
+    assert_eq!(
+        violated_invariants(&image),
+        vec!["I-9.16"],
+        "判红的只有树表种类那一条"
+    );
+    for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
+        let reason = reason_not_judged(&image, invariant);
+        assert!(
+            reason.contains(NOT_JUDGED_INODE_TREE_NAMED_BY_NO_SINGLE_DELETED_INODES_ENTRY),
+            "{invariant} 在没有 deleted_inodes 条目指着 inode 树的那一版判不了：{reason}"
+        );
+    }
+}
+
+/// L1c：三个单元的文件删到「意图、留 2、留 1」，只把中间那一版（留 2）树表里 `deleted_inodes` 那一条的种类码改成 13，最新的根写对。
+/// 中间那一版 1 号 nlink 0、还有两个单元、不在任何队列里；改法 A 之前 I-9.5、I-4.7、I-4.6 都由别的版本判成立（攻方量过）。
+#[test]
+fn an_intermediate_version_without_a_deleted_inodes_entry_leaves_the_deletion_queue_invariants_not_judged_while_the_newest_root_is_clean(
+) {
+    let mut pool =
+        pool_with_a_three_unit_file("deletion-checker-l1c-intermediate-unregistered-kind");
+    publish_the_intent(&mut pool);
+    let intermediate = publish_the_next_step_in_batches_of_one(&mut pool);
+    let newest = publish_the_next_step_in_batches_of_one(&mut pool);
+    let intermediate_tree_table_slot = intermediate.unit(TransactionUnit::TreeTable).slot;
+    assert_ne!(
+        intermediate_tree_table_slot,
+        newest.unit(TransactionUnit::TreeTable).slot,
+        "每一版重写树表：中间那一版的树表只有它自己引用"
+    );
+    let clean = pool.memory_pool();
+    assert_the_base_image_is_clean(&clean, "删到留 1");
+    let mut image = clean.clone();
+    give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at(
+        &mut image,
+        intermediate_tree_table_slot,
+    );
+    assert_eq!(
+        violated_invariants(&image),
+        vec!["I-9.16"],
+        "判红的只有树表种类那一条"
+    );
+    for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
+        let reason = reason_not_judged(&image, invariant);
+        assert!(
+            reason.contains(NOT_JUDGED_INODE_TREE_NAMED_BY_NO_SINGLE_DELETED_INODES_ENTRY),
+            "{invariant} 在中间那一版判不了、不由别的版本判成立：{reason}"
+        );
+    }
+}
+
+/// L2（攻方报告第二节）：L1a 之上把 1 号的 size 多加一个净荷容量（要 4 个单元、有 3 个）。改法 A 之前 I-4.6 报不适用，理由
+/// 「走读到的每一版 inode 树里都没有 nlink = 0 的记录」与镜像不符（那一版 1 号 nlink 0）；现在那一版判不了，理由写的是配不上。
+#[test]
+fn a_size_ahead_of_the_data_units_on_a_version_without_a_deleted_inodes_entry_is_not_judged_with_the_reason_of_the_missing_entry(
+) {
+    let clean = image_after_the_intent_of_a_three_unit_file("deletion-checker-l2-size-ahead");
+    assert_the_base_image_is_clean(&clean, "意图之后");
+    let mut image = clean.clone();
+    raise_the_size_of_the_first_file_by_one_data_unit_payload(&mut image);
+    let tree_table_slot = tree_table_slot_of_the_newest_root(&image);
+    give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at(
+        &mut image,
+        tree_table_slot,
+    );
+    let reason = reason_not_judged(
+        &image,
+        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+    );
+    assert!(
+        reason.contains(NOT_JUDGED_INODE_TREE_NAMED_BY_NO_SINGLE_DELETED_INODES_ENTRY),
+        "I-4.6 在那一版判不了，理由写配不上：{reason}"
+    );
+    assert!(
+        !reason.contains("没有 nlink = 0 的记录"),
+        "理由不写镜像里不成立的事（那一版 1 号 nlink = 0）：{reason}"
+    );
+}
+
+/// `DeletionScanGap::DeletedInodesHeadIdentifierNamesNoSingleInodeTree`（攻方 L1d 那一形）：意图之后那一版树表里 `deleted_inodes` 那一条的
+/// 头 ID 从 inode 树的树 ID 改成 0，配不上任何一条 inode 树条目。三条在那一版判不了，理由先记头 ID 配不上那一句；I-9.27 判红。
+#[test]
+fn a_deleted_inodes_entry_whose_head_identifier_names_no_inode_tree_leaves_the_deletion_queue_invariants_not_judged(
+) {
+    let clean =
+        image_after_the_intent_of_a_three_unit_file("deletion-checker-deleted-inodes-head-zero");
+    assert_the_base_image_is_clean(&clean, "意图之后");
+    let mut image = clean.clone();
+    let tree_table_slot = tree_table_slot_of_the_newest_root(&image);
+    set_the_head_identifier_of_the_tree_table_entry_of_kind_at(
+        &mut image,
+        tree_table_slot,
+        TREE_KIND_DELETED_INODES,
+        0,
+    );
+    assert_eq!(
+        violated_invariants(&image),
+        vec!["I-9.27"],
+        "判红的只有头 ID 取值那一条"
+    );
+    for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
+        let reason = reason_not_judged(&image, invariant);
+        assert!(
+            reason.contains(NOT_JUDGED_DELETED_INODES_HEAD_IDENTIFIER_NAMES_NO_SINGLE_INODE_TREE),
+            "{invariant} 在头 ID 配不上的那一版判不了：{reason}"
+        );
+    }
+}
+
+/// `DeletionScanGap::ExtentTreeOfTheInodeTreeNotSingle`：意图之后那一版树表里 extent 树那一条的头 ID 从 inode 树的树 ID 改成 0。
+/// I-9.5 不读 extent 树，照判、成立；I-4.7、I-4.6 在那一版判不了。I-9.27 判红。
+#[test]
+fn an_extent_tree_entry_whose_head_identifier_names_no_inode_tree_leaves_the_data_unit_invariants_not_judged(
+) {
+    let clean = image_after_the_intent_of_a_three_unit_file("deletion-checker-extent-head-zero");
+    assert_the_base_image_is_clean(&clean, "意图之后");
+    let mut image = clean.clone();
+    let tree_table_slot = tree_table_slot_of_the_newest_root(&image);
+    set_the_head_identifier_of_the_tree_table_entry_of_kind_at(
+        &mut image,
+        tree_table_slot,
+        TREE_KIND_EXTENT,
+        0,
+    );
+    assert_eq!(
+        violated_invariants(&image),
+        vec!["I-9.27"],
+        "判红的只有头 ID 取值那一条"
+    );
+    assert_eq!(
+        verdict_of(
+            &image,
+            DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES
+        ),
+        InvariantVerdict::Holds,
+        "I-9.5 不读 extent 树，照判、成立"
+    );
+    for invariant in [
+        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+    ] {
+        let reason = reason_not_judged(&image, invariant);
+        assert!(
+            reason.contains(NOT_JUDGED_EXTENT_TREE_OF_THE_INODE_TREE_NOT_SINGLE),
+            "{invariant} 在 extent 树配不上的那一版判不了：{reason}"
+        );
+    }
+}
+
+/// `DeletionScanGap::ExtentTreeUnreadable`：意图之后那一版 extent 树根两份全零（不补引用链）。I-9.5 照判、成立；I-4.7、I-4.6 判不了。
+#[test]
+fn an_extent_tree_that_does_not_read_back_leaves_the_data_unit_invariants_of_the_deletion_not_judged(
+) {
+    let mut image =
+        image_after_the_intent_of_a_three_unit_file("deletion-checker-extent-root-zeroed");
+    let extent_root_slot =
+        root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_EXTENT);
+    zero_both_copies_of_the_unit_at(&mut image, extent_root_slot, node_bytes());
+    assert_eq!(
+        verdict_of(
+            &image,
+            DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES
+        ),
+        InvariantVerdict::Holds,
+        "I-9.5 不读 extent 树，照判、成立"
+    );
+    for invariant in [
+        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+    ] {
+        let reason = reason_not_judged(&image, invariant);
+        assert!(
+            reason.contains(NOT_JUDGED_EXTENT_TREE_UNREADABLE),
+            "{invariant} 在 extent 树读不全的那一版判不了、不判红：{reason}"
+        );
+    }
+}
+
+/// `DeletionScanGap::DeletedInodesRootUnreadable`：意图之后那一版 `deleted_inodes` 根（队列 {1}）两份全零（不补引用链）。三条判不了、不判红。
+#[test]
+fn a_deleted_inodes_root_that_does_not_read_back_leaves_the_deletion_queue_invariants_not_judged() {
+    let mut image =
+        image_after_the_intent_of_a_three_unit_file("deletion-checker-deleted-inodes-root-zeroed");
+    let deleted_inodes_root_slot =
+        root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_DELETED_INODES);
+    zero_both_copies_of_the_unit_at(&mut image, deleted_inodes_root_slot, node_bytes());
+    for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
+        let reason = reason_not_judged(&image, invariant);
+        assert!(
+            reason.contains(NOT_JUDGED_DELETED_INODES_ROOT_UNREADABLE),
+            "{invariant} 在队列读不出的那一版判不了、不判红：{reason}"
+        );
+    }
+}
+
+/// `DeletionScanGap::TreeTableUnreadable`：三个单元的文件删到「意图、留 2、留 1」，中间那一版（留 2）的树表两份全零（不补引用链）。
+/// 那一版的删除态四条（I-9.5、I-4.7、I-4.6、I-3.4）判不了、不判红；别的版本照判。
+#[test]
+fn an_intermediate_tree_table_that_does_not_read_back_leaves_the_deletion_invariants_not_judged() {
+    let mut pool = pool_with_a_three_unit_file("deletion-checker-intermediate-tree-table-zeroed");
+    publish_the_intent(&mut pool);
+    let intermediate = publish_the_next_step_in_batches_of_one(&mut pool);
+    publish_the_next_step_in_batches_of_one(&mut pool);
+    let mut image = pool.memory_pool();
+    zero_both_copies_of_the_unit_at(
+        &mut image,
+        intermediate.unit(TransactionUnit::TreeTable).slot,
+        node_bytes(),
+    );
+    for invariant in DELETION_QUEUE_SCAN_INVARIANTS
+        .into_iter()
+        .chain([PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO])
+    {
+        let reason = reason_not_judged(&image, invariant);
+        assert!(
+            reason.contains(NOT_JUDGED_TREE_TABLE_UNREADABLE),
+            "{invariant} 在树表读不出的那一版判不了、不判红：{reason}"
+        );
+    }
+}
+
+/// `DeletionScanGap::TreeTableEntryShorterThanTheFieldTable`：同上那三步，中间那一版（留 2）的树表条目宽收窄到 key 宽 8（只留第一条、
+/// 别的自描述字段自洽，重封、补引用链）。那一版的删除态四条判不了、不判红。
+#[test]
+fn an_intermediate_tree_table_whose_entries_are_narrower_than_the_field_table_leaves_the_deletion_invariants_not_judged(
+) {
+    let mut pool = pool_with_a_three_unit_file("deletion-checker-intermediate-tree-table-narrow");
+    publish_the_intent(&mut pool);
+    let intermediate = publish_the_next_step_in_batches_of_one(&mut pool);
+    publish_the_next_step_in_batches_of_one(&mut pool);
+    let mut image = pool.memory_pool();
+    narrow_the_entries_of_the_tree_table_at_to_the_key_width(
+        &mut image,
+        intermediate.unit(TransactionUnit::TreeTable).slot,
+    );
+    for invariant in DELETION_QUEUE_SCAN_INVARIANTS
+        .into_iter()
+        .chain([PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO])
+    {
+        let reason = reason_not_judged(&image, invariant);
+        assert!(
+            reason.contains(NOT_JUDGED_TREE_TABLE_ENTRY_SHORTER_THAN_THE_FIELD_TABLE),
+            "{invariant} 在树表条目窄的那一版判不了、不判红：{reason}"
+        );
+    }
+}
+
+/// `DeletionScanGap::TombstoneTreeUnreadable`：意图之后所选根墓碑树里那片墓碑容器两份全零（不补引用链）。I-9.6 前一半（水位 > 树内最大 key）
+/// 照判、成立，墓碑那一半判不了：整条报不适用带墓碑树读不全的理由，不报成立。
+#[test]
+fn a_tombstone_container_that_does_not_read_back_leaves_the_watermark_invariant_not_judged() {
+    let mut image =
+        image_after_the_intent_of_a_three_unit_file("deletion-checker-tombstone-container-zeroed");
+    let container_slot = first_tombstone_container_slot_of_the_newest_root(&image);
+    zero_both_copies_of_the_unit_at(&mut image, container_slot, data_unit_bytes());
+    let reason = reason_not_judged(&image, "I-9.6");
+    assert!(
+        reason.contains(NOT_JUDGED_TOMBSTONE_TREE_UNREADABLE),
+        "I-9.6 墓碑那一半判不了、不报成立：{reason}"
+    );
+}
+
+/// I-9.6 墓碑那一半「已发布」那一支（`.claude/kb/invariants.md` I-9.6 那一行：只算已发布的墓碑）：意图之后那片墓碑容器头里写序的实例代号
+/// 1 → 2（大于挂载根的实例，已发布谓词判它不是已发布），墓碑对象 ID 也抬到水位 2。这片容器不算进墓碑那一半，I-9.6 照旧成立
+/// （对照：只抬对象 ID、不改实例的那一份红 I-9.6，见 `a_tombstone_whose_object_identifier_reaches_the_inode_number_watermark_reddens_only_the_watermark_invariant`）。
+#[test]
+fn a_tombstone_container_outside_the_published_predicate_does_not_count_toward_the_watermark() {
+    let clean =
+        image_after_the_intent_of_a_three_unit_file("deletion-checker-tombstone-not-published");
+    assert_the_base_image_is_clean(&clean, "意图之后");
+    let mut image = clean.clone();
+    move_the_tombstone_container_to_an_unpublished_instance_and_raise_its_object_identifier(
+        &mut image,
+    );
+    assert_eq!(
+        verdict_of(&image, "I-9.6"),
+        InvariantVerdict::Holds,
+        "没发布的墓碑容器不算进墓碑那一半：I-9.6 成立"
+    );
+}
+
+/// I-4.6 size = 0 那一支（`.claude/kb/invariants.md` I-4.6 那一行：size 为 0 时 0 或 1 个单元）：第一个文件顺序写成空内容（size 0、一个数据单元，
+/// 长度 0 的顺序写照样写一个单元），再发 unlink 的意图——1 号 nlink 0、size 0、一个单元，是发布路径自己写出来的合法中间态。
+#[test]
+fn an_inode_being_deleted_with_size_zero_and_one_data_unit_holds_the_intermediate_state_invariant()
+{
+    let mut pool = pool_with_a_first_file_of_size_zero_and_one_data_unit(
+        "deletion-checker-size-zero-one-unit",
+    );
+    publish_the_intent(&mut pool);
+    let image = pool.memory_pool();
+    assert_eq!(
+        violated_invariants(&image),
+        Vec::<&str>::new(),
+        "size 0、一个单元的文件发完意图之后没有违例"
+    );
+    for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
+        assert_eq!(
+            verdict_of(&image, invariant),
+            InvariantVerdict::Holds,
+            "{invariant} 真被评估过且成立"
+        );
+    }
+}
+
+/// 判决第二节 F4-a：判回退候选集里每一版，不只判最新根；判的是 I-9.5。第一个文件之后两次各建一个空 inode（2 号、3 号），
+/// 只把中间那一版（建 2 号那次）inode 叶里 2 号的 nlink 改成 0，最新的根那一版是对的。
+#[test]
+fn an_intermediate_version_with_an_inode_record_of_nlink_zero_outside_the_queue_reddens_while_the_newest_root_is_clean(
+) {
+    let mut pool = build_pool("deletion-checker-intermediate-zombie-record");
+    let intermediate = publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+    let newest = publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+    let leftmost_leaf =
+        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST);
+    let intermediate_leaf_slot = intermediate.unit(leftmost_leaf).slot;
+    assert_ne!(
+        intermediate_leaf_slot,
+        newest.unit(leftmost_leaf).slot,
+        "每次建 inode 重写 inode 叶：中间那一版的叶只有它自己引用"
+    );
+    let clean = pool.memory_pool();
+    assert_the_base_image_is_clean(&clean, "建了两次空 inode");
+    let mut image = clean.clone();
+    set_the_nlink_of_an_inode_record_in_the_leaf_at_to_zero(&mut image, intermediate_leaf_slot, 2);
+    assert_eq!(
+        violated_invariants(&image),
+        vec![DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES],
+        "中间那一版写坏、最新根写对：只红 I-9.5"
+    );
+    let detail = first_violation_of(
+        &image,
+        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+    );
+    assert!(
+        detail.contains(&format!("txg {}）", intermediate.root.checkpoint_txg.0))
+            && detail.contains("inode 2 的记录 nlink = 0"),
+        "违例说明点名中间那一版与那条记录：{detail}"
+    );
+}
+
+/// 同上，判的是 I-4.7：只把中间那一版（建 2 号那次）inode 叶里 1 号的记录拿掉、extent 树不动，最新的根那一版是对的。
+#[test]
+fn an_intermediate_version_with_data_units_of_an_inode_without_a_record_reddens_while_the_newest_root_is_clean(
+) {
+    let mut pool = build_pool("deletion-checker-intermediate-orphan-data-units");
+    let intermediate = publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+    let newest = publish_new_inodes_and_advance_the_pool(&mut pool, 1);
+    let leftmost_leaf =
+        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST);
+    let intermediate_leaf_slot = intermediate.unit(leftmost_leaf).slot;
+    assert_ne!(
+        intermediate_leaf_slot,
+        newest.unit(leftmost_leaf).slot,
+        "每次建 inode 重写 inode 叶：中间那一版的叶只有它自己引用"
+    );
+    let clean = pool.memory_pool();
+    assert_the_base_image_is_clean(&clean, "建了两次空 inode");
+    let mut image = clean.clone();
+    drop_the_record_of_the_first_file_in_the_leaf_at_and_keep_its_data_units(
+        &mut image,
+        intermediate_leaf_slot,
+    );
+    assert_eq!(
+        violated_invariants(&image),
+        vec![INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT],
+        "中间那一版写坏、最新根写对：只红 I-4.7"
+    );
+    let detail = first_violation_of(
+        &image,
+        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+    );
+    assert!(
+        detail.contains(&format!("txg {}）", intermediate.root.checkpoint_txg.0))
+            && detail.contains("inode 1 处于删除中（inode 树里没有它的记录）"),
+        "违例说明点名中间那一版与那个号：{detail}"
+    );
+}
+
+/// C708（树里 nlink = 0 的 inode 不在删除队列里，池级 checker 判不出） 还差的那一形：I-9.5 正方向「队列里的号在树里 nlink ≠ 0」。
+/// 意图之后队列 {1} 不动，1 号那条记录的 nlink 0 → 1。这一形必与 I-9.18（删除队列里的号有谱系上的墓碑）、I-9.19（谱系上的墓碑不指活对象）
+/// 同红（设计轮判决第二节最后一行），那两条这一版 checker 还没实现（不在 `IMPLEMENTED_INVARIANTS` 里），不在判红集合里。
+#[test]
+fn a_queue_number_whose_inode_record_has_nlink_one_reddens_the_deletion_queue_invariant() {
+    let clean =
+        image_after_the_intent_of_a_three_unit_file("deletion-checker-queued-record-of-nlink-one");
+    assert_the_base_image_is_clean(&clean, "意图之后");
+    let mut image = clean.clone();
+    set_the_nlink_of_the_queued_first_file_back_to_one(&mut image);
+    let violated = violated_invariants(&image);
+    assert!(
+        violated.contains(&DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES),
+        "I-9.5 在判红集合里：{violated:?}"
+    );
+    assert_eq!(
+        violated,
+        vec![DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES],
+        "这一版 checker 上判红的只有 I-9.5"
+    );
+    let detail = first_violation_of(
+        &image,
+        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
+    );
+    assert!(
+        detail.contains("deleted_inodes 里的号 1") && detail.contains("nlink = 1，不是 0"),
+        "违例说明点名队列里那个号与它的 nlink：{detail}"
+    );
+}
````

### a：mutations-append.tsv

````
第20批 L1 改法 A 反方向不配：没有 deleted_inodes 条目指着的 inode 树一次都不判（L1a 种类码 13 报成立）	crates/singlefs-checker/src/walk.rs	        if deleted_inodes_entries_naming_the_inode_tree != 1 {	        if false && deleted_inodes_entries_naming_the_inode_tree != 1 {	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_inode_tree_whose_deleted_inodes_entry_carries_an_unregistered_kind_leaves_the_deletion_queue_invariants_not_judged
第20批 L1 改法 A 反方向不配：deleted_inodes 条目整条拿掉的那一版报成立（L1b）	crates/singlefs-checker/src/walk.rs	        if deleted_inodes_entries_naming_the_inode_tree != 1 {	        if false && deleted_inodes_entries_naming_the_inode_tree != 1 {	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_inode_tree_without_a_deleted_inodes_entry_in_the_tree_table_leaves_the_deletion_queue_invariants_not_judged
第20批 L1 改法 A 反方向不配：只改中间版时三条由别的版本判成立（L1c）	crates/singlefs-checker/src/walk.rs	        if deleted_inodes_entries_naming_the_inode_tree != 1 {	        if false && deleted_inodes_entries_naming_the_inode_tree != 1 {	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_intermediate_version_without_a_deleted_inodes_entry_leaves_the_deletion_queue_invariants_not_judged_while_the_newest_root_is_clean
第20批 L2 改法 A 反方向不配：I-4.6 不适用的理由写成「每一版都没有 nlink = 0 的记录」	crates/singlefs-checker/src/walk.rs	        if deleted_inodes_entries_naming_the_inode_tree != 1 {	        if false && deleted_inodes_entries_naming_the_inode_tree != 1 {	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_size_ahead_of_the_data_units_on_a_version_without_a_deleted_inodes_entry_is_not_judged_with_the_reason_of_the_missing_entry
第20批 C708 I-9.5 正方向不核 nlink：队列里的号在树里 nlink = 1 放过	crates/singlefs-checker/src/walk.rs	.find(|inode| records.get(inode).is_none_or(|record| record.nlink != 0));	.find(|inode| records.get(inode).is_none());	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_queue_number_whose_inode_record_has_nlink_one_reddens_the_deletion_queue_invariant
第20批 I-4.6 size = 0 那一支只认 0 个单元（size 0、一个单元的合法中间态误红）	crates/singlefs-checker/src/walk.rs	            data_units <= 1	            data_units == 0	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_inode_being_deleted_with_size_zero_and_one_data_unit_holds_the_intermediate_state_invariant
第20批 I-9.6 墓碑那一半不按已发布谓词滤容器（没发布的墓碑容器算进水位）	crates/singlefs-checker/src/walk.rs	                    if !is_published {\n                        continue;\n                    }	                    if !is_published && is_published {\n                        continue;\n                    }	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_tombstone_container_outside_the_published_predicate_does_not_count_toward_the_watermark
第20批 删除态扫描只扫最新根（中间那一版 nlink = 0 的记录不在队列里漏判，I-9.5）	crates/singlefs-checker/src/walk.rs	        candidate_indexes\n            .iter()\n            .map(|index| {\n                let root = &roots[*index].2;\n                VersionToScanTheDeletionStateOf {	        [newest_index]\n            .iter()\n            .map(|index| {\n                let root = &roots[*index].2;\n                VersionToScanTheDeletionStateOf {	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_intermediate_version_with_an_inode_record_of_nlink_zero_outside_the_queue_reddens_while_the_newest_root_is_clean
第20批 删除态扫描只扫最新根（中间那一版没有记录的号还有单元漏判，I-4.7）	crates/singlefs-checker/src/walk.rs	        candidate_indexes\n            .iter()\n            .map(|index| {\n                let root = &roots[*index].2;\n                VersionToScanTheDeletionStateOf {	        [newest_index]\n            .iter()\n            .map(|index| {\n                let root = &roots[*index].2;\n                VersionToScanTheDeletionStateOf {	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_intermediate_version_with_data_units_of_an_inode_without_a_record_reddens_while_the_newest_root_is_clean
第20批 射程里判不了一律不记（读不全、配不上的那一版静默成立）	crates/singlefs-checker/src/image.rs	        self.part_of_the_range_not_judged\n            .entry(invariant)\n            .or_insert(reason);	        let _ = (invariant, reason);	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_intermediate_tree_table_that_does_not_read_back_leaves_the_deletion_invariants_not_judged
第20批 删除态扫描树表读不出的那一版静默跳过（TreeTableUnreadable 不记）	crates/singlefs-checker/src/walk.rs	            None => Some(DeletionScanGap::TreeTableUnreadable),	            None => None,	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_intermediate_tree_table_that_does_not_read_back_leaves_the_deletion_invariants_not_judged
第20批 删除态扫描树表条目窄的那一版静默跳过（TreeTableEntryShorterThanTheFieldTable 不记）	crates/singlefs-checker/src/walk.rs	                Some(DeletionScanGap::TreeTableEntryShorterThanTheFieldTable)	                None	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_intermediate_tree_table_whose_entries_are_narrower_than_the_field_table_leaves_the_deletion_invariants_not_judged
第20批 deleted_inodes 头 ID 配不上 inode 树时按空队列与零条记录判（DeletedInodesHeadIdentifierNamesNoSingleInodeTree 不记）	crates/singlefs-checker/src/walk.rs	            [] | [_, _, ..] => {\n                Err(DeletionScanGap::DeletedInodesHeadIdentifierNamesNoSingleInodeTree)\n            }	            [] | [_, _, ..] => Ok((BTreeSet::new(), BTreeMap::new())),	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_deleted_inodes_entry_whose_head_identifier_names_no_inode_tree_leaves_the_deletion_queue_invariants_not_judged
第20批 deleted_inodes 根读不出时按空队列判（DeletedInodesRootUnreadable 不记，I-9.5 反方向误红）	crates/singlefs-checker/src/walk.rs	    let root = read_index_node_without_judging(reader, root_pointer, cache)\n        .ok_or(DeletionScanGap::DeletedInodesRootUnreadable)?;	    let Some(root) = read_index_node_without_judging(reader, root_pointer, cache) else {\n        return Ok(BTreeSet::new());\n    };	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_deleted_inodes_root_that_does_not_read_back_leaves_the_deletion_queue_invariants_not_judged
第20批 extent 树配不上 inode 树时按零个单元判（ExtentTreeOfTheInodeTreeNotSingle 不记，I-4.6 误红）	crates/singlefs-checker/src/walk.rs	            [] | [_, _, ..] => Err(DeletionScanGap::ExtentTreeOfTheInodeTreeNotSingle),	            [] | [_, _, ..] => Ok(BTreeMap::new()),	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_extent_tree_entry_whose_head_identifier_names_no_inode_tree_leaves_the_data_unit_invariants_not_judged
第20批 extent 树读不全时按零个单元判（ExtentTreeUnreadable 不记，I-4.6 误红）	crates/singlefs-checker/src/walk.rs	    .ok_or(DeletionScanGap::ExtentTreeUnreadable)?;	    .ok_or(DeletionScanGap::ExtentTreeUnreadable);\n    let Ok(extent_tree) = extent_tree else {\n        return Ok(data_units_of_each_inode);\n    };	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_extent_tree_that_does_not_read_back_leaves_the_data_unit_invariants_of_the_deletion_not_judged
第20批 I-9.6 墓碑树读不全不记判不了（墓碑那一半静默成立）	crates/singlefs-checker/src/walk.rs	            Err(gap) => judgements.part_of_the_range_not_judged("I-9.6", gap.reason()),	            Err(_gap) => {}	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	a_tombstone_container_that_does_not_read_back_leaves_the_watermark_invariant_not_judged
````

## 实现员 g：crates.patch（/tmp/claude-1000/m3-batch20/g/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -131,6 +131,9 @@
 const CONTAINER_TREE_ENTRY_BYTES: u32 = 120u;
 const CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET: u32 = 34u;
 const FIRST_INODE_NUMBER: u32 = 1u;
+// inode 记录里 nlink 的偏移（4 字节，D8（核心索引结构） 已定项 6 字段表）与删除中的 inode 的 nlink（records.rs 的 NLINK_OF_AN_INODE_BEING_DELETED）
+const INODE_RECORD_NLINK_OFFSET: u32 = 36u;
+const NLINK_OF_AN_INODE_BEING_DELETED: u32 = 0u;
 const MAGIC_SFSB: u32 = 0x42534653u;
 const MAGIC_SFSR: u32 = 0x52534653u;
 const MAGIC_SFSU: u32 = 0x55534653u;
@@ -3167,6 +3170,7 @@
         return walk_failed(45u);
     }
     var found_inode = false;
+    var inode_nlink = 0u;
     var inode_size = vec2<u32>(0u, 0u);
     var inode_object_birth = vec2<u32>(0u, 0u);
     for (var e = 0u; e < inode_root.entry_count; e = e + 1u) {
@@ -3214,6 +3218,7 @@
             if (eq64(inode, u64_of(FIRST_INODE_NUMBER))) {
                 found_inode = true;
                 inode_object_birth = rd64(leaf.base, ro + 8u);
+                inode_nlink = rd32(leaf.base, ro + INODE_RECORD_NLINK_OFFSET);
                 inode_size = rd64(leaf.base, ro + 40u);
             }
         }
@@ -3221,6 +3226,11 @@
     if (!found_inode) {
         return walk_no_file();
     }
+    // 删除中的 inode（nlink = 0）：从意图那次发布起冷走读答「没有文件」，与 recovery.rs 冷走读同一判、同一处（取到第一个文件那条记录之后、
+    // 核单元数之前；D8（核心索引结构） 已定项 16 第 4 条）
+    if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {
+        return walk_no_file();
+    }
     // 数据单元数 = max(1, ⌈size ÷ 32634⌉)
     let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);
     var expected_units = divmod64(add64_saturating(inode_size, sub64(payload, u64_of(1u))), payload).quotient;
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -9,8 +9,9 @@
 //! I-9.28 比到候选集之外的根与树表读不出的根、I-7.8 扫描方向的认领规则、I-7.9 两支只取锚在镜像最新根上的一圈窗口里的根，也各有流；
 //! 树表 11 条与种类 9–11（谱系树的容器记录宽、谱系树根的树 ID、谱系树与 logged_ops 两条的发号次序）各有一条，树表条数不超过扫描内核的上限取卡之前就判。
 //! 里程碑三·3 删除线（I-9.5、I-3.4、I-4.7、I-4.6、I-9.6 墓碑那一半）：删除中间态（unlink 意图与第一批）一条流钉四条判过而成立，
-//! 每条各一条只改坏它管的那个字段的流钉 CPU 红、GPU 逐位跟上；走过 nlink = 0 那几版的流只比池级 checker 那一半（恢复那一半今天对不上，见
-//! `compare_the_checker_on_a_deletion_flow`）。
+//! 每条各一条只改坏它管的那个字段的流钉 CPU 红、GPU 逐位跟上；走过 nlink = 0 那几版的流与别的流一样整份比（红位、池级 checker、两遍恢复的结局，
+//! GPU 恢复内核与 CPU 冷走读一样把 nlink = 0 的第一个文件答成没有文件，C712（GPU 恢复内核不判 nlink = 0））；删除中间态那条流上展开上限
+//! 不展开的段（新建文件、顺序写、意图、第一批那几次发布的单元写）另把「之前的段全落、这一段只差一个写」的每个状态各比一次。
 //! I-9.16 ③ ④（种类重复、缺 deadlist）、根槽 428..457（I-2.4）、带树一版根记录里的分配记录树根（I-5.4）各一条坏流整份比；指针头部的
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
 //! extent 下段叶的数据指针、分配记录树内部条目子指针上各一条坏流整份比（core 读这几条指针都判、GPU 恢复内核照判）。
@@ -1873,17 +1874,28 @@
             Layer0SegmentExpansion::NotExpanded
         }
     };
+    compare_on_every_state_of_these_segments(card, flow, &flow.segments, &expansion)
+}
+
+/// 同 [`compare_on_every_state`]，切段与每段怎么展开由调用方给（`segments` 与 `flow.segments` 罩同一组写、只是分法不同时，
+/// 状态仍是这条流全量枚举里的状态）。
+fn compare_on_every_state_of_these_segments(
+    card: &GpuCard,
+    flow: &RecordedFlow,
+    segments: &[Vec<usize>],
+    expansion: &(dyn Fn(usize, &[usize]) -> Layer0SegmentExpansion + Sync),
+) -> Comparison {
     let judge_flow = JudgeFlow {
         base: &flow.base,
         writes: &flow.writes,
-        segments: &flow.segments,
+        segments,
         judged_root_index: flow.judged_root_index,
         versions: &flow.versions,
-        expansion: &expansion,
+        expansion,
     };
     let tables_started = Instant::now();
     let tables = build_judge_tables(&judge_flow).expect("建得出表");
-    let state_count = layer0_plan_state_count(&flow.base, &flow.writes, &flow.segments, &expansion);
+    let state_count = layer0_plan_state_count(&flow.base, &flow.writes, segments, expansion);
     assert_eq!(tables.state_count, state_count);
     let words = tables.words();
     eprintln!(
@@ -1918,13 +1930,6 @@
                 let flow_reference = &*flow;
                 let next_slice = &next_slice;
                 scope.spawn(move || {
-                    let expansion_in_thread = move |_segment_index: usize, segment: &[usize]| {
-                        if segment.len() <= limit {
-                            Layer0SegmentExpansion::EveryProperSubset
-                        } else {
-                            Layer0SegmentExpansion::NotExpanded
-                        }
-                    };
                     let mut judged_slices: Vec<(u64, Vec<JudgedOnCpu>)> = Vec::new();
                     loop {
                         let index = next_slice.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
@@ -1936,10 +1941,10 @@
                         let _state_count = judge_layer0_state_range(
                             &flow_reference.base,
                             &flow_reference.writes,
-                            &flow_reference.segments,
+                            segments,
                             flow_reference.judged_root_index,
                             &flow_reference.versions,
-                            &expansion_in_thread,
+                            expansion,
                             range,
                             &mut |_ordinal, _image, judgement| {
                                 judged.push((
@@ -4870,42 +4875,8 @@
         .copy_from_slice(&0u32.to_le_bytes());
 }
 
-/// 删除线那几条流的对拍：池级 checker 的违例位逐状态逐位相同、一个状态都不许判不了；CPU 要在这条流上红 `must_red`（`None` 不要求）。
-/// 恢复那一半这里不比：CPU 恢复从意图那一版起把 nlink = 0 的第一个文件答成没有文件（`singlefs_core::recovery`，D8（核心索引结构） 已定项 16
-/// 第 4 条），GPU 恢复内核（`crash_judge_common.wgsl` 找第一个文件那一段）不看 nlink、照读文件，两边在删除中间态上的恢复结局对不上
-/// （第 18 批 P2 交回报告里写明、由主 agent 定怎么修），这几条流只钉池级 checker 那一半；恢复的不一致数照样打在 `CRASH_JUDGE_GPU` 那一行。
-fn compare_the_checker_on_a_deletion_flow(
-    test_name: &str,
-    flow: &RecordedFlow,
-    must_red: Option<&'static str>,
-) -> Option<Comparison> {
-    let card = one_usable_card_or_report_not_run(test_name)?;
-    let comparison = compare_on_every_state(&card, flow, expand_up_to());
-    if let Some(invariant) = must_red {
-        assert!(
-            comparison.cpu_checker_red_invariants.contains(invariant),
-            "流 {} 上 CPU 池级 checker 要红 {invariant}（红过的：{:?}）",
-            flow.name,
-            comparison.cpu_checker_red_invariants
-        );
-    }
-    assert!(
-        comparison.checker_mismatches.is_empty(),
-        "流 {}：{} 个状态池级 checker 的违例位 GPU 与 CPU 不一致或判不了（前几个）：\n{}",
-        flow.name,
-        comparison.checker_mismatches.len(),
-        comparison
-            .checker_mismatches
-            .iter()
-            .filter(|line| !line.is_empty())
-            .cloned()
-            .collect::<Vec<_>>()
-            .join("\n")
-    );
-    Some(comparison)
-}
-
-/// 删除中间态（意图之后、第一批之后）的每个状态上 GPU 与 CPU 池级 checker 逐位相同；删除线四条在这条流上 CPU 判过而成立、一个状态都不红
+/// 删除中间态（意图之后、第一批之后）的每个状态上 GPU 与 CPU 整份相同（红位、池级 checker 的违例位、两遍恢复落到的根与结局：意图那一版起
+/// 两边都把 nlink = 0 的第一个文件答成没有文件，C712（GPU 恢复内核不判 nlink = 0））；删除线四条在这条流上 CPU 判过而成立、一个状态都不红
 /// （I-9.5 队列 {1} 与 nlink = 0 的号集合相同、I-3.4 标签 4 那一行在且为 0、I-4.7 号 1 在队列里、I-4.6 单元数等于 ⌈size ÷ 净荷容量⌉）。
 #[test]
 fn the_gpu_checker_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress() {
@@ -4913,13 +4884,19 @@
         "crash-judge-gpu-deletion-in-progress",
         "deletion-through-the-first-batch",
     );
-    let Some(comparison) = compare_the_checker_on_a_deletion_flow(
+    let Some(card) = one_usable_card_or_report_not_run(
         "the_gpu_checker_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress",
-        &flow,
-        None,
     ) else {
         return;
     };
+    let comparison = compare_on_every_state(&card, &flow, expand_up_to());
+    assert!(
+        comparison.mismatches.is_empty(),
+        "流 {}：{} 个状态 GPU 与 CPU 不一致（前几个）：\n{}",
+        flow.name,
+        comparison.mismatches.len(),
+        mismatch_listing(&comparison)
+    );
     for invariant in ["I-9.5", "I-3.4", "I-4.7", "I-4.6"] {
         assert!(
             comparison.cpu_checker_held_invariants.contains(invariant),
@@ -4934,6 +4911,144 @@
     }
 }
 
+/// 一条流换一种切段，造出「之前的段全落、第 `segment_index` 段只差第 `missing_write_index` 次写」那一个状态：`segments` 把原来那一段拆成
+/// 「这一段其余的写」与「只有这一次写」前后两段（别的段照旧），只展开后一段（下标 `segment_index + 1`）。持久集合与原来那一段全量展开
+/// （[`Layer0SegmentExpansion::EveryProperSubset`]）枚举到的状态之一逐个相同；状态只有它与最后全落那一个。
+struct SegmentsMissingOneWrite {
+    segment_index: usize,
+    missing_write_index: usize,
+    segments: Vec<Vec<usize>>,
+}
+
+/// 写数超过 `limit`（按这个展开上限不展开）的每一段、段里每一次写各一种切法（[`SegmentsMissingOneWrite`]）。
+///
+/// # Panics
+/// 这几段里有不是单元写的写：单元写是 COW、只取两态（`TearableInPlaceOverwrites` 认法第 1 条），拆段不改它落不落；原地覆写的撕裂镜像要重放
+/// 同一段里更晚的重叠写，拆开之后那几次写归到前一段，造出的状态就不是原来那一段的了。
+fn segments_missing_one_write_of_each_unexpanded_segment(
+    flow: &RecordedFlow,
+    limit: usize,
+) -> Vec<SegmentsMissingOneWrite> {
+    let mut variants = Vec::new();
+    for (segment_index, segment) in flow.segments.iter().enumerate() {
+        if segment.len() <= limit {
+            continue;
+        }
+        for write_index in segment {
+            assert_eq!(
+                flow.writes[*write_index].kind,
+                StepKind::UnitWrite,
+                "流 {} 不展开的第 {segment_index} 段里第 {write_index} 次写要是单元写，拆段才不改它的落地",
+                flow.name
+            );
+        }
+        for missing_write_index in segment {
+            let rest_of_the_segment: Vec<usize> = segment
+                .iter()
+                .copied()
+                .filter(|write_index| write_index != missing_write_index)
+                .collect();
+            let mut segments = Vec::with_capacity(flow.segments.len() + 1);
+            segments.extend(flow.segments[..segment_index].iter().cloned());
+            segments.push(rest_of_the_segment);
+            segments.push(vec![*missing_write_index]);
+            segments.extend(flow.segments[segment_index + 1..].iter().cloned());
+            variants.push(SegmentsMissingOneWrite {
+                segment_index,
+                missing_write_index: *missing_write_index,
+                segments,
+            });
+        }
+    }
+    variants
+}
+
+/// 删除中间态那条流上，展开上限不展开的段（新建文件、顺序写三个单元、意图、第一批那几次发布的单元写，每段二三十个写）在
+/// [`compare_on_every_state`] 里一个状态都不比：「这一次发布的单元写落了一半」时 GPU 的删除态扫描与恢复内核从没和 CPU 比过
+/// （第 18 批代码轮攻方报告第四节）。这里每段每次写各比一次「之前的段全落、这一段只差这一次写」的状态（连同最后全落那一个），
+/// GPU 与 CPU 整份相同；CPU 那一侧逐状态现判，不抄数。第一批那一段的这些状态上最新落盘的根是意图那一版，第一个文件 nlink = 0，
+/// 两边都要答没有文件（C712（GPU 恢复内核不判 nlink = 0））。每种切法恰好两个状态在取卡之前判，没卡的机器上也判到。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unexpanded_segment_of_a_deletion_in_progress(
+) {
+    let flow = record_new_pool_through_the_first_deletion_batch(
+        "crash-judge-gpu-deletion-missing-one-write",
+        "deletion-through-the-first-batch-missing-one-write",
+    );
+    let limit = expand_up_to();
+    let variants = segments_missing_one_write_of_each_unexpanded_segment(&flow, limit);
+    assert!(
+        !variants.is_empty(),
+        "流 {} 按展开上限 {limit} 要有不展开的段（发布的单元写），这条用例才比得到它们",
+        flow.name
+    );
+    let expansion_of = |variant: &SegmentsMissingOneWrite| {
+        let expanded_segment_index = variant.segment_index + 1;
+        move |segment_index: usize, _segment: &[usize]| {
+            if segment_index == expanded_segment_index {
+                Layer0SegmentExpansion::EveryProperSubset
+            } else {
+                Layer0SegmentExpansion::NotExpanded
+            }
+        }
+    };
+    for variant in &variants {
+        assert_eq!(
+            layer0_plan_state_count(
+                &flow.base,
+                &flow.writes,
+                &variant.segments,
+                &expansion_of(variant)
+            ),
+            2,
+            "第 {} 段只差第 {} 次写的切法只有那一个状态与最后全落那一个",
+            variant.segment_index,
+            variant.missing_write_index
+        );
+    }
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unexpanded_segment_of_a_deletion_in_progress",
+    ) else {
+        return;
+    };
+    let mut failures = Vec::new();
+    for variant in &variants {
+        let comparison = compare_on_every_state_of_these_segments(
+            &card,
+            &flow,
+            &variant.segments,
+            &expansion_of(variant),
+        );
+        if !comparison.mismatches.is_empty() {
+            failures.push(format!(
+                "第 {} 段只差第 {} 次写：\n{}",
+                variant.segment_index,
+                variant.missing_write_index,
+                mismatch_listing(&comparison)
+            ));
+        }
+    }
+    eprintln!(
+        "CRASH_JUDGE_GPU_MISSING_ONE_WRITE flow={} limit={limit} unexpanded_segment_lengths={:?} states_missing_one_write={} failed={}",
+        flow.name,
+        flow.segments
+            .iter()
+            .enumerate()
+            .filter(|(_, segment)| segment.len() > limit)
+            .map(|(segment_index, segment)| (segment_index, segment.len()))
+            .collect::<Vec<_>>(),
+        variants.len(),
+        failures.len()
+    );
+    assert!(
+        failures.is_empty(),
+        "流 {}：{} 个只差一个写的状态 GPU 与 CPU 不一致：\n{}",
+        flow.name,
+        failures.len(),
+        failures.join("\n\n")
+    );
+}
+
 #[test]
 fn the_gpu_checker_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record() {
     let flow = record_deletion_with_a_resealed_field(
@@ -4942,10 +5057,10 @@
         &is_a_deleted_inodes_root,
         &put_an_inode_number_without_a_record_into_the_deletion_queue_root,
     );
-    compare_the_checker_on_a_deletion_flow(
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
         "the_gpu_checker_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record",
         &flow,
-        Some("I-9.5"),
+        "I-9.5",
     );
 }
 
@@ -4957,10 +5072,10 @@
         &is_an_accounting_leaf,
         &set_the_pending_deletion_occupancy_row_to_one,
     );
-    compare_the_checker_on_a_deletion_flow(
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
         "the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero",
         &flow,
-        Some("I-3.4"),
+        "I-3.4",
     );
 }
 
@@ -4972,10 +5087,10 @@
         &is_an_accounting_leaf,
         &drop_the_pending_deletion_occupancy_row,
     );
-    compare_the_checker_on_a_deletion_flow(
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
         "the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_missing",
         &flow,
-        Some("I-3.4"),
+        "I-3.4",
     );
 }
 
@@ -5017,10 +5132,10 @@
         &is_an_inode_leaf_container,
         &raise_the_size_of_the_first_file_by_one_data_unit_payload,
     );
-    compare_the_checker_on_a_deletion_flow(
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
         "the_gpu_checker_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units",
         &flow,
-        Some("I-4.6"),
+        "I-4.6",
     );
 }
 
@@ -5033,9 +5148,9 @@
         &is_a_tombstone_container,
         &raise_the_first_tombstone_object_identifier_past_the_watermark,
     );
-    compare_the_checker_on_a_deletion_flow(
+    assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
         "the_gpu_checker_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark",
         &flow,
-        Some("I-9.6"),
+        "I-9.6",
     );
 }
````

### g：mutations-append.tsv

````
第 20 批 C712 GPU 判器：恢复内核不判 nlink = 0，删除中间态那条流上意图那一版起照读文件（整份比的删除中间态用例红）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {	    if (false && inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_checker_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress	the_gpu_checker_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress
第 20 批 C712 GPU 判器：恢复内核不判 nlink = 0，第一批那一段只差一个写的状态上照读意图那一版的文件（改法 C 那条用例红）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {	    if (false && inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unexpanded_segment_of_a_deletion_in_progress	the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unexpanded_segment_of_a_deletion_in_progress
第 20 批 C712 GPU 判器：恢复内核不判 nlink = 0，墓碑坏流上意图那一版起照读文件（换回整份比的坏流用例红）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {	    if (false && inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_checker_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark	the_gpu_checker_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark
第 20 批 C712 GPU 判器：恢复内核的 nlink = 0 判挪到核单元数之后，删除中而 size 多一个净荷的那一版判成走读失败而不是没有文件	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {\n        return walk_no_file();\n    }\n    // 数据单元数 = max(1, ⌈size ÷ 32634⌉)\n    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);\n    var expected_units = divmod64(add64_saturating(inode_size, sub64(payload, u64_of(1u))), payload).quotient;\n    if (is_zero64(expected_units)) {\n        expected_units = u64_of(1u);\n    }\n    if (!eq64(expected_units, u64_of(data_count))) {\n        return walk_failed(54u);\n    }\n	    // 数据单元数 = max(1, ⌈size ÷ 32634⌉)\n    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);\n    var expected_units = divmod64(add64_saturating(inode_size, sub64(payload, u64_of(1u))), payload).quotient;\n    if (is_zero64(expected_units)) {\n        expected_units = u64_of(1u);\n    }\n    if (!eq64(expected_units, u64_of(data_count))) {\n        return walk_failed(54u);\n    }\n    if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {\n        return walk_no_file();\n    }\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_checker_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units	the_gpu_checker_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units
````

## 实现员 x：crates.patch（/tmp/claude-1000/m3-batch20/x/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs b/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs
--- a/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs
+++ b/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs
@@ -1,7 +1,8 @@
 //! 里程碑三·6（`.claude/kb/milestone/03-third-txn.md` 第三节「三·6」）干跑的 `crates/` 侧只读导出：删除写出的字节。
 //!
-//! 在两块内存盘上依次跑 `singlefs_harness::scenario::run_deletion_history` 的两段历史（H1、H2：E142 那套参数、
-//! 三个数据单元的第一个文件、只供测试的批大小 1），把 unlink 之后的窗口（意图那次发布之前录制流的长度起，到这段历史结束为止）
+//! 在两块内存盘上依次跑 `singlefs_harness::scenario::run_deletion_history` 的三段历史（H1、H2、H3：E142 那套参数、
+//! 三个数据单元的第一个文件、只供测试的批大小 1；H3 在顺序写之前多建一个 size 0 的 2 号 inode、意图的调用方时间晚顺序写 60 秒，
+//! 它的 `name=impl_config` 多打一格 `intent_write_time_seconds=`，E172 跑前登记第五节 5.3），把 unlink 之后的窗口（意图那次发布之前录制流的长度起，到这段历史结束为止）
 //! 里**每一次写**按（步号、设备、字节偏移、字节长度、种类、sha256、整段十六进制）原样打出来，屏障各打一行，
 //! 每一段（一次发布，或不属于任何一次发布的几次写）前后各打一行边界：发布的边界带 txg、哪一种发布、这次重写的角色。
 //! **不带任何位置寻址的布局知识**：不按偏移归类；段的边界与种类来自场景函数（直接调的发布按调用前后录制流的长度，
@@ -21,14 +22,12 @@
 use singlefs_harness::memory_pool::SparseBlockDevice;
 use singlefs_harness::scenario::{
     deletion_history_batch_size, e142_parameters, run_deletion_history,
-    three_data_unit_file_content, DeletionHistory, DeletionHistorySegment,
-    E142_FILESYSTEM_IDENTIFIER, FIXED_WRITE_TIME_SECONDS,
+    three_data_unit_file_content, DeletionHistory, DeletionHistoryIntentWriteTime,
+    DeletionHistorySegment, E142_FILESYSTEM_IDENTIFIER, FIXED_WRITE_TIME_SECONDS,
     SECONDS_FROM_THE_FIRST_FILE_TO_THE_SEQUENTIAL_WRITE_IN_THE_DELETION_HISTORIES,
 };
 use singlefs_harness::sha256::sha256_hexadecimal;
-use singlefs_harness::{
-    RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream,
-};
+use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};
 
 /// E142 那套几何：两块同构盘，`physical_block_size` = io_min = 512（与 `e142_new_pool_file_creation_write_dump` 同一组值）。
 const DEVICE_COUNT: u32 = 2;
@@ -50,32 +49,6 @@
     }
 }
 
-fn kind_name(kind: RecordedOperationKind) -> &'static str {
-    match kind {
-        RecordedOperationKind::Write => "write",
-        RecordedOperationKind::WriteForceUnitAccess => "write_fua",
-        RecordedOperationKind::WriteZeroes => "write_zeroes",
-        RecordedOperationKind::Barrier => "barrier",
-    }
-}
-
-/// 屏障没有字节内容；写清零的内容按长度铺 0（整段全 0 由偏移与长度全定）。
-fn operation_bytes(retained: &RetainedOperation) -> Option<Vec<u8>> {
-    match retained.operation.kind {
-        RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
-            retained.contents.clone()
-        }
-        RecordedOperationKind::WriteZeroes => {
-            Some(vec![
-                0u8;
-                usize::try_from(retained.operation.length)
-                    .expect("窗口里的写清零长度装得进 usize")
-            ])
-        }
-        RecordedOperationKind::Barrier => None,
-    }
-}
-
 /// 一段前后那两行边界的正文（不带 `edge=` 与 `step=`）。
 fn segment_boundary_fields(segment: &DeletionHistorySegment) -> (String, String) {
     match segment {
@@ -129,8 +102,19 @@
         .unwrap_or_else(|failure| panic!("{} 在内存盘上没走完：{failure:?}", history.name()));
     let history_name = history.name();
     let content_bytes = three_data_unit_file_content().len();
+    // 打的是场景交给意图那次发布的调用方时间（`DeletionHistoryRun::intent_write_time_seconds`），不从 core 交回的记录里读：
+    // 比对侧拿它判记录里写的是不是调用方给的那一个。H1、H2 不打这一格，它们的导出与 E172 第一段那一份逐字节相同。
+    let intent_write_time_field = match history.intent_write_time() {
+        DeletionHistoryIntentWriteTime::TheFileRecordsCurrentWriteTime => String::new(),
+        DeletionHistoryIntentWriteTime::SecondsAfterTheSequentialWrite => {
+            format!(
+                " intent_write_time_seconds={}",
+                run.intent_write_time_seconds
+            )
+        }
+    };
     emitter.emit(&format!(
-        "name=impl_config history={history_name} devices={DEVICE_COUNT} image_bytes={TEST_IMAGE_DEFAULT_BYTES} physical_block_size={PHYSICAL_BLOCK_SIZE_IN_BYTES} minimum_input_output_bytes={MINIMUM_INPUT_OUTPUT_BYTES} region_device_0={} region_device_1={} region_device_2={} slots_per_region={} journal_ring_bytes={} first_file_bytes_before_the_sequential_write={} file_bytes={content_bytes} batch_branch={} batch_data_units={} write_time_seconds={FIXED_WRITE_TIME_SECONDS} sequential_write_time_seconds={} filesystem_identifier={} mkfs_operations={}",
+        "name=impl_config history={history_name} devices={DEVICE_COUNT} image_bytes={TEST_IMAGE_DEFAULT_BYTES} physical_block_size={PHYSICAL_BLOCK_SIZE_IN_BYTES} minimum_input_output_bytes={MINIMUM_INPUT_OUTPUT_BYTES} region_device_0={} region_device_1={} region_device_2={} slots_per_region={} journal_ring_bytes={} first_file_bytes_before_the_sequential_write={} file_bytes={content_bytes} batch_branch={} batch_data_units={} write_time_seconds={FIXED_WRITE_TIME_SECONDS} sequential_write_time_seconds={} filesystem_identifier={} mkfs_operations={}{intent_write_time_field}",
         parameters.region_devices[0].0,
         parameters.region_devices[1].0,
         parameters.region_devices[2].0,
@@ -189,14 +173,15 @@
                 continue;
             }
             write_count += 1;
-            let bytes = operation_bytes(retained)
+            let bytes = retained
+                .written_bytes_rebuilding_zero_fills()
                 .expect("write / write_fua / write_zeroes 都能重建出完整字节");
             emitter.emit(&format!(
                 "name=device_region_bytes history={history_name} step={step} device={} offset={} length={} kind={} sha256={} hexadecimal={}",
                 operation.device.0,
                 operation.offset.0,
                 operation.length,
-                kind_name(operation.kind),
+                operation.kind.name(),
                 sha256_hexadecimal(&bytes),
                 hexadecimal_text(&bytes),
             ));
@@ -230,45 +215,3 @@
     }
     emitter.finish();
 }
-
-#[cfg(test)]
-mod tests {
-    use super::{kind_name, operation_bytes};
-    use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
-    use singlefs_harness::{RecordedOperation, RecordedOperationKind, RetainedOperation};
-
-    #[test]
-    fn kind_name_covers_every_recorded_operation_kind_without_a_wildcard_arm() {
-        assert_eq!(kind_name(RecordedOperationKind::Write), "write");
-        assert_eq!(
-            kind_name(RecordedOperationKind::WriteForceUnitAccess),
-            "write_fua"
-        );
-        assert_eq!(
-            kind_name(RecordedOperationKind::WriteZeroes),
-            "write_zeroes"
-        );
-        assert_eq!(kind_name(RecordedOperationKind::Barrier), "barrier");
-    }
-
-    #[test]
-    fn operation_bytes_rebuilds_an_all_zero_buffer_for_write_zeroes_and_none_for_a_barrier() {
-        let operation = |kind, length| RecordedOperation {
-            device: DeviceIdentity(0),
-            kind,
-            offset: DeviceOffsetInBytes(0),
-            length,
-            content_hash: 0,
-        };
-        let write_zeroes = RetainedOperation {
-            operation: operation(RecordedOperationKind::WriteZeroes, 4),
-            contents: None,
-        };
-        let barrier = RetainedOperation {
-            operation: operation(RecordedOperationKind::Barrier, 0),
-            contents: None,
-        };
-        assert_eq!(operation_bytes(&write_zeroes), Some(vec![0, 0, 0, 0]));
-        assert_eq!(operation_bytes(&barrier), None);
-    }
-}
diff --git a/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs b/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs
--- a/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs
+++ b/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs
@@ -16,7 +16,9 @@
 //! 不带参数时两块盘各 `TEST_IMAGE_DEFAULT_BYTES`（4 GiB），输出与这个 bin 只收零个参数时逐字节相同；`--device-bytes` 换每盘字节数
 //! （E142 第 22 次第二段的 P8G：两块 8 GiB 盘，`research/prompts/e142-r22-prereg.md` 读法 R67），别的参数照旧，
 //! `make_filesystem` 收不收由它自己判（环不超过盘的四分之一）。`name=impl_config` 的 `image_bytes=` 打的是这一次的每盘字节数。
-//! 不读环境、不碰真设备、不写任何文件。退出码：0 = 正常；1 = 命令行参数不是上面两种之一。
+//! 不读环境、不碰真设备、不写任何文件。退出码：0 = 正常；1 = 命令行参数不是上面两种之一，或每盘字节数解得开而
+//! `make_filesystem` 不收（例：`--device-bytes 0`），两种都在 stderr 打原因（后一种打 mkfs 交回的错）；
+//! 101 = mkfs 收了、后面某一步（取号、暖机、新池新建文件）在内存盘上没走通，panic 带那一步与原因。
 
 use singlefs_core::address::DeviceIdentity;
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
@@ -24,8 +26,9 @@
 use singlefs_harness::hexadecimal::hexadecimal_text;
 use singlefs_harness::memory_pool::SparseBlockDevice;
 use singlefs_harness::scenario::{
-    e142_parameters, run_new_pool_file_creation, ScenarioPoint, E142_FILESYSTEM_IDENTIFIER,
-    FIRST_FILE_BYTES, FIXED_WRITE_TIME_SECONDS,
+    e142_parameters, run_new_pool_file_creation, NewPoolFileCreationPathFailure,
+    NewPoolFileCreationPathStep, ScenarioPoint, E142_FILESYSTEM_IDENTIFIER, FIRST_FILE_BYTES,
+    FIXED_WRITE_TIME_SECONDS,
 };
 use singlefs_harness::sha256::sha256_hexadecimal;
 use singlefs_harness::{
@@ -58,36 +61,63 @@
     }
 }
 
-fn kind_name(kind: RecordedOperationKind) -> &'static str {
-    match kind {
-        RecordedOperationKind::Write => "write",
-        RecordedOperationKind::WriteForceUnitAccess => "write_fua",
-        RecordedOperationKind::WriteZeroes => "write_zeroes",
-        RecordedOperationKind::Barrier => "barrier",
-    }
+/// 换每盘字节数的那个选项。
+const DEVICE_BYTES_OPTION: &str = "--device-bytes";
+
+/// 命令行给得不对时的退出码（文件头「退出码」那一句）：参数写法不对，或每盘字节数 mkfs 不收。
+const EXIT_CODE_OF_A_REFUSED_COMMAND_LINE: i32 = 1;
+
+/// 整条路没走完时这个 bin 怎么收场。几何与负载的参数都是本地常量，命令行只换每盘字节数，所以 mkfs 不收就是给的每盘字节数不对，
+/// 照命令行给错了收场；mkfs 收了之后哪一步没走通，是实现在内存盘上的毛病，照旧 panic。
+#[derive(Debug, PartialEq, Eq)]
+enum EndingOfAnUnfinishedPath {
+    /// 退 [`EXIT_CODE_OF_A_REFUSED_COMMAND_LINE`]，打 mkfs 交回的错。
+    DeviceBytesRefusedByMakeFilesystem { reason: String },
+    /// panic（退 101），带停在哪一步与那一步的账。
+    StepAfterMakeFilesystemFailed { reason: String },
 }
 
-/// 屏障没有字节内容（不算一次「写」，R2）；写清零的内容按长度铺 0——与 `lib.rs` 的模块注释同一个理由
-/// （整段全 0 由 offset 与 length 全定，重放的人不用真的把它摆出来）。新池新建文件窗口按跑前登记的几何
-/// 不会出现大到扛不住的写清零，这里直接摆出来。
-fn operation_bytes(retained: &RetainedOperation) -> Option<Vec<u8>> {
-    match retained.operation.kind {
-        RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
-            retained.contents.clone()
+fn ending_of_an_unfinished_path(
+    failure: &NewPoolFileCreationPathFailure,
+) -> EndingOfAnUnfinishedPath {
+    match failure.failed_step {
+        NewPoolFileCreationPathStep::MakeFilesystem => {
+            EndingOfAnUnfinishedPath::DeviceBytesRefusedByMakeFilesystem {
+                reason: failure.render(),
+            }
         }
-        RecordedOperationKind::WriteZeroes => {
-            Some(vec![
-                0u8;
-                usize::try_from(retained.operation.length)
-                    .expect("窗口里的写清零长度装得进 usize")
-            ])
+        NewPoolFileCreationPathStep::InstanceAcquisition
+        | NewPoolFileCreationPathStep::WarmUp
+        | NewPoolFileCreationPathStep::NewPoolFileCreation => {
+            EndingOfAnUnfinishedPath::StepAfterMakeFilesystemFailed {
+                reason: format!("{failure:?}"),
+            }
         }
-        RecordedOperationKind::Barrier => None,
     }
 }
 
-/// 换每盘字节数的那个选项。
-const DEVICE_BYTES_OPTION: &str = "--device-bytes";
+/// 两块同构内存盘，各包一层录制器、共用 `stream`。
+fn two_recording_memory_devices(
+    device_bytes: u64,
+    stream: &SharedStream,
+) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
+    (0..DEVICE_COUNT)
+        .map(|device_number| {
+            let identity = DeviceIdentity(device_number);
+            (
+                identity,
+                RecordingBlockDevice::with_shared_stream(
+                    identity,
+                    SparseBlockDevice::new(
+                        device_bytes,
+                        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE_IN_BYTES),
+                    ),
+                    stream.clone(),
+                ),
+            )
+        })
+        .collect()
+}
 
 /// 命令行给的每盘字节数：零个参数是 `TEST_IMAGE_DEFAULT_BYTES`，`--device-bytes <十进制字节数>` 是给的那个数；
 /// 别的写法（多给、少给、数解不开）交回给人看的一句。盘够不够大不在这里判，交给 `make_filesystem`。
@@ -109,7 +139,7 @@
         Ok(device_bytes) => device_bytes,
         Err(refusal) => {
             eprintln!("e142_new_pool_file_creation_write_dump {refusal}");
-            std::process::exit(1);
+            std::process::exit(EXIT_CODE_OF_A_REFUSED_COMMAND_LINE);
         }
     };
 
@@ -132,28 +162,12 @@
 
     let parameters = e142_parameters(PHYSICAL_BLOCK_SIZE_IN_BYTES, MINIMUM_INPUT_OUTPUT_BYTES);
     let stream = SharedStream::retaining_contents();
-    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0
-        ..DEVICE_COUNT)
-        .map(|device_number| {
-            let identity = DeviceIdentity(device_number);
-            (
-                identity,
-                RecordingBlockDevice::with_shared_stream(
-                    identity,
-                    SparseBlockDevice::new(
-                        device_bytes,
-                        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_SIZE_IN_BYTES),
-                    ),
-                    stream.clone(),
-                ),
-            )
-        })
-        .collect();
+    let mut devices = two_recording_memory_devices(device_bytes, &stream);
 
     // 窗口的起点：`ScenarioPoint::BeforeNewPoolFileCreation` 那一次回调时录制流的长度
     // （与 `new_pool_file_creation_region_bytes.rs:94-101` 同一个取法，跑前登记第一节「装置写在哪」第 2 条末句）。
     let mut steps_before_new_pool_file_creation: Option<usize> = None;
-    let run =
+    let path =
         run_new_pool_file_creation(
             &parameters,
             &mut devices,
@@ -164,8 +178,21 @@
                     steps_before_new_pool_file_creation = Some(stream.operation_count());
                 }
             },
-        )
-        .expect("整条路（mkfs → 取号 → 暖机 → 新池新建文件）在内存盘上跑得通");
+        );
+    let run = match path {
+        Ok(run) => run,
+        Err(failure) => match ending_of_an_unfinished_path(&failure) {
+            EndingOfAnUnfinishedPath::DeviceBytesRefusedByMakeFilesystem { reason } => {
+                eprintln!(
+                    "e142_new_pool_file_creation_write_dump 每盘 {device_bytes} 字节 mkfs 不收：{reason}"
+                );
+                std::process::exit(EXIT_CODE_OF_A_REFUSED_COMMAND_LINE);
+            }
+            EndingOfAnUnfinishedPath::StepAfterMakeFilesystemFailed { reason } => panic!(
+                "整条路（mkfs → 取号 → 暖机 → 新池新建文件）mkfs 之后在内存盘上没走通：{reason}"
+            ),
+        },
+    };
     let steps_before_new_pool_file_creation = steps_before_new_pool_file_creation
         .expect("run_new_pool_file_creation 一定走过 BeforeNewPoolFileCreation");
 
@@ -229,14 +256,16 @@
             continue;
         }
         write_count += 1;
-        let bytes =
-            operation_bytes(retained).expect("write / write_fua / write_zeroes 都能重建出完整字节");
+        // 屏障没有字节内容（不算一次「写」，R2）；新池新建文件窗口按跑前登记的几何不会出现大到扛不住的写清零，铺 0 直接摆出来。
+        let bytes = retained
+            .written_bytes_rebuilding_zero_fills()
+            .expect("write / write_fua / write_zeroes 都能重建出完整字节");
         emitter.emit(&format!(
             "name=device_region_bytes step={step_index} device={} offset={} length={} kind={} sha256={} hexadecimal={}",
             operation.device.0,
             operation.offset.0,
             operation.length,
-            kind_name(operation.kind),
+            operation.kind.name(),
             sha256_hexadecimal(&bytes),
             hexadecimal_text(&bytes),
         ));
@@ -250,9 +279,16 @@
 
 #[cfg(test)]
 mod tests {
-    use super::{device_bytes_from_the_command_line, kind_name, operation_bytes};
-    use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
-    use singlefs_harness::{RecordedOperation, RecordedOperationKind, RetainedOperation};
+    use super::{
+        device_bytes_from_the_command_line, ending_of_an_unfinished_path,
+        two_recording_memory_devices, EndingOfAnUnfinishedPath, MINIMUM_INPUT_OUTPUT_BYTES,
+        PHYSICAL_BLOCK_SIZE_IN_BYTES,
+    };
+    use singlefs_harness::scenario::{
+        e142_parameters, run_new_pool_file_creation, NewPoolFileCreationPathFailure,
+        NewPoolFileCreationPathStep,
+    };
+    use singlefs_harness::SharedStream;
 
     fn arguments(words: &[&str]) -> Vec<String> {
         words.iter().map(|word| (*word).to_string()).collect()
@@ -291,63 +327,57 @@
         }
     }
 
+    /// `--device-bytes 0` 解得开（第二种写法），mkfs 不收：照命令行给错了收场、带 mkfs 的原因，不 panic（代码轮判决 N9）。
     #[test]
-    fn kind_name_covers_every_recorded_operation_kind_without_a_wildcard_arm() {
-        assert_eq!(kind_name(RecordedOperationKind::Write), "write");
-        assert_eq!(
-            kind_name(RecordedOperationKind::WriteForceUnitAccess),
-            "write_fua"
-        );
-        assert_eq!(
-            kind_name(RecordedOperationKind::WriteZeroes),
-            "write_zeroes"
-        );
-        assert_eq!(kind_name(RecordedOperationKind::Barrier), "barrier");
-    }
-
-    fn retained(
-        kind: RecordedOperationKind,
-        length: u64,
-        contents: Option<Vec<u8>>,
-    ) -> RetainedOperation {
-        RetainedOperation {
-            operation: RecordedOperation {
-                device: DeviceIdentity(0),
-                kind,
-                offset: DeviceOffsetInBytes(0),
-                length,
-                content_hash: 0,
-            },
-            contents,
+    fn zero_device_bytes_parse_but_make_filesystem_refuses_them_and_the_bin_ends_as_a_refused_command_line(
+    ) {
+        let device_bytes =
+            device_bytes_from_the_command_line(&["--device-bytes".to_string(), "0".to_string()])
+                .expect("0 是一个十进制字节数，解得开");
+        let stream = SharedStream::retaining_contents();
+        let mut devices = two_recording_memory_devices(device_bytes, &stream);
+        let failure = match run_new_pool_file_creation(
+            &e142_parameters(PHYSICAL_BLOCK_SIZE_IN_BYTES, MINIMUM_INPUT_OUTPUT_BYTES),
+            &mut devices,
+            &stream,
+            |_point, _devices| {},
+        ) {
+            Ok(_) => panic!("每盘 0 字节 mkfs 不该收"),
+            Err(failure) => failure,
+        };
+        match ending_of_an_unfinished_path(&failure) {
+            EndingOfAnUnfinishedPath::DeviceBytesRefusedByMakeFilesystem { reason } => assert!(
+                reason.contains("JournalRingTooLargeForDevice"),
+                "原因要带 mkfs 交回的错：{reason}"
+            ),
+            EndingOfAnUnfinishedPath::StepAfterMakeFilesystemFailed { reason } => {
+                panic!("mkfs 拒的值要照命令行给错了收场，不 panic：{reason}")
+            }
         }
+        assert_eq!(stream.operation_count(), 0, "mkfs 拒在任何写之前");
     }
 
+    /// mkfs 收了而后面某一步没走通的，照旧当实现的毛病（panic），不混进命令行给错了那一种。
     #[test]
-    fn operation_bytes_returns_the_retained_content_for_a_plain_write() {
-        let write = retained(RecordedOperationKind::Write, 3, Some(vec![1, 2, 3]));
-        assert_eq!(operation_bytes(&write), Some(vec![1, 2, 3]));
-    }
-
-    #[test]
-    fn operation_bytes_returns_the_retained_content_for_a_force_unit_access_write() {
-        let write = retained(
-            RecordedOperationKind::WriteForceUnitAccess,
-            2,
-            Some(vec![9, 9]),
-        );
-        assert_eq!(operation_bytes(&write), Some(vec![9, 9]));
-    }
-
-    #[test]
-    fn operation_bytes_rebuilds_an_all_zero_buffer_for_write_zeroes_even_without_retained_content()
-    {
-        let write_zeroes = retained(RecordedOperationKind::WriteZeroes, 4, None);
-        assert_eq!(operation_bytes(&write_zeroes), Some(vec![0, 0, 0, 0]));
-    }
-
-    #[test]
-    fn operation_bytes_is_none_for_a_barrier_which_carries_no_bytes() {
-        let barrier = retained(RecordedOperationKind::Barrier, 0, None);
-        assert_eq!(operation_bytes(&barrier), None);
+    fn a_failure_after_make_filesystem_stays_an_implementation_failure() {
+        for failed_step in [
+            NewPoolFileCreationPathStep::InstanceAcquisition,
+            NewPoolFileCreationPathStep::WarmUp,
+            NewPoolFileCreationPathStep::NewPoolFileCreation,
+        ] {
+            let failure = NewPoolFileCreationPathFailure {
+                failed_step,
+                cause: "内存盘上没走通".to_string(),
+                writes_of_failed_publishes: Vec::new(),
+                writes_of_persisted_publishes: Vec::new(),
+            };
+            assert!(
+                matches!(
+                    ending_of_an_unfinished_path(&failure),
+                    EndingOfAnUnfinishedPath::StepAfterMakeFilesystemFailed { .. }
+                ),
+                "{failed_step:?} 停下是实现的毛病"
+            );
+        }
     }
 }
diff --git a/crates/singlefs-harness/src/lib.rs b/crates/singlefs-harness/src/lib.rs
--- a/crates/singlefs-harness/src/lib.rs
+++ b/crates/singlefs-harness/src/lib.rs
@@ -40,6 +40,19 @@
     Barrier,
 }
 
+impl RecordedOperationKind {
+    /// 流文本（[`RecordedOperation::to_stream_line`] 的 `kind=`）与各导出装置结果行（`kind=`）里的种类名：全仓这一处定义，不各抄一份。
+    #[must_use]
+    pub const fn name(self) -> &'static str {
+        match self {
+            RecordedOperationKind::Write => "write",
+            RecordedOperationKind::WriteForceUnitAccess => "write_fua",
+            RecordedOperationKind::WriteZeroes => "write_zeroes",
+            RecordedOperationKind::Barrier => "barrier",
+        }
+    }
+}
+
 /// 录制流的一条。
 #[derive(Clone, Debug, PartialEq, Eq)]
 pub struct RecordedOperation {
@@ -56,12 +69,7 @@
     /// 一行一条，机器能重新解析；字段顺序固定。
     #[must_use]
     pub fn to_stream_line(&self) -> String {
-        let kind = match self.kind {
-            RecordedOperationKind::Write => "write",
-            RecordedOperationKind::WriteForceUnitAccess => "write_fua",
-            RecordedOperationKind::WriteZeroes => "write_zeroes",
-            RecordedOperationKind::Barrier => "barrier",
-        };
+        let kind = self.kind.name();
         format!(
             "device={} kind={kind} offset={} length={} hash={:016x}",
             self.device.0, self.offset.0, self.length, self.content_hash
@@ -110,6 +118,31 @@
     pub contents: Option<Vec<u8>>,
 }
 
+impl RetainedOperation {
+    /// 这一步写到盘上的字节：普通写与 FUA 写是留下的内容（没开内容保留的流里是 `None`）；写清零按长度现铺 0
+    /// （录制器不为它留内容，整段全 0 由偏移与长度全定，见 [`RecordingBlockDevice`] 的 `write_zeroes_at`）；屏障没有字节，`None`。
+    /// 写清零真的摆进内存：mkfs 那一次清 768 MiB，调用方只对窗口里的写调它。
+    ///
+    /// # Panics
+    /// 写清零的长度装不进 `usize`（只在 32 位目标上可能）。
+    #[must_use]
+    pub fn written_bytes_rebuilding_zero_fills(&self) -> Option<Vec<u8>> {
+        match self.operation.kind {
+            RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
+                self.contents.clone()
+            }
+            RecordedOperationKind::WriteZeroes => Some(vec![
+                0u8;
+                usize::try_from(self.operation.length)
+                    .expect(
+                        "写清零的长度装得进 usize：64 位目标上恒成立"
+                    )
+            ]),
+            RecordedOperationKind::Barrier => None,
+        }
+    }
+}
+
 /// 录制流里一段操作是从哪个入口发出来的（C557（卸载记号只由卸载入口打没有检查）：录制流要记每次发布的入口）。
 /// 块设备一层看不见入口，由驱动这条流的一方在调入口时记（[`SharedStream::record_entry`]）。
 /// 只登记要核的那一个：正常卸载那一串；没记下入口的操作一律按「不是卸载入口发的」算。
@@ -619,6 +652,64 @@
         assert_eq!(before, [1u8; 512], "清零段之外一个字节不动");
     }
 
+    /// 种类名是流文本与各导出结果行里的 `kind=`：每一种一个名字，四个互不相同（E142、E172 的比对侧按这几个字面认种类）。
+    #[test]
+    fn recorded_operation_kind_names_are_the_four_literals_the_dumps_print() {
+        assert_eq!(RecordedOperationKind::Write.name(), "write");
+        assert_eq!(
+            RecordedOperationKind::WriteForceUnitAccess.name(),
+            "write_fua"
+        );
+        assert_eq!(
+            RecordedOperationKind::WriteZeroes.name(),
+            "write_zeroes",
+            "写清零打成 write 的话，比对侧会去流里找它没留的内容"
+        );
+        assert_eq!(RecordedOperationKind::Barrier.name(), "barrier");
+    }
+
+    /// 普通写与 FUA 写交回留下的内容；写清零没留内容也按长度铺 0；屏障没有字节。
+    #[test]
+    fn written_bytes_are_the_retained_content_the_zero_fill_by_length_and_none_for_a_barrier() {
+        let retained = |kind, length, contents| RetainedOperation {
+            operation: RecordedOperation {
+                device: DeviceIdentity(0),
+                kind,
+                offset: DeviceOffsetInBytes(0),
+                length,
+                content_hash: 0,
+            },
+            contents,
+        };
+        assert_eq!(
+            retained(RecordedOperationKind::Write, 3, Some(vec![1, 2, 3]))
+                .written_bytes_rebuilding_zero_fills(),
+            Some(vec![1, 2, 3]),
+            "普通写是留下的内容"
+        );
+        assert_eq!(
+            retained(
+                RecordedOperationKind::WriteForceUnitAccess,
+                2,
+                Some(vec![9, 9])
+            )
+            .written_bytes_rebuilding_zero_fills(),
+            Some(vec![9, 9]),
+            "FUA 写是留下的内容"
+        );
+        assert_eq!(
+            retained(RecordedOperationKind::WriteZeroes, 4, None)
+                .written_bytes_rebuilding_zero_fills(),
+            Some(vec![0, 0, 0, 0]),
+            "写清零没留内容，按长度铺 4 个 0"
+        );
+        assert_eq!(
+            retained(RecordedOperationKind::Barrier, 0, None).written_bytes_rebuilding_zero_fills(),
+            None,
+            "屏障没有字节（不算一次写）"
+        );
+    }
+
     /// 全 0 串的哈希走的是平方乘的捷径，必须与真的把那一串 0 喂给 [`fnv1a_64`] 逐位相同。
     /// 长度取 0、1、2、3、512、4096 与一个不是 2 的幂的奇数长（平方乘的两条臂都要走到）。
     #[test]
diff --git a/crates/singlefs-harness/src/scenario.rs b/crates/singlefs-harness/src/scenario.rs
--- a/crates/singlefs-harness/src/scenario.rs
+++ b/crates/singlefs-harness/src/scenario.rs
@@ -18,9 +18,9 @@
 use singlefs_core::root_ring::RootRingSlotsPerRegion;
 use singlefs_core::system_configuration::SystemImmutableSizes;
 use singlefs_core::transaction::{
-    acquire_instance, publish_first_file, publish_sequential_write, publish_unlink_intent,
-    publish_unlink_step, warm_up, FirstFile, PoolVersion, PoolWriter, TransactionOutput,
-    WarmUpOutput, FIRST_INODE_NUMBER,
+    acquire_instance, publish_first_file, publish_new_inodes, publish_sequential_write,
+    publish_unlink_intent, publish_unlink_step, warm_up, FirstFile, PoolVersion, PoolWriter,
+    TransactionOutput, WarmUpOutput, FIRST_INODE_NUMBER,
 };
 use singlefs_core::unit::data_unit_payload_capacity;
 use singlefs_core::unlink::{
@@ -223,7 +223,7 @@
     ))
 }
 
-/// 新池新建文件那个实例：删除那两段历史里在这个进程里发的每一次（顺序写、意图、H1 的各批）都在它里面发。
+/// 新池新建文件那个实例：删除历史里在这个进程里发的每一次（H3 建 size 0 inode 那次、顺序写、意图、H1 与 H3 的各批）都在它里面发。
 const INSTANCE_OF_THE_NEW_POOL_FILE_CREATION: InstanceGeneration = InstanceGeneration(1);
 
 /// 删除那两段历史删的是第一个文件。
@@ -232,6 +232,11 @@
 /// 删除那两段历史里顺序写比新池新建文件晚几秒（与 harness 档 unlink 那几条用例的 `OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE` 同一个数）。
 pub const SECONDS_FROM_THE_FIRST_FILE_TO_THE_SEQUENTIAL_WRITE_IN_THE_DELETION_HISTORIES: u64 = 60;
 
+/// H3 那个 size 0 inode 建出来时的写入时间。E172 跑前登记第五节 5.3 没给这个时间；取新池新建文件那一个，
+/// 与顺序写（晚 60 秒）、意图（再晚 60 秒）三个时间各不相同，比对侧读错了哪条记录的时间看得出来。
+pub const WRITE_TIME_SECONDS_OF_THE_SIZE_ZERO_INODE_BESIDE_THE_DELETED_FILE: u64 =
+    FIXED_WRITE_TIME_SECONDS;
+
 /// 删除那两段历史里第一个文件顺序写成的内容：两个满净荷加半个净荷，三个数据单元（与 harness 档 unlink 那几条用例的
 /// `pool_with_a_three_unit_file` 同一份字节）。
 #[must_use]
@@ -262,27 +267,81 @@
     }
 }
 
-/// 三·6 干跑导出的两段删除历史（调查报告第 4.3 节表里的 H1、H2）。
+/// H3 里意图那次的调用方时间比顺序写晚几秒（E172 跑前登记第五节 5.3 ②：与那条记录现有的时间不同，意图写的是不是调用方给的那一个才分辨得出）。
+pub const SECONDS_FROM_THE_SEQUENTIAL_WRITE_TO_THE_INTENT_IN_THE_HISTORY_BESIDE_A_SIZE_ZERO_INODE:
+    u64 = 60;
+
+/// 三·6 干跑导出的删除历史（调查报告第 4.3 节表里的 H1、H2；H3 是 E172 跑前登记第五节 5.3 加的变体）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum DeletionHistory {
     /// H1：意图与每一批都在新池新建文件那个进程里发，直到队列空。
     IntentAndEveryBatchInProcess,
     /// H2：意图在这个进程里发，正常卸载，同一组盘重开可写挂载，挂载续做在写行与暖机之后把队列走到空。
     IntentThenUnmountThenMountContinuation,
+    /// H3：同 H1，只改两处——顺序写之前在同一个实例里建一个 size 0 的 inode（号 2，与 1 号在同一片 inode 叶容器里）；
+    /// 意图的调用方时间 = 顺序写的时间 + [`SECONDS_FROM_THE_SEQUENTIAL_WRITE_TO_THE_INTENT_IN_THE_HISTORY_BESIDE_A_SIZE_ZERO_INODE`]。
+    LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess,
+}
+
+/// 删除历史里被删的第一个文件旁边还有没有别的 inode。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum InodesBesideTheDeletedFile {
+    /// 只有第一个文件（H1、H2）。
+    NoOtherInode,
+    /// 顺序写之前在同一个实例里用 `publish_new_inodes` 建一个 size 0 的 inode（H3）。
+    OneSizeZeroInodeCreatedBeforeTheSequentialWrite,
+}
+
+/// 意图那次发布的调用方时间从哪来。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum DeletionHistoryIntentWriteTime {
+    /// 第一个文件那条记录现有的写入时间（顺序写写进去的那一个）：与 harness 档 unlink 那几条用例同一个取法（H1、H2）。
+    TheFileRecordsCurrentWriteTime,
+    /// 那条记录现有的写入时间再加
+    /// [`SECONDS_FROM_THE_SEQUENTIAL_WRITE_TO_THE_INTENT_IN_THE_HISTORY_BESIDE_A_SIZE_ZERO_INODE`]（H3）。
+    SecondsAfterTheSequentialWrite,
 }
 
 impl DeletionHistory {
-    pub const EVERY_HISTORY: [DeletionHistory; 2] = [
+    pub const EVERY_HISTORY: [DeletionHistory; 3] = [
         DeletionHistory::IntentAndEveryBatchInProcess,
         DeletionHistory::IntentThenUnmountThenMountContinuation,
+        DeletionHistory::LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess,
     ];
 
-    /// 结果行里的名字，与调查报告那张表的行名相同。
+    /// 结果行里的名字，与调查报告那张表的行名相同（H3 与 E172 跑前登记第五节 5.3 的名字相同）。
     #[must_use]
     pub const fn name(self) -> &'static str {
         match self {
             DeletionHistory::IntentAndEveryBatchInProcess => "h1",
             DeletionHistory::IntentThenUnmountThenMountContinuation => "h2",
+            DeletionHistory::LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess => "h3",
+        }
+    }
+
+    #[must_use]
+    pub const fn inodes_beside_the_deleted_file(self) -> InodesBesideTheDeletedFile {
+        match self {
+            DeletionHistory::IntentAndEveryBatchInProcess
+            | DeletionHistory::IntentThenUnmountThenMountContinuation => {
+                InodesBesideTheDeletedFile::NoOtherInode
+            }
+            DeletionHistory::LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess => {
+                InodesBesideTheDeletedFile::OneSizeZeroInodeCreatedBeforeTheSequentialWrite
+            }
+        }
+    }
+
+    #[must_use]
+    pub const fn intent_write_time(self) -> DeletionHistoryIntentWriteTime {
+        match self {
+            DeletionHistory::IntentAndEveryBatchInProcess
+            | DeletionHistory::IntentThenUnmountThenMountContinuation => {
+                DeletionHistoryIntentWriteTime::TheFileRecordsCurrentWriteTime
+            }
+            DeletionHistory::LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess => {
+                DeletionHistoryIntentWriteTime::SecondsAfterTheSequentialWrite
+            }
         }
     }
 }
@@ -376,6 +435,9 @@
     pub mkfs_operation_count: usize,
     /// 意图那次发布之前录制流的长度：导出的窗口从这里起。
     pub operations_before_the_unlink: usize,
+    /// 场景交给意图那次发布的调用方时间（[`DeletionHistory::intent_write_time`] 定取法）。是交进去的值，不是 core 写进记录的值：
+    /// 两者相不相等正是要判的。
+    pub intent_write_time_seconds: u64,
     /// unlink 之后的每一段，按录制流的先后，首尾相接铺满 [`operations_before_the_unlink`](Self::operations_before_the_unlink) 到流尾。
     pub segments: Vec<DeletionHistorySegment>,
     /// 这段历史最后的现行那一版：H1 是这个进程最后一批，H2 是挂载交回的现行那一版。
@@ -386,6 +448,8 @@
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub enum DeletionHistoryStep {
     NewPoolFileCreationPath(NewPoolFileCreationPathStep),
+    /// H3 顺序写之前建 size 0 inode 的那次发布。
+    SizeZeroInodeCreation,
     SequentialWrite,
     UnlinkIntent,
     UnlinkBatchInProcess,
@@ -537,7 +601,8 @@
 /// （与 [`run_new_pool_file_creation`] 同一条路）→ 同一个实例里把第一个文件顺序写成三个数据单元（[`three_data_unit_file_content`]）
 /// → unlink，批大小 [`deletion_history_batch_size`]。H1 意图与每一批都在这个进程里发，直到队列空；H2 意图在这个进程里发，
 /// 正常卸载（[`unmount`]，录制流把那一截记成卸载入口），丢掉这个进程的分配器，同一组盘重开可写挂载（[`deletion_history_switches`]）：
-/// 挂载续做在写行与暖机之后把队列走到空。
+/// 挂载续做在写行与暖机之后把队列走到空。H3 照 H1，只多两处（[`DeletionHistory::inodes_beside_the_deleted_file`]、
+/// [`DeletionHistory::intent_write_time`]）：顺序写之前先建一个 size 0 的 inode，意图的调用方时间晚顺序写 60 秒。
 ///
 /// 交回 unlink 之前录制流的长度与它之后的每一段：直接调的发布各一段；卸载与挂载里的几次发布按 core 交回的每次发布写账
 /// （`TransactionOutput::writes`，一次写调用 = 交给一块盘的一次写）在流里切开（[`spans_of_consecutive_parts_by_their_write_calls`]）。
@@ -563,11 +628,30 @@
             failure.cause,
         )
     })?;
+    let version_before_the_sequential_write = match history.inodes_beside_the_deleted_file() {
+        InodesBesideTheDeletedFile::NoOtherInode => new_pool_file_creation.output,
+        InodesBesideTheDeletedFile::OneSizeZeroInodeCreatedBeforeTheSequentialWrite => {
+            publish_new_inodes(
+                &mut PoolWriter::new(parameters, devices.as_mut_slice()),
+                &mut allocator,
+                &new_pool_file_creation.output,
+                1,
+                WRITE_TIME_SECONDS_OF_THE_SIZE_ZERO_INODE_BESIDE_THE_DELETED_FILE,
+                INSTANCE_OF_THE_NEW_POOL_FILE_CREATION,
+            )
+            .map_err(|error| {
+                DeletionHistoryFailure::at(
+                    DeletionHistoryStep::SizeZeroInodeCreation,
+                    format!("{error:?}"),
+                )
+            })?
+        }
+    };
     let content = three_data_unit_file_content();
     let three_units = publish_sequential_write(
         &mut PoolWriter::new(parameters, devices.as_mut_slice()),
         &mut allocator,
-        &new_pool_file_creation.output,
+        &version_before_the_sequential_write,
         FirstFile {
             content: &content,
             write_time_seconds: FIXED_WRITE_TIME_SECONDS
@@ -579,17 +663,23 @@
         DeletionHistoryFailure::at(DeletionHistoryStep::SequentialWrite, format!("{error:?}"))
     })?;
     let operations_before_the_unlink = stream.operation_count();
-    // 意图带第一个文件那条记录现有的写入时间：与 harness 档 unlink 那几条用例同一个取法。
     let write_time_seconds = three_units
         .inode_record
         .expect("顺序写之后第一个文件那条记录在树里")
         .write_time_seconds;
+    let intent_write_time_seconds = match history.intent_write_time() {
+        DeletionHistoryIntentWriteTime::TheFileRecordsCurrentWriteTime => write_time_seconds,
+        DeletionHistoryIntentWriteTime::SecondsAfterTheSequentialWrite => {
+            write_time_seconds
+                + SECONDS_FROM_THE_SEQUENTIAL_WRITE_TO_THE_INTENT_IN_THE_HISTORY_BESIDE_A_SIZE_ZERO_INODE
+        }
+    };
     let intent = publish_unlink_intent(
         &mut PoolWriter::new(parameters, devices.as_mut_slice()),
         &mut allocator,
         &three_units,
         FILE_DELETED_IN_THE_DELETION_HISTORIES,
-        write_time_seconds,
+        intent_write_time_seconds,
         INSTANCE_OF_THE_NEW_POOL_FILE_CREATION,
     )
     .map_err(|error| {
@@ -601,7 +691,8 @@
         operations: operations_before_the_unlink..stream.operation_count(),
     };
     let (later_segments, final_version) = match history {
-        DeletionHistory::IntentAndEveryBatchInProcess => {
+        DeletionHistory::IntentAndEveryBatchInProcess
+        | DeletionHistory::LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess => {
             every_batch_in_process(parameters, devices, stream, &mut allocator, intent)?
         }
         DeletionHistory::IntentThenUnmountThenMountContinuation => {
@@ -611,6 +702,7 @@
     Ok(DeletionHistoryRun {
         mkfs_operation_count: new_pool_file_creation.mkfs_operation_count,
         operations_before_the_unlink,
+        intent_write_time_seconds,
         segments: std::iter::once(intent_segment)
             .chain(later_segments)
             .collect(),
diff --git a/crates/singlefs-harness/tests/deletion_scenario_writes.rs b/crates/singlefs-harness/tests/deletion_scenario_writes.rs
--- a/crates/singlefs-harness/tests/deletion_scenario_writes.rs
+++ b/crates/singlefs-harness/tests/deletion_scenario_writes.rs
@@ -3,9 +3,11 @@
 //! 进程里那三批的单元数照 `unlink_single_file_publishes.rs` 现钉的角色集合：意图 13、3 → 2 那一批 12、最后一批 10；2 → 1 那一批 11。
 //! 挂载续做那三批在卸载、写行与暖机之后发，分配记录树根之下多改一对叶（`t5@0.0.62`、`t5@0.1.62`），后两批各多 2 个；
 //! 分配记录树根之下那几片之外的角色数两段历史逐批相同（9、8、7、6）。
+//! H3（E172 跑前登记第五节 5.3）只钉它自己的形状：2 号 size 0 的 inode 先建、与 1 号同一片叶，意图时间晚顺序写 60 秒、各批照抄。
 
 use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::records::InodeRecord;
 use singlefs_core::transaction::{PoolVersion, TransactionOutput, TransactionUnit};
 use singlefs_core::write_accounting::WrittenStructureKind;
 use singlefs_format::TEST_IMAGE_DEFAULT_BYTES;
@@ -295,3 +297,115 @@
     );
     assert_eq!(segment_mismatches(&run, &operations), Vec::<String>::new());
 }
+
+/// 一版 inode 树里号是 `inode` 的那条记录，连同它住的那片叶容器在左起第几片。
+fn leaf_container_index_and_record(
+    version: &TransactionOutput,
+    inode: u64,
+) -> Option<(usize, InodeRecord)> {
+    version
+        .inode_leaf_containers
+        .iter()
+        .enumerate()
+        .find_map(|(container_index, container)| {
+            container
+                .contents
+                .records
+                .iter()
+                .find(|record| record.inode == inode)
+                .map(|record| (container_index, *record))
+        })
+}
+
+#[test]
+fn the_history_beside_a_size_zero_inode_creates_inode_two_before_the_sequential_write_and_gives_the_intent_sixty_seconds_after_the_write_which_every_batch_copies(
+) {
+    let (run, operations) = run_on_two_memory_devices(
+        DeletionHistory::LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess,
+    );
+    let deletion_publishes: Vec<(DeletionHistoryPublishKind, &TransactionOutput)> = publishes(&run);
+    assert_eq!(
+        deletion_publishes
+            .iter()
+            .map(|(kind, version)| (*kind, version.data_pointers.len()))
+            .collect::<Vec<_>>(),
+        [
+            (DeletionHistoryPublishKind::UnlinkIntent, 3),
+            (DeletionHistoryPublishKind::UnlinkBatchInProcess, 2),
+            (DeletionHistoryPublishKind::UnlinkBatchInProcess, 1),
+            (DeletionHistoryPublishKind::UnlinkBatchInProcess, 0),
+        ],
+        "同 H1：意图之后三批都在这个进程里发，批大小 1"
+    );
+    let (_, intent) = deletion_publishes[0];
+    let intent_txg = intent.root.checkpoint_txg.0;
+    assert_eq!(
+        deletion_publishes
+            .iter()
+            .map(|(_, version)| version.root.checkpoint_txg.0)
+            .collect::<Vec<_>>(),
+        (intent_txg..intent_txg + 4).collect::<Vec<_>>(),
+        "四次发布按 txg 连着"
+    );
+
+    let (container_of_the_file, file_at_the_intent) =
+        leaf_container_index_and_record(intent, 1).expect("意图那一版 1 号在树里");
+    let (container_of_the_size_zero_inode, size_zero_inode) =
+        leaf_container_index_and_record(intent, 2).expect("意图那一版 2 号在树里");
+    assert_eq!(
+        container_of_the_size_zero_inode, container_of_the_file,
+        "2 号与 1 号在同一片 inode 叶容器里"
+    );
+    assert_eq!(
+        (size_zero_inode.nlink, size_zero_inode.size),
+        (1, 0),
+        "2 号是活着的、size 0"
+    );
+    assert_eq!(
+        size_zero_inode.object_birth.0 + 2,
+        intent_txg,
+        "2 号建在顺序写之前：建 2 号、顺序写、意图三次发布连着"
+    );
+    assert_eq!(
+        size_zero_inode.write_time_seconds, 1_788_000_000,
+        "2 号建出来时取新池新建文件的时间"
+    );
+
+    assert_eq!(
+        run.intent_write_time_seconds, 1_788_000_120,
+        "交给意图的调用方时间 = 1 788 000 000 + 60（顺序写）+ 60"
+    );
+    assert_eq!(
+        (
+            file_at_the_intent.nlink,
+            file_at_the_intent.write_time_seconds
+        ),
+        (0, 1_788_000_120),
+        "意图把 1 号写成 nlink 0，时间是调用方给的那一个、不是顺序写写进去的 1 788 000 060"
+    );
+    for (_, batch) in &deletion_publishes[1..3] {
+        let (_, file_after_the_batch) =
+            leaf_container_index_and_record(batch, 1).expect("最后一批之前 1 号还在树里");
+        assert_eq!(
+            file_after_the_batch.write_time_seconds, 1_788_000_120,
+            "txg {} 那一批照抄意图那次写进去的时间",
+            batch.root.checkpoint_txg.0
+        );
+    }
+    let (_, final_batch) = deletion_publishes[3];
+    assert_eq!(
+        leaf_container_index_and_record(final_batch, 1),
+        None,
+        "最后一批删掉 1 号那条记录"
+    );
+    assert_eq!(
+        leaf_container_index_and_record(final_batch, 2).map(|(_, record)| record),
+        Some(size_zero_inode),
+        "2 号那条记录从意图到最后一批一个字段都不变"
+    );
+    assert!(
+        matches!(&run.final_version, PoolVersion::WithFile(version) if version.deleted_inodes.is_empty() && version.inode_record.is_none()),
+        "最后一批之后队列空；还剩 2 号，仍是带文件的一版（D8 已定项 16 第 5 条）"
+    );
+    assert_eq!(segment_mismatches(&run, &operations), Vec::<String>::new());
+}
````

### x：mutations-append.tsv

````
三·6 删除历史 H3：意图的调用方时间不加 60 秒（与那条记录现有的时间相同，意图写的是不是调用方给的时间又分辨不了）	crates/singlefs-harness/src/scenario.rs	            write_time_seconds\n                + SECONDS_FROM_THE_SEQUENTIAL_WRITE_TO_THE_INTENT_IN_THE_HISTORY_BESIDE_A_SIZE_ZERO_INODE	            write_time_seconds	-p singlefs-harness --test deletion_scenario_writes	the_history_beside_a_size_zero_inode_creates_inode_two_before_the_sequential_write_and_gives_the_intent_sixty_seconds_after_the_write_which_every_batch_copies
三·6 删除历史 H3：顺序写之前一个 inode 都不建（publish_new_inodes 建 0 个，叶里没有 2 号）	crates/singlefs-harness/src/scenario.rs	                1,\n                WRITE_TIME_SECONDS_OF_THE_SIZE_ZERO_INODE_BESIDE_THE_DELETED_FILE,	                0,\n                WRITE_TIME_SECONDS_OF_THE_SIZE_ZERO_INODE_BESIDE_THE_DELETED_FILE,	-p singlefs-harness --test deletion_scenario_writes	the_history_beside_a_size_zero_inode_creates_inode_two_before_the_sequential_write_and_gives_the_intent_sixty_seconds_after_the_write_which_every_batch_copies
录制流种类名挪进 harness：写清零打成 write（导出比对侧会去流里找它没留的内容）	crates/singlefs-harness/src/lib.rs	            RecordedOperationKind::WriteZeroes => "write_zeroes",	            RecordedOperationKind::WriteZeroes => "write",	-p singlefs-harness --lib	recorded_operation_kind_names_are_the_four_literals_the_dumps_print
录制流写的字节挪进 harness：屏障也重建出（空的）字节	crates/singlefs-harness/src/lib.rs	            RecordedOperationKind::Barrier => None,	            RecordedOperationKind::Barrier => Some(Vec::new()),	-p singlefs-harness --lib	written_bytes_are_the_retained_content_the_zero_fill_by_length_and_none_for_a_barrier
录制流写的字节挪进 harness：写清零按长度铺 1 而不是 0	crates/singlefs-harness/src/lib.rs	            RecordedOperationKind::WriteZeroes => Some(vec![\n                0u8;	            RecordedOperationKind::WriteZeroes => Some(vec![\n                1u8;	-p singlefs-harness --lib	written_bytes_are_the_retained_content_the_zero_fill_by_length_and_none_for_a_barrier
录制流写的字节挪进 harness：普通写与 FUA 写交回空（不交留下的内容）	crates/singlefs-harness/src/lib.rs	                self.contents.clone()	                None	-p singlefs-harness --lib	written_bytes_are_the_retained_content_the_zero_fill_by_length_and_none_for_a_barrier
E142 导出 N9：mkfs 拒的每盘字节数又当实现的毛病 panic（退 101，与文件头退出码对不上）	crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs	        NewPoolFileCreationPathStep::MakeFilesystem => {\n            EndingOfAnUnfinishedPath::DeviceBytesRefusedByMakeFilesystem {	        NewPoolFileCreationPathStep::MakeFilesystem => {\n            EndingOfAnUnfinishedPath::StepAfterMakeFilesystemFailed {	-p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump	zero_device_bytes_parse_but_make_filesystem_refuses_them_and_the_bin_ends_as_a_refused_command_line
E142 导出 N9：暖机停下也当成 mkfs 拒了每盘字节数（实现的毛病退 1、不 panic）	crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs	        NewPoolFileCreationPathStep::InstanceAcquisition\n        | NewPoolFileCreationPathStep::WarmUp\n        | NewPoolFileCreationPathStep::NewPoolFileCreation => {	        NewPoolFileCreationPathStep::WarmUp => EndingOfAnUnfinishedPath::DeviceBytesRefusedByMakeFilesystem { reason: failure.render() },\n        NewPoolFileCreationPathStep::InstanceAcquisition\n        | NewPoolFileCreationPathStep::NewPoolFileCreation => {	-p singlefs-checker-tier --bin e142_new_pool_file_creation_write_dump	a_failure_after_make_filesystem_stays_an_implementation_failure
````

## 实现员 x2：crates.patch（/tmp/claude-1000/m3-batch20/x2/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs b/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs
index da43591..317db33 100644
--- a/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs
+++ b/crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs
@@ -1,7 +1,8 @@
 //! 里程碑三·6（`.claude/kb/milestone/03-third-txn.md` 第三节「三·6」）干跑的 `crates/` 侧只读导出：删除写出的字节。
 //!
-//! 在两块内存盘上依次跑 `singlefs_harness::scenario::run_deletion_history` 的三段历史（H1、H2、H3：E142 那套参数、
-//! 三个数据单元的第一个文件、只供测试的批大小 1；H3 在顺序写之前多建一个 size 0 的 2 号 inode、意图的调用方时间晚顺序写 60 秒，
+//! 在两块内存盘上依次跑 `singlefs_harness::scenario::run_deletion_history` 的几段历史（E142 那套参数、
+//! 三个数据单元的第一个文件、只供测试的批大小 1；不带参数跑 H1、H2，`--history <名>` 只跑点名的那一段；
+//! H3 在顺序写之前多建一个 size 0 的 2 号 inode、意图的调用方时间晚顺序写 60 秒，
 //! 它的 `name=impl_config` 多打一格 `intent_write_time_seconds=`，E172 跑前登记第五节 5.3），把 unlink 之后的窗口（意图那次发布之前录制流的长度起，到这段历史结束为止）
 //! 里**每一次写**按（步号、设备、字节偏移、字节长度、种类、sha256、整段十六进制）原样打出来，屏障各打一行，
 //! 每一段（一次发布，或不属于任何一次发布的几次写）前后各打一行边界：发布的边界带 txg、哪一种发布、这次重写的角色。
@@ -11,9 +12,12 @@
 //!
 //! 只驱动、只观测：不改 `singlefs-core`、`singlefs-checker`、`singlefs-format`。
 //!
-//!   deletion_write_dump
+//!   deletion_write_dump                    导 H1、H2（E172 第一段那一份，`research/scripts/replay.sh` 里 E172 那一行逐字节比它）
+//!   deletion_write_dump --history <名>     只导点名的那一段，名字是 h1、h2、h3 之一（E172 第三段用 `--history h3`）
 //!
-//! 不收参数、不读环境、不碰真设备、不写任何文件。退出码：0 = 正常；1 = 命令行给了参数。
+//! H3 不进不带参数的导出：进了，E172 第一段那一份与它的复跑就对不上了。
+//! 不读环境、不碰真设备、不写任何文件。退出码：0 = 正常；1 = 命令行不对（不认识的参数、`--history` 后面缺名字、
+//! 名字不是 h1 / h2 / h3、名字后面还有参数），stderr 打原因与用法，一行都不导；101 = 某段历史在内存盘上没走完（panic 带原因）。
 
 use singlefs_core::address::DeviceIdentity;
 use singlefs_core::block_device::PhysicalBlockSizeInBytes;
@@ -34,6 +38,68 @@ const DEVICE_COUNT: u32 = 2;
 const PHYSICAL_BLOCK_SIZE_IN_BYTES: u32 = 512;
 const MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;
 
+/// 点名一段历史的参数，后面跟 `DeletionHistory::name()` 那个名字。
+const HISTORY_OPTION: &str = "--history";
+
+/// 不带参数时导的历史：E172 第一段那一份导出就是这两段。
+const HISTORIES_DUMPED_WITHOUT_ARGUMENTS: [DeletionHistory; 2] = [
+    DeletionHistory::IntentAndEveryBatchInProcess,
+    DeletionHistory::IntentThenUnmountThenMountContinuation,
+];
+
+/// 命令行不对时的退出码（文件头「退出码」那一句）。
+const EXIT_CODE_OF_A_REFUSED_COMMAND_LINE: i32 = 1;
+
+/// 这一次导哪几段历史。
+#[derive(Debug, PartialEq, Eq)]
+enum DumpedHistories {
+    /// 不带参数：[`HISTORIES_DUMPED_WITHOUT_ARGUMENTS`]。
+    HistoriesOfTheE172FirstSegment,
+    /// `--history <名>`：只导这一段。
+    OneNamedHistory(DeletionHistory),
+}
+
+impl DumpedHistories {
+    fn histories_in_dump_order(&self) -> Vec<DeletionHistory> {
+        match self {
+            DumpedHistories::HistoriesOfTheE172FirstSegment => {
+                HISTORIES_DUMPED_WITHOUT_ARGUMENTS.to_vec()
+            }
+            DumpedHistories::OneNamedHistory(history) => vec![*history],
+        }
+    }
+}
+
+/// 命令行哪里不对。
+#[derive(Debug, PartialEq, Eq)]
+enum CommandLineRefusal {
+    /// 第一个参数不是 [`HISTORY_OPTION`]。
+    UnknownArgument(String),
+    /// [`HISTORY_OPTION`] 后面没有名字。
+    HistoryOptionWithoutAName,
+    /// 名字不是任何一段历史的 `DeletionHistory::name()`。
+    UnknownHistoryName(String),
+    /// 名字后面还有参数。
+    ArgumentsAfterTheHistoryName(Vec<String>),
+}
+
+/// 把命令行（不含程序名）解成这一次导哪几段历史。
+fn parse_dumped_histories(arguments: &[String]) -> Result<DumpedHistories, CommandLineRefusal> {
+    match arguments {
+        [] => Ok(DumpedHistories::HistoriesOfTheE172FirstSegment),
+        [option] if option == HISTORY_OPTION => Err(CommandLineRefusal::HistoryOptionWithoutAName),
+        [option, history_name] if option == HISTORY_OPTION => DeletionHistory::EVERY_HISTORY
+            .into_iter()
+            .find(|history| history.name() == history_name)
+            .map(DumpedHistories::OneNamedHistory)
+            .ok_or_else(|| CommandLineRefusal::UnknownHistoryName(history_name.clone())),
+        [option, _history_name, arguments_after_the_name @ ..] if option == HISTORY_OPTION => Err(
+            CommandLineRefusal::ArgumentsAfterTheHistoryName(arguments_after_the_name.to_vec()),
+        ),
+        [first_argument, ..] => Err(CommandLineRefusal::UnknownArgument(first_argument.clone())),
+    }
+}
+
 struct Emitter {
     emitted: u64,
 }
@@ -205,13 +271,116 @@ fn dump_one_history(history: DeletionHistory, emitter: &mut Emitter) {
 
 fn main() {
     let arguments: Vec<String> = std::env::args().skip(1).collect();
-    if !arguments.is_empty() {
-        eprintln!("deletion_write_dump 不收参数，收到 {arguments:?}");
-        std::process::exit(1);
-    }
+    let dumped_histories = parse_dumped_histories(&arguments).unwrap_or_else(|refusal| {
+        let history_names = DeletionHistory::EVERY_HISTORY
+            .map(DeletionHistory::name)
+            .join(" | ");
+        eprintln!(
+            "deletion_write_dump 命令行不对：{refusal:?}（收到 {arguments:?}）；用法：不带参数（导 h1、h2）或 {HISTORY_OPTION} <{history_names}>"
+        );
+        std::process::exit(EXIT_CODE_OF_A_REFUSED_COMMAND_LINE)
+    });
     let mut emitter = Emitter { emitted: 0 };
-    for history in DeletionHistory::EVERY_HISTORY {
+    for history in dumped_histories.histories_in_dump_order() {
         dump_one_history(history, &mut emitter);
     }
     emitter.finish();
 }
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn command_line(arguments: &[&str]) -> Vec<String> {
+        arguments
+            .iter()
+            .map(|argument| (*argument).to_string())
+            .collect()
+    }
+
+    #[test]
+    fn no_arguments_dump_h1_then_h2_and_leave_h3_out() {
+        let dumped_histories = parse_dumped_histories(&[]).expect("不带参数是认得的命令行");
+        assert_eq!(
+            dumped_histories.histories_in_dump_order(),
+            vec![
+                DeletionHistory::IntentAndEveryBatchInProcess,
+                DeletionHistory::IntentThenUnmountThenMountContinuation,
+            ],
+            "不带参数导 H1、H2（这个次序），不导 H3：E172 第一段那一份导出与它的逐字节复跑就是这两段"
+        );
+    }
+
+    #[test]
+    fn the_history_option_with_h3_dumps_only_h3() {
+        let dumped_histories = parse_dumped_histories(&command_line(&["--history", "h3"]))
+            .expect("`--history h3` 是认得的命令行");
+        assert_eq!(
+            dumped_histories.histories_in_dump_order(),
+            vec![DeletionHistory::LaterIntentTimeBesideASizeZeroInodeAndEveryBatchInProcess],
+            "`--history h3` 只导 H3 一段"
+        );
+    }
+
+    #[test]
+    fn the_history_option_dumps_exactly_the_history_whose_name_it_gives() {
+        for history in DeletionHistory::EVERY_HISTORY {
+            assert_eq!(
+                parse_dumped_histories(&command_line(&["--history", history.name()])),
+                Ok(DumpedHistories::OneNamedHistory(history)),
+                "`--history {}` 只点名那一段",
+                history.name()
+            );
+        }
+    }
+
+    #[test]
+    fn an_unknown_first_argument_is_refused() {
+        assert_eq!(
+            parse_dumped_histories(&command_line(&["h3"])),
+            Err(CommandLineRefusal::UnknownArgument("h3".to_string())),
+            "名字前面没有 `--history` 不认"
+        );
+        assert_eq!(
+            parse_dumped_histories(&command_line(&["--history=h3"])),
+            Err(CommandLineRefusal::UnknownArgument(
+                "--history=h3".to_string()
+            )),
+            "`--history=<名>` 这种写法不认"
+        );
+    }
+
+    #[test]
+    fn the_history_option_without_a_name_is_refused() {
+        assert_eq!(
+            parse_dumped_histories(&command_line(&["--history"])),
+            Err(CommandLineRefusal::HistoryOptionWithoutAName),
+            "`--history` 后面缺名字不认"
+        );
+    }
+
+    #[test]
+    fn a_history_name_that_is_not_h1_h2_or_h3_is_refused() {
+        assert_eq!(
+            parse_dumped_histories(&command_line(&["--history", "h4"])),
+            Err(CommandLineRefusal::UnknownHistoryName("h4".to_string())),
+            "h4 不是任何一段历史的名字"
+        );
+        assert_eq!(
+            parse_dumped_histories(&command_line(&["--history", "H3"])),
+            Err(CommandLineRefusal::UnknownHistoryName("H3".to_string())),
+            "名字按 `DeletionHistory::name()` 原样比，大写不认"
+        );
+    }
+
+    #[test]
+    fn arguments_after_the_history_name_are_refused() {
+        assert_eq!(
+            parse_dumped_histories(&command_line(&["--history", "h3", "h1"])),
+            Err(CommandLineRefusal::ArgumentsAfterTheHistoryName(vec![
+                "h1".to_string()
+            ])),
+            "一次只点名一段，名字后面多出的参数不认"
+        );
+    }
+}
````

### x2：mutations-append.tsv

````
三·6 删除导出参数：不带参数也导 H3（E172 第一段那一份与它的逐字节复跑对不上）	crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs	                HISTORIES_DUMPED_WITHOUT_ARGUMENTS.to_vec()	                DeletionHistory::EVERY_HISTORY.to_vec()	-p singlefs-checker-tier --bin deletion_write_dump	no_arguments_dump_h1_then_h2_and_leave_h3_out
三·6 删除导出参数：--history 按名字找错了那一段（--history h3 导出的不是 H3）	crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs	            .find(|history| history.name() == history_name)	            .find(|history| history.name() != history_name)	-p singlefs-checker-tier --bin deletion_write_dump	the_history_option_with_h3_dumps_only_h3
三·6 删除导出参数：不认识的第一个参数照不带参数导 H1、H2（不退 1）	crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs	        [first_argument, ..] => Err(CommandLineRefusal::UnknownArgument(first_argument.clone())),	        [_first_argument, ..] => Ok(DumpedHistories::HistoriesOfTheE172FirstSegment),	-p singlefs-checker-tier --bin deletion_write_dump	an_unknown_first_argument_is_refused
三·6 删除导出参数：--history 后面缺名字照不带参数导 H1、H2（不退 1）	crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs	        [option] if option == HISTORY_OPTION => Err(CommandLineRefusal::HistoryOptionWithoutAName),	        [option] if option == HISTORY_OPTION => Ok(DumpedHistories::HistoriesOfTheE172FirstSegment),	-p singlefs-checker-tier --bin deletion_write_dump	the_history_option_without_a_name_is_refused
三·6 删除导出参数：不认识的历史名照不带参数导 H1、H2（不退 1）	crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs	            .ok_or_else(|| CommandLineRefusal::UnknownHistoryName(history_name.clone())),	            .or(Some(DumpedHistories::HistoriesOfTheE172FirstSegment))\n            .ok_or_else(|| CommandLineRefusal::UnknownHistoryName(history_name.clone())),	-p singlefs-checker-tier --bin deletion_write_dump	a_history_name_that_is_not_h1_h2_or_h3_is_refused
三·6 删除导出参数：历史名后面多出的参数不管（不退 1）	crates/singlefs-checker-tier/src/bin/deletion_write_dump.rs	        [option, history_name] if option == HISTORY_OPTION => DeletionHistory::EVERY_HISTORY	        [option, history_name, ..] if option == HISTORY_OPTION => DeletionHistory::EVERY_HISTORY	-p singlefs-checker-tier --bin deletion_write_dump	arguments_after_the_history_name_are_refused
````

### x2：mutations-delete.txt

````
三·6 删除导出：写清零的种类名打成 write
三·6 删除导出：屏障也重建出（空的）字节
````

