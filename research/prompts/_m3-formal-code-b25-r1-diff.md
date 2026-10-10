# 第 25 批代码轮第一轮：改动全文（2026-10-05）

## 一、25-1：`git diff HEAD`（HEAD + 25-1 补丁与主仓逐字节相同，主 agent 核过）

````diff
diff --git a/crates/singlefs-harness/src/model.rs b/crates/singlefs-harness/src/model.rs
index 830cd586..05035ac5 100644
--- a/crates/singlefs-harness/src/model.rs
+++ b/crates/singlefs-harness/src/model.rs
@@ -17,6 +17,7 @@
 //! 按位置寻址之后（D8（核心索引结构） 已定项 14）没有「一个节点装不下」那道墙，模型也就没有那条拒绝理由。
 
 use std::collections::{BTreeMap, BTreeSet};
+use std::num::NonZeroU64;
 use std::rc::Rc;
 
 use singlefs_format::{
@@ -24,10 +25,11 @@ use singlefs_format::{
     ALLOCATION_RECORD_TREE_LEAF_SLOTS, CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES,
     DATA_UNIT_PAYLOAD_OFFSET, EXTENT_TREE_INTERNAL_FANOUT, EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
     INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE, INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT,
-    INSTANCE_TABLE_PAGE_RECORDS, JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT,
-    LINEAGE_CONTAINER_RECORDS, MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES, NODE_BYTES,
-    NODE_POINTER_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES, ROOT_RING_REGIONS,
-    ROOT_RING_REGION_DEVICES, ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM, SLOT_BYTES,
+    INODE_INTERNAL_ENTRY, INODE_LEAF_RECORDS, INSTANCE_TABLE_PAGE_RECORDS,
+    JOURNAL_RING_DEFAULT_BYTES, JOURNAL_RING_START_SLOT, LINEAGE_CONTAINER_RECORDS,
+    MAPPING_ENTRY_BYTES, MAPPING_KEY_BYTES, NODE_BYTES, NODE_POINTER_BYTES,
+    NONCE_MAC_ALGORITHM_RESERVED_BYTES, ROOT_RING_REGIONS, ROOT_RING_REGION_DEVICES,
+    ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM, SLOT_BYTES,
 };
 
 /// 模型里的 checkpoint 号（D16（发布语义） 已定项 6：每次发布 + 1）。与实现的 `CheckpointTxg` 各自声明（D13（验证路线） 已定项 5）。
@@ -62,12 +64,31 @@ const MAKE_FILESYSTEM_TXG: ModelCheckpointTxg = ModelCheckpointTxg(0);
 const POOL_WIDE_ACCOUNTING_ROWS: u64 = 3;
 const ACCOUNTING_ROWS_PER_DEVICE: u64 = 6;
 
+/// inode 树里最左那片之外的一片叶容器：它在树里左起第几片（从 0 数，最左那片是 0，这里恒不小于 1）。
+/// 只经 [`ModelUnitRole::inode_leaf_container_at`] 造，第 0 片造不出这个类型（那一片是 [`ModelUnitRole::InodeLeaf`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
+pub struct ModelInodeLeafContainerRightOfTheLeftmost {
+    position_from_the_leftmost: NonZeroU64,
+}
+
+impl ModelInodeLeafContainerRightOfTheLeftmost {
+    /// 它在树里左起第几片（从 0 数，恒不小于 1）。
+    #[must_use]
+    pub fn position_from_the_leftmost(self) -> u64 {
+        self.position_from_the_leftmost.get()
+    }
+}
+
 /// 一版里的一个角色：第一个文件版本的八个单元（D3（空间分配） 已定项 10 ⑤ 的次序）与实例表单元（D18（块里携带什么信息） 已定项 11）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
 pub enum ModelUnitRole {
     Data,
     ExtentRoot,
+    /// inode 树最左那片叶容器：第一个文件那条记录住它，删完之后建的 inode 前 233 条也住它。
     InodeLeaf,
+    /// inode 树最左那片之外的每一片叶容器各是一个角色（删完之后建满 233 条起才有，D8（核心索引结构） 已定项 6：插入落在最右那片、
+    /// 满了在末尾分裂、左半不动）：各片的分配代不同（只有新记录落的那一片这次重写），模型逐片记、逐片比。
+    InodeLeafRightOfTheLeftmost(ModelInodeLeafContainerRightOfTheLeftmost),
     InodeRoot,
     AllocationTree,
     AccountingTree,
@@ -83,6 +104,20 @@ pub enum ModelUnitRole {
 }
 
 impl ModelUnitRole {
+    /// inode 树左起第 `position_from_the_leftmost` 片叶容器（从 0 数）的角色：第 0 片是 [`ModelUnitRole::InodeLeaf`]，
+    /// 之后每一片是 [`ModelUnitRole::InodeLeafRightOfTheLeftmost`]。
+    #[must_use]
+    pub fn inode_leaf_container_at(position_from_the_leftmost: u64) -> ModelUnitRole {
+        match NonZeroU64::new(position_from_the_leftmost) {
+            None => ModelUnitRole::InodeLeaf,
+            Some(position_from_the_leftmost) => ModelUnitRole::InodeLeafRightOfTheLeftmost(
+                ModelInodeLeafContainerRightOfTheLeftmost {
+                    position_from_the_leftmost,
+                },
+            ),
+        }
+    }
+
     /// 占几个槽：数据单元与打包记录单元（inode 叶、实例表）恒 32 KiB（D4（校验和位置） 已定项 5），索引节点恒 16 KiB
     /// （D8（核心索引结构） 已定项 2），落点粒度 16 KiB（D3（空间分配） 已定项 7）。
     #[must_use]
@@ -90,6 +125,7 @@ impl ModelUnitRole {
         match self {
             ModelUnitRole::Data
             | ModelUnitRole::InodeLeaf
+            | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
             | ModelUnitRole::InstanceTable
             | ModelUnitRole::TombstoneLeaf => DATA_UNIT_BYTES / SLOT_BYTES,
             ModelUnitRole::ExtentRoot
@@ -137,9 +173,35 @@ enum ModelPublishKind {
     /// 同一次发布；重写 extent 零条目根与 inode 零条目根，加四个固定点单元。
     FinalDeletionBatch,
     /// 建一个 size 0 的 inode（`transaction::publish_new_inodes`，count 1；D8（核心索引结构） 已定项 16 第 5 条：删完之后只许建 inode、
-    /// 写行与空发布）：inode 叶与根（新记录落在唯一那片叶容器里），加四个固定点单元；数据单元与 extent 树根照抄（新建的 inode 长度 0）。
+    /// 写行与空发布），新记录落在最左那片叶容器里（删完之后建的前 233 条，[`inode_leaf_container_position_of_a_new_record`]）：
+    /// inode 叶与根，加四个固定点单元；数据单元与 extent 树根照抄（新建的 inode 长度 0）。
     /// 执行器只在第一个文件已删完的一版上调（`history.rs` 的 `HistoryOperation::CreateInode`）。
     NewInode,
+    /// 同 [`ModelPublishKind::NewInode`]，新记录落在最左那片之外的那一片里（删完之后建的第 234 条起）：只重写那一片与 inode 根，
+    /// 左边那几片一个字节都不动、分配代不变（D8（核心索引结构） 已定项 6：分裂点取末尾，左半保留全部原有记录与身份；
+    /// 「更新一条记录 = COW 叶 + 全部祖先」）。
+    NewInodeInALeafContainerRightOfTheLeftmost(ModelInodeLeafContainerRightOfTheLeftmost),
+}
+
+/// inode 号的宽：内部节点条目里的分隔 key 8 字节（D8（核心索引结构） 已定项 6：条目 = 分隔 key 8 + 身份引用 26 + 子指针 86）。
+const INODE_TREE_KEY_BYTES: u64 = 8;
+
+/// 删完第一个文件之后建的第 `records_before` 条（从 0 数，也就是这次之前树里已有几条）落在 inode 树左起第几片叶容器（从 0 数）。
+/// D8（核心索引结构） 已定项 6：插入永远落在最右的叶，满 233 条（`INODE_LEAF_RECORDS`）就在末尾分裂、左半不动；模型罩的历史里
+/// 这棵树删完第一个文件之后是空的、之后只经建 inode 往右追加，所以左边每片都装满 233 条、第 k 条落在第 ⌊k ÷ 233⌋ 片。
+///
+/// # Panics
+/// 落点超出一个码 2 根装得下的叶容器数（扇出 135，D8（核心索引结构） 已定项 6）：根要长一层，实现今天在那里拒
+/// （`MoreLeafContainersThanOneRootNodeHolds`），模型罩的历史建不到三万多个 inode。
+fn inode_leaf_container_position_of_a_new_record(records_before: u64) -> u64 {
+    let position_from_the_leftmost = records_before / INODE_LEAF_RECORDS;
+    let leaf_containers_one_root_holds =
+        index_node_entry_capacity(INODE_TREE_KEY_BYTES, INODE_INTERNAL_ENTRY);
+    assert!(
+        position_from_the_leftmost < leaf_containers_one_root_holds,
+        "模型今天只罩 inode 树根之下直接挂叶容器的历史（一个根装 {leaf_containers_one_root_holds} 片）：第 {records_before} 条落在第 {position_from_the_leftmost} 片"
+    );
+    position_from_the_leftmost
 }
 
 /// 装得下 `rows` 行要几片（D18（块里携带什么信息） 已定项 11：一片 370 条记录，链指针记录恒为一片的最后一条、数据行 369；
@@ -167,7 +229,8 @@ impl ModelPublishKind {
             | ModelPublishKind::UnlinkIntent
             | ModelPublishKind::DeletionBatchKeepingDataUnits
             | ModelPublishKind::FinalDeletionBatch
-            | ModelPublishKind::NewInode => false,
+            | ModelPublishKind::NewInode
+            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => false,
         }
     }
 
@@ -182,7 +245,8 @@ impl ModelPublishKind {
             ModelPublishKind::UnlinkIntent
             | ModelPublishKind::DeletionBatchKeepingDataUnits
             | ModelPublishKind::FinalDeletionBatch
-            | ModelPublishKind::NewInode => 1,
+            | ModelPublishKind::NewInode
+            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => 1,
             ModelPublishKind::RowsOnFileVersion
             | ModelPublishKind::EmptyOnFileVersion
             | ModelPublishKind::RowsOnVersionWithoutFile
@@ -206,13 +270,37 @@ impl ModelPublishKind {
             | ModelPublishKind::ZeroUnit
             | ModelPublishKind::UnlinkIntent
             | ModelPublishKind::DeletionBatchKeepingDataUnits
-            | ModelPublishKind::NewInode => &[],
+            | ModelPublishKind::NewInode
+            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => &[],
         }
     }
 
-    /// 这次发布重写的角色。分配记录树（`ModelUnitRole::AllocationTree`）在这里只记它的根：根之下重写几个节点按位置寻址现算上界
+    /// 这次发布重写的角色：种类定死的那几个（[`ModelPublishKind::rewritten_roles_fixed_by_the_kind`]），建 inode 落在最左那片之外时
+    /// 再加新记录落的那一片。
+    fn rewritten_roles(self) -> Vec<ModelUnitRole> {
+        let mut roles = self.rewritten_roles_fixed_by_the_kind().to_vec();
+        match self {
+            ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(leaf_container) => {
+                roles.push(ModelUnitRole::InodeLeafRightOfTheLeftmost(leaf_container));
+            }
+            ModelPublishKind::FirstFileVersion
+            | ModelPublishKind::OverwriteFileVersion
+            | ModelPublishKind::RowsOnFileVersion
+            | ModelPublishKind::EmptyOnFileVersion
+            | ModelPublishKind::RowsOnVersionWithoutFile
+            | ModelPublishKind::ZeroUnit
+            | ModelPublishKind::UnlinkIntent
+            | ModelPublishKind::DeletionBatchKeepingDataUnits
+            | ModelPublishKind::FinalDeletionBatch
+            | ModelPublishKind::NewInode => {}
+        }
+        roles
+    }
+
+    /// 这次发布重写的、由种类定死的角色（建 inode 落在最左那片之外时那一片不在这里，[`ModelPublishKind::rewritten_roles`] 另加）。
+    /// 分配记录树（`ModelUnitRole::AllocationTree`）在这里只记它的根：根之下重写几个节点按位置寻址现算上界
     /// （`IdealModel::allocation_record_tree_nodes_rewritten_upper_bound`），模型不记是哪几个。
-    fn rewritten_roles(self) -> &'static [ModelUnitRole] {
+    fn rewritten_roles_fixed_by_the_kind(self) -> &'static [ModelUnitRole] {
         match self {
             ModelPublishKind::FirstFileVersion | ModelPublishKind::OverwriteFileVersion => &[
                 ModelUnitRole::Data,
@@ -279,6 +367,14 @@ impl ModelPublishKind {
                 ModelUnitRole::MappingTree,
                 ModelUnitRole::TreeTable,
             ],
+            // 新记录落的那一片（最左那片之外）由 `rewritten_roles` 另加；最左那片不动。
+            ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => &[
+                ModelUnitRole::InodeRoot,
+                ModelUnitRole::AllocationTree,
+                ModelUnitRole::AccountingTree,
+                ModelUnitRole::MappingTree,
+                ModelUnitRole::TreeTable,
+            ],
         }
     }
 }
@@ -311,7 +407,12 @@ pub enum ModelVersionContents {
     /// 删完第一个文件之后建过 inode：inode 树里只有 size 0 的记录、没有 inode 1，没有数据单元、队列空，读回「没有文件」；
     /// 实现把它归带文件的一版（inode 树有条目，D8（核心索引结构） 已定项 16 第 5 条的判定不成立）。
     /// `trees_written_by`：最近一次建 inode 那次发布（回退到这一版的根照抄它）。
-    InodesWithoutTheFirstFile { trees_written_by: ModelRootKey },
+    /// `inode_records_in_the_tree`：这一版 inode 树里有几条记录（删完之后建了几个）：下一个建的落在哪一片叶容器由它定
+    /// （[`inode_leaf_container_position_of_a_new_record`]）。
+    InodesWithoutTheFirstFile {
+        trees_written_by: ModelRootKey,
+        inode_records_in_the_tree: u64,
+    },
 }
 
 impl ModelVersionContents {
@@ -349,9 +450,24 @@ impl ModelVersionContents {
                 trees_written_by, ..
             }
             | ModelVersionContents::TreesAndNoFile { trees_written_by }
-            | ModelVersionContents::InodesWithoutTheFirstFile { trees_written_by } => {
-                Some(*trees_written_by)
-            }
+            | ModelVersionContents::InodesWithoutTheFirstFile {
+                trees_written_by, ..
+            } => Some(*trees_written_by),
+        }
+    }
+
+    /// 删完第一个文件之后建的 inode 在这一版 inode 树里有几条：有树而文件集合为空的一版 0 条；
+    /// 别的几种不是第一个文件已删完的一版，None（建 inode 只接在这两种后面）。
+    fn inode_records_created_after_the_first_file_was_deleted(&self) -> Option<u64> {
+        match self {
+            ModelVersionContents::TreesAndNoFile { .. } => Some(0),
+            ModelVersionContents::InodesWithoutTheFirstFile {
+                inode_records_in_the_tree,
+                ..
+            } => Some(*inode_records_in_the_tree),
+            ModelVersionContents::WithoutTrees
+            | ModelVersionContents::LiveFile(_)
+            | ModelVersionContents::FileBeingDeleted { .. } => None,
         }
     }
 
@@ -610,9 +726,10 @@ fn central_mapping_entries_other_than_the_allocation_record_tree(
     let entries_of_one_node_roles: u64 = role_written_at
         .keys()
         .map(|role| match role {
-            // 模型罩的一版里这几个角色各一个单元（上段只有根兼叶、一片 inode 叶容器、记账树一个节点、墓碑树一片容器加根）。
+            // 模型罩的一版里这几个角色各一个单元（上段只有根兼叶、每片 inode 叶容器各是一个角色、记账树一个节点、墓碑树一片容器加根）。
             ModelUnitRole::ExtentRoot
             | ModelUnitRole::InodeLeaf
+            | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
             | ModelUnitRole::InodeRoot
             | ModelUnitRole::AccountingTree
             | ModelUnitRole::DeletedInodesRoot
@@ -1540,7 +1657,7 @@ impl IdealModel {
             | ModelVersionContents::InodesWithoutTheFirstFile { .. } => None,
         };
         let rewritten = kind.rewritten_roles();
-        for role in rewritten {
+        for role in &rewritten {
             role_written_at.insert(*role, checkpoint_txg);
         }
         let mut contents = match (kind, new_file_content) {
@@ -1569,8 +1686,17 @@ impl IdealModel {
             (ModelPublishKind::FinalDeletionBatch, None) => ModelVersionContents::TreesAndNoFile {
                 trees_written_by: key,
             },
-            (ModelPublishKind::NewInode, None) => ModelVersionContents::InodesWithoutTheFirstFile {
+            (
+                ModelPublishKind::NewInode
+                | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_),
+                None,
+            ) => ModelVersionContents::InodesWithoutTheFirstFile {
                 trees_written_by: key,
+                inode_records_in_the_tree: previous
+                    .contents
+                    .inode_records_created_after_the_first_file_was_deleted()
+                    .expect("建 inode 只接在第一个文件已删完的一版后面（`answer_create_inode` 判过）")
+                    + 1,
             },
             (
                 ModelPublishKind::RowsOnFileVersion
@@ -1588,7 +1714,8 @@ impl IdealModel {
                 | ModelPublishKind::UnlinkIntent
                 | ModelPublishKind::DeletionBatchKeepingDataUnits
                 | ModelPublishKind::FinalDeletionBatch
-                | ModelPublishKind::NewInode,
+                | ModelPublishKind::NewInode
+                | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_),
                 Some(_),
             ) => panic!("只有写文件内容的两种发布带内容，别的都不带：{kind:?}"),
         };
@@ -1615,11 +1742,24 @@ impl IdealModel {
                         }
                     }
                 }
+                // 最左那片之外的叶容器同样整组取目标那一版的：现行那一版有、目标那一版没有的那几片这一版也没有。
+                role_written_at.retain(|role, _| {
+                    !matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))
+                });
+                role_written_at.extend(
+                    target_root
+                        .role_written_at
+                        .iter()
+                        .filter(|(role, _)| {
+                            matches!(role, ModelUnitRole::InodeLeafRightOfTheLeftmost(_))
+                        })
+                        .map(|(role, written_at)| (*role, *written_at)),
+                );
             }
         }
         let mut slots_written = 0;
         let mut rewrites_the_allocation_record_tree = false;
-        for role in rewritten {
+        for role in &rewritten {
             match role {
                 ModelUnitRole::AllocationTree => rewrites_the_allocation_record_tree = true,
                 // 内容的每个数据单元都在这次写（模型把它们记成同一个角色、同一个分配代）。
@@ -1629,6 +1769,7 @@ impl IdealModel {
                 // 中央映射树在这里只记它的根：根之下的节点数按上界另算（下面）。
                 ModelUnitRole::ExtentRoot
                 | ModelUnitRole::InodeLeaf
+                | ModelUnitRole::InodeLeafRightOfTheLeftmost(_)
                 | ModelUnitRole::InodeRoot
                 | ModelUnitRole::AccountingTree
                 | ModelUnitRole::MappingTree
@@ -1656,7 +1797,8 @@ impl IdealModel {
             | ModelPublishKind::ZeroUnit
             | ModelPublishKind::UnlinkIntent
             | ModelPublishKind::FinalDeletionBatch
-            | ModelPublishKind::NewInode => 0,
+            | ModelPublishKind::NewInode
+            | ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => 0,
         };
         slots_written += extent_lower_segment_nodes_written * (NODE_BYTES / SLOT_BYTES);
         // 实例表链多于一片时第 1 片起每一片也是一个重写的单元：`rewritten_roles` 里实例表只记一个角色，多出来的几片在这里补上。
@@ -2014,12 +2156,29 @@ impl IdealModel {
                 ));
             }
         }
+        let inode_records_before = current
+            .contents
+            .inode_records_created_after_the_first_file_was_deleted()
+            .expect("上面只放过第一个文件已删完的两种");
+        // 新记录落在哪一片叶容器定这次重写哪一片（D8（核心索引结构） 已定项 6）：最左那片之外的那几片各是一个角色。
+        let kind = match NonZeroU64::new(inode_leaf_container_position_of_a_new_record(
+            inode_records_before,
+        )) {
+            None => ModelPublishKind::NewInode,
+            Some(position_from_the_leftmost) => {
+                ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(
+                    ModelInodeLeafContainerRightOfTheLeftmost {
+                        position_from_the_leftmost,
+                    },
+                )
+            }
+        };
         let root = self.next_root(
             current,
             session.instance,
             ModelCheckpointTxg(current.key.checkpoint_txg.0 + 1),
             ModelJournalCounter(self.highest_journal_counter.0 + 1),
-            ModelPublishKind::NewInode,
+            kind,
             current.rollback_floor,
             None,
             Rc::clone(&current.instance_table_rows),
diff --git a/crates/singlefs-harness/src/model_comparison.rs b/crates/singlefs-harness/src/model_comparison.rs
index 9eacefdc..86862b32 100644
--- a/crates/singlefs-harness/src/model_comparison.rs
+++ b/crates/singlefs-harness/src/model_comparison.rs
@@ -11,7 +11,6 @@ use std::collections::{BTreeMap, BTreeSet};
 use singlefs_core::address::{CheckpointTxg, DataUnitIndexInFile, InstanceGeneration, SlotNumber};
 use singlefs_core::allocator::{AllocationRecord, PlacementRefusal};
 use singlefs_core::block_device::BlockDeviceError;
-use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
 use singlefs_core::instance_table::{
     InstanceTableChainRecord, InstanceTablePage, InstanceTablePageIndex,
 };
@@ -74,16 +73,11 @@ pub fn model_unit_role(unit: TransactionUnit) -> Option<ModelUnitRole> {
         TransactionUnit::ExtentUpperNodeBelowTheRoot(position) => {
             panic!("模型今天只罩 inode 1 一个文件（上段只有根兼叶）：{position:?}")
         }
-        // 模型罩的发布里 inode 树恒只有最左那一片叶容器：随机历史只在删完第一个文件之后建 inode、一次一个
-        // （`history.rs` 的 `HistoryOperation::CreateInode`），一片叶容器装 233 条，几百步的历史建不满；
-        // 第二片只会从一次建很多个、或第一个文件旁边建 inode 那几条路径出来，随机历史与层 0 的固定脚本都不走。
+        // inode 树的每片叶容器按它在树里左起的位置各是一个角色（最左那片是 `InodeLeaf`，之后每片各一个）：删完第一个文件之后
+        // 一次建一个 inode（`history.rs` 的 `HistoryOperation::CreateInode`），满 233 条在末尾分裂、左半不动（D8（核心索引结构） 已定项 6），
+        // 各片的分配代不同，模型逐片记、逐片比。位置是实现发布路径里的角色序号（`inode_tree::InodeLeafContainerIndexInTree`）。
         TransactionUnit::InodeLeafContainer(index) => {
-            assert_eq!(
-                index,
-                InodeLeafContainerIndexInTree::LEFTMOST,
-                "模型今天只罩一片叶容器的那几种发布"
-            );
-            ModelUnitRole::InodeLeaf
+            ModelUnitRole::inode_leaf_container_at(u64::from(index.0))
         }
         TransactionUnit::InodeRoot => ModelUnitRole::InodeRoot,
         TransactionUnit::AllocationTree => ModelUnitRole::AllocationTree,
@@ -683,7 +677,8 @@ pub fn refusal_reason_of_publish_error(error: &PublishError) -> ObservedRefusalR
         // 释放判定路径的五种：上一版的映射或分配记录与上一版对不上、盘上那条指针的两条位置条目不同槽，健康的历史里不该出现。
         // 分配记录树重写集合迭代不收敛那一条同理：只有强制复用窗口为 0 的只供测试的开关下才可能走到，随机历史不装那个开关。
         // inode 树写入被拒那一条同样：随机历史只在删完第一个文件之后建 inode、一次一个（`HistoryOperation::CreateInode`），
-        // inode 树恒只有一片叶容器、新记录追加在末尾，它跑的那几种发布每次最多改一片叶容器、重写的角色最多九个。
+        // 新记录恒追加在最右那片叶、满 233 条在末尾分裂；叶容器数到一个根装得下的 135 片之前（三万多次建 inode）不拒，
+        // 它跑的那几种发布每次最多改一片叶容器、重写的角色最多九个。
         // 多层码 2 树算不出形状那一条同理：两棵树装不下一个节点时分裂、不拒，只有长到 257 层或上一版的形状按分隔 key 走不通才拒，
         // 随机历史里两棵树恒只有一个节点；它出现就是模型与实现对不上。
         PublishError::MultiLevelCodeTwoTreeRefused { .. }
diff --git a/crates/singlefs-harness/tests/model_compares_inode_creations_across_two_inode_leaf_containers.rs b/crates/singlefs-harness/tests/model_compares_inode_creations_across_two_inode_leaf_containers.rs
new file mode 100644
index 00000000..0cebffd2
--- /dev/null
+++ b/crates/singlefs-harness/tests/model_compares_inode_creations_across_two_inode_leaf_containers.rs
@@ -0,0 +1,90 @@
+//! O1（理想模型对拍，D13（验证路线） 已定项 6）罩 inode 树第二片及以后的叶容器（C673 ⑦「历史加跨一片叶」）：
+//! 删完第一个文件之后一次建一个 inode，建满 233 条之后下一条在末尾分裂出第二片（D8（核心索引结构） 已定项 6：左半不动、
+//! 只重写新记录落的那一片与根），模型逐片记各片叶容器的分配代、逐次与实现交回的分配记录比，每一步都对得上。
+
+use singlefs_core::admission::SpaceAdmission;
+use singlefs_format::INODE_LEAF_RECORDS;
+use singlefs_harness::history::{
+    execute_history_with, AppliedEffect, GeneratedHistory, HistoryDeviceWidth, HistoryEnding,
+    HistoryExecution, HistoryOperation, HistoryOperationKind, HistorySeed, HistoryStartingPoint,
+    PerStepChecker, StepOutcome,
+};
+use singlefs_harness::SharedStream;
+
+/// 删完第一个文件之后建几个 inode：建满第一片的 233 条，再往第二片里建 4 条——第 234 条分裂出第二片，
+/// 之后三条只重写第二片、第一片的分配代停在第 233 条那次。
+const INODE_CREATIONS_INTO_THE_SECOND_LEAF_CONTAINER: u64 = INODE_LEAF_RECORDS + 4;
+
+/// mkfs、取号、暖机、第一个文件（起点 `AfterFirstFile`），unlink 第一个文件（意图那次 + 最后一批，队列走空），
+/// 之后一次建一个 inode，建到 inode 树有两片叶容器再多建三个。模型与实现每一步都对得上：历史跑完、没有新发现，
+/// unlink 与每一次建 inode 都做成。
+/// 每一步的池级 checker 不跑（`PerStepChecker::Skipped`）：这条用例判的是 O1 逐片记的账，跑 checker 时 debug 下要六分多钟。
+/// 判别力：模型把第二片的分配代记成第一片的（建第二片里的 inode 时重写的叶容器记成最左那片），或胶水把第二片当成最左那片，
+/// 建第 234 个 inode 那一步判对不上、历史停在那里。
+#[test]
+fn creating_inodes_one_at_a_time_past_the_first_leaf_container_is_judged_by_the_model_container_by_container(
+) {
+    let creations = usize::try_from(INODE_CREATIONS_INTO_THE_SECOND_LEAF_CONTAINER)
+        .expect("几百次装得进 usize");
+    let mut operations = vec![HistoryOperation::Unlink];
+    operations.extend(std::iter::repeat_n(
+        HistoryOperation::CreateInode,
+        creations,
+    ));
+    let history = GeneratedHistory {
+        seed: HistorySeed(0),
+        starting_point: HistoryStartingPoint::AfterFirstFile,
+        operations,
+    };
+    let run = execute_history_with(
+        &history,
+        HistoryExecution {
+            per_step_checker: PerStepChecker::Skipped,
+            device_width: HistoryDeviceWidth::FourGibibytes,
+            space_admission: SpaceAdmission::JudgedByTheFormula,
+        },
+        &SharedStream::new(),
+        &mut |_| {},
+    );
+    assert_eq!(
+        run.ending,
+        HistoryEnding::Completed,
+        "跑到第 {} 步：{:?}",
+        run.outcomes.len(),
+        run.outcomes.last()
+    );
+    let not_published: Vec<(usize, &StepOutcome)> = run
+        .outcomes
+        .iter()
+        .enumerate()
+        .filter(|(_, outcome)| {
+            !matches!(
+                outcome,
+                StepOutcome::Applied(AppliedEffect::Published { .. })
+            )
+        })
+        .collect();
+    assert!(
+        not_published.is_empty(),
+        "unlink 与每一次建 inode 都做成：{not_published:?}"
+    );
+    let created = run
+        .tally
+        .operations_by_kind
+        .get(&HistoryOperationKind::CreateInode)
+        .copied()
+        .expect("建过 inode：这一类在计数里");
+    assert_eq!(
+        created.applied, INODE_CREATIONS_INTO_THE_SECOND_LEAF_CONTAINER,
+        "建过的 inode 多于一片叶容器装的 {INODE_LEAF_RECORDS} 条：{created:?}"
+    );
+    let counts = &run.tally.model_counts;
+    assert!(
+        counts.successes_matched > INODE_CREATIONS_INTO_THE_SECOND_LEAF_CONTAINER,
+        "模型判过 unlink 与每一次建 inode：{counts:?}"
+    );
+    assert!(
+        counts.allocation_records_compared > INODE_CREATIONS_INTO_THE_SECOND_LEAF_CONTAINER,
+        "每一次建 inode 都逐个角色比过分配记录：{counts:?}"
+    );
+}
````

## 二、25-3a：checker 档依赖 RefFS（`git diff HEAD`）

````diff
diff --git a/Cargo.lock b/Cargo.lock
index babd2af5..c59c6d6e 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -805,6 +805,7 @@ dependencies = [
  "singlefs-core",
  "singlefs-format",
  "singlefs-harness",
+ "singlefs-reference-filesystem",
  "wgpu",
 ]
 
diff --git a/crates/singlefs-checker-tier/Cargo.toml b/crates/singlefs-checker-tier/Cargo.toml
index 39620fd6..f1d6f516 100644
--- a/crates/singlefs-checker-tier/Cargo.toml
+++ b/crates/singlefs-checker-tier/Cargo.toml
@@ -10,6 +10,8 @@ singlefs-checker = { path = "../singlefs-checker" }
 singlefs-core = { path = "../singlefs-core" }
 singlefs-format = { path = "../singlefs-format" }
 singlefs-harness = { path = "../singlefs-harness" }
+# RefFS（D13（验证路线） 已定项 3，E171 第 1 次重跑登记修订 2 第 2.3 节）：checker 档的装置只读它、只调它判允许集合，不改它的源码，它的依赖闭包与单测归它自己（harness 档）。
+singlefs-reference-filesystem = { path = "../singlefs-reference-filesystem" }
 # 崩溃放量的判定存储（src/verdict_store.rs，里程碑三第十一项；用户 2026-09-28 选 RocksDB）。版本钉成与 research/Cargo.lock 相同，
 # E162（崩溃放量判定块存储选型） K 臂杀进程量的就是这一版，默认特性也照那边不动（K 臂用的 LZ4 压缩在默认特性里）。
 # 只进 checker 档：harness 档、池级 checker、core 都不依赖它（.claude/rules/verification.md「定义与名字」，门禁 94 号）。
````

## 三、25-3a 与 25-3：E171 装置（暂存区那一版 → 冻结树那一版，`git diff -- <装置>`；装置整份不在 HEAD 里，全文读冻结副本）

````diff
diff --git a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
index d3b83ae0..356b05dc 100644
--- a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
+++ b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
@@ -20,25 +20,21 @@
 //!
 //! `baseline`：回比本地常量（S1）、三批取样、写导出文件；`row`：不回比常量、只取批 1 与批 2。`stream-constants`：只驱动选中的流、
 //! 打 S1 回比与 Q5 造法在基线上的几样（填本地常量用，不写导出、不判态）。`confirmed-loss-form`：流二驱到第 3 次调用（从 0 数）、
-//! 照读法 7 造「已确认的丢了」那一形、末段全枚举，每态算 O1 三个子列与附带的 O2。目录已存在就拒绝。
+//! 照读法 7 造「已确认的丢了」那一形；方向相反的取样点末段全枚举，主取样点先过「等价类对拍」、再按等价类判（登记修订 2 第 2.1 节
+//! 候选二、修订 3），每个判过的态算 O1 三个子列与附带的 O2。目录已存在就拒绝。
 //! 标准输出：`E7RESULT` 行（与跑几次无关）与 `E7TIMING` 行（耗时）；收尾 `name=done emitted=N`。
-//! 退出码：0 正常；2 命令行不对或输出目录已存在；3 停机条款 S1、S3、S5、S9 没过（产物照打，交主 agent）。
+//! 退出码：0 正常；2 命令行不对或输出目录已存在；3 停机条款 S1、S3、S5、S9、S13 没过，或 `confirmed-loss-form` 那一趟 Q5 没跑
+//! （判决行带 `q5_not_run=`）（产物照打，交主 agent）。
 
 #[path = "../../../singlefs-harness/tests/common/mod.rs"]
 mod common;
 
-/// RefFS（`crates/singlefs-reference-filesystem`，D13（验证路线） 已定项 3）的源码原样编进来：checker 档包没有直接依赖它
-/// （加依赖要改 `crates/singlefs-checker-tier/Cargo.toml` 与 `Cargo.lock`，越出这一段执行员的写范围，交主 agent 定），
-/// 翻译层 `reference_filesystem_translation.rs` 的 `events_of` 只认 `history.rs` 的 `ContentChoice` 生成的内容、交不出本装置三条流的内容，
-/// 所以照那份映射在本装置里交事件（[`reference_filesystem_column`]），要直接用 RefFS 的类型。编的是同一份源文件、同一份语义；
-/// 它自己的 `#[cfg(test)]` 用例随本装置的单测一起编、一起跑。
-#[allow(
-    dead_code,
-    reason = "RefFS 的公开项本装置只用其中几样（建池、交操作、崩溃并恢复），别的没人调"
-)]
-#[path = "../../../singlefs-reference-filesystem/src/lib.rs"]
-mod reference_filesystem;
+// RefFS（`crates/singlefs-reference-filesystem`，D13（验证路线） 已定项 3）：checker 档包直接依赖它（登记修订 2 第 2.3 节），
+// 类型与 harness 翻译层用的是同一份；翻译层的 `events_of` 交不出本装置三条流的内容，所以照那份映射在本装置里交事件，
+// 要直接用 RefFS 的类型。它自己的单测归它自己的包（harness 档），不再编进本装置。
+use singlefs_reference_filesystem as reference_filesystem;
 
+use std::cell::RefCell;
 use std::collections::{BTreeMap, BTreeSet};
 use std::path::{Path, PathBuf};
 use std::rc::Rc;
@@ -263,8 +259,22 @@ const LOCAL_CONFIRMED_LOSS_REVERSE_TRUNCATION: usize = 57;
 const LOCAL_CONFIRMED_LOSS_LAST_SEGMENT_WRITES: usize = 29;
 const LOCAL_CONFIRMED_LOSS_LAST_SEGMENT_TEARABLE_WRITES: usize = 2;
 const LOCAL_CONFIRMED_LOSS_STATES: u64 = 1_207_959_552;
-/// `confirmed-loss-form` 一趟判得完的态数上限：每态推 5–50 毫秒，上限这么多约是一天以内。超过的那一形不跑、打一行 `not_run` 退 3，
-/// 交主 agent（主取样点照读法 7 字面末段 29 次写、2 次可撕裂，1 207 959 552 态，第 1 次重跑第一段报告「Q5 主取样点跑不了」）。
+// 主取样点按等价类判（登记修订 2 第 2.1 节候选二、修订 3）的起点分法（类核对 ③ 细分之前）：键里是末段中不属于第 3 次调用的写（K），
+// 类里是第 3 次调用的写（C）。值照第 2.1 节「末段是什么」与「Q5-等价类」两段：K 5 个写、取法 2³ × 3² = 72 种，C 24 个写，
+// 类数 72 × 2 = 144，「C 一个没落」那一类各 1 态、「C 至少落一个」那一类各 2²⁴ − 1 = 16 777 215 态，要照原路径判的态 72 + 72 = 144。
+/// K 的取法数。
+const LOCAL_CONFIRMED_LOSS_KEY_CHOICES: u64 = 72;
+/// C 的写数。
+const LOCAL_CONFIRMED_LOSS_IN_FLIGHT_WRITES: usize = 24;
+/// 类数。
+const LOCAL_CONFIRMED_LOSS_CLASSES: u64 = 144;
+/// 「C 至少落一个」那一类的态数。
+const LOCAL_CONFIRMED_LOSS_STATES_OF_A_CLASS_WITH_IN_FLIGHT_WRITES_LANDED: u64 = 16_777_215;
+/// 要照原路径判的态数（每种取法的代表加「C 全落」那一态）。
+const LOCAL_CONFIRMED_LOSS_JUDGED_STATES: u64 = 144;
+/// `confirmed-loss-form` 一趟判得完的态数上限：每态推 5–50 毫秒，上限这么多约是一天以内。方向相反的取样点全枚举，超过的不跑、
+/// 打一行 `not_run` 退 3 交主 agent；主取样点按等价类判（登记修订 2 第 2.1 节候选二、修订 3），这个上限管的是类核对 ③ 细分之后
+/// 要照原路径判的态数（每种取法的代表加「C 全落」那一态），超过走停机 S13（主取样点全枚举是 1 207 959 552 态，第 1 次重跑第一段跑不了）。
 const CONFIRMED_LOSS_STATE_LIMIT: u64 = 10_000_000;
 
 /// 取样：批 1 每段内点的分母、批 2 的分母、批 3 的分母（登记 5.4）。
@@ -412,8 +422,9 @@ enum ModelVerdictOfCall {
     SkippedAfterTheModelStopped,
     /// 这次调用 panic：实现自崩、不比，不红不绿（读法 5）；模型停在这次调用。
     NotComparedBecauseTheImplementationCrashed,
-    /// 调用成功了，而胶水造不出交给模型的观测（`model_comparison` 的断言：模型今天只罩一片 inode 叶容器的那几种发布）：
-    /// 不比、模型停在这次调用。读法 6 的三处来源之外的一处，停机 S5 记它（流三跨一片叶那一次撞上，第 1 次重跑第一段报告交主 agent）。
+    /// 调用成功了，而胶水造不出交给模型的观测（`model_comparison::model_unit_role` 里模型还不罩的几种单元在这一版里出现，胶水 panic）：
+    /// 不比、模型停在这次调用。读法 6 的三处来源之外的一处，停机 S5 记它（第一段流三跨一片叶那一次撞上过；理想模型那件修理之后
+    /// 胶水按片记 inode 叶容器的角色、不再断言，登记修订 2 第 2.2 节：合入后的树上再出现就是 S5）。
     ModelCannotTakeTheObservation { what: String },
 }
 
@@ -1020,6 +1031,15 @@ fn memory_objects_of(
     ]
 }
 
+/// 流三的一次发布（[`LiveStream::step_third_stream_publish`] 的 `publish`）：经 `PoolWriter` 调实现的发布函数，交回那一版或发布错误。
+type ThirdStreamPublish<'closure> = dyn Fn(
+        &mut PoolWriter<'_, common::Recorded>,
+        &mut PoolAllocator,
+        &TransactionOutput,
+        InstanceGeneration,
+    ) -> Result<TransactionOutput, singlefs_core::transaction::PublishError>
+    + 'closure;
+
 /// 一条流驱动时的池：只做过 mkfs 的两块文件镜像（`FormattedPool` 管镜像文件，丢掉就删）、分配器、现行那一版、层 0 手写 oracle 的版本表。
 /// 流一照抄 `tests/common/mod.rs` 第 677 行 `build_pool` 的取号、暖机、新池新建文件三步，流二照抄
 /// `tests/crash_enumeration_fixed_script_stream.rs` 第 218 行起的 `Script::ReuseAfterRaisingFloorThenNormalUnmount` 那一支，
