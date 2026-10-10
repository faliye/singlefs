# 删除批第 21 批合入的改动原样（2026-10-05）

## 实现员 a：crates.patch（/tmp/claude-1000/m3-batch21/a/patch/crates.patch）

````diff
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -7940,6 +7940,7 @@
     TreeTableEntryShorterThanTheFieldTable,
     DeletedInodesHeadIdentifierNamesNoSingleInodeTree,
     InodeTreeNamedByNoSingleDeletedInodesEntry,
+    ExtentTreeHeadIdentifierNamesNoSingleInodeTree,
     DeletedInodesRootUnreadable,
     InodeTreeUnreadable,
     ExtentTreeOfTheInodeTreeNotSingle,
@@ -7963,6 +7964,9 @@
             DeletionScanGap::InodeTreeNamedByNoSingleDeletedInodesEntry => {
                 "走读到的某一版树表里有一棵 inode 树，同一张表里偏移 116 的头 ID 指着它的 deleted_inodes 条目不是恰好一条：那一版这棵树的队列与记录判不了"
             }
+            DeletionScanGap::ExtentTreeHeadIdentifierNamesNoSingleInodeTree => {
+                "走读到的某一版树表里有一棵 extent 树，偏移 116 的头 ID 配不上同一张表里恰好一条 inode 树条目：那一版的数据单元数判不了"
+            }
             DeletionScanGap::DeletedInodesRootUnreadable => {
                 "走读到的某一版 deleted_inodes 根读不出，或不是层级 0、key 宽与条目宽 8 的码 2 根：那一版的队列判不了"
             }
@@ -8083,7 +8087,9 @@
 
 /// inode 树一个内部节点下面的记录逐条收进 `records`：类型段 2 的叶容器（码 3 打包记录类型 2，记录宽 140）读它的每条记录，
 /// 类型段 0 的内部节点（层级 = 父层级 − 1，且 ≥ 1）往下走。读不出、形状不对的交回 [`DeletionScanGap::InodeTreeUnreadable`]。
-/// 同一个号出现两条时后读到的那条盖掉先读到的（号的唯一性由主走读的 I-9.4、I-1.1 判）。递归的上界：每下一层层级减一。
+/// 同一个号出现两条时后读到的那条盖掉先读到的。同一片叶里同号两条今天没有不变量管：I-9.4 只管记录号不小于容器号与跨容器的序，
+/// 容器里记录号严格递增没有哪一条判（代码轮判决 `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节 S4a，条款缺口，
+/// 归里程碑三·3 的设计轮那一问）。递归的上界：每下一层层级减一。
 fn collect_inode_records_below(
     reader: &dyn ImageReader,
     node: &crate::IndexNodeView,
@@ -8341,7 +8347,8 @@
 
 /// I-9.5（两个方向）、I-4.7（对象取丙）、I-4.6（取乙）在一版上判：这一版树表里每一条 `deleted_inodes` 条目经偏移 116 的头 ID 配同一张表里
 /// 树 ID 等于它的那一条 inode 树条目（判决第二节 F4-b；配不上、配到不止一条记判不了），extent 树同样按头 ID 配这棵 inode 树；
-/// 反方向也配：每一棵 inode 树都要恰好一条 `deleted_inodes` 条目指着它，不是恰好一条的记判不了。
+/// 反方向也配：每一棵 inode 树都要恰好一条 `deleted_inodes` 条目指着它，每一棵 extent 树的头 ID 都要配上恰好一条 inode 树，
+/// 不是恰好一条的记判不了。
 /// I-9.5 每一版每个方向各判一次（队列与 nlink = 0 的号集合两两比，干净的一版也真比过）；I-4.7 每个处于删除中的号判一次、
 /// I-4.6 每条 nlink = 0 的记录判一次（一个都没有时没有对象）。
 fn judge_deletion_queue_of_a_version(
@@ -8463,6 +8470,32 @@
                 );
             }
         }
+    }
+    // extent 树那一侧也反方向配（代码轮判决 `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节 S1、第三节第 1 条）：
+    // 上面那一遍只读头 ID 等于这棵 inode 树的 extent 树，头 ID 配不上 inode 树的 extent 树一次都不读、也不记判不了——一条空根条目
+    // 冒充这棵 inode 树的 extent 树、真 extent 树头 ID 指别处时，I-4.7、I-4.6 按零个单元判、报没有对象。这里每一棵 extent 树的头 ID
+    // 都要配上恰好一条 inode 树，配不上的记判不了（设计轮判决 F4-b「配不上记判不了」）。放在上面那一遍之后：没有 extent 树指着
+    // inode 树的那一形同时也是这一形，先记上面那一句更近的理由。I-9.5 不读 extent 树，不记。
+    for extent_tree_entry in tree_table_entries
+        .iter()
+        .filter(|entry| kind_of(entry) == TREE_KIND_EXTENT)
+    {
+        let inode_tree = head_identifier_of(&extent_tree_entry);
+        let inode_trees_named_by_the_extent_tree = tree_table_entries
+            .iter()
+            .filter(|entry| kind_of(entry) == TREE_KIND_INODE && read_u64(entry, 0) == inode_tree)
+            .count();
+        if inode_trees_named_by_the_extent_tree != 1 {
+            for invariant in [
+                INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+                DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+            ] {
+                judgements.part_of_the_range_not_judged(
+                    invariant,
+                    DeletionScanGap::ExtentTreeHeadIdentifierNamesNoSingleInodeTree.reason(),
+                );
+            }
+        }
     }
 }
 
--- a/crates/singlefs-harness/src/lib.rs
+++ b/crates/singlefs-harness/src/lib.rs
@@ -41,7 +41,10 @@
 }
 
 impl RecordedOperationKind {
-    /// 流文本（[`RecordedOperation::to_stream_line`] 的 `kind=`）与各导出装置结果行（`kind=`）里的种类名：全仓这一处定义，不各抄一份。
+    /// 流文本（[`RecordedOperation::to_stream_line`] 的 `kind=`）与各导出装置结果行（`kind=`）里的种类名。新写的地方都调它，不再抄一份；
+    /// checker 档里还有四份装置各自手抄着同样四个名字（`src/bin/` 下 `e170_o3_spec_executor_export.rs`、`e171_three_oracle_collusion.rs`、
+    /// `e171_three_oracle_collusion_after_the_oracle_fixes.rs`、`e142_new_pool_file_creation_write_dump_one_device.rs`；前三份另有一份
+    /// 按名字反查种类的），没挪过来，改名时那四份要跟着改。
     #[must_use]
     pub const fn name(self) -> &'static str {
         match self {
--- a/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs
+++ b/crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs
@@ -17,7 +17,8 @@
 use singlefs_core::checksum::crc32_castagnoli;
 use singlefs_core::pointer::NodePointer;
 use singlefs_core::records::{
-    TREE_KIND_ACCOUNTING, TREE_KIND_DELETED_INODES, TREE_KIND_INODE, TREE_KIND_TOMBSTONE,
+    TREE_KIND_ACCOUNTING, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE,
+    TREE_KIND_LIVELIST, TREE_KIND_TOMBSTONE,
 };
 use singlefs_core::transaction::{
     publish_new_inodes, publish_unlink_intent, publish_unlink_step, PoolWriter, TransactionOutput,
@@ -82,6 +83,9 @@
 const TREE_TABLE_ENTRY_BYTES: usize = 200;
 const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;
 const TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET: usize = 116;
+/// 树表条目里根指针 86 字节占 [14, 100)（`walk.rs` 读 `entry[14..100]`）。
+const TREE_TABLE_ENTRY_ROOT_POINTER_START: usize = 14;
+const TREE_TABLE_ENTRY_ROOT_POINTER_END: usize = 100;
 /// 种类码登记表（D8（核心索引结构） 已定项 9）里没有的一个码（攻方 L1a 把 `deleted_inodes` 那一条的种类 9 改成它）。
 const TREE_KIND_NOT_REGISTERED: u16 = 13;
 /// 码 2 节点头里条目宽 2 字节在 84 + 2k、最大 key 在 52 + k（D18（块里携带什么信息） 已定项 7、18）。
@@ -762,6 +766,52 @@
             );
         },
     );
+}
+
+/// S1（代码轮判决 `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节，攻方报告第二节 2.2）：`tree_table_slot` 那张树表里
+/// 根指针全零的 livelist 条目（种类 6）种类改成 1、头 ID 改成 inode 树的树 ID，冒充这棵 inode 树的 extent 树；真 extent 树那一条的头 ID 改成 0。
+/// 树 ID、根指针都不动。只从 inode 树一侧按头 ID 配 extent 树的话，配上的是那条空根，真 extent 树一次都不读。
+pub fn let_the_empty_livelist_entry_stand_in_for_the_extent_tree_in_the_tree_table_at(
+    image: &mut MemoryPool,
+    tree_table_slot: SlotNumber,
+) {
+    change_the_unit_and_propagate_its_checksum(
+        image,
+        tree_table_slot,
+        node_bytes(),
+        |tree_table| {
+            let inode_tree_index =
+                index_of_the_only_tree_table_entry_of_kind(tree_table, TREE_KIND_INODE);
+            let inode_tree = get_u64(tree_table, tree_table_entry_start(inode_tree_index));
+            let extent_tree_index =
+                index_of_the_only_tree_table_entry_of_kind(tree_table, TREE_KIND_EXTENT);
+            let livelist_index =
+                index_of_the_only_tree_table_entry_of_kind(tree_table, TREE_KIND_LIVELIST);
+            let livelist_entry = tree_table_entry_start(livelist_index);
+            assert!(
+                tree_table[livelist_entry + TREE_TABLE_ENTRY_ROOT_POINTER_START
+                    ..livelist_entry + TREE_TABLE_ENTRY_ROOT_POINTER_END]
+                    .iter()
+                    .all(|byte| *byte == 0),
+                "livelist 条目的根指针全零（空根）"
+            );
+            set_u64(
+                tree_table,
+                tree_table_entry_start(extent_tree_index) + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
+                0,
+            );
+            set_u16(
+                tree_table,
+                livelist_entry + TREE_TABLE_ENTRY_KIND_OFFSET,
+                TREE_KIND_EXTENT,
+            );
+            set_u64(
+                tree_table,
+                livelist_entry + TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET,
+                inode_tree,
+            );
+        },
+    );
 }
 
 /// `tree_table_slot` 那张树表的条目宽收窄到 key 宽（8）：条目数改 1、声明长度改 8、最大 key 贴到剩下那一条的 key 上、
--- a/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
+++ b/crates/singlefs-harness/tests/pool_checker_judges_the_deletion_state_of_every_walked_version.rs
@@ -10,6 +10,7 @@
 
 use common::build_pool;
 use common_deletion_in_progress::{
+    accounting_tree_slot_of_the_newest_root,
     change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at,
     drop_the_pending_deletion_occupancy_row,
     drop_the_record_of_the_first_file_and_keep_its_data_units,
@@ -17,6 +18,7 @@
     first_tombstone_container_slot_of_the_newest_root,
     give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at,
     image_after_the_first_batch_of_a_three_unit_file, image_after_the_intent_of_a_three_unit_file,
+    let_the_empty_livelist_entry_stand_in_for_the_extent_tree_in_the_tree_table_at,
     move_the_tombstone_container_to_an_unpublished_instance_and_raise_its_object_identifier,
     narrow_the_entries_of_the_tree_table_at_to_the_key_width,
     pool_with_a_first_file_of_size_zero_and_one_data_unit, pool_with_a_three_unit_file,
@@ -83,7 +85,10 @@
     "deleted_inodes 条目偏移 116 的头 ID 配不上同一张树表里恰好一条 inode 树条目";
 const NOT_JUDGED_EXTENT_TREE_OF_THE_INODE_TREE_NOT_SINGLE: &str =
     "头 ID 指着这棵 inode 树的 extent 树条目不是恰好一条";
+const NOT_JUDGED_EXTENT_TREE_HEAD_IDENTIFIER_NAMES_NO_SINGLE_INODE_TREE: &str =
+    "有一棵 extent 树，偏移 116 的头 ID 配不上同一张表里恰好一条 inode 树条目";
 const NOT_JUDGED_EXTENT_TREE_UNREADABLE: &str = "extent 树读不全：那一版每个号的数据单元数判不了";
+const NOT_JUDGED_ACCOUNTING_TREE_UNREADABLE: &str = "记账树读不全";
 const NOT_JUDGED_DELETED_INODES_ROOT_UNREADABLE: &str = "deleted_inodes 根读不出";
 const NOT_JUDGED_TREE_TABLE_UNREADABLE: &str = "树表读不出";
 const NOT_JUDGED_TREE_TABLE_ENTRY_SHORTER_THAN_THE_FIELD_TABLE: &str =
@@ -610,6 +615,72 @@
     }
 }
 
+/// S1（代码轮判决 `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节，攻方报告第二节 2.2）：I-4.7 那一形坏镜像
+/// （1 号记录拿掉、单元留着、号不在队列）之上，最新那一版树表里空根的 livelist 条目改成种类 1、头 ID 指 inode 树，真 extent 树头 ID 改 0。
+/// 只从 inode 树一侧配 extent 树时配上的是那条空根，I-4.7 报「没有处于删除中的 inode」，而真 extent 树里 1 号还有一个单元；
+/// extent 树也反方向配之后，真 extent 树配不上 inode 树，I-4.7、I-4.6 在那一版判不了。判红的只有 I-9.16（种类 1 两条）与 I-9.27（头 ID 取值）。
+#[test]
+fn an_empty_entry_standing_in_for_the_extent_tree_while_the_real_extent_tree_names_no_inode_tree_leaves_the_data_unit_invariants_not_judged(
+) {
+    let clean =
+        pool_with_new_inodes("deletion-checker-s1-empty-entry-as-extent-tree", 1).memory_pool();
+    assert_the_base_image_is_clean(&clean, "建了一个空 inode 之后");
+    for invariant in [
+        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
+        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
+    ] {
+        let reason = reason_not_judged(&clean, invariant);
+        assert!(
+            !reason.contains(NOT_JUDGED_EXTENT_TREE_HEAD_IDENTIFIER_NAMES_NO_SINGLE_INODE_TREE),
+            "{invariant} 在合法镜像上照旧报没有对象，不报 extent 树配不上：{reason}"
+        );
+    }
+    let mut image = clean.clone();
+    drop_the_record_of_the_first_file_and_keep_its_data_units(&mut image);
+    let tree_table_slot = tree_table_slot_of_the_newest_root(&image);
+    let_the_empty_livelist_entry_stand_in_for_the_extent_tree_in_the_tree_table_at(
+        &mut image,
+        tree_table_slot,
+    );
+    assert_eq!(
+        violated_invariants(&image),
+        vec!["I-9.16", "I-9.27"],
+        "判红的只有树表种类与头 ID 取值那两条"
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
+            reason.contains(NOT_JUDGED_EXTENT_TREE_HEAD_IDENTIFIER_NAMES_NO_SINGLE_INODE_TREE),
+            "{invariant} 在真 extent 树配不上 inode 树的那一版判不了，不报没有对象：{reason}"
+        );
+    }
+}
+
+/// `DeletionScanGap::AccountingTreeUnreadable`：写完新池新建文件那一版记账树根（根兼叶）两份全零（不补引用链）。
+/// I-3.4 在那一版判不了：不判红、不报成立（更早的版本照判、成立，也压不过这一版的判不了）。
+#[test]
+fn an_accounting_tree_that_does_not_read_back_leaves_the_pending_deletion_invariant_not_judged() {
+    let mut image = build_pool("deletion-checker-accounting-root-zeroed").memory_pool();
+    let accounting_root_slot = accounting_tree_slot_of_the_newest_root(&image);
+    zero_both_copies_of_the_unit_at(&mut image, accounting_root_slot, node_bytes());
+    let reason = reason_not_judged(&image, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO);
+    assert!(
+        reason.contains(NOT_JUDGED_ACCOUNTING_TREE_UNREADABLE),
+        "I-3.4 在记账树读不全的那一版判不了、不判红：{reason}"
+    );
+}
+
 /// `DeletionScanGap::ExtentTreeUnreadable`：意图之后那一版 extent 树根两份全零（不补引用链）。I-9.5 照判、成立；I-4.7、I-4.6 判不了。
 #[test]
 fn an_extent_tree_that_does_not_read_back_leaves_the_data_unit_invariants_of_the_deletion_not_judged(
````

## 实现员 a：mutations-append.tsv（/tmp/claude-1000/m3-batch21/a/patch/mutations-append.tsv）

````text
第21批 extent 树不反方向配（S1：空根条目冒充 extent 树、真 extent 树头 ID 配不上 inode 树，I-4.7、I-4.6 报没有对象）	crates/singlefs-checker/src/walk.rs	        if inode_trees_named_by_the_extent_tree != 1 {	        if false && inode_trees_named_by_the_extent_tree != 1 {	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_empty_entry_standing_in_for_the_extent_tree_while_the_real_extent_tree_names_no_inode_tree_leaves_the_data_unit_invariants_not_judged
第21批 记账树根读不出时按零行判（AccountingTreeUnreadable 不记，I-3.4 按缺行误红）	crates/singlefs-checker/src/walk.rs	    let node = read_index_node_without_judging(reader, node_pointer, cache)\n        .ok_or(DeletionScanGap::AccountingTreeUnreadable)?;	    let Some(node) = read_index_node_without_judging(reader, node_pointer, cache) else {\n        return Ok(());\n    };	-p singlefs-harness --test pool_checker_judges_the_deletion_state_of_every_walked_version	an_accounting_tree_that_does_not_read_back_leaves_the_pending_deletion_invariant_not_judged
````

## 实现员 w：crates.patch（/tmp/claude-1000/m3-batch21/w/patch/crates.patch）

````diff
--- a/crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs
+++ b/crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs
@@ -1359,9 +1359,9 @@
         tally.first_ignored_violation
     );
     assert_eq!(tally.failed_states, 0);
-    // 实例 2 的根为最新的那 12 个状态：残留记录那一版（(1, 5)，根槽从没落盘、只由记录施加出来）自己的四个固定点单元
-    // 50261–50264 被写行发布 txg 6 释放进 defer、仍算已分配，环里没有一条根引用它们。2026-09-17 写这条用例时它们在
-    // I-3.1（已分配统计对得上） 上判红、差 65536 字节；2026-09-23 用户定候选 b（增补 2 收口表第 54 行）：checker 的候选集
+    // 实例 2 的根为最新的那 12 个状态：残留记录那一版（(1, 5)，根槽从没落盘、只由记录施加出来）自己的固定点单元
+    // （分配记录树 5 个节点 + 记账 + 映射 + 树表，每盘 8 槽）被写行发布 txg 6 释放进 defer、仍算已分配，环里没有一条根引用它们。
+    // 2026-09-17 写这条用例时那一版的固定点是四个单元（50261–50264），它们在 I-3.1（已分配统计对得上） 上判红、差 65536 字节；2026-09-23 用户定候选 b（增补 2 收口表第 54 行）：checker 的候选集
     // 补上「由记录施加出来、根槽从没落盘的那一版」（`singlefs_checker::walk` 的 `versions_applied_only_by_records`），
     // 那 12 个状态从此一条都不红。把那一版从候选集里拿掉，这一行在 I-3.1 上当场红回 12。
     assert_checker_and_record_checker_counts(&tally, &[]);
--- a/crates/singlefs-core/src/transaction.rs
+++ b/crates/singlefs-core/src/transaction.rs
@@ -276,6 +276,25 @@
             }
         }
     }