@@ -1594,13 +1614,7 @@ impl LiveStream {
         &mut self,
         label: &'static str,
         answer: fn(&IdealModel) -> Result<ModelAnswer, ModelDisagreement>,
-        publish: &dyn Fn(
-            &mut PoolWriter<'_, common::Recorded>,
-            &mut PoolAllocator,
-            &TransactionOutput,
-            InstanceGeneration,
-        )
-            -> Result<TransactionOutput, singlefs_core::transaction::PublishError>,
+        publish: &ThirdStreamPublish<'_>,
     ) -> bool {
         let parameters = common::parameters();
         let instance = self.instance.expect("取号成功才走到这一步");
@@ -1615,7 +1629,9 @@ impl LiveStream {
             });
         let published = match result {
             Ok(output) => {
-                // 胶水在这一版跨出一片 inode 叶容器时断言失败（模型今天只罩一片叶容器）：接住，记「模型接不了这份观测」、模型停在这次调用。
+                // 胶水造观测时 panic（`model_comparison::model_unit_role` 里模型还不罩的几种单元：extent 上段根之下的节点、记账树根之下的节点、
+                // 墓碑树第二片起的容器与根之下的节点）：接住，记「模型接不了这份观测」、模型停在这次调用。跨出一片 inode 叶容器不再走到这里
+                // （理想模型那件修理之后胶水按片记角色，不再断言；登记修订 2 第 2.2 节）。
                 let observed_root = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                     observed_root_of_file_version(&output)
                 }));
@@ -2722,6 +2738,10 @@ fn violated_invariant_names(verdicts: &[(&'static str, InvariantVerdict)]) -> Ve
 
 /// RefFS 翻这一条流停在哪一步、为什么（名字与 `reference_filesystem_translation::TranslationStopReason::name` 交出的一字不差，
 /// 比对器按名字认弃权种类）。
+#[allow(
+    clippy::enum_variant_names,
+    reason = "变体名逐个照抄 harness `TranslationStopReason` 的同名臂（注释与 `events_of` 按这几个名字对照），不另起一套"
+)]
 #[derive(Clone, Debug, PartialEq, Eq)]
 enum ReferenceFileSystemStop {
     /// `events_of` 的 `RollBackWhileMounted` 那一臂：回退没返回 Ok。
@@ -3384,6 +3404,9 @@ struct StateObservation {
     o1_nanoseconds: u128,
     o2_nanoseconds: u128,
     recovery_nanoseconds: u128,
+    /// 看 journal 那一遍的整份恢复报告：只在 Q5 主取样点按等价类判的那一趟留（类核对 ①、④ 与「等价类对拍」逐字段比它），
+    /// 别的趟与恢复 panic 的态是 None。
+    consulted_report: Option<RecoveryReport>,
 }
 
 /// 一次观测的状态：阳性对照里「批 1 第一个读回了文件的态」只翻一次（O1-崩溃判官与 RefFS 各一次）、「见证」只改一次。
@@ -3392,6 +3415,14 @@ struct ObservationControls {
     crash_judge_control_pending: bool,
     reference_filesystem_control_pending: bool,
     witness_control_pending: bool,
+    consulted_reports: ConsultedReports,
+}
+
+/// 观测时留不留看 journal 那一遍的整份恢复报告（[`StateObservation::consulted_report`]）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum ConsultedReports {
+    Dropped,
+    Kept,
 }
 
 /// 把写表每一项落在哪一次调用里算出来：写在录制流里的下标落在 `first_operation` 不超过它的最后一次调用里。
@@ -3641,6 +3672,10 @@ fn observation_of_a_judged_state(
         o1_nanoseconds,
         o2_nanoseconds,
         recovery_nanoseconds,
+        consulted_report: match controls.consulted_reports {
+            ConsultedReports::Dropped => None,
+            ConsultedReports::Kept => Some(judgement.consulted.clone()),
+        },
     }
 }
 
@@ -3705,6 +3740,7 @@ fn observation_of_a_panicking_state(
         o1_nanoseconds: 0,
         o2_nanoseconds,
         recovery_nanoseconds: 0,
+        consulted_report: None,
     }
 }
 
@@ -3714,6 +3750,7 @@ fn observe_sampled_states(
     context: &ObservationContext<'_>,
     sampled: &SampledOrdinals,
     switches: RunSwitches,
+    consulted_reports: ConsultedReports,
 ) -> Vec<StateObservation> {
     let mut batch_of_ordinal: BTreeMap<u64, u8> = BTreeMap::new();
     for (batch_offset, batch) in sampled.batches.iter().enumerate() {
@@ -3739,53 +3776,56 @@ fn observe_sampled_states(
         reference_filesystem_control_pending: switches.positive_control
             == Some(PositiveControl::ReferenceFileSystem),
         witness_control_pending: switches.positive_control == Some(PositiveControl::Witness),
+        consulted_reports,
     };
     let mut observations: Vec<StateObservation> = Vec::new();
     let mut panicking_states: Vec<(u64, String)> = Vec::new();
-    let mut judge_run = |run: std::ops::Range<u64>| -> Result<(), u64> {
-        let mut next_ordinal = run.start;
-        let judged = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
-            judge_layer0_state_range(
-                &stream.base,
-                &stream.writes,
-                &stream.segments,
-                stream.judged_root_index,
-                &stream.versions,
-                &full_expansion,
-                run.clone(),
-                &mut |ordinal, image, judgement| {
-                    observations.push(observation_of_a_judged_state(
-                        context,
-                        batch_of_ordinal[&ordinal],
-                        ordinal,
-                        image,
-                        judgement,
-                        &mut controls,
-                    ));
-                    next_ordinal = ordinal + 1;
-                },
-            )
-        }));
-        match judged {
-            Ok(state_count_of_the_stream) => {
-                assert_eq!(
-                    state_count_of_the_stream, context.tables.state_count,
-                    "judge_layer0_state_range 交回的整条流状态数与枚举表的相同"
-                );
-                Ok(())
-            }
-            Err(payload) => {
-                panicking_states.push((next_ordinal, panic_text(&*payload)));
-                Err(next_ordinal)
-            }
-        }
-    };
-    // panic 的序号与消息在 `judge_run` 里记（它手里有 panic 的消息）；这里只核每一个都记到了。
+    // panic 的序号与消息在 `judge_run` 里记（它手里有 panic 的消息）；块之后只核每一个都记到了。`judge_run` 借着
+    // `observations` 与 `panicking_states`，放进这一块，块结束借用就还回来。
     let mut panicking_ordinals_noted: Vec<u64> = Vec::new();