+
+    /// 没有上一版的那次发布（新池新建文件）从哪个号起发：树里只有 inode 1，下一个可用号是 2（字节表六那一行）。
+    /// 这一次没有上一版的水位可压，产品路径恒走 `FromThePreviousVersionsAccounting` 那一臂。
+    ///
+    /// # Panics
+    /// 开关装着（`PressedBackForTests`）：这次发布不读上一版的水位，压的号用不上，装着它走到这里是开关给错了（C479 的开关只给有上一版的发布）。
+    /// 在发布路径读它的那一处在任何落盘动作之前。
+    #[must_use]
+    pub fn watermark_of_a_publish_without_a_previous_version(self) -> u64 {
+        match self {
+            InodeNumberWatermarkSource::FromThePreviousVersionsAccounting => FIRST_INODE_NUMBER + 1,
+            InodeNumberWatermarkSource::PressedBackForTests(pressed_back) => panic!(
+                "开关把 inode 号水位压到 {}，而这次发布没有上一版（新池新建文件，水位恒 {}）：\
+                 开关只给有上一版的发布，装着它走到这里是开关给错了",
+                pressed_back.next_inode_number(),
+                FIRST_INODE_NUMBER + 1
+            ),
+        }
+    }
 }
 
 /// 一个池的写入口：几块盘 + mkfs 参数（几何与区域归属都从这里取）。
@@ -388,7 +407,8 @@
 
     /// 装上「这次发布从哪个 inode 号水位起算」这个只供测试的开关（[`InodeNumberWatermarkSource`]）。产品路径一处都不调它。
     /// 装着时这个写入口发的每一次有上一版的发布都从压回去的号起算（建 inode 从它起发号，记账里写它加这次新建的个数）；
-    /// 没有上一版的那次（新池新建文件，水位恒 2）不读上一版的水位，开关不管它。
+    /// 没有上一版的那次（新池新建文件，水位恒 2）不读上一版的水位：开关装着走到那一次当场 panic
+    /// （[`InodeNumberWatermarkSource::watermark_of_a_publish_without_a_previous_version`]），在任何落盘动作之前。
     pub fn set_inode_number_watermark_source(&mut self, source: InodeNumberWatermarkSource) {
         self.inode_number_watermark_source = source;
     }
@@ -2784,7 +2804,7 @@
     /// 只有一条记录的发布、从盘上重建出来的版本都为空。
     pub earlier_records_of_this_publish: Vec<WrittenJournalRecord>,
     /// 这一版全部角色的单元：这次重写的是新装的，没重写的从上一版照抄（文件 / 固定点角色按 bump 次序——每个数据单元、extent 根、
-    /// 每片 inode 叶容器、inode 根、四个固定点单元——实例表链的各片在末尾（按链上的次序，第 0 片在前）、mkfs 之后第一次重写之前不在）。
+    /// 每片 inode 叶容器、inode 根、固定点那几族（分配记录树、记账树、中央映射树各连同根之下的节点，加树表单元）——实例表链的各片在末尾（按链上的次序，第 0 片在前）、mkfs 之后第一次重写之前不在）。
     pub units: Vec<PublishedUnit>,
     /// 这次发布真正写出的角色，按写出的次序（点名项与录制流里的单元写只有这些）。
     pub rewritten: Vec<TransactionUnit>,
@@ -4325,7 +4345,7 @@
     }
 
     /// 暖机那几次空发布的形状：不写文件内容、不碰 inode 树、实例表照抄（`publish_empty_after` 交给 `publish_version` 的那张计划同形，
-    /// 那里有一条断言钉住两者相等）。记账树已经存在 ⇒ 照样重写四个固定点单元（D16（发布语义） 已定项 9）。
+    /// 那里有一条断言钉住两者相等）。记账树已经存在 ⇒ 照样重写固定点单元（D16（发布语义） 已定项 9；新池几何下分配记录树 5 个节点 + 记账 + 映射 + 树表 = 8 个，字节表八）。
     pub const EMPTY_PUBLISH: PublishShape = PublishShape {
         rewritten_data_units: 0,
         rewritten_inode_leaf_containers: 0,
@@ -8008,8 +8028,10 @@
         Some(previous_version) => pool
             .inode_number_watermark_source()
             .watermark_this_publish_starts_from(previous_version.inode_number_watermark()),
-        // 新池新建文件：树里只有 inode 1，下一个可用号是 2（字节表六那一行）。
-        None => FIRST_INODE_NUMBER + 1,
+        // 新池新建文件：树里只有 inode 1，下一个可用号是 2（字节表六那一行）；开关装着走到这里当场 panic（开关给错了）。
+        None => pool
+            .inode_number_watermark_source()
+            .watermark_of_a_publish_without_a_previous_version(),
     } + new_inodes_of_this_publish;
     let device_identities_of_the_accounting_rows: Vec<DeviceIdentity> = allocator
         .devices
@@ -9293,7 +9315,7 @@
         );
     }
 
-    /// 一次发布的重写角色：`data_units` 个数据单元、extent 根、`leaf_containers` 片 inode 叶容器、inode 根与四个固定点单元，按 bump 次序。
+    /// 一次发布的重写角色：`data_units` 个数据单元、extent 根、`leaf_containers` 片 inode 叶容器、inode 根与四个固定点角色的根（分配记录树、记账树、映射树、树表；根之下的节点不在这里），按 bump 次序。
     fn rewritten_roles(data_units: u64, leaf_containers: usize) -> Vec<TransactionUnit> {
         (0..data_units)
             .map(|index| TransactionUnit::Data(DataUnitIndexInFile(index)))
@@ -9431,4 +9453,26 @@
         );
         let _ = not_pressed_back.watermark_this_publish_starts_from(5);
     }
+
+    #[test]
+    fn a_publish_without_a_previous_version_starts_the_inode_number_watermark_at_two_on_the_product_path(
+    ) {
+        assert_eq!(
+            InodeNumberWatermarkSource::FromThePreviousVersionsAccounting
+                .watermark_of_a_publish_without_a_previous_version(),
+            2,
+            "新池新建文件：树里只有 inode 1，下一个可用号是 2（字节表六那一行）"
+        );
+    }
+
+    #[test]
+    #[should_panic(expected = "开关只给有上一版的发布，装着它走到这里是开关给错了")]
+    fn a_pressed_back_inode_number_watermark_on_a_publish_without_a_previous_version_is_a_misused_switch(
+    ) {
+        let pressed_back = InodeNumberWatermarkSource::PressedBackForTests(
+            InodeNumberWatermarkPressedBackForTests::new(FIRST_INODE_NUMBER)
+                .expect("第一个 inode 号压得到"),
+        );
+        let _ = pressed_back.watermark_of_a_publish_without_a_previous_version();
+    }
 }
--- a/crates/singlefs-harness/src/history.rs
+++ b/crates/singlefs-harness/src/history.rs
@@ -1691,7 +1691,7 @@
     NoWritableSession,
     /// 现行版本树表 0 条：覆盖写与抬 F 都要一个带文件的上一版（`TransactionOutput`）。
     CurrentVersionWithoutFile,
-    /// 现行版本带文件：零单元发布只给树表 0 条的一版；带文件的一版上的空发布要重写四个固定点单元，那是挂载内部的 `publish_version`，
+    /// 现行版本带文件：零单元发布只给树表 0 条的一版；带文件的一版上的空发布要重写固定点单元（分配记录树要改的那几个节点 + 记账 + 映射 + 树表），那是挂载内部的 `publish_version`，
     /// 不是这个入口。生成器的前提，没有条款明写调用约定，按入口的文档注释取的读法——`crates/singlefs-core/src/transaction.rs` 第 506 行
     /// 「零单元发布（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）」，已定项 9 在 `.claude/kb/decisions/16-发布语义.md` 第 192 行。
     /// 以后要测：前提之外调它应当返回 `Err`，不许 panic（增补 3 记着，2026-09-18 主 agent 定）。
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -143,17 +143,18 @@
 /// 一次发布重写哪些角色。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum ModelPublishKind {
-    /// 第一个文件版本：四个文件角色，加四个固定点单元（D16（发布语义） 已定项 9：记账行每发布重写，连带记账树节点、分配记录、
-    /// 映射条目与树表单元）。与覆盖写重写的角色相同，分开只为一件事——它走 `previous: None`，这一版的内存态从头建。
+    /// 第一个文件版本：四个文件角色，加固定点单元——四个固定点角色（分配记录树、记账树、映射树、树表），分配记录树按位置寻址
+    /// 一次重写根与根之下要改的那几个节点（D8（核心索引结构） 已定项 14；新池几何下 5 个，固定点合计 8 个单元，字节表八），
+    /// D16（发布语义） 已定项 9：记账行每发布重写，连带记账树节点、分配记录、映射条目与树表单元。与覆盖写重写的角色相同，分开只为一件事——它走 `previous: None`，这一版的内存态从头建。
     FirstFileVersion,
     /// 整份重写文件内容：覆盖写（一个数据单元以内）与顺序写（跨几个数据单元，`transaction::publish_sequential_write`）。
     /// 与第一个文件版本重写的角色相同，接在上一版的内存态后面；内容的每个数据单元都在这次写（同一个分配代），
     /// 多于一个单元时 extent 树下段的节点（叶罩 144 个单元、内部扇出 147，D8（核心索引结构） 已定项 14）也在这次写，
     /// 一个单元一个事务（D16（发布语义） 已定项 5 的切分纪律）。
     OverwriteFileVersion,
-    /// 写行：实例表单元（D18（块里携带什么信息） 已定项 11：每次可写挂载都写行）加四个固定点单元（D16（发布语义） 已定项 9）。
+    /// 写行：实例表单元（D18（块里携带什么信息） 已定项 11：每次可写挂载都写行）加固定点单元（同第一个文件版本那几族，D16（发布语义） 已定项 9）。
     RowsOnFileVersion,
-    /// 树表不是 0 条时的空发布（暖机、抬 F）：四个固定点单元（D16（发布语义） 已定项 9）。
+    /// 树表不是 0 条时的空发布（暖机、抬 F）：固定点单元（同第一个文件版本那几族，D16（发布语义） 已定项 9）。
     EmptyOnFileVersion,
     /// 树表 0 条的一版上写行：只重写实例表这一个单元（D18（块里携带什么信息） 已定项 11 要求每次可写挂载都写行；
     /// 这一版没有记账树，D16（发布语义） 已定项 9 的那五样——记账行、记账树节点、分配记录、映射条目、树表单元——一样都不写）。
@@ -161,20 +162,20 @@
     /// 树表 0 条上的空发布：零单元（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）。
     ZeroUnit,
     /// unlink 的意图那次（D8（核心索引结构） 已定项 16 第 1 条）：inode 叶与根（nlink 写 0）、`deleted_inodes` 根（加 key）、
-    /// 墓碑树开放容器与根（追加一条墓碑），加四个固定点单元；数据单元与 extent 树照抄。
+    /// 墓碑树开放容器与根（追加一条墓碑），加固定点单元；数据单元与 extent 树照抄。
     UnlinkIntent,
     /// 不是最后一批的删除批（D8（核心索引结构） 已定项 16 第 2 条：剩下的单元多于产品批大小 144 时从尾部释放 144 个）：
     /// 留下的数据单元照抄（分配代不变）；重写被截掉那一片下段叶与它的祖先（下段节点不对应角色，占槽按下段层数取上界）、
-    /// extent 上段根、inode 叶与根（记录的 size 改成剩下单元数 × 净荷容量，nlink 仍 0），加四个固定点单元；
+    /// extent 上段根、inode 叶与根（记录的 size 改成剩下单元数 × 净荷容量，nlink 仍 0），加固定点单元；
     /// `deleted_inodes` 那条 key 与墓碑树不动。这一版仍是删除中的那一版。
     DeletionBatchKeepingDataUnits,
     /// 最后一批（D8（核心索引结构） 已定项 16 第 2 条）：剩下的单元不多于产品批大小 144 时意图之后（或上一批之后）的下一步——释放剩下的单元、
     /// 删 inode 记录（树里只有 inode 1，叶容器一片都不剩）、删 `deleted_inodes` 那条 key（删空取根指针置零、不写它的节点），
-    /// 同一次发布；重写 extent 零条目根与 inode 零条目根，加四个固定点单元。
+    /// 同一次发布；重写 extent 零条目根与 inode 零条目根，加固定点单元。
     FinalDeletionBatch,
     /// 建一个 size 0 的 inode（`transaction::publish_new_inodes`，count 1；D8（核心索引结构） 已定项 16 第 5 条：删完之后只许建 inode、
     /// 写行与空发布），新记录落在最左那片叶容器里（删完之后建的前 233 条，[`inode_leaf_container_position_of_a_new_record`]）：
-    /// inode 叶与根，加四个固定点单元；数据单元与 extent 树根照抄（新建的 inode 长度 0）。
+    /// inode 叶与根，加固定点单元；数据单元与 extent 树根照抄（新建的 inode 长度 0）。
     /// 执行器只在第一个文件已删完的一版上调（`history.rs` 的 `HistoryOperation::CreateInode`）。
     NewInode,
     /// 同 [`ModelPublishKind::NewInode`]，新记录落在最左那片之外的那一片里（删完之后建的第 234 条起）：只重写那一片与 inode 根，
@@ -2525,7 +2526,7 @@
     ) -> Result<ModelAnswer, ModelDisagreement> {
         let session = self.open_session()?;
         let current = &session.current;
-        // 抬 F 的空发布要重写四个固定点单元，接在有树的一版后面：执行器只在带文件（含删除中）的一版上调，
+        // 抬 F 的空发布要重写固定点单元，接在有树的一版后面：执行器只在带文件（含删除中）的一版上调，
         // 挂载与会话为空间推的那几串还会接在有树而文件集合为空的一版后面。
         if !current.contents.has_trees() {
             return Err(ModelDisagreement::new(
@@ -2574,7 +2575,7 @@
     }
 
     /// 抬 F 那一串（D16（发布语义） 已定项 1「抬 F 那一串」：准入与卸载共用）接在会话现行那一版后面写出的根：带 `new_floor` 的空发布
-    /// （有树的一版上重写四个固定点单元），推到每块盘上都有一条为止（「生效」那一行），至多 R 次。
+    /// （有树的一版上重写固定点单元），推到每块盘上都有一条为止（「生效」那一行），至多 R 次。
     /// 先写系统配置那一步不属于任何一次发布（已定项 7），模型不建系统配置，这里只有根。
     fn empty_publishes_carrying_the_floor_to_every_device(
         &self,
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -4784,9 +4784,9 @@
 }
 
 /// 增补 2 收口表第 54 行（2026-09-23 用户定候选 b）的两半，建在同一份镜像上：
-/// ① **误报消失**：可写挂载接在「由记录施加出来、根槽从没落盘」的那一版之后，那一版被换下的四个固定点单元在 defer 里、仍算已分配，
+/// ① **误报消失**：可写挂载接在「由记录施加出来、根槽从没落盘」的那一版之后，那一版被换下的固定点 8 个单元（分配记录树 5 个节点 + 记账 + 映射 + 树表）在 defer 里、仍算已分配，
 ///    根环里没有一条根引用它们；checker 把那一版并进遍历之后 I-3.1（已分配统计对得上） 真被评估过且成立，别的也一条不红。
-///    候选集不补这一版（`versions_applied_only_by_records` 交回空），这一段就在 I-3.1 上红、差 65 536 字节（每盘 4 槽）。
+///    候选集不补这一版（`versions_applied_only_by_records` 交回空），这一段就在 I-3.1 上红、盘 0 上差 131 072 字节（8 槽；2026-10-05 在 `crates/mutations.tsv` 里 `take(0)` 那条变异下量的）。
 /// ② **改完仍会红**：同一份镜像上记账盘 0 再泄漏一槽（「已分配」+16384、「空闲」−16384），I-3.1 照样红
 ///    （I-3.11（已分配减 defer 等于最新根走读） 跟着红：defer 行没动），别的不红，
 ///    说明文字里的机理标识报出那一版确实并进了遍历——补上的只是等式右边缺的那一版，没把泄漏一起吞掉。
--- a/crates/singlefs-harness/tests/many_inodes_write_and_read.rs
+++ b/crates/singlefs-harness/tests/many_inodes_write_and_read.rs
@@ -154,7 +154,7 @@
         "右半出生树 = 执行这次分裂的那棵树"
     );
 
-    // ③ 这次重写了哪些角色：两片叶容器 + inode 根 + 四个固定点单元，加分配记录树按位置寻址（D8（核心索引结构） 已定项 14）
+    // ③ 这次重写了哪些角色：两片叶容器 + inode 根 + 四个固定点角色的根（分配记录树、记账、映射、树表），加分配记录树按位置寻址（D8（核心索引结构） 已定项 14）
     // 重写的根之下那几个节点（4 GiB 两块盘上根在第 2 层，改的记录都在两块盘各自的叶 61 里，先叶后根）；数据单元与 extent 树根照抄。
     assert_eq!(
         after.rewritten,
@@ -313,7 +313,7 @@
     }
 }
 
-/// 一次只建一个 inode、离 233 还远：树里仍只有一片叶容器，重写的也只有它与根 + 四个固定点单元（加分配记录树根之下那几个节点）。
+/// 一次只建一个 inode、离 233 还远：树里仍只有一片叶容器，重写的也只有它与根 + 四个固定点角色的根（分配记录树、记账、映射、树表；加分配记录树根之下那几个节点）。
 /// 这一档钉住「没跨过门槛就不分裂」——分裂条件写成「≥ 233」或「> 0」时这条红。
 #[test]
 fn creating_one_inode_below_the_threshold_keeps_one_container_and_bumps_the_watermark_by_one() {
--- a/crates/singlefs-harness/tests/unlink_in_histories_judged_by_the_model.rs
+++ b/crates/singlefs-harness/tests/unlink_in_histories_judged_by_the_model.rs
@@ -18,7 +18,7 @@
 const FAST_TIER_OPERATIONS_PER_HISTORY: usize = 30;
 
 /// 从第一个文件起：unlink 第一个文件（意图那次 + 最后一批，模型逐次比两条根、删除那两棵树的分配代）、冷启动读回「没有文件」、
-/// 可写挂载（写行与暖机接在有树而文件集合为空的一版后面，重写四个固定点单元）、在它上面发第一个文件被拒（树表不是 0 条）、
+/// 可写挂载（写行与暖机接在有树而文件集合为空的一版后面，写行与暖机都重写固定点那几个单元：分配记录树要改的那几个节点 + 记账 + 映射 + 树表，写行另加实例表单元）、在它上面发第一个文件被拒（树表不是 0 条）、
 /// 再冷启动仍读回「没有文件」。每一步之后跑池级 checker，模型与实现每一步都对得上：历史跑完，没有新发现。
 /// 判别力：模型删除那一臂改坏（最后一批不去掉数据单元与 `deleted_inodes` 根的角色、删后读回旧内容）时那一步判对不上、历史停在那里。
 #[test]
--- a/crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs
+++ b/crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs
@@ -133,7 +133,7 @@
         .expect("带设备维的行每盘一行")
 }
 
-/// 验收第一、二条：取号 2；行 (1, 4, 0)；写行发布 txg 5 重写实例表 + 四个固定点单元、事务号 0、反向链 0；暖机两次落到两块盘；
+/// 验收第一、二条：取号 2；行 (1, 4, 0)；写行发布 txg 5 重写实例表 + 固定点 8 个单元（分配记录树 5 个节点 + 记账 + 映射 + 树表）、事务号 0、反向链 0；暖机两次落到两块盘；
 /// 发布 C 接在后面（txg 8、事务号 1、反向链接 txg 7 那条）；冷启动择 (2, 8) 读回第三次的内容；记账与分配器对得上；checker 全绿。
 #[test]
 fn remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version()
@@ -197,7 +197,7 @@
                 TransactionUnit::TreeTable,
             ])
             .collect::<Vec<_>>(),