-    judge_runs_continuing_past_panicking_states(&runs, &mut judge_run, &mut |ordinal| {
-        panicking_ordinals_noted.push(ordinal);
-    });
-    drop(judge_run);
+    {
+        let mut judge_run = |run: std::ops::Range<u64>| -> Result<(), u64> {
+            let mut next_ordinal = run.start;
+            let judged = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+                judge_layer0_state_range(
+                    &stream.base,
+                    &stream.writes,
+                    &stream.segments,
+                    stream.judged_root_index,
+                    &stream.versions,
+                    &full_expansion,
+                    run.clone(),
+                    &mut |ordinal, image, judgement| {
+                        observations.push(observation_of_a_judged_state(
+                            context,
+                            batch_of_ordinal[&ordinal],
+                            ordinal,
+                            image,
+                            judgement,
+                            &mut controls,
+                        ));
+                        next_ordinal = ordinal + 1;
+                    },
+                )
+            }));
+            match judged {
+                Ok(state_count_of_the_stream) => {
+                    assert_eq!(
+                        state_count_of_the_stream, context.tables.state_count,
+                        "judge_layer0_state_range 交回的整条流状态数与枚举表的相同"
+                    );
+                    Ok(())
+                }
+                Err(payload) => {
+                    panicking_states.push((next_ordinal, panic_text(&*payload)));
+                    Err(next_ordinal)
+                }
+            }
+        };
+        judge_runs_continuing_past_panicking_states(&runs, &mut judge_run, &mut |ordinal| {
+            panicking_ordinals_noted.push(ordinal);
+        });
+    }
     assert_eq!(
         panicking_ordinals_noted,
         panicking_states
@@ -4369,11 +4409,13 @@ enum RunKind {
     BadImplementationRow,
 }
 
-/// 判哪些态：基线与逐行那一趟照登记 5.4 取批（逐行那一趟不取批 3）；Q5 那一形取末段整段全枚举再加最后那个全持久态（读法 7），都记在批 1。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+/// 判哪些态：基线与逐行那一趟照登记 5.4 取批（逐行那一趟不取批 3）；Q5 那一形取末段整段全枚举再加最后那个全持久态（读法 7），都记在批 1；
+/// Q5 主取样点按等价类判（登记修订 2 第 2.1 节候选二、修订 3）只判点名的那几态（各类的代表与「C 全落」那一态），记在批 1、留整份恢复报告。
+#[derive(Clone, Debug, PartialEq, Eq)]
 enum StateSelection {
     RegisteredBatches { third_batch: bool },
     WholeLastSegmentAndTheFullyPersistedState,
+    ListedStatesKeepingTheConsultedReports(BTreeSet<u64>),
 }
 
 fn export_stream(
@@ -4481,7 +4523,7 @@ fn export_prepared_stream(
         })
         .collect();
     let recorded_write_count = stream.writes.len();
-    let sampled = match selection {
+    let sampled = match &selection {
         StateSelection::RegisteredBatches { third_batch } => {
             let mut sampled = sample_ordinals(
                 &tables.state_ranges_by_segment,
@@ -4495,7 +4537,7 @@ fn export_prepared_stream(
                     )
                 },
             );
-            if !third_batch {
+            if !*third_batch {
                 sampled.batches[2].clear();
             }
             sampled
@@ -4513,6 +4555,15 @@ fn export_prepared_stream(
                 torn_scans: Vec::new(),
             }
         }
+        StateSelection::ListedStatesKeepingTheConsultedReports(listed) => SampledOrdinals {
+            batches: [listed.clone(), BTreeSet::new(), BTreeSet::new()],
+            torn_scans: Vec::new(),
+        },
+    };
+    let consulted_reports = match &selection {
+        StateSelection::RegisteredBatches { .. }
+        | StateSelection::WholeLastSegmentAndTheFullyPersistedState => ConsultedReports::Dropped,
+        StateSelection::ListedStatesKeepingTheConsultedReports(_) => ConsultedReports::Kept,
     };
     let reference_filesystem = reference_filesystem_stream_of(
         &stream.calls,
@@ -4541,7 +4592,7 @@ fn export_prepared_stream(
         call_of_write: call_index_of_each_write(stream),
     };
     let observe_started = Instant::now();
-    let observations = observe_sampled_states(&context, &sampled, switches);
+    let observations = observe_sampled_states(&context, &sampled, switches, consulted_reports);
     let observe_nanoseconds = observe_started.elapsed().as_nanos();
     let write_started = Instant::now();
     let files = write_stream_files(
@@ -7012,15 +7063,68 @@ fn second_stream_through_the_first_overwrite(switches: RunSwitches) -> LiveStrea
     live
 }
 
+/// Q5 那一形截在哪里。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum ConfirmedLossTruncation {
+    /// 主取样点：第 3 次调用发出的第一道屏障之前（读法 7）。
+    BeforeTheInFlightBarrier,
+    /// 方向相反的取样点：第 2 次调用的返回点（读法 7、第八节）。
+    AtTheConfirmedReturnPoint,
+    /// 「等价类对拍」那一形：第 3 次调用发出的前这么多个写之后（登记修订 2 第 2.1 节候选二最后一条，即候选三的造法）。
+    AfterTheFirstInFlightWrites(usize),
+}
+
+/// 第 3 次调用发出的第 `writes_kept` 个写之后的下标（按发出次序数写，屏障不数）；主取样点截断之前第 3 次调用发出的写不够这么多个，
+/// 截在主取样点的截断处（那里第 3 次调用的写已经全留着）。
+fn truncation_after_the_first_writes_of_the_in_flight_call(
+    operations: &[RetainedOperation],
+    calls: &[RecordedCall],
+    shape: &ConfirmedLossShape,
+    writes_kept: usize,
+) -> usize {
+    let in_flight = &calls[CONFIRMED_LOSS_IN_FLIGHT_CALL];
+    let mut writes_seen = 0usize;
+    for (index, retained) in operations
+        .iter()
+        .enumerate()
+        .take(shape.truncation)
+        .skip(in_flight.first_operation)
+    {
+        match retained.operation.kind {
+            RecordedOperationKind::Write
+            | RecordedOperationKind::WriteForceUnitAccess
+            | RecordedOperationKind::WriteZeroes => writes_seen += 1,
+            RecordedOperationKind::Barrier => {}
+        }
+        if writes_seen == writes_kept {
+            return index + 1;
+        }
+    }
+    shape.truncation
+}
+
 /// Q5 那一形的流：改过、截过的写流重新切段，调用的下标照改之后的写流换，第 3 次调用记「在途」、返回点取截断处。
 fn confirmed_loss_prepared_stream(
     live: LiveStream,
-    at_the_return_point: bool,
+    truncation: ConfirmedLossTruncation,
 ) -> (PreparedStream, ConfirmedLossShape) {
     let mkfs_operation_count = live.formatted.mkfs_operation_count;
     let all_operations = live.formatted.retained_operations();
     let operations = &all_operations[mkfs_operation_count..];
-    let shape = confirmed_loss_shape_of(operations, &live.driver.calls);
+    let mut shape = confirmed_loss_shape_of(operations, &live.driver.calls);
+    let at_the_return_point = match truncation {
+        ConfirmedLossTruncation::BeforeTheInFlightBarrier => false,
+        ConfirmedLossTruncation::AtTheConfirmedReturnPoint => true,
+        ConfirmedLossTruncation::AfterTheFirstInFlightWrites(writes_kept) => {
+            shape.truncation = truncation_after_the_first_writes_of_the_in_flight_call(
+                operations,
+                &live.driver.calls,
+                &shape,
+                writes_kept,
+            );
+            false
+        }
+    };
     let (kept, new_index_of) = confirmed_loss_operations(operations, &shape, at_the_return_point);
     let remap = |old: usize| -> usize {
         new_index_of
@@ -7097,7 +7201,8 @@ fn last_segment_product(
 /// S1 里 Q5 造法在基线上的几样（5.3）：驱动一遍流二前四次调用，算造法下标、末段写数、可撕裂写数与全枚举态数，对本地常量。
 fn confirmed_loss_form_checks_against_the_crates() -> Vec<StopCheck> {
     let live = second_stream_through_the_first_overwrite(RunSwitches::default());
-    let (prepared, shape) = confirmed_loss_prepared_stream(live, false);
+    let (prepared, shape) =
+        confirmed_loss_prepared_stream(live, ConfirmedLossTruncation::BeforeTheInFlightBarrier);
     let tables = layer0_enumeration_tables(
         &prepared.base,
         &prepared.writes,
@@ -7105,7 +7210,47 @@ fn confirmed_loss_form_checks_against_the_crates() -> Vec<StopCheck> {
         &full_expansion,
     );
     let (writes, tearable, product) = last_segment_product(&prepared, &tables);
+    // 等价类的起点分法（候选二）：只看末段每个写属于哪次调用与各自的进制，不跑恢复（细分要跑恢复，归 `confirmed-loss-form` 那一趟）。
+    let digits = LastSegmentDigits::of(&prepared, &tables);
+    let partition = ConfirmedLossPartition::before_any_refinement(
+        &call_index_of_each_write(&prepared),
+        &digits,
+        CONFIRMED_LOSS_IN_FLIGHT_CALL,
+    );
+    let key_choices = partition.keyed_assignments(&digits);
+    let states_of_a_landed_class = partition.states_with_the_in_flight_writes_landed(&digits);
     vec![
+        StopCheck::new(
+            "confirmed_loss_key_choices",
+            LOCAL_CONFIRMED_LOSS_KEY_CHOICES,
+            key_choices,
+        ),
+        StopCheck::new(
+            "confirmed_loss_in_flight_writes",
+            LOCAL_CONFIRMED_LOSS_IN_FLIGHT_WRITES,
+            partition.class_positions.len(),
+        ),
+        StopCheck::new(
+            "confirmed_loss_classes",
+            LOCAL_CONFIRMED_LOSS_CLASSES,
+            key_choices * 2,
+        ),
+        StopCheck::new(
+            "confirmed_loss_states_of_a_class_with_in_flight_writes_landed",
+            LOCAL_CONFIRMED_LOSS_STATES_OF_A_CLASS_WITH_IN_FLIGHT_WRITES_LANDED,
+            states_of_a_landed_class,
+        ),
+        StopCheck::new(
+            "confirmed_loss_judged_states",
+            LOCAL_CONFIRMED_LOSS_JUDGED_STATES,
+            partition.judged_state_count(&digits),
+        ),
+        // 各类态数之和（「C 一个没落」各 1 态加「C 至少落一个」各那么多态）等于全枚举态数：写成加法（第七节 A10）。
+        StopCheck::new(
+            "confirmed_loss_class_states_sum_to_the_states",
+            LOCAL_CONFIRMED_LOSS_STATES,
+            key_choices * (1 + states_of_a_landed_class),
+        ),
         StopCheck::new(
             "confirmed_loss_removed_barriers",
             LOCAL_CONFIRMED_LOSS_REMOVED_BARRIERS.to_vec(),
@@ -7153,10 +7298,16 @@ fn confirmed_loss_form_checks_against_the_crates() -> Vec<StopCheck> {
     ]
 }
 
-/// `confirmed-loss-form` 模式：造那一形、末段全枚举、每态算 O1 三个子列与附带的 O2，导出中性格式（`<输出>/confirmed_loss_form/<流二目录名>/`），
-/// 打 Q5 的计数（读法 7、第六节 Q5-有效 / Q5-判 / Q5-阴性、第八节轨迹）。
+/// `confirmed-loss-form` 模式：造那一形、每态算 O1 三个子列与附带的 O2，导出中性格式（`<输出>/confirmed_loss_form/<流二目录名>/`），
+/// 打 Q5 的计数（读法 7、第六节 Q5-有效 / Q5-判 / Q5-阴性、第八节轨迹）。方向相反的取样点末段全枚举；主取样点按等价类判
+/// （登记修订 2 第 2.1 节候选二、修订 3）：先过「等价类对拍」，再按类判主取样点，类核对 ①–④ 或对拍不过走停机 S13。
 fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
     let mut lines = ResultLines { emitted: 0 };
+    let truncation = if command_line.truncate_at_the_return_point {
+        ConfirmedLossTruncation::AtTheConfirmedReturnPoint
+    } else {
+        ConfirmedLossTruncation::BeforeTheInFlightBarrier
+    };
     let sampling_point = if command_line.truncate_at_the_return_point {
         "reverse_at_the_return_point_of_call_2"
     } else {
@@ -7176,10 +7327,41 @@ fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
             check.holds()
         ));
     }
+    if !command_line.truncate_at_the_return_point {
+        // 装置先过「等价类对拍」再跑主取样点（修订 2 第 2.1 节「定之前与定之后」末句）。
+        let (pairing_stream, _) = confirmed_loss_prepared_stream(
+            second_stream_through_the_first_overwrite(RunSwitches::default()),
+            ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
+                EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT,
+            ),
+        );
+        let pairing = equivalence_pairing_of(&pairing_stream);
+        lines.result(&format!(
+            "name=q5_equivalence_pairing in_flight_writes_kept={EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT} states={} product_of_the_choices={} class_states={} classes={} mismatching_states={} crash_judge_not_applicable_states={} {} {}",
+            pairing.states,
+            pairing.product_of_the_choices,
+            pairing.class_states,
+            pairing.classes,
+            pairing.mismatching_states,
+            pairing
+                .observations
+                .iter()
+                .filter(|observation| observation.crash_judge == CrashJudgeVerdict::NotApplicable)
+                .count(),
+            equivalence_check_fields(&pairing.classification, &pairing.checks),
+            partition_fields(&pairing.classification, &pairing.checks)
+        ));
+        if pairing.stops_at_s13() {
+            lines.result(&format!(
+                "name=verdict s1_failed_checks={failed_checks} s13=equivalence_pairing q5_not_run=equivalence_pairing_failed"
+            ));
+            lines.finish();
+            return 3;
+        }
+    }
     let started = Instant::now();
     let live = second_stream_through_the_first_overwrite(RunSwitches::default());
-    let (prepared, shape) =
-        confirmed_loss_prepared_stream(live, command_line.truncate_at_the_return_point);
+    let (prepared, shape) = confirmed_loss_prepared_stream(live, truncation);
     let drive_nanoseconds = started.elapsed().as_nanos();
     let confirmed_root = prepared
         .writes
@@ -7202,64 +7384,167 @@ fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
         prepared.calls[CONFIRMED_LOSS_CONFIRMED_CALL].outcome == CallOutcome::Succeeded
             && prepared.calls[CONFIRMED_LOSS_CONFIRMED_CALL].return_point <= prepared.operations.len(),
     ));
-    let planned_tables = layer0_enumeration_tables(
-        &prepared.base,
-        &prepared.writes,
-        &prepared.segments,
-        &full_expansion,
-    );
-    let (_, _, planned_states) = last_segment_product(&prepared, &planned_tables);
-    if planned_states > CONFIRMED_LOSS_STATE_LIMIT {
-        lines.result(&format!(
-            "name=q5 sampling_point={sampling_point} not_run=too_many_states states={planned_states} limit={CONFIRMED_LOSS_STATE_LIMIT}"
-        ));
-        lines.result(&format!(
-            "name=verdict s1_failed_checks={failed_checks} q5_not_run=too_many_states"
-        ));
-        lines.finish();
-        return 3;
-    }
     let directory = command_line
         .output_directory
         .join("confirmed_loss_form")
         .join(StreamName::FixedScriptThroughEThenUnmount.directory_name());
-    let mut export = export_prepared_stream(
-        &prepared,
-        &directory,
-        false,
-        RunSwitches::default(),
-        StateSelection::WholeLastSegmentAndTheFullyPersistedState,
-        started,
-        drive_nanoseconds,
-    );
-    let s9 = s9_checks_of_a_stream(&export);
-    export.checks.extend(s9);
-    let totals = report_stream(&mut lines, &export);
-    failed_checks += totals.failed_checks;
-    let tables = layer0_enumeration_tables(
-        &prepared.base,
-        &prepared.writes,
-        &prepared.segments,
-        &full_expansion,
-    );
-    let (last_segment_writes, tearable, product) = last_segment_product(&prepared, &tables);
+    let observation_tables = ObservationTables::of(&prepared);
+    let (last_segment_writes, tearable, product) =
+        last_segment_product(&prepared, &observation_tables.tables);
     let last_segment: Vec<usize> = prepared.segments.last().cloned().unwrap_or_default();
-    report_the_confirmed_loss(
-        &mut lines,
-        sampling_point,
-        &export.observations,
-        confirmed_root,
-        &last_segment,
-        (last_segment_writes, tearable, product),
-    );
+    let (export, totals, stops_at_s13) = if command_line.truncate_at_the_return_point {
+        // 方向相反的取样点照读法 7 全枚举（修订 2 的五个候选都不改它）；态数超上限不跑。
+        if product > CONFIRMED_LOSS_STATE_LIMIT {
+            lines.result(&format!(
+                "name=q5 sampling_point={sampling_point} not_run=too_many_states states={product} limit={CONFIRMED_LOSS_STATE_LIMIT}"
+            ));
+            lines.result(&format!(
+                "name=verdict s1_failed_checks={failed_checks} q5_not_run=too_many_states"
+            ));
+            lines.finish();
+            return 3;
+        }
+        let mut export = export_prepared_stream(
+            &prepared,
+            &directory,
+            false,
+            RunSwitches::default(),
+            StateSelection::WholeLastSegmentAndTheFullyPersistedState,
+            started,
+            drive_nanoseconds,
+        );
+        let s9 = s9_checks_of_a_stream(&export);
+        export.checks.extend(s9);
+        let totals = report_stream(&mut lines, &export);
+        let rows: Vec<ConfirmedLossTallyRow<'_>> = export
+            .observations
+            .iter()
+            .map(|observation| ConfirmedLossTallyRow::of_a_judged_state(observation, &last_segment))
+            .collect();
+        report_the_confirmed_loss(
+            &mut lines,
+            sampling_point,
+            &rows,
+            confirmed_root,
+            (last_segment_writes, tearable, product),
+            "",
+        );
+        (export, totals, false)
+    } else {
+        let context = observation_tables.context(&prepared);
+        let digits = LastSegmentDigits::of(&prepared, &observation_tables.tables);
+        let classification = classify_the_last_segment(
+            &context,
+            &digits,
+            CONFIRMED_LOSS_IN_FLIGHT_CALL,
+            CONFIRMED_LOSS_STATE_LIMIT,
+        );
+        if classification.stopped_at_the_state_limit {
+            lines.result(&format!(
+                "name=q5_equivalence sampling_point={sampling_point} {}",
+                equivalence_check_fields(&classification, &EquivalenceCheckCounts::default())
+            ));
+            lines.result(&format!(
+                "name=q5_partition sampling_point={sampling_point} {}",
+                partition_fields(&classification, &EquivalenceCheckCounts::default())
+            ));
+            lines.result(&format!(
+                "name=verdict s1_failed_checks={failed_checks} s13=state_limit q5_not_run=too_many_states_after_refinement"
+            ));
+            lines.finish();
+            return 3;
+        }
+        let judged = classification.judged_ordinals(&digits);
+        let judged_states = judged.len();
+        let mut export = export_prepared_stream(
+            &prepared,
+            &directory,
+            false,
+            RunSwitches::default(),
+            StateSelection::ListedStatesKeepingTheConsultedReports(judged),
+            started,
+            drive_nanoseconds,
+        );
+        let s9 = s9_checks_of_a_stream(&export);
+        export.checks.extend(s9);
+        let totals = report_stream(&mut lines, &export);
+        if let Some(why) = &export.not_exported {
+            // 这一形截出来的写流里没有可判的根槽写（`export_prepared_stream` 记 `no_root_write_to_judge`）：类核对与对拍都没走到，
+            // 不是 S13；记 Q5 没跑、不进判定（登记「主 agent 认定：25-3a 装置返工交回」第 6 条）。
+            lines.result(&format!(
+                "name=q5 sampling_point={sampling_point} not_run=no_root_write not_exported_why={}",
+                field_text(why)
+            ));
+            lines.result(&format!(
+                "name=verdict s1_failed_checks={} s4_digest_mismatches={} s3_o2_judgement_mismatches={} states=0 s13=false q5_not_run=no_root_write",
+                failed_checks + totals.failed_checks,
+                totals.rebuilt_digest_mismatches,
+                totals.s3_mismatches
+            ));
+            lines.finish();
+            return 3;
+        } else {
+            let evaluation = evaluate_the_classes(
+                &context,
+                &digits,
+                &classification,
+                &export.observations,
+                CONFIRMED_LOSS_IN_FLIGHT_CALL,
+            );
+            lines.result(&format!(
+                "name=q5_equivalence sampling_point={sampling_point} {}",
+                equivalence_check_fields(&classification, &evaluation.checks)
+            ));
+            lines.result(&format!(
+                "name=q5_partition sampling_point={sampling_point} {}",
+                partition_fields(&classification, &evaluation.checks)
+            ));
+            for class in evaluation.classes.values() {
+                let lost = confirmed_root
+                    .and_then(|confirmed| {
+                        lost_the_confirmed_version(class.consulted_root, confirmed)
+                    })
+                    .map_or_else(|| "none".to_string(), |lost| lost.to_string());
+                lines.result(&format!(
+                    "name=q5_class sampling_point={sampling_point} keyed_landing={} in_flight_writes={} states={} consulted_root={} lost_the_confirmed_version={lost} crash_judge={} reference_filesystem={}",
+                    class.keyed_landing,
+                    class.key.in_flight_writes_landed.word(),
+                    class.states,
+                    root_text(class.consulted_root),
+                    class.columns.crash_judge.word(),
+                    class.columns.reference_filesystem.word()
+                ));
+            }
+            let rows: Vec<ConfirmedLossTallyRow<'_>> = evaluation
+                .classes
+                .values()
+                .map(ConfirmedLossTallyRow::of_a_class)
+                .collect();
+            report_the_confirmed_loss(
+                &mut lines,
+                sampling_point,
+                &rows,
+                confirmed_root,
+                (last_segment_writes, tearable, product),
+                &format!(" derived_by=equivalence_classes judged_states={judged_states}"),
+            );
+            let stops = equivalence_stops_at_s13(&classification, &evaluation.checks);
+            (export, totals, stops)
+        }
+    };
+    failed_checks += totals.failed_checks;
     lines.result(&format!(
-        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={} s3_o2_judgement_mismatches={} states={}",
+        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={} s3_o2_judgement_mismatches={} states={} s13={stops_at_s13}",
         totals.rebuilt_digest_mismatches,
         totals.s3_mismatches,
         export.observations.len()
     ));
     lines.finish();
-    if failed_checks > 0 || totals.rebuilt_digest_mismatches > 0 || totals.s3_mismatches > 0 {
+    if failed_checks > 0
+        || totals.rebuilt_digest_mismatches > 0
+        || totals.s3_mismatches > 0
+        || stops_at_s13
+    {
         3
     } else {
         0
@@ -7276,130 +7561,1099 @@ fn lost_the_confirmed_version(
     })
 }
 
-/// Q5 的计数行（第六节 Q5-有效、Q5-判、Q5-阴性；第七节 A10；第八节轨迹：按末段里落了几个写排开）。
+/// Q5 计数的一行：全枚举时是一个判过的态，按等价类判时是一类。恢复落到的版本、两个崩溃子列，与这一行的态按「末段里落了几个写」
+/// 排开的态数（键是落了几个写，值是态数；全枚举的一态只有一格、值是 1）。
+struct ConfirmedLossTallyRow<'row> {
+    /// 这一行罩几态（全枚举的一态是 1，等价类的一类是这一类的态数）。
+    states: u64,
+    consulted_root: Option<(InstanceGeneration, CheckpointTxg)>,
+    crash_judge: &'row CrashJudgeVerdict,
+    reference_filesystem: &'row ReferenceFileSystemVoteOfAState,
+    states_by_writes_landed_in_the_last_segment: BTreeMap<usize, u64>,
+}
+
+impl<'row> ConfirmedLossTallyRow<'row> {
+    fn of_a_judged_state(observation: &'row StateObservation, last_segment: &[usize]) -> Self {
+        let landed = last_segment
+            .iter()
+            .filter(|write_index| {
+                observation
+                    .landing_vector
+                    .as_bytes()
+                    .get(**write_index)
+                    .is_some_and(|landing| *landing != b'0')
+            })
+            .count();
+        Self {
+            states: 1,
+            consulted_root: observation.consulted_root,
+            crash_judge: &observation.crash_judge,
+            reference_filesystem: &observation.reference_filesystem,
+            states_by_writes_landed_in_the_last_segment: BTreeMap::from([(landed, 1)]),
+        }
+    }
+
+    fn of_a_class(class: &'row ConfirmedLossClass) -> Self {
+        Self {
+            states: class.states,
+            consulted_root: class.consulted_root,
+            crash_judge: &class.columns.crash_judge,
+            reference_filesystem: &class.columns.reference_filesystem,
+            states_by_writes_landed_in_the_last_segment: class
+                .states_by_writes_landed_in_the_last_segment
+                .clone(),
+        }
+    }
+}
+
+/// Q5 的计数行（第六节 Q5-有效、Q5-判、Q5-阴性；第七节 A10；第八节轨迹：按末段里落了几个写排开）。每一行按它罩的态数计
+/// （全枚举的一态计 1，等价类的一类计这一类的态数）；`derived_by` 接在 `name=q5` 那一行末尾（按等价类判时写 `derived_by=…`）。
 fn report_the_confirmed_loss(
     lines: &mut ResultLines,
     sampling_point: &str,
-    observations: &[StateObservation],
+    rows: &[ConfirmedLossTallyRow<'_>],
     confirmed_root: Option<(InstanceGeneration, CheckpointTxg)>,
-    last_segment: &[usize],
     (last_segment_writes, tearable, product): (usize, usize, u64),
+    derived_by: &str,
 ) {
     let Some(confirmed_root) = confirmed_root else {
         lines.result(&format!(
-            "name=q5 sampling_point={sampling_point} confirmed_root=none"
+            "name=q5 sampling_point={sampling_point} confirmed_root=none{derived_by}"
         ));
         return;
     };
-    let lost: Vec<&StateObservation> = observations
+    let lost: Vec<&ConfirmedLossTallyRow<'_>> = rows
         .iter()
-        .filter(|observation| {
-            lost_the_confirmed_version(observation.consulted_root, confirmed_root) == Some(true)
-        })
+        .filter(|row| lost_the_confirmed_version(row.consulted_root, confirmed_root) == Some(true))
         .collect();
-    let not_lost: Vec<&StateObservation> = observations
+    let not_lost: Vec<&ConfirmedLossTallyRow<'_>> = rows
         .iter()
-        .filter(|observation| {
-            lost_the_confirmed_version(observation.consulted_root, confirmed_root) == Some(false)
-        })
+        .filter(|row| lost_the_confirmed_version(row.consulted_root, confirmed_root) == Some(false))
         .collect();
-    let without_a_root = observations
+    fn states_of(
+        selected: &[&ConfirmedLossTallyRow<'_>],
+        predicate: &dyn Fn(&ConfirmedLossTallyRow<'_>) -> bool,
+    ) -> u64 {
+        selected
+            .iter()
+            .filter(|row| predicate(row))
+            .map(|row| row.states)
+            .sum()
+    }
+    fn reference_filesystem_red(row: &ConfirmedLossTallyRow<'_>) -> bool {
+        row.reference_filesystem.word() == "red"
+    }
+    fn model_red(row: &ConfirmedLossTallyRow<'_>) -> bool {
+        row.crash_judge.word() == "red"
+    }
+    fn both_green(row: &ConfirmedLossTallyRow<'_>) -> bool {
+        row.reference_filesystem.word() == "green" && row.crash_judge.word() == "green"
+    }
+    fn every_row(_: &ConfirmedLossTallyRow<'_>) -> bool {
+        true
+    }
+    let states: u64 = rows.iter().map(|row| row.states).sum();
+    let without_a_root: u64 = rows
         .iter()
-        .filter(|observation| observation.consulted_root.is_none())
-        .count();
-    let reference_filesystem_red =
-        |state: &StateObservation| state.reference_filesystem.word() == "red";
-    let model_red = |state: &StateObservation| state.crash_judge.word() == "red";
-    let both_green = |state: &StateObservation| {
-        state.reference_filesystem.word() == "green" && state.crash_judge.word() == "green"
-    };
+        .filter(|row| row.consulted_root.is_none())
+        .map(|row| row.states)
+        .sum();
     lines.result(&format!(
-        "name=q5 sampling_point={sampling_point} states={} last_segment_writes={last_segment_writes} last_segment_tearable_writes={tearable} product_of_the_choices={product} a10_states_equal_the_product={} lost_the_confirmed_version={} reference_filesystem_red={} model_crash_judge_red={} both_green={} reference_filesystem_abstained={} not_lost_states={} not_lost_o1_red={} states_without_a_root={without_a_root}",
-        observations.len(),
-        u64::try_from(observations.len()).expect("态数") == product,
-        lost.len(),
-        lost.iter().filter(|state| reference_filesystem_red(state)).count(),
-        lost.iter().filter(|state| model_red(state)).count(),
-        lost.iter().filter(|state| both_green(state)).count(),
-        lost.iter().filter(|state| state.reference_filesystem.word() == "abstained").count(),
-        not_lost.len(),
-        not_lost.iter().filter(|state| reference_filesystem_red(state) || model_red(state)).count(),
+        "name=q5 sampling_point={sampling_point} states={states} last_segment_writes={last_segment_writes} last_segment_tearable_writes={tearable} product_of_the_choices={product} a10_states_equal_the_product={} lost_the_confirmed_version={} reference_filesystem_red={} model_crash_judge_red={} both_green={} reference_filesystem_abstained={} not_lost_states={} not_lost_o1_red={} states_without_a_root={without_a_root}{derived_by}",
+        states == product,
+        states_of(&lost, &every_row),
+        states_of(&lost, &reference_filesystem_red),
+        states_of(&lost, &model_red),
+        states_of(&lost, &both_green),
+        states_of(&lost, &|row| row.reference_filesystem.word() == "abstained"),
+        states_of(&not_lost, &every_row),
+        states_of(&not_lost, &|row| reference_filesystem_red(row) || model_red(row)),
     ));
-    let persisted_in_the_last_segment = |observation: &StateObservation| -> usize {
-        last_segment
-            .iter()
-            .filter(|write_index| {
-                observation
-                    .landing_vector
-                    .as_bytes()
-                    .get(**write_index)
-                    .is_some_and(|landing| *landing != b'0')
-            })
-            .count()
-    };
-    let mut by_landed: BTreeMap<usize, (usize, usize, usize)> = BTreeMap::new();
-    for observation in &lost {
-        let entry = by_landed
-            .entry(persisted_in_the_last_segment(observation))
-            .or_insert((0, 0, 0));
-        entry.0 += 1;
-        entry.1 += usize::from(reference_filesystem_red(observation));
-        entry.2 += usize::from(model_red(observation));
-    }
-    for (landed, (states, reference_filesystem_red_states, model_red_states)) in by_landed {
+    let mut by_landed: BTreeMap<usize, (u64, u64, u64)> = BTreeMap::new();
+    for row in &lost {
+        for (landed, landed_states) in &row.states_by_writes_landed_in_the_last_segment {
+            let entry = by_landed.entry(*landed).or_insert((0, 0, 0));
+            entry.0 += landed_states;
+            if reference_filesystem_red(row) {
+                entry.1 += landed_states;
+            }
+            if model_red(row) {
+                entry.2 += landed_states;
+            }
+        }
+    }
+    for (landed, (lost_states, reference_filesystem_red_states, model_red_states)) in by_landed {
         lines.result(&format!(
-            "name=q5_trajectory sampling_point={sampling_point} writes_landed_in_the_last_segment={landed} lost_states={states} reference_filesystem_red={reference_filesystem_red_states} model_crash_judge_red={model_red_states}"
+            "name=q5_trajectory sampling_point={sampling_point} writes_landed_in_the_last_segment={landed} lost_states={lost_states} reference_filesystem_red={reference_filesystem_red_states} model_crash_judge_red={model_red_states}"
         ));
     }
 }
 
-/// `stream-constants` 模式：只驱动、只回比（S1 的流相关几条与状态数几条、与流无关的几条、Q5 造法那几样），不写导出、不判态。
-fn run_stream_constants(streams: &[StreamName]) -> i32 {
-    let mut lines = ResultLines { emitted: 0 };
-    lines.result("name=config segment=s1 mode=stream-constants");
-    let mut failed_checks = 0usize;
-    let mut checks = stream_independent_checks();
-    checks.extend(stream_independent_checks_of_the_first_rerun());
-    checks.extend(confirmed_loss_form_checks_against_the_crates());
-    for name in streams {
-        let stream = prepare_stream(*name, RunSwitches::default(), true);
+// ───── Q5 主取样点按等价类判（登记修订 2 第 2.1 节候选二；修订 3 定它） ─────
+//
+// 态集合与读法 7 原句相同（末段整写子集全枚举、可撕裂的写另加撕裂态）；每态的 O1 两个崩溃子列与 RefFS 一票不逐态算，由等价类的代表推出。
+// 两态等价 ⇔ 键里那几个写（第 2 次调用的写 K，加上类核对 ③ 细分移进来的第 3 次调用的写）取法相同，且留在类里的第 3 次调用的写（C）
+// 「至少落了一个」这一位相同。代表是「C 一个都没落」那一态；「C 至少落一个」那一类按代表推，另把「C 全落」那一态照原路径判一遍对照（类核对 ④）。
+
+/// 「等价类对拍」那一形截到第 3 次调用发出的前几个写之后（修订 2 第 2.1 节候选二「用例钉住等价关系」：候选三 k = 2，72 × 4 = 288 态）。
+const EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT: usize = 2;
+
+/// [`ObservationContext`] 借的几张表，由一条流现算（与 [`export_prepared_stream`] 里同一套算法；按等价类判与「等价类对拍」用）。
+struct ObservationTables {
+    tables: Layer0EnumerationTables,
+    facts: StateMasks,
+    torn_image_index_by_write: BTreeMap<usize, usize>,
+    reference_filesystem: ReferenceFileSystemStream,
+}
+
+impl ObservationTables {
+    fn of(stream: &PreparedStream) -> Self {
         let tables = layer0_enumeration_tables(
             &stream.base,
             &stream.writes,
             &stream.segments,
             &full_expansion,
         );
-        let full_states = tables.state_count;
-        let quick_tier_states = layer0_plan_state_count(
-            &stream.base,
+        let facts = StateMasks::of(&tables, &stream.segments);
+        let torn_image_index_by_write = tables
+            .torn_images
+            .iter()
+            .map(|row| (row.write_index, row.torn_image_index))
+            .collect();
+        let reference_filesystem = reference_filesystem_stream_of(
+            &stream.calls,
+            &stream.operations,
             &stream.writes,
-            &stream.segments,
-            &quick_tier_expansion,
+            &stream.stream_indexes,
         );
-        checks.extend(stream.checks.iter().cloned());
-        checks.extend(state_count_checks_against_the_crates(
-            *name,
-            &stream,
-            &tables,
-            flow_facts_state_count(&stream, &tables),
-            full_states,
-            quick_tier_states,
-        ));
-        lines.result(&format!(
-            "name=stream_constants stream={} ending={} calls={} writes={} segments={} reference_filesystem_segments={}",
-            name.directory_name(),
-            ending_text(&stream.ending),
-            stream.calls.len(),
-            stream.writes.len(),
-            stream.segments.len(),
-            segment_closing_stream_indexes(&stream.operations).len()
-        ));
+        Self {
+            tables,
+            facts,
+            torn_image_index_by_write,
+            reference_filesystem,
+        }
     }
-    for check in checks {
-        failed_checks += usize::from(!check.holds());
-        lines.result(&format!(
-            "name=s1 check={} local={} crates={} holds={}",
-            check.name,
+
+    fn context<'tables>(
+        &'tables self,
+        stream: &'tables PreparedStream,
+    ) -> ObservationContext<'tables> {
+        ObservationContext {
+            stream,
+            tables: &self.tables,
+            facts: &self.facts,
+            torn_image_index_by_write: &self.torn_image_index_by_write,
+            reference_filesystem: &self.reference_filesystem,
+            call_of_write: call_index_of_each_write(stream),
+        }
+    }
+}
+
+/// 末段的写按层 0 的混合进制排开（`crash.rs` 的 `Layer0StatePlan::assign_landings`）：段内序号 = Σ 数字 × 权，末段第一个写是最低位，
+/// 进制是它能取几态（整写 2、可撕裂的 3），数字 0 是「没落」、最大是「持久」（可撕裂的写 1 是撕裂）。末段之后没有段，
+/// 段内序号取满（每位取最大）就是最后那个全持久态，所以末段与全持久态合起来恰是 `first_ordinal..first_ordinal + 组合数`。
+struct LastSegmentDigits {
+    first_ordinal: u64,
+    write_indexes: Vec<usize>,
+    radixes: Vec<u64>,
+    weights: Vec<u64>,
+}
+
+impl LastSegmentDigits {
+    fn of(stream: &PreparedStream, tables: &Layer0EnumerationTables) -> Self {
+        let write_indexes = stream.segments.last().cloned().unwrap_or_default();
+        let first_ordinal = tables
+            .state_ranges_by_segment
+            .last()
+            .map_or(0, |state_range| state_range.start);
+        let radixes: Vec<u64> = write_indexes
+            .iter()
+            .map(|write_index| {
+                if tables.is_tearable_by_write[*write_index] {
+                    3
+                } else {
+                    2
+                }
+            })
+            .collect();
+        let mut weights = Vec::with_capacity(radixes.len());
+        let mut weight = 1u64;
+        for radix in &radixes {
+            weights.push(weight);
+            weight = weight
+                .checked_mul(*radix)
+                .expect("末段的组合数装得进 u64（层 0 的计划同样要求）");
+        }
+        assert_eq!(
+            first_ordinal + weight,
+            tables.state_count,
+            "末段是最后一段：末段的组合数加上它的起点，恰是整条流的态数（最后那个全持久态是段内序号取满的那一个）"
+        );
+        Self {
+            first_ordinal,
+            write_indexes,
+            radixes,
+            weights,
+        }
+    }
+
+    /// 末段第 `position` 个写在 `ordinal` 那一态上的数字。
+    fn digit_of(&self, ordinal: u64, position: usize) -> u64 {
+        ((ordinal - self.first_ordinal) / self.weights[position]) % self.radixes[position]
+    }
+
+    /// 这几个位置上的写各取几态、全部组合的个数。
+    fn product_of_the_radixes(&self, positions: &[usize]) -> u64 {
+        positions
+            .iter()
+            .try_fold(1u64, |product, position| {
+                product.checked_mul(self.radixes[*position])
+            })
+            .expect("末段的组合数装得进 u64")
+    }
+}
+
+/// 留在类里的第 3 次调用的写（C）在一态上落没落。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+enum InFlightWritesLanded {
+    NoneLanded,
+    AtLeastOneLanded,
+}
+
+impl InFlightWritesLanded {
+    fn word(self) -> &'static str {
+        match self {
+            Self::NoneLanded => "none",
+            Self::AtLeastOneLanded => "at_least_one",
+        }
+    }
+}
+
+/// 一态落在哪一类：键里那几个写的取法（按键里的位置排的混合进制序号）与 C 落没落。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+struct ConfirmedLossClassKey {
+    keyed_assignment: u64,
+    in_flight_writes_landed: InFlightWritesLanded,
+}
+
+/// 等价关系的分法：键里的写（K 加上细分移进来的 C 写）与留在类里的 C 写，都是末段里的位置（[`LastSegmentDigits`] 的下标），升序。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct ConfirmedLossPartition {
+    keyed_positions: Vec<usize>,
+    class_positions: Vec<usize>,
+}
+
+impl ConfirmedLossPartition {
+    /// 细分之前的起点分法：键 = 末段里不属于 `in_flight_call` 的写，类里 = 属于它的写（`call_of_write` 是 [`call_index_of_each_write`] 的交回）。
+    fn before_any_refinement(
+        call_of_write: &[Option<usize>],
+        digits: &LastSegmentDigits,
+        in_flight_call: usize,
+    ) -> Self {
+        let (class_positions, keyed_positions): (Vec<usize>, Vec<usize>) =
+            (0..digits.write_indexes.len()).partition(|position| {
+                call_of_write[digits.write_indexes[*position]] == Some(in_flight_call)
+            });
+        Self {
+            keyed_positions,
+            class_positions,
+        }
+    }
+
+    /// 键里那几个写有几种取法（每种取法两类：C 一个没落、C 至少落一个；C 一个都不剩时只有前一类）。
+    fn keyed_assignments(&self, digits: &LastSegmentDigits) -> u64 {
+        digits.product_of_the_radixes(&self.keyed_positions)
+    }
+
+    /// 键里第 `keyed_assignment` 种取法、C 一个都没落的那一态（这一种取法的代表）。
+    fn representative_ordinal(&self, digits: &LastSegmentDigits, keyed_assignment: u64) -> u64 {
+        let mut remaining = keyed_assignment;
+        let mut ordinal = digits.first_ordinal;
+        for position in &self.keyed_positions {
+            ordinal += (remaining % digits.radixes[*position]) * digits.weights[*position];
+            remaining /= digits.radixes[*position];
+        }
+        assert_eq!(
+            remaining, 0,
+            "取法序号 {keyed_assignment} 超出键里那几个写的组合数"
+        );
+        ordinal
+    }
+
+    /// 同一种取法、C 全落的那一态。
+    fn fully_landed_ordinal(&self, digits: &LastSegmentDigits, keyed_assignment: u64) -> u64 {
+        self.representative_ordinal(digits, keyed_assignment)
+            + self
+                .class_positions
+                .iter()
+                .map(|position| (digits.radixes[*position] - 1) * digits.weights[*position])
+                .sum::<u64>()
+    }
+
+    /// 一态落在哪一类。
+    fn class_of_ordinal(&self, digits: &LastSegmentDigits, ordinal: u64) -> ConfirmedLossClassKey {
+        let mut keyed_assignment = 0u64;
+        let mut place = 1u64;
+        for position in &self.keyed_positions {
+            keyed_assignment += digits.digit_of(ordinal, *position) * place;
+            place *= digits.radixes[*position];
+        }
+        let in_flight_writes_landed = if self
+            .class_positions
+            .iter()
+            .any(|position| digits.digit_of(ordinal, *position) != 0)
+        {
+            InFlightWritesLanded::AtLeastOneLanded
+        } else {
+            InFlightWritesLanded::NoneLanded
+        };
+        ConfirmedLossClassKey {
+            keyed_assignment,
+            in_flight_writes_landed,
+        }
+    }
+
+    /// 「C 至少落一个」那一类的态数：C 每个写的取法数之积，减去一个都没落的那一态（C 都只取两态时是 2 的 C 写数次方减一）。
+    fn states_with_the_in_flight_writes_landed(&self, digits: &LastSegmentDigits) -> u64 {
+        digits.product_of_the_radixes(&self.class_positions) - 1
+    }
+
+    /// 「C 至少落一个」那一类按落了几个 C 写排开的态数（下标 j 是落了几个，第 0 格是 0）：Π (1 + (进制 − 1)·x) 的各项系数去掉常数项，
+    /// C 都只取两态时第 j 格是 C(C 写数, j)。撕裂算落了（与 `q5_trajectory` 按落地串数「不是 0」同一口径）。
+    fn states_by_in_flight_writes_landed(&self, digits: &LastSegmentDigits) -> Vec<u64> {
+        let mut coefficients = vec![1u64];
+        for position in &self.class_positions {
+            let landed_choices = digits.radixes[*position] - 1;
+            let mut next = vec![0u64; coefficients.len() + 1];
+            for (landed, states) in coefficients.iter().enumerate() {
+                next[landed] += states;
+                next[landed + 1] += states * landed_choices;
+            }
+            coefficients = next;
+        }
+        coefficients[0] = 0;
+        coefficients
+    }
+
+    /// 要照原路径判的态数：每种取法的代表，加上 C 还剩写时同一取法的「C 全落」那一态。
+    fn judged_state_count(&self, digits: &LastSegmentDigits) -> u64 {
+        let per_assignment = if self.class_positions.is_empty() {
+            1
+        } else {
+            2
+        };
+        self.keyed_assignments(digits)
+            .checked_mul(per_assignment)
+            .expect("要判的态数装得进 u64")
+    }
+}
+
+/// 恢复读过的一段字节（盘、起点、终点，都按字节；终点不含）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+struct RecordedReadRange {
+    device: DeviceIdentity,
+    start_in_bytes: u64,
+    end_in_bytes: u64,
+}
+
+/// 类核对 ③ 的读者：包在崩溃镜像外面、只转发、只记（做法照 `e161_crash_state_dedup_and_time_split.rs` 的 `RecordingReader`，
+/// 本装置另写一份、不引它）。答案一律原样交回。每一次读记它的字节段；journal 位置提示的答由环里落了的写定，记成读了整个环
+/// （宽了只会多细分，不会漏）；盘身份与盘的字节数只看基镜像，与哪几个写落了无关，不记。
+struct ReadRecordingPoolReader<'reader, 'image> {
+    image: &'reader CrashImage<'image>,
+    read_ranges: RefCell<Vec<RecordedReadRange>>,
+}
+
+impl PoolReader for ReadRecordingPoolReader<'_, '_> {
+    fn device_identities(&self) -> Vec<DeviceIdentity> {
+        PoolReader::device_identities(self.image)
+    }
+
+    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
+        PoolReader::device_size_in_bytes(self.image, device)
+    }
+
+    fn read(
+        &self,
+        device: DeviceIdentity,
+        offset: DeviceOffsetInBytes,
+        length: usize,
+    ) -> Option<Vec<u8>> {
+        self.read_ranges.borrow_mut().push(RecordedReadRange {
+            device,
+            start_in_bytes: offset.0,
+            end_in_bytes: offset.0 + u64::try_from(length).expect("读长装得进 u64"),
+        });
+        PoolReader::read(self.image, device, offset, length)
+    }
+
+    fn journal_record_offsets_hint(
+        &self,
+        device: DeviceIdentity,
+        ring_start: DeviceOffsetInBytes,
+        ring_bytes: u64,
+    ) -> Option<Vec<DeviceOffsetInBytes>> {
+        self.read_ranges.borrow_mut().push(RecordedReadRange {
+            device,
+            start_in_bytes: ring_start.0,
+            end_in_bytes: ring_start.0 + ring_bytes,
+        });
+        PoolReader::journal_record_offsets_hint(self.image, device, ring_start, ring_bytes)
+    }
+}
+
+/// 看 journal 那一遍恢复在一态上的结局：交回了报告，或 panic 了（panic 的消息）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+enum ConsultedRecovery {
+    Reported(Box<RecoveryReport>),
+    Panicked(String),
+}
+
+impl ConsultedRecovery {
+    /// 一个判过的态上记下的那一遍（只在留了整份报告的那一趟有）：恢复 panic 的态是 `Panicked`；没留报告的交 `None`。
+    fn of_the_observation(observation: &StateObservation) -> Option<Self> {
+        match (&observation.consulted_report, &observation.recovery_panic) {
+            (Some(report), None) => Some(Self::Reported(Box::new(report.clone()))),
+            (None, Some(message)) => Some(Self::Panicked(message.clone())),
+            (None, None) | (Some(_), Some(_)) => None,
+        }
+    }
+
+    fn effective_root(&self) -> Option<(InstanceGeneration, CheckpointTxg)> {
+        match self {
+            Self::Reported(report) => report.effective_root,
+            Self::Panicked(_) => None,
+        }
+    }
+}
+
+/// 在一态的崩溃镜像上经 [`ReadRecordingPoolReader`] 跑看 journal 的那一遍恢复：交回恢复的结局与它读过的字节段
+/// （panic 时读段记到 panic 那一刻为止：同样的读答同样的字节，恢复在同一处 panic）。
+fn consulted_recovery_recording_the_reads(
+    image: &CrashImage<'_>,
+) -> (ConsultedRecovery, Vec<RecordedReadRange>) {
+    let reader = ReadRecordingPoolReader {
+        image,
+        read_ranges: RefCell::new(Vec::new()),
+    };
+    let recovered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
+        singlefs_core::recovery::recover(&reader, singlefs_core::recovery::JournalPolicy::Consult)
+    }));
+    let recovery = match recovered {
+        Ok(report) => ConsultedRecovery::Reported(Box::new(report)),
+        Err(payload) => ConsultedRecovery::Panicked(panic_text(&*payload)),
+    };
+    (recovery, reader.read_ranges.into_inner())
+}
+
+/// 读过的字节段与一个写的字节段（同一块盘上）相交。
+fn recorded_reads_overlap_the_write(
+    read_ranges: &[RecordedReadRange],
+    write: &RetainedWrite,
+) -> bool {
+    let write_start = write.offset.0;
+    let write_end = write_start + write.length_in_bytes();
+    read_ranges.iter().any(|read| {
+        read.device == write.device
+            && read.start_in_bytes < write_end
+            && write_start < read.end_in_bytes
+    })
+}
+
+/// 类核对 ③ 细分到不动点之后的分法，与最后一遍在每个代表上的恢复（键是代表的序号）。
+struct ConfirmedLossClassification {
+    partition: ConfirmedLossPartition,
+    /// 原来的 C 写数（末段里属于第 3 次调用的写）。
+    in_flight_writes: usize,
+    refinements: usize,
+    /// 细分移进键的 C 写，末段里的位置。
+    moved_positions: Vec<usize>,
+    representative_recoveries: BTreeMap<u64, ConsultedRecovery>,
+    /// 停在上限时最后一遍还与读段相交的 C 写个数（到了不动点是 0）。
+    overlapping_positions_left: usize,
+    /// 细分之后要判的态数超过上限（停机 S13）。
+    stopped_at_the_state_limit: bool,
+    /// 最后那个分法要照原路径判的态数（停在上限时是超过上限的那一个分法的）。
+    judged_states_planned: u64,
+}
+
+impl ConfirmedLossClassification {
+    /// 要照原路径判的态：每种取法的代表，加上 C 还剩写时同一取法的「C 全落」那一态。
+    fn judged_ordinals(&self, digits: &LastSegmentDigits) -> BTreeSet<u64> {
+        let mut judged = BTreeSet::new();
+        for keyed_assignment in 0..self.partition.keyed_assignments(digits) {
+            judged.insert(
+                self.partition
+                    .representative_ordinal(digits, keyed_assignment),
+            );
+            if !self.partition.class_positions.is_empty() {
+                judged.insert(
+                    self.partition
+                        .fully_landed_ordinal(digits, keyed_assignment),
+                );
+            }
+        }
+        judged
+    }
+}
+
+/// 分类并细分（类核对 ③）：起点是键 = 末段里不属于 `in_flight_call` 的写、类里 = 属于它的写；每一遍在每种取法的代表上经只转发、只记的读者
+/// 跑看 journal 的恢复，读段与类里某个写相交就把那个写移进键、重做，直到一遍里一个都不相交（不动点）。移进键之后要判的态数超过
+/// `state_limit` 就停（停机 S13）。迭代上界：每一遍不是到不动点就是至少移走一个写，最多 C 写数 + 1 遍。
+fn classify_the_last_segment(
+    context: &ObservationContext<'_>,
+    digits: &LastSegmentDigits,
+    in_flight_call: usize,
+    state_limit: u64,
+) -> ConfirmedLossClassification {
+    let stream = context.stream;
+    let partition = ConfirmedLossPartition::before_any_refinement(
+        &context.call_of_write,
+        digits,
+        in_flight_call,
+    );
+    let mut classification = ConfirmedLossClassification {
+        in_flight_writes: partition.class_positions.len(),
+        partition,
+        refinements: 0,
+        moved_positions: Vec::new(),
+        representative_recoveries: BTreeMap::new(),
+        overlapping_positions_left: 0,
+        stopped_at_the_state_limit: false,
+        judged_states_planned: 0,
+    };
+    loop {
+        classification.judged_states_planned = classification.partition.judged_state_count(digits);
+        if classification.judged_states_planned > state_limit {
+            classification.stopped_at_the_state_limit = true;
+            return classification;
+        }
+        let mut overlapping: BTreeSet<usize> = BTreeSet::new();
+        let mut recoveries = BTreeMap::new();
+        for keyed_assignment in 0..classification.partition.keyed_assignments(digits) {
+            let representative = classification
+                .partition
+                .representative_ordinal(digits, keyed_assignment);
+            let image = CrashImage {
+                base: &stream.base,
+                writes: &context.tables.enumerated_writes,
+                persisted: context.facts.persisted_writes_of_state(representative),
+            };
+            let (recovery, read_ranges) = consulted_recovery_recording_the_reads(&image);
+            for position in &classification.partition.class_positions {
+                let write = &stream.writes[digits.write_indexes[*position]];
+                if recorded_reads_overlap_the_write(&read_ranges, write) {
+                    overlapping.insert(*position);
+                }
+            }
+            recoveries.insert(representative, recovery);
+        }
+        classification.representative_recoveries = recoveries;
+        if overlapping.is_empty() {
+            return classification;
+        }
+        let mut refined = classification.partition.clone();
+        refined
+            .class_positions
+            .retain(|position| !overlapping.contains(position));
+        refined.keyed_positions.extend(overlapping.iter().copied());
+        refined.keyed_positions.sort_unstable();
+        if refined.judged_state_count(digits) > state_limit {
+            classification.judged_states_planned = refined.judged_state_count(digits);
+            classification.overlapping_positions_left = overlapping.len();
+            classification.stopped_at_the_state_limit = true;
+            return classification;
+        }
+        classification.partition = refined;
+        classification.moved_positions.extend(overlapping);
+        classification.moved_positions.sort_unstable();
+        classification.refinements += 1;
+    }
+}
+
+/// O1 两个崩溃子列与 RefFS 一票，只由修订 2 第 2.1 节「为什么这样分得开」列的四样输入算：看 journal 那一遍的恢复（读回由它出）、
+/// 盘上已持久的最新根、触达的最后一次调用、段号（RefFS 的崩溃时刻）。等价类按它推；类核对 ① 拿它在判过的每一态上重算，
+/// 与 [`observation_of_a_judged_state`] 那一路比（那一路多吃一样输入就对不上）。恢复 panic 的两列都记恢复 panic（读法 5）。
+fn crash_columns_of_the_four_inputs(
+    context: &ObservationContext<'_>,
+    recovery: &ConsultedRecovery,
+    newest_persisted_root_key: Option<ModelRootKey>,
+    touched_call: Option<usize>,
+    segment_index: usize,
+) -> (CrashJudgeVerdict, ReferenceFileSystemVoteOfAState) {
+    match recovery {
+        ConsultedRecovery::Panicked(_) => (
+            CrashJudgeVerdict::RecoveryPanicked,
+            ReferenceFileSystemVoteOfAState::RecoveryPanicked,
+        ),
+        ConsultedRecovery::Reported(report) => {
+            let read_back = observed_read_back_after_a_crash(report);
+            let crash_judge =
+                if crash_judge_is_applicable(touched_call, context.stream.model_stopped_at_call) {
+                    crash_judge_verdict(
+                        &context.stream.committed_versions,
+                        newest_persisted_root_key,
+                        &read_back,
+                    )
+                } else {
+                    CrashJudgeVerdict::NotApplicable
+                };
+            let reference_filesystem = reference_filesystem_vote_of_a_state(
+                context.reference_filesystem,
+                context.reference_filesystem_moment_of(segment_index),
+                &read_back,
+            );
+            (crash_judge, reference_filesystem)
+        }
+    }
+}
+
+/// 一态上「盘上已持久的最新根」（[`crash_judge_of_the_state`] 同一种数法：只看写表里的原写）。
+fn newest_persisted_root_key_of_the_state(
+    context: &ObservationContext<'_>,
+    ordinal: u64,
+) -> Option<ModelRootKey> {
+    let stream = context.stream;
+    let persisted = context.facts.persisted_writes_of_state(ordinal);
+    newest_persisted_root(&stream.writes, &persisted[..stream.writes.len()])
+        .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg))
+}
+
+/// 一类的触达调用：「C 一个没落」那一类是代表自己的；「C 至少落一个」那一类再算上类里那几个写的调用。
+fn touched_call_of_the_class(
+    representative_touched_call: Option<usize>,
+    in_flight_writes_landed: InFlightWritesLanded,
+    call_of_the_class_writes: Option<usize>,
+) -> Option<usize> {
+    match in_flight_writes_landed {
+        InFlightWritesLanded::NoneLanded => representative_touched_call,
+        InFlightWritesLanded::AtLeastOneLanded => {
+            representative_touched_call.max(call_of_the_class_writes)
+        }
+    }
+}
+
+/// 一类的恢复与两个崩溃子列（「C 一个没落」那一类就是代表照原路径判出的；「C 至少落一个」那一类由代表推出）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+struct ClassColumns {
+    recovery: ConsultedRecovery,
+    crash_judge: CrashJudgeVerdict,
+    reference_filesystem: ReferenceFileSystemVoteOfAState,
+}
+
+impl ClassColumns {
+    fn of_the_observation(observation: &StateObservation) -> Option<Self> {
+        ConsultedRecovery::of_the_observation(observation).map(|recovery| Self {
+            recovery,
+            crash_judge: observation.crash_judge.clone(),
+            reference_filesystem: observation.reference_filesystem.clone(),
+        })
+    }
+}
+
+/// 一类（`name=q5_class` 一行，Q5 计数的一行）。
+struct ConfirmedLossClass {
+    key: ConfirmedLossClassKey,
+    /// 键里那几个写在代表上的落地串（字符同 `states.tsv`：0 没落、1 撕裂、2 持久），按末段里的位置排。
+    keyed_landing: String,
+    states: u64,
+    states_by_writes_landed_in_the_last_segment: BTreeMap<usize, u64>,
+    columns: ClassColumns,
+    consulted_root: Option<(InstanceGeneration, CheckpointTxg)>,
+}
+
+/// 类核对的各项计数（`name=q5_equivalence` 一行；任一项不是 0 走停机 S13）。
+#[derive(Clone, Debug, Default, PartialEq, Eq)]
+struct EquivalenceCheckCounts {
+    /// ①：判过的态上，两个崩溃子列与 RefFS 一票照原路径算的，与只由四样输入重算的对不上的态数。
+    input_list_mismatches: usize,
+    /// ②：C 的写经 `geometry.classify` 是根槽写、系统配置槽写或 journal 记录写的个数。
+    in_flight_writes_of_a_forbidden_kind: usize,
+    /// ④ 前一半：代表上经读者跑的那一遍恢复，与层 0 判同一态交的 `judgement.consulted` 不同的代表数。
+    recording_report_mismatches: usize,
+    /// ④ 后一半：「C 全落」那一态照原路径判的恢复、两个崩溃子列与 RefFS 一票，与它所在的类推出的不同的态数。
+    derived_class_mismatches: usize,
+    /// 代表与「C 全落」那一态的落地串不是分法说的那样（代表上类里的写不全是 0、「C 全落」上不全是持久，或代表不在它那一类）的态数。
+    class_landing_mismatches: usize,
+    /// C 的写经 `geometry.classify` 各是哪一种（逐个列，按种类数）。
+    in_flight_write_kinds: BTreeMap<&'static str, usize>,
+}
+
+/// 各类与类核对 ①、②、④ 的计数（③ 在 [`classify_the_last_segment`] 里）。
+struct ConfirmedLossClassEvaluation {
+    classes: BTreeMap<ConfirmedLossClassKey, ConfirmedLossClass>,
+    checks: EquivalenceCheckCounts,
+}
+
+/// 由判过的态（每种取法的代表与「C 全落」那一态，都照原路径判、留了整份恢复报告）推出各类，并做类核对 ①、②、④。
+///
+/// # Panics
+/// 判过的态里缺某种取法的代表或「C 全落」那一态（调用方没照 [`ConfirmedLossClassification::judged_ordinals`] 判）。
+fn evaluate_the_classes(
+    context: &ObservationContext<'_>,
+    digits: &LastSegmentDigits,
+    classification: &ConfirmedLossClassification,
+    observations: &[StateObservation],
+    in_flight_call: usize,
+) -> ConfirmedLossClassEvaluation {
+    let stream = context.stream;
+    let partition = &classification.partition;
+    let observation_of_ordinal: BTreeMap<u64, &StateObservation> = observations
+        .iter()
+        .map(|observation| (observation.ordinal, observation))
+        .collect();
+    let mut checks = EquivalenceCheckCounts::default();
+    let geometry = common::geometry();
+    for write_index in digits.write_indexes.iter().copied() {
+        if context.call_of_write[write_index] != Some(in_flight_call) {
+            continue;
+        }
+        let kind =
+            geometry.classify(&stream.operations[stream.stream_indexes[write_index]].operation);
+        *checks.in_flight_write_kinds.entry(kind.name()).or_insert(0) += 1;
+        match kind {
+            StepKind::RootRecordFua
+            | StepKind::SystemConfigurationSlot
+            | StepKind::JournalRecord => {
+                checks.in_flight_writes_of_a_forbidden_kind += 1;
+            }
+            StepKind::ZeroFill | StepKind::UnitWrite | StepKind::Barrier => {}
+        }
+    }
+    for observation in observations {
+        let Some(recovery) = ConsultedRecovery::of_the_observation(observation) else {
+            continue;
+        };
+        let recomputed = crash_columns_of_the_four_inputs(
+            context,
+            &recovery,
+            newest_persisted_root_key_of_the_state(context, observation.ordinal),
+            observation.touched_call,
+            observation.segment_index,
+        );
+        if recomputed
+            != (
+                observation.crash_judge.clone(),
+                observation.reference_filesystem.clone(),
+            )
+        {
+            checks.input_list_mismatches += 1;
+        }
+    }
+    let call_of_the_class_writes = partition
+        .class_positions
+        .iter()
+        .filter_map(|position| context.call_of_write[digits.write_indexes[*position]])
+        .max();
+    let keyed_landed_and_landing = |observation: &StateObservation| -> (usize, String) {
+        let keyed_landing: String = partition
+            .keyed_positions
+            .iter()
+            .map(|position| {
+                char::from(observation.landing_vector.as_bytes()[digits.write_indexes[*position]])
+            })
+            .collect();
+        let landed = keyed_landing
+            .chars()
+            .filter(|landing| *landing != '0')
+            .count();
+        (landed, keyed_landing)
+    };
+    let class_landings_are = |observation: &StateObservation, expected: char| -> bool {
+        partition.class_positions.iter().all(|position| {
+            char::from(observation.landing_vector.as_bytes()[digits.write_indexes[*position]])
+                == expected
+        })
+    };
+    let mut classes = BTreeMap::new();
+    for keyed_assignment in 0..partition.keyed_assignments(digits) {
+        let representative_ordinal = partition.representative_ordinal(digits, keyed_assignment);
+        let representative = observation_of_ordinal
+            .get(&representative_ordinal)
+            .copied()
+            .expect("每种取法的代表都照原路径判过（judged_ordinals）");
+        let none_key = ConfirmedLossClassKey {
+            keyed_assignment,
+            in_flight_writes_landed: InFlightWritesLanded::NoneLanded,
+        };
+        if partition.class_of_ordinal(digits, representative_ordinal) != none_key
+            || !class_landings_are(representative, '0')
+        {
+            checks.class_landing_mismatches += 1;
+        }
+        let recording_recovery = classification
+            .representative_recoveries
+            .get(&representative_ordinal)
+            .expect("类核对 ③ 的最后一遍在每个代表上都跑过");
+        let representative_columns = ClassColumns::of_the_observation(representative)
+            .expect("按等价类判的那一趟留了整份恢复报告");
+        if *recording_recovery != representative_columns.recovery {
+            checks.recording_report_mismatches += 1;
+        }
+        let (keyed_landed, keyed_landing) = keyed_landed_and_landing(representative);
+        classes.insert(
+            none_key,
+            ConfirmedLossClass {
+                key: none_key,
+                keyed_landing: keyed_landing.clone(),
+                states: 1,
+                states_by_writes_landed_in_the_last_segment: BTreeMap::from([(keyed_landed, 1)]),
+                consulted_root: representative.consulted_root,
+                columns: representative_columns,
+            },
+        );
+        if partition.class_positions.is_empty() {
+            continue;
+        }
+        let in_flight_writes_landed_in_the_class = InFlightWritesLanded::AtLeastOneLanded;
+        let derived_touched_call = touched_call_of_the_class(
+            representative.touched_call,
+            in_flight_writes_landed_in_the_class,
+            call_of_the_class_writes,
+        );
+        let (crash_judge, reference_filesystem) = crash_columns_of_the_four_inputs(
+            context,
+            recording_recovery,
+            newest_persisted_root_key_of_the_state(context, representative_ordinal),
+            derived_touched_call,
+            representative.segment_index,
+        );
+        let derived = ClassColumns {
+            recovery: recording_recovery.clone(),
+            crash_judge,
+            reference_filesystem,
+        };
+        let fully_landed_ordinal = partition.fully_landed_ordinal(digits, keyed_assignment);
+        let fully_landed = observation_of_ordinal
+            .get(&fully_landed_ordinal)
+            .copied()
+            .expect("每种取法的「C 全落」那一态都照原路径判过（judged_ordinals）");
+        let some_key = ConfirmedLossClassKey {
+            keyed_assignment,
+            in_flight_writes_landed: InFlightWritesLanded::AtLeastOneLanded,
+        };
+        let highest_digit_landing = '2';
+        if partition.class_of_ordinal(digits, fully_landed_ordinal) != some_key
+            || !class_landings_are(fully_landed, highest_digit_landing)
+        {
+            checks.class_landing_mismatches += 1;
+        }
+        if ClassColumns::of_the_observation(fully_landed).as_ref() != Some(&derived) {
+            checks.derived_class_mismatches += 1;
+        }
+        let states_by_writes_landed_in_the_last_segment = partition
+            .states_by_in_flight_writes_landed(digits)
+            .into_iter()
+            .enumerate()
+            .filter(|(_, states)| *states > 0)
+            .map(|(in_flight_landed, states)| (keyed_landed + in_flight_landed, states))
+            .collect();
+        classes.insert(
+            some_key,
+            ConfirmedLossClass {
+                key: some_key,
+                keyed_landing,
+                states: partition.states_with_the_in_flight_writes_landed(digits),
+                states_by_writes_landed_in_the_last_segment,
+                consulted_root: derived.recovery.effective_root(),
+                columns: derived,
+            },
+        );
+    }
+    ConfirmedLossClassEvaluation { classes, checks }
+}
+
+/// 类核对 ①–④ 任一不过，或细分到上限：停机 S13（修订 2 第 2.1 节）。
+fn equivalence_stops_at_s13(
+    classification: &ConfirmedLossClassification,
+    checks: &EquivalenceCheckCounts,
+) -> bool {
+    classification.stopped_at_the_state_limit
+        || classification.overlapping_positions_left > 0
+        || checks.input_list_mismatches > 0
+        || checks.in_flight_writes_of_a_forbidden_kind > 0
+        || checks.recording_report_mismatches > 0
+        || checks.derived_class_mismatches > 0
+        || checks.class_landing_mismatches > 0
+}
+
+/// `name=q5_equivalence` 那一行（与对拍那一行）的类核对字段：①–④ 各自的计数、细分次数与移进键的写、停没停在上限与 S13 的判定。
+/// 停机 S13 看的是前六个计数与 `stopped_at_the_state_limit`（细分次数不是：③ 不相交时细分是照登记做，细分到上限才停）。
+fn equivalence_check_fields(
+    classification: &ConfirmedLossClassification,
+    checks: &EquivalenceCheckCounts,
+) -> String {
+    format!(
+        "input_list_mismatches={} in_flight_writes_of_a_forbidden_kind={} read_overlaps_left={} recording_report_mismatches={} derived_class_mismatches={} class_landing_mismatches={} refinements={} moved_last_segment_positions={} stopped_at_the_state_limit={} s13={}",
+        checks.input_list_mismatches,
+        checks.in_flight_writes_of_a_forbidden_kind,
+        classification.overlapping_positions_left,
+        checks.recording_report_mismatches,
+        checks.derived_class_mismatches,
+        checks.class_landing_mismatches,
+        classification.refinements,
+        field_text(&format!("{:?}", classification.moved_positions)),
+        classification.stopped_at_the_state_limit,
+        equivalence_stops_at_s13(classification, checks)
+    )
+}
+
+/// `name=q5_partition` 那一行（与对拍那一行）的分法字段：C 的写数与经 `geometry.classify` 逐个列的种类（类核对 ②）、键里与类里的写
+/// （末段里的位置，A10 比对器按它现算各类态数）、要照原路径判的态数与上限。
+fn partition_fields(
+    classification: &ConfirmedLossClassification,
+    checks: &EquivalenceCheckCounts,
+) -> String {
+    let kinds: Vec<String> = checks
+        .in_flight_write_kinds
+        .iter()
+        .map(|(kind, count)| format!("{kind}:{count}"))
+        .collect();
+    format!(
+        "in_flight_writes={} in_flight_write_kinds={} keyed_last_segment_positions={} class_last_segment_positions={} judged_states_planned={} state_limit={CONFIRMED_LOSS_STATE_LIMIT}",
+        classification.in_flight_writes,
+        if kinds.is_empty() { "-".to_string() } else { kinds.join(",") },
+        field_text(&format!("{:?}", classification.partition.keyed_positions)),
+        field_text(&format!("{:?}", classification.partition.class_positions)),
+        classification.judged_states_planned
+    )
+}
+
+/// 「等价类对拍」（修订 2 第 2.1 节候选二「用例钉住等价关系」，每一跑也做一遍）：末段与全持久态全枚举、逐态照原路径判，再按等价类推，
+/// 逐态比恢复、两个崩溃子列与 RefFS 一票。
+struct EquivalencePairing {
+    states: u64,
+    product_of_the_choices: u64,
+    /// 各类态数之和（A10：必须等于末段每个写的取法数之积）。
+    class_states: u64,
+    classes: usize,
+    mismatching_states: usize,
+    classification: ConfirmedLossClassification,
+    checks: EquivalenceCheckCounts,
+    observations: Vec<StateObservation>,
+}
+
+impl EquivalencePairing {
+    /// 对拍有一态不同、各类态数之和不等于组合数，或类核对不过：停机 S13。
+    fn stops_at_s13(&self) -> bool {
+        self.mismatching_states > 0
+            || self.class_states != self.product_of_the_choices
+            || equivalence_stops_at_s13(&self.classification, &self.checks)
+    }
+}
+
+/// 在一条 Q5 那一形的流上做「等价类对拍」。
+fn equivalence_pairing_of(stream: &PreparedStream) -> EquivalencePairing {
+    let observation_tables = ObservationTables::of(stream);
+    let context = observation_tables.context(stream);
+    let digits = LastSegmentDigits::of(stream, &observation_tables.tables);
+    let product_of_the_choices =
+        digits.product_of_the_radixes(&(0..digits.write_indexes.len()).collect::<Vec<usize>>());
+    let classification = classify_the_last_segment(
+        &context,
+        &digits,
+        CONFIRMED_LOSS_IN_FLIGHT_CALL,
+        CONFIRMED_LOSS_STATE_LIMIT,
+    );
+    let every_state: BTreeSet<u64> =
+        (digits.first_ordinal..observation_tables.tables.state_count).collect();
+    let observations = observe_sampled_states(
+        &context,
+        &SampledOrdinals {
+            batches: [every_state, BTreeSet::new(), BTreeSet::new()],
+            torn_scans: Vec::new(),
+        },
+        RunSwitches::default(),
+        ConsultedReports::Kept,
+    );
+    if classification.stopped_at_the_state_limit {
+        return EquivalencePairing {
+            states: u64::try_from(observations.len()).expect("态数"),
+            product_of_the_choices,
+            class_states: 0,
+            classes: 0,
+            mismatching_states: 0,
+            classification,
+            checks: EquivalenceCheckCounts::default(),
+            observations,
+        };
+    }
+    let evaluation = evaluate_the_classes(
+        &context,
+        &digits,
+        &classification,
+        &observations,
+        CONFIRMED_LOSS_IN_FLIGHT_CALL,
+    );
+    let mismatching_states = observations
+        .iter()
+        .filter(|observation| {
+            let key = classification
+                .partition
+                .class_of_ordinal(&digits, observation.ordinal);
+            evaluation.classes.get(&key).map(|class| &class.columns)
+                != ClassColumns::of_the_observation(observation).as_ref()
+        })
+        .count();
+    EquivalencePairing {
+        states: u64::try_from(observations.len()).expect("态数"),
+        product_of_the_choices,
+        class_states: evaluation.classes.values().map(|class| class.states).sum(),
+        classes: evaluation.classes.len(),
+        mismatching_states,
+        classification,
+        checks: evaluation.checks,
+        observations,
+    }
+}
+
+/// `stream-constants` 模式：只驱动、只回比（S1 的流相关几条与状态数几条、与流无关的几条、Q5 造法那几样），不写导出、不判态。
+fn run_stream_constants(streams: &[StreamName]) -> i32 {
+    let mut lines = ResultLines { emitted: 0 };
+    lines.result("name=config segment=s1 mode=stream-constants");
+    let mut failed_checks = 0usize;
+    let mut checks = stream_independent_checks();
+    checks.extend(stream_independent_checks_of_the_first_rerun());
+    checks.extend(confirmed_loss_form_checks_against_the_crates());
+    for name in streams {
+        let stream = prepare_stream(*name, RunSwitches::default(), true);
+        let tables = layer0_enumeration_tables(
+            &stream.base,
+            &stream.writes,
+            &stream.segments,
+            &full_expansion,
+        );
+        let full_states = tables.state_count;
+        let quick_tier_states = layer0_plan_state_count(
+            &stream.base,
+            &stream.writes,
+            &stream.segments,
+            &quick_tier_expansion,
+        );
+        checks.extend(stream.checks.iter().cloned());
+        checks.extend(state_count_checks_against_the_crates(
+            *name,
+            &stream,
+            &tables,
+            flow_facts_state_count(&stream, &tables),
+            full_states,
+            quick_tier_states,
+        ));
+        lines.result(&format!(
+            "name=stream_constants stream={} ending={} calls={} writes={} segments={} reference_filesystem_segments={}",
+            name.directory_name(),
+            ending_text(&stream.ending),
+            stream.calls.len(),
+            stream.writes.len(),
+            stream.segments.len(),
+            segment_closing_stream_indexes(&stream.operations).len()
+        ));
+    }
+    for check in checks {
+        failed_checks += usize::from(!check.holds());
+        lines.result(&format!(
+            "name=s1 check={} local={} crates={} holds={}",
+            check.name,
             field_text(&check.local),
             field_text(&check.crates),
             check.holds()
@@ -8620,7 +9874,7 @@ mod tests {
     }
 
     /// 见证判据的两支：干净的池上两支都假、c_见证 是所选那一版末条记录的计数器；一块盘一槽系统配置撕裂（槽里一个字节翻一位、自证不过）记 (i)、
-    /// 不记 (ii)（V11：判不出那一支并进 (ii)，这里红）；每块盘两槽的 journal_tail 抬过所选那一版的末条（阳性对照「见证」的造法）记 (ii) 第一句。
+    /// 不记 (ii)；每块盘两槽的 journal_tail 抬过所选那一版的末条（阳性对照「见证」的造法）记 (ii) 第一句。
     #[test]
     fn a_torn_system_configuration_slot_is_the_undecidable_branch_and_a_raised_tail_is_the_counter_branch(
     ) {
@@ -8650,6 +9904,82 @@ mod tests {
         );
     }
 
+    /// 每块盘两槽的 journal_tail 抬过所选那一版的末条之后，再撕盘 1 的一槽：只记 (i)、不记 (ii)，另三槽抬过的 tail 还在、c_见证 不变
+    /// （登记修订 2 第 2.4 节；V11：判不出那一支并进 (ii)，三句的第一句为真，这里红）。
+    #[test]
+    fn a_torn_slot_beside_raised_tails_is_only_the_undecidable_branch() {
+        let (clean, selected) =
+            first_file_pool_and_its_selected_version("e171r1-unit-witness-torn-beside-raised");
+        let mut raised = clean.clone();
+        assert_eq!(
+            raise_the_witnessed_counter_past_the_selected_version(&mut raised, selected),
+            4
+        );
+        let raised_criterion = witness_criterion_on_the_crash_image(&raised, selected);
+        let mut torn = raised.clone();
+        let device = torn.devices.get_mut(&DeviceIdentity(1)).expect("盘 1");
+        let spacing = u64::from(common::parameters().geometry.fixed_structure_slot_spacing);
+        let mut slot_bytes = device.read(DeviceOffsetInBytes(spacing), 4096);
+        slot_bytes[100] ^= 1;
+        device.write(DeviceOffsetInBytes(spacing), &slot_bytes);
+        let torn_criterion = witness_criterion_on_the_crash_image(&torn, selected);
+        assert!(
+            torn_criterion.some_device_has_fewer_than_two_verified_slots,
+            "盘 1 撕了一槽，自证过的少于两槽：{torn_criterion:?}"
+        );
+        assert_eq!(
+            torn_criterion.counter_sentence, None,
+            "判不出那一支为真时计数那一支不判：{torn_criterion:?}"
+        );
+        assert_eq!(
+            torn_criterion.witnessed_counter, raised_criterion.witnessed_counter,
+            "另三槽抬过的 tail 还在，c_见证 与撕之前那份相同（取样点敏感）"
+        );
+    }
+
+    /// 流二第 3 次调用 panic（阳性对照 panic 的造法）：整条流照常导出、取样态照判，不退回「整条流不导出」
+    /// （登记修订 2 第 2.4 节；V4：panic 那一支退回不导出，这里红）。
+    #[test]
+    fn a_stream_stopped_by_a_panic_is_exported_and_its_sampled_states_are_judged() {
+        let switches = RunSwitches {
+            positive_control: Some(PositiveControl::PanicInTheCall),
+            disabled_oracle: None,
+            writable_mount_probe: false,
+        };
+        let stream = prepare_stream(StreamName::FixedScriptThroughEThenUnmount, switches, false);
+        let directory =
+            std::env::temp_dir().join(format!("e171r1-unit-panic-export-{}", std::process::id()));
+        let export = export_prepared_stream(
+            &stream,
+            &directory,
+            false,
+            switches,
+            StateSelection::RegisteredBatches { third_batch: false },
+            Instant::now(),
+            0,
+        );
+        std::fs::remove_dir_all(&directory)
+            .unwrap_or_else(|error| panic!("删 {}：{error}", directory.display()));
+        assert!(
+            matches!(
+                stream.ending,
+                StreamEnding::StoppedByAPanic { call_index: 3, .. }
+            ),
+            "{:?}",
+            stream.ending
+        );
+        assert_eq!(export.not_exported, None, "panic 截断的流照常导出");
+        assert!(!export.observations.is_empty(), "取样态照判");
+        assert_eq!(
+            stream.calls[..PANIC_CONTROL_CALL]
+                .iter()
+                .filter(|call| call.model.was_judged_by_the_comparison())
+                .count(),
+            1,
+            "panic 之前对拍判过的调用恰 1 次（第 2 次调用）"
+        );
+    }
+
     /// 阳性对照「见证」那一态上的探针：可写挂载拒、拒因是系统配置见证、第二截判不合法、没满足的条件是 C670 那一条（第七节 A4；
     /// V10：「没注入读故障」照「注入了」交，第二截判合法，这里红）。干净的池上可写挂载挂上。
     #[test]
@@ -8724,9 +10054,154 @@ mod tests {
         assert_eq!(reverse.len(), 5);
     }
 
+    /// 「等价类对拍」那一形的流（流二同一种造法，截到第 3 次调用发出的前 2 个写之后，72 × 4 = 288 态）。
+    fn equivalence_pairing_stream() -> PreparedStream {
+        confirmed_loss_prepared_stream(
+            second_stream_through_the_first_overwrite(RunSwitches::default()),
+            ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
+                EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT,
+            ),
+        )
+        .0
+    }
+
+    /// 等价关系（登记修订 2 第 2.1 节候选二「用例钉住等价关系」）：288 态那一形上全枚举逐态判与按等价类推，每态的恢复报告、两个崩溃子列
+    /// 与 RefFS 一票逐态相同，类核对 ①–④ 都过；模型照常那一遍与 `model_stopped_at_call` 设成第 3 次调用那一遍各做一遍（V17：键去掉
+    /// 「C 至少落一个」那一位，第二遍里「C 落了」的态模型崩溃判官不适用、代表上适用，这里红）。各类态数之和等于末段每个写的取法数之积，
+    /// 主取样点上（不判态，只分类）是 72 × 2²⁴（V19：「C 至少落一个」那一类的态数写成 2 的 C 写数次方，这里红）。
+    #[test]
+    fn equivalence_classes_reproduce_every_state_with_and_without_the_model_stopped_at_the_in_flight_call(
+    ) {
+        let mut stream = equivalence_pairing_stream();
+        let pairing = equivalence_pairing_of(&stream);
+        assert_eq!(pairing.states, 288, "72 × 2² 态全枚举");
+        assert_eq!(
+            pairing.class_states, pairing.product_of_the_choices,
+            "各类态数之和等于末段每个写的取法数之积（A10）"
+        );
+        assert_eq!(pairing.classes, 144, "72 种取法 × C 落没落");
+        assert_eq!(
+            pairing.classification.refinements, 0,
+            "C 的写不在代表的读段里"
+        );
+        assert_eq!(
+            pairing.checks,
+            EquivalenceCheckCounts {
+                input_list_mismatches: 0,
+                in_flight_writes_of_a_forbidden_kind: 0,
+                recording_report_mismatches: 0,
+                derived_class_mismatches: 0,
+                class_landing_mismatches: 0,
+                in_flight_write_kinds: BTreeMap::from([("unit_write", 2)]),
+            },
+            "类核对 ①、②、④ 都过"
+        );
+        assert_eq!(pairing.mismatching_states, 0, "模型照常：逐态相同");
+        assert!(!pairing.stops_at_s13());
+        stream.model_stopped_at_call = Some(CONFIRMED_LOSS_IN_FLIGHT_CALL);
+        let stopped = equivalence_pairing_of(&stream);
+        let not_applicable_states = stopped
+            .observations
+            .iter()
+            .filter(|observation| observation.crash_judge == CrashJudgeVerdict::NotApplicable)
+            .count();
+        assert_eq!(
+            not_applicable_states,
+            288 - 72,
+            "模型停在第 3 次调用：C 落了的态（每种取法 3 态）模型崩溃判官不适用，代表上适用（取样点敏感）"
+        );
+        assert_eq!(
+            stopped.mismatching_states, 0,
+            "模型停在第 3 次调用：逐态相同"
+        );
+        assert!(!stopped.stops_at_s13());
+        let (main, _) = confirmed_loss_prepared_stream(
+            second_stream_through_the_first_overwrite(RunSwitches::default()),
+            ConfirmedLossTruncation::BeforeTheInFlightBarrier,
+        );
+        let main_tables = ObservationTables::of(&main);
+        let main_context = main_tables.context(&main);
+        let main_digits = LastSegmentDigits::of(&main, &main_tables.tables);
+        let (class_positions, keyed_positions): (Vec<usize>, Vec<usize>) =
+            (0..main_digits.write_indexes.len()).partition(|position| {
+                main_context.call_of_write[main_digits.write_indexes[*position]]
+                    == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL)
+            });
+        let main_partition = ConfirmedLossPartition {
+            keyed_positions,
+            class_positions,
+        };
+        assert_eq!(
+            main_partition.keyed_assignments(&main_digits)
+                * (1 + main_partition.states_with_the_in_flight_writes_landed(&main_digits)),
+            LOCAL_CONFIRMED_LOSS_STATES,
+            "主取样点各类态数之和是 72 × 2²⁴"
+        );
+    }
+
+    /// 类核对 ③ 细分（登记修订 2 第 2.1 节；V18：③ 恒判不相交，这里红）：288 态那一形上把第 3 次调用的第 1 个写挪到第 2 次调用第一个
+    /// journal 记录写的同一个落点（手造写流），代表上的恢复读到它的字节段，那个写移进键（细分一次）；那个写落没落，恢复有不同的态
+    /// （取样点敏感）；细分之后按等价类推的与全枚举逐态相同。
+    #[test]
+    fn an_in_flight_write_over_a_journal_record_of_the_confirmed_call_is_moved_into_the_key() {
+        let mut stream = equivalence_pairing_stream();
+        let call_of_write = call_index_of_each_write(&stream);
+        let last_segment = stream.segments.last().cloned().expect("末段");
+        let journal_record_write = last_segment
+            .iter()
+            .copied()
+            .find(|write_index| {
+                call_of_write[*write_index] == Some(CONFIRMED_LOSS_CONFIRMED_CALL)
+                    && stream.writes[*write_index].kind == StepKind::JournalRecord
+            })
+            .expect("第 2 次调用在末段里写 journal 记录");
+        let first_in_flight_position = last_segment
+            .iter()
+            .position(|write_index| {
+                call_of_write[*write_index] == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL)
+            })
+            .expect("末段里有第 3 次调用的写");
+        let first_in_flight_write = last_segment[first_in_flight_position];
+        stream.writes[first_in_flight_write].device = stream.writes[journal_record_write].device;
+        stream.writes[first_in_flight_write].offset = stream.writes[journal_record_write].offset;
+        let pairing = equivalence_pairing_of(&stream);
+        assert_eq!(pairing.states, 288);
+        assert_eq!(
+            pairing.classification.moved_positions,
+            vec![first_in_flight_position],
+            "压在 journal 记录上的那个写移进键"
+        );
+        assert_eq!(pairing.classification.refinements, 1);
+        let tables = ObservationTables::of(&stream);
+        let digits = LastSegmentDigits::of(&stream, &tables.tables);
+        let weight = digits.weights[first_in_flight_position];
+        let observation_of_ordinal: BTreeMap<u64, &StateObservation> = pairing
+            .observations
+            .iter()
+            .map(|observation| (observation.ordinal, observation))
+            .collect();
+        let differing_when_that_write_lands = pairing
+            .observations
+            .iter()
+            .filter(|observation| {
+                digits.digit_of(observation.ordinal, first_in_flight_position) == 1
+            })
+            .filter(|observation| {
+                observation_of_ordinal[&(observation.ordinal - weight)].consulted_report
+                    != observation.consulted_report
+            })
+            .count();
+        assert!(
+            differing_when_that_write_lands > 0,
+            "那个写落没落，有恢复不同的态"
+        );
+        assert_eq!(pairing.mismatching_states, 0, "细分之后逐态相同");
+        assert!(!pairing.stops_at_s13());
+    }
+
     /// 流三照读法 2 驱动：删第一个文件（意图一次、走空一步）、一次建一个 inode 到叶容器第一次变成两片，再多建一次
-    /// （V9：停在 K − 1 次，叶容器还是一片，这里红）。叶容器变成两片之前的每次调用都对拍过；变成两片的那一次胶水接不了
-    /// （模型今天只罩一片叶容器），记「模型接不了这份观测」、模型停在那里（停机 S5 的那一处来源）。
+    /// （V9：停在 K − 1 次，叶容器还是一片，这里红）。取号、暖机之外的每次调用都对拍过、对得上，变成两片的那一次与之后那一次也是
+    /// （理想模型那件修理之后胶水按片记叶容器的角色，登记修订 2 第 2.2 节）：没有一次「模型接不了这份观测」，模型一次都不停。
     #[test]
     fn the_third_stream_stops_one_creation_after_the_inode_leaf_splits() {
         let mut live = LiveStream::start(
@@ -8749,19 +10224,14 @@ mod tests {
             .calls
             .iter()
             .all(|call| call.outcome == CallOutcome::Succeeded));
-        let split_call = live
-            .driver
-            .calls
-            .iter()
-            .position(|call| {
-                matches!(
-                    call.model,
-                    ModelVerdictOfCall::ModelCannotTakeTheObservation { .. }
-                )
-            })
-            .expect("跨叶那一次胶水接不了");
-        assert_eq!(live.driver.model_stopped_at_call, Some(split_call));
-        assert!(live.driver.calls[..split_call].iter().all(|call| {
+        assert!(!live.driver.calls.iter().any(|call| {
+            matches!(
+                call.model,
+                ModelVerdictOfCall::ModelCannotTakeTheObservation { .. }
+            )
+        }));
+        assert_eq!(live.driver.model_stopped_at_call, None);
+        assert!(live.driver.calls.iter().all(|call| {
             call.model == ModelVerdictOfCall::Agrees
                 || call.label == "acquire"
                 || call.label == "warm_up"
````

## 四、变异行（`crates/mutations.tsv` 里这一批点名的全部行：25-1 三行、E171R1 装置 28 行）

````text
E171R1 装置 O1-崩溃判官的结果丢掉（照抄原装置那一行，原登记第九节 X4）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    match crash_recovery_disagreement(committed_versions, newest_persisted_root_key, read_back) {	    match crash_recovery_disagreement(committed_versions, newest_persisted_root_key, read_back).filter(|_| false) {	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_read_back_of_a_version_the_model_never_committed_is_red_and_a_committed_one_is_green
E171R1 装置 O1-对拍判出的不同丢掉（照抄原装置那一行，原登记第九节 X5）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            answer(&self.model).and_then(|answer| self.model.judge_and_advance(&answer, &observed));	            answer(&self.model).and_then(|answer| self.model.judge_and_advance(&answer, &observed).or_else(|_| Ok(Default::default())));	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	an_implementation_refusal_the_model_does_not_allow_is_recorded_as_a_disagreement
E171R1 装置 O2 把 Violated 当 Holds（照抄原装置那一行，原登记第九节 X6）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    verdicts\n        .iter()\n        .filter(|(_, verdict)| match verdict {\n            InvariantVerdict::Violated(_) => true,	    verdicts\n        .iter()\n        .filter(|(_, verdict)| match verdict {\n            InvariantVerdict::Violated(_) => false,	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	only_a_violated_invariant_counts_as_a_red_pool_checker_verdict
E171R1 装置批 1 漏掉全持久态（照抄原装置那一行，原登记第九节 X13）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    first.insert(state_count - 1);	    let _ = state_count;	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	sampled_batches_are_disjoint_and_include_the_last_state_and_up_to_four_torn_states
E171R1 装置 O1-崩溃判官每个触达了的态都记不适用（照抄原装置那一行，原登记第九节 X14）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    !touched_call.is_some_and(|call| model_stopped_at_call.is_some_and(|stopped| call >= stopped))	    !touched_call.is_some_and(|_| true)	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_crash_judge_is_not_applicable_only_from_the_call_where_the_model_stopped
E171R1 装置 取号或暖机被拒不记成 O1-对拍的红（照抄原装置那一行，代码轮第三轮判决 K4-c 甲）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        self.calls[call_index].model = ModelVerdictOfCall::Disagrees {\n            aspect:\n                singlefs_harness::model::ModelDisagreementAspect::RefusedWhenModelRequiresSuccess	        let _discarded = ModelVerdictOfCall::Disagrees {\n            aspect:\n                singlefs_harness::model::ModelDisagreementAspect::RefusedWhenModelRequiresSuccess	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_refused_acquire_on_a_stream_without_faults_is_recorded_as_a_comparison_red_and_stops_the_model
E171R1 装置 关掉 O1 时被拒的取号照记红（照抄原装置那一行）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        if !self.model_is_enabled() {\n            return;\n        }\n        let call_index = self.calls.len() - 1;\n        self.calls[call_index].model = ModelVerdictOfCall::Disagrees {\n            aspect:\n	        let call_index = self.calls.len() - 1;\n        self.calls[call_index].model = ModelVerdictOfCall::Disagrees {\n            aspect:\n	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	with_the_model_switched_off_a_refused_acquire_is_not_a_comparison_red
E171R1 装置 取号 panic 也记成 O1-对拍的红（照抄原装置那一行；第 1 次重跑读法 5：panic 记实现自崩、不比）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        let Some(failure) = failure else {\n            return;\n        };\n        if !self.model_is_enabled() {	        let failure = failure.expect("panic 也记");\n        if !self.model_is_enabled() {	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_panic_in_the_acquire_is_an_implementation_crash_and_not_a_comparison_red
E171R1 装置 逐行 S5 把返回了错误的调用改成只数 panic 的调用（照抄原装置那一行）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        .filter(|call| matches!(call.outcome, CallOutcome::ReturnedError(_)))	        .filter(|call| matches!(call.outcome, CallOutcome::Panicked(_)))	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_row_s5_counts_the_errored_calls_left_unjudged_and_the_not_applicable_states_from_another_source
E171R1 装置 逐行 S5 把不适用的来源恒算合规（照抄原装置那一行，锚点换成读法 6 三处来源那一段）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        calls.get(stopped).is_some_and(|call| {	        calls.get(stopped).is_some_and(|_| true) || calls.get(stopped).is_some_and(|call| {	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_row_s5_counts_the_errored_calls_left_unjudged_and_the_not_applicable_states_from_another_source
E171R1 装置 逐行那一趟不判 S5（照抄原装置那一行）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            if export.ending.is_some() && switches.disabled_oracle.is_none() {	            if false && export.ending.is_some() && switches.disabled_oracle.is_none() {	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_row_run_judges_the_row_s5_and_the_baseline_run_judges_the_baseline_s5
E171R1 装置 关掉 O1 的逐行那一趟照判 S5（照抄原装置那一行）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            if export.ending.is_some() && switches.disabled_oracle.is_none() {	            if export.ending.is_some() {	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_row_run_judges_the_row_s5_and_the_baseline_run_judges_the_baseline_s5
E171R1 装置 逐行 S5 又把模型停下之后的错误调用数成「没比」（照抄原装置那一行）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        .filter(|call| matches!(call.model, ModelVerdictOfCall::NotModelled))	        .filter(|call| matches!(call.model, ModelVerdictOfCall::NotModelled | ModelVerdictOfCall::SkippedAfterTheModelStopped))	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_row_s5_counts_the_errored_calls_left_unjudged_and_the_not_applicable_states_from_another_source
E171R1 装置 V1 卸载那一步不调模型的卸载答（第 1 次重跑登记第九节 V1：模型停在卸载）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	                self.driver.judge(\n                    |model| model.answer_unmount(),\n                    ObservedOutcome::Succeeded(observed),\n                );	                let _ = observed;\n                self.driver.stop_the_model_at(self.driver.calls.len() - 1);	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_unmount_is_compared_with_the_model_and_its_control_turns_only_the_unmount_red
E171R1 装置 V2 交回值摘要取成常量（第九节 V2）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        whole: sha256_hexadecimal(text.as_bytes()),	        whole: sha256_hexadecimal(b"constant"),	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_return_value_digest_names_every_top_level_field_and_changes_with_one_field
E171R1 装置 V3 内存对象不记分配器那一项（第九节 V3）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        ("allocator", debug_text(allocator)),\n		-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_memory_objects_are_the_allocator_the_instance_the_warm_up_the_current_version_and_the_mount_output
E171R1 装置 V5 态上的恢复 panic 之后的态不再判（第九节 V5）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	                    remaining = panicking_ordinal + 1..remaining.end;	                    remaining = remaining.end..remaining.end;	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_state_whose_recovery_panics_is_noted_and_the_states_after_it_are_still_judged
E171R1 装置 V6 RefFS 的票恒为绿（第九节 V6）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    let mut crash_falls_in = None;	    let mut crash_falls_in = None;\n    if moment != ReferenceFileSystemCrashMoment::InsideStep(StepPosition::StartingPoint) {\n        return ReferenceFileSystemVoteOfAState::Green { allowed: String::new() };\n    }	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_crash_in_the_segment_the_publish_closes_falls_inside_that_publish
E171R1 装置 V7 RefFS 的崩溃时刻把关段那道屏障归到下一次调用（第九节 V7）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            stream_length_after_the_step: call.return_point,	            stream_length_after_the_step: call.first_operation,	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_crash_in_the_segment_the_publish_closes_falls_inside_that_publish
E171R1 装置 V8 在途的发布调用记「成」（第九节 V8）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        CallOutcome::ReturnedError(_)\n        | CallOutcome::Panicked(_)\n        | CallOutcome::InFlightAtTheTruncation => OperationOutcome::FailedAfterWriting,\n    };\n    match call.label {	        CallOutcome::InFlightAtTheTruncation => OperationOutcome::Succeeded,\n        CallOutcome::ReturnedError(_) | CallOutcome::Panicked(_) => OperationOutcome::FailedAfterWriting,\n    };\n    match call.label {	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	an_overwrite_in_flight_at_the_truncation_leaves_the_confirmed_version_allowed
E171R1 装置 V9 流三建 inode 停在叶容器变成两片的那一次（第九节 V9 的同形：少建最后那一次）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	                .is_some_and(|split| self.inode_creations == split + 1)	                .is_some_and(|split| self.inode_creations == split)	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_third_stream_stops_one_creation_after_the_inode_leaf_splits
E171R1 装置 V10 探针照「注入了读故障」交第二截（第九节 V10）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	const READ_FAULTS_OF_A_LAYER0_STATE: ReadFaultsInTheCrashState =\n    ReadFaultsInTheCrashState::NoneInjected;	const READ_FAULTS_OF_A_LAYER0_STATE: ReadFaultsInTheCrashState =\n    ReadFaultsInTheCrashState::Injected;	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_witness_control_is_refused_for_the_witness_and_judged_illegal_by_the_second_crop
E171R1 装置 V12 Q5 那一形不把根槽 FUA 改成普通写（第九节 V12）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	            changed.operation.kind = RecordedOperationKind::Write;\n		-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	the_confirmed_loss_form_drops_the_barriers_makes_the_root_write_plain_and_truncates
O1 跨叶容器（理想模型自身）：第二片晚一条才分出（一片按 234 条记，第 234 个 inode 的新记录与重写记成最左那片）	crates/singlefs-harness/src/model.rs	    let position_from_the_leftmost = records_before / INODE_LEAF_RECORDS;	    let position_from_the_leftmost = records_before / (INODE_LEAF_RECORDS + 1);	-p singlefs-harness --test model_compares_inode_creations_across_two_inode_leaf_containers	creating_inodes_one_at_a_time_past_the_first_leaf_container_is_judged_by_the_model_container_by_container
O1 跨叶容器（胶水）：实现交回的每片叶容器都当成最左那片	crates/singlefs-harness/src/model_comparison.rs	            ModelUnitRole::inode_leaf_container_at(u64::from(index.0))	            ModelUnitRole::inode_leaf_container_at(u64::from(index.0).min(0))	-p singlefs-harness --test model_compares_inode_creations_across_two_inode_leaf_containers	creating_inodes_one_at_a_time_past_the_first_leaf_container_is_judged_by_the_model_container_by_container
O1 跨叶容器（理想模型自身）：新记录落在第二片时最左那片也记成这次重写（分配代按第一片记）	crates/singlefs-harness/src/model.rs	            ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => &[\n                ModelUnitRole::InodeRoot,	            ModelPublishKind::NewInodeInALeafContainerRightOfTheLeftmost(_) => &[\n                ModelUnitRole::InodeLeaf,\n                ModelUnitRole::InodeRoot,	-p singlefs-harness --test model_compares_inode_creations_across_two_inode_leaf_containers	creating_inodes_one_at_a_time_past_the_first_leaf_container_is_judged_by_the_model_container_by_container
E171R1 装置 V4 panic 那一支退回「整条流不导出」（登记修订 2 第 2.4 节）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    export.ending = Some(stream.ending.clone());	    export.ending = Some(stream.ending.clone());\n    if matches!(stream.ending, StreamEnding::StoppedByAPanic { .. }) {\n        export.not_exported = Some("implementation_crashed".to_string());\n        return export;\n    }	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_stream_stopped_by_a_panic_is_exported_and_its_sampled_states_are_judged
E171R1 装置 V11 判不出那一支并进计数那一支（登记修订 2 第 2.4 节）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    let counter_sentence = if some_device_has_fewer_than_two_verified_slots\n        || witnessed_counter == 0	    let counter_sentence = if witnessed_counter == 0	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	a_torn_slot_beside_raised_tails_is_only_the_undecidable_branch
E171R1 装置 V17 Q5 等价类的键去掉「C 至少落一个」那一位（登记修订 2 第 2.1 节候选二）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        let in_flight_writes_landed_in_the_class = InFlightWritesLanded::AtLeastOneLanded;	        let in_flight_writes_landed_in_the_class = InFlightWritesLanded::NoneLanded;	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	equivalence_classes_reproduce_every_state_with_and_without_the_model_stopped_at_the_in_flight_call
E171R1 装置 V18 Q5 类核对 ③ 恒判不相交（登记修订 2 第 2.1 节候选二）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	    read_ranges.iter().any(|read| {\n        read.device == write.device	    read_ranges.iter().any(|_| false) && read_ranges.iter().any(|read| {\n        read.device == write.device	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	an_in_flight_write_over_a_journal_record_of_the_confirmed_call_is_moved_into_the_key
E171R1 装置 V19 Q5「C 至少落一个」那一类的态数写成 2 的 C 写数次方（登记修订 2 第 2.1 节候选二）	crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs	        digits.product_of_the_radixes(&self.class_positions) - 1	        digits.product_of_the_radixes(&self.class_positions)	-p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes	equivalence_classes_reproduce_every_state_with_and_without_the_model_stopped_at_the_in_flight_call
````