-        "写行发布重写实例表 + 四个固定点单元（记账树已存在 ⇒ 空发布也重写，D16 已定项 9）+ 分配记录树根之下那四个节点"
+        "写行发布重写实例表 + 固定点 8 个单元（记账树已存在 ⇒ 空发布也重写，D16 已定项 9）：分配记录树根之下那四个节点、四个固定点角色的根（分配记录树、记账、映射、树表）"
     );
     assert_eq!(row.record.named.len(), 9, "点名的只有重写的九个");
     assert_eq!(
@@ -228,7 +228,7 @@
     assert_eq!(
         row.released.len(),
         9,
-        "写行发布释放 mkfs 的实例表单元 + B 的四个固定点单元与分配记录树根之下那四个节点"
+        "写行发布释放 mkfs 的实例表单元 + B 的固定点 8 个单元（四个固定点角色的根与分配记录树根之下那四个节点）"
     );
 
     assert_eq!(
@@ -264,7 +264,7 @@
         assert_eq!(
             publish.released.len(),
             8,
-            "每次暖机释放上一次的四个固定点单元与分配记录树根之下那四个节点"
+            "每次暖机释放上一次的固定点 8 个单元（四个固定点角色的根与分配记录树根之下那四个节点）"
         );
     }
     let current = mounted
@@ -324,7 +324,7 @@
     assert_eq!(
         third_publish.released.len(),
         12,
-        "释放 B 的四个文件单元 + txg 7 的四个固定点单元与分配记录树根之下那四个节点"
+        "释放 B 的四个文件单元 + txg 7 的固定点 8 个单元（四个固定点角色的根与分配记录树根之下那四个节点）"
     );
     assert_eq!(
         third_publish
--- a/crates/singlefs-harness/tests/write_accounting_per_structure.rs
+++ b/crates/singlefs-harness/tests/write_accounting_per_structure.rs
@@ -199,8 +199,8 @@
     expected_by_kind(&[])
 }
 
-/// 后续可写挂载写行那次发布：实例表单元 32768 + 分配记录、记账、映射、树表四个固定点单元各 16384（字节表八「第一次之后的可写挂载（写行）」），
-/// 分配记录树是五个节点。
+/// 后续可写挂载写行那次发布：实例表单元 32768 + 固定点 8 个单元各 16384——分配记录树 5 个节点 + 记账 + 映射 + 树表
+/// （字节表八「第一次之后的可写挂载（写行）」，每盘 9 个单元）。
 fn row_publish_by_kind() -> Vec<(WrittenStructureKind, WriteCallsAndBytes)> {
     expected_by_kind(&[
         (WrittenStructureKind::AllocationRecordTreeNode, 16384),
@@ -211,8 +211,8 @@
     ])
 }
 
-/// 后续可写挂载的暖机空发布：四个固定点单元各 16384（字节表八「空发布（暖机，后续可写挂载的实例，写 c_max 个固定点单元）」），
-/// 分配记录树是五个节点。
+/// 后续可写挂载的暖机空发布：固定点 8 个单元各 16384——分配记录树 5 个节点 + 记账 + 映射 + 树表
+/// （字节表八「空发布（暖机，后续可写挂载的实例，写 c_max 个固定点单元）」）。
 fn later_warm_up_publish_by_kind() -> Vec<(WrittenStructureKind, WriteCallsAndBytes)> {
     expected_by_kind(&[
         (WrittenStructureKind::AllocationRecordTreeNode, 16384),
@@ -453,13 +453,13 @@
     assert_eq!(
         row_publish.writes.total(),
         calls_and_bytes(23, 344_576),
-        "写行：实例表单元 + 四个固定点单元（分配记录树五个节点）× 2 盘 = 18 次 327 680 字节 + 记录、根槽、系统配置槽 5 次 16 896"
+        "写行：（实例表单元 + 固定点 8 个单元：分配记录树 5 个节点 + 记账 + 映射 + 树表）× 2 盘 = 18 次 327 680 字节 + 记录、根槽、系统配置槽 5 次 16 896"
     );
     for (warm_up_index, warm_up_publish) in warm_up_publishes.iter().enumerate() {
         assert_eq!(
             warm_up_publish.writes.total(),
             calls_and_bytes(21, 279_040),
-            "第 {} 次暖机：四个固定点单元（分配记录树五个节点）× 2 盘 = 16 次 262 144 字节 + 5 次 16 896",
+            "第 {} 次暖机：固定点 8 个单元（分配记录树 5 个节点 + 记账 + 映射 + 树表）× 2 盘 = 16 次 262 144 字节 + 5 次 16 896",
             warm_up_index + 1
         );
     }
--- /dev/null
+++ b/crates/singlefs-harness/tests/inode_number_watermark_switch_on_a_publish_without_a_previous_version.rs
@@ -0,0 +1,97 @@
+//! inode 号水位那个只供测试的开关（`PoolWriter::set_inode_number_watermark_source`，`.claude/kb/checks-owed.md` C479）
+//! 只给有上一版的发布：新池新建文件没有上一版、水位恒 2，开关装着走到那一次是开关给错了，当场 panic，
+//! 而且 panic 在任何落盘动作之前（录制流一步都不多）。产品路径不装开关，不受影响。
+
+mod common;
+
+use std::panic::{catch_unwind, AssertUnwindSafe};
+
+use common::{file_content, format_pool, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
+use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
+use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
+use singlefs_core::make_filesystem::{
+    root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
+};
+use singlefs_core::transaction::{
+    acquire_instance, publish_first_file, warm_up, FirstFile,
+    InodeNumberWatermarkPressedBackForTests, InodeNumberWatermarkSource, PoolWriter,
+};
+
+/// mkfs → 取号 → 暖机之后、新池新建文件之前装上压回去的水位，再发新池新建文件：panic 消息说开关给错了，
+/// 录制流在这次发布前后一样长（panic 在任何写与屏障之前）。
+#[test]
+fn a_pressed_back_inode_number_watermark_on_the_new_pool_file_creation_panics_before_any_write() {
+    let publish_parameters = parameters();
+    let mut formatted = format_pool("inode-number-watermark-switch-on-new-pool-file-creation");
+    let mut allocator = PoolAllocator::new(vec![
+        DeviceFreeMap::new(DeviceIdentity(0), IMAGE_BYTES),
+        DeviceFreeMap::new(DeviceIdentity(1), IMAGE_BYTES),
+    ]);
+    allocator.mark_format_time_units(
+        Placement {
+            slot: INSTANCE_TABLE_SLOT,
+            span: 2,
+        },
+        Placement {
+            slot: TREE_TABLE_GENESIS_SLOT,
+            span: 1,
+        },
+    );
+    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
+        &publish_parameters,
+        &formatted.genesis,
+    ));
+    let content = file_content();
+    let genesis_root = formatted.genesis.root;
+    let stream = formatted.stream.clone();
+    let devices = formatted.devices.as_mut().expect("镜像还开着");
+    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+    let instance = acquire_instance(&mut writer).expect("取号");
+    assert_eq!(instance, InstanceGeneration(1));
+    let warmed_up = warm_up(&mut writer, &genesis_root, instance).expect("暖机");
+    let pressed_back = InodeNumberWatermarkSource::PressedBackForTests(
+        InodeNumberWatermarkPressedBackForTests::new(2).expect("2 不小于第一个 inode 号"),
+    );
+    writer.set_inode_number_watermark_source(pressed_back);
+    assert_eq!(
+        writer.inode_number_watermark_source().branch_name(),
+        "inode_number_watermark=pressed_back_for_tests",
+        "运行时看得出开关装着"
+    );
+    let operations_before_the_publish = stream.operation_count();
+
+    let outcome = catch_unwind(AssertUnwindSafe(|| {
+        publish_first_file(
+            &mut writer,
+            &mut allocator,
+            warmed_up.roots.last().expect("暖机两代根"),
+            FirstFile {
+                content: &content,
+                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
+            },
+            instance,
+            &warmed_up.last_record_bytes,
+        )
+    }));
+
+    let panic_payload = match outcome {
+        Ok(published) => panic!(
+            "开关装着发新池新建文件应当当场 panic，却交回了 {:?}",
+            published.map(|output| output.root.checkpoint_txg)
+        ),
+        Err(payload) => payload,
+    };
+    let panic_message = panic_payload
+        .downcast_ref::<String>()
+        .cloned()
+        .expect("panic! 带格式参数，载荷是 String");
+    assert!(
+        panic_message.contains("开关只给有上一版的发布，装着它走到这里是开关给错了"),
+        "panic 说的是开关给错了：{panic_message}"
+    );
+    assert_eq!(
+        stream.operation_count(),
+        operations_before_the_publish,
+        "panic 在任何写与屏障之前：录制流一步都不多"
+    );
+}
````

## 实现员 w：mutations-append.tsv（/tmp/claude-1000/m3-batch21/w/patch/mutations-append.tsv）

````text
第 21 批 C479 新池新建文件那一支：开关装着也不 panic、照产品路径从 2 起号（方法里开关那一臂改成交回第三个参数）	crates/singlefs-core/src/transaction.rs	            InodeNumberWatermarkSource::PressedBackForTests(pressed_back) => panic!(	            InodeNumberWatermarkSource::PressedBackForTests(pressed_back) => (|_message: &str, _pressed_back_number: u64, product_watermark: u64| product_watermark)(	-p singlefs-core --lib	a_pressed_back_inode_number_watermark_on_a_publish_without_a_previous_version_is_a_misused_switch
第 21 批 C479 新池新建文件那一支：发布路径不读开关、照旧写死 2（开关装着发新池新建文件也不 panic）	crates/singlefs-core/src/transaction.rs	        None => pool\n            .inode_number_watermark_source()\n            .watermark_of_a_publish_without_a_previous_version(),	        None => FIRST_INODE_NUMBER + 1,	-p singlefs-harness --test inode_number_watermark_switch_on_a_publish_without_a_previous_version	a_pressed_back_inode_number_watermark_on_the_new_pool_file_creation_panics_before_any_write
第 21 批 C479 新池新建文件那一支：产品路径那一臂起号写成 1（少加一）	crates/singlefs-core/src/transaction.rs	            InodeNumberWatermarkSource::FromThePreviousVersionsAccounting => FIRST_INODE_NUMBER + 1,	            InodeNumberWatermarkSource::FromThePreviousVersionsAccounting => FIRST_INODE_NUMBER,	-p singlefs-core --lib	a_publish_without_a_previous_version_starts_the_inode_number_watermark_at_two_on_the_product_path
````

## 实现员 g：crates.patch（/tmp/claude-1000/m3-batch21/g/patch/crates.patch）

````diff
diff --git a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
index 2ef0afc..5aca3a7 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
@@ -2063,8 +2063,8 @@ fn judge_first_data_unit_of_the_versions_building_trees_from_the_mkfs_watermark(
 // 头 ID 指着这棵 inode 树的 extent 树不是恰好一条）在这里一格都不置位；CPU 按树表单元去重只扫一次，违例位是并集，这里不去重。
 // I-4.4 第一版报不适用（check_pool_image 恒报），内核不置它那一位。
 
-// inode 记录里 nlink（4 字节）与 size（8 字节）的偏移、inode 记录宽（walk.rs INODE_RECORD_NLINK_OFFSET / INODE_RECORD_SIZE_OFFSET，类型 2 登记 140）
-const DELETION_INODE_RECORD_NLINK_OFFSET: u32 = 36u;
+// inode 记录里 size（8 字节）的偏移、inode 记录宽（walk.rs INODE_RECORD_SIZE_OFFSET，类型 2 登记 140）；nlink 的偏移（walk.rs INODE_RECORD_NLINK_OFFSET）
+// 用共用那一半的 INODE_RECORD_NLINK_OFFSET（crash_judge_common.wgsl 拼在三个内核前面），不另定义
 const DELETION_INODE_RECORD_SIZE_OFFSET: u32 = 40u;
 const DELETION_PACKED_TYPE_INODE: u32 = 2u;
 const DELETION_INODE_RECORD_BYTES: u32 = 140u;
@@ -2195,7 +2195,7 @@ fn d_read_inode_records(root_pointer: Ptr) -> bool {
             }
             for (var r = 0u; r < container.record_count; r = r + 1u) {
                 let record = packed_record_offset(container, r);
-                if (!d_note_inode_record(rd64(container.base, record), rd32(container.base, record + DELETION_INODE_RECORD_NLINK_OFFSET),
+                if (!d_note_inode_record(rd64(container.base, record), rd32(container.base, record + INODE_RECORD_NLINK_OFFSET),
                     rd64(container.base, record + DELETION_INODE_RECORD_SIZE_OFFSET))) {
                     return false;
                 }
diff --git a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
index 306dd0f..ac511c8 100644
--- a/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
+++ b/crates/singlefs-checker-tier/src/crash_judge_common.wgsl
@@ -131,7 +131,8 @@ const CONTAINER_NUMBER_KEY_WIDTH: u32 = 8u;
 const CONTAINER_TREE_ENTRY_BYTES: u32 = 120u;
 const CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET: u32 = 34u;
 const FIRST_INODE_NUMBER: u32 = 1u;
-// inode 记录里 nlink 的偏移（4 字节，D8（核心索引结构） 已定项 6 字段表）与删除中的 inode 的 nlink（records.rs 的 NLINK_OF_AN_INODE_BEING_DELETED）
+// inode 记录里 nlink 的偏移（4 字节，D8（核心索引结构） 已定项 6 字段表；恢复内核找第一个文件与扫描内核的删除态扫描都读这一份）
+// 与删除中的 inode 的 nlink（records.rs 的 NLINK_OF_AN_INODE_BEING_DELETED）
 const INODE_RECORD_NLINK_OFFSET: u32 = 36u;
 const NLINK_OF_AN_INODE_BEING_DELETED: u32 = 0u;
 const MAGIC_SFSB: u32 = 0x42534653u;
@@ -2306,8 +2307,9 @@ struct ExtentRead {
     ok: bool,
     node_count: u32,
 };
-// 判一个 extent 树节点（extent_tree::ExtentTreeReading::read_and_judge_node，Every）
-fn judge_extent_node(n: Node, tree: vec2<u32>, p: Ptr, root: RootFields, fsid_low: vec2<u32>, expected_level: u32,
+// 判一个 extent 树节点（extent_tree::ExtentTreeReading::read_and_judge_node，Every）；is_the_root_of_the_tree 说它是不是整棵树的根
+// （上段的根，树表条目指着它；下段的根不算）
+fn judge_extent_node(n: Node, tree: vec2<u32>, p: Ptr, root: RootFields, fsid_low: vec2<u32>, expected_level: u32, is_the_root_of_the_tree: bool,
     range_inode_first: vec2<u32>, range_offset_first: vec2<u32>, range_inode_last: vec2<u32>, range_offset_last: vec2<u32>) -> bool {
     if (!n.ok) {
         return false;
@@ -2322,7 +2324,8 @@ fn judge_extent_node(n: Node, tree: vec2<u32>, p: Ptr, root: RootFields, fsid_lo
         || !extent_key_equals(n.base, n.largest_offset, range_inode_last, range_offset_last)) {
         return false;
     }
-    if (n.entry_count == 0u) {
+    // 零条目只许上段的根兼叶：第一个文件删完之后的零条目根（D8（核心索引结构） 已定项 16 第 5 条），与 read_and_judge_node 同一判
+    if (n.entry_count == 0u && !(is_the_root_of_the_tree && expected_level == 0u)) {
         return false;
     }
     return true;
@@ -2361,7 +2364,7 @@ fn read_upper_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_of
             let span = extent_upper_span(level);
             let first_inode = mul64_64_saturating(pending_index, span);
             let last_inode = add64_saturating(first_inode, sub64(span, u64_of(1u)));
-            if (!judge_extent_node(n, tree, pending_pointer, root, fsid_low, level, first_inode, vec2<u32>(0u, 0u), last_inode, vec2<u32>(NONE, NONE))) {
+            if (!judge_extent_node(n, tree, pending_pointer, root, fsid_low, level, depth == 0u, first_inode, vec2<u32>(0u, 0u), last_inode, vec2<u32>(NONE, NONE))) {
                 return out;
             }
             nodes = nodes + 1u;
@@ -2527,7 +2530,7 @@ fn read_lower_segment(root_pointer: Ptr, root_pointer_base: u32, root_pointer_of
             let last_unit = add64_saturating(first_unit, sub64(span, u64_of(1u)));
             let first_offset = mul64_64_saturating(first_unit, payload);
             let last_offset = sub64_saturating(mul64_64_saturating(add64_saturating(last_unit, u64_of(1u)), payload), u64_of(1u));
-            if (!judge_extent_node(n, tree, pending_pointer, root, fsid_low, level, inode, first_offset, inode, last_offset)) {
+            if (!judge_extent_node(n, tree, pending_pointer, root, fsid_low, level, false, inode, first_offset, inode, last_offset)) {
                 return out;
             }
             nodes = nodes + 1u;
@@ -2680,7 +2683,14 @@ fn read_extent_tree_of_the_first_file(root_pointer: Ptr, root_pointer_base: u32,
             data_count = data_count + 1u;
         }
     }
-    // extents_of_the_first_file：恰好一条叶条目、是第一个文件、至少一个数据单元、单元从 0 起连号
+    // extents_of_the_first_file：上段一条叶条目都没有是第一个文件删完之后的零条目根（D8（核心索引结构） 已定项 16 第 5 条），
+    // 没有数据单元、没有下段，读得出（同 recovery.rs extents_of_the_first_file 那一支）；结局由后面 inode 树里第一个文件那条记录定
+    if (upper_count == 0u) {
+        out.ok = true;
+        out.node_count = nodes;
+        return out;
+    }
+    // 否则恰好一条叶条目、是第一个文件、至少一个数据单元、单元从 0 起连号
     if (upper_count != 1u) {
         return out;
     }
diff --git a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
index 050b98b..5da98a7 100644
--- a/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
+++ b/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs
@@ -10,7 +10,8 @@
 //! 树表 11 条与种类 9–11（谱系树的容器记录宽、谱系树根的树 ID、谱系树与 logged_ops 两条的发号次序）各有一条，树表条数不超过扫描内核的上限取卡之前就判。
 //! 里程碑三·3 删除线（I-9.5、I-3.4、I-4.7、I-4.6、I-9.6 墓碑那一半）：删除中间态（unlink 意图与第一批）一条流钉四条判过而成立，
 //! 每条各一条只改坏它管的那个字段的流钉 CPU 红、GPU 逐位跟上；走过 nlink = 0 那几版的流与别的流一样整份比（红位、池级 checker、两遍恢复的结局，
-//! GPU 恢复内核与 CPU 冷走读一样把 nlink = 0 的第一个文件答成没有文件，C712（GPU 恢复内核不判 nlink = 0））；删除中间态那条流上展开上限
+//! GPU 恢复内核与 CPU 冷走读一样把 nlink = 0 的第一个文件答成没有文件，C712（GPU 恢复内核不判 nlink = 0））；另一条流删到最后一批，
+//! 那一版起 extent 树只剩上段零条目根，同样整份比（两边都答没有文件）；删除中间态那条流上展开上限
 //! 不展开的段（新建文件、顺序写、意图、第一批那几次发布的单元写）另把「之前的段全落、这一段只差一个写」的每个状态各比一次。
 //! I-9.16 ③ ④（种类重复、缺 deadlist）、根槽 428..457（I-2.4）、带树一版根记录里的分配记录树根（I-5.4）各一条坏流整份比；指针头部的
 //! extent 偏移（I-2.4）在树表条目的树根指针、根记录的实例表指针、inode 内部条目子指针、extent 上段叶内联的数据指针与下段根指针、
@@ -4931,18 +4932,56 @@ fn record_new_pool_through_the_first_deletion_batch(tag: &str, name: &'static st
     recorded_flow_of_the_pool(
         &pool,
         name,
-        vec![
-            PublishedVersion {
-                instance: InstanceGeneration(1),
-                checkpoint_txg: CheckpointTxg(3),
-                content: file_content(),
-            },
-            PublishedVersion {
-                instance: InstanceGeneration(1),
-                checkpoint_txg: CheckpointTxg(4),
-                content: content_of_three_data_units(),
-            },
-        ],
+        versions_with_the_first_file_before_its_deletion(),
+    )
+}
+
+/// 删除线那几条流上读得出第一个文件的两版：新建文件（txg 3）与顺序写成三个数据单元（txg 4）；意图那一版起恢复答「没有文件」。
+fn versions_with_the_first_file_before_its_deletion() -> Vec<PublishedVersion> {
+    vec![
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
+    ]
+}
+
+/// 三个单元的文件按批大小 1 删几批删完：前两批各放掉尾部一个单元，最后一批删记录、队列里的号与最后一个单元
+/// （`unlink_single_file_publishes.rs` 的 `the_final_batch_removes_the_record_the_queue_key_and_every_remaining_unit_in_the_same_publish`）。
+const BATCHES_OF_ONE_THAT_DELETE_A_THREE_UNIT_FILE: usize = 3;
+
+/// 删完的流：同 [`record_new_pool_through_the_first_deletion_batch`] 的搭建，再一批批删到最后一批（txg 6、7、8）。最后一批那一版起
+/// extent 树只剩上段零条目根（D8（核心索引结构） 已定项 16 第 5 条）、没有下段与数据单元，inode 树里没有第一个文件那条记录，
+/// 队列空；恢复从意图那一版起答「没有文件」，带文件的版本同样只有 txg 3、4。
+///
+/// # Panics
+/// 最后一批那一版还有第一个文件的记录、队列里的号、数据单元或下段，或 extent 上段不是只剩一个节点：那这条流就没走到零条目根。
+fn record_new_pool_through_the_last_deletion_batch(tag: &str, name: &'static str) -> RecordedFlow {
+    let mut pool = pool_with_a_three_unit_file(tag);
+    publish_the_intent(&mut pool);
+    let last_batch = (0..BATCHES_OF_ONE_THAT_DELETE_A_THREE_UNIT_FILE)
+        .map(|_batch_index| publish_the_next_step_in_batches_of_one(&mut pool))
+        .last()
+        .expect("删完要至少一批");
+    assert!(
+        last_batch.inode_record.is_none()
+            && last_batch.deleted_inodes.is_empty()
+            && last_batch.data_pointers.is_empty()
+            && last_batch.extent_tree.lower_nodes.is_empty()
+            && last_batch.extent_tree.upper_nodes.len() == 1,
+        "第 {BATCHES_OF_ONE_THAT_DELETE_A_THREE_UNIT_FILE} 批是最后一批：记录、队列里的号、数据单元与下段都不剩，extent 树只剩上段零条目根（{:?}）",
+        last_batch.extent_tree
+    );
+    recorded_flow_of_the_pool(
+        &pool,
+        name,
+        versions_with_the_first_file_before_its_deletion(),
     )
 }
 
@@ -5160,13 +5199,13 @@ fn set_the_nlink_of_the_new_inode_to_zero(bytes: &mut [u8]) {
 /// 两边都把 nlink = 0 的第一个文件答成没有文件，C712（GPU 恢复内核不判 nlink = 0））；删除线四条在这条流上 CPU 判过而成立、一个状态都不红
 /// （I-9.5 队列 {1} 与 nlink = 0 的号集合相同、I-3.4 标签 4 那一行在且为 0、I-4.7 号 1 在队列里、I-4.6 单元数等于 ⌈size ÷ 净荷容量⌉）。
 #[test]
-fn the_gpu_checker_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress() {
+fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress() {
     let flow = record_new_pool_through_the_first_deletion_batch(
         "crash-judge-gpu-deletion-in-progress",
         "deletion-through-the-first-batch",
     );
     let Some(card) = one_usable_card_or_report_not_run(
-        "the_gpu_checker_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress",
+        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress",
     ) else {
         return;
     };
@@ -5192,6 +5231,43 @@ fn the_gpu_checker_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress(
     }
 }
 
+/// 删到最后一批（意图之后三批）的每个状态上 GPU 与 CPU 整份相同（红位、池级 checker 的违例位、两遍恢复落到的根与结局）：最后一批那一版起
+/// extent 树只剩上段零条目根，CPU 冷走读把它当第一个文件删完之后的合法形态收下（`recovery.rs` 的 `extents_of_the_first_file`）、
+/// inode 树里没有那条记录、答没有文件，GPU 恢复内核同判（`judge_extent_node` 只放过上段根兼叶的零条目，读第一个文件放过零条叶条目）；
+/// 删除线四条在这条流上 CPU 判过而成立、一个状态都不红。
+#[test]
+fn the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch() {
+    let flow = record_new_pool_through_the_last_deletion_batch(
+        "crash-judge-gpu-deletion-through-the-last-batch",
+        "deletion-through-the-last-batch",
+    );
+    let Some(card) = one_usable_card_or_report_not_run(
+        "the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch",
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
+    for invariant in ["I-9.5", "I-3.4", "I-4.7", "I-4.6"] {
+        assert!(
+            comparison.cpu_checker_held_invariants.contains(invariant),
+            "删到最后一批的流上 CPU 池级 checker 判过 {invariant} 而成立（成立过的：{:?}）",
+            comparison.cpu_checker_held_invariants
+        );
+        assert!(
+            !comparison.cpu_checker_red_invariants.contains(invariant),
+            "删到最后一批的流上 CPU 池级 checker 不红 {invariant}（红过的：{:?}）",
+            comparison.cpu_checker_red_invariants
+        );
+    }
+}
+
 /// 一条流换一种切段，造出「之前的段全落、第 `segment_index` 段只差第 `missing_write_index` 次写」那一个状态：`segments` 把原来那一段拆成
 /// 「这一段其余的写」与「只有这一次写」前后两段（别的段照旧），只展开后一段（下标 `segment_index + 1`）。持久集合与原来那一段全量展开
 /// （[`Layer0SegmentExpansion::EveryProperSubset`]）枚举到的状态之一逐个相同；状态只有它与最后全落那一个。
@@ -5331,7 +5407,7 @@ fn the_gpu_judge_agrees_with_the_cpu_on_every_state_missing_one_write_of_an_unex
 }
 
 #[test]
-fn the_gpu_checker_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record() {
+fn the_gpu_judge_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record() {
     let flow = record_deletion_with_a_resealed_field(
         "crash-judge-gpu-i95",
         "deletion-queue-names-an-inode-without-a-record",
@@ -5339,14 +5415,14 @@ fn the_gpu_checker_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_wi
         &put_an_inode_number_without_a_record_into_the_deletion_queue_root,
     );
     assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
-        "the_gpu_checker_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record",
+        "the_gpu_judge_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record",
         &flow,
         "I-9.5",
     );
 }
 
 #[test]
-fn the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero() {
+fn the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero() {
     let flow = record_deletion_with_a_resealed_field(
         "crash-judge-gpu-i34",
         "deletion-pending-occupancy-row-one",
@@ -5354,14 +5430,14 @@ fn the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_i
         &set_the_pending_deletion_occupancy_row_to_one,
     );
     assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
-        "the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero",
+        "the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero",
         &flow,
         "I-3.4",
     );
 }
 
 #[test]
-fn the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_missing() {
+fn the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_missing() {
     let flow = record_deletion_with_a_resealed_field(
         "crash-judge-gpu-i34-missing-row",
         "deletion-pending-occupancy-row-missing",
@@ -5369,7 +5445,7 @@ fn the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_i
         &drop_the_pending_deletion_occupancy_row,
     );
     assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
-        "the_gpu_checker_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_missing",
+        "the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_missing",
         &flow,
         "I-3.4",
     );
@@ -5405,7 +5481,7 @@ fn the_gpu_judge_agrees_with_the_cpu_when_a_record_of_nlink_zero_is_not_in_the_d
 }
 
 #[test]
-fn the_gpu_checker_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units(
+fn the_gpu_judge_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units(
 ) {
     let flow = record_deletion_with_a_resealed_field(
         "crash-judge-gpu-i46",
@@ -5414,15 +5490,15 @@ fn the_gpu_checker_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_a
         &raise_the_size_of_the_first_file_by_one_data_unit_payload,
     );
     assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
-        "the_gpu_checker_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units",
+        "the_gpu_judge_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units",
         &flow,
         "I-4.6",
     );
 }
 
 #[test]
-fn the_gpu_checker_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark(
-) {
+fn the_gpu_judge_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark()
+{
     let flow = record_deletion_with_a_resealed_field(
         "crash-judge-gpu-i96-tombstone",
         "deletion-tombstone-past-the-watermark",
@@ -5430,7 +5506,7 @@ fn the_gpu_checker_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_in
         &raise_the_first_tombstone_object_identifier_past_the_watermark,
     );
     assert_the_gpu_judge_agrees_with_the_cpu_on_a_flow_reddening(
-        "the_gpu_checker_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark",
+        "the_gpu_judge_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark",
         &flow,
         "I-9.6",
     );
````

## 实现员 g：mutations-append.tsv（/tmp/claude-1000/m3-batch21/g/patch/mutations-append.tsv）

````text
第 21 批 N5 GPU 判器：读第一个文件不放过零条叶条目，删到最后一批那一版起 GPU 走读失败而 CPU 答没有文件（删到最后一批的整份比用例红）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    if (upper_count == 0u) {\n        out.ok = true;	    if (false && upper_count == 0u) {\n        out.ok = true;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch	the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch
第 21 批 N5 GPU 判器：judge_extent_node 不放过上段根兼叶的零条目，删到最后一批那一版起 GPU 走读失败而 CPU 答没有文件（删到最后一批的整份比用例红）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	if (n.entry_count == 0u && !(is_the_root_of_the_tree && expected_level == 0u)) {	if (n.entry_count == 0u) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch	the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_through_the_last_batch
第 21 批 nlink 常量合一：共用那一半的 nlink 偏移读成 gid（偏移 32），恢复内核与扫描内核一起读错（删除中间态的整份比用例红）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	const INODE_RECORD_NLINK_OFFSET: u32 = 36u;	const INODE_RECORD_NLINK_OFFSET: u32 = 32u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress	the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress
````

## 实现员 g：mutations-replacements.tsv（/tmp/claude-1000/m3-batch21/g/patch/mutations-replacements.tsv）

````text
三·3 GPU 判器：I-9.5 正方向不判（队列里没有记录、或记录 nlink ≠ 0 的号放过）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	            if (row == NONE || sw(row + 2u) != 0u) {	            if (false && (row == NONE || sw(row + 2u) != 0u)) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record	the_gpu_judge_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record
三·3 GPU 判器：I-3.4 只判缺行、不判标签 4 那一行的值	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    if (!has_the_pending_deletion_row || some_pending_deletion_value_is_not_zero) {	    if (!has_the_pending_deletion_row) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero	the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero
三·3 GPU 判器：I-3.4 只判值、不判缺行	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    if (!has_the_pending_deletion_row || some_pending_deletion_value_is_not_zero) {	    if (some_pending_deletion_value_is_not_zero) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_missing	the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_missing
三·3 GPU 判器：I-4.6 不判（nlink = 0 的记录单元数与 size 对不上放过）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	        if (!matches_the_size) {	        if (false) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units	the_gpu_judge_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units
三·3 GPU 判器：I-9.6 墓碑那一半不判（已发布墓碑的对象 ID 不小于水位放过）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    if (has_a_published_tombstone && !lt64(largest_object_identifier, watermark)) {	    if (false && has_a_published_tombstone && !lt64(largest_object_identifier, watermark)) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark	the_gpu_judge_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark
三·3 GPU 判器：extent 树下段叶的数据单元不按号数（删除中 inode 的单元数读成 0）	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	                    r_count_data_unit_of_inode(inode, lower_data_pointer);	                    let lower_data_pointer_not_counted = lower_data_pointer;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress	the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress
三·3 GPU 判器：扫描内核没接删除态扫描	crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl	    judge_deletion_state_of_the_versions();	    let deletion_state_not_judged = 0u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero	the_gpu_judge_agrees_with_the_cpu_when_the_pending_deletion_occupancy_row_is_not_zero
三·3 GPU 判器：I-9.5 的下标错位成 I-3.4 那一位（70）	crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl	const INVARIANT_I_9_5: u32 = 69u;	const INVARIANT_I_9_5: u32 = 70u;	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record	the_gpu_judge_agrees_with_the_cpu_when_the_deletion_queue_names_an_inode_without_a_record
第 20 批 C712 GPU 判器：恢复内核不判 nlink = 0，删除中间态那条流上意图那一版起照读文件（整份比的删除中间态用例红）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {	    if (false && inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress	the_gpu_judge_agrees_with_the_cpu_on_every_state_of_a_deletion_in_progress
第 20 批 C712 GPU 判器：恢复内核不判 nlink = 0，墓碑坏流上意图那一版起照读文件（换回整份比的坏流用例红）	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {	    if (false && inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark	the_gpu_judge_agrees_with_the_cpu_when_a_published_tombstone_reaches_the_inode_number_watermark
第 20 批 C712 GPU 判器：恢复内核的 nlink = 0 判挪到核单元数之后，删除中而 size 多一个净荷的那一版判成走读失败而不是没有文件	crates/singlefs-checker-tier/src/crash_judge_common.wgsl	    if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {\n        return walk_no_file();\n    }\n    // 数据单元数 = max(1, ⌈size ÷ 32634⌉)\n    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);\n    var expected_units = divmod64(add64_saturating(inode_size, sub64(payload, u64_of(1u))), payload).quotient;\n    if (is_zero64(expected_units)) {\n        expected_units = u64_of(1u);\n    }\n    if (!eq64(expected_units, u64_of(data_count))) {\n        return walk_failed(54u);\n    }\n	    // 数据单元数 = max(1, ⌈size ÷ 32634⌉)\n    let payload = u64_of(DATA_UNIT_PAYLOAD_CAPACITY);\n    var expected_units = divmod64(add64_saturating(inode_size, sub64(payload, u64_of(1u))), payload).quotient;\n    if (is_zero64(expected_units)) {\n        expected_units = u64_of(1u);\n    }\n    if (!eq64(expected_units, u64_of(data_count))) {\n        return walk_failed(54u);\n    }\n    if (inode_nlink == NLINK_OF_AN_INODE_BEING_DELETED) {\n        return walk_no_file();\n    }\n	-p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- the_gpu_judge_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units	the_gpu_judge_agrees_with_the_cpu_when_the_size_of_an_inode_being_deleted_asks_for_more_data_units
````

## 实现员 r：crates.patch（/tmp/claude-1000/m3-batch21/r/patch/crates.patch）

````diff
diff --git a/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs b/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs
index 6002fe1..3c6b9c7 100644
--- a/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs
+++ b/crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs
@@ -2,8 +2,14 @@
 //! 水位只增，「复用一个已发过的号」造不出来；开关（`PoolWriter::set_inode_number_watermark_source`）把这次发布的起号压回去，
 //! 再建一个 inode，看池级 checker 的 I-9.6（水位大于两处最大号）拦不拦得住。
 //! 判别力自证：同一段流把开关关掉，I-9.6 由红转绿。
-//! 「水位压回去、但那个号原来的记录已被换掉」那一形（压回的正是发过的最大号：它的记录还在树里被原地换掉，或已 unlink 删完、
-//! 墓碑在所选根的墓碑树里）I-9.6 拦不住，这里不钉，交主 agent 定要不要另立一条不变量（C479 那一行）。
+//! I-9.6 拦不住的那一片（攻方 `research/prompts/m3-batch19-code-r1-opus-output.md`「N7」一节逐点扫过）：压回到 p、建 n 个之后记账的水位是 p + n，
+//! 只要 p + n 大于 checker 判的两处最大号——它走到的各条根里 inode 树的最大 key，与所选根墓碑树里已发布墓碑的最大对象 ID——两半都成立，
+//! 被重发的那几个号（p 到 p + n − 1 里原来发过的：记录还在树里被原地换掉，或已 unlink 删完、墓碑在所选根的墓碑树里）I-9.6 拦不住。
+//! 只建一个（n = 1）时这一片正是「压回的正是发过的最大号」；建几个时压回的号可以低于发过的最大号（攻方那 90 个判不出的点里 42 个是这样，
+//! 都在 n ≥ 2）。这一片这里不钉，交主 agent 定要不要另立一条不变量（C479 那一行；判决 `research/prompts/m3-batch19-code-r1-main-verification.md`
+//! 第二节「N7 附带」那一行交给 C479 那一问当材料）。
+//! p + n 不大于墓碑最大对象号、又大于各条根里 inode 树最大 key 的那一形，I-9.6 墓碑那一半判得出，钉在
+//! [`creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant`]。
 
 mod common;
 mod common_deletion_in_progress;
@@ -15,10 +21,16 @@ use common_deletion_in_progress::{
     WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
 };
 use singlefs_checker::image::InvariantVerdict;
+use singlefs_core::address::{CheckpointTxg, InodeNumber};
+use singlefs_core::mount::{
+    raise_rollback_floor_to_the_admission_ceiling, RaiseToTheAdmissionCeiling, ShadowLedger,
+};
 use singlefs_core::transaction::{
-    publish_new_inodes, InodeNumberWatermarkPressedBackForTests, InodeNumberWatermarkSource,
-    PoolWriter, TransactionOutput,
+    publish_new_inodes, publish_overwrite, publish_unlink_intent, publish_unlink_step, FirstFile,
+    InodeNumberWatermarkPressedBackForTests, InodeNumberWatermarkSource, PoolWriter,
+    TransactionOutput,
 };
+use singlefs_core::unlink::UnlinkBatchSize;
 use singlefs_harness::memory_pool::MemoryPool;
 
 /// 接着现行那一版建一个 inode，写入口上装着 `source` 这个水位来源；交回这一版与它的镜像。
@@ -128,3 +140,184 @@ fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_key_r
         "开关关着的同一段流一条违例都没有"
     );
 }
+
+/// 在这个进程里把 `inode` unlink 到底：发意图，再按产品批大小一批一批发到删除队列空（它的记录删掉，墓碑留在墓碑树里）。
+fn unlink_to_the_end_in_process(pool: &mut BuiltPool, inode: InodeNumber) {
+    let publish_parameters = parameters();
+    {
+        let previous = pool.output.clone();
+        let devices = pool.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        pool.output = publish_unlink_intent(
+            &mut writer,
+            &mut pool.allocator,
+            &previous,
+            inode,
+            WRITE_TIME_SECONDS_OF_THE_NEW_INODES,
+            INSTANCE_OF_THE_PUBLISHES,
+        )
+        .expect("意图");
+    }
+    let mut steps_published = 0;
+    // 迭代上界：建出来的空 inode 没有数据单元，意图之后一批就删完；多走一步就是删不完。
+    while !pool.output.deleted_inodes.is_empty() {
+        assert!(steps_published < 1, "空 inode 意图之后一批就删完");
+        let previous = pool.output.clone();
+        let devices = pool.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        pool.output = publish_unlink_step(
+            &mut writer,
+            &mut pool.allocator,
+            &previous,
+            inode,
+            UnlinkBatchSize::OneExtentLowerLeaf,
+            INSTANCE_OF_THE_PUBLISHES,
+        )
+        .expect("删除的下一步");
+        steps_published += 1;
+    }
+}
+
+/// 在这个进程里把第一个文件覆盖写 `times` 次（每次一份不同的 3000 字节、写入时间各加一秒）：每次换一版 inode 记录与 extent 树，
+/// 根都是「非空」的（抬 F 的上限按第 4 新的非空有效根取）。
+fn overwrite_the_first_file_in_process(pool: &mut BuiltPool, times: u64) {
+    let publish_parameters = parameters();
+    // 迭代上界是 `times`。
+    for overwrite_index in 0..times {
+        let content: Vec<u8> = (0..3000_u64)
+            .map(|index| {
+                u8::try_from((index * 17 + overwrite_index * 5 + 3) % 251).expect("小于 256")
+            })
+            .collect();
+        let previous = pool.output.clone();
+        let devices = pool.devices.as_mut().expect("镜像还开着");
+        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
+        pool.output = publish_overwrite(
+            &mut writer,
+            &mut pool.allocator,
+            &previous,
+            FirstFile {
+                content: &content,
+                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_NEW_INODES + 1 + overwrite_index,
+            },
+            INSTANCE_OF_THE_PUBLISHES,
+        )
+        .unwrap_or_else(|refusal| panic!("第 {overwrite_index} 次覆盖写：{refusal:?}"));
+    }
+}
+
+/// 删完那一版之后覆盖写几次：删除最后一批那条根之后再有三条非空的根，抬 F 的上限（第 4 新的非空有效根）才走到删除最后一批那条根。
+const OVERWRITES_AFTER_THE_DELETION_SO_THE_CEILING_REACHES_ITS_LAST_ROOT: u64 = 3;
+
+/// 新池新建文件之后建 inode 2、3（水位 4），把 3 号 unlink 到底（记录删掉、墓碑的对象 ID 是 3），覆盖写第一个文件 3 次，再把 F 抬到准入上限：
+/// 上限走到删除最后一批那条根，还带着 3 号记录的那几条根（意图那一条与更早的）都低于 F，池级 checker 不再走它们。
+/// 交回删除最后一批那条根的 txg。
+fn pool_whose_inode_three_is_unlinked_and_whose_floor_is_raised_past_its_record(
+    tag: &str,
+) -> (BuiltPool, CheckpointTxg) {
+    let mut pool = pool_with_new_inodes(tag, 2);
+    assert_eq!(
+        pool.output.inode_number_watermark(),
+        4,
+        "inode 1 到 3 发过，水位 4"
+    );
+    unlink_to_the_end_in_process(&mut pool, InodeNumber(3));
+    assert_eq!(object_birth_of(&pool.output, 3), None, "3 号那条记录删掉了");
+    let txg_of_the_last_deletion_step = pool.output.root.checkpoint_txg;
+    overwrite_the_first_file_in_process(
+        &mut pool,
+        OVERWRITES_AFTER_THE_DELETION_SO_THE_CEILING_REACHES_ITS_LAST_ROOT,
+    );
+    let publish_parameters = parameters();
+    let devices = pool.devices.as_mut().expect("镜像还开着");
+    let raised = raise_rollback_floor_to_the_admission_ceiling(
+        &publish_parameters,
+        devices,
+        &mut pool.allocator,
+        &mut pool.output,
+        ShadowLedger::Off,
+    )
+    .unwrap_or_else(|error| panic!("抬 F 到准入上限：{error:?}"));
+    match raised {
+        RaiseToTheAdmissionCeiling::Raised(_) => {}
+        RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling } => {
+            panic!("删完之后覆盖写三次，上限高于 F：F {floor:?}、上限 {ceiling:?}")
+        }
+    }
+    assert!(
+        pool.output.root.rollback_floor >= txg_of_the_last_deletion_step,
+        "F 抬到 {:?}，不低于删除最后一批那条根的 txg {txg_of_the_last_deletion_step:?}：还带着 3 号记录的根都低于 F",
+        pool.output.root.rollback_floor
+    );
+    (pool, txg_of_the_last_deletion_step)
+}
+
+/// C479 变体 c（第 19 批 S 的探针那一形，判决 `research/prompts/m3-batch19-code-r1-main-verification.md` 第二节「N7 附带」那一行）：
+/// 建 inode 2、3，把 3 号 unlink 到底（墓碑的对象 ID 是 3），覆盖写第一个文件 3 次、把 F 抬到删除最后一批那条根，再把水位压回 2（低于墓碑最大对象号 3）
+/// 建一个 inode：2 号被重发（树里那条记录原地换成新对象），记账的水位写成 3。池级 checker 走到的根里 inode 树最大 key 是 2，水位 3 大于它，
+/// 那一半成立；所选根墓碑树里已发布墓碑的最大对象 ID 是 3，水位不大于它，I-9.6 判红，说明写的是墓碑那一半。同一段流开关关着时
+/// 新建的拿 4 号、水位 5，I-9.6 成立、一条违例都没有。
+/// 不抬 F 时池级 checker 还走着意图那一条根（3 号记录 nlink = 0 还在树里），树内最大 key 那一半先判出 3，说明写的是树那一半（S 的探针量过），
+/// 判不出墓碑那一半拦不拦得住。
+/// 判别力：墓碑那一半不比水位（`walk.rs` 里那一判恒真）时 I-9.6 成立，这一条红；关开关那一半由红转绿是判别力自证。
+#[test]
+fn creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant(
+) {
+    let (mut pressed_pool, txg_of_the_last_deletion_step) =
+        pool_whose_inode_three_is_unlinked_and_whose_floor_is_raised_past_its_record(
+            "watermark-pressed-back-below-the-largest-tombstone",
+        );
+    let (pressed, pressed_image) =
+        create_one_inode_with_the_watermark_from(&mut pressed_pool, pressed_back_to(2));
+    assert_eq!(
+        pressed.inode_number_watermark(),
+        3,
+        "从压回去的 2 起发一个，水位写成 3"
+    );
+    assert_eq!(
+        object_birth_of(&pressed, 2),
+        Some(pressed.root.checkpoint_txg.0),
+        "2 号被重发：树里那条记录换成这次发布建的新对象"
+    );
+    match verdict_of(&pressed_image, "I-9.6") {
+        InvariantVerdict::Violated(detail) => assert!(
+            detail.starts_with("记账里的 inode 号水位 3 不大于所选根")
+                && detail.ends_with("的墓碑树里已发布的墓碑记录的最大对象 ID 3"),
+            "违例说明写的是墓碑那一半（删除最后一批那条根 txg {txg_of_the_last_deletion_step:?} 之后走到的根里 inode 树最大 key 是 2，树那一半成立）：{detail}"
+        ),
+        other_verdict @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
+            panic!("水位压回到墓碑最大对象号之下再建，I-9.6 墓碑那一半应当判红，实际 {other_verdict:?}")
+        }
+    }
+    assert_eq!(
+        violated_invariants(&pressed_image),
+        vec!["I-9.6"],
+        "只红 I-9.6"
+    );
+
+    let (mut control_pool, _) =
+        pool_whose_inode_three_is_unlinked_and_whose_floor_is_raised_past_its_record(
+            "watermark-not-pressed-back-below-the-largest-tombstone-control",
+        );
+    let (control, control_image) = create_one_inode_with_the_watermark_from(
+        &mut control_pool,
+        InodeNumberWatermarkSource::FromThePreviousVersionsAccounting,
+    );
+    assert_eq!(
+        control.inode_number_watermark(),
+        5,
+        "开关关着：发 4 号，水位 5"
+    );
+    assert_eq!(
+        verdict_of(&control_image, "I-9.6"),
+        InvariantVerdict::Holds,
+        "开关关着的同一段流 I-9.6 成立"
+    );
+    assert_eq!(
+        violated_invariants(&control_image),
+        Vec::<&str>::new(),
+        "开关关着的同一段流一条违例都没有"
+    );
+}
diff --git a/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs b/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
index afa5639..ba29020 100644
--- a/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
+++ b/crates/singlefs-harness/tests/the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads.rs
@@ -612,12 +612,14 @@ fn slots_released_by_the_current_version(pool: &PoolUnderTest<SparseBlockDevice>
         .collect()
 }
 
-/// 放开挪到暖机之前那一刀（判决 `research/prompts/m3-batch17-code-r1-main-verification.md` 第二节 K1-N4 行「钉不住放开挪到暖机之前」）：
+/// 放开挪到暖机之前那一刀（出处：第 17 批正文 `research/prompts/_m3-batch17-code-r1-body.md` 第 27 行「「放开挪到暖机之前」那一形没有用例钉」）：
 /// 不补环窗口、准入关着造的窄盘（单元区 176 槽），第一个文件之后不再写；卸载只落下第一个写就崩，两块盘都在、再可写挂载（准入判着、影子账开、
 /// 批大小 1）做成。写行那次与暖机每一次写下的单元一个都不落在卸载之前现行那一版换下的槽上：那几个槽扣到暖机落满每块盘才放开。
 /// 判别力：`mount::establish_instance` 里那一句 `release_reclaim_holds` 挪到暖机之前时，暖机那次的单元写进换下的槽 1410（盘 1 上最新的根还引用它）：
 /// 取号之前在分配器拷贝上预演的落点与真发的不同，挂载在预演对真发那一判上 panic，这一条红；那一判不在时，最后一项红
 /// （在副本里关掉那一判量过：暖机落在换下的槽 [1410] 上）。只读挂载丢盘的那几条都在写行那次就崩，暖机没跑到，判不出这一刀。
+/// 卸载只落下第一个写，这次挂载的暖机只推一次：「第一次暖机之后就放开」与「暖机落满之后放开」在这里是同一刻，判不出，那一刀见
+/// [`after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_in_either_warm_up`]。
 #[test]
 fn after_a_crashed_unmount_of_a_narrow_pool_the_next_writable_mount_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_until_its_warm_up_landed_on_every_device(
 ) {
@@ -662,6 +664,82 @@ fn after_a_crashed_unmount_of_a_narrow_pool_the_next_writable_mount_writes_no_un
     );
 }
 
+/// 第一个文件之后、卸载之前覆盖写几次（攻方 `research/prompts/m3-batch19-code-r1-opus-output.md`「N3」一节 K = 3 那份历史）。
+const OVERWRITES_BEFORE_AN_UNMOUNT_WHOSE_NEXT_MOUNT_WARMS_UP_TWICE: usize = 3;
+
+/// 卸载落下这么多个写就崩：两块盘的系统配置都带了新 F，第一次抬 F 空发布的 journal 记录已落、它的根没落；这时下一次可写挂载的暖机要推两次
+/// （攻方同一节：卸载崩在第 15、16 个写之后暖机推两次）。
+const UNMOUNT_WRITES_LANDED_BEFORE_A_MOUNT_THAT_WARMS_UP_TWICE: u64 = 15;
+
+/// 放开挪到第一次暖机之后那一刀（判决 `research/prompts/m3-batch19-code-r1-main-verification.md` 第二节「K2 N3 / N4 错实现漏红」那一行，
+/// 攻方同一份报告改法表 G1）：不补环窗口、准入关着造的窄盘（单元区 176 槽），第一个文件之后覆盖写 3 次；卸载落下 15 个写就崩，两块盘都在、
+/// 再可写挂载（准入判着、影子账开、批大小 1）做成，这次挂载的暖机推两次。写行那次与两次暖机每一次写下的单元一个都不落在卸载之前现行那一版
+/// 换下的槽上：那几个槽的释放代是现行那一版的 txg，高于盘 1 上最新的根、不高于 F_生效，扣到暖机在每块盘上都落了根才放开（D16（发布语义）
+/// 已定项 1「正常卸载」那一行，`mount::rebuilt_allocator` 扣、`mount::establish_instance` 暖机循环之后放开）。
+/// 判别力：第一次暖机之后就放开、取号之前的预演同样在那里放开（预演对真发那一判不分叉）时，第二次暖机的单元写进换下的槽，最后一项红
+/// （攻方原型上量过：这份历史写进换下的槽 2 个，`research/prompts/m3-batch19-code-r1-opus-model/out/n34-176-3-1-a036.txt` 第 150 行
+/// `inR=2`；丢盘 0 之后剩下那块盘读文件报 `DataUnitChecksumMismatchEverywhere`）。只在真发那一处放开、预演不放开时，挂载在预演对真发那一判上
+/// panic，这一条同样红。卸载只落下第一个写的那几条暖机只推一次，「第一次暖机之后」与「暖机落满之后」是同一刻，判不出这一刀。
+#[test]
+fn after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_in_either_warm_up(
+) {
+    let device_width = HistoryDeviceWidth::UnitAreaOf384Slots;
+    let mut pool =
+        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
+            device_width,
+            176,
+        );
+    for overwrite_index in 0..OVERWRITES_BEFORE_AN_UNMOUNT_WHOSE_NEXT_MOUNT_WARMS_UP_TWICE {
+        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
+            panic!("第一个文件之后第 {overwrite_index} 次覆盖写：{refusal:?}")
+        });
+    }
+    let released_before_the_unmount = slots_released_by_the_current_version(&pool);
+    assert!(
+        !released_before_the_unmount.is_empty(),
+        "卸载之前现行那一版换下过槽（最后一次覆盖写换下上一版的单元）"
+    );
+    let after_the_unmount = image_after_an_unmount_crashing_after(
+        &pool,
+        device_width,
+        UNMOUNT_WRITES_LANDED_BEFORE_A_MOUNT_THAT_WARMS_UP_TWICE,
+    );
+    let mut remount_devices = plain_devices_on(&after_the_unmount);
+    let mounted = mount_writable_through(
+        RemountEntry::WithDeletionBatchesOfOne,
+        &pool,
+        &mut remount_devices,
+    )
+    .unwrap_or_else(|error| panic!("卸载半途崩溃之后两块盘都在、再可写挂载：{error:?}"));
+    assert_eq!(
+        mounted.output.warm_up_publishes.len(),
+        2,
+        "这次挂载在写行之后推了两次暖机"
+    );
+    let slots_taken_into_the_released_ones_by_every_publish: Vec<Vec<u64>> =
+        std::iter::once(&mounted.output.row_publish)
+            .chain(&mounted.output.warm_up_publishes)
+            .map(|version| {
+                version
+                    .version_of_trees()
+                    .map(|version| {
+                        version
+                            .units
+                            .iter()
+                            .flat_map(|unit| unit.slot.0..unit.slot.0 + unit.identity.span_slots())
+                            .filter(|slot| released_before_the_unmount.contains(slot))
+                            .collect()
+                    })
+                    .unwrap_or_default()
+            })
+            .collect();
+    assert_eq!(
+        slots_taken_into_the_released_ones_by_every_publish,
+        vec![Vec::<u64>::new(); 3],
+        "写行那次、第一次暖机、第二次暖机写下的单元（按这个次序一次一项）都不落在卸载之前现行那一版换下的槽 {released_before_the_unmount:?} 上"
+    );
+}
+
 /// 第一个文件先长到这么多个数据单元、再缩成一个单元：缩的那次换下的槽够多，扣着它们第一次写就长不回去（单元区 176 槽）。
 const DATA_UNITS_OF_THE_FILE_BEFORE_IT_IS_SHRUNK: usize = 36;
 
@@ -671,6 +749,14 @@ const DATA_UNITS_OF_THE_FILE_BEFORE_IT_IS_SHRUNK: usize = 36;
 /// 挂载交回之后第一次写把文件顺序写回 36 个单元：一串抬 F 都不推、只发成这一版，F 停在挂载那一刻的值。
 /// 判别力：删掉 `mount::establish_instance` 暖机之后那一句 `release_reclaim_holds` 时，缩的那次换下的槽留着扣住位交给会话，落点取不到，
 /// 这次写拒成空间不够（`NoSpaceAfterRaisingTheFloor`，副本里量过：单元区 176 槽上文件长到 32–40 个单元的每一点都是这个结局，不删时都写成、不推抬 F），这一条红。
+/// 这个结局只在准入关着时走得到：这块 176 槽的窄盘照式子判可用本来就是负的，准入开着时第一次写在准入处就被拒（`SpaceAdmissionRefused`），
+/// 走不到落点那一判（正推 `research/prompts/m3-batch19-code-r1-sonnet-output.md` 第 4 节「范围」那一条在基线上量过）；所以挂载与第一次写都关着准入。
+/// 判别力的边在文件长到多少个单元（攻方 `research/prompts/m3-batch19-code-r1-opus-output.md`「N4」一节，单元区 176、192、240 槽上扫过 8–52 个单元）：
+/// 长到 24 个单元以内、或单元区 240 槽时删掉那一句照样写成（不扣也放得下）；30 个单元只在暖机推两次的 3 个崩溃态上红（这一条卸载只落下第一个写，
+/// 暖机推一次，30 判不出）；这里取的 36 在会红的区里，下沿在 30 与 36 之间（31–35 攻方没量）。哪一次改动让每版元数据少占十几个槽，
+/// 这一条在删掉那一句时会悄悄变绿；4 GiB 那条直接数扣住位，不靠落点取不取得到，照样红。
+/// 最后那一项（推了几串抬 F、发成几版、F 动没动）另有一刀：顺序写那一版的根顺手把 F 带成上一版的 txg（不推抬 F 那一串就挪 F）时这次写照样写成，
+/// 只有 F 那一样不同，那一项红。
 #[test]
 fn after_a_crashed_unmount_of_a_narrow_pool_whose_file_was_shrunk_the_first_write_of_the_next_writable_mount_regrows_it_without_raising_the_floor(
 ) {
diff --git a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
index 056c2b5..e814472 100644
--- a/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
+++ b/crates/singlefs-harness/tests/unlink_mount_continuation_and_the_emptied_version.rs
@@ -501,12 +501,21 @@ fn the_intent_and_a_middle_batch_each_set_the_change_count_to_the_txg_of_their_o
     );
 }
 
-/// 第一个文件那条记录在 `version` 这一版里、两块盘上各读出来的三个时间秒（偏移 64 起 atime、mtime、ctime 各 8 字节，D8（核心索引结构）
-/// 已定项 6 字段表）：按盘读这一版每片 inode 叶容器的单元字节，在码 3 记录区（`PACKED_UNIT_RECORDS_OFFSET` 起、每条 `INODE_RECORD_BYTES`）里
-/// 找 inode 号是第一个文件的那一条。读的是盘，不是内存里那份记录。
+/// 第一个文件那条记录在 `version` 这一版里、两块盘上各读出来的三个时间秒（[`time_seconds_of_an_inode_on_each_device`]）。
 fn time_seconds_of_the_first_file_on_each_device(
     image: &MemoryPool,
     version: &TransactionOutput,
+) -> Vec<[u64; 3]> {
+    time_seconds_of_an_inode_on_each_device(image, version, FIRST_FILE)
+}
+
+/// `inode` 那条记录在 `version` 这一版里、两块盘上各读出来的三个时间秒（偏移 64 起 atime、mtime、ctime 各 8 字节，D8（核心索引结构）
+/// 已定项 6 字段表）：按盘读这一版每片 inode 叶容器的单元字节，在码 3 记录区（`PACKED_UNIT_RECORDS_OFFSET` 起、每条 `INODE_RECORD_BYTES`）里
+/// 找 inode 号是它的那一条。读的是盘，不是内存里那份记录。
+fn time_seconds_of_an_inode_on_each_device(
+    image: &MemoryPool,
+    version: &TransactionOutput,
+    inode: InodeNumber,
 ) -> Vec<[u64; 3]> {
     let record_bytes = usize::try_from(INODE_RECORD_BYTES).expect("140");
     let records_offset = usize::try_from(PACKED_UNIT_RECORDS_OFFSET).expect("136");
@@ -533,7 +542,7 @@ fn time_seconds_of_the_first_file_on_each_device(
                     );
                     unit_bytes[records_offset..]
                         .chunks_exact(record_bytes)
-                        .find(|record| field_at(record, 0) == FIRST_FILE.0)
+                        .find(|record| field_at(record, 0) == inode.0)
                         .map(|record| {
                             [
                                 field_at(record, 64),
@@ -542,7 +551,9 @@ fn time_seconds_of_the_first_file_on_each_device(
                             ]
                         })
                 })
-                .expect("这一版的 inode 叶容器里有第一个文件那条记录（意图与中间各批之后 nlink = 0、还在树里）")
+                .unwrap_or_else(|| {
+                    panic!("这一版的 inode 叶容器里有 inode {} 那条记录（意图与中间各批之后 nlink = 0、还在树里）", inode.0)
+                })
         })
         .collect()
 }
@@ -789,6 +800,100 @@ fn a_session_unlink_after_a_new_inode_with_a_later_time_in_the_same_mount_writes
     );
 }
 
+/// 同一次挂载里后一次 unlink 给的时间比这条记录上一版晚几秒：早于前一次给的（上一版加 600 秒）、不早于中间建的那个 inode 的时间（上一版）。
+const SECONDS_AFTER_THE_PREVIOUS_VERSION_OF_THE_SECOND_UNLINK_IN_THE_SAME_MOUNT: u64 = 300;
+
+/// `version` 这一版的 inode 树里有没有 `inode` 那条记录。
+fn version_holds_the_record_of(version: &TransactionOutput, inode: InodeNumber) -> bool {
+    version
+        .inode_leaf_container_contents()
+        .iter()
+        .flat_map(|container| container.records.iter())
+        .any(|record| record.inode == inode.0)
+}
+
+/// 同一次挂载里 unlink 两次、后一次给的时间更早（判决 `research/prompts/m3-batch19-code-r1-main-verification.md` 第二节「K2 N1 错实现漏红」那一行，
+/// 攻方 `research/prompts/m3-batch19-code-r1-opus-output.md`「N1」一节）：先 unlink 第一个文件、给上一版加 600 秒（会话走完意图与三步，这一版删空），
+/// 再建一个 inode（时间就是上一版的，不晚于后一次给的），再 unlink 它、给上一版加 300 秒。后一次那个 inode 那条记录只在意图那一版里
+/// （它没有数据单元，意图之后一批就删完），意图那一版两块盘上那条记录的三个时间秒都等于后一次给的时间。上面几条每次挂载只 unlink 一次。
+/// 判别力：会话那一层只对 unlink 只进不退（unlink 的时间取给的与这次挂载里上一次 unlink 用过的时间里较大的那个，攻方那一刀 q-unlink-mono）时
+/// 写下的是前一次的上一版加 600 秒，这一条红；只看这条记录上一版的时间、只看上一版各条记录最晚的时间、只记建 inode 给过的时间的几种写法
+/// 写下的都是后一次给的时间，照旧绿。
+#[test]
+fn two_session_unlinks_in_the_same_mount_the_second_given_an_earlier_time_write_the_second_given_time_into_the_three_time_fields_of_its_intent_on_disk(
+) {
+    let mut pool =
+        pool_with_a_three_unit_file("write-time-of-a-second-session-unlink-earlier-than-the-first");
+    let time_of_the_previous_version = write_time_seconds_of_the_first_file(&pool);
+    let time_given_to_the_first_unlink =
+        GivenTimeAgainstThePreviousVersion::Later.given_time(time_of_the_previous_version);
+    let time_given_to_the_second_unlink = time_of_the_previous_version
+        .checked_add(SECONDS_AFTER_THE_PREVIOUS_VERSION_OF_THE_SECOND_UNLINK_IN_THE_SAME_MOUNT)
+        .expect("上一版的时间是测试池写的固定时间，加 300 秒不溢出");
+    let time_of_the_new_inode = time_of_the_previous_version;
+    assert!(
+        time_of_the_new_inode <= time_given_to_the_second_unlink
+            && time_given_to_the_second_unlink < time_given_to_the_first_unlink,
+        "建的那个 inode 的时间 {time_of_the_new_inode} 不晚于后一次给的 {time_given_to_the_second_unlink}，后一次早于前一次给的 {time_given_to_the_first_unlink}"
+    );
+    let (mounted, mut devices) = remount_with_batches_of_one(&mut pool);
+    let (_, mut session) = MountedSession::of_the_mount(mounted);
+    session
+        .publish_user_change(
+            &mut devices,
+            UserChange::Unlink {
+                inode: FIRST_FILE,
+                write_time_seconds: time_given_to_the_first_unlink,
+            },
+        )
+        .expect("同一次挂载里前一次 unlink 第一个文件");
+    let created = session
+        .publish_user_change(
+            &mut devices,
+            UserChange::NewInodes {
+                count: 1,
+                write_time_seconds: time_of_the_new_inode,
+            },
+        )
+        .expect("删空的那一版上建一个 inode");
+    let new_inodes: Vec<InodeNumber> = created
+        .versions_published
+        .last()
+        .expect("建 inode 发成一版")
+        .inode_leaf_container_contents()
+        .iter()
+        .flat_map(|container| container.records.iter())
+        .map(|record| InodeNumber(record.inode))
+        .collect();
+    let [new_inode] = new_inodes[..] else {
+        panic!("删空的那一版上建一个之后树里只有新建的那一条记录：{new_inodes:?}");
+    };
+    let published = session
+        .publish_user_change(
+            &mut devices,
+            UserChange::Unlink {
+                inode: new_inode,
+                write_time_seconds: time_given_to_the_second_unlink,
+            },
+        )
+        .expect("同一次挂载里后一次 unlink 新建的那个 inode");
+    drop(devices);
+    let image = pool.memory_pool();
+    let on_disk_in_every_version_with_the_record: Vec<Vec<[u64; 3]>> = published
+        .versions_published
+        .iter()
+        .filter(|version| version_holds_the_record_of(version, new_inode))
+        .map(|version| time_seconds_of_an_inode_on_each_device(&image, version, new_inode))
+        .collect();
+    assert_eq!(
+        on_disk_in_every_version_with_the_record,
+        vec![vec![[time_given_to_the_second_unlink; 3]; 2]],
+        "后一次 unlink 的意图那一版（还带那条记录的只有这一版），两块盘上 inode {} 那条记录的 atime、mtime、ctime 秒都是后一次给的时间，\
+         不是前一次给的 {time_given_to_the_first_unlink}",
+        new_inode.0
+    );
+}
+
 /// 给 unlink 的时间超过 32 位（2^32 + 7 秒）：意图那一版与留 2、留 1 两批两块盘上那条记录的三个时间秒都是整个 64 位值。
 /// 判别力：意图写时间时按 32 位存（攻方第三节 qfu）时写下的是 7，这一条红；给的时间小于 2^32 的几条照旧绿。
 #[test]
diff --git a/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs b/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs
index fad78e4..e747217 100644
--- a/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs
+++ b/crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs
@@ -1,7 +1,11 @@
 //! unlink 单次发布的写入量不随文件长（`.claude/kb/checks-owed.md` C2（无界操作做成单事务），用户 2026-10-05 同意把 C2 的对象补上 unlink；
-//! D8（核心索引结构） 已定项 16 第 2 条「从尾部分批」）：几份单元数不同的文件（1、144、145、432 个数据单元，跨过一批 144 个的边界）
-//! 各按产品批大小（`UnlinkBatchSize::OneExtentLowerLeaf`）发意图、走完全部批，量每一次发布（意图、每一批）交给设备的写调用与字节
-//! （`TransactionOutput::writes`，发布路径逐个写调用记的账），与每一次换下的数据单元个数。
+//! D8（核心索引结构） 已定项 16 第 2 条「从尾部分批」）：几份单元数不同的文件（快档 1、144、145、432、433 个数据单元，跨过一批 144 个的边界；
+//! 慢档 2304、4608 个）各按产品批大小（`UnlinkBatchSize::OneExtentLowerLeaf`）发意图、走完全部批，量每一次发布（意图、每一批）交给设备的
+//! 写调用与字节（`TransactionOutput::writes`，发布路径逐个写调用记的账），与每一次换下的数据单元个数。
+//! 每一批换下的个数照条款算出来逐项比（意图那次 0 个，之后每批恰好 min(一批 144 个, 剩下的)），不只比「不多于一批」：
+//! 「每批取 max(144, ⌈剩下 ÷ 3⌉)」那种批数封顶的写法在 432 个单元以内与产品逐项相同，433 个单元第一批就换下 145 个
+//! （判决 `research/prompts/m3-batch19-code-r1-main-verification.md` 第二节 K1 N6 那一行，攻方 `research/prompts/m3-batch19-code-r1-opus-output.md`
+//! 「N6」一节）。
 
 mod common;
 
@@ -9,24 +13,58 @@ use common::{
     build_pool, parameters, sequential_write_in_process_as_instance_one_and_advance_the_pool,
 };
 use singlefs_core::address::{InodeNumber, InstanceGeneration};
+use singlefs_core::admission::ring_window_on_one_device;
 use singlefs_core::transaction::{
     publish_unlink_intent, publish_unlink_step, PoolWriter, TransactionOutput, FIRST_INODE_NUMBER,
 };
 use singlefs_core::unit::data_unit_payload_capacity;
 use singlefs_core::unlink::UnlinkBatchSize;
-use singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS;
+use singlefs_format::{
+    EXTENT_TREE_LOWER_LEAF_DATA_UNITS, JOURNAL_NAMED_ENTRIES_PER_RECORD, ROOT_RING_REGIONS,
+    SLOT_BYTES,
+};
 
 const FIRST_FILE: InodeNumber = InodeNumber(FIRST_INODE_NUMBER);
 
 /// 新池新建文件那个实例：意图与各批都在这个进程里发。
 const INSTANCE_OF_THE_PROCESS: InstanceGeneration = InstanceGeneration(1);
 
-/// 最大一份文件里写调用最多的那次发布，至多比一个单元的文件里写调用最多的那次多这么多个写调用（推的，逐项见下，没逐项核到代码里每一处）：
-/// 每个单元两块盘各写一份（`admission::UnitsOfAPublishToLand` 的文档：第一版每个单元落池里每一块盘），一批走的那条路径上比一个单元的文件
-/// 至多多三个节点——文件过了一片下段叶（144 个单元）extent 树多一层上段（D8（核心索引结构） 已定项 16 第 2 条的 K 取一片下段叶的单元数）、
-/// 一批换下的 144 个单元的记录与映射条目各至多多罩一片分配记录树叶与一片中央映射树叶。2 × 3 = 6。
-/// 不随文件的单元数长：432 个单元的文件与 145 个单元的文件，一批走的路径一样长、罩的叶一样多。
-const WRITE_CALLS_THE_LARGEST_PUBLISH_OF_A_LARGER_FILE_MAY_ADD_OVER_A_ONE_UNIT_FILE: u64 = 6;
+/// 测试池的盘数：每个单元每块盘各写一份（`admission::UnitsOfAPublishToLand` 的文档：第一版每个单元落池里每一块盘），
+/// journal 记录这里也按每块盘一份算（往多里算）。
+const DEVICES_OF_THE_TEST_POOL: u64 = 2;
+
+/// 任一份文件里写调用最多的那次发布，至多比一个单元的文件里写调用最多的那次多几个写调用：从代码算，不随文件的单元数长。
+/// 一次删除类发布每块盘新占的槽至多 W 个（`admission::slots_one_deletion_publish_newly_takes_on_one_device_at_most`，
+/// D28（挂载期承诺量） 已定项 1「环窗口」那一条的 W = 24 + Σ_ℓ min(N_ℓ, 2·(K + 26))，准入按它给环窗口扣空间；N_ℓ 只随盘的单元区定，
+/// K 是批大小）。这里不另抄那条式子，按这个池的分配器经 `admission::ring_window_on_one_device` 现算环窗口（= (根环槽数 + 1) × W 个槽）再除回 W。
+/// 发布是写时复制，这次写出的每个单元都是新占的落点、至少一槽，每块盘一个写调用：单元的写调用至多「盘数 × W」；
+/// 点名项一条 journal 记录装 `JOURNAL_NAMED_ENTRIES_PER_RECORD` 项，记录至多多 ⌈W ÷ 64⌉ 条、每块盘一份。
+/// 一个单元的文件那次至少也写一条记录与一个根槽，所以差至多「盘数 × (W + ⌈W ÷ 64⌉)」。
+/// 这是上界，不是钉的数：钉的数在各条用例末尾（432、433 个单元的文件最多那次 37、43 次，比一个单元的文件的 31 次多 6、12；
+/// 慢档 2304、4608 个单元的文件最多那次 45、53 次，多 14、22。多出来的几乎都在分配记录树根之下那几个节点：
+/// 按槽号按位置寻址，一批换下的记录与这次重写的单元的新旧落点散在几片叶上，片数随落点怎么排有起落，W 式子里分配记录树那一项罩着它）。
+fn write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file() -> u64 {
+    let pool = build_pool("unlink-writes-per-publish-bound-from-the-ring-window");
+    let slots_per_region = parameters().geometry.root_ring_slots_per_region;
+    let ring_window_in_slots = ring_window_on_one_device(
+        &pool.allocator,
+        slots_per_region,
+        EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
+    )
+    .0 / SLOT_BYTES;
+    let roots_in_the_ring_plus_one = ROOT_RING_REGIONS * slots_per_region.count() + 1;
+    assert_eq!(
+        ring_window_in_slots % roots_in_the_ring_plus_one,
+        0,
+        "环窗口是 (根环槽数 + 1) × W 个槽"
+    );
+    let slots_one_deletion_publish_newly_takes_on_one_device_at_most =
+        ring_window_in_slots / roots_in_the_ring_plus_one;
+    DEVICES_OF_THE_TEST_POOL
+        * (slots_one_deletion_publish_newly_takes_on_one_device_at_most
+            + slots_one_deletion_publish_newly_takes_on_one_device_at_most
+                .div_ceil(JOURNAL_NAMED_ENTRIES_PER_RECORD))
+}
 
 /// unlink 的一次发布交给设备的写，与这次换下的数据单元个数。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -120,47 +158,78 @@ fn largest_write_calls_of_one_publish(writes: &[WritesOfOnePublish]) -> u64 {
         .expect("意图那次至少一次发布")
 }
 
-/// 四份文件（1、144、145、432 个数据单元）：
-/// ① 每一次发布换下的数据单元不多于一批（144 个，`EXTENT_TREE_LOWER_LEAF_DATA_UNITS`）；
-/// ② 每份文件里写调用最多的那次发布，至多比一个单元的文件里的多 [`WRITE_CALLS_THE_LARGEST_PUBLISH_OF_A_LARGER_FILE_MAY_ADD_OVER_A_ONE_UNIT_FILE`]；
-/// ③ 每一次发布的写调用钉成绝对值（发布路径今天写出的形状：一个单元的文件意图 31、最后一批 25；432 个单元的文件意图 33、三批 37、35、33）。
-/// 判别力：批大小改成整个文件一批（`UnlinkBatchSize::data_units_per_batch` 的产品那一臂取 `u32::MAX`）时，145 与 432 个单元的文件
-/// 意图之后只有一批、换下 145 与 432 个单元，①③ 红。这几个规模上整个文件一批的单次写调用不比分批的多（副本里量过：432 个单元的文件
-/// 那一批 29 次，分批时最多 37 次），② 判不出这一刀；要到 2304 个单元以上整个文件一批的单次写调用才超过分批的（副本里 release 量过），
-/// 那几个规模单跑十几秒到几十秒，没进这一条。
+/// 每一次发布换下的数据单元个数照条款该是多少（D8（核心索引结构） 已定项 16 第 2 条「从尾部分批」，产品每批 K 取一片下段叶的单元数
+/// `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`）：意图那次 0 个，之后每批恰好 min(K, 剩下的)，剩下 0 个为止。按先后，第一项是意图那次。
+fn data_units_released_by_every_publish_as_the_clause_gives(
+    data_units_of_the_file: usize,
+) -> Vec<usize> {
+    let data_units_per_batch =
+        usize::try_from(EXTENT_TREE_LOWER_LEAF_DATA_UNITS).expect("144 装得进 usize");
+    let mut released_by_every_publish = vec![0];
+    let mut data_units_remaining = data_units_of_the_file;
+    // 迭代上界：每一轮剩下的至少少 1（K ≥ 1），⌈单元数 ÷ K⌉ 轮走到 0。
+    while data_units_remaining > 0 {
+        let released_by_this_batch = data_units_remaining.min(data_units_per_batch);
+        released_by_every_publish.push(released_by_this_batch);
+        data_units_remaining -= released_by_this_batch;
+    }
+    released_by_every_publish
+}
+
+fn data_units_released_by_every_publish(writes: &[WritesOfOnePublish]) -> Vec<usize> {
+    writes
+        .iter()
+        .map(|publish| publish.data_units_released)
+        .collect()
+}
+
+/// 五份文件（1、144、145、432、433 个数据单元）：
+/// ① 每一次发布换下的数据单元个数与条款算出来的逐项相同（意图 0 个，之后每批恰好 min(144, 剩下的)）；
+/// ② 每份文件里写调用最多的那次发布，至多比一个单元的文件里的多 [`write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file`]
+/// （从 W 式子算，不随文件长）；
+/// ③ 每一次发布的写调用钉成绝对值（发布路径今天写出的形状：一个单元的文件意图 31、最后一批 25；432 个单元的文件意图 33、三批 37、35、33；
+/// 433 个单元的文件四批里第二批 43）。
+/// 判别力：批大小改成整个文件一批（`UnlinkBatchSize::data_units_per_batch` 的产品那一臂取 `u32::MAX`）时，145、432、433 个单元的文件
+/// 意图之后只有一批、换下整个文件，①③ 红。这几个规模上整个文件一批的单次写调用不比分批的多（副本里量过：432 个单元的文件
+/// 那一批 29 次，分批时最多 37 次），② 判不出这一刀（② 是 W 式子的上界，只拦超出准入预留的那一种）；2304、4608 个单元那两点在慢档
+/// [`unlink_of_files_of_sixteen_and_thirty_two_batches_releases_exactly_one_batch_or_the_rest_per_publish_and_its_largest_publish_stays_within_a_constant_of_a_one_unit_file`]。
+/// 每批取 max(144, ⌈剩下 ÷ 3⌉) 时 433 个单元的文件第一批换下 145 个，① 红（432 个单元以内与产品逐项相同）。
 // harness-test-granularity:one-scenario 斜率要几份不同单元数的文件一起比，拆开就没有斜率
 #[test]
-fn unlink_of_files_of_one_to_three_batches_releases_at_most_one_batch_per_publish_and_its_largest_publish_does_not_grow_with_the_file(
+fn unlink_of_files_of_one_to_four_batches_releases_exactly_one_batch_or_the_rest_per_publish_and_its_largest_publish_does_not_grow_with_the_file(
 ) {
     let one_data_unit = writes_of_every_publish_of_an_unlink_in_product_batches(1);
     let one_batch = writes_of_every_publish_of_an_unlink_in_product_batches(144);
     let one_batch_and_one_unit = writes_of_every_publish_of_an_unlink_in_product_batches(145);
     let three_batches = writes_of_every_publish_of_an_unlink_in_product_batches(432);
+    let three_batches_and_one_unit = writes_of_every_publish_of_an_unlink_in_product_batches(433);
     let every_file = [
         (1, &one_data_unit),
         (144, &one_batch),
         (145, &one_batch_and_one_unit),
         (432, &three_batches),
+        (433, &three_batches_and_one_unit),
     ];
     for (data_units_of_the_file, writes) in every_file {
         println!("{data_units_of_the_file} 个数据单元的文件，每一次发布（意图、各批）：{writes:?}");
     }
     for (data_units_of_the_file, writes) in every_file {
-        assert!(
-            writes.iter().all(|publish| u64::try_from(publish.data_units_released)
-                .expect("单元数装得进 u64")
-                <= EXTENT_TREE_LOWER_LEAF_DATA_UNITS),
-            "{data_units_of_the_file} 个数据单元的文件，每一次发布换下的数据单元不多于一批 {EXTENT_TREE_LOWER_LEAF_DATA_UNITS} 个：{writes:?}"
+        assert_eq!(
+            data_units_released_by_every_publish(writes),
+            data_units_released_by_every_publish_as_the_clause_gives(data_units_of_the_file),
+            "{data_units_of_the_file} 个数据单元的文件，意图那次换下 0 个，之后每批恰好换下 min({EXTENT_TREE_LOWER_LEAF_DATA_UNITS}, 剩下的) 个"
         );
     }
     let largest_of_the_one_unit_file = largest_write_calls_of_one_publish(&one_data_unit);
+    let write_calls_it_may_add =
+        write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file();
+    println!("W 式子算的上界：写调用最多的那次至多比一个单元的文件多 {write_calls_it_may_add} 次");
     for (data_units_of_the_file, writes) in every_file {
         assert!(
             largest_write_calls_of_one_publish(writes)
-                <= largest_of_the_one_unit_file
-                    + WRITE_CALLS_THE_LARGEST_PUBLISH_OF_A_LARGER_FILE_MAY_ADD_OVER_A_ONE_UNIT_FILE,
+                <= largest_of_the_one_unit_file + write_calls_it_may_add,
             "{data_units_of_the_file} 个数据单元的文件，写调用最多的那次发布 {} 次，至多比一个单元的文件的 {largest_of_the_one_unit_file} 次多 \
-             {WRITE_CALLS_THE_LARGEST_PUBLISH_OF_A_LARGER_FILE_MAY_ADD_OVER_A_ONE_UNIT_FILE} 次",
+             {write_calls_it_may_add} 次（W 式子算的上界）",
             largest_write_calls_of_one_publish(writes)
         );
     }
@@ -170,13 +239,69 @@ fn unlink_of_files_of_one_to_three_batches_releases_at_most_one_batch_per_publis
             write_calls_of_every_publish(&one_batch),
             write_calls_of_every_publish(&one_batch_and_one_unit),
             write_calls_of_every_publish(&three_batches),
+            write_calls_of_every_publish(&three_batches_and_one_unit),
         ],
         [
             vec![31, 25],
             vec![31, 29],
             vec![31, 31, 29],
             vec![33, 37, 35, 33],
+            vec![33, 39, 43, 39, 29],
+        ],
+        "1、144、145、432、433 个数据单元的文件，每一次发布（意图、各批）的写调用"
+    );
+}
+
+/// 慢档：2304、4608 个数据单元的文件（16 批、32 批，判决 `research/prompts/m3-batch19-code-r1-main-verification.md` 第三节第 3 条「慢档 2304、4608
+/// 两点量斜率」），与一个单元的文件比：
+/// ① 每一次发布换下的数据单元个数与条款算出来的逐项相同（意图 0 个，之后每批恰好 144 个）；
+/// ② 写调用最多的那次发布，至多比一个单元的文件里的多 [`write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file`]
+/// （从 W 式子算，与文件的单元数无关）；
+/// ③ 两份文件写调用最多的那次钉成绝对值：2304 个单元 45 次、4608 个单元 53 次（一个单元的文件 31 次）。
+/// 判别力：每批取 max(144, ⌈剩下 ÷ 3⌉) 时两份文件意图之后只有三批、第一批换下 768 与 1536 个，① 红；批大小改成整个文件一批时同样 ① 红。
+/// 一批重写的节点随文件长（例如每批把 extent 树下段或中央映射树整棵重写）而批数不变的写法，③ 红；长到超过 W 式子罩得住的，② 也红。
+#[test]
+#[ignore = "harness 慢档用例：两份几千个数据单元的文件各顺序写一次、再分十几到三十几批删完；判决点名放慢档，debug 下单跑量过 28、55、63 秒（随机器负载）"]
+fn unlink_of_files_of_sixteen_and_thirty_two_batches_releases_exactly_one_batch_or_the_rest_per_publish_and_its_largest_publish_stays_within_a_constant_of_a_one_unit_file(
+) {
+    let one_data_unit = writes_of_every_publish_of_an_unlink_in_product_batches(1);
+    let sixteen_batches = writes_of_every_publish_of_an_unlink_in_product_batches(2304);
+    let thirty_two_batches = writes_of_every_publish_of_an_unlink_in_product_batches(4608);
+    let every_file = [
+        (1, &one_data_unit),
+        (2304, &sixteen_batches),
+        (4608, &thirty_two_batches),
+    ];
+    for (data_units_of_the_file, writes) in every_file {
+        println!("{data_units_of_the_file} 个数据单元的文件，每一次发布（意图、各批）：{writes:?}");
+    }
+    for (data_units_of_the_file, writes) in every_file {
+        assert_eq!(
+            data_units_released_by_every_publish(writes),
+            data_units_released_by_every_publish_as_the_clause_gives(data_units_of_the_file),
+            "{data_units_of_the_file} 个数据单元的文件，意图那次换下 0 个，之后每批恰好换下 min({EXTENT_TREE_LOWER_LEAF_DATA_UNITS}, 剩下的) 个"
+        );
+    }
+    let largest_of_the_one_unit_file = largest_write_calls_of_one_publish(&one_data_unit);
+    let write_calls_it_may_add =
+        write_calls_the_largest_publish_of_any_file_may_add_over_a_one_unit_file();
+    println!("W 式子算的上界：写调用最多的那次至多比一个单元的文件多 {write_calls_it_may_add} 次");
+    for (data_units_of_the_file, writes) in every_file {
+        assert!(
+            largest_write_calls_of_one_publish(writes)
+                <= largest_of_the_one_unit_file + write_calls_it_may_add,
+            "{data_units_of_the_file} 个数据单元的文件，写调用最多的那次发布 {} 次，至多比一个单元的文件的 {largest_of_the_one_unit_file} 次多 \
+             {write_calls_it_may_add} 次（W 式子算的上界）",
+            largest_write_calls_of_one_publish(writes)
+        );
+    }
+    assert_eq!(
+        [
+            largest_write_calls_of_one_publish(&one_data_unit),
+            largest_write_calls_of_one_publish(&sixteen_batches),
+            largest_write_calls_of_one_publish(&thirty_two_batches),
         ],
-        "1、144、145、432 个数据单元的文件，每一次发布（意图、各批）的写调用"
+        [31, 45, 53],
+        "1、2304、4608 个数据单元的文件，写调用最多的那次发布"
     );
 }
````

## 实现员 r：mutations-append.tsv（/tmp/claude-1000/m3-batch21/r/patch/mutations-append.tsv）

````text
删除批第 21 批 N3：可写挂载在第一次暖机之后就放开重建分配器时扣住的那一截，预演对真发那一判不比（等于取号之前的预演也在那里放开，攻方 m3-batch19-code-r1 臂 6），暖机推两次时第二次暖机写进卸载之前现行那一版换下的槽	crates/singlefs-core/src/mount.rs	        warm_up_publishes.push(next.clone());\n        current = next;\n    }\n    // 暖机走完、本实例的根在每块盘上都落了（`warm_up_publish_txgs` 按根环落点公式推到覆盖每块盘为止），重建分配器时扣住的那一截放开\n    // （`rebuilt_allocator`，用户 2026-10-04 选甲）：此后丢哪块盘，恢复择到的都不低于本实例的根，它们不引用这一截。\n    // 放开的只有那一截：扣住位只有两处会置——重建分配器那一处与抬 F 那一串（`raise_rollback_floor`），这次挂载的抬 F 都排在这一句之后\n    // （写行之后的准入 `push_floor_raises_after_the_row_publish`、续做里推的那几串），它们置的扣住位归它们自己放（`raise_rollback_floor`）；\n    // 走到这里分配器上的扣住位全是重建那一处置的。暖机没走完（取号之前被拒、写行或暖机落盘途中报错）时挂载整次报错，分配器随错丢掉，\n    // 扣住的那一截不交给任何会话。\n    allocator.release_reclaim_holds();\n    // 取号之前在拷贝上取的落点就是真发起来取的：同一个分配器、同一串分配动作。这一串里有一份换下的单元读盘核校验和对不上时不比——\n    // 真发时那一份的分配记录留在已分配，拷贝上没做那一步、照常释放，那一槽若在这一串里被回收（崩溃恢复落到环里最旧的那条根、其余全被抛弃），两边可以不同\n    // （`dry_run_of_the_publishes_after_acquisition` 的文档注释）；那一格取号之前判的与真发的不是同一串，是这一版留着的缺口。\n    let placements_taken: Vec<PlacementsTakenByOnePublish> = std::iter::once(&row_publish)\n        .chain(&warm_up_publishes)\n        .map(placements_taken_by)\n        .collect();\n    let some_copy_was_quarantined = std::iter::once(&row_publish)\n        .chain(&warm_up_publishes)\n        .filter_map(PoolVersion::version_of_trees)\n        .any(|version| {\n            !version\n                .quarantined_after_release_checksum_mismatch\n                .is_empty()\n        });\n    if let (Some(planned), false) = (&placements_planned, some_copy_was_quarantined) {	        warm_up_publishes.push(next.clone());\n        if warm_up_publishes.len() == 1 {\n            allocator.release_reclaim_holds();\n        }\n        current = next;\n    }\n    // 暖机走完、本实例的根在每块盘上都落了（`warm_up_publish_txgs` 按根环落点公式推到覆盖每块盘为止），重建分配器时扣住的那一截放开\n    // （`rebuilt_allocator`，用户 2026-10-04 选甲）：此后丢哪块盘，恢复择到的都不低于本实例的根，它们不引用这一截。\n    // 放开的只有那一截：扣住位只有两处会置——重建分配器那一处与抬 F 那一串（`raise_rollback_floor`），这次挂载的抬 F 都排在这一句之后\n    // （写行之后的准入 `push_floor_raises_after_the_row_publish`、续做里推的那几串），它们置的扣住位归它们自己放（`raise_rollback_floor`）；\n    // 走到这里分配器上的扣住位全是重建那一处置的。暖机没走完（取号之前被拒、写行或暖机落盘途中报错）时挂载整次报错，分配器随错丢掉，\n    // 扣住的那一截不交给任何会话。\n    allocator.release_reclaim_holds();\n    // 取号之前在拷贝上取的落点就是真发起来取的：同一个分配器、同一串分配动作。这一串里有一份换下的单元读盘核校验和对不上时不比——\n    // 真发时那一份的分配记录留在已分配，拷贝上没做那一步、照常释放，那一槽若在这一串里被回收（崩溃恢复落到环里最旧的那条根、其余全被抛弃），两边可以不同\n    // （`dry_run_of_the_publishes_after_acquisition` 的文档注释）；那一格取号之前判的与真发的不是同一串，是这一版留着的缺口。\n    let placements_taken: Vec<PlacementsTakenByOnePublish> = std::iter::once(&row_publish)\n        .chain(&warm_up_publishes)\n        .map(placements_taken_by)\n        .collect();\n    let some_copy_was_quarantined = std::iter::once(&row_publish)\n        .chain(&warm_up_publishes)\n        .filter_map(PoolVersion::version_of_trees)\n        .any(|version| {\n            !version\n                .quarantined_after_release_checksum_mismatch\n                .is_empty()\n        });\n    if let (Some(planned), false) = (&placements_planned, some_copy_was_quarantined || true) {	-p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads -- after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times	after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_in_either_warm_up
删除批第 21 批 N3：可写挂载只在真发那一处第一次暖机之后就放开、取号之前的预演照旧暖机落满才放开（攻方 m3-batch19-code-r1 臂 5），暖机推两次时挂载在预演对真发那一判上 panic	crates/singlefs-core/src/mount.rs	        warm_up_publishes.push(next.clone());\n        current = next;\n    }\n	        warm_up_publishes.push(next.clone());\n        if warm_up_publishes.len() == 1 {\n            allocator.release_reclaim_holds();\n        }\n        current = next;\n    }\n	-p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads -- after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times	after_a_crashed_unmount_of_a_narrow_pool_overwritten_three_times_the_next_writable_mount_warming_up_twice_writes_no_unit_into_a_slot_the_version_before_the_unmount_released_in_either_warm_up
删除批第 21 批 N4：顺序写那一版的根把 F 带成上一版的 txg（不推抬 F 那一串、不写系统配置就挪 F），挂载交回之后第一次写照样写成、只有 F 那一样不同	crates/singlefs-core/src/transaction.rs	            tree_identifier_watermark: previous.root.tree_identifier_watermark,\n            rollback_floor: previous.root.rollback_floor,\n        },\n        Some(previous),\n    )\n}\n\n/// 建 N 个 inode	            tree_identifier_watermark: previous.root.tree_identifier_watermark,\n            rollback_floor: previous.root.checkpoint_txg,\n        },\n        Some(previous),\n    )\n}\n\n/// 建 N 个 inode	-p singlefs-harness --test the_mount_after_a_crashed_unmount_keeps_what_the_other_device_reads -- after_a_crashed_unmount_of_a_narrow_pool_whose_file_was_shrunk	after_a_crashed_unmount_of_a_narrow_pool_whose_file_was_shrunk_the_first_write_of_the_next_writable_mount_regrows_it_without_raising_the_floor
删除批第 21 批 Q-F：会话那一层只对 unlink 只进不退（unlink 给意图的时间取给的与这个线程上一次会话 unlink 用过的时间里较大的那个，攻方 m3-batch19-code-r1 q-unlink-mono），同一次挂载里两次 unlink、后一次时间更早	crates/singlefs-core/src/mounted_session.rs	            inode,\n            write_time_seconds,\n            instance,\n        ),	            inode,\n            {\n                thread_local! {\n                    static LATEST_TIME_GIVEN_TO_AN_UNLINK_IN_THIS_THREAD: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };\n                }\n                LATEST_TIME_GIVEN_TO_AN_UNLINK_IN_THIS_THREAD.with(|latest| {\n                    let clamped = latest.get().max(write_time_seconds);\n                    latest.set(clamped);\n                    clamped\n                })\n            },\n            instance,\n        ),	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version -- session_unlink	two_session_unlinks_in_the_same_mount_the_second_given_an_earlier_time_write_the_second_given_time_into_the_three_time_fields_of_its_intent_on_disk
删除批第 21 批 Q-F qfm：会话里时间只进不退的单处写法——这个线程上建 inode 给过的最晚时间记着，unlink 给意图的时间取给的与它里较大的那个（攻方 m3-batch17-code-r1 qfm 只记建 inode 那一半；取代第 19 批那一行「取现行那一版各条记录最晚的写入时间」，那一行另被早于上一版的会话用例杀），同一次挂载里先建一个更晚的 inode 再 unlink	crates/singlefs-core/src/mounted_session.rs	    match change {\n        UserChange::Unlink {\n            inode,\n            write_time_seconds,\n        } => publish_unlink_intent(\n            &mut writer,\n            allocator,\n            previous,\n            inode,\n            write_time_seconds,\n            instance,\n        ),\n        UserChange::Overwrite(file) => {\n            publish_overwrite(&mut writer, allocator, previous, file, instance)\n        }\n        UserChange::SequentialWrite(file) => {\n            publish_sequential_write(&mut writer, allocator, previous, file, instance)\n        }\n        UserChange::NewInodes {\n            count,\n            write_time_seconds,\n        } => publish_new_inodes(\n            &mut writer,\n            allocator,\n            previous,\n            count,\n            write_time_seconds,\n            instance,\n        ),\n    }\n	    thread_local! {\n        static LATEST_TIME_GIVEN_TO_NEW_INODES_IN_THIS_THREAD: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };\n    }\n    match change {\n        UserChange::Unlink {\n            inode,\n            write_time_seconds,\n        } => publish_unlink_intent(\n            &mut writer,\n            allocator,\n            previous,\n            inode,\n            write_time_seconds.max(LATEST_TIME_GIVEN_TO_NEW_INODES_IN_THIS_THREAD.with(std::cell::Cell::get)),\n            instance,\n        ),\n        UserChange::Overwrite(file) => {\n            publish_overwrite(&mut writer, allocator, previous, file, instance)\n        }\n        UserChange::SequentialWrite(file) => {\n            publish_sequential_write(&mut writer, allocator, previous, file, instance)\n        }\n        UserChange::NewInodes {\n            count,\n            write_time_seconds,\n        } => publish_new_inodes(\n            {\n                LATEST_TIME_GIVEN_TO_NEW_INODES_IN_THIS_THREAD\n                    .with(|latest| latest.set(latest.get().max(write_time_seconds)));\n                &mut writer\n            },\n            allocator,\n            previous,\n            count,\n            write_time_seconds,\n            instance,\n        ),\n    }\n	-p singlefs-harness --test unlink_mount_continuation_and_the_emptied_version -- session_unlink	a_session_unlink_after_a_new_inode_with_a_later_time_in_the_same_mount_writes_the_given_time_into_the_three_time_fields_of_the_intent_and_of_every_batch_on_disk
删除批第 21 批 C2：unlink 的产品批大小取 max(144, ⌈剩下 ÷ 3⌉)（一次 unlink 至多三批的捷径，攻方 m3-batch19-code-r1 c2-third），433 个单元的文件第一批换下 145 个	crates/singlefs-core/src/unlink.rs	    let data_units_per_batch = batch_size.data_units_per_batch();\n    if data_units_remaining <= data_units_per_batch {	    let data_units_per_batch = match batch_size {\n        UnlinkBatchSize::OneExtentLowerLeaf => batch_size\n            .data_units_per_batch()\n            .max(data_units_remaining.div_ceil(3)),\n        UnlinkBatchSize::CappedForTests(_) => batch_size.data_units_per_batch(),\n    };\n    if data_units_remaining <= data_units_per_batch {	-p singlefs-harness --test unlink_writes_per_transaction_do_not_grow_with_the_file -- unlink_of_files_of_one_to_four_batches	unlink_of_files_of_one_to_four_batches_releases_exactly_one_batch_or_the_rest_per_publish_and_its_largest_publish_does_not_grow_with_the_file
删除批第 21 批 C2 慢档：unlink 的产品批大小取 max(144, ⌈剩下 ÷ 3⌉)（攻方 m3-batch19-code-r1 c2-third），2304、4608 个单元的文件第一批换下 768、1536 个	crates/singlefs-core/src/unlink.rs	    let data_units_per_batch = batch_size.data_units_per_batch();\n    if data_units_remaining <= data_units_per_batch {	    let data_units_per_batch = match batch_size {\n        UnlinkBatchSize::OneExtentLowerLeaf => batch_size\n            .data_units_per_batch()\n            .max(data_units_remaining.div_ceil(3)),\n        UnlinkBatchSize::CappedForTests(_) => batch_size.data_units_per_batch(),\n    };\n    if data_units_remaining <= data_units_per_batch {	-p singlefs-harness --test unlink_writes_per_transaction_do_not_grow_with_the_file -- --include-ignored unlink_of_files_of_sixteen_and_thirty_two_batches	unlink_of_files_of_sixteen_and_thirty_two_batches_releases_exactly_one_batch_or_the_rest_per_publish_and_its_largest_publish_stays_within_a_constant_of_a_one_unit_file
C479 变体 c：I-9.6 墓碑那一半不比水位（水位压回墓碑最大对象号之下再建一个，走到的根里 inode 树最大 key 那一半成立，判不出）	crates/singlefs-checker/src/walk.rs	watermark > largest_tombstone_object	true	-p singlefs-harness --test inode_number_watermark_pressed_back_reissues_a_number -- creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone	creating_an_inode_after_the_watermark_is_pressed_back_below_the_largest_tombstone_reddens_the_tombstone_half_of_the_watermark_invariant
````

## 实现员 r：mutations-replacements.tsv（/tmp/claude-1000/m3-batch21/r/patch/mutations-replacements.tsv）

````text
删除批第 19 批 C2：unlink 的产品批大小改成整个文件一批（data_units_per_batch 的产品那一臂取 u32::MAX），一次发布换下整个文件的数据单元	crates/singlefs-core/src/unlink.rs	            UnlinkBatchSize::OneExtentLowerLeaf => EXTENT_TREE_LOWER_LEAF_DATA_UNITS,	            UnlinkBatchSize::OneExtentLowerLeaf => u32::MAX as u64,	-p singlefs-harness --test unlink_writes_per_transaction_do_not_grow_with_the_file	unlink_of_files_of_one_to_four_batches_releases_exactly_one_batch_or_the_rest_per_publish_and_its_largest_publish_does_not_grow_with_the_file
````

## 实现员 r：mutations-delete.txt（/tmp/claude-1000/m3-batch21/r/patch/mutations-delete.txt）

````text
删除批第 19 批 Q-F qfm：会话那一层 unlink 给意图的时间取给的与现行那一版里各条 inode 记录最晚的写入时间里较大的那个（会话里时间只进不退的单处写法，攻方 m3-batch17-code-r1 qfm），同一次挂载里先建一个更晚的 inode 再 unlink
````
